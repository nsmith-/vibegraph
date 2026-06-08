//! Lorentz representation traits and concrete basis implementations.
//!
//! ## Representation hierarchy
//!
//! The Lorentz group Spin(1,3) ≅ SL(2,ℂ) has irreducible representations
//! labelled by two half-integers `(j_L, j_R)`. The physically relevant ones are:
//!
//! | Trait | Rep | Dim | Description |
//! |-------|-----|-----|-------------|
//! | [`LorentzRepr`] | base | — | Marker + fiber type for any rep |
//! | [`SpinorRepr`] | (½,0)⊕(0,½) | 4 | Dirac spinor (Weyl decomposed) |
//! | [`VectorRepr`] | (½,½) | 4 | tangent / polarisation vector |
//! | [`ScalarRepr`] | (0,0) | 1 | Lorentz scalar |
//!
//! ## Concrete types
//!

use num_traits::Zero;

use super::vectorspace::{impl_add_for_array, impl_mul_for_array, VectorSpace};
use super::{r, ri, Real, C};

// ─────────────────────────────────────────────────────────────────────────────
// Helicity and charge labels
// ─────────────────────────────────────────────────────────────────────────────

/// Spinor helicity label: the sign of the projection of spin onto momentum.
///
/// Corresponds to the HELAS `nhel` parameter (±1). The name `Up`/`Down`
/// matches the convention that `Up` (positive helicity, right-handed) has
/// `λ = +½` and `Down` (negative helicity, left-handed) has `λ = −½`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpinorHelicity {
    /// Positive helicity: `nhel = +1`.
    Up,
    /// Negative helicity: `nhel = −1`.
    Down,
}

impl SpinorHelicity {
    /// Return `+1` or `−1` as an `i32`.
    #[inline(always)]
    pub fn sign(self) -> i32 {
        match self {
            SpinorHelicity::Up => 1,
            SpinorHelicity::Down => -1,
        }
    }
}

impl std::fmt::Display for SpinorHelicity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpinorHelicity::Up => write!(f, "↑"),
            SpinorHelicity::Down => write!(f, "↓"),
        }
    }
}

/// Particle-vs-antiparticle label.
///
/// Corresponds to the HELAS `nsf` parameter (+1 for particle, −1 for
/// antiparticle).  The signed momentum stored in
/// [`DiracWf`](crate::helas::wavefn::DiracWf) is `p * nsf.sign()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Charge {
    /// Particle (e.g. e⁻, q): `nsf = +1`.
    Particle,
    /// Antiparticle (e.g. e⁺, q̄): `nsf = −1`.
    Antiparticle,
}

impl Charge {
    /// Return `+1` or `−1` as an `i32`.
    #[inline(always)]
    pub fn sign(self) -> i32 {
        match self {
            Charge::Particle => 1,
            Charge::Antiparticle => -1,
        }
    }
}

impl std::fmt::Display for Charge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Charge::Particle => write!(f, "particle"),
            Charge::Antiparticle => write!(f, "antiparticle"),
        }
    }
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
// pub trait LorentzRepr<F: Real>: VectorSpace<F> {}
pub trait LorentzRepr<F: Real>: Sized + Copy + 'static {}

// Implement LorentzRepr for the (complex) scalar representation
impl<F: Real> LorentzRepr<F> for F {}
impl<F: Real> LorentzRepr<F> for C<F> {}

/// Antisymmetric rank-2 Lorentz tensor (placeholder type).
///
/// Represents a tensor `T^{μν} = -T^{νμ}` such as the output of `σ^μν = i/2 [γ^μ, γ^ν]`.
#[derive(Clone, Copy, Debug)]
pub struct Rank2Tensor<F: Real>(pub [C<F>; 6]);

impl<F: Real> LorentzRepr<F> for Rank2Tensor<F> {}

/// Spin-1 Lorentz representation
pub trait VectorRepr<F: Real>: LorentzRepr<F> {}

/// A contravariant 4-momentum vector `p^μ = [E, p_x, p_y, p_z]`.
///
/// This type wraps a real `[F; 4]` array.  It is distinct from
/// [`MinkowskiRep::Fiber`], which is a complex 4-component HELAS Lorentz
/// object (`[C<F>; 4]`).  Until explicit `Vector`/`Covector` newtypes are
/// introduced, index variance is handled by convention at contraction sites.
///
/// # TODO
/// Implement `LorentzRepr<F>` for a `RealVectorRep` marker type whose fiber is
/// `FourMomentum<F>`, completing the bundle-theoretic picture for kinematics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LorentzVector<F: Real>(pub [F; 4]);

impl_add_for_array!(LorentzVector<F>, 4);
impl_mul_for_array!(LorentzVector<F>, F, 4);

impl<F: Real> Zero for LorentzVector<F> {
    #[inline(always)]
    fn zero() -> Self {
        LorentzVector([F::zero(); 4])
    }

    fn is_zero(&self) -> bool {
        self.0.iter().all(|x| x.is_zero())
    }
}

impl<F: Real> VectorSpace<F> for LorentzVector<F> {}

impl<F: Real> LorentzRepr<F> for LorentzVector<F> {}

impl<F: Real> VectorRepr<F> for LorentzVector<F> {}

impl<F: Real> LorentzVector<F> {
    /// Construct from individual components `[E, px, py, pz]`.
    #[inline(always)]
    pub fn new(e: F, px: F, py: F, pz: F) -> Self {
        LorentzVector([e, px, py, pz])
    }

    /// Construct from mass and cartesian 3-momentum
    #[inline(always)]
    pub fn from_pxpypzmass(px: F, py: F, pz: F, mass: F) -> Self {
        let p3_squared = px * px + py * py + pz * pz;
        let e = (p3_squared + mass * mass).sqrt();
        LorentzVector([e, px, py, pz])
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

impl<F: Real> std::ops::Index<usize> for LorentzVector<F> {
    type Output = F;
    #[inline(always)]
    fn index(&self, i: usize) -> &F {
        &self.0[i]
    }
}

/// A complex (e.g. polarisation) 4-vector.
///
/// This is the fiber type for [`SpinorRepr::left_current`] and [`SpinorRepr::right_current`].
#[derive(Clone, Copy, Debug)]
pub struct ComplexVector<F: Real>(pub [C<F>; 4]);

impl_add_for_array!(ComplexVector<F>, 4);
impl_mul_for_array!(ComplexVector<F>, F, 4);
impl_mul_for_array!(ComplexVector<F>, C<F>, 4);

impl<F: Real> Zero for ComplexVector<F> {
    #[inline(always)]
    fn zero() -> Self {
        ComplexVector([C::zero(); 4])
    }

    fn is_zero(&self) -> bool {
        self.0.iter().all(|c| c.is_zero())
    }
}

impl<F: Real> VectorSpace<F> for ComplexVector<F> {}
impl<F: Real> VectorSpace<C<F>> for ComplexVector<F> {}

impl<F: Real> LorentzRepr<F> for ComplexVector<F> {}

impl<F: Real> VectorRepr<F> for ComplexVector<F> {}

impl<F: Real> std::ops::Index<usize> for ComplexVector<F> {
    type Output = C<F>;
    #[inline(always)]
    fn index(&self, i: usize) -> &C<F> {
        &self.0[i]
    }
}

/// Spin-½ Lorentz representation.
///
/// This is a trait to allow for multiple concrete bases (e.g. Weyl, Dirac) to be implemented.
pub trait SpinorRepr<F: Real>: LorentzRepr<F> {
    /// Left-handed fermion current  `J_L^μ = v̄_out γ^μ P_L u_in`.
    fn left_current(fo: &Self, fi: &Self) -> ComplexVector<F>;

    /// Right-handed fermion current  `J_R^μ = v̄_out γ^μ P_R u_in`.
    fn right_current(fo: &Self, fi: &Self) -> ComplexVector<F>;

    /// Left projection: `P_L = (1 - γ^5)/2` — zero the right-chiral (indices 2-3) components.
    fn project_left(self) -> Self;

    /// Right projection: `P_R = (1 + γ^5)/2` — zero the left-chiral (indices 0-1) components.
    fn project_right(self) -> Self;

    /// Scalar bilinear with chiral structure: `f̄ Γ f` where Γ ∈ {Identity, P_L, P_R}.
    ///
    /// In the Weyl basis with the `fo` swap convention:
    /// - `fo` has indices 0,1=RIGHT-chiral, 2,3=LEFT-chiral
    /// - `fi` has indices 0,1=LEFT-chiral, 2,3=RIGHT-chiral
    fn scalar_bilinear(fo: &Self, fi: &Self, chirality: Chirality) -> C<F>;
}

/// Chirality label for chiral projections and bilinears.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chirality {
    /// Left-handed: `P_L = (1 - γ^5)/2` — projects onto left-chiral (undotted Weyl) components.
    Left,
    /// Right-handed: `P_R = (1 + γ^5)/2` — projects onto right-chiral (dotted Weyl) components.
    Right,
    /// Both: identity projector — includes both chiralities.
    Both,
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
pub struct Bispinor<F: Real>(pub [C<F>; 4]);

impl_add_for_array!(Bispinor<F>, 4);
impl_mul_for_array!(Bispinor<F>, F, 4);
impl_mul_for_array!(Bispinor<F>, C<F>, 4);

impl<F: Real> Zero for Bispinor<F> {
    #[inline(always)]
    fn zero() -> Self {
        Bispinor([C::zero(); 4])
    }

    fn is_zero(&self) -> bool {
        self.0.iter().all(|x| x.is_zero())
    }
}

impl<F: Real> VectorSpace<F> for Bispinor<F> {}

impl<F: Real> Bispinor<F> {
    pub fn ixxxxx(p: LorentzVector<F>, mass: F, nhel: SpinorHelicity, nsf: Charge) -> Self {
        Bispinor {
            0: weyl_ixxxxx(p, mass, nhel, nsf),
        }
    }

    pub fn oxxxxx(p: LorentzVector<F>, mass: F, nhel: SpinorHelicity, nsf: Charge) -> Self {
        Bispinor {
            0: weyl_oxxxxx(p, mass, nhel, nsf),
        }
    }

    pub fn dirac_conjugate(&self) -> Self {
        // In the Weyl basis, the Dirac conjugate ψ̄ = ψ† γ^0
        // swaps the left and right components and takes the Hermitian conjugate:
        // ψ̄ = [χ†, ψ†] = [ψ[2]*, ψ[3]*, ψ[0]*, ψ[1]*]
        Bispinor {
            0: [
                self.0[2].conj(),
                self.0[3].conj(),
                self.0[0].conj(),
                self.0[1].conj(),
            ],
        }
    }
}

impl<F: Real> LorentzRepr<F> for Bispinor<F> {}

impl<F: Real> SpinorRepr<F> for Bispinor<F> {
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
    fn left_current(fo: &Self, fi: &Self) -> ComplexVector<F> {
        let fo = &fo.0;
        let fi = &fi.0;
        ComplexVector {
            0: [
                fo[2] * fi[0] + fo[3] * fi[1],
                -(fo[2] * fi[1] + fo[3] * fi[0]),
                ri(F::one()) * (fo[2] * fi[1] - fo[3] * fi[0]),
                -fo[2] * fi[0] + fo[3] * fi[1],
            ],
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
    fn right_current(fo: &Self, fi: &Self) -> ComplexVector<F> {
        let fo = &fo.0;
        let fi = &fi.0;
        ComplexVector {
            0: [
                fo[0] * fi[2] + fo[1] * fi[3],
                fo[0] * fi[3] + fo[1] * fi[2],
                -ri(F::one()) * (fo[0] * fi[3] - fo[1] * fi[2]),
                fo[0] * fi[2] - fo[1] * fi[3],
            ],
        }
    }

    /// Left projection: zero the right-chiral (indices 2-3) components, keeping left-chiral (0-1).
    fn project_left(self) -> Self {
        Bispinor([
            self.0[0],
            self.0[1],
            C::new(F::zero(), F::zero()),
            C::new(F::zero(), F::zero()),
        ])
    }

    /// Right projection: zero the left-chiral (indices 0-1) components, keeping right-chiral (2-3).
    fn project_right(self) -> Self {
        Bispinor([
            C::new(F::zero(), F::zero()),
            C::new(F::zero(), F::zero()),
            self.0[2],
            self.0[3],
        ])
    }

    /// Scalar bilinear contraction: `f̄ Γ f` where `Γ` encodes chirality.
    ///
    /// With `fo` having the sfomeg swap convention (indices 0,1=RIGHT-chiral, 2,3=LEFT-chiral)
    /// and `fi` with indices 0,1=LEFT-chiral, 2,3=RIGHT-chiral:
    /// - Left (P_L): `fi_left · fo_left = fi[0]·fo[2] + fi[1]·fo[3]`
    /// - Right (P_R): `fi_right · fo_right = fi[2]·fo[0] + fi[3]·fo[1]`
    /// - Both (Identity): both left and right contractions.
    fn scalar_bilinear(fo: &Self, fi: &Self, chirality: Chirality) -> C<F> {
        let fo = &fo.0;
        let fi = &fi.0;
        let result = match chirality {
            Chirality::Left => fi[0] * fo[2] + fi[1] * fo[3],
            Chirality::Right => fi[2] * fo[0] + fi[3] * fo[1],
            Chirality::Both => (fi[0] * fo[2] + fi[1] * fo[3]) + (fi[2] * fo[0] + fi[3] * fo[1]),
        };
        result
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers: the actual WeylBasis numerics (moved from repr.rs)
// ─────────────────────────────────────────────────────────────────────────────

/// Incoming fermion wavefunction (column spinor).
///
/// Mirrors Fortran `ixxxxx` exactly.
fn weyl_ixxxxx<F: Real>(
    p: LorentzVector<F>,
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
            let omega0 = (p[0] + pp).sqrt();
            let omega = [omega0, mass / omega0];

            let ip = ((1 + nh) / 2) as usize;
            let im = ((1 - nh) / 2) as usize;

            let sfomeg = [r(sf[0] * omega[ip]), r(sf[1] * omega[im])];

            let pp3 = (pp + p[3]).max(F::zero());
            let chi0 = r((pp3 / (two * pp)).sqrt());
            let chi1 = if pp3 > F::zero() {
                C::new(F::from(nh).unwrap() * p[1], p[2]) / r((two * pp * pp3).sqrt())
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
        let sqp0p3 = if p[1] == F::zero() && p[2] == F::zero() && p[3] < F::zero() {
            F::zero()
        } else {
            (p.e() + p[3]).max(F::zero()).sqrt() * F::from(nsf_i).unwrap()
        };
        let chi0 = r(sqp0p3);
        let chi1 = if sqp0p3 == F::zero() {
            r(F::from(-nhel.sign()).unwrap() * (two * p[0]).sqrt())
        } else {
            C::new(F::from(nh).unwrap() * p[1], p[2]) / r(sqp0p3)
        };

        if nh == 1 {
            fi[0] = r(F::zero());
            fi[1] = r(F::zero());
            fi[2] = chi0;
            fi[3] = chi1;
        } else {
            fi[0] = chi1;
            fi[1] = chi0;
            fi[2] = r(F::zero());
            fi[3] = r(F::zero());
        }
    }

    fi
}

/// Outgoing fermion wavefunction (row spinor / Dirac conjugate).
///
/// Mirrors Fortran `oxxxxx` exactly. Key differences from `ixxxxx`:
/// - `chi[1]` uses `−p[2]` (complex conjugate of the transverse phase),
/// - `sfomeg[0]` ↔ `sfomeg[1]` swapped in the component assignment,
/// - At rest: `ip_i = −((1+nh)/2)` instead of `+(1+nh)/2`.
fn weyl_oxxxxx<F: Real>(
    p: LorentzVector<F>,
    mass: F,
    nhel: SpinorHelicity,
    nsf: Charge,
) -> [C<F>; 4] {
    let two = F::one() + F::one();
    let nh = nhel.sign() * nsf.sign();
    let nsf_i = nsf.sign();

    let mut fo = [C::new(F::zero(), F::zero()); 4];

    if mass != F::zero() {
        let pp = p.p3().min(p.e());

        if pp == F::zero() {
            // ── at rest ───────────────────────────────────────────────────
            let sqm0 = mass.abs().sqrt();
            let sqm1 = sqm0 * mass.signum();
            let sqm = [sqm0, sqm1];

            let ip_i = -((1 + nh) / 2);
            let im_i = (1 - nh) / 2;
            let neg_ip = (-ip_i) as usize;
            let im = im_i as usize;

            fo[0] = r(F::from(im_i).unwrap() * sqm[im]);
            fo[1] = r(F::from(ip_i * nsf_i).unwrap() * sqm[im]);
            fo[2] = r(F::from(im_i * nsf_i).unwrap() * sqm[neg_ip]);
            fo[3] = r(F::from(ip_i).unwrap() * sqm[neg_ip]);
        } else {
            // ── massive, moving ───────────────────────────────────────────
            let sf = [
                F::from(1 + nsf_i + (1 - nsf_i) * nh).unwrap() / two,
                F::from(1 + nsf_i - (1 - nsf_i) * nh).unwrap() / two,
            ];
            let omega0 = (p[0] + pp).sqrt();
            let omega = [omega0, mass / omega0];

            let ip = ((1 + nh) / 2) as usize;
            let im = ((1 - nh) / 2) as usize;

            let sfomeg = [r(sf[0] * omega[ip]), r(sf[1] * omega[im])];

            let pp3 = (pp + p[3]).max(F::zero());
            let chi0 = r((pp3 / (two * pp)).sqrt());
            // chi[1] uses −p[2] (conjugate)
            let chi1 = if pp3 > F::zero() {
                C::new(F::from(nh).unwrap() * p[1], -p[2]) / r((two * pp * pp3).sqrt())
            } else {
                r(F::from(-nh).unwrap())
            };
            let chi = [chi0, chi1];

            // sfomeg[0] ↔ sfomeg[1] swapped vs ixxxxx
            fo[0] = sfomeg[1] * chi[im];
            fo[1] = sfomeg[1] * chi[ip];
            fo[2] = sfomeg[0] * chi[im];
            fo[3] = sfomeg[0] * chi[ip];
        }
    } else {
        // ── massless ──────────────────────────────────────────────────────
        let sqp0p3 = if p[1] == F::zero() && p[2] == F::zero() && p[3] < F::zero() {
            F::zero()
        } else {
            (p.e() + p[3]).max(F::zero()).sqrt() * F::from(nsf_i).unwrap()
        };
        let chi0 = r(sqp0p3);
        // chi[1] uses −p[2] (conjugate) and NHEL (not nh) when sqp0p3 == 0
        let chi1 = if sqp0p3 == F::zero() {
            r(F::from(-nhel.sign()).unwrap() * (two * p[0]).sqrt())
        } else {
            C::new(F::from(nh).unwrap() * p[1], -p[2]) / r(sqp0p3)
        };

        if nh == 1 {
            fo[0] = chi0;
            fo[1] = chi1;
            fo[2] = r(F::zero());
            fo[3] = r(F::zero());
        } else {
            fo[0] = r(F::zero());
            fo[1] = r(F::zero());
            fo[2] = chi1;
            fo[3] = chi0;
        }
    }

    fo
}
