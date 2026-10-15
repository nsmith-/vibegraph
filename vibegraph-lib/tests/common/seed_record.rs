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
//! The file depends on `std`, `serde` and the sibling `seeds` module only, so the
//! tool that writes records includes it by path and the two sides cannot
//! disagree about the format.
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::seeds::{combine, SeedMean};

/// The record format this module reads and writes.
pub const SCHEMA: u32 = 1;

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
}

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
pub const SCREEN_PULL_LIMIT: f64 = 2.0;

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

    /// Every structural promise a reader relies on, as an error naming the first
    /// one broken: the schema, distinct seeds, every seed at one of the two
    /// budgets, and finite values.
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
        })
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

/// [`reference`] for one process's share of the row.
pub fn part_reference(output_dir: &Path, row: &str, lprup: &str) -> Option<SeedReference> {
    let reference = read(output_dir, row)?.part_reference(lprup);
    eprintln!("{} (process @{lprup})", reference.describe(row));
    Some(reference)
}
