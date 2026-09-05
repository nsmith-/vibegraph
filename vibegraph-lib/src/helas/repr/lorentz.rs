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

/// Multiplication by `i`, as a component swap rather than a complex product.
#[inline(always)]
fn mul_i<F: Real>(z: C<F>) -> C<F> {
    C::new(-z.im, z.re)
}

/// Multiplication by `−i`, as a component swap rather than a complex product.
#[inline(always)]
fn mul_neg_i<F: Real>(z: C<F>) -> C<F> {
    C::new(z.im, -z.re)
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

    /// Apply a 4×4 Weyl-basis operator to a bispinor of this adjoint.
    ///
    /// The open spinor index sits on a different side of the operator for each
    /// adjoint, so a ket takes the column action `M ψ` and a bra the row action
    /// `ψ̄ M` — the transpose of the ket action, which is what the bra's storage
    /// of the row vector `ψ† γ⁰` calls for.
    fn apply_weyl_matrix<F: Real>(psi: &[C<F>; 4], m: &[[C<F>; 4]; 4]) -> [C<F>; 4];
}

/// Contract one length-4 complex row against a length-4 complex column.
#[inline(always)]
fn dot4<F: Real>(a: [C<F>; 4], b: [C<F>; 4]) -> C<F> {
    cmul_add(
        a[0],
        b[0],
        cmul_add(a[1], b[1], cmul_add(a[2], b[2], cmul(a[3], b[3]))),
    )
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

    /// Ket column action `(M ψ)_i = M_{ij} ψ_j`.
    #[inline(always)]
    fn apply_weyl_matrix<F: Real>(psi: &[C<F>; 4], m: &[[C<F>; 4]; 4]) -> [C<F>; 4] {
        std::array::from_fn(|i| dot4(m[i], *psi))
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

    /// Bra row action `(ψ̄ M)_j = ψ̄_i M_{ij}`.
    #[inline(always)]
    fn apply_weyl_matrix<F: Real>(psi: &[C<F>; 4], m: &[[C<F>; 4]; 4]) -> [C<F>; 4] {
        std::array::from_fn(|j| dot4(*psi, [m[0][j], m[1][j], m[2][j], m[3][j]]))
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

    /// Tensor bilinear contraction: `f̄ σ^{μν} Γ f` where `σ^{μν} = (i/2)[γ^μ, γ^ν]`
    /// and `Γ` encodes chirality.
    ///
    /// The result holds the contravariant `T^{μν}` in [`AsymRank2Tensor`]'s
    /// six-slot order. `σ^{μν}` commutes with `γ^5`, so the chiral projector may be
    /// read on either side of it.
    fn tensor_bilinear(&self, fi: &Self::Dual, chirality: Chirality) -> AsymRank2Tensor<F>
    where
        Adj: IsBra;

    /// All sixteen bilinears `f̄ Γ_A f` of a fermion line at once: the line's
    /// current in the graded Dirac basis.
    ///
    /// The five grades are exactly the scalar, vector, tensor, axial-vector and
    /// pseudoscalar bilinears with no relative normalisation — that is what
    /// [`Multivector`]'s choice of `γ^5 γ^μ` for its grade-3 basis element buys.
    /// Two identities fix the overall normalisation, both pinned in this module's
    /// tests:
    ///
    /// - the outer product is `ψ ψ̄ = ¼ Σ_A (ψ̄ Γ_A ψ) Γ^A`, i.e. a quarter of this
    ///   multivector read as an operator;
    /// - `ψ̄ M ψ = ⟨fierz_coefficients(ψ̄, ψ), M⟩` for every Clifford element `M`,
    ///   with `⟨·,·⟩` the Fierz pairing [`Multivector::fierz_pairing`].
    #[inline]
    fn fierz_coefficients(&self, fi: &Self::Dual) -> Multivector<F>
    where
        Adj: IsBra,
    {
        Multivector::new(
            self.scalar_bilinear(fi, Chirality::Both),
            self.vector_bilinear(fi, Chirality::Both),
            self.tensor_bilinear(fi, Chirality::Both),
            self.axial_vector_bilinear(fi, Chirality::Both),
            self.pseudoscalar_bilinear(fi, Chirality::Both),
        )
    }

    /// Apply a Clifford-algebra element: `M ψ` on a ket, `ψ̄ M` on a bra.
    ///
    /// This is the general form of [`slash`](Self::slash), which is the grade-1
    /// case: `psi.apply(&Multivector::from_gamma(&v)) == psi.slash(&v)`.
    fn apply(self, m: &Multivector<F>) -> Self;
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

    /// Tensor bilinear contraction `f̄ σ^{μν} Γ f`.
    ///
    /// `σ^{μν}` is block diagonal in the Weyl basis, so a chiral projector simply
    /// selects one block. Writing `L^a` and `R^a` for the two Pauli sandwiches
    /// `fo_i (σ^a)_{ij} fi_j` over the left-chiral (0,1) and right-chiral (2,3)
    /// index pairs, the blocks are `σ^{0i} = diag(−i σ^i, +i σ^i)` and
    /// `σ^{jk} = diag(ε^{jkl} σ^l, ε^{jkl} σ^l)`, giving
    /// `T^{0i} = i(R^i − L^i)`, `T^{12} = L³+R³`, `T^{13} = −(L²+R²)`,
    /// `T^{23} = L¹+R¹`.
    #[inline]
    fn tensor_bilinear(&self, fi: &Self::Dual, chirality: Chirality) -> AsymRank2Tensor<F>
    where
        Adj: IsBra,
    {
        let fo = &self.0;
        let fi = &fi.0;
        // ⟨(x,y)| σ^a |(z,w)⟩ for a = 1,2,3, the two-component Pauli sandwich.
        let pauli_sandwich = |x: C<F>, y: C<F>, z: C<F>, w: C<F>| -> [C<F>; 3] {
            let xw = cmul(x, w);
            let yz = cmul(y, z);
            [xw + yz, mul_i(yz - xw), cmul(x, z) - cmul(y, w)]
        };
        let left = match chirality {
            Chirality::Right => [C::zero(); 3],
            _ => pauli_sandwich(fo[0], fo[1], fi[0], fi[1]),
        };
        let right = match chirality {
            Chirality::Left => [C::zero(); 3],
            _ => pauli_sandwich(fo[2], fo[3], fi[2], fi[3]),
        };
        let sum: [C<F>; 3] = std::array::from_fn(|i| left[i] + right[i]);
        AsymRank2Tensor::new([
            mul_i(right[0] - left[0]),
            mul_i(right[1] - left[1]),
            mul_i(right[2] - left[2]),
            sum[2],
            -sum[1],
            sum[0],
        ])
    }

    #[inline]
    fn apply(self, m: &Multivector<F>) -> Self {
        Bispinor::from_array(Adj::apply_weyl_matrix(&self.0, &m.to_weyl_matrix()))
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

// ─────────────────────────────────────────────────────────────────────────────
// Levi-Civita primitives
// ─────────────────────────────────────────────────────────────────────────────

/// The four cofactors `C_τ = ε_{μνρτ} a^μ b^ν c^ρ` of the last row: the
/// covariant components of [`epsilon_vector`] before the index is raised.
fn epsilon_cofactors<F: Real>(
    a: &ComplexVector<F, Contravariant>,
    b: &ComplexVector<F, Contravariant>,
    c: &ComplexVector<F, Contravariant>,
) -> [C<F>; 4] {
    // det of the 3×3 built from rows a, b, c on columns (i, j, k).
    let minor = |i: usize, j: usize, k: usize| -> C<F> {
        cmul(a.0[i], cmul(b.0[j], c.0[k]) - cmul(b.0[k], c.0[j]))
            - cmul(a.0[j], cmul(b.0[i], c.0[k]) - cmul(b.0[k], c.0[i]))
            + cmul(a.0[k], cmul(b.0[i], c.0[j]) - cmul(b.0[j], c.0[i]))
    };
    [
        -minor(1, 2, 3),
        minor(0, 2, 3),
        -minor(0, 1, 3),
        minor(0, 1, 2),
    ]
}

/// Fully contracted Levi-Civita symbol `ε_{μνρσ} a^μ b^ν c^ρ d^σ`.
///
/// # Convention
///
/// `ε^{0123} = −1`, equivalently `ε_{0123} = +1`. This is ALOHA's convention:
/// `aloha/aloha_object.py::L_Epsilon.give_parity` stores the component
/// `(l1,l2,l3,l4)` of the upper-index object as `−sign(perm)` and applies the
/// metric at contraction time. With four contravariant arguments the value is
/// therefore `+1` on `(e₀, e₁, e₂, e₃)`, i.e. the determinant of the matrix whose
/// rows are the arguments in the `[E, px, py, pz]` layout.
///
/// The sign is a hypothesis about MadGraph until an MG comparison exercises it;
/// what is *not* a hypothesis is that everything in this module — the Hodge dual
/// and the `σ^{μν} γ^5` identity included — is derived from the value stated here,
/// so a single flip propagates consistently.
pub fn epsilon4<F: Real>(
    a: &ComplexVector<F, Contravariant>,
    b: &ComplexVector<F, Contravariant>,
    c: &ComplexVector<F, Contravariant>,
    d: &ComplexVector<F, Contravariant>,
) -> C<F> {
    dot4(epsilon_cofactors(a, b, c), d.0)
}

/// The contravariant vector `E^σ = ε^{μνρσ} a_μ b_ν c_ρ`.
///
/// Characterised by `E·d = epsilon4(a, b, c, d)` for every `d`, with `·` the
/// Minkowski contraction — which is what makes it composable as a three-vectors-in,
/// one-vector-out current. Same convention as [`epsilon4`].
pub fn epsilon_vector<F: Real>(
    a: &ComplexVector<F, Contravariant>,
    b: &ComplexVector<F, Contravariant>,
    c: &ComplexVector<F, Contravariant>,
) -> ComplexVector<F, Contravariant> {
    let cof = epsilon_cofactors(a, b, c);
    ComplexVector::new([cof[0], -cof[1], -cof[2], -cof[3]])
}

// ─────────────────────────────────────────────────────────────────────────────
// Grade-2 slice: the antisymmetric rank-2 tensor
// ─────────────────────────────────────────────────────────────────────────────

/// Antisymmetric rank-2 Lorentz tensor `T^{μν} = −T^{νμ}`.
///
/// This is the six-dimensional `(1,0) ⊕ (0,1)` representation, and the grade-2
/// slice of [`Multivector`] — the shape of `σ^{μν} = (i/2)[γ^μ, γ^ν]`.
///
/// # Component order
///
/// The six stored slots are the **contravariant** components `T^{μν}` for the
/// index pairs `μ < ν` in lexicographic order:
///
/// | slot | 0 | 1 | 2 | 3 | 4 | 5 |
/// |------|---|---|---|---|---|---|
/// | `(μ,ν)` | (0,1) | (0,2) | (0,3) | (1,2) | (1,3) | (2,3) |
///
/// [`get`](Self::get) reads any `(μ,ν)` with the antisymmetry applied. Read as a
/// Clifford element the tensor is `Σ_{μ<ν} T^{μν} σ_{μν} = ½ T^{μν} σ_{μν}`, so
/// `σ^{μν}` for one index pair is the tensor with a single unit slot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AsymRank2Tensor<F: Real>([C<F>; 6]);

impl<F: Real> ArrayBacked<C<F>, 6> for AsymRank2Tensor<F> {
    #[inline(always)]
    fn as_array(&self) -> &[C<F>; 6] {
        &self.0
    }

    #[inline(always)]
    fn from_array(arr: [C<F>; 6]) -> Self {
        AsymRank2Tensor(arr)
    }
}

impl_vectorspace!(impl[F: Real] AsymRank2Tensor<F>, scalar = C<F>);
impl_mul_for_array!(impl[F: Real] AsymRank2Tensor<F>, scalar = F);

impl<F: Real> LorentzRepr<F> for AsymRank2Tensor<F> {
    type Scalar = C<F>;
}

impl<F: Real> AsymRank2Tensor<F> {
    /// The `(μ, ν)` index pair held in each of the six slots.
    pub const INDEX_PAIRS: [(usize, usize); 6] = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];

    /// Build from the six `μ < ν` components, in [`INDEX_PAIRS`](Self::INDEX_PAIRS) order.
    #[inline(always)]
    pub fn new(components: [C<F>; 6]) -> Self {
        AsymRank2Tensor(components)
    }

    /// The six stored components, in [`INDEX_PAIRS`](Self::INDEX_PAIRS) order.
    #[inline(always)]
    pub fn components(&self) -> &[C<F>; 6] {
        &self.0
    }

    /// The `i`-th stored component.
    #[inline(always)]
    pub fn component(&self, i: usize) -> C<F> {
        self.0[i]
    }

    /// Slot holding `T^{μν}` for `μ < ν`.
    #[inline(always)]
    fn slot(mu: usize, nu: usize) -> usize {
        mu * (5 - mu) / 2 + nu - 1
    }

    /// `T^{μν}` for any index pair, with the antisymmetry applied:
    /// `T^{νμ} = −T^{μν}` and `T^{μμ} = 0`.
    #[inline]
    pub fn get(&self, mu: usize, nu: usize) -> C<F> {
        match mu.cmp(&nu) {
            std::cmp::Ordering::Equal => C::zero(),
            std::cmp::Ordering::Less => self.0[Self::slot(mu, nu)],
            std::cmp::Ordering::Greater => -self.0[Self::slot(nu, mu)],
        }
    }

    /// The antisymmetrised outer product `T^{μν} = a^μ b^ν − a^ν b^μ`.
    #[inline]
    pub fn wedge(a: &ComplexVector<F, Contravariant>, b: &ComplexVector<F, Contravariant>) -> Self {
        AsymRank2Tensor(std::array::from_fn(|s| {
            let (mu, nu) = Self::INDEX_PAIRS[s];
            cmul(a.0[mu], b.0[nu]) - cmul(a.0[nu], b.0[mu])
        }))
    }

    /// Full contraction with another tensor: `T^{μν} S_{μν}`.
    ///
    /// Summing over all `μν` is twice the sum over `μ < ν`, and the two metric
    /// factors give `g_{00} g_{ii} = −1` on the three `(0,i)` slots and `+1` on the
    /// three spatial ones.
    #[inline]
    pub fn contract(&self, other: &Self) -> C<F> {
        let s = &other.0;
        let t = &self.0;
        let spatial = cmul_add(t[3], s[3], cmul_add(t[4], s[4], cmul(t[5], s[5])));
        let boost = cmul_add(t[0], s[0], cmul_add(t[1], s[1], cmul(t[2], s[2])));
        (spatial - boost) * (F::one() + F::one())
    }

    /// Lower (equivalently raise) both indices: `T_{μν} = g_{μα} g_{νβ} T^{αβ}`.
    /// Only the three `(0,i)` slots change sign.
    #[inline]
    pub fn dualize(&self) -> Self {
        let t = &self.0;
        AsymRank2Tensor([-t[0], -t[1], -t[2], t[3], t[4], t[5]])
    }

    /// Hodge dual `(⋆T)^{μν} = ½ ε^{μνρσ} T_{ρσ}` at [`epsilon4`]'s convention
    /// `ε^{0123} = −1`.
    ///
    /// Writing the tensor as the boost-like `k^i = T^{0i}` and rotation-like
    /// `m^i = ½ ε^{ijk} T^{jk}` three-vectors, the dual is `(k, m) ↦ (−m, k)`, so
    /// `⋆⋆ = −1` as it must in Lorentzian signature. On the Weyl blocks of the
    /// corresponding Clifford element it acts as `−i` on the left-chiral block and
    /// `+i` on the right-chiral one — the (anti-)self-dual split. Flipping the ε
    /// convention would exchange the two blocks' eigenvalues.
    #[inline]
    pub fn hodge_dual(&self) -> Self {
        let t = &self.0;
        AsymRank2Tensor([-t[5], t[4], -t[3], t[2], -t[1], t[0]])
    }

    /// `T^{μν} a_μ b_ν`, lowering the two contravariant arguments here.
    #[inline]
    pub fn contract_vectors(
        &self,
        a: &ComplexVector<F, Contravariant>,
        b: &ComplexVector<F, Contravariant>,
    ) -> C<F> {
        let al = a.dualize();
        let bl = b.dualize();
        Self::INDEX_PAIRS
            .iter()
            .enumerate()
            .fold(C::zero(), |acc, (s, &(mu, nu))| {
                let anti = cmul(al.0[mu], bl.0[nu]) - cmul(al.0[nu], bl.0[mu]);
                cmul_add(self.0[s], anti, acc)
            })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// The graded Clifford element
// ─────────────────────────────────────────────────────────────────────────────

/// 2×2 block `α I₂ + u⃗·σ⃗` in the Pauli basis.
#[inline(always)]
fn pauli_block<F: Real>(alpha: C<F>, u: [C<F>; 3]) -> [[C<F>; 2]; 2] {
    let off = mul_i(u[1]);
    [[alpha + u[2], u[0] - off], [u[0] + off, alpha - u[2]]]
}

/// Inverse of [`pauli_block`]: recover `(α, u⃗)` from a 2×2 block.
#[inline(always)]
fn unpauli_block<F: Real>(blk: &[[C<F>; 2]; 2]) -> (C<F>, [C<F>; 3]) {
    let half = F::one() / (F::one() + F::one());
    (
        (blk[0][0] + blk[1][1]) * half,
        [
            (blk[0][1] + blk[1][0]) * half,
            mul_i(blk[0][1] - blk[1][0]) * half,
            (blk[0][0] - blk[1][1]) * half,
        ],
    )
}

/// An element of the complexified spacetime Clifford algebra `Cl(1,3) ⊗ ℂ`, held
/// in the graded Dirac basis `1 + 4 + 6 + 4 + 1`.
///
/// Every 4×4 spinor-space operator is a unique combination of sixteen basis
/// elements, which this type stores by grade:
///
/// | grade | basis element | coefficient | slots |
/// |-------|---------------|-------------|-------|
/// | 0 | `1` | `s` | 0 |
/// | 1 | `γ_μ` | `v^μ` | 1–4 |
/// | 2 | `σ_{μν}` (`μ<ν`) | `T^{μν}` | 5–10 |
/// | 3 | `γ^5 γ_μ` | `a^μ` | 11–14 |
/// | 4 | `γ^5` | `p` | 15 |
///
/// so that `M = s·1 + v^μ γ_μ + ½ T^{μν} σ_{μν} + a^μ γ^5 γ_μ + p γ^5`. Coefficients
/// carry the index variance opposite to their basis element, so every grade's
/// contraction is the plain Minkowski one and no grade needs a raise at use.
///
/// The point of the graded basis is that `γ^μ γ^ν = g^{μν} − i σ^{μν}` puts a
/// γγ-chain in grades 0 and 2 alone, a chiral projector moves weight between the
/// even grades and between the odd ones, and the antisymmetric rank-2 tensor a
/// fermion line produces is the grade-2 slice rather than something extracted from
/// sixteen `(μ,ν)` components.
///
/// # Why `γ^5 γ^μ` and not `γ^μ γ^5`
///
/// The two orderings differ by a sign, and this one is what makes
/// [`SpinorRepr::fierz_coefficients`] the five bilinears with no sign fixups: for
/// `X = ψ ψ̄`, the coefficient of `γ^5 γ_ν` is `¼ Tr[X γ^ν γ^5] = ¼ (ψ̄ γ^ν γ^5 ψ)`,
/// where in the `γ^μ γ^5` basis it would be its negative. The sign is intrinsic to
/// the grade and has to live somewhere: here it sits in the pairing
/// ([`fierz_pairing`](Self::fierz_pairing)), whose grade-3 term is `−a_M·a_N`
/// because `¼ Tr[γ^5 γ_μ γ^5 γ_ν] = −g_{μν}`.
///
/// # Grade parity and chirality
///
/// In the Weyl basis the even grades (0, 2, 4) are block diagonal and the odd
/// grades (1, 3) are block off-diagonal, so even grades preserve a spinor's chiral
/// blocks and odd grades swap them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Multivector<F: Real>([C<F>; 16]);

impl<F: Real> ArrayBacked<C<F>, 16> for Multivector<F> {
    #[inline(always)]
    fn as_array(&self) -> &[C<F>; 16] {
        &self.0
    }

    #[inline(always)]
    fn from_array(arr: [C<F>; 16]) -> Self {
        Multivector(arr)
    }
}

impl_vectorspace!(impl[F: Real] Multivector<F>, scalar = C<F>);
impl_mul_for_array!(impl[F: Real] Multivector<F>, scalar = F);

impl<F: Real> LorentzRepr<F> for Multivector<F> {
    type Scalar = C<F>;
}

impl<F: Real> Multivector<F> {
    /// First slot of the grade-1 (vector) coefficients.
    const VECTOR: usize = 1;
    /// First slot of the grade-2 (bivector) coefficients.
    const BIVECTOR: usize = 5;
    /// First slot of the grade-3 (axial-vector) coefficients.
    const AXIAL: usize = 11;
    /// Slot of the grade-4 (pseudoscalar) coefficient.
    const PSEUDOSCALAR: usize = 15;

    /// Assemble from the five graded parts.
    #[inline]
    pub fn new(
        scalar: C<F>,
        vector: ComplexVector<F, Contravariant>,
        bivector: AsymRank2Tensor<F>,
        axial: ComplexVector<F, Contravariant>,
        pseudoscalar: C<F>,
    ) -> Self {
        let mut c = [C::zero(); 16];
        c[0] = scalar;
        c[Self::VECTOR..Self::BIVECTOR].copy_from_slice(vector.as_array());
        c[Self::BIVECTOR..Self::AXIAL].copy_from_slice(bivector.as_array());
        c[Self::AXIAL..Self::PSEUDOSCALAR].copy_from_slice(axial.as_array());
        c[Self::PSEUDOSCALAR] = pseudoscalar;
        Multivector(c)
    }

    /// Grade-0 coefficient `s`.
    #[inline(always)]
    pub fn scalar(&self) -> C<F> {
        self.0[0]
    }

    /// Grade-1 coefficients `v^μ` (contravariant, contracted against `γ_μ`).
    #[inline(always)]
    pub fn vector(&self) -> ComplexVector<F, Contravariant> {
        ComplexVector::from_array(std::array::from_fn(|i| self.0[Self::VECTOR + i]))
    }

    /// Grade-2 coefficients `T^{μν}` (contracted against `σ_{μν}`, `μ < ν`).
    #[inline(always)]
    pub fn bivector(&self) -> AsymRank2Tensor<F> {
        AsymRank2Tensor::from_array(std::array::from_fn(|i| self.0[Self::BIVECTOR + i]))
    }

    /// Grade-3 coefficients `a^μ` (contracted against `γ^5 γ_μ`).
    #[inline(always)]
    pub fn axial(&self) -> ComplexVector<F, Contravariant> {
        ComplexVector::from_array(std::array::from_fn(|i| self.0[Self::AXIAL + i]))
    }

    /// Grade-4 coefficient `p`.
    #[inline(always)]
    pub fn pseudoscalar(&self) -> C<F> {
        self.0[Self::PSEUDOSCALAR]
    }

    /// The identity element `1`.
    #[inline]
    pub fn identity() -> Self {
        Self::from_scalar(C::new(F::one(), F::zero()))
    }

    /// The pure grade-0 element `c·1`.
    #[inline]
    pub fn from_scalar(c: C<F>) -> Self {
        let mut m = Self::zero();
        m.0[0] = c;
        m
    }

    /// The pure grade-4 element `c γ^5`.
    #[inline]
    pub fn from_pseudoscalar(c: C<F>) -> Self {
        let mut m = Self::zero();
        m.0[Self::PSEUDOSCALAR] = c;
        m
    }

    /// The gamma-slash `v̸ = v^μ γ_μ = v_μ γ^μ`.
    #[inline]
    pub fn from_gamma(v: &ComplexVector<F, Contravariant>) -> Self {
        let mut m = Self::zero();
        m.0[Self::VECTOR..Self::BIVECTOR].copy_from_slice(v.as_array());
        m
    }

    /// The axial slash `γ^5 v̸ = v^μ γ^5 γ_μ`.
    #[inline]
    pub fn from_axial(v: &ComplexVector<F, Contravariant>) -> Self {
        let mut m = Self::zero();
        m.0[Self::AXIAL..Self::PSEUDOSCALAR].copy_from_slice(v.as_array());
        m
    }

    /// The pure grade-2 element `½ T^{μν} σ_{μν}`.
    #[inline]
    pub fn from_bivector(t: &AsymRank2Tensor<F>) -> Self {
        let mut m = Self::zero();
        m.0[Self::BIVECTOR..Self::AXIAL].copy_from_slice(t.as_array());
        m
    }

    /// A chiral projector: `P_L = (1 − γ^5)/2`, `P_R = (1 + γ^5)/2`, or the
    /// identity for [`Chirality::Both`].
    #[inline]
    pub fn from_projector(chirality: Chirality) -> Self {
        let half = C::new(F::one() / (F::one() + F::one()), F::zero());
        match chirality {
            Chirality::Left => Self::new(
                half,
                ComplexVector::zero(),
                AsymRank2Tensor::zero(),
                ComplexVector::zero(),
                -half,
            ),
            Chirality::Right => Self::new(
                half,
                ComplexVector::zero(),
                AsymRank2Tensor::zero(),
                ComplexVector::zero(),
                half,
            ),
            Chirality::Both => Self::identity(),
        }
    }

    /// The two-gamma chain `a̸ b̸ = (a·b) − i σ^{μν} a_μ b_ν`, which lives entirely
    /// in grades 0 and 2.
    #[inline]
    pub fn from_gamma_pair(
        a: &ComplexVector<F, Contravariant>,
        b: &ComplexVector<F, Contravariant>,
    ) -> Self {
        let mut m = Self::zero();
        m.0[0] = a.dualize().dot(b);
        let wedge = AsymRank2Tensor::wedge(a, b);
        for (slot, w) in wedge.as_array().iter().enumerate() {
            m.0[Self::BIVECTOR + slot] = mul_neg_i(*w);
        }
        m
    }

    /// The 4×4 matrix of this element in the Weyl basis (the basis
    /// [`Bispinor`] stores its components in).
    ///
    /// Blocking the matrix by chirality as `[[A, B], [C, D]]`, the even grades fill
    /// the diagonal blocks and the odd grades the off-diagonal ones:
    ///
    /// ```text
    /// A = (s − p) I₂ + (m⃗ + i k⃗)·σ⃗      B = (v⁰ − a⁰) I₂ − (v⃗ − a⃗)·σ⃗
    /// C = (v⁰ + a⁰) I₂ + (v⃗ + a⃗)·σ⃗      D = (s + p) I₂ + (m⃗ − i k⃗)·σ⃗
    /// ```
    ///
    /// with `k^i = T^{0i}` and `m^i = ½ ε^{ijk} T^{jk}` the boost- and
    /// rotation-like halves of the bivector. `m⃗ ± i k⃗` are its (anti-)self-dual
    /// parts, which is why the two diagonal blocks see different combinations.
    pub fn to_weyl_matrix(&self) -> [[C<F>; 4]; 4] {
        let (s, p) = (self.scalar(), self.pseudoscalar());
        let v = self.vector();
        let a = self.axial();
        let t = self.bivector();
        let k = [t.component(0), t.component(1), t.component(2)];
        let m = [t.component(5), -t.component(4), t.component(3)];

        let blk_a = pauli_block(s - p, std::array::from_fn(|i| m[i] + mul_i(k[i])));
        let blk_d = pauli_block(s + p, std::array::from_fn(|i| m[i] - mul_i(k[i])));
        // σ^μ = (I, σ⃗) sits in the upper-right block and σ̄^μ = (I, −σ⃗) in the
        // lower-left one, and the coefficients enter lowered: v_i = −v^i.
        let blk_b = pauli_block(
            v.0[0] - a.0[0],
            std::array::from_fn(|i| a.0[i + 1] - v.0[i + 1]),
        );
        let blk_c = pauli_block(
            v.0[0] + a.0[0],
            std::array::from_fn(|i| v.0[i + 1] + a.0[i + 1]),
        );

        std::array::from_fn(|row| {
            let (left, right) = if row < 2 {
                (&blk_a, &blk_b)
            } else {
                (&blk_c, &blk_d)
            };
            let r = row % 2;
            [left[r][0], left[r][1], right[r][0], right[r][1]]
        })
    }

    /// Recover the sixteen graded coefficients from a Weyl-basis matrix — the
    /// inverse of [`to_weyl_matrix`](Self::to_weyl_matrix), and the trace
    /// projection `c_A ∝ Tr[X Γ_A]` written out in blocks.
    pub fn from_weyl_matrix(mat: &[[C<F>; 4]; 4]) -> Self {
        let block = |r: usize, c: usize| {
            [
                [mat[r][c], mat[r][c + 1]],
                [mat[r + 1][c], mat[r + 1][c + 1]],
            ]
        };
        let (alpha_a, ua) = unpauli_block(&block(0, 0));
        let (alpha_b, ub) = unpauli_block(&block(0, 2));
        let (alpha_c, uc) = unpauli_block(&block(2, 0));
        let (alpha_d, ud) = unpauli_block(&block(2, 2));
        let half = F::one() / (F::one() + F::one());

        let m: [C<F>; 3] = std::array::from_fn(|i| (ua[i] + ud[i]) * half);
        // ua − ud = 2 i k⃗
        let k: [C<F>; 3] = std::array::from_fn(|i| mul_neg_i(ua[i] - ud[i]) * half);
        let vs: [C<F>; 3] = std::array::from_fn(|i| (uc[i] - ub[i]) * half);
        let ax: [C<F>; 3] = std::array::from_fn(|i| (uc[i] + ub[i]) * half);

        Self::new(
            (alpha_a + alpha_d) * half,
            ComplexVector::new([(alpha_b + alpha_c) * half, vs[0], vs[1], vs[2]]),
            AsymRank2Tensor::new([k[0], k[1], k[2], m[2], -m[1], m[0]]),
            ComplexVector::new([(alpha_c - alpha_b) * half, ax[0], ax[1], ax[2]]),
            (alpha_d - alpha_a) * half,
        )
    }

    /// The Clifford (geometric) product `self · rhs` — what composes γ-chains.
    ///
    /// Computed through the faithful 4×4 Weyl-basis representation rather than a
    /// table of 256 structure constants; the products of the basis elements are
    /// pinned against explicitly built gamma matrices in this module's tests.
    pub fn clifford_product(&self, rhs: &Self) -> Self {
        let a = self.to_weyl_matrix();
        let b = rhs.to_weyl_matrix();
        let prod: [[C<F>; 4]; 4] = std::array::from_fn(|i| {
            std::array::from_fn(|j| dot4(a[i], [b[0][j], b[1][j], b[2][j], b[3][j]]))
        });
        Self::from_weyl_matrix(&prod)
    }

    /// The Fierz pairing `⟨M, N⟩ = ¼ Tr[M N]`, evaluated on coefficients.
    ///
    /// Fierz orthogonality `¼ Tr[Γ_A Γ^B] = ±δ_A^B` makes the pairing grade
    /// diagonal, which is what turns a tensor⊗tensor contraction of two fermion
    /// lines into a dot product of their coefficient vectors. The grade-3 term
    /// carries a minus sign: `¼ Tr[γ^5 γ_μ γ^5 γ_ν] = −g_{μν}`.
    pub fn fierz_pairing(&self, other: &Self) -> C<F> {
        let half = F::one() / (F::one() + F::one());
        let scalars = cmul_add(
            self.scalar(),
            other.scalar(),
            cmul(self.pseudoscalar(), other.pseudoscalar()),
        );
        let vectors = self.vector().dualize().dot(&other.vector());
        let axials = self.axial().dualize().dot(&other.axial());
        let bivectors = self.bivector().contract(&other.bivector()) * half;
        scalars + vectors - axials + bivectors
    }
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
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

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

            // The tensor grade vanishes: Tr[(p̸ ± m) σ^{μν}] = 0 because the trace
            // of one and of three gamma matrices does.
            let u_tensor = up.bar().tensor_bilinear(&up, Chirality::Both)
                + um.bar().tensor_bilinear(&um, Chirality::Both);
            let v_tensor = vp.bar().tensor_bilinear(&vp, Chirality::Both)
                + vm.bar().tensor_bilinear(&vm, Chirality::Both);
            for slot in 0..6 {
                assert!(
                    u_tensor.component(slot).norm() < EPS_ABS,
                    "Fermion tensor bilinear nonzero for helicity-summed spinors: slot {} = {}",
                    slot,
                    u_tensor.component(slot)
                );
                assert!(
                    v_tensor.component(slot).norm() < EPS_ABS,
                    "Antifermion tensor bilinear nonzero for helicity-summed spinors: slot {} = {}",
                    slot,
                    v_tensor.component(slot)
                );
            }

            // Σ_h ū γ^μ γ^ν u = Tr[(p̸ + m) γ^μ γ^ν] = 4m g^{μν}, and −4m g^{μν} for
            // v. Reached through two slashes of covariant basis covectors rather
            // than through `γ^μ γ^ν = g^{μν} − i σ^{μν}`, so the check does not
            // presuppose the decomposition the tensor bilinear is built on.
            for (mu, nu) in iproduct!(0..4, 0..4) {
                let (em, en) = (covariant_basis(mu), covariant_basis(nu));
                let two_slash = |psi: &Bispinor<f64, Ket>| {
                    psi.bar()
                        .scalar_bilinear(&psi.slash(&en).slash(&em), Chirality::Both)
                };
                let expected = if mu == nu {
                    4.0 * mass * minkowski_sign(mu)
                } else {
                    0.0
                };
                let u_gg = two_slash(&up) + two_slash(&um);
                let v_gg = two_slash(&vp) + two_slash(&vm);
                assert!(
                    (u_gg - expected).norm() < EPS_ABS,
                    "Fermion γ^μγ^ν bilinear mismatch at ({mu},{nu}): {u_gg} vs {expected}"
                );
                assert!(
                    (v_gg + expected).norm() < EPS_ABS,
                    "Antifermion γ^μγ^ν bilinear mismatch at ({mu},{nu}): {v_gg} vs {}",
                    -expected
                );
            }

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

    // ─────────────────────────────────────────────────────────────────────────
    // Explicit Weyl-basis Dirac matrices
    //
    // Built here from the Pauli blocks and assembled by commutators, so they are
    // an independent reference for the closed-form block expressions
    // `Multivector::to_weyl_matrix` uses. Every convention this module carries —
    // the chiral block layout, γ⁵'s sign, the ε sign — is checked against these.
    // ─────────────────────────────────────────────────────────────────────────

    type Mat4 = [[C<f64>; 4]; 4];

    /// `σ^1, σ^2, σ^3` at indices 0–2; the 2×2 identity at any other index.
    fn pauli(a: usize) -> [[C<f64>; 2]; 2] {
        let (z, o) = (C::zero(), C::one());
        match a {
            0 => [[z, o], [o, z]],
            1 => [[z, -C::i()], [C::i(), z]],
            2 => [[o, z], [z, -o]],
            _ => [[o, z], [z, o]],
        }
    }

    /// `γ^μ = [[0, σ^μ], [σ̄^μ, 0]]` blocked as (left-chiral, right-chiral), with
    /// `σ^μ = (I, σ⃗)` and `σ̄^μ = (I, −σ⃗)`.
    fn gamma(mu: usize) -> Mat4 {
        let (s, sbar) = if mu == 0 {
            (pauli(4), pauli(4))
        } else {
            let p = pauli(mu - 1);
            (p, [[-p[0][0], -p[0][1]], [-p[1][0], -p[1][1]]])
        };
        let mut m = [[C::zero(); 4]; 4];
        for (i, j) in iproduct!(0..2, 0..2) {
            m[i][2 + j] = s[i][j];
            m[2 + i][j] = sbar[i][j];
        }
        m
    }

    /// `γ⁵ = i γ⁰γ¹γ²γ³`.
    fn gamma5() -> Mat4 {
        let g = mat_mul(
            &mat_mul(&gamma(0), &gamma(1)),
            &mat_mul(&gamma(2), &gamma(3)),
        );
        mat_scale(&g, C::i())
    }

    /// `σ^{μν} = (i/2)[γ^μ, γ^ν]`.
    fn sigma(mu: usize, nu: usize) -> Mat4 {
        let comm = mat_sub(
            &mat_mul(&gamma(mu), &gamma(nu)),
            &mat_mul(&gamma(nu), &gamma(mu)),
        );
        mat_scale(&comm, C::new(0.0, 0.5))
    }

    fn mat_identity() -> Mat4 {
        std::array::from_fn(|i| std::array::from_fn(|j| if i == j { C::one() } else { C::zero() }))
    }

    fn mat_mul(a: &Mat4, b: &Mat4) -> Mat4 {
        std::array::from_fn(|i| {
            std::array::from_fn(|j| (0..4).map(|k| a[i][k] * b[k][j]).sum::<C<f64>>())
        })
    }

    fn mat_add(a: &Mat4, b: &Mat4) -> Mat4 {
        std::array::from_fn(|i| std::array::from_fn(|j| a[i][j] + b[i][j]))
    }

    fn mat_sub(a: &Mat4, b: &Mat4) -> Mat4 {
        std::array::from_fn(|i| std::array::from_fn(|j| a[i][j] - b[i][j]))
    }

    fn mat_scale(a: &Mat4, s: C<f64>) -> Mat4 {
        std::array::from_fn(|i| std::array::from_fn(|j| a[i][j] * s))
    }

    fn mat_max_diff(a: &Mat4, b: &Mat4) -> f64 {
        iproduct!(0..4, 0..4).fold(0.0_f64, |acc, (i, j)| acc.max((a[i][j] - b[i][j]).norm()))
    }

    fn mv_max_diff(a: &Multivector<f64>, b: &Multivector<f64>) -> f64 {
        (0..16).fold(0.0_f64, |acc, i| {
            acc.max((a.as_array()[i] - b.as_array()[i]).norm())
        })
    }

    /// `g^{μμ}` — `+1` for the time component, `−1` for the spatial ones.
    fn minkowski_sign(mu: usize) -> f64 {
        if mu == 0 {
            1.0
        } else {
            -1.0
        }
    }

    fn contravariant_basis(mu: usize) -> ComplexVector<f64, Contravariant> {
        ComplexVector::from_array(std::array::from_fn(|i| {
            if i == mu {
                C::one()
            } else {
                C::zero()
            }
        }))
    }

    /// The covector `δ_α^μ`: slashing it yields `γ^μ` with its index up, since
    /// [`DiracAdjoint::slash_bispinor`] reads its argument as the covariant `v_α`.
    fn covariant_basis(mu: usize) -> ComplexVector<f64, Covariant> {
        ComplexVector::from_array(std::array::from_fn(|i| {
            if i == mu {
                C::one()
            } else {
                C::zero()
            }
        }))
    }

    /// `ε^{μνρσ}` with all indices up: four metric factors flip the sign of the
    /// all-lower symbol [`epsilon4`] returns on contravariant basis vectors.
    fn epsilon_upper(mu: usize, nu: usize, rho: usize, sig: usize) -> C<f64> {
        -epsilon4(
            &contravariant_basis(mu),
            &contravariant_basis(nu),
            &contravariant_basis(rho),
            &contravariant_basis(sig),
        )
    }

    fn permutation_sign(p: &[usize]) -> f64 {
        let mut sign = 1.0;
        for i in 0..p.len() {
            for j in (i + 1)..p.len() {
                if p[i] > p[j] {
                    sign = -sign;
                }
            }
        }
        sign
    }

    fn rand_re(rng: &mut StdRng) -> f64 {
        rng.random::<f64>() * 4.0 - 2.0
    }

    fn rand_c(rng: &mut StdRng) -> C<f64> {
        C::new(rand_re(rng), rand_re(rng))
    }

    fn rand_cvec(rng: &mut StdRng) -> ComplexVector<f64, Contravariant> {
        ComplexVector::new(std::array::from_fn(|_| rand_c(rng)))
    }

    fn rand_tensor(rng: &mut StdRng) -> AsymRank2Tensor<f64> {
        AsymRank2Tensor::new(std::array::from_fn(|_| rand_c(rng)))
    }

    fn rand_multivector(rng: &mut StdRng) -> Multivector<f64> {
        Multivector::from_array(std::array::from_fn(|_| rand_c(rng)))
    }

    /// `Σ_A c_A Γ_A` assembled from the explicit gamma matrices above — the
    /// independent reference for [`Multivector::to_weyl_matrix`].
    fn expand_basis(mv: &Multivector<f64>) -> Mat4 {
        let g5 = gamma5();
        let mut out = mat_scale(&mat_identity(), mv.scalar());
        out = mat_add(&out, &mat_scale(&g5, mv.pseudoscalar()));
        let (v, a) = (mv.vector(), mv.axial());
        for mu in 0..4 {
            // v^μ γ_μ and a^μ γ⁵ γ_μ, with γ_μ = g_{μν} γ^ν
            let lowered = mat_scale(&gamma(mu), C::from(minkowski_sign(mu)));
            out = mat_add(&out, &mat_scale(&lowered, v.component(mu)));
            out = mat_add(&out, &mat_scale(&mat_mul(&g5, &lowered), a.component(mu)));
        }
        let t = mv.bivector();
        for (slot, &(mu, nu)) in AsymRank2Tensor::<f64>::INDEX_PAIRS.iter().enumerate() {
            // Σ_{μ<ν} T^{μν} σ_{μν}, with σ_{μν} = g_{μα} g_{νβ} σ^{αβ}
            let lowered = mat_scale(
                &sigma(mu, nu),
                C::from(minkowski_sign(mu) * minkowski_sign(nu)),
            );
            out = mat_add(&out, &mat_scale(&lowered, t.component(slot)));
        }
        out
    }

    /// The Levi-Civita convention: `ε^{0123} = −1`, every permutation carries its
    /// parity, repeats vanish, and [`epsilon_vector`] is the partial contraction of
    /// [`epsilon4`].
    ///
    /// Blind spot: this fixes ε's normalisation and all its relative signs, but a
    /// consumer that uses ε an even number of times cannot see the overall sign.
    /// `test_sigma_gamma5_epsilon_identity` is what ties that sign to the module's
    /// chirality convention.
    #[test]
    fn test_epsilon_convention() {
        let e: Vec<_> = (0..4).map(contravariant_basis).collect();

        // ALOHA stores the upper-index component as −sign(perm), so ε^{0123} = −1
        // and the all-lower symbol this function returns on contravariant
        // arguments is ε_{0123} = +1.
        assert_eq!(epsilon4(&e[0], &e[1], &e[2], &e[3]), C::one());
        assert_eq!(epsilon_upper(0, 1, 2, 3), -C::one());

        for perm in (0..4).permutations(4) {
            assert_eq!(
                epsilon4(&e[perm[0]], &e[perm[1]], &e[perm[2]], &e[perm[3]]),
                C::from(permutation_sign(&perm)),
                "ε parity wrong at {perm:?}"
            );
        }
        for (i, j, k, l) in iproduct!(0..4, 0..4, 0..4, 0..4) {
            if [i, j, k, l].iter().unique().count() < 4 {
                assert_eq!(
                    epsilon4(&e[i], &e[j], &e[k], &e[l]),
                    C::zero(),
                    "ε nonzero on a repeated index at ({i},{j},{k},{l})"
                );
            }
        }

        let mut rng = StdRng::seed_from_u64(0xE7_0001);
        for _ in 0..32 {
            let (a, b, c, d) = (
                rand_cvec(&mut rng),
                rand_cvec(&mut rng),
                rand_cvec(&mut rng),
                rand_cvec(&mut rng),
            );
            let ev = epsilon_vector(&a, &b, &c);
            assert!(
                (ev.lower().dot(&d) - epsilon4(&a, &b, &c, &d)).norm() < EPS_ABS,
                "epsilon_vector does not contract to epsilon4"
            );
            // Antisymmetry in the three contracted slots.
            assert!(
                (epsilon_vector(&b, &a, &c) + ev).bare_norm_sq() < EPS_ABS,
                "epsilon_vector not antisymmetric in its first two arguments"
            );
        }
    }

    /// `σ^{μν} γ⁵ = (i/2)·s·ε^{μνρσ} σ_{ρσ}` with `s = −1`.
    ///
    /// The sign is forced by two independent conventions meeting: the Weyl basis
    /// fixes `γ⁵ = diag(−1,−1,+1,+1)`, and [`epsilon4`] fixes `ε^{0123} = −1`. In
    /// the textbook `ε^{0123} = +1` convention the same identity has `s = +1`, so
    /// this test is where a flipped ε convention becomes visible without consulting
    /// MadGraph. The `s = +1` form is asserted to be *wrong*, which is what makes
    /// the check falsifiable rather than vacuous.
    #[test]
    fn test_sigma_gamma5_epsilon_identity() {
        // The Weyl basis γ⁵ that `project_left`/`project_right` assume.
        let g5 = gamma5();
        let expected_g5: Mat4 = std::array::from_fn(|i| {
            std::array::from_fn(|j| {
                if i != j {
                    C::zero()
                } else if i < 2 {
                    -C::one()
                } else {
                    C::one()
                }
            })
        });
        assert!(mat_max_diff(&g5, &expected_g5) < EPS_ABS);

        for (mu, nu) in iproduct!(0..4, 0..4) {
            let lhs = mat_mul(&sigma(mu, nu), &g5);
            let mut rhs = [[C::zero(); 4]; 4];
            for (rho, sig) in iproduct!(0..4, 0..4) {
                let lowered = mat_scale(
                    &sigma(rho, sig),
                    C::from(minkowski_sign(rho) * minkowski_sign(sig)),
                );
                rhs = mat_add(&rhs, &mat_scale(&lowered, epsilon_upper(mu, nu, rho, sig)));
            }
            let rhs = mat_scale(&rhs, C::new(0.0, 0.5));

            assert!(
                mat_max_diff(&lhs, &mat_scale(&rhs, -C::one())) < EPS_ABS,
                "σ^{{{mu}{nu}}} γ⁵ ≠ −(i/2) ε^{{{mu}{nu}ρσ}} σ_ρσ"
            );
            if mu != nu {
                assert!(
                    mat_max_diff(&lhs, &rhs) > 1.0,
                    "the s = +1 form is indistinguishable at ({mu},{nu}); the test cannot see the ε sign"
                );
            }
        }
    }

    /// The grade-2 slice: index accessors, contraction, index lowering, the Hodge
    /// dual against its defining formula, and the (anti-)self-dual split.
    #[test]
    fn test_asym_rank2_tensor() {
        let mut rng = StdRng::seed_from_u64(0xA2_0001);
        for _ in 0..32 {
            let t = rand_tensor(&mut rng);
            let s = rand_tensor(&mut rng);

            for (mu, nu) in iproduct!(0..4, 0..4) {
                assert_eq!(t.get(mu, nu), -t.get(nu, mu));
            }
            for (slot, &(mu, nu)) in AsymRank2Tensor::<f64>::INDEX_PAIRS.iter().enumerate() {
                assert_eq!(t.get(mu, nu), t.component(slot));
            }

            // T^{μν} S_{μν} summed over every index pair
            let brute: C<f64> = iproduct!(0..4, 0..4)
                .map(|(mu, nu)| {
                    t.get(mu, nu) * s.get(mu, nu) * (minkowski_sign(mu) * minkowski_sign(nu))
                })
                .sum();
            assert!((t.contract(&s) - brute).norm() < EPS_ABS);

            // dualize lowers both indices and is its own inverse
            for (mu, nu) in iproduct!(0..4, 0..4) {
                let lowered = t.dualize().get(mu, nu);
                let expect = t.get(mu, nu) * (minkowski_sign(mu) * minkowski_sign(nu));
                assert!((lowered - expect).norm() < EPS_ABS);
            }
            assert!(mv_diff_tensor(&t.dualize().dualize(), &t) < EPS_ABS);

            // (⋆T)^{μν} = ½ ε^{μνρσ} T_{ρσ}
            let lowered = t.dualize();
            let expect = AsymRank2Tensor::new(std::array::from_fn(|slot| {
                let (mu, nu) = AsymRank2Tensor::<f64>::INDEX_PAIRS[slot];
                iproduct!(0..4, 0..4)
                    .map(|(rho, sig)| epsilon_upper(mu, nu, rho, sig) * lowered.get(rho, sig))
                    .sum::<C<f64>>()
                    * 0.5
            }));
            assert!(
                mv_diff_tensor(&t.hodge_dual(), &expect) < EPS_ABS,
                "hodge_dual disagrees with ½ ε^{{μνρσ}} T_ρσ"
            );
            // ⋆⋆ = −1 in Lorentzian signature
            assert!(mv_diff_tensor(&t.hodge_dual().hodge_dual(), &(-t)) < EPS_ABS);

            // T^{μν} a_μ b_ν, and the wedge that builds it
            let (a, b) = (rand_cvec(&mut rng), rand_cvec(&mut rng));
            let (al, bl) = (a.dualize(), b.dualize());
            let brute: C<f64> = iproduct!(0..4, 0..4)
                .map(|(mu, nu)| t.get(mu, nu) * al.component(mu) * bl.component(nu))
                .sum();
            assert!((t.contract_vectors(&a, &b) - brute).norm() < EPS_ABS);

            let w = AsymRank2Tensor::wedge(&a, &b);
            for (mu, nu) in iproduct!(0..4, 0..4) {
                let expect = a.component(mu) * b.component(nu) - a.component(nu) * b.component(mu);
                assert!((w.get(mu, nu) - expect).norm() < EPS_ABS);
            }
        }
    }

    fn mv_diff_tensor(a: &AsymRank2Tensor<f64>, b: &AsymRank2Tensor<f64>) -> f64 {
        (0..6).fold(0.0_f64, |acc, i| {
            acc.max((a.component(i) - b.component(i)).norm())
        })
    }

    /// [`Multivector::to_weyl_matrix`] against the explicitly built basis matrices,
    /// element by element and on random coefficients, plus the round trip through
    /// [`Multivector::from_weyl_matrix`].
    ///
    /// Blind spot: agreement here says the closed-form blocks reproduce the basis
    /// expansion, not that either matches MadGraph's γ-matrix conventions — only an
    /// amplitude comparison can say that.
    #[test]
    fn test_multivector_weyl_matrix() {
        assert!(
            mat_max_diff(
                &Multivector::<f64>::identity().to_weyl_matrix(),
                &mat_identity()
            ) < EPS_ABS
        );
        assert!(
            mat_max_diff(
                &Multivector::from_pseudoscalar(C::<f64>::one()).to_weyl_matrix(),
                &gamma5()
            ) < EPS_ABS
        );
        for mu in 0..4 {
            // from_gamma(e_μ) = e_μ^α γ_α = γ_μ
            let lowered = mat_scale(&gamma(mu), C::from(minkowski_sign(mu)));
            assert!(
                mat_max_diff(
                    &Multivector::from_gamma(&contravariant_basis(mu)).to_weyl_matrix(),
                    &lowered
                ) < EPS_ABS,
                "from_gamma mismatch at μ = {mu}"
            );
            assert!(
                mat_max_diff(
                    &Multivector::from_axial(&contravariant_basis(mu)).to_weyl_matrix(),
                    &mat_mul(&gamma5(), &lowered)
                ) < EPS_ABS,
                "from_axial is not γ⁵ γ_μ at μ = {mu}"
            );
        }
        for (slot, &(mu, nu)) in AsymRank2Tensor::<f64>::INDEX_PAIRS.iter().enumerate() {
            let unit = AsymRank2Tensor::new(std::array::from_fn(|i| {
                if i == slot {
                    C::one()
                } else {
                    C::zero()
                }
            }));
            let lowered = mat_scale(
                &sigma(mu, nu),
                C::from(minkowski_sign(mu) * minkowski_sign(nu)),
            );
            assert!(
                mat_max_diff(
                    &Multivector::from_bivector(&unit).to_weyl_matrix(),
                    &lowered
                ) < EPS_ABS,
                "bivector slot {slot} is not σ_{{{mu}{nu}}}"
            );
        }
        for chirality in [Chirality::Left, Chirality::Right, Chirality::Both] {
            let sign = match chirality {
                Chirality::Left => -1.0,
                Chirality::Right => 1.0,
                Chirality::Both => 0.0,
            };
            let expect = mat_add(
                &mat_scale(
                    &mat_identity(),
                    C::from(if sign == 0.0 { 1.0 } else { 0.5 }),
                ),
                &mat_scale(&gamma5(), C::from(0.5 * sign)),
            );
            assert!(
                mat_max_diff(
                    &Multivector::<f64>::from_projector(chirality).to_weyl_matrix(),
                    &expect
                ) < EPS_ABS
            );
        }

        let mut rng = StdRng::seed_from_u64(0x3F_0001);
        for _ in 0..64 {
            let mv = rand_multivector(&mut rng);
            let mat = mv.to_weyl_matrix();
            assert!(
                mat_max_diff(&mat, &expand_basis(&mv)) < EPS_ABS,
                "to_weyl_matrix disagrees with Σ_A c_A Γ_A"
            );
            assert!(
                mv_max_diff(&Multivector::from_weyl_matrix(&mat), &mv) < EPS_ABS,
                "from_weyl_matrix is not the inverse of to_weyl_matrix"
            );
        }
    }

    /// The Clifford product: the γγ chain lands in grades 0 and 2 with the
    /// coefficients `a̸ b̸ = a·b − i σ^{μν} a_μ b_ν` predicts, projectors behave as
    /// projectors, `γ⁵ v̸` is the grade-3 basis element, and the product is
    /// associative.
    #[test]
    fn test_clifford_product() {
        let mut rng = StdRng::seed_from_u64(0xC1_0001);
        for _ in 0..32 {
            let (a, b) = (rand_cvec(&mut rng), rand_cvec(&mut rng));
            let chained =
                Multivector::from_gamma(&a).clifford_product(&Multivector::from_gamma(&b));
            let closed = Multivector::from_gamma_pair(&a, &b);
            assert!(
                mv_max_diff(&chained, &closed) < EPS_ABS,
                "a̸b̸ from the Clifford product disagrees with the grade-0/2 closed form"
            );
            // Odd grades are empty and the scalar grade is the Minkowski dot.
            assert!(chained.vector().bare_norm_sq() < EPS_ABS);
            assert!(chained.axial().bare_norm_sq() < EPS_ABS);
            assert!(chained.pseudoscalar().norm() < EPS_ABS);
            assert!((chained.scalar() - a.dualize().dot(&b)).norm() < EPS_ABS);

            // γ⁵ v̸ is exactly the grade-3 part: the basis-element ordering claim.
            let g5v = Multivector::from_pseudoscalar(C::<f64>::one())
                .clifford_product(&Multivector::from_gamma(&a));
            assert!(
                mv_max_diff(&g5v, &Multivector::from_axial(&a)) < EPS_ABS,
                "γ⁵ v̸ is not the grade-3 basis element"
            );
            // and the opposite ordering is its negative
            let vg5 = Multivector::from_gamma(&a)
                .clifford_product(&Multivector::from_pseudoscalar(C::<f64>::one()));
            assert!(mv_max_diff(&vg5, &(-Multivector::from_axial(&a))) < EPS_ABS);

            let (x, y, z) = (
                rand_multivector(&mut rng),
                rand_multivector(&mut rng),
                rand_multivector(&mut rng),
            );
            assert!(
                mv_max_diff(
                    &x.clifford_product(&y).clifford_product(&z),
                    &x.clifford_product(&y.clifford_product(&z))
                ) < 1e-10,
                "Clifford product is not associative"
            );
            assert!(mv_max_diff(&x.clifford_product(&Multivector::identity()), &x) < EPS_ABS);
        }

        let pl = Multivector::<f64>::from_projector(Chirality::Left);
        let pr = Multivector::<f64>::from_projector(Chirality::Right);
        assert!(mv_max_diff(&pl.clifford_product(&pl), &pl) < EPS_ABS);
        assert!(mv_max_diff(&pr.clifford_product(&pr), &pr) < EPS_ABS);
        assert!(pl
            .clifford_product(&pr)
            .as_array()
            .iter()
            .all(|c| c.norm() < EPS_ABS));
        assert!(mv_max_diff(&(pl + pr), &Multivector::identity()) < EPS_ABS);
    }

    /// Even grades preserve a spinor's chiral blocks and odd grades swap them.
    ///
    /// This is the structural content of the `1+4+6+4+1` grading in the Weyl basis,
    /// and it holds for both adjoints because the bra action is the transpose of the
    /// ket action, not a chiral swap.
    #[test]
    fn test_grade_parity_and_chirality() {
        let mut rng = StdRng::seed_from_u64(0x9A_0001);
        for _ in 0..16 {
            let even = [
                Multivector::from_scalar(rand_c(&mut rng)),
                Multivector::from_bivector(&rand_tensor(&mut rng)),
                Multivector::from_pseudoscalar(rand_c(&mut rng)),
            ];
            let odd = [
                Multivector::from_gamma(&rand_cvec(&mut rng)),
                Multivector::from_axial(&rand_cvec(&mut rng)),
            ];
            let (l0, l1) = (rand_c(&mut rng), rand_c(&mut rng));
            let left_ket = Bispinor::<f64, Ket>::from_components([l0, l1, C::zero(), C::zero()]);
            let left_bra = Bispinor::<f64, Bra>::from_components([l0, l1, C::zero(), C::zero()]);

            for m in &even {
                for out in [left_ket.apply(m).0, left_bra.apply(m).0] {
                    assert!(
                        out[2].norm() < EPS_ABS && out[3].norm() < EPS_ABS,
                        "an even grade moved weight out of the left-chiral block"
                    );
                }
            }
            for m in &odd {
                for out in [left_ket.apply(m).0, left_bra.apply(m).0] {
                    assert!(
                        out[0].norm() < EPS_ABS && out[1].norm() < EPS_ABS,
                        "an odd grade left weight in the left-chiral block"
                    );
                }
            }
        }
    }

    /// [`SpinorRepr::apply`] reduces to the hand-written kernels it generalises:
    /// the grade-1 element is [`SpinorRepr::slash`] and the projector grades are
    /// [`SpinorRepr::project_left`]/[`SpinorRepr::project_right`], on both adjoints.
    /// Also `ψ̄ (M ψ) = (ψ̄ M) ψ`.
    #[test]
    fn test_apply_matches_existing_kernels() {
        let mut rng = StdRng::seed_from_u64(0x5A_0001);
        for (p, mass, nhel, nsf) in spinor_test_cases() {
            let ket = Bispinor::<f64, Ket>::from_momentum(p, mass, nhel, nsf);
            let bra = Bispinor::<f64, Bra>::from_momentum(p, mass, nhel, nsf);
            for _ in 0..4 {
                let v = rand_cvec(&mut rng);
                let slash = Multivector::from_gamma(&v);
                assert!(
                    (ket.apply(&slash) - ket.slash(&v)).bare_norm_sq() < EPS_ABS,
                    "ket apply(γ) does not reproduce slash"
                );
                assert!(
                    (bra.apply(&slash) - bra.slash(&v)).bare_norm_sq() < EPS_ABS,
                    "bra apply(γ) does not reproduce slash"
                );
                let m = rand_multivector(&mut rng);
                let via_ket = bra.scalar_bilinear(&ket.apply(&m), Chirality::Both);
                let via_bra = bra.apply(&m).scalar_bilinear(&ket, Chirality::Both);
                assert!(
                    (via_ket - via_bra).norm() < 1e-11,
                    "ψ̄(Mψ) ≠ (ψ̄M)ψ: {via_ket} vs {via_bra}"
                );
            }
            assert_eq!(
                ket.apply(&Multivector::from_projector(Chirality::Left)),
                ket.project_left()
            );
            assert_eq!(
                ket.apply(&Multivector::from_projector(Chirality::Right)),
                ket.project_right()
            );
            assert_eq!(
                bra.apply(&Multivector::from_projector(Chirality::Left)),
                bra.project_left()
            );
        }
    }

    /// The Fierz pairing on coefficients equals `¼ Tr[M N]` on matrices.
    #[test]
    fn test_fierz_pairing_is_quarter_trace() {
        let mut rng = StdRng::seed_from_u64(0xF1_0001);
        for _ in 0..64 {
            let (m, n) = (rand_multivector(&mut rng), rand_multivector(&mut rng));
            let prod = mat_mul(&m.to_weyl_matrix(), &n.to_weyl_matrix());
            let trace: C<f64> = (0..4).map(|i| prod[i][i]).sum();
            assert!(
                (m.fierz_pairing(&n) - trace * 0.25).norm() < 1e-11,
                "fierz_pairing ≠ ¼ Tr[MN]"
            );
            assert!((m.fierz_pairing(&n) - n.fierz_pairing(&m)).norm() < 1e-12);
        }
    }

    /// `f̄ a̸ b̸ f = (a·b) f̄ f − i a_μ b_ν f̄ σ^{μν} f`, per chirality.
    ///
    /// This is the identity that makes the grade-2 slice the tensor bilinear: the
    /// left side goes through two hand-written slashes and the scalar bilinear, the
    /// right through [`SpinorRepr::tensor_bilinear`], so a sign or component-order
    /// error in either shows up as a disagreement.
    #[test]
    fn test_gamma_pair_bilinear_identity() {
        let mut rng = StdRng::seed_from_u64(0x2B_0001);
        for (p, mass, nhel, nsf) in spinor_test_cases() {
            let fi = Bispinor::<f64, Ket>::from_momentum(p, mass, nhel, nsf);
            let fo = fi.bar();
            for _ in 0..4 {
                let (a, b) = (rand_cvec(&mut rng), rand_cvec(&mut rng));
                let a_dot_b = a.dualize().dot(&b);
                for chirality in [Chirality::Left, Chirality::Right, Chirality::Both] {
                    let projected = match chirality {
                        Chirality::Left => fi.project_left(),
                        Chirality::Right => fi.project_right(),
                        Chirality::Both => fi,
                    };
                    let lhs = fo.scalar_bilinear(&projected.slash(&b).slash(&a), Chirality::Both);
                    let tensor = fo.tensor_bilinear(&fi, chirality);
                    let rhs = a_dot_b * fo.scalar_bilinear(&fi, chirality)
                        - C::<f64>::i() * tensor.contract_vectors(&a, &b);
                    assert!(
                        (lhs - rhs).norm() < 1e-11,
                        "f̄ a̸b̸ f identity fails at chirality {chirality:?}: {lhs} vs {rhs}"
                    );
                }
            }
        }
    }

    /// The per-helicity Fierz reconstruction `ψ ψ̄ = ¼ Σ_A (ψ̄ Γ_A ψ) Γ^A`, and the
    /// pairing identity `ψ̄ M ψ = ⟨fierz(ψ̄, ψ), M⟩`.
    ///
    /// This is the finest oracle in this module: it compares the outer product of
    /// the spinor components — sixteen complex numbers — against the sixteen
    /// bilinears read back through the graded basis, one helicity at a time. It
    /// passes only if every bilinear's normalisation, sign and index order is
    /// mutually consistent, which the helicity-summed relations in
    /// `test_completeness_relations` provably cannot see: those sum the two
    /// helicities first and so are blind to any error antisymmetric in helicity, and
    /// they project onto only five of the sixteen basis directions.
    ///
    /// The *diagonal* form `ψ ψ̄` has a degeneracy of its own —
    /// `ψ̄ σ^{0i} ψ ∝ (p⃗ × s⃗)^i` vanishes identically when the spin is along the
    /// momentum, so a helicity eigenstate leaves the three boost-like grade-2 slots
    /// at zero and cannot see an error in them. The general outer product `ψ φ̄`
    /// with an unrelated `φ̄` below is what covers all sixteen.
    #[test]
    fn test_fierz_reconstruction() {
        let mut rng = StdRng::seed_from_u64(0x4E_0001);
        for (p, mass, nhel, nsf) in spinor_test_cases() {
            let psi = Bispinor::<f64, Ket>::from_momentum(p, mass, nhel, nsf);
            let psibar = psi.bar();
            let coefficients = psibar.fierz_coefficients(&psi);

            let outer: Mat4 = std::array::from_fn(|i| {
                std::array::from_fn(|j| psi.component(i) * psibar.component(j))
            });
            let rebuilt = mat_scale(&coefficients.to_weyl_matrix(), C::from(0.25));
            assert!(
                mat_max_diff(&outer, &rebuilt) < EPS_ABS,
                "ψψ̄ ≠ ¼ Σ_A (ψ̄ Γ_A ψ) Γ^A for p = {p}, m = {mass}, {nhel}, {nsf:?}"
            );

            // Each grade is the corresponding bilinear, with no rescaling.
            assert_eq!(
                coefficients.scalar(),
                psibar.scalar_bilinear(&psi, Chirality::Both)
            );
            assert_eq!(
                coefficients.bivector(),
                psibar.tensor_bilinear(&psi, Chirality::Both)
            );
            assert_eq!(
                coefficients.axial(),
                psibar.axial_vector_bilinear(&psi, Chirality::Both)
            );

            for _ in 0..4 {
                let other = Bispinor::<f64, Bra>::from_components(std::array::from_fn(|_| {
                    rand_c(&mut rng)
                }));
                let general: Mat4 = std::array::from_fn(|i| {
                    std::array::from_fn(|j| psi.component(i) * other.component(j))
                });
                let rebuilt_general = mat_scale(
                    &other.fierz_coefficients(&psi).to_weyl_matrix(),
                    C::from(0.25),
                );
                assert!(
                    mat_max_diff(&general, &rebuilt_general) < 1e-11,
                    "ψφ̄ ≠ ¼ Σ_A (φ̄ Γ_A ψ) Γ^A"
                );

                let m = rand_multivector(&mut rng);
                let direct = psibar.scalar_bilinear(&psi.apply(&m), Chirality::Both);
                assert!(
                    (direct - coefficients.fierz_pairing(&m)).norm() < 1e-11,
                    "ψ̄ M ψ ≠ ⟨fierz(ψ̄, ψ), M⟩: {direct} vs {}",
                    coefficients.fierz_pairing(&m)
                );
            }
        }
    }
}
