//! Generalized SU(3) color tensors and their single-object simplification
//! rules, ported from MadGraph's `color_algebra.py`.
//!
//! Index convention (MadGraph / the UFO paper): in `T(a1..an, i, j)` the
//! adjoint indices come first, then the fundamental (**3**) index `i`, then
//! the antifundamental (**3̄**) index `j`. A negative index is a summed
//! (contracted) index; a positive index labels an external leg.
//!
//! Only the structures the Standard Model tree-level algebra needs are
//! representable: [`ColorTensor::T`], [`ColorTensor::Tr`], [`ColorTensor::F`],
//! [`ColorTensor::D`], and [`ColorTensor::One`]. Sextet (`K6`/`K6Bar`/`T6`) and
//! baryonic (`Epsilon`) tensors are deliberately *not* representable; colorize
//! must reject them with [`ColorAlgebraError::Unsupported`] before reaching the
//! algebra engine.

use thiserror::Error;

use super::coeff::ColorCoeff;
use super::factor::{ColorFactor, ColorString};

/// A color index. Negative values are summed (contracted) indices, positive
/// values label external legs.
pub type Idx = i32;

/// A color structure the algebra engine cannot handle.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ColorAlgebraError {
    /// A sextet or baryonic tensor (`K6`, `K6Bar`, `T6`, `Epsilon`) was
    /// encountered; these are outside the tree-level SM color vocabulary.
    #[error("unsupported color structure '{0}': sextet/baryonic tensors are not implemented")]
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
/// names as Python strings: `ColorOne` < `T` < `Tr` < `d` < `f`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TensorKind {
    One,
    T,
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
            ColorTensor::F(a, b, c) | ColorTensor::D(a, b, c) => vec![*a, *b, *c],
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
        }
    }

    /// Complex conjugate of a single tensor.
    ///
    /// `T(a,b,c,i,j)* = T(c,b,a,j,i)` (reverse the adjoint chain, swap the two
    /// fundamental indices); every other tensor conjugates by reversing its
    /// index list.
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
            _ => None,
        }
    }
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
