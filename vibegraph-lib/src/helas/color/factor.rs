//! Color strings, color factors, and the fixpoint simplification engine.
//!
//! A [`ColorString`] is a [`ColorCoeff`] times an ordered product of
//! [`ColorTensor`]s; a [`ColorFactor`] is a sum of color strings. The engine
//! ports MadGraph's `ColorString.simplify` / `ColorFactor.simplify` /
//! `full_simplify` faithfully: single-object rules are applied before pair
//! rules, only the first applicable rewrite fires per pass, similar strings
//! are merged, and the whole factor is iterated to a fixed point.

use std::collections::HashMap;

use super::coeff::ColorCoeff;
use super::tensor::{ColorTensor, Idx, TensorKind};

/// Upper bound on simplification passes. The SU(3) rules strictly reduce the
/// structures they touch, so a real computation converges far below this; a
/// blown limit means a rule is cycling and is worth a loud failure.
const MAX_PASSES: usize = 100_000;

/// One term of a [`ColorFactor`]: a scalar coefficient times a product of
/// color tensors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColorString {
    pub coeff: ColorCoeff,
    pub tensors: Vec<ColorTensor>,
}

/// A color string in the immutable form MadGraph uses as a basis key: the
/// tensors as `(kind, indices)` pairs, sorted. The scalar coefficient is *not*
/// part of the key.
pub type ImmutableString = Vec<(TensorKind, Vec<Idx>)>;

/// A color string reduced to canonical form: the immutable form with every
/// index relabelled `1, 2, 3, …` by order of first appearance, then re-sorted.
/// Two color strings share a canonical form iff they are equal up to a
/// relabelling of their indices.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CanonicalString(pub Vec<(TensorKind, Vec<Idx>)>);

impl ColorString {
    /// A string of the given tensors with unit coefficient.
    pub fn new(tensors: Vec<ColorTensor>) -> Self {
        ColorString {
            coeff: ColorCoeff::one(),
            tensors,
        }
    }

    /// A tensor-free string carrying only a scalar coefficient.
    pub fn scalar(coeff: ColorCoeff) -> Self {
        ColorString {
            coeff,
            tensors: Vec::new(),
        }
    }

    /// Multiply in place by another string: coefficients combine and the other
    /// string's tensors are appended.
    fn product(&mut self, other: &ColorString) {
        self.coeff = self.coeff.mul(&other.coeff);
        self.tensors.extend(other.tensors.iter().cloned());
    }

    /// Sort the tensors by their flattened index arrays (stable), matching
    /// MadGraph's within-string ordering.
    fn sort_tensors(&mut self) {
        self.tensors.sort_by_key(|a| a.indices());
    }

    /// Complex conjugate: conjugate the coefficient and every tensor.
    pub fn conj(&self) -> ColorString {
        ColorString {
            coeff: self.coeff.conj(),
            tensors: self.tensors.iter().map(ColorTensor::conj).collect(),
        }
    }

    /// Reconstruct a unit-coefficient string from an immutable representation
    /// (the inverse of [`ColorString::to_immutable`]). A lone `ColorOne` entry
    /// yields an empty tensor product.
    pub fn from_immutable(rep: &ImmutableString) -> ColorString {
        let tensors = rep
            .iter()
            .filter(|(kind, _)| *kind != TensorKind::One)
            .map(|(kind, idxs)| ColorTensor::from_immutable(*kind, idxs))
            .collect();
        ColorString {
            coeff: ColorCoeff::one(),
            tensors,
        }
    }

    /// The immutable (basis-key) form: `(kind, indices)` pairs, sorted. An
    /// empty product folds to a single `ColorOne` entry.
    pub fn to_immutable(&self) -> ImmutableString {
        let mut list: ImmutableString = self
            .tensors
            .iter()
            .map(|t| (t.kind(), t.indices()))
            .collect();
        if list.is_empty() && !self.coeff.is_zero() {
            list.push((TensorKind::One, Vec::new()));
        }
        list.sort();
        list
    }

    /// The canonical form together with the index relabelling used to build it
    /// (`old_index -> new_index`). Indices are numbered `1, 2, 3, …` in order
    /// of first appearance across the sorted immutable form.
    pub fn to_canonical(&self) -> (CanonicalString, HashMap<Idx, Idx>) {
        let immutable = self.to_immutable();
        let mut repl: HashMap<Idx, Idx> = HashMap::new();
        let mut next: Idx = 1;
        let mut out: Vec<(TensorKind, Vec<Idx>)> = Vec::with_capacity(immutable.len());
        for (kind, idxs) in &immutable {
            let mut renamed = Vec::with_capacity(idxs.len());
            for &idx in idxs {
                let ni = *repl.entry(idx).or_insert_with(|| {
                    let v = next;
                    next += 1;
                    v
                });
                renamed.push(ni);
            }
            out.push((*kind, renamed));
        }
        out.sort();
        (CanonicalString(out), repl)
    }

    /// The canonical form alone.
    pub fn canonical(&self) -> CanonicalString {
        self.to_canonical().0
    }

    /// Whether two strings may be added: identical concrete tensor structure
    /// (same [`to_immutable`](ColorString::to_immutable) form), same `i` flag,
    /// same `Nc` power (the rational magnitudes need not match).
    ///
    /// The comparison is over the concrete indices, not the index-relabelled
    /// canonical form: `Tr(1,2,3,4)` and `Tr(1,2,4,3)` share a canonical form
    /// but are distinct color structures and must not be merged. This mirrors
    /// MadGraph's `is_similar`, whose `to_canonical()` comparison includes the
    /// index-replacement dict and so is likewise concrete.
    pub fn is_similar(&self, other: &ColorString) -> bool {
        self.coeff.can_add(&other.coeff) && self.to_immutable() == other.to_immutable()
    }

    /// Full equality used for the fixpoint test: similar *and* equal rational
    /// magnitude.
    pub fn equiv(&self, other: &ColorString) -> bool {
        self.is_similar(other) && self.coeff.q == other.coeff.q
    }

    /// Apply one simplification step to this string. Single-object rules are
    /// tried first (in tensor order), then pair rules (in nested order), and
    /// only the first applicable rewrite fires. Returns the replacement sum,
    /// or `None` if the string is irreducible.
    fn simplify(&self) -> Option<ColorFactor> {
        // Single-object rules.
        for i1 in 0..self.tensors.len() {
            if let Some(res) = self.tensors[i1].simplify() {
                let mut out = Vec::with_capacity(res.0.len());
                for second in &res.0 {
                    let mut first = self.clone();
                    first.tensors.remove(i1);
                    first.product(second);
                    first.sort_tensors();
                    out.push(first);
                }
                return Some(ColorFactor(out));
            }
        }
        // Pair rules.
        for i1 in 0..self.tensors.len() {
            for i2 in (i1 + 1)..self.tensors.len() {
                let res = self.tensors[i1]
                    .pair_simplify(&self.tensors[i2])
                    .or_else(|| self.tensors[i2].pair_simplify(&self.tensors[i1]));
                if let Some(res) = res {
                    let mut out = Vec::with_capacity(res.0.len());
                    for second in &res.0 {
                        let mut first = self.clone();
                        first.tensors.remove(i2);
                        first.tensors.remove(i1);
                        first.product(second);
                        first.sort_tensors();
                        out.push(first);
                    }
                    return Some(ColorFactor(out));
                }
            }
        }
        None
    }
}

/// A sum of [`ColorString`]s.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColorFactor(pub Vec<ColorString>);

impl ColorFactor {
    /// An empty (zero) color factor.
    pub fn zero() -> Self {
        ColorFactor(Vec::new())
    }

    /// Append a string, merging it into an existing similar string (adding the
    /// coefficients) when one is present.
    fn append_str(&mut self, new: ColorString) {
        for existing in &mut self.0 {
            if existing.is_similar(&new) {
                existing.coeff = existing.coeff.add(&new.coeff);
                return;
            }
        }
        self.0.push(new);
    }

    /// One simplification pass over every string, merging similar results and
    /// dropping strings whose coefficient has become zero.
    pub fn simplify(&self) -> ColorFactor {
        let mut out = ColorFactor::zero();
        for cs in &self.0 {
            match cs.simplify() {
                Some(res) => {
                    for s in res.0 {
                        out.append_str(s);
                    }
                }
                None => out.append_str(cs.clone()),
            }
        }
        out.0.retain(|s| !s.coeff.is_zero());
        out
    }

    /// Iterate [`simplify`](ColorFactor::simplify) to a fixed point.
    pub fn full_simplify(&self) -> ColorFactor {
        let mut result = self.clone();
        for _ in 0..MAX_PASSES {
            let next = result.simplify();
            if factor_equiv(&next, &result) {
                return next;
            }
            result = next;
        }
        panic!("ColorFactor::full_simplify did not converge within {MAX_PASSES} passes");
    }

    /// Complex conjugate: conjugate every string.
    pub fn conj(&self) -> ColorFactor {
        ColorFactor(self.0.iter().map(ColorString::conj).collect())
    }
}

/// Structural equality of two color factors, term by term, using
/// [`ColorString::equiv`]. Used only as the fixpoint test.
fn factor_equiv(a: &ColorFactor, b: &ColorFactor) -> bool {
    a.0.len() == b.0.len() && a.0.iter().zip(&b.0).all(|(x, y)| x.equiv(y))
}
