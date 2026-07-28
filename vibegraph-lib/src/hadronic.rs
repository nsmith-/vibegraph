//! Leading-order cross-section integrands built from the compiled helicity
//! amplitude ([`crate::helas::eval`]), the run-card cut filter ([`crate::cuts`]),
//! and the VEGAS integrator ([`crate::vegas`]):
//!
//! * [`DrellYanIntegrand`] — proton beams (`lpp = 1`), the hadronic Drell–Yan
//!   `p p → e⁺ e⁻` process convolved with parton distributions ([`crate::pdf`])
//!   over a `(τ, y) × cosθ` map (documented below).
//! * [`FixedBeamIntegrand`] — fixed-energy partonic beams (`lpp = 0`), an
//!   arbitrary MG-validated process with no PDF convolution over any final-state
//!   multiplicity, sampled with flat RAMBO or — once
//!   [`FixedBeamIntegrand::use_multichannel`] has run — a resonance-aware
//!   per-diagram multichannel map that resolves Breit–Wigner peaks.
//!
//! The initial-state flux and spin×colour averaging factors are derived per
//! process from its incoming legs ([`initial_spin_color_average`]).
//!
//! # Drell–Yan master formula
//!
//! # Master formula
//!
//! ```text
//! σ = Σ_q ∫ dx₁ dx₂ dcosθ  [ f_q(x₁,μF) f_q̄(x₂,μF) + f_q̄(x₁,μF) f_q(x₂,μF) ]
//!       · dσ̂_qq̄→ℓℓ/dcosθ (ŝ = x₁x₂ s)  · Θ_cuts
//! ```
//!
//! At LO the two lepton legs are back-to-back, so a single partonic differential
//! cross section per Z/γ coupling class (up-type `q ∈ {u, c}`, down-type
//! `q ∈ {d, s}`; MadGraph's 4-flavor proton has no `b` and no gluon-initiated
//! diagram at this order) covers every flavor in the class. Two amplitude
//! evaluators are built, one per class; the PDF luminosity is summed over the
//! flavors of each class and over both proton orderings.
//!
//! # Change of variables
//!
//! VEGAS samples `(u₁, u₂, u₃) ∈ [0,1]³`, mapped to `(τ, y, cosθ)` by
//!
//! ```text
//! τ = τ_min^(1−u₁)   (ln τ uniform),   dτ/du₁ = τ · ln(1/τ_min)
//! y = (2u₂ − 1)·y_max,  y_max = ½ ln(1/τ),   dy/du₂ = 2·y_max
//! cosθ = 2u₃ − 1,                            d(cosθ)/du₃ = 2
//! ```
//!
//! where `τ = ŝ/s = x₁x₂` and `y = ½ ln(x₁/x₂)`, with `τ_min = ŝ_min / s` from
//! the cut hint ([`Cuts::shat_min`]) — so a dilepton mass window is a
//! one-dimensional bound on `τ` rather than a thin diagonal band in `(x₁, x₂)`.
//! Because the LHAPDF grid returns `x·f(x)` and `x₁x₂ = τ` matches the `dτ`
//! Jacobian factor, the `1/x₁x₂` in `f = (x·f)/x` cancels the `τ`: the
//! luminosity is built directly from `x·f` products and the phase-space weight
//! is the bare `ln(1/τ_min) · 2·y_max`.
//!
//! # Frames
//!
//! The matrix element is evaluated in the **partonic CM** with the beams along
//! ±z — the frame the helicity-pruned [`BoundAmplitude::eval_m2`] requires — where
//! `|M|²` is a Lorentz invariant. The cut filter operates in the **lab frame**,
//! so the final-state momenta are boosted along z by the partonic-system rapidity
//! `y = ½ ln(x₁/x₂)` before [`Cuts::pass`], whose rapidity/pT observables are not
//! z-boost invariant.

use std::cell::{Cell, RefCell};
use std::f64::consts::PI;

use rand::SeedableRng;
use thiserror::Error;

use crate::coupling::alphas::{AlphaSError, RunningAlphaS};
use crate::coupling::scales::{ClusterTopology, EventScales, ScaleChoice, ScaleError, ScaleEvent};
use crate::coupling::topology::cluster_topology;
use crate::cuts::{CutError, Cuts, ExternalLeg};
use crate::diagrams::diagram::Diagram;
use crate::diagrams::{
    generate_from_proc_card, parse_proc_card, DiagramError, DiagramSet, ParsingOptions,
};
use crate::helas::color::flow_tags::select_flow;
use crate::helas::eval::{AmplitudeEvaluator, BoundAmplitude, ScaleAwareAmplitude, ScratchSpace};
use crate::helas::repr::lorentz::LorentzVector;
use crate::pdf::PdfMember;
use crate::phasespace::{
    lips2_jacobian_u, AlphaAdaptation, Channel, Combiner, DiagramChannel, MultiChannel,
    PhaseSpaceMap, PhaseSpacePoint, RamboChannel, GEV2_TO_PB,
};
use crate::runcard::RunCard;
use crate::select::select_index;
use crate::ufo::{EvaluatedModel, UFOModel};
use crate::unweight::ChannelIntegrand;
use crate::vegas::{VegasGrid, VegasResult};

type V = LorentzVector<f64>;

/// VEGAS integration dimensions for the `(τ, y, cosθ)` map (§2.5).
pub const VEGAS_NDIM: usize = 3;
pub const VEGAS_NBINS: usize = 64;
pub const VEGAS_ALPHA: f64 = 1.5;

/// Grid-damping exponent used once a resonance-aware multichannel map is
/// installed, in place of the [`VEGAS_ALPHA`] Lepage recommends for a raw
/// integrand.
///
/// Lepage's `1.5` assumes the grid must *discover* the integrand's structure. A
/// converged multichannel map has already flattened the peaks it knows about, so
/// what remains in the unit hypercube is close to featureless and the per-bin `f²`
/// statistics are dominated by sampling noise. At `1.5` the refinement amplifies
/// that noise: the grid concentrates into a spurious bin, later iterations sample
/// a narrow region where the integrand is smooth and so report a small integral
/// with a small variance, and — since iterations are combined by `1/σ²` — those
/// confident, wrong iterations dominate the result. Measured on
/// `e+ e- > mu+ mu- ta+ ta-` (25 channels), `1.5` collapses one seed in five to 36%
/// of the banked sigma with `chi2/dof ≈ 580`, while `0.5` is stable across every
/// seed *and* halves the error — the grid still absorbing the residual structure
/// the channel maps do not cover.
pub const VEGAS_ALPHA_MAPPED: f64 = 0.5;

/// RNG substream index the multichannel α-adaptation survey draws on, kept distinct
/// from the VEGAS integration substreams so the survey and the integral neither
/// share nor correlate their sampling sequences.
const MULTICHANNEL_ADAPT_STREAM: u64 = 0xA1FA_5EED;

/// First `ChaCha8Rng` stream id of the per-channel integrations, offset by the
/// channel index. Channel `j` draws from stream `CHANNEL_STREAM_BASE + j`, so the
/// terms of the channel-split estimator sample structurally independent sequences
/// under one seed and each replays on its own.
const CHANNEL_STREAM_BASE: u64 = 0xC7A0_0000;

/// Floor on a channel's per-iteration evaluation count, so a channel whose
/// selection weight rounds to nothing still gets a grid it can refine and a term
/// it can estimate. Sample budget is otherwise split as `αⱼ · neval`.
const MIN_CHANNEL_NEVAL: usize = 512;

/// RNG seed and draw budget for the setup-time probe that resolves a dynamic scale
/// once before integration begins.
const SCALE_PROBE_SEED: u64 = 0x5CA1_E9E0;
const SCALE_PROBE_DRAWS: usize = 64;

#[derive(Debug, Error)]
pub enum HadronicError {
    #[error("diagram enumeration failed: {0}")]
    Diagram(#[from] DiagramError),
    #[error("cut compilation failed: {0}")]
    Cut(#[from] CutError),
    #[error("amplitude compilation failed: {0}")]
    Compile(String),
    #[error(
        "unexpected Drell–Yan flavor content: up-type {up:?}, down-type {down:?} \
         (expected up {{2,4}}, down {{1,3}}, no b, no gluon-initiated diagrams)"
    )]
    UnexpectedFlavors { up: Vec<i32>, down: Vec<i32> },
    #[error("no diagrams generated for the up-type or down-type Drell–Yan subprocess")]
    MissingClass,
    #[error("proc card generated no non-empty subprocess")]
    NoSubprocess,
    #[error(
        "fixed-energy beams require every subprocess to share the same external \
         particle content, but the generated subprocesses differ"
    )]
    InconsistentExternals,
    #[error("run card scale prescription: {0}")]
    Scale(#[from] ScaleError),
    #[error("running strong coupling: {0}")]
    AlphaS(#[from] AlphaSError),
    #[error("parameter card supplies no strong coupling to run from")]
    MissingAlphaS,
}

/// The run card's per-event scale prescription, bound to one process.
///
/// Holds the compiled [`ScaleChoice`], the [`ClusterTopology`] its clustering
/// branch consults, and the running coupling that turns `μR` into `αs(μR)`.
///
/// The coupling is constructed only on request. `coupling::alphas` refuses a
/// `pdlabel` whose `αs` MadGraph delegates to LHAPDF, and a matrix element with no
/// strong coupling in it has no reason to meet that refusal, so a caller whose
/// amplitudes report [`ScaleAwareAmplitude::depends_on_alpha_s`] false asks for no
/// coupling and runs regardless of the label.
#[derive(Clone, Debug)]
pub struct EventScaleSource {
    kind: ScaleSourceKind,
    alpha_s: Option<RunningAlphaS>,
}

#[derive(Clone, Copy, Debug)]
enum ScaleSourceKind {
    /// Every scale is a constant, so no event kinematics are read at all.
    Constant(EventScales),
    PerEvent {
        choice: ScaleChoice,
        topology: Option<ClusterTopology>,
    },
}

impl EventScaleSource {
    /// One constant scale on both beams and no running coupling — the prescription
    /// a caller that supplies `μF` directly is asking for.
    pub fn constant(mu: f64) -> Self {
        EventScaleSource {
            kind: ScaleSourceKind::Constant(EventScales {
                mu_r: mu,
                mu_f: [mu, mu],
            }),
            alpha_s: None,
        }
    }

    /// Compile a run card's prescription. `topology` is consulted only by the
    /// clustering branch; `needs_alpha_s` decides whether a running coupling is
    /// built at all.
    pub fn from_run_card(
        card: &RunCard,
        param_card_as: f64,
        topology: Option<ClusterTopology>,
        needs_alpha_s: bool,
    ) -> Result<Self, HadronicError> {
        let choice = ScaleChoice::from_run_card(card)?;
        let alpha_s = needs_alpha_s
            .then(|| RunningAlphaS::from_run_card(card, param_card_as))
            .transpose()?;
        let kind = if choice.is_fully_fixed() {
            // A fully fixed prescription returns the card's constants without
            // reading the event, so any event resolves it.
            ScaleSourceKind::Constant(choice.scales(&ScaleEvent {
                incoming: [[0.0; 4]; 2],
                outgoing: &[],
                topology: None,
            })?)
        } else {
            ScaleSourceKind::PerEvent { choice, topology }
        };
        Ok(EventScaleSource { kind, alpha_s })
    }

    /// The scales, when they are the same on every event.
    pub fn constant_scales(&self) -> Option<EventScales> {
        match self.kind {
            ScaleSourceKind::Constant(scales) => Some(scales),
            ScaleSourceKind::PerEvent { .. } => None,
        }
    }

    /// The running coupling, or `None` when none was asked for.
    pub fn running_alpha_s(&self) -> Option<&RunningAlphaS> {
        self.alpha_s.as_ref()
    }

    /// The topology the clustering branch consults.
    pub fn topology(&self) -> Option<ClusterTopology> {
        match self.kind {
            ScaleSourceKind::Constant(_) => None,
            ScaleSourceKind::PerEvent { topology, .. } => topology,
        }
    }

    /// The scales for one event, from lab-frame momenta.
    pub fn scales(
        &self,
        incoming: [[f64; 4]; 2],
        outgoing: &[[f64; 4]],
    ) -> Result<EventScales, ScaleError> {
        match self.kind {
            ScaleSourceKind::Constant(scales) => Ok(scales),
            ScaleSourceKind::PerEvent { choice, topology } => choice.scales(&ScaleEvent {
                incoming,
                outgoing,
                topology,
            }),
        }
    }
}

/// What installing a run card's per-event scale did to one integrand.
#[derive(Clone, Debug)]
pub struct RunningCouplingReport {
    /// Whether any subprocess's matrix element moves with the strong coupling. When
    /// false no running coupling was constructed and the amplitudes are left where
    /// they were bound.
    pub depends_on_alpha_s: bool,
    /// The topology derived for the clustering branch.
    pub topology: Option<ClusterTopology>,
    /// The scales, when the prescription resolves to constants — then no event
    /// kinematics are read and the coupling is applied once rather than per point.
    pub constant_scales: Option<EventScales>,
    /// The coupling at that constant renormalisation scale.
    pub constant_alpha_s: Option<f64>,
    /// The coupling the amplitudes were bound at.
    pub alpha_s_ref: Option<f64>,
    /// Why a subprocess must re-evaluate the whole model on each scale change
    /// instead of scaling its constant pools — roughly two orders of magnitude
    /// slower per event, so it is reported rather than absorbed.
    pub fallbacks: Vec<String>,
}

/// Components in the `[E, px, py, pz]` layout the scale prescription reads.
fn components(p: &V) -> [f64; 4] {
    [p.e(), p.px(), p.py(), p.pz()]
}

/// The Z/γ coupling classes of LO Drell–Yan, resolved from the `p p > e+ e-`
/// diagram enumeration rather than hand-coded flavor lists.
///
/// The representative subprocess of each class has its beams ordered
/// `[q, q̄, e⁺, e⁻]` (quark on beam 1); `up_flavors` / `down_flavors` are the
/// positive quark PDG codes whose PDF luminosities feed the class.
pub struct FlavorClasses {
    pub up_set: DiagramSet,
    pub down_set: DiagramSet,
    pub up_flavors: Vec<i32>,
    pub down_flavors: Vec<i32>,
}

/// Generate the `p p → e⁺ e⁻` subprocesses — the input [`dy_flavor_classes`]
/// partitions. Kept separate so the assembly is driven by generated diagram sets
/// (from the caller's proc card) rather than a re-parsed hard-coded string.
pub fn generate_dy_subprocesses(model: &UFOModel) -> Result<Vec<DiagramSet>, HadronicError> {
    let opts = ParsingOptions::default();
    let card = parse_proc_card("generate p p > e+ e-", &opts)?;
    Ok(generate_from_proc_card(&card, model)?)
}

/// Partition the quark-initiated subprocesses of a `p p → e⁺ e⁻` enumeration into
/// up-type and down-type Z/γ coupling classes, asserting the partition matches
/// the expected massless 4-flavor content. `sets` come from the caller's proc
/// card (e.g. via [`generate_dy_subprocesses`]).
pub fn dy_flavor_classes(
    sets: Vec<DiagramSet>,
    model: &UFOModel,
) -> Result<FlavorClasses, HadronicError> {
    let mut up_flavors: Vec<i32> = Vec::new();
    let mut down_flavors: Vec<i32> = Vec::new();
    let mut up_set: Option<DiagramSet> = None;
    let mut down_set: Option<DiagramSet> = None;

    for set in sets {
        if set.diagrams.is_empty() {
            continue;
        }
        // A Drell–Yan subprocess is a quark–antiquark pair annihilating to the
        // charged-lepton pair. Skip anything else (identical-flavor gg, etc.).
        let Some(pdgs) = incoming_pdgs(model, &set) else {
            continue;
        };
        let (a, b) = (pdgs[0], pdgs[1]);
        if a != -b || a == 0 {
            continue;
        }
        let quark = a.abs();
        // Canonical orientation: quark (positive PDG) on beam 1.
        let quark_first = a > 0;

        match quark {
            2 | 4 => {
                push_unique(&mut up_flavors, quark);
                if quark == 2 && quark_first && up_set.is_none() {
                    up_set = Some(set);
                }
            }
            1 | 3 => {
                push_unique(&mut down_flavors, quark);
                if quark == 1 && quark_first && down_set.is_none() {
                    down_set = Some(set);
                }
            }
            _ => {
                // b-quark or any other flavor with a surviving diagram violates
                // the massless 4-flavor assumption.
                up_flavors.push(-quark);
            }
        }
    }

    up_flavors.sort_unstable();
    down_flavors.sort_unstable();
    if up_flavors != [2, 4] || down_flavors != [1, 3] {
        return Err(HadronicError::UnexpectedFlavors {
            up: up_flavors,
            down: down_flavors,
        });
    }
    let (Some(up_set), Some(down_set)) = (up_set, down_set) else {
        return Err(HadronicError::MissingClass);
    };

    Ok(FlavorClasses {
        up_set,
        down_set,
        up_flavors,
        down_flavors,
    })
}

fn incoming_pdgs(model: &UFOModel, set: &DiagramSet) -> Option<[i32; 2]> {
    if set.particles_in.len() != 2 {
        return None;
    }
    let pdg = |name: &str| -> Option<i32> {
        let id = model.particle_id(name)?;
        Some(model.particle(id).pdg_code as i32)
    };
    Some([pdg(&set.particles_in[0])?, pdg(&set.particles_in[1])?])
}

fn push_unique(v: &mut Vec<i32>, x: i32) {
    if !v.contains(&x) {
        v.push(x);
    }
}

/// A ready-to-integrate Drell–Yan integrand over `(x₁, x₂, cosθ)`.
///
/// Borrows the two class amplitudes, the PDF member, and the compiled cuts; owns
/// per-class evaluation scratch (behind [`RefCell`] so the integrand is `Fn`).
pub struct DrellYanIntegrand<'a> {
    subs: [BoundSubprocess<'a>; 2],
    pdf: &'a PdfMember,
    cuts: &'a Cuts,
    up_flavors: Vec<i32>,
    down_flavors: Vec<i32>,
    /// Initial-state spin×colour averaging factor, derived from the incoming
    /// quark–antiquark pair (`1/(2·2·3·3) = 1/36`).
    spin_color_avg: f64,
    /// Total hadronic invariant `s = (E₁+E₂)²` (head-on beams).
    s_had: f64,
    /// The renormalisation scale the coupling follows and the *per-beam*
    /// factorisation scale the parton distributions are read at. A run card that
    /// fixes both leaves this a constant, and then no event kinematics are read.
    scales: EventScaleSource,
    /// Reused marshalling buffer for the lab-frame outgoing momenta the scale
    /// prescription reads.
    scale_buf: RefCell<Vec<[f64; 4]>>,
    /// Lower support of the logarithmic τ = ŝ/s map, `ŝ_min / s`.
    tau_min: f64,
    ln_inv_tau_min: f64,
}

/// Index of the up-type and down-type coupling class in [`DrellYanIntegrand`]'s
/// subprocess pair.
const DY_UP: usize = 0;
const DY_DOWN: usize = 1;

/// The Drell–Yan integrand decomposed into physical factors at one VEGAS point,
/// for the pointwise integrand oracle ([`DrellYanIntegrand::debug_factors`]).
#[derive(Clone, Copy, Debug)]
pub struct PointFactors {
    pub x1: f64,
    pub x2: f64,
    pub cos_theta: f64,
    pub sqrt_shat: f64,
    /// Up-type PDF luminosity `Σ_{q∈{u,c}} [x·f_q(x₁)·x·f_q̄(x₂) + (x₁↔x₂)]`.
    pub lum_up: f64,
    /// Down-type PDF luminosity `Σ_{q∈{d,s}} [...]`.
    pub lum_down: f64,
    /// Up-type color+helicity-summed |M|² (MadGraph MATRIX1 convention).
    pub m2_up: f64,
    /// Down-type color+helicity-summed |M|².
    pub m2_down: f64,
    /// Partonic prefactor `1/(2ŝ) · LIPS · spin·colour average` (flux, the
    /// 2-body LIPS Jacobian, and the process-derived initial-state average `1/36`).
    pub phat: f64,
    /// `(τ, y)` phase-space Jacobian (the `1/τ` already divided out).
    pub jac: f64,
    /// Whether the lab-frame momenta pass every compiled cut.
    pub pass: bool,
    /// The integrand value (`0` when the cut fails).
    pub value: f64,
}

/// A VEGAS point mapped to Drell–Yan kinematics via `(τ, y)`.
struct MappedPoint {
    x1: f64,
    x2: f64,
    sqrt_shat: f64,
    cos_theta: f64,
    /// Phase-space Jacobian `dτ dy / (du₁ du₂)` with the `1/τ` from
    /// `f = (x·f)/x` on both legs already divided out.
    jac: f64,
}

impl<'a> DrellYanIntegrand<'a> {
    /// Build the integrand.
    ///
    /// * `up` / `down` — bound amplitudes for the up-/down-type representative
    ///   subprocess, each with legs ordered `[q, q̄, e⁺, e⁻]`.
    /// * `pdf` — the PDF member evaluated at `μF`.
    /// * `cuts` — the compiled cut filter (its [`Cuts::shat_min`] sets `x_min`).
    /// * `up_flavors` / `down_flavors` — positive quark PDG codes per class.
    /// * `sqrt_s_had` — total collider energy `√s = E₁ + E₂`.
    /// * `mu_f` — a constant factorization scale on both beams, replaced by the
    ///   run card's prescription by
    ///   [`use_run_card_scales`](Self::use_run_card_scales).
    ///
    /// The initial-state spin×colour averaging factor is derived from the up-type
    /// class's incoming legs ([`initial_spin_color_average`]); both classes share
    /// the same quark–antiquark initial state.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        up: &'a BoundAmplitude<'a, f64>,
        down: &'a BoundAmplitude<'a, f64>,
        pdf: &'a PdfMember,
        cuts: &'a Cuts,
        up_flavors: Vec<i32>,
        down_flavors: Vec<i32>,
        sqrt_s_had: f64,
        mu_f: f64,
        spin_color_avg: f64,
    ) -> Self {
        let s_had = sqrt_s_had * sqrt_s_had;
        let tau_min = cuts.shat_min() / s_had;
        DrellYanIntegrand {
            subs: [BoundSubprocess::fixed(up), BoundSubprocess::fixed(down)],
            pdf,
            cuts,
            up_flavors,
            down_flavors,
            spin_color_avg,
            s_had,
            scales: EventScaleSource::constant(mu_f),
            scale_buf: RefCell::new(Vec::with_capacity(2)),
            tau_min,
            ln_inv_tau_min: (1.0 / tau_min).ln(),
        }
    }

    /// Take the renormalisation and per-beam factorisation scales from the run card
    /// instead of the constant `μF` [`new`](Self::new) was given.
    ///
    /// `diagrams` are the up-type class's, from which the
    /// [`ClusterTopology`](crate::coupling::scales::ClusterTopology) the `-1` scale
    /// consults is derived; both classes are the same colour-singlet annihilation
    /// off quark beams, so one declaration serves them.
    ///
    /// A run card that fixes both scales leaves the prescription a constant, and the
    /// integrand then reads the parton distributions at exactly the scale it did
    /// before — which is what keeps a cross section banked against such a card where
    /// it was.
    pub fn use_run_card_scales(
        &mut self,
        diagrams: &[Diagram],
        model: &UFOModel,
        evaluated: &EvaluatedModel,
        card: &RunCard,
    ) -> Result<RunningCouplingReport, HadronicError> {
        let awareness = make_subs_scale_aware(&mut self.subs, evaluated);
        // Unlike a fixed-beam run, the factorisation scale here has a consumer
        // whatever the matrix element is made of, so the prescription is compiled
        // even when nothing moves with the strong coupling.
        let source = compile_scale_source(
            self.subs[DY_UP].evaluator(),
            diagrams,
            model,
            evaluated,
            card,
            awareness.depends_on_alpha_s,
        )?;
        let report = constant_scale_report(&self.subs, Some(&source), awareness);
        self.scales = source;
        Ok(report)
    }

    /// Lower support `ŝ_min / s` of the logarithmic τ map.
    pub fn tau_min(&self) -> f64 {
        self.tau_min
    }

    /// Partonic prefactor `flux · LIPS · spin·colour average` for the 2→2
    /// integrand: `1/(2ŝ)` flux, the 2-body LIPS Jacobian for the `cosθ ↔ u₃`
    /// map, and the process-derived initial-state average.
    fn partonic_prefactor(&self, sqrt_shat: f64) -> f64 {
        let flux = 1.0 / (2.0 * sqrt_shat * sqrt_shat);
        flux * lips2_jacobian_u(sqrt_shat) * self.spin_color_avg
    }

    /// Map a VEGAS point `u ∈ [0,1]³` to Drell–Yan kinematics.
    ///
    /// `τ = ŝ/s` is sampled logarithmically over `[τ_min, 1]`
    /// (`τ = τ_min^(1−u₁)`, so `τ ≥ τ_min` makes `ŝ ≥ ŝ_min` automatic — no
    /// low-side rejection band). The parton rapidity `y = ½ ln(x₁/x₂)` is
    /// sampled uniformly over its kinematic range `|y| ≤ ½ ln(1/τ)`, which keeps
    /// `x₁, x₂ ∈ [τ, 1]`. With this change of variables the mass window is a
    /// one-dimensional bound on `τ` that VEGAS resolves far better than the thin
    /// diagonal band the direct `(x₁, x₂)` map produces. `cosθ = 2u₃ − 1`.
    fn map_point(&self, u: &[f64]) -> MappedPoint {
        let tau = self.tau_min.powf(1.0 - u[0]);
        let sqrt_tau = tau.sqrt();
        let y_max = -0.5 * tau.ln();
        let y = (2.0 * u[1] - 1.0) * y_max;
        // dτ/du₁ = τ ln(1/τ_min); dy/du₂ = 2 y_max. The `1/τ` from `f = (x·f)/x`
        // on both legs cancels the τ, leaving ln(1/τ_min)·2·y_max.
        let jac = self.ln_inv_tau_min * 2.0 * y_max;
        MappedPoint {
            x1: sqrt_tau * y.exp(),
            x2: sqrt_tau * (-y).exp(),
            sqrt_shat: (tau * self.s_had).sqrt(),
            cos_theta: 2.0 * u[2] - 1.0,
            jac,
        }
    }

    /// The integrand value at a VEGAS point `u ∈ [0,1]³`, in natural units
    /// (GeV⁻²); its VEGAS integral is the hadronic cross section. Points whose
    /// `ŝ` falls below the cut window or whose lab-frame momenta fail a cut
    /// contribute exactly zero.
    pub fn value(&self, u: &[f64]) -> f64 {
        let m = self.map_point(u);

        let Kinematics { cm, lab } =
            build_kinematics(m.sqrt_shat, m.cos_theta, m.x1, m.x2, self.s_had);
        if !self.cuts.pass(&lab) {
            return 0.0;
        }

        let scales = self.event_scales(&lab);
        let lum_up = self.luminosity(&self.up_flavors, m.x1, m.x2, scales.mu_f);
        let lum_down = self.luminosity(&self.down_flavors, m.x1, m.x2, scales.mu_f);
        if lum_up == 0.0 && lum_down == 0.0 {
            return 0.0;
        }

        let phat = self.partonic_prefactor(m.sqrt_shat);

        self.apply_scale(scales.mu_r);
        let m2_up = self.subs[DY_UP].eval_m2(&cm);
        let m2_down = self.subs[DY_DOWN].eval_m2(&cm);

        m.jac * phat * (m2_up * lum_up + m2_down * lum_down)
    }

    /// The scales at one event, from the lab-frame momenta the prescription is
    /// defined in. A run card that fixes them reads no kinematics at all.
    fn event_scales(&self, lab: &[V]) -> EventScales {
        if let Some(fixed) = self.scales.constant_scales() {
            return fixed;
        }
        let mut buf = self.scale_buf.borrow_mut();
        buf.clear();
        buf.extend(lab[2..].iter().map(components));
        self.scales
            .scales([components(&lab[0]), components(&lab[1])], &buf)
            .unwrap_or_else(|e| panic!("per-event scale on a sampled point: {e}"))
    }

    /// Move both class amplitudes to the coupling `mu_r` implies. A Drell–Yan
    /// matrix element carries no strong coupling, so no running coupling is built
    /// for it and this returns without touching the pools.
    fn apply_scale(&self, mu_r: f64) {
        let Some(running) = self.scales.running_alpha_s() else {
            return;
        };
        let alpha_s = running.eval(mu_r);
        for sub in &self.subs {
            sub.set_alpha_s(alpha_s);
        }
    }

    /// The integrand decomposed into its physical factors at a VEGAS point — the
    /// target of the pointwise integrand oracle. `value` equals
    /// `jac · phat · (m2_up·lum_up + m2_down·lum_down)` when `pass`, else `0`.
    pub fn debug_factors(&self, u: &[f64]) -> PointFactors {
        let m = self.map_point(u);
        let Kinematics { cm, lab } =
            build_kinematics(m.sqrt_shat, m.cos_theta, m.x1, m.x2, self.s_had);
        let pass = self.cuts.pass(&lab);
        let scales = self.event_scales(&lab);
        let lum_up = self.luminosity(&self.up_flavors, m.x1, m.x2, scales.mu_f);
        let lum_down = self.luminosity(&self.down_flavors, m.x1, m.x2, scales.mu_f);
        let phat = self.partonic_prefactor(m.sqrt_shat);
        self.apply_scale(scales.mu_r);
        let m2_up = self.subs[DY_UP].eval_m2(&cm);
        let m2_down = self.subs[DY_DOWN].eval_m2(&cm);
        let value = if pass {
            m.jac * phat * (m2_up * lum_up + m2_down * lum_down)
        } else {
            0.0
        };
        PointFactors {
            x1: m.x1,
            x2: m.x2,
            cos_theta: m.cos_theta,
            sqrt_shat: m.sqrt_shat,
            lum_up,
            lum_down,
            m2_up,
            m2_down,
            phat,
            jac: m.jac,
            pass,
            value,
        }
    }

    /// Summed PDF luminosity for one coupling class:
    /// `Σ_q [ (x·f_q)(x₁) (x·f_q̄)(x₂) + (x·f_q̄)(x₁) (x·f_q)(x₂) ]`.
    ///
    /// `mu_f` is per beam, in beam order: MadGraph carries `q2fact(1)` and
    /// `q2fact(2)` separately and its clustering can put them at different values,
    /// so `x₁` is read at beam 1's scale and `x₂` at beam 2's.
    fn luminosity(&self, flavors: &[i32], x1: f64, x2: f64, mu_f: [f64; 2]) -> f64 {
        let q2 = [mu_f[0] * mu_f[0], mu_f[1] * mu_f[1]];
        let mut acc = 0.0;
        for &q in flavors {
            let fq1 = self.pdf.xfx_q2(q, x1, q2[0]);
            let fq2 = self.pdf.xfx_q2(q, x2, q2[1]);
            let fqb1 = self.pdf.xfx_q2(-q, x1, q2[0]);
            let fqb2 = self.pdf.xfx_q2(-q, x2, q2[1]);
            acc += fq1 * fqb2 + fqb1 * fq2;
        }
        acc
    }

    /// Integrate the cross section with VEGAS, returning `(σ, Δσ)` in picobarns.
    ///
    /// `niter` adaptation iterations of `neval` points each; `seed` drives a
    /// reproducible RNG.
    pub fn integrate(&self, neval: usize, niter: usize, seed: u64) -> (f64, f64) {
        let result = self.integrate_raw(neval, niter, seed);
        (result.integral * GEV2_TO_PB, result.std_dev * GEV2_TO_PB)
    }

    /// The raw VEGAS result (natural units, GeV⁻²), grid discarded.
    pub fn integrate_raw(&self, neval: usize, niter: usize, seed: u64) -> VegasResult {
        self.adapt_grid(neval, niter, seed).1
    }

    /// Run VEGAS adaptation, returning the trained grid alongside the result —
    /// the primitive the `integrate` CLI command serializes into its artifact.
    pub fn adapt_grid(&self, neval: usize, niter: usize, seed: u64) -> (VegasGrid, VegasResult) {
        let mut grid = VegasGrid::new(VEGAS_NDIM, VEGAS_NBINS, VEGAS_ALPHA);
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        let result = grid.adapt(|u| self.value(u), neval, niter, &mut rng);
        (grid, result)
    }

    /// Differential cross section `dσ/dm_ℓℓ` (pb/GeV) on a uniform `m_ℓℓ` grid
    /// over `[m_lo, m_hi]`, by plain Monte-Carlo binning of the integrand — an
    /// informational shape diagnostic, not a gated quantity.
    pub fn dsigma_dmll(
        &self,
        m_lo: f64,
        m_hi: f64,
        nbins: usize,
        neval: usize,
        seed: u64,
    ) -> Vec<f64> {
        use rand::Rng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        let mut bins = vec![0.0f64; nbins];
        let bin_w = (m_hi - m_lo) / nbins as f64;
        // Each sample contributes value(u) (already the full GeV⁻² integrand) /
        // neval to the estimate of σ; distribute it into the m_ℓℓ bin and divide
        // by the bin width to form a density.
        for _ in 0..neval {
            let u = [
                rng.random::<f64>(),
                rng.random::<f64>(),
                rng.random::<f64>(),
            ];
            let mll = self.map_point(&u).sqrt_shat;
            if mll < m_lo || mll >= m_hi {
                continue;
            }
            let w = self.value(&u);
            if w == 0.0 {
                continue;
            }
            let bin = (((mll - m_lo) / bin_w) as usize).min(nbins - 1);
            bins[bin] += w;
        }
        for b in &mut bins {
            *b *= GEV2_TO_PB / (neval as f64 * bin_w);
        }
        bins
    }
}

struct Kinematics {
    /// Partonic-CM momenta `[q, q̄, e⁺, e⁻]` with beams along ±z.
    cm: Vec<V>,
    /// Lab-frame momenta `[q, q̄, e⁺, e⁻]` (CM boosted along z by the parton
    /// system rapidity), for the cut filter.
    lab: Vec<V>,
}

/// Build the partonic-CM and lab-frame external momenta for a back-to-back
/// dilepton event. `cos_theta` is the CM polar angle of `e⁺`; the azimuth is
/// fixed (the total cross section is azimuthally symmetric and the two leptons
/// stay diametrically opposed in φ under a z-boost, so `Δφ = π` never trips the
/// `drll` cut).
fn build_kinematics(sqrt_shat: f64, cos_theta: f64, x1: f64, x2: f64, s_had: f64) -> Kinematics {
    let half = sqrt_shat / 2.0;
    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();

    let q = V::new(half, 0.0, 0.0, half);
    let qbar = V::new(half, 0.0, 0.0, -half);
    let ep = V::new(half, half * sin_theta, 0.0, half * cos_theta);
    let em = V::new(half, -half * sin_theta, 0.0, -half * cos_theta);
    let cm = vec![q, qbar, ep, em];

    // Parton-system boost from CM to lab along z: β = (x₁−x₂)/(x₁+x₂).
    let beta = (x1 - x2) / (x1 + x2);
    let e_beam = s_had.sqrt() / 2.0;
    let q_lab = V::new(x1 * e_beam, 0.0, 0.0, x1 * e_beam);
    let qbar_lab = V::new(x2 * e_beam, 0.0, 0.0, -x2 * e_beam);
    let lab = vec![q_lab, qbar_lab, boost_z(ep, beta), boost_z(em, beta)];

    Kinematics { cm, lab }
}

/// Boost a four-momentum along z with velocity `beta` (CM → lab for `beta > 0`).
fn boost_z(p: V, beta: f64) -> V {
    let gamma = 1.0 / (1.0 - beta * beta).sqrt();
    let e = gamma * (p.e() + beta * p.pz());
    let pz = gamma * (p.pz() + beta * p.e());
    V::new(e, p.px(), p.py(), pz)
}

/// Build the four external legs `[q, q̄, e⁺, e⁻]` for a Drell–Yan subprocess,
/// for [`Cuts::compile`]. The quark carries `quark_pdg` (positive), massless.
pub fn dy_external_legs(quark_pdg: i32) -> Vec<ExternalLeg> {
    vec![
        ExternalLeg::incoming(quark_pdg, 0.0),
        ExternalLeg::incoming(-quark_pdg, 0.0),
        ExternalLeg::outgoing(-11, 0.0),
        ExternalLeg::outgoing(11, 0.0),
    ]
}

/// Compile and helicity-prune a class amplitude from its representative
/// subprocess `DiagramSet`.
pub fn compile_class(
    set: &DiagramSet,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
) -> Result<AmplitudeEvaluator, HadronicError> {
    let mut evaluator = AmplitudeEvaluator::compile(set, model)
        .map_err(|e| HadronicError::Compile(e.to_string()))?;
    evaluator.prune_zero_helicities(evaluated);
    Ok(evaluator)
}

/// Number of spin (helicity / polarization) states of a particle, from its UFO
/// spin code (`2s+1`) and whether it is massless. A massless vector has no
/// longitudinal mode, so it carries two states rather than three.
fn spin_state_count(spin_code: i32, massless: bool) -> usize {
    match spin_code.abs() {
        1 => 1,                          // scalar
        2 => 2,                          // fermion
        3 => usize::from(!massless) + 2, // vector: 2 (massless) or 3 (massive)
        5 => 5,                          // spin-2
        other => panic!("unsupported spin code {other} for the spin average"),
    }
}

/// The initial-state spin×colour averaging factor `1 / Π_a (n_spin,a · n_colour,a)`
/// over the incoming legs of a compiled process.
///
/// Derived from the UFO particle data — spin code and colour-representation
/// dimension (`|color|`: singlet 1, fundamental 3, adjoint 8) — and the resolved
/// masses, so a process supplies its own averaging denominator instead of a
/// hand-coded constant (`1/(2·2·3·3) = 1/36` for a quark–antiquark initial state,
/// `1/(2·2) = 1/4` for `e⁺e⁻`, `1/(2·8·2·8) = 1/256` for `gg`).
pub fn initial_spin_color_average(
    eval: &AmplitudeEvaluator,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
) -> f64 {
    let mut denom = 1.0f64;
    for &id in eval.external_particles().iter().take(eval.n_in()) {
        let particle = model.particle(id);
        let massless = evaluated.mass(id) == 0.0;
        let n_spin = spin_state_count(particle.spin, massless);
        let n_color = particle.color.unsigned_abs() as usize;
        denom *= (n_spin * n_color) as f64;
    }
    1.0 / denom
}

/// The final-state identical-particle symmetry factor `1 / Π_s n_s!`.
///
/// A phase-space map covers the whole of `dΦ_n`, which counts every permutation of
/// a set of identical outgoing particles as a distinct configuration, while the
/// cross section counts each such configuration once. `g g → g g` is the case that
/// makes the difference visible: without the `1/2!` its cross section comes out
/// exactly twice MadGraph's, with everything else — the bit-exact `|M|²`, the flux,
/// and the `1/256` initial-state average — already agreeing.
pub fn final_state_symmetry_factor(eval: &AmplitudeEvaluator) -> f64 {
    let out = &eval.external_particles()[eval.n_in()..];
    // Each leg contributes the next factor of its own species' factorial, counted
    // by how many earlier legs it matches, so one pass builds `Π_s n_s!`.
    let multiplicities: f64 = out
        .iter()
        .enumerate()
        .map(|(i, id)| (out[..i].iter().filter(|&other| other == id).count() + 1) as f64)
        .product();
    1.0 / multiplicities
}

/// Build the [`ExternalLeg`] list (incoming legs first, then outgoing) for a
/// compiled process, reading PDG codes and pole masses from the model — the
/// input [`Cuts::compile`] classifies.
pub fn process_external_legs(
    eval: &AmplitudeEvaluator,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
) -> Vec<ExternalLeg> {
    eval.external_particles()
        .iter()
        .enumerate()
        .map(|(i, &id)| {
            let pdg = model.particle(id).pdg_code as i32;
            let mass = evaluated.mass(id);
            if i < eval.n_in() {
                ExternalLeg::incoming(pdg, mass)
            } else {
                ExternalLeg::outgoing(pdg, mass)
            }
        })
        .collect()
}

/// Compile every non-empty subprocess of a generated proc card into a
/// helicity-pruned evaluator, requiring that they share one external-particle
/// sequence so a single RAMBO mass list and one cut filter serve them all.
pub fn compile_subprocesses(
    sets: &[DiagramSet],
    model: &UFOModel,
    evaluated: &EvaluatedModel,
) -> Result<Vec<AmplitudeEvaluator>, HadronicError> {
    let mut evals = Vec::new();
    for set in sets {
        if set.diagrams.is_empty() {
            continue;
        }
        evals.push(compile_class(set, model, evaluated)?);
    }
    if evals.is_empty() {
        return Err(HadronicError::NoSubprocess);
    }
    let first: Vec<_> = evals[0].external_particles().to_vec();
    if evals[1..]
        .iter()
        .any(|e| e.external_particles() != first.as_slice())
    {
        return Err(HadronicError::InconsistentExternals);
    }
    Ok(evals)
}

/// One compiled subprocess feeding a summed matrix element: an amplitude and its
/// own evaluation scratch (behind [`RefCell`] so the integrand is `Fn`).
struct BoundSubprocess<'a> {
    amp: SubAmplitude<'a>,
    scratch: RefCell<ScratchSpace<f64>>,
}

/// A subprocess amplitude, held at the parameter card's own strong coupling or at
/// the one each event's renormalisation scale implies.
///
/// The scale-aware form owns its constant pools, so it is mutable state. It lives
/// behind the integrand's [`RefCell`]s, which makes the integrand `!Sync` and so
/// makes the failure a shared pool would produce — two threads reading each
/// other's coupling, giving a silently wrong `|M|²` with no panic and no NaN —
/// unrepresentable rather than merely avoided. A parallel driver builds one
/// integrand per thread, each forking its own amplitudes.
enum SubAmplitude<'a> {
    Fixed(&'a BoundAmplitude<'a, f64>),
    Running(RefCell<ScaleAwareAmplitude<'a, f64>>),
}

impl<'a> BoundSubprocess<'a> {
    fn fixed(amp: &'a BoundAmplitude<'a, f64>) -> Self {
        BoundSubprocess {
            scratch: RefCell::new(amp.scratch_space()),
            amp: SubAmplitude::Fixed(amp),
        }
    }

    /// The bound evaluator this subprocess was built from, the input a scale-aware
    /// copy is derived from.
    fn evaluator(&self) -> &'a AmplitudeEvaluator {
        match &self.amp {
            SubAmplitude::Fixed(amp) => amp.evaluator(),
            SubAmplitude::Running(amp) => amp.borrow().amplitude().evaluator(),
        }
    }

    /// Replace the amplitude by a scale-aware copy of itself, bound against
    /// `evaluated`. The copy starts at the parameter card's own coupling, with
    /// pools bit-for-bit those of the amplitude it replaces.
    fn make_scale_aware(&mut self, evaluated: &EvaluatedModel) {
        let amp = ScaleAwareAmplitude::<f64>::new(self.evaluator(), evaluated);
        self.amp = SubAmplitude::Running(RefCell::new(amp));
    }

    fn scale_aware(&self) -> Option<&RefCell<ScaleAwareAmplitude<'a, f64>>> {
        match &self.amp {
            SubAmplitude::Fixed(_) => None,
            SubAmplitude::Running(amp) => Some(amp),
        }
    }

    fn set_alpha_s(&self, alpha_s: f64) {
        if let SubAmplitude::Running(amp) = &self.amp {
            amp.borrow_mut().set_alpha_s(alpha_s);
        }
    }

    fn eval_m2(&self, momenta: &[V]) -> f64 {
        let scratch = &mut self.scratch.borrow_mut();
        match &self.amp {
            SubAmplitude::Fixed(amp) => amp.eval_m2(momenta, scratch),
            SubAmplitude::Running(amp) => amp.borrow().eval_m2(momenta, scratch),
        }
    }

    /// The per-combination `|M_c|²` and per-flow `JAMP2` diagonals at the current
    /// coupling — the two categorical weight vectors an event record's helicity and
    /// colour flow are drawn from. Neither enters `eval_m2`, so neither moves the
    /// cross section.
    fn eval_diagonals(&self, momenta: &[V], hel_m2: &mut [f64], jamp2: &mut [f64]) {
        let scratch = &mut self.scratch.borrow_mut();
        // The running form's own amplitude carries the pools the current scale set,
        // so both diagonals are read at the coupling `eval_m2` was taken at.
        match &self.amp {
            SubAmplitude::Fixed(amp) => {
                amp.eval_hel_m2(momenta, scratch, hel_m2);
                amp.eval_jamp2(momenta, scratch, jamp2);
            }
            SubAmplitude::Running(running) => {
                let amp = running.borrow();
                amp.amplitude().eval_hel_m2(momenta, scratch, hel_m2);
                amp.amplitude().eval_jamp2(momenta, scratch, jamp2);
            }
        }
    }
}

/// What turning a set of subprocesses scale-aware revealed about them.
struct ScaleAwareness {
    depends_on_alpha_s: bool,
    fallbacks: Vec<String>,
    alpha_s_ref: Option<f64>,
}

/// Replace every subprocess amplitude by a scale-aware copy of itself.
fn make_subs_scale_aware(
    subs: &mut [BoundSubprocess<'_>],
    evaluated: &EvaluatedModel,
) -> ScaleAwareness {
    for sub in subs.iter_mut() {
        sub.make_scale_aware(evaluated);
    }
    let scale_aware = || subs.iter().filter_map(BoundSubprocess::scale_aware);
    ScaleAwareness {
        depends_on_alpha_s: scale_aware().any(|a| a.borrow().depends_on_alpha_s()),
        fallbacks: scale_aware()
            .filter_map(|a| a.borrow().fallback().map(|f| f.to_string()))
            .collect(),
        alpha_s_ref: scale_aware().next().map(|a| a.borrow().alpha_s_ref()),
    }
}

/// Compile the run card's prescription for a process, deriving the topology its
/// clustering branch consults from the process's own diagrams.
///
/// The running coupling is built only when some subprocess actually moves with it,
/// which is what keeps a matrix element with no QCD in it away from a `pdlabel`
/// whose `αs` `coupling::alphas` refuses to supply.
fn compile_scale_source(
    rep: &AmplitudeEvaluator,
    diagrams: &[Diagram],
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    card: &RunCard,
    needs_alpha_s: bool,
) -> Result<EventScaleSource, HadronicError> {
    let topology = cluster_topology(
        diagrams,
        rep.external_particles(),
        rep.n_in(),
        model,
        card.maxjetflavor,
    );
    let param_card_as = evaluated.alpha_s().ok_or(HadronicError::MissingAlphaS)?;
    EventScaleSource::from_run_card(card, param_card_as, Some(topology), needs_alpha_s)
}

/// Hold every subprocess at the coupling a constant prescription implies, and
/// assemble the report describing what was installed.
fn constant_scale_report(
    subs: &[BoundSubprocess<'_>],
    source: Option<&EventScaleSource>,
    awareness: ScaleAwareness,
) -> RunningCouplingReport {
    let constant_scales = source.and_then(EventScaleSource::constant_scales);
    let constant_alpha_s = match (constant_scales, source.and_then(|s| s.running_alpha_s())) {
        (Some(scales), Some(running)) => Some(running.eval(scales.mu_r)),
        _ => None,
    };
    if let Some(alpha_s) = constant_alpha_s {
        for sub in subs {
            sub.set_alpha_s(alpha_s);
        }
    }
    RunningCouplingReport {
        depends_on_alpha_s: awareness.depends_on_alpha_s,
        topology: source.and_then(EventScaleSource::topology),
        constant_scales,
        constant_alpha_s,
        alpha_s_ref: awareness.alpha_s_ref,
        fallbacks: awareness.fallbacks,
    }
}

/// A ready-to-integrate cross section for a **fixed-energy, no-PDF** beam
/// configuration (`lpp = 0`) and an arbitrary final-state multiplicity, sampled
/// under VEGAS.
///
/// The incoming particles *are* the beam particles: `√ŝ = E₁ + E₂` is fixed, so
/// there is no `τ`/`x` sampling and no PDF luminosity. The phase-space
/// [`PhaseSpaceMap`] maps the VEGAS uniforms to `n` on-shell final-state momenta
/// summing to `(√ŝ, 0, 0, 0)` and supplies the invariant-volume weight; the two
/// beams sit at `√ŝ/2` along ±z, so the full external set is already the
/// partonic-CM, ±z-beam frame the helicity-pruned [`BoundAmplitude::eval_m2`]
/// requires, and (for symmetric beams) the lab frame coincides with it — the same
/// momenta feed the cut filter.
///
/// The map is flat [`RamboChannel`] by default. [`use_multichannel`] swaps in a
/// resonance-aware per-diagram [`MultiChannel`] combiner, α-adapted to this very
/// integrand, so a narrow Breit–Wigner peak (which flat RAMBO under-samples) is
/// importance-mapped and the integral converges at far lower variance. Both maps
/// carry the *same* invariant-volume weight normalisation (`R_n`, no `2π`), so the
/// master formula below is unchanged — only the sampling density is.
///
/// [`use_multichannel`]: FixedBeamIntegrand::use_multichannel
///
/// # Master formula
///
/// ```text
/// σ̂ = 1/(2ŝ) · ⟨spin·colour avg⟩ · S · ∫ dΦ_n Σ|M|²
///    = 1/(2ŝ) · avg · S · (2π)^{4−3n} · ⟨weight · Σ_sub |M_sub|²⟩_uniform
/// ```
///
/// where `Σ|M|²` is the colour+helicity-summed matrix element ([`eval_m2`]), the
/// `(2π)^{4−3n}` factor turns the map's invariant volume `R_n` into the full
/// `dΦ_n` measure, and `S = 1/Π_s n_s!` is the identical-particle symmetry factor
/// ([`final_state_symmetry_factor`]) that undoes `dΦ_n`'s over-counting of the
/// permutations of identical outgoing legs.
///
/// [`eval_m2`]: BoundAmplitude::eval_m2
pub struct FixedBeamIntegrand<'a> {
    subs: Vec<BoundSubprocess<'a>>,
    cuts: &'a Cuts,
    sqrt_s: f64,
    /// Unit-hypercube → phase-space map over the outgoing legs, on the fixed `√ŝ`
    /// and masses: flat [`RamboChannel`] by default, a resonance-aware
    /// [`MultiChannel`] once [`use_multichannel`](Self::use_multichannel) has run.
    sampler: Sampler,
    /// The outgoing pole masses in leg order, the map's targets.
    final_masses: Vec<f64>,
    /// `1 / Π_a (n_spin · n_colour)` over the incoming legs.
    spin_color_avg: f64,
    /// The final-state identical-particle factor `1 / Π_s n_s!`, derived from the
    /// process's outgoing legs ([`final_state_symmetry_factor`]).
    symmetry_factor: f64,
    /// The `(2π)^{4−3n}` measure factor.
    lips_2pi: f64,
    /// Beam energy `√ŝ/2`.
    beam_e: f64,
    /// Grid-damping exponent for the VEGAS pass, following the active sampler:
    /// [`VEGAS_ALPHA`] over the raw flat map, [`VEGAS_ALPHA_MAPPED`] once a
    /// multichannel map has already flattened the integrand's known peaks.
    vegas_alpha: f64,
    /// The run card's per-event renormalisation scale, once
    /// [`use_running_coupling`](Self::use_running_coupling) has installed it.
    /// `None`, or a prescription that resolves to a constant, leaves every
    /// subprocess at one coupling and costs nothing per point.
    scales: Option<EventScaleSource>,
    /// Reused marshalling buffer for the outgoing momenta the scale reads.
    scale_buf: RefCell<Vec<[f64; 4]>>,
    /// The last `(μR, αs(μR))` pair, so a repeated scale does not repeat the
    /// coupling's Newton solve. The prescription can be dynamic and still land on
    /// one value every event — a fixed-beam `2 → 2` clusters to `√ŝ/2` whatever the
    /// point — and that case is only visible per event, not from the run card.
    last_coupling: Cell<(f64, f64)>,
}

/// The phase-space map a [`FixedBeamIntegrand`] draws through.
///
/// Held as a closed set rather than a trait object because the two arms are
/// integrated differently: the flat map is one integral over one grid, while the
/// combiner's integral splits into one term — and one grid — per channel.
enum Sampler {
    Flat(RamboChannel<f64>),
    Multi(MultiChannel<f64>),
}

impl Sampler {
    fn ndim(&self) -> usize {
        match self {
            Sampler::Flat(c) => c.ndim(),
            Sampler::Multi(c) => c.ndim(),
        }
    }

    fn sample(&self, u: &[f64]) -> PhaseSpacePoint<f64> {
        match self {
            Sampler::Flat(c) => c.sample(u),
            Sampler::Multi(c) => c.sample(u),
        }
    }
}

/// One channel's share of a per-channel integration
/// ([`FixedBeamIntegrand::adapt_grids`]).
///
/// The integral is estimated as `Σⱼ ∫ dΦ f·αⱼgⱼ/g` — one VEGAS pass per term,
/// each over its channel's own `channel_ndim` coordinates — so every channel
/// carries its own trained grid, its own σⱼ ± Δσⱼ, and the sample budget it was
/// given. A run sampled by the flat map is the one-term case: `alpha = 1` and the
/// whole budget on a single grid.
#[derive(Debug, Clone)]
pub struct ChannelIntegration {
    /// The channel's selection weight `αⱼ` — both the weight in its term's
    /// integrand and the share of the sample budget it was allocated.
    pub alpha: f64,
    /// Evaluations per iteration this channel actually received.
    pub neval: usize,
    /// The grid trained on this channel's term.
    pub grid: VegasGrid,
    /// This term's integral and error, in natural units (GeV⁻²).
    pub result: VegasResult,
}

/// The discrete labels an accepted event carries besides its momenta, drawn by
/// [`FixedBeamIntegrand::select_event`].
///
/// Every one of them is summed over in the cross section, so none of them is a
/// sampling channel: they are read off diagonal accumulators after the fact, to
/// fill in an event record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventSelection {
    /// Index into the integrand's subprocesses
    /// ([`FixedBeamIntegrand::subprocess_evaluator`]).
    pub subprocess: usize,
    /// The helicity of each external leg, in process order.
    pub helicity: Vec<i32>,
    /// Index into the subprocess's colour-flow basis, and so into its
    /// [`ColorFlowTags`](crate::helas::color::flow_tags::ColorFlowTags) table.
    pub flow: usize,
}

/// Sum the per-channel terms: integrals add, errors add in quadrature, and the
/// χ²/dof is the pooled statistic (`Σ χ²ⱼ` over `Σ dofⱼ`) rather than any single
/// channel's.
fn combine_channels(per_channel: &[ChannelIntegration], niter: usize) -> VegasResult {
    let integral: f64 = per_channel.iter().map(|c| c.result.integral).sum();
    let variance: f64 = per_channel
        .iter()
        .map(|c| c.result.std_dev * c.result.std_dev)
        .sum();
    let dof_each = niter.saturating_sub(1);
    let chi2_per_dof = if dof_each > 0 && !per_channel.is_empty() {
        per_channel
            .iter()
            .map(|c| c.result.chi2_per_dof)
            .sum::<f64>()
            / per_channel.len() as f64
    } else {
        0.0
    };
    VegasResult {
        integral,
        std_dev: variance.sqrt(),
        chi2_per_dof,
    }
}

/// A channel's per-iteration evaluation count: its share `αⱼ · neval` of the
/// budget, floored so no channel goes unsampled.
fn channel_neval(alpha: f64, neval: usize) -> usize {
    let share = (alpha * neval as f64).round();
    let share = if share.is_finite() && share > 0.0 {
        share as usize
    } else {
        0
    };
    share.max(MIN_CHANNEL_NEVAL)
}

impl<'a> FixedBeamIntegrand<'a> {
    /// Build the integrand from one or more bound subprocess amplitudes sharing
    /// the same external state.
    ///
    /// * `amps` — bound amplitudes whose colour+helicity-summed |M|² are added
    ///   (a single subprocess for a fully-specified initial state).
    /// * `cuts` — the compiled cut filter.
    /// * `sqrt_s` — the fixed partonic energy `E₁ + E₂`.
    /// * `final_masses` — outgoing pole masses in leg order (the RAMBO targets).
    /// * `spin_color_avg` — the initial-state average ([`initial_spin_color_average`]).
    pub fn new(
        amps: Vec<&'a BoundAmplitude<'a, f64>>,
        cuts: &'a Cuts,
        sqrt_s: f64,
        final_masses: Vec<f64>,
        spin_color_avg: f64,
    ) -> Self {
        let n = final_masses.len();
        let symmetry_factor = final_state_symmetry_factor(amps[0].evaluator());
        let subs = amps.into_iter().map(BoundSubprocess::fixed).collect();
        let sampler = Sampler::Flat(RamboChannel::new(sqrt_s, final_masses.clone()));
        FixedBeamIntegrand {
            subs,
            cuts,
            sqrt_s,
            sampler,
            final_masses,
            spin_color_avg,
            symmetry_factor,
            lips_2pi: (2.0 * PI).powi(4 - 3 * n as i32),
            beam_e: sqrt_s / 2.0,
            vegas_alpha: VEGAS_ALPHA,
            scales: None,
            scale_buf: RefCell::new(Vec::with_capacity(n)),
            last_coupling: Cell::new((f64::NAN, f64::NAN)),
        }
    }

    /// Evaluate the matrix element at the strong coupling the run card's per-event
    /// renormalisation scale implies, instead of at the parameter card's own.
    ///
    /// Each subprocess is replaced by a scale-aware copy owning its constant pools;
    /// the [`ClusterTopology`](crate::coupling::scales::ClusterTopology) the `-1`
    /// scale consults is derived from `diagrams` rather than declared per process.
    /// A fixed-beam run has no parton distributions, so only `μR` is consumed here —
    /// the per-beam `μF` the same prescription produces has nothing to feed.
    ///
    /// Call this **before** [`use_multichannel`](Self::use_multichannel), so the
    /// α-adaptation survey sees the integrand the integration will see.
    ///
    /// The scale is resolved once here on a sampled, cut-passing phase-space point,
    /// so a prescription this crate refuses — an unimplemented clustering above all —
    /// stops the run at setup instead of at the first VEGAS point.
    pub fn use_running_coupling(
        &mut self,
        diagrams: &[Diagram],
        model: &UFOModel,
        evaluated: &EvaluatedModel,
        card: &RunCard,
    ) -> Result<RunningCouplingReport, HadronicError> {
        let awareness = make_subs_scale_aware(&mut self.subs, evaluated);
        // With no parton distributions to read and no strong coupling in the matrix
        // element, neither scale the prescription produces has a consumer, so the
        // prescription is not compiled at all — a process whose cluster scale this
        // crate refuses still integrates, because its cross section does not depend
        // on the scale that was refused.
        if !awareness.depends_on_alpha_s {
            self.scales = None;
            return Ok(constant_scale_report(&self.subs, None, awareness));
        }
        let source = compile_scale_source(
            self.subs[0].evaluator(),
            diagrams,
            model,
            evaluated,
            card,
            true,
        )?;
        if source.constant_scales().is_none() {
            self.probe_scale(&source)?;
        }
        let report = constant_scale_report(&self.subs, Some(&source), awareness);
        self.scales = Some(source);
        Ok(report)
    }

    /// Resolve the scale on the first cut-passing point of a fixed pseudo-random
    /// draw, so a refusal that would otherwise surface mid-integration surfaces at
    /// setup.
    ///
    /// The draw goes through the cut filter because the scale is only ever asked for
    /// on points that pass it: an unphysical configuration — a leg carrying no
    /// energy, say — has no beam measure to compare, and refusing on one would say
    /// nothing about the process.
    fn probe_scale(&self, source: &EventScaleSource) -> Result<(), HadronicError> {
        use rand::Rng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(SCALE_PROBE_SEED);
        for _ in 0..SCALE_PROBE_DRAWS {
            let u: Vec<f64> = (0..self.sampler.ndim())
                .map(|_| rng.random::<f64>())
                .collect();
            let point = self.sampler.sample(&u);
            let mut ext: Vec<V> = Vec::with_capacity(2 + point.momenta.len());
            ext.push(V::new(self.beam_e, 0.0, 0.0, self.beam_e));
            ext.push(V::new(self.beam_e, 0.0, 0.0, -self.beam_e));
            ext.extend_from_slice(&point.momenta);
            if self.cuts.pass(&ext) {
                self.event_scales_of(source, &point.momenta)?;
                return Ok(());
            }
        }
        Ok(())
    }

    /// The scales at one phase-space point, from the beams and the outgoing momenta.
    fn event_scales_of(
        &self,
        source: &EventScaleSource,
        momenta: &[V],
    ) -> Result<EventScales, ScaleError> {
        let mut buf = self.scale_buf.borrow_mut();
        buf.clear();
        buf.extend(momenta.iter().map(components));
        let beams = [
            [self.beam_e, 0.0, 0.0, self.beam_e],
            [self.beam_e, 0.0, 0.0, -self.beam_e],
        ];
        source.scales(beams, &buf)
    }

    /// Move every subprocess to the coupling this point's renormalisation scale
    /// implies. A constant prescription was applied once at installation, and a
    /// matrix element with no strong coupling in it has no coupling to move, so
    /// both return here without touching the momenta.
    fn apply_scale(&self, momenta: &[V]) {
        let Some(source) = &self.scales else { return };
        if source.constant_scales().is_some() {
            return;
        }
        let Some(running) = source.running_alpha_s() else {
            return;
        };
        let scales = self
            .event_scales_of(source, momenta)
            .unwrap_or_else(|e| panic!("per-event scale on a sampled point: {e}"));
        let (last_mu_r, last_alpha_s) = self.last_coupling.get();
        let alpha_s = if scales.mu_r == last_mu_r {
            last_alpha_s
        } else {
            let alpha_s = running.eval(scales.mu_r);
            self.last_coupling.set((scales.mu_r, alpha_s));
            alpha_s
        };
        for sub in &self.subs {
            sub.set_alpha_s(alpha_s);
        }
    }

    /// The uniforms the active phase-space map consumes as one mixture: `4n` for
    /// flat RAMBO, and `3n − 3` for the multichannel combiner (one
    /// channel-selection coordinate plus the `3n − 4` invariant/angle
    /// coordinates).
    ///
    /// This is the dimensionality of the single grid
    /// [`adapt_grid`](Self::adapt_grid) builds. The per-channel grids
    /// [`adapt_grids`](Self::adapt_grids) builds are over
    /// [`channel_grid_ndim`](Self::channel_grid_ndim) instead.
    pub fn vegas_ndim(&self) -> usize {
        self.sampler.ndim()
    }

    /// The colour+helicity-summed `Σ|M|²` at the outgoing momenta `momenta`, in the
    /// partonic-CM frame, with the beams prepended and the cut filter applied — the
    /// matrix-element part of the integrand as a function of the phase-space point.
    /// A configuration failing a cut returns exactly `0.0`, so it drops out of both
    /// the cross section and the α-adaptation survey.
    fn matrix_element(&self, momenta: &[V]) -> f64 {
        let ext = self.externals(momenta);
        if !self.cuts.pass(&ext) {
            return 0.0;
        }
        self.apply_scale(momenta);

        let mut m2 = 0.0;
        for sub in &self.subs {
            m2 += sub.eval_m2(&ext);
        }
        m2
    }

    /// The integrand value at a VEGAS point `u ∈ [0,1]^ndim`, in natural units
    /// (GeV⁻²); its VEGAS integral is the partonic cross section. Points whose
    /// momenta fail a cut contribute exactly zero.
    pub fn value(&self, u: &[f64]) -> f64 {
        let point = self.sampler.sample(u);
        let m2 = self.matrix_element(&point.momenta);
        if m2 == 0.0 {
            return 0.0;
        }
        self.prefactor() * point.weight * m2
    }

    /// The constants in front of `weight · Σ|M|²`: the `1/(2ŝ)` flux, the
    /// initial-state spin×colour average, the identical-particle symmetry factor,
    /// and the `(2π)^{4−3n}` measure factor.
    fn prefactor(&self) -> f64 {
        let flux = 1.0 / (2.0 * self.sqrt_s * self.sqrt_s);
        flux * self.spin_color_avg * self.symmetry_factor * self.lips_2pi
    }

    /// Replace flat RAMBO with a resonance-aware per-diagram [`MultiChannel`] built
    /// from `diagrams` (one [`DiagramChannel`] each, its propagator poles read from
    /// `model`), then α-adapt the channel mixture to *this* integrand and install
    /// the adapted combiner as the sampler.
    ///
    /// The α-adaptation surveys the combiner under the process's own `Σ|M|²` (the
    /// [`matrix_element`](Self::matrix_element) shape, cut included), so weight
    /// flows to the channels that carry the integrand's variance. Constant
    /// prefactors (flux, spin/colour average, the `2π` measure) are omitted from
    /// the survey integrand: they scale every channel's variance share equally and
    /// so leave the Kleiss–Pittau reallocation unchanged, while keeping the survey
    /// cheaper. The combiner shares RAMBO's `R_n` weight normalisation, so the
    /// master formula and every prefactor are untouched — only the sampling density
    /// changes, and the estimator stays unbiased for the same `σ̂`.
    ///
    /// Returns the α refinement path, or `None` if `diagrams` is empty (the flat
    /// sampler is then left in place).
    pub fn use_multichannel(
        &mut self,
        diagrams: &[Diagram],
        model: &EvaluatedModel,
        n_survey: usize,
        n_iter: usize,
        seed: u64,
    ) -> Option<AlphaAdaptation<f64>> {
        let channels: Vec<Box<dyn Channel<f64>>> = diagrams
            .iter()
            .map(|d| {
                Box::new(DiagramChannel::from_diagram(d, model, self.sqrt_s))
                    as Box<dyn Channel<f64>>
            })
            .collect();
        if channels.is_empty() {
            return None;
        }
        let mut combiner = MultiChannel::uniform(channels);
        let report = combiner.adapt_alphas(
            |momenta| self.matrix_element(momenta),
            seed,
            MULTICHANNEL_ADAPT_STREAM,
            n_survey,
            n_iter,
            0.5,
        );
        self.sampler = Sampler::Multi(combiner);
        self.vegas_alpha = VEGAS_ALPHA_MAPPED;
        Some(report)
    }

    /// The channels the integral is split across: one per diagram once a
    /// multichannel combiner is installed, and `1` under the flat map.
    pub fn channel_count(&self) -> usize {
        match &self.sampler {
            Sampler::Flat(_) => 1,
            Sampler::Multi(c) => c.channels().len(),
        }
    }

    /// The converged channel selection weights, or `[1.0]` under the flat map.
    pub fn channel_alphas(&self) -> Vec<f64> {
        match &self.sampler {
            Sampler::Flat(_) => vec![1.0],
            Sampler::Multi(c) => c.alphas().to_vec(),
        }
    }

    /// The uniforms one channel's grid is built over: `channel_ndim` for the
    /// combiner (no channel-selection coordinate — the channel is frozen), and the
    /// full map dimension under flat RAMBO.
    pub fn channel_grid_ndim(&self) -> usize {
        match &self.sampler {
            Sampler::Flat(c) => c.ndim(),
            Sampler::Multi(c) => c.channel_ndim(),
        }
    }

    /// The `j`-th term of the channel-split estimator at `u ∈ [0,1]^channel_ndim`,
    /// in natural units (GeV⁻²): the point is drawn from channel `j` alone and
    /// weighted by `αⱼ/g`, so the sum over channels of these terms' integrals is
    /// the same cross section [`value`](Self::value) integrates from the mixture.
    ///
    /// Under the flat map there is one channel and this is [`value`](Self::value).
    ///
    /// # Panics
    ///
    /// If `channel` is not a channel index ([`channel_count`](Self::channel_count)).
    pub fn value_in_channel(&self, channel: usize, u: &[f64]) -> f64 {
        let point = self.sample_channel(channel, u);
        let m2 = self.matrix_element(&point.momenta);
        if m2 == 0.0 {
            return 0.0;
        }
        self.prefactor() * point.weight * m2
    }

    /// [`value_in_channel`](Self::value_in_channel) with the outgoing momenta kept:
    /// `momenta` is overwritten with the point the value was taken at, in
    /// outgoing-leg order and in the partonic-CM frame.
    ///
    /// An accept/reject pass needs the momenta only for the points it keeps, so the
    /// trial loop runs on `value_in_channel` and reconstructs an accepted point
    /// through this — the same map at the same `u`, hence the same weight.
    pub fn event_in_channel(&self, channel: usize, u: &[f64], momenta: &mut Vec<V>) -> f64 {
        let point = self.sample_channel(channel, u);
        momenta.clear();
        momenta.extend_from_slice(&point.momenta);
        let m2 = self.matrix_element(&point.momenta);
        if m2 == 0.0 {
            return 0.0;
        }
        self.prefactor() * point.weight * m2
    }

    /// Draw the phase-space point channel `channel` maps `u` to, with its weight.
    fn sample_channel(&self, channel: usize, u: &[f64]) -> PhaseSpacePoint<f64> {
        match &self.sampler {
            Sampler::Flat(c) => {
                assert_eq!(channel, 0, "the flat map has a single channel");
                c.sample(u)
            }
            Sampler::Multi(c) => c.sample_channel(channel, u),
        }
    }

    /// The two incoming momenta: `√ŝ/2` along ±z, the beam configuration every
    /// evaluation in this integrand is made in.
    pub fn beams(&self) -> [V; 2] {
        [
            V::new(self.beam_e, 0.0, 0.0, self.beam_e),
            V::new(self.beam_e, 0.0, 0.0, -self.beam_e),
        ]
    }

    /// The external momenta an amplitude is evaluated at: the beams, then the
    /// outgoing legs.
    fn externals(&self, momenta: &[V]) -> Vec<V> {
        let mut ext = Vec::with_capacity(2 + momenta.len());
        ext.extend_from_slice(&self.beams());
        ext.extend_from_slice(momenta);
        ext
    }

    /// The subprocesses whose `|M|²` this integrand adds.
    pub fn subprocess_count(&self) -> usize {
        self.subs.len()
    }

    /// The compiled evaluator of one subprocess — the source of the external
    /// particle ids, the helicity combinations and the colour-flow tag table an
    /// event record is written from.
    pub fn subprocess_evaluator(&self, subprocess: usize) -> &'a AmplitudeEvaluator {
        self.subs[subprocess].evaluator()
    }

    /// Fill in an accepted event's discrete labels: which subprocess produced it,
    /// which helicity combination, and which colour flow.
    ///
    /// `momenta` are the accepted point's outgoing momenta and `u` three
    /// independent uniforms. The subprocess is drawn `∝ |M_s|²` (the incoherent sum
    /// this integrand forms), then within it the helicity `∝ |M_c|²`
    /// (MadGraph's `SELECT_HEL`) and the colour flow `∝ JAMP2(i)`
    /// (`SELECT_COLOR`).
    ///
    /// All three are *selections*, not sampling channels: the cross section sums
    /// over subprocesses, helicities and flows, and this reads accumulators that
    /// decomposition already contains. It enters no integrand and moves no cross
    /// section — a caller may skip it entirely and integrate the same number.
    ///
    /// `None` when the point carries no weight at all (outside the cuts, or a
    /// vanishing matrix element), where no label is defined.
    pub fn select_event(&self, momenta: &[V], u: [f64; 3]) -> Option<EventSelection> {
        let ext = self.externals(momenta);
        // The diagonals are read at the event's own coupling, the one its |M|² was
        // taken at.
        self.apply_scale(momenta);

        let m2: Vec<f64> = self.subs.iter().map(|s| s.eval_m2(&ext)).collect();
        let subprocess = select_index(&m2, u[0])?;
        let sub = &self.subs[subprocess];
        let eval = sub.evaluator();

        let mut hel_m2 = vec![0.0; eval.helicities().len()];
        let mut jamp2 = vec![0.0; eval.n_flows()];
        sub.eval_diagonals(&ext, &mut hel_m2, &mut jamp2);

        Some(EventSelection {
            subprocess,
            helicity: eval.select_helicity(&hel_m2, u[1])?.to_vec(),
            flow: select_flow(&jamp2, u[2])?,
        })
    }

    /// The final-state pole masses in outgoing-leg order.
    pub fn final_masses(&self) -> &[f64] {
        &self.final_masses
    }

    /// Integrate the cross section with VEGAS, returning `(σ, Δσ)` in picobarns.
    pub fn integrate(&self, neval: usize, niter: usize, seed: u64) -> (f64, f64) {
        let result = self.adapt_grids(neval, niter, seed).1;
        (result.integral * GEV2_TO_PB, result.std_dev * GEV2_TO_PB)
    }

    /// Run VEGAS adaptation over the mixture map as a single integral, returning
    /// the one trained grid alongside the result.
    ///
    /// The combiner's channel selection then occupies a coordinate of that grid.
    /// [`adapt_grids`](Self::adapt_grids) is what the cross section is taken
    /// from; this is the undivided comparison point.
    pub fn adapt_grid(&self, neval: usize, niter: usize, seed: u64) -> (VegasGrid, VegasResult) {
        let mut grid = VegasGrid::new(self.vegas_ndim(), VEGAS_NBINS, self.vegas_alpha);
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        let result = grid.adapt(|u| self.value(u), neval, niter, &mut rng);
        (grid, result)
    }

    /// Run one VEGAS adaptation **per channel**, returning each channel's trained
    /// grid and term alongside their sum — the primitive the `integrate` CLI
    /// command serializes into its artifact.
    ///
    /// Channel `j` is integrated over its own `channel_ndim` coordinates with the
    /// channel frozen ([`value_in_channel`](Self::value_in_channel)), on a sample
    /// budget of `αⱼ · neval` per iteration and its own RNG substream. The terms sum
    /// to the same cross section the mixture integrates, but each grid now refines a
    /// density *conditional* on its channel — the correlation a single separable
    /// grid over the mixture cannot represent, since the useful shape of the
    /// remaining coordinates depends on which channel was selected.
    ///
    /// Under the flat map this is a single full-budget pass, identical to
    /// [`adapt_grid`](Self::adapt_grid).
    pub fn adapt_grids(
        &self,
        neval: usize,
        niter: usize,
        seed: u64,
    ) -> (Vec<ChannelIntegration>, VegasResult) {
        self.adapt_grids_with(neval, niter, seed, self.vegas_alpha)
    }

    /// [`adapt_grids`](Self::adapt_grids) with the VEGAS grid-damping exponent
    /// supplied instead of taken from the active sampler — the seam a study of the
    /// refinement's own stability drives.
    pub fn adapt_grids_with(
        &self,
        neval: usize,
        niter: usize,
        seed: u64,
        vegas_alpha: f64,
    ) -> (Vec<ChannelIntegration>, VegasResult) {
        let alphas = self.channel_alphas();
        let ndim = self.channel_grid_ndim();
        let mut per_channel = Vec::with_capacity(alphas.len());
        for (j, &alpha) in alphas.iter().enumerate() {
            let n_j = if alphas.len() == 1 {
                neval
            } else {
                channel_neval(alpha, neval)
            };
            let mut grid = VegasGrid::new(ndim, VEGAS_NBINS, vegas_alpha);
            let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
            rng.set_stream(CHANNEL_STREAM_BASE + j as u64);
            rng.set_word_pos(0);
            let result = grid.adapt(|u| self.value_in_channel(j, u), n_j, niter, &mut rng);
            per_channel.push(ChannelIntegration {
                alpha,
                neval: n_j,
                grid,
                result,
            });
        }
        let total = combine_channels(&per_channel, niter);
        (per_channel, total)
    }
}

/// The seam an accept/reject pass drives this integrand through: the channels its
/// integral is split across, and one term's value at a point of that channel's own
/// grid.
impl ChannelIntegrand for FixedBeamIntegrand<'_> {
    fn channel_count(&self) -> usize {
        FixedBeamIntegrand::channel_count(self)
    }

    fn channel_grid_ndim(&self) -> usize {
        FixedBeamIntegrand::channel_grid_ndim(self)
    }

    fn value_in_channel(&self, channel: usize, u: &[f64]) -> f64 {
        FixedBeamIntegrand::value_in_channel(self, channel, u)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runcard::RunCard;
    use crate::ufo::sm::{sm_model, SMRestrict};

    fn model() -> std::sync::Arc<UFOModel> {
        sm_model(SMRestrict::Default)
    }

    #[test]
    fn dy_flavor_classes_partition_matches_enumeration() {
        let m = model();
        let fc = dy_flavor_classes(generate_dy_subprocesses(&m).unwrap(), &m).expect("classify DY");
        assert_eq!(fc.up_flavors, vec![2, 4]);
        assert_eq!(fc.down_flavors, vec![1, 3]);
        // The representative subprocess has the quark on beam 1.
        assert_eq!(fc.up_set.particles_in.len(), 2);
        assert_eq!(fc.down_set.particles_in.len(), 2);
    }

    #[test]
    fn class_matrix_element_is_flavor_independent_within_class() {
        // The "one σ̂ per class" premise: c c̄ must give the same |M|² as u ū at
        // identical partonic-CM kinematics (massless quarks, same couplings). A
        // convention/coupling regression that made them differ would fail here.
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let opts = ParsingOptions::default();
        let build = |proc: &str| {
            let card = parse_proc_card(&format!("generate {proc}"), &opts).unwrap();
            let sets = generate_from_proc_card(&card, &m).unwrap();
            let set = sets.into_iter().find(|s| !s.diagrams.is_empty()).unwrap();
            compile_class(&set, &m, &evaluated).unwrap()
        };
        let uu = build("u u~ > e+ e-");
        let cc = build("c c~ > e+ e-");
        let b_uu = BoundAmplitude::<f64>::bind(&uu, &evaluated);
        let b_cc = BoundAmplitude::<f64>::bind(&cc, &evaluated);
        let mut s_uu = b_uu.scratch_space();
        let mut s_cc = b_cc.scratch_space();

        let sqrt_shat = 200.0;
        for &cos in &[-0.7, -0.2, 0.3, 0.85] {
            let k = build_kinematics(sqrt_shat, cos, 0.02, 0.01, (13000.0f64).powi(2));
            let a = b_uu.eval_m2(&k.cm, &mut s_uu);
            let b = b_cc.eval_m2(&k.cm, &mut s_cc);
            let rel = (a - b).abs() / a.abs().max(1e-30);
            assert!(
                rel < 1e-12,
                "u ū vs c c̄ |M|² differ: {a} vs {b} (rel {rel:.2e})"
            );
        }
    }

    #[test]
    fn cut_indicator_zeros_the_integrand() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let fc = dy_flavor_classes(generate_dy_subprocesses(&m).unwrap(), &m).unwrap();
        let up = compile_class(&fc.up_set, &m, &evaluated).unwrap();
        let down = compile_class(&fc.down_set, &m, &evaluated).unwrap();
        let b_up = BoundAmplitude::<f64>::bind(&up, &evaluated);
        let b_down = BoundAmplitude::<f64>::bind(&down, &evaluated);

        // Default DY cuts. `value` applies the cut indicator before consuming the
        // PDF, so a cut-failing point returns before touching it; use a tiny
        // synthetic PDF member.
        let rc = RunCard::default();
        let cuts = Cuts::compile(&rc, &dy_external_legs(2)).unwrap();
        let pdf = tiny_pdf();

        let avg = initial_spin_color_average(&up, &m, &evaluated);
        let integ = DrellYanIntegrand::new(
            &b_up,
            &b_down,
            &pdf,
            &cuts,
            fc.up_flavors,
            fc.down_flavors,
            13000.0,
            91.188,
            avg,
        );
        // cosθ → 1 (u₃ ≈ 1) sends the leptons collinear with the beam: pT → 0
        // fails the ptl = 10 GeV cut, so the indicator zeros the integrand.
        assert_eq!(integ.value(&[0.5, 0.5, 0.99999]), 0.0);
        // A central, well-clear point at the ŝ floor is nonzero.
        assert!(integ.value(&[0.0, 0.5, 0.5]) > 0.0);
    }

    fn build_evaluator(proc: &str, m: &UFOModel, evaluated: &EvaluatedModel) -> AmplitudeEvaluator {
        let opts = ParsingOptions::default();
        let card = parse_proc_card(&format!("generate {proc}"), &opts).unwrap();
        let sets = generate_from_proc_card(&card, m).unwrap();
        let set = sets.into_iter().find(|s| !s.diagrams.is_empty()).unwrap();
        compile_class(&set, m, evaluated).unwrap()
    }

    #[test]
    fn spin_color_average_is_process_derived() {
        // The averaging denominator must fall out of the incoming legs' spin code
        // and colour dimension — not a hand-coded constant. These pin that
        // hypothesis: a miscount of spin states or colour dimension fails here.
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());

        // q q̄: 2 spin × 3 colour, twice.
        let uu = build_evaluator("u u~ > e+ e-", &m, &evaluated);
        assert_eq!(initial_spin_color_average(&uu, &m, &evaluated), 1.0 / 36.0);
        // e⁺e⁻: 2 spin, colour singlet, twice.
        let ee = build_evaluator("e+ e- > mu+ mu-", &m, &evaluated);
        assert_eq!(initial_spin_color_average(&ee, &m, &evaluated), 1.0 / 4.0);
        // gg: massless vector (2 spin) × adjoint colour (8), twice.
        let gg = build_evaluator("g g > t t~", &m, &evaluated);
        assert_eq!(initial_spin_color_average(&gg, &m, &evaluated), 1.0 / 256.0);
    }

    /// The identical-particle factor must fall out of the outgoing legs, not out of
    /// a table: `g g → g g` is the only MG-validated process with a repeated
    /// outgoing particle, and it is exactly the row whose cross section comes out
    /// twice MadGraph's without the factor.
    #[test]
    fn final_state_symmetry_factor_counts_identical_legs() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let factor =
            |proc: &str| final_state_symmetry_factor(&build_evaluator(proc, &m, &evaluated));
        assert_eq!(factor("g g > g g"), 0.5);
        assert_eq!(factor("g g > t t~"), 1.0);
        assert_eq!(factor("u u~ > u u~"), 1.0);
        assert_eq!(factor("e+ e- > mu+ mu-"), 1.0);
    }

    #[test]
    fn fixed_beam_integrand_finite_positive_2to2() {
        // The flat-RAMBO fixed-energy path on a clean s-channel 2→2 process:
        // a finite, positive σ, with the CM kinematics satisfying the pruned
        // evaluator's ±z-beam frame contract (it would assert otherwise).
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let opts = ParsingOptions::default();
        let card = parse_proc_card("generate e+ e- > mu+ mu-", &opts).unwrap();
        let sets = generate_from_proc_card(&card, &m).unwrap();
        let evals = compile_subprocesses(&sets, &m, &evaluated).unwrap();
        let bounds: Vec<_> = evals
            .iter()
            .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
            .collect();

        let legs = process_external_legs(&evals[0], &m, &evaluated);
        let cuts = Cuts::compile(&RunCard::default(), &legs).unwrap();
        let masses: Vec<f64> = evals[0].external_particles()[evals[0].n_in()..]
            .iter()
            .map(|&id| evaluated.mass(id))
            .collect();
        let avg = initial_spin_color_average(&evals[0], &m, &evaluated);

        let amps: Vec<&BoundAmplitude<f64>> = bounds.iter().collect();
        let integ = FixedBeamIntegrand::new(amps, &cuts, 500.0, masses, avg);
        assert_eq!(integ.vegas_ndim(), 8);
        let (sigma, err) = integ.integrate(20_000, 4, 0x5EED);
        assert!(sigma.is_finite() && sigma > 0.0, "sigma = {sigma}");
        assert!(err.is_finite() && err >= 0.0, "err = {err}");
    }

    /// Build the fixed-energy integrand builder + diagram list for a fixed-energy
    /// process at `sqrt_s`, holding the amplitude/cut/model state the closure borrows.
    fn fixed_energy_case(
        proc: &str,
    ) -> (
        std::sync::Arc<UFOModel>,
        EvaluatedModel,
        Vec<DiagramSet>,
        Vec<Diagram>,
    ) {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let opts = ParsingOptions::default();
        let card = parse_proc_card(&format!("generate {proc}"), &opts).unwrap();
        let sets = generate_from_proc_card(&card, &m).unwrap();
        let diagrams: Vec<Diagram> = sets
            .iter()
            .flat_map(|s| s.diagrams.iter().cloned())
            .collect();
        (m, evaluated, sets, diagrams)
    }

    /// Diagnostic: does the `from_diagram` multichannel reproduce the *phase-space
    /// volume* (integrand ≡ 1) for a real process? An unbiased combiner must, for
    /// any channel set — so a volume that departs from the analytic massless `V_n`
    /// localises a density/reciprocity bug in the per-diagram tree, independent of
    /// |M|². Run with `--ignored --nocapture`.
    #[test]
    #[ignore]
    fn probe_from_diagram_volume() {
        use crate::phasespace::rambo::massless_volume;
        use crate::phasespace::rng::SubStream;
        use crate::phasespace::{PhaseSpaceMap, RamboChannel};

        for proc in [
            "e+ e- > mu+ mu-",
            "e+ e- > ta+ ta- H",
            "e+ e- > mu+ mu- a",
            "e+ e- > mu+ mu- ta+ ta-",
        ] {
            let (m, evaluated, _sets, diagrams) = fixed_energy_case(proc);
            let rep_set = crate::diagrams::generate_from_proc_card(
                &parse_proc_card(&format!("generate {proc}"), &ParsingOptions::default()).unwrap(),
                &m,
            )
            .unwrap();
            let n_out = rep_set[0].particles_out.len();
            let sqrt_s = 500.0;
            let masses: Vec<f64> = vec![0.0; n_out];

            let channels: Vec<Box<dyn Channel<f64>>> = diagrams
                .iter()
                .map(|d| {
                    Box::new(DiagramChannel::from_diagram(d, &evaluated, sqrt_s))
                        as Box<dyn Channel<f64>>
                })
                .collect();
            let multi = MultiChannel::uniform(channels);
            let flat = RamboChannel::new(sqrt_s, masses.clone());
            let analytic = massless_volume(sqrt_s, n_out);

            let mc_vol = |map: &dyn PhaseSpaceMap<f64>, stream: u64| -> (f64, f64) {
                let mut s = SubStream::from_stream(0x5107, stream);
                let (mut sum, mut sq) = (0.0, 0.0);
                let nsamp = 2_000_000usize;
                for _ in 0..nsamp {
                    let u = s.uniforms::<f64>(map.ndim());
                    let w = map.sample(&u).weight;
                    sum += w;
                    sq += w * w;
                }
                let mean = sum / nsamp as f64;
                let err = ((sq / nsamp as f64 - mean * mean).max(0.0) / nsamp as f64).sqrt();
                (mean, err)
            };

            let (v_multi, e_multi) = mc_vol(&multi, 1);
            let (v_flat, e_flat) = mc_vol(&flat, 2);
            eprintln!(
                "[{proc}] n={n_out} {} diag | analytic V={analytic:.6e} | \
                 multi {v_multi:.6e} ± {e_multi:.2e} (dev {:+.2e}) | \
                 flat {v_flat:.6e} ± {e_flat:.2e} (dev {:+.2e})",
                diagrams.len(),
                v_multi / analytic - 1.0,
                v_flat / analytic - 1.0,
            );
        }
    }

    /// The production wiring's efficiency win: on a genuinely resonant fixed-energy
    /// process (`e+ e- > ta+ ta- h` at √s = 500 GeV, a Z → τ⁺τ⁻ pole in the τ-pair
    /// invariant) the per-diagram α-adapted [`MultiChannel`] sampler converges to a
    /// sharp σ̂ at a budget where flat RAMBO cannot resolve the pole at all.
    ///
    /// Flat RAMBO is the known-wrong baseline kept running alongside: because it
    /// almost never lands on the narrow peak, at an equal budget it under-counts σ̂ by
    /// orders of magnitude *and* its relative error stays large — the exact failure
    /// that lists this process SKIP for the flat sampler. The load-bearing figure of
    /// merit is therefore *relative* precision at equal budget (a peak-missing flat
    /// run has a small absolute error precisely because its samples are all ≈ 0), and
    /// the multichannel is orders of magnitude more precise. A wrong multichannel
    /// density would show up as a σ̂ that fails to match the (independently
    /// MG-banked) value in the σ gate, not here.
    #[test]
    fn multichannel_resolves_resonant_pole_flat_rambo_misses() {
        let (m, evaluated, sets, diagrams) = fixed_energy_case("e+ e- > ta+ ta- h");
        assert!(!diagrams.is_empty(), "process must enumerate diagrams");

        let evals = compile_subprocesses(&sets, &m, &evaluated).unwrap();
        let bounds: Vec<_> = evals
            .iter()
            .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
            .collect();
        let rep = &evals[0];
        let legs = process_external_legs(rep, &m, &evaluated);
        let cuts = Cuts::compile(&RunCard::default(), &legs).unwrap();
        let masses: Vec<f64> = rep.external_particles()[rep.n_in()..]
            .iter()
            .map(|&id| evaluated.mass(id))
            .collect();
        let avg = initial_spin_color_average(rep, &m, &evaluated);
        let sqrt_s = 500.0;

        let build = || {
            let amps: Vec<&BoundAmplitude<f64>> = bounds.iter().collect();
            FixedBeamIntegrand::new(amps, &cuts, sqrt_s, masses.clone(), avg)
        };

        // Flat RAMBO (the known-wrong baseline for a narrow pole) at a matched budget.
        let flat = build();
        let (sigma_flat, err_flat) = flat.integrate(60_000, 8, 0x5EED_1);

        // Per-diagram multichannel, α-adapted to this integrand, at the same budget.
        let mut multi = build();
        let report = multi
            .use_multichannel(&diagrams, &evaluated, 20_000, 6, 0x5EED_2)
            .expect("resonant process yields channels");
        let (sigma_mc, err_mc) = multi.integrate(60_000, 8, 0x5EED_3);

        let rel_flat = err_flat / sigma_flat.abs().max(1e-300);
        let rel_mc = err_mc / sigma_mc.abs().max(1e-300);
        eprintln!(
            "resonant σ̂(e+e- > ta+ ta- h): flat RAMBO {sigma_flat:.6e} ± {err_flat:.2e} pb \
             ({} dim, rel {rel_flat:.2e}) | multichannel {sigma_mc:.6e} ± {err_mc:.2e} pb \
             ({} dim, {} channels, rel {rel_mc:.2e}) | α = {:?}",
            flat.vegas_ndim(),
            multi.vegas_ndim(),
            diagrams.len(),
            report.trajectory.last().unwrap(),
        );

        assert!(
            sigma_mc.is_finite() && sigma_mc > 0.0,
            "multichannel σ̂ finite positive: {sigma_mc}"
        );
        // The multichannel converged to a sharp estimate.
        assert!(
            rel_mc < 1e-2,
            "multichannel did not converge: rel error {rel_mc:.2e}"
        );
        // The efficiency win at equal budget: the multichannel is far more precise
        // relative to its own estimate than flat RAMBO, which fails to resolve the pole.
        assert!(
            rel_mc < 0.1 * rel_flat,
            "multichannel not decisively more precise than flat RAMBO: \
             rel_mc {rel_mc:.2e} vs rel_flat {rel_flat:.2e}"
        );
        // And flat RAMBO visibly under-counts by missing the peak — the known-wrong
        // baseline firing.
        assert!(
            sigma_flat < 0.5 * sigma_mc,
            "flat RAMBO did not under-count the resonant σ̂ as expected: \
             flat {sigma_flat:.6e} vs multichannel {sigma_mc:.6e}"
        );
    }

    /// The production wiring's unbiasedness: on a fixed-energy process where flat
    /// RAMBO *does* converge (`e+ e- > mu+ mu-` at √s = 200 GeV, smooth and
    /// off-resonance), the per-diagram multichannel sampler integrates to the same
    /// σ̂ within the combined Monte-Carlo error. Swapping the flat map for the
    /// resonance-aware combiner must not move the cross section.
    #[test]
    fn multichannel_unbiased_vs_flat_where_both_converge() {
        let (m, evaluated, sets, diagrams) = fixed_energy_case("e+ e- > mu+ mu-");
        let evals = compile_subprocesses(&sets, &m, &evaluated).unwrap();
        let bounds: Vec<_> = evals
            .iter()
            .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
            .collect();
        let rep = &evals[0];
        let legs = process_external_legs(rep, &m, &evaluated);
        let cuts = Cuts::compile(&RunCard::default(), &legs).unwrap();
        let masses: Vec<f64> = rep.external_particles()[rep.n_in()..]
            .iter()
            .map(|&id| evaluated.mass(id))
            .collect();
        let avg = initial_spin_color_average(rep, &m, &evaluated);
        let sqrt_s = 200.0;

        let build = || {
            let amps: Vec<&BoundAmplitude<f64>> = bounds.iter().collect();
            FixedBeamIntegrand::new(amps, &cuts, sqrt_s, masses.clone(), avg)
        };

        let flat = build();
        let (sigma_flat, err_flat) = flat.integrate(60_000, 8, 0x5EED_4);

        let mut multi = build();
        multi
            .use_multichannel(&diagrams, &evaluated, 20_000, 6, 0x5EED_5)
            .expect("process yields channels");
        let (sigma_mc, err_mc) = multi.integrate(60_000, 8, 0x5EED_6);

        eprintln!(
            "convergent σ̂(e+e- > mu+ mu- @200): flat RAMBO {sigma_flat:.6e} ± {err_flat:.2e} pb | \
             multichannel {sigma_mc:.6e} ± {err_mc:.2e} pb ({} channels)",
            diagrams.len(),
        );

        let comb = (err_flat * err_flat + err_mc * err_mc).sqrt();
        assert!(
            (sigma_flat - sigma_mc).abs() < 5.0 * comb,
            "multichannel σ̂ {sigma_mc:.6e} ± {err_mc:.2e} disagrees with flat RAMBO \
             {sigma_flat:.6e} ± {err_flat:.2e} (5σ = {:.2e})",
            5.0 * comb
        );
    }

    /// Build the flat-map fixed-energy integrand for a coloured `2 → 2` process,
    /// returning it alongside the amplitude the selections are checked against.
    fn colored_2to2_case(
        m: &UFOModel,
        evaluated: &EvaluatedModel,
        sqrt_s: f64,
    ) -> (Vec<AmplitudeEvaluator>, Cuts, Vec<f64>, f64) {
        let opts = ParsingOptions::default();
        let card = parse_proc_card("generate u u~ > u u~", &opts).unwrap();
        let sets = generate_from_proc_card(&card, m).unwrap();
        let evals = compile_subprocesses(&sets, m, evaluated).unwrap();
        let rep = &evals[0];
        let legs = process_external_legs(rep, m, evaluated);
        // The default card's jet cuts keep the t-channel singularity out of the
        // sampled region, so the probe point below is an ordinary phase-space point.
        let cuts = Cuts::compile(&RunCard::default(), &legs).unwrap();
        let masses: Vec<f64> = rep.external_particles()[rep.n_in()..]
            .iter()
            .map(|&id| evaluated.mass(id))
            .collect();
        let avg = initial_spin_color_average(rep, m, evaluated);
        let _ = sqrt_s;
        (evals, cuts, masses, avg)
    }

    /// The per-event helicity and colour-flow draws must reproduce the diagonals
    /// they read — the property `SELECT_HEL` and `SELECT_COLOR` rest on, and the one
    /// a wrong accumulator or a mis-indexed draw would break while still returning
    /// perfectly plausible labels.
    #[test]
    fn selected_helicity_and_flow_frequencies_follow_the_diagonals() {
        use crate::phasespace::rng::SubStream;

        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let sqrt_s = 500.0;
        let (evals, cuts, masses, avg) = colored_2to2_case(&m, &evaluated, sqrt_s);
        let bounds: Vec<_> = evals
            .iter()
            .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
            .collect();
        let amps: Vec<&BoundAmplitude<f64>> = bounds.iter().collect();
        let integ = FixedBeamIntegrand::new(amps, &cuts, sqrt_s, masses, avg);

        // One ordinary phase-space point of the flat map.
        let u: Vec<f64> = (0..integ.channel_grid_ndim())
            .map(|k| 0.11 + 0.07 * (k as f64 % 7.0))
            .collect();
        let mut momenta = Vec::new();
        let value = integ.event_in_channel(0, &u, &mut momenta);
        assert!(value > 0.0, "the probe point must pass the cuts");

        // The diagonals the draws are supposed to follow, taken directly.
        let eval = integ.subprocess_evaluator(0);
        let mut scratch = bounds[0].scratch_space();
        let mut ext = integ.beams().to_vec();
        ext.extend_from_slice(&momenta);
        let mut hel_m2 = vec![0.0; eval.helicities().len()];
        let mut jamp2 = vec![0.0; eval.n_flows()];
        bounds[0].eval_hel_m2(&ext, &mut scratch, &mut hel_m2);
        bounds[0].eval_jamp2(&ext, &mut scratch, &mut jamp2);
        assert!(
            eval.n_flows() > 1,
            "a single flow makes the colour draw vacuous"
        );
        assert!(
            hel_m2.iter().filter(|&&w| w > 0.0).count() > 1,
            "a single contributing helicity makes the helicity draw vacuous"
        );

        let n = 200_000;
        let mut hel_counts = vec![0usize; hel_m2.len()];
        let mut flow_counts = vec![0usize; jamp2.len()];
        let mut s = SubStream::from_stream(0x5E1E_C701, 3);
        for _ in 0..n {
            let sel = integ
                .select_event(
                    &momenta,
                    [
                        s.next_uniform::<f64>(),
                        s.next_uniform::<f64>(),
                        s.next_uniform::<f64>(),
                    ],
                )
                .expect("a point with weight carries labels");
            let c = eval
                .helicities()
                .iter()
                .position(|h| h.as_slice() == sel.helicity.as_slice())
                .expect("a selected helicity is one of the combinations");
            hel_counts[c] += 1;
            flow_counts[sel.flow] += 1;
        }

        let check = |counts: &[usize], weights: &[f64], what: &str| {
            let total: f64 = weights.iter().sum();
            for (i, (&c, &w)) in counts.iter().zip(weights).enumerate() {
                let p = w / total;
                let f = c as f64 / n as f64;
                let sigma = (p * (1.0 - p) / n as f64).sqrt();
                assert!(
                    (f - p).abs() <= 5.0 * sigma + 1e-12,
                    "{what} {i}: frequency {f:.5} vs {p:.5} (5σ = {:.5})",
                    5.0 * sigma
                );
            }
        };
        check(&hel_counts, &hel_m2, "helicity");
        check(&flow_counts, &jamp2, "flow");
    }

    /// Selection must be provably neutral, not neutral by construction: the same
    /// accept/reject pass, driven from the same RNG, must produce bit-for-bit the
    /// same trials, the same accepted points and the same cross section whether or
    /// not every accepted event is labelled along the way.
    ///
    /// This fires on the two ways a selection can leak — consuming from the
    /// sampling stream, and leaving the amplitude's per-event state (its coupling,
    /// its scratch) different from how the integrand left it.
    #[test]
    fn labelling_events_moves_neither_the_trials_nor_the_cross_section() {
        use crate::phasespace::rng::SubStream;
        use crate::unweight::Unweighter;

        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let sqrt_s = 500.0;
        let (evals, cuts, masses, avg) = colored_2to2_case(&m, &evaluated, sqrt_s);
        let bounds: Vec<_> = evals
            .iter()
            .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
            .collect();
        let amps: Vec<&BoundAmplitude<f64>> = bounds.iter().collect();
        let mut integ = FixedBeamIntegrand::new(amps, &cuts, sqrt_s, masses, avg);
        let card = parse_proc_card("generate u u~ > u u~", &ParsingOptions::default()).unwrap();
        let diagrams: Vec<Diagram> = generate_from_proc_card(&card, &m)
            .unwrap()
            .iter()
            .flat_map(|s| s.diagrams.iter().cloned())
            .collect();
        integ.use_multichannel(&diagrams, &evaluated, 5_000, 4, 0x5EED_0);

        let (channels, _) = integ.adapt_grids(4_000, 3, 0x5EED);
        let build = || {
            Unweighter::scan(
                &integ,
                channels.iter().map(|c| (&c.grid, 20_000)),
                0x5CA7_0FF1,
            )
        };

        let mut labelled = build();
        let mut plain = build();
        let mut rng_a = rand_chacha::ChaCha8Rng::seed_from_u64(0xE7E7);
        let mut rng_b = rand_chacha::ChaCha8Rng::seed_from_u64(0xE7E7);
        // The labels ride on their own stream, the discipline a generator follows.
        let mut labels = SubStream::from_stream(0x5E1E_C702, 3);
        let mut momenta = Vec::new();
        let mut seen_helicities = std::collections::BTreeSet::new();
        let mut seen_flows = std::collections::BTreeSet::new();

        for _ in 0..20_000 {
            let a = labelled.trial(&integ, &mut rng_a);
            let b = plain.trial(&integ, &mut rng_b);
            match (&a, &b) {
                (Some(x), Some(y)) => {
                    assert_eq!(x.channel, y.channel);
                    assert_eq!(x.u, y.u);
                    assert_eq!(x.weight, y.weight);
                }
                (None, None) => {}
                _ => panic!("labelling changed which trials were accepted"),
            }
            if let Some(point) = a {
                integ.event_in_channel(point.channel, &point.u, &mut momenta);
                let sel = integ
                    .select_event(
                        &momenta,
                        [
                            labels.next_uniform::<f64>(),
                            labels.next_uniform::<f64>(),
                            labels.next_uniform::<f64>(),
                        ],
                    )
                    .expect("an accepted point carries labels");
                seen_helicities.insert(sel.helicity.clone());
                seen_flows.insert(sel.flow);
            }
        }

        let (sa, sb) = (labelled.stats(), plain.stats());
        assert_eq!(sa.trials, sb.trials);
        assert_eq!(sa.accepted, sb.accepted);
        assert_eq!(sa.ratio_sum, sb.ratio_sum, "the weight sum moved");
        assert_eq!(
            labelled.sigma_from_events(),
            plain.sigma_from_events(),
            "labelling moved the cross section"
        );
        assert!(
            sa.accepted > 100,
            "too few events for the check to mean much"
        );
        // A run that only ever produced one label would satisfy the above vacuously.
        assert!(seen_helicities.len() > 1 && seen_flows.len() > 1);
    }

    fn tiny_pdf() -> PdfMember {
        use crate::pdf::grid::SubGrid;
        // A single-knot-band synthetic grid covering the DY x/Q² region with a
        // constant x·f; only used where the integrand must short-circuit before
        // consuming the PDF, so the values are immaterial.
        let flavors = vec![-4, -3, -2, -1, 1, 2, 3, 4, 21];
        let nx = 2;
        let nq = 2;
        let xf = vec![0.1; nx * nq * flavors.len()];
        PdfMember::from_subgrids(vec![SubGrid {
            x: vec![1e-7, 1.0],
            q2: vec![1.0, 1e8],
            flavors,
            xf,
        }])
    }
}
