//! Colour-line tags per colour flow, in the Les Houches `ICOLUP` convention.
//!
//! Each element of a [`ColorBasis`] is keyed by a simplified colour structure —
//! a product of `T(a…, i, j)` chains and `Tr(a…)` traces over the *external*
//! colour indices. Read in the double-line (colour-flow) picture that key is
//! literally the flow's set of colour lines, so the LHE tag pair per leg is
//! derived from it rather than transcribed from a reference table.
//!
//! ## Reading a basis key as colour lines
//!
//! Every endpoint of a line is a `(leg, SU(3) index rep)` pair: a (anti)quark
//! leg carries one index, a gluon leg carries both a **3** and a **3̄**. The
//! chains contract them as
//!
//! - `T([a₁…aₙ], i, j)`: `(leg i, 3) — (a₁, 3̄)`, then `(a_k, 3) — (a_{k+1}, 3̄)`
//!   along the chain, then `(aₙ, 3) — (leg j, 3̄)`. With an empty adjoint list
//!   this is the single line `(leg i, 3) — (leg j, 3̄)` of a δ.
//! - `Tr([a₁…aₙ])`: the same links closed cyclically, `(aₙ, 3) — (a₁, 3̄)`.
//!
//! ## Amplitude index rep vs. physical colour
//!
//! The colour structure treats every leg as outgoing, so a leg's index rep is
//! the particle's own rep when the leg is outgoing and the conjugate rep when it
//! is incoming. `ICOLUP` slot 1 is the *physical* colour and slot 2 the physical
//! anticolour, which makes the endpoint → slot map depend on the leg's
//! direction: a **3** index lands in the colour slot for an outgoing leg and in
//! the anticolour slot for an incoming one, and vice versa for a **3̄** index.
//!
//! That crossing rule is not assumed: [`color_flow_tags`] checks, for every
//! flow, that the slots the derived lines occupy are exactly the slots the leg's
//! particle rep allows (a triplet fills only the colour slot, an antitriplet
//! only the anticolour slot, an octet both, a singlet neither), each exactly
//! once. Flipping the rule puts an incoming quark's line in its anticolour slot
//! and the check fails.
//!
//! ## Line labels
//!
//! Labels start at 501 (MadGraph's pool) and are handed out in the order the
//! lines are derived. Only the induced *connectivity* is physical — any
//! consistent relabelling describes the same event.

use super::colorize::ColorBasis;
use super::factor::ImmutableString;
use super::tensor::{ColorAlgebraError, Idx, TensorKind};
use crate::helas::repr::color::ColorRep;
use crate::select::select_index;

/// The label given to the first colour line of a flow, matching MadGraph's pool.
pub const FIRST_COLOR_LINE: u32 = 501;

/// The `ICOLUP` entry for "this leg has no line in this slot".
pub const NO_COLOR_LINE: u32 = 0;

/// `ICOLUP` slot index of the physical colour line.
const COLOR_SLOT: usize = 0;
/// `ICOLUP` slot index of the physical anticolour line.
const ANTICOLOR_SLOT: usize = 1;

/// One external leg as the flow table sees it: the colour rep its *particle*
/// carries, and whether the leg is incoming.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LegColor {
    /// The particle's own SU(3) rep (not the crossed, all-outgoing one).
    pub rep: ColorRep,
    /// Whether the leg is in the initial state.
    pub incoming: bool,
}

/// Which `ICOLUP` slots a leg of this colour rep occupies: `[colour, anticolour]`.
///
/// A triplet fills only the colour slot, an antitriplet only the anticolour slot,
/// an octet both and a singlet neither — the rule the derived lines are checked
/// against, and the same rule a written record can be scanned against without any
/// reference at all.
pub fn slots_for(rep: ColorRep) -> [bool; 2] {
    match rep {
        ColorRep::Singlet => [false, false],
        ColorRep::Triplet => [true, false],
        ColorRep::AntiTriplet => [false, true],
        ColorRep::Octet => [true, true],
    }
}

/// The SU(3) index rep an endpoint carries inside the colour structure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AmpRep {
    Three,
    ThreeBar,
}

impl AmpRep {
    /// The `ICOLUP` slot this endpoint's line occupies on `leg`.
    fn slot(self, leg: LegColor) -> usize {
        if (self == AmpRep::ThreeBar) != leg.incoming {
            ANTICOLOR_SLOT
        } else {
            COLOR_SLOT
        }
    }
}

/// The `(colour, anticolour)` line labels of every external leg, for every
/// colour flow of a subprocess.
///
/// Row-major over flows: flow `f`'s per-leg pairs are `tags[f]`, in the process's
/// external-leg order (incoming first). `0` means the leg has no line in that
/// slot.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ColorFlowTags {
    n_ext: usize,
    tags: Vec<[u32; 2]>,
}

impl ColorFlowTags {
    /// The number of colour flows (NCOLOR).
    pub fn n_flows(&self) -> usize {
        if self.n_ext == 0 {
            0
        } else {
            self.tags.len() / self.n_ext
        }
    }

    /// The number of external legs.
    pub fn n_ext(&self) -> usize {
        self.n_ext
    }

    /// Flow `f`'s `(colour, anticolour)` pair per external leg.
    pub fn flow(&self, f: usize) -> &[[u32; 2]] {
        &self.tags[f * self.n_ext..(f + 1) * self.n_ext]
    }

    /// The same flows read on reordered external legs: leg `i` of the result
    /// carries the tags of leg `order[i]` of `self`.
    ///
    /// Line labels travel with their legs, so every flow keeps the connectivity it
    /// had — this relabels which leg an endpoint sits on, it does not recolour
    /// anything. `None` unless `order` is a permutation of the legs.
    pub fn permuted(&self, order: &[usize]) -> Option<ColorFlowTags> {
        if order.len() != self.n_ext {
            return None;
        }
        let mut seen = vec![false; self.n_ext];
        for &leg in order {
            if std::mem::replace(seen.get_mut(leg)?, true) {
                return None;
            }
        }
        let mut tags = Vec::with_capacity(self.tags.len());
        for f in 0..self.n_flows() {
            let flow = self.flow(f);
            tags.extend(order.iter().map(|&leg| flow[leg]));
        }
        Some(ColorFlowTags {
            n_ext: self.n_ext,
            tags,
        })
    }

    /// The same flows under a new flow numbering: flow `f` of the result is flow
    /// `order[f]` of `self`.
    ///
    /// This renumbers the flows and touches nothing inside them, which is the
    /// opposite of [`permuted`](Self::permuted) — that keeps the numbering and moves
    /// the legs. It is what puts one subprocess's table into another's flow indexing,
    /// once the two bases have been paired up. `None` unless `order` is a permutation
    /// of the flows.
    pub fn reindexed(&self, order: &[usize]) -> Option<ColorFlowTags> {
        if order.len() != self.n_flows() {
            return None;
        }
        let mut seen = vec![false; self.n_flows()];
        for &f in order {
            if std::mem::replace(seen.get_mut(f)?, true) {
                return None;
            }
        }
        let mut tags = Vec::with_capacity(self.tags.len());
        for &f in order {
            tags.extend_from_slice(self.flow(f));
        }
        Some(ColorFlowTags {
            n_ext: self.n_ext,
            tags,
        })
    }

    /// The same flows read on legs carrying the conjugate colour reps: every leg's
    /// `[colour, anticolour]` pair exchanged, in every flow.
    ///
    /// Conjugating a basis key — `T(a₁…aₙ, i, j)* = T(aₙ…a₁, j, i)`, a trace
    /// reversed — flips the SU(3) index rep of every endpoint while leaving *which
    /// leg* each endpoint sits on and *which endpoints pair into a line* alone. A
    /// leg's `ICOLUP` slot is its endpoint's index rep crossed by the leg's
    /// direction, and the direction does not move, so every endpoint changes slot
    /// and nothing else changes. The exchange is therefore the whole of it.
    ///
    /// What this is for: a subprocess whose legs carry the conjugate reps of these
    /// shares its `|M|²`, its colour-factor matrix and its flow *count* with the one
    /// these tags were derived from, so an event of it can reuse this table — but
    /// only through this transformation, since the original's slots are the ones the
    /// original's reps allow.
    ///
    /// Note what the transformation is *not*: exchanging the slots only on the legs
    /// whose rep changed. That leaves a self-conjugate leg's endpoints where they
    /// were while its partners move, which breaks the lines it sits on while staying
    /// legal on every leg — so no slot-occupancy check can see it.
    ///
    /// **This relates a subprocess to its *full* conjugate and to nothing else.** A
    /// subprocess with only *some* legs conjugated — `u c~ > u c~` against
    /// `u c > u c` — is not related to this one by any slot operation at all:
    /// conjugating one end of a colour line re-routes it onto a different pair of
    /// legs, and exchanging slots can only move an endpoint between the two slots of
    /// the leg it already sits on. Such a subprocess's table has to come from its own
    /// colour basis.
    pub fn conjugated(&self) -> ColorFlowTags {
        ColorFlowTags {
            n_ext: self.n_ext,
            tags: self
                .tags
                .iter()
                .map(|&[colour, anticolour]| [anticolour, colour])
                .collect(),
        }
    }

    /// Check every flow's occupied slots against the reps `legs` carry.
    ///
    /// The same statement [`color_flow_tags`] makes as it derives a table, made
    /// again against a *different* leg list — the one an event record is about to
    /// be written on. A table carried from one subprocess to another passes only if
    /// the lines land in the slots the destination's own reps allow.
    pub fn check_legs(&self, legs: &[LegColor]) -> Result<(), ColorAlgebraError> {
        if legs.len() != self.n_ext {
            return Err(ColorAlgebraError::InconsistentColorFlow(format!(
                "{} colour-flow legs against {} reps",
                self.n_ext,
                legs.len()
            )));
        }
        for f in 0..self.n_flows() {
            let flow = self.flow(f);
            for (leg, info) in legs.iter().enumerate() {
                let got = [
                    flow[leg][COLOR_SLOT] != NO_COLOR_LINE,
                    flow[leg][ANTICOLOR_SLOT] != NO_COLOR_LINE,
                ];
                if got != slots_for(info.rep) {
                    return Err(ColorAlgebraError::InconsistentColorFlow(format!(
                        "flow {}: leg {} is a {:?} but its lines occupy \
                         (colour, anticolour) = {got:?}",
                        f + 1,
                        leg + 1,
                        info.rep
                    )));
                }
            }
        }
        Ok(())
    }

    /// The flow tags of a colour flow drawn with probability
    /// `JAMP2(i) / Σⱼ JAMP2(j)`; `None` when the weights carry no probability
    /// (all zero, negative, or non-finite).
    ///
    /// `u` is a uniform variate on `[0, 1)`.
    pub fn select(&self, jamp2: &[f64], u: f64) -> Option<&[[u32; 2]]> {
        select_flow(jamp2, u).map(|f| self.flow(f))
    }
}

/// Draw a colour-flow index with probability `JAMP2(i) / Σⱼ JAMP2(j)` from a
/// uniform variate `u ∈ [0, 1)`. `None` when the weights carry no probability.
///
/// This is a categorical draw off a diagonal accumulator; it selects a flow for
/// the event record and never enters the integrand, so it has no effect on the
/// cross section. It is the same draw the per-event helicity selection makes off
/// `|M_hel|²`, so both share one definition ([`select_index`]).
///
/// The unrestricted form. MadEvent's `SELECT_COLOR` runs it over the flows one
/// integration configuration admits ([`select_flow_reached_by`]) and falls back
/// to this when that mask carries no probability.
pub fn select_flow(jamp2: &[f64], u: f64) -> Option<usize> {
    select_index(jamp2, u)
}

/// The same draw restricted to the flows one diagram reaches at leading order in
/// `Nc` ([`LeadingColorFlows`]): weights stay `∝ JAMP2(i)`, but every flow the
/// diagram does not reach is given weight zero.
///
/// `reached` is that diagram's row; a row of the wrong length leaves the draw
/// unrestricted. When the reached flows carry no probability at this point —
/// none is reached, or all of their `JAMP2` vanish — the restriction is dropped
/// and the draw runs over every flow, which is what still labels an event whose
/// amplitude at this point is entirely subleading.
///
/// Both the mask and the fallback are MadEvent's `SELECT_COLOR`: it accumulates
/// `JAMP2` over the flows its `ICOLAMP` row admits, and re-accumulates over every
/// flow when that cumulant ends at zero.
///
/// The caller supplies the diagram; the event path picks it the way MadEvent
/// does, by drawing the integration configuration `∝ AMP2` (see
/// [`AmplitudeEvaluator::select_color_flow`](crate::helas::eval::AmplitudeEvaluator::select_color_flow),
/// which composes the two steps).
pub fn select_flow_reached_by(jamp2: &[f64], reached: &[bool], u: f64) -> Option<usize> {
    if reached.len() != jamp2.len() {
        return select_flow(jamp2, u);
    }
    let restricted: Vec<f64> = jamp2
        .iter()
        .zip(reached)
        .map(|(&w, &ok)| if ok { w } else { 0.0 })
        .collect();
    select_index(&restricted, u).or_else(|| select_flow(jamp2, u))
}

/// Which colour flows each diagram of a subprocess reaches at the basis's
/// highest power of `Nc` — MadGraph's `ICOLAMP`.
///
/// A diagram's colour factor spreads over several flows with different powers of
/// `Nc`; the flows carrying the highest power are that diagram's leading-colour
/// assignment, and the rest are its `1/N²`-suppressed pieces. MadEvent masks
/// `JAMP2` with the row of the configuration it is integrating before drawing an
/// event's colour flow, so this table is what decides which flow a Les Houches
/// record can carry.
///
/// The maximum is taken over the whole basis rather than per diagram, matching
/// MadGraph's `max_Nc`: a diagram that only ever appears suppressed reaches
/// nothing, and the draw off its row falls back to the unrestricted one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeadingColorFlows {
    n_flows: usize,
    /// Diagram-major: diagram `d`'s row is `reached[d * n_flows..][..n_flows]`.
    reached: Vec<bool>,
}

impl LeadingColorFlows {
    /// Read the table off a colour basis. `n_diagrams` is the subprocess's
    /// diagram count, so a diagram no flow references still gets a row.
    pub fn of(basis: &ColorBasis, n_diagrams: usize) -> Self {
        let n_flows = basis.ncolor();
        let max_nc = basis
            .elements
            .iter()
            .flat_map(|e| e.contributions.iter())
            .map(|c| c.coeff.nc_power)
            .max();
        let mut reached = vec![false; n_diagrams * n_flows];
        if let Some(max_nc) = max_nc {
            for (f, elem) in basis.elements.iter().enumerate() {
                for contrib in &elem.contributions {
                    if contrib.coeff.nc_power == max_nc && contrib.diagram < n_diagrams {
                        reached[contrib.diagram * n_flows + f] = true;
                    }
                }
            }
        }
        LeadingColorFlows { n_flows, reached }
    }

    /// The number of colour flows a row spans (NCOLOR).
    pub fn n_flows(&self) -> usize {
        self.n_flows
    }

    /// The number of diagrams the table has a row for.
    pub fn n_diagrams(&self) -> usize {
        if self.n_flows == 0 {
            0
        } else {
            self.reached.len() / self.n_flows
        }
    }

    /// Diagram `d`'s row, one flag per flow. Empty for a diagram outside the
    /// table, which leaves [`select_flow_reached_by`] unrestricted.
    pub fn reached_by(&self, diagram: usize) -> &[bool] {
        let start = diagram * self.n_flows;
        self.reached.get(start..start + self.n_flows).unwrap_or(&[])
    }
}

/// Derive the per-leg `(colour, anticolour)` tags of every flow in `basis`.
///
/// `legs` is the process's external legs in order (incoming first). Fails if a
/// basis key still holds an `f`/`d` tensor (the simplified basis is trace/δ
/// only), references a summed index, or produces a line assignment inconsistent
/// with the legs' colour reps.
pub fn color_flow_tags(
    basis: &ColorBasis,
    legs: &[LegColor],
) -> Result<ColorFlowTags, ColorAlgebraError> {
    let mut tags = Vec::with_capacity(basis.ncolor() * legs.len());
    for (f, elem) in basis.elements.iter().enumerate() {
        let flow = derive_flow(&elem.structure, legs)
            .map_err(|msg| ColorAlgebraError::InconsistentColorFlow(format!("flow {f}: {msg}")))?;
        tags.extend_from_slice(&flow);
    }
    Ok(ColorFlowTags {
        n_ext: legs.len(),
        tags,
    })
}

/// Accumulates one flow's colour lines into the per-leg `ICOLUP` pairs.
struct FlowBuilder<'a> {
    legs: &'a [LegColor],
    tags: Vec<[u32; 2]>,
    next_label: u32,
}

impl<'a> FlowBuilder<'a> {
    fn new(legs: &'a [LegColor]) -> Self {
        FlowBuilder {
            legs,
            tags: vec![[NO_COLOR_LINE; 2]; legs.len()],
            next_label: FIRST_COLOR_LINE,
        }
    }

    /// Join two endpoints with a fresh line label.
    fn connect(&mut self, a: (Idx, AmpRep), b: (Idx, AmpRep)) -> Result<(), String> {
        let label = self.next_label;
        self.next_label += 1;
        self.write(a, label)?;
        self.write(b, label)
    }

    fn write(&mut self, (idx, rep): (Idx, AmpRep), label: u32) -> Result<(), String> {
        if idx <= 0 {
            return Err(format!("summed colour index {idx} in a basis key"));
        }
        let leg = usize::try_from(idx - 1).expect("positive colour index");
        let info = *self
            .legs
            .get(leg)
            .ok_or_else(|| format!("colour index {idx} exceeds {} legs", self.legs.len()))?;
        let slot = rep.slot(info);
        if self.tags[leg][slot] != NO_COLOR_LINE {
            return Err(format!("leg {idx} slot {} written twice", slot + 1));
        }
        self.tags[leg][slot] = label;
        Ok(())
    }

    /// The occupied slots must be exactly the ones each leg's particle rep has.
    fn finish(self) -> Result<Vec<[u32; 2]>, String> {
        for (leg, info) in self.legs.iter().enumerate() {
            let want = slots_for(info.rep);
            let got = [
                self.tags[leg][COLOR_SLOT] != NO_COLOR_LINE,
                self.tags[leg][ANTICOLOR_SLOT] != NO_COLOR_LINE,
            ];
            if got != want {
                return Err(format!(
                    "leg {} is a {:?} but its lines occupy (colour, anticolour) = {got:?}",
                    leg + 1,
                    info.rep
                ));
            }
        }
        Ok(self.tags)
    }
}

/// Walk one basis key's `T`/`Tr` chains into colour lines.
fn derive_flow(structure: &ImmutableString, legs: &[LegColor]) -> Result<Vec<[u32; 2]>, String> {
    let mut builder = FlowBuilder::new(legs);
    for (kind, idxs) in structure {
        match kind {
            TensorKind::One => {}
            TensorKind::T => {
                if idxs.len() < 2 {
                    return Err("T tensor with fewer than two indices".into());
                }
                let (adj, ends) = idxs.split_at(idxs.len() - 2);
                let (i, j) = (ends[0], ends[1]);
                let mut left = (i, AmpRep::Three);
                for &a in adj {
                    builder.connect(left, (a, AmpRep::ThreeBar))?;
                    left = (a, AmpRep::Three);
                }
                builder.connect(left, (j, AmpRep::ThreeBar))?;
            }
            TensorKind::Tr => {
                if idxs.is_empty() {
                    return Err("empty trace".into());
                }
                for (k, &a) in idxs.iter().enumerate() {
                    let next = idxs[(k + 1) % idxs.len()];
                    builder.connect((a, AmpRep::Three), (next, AmpRep::ThreeBar))?;
                }
            }
            TensorKind::F | TensorKind::D => {
                return Err(format!("{kind:?} tensor survives in a basis key"));
            }
        }
    }
    builder.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quark(incoming: bool) -> LegColor {
        LegColor {
            rep: ColorRep::Triplet,
            incoming,
        }
    }

    fn antiquark(incoming: bool) -> LegColor {
        LegColor {
            rep: ColorRep::AntiTriplet,
            incoming,
        }
    }

    fn gluon(incoming: bool) -> LegColor {
        LegColor {
            rep: ColorRep::Octet,
            incoming,
        }
    }

    fn t(idxs: &[Idx]) -> (TensorKind, Vec<Idx>) {
        (TensorKind::T, idxs.to_vec())
    }

    fn tr(idxs: &[Idx]) -> (TensorKind, Vec<Idx>) {
        (TensorKind::Tr, idxs.to_vec())
    }

    /// `u u~ > u u~`, flow `T(2,1) T(3,4)`: the s-channel-like flow that joins
    /// the incoming pair and the outgoing pair. Reproduces MadGraph's
    /// `leshouche.inc` row `501, 0, 502, 0 / 0, 501, 0, 502` label for label.
    #[test]
    fn uux_annihilation_flow_matches_madgraph_labels() {
        let legs = [quark(true), antiquark(true), quark(false), antiquark(false)];
        let flow = derive_flow(&vec![t(&[2, 1]), t(&[3, 4])], &legs).expect("derive");
        assert_eq!(flow, vec![[501, 0], [0, 501], [502, 0], [0, 502]]);
    }

    /// `u u~ > u u~`, flow `T(2,4) T(3,1)`: the t-channel-like flow, with the
    /// incoming quark's line continuing into the outgoing quark.
    #[test]
    fn uux_exchange_flow_matches_madgraph_labels() {
        let legs = [quark(true), antiquark(true), quark(false), antiquark(false)];
        let flow = derive_flow(&vec![t(&[2, 4]), t(&[3, 1])], &legs).expect("derive");
        assert_eq!(flow, vec![[502, 0], [0, 501], [502, 0], [0, 501]]);
    }

    /// `g g > t t~`, flow `T(1,2,3,4)`: the quark line threads both gluons.
    #[test]
    fn ggttx_chain_matches_madgraph_labels() {
        let legs = [gluon(true), gluon(true), quark(false), antiquark(false)];
        let flow = derive_flow(&vec![t(&[1, 2, 3, 4])], &legs).expect("derive");
        assert_eq!(flow, vec![[501, 502], [502, 503], [501, 0], [0, 503]]);
    }

    /// `g g > g g`, flow `Tr(1,2,3,4)`: a closed four-gluon colour loop, matching
    /// MadGraph's `leshouche.inc` flow 1 label for label.
    #[test]
    fn gggg_trace_matches_madgraph_labels() {
        let legs = [gluon(true), gluon(true), gluon(false), gluon(false)];
        let flow = derive_flow(&vec![tr(&[1, 2, 3, 4])], &legs).expect("derive");
        assert_eq!(flow, vec![[504, 501], [501, 502], [503, 502], [504, 503]]);
    }

    /// The endpoint → `ICOLUP` slot map depends on the leg's direction: the same
    /// structure read with every leg outgoing puts an incoming quark's line in
    /// the wrong slot and trips the rep-consistency check.
    #[test]
    fn crossing_rule_is_not_free() {
        let structure = vec![t(&[2, 1]), t(&[3, 4])];
        let all_outgoing = [
            quark(false),
            antiquark(false),
            quark(false),
            antiquark(false),
        ];
        let err = derive_flow(&structure, &all_outgoing).expect_err("crossing must be checked");
        assert!(err.contains("lines occupy"), "unexpected error: {err}");
    }

    /// A basis key referencing a leg the process does not have is rejected
    /// rather than silently indexing past the table.
    #[test]
    fn out_of_range_index_is_rejected() {
        let legs = [quark(true), antiquark(true)];
        let err = derive_flow(&vec![t(&[2, 3])], &legs).expect_err("index must be checked");
        assert!(err.contains("exceeds"), "unexpected error: {err}");
    }

    /// `g u > g u` and its conjugate member `g u~ > g u~`, as MadGraph's own
    /// `leshouche.inc` for `P1_gq_gq` spells them: `isproc 1` and `isproc 2`,
    /// `[flow][leg] = [colour, anticolour]`.
    const GQ_GQ: [[[u32; 2]; 4]; 2] = [
        [[501, 502], [503, 0], [503, 502], [501, 0]],
        [[503, 502], [502, 0], [503, 501], [501, 0]],
    ];
    const GQX_GQX: [[[u32; 2]; 4]; 2] = [
        [[501, 502], [0, 501], [503, 502], [0, 503]],
        [[503, 502], [0, 501], [503, 501], [0, 502]],
    ];

    /// The colour lines a tag row induces, labels discarded: any consistent
    /// relabelling is the same flow.
    fn connectivity(tags: &[[u32; 2]]) -> std::collections::BTreeSet<Vec<(usize, usize)>> {
        let mut lines: std::collections::BTreeMap<u32, Vec<(usize, usize)>> =
            std::collections::BTreeMap::new();
        for (leg, pair) in tags.iter().enumerate() {
            for (slot, &label) in pair.iter().enumerate() {
                if label != NO_COLOR_LINE {
                    lines.entry(label).or_default().push((leg, slot));
                }
            }
        }
        lines.into_values().collect()
    }

    /// A one-element basis around a hand-written key, so a single flow can be run
    /// through [`ColorFlowTags`]' own transformations.
    fn one_flow(key: (TensorKind, Vec<Idx>), legs: &[LegColor]) -> ColorFlowTags {
        use crate::helas::color::colorize::{BasisElement, ColorBasis};
        let basis = ColorBasis {
            elements: vec![BasisElement {
                structure: vec![key],
                contributions: Vec::new(),
            }],
            cf_matrix: vec![num_rational::Ratio::from_integer(1)],
        };
        color_flow_tags(&basis, legs).expect("flow tags")
    }

    /// Conjugating a basis key and conjugating its tags are the same operation.
    ///
    /// `g u > g u`'s flow `T([1,3], 4, 2)` against `g u~ > g u~`'s
    /// `T([3,1], 2, 4)` — the key with its adjoint chain reversed and its two
    /// fundamental ends traded, which is what charge conjugation does to a colour
    /// structure. Deriving the conjugate key on the conjugate legs and exchanging
    /// both slots of the original's tags must reach the same colour lines, and both
    /// must be the lines MadGraph's `isproc 2` table carries.
    ///
    /// That the conjugate lands at MadGraph's *other* flow index is the point: the
    /// member's basis is the representative's with the flows permuted, so a member's
    /// event keeps the representative's flow index and takes the conjugated tags.
    #[test]
    fn conjugating_a_flow_is_the_slot_exchange_and_nothing_else() {
        let legs = [gluon(true), quark(true), gluon(false), quark(false)];
        let conj_legs = [gluon(true), antiquark(true), gluon(false), antiquark(false)];

        let ours = one_flow(t(&[1, 3, 4, 2]), &legs);
        assert_eq!(
            ours.flow(0),
            GQ_GQ[0],
            "the representative's own flow moved"
        );

        let by_key = one_flow(t(&[3, 1, 2, 4]), &conj_legs);
        let by_slots = ours.conjugated();
        assert_eq!(
            connectivity(by_key.flow(0)),
            connectivity(by_slots.flow(0)),
            "conjugating the key and exchanging the slots disagree"
        );
        assert_eq!(
            connectivity(by_slots.flow(0)),
            connectivity(&GQX_GQX[1]),
            "the conjugated flow is not MadGraph's isproc 2 flow 2"
        );

        // The other direction, so the index reversal is pinned rather than sampled.
        let other = one_flow(t(&[3, 1, 4, 2]), &legs);
        assert_eq!(
            connectivity(other.flow(0)),
            connectivity(&GQ_GQ[1]),
            "the representative's second flow is not MadGraph's isproc 1 flow 2"
        );
        assert_eq!(
            connectivity(other.conjugated().flow(0)),
            connectivity(&GQX_GQX[0]),
            "the conjugated second flow is not MadGraph's isproc 2 flow 1"
        );

        // A per-leg exchange restricted to the legs whose rep changed is legal on
        // every leg and still wrong, which is why the transformation is global.
        let per_leg: Vec<[u32; 2]> = ours
            .flow(0)
            .iter()
            .enumerate()
            .map(|(leg, &[c, a])| if leg % 2 == 1 { [a, c] } else { [c, a] })
            .collect();
        assert_ne!(
            connectivity(&per_leg),
            connectivity(&GQX_GQX[1]),
            "the quark-only exchange must not reproduce the conjugate's lines"
        );
    }

    /// A representative's tags are illegal on a conjugate member's legs.
    ///
    /// The check runs against the reps the *caller* supplies, not against anything
    /// read back out of the tags, so carrying `g u > g u`'s table onto `g u~ > g u~`
    /// is refused naming the leg and the rep that refused it.
    #[test]
    fn the_representatives_tags_are_illegal_on_a_conjugate_members_legs() {
        let legs = [gluon(true), quark(true), gluon(false), quark(false)];
        let conj_legs = [gluon(true), antiquark(true), gluon(false), antiquark(false)];
        let tags = one_flow(t(&[1, 3, 4, 2]), &legs);

        tags.check_legs(&legs).expect("legal on its own legs");
        let err = tags
            .check_legs(&conj_legs)
            .expect_err("the representative's slots cannot be legal on antiquarks");
        let msg = err.to_string();
        assert!(msg.contains("leg 2"), "unexpected error: {msg}");
        assert!(msg.contains("AntiTriplet"), "unexpected error: {msg}");

        tags.conjugated()
            .check_legs(&conj_legs)
            .expect("the conjugated table is legal on the conjugate member");
        tags.conjugated()
            .check_legs(&legs)
            .expect_err("and illegal back on the representative");

        // A leg list of the wrong length is refused rather than checked partially.
        tags.check_legs(&legs[..3]).expect_err("leg count");
    }

    /// The categorical draw follows the JAMP2 weights: cumulative boundaries
    /// land on the right flow, and a zero-weight flow is never drawn.
    #[test]
    fn select_flow_follows_the_weights() {
        let w = [1.0, 0.0, 3.0];
        assert_eq!(select_flow(&w, 0.0), Some(0));
        assert_eq!(select_flow(&w, 0.2), Some(0));
        assert_eq!(select_flow(&w, 0.25), Some(2));
        assert_eq!(select_flow(&w, 0.999), Some(2));
        assert_eq!(select_flow(&[0.0, 0.0], 0.5), None);
        assert_eq!(select_flow(&[f64::NAN], 0.5), None);
    }
}

#[cfg(test)]
mod masked_draw_tests {
    use super::*;

    /// The mask zeroes the flows a diagram does not reach, and the surviving
    /// weights keep their `JAMP2` ratios: a diagram reaching only the smaller flow
    /// must select it every time, which no reweighting of the full vector does.
    #[test]
    fn the_mask_removes_the_flows_a_diagram_does_not_reach() {
        let jamp2 = [9.0, 1.0];
        assert_eq!(select_flow_reached_by(&jamp2, &[false, true], 0.0), Some(1));
        assert_eq!(
            select_flow_reached_by(&jamp2, &[false, true], 0.999),
            Some(1)
        );
        assert_eq!(
            select_flow_reached_by(&jamp2, &[true, false], 0.999),
            Some(0)
        );
        // Reaching both is the unrestricted draw, boundary included.
        assert_eq!(select_flow_reached_by(&jamp2, &[true, true], 0.89), Some(0));
        assert_eq!(select_flow_reached_by(&jamp2, &[true, true], 0.91), Some(1));
        assert_eq!(select_flow(&jamp2, 0.91), Some(1));
    }

    /// Every route to "the mask carries no probability" falls back to the
    /// unrestricted draw rather than to `None`: an event whose amplitude at this
    /// point is entirely subleading still gets a colour flow, which is what
    /// MadEvent's `SELECT_COLOR` does when its masked cumulant ends at zero.
    #[test]
    fn a_mask_carrying_no_probability_falls_back_to_every_flow() {
        let jamp2 = [9.0, 1.0];
        // Reaches nothing.
        assert_eq!(
            select_flow_reached_by(&jamp2, &[false, false], 0.5),
            Some(0)
        );
        // Reaches a flow whose JAMP2 vanishes at this point.
        assert_eq!(
            select_flow_reached_by(&[0.0, 1.0], &[true, false], 0.5),
            Some(1)
        );
        // A row of the wrong length is not a mask for these weights.
        assert_eq!(select_flow_reached_by(&jamp2, &[true], 0.5), Some(0));
        // With nothing to draw from at all there is still no label.
        assert_eq!(
            select_flow_reached_by(&[0.0, 0.0], &[true, true], 0.5),
            None
        );
    }
}
