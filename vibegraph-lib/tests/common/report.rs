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
//! | `process` | the process string that was integrated |
//! | `sigma_vg_pb` / `sigma_vg_err_pb` | this crate's cross section and its Monte-Carlo error |
//! | `sigma_mg_pb` / `sigma_mg_err_pb` | the banked MadGraph value and its error |
//! | `pull` | `(sigma_vg − sigma_mg) / sqrt(err_vg² + err_mg²)`, signed |
//! | `rel` | `sigma_vg / sigma_mg − 1`, signed |
//! | `chi2_dof` | over a seed sweep, the scatter of the seeds about their own mean in units of their quoted errors; otherwise the integration's own χ²/dof |
//! | `seeds` | the RNG seeds the measurement was taken on |
//! | `per_seed` | one `{seed, sigma_pb, sigma_err_pb}` per seed, in sweep order |
//! | `neval` / `niter` | the VEGAS budget per seed |
//! | `subsampler` | per sampling channel, what the rule-based composition chose |
//! | `note` | free text a reader needs to interpret the cell; `null` when none |
//!
//! `pull` and `rel` are signed here even where the gate asserts on their
//! magnitude: a table of one-sided numbers hides whether a family of rows leans
//! the same way.

#![allow(dead_code)]

use std::path::PathBuf;

use serde::Serialize;
use vibegraph::artifact::ChannelSampler;

/// The schema version the files below are written under.
pub const SCHEMA: u32 = 1;

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
        let dir = report_dir().join(self.category);
        std::fs::create_dir_all(&dir).unwrap_or_else(|e| {
            panic!("cannot create the report directory {}: {e}", dir.display())
        });
        let name = match &self.variant {
            Some(v) => format!("{}__{v}.json", self.row),
            None => format!("{}.json", self.row),
        };
        let path = dir.join(name);
        let text = serde_json::to_string_pretty(self).expect("row serialises");
        std::fs::write(&path, text + "\n")
            .unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
        eprintln!("[report] wrote {}", path.display());
    }
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
