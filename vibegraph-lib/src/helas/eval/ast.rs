//! Compiled amplitude evaluator AST types.
//!
//! A `DiagramAst` represents a single Feynman diagram in an efficiently-evaluable form.
//! It stores descriptor types at compile time (external leg info, propagator params, vertex
//! dispatches) and is evaluated at runtime against external momenta + helicity configurations.

use std::ops::{Add, Mul};

use super::dispatch::RootedTerm;
use crate::helas::repr::numbers::Charge;
use crate::helas::repr::{lorentz::LorentzVector, Real, C};
use crate::helas::wavefn::{InDiracWf, OutDiracWf, ScalarWf, VectorWf};
use crate::ufo::couplings::CouplingId;
use crate::ufo::lorentz::LorentzId;
use crate::ufo::particles::ParticleId;
use crate::ufo::vertices::VertexId;
use crate::ufo::UFOModel;

/// A runtime wavefunction register (holds one particle's wavefunction).
///
/// Fermion slots carry their flow direction in the type: a column (ket, `u`/`v`)
/// current is [`WaveformSlot::FermionIn`] and a row (bra, `ū`/`v̄`) current is
/// [`WaveformSlot::FermionOut`]. An off-shell current produced by a `GammaIout`-style
/// node is flow-in; a `GammaJout`-style node is flow-out. Consumers request the flow
/// they need (see [`WaveformSlot::expect_fermion_in`] / [`WaveformSlot::expect_fermion_out`]),
/// applying the Dirac adjoint only when the topology genuinely needs the opposite flow.
#[derive(Clone, Debug, Copy)]
pub enum WaveformSlot<F: Real> {
    /// Flow-in (column / ket) Dirac spinor or off-shell fermion current
    FermionIn(InDiracWf<F>),
    /// Flow-out (row / bra) Dirac spinor or off-shell fermion current
    FermionOut(OutDiracWf<F>),
    /// 4-component polarization / off-shell vector current
    Vector(VectorWf<F>),
    /// Scalar amplitude + momentum
    Scalar(ScalarWf<F>),
    /// Empty slot (not yet computed)
    Empty,
}

impl<F: Real> Add for WaveformSlot<F> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        use WaveformSlot::*;
        match (self, other) {
            (Empty, x) | (x, Empty) => x,
            (Scalar(s1), Scalar(s2)) => {
                assert_eq!(
                    s1.momentum, s2.momentum,
                    "Cannot add scalar waveforms with different momenta"
                );
                WaveformSlot::Scalar(ScalarWf {
                    value: s1.value + s2.value,
                    momentum: s1.momentum,
                })
            }
            (Vector(v1), Vector(v2)) => {
                assert_eq!(
                    v1.momentum, v2.momentum,
                    "Cannot add vector waveforms with different momenta"
                );
                WaveformSlot::Vector(VectorWf {
                    eps: v1.eps + v2.eps,
                    momentum: v1.momentum,
                })
            }
            (FermionIn(f1), FermionIn(f2)) => {
                assert_eq!(
                    f1.momentum, f2.momentum,
                    "Cannot add fermion waveforms with different momenta"
                );
                WaveformSlot::FermionIn(InDiracWf::from_spinor(f1.spinor + f2.spinor, f1.momentum))
            }
            (FermionOut(f1), FermionOut(f2)) => {
                assert_eq!(
                    f1.momentum, f2.momentum,
                    "Cannot add fermion waveforms with different momenta"
                );
                WaveformSlot::FermionOut(OutDiracWf::from_spinor(
                    f1.spinor + f2.spinor,
                    f1.momentum,
                ))
            }
            _ => panic!("Addition only implemented for matching waveform variants"),
        }
    }
}

impl<F> Mul<WaveformSlot<F>> for C<F>
where
    F: Real,
{
    type Output = WaveformSlot<F>;

    fn mul(self, rhs: WaveformSlot<F>) -> WaveformSlot<F> {
        use WaveformSlot::*;
        match rhs {
            Empty => Empty,
            Scalar(s) => Scalar(ScalarWf {
                value: self * s.value,
                momentum: s.momentum,
            }),
            Vector(v) => Vector(VectorWf {
                eps: v.eps * self,
                momentum: v.momentum,
            }),
            FermionIn(f) => FermionIn(InDiracWf::from_spinor(f.spinor * self, f.momentum)),
            FermionOut(f) => FermionOut(OutDiracWf::from_spinor(f.spinor * self, f.momentum)),
        }
    }
}

impl<F: Real> WaveformSlot<F> {
    pub fn momentum(&self) -> Option<LorentzVector<F>> {
        match self {
            WaveformSlot::FermionIn(f) => Some(f.momentum),
            WaveformSlot::FermionOut(f) => Some(f.momentum),
            WaveformSlot::Vector(v) => Some(v.momentum),
            WaveformSlot::Scalar(s) => Some(s.momentum),
            WaveformSlot::Empty => None,
        }
    }

    /// Extract a flow-in (column / ket) fermion, applying the Dirac adjoint if
    /// the slot holds a flow-out current (the topology asked for the opposite flow).
    pub fn expect_fermion_in(self) -> InDiracWf<F> {
        match self {
            WaveformSlot::FermionIn(f) => f,
            // A fermion line carries one flow throughout. With flow-typed externals
            // (`build_external_slot`) and flow-preserving currents, the flow a
            // consumer needs always matches the slot — a flow-out slot here means
            // the dispatch mis-assigned the flow, so panic instead of silently
            // applying a (physically wrong) mid-line Dirac adjoint.
            WaveformSlot::FermionOut(_) => {
                panic!("expect_fermion_in: slot is flow-OUT (fermion-flow mismatch)")
            }
            _ => panic!("expected a fermion waveform slot"),
        }
    }

    /// Extract a flow-out (row / bra) fermion, applying the Dirac adjoint if
    /// the slot holds a flow-in current (the topology asked for the opposite flow).
    pub fn expect_fermion_out(self) -> OutDiracWf<F> {
        match self {
            WaveformSlot::FermionOut(f) => f,
            // See expect_fermion_in: flow is an enforced invariant, not coerced.
            WaveformSlot::FermionIn(_) => {
                panic!("expect_fermion_out: slot is flow-IN (fermion-flow mismatch)")
            }
            _ => panic!("expected a fermion waveform slot"),
        }
    }
}

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
                super::dispatch::root_term(term, &lorentz.spins, result_leg_idx)
                    .expect("Unable to root term from Lorentz expression")
            })
            .collect();

        VertexTerm {
            terms,
            spins: lorentz.spins.clone(),
            coupling_id,
        }
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
