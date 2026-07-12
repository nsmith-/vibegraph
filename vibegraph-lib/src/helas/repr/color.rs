//! Color (gauge group) representation traits.
//!
//! ## Design: exact group-theoretic scalars
//!
//! The Casimir invariants and Dynkin indices are rational numbers; using `f64`
//! for them would introduce rounding errors in expressions that should be exact
//! fractions. Instead, the trait uses an associated type `GroupScalar` and
//! concrete implementations return `num_rational::Ratio<i64>`.
//!
//! ## Scalar library
//!
//! The scalar type is `num_rational::Ratio<i64>`: exact rational arithmetic
//! over machine integers, with checked operations that panic on overflow (see
//! the `color` algebra engine). The [`GroupScalar`] trait boundary insulates
//! downstream code from that choice, so `Ratio<i128>` — or an arbitrary-
//! precision crate — remains a drop-in escape hatch if the tree-level factors
//! ever outgrow `i64`.
//!
//! ## SU(3) representations
//!
//! | Type            | dim | C2(R) | T(R) |
//! |-----------------|-----|-------|------|
//! | SU3Fundamental  |   3 |   4/3 |  1/2 |
//! | SU3Adjoint      |   8 |     3 |    3 |
//! | ColorSinglet    |   1 |     0 |    0 |

use super::{Real, C};
use num_rational::Ratio;

/// An SU(3) color representation, tagged by its UFO color charge.
///
/// This is the lightweight, runtime-value counterpart of the [`ColorRepr`]
/// marker types: colorize and the `Identity` resolution key off it, and each
/// marker type names its rep through [`ColorRepr::REP`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorRep {
    /// The trivial **1** (leptons, photon, …).
    Singlet,
    /// The fundamental **3** (quarks).
    Triplet,
    /// The antifundamental **3̄** (antiquarks).
    AntiTriplet,
    /// The adjoint **8** (gluons).
    Octet,
}

impl ColorRep {
    /// Map a UFO `color` charge to a representation: `1 → Singlet`,
    /// `3 → Triplet`, `-3 → AntiTriplet`, `8 → Octet`. The self-conjugate reps
    /// also accept their negated charge, which the antiparticle constructor
    /// produces (`color: -self.color`): `-1 → Singlet`, `-8 → Octet`. Any other
    /// value (e.g. a sextet `±6`) returns `None`.
    pub fn from_ufo(color: i32) -> Option<Self> {
        match color {
            1 | -1 => Some(ColorRep::Singlet),
            3 => Some(ColorRep::Triplet),
            -3 => Some(ColorRep::AntiTriplet),
            8 | -8 => Some(ColorRep::Octet),
            _ => None,
        }
    }

    /// The conjugate representation (`3 ↔ 3̄`; self-conjugate otherwise).
    pub fn anti(self) -> Self {
        match self {
            ColorRep::Singlet => ColorRep::Singlet,
            ColorRep::Triplet => ColorRep::AntiTriplet,
            ColorRep::AntiTriplet => ColorRep::Triplet,
            ColorRep::Octet => ColorRep::Octet,
        }
    }
}

#[cfg(test)]
mod color_rep_tests {
    use super::ColorRep;

    /// The antiparticle constructor negates the UFO `color` charge, so the
    /// self-conjugate reps arrive as `-1` (singlet) and `-8` (octet) on internal
    /// lines; both must resolve to the same rep as their positive charge.
    #[test]
    fn from_ufo_self_conjugate_negated() {
        assert_eq!(ColorRep::from_ufo(-1), Some(ColorRep::Singlet));
        assert_eq!(ColorRep::from_ufo(1), Some(ColorRep::Singlet));
        assert_eq!(ColorRep::from_ufo(-8), Some(ColorRep::Octet));
        assert_eq!(ColorRep::from_ufo(8), Some(ColorRep::Octet));
    }

    /// The triplet stays chiral: `3` and `-3` are distinct conjugate reps.
    #[test]
    fn from_ufo_triplet_is_chiral() {
        assert_eq!(ColorRep::from_ufo(3), Some(ColorRep::Triplet));
        assert_eq!(ColorRep::from_ufo(-3), Some(ColorRep::AntiTriplet));
        assert_eq!(
            ColorRep::from_ufo(3).map(ColorRep::anti),
            ColorRep::from_ufo(-3)
        );
    }

    /// Sextets (and any other charge) remain unsupported.
    #[test]
    fn from_ufo_sextet_unsupported() {
        assert_eq!(ColorRep::from_ufo(6), None);
        assert_eq!(ColorRep::from_ufo(-6), None);
    }
}

/// Exact rational scalar for group-theoretic constants.
///
/// Implemented by any type that behaves as a rational number: arithmetic,
/// copy semantics, ordering, and display. Concrete implementations should use
/// `Ratio<i64>` (from `num-rational`) for Casimir invariants.
///
/// The choice of library (num-rational vs. rug for arbitrary precision) is
/// deferred; this trait boundary insulates downstream code from the decision.
pub trait GroupScalar:
    num_traits::Num + Copy + std::fmt::Debug + std::fmt::Display + PartialOrd + 'static
{
}

impl GroupScalar for Ratio<i64> {}
impl GroupScalar for i64 {}
impl GroupScalar for i32 {}

/// A colour representation of gauge group G.
///
/// Types implementing this trait are marker types for one irreducible
/// representation of G. They do not store numerical data; the data lives in
/// the `Color` fiber.
///
/// The numeric `Color` fiber (`[C<F>; DIM]`) is the typed vocabulary of
/// hand-built wavefunction objects; the symbolic color pipeline (the `color`
/// algebra engine) does not use it — the runtime carries no color vector.
///
/// ## Associated items
/// - `Color` -- the complex vector space carrying the representation
/// - `GroupScalar` -- exact rational type for group-theoretic constants
/// - `DIM` -- dimension of the representation
/// - `casimir()` -- quadratic Casimir C2(R) defined by T^a T^a = C2(R) * 1
/// - `dynkin()` -- Dynkin index T(R) defined by Tr[T^a T^b] = T(R) * delta^{ab}
pub trait ColorRepr<F: Real>: Sized + Copy + 'static {
    /// The fiber over each momentum point: a complex vector of dimension `DIM`.
    type Color: Copy;

    /// Exact rational type for group-theoretic constants.
    type GroupScalar: self::GroupScalar;

    /// Dimension of the representation.
    const DIM: usize;

    /// The runtime-value tag for this representation.
    const REP: ColorRep;

    /// Quadratic Casimir invariant C2(R).
    ///
    /// Defined by T^a_{ij} T^a_{jk} = C2(R) * delta_{ik}.
    fn casimir() -> Self::GroupScalar;

    /// Dynkin index T(R).
    ///
    /// Defined by Tr[T^a T^b] = T(R) * delta^{ab}.
    fn dynkin() -> Self::GroupScalar;
}

// ─────────────────────────────────────────────────────────────────────────────
// SU(3) fundamental representation (quark triplet)
// ─────────────────────────────────────────────────────────────────────────────

/// Marker type for the SU(3) fundamental representation.
///
/// Quarks transform in this **3** representation. The generators T^a (a=1..8)
/// are 3x3 traceless Hermitian matrices normalised as Tr[T^a T^b] = (1/2) delta^{ab}.
///
/// Group-theoretic values:
/// - `DIM = 3`
/// - `C2(F) = 4/3` (quadratic Casimir for the fundamental)
/// - `T(F) = 1/2` (Dynkin index)
#[derive(Clone, Copy, Debug)]
pub struct SU3Fundamental;

impl<F: Real> ColorRepr<F> for SU3Fundamental {
    type Color = [C<F>; 3];
    type GroupScalar = Ratio<i64>;
    const DIM: usize = 3;
    const REP: ColorRep = ColorRep::Triplet;

    fn casimir() -> Ratio<i64> {
        Ratio::new(4, 3)
    }

    fn dynkin() -> Ratio<i64> {
        Ratio::new(1, 2)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SU(3) adjoint representation (gluon octet)
// ─────────────────────────────────────────────────────────────────────────────

/// Marker type for the SU(3) adjoint representation.
///
/// Gluons transform in this **8** representation. The generators are the
/// structure constants `(T^a)_{bc} = -i f^{abc}`.
///
/// Group-theoretic values:
/// - `DIM = 8`
/// - `C2(A) = 3` (quadratic Casimir for the adjoint)
/// - `T(A) = 3` (Dynkin index)
///
/// ## Symbolic color, not numeric structure constants
///
/// Color is factored out of the amplitude *symbolically*: the `color` algebra
/// engine reduces every color structure to a basis of generalized traces/deltas
/// with exact rational coefficients, and the constant color matrix is evaluated
/// once per process. The floating-point runtime therefore never contracts
/// `f^{abc}` numerically — there is no structure-constant table, and none is
/// needed. (The Mangano–Parke–Xu leading-Nc flow decomposition is likewise not
/// used here; it returns only for LHEF color tags, a separate feature.)
#[derive(Clone, Copy, Debug)]
pub struct SU3Adjoint;

impl<F: Real> ColorRepr<F> for SU3Adjoint {
    type Color = [C<F>; 8];
    type GroupScalar = Ratio<i64>;
    const DIM: usize = 8;
    const REP: ColorRep = ColorRep::Octet;

    fn casimir() -> Ratio<i64> {
        Ratio::new(3, 1)
    }

    fn dynkin() -> Ratio<i64> {
        Ratio::new(3, 1)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Color singlet (trivial representation)
// ─────────────────────────────────────────────────────────────────────────────

/// Marker type for the colour-neutral (singlet) representation.
///
/// Leptons and photons are colour-neutral. The colour factor for any amplitude
/// involving only singlets is simply 1; the wavefunction is `C<F> = 1 + 0i`.
///
/// Group-theoretic values: DIM=1, C2=0, T=0.
#[derive(Clone, Copy, Debug)]
pub struct ColorSinglet;

impl<F: Real> ColorRepr<F> for ColorSinglet {
    type Color = C<F>;
    type GroupScalar = Ratio<i64>;
    const DIM: usize = 1;
    const REP: ColorRep = ColorRep::Singlet;

    fn casimir() -> Ratio<i64> {
        Ratio::new(0, 1)
    }

    fn dynkin() -> Ratio<i64> {
        Ratio::new(0, 1)
    }
}
