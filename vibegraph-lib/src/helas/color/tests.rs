//! Unit tests for the symbolic color algebra.
//!
//! Group-theoretic closures are checked against the exact Casimir/Dynkin
//! constants carried by the `repr::color` representation types, so the algebra
//! is validated against an independent oracle rather than itself.

use num_rational::Ratio;

use super::coeff::ColorCoeff;
use super::factor::{ColorFactor, ColorString};
use super::tensor::ColorTensor;
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
            .map(|(kind, idx)| ColorTensor::from_immutable(*kind, idx))
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

// ── Four-quark contact structures ─────────────────────────────────────────

/// A color string's tensors in the order its basis key uses (`to_immutable`
/// sorts them; `full_simplify` leaves a term it did not have to rewrite alone).
fn sorted(tensors: &[ColorTensor]) -> Vec<ColorTensor> {
    let mut out = tensors.to_vec();
    out.sort_by_key(|t| (t.kind(), t.indices()));
    out
}

/// The two colour structures a four-quark contact vertex writes reduce into one
/// two-element basis, with the decomposition and colour matrix MadGraph's own
/// `color_algebra.py` gives for the same index-labelled products.
///
/// Reference values, from the pinned submodule copy
/// (`research/refs/mg5amcnlo/madgraph/core/color_algebra.py`) on the strings
/// `u u~ > t t~` produces once the leg labels are substituted:
///
/// ```text
/// T(3,4)T(2,1)             -> (1 T(3,4) T(2,1))
/// T(-1,3,4)T(-1,2,1)       -> (1/2 T(2,4) T(3,1)) + (-1/2 1/Nc T(2,1) T(3,4))
/// <0|0> = 9   <0|1> = 3   <1|1> = 9        (Nc = 3)
/// ```
///
/// The singlet's own key and the octet's Fierz image are the *same* two
/// structures, which is what makes MadGraph's `NCOLOR` 2 rather than 4; both
/// readings of the delta cannot coexist in one basis, and the `3`/`3̄` slot
/// convention that keeps them consistent is [`convert_expr`]'s transpose.
///
/// [`convert_expr`]: super::colorize
#[test]
fn four_quark_contact_structures_share_one_basis() {
    let singlet = ColorFactor(vec![ColorString::new(vec![delta(3, 4), delta(2, 1)])]);
    let reduced = singlet.full_simplify();
    assert_eq!(
        reduced.0.len(),
        1,
        "singlet reduces to one term: {reduced:?}"
    );
    assert_eq!(
        sorted(&reduced.0[0].tensors),
        vec![delta(2, 1), delta(3, 4)]
    );
    assert_eq!(reduced.0[0].coeff.eval_nc(3), Ratio::from_integer(1));

    let octet = ColorFactor(vec![ColorString::new(vec![t(&[-1], 3, 4), t(&[-1], 2, 1)])]);
    let reduced = octet.full_simplify();
    let mut terms: Vec<(Vec<ColorTensor>, Ratio<i64>)> = reduced
        .0
        .iter()
        .map(|s| {
            assert!(!s.coeff.imag, "the octet Fierz is real: {s:?}");
            (sorted(&s.tensors), s.coeff.eval_nc(3))
        })
        .collect();
    terms.sort_by_key(|(ts, _)| format!("{ts:?}"));
    assert_eq!(
        terms,
        vec![
            (vec![delta(2, 1), delta(3, 4)], Ratio::new(-1, 6)),
            (vec![delta(2, 4), delta(3, 1)], Ratio::new(1, 2)),
        ],
        "octet Fierz against color_algebra.py"
    );
}

/// The colour matrix over that two-element basis, against `color_algebra.py`'s
/// `[[9, 3], [3, 9]]` — the numbers MadGraph's generated `matrix1_orig.f` writes
/// packed as `DATA (CF(I),I=1,2) /9,6/`, `DATA (CF(I),I=3,3) /9/` with `DENOM` 1.
#[test]
fn four_quark_basis_colour_matrix() {
    let basis = [
        vec![delta(2, 1), delta(3, 4)],
        vec![delta(2, 4), delta(3, 1)],
    ];
    let expected = [[9, 3], [3, 9]];
    for (i, bi) in basis.iter().enumerate() {
        for (j, bj) in basis.iter().enumerate() {
            let conj: Vec<ColorTensor> = bj.iter().map(ColorTensor::conj).collect();
            let mut tensors = bi.clone();
            tensors.extend(conj);
            let (re, im) =
                eval_scalar(&ColorFactor(vec![ColorString::new(tensors)]).full_simplify());
            assert_eq!(im, Ratio::from_integer(0), "CF[{i}][{j}] is real");
            assert_eq!(
                re,
                Ratio::from_integer(expected[i][j]),
                "CF[{i}][{j}] against color_algebra.py"
            );
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

// ── Baryonic invariants ───────────────────────────────────────────────────

/// `Epsilon` is totally antisymmetric: an odd permutation of its indices is the
/// negative of the sorted tensor, an even one the tensor itself.
///
/// The rewrite is what makes two epsilons written in different index orders
/// meet on one basis key, so getting its sign wrong would not fail loudly — it
/// would quietly put a `−1` on one diagram of a multi-diagram basis.
#[test]
fn epsilon_reorders_with_the_permutation_sign() {
    let sorted = ColorTensor::Epsilon(1, 2, 3);
    let cases = [
        ([1, 3, 2], -1i64),
        ([2, 1, 3], -1),
        ([3, 2, 1], -1),
        ([2, 3, 1], 1),
        ([3, 1, 2], 1),
    ];
    for ([a, b, c], sign) in cases {
        let out = ColorFactor(vec![ColorString::new(vec![ColorTensor::Epsilon(a, b, c)])])
            .full_simplify();
        assert_eq!(
            out.0.len(),
            1,
            "Epsilon({a},{b},{c}) must reduce to one term, got {out:?}"
        );
        assert_eq!(out.0[0].tensors, vec![sorted.clone()]);
        assert_eq!(
            out.0[0].coeff.eval_nc(3),
            Ratio::from_integer(sign),
            "Epsilon({a},{b},{c}) sign"
        );
    }
}

/// `EpsilonBar` reorders the same way, on its own indices.
#[test]
fn epsilon_bar_reorders_with_the_permutation_sign() {
    let out = ColorFactor(vec![ColorString::new(vec![ColorTensor::EpsilonBar(
        2, 1, 3,
    )])])
    .full_simplify();
    assert_eq!(out.0.len(), 1);
    assert_eq!(out.0[0].tensors, vec![ColorTensor::EpsilonBar(1, 2, 3)]);
    assert_eq!(out.0[0].coeff.eval_nc(3), Ratio::from_integer(-1));
}

/// Conjugation exchanges the two baryonic invariants and **keeps their index
/// order**: `Epsilon(i,j,k)* = EpsilonBar(i,j,k)`, not `EpsilonBar(k,j,i)`.
///
/// Reversing would be an odd permutation of three indices, so the two readings
/// differ by a `−1` on every colour-matrix entry an epsilon reaches.
#[test]
fn epsilon_conjugates_into_epsilon_bar_in_place() {
    assert_eq!(
        ColorTensor::Epsilon(1, 2, 3).conj(),
        ColorTensor::EpsilonBar(1, 2, 3)
    );
    assert_eq!(
        ColorTensor::EpsilonBar(1, 2, 3).conj(),
        ColorTensor::Epsilon(1, 2, 3)
    );
}

/// A delta walks through an epsilon, renaming the index it shares with it:
/// `e_ijk δ(l,k) = e_ijl` and `ebar_ijk δ(k,l) = ebar_ijl`. The two absorb
/// *opposite* ends of the delta, because an epsilon's indices are fundamental
/// and an epsilon-bar's antifundamental.
#[test]
fn epsilon_absorbs_a_delta_at_the_matching_end() {
    let eps = ColorFactor(vec![ColorString::new(vec![
        ColorTensor::Epsilon(1, 2, -1),
        delta(3, -1),
    ])])
    .full_simplify();
    assert_eq!(eps.0.len(), 1);
    assert_eq!(eps.0[0].tensors, vec![ColorTensor::Epsilon(1, 2, 3)]);

    let bar = ColorFactor(vec![ColorString::new(vec![
        ColorTensor::EpsilonBar(1, 2, -1),
        delta(-1, 3),
    ])])
    .full_simplify();
    assert_eq!(bar.0.len(), 1);
    assert_eq!(bar.0[0].tensors, vec![ColorTensor::EpsilonBar(1, 2, 3)]);
}

/// A contracted epsilon pair collapses to a *difference* of two delta products:
/// `e_{xij} ebar_{xkl} = δ(i,k)δ(j,l) − δ(i,l)δ(j,k)`.
///
/// This is the whole content of the `p3 r3 > p3 r3` diquark row — MadGraph
/// writes it as `JAMP(1) = AMP(1) + AMP(2)`, `JAMP(2) = −AMP(1)` — so the
/// relative minus is checked here at the level the row can only see squared.
#[test]
fn contracted_epsilon_pair_is_the_antisymmetrised_delta_product() {
    let out = ColorFactor(vec![ColorString::new(vec![
        ColorTensor::Epsilon(-1, 1, 2),
        ColorTensor::EpsilonBar(-1, 3, 4),
    ])])
    .full_simplify();
    assert_eq!(out.0.len(), 2, "expected two delta products, got {out:?}");
    let term = |tensors: Vec<ColorTensor>| {
        out.0
            .iter()
            .find(|s| s.tensors == tensors)
            .unwrap_or_else(|| panic!("missing term {tensors:?} in {out:?}"))
    };
    assert_eq!(
        term(vec![delta(1, 3), delta(2, 4)]).coeff.eval_nc(3),
        Ratio::from_integer(1)
    );
    assert_eq!(
        term(vec![delta(1, 4), delta(2, 3)]).coeff.eval_nc(3),
        Ratio::from_integer(-1)
    );
}

/// Two uncontracted epsilons expand into the six terms of `det δ`, three at
/// `+1` and three at `−1`.
#[test]
fn uncontracted_epsilon_pair_expands_into_six_delta_products() {
    let out = ColorFactor(vec![ColorString::new(vec![
        ColorTensor::Epsilon(1, 2, 3),
        ColorTensor::EpsilonBar(4, 5, 6),
    ])])
    .full_simplify();
    assert_eq!(out.0.len(), 6, "expected six terms, got {out:?}");
    let plus = out
        .0
        .iter()
        .filter(|s| s.coeff.eval_nc(3) == Ratio::from_integer(1))
        .count();
    assert_eq!(plus, 3, "three of the six terms carry +1: {out:?}");
}

/// The fully contracted pair is `ε_{ijk} ε̄_{ijk} = 3! = 6`, an oracle for the
/// two-term rule that involves neither MadGraph nor the rest of this engine:
/// it is the number of orderings of three colours, and it comes out of
/// `Nc² − Nc` at `Nc = 3` only if the relative minus is there.
#[test]
fn fully_contracted_epsilon_pair_counts_the_colour_orderings() {
    let out = ColorFactor(vec![ColorString::new(vec![
        ColorTensor::Epsilon(-1, -2, -3),
        ColorTensor::EpsilonBar(-1, -2, -3),
    ])])
    .full_simplify();
    assert_eq!(
        eval_scalar(&out),
        (Ratio::from_integer(6), Ratio::from_integer(0))
    );
}

// ── Sextet Clebsch–Gordan coefficients ────────────────────────────────────

/// A sextet resonance's colour flow: summing over the **6** index leaves the
/// *symmetric* pair of delta products at one half each,
/// `K6(m,i,j) K6Bar(m,k,l) = ½(T(l,i)T(k,j) + T(k,i)T(l,j))`.
///
/// The two halves are what MadGraph writes as `JAMP(1) = 0.5·AMP(1) + AMP(2)`
/// and `JAMP(2) = 0.5·AMP(1)` on the `p3 r3 > p3 r3` sextet row, against the
/// `+1`/`−1` its baryonic sibling carries: the sextet is the symmetric half of
/// the same `3 ⊗ 3`, so the relative sign between the two terms is `+` here and
/// `−` there, and that contrast is the whole content of the two rows.
#[test]
fn sextet_clebsch_pair_splits_into_symmetric_delta_halves() {
    let out = ColorFactor(vec![ColorString::new(vec![
        ColorTensor::K6(-1, 1, 2),
        ColorTensor::K6Bar(-1, 3, 4),
    ])])
    .full_simplify();
    assert_eq!(out.0.len(), 2, "expected two delta products, got {out:?}");
    for s in &out.0 {
        assert_eq!(
            s.coeff.eval_nc(3),
            Ratio::new(1, 2),
            "both terms carry +1/2: {out:?}"
        );
    }
    // The engine keeps a string's tensors in its own sorted order, so the terms
    // are matched as sets.
    let has = |mut tensors: Vec<ColorTensor>| {
        tensors.sort_by_key(ColorTensor::indices);
        out.0.iter().any(|s| s.tensors == tensors)
    };
    assert!(has(vec![delta(4, 1), delta(3, 2)]), "{out:?}");
    assert!(has(vec![delta(3, 1), delta(4, 2)]), "{out:?}");
}

/// Contracting both triplet indices instead closes the pair into the sextet
/// delta, either way round — `K6` is symmetric in them.
#[test]
fn sextet_clebsch_pair_closes_into_the_sextet_delta() {
    for (a, b) in [((-1, -2), (-1, -2)), ((-1, -2), (-2, -1))] {
        let out = ColorFactor(vec![ColorString::new(vec![
            ColorTensor::K6(1, a.0, a.1),
            ColorTensor::K6Bar(2, b.0, b.1),
        ])])
        .full_simplify();
        assert_eq!(out.0.len(), 1, "{a:?} {b:?}: {out:?}");
        assert_eq!(out.0[0].tensors, vec![ColorTensor::T6(1, 2)]);
        assert_eq!(out.0[0].coeff.eval_nc(3), Ratio::from_integer(1));
    }
}

/// The closed sextet delta is the dimension of the **6**: `δ6_{mm} = ½Nc(Nc+1)`,
/// which is `6` at `Nc = 3`. An independent count of the rep this engine claims
/// to be carrying.
#[test]
fn sextet_delta_trace_is_the_sextet_dimension() {
    let out = ColorFactor(vec![ColorString::new(vec![ColorTensor::T6(-1, -1)])]).full_simplify();
    assert_eq!(
        eval_scalar(&out),
        (Ratio::from_integer(6), Ratio::from_integer(0))
    );
}

/// The whole chain at once: `K6(m,i,j) K6Bar(n,i,j) δ6(n,m)` must count the
/// sextet's dimension too, reaching it through the Clebsch coefficients rather
/// than through the delta rule alone.
#[test]
fn contracted_clebsch_pair_counts_the_sextet_dimension() {
    let out = ColorFactor(vec![ColorString::new(vec![
        ColorTensor::K6(-1, -3, -4),
        ColorTensor::K6Bar(-2, -3, -4),
        ColorTensor::T6(-2, -1),
    ])])
    .full_simplify();
    assert_eq!(
        eval_scalar(&out),
        (Ratio::from_integer(6), Ratio::from_integer(0))
    );
}

/// A sextet delta walks through a Clebsch coefficient, renaming its sextet
/// index: `δ6(m,n) K6(n,i,j) = K6(m,i,j)` and `δ6(m,n) K6Bar(m,i,j) =
/// K6Bar(n,i,j)`. The two absorb *opposite* ends, as their reps require.
#[test]
fn sextet_delta_renames_a_clebsch_sextet_index() {
    let k6 = ColorFactor(vec![ColorString::new(vec![
        ColorTensor::T6(1, -1),
        ColorTensor::K6(-1, 2, 3),
    ])])
    .full_simplify();
    assert_eq!(k6.0.len(), 1);
    assert_eq!(k6.0[0].tensors, vec![ColorTensor::K6(1, 2, 3)]);

    let k6bar = ColorFactor(vec![ColorString::new(vec![
        ColorTensor::T6(-1, 2),
        ColorTensor::K6Bar(-1, 3, 4),
    ])])
    .full_simplify();
    assert_eq!(k6bar.0.len(), 1);
    assert_eq!(k6bar.0[0].tensors, vec![ColorTensor::K6Bar(2, 3, 4)]);
}

/// Conjugation exchanges the two Clebsch coefficients keeping their index
/// order, and transposes the sextet delta.
#[test]
fn sextet_tensors_conjugate_into_their_partners() {
    assert_eq!(ColorTensor::K6(1, 2, 3).conj(), ColorTensor::K6Bar(1, 2, 3));
    assert_eq!(ColorTensor::K6Bar(1, 2, 3).conj(), ColorTensor::K6(1, 2, 3));
    assert_eq!(ColorTensor::T6(1, 2).conj(), ColorTensor::T6(2, 1));
}
