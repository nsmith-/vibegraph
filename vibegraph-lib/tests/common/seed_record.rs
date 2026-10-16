//! A MadEvent cross-section reference banked as independent seeds: the bundle's
//! per-row seed record, and the rule a σ gate reads it by.
//!
//! `validation/madgraph/bank_seeded_sigma.sh` runs each row's process directory
//! once per seed, each run in a freshly copied directory that no run has touched,
//! at the row's banked budget and at four times it. Per row it writes
//! `output/seeded_sigma/<row>/seed_record.json` beside the files each run kept
//! (`s<iseed>/`: the combined `results.dat`, every integration channel's
//! `G*/results.dat` with the `symfact.dat` that names them, the banner, and the
//! event file's `<init>` block). The reference bundle carries the directory.
//!
//! # The survey every seed runs
//!
//! Every banked run sets `hard_survey = 2` on its installed card. MadEvent's
//! default survey has left rows low by a missed survey grid or a long refine
//! chain, and the harder survey removes that at little cost. The setting is not
//! taken on trust: each seed records `min_events`/`max_events` from the run's own
//! `Source/run_config.inc`, which `pass_in_difficult_integration_mode`
//! (`madevent_interface.py`) rewrites to `2500·(2 + r)`/`10000·(1 + r)` for the
//! rate `r` it ran at. A card with `mmjj > 0.01·(ebeam1 + ebeam2)` enters that
//! mode at `r = 1` whatever `hard_survey` says, and the record shows it
//! ([`SurveySummary::as_set`]).
//!
//! # The heavy-tail flag
//!
//! Per integration channel, the seeds at the budget are combined like the row
//! ([`combine`]) and their χ²/dof read. A channel whose rare, very large weights
//! its own quote does not cover scatters far above one; the row's worst channel
//! above [`HEAVY_TAIL_CHI2_LIMIT`] flags it ([`HeavyTail`]). The flag reports; whether
//! the row's cell gates is the manifest's decision.
//!
//! # Two values per seed
//!
//! * **reported**: MadEvent's own `results.dat` total. Each channel's refine
//!   iterations enter it weighted by `mean²/variance` (`dsample.f`), so an
//!   iteration that missed a peak reads low *and* precise and counts for more.
//! * **equal weight**: per channel, the unweighted mean of its iterations, summed
//!   over channels. It is free of that weighting, and carries an optional-stopping
//!   exposure instead: how many iterations a channel takes depends on the
//!   precision it reached.
//!
//! A row reads its equal-weight values only where an adaptation-free estimate
//! has validated them (`validated_ew`); every other row reads the reported ones,
//! and the equal-weight values stay on the record beside them.
//!
//! # How a gate reads a record
//!
//! The seeds at the row's budget, under the seed policy ([`combine`]): their
//! unweighted mean, with `max(quoted, spread/√n)`. The seeds at four times the
//! budget are the record's screen against a budget shift and are not averaged in:
//! a mean over two budgets is no one estimator's expectation.
//!
//! The file depends on `std`, `serde`, `serde_json`, `puruspe` and the sibling
//! `seeds` module only, so the tool that writes records includes it by path and
//! the two sides cannot disagree about the format.
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::seeds::{combine, SeedMean};

/// The record format this module reads and writes.
pub const SCHEMA: u32 = 2;

/// Directory under the MadGraph work area (`validation/madgraph/output`) that
/// holds one subdirectory per seeded row.
pub const RECORD_DIR: &str = "seeded_sigma";

/// The record's file name inside a row's directory.
pub const RECORD_FILE: &str = "seed_record.json";

/// Fewest seeds at the row's budget a gate reads a record with: the seed
/// policy's floor.
pub const MIN_SEEDS: usize = 5;

/// The ratio of the screening budget to the row's budget.
pub const SCREEN_BUDGET_FACTOR: u64 = 4;

/// One row's seeds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeedRecord {
    pub schema: u32,
    pub row: String,
    /// How the runs were made: the driver, the process directory's source and
    /// the run card it ran on.
    pub generator: String,
    /// `VERSION` of the MadGraph tree that generated the process directory.
    pub mg_version: String,
    /// `pinned`, or `pinned + patches/<file>` for a patched copy of that tree.
    pub mg5_tree: String,
    /// The row's budget: the run card's `nevents` the reference is read at.
    pub budget_nevents: u64,
    /// Whether the gate reads the equal-weight values instead of the reported
    /// ones; set only for a row an adaptation-free estimate has validated them on.
    #[serde(default)]
    pub validated_ew: bool,
    /// The `hard_survey` every seed's installed run card set.
    pub hard_survey: u32,
    pub seeds: Vec<SeedRun>,
    /// What the writer computed from `seeds`, for a reader of the file. No gate
    /// reads it: [`SeedRecord::reference`] recomputes from the seeds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<RecordSummary>,
}

/// One seed: one fresh process directory, one `generate_events`.
///
/// A seed of a partitioned row (one run per window of a partition of the phase
/// space, each in its own directory) carries the sums over its windows, errors
/// in quadrature, and names the windows in `windows`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeedRun {
    pub iseed: u64,
    pub nevents: u64,
    pub reported_pb: f64,
    pub reported_err_pb: f64,
    pub equal_weight_pb: f64,
    pub equal_weight_err_pb: f64,
    /// Seconds of the whole `generate_events` call (compilation, survey, refine
    /// and events), summed over windows.
    pub wall_s: f64,
    /// Integration channels (`G*` directories `symfact.dat` lists), and the
    /// fewest and most refine iterations any of them recorded.
    pub channels: usize,
    pub iterations_min: usize,
    pub iterations_max: usize,
    /// The kept files, relative to the row's directory.
    pub files: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub windows: Vec<String>,
    /// Per process (`LPRUP`, the proc card's `@N`): the event file's `<init>`
    /// cross section as reported, and the equal-weight sum over the channels of
    /// that process's `P<N>_*` directories.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub by_lprup: BTreeMap<String, PartValues>,
    /// `hard_survey` on the run card the banner records.
    pub hard_survey: u32,
    /// `min_events` and `max_events` of the run's `Source/run_config.inc` as the
    /// run left it: the refine's per-iteration event range the survey mode set.
    pub min_events: u64,
    pub max_events: u64,
    /// Events the run's event file holds (in a partitioned seed, the fewest any
    /// window's holds: each window runs `nevents` of its own).
    pub events_written: u64,
    /// Per integration channel, `P<dir>/G<tag>` (`w<k>/P<dir>/G<tag>` in a
    /// window): its `results.dat` value and error, and its equal-weight value.
    pub by_channel: BTreeMap<String, ChannelValues>,
}

/// One integration channel of one seed.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChannelValues {
    pub value_pb: f64,
    pub err_pb: f64,
    pub equal_weight_pb: f64,
    pub iterations: usize,
}

/// One process's share of a seed.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PartValues {
    pub reported_pb: f64,
    pub reported_err_pb: f64,
    pub equal_weight_pb: f64,
    pub equal_weight_err_pb: f64,
}

/// The writer's account of a record, kept in the file for whoever reads it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecordSummary {
    pub reported: SummaryMean,
    pub equal_weight: SummaryMean,
    /// Equal weight minus reported, paired per seed at the budget: mean and
    /// standard error.
    pub ew_minus_reported_pb: f64,
    pub ew_minus_reported_err_pb: f64,
    pub screen: Screen,
    pub survey: SurveySummary,
    pub heavy_tail: HeavyTail,
}

/// The survey the seeds ran, read from their `run_config.inc`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SurveySummary {
    pub hard_survey: u32,
    /// Every distinct `[min_events, max_events]` over the seeds, each with the
    /// difficult-integration rate it is MadEvent's setting for (`None`: neither
    /// that mode's nor any rate's).
    pub run_config: Vec<RunConfig>,
    /// Every seed ran at the rate `hard_survey` asks for (`hard_survey = 0`: the
    /// default survey). `false` means the card's `hard_survey` was not what set
    /// the survey: the `mmjj` condition, or a template edited by hand.
    pub as_set: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunConfig {
    pub min_events: u64,
    pub max_events: u64,
    pub rate: Option<u32>,
}

/// `min_events`/`max_events` of a template no difficult-integration mode touched
/// (MadGraph 3.7.1's `madgraph/iolibs/template_files/madevent_run_config.inc`).
const DEFAULT_RUN_CONFIG: (u64, u64) = (1000, 2000);

/// The difficult-integration rate a `run_config.inc` pair is MadEvent's setting
/// for: `0` for the template's own, `r ≥ 1` for `2500·(2 + r)`/`10000·(1 + r)`.
pub fn survey_rate(min_events: u64, max_events: u64) -> Option<u32> {
    if (min_events, max_events) == DEFAULT_RUN_CONFIG {
        return Some(0);
    }
    let r = (min_events / 2500).checked_sub(2)?;
    (r >= 1 && min_events == 2500 * (2 + r) && max_events == 10000 * (1 + r))
        .then(|| u32::try_from(r).ok())
        .flatten()
}

/// A row's per-channel seed scatter at the budget.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HeavyTail {
    /// Channels every seed at the budget carries with a nonzero error.
    pub channels_tested: usize,
    /// Degrees of freedom of each channel's χ²: the seeds at the budget less one.
    pub dof: usize,
    /// The channel with the largest seed χ²/dof, if any was tested.
    pub worst: Option<ChannelScatter>,
    /// Every tested channel above [`HEAVY_TAIL_CHI2_LIMIT`], worst first.
    pub above_limit: Vec<ChannelScatter>,
    /// How many tested channels would sit above the limit by chance if every
    /// channel's quote were its true error: Σ P(χ²_dof > limit·dof).
    pub expected_above_by_chance: f64,
    /// The worst channel is above [`HEAVY_TAIL_CHI2_LIMIT`].
    pub flagged: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChannelScatter {
    pub channel: String,
    pub chi2_per_dof: f64,
    /// The channel's seed mean over the row's reported seed mean.
    pub share: f64,
}

/// The seed χ²/dof above which a channel is heavy-tailed: its quote does not
/// cover its scatter.
pub const HEAVY_TAIL_CHI2_LIMIT: f64 = 3.0;

/// [`SeedMean`] as written into a record.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SummaryMean {
    pub mean_pb: f64,
    pub err_pb: f64,
    pub quoted_pb: f64,
    pub spread_pb: f64,
    pub chi2_per_dof: f64,
    pub n: usize,
}

impl From<SeedMean> for SummaryMean {
    fn from(m: SeedMean) -> Self {
        SummaryMean {
            mean_pb: m.mean,
            err_pb: m.err,
            quoted_pb: m.quoted,
            spread_pb: m.spread,
            chi2_per_dof: m.chi2_per_dof,
            n: m.n,
        }
    }
}

/// The four-times-budget seeds against the budget's mean, on the estimator the
/// gate reads.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Screen {
    pub estimator: Estimator,
    /// Each screening seed's `(value − mean₁ₓ) / √(err² + err₁ₓ²)`.
    pub pulls: Vec<f64>,
    /// The screening seeds' own unweighted mean, read the same way; `None` with
    /// fewer than two of them.
    pub mean_pull: Option<f64>,
    /// `|mean_pull|` (or, with one screening seed, `|pulls[0]|`) above
    /// [`SCREEN_PULL_LIMIT`]: the row needs a budget ladder before it is banked.
    pub flagged: bool,
}

/// How far the screening seeds may sit from the budget's mean, in combined
/// standard errors, before the row is held for a budget ladder.
const SCREEN_PULL_LIMIT: f64 = 2.0;

/// Which of a seed's two values a reference is read from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Estimator {
    Reported,
    EqualWeight,
}

impl Estimator {
    pub fn label(self) -> &'static str {
        match self {
            Estimator::Reported => "reported",
            Estimator::EqualWeight => "equal-weight",
        }
    }
}

/// A record read under the seed policy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SeedReference {
    pub estimator: Estimator,
    pub read: SeedMean,
    pub budget_nevents: u64,
}

impl SeedReference {
    pub fn mean(&self) -> f64 {
        self.read.mean
    }

    pub fn err(&self) -> f64 {
        self.read.err
    }

    /// One line naming what a gate compared against.
    pub fn describe(&self, row: &str) -> String {
        format!(
            "[{row}] MadEvent reference from {RECORD_DIR}/{row}/{RECORD_FILE}: {} {} seeds at \
             nevents {}, {:.6e} +- {:.3e} pb (quoted {:.3e}, spread {:.3e}, chi2/dof {:.2})",
            self.read.n,
            self.estimator.label(),
            self.budget_nevents,
            self.read.mean,
            self.read.err,
            self.read.quoted,
            self.read.spread,
            self.read.chi2_per_dof,
        )
    }
}

impl SeedRun {
    pub fn value(&self, estimator: Estimator) -> (f64, f64) {
        match estimator {
            Estimator::Reported => (self.reported_pb, self.reported_err_pb),
            Estimator::EqualWeight => (self.equal_weight_pb, self.equal_weight_err_pb),
        }
    }
}

impl PartValues {
    pub fn value(&self, estimator: Estimator) -> (f64, f64) {
        match estimator {
            Estimator::Reported => (self.reported_pb, self.reported_err_pb),
            Estimator::EqualWeight => (self.equal_weight_pb, self.equal_weight_err_pb),
        }
    }
}

impl SeedRecord {
    /// The estimator a gate reads this row with.
    pub fn estimator(&self) -> Estimator {
        if self.validated_ew {
            Estimator::EqualWeight
        } else {
            Estimator::Reported
        }
    }

    /// The seeds run at the row's budget.
    pub fn at_budget(&self) -> Vec<&SeedRun> {
        self.seeds
            .iter()
            .filter(|s| s.nevents == self.budget_nevents)
            .collect()
    }

    /// The seeds run at [`SCREEN_BUDGET_FACTOR`] times the budget.
    pub fn screening(&self) -> Vec<&SeedRun> {
        self.seeds
            .iter()
            .filter(|s| s.nevents == SCREEN_BUDGET_FACTOR * self.budget_nevents)
            .collect()
    }

    /// Every promise a reader relies on, as an error naming the first one
    /// broken: the schema, distinct seeds, every seed at one of the two budgets,
    /// finite values, a positive reported cross section, an event file holding
    /// the events the seed ran for, and the record's `hard_survey` on every
    /// seed.
    pub fn check(&self) -> Result<(), String> {
        if self.schema != SCHEMA {
            return Err(format!(
                "[{}] seed record schema {} (this reader is {SCHEMA})",
                self.row, self.schema
            ));
        }
        let distinct: BTreeSet<u64> = self.seeds.iter().map(|s| s.iseed).collect();
        if distinct.len() != self.seeds.len() {
            return Err(format!(
                "[{}] {} seeds over {} distinct iseeds",
                self.row,
                self.seeds.len(),
                distinct.len()
            ));
        }
        for s in &self.seeds {
            if s.nevents != self.budget_nevents
                && s.nevents != SCREEN_BUDGET_FACTOR * self.budget_nevents
            {
                return Err(format!(
                    "[{}] seed {} ran {} events, neither the budget {} nor {}x it",
                    self.row, s.iseed, s.nevents, self.budget_nevents, SCREEN_BUDGET_FACTOR
                ));
            }
            let values = [
                s.reported_pb,
                s.reported_err_pb,
                s.equal_weight_pb,
                s.equal_weight_err_pb,
            ];
            if values.iter().any(|v| !v.is_finite()) {
                return Err(format!(
                    "[{}] seed {} has a non-finite value",
                    self.row, s.iseed
                ));
            }
            if s.reported_pb <= 0.0 {
                return Err(format!(
                    "[{}] seed {} reports {} pb, not a positive cross section",
                    self.row, s.iseed, s.reported_pb
                ));
            }
            if s.events_written < s.nevents {
                return Err(format!(
                    "[{}] seed {} wrote {} events of the {} it ran for",
                    self.row, s.iseed, s.events_written, s.nevents
                ));
            }
            if s.hard_survey != self.hard_survey {
                return Err(format!(
                    "[{}] seed {} ran hard_survey = {}, the record's runs set {}",
                    self.row, s.iseed, s.hard_survey, self.hard_survey
                ));
            }
        }
        Ok(())
    }

    /// The row's reference under the seed policy, on [`Self::estimator`].
    ///
    /// Panics on a record that fails [`Self::check`] or holds fewer than
    /// [`MIN_SEEDS`] seeds at its budget: a gate reading it would be reading
    /// less than the policy promises.
    pub fn reference(&self) -> SeedReference {
        self.reference_of(|s| s.value(self.estimator()))
    }

    /// One process's share (`LPRUP` = `id`), read the same way.
    pub fn part_reference(&self, id: &str) -> SeedReference {
        let estimator = self.estimator();
        self.reference_of(|s| {
            s.by_lprup
                .get(id)
                .unwrap_or_else(|| {
                    panic!(
                        "[{}] seed {} records no process {id} (it has {:?})",
                        self.row,
                        s.iseed,
                        s.by_lprup.keys().collect::<Vec<_>>()
                    )
                })
                .value(estimator)
        })
    }

    fn reference_of(&self, value: impl Fn(&SeedRun) -> (f64, f64)) -> SeedReference {
        if let Err(e) = self.check() {
            panic!("{e}");
        }
        let seeds = self.at_budget();
        assert!(
            seeds.len() >= MIN_SEEDS,
            "[{}] {} seeds at the budget {}, the seed policy reads no fewer than {MIN_SEEDS}",
            self.row,
            seeds.len(),
            self.budget_nevents
        );
        let values: Vec<(f64, f64)> = seeds.iter().map(|s| value(s)).collect();
        SeedReference {
            estimator: self.estimator(),
            read: combine(&values),
            budget_nevents: self.budget_nevents,
        }
    }

    /// What [`RecordSummary`] says about this record, computed from its seeds.
    /// `None` with fewer than two seeds at the budget, where no spread exists.
    pub fn summarise(&self) -> Option<RecordSummary> {
        let base = self.at_budget();
        if base.len() < 2 {
            return None;
        }
        let pairs = |e: Estimator| base.iter().map(|s| s.value(e)).collect::<Vec<_>>();
        let reported = combine(&pairs(Estimator::Reported));
        let equal_weight = combine(&pairs(Estimator::EqualWeight));
        let diffs: Vec<f64> = base
            .iter()
            .map(|s| s.equal_weight_pb - s.reported_pb)
            .collect();
        let n = diffs.len() as f64;
        let shift = diffs.iter().sum::<f64>() / n;
        let shift_err =
            (diffs.iter().map(|d| (d - shift).powi(2)).sum::<f64>() / (n - 1.0) / n).sqrt();

        let estimator = self.estimator();
        let gate = if estimator == Estimator::EqualWeight {
            equal_weight
        } else {
            reported
        };
        let screening: Vec<(f64, f64)> = self
            .screening()
            .iter()
            .map(|s| s.value(estimator))
            .collect();
        let pull = |(v, e): (f64, f64)| (v - gate.mean) / (e * e + gate.err * gate.err).sqrt();
        let pulls: Vec<f64> = screening.iter().copied().map(pull).collect();
        let mean_pull = (screening.len() >= 2).then(|| {
            let m = combine(&screening);
            pull((m.mean, m.err))
        });
        let worst = mean_pull.or_else(|| pulls.first().copied());
        let survey = self.survey();
        let heavy_tail = self.heavy_tail(reported.mean);
        Some(RecordSummary {
            reported: reported.into(),
            equal_weight: equal_weight.into(),
            ew_minus_reported_pb: shift,
            ew_minus_reported_err_pb: shift_err,
            screen: Screen {
                estimator,
                pulls,
                mean_pull,
                flagged: worst.is_some_and(|p| p.abs() > SCREEN_PULL_LIMIT),
            },
            survey,
            heavy_tail,
        })
    }

    /// The survey every seed, at either budget, ran.
    pub fn survey(&self) -> SurveySummary {
        let pairs: BTreeSet<(u64, u64)> = self
            .seeds
            .iter()
            .map(|s| (s.min_events, s.max_events))
            .collect();
        let run_config: Vec<RunConfig> = pairs
            .into_iter()
            .map(|(min_events, max_events)| RunConfig {
                min_events,
                max_events,
                rate: survey_rate(min_events, max_events),
            })
            .collect();
        let as_set = run_config.iter().all(|c| c.rate == Some(self.hard_survey));
        SurveySummary {
            hard_survey: self.hard_survey,
            run_config,
            as_set,
        }
    }

    /// Each channel's seed χ²/dof over the seeds at the budget, on its
    /// `results.dat` value and quote. `row_mean` scales each channel's share.
    pub fn heavy_tail(&self, row_mean: f64) -> HeavyTail {
        let base = self.at_budget();
        let dof = base.len().saturating_sub(1);
        let mut scatter: Vec<ChannelScatter> = Vec::new();
        if let Some(first) = base.first() {
            for name in first.by_channel.keys() {
                let values: Option<Vec<(f64, f64)>> = base
                    .iter()
                    .map(|s| {
                        s.by_channel
                            .get(name)
                            .filter(|c| c.err_pb > 0.0)
                            .map(|c| (c.value_pb, c.err_pb))
                    })
                    .collect();
                let Some(values) = values.filter(|v| v.len() >= 2) else {
                    continue;
                };
                let m = combine(&values);
                scatter.push(ChannelScatter {
                    channel: name.clone(),
                    chi2_per_dof: m.chi2_per_dof,
                    share: m.mean / row_mean,
                });
            }
        }
        scatter.sort_by(|a, b| b.chi2_per_dof.total_cmp(&a.chi2_per_dof));
        let chance = if dof > 0 {
            let k = dof as f64;
            puruspe::gammq(k / 2.0, HEAVY_TAIL_CHI2_LIMIT * k / 2.0)
        } else {
            0.0
        };
        let worst = scatter.first().cloned();
        HeavyTail {
            channels_tested: scatter.len(),
            dof,
            expected_above_by_chance: chance * scatter.len() as f64,
            flagged: worst
                .as_ref()
                .is_some_and(|w| w.chi2_per_dof > HEAVY_TAIL_CHI2_LIMIT),
            above_limit: scatter
                .into_iter()
                .filter(|c| c.chi2_per_dof > HEAVY_TAIL_CHI2_LIMIT)
                .collect(),
            worst,
        }
    }
}

/// Where a row's record sits under the MadGraph work area.
pub fn record_path(output_dir: &Path, row: &str) -> PathBuf {
    output_dir.join(RECORD_DIR).join(row).join(RECORD_FILE)
}

/// The row's seed record, or `None` when the work area holds none for it.
///
/// A record that exists and does not parse, or fails [`SeedRecord::check`], is a
/// broken bundle and panics: falling back to the single-run value would hide it.
pub fn read(output_dir: &Path, row: &str) -> Option<SeedRecord> {
    let path = record_path(output_dir, row);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => panic!("cannot read {}: {e}", path.display()),
    };
    let record: SeedRecord = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("{} does not parse: {e}", path.display()));
    assert_eq!(
        record.row,
        row,
        "{} records row {}",
        path.display(),
        record.row
    );
    if let Err(e) = record.check() {
        panic!("{}: {e}", path.display());
    }
    Some(record)
}

/// The row's MadEvent reference read from its seed record, or `None` when it
/// has none. A gate falls back to its single-run value on `None` only.
pub fn reference(output_dir: &Path, row: &str) -> Option<SeedReference> {
    let reference = read(output_dir, row)?.reference();
    eprintln!("{}", reference.describe(row));
    Some(reference)
}

/// [`reference()`] for one process's share of the row.
pub fn part_reference(output_dir: &Path, row: &str, lprup: &str) -> Option<SeedReference> {
    let reference = read(output_dir, row)?.part_reference(lprup);
    eprintln!("{} (process @{lprup})", reference.describe(row));
    Some(reference)
}
