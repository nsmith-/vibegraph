//! Compiled per-process phase-space cut filter, matching the conventions of
//! MadGraph's generated `SubProcesses/cuts.f` (LO template).
//!
//! [`Cuts::compile`] classifies each final-state leg into MadGraph's letter
//! classes and bakes the active thresholds into a flat check list;
//! [`Cuts::pass`] is the phase-space indicator used by the integrand.
//!
//! ## Conventions pinned against the LO `cuts.f` / `kin_functions.f`
//!
//! - **Rapidity, not pseudorapidity.** The single-leg `eta{j,b,a,l}` cut and the
//!   ΔR separation both use `rap(p) = ½·ln((E+pz)/(E−pz))`
//!   (`kin_functions.f:95` `rap`; `cuts.f:426` applies `abs(rap)`). For massless
//!   legs this equals pseudorapidity, but the enforced definition is rapidity.
//!   The lab-frame boost of the partonic CM (`cm_rap` in `rap`) is assumed
//!   already applied to the momenta passed to [`Cuts::pass`].
//! - **ΔR² = Δφ² + Δy²** (`kin_functions.f:42` `R2`), with
//!   `Δφ = acos(clamp((px₁px₂+py₁py₂)/(|pt₁||pt₂|), ±0.99999999))`
//!   (`kin_functions.f:180` `DELTA_PHI`) — the azimuthal opening angle in
//!   `[0, π]`, so wrap-around is intrinsic. `setcuts.f:345` stores the raw `dr`
//!   value, but the `cuts.f` FIRSTTIME block squares it once
//!   (`r2min = r2min·|r2min|`, `cuts.f:219-221`, "Since r2 returns distance
//!   squared") before the `r2(...) < r2min` comparison (`cuts.f:429`), so the
//!   effective bound is the standard `ΔR ≥ dr`. Stored here as a signed square
//!   `dr·|dr|`, exactly like the mass/ptll thresholds.
//! - **Invariant-mass / ptll thresholds are signed squares.** `mm{...}` is
//!   stored as `mm·|mm|` and compared against `(p_i+p_j)²`
//!   (`setcuts.f:399`, `cuts.f:485`); `ptll` as `ptll·|ptll|` against
//!   `(Σpx)²+(Σpy)²` (`setcuts.f:479`, `cuts.f:462`).
//! - **ŝ window** compares `(p₁+p₂)²` against `dsqrt_shat²` /
//!   `dsqrt_shatmax²` (`cuts.f:312`).
//! - **Class membership** (`setcuts.f:217`): a final leg is a jet if
//!   `|pdg| ≤ min(maxjetflavor,6)` or `|pdg| = 21`; a b if
//!   `maxjetflavor < |pdg| ≤ 5`; a charged lepton if `|pdg| ∈ {11,13,15}`; a
//!   photon if `pdg = 22`; a neutrino if `|pdg| ∈ {12,14,16}`. A leg is heavy
//!   if its mass exceeds 10 GeV. Single-leg cuts are skipped (`do_cuts = false`)
//!   for neutrinos and for masses above 20 GeV (`setcuts.f:212`).
//!
//! Cut families implemented here: ŝ window, single-leg pT/E/η (classes
//! j/b/a/l), pairwise ΔR and invariant mass, `ptll`, and `mmnl`. Every other
//! `cut=`-tagged parameter is parse-and-detect: [`Cuts::compile`] hard-errors
//! with [`CutError::UnimplementedCutActive`] if its value deviates from the MG
//! default, so an active but unimplemented cut is never silently ignored.

use thiserror::Error;

use crate::helas::repr::lorentz::LorentzVector;
use crate::helas::repr::Real;
use crate::runcard::{param_default, ParamValue, RunCard};

/// One external leg's identity, the classification input for [`Cuts::compile`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExternalLeg {
    /// PDG code (signed).
    pub pdg: i32,
    /// Pole mass in GeV.
    pub mass: f64,
    /// True for final-state legs (cuts apply); false for incoming beams.
    pub is_final: bool,
}

impl ExternalLeg {
    pub fn incoming(pdg: i32, mass: f64) -> Self {
        Self {
            pdg,
            mass,
            is_final: false,
        }
    }
    pub fn outgoing(pdg: i32, mass: f64) -> Self {
        Self {
            pdg,
            mass,
            is_final: true,
        }
    }
}

#[derive(Debug, Error)]
pub enum CutError {
    #[error(
        "run card activates cut '{name}' (= {value}, MG default {default}) which is not implemented; \
         implement it or restore the default before integrating"
    )]
    UnimplementedCutActive {
        name: String,
        value: String,
        default: String,
    },
}

/// The MadGraph letter class of a final-state leg used for pairwise cuts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Letter {
    Jet,
    B,
    Lepton,
    Photon,
}

#[derive(Clone, Copy, Debug)]
struct LegInfo {
    /// Index into the momentum slice given to [`Cuts::pass`].
    idx: usize,
    pdg: i32,
    letter: Option<Letter>,
    is_neutrino: bool,
    /// Single-leg and pairwise dr/mass cuts apply (mass ≤ 20 GeV, not a neutrino).
    do_cuts: bool,
}

#[derive(Clone, Copy, Debug)]
struct SingleLegCut {
    idx: usize,
    pt_min: f64,
    pt_max: f64,
    e_min: f64,
    e_max: f64,
    eta_min: f64,
    eta_max: f64,
}

#[derive(Clone, Copy, Debug)]
struct PairCut {
    i: usize,
    j: usize,
    /// signed-square ΔR lower bound (`dr·|dr|`); 0 = inactive.
    dr2_min: f64,
    /// signed-square ΔR upper bound (`dr·|dr|`); < 0 = inactive.
    dr2_max: f64,
    /// signed-square invariant-mass lower bound; 0 = inactive.
    m2_min: f64,
    /// signed-square invariant-mass upper bound; < 0 = inactive.
    m2_max: f64,
    /// signed-square pair-pT lower bound; 0 = inactive.
    ptll2_min: f64,
    /// signed-square pair-pT upper bound; < 0 = inactive.
    ptll2_max: f64,
}

impl PairCut {
    fn is_active(&self) -> bool {
        self.dr2_min > 0.0
            || self.dr2_max >= 0.0
            || self.m2_min != 0.0
            || self.m2_max >= 0.0
            || self.ptll2_min > 0.0
            || self.ptll2_max >= 0.0
    }
}

/// Combined-lepton-plus-neutrino invariant-mass window (`mmnl`).
#[derive(Clone, Debug)]
struct MmnlCut {
    min: f64,
    max: f64,
    /// Indices of final-state leptons and neutrinos whose momenta are summed.
    members: Vec<usize>,
}

/// A compiled, process-specific cut filter.
#[derive(Clone, Debug)]
pub struct Cuts {
    /// Indices of incoming legs (their momentum sum defines ŝ).
    incoming: Vec<usize>,
    shat_min_sq: f64,
    shat_max_sq: f64,
    single: Vec<SingleLegCut>,
    pairs: Vec<PairCut>,
    mmnl: Option<MmnlCut>,
    shat_min_hint: f64,
}

/// Cut parameters implemented by [`Cuts::compile`]; every other `cut=`-tagged
/// parameter is parse-and-detect.
const UNIMPLEMENTED_CUTS: &[&str] = &[
    "misset",
    "missetmax",
    "ptheavy",
    "ptonium",
    "etaonium",
    "xptj",
    "xptb",
    "xpta",
    "xptl",
    "ptj1min",
    "ptj1max",
    "ptj2min",
    "ptj2max",
    "ptj3min",
    "ptj3max",
    "ptj4min",
    "ptj4max",
    "cutuse",
    "ptl1min",
    "ptl1max",
    "ptl2min",
    "ptl2max",
    "ptl3min",
    "ptl3max",
    "ptl4min",
    "ptl4max",
    "htjmin",
    "htjmax",
    "ihtmin",
    "ihtmax",
    "ht2min",
    "ht3min",
    "ht4min",
    "ht2max",
    "ht3max",
    "ht4max",
    "ptgmin",
    "xetamin",
    "deltaeta",
    "ktdurham",
    "dparameter",
    "ptlund",
    "xqcut",
    "pt_min_pdg",
    "pt_max_pdg",
    "E_min_pdg",
    "E_max_pdg",
    "eta_min_pdg",
    "eta_max_pdg",
    "mxx_min_pdg",
];

/// signed square used by MadGraph for mass/ptll thresholds: `x·|x|`.
fn signed_sq(x: f64) -> f64 {
    x * x.abs()
}

impl Cuts {
    /// Compile the run card's cuts for a specific external-leg assignment.
    pub fn compile(rc: &RunCard, legs: &[ExternalLeg]) -> Result<Cuts, CutError> {
        detect_unimplemented(rc)?;

        let maxjetflavor = rc.maxjetflavor;
        let incoming: Vec<usize> = legs
            .iter()
            .enumerate()
            .filter(|(_, l)| !l.is_final)
            .map(|(i, _)| i)
            .collect();

        let infos: Vec<LegInfo> = legs
            .iter()
            .enumerate()
            .filter(|(_, l)| l.is_final)
            .map(|(idx, l)| classify(idx, l, maxjetflavor))
            .collect();

        // Single-leg pT / E / η for classes j/b/a/l.
        let mut single = Vec::new();
        for info in &infos {
            if !info.do_cuts {
                continue;
            }
            let Some(letter) = info.letter else { continue };
            let c = letter_char(letter);
            let pt_min = if letter == Letter::Photon {
                rc.float("pta").max(rc.float("ptgmin"))
            } else {
                rc.float(&format!("pt{c}"))
            };
            single.push(SingleLegCut {
                idx: info.idx,
                pt_min,
                pt_max: rc.float(&format!("pt{c}max")),
                e_min: rc.float(&format!("e{c}")),
                e_max: rc.float(&format!("e{c}max")),
                eta_min: rc.float(&format!("eta{c}min")),
                eta_max: rc.float(&format!("eta{c}")),
            });
        }

        // Pairwise ΔR / invariant mass / ptll.
        let mut pairs = Vec::new();
        for a in 0..infos.len() {
            for b in (a + 1)..infos.len() {
                let li = infos[a];
                let lj = infos[b];
                let (dr2_min, dr2_max) = pair_dr(rc, li, lj);
                let (m2_min, m2_max) = pair_mass(rc, li, lj);
                let (ptll2_min, ptll2_max) = pair_ptll(rc, li, lj);
                let pc = PairCut {
                    i: li.idx,
                    j: lj.idx,
                    dr2_min,
                    dr2_max,
                    m2_min,
                    m2_max,
                    ptll2_min,
                    ptll2_max,
                };
                if pc.is_active() {
                    pairs.push(pc);
                }
            }
        }

        // mmnl: invariant mass of the summed lepton + neutrino system.
        let mmnl = rc.float("mmnl");
        let mmnlmax = rc.float("mmnlmax");
        let mmnl_cut = if mmnl > 0.0 || mmnlmax >= 0.0 {
            let members: Vec<usize> = infos
                .iter()
                .filter(|i| i.letter == Some(Letter::Lepton) || i.is_neutrino)
                .map(|i| i.idx)
                .collect();
            (!members.is_empty()).then_some(MmnlCut {
                min: mmnl,
                max: mmnlmax,
                members,
            })
        } else {
            None
        };

        let dsqrt_shat = rc.float("dsqrt_shat");
        let dsqrt_shatmax = rc.float("dsqrt_shatmax");
        let shat_min_hint = shat_min_hint(rc, &infos);

        Ok(Cuts {
            incoming,
            shat_min_sq: dsqrt_shat * dsqrt_shat,
            shat_max_sq: if dsqrt_shatmax == -1.0 {
                -1.0
            } else {
                dsqrt_shatmax * dsqrt_shatmax
            },
            single,
            pairs,
            mmnl: mmnl_cut,
            shat_min_hint,
        })
    }

    /// A conservative lower bound on ŝ implied by the active cuts, for a
    /// hadronic x-integration mapping (`x_min = shat_min / s`). Never exceeds the
    /// true ŝ of a surviving point.
    pub fn shat_min(&self) -> f64 {
        self.shat_min_hint
    }

    /// Phase-space indicator: `true` iff `momenta` (all external legs, in the
    /// order given to [`Cuts::compile`]) pass every compiled cut.
    pub fn pass<F: Real>(&self, momenta: &[LorentzVector<F>]) -> bool {
        // ŝ window.
        if self.shat_min_sq > 0.0 || self.shat_max_sq >= 0.0 {
            let mut sum = LorentzVector::<F>::new(F::zero(), F::zero(), F::zero(), F::zero());
            for &i in &self.incoming {
                sum = sum + momenta[i];
            }
            let shat = sum.m2();
            if shat < c::<F>(self.shat_min_sq) {
                return false;
            }
            if self.shat_max_sq >= 0.0 && shat > c::<F>(self.shat_max_sq) {
                return false;
            }
        }

        // Single-leg pT / E / η.
        for s in &self.single {
            let p = momenta[s.idx];
            let pt = transverse(p);
            if pt < c::<F>(s.pt_min) {
                return false;
            }
            if s.pt_max >= 0.0 && pt > c::<F>(s.pt_max) {
                return false;
            }
            let e = p.e();
            // MG uses a strict lower bound on energy (`p(0) .le. emin` fails).
            if e <= c::<F>(s.e_min) {
                return false;
            }
            if s.e_max >= 0.0 && e > c::<F>(s.e_max) {
                return false;
            }
            let absrap = rapidity(p).abs();
            if s.eta_max >= 0.0 && absrap > c::<F>(s.eta_max) {
                return false;
            }
            if absrap < c::<F>(s.eta_min) {
                return false;
            }
        }

        // Pairwise ΔR / invariant mass / ptll.
        for pc in &self.pairs {
            let pi = momenta[pc.i];
            let pj = momenta[pc.j];
            if pc.dr2_min > 0.0 || pc.dr2_max >= 0.0 {
                let r2 = delta_r2(pi, pj);
                if r2 < c::<F>(pc.dr2_min) {
                    return false;
                }
                if pc.dr2_max >= 0.0 && r2 > c::<F>(pc.dr2_max) {
                    return false;
                }
            }
            if pc.m2_min != 0.0 || pc.m2_max >= 0.0 {
                let m2 = (pi + pj).m2();
                if !mass_window_ok(m2, pc.m2_min, pc.m2_max) {
                    return false;
                }
            }
            if pc.ptll2_min > 0.0 || pc.ptll2_max >= 0.0 {
                let pt2 = pair_pt2(pi, pj);
                if pt2 < c::<F>(pc.ptll2_min) {
                    return false;
                }
                if pc.ptll2_max >= 0.0 && pt2 > c::<F>(pc.ptll2_max) {
                    return false;
                }
            }
        }

        // mmnl: mass of the combined lepton + neutrino system.
        if let Some(m) = &self.mmnl {
            let mut sum = LorentzVector::<F>::new(F::zero(), F::zero(), F::zero(), F::zero());
            for &i in &m.members {
                sum = sum + momenta[i];
            }
            let mass = sum.m2().sqrt();
            if mass < c::<F>(m.min) {
                return false;
            }
            if m.max >= 0.0 && mass > c::<F>(m.max) {
                return false;
            }
        }

        true
    }
}

fn detect_unimplemented(rc: &RunCard) -> Result<(), CutError> {
    for &name in UNIMPLEMENTED_CUTS {
        let (Some(cur), Some(def)) = (rc.get(name), param_default(name)) else {
            continue;
        };
        if *cur != def {
            return Err(CutError::UnimplementedCutActive {
                name: name.to_string(),
                value: describe(cur),
                default: describe(&def),
            });
        }
    }
    Ok(())
}

fn describe(v: &ParamValue) -> String {
    match v {
        ParamValue::Float(x) => x.to_string(),
        ParamValue::Int(i) => i.to_string(),
        ParamValue::Bool(b) => b.to_string(),
        ParamValue::Str(s) | ParamValue::Opaque(s) => format!("'{s}'"),
    }
}

fn classify(idx: usize, leg: &ExternalLeg, maxjetflavor: i64) -> LegInfo {
    let a = leg.pdg.unsigned_abs() as i64;
    let is_neutrino = matches!(a, 12 | 14 | 16);
    let letter = if a == 21 || a <= maxjetflavor.min(6) {
        Some(Letter::Jet)
    } else if a >= maxjetflavor + 1 && a <= 5 {
        Some(Letter::B)
    } else if matches!(a, 11 | 13 | 15) {
        Some(Letter::Lepton)
    } else if leg.pdg == 22 {
        Some(Letter::Photon)
    } else {
        None
    };
    // Neutrinos and heavy (> 20 GeV) resonances receive no single-leg / dr / mass cuts.
    let do_cuts = !is_neutrino && leg.mass <= 20.0;
    LegInfo {
        idx,
        pdg: leg.pdg,
        letter,
        is_neutrino,
        do_cuts,
    }
}

fn letter_char(l: Letter) -> char {
    match l {
        Letter::Jet => 'j',
        Letter::B => 'b',
        Letter::Lepton => 'l',
        Letter::Photon => 'a',
    }
}

/// The unordered class-pair tag (e.g. "jl") for two letters, in the order
/// MadGraph names its `dr`/`mm` parameters.
fn pair_tag(a: Letter, b: Letter) -> Option<&'static str> {
    use Letter::*;
    let key = |x: Letter| match x {
        Jet => 0,
        B => 1,
        Lepton => 2,
        Photon => 3,
    };
    let (lo, hi) = if key(a) <= key(b) { (a, b) } else { (b, a) };
    Some(match (lo, hi) {
        (Jet, Jet) => "jj",
        (B, B) => "bb",
        (Lepton, Lepton) => "ll",
        (Photon, Photon) => "aa",
        (Jet, B) => "bj",
        (Jet, Lepton) => "jl",
        (Jet, Photon) => "aj",
        (B, Lepton) => "bl",
        (B, Photon) => "ab",
        (Lepton, Photon) => "al",
        // `lo` has the not-greater class key, so the transposed pairs never occur.
        _ => unreachable!("class pair ordered by key"),
    })
}

fn pair_dr(rc: &RunCard, li: LegInfo, lj: LegInfo) -> (f64, f64) {
    if !(li.do_cuts && lj.do_cuts) {
        return (0.0, -1.0);
    }
    let (Some(la), Some(lb)) = (li.letter, lj.letter) else {
        return (0.0, -1.0);
    };
    let Some(tag) = pair_tag(la, lb) else {
        return (0.0, -1.0);
    };
    // MG stores the raw `dr` value (setcuts.f:345) then squares it once in the
    // cuts.f FIRSTTIME block (`r2min = r2min*dabs(r2min)`, cuts.f:219-221) before
    // comparing against the distance-squared `r2`. Mirror that as a signed square,
    // which also preserves the -1 disabled sentinel for the `max` threshold.
    (
        signed_sq(rc.float(&format!("dr{tag}"))),
        signed_sq(rc.float(&format!("dr{tag}max"))),
    )
}

fn pair_mass(rc: &RunCard, li: LegInfo, lj: LegInfo) -> (f64, f64) {
    if !(li.do_cuts && lj.do_cuts) {
        return (0.0, -1.0);
    }
    let (Some(la), Some(lb)) = (li.letter, lj.letter) else {
        return (0.0, -1.0);
    };
    // The ll invariant-mass cut only applies to same-flavour opposite-charge
    // lepton pairs (setcuts.f:396); other same-class pairs apply unconditionally.
    let applies = match (la, lb) {
        (Letter::Lepton, Letter::Lepton) => {
            li.pdg.unsigned_abs() == lj.pdg.unsigned_abs() && li.pdg * lj.pdg < 0
        }
        _ => la == lb, // jj / bb / aa
    };
    if !applies {
        return (0.0, -1.0);
    }
    let Some(tag) = pair_tag(la, lb) else {
        return (0.0, -1.0);
    };
    (
        signed_sq(rc.float(&format!("mm{tag}"))),
        signed_sq(rc.float(&format!("mm{tag}max"))),
    )
}

fn pair_ptll(rc: &RunCard, li: LegInfo, lj: LegInfo) -> (f64, f64) {
    // ptll applies to same-flavour opposite-charge lepton pairs, lepton-neutrino
    // pairs, and neutrino-neutrino pairs (setcuts.f:473), regardless of do_cuts.
    let is_ll = li.letter == Some(Letter::Lepton)
        && lj.letter == Some(Letter::Lepton)
        && li.pdg.unsigned_abs() == lj.pdg.unsigned_abs()
        && li.pdg * lj.pdg < 0;
    let is_nl = (li.is_neutrino && lj.letter == Some(Letter::Lepton))
        || (li.letter == Some(Letter::Lepton) && lj.is_neutrino);
    let is_nn = li.is_neutrino && lj.is_neutrino;
    if is_ll || is_nl || is_nn {
        (
            signed_sq(rc.float("ptllmin")),
            signed_sq(rc.float("ptllmax")),
        )
    } else {
        (0.0, -1.0)
    }
}

/// Conservative lower bound on ŝ from the active cuts:
/// `max(dsqrt_shat², min dilepton-mass², two-lepton back-to-back pT bound²)`.
fn shat_min_hint(rc: &RunCard, infos: &[LegInfo]) -> f64 {
    let dsqrt_shat = rc.float("dsqrt_shat");
    let mut hint = dsqrt_shat * dsqrt_shat;

    // A same-flavour opposite-charge lepton pair carries the mmll bound;
    // m_pair ≤ √ŝ makes mmll² a valid lower bound on ŝ.
    let mmll = rc.float("mmll");
    if mmll > 0.0 {
        let has_ll_pair = infos.iter().enumerate().any(|(a, li)| {
            infos.iter().skip(a + 1).any(|lj| {
                li.letter == Some(Letter::Lepton)
                    && lj.letter == Some(Letter::Lepton)
                    && li.pdg.unsigned_abs() == lj.pdg.unsigned_abs()
                    && li.pdg * lj.pdg < 0
            })
        });
        if has_ll_pair {
            hint = hint.max(mmll * mmll);
        }
    }

    // Exactly two final-state leptons back-to-back at LO: each with pT ≥ ptl
    // implies m_ll ≥ 2·ptl, hence ŝ ≥ (2·ptl)².
    let leptons: Vec<&LegInfo> = infos
        .iter()
        .filter(|i| i.letter == Some(Letter::Lepton))
        .collect();
    if infos.len() == 2 && leptons.len() == 2 {
        let ptl = rc.float("ptl");
        if ptl > 0.0 {
            let bound = 2.0 * ptl;
            hint = hint.max(bound * bound);
        }
    }

    hint
}

// ── kinematic helpers (mirroring kin_functions.f) ───────────────────────────

#[inline]
fn c<F: Real>(x: f64) -> F {
    F::from(x).expect("f64 threshold representable in F")
}

/// Transverse momentum `√(px² + py²)` (`kin_functions.f:212` `pt`).
fn transverse<F: Real>(p: LorentzVector<F>) -> F {
    (p.px() * p.px() + p.py() * p.py()).sqrt()
}

/// Rapidity `½·ln((E+pz)/(E−pz))` (`kin_functions.f:95` `rap`). When `E ≤ |pz|`
/// MadGraph returns a large negative sentinel so `|rap|` overflows any active η
/// window; we mirror that so such legs fail an active cut.
fn rapidity<F: Real>(p: LorentzVector<F>) -> F {
    let e = p.e();
    let pz = p.pz();
    if e <= pz.abs() {
        return c::<F>(-1e99);
    }
    c::<F>(0.5) * ((e + pz) / (e - pz)).ln()
}

/// Azimuthal opening angle in `[0, π]` (`kin_functions.f:164` `DELTA_PHI`).
fn delta_phi<F: Real>(pi: LorentzVector<F>, pj: LorentzVector<F>) -> F {
    let denom = (pi.px() * pi.px() + pi.py() * pi.py()).sqrt()
        * (pj.px() * pj.px() + pj.py() * pj.py()).sqrt();
    let cos = (pi.px() * pj.px() + pi.py() * pj.py()) / denom;
    let cos = cos.max(c::<F>(-0.99999999)).min(c::<F>(0.99999999));
    cos.acos()
}

/// ΔR² = Δφ² + Δy² (`kin_functions.f:42` `R2`).
fn delta_r2<F: Real>(pi: LorentzVector<F>, pj: LorentzVector<F>) -> F {
    let dphi = delta_phi(pi, pj);
    let dy = rapidity(pi) - rapidity(pj);
    dphi * dphi + dy * dy
}

/// Squared transverse momentum of the pair sum (`kin_functions.f:76` `PtDot`).
fn pair_pt2<F: Real>(pi: LorentzVector<F>, pj: LorentzVector<F>) -> F {
    let px = pi.px() + pj.px();
    let py = pi.py() + pj.py();
    px * px + py * py
}

/// Invariant-mass² window test replicating the normal/inverted branches of
/// `cuts.f:486`. `m2_min`/`m2_max` are signed squares; `m2_max < 0` disables the
/// upper bound.
fn mass_window_ok<F: Real>(m2: F, m2_min: f64, m2_max: f64) -> bool {
    if m2_min <= m2_max || m2_max < 0.0 {
        // Normal window: reject below min or above max.
        if m2 < c::<F>(m2_min) {
            return false;
        }
        if m2_max >= 0.0 && m2 > c::<F>(m2_max) {
            return false;
        }
        true
    } else {
        // Inverted window (veto band): reject strictly between max and min.
        !(m2 > c::<F>(m2_max) && m2 < c::<F>(m2_min))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type V = LorentzVector<f64>;

    fn card(text: &str) -> RunCard {
        RunCard::parse(text).unwrap()
    }

    /// Massless four-momentum from transverse momentum, rapidity, azimuth.
    /// For a massless leg rapidity equals the `y` argument exactly.
    fn lep(pt: f64, y: f64, phi: f64) -> V {
        V::new(pt * y.cosh(), pt * phi.cos(), pt * phi.sin(), pt * y.sinh())
    }

    fn beams(e: f64) -> (V, V) {
        (V::new(e, 0.0, 0.0, e), V::new(e, 0.0, 0.0, -e))
    }

    fn dy_legs() -> Vec<ExternalLeg> {
        vec![
            ExternalLeg::incoming(2, 0.0),
            ExternalLeg::incoming(-2, 0.0),
            ExternalLeg::outgoing(11, 0.0),
            ExternalLeg::outgoing(-11, 0.0),
        ]
    }

    /// [in1, in2, e-, e+] with the two leptons back-to-back, well clear of the
    /// default lepton cuts, and beams giving a comfortable ŝ.
    fn dy_momenta(l1: V, l2: V) -> Vec<V> {
        let (b1, b2) = beams(500.0);
        vec![b1, b2, l1, l2]
    }

    // ── single-leg pT ───────────────────────────────────────────────────
    #[test]
    fn pt_min_boundary() {
        // Isolate ptl: disable eta and dr.
        let cuts = Cuts::compile(&card("-1 = etal\n0 = drll\n"), &dy_legs()).unwrap();
        let other = lep(50.0, 0.0, std::f64::consts::PI);
        // ptl default = 10; pt < 10 fails, pt == 10 passes (strict `<`).
        assert!(!cuts.pass(&dy_momenta(lep(9.9, 0.0, 0.0), other)));
        assert!(cuts.pass(&dy_momenta(lep(10.0, 0.0, 0.0), other)));
        assert!(cuts.pass(&dy_momenta(lep(10.1, 0.0, 0.0), other)));
    }

    #[test]
    fn pt_max_boundary() {
        let cuts = Cuts::compile(&card("-1 = etal\n0 = drll\n40 = ptlmax\n"), &dy_legs()).unwrap();
        let other = lep(30.0, 0.0, std::f64::consts::PI);
        assert!(cuts.pass(&dy_momenta(lep(40.0, 0.0, 0.0), other)));
        assert!(!cuts.pass(&dy_momenta(lep(40.1, 0.0, 0.0), other)));
    }

    // ── single-leg E ────────────────────────────────────────────────────
    #[test]
    fn e_min_boundary_is_strict() {
        // el = 15; energy of a y=0 lepton is its pT. E <= el fails (strict).
        let cuts =
            Cuts::compile(&card("0 = ptl\n-1 = etal\n0 = drll\n15 = el\n"), &dy_legs()).unwrap();
        let other = lep(50.0, 0.0, std::f64::consts::PI);
        assert!(!cuts.pass(&dy_momenta(lep(14.0, 0.0, 0.0), other)));
        assert!(!cuts.pass(&dy_momenta(lep(15.0, 0.0, 0.0), other))); // equality fails
        assert!(cuts.pass(&dy_momenta(lep(16.0, 0.0, 0.0), other)));
    }

    #[test]
    fn e_max_boundary() {
        let cuts = Cuts::compile(
            &card("0 = ptl\n-1 = etal\n0 = drll\n30 = elmax\n"),
            &dy_legs(),
        )
        .unwrap();
        let other = lep(20.0, 0.0, std::f64::consts::PI);
        assert!(cuts.pass(&dy_momenta(lep(30.0, 0.0, 0.0), other)));
        assert!(!cuts.pass(&dy_momenta(lep(31.0, 0.0, 0.0), other)));
    }

    // ── single-leg η (rapidity) + sign symmetry ─────────────────────────
    #[test]
    fn eta_max_boundary_and_sign_symmetry() {
        // etal default 2.5; |y| > 2.5 fails, symmetric in sign.
        let cuts = Cuts::compile(&card("0 = ptl\n0 = drll\n"), &dy_legs()).unwrap();
        let other = lep(50.0, 0.0, std::f64::consts::PI);
        assert!(cuts.pass(&dy_momenta(lep(20.0, 2.4, 0.0), other)));
        assert!(!cuts.pass(&dy_momenta(lep(20.0, 2.6, 0.0), other)));
        assert!(cuts.pass(&dy_momenta(lep(20.0, -2.4, 0.0), other)));
        assert!(!cuts.pass(&dy_momenta(lep(20.0, -2.6, 0.0), other)));
    }

    #[test]
    fn eta_min_boundary() {
        // etalmin = 1.0; |y| < 1.0 fails.
        let cuts = Cuts::compile(
            &card("0 = ptl\n5 = etal\n0 = drll\n1.0 = etalmin\n"),
            &dy_legs(),
        )
        .unwrap();
        let other = lep(50.0, 3.0, std::f64::consts::PI);
        assert!(!cuts.pass(&dy_momenta(lep(20.0, 0.9, 0.0), other)));
        assert!(cuts.pass(&dy_momenta(lep(20.0, 1.1, 0.0), other)));
    }

    // ── pairwise ΔR incl. φ wrap-around ─────────────────────────────────
    #[test]
    fn delta_r_boundary() {
        // drll defaults to 0.4; the stored bound is the signed square dr·|dr| =
        // 0.16, compared against ΔR² (cuts.f FIRSTTIME squaring). With Δy = 0 the
        // effective cut is the standard ΔR ≥ 0.4: Δφ = 0.3 ⇒ ΔR² = 0.09 < 0.16
        // fails; Δφ = 0.5 ⇒ ΔR² = 0.25 > 0.16 passes.
        let cuts = Cuts::compile(&card("0 = ptl\n-1 = etal\n"), &dy_legs()).unwrap();
        assert!(!cuts.pass(&dy_momenta(lep(30.0, 0.0, 0.0), lep(30.0, 0.0, 0.3))));
        assert!(cuts.pass(&dy_momenta(lep(30.0, 0.0, 0.0), lep(30.0, 0.0, 0.5))));
    }

    #[test]
    fn delta_r_phi_wraparound() {
        // Two leptons straddling the 2π seam are actually close in azimuth
        // (opening angle ≈ 0.18 rad), so the drll cut must reject them.
        let cuts = Cuts::compile(&card("0 = ptl\n-1 = etal\n"), &dy_legs()).unwrap();
        let a = lep(30.0, 0.0, 0.1);
        let b = lep(30.0, 0.0, 6.2); // ≈ 2π − 0.083
        assert!(!cuts.pass(&dy_momenta(a, b)));
        // Sanity: the true opening angle ≈ 0.18 gives ΔR² well below the drll=0.4
        // bound (0.4² = 0.16), so it is rejected; the naive |Δφ| ≈ 6.1 would not.
        assert!(delta_r2(a, b) < 0.4 * 0.4);
    }

    // ── pairwise invariant mass ─────────────────────────────────────────
    #[test]
    fn mass_min_max_window() {
        // Back-to-back leptons at y=0: m_ll = 2·pT.
        let cuts = Cuts::compile(
            &card("0 = ptl\n-1 = etal\n0 = drll\n50 = mmll\n120 = mmllmax\n"),
            &dy_legs(),
        )
        .unwrap();
        let pair = |pt: f64| dy_momenta(lep(pt, 0.0, 0.0), lep(pt, 0.0, std::f64::consts::PI));
        assert!(!cuts.pass(&pair(24.0))); // m = 48 < 50
        assert!(cuts.pass(&pair(30.0))); // m = 60
        assert!(cuts.pass(&pair(60.0))); // m = 120 (== max, passes)
        assert!(!cuts.pass(&pair(61.0))); // m = 122 > 120
    }

    #[test]
    fn mass_cut_only_same_flavor_opposite_charge() {
        // Two same-charge leptons (e-, e-) are not an mmll pair → no mass cut.
        let legs = vec![
            ExternalLeg::incoming(2, 0.0),
            ExternalLeg::incoming(-2, 0.0),
            ExternalLeg::outgoing(11, 0.0),
            ExternalLeg::outgoing(11, 0.0),
        ];
        let cuts =
            Cuts::compile(&card("0 = ptl\n-1 = etal\n0 = drll\n1000 = mmll\n"), &legs).unwrap();
        // m = 60 would fail an active mmll, but the cut does not apply here.
        assert!(cuts.pass(&dy_momenta(
            lep(30.0, 0.0, 0.0),
            lep(30.0, 0.0, std::f64::consts::PI)
        )));
    }

    // ── ŝ window ────────────────────────────────────────────────────────
    #[test]
    fn shat_window() {
        let cuts =
            Cuts::compile(&card("100 = dsqrt_shat\n200 = dsqrt_shatmax\n"), &dy_legs()).unwrap();
        // Final-state leptons chosen to pass default single-leg + dr cuts.
        let l1 = lep(40.0, 0.0, 0.0);
        let l2 = lep(40.0, 0.0, std::f64::consts::PI);
        let with = |e: f64| {
            let (b1, b2) = beams(e);
            vec![b1, b2, l1, l2]
        };
        assert!(!cuts.pass(&with(40.0))); // √ŝ = 80 < 100
        assert!(cuts.pass(&with(75.0))); // √ŝ = 150 ∈ [100,200]
        assert!(!cuts.pass(&with(120.0))); // √ŝ = 240 > 200
    }

    // ── ptll (dilepton-system pT) ───────────────────────────────────────
    #[test]
    fn ptll_window() {
        // Both leptons at φ = 0: pair pT = pt1 + pt2.
        let cuts = Cuts::compile(
            &card("0 = ptl\n-1 = etal\n0 = drll\n20 = ptllmin\n"),
            &dy_legs(),
        )
        .unwrap();
        let pair = |pt: f64| dy_momenta(lep(pt, 0.0, 0.0), lep(pt, 0.0, 0.0));
        assert!(!cuts.pass(&pair(5.0))); // pair pT = 10 < 20
        assert!(cuts.pass(&pair(15.0))); // pair pT = 30 > 20
    }

    // ── mmnl (lepton + neutrino system) ─────────────────────────────────
    #[test]
    fn mmnl_window() {
        // e- + ν_e; the mmnl cut is on the mass of their sum.
        let legs = vec![
            ExternalLeg::incoming(2, 0.0),
            ExternalLeg::incoming(-1, 0.0),
            ExternalLeg::outgoing(11, 0.0),
            ExternalLeg::outgoing(12, 0.0),
        ];
        let cuts =
            Cuts::compile(&card("0 = ptl\n-1 = etal\n0 = drll\n50 = mmnl\n"), &legs).unwrap();
        let sys = |pt: f64| {
            let (b1, b2) = beams(500.0);
            vec![
                b1,
                b2,
                lep(pt, 0.0, 0.0),
                lep(pt, 0.0, std::f64::consts::PI),
            ]
        };
        assert!(!cuts.pass(&sys(24.0))); // m = 48 < 50
        assert!(cuts.pass(&sys(30.0))); // m = 60 > 50
    }

    // ── shat_min hint ───────────────────────────────────────────────────
    #[test]
    fn shat_min_hint_combinations() {
        // Default DY: only the 2·ptl bound is active ⇒ (2·10)² = 400.
        let c0 = Cuts::compile(&RunCard::default(), &dy_legs()).unwrap();
        assert_eq!(c0.shat_min(), 400.0);

        // dsqrt_shat dominates.
        let c1 = Cuts::compile(&card("50 = dsqrt_shat\n"), &dy_legs()).unwrap();
        assert_eq!(c1.shat_min(), 2500.0);

        // mmll dominates.
        let c2 = Cuts::compile(&card("60 = mmll\n"), &dy_legs()).unwrap();
        assert_eq!(c2.shat_min(), 3600.0);

        // ptl bound when the others are small.
        let c3 = Cuts::compile(&card("25 = ptl\n"), &dy_legs()).unwrap();
        assert_eq!(c3.shat_min(), 2500.0); // (2·25)²
    }

    // ── class membership via maxjetflavor boundary ──────────────────────
    #[test]
    fn maxjetflavor_moves_b_to_jet() {
        // A pdg=5 final leg: b-class at maxjetflavor=4 (ptb=0, no cut) but a
        // jet at maxjetflavor=5 (ptj=20 applies).
        let legs = vec![
            ExternalLeg::incoming(2, 0.0),
            ExternalLeg::incoming(-2, 0.0),
            ExternalLeg::outgoing(5, 0.0),
            ExternalLeg::outgoing(-5, 0.0),
        ];
        let low_pt = |pt: f64| {
            let (b1, b2) = beams(500.0);
            // Large ΔR (back-to-back) so no dr cut interferes; drll only fires
            // for leptons anyway.
            vec![
                b1,
                b2,
                lep(pt, 0.0, 0.0),
                lep(pt, 0.0, std::f64::consts::PI),
            ]
        };
        let c_mjf4 = Cuts::compile(&card("20 = ptj\n"), &legs).unwrap();
        assert!(c_mjf4.pass(&low_pt(5.0))); // b-class, no pt cut

        let c_mjf5 = Cuts::compile(&card("20 = ptj\n5 = maxjetflavor\n"), &legs).unwrap();
        assert!(!c_mjf5.pass(&low_pt(5.0))); // jet-class, ptj=20 rejects
        assert!(c_mjf5.pass(&low_pt(25.0)));
    }

    #[test]
    fn photon_class_and_neutrino_suppression() {
        // Photon (pdg=22) gets the 'a' cuts (pta=10 default); neutrino (pdg=12)
        // receives no single-leg cut.
        let legs = vec![
            ExternalLeg::incoming(2, 0.0),
            ExternalLeg::incoming(-2, 0.0),
            ExternalLeg::outgoing(22, 0.0),
            ExternalLeg::outgoing(12, 0.0),
        ];
        let cuts = Cuts::compile(&RunCard::default(), &legs).unwrap();
        let (b1, b2) = beams(500.0);
        // Photon pT below pta=10 fails; neutrino pT is irrelevant.
        assert!(!cuts.pass(&vec![
            b1,
            b2,
            lep(5.0, 0.0, 0.0),
            lep(0.01, 0.0, std::f64::consts::PI)
        ]));
        assert!(cuts.pass(&vec![
            b1,
            b2,
            lep(15.0, 0.0, 0.0),
            lep(0.01, 0.0, std::f64::consts::PI)
        ]));
    }

    #[test]
    fn heavy_leg_gets_no_single_leg_cut() {
        // A Z (pdg=23, mass 91 > 20) is do_cuts=false.
        let legs = vec![
            ExternalLeg::incoming(2, 0.0),
            ExternalLeg::incoming(-2, 0.0),
            ExternalLeg::outgoing(23, 91.1876),
        ];
        let cuts = Cuts::compile(&RunCard::default(), &legs).unwrap();
        let (b1, b2) = beams(500.0);
        // Even a tiny-pT, high-rapidity Z passes (no single-leg cut applies).
        assert!(cuts.pass(&vec![b1, b2, lep(0.5, 6.0, 0.0)]));
    }

    // ── parse-and-detect of unimplemented active cuts ───────────────────
    #[test]
    fn default_card_compiles() {
        assert!(Cuts::compile(&RunCard::default(), &dy_legs()).is_ok());
    }

    #[test]
    fn unimplemented_active_cut_errors() {
        let err = Cuts::compile(&card("30 = ptj1min\n"), &dy_legs()).unwrap_err();
        match err {
            CutError::UnimplementedCutActive { name, .. } => assert_eq!(name, "ptj1min"),
        }
    }

    #[test]
    fn unimplemented_cut_at_default_is_ok() {
        // ptj1min at its default (0) does not trip the detector.
        assert!(Cuts::compile(&card("0 = ptj1min\n"), &dy_legs()).is_ok());
    }

    #[test]
    fn misset_activation_errors() {
        let err = Cuts::compile(&card("15 = misset\n"), &dy_legs()).unwrap_err();
        assert!(matches!(err, CutError::UnimplementedCutActive { .. }));
    }
}
