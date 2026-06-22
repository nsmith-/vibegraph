//! Runtime amplitude evaluation: a single forward pass over the folded `Ast`.
//!
//! [`BoundAmplitude`] holds a compiled [`AmplitudeEvaluator`] together with its
//! card-resolved constant pools (see [`BoundAmplitude::bind`]). For each
//! phase-space point it walks the arena in storage (topological) order, reducing each
//! node from its already-computed children via the single [`apply`] match.

use crate::helas::repr::lorentz::{
    Bispinor, ComplexVector, LorentzVector, SpinorFlow, SpinorRepr, VectorRepr,
};
use crate::helas::repr::numbers::{Charge, Chirality, SpinorHelicity};
use crate::helas::repr::{ri, Real, C};
use crate::helas::wavefn::{InDiracWf, OutDiracWf, ScalarWf, VectorWf};
use num_complex::ComplexFloat;
use num_traits::{FromPrimitive, Zero};

use super::ast::Ast;
use super::compile::AmplitudeEvaluator;
use super::op::{Const, Node, Op};
use super::tree::Tree;
use super::waveform_slot::WaveformSlot;
use crate::ufo::EvaluatedModel;

#[cfg(test)]
use super::fold::Folded;
#[cfg(test)]
use super::lower;
#[cfg(test)]
use super::root_diagram::{compile_diagram_ast, DiagramEval};

/// A compiled amplitude bound to a parameter card at scalar precision `F`.
///
/// Created by [`BoundAmplitude::bind`]: it borrows the card-independent
/// [`AmplitudeEvaluator`] and owns the resolved constant pools (`consts_c` couplings,
/// `consts_f` masses/widths/coeffs), so evaluation is pure kinematics — no parameter
/// lookups on the hot path.
#[derive(Debug)]
pub struct BoundAmplitude<'a, F: Real> {
    eval: &'a AmplitudeEvaluator,
    consts_c: Box<[C<F>]>,
    consts_f: Box<[F]>,
}

impl<'a, F: Real + FromPrimitive> BoundAmplitude<'a, F> {
    /// Resolve a parameter card at scalar precision `F` against a compiled
    /// [`AmplitudeEvaluator`], baking all couplings/masses/widths into the constant
    /// pools. The same evaluator binds against any card and precision.
    pub fn bind(eval: &'a AmplitudeEvaluator, evaluated: &EvaluatedModel) -> Self {
        let (consts_c, consts_f) = eval.folded().pools::<F>(evaluated);
        BoundAmplitude::new(eval, consts_c, consts_f)
    }

    /// Build from a compiled evaluator and its card-resolved pools (see [`bind`]).
    ///
    /// [`bind`]: BoundAmplitude::bind
    pub(super) fn new(
        eval: &'a AmplitudeEvaluator,
        consts_c: Box<[C<F>]>,
        consts_f: Box<[F]>,
    ) -> Self {
        BoundAmplitude {
            eval,
            consts_c,
            consts_f,
        }
    }

    /// The compiled (card-independent) evaluator this amplitude is bound to.
    pub fn evaluator(&self) -> &'a AmplitudeEvaluator {
        self.eval
    }

    /// Evaluate |M|² summed over all helicities.
    ///
    /// `momenta` are the external 4-momenta `[E, px, py, pz]`, incoming legs first then
    /// outgoing. Returns Σ_{helicities} |M|² (summed, not averaged).
    pub fn eval_m2(&self, momenta: &[LorentzVector<F>]) -> F {
        if momenta.len() != self.eval.n_ext() {
            return F::zero();
        }
        self.eval
            .helicities()
            .iter()
            .map(|hel| self.run(momenta, hel, None).norm_sqr())
            .fold(F::zero(), |acc, x| acc + x)
    }

    /// Evaluate the complex amplitude M for a single helicity configuration (the
    /// coherent sum over all diagrams).
    pub fn eval_amplitude(&self, momenta: &[LorentzVector<F>], helicities: &[i32]) -> C<F> {
        if momenta.len() != self.eval.n_ext() || helicities.len() != self.eval.n_ext() {
            return C::new(F::zero(), F::zero());
        }
        self.run(momenta, helicities, None)
    }

    /// Walk the folded arena for one (momenta, helicity) point. `ward_leg` gauge-
    /// substitutes one external boson's polarisation with its momentum (test-only).
    fn run(
        &self,
        momenta: &[LorentzVector<F>],
        helicities: &[i32],
        ward_leg: Option<usize>,
    ) -> C<F> {
        run_forward(
            &self.eval.folded().ast,
            &self.consts_c,
            &self.consts_f,
            momenta,
            helicities,
            ward_leg,
        )
    }

    /// Test-only: evaluate the amplitude with one external boson's polarisation ε^μ
    /// replaced by its 4-momentum q^μ (full-amplitude Ward-identity check).
    #[cfg(test)]
    fn eval_amplitude_ward(
        &self,
        momenta: &[LorentzVector<F>],
        helicities: &[i32],
        ward_leg: usize,
    ) -> C<F> {
        self.run(momenta, helicities, Some(ward_leg))
    }
}

/// Evaluate the folded arena in one forward pass, returning the root [`WaveformSlot`].
///
/// Nodes are visited in arena (storage) order; since children always have smaller ids
/// than their parents, each node's children are already computed and read from `res` by
/// id, so a shared (DAG) node is evaluated exactly once. For a whole amplitude the root
/// is a scalar; rooting a sub-tree (tests) can return any slot.
fn run_forward_slot<F: Real>(
    ast: &Ast<Const>,
    consts_c: &[C<F>],
    consts_f: &[F],
    momenta: &[LorentzVector<F>],
    helicities: &[i32],
    ward_leg: Option<usize>,
) -> WaveformSlot<F> {
    let mut res: Vec<WaveformSlot<F>> = Vec::with_capacity(ast.len());
    let mut kids: Vec<WaveformSlot<F>> = Vec::new();
    for id in ast.iter() {
        kids.clear();
        kids.extend(ast.children(id).map(|c| res[c as usize]));
        let value = apply(
            ast.value(id),
            &kids,
            momenta,
            helicities,
            consts_c,
            consts_f,
            ward_leg,
        );
        res.push(value);
    }
    res[ast.root() as usize]
}

/// Evaluate the whole-amplitude folded arena in one forward pass, returning the root
/// scalar = M.
fn run_forward<F: Real>(
    ast: &Ast<Const>,
    consts_c: &[C<F>],
    consts_f: &[F],
    momenta: &[LorentzVector<F>],
    helicities: &[i32],
    ward_leg: Option<usize>,
) -> C<F> {
    match run_forward_slot(ast, consts_c, consts_f, momenta, helicities, ward_leg) {
        WaveformSlot::Scalar(s) => s.value,
        WaveformSlot::Empty => C::new(F::zero(), F::zero()),
        other => panic!("amplitude root is not a scalar: {other:?}"),
    }
}

/// Reduce one folded node from its children's already-evaluated results. The single
/// match over `Op`: constant leaves resolve from the pools; `External`/`Propagate` build
/// wavefunctions; `Mul`/`Add` are the algebraic combinators; the Lorentz primitives
/// dispatch to the shared helpers below.
#[allow(clippy::too_many_arguments)]
fn apply<F: Real>(
    node: &Node<Const>,
    children: &[WaveformSlot<F>],
    momenta: &[LorentzVector<F>],
    helicities: &[i32],
    consts_c: &[C<F>],
    consts_f: &[F],
    ward_leg: Option<usize>,
) -> WaveformSlot<F> {
    match node.op {
        Op::Coupling => {
            let Const::Complex(i) = node.leaf else {
                panic!("Coupling node without a complex-pool index");
            };
            WaveformSlot::Scalar(ScalarWf {
                value: consts_c[i as usize],
                momentum: LorentzVector::zero(),
            })
        }
        Op::Mass | Op::Width | Op::Coeff => {
            let Const::Real(i) = node.leaf else {
                panic!("real-const node without a real-pool index");
            };
            WaveformSlot::Real(consts_f[i as usize])
        }
        Op::External => {
            let Const::Ext {
                leg_idx,
                spin,
                charge,
                incoming,
            } = node.leaf
            else {
                panic!("External node without leg info");
            };
            // Ward-identity gauge substitution (test-only): replace the chosen
            // external boson's polarisation ε^μ with its own 4-momentum q^μ. The
            // coherent diagram sum must then vanish (current conservation).
            if ward_leg == Some(leg_idx) {
                let q = momenta[leg_idx];
                return WaveformSlot::Vector(VectorWf {
                    eps: ComplexVector::from(q),
                    momentum: q,
                });
            }
            let mass = expect_real(children[0]);
            build_external_core(
                momenta[leg_idx],
                helicities[leg_idx],
                spin,
                charge,
                incoming,
                mass,
            )
        }
        Op::Propagate => {
            let mass = expect_real(children[1]);
            let width = expect_real(children[2]);
            propagate_core(&children[0], mass, width)
        }
        Op::Add => children
            .iter()
            .copied()
            .fold(WaveformSlot::Empty, |acc, x| acc + x),
        Op::Mul => mul_apply(children),
        Op::PMom => {
            let momentum = children[0].momentum().expect("PMom: empty slot");
            WaveformSlot::Vector(VectorWf {
                eps: ComplexVector::from(momentum),
                momentum,
            })
        }
        // Lorentz primitives: each reads its operands from `children` and dispatches to
        // the shared primitive helper below.
        Op::GammaVout => gamma_vout(children),
        Op::GammaIout | Op::GammaOout => off_shell_fermion_current(children[0], children[1]),
        Op::ProjM => chiral_project(children[0], Chirality::Left),
        Op::ProjP => chiral_project(children[0], Chirality::Right),
        Op::ProjMAmp => scalar_bilinear_current(children, Chirality::Left),
        Op::ProjPAmp => scalar_bilinear_current(children, Chirality::Right),
        Op::Metric => metric_contract(children),
        Op::MetricVout => metric_vout(children),
        Op::IdentityAmp => scalar_bilinear_current(children, Chirality::Both),
    }
}

/// Extract a bare real constant from a [`WaveformSlot::Real`] child.
fn expect_real<F: Real>(slot: WaveformSlot<F>) -> F {
    match slot {
        WaveformSlot::Real(r) => r,
        other => panic!("expected a real-constant slot, got {other:?}"),
    }
}

/// n-ary product (the `Mul` op). Scalar/real children fold into a complex coefficient
/// (reals kept in `F`); at most one non-scalar child carries the output type and absorbs
/// the scalar momentum.
fn mul_apply<F: Real>(children: &[WaveformSlot<F>]) -> WaveformSlot<F> {
    let mut real_acc = F::one();
    let mut cplx_acc = C::new(F::one(), F::zero());
    let mut scalar_mom = LorentzVector::zero();
    let mut non_scalar = WaveformSlot::Empty;
    for &child in children {
        match child {
            WaveformSlot::Real(r) => real_acc = real_acc * r,
            WaveformSlot::Scalar(s) => {
                cplx_acc = cplx_acc * s.value;
                scalar_mom = scalar_mom + s.momentum;
            }
            WaveformSlot::Empty => {}
            other => {
                assert!(
                    matches!(non_scalar, WaveformSlot::Empty),
                    "Mul: at most one non-scalar child"
                );
                non_scalar = other;
            }
        }
    }
    let coeff = cplx_acc * real_acc;
    match non_scalar {
        WaveformSlot::Empty => WaveformSlot::Scalar(ScalarWf {
            value: coeff,
            momentum: scalar_mom,
        }),
        // Route the scalar factors' momentum into the surviving non-scalar current so
        // the propagator sees the conserved q.
        other => match coeff * other {
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

/// Test helper: lower a single diagram (symmetry × Fermi sign folded in) and run the
/// unified forward pass, returning the root [`WaveformSlot`]. With a `ContractAmplitude`
/// root this is the scalar amplitude; rooting at an off-shell current returns that
/// current, which lets the cross-checks read an intermediate node through production.
#[cfg(test)]
fn eval_single_diagram_slot<F: Real + FromPrimitive>(
    diagram: &DiagramEval,
    momenta: &[LorentzVector<F>],
    helicities: &[i32],
    evaluated: &EvaluatedModel,
) -> WaveformSlot<F> {
    let symbolic = lower::lower(std::slice::from_ref(diagram));
    let folded = Folded::build(&symbolic);
    let (consts_c, consts_f) = folded.pools::<F>(evaluated);
    run_forward_slot(&folded.ast, &consts_c, &consts_f, momenta, helicities, None)
}

/// Test helper: the scalar amplitude of a single diagram (see
/// [`eval_single_diagram_slot`]). Used by the per-diagram probes.
#[cfg(test)]
fn eval_single_diagram<F: Real + FromPrimitive>(
    diagram: &DiagramEval,
    momenta: &[LorentzVector<F>],
    helicities: &[i32],
    evaluated: &EvaluatedModel,
) -> C<F> {
    match eval_single_diagram_slot(diagram, momenta, helicities, evaluated) {
        WaveformSlot::Scalar(s) => s.value,
        WaveformSlot::Empty => C::new(F::zero(), F::zero()),
        other => panic!("amplitude root is not a scalar: {other:?}"),
    }
}

/// Build an external wavefunction from its kinematics + interned mass.
fn build_external_core<F: Real>(
    momentum: LorentzVector<F>,
    helicity: i32,
    spin: i32,
    charge: Charge,
    is_incoming: bool,
    mass: F,
) -> WaveformSlot<F> {
    match spin {
        1 => WaveformSlot::Scalar(ScalarWf::sxxxxx(momentum, if is_incoming { -1 } else { 1 })),
        2 => {
            let hel = match helicity {
                -1 => SpinorHelicity::Down,
                1 => SpinorHelicity::Up,
                other => panic!("invalid fermion helicity {other}"),
            };
            // HELAS external flow: a leg is a ket (flow-in, ixxxxx) iff it is an
            // incoming particle or an outgoing antiparticle; otherwise it is a bra
            // (flow-out, oxxxxx). Equivalently flow-in ⟺ (is_incoming == is_particle).
            let is_particle = matches!(charge, Charge::Particle);
            if is_incoming == is_particle {
                WaveformSlot::FermionIn(InDiracWf::from_momentum(momentum, mass, hel, charge))
            } else {
                WaveformSlot::FermionOut(OutDiracWf::from_momentum(momentum, mass, hel, charge))
            }
        }
        3 => {
            let wf = VectorWf::vxxxxx(momentum, mass, helicity, if is_incoming { -1 } else { 1 });
            WaveformSlot::Vector(wf)
        }
        other => panic!("unsupported external spin code: {other}"),
    }
}

/// Apply a propagator with interned mass/width to an off-shell current. The current
/// already carries the conserved routed momentum (matching reference HELAS, where the
/// off-shell current routines output it: `fvixxx` q=fi−vc, `fvoxxx` q=fo+vc,
/// `jioxxx` jmom=fo−fi).
fn propagate_core<F: Real>(input: &WaveformSlot<F>, mass: F, width: F) -> WaveformSlot<F> {
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
        WaveformSlot::Real(_) => panic!("propagate step read a real-constant slot"),
        WaveformSlot::Empty => panic!("propagate step read an empty slot"),
    }
}

/// Resolve the two fermion legs of a bilinear into `(bra = flow-out, ket = flow-in,
/// reversed)` by their *actual* runtime flow, not the UFO `Gamma` i/j position. A
/// fermion line carries one flow throughout, so with physically-typed externals
/// (see `build_external_core`) and flow-preserving currents, the two fermions
/// meeting at any vertex always have opposite flow.
///
/// `reversed` is `true` when the slots arrive in `(flow-in, flow-out)` order, i.e.
/// the line runs against the vertex's defined flow; callers use it to apply the
/// flow-reversal sign η_Γ of their Lorentz structure.
fn resolve_bra_ket<F: Real>(
    a: WaveformSlot<F>,
    b: WaveformSlot<F>,
) -> (OutDiracWf<F>, InDiracWf<F>, bool) {
    match (a, b) {
        (WaveformSlot::FermionOut(fo), WaveformSlot::FermionIn(fi)) => (fo, fi, false),
        (WaveformSlot::FermionIn(fi), WaveformSlot::FermionOut(fo)) => (fo, fi, true),
        _ => panic!("a fermion bilinear needs exactly one flow-in and one flow-out leg"),
    }
}

/// Off-shell fermion current from an FFV `Gamma` vertex (one vector leg `mu` +
/// one continuing fermion leg `f`). The current **follows the input fermion's
/// flow**, so no mid-line Dirac adjoint is ever needed:
///   - flow-in (ket): `ε̸ψ`, q = f.p − v.p   (Fortran `fvixxx`)
///   - flow-out (bra): `ψ̄ε̸`, q = f.p + v.p   (Fortran `fvoxxx`)
///
/// `Bispinor::slash` is flow-dependent, so the left/right action is automatic.
/// The propagator `(q̸+m)/D` is applied in a separate `Propagate` step.
/// Continue a fermion line by slashing the input fermion `fermion` with the
/// vector current `v` (fvixxx/fvoxxx chosen at runtime by the fermion's flow).
fn off_shell_fermion_current<F: Real>(
    v: WaveformSlot<F>,
    fermion: WaveformSlot<F>,
) -> WaveformSlot<F> {
    let WaveformSlot::Vector(v) = v else {
        panic!("off-shell fermion current: expected vector input");
    };
    match fermion {
        WaveformSlot::FermionIn(fi) => WaveformSlot::FermionIn(InDiracWf::from_spinor(
            fi.spinor.slash(&v.eps),
            fi.momentum - v.momentum,
        )),
        WaveformSlot::FermionOut(fo) => WaveformSlot::FermionOut(OutDiracWf::from_spinor(
            fo.spinor.slash(&v.eps),
            fo.momentum + v.momentum,
        )),
        _ => panic!("off-shell fermion current: expected fermion input"),
    }
}

// ──────────────────────── Lorentz primitive helpers ────────────────────────
// Each takes the already-evaluated `children` in operand order; the runtime [`apply`]
// dispatches to these by `Op`. The cross-check tests call them directly.

/// `GammaVout`: two fermions → off-shell vector current `ψ̄ γ^μ ψ`.
fn gamma_vout<F: Real>(children: &[WaveformSlot<F>]) -> WaveformSlot<F> {
    let (fo, fi, reversed) = resolve_bra_ket(children[0], children[1]);
    let eps = fo.vector_bilinear(&fi, Chirality::Both);
    // Reading the fermion line against the vertex's defined flow conjugates the
    // structure as C γ^{μT} C⁻¹ = −γ^μ, so the vector current picks up a relative −1.
    // (Scalar/pseudoscalar structures have +1 and need no flip.)
    WaveformSlot::Vector(VectorWf {
        eps: if reversed { -eps } else { eps },
        momentum: fo.momentum - fi.momentum,
    })
}

/// `ProjM`/`ProjP`: chiral projection on a continuing fermion current, preserving the
/// input flow. `project_left`/`project_right` are flow-dependent (a bra projects
/// different components than a ket), so the same call is correct for both flows.
fn chiral_project<F: Real>(child: WaveformSlot<F>, chirality: Chirality) -> WaveformSlot<F> {
    fn project<F: Real, Fl: SpinorFlow>(
        s: Bispinor<F, Fl>,
        chirality: Chirality,
    ) -> Bispinor<F, Fl> {
        match chirality {
            Chirality::Left => s.project_left(),
            Chirality::Right => s.project_right(),
            Chirality::Both => s,
        }
    }
    match child {
        WaveformSlot::FermionIn(f) => WaveformSlot::FermionIn(InDiracWf::from_spinor(
            project(f.spinor, chirality),
            f.momentum,
        )),
        WaveformSlot::FermionOut(f) => WaveformSlot::FermionOut(OutDiracWf::from_spinor(
            project(f.spinor, chirality),
            f.momentum,
        )),
        _ => panic!("chiral projection: expected fermion input"),
    }
}

/// `Metric`: contract two vectors → scalar.
fn metric_contract<F: Real>(children: &[WaveformSlot<F>]) -> WaveformSlot<F> {
    let WaveformSlot::Vector(v1) = children[0] else {
        panic!("Metric: expected vector input");
    };
    let WaveformSlot::Vector(v2) = children[1] else {
        panic!("Metric: expected vector input");
    };
    WaveformSlot::Scalar(ScalarWf {
        value: v1.eps.dot(&v2.eps.lower()),
        momentum: v1.momentum + v2.momentum,
    })
}

/// `MetricVout`: off-shell vector current of a `Metric(out, v)` structure — the metric
/// raises the output index on the partner vector `v`. Matches ALOHA `VVS1P1N_1`
/// (`V1^0 = -i·V^0`, `V1^j = +i·V^j`, i.e. `-i·g·V`); the explicit `-i` is the vertex
/// factor on top of the coupling (the UFO GC for HVV already carries its own `i`). A
/// trailing scalar leg (the Higgs) multiplies in at the enclosing `Mul`.
fn metric_vout<F: Real>(children: &[WaveformSlot<F>]) -> WaveformSlot<F> {
    let WaveformSlot::Vector(vin) = children[0] else {
        panic!("MetricVout: expected vector input");
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

/// `ProjMAmp`/`ProjPAmp`/`IdentityAmp`: scalar bilinear `ψ̄ Γ ψ` (`Γ = P_L`, `P_R`, or
/// `1`); the bra/ket are picked by the legs' actual flow.
fn scalar_bilinear_current<F: Real>(
    children: &[WaveformSlot<F>],
    chirality: Chirality,
) -> WaveformSlot<F> {
    let (fo, fi_col, _) = resolve_bra_ket(children[0], children[1]);
    let value = Bispinor::scalar_bilinear(&fo.spinor, &fi_col.spinor, chirality);
    WaveformSlot::Scalar(ScalarWf {
        value,
        momentum: fo.momentum - fi_col.momentum,
    })
}

#[cfg(test)]
mod tests {
    use itertools::iproduct;
    use num_complex::Complex64;

    use super::*;
    use crate::{
        helas::{
            eval::diagram_eval::{ExtLegInfo, PropInfo, VertexInfo, VertexTerm},
            eval::root_diagram::{EvalNode, EvalNodeId},
            ffv2_4_3, iovxxx, jioxxx,
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
        // VVS1 `Metric(1,2)` rooted at vector leg 1 is a `MetricVout` current on the
        // partner vector V2, with the spectator scalar leg S multiplied in (the `Mul`
        // the rooted tree carries). Both primitives here are the production helpers.
        let out = mul_apply(&[
            metric_vout(&[WaveformSlot::Vector(v2)]),
            WaveformSlot::Scalar(s),
        ]);
        let WaveformSlot::Vector(out) = out else {
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
        for (mu, &exp) in expect.iter().enumerate() {
            let got = out.eps.component(mu);
            assert!(
                (got - exp).norm() < 1e-12,
                "component {mu}: got {got:?}, ALOHA expects {exp:?}",
            );
        }
        // Momentum is conserved through the vertex: q = p_V2 + p_S.
        assert_eq!(out.momentum, v2.momentum + s.momentum);
    }

    /// Cross-check the s-channel FFV current and amplitude — evaluated through the
    /// production `run_forward` path — against the `jioxxx`/`iovxxx` reference routines.
    ///
    /// Hand-built single diagrams (e⁺e⁻ → boson* [→ μ⁺μ⁻]) are assembled from `EvalNode`s
    /// and run through the same lower → fold → `run_forward` runtime used for real
    /// amplitudes. Rooting at the propagator returns the dressed s-channel current, which
    /// must equal `jioxxx` for every FFV structure (vector / left / left+2·right) and both
    /// the photon and Z propagators. For the unambiguous vector coupling (FFV1) the full
    /// μ⁺μ⁻ amplitude is also cross-checked against `iovxxx(·, jioxxx(·))`.
    #[test]
    fn test_eval_jioxxx() {
        let model = sm_model();
        let empty_card = "".parse::<ParamCard>().unwrap();
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
                incoming: true,
            };
            let leg2_info = ExtLegInfo {
                leg_idx: 1,
                id: inpart_p_id,
                spin: 2,
                charge: Charge::Antiparticle,
                incoming: true,
            };
            let leg3_info = ExtLegInfo {
                leg_idx: 2,
                id: outpart_id,
                spin: 2,
                charge: Charge::Particle,
                incoming: false,
            };
            let leg4_info = ExtLegInfo {
                leg_idx: 3,
                id: outpart_p_id,
                spin: 2,
                charge: Charge::Antiparticle,
                incoming: false,
            };
            let vertex_info = VertexInfo {
                terms: vec![VertexTerm::from_ufo(
                    model,
                    lorentz_id,
                    "asdf",
                    coupling_id,
                    Some(2),
                    None,
                )
                .unwrap()],
            };
            let prop_info = PropInfo { id: prop_id };
            let amp_info = VertexInfo {
                terms: vec![VertexTerm::from_ufo(
                    model,
                    lorentz_id,
                    "asdf",
                    coupling_id,
                    None,
                    None,
                )
                .unwrap()],
            };

            // Single s-channel current sub-diagram e⁺e⁻ → (FFV) → boson*: the two
            // externals feed the off-shell current (rooted at the vector leg), and the
            // propagator dresses it. Rooting at the propagator makes `run_forward` return
            // the dressed current itself (a vector), so we read it straight from the
            // production pass. Children reference earlier nodes by index.
            let current_diagram = DiagramEval::from_nodes(
                2,
                vec![
                    EvalNode::External(leg1_info.clone()),
                    EvalNode::External(leg2_info.clone()),
                    EvalNode::OffShellCurrent {
                        info: vertex_info.clone(),
                        flow: None,
                        children: vec![EvalNodeId::new(0), EvalNodeId::new(1)],
                    },
                    EvalNode::Propagate {
                        info: prop_info.clone(),
                        flow: None,
                        child: EvalNodeId::new(2),
                    },
                ],
            );
            // The full diagram extends it with the μ⁺μ⁻ sink contraction (a scalar M).
            let amp_diagram = DiagramEval::from_nodes(
                4,
                vec![
                    EvalNode::External(leg1_info),
                    EvalNode::External(leg2_info),
                    EvalNode::OffShellCurrent {
                        info: vertex_info,
                        flow: None,
                        children: vec![EvalNodeId::new(0), EvalNodeId::new(1)],
                    },
                    EvalNode::Propagate {
                        info: prop_info,
                        flow: None,
                        child: EvalNodeId::new(2),
                    },
                    EvalNode::External(leg3_info),
                    EvalNode::External(leg4_info),
                    EvalNode::ContractAmplitude {
                        info: amp_info,
                        children: vec![EvalNodeId::new(4), EvalNodeId::new(5), EvalNodeId::new(3)],
                    },
                ],
            );

            let hels = [SpinorHelicity::Down, SpinorHelicity::Up];
            for (hel1, hel2, hel3, hel4) in iproduct!(hels, hels, hels, hels) {
                // Physical flow (per the leg charge labels): leg1 (Particle, in) and
                // leg4 (Antiparticle, out) are kets; leg2 (Antiparticle, in) and
                // leg3 (Particle, out) are bras. The reference s-channel current is
                // jioxxx(fo=leg2 bra, fi=leg1 ket); the sink is iovxxx.
                let fi_em = InDiracWf::from_momentum(p_in_m, m_in, hel1, Charge::Particle);
                let fo_ep = OutDiracWf::from_momentum(p_in_p, m_in, hel2, Charge::Antiparticle);
                let v_gamma_exp = jioxxx(&fo_ep, &fi_em, gc, mprop, wprop);

                // The dressed s-channel current from the production pass must match jioxxx
                // exactly (value + routed momentum jmom = fo.p − fi.p), for every FFV
                // structure (vector / left / left+2·right) and both propagators.
                let WaveformSlot::Vector(v_gamma) = eval_single_diagram_slot(
                    &current_diagram,
                    &[p_in_m, p_in_p],
                    &[hel1.sign(), hel2.sign()],
                    &evaluated,
                ) else {
                    panic!("s-channel current must evaluate to a vector");
                };
                assert_eq!(
                    v_gamma.momentum, v_gamma_exp.momentum,
                    "current momentum ({coup_str}/{prop_name}, hel {hel1}{hel2})"
                );
                let cdiff: f64 = (v_gamma.eps - v_gamma_exp.eps).bare_norm_sq();
                assert!(
                    cdiff < 1e-8,
                    "current vs jioxxx ({coup_str}/{prop_name}, hel {hel1}{hel2}): diff={cdiff}"
                );

                // The vector (FFV1) coupling has no chirality ambiguity, so the full
                // amplitude reproduces the composed reference iovxxx∘jioxxx up to one
                // global convention factor of −i (the i the routines fold into the
                // amplitude vs. the i the UFO coupling carries at each vertex; it drops
                // out of |M|²). The pure-chiral FFV2/FFV4 sinks use a different HELAS
                // gc=[gl,gr] decomposition, so only their current is cross-checked above;
                // the chiral amplitude sink is covered by the full-process tests
                // (`test_whole_amplitude_equals_diagram_sum_eemumu`, `validate_helas`).
                if coup_str == "FFV1" {
                    let fo_out_m =
                        OutDiracWf::from_momentum(p_out_m, m_out, hel3, Charge::Particle);
                    let fi_out_p =
                        InDiracWf::from_momentum(p_out_p, m_out, hel4, Charge::Antiparticle);
                    let amp_exp = iovxxx(&fo_out_m, &fi_out_p, &v_gamma_exp, gc);

                    let momenta = [p_in_m, p_in_p, p_out_m, p_out_p];
                    let hel_codes = [hel1.sign(), hel2.sign(), hel3.sign(), hel4.sign()];
                    let got = eval_single_diagram(&amp_diagram, &momenta, &hel_codes, &evaluated);

                    let want = amp_exp * -Complex64::i();
                    let diff = (got - want).norm();
                    assert!(
                        diff < 1e-8,
                        "amplitude vs iovxxx∘jioxxx ({coup_str}/{prop_name}, \
                         hel {hel1}{hel2}{hel3}{hel4}): got={got:.6e} want={want:.6e} diff={diff}"
                    );
                }
            }
        }
    }

    /// Cross-check the production *combined* SM Z off-shell current — built through
    /// `run_forward` from a two-term (FFV2 ⊕ FFV4) vertex — against the ALOHA
    /// `FFV2_4_3` reference routine.
    ///
    /// `FFV2_4_3` adds the pure-left (FFV2, ProjM) and left+2·right (FFV4,
    /// ProjM + 2·ProjP) Lorentz structures with independent couplings — exactly the
    /// SM ℓ̄ℓZ current. Here both structures carry the same coupling `GC_3`, so the
    /// evaluator's combined current equals `jioxxx([2g, 2g])` and ALOHA's
    /// `ffv2_4_3(g, g)`. The two differ only by the global `−i` that ALOHA folds into
    /// each Lorentz structure while vibegraph carries it in the UFO coupling, so the
    /// production current matches `i · ffv2_4_3`. Both the massless photon (no
    /// longitudinal term) and the massive Z (OM3 ≠ 0, exercising the `P3·P3/M²`
    /// longitudinal subtraction) propagators are checked.
    #[test]
    fn test_eval_ffv2_4_3() {
        let model = sm_model();
        let evaluated = model.evaluate(&"".parse::<ParamCard>().unwrap());

        let coupling_id = model.coupling_id("GC_3").unwrap();
        let g = evaluated.coupling(coupling_id).im; // real chiral coupling

        let ffv2_id = model.lorentz_id("FFV2").unwrap();
        let ffv4_id = model.lorentz_id("FFV4").unwrap();

        let inpart_id = model.particle_id("e+").unwrap();
        let inpart_p_id = model.particle_id("e-").unwrap();
        let m_in = evaluated.mass(inpart_id);

        let leg1_info = ExtLegInfo {
            leg_idx: 0,
            id: inpart_id,
            spin: 2,
            charge: Charge::Particle,
            incoming: true,
        };
        let leg2_info = ExtLegInfo {
            leg_idx: 1,
            id: inpart_p_id,
            spin: 2,
            charge: Charge::Antiparticle,
            incoming: true,
        };

        // Two-term vertex: FFV2 (left) ⊕ FFV4 (left + 2·right), both with GC_3.
        let vertex_info = VertexInfo {
            terms: vec![
                VertexTerm::from_ufo(model, ffv2_id, "asdf", coupling_id, Some(2), None).unwrap(),
                VertexTerm::from_ufo(model, ffv4_id, "asdf", coupling_id, Some(2), None).unwrap(),
            ],
        };

        let i = Complex64::i();
        let hels = [SpinorHelicity::Down, SpinorHelicity::Up];
        // q² = s, so sqrts ≈ MZ drives the internal Z onto its pole — the regime where
        // the longitudinal q^μq^ν/m² subtraction dominates and any spinor-basis or OM3
        // mismatch would show up. sqrts = 1 keeps a deep-off-pole point for contrast.
        for (sqrts, prop_name) in iproduct!([1.0_f64, 91.188], ["a", "Z"]) {
            let p3_in = (sqrts * sqrts / 4.0 - m_in * m_in).sqrt();
            let p_in_m = LorentzVector::from_pxpypzmass(0.0, 0.0, -p3_in, m_in);
            let p_in_p = LorentzVector::from_pxpypzmass(0.0, 0.0, p3_in, m_in);

            let prop_id = model.particle_id(prop_name).unwrap();
            let mprop = evaluated.mass(prop_id);
            let wprop = evaluated.width(prop_id);

            let current_diagram = DiagramEval::from_nodes(
                2,
                vec![
                    EvalNode::External(leg1_info.clone()),
                    EvalNode::External(leg2_info.clone()),
                    EvalNode::OffShellCurrent {
                        info: vertex_info.clone(),
                        flow: None,
                        children: vec![EvalNodeId::new(0), EvalNodeId::new(1)],
                    },
                    EvalNode::Propagate {
                        info: PropInfo { id: prop_id },
                        flow: None,
                        child: EvalNodeId::new(2),
                    },
                ],
            );

            for (hel1, hel2) in iproduct!(hels, hels) {
                let fi_em = InDiracWf::from_momentum(p_in_m, m_in, hel1, Charge::Particle);
                let fo_ep = OutDiracWf::from_momentum(p_in_p, m_in, hel2, Charge::Antiparticle);

                // Literal ALOHA FFV2_4_3 reference: FFV2(g) + FFV4(g).
                let aloha = ffv2_4_3(
                    &fi_em,
                    &fo_ep,
                    Complex64::from(g),
                    Complex64::from(g),
                    mprop,
                    wprop,
                );

                // Faithfulness: the transcribed ALOHA current equals our validated
                // `jioxxx` in the equivalent [gL, gR] = [2g, 2g] chiral basis, times −i.
                let jio = jioxxx(&fo_ep, &fi_em, [2.0 * g, 2.0 * g], mprop, wprop);
                for mu in 0..4 {
                    let diff = (aloha.eps.component(mu) - (-i) * jio.eps.component(mu)).norm();
                    assert!(
                        diff < 1e-10,
                        "ffv2_4_3 vs −i·jioxxx (√s={sqrts}, {prop_name}, hel {hel1}{hel2}, μ={mu}): diff={diff}"
                    );
                }

                // Headline: the production combined current (run_forward) matches the
                // ALOHA reference up to the global −i UFO-coupling convention factor.
                let WaveformSlot::Vector(got) = eval_single_diagram_slot(
                    &current_diagram,
                    &[p_in_m, p_in_p],
                    &[hel1.sign(), hel2.sign()],
                    &evaluated,
                ) else {
                    panic!("combined Z current must evaluate to a vector");
                };
                assert_eq!(
                    got.momentum, aloha.momentum,
                    "current momentum (√s={sqrts}, {prop_name}, hel {hel1}{hel2})"
                );
                for mu in 0..4 {
                    let diff = (got.eps.component(mu) - i * aloha.eps.component(mu)).norm();
                    assert!(
                        diff < 1e-8,
                        "eval current vs i·ffv2_4_3 (√s={sqrts}, {prop_name}, hel {hel1}{hel2}, μ={mu}): diff={diff}"
                    );
                }
            }
        }
    }

    /// Ward identity for the off-shell **Z** current: built from a **massless** fermion
    /// pair it must be transverse, `q_μ J^μ = 0`, so the `q^μq^ν/m²` longitudinal piece
    /// of the massive-vector propagator decouples.
    ///
    /// This targets the one continuum-residual mechanism that survives every other test:
    /// the longitudinal Z mode on the massless spine. Unlike the external-photon Ward
    /// tests (`test_ward_identity_full_amplitude_*`), which only constrain the conserved
    /// **vector** current, this uses the real `ℓ̄ℓZ` couplings (FFV2·GC_50 + FFV4·GC_59,
    /// `gL ≠ gR`) so the current carries a genuine **axial** part — the parity-odd piece
    /// that, if its conservation were broken on the massless line, would leave a residual
    /// longitudinal contribution and reweight the L/R (parity-conjugate) helicities. The
    /// axial current's divergence is `∝ 2m·(pseudoscalar)`, so transversality is exact
    /// only for massless fermions; the contraction is checked at the very `q²/m_Z²`
    /// (`√s = m_Z`) where the longitudinal numerator is largest.
    #[test]
    fn test_longitudinal_z_current_transverse_for_massless_fermions() {
        let model = sm_model();
        let evaluated = model.evaluate(&"".parse::<ParamCard>().unwrap());

        // Real ℓ̄ℓZ vertex (SM V_107): FFV2·GC_50 (pure left) ⊕ FFV4·GC_59 (left+2·right),
        // i.e. gL = GC_50+GC_59, gR = 2·GC_59 — a parity-violating (gL ≠ gR) current.
        let gc50 = model.coupling_id("GC_50").unwrap();
        let gc59 = model.coupling_id("GC_59").unwrap();
        let ffv2_id = model.lorentz_id("FFV2").unwrap();
        let ffv4_id = model.lorentz_id("FFV4").unwrap();
        // Sanity: this is genuinely chiral (the axial part is non-trivial).
        assert_ne!(
            evaluated.coupling(gc50),
            evaluated.coupling(gc59),
            "test needs gL ≠ gR to exercise the axial current"
        );

        let inpart_id = model.particle_id("e+").unwrap();
        let inpart_p_id = model.particle_id("e-").unwrap();
        let m_in = evaluated.mass(inpart_id);
        assert_eq!(
            m_in, 0.0,
            "Ward identity requires massless producing fermions"
        );

        let leg1_info = ExtLegInfo {
            leg_idx: 0,
            id: inpart_id,
            spin: 2,
            charge: Charge::Particle,
            incoming: true,
        };
        let leg2_info = ExtLegInfo {
            leg_idx: 1,
            id: inpart_p_id,
            spin: 2,
            charge: Charge::Antiparticle,
            incoming: true,
        };

        let vertex_info = VertexInfo {
            terms: vec![
                VertexTerm::from_ufo(model, ffv2_id, "1", gc50, Some(2), None).unwrap(),
                VertexTerm::from_ufo(model, ffv4_id, "1", gc59, Some(2), None).unwrap(),
            ],
        };

        let z_id = model.particle_id("Z").unwrap();
        let hels = [SpinorHelicity::Down, SpinorHelicity::Up];
        // √s = m_Z drives q² to the pole region, maximising the longitudinal numerator.
        for sqrts in [1.0_f64, 91.1876] {
            let p3 = sqrts / 2.0; // massless ⇒ |p| = √s/2
            let p_in_m = LorentzVector::from_pxpypzmass(0.0, 0.0, -p3, 0.0);
            let p_in_p = LorentzVector::from_pxpypzmass(0.0, 0.0, p3, 0.0);

            let current_diagram = DiagramEval::from_nodes(
                2,
                vec![
                    EvalNode::External(leg1_info.clone()),
                    EvalNode::External(leg2_info.clone()),
                    EvalNode::OffShellCurrent {
                        info: vertex_info.clone(),
                        flow: None,
                        children: vec![EvalNodeId::new(0), EvalNodeId::new(1)],
                    },
                    EvalNode::Propagate {
                        info: PropInfo { id: z_id },
                        flow: None,
                        child: EvalNodeId::new(2),
                    },
                ],
            );

            // Track the largest current over helicities so transversality is not
            // vacuously satisfied (the chiral coupling kills the equal-helicity combos).
            let mut max_jnorm = 0.0_f64;
            for (hel1, hel2) in iproduct!(hels, hels) {
                let WaveformSlot::Vector(j) = eval_single_diagram_slot(
                    &current_diagram,
                    &[p_in_m, p_in_p],
                    &[hel1.sign(), hel2.sign()],
                    &evaluated,
                ) else {
                    panic!("Z current must evaluate to a vector");
                };

                // Minkowski contraction q·J = q⁰J⁰ − q⃗·J⃗ with the current's own momentum.
                let q = j.momentum;
                let qdotj = Complex64::from(q.e()) * j.eps.component(0)
                    - Complex64::from(q.px()) * j.eps.component(1)
                    - Complex64::from(q.py()) * j.eps.component(2)
                    - Complex64::from(q.pz()) * j.eps.component(3);

                let jnorm = (0..4)
                    .map(|k| j.eps.component(k).norm_sqr())
                    .sum::<f64>()
                    .sqrt();
                let qnorm =
                    (q.e() * q.e() + q.px() * q.px() + q.py() * q.py() + q.pz() * q.pz()).sqrt();
                max_jnorm = max_jnorm.max(jnorm);

                // q·J must vanish relative to |q||J| (absolute floor covers the
                // equal-helicity combos where the current itself is zero).
                assert!(
                    qdotj.norm() < 1e-9 * qnorm * jnorm + 1e-12,
                    "longitudinal Z fails to decouple for massless fermions \
                     (√s={sqrts}, hel {hel1}{hel2}): q·J={qdotj} vs |q||J|={}",
                    qnorm * jnorm
                );
            }
            assert!(
                max_jnorm > 1e-6,
                "Z current vacuously zero at all helicities (√s={sqrts})"
            );
        }
    }

    /// Cross-check the production off-shell fermion current (`off_shell_fermion_current`
    /// + `propagate_core`) against the `fvixxx`/`fvoxxx` reference routines.
    ///
    /// The current follows the input fermion's flow: seeding it from a flow-in (ket)
    /// fermion is `fvixxx`; from a flow-out (bra) fermion is `fvoxxx`. The runtime
    /// applies the bare γ^μ vertex structure and the Dirac propagator as two steps, so
    /// we compare against the reference (which folds both in) with a unit coupling. As
    /// in `test_eval_jioxxx`, the propagator carries the routed momentum unchanged.
    #[test]
    fn test_eval_off_shell_fermion_vs_fvixxx() {
        use crate::helas::vertex::{fvixxx, fvoxxx};

        let model = sm_model();
        let evaluated = model.evaluate(&"".parse::<ParamCard>().unwrap());

        // Off-shell fermion line propagates an electron.
        let prop_id = model.particle_id("e-").unwrap();
        let mass = evaluated.mass(prop_id);
        let width = evaluated.width(prop_id);

        // UFO convention: coupling includes i. fvixxx/fvoxxx fold the vertex factor in,
        // so we cross-check the bare structure + propagator at unit coupling [1, 1].
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
            // ── fvixxx: off-shell current seeded from the flow-in (ket) fermion ──
            let fi = InDiracWf::from_momentum(p_f, mass, hel, charge);
            let vertex =
                off_shell_fermion_current(WaveformSlot::Vector(v), WaveformSlot::FermionIn(fi));
            let WaveformSlot::FermionIn(got) = propagate_core(&vertex, mass, width) else {
                panic!("expected flow-in fermion from propagation");
            };
            let want = fvixxx(&fi, &v, [g.im, g.im], mass, width);
            // The fermion propagator carries the accumulated momentum unchanged
            // (no flip), matching fvixxx's `q = fi.p + v.p`.
            assert_eq!(
                got.momentum, want.momentum,
                "fvixxx momentum (hel {hel}, {charge:?})"
            );
            let diff: f64 = (got.spinor - want.spinor).bare_norm_sq();
            assert!(
                diff < 1e-10,
                "off-shell current vs fvixxx diff={diff} (hel {hel}, {charge:?})"
            );

            // ── fvoxxx: off-shell current seeded from the flow-out (bra) fermion ──
            // The current follows the input fermion's flow, so the input slot must
            // itself be flow-out (a bra) to produce a flow-out current.
            let fo = fi.to_outgoing();
            let vertex =
                off_shell_fermion_current(WaveformSlot::Vector(v), WaveformSlot::FermionOut(fo));
            let WaveformSlot::FermionOut(got) = propagate_core(&vertex, mass, width) else {
                panic!("expected flow-out fermion from propagation");
            };
            let want = fvoxxx(&fo, &v, [g.im, g.im], mass, width);
            assert_eq!(
                got.momentum, want.momentum,
                "fvoxxx momentum (hel {hel}, {charge:?})"
            );
            let diff: f64 = (got.spinor - want.spinor).bare_norm_sq();
            assert!(
                diff < 1e-10,
                "off-shell current vs fvoxxx diff={diff} (hel {hel}, {charge:?})"
            );
        }
    }

    /// Cross-check the production *chiral* off-shell fermion current — the path an
    /// e-line uses when it absorbs an internal **Z** (FFV2/FFV4, gL≠gR) — against the
    /// independent ALOHA `FFV2_2` routine.
    ///
    /// This is the one Z-specific fermion path never validated before: SESSION 6b's
    /// chain check used a pure-vector `γ q̸ γ` (no projector), and
    /// `test_eval_off_shell_fermion_vs_fvixxx` uses the vector coupling `gc=[g,g]`
    /// (P_L+P_R, projector-insensitive). The per-diagram matcher shows each internal Z
    /// injects a ~5% helicity-dependent error while photons (vector current) are exact,
    /// pointing straight at the chiral fermion current.
    ///
    /// The production tree for `Gamma(3,2,-1)·ProjM(-1,1)` rooted at the output fermion
    /// is `Propagate ∘ off_shell_fermion_current ∘ chiral_project(Left)`. It must equal
    /// our `fvixxx([1,0])` (self-consistency) and ALOHA `FFV2_2` up to the global `−i`
    /// UFO-coupling factor (`got = i·ffv2_2(1)`).
    #[test]
    fn test_chiral_off_shell_fermion_vs_ffv2_2() {
        use crate::helas::vertex::{ffv2_2, ffv4_2, fvixxx};

        // Generic vector input (transverse + longitudinal parts) — any ε exercises the
        // linear map; an internal-Z current is just one such ε.
        let v = VectorWf {
            eps: ComplexVector::new([
                Complex64::new(1.0, 0.0),
                Complex64::new(0.5, 0.2),
                Complex64::new(0.3, 0.1),
                Complex64::new(0.4, 0.0),
            ]),
            momentum: LorentzVector::new(50.0, 10.0, 0.0, 20.0),
        };
        let i = Complex64::i();
        let hels = [SpinorHelicity::Down, SpinorHelicity::Up];
        // Massless (electron line) and massive (exercises the FFV2_2 M2 terms F2(5,6)).
        for (mass, width) in [(0.0_f64, 0.0_f64), (0.106, 0.0)] {
            let p_f = LorentzVector::from_pxpypzmass(30.0, 0.0, 40.0, mass);
            for (hel, charge) in iproduct!(hels, [Charge::Particle, Charge::Antiparticle]) {
                let fi = InDiracWf::from_momentum(p_f, mass, hel, charge);

                // Production composition for the chiral (ProjM) fermion current.
                let projected = chiral_project(WaveformSlot::FermionIn(fi), Chirality::Left);
                let vertex = off_shell_fermion_current(WaveformSlot::Vector(v), projected);
                let WaveformSlot::FermionIn(got) = propagate_core(&vertex, mass, width) else {
                    panic!("expected flow-in fermion from chiral propagation");
                };

                // (a) Self-consistency: equals our pure-left fvixxx helper.
                let fvi = fvixxx(&fi, &v, [1.0, 0.0], mass, width);
                assert_eq!(got.momentum, fvi.momentum);
                let d_self: f64 = (got.spinor - fvi.spinor).bare_norm_sq();
                assert!(
                    d_self < 1e-10,
                    "chiral current vs fvixxx[1,0] (m={mass}, {hel}, {charge:?}): diff={d_self}"
                );

                // (b) Independent ALOHA FFV2_2 (the decisive check), up to the −i factor.
                let aloha = ffv2_2(&fi, &v, Complex64::from(1.0), mass, width);
                assert_eq!(
                    got.momentum, aloha.momentum,
                    "ffv2_2 momentum (m={mass}, {hel}, {charge:?})"
                );
                let d_aloha: f64 = (got.spinor - aloha.spinor * i).bare_norm_sq();
                assert!(
                    d_aloha < 1e-10,
                    "chiral current vs i·ffv2_2 (m={mass}, {hel}, {charge:?}): diff={d_aloha}"
                );

                // (c) Full Z fermion current FFV4 = P_L + 2·P_R (exercises the ProjP /
                // right path and its coefficient): the tree sums the two projected
                // slashes BEFORE the shared propagator.
                let WaveformSlot::FermionIn(left) = off_shell_fermion_current(
                    WaveformSlot::Vector(v),
                    chiral_project(WaveformSlot::FermionIn(fi), Chirality::Left),
                ) else {
                    unreachable!()
                };
                let WaveformSlot::FermionIn(right) = off_shell_fermion_current(
                    WaveformSlot::Vector(v),
                    chiral_project(WaveformSlot::FermionIn(fi), Chirality::Right),
                ) else {
                    unreachable!()
                };
                let summed = WaveformSlot::FermionIn(InDiracWf::from_spinor(
                    left.spinor + right.spinor * 2.0,
                    left.momentum,
                ));
                let WaveformSlot::FermionIn(got4) = propagate_core(&summed, mass, width) else {
                    unreachable!()
                };
                let aloha4 = ffv4_2(&fi, &v, Complex64::from(1.0), mass, width);
                let d4: f64 = (got4.spinor - aloha4.spinor * i).bare_norm_sq();
                assert!(
                    d4 < 1e-10,
                    "FFV4 chiral current vs i·ffv4_2 (m={mass}, {hel}, {charge:?}): diff={d4}"
                );
            }
        }
    }

    /// Validate **both leg rootings** of the production *chiral* off-shell fermion
    /// current against a textbook Dirac-matrix reconstruction (flow-IN / ket input).
    ///
    /// An FFV2/FFV4 vertex `ψ̄ γ^μ P ψ` rooted at a fermion output leg can land the
    /// projector on either side of the gamma:
    ///   • **leg-0** (`Some(0)`, `ProjM`/column leg): `Propagate ∘ chiral_project ∘
    ///     off_shell_fermion_current` = `P·ε̸·ψ` (projector AFTER the gamma);
    ///   • **leg-1** (`Some(1)`, gamma's row leg): `Propagate ∘ off_shell_fermion_current
    ///     ∘ chiral_project` = `ε̸·P·ψ` (projector BEFORE the gamma).
    /// Since `γ^μ P_L = P_R γ^μ`, the two carry OPPOSITE chirality — genuinely distinct
    /// code paths that ee→μμ (vector output) and the leg-2 tests never exercise. The
    /// per-diagram matcher + `test_espine_eline_z_absorption_ratio_vs_mg` show the
    /// production e+-spine uses **leg-1**, and that its **flow-OUT (bra) realization**
    /// mis-weights the hel-42 Z polarisation by 0.6403 vs MadGraph. This test confirms
    /// the **flow-IN (ket)** realization of *both* rootings is textbook-exact, so the
    /// residual is isolated to the flow-OUT (`GammaOout`) chiral path — not the ordering.
    ///
    /// Reference: `S(q)·P·ε̸·ψ` (leg-0) and `S(q)·ε̸·P·ψ` (leg-1), Dirac propagator
    /// `S(q) = (q̸ + m)/(q² − m²)`, `q = ψ.p − v.p`, built from explicit Weyl-basis γ
    /// matrices independent of the evaluator's representation. The evaluator carries an
    /// overall `−1` from `propagate_core`, so `eval == −1 · reference`. FFV2 uses
    /// `P = P_L`; FFV4 uses `P = P_L + 2P_R`.
    #[test]
    fn test_chiral_off_shell_fermion_espine_vs_textbook() {
        // Weyl basis γ^μ = [[0,σ^μ],[σ̄^μ,0]], σ^μ=(I,σ_i), σ̄^μ=(I,−σ_i); metric (+,−,−,−).
        type M4 = [[Complex64; 4]; 4];
        let z = Complex64::new(0.0, 0.0);
        let o = Complex64::new(1.0, 0.0);
        let ii = Complex64::new(0.0, 1.0);
        let g0: M4 = [[z, z, o, z], [z, z, z, o], [o, z, z, z], [z, o, z, z]];
        let g1: M4 = [[z, z, z, o], [z, z, o, z], [z, -o, z, z], [-o, z, z, z]];
        let g2: M4 = [[z, z, z, -ii], [z, z, ii, z], [z, ii, z, z], [-ii, z, z, z]];
        let g3: M4 = [[z, z, o, z], [z, z, z, -o], [-o, z, z, z], [z, o, z, z]];
        let matvec = |m: &M4, x: &[Complex64; 4]| -> [Complex64; 4] {
            core::array::from_fn(|r| (0..4).map(|c| m[r][c] * x[c]).sum())
        };
        let add = |a: [Complex64; 4], b: [Complex64; 4]| -> [Complex64; 4] {
            core::array::from_fn(|k| a[k] + b[k])
        };
        let scale = |s: Complex64, a: [Complex64; 4]| -> [Complex64; 4] {
            core::array::from_fn(|k| s * a[k])
        };
        // Covariant slash v̸ = γ^0 v^0 − γ^1 v^1 − γ^2 v^2 − γ^3 v^3.
        let slash = |v: &[Complex64; 4], x: &[Complex64; 4]| -> [Complex64; 4] {
            let mut r = scale(v[0], matvec(&g0, x));
            r = add(r, scale(-v[1], matvec(&g1, x)));
            r = add(r, scale(-v[2], matvec(&g2, x)));
            add(r, scale(-v[3], matvec(&g3, x)))
        };

        let v = VectorWf {
            eps: ComplexVector::new([
                Complex64::new(1.0, 0.0),
                Complex64::new(0.5, 0.2),
                Complex64::new(0.3, 0.1),
                Complex64::new(0.4, 0.0),
            ]),
            momentum: LorentzVector::new(50.0, 10.0, 0.0, 20.0),
        };
        let eps = core::array::from_fn(|k| v.eps.component(k));
        let hels = [SpinorHelicity::Down, SpinorHelicity::Up];
        for (mass, width) in [(0.0_f64, 0.0_f64), (0.106, 0.0)] {
            let p_f = LorentzVector::from_pxpypzmass(30.0, 0.0, 40.0, mass);
            for (hel, charge) in iproduct!(hels, [Charge::Particle, Charge::Antiparticle]) {
                let fi = InDiracWf::from_momentum(p_f, mass, hel, charge);
                let psi: [Complex64; 4] = core::array::from_fn(|k| fi.spinor.component(k));
                let q = fi.momentum - v.momentum;
                let qvec = [
                    Complex64::from(q.e()),
                    Complex64::from(q.px()),
                    Complex64::from(q.py()),
                    Complex64::from(q.pz()),
                ];
                let denom = Complex64::from(q.m2() - mass * mass);

                // Textbook S(q)·P·ε̸·ψ for P = P_L (FFV2) and P = P_L + 2P_R (FFV4),
                // with the evaluator's overall −1 from propagate_core folded in.
                let textbook = |pl: Complex64, pr: Complex64| -> [Complex64; 4] {
                    let eps_psi = slash(&eps, &psi);
                    // P · ε̸ψ with chiral weights (P_L keeps [0,1], P_R keeps [2,3]).
                    let projected = [
                        pl * eps_psi[0],
                        pl * eps_psi[1],
                        pr * eps_psi[2],
                        pr * eps_psi[3],
                    ];
                    // (q̸ + m)/(q²−m²), then ×(−1).
                    let qslash = slash(&qvec, &projected);
                    let massterm = scale(Complex64::from(mass), projected);
                    scale(-o / denom, add(qslash, massterm))
                };

                // ── FFV2 e-spine current: ProjM(ε̸·ψ) propagated ─────────────────
                let curr =
                    off_shell_fermion_current(WaveformSlot::Vector(v), WaveformSlot::FermionIn(fi));
                let WaveformSlot::FermionIn(got2) =
                    propagate_core(&chiral_project(curr, Chirality::Left), mass, width)
                else {
                    panic!("expected flow-in fermion from chiral propagation");
                };
                let want2 = textbook(o, z);
                for k in 0..4 {
                    let d = (got2.spinor.component(k) - want2[k]).norm();
                    assert!(
                        d < 1e-10,
                        "FFV2 e-spine vs textbook (m={mass}, {hel}, {charge:?}, comp {k}): {d}"
                    );
                }

                // ── FFV4 e-spine current: ProjM(ε̸ψ) + 2·ProjP(ε̸ψ) propagated ───
                let mk = |chi| {
                    let WaveformSlot::FermionIn(c) = chiral_project(
                        off_shell_fermion_current(
                            WaveformSlot::Vector(v),
                            WaveformSlot::FermionIn(fi),
                        ),
                        chi,
                    ) else {
                        unreachable!()
                    };
                    c
                };
                let left = mk(Chirality::Left);
                let right = mk(Chirality::Right);
                let summed = WaveformSlot::FermionIn(InDiracWf::from_spinor(
                    left.spinor + right.spinor * 2.0,
                    left.momentum,
                ));
                let WaveformSlot::FermionIn(got4) = propagate_core(&summed, mass, width) else {
                    unreachable!()
                };
                let want4 = textbook(o, Complex64::new(2.0, 0.0));
                for k in 0..4 {
                    let d = (got4.spinor.component(k) - want4[k]).norm();
                    assert!(
                        d < 1e-10,
                        "FFV4 e-spine vs textbook (m={mass}, {hel}, {charge:?}, comp {k}): {d}"
                    );
                }

                // ══ The OTHER rooting: leg-1 (`Some(1)`), projector BEFORE the gamma ══
                // The production e+-spine Z absorption roots the FFV2/FFV4 vertex at the
                // gamma's row/output leg, giving `GammaXout(V, ProjM(F))` = `ε̸·P_χ·ψ` —
                // the mirror of the leg-0 `P_χ·ε̸·ψ` above. Since `γ^μ P_L = P_R γ^μ` the
                // two rootings carry OPPOSITE chirality, so this is a genuinely distinct
                // current. (This is the rooting `test_espine_eline_z_absorption_ratio_vs_mg`
                // shows is the production-consistent one — matches MadGraph at hel 38.)
                //
                // NOTE ON FLOW: here the input is a flow-IN ket, so this exercises the
                // `fvixxx`/`GammaIout` realization, which equals ALOHA FFV2_2 (see
                // `test_chiral_off_shell_fermion_vs_ffv2_2`). The production e+ spine is a
                // flow-OUT bra (`GammaOout`) — its 0.6403 hel-42 mismatch vs MadGraph is
                // pinned by `test_espine_eline_z_absorption_ratio_vs_mg`; that flow-out
                // realization is the remaining isolated suspect this ket path does not cover.
                let textbook_proj_first = |pl: Complex64, pr: Complex64| -> [Complex64; 4] {
                    // ε̸ · (P_L+P_R-weighted ψ), then (q̸+m)/(q²−m²), then ×(−1).
                    let projected = [pl * psi[0], pl * psi[1], pr * psi[2], pr * psi[3]];
                    let eps_proj = slash(&eps, &projected);
                    let qslash = slash(&qvec, &eps_proj);
                    let massterm = scale(Complex64::from(mass), eps_proj);
                    scale(-o / denom, add(qslash, massterm))
                };

                // FFV2 leg-1: ε̸·P_L·ψ propagated.
                let WaveformSlot::FermionIn(g2b) = propagate_core(
                    &off_shell_fermion_current(
                        WaveformSlot::Vector(v),
                        chiral_project(WaveformSlot::FermionIn(fi), Chirality::Left),
                    ),
                    mass,
                    width,
                ) else {
                    panic!("expected flow-in fermion");
                };
                let want2b = textbook_proj_first(o, z);
                for k in 0..4 {
                    let d = (g2b.spinor.component(k) - want2b[k]).norm();
                    assert!(
                        d < 1e-10,
                        "FFV2 leg-1 (ε̸·P_L·ψ) vs textbook (m={mass}, {hel}, {charge:?}, comp {k}): {d}"
                    );
                }

                // FFV4 leg-1: ε̸·(P_L+2P_R)·ψ propagated — project the INPUT first, slash
                // after (mirror of `mk`, which projects after the slash for leg-0).
                let mk1 = |chi| {
                    let WaveformSlot::FermionIn(c) = off_shell_fermion_current(
                        WaveformSlot::Vector(v),
                        chiral_project(WaveformSlot::FermionIn(fi), chi),
                    ) else {
                        unreachable!()
                    };
                    c
                };
                let l1 = mk1(Chirality::Left);
                let r1 = mk1(Chirality::Right);
                let summed1 = WaveformSlot::FermionIn(InDiracWf::from_spinor(
                    l1.spinor + r1.spinor * 2.0,
                    l1.momentum,
                ));
                let WaveformSlot::FermionIn(g4b) = propagate_core(&summed1, mass, width) else {
                    unreachable!()
                };
                let want4b = textbook_proj_first(o, Complex64::new(2.0, 0.0));
                for k in 0..4 {
                    let d = (g4b.spinor.component(k) - want4b[k]).norm();
                    assert!(
                        d < 1e-10,
                        "FFV4 leg-1 (ε̸·(P_L+2P_R)·ψ) vs textbook (m={mass}, {hel}, {charge:?}, comp {k}): {d}"
                    );
                }
            }
        }
    }

    /// THE BUG: the flow-OUT (bra) leg-1 chiral absorption projects the bra on the WRONG
    /// SIDE of the slash — the source of the hel-42 0.6403 reweighting
    /// (`test_espine_eline_z_absorption_ratio_vs_mg`).
    ///
    /// The leg-1 rooting (`GammaXout(V, ProjM(F))`) projects the *input* fermion, then
    /// slashes. For a **ket** input that gives `ε̸·(P_χ ψ)` = `ε̸·P_χ·ψ` — projector
    /// adjacent to the ket (right side), which IS the physical vertex (the flow-IN leg-1
    /// path in `test_chiral_off_shell_fermion_espine_vs_textbook` passes). For a **bra**
    /// input it gives `(ψ̄·P_χ)·ε̸` = `ψ̄·P_χ·ε̸` — projector adjacent to the bra (left
    /// side). But the physical vertex `(bra) γ^μ P_χ (ket)` (established by the validated
    /// leg-2 jioxxx current) requires the projector on the FAR side: `ψ̄·ε̸·P_χ`. The two
    /// differ by `ψ̄·ε̸·(P_L−P_R)·S = ψ̄·ε̸·γ5·S`, which is POLARISATION-dependent — it
    /// (near-)vanishes for the hel-38 Z polarisation but not hel-42, giving the
    /// helicity-dependent 0.64 (a flat gL↔gR swap on a massless line would be constant).
    ///
    /// This test reconstructs a storage-independent **bra** textbook via the bilinear
    /// scalar `R_out · χ_ref` (a flow-out spinor dotted with a ket gives the Lorentz
    /// scalar `ψ̄ … χ`), validated FIRST on the chirality-blind photon. It then PINS the
    /// eval's current (buggy) behaviour `ψ̄·P_χ·ε̸·S` and asserts it differs from the
    /// physical `ψ̄·ε̸·P_χ·S`. **Flip these to the gamma-first form when the rooting/flow
    /// is fixed** (so a flow-out leg-1 bra projects after the slash).
    #[test]
    fn test_chiral_off_shell_fermion_flowout_vs_textbook() {
        type M4 = [[Complex64; 4]; 4];
        let z = Complex64::new(0.0, 0.0);
        let o = Complex64::new(1.0, 0.0);
        let ii = Complex64::new(0.0, 1.0);
        let g0: M4 = [[z, z, o, z], [z, z, z, o], [o, z, z, z], [z, o, z, z]];
        let g1: M4 = [[z, z, z, o], [z, z, o, z], [z, -o, z, z], [-o, z, z, z]];
        let g2: M4 = [[z, z, z, -ii], [z, z, ii, z], [z, ii, z, z], [-ii, z, z, z]];
        let g3: M4 = [[z, z, o, z], [z, z, z, -o], [-o, z, z, z], [z, o, z, z]];
        let matmul = |a: &M4, b: &M4| -> M4 {
            core::array::from_fn(|r| {
                core::array::from_fn(|c| (0..4).map(|k| a[r][k] * b[k][c]).sum())
            })
        };
        let matadd = |a: &M4, b: &M4| -> M4 {
            core::array::from_fn(|r| core::array::from_fn(|c| a[r][c] + b[r][c]))
        };
        let smul = |s: Complex64, a: &M4| -> M4 {
            core::array::from_fn(|r| core::array::from_fn(|c| s * a[r][c]))
        };
        let ident: M4 =
            core::array::from_fn(|r| core::array::from_fn(|c| if r == c { o } else { z }));
        // Slash matrix v̸ = γ^0 v0 − γ^1 v1 − γ^2 v2 − γ^3 v3 (contravariant v).
        let slashm = |v: &[Complex64; 4]| -> M4 {
            let mut m = smul(v[0], &g0);
            m = matadd(&m, &smul(-v[1], &g1));
            m = matadd(&m, &smul(-v[2], &g2));
            matadd(&m, &smul(-v[3], &g3))
        };
        let rowmat = |r: &[Complex64; 4], m: &M4| -> [Complex64; 4] {
            core::array::from_fn(|c| (0..4).map(|k| r[k] * m[k][c]).sum())
        };
        let dot = |a: &[Complex64; 4], b: &[Complex64; 4]| -> Complex64 {
            (0..4).map(|k| a[k] * b[k]).sum()
        };

        let v = VectorWf {
            eps: ComplexVector::new([
                Complex64::new(1.0, 0.0),
                Complex64::new(0.5, 0.2),
                Complex64::new(0.3, 0.1),
                Complex64::new(0.4, 0.0),
            ]),
            momentum: LorentzVector::new(50.0, 10.0, 0.0, 20.0),
        };
        let eps: [Complex64; 4] = core::array::from_fn(|k| v.eps.component(k));
        let eslash = slashm(&eps);
        // Arbitrary reference ket (probes all components).
        let chi_ref = [
            Complex64::new(0.7, 0.1),
            Complex64::new(-0.2, 0.4),
            Complex64::new(0.5, -0.3),
            Complex64::new(0.1, 0.6),
        ];
        let proj = |pl: Complex64, pr: Complex64| -> M4 {
            let mut m = ident;
            m[0][0] = pl;
            m[1][1] = pl;
            m[2][2] = pr;
            m[3][3] = pr;
            m
        };

        let hels = [SpinorHelicity::Down, SpinorHelicity::Up];
        for (mass, width) in [(0.0_f64, 0.0_f64), (0.106, 0.0)] {
            let p_f = LorentzVector::from_pxpypzmass(30.0, 0.0, 40.0, mass);
            for (hel, charge) in iproduct!(hels, [Charge::Particle, Charge::Antiparticle]) {
                let fo = OutDiracWf::from_momentum(p_f, mass, hel, charge);
                let ket = InDiracWf::from_momentum(p_f, mass, hel, charge);
                let psi: [Complex64; 4] = core::array::from_fn(|k| ket.spinor.component(k));
                // Physical bra ψ̄ = ψ† γ⁰  (OutDiracWf::from_momentum == bar(InDiracWf)).
                let psibar: [Complex64; 4] =
                    core::array::from_fn(|j| (0..4).map(|i| psi[i].conj() * g0[i][j]).sum());

                // q = fo.p + v.p (flow-out current momentum); S(q) with eval's −1.
                let q = fo.momentum + v.momentum;
                let qvec = [
                    Complex64::from(q.e()),
                    Complex64::from(q.px()),
                    Complex64::from(q.py()),
                    Complex64::from(q.pz()),
                ];
                let denom = Complex64::from(q.m2() - mass * mass);
                let sprop = smul(
                    -o / denom,
                    &matadd(&slashm(&qvec), &smul(Complex64::from(mass), &ident)),
                );

                // Textbook scalar for a bra operator `op` (applied to ψ̄ from the right),
                // propagated, then contracted with the reference ket.
                let textbook = |op: &M4| -> Complex64 {
                    let r = rowmat(&psibar, op);
                    let r = rowmat(&r, &sprop);
                    dot(&r, &chi_ref)
                };
                // Eval flow-out scalar: dot the produced bra with the reference ket.
                let eval_scalar = |r: &OutDiracWf<f64>| -> Complex64 {
                    let comps: [Complex64; 4] = core::array::from_fn(|k| r.spinor.component(k));
                    dot(&comps, &chi_ref)
                };

                // ── Photon (no projector): validates the bra machinery end-to-end ──
                let WaveformSlot::FermionOut(ph) = propagate_core(
                    &off_shell_fermion_current(
                        WaveformSlot::Vector(v),
                        WaveformSlot::FermionOut(fo),
                    ),
                    mass,
                    width,
                ) else {
                    panic!("expected flow-out fermion");
                };
                let s_ph_eval = eval_scalar(&ph);
                let s_ph_book = textbook(&eslash); // ψ̄·ε̸·S
                let d_ph = (s_ph_eval - s_ph_book).norm();
                assert!(
                    d_ph < 1e-9 * (s_ph_book.norm() + 1.0),
                    "PHOTON flow-out bra recon failed (m={mass}, {hel}, {charge:?}): \
                     eval={s_ph_eval:.5e} book={s_ph_book:.5e} d={d_ph:.2e}"
                );

                // Project-bra-first builder for the eval flow-out current.
                let eval_flowout = |proj_chi: Option<Chirality>, tworight: bool| -> Complex64 {
                    // FFV2: single P_L term. FFV4: P_L + 2·P_R.
                    let build = |chi: Chirality| {
                        let WaveformSlot::FermionOut(c) = off_shell_fermion_current(
                            WaveformSlot::Vector(v),
                            chiral_project(WaveformSlot::FermionOut(fo.clone()), chi),
                        ) else {
                            unreachable!()
                        };
                        c
                    };
                    let combined = match (proj_chi, tworight) {
                        (Some(chi), false) => build(chi).spinor, // FFV2: P_L only
                        (None, true) => {
                            build(Chirality::Left).spinor + build(Chirality::Right).spinor * 2.0
                        } // FFV4
                        _ => unreachable!(),
                    };
                    let WaveformSlot::FermionOut(r) = propagate_core(
                        &WaveformSlot::FermionOut(OutDiracWf::from_spinor(
                            combined,
                            fo.momentum + v.momentum,
                        )),
                        mass,
                        width,
                    ) else {
                        unreachable!()
                    };
                    eval_scalar(&r)
                };

                // The eval projects the bra BEFORE slashing: `ψ̄·P_χ·ε̸·S`. Because
                // `P_L·ε̸ = ε̸·P_R`, that is the OPPOSITE-chirality action of the nominal
                // `P_L` vertex on the bra — the precise flow-out structure, here pinned
                // against the textbook for every helicity/charge/mass.
                // FFV2 (P_L vertex):
                let s_ffv2 = eval_flowout(Some(Chirality::Left), false);
                let book_ffv2 = textbook(&matmul(&proj(o, z), &eslash)); // ψ̄·P_L·ε̸·S
                let d2 = (s_ffv2 - book_ffv2).norm();
                assert!(
                    d2 < 1e-9 * (book_ffv2.norm() + 1.0),
                    "FFV2 flow-out (m={mass}, {hel}, {charge:?}): eval={s_ffv2:.5e} vs ψ̄·P_L·ε̸·S={book_ffv2:.5e}, d={d2:.2e}"
                );
                // It must NOT equal the gamma-first `ψ̄·ε̸·P_L·S` (off the massless point).
                let book_gammafirst = textbook(&matmul(&eslash, &proj(o, z)));
                if mass == 0.0 && (book_ffv2 - book_gammafirst).norm() > 1e-6 {
                    assert!(
                        (s_ffv2 - book_gammafirst).norm() > 1e-6,
                        "FFV2 flow-out unexpectedly matched the gamma-first ordering"
                    );
                }

                // FFV4 (P_L + 2P_R vertex):
                let s_ffv4 = eval_flowout(None, true);
                // Eval order: ψ̄·(P_L+2P_R)·ε̸·S.
                let book_ffv4 = textbook(&matmul(
                    &matadd(&proj(o, z), &smul(o + o, &proj(z, o))),
                    &eslash,
                ));
                let d4 = (s_ffv4 - book_ffv4).norm();
                assert!(
                    d4 < 1e-9 * (book_ffv4.norm() + 1.0),
                    "FFV4 flow-out (m={mass}, {hel}, {charge:?}): eval={s_ffv4:.5e} vs ψ̄·(P_L+2P_R)·ε̸·S={book_ffv4:.5e}, d={d4:.2e}"
                );
            }
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
            .and_then(|s| s.parse::<ParamCard>().ok())
            .expect("ee_to_mumu_tata_qcd0 param_card_masslesstau.dat");
        let evaluated = model.evaluate(&card);

        let opts = ParsingOptions::default();
        let pc = parse_proc_card("generate e+ e- > mu+ mu- ta+ ta- QCD=0", &opts).unwrap();
        let sets = generate_from_proc_card(&pc, model).unwrap();
        let set = &sets[0];

        let evaluator = AmplitudeEvaluator::compile(set, model).expect("should compile");
        let ast = &evaluator.folded().ast;
        eprintln!("folded ast = {}", ast);

        let asts = compile_diagram_ast(set, model).unwrap();
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

        let prop_sig = |ast: &DiagramEval| -> String {
            let names: Vec<String> = ast
                .propagator_particles()
                .map(|id| model.particle(id).name.clone())
                .collect();
            // don't sort so we can tell e.g. a+ta-+Z from Z+ta-+a
            // names.sort();
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
        // The e-spine relative −1 vs MadGraph is carried by `fermi_sign`
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
                .map(|ast| eval_single_diagram(ast, &p, hel, &evaluated))
                .collect();
            let a_tot: C<f64> = amps.iter().fold(C::new(0.0, 0.0), |a, b| a + *b);
            m2_total += a_tot.norm_sqr();
            for (i, a) in amps.iter().enumerate() {
                diag[i] += a.norm_sqr();
                rrow[i] += a.conj() * a_tot;
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

    /// Cross-check the production scalar bilinear (`scalar_bilinear_current` × scalar
    /// leg) against the `iosxxx` reference routine.
    ///
    /// FFS1 (`ProjM`) is the left bilinear `ψ̄ P_L ψ`; FFS3 (`ProjP`) the right one.
    /// Each is multiplied by the off-shell scalar leg (the `Mul` the rooted FFS tree
    /// carries). `iosxxx` uses gc=[g,0] (left) for FFS1 and gc=[0,g] (right) for FFS3.
    #[test]
    fn test_eval_proj_amp_vs_iosxxx() {
        use crate::helas::vertex::iosxxx;
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

        let hels = [SpinorHelicity::Down, SpinorHelicity::Up];
        for (hel1, hel2) in iproduct!(hels, hels) {
            for charge in [Charge::Particle, Charge::Antiparticle] {
                let fo = OutDiracWf::from_momentum(p_fo, mass, hel1, charge);
                let fi = InDiracWf::from_momentum(p_fi, mass, hel2, charge);

                let left_ref = iosxxx(&fo, &fi, &s_wf, [g, Complex64::new(0.0, 0.0)]);
                let right_ref = iosxxx(&fo, &fi, &s_wf, [Complex64::new(0.0, 0.0), g]);

                // leg1 = fi (column / flow-in), leg2 = fo (row / flow-out), leg3 = scalar.
                let fi_slot = WaveformSlot::FermionIn(fi);
                let fo_slot = WaveformSlot::FermionOut(fo);
                let s_slot = WaveformSlot::Scalar(s_wf);

                // FFS1: left bilinear × s
                let WaveformSlot::Scalar(got1) = mul_apply(&[
                    scalar_bilinear_current(&[fi_slot, fo_slot], Chirality::Left),
                    s_slot,
                ]) else {
                    panic!("FFS1 did not produce a scalar");
                };
                let diff1 = (got1.value - left_ref).norm();
                assert!(
                    diff1 < 1e-10,
                    "left bilinear vs iosxxx diff={diff1} (hel {hel1},{hel2}, {charge:?})"
                );

                // FFS3: right bilinear × s
                let WaveformSlot::Scalar(got3) = mul_apply(&[
                    scalar_bilinear_current(&[fi_slot, fo_slot], Chirality::Right),
                    s_slot,
                ]) else {
                    panic!("FFS3 did not produce a scalar");
                };
                let diff3 = (got3.value - right_ref).norm();
                assert!(
                    diff3 < 1e-10,
                    "right bilinear vs iosxxx diff={diff3} (hel {hel1},{hel2}, {charge:?})"
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
    ///
    /// If the relative phases/signs between continuum diagrams are wrong (the
    /// diagnosed bug), this sum will NOT cancel.
    ///
    /// Largest U(1) Ward residual `|Σ_diagrams M(ε_γ→k_γ)| / max|M|`, maximised
    /// over all helicity configurations, for `proc` at momenta `p` with the photon
    /// on `ward_leg` replaced by its 4-momentum. Lepton masses are zeroed so the
    /// hand-built massless momenta are exactly on-shell (else the spinors fail the
    /// Dirac equation and Ward picks up an O(m²/s) artifact). Returns ~0 (machine
    /// precision) iff the coherent sum over diagrams gauge-cancels correctly.
    fn ward_max_ratio(proc: &str, p: &[LorentzVector<f64>], ward_leg: usize) -> f64 {
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};

        let model = sm_model();
        let evaluated = model.evaluate(
            &"Block MASS\n 11 0.0\n 13 0.0\n 15 0.0\n"
                .parse::<ParamCard>()
                .unwrap(),
        );
        let opts = ParsingOptions::default();
        let card = parse_proc_card(proc, &opts).unwrap();
        let sets = generate_from_proc_card(&card, model).unwrap();
        let eval = AmplitudeEvaluator::compile(&sets[0], model).unwrap();
        let bound = BoundAmplitude::<f64>::bind(&eval, &evaluated);

        let global_scale = eval
            .helicities()
            .iter()
            .map(|hel| bound.eval_amplitude(p, hel).norm())
            .fold(0.0_f64, f64::max)
            .max(1e-30);

        eval.helicities()
            .iter()
            .map(|hel| bound.eval_amplitude_ward(p, hel, ward_leg).norm() / global_scale)
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

    /// The unified `Ast<Sym>` round-trips through its s-expression `Display`/`FromStr`
    /// (the egglog boundary): re-rendering the parsed tree reproduces the original
    /// string exactly.
    #[test]
    fn test_sexpr_roundtrip_eemumu() {
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use crate::helas::eval::Sym;

        let model = sm_model();
        let opts = ParsingOptions::default();
        let card = parse_proc_card("generate e+ e- > mu+ mu-", &opts).unwrap();
        let sets = generate_from_proc_card(&card, model).unwrap();
        let diagrams = compile_diagram_ast(&sets[0], model).unwrap();

        let ast = lower::lower(&diagrams);
        let rendered = ast.to_string();
        let reparsed: Ast<Sym> = rendered.parse().expect("s-expr should re-parse");
        // `Display` expands the shared (DAG) currents into a tree, so the reparsed arena
        // has at least as many nodes; the rendered string is the stable invariant
        // (re-merging shared subterms is the future hash-consing/egglog pass).
        assert_eq!(
            rendered,
            reparsed.to_string(),
            "s-expr round-trip changed the tree"
        );
        assert!(reparsed.len() >= ast.len());
    }

    /// The whole-amplitude AST (one `Add` over all diagrams) reproduces the explicit
    /// coherent sum over per-diagram amplitudes, for every helicity of e+e-→μ+μ-.
    /// Guards the final diagram-sum `Add` and the symmetry/Fermi-sign folding.
    #[test]
    fn test_whole_amplitude_equals_diagram_sum_eemumu() {
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};

        let model = sm_model();
        let evaluated = model.evaluate(
            &"Block MASS\n 11 0.0\n 13 0.0\n"
                .parse::<ParamCard>()
                .unwrap(),
        );
        let opts = ParsingOptions::default();
        let card = parse_proc_card("generate e+ e- > mu+ mu-", &opts).unwrap();
        let sets = generate_from_proc_card(&card, model).unwrap();
        let eval = AmplitudeEvaluator::compile(&sets[0], model).unwrap();
        let bound = BoundAmplitude::<f64>::bind(&eval, &evaluated);
        let diagrams = compile_diagram_ast(&sets[0], model).unwrap();

        let st = 0.6_f64;
        let ct = (1.0 - st * st).sqrt();
        let p = [
            LorentzVector::new(50.0, 0.0, 0.0, 50.0),
            LorentzVector::new(50.0, 0.0, 0.0, -50.0),
            LorentzVector::new(50.0, 50.0 * st, 0.0, 50.0 * ct),
            LorentzVector::new(50.0, -50.0 * st, 0.0, -50.0 * ct),
        ];

        for hel in eval.helicities() {
            let whole = bound.eval_amplitude(&p, hel);
            let parts = diagrams
                .iter()
                .map(|d| eval_single_diagram(d, &p, hel, &evaluated))
                .fold(C::new(0.0, 0.0), |a, b| a + b);
            assert!(
                (whole - parts).norm() <= 1e-12 * (whole.norm() + 1e-30),
                "whole-amplitude AST disagrees with per-diagram sum for hel {hel:?}: \
                 whole={whole:.6e} parts={parts:.6e}"
            );
        }
    }

    /// Cross-check the Z current from the *outgoing* mu-pair against MadGraph's W11
    /// intermediate wavefunction for helicities 38 (correct in total |M|²) and 42
    /// (wrong, ratio ≈ 0.640).
    ///
    /// The test sets up FFV2·GC_50 ⊕ FFV4·GC_59 with OUTGOING mu+ (Antiparticle,
    /// incoming=false) and mu- (Particle, incoming=false) at the CSV point-0 momenta,
    /// then propagates through the Z. The expected result (MG W11 × i, from the
    /// MG_EVAL_WFUNCS probe) is hardcoded.
    ///
    /// Run: cargo test -p vibegraph-lib \
    ///   --lib helas::eval::run::tests::test_z_current_outgoing_mupair_vs_mg -- --nocapture
    #[test]
    fn test_z_current_outgoing_mupair_vs_mg() {
        use num_complex::Complex64;

        let model = sm_model();
        // Use MadGraph's massless-tau param card so couplings match the probe.
        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let card_path = std::path::Path::new(&manifest).join(
            "../validation/madgraph/output/ee_to_mumu_tata_qcd0/Cards/param_card_masslesstau.dat",
        );
        let card = std::fs::read_to_string(&card_path)
            .ok()
            .and_then(|s| s.parse::<ParamCard>().ok())
            .expect("param_card_masslesstau.dat not found — run `pixi run -e madgraph build-diagrams` first");
        let evaluated = model.evaluate(&card);

        let gc50 = model.coupling_id("GC_50").unwrap();
        let gc59 = model.coupling_id("GC_59").unwrap();
        let ffv2_id = model.lorentz_id("FFV2").unwrap();
        let ffv4_id = model.lorentz_id("FFV4").unwrap();
        let mu_id = model.particle_id("mu-").unwrap();
        let amu_id = model.particle_id("mu+").unwrap();
        let z_id = model.particle_id("Z").unwrap();

        // CSV point-0 momenta: mu+ (outgoing antiparticle) and mu- (outgoing particle).
        // MG Fortran leg ordering: P(0,3)=mu+, P(0,4)=mu- (leg indices 3,4 in Fortran = 2,3 in Python).
        let p_mup = LorentzVector::new(
            130.98844490914234,
            -106.66561232781022,
            -0.9379201403415187,
            -76.02328690775641,
        );
        let p_mum = LorentzVector::new(
            167.2530959714149,
            134.2336665209957,
            -62.607066356179416,
            -77.68703963098595,
        );

        // OUTGOING mu+ = Antiparticle; OUTGOING mu- = Particle.
        // leg_idx is the index into the momenta/helicities slice passed to eval, so 0/1.
        let leg_mup = ExtLegInfo {
            leg_idx: 0,
            id: amu_id,
            spin: 2,
            charge: Charge::Antiparticle,
            incoming: false,
        };
        let leg_mum = ExtLegInfo {
            leg_idx: 1,
            id: mu_id,
            spin: 2,
            charge: Charge::Particle,
            incoming: false,
        };

        // FFV2·GC_50 + FFV4·GC_59 = SM ℓ̄ℓZ vertex (same as test_eval_ffv2_4_3 but
        // rooted at the Z (vector output leg = leg index 2 in the vertex).
        let vertex_info = VertexInfo {
            terms: vec![
                VertexTerm::from_ufo(model, ffv2_id, "1", gc50, Some(2), None).unwrap(),
                VertexTerm::from_ufo(model, ffv4_id, "1", gc59, Some(2), None).unwrap(),
            ],
        };

        let mz = evaluated.mass(z_id);

        let current_diagram = DiagramEval::from_nodes(
            2,
            vec![
                EvalNode::External(leg_mup.clone()),
                EvalNode::External(leg_mum.clone()),
                EvalNode::OffShellCurrent {
                    info: vertex_info.clone(),
                    flow: None,
                    children: vec![EvalNodeId::new(0), EvalNodeId::new(1)],
                },
                EvalNode::Propagate {
                    info: PropInfo { id: z_id },
                    flow: None,
                    child: EvalNodeId::new(2),
                },
            ],
        );

        // MG W11 values (indices [2..6], i.e. the 4 eps components) from MG_EVAL_WFUNCS,
        // measured at CSV point 0 via the probe_wfuncs.py script.
        // Convention check (contrast with test_eval_ffv2_4_3 incoming case): VG = MG here.
        // Hel 38: (e+:+1, e-:-1, mu+:-1, mu-:+1, ta+:+1, ta-:-1)
        //   → mu+ nhel=-1, mu- nhel=+1 → Down/Up
        // Hel 42: (e+:+1, e-:-1, mu+:+1, mu-:-1, ta+:+1, ta-:-1)
        //   → mu+ nhel=+1, mu- nhel=-1 → Up/Down
        let mg_w11 = [
            // hel 38: [eps_t, eps_x, eps_y, eps_z] from MG probe
            [
                Complex64::new(4.20870284e-04, 3.03971367e-04),
                Complex64::new(9.71186197e-05, -1.51285696e-04),
                Complex64::new(1.55174433e-04, -7.61571054e-04),
                Complex64::new(-8.63339445e-04, -3.02084576e-04),
            ],
            // hel 42: [eps_t, eps_x, eps_y, eps_z]
            [
                Complex64::new(-5.22725529e-04, 3.84361483e-04),
                Complex64::new(-1.22982451e-04, -1.88027964e-04),
                Complex64::new(-2.02039917e-04, -9.50088237e-04),
                Complex64::new(1.07570329e-03, -3.86719509e-04),
            ],
        ];

        let test_cases = [
            // hel 38: mu+ Down, mu- Up
            (SpinorHelicity::Down, SpinorHelicity::Up, mg_w11[0], "hel38"),
            // hel 42: mu+ Up, mu- Down
            (SpinorHelicity::Up, SpinorHelicity::Down, mg_w11[1], "hel42"),
        ];

        for (hel_mup, hel_mum, mg_eps, label) in test_cases {
            let WaveformSlot::Vector(got) = eval_single_diagram_slot(
                &current_diagram,
                &[p_mup, p_mum],
                &[hel_mup.sign(), hel_mum.sign()],
                &evaluated,
            ) else {
                panic!("Z current ({label}) must evaluate to a vector");
            };

            eprintln!("\n{label}: mu+ hel={hel_mup}, mu- hel={hel_mum}");
            let mut convention_factor = Complex64::from(0.0);
            for mu in 0..4 {
                let vg = got.eps.component(mu);
                let mg = mg_eps[mu];
                let diff_direct = (vg - mg).norm();
                let diff_i = (vg - Complex64::i() * mg).norm();
                // Report which convention (VG=MG or VG=i×MG) fits better.
                if mg.norm() > 1e-30 {
                    let best = if diff_direct < diff_i {
                        "VG=MG"
                    } else {
                        "VG=i×MG"
                    };
                    eprintln!(
                        "  eps[{mu}]: VG={:.6e}+{:.6e}i  MG={:.6e}+{:.6e}i  \
                         diff_direct={:.2e}  diff_i={:.2e}  → {best}",
                        vg.re, vg.im, mg.re, mg.im, diff_direct, diff_i
                    );
                    convention_factor = convention_factor + vg / mg;
                }
            }
            eprintln!(
                "  mean VG/MG = {:.6}",
                convention_factor / Complex64::from(4.0)
            );

            // The outgoing mu-pair convention is VG = MG (no i factor).
            for mu in 0..4 {
                let vg = got.eps.component(mu);
                let expected = mg_eps[mu];
                let diff = (vg - expected).norm();
                assert!(
                    diff < 5e-10 * mz as f64,
                    "Z current from outgoing mu-pair ({label}, μ={mu}): \
                     VG={vg:.4e} vs MG={expected:.4e}, diff={diff:.2e}"
                );
            }
        }
    }

    /// Cross-check two *intermediate* wavefunctions on the e-spine / τ-line against
    /// MadGraph's `MG_EVAL_WFUNCS` dump (`probe_wfuncs.py`), at the CSV point-0 momenta:
    ///
    ///   W10 = Z current radiated by the e-spine, `FFV2_4_3(e-, e+)` — a vector;
    ///   W9  = off-shell τ⁻ after absorbing that Z, `FFV2_4_2(τ-, W10)` — a fermion.
    ///
    /// W10 follows the validated [`test_eval_ffv2_4_3`] structure but at the *physical*
    /// e⁺e⁻ momenta, and W9 is the first **fermion-absorption** step validated against
    /// MadGraph's real numbers (previously only vs the textbook / ALOHA primitives).
    ///
    /// Both depend only on the e/τ helicities, which are **identical** at hel 38
    /// (correct) and hel 42 (wrong: VG/MG ≈ 0.64) — only the μ-pair flips between them.
    /// So this test cannot itself exhibit the 0.64; a pass localises the residual to the
    /// μ-current absorption path (the off-shell electron after absorbing W11 / W8), which
    /// the MG probe does not yet dump.
    ///
    /// Run: cargo test -p vibegraph-lib \
    ///   helas::eval::run::tests::test_espine_z_and_tau_absorption_vs_mg -- --nocapture
    #[test]
    fn test_espine_z_and_tau_absorption_vs_mg() {
        use num_complex::Complex64;

        let model = sm_model();
        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let card_path = std::path::Path::new(&manifest).join(
            "../validation/madgraph/output/ee_to_mumu_tata_qcd0/Cards/param_card_masslesstau.dat",
        );
        let card = std::fs::read_to_string(&card_path)
            .ok()
            .and_then(|s| s.parse::<ParamCard>().ok())
            .expect("param_card_masslesstau.dat not found — run `pixi run -e madgraph build-diagrams` first");
        let evaluated = model.evaluate(&card);

        let gc50 = model.coupling_id("GC_50").unwrap();
        let gc59 = model.coupling_id("GC_59").unwrap();
        let ffv2_id = model.lorentz_id("FFV2").unwrap();
        let ffv4_id = model.lorentz_id("FFV4").unwrap();
        let ep_id = model.particle_id("e+").unwrap();
        let em_id = model.particle_id("e-").unwrap();
        let tam_id = model.particle_id("ta-").unwrap();
        let z_id = model.particle_id("Z").unwrap();

        // CSV point-0 momenta (E, px, py, pz). e⁺e⁻ head-on along z → the e-spine boson
        // is produced at rest, q = (500, 0, 0, 0).
        let p_ep = LorentzVector::new(250.0, 0.0, 0.0, 250.0);
        let p_em = LorentzVector::new(250.0, 0.0, 0.0, -250.0);
        let p_tam = LorentzVector::new(
            94.5533499515044,
            -18.39281604525598,
            -22.219961151047247,
            90.04617499607066,
        );

        // FFV2·GC_50 ⊕ FFV4·GC_59 = SM ℓ̄ℓZ vertex. Rooted at the vector leg (index 2)
        // it is the off-shell Z *current*; rooted at the fermion leg (index 1) it is the
        // off-shell *fermion* after absorbing the Z (MG's FFV2_4_2).
        let z_current_vertex = VertexInfo {
            terms: vec![
                VertexTerm::from_ufo(model, ffv2_id, "1", gc50, Some(2), None).unwrap(),
                VertexTerm::from_ufo(model, ffv4_id, "1", gc59, Some(2), None).unwrap(),
            ],
        };

        // F1 = e- (particle ket), F2 = e+ (antiparticle bra), both INCOMING; τ⁻ outgoing.
        let leg_em = ExtLegInfo {
            leg_idx: 0,
            id: em_id,
            spin: 2,
            charge: Charge::Particle,
            incoming: true,
        };
        let leg_ep = ExtLegInfo {
            leg_idx: 1,
            id: ep_id,
            spin: 2,
            charge: Charge::Antiparticle,
            incoming: true,
        };
        let leg_tam = ExtLegInfo {
            leg_idx: 2,
            id: tam_id,
            spin: 2,
            charge: Charge::Particle,
            incoming: false,
        };

        // Fermion-output rooting needs the continuing line's flow. τ⁻ (outgoing
        // particle) is a bra → Flow::Out, inherited by the off-shell τ⁻.
        let tau_flow = leg_tam.flow().unwrap();
        let z_absorb_vertex = VertexInfo {
            terms: vec![
                VertexTerm::from_ufo(model, ffv2_id, "1", gc50, Some(1), Some(tau_flow)).unwrap(),
                VertexTerm::from_ufo(model, ffv4_id, "1", gc59, Some(1), Some(tau_flow)).unwrap(),
            ],
        };

        // W10: e-, e+ → Z current → propagate(Z).
        let w10_diagram = DiagramEval::from_nodes(
            2,
            vec![
                EvalNode::External(leg_em.clone()),
                EvalNode::External(leg_ep.clone()),
                EvalNode::OffShellCurrent {
                    info: z_current_vertex.clone(),
                    flow: None,
                    children: vec![EvalNodeId::new(0), EvalNodeId::new(1)],
                },
                EvalNode::Propagate {
                    info: PropInfo { id: z_id },
                    flow: None,
                    child: EvalNodeId::new(2),
                },
            ],
        );

        // W9: extend W10 by τ⁻ absorbing it (FFV2_4_2 → off-shell τ⁻ → propagate(τ)).
        let w9_diagram = DiagramEval::from_nodes(
            3,
            vec![
                EvalNode::External(leg_em),
                EvalNode::External(leg_ep),
                EvalNode::OffShellCurrent {
                    info: z_current_vertex,
                    flow: None,
                    children: vec![EvalNodeId::new(0), EvalNodeId::new(1)],
                },
                EvalNode::Propagate {
                    info: PropInfo { id: z_id },
                    flow: None,
                    child: EvalNodeId::new(2),
                },
                EvalNode::External(leg_tam),
                EvalNode::OffShellCurrent {
                    info: z_absorb_vertex,
                    flow: Some(tau_flow),
                    children: vec![EvalNodeId::new(4), EvalNodeId::new(3)],
                },
                EvalNode::Propagate {
                    info: PropInfo { id: tam_id },
                    flow: Some(tau_flow),
                    child: EvalNodeId::new(5),
                },
            ],
        );

        // e+ nhel=+1 (Up), e- nhel=-1 (Down), τ- nhel=-1 (Down) — fixed across hel 38/42.
        let hel_em = SpinorHelicity::Down;
        let hel_ep = SpinorHelicity::Up;
        let hel_tam = SpinorHelicity::Down;

        // MG W10 (Z[e-spine]) and W9 (off-shell τ-): physical components are array
        // slots [2..6]; [0]=P0+iP3, [1]=P1+iP2 carry the momentum (used to label only).
        let mg_w10 = [
            Complex64::new(0.0, 0.0),
            Complex64::new(-4.2562490448e-04, 3.9206234096e-07),
            Complex64::new(-3.9206234096e-07, -4.2562490448e-04),
            Complex64::new(0.0, 0.0),
        ];
        let mg_w9 = [
            Complex64::new(7.5845893607e-06, -6.9865081405e-09),
            Complex64::new(-2.8186273499e-07, -3.3987438764e-07),
            Complex64::new(-9.0999412971e-09, 8.3823673144e-12),
            Complex64::new(-2.1193052488e-09, -2.5554906138e-09),
        ];

        // Detect the global convention phase (VG = factor · MG) from MG's largest
        // component, then assert every component agrees up to that single unit factor.
        fn check(label: &str, vg: [Complex64; 4], mg: [Complex64; 4]) {
            let kmax = (0..4)
                .max_by(|&a, &b| mg[a].norm().total_cmp(&mg[b].norm()))
                .unwrap();
            let factor = vg[kmax] / mg[kmax];
            eprintln!(
                "\n{label}: convention factor VG/MG = {:.6}+{:.6}i  |factor|={:.6}",
                factor.re,
                factor.im,
                factor.norm()
            );
            for k in 0..4 {
                let want = factor * mg[k];
                let diff = (vg[k] - want).norm();
                eprintln!(
                    "  [{k}] VG={:+.6e}{:+.6e}i  MG={:+.6e}{:+.6e}i  factor·MG={:+.6e}{:+.6e}i  diff={:.2e}",
                    vg[k].re, vg[k].im, mg[k].re, mg[k].im, want.re, want.im, diff
                );
            }
            assert!(
                (factor.norm() - 1.0).abs() < 1e-6,
                "{label}: VG/MG must be a pure phase, got |factor|={:.6}",
                factor.norm()
            );
            let scale = mg.iter().map(|c| c.norm()).fold(0.0, f64::max);
            for k in 0..4 {
                let diff = (vg[k] - factor * mg[k]).norm();
                assert!(
                    diff < 1e-9 * scale,
                    "{label} component [{k}]: VG={:.4e} vs factor·MG={:.4e}, diff={diff:.2e}",
                    vg[k],
                    factor * mg[k]
                );
            }
        }

        let WaveformSlot::Vector(w10) = eval_single_diagram_slot(
            &w10_diagram,
            &[p_em, p_ep],
            &[hel_em.sign(), hel_ep.sign()],
            &evaluated,
        ) else {
            panic!("W10 (e-spine Z current) must evaluate to a vector");
        };
        eprintln!("W10 momentum: VG={:?}  (MG q=(-500,0,0,0))", w10.momentum);
        check(
            "W10 = Z[e-spine] (FFV2_4_3)",
            core::array::from_fn(|k| w10.eps.component(k)),
            mg_w10,
        );

        // W9 is the off-shell τ⁻ *fermion* after absorbing W10. MadGraph builds the
        // outgoing τ⁻ as a flow-IN spinor (`IXXXXX`, nsf=−1), vibegraph as flow-OUT — an
        // equivalent fermion-line orientation that washes out of vector currents (hence
        // W10/W11 match MG) but flips the chiral half a raw off-shell fermion is stored
        // in. So MG's W9 raw array is NOT bit-comparable to VG's; we instead pin the
        // routing (momentum) and the orientation signature (which chiral half is filled),
        // and leave the absorption *physics* to `test_chiral_off_shell_fermion_vs_ffv2_2`.
        let (w9_comps, w9_mom): ([Complex64; 4], _) = match eval_single_diagram_slot(
            &w9_diagram,
            &[p_em, p_ep, p_tam],
            &[hel_em.sign(), hel_ep.sign(), hel_tam.sign()],
            &evaluated,
        ) {
            WaveformSlot::FermionOut(f) => {
                (core::array::from_fn(|k| f.spinor.component(k)), f.momentum)
            }
            WaveformSlot::FermionIn(f) => {
                (core::array::from_fn(|k| f.spinor.component(k)), f.momentum)
            }
            other => panic!("W9 (off-shell τ-) must be a fermion, got {other:?}"),
        };
        let mg_w9_mom = LorentzVector::new(
            -405.4466500484956,
            -18.39281604525598,
            -22.219961151047247,
            90.04617499607066,
        );
        eprintln!("\nW9 = off-shell τ- after Z absorption (FFV2_4_2)");
        eprintln!("  momentum: VG={w9_mom:?}");
        for k in 0..4 {
            eprintln!(
                "  [{k}] VG={:+.4e}{:+.4e}i   MG={:+.4e}{:+.4e}i",
                w9_comps[k].re, w9_comps[k].im, mg_w9[k].re, mg_w9[k].im
            );
        }
        let mom_diff = (w9_mom - mg_w9_mom).bare_norm_sq().sqrt();
        assert!(
            mom_diff < 1e-9 * 500.0,
            "W9 routed momentum must match MG: VG={w9_mom:?} vs MG={mg_w9_mom:?}, diff={mom_diff:.2e}"
        );
        // Massless τ ⇒ a single chiral half is populated. VG (flow-out) fills the
        // right half [2,3]; MG (flow-in) the left half [0,1] — the orientation signature.
        let vg_left = w9_comps[0].norm() + w9_comps[1].norm();
        let vg_right = w9_comps[2].norm() + w9_comps[3].norm();
        assert!(
            vg_right > 1e6 * vg_left,
            "VG flow-out W9 must populate the right-chiral half (orientation signature): \
             left={vg_left:.2e} right={vg_right:.2e}"
        );
    }

    /// Localise the per-Z 0.64 helicity reweighting to the **e-line Z absorption**.
    ///
    /// Controlled experiment on the e+-spine (MadGraph AMP(18) vs AMP(22), CSV point 0):
    /// the off-shell electron is built two ways that differ ONLY in the μ-side boson —
    ///   γ-path: e⁺ absorbs γ[μ] = `FFV1_1(e⁺, FFV1P0_3(μ-,μ+))`   (→ AMP(18), VG matches MG)
    ///   Z-path: e⁺ absorbs Z[μ] = `FFV2_4_1(e⁺, FFV2_4_3(μ-,μ+))` (→ AMP(22))
    /// Both μ-currents are already validated == MG bit-for-bit, and the **γ path matches
    /// MadGraph's off-shell electron exactly** (up to the global −i), pinning the
    /// rooting/flow/propagator machinery (γ couples L=R, so it is chirality-blind).
    ///
    /// Against MadGraph's actual off-shell electron (`probe_wfuncs.py`, slots 6=γ, 7=Z),
    /// the Z-path off-shell electron then:
    ///   • **matches MG exactly at hel 38** (the correct helicity), and
    ///   • is **0.6403 × MG at hel 42** (the flipped-μ helicity) — the exact per-Z 0.64.
    /// Every input to that vertex is correct, so the residual lives entirely in VG's
    /// chiral (FFV2/FFV4) e-line absorption at hel 42. This is a **characterisation test**:
    /// the hel-42 assertion pins the bug at 0.6403 and must be tightened to `== MG` once
    /// the absorption is fixed.
    #[test]
    fn test_espine_eline_z_absorption_ratio_vs_mg() {
        use num_complex::Complex64;

        let model = sm_model();
        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let card_path = std::path::Path::new(&manifest).join(
            "../validation/madgraph/output/ee_to_mumu_tata_qcd0/Cards/param_card_masslesstau.dat",
        );
        let card = std::fs::read_to_string(&card_path)
            .ok()
            .and_then(|s| s.parse::<ParamCard>().ok())
            .expect("param_card_masslesstau.dat not found — run `pixi run -e madgraph build-diagrams` first");
        let evaluated = model.evaluate(&card);

        let gc3 = model.coupling_id("GC_3").unwrap();
        let gc50 = model.coupling_id("GC_50").unwrap();
        let gc59 = model.coupling_id("GC_59").unwrap();
        let ffv1_id = model.lorentz_id("FFV1").unwrap();
        let ffv2_id = model.lorentz_id("FFV2").unwrap();
        let ffv4_id = model.lorentz_id("FFV4").unwrap();
        let ep_id = model.particle_id("e+").unwrap();
        let em_id = model.particle_id("e-").unwrap();
        let mu_id = model.particle_id("mu-").unwrap();
        let amu_id = model.particle_id("mu+").unwrap();
        let a_id = model.particle_id("a").unwrap();
        let z_id = model.particle_id("Z").unwrap();

        let p_ep = LorentzVector::new(250.0, 0.0, 0.0, 250.0);
        let p_mup = LorentzVector::new(
            130.98844490914234,
            -106.66561232781022,
            -0.9379201403415187,
            -76.02328690775641,
        );
        let p_mum = LorentzVector::new(
            167.2530959714149,
            134.2336665209957,
            -62.607066356179416,
            -77.68703963098595,
        );

        let leg_mum = ExtLegInfo {
            leg_idx: 0,
            id: mu_id,
            spin: 2,
            charge: Charge::Particle,
            incoming: false,
        };
        let leg_mup = ExtLegInfo {
            leg_idx: 1,
            id: amu_id,
            spin: 2,
            charge: Charge::Antiparticle,
            incoming: false,
        };
        let leg_ep = ExtLegInfo {
            leg_idx: 2,
            id: ep_id,
            spin: 2,
            charge: Charge::Antiparticle,
            incoming: true,
        };
        let ep_flow = leg_ep.flow().unwrap();

        // Build the off-shell-electron sub-diagram for a given boson (γ via FFV1/GC_3,
        // or Z via FFV2⊕FFV4/GC_50,GC_59): μ-pair → boson current → e⁺ absorbs it.
        let make_diagram = |current_vertex: VertexInfo, absorb_vertex: VertexInfo, boson| {
            DiagramEval::from_nodes(
                3,
                vec![
                    EvalNode::External(leg_mum.clone()),
                    EvalNode::External(leg_mup.clone()),
                    EvalNode::OffShellCurrent {
                        info: current_vertex,
                        flow: None,
                        children: vec![EvalNodeId::new(0), EvalNodeId::new(1)],
                    },
                    EvalNode::Propagate {
                        info: PropInfo { id: boson },
                        flow: None,
                        child: EvalNodeId::new(2),
                    },
                    EvalNode::External(leg_ep.clone()),
                    EvalNode::OffShellCurrent {
                        info: absorb_vertex,
                        flow: Some(ep_flow),
                        children: vec![EvalNodeId::new(4), EvalNodeId::new(3)],
                    },
                    EvalNode::Propagate {
                        info: PropInfo { id: em_id },
                        flow: Some(ep_flow),
                        child: EvalNodeId::new(5),
                    },
                ],
            )
        };

        let gamma_current = VertexInfo {
            terms: vec![VertexTerm::from_ufo(model, ffv1_id, "1", gc3, Some(2), None).unwrap()],
        };
        let gamma_absorb = VertexInfo {
            terms: vec![
                VertexTerm::from_ufo(model, ffv1_id, "1", gc3, Some(1), Some(ep_flow)).unwrap(),
            ],
        };
        let z_current = VertexInfo {
            terms: vec![
                VertexTerm::from_ufo(model, ffv2_id, "1", gc50, Some(2), None).unwrap(),
                VertexTerm::from_ufo(model, ffv4_id, "1", gc59, Some(2), None).unwrap(),
            ],
        };
        let z_absorb = VertexInfo {
            terms: vec![
                VertexTerm::from_ufo(model, ffv2_id, "1", gc50, Some(1), Some(ep_flow)).unwrap(),
                VertexTerm::from_ufo(model, ffv4_id, "1", gc59, Some(1), Some(ep_flow)).unwrap(),
            ],
        };

        let gamma_diagram = make_diagram(gamma_current, gamma_absorb, a_id);
        let z_diagram = make_diagram(z_current, z_absorb, z_id);

        let off_shell_e = |diagram: &DiagramEval, hmum, hmup| -> [Complex64; 4] {
            match eval_single_diagram_slot(
                diagram,
                &[p_mum, p_mup, p_ep],
                &[hmum, hmup, 1], // e+ helicity +1 (Up) for both hel 38 and 42
                &evaluated,
            ) {
                WaveformSlot::FermionOut(f) => core::array::from_fn(|k| f.spinor.component(k)),
                WaveformSlot::FermionIn(f) => core::array::from_fn(|k| f.spinor.component(k)),
                other => panic!("off-shell e must be a fermion, got {other:?}"),
            }
        };

        // r = eZ / eγ at the dominant component (cancels the orientation convention).
        let ratio = |ez: [Complex64; 4], eg: [Complex64; 4]| -> Complex64 {
            let k = (0..4)
                .max_by(|&a, &b| eg[a].norm().total_cmp(&eg[b].norm()))
                .unwrap();
            ez[k] / eg[k]
        };

        // MadGraph off-shell electron (probe_wfuncs.py slots 6=γ, 7=Z; physical [2..6]).
        let mg_ratio = |ez: [Complex64; 4], eg: [Complex64; 4]| ratio(ez, eg);
        let mg = [
            // (label, mu-, mu+ helicity codes, eγ[2..6], eZ[2..6])
            (
                "hel38",
                1_i32,
                -1_i32,
                [
                    Complex64::ZERO,
                    Complex64::ZERO,
                    Complex64::new(-2.108086e-05, -4.566556e-06),
                    Complex64::new(3.215450e-05, 1.827798e-05),
                ],
                [
                    Complex64::ZERO,
                    Complex64::ZERO,
                    Complex64::new(8.735431e-06, 1.853315e-06),
                    Complex64::new(-1.334406e-05, -7.510224e-06),
                ],
            ),
            (
                "hel42",
                -1_i32,
                1_i32,
                [
                    Complex64::ZERO,
                    Complex64::ZERO,
                    Complex64::new(1.989177e-05, -1.598997e-05),
                    Complex64::new(2.804911e-05, -1.685689e-05),
                ],
                [
                    Complex64::ZERO,
                    Complex64::ZERO,
                    Complex64::new(1.025656e-05, -8.316966e-06),
                    Complex64::new(1.447519e-05, -8.783500e-06),
                ],
            ),
        ];

        let i = Complex64::i();
        for (label, hmum, hmup, mg_eg, mg_ez) in mg {
            let vg_eg = off_shell_e(&gamma_diagram, hmum, hmup);
            let vg_ez = off_shell_e(&z_diagram, hmum, hmup);
            let r_vg = ratio(vg_ez, vg_eg);
            let r_mg = mg_ratio(mg_ez, mg_eg);
            eprintln!(
                "\n{label}: e-line off-shell electron, Z/γ ratio  VG={:+.5}{:+.5}i  MG={:+.5}{:+.5}i",
                r_vg.re, r_vg.im, r_mg.re, r_mg.im
            );

            // Photon absorption is chirality-blind (γ couples L=R), so it pins the
            // rooting/flow/propagator/momentum machinery: VG's γ-path off-shell electron
            // must equal MadGraph's up to the global −i UFO-coupling phase. (The Z-path
            // carries the chiral physics and is the localiser — printed above.)
            let kmax = (0..4)
                .max_by(|&a, &b| mg_eg[a].norm().total_cmp(&mg_eg[b].norm()))
                .unwrap();
            let scale = mg_eg[kmax].norm();
            for k in 0..4 {
                let diff = (vg_eg[k] - (-i) * mg_eg[k]).norm();
                assert!(
                    diff < 1e-6 * scale,
                    "{label} γ-path off-shell e [{k}]: VG={:.4e} vs −i·MG={:.4e}, diff={diff:.2e}",
                    vg_eg[k],
                    -i * mg_eg[k]
                );
            }

            // Z path: identical machinery, only the chiral (FFV2/FFV4) vertex differs.
            // Factor of VG's Z off-shell electron vs the correct value (−i·MG): must be
            // 1 at hel 38 (correct) and is exactly 0.6403 at hel 42 (the per-Z bug).
            let kz = (0..4)
                .max_by(|&a, &b| mg_ez[a].norm().total_cmp(&mg_ez[b].norm()))
                .unwrap();
            let zfac = vg_ez[kz] / ((-i) * mg_ez[kz]);
            let expected = if label == "hel38" { 1.0 } else { 0.6403 };
            eprintln!(
                "  Z-path VG/(−i·MG) = {:+.4}{:+.4}i   (expected ≈ {expected})",
                zfac.re, zfac.im
            );
            assert!(
                (zfac.re - expected).abs() < 2e-3 && zfac.im.abs() < 2e-3,
                "{label} Z-path off-shell e: VG/(−i·MG)={zfac:.4}, expected ≈ {expected}"
            );
        }
    }
}
