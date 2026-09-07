//! Pass-1+2 vertex/leg descriptors: the symbolic, model-bound data carried by a rooted
//! diagram node.
//!
//! These `*Info` types ([`ExtLegInfo`], [`PropInfo`], [`VertexInfo`]/[`VertexTerm`]) are
//! the payloads of [`EvalNode`](super::root_diagram::EvalNode): vertices still carry
//! model ids (`CouplingId`/`ParticleId`) and rooted Lorentz contraction trees. The
//! per-diagram artifact that assembles them ([`DiagramEval`](super::compile::DiagramEval))
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
pub struct ExtLegInfo {
    /// Particle information
    pub id: ParticleId,
    /// Index into external leg array (0..n_in are incoming; n_in.. are outgoing)
    pub leg_idx: usize,
    /// Spin code (UFOModel convention: 2s+1)
    pub spin: i32,
    /// Charge
    pub charge: Charge,
    /// Whether this leg is incoming (`leg_idx < n_in`); selects ket/bra adjoint and the
    /// HELAS `nsf` sign of the external wavefunction.
    pub incoming: bool,
}

impl ExtLegInfo {
    /// Spinor adjoint of this external leg, or `None` for non-Dirac legs.
    ///
    /// Mirrors the HELAS external-adjoint rule applied at eval time in
    /// `build_external_core`: a Dirac leg is a ket (ket) iff it is an incoming
    /// particle or an outgoing antiparticle, i.e. `incoming == is_particle`.
    pub fn adjoint(&self) -> Option<Adjoint> {
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
pub struct PropInfo {
    /// Particle information
    pub id: ParticleId,
    /// True iff the line is t-channel — it separates the two initial-state legs
    /// (exactly one incoming external in its subtree), so its momentum is
    /// spacelike and can never resonate. MadGraph passes ZERO width for such
    /// propagators (cf. the Bhabha t-channel Z, `FFV2_4_3(..., MDL_MZ, ZERO, ...)`)
    /// and the width is dropped at lowering accordingly.
    pub t_channel: bool,
}

/// One (lorentz_structure, coupling_constant) pair at a vertex.
///
/// The `LorentzId` is stored so that the AST remains independent of the UFO model reference.
/// At compile time the `LorentzExpr` is pattern-matched into `RootedTerm`
/// At eval time, `coupling_id` is resolved via `EvaluatedModel::coupling(id)`.
#[derive(Clone, Debug)]
pub struct VertexTerm {
    /// Pre-compiled rooted dispatch (from LorentzTerm pattern match, rooted at output leg)
    pub terms: Vec<RootedTerm>,
    /// Model's coupling ID (resolved via EvaluatedModel at eval time)
    pub coupling_id: CouplingId,
}

impl VertexTerm {
    /// Generate a VertexTerm from a UFO vertex definition, given the model and the desired index of the result leg.
    ///
    /// result_leg_idx is 0-indexed here
    #[allow(clippy::too_many_arguments)]
    pub fn from_ufo(
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

    /// The rooting-convention sign shared by this vertex-term's Lorentz terms (see
    /// [`RootedTerm::build_sign`]). Every term of a vertex carries the same sign — the
    /// convention −1 is a property of the vertex current, not of an individual coupling
    /// term — so a chiral `ProjM`+`ProjP` pair flips together (verified across the
    /// MG-validated set: no vertex has mixed per-term build signs). Empty term list → `+1`.
    fn build_sign(&self) -> i8 {
        let mut it = self.terms.iter().map(|t| t.build_sign);
        let Some(first) = it.next() else { return 1 };
        assert!(
            it.all(|s| s == first),
            "a vertex's Lorentz terms carry mixed rooting-convention signs — the \
             per-vertex sign factorization does not hold for this structure"
        );
        first
    }

    /// Whether this vertex-term's bilinear carries a Dirac matrix (see
    /// [`RootedTerm::carries_dirac_matrix`]). Uniform across a vertex's terms for the
    /// same reason as [`build_sign`](Self::build_sign): the terms share the vertex's
    /// fermion legs, and a gauge bilinear does not sit in the same current as a
    /// Dirac-matrix-free one. Empty term list → `false`.
    fn carries_dirac_matrix(&self) -> bool {
        let mut it = self.terms.iter().map(|t| t.carries_dirac_matrix);
        let Some(first) = it.next() else { return false };
        assert!(
            it.all(|s| s == first),
            "a vertex's Lorentz terms disagree on carrying a Dirac matrix"
        );
        first
    }

    /// The runtime `reversed`-bilinear parity shared by this vertex-term's Lorentz terms
    /// (see [`RootedTerm::reversed_sign`]). Uniform across the terms for the same reason
    /// as [`build_sign`](Self::build_sign) (they share the vertex's fermion legs). Empty
    /// term list → `+1`.
    fn reversed_sign(&self) -> i8 {
        let mut it = self.terms.iter().map(|t| t.reversed_sign);
        let Some(first) = it.next() else { return 1 };
        assert!(
            it.all(|s| s == first),
            "a vertex's Lorentz terms carry mixed reversed-bilinear parities"
        );
        first
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
pub struct VertexInfo {
    /// Sum over Lorentz + color terms
    pub terms: Vec<VertexTerm>,
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
    pub fn from_ufo(
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

    /// The rooting-convention sign of this vertex, common to all its `(color, lorentz)`
    /// terms (see [`VertexTerm::build_sign`]). Product of these over a diagram's vertices,
    /// evaluated at the canonical `VtxIdx(0)` rooting, is the diagram's
    /// [`build_convention_sign`](super::root_diagram::DiagramEvalTree::build_convention_sign).
    pub(super) fn build_sign(&self) -> i8 {
        let mut it = self.terms.iter().map(|t| t.build_sign());
        let Some(first) = it.next() else { return 1 };
        assert!(
            it.all(|s| s == first),
            "a vertex's couplings carry mixed rooting-convention signs"
        );
        first
    }

    /// Whether this vertex's bilinear carries a Dirac matrix, common to all its terms
    /// (see [`VertexTerm::carries_dirac_matrix`]). Read once per vertex on a fermion
    /// line by [`spine_sign_from_flow`](super::root_diagram::spine_sign_from_flow).
    pub(super) fn carries_dirac_matrix(&self) -> bool {
        let mut it = self.terms.iter().map(|t| t.carries_dirac_matrix());
        let Some(first) = it.next() else { return false };
        assert!(
            it.all(|s| s == first),
            "a vertex's couplings disagree on carrying a Dirac matrix"
        );
        first
    }

    /// The reversed-bilinear parity of this vertex, common to all its terms (see
    /// [`VertexTerm::reversed_sign`]). Product over a diagram's vertices at the canonical
    /// rooting is [`reversed_convention_sign`](super::root_diagram::DiagramEvalTree::reversed_convention_sign).
    pub(super) fn reversed_sign(&self) -> i8 {
        let mut it = self.terms.iter().map(|t| t.reversed_sign());
        let Some(first) = it.next() else { return 1 };
        assert!(
            it.all(|s| s == first),
            "a vertex's couplings carry mixed reversed-bilinear parities"
        );
        first
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
