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
#[derive(Clone, Debug, PartialEq, Eq)]
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
/// This is a categorical draw off a diagonal accumulator (MadGraph's
/// `SELECT_COLOR`); it selects a flow for the event record and never enters the
/// integrand, so it has no effect on the cross section. It is the same draw the
/// per-event helicity selection makes off `|M_hel|²`, so both share one
/// definition ([`select_index`]).
pub fn select_flow(jamp2: &[f64], u: f64) -> Option<usize> {
    select_index(jamp2, u)
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
            let want = match info.rep {
                ColorRep::Singlet => [false, false],
                ColorRep::Triplet => [true, false],
                ColorRep::AntiTriplet => [false, true],
                ColorRep::Octet => [true, true],
            };
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
