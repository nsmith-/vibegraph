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
//! | [`VectorRepr`] | (½,½) | 4 | Cotangent / polarisation vector |
//! | [`ScalarRepr`] | (0,0) | 1 | Lorentz scalar |
//!
//! ## Basis choice
//!
//! [`SpinorRepr`] is parameterised by a *basis marker type* `B` (e.g.
//! [`WeylBasis`], [`DiracBasis`]). Changing the basis is a unitary rotation of
//! the fiber components; the intertwiner algebra is unchanged.
//!
//! ## `SpinorRepr` as a subtrait of `LorentzRepr`
//!
//! [`SpinorRepr<F>`] is declared as a subtrait of [`LorentzRepr<F>`] with
//! `Fiber = [C<F>; 4]`. This removes the redundant `type Spinor` associated type
//! and makes the spinor bundle fit directly into the representation hierarchy.
//! All callers use `B::Fiber` (or equivalently `[C<F>; 4]`) for spinor components.

use super::{C, Real, r, ri};

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
            SpinorHelicity::Up => write!(f, "+1"),
            SpinorHelicity::Down => write!(f, "-1"),
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

// ─────────────────────────────────────────────────────────────────────────────
// FourMomentum — typed real 4-vector for particle kinematics
// ─────────────────────────────────────────────────────────────────────────────

/// A contravariant 4-momentum vector `p^μ = [E, p_x, p_y, p_z]`.
///
/// This type wraps a real `[F; 4]` array.  It is distinct from
/// [`MinkowskiRep::Fiber`], which is a *complex* covariant polarisation vector
/// `[C<F>; 4]` (lower index, metric signs absorbed).  The distinction between
/// contravariant (upper) and covariant (lower) 4-vectors is important: momenta
/// are contravariant, while currents and polarisations are covariant.
///
/// # TODO
/// Implement `LorentzRepr<F>` for a `RealVectorRep` marker type whose fiber is
/// `FourMomentum<F>`, completing the bundle-theoretic picture for kinematics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FourMomentum<F: Real>(pub [F; 4]);

impl<F: Real> FourMomentum<F> {
    /// Construct from individual components `[E, px, py, pz]`.
    #[inline(always)]
    pub fn new(e: F, px: F, py: F, pz: F) -> Self {
        FourMomentum([e, px, py, pz])
    }

    /// Zero momentum (useful as a dummy argument for algebraic intertwiners).
    #[inline(always)]
    pub fn zero() -> Self {
        FourMomentum([F::zero(); 4])
    }

    /// Energy component E = p^0.
    #[inline(always)]
    pub fn e(self) -> F {
        self.0[0]
    }

    /// Return the signed momentum `self * sign`, used by wavefunction factories.
    #[inline(always)]
    pub fn scaled(self, sign: i32) -> Self {
        let s = F::from(sign).unwrap();
        FourMomentum([self.0[0] * s, self.0[1] * s, self.0[2] * s, self.0[3] * s])
    }
}

impl<F: Real> std::ops::Index<usize> for FourMomentum<F> {
    type Output = F;
    #[inline(always)]
    fn index(&self, i: usize) -> &F {
        &self.0[i]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// LorentzRepr — base trait
// ─────────────────────────────────────────────────────────────────────────────

/// Base trait for a Lorentz representation.
///
/// A type implementing this trait is a *marker* for a specific vector bundle
/// over momentum space. The associated [`Fiber`](LorentzRepr::Fiber) type holds
/// the numerical data for one wavefunction section (one "slot" at a vertex leg).
///
/// # Type parameters
/// - `F` — the real scalar field (e.g. `f64`)
pub trait LorentzRepr<F: Real>: Sized + Copy + 'static {
    /// The concrete type that stores one section of this bundle.
    ///
    /// For a spin-½ field: `[C<F>; 4]` (Dirac spinor).
    /// For a spin-1 field: `[C<F>; 4]` (polarisation vector).
    /// For a spin-0 field: `C<F>` (complex scalar).
    type Fiber: Copy + std::fmt::Debug;
}

// ─────────────────────────────────────────────────────────────────────────────
// SpinorRepr — spin-½
// ─────────────────────────────────────────────────────────────────────────────

/// Spin-½ Lorentz representation with explicit basis choice.
///
/// A type `B` implementing this trait is a marker for a particular basis of the
/// Dirac spinor bundle `S_L ⊕ S_R`. Because `SpinorRepr<F>` is a subtrait of
/// `LorentzRepr<F, Fiber = [C<F>; 4]>`, the fiber type is always four complex
/// components; use `B::Fiber` (= `[C<F>; 4]`) in place of the former `B::Spinor`.
///
/// # Factories
/// - [`ixxxxx`](SpinorRepr::ixxxxx) — incoming column spinor `u(p,λ)` or `v(p,λ)`
/// - [`oxxxxx`](SpinorRepr::oxxxxx) — outgoing row spinor `ū(p,λ)` or `v̄(p,λ)`
///
/// # Bilinear currents
/// - [`left_current`](SpinorRepr::left_current) — `J_L^μ = v̄_out γ^μ P_L u_in`
/// - [`right_current`](SpinorRepr::right_current) — `J_R^μ = v̄_out γ^μ P_R u_in`
///
/// The currents return *contravariant* 4-vector components; the [`crate::helas::vertex`]
/// functions contract them with polarisation vectors using the Minkowski metric.
pub trait SpinorRepr<F: Real>: LorentzRepr<F, Fiber = [C<F>; 4]> {
    /// Flowing-IN spinor wavefunction (HELAS `ixxxxx`).
    ///
    /// Constructs a column Dirac spinor `u(p, λ)` (particle) or `v(p, λ)`
    /// (antiparticle) for a particle flowing *into* the diagram.
    ///
    /// - `p` = `[E, px, py, pz]` (contravariant 4-momentum)
    /// - `mass` ≥ 0; use `mass = 0` for massless particles
    /// - `nhel` — helicity: [`SpinorHelicity::Up`] (+1) or [`SpinorHelicity::Down`] (−1)
    /// - `nsf` — [`Charge::Particle`] (+1) or [`Charge::Antiparticle`] (−1)
    fn ixxxxx(p: FourMomentum<F>, mass: F, nhel: SpinorHelicity, nsf: Charge) -> [C<F>; 4];

    /// Flowing-OUT spinor wavefunction (HELAS `oxxxxx`).
    ///
    /// Constructs the Dirac-conjugate row spinor `ū(p, λ)` or `v̄(p, λ)`
    /// for a particle flowing *out of* the diagram.
    ///
    /// # Relation to `ixxxxx`
    ///
    /// The outgoing wavefunction is the Dirac conjugate of the incoming:
    /// `ū(p,λ) = u(p,λ)† γ^0`. In the Weyl basis `γ^0 = [[0, I₂],[I₂, 0]]`,
    /// this swaps the left- and right-chiral blocks and complex-conjugates
    /// the transverse phase (replacing `p_y → −p_y` in the `χ₁` spinor and
    /// exchanging `sfomeg[0] ↔ sfomeg[1]`).  The two factories could be unified
    /// by implementing `oxxxxx` as `ixxxxx` followed by Dirac conjugation, but
    /// keeping them separate avoids the extra allocation.
    fn oxxxxx(p: FourMomentum<F>, mass: F, nhel: SpinorHelicity, nsf: Charge) -> [C<F>; 4];

    /// Left-handed fermion current  `J_L^μ = v̄_out γ^μ P_L u_in`.
    ///
    /// In the Weyl basis, with component ordering
    /// `[ψ_L^0, ψ_L^1, ψ_R^{0̇}, ψ_R^{1̇}]`, the left current is:
    ///
    /// \`\`\`text
    /// J_L^μ = (σ̄^μ)_{α̇β} fo[2+α̇] fi[β]
    /// \`\`\`
    ///
    /// where `σ̄^0 = I₂`, `σ̄^i = −σ^i` (Pauli), and `fo[2:4]` / `fi[0:2]`
    /// are the right-chiral / left-chiral Weyl spinor components.
    fn left_current(fo: &[C<F>; 4], fi: &[C<F>; 4]) -> [C<F>; 4];

    /// Right-handed fermion current  `J_R^μ = v̄_out γ^μ P_R u_in`.
    ///
    /// In the Weyl basis:
    ///
    /// \`\`\`text
    /// J_R^μ = (σ^μ)^{αβ̇} fo[α] fi[2+β̇]
    /// \`\`\`
    ///
    /// where `σ^0 = I₂`, `σ^i = +σ^i` (Pauli), and `fo[0:2]` / `fi[2:4]`
    /// are the left-chiral / right-chiral Weyl spinor components.
    fn right_current(fo: &[C<F>; 4], fi: &[C<F>; 4]) -> [C<F>; 4];
}

// ─────────────────────────────────────────────────────────────────────────────
// VectorRepr — spin-1
// ─────────────────────────────────────────────────────────────────────────────

/// Spin-1 Lorentz representation: a polarisation vector in `T*M`.
///
/// A type `B` implementing this trait is a marker for a spin-1 (gauge boson)
/// wavefunction basis. The fiber is always `[C<F>; 4]` (four covariant
/// polarisation components with metric signs absorbed per the HELAS convention).
///
/// The factory [`vxxxxx`](VectorRepr::vxxxxx) mirrors the HELAS `vxxxxx` routine.
///
/// # TODO
/// - Implement `vxxxxx` for [`MinkowskiRep`] (HELAS polarisation vectors)
/// - Add `jxxxxx` (off-shell vector current factory) and `vvvxxx` (3-boson vertex)
pub trait VectorRepr<F: Real>: LorentzRepr<F, Fiber = [C<F>; 4]> {
    /// External on-shell vector (polarisation) wavefunction (HELAS `vxxxxx`).
    ///
    /// - `p` = `[E, px, py, pz]` (contravariant 4-momentum)
    /// - `mass` = 0 for photon/gluon; > 0 for W/Z
    /// - `nhel = ±1` (transverse), or `0` (longitudinal, massive only)
    /// - `nsv = +1` for initial-state, `−1` for final-state
    fn vxxxxx(p: FourMomentum<F>, mass: F, nhel: i32, nsv: i32) -> [C<F>; 4];
}

// ─────────────────────────────────────────────────────────────────────────────
// ScalarRepr — spin-0
// ─────────────────────────────────────────────────────────────────────────────

/// Spin-0 Lorentz representation: a complex scalar wavefunction.
///
/// The fiber is `C<F>`. For an external on-shell scalar the wavefunction is
/// trivially `1 + 0i`; off-shell scalars carry a non-trivial complex value
/// set by the propagator.
///
/// # TODO
/// - Implement `sxxxxx` for [`ScalarField`]
pub trait ScalarRepr<F: Real>: LorentzRepr<F, Fiber = C<F>> {
    /// External scalar wavefunction (HELAS `sxxxxx`).
    ///
    /// For an on-shell external scalar, returns `1 + 0i`. The momentum is stored
    /// alongside the wavefunction (by the caller in a `ScalarWf` struct, to be
    /// added to `wavefn.rs`).
    ///
    /// - `p` = `[E, px, py, pz]` (unused for on-shell case but provided
    ///   for consistency with the HELAS calling convention)
    /// - `nsf` — [`Charge::Particle`] or [`Charge::Antiparticle`]
    fn sxxxxx(p: FourMomentum<F>, nsf: Charge) -> C<F>;
}

// ─────────────────────────────────────────────────────────────────────────────
// Weyl (chiral) basis — canonical implementation of SpinorRepr + LorentzRepr
// ─────────────────────────────────────────────────────────────────────────────

/// Marker type selecting the Weyl (chiral) basis.
///
/// In this basis the 4-component Dirac spinor is arranged as
/// `[ψ_0, ψ_1, χ^{0̇}, χ^{1̇}]` where `ψ_α` (indices 0–1) are the left-chiral
/// (undotted) Weyl spinor components and `χ^{α̇}` (indices 2–3) are the
/// right-chiral (dotted) components.
///
/// This choice makes the Lorentz decomposition `S = S_L ⊕ S_R` manifest, which
/// is the geometrically natural basis for computing helicity amplitudes.
///
/// The `left_current` and `right_current` implementations match the Fortran
/// HELAS routines `iovxxx` lines 86–89 exactly.
#[derive(Clone, Copy, Debug)]
pub struct WeylBasis;

impl<F: Real> LorentzRepr<F> for WeylBasis {
    type Fiber = [C<F>; 4];
}

impl<F: Real> SpinorRepr<F> for WeylBasis {
    fn ixxxxx(p: FourMomentum<F>, mass: F, nhel: SpinorHelicity, nsf: Charge) -> [C<F>; 4] {
        weyl_ixxxxx(p, mass, nhel, nsf)
    }

    fn oxxxxx(p: FourMomentum<F>, mass: F, nhel: SpinorHelicity, nsf: Charge) -> [C<F>; 4] {
        weyl_oxxxxx(p, mass, nhel, nsf)
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
    fn left_current(fo: &[C<F>; 4], fi: &[C<F>; 4]) -> [C<F>; 4] {
        [
            fo[2] * fi[0] + fo[3] * fi[1],
            -(fo[2] * fi[1] + fo[3] * fi[0]),
            ri(F::one()) * (fo[2] * fi[1] - fo[3] * fi[0]),
            -fo[2] * fi[0] + fo[3] * fi[1],
        ]
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
    fn right_current(fo: &[C<F>; 4], fi: &[C<F>; 4]) -> [C<F>; 4] {
        [
            fo[0] * fi[2] + fo[1] * fi[3],
            fo[0] * fi[3] + fo[1] * fi[2],
            -ri(F::one()) * (fo[0] * fi[3] - fo[1] * fi[2]),
            fo[0] * fi[2] - fo[1] * fi[3],
        ]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Dirac basis — stub
// ─────────────────────────────────────────────────────────────────────────────

/// Marker type selecting the Dirac (standard) basis.
///
/// In the Dirac basis, γ^0 is block-diagonal with `(I, -I)` blocks and the
/// large/small components separate in the non-relativistic limit:
///
/// \`\`\`text
/// γ^0 = diag(I₂, -I₂),   γ^i = [[0, σ^i], [-σ^i, 0]]
/// \`\`\`
///
/// The relationship to [`WeylBasis`] is a unitary `4×4` rotation:
/// `ψ_Dirac = U ψ_Weyl` where `U = [[I, I], [I, -I]] / √2`.
///
/// # TODO
/// Implement `ixxxxx`, `oxxxxx`, `left_current`, `right_current` in the Dirac
/// basis. The implementations should be related to `WeylBasis` by applying `U`
/// before and `U†` after each computation.
#[derive(Clone, Copy, Debug)]
pub struct DiracBasis;

impl<F: Real> LorentzRepr<F> for DiracBasis {
    type Fiber = [C<F>; 4];
}

impl<F: Real> SpinorRepr<F> for DiracBasis {
    fn ixxxxx(_p: FourMomentum<F>, _mass: F, _nhel: SpinorHelicity, _nsf: Charge) -> [C<F>; 4] {
        todo!("DiracBasis::ixxxxx — apply U to WeylBasis::ixxxxx result")
    }

    fn oxxxxx(_p: FourMomentum<F>, _mass: F, _nhel: SpinorHelicity, _nsf: Charge) -> [C<F>; 4] {
        todo!("DiracBasis::oxxxxx — apply U to WeylBasis::oxxxxx result")
    }

    fn left_current(_fo: &[C<F>; 4], _fi: &[C<F>; 4]) -> [C<F>; 4] {
        todo!("DiracBasis::left_current — rotate to Weyl, compute, rotate back")
    }

    fn right_current(_fo: &[C<F>; 4], _fi: &[C<F>; 4]) -> [C<F>; 4] {
        todo!("DiracBasis::right_current — rotate to Weyl, compute, rotate back")
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MinkowskiRep — spin-1 marker
// ─────────────────────────────────────────────────────────────────────────────

/// Marker type for the Minkowski (covariant) vector representation.
///
/// Selects the standard `(½,½)` representation of Spin(1,3): a 4-vector
/// `A^μ = [A^0, A^1, A^2, A^3]` stored with metric signs absorbed (HELAS
/// convention). Implements [`VectorRepr<F>`].
///
/// # TODO
/// Implement [`VectorRepr::vxxxxx`] — the external polarisation vector:
/// - Massless (nhel = ±1): circular polarisation `ε^±`
/// - Massive (nhel = ±1,0): three physical polarisations including longitudinal
#[derive(Clone, Copy, Debug)]
pub struct MinkowskiRep;

impl<F: Real> LorentzRepr<F> for MinkowskiRep {
    type Fiber = [C<F>; 4];
}

impl<F: Real> VectorRepr<F> for MinkowskiRep {
    fn vxxxxx(_p: FourMomentum<F>, _mass: F, _nhel: i32, _nsv: i32) -> [C<F>; 4] {
        todo!("MinkowskiRep::vxxxxx — construct external polarisation vector")
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ScalarField — spin-0 marker
// ─────────────────────────────────────────────────────────────────────────────

/// Marker type for the trivial (scalar) Lorentz representation.
///
/// Implements [`ScalarRepr<F>`]. The fiber is a single complex number `C<F>`,
/// equal to `1+0i` for on-shell external scalars.
///
/// # TODO
/// Implement [`ScalarRepr::sxxxxx`]. For an on-shell external scalar with unit
/// normalisation the wavefunction is simply `1.0 + 0.0i`.
#[derive(Clone, Copy, Debug)]
pub struct ScalarField;

impl<F: Real> LorentzRepr<F> for ScalarField {
    type Fiber = C<F>;
}

impl<F: Real> ScalarRepr<F> for ScalarField {
    fn sxxxxx(_p: FourMomentum<F>, _nsf: Charge) -> C<F> {
        todo!("ScalarField::sxxxxx — return 1+0i for on-shell external scalar")
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers: the actual WeylBasis numerics (moved from repr.rs)
// ─────────────────────────────────────────────────────────────────────────────

/// Incoming fermion wavefunction (column spinor).
///
/// Mirrors Fortran `ixxxxx` exactly.
fn weyl_ixxxxx<F: Real>(
    p: FourMomentum<F>,
    mass: F,
    nhel: SpinorHelicity,
    nsf: Charge,
) -> [C<F>; 4] {
    let two = F::one() + F::one();
    let nh = nhel.sign() * nsf.sign();
    let nsf_i = nsf.sign();

    let mut fi = [C::new(F::zero(), F::zero()); 4];

    if mass != F::zero() {
        let pp = (p[1] * p[1] + p[2] * p[2] + p[3] * p[3]).sqrt().min(p[0]);

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
            (p[0] + p[3]).max(F::zero()).sqrt() * F::from(nsf_i).unwrap()
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
    p: FourMomentum<F>,
    mass: F,
    nhel: SpinorHelicity,
    nsf: Charge,
) -> [C<F>; 4] {
    let two = F::one() + F::one();
    let nh = nhel.sign() * nsf.sign();
    let nsf_i = nsf.sign();

    let mut fo = [C::new(F::zero(), F::zero()); 4];

    if mass != F::zero() {
        let pp = (p[1] * p[1] + p[2] * p[2] + p[3] * p[3]).sqrt().min(p[0]);

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
            (p[0] + p[3]).max(F::zero()).sqrt() * F::from(nsf_i).unwrap()
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
