//! Hadronic leading-order Drell–Yan cross section: `p p → e⁺ e⁻` assembled from
//! the parton distributions ([`crate::pdf`]), the compiled helicity amplitude
//! ([`crate::helas::eval`]), the run-card cut filter ([`crate::cuts`]), and the
//! VEGAS integrator ([`crate::vegas`]).
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
//! VEGAS samples `(u₁, u₂, u₃) ∈ [0,1]³`, mapped to `(x₁, x₂, cosθ)` by
//!
//! ```text
//! xᵢ = x_min^(1−uᵢ)   (ln xᵢ uniform),   dxᵢ/duᵢ = xᵢ · ln(1/x_min)
//! cosθ = 2u₃ − 1,                        d(cosθ)/du₃ = 2
//! ```
//!
//! with `x_min = ŝ_min / s` from the cut hint ([`Cuts::shat_min`]). Because the
//! LHAPDF grid returns `x·f(x)` and the logarithmic map carries a Jacobian factor
//! `xᵢ`, the `1/xᵢ` in `f = (x·f)/x` cancels: the luminosity is built directly
//! from `x·f` products and each x-integration contributes a bare `ln(1/x_min)`.
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

use rand::SeedableRng;
use thiserror::Error;

use crate::cuts::{CutError, Cuts, ExternalLeg};
use crate::diagrams::{
    generate_from_proc_card, parse_proc_card, DiagramError, DiagramSet, ParsingOptions,
};
use crate::helas::eval::{AmplitudeEvaluator, BoundAmplitude, ScratchSpace};
use crate::helas::repr::lorentz::LorentzVector;
use crate::pdf::PdfMember;
use crate::phasespace::{prefactor2, GEV2_TO_PB};
use crate::ufo::{EvaluatedModel, UFOModel};
use crate::vegas::{VegasGrid, VegasResult};

type V = LorentzVector<f64>;

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

/// Enumerate `p p → e⁺ e⁻` and partition the quark-initiated subprocesses into
/// up-type and down-type Z/γ coupling classes, asserting the partition matches
/// the expected massless 4-flavor content.
pub fn dy_flavor_classes(model: &UFOModel) -> Result<FlavorClasses, HadronicError> {
    let opts = ParsingOptions::default();
    let card = parse_proc_card("generate p p > e+ e-", &opts)?;
    let sets = generate_from_proc_card(&card, model)?;

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
    /// Total hadronic invariant `s = (E₁+E₂)²` (head-on beams).
    s_had: f64,
    /// Factorization scale squared `μF²`.
    mu_f2: f64,
    /// Lower support of the logarithmic τ = ŝ/s map, `ŝ_min / s`.
    tau_min: f64,
    ln_inv_tau_min: f64,
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

        // Partonic prefactor: flux 1/(2ŝ), spin average 1/4, color average 1/9,
        // and the 2-body LIPS Jacobian for the cosθ ↔ u₃ map — all but the 1/9 in
        // `prefactor2`, which is written for the initial-state-spin-summed 2→2.
        let phat = prefactor2(m.sqrt_shat) / 9.0;

        let m2_up = self.up.eval_m2(&cm, &mut self.up_scratch.borrow_mut());
        let m2_down = self.down.eval_m2(&cm, &mut self.down_scratch.borrow_mut());

        m.jac * phat * (m2_up * lum_up + m2_down * lum_down)
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
        let mut grid = VegasGrid::new(3, 64, 1.5);
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        grid.adapt(|u| self.value(u), neval, niter, &mut rng)
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
        let fc = dy_flavor_classes(&m).expect("classify DY");
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
        let fc = dy_flavor_classes(&m).unwrap();
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

        let integ = DrellYanIntegrand::new(
            &b_up,
            &b_down,
            &pdf,
            &cuts,
            fc.up_flavors,
            fc.down_flavors,
            13000.0,
            91.188,
        );
        // cosθ → 1 (u₃ ≈ 1) sends the leptons collinear with the beam: pT → 0
        // fails the ptl = 10 GeV cut, so the indicator zeros the integrand.
        assert_eq!(integ.value(&[0.5, 0.5, 0.99999]), 0.0);
        // A central, well-clear point at the ŝ floor is nonzero.
        assert!(integ.value(&[0.0, 0.5, 0.5]) > 0.0);
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
