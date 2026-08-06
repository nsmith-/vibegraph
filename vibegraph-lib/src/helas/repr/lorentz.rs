//! Lorentz representation traits and concrete basis implementations.
//!
//! The Lorentz group Spin(1,3) ≅ SL(2,ℂ) has irreducible representations
//! labelled by two half-integers `(j_L, j_R)`. This module defines the base
//! trait [`LorentzRepr`] for any representation, and specialized traits like
//! [`SpinorRepr`] and [`VectorRepr`] for physics-specific operations. We also
//! provide concrete implementations.

use std::marker::PhantomData;

use num_traits::Zero;

use crate::helas::repr::numbers::Chirality;
use crate::helas::repr::vectorspace::impl_mul_for_array;

use super::numbers::{Charge, SpinorHelicity};
use super::vectorspace::{impl_vectorspace, ArrayBacked};
use super::{r, ri, Real, C};

// Complex multiply-accumulate expressed through the real fused multiply-add
// (`F::mul_add`). This lowers to a hardware FMA on both scalar `f64` and the SIMD
// lane field: the lane type implements `Float::mul_add` (a method) but not the
// `num_traits::MulAdd` trait, so `Complex::mul_add` is unavailable there — routing
// through the real `mul_add` keeps one code path that fuses on every `F: Real`.
// A single shared path also keeps the lane result bit-identical to the scalar one.

/// Complex product `a * b` (three real FMAs after the leading `re`/`im` products).
#[inline(always)]
fn cmul<F: Real>(a: C<F>, b: C<F>) -> C<F> {
    let re = (-a.im).mul_add(b.im, a.re * b.re);
    let im = a.re.mul_add(b.im, a.im * b.re);
    C::new(re, im)
}

/// Complex multiply-add `a * b + c`.
#[inline(always)]
fn cmul_add<F: Real>(a: C<F>, b: C<F>, c: C<F>) -> C<F> {
    let re = a.re.mul_add(b.re, c.re) - a.im * b.im;
    let im = a.re.mul_add(b.im, a.im.mul_add(b.re, c.im));
    C::new(re, im)
}

/// We will have some marker traits that are sealed in this module
/// (i.e. no external code can implement them)
mod sealed {
    pub trait Sealed {}
}

/// Base trait for a Lorentz representation.
///
/// Every Lorentz representation is a vector space over a real or complex scalar field `F`.
/// This trait marker indicates membership in an associated fiber bundle to the SO(1,3)
/// principle bundle; specialized subtypes like [`SpinorRepr`] and [`VectorRepr`] add
/// physics-specific operations.
///
/// # Type parameters
/// - `F` — the real scalar type (e.g. `f64`)
///
/// # Associated types
/// - `Scalar` — the scalar field of the representation (e.g. `F` or `C<F>` depending on the rep)
///
/// Representations that are self-dual (e.g. (½,½) for LorentzVector) have `Scalar = F`, whle
/// representations that are not (e.g. Weyl spinors: (½,0) has dual repr (0,½)) require `Scalar = C<F>`
/// to allow for complex coefficients.
pub trait LorentzRepr<F: Real>: Sized + Copy + 'static + PartialEq {
    /// Scalar type of this representation
    ///
    /// Usually `F` or [`C<F>`] depending on the representation
    type Scalar;
}

// Implement LorentzRepr for the (complex) scalar representation
impl<F: Real> LorentzRepr<F> for F {
    type Scalar = F;
}
impl<F: Real> LorentzRepr<F> for C<F> {
    type Scalar = C<F>;
}

// Next easiest is the vector representations

/// Marker trait for vector variance (contravariant vs. covariant).
pub trait Variance: sealed::Sealed + Copy + PartialEq + Eq + 'static {
    type Dual: Variance;
    const COVARIANT: bool;
}

/// Marker type for contravariant vectors (e.g. 4-momentum `p^μ`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Contravariant;
impl sealed::Sealed for Contravariant {}
impl Variance for Contravariant {
    type Dual = Covariant;
    const COVARIANT: bool = false;
}

/// Marker type for covariant vectors (e.g. polarisation vector `ε_μ`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Covariant;
impl sealed::Sealed for Covariant {}
impl Variance for Covariant {
    type Dual = Contravariant;
    const COVARIANT: bool = true;
}

/// Spin-1 Lorentz representation
pub trait VectorRepr<F: Real, V: Variance = Contravariant>: LorentzRepr<F> {
    type Dual: VectorRepr<F, V::Dual, Scalar = Self::Scalar>;

    /// Contract with a vector of the opposite variance to get a scalar: `v_μ w^μ` or `v^μ w_μ`
    fn dot(&self, other: &Self::Dual) -> Self::Scalar;

    /// Raise or lower the index using the Minkowski metric: `v^μ = g^μν v_ν` or `v_μ = g_μν v^ν`.
    ///
    /// Implementation note: suggest also implementing lower() and raise() on the concrete types
    /// of appropriate variance, calling this method internally.
    fn dualize(&self) -> Self::Dual;

    /// Bare normalization of the vector, without any metric contractions.
    ///
    /// This is not a basis-independent or Lorentz-invariant quantity, but it is useful for testing.
    #[cfg(test)]
    fn bare_norm_sq(self) -> F
    where
        F: std::iter::Sum;

    /// The `i`-th component in the underlying cartesian basis.
    ///
    /// Like [`bare_norm_sq`](Self::bare_norm_sq), this exposes the raw basis
    /// coordinates rather than a Lorentz-invariant quantity; it is mainly useful
    /// for testing and serialization.
    fn component(&self, i: usize) -> Self::Scalar;
}

/// A real 4-momentum vector in cartesian basis (E, px, py, pz).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LorentzVector<F: Real, V: Variance = Contravariant>([F; 4], PhantomData<V>);

impl<F: Real, V: Variance> ArrayBacked<F, 4> for LorentzVector<F, V> {
    fn as_array(&self) -> &[F; 4] {
        &self.0
    }

    fn from_array(arr: [F; 4]) -> Self {
        LorentzVector(arr, PhantomData)
    }
}

impl_vectorspace!(impl[F: Real, V: Variance] LorentzVector<F, V>, scalar = F);

impl<F: Real, V: Variance> LorentzRepr<F> for LorentzVector<F, V> {
    type Scalar = F;
}

impl<F: Real, V: Variance> VectorRepr<F, V> for LorentzVector<F, V> {
    type Dual = LorentzVector<F, V::Dual>;

    fn dot(&self, other: &Self::Dual) -> Self::Scalar {
        // if other is truly dual, the contraction is a simple dot product
        self.0[0] * other.0[0]
            + self.0[1] * other.0[1]
            + self.0[2] * other.0[2]
            + self.0[3] * other.0[3]
    }

    fn dualize(&self) -> Self::Dual {
        let arr = self.as_array();
        LorentzVector::from_array([arr[0], -arr[1], -arr[2], -arr[3]])
    }

    #[cfg(test)]
    fn bare_norm_sq(self) -> F
    where
        F: std::iter::Sum,
    {
        self.0.iter().map(|x| *x * *x).sum()
    }

    fn component(&self, i: usize) -> Self::Scalar {
        self.0[i]
    }
}

impl<F: Real> LorentzVector<F, Covariant> {
    /// Raise the index to get a contravariant vector: `p^μ = g^μν p_ν`.
    #[inline(always)]
    pub fn raise(self) -> LorentzVector<F, Contravariant> {
        self.dualize()
    }
}

impl<F: Real> LorentzVector<F, Contravariant> {
    /// Lower the index to get a covariant vector: `p_μ = g_μν p^ν`.
    #[inline(always)]
    pub fn lower(self) -> LorentzVector<F, Covariant> {
        self.dualize()
    }

    /// Active Lorentz boost by velocity `beta = [βx, βy, βz]` (|β| < 1):
    /// `E' = γ(E + β⃗·p⃗)`, `p⃗' = p⃗ + β⃗ (γ²/(γ+1) β⃗·p⃗ + γE)`.
    #[inline]
    pub fn boost(self, beta: [F; 3]) -> Self {
        let b2 = beta[0] * beta[0] + beta[1] * beta[1] + beta[2] * beta[2];
        let gamma = F::one() / (F::one() - b2).sqrt();
        let bp = beta[0] * self.px() + beta[1] * self.py() + beta[2] * self.pz();
        let coef = gamma * gamma / (gamma + F::one()) * bp + gamma * self.e();
        Self::new(
            gamma * (self.e() + bp),
            self.px() + coef * beta[0],
            self.py() + coef * beta[1],
            self.pz() + coef * beta[2],
        )
    }
}

impl<F: Real, V: Variance> LorentzVector<F, V> {
    /// Construct from individual components `[E, px, py, pz]`.
    #[inline(always)]
    pub fn new(e: F, px: F, py: F, pz: F) -> Self {
        LorentzVector([e, px, py, pz], PhantomData)
    }

    /// Construct from mass and cartesian 3-momentum
    #[inline(always)]
    pub fn from_pxpypzmass(px: F, py: F, pz: F, mass: F) -> Self {
        let p3_squared = px * px + py * py + pz * pz;
        let e = (p3_squared + mass * mass).sqrt();
        LorentzVector([e, px, py, pz], PhantomData)
    }

    /// Construct from mass and spherical 3-momentum
    /// `p = |p| (sinθ cosϕ, sinθ sinϕ, cosθ)`
    /// `θ` = polar angle from +z axis, `ϕ` = azimuthal angle in x-y plane from +x axis
    /// `p3` = momentum magnitude |p| = √(px² + py² + pz²)
    #[inline(always)]
    pub fn from_p_theta_phi_mass(p3: F, theta: F, phi: F, mass: F) -> Self {
        let px = p3 * theta.sin() * phi.cos();
        let py = p3 * theta.sin() * phi.sin();
        let pz = p3 * theta.cos();
        Self::from_pxpypzmass(px, py, pz, mass)
    }

    /// Energy component E = p^0.
    #[inline(always)]
    pub fn e(self) -> F {
        self.0[0]
    }

    /// x-component of momentum pˣ = p^1.
    #[inline(always)]
    pub fn px(self) -> F {
        self.0[1]
    }

    /// y-component of momentum pʸ = p^2.
    #[inline(always)]
    pub fn py(self) -> F {
        self.0[2]
    }

    /// z-component of momentum pᶻ = p^3.
    #[inline(always)]
    pub fn pz(self) -> F {
        self.0[3]
    }

    /// Momentum magnitude squared |p|² = px² + py² + pz²
    #[inline(always)]
    pub fn p3_squared(self) -> F {
        self.0[3].mul_add(
            self.0[3],
            self.0[2].mul_add(self.0[2], self.0[1] * self.0[1]),
        )
    }

    /// Momentum magnitude |p| = √(px² + py² + pz²).
    #[inline(always)]
    pub fn p3(self) -> F {
        self.p3_squared().sqrt()
    }

    /// Invariant mass squared m² = E² - |p|².
    #[inline(always)]
    pub fn m2(self) -> F {
        self.e().mul_add(self.e(), -self.p3_squared())
    }

    /// Invariant mass m = √(E² - |p|²).
    #[inline(always)]
    pub fn m(self) -> F {
        self.m2().sqrt()
    }
}

impl<F: Real + std::fmt::Display> std::fmt::Display for LorentzVector<F, Contravariant> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LorentzVector({}, {}, {}, {})",
            self.e(),
            self.px(),
            self.py(),
            self.pz()
        )
    }
}

impl<F: Real + std::fmt::Display> std::fmt::Display for LorentzVector<F, Covariant> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LorentzVector_co({}, {}, {}, {})",
            self.e(),
            self.px(),
            self.py(),
            self.pz()
        )
    }
}

/// A complex (e.g. polarisation) 4-vector.
///
/// This is the fiber type for [`SpinorRepr::left_current`] and [`SpinorRepr::right_current`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComplexVector<F: Real, V: Variance = Contravariant>([C<F>; 4], PhantomData<V>);

impl<F: Real, V: Variance> ArrayBacked<C<F>, 4> for ComplexVector<F, V> {
    fn as_array(&self) -> &[C<F>; 4] {
        &self.0
    }

    fn from_array(arr: [C<F>; 4]) -> Self {
        ComplexVector(arr, PhantomData)
    }
}

impl_vectorspace!(impl[F: Real, V: Variance] ComplexVector<F, V>, scalar = C<F>);

// Allow scalar multiplication by a real (performance optimization)
impl_mul_for_array!(impl[F: Real, V: Variance] ComplexVector<F, V>, scalar = F);

impl<F: Real, V: Variance> LorentzRepr<F> for ComplexVector<F, V> {
    type Scalar = C<F>;
}

impl<F: Real, V: Variance> VectorRepr<F, V> for ComplexVector<F, V> {
    type Dual = ComplexVector<F, V::Dual>;

    fn dot(&self, other: &Self::Dual) -> Self::Scalar {
        // Dual basis has the metric built in, so the contraction is a simple dot product
        let acc = cmul(self.0[0], other.0[0]);
        let acc = cmul_add(self.0[1], other.0[1], acc);
        let acc = cmul_add(self.0[2], other.0[2], acc);
        cmul_add(self.0[3], other.0[3], acc)
    }

    fn dualize(&self) -> Self::Dual {
        let arr = self.as_array();
        ComplexVector::from_array([arr[0], -arr[1], -arr[2], -arr[3]])
    }

    #[cfg(test)]
    fn bare_norm_sq(self) -> F
    where
        F: std::iter::Sum,
    {
        self.0.iter().map(|x| x.norm_sqr()).sum()
    }

    fn component(&self, i: usize) -> Self::Scalar {
        self.0[i]
    }
}

impl<F: Real> ComplexVector<F, Covariant> {
    /// Raise the index to get a contravariant vector: `ε^μ = g^μν ε_ν`.
    #[inline(always)]
    pub fn raise(self) -> ComplexVector<F, Contravariant> {
        self.dualize()
    }
}

impl<F: Real> ComplexVector<F, Contravariant> {
    /// Lower the index to get a covariant vector: `ε_μ = g_μν ε^ν`.
    #[inline(always)]
    pub fn lower(self) -> ComplexVector<F, Covariant> {
        self.dualize()
    }
}

impl<F: Real, V: Variance> From<LorentzVector<F, V>> for ComplexVector<F, V> {
    #[inline(always)]
    fn from(lv: LorentzVector<F, V>) -> Self {
        ComplexVector(
            [
                C::new(lv.0[0], F::zero()),
                C::new(lv.0[1], F::zero()),
                C::new(lv.0[2], F::zero()),
                C::new(lv.0[3], F::zero()),
            ],
            PhantomData,
        )
    }
}

impl<F: Real, V: Variance> ComplexVector<F, V> {
    pub fn new(eps: [C<F>; 4]) -> Self {
        ComplexVector(eps, PhantomData)
    }

    /// Specialized Minkowski dot product for a `ComplexVector` and a `LorentzVector`.
    ///
    /// Slightly more efficient than converting the `LorentzVector` to a
    /// `ComplexVector` and using the `dot` method
    #[inline(always)]
    pub fn dot_lorentz(&self, other: &LorentzVector<F, V>) -> C<F> {
        // Here the variance is THE SAME for both, so we need to manually insert the
        // metric signs in the contraction. Each term is complex×real, so the real and
        // imaginary parts accumulate through independent real FMA chains.
        let o = &other.0;
        let acc = |sel: fn(&C<F>) -> F| {
            let a = sel(&self.0[0]) * o[0];
            let a = (-sel(&self.0[1])).mul_add(o[1], a);
            let a = (-sel(&self.0[2])).mul_add(o[2], a);
            (-sel(&self.0[3])).mul_add(o[3], a)
        };
        C::new(acc(|c| c.re), acc(|c| c.im))
    }
}

impl<F: Real + std::fmt::Display> std::fmt::Display for ComplexVector<F, Contravariant> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ComplexVector({}, {}, {}, {})",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

impl<F: Real + std::fmt::Display> std::fmt::Display for ComplexVector<F, Covariant> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ComplexVector_co({}, {}, {}, {})",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

/// Sealed trait for the spinor Dirac-adjoint side (bra/ket), implemented by `Ket` and `Bra`.
pub trait DiracAdjoint: sealed::Sealed + Copy + PartialEq + Eq + 'static {
    type Dual: DiracAdjoint;
    const KET: bool;

    /// Assemble a (massive) bispinor with this adjoint (bra/ket)
    fn build_bispinor<F: Real>(
        p: LorentzVector<F, Contravariant>,
        mass: F,
        nhel: SpinorHelicity,
        nsf: Charge,
    ) -> Bispinor<F, Self>;

    /// Apply the gamma-slash `v̸ = γ^μ v_μ` to a bispinor of this adjoint.
    ///
    /// `v` holds the **covariant** components `v_μ`; the kernel sums them against
    /// `γ^μ` directly. Variance handling (lowering a contravariant input) is the
    /// caller's job — see [`SpinorRepr::slash`].
    ///
    /// The action depends on the adjoint because the open spinor index sits on a
    /// different side of `v̸`:
    /// - **ket**: the left action `v̸ ψ`;
    /// - **bra**: the right action `ψ̄ v̸`.
    fn slash_bispinor<F: Real>(psi: &[C<F>; 4], v: &[C<F>; 4]) -> [C<F>; 4];
}

/// Marker for ket spinors (`u`/`v` columns).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ket;
impl sealed::Sealed for Ket {}
impl DiracAdjoint for Ket {
    type Dual = Bra;
    const KET: bool = true;

    fn build_bispinor<F: Real>(
        p: LorentzVector<F, Contravariant>,
        mass: F,
        nhel: SpinorHelicity,
        nsf: Charge,
    ) -> Bispinor<F, Self> {
        Bispinor::from_array(weyl_ixxxxx(p, mass, nhel, nsf))
    }

    /// Ket left action `v̸ ψ`. In the Weyl basis `γ^μ = [[0, σ^μ], [σ̄^μ, 0]]`,
    /// so the left-chiral output is `(σ·v) ψ_R` and the right-chiral output is
    /// `(σ̄·v) ψ_L`, with
    /// `σ·v  = [[v₀+v₃, v₁−iv₂], [v₁+iv₂, v₀−v₃]]` and
    /// `σ̄·v = [[v₀−v₃, −(v₁−iv₂)], [−(v₁+iv₂), v₀+v₃]]`.
    #[inline(always)]
    fn slash_bispinor<F: Real>(psi: &[C<F>; 4], v: &[C<F>; 4]) -> [C<F>; 4] {
        let i = ri(F::one());
        let v0_p_v3 = v[0] + v[3];
        let v0_m_v3 = v[0] - v[3];
        let v1_m_iv2 = v[1] - i * v[2];
        let v1_p_iv2 = v[1] + i * v[2];

        // ψ_L ← (σ·v) ψ_R
        let l1 = cmul_add(v0_p_v3, psi[2], cmul(v1_m_iv2, psi[3]));
        let l2 = cmul_add(v1_p_iv2, psi[2], cmul(v0_m_v3, psi[3]));
        // ψ_R ← (σ̄·v) ψ_L
        let r1 = cmul_add(v0_m_v3, psi[0], -cmul(v1_m_iv2, psi[1]));
        let r2 = cmul_add(v0_p_v3, psi[1], -cmul(v1_p_iv2, psi[0]));

        [l1, l2, r1, r2]
    }
}

/// Marker for bra spinors (`ū`/`v̄` rows).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bra;
impl sealed::Sealed for Bra {}
impl DiracAdjoint for Bra {
    type Dual = Ket;
    const KET: bool = false;

    fn build_bispinor<F: Real>(
        p: LorentzVector<F, Contravariant>,
        mass: F,
        nhel: SpinorHelicity,
        nsf: Charge,
    ) -> Bispinor<F, Self> {
        Bispinor::from_array(weyl_ixxxxx(p, mass, nhel, nsf)).bar()
    }

    /// Bra right action `ψ̄ v̸`, stored in the same standard bra layout the
    /// bra spinor uses (`ψ̄ = ψ† γ⁰`, i.e. `bar`). This is the row–matrix
    /// product `(ψ̄) · v̸` written component-wise: the left block `(ψ̄_L)` multiplies
    /// `σ̄·v` into columns 0–1 and the right block `(ψ̄_R)` multiplies `σ·v` into
    /// columns 2–3. It is the transpose, NOT the chiral-swap, of the ket action,
    /// so that a plain dot with a ket reproduces the Lorentz scalar `ψ̄ v̸ ket`.
    #[inline(always)]
    fn slash_bispinor<F: Real>(psi: &[C<F>; 4], v: &[C<F>; 4]) -> [C<F>; 4] {
        let i = ri(F::one());
        let v0_p_v3 = v[0] + v[3];
        let v0_m_v3 = v[0] - v[3];
        let v1_m_iv2 = v[1] - i * v[2];
        let v1_p_iv2 = v[1] + i * v[2];

        [
            cmul_add(v0_m_v3, psi[2], -cmul(v1_p_iv2, psi[3])),
            cmul_add(v0_p_v3, psi[3], -cmul(v1_m_iv2, psi[2])),
            cmul_add(v0_p_v3, psi[0], cmul(v1_p_iv2, psi[1])),
            cmul_add(v1_m_iv2, psi[0], cmul(v0_m_v3, psi[1])),
        ]
    }
}

/// Marker for adjoints that can sit on the bra side of a bilinear.
pub trait IsBra: DiracAdjoint {}
impl IsBra for Bra {}

/// Spin-½ Lorentz representation.
///
/// This is a trait to allow for multiple concrete bases (e.g. Weyl, Dirac) to be implemented.
pub trait SpinorRepr<F: Real, Adj: DiracAdjoint = Ket>: LorentzRepr<F> {
    /// The dual representation (e.g. for Weyl spinors, the dual of (½,0) is (0,½)).
    type Dual: SpinorRepr<F, Adj::Dual, Scalar = Self::Scalar>;

    /// Dirac adjoint (dualize operation for bispinors)
    ///
    /// `ψ̄ = ψ† γ^0` and its inverse `ψ = γ^0 ψ̄†`.
    /// Implementations should add bar() and unbar() methods that call this internally.
    fn dualize(&self) -> Self::Dual;

    /// Left projection: `P_L = (1 - γ^5)/2` — zero the right-chiral (indices 2-3) components.
    fn project_left(self) -> Self;

    /// Right projection: `P_R = (1 + γ^5)/2` — zero the left-chiral (indices 0-1) components.
    fn project_right(self) -> Self;

    /// Apply the gamma-slash `v̸ = γ^μ v_μ`, returning `v̸ · self`.
    ///
    /// Used to attach a vector leg to a fermion line (the off-shell-current
    /// vertex factor `γ^μ ε_μ`) and to build the Dirac propagator numerator
    /// `q̸ + m`.
    fn slash<V: Variance>(self, v: &ComplexVector<F, V>) -> Self;

    /// Bare normalization of the spinor, without any gamma matrices or projections.
    ///
    /// This is not a basis-independent or Lorentz-invariant quantity, but it is useful for testing
    #[cfg(test)]
    fn bare_norm_sq(self) -> F
    where
        F: std::iter::Sum;

    // Things only applicable to the bra (out) side of a bilinear follow

    /// Left-handed fermion current  `J_L^μ = v̄_out γ^μ P_L u_in`.
    fn left_current(&self, fi: &Self::Dual) -> ComplexVector<F, Contravariant>
    where
        Adj: IsBra;

    /// Right-handed fermion current  `J_R^μ = v̄_out γ^μ P_R u_in`.
    fn right_current(&self, fi: &Self::Dual) -> ComplexVector<F, Contravariant>
    where
        Adj: IsBra;

    /// Scalar bilinear with chiral structure: `f̄ Γ f` where Γ ∈ {Identity, P_L, P_R}.
    fn scalar_bilinear(&self, fi: &Self::Dual, chirality: Chirality) -> C<F>
    where
        Adj: IsBra;

    /// Pseudoscalar bilinear with chiral structure: `f̄ γ^5 Γ f` where Γ ∈ {Identity, P_L, P_R}.
    fn pseudoscalar_bilinear(&self, fi: &Self::Dual, chirality: Chirality) -> C<F>
    where
        Adj: IsBra;

    /// Vector bilinear contraction: `f̄ γ^μ Γ f` where `Γ` encodes chirality.
    ///
    /// This can be implemented using the left and right currents:
    /// - Left (P_L): `J_L^μ = v̄_out γ^μ P_L u_in`
    /// - Right (P_R): `J_R^μ = v̄_out γ^μ P_R u_in`
    /// - Both (Identity): `J^μ = J_L^μ + J_R^μ`
    #[inline(always)]
    fn vector_bilinear(
        &self,
        fi: &Self::Dual,
        chirality: Chirality,
    ) -> ComplexVector<F, Contravariant>
    where
        Adj: IsBra,
    {
        match chirality {
            Chirality::Left => self.left_current(fi),
            Chirality::Right => self.right_current(fi),
            Chirality::Both => {
                let left = self.left_current(fi);
                let right = self.right_current(fi);
                left + right
            }
        }
    }

    /// Axial vector bilinear contraction: `f̄ γ^μ γ^5 Γ f` where `Γ` encodes chirality.
    ///
    /// This can be implemented using the left and right currents, with a relative minus sign
    /// because `γ^5` acts as +1 on right-chiral and -1 on left-chiral components.
    fn axial_vector_bilinear(
        &self,
        fi: &Self::Dual,
        chirality: Chirality,
    ) -> ComplexVector<F, Contravariant>
    where
        Adj: IsBra,
    {
        match chirality {
            Chirality::Left => -self.left_current(fi),
            Chirality::Right => self.right_current(fi),
            Chirality::Both => {
                let left = self.left_current(fi);
                let right = self.right_current(fi);
                right - left
            }
        }
    }

    // TODO: tensor bilinear `f̄ σ^μν Γ f` where `σ^μν = i/2 [γ^μ, γ^ν]` and `Γ` encodes chirality.
}

/// A concrete Spin(1,3) representation: the Weyl basis for Dirac spinors.
///
/// In this basis the 4-component Dirac spinor is arranged as
/// $[ψ_0, ψ_1, χ^0̇, χ^1̇]$ where $ψ_α$ (indices 0–1) are the left-chiral
/// (undotted, transforming under (½,0) representation) Weyl spinor
/// components and $χ^α̇$ (indices 2–3) are the right-chiral (dotted,
/// transfroming under (0,½) representation) components.
///
/// This choice makes the Lorentz decomposition `S = S_L ⊕ S_R` manifest, which
/// is the geometrically natural basis for computing helicity amplitudes.
///
/// The `left_current` and `right_current` implementations match the Fortran
/// HELAS routines `iovxxx` lines 86–89 exactly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bispinor<F: Real, Adj: DiracAdjoint>([C<F>; 4], PhantomData<Adj>);

impl<F: Real, Adj: DiracAdjoint> ArrayBacked<C<F>, 4> for Bispinor<F, Adj> {
    #[inline(always)]
    fn as_array(&self) -> &[C<F>; 4] {
        &self.0
    }

    #[inline(always)]
    fn from_array(arr: [C<F>; 4]) -> Self {
        Bispinor(arr, PhantomData)
    }
}

impl_vectorspace!(impl[F: Real, Adj: DiracAdjoint] Bispinor<F, Adj>, scalar = C<F>);

// Allow scalar multiplication by a real (performance optimization)
impl_mul_for_array!(impl[F: Real, Adj: DiracAdjoint] Bispinor<F, Adj>, scalar = F);

impl<F: Real, Adj: DiracAdjoint> LorentzRepr<F> for Bispinor<F, Adj> {
    type Scalar = C<F>;
}

impl<F: Real, Adj: DiracAdjoint> SpinorRepr<F, Adj> for Bispinor<F, Adj> {
    type Dual = Bispinor<F, Adj::Dual>;

    #[inline(always)]
    fn dualize(&self) -> Self::Dual {
        let arr = self.as_array();
        Bispinor::from_array([arr[2].conj(), arr[3].conj(), arr[0].conj(), arr[1].conj()])
    }

    /// Left projection: zero the right-chiral (indices 2-3) components, keeping left-chiral (0-1).
    #[inline(always)]
    fn project_left(self) -> Self {
        // For either adjoint, the left projection is the same, because P_L doesn't "commute" with Dirac adjoint
        let psi = self.as_array();
        Bispinor::from_array([psi[0], psi[1], C::zero(), C::zero()])
    }

    /// Right projection: zero the left-chiral (indices 0-1) components, keeping right-chiral (2-3).
    #[inline(always)]
    fn project_right(self) -> Self {
        // For either adjoint, the right projection is the same, because P_R doesn't "commute" with Dirac adjoint
        let psi = self.as_array();
        Bispinor::from_array([C::zero(), C::zero(), psi[2], psi[3]])
    }

    /// Apply the gamma-slash `v̸ = γ^μ v_μ`.
    ///
    /// Two orthogonal axes determine the action:
    /// - The **adjoint** picks the side: a ket takes the left action `v̸ψ`, a
    ///   bra the right action `ψ̄v̸` (distinct component formulas — see
    ///   [`DiracAdjoint::slash_bispinor`]).
    /// - **Variance** fixes the contraction. `v̸ = γ^μ v_μ` is a single operator;
    ///   the kernel [`DiracAdjoint::slash_bispinor`] sums the stored components
    ///   against `γ^μ` directly, which equals `γ^μ v_μ` only for a **covariant**
    ///   `v`. A contravariant `v` is lowered first, so the result is the same
    ///   physical operator regardless of how the vector was stored.
    #[inline(always)]
    fn slash<V: Variance>(self, v: &ComplexVector<F, V>) -> Self {
        if V::COVARIANT {
            Bispinor::from_array(Adj::slash_bispinor(&self.0, &v.0))
        } else {
            // Contravariant input: lower to v_μ = g_μν v^ν before contracting.
            Bispinor::from_array(Adj::slash_bispinor(&self.0, &v.dualize().0))
        }
    }

    #[cfg(test)]
    fn bare_norm_sq(self) -> F
    where
        F: std::iter::Sum,
    {
        self.0.iter().map(|x| x.norm_sqr()).sum()
    }

    /// Left current
    ///
    /// Formula: `J_L^μ = (σ̄^μ)_{α̇β} fo[2+α̇] fi[β]`
    /// with `σ̄^0 = I₂`, `σ̄^i = −σ^i`:
    ///
    /// | μ | σ̄^μ | component |
    /// |---|------|-----------|
    /// | 0 | I₂   | `fo[2]·fi[0] + fo[3]·fi[1]` |
    /// | 1 | −σ¹  | `−(fo[2]·fi[1] + fo[3]·fi[0])` |
    /// | 2 | −σ²  | `i(fo[2]·fi[1] − fo[3]·fi[0])` |
    /// | 3 | −σ³  | `−fo[2]·fi[0] + fo[3]·fi[1]` |
    #[inline(always)]
    fn left_current(&self, fi: &Self::Dual) -> ComplexVector<F, Contravariant>
    where
        Adj: IsBra,
    {
        let fo = &self.0;
        let fi = &fi.0;
        // Every component is a sum or difference of the same four products, so
        // they are named once. Folding a product into a multiply-add per
        // component instead would compute each of the four twice, since a fused
        // `a*b + c` is not a common subexpression of a bare `a*b`.
        let a = cmul(fo[2], fi[0]);
        let b = cmul(fo[3], fi[1]);
        let c = cmul(fo[2], fi[1]);
        let d = cmul(fo[3], fi[0]);
        ComplexVector(
            [a + b, -(c + d), ri(F::one()) * (c - d), b - a],
            PhantomData,
        )
    }

    /// Right current
    ///
    /// Formula: `J_R^μ = (σ^μ)^{αβ̇} fo[α] fi[2+β̇]`
    /// with `σ^0 = I₂`, `σ^i = +σ^i`:
    ///
    /// | μ | σ^μ | component |
    /// |---|-----|-----------|
    /// | 0 | I₂  | `fo[0]·fi[2] + fo[1]·fi[3]` |
    /// | 1 | +σ¹ | `fo[0]·fi[3] + fo[1]·fi[2]` |
    /// | 2 | +σ²  | `−i(fo[0]·fi[3] − fo[1]·fi[2])` |
    /// | 3 | +σ³  | `fo[0]·fi[2] − fo[1]·fi[3]` |
    #[inline(always)]
    fn right_current(&self, fi: &Self::Dual) -> ComplexVector<F, Contravariant>
    where
        Adj: IsBra,
    {
        let fo = &self.0;
        let fi = &fi.0;
        // As in `left_current`: name the four products so none is computed twice.
        let a = cmul(fo[0], fi[2]);
        let b = cmul(fo[1], fi[3]);
        let c = cmul(fo[0], fi[3]);
        let d = cmul(fo[1], fi[2]);
        ComplexVector([a + b, c + d, -ri(F::one()) * (c - d), a - b], PhantomData)
    }

    /// Scalar bilinear contraction: `f̄ Γ f` where `Γ` encodes chirality.
    ///
    /// This is the genuine Lorentz scalar `ψ̄ Γ ψ = ψ† γ⁰ Γ ψ`. The Dirac adjoint
    /// `fo` already carries the `γ⁰` block-swap (it is stored as `ψ† γ⁰`), so the
    /// contraction with the ket `fi` is a *plain* component dot — pairing matching
    /// storage indices, NOT a re-swapped one. (Re-swapping would cancel the `γ⁰`
    /// and yield the density `ψ† ψ = J⁰` instead of the Lorentz scalar.)
    ///
    /// With `fo = ψ̄` and `fi = ψ` (indices 0,1 = left-chiral, 2,3 = right-chiral):
    /// - Left (P_L):  `ψ̄ P_L ψ = fo[0]·fi[0] + fo[1]·fi[1]`
    /// - Right (P_R): `ψ̄ P_R ψ = fo[2]·fi[2] + fo[3]·fi[3]`
    /// - Both (Identity): the sum of both.
    #[inline(always)]
    fn scalar_bilinear(&self, fi: &Self::Dual, chirality: Chirality) -> C<F>
    where
        Adj: IsBra,
    {
        let fo = &self.0;
        let fi = &fi.0;
        match chirality {
            Chirality::Left => cmul_add(fo[0], fi[0], cmul(fo[1], fi[1])),
            Chirality::Right => cmul_add(fo[2], fi[2], cmul(fo[3], fi[3])),
            Chirality::Both => {
                let l = cmul_add(fo[0], fi[0], cmul(fo[1], fi[1]));
                cmul_add(fo[2], fi[2], cmul_add(fo[3], fi[3], l))
            }
        }
    }

    /// Pseudoscalar bilinear contraction: `f̄ γ^5 Γ f` where `Γ` encodes chirality.
    /// With `γ^5` acting as +1 on right-chiral and -1 on left-chiral components, this is:
    /// - Left (P_L): `-(ψ̄ P_L ψ) = -(fo[0]·fi[0] + fo[1]·fi[1])`
    /// - Right (P_R): `ψ̄ P_R ψ = fo[2]·fi[2] + fo[3]·fi[3]`
    /// - Both (Identity): `ψ̄ P_R ψ - ψ̄ P_L ψ`
    #[inline(always)]
    fn pseudoscalar_bilinear(&self, fi: &Self::Dual, chirality: Chirality) -> C<F>
    where
        Adj: IsBra,
    {
        match chirality {
            Chirality::Left => -self.scalar_bilinear(fi, Chirality::Left),
            Chirality::Right => self.scalar_bilinear(fi, Chirality::Right),
            Chirality::Both => {
                self.scalar_bilinear(fi, Chirality::Right)
                    - self.scalar_bilinear(fi, Chirality::Left)
            }
        }
    }
}

impl<F: Real, Adj: DiracAdjoint> Bispinor<F, Adj> {
    /// The `i`-th Weyl-basis spinor component (0,1 left-chiral; 2,3 right-chiral).
    ///
    /// This index matches the HELAS/ALOHA 6-component fermion array: `component(k)`
    /// is `F(3+k)`. Exposes the raw basis coordinate, useful for cross-checks.
    #[inline(always)]
    pub fn component(&self, i: usize) -> C<F> {
        self.0[i]
    }

    /// Build a spinor from its four Weyl-basis components (the inverse of
    /// [`component`](Self::component)). The component order is the HELAS/ALOHA
    /// `F(3..6)` layout; mainly useful for cross-checks against reference routines.
    #[inline(always)]
    pub fn from_components(comps: [C<F>; 4]) -> Self {
        Bispinor(comps, PhantomData)
    }

    /// Construct a spinor from a 4-momentum, mass, helicity, and adjoint (bra/ket).
    #[inline(always)]
    pub fn from_momentum(
        p: LorentzVector<F, Contravariant>,
        mass: F,
        nhel: SpinorHelicity,
        nsf: Charge,
    ) -> Self {
        Adj::build_bispinor(p, mass, nhel, nsf)
    }
}

impl<F: Real> Bispinor<F, Ket> {
    /// Bar the spinor to get the bra: `ū = ψ† γ^0`.
    #[inline(always)]
    pub fn bar(self) -> Bispinor<F, Bra> {
        self.dualize()
    }
}

impl<F: Real> Bispinor<F, Bra> {
    /// Unbar the spinor to get the ket: `u = γ^0 ψ̄†`.
    #[inline(always)]
    pub fn unbar(self) -> Bispinor<F, Ket> {
        self.dualize()
    }
}

impl<F: Real + std::fmt::Display> std::fmt::Display for Bispinor<F, Ket> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "|{}, {}, {}, {}>",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

impl<F: Real + std::fmt::Display> std::fmt::Display for Bispinor<F, Bra> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<{}, {}, {}, {}|",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

/// Antisymmetric rank-2 Lorentz tensor (placeholder type).
///
/// Represents a tensor `T^{μν} = -T^{νμ}` such as the output of `σ^μν = i/2 [γ^μ, γ^ν]`.
/// It is the (1,0) ⊕ (0,1) representation of the Lorentz group, which is 6-dimensional.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AsymRank2Tensor<F: Real>(pub [C<F>; 6]);

impl<F: Real> LorentzRepr<F> for AsymRank2Tensor<F> {
    type Scalar = C<F>;
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers: the actual WeylBasis numerics (moved from repr.rs)
// ─────────────────────────────────────────────────────────────────────────────

/// Incoming fermion wavefunction (column spinor).
///
/// Mirrors Fortran `ixxxxx` exactly.
fn weyl_ixxxxx<F: Real>(
    p: LorentzVector<F, Contravariant>,
    mass: F,
    nhel: SpinorHelicity,
    nsf: Charge,
) -> [C<F>; 4] {
    let two = F::one() + F::one();
    let nh = nhel.sign() * nsf.sign();
    let nsf_i = nsf.sign();

    let mut fi = [C::new(F::zero(), F::zero()); 4];

    if mass != F::zero() {
        let pp = p.p3().min(p.e());

        if pp == F::zero() {
            // ── at rest ───────────────────────────────────────────────────
            let sqm0 = mass.abs().sqrt();
            let sqm1 = sqm0 * mass.signum();
            let sqm = [sqm0, sqm1];

            let ip_i = (1 + nh) / 2;
            let im_i = (1 - nh) / 2;
            let ip = ip_i as usize;
            let im = im_i as usize;

            fi[0] = r(F::from(ip_i).unwrap() * sqm[ip]);
            fi[1] = r(F::from(im_i * nsf_i).unwrap() * sqm[ip]);
            fi[2] = r(F::from(ip_i * nsf_i).unwrap() * sqm[im]);
            fi[3] = r(F::from(im_i).unwrap() * sqm[im]);
        } else {
            // ── massive, moving ───────────────────────────────────────────
            let sf = [
                F::from(1 + nsf_i + (1 - nsf_i) * nh).unwrap() / two,
                F::from(1 + nsf_i - (1 - nsf_i) * nh).unwrap() / two,
            ];
            let omega0 = (p.e() + pp).sqrt();
            let omega = [omega0, mass / omega0];

            let ip = ((1 + nh) / 2) as usize;
            let im = ((1 - nh) / 2) as usize;

            let sfomeg = [r(sf[0] * omega[ip]), r(sf[1] * omega[im])];

            let pp3 = (pp + p.pz()).max(F::zero());
            let chi0 = r((pp3 / (two * pp)).sqrt());
            let chi1 = if pp3 > F::zero() {
                C::new(F::from(nh).unwrap() * p.px(), p.py()) / r((two * pp * pp3).sqrt())
            } else {
                r(F::from(-nh).unwrap())
            };
            let chi = [chi0, chi1];

            fi[0] = sfomeg[0] * chi[im];
            fi[1] = sfomeg[0] * chi[ip];
            fi[2] = sfomeg[1] * chi[im];
            fi[3] = sfomeg[1] * chi[ip];
        }
    } else {
        // ── massless ──────────────────────────────────────────────────────
        let sqp0p3 = if p.px() == F::zero() && p.py() == F::zero() && p.pz() < F::zero() {
            F::zero()
        } else {
            (p.e() + p.pz()).max(F::zero()).sqrt() * F::from(nsf_i).unwrap()
        };
        let chi0 = r(sqp0p3);
        let chi1 = if sqp0p3 == F::zero() {
            r(F::from(-nhel.sign()).unwrap() * (two * p.e()).sqrt())
        } else {
            C::new(F::from(nh).unwrap() * p.px(), p.py()) / r(sqp0p3)
        };

        if nh == 1 {
            fi[0] = C::zero();
            fi[1] = C::zero();
            fi[2] = chi0;
            fi[3] = chi1;
        } else {
            fi[0] = chi1;
            fi[1] = chi0;
            fi[2] = C::zero();
            fi[3] = C::zero();
        }
    }

    fi
}

#[cfg(test)]
mod tests {
    use std::array;

    use itertools::{iproduct, Itertools};
    use num_traits::One;

    use super::*;

    /// Absolute tolerance for floating-point comparisons in these tests.
    const EPS_ABS: f64 = 1e-12;

    /// Generate a variety of momenta test cases
    ///
    /// Exercises all 3 axes, massive and massless
    fn momenta_test_cases() -> impl Iterator<Item = (LorentzVector<f64>, f64)> {
        let momenta = [
            LorentzVector::from_pxpypzmass(1.0, 0.0, 0.0, 0.0),
            LorentzVector::from_pxpypzmass(0.0, 1.0, 0.0, 0.0),
            LorentzVector::from_pxpypzmass(0.0, 0.0, 1.0, 0.0),
            LorentzVector::from_pxpypzmass(1.0, 0.0, 0.0, 0.5),
            LorentzVector::from_pxpypzmass(0.0, 1.0, 0.0, 0.5),
            LorentzVector::from_pxpypzmass(0.0, 0.0, 1.0, 0.5),
            LorentzVector::from_pxpypzmass(1.0, 2.0, 3.0, 0.0),
            LorentzVector::from_pxpypzmass(1.0, 2.0, 3.0, 0.5),
        ];
        let masses = [0.0, 0.5, 1.0];
        iproduct!(momenta, masses)
    }

    /// Generate a variety of test cases for spinor bilinears and currents.
    ///
    /// On-shell and off-shell, all helicity and charge combinations.
    fn spinor_test_cases() -> impl Iterator<Item = (LorentzVector<f64>, f64, SpinorHelicity, Charge)>
    {
        let helicities = [SpinorHelicity::Up, SpinorHelicity::Down];
        let charges = [Charge::Particle, Charge::Antiparticle];

        iproduct!(momenta_test_cases(), helicities, charges).map(|((p, m), h, c)| (p, m, h, c))
    }

    #[test]
    fn test_scalar_bilinear_norm() {
        for (p, mass, nhel, nsf) in spinor_test_cases() {
            let psi = Bispinor::from_momentum(p, mass, nhel, nsf);
            let psibar = psi.bar();
            let bilinear = psibar.scalar_bilinear(&psi, Chirality::Both);

            // ψ̄ψ = ψ† γ⁰ ψ is real (γ⁰ is Hermitian), on or off shell.
            assert!(
                bilinear.im.abs() < EPS_ABS,
                "Scalar bilinear ψ̄ψ should be real, got {}",
                bilinear
            );

            if (p.m2() - mass * mass).abs() < EPS_ABS {
                // On-shell spinors

                // The genuine Lorentz scalar is ψ̄ψ = 2 m · nsf  (u → +2m, v → −2m;
                // zero for massless).
                let expected = 2.0 * mass * nsf.sign() as f64;
                assert!(
                    (bilinear - expected).norm() < EPS_ABS,
                    "On-shell scalar bilinear mismatch: ψ̄ψ = {}, 2m·nsf = {}",
                    bilinear,
                    expected
                );

                // Satisfy Dirac equation
                let dirac_in =
                    psi.slash(&ComplexVector::from(p)) - psi * (nsf.sign() as f64 * mass);
                let norm_sq = psibar.scalar_bilinear(&dirac_in, Chirality::Both);
                assert!(
                    norm_sq.norm() < EPS_ABS,
                    "Spinor does not satisfy Dirac equation: (p̸ ± m) ψ has norm squared = {}",
                    norm_sq
                );
                let dirac_out =
                    psibar.slash(&ComplexVector::from(p)) - psibar * (nsf.sign() as f64 * mass);
                let norm_sq = dirac_out.scalar_bilinear(&psi, Chirality::Both);
                assert!(
                    norm_sq.norm() < EPS_ABS,
                    "Spinor does not satisfy Dirac equation: ψ̄ (p̸ ± m) has norm squared = {}",
                    norm_sq
                );
            }
        }
    }

    /// Test spinor compleness relations
    ///
    /// We expect the following completeness relations to hold for the spinors:
    /// - For ket spinors: `∑_h u(p, h) ū(p, h) = p̸ + m`
    /// - For bra spinors: `∑_h v(p, h) v̄(p, h) = p̸ - m`
    ///
    /// We can test these using Fierz identities (i.e. use the bilinears to
    /// project the completeness relation onto a basis of gamma matrices and
    /// check that the coefficients match those of `p̸ ± m`). Note the Fierz
    /// identity rewrite introduces a factor of 4 to all terms.
    #[test]
    fn test_completeness_relations() {
        for (p, mass) in momenta_test_cases() {
            if (p.m2() - mass * mass).abs() > EPS_ABS {
                // Only valid for on-shell spinors
                continue;
            }
            let up = Bispinor::from_momentum(p, mass, SpinorHelicity::Up, Charge::Particle);
            let um = Bispinor::from_momentum(p, mass, SpinorHelicity::Down, Charge::Particle);
            let vp = Bispinor::from_momentum(p, mass, SpinorHelicity::Up, Charge::Antiparticle);
            let vm = Bispinor::from_momentum(p, mass, SpinorHelicity::Down, Charge::Antiparticle);

            // Helicity-summed scalar bilinears are the traces of the completeness
            // relations: Σ_h ū u = Tr[p̸ + m] = 4m and Σ_h v̄ v = Tr[p̸ − m] = −4m.
            let u_scalar = up.bar().scalar_bilinear(&up, Chirality::Both)
                + um.bar().scalar_bilinear(&um, Chirality::Both);
            let v_scalar = vp.bar().scalar_bilinear(&vp, Chirality::Both)
                + vm.bar().scalar_bilinear(&vm, Chirality::Both);
            assert!(
                (u_scalar - 4.0 * mass).norm() < EPS_ABS,
                "Fermion scalar bilinear does not match: u_scalar = {}, 4m = {}",
                u_scalar,
                4.0 * mass
            );
            assert!(
                (v_scalar + 4.0 * mass).norm() < EPS_ABS,
                "Antifermion scalar bilinear does not match: v_scalar = {}, -4m = {}",
                v_scalar,
                -4.0 * mass
            );

            // The vector bilinear should give 4x the momentum term
            let u_vector = up.bar().vector_bilinear(&up, Chirality::Both)
                + um.bar().vector_bilinear(&um, Chirality::Both);
            let v_vector = vp.bar().vector_bilinear(&vp, Chirality::Both)
                + vm.bar().vector_bilinear(&vm, Chirality::Both);
            let expected = ComplexVector::from(p * 4.0);
            assert!(
                (u_vector - expected).bare_norm_sq() < EPS_ABS,
                "Fermion vector bilinear does not match: u_vector = {}, 4p = {}",
                u_vector,
                expected
            );
            assert!(
                (v_vector - expected).bare_norm_sq() < EPS_ABS,
                "Antifermion vector bilinear does not match: v_vector = {}, 4p = {}",
                v_vector,
                expected
            );

            // TODO: when tensor bilinears are implemented, test those as well to fully check the completeness relations.

            // The axial vector bilinear should vanish for the helicity-summed spinors
            let u_axial_vector = up.bar().axial_vector_bilinear(&up, Chirality::Both)
                + um.bar().axial_vector_bilinear(&um, Chirality::Both);
            let v_axial_vector = vp.bar().axial_vector_bilinear(&vp, Chirality::Both)
                + vm.bar().axial_vector_bilinear(&vm, Chirality::Both);
            assert!(
                u_axial_vector.bare_norm_sq() < EPS_ABS,
                "Fermion axial vector bilinear nonzero for helicity-summed spinors: u_axial_vector = {}",
                u_axial_vector
            );
            assert!(
                v_axial_vector.bare_norm_sq() < EPS_ABS,
                "Antifermion axial vector bilinear nonzero for helicity-summed spinors: v_axial_vector = {}",
                v_axial_vector
            );

            // The pseudoscalar bilinear should also vanish for the helicity-summed spinors
            let u_pseudoscalar = up.bar().pseudoscalar_bilinear(&up, Chirality::Both)
                + um.bar().pseudoscalar_bilinear(&um, Chirality::Both);
            let v_pseudoscalar = vp.bar().pseudoscalar_bilinear(&vp, Chirality::Both)
                + vm.bar().pseudoscalar_bilinear(&vm, Chirality::Both);
            assert!(
                u_pseudoscalar.norm() < EPS_ABS,
                "Fermion pseudoscalar bilinear nonzero for helicity-summed spinors: u_pseudoscalar = {}",
                u_pseudoscalar
            );
            assert!(
                v_pseudoscalar.norm() < EPS_ABS,
                "Antifermion pseudoscalar bilinear nonzero for helicity-summed spinors: v_pseudoscalar = {}",
                v_pseudoscalar
            );
        }
    }

    /// Test spinor projectors for consistency between ket and bra adjoints and with the currents.
    #[test]
    fn test_spinor_projectors() {
        let in_cases = spinor_test_cases()
            .map(|(p, mass, nhel, nsf)| Bispinor::<_, Ket>::from_momentum(p, mass, nhel, nsf))
            .collect_vec();
        let out_cases = spinor_test_cases()
            .map(|(p, mass, nhel, nsf)| Bispinor::<_, Bra>::from_momentum(p, mass, nhel, nsf))
            .collect_vec();
        for (psi_in, psi_out) in iproduct!(in_cases, out_cases) {
            // Projectors should flip with the adjoint reversal operation (bar/unbar)
            assert_eq!(psi_in.project_left().bar(), psi_in.bar().project_right());
            assert_eq!(psi_in.project_right().bar(), psi_in.bar().project_left());

            assert_eq!(
                psi_out.project_left().unbar(),
                psi_out.unbar().project_right()
            );
            assert_eq!(
                psi_out.project_right().unbar(),
                psi_out.unbar().project_left()
            );

            // Test that the spinor projectors are consistent with the currents
            let left_current = psi_out.left_current(&psi_in);
            let right_current = psi_out.right_current(&psi_in);
            // on the outgoing, we have opposite projection
            let left_current_proj = psi_out
                .project_right()
                .vector_bilinear(&psi_in, Chirality::Both);
            let right_current_proj = psi_out
                .project_left()
                .vector_bilinear(&psi_in, Chirality::Both);
            assert_eq!(left_current, left_current_proj);
            assert_eq!(right_current, right_current_proj);
            // in the incoming, we have the given projection
            let left_current_proj =
                psi_out.vector_bilinear(&psi_in.project_left(), Chirality::Both);
            let right_current_proj =
                psi_out.vector_bilinear(&psi_in.project_right(), Chirality::Both);
            assert_eq!(left_current, left_current_proj);
            assert_eq!(right_current, right_current_proj);

            // Slash is consistent with the currents: `ψ̄ v̸ ψ = v_μ J^μ`, where
            // `J^μ = ψ̄ γ^μ ψ` is the (contravariant) vector bilinear. Probing with
            // each contravariant basis vector `e_basis`, the variance-aware slash
            // lowers it, so the genuine scalar contraction returns `v_μ J^μ` (the
            // metric-contracted current component), not the raw `J^basis`.
            let current = psi_out.vector_bilinear(&psi_in, Chirality::Both);
            for basis in 0..4 {
                let v: ComplexVector<f64, Contravariant> =
                    ComplexVector::from_array(array::from_fn(|i| {
                        if i == basis {
                            C::one()
                        } else {
                            C::zero()
                        }
                    }));
                let expected = v.lower().dot(&current); // v_μ J^μ

                // Ket path: ψ̄ (v̸ ψ)
                let slash_in = psi_out.scalar_bilinear(&psi_in.slash(&v), Chirality::Both);
                assert!(
                    (slash_in - expected).norm() < EPS_ABS,
                    "Ket slash inconsistent with vector bilinear: ψ̄ v̸ ψ = {}, v_μ J^μ = {} (basis {})",
                    slash_in,
                    expected,
                    basis
                );
                // Bra path: (ψ̄ v̸) ψ — must agree with the ket path.
                let slash_out = psi_out.slash(&v).scalar_bilinear(&psi_in, Chirality::Both);
                assert!(
                    (slash_out - expected).norm() < EPS_ABS,
                    "Bra slash inconsistent with vector bilinear: ψ̄ v̸ ψ = {}, v_μ J^μ = {} (basis {})",
                    slash_out,
                    expected,
                    basis
                );
            }
        }
    }
}
