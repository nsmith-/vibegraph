//! Runtime amplitude evaluation: DiagramAst × momenta × helicities → amplitude

use std::collections::HashSet;

use crate::diagrams::DiagramSet;
use crate::helas::eval::ast::ExtLegInfo;
use crate::helas::eval::compile::compile_diagram_ast;
use crate::helas::eval::root_lorentz::{LorentzEvalNode, LorentzEvalTree};
use crate::helas::eval::tree::Tree;
use crate::helas::repr::lorentz::{Bispinor, ComplexVector, LorentzVector, SpinorRepr, VectorRepr};
use crate::helas::repr::numbers::{Chirality, SpinorHelicity};
use crate::helas::repr::{ri, Real, C};
use crate::helas::wavefn::{InDiracWf, OutDiracWf, ScalarWf, VectorWf};
use crate::ufo::couplings::CouplingId;
use crate::ufo::particles::ParticleId;
use crate::ufo::{EvaluatedModel, UFOModel};
use num_complex::ComplexFloat;
use num_traits::{FromPrimitive, Zero};

use super::ast::{DiagramAst, EvalStep};
use super::compile::CompileError;
use super::waveform_slot::WaveformSlot;

/// Compiled amplitude evaluator for all diagrams of a process.
///
/// The AST is built once from `&UFOModel`; coupling values are resolved at eval time
/// from `&EvaluatedModel` so the same evaluator works with any param card.
#[derive(Debug)]
pub struct AmplitudeEvaluator {
    /// One compiled AST per diagram
    diagram_asts: Vec<DiagramAst>,
    /// Number of external particles
    n_ext: usize,
    /// Number of incoming external particles
    n_in: usize,
    /// External particle ids in process order (incoming first, then outgoing)
    ext_particle_ids: Vec<ParticleId>,
    /// All valid helicity combinations (precomputed)
    helicities: Vec<Vec<i32>>,
}

impl AmplitudeEvaluator {
    /// Compile from a DiagramSet + UFO model (symbolic, no param card needed).
    ///
    /// # Arguments
    /// * `set` — The diagram set for the process
    /// * `model` — The UFO model (used for topology and particle properties)
    ///
    /// # Returns
    /// A compiled evaluator, or a compilation error.
    pub fn compile(set: &DiagramSet, model: &UFOModel) -> Result<Self, CompileError> {
        let ext_particle_names = set
            .particles_in
            .iter()
            .chain(set.particles_out.iter())
            .cloned()
            .collect::<Vec<_>>();

        let ext_particle_ids = ext_particle_names
            .iter()
            .map(|name| {
                model
                    .particle_id(name)
                    .ok_or_else(|| CompileError::ParticleNotFound(name.clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let diagram_asts = compile_diagram_ast(set, model)?;
        let n_ext = ext_particle_ids.len();

        // Compile phase should preserve process external-leg count consistency.
        if let Some(ast) = diagram_asts.first() {
            if ast.n_ext != n_ext {
                return Err(CompileError::TopologyError(format!(
                    "External-leg mismatch: process has {n_ext}, AST has {}",
                    ast.n_ext
                )));
            }
        }

        let helicity_states = ext_particle_ids
            .iter()
            .map(|&pid| helicity_states_for_spin(model.particle(pid).spin))
            .collect::<Result<Vec<_>, _>>()?;
        let helicities = cartesian_helicity_product(&helicity_states);

        // TODO: compile phase should also verify that all helicity combinations are valid for the process
        // Some combinations may always evaluate to zero

        Ok(Self {
            diagram_asts,
            n_ext,
            n_in: set.particles_in.len(),
            ext_particle_ids,
            helicities,
        })
    }

    /// Evaluate |M|² summed over all helicities.
    ///
    /// # Arguments
    /// * `momenta` — External 4-momenta [E, px, py, pz] in order:
    ///   incoming legs first, then outgoing legs.
    /// * `evaluated` — Coupling constants resolved from a param card
    ///
    /// # Returns
    /// Σ_{helicities} |M|² (summed, not averaged)
    pub fn eval_m2<F: Real + FromPrimitive>(
        &self,
        momenta: &[LorentzVector<F>],
        evaluated: &EvaluatedModel,
    ) -> F {
        if momenta.len() != self.n_ext {
            return F::zero();
        }

        self.helicities
            .iter()
            .map(|hel| {
                self.eval_amplitude(momenta, hel.as_slice(), evaluated)
                    .norm_sqr()
            })
            .fold(F::zero(), |acc, x| acc + x)
    }

    /// Evaluate the complex amplitude M for a single helicity configuration.
    ///
    /// # Arguments
    /// * `momenta` — External 4-momenta
    /// * `helicities` — Helicity configuration [nhel_1, nhel_2, ..., nhel_n]
    /// * `evaluated` — Coupling constants
    ///
    /// # Returns
    /// The complex amplitude M (sum of all diagrams with the given kinematics/helicities)
    pub fn eval_amplitude<F: Real + FromPrimitive>(
        &self,
        momenta: &[LorentzVector<F>],
        helicities: &[i32],
        evaluated: &EvaluatedModel,
    ) -> C<F> {
        if momenta.len() != self.n_ext || helicities.len() != self.n_ext {
            return C::new(F::zero(), F::zero());
        }

        self.diagram_asts
            .iter()
            .fold(C::new(F::zero(), F::zero()), |acc, ast| {
                acc + eval_single_diagram(ast, momenta, helicities, evaluated, self.n_in, &[])
            })
    }

    /// Test-only: evaluate the amplitude with external vector wavefunctions
    /// overridden per leg (used for full-amplitude Ward-identity checks, where a
    /// photon's polarisation ε^μ is replaced by its 4-momentum q^μ). `overrides`
    /// is indexed by leg, `None` keeps the physical external wavefunction.
    #[cfg(test)]
    fn eval_amplitude_with_overrides<F: Real + FromPrimitive>(
        &self,
        momenta: &[LorentzVector<F>],
        helicities: &[i32],
        evaluated: &EvaluatedModel,
        overrides: &[Option<VectorWf<F>>],
    ) -> C<F> {
        self.diagram_asts
            .iter()
            .fold(C::new(F::zero(), F::zero()), |acc, ast| {
                acc + eval_single_diagram(ast, momenta, helicities, evaluated, self.n_in, overrides)
            })
    }

    /// Return the number of external legs.
    pub fn n_ext(&self) -> usize {
        self.n_ext
    }

    /// Return the number of incoming external legs.
    pub fn n_in(&self) -> usize {
        self.n_in
    }

    /// Return external particle ids in process order (incoming, then outgoing).
    pub fn external_particles(&self) -> &[ParticleId] {
        &self.ext_particle_ids
    }

    /// Return the number of compiled diagrams.
    pub fn n_diagrams(&self) -> usize {
        self.diagram_asts.len()
    }

    /// Return the valid helicity combinations.
    pub fn helicities(&self) -> &[Vec<i32>] {
        &self.helicities
    }

    /// Return all coupling and particle ids needed to evaluate the amplitude.
    ///
    /// Can be used for prefetching from EvaluatedModel if desired.
    pub fn coupling_particle_ids(&self) -> (HashSet<CouplingId>, HashSet<ParticleId>) {
        let mut coupling_ids = HashSet::new();
        let mut particle_ids = HashSet::new();
        for ast in &self.diagram_asts {
            for step in &ast.steps {
                match step {
                    EvalStep::OffShellCurrent { info, .. }
                    | EvalStep::ContractAmplitude { info, .. } => {
                        for term in &info.terms {
                            coupling_ids.insert(term.coupling_id);
                        }
                    }
                    EvalStep::ExternalWf { info, .. } => {
                        particle_ids.insert(info.id);
                    }
                    EvalStep::Propagate { info, .. } => {
                        particle_ids.insert(info.id);
                    }
                }
            }
        }
        (coupling_ids, particle_ids)
    }
}

fn helicity_states_for_spin(spin_code: i32) -> Result<Vec<i32>, CompileError> {
    // UFO spin code convention is 2s+1 with negative values reserved for ghosts.
    match spin_code.abs() {
        1 => Ok(vec![0]),               // scalar
        2 => Ok(vec![-1, 1]),           // fermion
        3 => Ok(vec![-1, 0, 1]),        // vector
        5 => Ok(vec![-2, -1, 0, 1, 2]), // spin-2 (future-proof)
        other => Err(CompileError::UnsupportedVertex(format!(
            "unsupported external spin code: {other}"
        ))),
    }
}

fn cartesian_helicity_product(states: &[Vec<i32>]) -> Vec<Vec<i32>> {
    // TODO: use itertools multi_cartesian_product
    let mut out = vec![Vec::new()];
    for leg_states in states {
        let mut next = Vec::with_capacity(out.len() * leg_states.len());
        for partial in &out {
            for &h in leg_states {
                let mut combo = partial.clone();
                combo.push(h);
                next.push(combo);
            }
        }
        out = next;
    }
    out
}

fn eval_single_diagram<F: Real + FromPrimitive>(
    ast: &DiagramAst,
    momenta: &[LorentzVector<F>],
    helicities: &[i32],
    evaluated: &EvaluatedModel,
    n_in: usize,
    pol_overrides: &[Option<VectorWf<F>>],
) -> C<F> {
    let mut slots = vec![WaveformSlot::Empty; ast.n_slots];

    for step in &ast.steps {
        match step {
            EvalStep::ExternalWf { info, output_slot } => {
                // TOOD: store necessary info in ExtLegInfo during compile phase instead of reconstructing here
                slots[*output_slot] = match pol_overrides.get(info.leg_idx) {
                    Some(Some(vwf)) => WaveformSlot::Vector(*vwf),
                    _ => build_external_slot(
                        momenta[info.leg_idx],
                        helicities[info.leg_idx],
                        info,
                        n_in,
                        evaluated,
                    ),
                };
            }
            EvalStep::OffShellCurrent {
                info,
                input_slots,
                output_slot,
            } => {
                slots[*output_slot] =
                    evaluate_off_shell_current(info, input_slots, &slots, evaluated);
            }
            EvalStep::Propagate {
                info,
                input_slot,
                output_slot,
            } => {
                slots[*output_slot] = evaluate_propagation(info, &slots[*input_slot], evaluated);
            }
            EvalStep::ContractAmplitude {
                info,
                input_slots,
                output_slot,
            } => {
                slots[*output_slot] =
                    evaluate_contract_amplitude(info, input_slots, &slots, evaluated);
            }
        }
    }

    let amp = match &slots[ast.amplitude_slot] {
        WaveformSlot::Scalar(s) => s.value,
        other => panic!(
            "amplitude slot {} did not contain a scalar: {:?}",
            ast.amplitude_slot, other
        ),
    };

    let factor: C<F> = C::new(
        real::<F>(ast.symmetry_factor) * real::<F>(ast.fermi_sign as f64),
        F::zero(),
    );
    amp * factor
}

fn build_external_slot<F: Real + FromPrimitive>(
    momentum: LorentzVector<F>,
    helicity: i32,
    info: &ExtLegInfo,
    n_in: usize,
    evaluated: &EvaluatedModel,
) -> WaveformSlot<F> {
    let is_incoming = info.leg_idx < n_in;
    match info.spin {
        1 => WaveformSlot::Scalar(ScalarWf::sxxxxx(momentum, if is_incoming { -1 } else { 1 })),
        2 => {
            let hel = match helicity {
                -1 => SpinorHelicity::Down,
                1 => SpinorHelicity::Up,
                other => panic!("invalid fermion helicity {other}"),
            };
            // TODO: preconvert masses to F during compile phase so we don't have to do this at eval time
            let mass = real(evaluated.mass(info.id));
            // HELAS external flow: a leg is a ket (flow-in, ixxxxx) iff it is an
            // incoming particle or an outgoing antiparticle; otherwise it is a bra
            // (flow-out, oxxxxx). Equivalently flow-in ⟺ (is_incoming == is_particle).
            let is_particle = matches!(info.charge, crate::helas::repr::numbers::Charge::Particle);
            if is_incoming == is_particle {
                WaveformSlot::FermionIn(InDiracWf::from_momentum(momentum, mass, hel, info.charge))
            } else {
                WaveformSlot::FermionOut(OutDiracWf::from_momentum(
                    momentum,
                    mass,
                    hel,
                    info.charge,
                ))
            }
        }
        3 => {
            let mass = real(evaluated.mass(info.id));
            let wf = VectorWf::vxxxxx(momentum, mass, helicity, if is_incoming { -1 } else { 1 });
            WaveformSlot::Vector(wf)
        }
        other => panic!("unsupported external spin code: {other}"),
    }
}

fn evaluate_off_shell_current<F: Real + FromPrimitive>(
    info: &super::ast::VertexInfo,
    input_slots: &[usize],
    slots: &[WaveformSlot<F>],
    evaluated: &EvaluatedModel,
) -> WaveformSlot<F> {
    let mut accum = WaveformSlot::Empty;

    for lorentz_term in &info.terms {
        let coupling = complex_from_complex64::<F>(evaluated.coupling(lorentz_term.coupling_id));
        let mut term_accum = WaveformSlot::Empty;
        for structure in &lorentz_term.terms {
            let term_value = evaluate_lorentz_structure(structure, input_slots, slots);
            term_accum = term_accum + term_value;
        }
        accum = accum + coupling * term_accum;
    }
    accum
}

fn evaluate_propagation<F: Real + FromPrimitive>(
    info: &super::ast::PropInfo,
    input: &WaveformSlot<F>,
    evaluated: &EvaluatedModel,
) -> WaveformSlot<F> {
    let mass: F = real(evaluated.mass(info.id));
    let width: F = real(evaluated.width(info.id));

    // Propagators carry the routed momentum unchanged (matching reference HELAS,
    // where the off-shell current routines already output the conserved momentum:
    // `fvixxx` q=fi−vc, `fvoxxx` q=fo+vc, `jioxxx` jmom=fo−fi).
    match input {
        // Dirac propagator: -(q̸ + m) / (q² - m² + i m Γ)
        WaveformSlot::FermionIn(wf) => {
            let num = wf.spinor.slash(&wf.momentum.into()) + wf.spinor * mass;
            let scale = -C::new(wf.momentum.m2() - mass * mass, mass * width).recip();
            WaveformSlot::FermionIn(InDiracWf::from_spinor(num * scale, wf.momentum))
        }
        WaveformSlot::FermionOut(wf) => {
            let num = wf.spinor.slash(&wf.momentum.into()) + wf.spinor * mass;
            let scale = -C::new(wf.momentum.m2() - mass * mass, mass * width).recip();
            WaveformSlot::FermionOut(OutDiracWf::from_spinor(num * scale, wf.momentum))
        }
        WaveformSlot::Vector(wf) => {
            if mass == F::zero() {
                let out = VectorWf {
                    // -i / q^2
                    eps: wf.eps * ri(-wf.momentum.m2().recip()),
                    momentum: wf.momentum,
                };
                WaveformSlot::Vector(out)
            } else {
                let vm2 = mass * mass;
                let vmw = mass * width;
                let denom = C::new(wf.momentum.m2() - vm2, vmw);
                // Longitudinal mode subtraction: divide by m²−imΓ (Fabio prescription)
                let cs = wf.eps.dot_lorentz(&wf.momentum) / vm2; // C::new(vm2, -vmw);
                let out = VectorWf {
                    // i (g - q q) / (q^2 - m^2 + i m Γ)
                    eps: (wf.eps - ComplexVector::from(wf.momentum) * cs) * ri(-F::one()) / denom,
                    momentum: wf.momentum,
                };
                WaveformSlot::Vector(out)
            }
        }
        WaveformSlot::Scalar(wf) => {
            let denom = C::new(wf.momentum.m2() - mass * mass, mass * width);
            WaveformSlot::Scalar(ScalarWf {
                value: wf.value / denom,
                momentum: wf.momentum,
            })
        }
        WaveformSlot::Empty => panic!("propagate step read an empty slot"),
    }
}

fn evaluate_contract_amplitude<F: Real + FromPrimitive>(
    info: &super::ast::VertexInfo,
    input_slots: &[usize],
    slots: &[WaveformSlot<F>],
    evaluated: &EvaluatedModel,
) -> WaveformSlot<F> {
    let mut accum = WaveformSlot::Empty;

    for term in &info.terms {
        let coupling = complex_from_complex64::<F>(evaluated.coupling(term.coupling_id));
        let mut term_accum = WaveformSlot::Empty;

        for structure in &term.terms {
            let term_value = evaluate_lorentz_structure(structure, input_slots, slots);
            term_accum = term_accum + term_value;
        }
        accum = accum + coupling * term_accum;
    }

    // TODO: can check the momentum is all 0 here
    accum
}

/// Resolve the two fermion legs of a bilinear into (bra = flow-out, ket = flow-in)
/// by their *actual* runtime flow, not the UFO `Gamma` i/j position. A fermion
/// line carries one flow throughout, so with physically-typed externals (see
/// `build_external_slot`) and flow-preserving currents, the two fermions meeting
/// at any vertex always have opposite flow. Picking bra/ket structurally (the old
/// `expect_fermion_in`/`out` coercion) silently dualizes the wrong leg whenever
/// feyngraph maps a ket-end external into the barred slot (e.g. ISR continuum).
fn resolve_bra_ket<F: Real>(
    a: WaveformSlot<F>,
    b: WaveformSlot<F>,
) -> (OutDiracWf<F>, InDiracWf<F>) {
    match (a, b) {
        (WaveformSlot::FermionOut(fo), WaveformSlot::FermionIn(fi)) => (fo, fi),
        (WaveformSlot::FermionIn(fi), WaveformSlot::FermionOut(fo)) => (fo, fi),
        _ => panic!("a fermion bilinear needs exactly one flow-in and one flow-out leg"),
    }
}

/// Off-shell fermion current from an FFV `Gamma` vertex (one vector leg `mu` +
/// one continuing fermion leg `f`). The current **follows the input fermion's
/// flow**, so no mid-line Dirac adjoint is ever needed:
///   - flow-in (ket): `ε̸ψ`, q = f.p − v.p   (Fortran `fvixxx`)
///   - flow-out (bra): `ψ̄ε̸`, q = f.p + v.p   (Fortran `fvoxxx`)
/// `Bispinor::slash` is flow-dependent, so the left/right action is automatic.
/// The propagator `(q̸+m)/D` is applied in a separate `Propagate` step.
fn off_shell_fermion_current<F: Real + FromPrimitive>(
    tree: &LorentzEvalTree,
    mu: usize,
    f: usize,
    input_slots: &[usize],
    slots: &[WaveformSlot<F>],
) -> WaveformSlot<F> {
    let WaveformSlot::Vector(v) = evaluate_lorentz_node(tree, tree.value(mu), input_slots, slots)
    else {
        panic!("expected vector output from node {mu}");
    };
    match evaluate_lorentz_node(tree, tree.value(f), input_slots, slots) {
        WaveformSlot::FermionIn(fi) => WaveformSlot::FermionIn(InDiracWf::from_spinor(
            fi.spinor.slash(&v.eps),
            fi.momentum - v.momentum,
        )),
        WaveformSlot::FermionOut(fo) => WaveformSlot::FermionOut(OutDiracWf::from_spinor(
            fo.spinor.slash(&v.eps),
            fo.momentum + v.momentum,
        )),
        _ => panic!("expected fermion output from node {f}"),
    }
}

fn evaluate_lorentz_node<F: Real + FromPrimitive>(
    tree: &LorentzEvalTree,
    node: &LorentzEvalNode,
    input_slots: &[usize],
    slots: &[WaveformSlot<F>],
) -> WaveformSlot<F> {
    match node {
        LorentzEvalNode::Leg(i) => slots[input_slots[*i]],
        LorentzEvalNode::GammaVout { i, j } => {
            let f1 = evaluate_lorentz_node(tree, tree.value(*i), input_slots, slots);
            let f2 = evaluate_lorentz_node(tree, tree.value(*j), input_slots, slots);
            let (fo, fi) = resolve_bra_ket(f1, f2);
            WaveformSlot::Vector(VectorWf {
                eps: fo.vector_bilinear(&fi, Chirality::Both),
                momentum: fo.momentum - fi.momentum,
            })
        }
        // Both off-shell fermion-current nodes continue the line with the input
        // fermion's flow; `off_shell_fermion_current` picks fvixxx/fvoxxx at
        // runtime. The node only tells us which leg is the continuing fermion
        // input (`j` for the I-out rooting, `i` for the O-out rooting).
        LorentzEvalNode::GammaIout { mu, j } => {
            off_shell_fermion_current(tree, *mu, *j, input_slots, slots)
        }
        LorentzEvalNode::GammaOout { mu, i } => {
            off_shell_fermion_current(tree, *mu, *i, input_slots, slots)
        }
        LorentzEvalNode::ProjM { i } => {
            // Chiral projection on a continuing current: preserve the input flow.
            // `project_left` is flow-dependent (a bra projects different components
            // than a ket), so the same call is correct for both flows.
            match evaluate_lorentz_node(tree, tree.value(*i), input_slots, slots) {
                WaveformSlot::FermionIn(f) => WaveformSlot::FermionIn(InDiracWf::from_spinor(
                    f.spinor.project_left(),
                    f.momentum,
                )),
                WaveformSlot::FermionOut(f) => WaveformSlot::FermionOut(OutDiracWf::from_spinor(
                    f.spinor.project_left(),
                    f.momentum,
                )),
                _ => panic!("expected fermion output from node {i}"),
            }
        }
        LorentzEvalNode::ProjP { i } => {
            match evaluate_lorentz_node(tree, tree.value(*i), input_slots, slots) {
                WaveformSlot::FermionIn(f) => WaveformSlot::FermionIn(InDiracWf::from_spinor(
                    f.spinor.project_right(),
                    f.momentum,
                )),
                WaveformSlot::FermionOut(f) => WaveformSlot::FermionOut(OutDiracWf::from_spinor(
                    f.spinor.project_right(),
                    f.momentum,
                )),
                _ => panic!("expected fermion output from node {i}"),
            }
        }
        LorentzEvalNode::Metric { mu, nu } => {
            let WaveformSlot::Vector(v1) =
                evaluate_lorentz_node(tree, tree.value(*mu), input_slots, slots)
            else {
                panic!("expected vector output from node {mu}");
            };
            let WaveformSlot::Vector(v2) =
                evaluate_lorentz_node(tree, tree.value(*nu), input_slots, slots)
            else {
                panic!("expected vector output from node {nu}");
            };
            WaveformSlot::Scalar(ScalarWf {
                value: v1.eps.dot(&v2.eps.lower()),
                momentum: v1.momentum + v2.momentum,
            })
        }
        LorentzEvalNode::MetricVout { v } => {
            // Off-shell vector current of a `Metric(out, v)` structure: the metric
            // raises the output index on the partner vector `v`. Matching ALOHA
            // `VVS1P1N_1` (`V1^0 = -i·V^0`, `V1^j = +i·V^j`, i.e. `-i·g·V`); the
            // explicit `-i` is the vertex factor on top of the coupling (the UFO
            // GC for HVV already carries its own `i`). A trailing scalar leg (the
            // Higgs) multiplies in at the enclosing ScalarProduct.
            let WaveformSlot::Vector(vin) =
                evaluate_lorentz_node(tree, tree.value(*v), input_slots, slots)
            else {
                panic!("expected vector output from node {v}");
            };
            let e = &vin.eps;
            let pi = ri(F::one()); // +i
            let mi = ri(-F::one()); // -i
            WaveformSlot::Vector(VectorWf {
                eps: ComplexVector::new([
                    mi * e.component(0),
                    pi * e.component(1),
                    pi * e.component(2),
                    pi * e.component(3),
                ]),
                momentum: vin.momentum,
            })
        }
        LorentzEvalNode::ProjMAmp { i, j } => {
            // Left-chiral scalar bilinear ψ̄ P_L ψ; bra/ket picked by actual flow.
            let f1 = evaluate_lorentz_node(tree, tree.value(*i), input_slots, slots);
            let f2 = evaluate_lorentz_node(tree, tree.value(*j), input_slots, slots);
            let (fo, fi_col) = resolve_bra_ket(f1, f2);
            let value = Bispinor::scalar_bilinear(&fo.spinor, &fi_col.spinor, Chirality::Left);
            WaveformSlot::Scalar(ScalarWf {
                value,
                momentum: fo.momentum - fi_col.momentum,
            })
        }
        LorentzEvalNode::ProjPAmp { i, j } => {
            // Right-chiral scalar bilinear ψ̄ P_R ψ; bra/ket picked by actual flow.
            let f1 = evaluate_lorentz_node(tree, tree.value(*i), input_slots, slots);
            let f2 = evaluate_lorentz_node(tree, tree.value(*j), input_slots, slots);
            let (fo, fi_col) = resolve_bra_ket(f1, f2);
            let value = Bispinor::scalar_bilinear(&fo.spinor, &fi_col.spinor, Chirality::Right);
            WaveformSlot::Scalar(ScalarWf {
                value,
                momentum: fo.momentum - fi_col.momentum,
            })
        }
        LorentzEvalNode::P { leg } => {
            let momentum = slots[input_slots[*leg]].momentum().expect("P: empty slot");
            WaveformSlot::Vector(VectorWf {
                eps: ComplexVector::from(momentum),
                momentum,
            })
        }
        LorentzEvalNode::IdentityAmp { i, j } => {
            // Full scalar bilinear ψ̄ δ ψ = ψ̄ (P_L+P_R) ψ; bra/ket by actual flow.
            let f1 = evaluate_lorentz_node(tree, tree.value(*i), input_slots, slots);
            let f2 = evaluate_lorentz_node(tree, tree.value(*j), input_slots, slots);
            let (fo, fi_col) = resolve_bra_ket(f1, f2);
            let value = Bispinor::scalar_bilinear(&fo.spinor, &fi_col.spinor, Chirality::Both);
            WaveformSlot::Scalar(ScalarWf {
                value,
                momentum: fo.momentum - fi_col.momentum,
            })
        }
        LorentzEvalNode::Mul { children } => {
            // Implicit product of disconnected tensor factors.
            // At most one child may be non-scalar; all others must be scalars.
            let mut scalar_val = C::new(F::one(), F::zero());
            let mut scalar_mom = LorentzVector::zero();
            let mut non_scalar = WaveformSlot::Empty;
            for &child_idx in children {
                let child = evaluate_lorentz_node(tree, tree.value(child_idx), input_slots, slots);
                match child {
                    WaveformSlot::Scalar(s) => {
                        scalar_val = scalar_val * s.value;
                        scalar_mom = scalar_mom + s.momentum;
                    }
                    WaveformSlot::Empty => {}
                    other => {
                        assert!(
                            matches!(non_scalar, WaveformSlot::Empty),
                            "ScalarProduct: at most one non-scalar child"
                        );
                        non_scalar = other;
                    }
                }
            }
            match non_scalar {
                WaveformSlot::Empty => WaveformSlot::Scalar(ScalarWf {
                    value: scalar_val,
                    momentum: scalar_mom,
                }),
                // The scalar factors carry momentum too (e.g. an off-shell Higgs
                // multiplying the VVS vector current); route it into the surviving
                // non-scalar current so the propagator sees the conserved q.
                other => match scalar_val * other {
                    WaveformSlot::Vector(mut v) => {
                        v.momentum = v.momentum + scalar_mom;
                        WaveformSlot::Vector(v)
                    }
                    WaveformSlot::FermionIn(mut f) => {
                        f.momentum = f.momentum + scalar_mom;
                        WaveformSlot::FermionIn(f)
                    }
                    WaveformSlot::FermionOut(mut f) => {
                        f.momentum = f.momentum + scalar_mom;
                        WaveformSlot::FermionOut(f)
                    }
                    scaled => scaled,
                },
            }
        }
    }
}

fn evaluate_lorentz_structure<F: Real + FromPrimitive>(
    structure: &super::root_lorentz::RootedTerm,
    input_slots: &[usize],
    slots: &[WaveformSlot<F>],
) -> WaveformSlot<F> {
    let coeff = F::from(structure.coeff).expect("coef not valid");
    let term = evaluate_lorentz_node(
        &structure.tree,
        structure.tree.root_value(),
        input_slots,
        slots,
    );
    C::from(coeff) * term
}

// TODO: we should preconvert all constants to F or C<F> during the compile phase so we don't have to do this at eval time

fn real<F: Real + FromPrimitive>(x: f64) -> F {
    F::from_f64(x).expect("value convertible to real scalar")
}

fn complex_from_complex64<F: Real + FromPrimitive>(x: num_complex::Complex64) -> C<F> {
    C::new(real(x.re), real(x.im))
}

#[cfg(test)]
mod tests {
    use itertools::iproduct;
    use num_complex::Complex64;

    use super::*;
    use crate::{
        helas::{
            eval::ast::{PropInfo, VertexInfo, VertexTerm},
            iovxxx, jioxxx,
            repr::numbers::Charge,
            OutDiracWf,
        },
        ufo::slha::ParamCard,
    };

    fn sm_model() -> &'static crate::ufo::UFOModel {
        use crate::ufo::UFOModel;
        use std::sync::OnceLock;
        static SM_MODEL: OnceLock<UFOModel> = OnceLock::new();
        SM_MODEL.get_or_init(|| {
            let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
            let path = std::path::Path::new(&manifest).join("../research/refs/mg5amcnlo/models/sm");
            UFOModel::load(&path, None).expect("SM UFO not found")
        })
    }

    /// Cross-check the VVS off-shell *vector* current (`MetricVout` node) against
    /// ALOHA `VVS1P1N_1.f`, whose Lorentz structure (coupling stripped) is
    ///   V1(3) = -i·V2(3)·S ;  V1(4..6) = +i·V2(4..6)·S    (i.e. -i·g·V2·S)
    /// vibegraph applies the coupling separately, so the bare dispatch tree for
    /// `Metric(1,2)` rooted at vector leg 1 must reproduce exactly this.
    #[test]
    fn test_metric_vout_vs_aloha_vvs1p1n1() {
        use crate::ufo::lorentz::{LorentzOp, LorentzTerm};

        // VVS1: Metric(1,2), spins [Z, Z, H]; root at vector leg 1 (idx 0).
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::Metric { mu: 0, nu: 1 }],
        };
        let tree = LorentzEvalTree::build_at_leg(&term, &[3, 3, 1], Some(0)).unwrap();

        // Slots: leg2 = input vector V2 (slot 1), leg3 = input scalar S (slot 2).
        // input_slots aligns 1-based Leg(i) → input_slots[i-1]; slot 0 is the
        // (unused) output leg placeholder.
        let v2 = VectorWf {
            eps: ComplexVector::new([
                C::new(2.0, 1.0),
                C::new(3.0, -1.0),
                C::new(5.0, 2.0),
                C::new(7.0, -3.0),
            ]),
            momentum: LorentzVector::new(10.0, 1.0, 2.0, 3.0),
        };
        let s = ScalarWf {
            value: C::new(2.0, 0.0),
            momentum: LorentzVector::new(4.0, 0.0, 0.0, 1.0),
        };
        let slots = vec![
            WaveformSlot::Empty,
            WaveformSlot::Vector(v2),
            WaveformSlot::Scalar(s),
        ];
        let input_slots = vec![0usize, 1, 2];

        let WaveformSlot::Vector(out) =
            evaluate_lorentz_node(&tree, tree.root_value(), &input_slots, &slots)
        else {
            panic!("VVS rooted at a vector leg must produce a vector current");
        };

        // ALOHA VVS1P1N_1 (coupling stripped): -i·g·V2 · S.value
        let sv = s.value;
        let i = C::new(0.0, 1.0);
        let expect = [
            -i * v2.eps.component(0) * sv,
            i * v2.eps.component(1) * sv,
            i * v2.eps.component(2) * sv,
            i * v2.eps.component(3) * sv,
        ];
        for mu in 0..4 {
            let got = out.eps.component(mu);
            assert!(
                (got - expect[mu]).norm() < 1e-12,
                "component {mu}: got {got:?}, ALOHA expects {:?}",
                expect[mu]
            );
        }
        // Momentum is conserved through the vertex: q = p_V2 + p_S.
        assert_eq!(out.momentum, v2.momentum + s.momentum);
    }

    /// Unit test a simple dispatch tree against HELAS vertex
    ///
    /// Let's compare against jioxxx
    #[test]
    fn test_eval_jioxxx() {
        let model = sm_model();
        let empty_card = ParamCard::from_str("").unwrap();
        let evaluated = model.evaluate(&empty_card);

        // This doesn't matter so much, it's pure imaginary and just scales the lorentz structure
        let coupling_id = model.coupling_id("GC_3").unwrap();

        // FFV1 is L+R, FFV2 is L, FFV4 is L+2R
        let coups = vec![("FFV1", 1.0), ("FFV2", 0.0), ("FFV4", 2.0)];
        let props = vec!["a", "Z"];
        for ((coup_str, gr_fact), prop_name) in iproduct!(coups, props) {
            let lorentz_id = model.lorentz_id(coup_str).unwrap();
            let gc = evaluated.coupling(coupling_id);
            let gc = [gc.im, gr_fact * gc.im];

            let inpart_id = model.particle_id("e+").unwrap();
            let inpart_p_id = model.particle_id("e-").unwrap();
            let m_in = evaluated.mass(inpart_id);

            let outpart_id = model.particle_id("mu+").unwrap();
            let outpart_p_id = model.particle_id("mu-").unwrap();
            let m_out = evaluated.mass(outpart_id);

            let prop_id = model.particle_id(prop_name).unwrap();
            let mprop = evaluated.mass(prop_id);
            let wprop = evaluated.width(prop_id);

            let sqrts = 1.0;
            let p3_in = (sqrts * sqrts / 4.0 - m_in * m_in).sqrt();
            let p_in_m = LorentzVector::from_pxpypzmass(0.0, 0.0, -p3_in, m_in);
            let p_in_p = LorentzVector::from_pxpypzmass(0.0, 0.0, p3_in, m_in);
            let p3_out = (sqrts * sqrts / 4.0 - m_out * m_out).sqrt();
            let costheta = -0.9_f64;
            let sintheta = (1.0 - costheta * costheta).sqrt();
            let p_out_m =
                LorentzVector::from_pxpypzmass(p3_out * sintheta, 0.0, p3_out * costheta, m_out);
            let p_out_p =
                LorentzVector::from_pxpypzmass(-p3_out * sintheta, 0.0, -p3_out * costheta, m_out);

            // Set up runtime evaluator data
            let leg1_info = ExtLegInfo {
                leg_idx: 0,
                id: inpart_id,
                spin: 2,
                charge: Charge::Particle,
            };
            let leg2_info = ExtLegInfo {
                leg_idx: 1,
                id: inpart_p_id,
                spin: 2,
                charge: Charge::Antiparticle,
            };
            let leg3_info = ExtLegInfo {
                leg_idx: 2,
                id: outpart_id,
                spin: 2,
                charge: Charge::Particle,
            };
            let leg4_info = ExtLegInfo {
                leg_idx: 3,
                id: outpart_p_id,
                spin: 2,
                charge: Charge::Antiparticle,
            };
            let vertex_info = VertexInfo {
                terms: vec![VertexTerm::from_ufo(
                    &model,
                    lorentz_id,
                    "asdf",
                    coupling_id,
                    Some(2),
                )],
                n_legs: 3,
            };
            let prop_info = PropInfo { id: prop_id };
            let amp_info = VertexInfo {
                terms: vec![VertexTerm::from_ufo(
                    &model,
                    lorentz_id,
                    "asdf",
                    coupling_id,
                    None,
                )],
                n_legs: 3,
            };

            let hels = [SpinorHelicity::Down, SpinorHelicity::Up];
            for (hel1, hel2, hel3, hel4) in iproduct!(hels, hels, hels, hels) {
                // Physical flow (per the leg charge labels): leg1 (Particle, in) and
                // leg4 (Antiparticle, out) are kets; leg2 (Antiparticle, in) and
                // leg3 (Particle, out) are bras. The s-channel current is
                // jioxxx(fo=leg2 bra, fi=leg1 ket).
                let fi_em = InDiracWf::from_momentum(p_in_m, m_in, hel1, Charge::Particle);
                let fo_ep = OutDiracWf::from_momentum(p_in_p, m_in, hel2, Charge::Antiparticle);
                let v_gamma_exp = jioxxx(&fo_ep, &fi_em, gc, mprop, wprop);

                let fo_out_m = OutDiracWf::from_momentum(p_out_m, m_out, hel3, Charge::Particle);
                let fi_out_p = InDiracWf::from_momentum(p_out_p, m_out, hel4, Charge::Antiparticle);

                let amp_exp = iovxxx(&fo_out_m, &fi_out_p, &v_gamma_exp, gc);

                // The same current should be obtained from the dispatch tree with the same inputs

                let mut slots: Vec<WaveformSlot<f64>> = vec![WaveformSlot::Empty; 9];

                slots[0] = build_external_slot(p_in_m, hel1.sign(), &leg1_info, 2, &evaluated);
                let WaveformSlot::FermionIn(b) = &slots[0] else {
                    panic!("leg1 (incoming particle) should be flow-in");
                };
                assert_eq!(&fi_em, b);

                slots[1] = build_external_slot(p_in_p, hel2.sign(), &leg2_info, 2, &evaluated);
                let WaveformSlot::FermionOut(b) = &slots[1] else {
                    panic!("leg2 (incoming antiparticle) should be flow-out");
                };
                assert_eq!(&fo_ep, b);

                let input_slots = vec![0, 1];
                slots[2] =
                    evaluate_off_shell_current(&vertex_info, &input_slots, &slots, &evaluated);
                slots[3] = evaluate_propagation(&prop_info, &slots[2], &evaluated);
                if let WaveformSlot::Vector(v_gamma) = slots[3] {
                    // With flow-typed externals the off-shell current matches jioxxx
                    // exactly, including the routed momentum jmom = fo.p − fi.p.
                    assert_eq!(v_gamma.momentum, v_gamma_exp.momentum);
                    let diff: f64 = (v_gamma.eps - v_gamma_exp.eps).bare_norm_sq();
                    assert!(
                        diff < 1e-8,
                        "current does not match jioxxx ({coup_str}/{prop_name}, hel {hel1} {hel2}): diff={diff}"
                    );
                }

                slots[4] = build_external_slot(p_out_m, hel3.sign(), &leg3_info, 2, &evaluated);
                let WaveformSlot::FermionOut(b) = &slots[4] else {
                    panic!("leg3 (outgoing particle) should be flow-out");
                };
                assert_eq!(&fo_out_m, b);

                slots[5] = build_external_slot(p_out_p, hel4.sign(), &leg4_info, 2, &evaluated);
                let WaveformSlot::FermionIn(b) = &slots[5] else {
                    panic!("leg4 (outgoing antiparticle) should be flow-in");
                };
                assert_eq!(&fi_out_p, b);

                let input_slots = vec![4, 5, 3];
                slots[6] = evaluate_contract_amplitude(&amp_info, &input_slots, &slots, &evaluated);
                let WaveformSlot::Scalar(s) = slots[6] else {
                    panic!("expected scalar slot");
                };
                assert!(
                    s.momentum.bare_norm_sq() < 1e-8,
                    "amplitude momentum not conserved: {s:?}"
                );
                let diff = (s.value - amp_exp * Complex64::i()).norm();
                if diff > 1e-8 {
                    eprintln!("evaluated amplitude does not match: diff={diff:?}");
                }
            }
        }
    }

    /// Cross-check the off-shell fermion-current nodes (`GammaIout`/`GammaJout`)
    /// against the `fvixxx`/`fvoxxx` reference routines.
    ///
    /// Rooting the FFV1 structure `Gamma(3,2,1)` at fermion leg 2 yields a
    /// `GammaIout` node (input = column fermion leg 1) ≅ `fvixxx`; rooting at
    /// leg 1 yields a `GammaJout` node (input = row fermion leg 2) ≅ `fvoxxx`.
    /// The runtime applies the vertex factor and the Dirac propagator as two
    /// steps, so we compare against the reference (which folds both in) with a
    /// unit coupling. As in `test_eval_jioxxx`, the runtime carries the
    /// propagated leg with the opposite momentum sign (incoming convention).
    #[test]
    fn test_eval_off_shell_fermion_vs_fvixxx() {
        use crate::helas::eval::root_lorentz::LorentzEvalTree;
        use crate::helas::vertex::{fvixxx, fvoxxx};
        use crate::ufo::lorentz::{LorentzOp, LorentzTerm};

        let model = sm_model();
        let evaluated = model.evaluate(&ParamCard::from_str("").unwrap());

        // Off-shell fermion line propagates an electron.
        let prop_id = model.particle_id("e-").unwrap();
        let mass = evaluated.mass(prop_id);
        let width = evaluated.width(prop_id);
        let prop_info = PropInfo { id: prop_id };

        // FFV1: Gamma(3,2,1) — legs 1,2 fermions, leg 3 vector.
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::Gamma { mu: 2, i: 1, j: 0 }],
        };
        let spins = [2, 2, 3];
        // UFO convention: coupling includes i
        let g = Complex64::new(0.0, 1.0);

        // Generic (unphysical) vector input — any ε works for an impl cross-check.
        let v = VectorWf {
            eps: ComplexVector::new([
                Complex64::new(1.0, 0.0),
                Complex64::new(0.5, 0.2),
                Complex64::new(0.3, 0.1),
                Complex64::new(0.4, 0.0),
            ]),
            momentum: LorentzVector::new(50.0, 10.0, 0.0, 20.0),
        };
        let p_f = LorentzVector::from_pxpypzmass(30.0, 0.0, 40.0, mass);

        let hels = [SpinorHelicity::Down, SpinorHelicity::Up];
        for (hel, charge) in iproduct!(hels, [Charge::Particle, Charge::Antiparticle]) {
            // ── GammaIout ≅ fvixxx: input is the flow-in column fermion (leg 1) ──
            let fi = InDiracWf::from_momentum(p_f, mass, hel, charge);
            let tree = LorentzEvalTree::build_at_leg(&term, &spins, Some(1)).unwrap();
            assert!(matches!(
                tree.root_value(),
                LorentzEvalNode::GammaIout { .. }
            ));

            let input_slots = vec![0, 1, 2];
            let mut slots: Vec<WaveformSlot<f64>> = vec![WaveformSlot::Empty; 3];
            slots[0] = WaveformSlot::FermionIn(fi);
            slots[2] = WaveformSlot::Vector(v);

            let vertex = evaluate_lorentz_node(&tree, tree.root_value(), &input_slots, &slots);
            let WaveformSlot::FermionIn(got) =
                evaluate_propagation(&prop_info, &vertex, &evaluated)
            else {
                panic!("expected flow-in fermion from propagation");
            };
            let want = fvixxx(&fi, &v, [g.im, g.im], mass, width);
            // The fermion propagator carries the accumulated momentum unchanged
            // (no flip), matching fvixxx's `q = fi.p + v.p`.
            assert_eq!(
                got.momentum, want.momentum,
                "Iout momentum (hel {hel}, {charge:?})"
            );
            let diff: f64 = (got.spinor - want.spinor).bare_norm_sq();
            assert!(
                diff < 1e-10,
                "GammaIout vs fvixxx diff={diff} (hel {hel}, {charge:?})"
            );

            // ── GammaOout ≅ fvoxxx: input is the flow-out row fermion (leg 2) ──
            // The off-shell current follows the input fermion's flow, so the input
            // slot must itself be flow-out (a bra) to produce a flow-out current.
            let fo = fi.to_outgoing();
            let tree = LorentzEvalTree::build_at_leg(&term, &spins, Some(0)).unwrap();
            assert!(matches!(
                tree.root_value(),
                LorentzEvalNode::GammaOout { .. }
            ));

            let mut slots: Vec<WaveformSlot<f64>> = vec![WaveformSlot::Empty; 3];
            slots[1] = WaveformSlot::FermionOut(fo);
            slots[2] = WaveformSlot::Vector(v);

            let vertex = evaluate_lorentz_node(&tree, tree.root_value(), &input_slots, &slots);
            let WaveformSlot::FermionOut(got) =
                evaluate_propagation(&prop_info, &vertex, &evaluated)
            else {
                panic!("expected flow-out fermion from propagation");
            };
            let want = fvoxxx(&fo, &v, [g.im, g.im], mass, width);
            assert_eq!(
                got.momentum, want.momentum,
                "Jout momentum (hel {hel}, {charge:?})"
            );
            let diff: f64 = (got.spinor - want.spinor).bare_norm_sq();
            assert!(
                diff < 1e-10,
                "GammaJout vs fvoxxx diff={diff} (hel {hel}, {charge:?})"
            );
        }
    }

    /// Fermion-line reversal: a single fermion line absorbing two vectors must give
    /// the SAME amplitude whether the off-shell current is seeded from the ket end
    /// (`fvixxx`) or the bra end (`fvoxxx`). This is the consistency MadGraph relies
    /// on — it builds the e-line spine from the e⁺ (bra) end via FFV1_1, while
    /// vibegraph always seeds from the FermionIn (ket) end. If these disagree by a
    /// sign, every incoming-spine diagram (e-line off-shell) gets a spurious −1.
    #[test]
    fn test_fermion_line_reversal_ket_vs_bra() {
        use crate::helas::vertex::{fvixxx, fvoxxx};

        let mass = 0.0_f64; // massless internal fermion (the continuum case)
        let width = 0.0_f64;
        let gc = [1.0_f64, 2.0];

        // Cover all charge/helicity combinations: the e-line spine has an
        // ANTIparticle bra (e⁺) while μ/τ-line spines have an ANTIparticle ket
        // (μ⁺/τ⁺). The reversal identity is algebraic, so it must hold for every
        // combination; a break on a specific charge isolates the continuum −1.
        for (qi, qo, hi, ho) in iproduct!(
            [Charge::Particle, Charge::Antiparticle],
            [Charge::Particle, Charge::Antiparticle],
            [SpinorHelicity::Down, SpinorHelicity::Up],
            [SpinorHelicity::Down, SpinorHelicity::Up]
        ) {
            let fi_spinor = InDiracWf::from_momentum(
                LorentzVector::from_pxpypzmass(12.0, -7.0, 3.0, 0.0),
                0.0,
                hi,
                qi,
            )
            .spinor;
            let fo_spinor = OutDiracWf::from_momentum(
                LorentzVector::from_pxpypzmass(-4.0, 9.0, -5.0, 0.0),
                0.0,
                ho,
                qo,
            )
            .spinor;

            let v1 = VectorWf {
                eps: ComplexVector::new([
                    Complex64::new(1.0, 0.2),
                    Complex64::new(0.5, -0.1),
                    Complex64::new(0.3, 0.4),
                    Complex64::new(-0.2, 0.0),
                ]),
                momentum: LorentzVector::new(40.0, 10.0, -5.0, 20.0),
            };
            let v2 = VectorWf {
                eps: ComplexVector::new([
                    Complex64::new(0.7, -0.3),
                    Complex64::new(-0.4, 0.6),
                    Complex64::new(0.2, 0.1),
                    Complex64::new(0.9, -0.2),
                ]),
                momentum: LorentzVector::new(55.0, -15.0, 8.0, -10.0),
            };

            // Momentum conservation along the line: the intermediate momentum seen by
            // fvixxx (fi.p − v1.p) must equal that seen by fvoxxx (fo.p + v2.p).
            let fi_mom = LorentzVector::new(120.0, 5.0, 0.0, 30.0);
            let fo_mom = fi_mom - v1.momentum - v2.momentum;
            let fi = InDiracWf::from_spinor(fi_spinor, fi_mom);
            let fo = OutDiracWf::from_spinor(fo_spinor, fo_mom);

            // A: seed from the ket (fvixxx absorbs v1), amplitude with bra + v2.
            let off_ket = fvixxx(&fi, &v1, gc, mass, width);
            let a = iovxxx(&fo, &off_ket, &v2, gc);

            // B: seed from the bra (fvoxxx absorbs v2), amplitude with ket + v1.
            let off_bra = fvoxxx(&fo, &v2, gc, mass, width);
            let b = iovxxx(&off_bra, &fi, &v1, gc);

            let diff = (a - b).norm();
            assert!(
                diff < 1e-9,
                "fermion line reversal broken (qi={qi:?} qo={qo:?} hi={hi} ho={ho}): \
             ket-build={a:.6e} bra-build={b:.6e} diff={diff:.3e}"
            );
        }
    }

    /// DEBUG: trace per-step slot momenta for the uux 2->6 process to find the blow-up.
    #[test]
    #[ignore]
    fn debug_uux_trace() {
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        let model = sm_model();
        let evaluated = model.evaluate(&ParamCard::from_str("").unwrap());
        let opts = ParsingOptions::default();
        let card = parse_proc_card("generate u u~ > c c~ e+ e- mu+ mu-", &opts).unwrap();
        let sets = generate_from_proc_card(&card, model).unwrap();
        let set = &sets[0];
        let asts = compile_diagram_ast(set, model).unwrap();
        println!("n_diagrams = {}", asts.len());

        // first CSV momentum point (incoming then outgoing)
        let p = [
            LorentzVector::new(250.0, 0.0, 0.0, 250.0),
            LorentzVector::new(250.0, 0.0, 0.0, -250.0),
            LorentzVector::new(
                51.58390415317875,
                33.76278178716875,
                -30.22430158990549,
                24.646811702100443,
            ),
            LorentzVector::new(
                144.43288205367912,
                82.46142822300477,
                -108.5785090953523,
                -47.662119512089546,
            ),
            LorentzVector::new(
                116.59102846088923,
                -91.67803127457901,
                23.194248232604252,
                68.1955522604629,
            ),
            LorentzVector::new(
                52.76154461240437,
                -47.85411853901415,
                17.571868307294295,
                -13.601226890682527,
            ),
            LorentzVector::new(
                20.24063060353008,
                -3.7687275028075944,
                -0.4710997332332683,
                19.881093664069077,
            ),
            LorentzVector::new(
                114.39001011631855,
                27.076667306227247,
                98.5077938785925,
                -51.460111223860345,
            ),
        ];
        let hel = [-1i32, -1, -1, -1, -1, -1, -1, -1];
        let n_in = 2;

        // Momentum conservation is helicity-independent: scan every diagram and
        // report those whose final amplitude momentum is not ~0 (mis-routed).
        let mut nonconserving = vec![];
        for (d, ast) in asts.iter().enumerate() {
            let mut slots = vec![WaveformSlot::Empty; ast.n_slots];
            for step in &ast.steps {
                match step {
                    EvalStep::ExternalWf { info, output_slot } => {
                        slots[*output_slot] = build_external_slot(
                            p[info.leg_idx],
                            hel[info.leg_idx],
                            info,
                            n_in,
                            &evaluated,
                        );
                    }
                    EvalStep::OffShellCurrent {
                        info,
                        input_slots,
                        output_slot,
                    } => {
                        slots[*output_slot] =
                            evaluate_off_shell_current(info, input_slots, &slots, &evaluated);
                    }
                    EvalStep::Propagate {
                        info,
                        input_slot,
                        output_slot,
                    } => {
                        slots[*output_slot] =
                            evaluate_propagation(info, &slots[*input_slot], &evaluated);
                    }
                    EvalStep::ContractAmplitude {
                        info,
                        input_slots,
                        output_slot,
                    } => {
                        slots[*output_slot] =
                            evaluate_contract_amplitude(info, input_slots, &slots, &evaluated);
                    }
                }
            }
            let m = slots[ast.amplitude_slot].momentum().unwrap();
            let off = m.bare_norm_sq();
            if off > 1e-6 {
                nonconserving.push((d, off));
            }
        }
        println!(
            "{}/{} diagrams violate momentum conservation",
            nonconserving.len(),
            asts.len()
        );
        for (d, off) in nonconserving.iter().take(5) {
            println!("  diagram {} amp momentum |sum|={:.3e}", d, off);
        }

        // find max-magnitude diagram for this helicity
        let mut worst = (0usize, 0.0f64);
        for (d, ast) in asts.iter().enumerate() {
            let amp = eval_single_diagram(ast, &p, &hel, &evaluated, n_in, &[]);
            if amp.norm() > worst.1 {
                worst = (d, amp.norm());
            }
        }
        println!("worst diagram {} |amp|={:.3e}", worst.0, worst.1);

        // trace the worst diagram step by step
        let ast = &asts[worst.0];
        let mut slots = vec![WaveformSlot::Empty; ast.n_slots];
        for step in &ast.steps {
            match step {
                EvalStep::ExternalWf { info, output_slot } => {
                    slots[*output_slot] = build_external_slot(
                        p[info.leg_idx],
                        hel[info.leg_idx],
                        info,
                        n_in,
                        &evaluated,
                    );
                    println!("Ext leg {} (spin {} charge {:?} incoming {}) raw_E {} -> slot {} stored_E {:?}",
                        info.leg_idx, info.spin, info.charge, info.leg_idx < n_in,
                        p[info.leg_idx].e(), output_slot,
                        slots[*output_slot].momentum().map(|m| m.e()));
                }
                EvalStep::OffShellCurrent {
                    info,
                    input_slots,
                    output_slot,
                } => {
                    slots[*output_slot] =
                        evaluate_off_shell_current(info, input_slots, &slots, &evaluated);
                    println!(
                        "OffShell inputs {:?} -> slot {} mom {:?}",
                        input_slots,
                        output_slot,
                        slots[*output_slot].momentum().map(|m| (m.e(), m.m2()))
                    );
                }
                EvalStep::Propagate {
                    info,
                    input_slot,
                    output_slot,
                } => {
                    let m = evaluated.mass(info.id);
                    slots[*output_slot] =
                        evaluate_propagation(info, &slots[*input_slot], &evaluated);
                    let mom = slots[*output_slot].momentum().unwrap();
                    println!(
                        "Propagate slot {}->{} mass {} q2 {:.4e} (q2-m2 {:.4e})",
                        input_slot,
                        output_slot,
                        m,
                        mom.m2(),
                        mom.m2() - m * m
                    );
                }
                EvalStep::ContractAmplitude {
                    info,
                    input_slots,
                    output_slot,
                } => {
                    slots[*output_slot] =
                        evaluate_contract_amplitude(info, input_slots, &slots, &evaluated);
                    println!(
                        "Contract inputs {:?} -> slot {} mom {:?}",
                        input_slots,
                        output_slot,
                        slots[*output_slot].momentum().map(|m| (m.e(), m.m2()))
                    );
                }
            }
        }
    }

    /// PROBE: per-diagram amplitude breakdown for the uux 2->6 process.
    ///
    /// Identifies the diagram classes (by propagator content), measures the
    /// gauge cancellation (|Σa|² vs Σ|a|²), and compares the total |M|² to the
    /// MadGraph reference. MadGraph (matrix1_orig.f) has NGRAPHS=579 for this
    /// exact process (IDUP 2,-2,4,-4,-11,11,-13,13) and sums AMP() in plain
    /// COMPLEX*16 (no Kahan / quad precision).
    #[test]
    #[ignore]
    fn probe_uux_diagrams() {
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use std::collections::BTreeMap;

        let model = sm_model();
        let evaluated = model.evaluate(&ParamCard::from_str("").unwrap());
        let opts = ParsingOptions::default();
        let card = parse_proc_card("generate u u~ > c c~ e+ e- mu+ mu- QCD=0", &opts).unwrap();
        let sets = generate_from_proc_card(&card, model).unwrap();
        let set = &sets[0];
        let asts = compile_diagram_ast(set, model).unwrap();
        let n_in = 2;
        println!("n_diagrams = {} (MadGraph NGRAPHS = 579)", asts.len());

        let p = [
            LorentzVector::new(250.0, 0.0, 0.0, 250.0),
            LorentzVector::new(250.0, 0.0, 0.0, -250.0),
            LorentzVector::new(
                51.58390415317875,
                33.76278178716875,
                -30.22430158990549,
                24.646811702100443,
            ),
            LorentzVector::new(
                144.43288205367912,
                82.46142822300477,
                -108.5785090953523,
                -47.662119512089546,
            ),
            LorentzVector::new(
                116.59102846088923,
                -91.67803127457901,
                23.194248232604252,
                68.1955522604629,
            ),
            LorentzVector::new(
                52.76154461240437,
                -47.85411853901415,
                17.571868307294295,
                -13.601226890682527,
            ),
            LorentzVector::new(
                20.24063060353008,
                -3.7687275028075944,
                -0.4710997332332683,
                19.881093664069077,
            ),
            LorentzVector::new(
                114.39001011631855,
                27.076667306227247,
                98.5077938785925,
                -51.460111223860345,
            ),
        ];

        // Propagator signature of a diagram = sorted multiset of internal particle names.
        let prop_sig = |ast: &DiagramAst| -> Vec<String> {
            let mut names: Vec<String> = ast
                .steps
                .iter()
                .filter_map(|s| match s {
                    EvalStep::Propagate { info, .. } => Some(model.particle(info.id).name.clone()),
                    _ => None,
                })
                .collect();
            names.sort();
            names
        };

        // 1) How many diagrams contain each internal particle?
        let mut contains: BTreeMap<String, usize> = BTreeMap::new();
        for ast in &asts {
            let uniq: std::collections::BTreeSet<String> = prop_sig(ast).into_iter().collect();
            for name in uniq {
                *contains.entry(name).or_default() += 1;
            }
        }
        println!("\ndiagrams containing each internal particle:");
        for (k, v) in &contains {
            println!("  {k:<6} : {v}");
        }

        // 1b) Which diagrams panic during evaluation? Group by propagator signature.
        let hel0 = [-1i32; 8];
        let mut panic_sigs: BTreeMap<String, usize> = BTreeMap::new();
        let mut ok_count = 0usize;
        std::panic::set_hook(Box::new(|_| {})); // silence per-diagram panic spew
        for ast in &asts {
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                eval_single_diagram(ast, &p, &hel0, &evaluated, n_in, &[])
            }));
            match res {
                Ok(_) => ok_count += 1,
                Err(_) => *panic_sigs.entry(prop_sig(ast).join("+")).or_default() += 1,
            }
        }
        let _ = std::panic::take_hook();
        let n_panic: usize = panic_sigs.values().sum();
        println!("\nevaluation: {ok_count} ok, {n_panic} PANIC");
        println!("panicking diagrams by propagator signature:");
        for (sig, c) in &panic_sigs {
            println!("  [{sig}] x{c}");
        }
        if n_panic > 0 {
            println!("\n(stopping before |M|² — some diagrams don't evaluate yet)");
            return;
        }

        // 2) Per-helicity cancellation + total |M|^2 over all helicities.
        let hel_states = [-1i32, 1];
        let mut total_m2 = 0.0f64;
        let mut worst_cancel = (Vec::new(), 1.0f64, 0.0f64); // (hel, |Σa|²/Σ|a|², Σ|a|²)
                                                             // count helicity combos
        let mut n_hel = 0usize;
        let mut combos: Vec<Vec<i32>> = vec![vec![]];
        for _ in 0..8 {
            let mut next = vec![];
            for c in &combos {
                for &h in &hel_states {
                    let mut cc = c.clone();
                    cc.push(h);
                    next.push(cc);
                }
            }
            combos = next;
        }
        let has_higgs: Vec<bool> = asts
            .iter()
            .map(|a| prop_sig(a).contains(&"H".to_string()))
            .collect();
        let fsign: Vec<f64> = asts.iter().map(|a| a.fermi_sign as f64).collect();
        let mut total_m2_noh = 0.0f64;
        let mut total_m2_noh_nosign = 0.0f64; // continuum coherent sum with fermi_sign stripped
        let mut incoh_noh = 0.0f64; // Σ_hel Σ_d |a_d|² over continuum diagrams
        for hel in &combos {
            let amps: Vec<C<f64>> = asts
                .iter()
                .map(|ast| eval_single_diagram(ast, &p, hel, &evaluated, n_in, &[]))
                .collect();
            let sum: C<f64> = amps.iter().fold(C::new(0.0, 0.0), |a, b| a + *b);
            let sum_noh: C<f64> = amps
                .iter()
                .zip(&has_higgs)
                .filter(|(_, h)| !**h)
                .fold(C::new(0.0, 0.0), |a, (b, _)| a + *b);
            let sum_noh_nosign: C<f64> = amps
                .iter()
                .zip(&has_higgs)
                .zip(&fsign)
                .filter(|((_, h), _)| !**h)
                .fold(C::new(0.0, 0.0), |a, ((b, _), fs)| a + *b / *fs);
            total_m2_noh_nosign += sum_noh_nosign.norm_sqr();
            let sum_abs2: f64 = amps.iter().map(|a| a.norm_sqr()).sum();
            incoh_noh += amps
                .iter()
                .zip(&has_higgs)
                .filter(|(_, h)| !**h)
                .map(|(a, _)| a.norm_sqr())
                .sum::<f64>();
            total_m2 += sum.norm_sqr();
            total_m2_noh += sum_noh.norm_sqr();
            n_hel += 1;
            if sum_abs2 > 0.0 {
                let ratio = sum.norm_sqr() / sum_abs2;
                if sum_abs2 > worst_cancel.2 {
                    worst_cancel = (hel.clone(), ratio, sum_abs2);
                }
            }
        }
        // color factor for uux (two quark lines, Nc^2 = 9)
        let total_m2 = total_m2 * 9.0;
        let total_m2_noh = total_m2_noh * 9.0;
        let mg_ref = 2.9422266141524934e-18; // CSV point-0 reference
        println!("\n{n_hel} helicity combos");
        println!("MG reference (pt0)        = {mg_ref:.6e}");
        println!(
            "Σ_hel |Σ_d a_d|² × cf(9)  = {total_m2:.6e}  (ratio {:.3e})",
            total_m2 / mg_ref
        );
        println!(
            "  excluding 3 Higgs diags = {total_m2_noh:.6e}  (ratio {:.3e})",
            total_m2_noh / mg_ref
        );
        let incoh_noh = incoh_noh * 9.0;
        let total_m2_noh_nosign = total_m2_noh_nosign * 9.0;
        println!(
            "  continuum cancellation: coherent Σ|Σa|²={total_m2_noh:.4e}  incoherent ΣΣ|a|²={incoh_noh:.4e}  (coh/incoh {:.3e})",
            total_m2_noh / incoh_noh
        );
        let fdist = fsign.iter().filter(|&&s| s < 0.0).count();
        println!(
            "  fermi_sign stripped: continuum |M|²={total_m2_noh_nosign:.4e} (ratio to MG {:.3e}); {fdist}/{} diagrams have fermi_sign=-1",
            total_m2_noh_nosign / mg_ref,
            asts.len()
        );
        println!(
            "loudest-helicity cancellation: Σ|a_d|²={:.4e}  |Σa_d|²/Σ|a_d|²={:.4e}  hel={:?}",
            worst_cancel.2, worst_cancel.1, worst_cancel.0
        );

        // 3) For that loudest helicity, top contributors grouped by signature.
        let hel = &worst_cancel.0;
        let mut by_sig: BTreeMap<String, (usize, f64, C<f64>)> = BTreeMap::new();
        for ast in &asts {
            let a = eval_single_diagram(ast, &p, hel, &evaluated, n_in, &[]);
            let sig = prop_sig(ast).join("+");
            let e = by_sig.entry(sig).or_insert((0, 0.0, C::new(0.0, 0.0)));
            e.0 += 1;
            e.1 += a.norm_sqr();
            e.2 = e.2 + a;
        }
        println!("\nby propagator signature @ loudest helicity (count, Σ|a|², |Σa|):");
        let mut rows: Vec<_> = by_sig.into_iter().collect();
        rows.sort_by(|a, b| b.1 .1.partial_cmp(&a.1 .1).unwrap());
        for (sig, (cnt, s2, s)) in rows.iter().take(20) {
            println!(
                "  [{sig:<28}] x{cnt:<3} Σ|a|²={:.4e} |Σa|={:.4e}",
                s2,
                s.norm()
            );
        }
    }

    /// Per-diagram amplitude dump for e+ e- > mu+ mu- ta+ ta- (QCD=0), the
    /// minimal chained-off-shell-fermion-current reproducer of the uux continuum
    /// relative-phase bug (2→4, 25 diagrams, colorless).  Computes the two
    /// basis-independent helicity-summed quantities (invariant under the massive-τ
    /// spin-basis ambiguity) at CSV point 0 to cross-check against MadGraph's
    /// per-diagram AMP() (validation/madgraph/probe_amp.py):
    ///   diag[i] = Σ_hel |a_i|²            (per-diagram magnitude → match diagrams)
    ///   Rrow[i] = Σ_hel conj(a_i)·a_total (contribution to |M|²; Σ Re = |M|²)
    ///
    /// Run: cargo test -p vibegraph-lib --features extended-validation \
    ///        --lib helas::eval::run::tests::probe_eemumutata_diagrams -- --ignored --nocapture
    #[test]
    #[ignore]
    fn probe_eemumutata_diagrams() {
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};

        let model = sm_model();
        // Use MadGraph's exact param card for this process (massive τ, massless e/μ).
        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let card_path = std::path::Path::new(&manifest).join(
            "../validation/madgraph/output/ee_to_mumu_tata_qcd0/Cards/param_card_masslesstau.dat",
        );
        let card = std::fs::read_to_string(&card_path)
            .ok()
            .and_then(|s| ParamCard::from_str(&s).ok())
            .expect("ee_to_mumu_tata_qcd0 param_card_masslesstau.dat");
        let evaluated = model.evaluate(&card);

        let opts = ParsingOptions::default();
        let pc = parse_proc_card("generate e+ e- > mu+ mu- ta+ ta- QCD=0", &opts).unwrap();
        let sets = generate_from_proc_card(&pc, model).unwrap();
        let set = &sets[0];
        let asts = compile_diagram_ast(set, model).unwrap();
        let n_in = 2;
        println!("n_diagrams = {} (MadGraph NGRAPHS = 25)", asts.len());

        // CSV point-0 momenta, order [e+,e-,mu+,mu-,ta+,ta-].
        let p = [
            LorentzVector::new(250.0, 0.0, 0.0, 250.0),
            LorentzVector::new(250.0, 0.0, 0.0, -250.0),
            LorentzVector::new(
                130.98844490914234,
                -106.66561232781022,
                -0.9379201403415187,
                -76.02328690775641,
            ),
            LorentzVector::new(
                167.2530959714149,
                134.2336665209957,
                -62.607066356179416,
                -77.68703963098595,
            ),
            LorentzVector::new(
                94.5533499515044,
                -18.39281604525598,
                -22.219961151047247,
                90.04617499607066,
            ),
            LorentzVector::new(
                107.2051091679384,
                -9.175238147929512,
                85.76494764756818,
                63.66415154267164,
            ),
        ];

        let prop_sig = |ast: &DiagramAst| -> String {
            let mut names: Vec<String> = ast
                .steps
                .iter()
                .filter_map(|s| match s {
                    EvalStep::Propagate { info, .. } => Some(model.particle(info.id).name.clone()),
                    _ => None,
                })
                .collect();
            names.sort();
            names.join("+")
        };

        // All 64 helicity combos for 6 external legs.
        let mut combos: Vec<Vec<i32>> = vec![vec![]];
        for _ in 0..6 {
            let mut next = vec![];
            for c in &combos {
                for &h in &[-1i32, 1] {
                    let mut cc = c.clone();
                    cc.push(h);
                    next.push(cc);
                }
            }
            combos = next;
        }

        // print all asts
        for (i, ast) in asts.iter().enumerate() {
            println!("AST {}: {}", i, ast);
        }
        let n = asts.len();
        // The e-spine relative −1 vs MadGraph is now carried by `fermi_sign`
        // (topo_sort::initial_state_spine_sign), applied inside
        // eval_single_diagram — no manual flip here.
        let mut diag = vec![0.0f64; n];
        let mut rrow = vec![C::new(0.0, 0.0); n];
        let mut m2_total = 0.0f64;
        let mut amp_hel0 = None;
        let mut full: Vec<Vec<C<f64>>> = vec![Vec::with_capacity(combos.len()); n]; // [diagram][hel]
        for hel in &combos {
            let amps: Vec<C<f64>> = asts
                .iter()
                .enumerate()
                .map(|(_, ast)| eval_single_diagram(ast, &p, hel, &evaluated, n_in, &[]))
                .collect();
            let a_tot: C<f64> = amps.iter().fold(C::new(0.0, 0.0), |a, b| a + *b);
            m2_total += a_tot.norm_sqr();
            for (i, a) in amps.iter().enumerate() {
                diag[i] += a.norm_sqr();
                rrow[i] = rrow[i] + a.conj() * a_tot;
                full[i].push(*a);
            }
            if hel == &[-1, 1, -1, 1, -1, 1] {
                amp_hel0 = Some(amps);
            }
        }
        // Write the full [diagram][helicity] complex array + per-diagram sig for the
        // Python matcher (validation/madgraph/match_amps.py). Same helicity order as
        // itertools.product((-1,1), repeat=6) (leg0 slowest).
        {
            use std::fmt::Write as _;
            let mut s = String::new();
            for i in 0..n {
                let _ = write!(s, "{}\t{}", i, prop_sig(&asts[i]));
                for a in &full[i] {
                    let _ = write!(s, "\t{}\t{}", a.re, a.im);
                }
                s.push('\n');
            }
            let out = std::path::Path::new(&manifest)
                .join("../validation/madgraph/output/vibegraph_amps_full.txt");
            std::fs::write(&out, s).unwrap();
            println!("wrote {}", out.display());
        }
        let rsum: f64 = rrow.iter().map(|r| r.re).sum();
        println!("vibegraph total |M|² = {m2_total:.10e}");
        println!("(MG CSV point-0 ref  = 1.1519918572120465e-10)");
        println!("check Σ Re(Rrow)     = {rsum:.10e}");

        for (i, amp) in amp_hel0.unwrap().iter().enumerate() {
            // Diagram {i:02}  {amp.real:+.8e} {amp.imag:+.8e} (diag[{i}] = {diag[i]:.8e})
            println!(
                "Diagram {i:02}  {are:+.8e} {aim:+.8e} (diag[{i}] = {diag_i:.8e})  [{sig}]",
                are = amp.re,
                aim = amp.im,
                diag_i = diag[i],
                sig = prop_sig(&asts[i])
            );
        }

        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&a, &b| diag[b].partial_cmp(&diag[a]).unwrap());
        println!("\n  vibegraph diagrams sorted by magnitude:");
        println!("  rank  diag_mag        Re(Rrow)         sig");
        for (rank, &i) in order.iter().enumerate() {
            println!(
                "  {rank:3}   {:.6e}  {:+.6e}  [{}]",
                diag[i],
                rrow[i].re,
                prop_sig(&asts[i])
            );
        }
    }

    /// Cross-check `ProjMAmp` / `ProjPAmp` nodes against the `iosxxx` reference routine.
    ///
    /// FFS1: ProjM(2,1) rooted at amplitude → ScalarProduct[ProjMAmp, Leg(3)]
    /// FFS3: ProjP(2,1) rooted at amplitude → ScalarProduct[ProjPAmp, Leg(3)]
    ///
    /// Slot ordering: input_slots=[leg1_slot, leg2_slot, leg3_slot].
    /// leg1 = fi (column/incoming), leg2 = fo (row/outgoing), leg3 = scalar.
    /// iosxxx uses gc=[g,0] (left-only) for FFS1 and gc=[0,g] (right-only) for FFS3.
    #[test]
    fn test_eval_proj_amp_vs_iosxxx() {
        use crate::helas::vertex::iosxxx;
        use crate::ufo::lorentz::{LorentzOp, LorentzTerm};
        use num_complex::Complex64;

        let mass = 0.511e-3_f64;
        let p_fi = LorentzVector::from_pxpypzmass(30.0, 0.0, 40.0, mass);
        let p_fo = LorentzVector::from_pxpypzmass(-20.0, 10.0, -30.0, mass);
        let p_s = -(p_fi + p_fo);
        let s_wf = ScalarWf {
            value: Complex64::new(0.7, -0.3),
            momentum: p_s,
        };
        let g = Complex64::new(1.0, 0.0);

        let ffs1 = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::ProjM { i: 1, j: 0 }],
        };
        let ffs3 = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::ProjP { i: 1, j: 0 }],
        };
        let spins = [2, 2, 1];

        let hels = [SpinorHelicity::Down, SpinorHelicity::Up];
        for (hel1, hel2) in iproduct!(hels, hels) {
            for charge in [Charge::Particle, Charge::Antiparticle] {
                let fo = OutDiracWf::from_momentum(p_fo, mass, hel1, charge);
                let fi = InDiracWf::from_momentum(p_fi, mass, hel2, charge);

                let left_ref = iosxxx(&fo, &fi, &s_wf, [g, Complex64::new(0.0, 0.0)]);
                let right_ref = iosxxx(&fo, &fi, &s_wf, [Complex64::new(0.0, 0.0), g]);

                // Build slots: leg1=fi (col / flow-in), leg2=fo (row / flow-out), leg3=scalar
                let mut slots: Vec<WaveformSlot<f64>> = vec![WaveformSlot::Empty; 3];
                slots[0] = WaveformSlot::FermionIn(fi);
                slots[1] = WaveformSlot::FermionOut(fo);
                slots[2] = WaveformSlot::Scalar(s_wf);
                let input_slots = vec![0usize, 1, 2];

                // FFS1: ProjM(2,1) → left bilinear × s
                let tree1 = LorentzEvalTree::build_at_leg(&ffs1, &spins, None).unwrap();
                let WaveformSlot::Scalar(got1) =
                    evaluate_lorentz_node(&tree1, tree1.root_value(), &input_slots, &slots)
                else {
                    panic!("FFS1 did not produce a scalar");
                };
                let diff1 = (got1.value - left_ref).norm();
                assert!(
                    diff1 < 1e-10,
                    "ProjMAmp vs iosxxx left diff={diff1} (hel {hel1},{hel2}, {charge:?})"
                );

                // FFS3: ProjP(2,1) → right bilinear × s
                let tree3 = LorentzEvalTree::build_at_leg(&ffs3, &spins, None).unwrap();
                let WaveformSlot::Scalar(got3) =
                    evaluate_lorentz_node(&tree3, tree3.root_value(), &input_slots, &slots)
                else {
                    panic!("FFS3 did not produce a scalar");
                };
                let diff3 = (got3.value - right_ref).norm();
                assert!(
                    diff3 < 1e-10,
                    "ProjPAmp vs iosxxx right diff={diff3} (hel {hel1},{hel2}, {charge:?})"
                );
            }
        }
    }

    /// Full-amplitude Ward identity for a 2→3 process with a final-state photon:
    /// `e+ e- > mu+ mu- a`. Replacing the external photon's polarisation ε^μ with
    /// its 4-momentum k^μ must make the *coherent sum over all diagrams* vanish
    /// (U(1) gauge invariance / current conservation). Unlike the single-current
    /// unit Ward tests, this exercises the multi-vertex paths the uux continuum
    /// depends on but 2→2 ee→μμ never hits:
    ///   - a fermion propagator chaining two vertices on one line (FSR: the muon
    ///     line absorbs the s-channel boson, propagates, then radiates the photon),
    ///   - an off-shell γ/Z (internal `VectorWf`, −i/q²) absorbed by a fermion line
    ///     via `GammaIout`/`GammaJout`.
    /// If the relative phases/signs between continuum diagrams are wrong (the
    /// diagnosed bug), this sum will NOT cancel.
    /// Largest U(1) Ward residual `|Σ_diagrams M(ε_γ→k_γ)| / max|M|`, maximised
    /// over all helicity configurations, for `proc` at momenta `p` with the photon
    /// on `ward_leg` replaced by its 4-momentum. Lepton masses are zeroed so the
    /// hand-built massless momenta are exactly on-shell (else the spinors fail the
    /// Dirac equation and Ward picks up an O(m²/s) artifact). Returns ~0 (machine
    /// precision) iff the coherent sum over diagrams gauge-cancels correctly.
    fn ward_max_ratio(proc: &str, p: &[LorentzVector<f64>], ward_leg: usize) -> f64 {
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};

        let model = sm_model();
        let evaluated = model
            .evaluate(&ParamCard::from_str("Block MASS\n 11 0.0\n 13 0.0\n 15 0.0\n").unwrap());
        let opts = ParsingOptions::default();
        let card = parse_proc_card(proc, &opts).unwrap();
        let sets = generate_from_proc_card(&card, model).unwrap();
        let eval = AmplitudeEvaluator::compile(&sets[0], model).unwrap();

        let k = p[ward_leg];
        let mut overrides: Vec<Option<VectorWf<f64>>> = vec![None; p.len()];
        overrides[ward_leg] = Some(VectorWf {
            eps: ComplexVector::from(k),
            momentum: k,
        });

        let global_scale = eval
            .helicities()
            .iter()
            .map(|hel| eval.eval_amplitude(p, hel, &evaluated).norm())
            .fold(0.0_f64, f64::max)
            .max(1e-30);

        eval.helicities()
            .iter()
            .map(|hel| {
                eval.eval_amplitude_with_overrides(p, hel, &evaluated, &overrides)
                    .norm()
                    / global_scale
            })
            .fold(0.0_f64, f64::max)
    }

    /// Full-amplitude Ward identity for a 2→3 process with a final-state photon:
    /// `e+ e- > mu+ mu- a`. Replacing the external photon's polarisation ε^μ with
    /// its 4-momentum k^μ must make the *coherent sum over all diagrams* vanish
    /// (U(1) gauge invariance). Exercises the multi-vertex paths the uux continuum
    /// depends on but 2→2 ee→μμ never hits: a fermion propagator chaining two
    /// vertices on one line, and an off-shell γ/Z (internal `VectorWf`, −i/q²)
    /// absorbed by a fermion line via the off-shell-current nodes.
    #[test]
    fn test_ward_identity_full_amplitude_eemumua() {
        // Massless on-shell momenta in the e+e- CM frame, √s = 100; photon = leg 4.
        let s40 = 5.0 * 40.0_f64.sqrt();
        let p = [
            LorentzVector::new(50.0, 0.0, 0.0, 50.0),   // e+ (leg 0)
            LorentzVector::new(50.0, 0.0, 0.0, -50.0),  // e- (leg 1)
            LorentzVector::new(30.0, 30.0, 0.0, 0.0),   // mu+ (leg 2)
            LorentzVector::new(35.0, -15.0, s40, 0.0),  // mu- (leg 3)
            LorentzVector::new(35.0, -15.0, -s40, 0.0), // a   (leg 4)
        ];
        let ratio = ward_max_ratio("generate e+ e- > mu+ mu- a", &p, 4);
        assert!(
            ratio < 1e-9,
            "2→3 Ward identity violated: max |k·M|/scale = {ratio:.3e}"
        );
    }

    /// Quark-line counterpart of the 2→3 Ward test: `u u~ > mu+ mu- a`. The photon
    /// radiates off the (massless) initial-state up-quark line or the final muon
    /// line; the up-type quark FFV couplings (and the quark off-shell current) must
    /// gauge-cancel just like the leptonic case. This is the quark-continuum path
    /// the uux 2→6 process depends on.
    #[test]
    fn test_ward_identity_full_amplitude_uumumua() {
        let s40 = 5.0 * 40.0_f64.sqrt();
        let p = [
            LorentzVector::new(50.0, 0.0, 0.0, 50.0),   // u   (leg 0)
            LorentzVector::new(50.0, 0.0, 0.0, -50.0),  // u~  (leg 1)
            LorentzVector::new(30.0, 30.0, 0.0, 0.0),   // mu+ (leg 2)
            LorentzVector::new(35.0, -15.0, s40, 0.0),  // mu- (leg 3)
            LorentzVector::new(35.0, -15.0, -s40, 0.0), // a   (leg 4)
        ];
        let ratio = ward_max_ratio("generate u u~ > mu+ mu- a", &p, 4);
        assert!(
            ratio < 1e-9,
            "u u~ Ward identity violated: max |k·M|/scale = {ratio:.3e}"
        );
    }

    /// 2→5 Ward identity with THREE fermion lines: `e+ e- > mu+ mu- ta+ ta- a`.
    /// With three lepton lines joined by internal bosons, the boson-tree forces at
    /// least one fermion line to absorb TWO *internal* (off-shell, −i/q²) bosons in
    /// series — the exact path the uux 2→6 continuum needs but the 2→3/2→4 photon
    /// tests (one internal boson + external photons) never exercise. The photon
    /// (leg 6) is Ward-substituted.
    ///
    /// Regression guard for the FFS off-shell *scalar* (Higgs) current momentum bug:
    /// it used `fo.p + fi.p`, while the analogous off-shell vector current
    /// `GammaVout` uses `fo.p − fi.p` (the HELAS jioxxx convention). The sum is
    /// harmless at the amplitude sink (momentum unused there) but non-conserving when
    /// the scalar is an off-shell Higgs current feeding a VVS vertex — which only
    /// happens with ≥3 fermion lines. See `probe_2to5_momentum`.
    #[test]
    fn test_ward_identity_full_amplitude_eemumutata_a() {
        let r3 = 3.0_f64.sqrt();
        let p = [
            LorentzVector::new(50.0, 0.0, 0.0, 50.0),  // e+  (leg 0)
            LorentzVector::new(50.0, 0.0, 0.0, -50.0), // e-  (leg 1)
            LorentzVector::new(20.0, 20.0, 0.0, 0.0),  // mu+ (leg 2)
            LorentzVector::new(20.0, -20.0, 0.0, 0.0), // mu- (leg 3)
            LorentzVector::new(20.0, 0.0, 20.0, 0.0),  // ta+ (leg 4)
            LorentzVector::new(20.0, 0.0, -10.0, 10.0 * r3), // ta- (leg 5)
            LorentzVector::new(20.0, 0.0, -10.0, -10.0 * r3), // a  (leg 6)
        ];
        let ratio = ward_max_ratio("generate e+ e- > mu+ mu- ta+ ta- a", &p, 6);
        assert!(
            ratio < 1e-9,
            "2→5 Ward identity violated: max |k·M|/scale = {ratio:.3e}"
        );
    }

    /// Full-amplitude Ward identity for a 2→4 process with TWO final-state photons:
    /// `e+ e- > mu+ mu- a a`. Beyond the 2→3 test this exercises a fermion line
    /// with THREE attachments (s-channel boson + two photons) → a *chained*
    /// off-shell fermion current (two fermion propagators in series), and
    /// `GammaVout` built from off-shell currents — the longer chains the 2→6 uux
    /// continuum needs. Ward-substituting one photon must still cancel the sum.
    #[test]
    fn test_ward_identity_full_amplitude_eemumuaa() {
        // Equal-energy massless tetrahedral final state (Σp⃗=0, ΣE=√s=100), e+e- on z.
        let c = 25.0 / 3.0_f64.sqrt();
        let p = [
            LorentzVector::new(50.0, 0.0, 0.0, 50.0),  // e+  (leg 0)
            LorentzVector::new(50.0, 0.0, 0.0, -50.0), // e-  (leg 1)
            LorentzVector::new(25.0, c, c, c),         // mu+ (leg 2)
            LorentzVector::new(25.0, c, -c, -c),       // mu- (leg 3)
            LorentzVector::new(25.0, -c, c, -c),       // a   (leg 4)
            LorentzVector::new(25.0, -c, -c, c),       // a   (leg 5)
        ];
        let ratio = ward_max_ratio("generate e+ e- > mu+ mu- a a", &p, 4);
        assert!(
            ratio < 1e-9,
            "2→4 Ward identity violated: max |k·M|/scale = {ratio:.3e}"
        );
    }

    /// DIAGNOSTIC: per-diagram Ward breakdown for `e+ e- > mu+ mu- a`.
    /// Prints each diagram's propagator signature and its Ward-substituted
    /// (ε_γ→k_γ) complex amplitude, so we can see which group fails to telescope.
    #[test]
    #[ignore]
    fn probe_ward_eemumua() {
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};

        let model = sm_model();
        let evaluated = model.evaluate(&ParamCard::from_str("").unwrap());
        let opts = ParsingOptions::default();
        let card = parse_proc_card("generate e+ e- > mu+ mu- a", &opts).unwrap();
        let sets = generate_from_proc_card(&card, model).unwrap();
        let set = &sets[0];
        let asts = compile_diagram_ast(set, model).unwrap();
        let n_in = 2;

        let s40 = 5.0 * 40.0_f64.sqrt();
        let p = [
            LorentzVector::new(50.0, 0.0, 0.0, 50.0),
            LorentzVector::new(50.0, 0.0, 0.0, -50.0),
            LorentzVector::new(30.0, 30.0, 0.0, 0.0),
            LorentzVector::new(35.0, -15.0, s40, 0.0),
            LorentzVector::new(35.0, -15.0, -s40, 0.0),
        ];
        let k = p[4];
        let mut overrides: Vec<Option<VectorWf<f64>>> = vec![None; 5];
        overrides[4] = Some(VectorWf {
            eps: ComplexVector::from(k),
            momentum: k,
        });

        let prop_sig = |ast: &DiagramAst| -> Vec<String> {
            let mut names: Vec<String> = ast
                .steps
                .iter()
                .filter_map(|s| match s {
                    EvalStep::Propagate { info, .. } => Some(model.particle(info.id).name.clone()),
                    _ => None,
                })
                .collect();
            names.sort();
            names
        };

        // Use a helicity combo where the violation is large.
        for hel in [[-1, -1, -1, -1, 1], [1, -1, -1, 1, 1], [-1, 1, 1, -1, 1]] {
            println!("\n=== fermion helicities {hel:?} (photon overridden ε→k) ===");
            let mut sum = C::new(0.0, 0.0);
            for (d, ast) in asts.iter().enumerate() {
                let a = eval_single_diagram(ast, &p, &hel, &evaluated, n_in, &overrides);
                sum += a;
                println!(
                    "  diag {d}: props {:?}  k·M = {:+.4e} {:+.4e}i  |a|={:.3e}",
                    prop_sig(ast),
                    a.re,
                    a.im,
                    a.norm()
                );
            }
            println!(
                "  Σ k·M = {:+.4e} {:+.4e}i  |Σ|={:.3e}",
                sum.re,
                sum.im,
                sum.norm()
            );
        }

        // Step-by-step trace of one FSR diagram (0) and one ISR diagram (6).
        let hel = [1, -1, -1, 1, 1];
        for d in [0usize, 2, 4, 6] {
            println!(
                "\n--- TRACE diag {d} (props {:?}) hel {hel:?} ---",
                prop_sig(&asts[d])
            );
            let ast = &asts[d];
            let mut slots = vec![WaveformSlot::Empty; ast.n_slots];
            for step in &ast.steps {
                match step {
                    EvalStep::ExternalWf { info, output_slot } => {
                        slots[*output_slot] = match overrides.get(info.leg_idx) {
                            Some(Some(v)) => WaveformSlot::Vector(*v),
                            _ => build_external_slot(
                                p[info.leg_idx],
                                hel[info.leg_idx],
                                info,
                                n_in,
                                &evaluated,
                            ),
                        };
                        println!(
                            "  Ext leg {} spin {} charge {:?} in {} -> slot {} mom_E {:?}",
                            info.leg_idx,
                            info.spin,
                            info.charge,
                            info.leg_idx < n_in,
                            output_slot,
                            slots[*output_slot].momentum().map(|m| m.e())
                        );
                    }
                    EvalStep::OffShellCurrent {
                        info,
                        input_slots,
                        output_slot,
                    } => {
                        slots[*output_slot] =
                            evaluate_off_shell_current(info, input_slots, &slots, &evaluated);
                        let kind = match &slots[*output_slot] {
                            WaveformSlot::FermionIn(_) => "FermionIn",
                            WaveformSlot::FermionOut(_) => "FermionOut",
                            WaveformSlot::Vector(_) => "Vector",
                            WaveformSlot::Scalar(_) => "Scalar",
                            WaveformSlot::Empty => "Empty",
                        };
                        println!(
                            "  OffShell in {:?} -> slot {} {kind} mom {:?}",
                            input_slots,
                            output_slot,
                            slots[*output_slot].momentum().map(|m| (m.e(), m.m2()))
                        );
                    }
                    EvalStep::Propagate {
                        info,
                        input_slot,
                        output_slot,
                    } => {
                        let m = evaluated.mass(info.id);
                        slots[*output_slot] =
                            evaluate_propagation(info, &slots[*input_slot], &evaluated);
                        let mom = slots[*output_slot].momentum().unwrap();
                        println!(
                            "  Propagate {} ({}) slot {}->{} q2 {:.4e} (q2-m2 {:.4e})",
                            model.particle(info.id).name,
                            m,
                            input_slot,
                            output_slot,
                            mom.m2(),
                            mom.m2() - m * m
                        );
                    }
                    EvalStep::ContractAmplitude {
                        info,
                        input_slots,
                        output_slot,
                    } => {
                        slots[*output_slot] =
                            evaluate_contract_amplitude(info, input_slots, &slots, &evaluated);
                        if let WaveformSlot::Scalar(s) = &slots[*output_slot] {
                            println!(
                                "  Contract in {:?} -> amp {:+.4e}{:+.4e}i",
                                input_slots, s.value.re, s.value.im
                            );
                        }
                    }
                }
            }
        }
    }

    /// DIAGNOSTIC: momentum-routing consistency for `e+ e- > mu+ mu- ta+ ta- a`.
    /// Re-runs each diagram and reports the momentum each Propagate/Contract step
    /// produces; the amplitude-sink momentum must be IDENTICAL across all diagrams
    /// (a single convention). Any outlier is a mis-routed diagram. Also flags any
    /// internal propagator whose q² is wildly off (a sign-flipped boson momentum
    /// corrupts the downstream fermion propagator's q²).
    #[test]
    #[ignore]
    fn probe_2to5_momentum() {
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use std::collections::BTreeMap;

        let model = sm_model();
        let evaluated = model
            .evaluate(&ParamCard::from_str("Block MASS\n 11 0.0\n 13 0.0\n 15 0.0\n").unwrap());
        let opts = ParsingOptions::default();
        let card = parse_proc_card("generate e+ e- > mu+ mu- ta+ ta- a", &opts).unwrap();
        let sets = generate_from_proc_card(&card, model).unwrap();
        let asts = compile_diagram_ast(&sets[0], model).unwrap();
        let n_in = 2;
        println!("n_diagrams = {}", asts.len());

        let r3 = 3.0_f64.sqrt();
        let p = [
            LorentzVector::new(50.0, 0.0, 0.0, 50.0),
            LorentzVector::new(50.0, 0.0, 0.0, -50.0),
            LorentzVector::new(20.0, 20.0, 0.0, 0.0),
            LorentzVector::new(20.0, -20.0, 0.0, 0.0),
            LorentzVector::new(20.0, 0.0, 20.0, 0.0),
            LorentzVector::new(20.0, 0.0, -10.0, 10.0 * r3),
            LorentzVector::new(20.0, 0.0, -10.0, -10.0 * r3),
        ];
        let hel = [1, -1, -1, 1, -1, 1, 1];

        // Per diagram: amplitude-sink total momentum (rounded) → count.
        let mut sink_totals: BTreeMap<(i64, i64, i64, i64), usize> = BTreeMap::new();
        let mut traced = 0;
        for (d, ast) in asts.iter().enumerate() {
            let mut slots = vec![WaveformSlot::Empty; ast.n_slots];
            let mut log: Vec<String> = Vec::new();
            for step in &ast.steps {
                match step {
                    EvalStep::ExternalWf { info, output_slot } => {
                        slots[*output_slot] = build_external_slot(
                            p[info.leg_idx],
                            hel[info.leg_idx],
                            info,
                            n_in,
                            &evaluated,
                        );
                        let m = slots[*output_slot].momentum().unwrap();
                        log.push(format!(
                            "  Ext leg {} ({:?}) -> slot {} mom ({:.1},{:.1},{:.1},{:.1})",
                            info.leg_idx,
                            info.charge,
                            output_slot,
                            m.e(),
                            m.px(),
                            m.py(),
                            m.pz()
                        ));
                    }
                    EvalStep::OffShellCurrent {
                        info,
                        input_slots,
                        output_slot,
                    } => {
                        slots[*output_slot] =
                            evaluate_off_shell_current(info, input_slots, &slots, &evaluated);
                        let m = slots[*output_slot].momentum().unwrap();
                        let kind = match &slots[*output_slot] {
                            WaveformSlot::FermionIn(_) => "Fin",
                            WaveformSlot::FermionOut(_) => "Fout",
                            WaveformSlot::Vector(_) => "Vec",
                            _ => "?",
                        };
                        log.push(format!(
                            "  OffShell in {:?} -> slot {} {kind} mom ({:.1},{:.1},{:.1},{:.1})",
                            input_slots,
                            output_slot,
                            m.e(),
                            m.px(),
                            m.py(),
                            m.pz()
                        ));
                    }
                    EvalStep::Propagate {
                        info,
                        input_slot,
                        output_slot,
                    } => {
                        slots[*output_slot] =
                            evaluate_propagation(info, &slots[*input_slot], &evaluated);
                        let m = slots[*output_slot].momentum().unwrap();
                        log.push(format!(
                            "  Propagate {} slot {}->{} mom ({:.1},{:.1},{:.1},{:.1}) q2={:.3e}",
                            model.particle(info.id).name,
                            input_slot,
                            output_slot,
                            m.e(),
                            m.px(),
                            m.py(),
                            m.pz(),
                            m.m2()
                        ));
                    }
                    EvalStep::ContractAmplitude {
                        info,
                        input_slots,
                        output_slot,
                    } => {
                        slots[*output_slot] =
                            evaluate_contract_amplitude(info, input_slots, &slots, &evaluated);
                        log.push(format!("  Contract in {input_slots:?}"));
                    }
                }
            }
            let m = slots[ast.amplitude_slot]
                .momentum()
                .unwrap_or(LorentzVector::new(0.0, 0.0, 0.0, 0.0));
            let key = (
                (m.e() * 1e3).round() as i64,
                (m.px() * 1e3).round() as i64,
                (m.py() * 1e3).round() as i64,
                (m.pz() * 1e3).round() as i64,
            );
            *sink_totals.entry(key).or_default() += 1;
            // Trace the first 2 mis-routed diagrams (sink total != 0).
            if key != (0, 0, 0, 0) && traced < 2 {
                traced += 1;
                println!("\n--- MIS-ROUTED diag {d}: sink total {key:?} ---");
                for l in &log {
                    println!("{l}");
                }
            }
        }
        println!("\ndistinct amplitude-sink momentum totals (E,px,py,pz)*1e3 → count:");
        for (k, v) in &sink_totals {
            println!("  {k:?} : {v}");
        }
    }

    /// DIAGNOSTIC: momentum-routing consistency for the uux 2→6 continuum. Same
    /// idea as `probe_2to5_momentum`: the amplitude-sink momentum must be identical
    /// across all 579 diagrams. Groups outliers by propagator signature so we can
    /// see which diagram class (if any) mis-routes.
    #[test]
    #[ignore]
    fn probe_uux_momentum() {
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use std::collections::BTreeMap;

        let model = sm_model();
        let evaluated = model.evaluate(&ParamCard::from_str("").unwrap());
        let opts = ParsingOptions::default();
        let card = parse_proc_card("generate u u~ > c c~ e+ e- mu+ mu- QCD=0", &opts).unwrap();
        let sets = generate_from_proc_card(&card, model).unwrap();
        let asts = compile_diagram_ast(&sets[0], model).unwrap();
        let n_in = 2;
        println!("n_diagrams = {}", asts.len());

        let p = [
            LorentzVector::new(250.0, 0.0, 0.0, 250.0),
            LorentzVector::new(250.0, 0.0, 0.0, -250.0),
            LorentzVector::new(
                51.58390415317875,
                33.76278178716875,
                -30.22430158990549,
                24.646811702100443,
            ),
            LorentzVector::new(
                144.43288205367912,
                82.46142822300477,
                -108.5785090953523,
                -47.662119512089546,
            ),
            LorentzVector::new(
                116.59102846088923,
                -91.67803127457901,
                23.194248232604252,
                68.1955522604629,
            ),
            LorentzVector::new(
                52.76154461240437,
                -47.85411853901415,
                17.571868307294295,
                -13.601226890682527,
            ),
            LorentzVector::new(
                20.24063060353008,
                -3.7687275028075944,
                -0.4710997332332683,
                19.881093664069077,
            ),
            LorentzVector::new(
                114.39001011631855,
                27.076667306227247,
                98.5077938785925,
                -51.460111223860345,
            ),
        ];
        let hel = [1, -1, 1, -1, 1, -1, 1, -1];

        let prop_sig = |ast: &DiagramAst| -> Vec<String> {
            let mut names: Vec<String> = ast
                .steps
                .iter()
                .filter_map(|s| match s {
                    EvalStep::Propagate { info, .. } => Some(model.particle(info.id).name.clone()),
                    _ => None,
                })
                .collect();
            names.sort();
            names
        };

        let mut sink_totals: BTreeMap<(i64, i64, i64, i64), usize> = BTreeMap::new();
        let mut bad_sigs: BTreeMap<Vec<String>, usize> = BTreeMap::new();
        for ast in &asts {
            let mut slots = vec![WaveformSlot::Empty; ast.n_slots];
            for step in &ast.steps {
                match step {
                    EvalStep::ExternalWf { info, output_slot } => {
                        slots[*output_slot] = build_external_slot(
                            p[info.leg_idx],
                            hel[info.leg_idx],
                            info,
                            n_in,
                            &evaluated,
                        );
                    }
                    EvalStep::OffShellCurrent {
                        info,
                        input_slots,
                        output_slot,
                    } => {
                        slots[*output_slot] =
                            evaluate_off_shell_current(info, input_slots, &slots, &evaluated);
                    }
                    EvalStep::Propagate {
                        info,
                        input_slot,
                        output_slot,
                    } => {
                        slots[*output_slot] =
                            evaluate_propagation(info, &slots[*input_slot], &evaluated);
                    }
                    EvalStep::ContractAmplitude {
                        info,
                        input_slots,
                        output_slot,
                    } => {
                        slots[*output_slot] =
                            evaluate_contract_amplitude(info, input_slots, &slots, &evaluated);
                    }
                }
            }
            let m: LorentzVector<f64> = slots[ast.amplitude_slot]
                .momentum()
                .unwrap_or(LorentzVector::new(0.0, 0.0, 0.0, 0.0));
            let key = (
                (m.e() * 1e2).round() as i64,
                (m.px() * 1e2).round() as i64,
                (m.py() * 1e2).round() as i64,
                (m.pz() * 1e2).round() as i64,
            );
            *sink_totals.entry(key).or_default() += 1;
            if key != (0, 0, 0, 0) {
                *bad_sigs.entry(prop_sig(ast)).or_default() += 1;
            }
        }
        println!(
            "distinct amplitude-sink totals → count: {}",
            sink_totals.len()
        );
        for (k, v) in &sink_totals {
            println!("  {k:?} : {v}");
        }
        println!("mis-routed diagram propagator signatures → count:");
        for (k, v) in &bad_sigs {
            println!("  {k:?} : {v}");
        }
    }
}
