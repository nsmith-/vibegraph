//! Two-sample tests for comparing one event sample against another.
//!
//! Both tests here answer the same question — *could these two samples have come
//! from the same distribution?* — for the two kinds of column an event file
//! carries: a continuous observable (an invariant mass, an angle) through the
//! Kolmogorov–Smirnov statistic, and a categorical one (a helicity combination, a
//! colour connectivity, a flavour assignment) through the χ² homogeneity
//! statistic.
//!
//! # Weights are not decoration
//!
//! An accept/reject pass that keeps a point whose weight exceeded its channel's
//! estimated maximum hands it over *at a weight above one*, so an unweighted
//! sample is only nearly unweighted. Treating those events as unit-weight
//! misstates the distribution they represent by exactly the tail that is hardest
//! to sample, so both tests below take a weight per entry and use the weighted
//! empirical distribution.
//!
//! The price is that a weighted sample of `n` entries carries less information
//! than `n` independent ones. The standard summary of how much less is
//! [`effective_size`], `(Σw)²/Σw²`, which is `n` for equal weights and falls as
//! the weights spread; it is what both tests use in place of a count when they
//! turn a statistic into a p-value.
//!
//! # What these tests provably cannot detect
//!
//! * The KS statistic is a *maximum CDF gap*. It is most sensitive near the
//!   median of the distribution and least sensitive in the tails, so a
//!   misrepresented tail of small probability is exactly what it sees worst.
//!   A region carrying a percent of the sample can be wrong by tens of percent
//!   and leave the statistic inside its null distribution — which is why a
//!   localised discrepancy is chased with a binned comparison, not with this.
//! * Neither test sees anything about the *pairing* of columns: two samples with
//!   identical marginals and opposite correlations pass both, once per column.
//! * A p-value is a statement about a *fixed* observable chosen before the data.
//!   Taking the smallest p over many observables inflates the apparent
//!   significance, and the threshold a caller compares against has to account for
//!   how many were taken.

use puruspe::gammq;

/// Why a sample could not be tested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatsError {
    /// A sample had no entries, or no weight at all.
    Empty,
    /// A value or a weight was not finite, or a weight was negative — neither
    /// has an empirical distribution function.
    NotFinite,
    /// Fewer than two categories survived, so the χ² has no degrees of freedom.
    NoDegreesOfFreedom,
}

/// The effective number of independent entries a weighted sample carries,
/// `(Σw)²/Σw²` — `n` when the weights are equal, and less otherwise.
pub fn effective_size(weights: impl IntoIterator<Item = f64>) -> f64 {
    let (sum, sum_sq) = weights
        .into_iter()
        .fold((0.0, 0.0), |(s, q), w| (s + w, q + w * w));
    if sum_sq > 0.0 {
        sum * sum / sum_sq
    } else {
        0.0
    }
}

/// The outcome of a two-sample Kolmogorov–Smirnov test.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KsTest {
    /// The largest absolute gap between the two weighted empirical CDFs.
    pub d: f64,
    /// The probability of a gap at least this large if both samples came from
    /// one distribution.
    pub p: f64,
    /// [`effective_size`] of each sample, and the combined size the p-value is
    /// computed at.
    pub n_eff_a: f64,
    pub n_eff_b: f64,
}

/// Two-sample KS on weighted samples, each entry a `(value, weight)` pair in any
/// order.
///
/// The statistic is the maximum gap between the two weighted empirical CDFs; the
/// p-value is the asymptotic Kolmogorov distribution evaluated at the effective
/// sample size,
///
/// ```text
/// λ = (√nₑ + 0.12 + 0.11/√nₑ)·D,   Q(λ) = 2 Σ_{j≥1} (−1)^{j−1} exp(−2j²λ²),
/// nₑ = n_a·n_b/(n_a + n_b)
/// ```
///
/// with the `0.12 + 0.11/√nₑ` correction that makes the asymptotic form usable
/// down to small samples (Press et al., *Numerical Recipes* 3rd ed., §14.3.3,
/// eqs. 14.3.9 and 14.3.18). Substituting the effective size for the count is the
/// usual treatment of weighted samples: it is exact for equal weights and
/// conservative — a larger p — as the weights spread.
///
/// Ties are handled by advancing both CDFs past *every* entry sharing a value
/// before the gap is measured, so a value present in both samples contributes one
/// comparison rather than two order-dependent ones.
pub fn ks_two_sample(a: &[(f64, f64)], b: &[(f64, f64)]) -> Result<KsTest, StatsError> {
    let mut a = prepare(a)?;
    let mut b = prepare(b)?;
    a.sort_by(|x, y| x.0.total_cmp(&y.0));
    b.sort_by(|x, y| x.0.total_cmp(&y.0));

    let wa: f64 = a.iter().map(|e| e.1).sum();
    let wb: f64 = b.iter().map(|e| e.1).sum();
    if wa <= 0.0 || wb <= 0.0 {
        return Err(StatsError::Empty);
    }

    let (mut i, mut j) = (0usize, 0usize);
    let (mut fa, mut fb, mut d) = (0.0f64, 0.0f64, 0.0f64);
    while i < a.len() && j < b.len() {
        let x = a[i].0.min(b[j].0);
        while i < a.len() && a[i].0 <= x {
            fa += a[i].1 / wa;
            i += 1;
        }
        while j < b.len() && b[j].0 <= x {
            fb += b[j].1 / wb;
            j += 1;
        }
        d = d.max((fa - fb).abs());
    }
    // Whichever sample is exhausted first sits at 1 while the other climbs to it,
    // so the largest remaining gap is the one at this point.
    d = d.max((fa - fb).abs());

    let n_eff_a = effective_size(a.iter().map(|e| e.1));
    let n_eff_b = effective_size(b.iter().map(|e| e.1));
    let n_e = n_eff_a * n_eff_b / (n_eff_a + n_eff_b);
    let root = n_e.sqrt();
    let p = kolmogorov_q((root + 0.12 + 0.11 / root) * d);
    Ok(KsTest {
        d,
        p,
        n_eff_a,
        n_eff_b,
    })
}

/// `Q(λ) = 2 Σ_{j≥1} (−1)^{j−1} exp(−2j²λ²)`, the asymptotic probability that a
/// KS statistic exceeds `λ/√nₑ` under the null.
///
/// The alternating series converges fast for the `λ` that matter and is summed
/// until a term is negligible against the running sum; for very small `λ` it does
/// not converge in a bounded number of terms, and the limit `Q(0) = 1` is
/// returned (Press et al., *Numerical Recipes* 3rd ed., §14.3.3).
fn kolmogorov_q(lambda: f64) -> f64 {
    if !(lambda > 0.0) {
        return 1.0;
    }
    let a2 = -2.0 * lambda * lambda;
    let mut sum = 0.0f64;
    let mut fac = 2.0f64;
    let mut previous = 0.0f64;
    for j in 1..=200 {
        let term = fac * (a2 * (j * j) as f64).exp();
        sum += term;
        if term.abs() <= 1e-3 * previous.abs() || term.abs() <= 1e-8 * sum.abs() {
            return sum.clamp(0.0, 1.0);
        }
        fac = -fac;
        previous = term;
    }
    1.0
}

fn prepare(sample: &[(f64, f64)]) -> Result<Vec<(f64, f64)>, StatsError> {
    if sample.is_empty() {
        return Err(StatsError::Empty);
    }
    if sample
        .iter()
        .any(|&(v, w)| !v.is_finite() || !w.is_finite() || w < 0.0)
    {
        return Err(StatsError::NotFinite);
    }
    Ok(sample.to_vec())
}

/// The outcome of a χ² homogeneity test between two categorical samples.
#[derive(Debug, Clone, PartialEq)]
pub struct Chi2Test {
    pub chi2: f64,
    pub dof: usize,
    /// The probability of a χ² at least this large under the null.
    pub p: f64,
    /// Categories that entered the sum, including the pooled one if there is one.
    pub categories: usize,
    /// Categories too sparse to be compared on their own, pooled into a single
    /// residual category.
    pub pooled: usize,
    /// The share of the two samples' combined counts that landed in the pooled
    /// category — how much of the comparison is only made in aggregate.
    pub pooled_share: f64,
}

/// The fewest combined counts a category needs before its own χ² term is
/// meaningful; below it the asymptotic distribution of the term is not χ² with
/// one degree of freedom. The conventional threshold (Press et al., *Numerical
/// Recipes* 3rd ed., §14.3.1).
const MIN_CATEGORY_COUNT: f64 = 5.0;

/// χ² homogeneity between two categorical samples given as per-category counts in
/// a common category order.
///
/// ```text
/// χ² = Σᵢ (√(B/A)·aᵢ − √(A/B)·bᵢ)² / (aᵢ + bᵢ),   A = Σaᵢ, B = Σbᵢ
/// ```
///
/// — the two-dataset form that does not assume the samples have the same total
/// (Press et al., *Numerical Recipes* 3rd ed., §14.3.1, `chstwo`, eq. 14.3.3),
/// with one degree of freedom lost to the shared normalisation. Categories whose
/// combined count falls below [`MIN_CATEGORY_COUNT`] are summed into a single
/// residual category rather than dropped, so a discrepancy spread over many rare
/// categories still contributes; the residual itself is dropped only if even it
/// stays below the threshold, and [`Chi2Test::pooled_share`] reports how much of
/// the sample that is.
///
/// The counts may be *effective* counts of a weighted sample
/// ([`effective_counts`]); the formula does not care, but the p-value is only as
/// good as the claim that the counts are Poisson-like, which is what the
/// effective scaling buys.
pub fn chi2_homogeneity(a: &[f64], b: &[f64]) -> Result<Chi2Test, StatsError> {
    if a.len() != b.len() || a.is_empty() {
        return Err(StatsError::Empty);
    }
    if a.iter().chain(b).any(|c| !c.is_finite() || *c < 0.0) {
        return Err(StatsError::NotFinite);
    }
    let total_a: f64 = a.iter().sum();
    let total_b: f64 = b.iter().sum();
    if total_a <= 0.0 || total_b <= 0.0 {
        return Err(StatsError::Empty);
    }
    let (ra, rb) = ((total_b / total_a).sqrt(), (total_a / total_b).sqrt());

    let mut chi2 = 0.0;
    let mut used = 0usize;
    let mut pooled = 0usize;
    let (mut pool_a, mut pool_b) = (0.0f64, 0.0f64);
    for (&ai, &bi) in a.iter().zip(b) {
        if ai + bi < MIN_CATEGORY_COUNT {
            if ai + bi > 0.0 {
                pooled += 1;
                pool_a += ai;
                pool_b += bi;
            }
            continue;
        }
        let term = ra * ai - rb * bi;
        chi2 += term * term / (ai + bi);
        used += 1;
    }
    let pooled_share = (pool_a + pool_b) / (total_a + total_b);
    if pool_a + pool_b >= MIN_CATEGORY_COUNT {
        let term = ra * pool_a - rb * pool_b;
        chi2 += term * term / (pool_a + pool_b);
        used += 1;
    }
    if used < 2 {
        return Err(StatsError::NoDegreesOfFreedom);
    }
    let dof = used - 1;
    let p = gammq(dof as f64 / 2.0, chi2 / 2.0);
    Ok(Chi2Test {
        chi2,
        dof,
        p,
        categories: used,
        pooled,
        pooled_share,
    })
}

/// Per-category weight sums rescaled so that the total is the sample's effective
/// size — the counts [`chi2_homogeneity`] wants from a weighted sample.
///
/// The whole sample is scaled by one factor rather than each category by its own
/// effective size. A per-category factor would be an estimate from that
/// category's own handful of entries and would move the *shape* being tested;
/// one global factor leaves the shape alone and only deflates the totals by the
/// dispersion of the weights, which is the direction that costs sensitivity
/// rather than inventing it.
pub fn effective_counts(sum_w: &[f64], sum_w2: &[f64]) -> Vec<f64> {
    let total: f64 = sum_w.iter().sum();
    let total_sq: f64 = sum_w2.iter().sum();
    if total <= 0.0 || total_sq <= 0.0 {
        return vec![0.0; sum_w.len()];
    }
    let scale = total / total_sq;
    sum_w.iter().map(|w| w * scale).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;

    fn unit(values: &[f64]) -> Vec<(f64, f64)> {
        values.iter().map(|&v| (v, 1.0)).collect()
    }

    /// The statistic itself, on a case whose CDFs can be drawn by hand: the two
    /// samples interleave, and the largest gap is 1/2 − 1/4.
    #[test]
    fn the_statistic_is_the_largest_cdf_gap() {
        let a = unit(&[0.0, 1.0, 2.0, 3.0]);
        let b = unit(&[0.5, 1.5, 2.5, 3.5]);
        let ks = ks_two_sample(&a, &b).unwrap();
        assert!((ks.d - 0.25).abs() < 1e-12, "D = {}", ks.d);
        assert_eq!(ks.n_eff_a, 4.0);
    }

    /// A tail gap is found wherever it sits, and the sign of the difference does
    /// not matter.
    #[test]
    fn the_statistic_is_symmetric_in_its_arguments() {
        let a = unit(&[0.0, 1.0, 2.0, 3.0, 10.0]);
        let b = unit(&[0.0, 1.0, 2.0, 3.0, 4.0]);
        let ab = ks_two_sample(&a, &b).unwrap();
        let ba = ks_two_sample(&b, &a).unwrap();
        assert_eq!(ab.d, ba.d);
        assert_eq!(ab.p, ba.p);
    }

    /// Under the null the p-value is uniform. Checked as a distribution over many
    /// independent pairs rather than on one draw: a single p says nothing.
    #[test]
    fn p_values_are_uniform_when_both_samples_share_a_distribution() {
        let mut rng = ChaCha8Rng::seed_from_u64(0x4B53_0001);
        let mut ps = Vec::new();
        for _ in 0..400 {
            let a: Vec<(f64, f64)> = (0..300).map(|_| (rng.random::<f64>(), 1.0)).collect();
            let b: Vec<(f64, f64)> = (0..200).map(|_| (rng.random::<f64>(), 1.0)).collect();
            ps.push(ks_two_sample(&a, &b).unwrap().p);
        }
        let below_5 = ps.iter().filter(|&&p| p < 0.05).count();
        let below_50 = ps.iter().filter(|&&p| p < 0.5).count();
        // 400 draws: 20 ± 4.4 expected below 0.05, 200 ± 10 below 0.5.
        assert!((8..=36).contains(&below_5), "{below_5}/400 below 0.05");
        assert!((165..=235).contains(&below_50), "{below_50}/400 below 0.5");
        assert!(
            ps.iter().all(|&p| (0.0..=1.0).contains(&p)),
            "p-values leave [0,1]"
        );
    }

    /// A shift of a fifth of the range is caught at 300 against 200 entries.
    #[test]
    fn a_shifted_distribution_is_caught() {
        let mut rng = ChaCha8Rng::seed_from_u64(0x4B53_0002);
        let a: Vec<(f64, f64)> = (0..300).map(|_| (rng.random::<f64>(), 1.0)).collect();
        let b: Vec<(f64, f64)> = (0..200).map(|_| (rng.random::<f64>() + 0.2, 1.0)).collect();
        let ks = ks_two_sample(&a, &b).unwrap();
        assert!(ks.p < 1e-4, "p = {} for a 0.2 shift", ks.p);
    }

    /// The weights are the point of the weighted form: the same points carrying
    /// weight `2x` represent the density `2x`, and the test must see that — both
    /// by rejecting the uniform sample it is not, and by accepting the `2x` sample
    /// it is. Pretending the weights were 1 would invert both answers.
    #[test]
    fn weights_move_the_distribution_being_tested() {
        let mut rng = ChaCha8Rng::seed_from_u64(0x4B53_0003);
        let uniform: Vec<(f64, f64)> = (0..4000).map(|_| (rng.random::<f64>(), 1.0)).collect();
        let reweighted: Vec<(f64, f64)> = (0..4000)
            .map(|_| {
                let x = rng.random::<f64>();
                (x, 2.0 * x)
            })
            .collect();
        // sqrt(u) has density 2x on [0,1].
        let drawn_from_2x: Vec<(f64, f64)> = (0..4000)
            .map(|_| (rng.random::<f64>().sqrt(), 1.0))
            .collect();

        let against_uniform = ks_two_sample(&reweighted, &uniform).unwrap();
        let against_2x = ks_two_sample(&reweighted, &drawn_from_2x).unwrap();
        assert!(
            against_uniform.p < 1e-12,
            "weighted sample passed as uniform: p = {}",
            against_uniform.p
        );
        assert!(
            against_2x.p > 0.01,
            "weighted sample rejected against its own density: p = {}",
            against_2x.p
        );
        // The weights cost information: 4000 entries weighted by 2x carry three
        // quarters of that many independent ones.
        assert!(
            (against_2x.n_eff_a / 4000.0 - 0.75).abs() < 0.03,
            "effective size {} of 4000",
            against_2x.n_eff_a
        );
    }

    #[test]
    fn a_sample_with_a_non_finite_entry_is_refused_rather_than_ordered() {
        let a = unit(&[0.0, f64::NAN]);
        let b = unit(&[0.0, 1.0]);
        assert_eq!(ks_two_sample(&a, &b), Err(StatsError::NotFinite));
        assert_eq!(ks_two_sample(&[], &b), Err(StatsError::Empty));
    }

    /// The χ² value on counts small enough to add up by hand, against the
    /// tabulated 5% point of the χ² distribution with two degrees of freedom
    /// (5.991).
    #[test]
    fn the_chi_squared_matches_a_hand_computed_table() {
        // Equal totals, so the scale factors are 1 and each term is
        // (a−b)²/(a+b): 1/41 + 4/38 + 1/21 = 0.1774…
        let a = [20.0, 20.0, 10.0];
        let b = [21.0, 18.0, 11.0];
        let test = chi2_homogeneity(&a, &b).unwrap();
        assert!((test.chi2 - (1.0 / 41.0 + 4.0 / 38.0 + 1.0 / 21.0)).abs() < 1e-12);
        assert_eq!(test.dof, 2);
        assert!(test.p > 0.9, "p = {}", test.p);

        // And the p-value against the table: 5.991 is the 5% point of the χ²
        // distribution with two degrees of freedom, 18.307 the 5% point with ten.
        assert!((gammq(1.0, 5.991 / 2.0) - 0.05).abs() < 1e-4);
        assert!((gammq(5.0, 18.307 / 2.0) - 0.05).abs() < 1e-4);
    }

    /// Under the null the χ² p-value is uniform too, and a genuinely different
    /// set of category probabilities is caught.
    #[test]
    fn chi_squared_separates_equal_from_different_category_probabilities() {
        let mut rng = ChaCha8Rng::seed_from_u64(0x4331_0001);
        let draw = |rng: &mut ChaCha8Rng, p: &[f64], n: usize| {
            let mut counts = vec![0.0; p.len()];
            for _ in 0..n {
                let u: f64 = rng.random();
                let mut acc = 0.0;
                for (k, pk) in p.iter().enumerate() {
                    acc += pk;
                    if u < acc {
                        counts[k] += 1.0;
                        break;
                    }
                }
            }
            counts
        };
        let same = [0.4, 0.3, 0.2, 0.1];
        let other = [0.34, 0.3, 0.2, 0.16];

        let mut below_5 = 0;
        for _ in 0..200 {
            let a = draw(&mut rng, &same, 2000);
            let b = draw(&mut rng, &same, 1000);
            if chi2_homogeneity(&a, &b).unwrap().p < 0.05 {
                below_5 += 1;
            }
        }
        assert!((3..=22).contains(&below_5), "{below_5}/200 below 0.05");

        let a = draw(&mut rng, &same, 20000);
        let b = draw(&mut rng, &other, 10000);
        let test = chi2_homogeneity(&a, &b).unwrap();
        assert!(test.p < 1e-6, "p = {} for a 6% shift in one bin", test.p);
    }

    /// Sparse categories are pooled rather than compared one entry at a time, and
    /// the share that went into the pool is reported instead of hidden.
    #[test]
    fn sparse_categories_are_pooled_and_reported() {
        let a = [100.0, 100.0, 1.0, 1.0, 1.0, 0.0];
        let b = [100.0, 100.0, 1.0, 1.0, 1.0, 0.0];
        let test = chi2_homogeneity(&a, &b).unwrap();
        // Two dense categories plus one pooled residual of six entries.
        assert_eq!(test.categories, 3);
        assert_eq!(test.pooled, 3);
        assert!((test.pooled_share - 6.0 / 406.0).abs() < 1e-12);
        assert_eq!(test.dof, 2);
    }

    #[test]
    fn a_single_category_has_no_degrees_of_freedom() {
        assert_eq!(
            chi2_homogeneity(&[10.0], &[12.0]),
            Err(StatsError::NoDegreesOfFreedom)
        );
    }

    /// Effective counts keep the shape and lose only the total, which is what
    /// makes them safe to feed a χ² built for counts.
    #[test]
    fn effective_counts_rescale_the_total_and_not_the_shape() {
        // Half the entries at weight 1, half at weight 3: 200 entries carrying
        // (Σw)²/Σw² = 400²/(100·1 + 100·9) = 160 effective ones.
        let sum_w = [100.0, 300.0];
        let sum_w2 = [100.0, 900.0];
        let counts = effective_counts(&sum_w, &sum_w2);
        assert!((counts.iter().sum::<f64>() - 160.0).abs() < 1e-9);
        assert!((counts[1] / counts[0] - 3.0).abs() < 1e-12);
    }
}
