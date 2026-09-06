//! The part of `validation/manifest.toml` a gate has to read: which rows the
//! pinned reference bundle carries.
//!
//! A gate iterates over the runs its own committed reference names, and a run it
//! cannot find is normally an incomplete environment —
//! [`vibegraph::validation::require`] says exactly that. The exception is
//! *declared*: a row marked `bundled = false` has banked artifacts that exist in
//! a local work area and are deliberately not in the pinned bundle yet, so a
//! checkout that fetched the bundle and does not have that run has a **complete**
//! environment with respect to what the bundle promises. Distinguishing the two
//! needs the manifest, because nothing else in the tree records which rows the
//! bundle carries.
//!
//! `key`, `bundled`, `model`, `restrict` and each category cell's `tier` and
//! `mode` are read. The report collator parses the whole manifest and is the
//! authority on its shape; these are the fields a gate has to agree with it
//! about, so they are deserialised the same way — `bundled` defaulting to true,
//! which is what an entry that says nothing means.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Manifest {
    #[serde(rename = "process")]
    processes: Vec<Process>,
}

#[derive(Debug, Deserialize)]
struct Process {
    key: String,
    #[serde(default = "bundled_by_default")]
    bundled: bool,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    restrict: Option<String>,
    #[serde(default)]
    mg_amplitude: Option<MgAmplitude>,
    #[serde(default)]
    categories: Categories,
}

#[derive(Debug, Deserialize)]
struct MgAmplitude {
    process: String,
}

fn bundled_by_default() -> bool {
    true
}

#[derive(Debug, Default, Deserialize)]
struct Categories {
    #[serde(default)]
    diagrams: Option<Cell>,
    #[serde(default)]
    amplitudes: Option<Cell>,
}

#[derive(Debug, Deserialize)]
struct Cell {
    tier: String,
    #[serde(default)]
    mode: Option<String>,
}

pub fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/manifest.toml")
}

fn load_manifest() -> Manifest {
    let path = manifest_path();
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    toml::from_str(&text).unwrap_or_else(|e| panic!("{} does not parse: {e}", path.display()))
}

/// The keys of every row the pinned reference bundle does **not** carry.
///
/// Reading the absent set rather than the present one is deliberate: a row the
/// manifest does not mention at all is not exempt from anything, so it stays out
/// of this set and a gate that cannot find its run still fails.
pub fn unbundled_rows() -> BTreeSet<String> {
    load_manifest()
        .processes
        .into_iter()
        .filter(|p| !p.bundled)
        .map(|p| p.key)
        .collect()
}

/// The keys of every row the manifest declares `diagrams` hermetic — the set
/// the committed `validation/madgraph/diagrams.json` must cover exactly.
pub fn hermetic_diagram_rows() -> BTreeSet<String> {
    load_manifest()
        .processes
        .into_iter()
        .filter(|p| {
            p.categories
                .diagrams
                .as_ref()
                .is_some_and(|c| c.tier == "hermetic")
        })
        .map(|p| p.key)
        .collect()
}

/// The UFO model a row's reference was generated against, where it is not the
/// interned Standard Model.
#[derive(Debug, Clone)]
pub struct RowModel {
    /// Repository-relative UFO directory, as the row's `.mg5` script imports it.
    pub dir: String,
    /// The restrict card's name: `restrict_<name>.dat`, as `import model
    /// <dir>-<name>` selects it.
    pub restrict: Option<String>,
}

impl RowModel {
    /// The UFO directory, absolute.
    pub fn dir_path(&self) -> PathBuf {
        repo_root().join(&self.dir)
    }

    /// The restrict card, searched where each of the two kinds of card lives:
    /// inside the model directory for the ones the model ships, and under
    /// `validation/madgraph/cards/<family>/` for the ones this repository
    /// authors — the vendored directories are committed byte for byte against a
    /// `SHA256SUMS` manifest, so an authored card cannot live in one.
    /// `validation/madgraph/build.sh` copies both into the work-area model copy,
    /// which is how MadGraph sees one directory holding all of them.
    pub fn restrict_card(&self) -> Option<PathBuf> {
        let name = self.restrict.as_ref()?;
        let file = format!("restrict_{name}.dat");
        let shipped = self.dir_path().join(&file);
        if shipped.exists() {
            return Some(shipped);
        }
        let family = self
            .dir
            .rsplit('/')
            .next()
            .filter(|d| d.starts_with("SMEFTsim_"))
            .map(|_| "smeft")
            .unwrap_or_else(|| self.dir.rsplit('/').next().unwrap_or(""));
        Some(
            repo_root()
                .join("validation/madgraph/cards")
                .join(family)
                .join(file),
        )
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// Every row that names a UFO model of its own, by key.
pub fn row_models() -> BTreeMap<String, RowModel> {
    load_manifest()
        .processes
        .into_iter()
        .filter_map(|p| {
            let dir = p.model?;
            Some((
                p.key,
                RowModel {
                    dir,
                    restrict: p.restrict,
                },
            ))
        })
        .collect()
}

/// Each row's declared mode for one category — `gate` or `info`.
///
/// A gate reads this rather than carrying its own list of exempt rows: the
/// manifest is where a cell's enforcement is declared, and a second list is a
/// second place for the two to disagree. A row whose cell declares no mode
/// (`blocked`, `covered-by`, `uncovered`) is absent from the map.
pub fn category_modes(category: &str) -> BTreeMap<String, String> {
    load_manifest()
        .processes
        .into_iter()
        .filter_map(|p| {
            let cell = match category {
                "diagrams" => p.categories.diagrams,
                "amplitudes" => p.categories.amplitudes,
                other => panic!("no category '{other}' in the manifest"),
            }?;
            Some((p.key, cell.mode?))
        })
        .collect()
}

/// The process string each row's amplitude table was banked for, by row key.
///
/// The banked table carries the same string, and the amplitude gate enumerates
/// from it; reading it back from the manifest is what keeps a committed table and
/// the declaration it was generated from from drifting apart. It is not always the
/// row's own `process`: `pp_to_ll_qcd0` gates a hadronic process at the diagram
/// level and one of its partonic subprocesses at the amplitude level.
pub fn mg_amplitude_processes() -> BTreeMap<String, String> {
    load_manifest()
        .processes
        .into_iter()
        .filter_map(|p| Some((p.key, p.mg_amplitude?.process)))
        .collect()
}
