//! Classic VEGAS adaptive Monte Carlo integrator (Lepage 1978).
//!
//! Integrates a function `f: [0,1]^N → ℝ` by iteratively reshaping a
//! piecewise-uniform importance-sampling grid to concentrate evaluations
//! where `|f|` is large.
//!
//! # Algorithm summary
//!
//! The integration domain `[0,1]^N` is partitioned into `nbins` equal-probability
//! bins per dimension.  Each iteration:
//!
//! 1. Draw `neval` random samples; accumulate the integral estimate and
//!    per-bin f² weights.
//! 2. Refine the grid: rescale bin boundaries so that each bin captures an
//!    equal share of `∫ |f| dx` (Lepage's importance-function update with
//!    α-damping to suppress noise).
//!
//! Multiple iterations are combined with `1/σ²` weighting; the χ²/dof
//! across iterations diagnoses convergence.
//!
//! # Quick start
//!
//! ```rust
//! use vibegraph::vegas::Vegas;
//! use rand::SeedableRng;
//!
//! let mut vegas = Vegas::new(1, 50, 1.5);
//! let mut rng = rand::rngs::StdRng::seed_from_u64(0);
//! // Integrate f(u) = 3u² over [0,1] — exact answer is 1.
//! let result = vegas.integrate(|u| 3.0 * u[0] * u[0], 10_000, 5, &mut rng);
//! assert!((result.integral - 1.0).abs() < 0.01);
//! ```

use rand::Rng;

/// VEGAS adaptive importance-sampling grid and integrator.
///
/// The grid `xi[d][k]` stores the `k`-th bin boundary in dimension `d`,
/// with `xi[d][0] = 0` and `xi[d][nbins] = 1`.
pub struct Vegas {
    ndim: usize,
    nbins: usize,
    /// Grid-adaptation damping exponent.  Lepage recommends `α = 1.5`;
    /// reduce toward `0.5` for noisy integrands to slow adaptation.
    alpha: f64,
    /// Bin edges per dimension: `xi[d]` has length `nbins + 1`.
    xi: Vec<Vec<f64>>,
}

/// Result returned by [`Vegas::integrate`].
#[derive(Debug, Clone, Copy)]
pub struct VegasResult {
    /// Best estimate of the integral (weighted combination of all iterations).
    pub integral: f64,
    /// Standard deviation of the estimate.
    pub std_dev: f64,
    /// χ²/dof across iterations.  Values near 1 indicate consistency between
    /// iterations; large values warn of poor convergence or a pathological
    /// integrand.
    pub chi2_per_dof: f64,
}

impl Vegas {
    /// Create a new VEGAS integrator with a uniform initial grid.
    ///
    /// * `ndim`  – integration dimensions
    /// * `nbins` – bins per dimension (50–100 is typical)
    /// * `alpha` – grid-damping exponent (Lepage: 1.5)
    pub fn new(ndim: usize, nbins: usize, alpha: f64) -> Self {
        let xi = (0..ndim)
            .map(|_| (0..=nbins).map(|i| i as f64 / nbins as f64).collect())
            .collect();
        Vegas {
            ndim,
            nbins,
            alpha,
            xi,
        }
    }

    /// Integrate `f` over `[0, 1]^ndim`.
    ///
    /// * `f`     – integrand; receives a `&[f64]` of length `ndim`
    /// * `neval` – integrand evaluations per iteration
    /// * `niter` – number of adaptation iterations
    /// * `rng`   – random number source
    pub fn integrate(
        &mut self,
        mut f: impl FnMut(&[f64]) -> f64,
        neval: usize,
        niter: usize,
        rng: &mut impl Rng,
    ) -> VegasResult {
        let mut iter_results: Vec<(f64, f64)> = Vec::with_capacity(niter); // (I, σ²)

        for iter_idx in 0..niter {
            let (integral, var, d) = self.sample_iter(&mut f, neval, rng);
            iter_results.push((integral, var.max(f64::MIN_POSITIVE)));
            // Refine grid before all but the last iteration.
            if iter_idx + 1 < niter {
                self.refine_grid(&d, neval);
            }
        }

        // Combine iterations with 1/σ² weighting.
        let weight: f64 = iter_results.iter().map(|(_, v)| 1.0 / v).sum();
        let integral: f64 = iter_results.iter().map(|(i, v)| i / v).sum::<f64>() / weight;
        let std_dev = (1.0 / weight).sqrt();

        let chi2_per_dof = if niter > 1 {
            iter_results
                .iter()
                .map(|(i, v)| (i - integral).powi(2) / v)
                .sum::<f64>()
                / (niter - 1) as f64
        } else {
            0.0
        };

        VegasResult {
            integral,
            std_dev,
            chi2_per_dof,
        }
    }

    // ── Private helpers ────────────────────────────────────────────────────────

    /// Draw `neval` samples, return `(integral_estimate, variance, d)`.
    ///
    /// `d[dim][bin]` accumulates `fval²` for the grid-refinement step.
    fn sample_iter(
        &self,
        f: &mut impl FnMut(&[f64]) -> f64,
        neval: usize,
        rng: &mut impl Rng,
    ) -> (f64, f64, Vec<Vec<f64>>) {
        let nbins = self.nbins;
        let mut d = vec![vec![0.0_f64; nbins]; self.ndim];
        let mut x = vec![0.0_f64; self.ndim];
        let mut ks = vec![0_usize; self.ndim];

        let mut sum = 0.0_f64;
        let mut sum2 = 0.0_f64;

        for _ in 0..neval {
            let mut wgt = 1.0_f64;

            for dim in 0..self.ndim {
                let u: f64 = rng.random();
                let pos = u * nbins as f64;
                let k = (pos as usize).min(nbins - 1);
                let r = pos - k as f64;
                let lo = self.xi[dim][k];
                let hi = self.xi[dim][k + 1];
                x[dim] = lo + r * (hi - lo);
                wgt *= nbins as f64 * (hi - lo); // Jacobian dx/du = bin_width, wgt = 1/p(x)
                ks[dim] = k;
            }

            let fval = f(&x) * wgt;
            let fval2 = fval * fval;
            sum += fval;
            sum2 += fval2;
            for dim in 0..self.ndim {
                d[dim][ks[dim]] += fval2;
            }
        }

        let n = neval as f64;
        let mean = sum / n;
        // Unbiased variance of the estimator of the mean.
        let variance = ((sum2 / n - mean * mean) / (n - 1.0)).max(0.0);

        (mean, variance, d)
    }

    /// Reshape the grid using accumulated `f²` weights from one iteration.
    ///
    /// Each dimension is processed independently:
    /// 1. Normalize by expected hits per bin.
    /// 2. Apply Lepage's α-damping to smooth the importance function.
    /// 3. Smooth with nearest-neighbour averaging.
    /// 4. Redistribute bin edges so each new bin captures equal total weight.
    fn refine_grid(&mut self, d: &[Vec<f64>], neval: usize) {
        let nbins = self.nbins;
        let hits_per_bin = (neval / nbins).max(1) as f64;

        for (dim, d_dim) in d.iter().enumerate() {
            // Step 1: normalise by expected hits per bin.
            let mut m: Vec<f64> = d_dim
                .iter()
                .map(|&v| (v / hits_per_bin).max(1e-100))
                .collect();

            // Step 2: Lepage α-damping  m[k] → avg × ((m[k]/avg − 1) / ln(m[k]/avg))^α
            let avg = (m.iter().sum::<f64>() / nbins as f64).max(1e-100);
            for mk in &mut m {
                let ratio = (*mk / avg).max(1e-100);
                *mk = if (ratio - 1.0).abs() < 1e-7 {
                    avg
                } else {
                    avg * ((ratio - 1.0) / ratio.ln()).powf(self.alpha)
                };
                *mk = mk.max(1e-100);
            }

            // Step 3: nearest-neighbour smoothing.
            let m_copy = m.clone();
            for k in 0..nbins {
                let left = if k > 0 { m_copy[k - 1] } else { 0.0 };
                let right = if k + 1 < nbins { m_copy[k + 1] } else { 0.0 };
                m[k] = ((left + m_copy[k] + right) / 3.0).max(1e-100);
            }

            // Step 4: redistribute bin edges.
            let total: f64 = m.iter().sum();
            let target = total / nbins as f64; // weight per new bin

            let mut new_xi = vec![0.0_f64; nbins + 1];
            new_xi[0] = 0.0;
            new_xi[nbins] = 1.0;

            let mut acc = 0.0_f64;
            let mut k_old = 0_usize;
            let mut k_new = 1_usize;

            while k_new < nbins {
                while acc < target && k_old < nbins {
                    acc += m[k_old];
                    k_old += 1;
                }
                // k_old has overshot by `overshoot`; interpolate within old bin k_old-1.
                let overshoot = acc - target;
                let old_width = self.xi[dim][k_old] - self.xi[dim][k_old - 1];
                let frac = overshoot / m[k_old - 1];
                new_xi[k_new] =
                    (self.xi[dim][k_old] - frac * old_width).clamp(new_xi[k_new - 1], 1.0);
                acc = overshoot;
                k_new += 1;
            }

            self.xi[dim] = new_xi;
        }
    }
}

// ── Unit tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn seeded_rng() -> impl Rng {
        rand::rngs::StdRng::seed_from_u64(12345)
    }

    /// Constant integrand: ∫₀¹ 1 du = 1.
    #[test]
    fn test_constant() {
        let mut v = Vegas::new(1, 50, 1.5);
        let mut rng = seeded_rng();
        let r = v.integrate(|_| 1.0, 10_000, 5, &mut rng);
        assert!(
            (r.integral - 1.0).abs() < 0.01,
            "constant: {:.6}",
            r.integral
        );
    }

    /// Linear: ∫₀¹ 2u du = 1.
    #[test]
    fn test_linear() {
        let mut v = Vegas::new(1, 50, 1.5);
        let mut rng = seeded_rng();
        let r = v.integrate(|u| 2.0 * u[0], 10_000, 5, &mut rng);
        assert!((r.integral - 1.0).abs() < 0.01, "linear: {:.6}", r.integral);
    }

    /// Quadratic: ∫₀¹ 3u² du = 1.
    #[test]
    fn test_quadratic() {
        let mut v = Vegas::new(1, 50, 1.5);
        let mut rng = seeded_rng();
        let r = v.integrate(|u| 3.0 * u[0] * u[0], 10_000, 5, &mut rng);
        assert!(
            (r.integral - 1.0).abs() < 0.02,
            "quadratic: {:.6}",
            r.integral
        );
    }

    /// Gaussian peak: ∫₀¹ exp(−(u−0.3)²/0.01) du = 0.1·(√π/2)·(erf(7)+erf(3)).
    ///
    /// Derivation: substitute t = (u−0.3)/0.1, du = 0.1·dt, limits t ∈ [−3, 7]:
    ///   = 0.1 · ∫₋₃⁷ exp(−t²) dt = 0.1 · (√π/2) · (erf(7) + erf(3))
    ///
    /// VEGAS should adapt to the peak and converge quickly.
    #[test]
    fn test_peaked() {
        let exact = {
            // Exact analytic result via erf substitution t = (u−0.3)/0.1.
            // ∫₀¹ exp(−(u−0.3)²/0.01) du = 0.1·(√π/2)·(erf(7)+erf(3))
            let half_sqrt_pi = std::f64::consts::PI.sqrt() / 2.0;
            0.1 * half_sqrt_pi * (libm::erf(7.0) + libm::erf(3.0))
        };
        let mut v = Vegas::new(1, 50, 1.5);
        let mut rng = seeded_rng();
        let r = v.integrate(
            |u| (-(u[0] - 0.3).powi(2) / 0.01).exp(),
            50_000,
            10,
            &mut rng,
        );
        let rel = (r.integral - exact).abs() / exact;
        assert!(
            rel < 0.01,
            "peaked: got {:.6}, exact {exact:.6}, rel {rel:.4}",
            r.integral
        );
    }

    /// 2D: ∫₀¹∫₀¹ (u + v) du dv = 1.
    #[test]
    fn test_2d() {
        let mut v = Vegas::new(2, 50, 1.5);
        let mut rng = seeded_rng();
        let r = v.integrate(|u| u[0] + u[1], 20_000, 5, &mut rng);
        assert!((r.integral - 1.0).abs() < 0.02, "2d: {:.6}", r.integral);
    }
}
