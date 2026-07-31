//! Flavour decomposition of a hadron-collider process: which of its enumerated
//! subprocesses share one matrix element, and which parton-distribution
//! luminosity each of them carries.
//!
//! A proc card written with beam multiparticles (`p p > l+ l- j`) expands into
//! many concrete subprocesses, most of which are the *same* partonic calculation
//! with a different flavour label on the incoming legs. Compiling and evaluating
//! each of them separately would be arithmetic wasted on a distinction the matrix
//! element does not make. [`derive_flavor_groups`] partitions them into
//! [`FlavorGroup`]s — one compiled amplitude, one phase-space map, one cut filter
//! per group — and gives each group the summed luminosity of the flavours inside
//! it.
//!
//! # The grouping rule
//!
//! Two subprocesses join a group when their `|M|²` agree, to
//! [`GROUP_REL_TOL`], at a shared set of probe phase-space points spanning
//! several partonic energies. Nothing is hand-listed: the coupling classes
//! (up-type vs down-type), the generation copies (`u`/`c`, `d`/`s`), the lepton
//! flavours (`e`/`mu`) and the separation of `q g` from `q̄ g` all fall out of
//! the measurement. Grouping is refused unless the members also share the
//! *outgoing pole masses* (so one phase-space map serves the group), an equal
//! [`Cuts`] filter (so one cut indicator does) and an equal colour basis (so an
//! event's colour flow can be read off the representative), and unless distinct
//! groups separate by more than [`GROUP_SEPARATION_MIN`] — a partition resting
//! on a knife edge is a measurement that failed, not a decomposition.
//!
//! The extra requirements are there because `|M|²` is a sum: it is blind to a
//! global phase, and it would not move if two members' colour bases differed by a
//! relabelling. `|M|²` equality alone is therefore not enough to license reusing
//! one member's colour flow, cuts or phase-space map for another.
//!
//! # Both beam orderings
//!
//! Diagram enumeration emits **one ordering per unordered initial state**: `g u`
//! is generated, `u g` is not. Both are physical — the parton distributions of
//! the two beams are evaluated at different momentum fractions — so the missing
//! ordering must be restored, and it is not free. The identity is
//!
//! ```text
//! |M_{b a}(p₁, p₂, q)|² = |M_{a b}(p₁, p₂, R q)|²,   R: (E, pₓ, p_y, p_z) ↦ (E, pₓ, −p_y, −p_z)
//! ```
//!
//! `R` is the rotation by π about the x axis, which maps a partonic-CM beam
//! momentum onto the other beam's (`R p₁ = p₂`); rotating the mirrored
//! configuration by it puts the beams back where the amplitude expects them and
//! leaves the outgoing legs reflected. So a group contributes
//!
//! ```text
//! xf_a(x₁)·xf_b(x₂)·|M(q)|²  +  xf_b(x₁)·xf_a(x₂)·|M(R q)|²
//! ```
//!
//! under one cut indicator: `R` is an argument to the matrix element, not a
//! change to the event, whose final state stays `q`. [`FlavorGroup::mirror_into`]
//! builds the reflected argument and [`FlavorGroup::luminosity`] returns the two
//! sums. An initial state with `a == b` has only one ordering and contributes no
//! mirror term.
//!
//! # What this module does not decide
//!
//! The identical-particle symmetry factor is *asserted* to be one per subprocess
//! rather than carried, because a summed matrix element has no single owner for
//! it: the factor belongs to the final state, and a sum over subprocesses with
//! different final states would need it per term.
//!
//! # The integrand
//!
//! [`ProtonIntegrand`] convolves the decomposition with parton distributions over
//! a `(τ, y)` outer map and a per-diagram multichannel inner map, and presents the
//! result as a [`ChannelIntegrand`] so per-channel VEGAS banking, frozen-grid scans
//! and accept/reject need no hadronic special case. Its master formula, frames and
//! change of variables are documented on the type.

use std::cell::{Cell, RefCell};
use std::f64::consts::PI;

use rand::SeedableRng;
use thiserror::Error;

use crate::coupling::alphas::AlphaSSource;
use crate::coupling::scales::{EventScales, ScaleError};
use crate::cuts::{CutError, Cuts, ExternalLeg};
use crate::diagrams::diagram::Diagram;
use crate::diagrams::DiagramSet;
use crate::hadronic::{
    boost_z, channel_neval, combine_channels, compile_class, compile_scale_source, components,
    constant_scale_report, final_state_symmetry_factor, initial_spin_color_average,
    make_subs_scale_aware, process_external_legs, BoundSubprocess, ChannelIntegration,
    EventScaleSource, HadronicError, RunningCouplingReport, CHANNEL_STREAM_BASE, SCALE_PROBE_DRAWS,
    SCALE_PROBE_SEED, VEGAS_ALPHA_MAPPED, VEGAS_NBINS,
};
use crate::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
use crate::helas::repr::lorentz::LorentzVector;
use crate::pdf::grid::AlphaSInfo;
use crate::pdf::PdfMember;
use crate::phasespace::rng::SubStream;
use crate::phasespace::{
    kleiss_pittau_step, AlphaAdaptation, DiagramChannel, PhaseSpaceMap, RamboChannel,
    ScaledChannel, ScaledMultiChannel, GEV2_TO_PB,
};
use crate::runcard::RunCard;
use crate::ufo::{EvaluatedModel, UFOModel};
use crate::unweight::ChannelIntegrand;
use crate::vegas::{VegasGrid, VegasResult};

type V = LorentzVector<f64>;

/// Relative agreement at which two subprocesses' `|M|²` count as one group's.
///
/// Members of a group are the *same* arithmetic on differently-labelled legs, so
/// the observed agreement is exact (measured: bit-for-bit over every `p p → ℓℓj`
/// group). The bound is loose against that and still eleven orders below the
/// closest measured separation between distinct groups.
pub const GROUP_REL_TOL: f64 = 1e-10;

/// Minimum relative separation between two distinct groups at their best-separated
/// probe point.
///
/// Without it a partition could be produced by two subprocesses landing either
/// side of [`GROUP_REL_TOL`] by rounding, which is not a physical distinction.
/// Measured worst separation over `p p → ℓℓj`: `0.69`.
pub const GROUP_SEPARATION_MIN: f64 = 1e-6;

/// Probe points drawn at each of [`probe_energies`]' three energies.
const PROBE_POINTS_PER_ENERGY: usize = 4;

/// RNG stream the probe draws from, distinct from any integration stream.
const PROBE_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

/// One concrete external flavour assignment inside a group: the subprocess an
/// event of this group is finally labelled with.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Subprocess {
    /// PDG codes of the two incoming partons, in the enumerated beam order.
    pub incoming: [i32; 2],
    /// PDG codes of the outgoing legs, in the group's shared leg order.
    pub outgoing: Vec<i32>,
}

impl Subprocess {
    /// Whether the two beams carry different partons, and so whether the mirrored
    /// ordering is a second physical initial state rather than the same one.
    pub fn has_mirror(&self) -> bool {
        self.incoming[0] != self.incoming[1]
    }
}

/// A set of subprocesses sharing one matrix element, one phase-space map and one
/// cut filter, differing only in the parton-distribution luminosity they carry.
pub struct FlavorGroup {
    representative: DiagramSet,
    evaluator: AmplitudeEvaluator,
    legs: Vec<ExternalLeg>,
    cuts: Cuts,
    members: Vec<Subprocess>,
    spin_color_avg: f64,
}

impl FlavorGroup {
    /// The compiled, helicity-pruned amplitude every member shares.
    pub fn evaluator(&self) -> &AmplitudeEvaluator {
        &self.evaluator
    }

    /// The representative subprocess's diagrams — the input the per-diagram
    /// phase-space channels are derived from.
    pub fn diagrams(&self) -> &[Diagram] {
        &self.representative.diagrams
    }

    /// The representative subprocess's enumerated particle names.
    pub fn diagram_set(&self) -> &DiagramSet {
        &self.representative
    }

    /// External legs in process order (incoming first), for the representative.
    pub fn external_legs(&self) -> &[ExternalLeg] {
        &self.legs
    }

    /// The cut filter every member compiles to.
    pub fn cuts(&self) -> &Cuts {
        &self.cuts
    }

    /// Concrete flavour assignments summed into this group.
    pub fn members(&self) -> &[Subprocess] {
        &self.members
    }

    /// `1 / Π_a (n_spin · n_colour)` over the incoming legs.
    pub fn spin_color_average(&self) -> f64 {
        self.spin_color_avg
    }

    /// Outgoing pole masses in leg order — the phase-space map's targets.
    pub fn final_masses(&self) -> Vec<f64> {
        self.legs[self.n_in()..].iter().map(|l| l.mass).collect()
    }

    /// Number of incoming legs; two for every hadronic subprocess.
    pub fn n_in(&self) -> usize {
        self.evaluator.n_in()
    }

    /// Whether any member has distinct beam partons, i.e. whether the mirrored
    /// ordering carries luminosity at all. A group of identical-parton initial
    /// states (`g g`) has one ordering and no mirror term to evaluate.
    pub fn has_mirror(&self) -> bool {
        self.members.iter().any(Subprocess::has_mirror)
    }

    /// The argument the shared `|M|²` takes for the mirrored beam ordering:
    /// the beams untouched, the outgoing legs rotated by π about the x axis.
    ///
    /// The rotation maps each beam momentum onto the other's, so evaluating the
    /// group's amplitude here is evaluating the subprocess with its two incoming
    /// flavours exchanged. `out` is overwritten and may be a reused buffer.
    pub fn mirror_into(&self, momenta: &[V], out: &mut Vec<V>) {
        out.clear();
        out.extend_from_slice(&momenta[..self.n_in()]);
        out.extend(
            momenta[self.n_in()..]
                .iter()
                .map(|p| V::new(p.e(), p.px(), -p.py(), -p.pz())),
        );
    }

    /// Summed `x·f` luminosity of the group, as `[direct, mirror]`:
    ///
    /// ```text
    /// direct = Σ_members xf_a(x₁, μ_F1) · xf_b(x₂, μ_F2)
    /// mirror = Σ_members xf_b(x₁, μ_F1) · xf_a(x₂, μ_F2)     (a ≠ b only)
    /// ```
    ///
    /// `direct` weights `|M(q)|²` and `mirror` weights `|M(R q)|²`
    /// ([`mirror_into`](Self::mirror_into)). `mu_f` is per beam in beam order,
    /// since MadGraph's `q2fact(1)` and `q2fact(2)` need not coincide.
    ///
    /// A member whose beams carry the same parton has a single ordering, so it
    /// contributes to `direct` only — counting it twice would double that
    /// subprocess's cross section.
    pub fn luminosity(&self, pdf: &PdfMember, x1: f64, x2: f64, mu_f: [f64; 2]) -> [f64; 2] {
        let mut sums = [0.0; 2];
        for i in 0..self.members.len() {
            let m = self.member_luminosity(i, pdf, x1, x2, mu_f);
            sums[0] += m[0];
            sums[1] += m[1];
        }
        sums
    }

    /// One member's `[direct, mirror]` luminosity — the share that decides which
    /// concrete flavour an accepted event of this group is labelled with.
    pub fn member_luminosity(
        &self,
        member: usize,
        pdf: &PdfMember,
        x1: f64,
        x2: f64,
        mu_f: [f64; 2],
    ) -> [f64; 2] {
        let q2 = [mu_f[0] * mu_f[0], mu_f[1] * mu_f[1]];
        let [a, b] = self.members[member].incoming;
        let direct = pdf.xfx_q2(a, x1, q2[0]) * pdf.xfx_q2(b, x2, q2[1]);
        let mirror = if a == b {
            0.0
        } else {
            pdf.xfx_q2(b, x1, q2[0]) * pdf.xfx_q2(a, x2, q2[1])
        };
        [direct, mirror]
    }
}

/// The flavour decomposition of one hadronic process.
pub struct FlavorGroups {
    groups: Vec<FlavorGroup>,
}

impl FlavorGroups {
    pub fn groups(&self) -> &[FlavorGroup] {
        &self.groups
    }

    /// Total number of concrete subprocesses summed, over all groups — the count
    /// the enumeration produced.
    pub fn subprocess_count(&self) -> usize {
        self.groups.iter().map(|g| g.members.len()).sum()
    }
}

#[derive(Debug, Error)]
pub enum ProtonError {
    #[error(transparent)]
    Hadronic(#[from] HadronicError),
    #[error("cut compilation failed: {0}")]
    Cut(#[from] CutError),
    #[error("the enumeration produced no subprocess with diagrams")]
    NoSubprocess,
    #[error("subprocess {process} has {n_in} incoming legs; a beam decomposition needs 2")]
    NotTwoIncoming { process: String, n_in: usize },
    #[error(
        "subprocess {process} has an incoming leg of mass {mass} GeV; a parton distribution \
         supplies massless partons and the probe puts the beams on the light cone"
    )]
    MassiveInitialState { process: String, mass: f64 },
    #[error(
        "subprocess {process} carries an identical-particle symmetry factor {factor}, not 1; a \
         matrix element summed over subprocesses has no single owner for a final-state factor, \
         so each term would have to carry its own"
    )]
    IdenticalFinalState { process: String, factor: f64 },
    #[error(
        "subprocesses {a} and {b} have different outgoing masses, so no single phase-space map \
         covers the sum"
    )]
    UnequalFinalMasses { a: String, b: String },
    #[error(
        "subprocesses {a} and {b} share a matrix element but compile to different cut filters, \
         so no single cut indicator covers the sum"
    )]
    CutIndicatorDiffers { a: String, b: String },
    #[error(
        "subprocesses {a} and {b} share a matrix element but not a colour basis, so the flow an \
         event of {b} is labelled with cannot be read off {a}"
    )]
    ColorStructureDiffers { a: String, b: String },
    #[error(
        "the groups represented by {a} and {b} separate by only {rel:.3e} at their best-separated \
         probe point; a partition that close is rounding, not a coupling distinction"
    )]
    DegenerateGroups { a: String, b: String, rel: f64 },
    #[error(
        "groups {a} and {b} compile to different cut filters, so the sum has no single phase-space \
         indicator; a process whose groups are cut differently needs one indicator per group"
    )]
    GroupCutsDiffer { a: usize, b: usize },
    #[error(
        "{amps} bound amplitudes were supplied for {groups} flavour groups, in an order that must \
         match one for one"
    )]
    AmplitudeCount { amps: usize, groups: usize },
    #[error(
        "the bound amplitude at position {index} was not built from group {index}'s evaluator, so \
         the luminosity of one group would weight another's matrix element"
    )]
    AmplitudeMismatch { index: usize },
}

/// Partonic energies the grouping probe measures at, spread over more than a
/// decade so a coincidence at one energy cannot survive.
///
/// Scaled by the outgoing pole masses so a heavy final state is probed above its
/// threshold, with a floor for the massless case.
fn probe_energies(final_masses: &[f64]) -> [f64; 3] {
    let base = final_masses.iter().sum::<f64>().max(100.0);
    [3.0 * base, 5.0 * base, 13.0 * base]
}

/// Partonic-CM probe points: massless beams along ±z and a flat RAMBO draw over
/// the outgoing legs, at each of [`probe_energies`].
fn probe_momenta(final_masses: &[f64], seed: u64) -> Vec<Vec<V>> {
    let mut points = Vec::new();
    for (i, sqrt_s) in probe_energies(final_masses).into_iter().enumerate() {
        let rambo = RamboChannel::<f64>::new(sqrt_s, final_masses.to_vec());
        let mut stream = SubStream::from_stream(seed, i as u64);
        for _ in 0..PROBE_POINTS_PER_ENERGY {
            let u = stream.uniforms::<f64>(rambo.ndim());
            let drawn = rambo.sample(&u);
            let e = sqrt_s / 2.0;
            let mut momenta = vec![V::new(e, 0.0, 0.0, e), V::new(e, 0.0, 0.0, -e)];
            momenta.extend(drawn.momenta.iter().cloned());
            points.push(momenta);
        }
    }
    points
}

/// Worst relative disagreement between two probe traces.
fn worst_rel(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).abs() / x.abs().max(y.abs()).max(f64::MIN_POSITIVE))
        .fold(0.0f64, f64::max)
}

/// `u u~ > e+ e- g`-style label for an enumerated subprocess.
fn label(set: &DiagramSet) -> String {
    format!(
        "{} > {}",
        set.particles_in.join(" "),
        set.particles_out.join(" ")
    )
}

/// Partition a hadronic enumeration into flavour groups.
///
/// `sets` are the `DiagramSet`s of one proc card (empty ones ignored), `card` the
/// run card whose cuts every member must compile to alike. See the module
/// documentation for the rule and for what it refuses.
pub fn derive_flavor_groups(
    sets: Vec<DiagramSet>,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    card: &RunCard,
) -> Result<FlavorGroups, ProtonError> {
    let sets: Vec<DiagramSet> = sets
        .into_iter()
        .filter(|s| !s.diagrams.is_empty())
        .collect();
    if sets.is_empty() {
        return Err(ProtonError::NoSubprocess);
    }

    let labels: Vec<String> = sets.iter().map(label).collect();

    let mut compiled = Vec::with_capacity(sets.len());
    for (set, process) in sets.iter().zip(&labels) {
        let evaluator = compile_class(set, model, evaluated)?;
        if evaluator.n_in() != 2 {
            return Err(ProtonError::NotTwoIncoming {
                process: process.clone(),
                n_in: evaluator.n_in(),
            });
        }
        let legs = process_external_legs(&evaluator, model, evaluated);
        if let Some(leg) = legs[..2].iter().find(|l| l.mass != 0.0) {
            return Err(ProtonError::MassiveInitialState {
                process: process.clone(),
                mass: leg.mass,
            });
        }
        let factor = final_state_symmetry_factor(&evaluator);
        if factor != 1.0 {
            return Err(ProtonError::IdenticalFinalState {
                process: process.clone(),
                factor,
            });
        }
        let cuts = Cuts::compile(card, &legs)?;
        compiled.push((evaluator, legs, cuts));
    }

    let final_masses: Vec<f64> = compiled[0].1[2..].iter().map(|l| l.mass).collect();
    for (i, (_, legs, _)) in compiled.iter().enumerate().skip(1) {
        let masses: Vec<f64> = legs[2..].iter().map(|l| l.mass).collect();
        if masses != final_masses {
            return Err(ProtonError::UnequalFinalMasses {
                a: labels[0].clone(),
                b: labels[i].clone(),
            });
        }
    }

    let points = probe_momenta(&final_masses, PROBE_SEED);
    let traces: Vec<Vec<f64>> = compiled
        .iter()
        .map(|(evaluator, ..)| {
            let bound = BoundAmplitude::<f64>::bind(evaluator, evaluated);
            let mut scratch = bound.scratch_space();
            points
                .iter()
                .map(|k| bound.eval_m2(k, &mut scratch))
                .collect()
        })
        .collect();

    let mut partition: Vec<Vec<usize>> = Vec::new();
    for i in 0..compiled.len() {
        match partition
            .iter_mut()
            .find(|g| worst_rel(&traces[g[0]], &traces[i]) < GROUP_REL_TOL)
        {
            Some(group) => group.push(i),
            None => partition.push(vec![i]),
        }
    }

    for (a, ga) in partition.iter().enumerate() {
        for gb in &partition[a + 1..] {
            let rel = worst_rel(&traces[ga[0]], &traces[gb[0]]);
            if rel <= GROUP_SEPARATION_MIN {
                return Err(ProtonError::DegenerateGroups {
                    a: labels[ga[0]].clone(),
                    b: labels[gb[0]].clone(),
                    rel,
                });
            }
        }
    }

    let mut sets: Vec<Option<DiagramSet>> = sets.into_iter().map(Some).collect();
    let mut compiled: Vec<Option<_>> = compiled.into_iter().map(Some).collect();
    let mut groups = Vec::with_capacity(partition.len());
    for indices in &partition {
        let head = indices[0];
        let (head_eval, _, head_cuts) = compiled[head].as_ref().expect("group head unclaimed");
        for &i in &indices[1..] {
            let (member_eval, _, member_cuts) = compiled[i].as_ref().expect("member unclaimed");
            if member_cuts != head_cuts {
                return Err(ProtonError::CutIndicatorDiffers {
                    a: labels[head].clone(),
                    b: labels[i].clone(),
                });
            }
            // An event is labelled with its group representative's colour-flow
            // basis, so members must share it. `|M|²` on its own does not imply
            // they do: it is a sum over the basis and would not move if two
            // members' bases differed by a relabelling.
            if member_eval.n_flows() != head_eval.n_flows()
                || member_eval.cf_matrix() != head_eval.cf_matrix()
            {
                return Err(ProtonError::ColorStructureDiffers {
                    a: labels[head].clone(),
                    b: labels[i].clone(),
                });
            }
        }
        let members = indices
            .iter()
            .map(|&i| {
                let legs = &compiled[i].as_ref().expect("member unclaimed").1;
                Subprocess {
                    incoming: [legs[0].pdg, legs[1].pdg],
                    outgoing: legs[2..].iter().map(|l| l.pdg).collect(),
                }
            })
            .collect();
        let (evaluator, legs, cuts) = compiled[head].take().expect("group head unclaimed");
        let spin_color_avg = initial_spin_color_average(&evaluator, model, evaluated);
        groups.push(FlavorGroup {
            representative: sets[head].take().expect("group head unclaimed"),
            evaluator,
            legs,
            cuts,
            members,
            spin_color_avg,
        });
    }

    Ok(FlavorGroups { groups })
}

/// Unit-hypercube coordinates the `(τ, y)` outer map consumes. They are *prepended*
/// to each channel's own coordinates, so a channel grid is over
/// `OUTER_NDIM + channel_ndim`.
const OUTER_NDIM: usize = 2;

/// RNG substream the channel-weight survey draws on, kept distinct from the
/// per-channel integration streams so the survey and the integral neither share nor
/// correlate their sequences.
const ADAPT_STREAM: u64 = 0xA1FA_9110;

/// Which diagram of which flavour group a sampling channel was derived from.
///
/// The channels of every group are pooled into one mixture, so a channel is
/// identified by the pair rather than by a diagram index alone — the key a banked
/// per-channel grid is stored under.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelId {
    pub group: usize,
    pub diagram: usize,
}

/// A VEGAS point's outer coordinates, mapped to the partonic system.
struct OuterPoint {
    x1: f64,
    x2: f64,
    sqrt_shat: f64,
    /// `dτ dy / (du₀ du₁)` with the `1/τ` from `f = (x·f)/x` on both legs already
    /// divided out.
    jac: f64,
}

/// A ready-to-integrate hadronic cross section for an arbitrary flavour-decomposed
/// process at proton beams (`lpp = 1`), sampled by a per-diagram multichannel map at
/// each event's own partonic energy.
///
/// # Master formula
///
/// ```text
/// σ = ∫ dτ dy dΦ_n  Σ_g avg_g · [ L_g^direct(x₁,x₂,μF) |M_g(q)|²
///                               + L_g^mirror(x₁,x₂,μF) |M_g(Rq)|² ] · Θ_cuts(q) / (2ŝ)
/// ```
///
/// summed over the [`FlavorGroup`]s of the process, with `L^direct`/`L^mirror` the
/// group's two summed beam orderings ([`FlavorGroup::luminosity`]) and `R` the
/// mirror map ([`FlavorGroup::mirror_into`]). There is **one** cut indicator, on the
/// unreflected final state: the mirror is an argument to the matrix element, not a
/// second event. The identical-particle symmetry factor is one by construction —
/// [`derive_flavor_groups`] refuses a process where it is not.
///
/// # Change of variables
///
/// The outer two coordinates are Drell–Yan's, and for the same reason: a dilepton
/// mass window is a one-dimensional bound on `τ` rather than a thin diagonal band in
/// `(x₁, x₂)`.
///
/// ```text
/// τ = τ_min^(1−u₀)   (ln τ uniform),   dτ/du₀ = τ · ln(1/τ_min)
/// y = (2u₁ − 1)·y_max,  y_max = ½ ln(1/τ),   dy/du₁ = 2·y_max
/// ```
///
/// with `τ = ŝ/s = x₁x₂`, `y = ½ ln(x₁/x₂)` and `τ_min = ŝ_min/s` from
/// [`Cuts::shat_min`]. Since the grids return `x·f(x)` and `x₁x₂ = τ` matches the
/// `dτ` Jacobian, the `1/x₁x₂` in `f = (x·f)/x` cancels the `τ` and the luminosity is
/// built directly from `x·f` products, leaving the bare `ln(1/τ_min)·2·y_max`.
///
/// The remaining `3n−4` coordinates are the multichannel's, evaluated at the event's
/// own `√ŝ = √(τ s)` ([`ScaledMultiChannel`]) — the channel trees are `√ŝ`-independent
/// structures, so nothing is rebuilt per point.
///
/// # Frames
///
/// The matrix element is evaluated in the **partonic CM** with the beams along ±z,
/// the frame the helicity-pruned [`BoundAmplitude::eval_m2`] requires and the frame
/// the channel maps generate in. The cut filter and the scale prescription operate in
/// the **lab frame**, so the outgoing momenta are boosted along z by the partonic
/// rapidity `y` first.
pub struct ProtonIntegrand<'a> {
    groups: &'a FlavorGroups,
    subs: Vec<BoundSubprocess<'a>>,
    pdf: &'a PdfMember,
    /// The one cut filter every group compiles to.
    cuts: &'a Cuts,
    combiner: ScaledMultiChannel<f64>,
    channel_ids: Vec<ChannelId>,
    /// Total hadronic invariant `s = (E₁+E₂)²` (head-on beams).
    s_had: f64,
    sqrt_s_had: f64,
    /// Lower support of the logarithmic `τ = ŝ/s` map, `ŝ_min/s`.
    tau_min: f64,
    ln_inv_tau_min: f64,
    /// The `(2π)^{4−3n}` measure factor.
    lips_2pi: f64,
    scales: EventScaleSource,
    scale_buf: RefCell<Vec<[f64; 4]>>,
    cm_buf: RefCell<Vec<V>>,
    lab_buf: RefCell<Vec<V>>,
    mirror_buf: RefCell<Vec<V>>,
    /// The last `(μR, αs(μR))` pair, so a repeated scale does not repeat the
    /// coupling lookup.
    last_coupling: Cell<(f64, f64)>,
    vegas_alpha: f64,
}

impl<'a> ProtonIntegrand<'a> {
    /// Build the integrand over a process's flavour decomposition.
    ///
    /// * `groups` — the decomposition ([`derive_flavor_groups`]).
    /// * `amps` — one bound amplitude per group, **in group order**, each bound from
    ///   that group's own [`FlavorGroup::evaluator`]. The pairing is checked by
    ///   identity rather than trusted: crossing it would weight one group's matrix
    ///   element with another's luminosity, which no cross-section-level check
    ///   separates from a coupling error.
    /// * `model` — the evaluated model the channel trees read pole masses and widths
    ///   from.
    /// * `sqrt_s_had` — total collider energy `√s = E₁ + E₂`.
    /// * `mu_f` — a constant scale on both beams, replaced by the run card's
    ///   prescription by [`use_run_card_scales`](Self::use_run_card_scales).
    ///
    /// Every group's diagrams contribute a channel and all of them are pooled into
    /// one mixture, so a peak one group's own diagrams do not cover — the mirrored
    /// `g q` configuration above all — is still covered by another group's.
    ///
    /// The peripheral channels are floored at [`Cuts::spacelike_floor`], the scale
    /// the process's own transverse-momentum cuts imply. The floor is passed because
    /// a final state of more than two legs has no peripheral spine without one; it
    /// also reshapes a `2 → 2` process's spine, which leaves the estimator unbiased
    /// but does change the map a banked run was taken with.
    pub fn new(
        groups: &'a FlavorGroups,
        amps: &'a [BoundAmplitude<'a, f64>],
        model: &EvaluatedModel,
        pdf: &'a PdfMember,
        sqrt_s_had: f64,
        mu_f: f64,
    ) -> Result<Self, ProtonError> {
        if amps.len() != groups.groups().len() {
            return Err(ProtonError::AmplitudeCount {
                amps: amps.len(),
                groups: groups.groups().len(),
            });
        }
        for (i, (g, amp)) in groups.groups().iter().zip(amps).enumerate() {
            if !std::ptr::eq(amp.evaluator(), g.evaluator()) {
                return Err(ProtonError::AmplitudeMismatch { index: i });
            }
        }
        let cuts = groups.groups()[0].cuts();
        for (i, g) in groups.groups().iter().enumerate().skip(1) {
            if g.cuts() != cuts {
                return Err(ProtonError::GroupCutsDiffer { a: 0, b: i });
            }
        }

        let floor = cuts.spacelike_floor();
        let mut channels: Vec<Box<dyn ScaledChannel<f64>>> = Vec::new();
        let mut channel_ids = Vec::new();
        for (gi, g) in groups.groups().iter().enumerate() {
            for (di, d) in g.diagrams().iter().enumerate() {
                // The baked-in energy is unread through `ScaledChannel`, which takes
                // the event's own; the collider energy is the well-formed value to
                // leave it at.
                channels.push(Box::new(DiagramChannel::from_diagram_regulated(
                    d, model, sqrt_s_had, floor,
                )));
                channel_ids.push(ChannelId {
                    group: gi,
                    diagram: di,
                });
            }
        }

        let n_out = groups.groups()[0].final_masses().len();
        let s_had = sqrt_s_had * sqrt_s_had;
        let tau_min = cuts.shat_min() / s_had;
        Ok(ProtonIntegrand {
            subs: amps.iter().map(BoundSubprocess::fixed).collect(),
            groups,
            pdf,
            cuts,
            combiner: ScaledMultiChannel::uniform(channels),
            channel_ids,
            s_had,
            sqrt_s_had,
            tau_min,
            ln_inv_tau_min: (1.0 / tau_min).ln(),
            lips_2pi: (2.0 * PI).powi(4 - 3 * n_out as i32),
            scales: EventScaleSource::constant(mu_f),
            scale_buf: RefCell::new(Vec::with_capacity(n_out)),
            cm_buf: RefCell::new(Vec::with_capacity(2 + n_out)),
            lab_buf: RefCell::new(Vec::with_capacity(2 + n_out)),
            mirror_buf: RefCell::new(Vec::with_capacity(2 + n_out)),
            last_coupling: Cell::new((f64::NAN, f64::NAN)),
            vegas_alpha: VEGAS_ALPHA_MAPPED,
        })
    }

    /// Take the renormalisation and per-beam factorisation scales — and the strong
    /// coupling `μR` implies — from the run card instead of the constant `μF`
    /// [`new`](Self::new) was given.
    ///
    /// `alpha_s` is the `AlphaS_*` metadata of the set the beams read their densities
    /// from. A card whose `pdlabel` delegates `αs` to LHAPDF is resolved from that
    /// tabulation, and refuses rather than falling back to a beta-function solve the
    /// densities were not fitted with.
    ///
    /// The cluster topology the `-1` scale consults is derived from the first group's
    /// diagrams. Groups of one process need not share a topology, so a prescription
    /// that actually reads it is resolved once here on a sampled, cut-passing point:
    /// a clustering this crate refuses stops the run at setup rather than at the first
    /// VEGAS point, and before any per-group question arises.
    pub fn use_run_card_scales(
        &mut self,
        model: &UFOModel,
        evaluated: &'a EvaluatedModel,
        card: &RunCard,
        alpha_s: Option<&AlphaSInfo>,
    ) -> Result<RunningCouplingReport, ProtonError> {
        let awareness = make_subs_scale_aware(&mut self.subs, evaluated);
        // Unlike a fixed-beam run, the factorisation scale has a consumer whatever
        // the matrix element is made of, so the prescription is compiled even when
        // nothing moves with the strong coupling.
        let source = compile_scale_source(
            self.subs[0].evaluator(),
            self.groups.groups()[0].diagrams(),
            model,
            evaluated,
            card,
            alpha_s,
            awareness.depends_on_alpha_s,
        )?;
        if source.constant_scales().is_none() {
            self.probe_scale(&source)?;
        }
        let report = constant_scale_report(&self.subs, Some(&source), awareness);
        self.scales = source;
        Ok(report)
    }

    /// Resolve the scale on the first cut-passing point of a fixed pseudo-random
    /// draw, so a refusal that would otherwise surface mid-integration surfaces at
    /// setup.
    fn probe_scale(&self, source: &EventScaleSource) -> Result<(), ProtonError> {
        use rand::Rng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(SCALE_PROBE_SEED);
        let ndim = self.channel_grid_ndim();
        for _ in 0..SCALE_PROBE_DRAWS {
            let u: Vec<f64> = (0..ndim).map(|_| rng.random::<f64>()).collect();
            let m = self.map_point(&u);
            let pt = self
                .combiner
                .sample_channel_at(0, m.sqrt_shat, &u[OUTER_NDIM..]);
            self.build_frames(&m, &pt.momenta);
            let lab = self.lab_buf.borrow();
            if self.cuts.pass(&lab) {
                self.scales_of(source, &lab).map_err(HadronicError::from)?;
                return Ok(());
            }
        }
        Ok(())
    }

    /// Lower support `ŝ_min/s` of the logarithmic τ map.
    pub fn tau_min(&self) -> f64 {
        self.tau_min
    }

    /// The spacelike-pole floor (GeV²) the peripheral channels were built with, from
    /// the process's own cuts.
    pub fn spacelike_floor(&self) -> f64 {
        self.cuts.spacelike_floor()
    }

    /// Which diagram of which group each sampling channel came from, in channel
    /// order — the key a per-channel grid is banked under.
    pub fn channel_ids(&self) -> &[ChannelId] {
        &self.channel_ids
    }

    /// The channels the integral is split across: one per diagram of every group,
    /// pooled into a single mixture.
    pub fn channel_count(&self) -> usize {
        self.combiner.channels().len()
    }

    /// The current channel selection weights, in channel order.
    pub fn channel_alphas(&self) -> Vec<f64> {
        self.combiner.alphas().to_vec()
    }

    /// Install selection weights taken from a completed adaptation instead of
    /// re-surveyed — the exactness a replayed run needs, since `αⱼ` enters every
    /// channel's weight.
    ///
    /// # Panics
    ///
    /// If `alphas` is not one normalised set of positive weights per channel.
    pub fn set_channel_alphas(&mut self, alphas: Vec<f64>) {
        self.combiner.set_alphas(alphas);
    }

    /// The coordinates one channel's grid is built over: the two outer `(τ, y)`
    /// coordinates followed by the channel's own `3n − 4`.
    pub fn channel_grid_ndim(&self) -> usize {
        OUTER_NDIM + self.combiner.channel_ndim()
    }

    /// The strong coupling's source, once a run card installed one.
    pub fn alpha_s_source(&self) -> Option<&AlphaSSource> {
        self.scales.alpha_s()
    }

    /// Map a VEGAS point's outer coordinates to the partonic system.
    fn map_point(&self, u: &[f64]) -> OuterPoint {
        let tau = self.tau_min.powf(1.0 - u[0]);
        let sqrt_tau = tau.sqrt();
        let y_max = -0.5 * tau.ln();
        let y = (2.0 * u[1] - 1.0) * y_max;
        OuterPoint {
            x1: sqrt_tau * y.exp(),
            x2: sqrt_tau * (-y).exp(),
            sqrt_shat: (tau * self.s_had).sqrt(),
            jac: self.ln_inv_tau_min * 2.0 * y_max,
        }
    }

    /// Fill the partonic-CM and lab-frame external momenta of one point: beams first,
    /// then the outgoing legs.
    fn build_frames(&self, m: &OuterPoint, out: &[V]) {
        let e_cm = m.sqrt_shat / 2.0;
        let mut cm = self.cm_buf.borrow_mut();
        cm.clear();
        cm.push(V::new(e_cm, 0.0, 0.0, e_cm));
        cm.push(V::new(e_cm, 0.0, 0.0, -e_cm));
        cm.extend_from_slice(out);

        let e_beam = self.sqrt_s_had / 2.0;
        let beta = (m.x1 - m.x2) / (m.x1 + m.x2);
        let mut lab = self.lab_buf.borrow_mut();
        lab.clear();
        lab.push(V::new(m.x1 * e_beam, 0.0, 0.0, m.x1 * e_beam));
        lab.push(V::new(m.x2 * e_beam, 0.0, 0.0, -m.x2 * e_beam));
        lab.extend(out.iter().map(|p| boost_z(*p, beta)));
    }

    /// The integrand at one point with the phase-space map's own weight left out:
    /// the `(τ, y)` Jacobian, the flux, the `2π` measure and the luminosity-weighted
    /// sum over groups. Zero where the cuts reject the lab-frame configuration or no
    /// group carries luminosity.
    fn shape(&self, m: &OuterPoint, out: &[V]) -> f64 {
        self.build_frames(m, out);
        let cm = self.cm_buf.borrow();
        {
            let lab = self.lab_buf.borrow();
            if !self.cuts.pass(&lab) {
                return 0.0;
            }
        }
        let scales = self.event_scales();
        self.apply_scale(scales.mu_r);

        let mut acc = 0.0;
        let mut mirror = self.mirror_buf.borrow_mut();
        for (g, sub) in self.groups.groups().iter().zip(&self.subs) {
            let [direct, reflected] = g.luminosity(self.pdf, m.x1, m.x2, scales.mu_f);
            let mut term = 0.0;
            if direct != 0.0 {
                term += direct * sub.eval_m2(&cm);
            }
            // Zero for a group whose beams carry one parton ([`FlavorGroup::has_mirror`]),
            // so such a group costs one matrix element per point rather than two.
            if reflected != 0.0 {
                g.mirror_into(&cm, &mut mirror);
                term += reflected * sub.eval_m2(&mirror);
            }
            acc += g.spin_color_average() * term;
        }
        if acc == 0.0 {
            return 0.0;
        }
        let flux = 1.0 / (2.0 * m.sqrt_shat * m.sqrt_shat);
        m.jac * flux * self.lips_2pi * acc
    }

    /// The scales at the lab-frame point currently in the frame buffers.
    fn event_scales(&self) -> EventScales {
        if let Some(fixed) = self.scales.constant_scales() {
            return fixed;
        }
        let lab = self.lab_buf.borrow();
        self.scales_of(&self.scales, &lab)
            .unwrap_or_else(|e| panic!("per-event scale on a sampled point: {e}"))
    }

    fn scales_of(&self, source: &EventScaleSource, lab: &[V]) -> Result<EventScales, ScaleError> {
        let mut buf = self.scale_buf.borrow_mut();
        buf.clear();
        buf.extend(lab[2..].iter().map(components));
        source.scales([components(&lab[0]), components(&lab[1])], &buf)
    }

    /// Move every group's amplitude to the coupling `mu_r` implies. A constant
    /// prescription was applied once at installation and a matrix element with no
    /// strong coupling has none to move, so both return without touching the pools.
    fn apply_scale(&self, mu_r: f64) {
        if self.scales.constant_scales().is_some() {
            return;
        }
        let Some(source) = self.scales.alpha_s() else {
            return;
        };
        let (last_mu_r, last_alpha_s) = self.last_coupling.get();
        let alpha_s = if mu_r == last_mu_r {
            last_alpha_s
        } else {
            let alpha_s = source.eval(mu_r);
            self.last_coupling.set((mu_r, alpha_s));
            alpha_s
        };
        for sub in &self.subs {
            sub.set_alpha_s(alpha_s);
        }
    }

    /// The `channel`-th term of the channel-split estimator at
    /// `u ∈ [0,1]^channel_grid_ndim`, in natural units (GeV⁻²): the outer map takes
    /// `u[0..2]`, the channel's own map the rest, and the point is weighted by
    /// `αⱼ/g` so the terms' integrals sum to the cross section.
    ///
    /// # Panics
    ///
    /// If `channel` is not a channel index.
    pub fn value_in_channel(&self, channel: usize, u: &[f64]) -> f64 {
        let m = self.map_point(u);
        let point = self
            .combiner
            .sample_channel_at(channel, m.sqrt_shat, &u[OUTER_NDIM..]);
        let shape = self.shape(&m, &point.momenta);
        if shape == 0.0 {
            return 0.0;
        }
        shape * point.weight
    }

    /// Refine the channel selection weights toward the variance-minimising mixture,
    /// jointly over the `(group, diagram)` channel space.
    ///
    /// This is [`MultiChannel::adapt_alphas`](crate::phasespace::MultiChannel::adapt_alphas)'
    /// survey→refine loop driven from outside the combiner, because the integrand —
    /// not the combiner — owns the `(τ, y)` coordinates and so owns the energy each
    /// draw is made at. Each survey draws `n_survey` points from the *current*
    /// mixture over the full hypercube (outer coordinates, a channel-selection
    /// coordinate, then the channel's own), estimates every channel's variance share
    /// `Wⱼ = E_g[(f/g)²·gⱼ/g]`, and reallocates by [`kleiss_pittau_step`].
    ///
    /// The surveyed `f` is the whole integrand shape — the `(τ, y)` Jacobian, the
    /// flux, the cut and the luminosity-weighted sum over groups — so weight flows to
    /// the channels that carry variance *of the hadronic integral*, not of a partonic
    /// one at some representative energy.
    pub fn adapt_alphas(
        &mut self,
        seed: u64,
        n_survey: usize,
        n_iter: usize,
        damping: f64,
    ) -> AlphaAdaptation<f64> {
        let mut trajectory = vec![self.channel_alphas()];
        let mut variance_shares = vec![0.0; self.channel_count()];
        for it in 0..n_iter {
            let w = self.survey_variance(seed, ADAPT_STREAM + it as u64, n_survey);
            variance_shares = w.clone();
            let Some(raw) = kleiss_pittau_step(self.combiner.alphas(), &w, damping) else {
                break;
            };
            self.combiner.set_alphas(raw.clone());
            trajectory.push(raw);
        }
        AlphaAdaptation {
            trajectory,
            variance_shares,
        }
    }

    /// One survey pass: every channel's variance share `Wⱼ = E_g[(f/g)²·gⱼ/g]` under
    /// the current mixture. Each drawn point informs every channel, so the estimate
    /// is low-variance in the channels it does not draw from.
    fn survey_variance(&self, seed: u64, stream: u64, n_survey: usize) -> Vec<f64> {
        let n = self.channel_count();
        let mut w = vec![0.0; n];
        let mut s = SubStream::from_stream(seed, stream);
        let ndim = self.channel_grid_ndim() + 1;
        for _ in 0..n_survey {
            let u = s.uniforms::<f64>(ndim);
            let m = self.map_point(&u);
            let j = self.combiner.select(u[OUTER_NDIM]);
            let point = self
                .combiner
                .sample_channel_at(j, m.sqrt_shat, &u[OUTER_NDIM + 1..]);
            // `sample_channel_at` weights by `αⱼ/g`; the mixture that actually drew
            // this point has density `g`, so the mixture estimator is `f/g`.
            let g = self.combiner.alphas()[j] / point.weight;
            let est = self.shape(&m, &point.momenta) / g;
            if est == 0.0 {
                continue;
            }
            let est2 = est * est;
            for (wj, ch) in w.iter_mut().zip(self.combiner.channels()) {
                *wj += est2 * ch.density_at(m.sqrt_shat, &point.momenta) / g;
            }
        }
        let inv = 1.0 / n_survey as f64;
        for wj in &mut w {
            *wj *= inv;
        }
        w
    }

    /// Run one VEGAS adaptation per channel, returning each channel's trained grid and
    /// term alongside their sum — the primitive a banked integration serialises.
    ///
    /// Channel `j` is integrated over its own `channel_grid_ndim` coordinates with the
    /// channel frozen, on a budget of `αⱼ · neval` per iteration and its own RNG
    /// substream, exactly as the fixed-beam path does.
    pub fn adapt_grids(
        &self,
        neval: usize,
        niter: usize,
        seed: u64,
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
            let mut grid = VegasGrid::new(ndim, VEGAS_NBINS, self.vegas_alpha);
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

    /// Integrate the cross section with VEGAS, returning `(σ, Δσ)` in picobarns.
    pub fn integrate(&self, neval: usize, niter: usize, seed: u64) -> (f64, f64) {
        let result = self.adapt_grids(neval, niter, seed).1;
        (result.integral * GEV2_TO_PB, result.std_dev * GEV2_TO_PB)
    }
}

/// The seam a frozen-grid scan and an accept/reject pass drive this integrand
/// through — the same one the fixed-beam path exposes, so neither is rebuilt for
/// hadronic beams.
impl ChannelIntegrand for ProtonIntegrand<'_> {
    fn channel_count(&self) -> usize {
        ProtonIntegrand::channel_count(self)
    }

    fn channel_grid_ndim(&self) -> usize {
        ProtonIntegrand::channel_grid_ndim(self)
    }

    fn value_in_channel(&self, channel: usize, u: &[f64]) -> f64 {
        ProtonIntegrand::value_in_channel(self, channel, u)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::pdf::grid::SubGrid;
    use crate::ufo::sm::{sm_model, SMRestrict};
    use std::collections::BTreeSet;
    use std::sync::Arc;

    /// The multi-subprocess hadronic process the grouping rule is measured on:
    /// two beam multiparticles, two coupling classes, and a jet that is a gluon
    /// in one arrangement and a quark in the other.
    const LLJ: &str = "p p > l+ l- j QCD=2 QED=2";

    fn model() -> Arc<UFOModel> {
        sm_model(SMRestrict::Default)
    }

    fn enumerate(process: &str, model: &UFOModel) -> Vec<DiagramSet> {
        let opts = ParsingOptions::default();
        let card = parse_proc_card(&format!("generate {process}"), &opts).expect("proc card");
        generate_from_proc_card(&card, model).expect("enumeration")
    }

    fn derive(process: &str, model: &UFOModel, evaluated: &EvaluatedModel) -> FlavorGroups {
        derive_flavor_groups(
            enumerate(process, model),
            model,
            evaluated,
            &RunCard::default(),
        )
        .expect("flavour groups")
    }

    /// Probe points at energies and on a stream the partition was *not* derived
    /// from, so a within-group agreement measured here is a prediction.
    fn fresh_points(final_masses: &[f64]) -> Vec<Vec<V>> {
        let mut points = Vec::new();
        for (i, sqrt_s) in [220.0f64, 740.0, 2100.0].into_iter().enumerate() {
            let rambo = RamboChannel::<f64>::new(sqrt_s, final_masses.to_vec());
            let mut stream = SubStream::from_stream(0x0FF5_E7ED, i as u64);
            for _ in 0..12 {
                let u = stream.uniforms::<f64>(rambo.ndim());
                let drawn = rambo.sample(&u);
                let e = sqrt_s / 2.0;
                let mut momenta = vec![V::new(e, 0.0, 0.0, e), V::new(e, 0.0, 0.0, -e)];
                momenta.extend(drawn.momenta.iter().cloned());
                points.push(momenta);
            }
        }
        points
    }

    fn m2_trace(
        evaluator: &AmplitudeEvaluator,
        evaluated: &EvaluatedModel,
        points: &[Vec<V>],
    ) -> Vec<f64> {
        let bound = BoundAmplitude::<f64>::bind(evaluator, evaluated);
        let mut scratch = bound.scratch_space();
        points
            .iter()
            .map(|k| bound.eval_m2(k, &mut scratch))
            .collect()
    }

    /// A synthetic parton distribution whose `x·f` differs per flavour and whose
    /// `x` shape differs per flavour too, so a luminosity that swapped the two
    /// beam orderings or dropped one of them produces a different number. A
    /// common `x` shape would make the two orderings equal and the check vacuous.
    fn probe_pdf() -> PdfMember {
        let flavors = vec![-4, -3, -2, -1, 1, 2, 3, 4, 21];
        let x = vec![1e-7, 1.0];
        let q2 = vec![1.0, 1e8];
        let mut xf = Vec::new();
        for ix in 0..x.len() {
            for iq in 0..q2.len() {
                for ifl in 0..flavors.len() {
                    let shape = 1.0 + ix as f64 * 0.1 * (ifl + 1) as f64;
                    xf.push(0.01 * (ifl + 1) as f64 * shape * (1.0 + 0.5 * iq as f64));
                }
            }
        }
        PdfMember::from_subgrids(vec![SubGrid { x, q2, flavors, xf }])
    }

    #[test]
    fn llj_partitions_into_six_groups_of_four_concrete_subprocesses() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let groups = derive(LLJ, &m, &evaluated);

        assert_eq!(groups.subprocess_count(), 24);
        assert_eq!(groups.groups().len(), 6);

        // Each group is one coupling class × one colour arrangement: two quark
        // generations × two lepton flavours, all with the same `|M|²`.
        let mut initial_states: BTreeSet<[i32; 2]> = BTreeSet::new();
        for g in groups.groups() {
            assert_eq!(g.members().len(), 4);
            let pairs: BTreeSet<[i32; 2]> = g.members().iter().map(|s| s.incoming).collect();
            assert_eq!(pairs.len(), 2, "a group should hold two quark generations");
            // Two lepton flavours per initial state; `q q̄` groups share the
            // outgoing gluon, so it is the whole assignment that must be distinct.
            let distinct: BTreeSet<&Subprocess> = g.members().iter().collect();
            assert_eq!(distinct.len(), 4);
            initial_states.extend(pairs);
        }
        // Eight gluon-quark initial states and four annihilations, one ordering each.
        assert_eq!(initial_states.len(), 12);

        let averages: BTreeSet<String> = groups
            .groups()
            .iter()
            .map(|g| format!("{:.6}", g.spin_color_average()))
            .collect();
        let expected: BTreeSet<String> = [1.0 / 96.0, 1.0 / 36.0]
            .iter()
            .map(|v| format!("{v:.6}"))
            .collect();
        assert_eq!(averages, expected);
    }

    /// The partition predicts, rather than merely records: members agree at points
    /// it was not fitted on, and distinct groups disagree by a wide margin there.
    #[test]
    fn group_members_agree_where_the_partition_was_not_measured() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let groups = derive(LLJ, &m, &evaluated);
        let points = fresh_points(&groups.groups()[0].final_masses());

        let representatives: Vec<Vec<f64>> = groups
            .groups()
            .iter()
            .map(|g| m2_trace(g.evaluator(), &evaluated, &points))
            .collect();

        // Every enumerated subprocess is re-derived from the card and matched
        // against the partition, so both halves are checked: that it agrees with
        // the group it was put in, and that the group claims it as a member.
        let mut worst_within = 0.0f64;
        let mut matched = 0usize;
        for set in enumerate(LLJ, &m).iter().filter(|s| !s.diagrams.is_empty()) {
            let evaluator = compile_class(set, &m, &evaluated).expect("subprocess compiles");
            let legs = process_external_legs(&evaluator, &m, &evaluated);
            let subprocess = Subprocess {
                incoming: [legs[0].pdg, legs[1].pdg],
                outgoing: legs[2..].iter().map(|l| l.pdg).collect(),
            };
            let trace = m2_trace(&evaluator, &evaluated, &points);
            let hits: Vec<usize> = (0..groups.groups().len())
                .filter(|&i| worst_rel(&representatives[i], &trace) < 1e-12)
                .collect();
            assert_eq!(
                hits.len(),
                1,
                "{} matches {} groups at fresh points",
                label(set),
                hits.len()
            );
            assert!(
                groups.groups()[hits[0]].members().contains(&subprocess),
                "{} agrees with a group that does not list it",
                label(set)
            );
            worst_within = worst_within.max(worst_rel(&representatives[hits[0]], &trace));
            matched += 1;
        }
        assert_eq!(matched, groups.subprocess_count());

        let mut min_cross = f64::INFINITY;
        for (i, a) in representatives.iter().enumerate() {
            for b in &representatives[i + 1..] {
                min_cross = min_cross.min(worst_rel(a, b));
            }
        }
        eprintln!("within-group {worst_within:.3e}, cross-group {min_cross:.3e}");
        assert!(
            min_cross > 0.1,
            "two groups differ by only {min_cross:.3e}; the partition is not a coupling distinction"
        );
    }

    /// The mirror identity `|M_{b a}(q)|² = |M_{a b}(R q)|²`, against explicitly
    /// enumerated mirrored subprocesses.
    ///
    /// This is the term the enumeration does not produce: only one ordering of
    /// each unordered initial state exists, and its parton luminosity is not the
    /// other's. The identity control is the load-bearing half — evaluating the
    /// representative at the *unreflected* point (what dropping the mirror
    /// amounts to) is wrong at every probe point, by between 0.2% and a factor of
    /// 200.
    ///
    /// What this cannot see: `|M|²` is invariant under a further reflection in the
    /// `xz` plane, so the sign of `p_y` in `R` is unpinned — asserted below as a
    /// measured fact rather than left as an assumption. What `R` must do is
    /// reverse `p_z`, and that is what the identity control pins.
    #[test]
    fn the_mirrored_beam_ordering_needs_the_reflected_matrix_element() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let groups = derive(LLJ, &m, &evaluated);
        let points = fresh_points(&groups.groups()[0].final_masses());

        let (mut worst_mirror, mut min_identity, mut worst_py) = (0.0f64, f64::INFINITY, 0.0f64);
        for g in groups.groups() {
            let set = g.diagram_set();
            let swapped = format!(
                "{} {} > {} QCD=2 QED=2",
                set.particles_in[1],
                set.particles_in[0],
                set.particles_out.join(" ")
            );
            let mirror_set = enumerate(&swapped, &m)
                .into_iter()
                .find(|s| !s.diagrams.is_empty())
                .expect("the mirrored ordering enumerates");
            assert_eq!(
                mirror_set.diagrams.len(),
                set.diagrams.len(),
                "{swapped} is not the same process as {}",
                label(set)
            );
            let mirror_eval = compile_class(&mirror_set, &m, &evaluated).expect("mirror compiles");

            let bound = BoundAmplitude::<f64>::bind(g.evaluator(), &evaluated);
            let mut scratch = bound.scratch_space();
            let mirror_bound = BoundAmplitude::<f64>::bind(&mirror_eval, &evaluated);
            let mut mirror_scratch = mirror_bound.scratch_space();

            let mut reflected = Vec::new();
            for k in &points {
                let target = mirror_bound.eval_m2(k, &mut mirror_scratch);
                let rel = |x: f64| (x - target).abs() / target.abs().max(f64::MIN_POSITIVE);

                g.mirror_into(k, &mut reflected);
                worst_mirror = worst_mirror.max(rel(bound.eval_m2(&reflected, &mut scratch)));

                min_identity = min_identity.min(rel(bound.eval_m2(k, &mut scratch)));

                let py_flipped: Vec<V> = k
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        if i < g.n_in() {
                            *p
                        } else {
                            V::new(p.e(), p.px(), -p.py(), p.pz())
                        }
                    })
                    .collect();
                let direct = bound.eval_m2(k, &mut scratch);
                let flipped = bound.eval_m2(&py_flipped, &mut scratch);
                worst_py =
                    worst_py.max((direct - flipped).abs() / direct.abs().max(f64::MIN_POSITIVE));
            }
        }
        eprintln!(
            "mirror identity worst {worst_mirror:.3e}; dropping it costs at least \
             {min_identity:.3e}; an xz reflection alone moves |M|² by {worst_py:.3e}"
        );
        // The bound is set by the probe points, not by the identity. One of the 36
        // RAMBO draws is 8e-10 off the light cone and 2e-12 off momentum
        // conservation, where the two independently compiled programs route the
        // gauge-dependent parts differently and land 5.4e-13 apart; every other
        // point agrees to 4.4e-15. Nothing downstream sees this: the integrand
        // evaluates *one* program at both arguments.
        assert!(
            worst_mirror < 1e-11,
            "the reflected representative disagrees with the mirrored subprocess by \
             {worst_mirror:.3e}"
        );
        assert!(
            min_identity > 1e-3,
            "the mirror term is worth only {min_identity:.3e} at its weakest point; a dropped \
             mirror would not be visible here"
        );
        assert!(
            worst_py < 1e-12,
            "|M|² moved by {worst_py:.3e} under a reflection in the xz plane, so the sign of p_y \
             in the mirror map is pinned after all and this test's blind spot is misstated"
        );
    }

    /// One group per identical-particle-free subprocess is a premise, not a
    /// convention: a repeated outgoing species is refused rather than silently
    /// dropped into a summed matrix element with no owner for its `1/n!`.
    #[test]
    fn a_repeated_outgoing_species_is_refused() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        for g in derive(LLJ, &m, &evaluated).groups() {
            assert_eq!(final_state_symmetry_factor(g.evaluator()), 1.0);
        }
        let err = match derive_flavor_groups(
            enumerate("u u~ > g g", &m),
            &m,
            &evaluated,
            &RunCard::default(),
        ) {
            Ok(_) => panic!("two outgoing gluons must be refused"),
            Err(e) => e,
        };
        assert!(
            matches!(err, ProtonError::IdenticalFinalState { factor, .. } if factor == 0.5),
            "unexpected refusal: {err}"
        );
    }

    /// The generalisation lands on Drell–Yan's hand-derived classes, and its two
    /// orderings sum to the luminosity `DrellYanIntegrand` documents.
    #[test]
    fn drell_yan_classes_and_their_two_orderings_are_reproduced() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let groups = derive("p p > e+ e-", &m, &evaluated);
        assert_eq!(groups.groups().len(), 2);

        let pdf = probe_pdf();
        let (x1, x2, mu_f) = (0.03, 0.007, [91.188, 91.188]);
        let q2 = [mu_f[0] * mu_f[0], mu_f[1] * mu_f[1]];
        for g in groups.groups() {
            let quarks: Vec<i32> = g
                .members()
                .iter()
                .map(|s| s.incoming[0].abs())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            assert!(
                quarks == vec![2, 4] || quarks == vec![1, 3],
                "classes: {quarks:?}"
            );

            let expected: f64 = quarks
                .iter()
                .map(|&q| {
                    pdf.xfx_q2(q, x1, q2[0]) * pdf.xfx_q2(-q, x2, q2[1])
                        + pdf.xfx_q2(-q, x1, q2[0]) * pdf.xfx_q2(q, x2, q2[1])
                })
                .sum();
            let [direct, mirror] = g.luminosity(&pdf, x1, x2, mu_f);
            let rel = (direct + mirror - expected).abs() / expected;
            assert!(rel < 1e-14, "class luminosity off by {rel:.3e}");
            assert!(
                (direct - mirror).abs() / direct > 1e-3,
                "the two orderings coincide here, so this test cannot see a dropped one"
            );
        }
    }

    /// A `g g` initial state is one ordering, not two: its mirror carries no
    /// luminosity, while a `q q̄` group's does and differs from the direct term.
    #[test]
    fn an_identical_parton_initial_state_carries_a_single_ordering() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let groups = derive("p p > t t~ QED=0", &m, &evaluated);
        assert_eq!(groups.groups().len(), 2);
        assert_eq!(groups.subprocess_count(), 5);

        let pdf = probe_pdf();
        let (x1, x2, mu_f) = (0.05, 0.004, [173.0, 173.0]);
        let mut seen_gg = false;
        let mut bases = Vec::new();
        for g in groups.groups() {
            bases.push((g.evaluator().n_flows(), g.evaluator().cf_matrix().to_vec()));
            let [direct, mirror] = g.luminosity(&pdf, x1, x2, mu_f);
            if g.members()[0].incoming == [21, 21] {
                seen_gg = true;
                assert_eq!(g.members().len(), 1);
                assert!(!g.has_mirror());
                assert_eq!(mirror, 0.0);
                assert!((g.spin_color_average() - 1.0 / 256.0).abs() < 1e-15);
            } else {
                // At `QED = 0` every quark annihilation is the same colour and
                // coupling structure, so all four flavours are one group.
                assert_eq!(g.members().len(), 4);
                assert!(g.has_mirror());
                assert!(mirror > 0.0 && (direct - mirror).abs() / direct > 1e-3);
                assert!((g.spin_color_average() - 1.0 / 36.0).abs() < 1e-15);
            }
        }
        assert!(seen_gg, "no gluon-fusion group in p p > t t~");
        // The within-group colour-basis requirement is checked against a quantity
        // that separates real processes: gluon fusion and quark annihilation reach
        // the same final state through different colour bases. It does not fire
        // inside any group here, where the members differ by generation label alone.
        assert_ne!(bases[0], bases[1]);
    }

    /// A quark and its antiquark against a gluon are *different* groups, and the
    /// pointwise criterion is what says so.
    ///
    /// Their partonic cross sections agree within Monte-Carlo error — the banked
    /// `√ŝ = 500` runs give `0.11812 ± 0.00022 pb` for `g u > e+ e- u` against
    /// `0.11816 ± 0.00026 pb` for `g u~ > e+ e- u~` — so a criterion built on σ̂
    /// would have merged them, summing the antiquark's luminosity against the
    /// quark's matrix element. Pointwise `|M|²` separates them by more than 10%,
    /// which is also why they need separate colour structures downstream.
    #[test]
    fn a_quark_and_its_antiquark_against_a_gluon_do_not_share_a_matrix_element() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let groups = derive(LLJ, &m, &evaluated);
        let points = fresh_points(&groups.groups()[0].final_masses());

        let find = |beams: [i32; 2]| {
            groups
                .groups()
                .iter()
                .position(|g| g.members().iter().any(|s| s.incoming == beams))
                .unwrap_or_else(|| panic!("no group holds {beams:?}"))
        };
        let mut worst = 0.0f64;
        for quark in [2, 1] {
            let (q, qbar) = (find([21, quark]), find([21, -quark]));
            assert_ne!(q, qbar, "g q and g q̄ were merged into one group");
            let rel = worst_rel(
                &m2_trace(groups.groups()[q].evaluator(), &evaluated, &points),
                &m2_trace(groups.groups()[qbar].evaluator(), &evaluated, &points),
            );
            worst = worst.max(rel);
            assert!(
                rel > 0.1,
                "g {quark} and g -{quark} differ by only {rel:.3e}"
            );
        }
        eprintln!("g q vs g q̄ pointwise |M|² separation up to {worst:.3e}");
    }

    /// The shared-cut-filter requirement has teeth: a b-quark jet is a different
    /// filter from a light-quark jet at `maxjetflavor = 4`, where MadGraph cuts it
    /// with `ptb` instead of `ptj`.
    #[test]
    fn a_group_sharing_one_cut_filter_is_a_real_requirement() {
        let legs = |jet: i32, mass: f64| {
            vec![
                ExternalLeg::incoming(21, 0.0),
                ExternalLeg::incoming(jet, 0.0),
                ExternalLeg::outgoing(-11, 0.0),
                ExternalLeg::outgoing(11, 0.0),
                ExternalLeg::outgoing(jet, mass),
            ]
        };
        let mut card = RunCard::default();
        assert_eq!(card.maxjetflavor, 4);
        let light = Cuts::compile(&card, &legs(2, 0.0)).expect("light-jet cuts");
        let bottom = Cuts::compile(&card, &legs(5, 4.7)).expect("b-jet cuts");
        assert_ne!(
            light, bottom,
            "a b leg compiles to the same filter as a light jet, so the group check is vacuous"
        );

        card.maxjetflavor = 5;
        let bottom_as_jet = Cuts::compile(&card, &legs(5, 4.7)).expect("b-as-jet cuts");
        assert_eq!(
            light, bottom_as_jet,
            "the difference is the jet classification, not the leg's mass"
        );
    }

    /// The banked fixed-scale `p p > l+ l- j` configuration: 13 TeV proton beams,
    /// every scale fixed at `m_Z`, and the cuts the reference run was taken with.
    fn llj_card() -> RunCard {
        RunCard::parse(
            "  1 = lpp1\n  1 = lpp2\n  6500.0 = ebeam1\n  6500.0 = ebeam2\n\
             \x20 lhapdf = pdlabel\n  247000 = lhaid\n\
             \x20 True = fixed_ren_scale\n  True = fixed_fac_scale1\n  True = fixed_fac_scale2\n\
             \x20 91.188 = scale\n  91.188 = dsqrt_q2fact1\n  91.188 = dsqrt_q2fact2\n\
             \x20 -1 = dynamical_scale_choice\n\
             \x20 20.0 = ptj\n  10.0 = ptl\n  5.0 = etaj\n  2.5 = etal\n\
             \x20 0.4 = drll\n  0.4 = drjl\n  50.0 = mmll\n  4 = maxjetflavor\n",
        )
        .expect("run card")
    }

    const SQRT_S_HAD: f64 = 13000.0;
    const MU_F: f64 = 91.188;

    /// `αs` knots bracketing `m_Z`, so a card that delegates the coupling to the set
    /// resolves without the fetched grid.
    fn probe_alpha_s() -> AlphaSInfo {
        AlphaSInfo {
            mz: 91.1876,
            order_qcd: 1,
            kind: "ipol".to_string(),
            qs: vec![1.0, 91.1876, 109.8541, 10000.0],
            vals: vec![0.4, 0.1300028, 0.1262725, 0.08],
            lambda4: 0.0,
            lambda5: 0.0,
        }
    }

    /// A `√ŝ`-independent channel set built exactly as the integrand builds it — the
    /// oracle's own copy of the phase-space map, so the point and weight it compares
    /// against are reproduced rather than read out of the integrand.
    fn rebuild_channels(
        groups: &FlavorGroups,
        model: &EvaluatedModel,
        floor: f64,
    ) -> ScaledMultiChannel<f64> {
        let mut channels: Vec<Box<dyn ScaledChannel<f64>>> = Vec::new();
        for g in groups.groups() {
            for d in g.diagrams() {
                channels.push(Box::new(DiagramChannel::from_diagram_regulated(
                    d, model, SQRT_S_HAD, floor,
                )));
            }
        }
        ScaledMultiChannel::uniform(channels)
    }

    fn bind_all<'a>(
        groups: &'a FlavorGroups,
        evaluated: &'a EvaluatedModel,
    ) -> Vec<BoundAmplitude<'a, f64>> {
        groups
            .groups()
            .iter()
            .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), evaluated))
            .collect()
    }

    /// Every diagram of every group becomes a channel, they are pooled into one
    /// mixture, and the peripheral ones are floored at the scale the cuts imply.
    ///
    /// The floor is what *builds* those channels: at floor zero the same diagrams
    /// give an all-timelike tree, which is why it is passed rather than defaulted.
    #[test]
    fn every_diagram_of_every_group_becomes_a_floored_channel() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let card = llj_card();
        let groups = derive_flavor_groups(enumerate(LLJ, &m), &m, &evaluated, &card)
            .expect("flavour groups");
        let amps = bind_all(&groups, &evaluated);
        let pdf = probe_pdf();
        let integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
            .expect("integrand");

        assert_eq!(integ.channel_count(), 24);
        assert_eq!(integ.channel_grid_ndim(), 2 + 5);
        let ids: Vec<ChannelId> = integ.channel_ids().to_vec();
        for (g, group) in groups.groups().iter().enumerate() {
            for d in 0..group.diagrams().len() {
                assert!(ids.contains(&ChannelId {
                    group: g,
                    diagram: d
                }));
            }
        }
        assert_eq!(integ.spacelike_floor(), 400.0);

        let (mut floored, mut unfloored) = (0, 0);
        for g in groups.groups() {
            for d in g.diagrams() {
                let with = DiagramChannel::<f64>::from_diagram_regulated(
                    d,
                    &evaluated,
                    SQRT_S_HAD,
                    integ.spacelike_floor(),
                );
                let without =
                    DiagramChannel::<f64>::from_diagram_regulated(d, &evaluated, SQRT_S_HAD, 0.0);
                assert_eq!(
                    without.spine_pole(),
                    None,
                    "a three-body spine was built without a floor"
                );
                match with.spine_pole() {
                    Some(pole) => {
                        assert_eq!(pole, 400.0);
                        floored += 1;
                    }
                    None => unfloored += 1,
                }
            }
        }
        eprintln!("{floored} peripheral channels, {unfloored} all-timelike");
        assert!(
            floored > 0,
            "no channel carries a peripheral spine, so the floor is doing nothing here"
        );
    }

    /// The integrand's value at a point, against an assembly built from the process
    /// data alone.
    ///
    /// The oracle re-derives the `(τ, y)` map, both frames, the flux and the `2π`
    /// measure, and — decisively — takes the mirrored beam ordering from an
    /// **explicitly enumerated** `b a > …` subprocess evaluated at the *unreflected*
    /// point, with each member's `x·f` product formed directly from the parton
    /// distribution. So it shares neither [`FlavorGroup::luminosity`] nor
    /// [`FlavorGroup::mirror_into`] with the integrand: a dropped or mis-argued mirror
    /// term, a swapped beam ordering, or a lost spin/colour average all move it.
    ///
    /// What it cannot see: the phase-space weight, which it takes from its own copy of
    /// the same channel construction, and anything the cut filter and the parton
    /// distribution agree on being wrong about.
    #[test]
    fn a_point_reproduces_an_independently_assembled_integrand() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let card = llj_card();
        let groups = derive_flavor_groups(enumerate(LLJ, &m), &m, &evaluated, &card)
            .expect("flavour groups");
        let amps = bind_all(&groups, &evaluated);
        let pdf = probe_pdf();
        let integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
            .expect("integrand");

        // The mirrored ordering of each group, compiled from its own proc card.
        let mirrored: Vec<BoundAmplitude<f64>> = Vec::new();
        let mirror_evals: Vec<AmplitudeEvaluator> = groups
            .groups()
            .iter()
            .map(|g| {
                let set = g.diagram_set();
                let swapped = format!(
                    "{} {} > {} QCD=2 QED=2",
                    set.particles_in[1],
                    set.particles_in[0],
                    set.particles_out.join(" ")
                );
                let mirror_set = enumerate(&swapped, &m)
                    .into_iter()
                    .find(|s| !s.diagrams.is_empty())
                    .expect("the mirrored ordering enumerates");
                compile_class(&mirror_set, &m, &evaluated).expect("mirror compiles")
            })
            .collect();
        drop(mirrored);
        let mirror_bound: Vec<BoundAmplitude<f64>> = mirror_evals
            .iter()
            .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
            .collect();
        let direct_bound = bind_all(&groups, &evaluated);

        let combiner = rebuild_channels(&groups, &evaluated, 400.0);
        let cuts = groups.groups()[0].cuts();
        let lips_2pi = (2.0 * PI).powi(4 - 3 * 3);
        let tau_min = cuts.shat_min() / (SQRT_S_HAD * SQRT_S_HAD);
        let q2 = [MU_F * MU_F, MU_F * MU_F];

        let mut stream = SubStream::from_stream(0xBEEF_0D1E, 7);
        let (mut worst, mut checked, mut rejected, mut mirror_weight) = (0.0f64, 0, 0, 0.0f64);
        for trial in 0..400 {
            let u = stream.uniforms::<f64>(integ.channel_grid_ndim());
            let channel = trial % integ.channel_count();

            let tau = tau_min.powf(1.0 - u[0]);
            let y_max = -0.5 * tau.ln();
            let y = (2.0 * u[1] - 1.0) * y_max;
            let (x1, x2) = (tau.sqrt() * y.exp(), tau.sqrt() * (-y).exp());
            let sqrt_shat = (tau * SQRT_S_HAD * SQRT_S_HAD).sqrt();
            let jac = (1.0 / tau_min).ln() * 2.0 * y_max;

            let point = combiner.sample_channel_at(channel, sqrt_shat, &u[2..]);
            let e_cm = sqrt_shat / 2.0;
            let mut cm = vec![V::new(e_cm, 0.0, 0.0, e_cm), V::new(e_cm, 0.0, 0.0, -e_cm)];
            cm.extend_from_slice(&point.momenta);
            let e_beam = SQRT_S_HAD / 2.0;
            let beta = (x1 - x2) / (x1 + x2);
            let mut lab = vec![
                V::new(x1 * e_beam, 0.0, 0.0, x1 * e_beam),
                V::new(x2 * e_beam, 0.0, 0.0, -x2 * e_beam),
            ];
            lab.extend(point.momenta.iter().map(|p| boost_z(*p, beta)));

            let expected = if !cuts.pass(&lab) {
                rejected += 1;
                0.0
            } else {
                let mut acc = 0.0;
                for (gi, g) in groups.groups().iter().enumerate() {
                    let mut scratch = direct_bound[gi].scratch_space();
                    let m2_direct = direct_bound[gi].eval_m2(&cm, &mut scratch);
                    let mut mirror_scratch = mirror_bound[gi].scratch_space();
                    let m2_mirror = mirror_bound[gi].eval_m2(&cm, &mut mirror_scratch);
                    let mut term = 0.0;
                    for s in g.members() {
                        let [a, b] = s.incoming;
                        term += pdf.xfx_q2(a, x1, q2[0]) * pdf.xfx_q2(b, x2, q2[1]) * m2_direct;
                        if a != b {
                            let w = pdf.xfx_q2(b, x1, q2[0]) * pdf.xfx_q2(a, x2, q2[1]);
                            term += w * m2_mirror;
                            mirror_weight = mirror_weight
                                .max(w * m2_mirror / (term).abs().max(f64::MIN_POSITIVE));
                        }
                    }
                    acc += g.spin_color_average() * term;
                }
                let flux = 1.0 / (2.0 * sqrt_shat * sqrt_shat);
                jac * flux * lips_2pi * acc * point.weight
            };

            let got = integ.value_in_channel(channel, &u);
            let rel = (got - expected).abs() / expected.abs().max(f64::MIN_POSITIVE);
            worst = worst.max(rel);
            checked += 1;
        }
        eprintln!(
            "pointwise worst {worst:.3e} over {checked} points ({rejected} cut-rejected); \
             the mirror carries up to {mirror_weight:.3} of a group's term"
        );
        assert!(
            checked - rejected > 30,
            "only {} of {checked} points landed inside the cuts",
            checked - rejected
        );
        assert!(
            rejected > 0,
            "no point was cut-rejected, so the zero branch is unchecked"
        );
        assert!(
            mirror_weight > 0.1,
            "the mirror term is worth only {mirror_weight:.3e} of a group's contribution, so \
             this oracle could not see it dropped"
        );
        assert!(worst < 1e-13, "pointwise disagreement {worst:.3e}");
    }

    /// A bound amplitude paired with the wrong group is refused. Crossing the pairing
    /// weights one group's matrix element with another's luminosity — a smooth shift
    /// of the cross section with no other symptom — so the check is by pointer
    /// identity, not by shape.
    #[test]
    fn a_crossed_group_amplitude_pairing_is_refused() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let card = llj_card();
        let groups = derive_flavor_groups(enumerate(LLJ, &m), &m, &evaluated, &card)
            .expect("flavour groups");
        let pdf = probe_pdf();

        let mut crossed: Vec<BoundAmplitude<f64>> = bind_all(&groups, &evaluated);
        crossed.swap(0, 1);
        let err = match ProtonIntegrand::new(&groups, &crossed, &evaluated, &pdf, SQRT_S_HAD, MU_F)
        {
            Ok(_) => panic!("a crossed amplitude pairing must be refused"),
            Err(e) => e,
        };
        assert!(
            matches!(err, ProtonError::AmplitudeMismatch { index: 0 }),
            "unexpected refusal: {err}"
        );

        let short = &bind_all(&groups, &evaluated)[..2];
        assert!(matches!(
            ProtonIntegrand::new(&groups, short, &evaluated, &pdf, SQRT_S_HAD, MU_F),
            Err(ProtonError::AmplitudeCount { amps: 2, groups: 6 })
        ));
    }

    /// The fixed-scale card resolves to constants and reads `αs` from the set's own
    /// tabulation; the dynamical card of the same process is refused at setup, not at
    /// the first VEGAS point.
    #[test]
    fn a_dynamical_scale_is_refused_where_the_fixed_one_resolves() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let card = llj_card();
        let groups = derive_flavor_groups(enumerate(LLJ, &m), &m, &evaluated, &card)
            .expect("flavour groups");
        let amps = bind_all(&groups, &evaluated);
        let pdf = probe_pdf();
        let info = probe_alpha_s();

        let mut integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
            .expect("integrand");
        let report = integ
            .use_run_card_scales(&m, &evaluated, &card, Some(&info))
            .expect("the fixed prescription compiles");
        let constant = report
            .constant_scales
            .expect("the reference card fixes every scale");
        assert_eq!((constant.mu_r, constant.mu_f), (MU_F, [MU_F, MU_F]));
        assert!(
            report.depends_on_alpha_s,
            "a QCD=2 matrix element must move with the strong coupling"
        );
        // The coupling is the grid's, not the parameter card's: the card's own
        // `pdlabel` is what routes it there, and a beta-function solve is refused.
        let grid_alpha_s = report.constant_alpha_s.expect("a coupling was installed");
        assert!((grid_alpha_s - 0.1300028).abs() < 1e-6, "{grid_alpha_s}");
        assert!(
            (grid_alpha_s - report.alpha_s_ref.expect("bound coupling")).abs() > 1e-4,
            "the grid coupling coincides with the parameter card's, so this cannot tell \
             which source was used"
        );
        assert!(
            integ
                .use_run_card_scales(&m, &evaluated, &card, None)
                .is_err(),
            "a card whose alpha_s lives in the set must refuse when the set is withheld"
        );

        let mut text = String::new();
        for (name, value) in [
            ("fixed_ren_scale", "False"),
            ("fixed_fac_scale1", "False"),
            ("fixed_fac_scale2", "False"),
        ] {
            text.push_str(&format!("  {value} = {name}\n"));
        }
        let dynamical = RunCard::parse(&format!(
            "{}{text}",
            "  1 = lpp1\n  1 = lpp2\n  6500.0 = ebeam1\n  6500.0 = ebeam2\n\
             \x20 lhapdf = pdlabel\n  247000 = lhaid\n  -1 = dynamical_scale_choice\n\
             \x20 20.0 = ptj\n  10.0 = ptl\n  5.0 = etaj\n  2.5 = etal\n\
             \x20 0.4 = drll\n  0.4 = drjl\n  50.0 = mmll\n  4 = maxjetflavor\n"
        ))
        .expect("dynamical run card");
        let mut integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
            .expect("integrand");
        let err = match integ.use_run_card_scales(&m, &evaluated, &dynamical, Some(&info)) {
            Ok(report) => panic!("a clustered scale on three outgoing legs resolved: {report:?}"),
            Err(e) => e.to_string(),
        };
        assert!(
            err.contains("clustering") || err.contains("t-channel"),
            "unexpected refusal: {err}"
        );
    }

    /// The `ŝ` window the cuts *hint* is looser than the one they impose, and the
    /// integrand's `τ` map is built on the hint. Both halves matter: a hint above the
    /// true threshold would cut real phase space, and one far below it would leave the
    /// grid to find the ridge alone.
    ///
    /// The measurement below is the second half — what the looseness actually costs,
    /// and that every draw it admits below threshold is rejected by the cuts rather
    /// than contributing.
    #[test]
    fn the_tau_map_is_loose_below_the_true_threshold_by_a_measured_margin() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let card = llj_card();
        let groups = derive_flavor_groups(enumerate(LLJ, &m), &m, &evaluated, &card)
            .expect("flavour groups");
        let amps = bind_all(&groups, &evaluated);
        let pdf = probe_pdf();
        let integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
            .expect("integrand");

        // A jet above `ptj` recoiling against a dilepton pair above `mmll` needs
        // `√ŝ ≥ √(mmll² + ptj²) + ptj`.
        let (mmll, ptj) = (50.0f64, 20.0f64);
        let shat_true = ((mmll * mmll + ptj * ptj).sqrt() + ptj).powi(2);
        let shat_hint = integ.tau_min() * SQRT_S_HAD * SQRT_S_HAD;
        assert!(
            shat_hint <= shat_true,
            "the hint {shat_hint} is above the true threshold {shat_true}, so the map cuts \
             physical phase space"
        );
        eprintln!(
            "shat_min hint {shat_hint:.1} vs true {shat_true:.1}: loose by {:.2}x, \
             {:.2} of ln(1/tau_min) = {:.2}",
            shat_true / shat_hint,
            (shat_true / shat_hint).ln(),
            (1.0 / integ.tau_min()).ln()
        );

        let mut stream = SubStream::from_stream(0x7A0_1111, 3);
        let (mut below, mut n) = (0usize, 4000usize);
        for i in 0..n {
            let u = stream.uniforms::<f64>(integ.channel_grid_ndim());
            let tau = integ.tau_min().powf(1.0 - u[0]);
            if tau * SQRT_S_HAD * SQRT_S_HAD < shat_true {
                below += 1;
                let v = integ.value_in_channel(i % integ.channel_count(), &u);
                assert_eq!(
                    v, 0.0,
                    "a point below the true threshold carried weight {v}"
                );
            }
        }
        n = n.max(1);
        let share = below as f64 / n as f64;
        eprintln!(
            "{:.1}% of tau draws land below the true threshold",
            100.0 * share
        );
        assert!(
            share < 0.1,
            "{:.1}% of draws are wasted below threshold; the hint is too loose to leave alone",
            100.0 * share
        );
    }
}
