//! The σ gates' pulls against MadEvent's seed records, read as one population.
//!
//! Each σ gate writes one row file per measurement under
//! `target/validation-report/integrals/` (this side's value and error, the
//! MadEvent value and error it compared against, and its pull). One pull says
//! nothing; the population does. This tool joins every row file to the seed
//! record it was read against and re-reads the same population against each
//! value the record holds:
//!
//! * `R`: the reported values at the row's budget, under the seed policy
//!   (`combine`: unweighted mean, `max(quoted, spread/√n)`). This is what the
//!   gates read on every row not marked `validated_ew`, so it reproduces their
//!   pulls.
//! * `EW`: the equal-weight values at the budget, under the same rule.
//! * `EWmed`: the median of the equal-weight values, with the `EW` error
//!   scaled by `√(π/2)`, the asymptotic efficiency of a Gaussian median. It is
//!   printed beside rows whose equal-weight values are heavy-tailed: their
//!   seed spread is more than [`HEAVY_SPREAD_RATIO`] times the reported values'.
//! * `R4x`, `EW4x`: the seeds at four times the budget, under the same rule.
//!   Two seeds give a spread estimate with one degree of freedom, so a row's own
//!   4× pull carries little; across the population they screen whether the lean
//!   moves with the budget.
//!
//! A row file is joined to a record by value, not by name: the record whose
//! gate reading (total, or one process's `by_lprup` share) equals the row
//! file's MadEvent value to [`MATCH_REL`] is the one the gate read. A row file
//! no record reproduces (a control read from another reference) is listed and
//! left out.
//!
//! # What the population is read with
//!
//! Per estimator: the mean pull, its spread, and two standard errors of the
//! mean. The naive one treats rows as independent. They are not: every record
//! ran the same `iseed` list, and two rows whose processes differ only by a
//! parameter or a restriction draw nearly the same MadEvent fluctuations. Rows
//! whose reported seeds sit at the same standardized deviations (RMS
//! difference below [`SAME_DRAW_RMS`] over a shared `iseed` list), or that read
//! the same record, are joined into one cluster, provided their records share an
//! integration channel name (the same MadGraph subprocess directory and
//! channel), so that a chance match between unrelated processes does not link
//! them; and the clustered standard
//! error sums each cluster's residuals before squaring. This side's own
//! correlations (one RNG seed shared by every fixed-beam row) are not detected;
//! a cluster holding them would widen the error further.
//!
//! Relative units separate a shift of σ from underestimated errors: the
//! unweighted and the inverse-variance weighted mean of `σ_vg/σ_MG − 1`, and the
//! weighted mean's χ²/dof.
//!
//! The population is split by row class and by the record's paired
//! equal-weight − reported difference, to see whether the lean tracks it.
//!
//! # This side over several seeds
//!
//! `--this-side FILE` reads lines `record seed sigma_pb err_pb` (blank lines and
//! `#` comments skipped): this side's values of a record's row over several RNG
//! seeds, from a seed sweep. The population is then read a second time with each
//! such row's this-side value replaced by the seeds' seed-policy mean.
//!
//! Usage:
//!
//! ```text
//! sigma-pulls --report target/validation-report --output validation/madgraph/output \
//!             --manifest validation/manifest.toml [--this-side FILE]
//! ```
//!
//! It reads only, and writes nothing.

#[path = "../../../vibegraph-lib/tests/common/seeds.rs"]
mod seeds;

#[path = "../../../vibegraph-lib/tests/common/seed_record.rs"]
mod seed_record;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use seed_record::{Estimator, SeedRecord, SeedRun, RECORD_DIR, RECORD_FILE};
use seeds::{combine, SeedMean};

/// Relative agreement at which a row file's MadEvent value is the reading of a
/// record. Both sides compute the same `combine` over the same JSON numbers, so
/// they agree to rounding; any other record's reading differs by orders more.
const MATCH_REL: f64 = 1e-12;

/// A row's equal-weight values are heavy-tailed when their seed spread exceeds
/// this multiple of the reported values' spread over the same seeds.
const HEAVY_SPREAD_RATIO: f64 = 2.0;

/// Two rows draw the same MadEvent fluctuations when their reported seeds'
/// standardized deviations differ by less than this RMS over a shared `iseed`
/// list. Independent draws differ by about `√(2(n−1)/n)` (1.26 at five seeds);
/// one below `0.35` by chance has probability about 1% at five seeds
/// (the squared differences sum to twice a χ² on four degrees of freedom).
const SAME_DRAW_RMS: f64 = 0.35;

/// A row's paired equal-weight − reported difference counts as resolved above
/// this many of its own standard errors.
const EW_SHIFT_RESOLVED: f64 = 2.0;

/// The references a row's pull is re-read against.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Reference {
    Reported,
    EqualWeight,
    EqualWeightMedian,
    Reported4x,
    EqualWeight4x,
}

impl Reference {
    const ALL: [Reference; 5] = [
        Reference::Reported,
        Reference::EqualWeight,
        Reference::EqualWeightMedian,
        Reference::Reported4x,
        Reference::EqualWeight4x,
    ];

    fn label(self) -> &'static str {
        match self {
            Reference::Reported => "R",
            Reference::EqualWeight => "EW",
            Reference::EqualWeightMedian => "EWmed",
            Reference::Reported4x => "R4x",
            Reference::EqualWeight4x => "EW4x",
        }
    }
}

/// The class a row belongs to, for the population's split.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Class {
    FixedBeam,
    Bsm,
    Grammar,
    Hadronic,
    Mlm,
}

impl Class {
    const ALL: [Class; 5] = [
        Class::FixedBeam,
        Class::Bsm,
        Class::Grammar,
        Class::Hadronic,
        Class::Mlm,
    ];

    fn label(self) -> &'static str {
        match self {
            Class::FixedBeam => "fixed-beam SM",
            Class::Bsm => "toy/SMEFT",
            Class::Grammar => "grammar",
            Class::Hadronic => "hadronic",
            Class::Mlm => "MLM",
        }
    }

    /// MLM rows by name (`mlm`, `xqcut`); hadronic by a proton beam in the
    /// process; grammar by a restriction (`$`, a polarization `{`, or an
    /// s-channel requirement, a second `>`); toy/SMEFT by a manifest `model`;
    /// the rest fixed-beam Standard Model.
    fn of(row: &str, process: &str, has_model: bool) -> Class {
        if row.contains("mlm") || row.contains("xqcut") {
            Class::Mlm
        } else if process.trim_start().starts_with("p p") {
            Class::Hadronic
        } else if process.contains('$') || process.contains('{') || process.matches('>').count() > 1
        {
            Class::Grammar
        } else if has_model {
            Class::Bsm
        } else {
            Class::FixedBeam
        }
    }
}

/// One σ measurement a gate wrote: the fields of a row file this tool reads.
#[derive(Clone, Debug)]
struct Cell {
    row: String,
    variant: Option<String>,
    process: String,
    mode: String,
    sigma_vg: f64,
    sigma_vg_err: f64,
    sigma_mg: f64,
    pull: f64,
    seeds: usize,
}

impl Cell {
    /// The row file's name, marked where the cell is measured but not gated.
    fn name(&self) -> String {
        let base = match &self.variant {
            Some(v) => format!("{}__{v}", self.row),
            None => self.row.clone(),
        };
        if self.mode == "gate" {
            base
        } else {
            format!("{base} ({})", self.mode)
        }
    }

    /// A row file's cell. A missing or `null` number is an error: the gates
    /// write a non-finite value as `null`, and skipping that row would shrink
    /// the population unseen.
    fn parse(text: &str) -> Result<Cell, String> {
        let v: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        let num = |k: &str| {
            v.get(k)
                .and_then(serde_json::Value::as_f64)
                .ok_or_else(|| format!("{k:?} is missing or not a finite number"))
        };
        let (sigma_vg, sigma_vg_err, sigma_mg, pull) = (
            num("sigma_vg_pb")?,
            num("sigma_vg_err_pb")?,
            num("sigma_mg_pb")?,
            num("pull")?,
        );
        let s = |k: &str| {
            v.get(k)
                .and_then(serde_json::Value::as_str)
                .map(String::from)
        };
        Ok(Cell {
            row: s("row").ok_or("no row")?,
            variant: s("variant"),
            process: s("process").unwrap_or_default(),
            mode: s("mode").unwrap_or_default(),
            sigma_vg,
            sigma_vg_err,
            sigma_mg,
            pull,
            seeds: v
                .get("per_seed")
                .and_then(serde_json::Value::as_array)
                .map_or(1, Vec::len),
        })
    }
}

/// Which values of a record a row reads: the seed's total, or one process's
/// share.
#[derive(Clone, Debug, PartialEq)]
enum Part {
    Total,
    Lprup(String),
}

fn seed_value(run: &SeedRun, part: &Part, estimator: Estimator) -> Option<(f64, f64)> {
    match part {
        Part::Total => Some(run.value(estimator)),
        Part::Lprup(id) => run.by_lprup.get(id).map(|p| p.value(estimator)),
    }
}

fn values(seeds: &[&SeedRun], part: &Part, estimator: Estimator) -> Option<Vec<(f64, f64)>> {
    seeds
        .iter()
        .map(|s| seed_value(s, part, estimator))
        .collect()
}

/// A mean and its error, read some way from a set of seeds.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Value {
    mean: f64,
    err: f64,
}

fn median(xs: &[f64]) -> f64 {
    let mut v = xs.to_vec();
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        0.5 * (v[n / 2 - 1] + v[n / 2])
    }
}

/// The record's value on one reference, or `None` where the record has too few
/// seeds for it (fewer than two at four times the budget).
fn reference_value(record: &SeedRecord, part: &Part, reference: Reference) -> Option<Value> {
    let (seeds, estimator) = match reference {
        Reference::Reported => (record.at_budget(), Estimator::Reported),
        Reference::EqualWeight | Reference::EqualWeightMedian => {
            (record.at_budget(), Estimator::EqualWeight)
        }
        Reference::Reported4x => (record.screening(), Estimator::Reported),
        Reference::EqualWeight4x => (record.screening(), Estimator::EqualWeight),
    };
    let vals = values(&seeds, part, estimator)?;
    if vals.len() < 2 {
        return None;
    }
    let read = combine(&vals);
    Some(if reference == Reference::EqualWeightMedian {
        let xs: Vec<f64> = vals.iter().map(|v| v.0).collect();
        Value {
            mean: median(&xs),
            err: read.err * (std::f64::consts::PI / 2.0).sqrt(),
        }
    } else {
        Value {
            mean: read.mean,
            err: read.err,
        }
    })
}

/// The reading a gate takes of the record (its own estimator, at the budget).
fn gate_reading(record: &SeedRecord, part: &Part) -> Option<f64> {
    let vals = values(&record.at_budget(), part, record.estimator())?;
    (vals.len() >= 2).then(|| combine(&vals).mean)
}

/// The record, and the part of it, whose gate reading is `sigma_mg`.
fn find_record(
    records: &BTreeMap<String, SeedRecord>,
    sigma_mg: f64,
) -> Result<Option<(&SeedRecord, Part)>, String> {
    let mut hits = Vec::new();
    for record in records.values() {
        let mut parts = vec![Part::Total];
        if let Some(first) = record.at_budget().first() {
            parts.extend(first.by_lprup.keys().cloned().map(Part::Lprup));
        }
        for part in parts {
            if let Some(m) = gate_reading(record, &part) {
                if ((m - sigma_mg) / sigma_mg).abs() < MATCH_REL {
                    hits.push((record, part));
                }
            }
        }
    }
    match hits.len() {
        0 => Ok(None),
        1 => Ok(hits.pop()),
        _ => Err(format!(
            "MadEvent value {sigma_mg:e} is the reading of several records: {}",
            hits.iter()
                .map(|(r, p)| format!("{} {p:?}", r.row))
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// One row file joined to its record, re-read on every reference.
#[derive(Clone, Debug)]
struct Row {
    cell: Cell,
    record: String,
    part: Part,
    class: Class,
    /// This side's value, its error, how many seeds it is the mean of, and its
    /// seeds' χ²/dof where it is a mean read here.
    this: (f64, f64, usize, Option<f64>),
    refs: BTreeMap<Reference, Value>,
    /// Reported over the budget seeds: their standardized deviations, by iseed.
    draws: Vec<(u64, f64)>,
    /// The integration channels (`P<dir>/G<tag>`) the record's first seed at
    /// the budget carries.
    channels: BTreeSet<String>,
    /// Paired equal-weight − reported difference relative to the reported mean,
    /// and its standard error.
    ew_shift: (f64, f64),
    heavy: bool,
}

impl Row {
    fn pull(&self, reference: Reference) -> Option<f64> {
        let r = self.refs.get(&reference)?;
        Some((self.this.0 - r.mean) / self.this.1.hypot(r.err))
    }

    /// `σ_vg/σ_ref − 1` and its error.
    fn rel(&self, reference: Reference) -> Option<(f64, f64)> {
        let r = self.refs.get(&reference)?;
        Some((
            self.this.0 / r.mean - 1.0,
            self.this.1.hypot(r.err) / r.mean,
        ))
    }

    fn mg_variance_share(&self) -> f64 {
        let r = self.refs[&Reference::Reported].err;
        r * r / (r * r + self.this.1 * self.this.1)
    }

    fn ew_shift_z(&self) -> f64 {
        if self.ew_shift.1 > 0.0 {
            self.ew_shift.0 / self.ew_shift.1
        } else {
            0.0
        }
    }
}

fn build_row(cell: Cell, record: &SeedRecord, part: Part, has_model: bool) -> Row {
    let refs = Reference::ALL
        .iter()
        .filter_map(|&r| reference_value(record, &part, r).map(|v| (r, v)))
        .collect();
    let budget = record.at_budget();
    let reported = values(&budget, &part, Estimator::Reported).expect("the gate read it");
    let ew = values(&budget, &part, Estimator::EqualWeight).expect("the gate read it");
    let r_read = combine(&reported);
    let e_read = combine(&ew);
    let draws = budget
        .iter()
        .zip(&reported)
        .map(|(s, (x, e))| (s.iseed, (x - r_read.mean) / e))
        .collect();
    let diffs: Vec<f64> = ew.iter().zip(&reported).map(|(e, r)| e.0 - r.0).collect();
    let n = diffs.len() as f64;
    let dmean = diffs.iter().sum::<f64>() / n;
    let dvar = diffs.iter().map(|d| (d - dmean).powi(2)).sum::<f64>() / (n - 1.0);
    Row {
        class: Class::of(&cell.row, &cell.process, has_model),
        this: (cell.sigma_vg, cell.sigma_vg_err, cell.seeds, None),
        cell,
        record: record.row.clone(),
        part,
        refs,
        draws,
        channels: budget
            .first()
            .map(|s| s.by_channel.keys().cloned().collect())
            .unwrap_or_default(),
        ew_shift: (dmean / r_read.mean, (dvar / n).sqrt() / r_read.mean),
        heavy: e_read.spread > HEAVY_SPREAD_RATIO * r_read.spread,
    }
}

/// Clusters of rows that are not independent draws, as indices into `rows`.
fn clusters(rows: &[Row]) -> Vec<Vec<usize>> {
    let n = rows.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn root(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    for i in 0..n {
        for j in i + 1..n {
            if same_draw(&rows[i], &rows[j]) {
                let (a, b) = (root(&mut parent, i), root(&mut parent, j));
                parent[a] = b;
            }
        }
    }
    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for i in 0..n {
        let r = root(&mut parent, i);
        groups.entry(r).or_default().push(i);
    }
    groups.into_values().collect()
}

fn same_draw(a: &Row, b: &Row) -> bool {
    if a.record == b.record {
        return true;
    }
    let seeds_a: Vec<u64> = a.draws.iter().map(|d| d.0).collect();
    let seeds_b: Vec<u64> = b.draws.iter().map(|d| d.0).collect();
    if seeds_a != seeds_b || seeds_a.len() < 3 || a.channels.is_disjoint(&b.channels) {
        return false;
    }
    let rms = (a
        .draws
        .iter()
        .zip(&b.draws)
        .map(|(x, y)| (x.1 - y.1).powi(2))
        .sum::<f64>()
        / a.draws.len() as f64)
        .sqrt();
    rms < SAME_DRAW_RMS
}

/// A population of values with the clusters they fall into.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Population {
    n: usize,
    mean: f64,
    sd: f64,
    se: f64,
    clusters: usize,
    se_clustered: f64,
    median: f64,
    positive: usize,
}

/// `values[i]` belongs to the cluster `cluster_of[i]`. The clustered standard
/// error is `√(G/(G−1) · Σ_c (Σ_{i∈c} (xᵢ − x̄))²) / N` over `G` clusters.
fn population(values: &[f64], cluster_of: &[usize]) -> Option<Population> {
    let n = values.len();
    if n < 2 {
        return None;
    }
    let nf = n as f64;
    let mean = values.iter().sum::<f64>() / nf;
    let sd = (values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (nf - 1.0)).sqrt();
    let mut sums: BTreeMap<usize, f64> = BTreeMap::new();
    for (x, &c) in values.iter().zip(cluster_of) {
        *sums.entry(c).or_default() += x - mean;
    }
    let g = sums.len();
    let se_clustered = if g > 1 {
        let gf = g as f64;
        (gf / (gf - 1.0) * sums.values().map(|s| s * s).sum::<f64>()).sqrt() / nf
    } else {
        f64::NAN
    };
    Some(Population {
        n,
        mean,
        sd,
        se: sd / nf.sqrt(),
        clusters: g,
        se_clustered,
        median: median(values),
        positive: values.iter().filter(|&&x| x > 0.0).count(),
    })
}

fn population_line(label: &str, p: &Population) -> String {
    format!(
        "{label:<28} N {:>3} | mean {:+.3} | sd {:.3} | se {:.3} ({:+.1} se) | clustered se {:.3} \
         over {} clusters ({:+.1} se) | median {:+.3} | {} of {} positive",
        p.n,
        p.mean,
        p.sd,
        p.se,
        p.mean / p.se,
        p.se_clustered,
        p.clusters,
        p.mean / p.se_clustered,
        p.median,
        p.positive,
        p.n
    )
}

/// Unweighted and inverse-variance weighted means of relative offsets, and the
/// weighted mean's χ²/dof.
#[derive(Clone, Copy, Debug, PartialEq)]
struct RelativeMeans {
    unweighted: f64,
    unweighted_se: f64,
    weighted: f64,
    weighted_err: f64,
    chi2_per_dof: f64,
}

fn relative_means(rels: &[(f64, f64)]) -> Option<RelativeMeans> {
    let n = rels.len();
    if n < 2 {
        return None;
    }
    let nf = n as f64;
    let un = rels.iter().map(|r| r.0).sum::<f64>() / nf;
    let un_sd = (rels.iter().map(|r| (r.0 - un).powi(2)).sum::<f64>() / (nf - 1.0)).sqrt();
    let w: f64 = rels.iter().map(|r| 1.0 / (r.1 * r.1)).sum();
    let wm = rels.iter().map(|r| r.0 / (r.1 * r.1)).sum::<f64>() / w;
    let chi2 = rels.iter().map(|r| ((r.0 - wm) / r.1).powi(2)).sum::<f64>() / (nf - 1.0);
    Some(RelativeMeans {
        unweighted: un,
        unweighted_se: un_sd / nf.sqrt(),
        weighted: wm,
        weighted_err: w.recip().sqrt(),
        chi2_per_dof: chi2,
    })
}

/// Pearson correlation of two equal-length series.
fn correlation(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    let mx = x.iter().sum::<f64>() / n;
    let my = y.iter().sum::<f64>() / n;
    let sxy: f64 = x.iter().zip(y).map(|(a, b)| (a - mx) * (b - my)).sum();
    let sxx: f64 = x.iter().map(|a| (a - mx).powi(2)).sum();
    let syy: f64 = y.iter().map(|b| (b - my).powi(2)).sum();
    sxy / (sxx * syy).sqrt()
}

fn fmt_opt(v: Option<f64>) -> String {
    v.map_or_else(|| "   -  ".to_string(), |p| format!("{p:+6.2}"))
}

fn print_rows(rows: &[Row]) {
    println!(
        "{:<28} {:<14} {:>3} {:>6} | {:>6} {:>6} {:>6} {:>6} {:>6} | {:>8} {:>8} | {:>9} {:>6} | {:>5} | heavy",
        "cell", "class", "n", "cell", "R", "EW", "EWmed", "R4x", "EW4x", "rel R%", "err%", "EW-R %", "z", "MG var"
    );
    for r in rows {
        let (rel, rel_err) = r.rel(Reference::Reported).unwrap_or((f64::NAN, f64::NAN));
        println!(
            "{:<28} {:<14} {:>3} {:+6.2} | {} {} {} {} {} | {:+8.4} {:8.4} | {:+9.4} {:+6.1} | {:5.2} | {}",
            r.cell.name(),
            r.class.label(),
            r.this.2,
            r.cell.pull,
            fmt_opt(r.pull(Reference::Reported)),
            fmt_opt(r.pull(Reference::EqualWeight)),
            if r.heavy {
                fmt_opt(r.pull(Reference::EqualWeightMedian))
            } else {
                "   .  ".to_string()
            },
            fmt_opt(r.pull(Reference::Reported4x)),
            fmt_opt(r.pull(Reference::EqualWeight4x)),
            100.0 * rel,
            100.0 * rel_err,
            100.0 * r.ew_shift.0,
            r.ew_shift_z(),
            r.mg_variance_share(),
            if r.heavy { "HEAVY" } else { "" }
        );
    }
}

/// The references whose offsets from the this-side value are printed in
/// relative units and by class.
const RE_READS: [Reference; 4] = [
    Reference::Reported,
    Reference::EqualWeight,
    Reference::Reported4x,
    Reference::EqualWeight4x,
];

/// The population of `reference`'s pulls over the rows `keep` selects;
/// `cluster_of[i]` is row `i`'s cluster.
fn pull_population(
    rows: &[Row],
    cluster_of: &[usize],
    keep: &dyn Fn(&Row) -> bool,
    reference: Reference,
) -> Option<Population> {
    let (vals, cl): (Vec<f64>, Vec<usize>) = rows
        .iter()
        .zip(cluster_of)
        .filter(|(r, _)| keep(r))
        .filter_map(|(r, &c)| r.pull(reference).map(|p| (p, c)))
        .unzip();
    population(&vals, &cl)
}

/// One printed line of unweighted and weighted relative means, the label padded
/// to `width`.
fn relative_line(label: &str, width: usize, m: &RelativeMeans) -> String {
    format!(
        "{label:<width$} unweighted {:+.4}% +- {:.4}% | weighted {:+.4}% +- {:.4}%, chi2/dof {:.2}",
        100.0 * m.unweighted,
        100.0 * m.unweighted_se,
        100.0 * m.weighted,
        100.0 * m.weighted_err,
        m.chi2_per_dof
    )
}

fn print_pull_populations(rows: &[Row], cluster_of: &[usize]) {
    println!("-- pulls, every row --");
    for reference in Reference::ALL {
        if reference == Reference::EqualWeightMedian {
            let (mixed, cl): (Vec<f64>, Vec<usize>) = rows
                .iter()
                .zip(cluster_of)
                .filter_map(|(r, &c)| {
                    r.pull(if r.heavy {
                        Reference::EqualWeightMedian
                    } else {
                        Reference::EqualWeight
                    })
                    .map(|p| (p, c))
                })
                .unzip();
            if let Some(p) = population(&mixed, &cl) {
                println!("{}", population_line("EW, median on heavy rows", &p));
            }
            continue;
        }
        if let Some(p) = pull_population(rows, cluster_of, &|_| true, reference) {
            println!("{}", population_line(reference.label(), &p));
        }
    }
}

fn print_relative_offsets(rows: &[Row]) {
    println!("-- relative offsets sigma_vg/sigma_ref - 1 --");
    for reference in RE_READS {
        let rels: Vec<(f64, f64)> = rows.iter().filter_map(|r| r.rel(reference)).collect();
        if let Some(m) = relative_means(&rels) {
            println!("{}", relative_line(reference.label(), 5, &m));
        }
    }
}

fn print_reference_shifts(rows: &[Row]) {
    println!("-- the reference against itself, relative to R (no this-side value enters) --");
    let shift_of = |a: Reference, b: Reference| -> Vec<(f64, f64)> {
        rows.iter()
            .filter_map(|r| {
                let (x, y) = (r.refs.get(&a)?, r.refs.get(&b)?);
                Some((x.mean / y.mean - 1.0, x.err.hypot(y.err) / y.mean))
            })
            .collect()
    };
    let paired: Vec<(f64, f64)> = rows
        .iter()
        .map(|r| r.ew_shift)
        .filter(|s| s.1 > 0.0)
        .collect();
    for (label, shifts) in [
        ("EW - R, paired per seed", paired),
        (
            "R4x - R",
            shift_of(Reference::Reported4x, Reference::Reported),
        ),
        (
            "EW4x - EW",
            shift_of(Reference::EqualWeight4x, Reference::EqualWeight),
        ),
        (
            "EW4x - R",
            shift_of(Reference::EqualWeight4x, Reference::Reported),
        ),
    ] {
        if let Some(m) = relative_means(&shifts) {
            println!("{}", relative_line(label, 24, &m));
        }
    }
}

fn print_splits(rows: &[Row], cluster_of: &[usize]) {
    println!("-- by class --");
    for class in Class::ALL {
        for reference in RE_READS {
            let keep = |r: &Row| r.class == class;
            if let Some(p) = pull_population(rows, cluster_of, &keep, reference) {
                println!(
                    "{}",
                    population_line(&format!("{} {}", class.label(), reference.label()), &p)
                );
            }
        }
    }
    println!(
        "-- by the record's paired EW - R difference (resolved: |z| > {EW_SHIFT_RESOLVED}) --"
    );
    let resolved = |r: &Row| r.ew_shift_z().abs() > EW_SHIFT_RESOLVED;
    for (label, keep) in [
        ("EW-R resolved", &resolved as &dyn Fn(&Row) -> bool),
        ("EW-R not resolved", &|r: &Row| !resolved(r)),
    ] {
        for reference in [Reference::Reported, Reference::EqualWeight] {
            if let Some(p) = pull_population(rows, cluster_of, keep, reference) {
                println!(
                    "{}",
                    population_line(&format!("{label} {}", reference.label()), &p)
                );
            }
        }
    }
}

fn print_shift_correlation(rows: &[Row]) {
    let both: Vec<(f64, f64)> = rows
        .iter()
        .filter_map(|r| {
            let r_pull = r.pull(Reference::Reported)?;
            let r_ref = r.refs[&Reference::Reported];
            Some((
                r_pull,
                r.ew_shift.0 * r_ref.mean / r.this.1.hypot(r_ref.err),
            ))
        })
        .collect();
    let (p, shift): (Vec<f64>, Vec<f64>) = both.into_iter().unzip();
    println!(
        "correlation of the R pull with the EW - R shift in pull units: {:+.3} over {} rows",
        correlation(&p, &shift),
        p.len()
    );
}

fn print_population(title: &str, rows: &[Row]) {
    println!("\n== {title}: {} rows ==", rows.len());
    let groups = clusters(rows);
    let mut cluster_of = vec![0; rows.len()];
    for (c, g) in groups.iter().enumerate() {
        for &i in g {
            cluster_of[i] = c;
        }
    }
    for g in groups.iter().filter(|g| g.len() > 1) {
        println!(
            "cluster: {}",
            g.iter()
                .map(|&i| rows[i].cell.name())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    print_pull_populations(rows, &cluster_of);
    print_relative_offsets(rows);
    print_reference_shifts(rows);
    print_splits(rows, &cluster_of);
    print_shift_correlation(rows);
}

/// Every record under `<output>/seeded_sigma/`, by row.
fn read_records(output: &Path) -> Result<BTreeMap<String, SeedRecord>, String> {
    let dir = output.join(RECORD_DIR);
    let mut records = BTreeMap::new();
    let entries = std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in entries {
        let dir_path = entry.map_err(|e| e.to_string())?.path();
        let path = dir_path.join(RECORD_FILE);
        if !path.is_file() {
            continue;
        }
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let record: SeedRecord =
            serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        record
            .check()
            .map_err(|e| format!("{}: {e}", path.display()))?;
        if dir_path.file_name().and_then(|n| n.to_str()) != Some(record.row.as_str()) {
            return Err(format!(
                "{} records row {}, not the row its directory names",
                path.display(),
                record.row
            ));
        }
        records.insert(record.row.clone(), record);
    }
    Ok(records)
}

/// Every σ row file under `<report>/integrals/`.
fn read_cells(report: &Path) -> Result<Vec<Cell>, String> {
    let dir = report.join("integrals");
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let path = entry.map_err(|e| format!("{}: {e}", dir.display()))?.path();
        if path.extension().is_some_and(|x| x == "json") {
            paths.push(path);
        }
    }
    paths.sort();
    let mut cells = Vec::new();
    for path in paths {
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        cells.push(Cell::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?);
    }
    Ok(cells)
}

/// The manifest's process keys that name a UFO model of their own; the interned
/// Standard Model (`model = "sm"`, under whatever restriction) is not one.
fn rows_with_model(manifest: &Path) -> Result<Vec<String>, String> {
    let text =
        std::fs::read_to_string(manifest).map_err(|e| format!("{}: {e}", manifest.display()))?;
    let value: toml::Value = toml::from_str(&text).map_err(|e| e.to_string())?;
    Ok(value
        .get("process")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter(|p| {
            p.get("model")
                .and_then(toml::Value::as_str)
                .is_some_and(|m| m != "sm")
        })
        .filter_map(|p| p.get("key").and_then(toml::Value::as_str).map(String::from))
        .collect())
}

/// `record seed sigma_pb err_pb` lines, grouped by record.
fn parse_this_side(text: &str) -> Result<BTreeMap<String, Vec<(f64, f64)>>, String> {
    let mut out: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    for (k, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split_whitespace().collect();
        let [row, seed, value, err] = f[..] else {
            return Err(format!(
                "line {}: expected `record seed sigma_pb err_pb`",
                k + 1
            ));
        };
        let parse = |s: &str| {
            s.parse::<f64>()
                .map_err(|_| format!("line {}: {s:?} is not a number", k + 1))
        };
        if !seen.insert((row.to_string(), seed.to_string())) {
            return Err(format!(
                "line {}: seed {seed} of {row} appears twice",
                k + 1
            ));
        }
        out.entry(row.to_string())
            .or_default()
            .push((parse(value)?, parse(err)?));
    }
    Ok(out)
}

/// The rows whose record the sweep covers, with this side replaced by the
/// sweep's seed-policy mean. A sweep record that no row reads, or that holds
/// one seed, is an error: dropping it would shrink the swept population unseen.
fn with_this_side(
    rows: &[Row],
    sweep: &BTreeMap<String, Vec<(f64, f64)>>,
) -> Result<Vec<Row>, String> {
    for (record, seeds) in sweep {
        if !rows.iter().any(|r| &r.record == record) {
            return Err(format!("the sweep names {record}, which no row file reads"));
        }
        if seeds.len() < 2 {
            return Err(format!(
                "the sweep holds {} seed of {record}; the seed policy needs two",
                seeds.len()
            ));
        }
    }
    Ok(rows
        .iter()
        .filter_map(|r| {
            let seeds = sweep.get(&r.record).filter(|_| r.part == Part::Total)?;
            let read: SeedMean = combine(seeds);
            let mut row = r.clone();
            row.this = (read.mean, read.err, read.n, Some(read.chi2_per_dof));
            Some(row)
        })
        .collect())
}

struct Args {
    report: PathBuf,
    output: PathBuf,
    manifest: PathBuf,
    this_side: Option<PathBuf>,
}

fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut map: BTreeMap<&str, &str> = BTreeMap::new();
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        match flag.as_str() {
            "--report" | "--output" | "--manifest" | "--this-side" => {
                let value = it.next().ok_or_else(|| format!("{flag} needs a value"))?;
                map.insert(flag.as_str(), value.as_str());
            }
            other => return Err(format!("unknown argument {other:?}")),
        }
    }
    let get = |k: &str| {
        map.get(k)
            .map(PathBuf::from)
            .ok_or_else(|| format!("{k} is required"))
    };
    Ok(Args {
        report: get("--report")?,
        output: get("--output")?,
        manifest: get("--manifest")?,
        this_side: map.get("--this-side").map(PathBuf::from),
    })
}

/// Each cell joined to the record it was read against, and the cells no record
/// reproduces.
fn join_rows(
    records: &BTreeMap<String, SeedRecord>,
    cells: Vec<Cell>,
    with_model: &[String],
) -> Result<(Vec<Row>, Vec<Cell>), String> {
    let mut rows = Vec::new();
    let mut unmatched = Vec::new();
    for cell in cells {
        match find_record(records, cell.sigma_mg)? {
            Some((record, part)) => {
                let has_model = with_model.contains(&cell.row);
                rows.push(build_row(cell, record, part, has_model));
            }
            None => unmatched.push(cell),
        }
    }
    Ok((rows, unmatched))
}

/// Every row's reported re-read must give the pull its gate wrote. A NaN on
/// either side is a mismatch.
fn check_reproduction(rows: &[Row]) -> Result<(), String> {
    for r in rows {
        let gate = r.pull(Reference::Reported).unwrap_or(f64::NAN);
        let agrees = (gate - r.cell.pull).abs() <= 1e-9;
        if !agrees {
            return Err(format!(
                "{}: the gate's pull {} is not the reported re-read {gate}",
                r.cell.name(),
                r.cell.pull
            ));
        }
    }
    Ok(())
}

fn run(args: &Args) -> Result<(), String> {
    let records = read_records(&args.output)?;
    let cells = read_cells(&args.report)?;
    let with_model = rows_with_model(&args.manifest)?;
    let (rows, unmatched) = join_rows(&records, cells, &with_model)?;
    for cell in &unmatched {
        println!(
            "not read from a seed record: {} (MadEvent {:e})",
            cell.name(),
            cell.sigma_mg
        );
    }
    check_reproduction(&rows)?;
    let read: Vec<&str> = rows.iter().map(|r| r.record.as_str()).collect();
    let unread: Vec<&str> = records
        .keys()
        .map(String::as_str)
        .filter(|k| !read.contains(k))
        .collect();
    println!("records no row file read: {}", unread.join(", "));
    println!("\n== rows as the gates read them ==");
    print_rows(&rows);
    print_population("pulls as the gates measured them", &rows);
    if let Some(path) = &args.this_side {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let sweep = parse_this_side(&text)?;
        let swept = with_this_side(&rows, &sweep)?;
        println!("\n== rows with this side read over its seed sweep ==");
        print_rows(&swept);
        for r in &swept {
            println!(
                "{}: this side {} seeds, chi2/dof {:.2}",
                r.cell.name(),
                r.this.2,
                r.this.3.unwrap_or(f64::NAN)
            );
        }
        let names: Vec<&str> = swept.iter().map(|r| r.record.as_str()).collect();
        let same: Vec<Row> = rows
            .iter()
            .filter(|r| names.contains(&r.record.as_str()))
            .cloned()
            .collect();
        print_population("the swept rows, as the gates measured them", &same);
        print_population("the swept rows, this side over its sweep", &swept);
        let mixed: Vec<Row> = rows
            .iter()
            .map(|r| {
                swept
                    .iter()
                    .find(|s| s.record == r.record && s.part == r.part)
                    .unwrap_or(r)
                    .clone()
            })
            .collect();
        print_population("every row, swept rows over their sweep", &mixed);
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse_args(&args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("sigma-pulls: {e}");
            return ExitCode::from(2);
        }
    };
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("sigma-pulls: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use seed_record::{PartValues, SCHEMA};

    fn run_at(iseed: u64, nevents: u64, reported: (f64, f64), ew: f64, channel: &str) -> SeedRun {
        SeedRun {
            iseed,
            nevents,
            reported_pb: reported.0,
            reported_err_pb: reported.1,
            equal_weight_pb: ew,
            equal_weight_err_pb: reported.1,
            wall_s: 1.0,
            channels: 1,
            iterations_min: 3,
            iterations_max: 3,
            files: format!("s{iseed}"),
            windows: Vec::new(),
            by_lprup: BTreeMap::new(),
            hard_survey: 2,
            min_events: 10000,
            max_events: 30000,
            events_written: nevents,
            by_channel: [(
                channel.to_string(),
                seed_record::ChannelValues {
                    value_pb: reported.0,
                    err_pb: reported.1,
                    equal_weight_pb: ew,
                    iterations: 3,
                },
            )]
            .into(),
        }
    }

    /// Seeds at the budget with reported deviations `devs` and equal-weight
    /// deviations `ew_devs` (both in units of the quoted error 0.01 about 1.0),
    /// and two seeds at four times it, on integration channel `channel`.
    fn record_of(row: &str, devs: &[f64], ew_devs: &[f64], channel: &str) -> SeedRecord {
        let mut seeds: Vec<SeedRun> = devs
            .iter()
            .zip(ew_devs)
            .enumerate()
            .map(|(k, (d, e))| {
                run_at(
                    20261101 + k as u64,
                    10000,
                    (1.0 + 0.01 * d, 0.01),
                    1.0 + 0.01 * e,
                    channel,
                )
            })
            .collect();
        seeds.push(run_at(20261111, 40000, (1.0, 0.005), 1.0, channel));
        seeds.push(run_at(20261112, 40000, (1.002, 0.005), 1.002, channel));
        SeedRecord {
            schema: SCHEMA,
            row: row.to_string(),
            generator: "test".into(),
            mg_version: "3.7.1".into(),
            mg5_tree: "pinned".into(),
            budget_nevents: 10000,
            validated_ew: false,
            hard_survey: 2,
            seeds,
            summary: None,
        }
    }

    /// Five seeds at the budget with deviations `devs`, equal-weight values
    /// `ew_offset` above the reported ones.
    fn record(row: &str, devs: [f64; 5], ew_offset: f64) -> SeedRecord {
        let ew: Vec<f64> = devs.iter().map(|d| d + 100.0 * ew_offset).collect();
        record_of(row, &devs, &ew, "P1_ll_ll/G1")
    }

    fn cell(row: &str, sigma_vg: f64, err: f64, sigma_mg: f64, pull: f64) -> Cell {
        Cell {
            row: row.into(),
            variant: None,
            process: "e+ e- > mu+ mu-".into(),
            mode: "gate".into(),
            sigma_vg,
            sigma_vg_err: err,
            sigma_mg,
            pull,
            seeds: 1,
        }
    }

    /// The row a gate would have written for `record` with this side at
    /// `sigma_vg ± 0.01`: its own pull is the reported re-read.
    fn gate_row(record: &SeedRecord, sigma_vg: f64) -> Row {
        let m = gate_reading(record, &Part::Total).unwrap();
        let mut row = build_row(
            cell(&record.row, sigma_vg, 0.01, m, 0.0),
            record,
            Part::Total,
            false,
        );
        row.cell.pull = row.pull(Reference::Reported).unwrap();
        row
    }

    fn records(list: Vec<SeedRecord>) -> BTreeMap<String, SeedRecord> {
        list.into_iter().map(|r| (r.row.clone(), r)).collect()
    }

    #[test]
    fn a_row_file_joins_the_record_whose_reading_it_carries() {
        let a = record("a", [1.0, -1.0, 0.5, -0.5, 0.0], 0.0);
        let b = record("b", [0.2, -1.0, 0.5, -0.5, 0.3], 0.0);
        let mean_b = gate_reading(&b, &Part::Total).unwrap();
        let records = records(vec![a, b]);
        let (found, part) = find_record(&records, mean_b).unwrap().unwrap();
        assert_eq!(found.row, "b");
        assert_eq!(part, Part::Total);
        // A reading off by rounding still joins; one off by 1e-10 is another value.
        assert!(find_record(&records, mean_b * (1.0 + 1e-13))
            .unwrap()
            .is_some());
        assert!(find_record(&records, mean_b * (1.0 + 1e-10))
            .unwrap()
            .is_none());
    }

    #[test]
    fn a_reading_two_records_carry_is_an_error_not_the_first() {
        let devs = [1.0, -1.0, 0.5, -0.5, 0.0];
        let twin = records(vec![record("a", devs, 0.0), record("a2", devs, 0.0)]);
        let m = gate_reading(&twin["a"], &Part::Total).unwrap();
        let err = find_record(&twin, m).unwrap_err();
        assert!(
            err.contains("several records") && err.contains("a2"),
            "{err}"
        );
    }

    #[test]
    fn a_process_share_joins_by_its_own_reading() {
        let mut rec = record("a", [1.0, -1.0, 0.5, -0.5, 0.0], 0.0);
        for s in &mut rec.seeds {
            let share = |f: f64| PartValues {
                reported_pb: f * s.reported_pb,
                reported_err_pb: f * s.reported_err_pb,
                equal_weight_pb: f * s.equal_weight_pb,
                equal_weight_err_pb: f * s.equal_weight_err_pb,
            };
            s.by_lprup = [("1".to_string(), share(0.4)), ("2".to_string(), share(0.6))].into();
        }
        let total = gate_reading(&rec, &Part::Total).unwrap();
        let records = records(vec![rec]);
        let (_, part) = find_record(&records, 0.6 * total).unwrap().unwrap();
        assert_eq!(part, Part::Lprup("2".into()));
        let (_, part) = find_record(&records, total).unwrap().unwrap();
        assert_eq!(part, Part::Total);
        let (_, part) = find_record(&records, 0.6 * total).unwrap().unwrap();
        let row = build_row(
            cell("a", 0.6, 0.01, 0.6 * total, 0.0),
            &records["a"],
            part,
            false,
        );
        assert!((row.refs[&Reference::Reported].mean - 0.6 * total).abs() < 1e-12);
    }

    #[test]
    fn cells_join_their_records_and_the_rest_are_returned() {
        let rec = record("a", [1.0, -1.0, 0.5, -0.5, 0.0], 0.0);
        let m = gate_reading(&rec, &Part::Total).unwrap();
        let records = records(vec![rec]);
        let cells = vec![
            cell("a", 1.0, 0.01, m, 0.0),
            cell("control", 1.0, 0.01, 2.5, 0.0),
        ];
        let (rows, unmatched) = join_rows(&records, cells.clone(), &["a".to_string()]).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].record, "a");
        assert_eq!(rows[0].class, Class::Bsm);
        assert_eq!(
            unmatched.iter().map(|c| c.row.as_str()).collect::<Vec<_>>(),
            ["control"]
        );
        let (rows, _) = join_rows(&records, cells, &[]).unwrap();
        assert_eq!(rows[0].class, Class::FixedBeam);
    }

    #[test]
    fn the_gates_pull_must_be_the_reported_reread() {
        let rec = record("a", [1.0, -1.0, 0.5, -0.5, 0.0], 0.0);
        let mut row = gate_row(&rec, 1.01);
        assert!(check_reproduction(std::slice::from_ref(&row)).is_ok());
        let pull = row.cell.pull;
        row.cell.pull = pull + 1e-10;
        assert!(check_reproduction(std::slice::from_ref(&row)).is_ok());
        row.cell.pull = pull + 1e-6;
        let err = check_reproduction(std::slice::from_ref(&row)).unwrap_err();
        assert!(
            err.starts_with("a: the gate's pull") && err.contains("reported re-read"),
            "{err}"
        );
        row.cell.pull = f64::NAN;
        assert!(check_reproduction(&[row]).is_err());
    }

    #[test]
    fn the_reported_reread_reproduces_the_gates_pull_and_ew_moves_it() {
        let rec = record("a", [1.0, -1.0, 0.5, -0.5, 0.0], 0.002);
        let r = reference_value(&rec, &Part::Total, Reference::Reported).unwrap();
        // Mean of the deviations is zero; the quoted error 0.01 over five seeds
        // is 0.01/√5, above the seeds' spread 0.01·√(2.5/4)/√5.
        assert!((r.mean - 1.0).abs() < 1e-12);
        assert!((r.err - 0.01 / 5f64.sqrt()).abs() < 1e-12);
        let pull = (1.01 - r.mean) / 0.01f64.hypot(r.err);
        assert!((pull - 0.01 / 0.01f64.hypot(0.01 / 5f64.sqrt())).abs() < 1e-12);
        let row = build_row(
            cell("a", 1.01, 0.01, r.mean, pull),
            &rec,
            Part::Total,
            false,
        );
        assert!((row.pull(Reference::Reported).unwrap() - pull).abs() < 1e-12);
        let ew = row.refs[&Reference::EqualWeight];
        assert!((ew.mean - (r.mean + 0.002)).abs() < 1e-12);
        assert!(row.pull(Reference::EqualWeight).unwrap() < pull);
        assert!((row.ew_shift.0 - 0.002 / r.mean).abs() < 1e-12);
        let four = row.refs[&Reference::Reported4x];
        assert!((four.mean - 1.001).abs() < 1e-12);
        // Two seeds of quoted error 0.005: 0.005/√2 on the mean, above their
        // spread 0.002/√2/√2.
        assert!((four.err - 0.005 / 2f64.sqrt()).abs() < 1e-12);
        let ew_four = row.refs[&Reference::EqualWeight4x];
        assert!((ew_four.mean - 1.001).abs() < 1e-12);
        let (rel, rel_err) = row.rel(Reference::Reported).unwrap();
        assert!((rel - 0.01).abs() < 1e-12);
        assert!((rel_err - 0.01f64.hypot(r.err)).abs() < 1e-12);
    }

    #[test]
    fn a_rows_statistics_follow_its_seeds() {
        let devs = [1.0, -1.0, 0.5, -0.5, 0.0];
        // Equal-weight values 0.001..0.003 above the reported ones.
        let rec = record_of("a", &devs, &[1.1, -0.7, 0.7, -0.3, 0.2], "P1_ll_ll/G1");
        let row = gate_row(&rec, 1.01);
        // Differences 0.001, 0.003, 0.002, 0.002, 0.002: mean 0.002, sample
        // variance 5e-7, error √(5e-7/5) on the mean, all over the reported 1.0.
        assert!((row.ew_shift.0 - 0.002).abs() < 1e-12);
        assert!((row.ew_shift.1 - (5e-7f64 / 5.0).sqrt()).abs() < 1e-12);
        assert!((row.ew_shift_z() - 0.002 / (1e-7f64).sqrt()).abs() < 1e-9);
        // Draws are the reported deviations from their mean in quoted errors.
        for ((_, d), want) in row.draws.iter().zip(devs) {
            assert!((d - want).abs() < 1e-9);
        }
        let iseeds: Vec<u64> = row.draws.iter().map(|d| d.0).collect();
        assert_eq!(iseeds, (20261101..20261106).collect::<Vec<u64>>());
        // The reference's variance over the sum of both variances.
        assert!((row.mg_variance_share() - 1.0 / 6.0).abs() < 1e-12);
        assert!(!row.heavy);
        // Equal-weight values ten times as scattered are heavy-tailed.
        let wide: Vec<f64> = devs.iter().map(|d| 10.0 * d).collect();
        let heavy = gate_row(&record_of("h", &devs, &wide, "P1_ll_ll/G1"), 1.01);
        assert!(heavy.heavy);
    }

    #[test]
    fn the_median_reference_reads_the_middle_seed_with_the_median_error() {
        let devs = [1.0, -1.0, 0.5, -0.5, 3.0];
        let rec = record_of("m", &devs, &devs, "P1_ll_ll/G1");
        let row = gate_row(&rec, 1.01);
        let ew = row.refs[&Reference::EqualWeight];
        let med = row.refs[&Reference::EqualWeightMedian];
        assert!((ew.mean - 1.006).abs() < 1e-12);
        assert!((med.mean - 1.005).abs() < 1e-12);
        assert!((med.err / ew.err - (std::f64::consts::PI / 2.0).sqrt()).abs() < 1e-12);
        assert_eq!(median(&[3.0, 1.0, 2.0]), 2.0);
        assert_eq!(median(&[4.0, 1.0, 3.0, 2.0]), 2.5);
    }

    #[test]
    fn rows_drawing_the_same_fluctuations_share_a_cluster() {
        let devs = [1.3, -0.4, 0.2, -1.1, 0.0];
        let chan = "P1_ll_ll/G1";
        // Moving one seed by x moves the standardized deviations by 0.8x on it
        // and 0.2x on the other four: an RMS of 0.4x. "far" moves the opposite way,
        // so it is 0.5 from "a" and 0.7 from "near".
        let nudged = |x: f64| {
            let mut d = devs;
            d[0] += x;
            d
        };
        let build =
            |recs: Vec<SeedRecord>| -> Vec<Row> { recs.iter().map(|r| gate_row(r, 1.0)).collect() };
        let rows = build(vec![
            record_of("a", &devs, &devs, chan),
            record_of("near", &nudged(0.5), &nudged(0.5), chan),
            record_of("far", &nudged(-1.25), &nudged(-1.25), chan),
            record_of("other_channel", &devs, &devs, "P2_gg_gg/G3"),
            record_of("c", &[-0.9, 1.2, -0.3, 0.8, -0.8], &devs, chan),
        ]);
        let groups = clusters(&rows);
        assert_eq!(
            groups,
            vec![vec![0, 1], vec![2], vec![3], vec![4]],
            "RMS 0.2 joins, 0.5 and a disjoint channel do not"
        );
    }

    #[test]
    fn two_rows_of_one_record_share_a_cluster_even_with_two_seeds() {
        // Two seeds at the budget are too few to compare draws.
        let few = |row: &str| {
            let mut r = record_of(row, &[1.0, -1.0], &[1.0, -1.0], "P1_ll_ll/G1");
            r.seeds.retain(|s| s.nevents == 10000);
            r
        };
        let (a, b) = (few("a"), few("b"));
        let rows = [gate_row(&a, 1.0), gate_row(&a, 1.02), gate_row(&b, 1.0)];
        assert_eq!(clusters(&rows), vec![vec![0, 1], vec![2]]);
    }

    #[test]
    fn the_clustered_error_counts_a_duplicated_row_once() {
        let xs = [1.0, 1.0, -1.0, 0.5, -0.5];
        let independent = population(&xs, &[0, 1, 2, 3, 4]).unwrap();
        // Mean 0.2, squares about it sum to 3.3, sample variance 3.3/4.
        assert!((independent.mean - 0.2).abs() < 1e-15);
        assert!((independent.sd - 0.908_295_106_229_247_6).abs() < 1e-12);
        assert!((independent.se - independent.sd / 5f64.sqrt()).abs() < 1e-15);
        assert_eq!((independent.median, independent.positive), (0.5, 3));
        assert_eq!(population(&[0.0, 1.0], &[0, 1]).unwrap().positive, 1);
        let mean = 0.2;
        let sums: [f64; 4] = [2.0 * (1.0 - mean), -1.0 - mean, 0.5 - mean, -0.5 - mean];
        let expected = (4.0 / 3.0 * sums.iter().map(|s| s * s).sum::<f64>()).sqrt() / 5.0;
        let clustered = population(&xs, &[0, 0, 1, 2, 3]).unwrap();
        assert_eq!(clustered.clusters, 4);
        assert!((clustered.se_clustered - expected).abs() < 1e-15);
        assert!(clustered.se_clustered > independent.se_clustered);
        let one = population(&xs, &[0; 5]).unwrap();
        assert_eq!(one.clusters, 1);
        assert!(one.se_clustered.is_nan());
        assert!(population(&xs[..1], &[0]).is_none());
    }

    #[test]
    fn relative_means_weight_by_inverse_variance() {
        let m = relative_means(&[(0.01, 0.01), (0.0, 0.001)]).unwrap();
        assert!((m.unweighted - 0.005).abs() < 1e-15);
        // Sample sd of {0.01, 0} is 0.01/√2, over √2.
        assert!((m.unweighted_se - 0.005).abs() < 1e-15);
        let w = (0.01 / 1e-4 + 0.0) / (1e4 + 1e6);
        assert!((m.weighted - w).abs() < 1e-15);
        assert!((m.weighted_err - (1e4f64 + 1e6).recip().sqrt()).abs() < 1e-15);
        // χ² = ((0.01 − w)/0.01)² + (w/0.001)² over one degree of freedom.
        assert!((m.chi2_per_dof - 0.990_099_009_900_990_1).abs() < 1e-12);
        assert!(relative_means(&[(0.01, 0.01)]).is_none());
    }

    #[test]
    fn the_correlation_is_the_normalised_one() {
        let x = [1.0, 2.0, 3.0, 4.0];
        assert!((correlation(&x, &[2.0, 4.0, 6.0, 8.0]) - 1.0).abs() < 1e-15);
        assert!((correlation(&x, &[4.0, 3.0, 2.0, 1.0]) + 1.0).abs() < 1e-15);
    }

    #[test]
    fn classes_follow_beam_restriction_and_model() {
        assert_eq!(
            Class::of("pp_to_llj_mlm", "p p > e+ e- j", false),
            Class::Mlm
        );
        assert_eq!(Class::of("pp_to_jj_xqcut", "p p > j j", false), Class::Mlm);
        assert_eq!(Class::of("pp_to_bb", "p p > b b~", false), Class::Hadronic);
        assert_eq!(
            Class::of("ee_to_mumu_zonly", "e+ e- > z > mu+ mu-", false),
            Class::Grammar
        );
        assert_eq!(
            Class::of("ee_to_ee_nsz", "e+ e- > e+ e- $$ z", false),
            Class::Grammar
        );
        assert_eq!(
            Class::of("ee_to_wp0wm", "e+ e- > w+{0} w-", false),
            Class::Grammar
        );
        assert_eq!(
            Class::of("ee_to_ttx_smeft", "e+ e- > t t~ NP<=1", true),
            Class::Bsm
        );
        assert_eq!(
            Class::of("ee_to_ttx", "e+ e- > t t~", false),
            Class::FixedBeam
        );
    }

    #[test]
    fn the_this_side_file_groups_seeds_by_record() {
        let text = "# sweep\nee_to_ttx 1 0.549 0.0004\n\nee_to_ttx 11 0.548 0.0004\nuux_to_mumu 1 0.0768 6e-5\n";
        let sweep = parse_this_side(text).unwrap();
        assert_eq!(sweep["ee_to_ttx"].len(), 2);
        assert_eq!(sweep["uux_to_mumu"], vec![(0.0768, 6e-5)]);
    }

    #[test]
    fn a_malformed_this_side_file_is_refused_by_cause() {
        let err = parse_this_side("ee_to_ttx 1 0.5\n").unwrap_err();
        assert!(
            err.starts_with("line 1: expected `record seed sigma_pb err_pb`"),
            "{err}"
        );
        let err = parse_this_side("a 1 0.5 0.1\na 2 0.x 0.1\n").unwrap_err();
        assert!(err.starts_with("line 2: \"0.x\" is not a number"), "{err}");
        let err = parse_this_side("a 1 0.5 0.1\na 1 0.6 0.1\n").unwrap_err();
        assert!(
            err.contains("line 2") && err.contains("appears twice"),
            "{err}"
        );
    }

    #[test]
    fn this_side_over_a_sweep_is_the_seed_policy_mean() {
        let rec = record("a", [1.0, -1.0, 0.5, -0.5, 0.0], 0.0);
        let other = record("b", [0.2, -1.0, 0.5, -0.5, 0.3], 0.0);
        let rows = [gate_row(&rec, 1.01), gate_row(&other, 1.0)];
        let sweep = parse_this_side("a 1 1.0 0.01\na 2 1.02 0.01\na 3 1.04 0.01\n").unwrap();
        let swept = with_this_side(&rows, &sweep).unwrap();
        assert_eq!(swept.len(), 1);
        let (mean, err, n, chi2) = swept[0].this;
        assert!((mean - 1.02).abs() < 1e-12);
        // Quoted 0.01/√3; spread 0.02/√3 is larger and is the error read.
        assert!((err - 0.02 / 3f64.sqrt()).abs() < 1e-12);
        assert_eq!(n, 3);
        // Deviations ∓0.02 and 0 in quoted errors 0.01, over two degrees of freedom.
        assert!((chi2.unwrap() - 4.0).abs() < 1e-12);
        let unknown = parse_this_side("zz 1 1.0 0.01\nzz 2 1.0 0.01\n").unwrap();
        let err = with_this_side(&rows, &unknown).unwrap_err();
        assert!(err.contains("names zz"), "{err}");
        let single = parse_this_side("a 1 1.0 0.01\n").unwrap();
        let err = with_this_side(&rows, &single).unwrap_err();
        assert!(err.contains("holds 1 seed of a"), "{err}");
    }

    fn row_file(dir: &Path, name: &str, body: &str) {
        std::fs::write(dir.join(name), body).unwrap();
    }

    const CELL: &str = r#"{"row": "r", "variant": null, "process": "e+ e- > mu+ mu-", "mode": "gate",
        "sigma_vg_pb": 1.0, "sigma_vg_err_pb": 0.1, "sigma_mg_pb": 2.0, "pull": -0.5,
        "per_seed": [{}, {}]}"#;

    #[test]
    fn a_row_file_with_a_missing_or_null_number_is_an_error_naming_it() {
        let cell = Cell::parse(CELL).unwrap();
        assert_eq!((cell.sigma_mg, cell.pull, cell.seeds), (2.0, -0.5, 2));
        for field in ["sigma_vg_pb", "sigma_vg_err_pb", "sigma_mg_pb", "pull"] {
            let missing = CELL.replace(&format!("\"{field}\""), "\"renamed\"");
            let err = Cell::parse(&missing).unwrap_err();
            assert!(err.contains(&format!("\"{field}\"")), "{err}");
        }
        let nan = CELL.replace("\"pull\": -0.5", "\"pull\": null");
        assert!(Cell::parse(&nan).unwrap_err().contains("\"pull\""));
        assert!(Cell::parse(&CELL.replace("\"row\": \"r\",", ""))
            .unwrap_err()
            .contains("no row"));
    }

    #[test]
    fn the_row_files_of_a_report_are_all_read_or_the_read_fails() {
        let report = tempfile::tempdir().unwrap();
        let integrals = report.path().join("integrals");
        std::fs::create_dir(&integrals).unwrap();
        row_file(&integrals, "r.json", CELL);
        row_file(&integrals, "notes.txt", "not a row file");
        let cells = read_cells(report.path()).unwrap();
        assert_eq!(cells.len(), 1);
        row_file(&integrals, "bad.json", &CELL.replace("\"pull\"", "\"p\""));
        let err = read_cells(report.path()).unwrap_err();
        assert!(
            err.contains("bad.json") && err.contains("\"pull\""),
            "{err}"
        );
        let empty = tempfile::tempdir().unwrap();
        assert!(read_cells(empty.path()).unwrap_err().contains("integrals"));
    }

    #[test]
    fn a_record_must_sit_in_the_directory_of_its_row() {
        let output = tempfile::tempdir().unwrap();
        let write = |dir: &str, rec: &SeedRecord| {
            let d = output.path().join(RECORD_DIR).join(dir);
            std::fs::create_dir_all(&d).unwrap();
            std::fs::write(d.join(RECORD_FILE), serde_json::to_string(rec).unwrap()).unwrap();
        };
        let rec = record("a", [1.0, -1.0, 0.5, -0.5, 0.0], 0.0);
        write("a", &rec);
        std::fs::create_dir_all(output.path().join(RECORD_DIR).join("no_record")).unwrap();
        let read = read_records(output.path()).unwrap();
        assert_eq!(read.keys().collect::<Vec<_>>(), ["a"]);
        write("b", &rec);
        let err = read_records(output.path()).unwrap_err();
        assert!(err.contains("records row a, not the row"), "{err}");
    }
}
