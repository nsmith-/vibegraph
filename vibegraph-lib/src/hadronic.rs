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

use std::cell::RefCell;
use std::f64::consts::PI;

use rand::SeedableRng;
use thiserror::Error;

use crate::cuts::{CutError, Cuts, ExternalLeg};
use crate::diagrams::diagram::Diagram;
use crate::diagrams::{
    generate_from_proc_card, parse_proc_card, DiagramError, DiagramSet, ParsingOptions,
};
use crate::helas::eval::{AmplitudeEvaluator, BoundAmplitude, ScratchSpace};
use crate::helas::repr::lorentz::LorentzVector;
use crate::pdf::PdfMember;
use crate::phasespace::{
    lips2_jacobian_u, AlphaAdaptation, Channel, DiagramChannel, MultiChannel, PhaseSpaceMap,
    RamboChannel, GEV2_TO_PB,
};
use crate::ufo::{EvaluatedModel, UFOModel};
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
    up: &'a BoundAmplitude<'a, f64>,
    down: &'a BoundAmplitude<'a, f64>,
    up_scratch: RefCell<ScratchSpace<f64>>,
    down_scratch: RefCell<ScratchSpace<f64>>,
    pdf: &'a PdfMember,
    cuts: &'a Cuts,
    up_flavors: Vec<i32>,
    down_flavors: Vec<i32>,
    /// Initial-state spin×colour averaging factor, derived from the incoming
    /// quark–antiquark pair (`1/(2·2·3·3) = 1/36`).
    spin_color_avg: f64,
    /// Total hadronic invariant `s = (E₁+E₂)²` (head-on beams).
    s_had: f64,
    /// Factorization scale squared `μF²`.
    mu_f2: f64,
    /// Lower support of the logarithmic τ = ŝ/s map, `ŝ_min / s`.
    tau_min: f64,
    ln_inv_tau_min: f64,
}

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
    /// * `mu_f` — factorization scale.
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
            up_scratch: RefCell::new(up.scratch_space()),
            down_scratch: RefCell::new(down.scratch_space()),
            up,
            down,
            pdf,
            cuts,
            up_flavors,
            down_flavors,
            spin_color_avg,
            s_had,
            mu_f2: mu_f * mu_f,
            tau_min,
            ln_inv_tau_min: (1.0 / tau_min).ln(),
        }
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

        let lum_up = self.luminosity(&self.up_flavors, m.x1, m.x2);
        let lum_down = self.luminosity(&self.down_flavors, m.x1, m.x2);
        if lum_up == 0.0 && lum_down == 0.0 {
            return 0.0;
        }

        let phat = self.partonic_prefactor(m.sqrt_shat);

        let m2_up = self.up.eval_m2(&cm, &mut self.up_scratch.borrow_mut());
        let m2_down = self.down.eval_m2(&cm, &mut self.down_scratch.borrow_mut());

        m.jac * phat * (m2_up * lum_up + m2_down * lum_down)
    }

    /// The integrand decomposed into its physical factors at a VEGAS point — the
    /// target of the pointwise integrand oracle. `value` equals
    /// `jac · phat · (m2_up·lum_up + m2_down·lum_down)` when `pass`, else `0`.
    pub fn debug_factors(&self, u: &[f64]) -> PointFactors {
        let m = self.map_point(u);
        let Kinematics { cm, lab } =
            build_kinematics(m.sqrt_shat, m.cos_theta, m.x1, m.x2, self.s_had);
        let pass = self.cuts.pass(&lab);
        let lum_up = self.luminosity(&self.up_flavors, m.x1, m.x2);
        let lum_down = self.luminosity(&self.down_flavors, m.x1, m.x2);
        let phat = self.partonic_prefactor(m.sqrt_shat);
        let m2_up = self.up.eval_m2(&cm, &mut self.up_scratch.borrow_mut());
        let m2_down = self.down.eval_m2(&cm, &mut self.down_scratch.borrow_mut());
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
    fn luminosity(&self, flavors: &[i32], x1: f64, x2: f64) -> f64 {
        let mut acc = 0.0;
        for &q in flavors {
            let fq1 = self.pdf.xfx_q2(q, x1, self.mu_f2);
            let fq2 = self.pdf.xfx_q2(q, x2, self.mu_f2);
            let fqb1 = self.pdf.xfx_q2(-q, x1, self.mu_f2);
            let fqb2 = self.pdf.xfx_q2(-q, x2, self.mu_f2);
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

/// One compiled subprocess feeding a summed matrix element: a bound amplitude and
/// its own evaluation scratch (behind [`RefCell`] so the integrand is `Fn`).
struct BoundSubprocess<'a> {
    amp: &'a BoundAmplitude<'a, f64>,
    scratch: RefCell<ScratchSpace<f64>>,
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
/// σ̂ = 1/(2ŝ) · ⟨spin·colour avg⟩ · ∫ dΦ_n Σ|M|²
///    = 1/(2ŝ) · avg · (2π)^{4−3n} · ⟨weight · Σ_sub |M_sub|²⟩_uniform
/// ```
///
/// where `Σ|M|²` is the colour+helicity-summed matrix element ([`eval_m2`]) and
/// the `(2π)^{4−3n}` factor turns the map's invariant volume `R_n` into the full
/// `dΦ_n` measure.
///
/// [`eval_m2`]: BoundAmplitude::eval_m2
pub struct FixedBeamIntegrand<'a> {
    subs: Vec<BoundSubprocess<'a>>,
    cuts: &'a Cuts,
    sqrt_s: f64,
    /// Unit-hypercube → phase-space map over the outgoing legs, on the fixed `√ŝ`
    /// and masses: flat [`RamboChannel`] by default, a resonance-aware
    /// [`MultiChannel`] once [`use_multichannel`](Self::use_multichannel) has run.
    sampler: Box<dyn PhaseSpaceMap<f64>>,
    /// The outgoing pole masses in leg order, the map's targets.
    final_masses: Vec<f64>,
    /// `1 / Π_a (n_spin · n_colour)` over the incoming legs.
    spin_color_avg: f64,
    /// The `(2π)^{4−3n}` measure factor.
    lips_2pi: f64,
    /// Beam energy `√ŝ/2`.
    beam_e: f64,
    /// Grid-damping exponent for the VEGAS pass, following the active sampler:
    /// [`VEGAS_ALPHA`] over the raw flat map, [`VEGAS_ALPHA_MAPPED`] once a
    /// multichannel map has already flattened the integrand's known peaks.
    vegas_alpha: f64,
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
        let subs = amps
            .into_iter()
            .map(|amp| BoundSubprocess {
                amp,
                scratch: RefCell::new(amp.scratch_space()),
            })
            .collect();
        let sampler = Box::new(RamboChannel::new(sqrt_s, final_masses.clone()));
        FixedBeamIntegrand {
            subs,
            cuts,
            sqrt_s,
            sampler,
            final_masses,
            spin_color_avg,
            lips_2pi: (2.0 * PI).powi(4 - 3 * n as i32),
            beam_e: sqrt_s / 2.0,
            vegas_alpha: VEGAS_ALPHA,
        }
    }

    /// VEGAS dimensionality — the uniforms the active phase-space map consumes.
    /// `4n` for flat RAMBO; `3n − 3` (one channel-selection coordinate plus the
    /// `3n − 4` invariant/angle coordinates) for the multichannel combiner.
    pub fn vegas_ndim(&self) -> usize {
        self.sampler.ndim()
    }

    /// The colour+helicity-summed `Σ|M|²` at the outgoing momenta `momenta`, in the
    /// partonic-CM frame, with the beams prepended and the cut filter applied — the
    /// matrix-element part of the integrand as a function of the phase-space point.
    /// A configuration failing a cut returns exactly `0.0`, so it drops out of both
    /// the cross section and the α-adaptation survey.
    fn matrix_element(&self, momenta: &[V]) -> f64 {
        let mut ext: Vec<V> = Vec::with_capacity(2 + momenta.len());
        ext.push(V::new(self.beam_e, 0.0, 0.0, self.beam_e));
        ext.push(V::new(self.beam_e, 0.0, 0.0, -self.beam_e));
        ext.extend_from_slice(momenta);

        if !self.cuts.pass(&ext) {
            return 0.0;
        }

        let mut m2 = 0.0;
        for sub in &self.subs {
            m2 += sub.amp.eval_m2(&ext, &mut sub.scratch.borrow_mut());
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
        let flux = 1.0 / (2.0 * self.sqrt_s * self.sqrt_s);
        flux * self.spin_color_avg * self.lips_2pi * point.weight * m2
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
        self.sampler = Box::new(combiner);
        self.vegas_alpha = VEGAS_ALPHA_MAPPED;
        Some(report)
    }

    /// The final-state pole masses in outgoing-leg order.
    pub fn final_masses(&self) -> &[f64] {
        &self.final_masses
    }

    /// Integrate the cross section with VEGAS, returning `(σ, Δσ)` in picobarns.
    pub fn integrate(&self, neval: usize, niter: usize, seed: u64) -> (f64, f64) {
        let result = self.adapt_grid(neval, niter, seed).1;
        (result.integral * GEV2_TO_PB, result.std_dev * GEV2_TO_PB)
    }

    /// Run VEGAS adaptation, returning the trained grid alongside the result — the
    /// primitive the `integrate` CLI command serializes into its artifact.
    pub fn adapt_grid(&self, neval: usize, niter: usize, seed: u64) -> (VegasGrid, VegasResult) {
        let mut grid = VegasGrid::new(self.vegas_ndim(), VEGAS_NBINS, self.vegas_alpha);
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        let result = grid.adapt(|u| self.value(u), neval, niter, &mut rng);
        (grid, result)
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
