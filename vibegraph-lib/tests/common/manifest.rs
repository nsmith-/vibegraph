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
//! `key`, `bundled` and `categories.diagrams.tier` are read. The report
//! collator parses the whole manifest and is the authority on its shape; these
//! are the fields a gate has to agree with it about, so they are deserialised
//! the same way — `bundled` defaulting to true, which is what an entry that
//! says nothing means.

use std::collections::BTreeSet;
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
    categories: Categories,
}

fn bundled_by_default() -> bool {
    true
}

#[derive(Debug, Default, Deserialize)]
struct Categories {
    #[serde(default)]
    diagrams: Option<Cell>,
}

#[derive(Debug, Deserialize)]
struct Cell {
    tier: String,
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
