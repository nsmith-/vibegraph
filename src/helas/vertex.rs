use crate::helas::repr::{C, Real, SpinorRepr, r};
use crate::helas::wavefn::{DiracWf, VectorWf};

// ──────────────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Minkowski inner product with metric (+,−,−,−), where the *metric signs are
/// already absorbed* into the current components (HELAS convention).
///
/// Formula: `a[0]*b[0] − a[1]*b[1] − a[2]*b[2] − a[3]*b[3]`
#[inline]
fn mink_dot<F: Real>(a: [C<F>; 4], b: [C<F>; 4]) -> C<F> {
    a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3]
}

/// Minkowski inner product of a real momentum with a complex current.
#[inline]
fn mink_dot_q<F: Real>(q: [F; 4], c: [C<F>; 4]) -> C<F> {
    r(q[0]) * c[0] - r(q[1]) * c[1] - r(q[2]) * c[2] - r(q[3]) * c[3]
}

// ──────────────────────────────────────────────────────────────────────────────
// j3xxxx — off-shell W³ (γ + Z combined) current
// ──────────────────────────────────────────────────────────────────────────────

/// Off-shell W³ boson current from two external fermion legs.
///
/// Corresponds to the HELAS routine `j3xxxx`.  Uses Weinberg-angle mixing
/// extracted from the coupling ratio gzf[1]/gaf[1] — this matches the exact
/// Fortran logic where the photon and Z propagators are combined in the W³
/// eigenstate basis.
///
/// # Arguments
/// * `fo`     – flowing-OUT fermion wavefunction
/// * `fi`     – flowing-IN  fermion wavefunction
/// * `gaf`    – photon couplings  `[g_L^γ, g_R^γ]`
/// * `gzf`    – Z couplings       `[g_L^Z, g_R^Z]`
/// * `zmass`  – Z mass
/// * `zwidth` – Z total decay width (Breit-Wigner)
pub fn j3xxxx<F: Real, B: SpinorRepr<F>>(
    fo: &DiracWf<F, B>,
    fi: &DiracWf<F, B>,
    gaf: [F; 2],
    gzf: [F; 2],
    zmass: F,
    zwidth: F,
) -> VectorWf<F> {
    // Off-shell momentum: fo.p − fi.p
    let jmom = std::array::from_fn(|mu| fo.momentum[mu] - fi.momentum[mu]);
    // Propagator momentum (inflow convention)
    let q = [-jmom[0], -jmom[1], -jmom[2], -jmom[3]];
    let q2 = q[0] * q[0] - q[1] * q[1] - q[2] * q[2] - q[3] * q[3];

    let zm2 = zmass * zmass;
    let zmw = zmass * zwidth;

    // ── Weinberg angle from coupling ratio  gzf[1] / gaf[1] ─────────────
    let ratio = gzf[1] / gaf[1];
    let cw = F::one() / (F::one() + ratio * ratio).sqrt();
    let sw = ((F::one() - cw) * (F::one() + cw)).sqrt();

    let ga3l = gaf[0] * sw; // photon left coupling
    let gz3l = gzf[0] * cw; // Z left coupling
    let gn = gaf[1] * sw; // combined right coupling

    // ── Propagators ───────────────────────────────────────────────────────
    let da = F::one() / q2; // real photon: 1/q²
    let dz = C::new(F::one(), F::zero())                 // Z: 1/(q²−mZ²+imZΓZ)
        / C::new(q2 - zm2, zmw);
    let ddif = C::new(-zm2, zmw) * r(da) * dz; // ≈ da for mZ → ∞

    // ── Bilinear currents (metric signs absorbed) ─────────────────────────
    let cl = B::left_current(&fo.spinor, &fi.spinor);
    let cr = B::right_current(&fo.spinor, &fi.spinor);

    // Longitudinal-mode projections divided by complex mZ²
    let cm2 = C::new(zm2, -zmw);
    let csl = mink_dot_q(q, cl) / cm2;
    let csr = mink_dot_q(q, cr) / cm2;

    // ── Output polarisation vector ────────────────────────────────────────
    // eps[μ] = gz3l·dz·(cl[μ] − q[μ]·csl)
    //        + ga3l·da·cl[μ]
    //        + gn·(cr[μ]·ddif + q[μ]·csr·dz)
    let eps: [C<F>; 4] = std::array::from_fn(|mu| {
        let qmu = r(q[mu]);
        r(gz3l) * dz * (cl[mu] - qmu * csl)
            + r(ga3l) * r(da) * cl[mu]
            + r(gn) * (cr[mu] * ddif + qmu * csr * dz)
    });

    VectorWf {
        eps,
        momentum: jmom,
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// jioxxx — off-shell single-boson current
// ──────────────────────────────────────────────────────────────────────────────

/// Off-shell single-boson current from a fermion pair, with independent
/// left/right couplings.
///
/// Mirrors the HELAS `jioxxx` routine.  Uses Feynman gauge for massless bosons
/// and unitary gauge (with the Fabio fixed-width prescription) for massive ones.
///
/// Unlike [`j3xxxx`], this routine handles a single boson species (photon *or*
/// Z) at a time.  To compute the full SM `e⁺e⁻ → μ⁺μ⁻` matrix element, call
/// this once for the photon and once for the Z, then sum the resulting
/// amplitudes before squaring.
///
/// # Arguments
/// * `fo`     – flowing-OUT fermion wavefunction (e.g. e⁺ in the electron current)
/// * `fi`     – flowing-IN  fermion wavefunction (e.g. e⁻)
/// * `gc`     – couplings `[g_L, g_R]` (left/right-handed, real)
/// * `vmass`  – boson mass (0 for photon)
/// * `vwidth` – boson total width (0 for stable)
pub fn jioxxx<F: Real, B: SpinorRepr<F>>(
    fo: &DiracWf<F, B>,
    fi: &DiracWf<F, B>,
    gc: [F; 2],
    vmass: F,
    vwidth: F,
) -> VectorWf<F> {
    // Off-shell momentum: jmom = fo.p − fi.p  (outflow convention)
    let jmom = std::array::from_fn(|mu| fo.momentum[mu] - fi.momentum[mu]);
    let q = jmom;
    let q2 = q[0] * q[0] - q[1] * q[1] - q[2] * q[2] - q[3] * q[3];

    let cl = B::left_current(&fo.spinor, &fi.spinor);
    let cr = B::right_current(&fo.spinor, &fi.spinor);
    let blin: [C<F>; 4] = std::array::from_fn(|mu| r(gc[0]) * cl[mu] + r(gc[1]) * cr[mu]);

    let eps = if vmass == F::zero() {
        // Massless: Feynman gauge — propagator is real 1/q²
        let d = r(F::one() / q2);
        std::array::from_fn(|mu| blin[mu] * d)
    } else {
        // Massive: unitary gauge with Fabio fixed-width complex denominator
        let vm2 = vmass * vmass;
        let vmw = vmass * vwidth;
        let denom = C::new(q2 - vm2, vmw);
        // Longitudinal mode subtraction: divide by m²−imΓ (Fabio prescription)
        let cm2 = C::new(vm2, -vmw);
        let cs = mink_dot_q(q, blin) / cm2;
        let d = C::new(F::one(), F::zero()) / denom;
        std::array::from_fn(|mu| (blin[mu] - cs * r(q[mu])) * d)
    };

    VectorWf {
        eps,
        momentum: jmom,
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// iovxxx — amplitude: fermion–fermion–vector contraction
// ──────────────────────────────────────────────────────────────────────────────

/// Compute the amplitude ⟨fo | γ^μ (gL P_L + gR P_R) | fi⟩ · ε_μ.
///
/// Corresponds to HELAS `iovxxx`.
///
/// # Arguments
/// * `fo`  – flowing-OUT fermion wavefunction
/// * `fi`  – flowing-IN  fermion wavefunction
/// * `v`   – vector (gauge boson) wavefunction
/// * `gc`  – gauge couplings  `[g_L, g_R]`
///
/// # Returns
/// The complex-valued Lorentz-invariant amplitude.
pub fn iovxxx<F: Real, B: SpinorRepr<F>>(
    fo: &DiracWf<F, B>,
    fi: &DiracWf<F, B>,
    v: &VectorWf<F>,
    gc: [C<F>; 2],
) -> C<F> {
    let cl = B::left_current(&fo.spinor, &fi.spinor);
    let cr = B::right_current(&fo.spinor, &fi.spinor);

    // M = gc[0] * (C_L · V) + gc[1] * (C_R · V)
    gc[0] * mink_dot(cl, v.eps) + gc[1] * mink_dot(cr, v.eps)
}
