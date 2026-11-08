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
//!   (`kin_functions.f:164` `DELTA_PHI`, the clamp at `:181-182`) — the azimuthal opening angle in
//!   `[0, π]`, so wrap-around is intrinsic. `setcuts.f:346` stores the raw `dr`
//!   value, but the `cuts.f` FIRSTTIME block squares it once
//!   (`r2min = r2min·|r2min|`, `cuts.f:225` in `passcuts`, "Since r2 returns distance
//!   squared") before the `r2(...) < r2min` comparison (`cuts.f:443`), so the
//!   effective bound is the standard `ΔR ≥ dr`. Stored here as a signed square
//!   `dr·|dr|`, exactly like the mass/ptll thresholds.
//! - **Invariant-mass / ptll thresholds are signed squares.** `mm{...}` is
//!   stored as `mm·|mm|` and compared against `(p_i+p_j)²`
//!   (`setcuts.f:399`, `cuts.f:485`); `ptll` as `ptll·|ptll|` against
//!   `(Σpx)²+(Σpy)²` (`setcuts.f:479`, `cuts.f:462`).
//! - **ŝ window** compares `(p₁+p₂)²` against `dsqrt_shat²` /
//!   `dsqrt_shatmax²` (`cuts.f:310`), on a process with two incoming legs only:
//!   `cuts.f` guards it with `nincoming.eq.2`, so a decay, whose `ŝ` is its pole
//!   mass squared, never reads it.
//! - **Class membership** (`setcuts.f:217`): a final leg is a jet if
//!   `|pdg| ≤ min(maxjetflavor,6)` or `|pdg| = 21`; a b if
//!   `maxjetflavor < |pdg| ≤ 5`; a charged lepton if `|pdg| ∈ {11,13,15}`; a
//!   photon if `pdg = 22`; a neutrino if `|pdg| ∈ {12,14,16}`. A leg is heavy
//!   if its mass exceeds 10 GeV. Single-leg cuts are skipped (`do_cuts = false`)
//!   for neutrinos and for masses above 20 GeV (`setcuts.f:212`).
//!
//! - **Decay-chain windows** (`cut_bw`, `myamp.f:76`): a propagator a decay
//!   chain forces on shell (`gForceBW = 1`) rejects the point unless
//!   `|√(p²) − M| < bwcutoff·Γ`, with `Γ` floored at `M·small_width_treatment`
//!   (`myamp.f:170`); a line of zero width is never tested (`myamp.f:164`). The
//!   test is MadEvent's `this_config`'s forced lines. Every configuration of a
//!   decay chain without identical particles across its decays forces the same
//!   lines, so the cut is a property of the subprocess; where identical particles
//!   let different diagrams force different leg sets, a point passes when every
//!   forced line of *some* diagram is inside its window ([`ForcedResonances`]).
//! - **`cut_decays`** (`setcuts.f:192`): at `F`, a final-state leg descending
//!   from a forced line gets `do_cuts = false`, which switches off its single-leg
//!   cuts and every pairwise `ΔR` and invariant-mass cut it takes part in; `ptll`
//!   and `mmnl` are set without reading `do_cuts` (`setcuts.f:431`, `:473`) and
//!   stay on. MadEvent reads the forced lines of configuration 1
//!   (`check_decay`, `setcuts.f:995`); here a leg counts as a decay product when
//!   it descends from a forced line in every diagram, which is the same set on a
//!   card without identical particles across its decays. An exemption that would
//!   leave uncut a massless coloured leg descending from no forced line in some
//!   diagram is refused ([`CutError::IrUnsafeExemption`]): that leg's collinear
//!   region is divergent.
//! - **Massless soft and collinear limits**: a card whose diagrams reach a
//!   [`MasslessLimit`] that its compiled cuts leave open is refused, because σ
//!   diverges logarithmically there and a run reports how far its sampler
//!   reached. A final-state splitting of two massless legs off a massless
//!   propagator (`q → q g`, `g → g g`, `q → q γ`, `ℓ → ℓ γ`, `γ* → f f̄`) needs a
//!   ΔR or invariant-mass minimum on the pair, which transverse-momentum cuts
//!   cannot replace ([`CutError::UncutCollinearSplitting`], and
//!   [`CutError::UncutPhotonSplitting`] for a photon and a parton, where photon
//!   isolation, MadGraph's other regulator, is not implemented). A soft massless
//!   vector boson needs its own `pT` or energy minimum, which a ΔR cut cannot
//!   replace ([`CutError::UncutSoftEmission`]). An emission collinear with a
//!   beam through a massless spacelike propagator needs the leg's `pT` minimum or
//!   a rapidity maximum ([`CutError::UncutInitialCollinear`]), whatever the
//!   exchanged line: through a t-channel photon off a massless quark it is the
//!   equivalent-photon limit, whose `∫ dt/t` reaches `t = 0`. Partons above
//!   `maxjetflavor` take the b-class cuts and are read
//!   the same way. Only diagram-aware compilation ([`Cuts::compile_with`] with
//!   [`ForcedResonances::of`]) sees the limits.
//!
//! - **HT** (`cuts.f:874-913`): `htj` is the scalar sum of the transverse
//!   momenta of the jet-class legs, `iht` that sum plus the b-class legs, so
//!   which partons each counts moves with `maxjetflavor`. No pT or rapidity
//!   preselection applies, and `do_cuts` is not read: decay products and heavy
//!   jet-class legs count. A sum below its minimum fails (`<`), above a maximum
//!   `≥ 0` fails (`>`). Transverse momenta are boost-invariant along the beam,
//!   so the frame does not matter.
//!
//! - **`pt_min_pdg`** (`banner.py:4700-4760`, `setcuts.f:318-334`): every
//!   outgoing leg whose `|PDG|` is listed gets `pT ≥ threshold` and no other
//!   single-leg cut, whatever its class, mass or flavour; only `cut_decays = F`
//!   exempts a decay product. Its `smin` term (`setcuts.f:678-690`) is not
//!   reproduced: the ŝ floor here is derived from the thresholds, see
//!   [`Cuts::shat_min`]. That term reads the wrong loop index and can raise
//!   MadEvent's `τ` floor past the cuts on some cards, which are refused.
//!
//! Cut families implemented here: ŝ window, single-leg pT/E/η (classes
//! j/b/a/l), the per-PDG pT minimum, pairwise ΔR and invariant mass, `ptll`,
//! `mmnl`, `htj` and `iht`, and the decay-chain Breit–Wigner windows. Every other
//! `cut=`-tagged parameter is parse-and-detect: [`Cuts::compile`] hard-errors
//! with [`CutError::UnimplementedCutActive`] if its value deviates from the MG
//! default, so an active but unimplemented cut is never silently ignored.

use thiserror::Error;

use crate::diagrams::diagram::{Diagram, LegIdx, OnShell, PropIdx, Ray};
use crate::helas::repr::lorentz::LorentzVector;
use crate::helas::repr::Real;
use crate::runcard::{decay_cut_reset, param_default, ParamValue, RunCard};
use crate::ufo::particles::ParticleId;
use crate::ufo::EvaluatedModel;

/// MadGraph's `small_width_treatment` default, the floor on every width as a
/// fraction of its mass. The run card cannot move it: a value off the default is
/// refused at parse.
pub(crate) const SMALL_WIDTH_TREATMENT: f64 = 1e-6;

/// One propagator a decay chain forces on shell: the final-state legs whose
/// momenta it carries, and the pole it is kept near.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ForcedLine {
    /// The outgoing legs the line carries, bit `k` naming the `k`-th final-state
    /// leg.
    pub(crate) slots: u64,
    pub(crate) mass: f64,
    pub(crate) width: f64,
}

impl ForcedLine {
    /// The width the window is measured in: MadGraph's `prwidth_tmp`, the width
    /// floored at `small_width_treatment` times the mass, or zero for a line
    /// without width.
    pub(crate) fn window_width(&self) -> f64 {
        if self.width > 0.0 {
            self.width.max(self.mass * SMALL_WIDTH_TREATMENT)
        } else {
            0.0
        }
    }

    /// The window `(M − bwcutoff·Γ, M + bwcutoff·Γ)` on the line's invariant
    /// mass (GeV), or `None` for a line of zero width, which is never cut.
    pub(crate) fn mass_window(&self, bwcutoff: f64) -> Option<(f64, f64)> {
        let width = self.window_width();
        (width > 0.0).then_some((self.mass - bwcutoff * width, self.mass + bwcutoff * width))
    }
}

/// The lines a subprocess's diagrams force on shell, one set per diagram.
///
/// A decay chain without identical particles across its decays forces the same
/// lines in every diagram, and this holds one set. With identical particles the
/// stitched diagrams are closed under their permutation, so different diagrams
/// force different leg sets (`e+ e- > z z, z > e+ e-` forces `{e+₁ e-₁}{e+₂ e-₂}`
/// in two diagrams and `{e+₁ e-₂}{e+₂ e-₁}` in the other two); each distinct set
/// is kept once. The collection is closed under the same permutations, so a
/// predicate reading it symmetrically — "every line of some set" — is symmetric
/// under exchange of identical particles, as the identical-particle factor
/// requires.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ForcedResonances {
    patterns: Vec<Vec<ForcedLine>>,
    /// The outgoing legs, bit `k` naming the `k`-th, that are massless and
    /// coloured and descend from no forced line in at least one diagram.
    massless_coloured_core: u64,
    /// The soft and collinear limits of massless legs that some diagram
    /// reaches, each once, sorted.
    massless_limits: Vec<MasslessLimit>,
}

impl ForcedResonances {
    /// No forced line: a process without decay chains.
    pub(crate) fn none() -> Self {
        ForcedResonances::default()
    }

    /// The forced lines of `diagrams`, which share one external-leg order.
    pub fn of(diagrams: &[Diagram], model: &EvaluatedModel) -> Self {
        let mut patterns: Vec<Vec<ForcedLine>> = Vec::new();
        let mut massless_coloured_core = 0u64;
        let mut massless_limits: Vec<MasslessLimit> = Vec::new();
        for diagram in diagrams {
            for limit in diagram_massless_limits(diagram, model) {
                if !massless_limits.contains(&limit) {
                    massless_limits.push(limit);
                }
            }
            let mut lines: Vec<ForcedLine> = forced_lines(diagram, model);
            let products = lines.iter().fold(0u64, |m, l| m | l.slots);
            massless_coloured_core |= massless_coloured_finals(diagram, model) & !products;
            if lines.is_empty() {
                continue;
            }
            lines.sort_by(|a, b| {
                a.slots
                    .cmp(&b.slots)
                    .then(a.mass.total_cmp(&b.mass))
                    .then(a.width.total_cmp(&b.width))
            });
            if !patterns.contains(&lines) {
                patterns.push(lines);
            }
        }
        patterns.sort_by(|a, b| {
            let key = |p: &Vec<ForcedLine>| p.iter().map(|l| l.slots).collect::<Vec<_>>();
            key(a).cmp(&key(b))
        });
        massless_limits.sort_unstable();
        ForcedResonances {
            patterns,
            massless_coloured_core,
            massless_limits,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// The distinct sets of forced lines, one per group of diagrams that force
    /// the same lines.
    pub(crate) fn patterns(&self) -> &[Vec<ForcedLine>] {
        &self.patterns
    }

    /// The outgoing legs that descend from a forced line in every diagram, bit
    /// `k` naming the `k`-th final-state leg: the legs `cut_decays = F` leaves
    /// uncut.
    pub(crate) fn decay_products(&self) -> u64 {
        self.patterns
            .iter()
            .map(|p| p.iter().fold(0u64, |m, l| m | l.slots))
            .reduce(|a, b| a & b)
            .unwrap_or(0)
    }
}

/// The outgoing legs of `diagram` that are massless and carry colour, bit `k`
/// naming the `k`-th final-state leg.
fn massless_coloured_finals(diagram: &Diagram, model: &EvaluatedModel) -> u64 {
    diagram.legs[diagram.n_in..]
        .iter()
        .enumerate()
        .filter(|(k, _)| *k < 64)
        .filter(|(_, leg)| {
            model.model().particle(leg.particle).color.abs() != 1 && model.mass(leg.particle) == 0.0
        })
        .fold(0u64, |m, (k, _)| m | (1u64 << k))
}

/// A soft or collinear limit of massless legs in which a propagator of a kept
/// diagram goes on shell. The integrand grows without bound there, and σ
/// diverges logarithmically unless a cut keeps every accepted point away from
/// the limit. Outgoing legs are named by position among the final-state legs
/// (the `k`-th is `k`), incoming legs by their index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MasslessLimit {
    /// Massless outgoing legs `a < b` meet at a vertex whose third line is a
    /// massless propagator: `q → q g`, `g → g g`, `g → q q̄`, `q → q γ`,
    /// `ℓ → ℓ γ`, `γ* → f f̄`. The propagator goes as
    /// `1/s_ab = 1/(2 pT_a pT_b (cosh Δy − cos Δφ))`, which vanishes as the two
    /// become collinear whatever their transverse momenta, so only a cut on the
    /// pair itself (a ΔR or invariant-mass minimum) regulates it.
    FinalCollinear { a: usize, b: usize },
    /// A massless outgoing leg meets massless incoming leg `beam` at a vertex
    /// whose third line is a massless spacelike propagator: `t → 0` as the leg
    /// becomes collinear with the beam. Its transverse-momentum minimum
    /// regulates it, and so does a rapidity maximum at fixed ŝ (a leg at bounded
    /// rapidity reaches `t → 0` only by becoming soft, which is integrable for a
    /// fermion and is [`MasslessLimit::Soft`] for a vector boson).
    InitialCollinear { beam: usize, leg: usize },
    /// A massless outgoing vector boson leaves an external line through a
    /// propagator of that line's mass, which goes as `1/(2p·k)` and diverges as
    /// the boson's energy vanishes, at any angle. Its transverse-momentum or
    /// energy minimum regulates it.
    Soft { leg: usize },
}

/// The [`MasslessLimit`]s `diagram` reaches.
///
/// Only three-point vertices with one propagator and two external legs are
/// read: a propagator between two vertices of that shape is the only line whose
/// invariant is a function of two external momenta alone. A splitting of the
/// whole final state of a 2 → 2 process is not a collinear limit (its
/// invariant is ŝ), and neither is a soft boson whose only partner in the final
/// state is massless (it drives ŝ to zero): both are limits of the ŝ floor.
fn diagram_massless_limits(diagram: &Diagram, model: &EvaluatedModel) -> Vec<MasslessLimit> {
    let n_in = diagram.n_in;
    let n_final = diagram.legs.len() - n_in;
    let particle = |i: usize| diagram.legs[i].particle;
    let massless = |id: ParticleId| model.mass(id) == 0.0;
    let is_vector = |id: ParticleId| model.model().particle(id).spin == 3;
    let mut limits = Vec::new();
    for vertex in &diagram.vertices {
        if vertex.rays.len() != 3 {
            continue;
        }
        let mut external: Vec<usize> = Vec::new();
        let mut props: Vec<ParticleId> = Vec::new();
        for ray in &vertex.rays {
            match *ray {
                Ray::Leg(LegIdx(i)) => external.push(i),
                Ray::Prop { prop, .. } => props.push(diagram.props[prop.0].particle),
            }
        }
        let (&[prop], &[x, y]) = (props.as_slice(), external.as_slice()) else {
            continue;
        };
        let all_massless = massless(prop) && massless(particle(x)) && massless(particle(y));
        match (x >= n_in, y >= n_in) {
            (true, true) if n_final >= 3 && all_massless => {
                let (a, b) = (x.min(y) - n_in, x.max(y) - n_in);
                limits.push(MasslessLimit::FinalCollinear { a, b });
            }
            (true, false) | (false, true) if all_massless => {
                let (beam, out) = if x < n_in { (x, y) } else { (y, x) };
                limits.push(MasslessLimit::InitialCollinear {
                    beam,
                    leg: out - n_in,
                });
            }
            _ => {}
        }
        for (k, emitter) in [(x, y), (y, x)] {
            if k < n_in || !massless(particle(k)) || !is_vector(particle(k)) {
                continue;
            }
            if model.mass(prop) != model.mass(particle(emitter)) {
                continue;
            }
            let lone_massless_partner = n_final == 2
                && (n_in..diagram.legs.len())
                    .filter(|&i| i != k)
                    .all(|i| massless(particle(i)));
            if !lone_massless_partner {
                limits.push(MasslessLimit::Soft { leg: k - n_in });
            }
        }
    }
    limits
}

/// Whether the compiled cuts keep every accepted point away from `limit`.
///
/// `infos` are the final-state legs in order, `single` and `pairs` the compiled
/// single-leg and pair cuts, `legs` every external leg. A leg the filter does
/// not cut at all (an uncut decay product, a leg in no class) regulates
/// nothing.
fn limit_regulated(
    limit: MasslessLimit,
    infos: &[LegInfo],
    single: &[SingleLegCut],
    pairs: &[PairCut],
    legs: &[ExternalLeg],
) -> bool {
    let single_of = |k: usize| single.iter().find(|s| s.idx == infos[k].idx);
    let pair_of = |i: usize, j: usize| {
        pairs
            .iter()
            .find(move |pc| (pc.i, pc.j) == (i, j) || (pc.i, pc.j) == (j, i))
    };
    match limit {
        MasslessLimit::FinalCollinear { a, b } => pair_of(infos[a].idx, infos[b].idx)
            .is_some_and(|pc| pc.dr2_min > 0.0 || pc.m2_min > 0.0),
        MasslessLimit::InitialCollinear { leg, .. } => {
            single_of(leg).is_some_and(|s| s.pt_min > 0.0 || s.eta_max >= 0.0)
        }
        MasslessLimit::Soft { leg } => {
            let idx = infos[leg].idx;
            single_of(leg).is_some_and(|s| s.pt_min > 0.0 || s.e_min > 0.0)
                || pairs
                    .iter()
                    .filter(|pc| pc.i == idx || pc.j == idx)
                    .any(|pc| {
                        let partner = if pc.i == idx { pc.j } else { pc.i };
                        pc.m2_min > legs[partner].mass * legs[partner].mass
                    })
        }
    }
}

/// Refuse a card whose compiled cuts leave one of `forced`'s
/// [`MasslessLimit`]s uncut.
///
/// The run would otherwise report a number set by how far its sampler happens
/// to reach into the logarithm, growing with the budget. MadGraph returns such a
/// number too, its own phase space's effective cutoff. Frixione photon
/// isolation (`cuts.f:1104-1222`), which would regulate a photon's collinear
/// limit with a parton, is refused before this check ([`detect_unimplemented`]),
/// and so is `ptheavy`; MLM's `xqcut` reaches the filter as the `ptj` and `mmjj`
/// it rewrites.
fn refuse_unregulated_limits(
    forced: &ForcedResonances,
    infos: &[LegInfo],
    single: &[SingleLegCut],
    pairs: &[PairCut],
    legs: &[ExternalLeg],
) -> Result<(), CutError> {
    match forced
        .massless_limits
        .iter()
        .find(|&&limit| !limit_regulated(limit, infos, single, pairs, legs))
    {
        Some(&limit) => Err(unregulated_limit_error(limit, infos, legs)),
        None => Ok(()),
    }
}

/// The refusal of an uncut `limit`, naming its legs and the run-card cuts that
/// would regulate it.
fn unregulated_limit_error(
    limit: MasslessLimit,
    infos: &[LegInfo],
    legs: &[ExternalLeg],
) -> CutError {
    let decay_note = |k: usize| {
        infos[k]
            .uncut_decay
            .then_some("cut_decays = T (cut_decays = F leaves this decay product uncut)")
    };
    let single_regulator = |k: usize, pt_or: &str| -> String {
        if let Some(note) = decay_note(k) {
            return note.to_string();
        }
        match infos[k].letter {
            Some(l) => {
                let c = letter_char(l);
                format!("pt{c} > 0 or {pt_or}{c}")
            }
            None => "no run-card cut, since the leg is in no cut class".to_string(),
        }
    };
    match limit {
        MasslessLimit::FinalCollinear { a, b } => {
            let (la, lb) = (infos[a].letter, infos[b].letter);
            let parton = |l: Option<Letter>| matches!(l, Some(Letter::Jet | Letter::B));
            if la == Some(Letter::Photon) && parton(lb) || lb == Some(Letter::Photon) && parton(la)
            {
                let (photon, parton) = if la == Some(Letter::Photon) {
                    (a, b)
                } else {
                    (b, a)
                };
                return CutError::UncutPhotonSplitting {
                    photon,
                    parton,
                    pdg: infos[parton].pdg,
                };
            }
            let regulator = match (decay_note(a).or(decay_note(b)), la.zip(lb)) {
                (Some(note), _) => note.to_string(),
                (None, Some((x, y))) => {
                    let tag = pair_tag(x, y).unwrap_or_default();
                    if x == y {
                        format!("dr{tag} > 0 or mm{tag} > 0")
                    } else {
                        format!("dr{tag} > 0")
                    }
                }
                (None, None) => "no run-card cut, since a leg is in no cut class".to_string(),
            };
            CutError::UncutCollinearSplitting {
                a,
                b,
                pdg_a: infos[a].pdg,
                pdg_b: infos[b].pdg,
                regulator,
            }
        }
        MasslessLimit::InitialCollinear { beam, leg } => CutError::UncutInitialCollinear {
            beam,
            beam_pdg: legs[beam].pdg,
            leg,
            pdg: infos[leg].pdg,
            regulator: single_regulator(leg, "a rapidity maximum eta"),
        },
        MasslessLimit::Soft { leg } => CutError::UncutSoftEmission {
            leg,
            pdg: infos[leg].pdg,
            regulator: single_regulator(leg, "e"),
        },
    }
}

/// Refuse a `cut_decays` exemption, `uncut` naming the exempt outgoing legs,
/// that leaves uncut a massless coloured leg which descends from no forced line
/// in some diagram of `forced`.
///
/// In that diagram the leg is a core parton, whose emission off a beam or
/// another parton is collinear-singular: the integrand grows as `1/pT²` on the
/// uncut region and σ diverges logarithmically. The channels would not show it,
/// because [`Cuts::spacelike_floor`] caps every spacelike transfer at the
/// hardest `pT` threshold, which bounds the accepted region only while every such
/// leg is cut; the run would report a finite, wrong σ. An exemption of legs that
/// are decay products in every diagram never trips this.
fn refuse_ir_unsafe_exemption(
    uncut: u64,
    forced: &ForcedResonances,
    legs: &[ExternalLeg],
) -> Result<(), CutError> {
    let unsafe_legs = uncut & forced.massless_coloured_core;
    if unsafe_legs == 0 {
        return Ok(());
    }
    let leg = unsafe_legs.trailing_zeros() as usize;
    let pdg = legs
        .iter()
        .filter(|l| l.is_final)
        .nth(leg)
        .map_or(0, |l| l.pdg);
    Err(CutError::IrUnsafeExemption { leg, pdg })
}

/// The forced lines of one diagram.
pub(crate) fn forced_lines(diagram: &Diagram, model: &EvaluatedModel) -> Vec<ForcedLine> {
    let n_in = diagram.n_in;
    diagram
        .props
        .iter()
        .enumerate()
        .filter(|(_, p)| p.onshell == OnShell::Forced)
        .filter_map(|(i, p)| {
            let side = diagram.final_state_side(PropIdx(i))?;
            let slots = side
                .iter()
                .filter(|l| l.0 >= n_in)
                .fold(0u64, |m, l| m | (1u64 << (l.0 - n_in)));
            Some(ForcedLine {
                slots,
                mass: model.mass(p.particle),
                width: model.width(p.particle),
            })
        })
        .collect()
}

/// One forced line as the compiled filter tests it.
#[derive(Clone, Debug, PartialEq)]
struct WindowCut {
    /// Indices into the momentum slice given to [`Cuts::pass`].
    members: Vec<usize>,
    mass: f64,
    /// `bwcutoff` times the line's window width.
    half_width: f64,
}

/// One external leg's identity, the classification input for [`Cuts::compile`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExternalLeg {
    /// PDG code (signed).
    pub(crate) pdg: i32,
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
    #[error("run card's pt_min_pdg = {value} is refused: {reason}")]
    PdgCutRefused { value: String, reason: String },
    #[error(
        "cut_decays = F would leave final-state leg {leg} (PDG {pdg}) uncut, but it is a \
         massless coloured core parton in a kept diagram: the exemption is IR-unsafe, its \
         collinear region diverges, and the channels' spacelike floor would cap it silently"
    )]
    IrUnsafeExemption { leg: usize, pdg: i32 },
    #[error(
        "final-state photon (outgoing leg {photon}) and massless coloured parton (outgoing leg \
         {parton}, PDG {pdg}) meet at a vertex of a kept diagram, and the run card leaves their \
         collinear limit uncut: no ΔR minimum between them (draj, or drab for a b-class parton) \
         and no photon isolation. The q → qγ splitting then diverges logarithmically and any \
         cross section reported is set by how far the sampler reaches into it; set draj > 0"
    )]
    UncutPhotonSplitting {
        photon: usize,
        parton: usize,
        pdg: i32,
    },
    #[error(
        "final-state legs {a} (PDG {pdg_a}) and {b} (PDG {pdg_b}) split off one massless \
         propagator in a kept diagram, and the run card leaves their collinear limit uncut: the \
         propagator diverges as the two become collinear whatever their transverse momenta, σ \
         diverges logarithmically, and any cross section reported is set by how far the sampler \
         reaches into it; regulate it with {regulator}"
    )]
    UncutCollinearSplitting {
        a: usize,
        b: usize,
        pdg_a: i32,
        pdg_b: i32,
        regulator: String,
    },
    #[error(
        "final-state leg {leg} (PDG {pdg}) leaves incoming leg {beam} (PDG {beam_pdg}) through a \
         massless spacelike propagator in a kept diagram, and the run card leaves its collinear \
         limit with the beam uncut: the propagator diverges as the leg becomes collinear with \
         the beam, σ diverges logarithmically, and any cross section reported is set by how far \
         the sampler reaches into it; regulate it with {regulator}"
    )]
    UncutInitialCollinear {
        beam: usize,
        beam_pdg: i32,
        leg: usize,
        pdg: i32,
        regulator: String,
    },
    #[error(
        "final-state massless vector boson (outgoing leg {leg}, PDG {pdg}) is emitted off an \
         external line in a kept diagram, and the run card leaves its soft limit uncut: the \
         emitting propagator diverges as the boson's energy vanishes, σ diverges \
         logarithmically, and any cross section reported is set by how far the sampler reaches \
         into it; regulate it with {regulator}"
    )]
    UncutSoftEmission {
        leg: usize,
        pdg: i32,
        regulator: String,
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
    /// Single-leg and pairwise dr/mass cuts apply (mass ≤ 20 GeV, not a neutrino,
    /// and not a decay product under `cut_decays = F`).
    do_cuts: bool,
    /// A decay product under `cut_decays = F`: the one exclusion the per-PDG
    /// cuts read (`setcuts.f:322`), where `do_cuts` also covers mass and
    /// neutrino flavour.
    uncut_decay: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SingleLegCut {
    idx: usize,
    pt_min: f64,
    pt_max: f64,
    e_min: f64,
    e_max: f64,
    eta_min: f64,
    eta_max: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
#[derive(Clone, Debug, PartialEq)]
struct MmnlCut {
    min: f64,
    max: f64,
    /// Indices of final-state leptons and neutrinos whose momenta are summed.
    members: Vec<usize>,
}

/// The scalar-sum transverse-momentum cuts (`cuts.f:874-913`): `htj`, the sum
/// over the jet-class legs, and the inclusive `iht`, that sum plus the b-class
/// legs. Membership is the leg's class alone, not its `do_cuts` flag, because
/// `cuts.f:524-534` fills the sums from `is_a_j` / `is_a_b` without reading it:
/// decay products under `cut_decays = F` and legs above 20 GeV count.
#[derive(Clone, Debug, PartialEq)]
struct HtCut {
    /// Indices of the jet-class final legs.
    jets: Vec<usize>,
    /// Indices of the b-class final legs.
    bjets: Vec<usize>,
    /// `htjmin`; a sum below it fails (`<`, so the threshold itself passes).
    htj_min: f64,
    /// `htjmax`; < 0 = inactive.
    htj_max: f64,
    /// `ihtmin`.
    iht_min: f64,
    /// `ihtmax`; < 0 = inactive.
    iht_max: f64,
}

/// A compiled, process-specific cut filter.
///
/// Equality is equality of the filter, not of the process it was compiled for:
/// every field is a leg *index* and a threshold, with the PDG codes and the
/// [`Letter`] classes already consumed by [`Cuts::compile`]. Two processes whose
/// legs classify the same way therefore compile to equal filters, which is what
/// lets a caller check that a set of subprocesses shares one cut indicator.
#[derive(Clone, Debug, PartialEq)]
pub struct Cuts {
    /// Indices of incoming legs (their momentum sum defines ŝ).
    incoming: Vec<usize>,
    /// Indices of final-state legs, in the order given to [`Cuts::compile`]. Bit
    /// `k` of a [`Cuts::timelike_floor`] subsystem names `finals[k]`.
    finals: Vec<usize>,
    shat_min_sq: f64,
    shat_max_sq: f64,
    single: Vec<SingleLegCut>,
    pairs: Vec<PairCut>,
    mmnl: Option<MmnlCut>,
    ht: Option<HtCut>,
    shat_min_hint: f64,
    /// An upper bound on ŝ (GeV²) the decay-chain windows imply, infinite where
    /// they imply none ([`Cuts::shat_max`]).
    shat_max_hint: f64,
    /// The decay-chain windows, one list per set of lines some diagram forces; a
    /// point passes when every window of one list holds. Empty without a decay
    /// chain.
    windows: Vec<Vec<WindowCut>>,
    /// `bwcutoff`, the window half-width in widths.
    bwcutoff: f64,
}

/// Cut parameters implemented by [`Cuts::compile`]; every other `cut=`-tagged
/// parameter is parse-and-detect.
const UNIMPLEMENTED_CUTS: &[&str] = &[
    "misset",
    "missetmax",
    "ptheavy",
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
    "pt_max_pdg",
    "E_min_pdg",
    "E_max_pdg",
    "eta_min_pdg",
    "eta_max_pdg",
    "mxx_min_pdg",
];

/// Requirements that read as off at any value at or below zero, so only a
/// positive value is refused: "at least one jet (photon, b, lepton) with
/// `pT > xpt*`". Every MadGraph read of them is guarded by `> 0` or folded
/// through `max(·, 0)`: the cut itself (`cuts.f:981`, `:994`, `:1007`,
/// `:1020`), the `smin` hint (`setcuts.f:540`, `:584`, `:591`, `:613`, `:622`,
/// `:645`, `:659`), the grouped-subprocess consistency report (`setcuts.f:752`,
/// `:760`, `:763`, `:772`) and the reset for a process without the class
/// (`setcuts.f:835-879`). `reweight.f`, `setscales.f` and `initcluster.f`
/// declare the common block and read none of the four.
const NONPOSITIVE_IS_OFF: &[&str] = &["xptj", "xptb", "xpta", "xptl"];

/// The PDG codes `banner.py:4722` means to refuse a per-PDG cut on: the light
/// quarks, the b, the charged leptons, the gluon and the photon, whose class
/// cuts the per-PDG block would replace. The pinned `banner.py` compares the
/// card's string keys against these integers, so the test never fires and
/// MadEvent applies the cut, replacing the class's pT, energy and rapidity
/// cuts with the per-PDG ones; refusing the card is what MadGraph intends and
/// keeps both sides out of that path.
const PDG_CUT_LIGHT: &[u32] = &[1, 2, 3, 4, 5, 11, 13, 15, 21, 22];

/// `run.inc`'s `pdg_cut(0:25)`: at most 25 distinct PDG codes
/// (`banner.py:4714`).
const PDG_CUT_MAX_ENTRIES: usize = 25;

/// MadGraph's per-PDG transverse-momentum minimum, `pt_min_pdg`, as
/// `banner.py:4700-4760` turns it into `pdg_cut` / `ptmin4pdg`: one threshold
/// per listed `|PDG|`, applied to particle and antiparticle alike.
#[derive(Clone, Debug, Default, PartialEq)]
struct PdgPtMin {
    entries: Vec<(u32, f64)>,
}

impl PdgPtMin {
    /// The card's `pt_min_pdg`, with MadGraph's refusals: a negative code
    /// (`banner.py:4717`), more than 25 codes (`:4714`), and the light codes
    /// MadGraph means to refuse ([`PDG_CUT_LIGHT`]). Code 0 is refused too: it
    /// names no particle, and listed first it would read as the empty list
    /// (`setcuts.f:318` tests `pdg_cut(1).ne.0`) and drop every other entry.
    fn from_card(rc: &RunCard) -> Result<Self, CutError> {
        let text = match rc.get("pt_min_pdg") {
            None => return Ok(Self::default()),
            Some(v) => v.as_str().to_string(),
        };
        let refuse = |reason: String| CutError::PdgCutRefused {
            value: format!("'{text}'"),
            reason,
        };
        let mut entries: Vec<(u32, f64)> = Vec::new();
        for (key, value) in parse_pdg_dict(&text).map_err(refuse)? {
            // `banner.py:4712` drops these two bookkeeping keys from the list.
            if key == "__type__" || key == "default" {
                continue;
            }
            let pdg: i64 = key
                .parse()
                .map_err(|_| refuse(format!("'{key}' is not a PDG code")))?;
            let pt: f64 = value
                .parse()
                .map_err(|_| refuse(format!("'{value}' is not a number")))?;
            if pdg < 0 {
                return Err(refuse(format!(
                    "PDG {pdg} is negative; MadGraph applies a per-PDG cut to particle and \
                     antiparticle alike and refuses a negative code (banner.py:4717)"
                )));
            }
            if pdg == 0 {
                return Err(refuse("PDG 0 names no particle".to_string()));
            }
            let pdg =
                u32::try_from(pdg).map_err(|_| refuse(format!("PDG {pdg} is out of range")))?;
            if PDG_CUT_LIGHT.contains(&pdg) {
                return Err(refuse(format!(
                    "PDG {pdg} is a light quark, b, charged lepton, gluon or photon, whose class \
                     cuts apply instead; banner.py:4722 means to refuse it, and its test never \
                     fires because it compares string keys with integers"
                )));
            }
            match entries.iter_mut().find(|(p, _)| *p == pdg) {
                Some(entry) => entry.1 = pt,
                None => entries.push((pdg, pt)),
            }
        }
        if entries.len() > PDG_CUT_MAX_ENTRIES {
            return Err(refuse(format!(
                "{} PDG codes are listed and MadGraph allows at most {PDG_CUT_MAX_ENTRIES} \
                 (banner.py:4714)",
                entries.len()
            )));
        }
        Ok(Self { entries })
    }

    /// Refuse the cards on which MadEvent's `smin` from the per-PDG cuts can
    /// exceed what the cuts allow and so remove phase space.
    ///
    /// `setcuts.f:678-690` counts the legs of the `i`-th listed code with
    /// `abs(idup(i,1,iproc)).eq.pdg_cut(i)` and `do_cuts(i)`, the list index
    /// where the leg index `j` is meant. The test then does not depend on `j`:
    /// it fails on every iteration, adding nothing, or passes on all
    /// `nexternal − nincoming + 1` of them, adding that many times
    /// `pmass(nexternal)² + ptmin²` to `smin`, which `myamp.f:569-570` makes the
    /// lower edge of the `τ` draw. Legs 1 and 2 are incoming partons, whose
    /// codes are refused here, so the overcount needs at least three listed
    /// codes and a listed outgoing leg with `do_cuts` set: at most 20 GeV, not
    /// a neutrino, not an uncut decay product. Which code is `i`-th is the
    /// iteration order of a Python set of strings (`banner.py:4726`), so
    /// whether a given run overcounts depends on its hash seed and cannot be
    /// reproduced; such a card is refused, whatever leg position it lists.
    fn refuse_smin_overcount(&self, infos: &[LegInfo]) -> Result<(), CutError> {
        if self.entries.len() < 3 {
            return Ok(());
        }
        let Some(leg) = infos
            .iter()
            .find(|i| i.do_cuts && self.threshold(i.pdg).is_some())
        else {
            return Ok(());
        };
        Err(CutError::PdgCutRefused {
            value: format!("{:?}", self.entries),
            reason: format!(
                "it lists {} codes and the outgoing PDG {} at or below 20 GeV, which \
                 setcuts.f:681 counts once per leg of the process when the code's place in \
                 MadGraph's hash-ordered pdg_cut list matches the leg's: its smin then \
                 exceeds what the cut allows and removes phase space, in a run-dependent way",
                self.entries.len(),
                leg.pdg
            ),
        })
    }

    /// The threshold for an outgoing leg of code `pdg`, if its `|PDG|` is listed.
    fn threshold(&self, pdg: i32) -> Option<f64> {
        let a = pdg.unsigned_abs();
        self.entries
            .iter()
            .find(|(p, _)| *p == a)
            .map(|(_, pt)| *pt)
    }
}

/// The `(key, value)` pairs of a dict-valued run-card entry, read as
/// `banner.py:1220-1251` reads one: `{k1: v1, k2: v2}` (a trailing comma ends
/// the list, quotes around a key are dropped, later keys overwrite earlier
/// ones) or a single `k : v`, `k, v` or `k v`. The empty string is the empty
/// dict.
fn parse_pdg_dict(text: &str) -> Result<Vec<(String, String)>, String> {
    let text = text.trim();
    let unquote = |x: &str| -> String {
        let x = x.trim();
        let quoted =
            x.len() >= 2 && (x.starts_with('"') || x.starts_with('\'')) && x.ends_with(&x[..1]);
        if quoted {
            x[1..x.len() - 1].to_string()
        } else {
            x.to_string()
        }
    };
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let malformed = || format!("'{text}' is not a dict of PDG codes to thresholds");
    if let Some(body) = text.strip_prefix('{').and_then(|b| b.strip_suffix('}')) {
        let mut pairs = Vec::new();
        for pair in body.split(',') {
            if pair.trim().is_empty() {
                break;
            }
            let (k, v) = pair.rsplit_once(':').ok_or_else(malformed)?;
            pairs.push((unquote(k), v.trim().to_string()));
        }
        return Ok(pairs);
    }
    let (k, v) = if text.contains(',') {
        text.split_once(',').ok_or_else(malformed)?
    } else if text.contains(':') {
        text.split_once(':').ok_or_else(malformed)?
    } else {
        text.split_once(char::is_whitespace).ok_or_else(malformed)?
    };
    if v.contains([',', ':']) {
        return Err(malformed());
    }
    Ok(vec![(unquote(k), v.trim().to_string())])
}

/// signed square used by MadGraph for mass/ptll thresholds: `x·|x|`.
fn signed_sq(x: f64) -> f64 {
    x * x.abs()
}

/// Upper bound on the azimuthal separation `DELTA_PHI`'s cosine clamp can invent:
/// `acos(0.99999999)`, rounded up. A pair at zero true separation is reported this
/// far apart, so a `ΔR` threshold accepts pairs whose true separation is smaller
/// than it by up to this much; every derived separation bound is taken at the
/// correspondingly relaxed radius.
const DELTA_PHI_CLAMP_SLACK: f64 = 1.5e-4;

/// `min(cosh Δy − cos Δφ)` over the separations a `ΔR ≥ R` threshold accepts,
/// given as the stored signed square `R·|R|`. The minimisation is derived in
/// [`Cuts::timelike_floor`].
fn min_separation_gap(dr2_min: f64) -> f64 {
    let r2 = (dr2_min - DELTA_PHI_CLAMP_SLACK * DELTA_PHI_CLAMP_SLACK).max(0.0);
    let r = r2.sqrt();
    if r <= std::f64::consts::PI {
        1.0 - r.cos()
    } else {
        let dy2 = r2 - std::f64::consts::PI * std::f64::consts::PI;
        dy2.sqrt().cosh() + 1.0
    }
}

/// The decay-chain window cuts, one list per pattern of `forced` lines: each line
/// of non-zero window width becomes a cut over the final legs (indexed by
/// `finals`) it decays to, `bwcutoff` widths either side of its pole.
fn decay_windows(
    forced: &ForcedResonances,
    finals: &[usize],
    bwcutoff: f64,
) -> Vec<Vec<WindowCut>> {
    forced
        .patterns()
        .iter()
        .map(|lines| {
            lines
                .iter()
                .filter_map(|line| {
                    let width = line.window_width();
                    (width > 0.0).then(|| WindowCut {
                        members: finals
                            .iter()
                            .enumerate()
                            .filter(|(k, _)| *k < 64 && line.slots & (1u64 << k) != 0)
                            .map(|(_, &idx)| idx)
                            .collect(),
                        mass: line.mass,
                        half_width: bwcutoff * width,
                    })
                })
                .collect()
        })
        .collect()
}

/// Single-leg pT / E / η cuts: the j/b/a/l class cuts on every final leg whose
/// `do_cuts` is set, and the per-PDG cut on every final leg whose `|PDG|` it
/// lists.
///
/// The per-PDG block (`setcuts.f:318-334`) runs after the class block and
/// overwrites all six of a listed leg's thresholds, the ones `pt_min_pdg` does
/// not set at their inactive values. It reads `do_cuts` only through
/// `cut_decays`: a leg above 20 GeV or a neutrino is cut, a decay product under
/// `cut_decays = F` is not.
fn single_leg_cuts(rc: &RunCard, infos: &[LegInfo], pdg: &PdgPtMin) -> Vec<SingleLegCut> {
    let mut single = Vec::new();
    for info in infos {
        if !info.uncut_decay {
            if let Some(pt_min) = pdg.threshold(info.pdg) {
                single.push(SingleLegCut {
                    idx: info.idx,
                    pt_min,
                    pt_max: -1.0,
                    e_min: 0.0,
                    e_max: -1.0,
                    eta_min: 0.0,
                    eta_max: -1.0,
                });
                continue;
            }
        }
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
    single
}

/// Pairwise ΔR / invariant-mass / ptll cuts over every unordered pair of final
/// legs, keeping the pairs with at least one active threshold.
fn pair_cuts(rc: &RunCard, infos: &[LegInfo]) -> Vec<PairCut> {
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
    pairs
}

/// The `mmnl` cut on the invariant mass of the summed lepton + neutrino system,
/// or `None` when it is inactive or the process has no such leg.
fn mmnl_cut(rc: &RunCard, infos: &[LegInfo]) -> Option<MmnlCut> {
    let mmnl = rc.float("mmnl");
    let mmnlmax = rc.float("mmnlmax");
    if !(mmnl > 0.0 || mmnlmax >= 0.0) {
        return None;
    }
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
}

/// The HT cuts, or `None` when every threshold is at a value no event fails:
/// `htjmin` and `ihtmin` at or below zero (a sum of transverse momenta is never
/// negative) and `htjmax` and `ihtmax` negative (`cuts.f` reads a maximum only
/// when it is `≥ 0`). The MadGraph defaults `0, -1, 0, -1` are that case.
fn ht_cut(rc: &RunCard, infos: &[LegInfo]) -> Option<HtCut> {
    let htj_min = rc.float("htjmin");
    let htj_max = rc.float("htjmax");
    let iht_min = rc.float("ihtmin");
    let iht_max = rc.float("ihtmax");
    if !(htj_min > 0.0 || htj_max >= 0.0 || iht_min > 0.0 || iht_max >= 0.0) {
        return None;
    }
    let of = |letter: Letter| -> Vec<usize> {
        infos
            .iter()
            .filter(|i| i.letter == Some(letter))
            .map(|i| i.idx)
            .collect()
    };
    Some(HtCut {
        jets: of(Letter::Jet),
        bjets: of(Letter::B),
        htj_min,
        htj_max,
        iht_min,
        iht_max,
    })
}

impl Cuts {
    /// Compile the run card's cuts for a specific external-leg assignment of a
    /// process without decay chains.
    pub fn compile(rc: &RunCard, legs: &[ExternalLeg]) -> Result<Cuts, CutError> {
        Self::compile_with(rc, legs, &ForcedResonances::none())
    }

    /// Compile the run card's cuts for an external-leg assignment whose diagrams
    /// force the lines of `forced` on shell: [`Cuts::compile`] plus the
    /// decay-chain windows, and `cut_decays` read against the decay products.
    pub fn compile_with(
        rc: &RunCard,
        legs: &[ExternalLeg],
        forced: &ForcedResonances,
    ) -> Result<Cuts, CutError> {
        detect_unimplemented(rc)?;
        let pdg_pt_min = PdgPtMin::from_card(rc)?;
        let uncut_products = if cut_decays(rc) {
            0
        } else {
            forced.decay_products()
        };
        refuse_ir_unsafe_exemption(uncut_products, forced, legs)?;

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
            .enumerate()
            .map(|(k, (idx, l))| {
                let mut info = classify(idx, l, maxjetflavor);
                if k < 64 && uncut_products & (1u64 << k) != 0 {
                    info.do_cuts = false;
                    info.uncut_decay = true;
                }
                info
            })
            .collect();
        pdg_pt_min.refuse_smin_overcount(&infos)?;
        let finals: Vec<usize> = infos.iter().map(|i| i.idx).collect();
        let bwcutoff = rc.float("bwcutoff");
        let windows = decay_windows(forced, &finals, bwcutoff);

        let single = single_leg_cuts(rc, &infos, &pdg_pt_min);
        let pairs = pair_cuts(rc, &infos);
        refuse_unregulated_limits(forced, &infos, &single, &pairs, legs)?;
        let mmnl_cut = mmnl_cut(rc, &infos);
        let ht = ht_cut(rc, &infos);

        let dsqrt_shat = rc.float("dsqrt_shat");
        let dsqrt_shatmax = rc.float("dsqrt_shatmax");
        let two_incoming = incoming.len() == 2;
        let shat_min_hint = shat_min_hint(rc, legs, &infos, &single)
            .max(ht_shat_floor(ht.as_ref()))
            .max(forced_shat_floor(legs, forced, bwcutoff));
        let shat_max_hint = forced_shat_ceiling(finals.len(), forced, bwcutoff);

        tracing::debug!(
            single_leg = single.len(),
            pairs = pairs.len(),
            mmnl = mmnl_cut.is_some(),
            ht = ht.is_some(),
            sqrt_shat_min = dsqrt_shat,
            sqrt_shat_max = dsqrt_shatmax,
            "compiled cuts"
        );

        Ok(Cuts {
            incoming,
            finals,
            shat_min_sq: if two_incoming {
                dsqrt_shat * dsqrt_shat
            } else {
                0.0
            },
            shat_max_sq: if dsqrt_shatmax == -1.0 || !two_incoming {
                -1.0
            } else {
                dsqrt_shatmax * dsqrt_shatmax
            },
            single,
            pairs,
            mmnl: mmnl_cut,
            ht,
            shat_min_hint,
            shat_max_hint,
            windows,
            bwcutoff,
        })
    }

    /// `bwcutoff`: how many widths either side of its pole a forced line's mass
    /// may lie.
    pub(crate) fn bwcutoff(&self) -> f64 {
        self.bwcutoff
    }

    /// Whether any line of the process is forced on shell.
    #[cfg(test)]
    pub(crate) fn has_windows(&self) -> bool {
        !self.windows.is_empty()
    }

    /// A conservative lower bound on ŝ implied by the active cuts, for a
    /// hadronic x-integration mapping (`x_min = shat_min / s`). Never exceeds the
    /// true ŝ of a surviving point.
    pub fn shat_min(&self) -> f64 {
        self.shat_min_hint
    }

    /// A conservative upper bound on ŝ (GeV²) implied by the decay-chain windows,
    /// for the upper end of a hadronic `τ` map: a forced line carrying the whole
    /// final state holds `√ŝ` inside its window, so no surviving point lies above
    /// it. Infinite where no window bounds the whole final state. Never below the
    /// true ŝ of a surviving point.
    pub(crate) fn shat_max(&self) -> f64 {
        self.shat_max_hint
    }

    /// A regulator scale (GeV²) for the spacelike propagator pole of a peripheral
    /// phase-space channel: the square of the largest single-leg transverse-momentum
    /// threshold the active cuts impose on a final-state leg, or `0` when no such
    /// threshold is active.
    ///
    /// A peripheral emission off a massless beam that puts a massless system at
    /// transverse momentum `pT` transfers `|t| = 2E_beam(m² + pT²)/(E + p_z) ≥ pT²`,
    /// so a leg held above `pT_min` holds the transfer of the rung that produced it
    /// above `pT_min²`. Transverse balance carries the same scale to the *other*
    /// side of the rung whenever the recoil is the only thing balancing that leg,
    /// which for a three-body final state is always: the system opposite the jet
    /// carries the jet's own `pT`. Past three outgoing legs a partition can balance
    /// internally, so this stops being a bound and stays a scale.
    ///
    /// The floor does two things in a channel built by
    /// [`DiagramChannel::from_diagram_regulated`]. As a *density regulator* it
    /// sets each rung's propagator pole, entering the `t` draw and the `t` measure
    /// alike, so its size moves only the efficiency — and the well-posedness of a
    /// draw whose pole would otherwise sit on the transfer's own rounding noise.
    /// As a *support bound* it also caps each rung's transfer at `t ≤ −floor`,
    /// above which the channel's density is exactly zero. The estimator is then
    /// unbiased only while the channel set as a whole covers every point the cuts
    /// accept: past three outgoing legs, where the floor is a scale rather than a
    /// bound, that is what the coverage gates check, not something the floor
    /// guarantees. A run with no active single-leg `pT` cut gets `0`: its
    /// peripheral channels are still built, unbounded, with the pole floored at
    /// the channel's own default scale rather than at one read off other cuts.
    ///
    /// [`DiagramChannel::from_diagram_regulated`]:
    ///     crate::phasespace::diagram_channel::DiagramChannel::from_diagram_regulated
    pub fn spacelike_floor(&self) -> f64 {
        let pt_min = self
            .single
            .iter()
            .map(|c| c.pt_min)
            .fold(0.0f64, f64::max)
            .max(0.0);
        pt_min * pt_min
    }

    /// A lower bound (GeV²) on the invariant mass² of the final-state subsystem
    /// `slots` — bit `k` naming the `k`-th final-state leg in [`Cuts::compile`]'s
    /// leg order — holding at every configuration [`Cuts::pass`] accepts. Zero when
    /// the active cuts imply none.
    ///
    /// This is the timelike counterpart of [`spacelike_floor`](Self::spacelike_floor),
    /// and unlike it, it is a *kinematic* statement rather than a regulator scale: a
    /// channel that draws a subsystem's invariant from this floor upward loses only
    /// configurations the cuts reject, so the estimator stays unbiased — but only
    /// while the bound is provable, which is what the derivation below is for. An
    /// over-tight bound cuts into the accepted region and biases σ silently.
    ///
    /// ## Adding legs cannot lower the bound
    ///
    /// Every bound below is read off a *sub*-multiset `T ⊆ S` of the subsystem's
    /// legs, and `m²(S) ≥ m²(T)`: writing `A = Σ_{T} p`, `B = Σ_{S∖T} p`,
    /// `(A+B)² = A² + B² + 2A·B` with `B² ≥ 0` and
    /// `A·B = E_A E_B − p⃗_A·p⃗_B ≥ E_A E_B − |p⃗_A||p⃗_B| ≥ 0`, since a sum of
    /// on-shell final-state momenta is future-pointing with `E ≥ |p⃗|`. So a bound
    /// on a pair inside `S` is a bound on `S`.
    ///
    /// ## The pair bounds
    ///
    /// An explicit pairwise invariant-mass threshold (`mmll` and friends) is the
    /// bound itself, taken only from a *normal* window — an inverted one
    /// (`m2_max < m2_min`, a veto band) admits masses below `m2_min` and implies
    /// nothing.
    ///
    /// The transverse-momentum and separation thresholds imply one too. With
    /// rapidity `y` (what `cuts.f` cuts on) and `m_T = √(m² + p_T²)`,
    ///
    /// ```text
    ///   (p_i + p_j)² = m_i² + m_j² + 2(m_Ti m_Tj cosh Δy − p_Ti p_Tj cos Δφ)
    ///               ≥ 2 p_Ti p_Tj (cosh Δy − cos Δφ),
    /// ```
    ///
    /// using `m_T ≥ p_T ≥ 0` and `cosh ≥ 0`. The accepted region has
    /// `p_Ti ≥ p_Ti^min`, `p_Tj ≥ p_Tj^min` and `Δy² + Δφ² ≥ R²`, so with both
    /// thresholds positive the bound is `2 p_Ti^min p_Tj^min · g_min(R)`, where
    /// `g(Δy, Δφ) = cosh Δy − cos Δφ` is minimised over the accepted separations.
    ///
    /// `g` increases in `|Δy|` and in `Δφ` on `[0, π]`, so shrinking any point of
    /// the region radially toward the origin lowers `g` until it reaches the circle
    /// `Δy² + Δφ² = R²`: the minimum sits there. On the circle, with `Δy = a` and
    /// `Δφ = b = √(R² − a²)`,
    /// `d/da [cosh a − cos b] = sinh a − a·sin(b)/b ≥ 0` for `a ≥ 0`, since
    /// `sinh a ≥ a` and `sin b ≤ b`. So the minimum is at `Δy = 0, Δφ = R` and
    /// `g_min = 1 − cos R` — for `R ≤ π`. A larger `R` puts that point outside the
    /// strip `Δφ ≤ π` the opening angle lives in, and the minimum moves to the
    /// strip's corner `Δφ = π, |Δy| = √(R² − π²)`, giving `cosh√(R² − π²) + 1`.
    ///
    /// `R` is taken a hair below the nominal threshold ([`DELTA_PHI_CLAMP_SLACK`]),
    /// because `DELTA_PHI` clamps its cosine and so *reports* a pair at zero
    /// azimuthal separation as being that far apart: a point whose true separation
    /// is `√(R² − slack²)` can be accepted.
    ///
    /// The bound needs both legs held above a positive `p_T`, which also disposes of
    /// the rapidity sentinel: `E ≤ |p_z|` — which `rapidity` reports as `−1e99` and
    /// which would let a pair pass `ΔR` at an arbitrary true separation — needs
    /// `m² ≤ −p_T²`, impossible for an on-shell leg carrying `p_T ≥ p_T^min > 0`.
    ///
    /// ## `mmnl`
    ///
    /// The combined lepton-plus-neutrino mass threshold bounds its own member set,
    /// hence any subsystem containing it.
    pub fn timelike_floor(&self, slots: u64) -> f64 {
        let holds = |idx: usize| -> bool {
            self.finals
                .iter()
                .position(|&f| f == idx)
                .is_some_and(|k| k < 64 && slots & (1u64 << k) != 0)
        };
        let mut floor = 0.0f64;
        for pc in &self.pairs {
            if holds(pc.i) && holds(pc.j) {
                floor = floor.max(self.pair_mass2_floor(pc));
            }
        }
        if let Some(m) = &self.mmnl {
            if m.min > 0.0 && !m.members.is_empty() && m.members.iter().all(|&i| holds(i)) {
                floor = floor.max(m.min * m.min);
            }
        }
        floor
    }

    /// A lower bound (GeV) on the energy, in any frame reached from the lab by a
    /// boost along the beam axis, of the final-state subsystem `slots` (bit `k`
    /// naming the `k`-th final-state leg in [`Cuts::compile`]'s leg order), holding
    /// at every configuration [`Cuts::pass`] accepts. Zero when the active cuts
    /// imply none.
    ///
    /// Each leg's energy is at least its transverse momentum in every such frame,
    /// `E² = p_T² + p_z² + m²` with `p_T` invariant under the boost, and at least
    /// the `emin` threshold in the lab; a subsystem's energy is the sum of its legs'.
    /// The bound is what regulates a soft-shaped angular draw at the cuts
    /// ([`DiagramChannel::with_split_angles`]), so that a map built for a
    /// `1/E` rise does not spend its draws below the threshold that rejects them.
    ///
    /// [`DiagramChannel::with_split_angles`]:
    ///     crate::phasespace::diagram_channel::DiagramChannel::with_split_angles
    pub(crate) fn energy_floor(&self, slots: u64) -> f64 {
        self.finals
            .iter()
            .enumerate()
            .filter(|(k, _)| *k < 64 && slots & (1u64 << k) != 0)
            .map(|(_, &idx)| {
                self.single
                    .iter()
                    .find(|c| c.idx == idx)
                    .map_or(0.0, |c| c.pt_min.max(c.e_min).max(0.0))
            })
            .sum()
    }

    /// The lower bound on one pair's invariant mass², from its own thresholds.
    /// See [`timelike_floor`](Self::timelike_floor) for the derivation.
    fn pair_mass2_floor(&self, pc: &PairCut) -> f64 {
        let mut floor = 0.0f64;
        // A normal window's lower edge is the bound; an inverted one (a veto band)
        // admits everything below `m2_min` and implies nothing.
        if pc.m2_min > 0.0 && (pc.m2_max < 0.0 || pc.m2_min <= pc.m2_max) {
            floor = floor.max(pc.m2_min);
        }
        let (pt_i, pt_j) = (self.pt_min_of(pc.i), self.pt_min_of(pc.j));
        if pc.dr2_min > 0.0 && pt_i > 0.0 && pt_j > 0.0 {
            floor = floor.max(2.0 * pt_i * pt_j * min_separation_gap(pc.dr2_min));
        }
        floor
    }

    /// The transverse-momentum threshold the compiled single-leg cuts hold leg
    /// `idx` above, or `0` when it has none.
    fn pt_min_of(&self, idx: usize) -> f64 {
        self.single
            .iter()
            .find(|c| c.idx == idx)
            .map_or(0.0, |c| c.pt_min.max(0.0))
    }

    /// Whether the filter rejects every configuration in which the outgoing legs
    /// carry no transverse momentum: some outgoing leg has a positive `pT`
    /// minimum, which [`Cuts::pass`] fails at `pT = 0` (`pt < pt_min`, as
    /// `cuts.f` compares it).
    ///
    /// A `2 → 1` subprocess at collinear beams puts its single outgoing particle
    /// at exactly `pT = 0`, so for one this is the statement that the cuts zero
    /// it outright. A sufficient condition read off the thresholds: `false` does
    /// not mean such a configuration passes.
    pub fn rejects_zero_transverse_momentum(&self) -> bool {
        self.single.iter().any(|s| s.pt_min > 0.0)
    }

    /// Whether the compiled filter admits every point of every phase space: no ŝ
    /// window, no decay-chain window, no lepton-plus-neutrino mass window, no HT
    /// threshold, no pairwise threshold, and every single-leg threshold at a value no momentum
    /// fails.
    ///
    /// A sufficient condition, read off the thresholds and not off any sample: a
    /// leg's transverse momentum and absolute rapidity are non-negative and its
    /// energy is positive wherever its momentum is on a mass shell with a massive
    /// parent, so a lower bound at or below zero and an absent upper bound reject
    /// nothing. A filter for which this is `false` may still admit everything.
    pub(crate) fn admits_every_point(&self) -> bool {
        // Both structs are destructured in full, so a threshold added to either
        // does not compile until this method has decided whether it can reject.
        let Cuts {
            incoming: _,
            finals: _,
            shat_min_sq,
            shat_max_sq,
            single,
            pairs,
            mmnl,
            ht,
            // A sampling aid derived from the thresholds above; it rejects nothing.
            shat_min_hint: _,
            shat_max_hint: _,
            windows,
            // The width of the decay-chain windows, which `windows` already covers.
            bwcutoff: _,
        } = self;
        *shat_min_sq <= 0.0
            && *shat_max_sq < 0.0
            && pairs.is_empty()
            && mmnl.is_none()
            && ht.is_none()
            && windows.is_empty()
            && single.iter().all(|s| {
                let SingleLegCut {
                    idx: _,
                    pt_min,
                    pt_max,
                    e_min,
                    e_max,
                    eta_min,
                    eta_max,
                } = s;
                *pt_min <= 0.0
                    && *pt_max < 0.0
                    && *e_min <= 0.0
                    && *e_max < 0.0
                    && *eta_min <= 0.0
                    && *eta_max < 0.0
            })
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

        if !self.windows.is_empty() && !self.windows.iter().any(|w| inside_windows(w, momenta)) {
            return false;
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

        if let Some(h) = &self.ht {
            if !ht_ok(h, momenta) {
                return false;
            }
        }

        true
    }
}

/// The HT cuts on one point (`cuts.f:874-913`). With no jet-class leg `htj` is
/// zero, which reproduces `cuts.f`'s separate branch: `htjmin > 0` rejects
/// (`cuts.f:817`) and the `htj` maximum is never read (`:842`). The inclusive sum
/// is checked whether or not a jet is present (`:908`).
fn ht_ok<F: Real>(h: &HtCut, momenta: &[LorentzVector<F>]) -> bool {
    let sum = |legs: &[usize]| {
        legs.iter()
            .fold(F::zero(), |acc, &i| acc + transverse(momenta[i]))
    };
    let htj = sum(&h.jets);
    if htj < c::<F>(h.htj_min) || (h.htj_max >= 0.0 && htj > c::<F>(h.htj_max)) {
        return false;
    }
    let iht = htj + sum(&h.bjets);
    !(iht < c::<F>(h.iht_min) || (h.iht_max >= 0.0 && iht > c::<F>(h.iht_max)))
}

/// A lower bound on ŝ from the HT minima. In the partonic centre of mass
/// `√ŝ = Σ E ≥ Σ p_T` over any subset of the final legs, since `E ≥ p_T` for each
/// and `p_T` is invariant under the boost along the beam; an accepted point has
/// `iht ≥ htj ≥ htjmin` and `iht ≥ ihtmin`. A minimum whose sum has no leg to run
/// over rejects every point and is left out, so the bound never exceeds what an
/// empty sum would let through. Zero without HT cuts.
fn ht_shat_floor(ht: Option<&HtCut>) -> f64 {
    let Some(h) = ht else { return 0.0 };
    let mut floor = 0.0f64;
    if !h.jets.is_empty() {
        floor = floor.max(h.htj_min);
    }
    if !h.jets.is_empty() || !h.bjets.is_empty() {
        floor = floor.max(h.iht_min);
    }
    floor * floor
}

/// Whether every forced line of one set sits inside its window,
/// `|√(p²) − M| < bwcutoff·Γ` (`myamp.f:179`).
fn inside_windows<F: Real>(windows: &[WindowCut], momenta: &[LorentzVector<F>]) -> bool {
    windows.iter().all(|w| {
        let mut sum = LorentzVector::<F>::new(F::zero(), F::zero(), F::zero(), F::zero());
        for &i in &w.members {
            sum = sum + momenta[i];
        }
        let xmass = sum.m2().max(F::zero()).sqrt();
        (xmass - c::<F>(w.mass)).abs() < c::<F>(w.half_width)
    })
}

/// `cut_decays`: whether decay products receive the run card's cuts.
fn cut_decays(rc: &RunCard) -> bool {
    matches!(rc.get("cut_decays"), Some(ParamValue::Bool(true)))
}

/// A lower bound on ŝ from the decay-chain windows: in the partonic centre of
/// mass `√ŝ = Σ E ≥ Σ m` over any partition of the final state into systems, so
/// the outermost forced lines of a set, each at least `M − bwcutoff·Γ`, and the
/// remaining legs at their pole masses bound it. A point passes on some set, so
/// the bound is the least over the sets. Zero without forced lines.
fn forced_shat_floor(legs: &[ExternalLeg], forced: &ForcedResonances, bwcutoff: f64) -> f64 {
    if forced.is_empty() {
        return 0.0;
    }
    let finals: Vec<f64> = legs.iter().filter(|l| l.is_final).map(|l| l.mass).collect();
    forced
        .patterns()
        .iter()
        .map(|lines| {
            let mut covered = 0u64;
            let mut sum = 0.0f64;
            // Widest first, so a line nested inside a cut one is skipped.
            let mut outer: Vec<&ForcedLine> = lines.iter().collect();
            outer.sort_by_key(|l| std::cmp::Reverse(l.slots.count_ones()));
            for line in outer {
                if line.slots & covered != 0 {
                    continue;
                }
                let own: f64 = (0..finals.len())
                    .filter(|&k| k < 64 && line.slots & (1u64 << k) != 0)
                    .map(|k| finals[k].max(0.0))
                    .sum();
                let window = line
                    .mass_window(bwcutoff)
                    .map_or(0.0, |(lo, _)| lo.max(0.0));
                sum += own.max(window);
                covered |= line.slots;
            }
            sum += (0..finals.len())
                .filter(|&k| k >= 64 || covered & (1u64 << k) == 0)
                .map(|k| finals[k].max(0.0))
                .sum::<f64>();
            sum * sum
        })
        .fold(f64::INFINITY, f64::min)
}

/// An upper bound on ŝ from the decay-chain windows: a set whose forced lines
/// include one carrying all `n_final` final-state legs holds `√ŝ` below that
/// line's `M + bwcutoff·Γ`; a set without one leaves ŝ unbounded. A point passes
/// on some set, so the bound is the greatest over the sets. Infinite without
/// forced lines.
fn forced_shat_ceiling(n_final: usize, forced: &ForcedResonances, bwcutoff: f64) -> f64 {
    if forced.is_empty() || n_final == 0 || n_final > 64 {
        return f64::INFINITY;
    }
    let all = if n_final == 64 {
        u64::MAX
    } else {
        (1u64 << n_final) - 1
    };
    forced
        .patterns()
        .iter()
        .map(|lines| {
            lines
                .iter()
                .filter(|l| l.slots == all)
                .filter_map(|l| l.mass_window(bwcutoff))
                .map(|(_, hi)| hi * hi)
                .fold(f64::INFINITY, f64::min)
        })
        .fold(0.0f64, f64::max)
}

/// Upper bounds on one object's (or the missing system's) transverse momentum:
/// `missetmax` bounds the transverse momentum of the neutrino momentum sum,
/// `ptj<n>max` and `ptl<n>max` that of the `n`-th hardest jet or lepton.
///
/// Such a momentum cannot reach half the partonic centre-of-mass energy: in the
/// partonic rest frame the object's momentum balances the rest of the final
/// state, so `|p| <= E_object` and `|p| <= E_rest` give `|p| <= sqrt(s_hat)/2`,
/// and the transverse momentum is boost-invariant along the beam. Head-on beams
/// collide at `sqrt(s_hat) <= ebeam1 + ebeam2` whatever their masses (`2 sqrt(ebeam1
/// ebeam2)` for massless proton beams, which is no larger), so a bound at or above
/// `(ebeam1 + ebeam2) / 2` cannot exclude any point and reads as the default.
const SINGLE_PT_MAXIMA: &[&str] = &[
    "missetmax",
    "ptj1max",
    "ptj2max",
    "ptj3max",
    "ptj4max",
    "ptl1max",
    "ptl2max",
    "ptl3max",
    "ptl4max",
];

/// Refuse a run card that activates a cut this crate parses but does not apply,
/// or a `pt_min_pdg` MadGraph refuses. Reads the run card alone, so callers run
/// it before any process enumeration. A single-object transverse-momentum
/// maximum at or above the collider's ceiling reads as its default, and so does
/// an `xpt*` requirement at or below zero.
pub fn detect_unimplemented(rc: &RunCard) -> Result<(), CutError> {
    PdgPtMin::from_card(rc)?;
    let pt_ceiling = 0.5 * (rc.ebeam1 + rc.ebeam2);
    for &name in UNIMPLEMENTED_CUTS {
        let (Some(cur), Some(def)) = (rc.get(name), param_default(name)) else {
            continue;
        };
        // `remove_all_cut`'s reset value is off too: `ktdurham = 0`, `ptlund = 0`
        // and `deltaeta = -1` fail the `> 0` guards `cuts.f` reads them through,
        // exactly as their LO defaults do.
        if *cur == def || Some(cur) == decay_cut_reset(name).as_ref() {
            continue;
        }
        if SINGLE_PT_MAXIMA.contains(&name)
            && matches!(cur, ParamValue::Float(x) if *x >= pt_ceiling)
        {
            continue;
        }
        if NONPOSITIVE_IS_OFF.contains(&name) && matches!(cur, ParamValue::Float(x) if *x <= 0.0) {
            continue;
        }
        return Err(CutError::UnimplementedCutActive {
            name: name.to_string(),
            value: describe(cur),
            default: describe(&def),
        });
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

/// The cut class `cuts.f` files an outgoing particle under — `j`, `b`, `l` or
/// `a` — or `None` for one no class-keyed cut reads.
pub(crate) fn cut_class(pdg: i32, maxjetflavor: i64) -> Option<char> {
    classify(0, &ExternalLeg::outgoing(pdg, 0.0), maxjetflavor)
        .letter
        .map(letter_char)
}

fn classify(idx: usize, leg: &ExternalLeg, maxjetflavor: i64) -> LegInfo {
    let a = leg.pdg.unsigned_abs() as i64;
    let is_neutrino = matches!(a, 12 | 14 | 16);
    let letter = if a == 21 || a <= maxjetflavor.min(6) {
        Some(Letter::Jet)
    } else if a > maxjetflavor && a <= 5 {
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
        uncut_decay: false,
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
    // MG stores the raw `dr` value (setcuts.f:346) then squares it once in the
    // cuts.f FIRSTTIME block (`r2min = r2min*dabs(r2min)`, cuts.f:225) before
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
/// `max(dsqrt_shat², min dilepton-mass², (Σ pT threshold)², (Σ final mass)²)`.
///
/// The last two are the general bounds, and they are general because they are
/// read off the partonic centre of mass, where `√ŝ = Σᵢ Eᵢ` over the final state.
/// A boost along the beam leaves each leg's transverse momentum alone, so the
/// `pT` a lab-frame cut holds a leg above bounds that leg's energy in *that*
/// frame too, and `Eᵢ ≥ max(mᵢ, pTᵢ)` gives one bound per branch of the max:
///
///   `√ŝ ≥ Σᵢ pTᵢ^min`      and      `√ŝ ≥ Σᵢ mᵢ`.
///
/// Both hold for any number of outgoing legs — no back-to-back argument and no
/// two-body assumption enters — and they are the two `setcuts.f` derives as
/// `smin_p**2` (its per-letter-class transverse sum, `setcuts.f:527-676`) and
/// `smin = max(smin, (Σ pmass(i))**2, dsqrt_shat**2)` (`setcuts.f:702-707`).
/// Two departures from its arithmetic, both in the direction of the derivation:
/// MadGraph sums the transverse term per letter class and *adds* the classes,
/// where summing over all classes at once is the same bound when one class is
/// cut and a tighter one when several are; and its per-leg term is
/// `max(e_X, pt_X, …)`, where only the transverse threshold is used here,
/// because an energy cut is a lab-frame quantity and the sum is taken in the
/// partonic centre of mass.
///
/// A run whose final state is massless and uncut still gets `0` here, which is
/// what `dsqrt_shat` is for: no cut implies no threshold.
fn shat_min_hint(
    rc: &RunCard,
    legs: &[ExternalLeg],
    infos: &[LegInfo],
    single: &[SingleLegCut],
) -> f64 {
    let dsqrt_shat = rc.float("dsqrt_shat");
    let mut hint = dsqrt_shat * dsqrt_shat;

    // A same-flavour opposite-charge lepton pair carries the mmll bound;
    // m_pair ≤ √ŝ makes mmll² a valid lower bound on ŝ.
    let mmll = rc.float("mmll");
    if mmll > 0.0 {
        let has_ll_pair = infos.iter().enumerate().any(|(a, li)| {
            infos.iter().skip(a + 1).any(|lj| {
                li.do_cuts
                    && lj.do_cuts
                    && li.letter == Some(Letter::Lepton)
                    && lj.letter == Some(Letter::Lepton)
                    && li.pdg.unsigned_abs() == lj.pdg.unsigned_abs()
                    && li.pdg * lj.pdg < 0
            })
        });
        if has_ll_pair {
            hint = hint.max(mmll * mmll);
        }
    }

    // Σ over the legs a single-leg cut holds above a transverse threshold. Legs
    // without one contribute nothing, which is what makes the sum a bound rather
    // than an estimate.
    let pt_sum: f64 = single.iter().map(|c| c.pt_min.max(0.0)).sum();
    hint = hint.max(pt_sum * pt_sum);

    // Σ over every final-state mass, cut or not: the production threshold.
    let mass_sum: f64 = legs
        .iter()
        .filter(|l| l.is_final)
        .map(|l| l.mass.max(0.0))
        .sum();
    hint = hint.max(mass_sum * mass_sum);

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

    // ── filters that admit every point ──────────────────────────────────

    fn z_decay_legs() -> Vec<ExternalLeg> {
        vec![
            ExternalLeg::incoming(23, 91.1876),
            ExternalLeg::outgoing(11, 0.0),
            ExternalLeg::outgoing(-11, 0.0),
        ]
    }

    /// Every lepton threshold the card has, switched off.
    const LEPTON_CUTS_OFF: &str = "0 = ptl\n-1 = ptlmax\n0 = el\n-1 = elmax\n-1 = etal\n\
                                   0 = etalmin\n0 = drll\n-1 = drllmax\n0 = mmll\n-1 = mmllmax\n";

    #[test]
    fn a_filter_with_every_threshold_off_admits_every_point() {
        let legs = z_decay_legs();
        let off = Cuts::compile(&card(LEPTON_CUTS_OFF), &legs).unwrap();
        assert!(off.admits_every_point());
        let decay = Cuts::compile(&RunCard::decay_default(), &legs).unwrap();
        assert!(decay.admits_every_point());
        let lo_default = Cuts::compile(&card(""), &legs).unwrap();
        assert!(!lo_default.admits_every_point());
    }

    /// Each family of threshold, switched on alone against an otherwise empty
    /// filter, withdraws the claim.
    #[test]
    fn each_threshold_withdraws_the_claim_that_nothing_is_rejected() {
        let legs = z_decay_legs();
        for on in [
            "1 = ptl",
            "50 = ptlmax",
            "1 = el",
            "50 = elmax",
            "2.5 = etal",
            "0.5 = etalmin",
            "0.4 = drll",
            "3 = drllmax",
            "1 = mmll",
            "50 = mmllmax",
            "1 = mmnl",
            "50 = mmnlmax",
            "1 = htjmin",
            "50 = htjmax",
            "1 = ihtmin",
            "50 = ihtmax",
        ] {
            let cuts = Cuts::compile(&card(&format!("{LEPTON_CUTS_OFF}{on}\n")), &legs).unwrap();
            assert!(!cuts.admits_every_point(), "{on}");
        }
    }

    /// The ŝ window and the decay-chain windows are thresholds too.
    #[test]
    fn a_shat_window_or_a_decay_window_withdraws_the_claim() {
        let off = |extra: &str| card(&format!("{LEPTON_CUTS_OFF}{extra}"));
        let open = Cuts::compile(&off(""), &dy_legs()).unwrap();
        assert!(open.admits_every_point());
        for on in ["100 = dsqrt_shat\n", "1000 = dsqrt_shatmax\n"] {
            let cuts = Cuts::compile(&off(on), &dy_legs()).unwrap();
            assert!(!cuts.admits_every_point(), "{on}");
        }

        let lines = forced(vec![vec![z_line(0b0011), z_line(0b1100)]]);
        let legs = four_lepton_legs();
        let unforced = Cuts::compile(&off(""), &legs).unwrap();
        assert!(unforced.admits_every_point());
        let windowed = Cuts::compile_with(&off(""), &legs, &lines).unwrap();
        assert!(windowed.has_windows() && !windowed.admits_every_point());
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

    /// `p p > b b~` external legs at `maxjetflavor = 4`, so both b quarks
    /// classify as the `b` letter and carry `ptb` rather than `ptj`.
    fn bb_legs(mass: f64) -> Vec<ExternalLeg> {
        vec![
            ExternalLeg::incoming(21, 0.0),
            ExternalLeg::incoming(21, 0.0),
            ExternalLeg::outgoing(5, mass),
            ExternalLeg::outgoing(-5, mass),
        ]
    }

    /// The floor a purely hadronic final state gets, against the number
    /// MadGraph's own `setcuts.f` computes for the same card.
    ///
    /// For the banked `pp_to_bb_fixed` card — `ptb = 20`, `mb = 4.7`,
    /// `mmbb = dsqrt_shat = 0` — `setcuts.f`'s b-class branch accumulates
    /// `smin_p = max(eb,ptb,xptb,0) + max(eb,ptb,0) = 40` and takes
    /// `max(smin_p**2, -2·mb**2, 0) = 1600`, then `max(smin, (2·mb)**2, 0)`
    /// leaves it at 1600. Nothing here may exceed that: the banked run
    /// integrated `τ` from `1600/s` up, so a higher floor would clip a region
    /// its cross section covers.
    #[test]
    fn a_hadronic_final_state_gets_a_floor_from_its_pt_cut() {
        let bb = card("4 = maxjetflavor\n20.0 = ptb\n5.0 = etab\n0.0 = mmbb\n0.0 = dsqrt_shat\n");
        let cuts = Cuts::compile(&bb, &bb_legs(4.7)).unwrap();
        assert_eq!(cuts.shat_min(), 1600.0);

        // The mass sum takes over once it beats the transverse sum: two 100 GeV
        // legs cannot be made below (2·100)² whatever the pT threshold says.
        let heavy = Cuts::compile(&bb, &bb_legs(100.0)).unwrap();
        assert_eq!(heavy.shat_min(), 40000.0);
    }

    /// The `(τ, y)` map divides by `ln(1/τ_min)`, so `shat_min = 0` is not a
    /// loose floor but an unusable one. Every final state with a mass or a
    /// transverse threshold has to give a positive value, whatever its flavour
    /// content — that is the property, not the individual numbers.
    #[test]
    fn every_massive_or_pt_cut_final_state_has_a_positive_floor() {
        let cases: [(&str, Vec<ExternalLeg>); 5] = [
            // b quarks: a pT cut and a mass, no lepton anywhere.
            ("4 = maxjetflavor\n20.0 = ptb\n", bb_legs(4.7)),
            // Massless jets: the pT cut alone.
            (
                "4 = maxjetflavor\n20.0 = ptj\n",
                vec![
                    ExternalLeg::incoming(21, 0.0),
                    ExternalLeg::incoming(21, 0.0),
                    ExternalLeg::outgoing(21, 0.0),
                    ExternalLeg::outgoing(21, 0.0),
                ],
            ),
            // Photons.
            (
                "10.0 = pta\n",
                vec![
                    ExternalLeg::incoming(2, 0.0),
                    ExternalLeg::incoming(-2, 0.0),
                    ExternalLeg::outgoing(22, 0.0),
                    ExternalLeg::outgoing(22, 0.0),
                ],
            ),
            // Top pairs: `do_cuts` is off above 20 GeV, so the mass sum is the
            // only bound left and it has to be the one that fires.
            (
                "4 = maxjetflavor\n",
                vec![
                    ExternalLeg::incoming(21, 0.0),
                    ExternalLeg::incoming(21, 0.0),
                    ExternalLeg::outgoing(6, 173.0),
                    ExternalLeg::outgoing(-6, 173.0),
                ],
            ),
            // Leptons, the case that already worked.
            ("", dy_legs()),
        ];
        for (text, legs) in cases {
            let cuts = Cuts::compile(&card(text), &legs).unwrap();
            let shat_min = cuts.shat_min();
            assert!(
                shat_min > 0.0 && shat_min.is_finite(),
                "legs {legs:?} under card {text:?} left shat_min = {shat_min}"
            );
            assert!(
                (1.0f64 / (shat_min / (13000.0 * 13000.0))).ln().is_finite(),
                "legs {legs:?} under card {text:?} leave ln(1/tau_min) unusable"
            );
        }
    }

    // ── spacelike floor ─────────────────────────────────────────────────

    /// `p p > l+ l- j` external legs: `[u, u~, e+, e-, g]`.
    fn llj_legs() -> Vec<ExternalLeg> {
        vec![
            ExternalLeg::incoming(2, 0.0),
            ExternalLeg::incoming(-2, 0.0),
            ExternalLeg::outgoing(-11, 0.0),
            ExternalLeg::outgoing(11, 0.0),
            ExternalLeg::outgoing(21, 0.0),
        ]
    }

    /// The energy floor of a subsystem is the sum of its legs' transverse-momentum
    /// thresholds, the `emin` threshold where it is the larger one, and nothing for
    /// a leg the cuts leave free.
    #[test]
    fn energy_floor_sums_the_legs_thresholds() {
        // llj final-state order: l+, l-, j ⇒ ptl + ptl + ptj = 10 + 10 + 20.
        let llj = Cuts::compile(&RunCard::default(), &llj_legs()).unwrap();
        assert_eq!(llj.energy_floor(0b001), 10.0);
        assert_eq!(llj.energy_floor(0b100), 20.0);
        assert_eq!(llj.energy_floor(0b011), 20.0);
        assert_eq!(llj.energy_floor(0b111), 40.0);
        assert_eq!(llj.energy_floor(0), 0.0);

        // `ej` above `ptj` takes over for the jet and only for the jet.
        let energetic = Cuts::compile(&card("50 = ej\n"), &llj_legs()).unwrap();
        assert_eq!(energetic.energy_floor(0b100), 50.0);
        assert_eq!(energetic.energy_floor(0b011), 20.0);

        // A card that switches the thresholds off implies nothing.
        let free = Cuts::compile(&card("0 = ptj\n0 = ptl\n"), &llj_legs()).unwrap();
        assert_eq!(free.energy_floor(0b111), 0.0);
    }

    /// With `xqcut` set the resolved card carries `setcuts.f`'s rewrites, and the
    /// filter applies them: the jet threshold is `xqcut`, the lepton–jet
    /// separation is gone, and the jet–jet mass threshold is `xqcut`, each only
    /// on legs `do_cuts` admits.
    #[test]
    fn an_xqcut_card_cuts_with_the_rewritten_jet_thresholds() {
        let rc = card("30 = xqcut\n");
        let llj = Cuts::compile(&rc, &llj_legs()).unwrap();
        assert_eq!(llj.spacelike_floor(), 900.0);
        let (b1, b2) = beams(500.0);
        let lp = lep(50.0, 0.0, 0.0);
        let lm = lep(50.0, 0.3, std::f64::consts::PI);
        // A jet at 29 GeV fails, at 31 passes, and passes 0.1 away from a lepton.
        assert!(!llj.pass(&[b1, b2, lp, lm, lep(29.0, 1.0, 1.0)]));
        assert!(llj.pass(&[b1, b2, lp, lm, lep(31.0, 1.0, 1.0)]));
        assert!(llj.pass(&[b1, b2, lp, lm, lep(31.0, 0.1, 0.0)]));
        let plain = Cuts::compile(&RunCard::default(), &llj_legs()).unwrap();
        assert!(!plain.pass(&[b1, b2, lp, lm, lep(31.0, 0.1, 0.0)]));

        // Two jets: `mmjj = 30` and `drjj = 0`.
        let jj = vec![
            ExternalLeg::incoming(21, 0.0),
            ExternalLeg::incoming(21, 0.0),
            ExternalLeg::outgoing(21, 0.0),
            ExternalLeg::outgoing(21, 0.0),
        ];
        let jj_cuts = Cuts::compile(&rc, &jj).unwrap();
        // Close together: pair mass ≈ 40 · 0.3 = 12 < 30, refused by mmjj alone.
        assert!(!jj_cuts.pass(&[b1, b2, lep(40.0, 0.0, 0.0), lep(40.0, 0.3, 0.0)]));
        assert!(jj_cuts.pass(&[b1, b2, lep(40.0, 0.0, 0.0), lep(40.0, 0.9, 0.0)]));

        // `do_cuts`: a leg above 20 GeV of mass is exempt from every jet cut.
        let heavy = vec![
            ExternalLeg::incoming(21, 0.0),
            ExternalLeg::incoming(21, 0.0),
            ExternalLeg::outgoing(1, 25.0),
            ExternalLeg::outgoing(-1, 25.0),
        ];
        let heavy_cuts = Cuts::compile(&rc, &heavy).unwrap();
        assert_eq!(heavy_cuts.spacelike_floor(), 0.0);
    }

    /// The phase-space floors MadEvent derives from `xqcut` (`setcuts.f`
    /// `setxqcuts`, read in `myamp.f` `set_peaks`) are a jet's energy floor
    /// `√(xqcut² − m²)` and an s-channel jet pair's mass floor `xqcut`. After the
    /// `ptj = mmjj = xqcut` rewrite the compiled cuts carry both, so the channels
    /// the maps build from `energy_floor`, `timelike_floor` and `spacelike_floor`
    /// already start there: a massless jet's energy floor is its `pT` threshold
    /// `xqcut`, a jet pair's mass floor is `mmjj = xqcut`, and the spacelike
    /// regulator is `xqcut²`. A rewrite that stopped reaching any of the three
    /// would show here.
    #[test]
    fn madevents_xqcut_floors_reach_the_maps_through_the_rewritten_cuts() {
        let rc = card("30 = xqcut\n50 = mmll\n");
        let lljj = vec![
            ExternalLeg::incoming(21, 0.0),
            ExternalLeg::incoming(21, 0.0),
            ExternalLeg::outgoing(11, 0.0),
            ExternalLeg::outgoing(-11, 0.0),
            ExternalLeg::outgoing(21, 0.0),
            ExternalLeg::outgoing(2, 0.0),
        ];
        let cuts = Cuts::compile(&rc, &lljj).unwrap();
        let (leptons, jet, other_jet) = (0b0011, 0b0100, 0b1000);
        assert_eq!(cuts.energy_floor(jet), 30.0);
        assert_eq!(cuts.energy_floor(other_jet), 30.0);
        assert_eq!(cuts.timelike_floor(jet | other_jet), 900.0);
        assert_eq!(cuts.timelike_floor(leptons), 2500.0);
        assert_eq!(cuts.spacelike_floor(), 900.0);
        // Without `xqcut` the same card's jets keep `ptj = 20`, and their pair
        // floor is only the one `drjj = 0.4` implies, `2·20²·(1 − cos 0.4) ≈ 63`.
        let plain = Cuts::compile(&card("50 = mmll\n"), &lljj).unwrap();
        assert_eq!(plain.energy_floor(jet), 20.0);
        assert!(plain.timelike_floor(jet | other_jet) < 64.0);
    }

    /// MadEvent's lower limit on τ under `xqcut` is `(Σ xe)²/s` with a jet's
    /// floor `max(ptj, sqrt(xqcut² − m²))` (`myamp.f`, `setxqcuts`). With `ptj`
    /// rewritten to `xqcut` that limit is implied by the cuts: no accepted point
    /// sits below it, so it moves no cross section. A limit that did cut would
    /// show here as an accepted point below it.
    #[test]
    fn madevents_xqcut_tau_floor_is_implied_by_the_rewritten_cuts() {
        use rand::{Rng, SeedableRng};
        let rc = card("30 = xqcut\n");
        let cuts = Cuts::compile(&rc, &llj_legs()).unwrap();
        // Leptons at `ptl`, the jet at `max(ptj, xqcut)`.
        let etot: f64 = rc.float("ptl") * 2.0 + rc.float("ptj").max(30.0);
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0x5EED_41);
        let mut accepted = 0;
        for _ in 0..200_000 {
            let l1 = lep(
                rng.random_range(5.0..80.0),
                rng.random_range(-3.0..3.0),
                rng.random_range(0.0..6.3),
            );
            let l2 = lep(
                rng.random_range(5.0..80.0),
                rng.random_range(-3.0..3.0),
                rng.random_range(0.0..6.3),
            );
            let (px, py) = (-(l1.px() + l2.px()), -(l1.py() + l2.py()));
            let pt = (px * px + py * py).sqrt();
            let y: f64 = rng.random_range(-4.0..4.0);
            let j = V::new(pt * y.cosh(), px, py, pt * y.sinh());
            let (e, pz) = (l1.e() + l2.e() + j.e(), l1.pz() + l2.pz() + j.pz());
            let b1 = V::new((e + pz) / 2.0, 0.0, 0.0, (e + pz) / 2.0);
            let b2 = V::new((e - pz) / 2.0, 0.0, 0.0, -(e - pz) / 2.0);
            if cuts.pass(&[b1, b2, l1, l2, j]) {
                accepted += 1;
                let shat = e * e - pz * pz;
                assert!(
                    shat >= etot * etot,
                    "ŝ = {shat} below the τ floor {}",
                    etot * etot
                );
            }
        }
        assert!(accepted > 1000, "only {accepted} accepted points");
    }

    #[test]
    fn spacelike_floor_is_the_hardest_single_leg_pt_squared() {
        // The banked llj card: ptj = 20 over ptl = 10 ⇒ |t| ≳ 400 GeV².
        let llj = Cuts::compile(&RunCard::default(), &llj_legs()).unwrap();
        assert_eq!(llj.spacelike_floor(), 400.0);

        // The jet is what carries it: raise the lepton threshold past the jet's
        // and the floor follows the leptons instead.
        let leptonic = Cuts::compile(&card("30 = ptl\n"), &llj_legs()).unwrap();
        assert_eq!(leptonic.spacelike_floor(), 900.0);

        // Drell-Yan has no jet, so its floor is the lepton threshold.
        let dy = Cuts::compile(&RunCard::default(), &dy_legs()).unwrap();
        assert_eq!(dy.spacelike_floor(), 100.0);
    }

    /// A run with no active single-leg `pT` cut has no scale to bound the transfer
    /// with, and says so rather than inventing one: a zero floor leaves every
    /// peripheral channel's support whole, and the channel floors its pole from a
    /// default scale of its own.
    #[test]
    fn spacelike_floor_is_zero_without_an_active_pt_cut() {
        let none = Cuts::compile(&card("0 = ptj\n0 = ptl\n"), &llj_legs()).unwrap();
        assert_eq!(none.spacelike_floor(), 0.0);
    }

    /// The floor is read off the *compiled* cuts, so a leg that MadGraph exempts
    /// from single-leg cuts contributes no scale. A `pdg = 5` final leg is a b at
    /// `maxjetflavor = 4` (`ptb = 0`) and a jet at `5` (`ptj = 20`).
    #[test]
    fn spacelike_floor_follows_class_membership() {
        let legs = vec![
            ExternalLeg::incoming(2, 0.0),
            ExternalLeg::incoming(-2, 0.0),
            ExternalLeg::outgoing(5, 0.0),
            ExternalLeg::outgoing(-5, 0.0),
        ];
        let as_b = Cuts::compile(&card("4 = maxjetflavor\n"), &legs).unwrap();
        assert_eq!(as_b.spacelike_floor(), 0.0);
        let as_jet = Cuts::compile(&card("5 = maxjetflavor\n"), &legs).unwrap();
        assert_eq!(as_jet.spacelike_floor(), 400.0);
    }

    /// The transfer bound the floor is derived from: a massless beam emitting a
    /// massless leg at transverse momentum `pT` transfers at least `pT²`. Checked
    /// against `t` computed from the momenta, over a spread of rapidities and
    /// collision energies, so the derivation is pinned rather than asserted.
    #[test]
    fn a_transverse_momentum_threshold_bounds_the_transfer_it_implies() {
        let pt = 20.0;
        for &sqrt_s in &[100.0, 500.0, 13000.0] {
            let (b1, _) = beams(0.5 * sqrt_s);
            for i in 0..40 {
                let y = -4.0 + 0.2 * f64::from(i);
                let emitted = lep(pt, y, 0.7);
                // Only rapidities the collision energy can actually reach.
                if emitted.e() > 0.5 * sqrt_s {
                    continue;
                }
                let t = (b1 - emitted).m2();
                assert!(t <= 0.0, "the rung is spacelike: t = {t}");
                assert!(
                    -t >= pt * pt - 1e-9,
                    "sqrt_s = {sqrt_s}, y = {y}: |t| = {} below pT² = {}",
                    -t,
                    pt * pt
                );
            }
        }
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
        assert!(!cuts.pass(&[
            b1,
            b2,
            lep(5.0, 0.0, 0.0),
            lep(0.01, 0.0, std::f64::consts::PI)
        ]));
        assert!(cuts.pass(&[
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
        assert!(cuts.pass(&[b1, b2, lep(0.5, 6.0, 0.0)]));
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
            other => panic!("{other}"),
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

    /// Cards at 13 TeV (`ebeam = 6500`): the transverse-momentum ceiling is 6500.
    const LHC: &str = "6500 = ebeam1\n6500 = ebeam2\n";

    fn lhc(extra: &str) -> RunCard {
        card(&format!("{LHC}{extra}"))
    }

    #[test]
    fn detection_needs_no_legs() {
        assert!(detect_unimplemented(&card("")).is_ok());
        assert!(matches!(
            detect_unimplemented(&card("30 = ptj1min\n")),
            Err(CutError::UnimplementedCutActive { name, .. }) if name == "ptj1min"
        ));
    }

    /// The nine single-object transverse-momentum maxima, written out here so
    /// that membership of `SINGLE_PT_MAXIMA` is checked against an independent list.
    const NINE_PT_MAXIMA: [&str; 9] = [
        "missetmax",
        "ptj1max",
        "ptj2max",
        "ptj3max",
        "ptj4max",
        "ptl1max",
        "ptl2max",
        "ptl3max",
        "ptl4max",
    ];

    #[test]
    fn unbindable_pt_maxima_read_as_default() {
        for name in NINE_PT_MAXIMA {
            let cut = lhc(&format!("1e5 = {name}\n"));
            assert!(detect_unimplemented(&cut).is_ok(), "{name}");
            assert!(Cuts::compile(&cut, &dy_legs()).is_ok(), "{name}");
        }
    }

    /// Every other unimplemented cut stays refused at a value no collider can
    /// bind, so the no-op rule covers exactly `NINE_PT_MAXIMA`.
    #[test]
    fn no_other_unimplemented_cut_reads_an_unbindable_value_as_default() {
        let mut refused = 0;
        for &name in UNIMPLEMENTED_CUTS {
            if NINE_PT_MAXIMA.contains(&name) {
                continue;
            }
            let Ok(cut) = RunCard::parse(&format!("{LHC}1e5 = {name}\n")) else {
                continue;
            };
            assert!(
                matches!(
                    detect_unimplemented(&cut),
                    Err(CutError::UnimplementedCutActive { .. })
                ),
                "{name} at 1e5"
            );
            refused += 1;
        }
        assert!(refused > 20, "only {refused} cuts exercised");
    }

    #[test]
    fn bindable_pt_maxima_are_refused() {
        for name in NINE_PT_MAXIMA {
            let err = detect_unimplemented(&lhc(&format!("500 = {name}\n"))).unwrap_err();
            match err {
                CutError::UnimplementedCutActive { name: n, value, .. } => {
                    assert_eq!(n, name);
                    assert_eq!(value, "500");
                }
                other => panic!("{name}: {other}"),
            }
        }
    }

    #[test]
    fn pt_maximum_threshold_is_half_the_collider_energy() {
        assert!(detect_unimplemented(&lhc("6500 = missetmax\n")).is_ok());
        assert!(detect_unimplemented(&lhc("6499.9 = missetmax\n")).is_err());
        let beams = "100 = ebeam1\n50 = ebeam2\n";
        assert!(detect_unimplemented(&card(&format!("{beams}75 = ptj1max\n"))).is_ok());
        assert!(detect_unimplemented(&card(&format!("{beams}74.9 = ptj1max\n"))).is_err());
    }

    #[test]
    fn unbindable_value_does_not_excuse_other_cuts() {
        match detect_unimplemented(&lhc("1e5 = missetmax\n1e5 = ht2max\n")) {
            Err(CutError::UnimplementedCutActive { name, .. }) => assert_eq!(name, "ht2max"),
            Ok(()) => panic!("ht2max = 1e5 is an active partial-sum cut"),
            Err(other) => panic!("{other}"),
        }
        assert!(detect_unimplemented(&lhc("1e5 = ptj1min\n")).is_err());
    }

    /// MadGraph never applies `ptonium` or `etaonium` (`banner.py` registers
    /// them as legacy, out of `run_card.inc`), so no value is refused and none
    /// changes the compiled cuts. The values are the ones CMS cards ship
    /// (`0 = ptonium`, `-1 = etaonium`), MadGraph's defaults, and two that
    /// would bind if anything read them.
    #[test]
    fn onium_cuts_are_accepted_at_any_value_and_change_nothing() {
        let reference = Cuts::compile(&lhc(""), &dy_legs()).unwrap();
        for extra in [
            "0 = ptonium\n-1 = etaonium\n",
            "1 = ptonium\n0.6 = etaonium\n",
            "500 = ptonium\n0.01 = etaonium\n",
        ] {
            let rc = lhc(extra);
            assert!(detect_unimplemented(&rc).is_ok(), "{extra}");
            assert_eq!(
                Cuts::compile(&rc, &dy_legs()).unwrap(),
                reference,
                "{extra}"
            );
        }
    }

    // ── cut-implied timelike floors ─────────────────────────────────────

    /// The slack allowed for `DELTA_PHI`'s cosine clamp has to cover the angle the
    /// clamp actually invents, or a bound derived at the relaxed radius is not a
    /// bound at all.
    #[test]
    fn the_clamp_slack_covers_the_angle_the_clamp_invents() {
        assert!(DELTA_PHI_CLAMP_SLACK >= (0.99999999f64).acos());
    }

    /// The `ptl`/`ΔR` pair bound at `p p > l+ l- j`'s thresholds, against the value
    /// worked out by hand: `2·10·10·(1 − cos 0.4) = 15.79 GeV²`, so `m_ll ≥ 3.97`.
    #[test]
    fn the_llj_lepton_pair_floor_is_the_ptl_and_dr_minimum() {
        let cuts = Cuts::compile(&card("10 = ptl\n0.4 = drll\n"), &dy_legs()).unwrap();
        let floor = cuts.timelike_floor(0b11);
        let hand = 2.0 * 10.0 * 10.0 * (1.0 - 0.4f64.cos());
        assert!(
            (floor - hand).abs() < 1e-5 * hand,
            "floor {floor} against {hand} by hand"
        );
        assert!(
            (floor.sqrt() - 3.9734).abs() < 1e-3,
            "m_ll floor {}",
            floor.sqrt()
        );
    }

    /// An explicit pair-mass threshold is the bound where it is the tighter one,
    /// and a subsystem no threshold speaks about gets none.
    #[test]
    fn an_explicit_pair_mass_threshold_takes_over_from_the_derived_one() {
        let cuts = Cuts::compile(&card("10 = ptl\n0.4 = drll\n50 = mmll\n"), &dy_legs()).unwrap();
        assert_eq!(cuts.timelike_floor(0b11), 2500.0);
        // One leg alone is bounded by nothing pairwise.
        assert_eq!(cuts.timelike_floor(0b01), 0.0);
        assert_eq!(cuts.timelike_floor(0b10), 0.0);
    }

    /// An **inverted** mass window is a veto band, not a lower bound: it accepts
    /// everything below `mmll`, so the floor must fall back to the derived bound —
    /// and a point inside the accepted low-mass region proves the window really is
    /// the inverted one.
    #[test]
    fn a_veto_band_implies_no_mass_floor() {
        let cuts = Cuts::compile(
            &card("10 = ptl\n0.4 = drll\n-1 = etal\n50 = mmll\n20 = mmllmax\n"),
            &dy_legs(),
        )
        .unwrap();
        let derived = 2.0 * 10.0 * 10.0 * (1.0 - 0.4f64.cos());
        let floor = cuts.timelike_floor(0b11);
        assert!(
            floor < derived * 1.001 && floor > derived * 0.999,
            "floor {floor}"
        );

        // A pair below the band, separated widely enough to clear `drll`: accepted,
        // and well below the `2500` an inverted window's lower edge would have
        // imposed.
        let (l1, l2) = (lep(48.0, 0.0, 0.0), lep(48.0, 0.0, 0.41));
        let m2 = (l1 + l2).m2();
        assert!(m2 < 400.0, "m2 {m2} is not below the veto band");
        assert!(cuts.pass(&dy_momenta(l1, l2)), "m2 {m2} was rejected");
        assert!(m2 >= floor, "m2 {m2} below the floor {floor}");
    }

    /// The minimisation behind the `p_T`/`ΔR` bound, sampled on the boundary it is
    /// taken over.
    ///
    /// Both legs sit at the transverse threshold and the pair at the separation
    /// threshold, swept around the `(Δy, Δφ)` circle and boosted along the beam.
    /// Every such configuration is accepted, so every one of them has to lie at or
    /// above the floor — and the sweep's own minimum has to come back down to it,
    /// or the bound is loose enough to be cutting into the accepted region
    /// somewhere else.
    #[test]
    fn the_pair_floor_is_attained_on_the_accepted_boundary() {
        let cuts = Cuts::compile(&card("10 = ptl\n0.4 = drll\n-1 = etal\n"), &dy_legs()).unwrap();
        let floor = cuts.timelike_floor(0b11);
        // A hair outside the threshold circle, so the sweep is inside the accepted
        // region rather than on its numerical edge.
        let r = 0.4 * (1.0 + 1e-9);

        let mut lowest = f64::INFINITY;
        let mut accepted = 0;
        for k in 0..=64 {
            let theta = 0.5 * std::f64::consts::PI * k as f64 / 64.0;
            let (dy, dphi) = (r * theta.cos(), r * theta.sin());
            for &boost in &[0.0f64, 0.7, -1.9, 4.0] {
                for &extra in &[0.0f64, 5.0, 40.0] {
                    // A hair above the transverse threshold too: `pt` is rebuilt
                    // from the components and a leg placed exactly at `ptl` can
                    // round below it.
                    let l1 = lep(10.0 * (1.0 + 1e-9), boost, 0.0);
                    let l2 = lep((10.0 + extra) * (1.0 + 1e-9), boost + dy, dphi);
                    assert!(
                        cuts.pass(&dy_momenta(l1, l2)),
                        "the boundary configuration (θ {theta}, boost {boost}, extra {extra}) \
                         is not accepted, so the sweep is not sampling the accepted region"
                    );
                    accepted += 1;
                    let m2 = (l1 + l2).m2();
                    assert!(
                        m2 >= floor,
                        "an accepted pair at m² {m2} sits below the floor {floor} \
                         (θ {theta}, boost {boost}, extra {extra})"
                    );
                    lowest = lowest.min(m2);
                }
            }
        }
        assert!(accepted > 100, "{accepted} boundary configurations");
        assert!(
            lowest < floor * 1.000_01,
            "the sweep's minimum {lowest} stands well above the floor {floor}, so the \
             minimisation is not the one the bound claims"
        );
    }

    /// The bound over a whole final state, against configurations drawn rather than
    /// constructed: no point the cuts accept may sit below any subsystem's floor.
    ///
    /// This is the bias falsifier. The floors narrow where each channel puts its
    /// invariant draws, and the estimator survives that only because the region
    /// given up is entirely outside the cuts — which is this test's claim, taken
    /// over every subsystem of the final state at once.
    /// The energy floor is a bound, not a scale: no configuration the cuts accept
    /// puts any subsystem below it, in the partonic frame or in a frame boosted
    /// along the beam axis. A shaped angular draw confines itself to the window
    /// the floor admits, so an accepted point outside it would be a region that
    /// channel can never generate while its density still reports it — a bias,
    /// not a variance. Uses `g u > e+ e- u`'s legs and its banked card thresholds.
    #[test]
    fn no_accepted_configuration_sits_below_an_energy_floor() {
        use crate::phasespace::channel::PhaseSpaceMap;
        use crate::phasespace::rng::SubStream;
        use crate::phasespace::RamboChannel;

        let legs = vec![
            ExternalLeg::incoming(21, 0.0),
            ExternalLeg::incoming(2, 0.0),
            ExternalLeg::outgoing(-11, 0.0),
            ExternalLeg::outgoing(11, 0.0),
            ExternalLeg::outgoing(2, 0.0),
        ];
        let cuts = Cuts::compile(
            &card("20 = ptj\n10 = ptl\n5 = etaj\n2.5 = etal\n0.4 = drll\n0.4 = drjl\n"),
            &legs,
        )
        .unwrap();
        let floors: Vec<f64> = (0u64..8).map(|m| cuts.energy_floor(m)).collect();
        assert_eq!(
            floors[0b111], 40.0,
            "every leg must carry a floor for this to check anything"
        );

        let rambo = RamboChannel::<f64>::new(500.0, vec![0.0; 3]);
        let mut stream = SubStream::from_stream(0xE_F100, 3);
        let (mut accepted, mut tightest) = (0usize, f64::INFINITY);
        for _ in 0..200_000 {
            let u = stream.uniforms::<f64>(rambo.ndim());
            let out = rambo.sample(&u).momenta;
            let (b1, b2) = beams(250.0);
            let momenta = vec![b1, b2, out[0], out[1], out[2]];
            if !cuts.pass(&momenta) {
                continue;
            }
            accepted += 1;
            // A longitudinal boost moves no pT, so the bound must survive one; the
            // rapidity is the widest a 13 TeV collision gives a 500 GeV system.
            for y in [0.0f64, 1.5, -2.5] {
                let (ch, sh) = (y.cosh(), y.sinh());
                for (mask, &floor) in floors.iter().enumerate().skip(1) {
                    let e: f64 = (0..3)
                        .filter(|slot| mask & (1 << slot) != 0)
                        .map(|slot| {
                            let p = momenta[2 + slot];
                            ch * p.e() + sh * p.pz()
                        })
                        .sum();
                    assert!(
                        e >= floor,
                        "an accepted point puts subsystem {mask:03b} at E {e} (y = {y}), below \
                         its floor {floor}"
                    );
                    tightest = tightest.min(e / floor);
                }
            }
        }
        assert!(accepted > 5_000, "only {accepted} accepted points");
        eprintln!("{accepted} accepted, closest approach to an energy floor {tightest:.4}");
    }

    #[test]
    fn no_accepted_configuration_sits_below_a_subsystem_floor() {
        use crate::phasespace::channel::PhaseSpaceMap;
        use crate::phasespace::rng::SubStream;
        use crate::phasespace::RamboChannel;

        // `p p > l+ l- j` thresholds, and the same with the lepton pair held off the
        // photon pole — the two cards the weight-tail probe compares.
        for text in [
            "20 = ptj\n10 = ptl\n5 = etaj\n2.5 = etal\n0.4 = drll\n0.4 = drjl\n0 = mmll\n",
            "20 = ptj\n10 = ptl\n5 = etaj\n2.5 = etal\n0.4 = drll\n0.4 = drjl\n50 = mmll\n",
        ] {
            let legs = vec![
                ExternalLeg::incoming(2, 0.0),
                ExternalLeg::incoming(-2, 0.0),
                ExternalLeg::outgoing(-11, 0.0),
                ExternalLeg::outgoing(11, 0.0),
                ExternalLeg::outgoing(21, 0.0),
            ];
            let cuts = Cuts::compile(&card(text), &legs).unwrap();
            let floors: Vec<f64> = (0u64..8).map(|m| cuts.timelike_floor(m)).collect();
            assert!(
                floors.iter().any(|&f| f > 0.0),
                "no floor is active, so this card checks nothing"
            );

            let rambo = RamboChannel::<f64>::new(300.0, vec![0.0; 3]);
            let mut stream = SubStream::from_stream(0x7F_1005, 3);
            let (mut accepted, mut tightest) = (0usize, f64::INFINITY);
            for _ in 0..40_000 {
                let u = stream.uniforms::<f64>(rambo.ndim());
                let out = rambo.sample(&u).momenta;
                let (b1, b2) = beams(150.0);
                let momenta = vec![b1, b2, out[0], out[1], out[2]];
                if !cuts.pass(&momenta) {
                    continue;
                }
                accepted += 1;
                for (mask, &floor) in floors.iter().enumerate().skip(1) {
                    if floor <= 0.0 {
                        continue;
                    }
                    let mut sum = V::new(0.0, 0.0, 0.0, 0.0);
                    for slot in 0..3 {
                        if mask & (1 << slot) != 0 {
                            sum = sum + momenta[2 + slot];
                        }
                    }
                    let m2 = sum.m2();
                    assert!(
                        m2 >= floor,
                        "[{text}] an accepted point puts subsystem {mask:03b} at m² {m2}, \
                         below its floor {floor}"
                    );
                    tightest = tightest.min(m2 / floor);
                }
            }
            assert!(accepted > 500, "[{text}] only {accepted} accepted points");
            eprintln!(
                "[mmll {}] {accepted} accepted, closest approach to a floor {tightest:.4}",
                cuts.timelike_floor(0b011).sqrt(),
            );
        }
    }

    // ── decay-chain windows and cut_decays ────────────────────────────────

    const MZ: f64 = 91.188;
    const WZ: f64 = 2.441404;

    /// `e+ e- > e+ e- mu+ mu-`, legs 2..6.
    fn four_lepton_legs() -> Vec<ExternalLeg> {
        vec![
            ExternalLeg::incoming(-11, 0.0),
            ExternalLeg::incoming(11, 0.0),
            ExternalLeg::outgoing(-11, 0.0),
            ExternalLeg::outgoing(11, 0.0),
            ExternalLeg::outgoing(-13, 0.0),
            ExternalLeg::outgoing(13, 0.0),
        ]
    }

    fn z_line(slots: u64) -> ForcedLine {
        ForcedLine {
            slots,
            mass: MZ,
            width: WZ,
        }
    }

    fn forced(patterns: Vec<Vec<ForcedLine>>) -> ForcedResonances {
        ForcedResonances {
            patterns,
            massless_coloured_core: 0,
            massless_limits: Vec::new(),
        }
    }

    /// Outgoing legs `(a, b, c, d)` with `m(a b) = m1`, `m(c d) = m2`, `m(a d)`
    /// and `m(c b)` far from the pole, and every leg at `pT = m/2` in the
    /// transverse plane.
    fn two_pairs(m1: f64, m2: f64) -> Vec<V> {
        let (b1, b2) = beams(250.0);
        vec![
            b1,
            b2,
            V::new(m1 / 2.0, m1 / 2.0, 0.0, 0.0),
            V::new(m1 / 2.0, -m1 / 2.0, 0.0, 0.0),
            V::new(m2 / 2.0, 0.0, m2 / 2.0, 0.0),
            V::new(m2 / 2.0, 0.0, -m2 / 2.0, 0.0),
        ]
    }

    /// `|m − M| < bwcutoff·Γ` on every forced line, strictly, at the run card's
    /// `bwcutoff`.
    #[test]
    fn a_forced_line_is_cut_outside_its_window() {
        let lines = forced(vec![vec![z_line(0b0011), z_line(0b1100)]]);
        for bwcutoff in [15.0, 5.0] {
            let rc = card(&format!("{bwcutoff} = bwcutoff\n"));
            let cuts = Cuts::compile_with(&rc, &four_lepton_legs(), &lines).unwrap();
            let edge = bwcutoff * WZ;
            assert!(cuts.pass(&two_pairs(MZ + 0.999 * edge, MZ - 0.999 * edge)));
            assert!(!cuts.pass(&two_pairs(MZ + 1.001 * edge, MZ)));
            assert!(!cuts.pass(&two_pairs(MZ, MZ - 1.001 * edge)));
        }
    }

    /// A line without width is never cut (`myamp.f` tests only `prwidth > 0`), and
    /// a width below `small_width_treatment · M` is measured at that floor.
    #[test]
    fn the_window_width_is_madgraphs() {
        let rc = card("");
        let zero = forced(vec![vec![ForcedLine {
            slots: 0b0011,
            mass: MZ,
            width: 0.0,
        }]]);
        // The muons are not decay products here, and sit above the default `ptl`.
        let cuts = Cuts::compile_with(&rc, &four_lepton_legs(), &zero).unwrap();
        assert!(cuts.pass(&two_pairs(10.0, 91.0)));
        let narrow = ForcedLine {
            slots: 0b0011,
            mass: MZ,
            width: 1e-12,
        };
        assert_eq!(narrow.window_width(), MZ * SMALL_WIDTH_TREATMENT);
        let cuts =
            Cuts::compile_with(&rc, &four_lepton_legs(), &forced(vec![vec![narrow]])).unwrap();
        let edge = 15.0 * MZ * SMALL_WIDTH_TREATMENT;
        assert!(cuts.pass(&two_pairs(MZ + 0.99 * edge, 91.0)));
        assert!(!cuts.pass(&two_pairs(MZ + 1.01 * edge, 91.0)));
    }

    /// Where different diagrams force different leg sets, a point passes when every
    /// line of *one* set is inside its window.
    #[test]
    fn a_point_passes_on_any_one_set_of_forced_lines() {
        let rc = card("");
        let both = forced(vec![
            vec![z_line(0b0011), z_line(0b1100)],
            vec![z_line(0b1001), z_line(0b0110)],
        ]);
        let cuts = Cuts::compile_with(&rc, &four_lepton_legs(), &both).unwrap();
        // Two back-to-back pairs on the pole along one axis, so the crossed pairs
        // are collinear and massless.
        let (b1, b2) = beams(250.0);
        let h = MZ / 2.0;
        let straight = vec![
            b1,
            b2,
            V::new(h, h, 0.0, 0.0),
            V::new(h, -h, 0.0, 0.0),
            V::new(h, -h, 0.0, 0.0),
            V::new(h, h, 0.0, 0.0),
        ];
        assert!(cuts.pass(&straight));
        // The second and fourth legs exchanged: now only the crossed pairs sit on
        // the pole.
        let mut crossed = straight.clone();
        crossed.swap(3, 5);
        assert!(cuts.pass(&crossed));
        let only_first = Cuts::compile_with(
            &rc,
            &four_lepton_legs(),
            &forced(vec![vec![z_line(0b0011), z_line(0b1100)]]),
        )
        .unwrap();
        assert!(!only_first.pass(&crossed));
        assert!(!cuts.pass(&two_pairs(MZ, 20.0)));
    }

    /// `cut_decays = F` leaves a decay product uncut — single-leg and pair cuts
    /// alike — while a leg no forced line produces keeps its cuts; `T` cuts both.
    /// The forced line has no width here, so its window cuts nothing and only the
    /// run card's cuts act.
    #[test]
    fn cut_decays_selects_whether_decay_products_are_cut() {
        let widthless = forced(vec![vec![ForcedLine {
            slots: 0b0011,
            mass: MZ,
            width: 0.0,
        }]]);
        let compiled =
            |text: &str| Cuts::compile_with(&card(text), &four_lepton_legs(), &widthless).unwrap();
        let pt_off = compiled("10 = ptl\n-1 = etal\n0 = drll\nFalse = cut_decays\n");
        let pt_on = compiled("10 = ptl\n-1 = etal\n0 = drll\nTrue = cut_decays\n");
        let hard = two_pairs(MZ, MZ);
        assert!(pt_off.pass(&hard) && pt_on.pass(&hard));
        // A decay product below `ptl`.
        let mut soft_product = two_pairs(MZ, MZ);
        soft_product[2] = V::new(5.0, 5.0, 0.0, 0.0);
        assert!(pt_off.pass(&soft_product));
        assert!(!pt_on.pass(&soft_product));
        // A leg no forced line produces, below `ptl`: cut either way.
        let mut soft_other = two_pairs(MZ, MZ);
        soft_other[4] = V::new(5.0, 0.0, 5.0, 0.0);
        assert!(!pt_off.pass(&soft_other) && !pt_on.pass(&soft_other));
        // `drll` between the two products, nearly collinear.
        let dr_off = compiled("0 = ptl\n-1 = etal\n0.4 = drll\nFalse = cut_decays\n");
        let dr_on = compiled("0 = ptl\n-1 = etal\n0.4 = drll\nTrue = cut_decays\n");
        let mut collinear = two_pairs(MZ, MZ);
        collinear[2] = V::new(30.0, 30.0, 0.0, 0.0);
        collinear[3] = V::new(20.0, 20.0, 0.1, 0.0);
        assert!(dr_off.pass(&collinear));
        assert!(!dr_on.pass(&collinear));
    }

    /// The legs `cut_decays = F` leaves uncut are the products of a forced line in
    /// every set.
    #[test]
    fn decay_products_are_those_of_every_set() {
        let paired = forced(vec![
            vec![z_line(0b0011), z_line(0b1100)],
            vec![z_line(0b1001), z_line(0b0110)],
        ]);
        assert_eq!(paired.decay_products(), 0b1111);
        let partial = forced(vec![vec![z_line(0b0011)], vec![z_line(0b0110)]]);
        assert_eq!(partial.decay_products(), 0b0010);
        assert_eq!(ForcedResonances::none().decay_products(), 0);
    }

    /// Each outermost forced line holds its system above `M − bwcutoff·Γ`, so their
    /// sum bounds `√ŝ` from below, as the pole masses of the undecayed legs do.
    #[test]
    fn forced_lines_bound_shat() {
        let rc = card("0 = ptl\n0 = drll\n");
        let lines = forced(vec![vec![z_line(0b0011), z_line(0b1100)]]);
        let cuts = Cuts::compile_with(&rc, &four_lepton_legs(), &lines).unwrap();
        let expected = 2.0 * (MZ - 15.0 * WZ);
        assert!((cuts.shat_min() - expected * expected).abs() < 1e-9 * expected * expected);
        let unforced = Cuts::compile(&rc, &four_lepton_legs()).unwrap();
        assert_eq!(unforced.shat_min(), 0.0);
        assert!(!unforced.has_windows() && cuts.has_windows());
    }

    /// A forced line carrying the whole final state holds `√ŝ` below
    /// `M + bwcutoff·Γ`, with `Γ` floored at `M·small_width_treatment`; a line
    /// carrying only part of it, a set without such a line, and a process without
    /// forced lines leave ŝ unbounded.
    #[test]
    fn a_whole_state_forced_line_bounds_shat_from_above() {
        let rc = card("0 = ptl\n0 = drll\n");
        let whole = forced(vec![vec![z_line(0b0011), z_line(0b1111)]]);
        let cuts = Cuts::compile_with(&rc, &four_lepton_legs(), &whole).unwrap();
        let ceiling = MZ + 15.0 * WZ;
        assert!((cuts.shat_max() - ceiling * ceiling).abs() < 1e-9 * ceiling * ceiling);
        assert!(cuts.shat_min() < cuts.shat_max());

        let narrow = forced(vec![vec![ForcedLine {
            slots: 0b1111,
            mass: 3500.0,
            width: 1e-3,
        }]]);
        let narrow = Cuts::compile_with(&rc, &four_lepton_legs(), &narrow).unwrap();
        let ceiling = 3500.0 + 15.0 * 3500.0 * SMALL_WIDTH_TREATMENT;
        assert!((narrow.shat_max() - ceiling * ceiling).abs() < 1e-9 * ceiling * ceiling);

        let partial = forced(vec![vec![z_line(0b0011), z_line(0b1100)]]);
        let either = forced(vec![vec![z_line(0b1111)], vec![z_line(0b0011)]]);
        for lines in [partial, either, ForcedResonances::none()] {
            let cuts = Cuts::compile_with(&rc, &four_lepton_legs(), &lines).unwrap();
            assert_eq!(cuts.shat_max(), f64::INFINITY, "{lines:?}");
        }
    }

    /// The forced lines read off stitched diagrams: one set on a chain without
    /// identical particles across its decays, two on one with, and the nested `W`
    /// inside its top. The slots are the positions of the decay products in the
    /// subprocess's own final state, which is also the evaluator's leg order.
    #[test]
    fn forced_lines_of_stitched_diagrams() {
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use crate::ufo::sm::{sm_model, SMRestrict};
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let sets = |text: &str| {
            let card = parse_proc_card(text, &ParsingOptions::default()).unwrap();
            generate_from_proc_card(&card, &model).unwrap()
        };
        let slots_of = |names: &[String], wanted: &[&str]| {
            names
                .iter()
                .enumerate()
                .filter(|(_, n)| wanted.contains(&n.as_str()))
                .fold(0u64, |m, (k, _)| m | (1 << k))
        };

        let emu = sets("generate e+ e- > z z, z > e+ e-, z > mu+ mu-");
        assert_eq!(emu.len(), 1);
        let set = &emu[0];
        // An outgoing leg carries its crossed antiparticle (`Leg::particle`), so
        // the leg order is read against the conjugates.
        for d in &set.diagrams {
            let names: Vec<&str> = d.legs[2..]
                .iter()
                .map(|l| {
                    let crossed = crate::diagrams::diagram::antiparticle(&model, l.particle);
                    model.particle(crossed).name.as_str()
                })
                .collect();
            assert_eq!(
                names,
                set.particles_out
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
            );
        }
        let f = ForcedResonances::of(&set.diagrams, &evaluated);
        assert_eq!(f.patterns().len(), 1);
        let mut got: Vec<u64> = f.patterns()[0].iter().map(|l| l.slots).collect();
        got.sort();
        let mut want = vec![
            slots_of(&set.particles_out, &["e+", "e-"]),
            slots_of(&set.particles_out, &["mu+", "mu-"]),
        ];
        want.sort();
        assert_eq!(got, want);
        assert_eq!(f.decay_products(), 0b1111);

        let ee = sets("generate e+ e- > z z, z > e+ e-");
        let f = ForcedResonances::of(&ee[0].diagrams, &evaluated);
        assert_eq!(f.patterns().len(), 2);
        assert_eq!(f.decay_products(), 0b1111);

        let nested = sets("generate e+ e- > t t~, (t > w+ b, w+ > e+ ve), t~ > w- b~");
        let set = &nested[0];
        let f = ForcedResonances::of(&set.diagrams, &evaluated);
        assert_eq!(f.patterns().len(), 1);
        let lines = &f.patterns()[0];
        assert_eq!(lines.len(), 3);
        let w = slots_of(&set.particles_out, &["e+", "ve"]);
        let t = w | slots_of(&set.particles_out, &["b"]);
        assert!(lines
            .iter()
            .any(|l| l.slots == w && (l.mass - 80.419).abs() < 1.0));
        assert!(lines
            .iter()
            .any(|l| l.slots == t && (l.mass - 173.0).abs() < 1.0));
        assert_eq!(f.decay_products(), (1 << set.particles_out.len()) - 1);
    }

    /// The stitched diagrams of one proc-card line's single subprocess under
    /// the default SM, with its external legs.
    fn stitched_subprocess(text: &str) -> (ForcedResonances, Vec<ExternalLeg>) {
        stitched_subprocess_in(text, crate::ufo::sm::SMRestrict::Default)
    }

    /// [`stitched_subprocess`] under the SM restriction `restrict`.
    fn stitched_subprocess_in(
        text: &str,
        restrict: crate::ufo::sm::SMRestrict,
    ) -> (ForcedResonances, Vec<ExternalLeg>) {
        use crate::diagrams::diagram::antiparticle;
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use crate::ufo::sm::sm_model;
        let model = sm_model(restrict);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let card = parse_proc_card(text, &ParsingOptions::default()).unwrap();
        let sets = generate_from_proc_card(&card, &model).unwrap();
        assert_eq!(sets.len(), 1, "{text}");
        let diagrams = &sets[0].diagrams;
        let first = &diagrams[0];
        let legs = first
            .legs
            .iter()
            .enumerate()
            .map(|(i, l)| {
                if i < first.n_in {
                    let pdg = model.particle(l.particle).pdg_code as i32;
                    ExternalLeg::incoming(pdg, evaluated.mass(l.particle))
                } else {
                    let crossed = antiparticle(&model, l.particle);
                    let pdg = model.particle(crossed).pdg_code as i32;
                    ExternalLeg::outgoing(pdg, evaluated.mass(crossed))
                }
            })
            .collect();
        (ForcedResonances::of(diagrams, &evaluated), legs)
    }

    /// On `g g > z u u~, z > u u~` every quark is a core parton in some kept
    /// pairing, so exempting MadEvent's configuration-1 decay legs is refused,
    /// as is any single quark; the exemption `cut_decays = F` compiles to (the
    /// legs that are decay products in every pairing, none here) passes, and so
    /// does the non-empty one of the distinct-flavour chain.
    #[test]
    fn an_exemption_of_a_massless_coloured_core_leg_is_refused() {
        let (forced, legs) = stitched_subprocess("generate g g > z u u~, z > u u~");
        assert_eq!(forced.patterns().len(), 4);
        assert_eq!(forced.decay_products(), 0);
        assert_eq!(forced.massless_coloured_core, 0b1111);
        for mask in [0b0011u64, 0b1100, 0b0001, 0b1000] {
            let err = refuse_ir_unsafe_exemption(mask, &forced, &legs).unwrap_err();
            let CutError::IrUnsafeExemption { leg, pdg } = err else {
                panic!("{err}");
            };
            assert_eq!(leg, mask.trailing_zeros() as usize);
            assert_eq!(pdg.abs(), 2);
        }
        refuse_ir_unsafe_exemption(forced.decay_products(), &forced, &legs).unwrap();
        let rc = card("F = cut_decays\n20 = ptj\n5 = etaj\n0.4 = drjj\n");
        let cuts = Cuts::compile_with(&rc, &legs, &forced).unwrap();
        assert_eq!(cuts.spacelike_floor(), 400.0);

        let (forced, legs) = stitched_subprocess("generate g g > z u u~, z > d d~");
        assert_eq!(forced.patterns().len(), 1);
        let products = forced.decay_products();
        assert_eq!(products.count_ones(), 2);
        assert_eq!(forced.massless_coloured_core, 0b1111 & !products);
        Cuts::compile_with(&rc, &legs, &forced).unwrap();
        let core = forced.massless_coloured_core;
        assert!(refuse_ir_unsafe_exemption(products | core, &forced, &legs).is_err());
    }

    /// `u d > a u d` radiates the photon off a final-state quark, so at
    /// `draj = 0` with isolation off (`ptgmin = 0`) its `q → q γ` collinear
    /// limit is uncut and the card is refused; the MadGraph default `draj = 0.4`
    /// regulates it and compiles. A card that turns isolation on is refused
    /// earlier, as an unimplemented cut, not as this.
    ///
    /// A photon that meets no final-state quark at a vertex is not refused as
    /// this at `draj = 0`: with a gluon as the only parton (`u u~ > a g`), or
    /// beside quarks that come from a decay the photon is not part of
    /// (`u u~ > z a, z > d d~`).
    #[test]
    fn an_uncut_photon_quark_splitting_is_refused() {
        let (forced, legs) = stitched_subprocess("generate u d > a u d");
        let photon_pairs: Vec<MasslessLimit> = forced
            .massless_limits
            .iter()
            .copied()
            .filter(|l| matches!(l, MasslessLimit::FinalCollinear { a: 0, .. }))
            .collect();
        assert_eq!(
            photon_pairs,
            [
                MasslessLimit::FinalCollinear { a: 0, b: 1 },
                MasslessLimit::FinalCollinear { a: 0, b: 2 }
            ],
            "the photon meets both quarks"
        );
        let err = Cuts::compile_with(&card("0 = draj\n"), &legs, &forced).unwrap_err();
        let CutError::UncutPhotonSplitting {
            photon,
            parton,
            pdg,
        } = err
        else {
            panic!("refused for the wrong reason: {err}");
        };
        assert_eq!((photon, parton), (0, 1));
        assert_eq!(pdg, 2);
        eprintln!("u d > a u d at draj = 0: {err}");
        Cuts::compile_with(&card(""), &legs, &forced).expect("draj = 0.4 regulates it");
        Cuts::compile_with(&card("0.01 = draj\n"), &legs, &forced).expect("any draj > 0 does");
        let isolated = Cuts::compile_with(&card("0 = draj\n20 = ptgmin\n"), &legs, &forced);
        assert!(
            matches!(&isolated, Err(CutError::UnimplementedCutActive { name, .. }) if name == "ptgmin"),
            "{isolated:?}"
        );

        for text in ["generate u u~ > a g", "generate u u~ > z a, z > d d~"] {
            let (forced, legs) = stitched_subprocess(text);
            Cuts::compile_with(&card("0 = draj\n"), &legs, &forced)
                .unwrap_or_else(|e| panic!("{text}: {e}"));
        }
    }

    /// `u u~ > d d~ g` splits `g* → d d̄` and `q → q g` in the final state. At
    /// `drjj = 0` with the default `ptj = 20` the collinear limits are open,
    /// because a transverse-momentum cut does not keep two legs apart, and the
    /// card is refused naming the pair and its cuts; a `ΔR` or a mass minimum
    /// on the pair compiles, and so does `xqcut`, which reaches the filter as
    /// `mmjj = xqcut`. `g g > g g g` (`g → g g`) is refused the same way.
    #[test]
    fn an_uncut_final_state_parton_splitting_is_refused() {
        let (forced, legs) = stitched_subprocess("generate u u~ > d d~ g");
        let err = Cuts::compile_with(&card("0 = drjj\n"), &legs, &forced).unwrap_err();
        let CutError::UncutCollinearSplitting {
            a,
            b,
            pdg_a,
            pdg_b,
            ref regulator,
        } = err
        else {
            panic!("refused for the wrong reason: {err}");
        };
        assert_eq!((a, b, pdg_a, pdg_b), (0, 1, 1, -1), "{err}");
        assert_eq!(regulator, "drjj > 0 or mmjj > 0");
        eprintln!("u u~ > d d~ g at drjj = 0: {err}");
        for regulated in ["", "0.1 = drjj\n", "0 = drjj\n5 = mmjj\n", "30 = xqcut\n"] {
            Cuts::compile_with(&card(regulated), &legs, &forced)
                .unwrap_or_else(|e| panic!("{regulated:?}: {e}"));
        }

        let (forced, legs) = stitched_subprocess("generate g g > g g g");
        let err = Cuts::compile_with(&card("0 = drjj\n"), &legs, &forced).unwrap_err();
        assert!(
            matches!(
                err,
                CutError::UncutCollinearSplitting {
                    pdg_a: 21,
                    pdg_b: 21,
                    ..
                }
            ),
            "{err}"
        );
        Cuts::compile_with(&card(""), &legs, &forced).expect("drjj = 0.4 regulates g → g g");
    }

    /// A gluon off a quark line is soft-divergent at any angle: `u u~ > d d~ g`
    /// at `ptj = 0` keeps `drjj = 0.4` and `etaj = 5`, which regulate its
    /// collinear limits, and is refused for the soft one; an energy minimum
    /// regulates it. `u u~ > z g` (the gluon's only partner massive) is
    /// refused the same way.
    #[test]
    fn an_uncut_soft_gluon_is_refused() {
        for (text, leg) in [("generate u u~ > d d~ g", 2), ("generate u u~ > z g", 1)] {
            let (forced, legs) = stitched_subprocess(text);
            let err = Cuts::compile_with(&card("0 = ptj\n"), &legs, &forced).unwrap_err();
            let CutError::UncutSoftEmission {
                leg: got,
                pdg,
                ref regulator,
            } = err
            else {
                panic!("{text}: refused for the wrong reason: {err}");
            };
            assert_eq!((got, pdg), (leg, 21), "{text}");
            assert_eq!(regulator, "ptj > 0 or ej");
            eprintln!("{text} at ptj = 0: {err}");
            Cuts::compile_with(&card("0 = ptj\n1 = ej\n"), &legs, &forced)
                .unwrap_or_else(|e| panic!("{text}: {e}"));
        }
    }

    /// `g u > z u` emits the quark off the incoming gluon through a quark
    /// propagator: at `ptj = 0` and no rapidity cut (`etaj = -1`) its collinear
    /// limit with the beam is open and the card is refused. A rapidity maximum
    /// alone regulates it: at bounded rapidity the quark reaches the limit only
    /// by becoming soft, which a fermion does integrably.
    #[test]
    fn an_uncut_initial_state_splitting_is_refused() {
        let (forced, legs) = stitched_subprocess("generate g u > z u");
        let err = Cuts::compile_with(&card("0 = ptj\n-1 = etaj\n"), &legs, &forced).unwrap_err();
        let CutError::UncutInitialCollinear {
            beam,
            beam_pdg,
            leg,
            pdg,
            ref regulator,
        } = err
        else {
            panic!("refused for the wrong reason: {err}");
        };
        assert_eq!((beam, beam_pdg, leg, pdg), (0, 21, 1, 2), "{err}");
        assert_eq!(regulator, "ptj > 0 or a rapidity maximum etaj");
        eprintln!("g u > z u at ptj = 0, etaj = -1: {err}");
        Cuts::compile_with(&card("0 = ptj\n"), &legs, &forced).expect("etaj = 5 regulates it");
        Cuts::compile_with(&card("-1 = etaj\n"), &legs, &forced).expect("ptj = 20 regulates it");
    }

    /// The initial-state limit through an exchanged massless vector boson, the
    /// equivalent-photon limit, is refused like any other: `u d > e+ e- u d
    /// QCD=0` on the cuts of the electroweak `p p > e+ e- j j` card (`ptj = ptl =
    /// 0`, no rapidity or ΔR cut, `mmjj = 4`, `mmll = 50`) emits each outgoing
    /// quark off its beam through a t-channel photon, whose `1/t` pole no cut of
    /// that card keeps away. A rapidity maximum regulates it; with `mmll = 0` the
    /// card is refused first for its `γ* → e+ e-` splitting.
    #[test]
    fn the_equivalent_photon_limit_is_refused() {
        let ewk = |mmll: f64, etaj: f64| {
            format!(
                "0 = ptj\n0 = ptl\n{etaj} = etaj\n-1 = etal\n0 = drjj\n0 = drll\n0 = drjl\n\
                 4 = mmjj\n{mmll} = mmll\n5 = maxjetflavor\n"
            )
        };
        let (forced, legs) = stitched_subprocess("generate u d > e+ e- u d QCD=0");
        let err = Cuts::compile_with(&card(&ewk(50.0, -1.0)), &legs, &forced).unwrap_err();
        let CutError::UncutInitialCollinear {
            beam,
            beam_pdg,
            leg,
            pdg,
            ref regulator,
        } = err
        else {
            panic!("refused for the wrong reason: {err}");
        };
        assert_eq!((beam, beam_pdg, leg, pdg), (0, 2, 2, 2), "{err}");
        assert_eq!(regulator, "ptj > 0 or a rapidity maximum etaj");
        eprintln!("u d > e+ e- u d QCD=0 on the electroweak card: {err}");
        Cuts::compile_with(&card(&ewk(50.0, 5.0)), &legs, &forced)
            .expect("etaj = 5 regulates the beam-collinear limits");
        let err = Cuts::compile_with(&card(&ewk(0.0, 5.0)), &legs, &forced)
            .expect_err("γ* → e+ e- at mmll = drll = 0 is refused");
        assert!(
            matches!(
                err,
                CutError::UncutCollinearSplitting {
                    pdg_a: -11,
                    pdg_b: 11,
                    ..
                }
            ),
            "{err}"
        );
    }

    /// A `cut_decays = F` exemption of a massless colourless leg that is a core
    /// leg of a splitting is refused through the leg's own limits, not through
    /// [`refuse_ir_unsafe_exemption`], which reads coloured legs only. On
    /// `e+ e- > mu+ mu- a` with a synthetic forced line over the photon, the
    /// exempt photon loses `dral` and `pta`, which leaves the `μ → μ γ`
    /// collinear limit uncut; with one over the muon pair, the muons lose `drll`
    /// and `dral`, which leaves the `γ* → μ⁺ μ⁻` limit uncut first. Each refusal
    /// names `cut_decays`, and the same lines compile at `cut_decays = T`.
    #[test]
    fn an_exemption_of_a_massless_colourless_core_leg_is_refused() {
        let (mut forced, legs) = stitched_subprocess("generate e+ e- > mu+ mu- a");
        assert_eq!(forced.massless_coloured_core, 0);
        for (slots, pair) in [(0b100u64, (0, 2, -13, 22)), (0b011, (0, 1, -13, 13))] {
            forced.patterns = vec![vec![z_line(slots)]];
            assert_eq!(forced.decay_products(), slots);
            refuse_ir_unsafe_exemption(slots, &forced, &legs)
                .expect("the coloured-leg guard does not see a photon or a lepton");
            let err = Cuts::compile_with(&card("F = cut_decays\n"), &legs, &forced).unwrap_err();
            let CutError::UncutCollinearSplitting {
                a,
                b,
                pdg_a,
                pdg_b,
                ref regulator,
            } = err
            else {
                panic!("exempting {slots:#b}: refused for the wrong reason: {err}");
            };
            assert_eq!((a, b, pdg_a, pdg_b), pair, "{err}");
            assert!(regulator.starts_with("cut_decays = T"), "{err}");
            eprintln!("e+ e- > mu+ mu- a exempting {slots:#b}: {err}");
            Cuts::compile_with(&card("T = cut_decays\n"), &legs, &forced)
                .unwrap_or_else(|e| panic!("exempting nothing at cut_decays = T: {e}"));
        }
    }

    /// `e+ e- > mu+ mu- a` has massless muons in the default SM: at `dral = 0`
    /// its `μ → μ γ` collinear limit is open and the card is refused, and at
    /// `pta = 0` the photon's soft limit is; the defaults (`dral = 0.4`,
    /// `pta = 10`) regulate both.
    #[test]
    fn an_uncut_lepton_photon_splitting_is_refused() {
        let (forced, legs) = stitched_subprocess("generate e+ e- > mu+ mu- a");
        let err = Cuts::compile_with(&card("0 = dral\n"), &legs, &forced).unwrap_err();
        let CutError::UncutCollinearSplitting {
            a,
            b,
            pdg_a,
            pdg_b,
            ref regulator,
        } = err
        else {
            panic!("refused for the wrong reason: {err}");
        };
        assert_eq!((a, b, pdg_a, pdg_b), (0, 2, -13, 22), "{err}");
        assert_eq!(regulator, "dral > 0");
        eprintln!("e+ e- > mu+ mu- a at dral = 0: {err}");
        let err = Cuts::compile_with(&card("0 = pta\n"), &legs, &forced).unwrap_err();
        assert!(
            matches!(
                err,
                CutError::UncutSoftEmission {
                    leg: 2,
                    pdg: 22,
                    ..
                }
            ),
            "{err}"
        );
        Cuts::compile_with(&card(""), &legs, &forced).expect("the defaults regulate both");
    }

    /// Under `sm-no_b_mass` with `maxjetflavor = 4` the massless b is in the b
    /// class, whose separations default to zero: `g g > b b~ g` is refused for
    /// its `g* → b b̄` splitting, naming the b-class cuts. At
    /// `maxjetflavor = 5` the b is a jet and the jet defaults regulate it; with
    /// a massive b (the default SM) there is no collinear limit to cut.
    #[test]
    fn an_uncut_splitting_above_maxjetflavor_is_refused() {
        use crate::ufo::sm::SMRestrict;
        let (forced, legs) = stitched_subprocess_in("generate g g > b b~ g", SMRestrict::NoBMass);
        let err = Cuts::compile_with(&card(""), &legs, &forced).unwrap_err();
        let CutError::UncutCollinearSplitting {
            pdg_a,
            pdg_b,
            ref regulator,
            ..
        } = err
        else {
            panic!("refused for the wrong reason: {err}");
        };
        assert_eq!((pdg_a, pdg_b), (5, -5), "{err}");
        assert_eq!(regulator, "drbb > 0 or mmbb > 0");
        eprintln!("g g > b b~ g, massless b, maxjetflavor = 4: {err}");
        Cuts::compile_with(&card("5 = maxjetflavor\n"), &legs, &forced)
            .expect("the b is a jet at maxjetflavor = 5");
        let (forced, legs) = stitched_subprocess("generate g g > b b~ g");
        Cuts::compile_with(&card(""), &legs, &forced).expect("a massive b has no collinear limit");
    }

    // ── HT ──────────────────────────────────────────────────────────────

    // The HT tests place massless legs with `lep` at azimuth 0, π/2 or π, where
    // the computed `p_T` is `pt` to the bit, so a sum can sit exactly on a
    // threshold.
    use std::f64::consts::{FRAC_PI_2, PI};

    fn legs(incoming: [i32; 2], outgoing: &[i32]) -> Vec<ExternalLeg> {
        incoming
            .iter()
            .map(|&p| ExternalLeg::incoming(p, 0.0))
            .chain(outgoing.iter().map(|&p| ExternalLeg::outgoing(p, 0.0)))
            .collect()
    }

    fn with_beams(finals: &[V]) -> Vec<V> {
        let (b1, b2) = beams(3000.0);
        [b1, b2].into_iter().chain(finals.iter().copied()).collect()
    }

    /// `htj` is a scalar sum: two jets of `p_T = 150` at right angles sum to 300,
    /// where their vector sum would be 212. The minimum passes at equality and
    /// the maximum at equality (`cuts.f:892`'s `<` and `>`).
    #[test]
    fn htj_is_the_scalar_jet_sum_with_inclusive_edges() {
        let jj = legs([21, 21], &[21, 21]);
        let point = |a: f64, b: f64| with_beams(&[lep(a, -1.0, 0.0), lep(b, 1.0, FRAC_PI_2)]);
        let min = Cuts::compile(&card("300 = htjmin\n"), &jj).unwrap();
        assert!(min.pass(&point(150.0, 150.0)));
        assert!(!min.pass(&point(150.0, 149.99)));
        assert!(min.pass(&point(150.0, 150.01)));
        let max = Cuts::compile(&card("300 = htjmax\n"), &jj).unwrap();
        assert!(max.pass(&point(150.0, 150.0)));
        assert!(max.pass(&point(150.0, 149.99)));
        assert!(!max.pass(&point(150.0, 150.01)));
    }

    /// The same edges for the inclusive sum, here over jets alone.
    #[test]
    fn iht_has_the_same_edges() {
        let jj = legs([21, 21], &[21, 21]);
        let point = |a: f64, b: f64| with_beams(&[lep(a, -1.0, 0.0), lep(b, 1.0, FRAC_PI_2)]);
        let min = Cuts::compile(&card("300 = ihtmin\n"), &jj).unwrap();
        assert!(min.pass(&point(150.0, 150.0)));
        assert!(!min.pass(&point(150.0, 149.99)));
        let max = Cuts::compile(&card("300 = ihtmax\n"), &jj).unwrap();
        assert!(max.pass(&point(150.0, 150.0)));
        assert!(!max.pass(&point(150.0, 150.01)));
    }

    /// `g g > g b b~`, each leg at `p_T = 100`. At `maxjetflavor = 4` the b quarks
    /// are b-class: `htj = 100` and `iht = 300`, so they count for the inclusive
    /// sum and not for the jet sum. At `maxjetflavor = 5` they are jets and both
    /// sums are 300.
    #[test]
    fn b_quarks_count_for_iht_and_for_htj_only_as_jets() {
        let gbb = legs([21, 21], &[21, 5, -5]);
        let point = |pt_g: f64, pt_b: f64| {
            with_beams(&[
                lep(pt_g, 0.0, 0.0),
                lep(pt_b, -1.5, 0.0),
                lep(pt_b, 1.5, 0.0),
            ])
        };
        let at = |mjf: i32, cut: &str| {
            Cuts::compile(&card(&format!("{mjf} = maxjetflavor\n{cut}\n")), &gbb).unwrap()
        };

        assert!(at(4, "100 = htjmin").pass(&point(100.0, 100.0)));
        assert!(!at(4, "100.01 = htjmin").pass(&point(100.0, 100.0)));
        assert!(!at(4, "300 = htjmin").pass(&point(100.0, 100.0)));
        assert!(at(4, "300 = ihtmin").pass(&point(100.0, 100.0)));
        assert!(!at(4, "300 = ihtmin").pass(&point(100.0, 99.99)));
        assert!(at(4, "100 = htjmax").pass(&point(100.0, 400.0)));
        assert!(!at(4, "300 = ihtmax").pass(&point(100.0, 100.01)));

        assert!(at(5, "300 = htjmin").pass(&point(100.0, 100.0)));
        assert!(!at(5, "300 = htjmin").pass(&point(100.0, 99.99)));
        assert!(!at(5, "100 = htjmax").pass(&point(100.0, 100.0)));
        assert!(at(5, "300 = ihtmin").pass(&point(100.0, 100.0)));
    }

    /// `maxjetflavor = 3` files the charm quark under the b class too, so it
    /// leaves the jet sum and stays in the inclusive one.
    #[test]
    fn charm_leaves_htj_below_maxjetflavor_four() {
        let gcc = legs([21, 21], &[4, -4]);
        let point = with_beams(&[lep(150.0, -1.0, 0.0), lep(150.0, 1.0, 0.0)]);
        let at = |mjf: i32, cut: &str| {
            Cuts::compile(&card(&format!("{mjf} = maxjetflavor\n{cut}\n")), &gcc).unwrap()
        };
        assert!(at(4, "300 = htjmin").pass(&point));
        assert!(!at(3, "300 = htjmin").pass(&point));
        assert!(at(3, "300 = ihtmin").pass(&point));
    }

    /// Leptons and photons are in neither sum: the inclusive HT is over jets and
    /// b's, not over every visible leg.
    #[test]
    fn leptons_and_photons_are_in_neither_sum() {
        let lla = legs([2, -2], &[11, -11, 22, 21]);
        let point = with_beams(&[
            lep(200.0, -1.0, 0.0),
            lep(200.0, 1.0, FRAC_PI_2),
            lep(200.0, 2.0, 0.0),
            lep(50.0, 0.0, 0.0),
        ]);
        assert!(Cuts::compile(&card("50 = ihtmin\n"), &lla)
            .unwrap()
            .pass(&point));
        assert!(!Cuts::compile(&card("50.01 = ihtmin\n"), &lla)
            .unwrap()
            .pass(&point));
        assert!(!Cuts::compile(&card("50.01 = htjmin\n"), &lla)
            .unwrap()
            .pass(&point));
    }

    /// Without a jet the jet sum is zero: a positive `htjmin` rejects every point
    /// (`cuts.f:817`), a maximum rejects none, and the inclusive minimum still
    /// applies (`cuts.f:908` sits outside the `njets > 0` branch).
    #[test]
    fn a_process_without_jets_fails_every_positive_minimum() {
        let point = dy_momenta(lep(40.0, 0.5, 0.0), lep(40.0, -0.5, 3.0));
        let dy = |cut: &str| Cuts::compile(&card(&format!("{cut}\n")), &dy_legs()).unwrap();
        assert!(dy("").pass(&point));
        assert!(!dy("1e-9 = htjmin").pass(&point));
        assert!(!dy("1e-9 = ihtmin").pass(&point));
        assert!(dy("0 = htjmax").pass(&point));
        assert!(dy("0 = ihtmax").pass(&point));
        assert!(dy("-5 = htjmin").pass(&point));
    }

    /// The sums read the leg's class and not its `do_cuts` flag (`cuts.f:524-534`):
    /// the two jets of a forced `W` under `cut_decays = F` take no `ptj` cut, yet
    /// count for HT.
    #[test]
    fn decay_products_count_for_ht() {
        let udx = legs([2, -1], &[2, -1]);
        let forced = ForcedResonances {
            patterns: vec![vec![ForcedLine {
                slots: 0b11,
                mass: 80.4,
                width: 2.0,
            }]],
            massless_coloured_core: 0,
            massless_limits: Vec::new(),
        };
        // Back to back at y = 0, so m(jj) = 80.4 sits on the pole.
        let half = 40.2;
        let point = with_beams(&[lep(half, 0.0, 0.0), lep(half, 0.0, PI)]);
        let compile = |cut: &str| {
            Cuts::compile_with(
                &card(&format!("F = cut_decays\n50 = ptj\n{cut}\n")),
                &udx,
                &forced,
            )
            .unwrap()
        };
        assert!(compile("").pass(&point), "ptj is off on the decay products");
        assert!(compile("80 = htjmin").pass(&point));
        assert!(!compile("81 = htjmin").pass(&point));
        assert!(!compile("81 = ihtmin").pass(&point));
    }

    /// A jet-class leg above 20 GeV gets no single-leg cut and still counts:
    /// a top quark at `maxjetflavor = 6`.
    #[test]
    fn a_heavy_jet_class_leg_counts_for_ht() {
        let tt = vec![
            ExternalLeg::incoming(21, 0.0),
            ExternalLeg::incoming(21, 0.0),
            ExternalLeg::outgoing(6, 173.0),
            ExternalLeg::outgoing(-6, 173.0),
        ];
        let top = |px: f64| {
            let e = (px * px + 173.0f64 * 173.0).sqrt();
            V::new(e, px, 0.0, 0.0)
        };
        let point = with_beams(&[top(150.0), top(-150.0)]);
        let at = |cut: &str| {
            Cuts::compile(&card(&format!("6 = maxjetflavor\n200 = ptj\n{cut}\n")), &tt).unwrap()
        };
        assert!(at("").pass(&point), "ptj is off on a leg above 20 GeV");
        assert!(at("300 = htjmin").pass(&point));
        assert!(!at("300.01 = htjmin").pass(&point));
    }

    /// The defaults, and any minimum at or below zero with any negative maximum,
    /// compile no HT cut: the filter is the one a card without the four names
    /// compiles to.
    #[test]
    fn ht_defaults_compile_no_cut() {
        let jj = legs([21, 21], &[21, 21]);
        let bare = Cuts::compile(&card(""), &jj).unwrap();
        assert!(bare.ht.is_none());
        for off in [
            "0 = htjmin\n-1 = htjmax\n0 = ihtmin\n-1 = ihtmax\n",
            "-10 = htjmin\n-0.5 = htjmax\n-3 = ihtmin\n-2 = ihtmax\n",
        ] {
            assert_eq!(Cuts::compile(&card(off), &jj).unwrap(), bare, "{off}");
        }
        for on in ["1 = htjmin", "0 = htjmax", "1 = ihtmin", "0 = ihtmax"] {
            assert!(Cuts::compile(&card(&format!("{on}\n")), &jj)
                .unwrap()
                .ht
                .is_some());
        }
    }

    /// The four HT names are compiled, not refused; the partial sums `ht2min`..
    /// `ht4max` still are.
    #[test]
    fn ht_minima_and_maxima_are_not_refused() {
        for on in [
            "300 = htjmin",
            "900 = htjmax",
            "300 = ihtmin",
            "900 = ihtmax",
        ] {
            assert!(
                detect_unimplemented(&lhc(&format!("{on}\n"))).is_ok(),
                "{on}"
            );
        }
        for name in ["ht2min", "ht3min", "ht4min", "ht2max", "ht3max", "ht4max"] {
            assert!(
                detect_unimplemented(&lhc(&format!("100 = {name}\n"))).is_err(),
                "{name}"
            );
        }
    }

    /// `√ŝ ≥ Σ p_T`, so the larger HT minimum bounds ŝ, and the bound is reached:
    /// two jets back to back at `y = 0` with `htj = htjmin` have `ŝ = htjmin²`. A
    /// minimum with no leg to sum over contributes nothing.
    #[test]
    fn ht_minima_bound_shat() {
        let jj = legs([21, 21], &[21, 21]);
        let cuts = Cuts::compile(&card("300 = htjmin\n400 = ihtmin\n"), &jj).unwrap();
        assert_eq!(cuts.shat_min(), 400.0 * 400.0);
        let tight = Cuts::compile(&card("300 = htjmin\n"), &jj).unwrap();
        assert_eq!(tight.shat_min(), 300.0 * 300.0);
        let finals = [lep(150.0, 0.0, 0.0), lep(150.0, 0.0, PI)];
        assert!(tight.pass(&with_beams(&finals)));
        assert_eq!((finals[0] + finals[1]).m2(), 300.0 * 300.0);

        let bb = legs([21, 21], &[5, -5]);
        let only_b = Cuts::compile(&card("300 = htjmin\n400 = ihtmin\n"), &bb).unwrap();
        assert_eq!(only_b.shat_min(), 400.0 * 400.0);
        let none = Cuts::compile(&card("300 = htjmin\n400 = ihtmin\n"), &dy_legs()).unwrap();
        let bare = Cuts::compile(&card(""), &dy_legs()).unwrap();
        assert_eq!(none.shat_min(), bare.shat_min());
    }

    // ── xpt* at or below zero ─────────────────────────────────────────────

    /// Final states that give each `xpt*` class a leg: jets, b quarks, a
    /// photon and leptons.
    fn xpt_leg_sets() -> Vec<Vec<ExternalLeg>> {
        vec![
            legs([21, 21], &[21, 21]),
            legs([21, 21], &[5, -5]),
            legs([2, -2], &[22, 21]),
            dy_legs(),
        ]
    }

    /// `cuts.f` reads each `xpt*` behind `> 0` and `setcuts.f` through
    /// `max(·, 0)`, so a value at or below zero is the default: it passes
    /// detection and compiles to the default's filter on every class.
    #[test]
    fn a_nonpositive_xpt_compiles_to_the_default_cuts() {
        for name in NONPOSITIVE_IS_OFF {
            for value in ["-1", "0", "-0.5", "-100"] {
                let rc = lhc(&format!("{value} = {name}\n"));
                assert!(detect_unimplemented(&rc).is_ok(), "{name} = {value}");
                for set in xpt_leg_sets() {
                    let compiled = Cuts::compile(&rc, &set).unwrap();
                    let default = Cuts::compile(&lhc(""), &set).unwrap();
                    assert_eq!(compiled, default, "{name} = {value}");
                }
            }
        }
    }

    #[test]
    fn a_positive_xpt_is_refused() {
        for name in NONPOSITIVE_IS_OFF {
            for value in ["1e-9", "10", "1e5"] {
                match Cuts::compile(&lhc(&format!("{value} = {name}\n")), &dy_legs()) {
                    Err(CutError::UnimplementedCutActive { name: n, .. }) => assert_eq!(n, *name),
                    other => panic!("{name} = {value}: {other:?}"),
                }
            }
        }
    }

    // ── pt_min_pdg ─────────────────────────────────────────────────────────

    /// `g u > z u`, the Z at its pole mass.
    fn zj_legs() -> Vec<ExternalLeg> {
        vec![
            ExternalLeg::incoming(21, 0.0),
            ExternalLeg::incoming(2, 0.0),
            ExternalLeg::outgoing(23, MZ),
            ExternalLeg::outgoing(2, 0.0),
        ]
    }

    /// A leg of mass `m` at transverse momentum `pt`, rapidity `y`, azimuth `phi`.
    fn massive(m: f64, pt: f64, y: f64, phi: f64) -> V {
        let mt = (m * m + pt * pt).sqrt();
        V::new(mt * y.cosh(), pt * phi.cos(), pt * phi.sin(), mt * y.sinh())
    }

    /// A Z and a jet balanced in the transverse plane, the Z at `pt` and a
    /// rapidity no class cut would accept.
    fn zj_point(pt: f64) -> Vec<V> {
        with_beams(&[massive(MZ, pt, 6.0, 0.0), lep(pt, 0.0, PI)])
    }

    /// The Z is above 20 GeV, so no class cut reaches it, and the per-PDG minimum
    /// does: `pT < threshold` fails (`cuts.f:333`), the threshold itself passes,
    /// and no rapidity or energy cut comes with it.
    #[test]
    fn pt_min_pdg_cuts_a_listed_heavy_leg() {
        let bare = Cuts::compile(&lhc(""), &zj_legs()).unwrap();
        let cut = Cuts::compile(&lhc("{23: 100} = pt_min_pdg\n"), &zj_legs()).unwrap();
        assert_ne!(bare, cut);
        for pt in [20.0, 99.9, 100.0, 100.1, 400.0] {
            assert!(bare.pass(&zj_point(pt)), "unlisted at {pt}");
            assert_eq!(cut.pass(&zj_point(pt)), pt >= 100.0, "listed at {pt}");
        }
        // The jet keeps its class cuts: |y| = 6 fails etaj = 5.
        let far_jet = with_beams(&[massive(MZ, 150.0, 0.0, 0.0), lep(150.0, 6.0, PI)]);
        assert!(!cut.pass(&far_jet));
    }

    /// One threshold per `|PDG|`: an antiparticle is cut like its particle, and
    /// a code the process does not carry compiles to the default filter.
    #[test]
    fn pt_min_pdg_reads_the_absolute_code() {
        let w = |pdg: i32| {
            vec![
                ExternalLeg::incoming(1, 0.0),
                ExternalLeg::incoming(-2, 0.0),
                ExternalLeg::outgoing(pdg, 80.4),
                ExternalLeg::outgoing(21, 0.0),
            ]
        };
        let rc = lhc("{24: 50} = pt_min_pdg\n");
        let point = |pt: f64| with_beams(&[massive(80.4, pt, 0.0, 0.0), lep(pt, 0.0, PI)]);
        for pdg in [24, -24] {
            let cuts = Cuts::compile(&rc, &w(pdg)).unwrap();
            assert!(!cuts.pass(&point(49.0)), "{pdg}");
            assert!(cuts.pass(&point(51.0)), "{pdg}");
        }
        for set in [dy_legs(), zj_legs(), legs([21, 21], &[21, 21])] {
            assert_eq!(
                Cuts::compile(&rc, &set).unwrap(),
                Cuts::compile(&lhc(""), &set).unwrap()
            );
        }
    }

    /// `setcuts.f:318-334` overwrites every single-leg threshold of a listed
    /// leg, ignoring `do_cuts` but for `cut_decays`: a neutrino, uncut by
    /// class, gets the minimum and nothing else.
    #[test]
    fn pt_min_pdg_reaches_a_neutrino() {
        let set = legs([2, -2], &[12, -12]);
        let cuts = Cuts::compile(&lhc("{12: 30} = pt_min_pdg\n"), &set).unwrap();
        let point = |pt: f64, y: f64| with_beams(&[lep(pt, y, 0.0), lep(pt, -y, PI)]);
        assert!(!cuts.pass(&point(29.0, 0.0)));
        assert!(cuts.pass(&point(31.0, 0.0)));
        assert!(cuts.pass(&point(31.0, 8.0)));
    }

    /// A decay product is exempt under `cut_decays = F` and cut under
    /// `cut_decays = T`, though above 20 GeV it has no class cut either way.
    #[test]
    fn pt_min_pdg_follows_cut_decays() {
        let set = legs([21, 21], &[23, 23]);
        let parent = forced(vec![vec![ForcedLine {
            slots: 0b11,
            mass: 500.0,
            width: 0.0,
        }]]);
        let point = |pt: f64| with_beams(&[massive(MZ, pt, 0.0, 0.0), massive(MZ, pt, 0.0, PI)]);
        let exempt = Cuts::compile_with(
            &lhc("{23: 100} = pt_min_pdg\nF = cut_decays\n"),
            &set,
            &parent,
        )
        .unwrap();
        assert!(exempt.pass(&point(50.0)));
        let cut = Cuts::compile_with(
            &lhc("{23: 100} = pt_min_pdg\nT = cut_decays\n"),
            &set,
            &parent,
        )
        .unwrap();
        assert!(!cut.pass(&point(50.0)));
        assert!(cut.pass(&point(150.0)));
        let undecayed =
            Cuts::compile(&lhc("{23: 100} = pt_min_pdg\nF = cut_decays\n"), &set).unwrap();
        assert!(!undecayed.pass(&point(50.0)));
    }

    /// The ŝ floor counts the per-PDG minimum with the class cuts, `√ŝ ≥ Σ pT`,
    /// and stays below the least ŝ an accepted point can have: a Z at
    /// `pT = 100` recoiling against a jet needs `√ŝ ≥ m_T(Z) + 100`.
    #[test]
    fn pt_min_pdg_enters_the_shat_floor_as_a_bound() {
        let cuts = Cuts::compile(&lhc("{23: 100} = pt_min_pdg\n"), &zj_legs()).unwrap();
        assert_eq!(cuts.shat_min(), (100.0f64 + 20.0).powi(2));
        let at_threshold = zj_point(100.0);
        let shat = (at_threshold[2] + at_threshold[3]).m2();
        assert!(cuts.pass(&at_threshold));
        assert!(cuts.shat_min() < shat);
        assert!(cuts.spacelike_floor() == 100.0 * 100.0);
    }

    /// The `setcuts.f:681` overcount needs three listed codes and a listed
    /// outgoing leg with `do_cuts`; anything short of both compiles.
    #[test]
    fn pt_min_pdg_refuses_the_cards_madevent_overcounts_smin_on() {
        let light = |mass: f64| {
            vec![
                ExternalLeg::incoming(21, 0.0),
                ExternalLeg::incoming(21, 0.0),
                ExternalLeg::outgoing(1023, mass),
                ExternalLeg::outgoing(21, 0.0),
                ExternalLeg::outgoing(21, 0.0),
            ]
        };
        let three = lhc("{1023: 100, 23: 0, 24: 0} = pt_min_pdg\n");
        let two = lhc("{1023: 100, 23: 0} = pt_min_pdg\n");
        assert!(matches!(
            Cuts::compile(&three, &light(1.0)),
            Err(CutError::PdgCutRefused { .. })
        ));
        assert!(Cuts::compile(&two, &light(1.0)).is_ok());
        assert!(Cuts::compile(&three, &light(30.0)).is_ok());
        assert!(Cuts::compile(&three, &zj_legs()).is_ok());
        assert!(detect_unimplemented(&three).is_ok());
        let decayed = forced(vec![vec![ForcedLine {
            slots: 0b001,
            mass: 1.0,
            width: 0.0,
        }]]);
        let uncut = lhc("{1023: 100, 23: 0, 24: 0} = pt_min_pdg\nF = cut_decays\n");
        assert!(Cuts::compile_with(&uncut, &light(1.0), &decayed).is_ok());
    }

    #[test]
    fn pt_min_pdg_spellings_read_as_banner_py_reads_them() {
        let threshold = |text: &str| {
            let rc = lhc(&format!("{text} = pt_min_pdg\n"));
            PdgPtMin::from_card(&rc).unwrap().entries
        };
        assert_eq!(threshold("{23: 100}"), vec![(23, 100.0)]);
        assert_eq!(threshold("{'23': 100.0}"), vec![(23, 100.0)]);
        assert_eq!(
            threshold("{23: 100, 1023: 50.5,}"),
            vec![(23, 100.0), (1023, 50.5)]
        );
        assert_eq!(threshold("{23: 10, 23: 20}"), vec![(23, 20.0)]);
        assert_eq!(threshold("23 : 100"), vec![(23, 100.0)]);
        assert_eq!(threshold("{}"), vec![]);
        assert_eq!(PdgPtMin::from_card(&lhc("")).unwrap().entries, vec![]);
    }

    /// MadGraph's refusals: a negative code, more than 25 codes, and the light
    /// codes `banner.py:4722` means to refuse; plus code 0 and a malformed entry.
    #[test]
    fn pt_min_pdg_refusals() {
        let mut refused = vec![
            "{-23: 100}".to_string(),
            "{0: 10}".to_string(),
            "{z: 10}".to_string(),
            "{23: many}".to_string(),
        ];
        refused.extend(PDG_CUT_LIGHT.iter().map(|p| format!("{{{p}: 10}}")));
        refused.push(format!("{{23: 100, {}: 10}}", PDG_CUT_LIGHT[0]));
        let many: Vec<String> = (0..26).map(|k| format!("{}: 10", 9000001 + k)).collect();
        refused.push(format!("{{{}}}", many.join(", ")));
        for text in &refused {
            let rc = lhc(&format!("{text} = pt_min_pdg\n"));
            assert!(
                matches!(
                    detect_unimplemented(&rc),
                    Err(CutError::PdgCutRefused { .. })
                ),
                "{text}"
            );
            assert!(
                matches!(
                    Cuts::compile(&rc, &zj_legs()),
                    Err(CutError::PdgCutRefused { .. })
                ),
                "{text}"
            );
        }
        let at_cap: Vec<String> = (0..25).map(|k| format!("{}: 10", 9000001 + k)).collect();
        let rc = lhc(&format!("{{{}}} = pt_min_pdg\n", at_cap.join(", ")));
        assert!(detect_unimplemented(&rc).is_ok());
        for allowed in ["{6: 10}", "{12: 10}", "{16: 10}", "{1023: 100}", "{25: 0}"] {
            let rc = lhc(&format!("{allowed} = pt_min_pdg\n"));
            assert!(detect_unimplemented(&rc).is_ok(), "{allowed}");
        }
    }
}
