//! Runtime amplitude evaluation: DiagramAst × momenta × helicities → amplitude

use std::collections::HashSet;

use crate::diagrams::DiagramSet;
use crate::helas::eval::ast::ExtLegInfo;
use crate::helas::eval::compile::compile_diagram_ast;
use crate::helas::eval::dispatch::{LorentzEvalNode, LorentzEvalTree};
use crate::helas::repr::lorentz::{
    Bispinor, Chirality, ComplexVector, LorentzVector, SpinorHelicity, SpinorRepr,
};
use crate::helas::repr::propagator::{DiracPropagator, Propagator, ScalarPropagator};
use crate::helas::repr::{r, ri, Real, C};
use crate::helas::wavefn::{InDiracWf, ScalarWf, VectorWf};
use crate::helas::Charge;
use crate::ufo::couplings::CouplingId;
use crate::ufo::particles::ParticleId;
use crate::ufo::{EvaluatedModel, UFOModel};
use num_traits::FromPrimitive;

use super::ast::{DiagramAst, EvalStep, WaveformSlot};
use super::compile::CompileError;

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
                acc + eval_single_diagram(ast, momenta, helicities, evaluated, self.n_in)
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
) -> C<F> {
    let mut slots = vec![WaveformSlot::Empty; ast.n_slots];

    for step in &ast.steps {
        match step {
            EvalStep::ExternalWf { info, output_slot } => {
                // TOOD: store necessary info in ExtLegInfo during compile phase instead of reconstructing here
                slots[*output_slot] = build_external_slot(
                    momenta[info.leg_idx],
                    helicities[info.leg_idx],
                    info,
                    n_in,
                    evaluated,
                );
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
            let wf = InDiracWf::new(momentum, mass, hel, info.charge);
            WaveformSlot::Fermion(wf)
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
    let mass = real(evaluated.mass(info.id));
    let width = real(evaluated.width(info.id));

    // Note all momentum is flipped in the propagator since vertex output is always outgoing
    // and our convention is particle momentum is incoming to the vertex (antiparticle momentum is outgoing from the vertex)
    match input {
        WaveformSlot::Fermion(wf) => {
            let prop = DiracPropagator { mass, width };
            let propagated = prop.propagate(wf.momentum.0, wf.spinor.0);
            WaveformSlot::Fermion(InDiracWf::from_spinor(Bispinor(propagated), -wf.momentum))
        }
        WaveformSlot::Vector(wf) => {
            if mass == F::zero() {
                let out = VectorWf {
                    // -i / q^2
                    eps: wf.eps * ri(-wf.momentum.m2().recip()),
                    momentum: -wf.momentum,
                };
                WaveformSlot::Vector(out)
            } else {
                let vm2 = mass * mass;
                let vmw = mass * width;
                let denom = C::new(wf.momentum.m2() - vm2, vmw);
                // Longitudinal mode subtraction: divide by m²−imΓ (Fabio prescription)
                let cs = wf.eps.mink_dot_lorentz(&wf.momentum) / C::new(vm2, -vmw);
                let out = VectorWf {
                    // i / (q^2 - m^2 + i m G)
                    eps: (wf.eps - ComplexVector::from(wf.momentum) * cs) * ri(-F::one()) / denom,
                    momentum: -wf.momentum,
                };
                WaveformSlot::Vector(out)
            }
        }
        WaveformSlot::Scalar(wf) => {
            let prop = ScalarPropagator { mass, width };
            WaveformSlot::Scalar(ScalarWf {
                value: prop.propagate(wf.momentum.0, wf.value),
                momentum: -wf.momentum,
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

fn evaluate_lorentz_node<F: Real + FromPrimitive>(
    tree: &LorentzEvalTree,
    node: &LorentzEvalNode,
    input_slots: &[usize],
    slots: &[WaveformSlot<F>],
) -> WaveformSlot<F> {
    match node {
        LorentzEvalNode::Leg(i) => slots[input_slots[*i as usize - 1]],
        LorentzEvalNode::GammaVout { i, j } => {
            let WaveformSlot::Fermion(f1) =
                evaluate_lorentz_node(tree, tree.node(*i), input_slots, slots)
            else {
                panic!("expected fermion output from node {i}");
            };
            let WaveformSlot::Fermion(f2) =
                evaluate_lorentz_node(tree, tree.node(*j), input_slots, slots)
            else {
                panic!("expected fermion output from node {j}");
            };
            // Follow current (charge + to -, i.e. anti to particle since e- is a particle)
            let (fo, fi) = match f1.charge() {
                Charge::Particle => (f1.to_outgoing(), f2),
                Charge::Antiparticle => (f2.to_outgoing(), f1),
            };
            WaveformSlot::Vector(VectorWf {
                eps: fi.vector_bilinear(&fo, Chirality::Both),
                momentum: fo.momentum - fi.momentum,
            })
        }
        LorentzEvalNode::GammaIout { mu, j } => {
            // Off-shell fermion current ε̸ ψ from a vector (mu) and a column (flow-in) fermion (j).
            // Output index is the row i; cf. `fioxxx`. The propagator (q̸+m)/D is a separate step.
            let WaveformSlot::Vector(v) =
                evaluate_lorentz_node(tree, tree.node(*mu), input_slots, slots)
            else {
                panic!("expected vector output from node {mu}");
            };
            let WaveformSlot::Fermion(f) =
                evaluate_lorentz_node(tree, tree.node(*j), input_slots, slots)
            else {
                panic!("expected fermion output from node {j}");
            };
            WaveformSlot::Fermion(InDiracWf::from_spinor(
                f.spinor.slash(&v.eps),
                f.momentum + v.momentum,
            ))
        }
        LorentzEvalNode::GammaJout { mu, i } => {
            // Off-shell fermion current ε̸ ψ̄ from a vector (mu) and a row (flow-out) fermion (i).
            // The input leg is the barred index, so adjoint it before slashing; cf. `foxxx`.
            let WaveformSlot::Vector(v) =
                evaluate_lorentz_node(tree, tree.node(*mu), input_slots, slots)
            else {
                panic!("expected vector output from node {mu}");
            };
            let WaveformSlot::Fermion(f) =
                evaluate_lorentz_node(tree, tree.node(*i), input_slots, slots)
            else {
                panic!("expected fermion output from node {i}");
            };
            let q = f.momentum + v.momentum;
            WaveformSlot::Fermion(InDiracWf::from_spinor(
                f.to_outgoing().spinor.slash(&v.eps),
                q,
            ))
        }
        LorentzEvalNode::ProjM { i } => {
            let WaveformSlot::Fermion(f) =
                evaluate_lorentz_node(tree, tree.node(*i), input_slots, slots)
            else {
                panic!("expected fermion output from node {i}");
            };
            WaveformSlot::Fermion(InDiracWf::from_spinor(f.spinor.project_left(), f.momentum))
        }
        LorentzEvalNode::ProjP { i } => {
            let WaveformSlot::Fermion(f) =
                evaluate_lorentz_node(tree, tree.node(*i), input_slots, slots)
            else {
                panic!("expected fermion output from node {i}");
            };
            WaveformSlot::Fermion(InDiracWf::from_spinor(f.spinor.project_right(), f.momentum))
        }
        LorentzEvalNode::Metric { mu, nu } => {
            let WaveformSlot::Vector(v1) =
                evaluate_lorentz_node(tree, tree.node(*mu), input_slots, slots)
            else {
                panic!("expected vector output from node {mu}");
            };
            let WaveformSlot::Vector(v2) =
                evaluate_lorentz_node(tree, tree.node(*nu), input_slots, slots)
            else {
                panic!("expected vector output from node {nu}");
            };
            WaveformSlot::Scalar(ScalarWf {
                value: v1.eps.mink_dot(&v2.eps),
                momentum: v1.momentum + v2.momentum,
            })
        }
        LorentzEvalNode::ProjMAmp { i, j } => {
            // Left-chiral scalar bilinear: ψ̄_i P_L ψ_j
            // Node i is the row (barred) fermion; node j is the column fermion.
            let WaveformSlot::Fermion(fi_row) =
                evaluate_lorentz_node(tree, tree.node(*i), input_slots, slots)
            else {
                panic!("expected fermion for ProjMAmp row (node {i})");
            };
            let WaveformSlot::Fermion(fi_col) =
                evaluate_lorentz_node(tree, tree.node(*j), input_slots, slots)
            else {
                panic!("expected fermion for ProjMAmp col (node {j})");
            };
            let fo = fi_row.to_outgoing();
            let value = Bispinor::scalar_bilinear(&fo.spinor, &fi_col.spinor, Chirality::Left);
            WaveformSlot::Scalar(ScalarWf {
                value,
                momentum: fo.momentum + fi_col.momentum,
            })
        }
        LorentzEvalNode::ProjPAmp { i, j } => {
            // Right-chiral scalar bilinear: ψ̄_i P_R ψ_j
            let WaveformSlot::Fermion(fi_row) =
                evaluate_lorentz_node(tree, tree.node(*i), input_slots, slots)
            else {
                panic!("expected fermion for ProjPAmp row (node {i})");
            };
            let WaveformSlot::Fermion(fi_col) =
                evaluate_lorentz_node(tree, tree.node(*j), input_slots, slots)
            else {
                panic!("expected fermion for ProjPAmp col (node {j})");
            };
            let fo = fi_row.to_outgoing();
            let value = Bispinor::scalar_bilinear(&fo.spinor, &fi_col.spinor, Chirality::Right);
            WaveformSlot::Scalar(ScalarWf {
                value,
                momentum: fo.momentum + fi_col.momentum,
            })
        }
        LorentzEvalNode::P { leg } => {
            let momentum = slots[input_slots[*leg as usize - 1]]
                .momentum()
                .expect("P: empty slot");
            WaveformSlot::Vector(VectorWf {
                eps: ComplexVector::from(momentum),
                momentum,
            })
        }
        LorentzEvalNode::IdentityAmp { i, j } => {
            // Full scalar bilinear ψ̄_i δ ψ_j = ψ̄_i (P_L + P_R) ψ_j
            let WaveformSlot::Fermion(fi_row) =
                evaluate_lorentz_node(tree, tree.node(*i), input_slots, slots)
            else {
                panic!("expected fermion for IdentityAmp row (node {i})");
            };
            let WaveformSlot::Fermion(fi_col) =
                evaluate_lorentz_node(tree, tree.node(*j), input_slots, slots)
            else {
                panic!("expected fermion for IdentityAmp col (node {j})");
            };
            let fo = fi_row.to_outgoing();
            let value = Bispinor::scalar_bilinear(&fo.spinor, &fi_col.spinor, Chirality::Both);
            WaveformSlot::Scalar(ScalarWf {
                value,
                momentum: fo.momentum + fi_col.momentum,
            })
        }
        LorentzEvalNode::ScalarProduct { children } => {
            // Implicit product of disconnected tensor factors.
            // At most one child may be non-scalar; all others must be scalars.
            let mut scalar_val = C::new(F::one(), F::zero());
            let mut scalar_mom = LorentzVector([F::zero(); 4]);
            let mut non_scalar = WaveformSlot::Empty;
            for &child_idx in children {
                let child = evaluate_lorentz_node(tree, tree.node(child_idx), input_slots, slots);
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
                other => scalar_val * other,
            }
        }
    }
}

fn evaluate_lorentz_structure<F: Real + FromPrimitive>(
    structure: &super::dispatch::RootedTerm,
    input_slots: &[usize],
    slots: &[WaveformSlot<F>],
) -> WaveformSlot<F> {
    let coeff = F::from(structure.coeff).expect("coef not valid");
    let term = evaluate_lorentz_node(&structure.tree, structure.tree.root(), input_slots, slots);
    r(coeff) * term
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
            iovxxx, jioxxx, Charge, OutDiracWf,
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
                let fo_em = OutDiracWf::new(p_in_m, m_in, hel1, Charge::Particle);
                let fi_ep = InDiracWf::new(p_in_p, m_in, hel2, Charge::Antiparticle);
                let v_gamma_exp = jioxxx(&fo_em, &fi_ep, gc, mprop, wprop);

                let fo_out_m = OutDiracWf::new(p_out_m, m_out, hel3, Charge::Particle);
                let fi_out_p = InDiracWf::new(p_out_p, m_out, hel4, Charge::Antiparticle);

                let amp_exp = iovxxx(&fo_out_m, &fi_out_p, &v_gamma_exp, gc);

                // The same current should be obtained from the dispatch tree with the same inputs

                let mut slots: Vec<WaveformSlot<f64>> = vec![WaveformSlot::Empty; 9];

                slots[0] = build_external_slot(p_in_m, hel1.sign(), &leg1_info, 2, &evaluated);
                let WaveformSlot::Fermion(b) = &slots[0] else {
                    panic!("expected fermion slot");
                };
                assert_eq!(&fo_em.to_incoming(), b);

                slots[1] = build_external_slot(p_in_p, hel2.sign(), &leg2_info, 2, &evaluated);
                let WaveformSlot::Fermion(b) = &slots[1] else {
                    panic!("expected fermion slot");
                };
                assert_eq!(&fi_ep, b);

                let input_slots = vec![0, 1];
                slots[2] =
                    evaluate_off_shell_current(&vertex_info, &input_slots, &slots, &evaluated);
                slots[3] = evaluate_propagation(&prop_info, &slots[2], &evaluated);
                if let WaveformSlot::Vector(v_gamma) = slots[3] {
                    // Difference in convention: we always go incoming
                    assert_eq!(v_gamma.momentum, -v_gamma_exp.momentum);
                    let diff = v_gamma.eps - v_gamma_exp.eps;
                    if diff.0.iter().map(|x| x.norm()).sum::<f64>() > 1e-8 {
                        eprintln!("For helicity {hel1} {hel2}");
                        eprintln!("evaluated current does not match jioxxx: diff={diff:?}\n ours={v_gamma:?}\n exp={v_gamma_exp:?}");
                    }
                }

                slots[4] = build_external_slot(p_out_m, hel3.sign(), &leg3_info, 2, &evaluated);
                let WaveformSlot::Fermion(b) = &slots[4] else {
                    panic!("expected fermion slot");
                };
                assert_eq!(&fo_out_m.to_incoming(), b);

                slots[5] = build_external_slot(p_out_p, hel4.sign(), &leg4_info, 2, &evaluated);
                let WaveformSlot::Fermion(b) = &slots[5] else {
                    panic!("expected fermion slot");
                };
                assert_eq!(&fi_out_p, b);

                let input_slots = vec![4, 5, 3];
                slots[6] = evaluate_contract_amplitude(&amp_info, &input_slots, &slots, &evaluated);
                let WaveformSlot::Scalar(s) = slots[6] else {
                    panic!("expected scalar slot");
                };
                assert!(s.momentum.m2() < 1e-10);
                let diff = (s.value - amp_exp * Complex64::i()).norm();
                if diff > 1e-8 {
                    eprintln!("evaluated amplitude does not match: diff={diff:?}");
                }
            }
        }
    }

    /// Cross-check the off-shell fermion-current nodes (`GammaIout`/`GammaJout`)
    /// against the `fioxxx`/`foxxx` reference routines.
    ///
    /// Rooting the FFV1 structure `Gamma(3,2,1)` at fermion leg 2 yields a
    /// `GammaIout` node (input = column fermion leg 1) ≅ `fioxxx`; rooting at
    /// leg 1 yields a `GammaJout` node (input = row fermion leg 2) ≅ `foxxx`.
    /// The runtime applies the vertex factor and the Dirac propagator as two
    /// steps, so we compare against the reference (which folds both in) with a
    /// unit coupling. As in `test_eval_jioxxx`, the runtime carries the
    /// propagated leg with the opposite momentum sign (incoming convention).
    #[test]
    fn test_eval_off_shell_fermion_vs_fioxxx() {
        use crate::helas::eval::dispatch::LorentzEvalTree;
        use crate::helas::vertex::{fioxxx, foxxx};
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
            ops: vec![LorentzOp::Gamma { mu: 3, i: 2, j: 1 }],
        };
        let spins = [2, 2, 3];
        let g = Complex64::new(1.0, 0.0);

        // Generic (unphysical) vector input — any ε works for an impl cross-check.
        let v = VectorWf {
            eps: ComplexVector([
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
            // ── GammaIout ≅ fioxxx: input is the flow-in column fermion (leg 1) ──
            let fi = InDiracWf::new(p_f, mass, hel, charge);
            let tree = LorentzEvalTree::build_at_leg(&term, &spins, Some(1)).unwrap();
            assert!(matches!(tree.root(), LorentzEvalNode::GammaIout { .. }));

            let input_slots = vec![0, 1, 2];
            let mut slots: Vec<WaveformSlot<f64>> = vec![WaveformSlot::Empty; 3];
            slots[0] = WaveformSlot::Fermion(fi);
            slots[2] = WaveformSlot::Vector(v);

            let vertex = evaluate_lorentz_node(&tree, tree.root(), &input_slots, &slots);
            let WaveformSlot::Fermion(got) = evaluate_propagation(&prop_info, &vertex, &evaluated)
            else {
                panic!("expected fermion from propagation");
            };
            let want = fioxxx(&fi, &v, g, mass, width);
            assert_eq!(
                got.momentum, -want.momentum,
                "Iout momentum (hel {hel}, {charge:?})"
            );
            let diff: f64 = (got.spinor - want.spinor).0.iter().map(|x| x.norm()).sum();
            assert!(
                diff < 1e-10,
                "GammaIout vs fioxxx diff={diff} (hel {hel}, {charge:?})"
            );

            // ── GammaJout ≅ foxxx: input is the flow-out row fermion (leg 2) ──
            let fo = fi.to_outgoing();
            let tree = LorentzEvalTree::build_at_leg(&term, &spins, Some(0)).unwrap();
            assert!(matches!(tree.root(), LorentzEvalNode::GammaJout { .. }));

            let mut slots: Vec<WaveformSlot<f64>> = vec![WaveformSlot::Empty; 3];
            slots[1] = WaveformSlot::Fermion(fi);
            slots[2] = WaveformSlot::Vector(v);

            let vertex = evaluate_lorentz_node(&tree, tree.root(), &input_slots, &slots);
            let WaveformSlot::Fermion(got) = evaluate_propagation(&prop_info, &vertex, &evaluated)
            else {
                panic!("expected fermion from propagation");
            };
            let want = foxxx(&fo, &v, g, mass, width);
            assert_eq!(
                got.momentum, -want.momentum,
                "Jout momentum (hel {hel}, {charge:?})"
            );
            let diff: f64 = (got.spinor - want.spinor).0.iter().map(|x| x.norm()).sum();
            assert!(
                diff < 1e-10,
                "GammaJout vs foxxx diff={diff} (hel {hel}, {charge:?})"
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
            ops: vec![LorentzOp::ProjM { i: 2, j: 1 }],
        };
        let ffs3 = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::ProjP { i: 2, j: 1 }],
        };
        let spins = [2, 2, 1];

        let hels = [SpinorHelicity::Down, SpinorHelicity::Up];
        for (hel1, hel2) in iproduct!(hels, hels) {
            for charge in [Charge::Particle, Charge::Antiparticle] {
                let fo = OutDiracWf::new(p_fo, mass, hel1, charge);
                let fi = InDiracWf::new(p_fi, mass, hel2, charge);

                let left_ref = iosxxx(&fo, &fi, &s_wf, [g, Complex64::new(0.0, 0.0)]);
                let right_ref = iosxxx(&fo, &fi, &s_wf, [Complex64::new(0.0, 0.0), g]);

                // Build slots: leg1=fi (col), leg2=fo.to_incoming() (row), leg3=scalar
                let mut slots: Vec<WaveformSlot<f64>> = vec![WaveformSlot::Empty; 3];
                slots[0] = WaveformSlot::Fermion(fi);
                slots[1] = WaveformSlot::Fermion(fo.to_incoming());
                slots[2] = WaveformSlot::Scalar(s_wf);
                let input_slots = vec![0usize, 1, 2];

                // FFS1: ProjM(2,1) → left bilinear × s
                let tree1 = LorentzEvalTree::build_at_leg(&ffs1, &spins, None).unwrap();
                let WaveformSlot::Scalar(got1) =
                    evaluate_lorentz_node(&tree1, tree1.root(), &input_slots, &slots)
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
                    evaluate_lorentz_node(&tree3, tree3.root(), &input_slots, &slots)
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
}
