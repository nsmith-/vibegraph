//! Unit tests for the symbolic color algebra.
//!
//! Group-theoretic closures are checked against the exact Casimir/Dynkin
//! constants carried by the `repr::color` representation types, so the algebra
//! is validated against an independent oracle rather than itself.

use num_rational::Ratio;

use super::coeff::ColorCoeff;
use super::factor::{ColorFactor, ColorString};
use super::tensor::{ColorTensor, TensorKind};
use crate::helas::repr::color::{ColorRepr, SU3Adjoint, SU3Fundamental};

/// Convenience: a `T` with the given adjoint chain and fundamental indices.
fn t(adj: &[i32], i: i32, j: i32) -> ColorTensor {
    ColorTensor::T(adj.to_vec(), i, j)
}

/// Convenience: a Kronecker delta `δ_{ij} = T(i, j)`.
fn delta(i: i32, j: i32) -> ColorTensor {
    ColorTensor::T(Vec::new(), i, j)
}

/// Split a fully-reduced color factor (no tensors left) into its real and
/// imaginary rational parts at `Nc = 3`.
fn eval_scalar(cf: &ColorFactor) -> (Ratio<i64>, Ratio<i64>) {
    let mut re = Ratio::from_integer(0);
    let mut im = Ratio::from_integer(0);
    for s in &cf.0 {
        assert!(
            s.tensors.is_empty(),
            "eval_scalar on a factor with leftover tensors: {:?}",
            s.tensors
        );
        if s.coeff.imag {
            im += s.coeff.eval_nc(3);
        } else {
            re += s.coeff.eval_nc(3);
        }
    }
    (re, im)
}

// ── Casimir / Dynkin closures against repr::color oracles ─────────────────

/// `T(a,i,j)·T(a,j,k) = C_F·δ_{ik}` — the fundamental Casimir. The surviving
/// structure is a single delta on the free indices, and its coefficient sums
/// to `C_F = 4/3`.
#[test]
fn casimir_fundamental_closure() {
    // a = -1 (summed adjoint), j = -2 (summed fundamental), i = 1, k = 2.
    let s = ColorString::new(vec![t(&[-1], 1, -2), t(&[-1], -2, 2)]);
    let result = ColorFactor(vec![s]).full_simplify();

    // Every surviving term must be the delta δ_{1,2}.
    for term in &result.0 {
        assert_eq!(term.tensors, vec![delta(1, 2)]);
    }
    // Coefficients sum to the fundamental Casimir.
    let sum: Ratio<i64> = result.0.iter().map(|s| s.coeff.eval_nc(3)).sum();
    assert_eq!(sum, <SU3Fundamental as ColorRepr<f64>>::casimir());
    assert_eq!(sum, Ratio::new(4, 3));
}

/// `T(a,i,j)·T(a,j,i)` fully summed traces `T^a T^a` over the fundamental line:
/// `C_F·Nc = T(F)·(Nc²−1) = 4` at `Nc = 3`.
#[test]
fn casimir_trace_closure() {
    let s = ColorString::new(vec![t(&[-1], -2, -3), t(&[-1], -3, -2)]);
    let result = ColorFactor(vec![s]).full_simplify();
    let (re, im) = eval_scalar(&result);
    assert_eq!(im, Ratio::from_integer(0));

    let cf = <SU3Fundamental as ColorRepr<f64>>::casimir();
    assert_eq!(re, cf * Ratio::from_integer(3)); // C_F · Nc
    let tf = <SU3Fundamental as ColorRepr<f64>>::dynkin();
    assert_eq!(re, tf * Ratio::from_integer(8)); // T(F) · (Nc²−1)
    assert_eq!(re, Ratio::from_integer(4));
}

// ── f-contraction identities ──────────────────────────────────────────────

/// `f(a,x,y)·f(b,x,y) = Nc·δ^{ab} = 2·Nc·Tr(a,b)` — a single real term.
#[test]
fn ff_contraction_two_nc_trace() {
    // a = 1, b = 2 external adjoint; x = -1, y = -2 summed.
    let s = ColorString::new(vec![ColorTensor::F(1, -1, -2), ColorTensor::F(2, -1, -2)]);
    let result = ColorFactor(vec![s]).full_simplify();

    assert_eq!(result.0.len(), 1, "expected a single term: {result:?}");
    let term = &result.0[0];
    assert_eq!(term.tensors, vec![ColorTensor::Tr(vec![1, 2])]);
    assert!(!term.coeff.imag);
    assert_eq!(term.coeff.q, Ratio::from_integer(2));
    assert_eq!(term.coeff.nc_power, 1);
}

/// `f(a,x,y)·f(a,x,y)` fully contracted `= Nc·(Nc²−1) = 24` at `Nc = 3`, i.e.
/// `C_A·(Nc²−1)`.
#[test]
fn ff_fully_contracted_scalar() {
    let s = ColorString::new(vec![ColorTensor::F(-3, -1, -2), ColorTensor::F(-3, -1, -2)]);
    let result = ColorFactor(vec![s]).full_simplify();
    let (re, im) = eval_scalar(&result);
    assert_eq!(im, Ratio::from_integer(0));
    assert_eq!(re, Ratio::from_integer(24));
    let ca = <SU3Adjoint as ColorRepr<f64>>::casimir();
    assert_eq!(re, ca * Ratio::from_integer(8)); // C_A · (Nc²−1)
}

// ── Trace values ──────────────────────────────────────────────────────────

/// `Tr() = Nc`.
#[test]
fn empty_trace_is_nc() {
    let result = ColorFactor(vec![ColorString::new(vec![ColorTensor::Tr(vec![])])]).full_simplify();
    assert_eq!(result.0.len(), 1);
    let term = &result.0[0];
    assert!(term.tensors.is_empty());
    assert_eq!(term.coeff.nc_power, 1);
    assert_eq!(term.coeff.q, Ratio::from_integer(1));
    assert_eq!(term.coeff.eval_nc(3), Ratio::from_integer(3));
}

/// `Tr(a) = 0` — the single-index trace vanishes and drops out.
#[test]
fn single_trace_is_zero() {
    let result =
        ColorFactor(vec![ColorString::new(vec![ColorTensor::Tr(vec![1])])]).full_simplify();
    assert!(result.0.is_empty(), "Tr(a) should vanish: {result:?}");
}

// ── The SU(N) Fierz completeness identity, concretely ─────────────────────

/// `T^a_{ij} T^a_{kl} = ½(δ_{il}δ_{kj} − Nc⁻¹ δ_{ij}δ_{kl})`.
#[test]
fn fierz_completeness_identity() {
    // shared adjoint x = -1; i=1, j=2, k=3, l=4 external.
    let s = ColorString::new(vec![t(&[-1], 1, 2), t(&[-1], 3, 4)]);
    let result = ColorFactor(vec![s]).full_simplify();

    // Two terms: +½ δ_{14}δ_{32} and −½/Nc δ_{12}δ_{34}.
    assert_eq!(result.0.len(), 2, "{result:?}");

    let find = |tensors: Vec<ColorTensor>| {
        result
            .0
            .iter()
            .find(|s| {
                let mut got = s.tensors.clone();
                let mut want = tensors.clone();
                got.sort_by_key(|a| a.indices());
                want.sort_by_key(|a| a.indices());
                got == want
            })
            .unwrap_or_else(|| panic!("term not found in {result:?}"))
    };

    let direct = find(vec![delta(1, 4), delta(3, 2)]);
    assert_eq!(direct.coeff.q, Ratio::new(1, 2));
    assert_eq!(direct.coeff.nc_power, 0);
    assert!(!direct.coeff.imag);

    let trace = find(vec![delta(1, 2), delta(3, 4)]);
    assert_eq!(trace.coeff.q, Ratio::new(-1, 2));
    assert_eq!(trace.coeff.nc_power, -1);
    assert!(!trace.coeff.imag);
}

// ── Canonicalization ──────────────────────────────────────────────────────

/// Canonicalization renames *all* indices by position, so two strings with the
/// same index-sharing pattern share a canonical form even when their concrete
/// labels (summed or external) differ; a string with a different sharing
/// pattern does not. (This is why C3 keys the basis off `to_immutable`, which
/// keeps concrete indices, not `to_canonical`.)
#[test]
fn canonicalization_relabels_indices() {
    // Same pattern (shared −2 links j of the first T to i of the second),
    // different labels throughout.
    let a = ColorString::new(vec![t(&[-1], 5, -2), t(&[-1], -2, 7)]);
    let b = ColorString::new(vec![t(&[-9], 6, -8), t(&[-9], -8, 4)]);
    assert_eq!(a.canonical(), b.canonical());

    // Different pattern: the shared −2 is now the j index of *both* T's.
    let c = ColorString::new(vec![t(&[-1], 5, -2), t(&[-1], 7, -2)]);
    assert_ne!(a.canonical(), c.canonical());
}

/// Canonicalization is idempotent: re-canonicalizing a string built from a
/// canonical form yields the same canonical form.
#[test]
fn canonicalization_is_idempotent() {
    let s = ColorString::new(vec![ColorTensor::Tr(vec![-7, 3, -7]), t(&[-2], 1, -2)]);
    let (canon1, _) = s.to_canonical();
    // Rebuild a string directly from the canonical (kind, indices) form.
    let rebuilt = ColorString::new(
        canon1
            .0
            .iter()
            .map(|(kind, idx)| match kind {
                TensorKind::T => ColorTensor::T(
                    idx[..idx.len() - 2].to_vec(),
                    idx[idx.len() - 2],
                    idx[idx.len() - 1],
                ),
                TensorKind::Tr => ColorTensor::Tr(idx.clone()),
                TensorKind::F => ColorTensor::F(idx[0], idx[1], idx[2]),
                TensorKind::D => ColorTensor::D(idx[0], idx[1], idx[2]),
                TensorKind::One => ColorTensor::One,
            })
            .collect(),
    );
    assert_eq!(rebuilt.canonical(), canon1);
}

// ── Conjugation involution ────────────────────────────────────────────────

/// Complex conjugation is an involution on coefficients, tensors, and strings.
#[test]
fn conjugation_is_involution() {
    let coeff = ColorCoeff {
        q: Ratio::new(-3, 7),
        imag: true,
        nc_power: 2,
    };
    assert_eq!(coeff.conj().conj(), coeff);

    let tensors = vec![
        t(&[1, -2, 3], 4, -5),
        ColorTensor::Tr(vec![-2, 6, 7]),
        ColorTensor::F(1, 2, 3),
        ColorTensor::D(4, 5, 6),
        ColorTensor::One,
    ];
    for tensor in &tensors {
        assert_eq!(tensor.conj().conj(), *tensor);
    }

    let s = ColorString {
        coeff,
        tensors: tensors.clone(),
    };
    let ss = s.conj().conj();
    assert_eq!(ss.coeff, s.coeff);
    assert_eq!(ss.tensors, s.tensors);
}

/// `T(a,b,c,i,j)* = T(c,b,a,j,i)`.
#[test]
fn t_conjugate_reverses_and_swaps() {
    assert_eq!(t(&[1, 2, 3], 4, 5).conj(), t(&[3, 2, 1], 5, 4));
}

// ── Mixed-atom fixpoint & idempotence ─────────────────────────────────────

/// `full_simplify` of a mix of `f`, `T`, and the induced traces converges to an
/// irreducible factor, and applying it again is a no-op.
#[test]
fn full_simplify_mixed_atoms_is_a_fixpoint() {
    // f(x,1,2)·T(x,3,4): an f feeding a quark line through a shared gluon x.
    let s = ColorString::new(vec![ColorTensor::F(-1, 1, 2), t(&[-1], 3, 4)]);
    let once = ColorFactor(vec![s]).full_simplify();
    let twice = once.full_simplify();

    // Idempotent.
    assert_eq!(once.0.len(), twice.0.len());
    for (a, b) in once.0.iter().zip(&twice.0) {
        assert!(a.equiv(b), "not a fixpoint:\n{once:?}\nvs\n{twice:?}");
    }
    // Fully reduced: no f/d, and no T that closes to a trace.
    for term in &once.0 {
        for tensor in &term.tensors {
            assert!(
                !matches!(tensor, ColorTensor::F(..) | ColorTensor::D(..)),
                "leftover f/d in {term:?}"
            );
            if let ColorTensor::T(_, i, j) = tensor {
                assert_ne!(i, j, "leftover closeable T in {term:?}");
            }
        }
    }
}

// ── Overflow tripwire ─────────────────────────────────────────────────────

/// Coefficient multiplication panics rather than wrapping on `i64` overflow.
#[test]
#[should_panic(expected = "overflow")]
fn coeff_multiply_overflow_panics() {
    let big = ColorCoeff {
        q: Ratio::from_integer(i64::MAX),
        imag: false,
        nc_power: 0,
    };
    let _ = big.mul(&big);
}
