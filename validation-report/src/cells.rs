//! The row files the gates wrote, and the cell each one renders to.
//!
//! A row file is `target/validation-report/<category>/<row>[__<variant>].json`,
//! written by whichever gate measured that cell (the schema is documented in
//! `vibegraph-lib/tests/common/report.rs`). This module reads them, turns each
//! into the one-line metric its column shows, and — where a row was measured
//! more than once — decides which of the measurements the cell reports.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

use crate::manifest::Category;

/// The row-file schema this collator understands. A file written under a
/// different one is an error rather than a best-effort read: the fields it
/// renders are the fields whose meaning that number depends on.
pub const ROW_SCHEMA: u32 = 1;

/// The fields every row file carries, whatever its category.
#[derive(Debug, Deserialize)]
struct Common {
    schema: u32,
    row: String,
    variant: Option<String>,
    category: String,
    mode: String,
    status: String,
    #[serde(default)]
    process: String,
    #[serde(default)]
    note: Option<String>,
    #[serde(default)]
    duration_s: Option<f64>,
}

#[derive(Debug)]
pub struct RowFile {
    pub path: PathBuf,
    pub row: String,
    pub variant: Option<String>,
    pub category: Category,
    pub mode: String,
    pub status: String,
    pub process: String,
    pub note: Option<String>,
    /// Wall-clock seconds the gate spent measuring this row, where it timed
    /// itself. A measurement, not a verdict: nothing here reads it.
    pub duration_s: Option<f64>,
    value: Value,
}

impl RowFile {
    fn load(path: &Path) -> Result<RowFile, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let value: Value = serde_json::from_str(&text)
            .map_err(|e| format!("cannot parse {}: {e}", path.display()))?;
        let common: Common = serde_json::from_value(value.clone())
            .map_err(|e| format!("{} is missing a common field: {e}", path.display()))?;
        if common.schema != ROW_SCHEMA {
            return Err(format!(
                "{} is schema {} and this collator reads schema {ROW_SCHEMA}",
                path.display(),
                common.schema
            ));
        }
        let category = Category::parse(&common.category).ok_or_else(|| {
            format!(
                "{} names the category '{}', which is not one of the manifest's four",
                path.display(),
                common.category
            )
        })?;
        Ok(RowFile {
            path: path.to_path_buf(),
            row: common.row,
            variant: common.variant,
            category,
            mode: common.mode,
            status: common.status,
            process: common.process,
            note: common.note,
            duration_s: common.duration_s,
            value,
        })
    }

    fn f64_at(&self, field: &str) -> Result<f64, String> {
        self.value
            .get(field)
            .and_then(Value::as_f64)
            .ok_or_else(|| format!("{}: no numeric '{field}'", self.path.display()))
    }

    fn u64_at(&self, field: &str) -> Result<u64, String> {
        self.value
            .get(field)
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("{}: no integer '{field}'", self.path.display()))
    }

    fn str_at(&self, field: &str) -> Result<&str, String> {
        self.value
            .get(field)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{}: no string '{field}'", self.path.display()))
    }

    pub fn bool_at(&self, field: &str) -> Option<bool> {
        self.value.get(field).and_then(Value::as_bool)
    }

    /// How this measurement labels itself in the table when a row carries more
    /// than one.
    pub fn label(&self) -> &str {
        self.variant.as_deref().unwrap_or("default")
    }

    /// The cell text: the category's metric, as the column shows it.
    pub fn metric(&self) -> Result<String, String> {
        Ok(match self.category {
            Category::Diagrams => format!("{}/{}", self.u64_at("ours")?, self.u64_at("theirs")?),
            Category::Amplitudes => format!("max rel {}", exp(self.worst_amplitude_deviation()?)),
            Category::Integrals => format!(
                "pull {:+.2}, chi2/dof {}",
                self.f64_at("pull")?,
                chi2(self.f64_at("chi2_dof")?)
            ),
            Category::Samples => format!(
                "KS p {}, chi2 p {}",
                pval(self.f64_at("min_ks_p")?),
                pval(self.f64_at("min_chi2_p")?)
            ),
        })
    }

    /// The one number that distinguishes two measurements of the same row, for
    /// the cell that lists both.
    pub fn short_metric(&self) -> Result<String, String> {
        Ok(match self.category {
            Category::Diagrams => format!("{}/{}", self.u64_at("ours")?, self.u64_at("theirs")?),
            Category::Amplitudes => exp(self.worst_amplitude_deviation()?),
            Category::Integrals => format!("pull {:+.2}", self.f64_at("pull")?),
            Category::Samples => format!("KS p {}", pval(self.f64_at("min_ks_p")?)),
        })
    }

    /// The detail line the report's per-row breakdown carries: everything the
    /// one-line metric had to leave out.
    pub fn detail(&self) -> Result<String, String> {
        Ok(match self.category {
            Category::Diagrams => format!(
                "{} diagrams counted MadGraph's way against {}, over {} across every concrete subprocess",
                self.u64_at("ours")?,
                self.u64_at("theirs")?,
                self.u64_at("ours_all_subprocesses")?,
            ),
            Category::Amplitudes => {
                let per_diagram = match self.value.get("per_diagram").and_then(Value::as_f64) {
                    Some(v) => exp(v),
                    None => "not banked".to_string(),
                };
                let amp2 = match self.value.get("amp2").and_then(Value::as_f64) {
                    Some(v) => exp(v),
                    None => "grouping merged by MadGraph".to_string(),
                };
                format!(
                    "NGRAPHS {}, NCOLOR {}; |M|^2 max rel {} over {} grid points and {} over {} event points; \
                     per-diagram {per_diagram}, per-flow {}, JAMP2 {}; {} configurations: \
                     amplitude {}, AMP2 {amp2}, helicity pruning moves AMP2 by {}{}",
                    self.u64_at("n_graphs")?,
                    self.u64_at("n_flows")?,
                    exp(self.f64_at("max_rel_grid")?),
                    self.u64_at("points_grid")?,
                    exp(self.f64_at("max_rel_event")?),
                    self.u64_at("points_event")?,
                    exp(self.f64_at("per_flow")?),
                    exp(self.f64_at("jamp2")?),
                    self.u64_at("n_configs")?,
                    exp(self.f64_at("per_config")?),
                    exp(self.f64_at("amp2_pruned")?),
                    if self.bool_at("factorized") == Some(true) {
                        " (comparison factorized into its two projections)"
                    } else {
                        ""
                    },
                )
            }
            Category::Integrals => format!(
                "sigma {:.6} +- {:.6} pb against MadGraph {:.6} +- {:.6} pb, rel {:+.4}%, {} seed(s) at {} x {}",
                self.f64_at("sigma_vg_pb")?,
                self.f64_at("sigma_vg_err_pb")?,
                self.f64_at("sigma_mg_pb")?,
                self.f64_at("sigma_mg_err_pb")?,
                100.0 * self.f64_at("rel")?,
                self.value.get("seeds").and_then(Value::as_array).map_or(0, Vec::len),
                self.u64_at("neval")?,
                self.u64_at("niter")?,
            ),
            Category::Samples => format!(
                "{} MadGraph events, {} labelling, p-floor {}; worst observable {} at KS p {}, \
                 worst column {} at chi2 p {}",
                self.u64_at("mg_events")?,
                self.str_at("labelling")?,
                exp(self.f64_at("p_floor")?),
                self.str_at("worst_ks_observable")?,
                pval(self.f64_at("min_ks_p")?),
                self.str_at("worst_chi2_column")?,
                pval(self.f64_at("min_chi2_p")?),
            ),
        })
    }

    fn worst_amplitude_deviation(&self) -> Result<f64, String> {
        Ok(self
            .f64_at("max_rel_grid")?
            .max(self.f64_at("max_rel_event")?))
    }

    /// How bad this measurement is, so the worse of two is the one the cell
    /// reports. A failed gate outranks every passing measurement whatever its
    /// numbers say.
    pub fn severity(&self) -> f64 {
        let own = match self.category {
            Category::Diagrams => self
                .u64_at("ours")
                .and_then(|a| Ok(a.abs_diff(self.u64_at("theirs")?) as f64))
                .unwrap_or(f64::INFINITY),
            Category::Amplitudes => self.worst_amplitude_deviation().unwrap_or(f64::INFINITY),
            Category::Integrals => self.f64_at("pull").map(f64::abs).unwrap_or(f64::INFINITY),
            Category::Samples => {
                let ks = self.f64_at("min_ks_p").unwrap_or(0.0);
                let chi2 = self.f64_at("min_chi2_p").unwrap_or(0.0);
                -ks.min(chi2)
            }
        };
        if self.status == "fail" {
            own + 1e12
        } else {
            own
        }
    }
}

/// Every row file under the report directory, in a stable order.
pub fn load_all(report_dir: &Path) -> (Vec<RowFile>, Vec<String>) {
    let mut rows = Vec::new();
    let mut problems = Vec::new();
    for category in crate::manifest::CATEGORIES {
        let dir = report_dir.join(category.as_str());
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            // A category with no directory at all is not an error here: the
            // per-cell assertion below is what notices its rows are missing, and
            // it names them one by one.
            Err(_) => continue,
        };
        let mut paths: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|e| e == "json"))
            .collect();
        paths.sort();
        for path in paths {
            match RowFile::load(&path) {
                Ok(row) if row.category != category => problems.push(format!(
                    "{} is filed under {}/ and calls itself {}",
                    path.display(),
                    category.as_str(),
                    row.category.as_str()
                )),
                Ok(row) => rows.push(row),
                Err(e) => problems.push(e),
            }
        }
    }
    (rows, problems)
}

/// Two significant figures in scientific notation, which is the precision every
/// deviation in the table is read at.
pub fn exp(v: f64) -> String {
    format!("{v:.2e}")
}

/// A χ²/dof plainly where it is a statistic, in scientific notation where it is
/// no longer one.
///
/// A channel-split integration divides its budget over every channel, and a
/// channel whose term is identically zero in some iterations and denormal in
/// others has its `Δ²/σ²` divided by a variance floored at the smallest positive
/// double. The resulting χ² is not large, it is meaningless — and printed in
/// fixed notation it is a two-hundred-digit number across a table cell. The value
/// is passed through rather than clamped, because clamping would make a broken
/// statistic look like a merely bad one; only its width is bounded.
pub fn chi2(v: f64) -> String {
    if v.is_finite() && v.abs() < 1e4 {
        format!("{v:.2}")
    } else {
        format!("{v:.2e}")
    }
}

/// A p-value plainly where it is readable, in scientific notation where it is
/// small enough that the decimal form is a run of zeroes.
pub fn pval(p: f64) -> String {
    if p >= 1e-3 {
        format!("{p:.3}")
    } else if p > 0.0 {
        format!("{p:.1e}")
    } else {
        // The chi-squared tail underflows to zero long before the statistic
        // stops growing, so the cell says that rather than printing a p of 0.
        "<1e-300".to_string()
    }
}
