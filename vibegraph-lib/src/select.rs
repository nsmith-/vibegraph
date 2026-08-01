//! Categorical draws off a diagonal weight vector — the per-event *selections*
//! that fill in an event record.
//!
//! A selection is not a sampling channel. The cross section is computed with
//! helicities summed and colours contracted; once a phase-space point has been
//! accepted, one helicity combination, one integration configuration and one
//! colour flow are drawn from the per-combination `|M_hel|²`, per-configuration
//! `AMP2` and per-flow `JAMP2` diagonals (MadGraph's `SELECT_HEL` and
//! `SELECT_COLOR`) purely to fill in the event record. The draw reads an
//! accumulator that already exists, enters no integrand, and moves no cross
//! section.
//!
//! Both draws are the same categorical step over non-negative weights, defined
//! once here.

/// Draw an index with probability `weights[i] / Σⱼ weights[j]` from a uniform
/// variate `u ∈ [0, 1)`.
///
/// Returns `None` when the weights carry no probability — all zero, negative, or
/// non-finite — so a caller sees an unusable accumulator rather than a
/// silently-index-0 event.
///
/// Negative entries are not filtered: a weight vector that is meant to be a
/// squared modulus and is not should surface at its source, and a negative total
/// is refused here.
pub fn select_index(weights: &[f64], u: f64) -> Option<usize> {
    let total: f64 = weights.iter().sum();
    if !(total > 0.0) || !total.is_finite() {
        return None;
    }
    let target = u * total;
    let mut acc = 0.0;
    for (i, &w) in weights.iter().enumerate() {
        acc += w;
        if target < acc {
            return Some(i);
        }
    }
    // Only reachable through rounding at `u → 1`; the last entry with weight wins.
    weights.iter().rposition(|&w| w > 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phasespace::rng::SubStream;

    #[test]
    fn follows_the_cumulative_weights() {
        let w = [1.0, 0.0, 3.0];
        assert_eq!(select_index(&w, 0.0), Some(0));
        assert_eq!(select_index(&w, 0.2), Some(0));
        // A zero-weight entry is never selected: `u = 0.25` lands exactly on the
        // boundary and falls through to the next entry with weight.
        assert_eq!(select_index(&w, 0.25), Some(2));
        assert_eq!(select_index(&w, 0.999), Some(2));
    }

    #[test]
    fn refuses_weights_that_are_not_a_distribution() {
        assert_eq!(select_index(&[0.0, 0.0], 0.5), None);
        assert_eq!(select_index(&[f64::NAN], 0.5), None);
        assert_eq!(select_index(&[-1.0, -2.0], 0.5), None);
        assert_eq!(select_index(&[], 0.5), None);
        assert_eq!(select_index(&[f64::INFINITY, 1.0], 0.5), None);
    }

    /// The empirical frequencies converge to the weight fractions. This is the
    /// property both `SELECT_HEL` and `SELECT_COLOR` rest on, and the one a
    /// cumulative-sum off-by-one would break while still returning valid indices.
    #[test]
    fn frequencies_converge_to_the_weight_fractions() {
        let weights = [0.5, 3.0, 0.0, 1.25, 0.25];
        let total: f64 = weights.iter().sum();
        let n = 400_000;
        let mut counts = vec![0usize; weights.len()];
        let mut s = SubStream::from_stream(0x5E1E_C700, 1);
        for _ in 0..n {
            let i = select_index(&weights, s.next_uniform::<f64>())
                .expect("weights are a distribution");
            counts[i] += 1;
        }
        // 4σ on a binomial frequency at this sample size.
        for (i, (&c, &w)) in counts.iter().zip(&weights).enumerate() {
            let p = w / total;
            let f = c as f64 / n as f64;
            let sigma = (p * (1.0 - p) / n as f64).sqrt();
            assert!(
                (f - p).abs() <= 4.0 * sigma + f64::EPSILON,
                "index {i}: frequency {f:.5} vs probability {p:.5} (4σ = {:.5})",
                4.0 * sigma
            );
        }
    }
}
