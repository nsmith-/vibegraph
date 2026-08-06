//! Per-row JSON the validation report collator reads.
//!
//! Every gate that measures a per-process cell writes one file per measurement,
//! so the report is assembled from what the gates actually observed rather than
//! re-derived by a second driver. Files land at
//!
//! ```text
//! <target>/validation-report/<category>/<id>.json
//! ```
//!
//! where `<id>` is the row key, suffixed `__<variant>` when one row is measured
//! more than once (two run cards over the same process, say). The collator groups
//! by `row`.
//!
//! # Schema (`schema = 1`)
//!
//! | field | meaning |
//! |---|---|
//! | `schema` | this schema's version; bumped when a field's meaning changes |
//! | `row` | the process key of `validation/manifest.toml`'s `[[process]]` entry |
//! | `variant` | which measurement of that row this is; `null` when the row has one |
//! | `category` | the manifest category the cell belongs to (`"integrals"`) |
//! | `mode` | `"gate"` (a failure fails the suite) or `"info"` (measured, never enforced) |
//! | `status` | `"pass"`, `"fail"` or `"info"` — what this run observed |
//! | `process` | the process string the row measures |
//! | `note` | free text a reader needs to interpret the cell; `null` when none |
//! | `duration_s` | wall-clock seconds the measurement behind this cell took; `null` when it was not timed |
//!
//! Those eight fields are common to every category, and are what the collator
//! needs to place a cell and mark it. The measurement fields beneath them differ
//! per category; the `integrals` ones are below, and `diagrams`, `amplitudes`
//! and `samples` are documented on their own structs.
//!
//! | `integrals` field | meaning |
//! |---|---|
//! | `sigma_vg_pb` / `sigma_vg_err_pb` | this crate's cross section and its Monte-Carlo error |
//! | `sigma_mg_pb` / `sigma_mg_err_pb` | the banked MadGraph value and its error |
//! | `pull` | `(sigma_vg − sigma_mg) / sqrt(err_vg² + err_mg²)`, signed |
//! | `rel` | `sigma_vg / sigma_mg − 1`, signed |
//! | `chi2_dof` | over a seed sweep, the scatter of the seeds about their own mean in units of their quoted errors; otherwise the integration's own χ²/dof |
//! | `seeds` | the RNG seeds the measurement was taken on |
//! | `per_seed` | one `{seed, sigma_pb, sigma_err_pb}` per seed, in sweep order |
//! | `neval` / `niter` | the VEGAS budget per seed |
//! | `subsampler` | per sampling channel, what the rule-based composition chose |
//!
//! `pull` and `rel` are signed here even where the gate asserts on their
//! magnitude: a table of one-sided numbers hides whether a family of rows leans
//! the same way.
//!
//! # What `duration_s` is, and is not
//!
//! It is the wall time of one gate's own measurement of one row — the span a
//! [`Stopwatch`] covers, which starts where that row's work starts and is read at
//! `write()`. It is not a benchmark. `cargo test` runs a binary's tests in
//! parallel threads and the integrators fan out over rayon underneath them, so
//! rows measured concurrently each charge themselves the contended wall time and
//! their durations overlap; they do not sum to the suite's. Read the field as
//! the per-stage shape of a suite run on the machine `host.json` names, and take
//! a per-row cost from a run of that row alone.
//!
//! # The host record
//!
//! Timings without machine identity are noise, so the first row written by any
//! gate process also writes `<target>/validation-report/host.json`: the CPU, its
//! core classes, memory, OS, toolchain and the build settings of the test binary
//! doing the writing. One block per run rather than one per row — the rows
//! accompany it in the same directory. It stays out of the reference bundle: it
//! is a measurement about a machine, not a reference.

#![allow(dead_code)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::Once;
use std::time::Instant;

use serde::Serialize;
use serde_json::{json, Value};
use vibegraph::artifact::ChannelSampler;

/// The schema version the files below are written under.
pub const SCHEMA: u32 = 1;

/// The wall clock a row's `duration_s` is read off.
///
/// Started where a row's measurement starts, so the span it covers is the work
/// the cell rests on and not the gate's setup around it.
pub struct Stopwatch(Instant);

impl Default for Stopwatch {
    fn default() -> Self {
        Stopwatch::start()
    }
}

impl Stopwatch {
    pub fn start() -> Self {
        Stopwatch(Instant::now())
    }

    /// Elapsed seconds, as the row files record them.
    pub fn seconds(&self) -> f64 {
        self.0.elapsed().as_secs_f64()
    }
}

/// One measured `diagrams` cell: how many diagrams we enumerate against how many
/// MadGraph does, counted MadGraph's way (one representative per subprocess
/// class).
#[derive(Debug, Clone, Serialize)]
pub struct DiagramsRow {
    pub schema: u32,
    pub row: String,
    pub variant: Option<String>,
    pub category: &'static str,
    pub mode: &'static str,
    pub status: &'static str,
    pub process: String,
    /// Ours, grouped the way MadGraph groups: the `k` of `k/n`.
    pub ours: u32,
    /// MadGraph's own count from the committed reference: the `n`.
    pub theirs: u32,
    /// Every diagram of every concrete subprocess, ungrouped — larger than
    /// `ours` exactly where the process is a flavour group.
    pub ours_all_subprocesses: u32,
    pub note: Option<String>,
    /// Wall-clock seconds this row's own measurement took; `None` where the gate
    /// wrote the row without timing it.
    pub duration_s: Option<f64>,
}

impl DiagramsRow {
    pub fn new(row: &str, process: &str, mode: &'static str) -> Self {
        DiagramsRow {
            schema: SCHEMA,
            row: row.to_string(),
            variant: None,
            category: "diagrams",
            mode,
            status: if mode == "gate" { "pass" } else { "info" },
            process: process.to_string(),
            ours: 0,
            theirs: 0,
            ours_all_subprocesses: 0,
            note: None,
            duration_s: None,
        }
    }

    pub fn write(&self) {
        write_row(self.category, &self.row, self.variant.as_deref(), self);
    }
}

/// One measured `amplitudes` cell.
///
/// The metric a table renders is the larger of `max_rel_grid` and
/// `max_rel_event` — the worst `|M|²` deviation over both point sets. The
/// element-wise fields beneath it are the finer comparison the cell actually
/// rests on: per-diagram `AMP()` and per-flow `JAMP()` per helicity, each judged
/// after one fitted global phase.
#[derive(Debug, Clone, Serialize)]
pub struct AmplitudesRow {
    pub schema: u32,
    pub row: String,
    pub variant: Option<String>,
    pub category: &'static str,
    pub mode: &'static str,
    pub status: &'static str,
    pub process: String,
    pub n_graphs: usize,
    pub n_flows: usize,
    pub points_grid: usize,
    pub points_event: usize,
    pub max_rel_grid: f64,
    pub max_rel_event: f64,
    /// Largest element-wise deviation of the per-diagram `AMP()` comparison,
    /// relative to the largest MadGraph term. `null` where the table banks no
    /// per-diagram detail (the two 2→6 rows).
    pub per_diagram: Option<f64>,
    /// The same for the per-flow `JAMP()` comparison, which every row banks.
    pub per_flow: f64,
    /// `eval_jamp2` against Σ_hel |MadGraph JAMP|², the weight the colour-flow
    /// draw uses.
    pub jamp2: f64,
    /// The number of integration configurations, checked against the `AMP2()`
    /// accumulators MadGraph's own `matrix1.f` writes.
    pub n_configs: usize,
    /// Largest element-wise deviation of a configuration amplitude from
    /// MadGraph's `AMP()` under that configuration's own unit phase.
    pub per_config: f64,
    /// `eval_amp2` against Σ_hel |MadGraph AMP|² per configuration, the weight
    /// the configuration draw uses. `null` where MadGraph's own export merges
    /// configurations and the two groupings do not align.
    pub amp2: Option<f64>,
    /// How far helicity pruning moves `AMP2`, relative to the largest
    /// configuration at the point. Unlike |M|² this is not protected by the
    /// pruning threshold, so it is measured every run.
    pub amp2_pruned: f64,
    /// Whether the per-helicity × per-flow comparison had to be weakened to its
    /// two projections. The manifest states this per row where the question was
    /// asked; this is the measurement that claim is checked against.
    pub factorized: bool,
    pub note: Option<String>,
    /// Wall-clock seconds this row's own measurement took; `None` where the gate
    /// wrote the row without timing it.
    pub duration_s: Option<f64>,
}

impl AmplitudesRow {
    pub fn new(row: &str, process: &str, mode: &'static str) -> Self {
        AmplitudesRow {
            schema: SCHEMA,
            row: row.to_string(),
            variant: None,
            category: "amplitudes",
            mode,
            status: if mode == "gate" { "pass" } else { "info" },
            process: process.to_string(),
            n_graphs: 0,
            n_flows: 0,
            points_grid: 0,
            points_event: 0,
            max_rel_grid: 0.0,
            max_rel_event: 0.0,
            per_diagram: None,
            per_flow: 0.0,
            jamp2: 0.0,
            n_configs: 0,
            per_config: 0.0,
            amp2: None,
            amp2_pruned: 0.0,
            factorized: false,
            note: None,
            duration_s: None,
        }
    }

    pub fn write(&self) {
        write_row(self.category, &self.row, self.variant.as_deref(), self);
    }
}

/// One seed's own estimate inside a sweep.
#[derive(Debug, Clone, Serialize)]
pub struct SeedResult {
    pub seed: u64,
    pub sigma_pb: f64,
    pub sigma_err_pb: f64,
}

/// One sampling channel's composition, labelled by which channel it is.
#[derive(Debug, Clone, Serialize)]
pub struct ChannelSummary {
    /// How the channel is identified in its own decomposition — `"diagram 3"`,
    /// `"group 1 diagram 0"`.
    pub channel: String,
    pub sampler: ChannelSampler,
}

/// One measured `integrals` cell.
#[derive(Debug, Clone, Serialize)]
pub struct IntegralsRow {
    pub schema: u32,
    pub row: String,
    pub variant: Option<String>,
    pub category: &'static str,
    pub mode: &'static str,
    pub status: &'static str,
    pub process: String,
    pub sigma_vg_pb: f64,
    pub sigma_vg_err_pb: f64,
    pub sigma_mg_pb: f64,
    pub sigma_mg_err_pb: f64,
    pub pull: f64,
    pub rel: f64,
    pub chi2_dof: f64,
    pub seeds: Vec<u64>,
    pub per_seed: Vec<SeedResult>,
    pub neval: usize,
    pub niter: usize,
    pub subsampler: Vec<ChannelSummary>,
    pub note: Option<String>,
    /// Wall-clock seconds this row's own measurement took; `None` where the gate
    /// wrote the row without timing it.
    pub duration_s: Option<f64>,
}

impl IntegralsRow {
    /// A row with the identity fields set and every measurement left at zero,
    /// so a caller fills in what it measured and nothing silently defaults to a
    /// plausible-looking number.
    pub fn new(row: &str, process: &str, mode: &'static str) -> Self {
        IntegralsRow {
            schema: SCHEMA,
            row: row.to_string(),
            variant: None,
            category: "integrals",
            mode,
            status: if mode == "gate" { "pass" } else { "info" },
            process: process.to_string(),
            sigma_vg_pb: 0.0,
            sigma_vg_err_pb: 0.0,
            sigma_mg_pb: 0.0,
            sigma_mg_err_pb: 0.0,
            pull: 0.0,
            rel: 0.0,
            chi2_dof: 0.0,
            seeds: Vec::new(),
            per_seed: Vec::new(),
            neval: 0,
            niter: 0,
            subsampler: Vec::new(),
            note: None,
            duration_s: None,
        }
    }

    pub fn with_variant(mut self, variant: &str) -> Self {
        self.variant = Some(variant.to_string());
        self
    }

    /// Write the row under its category directory. The file name is the row key
    /// plus the variant, so two measurements of one row do not overwrite each
    /// other and a re-run overwrites its own file.
    pub fn write(&self) {
        write_row(self.category, &self.row, self.variant.as_deref(), self);
    }
}

/// One observable's Kolmogorov–Smirnov comparison.
#[derive(Debug, Clone, Serialize)]
pub struct KsCell {
    pub observable: String,
    /// The largest gap between the two weighted empirical CDFs.
    pub d: f64,
    pub p: f64,
}

/// One categorical column's χ² homogeneity comparison.
#[derive(Debug, Clone, Serialize)]
pub struct Chi2Cell {
    /// `SPINUP`, `ICOLUP` or `flavour`.
    pub column: String,
    pub chi2: f64,
    pub dof: usize,
    pub p: f64,
    /// Categories that carried their own χ² term, including the pooled residual.
    pub categories: usize,
    /// Distinct keys the two samples showed between them, before pooling — the
    /// difference from `categories` is how much of the column was too sparse to
    /// compare on its own.
    pub distinct_keys: usize,
    /// The share of the combined counts that landed in the pooled residual.
    pub pooled_share: f64,
    /// The two samples' effective counts per category, for a column with few
    /// enough of them to read. A χ² is one number and says only *that* two
    /// frequency tables differ; this is what says which category moved, and it is
    /// the whole evidence an informational cell carries.
    pub detail: Vec<CategoryCount>,
}

/// One category's effective count on each side.
#[derive(Debug, Clone, Serialize)]
pub struct CategoryCount {
    pub key: String,
    pub ours: f64,
    pub theirs: f64,
}

/// One generation seed's comparison against the banked sample.
#[derive(Debug, Clone, Serialize)]
pub struct SeedSample {
    pub seed: u64,
    pub events: usize,
    /// The cross section this seed's accept/reject pass recovered, in picobarns.
    pub sigma_pb: f64,
    pub ks: Vec<KsCell>,
    pub chi2: Vec<Chi2Cell>,
}

/// One measured `samples` cell.
///
/// The metric a table renders is `min_ks_p` (and `min_chi2_p` for the discrete
/// columns) — the smallest p-value over every observable and every generation
/// seed. It is a *minimum over many draws*, so it is not itself a p-value, and
/// `p_floor` records the threshold that was chosen for the number of draws taken.
#[derive(Debug, Clone, Serialize)]
pub struct SamplesRow {
    pub schema: u32,
    pub row: String,
    pub variant: Option<String>,
    pub category: &'static str,
    pub mode: &'static str,
    pub status: &'static str,
    pub process: String,
    /// `fine` when every event of both samples carries one final-state species
    /// multiset, `coarse` when the row is a flavour group and its legs are named
    /// by class.
    pub labelling: &'static str,
    /// Banked MadGraph events compared against, and the cross section they carry.
    pub mg_events: usize,
    pub sigma_mg_pb: f64,
    pub p_floor: f64,
    pub min_ks_p: f64,
    pub min_chi2_p: f64,
    /// The observable and column the minima came from.
    pub worst_ks_observable: String,
    pub worst_chi2_column: String,
    /// Observables that are constants of the process and so have no distribution
    /// to compare — named rather than silently dropped.
    pub constant_observables: Vec<String>,
    /// Categorical columns with a single category, where a homogeneity test has
    /// no degrees of freedom (a colourless process has one colour flow).
    pub single_category: Vec<String>,
    pub per_seed: Vec<SeedSample>,
    pub note: Option<String>,
    /// Wall-clock seconds this row's own measurement took; `None` where the gate
    /// wrote the row without timing it.
    pub duration_s: Option<f64>,
}

impl SamplesRow {
    pub fn new(row: &str, process: &str, mode: &'static str) -> Self {
        SamplesRow {
            schema: SCHEMA,
            row: row.to_string(),
            variant: None,
            category: "samples",
            mode,
            status: if mode == "gate" { "pass" } else { "info" },
            process: process.to_string(),
            labelling: "fine",
            mg_events: 0,
            sigma_mg_pb: 0.0,
            p_floor: 0.0,
            min_ks_p: 1.0,
            min_chi2_p: 1.0,
            worst_ks_observable: String::new(),
            worst_chi2_column: String::new(),
            constant_observables: Vec::new(),
            single_category: Vec::new(),
            per_seed: Vec::new(),
            note: None,
            duration_s: None,
        }
    }

    pub fn with_variant(mut self, variant: &str) -> Self {
        self.variant = Some(variant.to_string());
        self
    }

    /// Reduce the per-seed cells to the row's metric: the smallest p-value over
    /// every observable and every seed, and which column it came from.
    pub fn finish(&mut self) {
        let ks = self
            .per_seed
            .iter()
            .flat_map(|s| &s.ks)
            .min_by(|a, b| a.p.total_cmp(&b.p));
        if let Some(cell) = ks {
            self.min_ks_p = cell.p;
            self.worst_ks_observable = cell.observable.clone();
        }
        let chi2 = self
            .per_seed
            .iter()
            .flat_map(|s| &s.chi2)
            .min_by(|a, b| a.p.total_cmp(&b.p));
        if let Some(cell) = chi2 {
            self.min_chi2_p = cell.p;
            self.worst_chi2_column = cell.column.clone();
        }
        self.single_category.sort();
        self.single_category.dedup();
    }

    pub fn write(&self) {
        write_row(self.category, &self.row, self.variant.as_deref(), self);
    }
}

/// One `<category>/<row>[__<variant>].json` under the report directory.
fn write_row(category: &str, row: &str, variant: Option<&str>, value: &impl Serialize) {
    ensure_host_record();
    let dir = report_dir().join(category);
    std::fs::create_dir_all(&dir)
        .unwrap_or_else(|e| panic!("cannot create the report directory {}: {e}", dir.display()));
    let name = match variant {
        Some(v) => format!("{row}__{v}.json"),
        None => format!("{row}.json"),
    };
    let path = dir.join(name);
    let text = serde_json::to_string_pretty(value).expect("row serialises");
    std::fs::write(&path, text + "\n")
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
    eprintln!("[report] wrote {}", path.display());
}

/// The file name of the per-run machine-identity block the durations are read
/// against.
pub const HOST_RECORD: &str = "host.json";

/// Write `host.json` once per gate process, the first time that process records
/// a measurement.
///
/// Every gate binary of a run writes the same content, so the last writer wins
/// and the block describes the run. The write goes through a temporary file and
/// a rename so two gate processes racing on it cannot leave a half-written
/// record behind.
fn ensure_host_record() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let dir = report_dir();
        if std::fs::create_dir_all(&dir).is_err() {
            return;
        }
        let text = match serde_json::to_string_pretty(&host_record()) {
            Ok(t) => t + "\n",
            Err(_) => return,
        };
        let tmp = dir.join(format!("{HOST_RECORD}.{}", std::process::id()));
        if std::fs::write(&tmp, text).is_ok() {
            let _ = std::fs::rename(&tmp, dir.join(HOST_RECORD));
        }
    });
}

/// What a duration measured on this machine has to be read against: the CPU and
/// its core classes, memory, OS, the toolchain, and the build settings of the
/// binary taking the measurement.
///
/// Fields the OS does not expose are `null` rather than guessed — Apple Silicon
/// publishes no clock frequency through `sysctl`, and a plausible number in that
/// slot would be indistinguishable from a measured one.
fn host_record() -> Value {
    let exe = std::env::current_exe().ok();
    // `target/<profile>/deps/<binary>`: the profile the running gate was built
    // under, taken from where Cargo put it rather than from what a caller claims.
    let profile = exe.as_ref().and_then(|p| {
        p.parent()
            .filter(|d| d.file_name().is_some_and(|n| n == "deps"))
            .and_then(|d| d.parent())
            .and_then(|d| d.file_name())
            .map(|n| n.to_string_lossy().into_owned())
    });
    json!({
        "schema": 1,
        "captured": run_out("date", &["-u", "+%Y-%m-%dT%H:%M:%SZ"]),
        "cpu": {
            "arch": std::env::consts::ARCH,
            "model": cpu_model(),
            "logical_cpus": count("hw.logicalcpu", "processor"),
            "physical_cpus": count("hw.physicalcpu", "core id"),
            "performance_logical_cpus": sysctl_u64("hw.perflevel0.logicalcpu"),
            "efficiency_logical_cpus": sysctl_u64("hw.perflevel1.logicalcpu"),
            "frequency_hz": sysctl_u64("hw.cpufrequency_max"),
            "frequency_note": "null where the OS exposes no clock (Apple Silicon: \
                               `hw.cpufrequency`/`hw.cpufrequency_max` are empty)",
            "memory_bytes": sysctl_u64("hw.memsize").or_else(meminfo_bytes),
        },
        "scheduling": {
            "available_parallelism": std::thread::available_parallelism().map(|n| n.get()).ok(),
            "rust_test_threads": std::env::var("RUST_TEST_THREADS").ok(),
            "rayon_num_threads": std::env::var("RAYON_NUM_THREADS").ok(),
            "affinity": "none — no run pins itself to a core class, so on a hybrid CPU \
                         the scheduler is free to place work on either",
        },
        "os": {
            "family": std::env::consts::OS,
            "kernel": run_out("uname", &["-sr"]),
            "release": run_out("sw_vers", &["-productVersion"]).or_else(|| run_out("uname", &["-v"])),
        },
        "toolchain": {
            "rustc": run_out("rustc", &["--version"]),
            "cargo": run_out("cargo", &["--version"]),
            "rustup_toolchain": std::env::var("RUSTUP_TOOLCHAIN").ok(),
            "note": "the toolchain on PATH at measurement time; the gate binary was built \
                     by whatever `cargo test` resolved, which is the same one unless the \
                     default moved between the build and the run",
        },
        "build": {
            "profile": profile,
            // `run_out` joins a multi-line output with "; ", which is what
            // `rustc -vV` produces and what the host line has to be picked out of.
            "target": run_out("rustc", &["-vV"]).and_then(|v| {
                v.split("; ")
                    .find_map(|f| f.strip_prefix("host: "))
                    .map(str::to_string)
            }),
            "debug_assertions": cfg!(debug_assertions),
            "rustflags": std::env::var("RUSTFLAGS").ok(),
            "cargo_encoded_rustflags": std::env::var("CARGO_ENCODED_RUSTFLAGS").ok(),
            "note": "profile settings themselves live in the workspace `Cargo.toml`; \
                     `release-debug` is thin-LTO release codegen with debug info",
        },
    })
}

/// The trimmed first line of a command's standard output, or `None` if it did
/// not run or said nothing. A host block records what it could read.
fn run_out(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let joined: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    (!joined.is_empty()).then(|| joined.join("; "))
}

fn sysctl_u64(key: &str) -> Option<u64> {
    run_out("sysctl", &["-n", key])?.parse().ok()
}

fn cpu_model() -> Option<String> {
    sysctl_str("machdep.cpu.brand_string").or_else(|| {
        std::fs::read_to_string("/proc/cpuinfo").ok().and_then(|t| {
            t.lines()
                .find_map(|l| l.split_once("model name"))
                .and_then(|(_, rest)| rest.split_once(':'))
                .map(|(_, v)| v.trim().to_string())
        })
    })
}

fn sysctl_str(key: &str) -> Option<String> {
    run_out("sysctl", &["-n", key])
}

/// A CPU count from `sysctl` where there is one, else from the `/proc/cpuinfo`
/// key that carries it.
fn count(sysctl_key: &str, proc_key: &str) -> Option<u64> {
    sysctl_u64(sysctl_key).or_else(|| {
        let text = std::fs::read_to_string("/proc/cpuinfo").ok()?;
        let distinct: std::collections::BTreeSet<&str> = text
            .lines()
            .filter_map(|l| l.split_once(':'))
            .filter(|(k, _)| k.trim() == proc_key)
            .map(|(_, v)| v.trim())
            .collect();
        (!distinct.is_empty()).then_some(distinct.len() as u64)
    })
}

fn meminfo_bytes() -> Option<u64> {
    let text = std::fs::read_to_string("/proc/meminfo").ok()?;
    let kb: u64 = text
        .lines()
        .find_map(|l| l.strip_prefix("MemTotal:"))?
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;
    Some(kb * 1024)
}

/// `<target>/validation-report`, honouring `CARGO_TARGET_DIR` so a run with a
/// relocated target directory writes where the rest of the build output went.
pub fn report_dir() -> PathBuf {
    let target = match std::env::var_os("CARGO_TARGET_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target"),
    };
    target.join("validation-report")
}
