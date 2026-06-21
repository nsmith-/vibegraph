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

use super::root_lorentz::{Flow, RootLorentzError, RootedTerm};
use crate::helas::repr::numbers::Charge;
use crate::ufo::couplings::CouplingId;
use crate::ufo::lorentz::LorentzId;
use crate::ufo::particles::ParticleId;
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
    /// Whether this leg is incoming (`leg_idx < n_in`); selects ket/bra flow and the
    /// HELAS `nsf` sign of the external wavefunction.
    pub incoming: bool,
}

impl ExtLegInfo {
    /// Spinor flow of this external leg, or `None` for non-Dirac legs.
    ///
    /// Mirrors the HELAS external-flow rule applied at eval time in
    /// `build_external_core`: a Dirac leg is a ket (flow-in) iff it is an incoming
    /// particle or an outgoing antiparticle, i.e. `incoming == is_particle`.
    pub fn flow(&self) -> Option<Flow> {
        if self.spin != 2 {
            return None;
        }
        let is_particle = matches!(self.charge, Charge::Particle);
        Some(if self.incoming == is_particle {
            Flow::In
        } else {
            Flow::Out
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
    pub fn from_ufo(
        model: &UFOModel,
        lorentz_id: LorentzId,
        _color: &str, // TODO: handle color structures if needed
        coupling_id: CouplingId,
        result_leg_idx: Option<usize>,
        out_flow: Option<Flow>,
    ) -> Result<Self, RootLorentzError> {
        let lorentz = model.lorentz_struct(lorentz_id);

        let terms = lorentz
            .expr
            .iter()
            .map(|term| {
                super::root_lorentz::root_term(term, &lorentz.spins, result_leg_idx, out_flow)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(VertexTerm { terms, coupling_id })
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

impl VertexInfo {
    /// Generate VertexInfo from a UFO vertex definition, given the model and the desired index of the result leg.
    pub fn from_ufo(
        model: &UFOModel,
        id: VertexId,
        result_leg_idx: Option<usize>,
        out_flow: Option<Flow>,
    ) -> Result<Self, RootLorentzError> {
        let vertex = model.vertex_def(id);
        let terms = vertex
            .couplings
            .iter()
            .map(|(&(color_idx, lorentz_idx), coupling_id)| {
                VertexTerm::from_ufo(
                    model,
                    vertex.lorentz[lorentz_idx],
                    vertex.color[color_idx].as_str(),
                    *coupling_id,
                    result_leg_idx,
                    out_flow,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(VertexInfo { terms })
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
