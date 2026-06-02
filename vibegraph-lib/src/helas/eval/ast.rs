//! Compiled amplitude evaluator AST types.
//!
//! A `DiagramAst` represents a single Feynman diagram in an efficiently-evaluable form.
//! It stores descriptor types at compile time (external leg info, propagator params, vertex
//! dispatches) and is evaluated at runtime against external momenta + helicity configurations.

use crate::helas::repr::lorentz::LorentzVector;
use crate::helas::repr::{Real, C};
use crate::helas::wavefn::{DiracWf, ScalarWf, VectorWf};
use crate::ufo::couplings::CouplingId;
use crate::ufo::lorentz::LorentzId;

/// A runtime wavefunction register (holds one particle's wavefunction).
///
/// The `Flow` phantom tag on `DiracWf` enforces correct pairing at function call boundaries
/// but is meaningless for internal off-shell currents. Slots therefore hold `DiracWf<F>`
/// (default phantom): correctness is guaranteed by the AST topology (compile step knows
/// which vertex leg is "in" vs "out"), not by the type.
#[derive(Clone, Debug)]
pub enum WaveformSlot<F: Real> {
    /// 4-component Dirac spinor / off-shell fermion current
    Fermion(DiracWf<F>),
    /// 4-component polarization / off-shell vector current
    Vector(VectorWf<F>),
    /// Scalar amplitude + momentum
    Scalar(ScalarWf<F>),
    /// Empty slot (not yet computed)
    Empty,
}

/// Description of an external leg baked in at compile time.
#[derive(Clone, Debug)]
pub struct ExtLegInfo {
    /// Index into external leg array (0..n_in are incoming; n_in.. are outgoing)
    pub leg_idx: usize,
    /// UFO spin code (1=scalar, 2=fermion, 3=vector)
    pub spin: i32,
    /// Particle mass (GeV)
    pub mass: f64,
    /// True if incoming leg, false if outgoing
    pub is_incoming: bool,
    /// For debugging
    pub particle_name: String,
}

/// Description of an internal propagator.
#[derive(Clone, Debug)]
pub struct PropInfo {
    /// UFO spin code (determines which Propagator impl to use)
    pub spin: i32,
    /// Particle mass (GeV)
    pub mass: f64,
    /// Particle width (GeV)
    pub width: f64,
    /// Momentum = Σ_i coeff[i] * p_ext[i]; i indexes external legs in order
    /// Coefficients are ±1 indicating which external legs contribute (inflow direction)
    pub momentum_coeffs: Vec<i8>,
}

/// Pre-compiled dispatch tag, derived from LorentzExpr + spins at compile time.
/// Eliminates symbolic evaluation on the hot path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DispatchKind {
    /// FFV with left-chiral projector (ProjM)
    FfvProjM,
    /// FFV with right-chiral projector (ProjP)
    FfvProjP,
    /// FFS Yukawa (scalar coupling)
    Ffs,
    /// VVV triple gauge
    Vvv,
    /// VVVV quartic gauge
    Vvvv,
    /// VVS (Higgs coupling)
    Vvs,
    /// SSS scalar triple
    Sss,
    /// SSSS scalar quartic
    Ssss,
}

/// One (lorentz_structure, coupling_constant) pair at a vertex.
///
/// The `LorentzId` is stored so that the AST remains independent of the UFO model reference.
/// At compile time the `LorentzExpr` is pattern-matched into `DispatchKind`.
/// At eval time, `coupling_id` is resolved via `EvaluatedModel::coupling(id)`.
#[derive(Clone, Debug)]
pub struct VertexTerm {
    /// Model's lorentz structure ID (can resolve to LorentzExpr if needed)
    pub lorentz_id: LorentzId,
    /// Pre-compiled dispatch tag (from LorentzExpr pattern match)
    pub dispatch_kind: DispatchKind,
    /// Per-leg spin codes (from LorentzStructure.spins)
    pub spins: Vec<i32>,
    /// Model's coupling ID (resolved via EvaluatedModel at eval time)
    pub coupling_id: CouplingId,
}

/// Descriptor for one vertex with all its terms (sum over Lorentz × color).
#[derive(Clone, Debug)]
pub struct VertexInfo {
    /// Sum over Lorentz + color terms
    pub terms: Vec<VertexTerm>,
    /// Which vertex-local leg is the output (receives the off-shell current)
    /// Convention: the leg that connects toward the root of the tree
    pub result_leg_idx: usize,
    /// Total number of vertex legs
    pub n_legs: usize,
}

/// One compilation step in the diagram evaluation.
#[derive(Clone, Debug)]
pub enum EvalStep {
    /// Initialize an external wavefunction from momentum + helicity.
    ExternalWf {
        /// External leg descriptor
        info: ExtLegInfo,
        /// Index into the runtime helicity array (set at eval time)
        hel_index: usize,
        /// Which slot receives this wavefunction
        slot: usize,
    },

    /// Apply a vertex to compute an off-shell current (all but one leg known).
    OffShellCurrent {
        /// Vertex descriptor (terms + output leg)
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
        in_slot: usize,
        /// Output slot (propagated wavefunction)
        out_slot: usize,
    },

    /// Final vertex: all legs known → produces a complex scalar amplitude.
    /// This is typically stored in a temporary slot as a `WaveformSlot::Scalar`.
    ContractAmplitude {
        /// Vertex descriptor (terms + output leg)
        info: VertexInfo,
        /// All legs (inputs to the final vertex)
        input_slots: Vec<usize>,
        /// Slot receiving the amplitude (as WaveformSlot::Scalar)
        result_slot: usize,
    },
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
