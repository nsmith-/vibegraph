//! Compiled amplitude evaluator AST types.
//!
//! A `DiagramAst` represents a single Feynman diagram in an efficiently-evaluable form.
//! It stores descriptor types at compile time (external leg info, propagator params, vertex
//! dispatches) and is evaluated at runtime against external momenta + helicity configurations.

use itertools::Itertools;

use super::root_lorentz::RootedTerm;
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
    // TODO: mass: F
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
    /// Per-leg spin codes (from LorentzStructure.spins)
    pub spins: Vec<i32>,
    /// Model's coupling ID (resolved via EvaluatedModel at eval time)
    pub coupling_id: CouplingId,
}

impl VertexTerm {
    /// Generate a VertexTerm from a UFO vertex definition, given the model and the desired index of the result leg.
    ///
    /// TODO: move this to compile.rs as free function, so errors can propagate instead of panicking.
    ///
    /// result_leg_idx is 0-indexed here
    pub fn from_ufo(
        model: &UFOModel,
        lorentz_id: LorentzId,
        _color: &str, // TODO: handle color structures if needed
        coupling_id: CouplingId,
        result_leg_idx: Option<usize>,
    ) -> Self {
        let lorentz = model.lorentz_struct(lorentz_id);

        let terms = lorentz
            .expr
            .iter()
            .map(|term| {
                super::root_lorentz::root_term(term, &lorentz.spins, result_leg_idx)
                    .expect("Unable to root term from Lorentz expression")
            })
            .collect();

        VertexTerm {
            terms,
            spins: lorentz.spins.clone(),
            coupling_id,
        }
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
    /// Total number of vertex legs
    pub n_legs: usize,
}

impl VertexInfo {
    /// Generate VertexInfo from a UFO vertex definition, given the model and the desired index of the result leg.
    ///
    /// TODO: move this to compile.rs as free function, so errors can propagate instead of panicking.
    pub fn from_ufo(model: &UFOModel, id: VertexId, result_leg_idx: Option<usize>) -> Self {
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
                )
            })
            .collect();
        VertexInfo {
            terms,
            n_legs: vertex.particles.len(),
        }
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

/// One compilation step in the diagram evaluation.
#[derive(Clone, Debug)]
pub enum EvalStep {
    /// Initialize an external wavefunction from momentum + helicity.
    ExternalWf {
        /// External leg descriptor
        info: ExtLegInfo,
        /// Which slot receives this wavefunction
        output_slot: usize,
    },

    /// Apply a vertex to compute an off-shell current (all but one leg known).
    OffShellCurrent {
        /// Vertex descriptor (terms)
        info: VertexInfo,
        /// Slots for the known legs (inputs to the vertex)
        input_slots: Vec<usize>,
        /// Slot receiving the off-shell wavefunction (output)
        output_slot: usize,
    },

    /// Apply a propagator to an off-shell wavefunction.
    Propagate {
        /// Propagator descriptor (mass, width, momentum coeffs)
        info: PropInfo,
        /// Input slot (wavefunction to propagate)
        input_slot: usize,
        /// Output slot (propagated wavefunction)
        output_slot: usize,
    },

    /// Final vertex: all legs known → produces a complex scalar amplitude.
    /// This is typically stored in a temporary slot as a `WaveformSlot::Scalar`.
    ContractAmplitude {
        /// Vertex descriptor (terms)
        info: VertexInfo,
        /// All legs (inputs to the final vertex)
        input_slots: Vec<usize>,
        /// Slot receiving the amplitude (as WaveformSlot::Scalar)
        output_slot: usize,
    },
}

impl EvalStep {
    pub fn output_slot(&self) -> usize {
        match self {
            EvalStep::ExternalWf { output_slot, .. } => *output_slot,
            EvalStep::OffShellCurrent { output_slot, .. } => *output_slot,
            EvalStep::Propagate { output_slot, .. } => *output_slot,
            EvalStep::ContractAmplitude { output_slot, .. } => *output_slot,
        }
    }
}

impl std::fmt::Display for EvalStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalStep::ExternalWf { info, .. } => write!(f, "ExternalWf({})", info),
            EvalStep::OffShellCurrent {
                info, input_slots, ..
            } => {
                write!(f, "OffShellCurrent({}, {:?})", info, input_slots)
            }
            EvalStep::Propagate {
                info: PropInfo { id },
                input_slot,
                ..
            } => write!(f, "Propagate({:?}, slots[{}])", id, input_slot),
            EvalStep::ContractAmplitude {
                info, input_slots, ..
            } => {
                write!(f, "ContractAmplitude({}, {:?})", info, input_slots)
            }
        }
    }
}

/// A compiled representation of a single Feynman diagram.
///
/// The AST is built once from a `DiagramView` + `UFOModel` and then evaluated
/// rapidly at each phase-space point. It uses a slot machine: a fixed-length
/// array of `WaveformSlot` acts as a register file. Each `EvalStep` reads from
/// some slots and writes to one slot.
#[derive(Clone, Debug)]
pub struct DiagramAst {
    /// Number of external legs (determines array indexing for momenta)
    pub n_ext: usize,
    /// Total number of slots needed (= n_ext + internal propagators + temporaries)
    pub n_slots: usize,
    /// Steps in topological order (all inputs available before execution)
    pub steps: Vec<EvalStep>,
    /// Which slot holds the final Complex64 amplitude
    pub amplitude_slot: usize,
    /// Symmetry factor: 1 / (vertex_sym × propagator_sym)
    pub symmetry_factor: f64,
    /// ±1 from the diagram's Fermi permutation sign
    pub fermi_sign: i8,
}

impl DiagramAst {
    /// Create a new DiagramAst.
    pub fn new(
        n_ext: usize,
        n_slots: usize,
        steps: Vec<EvalStep>,
        amplitude_slot: usize,
        symmetry_factor: f64,
        fermi_sign: i8,
    ) -> Self {
        DiagramAst {
            n_ext,
            n_slots,
            steps,
            amplitude_slot,
            symmetry_factor,
            fermi_sign,
        }
    }
}

impl std::fmt::Display for DiagramAst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AST(external legs {}, slots {}) steps:\n{}",
            self.n_ext,
            self.n_slots,
            self.steps
                .iter()
                .map(|s| format!(" slots[{}] = {}", s.output_slot(), s))
                .join("\n")
        )
    }
}
