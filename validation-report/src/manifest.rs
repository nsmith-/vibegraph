//! `validation/manifest.toml` as this crate reads it.
//!
//! The manifest is the declared shape of the report: which rows exist, which of
//! the four categories each one is measured in, and — for the cells nothing
//! measures — why. Everything here is required rather than defaulted, so a key
//! that disappears from the manifest is a parse error and not a silently empty
//! cell.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Manifest {
    pub(crate) schema: u32,
    pub(crate) refdata: Refdata,
    #[serde(rename = "process")]
    pub(crate) processes: Vec<Process>,
    #[serde(default)]
    pub(crate) standalone: Vec<Standalone>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Refdata {
    pub(crate) version: u32,
    pub(crate) archive: String,
    pub(crate) url: String,
    pub(crate) sha256: String,
    pub(crate) size_bytes: u64,
    pub(crate) published: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Process {
    pub(crate) key: String,
    pub(crate) process: String,
    pub(crate) class: String,
    pub(crate) n_final: u32,
    pub(crate) rationale: String,
    /// The UFO model directory the row's reference was generated against, and
    /// the restrict card it was imported with. Absent where the row is the
    /// interned Standard Model, which is what most of the table is.
    #[serde(default)]
    pub(crate) model: Option<String>,
    #[serde(default)]
    pub(crate) restrict: Option<String>,
    /// A row whose reference run does not exist yet.
    #[serde(default)]
    pub(crate) status: Option<String>,
    /// A row whose banked artifacts are not in the pinned reference bundle, so a
    /// fetching checkout does not have them.
    #[serde(default = "yes")]
    pub(crate) bundled: bool,
    pub(crate) categories: Categories,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub(crate) struct Categories {
    pub(crate) diagrams: Cell,
    pub(crate) amplitudes: Cell,
    pub(crate) integrals: Cell,
    pub(crate) samples: Cell,
}

impl Categories {
    pub(crate) fn get(&self, category: Category) -> &Cell {
        match category {
            Category::Diagrams => &self.diagrams,
            Category::Amplitudes => &self.amplitudes,
            Category::Integrals => &self.integrals,
            Category::Samples => &self.samples,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Cell {
    pub(crate) tier: Tier,
    pub(crate) mode: Option<Mode>,
    /// What a `blocked` cell waits on.
    pub(crate) blocker: Option<String>,
    /// What a `covered-by` cell points at.
    #[serde(default)]
    pub(crate) rows: Vec<String>,
    pub(crate) note: Option<String>,
    /// An `amplitudes` claim about how the comparison ran, checked against the
    /// gate's own measurement where both are stated.
    pub(crate) factorized: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Tier {
    Hermetic,
    Banked,
    Long,
    Blocked,
    CoveredBy,
    Uncovered,
}

impl Tier {
    /// Whether a gate in this tier runs — and so writes a row file — under
    /// `pixi run validate`.
    pub(crate) fn is_measured_here(self) -> bool {
        matches!(self, Tier::Hermetic | Tier::Banked)
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Tier::Hermetic => "hermetic",
            Tier::Banked => "banked",
            Tier::Long => "long",
            Tier::Blocked => "blocked",
            Tier::CoveredBy => "covered-by",
            Tier::Uncovered => "uncovered",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Mode {
    Gate,
    Info,
}

impl Mode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Mode::Gate => "gate",
            Mode::Info => "info",
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Standalone {
    pub(crate) key: String,
    pub(crate) layer: String,
    #[serde(default)]
    pub(crate) targets: Vec<String>,
    pub(crate) rationale: String,
    #[serde(default)]
    pub(crate) note: Option<String>,
    /// A gate whose driver is not a Rust test names the pixi task that runs it,
    /// the environment that task needs, and the file it writes its verdict to.
    #[serde(default)]
    pub(crate) task: Option<String>,
    #[serde(default)]
    pub(crate) environment: Option<String>,
    #[serde(default)]
    pub(crate) row: Option<String>,
}

/// The four per-process categories, in the order the table's columns run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Category {
    Diagrams,
    Amplitudes,
    Integrals,
    Samples,
}

pub(crate) const CATEGORIES: [Category; 4] = [
    Category::Diagrams,
    Category::Amplitudes,
    Category::Integrals,
    Category::Samples,
];

impl Category {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Category::Diagrams => "diagrams",
            Category::Amplitudes => "amplitudes",
            Category::Integrals => "integrals",
            Category::Samples => "samples",
        }
    }

    pub(crate) fn parse(name: &str) -> Option<Category> {
        CATEGORIES.into_iter().find(|c| c.as_str() == name)
    }
}

impl Manifest {
    pub(crate) fn load(path: &Path) -> Result<Manifest, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        toml::from_str(&text).map_err(|e| format!("cannot parse {}: {e}", path.display()))
    }

    /// The rows in the order the table renders them: single-channel first, then
    /// multi-channel, each by increasing final-state multiplicity and otherwise
    /// in manifest order.
    pub(crate) fn ordered_rows(&self) -> Vec<&Process> {
        let mut rows: Vec<(usize, &Process)> = self.processes.iter().enumerate().collect();
        rows.sort_by_key(|(i, p)| (p.class != "single-channel", p.n_final, *i));
        rows.into_iter().map(|(_, p)| p).collect()
    }

    pub(crate) fn by_key(&self) -> BTreeMap<&str, &Process> {
        self.processes.iter().map(|p| (p.key.as_str(), p)).collect()
    }
}
