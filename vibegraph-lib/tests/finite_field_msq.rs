//! Functional reconstruction of the helicity-summed squared amplitude over a prime
//! field: how large is `Σ_hel A_i A_j*`, diagram pair by diagram pair, once it is
//! written as a polynomial in the Lorentz invariants?
//!
//! # The black box
//!
//! The evaluator is generic over its scalar field, so it runs unchanged over
//! [`Fz`]: an element of `Z_p` (with `i` adjoined through `num_complex`, which for
//! `p ≡ 3 (mod 4)` is the field `F_{p²}`, complex conjugation being its Frobenius)
//! carried together with an `f64` shadow of the same computation. Arithmetic is
//! exact mod `p`; the shadow decides every comparison the wavefunction routines
//! branch on (`min`, `max`, `== 0`), so the field computation follows the path the
//! real one takes.
//!
//! The square roots the external wavefunctions take are not rational, but the
//! helicity sum does not see them: `Σ_h u_h ū_h = p̸ + m`, and its vector
//! analogues, are polynomial in the momentum. Any choice of roots that keeps each
//! leg's completeness relation intact gives the reduction mod `p` of the exact
//! rational `Σ_hel A_i A_j*`. The root taken is `x^((p+1)/4)`, the unique root that
//! is itself a square, which makes it multiplicative (`√a √b = √(ab)`); with
//! `p ≡ 7 (mod 8)` the relations between the roots `weyl_ixxxxx` takes separately
//! then hold. That is a claim, and [`fz_wavefunctions_satisfy_completeness`] pins
//! it leg by leg. A point whose roots do not exist in `Z_p` is rejected.
//!
//! # The reconstruction
//!
//! For diagram pair `(i, j)` the denominator is known: the product `D_i D_j*` of the
//! two diagrams' propagators, `q² − M² + i M Γ` each (no width on a t-channel line,
//! as the evaluator lowers it). Multiplied out, `N_ij = D_i D_j* Σ_hel A_i A_j*` is a
//! polynomial in the independent dot products of the external momenta and, from five
//! legs on, linear in the Levi-Civita contractions `ε(p_a, p_b, p_c, p_d)`. A dense
//! ansatz of total degree `d` is fitted by exact row reduction mod `p` over random
//! rational phase-space points in random frames; `d` grows until held-out points
//! agree, which also checks the denominator and the Lorentz invariance of the box.
//! The count of non-zero coefficients is the size of the per-pair trace form.
//!
//! An external massless vector breaks the per-pair Lorentz invariance: HELAS sums
//! its polarizations to `−g + (p n + n p)/(p·n)` with the frame vector
//! `n = (p⁰, −p⃗)`, and the gauge terms cancel only in the sum over diagrams. Such
//! processes are sampled in the partonic centre-of-mass frame, where `n` is
//! rational in `p` and `P = p₁ + p₂`, at the price of one more known denominator
//! `(p·P)²` per vector leg. Their counts are those of that axial-type gauge, not of
//! the Feynman-gauge `−g` a symbolic trace program would use.
//!
//! What this does not see: term counts are basis-dependent (they are counted in
//! the dot-product basis below, without partial fractions or factorisation), and
//! a coefficient divisible by `p` would read as zero (probability ~`1/p`;
//! two primes must agree).
//!
//! `measure_trace_form` (ignored; run it with `--release -- --ignored --nocapture`)
//! prints the census and times the per-pair form against `eval_m2`.

mod common;

use std::cell::Cell;
use std::cmp::Ordering;
use std::num::FpCategory;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};
use std::time::Instant;

use num_bigint::BigInt;
use num_modular::{ModularCoreOps, ModularPow, ModularUnaryOps};
use num_rational::BigRational;
use num_traits::{Float, FloatConst, FromPrimitive, Num, NumCast, One, Signed, ToPrimitive, Zero};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

use vibegraph::diagrams::DiagramSet;
use vibegraph::helas::eval::{config_groups, AmplitudeEvaluator, BoundAmplitude};
use vibegraph::helas::repr::lorentz::{ComplexVector, Ket, SpinorRepr, VectorRepr};
use vibegraph::helas::repr::numbers::{Charge, SpinorHelicity};
use vibegraph::helas::repr::C;
use vibegraph::helas::wavefn::VectorWf;
use vibegraph::helas::{Bispinor, LorentzVector};
use vibegraph::ufo::{EvaluatedModel, UFOModel};

// ── the field ────────────────────────────────────────────────────────────────

thread_local! {
    static MODULUS: Cell<u64> = const { Cell::new(0) };
    static POISONED: Cell<bool> = const { Cell::new(false) };
}

fn modulus() -> u64 {
    let p = MODULUS.with(Cell::get);
    debug_assert!(p != 0, "no modulus set on this thread");
    p
}

fn set_modulus(p: u64) {
    assert_eq!(p % 8, 7, "the root convention needs p ≡ 7 (mod 8)");
    MODULUS.with(|m| m.set(p));
}

/// Mark the current evaluation as not a faithful image of the exact one.
fn poison() {
    POISONED.with(|c| c.set(true));
}

/// Clear the poison flag, returning whether it was set.
fn take_poison() -> bool {
    POISONED.with(|c| c.replace(false))
}

/// The primes the measurement runs over: the largest below `2^62` with
/// `p ≡ 7 (mod 8)`, so that sums of two residues do not overflow.
fn primes(count: usize) -> Vec<u64> {
    let mut out = Vec::with_capacity(count);
    let mut candidate = (1u64 << 62) - 1;
    while out.len() < count {
        if candidate % 8 == 7 && num_prime::nt_funcs::is_prime64(candidate) {
            out.push(candidate);
        }
        candidate -= 2;
    }
    out
}

fn reduce_i64(i: i64) -> u64 {
    (i as i128).rem_euclid(modulus() as i128) as u64
}

/// The exact image of an `f64`: every finite double is a dyadic rational.
fn reduce_f64(x: f64) -> u64 {
    if !x.is_finite() {
        poison();
        return 0;
    }
    if x == 0.0 {
        return 0;
    }
    let p = modulus();
    let (mantissa, exponent, sign) = Float::integer_decode(x);
    let scale = 2u64.powm(exponent.unsigned_abs() as u64, &p);
    let scale = if exponent >= 0 {
        scale
    } else {
        scale.invm(&p).expect("2 is invertible mod an odd prime")
    };
    let r = (mantissa % p).mulm(scale, &p);
    if sign < 0 {
        r.negm(&p)
    } else {
        r
    }
}

fn reduce_big(x: &BigInt) -> u64 {
    let p = BigInt::from(modulus());
    let r = ((x % &p) + &p) % &p;
    r.to_u64().expect("residue fits u64")
}

fn reduce_rational(x: &BigRational) -> u64 {
    let p = modulus();
    let d = reduce_big(x.denom());
    match d.invm(&p) {
        Some(inv) => reduce_big(x.numer()).mulm(inv, &p),
        None => {
            poison();
            0
        }
    }
}

/// A residue mod [`modulus`] with an `f64` shadow of the same computation.
#[derive(Clone, Copy, Debug)]
struct Fz {
    f: f64,
    m: u64,
}

impl Fz {
    fn from_rational(x: &BigRational) -> Self {
        Fz {
            f: x.to_f64().expect("finite rational"),
            m: reduce_rational(x),
        }
    }

    fn exact(f: f64) -> Self {
        Fz {
            f,
            m: reduce_f64(f),
        }
    }

    /// A value with no exact image (a transcendental function): poisons.
    fn inexact(f: f64) -> Self {
        poison();
        Fz { f, m: 0 }
    }
}

impl PartialEq for Fz {
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f
    }
}

impl PartialOrd for Fz {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.f.partial_cmp(&other.f)
    }
}

impl Add for Fz {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Fz {
            f: self.f + rhs.f,
            m: self.m.addm(rhs.m, &modulus()),
        }
    }
}

impl Sub for Fz {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Fz {
            f: self.f - rhs.f,
            m: self.m.subm(rhs.m, &modulus()),
        }
    }
}

impl Mul for Fz {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Fz {
            f: self.f * rhs.f,
            m: self.m.mulm(rhs.m, &modulus()),
        }
    }
}

impl Div for Fz {
    type Output = Self;
    #[inline]
    #[allow(clippy::suspicious_arithmetic_impl)] // division is multiplication by the inverse
    fn div(self, rhs: Self) -> Self {
        self * rhs.recip()
    }
}

impl Rem for Fz {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self {
        Fz::inexact(self.f % rhs.f)
    }
}

impl Neg for Fz {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Fz {
            f: -self.f,
            m: self.m.negm(&modulus()),
        }
    }
}

impl Zero for Fz {
    fn zero() -> Self {
        Fz { f: 0.0, m: 0 }
    }
    fn is_zero(&self) -> bool {
        self.f == 0.0
    }
}

impl One for Fz {
    fn one() -> Self {
        Fz { f: 1.0, m: 1 }
    }
}

impl Num for Fz {
    type FromStrRadixErr = <f64 as Num>::FromStrRadixErr;
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        f64::from_str_radix(s, radix).map(Fz::exact)
    }
}

impl ToPrimitive for Fz {
    fn to_i64(&self) -> Option<i64> {
        self.f.to_i64()
    }
    fn to_u64(&self) -> Option<u64> {
        self.f.to_u64()
    }
    fn to_f64(&self) -> Option<f64> {
        Some(self.f)
    }
}

impl NumCast for Fz {
    fn from<T: ToPrimitive>(n: T) -> Option<Self> {
        n.to_f64().map(Fz::exact)
    }
}

impl FromPrimitive for Fz {
    fn from_i64(n: i64) -> Option<Self> {
        Some(Fz {
            f: n as f64,
            m: reduce_i64(n),
        })
    }
    fn from_u64(n: u64) -> Option<Self> {
        Some(Fz {
            f: n as f64,
            m: n % modulus(),
        })
    }
    fn from_f64(n: f64) -> Option<Self> {
        Some(Fz::exact(n))
    }
}

macro_rules! inexact_unary {
    ($($f:ident),*) => {$(
        fn $f(self) -> Self {
            Fz::inexact(self.f.$f())
        }
    )*};
}

macro_rules! shadow_const {
    ($($f:ident => $v:expr),*) => {$(
        fn $f() -> Self {
            Fz { f: $v, m: 0 }
        }
    )*};
}

impl Float for Fz {
    shadow_const!(
        nan => f64::NAN,
        infinity => f64::INFINITY,
        neg_infinity => f64::NEG_INFINITY,
        neg_zero => -0.0
    );

    fn min_value() -> Self {
        Fz::exact(f64::MIN)
    }
    fn min_positive_value() -> Self {
        Fz::exact(f64::MIN_POSITIVE)
    }
    fn max_value() -> Self {
        Fz::exact(f64::MAX)
    }
    fn epsilon() -> Self {
        Fz::exact(f64::EPSILON)
    }

    fn is_nan(self) -> bool {
        self.f.is_nan()
    }
    fn is_infinite(self) -> bool {
        self.f.is_infinite()
    }
    fn is_finite(self) -> bool {
        self.f.is_finite()
    }
    fn is_normal(self) -> bool {
        self.f.is_normal()
    }
    fn classify(self) -> FpCategory {
        self.f.classify()
    }
    fn is_sign_positive(self) -> bool {
        self.f.is_sign_positive()
    }
    fn is_sign_negative(self) -> bool {
        self.f.is_sign_negative()
    }
    fn integer_decode(self) -> (u64, i16, i8) {
        Float::integer_decode(self.f)
    }

    fn abs(self) -> Self {
        if self.f < 0.0 {
            -self
        } else {
            self
        }
    }
    fn signum(self) -> Self {
        if self.f < 0.0 {
            -Fz::one()
        } else {
            Fz::one()
        }
    }
    fn max(self, other: Self) -> Self {
        if other.f > self.f {
            other
        } else {
            self
        }
    }
    fn min(self, other: Self) -> Self {
        if other.f < self.f {
            other
        } else {
            self
        }
    }
    fn abs_sub(self, other: Self) -> Self {
        if self.f > other.f {
            self - other
        } else {
            Fz::zero()
        }
    }

    fn mul_add(self, a: Self, b: Self) -> Self {
        self * a + b
    }
    fn recip(self) -> Self {
        let p = modulus();
        match self.m.invm(&p) {
            Some(inv) => Fz {
                f: self.f.recip(),
                m: inv,
            },
            None => Fz::inexact(self.f.recip()),
        }
    }
    fn powi(self, n: i32) -> Self {
        let base = if n < 0 { self.recip() } else { self };
        Fz {
            f: self.f.powi(n),
            m: base.m.powm(n.unsigned_abs() as u64, &modulus()),
        }
    }
    /// The root that is itself a square, `x^((p+1)/4)`: multiplicative, and with
    /// `p ≡ 7 (mod 8)` it sends `2` to a square too.
    fn sqrt(self) -> Self {
        let p = modulus();
        let r = self.m.powm((p + 1) / 4, &p);
        if !(self.f >= 0.0) || r.mulm(r, &p) != self.m {
            poison();
        }
        Fz {
            f: self.f.sqrt(),
            m: r,
        }
    }
    fn hypot(self, other: Self) -> Self {
        (self * self + other * other).sqrt()
    }

    inexact_unary!(
        floor, ceil, round, trunc, fract, exp, exp2, ln, log2, log10, cbrt, sin, cos, tan, asin,
        acos, atan, exp_m1, ln_1p, sinh, cosh, tanh, asinh, acosh, atanh
    );
    fn powf(self, n: Self) -> Self {
        Fz::inexact(self.f.powf(n.f))
    }
    fn log(self, base: Self) -> Self {
        Fz::inexact(self.f.log(base.f))
    }
    fn atan2(self, other: Self) -> Self {
        Fz::inexact(self.f.atan2(other.f))
    }
    fn sin_cos(self) -> (Self, Self) {
        (Fz::inexact(self.f.sin()), Fz::inexact(self.f.cos()))
    }
}

macro_rules! inexact_const {
    ($($f:ident => $v:expr),*) => {$(
        fn $f() -> Self {
            Fz::inexact($v)
        }
    )*};
}

impl FloatConst for Fz {
    inexact_const!(
        E => std::f64::consts::E,
        FRAC_1_PI => std::f64::consts::FRAC_1_PI,
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
        TAU => std::f64::consts::TAU,
        LOG10_2 => std::f64::consts::LOG10_2,
        LOG2_10 => std::f64::consts::LOG2_10
    );
    fn SQRT_2() -> Self {
        Fz::exact(2.0).sqrt()
    }
    fn FRAC_1_SQRT_2() -> Self {
        Fz::exact(0.5).sqrt()
    }
}

/// A complex residue as its two `Z_p` coordinates.
fn parts(z: C<Fz>) -> [u64; 2] {
    [z.re.m, z.im.m]
}

// ── kinematics ───────────────────────────────────────────────────────────────

type Q = BigRational;
type Mom = [Q; 4];

fn q(n: i64, d: i64) -> Q {
    Q::new(BigInt::from(n), BigInt::from(d))
}

fn small_rational(rng: &mut ChaCha8Rng, span: i64) -> Q {
    q(rng.random_range(-span..=span), rng.random_range(1..=span))
}

/// A rational unit 3-vector: the inverse stereographic image of `(u, v)`.
fn unit_vector(rng: &mut ChaCha8Rng) -> [Q; 3] {
    let u = small_rational(rng, 9);
    let v = small_rational(rng, 9);
    let r2 = &u * &u + &v * &v;
    let den = &r2 + Q::one();
    let two = q(2, 1);
    [&two * &u / &den, &two * &v / &den, (&r2 - Q::one()) / &den]
}

fn massless(energy: &Q, dir: &[Q; 3]) -> Mom {
    [
        energy.clone(),
        energy * &dir[0],
        energy * &dir[1],
        energy * &dir[2],
    ]
}

fn mdot(a: &Mom, b: &Mom) -> Q {
    &a[0] * &b[0] - &a[1] * &b[1] - &a[2] * &b[2] - &a[3] * &b[3]
}

fn madd(a: &Mom, b: &Mom) -> Mom {
    std::array::from_fn(|k| &a[k] + &b[k])
}

fn msub(a: &Mom, b: &Mom) -> Mom {
    std::array::from_fn(|k| &a[k] - &b[k])
}

fn mscale(a: &Mom, s: &Q) -> Mom {
    std::array::from_fn(|k| &a[k] * s)
}

/// Split a timelike `total` into two lightlike momenta, the first along `dir`.
fn split_massless(total: &Mom, dir: &[Q; 3]) -> (Mom, Mom) {
    let along = massless(&Q::one(), dir);
    let lambda = mdot(total, total) / (q(2, 1) * mdot(total, &along));
    let a = mscale(&along, &lambda);
    let b = msub(total, &a);
    (a, b)
}

/// A massive momentum of mass² `m2` built from two lightlike directions:
/// `k + m²/(2 k·n) n` is rational whenever `k`, `n` and `m²` are.
fn massive_from(k: &Mom, n: &Mom, m2: &Q) -> Mom {
    let c = m2 / (q(2, 1) * mdot(k, n));
    madd(k, &mscale(n, &c))
}

/// A rational 2 → n point: lightlike beams (back to back when `cm`, along random
/// directions otherwise), final-state masses squared as given (`None` for a
/// massless leg). Momentum conservation is closed by splitting what is left into
/// the last two massless final-state legs, so at least two must be massless.
///
/// `accept(leg, momentum)` vets each leg as it is drawn, and a refused leg is
/// redrawn on its own: the field box needs every leg's wavefunction roots to exist
/// mod `p`, which holds for about half the draws per root, and redrawing whole
/// points for it costs the product of those odds.
fn rational_point(
    rng: &mut ChaCha8Rng,
    out_masses: &[Option<Q>],
    cm: bool,
    accept: &dyn Fn(usize, &Mom) -> bool,
) -> Vec<Mom> {
    let massless_legs: Vec<usize> = (0..out_masses.len())
        .filter(|&k| out_masses[k].is_none())
        .collect();
    assert!(
        massless_legs.len() >= 2,
        "two massless final-state legs close the point"
    );
    let closing = [
        massless_legs[massless_legs.len() - 2],
        massless_legs[massless_legs.len() - 1],
    ];
    let n_out = out_masses.len();
    const TRIES: usize = 64;
    'point: loop {
        let (p1, p2) = loop {
            let e1 = q(rng.random_range(20..=60), rng.random_range(1..=3));
            let beam = unit_vector(rng);
            let p1 = massless(&e1, &beam);
            let p2 = if cm {
                massless(&e1, &[-&beam[0], -&beam[1], -&beam[2]])
            } else {
                let e2 = q(rng.random_range(20..=60), rng.random_range(1..=3));
                massless(&e2, &unit_vector(rng))
            };
            if accept(0, &p1) && accept(1, &p2) {
                break (p1, p2);
            }
        };
        let total = madd(&p1, &p2);
        let mut out: Vec<Option<Mom>> = vec![None; n_out];
        let mut rest = total.clone();
        for (k, mass) in out_masses.iter().enumerate() {
            if closing.contains(&k) {
                continue;
            }
            let mut drawn = None;
            for _ in 0..TRIES {
                let share = q(rng.random_range(1..=8), (4 * n_out) as i64);
                let energy = &rest[0] * &share;
                let along = massless(&energy, &unit_vector(rng));
                let p = match mass {
                    None => along,
                    Some(m2) => {
                        let n = massless(&(&energy * q(1, 4)), &unit_vector(rng));
                        if mdot(&along, &n).is_zero() {
                            continue;
                        }
                        massive_from(&along, &n, m2)
                    }
                };
                if accept(2 + k, &p) {
                    drawn = Some(p);
                    break;
                }
            }
            let Some(p) = drawn else { continue 'point };
            rest = msub(&rest, &p);
            out[k] = Some(p);
        }
        if rest[0] <= Q::zero() || mdot(&rest, &rest) <= Q::zero() {
            continue;
        }
        let mut closed = None;
        for _ in 0..TRIES {
            let (a, b) = split_massless(&rest, &unit_vector(rng));
            if a[0] > Q::zero()
                && b[0] > Q::zero()
                && accept(2 + closing[0], &a)
                && accept(2 + closing[1], &b)
            {
                closed = Some((a, b));
                break;
            }
        }
        let Some((a, b)) = closed else { continue };
        out[closing[0]] = Some(a);
        out[closing[1]] = Some(b);
        let mut all = vec![p1, p2];
        all.extend(out.into_iter().map(|p| p.expect("every leg drawn")));
        return all;
    }
}

/// Any point at all: the f64 side needs no roots.
fn any_leg(_: usize, _: &Mom) -> bool {
    true
}

fn to_fz_momentum(p: &Mom) -> LorentzVector<Fz> {
    LorentzVector::new(
        Fz::from_rational(&p[0]),
        Fz::from_rational(&p[1]),
        Fz::from_rational(&p[2]),
        Fz::from_rational(&p[3]),
    )
}

fn to_f64_momentum(p: &Mom) -> LorentzVector<f64> {
    let f = |x: &Q| x.to_f64().unwrap();
    LorentzVector::new(f(&p[0]), f(&p[1]), f(&p[2]), f(&p[3]))
}

/// The invariants the ansatz is written in: the dot products of every momentum
/// but `elim` (eliminated by momentum conservation), less the last such pair
/// (fixed by `p_elim² = m²`), and the Levi-Civita contractions of every four of
/// them. Exact, so the caller reduces them to whichever field it needs.
fn invariants_exact(moms: &[Mom], elim: usize) -> (Vec<Q>, Vec<Q>) {
    let basis: Vec<&Mom> = (0..moms.len())
        .filter(|&k| k != elim)
        .map(|k| &moms[k])
        .collect();
    let nb = basis.len();
    let mut dots = Vec::new();
    for a in 0..nb {
        for b in a + 1..nb {
            if (a, b) == (nb - 2, nb - 1) {
                continue;
            }
            dots.push(mdot(basis[a], basis[b]));
        }
    }
    let mut eps = Vec::new();
    for a in 0..nb {
        for b in a + 1..nb {
            for c in b + 1..nb {
                for d in c + 1..nb {
                    eps.push(det4([basis[a], basis[b], basis[c], basis[d]]));
                }
            }
        }
    }
    (dots, eps)
}

/// [`invariants_exact`] mod the current modulus.
fn invariants(moms: &[Mom], elim: usize) -> (Vec<u64>, Vec<u64>) {
    let (dots, eps) = invariants_exact(moms, elim);
    (
        dots.iter().map(reduce_rational).collect(),
        eps.iter().map(reduce_rational).collect(),
    )
}

fn det4(rows: [&Mom; 4]) -> Q {
    let m = |r: usize, c: usize| &rows[r][c];
    let det3 = |r: [usize; 3], c: [usize; 3]| -> Q {
        m(r[0], c[0]) * (m(r[1], c[1]) * m(r[2], c[2]) - m(r[1], c[2]) * m(r[2], c[1]))
            - m(r[0], c[1]) * (m(r[1], c[0]) * m(r[2], c[2]) - m(r[1], c[2]) * m(r[2], c[0]))
            + m(r[0], c[2]) * (m(r[1], c[0]) * m(r[2], c[1]) - m(r[1], c[1]) * m(r[2], c[0]))
    };
    let rest = [1, 2, 3];
    m(0, 0) * det3(rest, [1, 2, 3]) - m(0, 1) * det3(rest, [0, 2, 3])
        + m(0, 2) * det3(rest, [0, 1, 3])
        - m(0, 3) * det3(rest, [0, 1, 2])
}

// ── the black box ────────────────────────────────────────────────────────────

/// One propagator as the evaluator lowers it.
struct Propagator {
    momentum: Vec<i8>,
    mass: f64,
    width: f64,
}

/// A process prepared for sampling: the unpruned evaluator, the diagram order its
/// configuration amplitudes come in, and each diagram's propagators.
struct Process {
    label: String,
    set: DiagramSet,
    model: std::sync::Arc<UFOModel>,
    evaluated: EvaluatedModel,
    evaluator: AmplitudeEvaluator,
    /// Diagram index of each `run_config_amps` entry.
    order: Vec<usize>,
    /// Per diagram (in `order`), its propagators.
    props: Vec<Vec<Propagator>>,
    /// Final-state masses squared, exact, for the point generator.
    out_masses: Vec<Option<Q>>,
    /// External massless vector legs. Their helicity sum is HELAS's
    /// `−g + (p n + n p)/(p·n)` with `n = (p⁰, −p⃗)`, which a single diagram pair does
    /// not cancel (the Ward identity holds for the sum over diagrams only). In the
    /// partonic centre-of-mass frame `n = 2(p·t)t − p` with `t ∝ P = p₁ + p₂`, so the
    /// pair is rational in the invariants with the extra denominator `(p·P)²` per
    /// such leg — and only in that frame.
    vector_legs: Vec<usize>,
}

impl Process {
    fn new(process: &str) -> Self {
        let model = common::sm_model();
        let mut sets = common::generate_with(process, model.as_ref());
        assert_eq!(sets.len(), 1, "{process}: expected one subprocess");
        let set = sets.remove(0);
        let evaluator = AmplitudeEvaluator::compile(&set, model.as_ref()).expect("compiles");
        let evaluated = EvaluatedModel::from_model(model.clone());
        let groups = config_groups(&set.diagrams, model.as_ref());
        assert_eq!(
            groups.iter().map(Vec::len).collect::<Vec<_>>(),
            evaluator.config_amp_counts(),
            "{process}: one amplitude per diagram (colourless, no contact diagrams)"
        );
        let order: Vec<usize> = groups.into_iter().flatten().collect();
        assert_eq!(
            order.len(),
            set.diagrams.len(),
            "every diagram carries a configuration"
        );
        let props = order
            .iter()
            .map(|&d| {
                let diagram = &set.diagrams[d];
                diagram
                    .props
                    .iter()
                    .map(|prop| Propagator {
                        momentum: prop.momentum.clone(),
                        mass: evaluated.mass(prop.particle),
                        width: if prop.is_spacelike(diagram.n_in) {
                            0.0
                        } else {
                            evaluated.width(prop.particle)
                        },
                    })
                    .collect()
            })
            .collect();
        let out_masses = evaluator.external_particles()[evaluator.n_in()..]
            .iter()
            .map(|&id| {
                let m = evaluated.mass(id);
                (m != 0.0).then(|| {
                    let m = Q::from_float(m).expect("finite mass");
                    &m * &m
                })
            })
            .collect();
        let vector_legs = evaluator
            .external_particles()
            .iter()
            .enumerate()
            .filter(|&(_, &id)| model.particle(id).spin == 3 && evaluated.mass(id) == 0.0)
            .map(|(k, _)| k)
            .collect();
        Process {
            label: process.to_string(),
            vector_legs,
            set,
            model,
            evaluated,
            evaluator,
            order,
            props,
            out_masses,
        }
    }

    fn n_diagrams(&self) -> usize {
        self.order.len()
    }

    fn n_pairs(&self) -> usize {
        let n = self.n_diagrams();
        n * (n + 1) / 2
    }
}

/// `D = Π (q² − M² + i M Γ)` over a diagram's propagators.
fn denominator<F: vibegraph::helas::repr::Real + FromPrimitive>(
    props: &[Propagator],
    moms: &[LorentzVector<F>],
) -> C<F> {
    let mut d = C::new(F::one(), F::zero());
    for prop in props {
        let mut qv = [F::zero(); 4];
        for (k, &c) in prop.momentum.iter().enumerate() {
            if c == 0 {
                continue;
            }
            let s = F::from_i64(c as i64).unwrap();
            qv[0] = qv[0] + s * moms[k].e();
            qv[1] = qv[1] + s * moms[k].px();
            qv[2] = qv[2] + s * moms[k].py();
            qv[3] = qv[3] + s * moms[k].pz();
        }
        let q2 = qv[0] * qv[0] - qv[1] * qv[1] - qv[2] * qv[2] - qv[3] * qv[3];
        let m = F::from_f64(prop.mass).unwrap();
        let w = F::from_f64(prop.width).unwrap();
        d = d * C::new(q2 - m * m, m * w);
    }
    d
}

/// One sampled point: the invariants and every pair's numerator `N_ij`, mod `p`.
struct Sample {
    point: Vec<Mom>,
    dots: Vec<u64>,
    eps: Vec<u64>,
    /// `[re, im]` of `N_ij` for `i ≤ j`, row-major over the upper triangle.
    numerators: Vec<[u64; 2]>,
}

/// Per-pair `Σ_hel A_i A_j*` at one point, `i ≤ j`, in any field.
fn pair_sums<F: vibegraph::helas::repr::Real + FromPrimitive>(
    proc_: &Process,
    bound: &BoundAmplitude<F>,
    moms: &[LorentzVector<F>],
) -> Vec<C<F>> {
    let n = proc_.n_diagrams();
    let mut scratch = bound.scratch_space();
    let mut acc = vec![C::new(F::zero(), F::zero()); proc_.n_pairs()];
    for hel in proc_.evaluator.helicities() {
        let amps = bound.run_config_amps(moms, hel, &mut scratch);
        let mut k = 0;
        for i in 0..n {
            for j in i..n {
                acc[k] = acc[k] + amps[i] * amps[j].conj();
                k += 1;
            }
        }
    }
    acc
}

/// Whether the external wavefunctions of leg `k` at momentum `p` have their roots
/// in `Z_p`. The evaluator builds the same ones, so a leg failing here would only
/// be evaluated to be thrown away.
fn leg_roots_exist(proc_: &Process, k: usize, p: &Mom) -> bool {
    take_poison();
    let id = proc_.evaluator.external_particles()[k];
    let mom = to_fz_momentum(p);
    let mass = Fz::exact(proc_.evaluated.mass(id));
    match proc_.model.particle(id).spin {
        2 => {
            for charge in [Charge::Particle, Charge::Antiparticle] {
                for h in [SpinorHelicity::Up, SpinorHelicity::Down] {
                    let _ = Bispinor::<Fz, Ket>::from_momentum(mom, mass, h, charge);
                }
            }
        }
        3 => {
            for nsv in [-1, 1] {
                let _ = VectorWf::vxxxxx(mom, mass, 1, nsv);
            }
        }
        _ => {}
    }
    !take_poison()
}

/// Draw points from `seed` until one survives in `Z_p` (every root exists),
/// returning its sample. Sets this thread's modulus to `p`.
fn sample(proc_: &Process, bound: &BoundAmplitude<Fz>, p: u64, seed: u64) -> Sample {
    set_modulus(p);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    loop {
        let accept = |k: usize, p: &Mom| leg_roots_exist(proc_, k, p);
        let point = rational_point(
            &mut rng,
            &proc_.out_masses,
            !proc_.vector_legs.is_empty(),
            &accept,
        );
        take_poison();
        let moms: Vec<LorentzVector<Fz>> = point.iter().map(to_fz_momentum).collect();
        let sums = pair_sums(proc_, bound, &moms);
        let dens: Vec<C<Fz>> = proc_
            .props
            .iter()
            .map(|props| denominator(props, &moms))
            .collect();
        let total = madd(&point[0], &point[1]);
        let gauge = proc_.vector_legs.iter().fold(Fz::one(), |acc, &v| {
            let pv = Fz::from_rational(&mdot(&point[v], &total));
            acc * pv * pv
        });
        if take_poison() {
            continue;
        }
        let n = proc_.n_diagrams();
        let mut numerators = Vec::with_capacity(proc_.n_pairs());
        let mut k = 0;
        for i in 0..n {
            for j in i..n {
                numerators.push(parts(sums[k] * dens[i] * dens[j].conj() * gauge));
                k += 1;
            }
        }
        let (dots, eps) = invariants(&point, point.len() - 1);
        return Sample {
            point,
            dots,
            eps,
            numerators,
        };
    }
}

// ── the ansatz ───────────────────────────────────────────────────────────────

/// Exponent vectors of every monomial of total degree `≤ d` in `n` variables,
/// graded (all of degree 0, then 1, …).
fn monomials(n: usize, d: usize) -> Vec<Vec<u8>> {
    fn rec(n: usize, left: usize, prefix: &mut Vec<u8>, out: &mut Vec<Vec<u8>>) {
        if prefix.len() == n {
            if left == 0 {
                out.push(prefix.clone());
            }
            return;
        }
        for e in (0..=left).rev() {
            prefix.push(e as u8);
            rec(n, left - e, prefix, out);
            prefix.pop();
        }
    }
    let mut out = Vec::new();
    for deg in 0..=d {
        rec(n, deg, &mut Vec::new(), &mut out);
    }
    out
}

/// A column of the ansatz: a monomial in the dot products, times one Levi-Civita
/// contraction or none.
#[derive(Clone, Debug)]
struct Column {
    exps: Vec<u8>,
    eps: Option<usize>,
}

impl Column {
    fn degree(&self) -> usize {
        self.exps.iter().map(|&e| e as usize).sum::<usize>()
            + if self.eps.is_some() { 2 } else { 0 }
    }
}

fn ansatz(n_dots: usize, n_eps: usize, d: usize) -> Vec<Column> {
    let mut cols: Vec<Column> = monomials(n_dots, d)
        .into_iter()
        .map(|exps| Column { exps, eps: None })
        .collect();
    if d >= 2 {
        for l in 0..n_eps {
            cols.extend(
                monomials(n_dots, d - 2)
                    .into_iter()
                    .map(|exps| Column { exps, eps: Some(l) }),
            );
        }
    }
    cols
}

fn column_value(col: &Column, s: &Sample, p: u64) -> u64 {
    let mut v = match col.eps {
        Some(l) => s.eps[l],
        None => 1,
    };
    for (x, &e) in s.dots.iter().zip(&col.exps) {
        if e > 0 {
            v = v.mulm(x.powm(e as u64, &p), &p);
        }
    }
    v
}

/// The fit of every pair's numerator at one degree.
struct Fit {
    columns: Vec<Column>,
    /// Ansatz columns kept as pivots (the rest are linearly dependent on them as
    /// functions: Schouten and Gram relations).
    pivots: Vec<usize>,
    /// Per pair, `[re, im]` of each pivot column's coefficient.
    coeffs: Vec<Vec<[u64; 2]>>,
}

/// Row-reduce `[A | B]` mod `p` over `A`'s columns. Returns the pivot columns and
/// the reduced matrix, or `None` when a row beyond the rank has a non-zero
/// right-hand side (the ansatz does not contain the function).
fn reduce(mut rows: Vec<Vec<u64>>, n_cols: usize, p: u64) -> Option<(Vec<usize>, Vec<Vec<u64>>)> {
    let n_rows = rows.len();
    let width = rows[0].len();
    let mut pivots = Vec::new();
    let mut r = 0;
    for c in 0..n_cols {
        if r == n_rows {
            break;
        }
        let Some(k) = (r..n_rows).find(|&k| rows[k][c] != 0) else {
            continue;
        };
        rows.swap(r, k);
        let inv = rows[r][c].invm(&p).expect("non-zero pivot");
        for x in rows[r][c..].iter_mut() {
            *x = x.mulm(inv, &p);
        }
        let pivot_row = rows[r].clone();
        rows.par_iter_mut().enumerate().for_each(|(i, row)| {
            if i == r || row[c] == 0 {
                return;
            }
            let f = row[c];
            for (x, &y) in row[c..width].iter_mut().zip(&pivot_row[c..width]) {
                if y != 0 {
                    *x = x.subm(f.mulm(y, &p), &p);
                }
            }
        });
        pivots.push(c);
        r += 1;
    }
    let consistent = rows[r..]
        .iter()
        .all(|row| row[n_cols..].iter().all(|&x| x == 0));
    consistent.then_some((pivots, rows))
}

fn fit(samples: &[Sample], columns: Vec<Column>, p: u64) -> Option<Fit> {
    let n_pairs = samples[0].numerators.len();
    let n_cols = columns.len();
    let rows: Vec<Vec<u64>> = samples
        .par_iter()
        .map(|s| {
            let mut row: Vec<u64> = columns.iter().map(|c| column_value(c, s, p)).collect();
            for z in &s.numerators {
                row.extend_from_slice(z);
            }
            row
        })
        .collect();
    let (pivots, reduced) = reduce(rows, n_cols, p)?;
    let coeffs = (0..n_pairs)
        .map(|k| {
            (0..pivots.len())
                .map(|t| [reduced[t][n_cols + 2 * k], reduced[t][n_cols + 2 * k + 1]])
                .collect()
        })
        .collect();
    Some(Fit {
        columns,
        pivots,
        coeffs,
    })
}

/// Sample and fit at increasing degree until held-out points agree.
fn reconstruct(proc_: &Process, p: u64, seed: u64, max_degree: usize) -> (Fit, usize, Vec<Sample>) {
    set_modulus(p);
    let bound = BoundAmplitude::<Fz>::bind(&proc_.evaluator, &proc_.evaluated);
    let seed = seed << 32;
    let mut samples: Vec<Sample> = vec![sample(proc_, &bound, p, seed)];
    let (n_dots, n_eps) = (samples[0].dots.len(), samples[0].eps.len());
    const HELD_OUT: usize = 12;
    for d in 1..=max_degree {
        let columns = ansatz(n_dots, n_eps, d);
        let need = columns.len() + HELD_OUT;
        if samples.len() < need {
            let fresh: Vec<Sample> = (samples.len()..need)
                .into_par_iter()
                .map(|k| sample(proc_, &bound, p, seed + k as u64))
                .collect();
            samples.extend(fresh);
        }
        set_modulus(p);
        if let Some(fit) = fit(&samples[..need], columns, p) {
            return (fit, d, samples);
        }
    }
    panic!(
        "{}: no fit up to degree {max_degree} (wrong denominators, or a box that is not a \
         function of the invariants)",
        proc_.label
    );
}

// ── tests ────────────────────────────────────────────────────────────────────

/// The root convention keeps each external leg's completeness relation, which is
/// all the helicity sum depends on: `Σ_h u ū = p̸ + m`, `Σ_h v v̄ = p̸ − m`,
/// `Σ_λ ε ε* = −g + p p / M²` for a massive vector, and for a massless one
/// `−g + (p n + n p)/(p·n)` with HELAS's `n = (p⁰, −p⃗)`. Checked mod `p` at
/// rational momenta, massless and massive, on points whose roots exist.
#[test]
fn fz_wavefunctions_satisfy_completeness() {
    set_modulus(primes(1)[0]);
    let mut rng = ChaCha8Rng::seed_from_u64(0x5EED_41);
    let mut checked = [0usize; 4];
    let mut attempts = 0;
    while checked.iter().any(|&c| c < 8) {
        attempts += 1;
        assert!(attempts < 20_000, "too few points survive: {checked:?}");
        let energy = q(rng.random_range(5..=40), rng.random_range(1..=4));
        let k = massless(&energy, &unit_vector(&mut rng));
        let n = massless(&q(3, 1), &unit_vector(&mut rng));
        let m2 = q(rng.random_range(1..=30), rng.random_range(1..=5));
        for massive in [false, true] {
            let p = if massive {
                massive_from(&k, &n, &m2)
            } else {
                k.clone()
            };
            take_poison();
            let pz = to_fz_momentum(&p);
            // The evaluator's masses are f64 parameters; the generator's are exact
            // rationals, so the field sees the mass through its own square root.
            let mass = if massive {
                Fz::from_rational(&mdot(&p, &p)).sqrt()
            } else {
                Fz::zero()
            };
            if take_poison() {
                continue;
            }
            for (slot, charge, sign) in [(0, Charge::Particle, 1), (1, Charge::Antiparticle, -1)] {
                let slot = slot + 2 * (massive as usize);
                let us: Vec<Bispinor<Fz, Ket>> = [SpinorHelicity::Up, SpinorHelicity::Down]
                    .into_iter()
                    .map(|h| Bispinor::from_momentum(pz, mass, h, charge))
                    .collect();
                if take_poison() {
                    continue;
                }
                let slashed = ComplexVector::from(pz);
                for b in 0..4 {
                    let e: Bispinor<Fz, Ket> =
                        Bispinor::from_components(std::array::from_fn(|k| {
                            if k == b {
                                C::new(Fz::one(), Fz::zero())
                            } else {
                                C::new(Fz::zero(), Fz::zero())
                            }
                        }));
                    let mut lhs = [C::new(Fz::zero(), Fz::zero()); 4];
                    for u in &us {
                        let ub = u.bar();
                        let ub_e = (0..4).fold(C::new(Fz::zero(), Fz::zero()), |acc, k| {
                            acc + ub.component(k) * e.component(k)
                        });
                        for (l, x) in lhs.iter_mut().enumerate() {
                            *x = *x + u.component(l) * ub_e;
                        }
                    }
                    let rhs = e.slash(&slashed);
                    let m_signed = if sign > 0 { mass } else { -mass };
                    for (l, x) in lhs.iter().enumerate() {
                        let want = rhs.component(l) + e.component(l) * m_signed;
                        assert_eq!(
                            parts(*x),
                            parts(want),
                            "Σ u ū ≠ p̸ ± m (massive={massive}, charge={charge:?}) at component ({l},{b})"
                        );
                    }
                }
                checked[slot] += 1;
            }
        }
    }
    // Vectors: massive (λ = −1, 0, 1) and massless (λ = ±1).
    let mut vchecked = [0usize; 2];
    while vchecked.iter().any(|&c| c < 8) {
        attempts += 1;
        assert!(
            attempts < 40_000,
            "too few vector points survive: {vchecked:?}"
        );
        let energy = q(rng.random_range(5..=40), rng.random_range(1..=4));
        let k = massless(&energy, &unit_vector(&mut rng));
        let n = massless(&q(3, 1), &unit_vector(&mut rng));
        let m2 = q(rng.random_range(1..=30), rng.random_range(1..=5));
        for massive in [false, true] {
            let p = if massive {
                massive_from(&k, &n, &m2)
            } else {
                k.clone()
            };
            take_poison();
            let pz = to_fz_momentum(&p);
            let mass = if massive {
                Fz::from_rational(&mdot(&p, &p)).sqrt()
            } else {
                Fz::zero()
            };
            let hels: &[i32] = if massive { &[-1, 0, 1] } else { &[-1, 1] };
            let eps: Vec<VectorWf<Fz>> = hels
                .iter()
                .map(|&h| VectorWf::vxxxxx(pz, mass, h, 1))
                .collect();
            if take_poison() {
                continue;
            }
            let pv = [pz.e(), pz.px(), pz.py(), pz.pz()];
            let metric = [1.0, -1.0, -1.0, -1.0];
            let nv = [pz.e(), -pz.px(), -pz.py(), -pz.pz()];
            let p_dot_n = pv[0] * nv[0] - pv[1] * nv[1] - pv[2] * nv[2] - pv[3] * nv[3];
            for mu in 0..4 {
                for nu in 0..4 {
                    let mut lhs = C::new(Fz::zero(), Fz::zero());
                    for e in &eps {
                        lhs = lhs + e.eps.component(mu) * e.eps.component(nu).conj();
                    }
                    let g = if mu == nu {
                        Fz::exact(metric[mu])
                    } else {
                        Fz::zero()
                    };
                    let want = if massive {
                        -g + pv[mu] * pv[nu] / (mass * mass)
                    } else {
                        -g + (pv[mu] * nv[nu] + nv[mu] * pv[nu]) / p_dot_n
                    };
                    assert_eq!(
                        parts(lhs),
                        parts(C::new(want, Fz::zero())),
                        "Σ ε ε* wrong (massive={massive}) at ({mu},{nu})"
                    );
                }
            }
            vchecked[massive as usize] += 1;
        }
    }
}

/// Per-pair term counts of a fit.
struct Census {
    /// Non-zero complex coefficients per pair.
    per_pair: Vec<usize>,
    /// Distinct ansatz columns any pair uses.
    used_columns: usize,
}

fn census(fit: &Fit) -> Census {
    let mut used = vec![false; fit.pivots.len()];
    let per_pair = fit
        .coeffs
        .iter()
        .map(|c| {
            c.iter()
                .enumerate()
                .filter(|(t, z)| {
                    let nz = z[0] != 0 || z[1] != 0;
                    if nz {
                        used[*t] = true;
                    }
                    nz
                })
                .count()
        })
        .collect();
    Census {
        per_pair,
        used_columns: used.iter().filter(|&&u| u).count(),
    }
}

fn report(proc_: &Process, fit: &Fit, degree: usize, n_samples: usize, secs: f64) -> Census {
    let c = census(fit);
    let total: usize = c.per_pair.iter().sum();
    let max = c.per_pair.iter().max().copied().unwrap_or(0);
    let n_dots = fit.columns[0].exps.len();
    let n_eps = fit
        .columns
        .iter()
        .filter_map(|c| c.eps)
        .max()
        .map_or(0, |l| l + 1);
    let max_deg = fit
        .pivots
        .iter()
        .map(|&t| fit.columns[t].degree())
        .max()
        .unwrap_or(0);
    println!(
        "{}: {} diagrams, {} pairs, {} dot products + {} ε; degree {degree} \
         ({} columns, {} independent, max used degree {max_deg}); {} samples, {secs:.1} s",
        proc_.label,
        proc_.n_diagrams(),
        proc_.n_pairs(),
        n_dots,
        n_eps,
        fit.columns.len(),
        fit.pivots.len(),
        n_samples,
    );
    println!(
        "  terms per pair: total {total}, mean {:.1}, max {max}; distinct columns used {}",
        total as f64 / proc_.n_pairs() as f64,
        c.used_columns
    );
    c
}

// ── lifting to Q ─────────────────────────────────────────────────────────────

/// Chinese remaindering of residues `r_k mod p_k` into `(x mod M, M)`.
fn crt(residues: &[u64], primes: &[u64]) -> (BigInt, BigInt) {
    let mut x = BigInt::zero();
    let mut m = BigInt::one();
    for (&r, &p) in residues.iter().zip(primes) {
        let pb = BigInt::from(p);
        // x' ≡ x (mod m), x' ≡ r (mod p): x' = x + m·((r − x)·m⁻¹ mod p).
        let x_mod_p = ((&x % &pb) + &pb) % &pb;
        let m_mod_p = (&m % &pb).to_u64().unwrap();
        let inv = m_mod_p.invm(&p).expect("distinct primes");
        let diff = (r as i128 - x_mod_p.to_u64().unwrap() as i128).rem_euclid(p as i128) as u64;
        let t = diff.mulm(inv, &p);
        x += &m * BigInt::from(t);
        m *= &pb;
    }
    (x, m)
}

/// Wang's rational reconstruction: the `n/d ≡ a (mod m)` with `|n|, |d| ≤ √(m/2)`,
/// if there is one.
fn rational_reconstruct(a: &BigInt, m: &BigInt) -> Option<BigRational> {
    let bound = (m / BigInt::from(2)).sqrt();
    let (mut r0, mut r1) = (m.clone(), ((a % m) + m) % m);
    let (mut t0, mut t1) = (BigInt::zero(), BigInt::one());
    while r1 > bound {
        let qt = &r0 / &r1;
        let r2 = &r0 - &qt * &r1;
        let t2 = &t0 - &qt * &t1;
        (r0, r1) = (r1, r2);
        (t0, t1) = (t1, t2);
    }
    if t1.is_zero() || t1.abs() > bound {
        return None;
    }
    Some(BigRational::new(r1, t1))
}

/// Every pair's coefficients over `Q(i)`, from fits over enough primes that one
/// more prime confirms them.
struct Lifted {
    columns: Vec<Column>,
    pivots: Vec<usize>,
    /// Per pair, per pivot, `[re, im]`.
    coeffs: Vec<Vec<[BigRational; 2]>>,
    n_primes: usize,
}

fn lift(proc_: &Process, max_primes: usize) -> Lifted {
    let ps = primes(max_primes);
    let mut fits: Vec<Fit> = Vec::new();
    for (k, &p) in ps.iter().enumerate() {
        let (fit, _, _) = reconstruct(proc_, p, 7, 8);
        if let Some(first) = fits.first() {
            assert_eq!(
                first.pivots, fit.pivots,
                "pivot columns differ between primes"
            );
        }
        fits.push(fit);
        if k < 2 {
            continue;
        }
        // Reconstruct from all but the newest prime, confirm against the newest.
        let used = &ps[..k];
        let newest = ps[k];
        let mut ok = true;
        let mut coeffs = Vec::new();
        'pairs: for pair in 0..fits[0].coeffs.len() {
            let mut row = Vec::new();
            for t in 0..fits[0].pivots.len() {
                let mut z: Vec<BigRational> = Vec::with_capacity(2);
                for part in 0..2 {
                    let residues: Vec<u64> =
                        fits[..k].iter().map(|f| f.coeffs[pair][t][part]).collect();
                    let (x, m) = crt(&residues, used);
                    let Some(r) = rational_reconstruct(&x, &m) else {
                        ok = false;
                        break 'pairs;
                    };
                    set_modulus(newest);
                    if reduce_rational(&r) != fits[k].coeffs[pair][t][part] {
                        ok = false;
                        break 'pairs;
                    }
                    z.push(r);
                }
                row.push([z[0].clone(), z[1].clone()]);
            }
            coeffs.push(row);
        }
        if ok {
            return Lifted {
                columns: fits[0].columns.clone(),
                pivots: fits[0].pivots.clone(),
                coeffs,
                n_primes: k + 1,
            };
        }
    }
    panic!(
        "{}: coefficients not reconstructed with {max_primes} primes",
        proc_.label
    );
}

/// The f64 invariants of a point, in the order [`invariants`] gives them.
fn invariants_f64(moms: &[Mom]) -> (Vec<f64>, Vec<f64>) {
    let (dots, eps) = invariants_exact(moms, moms.len() - 1);
    let f = |x: &Q| x.to_f64().unwrap();
    (dots.iter().map(f).collect(), eps.iter().map(f).collect())
}

/// The lifted per-pair polynomials against the f64 evaluator's `Σ_hel A_i A_j*`,
/// at fresh physical points: the worst relative deviation, measured against the
/// largest single term so a cancelling sum is not judged by its remainder.
fn lifted_vs_f64(proc_: &Process, lifted: &Lifted, n_points: usize) -> f64 {
    let bound = BoundAmplitude::<f64>::bind(&proc_.evaluator, &proc_.evaluated);
    let mut rng = ChaCha8Rng::seed_from_u64(0xF64);
    let coeffs: Vec<Vec<[f64; 2]>> = lifted
        .coeffs
        .iter()
        .map(|row| {
            row.iter()
                .map(|z| [z[0].to_f64().unwrap(), z[1].to_f64().unwrap()])
                .collect()
        })
        .collect();
    let mut worst: f64 = 0.0;
    for _ in 0..n_points {
        let point = rational_point(
            &mut rng,
            &proc_.out_masses,
            !proc_.vector_legs.is_empty(),
            &any_leg,
        );
        let moms: Vec<LorentzVector<f64>> = point.iter().map(to_f64_momentum).collect();
        let sums = pair_sums(proc_, &bound, &moms);
        let dens: Vec<C<f64>> = proc_
            .props
            .iter()
            .map(|props| denominator(props, &moms))
            .collect();
        let total = madd(&point[0], &point[1]);
        let gauge: f64 = proc_
            .vector_legs
            .iter()
            .map(|&v| mdot(&point[v], &total).to_f64().unwrap().powi(2))
            .product();
        let (dots, eps) = invariants_f64(&point);
        let n = proc_.n_diagrams();
        let mut k = 0;
        for i in 0..n {
            for j in i..n {
                let mut value = C::new(0.0, 0.0);
                let mut scale: f64 = 0.0;
                for (t, &col) in lifted.pivots.iter().enumerate() {
                    let column = &lifted.columns[col];
                    let mut x = column.eps.map_or(1.0, |l| eps[l]);
                    for (v, &e) in dots.iter().zip(&column.exps) {
                        x *= v.powi(e as i32);
                    }
                    let term = C::new(coeffs[k][t][0], coeffs[k][t][1]) * x;
                    scale = scale.max(term.norm());
                    value += term;
                }
                let expected = sums[k] * dens[i] * dens[j].conj() * gauge;
                let dev = (value - expected).norm() / scale.max(f64::MIN_POSITIVE);
                worst = worst.max(dev);
                k += 1;
            }
        }
    }
    worst
}

/// The whole chain against the f64 evaluator: fits over several primes, lifted to
/// `Q(i)` by Chinese remaindering and rational reconstruction (confirmed by one
/// more prime), then evaluated in f64 at fresh physical points and compared with
/// the evaluator's own `Σ_hel A_i A_j*` times the pair's denominator.
///
/// Blind spot: the box *is* the evaluator, so this pins the field arithmetic, the
/// root convention and the fit, not the amplitudes, which the amplitude gate
/// holds against MadGraph. The comparison is relative to the largest single
/// term, so a numerator that cancels does not inflate it.
#[test]
fn per_pair_numerators_lift_to_the_f64_evaluator() {
    for (process, primes_needed) in [("e+ e- > mu+ mu-", 8), ("e+ e- > mu+ mu- a", 12)] {
        let proc_ = Process::new(process);
        let lifted = lift(&proc_, 24);
        let worst = lifted_vs_f64(&proc_, &lifted, 20);
        println!(
            "{process}: lifted with {} primes; worst deviation {worst:.2e}",
            lifted.n_primes
        );
        assert!(
            lifted.n_primes <= primes_needed + 4,
            "{process}: coefficient height grew ({} primes)",
            lifted.n_primes
        );
        assert!(
            worst < 1e-12,
            "{process}: lifted numerators off the f64 evaluator by {worst:.2e}"
        );
    }
}

// ── what the counts mean ─────────────────────────────────────────────────────

/// Per eliminated momentum, per pair, the term count of the same samples refitted
/// in that basis. The box values do not depend on the basis; only the columns do.
fn basis_probe(samples: &[Sample], degree: usize, p: u64) -> Vec<Vec<usize>> {
    set_modulus(p);
    let n = samples[0].point.len();
    (0..n)
        .map(|elim| {
            let rebased: Vec<Sample> = samples
                .iter()
                .map(|s| {
                    let (dots, eps) = invariants(&s.point, elim);
                    Sample {
                        point: Vec::new(),
                        dots,
                        eps,
                        numerators: s.numerators.clone(),
                    }
                })
                .collect();
            let columns = ansatz(rebased[0].dots.len(), rebased[0].eps.len(), degree);
            let need = columns.len() + 12;
            assert!(rebased.len() >= need, "too few samples to refit");
            let fit = fit(&rebased[..need], columns, p).expect("same degree in every basis");
            census(&fit).per_pair
        })
        .collect()
}

/// A diagram's propagators as an unordered set of `(±momentum, mass, width)`.
type Signature = Vec<(Vec<i8>, u64, u64)>;

fn denominator_signature(props: &[Propagator]) -> Signature {
    let mut sig: Signature = props
        .iter()
        .map(|prop| {
            let flip = prop
                .momentum
                .iter()
                .find(|&&c| c != 0)
                .is_some_and(|&c| c < 0);
            let m: Vec<i8> = prop
                .momentum
                .iter()
                .map(|&c| if flip { -c } else { c })
                .collect();
            (m, prop.mass.to_bits(), prop.width.to_bits())
        })
        .collect();
    sig.sort();
    sig
}

/// Terms left after merging every pair that shares its denominator `D_i D_j*`
/// into one numerator, counted as the union of the merged supports — what the
/// merge gives at generic colour and sign weights. A cancellation the true
/// weights produce (gauge cancellation within a class) would lower it further.
fn merged_terms(proc_: &Process, fit: &Fit) -> (usize, usize) {
    let sigs: Vec<_> = proc_
        .props
        .iter()
        .map(|p| denominator_signature(p))
        .collect();
    let n = proc_.n_diagrams();
    let mut classes: std::collections::BTreeMap<(Signature, Signature), Vec<bool>> =
        std::collections::BTreeMap::new();
    let mut k = 0;
    for i in 0..n {
        for j in i..n {
            let key = if sigs[i] <= sigs[j] {
                (sigs[i].clone(), sigs[j].clone())
            } else {
                (sigs[j].clone(), sigs[i].clone())
            };
            let support = classes
                .entry(key)
                .or_insert_with(|| vec![false; fit.pivots.len()]);
            for (t, z) in fit.coeffs[k].iter().enumerate() {
                if z[0] != 0 || z[1] != 0 {
                    support[t] = true;
                }
            }
            k += 1;
        }
    }
    let total = classes
        .values()
        .map(|s| s.iter().filter(|&&b| b).count())
        .sum();
    (classes.len(), total)
}

/// Rank of the pairs × columns coefficient matrix (real and imaginary parts as
/// separate rows): the number of distinct linear forms in the monomials every
/// pair's numerator is a combination of.
fn coefficient_rank(fit: &Fit, p: u64) -> usize {
    let rows: Vec<Vec<u64>> = fit
        .coeffs
        .iter()
        .flat_map(|c| {
            [
                c.iter().map(|z| z[0]).collect::<Vec<_>>(),
                c.iter().map(|z| z[1]).collect(),
            ]
        })
        .filter(|r: &Vec<u64>| r.iter().any(|&x| x != 0))
        .collect();
    let n_cols = rows[0].len();
    reduce(rows, n_cols, p).expect("no right-hand side").0.len()
}

/// The per-pair form, evaluated in f64 the plain way: every monomial of the
/// ansatz built by one multiplication from a lower one, every pair's numerator a
/// sparse complex dot product with them, times `1/D_i · 1/D_j*`. Coefficient
/// values are irrelevant to the cost, so the fit's residues stand in for them.
struct TraceForm {
    /// For each ansatz column, how to build it: `(factor a, factor b)` indices
    /// into the value table, which starts with the invariants.
    plan: Vec<(usize, usize)>,
    n_dots: usize,
    n_eps: usize,
    /// Per pair: `(value index, re, im)`.
    pairs: Vec<Vec<(usize, f64, f64)>>,
}

impl TraceForm {
    fn new(fit: &Fit) -> Self {
        let n_dots = fit.columns[0].exps.len();
        let n_eps = fit
            .columns
            .iter()
            .filter_map(|c| c.eps)
            .max()
            .map_or(0, |l| l + 1);
        // Value table: [1, dots…, eps…, columns…]; column c lives at `base + c`.
        let base = 1 + n_dots + n_eps;
        let index: std::collections::HashMap<(Vec<u8>, Option<usize>), usize> = fit
            .columns
            .iter()
            .enumerate()
            .map(|(c, col)| ((col.exps.clone(), col.eps), base + c))
            .collect();
        let plan = fit
            .columns
            .iter()
            .map(|col| {
                if let Some(l) = col.eps {
                    return (index[&(col.exps.clone(), None)], 1 + n_dots + l);
                }
                match col.exps.iter().position(|&e| e > 0) {
                    None => (0, 0),
                    Some(v) => {
                        let mut parent = col.exps.clone();
                        parent[v] -= 1;
                        (index[&(parent, None)], 1 + v)
                    }
                }
            })
            .collect();
        let pairs = fit
            .coeffs
            .iter()
            .map(|c| {
                c.iter()
                    .enumerate()
                    .filter(|(_, z)| z[0] != 0 || z[1] != 0)
                    .map(|(t, z)| {
                        (
                            base + fit.pivots[t],
                            z[0] as f64 * 1e-18,
                            z[1] as f64 * 1e-18,
                        )
                    })
                    .collect()
            })
            .collect();
        TraceForm {
            plan,
            n_dots,
            n_eps,
            pairs,
        }
    }

    fn terms(&self) -> usize {
        self.pairs.iter().map(Vec::len).sum()
    }

    fn eval(&self, proc_: &Process, moms: &[LorentzVector<f64>], values: &mut Vec<f64>) -> f64 {
        let n = moms.len();
        let basis = &moms[..n - 1];
        let dot = |a: &LorentzVector<f64>, b: &LorentzVector<f64>| {
            a.e() * b.e() - a.px() * b.px() - a.py() * b.py() - a.pz() * b.pz()
        };
        values.clear();
        values.push(1.0);
        let nb = basis.len();
        for a in 0..nb {
            for b in a + 1..nb {
                if (a, b) != (nb - 2, nb - 1) {
                    values.push(dot(&basis[a], &basis[b]));
                }
            }
        }
        let row = |p: &LorentzVector<f64>| [p.e(), p.px(), p.py(), p.pz()];
        for a in 0..nb {
            for b in a + 1..nb {
                for c in b + 1..nb {
                    for d in c + 1..nb {
                        let m = [
                            row(&basis[a]),
                            row(&basis[b]),
                            row(&basis[c]),
                            row(&basis[d]),
                        ];
                        values.push(det4_f64(&m));
                    }
                }
            }
        }
        debug_assert_eq!(values.len(), 1 + self.n_dots + self.n_eps);
        for &(a, b) in &self.plan {
            let v = values[a] * values[b];
            values.push(v);
        }
        let inv_d: Vec<C<f64>> = proc_
            .props
            .iter()
            .map(|props| denominator(props, moms).inv())
            .collect();
        let nd = proc_.n_diagrams();
        let mut total = 0.0;
        let mut k = 0;
        for i in 0..nd {
            for j in i..nd {
                let (mut re, mut im) = (0.0, 0.0);
                for &(v, cr, ci) in &self.pairs[k] {
                    re += cr * values[v];
                    im += ci * values[v];
                }
                let w = inv_d[i] * inv_d[j].conj();
                let t = re * w.re - im * w.im;
                total += if i == j { t } else { 2.0 * t };
                k += 1;
            }
        }
        total
    }
}

fn det4_f64(m: &[[f64; 4]; 4]) -> f64 {
    let det3 = |r: [usize; 3], c: [usize; 3]| -> f64 {
        m[r[0]][c[0]] * (m[r[1]][c[1]] * m[r[2]][c[2]] - m[r[1]][c[2]] * m[r[2]][c[1]])
            - m[r[0]][c[1]] * (m[r[1]][c[0]] * m[r[2]][c[2]] - m[r[1]][c[2]] * m[r[2]][c[0]])
            + m[r[0]][c[2]] * (m[r[1]][c[0]] * m[r[2]][c[1]] - m[r[1]][c[1]] * m[r[2]][c[0]])
    };
    let rest = [1, 2, 3];
    m[0][0] * det3(rest, [1, 2, 3]) - m[0][1] * det3(rest, [0, 2, 3])
        + m[0][2] * det3(rest, [0, 1, 3])
        - m[0][3] * det3(rest, [0, 1, 2])
}

/// Partonic-CM points with the beams along ±z, as the production (helicity-pruned)
/// evaluator requires.
fn cm_z_points(proc_: &Process, n: usize) -> Vec<Vec<LorentzVector<f64>>> {
    let mut rng = ChaCha8Rng::seed_from_u64(0xC3);
    (0..n)
        .map(|_| {
            let point = rational_point(&mut rng, &proc_.out_masses, true, &any_leg);
            // Rotate the beam onto +z in f64: the point is only for timing.
            let p: Vec<[f64; 4]> = point
                .iter()
                .map(|m| std::array::from_fn(|k| m[k].to_f64().unwrap()))
                .collect();
            let b = p[0];
            let norm = (b[1] * b[1] + b[2] * b[2] + b[3] * b[3]).sqrt();
            let z = [b[1] / norm, b[2] / norm, b[3] / norm];
            // Rodrigues rotation taking z onto ẑ.
            let axis = [z[1], -z[0], 0.0];
            let s = (axis[0] * axis[0] + axis[1] * axis[1]).sqrt();
            let c = z[2];
            p.iter()
                .map(|v| {
                    let w = [v[1], v[2], v[3]];
                    let r = if s < 1e-15 {
                        if c > 0.0 {
                            w
                        } else {
                            [w[0], -w[1], -w[2]]
                        }
                    } else {
                        let k = [axis[0] / s, axis[1] / s, 0.0];
                        let kxw = [
                            k[1] * w[2] - k[2] * w[1],
                            k[2] * w[0] - k[0] * w[2],
                            k[0] * w[1] - k[1] * w[0],
                        ];
                        let kdw = k[0] * w[0] + k[1] * w[1] + k[2] * w[2];
                        std::array::from_fn(|i| w[i] * c + kxw[i] * s + k[i] * kdw * (1.0 - c))
                    };
                    LorentzVector::new(v[0], r[0], r[1], r[2])
                })
                .collect()
        })
        .collect()
}

/// Nanoseconds per point: the production `eval_m2` (helicity-pruned, recycled)
/// against the per-pair form.
fn time_forms(proc_: &Process, fit: &Fit) -> (f64, f64) {
    let points = cm_z_points(proc_, 256);
    let mut pruned = AmplitudeEvaluator::compile(&proc_.set, proc_.model.as_ref()).unwrap();
    pruned.prune_zero_helicities(&proc_.evaluated);
    let bound = BoundAmplitude::<f64>::bind(&pruned, &proc_.evaluated);
    let mut scratch = bound.scratch_space();
    let form = TraceForm::new(fit);
    let mut values = Vec::new();
    let time = |f: &mut dyn FnMut(&[LorentzVector<f64>]) -> f64| {
        let mut sink = 0.0;
        let mut reps = 1;
        loop {
            let t = Instant::now();
            for _ in 0..reps {
                for p in &points {
                    sink += f(p);
                }
            }
            let el = t.elapsed().as_secs_f64();
            if el > 0.5 {
                std::hint::black_box(sink);
                return el * 1e9 / (reps * points.len()) as f64;
            }
            reps *= 2;
        }
    };
    let helicity = time(&mut |p| bound.eval_m2(p, &mut scratch));
    let trace = time(&mut |p| form.eval(proc_, p, &mut values));
    (helicity, trace)
}

/// One row of the measurement.
fn measure(process: &str, max_degree: usize, probe_bases: bool) {
    let proc_ = Process::new(process);
    let ps = primes(2);
    let t = Instant::now();
    let (fit, degree, samples) = reconstruct(&proc_, ps[0], 1, max_degree);
    let c = report(
        &proc_,
        &fit,
        degree,
        samples.len(),
        t.elapsed().as_secs_f64(),
    );
    let (fit2, degree2, _) = reconstruct(&proc_, ps[1], 2, max_degree);
    let c2 = census(&fit2);
    assert_eq!(
        degree, degree2,
        "{process}: the degree differs between primes"
    );
    assert_eq!(
        c.per_pair, c2.per_pair,
        "{process}: the term counts differ between primes"
    );
    let mut sorted = c.per_pair.clone();
    sorted.sort_unstable();
    let q = |f: f64| sorted[((sorted.len() - 1) as f64 * f).round() as usize];
    println!(
        "  per pair: min {} median {} max {}; second prime agrees pair by pair",
        q(0.0),
        q(0.5),
        q(1.0)
    );
    if probe_bases {
        let probe = basis_probe(&samples, degree, ps[0]);
        let per_basis: Vec<usize> = probe.iter().map(|v| v.iter().sum()).collect();
        let best: usize = (0..proc_.n_pairs())
            .map(|k| probe.iter().map(|v| v[k]).min().unwrap())
            .sum();
        println!(
            "  total by eliminated momentum: {per_basis:?}; each pair in its best basis: {best}"
        );
    }
    let (classes, merged) = merged_terms(&proc_, &fit);
    println!("  {classes} distinct denominators D_i D_j*; merged within each: {merged} terms");
    println!(
        "  rank of the pairs x columns coefficient matrix: {}",
        coefficient_rank(&fit, ps[0])
    );
    let (helicity_ns, trace_ns) = time_forms(&proc_, &fit);
    println!(
        "  per point: helicity eval_m2 {helicity_ns:.0} ns, per-pair form ({} terms) {trace_ns:.0} ns, ratio {:.1}",
        TraceForm::new(&fit).terms(),
        trace_ns / helicity_ns
    );
}

#[test]
#[ignore = "measurement: prints the per-pair trace-form census, ~minutes"]
fn measure_trace_form() {
    for (process, max_degree, probe_bases) in [
        ("e+ e- > mu+ mu-", 4, true),
        ("e+ e- > mu+ mu- a", 6, true),
        ("u u~ > e+ e- g", 6, true),
        ("g u > e+ e- u", 6, true),
        ("u d > e+ e- u d QCD=0", 5, true),
        ("e+ e- > mu+ mu- ta+ ta- QCD=0", 5, false),
    ] {
        measure(process, max_degree, probe_bases);
    }
}
