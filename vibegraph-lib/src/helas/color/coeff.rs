//! Exact scalar coefficient of a color string.
//!
//! A [`ColorCoeff`] is `q · i^imag · Nc^nc_power`, mirroring MadGraph's
//! `(fractions.Fraction, is_imaginary, Nc_power)` triple. All arithmetic is
//! exact rational over `i64`; every operation is *checked* and panics on
//! overflow. Tree-level SU(3) factors are tiny, so an overflow signals a bug
//! rather than a legitimately large number — the panic is a deliberate
//! tripwire. `Ratio<i128>` behind the `GroupScalar` boundary is the escape
//! hatch if that ever changes.

use num_rational::Ratio;

/// Greatest common divisor of two `i64` magnitudes.
fn gcd_i64(mut a: i64, mut b: i64) -> i64 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Checked rational multiply: cross-reduces before multiplying to avoid
/// spurious overflow, then panics if the genuine product does not fit `i64`.
fn checked_mul_ratio(a: Ratio<i64>, b: Ratio<i64>) -> Ratio<i64> {
    let (an, ad) = (*a.numer(), *a.denom());
    let (bn, bd) = (*b.numer(), *b.denom());
    let g1 = gcd_i64(an, bd).max(1);
    let g2 = gcd_i64(bn, ad).max(1);
    let num = (an / g1)
        .checked_mul(bn / g2)
        .expect("ColorCoeff multiply: i64 numerator overflow");
    let den = (ad / g2)
        .checked_mul(bd / g1)
        .expect("ColorCoeff multiply: i64 denominator overflow");
    Ratio::new(num, den)
}

/// Checked rational add over a common denominator; panics on overflow.
fn checked_add_ratio(a: Ratio<i64>, b: Ratio<i64>) -> Ratio<i64> {
    let (an, ad) = (*a.numer(), *a.denom());
    let (bn, bd) = (*b.numer(), *b.denom());
    let g = gcd_i64(ad, bd).max(1);
    let lcm = (ad / g)
        .checked_mul(bd)
        .expect("ColorCoeff add: i64 denominator overflow");
    let num = an
        .checked_mul(lcm / ad)
        .and_then(|x| x.checked_add(bn.checked_mul(lcm / bd).expect("ColorCoeff add: overflow")))
        .expect("ColorCoeff add: i64 numerator overflow");
    Ratio::new(num, lcm)
}

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
    pub q: Ratio<i64>,
    /// Whether the coefficient carries one factor of `i`.
    pub imag: bool,
    /// Power of the symbolic color count `Nc`.
    pub nc_power: i32,
}

impl ColorCoeff {
    /// The multiplicative identity `1`.
    pub fn one() -> Self {
        ColorCoeff {
            q: Ratio::from_integer(1),
            imag: false,
            nc_power: 0,
        }
    }

    /// The additive identity `0`.
    pub fn zero() -> Self {
        ColorCoeff {
            q: Ratio::from_integer(0),
            imag: false,
            nc_power: 0,
        }
    }

    /// A real rational coefficient `n/d` with no `i` and `Nc^0`.
    pub fn rational(n: i64, d: i64) -> Self {
        ColorCoeff {
            q: Ratio::new(n, d),
            imag: false,
            nc_power: 0,
        }
    }

    /// Whether the rational magnitude is zero.
    pub fn is_zero(&self) -> bool {
        *self.q.numer() == 0
    }

    /// Product of two coefficients, following complex algebra on the `i` flag:
    /// `i·i = −1` flips the sign and clears the flag; a single `i` sets it.
    pub fn mul(&self, other: &ColorCoeff) -> ColorCoeff {
        let mut q = checked_mul_ratio(self.q, other.q);
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
    pub fn can_add(&self, other: &ColorCoeff) -> bool {
        self.imag == other.imag && self.nc_power == other.nc_power
    }

    /// Sum of two addition-compatible coefficients.
    ///
    /// # Panics
    /// If the coefficients are not [`can_add`](ColorCoeff::can_add)-compatible.
    pub fn add(&self, other: &ColorCoeff) -> ColorCoeff {
        assert!(
            self.can_add(other),
            "ColorCoeff::add on incompatible coefficients"
        );
        ColorCoeff {
            q: checked_add_ratio(self.q, other.q),
            imag: self.imag,
            nc_power: self.nc_power,
        }
    }

    /// Complex conjugate: negates the magnitude iff the coefficient is
    /// imaginary; `Nc` power and the flag are unchanged.
    pub fn conj(&self) -> ColorCoeff {
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
            checked_mul_ratio(self.q, Ratio::from_integer(p))
        } else {
            let p = nc
                .checked_pow((-self.nc_power) as u32)
                .expect("ColorCoeff::eval_nc: Nc power overflow");
            checked_mul_ratio(self.q, Ratio::new(1, p))
        }
    }
}
