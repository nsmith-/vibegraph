//! Lorentz representation traits and concrete basis implementations.
//!
//! The Lorentz group Spin(1,3) ≅ SL(2,ℂ) has irreducible representations
//! labelled by two half-integers `(j_L, j_R)`. This module defines the base
//! trait [`LorentzRepr`] for any representation, and specialized traits like
//! [`SpinorRepr`] and [`VectorRepr`] for physics-specific operations. We also
//! provide concrete implementations.

use std::iter::Sum;
use std::marker::PhantomData;

use num_traits::Zero;

use crate::helas::repr::numbers::Chirality;
use crate::helas::repr::vectorspace::impl_mul_for_array;

use super::numbers::{Charge, SpinorHelicity};
use super::vectorspace::{impl_vectorspace, ArrayBacked};
use super::{r, ri, Real, C};

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
    fn bare_norm_sq(self) -> F
    where
        F: Sum;

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

    fn bare_norm_sq(self) -> F
    where
        F: Sum,
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
        self.0[1] * self.0[1] + self.0[2] * self.0[2] + self.0[3] * self.0[3]
    }

    /// Momentum magnitude |p| = √(px² + py² + pz²).
    #[inline(always)]
    pub fn p3(self) -> F {
        self.p3_squared().sqrt()
    }

    /// Invariant mass squared m² = E² - |p|².
    #[inline(always)]
    pub fn m2(self) -> F {
        self.e() * self.e() - self.p3_squared()
    }

    /// Invariant mass m = √(E² - |p|²).
    #[inline(always)]
    pub fn m(self) -> F {
        self.m2().sqrt()
    }
}

/// A complex (e.g. polarisation) 4-vector.
///
/// This is the fiber type for [`SpinorRepr::left_current`] and [`SpinorRepr::right_current`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComplexVector<F: Real, V: Variance>([C<F>; 4], PhantomData<V>);

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
        self.0[0] * other.0[0]
            + self.0[1] * other.0[1]
            + self.0[2] * other.0[2]
            + self.0[3] * other.0[3]
    }

    fn dualize(&self) -> Self::Dual {
        let arr = self.as_array();
        ComplexVector::from_array([arr[0], -arr[1], -arr[2], -arr[3]])
    }

    fn bare_norm_sq(self) -> F
    where
        F: Sum,
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
                C::new(lv.0[0], F::ZERO),
                C::new(lv.0[1], F::ZERO),
                C::new(lv.0[2], F::ZERO),
                C::new(lv.0[3], F::ZERO),
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
        // Here the variance is THE SAME for both, so we need to manually insert the metric signs in the contraction
        self.0[0] * other.0[0]
            - self.0[1] * other.0[1]
            - self.0[2] * other.0[2]
            - self.0[3] * other.0[3]
    }
}

/// Sealed trait for spinor flow direction, implemented by `FlowIn` and `FlowOut`.
pub trait SpinorFlow: sealed::Sealed + Copy + PartialEq + Eq + 'static {
    type Opposite: SpinorFlow;
    const INCOMING: bool;

    /// Assemble a (massive) bispinor with this flow direction
    fn build_bispinor<F: Real>(
        p: LorentzVector<F, Contravariant>,
        mass: F,
        nhel: SpinorHelicity,
        nsf: Charge,
    ) -> Bispinor<F, Self>;
}

/// Marker for flowing-IN spinors (`u`/`v` columns).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlowIn;
impl sealed::Sealed for FlowIn {}
impl SpinorFlow for FlowIn {
    type Opposite = FlowOut;
    const INCOMING: bool = true;

    fn build_bispinor<F: Real>(
        p: LorentzVector<F, Contravariant>,
        mass: F,
        nhel: SpinorHelicity,
        nsf: Charge,
    ) -> Bispinor<F, Self> {
        Bispinor::from_array(weyl_ixxxxx(p, mass, nhel, nsf))
    }
}

/// Marker for flowing-OUT spinors (`ū`/`v̄` rows).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlowOut;
impl sealed::Sealed for FlowOut {}
impl SpinorFlow for FlowOut {
    type Opposite = FlowIn;
    const INCOMING: bool = false;

    fn build_bispinor<F: Real>(
        p: LorentzVector<F, Contravariant>,
        mass: F,
        nhel: SpinorHelicity,
        nsf: Charge,
    ) -> Bispinor<F, Self> {
        Bispinor::from_array(weyl_ixxxxx(p, mass, nhel, nsf)).bar()
    }
}

/// Marker for flows that can sit on the bra (out) side of a bilinear.
pub trait OutFlow: SpinorFlow {}
impl OutFlow for FlowOut {}

/// Spin-½ Lorentz representation.
///
/// This is a trait to allow for multiple concrete bases (e.g. Weyl, Dirac) to be implemented.
pub trait SpinorRepr<F: Real, Flow: SpinorFlow = FlowIn>: LorentzRepr<F> {
    /// The dual representation (e.g. for Weyl spinors, the dual of (½,0) is (0,½)).
    type Dual: SpinorRepr<F, Flow::Opposite, Scalar = Self::Scalar>;

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
    fn bare_norm_sq(self) -> F
    where
        F: Sum;

    // Things only applicable to the bra (out) side of a bilinear follow

    /// Left-handed fermion current  `J_L^μ = v̄_out γ^μ P_L u_in`.
    fn left_current(&self, fi: &Self::Dual) -> ComplexVector<F, Contravariant>
    where
        Flow: OutFlow;

    /// Right-handed fermion current  `J_R^μ = v̄_out γ^μ P_R u_in`.
    fn right_current(&self, fi: &Self::Dual) -> ComplexVector<F, Contravariant>
    where
        Flow: OutFlow;

    /// Scalar bilinear with chiral structure: `f̄ Γ f` where Γ ∈ {Identity, P_L, P_R}.
    fn scalar_bilinear(&self, fi: &Self::Dual, chirality: Chirality) -> C<F>
    where
        Flow: OutFlow;

    /// Vector bilinear contraction: `f̄ γ^μ Γ f` where `Γ` encodes chirality.
    ///
    /// This can be implemented using the left and right currents:
    /// - Left (P_L): `J_L^μ = v̄_out γ^μ P_L u_in`
    /// - Right (P_R): `J_R^μ = v̄_out γ^μ P_R u_in`
    /// - Both (Identity): `J^μ = J_L^μ + J_R^μ`
    fn vector_bilinear(
        &self,
        fi: &Self::Dual,
        chirality: Chirality,
    ) -> ComplexVector<F, Contravariant>
    where
        Flow: OutFlow,
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
pub struct Bispinor<F: Real, Flow: SpinorFlow>([C<F>; 4], PhantomData<Flow>);

impl<F: Real, Flow: SpinorFlow> ArrayBacked<C<F>, 4> for Bispinor<F, Flow> {
    fn as_array(&self) -> &[C<F>; 4] {
        &self.0
    }

    fn from_array(arr: [C<F>; 4]) -> Self {
        Bispinor(arr, PhantomData)
    }
}

impl_vectorspace!(impl[F: Real, Flow: SpinorFlow] Bispinor<F, Flow>, scalar = C<F>);

// Allow scalar multiplication by a real (performance optimization)
impl_mul_for_array!(impl[F: Real, Flow: SpinorFlow] Bispinor<F, Flow>, scalar = F);

impl<F: Real, Flow: SpinorFlow> LorentzRepr<F> for Bispinor<F, Flow> {
    type Scalar = C<F>;
}

impl<F: Real, Flow: SpinorFlow> SpinorRepr<F, Flow> for Bispinor<F, Flow> {
    type Dual = Bispinor<F, Flow::Opposite>;

    fn dualize(&self) -> Self::Dual {
        let arr = self.as_array();
        Bispinor::from_array([arr[2].conj(), arr[3].conj(), arr[0].conj(), arr[1].conj()])
    }

    /// Left projection: zero the right-chiral (indices 2-3) components, keeping left-chiral (0-1).
    fn project_left(self) -> Self {
        Bispinor([self.0[0], self.0[1], C::ZERO, C::ZERO], PhantomData)
    }

    /// Right projection: zero the left-chiral (indices 0-1) components, keeping right-chiral (2-3).
    fn project_right(self) -> Self {
        Bispinor([C::ZERO, C::ZERO, self.0[2], self.0[3]], PhantomData)
    }

    /// Apply the gamma-slash `v̸ = γ^μ v_μ`.
    ///
    /// In the Weyl basis `γ^μ = [[0, σ̄^μ], [σ^μ, 0]]`, so the slash swaps the
    /// chiral blocks: the left-chiral output is `(σ̄·v) ψ_R` and the right-chiral
    /// output is `(σ·v) ψ_L`, with
    /// `σ·v  = [[v₀+v₃, v₁−iv₂], [v₁+iv₂, v₀−v₃]]` and
    /// `σ̄·v = [[v₀−v₃, −(v₁−iv₂)], [−(v₁+iv₂), v₀+v₃]]`.
    fn slash<V: Variance>(self, v: &ComplexVector<F, V>) -> Self {
        // TODO: does this depend on the variance?
        let psi = &self.0;
        let v = &v.0;
        let i = ri(F::one());

        // σ·v and σ̄·v share the v₁∓iv₂ off-diagonal entries.
        let v0_p_v3 = v[0] + v[3];
        let v0_m_v3 = v[0] - v[3];
        let v1_m_iv2 = v[1] - i * v[2];
        let v1_p_iv2 = v[1] + i * v[2];

        // ψ_L ← (σ̄·v) ψ_R,  σ̄·v = v₀ − v⃗·σ⃗ = [[v₀−v₃, −(v₁−iv₂)], [−(v₁+iv₂), v₀+v₃]]
        let l1 = v0_m_v3 * psi[2] - v1_m_iv2 * psi[3];
        let l2 = -v1_p_iv2 * psi[2] + v0_p_v3 * psi[3];
        // ψ_R ← (σ·v) ψ_L
        let r1 = v0_p_v3 * psi[0] + v1_m_iv2 * psi[1];
        let r2 = v1_p_iv2 * psi[0] + v0_m_v3 * psi[1];

        Bispinor([l1, l2, r1, r2], PhantomData)
    }

    fn bare_norm_sq(self) -> F
    where
        F: Sum,
    {
        self.0.iter().map(|x| x.norm_sqr()).sum()
    }

    /// Left current using right-chiral indices of `fo` and left-chiral indices of `fi`.
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
    fn left_current(&self, fi: &Self::Dual) -> ComplexVector<F, Contravariant>
    where
        Flow: OutFlow,
    {
        let fo = &self.0;
        let fi = &fi.0;
        ComplexVector {
            0: [
                fo[2] * fi[0] + fo[3] * fi[1],
                -(fo[2] * fi[1] + fo[3] * fi[0]),
                ri(F::one()) * (fo[2] * fi[1] - fo[3] * fi[0]),
                -fo[2] * fi[0] + fo[3] * fi[1],
            ],
            1: PhantomData,
        }
    }

    /// Right current using left-chiral indices of `fo` and right-chiral indices of `fi`.
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
    fn right_current(&self, fi: &Self::Dual) -> ComplexVector<F, Contravariant>
    where
        Flow: OutFlow,
    {
        let fo = &self.0;
        let fi = &fi.0;
        ComplexVector {
            0: [
                fo[0] * fi[2] + fo[1] * fi[3],
                fo[0] * fi[3] + fo[1] * fi[2],
                -ri(F::one()) * (fo[0] * fi[3] - fo[1] * fi[2]),
                fo[0] * fi[2] - fo[1] * fi[3],
            ],
            1: PhantomData,
        }
    }

    /// Scalar bilinear contraction: `f̄ Γ f` where `Γ` encodes chirality.
    ///
    /// With `fo` having the sfomeg swap convention (indices 0,1=RIGHT-chiral, 2,3=LEFT-chiral)
    /// and `fi` with indices 0,1=LEFT-chiral, 2,3=RIGHT-chiral:
    /// - Left (P_L): `fi_left · fo_left = fi[0]·fo[2] + fi[1]·fo[3]`
    /// - Right (P_R): `fi_right · fo_right = fi[2]·fo[0] + fi[3]·fo[1]`
    /// - Both (Identity): both left and right contractions.
    fn scalar_bilinear(&self, fi: &Self::Dual, chirality: Chirality) -> C<F>
    where
        Flow: OutFlow,
    {
        let fo = &self.0;
        let fi = &fi.0;
        let result = match chirality {
            Chirality::Left => fi[0] * fo[2] + fi[1] * fo[3],
            Chirality::Right => fi[2] * fo[0] + fi[3] * fo[1],
            Chirality::Both => (fi[0] * fo[2] + fi[1] * fo[3]) + (fi[2] * fo[0] + fi[3] * fo[1]),
        };
        result
    }
}

impl<F: Real, Flow: SpinorFlow> Bispinor<F, Flow> {
    /// Construct a spinor from a 4-momentum, mass, helicity, and fermion flow.
    #[inline(always)]
    pub fn from_momentum(
        p: LorentzVector<F, Contravariant>,
        mass: F,
        nhel: SpinorHelicity,
        nsf: Charge,
    ) -> Self {
        Flow::build_bispinor(p, mass, nhel, nsf)
    }
}

impl<F: Real> Bispinor<F, FlowIn> {
    /// Bar the spinor to get the outgoing flow: `ū = ψ† γ^0`.
    #[inline(always)]
    pub fn bar(self) -> Bispinor<F, FlowOut> {
        self.dualize()
    }
}

impl<F: Real> Bispinor<F, FlowOut> {
    /// Unbar the spinor to get the incoming flow: `u = γ^0 ψ̄†`.
    #[inline(always)]
    pub fn unbar(self) -> Bispinor<F, FlowIn> {
        self.dualize()
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
    let two = F::ONE + F::ONE;
    let nh = nhel.sign() * nsf.sign();
    let nsf_i = nsf.sign();

    let mut fi = [C::new(F::ZERO, F::ZERO); 4];

    if mass != F::ZERO {
        let pp = p.p3().min(p.e());

        if pp == F::ZERO {
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

            let pp3 = (pp + p.pz()).max(F::ZERO);
            let chi0 = r((pp3 / (two * pp)).sqrt());
            let chi1 = if pp3 > F::ZERO {
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
        let sqp0p3 = if p.px() == F::ZERO && p.py() == F::ZERO && p.pz() < F::ZERO {
            F::ZERO
        } else {
            (p.e() + p.pz()).max(F::ZERO).sqrt() * F::from(nsf_i).unwrap()
        };
        let chi0 = r(sqp0p3);
        let chi1 = if sqp0p3 == F::ZERO {
            r(F::from(-nhel.sign()).unwrap() * (two * p.e()).sqrt())
        } else {
            C::new(F::from(nh).unwrap() * p.px(), p.py()) / r(sqp0p3)
        };

        if nh == 1 {
            fi[0] = C::ZERO;
            fi[1] = C::ZERO;
            fi[2] = chi0;
            fi[3] = chi1;
        } else {
            fi[0] = chi1;
            fi[1] = chi0;
            fi[2] = C::ZERO;
            fi[3] = C::ZERO;
        }
    }

    fi
}
