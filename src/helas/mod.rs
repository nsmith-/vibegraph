pub mod repr;
pub mod vertex;
pub mod wavefn;

pub use repr::{Real, SpinorRepr, WeylBasis, C};
pub use vertex::{iovxxx, j3xxxx};
pub use wavefn::{DiracWf, VectorWf};

#[cfg(test)]
mod tests {
    use super::*;
    use repr::r;
    use itertools::iproduct;

    /// e⁺e⁻ → μ⁺μ⁻ via s-channel photon/Z.
    ///
    /// Kinematics (CoM frame, √s = 2, θ = 90°, all massless):
    ///   e⁻: p = (1, 0, 0,  1)
    ///   e⁺: p = (1, 0, 0, -1)
    ///   μ⁻: p = (1, 1, 0,  0)
    ///   μ⁺: p = (1,-1, 0,  0)
    ///
    /// Coupling choice:
    ///   gaf = [√2, √2],  gzf = [0, √2],  mZ = 1000,  wZ = 0
    ///
    /// This gives Weinberg angle cw = sw = 1/√2, gz3l = 0, ga3l = 1, gn = 1,
    /// so j3xxxx reduces to a pure photon propagator with unit effective coupling.
    ///
    /// The combined photon+Z propagator in the W³ basis is:
    ///
    ///   P^μν(q) = gz3l·dz·(g^μν − q^μq^ν/mZ²)  +  ga3l·da·g^μν
    ///           + gn·(ddif·g^μν + dz·q^μq^ν/mZ²)
    ///
    /// where da = 1/q², dz = 1/(q²−mZ²+imZΓZ), ddif = (−mZ²+imZΓZ)·da·dz.
    /// In the limit mZ → ∞, dz → 0 and ddif → da, so the Z contribution
    /// decouples and P^μν → (gz3l·0 + ga3l·da + gn·da)·g^μν = da·g^μν.
    ///
    /// Expected: Σ|M|² ≈ 4  (analytic: 4·e⁴·(1+cos²θ) = 4 at θ = 90°).
    /// The four non-zero helicity combinations each give |M|² ≈ 1.
    #[test]
    fn test_ee_to_mumu_spin_sum() {
        let s2 = 2.0_f64.sqrt();

        // 4-momenta [E, px, py, pz]
        let p_em = [1.0, 0.0, 0.0,  1.0]; // e⁻
        let p_ep = [1.0, 0.0, 0.0, -1.0]; // e⁺
        let p_mm = [1.0, 1.0, 0.0,  0.0]; // μ⁻
        let p_mp = [1.0,-1.0, 0.0,  0.0]; // μ⁺

        // Couplings that reduce j3xxxx to a pure vector photon
        let gaf = [s2, s2];
        let gzf = [0.0, s2];
        let zmass = 1000.0_f64;
        let zwidth = 0.0_f64;
        let gc = [r(1.0_f64), r(1.0_f64)]; // unit vector coupling in iovxxx

        let mut amp_sq_sum = 0.0;

        for (nhel_em, nhel_ep, nhel_mm, nhel_mp) in iproduct!([-1_i32, 1], [-1_i32, 1], [-1_i32, 1], [-1_i32, 1]) {
            // nsf: +1 for particle, -1 for antiparticle
            let fi_em = DiracWf::<f64, WeylBasis>::ixxxxx(p_em, 0.0, nhel_em,  1);
            let fo_ep = DiracWf::<f64, WeylBasis>::oxxxxx(p_ep, 0.0, nhel_ep, -1);
            let fi_mm = DiracWf::<f64, WeylBasis>::ixxxxx(p_mm, 0.0, nhel_mm,  1);
            let fo_mp = DiracWf::<f64, WeylBasis>::oxxxxx(p_mp, 0.0, nhel_mp, -1);

            // Off-shell photon from the electron current
            let v = j3xxxx(&fo_ep, &fi_em, gaf, gzf, zmass, zwidth);

            // Amplitude: contract muon current with photon
            let amp = iovxxx(&fo_mp, &fi_mm, &v, gc);

            amp_sq_sum += amp.norm_sqr();
        }

        assert!(
            (amp_sq_sum - 4.0).abs() < 1e-4,
            "Expected Σ|M|² ≈ 4.0, got {amp_sq_sum}"
        );
    }

    /// Check the 4 individually non-zero helicity amplitudes.
    #[test]
    fn test_ee_to_mumu_individual_helicities() {
        let s2 = 2.0_f64.sqrt();

        let p_em = [1.0, 0.0, 0.0,  1.0];
        let p_ep = [1.0, 0.0, 0.0, -1.0];
        let p_mm = [1.0, 1.0, 0.0,  0.0];
        let p_mp = [1.0,-1.0, 0.0,  0.0];

        let gaf = [s2, s2];
        let gzf = [0.0, s2];
        let gc = [r(1.0_f64), r(1.0_f64)];

        // The four non-zero combinations: helicity conservation in massless QED
        // requires λ(e⁻) = −λ(e⁺) and λ(μ⁻) = −λ(μ⁺).
        let nonzero = [(-1, 1, -1, 1), (-1, 1, 1, -1), (1, -1, -1, 1), (1, -1, 1, -1)];

        for &(nhel_em, nhel_ep, nhel_mm, nhel_mp) in &nonzero {
            let fi_em = DiracWf::<f64, WeylBasis>::ixxxxx(p_em, 0.0, nhel_em,  1);
            let fo_ep = DiracWf::<f64, WeylBasis>::oxxxxx(p_ep, 0.0, nhel_ep, -1);
            let fi_mm = DiracWf::<f64, WeylBasis>::ixxxxx(p_mm, 0.0, nhel_mm,  1);
            let fo_mp = DiracWf::<f64, WeylBasis>::oxxxxx(p_mp, 0.0, nhel_mp, -1);

            let v = j3xxxx(&fo_ep, &fi_em, gaf, gzf, 1000.0, 0.0);
            let amp = iovxxx(&fo_mp, &fi_mm, &v, gc);
            let m2 = amp.norm_sqr();

            assert!(
                (m2 - 1.0).abs() < 1e-4,
                "Helicity ({nhel_em},{nhel_ep},{nhel_mm},{nhel_mp}): |M|² = {m2}, expected ≈ 1"
            );
        }

        // The other 12 helicity combinations should vanish.
        for (nhel_em, nhel_ep, nhel_mm, nhel_mp) in iproduct!([-1_i32, 1], [-1_i32, 1], [-1_i32, 1], [-1_i32, 1]) {
            let combo = (nhel_em, nhel_ep, nhel_mm, nhel_mp);
            if nonzero.contains(&combo) { continue; }

            let fi_em = DiracWf::<f64, WeylBasis>::ixxxxx(p_em, 0.0, nhel_em,  1);
            let fo_ep = DiracWf::<f64, WeylBasis>::oxxxxx(p_ep, 0.0, nhel_ep, -1);
            let fi_mm = DiracWf::<f64, WeylBasis>::ixxxxx(p_mm, 0.0, nhel_mm,  1);
            let fo_mp = DiracWf::<f64, WeylBasis>::oxxxxx(p_mp, 0.0, nhel_mp, -1);

            let v = j3xxxx(&fo_ep, &fi_em, gaf, gzf, 1000.0, 0.0);
            let amp = iovxxx(&fo_mp, &fi_mm, &v, gc);
            let m2 = amp.norm_sqr();

            assert!(
                m2 < 1e-8,
                "Helicity ({nhel_em},{nhel_ep},{nhel_mm},{nhel_mp}): |M|² = {m2}, expected ≈ 0"
            );
        }
    }
}
