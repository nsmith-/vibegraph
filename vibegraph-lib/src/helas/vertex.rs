use crate::helas::repr::{
    intertwiner::{GammaL, GammaR, GammaV, Intertwiner2Leg},
    lorentz::{Bispinor, Chirality, ComplexVector, SpinorRepr},
    propagator::{DiracPropagator, Propagator, ScalarPropagator},
    r, Real, C,
};
use crate::helas::wavefn::{InDiracWf, OutDiracWf, ScalarWf, VectorWf};

// ──────────────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────────────

/// HELAS Lorentz contraction with Minkowski signature (+,−,−,−).
///
/// The spinor bilinears built by `GammaL/GammaR` already carry HELAS-specific
/// component signs/phases (matching `iovxxx`), but the final contraction is
/// still the Minkowski one used in the Fortran reference routine.
///
/// TODO: roll this into the [`LorentzRepr`] trait
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
pub fn j3xxxx<F: Real>(
    fo: &OutDiracWf<F>,
    fi: &InDiracWf<F>,
    gaf: [F; 2],
    gzf: [F; 2],
    zmass: F,
    zwidth: F,
) -> VectorWf<F> {
    // Off-shell momentum: fo.p − fi.p
    let jmom = fo.momentum - fi.momentum;
    // Propagator momentum (inflow convention)
    let q = -jmom;
    let q2 = q.m2();

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

    // ── Bilinear currents via GammaL / GammaR intertwiners ────────────────
    let cl = GammaL::apply(&(fo.spinor, fi.spinor));
    let cr = GammaR::apply(&(fo.spinor, fi.spinor));

    // Longitudinal-mode projections divided by complex mZ²
    let cm2 = C::new(zm2, -zmw);
    let csl = mink_dot_q(q.0, cl.0) / cm2;
    let csr = mink_dot_q(q.0, cr.0) / cm2;

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
        eps: ComplexVector { 0: eps },
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
pub fn jioxxx<F: Real>(
    fo: &OutDiracWf<F>,
    fi: &InDiracWf<F>,
    gc: [F; 2],
    vmass: F,
    vwidth: F,
) -> VectorWf<F> {
    // Off-shell momentum: jmom = fo.p − fi.p  (outflow convention)
    let jmom = fo.momentum - fi.momentum;
    let q = jmom;
    let q2 = q.m2();

    // Bilinear currents via GammaL / GammaR intertwiners
    let cl = fo.vector_bilinear(fi, Chirality::Left);
    let cr = fo.vector_bilinear(fi, Chirality::Right);
    let blin = cl * gc[0] + cr * gc[1]; // linear combination of left and right currents

    let eps = if vmass == F::zero() {
        // Massless: Feynman gauge — propagator is 1/q²
        blin / q2
    } else {
        // Massive: unitary gauge with Fabio fixed-width complex denominator
        let vm2 = vmass * vmass;
        let vmw = vmass * vwidth;
        let denom = C::new(q2 - vm2, vmw);
        // Longitudinal mode subtraction: divide by m²−imΓ (Fabio prescription)
        let cs = blin.mink_dot_lorentz(&q) / C::new(vm2, -vmw);
        (blin - ComplexVector::from(q) * cs) / denom
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
pub fn iovxxx<F: Real>(fo: &OutDiracWf<F>, fi: &InDiracWf<F>, v: &VectorWf<F>, gc: [F; 2]) -> C<F> {
    let cl = fo.vector_bilinear(fi, Chirality::Left);
    let cr = fo.vector_bilinear(fi, Chirality::Right);

    // M = gc[0] * (C_L · V) + gc[1] * (C_R · V)
    r(gc[0]) * cl.mink_dot(&v.eps) + r(gc[1]) * cr.mink_dot(&v.eps)
}

// ──────────────────────────────────────────────────────────────────────────────
// Phase 2: Off-shell vertex routines
// ──────────────────────────────────────────────────────────────────────────────

/// Off-shell fermion from incoming fermion + vector boson.
///
/// Corresponds to HELAS `fioxxx` (FFV1_2 in ALOHA).
/// Computes `Ψ = (q̸ + m) ψ / (q² − m² + imΓ)` where `q = fi.p + V.p`.
///
/// # Arguments
/// * `fi`    – flowing-IN  fermion wavefunction
/// * `v`     – vector (gauge boson) wavefunction
/// * `g`     – coupling constant (complex, includes vertex factor)
/// * `mass`  – off-shell fermion mass (usually 0 for light quarks)
/// * `width` – fermion width (usually 0 for stable)
///
/// # Returns
/// Off-shell outgoing fermion current as `OutDiracWf<F>`.
pub fn fioxxx<F: Real>(
    fi: &InDiracWf<F>,
    v: &VectorWf<F>,
    g: C<F>,
    mass: F,
    width: F,
) -> OutDiracWf<F> {
    // Accumulated momentum: q = fi.p + V.p (both inflow)
    let q = fi.momentum + v.momentum;

    // Apply the vertex coupling × GammaV to the incoming fermion
    // GammaV computes ε̸ ψ where ε is the vector polarization
    let gv_psi = GammaV::apply(&(v.eps, fi.spinor));

    // Create a DiracPropagator and propagate
    let prop = DiracPropagator { mass, width };
    let prop_psi = prop.propagate(q.0, gv_psi.0);

    // Scale by coupling: multiply by g
    let scaled_psi = [
        g * prop_psi[0],
        g * prop_psi[1],
        g * prop_psi[2],
        g * prop_psi[3],
    ];

    // Output momentum: off-shell fermion carries momentum q (outflow convention)
    OutDiracWf::from_spinor(Bispinor(scaled_psi), q)
}

/// Off-shell fermion from outgoing fermion + vector boson.
///
/// Corresponds to HELAS `foxxx` (FFV1_1 in ALOHA).
/// Similar to `fioxxx` but with outgoing fermion input.
/// Computes `Ψ = (q̸ + m) ψ / (q² − m² + imΓ)` where `q = fo.p + V.p`.
///
/// # Arguments
/// * `fo`    – flowing-OUT fermion wavefunction (acts as input here)
/// * `v`     – vector (gauge boson) wavefunction
/// * `g`     – coupling constant (complex, includes vertex factor)
/// * `mass`  – off-shell fermion mass
/// * `width` – fermion width
///
/// # Returns
/// Off-shell incoming fermion current as `InDiracWf<F>`.
pub fn foxxx<F: Real>(
    fo: &OutDiracWf<F>,
    v: &VectorWf<F>,
    g: C<F>,
    mass: F,
    width: F,
) -> InDiracWf<F> {
    // Accumulated momentum: q = fo.p + V.p (outflow convention)
    let q = fo.momentum + v.momentum;

    // Apply GammaV: ε̸ ψ
    let gv_psi = GammaV::apply(&(v.eps, fo.spinor));

    // Propagate
    let prop = DiracPropagator { mass, width };
    let prop_psi = prop.propagate(q.0, gv_psi.0);

    // Scale by coupling
    let scaled_psi = [
        g * prop_psi[0],
        g * prop_psi[1],
        g * prop_psi[2],
        g * prop_psi[3],
    ];

    // Output momentum: off-shell fermion carries accumulated momentum (inflow convention)
    InDiracWf::from_spinor(Bispinor(scaled_psi), q)
}

/// Off-shell vector from two vector bosons.
///
/// Corresponds to HELAS `jvvxxx` (VVV1P0_1 in ALOHA for massless case).
/// Computes the three-gauge-boson coupling.
/// `V_1^μ = g · P(q²) · [v2·v3 · (q_out^μ - q_in^μ) + v2^μ·(q_in·v3) - v3^μ·(q_in·v2)]`
/// where `q = v2.p + v3.p`.
///
/// # Arguments
/// * `v2`    – first input vector
/// * `v3`    – second input vector
/// * `g`     – coupling constant
/// * `mass`  – off-shell vector mass (0 for massless)
/// * `width` – off-shell vector width
///
/// # Returns
/// Off-shell vector boson as `VectorWf<F>`.
pub fn jvvxxx<F: Real>(
    v2: &VectorWf<F>,
    v3: &VectorWf<F>,
    g: C<F>,
    mass: F,
    width: F,
) -> VectorWf<F> {
    // Accumulated momentum: q = v2.p + v3.p (outflow for incoming vectors)
    let q = v2.momentum + v3.momentum;
    let q2 = q[0] * q[0] - q[1] * q[1] - q[2] * q[2] - q[3] * q[3];

    let m2 = mass * mass;
    let mw = mass * width;

    // Complex propagator denominator: q² − m² + imΓ
    let denom = C::new(q2 - m2, mw);

    // Extract polarization vectors
    let v2_eps = &v2.eps.0;
    let v3_eps = &v3.eps.0;

    // Compute contraction terms
    // TMP1 = v3·q (Minkowski)
    let tmp1 = mink_dot_q(q.0, [v3_eps[0], v3_eps[1], v3_eps[2], v3_eps[3]]);
    // TMP2 = v3·v2.p (Minkowski contraction)
    let tmp2 = mink_dot_q(v2.momentum.0, [v3_eps[0], v3_eps[1], v3_eps[2], v3_eps[3]]);
    // TMP3 = v2·q (Minkowski)
    let tmp3 = mink_dot_q(q.0, [v2_eps[0], v2_eps[1], v2_eps[2], v2_eps[3]]);
    // TMP4 = v2·v3.p (Minkowski)
    let tmp4 = mink_dot_q(v3.momentum.0, [v2_eps[0], v2_eps[1], v2_eps[2], v2_eps[3]]);
    // TMP5 = v2·v3 (Minkowski)
    let tmp5 = mink_dot(*v2_eps, *v3_eps);

    let scale = g / denom;

    // Build the off-shell vector
    // Formula from VVV1P0_1.f (ALOHA):
    // V1^μ = DENOM * [TMP5·(q2_μ - q3_μ) + v2^μ·(TMP1 - TMP2) + v3^μ·(TMP3 - TMP4)]
    let eps: [C<F>; 4] = std::array::from_fn(|mu| {
        let v2_mu_eps = v2_eps[mu];
        let v3_mu_eps = v3_eps[mu];
        // Note: this follows the ALOHA formula closely
        scale
            * (tmp5 * (r(v3.momentum[mu]) - r(v2.momentum[mu]))
                + v2_mu_eps * (tmp1 - tmp2)
                + v3_mu_eps * (tmp3 - tmp4))
    });

    VectorWf {
        eps: ComplexVector(eps),
        momentum: q,
    }
}

/// Off-shell scalar from two fermions.
///
/// Corresponds to HELAS `jsixxx`.
/// Computes the scalar current from a fermion pair.
///
/// # Arguments
/// * ` flowing-OUT fermionfo`    
/// * ` flowing-IN  fermionfi`    
/// * ` coupling constant (real or complex)g`     
/// * ` off-shell scalar massmass`  
/// * ` off-shell scalar widthwidth`
///
/// # Returns
/// Off-shell scalar current as `ScalarWf<F>`.
pub fn jsixxx<F: Real>(
    fo: &OutDiracWf<F>,
    fi: &InDiracWf<F>,
    g: C<F>,
    mass: F,
    width: F,
) -> ScalarWf<F> {
    // Accumulated momentum: q = fi.p + fo.p
    let q = fi.momentum + fo.momentum;

    // Scalar current: sum of left and right bilinears (identity structure)
    let scalar_value = Bispinor::scalar_bilinear(&fo.spinor, &fi.spinor, Chirality::Both);

    // Apply scalar propagator
    let prop = ScalarPropagator { mass, width };
    let prop_value = prop.propagate(q.0, scalar_value);

    // Scale by coupling
    let final_value = g * prop_value;

    ScalarWf {
        value: final_value,
        momentum: q,
    }
}

/// Amplitude: two fermions + scalar.
///
/// Corresponds to HELAS `iosxxx`.
/// Direct contraction of two fermions with a scalar:
/// Amplitude =        [g_(((((((fi_fffffffo_left) + g_(((((((fi_fffffffo_right)]rightRleftLS
///
/// # Arguments
/// * ` flowing-OUT fermionfo`  
/// * ` flowing-IN  fermionfi`  
/// * ` scalar wavefunctions`   
/// * ` coupling constants `[g_L, g_R]`gc`  
///
/// # Returns
/// Complex amplitude.
pub fn iosxxx<F: Real>(
    fo: &OutDiracWf<F>,
    fi: &InDiracWf<F>,
    s: &ScalarWf<F>,
    gc: [C<F>; 2],
) -> C<F> {
    // Compute left and right chiral bilinears using the new trait methods
    let left_contr = Bispinor::scalar_bilinear(&fo.spinor, &fi.spinor, Chirality::Left);
    let right_contr = Bispinor::scalar_bilinear(&fo.spinor, &fi.spinor, Chirality::Right);

    // Combine with couplings and scalar value
    s.value * (gc[0] * left_contr + gc[1] * right_contr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helas::repr::lorentz::{Charge, LorentzVector, SpinorHelicity};
    use num_complex::Complex64 as C64;

    /// Helper to create a simple LorentzVector.
    fn lorentz_vec(e: f64, px: f64, py: f64, pz: f64) -> LorentzVector<f64> {
        LorentzVector([e, px, py, pz])
    }

    #[test]
    fn test_fioxxx_basic() {
        // Create test wavefunctions
        let fi_mom = lorentz_vec(100.0, 0.0, 0.0, 100.0);
        let fi = InDiracWf::new(fi_mom, 0.0, SpinorHelicity::Up, Charge::Particle);

        let v_mom = lorentz_vec(10.0, 5.0, 0.0, 8.66);
        let v_eps = [
            C64::new(1.0, 0.0),
            C64::new(0.5, 0.2),
            C64::new(0.3, 0.1),
            C64::new(0.4, 0.0),
        ];
        let v = VectorWf {
            eps: ComplexVector(v_eps),
            momentum: v_mom,
        };

        // Apply fioxxx
        let coupling = C64::new(0.3, 0.0);
        let result = fioxxx(&fi, &v, coupling, 0.0, 0.0);

        // Basic sanity checks
        assert!(
            result.spinor.0.iter().any(|x| x.norm() > 0.0),
            "Result spinor should be non-zero"
        );

        // Check that output momentum is fi + v
        let expected_momentum = fi_mom + v_mom;
        for i in 0..4 {
            assert!(
                (result.momentum[i] - expected_momentum[i]).abs() < 1e-10,
                "Momentum mismatch at component {}: {} vs {}",
                i,
                result.momentum[i],
                expected_momentum[i]
            );
        }
    }

    #[test]
    fn test_foxxx_basic() {
        // Create test wavefunctions
        let fo_mom = lorentz_vec(100.0, 0.0, 0.0, 100.0);
        let fo = OutDiracWf::new(fo_mom, 0.0, SpinorHelicity::Up, Charge::Particle);

        let v_mom = lorentz_vec(10.0, 5.0, 0.0, 8.66);
        let v_eps = [
            C64::new(1.0, 0.0),
            C64::new(0.5, 0.2),
            C64::new(0.3, 0.1),
            C64::new(0.4, 0.0),
        ];
        let v = VectorWf {
            eps: ComplexVector(v_eps),
            momentum: v_mom,
        };

        // Apply foxxx
        let coupling = C64::new(0.3, 0.0);
        let result = foxxx(&fo, &v, coupling, 0.0, 0.0);

        // Basic sanity checks
        assert!(
            result.spinor.0.iter().any(|x| x.norm() > 0.0),
            "Result spinor should be non-zero"
        );

        // Check that output momentum is fo + v
        let expected_momentum = fo_mom + v_mom;
        for i in 0..4 {
            assert!(
                (result.momentum[i] - expected_momentum[i]).abs() < 1e-10,
                "Momentum mismatch at component {}",
                i
            );
        }
    }

    #[test]
    fn test_jvvxxx_basic() {
        // Create two vector wavefunctions
        let v2_mom = lorentz_vec(50.0, 30.0, 0.0, 40.0);
        let v2_eps = [
            C64::new(0.8, 0.0),
            C64::new(0.3, 0.1),
            C64::new(0.2, 0.0),
            C64::new(0.4, 0.0),
        ];
        let v2 = VectorWf {
            eps: ComplexVector(v2_eps),
            momentum: v2_mom,
        };

        let v3_mom = lorentz_vec(30.0, -20.0, 0.0, 22.4);
        let v3_eps = [
            C64::new(0.6, 0.0),
            C64::new(-0.2, 0.1),
            C64::new(0.1, 0.0),
            C64::new(0.2, 0.0),
        ];
        let v3 = VectorWf {
            eps: ComplexVector(v3_eps),
            momentum: v3_mom,
        };

        // Apply jvvxxx
        let coupling = C64::new(0.1, 0.0);
        let result = jvvxxx(&v2, &v3, coupling, 0.0, 0.0);

        // Basic sanity checks
        assert!(
            result.eps.0.iter().any(|x| x.norm() > 0.0),
            "Result polarization should be non-zero"
        );

        // Check that output momentum is v2 + v3
        let expected_momentum = v2_mom + v3_mom;
        for i in 0..4 {
            assert!(
                (result.momentum[i] - expected_momentum[i]).abs() < 1e-10,
                "Momentum mismatch at component {}",
                i
            );
        }
    }

    #[test]
    fn test_jsixxx_basic() {
        // Create fermion wavefunctions
        let fi_mom = lorentz_vec(50.0, 0.0, 0.0, 50.0);
        let fi = InDiracWf::new(fi_mom, 0.0, SpinorHelicity::Up, Charge::Particle);

        let fo_mom = lorentz_vec(30.0, 20.0, 0.0, 22.4);
        let fo = OutDiracWf::new(fo_mom, 0.0, SpinorHelicity::Up, Charge::Particle);

        // Apply jsixxx
        let coupling = C64::new(0.1, 0.0);
        let result = jsixxx(&fo, &fi, coupling, 0.0, 0.0);

        // Basic sanity check
        assert!(
            result.value.norm() > 0.0,
            "Result scalar should be non-zero"
        );

        // Check that output momentum is fi + fo
        let expected_momentum = fi_mom + fo_mom;
        for i in 0..4 {
            assert!(
                (result.momentum[i] - expected_momentum[i]).abs() < 1e-10,
                "Momentum mismatch at component {}",
                i
            );
        }
    }

    #[test]
    fn test_iosxxx_basic() {
        // Create fermion and scalar wavefunctions
        let fi_mom = lorentz_vec(50.0, 0.0, 0.0, 50.0);
        let fi = InDiracWf::new(fi_mom, 0.0, SpinorHelicity::Up, Charge::Particle);

        let fo_mom = lorentz_vec(30.0, 20.0, 0.0, 22.4);
        let fo = OutDiracWf::new(fo_mom, 0.0, SpinorHelicity::Up, Charge::Particle);

        let s_mom = lorentz_vec(20.0, -20.0, 0.0, 0.0);
        let s = ScalarWf {
            value: C64::new(1.0, 0.0),
            momentum: s_mom,
        };

        // Apply iosxxx
        let coupling = [C64::new(0.1, 0.0), C64::new(0.1, 0.0)];
        let result = iosxxx(&fo, &fi, &s, coupling);

        // Basic sanity check: the result should be non-zero
        assert!(result.norm() > 0.0, "Result amplitude should be non-zero");
    }
}
