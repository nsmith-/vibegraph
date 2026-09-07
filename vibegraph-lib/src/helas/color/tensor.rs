//! Generalized SU(3) color tensors and their single-object simplification
//! rules, ported from MadGraph's `color_algebra.py`.
//!
//! Index convention (MadGraph / the UFO paper): in `T(a1..an, i, j)` the
//! adjoint indices come first, then the fundamental (**3**) index `i`, then
//! the antifundamental (**3̄**) index `j`. A negative index is a summed
//! (contracted) index; a positive index labels an external leg.
//!
//! Representable: [`ColorTensor::T`], [`ColorTensor::Tr`], [`ColorTensor::F`],
//! [`ColorTensor::D`], [`ColorTensor::One`], the baryonic pair
//! [`ColorTensor::Epsilon`]/[`ColorTensor::EpsilonBar`], and the sextet
//! Clebsch-Gordan coefficients [`ColorTensor::K6`]/[`ColorTensor::K6Bar`] with
//! the sextet delta [`ColorTensor::T6`]. A `T6` carrying adjoint indices — the
//! sextet *generator*, whose expansion allocates fresh summed indices — is
//! rejected by colorize with [`ColorAlgebraError::Unsupported`] before it
//! reaches the algebra engine.

use thiserror::Error;

use super::coeff::ColorCoeff;
use super::factor::{ColorFactor, ColorString};

/// A color index. Negative values are summed (contracted) indices, positive
/// values label external legs.
pub type Idx = i32;

/// A color structure the algebra engine cannot handle.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ColorAlgebraError {
    /// A colour structure the engine does not represent — a sextet tensor
    /// (`K6`, `K6Bar`, `T6`) or a rep it cannot label — was encountered.
    #[error("unsupported color structure '{0}'")]
    Unsupported(String),
    /// A color basis key did not read as a consistent set of color lines over the
    /// process's external legs (see [`color_flow_tags`]).
    ///
    /// [`color_flow_tags`]: super::flow_tags::color_flow_tags
    #[error("inconsistent color flow: {0}")]
    InconsistentColorFlow(String),
}

/// Class tag used to order tensors inside an immutable/canonical form.
///
/// The discriminant order reproduces MadGraph's ordering of color-object class
/// names as Python strings: `ColorOne` < `Epsilon` < `EpsilonBar` < `K6` <
/// `K6Bar` < `T` < `T6` < `Tr` < `d` < `f`. Basis keys are sorted by this, so
/// the order is part of the interface to MadGraph's `JAMP` ordering and not a
/// free choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TensorKind {
    One,
    Epsilon,
    EpsilonBar,
    K6,
    K6Bar,
    T,
    T6,
    Tr,
    D,
    F,
}

/// A single generalized color tensor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColorTensor {
    /// `T(a1..an, i, j)`: a chain of fundamental generators, adjoint indices
    /// first, then the fundamental index `i` and antifundamental index `j`.
    /// With an empty adjoint list this is the Kronecker delta `δ_{ij}`.
    T(Vec<Idx>, Idx, Idx),
    /// `Tr(a1..an)`: a trace over a chain of fundamental generators.
    Tr(Vec<Idx>),
    /// `f(a,b,c)`: the totally antisymmetric structure constant.
    F(Idx, Idx, Idx),
    /// `d(a,b,c)`: the totally symmetric invariant.
    D(Idx, Idx, Idx),
    /// `Epsilon(i,j,k)`: the baryonic invariant of three fundamental indices.
    Epsilon(Idx, Idx, Idx),
    /// `EpsilonBar(i,j,k)`: the same over three antifundamental indices.
    EpsilonBar(Idx, Idx, Idx),
    /// `K6(m,i,j)`: the Clebsch–Gordan coefficient joining two antifundamental
    /// indices `i`, `j` to the sextet index `m`.
    K6(Idx, Idx, Idx),
    /// `K6Bar(m,i,j)`: the same joining two fundamental indices to a `6̄`.
    K6Bar(Idx, Idx, Idx),
    /// `T6(i,j)`: the sextet Kronecker delta `δ6_{ij}`.
    T6(Idx, Idx),
    /// The color identity `1` (no indices).
    One,
}

impl ColorTensor {
    /// The class tag, for immutable/canonical ordering.
    pub fn kind(&self) -> TensorKind {
        match self {
            ColorTensor::T(..) => TensorKind::T,
            ColorTensor::Tr(_) => TensorKind::Tr,
            ColorTensor::F(..) => TensorKind::F,
            ColorTensor::D(..) => TensorKind::D,
            ColorTensor::Epsilon(..) => TensorKind::Epsilon,
            ColorTensor::EpsilonBar(..) => TensorKind::EpsilonBar,
            ColorTensor::K6(..) => TensorKind::K6,
            ColorTensor::K6Bar(..) => TensorKind::K6Bar,
            ColorTensor::T6(..) => TensorKind::T6,
            ColorTensor::One => TensorKind::One,
        }
    }

    /// The tensor's index array, flattened in storage order. This is the sort
    /// key used to keep tensors in a deterministic order inside a color string
    /// (mirroring MadGraph's sort of the underlying integer arrays).
    pub fn indices(&self) -> Vec<Idx> {
        match self {
            ColorTensor::T(adj, i, j) => {
                let mut v = adj.clone();
                v.push(*i);
                v.push(*j);
                v
            }
            ColorTensor::Tr(adj) => adj.clone(),
            ColorTensor::F(a, b, c)
            | ColorTensor::D(a, b, c)
            | ColorTensor::Epsilon(a, b, c)
            | ColorTensor::EpsilonBar(a, b, c)
            | ColorTensor::K6(a, b, c)
            | ColorTensor::K6Bar(a, b, c) => vec![*a, *b, *c],
            ColorTensor::T6(i, j) => vec![*i, *j],
            ColorTensor::One => Vec::new(),
        }
    }

    /// Reconstruct a tensor from its immutable `(kind, indices)` pair (the
    /// inverse of [`ColorTensor::kind`] + [`ColorTensor::indices`]). For a `T`
    /// the last two indices are the fundamental/antifundamental pair and the
    /// rest are the adjoint chain.
    ///
    /// # Panics
    /// If `indices` is too short for `kind` (`T` needs ≥ 2, `F`/`D` need 3).
    pub fn from_immutable(kind: TensorKind, indices: &[Idx]) -> ColorTensor {
        match kind {
            TensorKind::One => ColorTensor::One,
            TensorKind::T => {
                let n = indices.len();
                assert!(n >= 2, "T tensor needs at least two indices");
                ColorTensor::T(indices[..n - 2].to_vec(), indices[n - 2], indices[n - 1])
            }
            TensorKind::Tr => ColorTensor::Tr(indices.to_vec()),
            TensorKind::F => {
                assert!(indices.len() == 3, "f tensor needs exactly three indices");
                ColorTensor::F(indices[0], indices[1], indices[2])
            }
            TensorKind::D => {
                assert!(indices.len() == 3, "d tensor needs exactly three indices");
                ColorTensor::D(indices[0], indices[1], indices[2])
            }
            TensorKind::Epsilon => {
                assert!(indices.len() == 3, "Epsilon needs exactly three indices");
                ColorTensor::Epsilon(indices[0], indices[1], indices[2])
            }
            TensorKind::EpsilonBar => {
                assert!(indices.len() == 3, "EpsilonBar needs exactly three indices");
                ColorTensor::EpsilonBar(indices[0], indices[1], indices[2])
            }
            TensorKind::K6 => {
                assert!(indices.len() == 3, "K6 needs exactly three indices");
                ColorTensor::K6(indices[0], indices[1], indices[2])
            }
            TensorKind::K6Bar => {
                assert!(indices.len() == 3, "K6Bar needs exactly three indices");
                ColorTensor::K6Bar(indices[0], indices[1], indices[2])
            }
            TensorKind::T6 => {
                assert!(indices.len() == 2, "T6 needs exactly two indices");
                ColorTensor::T6(indices[0], indices[1])
            }
        }
    }

    /// Complex conjugate of a single tensor.
    ///
    /// `T(a,b,c,i,j)* = T(c,b,a,j,i)` (reverse the adjoint chain, swap the two
    /// fundamental indices); `Epsilon* = EpsilonBar` and `K6* = K6Bar` and
    /// back, **keeping the index order** — conjugation exchanges the two
    /// representations rather than reordering one; every other tensor
    /// conjugates by reversing its index list.
    pub fn conj(&self) -> ColorTensor {
        match self {
            ColorTensor::T(adj, i, j) => {
                let mut r = adj.clone();
                r.reverse();
                ColorTensor::T(r, *j, *i)
            }
            ColorTensor::Tr(adj) => {
                let mut r = adj.clone();
                r.reverse();
                ColorTensor::Tr(r)
            }
            ColorTensor::F(a, b, c) => ColorTensor::F(*c, *b, *a),
            ColorTensor::D(a, b, c) => ColorTensor::D(*c, *b, *a),
            ColorTensor::Epsilon(a, b, c) => ColorTensor::EpsilonBar(*a, *b, *c),
            ColorTensor::EpsilonBar(a, b, c) => ColorTensor::Epsilon(*a, *b, *c),
            ColorTensor::K6(a, b, c) => ColorTensor::K6Bar(*a, *b, *c),
            ColorTensor::K6Bar(a, b, c) => ColorTensor::K6(*a, *b, *c),
            ColorTensor::T6(i, j) => ColorTensor::T6(*j, *i),
            ColorTensor::One => ColorTensor::One,
        }
    }

    /// Single-object simplification rules. Returns the replacement sum, or
    /// `None` if the tensor is already irreducible on its own.
    pub fn simplify(&self) -> Option<ColorFactor> {
        match self {
            ColorTensor::One => Some(ColorFactor(vec![ColorString::scalar(ColorCoeff::one())])),
            ColorTensor::F(a, b, c) => Some(f_to_traces(*a, *b, *c)),
            ColorTensor::D(a, b, c) => Some(d_to_traces(*a, *b, *c)),
            ColorTensor::Tr(idx) => tr_simplify(idx),
            ColorTensor::T(adj, i, j) => t_simplify(adj, *i, *j),
            ColorTensor::Epsilon(a, b, c) => {
                epsilon_sort(&[*a, *b, *c], |v| ColorTensor::Epsilon(v[0], v[1], v[2]))
            }
            ColorTensor::EpsilonBar(a, b, c) => {
                epsilon_sort(&[*a, *b, *c], |v| ColorTensor::EpsilonBar(v[0], v[1], v[2]))
            }
            // `K6`/`K6Bar` carry no single-object rule: `color_algebra.py`
            // leaves `use_symmetry` off, so the `K6(m,i,j) = K6(m,j,i)`
            // reordering never fires there either.
            ColorTensor::K6(..) | ColorTensor::K6Bar(..) => None,
            ColorTensor::T6(i, j) => t6_simplify(*i, *j),
        }
    }

    /// Two-object contraction rules. `self.pair_simplify(other)` is tried
    /// first, then `other.pair_simplify(self)`, so each ordered rule only
    /// needs to appear once.
    pub fn pair_simplify(&self, other: &ColorTensor) -> Option<ColorFactor> {
        match (self, other) {
            (ColorTensor::One, _) => Some(ColorFactor(vec![ColorString::new(vec![other.clone()])])),
            (ColorTensor::Tr(a), ColorTensor::Tr(b)) => tr_tr_pair(a, b),
            (ColorTensor::Tr(a), ColorTensor::T(adj, i, j)) => tr_t_pair(a, adj, *i, *j),
            (ColorTensor::T(sadj, si, sj), ColorTensor::T(oadj, oi, oj)) => {
                t_t_pair(sadj, *si, *sj, oadj, *oi, *oj)
            }
            (ColorTensor::Epsilon(a, b, c), ColorTensor::T(adj, i, j)) if adj.is_empty() => {
                // e(...,j,...) delta(i,j) = e(...,i,...)
                eps_delta_pair(&[*a, *b, *c], *j, *i, |v| {
                    ColorTensor::Epsilon(v[0], v[1], v[2])
                })
            }
            (ColorTensor::EpsilonBar(a, b, c), ColorTensor::T(adj, i, j)) if adj.is_empty() => {
                // ebar(...,i,...) delta(i,j) = ebar(...,j,...)
                eps_delta_pair(&[*a, *b, *c], *i, *j, |v| {
                    ColorTensor::EpsilonBar(v[0], v[1], v[2])
                })
            }
            (ColorTensor::Epsilon(a, b, c), ColorTensor::EpsilonBar(l, m, n)) => {
                Some(eps_epsbar_pair(&[*a, *b, *c], &[*l, *m, *n]))
            }
            (ColorTensor::K6(m, i, j), ColorTensor::K6Bar(n, k, l)) => {
                k6_k6bar_pair(*m, [*i, *j], *n, [*k, *l])
            }
            (ColorTensor::K6(m, i, j), ColorTensor::T(adj, x, y)) if adj.is_empty() => {
                // delta3(x,y) K6(m,x,k) = K6(m,k,y)
                k6_delta_pair(*m, [*i, *j], *x, *y, ColorTensor::K6)
            }
            (ColorTensor::K6Bar(m, i, j), ColorTensor::T(adj, x, y)) if adj.is_empty() => {
                // delta3(x,y) K6Bar(m,y,k) = K6Bar(m,k,x)
                k6_delta_pair(*m, [*i, *j], *y, *x, ColorTensor::K6Bar)
            }
            (ColorTensor::T6(i, j), ColorTensor::T6(k, l)) => (*k == *j)
                .then(|| ColorFactor(vec![ColorString::new(vec![ColorTensor::T6(*i, *l)])])),
            (ColorTensor::T6(m, n), ColorTensor::K6(a, i, j)) => (*a == *n)
                .then(|| ColorFactor(vec![ColorString::new(vec![ColorTensor::K6(*m, *i, *j)])])),
            (ColorTensor::T6(m, n), ColorTensor::K6Bar(a, i, j)) => (*a == *m)
                .then(|| ColorFactor(vec![ColorString::new(vec![ColorTensor::K6Bar(*n, *i, *j)])])),
            _ => None,
        }
    }
}

/// The sign of the permutation taking `lst` to its sorted order, by the
/// selection-sort walk `color_algebra.Epsilon.perm_parity` uses.
fn perm_parity(lst: &[Idx; 3]) -> i64 {
    let mut lst = *lst;
    let mut order = lst;
    order.sort_unstable();
    let mut parity = 1i64;
    for i in 0..lst.len() - 1 {
        if lst[i] != order[i] {
            parity = -parity;
            let mn = lst
                .iter()
                .position(|v| *v == order[i])
                .expect("sorted from lst");
            lst.swap(i, mn);
        }
    }
    parity
}

/// `Epsilon`/`EpsilonBar` single-object rule: rewrite to ascending index order,
/// carrying the permutation's sign (`epsilon(i,k,j) = -epsilon(i,j,k)`).
fn epsilon_sort(idx: &[Idx; 3], build: impl Fn([Idx; 3]) -> ColorTensor) -> Option<ColorFactor> {
    let mut sorted = *idx;
    sorted.sort_unstable();
    if sorted == *idx {
        return None;
    }
    Some(ColorFactor(vec![ColorString {
        coeff: ColorCoeff::rational(perm_parity(idx), 1),
        tensors: vec![build(sorted)],
    }]))
}

/// An epsilon absorbing a Kronecker delta: the index `from` on the epsilon is
/// replaced by `to`. `e_ijk T(l,k) = e_ijl` for an `Epsilon` (the delta's
/// antifundamental index is the one the epsilon carries) and
/// `ebar_ijk T(k,l) = ebar_ijl` for an `EpsilonBar`.
fn eps_delta_pair(
    idx: &[Idx; 3],
    from: Idx,
    to: Idx,
    build: impl Fn([Idx; 3]) -> ColorTensor,
) -> Option<ColorFactor> {
    let pos = idx.iter().position(|v| *v == from)?;
    let mut next = *idx;
    next[pos] = to;
    Some(ColorFactor(vec![ColorString::new(vec![build(next)])]))
}

/// `Epsilon`·`EpsilonBar`, MadGraph's two reduction rules.
///
/// With a summed index in common the pair collapses to two delta products,
/// `e_ijk ebar_ilm = T(j,l)T(k,m) - T(j,m)T(k,l)`, with both triples rotated so
/// the shared index leads. With no index in common it expands into the six
/// terms of `det(delta)`. Where several indices are shared MadGraph rotates on
/// the *last* one it finds, which this reproduces.
fn eps_epsbar_pair(eps: &[Idx; 3], aeps: &[Idx; 3]) -> ColorFactor {
    let mut common: Option<(usize, usize)> = None;
    for (pe, e) in eps.iter().enumerate() {
        if let Some(pa) = aeps.iter().position(|a| a == e) {
            common = Some((pe, pa));
        }
    }
    let delta = |x: Idx, y: Idx| ColorTensor::T(Vec::new(), x, y);
    if let Some((pe, pa)) = common {
        let rot = |v: &[Idx; 3], p: usize| [v[p], v[(p + 1) % 3], v[(p + 2) % 3]];
        let e = rot(eps, pe);
        let a = rot(aeps, pa);
        return ColorFactor(vec![
            ColorString {
                coeff: ColorCoeff::rational(1, 1),
                tensors: vec![delta(e[1], a[1]), delta(e[2], a[2])],
            },
            ColorString {
                coeff: ColorCoeff::rational(-1, 1),
                tensors: vec![delta(e[1], a[2]), delta(e[2], a[1])],
            },
        ]);
    }
    let [i, j, k] = *eps;
    let [l, m, n] = *aeps;
    let term = |q: i64, x: [(Idx, Idx); 3]| ColorString {
        coeff: ColorCoeff::rational(q, 1),
        tensors: x.iter().map(|&(a, b)| delta(a, b)).collect(),
    };
    ColorFactor(vec![
        term(1, [(i, l), (j, m), (k, n)]),
        term(1, [(i, m), (j, n), (k, l)]),
        term(1, [(i, n), (j, l), (k, m)]),
        term(-1, [(i, n), (j, m), (k, l)]),
        term(-1, [(i, m), (j, l), (k, n)]),
        term(-1, [(i, l), (j, n), (k, m)]),
    ])
}

/// The sextet delta's single-object rules: `delta6(i,i) = ½Nc(Nc+1)`; an open
/// `delta6(i,j)` is irreducible on its own.
fn t6_simplify(i: Idx, j: Idx) -> Option<ColorFactor> {
    if i != j {
        return None;
    }
    Some(ColorFactor(vec![
        ColorString {
            coeff: ColorCoeff {
                q: num_rational::Ratio::new(1, 2),
                imag: false,
                nc_power: 2,
            },
            tensors: Vec::new(),
        },
        ColorString {
            coeff: ColorCoeff {
                q: num_rational::Ratio::new(1, 2),
                imag: false,
                nc_power: 1,
            },
            tensors: Vec::new(),
        },
    ]))
}

/// `K6`·`K6Bar`, the three rules of `color_algebra.py`.
///
/// Sharing the sextet index sums over the **6** and leaves the symmetric pair of
/// delta products, `K6(m,i,j) K6Bar(m,k,l) = ½(T(l,i)T(k,j) + T(k,i)T(l,j))` —
/// the halves are what a sextet resonance's two colour flows carry. Sharing both
/// triplet indices instead closes the pair into the sextet delta `T6(m,n)`,
/// either way round, since `K6` is symmetric in them.
fn k6_k6bar_pair(m: Idx, ij: [Idx; 2], n: Idx, kl: [Idx; 2]) -> Option<ColorFactor> {
    let delta = |x: Idx, y: Idx| ColorTensor::T(Vec::new(), x, y);
    if m == n {
        return Some(ColorFactor(vec![
            ColorString {
                coeff: ColorCoeff::rational(1, 2),
                tensors: vec![delta(kl[1], ij[0]), delta(kl[0], ij[1])],
            },
            ColorString {
                coeff: ColorCoeff::rational(1, 2),
                tensors: vec![delta(kl[0], ij[0]), delta(kl[1], ij[1])],
            },
        ]));
    }
    let closed = (ij[1] == kl[0] && ij[0] == kl[1]) || (ij[0] == kl[0] && ij[1] == kl[1]);
    closed.then(|| ColorFactor(vec![ColorString::new(vec![ColorTensor::T6(m, n)])]))
}

/// A Kronecker delta walking into a `K6`/`K6Bar`: the matched triplet index is
/// replaced and moves to the end, the other one taking its place first —
/// `delta3(x,y) K6(m,x,k) = K6(m,k,y)`, verbatim from `color_algebra.py`.
fn k6_delta_pair(
    m: Idx,
    ij: [Idx; 2],
    matched: Idx,
    replacement: Idx,
    build: impl Fn(Idx, Idx, Idx) -> ColorTensor,
) -> Option<ColorFactor> {
    let pos = ij.iter().position(|v| *v == matched)?;
    let other = ij[1 - pos];
    Some(ColorFactor(vec![ColorString::new(vec![build(
        m,
        other,
        replacement,
    )])]))
}

/// `f(a,b,c) = -2i·Tr(a,b,c) + 2i·Tr(c,b,a)`.
fn f_to_traces(a: Idx, b: Idx, c: Idx) -> ColorFactor {
    ColorFactor(vec![
        ColorString {
            coeff: ColorCoeff {
                q: num_rational::Ratio::from_integer(-2),
                imag: true,
                nc_power: 0,
            },
            tensors: vec![ColorTensor::Tr(vec![a, b, c])],
        },
        ColorString {
            coeff: ColorCoeff {
                q: num_rational::Ratio::from_integer(2),
                imag: true,
                nc_power: 0,
            },
            tensors: vec![ColorTensor::Tr(vec![c, b, a])],
        },
    ])
}

/// `d(a,b,c) = 2·Tr(a,b,c) + 2·Tr(c,b,a)`.
fn d_to_traces(a: Idx, b: Idx, c: Idx) -> ColorFactor {
    ColorFactor(vec![
        ColorString {
            coeff: ColorCoeff::rational(2, 1),
            tensors: vec![ColorTensor::Tr(vec![a, b, c])],
        },
        ColorString {
            coeff: ColorCoeff::rational(2, 1),
            tensors: vec![ColorTensor::Tr(vec![c, b, a])],
        },
    ])
}

/// Coefficient `-1/(2·Nc)`, the recurring Fierz subtraction weight.
fn minus_half_over_nc() -> ColorCoeff {
    ColorCoeff {
        q: num_rational::Ratio::new(-1, 2),
        imag: false,
        nc_power: -1,
    }
}

/// `Tr` single-object rules: `Tr()=Nc`, `Tr(a)=0`, cyclic ordering from the
/// smallest index, and the within-trace Fierz identity.
fn tr_simplify(idx: &[Idx]) -> Option<ColorFactor> {
    // Tr(a) = 0
    if idx.len() == 1 {
        return Some(ColorFactor(vec![ColorString::scalar(ColorCoeff::zero())]));
    }
    // Tr() = Nc
    if idx.is_empty() {
        return Some(ColorFactor(vec![ColorString {
            coeff: ColorCoeff {
                q: num_rational::Ratio::from_integer(1),
                imag: false,
                nc_power: 1,
            },
            tensors: Vec::new(),
        }]));
    }
    // Cyclic: rotate to start from the smallest index.
    let minpos = idx
        .iter()
        .enumerate()
        .min_by_key(|(_, v)| **v)
        .map(|(p, _)| p)
        .expect("non-empty");
    if minpos != 0 {
        let mut rotated = idx[minpos..].to_vec();
        rotated.extend_from_slice(&idx[..minpos]);
        return Some(ColorFactor(vec![ColorString::new(vec![ColorTensor::Tr(
            rotated,
        )])]));
    }
    // Tr(a,x,b,x,c) = 1/2·Tr(a,c)·Tr(b) - 1/(2Nc)·Tr(a,b,c)
    for i1 in 0..idx.len() {
        for i2 in (i1 + 1)..idx.len() {
            if idx[i1] == idx[i2] {
                let a = &idx[..i1];
                let b = &idx[i1 + 1..i2];
                let c = &idx[i2 + 1..];
                let ac = concat(&[a, c]);
                let abc = concat(&[a, b, c]);
                return Some(ColorFactor(vec![
                    ColorString {
                        coeff: ColorCoeff::rational(1, 2),
                        tensors: vec![ColorTensor::Tr(ac), ColorTensor::Tr(b.to_vec())],
                    },
                    ColorString {
                        coeff: minus_half_over_nc(),
                        tensors: vec![ColorTensor::Tr(abc)],
                    },
                ]));
            }
        }
    }
    None
}

/// `T` single-object rules: `T(...,i,i)=Tr(...)` and the within-`T` Fierz
/// identity.
fn t_simplify(adj: &[Idx], i: Idx, j: Idx) -> Option<ColorFactor> {
    // T(a,b,c,...,i,i) = Tr(a,b,c,...)
    if i == j {
        return Some(ColorFactor(vec![ColorString::new(vec![ColorTensor::Tr(
            adj.to_vec(),
        )])]));
    }
    // T(a,x,b,x,c,i,j) = 1/2·T(a,c,i,j)·Tr(b) - 1/(2Nc)·T(a,b,c,i,j)
    for i1 in 0..adj.len() {
        for i2 in (i1 + 1)..adj.len() {
            if adj[i1] == adj[i2] {
                let a = &adj[..i1];
                let b = &adj[i1 + 1..i2];
                let c = &adj[i2 + 1..];
                let ac = concat(&[a, c]);
                let abc = concat(&[a, b, c]);
                return Some(ColorFactor(vec![
                    ColorString {
                        coeff: ColorCoeff::rational(1, 2),
                        tensors: vec![ColorTensor::T(ac, i, j), ColorTensor::Tr(b.to_vec())],
                    },
                    ColorString {
                        coeff: minus_half_over_nc(),
                        tensors: vec![ColorTensor::T(abc, i, j)],
                    },
                ]));
            }
        }
    }
    None
}

/// `Tr(a,x,b)·Tr(c,x,d) = 1/2·Tr(a,d,c,b) - 1/(2Nc)·Tr(a,b)·Tr(c,d)`.
fn tr_tr_pair(s: &[Idx], o: &[Idx]) -> Option<ColorFactor> {
    for i1 in 0..s.len() {
        for i2 in 0..o.len() {
            if s[i1] == o[i2] {
                let a = &s[..i1];
                let b = &s[i1 + 1..];
                let c = &o[..i2];
                let d = &o[i2 + 1..];
                let adcb = concat(&[a, d, c, b]);
                let ab = concat(&[a, b]);
                let cd = concat(&[c, d]);
                return Some(ColorFactor(vec![
                    ColorString {
                        coeff: ColorCoeff::rational(1, 2),
                        tensors: vec![ColorTensor::Tr(adcb)],
                    },
                    ColorString {
                        coeff: minus_half_over_nc(),
                        tensors: vec![ColorTensor::Tr(ab), ColorTensor::Tr(cd)],
                    },
                ]));
            }
        }
    }
    None
}

/// `Tr(a,x,b)·T(c,x,d,i,j) = 1/2·T(c,b,a,d,i,j) - 1/(2Nc)·Tr(a,b)·T(c,d,i,j)`.
fn tr_t_pair(s: &[Idx], oadj: &[Idx], oi: Idx, oj: Idx) -> Option<ColorFactor> {
    for i1 in 0..s.len() {
        for i2 in 0..oadj.len() {
            if s[i1] == oadj[i2] {
                let a = &s[..i1];
                let b = &s[i1 + 1..];
                let c = &oadj[..i2];
                let d = &oadj[i2 + 1..];
                let cbad = concat(&[c, b, a, d]);
                let ab = concat(&[a, b]);
                let cd = concat(&[c, d]);
                return Some(ColorFactor(vec![
                    ColorString {
                        coeff: ColorCoeff::rational(1, 2),
                        tensors: vec![ColorTensor::T(cbad, oi, oj)],
                    },
                    ColorString {
                        coeff: minus_half_over_nc(),
                        tensors: vec![ColorTensor::Tr(ab), ColorTensor::T(cd, oi, oj)],
                    },
                ]));
            }
        }
    }
    None
}

/// `T` product rules: chain merge `T(A,i,j)·T(B,j,k) = T(A,B,i,k)` and, when
/// the chains share an adjoint index instead, the `T·T` Fierz identity.
fn t_t_pair(sadj: &[Idx], si: Idx, sj: Idx, oadj: &[Idx], oi: Idx, oj: Idx) -> Option<ColorFactor> {
    // T(a,...,i,j)·T(b,...,j,k) = T(a,...,b,...,i,k)
    if sj == oi {
        let merged = concat(&[sadj, oadj]);
        return Some(ColorFactor(vec![ColorString::new(vec![ColorTensor::T(
            merged, si, oj,
        )])]));
    }
    // T(a,x,b,i,j)·T(c,x,d,k,l) = 1/2·T(a,d,i,l)·T(c,b,k,j)
    //                           - 1/(2Nc)·T(a,b,i,j)·T(c,d,k,l)
    for i1 in 0..sadj.len() {
        for i2 in 0..oadj.len() {
            if sadj[i1] == oadj[i2] {
                let a = &sadj[..i1];
                let b = &sadj[i1 + 1..];
                let c = &oadj[..i2];
                let d = &oadj[i2 + 1..];
                let ad = concat(&[a, d]);
                let cb = concat(&[c, b]);
                let ab = concat(&[a, b]);
                let cd = concat(&[c, d]);
                return Some(ColorFactor(vec![
                    ColorString {
                        coeff: ColorCoeff::rational(1, 2),
                        tensors: vec![ColorTensor::T(ad, si, oj), ColorTensor::T(cb, oi, sj)],
                    },
                    ColorString {
                        coeff: minus_half_over_nc(),
                        tensors: vec![ColorTensor::T(ab, si, sj), ColorTensor::T(cd, oi, oj)],
                    },
                ]));
            }
        }
    }
    None
}

/// Concatenate index slices into one vector.
fn concat(parts: &[&[Idx]]) -> Vec<Idx> {
    let mut v = Vec::new();
    for p in parts {
        v.extend_from_slice(p);
    }
    v
}
