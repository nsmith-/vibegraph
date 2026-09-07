//! Wavefunction types for HELAS external legs and off-shell currents.
//!
//! ## Conventions
//!
//! The conventions are those of the HELAS manual (KEK report 91-11, Murayama,
//! Watanabe and Hagiwara), Appendix A: the constructors below are transcriptions
//! of its `ixxxxx`, `vxxxxx` and `sxxxxx` routines, so that an amplitude built
//! from them agrees with MadGraph's generated code diagram by diagram and
//! helicity by helicity, phases included.
//!
//! Throughout, the metric is $(+,-,-,-)$, four-vectors are laid out as
//! $[E, p_x, p_y, p_z]$, and a wavefunction's components are stored in that same
//! order — $[\epsilon^0, \epsilon^1, \epsilon^2, \epsilon^3]$ for a vector, and
//! the four Weyl components (0,1 left-chiral; 2,3 right-chiral) for a spinor.
//! HELAS packs the momentum into two extra complex entries of the same array;
//! here it is a separate `momentum` field.
//!
//! ### Momentum-flow signs
//!
//! Every constructor takes a flow flag, and stores the flag times the momentum
//! rather than the momentum itself. The four-momentum passed in is always the
//! physical one; only the stored copy is signed.
//!
//! | constructor | flag | $+1$ | $-1$ | stored |
//! |---|---|---|---|---|
//! | [`DiracWf::from_momentum`] | `nsf`: [`Charge`] | [`Charge::Particle`], a $u$ spinor | [`Charge::Antiparticle`], a $v$ spinor | $n_{sf}\\,p$ |
//! | [`VectorWf::vxxxxx`] | `nsv`: `i32` | outgoing leg | incoming leg | $n_{sv}\\,p$ |
//! | [`ScalarWf::sxxxxx`] | `nss`: `i32` | outgoing leg | incoming leg | $n_{ss}\\,p$ |
//!
//! `nsv` and `nss` panic on any value other than $\pm 1$. The signed momentum is
//! what lets the off-shell-current routines in [`crate::helas::vertex`] add and
//! subtract leg momenta directly to obtain the momentum of the internal line;
//! [`DiracWf::charge`] reads the flag back off the sign of the stored energy.
//!
//! ### Spinor wavefunctions
//!
//! For Dirac spinors, we use the Weyl (chiral) basis where the $\gamma^5$ matrix is:
//! $$
//! \gamma^5 = \begin{pmatrix} -\mathbb{1} & 0 \\\\ 0 & \mathbb{1} \end{pmatrix}.
//! $$
//! where $\mathbb{1}$ is the 2×2 identity matrix.  This means the upper two components
//! of a Dirac spinor are left-chiral and the lower two are right-chiral.
//!
//! Correspondingly, the $\gamma^\mu$ matrices are defined as
//! $$
//! \gamma^\mu = \begin{pmatrix} 0 & \sigma^\mu \\\\ \bar{\sigma}^\mu & 0 \end{pmatrix},
//! $$
//! where $\sigma^\mu = (1, \vec{\sigma})$ and $\bar{\sigma}^\mu = (1, -\vec{\sigma})$ are
//! the Pauli matrices.
//!
//! The helicity eigenspinors $\chi_\pm$ for [`SpinorHelicity`] are defined as
//! $$
//! \chi_+(\vec{p}) = \frac{1}{\sqrt{2|\vec{p}|(|\vec{p}| + p_z)}} \begin{pmatrix} |\vec{p}| + p_z \\\\ p_x + i p_y \end{pmatrix}, \quad \chi_-(\vec{p}) = \frac{1}{\sqrt{2|\vec{p}|(|\vec{p}| + p_z)}} \begin{pmatrix} -p_x + i p_y \\\\ |\vec{p}| + p_z \end{pmatrix},
//! $$
//! where $\vec{p} = (p_x, p_y, p_z)$ is the 3-momentum. These eigenspinors satisfy
//! $$
//! \frac{\vec{\sigma} \cdot \vec{p}}{|\vec{p}|} \chi_\pm(\vec{p}) = \pm \chi_\pm(\vec{p}).
//! $$
//! For particles with $|\vec{p}| = -p_z$, where that normalisation is singular, the
//! limit $p_y = 0$, $p_x \to 0^-$ is taken to define the helicity eigenspinors,
//! which ensures a smooth limit as $|\vec{p}| \to 0$. Specifically, in this limit we have
//! $$
//! \chi_+(\vec{p}) \to \begin{pmatrix} 0 \\\\ -1 \end{pmatrix}, \quad \chi_-(\vec{p}) \to \begin{pmatrix} 1 \\\\ 0 \end{pmatrix}.
//! $$
//! The side of the limit is a convention, not a derived fact: approaching along
//! $p_x \to 0^+$ instead would negate both spinors, and that phase is visible in a
//! per-diagram comparison against the reference even though $|\mathcal{M}|^2$ is
//! blind to it.
//!
//! The Dirac bispinors $u$ and $v$ are then constructed from the helicity eigenspinors as follows:
//! $$
//! u(p) = \begin{pmatrix} \omega_\mp(p)\chi_\pm(\vec{p}) \\\\ \omega_\pm(p)\chi_\pm(\vec{p}) \end{pmatrix}, \quad v(p) = \begin{pmatrix} \mp\omega_\pm(p)\chi_\mp(\vec{p}) \\\\ \pm \omega_\mp(p)\chi_\mp(\vec{p}) \end{pmatrix},
//! $$
//!
//! where $\omega_\pm(p) = \sqrt{E \pm |\vec{p}|}$ are the energy-dependent factors that appear in the construction of the spinors.
//! The $u$ spinor will be used for [`Charge::Particle`] and the $v$ spinor for [`Charge::Antiparticle`] in the HELAS convention.
//!
//! Two limits of that construction are computed by their own branches.
//!
//! **At rest** ($\vec{p} = 0$, massive), where $\omega_+ = \omega_- = \sqrt{m}$ and
//! $\chi_\pm$ is undefined, the spinor is the $\vec{p} \to 0$ limit taken along
//! $+\hat{z}$, i.e. with $\chi_+ = (1, 0)^T$ and $\chi_- = (0, 1)^T$. A negative
//! `mass` is admitted: the two chiral blocks are built from $\sqrt{|m|}$ and
//! $\sqrt{|m|}\\,\mathrm{sgn}(m)$, so a negative `mass` flips their relative sign,
//! which is how HELAS carries a fermion whose mass parameter has been given a
//! negative sign.
//!
//! **Massless** (`mass == 0`), where $\omega_- = 0$ and $\omega_+ = \sqrt{2E}$, so the
//! bispinor is purely chiral: with $n_h = \lambda\\, n_{sf}$ the product of the helicity
//! and charge signs, only components 2 and 3 (right-chiral) are populated for
//! $n_h = +1$ and only components 0 and 1 (left-chiral) for $n_h = -1$, the other two
//! being exactly zero. This is what makes chirality-violating helicity combinations of a
//! massless process vanish bit-exactly rather than to rounding, and is the reason
//! helicity filtering can drop them. The $|\vec{p}| = -p_z$ limit above applies here
//! too, in the same $p_x \to 0^-$ form.
//!
//! ### Vector wavefunctions
//!
//! [`VectorWf::vxxxxx`] builds the polarisation vector of an external spin-1
//! particle of mass $m$ (`vmass`) and helicity $\lambda$ (`nhel`, one of
//! $-1, 0, +1$). Write $p_T = \sqrt{p_x^2 + p_y^2}$ and let
//! $$
//! \hat e_\theta = \frac{1}{|\vec{p}|\\,p_T}\left(p_x p_z,\\; p_y p_z,\\; -p_T^2\right),
//! \qquad
//! \hat e_\varphi = \frac{1}{p_T}\left(-p_y,\\; p_x,\\; 0\right)
//! $$
//! be the polar and azimuthal unit vectors of $\vec{p}$, so that
//! $(\hat e_\theta, \hat e_\varphi, \hat{p})$ is a right-handed orthonormal triad.
//!
//! **Transverse states** ($\lambda = \pm 1$), massive and massless alike:
//! $$
//! \epsilon^0 = 0, \qquad
//! \vec{\epsilon}(p, \lambda) = \frac{1}{\sqrt{2}}\left(-\lambda\\,\hat e_\theta + i\\,n_{sv}\\,\hat e_\varphi\right),
//! $$
//! that is,
//! $$
//! \epsilon^1 = \frac{1}{\sqrt{2}}\left(-\lambda\frac{p_x p_z}{|\vec{p}|\\,p_T} - i\\,n_{sv}\frac{p_y}{p_T}\right),
//! \quad
//! \epsilon^2 = \frac{1}{\sqrt{2}}\left(-\lambda\frac{p_y p_z}{|\vec{p}|\\,p_T} + i\\,n_{sv}\frac{p_x}{p_T}\right),
//! \quad
//! \epsilon^3 = \frac{\lambda}{\sqrt{2}}\frac{p_T}{|\vec{p}|}.
//! $$
//!
//! **Longitudinal state** ($\lambda = 0$; massive only):
//! $$
//! \epsilon^\mu(p, 0) = \frac{1}{m}\left(|\vec{p}|,\\; \frac{E}{|\vec{p}|}\\,\vec{p}\right).
//! $$
//! It is real and independent of $n_{sv}$.
//!
//! **At rest** ($\vec{p} = 0$, massive), where the triad is undefined, the routine
//! returns the $\hat{p} \to \hat{z}$ limit of those expressions:
//! $$
//! \epsilon^\mu(p, \pm 1) = \frac{1}{\sqrt{2}}\left(0,\\; -\lambda,\\; i\\,n_{sv},\\; 0\right),
//! \qquad
//! \epsilon^\mu(p, 0) = (0, 0, 0, 1).
//! $$
//!
//! **Along the $z$ axis** ($p_T = 0$, $\vec{p} \neq 0$), where $\hat e_\theta$ and
//! $\hat e_\varphi$ are undefined, the routine uses $\hat e_\theta = \hat{x}$ and
//! $\hat e_\varphi = \mathrm{sgn}(p_z)\\,\hat{y}$, so the transverse states are
//! $$
//! \epsilon^1 = -\frac{\lambda}{\sqrt{2}}, \qquad
//! \epsilon^2 = i\\,n_{sv}\frac{\mathrm{sgn}(p_z)}{\sqrt{2}}, \qquad
//! \epsilon^3 = 0,
//! $$
//! with $\mathrm{sgn}(0) = +1$; the longitudinal formula is unchanged. That triad is
//! the limit of the general expressions at $p_y = 0$ with $p_x \to 0$ from the side
//! on which $p_x$ and $p_z$ share a sign. Approaching from the other side would
//! negate the whole transverse vector, so, as for the spinors, the side is a HELAS
//! convention that has to be kept.
//!
//! **Incoming versus outgoing.** $n_{sv}$ enters only through the imaginary parts,
//! so `vxxxxx(p, m, nhel, -1)` and `vxxxxx(p, m, nhel, +1)` are componentwise
//! complex conjugates of each other (and their stored momenta differ in sign).
//! Read with $n_{sv} = -1$ the formulas above are the usual $\epsilon^\mu(p, \lambda)$
//! of an incoming boson; with $n_{sv} = +1$ they give $\epsilon^\mu(p, \lambda)^\ast$,
//! the conjugate that an outgoing boson contributes to the amplitude.
//!
//! **Normalisation and completeness.** For every helicity and either flow sign,
//! $$
//! \epsilon(p, \lambda) \cdot p = 0,
//! \qquad
//! \epsilon(p, \lambda) \cdot \epsilon(p, \lambda^{\prime})^\ast = -\delta_{\lambda\lambda^{\prime}} .
//! $$
//! A massive vector's three states are complete in the usual sense,
//! $$
//! \sum_{\lambda = -1, 0, +1} \epsilon^\mu(p, \lambda)\\,\epsilon^\nu(p, \lambda)^\ast
//!   = -g^{\mu\nu} + \frac{p^\mu p^\nu}{m^2},
//! $$
//! matching the unitary-gauge propagator numerator. A massless vector has only
//! $\lambda = \pm 1$, and this basis fixes $\epsilon^0 = 0$, the temporal gauge of
//! the frame the momenta are given in, so with $n^\mu = (1, 0, 0, 0)$
//! $$
//! \sum_{\lambda = \pm 1} \epsilon^\mu(p, \lambda)\\,\epsilon^\nu(p, \lambda)^\ast
//!   = -g^{\mu\nu} + \frac{p^\mu n^\nu + n^\mu p^\nu}{p \cdot n} - \frac{p^\mu p^\nu}{(p \cdot n)^2}.
//! $$
//! Every term beyond $-g^{\mu\nu}$ carries a $p^\mu$ or $p^\nu$ and so drops out of a
//! gauge-invariant sum by the Ward identity. Both relations are real, hence hold for
//! either sign of $n_{sv}$.
//!
//! **Edge cases.** The massive branch clamps $|\vec{p}|$ to $\min(E, |\vec{p}|)$ and
//! $p_T$ to $\min(|\vec{p}|, p_T)$, and the massless branch uses $E$ in place of
//! $|\vec{p}|$; on-shell momenta satisfy $E \ge |\vec{p}| \ge p_T$ with equality in
//! the first for $m = 0$, so the clamps bite only on input whose energy and
//! three-momentum disagree at rounding level, and both branches then agree with the
//! formulas above. The massless branch scales the imaginary parts by $n_{sv}$ rather
//! than $n_{sv}|\lambda|$, so `nhel = 0` with `vmass = 0` returns a non-zero vector
//! that is not a polarisation state: a massless vector has no longitudinal mode and
//! the routine is meaningful there only for `nhel` $= \pm 1$.
//!
//! ### Scalar wavefunctions
//!
//! HELAS gives an external scalar the wavefunction $1$ — all of the dynamics sits in
//! the couplings and propagators — so [`ScalarWf::sxxxxx`] stores `value = 1 + 0i`
//! and carries the flow-signed momentum $n_{ss}\\,p$ that downstream vertex routines
//! need for routing. It has no helicity argument.
use crate::helas::repr::lorentz::{
    Bispinor, Bra, ComplexVector, Contravariant, Covariant, DiracAdjoint, Ket, LorentzVector,
    SpinorRepr, Variance,
};
use crate::helas::repr::numbers::{Charge, Chirality, SpinorHelicity};
use crate::helas::repr::{Real, C};
use num_traits::Zero;

// ──────────────────────────────────────────────────────────────────────────────
// Spinor wavefunction
// ──────────────────────────────────────────────────────────────────────────────

/// A Dirac spinor wavefunction together with its (signed) 4-momentum.
///
/// `momentum` stores `p * nsf.sign()`: positive for particles, negative for
/// antiparticles.  This matches the HELAS convention used when building
/// currents and computing the s-channel propagator momentum.
///
/// `spinor` has type `B::Fiber` (= `[C<F>; 4]` for any `B: SpinorRepr<F>`),
/// since [`SpinorRepr<F>`] is a subtrait of [`crate::helas::repr::lorentz::LorentzRepr<F>`]
/// with `Fiber = [C<F>; 4]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiracWf<F: Real, Adj: DiracAdjoint> {
    pub spinor: Bispinor<F, Adj>,
    /// Signed momentum: particle → +p, antiparticle → −p
    pub momentum: LorentzVector<F, Contravariant>,
}

/// Flowing-IN typed spinor wavefunction.
pub type InDiracWf<F> = DiracWf<F, Ket>;

/// Flowing-OUT typed spinor wavefunction.
pub type OutDiracWf<F> = DiracWf<F, Bra>;

impl<F: Real, Adj: DiracAdjoint> DiracWf<F, Adj> {
    /// Build the $u$ or $v$ spinor of an on-shell external leg from its momentum,
    /// mass, helicity and charge, and store the flow-signed momentum `nsf * p`
    /// alongside it.
    ///
    /// The construction, including the massless and at-rest special cases, is the
    /// one described in the [module documentation](self). It transcribes the HELAS
    /// `ixxxxx` routine; the `Bra` instantiation is its Dirac conjugate
    /// $\bar{\psi} = \psi^\dagger \gamma^0$, the crate's `oxxxxx`.
    pub fn from_momentum(
        p: LorentzVector<F, Contravariant>,
        mass: F,
        nhel: SpinorHelicity,
        nsf: Charge,
    ) -> Self {
        let spinor = Bispinor::from_momentum(p, mass, nhel, nsf);
        Self {
            spinor,
            momentum: match nsf {
                Charge::Particle => p,      // u-spinor: stores +p
                Charge::Antiparticle => -p, // v-spinor: stores -p
            },
        }
    }

    /// Pair an already-built bispinor with a momentum, unchanged.
    ///
    /// The momentum is stored verbatim, so the caller is responsible for the
    /// flow sign; off-shell currents use this to wrap their own output.
    pub fn from_spinor(
        spinor: Bispinor<F, Adj>,
        momentum: LorentzVector<F, Contravariant>,
    ) -> Self {
        Self { spinor, momentum }
    }

    /// Return the charge (particle vs antiparticle) based on the sign of the energy component of the momentum.
    ///
    /// This relies on the HELAS convention that the momentum stored in the wavefunction is `p * nsf.sign()`,
    /// where `nsf` is the charge sign parameter used when constructing the spinor.
    pub fn charge(&self) -> Charge {
        if self.momentum.e().is_sign_positive() {
            Charge::Particle
        } else {
            Charge::Antiparticle
        }
    }

    /// Flip the adjoint (ket/bra) by taking the Dirac conjugate of the spinor (`u ↔ ū`).
    ///
    /// This is the bra/ket dual of the *same* physical particle, so the stored
    /// (HELAS-signed) momentum is carried through unchanged — matching the
    /// no-flip momentum routing used throughout off-shell-current evaluation.
    pub fn flip_adjoint(self) -> DiracWf<F, Adj::Dual> {
        DiracWf {
            spinor: self.spinor.dualize(),
            momentum: self.momentum,
        }
    }
}

impl<F: Real> InDiracWf<F> {
    /// Convert to a bra wavefunction by taking the Dirac conjugate of the spinor
    pub fn to_outgoing(self) -> OutDiracWf<F> {
        self.flip_adjoint()
    }
}

impl<F: Real> OutDiracWf<F> {
    /// Convert to a ket wavefunction by taking the Dirac conjugate of the spinor
    ///
    /// This is the inverse of [`InDiracWf::to_outgoing`].
    pub fn to_incoming(self) -> InDiracWf<F> {
        self.flip_adjoint()
    }

    pub fn scalar_bilinear(self, other: &InDiracWf<F>, chirality: Chirality) -> C<F> {
        Bispinor::scalar_bilinear(&self.spinor, &other.spinor, chirality)
    }

    pub fn vector_bilinear(
        self,
        other: &InDiracWf<F>,
        chirality: Chirality,
    ) -> ComplexVector<F, Contravariant> {
        Bispinor::vector_bilinear(&self.spinor, &other.spinor, chirality)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Vector (gauge boson) wavefunction
// ──────────────────────────────────────────────────────────────────────────────

/// An off-shell vector wavefunction: 4 complex polarisation components plus
/// the associated 4-momentum.
///
/// Used as both the result of `j3xxxx` and the input to `iovxxx`.
///
/// The polarisation carries its [`Variance`] in the type (`V`, default
/// [`Contravariant`]). External legs and the P-less off-shell currents are
/// contravariant `ε^μ`; index-lowering vertex/propagator kernels produce the
/// covariant `ε_μ`, so the raise/lower at the propagator seam is type-checked
/// rather than hand-coded. The `momentum` is always the physical contravariant
/// 4-momentum `p^μ`.
#[derive(Clone, Copy, Debug)]
pub struct VectorWf<F: Real, V: Variance = Contravariant> {
    /// Polarisation / Lorentz components in HELAS convention, at variance `V`.
    pub eps: ComplexVector<F, V>,
    pub momentum: LorentzVector<F, Contravariant>,
}

impl<F: Real> VectorWf<F, Covariant> {
    /// Raise the polarisation index: `ε^μ = g^{μν} ε_ν` (momentum unchanged).
    #[inline(always)]
    pub fn raise(self) -> VectorWf<F, Contravariant> {
        VectorWf {
            eps: self.eps.raise(),
            momentum: self.momentum,
        }
    }
}

impl<F: Real> VectorWf<F, Contravariant> {
    /// Lower the polarisation index: `ε_μ = g_{μν} ε^ν` (momentum unchanged).
    #[inline(always)]
    pub fn lower(self) -> VectorWf<F, Covariant> {
        VectorWf {
            eps: self.eps.lower(),
            momentum: self.momentum,
        }
    }
}

impl<F: Real> VectorWf<F, Contravariant> {
    /// On-shell polarization vector for a spin-1 external particle.
    ///
    /// # Arguments
    /// - `p`: four-momentum `[E, px, py, pz]` (on-shell: `p² = vmass²`)
    /// - `vmass`: particle mass
    /// - `nhel`: helicity `−1` (left), `0` (longitudinal), `+1` (right)
    /// - `nsv`: flow direction `+1` for outgoing, `−1` for incoming
    ///
    /// Returns a `VectorWf` holding the contravariant components $\epsilon^\mu$ and
    /// the flow-signed momentum `nsv * p`. With `nsv = -1` these are
    /// $\epsilon^\mu(p, \lambda)$; with `nsv = +1` they are the conjugate
    /// $\epsilon^\mu(p, \lambda)^\ast$ an outgoing leg contributes.
    ///
    /// # Implementation
    /// Converted from ALOHA `vxxxxx.F` (Fortran77 HELAS).
    /// Handles 5 cases: massive at-rest, massive along-z, massive general,
    /// massless along-z, massless general. The explicit polarisation vector of each,
    /// with its normalisation and completeness relation, is in the
    /// [module documentation](self).
    pub fn vxxxxx(p: LorentzVector<F, Contravariant>, vmass: F, nhel: i32, nsv: i32) -> Self {
        let two = F::one() + F::one();
        let sqh = (F::one() / two).sqrt();
        let hel = F::from(nhel).unwrap();
        let nsvahl = F::from(nsv).unwrap() * hel.abs();

        let p0 = p.e();
        let p1 = p.px();
        let p2 = p.py();
        let p3 = p.pz();

        let pt2 = p1 * p1 + p2 * p2;
        let pp3 = (pt2 + p3 * p3).sqrt();
        let pp = p0.min(pp3);
        let pt = pp.min(pt2.sqrt());

        let eps = if vmass != F::zero() {
            // Massive vector case
            let hel0 = F::one() - hel.abs();

            if pp == F::zero() {
                // At rest: use special case
                [
                    C::zero(),
                    C::new(-hel * sqh, F::zero()),
                    C::new(F::zero(), nsvahl * sqh),
                    C::new(hel0, F::zero()),
                ]
            } else {
                // Moving particle
                let emp = p0 / (vmass * pp);

                let eps0 = C::new(hel0 * pp / vmass, F::zero());
                let eps3 = C::new(hel0 * p3 * emp + hel * pt / pp * sqh, F::zero());

                let (eps1, eps2) = if pt != F::zero() {
                    let pzpt = p3 / (pp * pt) * sqh * hel;
                    let e1_re = hel0 * p1 * emp - p1 * pzpt;
                    let e1_im = -nsvahl * p2 / pt * sqh;
                    let e2_re = hel0 * p2 * emp - p2 * pzpt;
                    let e2_im = nsvahl * p1 / pt * sqh;
                    (C::new(e1_re, e1_im), C::new(e2_re, e2_im))
                } else {
                    let sign_p3 = if p3 >= F::zero() { F::one() } else { -F::one() };
                    (
                        C::new(-hel * sqh, F::zero()),
                        C::new(F::zero(), nsvahl * sign_p3 * sqh),
                    )
                };

                [eps0, eps1, eps2, eps3]
            }
        } else {
            // Massless vector case
            let pp_light = p0;
            let pt_light = (p1 * p1 + p2 * p2).sqrt();

            let eps0 = C::new(F::zero(), F::zero());
            let eps3 = C::new(hel * pt_light / pp_light * sqh, F::zero());

            let (eps1, eps2) = if pt_light != F::zero() {
                let pzpt = p3 / (pp_light * pt_light) * sqh * hel;
                let e1_re = -p1 * pzpt;
                let e1_im = -F::from(nsv).unwrap() * p2 / pt_light * sqh;
                let e2_re = -p2 * pzpt;
                let e2_im = F::from(nsv).unwrap() * p1 / pt_light * sqh;
                (C::new(e1_re, e1_im), C::new(e2_re, e2_im))
            } else {
                let sign_p3 = if p3 >= F::zero() { F::one() } else { -F::one() };
                (
                    C::new(-hel * sqh, F::zero()),
                    C::new(F::zero(), F::from(nsv).unwrap() * sign_p3 * sqh),
                )
            };

            [eps0, eps1, eps2, eps3]
        };

        VectorWf {
            eps: ComplexVector::new(eps),
            momentum: match nsv {
                1 => p,   // outgoing: +p
                -1 => -p, // incoming: -p
                _ => panic!("nsv must be ±1"),
            },
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Scalar wavefunction
// ──────────────────────────────────────────────────────────────────────────────

/// External scalar wavefunction: amplitude = 1, momentum stored for routing.
///
/// Used for Higgs bosons, pions, and other spin-0 particles.
/// The wavefunction value is always `1+0i`; momentum is stored to track
/// the internal leg momentum during off-shell vertex computations.
#[derive(Clone, Copy, Debug)]
pub struct ScalarWf<F: Real> {
    /// Scalar amplitude (always 1+0i for external scalars).
    pub value: C<F>,
    /// Signed momentum: particle → +p, antiparticle → −p
    pub momentum: LorentzVector<F, Contravariant>,
}

impl<F: Real> ScalarWf<F> {
    /// On-shell wavefunction for a spin-0 external particle.
    ///
    /// # Arguments
    /// - `p`: four-momentum `[E, px, py, pz]`
    /// - `nss`: flow direction `+1` for outgoing, `−1` for incoming
    ///
    /// Returns a `ScalarWf` with value = 1+0i and signed momentum.
    ///
    /// # Implementation
    /// Converted from ALOHA `sxxxxx.F` (Fortran77 HELAS).
    /// The scalar amplitude is trivial; this mainly stores momentum for routing.
    pub fn sxxxxx(p: LorentzVector<F, Contravariant>, nss: i32) -> Self {
        ScalarWf {
            value: C::new(F::one(), F::zero()),
            momentum: match nss {
                1 => p,   // outgoing: +p
                -1 => -p, // incoming: -p
                _ => panic!("nss must be ±1"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helas::repr::lorentz::VectorRepr;

    #[test]
    fn test_vxxxxx_massless_along_z_positive_helicity() {
        // Massless photon along +z axis with E=1: p = [1, 0, 0, 1]
        let p = LorentzVector::new(1.0, 0.0, 0.0, 1.0);
        let wf = VectorWf::vxxxxx(p, 0.0, 1, 1);

        // Momentum should be p (nsv=+1 for outgoing)
        assert_eq!(wf.momentum.e(), 1.0);
        assert_eq!(wf.momentum.pz(), 1.0);

        // For massless along z, ε_0 = 0
        assert_eq!(wf.eps.component(0).re, 0.0);
        assert_eq!(wf.eps.component(0).im, 0.0);
    }

    #[test]
    fn test_vxxxxx_massive_at_rest() {
        // Massive vector at rest: p = [m, 0, 0, 0]
        let p = LorentzVector::new(1.0, 0.0, 0.0, 0.0);
        let wf = VectorWf::vxxxxx(p, 1.0, 1, 1);

        // Signed momentum: nsv=1 means no sign change
        assert_eq!(wf.momentum.component(0), 1.0);

        // At rest with nhel=1, hel0=0, so ε_1 = -1/√2
        let sqh = 1.0 / 2.0_f64.sqrt();
        assert!((wf.eps.component(1).re + sqh).abs() < 1e-10);
    }

    #[test]
    fn test_vxxxxx_incoming_vs_outgoing() {
        let p = LorentzVector::new(2.0, 1.0, 0.5, 0.2);
        let wf_out = VectorWf::vxxxxx(p, 0.5, 0, 1); // outgoing (nsv=+1)
        let wf_in = VectorWf::vxxxxx(p, 0.5, 0, -1); // incoming (nsv=-1)

        // Momenta should have opposite signs
        assert_eq!(wf_out.momentum.component(0), 2.0);
        assert_eq!(wf_in.momentum.component(0), -2.0);
    }

    #[test]
    fn test_sxxxxx_scalar() {
        let p = LorentzVector::new(1.0, 0.5, 0.3, 0.2);
        let wf = ScalarWf::sxxxxx(p, 1);

        // Scalar amplitude is always 1+0i
        assert_eq!(wf.value.re, 1.0);
        assert_eq!(wf.value.im, 0.0);

        // Momentum should be +p (nsv=1)
        assert_eq!(wf.momentum.component(0), 1.0);
        assert_eq!(wf.momentum.component(1), 0.5);
    }

    #[test]
    fn test_sxxxxx_incoming() {
        let p = LorentzVector::new(2.0, 0.1, 0.2, 0.3);
        let wf = ScalarWf::sxxxxx(p, -1);

        // Momentum should be -p (nsv=-1)
        assert_eq!(wf.momentum.component(0), -2.0);
        assert_eq!(wf.momentum.component(1), -0.1);
    }

    fn generate_spinor_test_cases(
        onshell_only: bool,
    ) -> impl Iterator<
        Item = (
            (LorentzVector<f64, Contravariant>, f64),
            SpinorHelicity,
            Charge,
        ),
    > {
        let p1 = LorentzVector::new(2.0, 0.5, -0.3, 1.2);
        let p2 = LorentzVector::new((3.5_f64).sqrt(), 0.5, -1.0, 1.5);
        assert!(p2.m2().abs() < 1e-10, "p2 should be massless for this test");
        let offshell_cases = vec![
            (p1, 0.5), // off-shell massive
            (p1, 0.0), // off-shell massless
        ];
        let onshell_cases = vec![
            (p1, p1.m()), // on-shell massive
            (p2, 0.0),    // on-shell massless
        ];
        let lorentz_cases = if onshell_only {
            onshell_cases
        } else {
            offshell_cases.into_iter().chain(onshell_cases).collect()
        };
        itertools::iproduct!(
            lorentz_cases,
            [SpinorHelicity::Up, SpinorHelicity::Down],
            [Charge::Particle, Charge::Antiparticle]
        )
    }

    /// Test bilinear scalar norm for the spinors
    #[test]
    fn test_bilinear_scalar_norm() {
        for ((p, mass), nhel, nsf) in generate_spinor_test_cases(true) {
            let in_wf = InDiracWf::from_momentum(p, mass, nhel, nsf);
            let scalar_bilinear = in_wf.to_outgoing().scalar_bilinear(&in_wf, Chirality::Both);
            // HELAS convention for the scalar bilinear norm is 2 * mass when on-shell
            let expected = nsf.sign() as f64 * 2.0 * mass;
            assert!(
                (scalar_bilinear.re - expected).abs() < 1e-10,
                "Scalar bilinear norm failed for p={:?}, mass={}, nhel={:?}, nsf={:?}: got {}, expected {}",
                p, mass, nhel, nsf, scalar_bilinear.re, expected
            );
            assert!(
                scalar_bilinear.im.abs() < 1e-10,
                "Scalar bilinear should be real for p={:?}, mass={}, nhel={:?}, nsf={:?}: got imaginary part {}",
                p, mass, nhel, nsf, scalar_bilinear.im
            );

            // Opposite helicity should give zero scalar bilinear when on-shell
            let in_wf_oh = InDiracWf::from_momentum(p, mass, nhel.flip(), nsf);
            let scalar_bilinear_orthogonal = in_wf
                .to_outgoing()
                .scalar_bilinear(&in_wf_oh, Chirality::Both);
            assert!(
                scalar_bilinear_orthogonal.norm() < 1e-10,
                "Scalar bilinear should be orthogonal for opposite helicity: got {}",
                scalar_bilinear_orthogonal
            );
        }
    }

    /// Test the to_outgoing and to_incoming conversions between InDiracWf and OutDiracWf.
    #[test]
    fn test_in_out_conversion() {
        for ((p, mass), nhel, nsf) in generate_spinor_test_cases(false) {
            let in_wf = InDiracWf::from_momentum(p, mass, nhel, nsf);
            let out_wf = OutDiracWf::from_momentum(p, mass, nhel, nsf);
            let in_wf_converted = out_wf.to_incoming();
            let out_wf_converted = in_wf.to_outgoing();

            // The converted wavefunction should match the original
            assert_eq!(in_wf, in_wf_converted);
            assert_eq!(out_wf, out_wf_converted);
        }
    }
}
