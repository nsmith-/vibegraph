//! RAMBO phase-space map: uniforms in → on-shell momenta + weight out.
//!
//! [`rambo`] is the deterministic map at the core of flat phase-space sampling.
//! Given `4n` uniforms it builds `n` on-shell four-momenta with the requested
//! masses, summing to `(√ŝ, 0, 0, 0)` in the CM frame, and returns the
//! Kleiss–Stirling–Ellis weight so that a flat Monte-Carlo average of
//! `weight · f` estimates `∫ dR_n f` over the invariant phase-space volume
//! `R_n = ∫ Πᵢ d³pᵢ/(2Eᵢ) · δ⁴(P − Σpᵢ)` (the `(2π)` factors of the full
//! `dΦ_n` measure live in the cross-section prefactor, not here).
//!
//! Construction (Kleiss–Stirling–van der Bij 1986):
//!
//! 1. `n` isotropic massless vectors `qᵢ` with `q⁰ ∼ Γ(2)` are drawn from the
//!    uniforms — four per momentum: `cosθ`, `φ`, and two for the energy
//!    `q⁰ = −ln(r₃r₄)`.
//! 2. One boost + scale sends the ensemble onto total momentum `(√ŝ, 0, 0, 0)`,
//!    giving massless momenta `pᵢ` with weight `R_n`.
//! 3. If any mass is non-zero, a single Newton solve for the rescale factor `ξ`
//!    in `Σᵢ √(mᵢ² + ξ²|p⃗ᵢ|²) = √ŝ` maps `pᵢ → kᵢ = (√(mᵢ²+ξ²|p⃗ᵢ|²), ξ p⃗ᵢ)`,
//!    and the weight picks up the massive rescale Jacobian.
//!
//! Separating the uniforms from their source keeps the RNG (`super::rng`)
//! swappable and makes the map replayable against an external reference: feed
//! the same uniforms and the momenta must match.

use rand::Rng;

use crate::helas::repr::lorentz::LorentzVector;
use crate::helas::repr::Real;

/// One flat phase-space point: `n` on-shell momenta summing to `(√ŝ, 0, 0, 0)`,
/// the phase-space `weight`, and the massive rescale factor `xi` (`1` when all
/// final-state masses vanish).
#[derive(Clone, Debug)]
pub struct RamboPoint<F: Real> {
    pub momenta: Vec<LorentzVector<F>>,
    pub weight: F,
    pub xi: F,
}

/// Flat RAMBO map over the scalar field `F`.
///
/// Consumes exactly `4 · masses.len()` uniforms from `u` and returns `n` on-shell
/// four-momenta of the requested `masses` summing to `(√ŝ, 0, 0, 0)`, plus the
/// KSE phase-space weight. `masses` all zero takes the massless fast path (no
/// Newton solve, `xi = 1`).
///
/// Panics if `masses.len() < 2`, if `u.len() != 4·masses.len()`, or if `√ŝ` is
/// below the mass threshold `Σ mᵢ`.
pub fn rambo<F: Real>(sqrt_s: F, masses: &[F], u: &[F]) -> RamboPoint<F> {
    let n = masses.len();
    assert!(n >= 2, "RAMBO needs at least two final-state momenta");
    assert_eq!(u.len(), 4 * n, "RAMBO consumes exactly 4n uniforms");

    let massless = massless_momenta(sqrt_s, u);
    let volume = massless_volume(sqrt_s, n);

    if masses.iter().all(|&m| m == F::zero()) {
        return RamboPoint {
            momenta: massless,
            weight: volume,
            xi: F::one(),
        };
    }

    let mass_sum = masses.iter().fold(F::zero(), |acc, &m| acc + m);
    assert!(
        mass_sum < sqrt_s,
        "RAMBO mass threshold exceeds available energy"
    );

    // Solve Σᵢ √(mᵢ² + ξ²|p⃗ᵢ|²) = √ŝ for the momentum rescale ξ. The left-hand
    // side is monotone in ξ, so Newton from the massless-limit guess converges.
    let p2: Vec<F> = massless.iter().map(|p| p.p3_squared()).collect();
    let m2: Vec<F> = masses.iter().map(|&m| m * m).collect();

    let ratio = mass_sum / sqrt_s;
    let floor = F::from(1e-12).unwrap();
    let mut xi = (F::one() - ratio * ratio).max(floor).sqrt();
    let tol = F::from(1e-13).unwrap() * sqrt_s;
    for _ in 0..100 {
        let mut f = -sqrt_s;
        let mut df_denom = F::zero();
        for (&p2i, &m2i) in p2.iter().zip(&m2) {
            let e = (m2i + xi * xi * p2i).sqrt();
            f = f + e;
            df_denom = df_denom + p2i / e;
        }
        if f.abs() < tol {
            break;
        }
        xi = xi - f / (xi * df_denom);
    }

    let momenta: Vec<LorentzVector<F>> = massless
        .iter()
        .zip(&m2)
        .zip(&p2)
        .map(|((p, &m2i), &p2i)| {
            let e = (m2i + xi * xi * p2i).sqrt();
            LorentzVector::new(e, xi * p.px(), xi * p.py(), xi * p.pz())
        })
        .collect();

    let weight = volume * massive_jacobian(sqrt_s, &momenta);
    RamboPoint {
        momenta,
        weight,
        xi,
    }
}

/// Massless RAMBO: `n` isotropic massless four-momenta with total momentum
/// `(√s, 0, 0, 0)`, drawing its `4n` uniforms from `rng`.
///
/// The phase-space weight is uniform, so the points serve as unbiased kinematics
/// for evaluator tests and benchmarks; pair with beam momenta `(√s/2, 0, 0, ±√s/2)`
/// for a full 2 → n external set. This is the RNG-driven convenience form of the
/// massless case of [`rambo`].
pub fn rambo_massless(sqrt_s: f64, n: usize, rng: &mut impl Rng) -> Vec<LorentzVector<f64>> {
    assert!(n >= 2, "RAMBO needs at least two final-state momenta");
    let u: Vec<f64> = (0..4 * n).map(|_| rng.random::<f64>()).collect();
    massless_momenta(sqrt_s, &u)
}

/// Build `n = u.len()/4` massless momenta summing to `(√ŝ, 0, 0, 0)` from `4n`
/// uniforms, via the isotropic-`q` + single-boost RAMBO construction.
fn massless_momenta<F: Real>(sqrt_s: F, u: &[F]) -> Vec<LorentzVector<F>> {
    let n = u.len() / 4;
    let two = F::one() + F::one();
    // Isotropic null vectors q_i with q⁰ ~ Gamma(2): q⁰ = −ln(r₃·r₄).
    let q: Vec<[F; 4]> = (0..n)
        .map(|i| {
            let c = two * u[4 * i] - F::one();
            let phi = two * F::PI() * u[4 * i + 1];
            let e = -(u[4 * i + 2] * u[4 * i + 3]).ln();
            let st = (F::one() - c * c).sqrt();
            [e, e * st * phi.cos(), e * st * phi.sin(), e * c]
        })
        .collect();
    // Boost + scale the ensemble into the CM frame of total energy √ŝ.
    let tot = q.iter().fold([F::zero(); 4], |acc, qi| {
        [
            acc[0] + qi[0],
            acc[1] + qi[1],
            acc[2] + qi[2],
            acc[3] + qi[3],
        ]
    });
    let m = (tot[0] * tot[0] - tot[1] * tot[1] - tot[2] * tot[2] - tot[3] * tot[3]).sqrt();
    let b = [-tot[1] / m, -tot[2] / m, -tot[3] / m];
    let gamma = tot[0] / m;
    let a = F::one() / (F::one() + gamma);
    let x = sqrt_s / m;
    q.into_iter()
        .map(|qi| {
            let bq = b[0] * qi[1] + b[1] * qi[2] + b[2] * qi[3];
            let e = x * (gamma * qi[0] + bq);
            let f = x * (qi[0] + a * bq);
            LorentzVector::new(
                e,
                x * qi[1] + f * b[0],
                x * qi[2] + f * b[1],
                x * qi[3] + f * b[2],
            )
        })
        .collect()
}

/// Invariant massless phase-space volume
/// `R_n = (π/2)^{n-1} · ŝ^{n-2} / ((n-1)!·(n-2)!)`.
fn massless_volume<F: Real>(sqrt_s: F, n: usize) -> F {
    let s = sqrt_s * sqrt_s;
    let half_pi = F::PI() / (F::one() + F::one());
    let numer = half_pi.powi(n as i32 - 1) * s.powi(n as i32 - 2);
    numer / (factorial::<F>(n - 1) * factorial::<F>(n - 2))
}

/// Massive rescale Jacobian relating the massive weight to `R_n`
/// (Kleiss–Stirling–Ellis 1986):
/// `(Σ|k⃗ᵢ|/√ŝ)^{2n-3} · (Σ|k⃗ᵢ|²/Eᵢ)⁻¹ · √ŝ · Πᵢ(|k⃗ᵢ|/Eᵢ)`, evaluated on the
/// final (rescaled) momenta `k`. Reduces to `1` in the massless limit.
fn massive_jacobian<F: Real>(sqrt_s: F, k: &[LorentzVector<F>]) -> F {
    let n = k.len();
    let mut sum_p = F::zero();
    let mut sum_p2_over_e = F::zero();
    let mut prod_p_over_e = F::one();
    for ki in k {
        let p = ki.p3();
        let e = ki.e();
        sum_p = sum_p + p;
        sum_p2_over_e = sum_p2_over_e + p * p / e;
        prod_p_over_e = prod_p_over_e * (p / e);
    }
    (sum_p / sqrt_s).powi(2 * n as i32 - 3) / sum_p2_over_e * sqrt_s * prod_p_over_e
}

fn factorial<F: Real>(k: usize) -> F {
    (1..=k).fold(F::one(), |acc, j| acc * F::from(j).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phasespace::rng::SubStream;

    fn total_momentum<F: Real>(momenta: &[LorentzVector<F>]) -> [F; 4] {
        momenta.iter().fold([F::zero(); 4], |acc, p| {
            [
                acc[0] + p.e(),
                acc[1] + p.px(),
                acc[2] + p.py(),
                acc[3] + p.pz(),
            ]
        })
    }

    #[test]
    fn massless_volume_two_body() {
        // R_2 = π/2.
        let v: f64 = massless_volume(100.0, 2);
        assert!((v - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
    }

    #[test]
    fn massless_fast_path_has_unit_xi_and_r_n_weight() {
        let mut s = SubStream::from_stream(1, 0);
        let sqrt_s = 500.0;
        let u = s.uniforms::<f64>(24);
        let pt = rambo(sqrt_s, &[0.0; 6], &u);
        assert_eq!(pt.xi, 1.0);
        assert!((pt.weight - massless_volume::<f64>(sqrt_s, 6)).abs() < 1e-6);
    }

    #[test]
    fn conservation_and_on_shell_fuzz() {
        let mut s = SubStream::from_stream(0xF0F0, 9);
        let cases: &[(usize, &[f64], f64)] = &[
            (2, &[0.0, 0.0], 100.0),
            (3, &[0.0, 0.0, 0.0], 250.0),
            (2, &[80.4, 80.4], 400.0),
            (3, &[10.0, 20.0, 5.0], 300.0),
            (4, &[0.0, 5.0, 0.0, 30.0], 500.0),
            (6, &[0.0; 6], 500.0),
            (5, &[1.0, 2.0, 3.0, 4.0, 5.0], 60.0),
            // Threshold-adjacent: mass sum just under √ŝ.
            (3, &[30.0, 30.0, 30.0], 91.0),
            (2, &[45.0, 45.0], 90.5),
        ];
        for &(n, masses, sqrt_s) in cases {
            assert_eq!(masses.len(), n);
            for _ in 0..200 {
                let u = s.uniforms::<f64>(4 * n);
                let pt = rambo(sqrt_s, masses, &u);
                assert_eq!(pt.momenta.len(), n);
                let tot = total_momentum(&pt.momenta);
                assert!(
                    (tot[0] - sqrt_s).abs() < 1e-8 * sqrt_s,
                    "energy not conserved: {} vs {sqrt_s}",
                    tot[0]
                );
                for c in &tot[1..] {
                    assert!(c.abs() < 1e-8 * sqrt_s, "3-momentum not conserved: {c}");
                }
                for (p, &mass) in pt.momenta.iter().zip(masses) {
                    let want = mass * mass;
                    let scale = sqrt_s * sqrt_s;
                    assert!(
                        (p.m2() - want).abs() < 1e-7 * scale + 1e-7,
                        "off shell: m² = {} want {want}",
                        p.m2()
                    );
                    assert!(p.e() > 0.0 && p.e().is_finite());
                }
                assert!(pt.weight > 0.0 && pt.weight.is_finite());
            }
        }
    }

    #[test]
    fn f32_and_f64_agree_within_precision() {
        // The map is field-generic; f32 tracks f64 to single precision.
        let mut s = SubStream::from_stream(7, 2);
        let u64s = s.uniforms::<f64>(12);
        let u32s: Vec<f32> = u64s.iter().map(|&x| x as f32).collect();
        let masses64 = [10.0f64, 20.0, 5.0];
        let masses32 = [10.0f32, 20.0, 5.0];
        let a = rambo(300.0f64, &masses64, &u64s);
        let b = rambo(300.0f32, &masses32, &u32s);
        for (pa, pb) in a.momenta.iter().zip(&b.momenta) {
            assert!((pa.e() as f32 - pb.e()).abs() < 1e-2 * 300.0);
        }
    }
}
