//! Combining independent seeds of one estimate: the seed policy's one rule.
//!
//! Every place a validation target turns a set of equal-budget seeds into one
//! number — this crate's own seeds, or MadEvent's per-seed references — reads
//! them through [`combine`], so the two sides of a gate are the same statistic.
//!
//! The file depends on nothing but `std`, so a test target outside this crate's
//! `common` module (the CLI's) includes it by path and reads seeds the same way.
#![allow(dead_code)]

/// One seed's estimate and its quoted error.
pub trait SeedValue {
    fn value_and_err(&self) -> (f64, f64);
}

impl SeedValue for (f64, f64) {
    fn value_and_err(&self) -> (f64, f64) {
        *self
    }
}

/// Equal-budget seeds of one estimate, read under the seed policy.
///
/// * `mean` is the **unweighted** mean of the seeds;
/// * `err` is `max(quoted, spread)`, the uncertainty on `mean`;
/// * `quoted` is `√(Σσᵢ²)/n`, the seeds' own quoted errors carried onto the mean;
/// * `spread` is the seeds' sample standard deviation over `√n`;
/// * `chi2_per_dof` is the seeds' χ² about `mean` in units of their own quoted
///   errors, over `n − 1` degrees of freedom.
///
/// # Why unweighted
///
/// Seeds that run the same budget estimate the same integral with the same
/// expected variance, so each is one draw of equal standing. Weighting by
/// `1/σᵢ²` would make the weights draws too, correlated with the values they
/// weight: a VEGAS seed that missed a region reports a low integral *and* a low
/// variance, and a `1/σ²` mean up-weights exactly that seed — a confidently
/// wrong number, small error bar included. The unweighted mean has no such bias.
///
/// # Why `max(quoted, spread)`
///
/// A heavy-tailed estimator's quoted error understates its seed-to-seed scatter,
/// and on MadEvent's side seeds have scattered at χ²/dof far above one against
/// their quotes. Whichever of the two errors is larger is the one the mean is
/// read with; neither is allowed to undercut the other.
///
/// # Why the scatter stays separate
///
/// `chi2_per_dof` is the check that the seeds agree with their own quotes. It
/// is not folded into `err`: a gate bounds it on its own, because a scatter far
/// above one is a missed region and one far below is correlated seeds, and
/// neither shows in the mean.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SeedMean {
    pub mean: f64,
    pub err: f64,
    pub quoted: f64,
    pub spread: f64,
    pub chi2_per_dof: f64,
    pub n: usize,
}

/// Combine equal-budget seeds' `(value, quoted error)`; see [`SeedMean`].
///
/// Panics on fewer than two seeds: one seed has no spread, so the rule's error
/// and scatter do not exist for it.
pub fn combine<T: SeedValue>(seeds: &[T]) -> SeedMean {
    let values: Vec<(f64, f64)> = seeds.iter().map(SeedValue::value_and_err).collect();
    let n = values.len();
    assert!(
        n >= 2,
        "a seed combination needs two seeds or more, got {n}"
    );
    let nf = n as f64;
    let mean = values.iter().map(|(v, _)| v).sum::<f64>() / nf;
    let quoted = values.iter().map(|(_, e)| e * e).sum::<f64>().sqrt() / nf;
    let variance = values.iter().map(|(v, _)| (v - mean).powi(2)).sum::<f64>() / (nf - 1.0);
    let spread = (variance / nf).sqrt();
    let chi2_per_dof = values
        .iter()
        .map(|(v, e)| ((v - mean) / e).powi(2))
        .sum::<f64>()
        / (nf - 1.0);
    SeedMean {
        mean,
        err: quoted.max(spread),
        quoted,
        spread,
        chi2_per_dof,
        n,
    }
}

impl SeedMean {
    /// `(mean − reference) / √(err² + reference_err²)`.
    pub fn pull(&self, reference: f64, reference_err: f64) -> f64 {
        (self.mean - reference) / (self.err * self.err + reference_err * reference_err).sqrt()
    }
}
