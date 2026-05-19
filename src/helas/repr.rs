/// Trait alias for the floating-point scalar we use throughout HELAS.
/// Everything we need from `num_traits::Float` plus `Copy + 'static`.
pub trait Real: num_traits::Float + Copy + 'static + std::fmt::Debug {}
impl<F: num_traits::Float + Copy + 'static + std::fmt::Debug> Real for F {}

pub type C<F> = num_complex::Complex<F>;

/// Convenience: real → Complex
#[inline(always)]
pub fn r<F: Real>(x: F) -> C<F> {
    C::new(x, F::zero())
}

/// Convenience: imaginary unit times a real
#[inline(always)]
pub fn ri<F: Real>(x: F) -> C<F> {
    C::new(F::zero(), x)
}

// ──────────────────────────────────────────────────────────────────────────────
// Spinor representation trait
// ──────────────────────────────────────────────────────────────────────────────

/// A spinor wavefunction: 4-component Dirac spinor stored in some basis.
///
/// Implementors store the components and expose the bilinear currents needed
/// for the HELAS routines.  The canonical implementation is `WeylBasis`.
pub trait SpinorRepr<F: Real>: Sized + Copy + 'static {
    /// The concrete type that holds the 4 complex spinor components.
    type Spinor: Copy + std::fmt::Debug;

    /// flowing-IN spinor wavefunction (HELAS `ixxxxx`).
    ///
    /// * `p`   – 4-momentum  [E, px, py, pz]
    /// * `mass` – particle mass (non-negative)
    /// * `nhel` – helicity label (±1; 0 reserved for massive spin-0, unused here)
    /// * `nsf`  – particle/antiparticle sign: +1 = particle, −1 = antiparticle
    fn ixxxxx(p: [F; 4], mass: F, nhel: i32, nsf: i32) -> Self::Spinor;

    /// flowing-OUT spinor wavefunction (HELAS `oxxxxx`).
    fn oxxxxx(p: [F; 4], mass: F, nhel: i32, nsf: i32) -> Self::Spinor;

    /// Left-handed fermion current  J_L^μ = v̄_out γ^μ P_L u_in.
    ///
    /// Returns covariant components with Minkowski metric signs absorbed so
    /// that `Σ_μ C[μ] V[μ]` (plain dot, no extra signs) is the invariant.
    fn left_current(fo: &Self::Spinor, fi: &Self::Spinor) -> [C<F>; 4];

    /// Right-handed fermion current  J_R^μ = v̄_out γ^μ P_R u_in.
    fn right_current(fo: &Self::Spinor, fi: &Self::Spinor) -> [C<F>; 4];
}

// ──────────────────────────────────────────────────────────────────────────────
// Weyl (chiral) basis implementation
// ──────────────────────────────────────────────────────────────────────────────

/// Marker type selecting the Weyl (chiral) basis.
///
/// Layout: components [0,1] = ψ_α (left/undotted),  [2,3] = χ^{α̇} (right/dotted).
#[derive(Clone, Copy, Debug)]
pub struct WeylBasis;

impl<F: Real> SpinorRepr<F> for WeylBasis {
    type Spinor = [C<F>; 4];

    fn ixxxxx(p: [F; 4], mass: F, nhel: i32, nsf: i32) -> [C<F>; 4] {
        weyl_ixxxxx(p, mass, nhel, nsf)
    }

    fn oxxxxx(p: [F; 4], mass: F, nhel: i32, nsf: i32) -> [C<F>; 4] {
        weyl_oxxxxx(p, mass, nhel, nsf)
    }

    /// Left current: uses right-chiral (dotted) indices of *fo* and left-chiral
    /// (undotted) indices of *fi*.  Convention: metric signs absorbed.
    ///
    ///   c0l =  fo[2]*fi[0] + fo[3]*fi[1]
    ///   c1l = -(fo[2]*fi[1] + fo[3]*fi[0])
    ///   c2l =  i*(fo[2]*fi[1] - fo[3]*fi[0])
    ///   c3l = -fo[2]*fi[0] + fo[3]*fi[1]
    fn left_current(fo: &[C<F>; 4], fi: &[C<F>; 4]) -> [C<F>; 4] {
        [
            fo[2] * fi[0] + fo[3] * fi[1],
            -(fo[2] * fi[1] + fo[3] * fi[0]),
            ri(F::one()) * (fo[2] * fi[1] - fo[3] * fi[0]),
            -fo[2] * fi[0] + fo[3] * fi[1],
        ]
    }

    /// Right current: uses left-chiral (undotted) indices of *fo* and
    /// right-chiral (dotted) indices of *fi*.
    ///
    /// Components with metric signs absorbed so that `mink_dot(cr, vc)` gives
    /// the correct iovxxx contraction (matches Fortran iovxxx lines 86-89):
    ///
    ///   c0r =  fo[0]*fi[2] + fo[1]*fi[3]
    ///   c1r =  fo[0]*fi[3] + fo[1]*fi[2]   (NO minus sign)
    ///   c2r = -i*(fo[0]*fi[3] - fo[1]*fi[2])
    ///   c3r =  fo[0]*fi[2] - fo[1]*fi[3]
    fn right_current(fo: &[C<F>; 4], fi: &[C<F>; 4]) -> [C<F>; 4] {
        [
            fo[0] * fi[2] + fo[1] * fi[3],
            fo[0] * fi[3] + fo[1] * fi[2],
            -ri(F::one()) * (fo[0] * fi[3] - fo[1] * fi[2]),
            fo[0] * fi[2] - fo[1] * fi[3],
        ]
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Internal helpers: the actual numerics
// ──────────────────────────────────────────────────────────────────────────────

/// Incoming fermion wavefunction (column spinor).
///
/// Mirrors Fortran `ixxxxx` exactly.  `nsf = +1` for a particle (e.g. e⁻),
/// `nsf = -1` for an antiparticle (e.g. e⁺).  `nhel = ±1` is the helicity.
fn weyl_ixxxxx<F: Real>(p: [F; 4], mass: F, nhel: i32, nsf: i32) -> [C<F>; 4] {
    let two = F::one() + F::one();
    let nh = nhel * nsf;

    let mut fi = [C::new(F::zero(), F::zero()); 4];

    if mass != F::zero() {
        let pp = (p[1] * p[1] + p[2] * p[2] + p[3] * p[3]).sqrt().min(p[0]);

        if pp == F::zero() {
            // ── at rest ───────────────────────────────────────────────────
            // sqm[0] = sqrt(|mass|),  sqm[1] = sqm[0] * sign(mass)
            let sqm0 = mass.abs().sqrt();
            let sqm1 = sqm0 * mass.signum();
            let sqm = [sqm0, sqm1];

            // ip, im used both as 0/1 integer multipliers AND as sqm indices.
            let ip_i = (1 + nh) / 2; // 0 or 1
            let im_i = (1 - nh) / 2; // 1 or 0
            let ip = ip_i as usize;
            let im = im_i as usize;

            fi[0] = r(F::from(ip_i).unwrap() * sqm[ip]);
            fi[1] = r(F::from(im_i * nsf).unwrap() * sqm[ip]);
            fi[2] = r(F::from(ip_i * nsf).unwrap() * sqm[im]);
            fi[3] = r(F::from(im_i).unwrap() * sqm[im]);
        } else {
            // ── massive, moving ───────────────────────────────────────────
            // sf formula: sf[0] = (1+nsf+(1-nsf)*nh)/2, sf[1] = (1+nsf-(1-nsf)*nh)/2
            let sf = [
                F::from(1 + nsf + (1 - nsf) * nh).unwrap() / two,
                F::from(1 + nsf - (1 - nsf) * nh).unwrap() / two,
            ];
            // omega[0] = sqrt(E+|p|),  omega[1] = mass/omega[0]
            let omega0 = (p[0] + pp).sqrt();
            let omega = [omega0, mass / omega0];

            // ip = (1+nh)/2 (0 or 1),  im = (1-nh)/2 (1 or 0)
            let ip = ((1 + nh) / 2) as usize;
            let im = ((1 - nh) / 2) as usize;

            let sfomeg = [r(sf[0] * omega[ip]), r(sf[1] * omega[im])];

            let pp3 = (pp + p[3]).max(F::zero());
            // chi[0] = sqrt(pp3 / (2*pp)),  chi[1] = (nh*p1 + i*p2)/sqrt(2*pp*pp3)
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
            (p[0] + p[3]).max(F::zero()).sqrt() * F::from(nsf).unwrap()
        };
        let chi0 = r(sqp0p3);
        let chi1 = if sqp0p3 == F::zero() {
            r(F::from(-nhel).unwrap() * (two * p[0]).sqrt())
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
/// Mirrors Fortran `oxxxxx` exactly.  Differences from `ixxxxx`:
///   - chi₁ uses −p₂ (conjugate),
///   - sfomeg₀ ↔ sfomeg₁ swapped in the slot assignment,
///   - at-rest uses ip = −((1+nh)/2).
fn weyl_oxxxxx<F: Real>(p: [F; 4], mass: F, nhel: i32, nsf: i32) -> [C<F>; 4] {
    let two = F::one() + F::one();
    let nh = nhel * nsf;

    let mut fo = [C::new(F::zero(), F::zero()); 4];

    if mass != F::zero() {
        let pp = (p[1] * p[1] + p[2] * p[2] + p[3] * p[3]).sqrt().min(p[0]);

        if pp == F::zero() {
            // ── at rest ───────────────────────────────────────────────────
            // oxxxxx: ip = -((1+nh)/2), im = (1-nh)/2
            let sqm0 = mass.abs().sqrt();
            let sqm1 = sqm0 * mass.signum();
            let sqm = [sqm0, sqm1];

            let ip_i = -((1 + nh) / 2); // 0 or -1
            let im_i = (1 - nh) / 2; // 1 or 0
            let neg_ip = (-ip_i) as usize; // sqm index for sqm[-ip]: 0 or 1
            let im = im_i as usize;

            fo[0] = r(F::from(im_i).unwrap() * sqm[im]);
            fo[1] = r(F::from(ip_i * nsf).unwrap() * sqm[im]);
            fo[2] = r(F::from(im_i * nsf).unwrap() * sqm[neg_ip]);
            fo[3] = r(F::from(ip_i).unwrap() * sqm[neg_ip]);
        } else {
            // ── massive, moving ───────────────────────────────────────────
            let sf = [
                F::from(1 + nsf + (1 - nsf) * nh).unwrap() / two,
                F::from(1 + nsf - (1 - nsf) * nh).unwrap() / two,
            ];
            let omega0 = (p[0] + pp).sqrt();
            let omega = [omega0, mass / omega0];

            let ip = ((1 + nh) / 2) as usize;
            let im = ((1 - nh) / 2) as usize;

            // sfomeg same computation as ixxxxx …
            let sfomeg = [r(sf[0] * omega[ip]), r(sf[1] * omega[im])];

            let pp3 = (pp + p[3]).max(F::zero());
            let chi0 = r((pp3 / (two * pp)).sqrt());
            // … but chi₁ uses −p₂ (complex conjugate)
            let chi1 = if pp3 > F::zero() {
                C::new(F::from(nh).unwrap() * p[1], -p[2]) / r((two * pp * pp3).sqrt())
            } else {
                r(F::from(-nh).unwrap())
            };
            let chi = [chi0, chi1];

            // … and sfomeg₀ ↔ sfomeg₁ SWAPPED in assignment vs ixxxxx
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
            (p[0] + p[3]).max(F::zero()).sqrt() * F::from(nsf).unwrap()
        };
        let chi0 = r(sqp0p3);
        // chi₁ uses −p₂ (conjugate) and NHEL (not nh) when sqp0p3 == 0
        let chi1 = if sqp0p3 == F::zero() {
            r(F::from(-nhel).unwrap() * (two * p[0]).sqrt())
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
