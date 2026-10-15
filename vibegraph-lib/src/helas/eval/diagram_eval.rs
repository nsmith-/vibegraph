//! Pass-1+2 vertex/leg descriptors: the symbolic, model-bound data carried by a rooted
//! diagram node.
//!
//! These `*Info` types ([`ExtLegInfo`], [`PropInfo`], [`VertexInfo`]/[`VertexTerm`]) are
//! the payloads of [`EvalNode`](super::root_diagram::EvalNode): vertices still carry
//! model ids (`CouplingId`/`ParticleId`) and rooted Lorentz contraction trees. The
//! per-diagram artifact that assembles them ([`DiagramEval`](super::root_diagram::DiagramEval))
//! lives in [`super::compile`]; [`super::lower`] inlines them into the unified
//! [`Ast`](super::ast::Ast), which the runtime evaluates.

use itertools::Itertools;

use super::root_lorentz::{Adjoint, LegAdjoint, RootLorentzError, RootedTerm};
use crate::helas::repr::numbers::Charge;
use crate::ufo::couplings::CouplingId;
use crate::ufo::lorentz::LorentzId;
use crate::ufo::particles::ParticleId;
use crate::ufo::topo::{flow_groups, FlowGroup};
use crate::ufo::vertices::VertexId;
use crate::ufo::UFOModel;

/// Description of an external leg baked in at compile time.
#[derive(Clone, Debug)]
pub(crate) struct ExtLegInfo {
    /// Particle information
    pub(crate) id: ParticleId,
    /// Index into external leg array (0..n_in are incoming; n_in.. are outgoing)
    pub(crate) leg_idx: usize,
    /// Spin code (UFOModel convention: 2s+1)
    pub(crate) spin: i32,
    /// Charge
    pub(crate) charge: Charge,
    /// Whether this leg is incoming (`leg_idx < n_in`); selects ket/bra adjoint and the
    /// HELAS `nsf` sign of the external wavefunction.
    pub(crate) incoming: bool,
}

impl ExtLegInfo {
    /// Spinor adjoint of this external leg, or `None` for non-Dirac legs.
    ///
    /// Mirrors the HELAS external-adjoint rule applied at eval time in
    /// `build_external_core`: a Dirac leg is a ket (ket) iff it is an incoming
    /// particle or an outgoing antiparticle, i.e. `incoming == is_particle`.
    pub(crate) fn adjoint(&self) -> Option<Adjoint> {
        if self.spin != 2 {
            return None;
        }
        let is_particle = matches!(self.charge, Charge::Particle);
        Some(if self.incoming == is_particle {
            Adjoint::Ket
        } else {
            Adjoint::Bra
        })
    }
}

impl std::fmt::Display for ExtLegInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "id: {:?}, leg: {} {} {} }}",
            self.id, self.leg_idx, self.spin, self.charge
        )
    }
}

/// Description of an internal propagator.
#[derive(Clone, Debug)]
pub(crate) struct PropInfo {
    /// Particle information
    pub(crate) id: ParticleId,
    /// True iff the line is t-channel — it separates the two initial-state legs
    /// (exactly one incoming external in its subtree), so its momentum is
    /// spacelike and can never resonate. MadGraph passes ZERO width for such
    /// propagators (cf. the Bhabha t-channel Z, `FFV2_4_3(..., MDL_MZ, ZERO, ...)`)
    /// and the width is dropped at lowering accordingly.
    pub(crate) t_channel: bool,
}

/// One (lorentz_structure, coupling_constant) pair at a vertex.
///
/// The `LorentzId` is stored so that the AST remains independent of the UFO model reference.
/// At compile time the `LorentzExpr` is pattern-matched into `RootedTerm`
/// At eval time, `coupling_id` is resolved via `EvaluatedModel::coupling(id)`.
#[derive(Clone, Debug)]
pub(crate) struct VertexTerm {
    /// Pre-compiled rooted dispatch (from LorentzTerm pattern match, rooted at output leg)
    pub(crate) terms: Vec<RootedTerm>,
    /// Model's coupling ID (resolved via EvaluatedModel at eval time)
    pub(crate) coupling_id: CouplingId,
}

impl VertexTerm {
    /// Generate a VertexTerm from a UFO vertex definition, given the model and the desired index of the result leg.
    ///
    /// result_leg_idx is 0-indexed here
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_ufo(
        model: &UFOModel,
        lorentz_id: LorentzId,
        _color: &crate::ufo::color::ColorExpr, // TODO: handle color structures if needed
        coupling_id: CouplingId,
        flow: &[(usize, usize)],
        result_leg_idx: Option<usize>,
        flows: &[Option<LegAdjoint>],
    ) -> Result<Self, RootLorentzError> {
        let lorentz = model.lorentz_struct(lorentz_id);
        super::root_lorentz::reject_cyclic_structure(
            &lorentz.name,
            &lorentz.structure,
            &lorentz.spins,
            &lorentz.expr,
        )?;

        let terms = lorentz
            .expr
            .iter()
            .map(|term| {
                super::root_lorentz::root_term(term, &lorentz.spins, flow, result_leg_idx, flows)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(VertexTerm { terms, coupling_id })
    }

    /// The tensor-path sign shared by this vertex-term's Lorentz terms (see
    /// [`RootedTerm::build_sign`]); `+1` for tree-rooted terms and for an empty term
    /// list. Terms that disagree have no per-vertex sign to lift, and the vertex is
    /// refused.
    fn build_sign(&self) -> Result<i8, RootLorentzError> {
        uniform_sign(
            self.terms.iter().map(|t| t.build_sign),
            "tensor-path signs",
            self,
        )
    }

    /// Whether this vertex-term's Lorentz terms are evaluated through the rank-2 tensor
    /// path (see [`RootedTerm::tensor`]). Uniform across the terms: a vertex whose
    /// structures mix a cyclic tensor term with a tree-rootable one has no single
    /// per-vertex line convention. Empty term list → `false`.
    fn tensor(&self) -> bool {
        let mut it = self.terms.iter().map(|t| t.tensor);
        let Some(first) = it.next() else { return false };
        assert!(
            it.all(|s| s == first),
            "a vertex's Lorentz terms mix the tensor path with tree rooting"
        );
        first
    }

    /// The runtime `reversed`-bilinear parity shared by this vertex-term's Lorentz terms
    /// (see [`RootedTerm::reversed_sign`]). Uniform across the terms for the same reason
    /// as [`build_sign`](Self::build_sign) (they share the vertex's fermion legs). Empty
    /// term list → `+1`; terms that disagree are refused.
    fn reversed_sign(&self) -> Result<i8, RootLorentzError> {
        uniform_sign(
            self.terms.iter().map(|t| t.reversed_sign),
            "reversed-bilinear parities",
            self,
        )
    }

    /// Convert the rooted term
    fn render_term(&self) -> String {
        self.terms
            .iter()
            .map(|term| format!("{}", term))
            .collect::<Vec<_>>()
            .join("+")
    }
}

impl std::fmt::Display for VertexTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}*({})",
            self.coupling_id,
            self.render_term().as_str()
        )
    }
}

/// Descriptor for one vertex with all its terms (sum over Lorentz × color).
#[derive(Clone, Debug)]
pub(crate) struct VertexInfo {
    /// Sum over Lorentz + color terms
    pub(crate) terms: Vec<VertexTerm>,
}

/// The fermion-flow group `group` of `id`'s Lorentz structures: which of the
/// vertex's spinor pairings this occurrence of it uses.
///
/// The group index comes from the diagram
/// ([`Vertex::flow_group`](crate::diagrams::diagram::Vertex::flow_group)), which
/// carries it from the feyngraph vertex the enumeration chose.
pub(super) fn vertex_flow_group(model: &UFOModel, id: VertexId, group: usize) -> FlowGroup {
    let mut groups = flow_groups(model.vertex_def(id), &model.lorentz);
    assert!(
        group < groups.len(),
        "vertex '{}' has {} fermion-flow groups, asked for {group}",
        model.vertex_def(id).name,
        groups.len()
    );
    groups.swap_remove(group)
}

impl VertexInfo {
    /// Generate VertexInfo from a UFO vertex definition, restricted to a single
    /// color structure `color_idx` and a single fermion-flow group `flow_group`,
    /// given the model and the desired index of the result leg.
    ///
    /// Only the `(color_idx, lorentz_idx)` couplings whose first component equals
    /// `color_idx` are summed into the term list, so a vertex with several color
    /// structures (e.g. the 4-gluon vertex) compiles into a distinct term list per
    /// color-index chain. For the usual single-structure vertex every coupling key
    /// has first component `0`, so `color_idx == 0` keeps them all in their original
    /// iteration order.
    ///
    /// `flow_group` narrows the same way along the other axis: a four-fermion vertex
    /// whose structures contract its legs two different ways describes two fermion-line
    /// topologies, which the enumeration has already separated into two vertices, so
    /// only the structures of the named group belong to this occurrence.
    pub(crate) fn from_ufo(
        model: &UFOModel,
        id: VertexId,
        color_idx: usize,
        flow_group: usize,
        result_leg_idx: Option<usize>,
        flows: &[Option<LegAdjoint>],
    ) -> Result<Self, RootLorentzError> {
        let vertex = model.vertex_def(id);
        let group = vertex_flow_group(model, id, flow_group);
        let terms = vertex
            .couplings
            .iter()
            .filter(|(&(c, l), _)| c == color_idx && group.lorentz.contains(&l))
            .map(|(&(_, lorentz_idx), coupling_id)| {
                VertexTerm::from_ufo(
                    model,
                    vertex.lorentz[lorentz_idx],
                    &vertex.color[color_idx],
                    *coupling_id,
                    &group.flow,
                    result_leg_idx,
                    flows,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(VertexInfo { terms })
    }

    /// The tensor-path sign of this vertex, common to all its `(color, lorentz)` terms
    /// (see [`VertexTerm::build_sign`]); with the vertex as the amplitude root it is one
    /// factor of the diagram's convention sign (`root_diagram::vertex_root_signs`).
    ///
    /// A vertex whose couplings disagree on the sign is refused: the sign is lifted to
    /// one factor per vertex, which such a vertex does not have.
    pub(super) fn build_sign(&self) -> Result<i8, RootLorentzError> {
        let signs = self
            .terms
            .iter()
            .map(VertexTerm::build_sign)
            .collect::<Result<Vec<_>, _>>()?;
        uniform_sign(signs, "tensor-path signs", self)
    }

    /// Whether this vertex is evaluated through the rank-2 tensor path, common to all its
    /// terms (see [`VertexTerm::tensor`]).
    pub(super) fn tensor(&self) -> bool {
        let mut it = self.terms.iter().map(|t| t.tensor());
        let Some(first) = it.next() else { return false };
        assert!(
            it.all(|s| s == first),
            "a vertex's couplings mix the tensor path with tree rooting"
        );
        first
    }

    /// The reversed-bilinear parity of this vertex, common to all its terms (see
    /// [`VertexTerm::reversed_sign`]). With the vertex as the amplitude root it is one
    /// factor of the diagram's convention sign (`root_diagram::vertex_root_signs`); the
    /// product over a live tree is
    /// [`live_reversed_parity`](super::root_diagram::DiagramEvalTree::live_reversed_parity).
    pub(super) fn reversed_sign(&self) -> Result<i8, RootLorentzError> {
        let signs = self
            .terms
            .iter()
            .map(VertexTerm::reversed_sign)
            .collect::<Result<Vec<_>, _>>()?;
        uniform_sign(signs, "reversed-bilinear parities", self)
    }
}

/// The one sign every entry of `signs` carries (`+1` for none), or a refusal naming
/// `what` and the vertex when they differ.
fn uniform_sign(
    signs: impl IntoIterator<Item = i8>,
    what: &str,
    vertex: &impl std::fmt::Display,
) -> Result<i8, RootLorentzError> {
    let mut it = signs.into_iter();
    let Some(first) = it.next() else { return Ok(1) };
    if it.all(|s| s == first) {
        Ok(first)
    } else {
        Err(RootLorentzError::UnsupportedVertex(format!(
            "the structures of `{vertex}` carry mixed {what}, so the vertex has no single \
             sign to lift into the diagram's"
        )))
    }
}

impl std::fmt::Display for VertexInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            self.terms
                .iter()
                .map(|term| format!("{}", term))
                .join(" + ")
                .as_str(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ufo::lorentz::LorentzTerm;

    /// A plain all-scalar term rooted at the amplitude.
    fn scalar_term() -> RootedTerm {
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![],
        };
        super::super::root_lorentz::root_term(&term, &[1, 1, 1], &[], None, &[None; 3]).unwrap()
    }

    /// A vertex whose structures disagree on the tensor-path sign, or on the
    /// reversed-bilinear parity, is refused rather than given the first term's sign.
    #[test]
    fn mixed_vertex_signs_are_refused() {
        let plain = scalar_term();
        assert_eq!(plain.build_sign, 1);
        let mut flipped = plain.clone();
        flipped.build_sign = -1;
        let mut reversed = plain.clone();
        reversed.reversed_sign = -reversed.reversed_sign;

        let vertex = |a: &RootedTerm, b: &RootedTerm| VertexInfo {
            terms: vec![
                VertexTerm {
                    terms: vec![a.clone()],
                    coupling_id: CouplingId::from(0),
                },
                VertexTerm {
                    terms: vec![b.clone()],
                    coupling_id: CouplingId::from(1),
                },
            ],
        };
        assert_eq!(vertex(&plain, &plain).build_sign().unwrap(), 1);
        assert_eq!(vertex(&plain, &plain).reversed_sign().unwrap(), 1);
        for (what, a, b) in [
            (
                "across couplings",
                vertex(&plain, &flipped),
                vertex(&plain, &reversed),
            ),
            (
                "within one coupling",
                VertexInfo {
                    terms: vec![VertexTerm {
                        terms: vec![plain.clone(), flipped.clone()],
                        coupling_id: CouplingId::from(0),
                    }],
                },
                VertexInfo {
                    terms: vec![VertexTerm {
                        terms: vec![plain.clone(), reversed.clone()],
                        coupling_id: CouplingId::from(0),
                    }],
                },
            ),
        ] {
            let err = a.build_sign().unwrap_err();
            assert!(
                matches!(&err, RootLorentzError::UnsupportedVertex(why) if why.contains("tensor-path")),
                "{what}: {err:?}"
            );
            let err = b.reversed_sign().unwrap_err();
            assert!(
                matches!(&err, RootLorentzError::UnsupportedVertex(why) if why.contains("reversed-bilinear")),
                "{what}: {err:?}"
            );
        }
    }
}
