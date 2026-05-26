//! Color (gauge group) representation traits.
//!
//! ## Design: exact group-theoretic scalars
//!
//! The Casimir invariants and Dynkin indices are rational numbers; using `f64`
//! for them would introduce rounding errors in expressions that should be exact
//! fractions. Instead, the trait uses an associated type `GroupScalar` and
//! concrete implementations return `num_rational::Ratio<i64>`.
//!
//! # TODO: Library choice
//! Two crates are candidates for the scalar type:
//! - `num-rational` (current choice): exact rational arithmetic over machine integers.
//! - `rug` (via GMP/MPFR): arbitrary-precision integers/rationals. More powerful,
//!   but a heavier dependency. Worth revisiting once the amplitude pipeline matures.
//!
//! ## SU(3) representations
//!
//! | Type            | dim | C2(R) | T(R) |
//! |-----------------|-----|-------|------|
//! | SU3Fundamental  |   3 |   4/3 |  1/2 |
//! | SU3Adjoint      |   8 |     3 |    3 |
//! | ColorSinglet    |   1 |     0 |    0 |

use super::{C, Real};
use num_rational::Ratio;

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
/// The `f^{abc}` structure constants are stored as an `[[f64; 8]; 8]` constant
/// array internally.
///
/// # RESEARCH: Symbolic evaluation of color traces
///
/// Before performing numerical 8x8 matrix multiplications with the `f^{abc}`
/// structure constants, it may be possible to simplify color-factor expressions
/// symbolically using a non-commutative algebra library. Reducing traces of
/// products of generators to canonical form (via Fierz/Jacobi identities) first
/// can drastically reduce the number of floating-point operations at evaluation
/// time. Survey available Rust/Python non-commutative algebra tools and see
/// whether color-flow decomposition (Mangano-Parke-Xu) can be used to avoid
/// explicit `f^{abc}` contractions entirely.
#[derive(Clone, Copy, Debug)]
pub struct SU3Adjoint;

impl<F: Real> ColorRepr<F> for SU3Adjoint {
    type Color = [C<F>; 8];
    type GroupScalar = Ratio<i64>;
    const DIM: usize = 8;

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

    fn casimir() -> Ratio<i64> {
        Ratio::new(0, 1)
    }

    fn dynkin() -> Ratio<i64> {
        Ratio::new(0, 1)
    }
}
