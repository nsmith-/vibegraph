//! Stage 0 property-test harness for the typed-repr-conventions work.
//!
//! A reusable toolbox — **not** an equivalence test in itself — shared by two later
//! stages that certify *different* equivalences over the same machinery:
//! - Stage A: `fused == generic` (a peephole kernel reproduces the generic composition);
//! - Stage B: `new_composite == old_composite` (a convention refactor is amplitude-preserving
//!   at the contracted-observable seam).
//!
//! Both consume the same two pieces provided here:
//! 1. **Typed random-input generators** — ket/bra spinors, `ε` at each [`Variance`], momenta,
//!    scalars, reals — each already wrapped in the [`WaveformSlot`] currency the eval kernels
//!    take. Component generators ([`rand_c`], [`rand_momentum`]) are exposed too, so a stage
//!    can build a bespoke typed input the wrappers don't cover.
//! 2. **A comparison + driver core** ([`slots_approx_eq`], [`check_agree`]) that evaluates two
//!    kernels/subtrees on the same random inputs and asserts their [`WaveformSlot`] outputs agree.
//!
//! The identities the stages check are *algebraic*, so the generators deliberately produce
//! arbitrary (off-shell, EOM-violating) inputs: an identity that holds only on-shell would be
//! the wrong thing to certify. Because both kernels in a comparison receive the *same* input
//! vector, momentum routing stays consistent without any physical constraint.
//!
//! The comparator is strict on the slot *variant* (a `Vector` never compares equal to a
//! `VectorCo`). Stage B, which deliberately changes a primitive's output variance, must
//! therefore compose down to a scalar (the contracted observable) before comparing — exactly
//! the discipline its oracle requires.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use super::waveform_slot::WaveformSlot;
use crate::helas::repr::lorentz::{
    Bispinor, ComplexVector, Contravariant, Covariant, LorentzVector, VectorRepr,
};
use crate::helas::repr::C;
use crate::helas::wavefn::{InDiracWf, OutDiracWf, ScalarWf, VectorWf};

/// Concrete scalar field for the harness. The identities under test are algebraic and
/// hold for any field, so we fix `f64` rather than staying generic over [`Real`](crate::helas::repr::Real).
pub(crate) type F = f64;

/// A seeded RNG so any failure reproduces from the reported seed.
pub(crate) fn seeded_rng(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}

/// A random real in `(-2, 2)` — bounded away from zero-dominated and overflow regimes so
/// tolerances stay meaningful, but otherwise unconstrained.
pub(crate) fn rand_re(rng: &mut StdRng) -> F {
    rng.random::<F>() * 4.0 - 2.0
}

/// A random complex number with both parts drawn by [`rand_re`].
pub(crate) fn rand_c(rng: &mut StdRng) -> C<F> {
    C::new(rand_re(rng), rand_re(rng))
}

/// A random (real) contravariant 4-momentum `p^μ`. Not constrained on-shell.
pub(crate) fn rand_momentum(rng: &mut StdRng) -> LorentzVector<F, Contravariant> {
    LorentzVector::new(rand_re(rng), rand_re(rng), rand_re(rng), rand_re(rng))
}

/// A random four-component complex polarisation at variance `V` (caller annotates `V`).
fn rand_eps<V: crate::helas::repr::lorentz::Variance>(rng: &mut StdRng) -> ComplexVector<F, V> {
    ComplexVector::new([rand_c(rng), rand_c(rng), rand_c(rng), rand_c(rng)])
}

/// A random Weyl-basis bispinor at adjoint `Adj` (caller annotates `Adj`).
fn rand_bispinor<Adj: crate::helas::repr::lorentz::DiracAdjoint>(
    rng: &mut StdRng,
) -> Bispinor<F, Adj> {
    Bispinor::from_components([rand_c(rng), rand_c(rng), rand_c(rng), rand_c(rng)])
}

/// A random ket (flow-in / column `u`,`v`) fermion current slot.
pub(crate) fn rand_ket(rng: &mut StdRng) -> WaveformSlot<F> {
    WaveformSlot::FermionIn(InDiracWf::from_spinor(
        rand_bispinor(rng),
        rand_momentum(rng),
    ))
}

/// A random bra (flow-out / row `ū`,`v̄`) fermion current slot.
pub(crate) fn rand_bra(rng: &mut StdRng) -> WaveformSlot<F> {
    WaveformSlot::FermionOut(OutDiracWf::from_spinor(
        rand_bispinor(rng),
        rand_momentum(rng),
    ))
}

/// A random contravariant (`ε^μ`) vector current slot.
pub(crate) fn rand_vector(rng: &mut StdRng) -> WaveformSlot<F> {
    WaveformSlot::Vector(VectorWf {
        eps: rand_eps::<Contravariant>(rng),
        momentum: rand_momentum(rng),
    })
}

/// A random covariant (`ε_μ`) vector current slot — the `MetricVout`/`LowerVout` storage form.
pub(crate) fn rand_vector_co(rng: &mut StdRng) -> WaveformSlot<F> {
    WaveformSlot::VectorCo(VectorWf::<F, Covariant> {
        eps: rand_eps::<Covariant>(rng),
        momentum: rand_momentum(rng),
    })
}

/// A random scalar current slot (arbitrary complex amplitude + momentum).
pub(crate) fn rand_scalar(rng: &mut StdRng) -> WaveformSlot<F> {
    WaveformSlot::Scalar(ScalarWf {
        value: rand_c(rng),
        momentum: rand_momentum(rng),
    })
}

/// A random bare real-constant slot (mass/width/coefficient register).
pub(crate) fn rand_real(rng: &mut StdRng) -> WaveformSlot<F> {
    WaveformSlot::Real(rand_re(rng))
}

// ─────────────────────────── comparison + driver core ───────────────────────────

fn approx(a: F, b: F, tol: F) -> bool {
    (a - b).abs() <= tol
}

fn approx_c(a: C<F>, b: C<F>, tol: F) -> bool {
    (a - b).norm() <= tol
}

fn approx_mom(
    p: &LorentzVector<F, Contravariant>,
    q: &LorentzVector<F, Contravariant>,
    tol: F,
) -> bool {
    approx(p.e(), q.e(), tol)
        && approx(p.px(), q.px(), tol)
        && approx(p.py(), q.py(), tol)
        && approx(p.pz(), q.pz(), tol)
}

/// The name of a slot's variant, for mismatch diagnostics.
fn variant(slot: &WaveformSlot<F>) -> &'static str {
    match slot {
        WaveformSlot::FermionIn(_) => "FermionIn",
        WaveformSlot::FermionOut(_) => "FermionOut",
        WaveformSlot::Vector(_) => "Vector",
        WaveformSlot::VectorCo(_) => "VectorCo",
        WaveformSlot::Scalar(_) => "Scalar",
        WaveformSlot::Real(_) => "Real",
        WaveformSlot::Empty => "Empty",
    }
}

/// Compare two [`WaveformSlot`]s for approximate equality within `tol`.
///
/// Strict on the variant: differing variants (including `Vector` vs `VectorCo`) are always a
/// mismatch. Within a variant, every stored component *and* the routed momentum are compared —
/// a kernel that gets the algebra right but mis-routes momentum is a real defect. Returns a
/// human-readable description of the first mismatch, so the [`check_agree`] driver can report it.
pub(crate) fn slots_approx_eq(
    a: &WaveformSlot<F>,
    b: &WaveformSlot<F>,
    tol: F,
) -> Result<(), String> {
    use WaveformSlot::*;
    match (a, b) {
        (Real(x), Real(y)) => {
            if approx(*x, *y, tol) {
                Ok(())
            } else {
                Err(format!("Real mismatch: {x} vs {y} (tol {tol})"))
            }
        }
        (Scalar(x), Scalar(y)) => {
            if !approx_c(x.value, y.value, tol) {
                return Err(format!(
                    "Scalar value mismatch: {:?} vs {:?}",
                    x.value, y.value
                ));
            }
            if !approx_mom(&x.momentum, &y.momentum, tol) {
                return Err(format!(
                    "Scalar momentum mismatch: {:?} vs {:?}",
                    x.momentum, y.momentum
                ));
            }
            Ok(())
        }
        (Vector(x), Vector(y)) => cmp_eps(
            (0..4).map(|i| (x.eps.component(i), y.eps.component(i))),
            &x.momentum,
            &y.momentum,
            tol,
        ),
        (VectorCo(x), VectorCo(y)) => cmp_eps(
            (0..4).map(|i| (x.eps.component(i), y.eps.component(i))),
            &x.momentum,
            &y.momentum,
            tol,
        ),
        (FermionIn(x), FermionIn(y)) => cmp_spinor(
            (0..4).map(|i| (x.spinor.component(i), y.spinor.component(i))),
            &x.momentum,
            &y.momentum,
            tol,
        ),
        (FermionOut(x), FermionOut(y)) => cmp_spinor(
            (0..4).map(|i| (x.spinor.component(i), y.spinor.component(i))),
            &x.momentum,
            &y.momentum,
            tol,
        ),
        (Empty, Empty) => Ok(()),
        _ => Err(format!(
            "variant mismatch: {} vs {}",
            variant(a),
            variant(b)
        )),
    }
}

/// Component-wise comparison shared by the vector variants.
fn cmp_eps(
    comps: impl Iterator<Item = (C<F>, C<F>)>,
    pa: &LorentzVector<F, Contravariant>,
    pb: &LorentzVector<F, Contravariant>,
    tol: F,
) -> Result<(), String> {
    for (mu, (x, y)) in comps.enumerate() {
        if !approx_c(x, y, tol) {
            return Err(format!("vector component {mu} mismatch: {x:?} vs {y:?}"));
        }
    }
    if !approx_mom(pa, pb, tol) {
        return Err(format!("vector momentum mismatch: {pa:?} vs {pb:?}"));
    }
    Ok(())
}

/// Component-wise comparison shared by the fermion variants.
fn cmp_spinor(
    comps: impl Iterator<Item = (C<F>, C<F>)>,
    pa: &LorentzVector<F, Contravariant>,
    pb: &LorentzVector<F, Contravariant>,
    tol: F,
) -> Result<(), String> {
    for (k, (x, y)) in comps.enumerate() {
        if !approx_c(x, y, tol) {
            return Err(format!("spinor component {k} mismatch: {x:?} vs {y:?}"));
        }
    }
    if !approx_mom(pa, pb, tol) {
        return Err(format!("spinor momentum mismatch: {pa:?} vs {pb:?}"));
    }
    Ok(())
}

/// The harness core: draw `n` random input vectors from `gen` and assert that `lhs` and `rhs`
/// produce approximately-equal [`WaveformSlot`]s on each, within `tol`.
///
/// `lhs`/`rhs` are the two kernels/subtrees under comparison; both receive the *same* input
/// slice per sample. On the first disagreement this panics with the sample index, seed, the
/// inputs, and both outputs — everything needed to reproduce and localise the failure.
pub(crate) fn check_agree<G, L, R>(n: usize, seed: u64, tol: F, mut gen: G, lhs: L, rhs: R)
where
    G: FnMut(&mut StdRng) -> Vec<WaveformSlot<F>>,
    L: Fn(&[WaveformSlot<F>]) -> WaveformSlot<F>,
    R: Fn(&[WaveformSlot<F>]) -> WaveformSlot<F>,
{
    let mut rng = seeded_rng(seed);
    for k in 0..n {
        let inputs = gen(&mut rng);
        let a = lhs(&inputs);
        let b = rhs(&inputs);
        if let Err(msg) = slots_approx_eq(&a, &b, tol) {
            panic!(
                "check_agree mismatch at sample {k} (seed {seed}): {msg}\n  inputs: {inputs:?}\n  lhs:    {a:?}\n  rhs:    {b:?}"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: F = 1e-12;
    const N: usize = 200;

    /// Every generated slot compares equal to itself (comparator reflexivity across variants).
    #[test]
    fn comparator_is_reflexive_on_all_variants() {
        let mut rng = seeded_rng(1);
        let gens: [fn(&mut StdRng) -> WaveformSlot<F>; 7] = [
            rand_ket,
            rand_bra,
            rand_vector,
            rand_vector_co,
            rand_scalar,
            rand_real,
            |_| WaveformSlot::Empty,
        ];
        for _ in 0..N {
            for g in gens {
                let s = g(&mut rng);
                slots_approx_eq(&s, &s, TOL).expect("a slot must equal itself");
            }
        }
    }

    /// Differing variants never compare equal (the `Vector`/`VectorCo` distinction Stage B
    /// leans on).
    #[test]
    fn comparator_rejects_variant_mismatch() {
        let mut rng = seeded_rng(2);
        let v = rand_vector(&mut rng);
        let vco = rand_vector_co(&mut rng);
        assert!(slots_approx_eq(&v, &vco, TOL).is_err());
        assert!(slots_approx_eq(&v, &rand_scalar(&mut rng), TOL).is_err());
    }

    /// A perturbation just above `tol` is detected in each stored channel (value and momentum).
    #[test]
    fn comparator_detects_perturbations() {
        let mut rng = seeded_rng(3);
        // Scalar value.
        let s = ScalarWf {
            value: rand_c(&mut rng),
            momentum: rand_momentum(&mut rng),
        };
        let s_bumped = ScalarWf {
            value: s.value + C::new(10.0 * TOL, 0.0),
            momentum: s.momentum,
        };
        assert!(slots_approx_eq(
            &WaveformSlot::Scalar(s),
            &WaveformSlot::Scalar(s_bumped),
            TOL
        )
        .is_err());

        // Momentum routing.
        let s_moved = ScalarWf {
            value: s.value,
            momentum: s.momentum + LorentzVector::new(10.0 * TOL, 0.0, 0.0, 0.0),
        };
        assert!(slots_approx_eq(
            &WaveformSlot::Scalar(s),
            &WaveformSlot::Scalar(s_moved),
            TOL
        )
        .is_err());
    }

    /// The driver passes when both sides are the identical kernel.
    #[test]
    fn driver_passes_on_identical_kernels() {
        check_agree(
            N,
            10,
            TOL,
            |rng| vec![rand_vector(rng), rand_scalar(rng)],
            |c| c[0],
            |c| c[0],
        );
    }

    /// The driver *fails* when the two sides genuinely disagree — proving it is not vacuously
    /// green.
    #[test]
    #[should_panic(expected = "check_agree mismatch")]
    fn driver_catches_disagreement() {
        check_agree(
            N,
            11,
            TOL,
            |rng| vec![rand_scalar(rng)],
            |c| c[0],
            |c| match c[0] {
                WaveformSlot::Scalar(s) => WaveformSlot::Scalar(ScalarWf {
                    value: -s.value,
                    momentum: s.momentum,
                }),
                other => other,
            },
        );
    }
}
