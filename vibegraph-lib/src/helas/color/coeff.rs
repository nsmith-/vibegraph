//! Exact scalar coefficient of a color string.
//!
//! A [`ColorCoeff`] is `q · i^imag · Nc^nc_power`, mirroring MadGraph's
//! `(fractions.Fraction, is_imaginary, Nc_power)` triple. All arithmetic is
//! exact rational over `i64`; every operation goes through num-rational's
//! checked arithmetic and panics on overflow. Tree-level SU(3) factors are
//! tiny, so an overflow signals a bug rather than a legitimately large
//! number — the panic is a deliberate tripwire.

use num_rational::Ratio;
use num_traits::{CheckedAdd, CheckedMul};

/// The exact scalar prefactor of a color string: `q · i^imag · Nc^nc_power`.
///
/// The three pieces are kept separate exactly as MadGraph does: `q` is a
/// rational, `imag` flags a single factor of the imaginary unit, and
/// `nc_power` records the power of the (still symbolic) number of colors `Nc`.
/// Two coefficients may only be *added* when their `imag` flag and `nc_power`
/// agree (see [`ColorCoeff::can_add`]); multiplication combines all three.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorCoeff {
    /// Rational magnitude.
    pub(crate) q: Ratio<i64>,
    /// Whether the coefficient carries one factor of `i`.
    pub imag: bool,
    /// Power of the symbolic color count `Nc`.
    pub(crate) nc_power: i32,
}

impl ColorCoeff {
    /// The multiplicative identity `1`.
    pub(crate) fn one() -> Self {
        ColorCoeff {
            q: Ratio::from_integer(1),
            imag: false,
            nc_power: 0,
        }
    }

    /// The additive identity `0`.
    pub(crate) fn zero() -> Self {
        ColorCoeff {
            q: Ratio::from_integer(0),
            imag: false,
            nc_power: 0,
        }
    }

    /// A real rational coefficient `n/d` with no `i` and `Nc^0`.
    pub(crate) fn rational(n: i64, d: i64) -> Self {
        ColorCoeff {
            q: Ratio::new(n, d),
            imag: false,
            nc_power: 0,
        }
    }

    /// Whether the rational magnitude is zero.
    pub(crate) fn is_zero(&self) -> bool {
        *self.q.numer() == 0
    }

    /// Product of two coefficients, following complex algebra on the `i` flag:
    /// `i·i = −1` flips the sign and clears the flag; a single `i` sets it.
    pub(crate) fn mul(&self, other: &ColorCoeff) -> ColorCoeff {
        let mut q = self
            .q
            .checked_mul(&other.q)
            .expect("ColorCoeff multiply: i64 overflow");
        let nc_power = self
            .nc_power
            .checked_add(other.nc_power)
            .expect("ColorCoeff multiply: Nc power overflow");
        let imag = if self.imag && other.imag {
            q = -q;
            false
        } else {
            self.imag || other.imag
        };
        ColorCoeff { q, imag, nc_power }
    }

    /// Whether two coefficients are addition-compatible: same `i` flag and
    /// same `Nc` power (the color-tensor structure is compared separately, at
    /// the string level).
    pub(crate) fn can_add(&self, other: &ColorCoeff) -> bool {
        self.imag == other.imag && self.nc_power == other.nc_power
    }

    /// Sum of two addition-compatible coefficients.
    ///
    /// # Panics
    /// If the coefficients are not [`can_add`](ColorCoeff::can_add)-compatible.
    pub(crate) fn add(&self, other: &ColorCoeff) -> ColorCoeff {
        assert!(
            self.can_add(other),
            "ColorCoeff::add on incompatible coefficients"
        );
        ColorCoeff {
            q: self
                .q
                .checked_add(&other.q)
                .expect("ColorCoeff add: i64 overflow"),
            imag: self.imag,
            nc_power: self.nc_power,
        }
    }

    /// Complex conjugate: negates the magnitude iff the coefficient is
    /// imaginary; `Nc` power and the flag are unchanged.
    pub(crate) fn conj(&self) -> ColorCoeff {
        ColorCoeff {
            q: if self.imag { -self.q } else { self.q },
            imag: self.imag,
            nc_power: self.nc_power,
        }
    }

    /// Evaluate the `Nc` power at a concrete number of colors, returning the
    /// exact rational `q · nc^nc_power`. The `imag` flag is left to the caller.
    pub fn eval_nc(&self, nc: i64) -> Ratio<i64> {
        if self.nc_power >= 0 {
            let p = nc
                .checked_pow(self.nc_power as u32)
                .expect("ColorCoeff::eval_nc: Nc power overflow");
            self.q
                .checked_mul(&Ratio::from_integer(p))
                .expect("ColorCoeff::eval_nc: i64 overflow")
        } else {
            let p = nc
                .checked_pow((-self.nc_power) as u32)
                .expect("ColorCoeff::eval_nc: Nc power overflow");
            self.q
                .checked_mul(&Ratio::new(1, p))
                .expect("ColorCoeff::eval_nc: i64 overflow")
        }
    }
}
