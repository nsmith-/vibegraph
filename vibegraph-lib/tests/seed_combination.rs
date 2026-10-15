//! Known answers for the seed policy's combination rule, `common::seeds::combine`.
//!
//! Each case is worked in closed form, and each is built so that one way of
//! getting the rule wrong moves a field it pins: the seeds' values are unequal
//! and their errors unequal, so a `1/σ²`-weighted mean (or a χ² taken about one)
//! lands elsewhere, and one case is spread-dominated and the other
//! quoted-dominated, so dropping either term of `max(quoted, spread)` changes
//! the error on one of them.

mod common;

use common::report::SeedResult;
use common::seeds::combine;

fn close(what: &str, got: f64, want: f64) {
    assert!(
        (got - want).abs() <= 1e-12 * want.abs().max(1.0),
        "{what}: got {got:.17e}, want {want:.17e}"
    );
}

/// Four seeds scattering by more than they quote, the low one with the smallest
/// error: the missed-region shape the unweighted mean exists for. Deviations
/// −1, +3, −4, +2 about 101 give a sample variance of 30/3 = 10, so the spread
/// on the mean is √(10/4); the quotes combine to √6.25/4 = 0.625; and the χ² is
/// 1 + 9 + 64 + 1 over three degrees of freedom. A `1/σ²` mean would read 98.84.
#[test]
fn a_spread_dominated_sweep_reads_its_spread_about_the_unweighted_mean() {
    let c = combine(&[(100.0, 1.0), (104.0, 1.0), (97.0, 0.5), (103.0, 2.0)]);
    assert_eq!(c.n, 4);
    close("mean", c.mean, 101.0);
    close("quoted", c.quoted, 0.625);
    close("spread", c.spread, 2.5f64.sqrt());
    close("err", c.err, 2.5f64.sqrt());
    close("chi2/dof", c.chi2_per_dof, 25.0);
}

/// Three seeds scattering by less than they quote: the error is the quoted
/// √(0.36 + 0.64 + 1.44)/3, not the spread 0.2/√3. Deviations 0, +0.2, −0.2 give
/// χ² = (0.2/0.8)² + (0.2/1.2)² over two degrees of freedom. A `1/σ²` mean would
/// read 10.0345.
#[test]
fn a_quote_dominated_sweep_reads_its_quote() {
    let c = combine(&[(10.0, 0.6), (10.2, 0.8), (9.8, 1.2)]);
    close("mean", c.mean, 10.0);
    close("quoted", c.quoted, 2.44f64.sqrt() / 3.0);
    close("spread", c.spread, 0.2 / 3f64.sqrt());
    close("err", c.err, 2.44f64.sqrt() / 3.0);
    close("chi2/dof", c.chi2_per_dof, (0.0625 + 1.0 / 36.0) / 2.0);
}

/// The sweep form reads the same pairs, and the pull is formed on `err`.
#[test]
fn a_sweep_of_seed_results_combines_as_its_pairs() {
    let runs: Vec<SeedResult> = [(100.0, 1.0), (104.0, 1.0), (97.0, 0.5), (103.0, 2.0)]
        .iter()
        .zip(1u64..)
        .map(|(&(sigma_pb, sigma_err_pb), seed)| SeedResult {
            seed,
            sigma_pb,
            sigma_err_pb,
        })
        .collect();
    let c = combine(&runs);
    close("mean", c.mean, 101.0);
    close("err", c.err, 2.5f64.sqrt());
    // (101 − 99) / √(2.5 + 1.5): the spread, not the quote, in the denominator.
    close("pull", c.pull(99.0, 1.5f64.sqrt()), 1.0);
}

#[test]
#[should_panic(expected = "two seeds or more")]
fn one_seed_has_no_combination() {
    combine(&[(1.0, 0.1)]);
}
