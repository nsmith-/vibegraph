//! Runtime amplitude evaluation: DiagramAst × momenta × helicities → amplitude

use crate::diagrams::DiagramSet;
use crate::helas::eval::compile::compile_diagram_ast;
use crate::helas::repr::intertwiner::{GammaL, GammaR, Intertwiner2Leg};
use crate::helas::repr::lorentz::{Bispinor, Charge, ComplexVector, LorentzVector, SpinorHelicity};
use crate::helas::repr::propagator::{
    DiracPropagator, MassiveVectorPropagator, MasslessVectorPropagator, Propagator,
    ScalarPropagator,
};
use crate::helas::repr::{Real, C};
use crate::helas::wavefn::{InDiracWf, OutDiracWf, ScalarWf, VectorWf};
use crate::ufo::particles::ParticleId;
use crate::ufo::{EvaluatedModel, UFOModel};
use num_traits::FromPrimitive;

use super::ast::{DiagramAst, EvalStep, WaveformSlot};
use super::compile::CompileError;

/// Compiled amplitude evaluator for all diagrams of a process.
///
/// The AST is built once from `&UFOModel`; coupling values are resolved at eval time
/// from `&EvaluatedModel` so the same evaluator works with any param card.
pub struct AmplitudeEvaluator {
    /// One compiled AST per diagram
    diagram_asts: Vec<DiagramAst>,
    /// Number of external particles
    n_ext: usize,
    /// Number of incoming external particles
    n_in: usize,
    /// External particle ids in process order (incoming first, then outgoing)
    ext_particle_ids: Vec<ParticleId>,
    /// External particle spins in process order
    ext_spins: Vec<i32>,
    /// Whether each external fermion leg is an antiparticle in process order
    ext_is_antiparticle: Vec<bool>,
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

        let ext_spins = ext_particle_ids
            .iter()
            .map(|&pid| model.particle(pid).spin)
            .collect::<Vec<_>>();

        let ext_charges = ext_particle_names
            .iter()
            .zip(ext_particle_ids.iter())
            .map(|(_, &pid)| model.particle(pid).charge > 0.0)
            .collect::<Vec<_>>();

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
            ext_spins,
            ext_is_antiparticle: ext_charges,
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
                acc + eval_single_diagram(
                    ast,
                    momenta,
                    helicities,
                    evaluated,
                    self.n_in,
                    &self.ext_spins,
                    &self.ext_is_antiparticle,
                )
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
    ext_spins: &[i32],
    ext_is_antiparticle: &[bool],
) -> C<F> {
    let mut slots = vec![WaveformSlot::Empty; ast.n_slots];

    for step in &ast.steps {
        match step {
            EvalStep::ExternalWf { info, output_slot } => {
                slots[*output_slot] = build_external_slot(
                    momenta[info.leg_idx],
                    helicities[info.leg_idx],
                    ext_spins[info.leg_idx],
                    ext_is_antiparticle[info.leg_idx],
                    info.leg_idx < n_in,
                    evaluated,
                    info.id,
                );
            }
            EvalStep::OffShellCurrent {
                info,
                result_leg_idx,
                input_slots,
                output_slot,
            } => {
                slots[*output_slot] = evaluate_off_shell_current(
                    info,
                    *result_leg_idx,
                    input_slots,
                    &slots,
                    evaluated,
                );
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
    spin_code: i32,
    is_antiparticle: bool,
    is_incoming: bool,
    evaluated: &EvaluatedModel,
    particle_id: ParticleId,
) -> WaveformSlot<F> {
    match spin_code.abs() {
        1 => WaveformSlot::Scalar(ScalarWf::sxxxxx(momentum, if is_incoming { -1 } else { 1 })),
        2 => {
            let hel = match helicity {
                -1 => SpinorHelicity::Down,
                1 => SpinorHelicity::Up,
                other => panic!("invalid fermion helicity {other}"),
            };
            let mass = real(evaluated.mass(particle_id));
            let wf = if is_incoming == is_antiparticle {
                OutDiracWf::new(momentum, mass, hel, Charge::Antiparticle).to_incoming()
            } else {
                InDiracWf::new(momentum, mass, hel, Charge::Particle)
            };
            WaveformSlot::Fermion(wf)
        }
        3 => {
            let mass = real(evaluated.mass(particle_id));
            let wf = VectorWf::vxxxxx(momentum, mass, helicity, if is_incoming { -1 } else { 1 });
            WaveformSlot::Vector(wf)
        }
        other => panic!("unsupported external spin code: {other}"),
    }
}

fn evaluate_off_shell_current<F: Real + FromPrimitive>(
    info: &super::ast::VertexInfo,
    result_leg_idx: usize,
    input_slots: &[usize],
    slots: &[WaveformSlot<F>],
    evaluated: &EvaluatedModel,
) -> WaveformSlot<F> {
    let mut accum = C::new(F::zero(), F::zero());
    let kind = match info.terms.first() {
        Some(term) => term.dispatch_kind,
        None => {
            return WaveformSlot::Scalar(ScalarWf {
                value: accum,
                momentum: LorentzVector::zero(),
            })
        }
    };

    for term in &info.terms {
        let coupling = complex_from_complex64::<F>(evaluated.coupling(term.coupling_id));
        let result_spin = term.spins[result_leg_idx].abs();

        let term_value = match (kind, result_spin) {
            (
                super::dispatch::DispatchKind::FfvProjM | super::dispatch::DispatchKind::FfvProjP,
                3,
            ) => {
                let (fo, fi) =
                    fermion_pair_from_slots(input_slots, slots, evaluated, result_leg_idx);
                let current = match term.dispatch_kind {
                    super::dispatch::DispatchKind::FfvProjM => {
                        GammaL::apply(&(fo.spinor, fi.spinor))
                    }
                    super::dispatch::DispatchKind::FfvProjP => {
                        GammaR::apply(&(fo.spinor, fi.spinor))
                    }
                    _ => unreachable!(),
                };
                let eps = std::array::from_fn(|mu| coupling * current.0[mu]);
                return WaveformSlot::Vector(VectorWf {
                    eps: ComplexVector(eps),
                    momentum: fo.momentum - fi.momentum,
                });
            }
            (super::dispatch::DispatchKind::Ffs, 1) => {
                let (fo, fi) =
                    fermion_pair_from_slots(input_slots, slots, evaluated, result_leg_idx);
                let left = fo.spinor.0[2] * fi.spinor.0[0] + fo.spinor.0[3] * fi.spinor.0[1];
                let right = fo.spinor.0[0] * fi.spinor.0[2] + fo.spinor.0[1] * fi.spinor.0[3];
                let momentum = fo.momentum + fi.momentum;
                return WaveformSlot::Scalar(ScalarWf {
                    value: coupling * (left + right),
                    momentum,
                });
            }
            (super::dispatch::DispatchKind::Vvv, 3) => {
                let v1 = vector_slot(&slots[input_slots[0]]);
                let v2 = vector_slot(&slots[input_slots[1]]);
                let q = v1.momentum + v2.momentum;
                let v1_eps = v1.eps.0;
                let v2_eps = v2.eps.0;

                let tmp1 = dot_q_complex(q.0, v2_eps);
                let tmp2 = dot_q_complex(v1.momentum.0, v2_eps);
                let tmp3 = dot_q_complex(q.0, v1_eps);
                let tmp4 = dot_q_complex(v2.momentum.0, v1_eps);
                let tmp5 = dot_complex(v1_eps, v2_eps);

                let eps: [C<F>; 4] = std::array::from_fn(|mu| {
                    coupling
                        * (tmp5 * C::new(v2.momentum[mu] - v1.momentum[mu], F::zero())
                            + v1_eps[mu] * (tmp1 - tmp2)
                            + v2_eps[mu] * (tmp3 - tmp4))
                });

                return WaveformSlot::Vector(VectorWf {
                    eps: ComplexVector(eps),
                    momentum: q,
                });
            }
            _ => C::new(F::zero(), F::zero()),
        };

        accum = accum + term_value;
    }

    WaveformSlot::Scalar(ScalarWf {
        value: accum,
        momentum: LorentzVector::zero(),
    })
}

fn evaluate_propagation<F: Real + FromPrimitive>(
    info: &super::ast::PropInfo,
    input: &WaveformSlot<F>,
    evaluated: &EvaluatedModel,
) -> WaveformSlot<F> {
    let mass = real(evaluated.mass(info.id));
    let width = real(evaluated.width(info.id));

    match input {
        WaveformSlot::Fermion(wf) => {
            let prop = DiracPropagator { mass, width };
            let propagated = prop.propagate(wf.momentum.0, wf.spinor.0);
            WaveformSlot::Fermion(InDiracWf::from_spinor(Bispinor(propagated), wf.momentum))
        }
        WaveformSlot::Vector(wf) => {
            if mass == F::zero() {
                let prop = MasslessVectorPropagator;
                WaveformSlot::Vector(VectorWf {
                    eps: ComplexVector(prop.propagate(wf.momentum.0, wf.eps.0)),
                    momentum: wf.momentum,
                })
            } else {
                let prop = MassiveVectorPropagator { mass, width };
                WaveformSlot::Vector(VectorWf {
                    eps: ComplexVector(prop.propagate(wf.momentum.0, wf.eps.0)),
                    momentum: wf.momentum,
                })
            }
        }
        WaveformSlot::Scalar(wf) => {
            let prop = ScalarPropagator { mass, width };
            WaveformSlot::Scalar(ScalarWf {
                value: prop.propagate(wf.momentum.0, wf.value),
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
    let mut value = C::new(F::zero(), F::zero());
    let kind = match info.terms.first() {
        Some(term) => term.dispatch_kind,
        None => {
            return WaveformSlot::Scalar(ScalarWf {
                value,
                momentum: LorentzVector::zero(),
            })
        }
    };

    for term in &info.terms {
        let coupling = complex_from_complex64::<F>(evaluated.coupling(term.coupling_id));
        let term_value = match kind {
            super::dispatch::DispatchKind::FfvProjM | super::dispatch::DispatchKind::FfvProjP => {
                let (fo, fi) = fermion_pair_from_slots(input_slots, slots, evaluated, usize::MAX);
                let v = vector_slot(&slots[input_slots[2]]);
                let current = match term.dispatch_kind {
                    super::dispatch::DispatchKind::FfvProjM => {
                        GammaL::apply(&(fo.spinor, fi.spinor))
                    }
                    super::dispatch::DispatchKind::FfvProjP => {
                        GammaR::apply(&(fo.spinor, fi.spinor))
                    }
                    _ => unreachable!(),
                };
                coupling * dot_complex(current.0, v.eps.0)
            }
            super::dispatch::DispatchKind::Ffs => {
                let (fo, fi) = fermion_pair_from_slots(input_slots, slots, evaluated, usize::MAX);
                let s = scalar_slot(&slots[input_slots[2]]);
                let left = fo.spinor.0[2] * fi.spinor.0[0] + fo.spinor.0[3] * fi.spinor.0[1];
                let right = fo.spinor.0[0] * fi.spinor.0[2] + fo.spinor.0[1] * fi.spinor.0[3];
                s.value * coupling * (left + right)
            }
            super::dispatch::DispatchKind::Vvv => {
                let v1 = vector_slot(&slots[input_slots[0]]);
                let v2 = vector_slot(&slots[input_slots[1]]);
                let v3 = vector_slot(&slots[input_slots[2]]);
                let q = v1.momentum + v2.momentum;
                let amp = dot_complex(v1.eps.0, v2.eps.0)
                    * dot_complex(
                        v3.eps.0,
                        [
                            C::new(q[0], F::zero()),
                            C::new(q[1], F::zero()),
                            C::new(q[2], F::zero()),
                            C::new(q[3], F::zero()),
                        ],
                    )
                    + dot_complex(v1.eps.0, v3.eps.0)
                    + dot_complex(v2.eps.0, v3.eps.0);
                coupling * amp
            }
            super::dispatch::DispatchKind::Vvs => {
                let v1 = vector_slot(&slots[input_slots[0]]);
                let v2 = vector_slot(&slots[input_slots[1]]);
                let s = scalar_slot(&slots[input_slots[2]]);
                coupling * s.value * dot_complex(v1.eps.0, v2.eps.0)
            }
            super::dispatch::DispatchKind::Sss | super::dispatch::DispatchKind::Ssss => {
                let mut product = coupling;
                for slot in input_slots {
                    product = product * scalar_slot(&slots[*slot]).value;
                }
                product
            }
            super::dispatch::DispatchKind::Vvvv => C::new(F::zero(), F::zero()),
        };
        value = value + term_value;
    }

    WaveformSlot::Scalar(ScalarWf {
        value,
        momentum: LorentzVector::zero(),
    })
}

fn fermion_pair_from_slots<F: Real>(
    input_slots: &[usize],
    slots: &[WaveformSlot<F>],
    _evaluated: &EvaluatedModel,
    _result_leg_idx: usize,
) -> (OutDiracWf<F>, InDiracWf<F>) {
    let first = match &slots[input_slots[0]] {
        WaveformSlot::Fermion(wf) => *wf,
        other => panic!("expected fermion slot, got {:?}", other),
    };
    let second = match &slots[input_slots[1]] {
        WaveformSlot::Fermion(wf) => *wf,
        other => panic!("expected fermion slot, got {:?}", other),
    };
    (first.to_outgoing(), second)
}

fn vector_slot<F: Real>(slot: &WaveformSlot<F>) -> VectorWf<F> {
    match slot {
        WaveformSlot::Vector(wf) => *wf,
        other => panic!("expected vector slot, got {:?}", other),
    }
}

fn scalar_slot<F: Real>(slot: &WaveformSlot<F>) -> ScalarWf<F> {
    match slot {
        WaveformSlot::Scalar(wf) => *wf,
        other => panic!("expected scalar slot, got {:?}", other),
    }
}

fn dot_complex<F: Real>(a: [C<F>; 4], b: [C<F>; 4]) -> C<F> {
    a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3]
}

fn dot_q_complex<F: Real>(q: [F; 4], c: [C<F>; 4]) -> C<F> {
    C::new(q[0], F::zero()) * c[0]
        - C::new(q[1], F::zero()) * c[1]
        - C::new(q[2], F::zero()) * c[2]
        - C::new(q[3], F::zero()) * c[3]
}

fn real<F: Real + FromPrimitive>(x: f64) -> F {
    <F as FromPrimitive>::from_f64(x).expect("value convertible to real scalar")
}

fn complex_from_complex64<F: Real + FromPrimitive>(x: num_complex::Complex64) -> C<F> {
    C::new(real(x.re), real(x.im))
}
