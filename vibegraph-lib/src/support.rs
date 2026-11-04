//! Whether a completed integration's points carry the cross section it reports.
//!
//! A Monte Carlo estimate is an average over the points that were drawn, and only
//! the points of nonzero weight contribute to it. A run in which none has weight
//! reports `σ = 0 ± 0`, which is not a measurement: the cuts reject everything
//! the sampler reaches, or the matrix element vanishes at the parameters in force.
//! A run in which a handful have weight reports a number whose value and error
//! bar are both set by those few draws. Either is refused here with the counts
//! that tell the causes apart, before anything downstream can read it as a
//! result.

use std::cell::Cell;

use crate::budget::ConvergenceReport;

/// Fewest effective points ([`ConvergenceReport::effective_points`]) a cross
/// section is accepted on.
///
/// An effective sample size rather than a count of nonzero points, because the
/// failure it exists for comes in both shapes. A run that almost never reaches
/// the region carrying σ keeps one or two nonzero points; a run whose grid
/// found the region late keeps hundreds of nonzero points of which a handful
/// carry nearly all the weight. `(Σw)²/Σw²` is at most the nonzero count, so it
/// refuses the first shape whenever a count would, and it sees the second,
/// which a count cannot.
///
/// Set from measurement, every integration a gate or a seed sweep ran. The
/// banked cross-section rows (`validate_sigma`, `validate_hadronic`, five seeds
/// each at their own budgets, 320 integrations) reach `1.1·10⁵` at the least.
/// The smallest-budget command-line run, `p p > l+ l- j` on six channels at
/// `--neval 2000 --niter 2` under dynamical scales, reaches a median of about
/// 480 over 21 seeds, with a heavy lower tail down to 90 on one seed. A
/// `2 → 2` through a 1 MeV-wide resonance whose decay window the sampler almost
/// never lands in reaches 0 to 2 over six seeds, and `e+ e- > mu+ mu-` with a
/// `ptl` the beams can barely reach 1 to 143 over eighteen runs (six seeds at
/// each of three `ptl`). The floor is ten times the resonance's worst and four
/// and a half times below the lowest legitimate run measured.
pub const MIN_EFFECTIVE_POINTS: f64 = 20.0;

/// What an integrand's evaluations came to, point by point: how many it
/// evaluated, how many of those passed the cuts, and how many of those carried
/// a nonzero weight.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PointCensus {
    pub evaluated: u64,
    pub passed_cuts: u64,
    pub nonzero: u64,
}

/// The censuses' counts added: per-thread counts into an integrand's, or
/// per-part counts into a sum's.
impl std::iter::Sum for PointCensus {
    fn sum<I: Iterator<Item = PointCensus>>(censuses: I) -> PointCensus {
        censuses.fold(PointCensus::default(), |acc, c| PointCensus {
            evaluated: acc.evaluated + c.evaluated,
            passed_cuts: acc.passed_cuts + c.passed_cuts,
            nonzero: acc.nonzero + c.nonzero,
        })
    }
}

/// One thread's running [`PointCensus`], kept beside its evaluation scratch.
#[derive(Debug, Default)]
pub(crate) struct CensusCell(Cell<PointCensus>);

impl CensusCell {
    /// A point the cuts rejected.
    pub(crate) fn rejected(&self) {
        let mut c = self.0.get();
        c.evaluated += 1;
        self.0.set(c);
    }

    /// A point that passed the cuts and came to `weight`.
    pub(crate) fn passed(&self, weight: f64) {
        let mut c = self.0.get();
        c.evaluated += 1;
        c.passed_cuts += 1;
        c.nonzero += u64::from(weight != 0.0);
        self.0.set(c);
    }

    pub(crate) fn get(&self) -> PointCensus {
        self.0.get()
    }

    pub(crate) fn clear(&self) {
        self.0.set(PointCensus::default());
    }
}

/// Why a completed integration's cross section is refused.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum SupportError {
    /// No point the integration evaluated passed the cuts.
    #[error(
        "every one of the {evaluated} points the integration evaluated has zero weight, and \
         none passed the cuts: the cuts reject every point the sampler reached, whether the \
         run card's or the bwcutoff window a decay chain holds each of its resonances in",
        evaluated = census.evaluated
    )]
    CutsRejectAll { census: PointCensus },
    /// Points passed the cuts, and every one of them had zero weight.
    #[error(
        "every one of the {evaluated} points the integration evaluated has zero weight, \
         though {passed} passed the cuts: |M|² (times the parton luminosity, on proton \
         beams) is zero on every point that passed, so a coupling or width is zero at the \
         parameters in force, or the restriction removes the process",
        evaluated = census.evaluated,
        passed = census.passed_cuts
    )]
    MatrixElementZero { census: PointCensus },
    /// Too few points carry the estimate for it to mean anything.
    #[error(
        "the cross section is carried by an effective {effective:.1} points, below the \
         {minimum} it is accepted on: {nonzero} of the {kept} points it is averaged over \
         have nonzero weight, and a few of them carry nearly all of it ({passed} of the \
         {evaluated} points the integration evaluated passed the cuts), so the estimate and \
         its error are set by those few draws",
        passed = census.passed_cuts,
        evaluated = census.evaluated
    )]
    TooFewEffective {
        effective: f64,
        nonzero: u64,
        kept: u64,
        minimum: f64,
        census: PointCensus,
    },
}

/// Refuse a completed integration whose points do not carry its cross section:
/// no point of nonzero weight anywhere in it, told apart by whether any point
/// passed the cuts, or an effective sample size below [`MIN_EFFECTIVE_POINTS`]
/// among the points the reported estimate averages.
///
/// `census` is the integrand's own count over the integration's points, warm-up
/// included; `report` is that integration's.
pub fn check_support(report: &ConvergenceReport, census: PointCensus) -> Result<(), SupportError> {
    if census.nonzero == 0 {
        return Err(if census.passed_cuts == 0 {
            SupportError::CutsRejectAll { census }
        } else {
            SupportError::MatrixElementZero { census }
        });
    }
    if !(report.effective_points >= MIN_EFFECTIVE_POINTS) {
        return Err(SupportError::TooFewEffective {
            effective: report.effective_points,
            nonzero: report.nonzero_points,
            kept: report.kept_points,
            minimum: MIN_EFFECTIVE_POINTS,
            census,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budget::{integrate_channels, BlockAllocation, Budget, StopSignal};
    use crate::unweight::ChannelIntegrand;
    use crate::vegas::IterationCombination;

    struct Flat;

    impl ChannelIntegrand for Flat {
        fn channel_count(&self) -> usize {
            1
        }
        fn channel_grid_ndim(&self, _channel: usize) -> usize {
            1
        }
        fn value_in_channel(&self, _channel: usize, _u: &[f64]) -> f64 {
            1.0
        }
    }

    fn report() -> ConvergenceReport {
        integrate_channels(
            &Flat,
            &[1.0],
            0.0,
            IterationCombination::Unweighted,
            Budget::Fixed {
                neval: 1_000,
                niter: 4,
            },
            BlockAllocation::ByAlpha,
            7,
            &StopSignal::default(),
        )
        .2
    }

    const SOME: PointCensus = PointCensus {
        evaluated: 4_000,
        passed_cuts: 3_000,
        nonzero: 2_000,
    };

    /// The cause is read off the census: no point through the cuts is the
    /// cuts, points through them with no weight is the matrix element.
    #[test]
    fn a_run_without_weight_names_the_cuts_or_the_matrix_element() {
        let r = report();
        let none = PointCensus {
            evaluated: 4_000,
            ..PointCensus::default()
        };
        assert_eq!(
            check_support(&r, none),
            Err(SupportError::CutsRejectAll { census: none })
        );
        let passed = PointCensus {
            passed_cuts: 17,
            ..none
        };
        assert_eq!(
            check_support(&r, passed),
            Err(SupportError::MatrixElementZero { census: passed })
        );
    }

    /// The floor is on the effective sample size, strictly: at it a run
    /// passes, under it or undefined it is refused.
    #[test]
    fn the_floor_is_on_the_effective_sample_size() {
        let mut r = report();
        assert!(check_support(&r, SOME).is_ok());
        r.effective_points = MIN_EFFECTIVE_POINTS;
        assert!(check_support(&r, SOME).is_ok());
        for below in [MIN_EFFECTIVE_POINTS - 1e-9, 1.0, f64::NAN] {
            r.effective_points = below;
            assert!(
                matches!(
                    check_support(&r, SOME),
                    Err(SupportError::TooFewEffective { .. })
                ),
                "an effective {below} points passed"
            );
        }
    }
}
