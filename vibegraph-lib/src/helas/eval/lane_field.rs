//! The SIMD lane field: `N` phase-space points' worth of `f64` in one scalar.
//!
//! [`LaneField<N>`] wraps a `wide` vector (`f64x2`, `f64x4`, `f64x8`) and
//! implements `num_traits::Float`, so the evaluator's `F: Real` code runs on it
//! unchanged. The arithmetic the amplitude spends its time in — `+ − × ÷`,
//! negation, `sqrt`, `abs` and the fused `mul_add` — maps to a single packed
//! instruction on the target's widest enabled vector unit, which is what lets it
//! inline into the evaluator's dispatch loop. Every other `Float` method runs
//! per lane through `f64`'s own method.
//!
//! Each lane is bit-identical to the scalar `f64` computation: the packed
//! operations are the IEEE-exact ones (correctly rounded `+ − × ÷ sqrt`, sign-bit
//! `abs`/`neg`, single-rounding FMA), and everything else is literally the `f64`
//! method. `wide`'s own `mul_add` rounds twice when the target has no FMA unit,
//! while `f64::mul_add` always fuses, so the packed FMA is used only where it is
//! a hardware FMA and the per-lane `f64::mul_add` otherwise.
//!
//! Comparisons keep the pack-level semantics the lane-uniformity contract in
//! [`super::lanes`] is written against: `==` holds when every lane is equal, and
//! `<`/`>`/`partial_cmp` compare lane by lane lexicographically. The
//! `Float` predicates reduce over the pack: `is_nan`, `is_infinite` and
//! `is_sign_negative` hold if any lane does; `is_finite`, `is_normal` and
//! `is_sign_positive` only if every lane does. A pack converts to a primitive
//! (`ToPrimitive`, `integer_decode`) through lane 0.

use std::num::FpCategory;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use num_traits::{Float, FloatConst, Num, NumCast, One, ToPrimitive, Zero};

/// Lane-width marker: `Lanes<N>: SupportedLanes<N>` holds for every width with
/// a packed representation.
pub struct Lanes<const N: usize>;

/// The packed `f64` vector backing an `N`-lane [`LaneField`].
pub trait SupportedLanes<const N: usize> {
    type Pack: LanePack<N>;
}

/// The operations [`LaneField`] needs from its packed vector. Implemented for
/// `wide`'s `f64` vectors; each method is one packed instruction on a target
/// that enables the matching vector unit.
pub trait LanePack<const N: usize>:
    Copy
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + Send
    + Sync
    + 'static
{
    fn splat(x: f64) -> Self;
    fn from_array(lanes: [f64; N]) -> Self;
    fn to_array(self) -> [f64; N];
    fn sqrt(self) -> Self;
    fn abs(self) -> Self;
    /// `self * a + b` with a single rounding.
    fn fused_mul_add(self, a: Self, b: Self) -> Self;
}

macro_rules! impl_lane_pack {
    ($pack:ty, $n:literal) => {
        impl LanePack<$n> for $pack {
            #[inline(always)]
            fn splat(x: f64) -> Self {
                <$pack>::splat(x)
            }
            #[inline(always)]
            fn from_array(lanes: [f64; $n]) -> Self {
                <$pack>::new(lanes)
            }
            #[inline(always)]
            fn to_array(self) -> [f64; $n] {
                <$pack>::to_array(self)
            }
            #[inline(always)]
            fn sqrt(self) -> Self {
                <$pack>::sqrt(self)
            }
            #[inline(always)]
            fn abs(self) -> Self {
                <$pack>::abs(self)
            }
            #[inline(always)]
            fn fused_mul_add(self, a: Self, b: Self) -> Self {
                // `wide` fuses exactly when the target has a hardware FMA; without
                // one it rounds the product separately, so fall back to the
                // always-fused `f64::mul_add` per lane.
                if cfg!(any(
                    target_feature = "fma",
                    all(target_arch = "aarch64", target_feature = "neon")
                )) {
                    <$pack>::mul_add(self, a, b)
                } else {
                    let (x, a, b) = (self.to_array(), a.to_array(), b.to_array());
                    <$pack>::new(std::array::from_fn(|k| x[k].mul_add(a[k], b[k])))
                }
            }
        }

        impl SupportedLanes<$n> for Lanes<$n> {
            type Pack = $pack;
        }
    };
}

impl_lane_pack!(wide::f64x2, 2);
impl_lane_pack!(wide::f64x4, 4);
impl_lane_pack!(wide::f64x8, 8);

/// The scalar field for an `N`-wide lane pack: `N` phase-space points fed
/// through one `eval_m2` pass.
pub struct LaneField<const N: usize>(<Lanes<N> as SupportedLanes<N>>::Pack)
where
    Lanes<N>: SupportedLanes<N>;

impl<const N: usize> LaneField<N>
where
    Lanes<N>: SupportedLanes<N>,
{
    /// Every lane set to `x`.
    #[inline(always)]
    pub fn splat(x: f64) -> Self {
        Self(LanePack::splat(x))
    }

    /// One value per lane.
    #[inline(always)]
    pub fn from_array(lanes: [f64; N]) -> Self {
        Self(LanePack::from_array(lanes))
    }

    /// Every lane as a plain array.
    #[inline(always)]
    pub fn to_array(self) -> [f64; N] {
        self.0.to_array()
    }

    #[inline(always)]
    fn map(self, f: impl Fn(f64) -> f64) -> Self {
        let x = self.to_array();
        Self::from_array(std::array::from_fn(|k| f(x[k])))
    }

    #[inline(always)]
    fn zip(self, rhs: Self, f: impl Fn(f64, f64) -> f64) -> Self {
        let (x, y) = (self.to_array(), rhs.to_array());
        Self::from_array(std::array::from_fn(|k| f(x[k], y[k])))
    }
}

impl<const N: usize> Clone for LaneField<N>
where
    Lanes<N>: SupportedLanes<N>,
{
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<const N: usize> Copy for LaneField<N> where Lanes<N>: SupportedLanes<N> {}

impl<const N: usize> std::fmt::Debug for LaneField<N>
where
    Lanes<N>: SupportedLanes<N>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("LaneField").field(&self.to_array()).finish()
    }
}

impl<const N: usize> PartialEq for LaneField<N>
where
    Lanes<N>: SupportedLanes<N>,
{
    #[inline]
    fn eq(&self, rhs: &Self) -> bool {
        self.to_array() == rhs.to_array()
    }
}

impl<const N: usize> PartialOrd for LaneField<N>
where
    Lanes<N>: SupportedLanes<N>,
{
    #[inline]
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        self.to_array().partial_cmp(&rhs.to_array())
    }
}

macro_rules! impl_packed_binop {
    ($($op:ident::$f:ident),*) => {$(
        impl<const N: usize> $op for LaneField<N>
        where
            Lanes<N>: SupportedLanes<N>,
        {
            type Output = Self;
            #[inline(always)]
            fn $f(self, rhs: Self) -> Self {
                Self($op::$f(self.0, rhs.0))
            }
        }
    )*};
}

impl_packed_binop!(Add::add, Sub::sub, Mul::mul, Div::div);

impl<const N: usize> Rem for LaneField<N>
where
    Lanes<N>: SupportedLanes<N>,
{
    type Output = Self;
    #[inline]
    fn rem(self, rhs: Self) -> Self {
        self.zip(rhs, |x, y| x % y)
    }
}

impl<const N: usize> Neg for LaneField<N>
where
    Lanes<N>: SupportedLanes<N>,
{
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl<const N: usize> Zero for LaneField<N>
where
    Lanes<N>: SupportedLanes<N>,
{
    #[inline(always)]
    fn zero() -> Self {
        Self::splat(0.0)
    }
    #[inline]
    fn is_zero(&self) -> bool {
        self.to_array().iter().all(|x| x.is_zero())
    }
}

impl<const N: usize> One for LaneField<N>
where
    Lanes<N>: SupportedLanes<N>,
{
    #[inline(always)]
    fn one() -> Self {
        Self::splat(1.0)
    }
}

impl<const N: usize> Num for LaneField<N>
where
    Lanes<N>: SupportedLanes<N>,
{
    type FromStrRadixErr = <f64 as Num>::FromStrRadixErr;
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        f64::from_str_radix(s, radix).map(Self::splat)
    }
}

macro_rules! impl_to_primitive {
    ($($to:ident => $prim:ty),*) => {
        impl<const N: usize> ToPrimitive for LaneField<N>
        where
            Lanes<N>: SupportedLanes<N>,
        {
            $(
                #[inline]
                fn $to(&self) -> Option<$prim> {
                    self.to_array().first().and_then(ToPrimitive::$to)
                }
            )*
        }
    };
}

impl_to_primitive!(
    to_i8 => i8, to_i16 => i16, to_i32 => i32, to_i64 => i64, to_i128 => i128,
    to_isize => isize, to_u8 => u8, to_u16 => u16, to_u32 => u32, to_u64 => u64,
    to_u128 => u128, to_usize => usize, to_f32 => f32, to_f64 => f64
);

impl<const N: usize> NumCast for LaneField<N>
where
    Lanes<N>: SupportedLanes<N>,
{
    #[inline]
    fn from<P: ToPrimitive>(n: P) -> Option<Self> {
        <f64 as NumCast>::from(n).map(Self::splat)
    }
}

macro_rules! lanewise_unary {
    ($($f:ident),*) => {$(
        #[inline]
        fn $f(self) -> Self {
            self.map(f64::$f)
        }
    )*};
}

macro_rules! lanewise_binary {
    ($($f:ident),*) => {$(
        #[inline]
        fn $f(self, other: Self) -> Self {
            self.zip(other, f64::$f)
        }
    )*};
}

macro_rules! splat_const {
    ($($f:ident => $v:expr),*) => {$(
        #[inline(always)]
        fn $f() -> Self {
            Self::splat($v)
        }
    )*};
}

impl<const N: usize> Float for LaneField<N>
where
    Lanes<N>: SupportedLanes<N>,
{
    splat_const!(
        nan => f64::NAN,
        infinity => f64::INFINITY,
        neg_infinity => f64::NEG_INFINITY,
        neg_zero => -0.0,
        min_value => f64::MIN,
        min_positive_value => f64::MIN_POSITIVE,
        max_value => f64::MAX,
        epsilon => f64::EPSILON
    );

    #[inline]
    fn is_nan(self) -> bool {
        self.to_array().iter().any(|x| x.is_nan())
    }
    #[inline]
    fn is_infinite(self) -> bool {
        self.to_array().iter().any(|x| x.is_infinite())
    }
    #[inline]
    fn is_finite(self) -> bool {
        self.to_array().iter().all(|x| x.is_finite())
    }
    #[inline]
    fn is_normal(self) -> bool {
        self.to_array().iter().all(|x| x.is_normal())
    }
    #[inline]
    fn is_sign_positive(self) -> bool {
        self.to_array().iter().all(|x| x.is_sign_positive())
    }
    #[inline]
    fn is_sign_negative(self) -> bool {
        self.to_array().iter().any(|x| x.is_sign_negative())
    }

    /// NaN if any lane is; otherwise the most general category present, ordered
    /// Infinite > Subnormal > Normal > Zero.
    fn classify(self) -> FpCategory {
        let mut ret = FpCategory::Zero;
        for x in self.to_array() {
            match x.classify() {
                FpCategory::Nan => return FpCategory::Nan,
                FpCategory::Infinite => ret = FpCategory::Infinite,
                FpCategory::Subnormal if ret != FpCategory::Infinite => {
                    ret = FpCategory::Subnormal;
                }
                FpCategory::Normal if ret == FpCategory::Zero => ret = FpCategory::Normal,
                _ => {}
            }
        }
        ret
    }

    #[inline]
    fn integer_decode(self) -> (u64, i16, i8) {
        Float::integer_decode(self.to_array()[0])
    }

    #[inline(always)]
    fn sqrt(self) -> Self {
        Self(self.0.sqrt())
    }
    #[inline(always)]
    fn abs(self) -> Self {
        Self(self.0.abs())
    }
    #[inline(always)]
    fn mul_add(self, a: Self, b: Self) -> Self {
        Self(self.0.fused_mul_add(a.0, b.0))
    }

    lanewise_unary!(
        floor, ceil, round, trunc, fract, signum, recip, exp, exp2, ln, log2, log10, cbrt, sin,
        cos, tan, asin, acos, atan, exp_m1, ln_1p, sinh, cosh, tanh, asinh, acosh, atanh,
        to_degrees, to_radians
    );
    lanewise_binary!(powf, log, max, min, hypot, atan2);

    #[inline]
    fn abs_sub(self, other: Self) -> Self {
        self.zip(other, Float::abs_sub)
    }
    #[inline]
    fn powi(self, n: i32) -> Self {
        self.map(|x| x.powi(n))
    }
    #[inline]
    fn sin_cos(self) -> (Self, Self) {
        (self.map(f64::sin), self.map(f64::cos))
    }
}

impl<const N: usize> FloatConst for LaneField<N>
where
    Lanes<N>: SupportedLanes<N>,
{
    splat_const!(
        E => std::f64::consts::E,
        FRAC_1_PI => std::f64::consts::FRAC_1_PI,
        FRAC_1_SQRT_2 => std::f64::consts::FRAC_1_SQRT_2,
        FRAC_2_PI => std::f64::consts::FRAC_2_PI,
        FRAC_2_SQRT_PI => std::f64::consts::FRAC_2_SQRT_PI,
        FRAC_PI_2 => std::f64::consts::FRAC_PI_2,
        FRAC_PI_3 => std::f64::consts::FRAC_PI_3,
        FRAC_PI_4 => std::f64::consts::FRAC_PI_4,
        FRAC_PI_6 => std::f64::consts::FRAC_PI_6,
        FRAC_PI_8 => std::f64::consts::FRAC_PI_8,
        LN_10 => std::f64::consts::LN_10,
        LN_2 => std::f64::consts::LN_2,
        LOG10_E => std::f64::consts::LOG10_E,
        LOG2_E => std::f64::consts::LOG2_E,
        PI => std::f64::consts::PI,
        SQRT_2 => std::f64::consts::SQRT_2,
        TAU => std::f64::consts::TAU,
        LOG10_2 => std::f64::consts::LOG10_2,
        LOG2_10 => std::f64::consts::LOG2_10
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every packed operation against the scalar `f64` operation, bit for bit,
    /// on values chosen to exercise rounding (non-representable quotients, an
    /// FMA whose unfused product rounds differently) and signed zero.
    fn packed_ops_match_scalar<const N: usize>()
    where
        Lanes<N>: SupportedLanes<N>,
    {
        let xs: [f64; N] = std::array::from_fn(|k| 0.1 + 0.37 * k as f64 - 0.9 * (k % 2) as f64);
        let ys: [f64; N] = std::array::from_fn(|k| 3.0 - 1.1 * k as f64);
        let zs: [f64; N] = std::array::from_fn(|k| -1e-17 * (k + 1) as f64);
        let (x, y, z) = (
            LaneField::<N>::from_array(xs),
            LaneField::<N>::from_array(ys),
            LaneField::<N>::from_array(zs),
        );
        let check = |got: LaneField<N>, want: &dyn Fn(usize) -> f64, what: &str| {
            for (k, g) in got.to_array().into_iter().enumerate() {
                assert_eq!(g.to_bits(), want(k).to_bits(), "{what} lane {k} (N = {N})");
            }
        };
        check(x + y, &|k| xs[k] + ys[k], "add");
        check(x - y, &|k| xs[k] - ys[k], "sub");
        check(x * y, &|k| xs[k] * ys[k], "mul");
        check(x / y, &|k| xs[k] / ys[k], "div");
        check(-x, &|k| -xs[k], "neg");
        check(x.abs().sqrt(), &|k| xs[k].abs().sqrt(), "sqrt");
        check(x.mul_add(y, z), &|k| xs[k].mul_add(ys[k], zs[k]), "mul_add");
        check(x.atan2(y), &|k| xs[k].atan2(ys[k]), "atan2");
    }

    #[test]
    fn packed_ops_bit_identical_to_scalar() {
        packed_ops_match_scalar::<2>();
        packed_ops_match_scalar::<4>();
        packed_ops_match_scalar::<8>();
    }

    /// The `mul_add` probe above only discriminates if some lane's unfused
    /// `x*y + z` rounds differently from the fused one.
    #[test]
    fn mul_add_probe_separates_fused_from_unfused() {
        let (x, y, z) = (0.1_f64, 3.0_f64, -1e-17_f64);
        assert_ne!((x * y + z).to_bits(), x.mul_add(y, z).to_bits());
    }

    #[test]
    fn comparisons_reduce_over_the_pack() {
        let a = LaneField::<4>::from_array([1.0, 2.0, 3.0, 4.0]);
        let b = LaneField::<4>::from_array([1.0, 2.0, 5.0, 0.0]);
        assert!(a < b, "lexicographic: the first differing lane decides");
        assert!(a != b && a == a);
        assert!(!LaneField::<4>::from_array([1.0, -0.0, 1.0, 1.0]).is_sign_positive());
        assert!(LaneField::<4>::from_array([1.0, f64::NAN, 1.0, 1.0]).is_nan());
    }
}
