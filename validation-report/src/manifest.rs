//! `validation/manifest.toml` as this crate reads it.
//!
//! The manifest is the declared shape of the report: which rows exist, which of
//! the four categories each one is measured in, and — for the cells nothing
//! measures — why. What every row and cell must declare (a row's key, process,
//! class and four categories; a cell's tier) is required, so a key that
//! disappears from the manifest is a parse error and not a silently empty cell.
//! The rest is optional where its absence says something: a row with no `model`
//! is the interned Standard Model, a row with no `bundled` is in the bundle, a
//! cell with no `blocker`, `rows` or `note` has nothing of that kind to declare,
//! and the collator reports a cell whose tier needs one of them and lacks it.
//!
//! The closed vocabularies — a row's class, a cell's tier and mode, a standalone
//! gate's layer — are enums, so a misspelt value is a parse error too.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

/// The manifest schema this collator reads. A manifest declaring another one is
/// refused before anything is rendered from it: a changed schema is a changed
/// meaning for some field this crate reads.
pub(crate) const MANIFEST_SCHEMA: u32 = 1;

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
    pub(crate) class: Class,
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
    pub(crate) status: Option<ProcessStatus>,
    /// A row whose banked artifacts are not in the pinned reference bundle, so a
    /// fetching checkout does not have them.
    #[serde(default = "yes")]
    pub(crate) bundled: bool,
    pub(crate) categories: Categories,
}

fn yes() -> bool {
    true
}

/// Whether a row's phase space is one channel or several, which is what the
/// table is split on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Class {
    SingleChannel,
    MultiChannel,
}

impl Class {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Class::SingleChannel => "single-channel",
            Class::MultiChannel => "multi-channel",
        }
    }
}

/// The state a row declares when it is not an ordinary measured row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ProcessStatus {
    /// The row's reference run does not exist yet.
    Planned,
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

/// The layer a standalone gate runs in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Layer {
    Hermetic,
    Banked,
    /// Driven by a task of its own rather than by the test suite.
    Oracle,
}

impl Layer {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Layer::Hermetic => "hermetic",
            Layer::Banked => "banked",
            Layer::Oracle => "oracle",
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Standalone {
    pub(crate) key: String,
    pub(crate) layer: Layer,
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
        Manifest::parse(&text, path)
    }

    /// Parse a manifest's text, refusing one written under a schema other than
    /// [`MANIFEST_SCHEMA`]. `origin` names it in the errors.
    fn parse(text: &str, origin: &Path) -> Result<Manifest, String> {
        let manifest: Manifest =
            toml::from_str(text).map_err(|e| format!("cannot parse {}: {e}", origin.display()))?;
        if manifest.schema != MANIFEST_SCHEMA {
            return Err(format!(
                "{} is manifest schema {} and this collator reads schema {MANIFEST_SCHEMA}",
                origin.display(),
                manifest.schema
            ));
        }
        Ok(manifest)
    }

    /// The rows in the order the table renders them: single-channel first, then
    /// multi-channel, each by increasing final-state multiplicity and otherwise
    /// in manifest order.
    pub(crate) fn ordered_rows(&self) -> Vec<&Process> {
        let mut rows: Vec<(usize, &Process)> = self.processes.iter().enumerate().collect();
        rows.sort_by_key(|(i, p)| (p.class != Class::SingleChannel, p.n_final, *i));
        rows.into_iter().map(|(_, p)| p).collect()
    }

    pub(crate) fn by_key(&self) -> BTreeMap<&str, &Process> {
        self.processes.iter().map(|p| (p.key.as_str(), p)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn origin() -> &'static Path {
        Path::new("manifest.toml")
    }

    fn repo_manifest() -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/manifest.toml");
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
    }

    /// The committed manifest is the one this collator renders, so it parses
    /// under the schema the collator reads, closed vocabularies included.
    #[test]
    fn the_committed_manifest_parses() {
        let manifest = Manifest::parse(&repo_manifest(), origin()).expect("committed manifest");
        assert_eq!(manifest.schema, MANIFEST_SCHEMA);
        assert!(!manifest.processes.is_empty());
    }

    /// A manifest written under another schema is refused, naming both schemas,
    /// rather than rendered as if its fields meant what this collator reads.
    #[test]
    fn a_manifest_under_another_schema_is_refused() {
        let text = repo_manifest().replacen(
            &format!("\nschema = {MANIFEST_SCHEMA}\n"),
            &format!("\nschema = {}\n", MANIFEST_SCHEMA + 1),
            1,
        );
        let err = Manifest::parse(&text, origin()).expect_err("a foreign schema is refused");
        assert!(
            err.contains(&format!("manifest schema {}", MANIFEST_SCHEMA + 1))
                && err.contains(&format!("reads schema {MANIFEST_SCHEMA}")),
            "{err}"
        );
    }

    /// A value outside a closed vocabulary is a parse error, not a string the
    /// renderer would carry into the table.
    #[test]
    fn closed_vocabularies_refuse_unknown_values() {
        let text = repo_manifest();
        for (from, to) in [
            ("class = \"single-channel\"", "class = \"single_channel\""),
            ("layer = \"banked\"", "layer = \"bankd\""),
        ] {
            assert!(text.contains(from), "the manifest no longer spells {from}");
            let err = Manifest::parse(&text.replacen(from, to, 1), origin())
                .expect_err("an unknown value is refused");
            assert!(err.contains("unknown variant"), "{to}: {err}");
        }
    }
}
