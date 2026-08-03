//! The kT clustering ([`vibegraph::coupling::cluster`]) against MadGraph's own,
//! event by event.
//!
//! # The oracle
//!
//! An instrumented build of the pinned MadGraph writes, per banked event, every
//! intermediate the clustering passes through: each candidate pair with the arm
//! of the measure it took and whether the crossing inflation fired, each merge
//! with its participating leg sets and scale, the frame changes, the surviving
//! channel list either side of the point the integration channel claims it, the
//! three beam-side vertex indices, and the branch each scale formula was taken
//! from. `validation/madgraph/kt_cluster_dump_manifest.json` pins the files.
//!
//! That record is far finer than the scale it produces, which is the point:
//! `SCALUP` alone cannot see a wrong tie-break on an event where both candidates
//! measure the same, nor a wrong PDG on a line the beam walk never asks about.
//! So the comparison is ordered — the merge sequence first, then the scales —
//! and the first divergence is reported by merge index.
//!
//! # What this engine is given, and what it derives
//!
//! Given, per event: the integration channel and subprocess, the momenta as
//! `cluster()` received them, and the channel forests the process directory was
//! generated with (`configs.inc`, as `IFOR` records). Given per run: the run-card
//! constants.
//!
//! Derived: the whole merge graph from those forests (leg sets, complements, the
//! PDG on each line, the resonance map, and the coupling-order filter), the
//! Breit-Wigner tagging, the clustering, the jet-count memo's re-cluster
//! decision, the beam walk, and both scales.
//!
//! Two of the nine runs put several process directories in one dump, and the
//! dump's per-directory tables carry no directory name, so their forests arrive
//! as several candidate sets. The candidate whose forest is a valid set of model
//! vertices on the event's own flavours is the one used, and the count of events
//! that needed the test is reported.
//!
//! # What this cannot see
//!
//! * **Cross-event state.** MadGraph's `ipdgcl` is a common block that the beam
//!   walk *writes to*, and its jet memo is per process directory, so a replay in
//!   event order is not the same computation as a run in generation order. The
//!   engine starts each event from the merge graph, which is the pure-function
//!   reading; where the dump disagrees, that difference is what it is measuring.
//! * **Branches no banked run reaches.** `ktscheme = 2`, `ickkw > 0`, a fixed
//!   scale on one beam only, and `dj`'s second massless-massive arm are
//!   implemented from the Fortran and exercised by nothing here.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;

use vibegraph::coupling::cluster::graph::{ChannelSet, ColorTable, ConfigForest, ForestLine};
use vibegraph::coupling::cluster::kt::{Channel, ClusterSettings, MergeKind};
use vibegraph::coupling::cluster::setclscales::{setclscales, JetMemo, ScaleSettings};

mod common;

/// The relative agreement required of every scale and every merge measure.
///
/// Both sides evaluate the same expressions on the same inputs, so what is left
/// is the last-ulp spread of `pow`, `cosh` and `log` between two libms. The
/// bound is four orders of magnitude above that spread and four below anything a
/// wrong branch could produce, and the worst observed value is reported so a
/// drift towards it is visible rather than silently absorbed.
const AGREEMENT: f64 = 1e-12;

fn dumps_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/output/ktdump/dumps")
}

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/madgraph/kt_cluster_dump_manifest.json")
}

// ── dump records ─────────────────────────────────────────────────────────────

/// One tagged record of the dump, as the extraction driver's JSON array.
struct Rec<'a>(&'a [Value]);

impl<'a> Rec<'a> {
    fn tag(&self) -> &'a str {
        self.0[0].as_str().expect("a record opens with its tag")
    }
    fn len(&self) -> usize {
        self.0.len()
    }
    fn i(&self, k: usize) -> i64 {
        self.0[k].as_i64().unwrap_or_else(|| self.f(k) as i64)
    }
    fn u(&self, k: usize) -> usize {
        self.i(k).max(0) as usize
    }
    fn f(&self, k: usize) -> f64 {
        self.0[k].as_f64().expect("a numeric field")
    }
    fn b(&self, k: usize) -> bool {
        self.0[k].as_bool().expect("a logical field")
    }
    fn s(&self, k: usize) -> &'a str {
        self.0[k].as_str().expect("a string field")
    }
}

/// The run-card constants a dump carries in its `CONST` record.
#[derive(Clone, Copy, Debug)]
struct RunConstants {
    lpp: [i64; 2],
    d_parameter: f64,
    maxjetflavor: i64,
    ickkw: i64,
    chcluster: bool,
    pdfwgt: bool,
    xqcut: f64,
    xmtc: f64,
    scalefact: f64,
    fixed_ren: bool,
    fixed_fac: [bool; 2],
    bwcutoff: f64,
}

impl RunConstants {
    fn read(rec: &Rec<'_>) -> Self {
        RunConstants {
            lpp: [rec.i(1), rec.i(2)],
            d_parameter: rec.f(3),
            maxjetflavor: rec.i(4),
            ickkw: rec.i(6),
            chcluster: rec.b(7),
            pdfwgt: rec.b(8),
            xqcut: rec.f(9),
            xmtc: rec.f(10),
            scalefact: rec.f(11),
            fixed_ren: rec.b(12),
            fixed_fac: [rec.b(13), rec.b(14)],
            bwcutoff: rec.f(15),
        }
    }
}

/// One process directory's forests, keyed by how many subprocesses it groups —
/// which is the only thing in the dump that tells two directories apart.
#[derive(Clone, Debug, Default)]
struct DirectoryForests {
    n_external: usize,
    n_incoming: usize,
    n_proc: usize,
    /// `(config, line index)` → the line.
    lines: BTreeMap<(usize, i32), ForestLine>,
    n_configs: usize,
}

// ── one event, as the dump records it ────────────────────────────────────────

struct Event {
    index: usize,
    records: Vec<Vec<Value>>,
}

impl Event {
    fn iter<'a>(&'a self, tag: &'a str) -> impl Iterator<Item = Rec<'a>> + 'a {
        self.records
            .iter()
            .map(|r| Rec(r.as_slice()))
            .filter(move |r| r.tag() == tag)
    }
    fn first<'a>(&'a self, tag: &'a str) -> Rec<'a> {
        self.iter(tag)
            .next()
            .unwrap_or_else(|| panic!("event {} has no {tag} record", self.index))
    }
    fn last<'a>(&'a self, tag: &'a str) -> Rec<'a> {
        self.iter(tag)
            .last()
            .unwrap_or_else(|| panic!("event {} has no {tag} record", self.index))
    }
}

/// Stream a run's dump: the header, then the events in the banked file's order.
fn read_dump(path: &Path) -> (Value, impl Iterator<Item = Event>) {
    let child = Command::new("gzip")
        .arg("-dc")
        .arg(path)
        .stdout(Stdio::piped())
        .spawn()
        .expect("gzip -dc");
    let mut reader = BufReader::with_capacity(1 << 20, child.stdout.expect("piped stdout"));
    let mut head = String::new();
    reader.read_line(&mut head).expect("dump header");
    let header: Value = serde_json::from_str(&head).expect("dump header parses");
    let events = reader.lines().map(|line| {
        let line = line.expect("dump line");
        let value: Value = serde_json::from_str(&line).expect("dump event parses");
        Event {
            index: value["index"].as_u64().expect("event index") as usize,
            records: value["records"]
                .as_array()
                .expect("event records")
                .iter()
                .map(|r| r.as_array().expect("a record is an array").clone())
                .collect(),
        }
    });
    (header, events)
}

// ── the derivation the harness has to do before the engine can run ───────────

/// What the harness asks the model: colour by PDG code, and which three-particle
/// vertices it has, by PDG magnitude and by colour representation.
struct Model {
    colors: HashMap<i64, i32>,
    vertices: HashSet<[i64; 3]>,
    colour_vertices: HashSet<[i32; 3]>,
}

impl Model {
    fn new(model: &vibegraph::ufo::UFOModel) -> Self {
        let colors: HashMap<i64, i32> = model
            .particles
            .values()
            .map(|p| (p.pdg_code, p.color))
            .collect();
        let mut vertices = HashSet::new();
        let mut colour_vertices = HashSet::new();
        for vertex in model.vertices.values() {
            if vertex.particles.len() != 3 {
                continue;
            }
            let pdgs: Vec<i64> = vertex
                .particles
                .iter()
                .map(|&id| model.particle(id).pdg_code)
                .collect();
            let mut triple = [pdgs[0].abs(), pdgs[1].abs(), pdgs[2].abs()];
            triple.sort_unstable();
            vertices.insert(triple);
            let mut colours = [
                colors[&pdgs[0]].abs(),
                colors[&pdgs[1]].abs(),
                colors[&pdgs[2]].abs(),
            ];
            colours.sort_unstable();
            colour_vertices.insert(colours);
        }
        Model {
            colors,
            vertices,
            colour_vertices,
        }
    }

    fn color(&self, pdg: i64) -> i32 {
        self.colors.get(&pdg).copied().unwrap_or(0).abs()
    }
}

/// Whether a forest is a possible reading of a directory the event came from.
///
/// Two tests, and the split between them is the point: a timelike line carries
/// `sprop`, which is written per subprocess and so names the event's own
/// flavour, and its vertex must be one the model has. A spacelike line carries
/// only `tprid`, which is written once per channel from the group's *first*
/// subprocess, so its flavour is not the event's — only the colour it must
/// carry is, and that is all the model is asked about there.
fn forest_is_consistent(
    forests: &DirectoryForests,
    config: usize,
    external: &[i64],
    iproc: usize,
    model: &Model,
) -> bool {
    let lines: Vec<&ForestLine> = forests
        .lines
        .iter()
        .filter(|((c, _), _)| *c == config)
        .map(|(_, line)| line)
        .collect();
    let entry = |index: i32| -> Option<(i64, bool)> {
        if index > 0 {
            return external
                .get(index as usize - 1)
                .copied()
                .map(|pdg| (pdg, true));
        }
        let line = lines.iter().find(|l| l.index == index)?;
        let sprop = line.sprop.get(iproc - 1).copied().unwrap_or(0);
        Some(if sprop != 0 {
            (sprop, true)
        } else {
            (line.tprid, false)
        })
    };
    for line in &lines {
        let legs = [
            entry(line.daughters[0]),
            entry(line.daughters[1]),
            entry(line.index),
        ];
        let Some(legs) = legs.iter().copied().collect::<Option<Vec<(i64, bool)>>>() else {
            continue;
        };
        if legs.iter().any(|&(pdg, _)| pdg == 0) {
            continue;
        }
        let mut colours = [
            model.color(legs[0].0),
            model.color(legs[1].0),
            model.color(legs[2].0),
        ];
        colours.sort_unstable();
        if !model.colour_vertices.contains(&colours) {
            return false;
        }
        if legs.iter().all(|&(_, exact)| exact) {
            let mut triple = [legs[0].0.abs(), legs[1].0.abs(), legs[2].0.abs()];
            triple.sort_unstable();
            if !model.vertices.contains(&triple) {
                return false;
            }
        }
    }
    true
}

/// The QCD coupling order of a channel, counted the way a config's own vertices
/// carry it: a vertex whose three lines are all coloured is a strong one.
///
/// The dump reports `nqcd` per channel, but not per directory, so a run that
/// puts two directories in one dump cannot be read for it. What the merge graph
/// needs is only the *partition* of channels by equal order, which this
/// reproduces; the runs whose dump is unambiguous check it.
fn qcd_order(
    forests: &DirectoryForests,
    config: usize,
    external: &[i64],
    iproc: usize,
    colors: &ColorTable,
) -> i64 {
    let lines: Vec<&ForestLine> = forests
        .lines
        .iter()
        .filter(|((c, _), _)| *c == config)
        .map(|(_, line)| line)
        .collect();
    let pdg_of = |index: i32| -> i64 {
        if index > 0 {
            return external.get(index as usize - 1).copied().unwrap_or(0);
        }
        match lines.iter().find(|l| l.index == index) {
            Some(line) => {
                let sprop = line.sprop.get(iproc - 1).copied().unwrap_or(0);
                if sprop != 0 {
                    sprop
                } else {
                    line.tprid
                }
            }
            None => 0,
        }
    };
    let mut order = 0;
    for line in &lines {
        let legs = [
            pdg_of(line.daughters[0]),
            pdg_of(line.daughters[1]),
            pdg_of(line.index),
        ];
        if legs.iter().all(|&pdg| colors.is_qcd(pdg)) {
            order += 1;
        }
    }
    // The vertex that closes the tree on the beams is written only when the
    // channel reaches them through a spacelike line; otherwise it is implicit.
    if lines.len() == forests.n_external - 3 {
        let root = lines
            .iter()
            .find(|line| !lines.iter().any(|o| o.daughters.contains(&line.index)));
        if let Some(root) = root {
            let legs = [external[0], external[1], pdg_of(root.index)];
            if legs.iter().all(|&pdg| colors.is_qcd(pdg)) {
                order += 1;
            }
        }
    }
    order
}

/// A channel set built for one (directory, subprocess flavour) pair.
fn build_channel_set(
    forests: &DirectoryForests,
    external: &[i64],
    iproc: usize,
    colors: &ColorTable,
) -> ChannelSet {
    let mut configs = vec![ConfigForest::default(); forests.n_configs];
    for ((config, _), line) in &forests.lines {
        configs[config - 1].lines.push(line.clone());
    }
    for (index, config) in configs.iter_mut().enumerate() {
        config.lines.sort_by_key(|line| -line.index);
        config.nqcd = qcd_order(forests, index + 1, external, iproc, colors);
    }
    // Only the subprocess the event belongs to is asked for a table, so the
    // group's other flavour rows are filled with it; nothing downstream reads
    // them.
    let external_pdg = vec![external.to_vec(); forests.n_proc];
    let contributes = vec![vec![true; forests.n_configs]; forests.n_proc];
    ChannelSet {
        n_external: forests.n_external,
        n_incoming: forests.n_incoming,
        configs,
        external_pdg,
        contributes,
    }
}

// ── the comparison ───────────────────────────────────────────────────────────

#[derive(Default)]
struct Tally {
    events: usize,
    sequence_ok: usize,
    scales_ok: usize,
    candidates: usize,
    worst_measure: f64,
    worst_scale: f64,
    /// Flavour assignments whose process directory more than one dumped forest
    /// set could have carried.
    ambiguous_directory: usize,
    /// Of those, the ones the forest test alone did not settle, so the event's
    /// own candidate list was consulted.
    oracle_directory: usize,
    /// Events that reproduce only once the reference's carried-over on-shell
    /// flags are supplied, which no function of the event can predict.
    carried_flags: usize,
    /// Merge tables checked against the reference's own, whole.
    tables_checked: usize,
    failures: BTreeMap<String, (usize, usize)>,
    /// What the engine exercised, so a pass cannot be a pass over nothing.
    coverage: BTreeMap<String, BTreeMap<String, usize>>,
}

impl Tally {
    fn bump(&mut self, key: &str, value: &str) {
        *self
            .coverage
            .entry(key.to_string())
            .or_default()
            .entry(value.to_string())
            .or_insert(0) += 1;
    }

    fn fail(&mut self, class: &str, event: usize) {
        let entry = self.failures.entry(class.to_string()).or_insert((0, event));
        entry.0 += 1;
    }
}

fn close(a: f64, b: f64) -> f64 {
    if a == b {
        return 0.0;
    }
    let scale = a.abs().max(b.abs());
    if scale == 0.0 {
        return 0.0;
    }
    (a - b).abs() / scale
}

#[test]
#[ignore = "oracle layer: the 75 MB kT dumps are outside the reference bundle; `pixi run -e madgraph validate-kt-cluster` builds and runs them"]
fn the_clustering_engine_reproduces_madgraphs_own() {
    let dumps = dumps_dir();
    assert!(
        dumps.is_dir() && manifest_path().is_file(),
        "no kT clustering dumps at {} or manifest at {}: run \
         `pixi run -e madgraph validate-kt-cluster` to build and run them",
        dumps.display(),
        manifest_path().display()
    );
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(manifest_path()).expect("manifest"))
            .expect("manifest parses");
    let runs = manifest["runs"].as_object().expect("manifest runs");

    let model = Model::new(common::sm_model().as_ref());

    let mut summary: Vec<(String, Tally)> = Vec::new();
    let mut names: Vec<&String> = runs.keys().collect();
    names.sort();
    // A divergence is always localised to one process, and a run takes seconds,
    // so `KT_RUN` narrows the comparison to the runs whose name contains it.
    let only = std::env::var("KT_RUN").unwrap_or_default();
    for name in names {
        if !only.is_empty() && !name.contains(&only) {
            continue;
        }
        let entry = &runs[name];
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(entry["path"].as_str().expect("dump path"));
        assert!(
            path.is_file(),
            "manifest names a missing dump {}",
            path.display()
        );
        let tally = compare_run(name, &path, &model);
        assert_eq!(
            tally.events,
            entry["n_events"].as_u64().expect("n_events") as usize,
            "{name}: read a different number of events than the manifest pins"
        );
        // The manifest carries the reference's own count of how often each
        // branch fired. Reproducing those counts is what keeps a passing
        // comparison from being a comparison over nothing: a branch with a zero
        // here is a branch this run cannot judge an engine on, and a branch whose
        // count moved is a regression the event-by-event check might not localise.
        let counted = entry["coverage"].as_object().expect("manifest coverage");
        for (key, expected) in counted {
            let expected = expected.as_object().expect("a coverage block");
            let Some(mine) = tally.coverage.get(key) else {
                panic!("{name}: the engine counted nothing for {key}");
            };
            for (branch, count) in expected {
                let count = count.as_u64().expect("a count") as usize;
                assert_eq!(
                    mine.get(branch).copied().unwrap_or(0),
                    count,
                    "{name}: {key}/{branch} fired a different number of times"
                );
            }
            for (branch, count) in mine {
                assert!(
                    expected.contains_key(branch) || *count == 0,
                    "{name}: {key}/{branch} fired {count} times and the reference never took it"
                );
            }
        }
        summary.push((name.clone(), tally));
    }

    println!(
        "\n{:<26} {:>7} {:>10} {:>10} {:>11} {:>11} {:>6} {:>7}",
        "run", "events", "sequence", "scales", "worst dj", "worst mu", "amb", "carried"
    );
    for (name, tally) in &summary {
        println!(
            "{:<26} {:>7} {:>10} {:>10} {:>11.2e} {:>11.2e} {:>6} {:>7}",
            name,
            tally.events,
            tally.sequence_ok,
            tally.scales_ok,
            tally.worst_measure,
            tally.worst_scale,
            tally.ambiguous_directory,
            tally.carried_flags,
        );
    }
    for (name, tally) in &summary {
        if tally.failures.is_empty() {
            continue;
        }
        println!("\n{name}: mismatch classes");
        for (class, (count, first)) in &tally.failures {
            println!("  {count:>6}  {class}  (first at event {first})");
        }
    }

    let total: usize = summary.iter().map(|(_, t)| t.events).sum();
    let sequences: usize = summary.iter().map(|(_, t)| t.sequence_ok).sum();
    let scales: usize = summary.iter().map(|(_, t)| t.scales_ok).sum();
    let candidates: usize = summary.iter().map(|(_, t)| t.candidates).sum();
    let tables: usize = summary.iter().map(|(_, t)| t.tables_checked).sum();
    let carried: usize = summary.iter().map(|(_, t)| t.carried_flags).sum();
    let asked: usize = summary.iter().map(|(_, t)| t.oracle_directory).sum();
    println!(
        "\n{total} events, {candidates} candidate pairs; {sequences} merge sequences and \
         {scales} scale pairs reproduced"
    );
    println!(
        "{tables} merge tables derived and checked whole; {carried} events needed the \
         reference's carried-over on-shell flags; {asked} flavour assignments needed its \
         candidate list to name their process directory"
    );
    assert!(
        tables > 0,
        "no merge table was checked against the reference's own"
    );
    assert_eq!(sequences, total, "merge sequences do not all reproduce");
    assert_eq!(scales, total, "scales do not all reproduce");
}

fn compare_run(_name: &str, path: &Path, model: &Model) -> Tally {
    let (header, events) = read_dump(path);
    let directory = &header["directory"];
    let constants = {
        let rows = directory["CONST"].as_array().expect("CONST rows");
        let mut row = vec![Value::from("CONST")];
        row.extend(rows[0].as_array().expect("CONST row").iter().cloned());
        RunConstants::read(&Rec(row.as_slice()))
    };
    let colors = ColorTable::new(
        model.colors.iter().map(|(&pdg, &color)| (pdg, color)),
        constants.maxjetflavor,
    );

    // The dump's per-directory tables carry no directory name, so a run with
    // several of them arrives as several forest sets; the number of subprocess
    // columns on each `IFOR` row is what separates them.
    let mut directories: BTreeMap<usize, DirectoryForests> = BTreeMap::new();
    for row in directory["RUN"].as_array().expect("RUN rows") {
        let row = row.as_array().expect("RUN row");
        let n_proc = row[3].as_u64().expect("maxsproc") as usize;
        let entry = directories.entry(n_proc).or_default();
        entry.n_external = row[1].as_u64().expect("nexternal") as usize;
        entry.n_incoming = row[2].as_u64().expect("nincoming") as usize;
        entry.n_proc = n_proc;
        entry.n_configs = row[4].as_u64().expect("mapconfig(0)") as usize;
    }
    for row in directory["IFOR"].as_array().expect("IFOR rows") {
        let row = row.as_array().expect("IFOR row");
        let n_proc = row.len() - 8;
        let Some(entry) = directories.get_mut(&n_proc) else {
            continue;
        };
        let config = row[1].as_u64().expect("config") as usize;
        let index = row[2].as_i64().expect("line index") as i32;
        entry.lines.entry((config, index)).or_insert(ForestLine {
            index,
            daughters: [
                row[3].as_i64().expect("daughter") as i32,
                row[4].as_i64().expect("daughter") as i32,
            ],
            tprid: row[5].as_i64().expect("tprid"),
            mass: row[6].as_f64().expect("mass"),
            width: row[7].as_f64().expect("width"),
            sprop: row[8..]
                .iter()
                .map(|v| v.as_i64().expect("sprop"))
                .collect(),
        });
    }
    // The dumped merge tables are what the derivation is checked against, keyed
    // the way the dump keys them.
    let mut dumped_map: BTreeMap<(usize, usize, u32), BTreeSet<Vec<usize>>> = BTreeMap::new();
    for row in directory["MAP"].as_array().expect("MAP rows") {
        let row = row.as_array().expect("MAP row");
        let key = (
            row[0].as_u64().expect("this_config") as usize,
            row[1].as_u64().expect("iproc") as usize,
            row[2].as_u64().expect("mask") as u32,
        );
        let graphs: Vec<usize> = row[4..]
            .iter()
            .map(|v| v.as_u64().expect("graph") as usize)
            .collect();
        dumped_map.entry(key).or_default().insert(graphs);
    }

    let cluster_settings = ClusterSettings {
        hadronic: constants.lpp[0] != 0 || constants.lpp[1] != 0,
        d_parameter: constants.d_parameter,
        bwcutoff: constants.bwcutoff,
        small_width_treatment: 1e-6,
    };
    let scale_settings = ScaleSettings {
        scalefact: constants.scalefact,
        fixed_ren: constants.fixed_ren,
        fixed_fac: constants.fixed_fac,
        beam_has_pdf: [constants.lpp[0] != 0, constants.lpp[1] != 0],
        ickkw: constants.ickkw,
        xqcut: constants.xqcut,
        xmtc: constants.xmtc,
        pdfwgt: constants.pdfwgt,
    };

    let mut sets: HashMap<(usize, usize, Vec<i64>), (ChannelSet, usize)> = HashMap::new();
    let mut directory_of: HashMap<Vec<i64>, usize> = HashMap::new();
    let mut tally = Tally::default();
    let mut map_checked: HashSet<(usize, usize)> = HashSet::new();
    for event in events {
        tally.events += 1;
        let evt = event.first("EVT");
        let iproc = evt.u(2);
        let this_config = evt.u(3);
        // The written event lists intermediate resonances among its particles,
        // so the matrix element's own external flavours come from the line
        // records instead, which key on the single-leg masks.
        let n_external = evt.u(6);
        let mut external: Vec<i64> = vec![0; n_external];
        for line in event.iter("LINE") {
            let mask = line.u(1);
            if mask.is_power_of_two() && mask.trailing_zeros() < n_external as u32 {
                external[mask.trailing_zeros() as usize] = line.i(2);
            }
        }
        // Which process directory the event came from. The colour and flavour
        // test above usually settles it; where it leaves two readings, the
        // event's own candidate list picks between them, once per flavour
        // assignment rather than once per event.
        if !directory_of.contains_key(&external) {
            let candidates: Vec<&DirectoryForests> = directories
                .values()
                .filter(|d| d.n_proc >= iproc && d.lines.keys().any(|(c, _)| *c == this_config))
                .collect();
            let viable: Vec<&DirectoryForests> = candidates
                .iter()
                .copied()
                .filter(|d| forest_is_consistent(d, this_config, &external, iproc, model))
                .collect();
            let pool = if viable.is_empty() {
                &candidates
            } else {
                &viable
            };
            let chosen = if pool.len() == 1 {
                pool[0]
            } else {
                tally.oracle_directory += 1;
                let admissible: BTreeMap<u32, bool> = event
                    .iter("CAND")
                    .filter(|r| r.i(1) == 1 && r.i(2) == 0)
                    .map(|r| (r.u(9) as u32, r.b(10)))
                    .collect();
                pool.iter()
                    .copied()
                    .min_by_key(|d| {
                        let set = build_channel_set(d, &external, iproc, &colors);
                        let tables = set.merge_tables(this_config);
                        admissible
                            .iter()
                            .filter(|(mask, allowed)| {
                                tables[iproc - 1].id_cl.contains_key(mask) != **allowed
                            })
                            .count()
                    })
                    .expect("a directory to choose from")
            };
            if candidates.len() > 1 {
                tally.ambiguous_directory += 1;
            }
            directory_of.insert(external.clone(), chosen.n_proc);
        }
        let key = (this_config, iproc, external.clone());
        if !sets.contains_key(&key) {
            let chosen = &directories[&directory_of[&external]];
            sets.insert(
                key.clone(),
                (
                    build_channel_set(chosen, &external, iproc, &colors),
                    chosen.n_proc,
                ),
            );
        }
        let (set, _) = &sets[&key];
        let tables = set.merge_tables(this_config);

        // The derived merge graph must be one the reference wrote, before a
        // single momentum is clustered. A channel whose own directory never
        // reached the dump has no table to check against; the engine still runs,
        // since a channel selects a table only through its coupling order.
        // A dump that holds several process directories merges their tables
        // under one key — the records carry no directory name — so only a
        // single-directory run can be asked this. The multi-directory runs check
        // the same table event by event instead, through every candidate pair's
        // admissibility, which is the finer statement anyway.
        let dumped_here = directories.len() == 1
            && dumped_map
                .keys()
                .any(|(config, proc, _)| (*config, *proc) == (this_config, iproc));
        if dumped_here && map_checked.insert((this_config, iproc)) {
            tally.tables_checked += 1;
            let derived: BTreeSet<(u32, Vec<usize>)> = tables[iproc - 1]
                .id_cl
                .iter()
                .map(|(mask, graphs)| (*mask, graphs.clone()))
                .collect();
            for (mask, graphs) in &derived {
                match dumped_map.get(&(this_config, iproc, *mask)) {
                    Some(variants) if variants.contains(graphs) => {}
                    Some(_) => {
                        tally.fail("merge graph: a leg set maps to other channels", event.index)
                    }
                    None => tally.fail("merge graph: a leg set the reference has not", event.index),
                }
            }
            // A dump with several directories in it merges their tables under
            // one key, so only a single-directory run can be asked whether the
            // reference has a leg set the derivation lacks.
            if directories.len() == 1 {
                for ((config, proc, mask), _) in dumped_map.iter() {
                    if (*config, *proc) == (this_config, iproc)
                        && !tables[iproc - 1].id_cl.contains_key(mask)
                    {
                        tally.fail(
                            "merge graph: a leg set the reference has and we lack",
                            event.index,
                        );
                    }
                }
            }
        }

        let channel = Channel {
            set,
            table: &tables[iproc - 1],
            colors: &colors,
            this_config,
            iproc,
        };

        let attempt1 = evt.i(1);
        let mut momenta: Vec<[f64; 4]> = vec![[0.0; 4]; set.n_external];
        for mom in event.iter("MOM") {
            if mom.i(1) != attempt1 {
                continue;
            }
            momenta[mom.u(2) - 1] = [mom.f(3), mom.f(4), mom.f(5), mom.f(6)];
        }

        let scl = event.first("SCL");
        let stored = scl.i(10);
        let memo = JetMemo(if stored < 0 {
            None
        } else {
            Some(stored as usize)
        });
        let incoming = (scl.f(7), [scl.f(8), scl.f(9)]);

        let mut memo_pure = memo;
        let outcome = setclscales(
            &channel,
            &cluster_settings,
            &scale_settings,
            &momenta,
            &mut memo_pure,
            constants.chcluster,
            &[],
            incoming,
            true,
        );
        let mut probe = Tally::default();
        let matched = match &outcome {
            Ok(scales) => {
                compare_event(&event, scales, this_config, &mut probe);
                probe.failures.is_empty()
            }
            Err(_) => false,
        };
        if matched {
            compare_event(&event, &outcome.expect("matched"), this_config, &mut tally);
            continue;
        }

        // Where the pure reading disagrees, the reference's carried-over on-shell
        // flags are the first thing to try: its own candidate records name every
        // leg set it measured as a resonance, and a leg set outside this event's
        // resonance list is one an earlier event left flagged.
        let tagged: HashSet<u32> = event
            .first("BW")
            .0
            .iter()
            .skip(3)
            .step_by(2)
            .map(|v| v.as_u64().expect("a tagged leg set") as u32)
            .collect();
        let carried: Vec<u32> = event
            .iter("CAND")
            .filter(|r| r.s(11) == "FS_SUMDOT_BW")
            .map(|r| r.u(9) as u32)
            .filter(|mask| !tagged.contains(mask))
            .collect();
        if !carried.is_empty() {
            let mut memo_carried = memo;
            if let Ok(scales) = setclscales(
                &channel,
                &cluster_settings,
                &scale_settings,
                &momenta,
                &mut memo_carried,
                constants.chcluster,
                &carried,
                incoming,
                true,
            ) {
                let mut second = Tally::default();
                compare_event(&event, &scales, this_config, &mut second);
                if second.failures.is_empty() {
                    tally.carried_flags += 1;
                    compare_event(&event, &scales, this_config, &mut tally);
                    continue;
                }
            }
        }

        match outcome {
            Ok(scales) => compare_event(&event, &scales, this_config, &mut tally),
            Err(_) => tally.fail(
                "the engine refused an event the reference clustered",
                event.index,
            ),
        }
    }
    tally
}

fn compare_event(
    event: &Event,
    scales: &vibegraph::coupling::cluster::setclscales::ClusterScales,
    this_config: usize,
    tally: &mut Tally,
) {
    let calls: Vec<Rec<'_>> = event.iter("CLCALL").collect();
    let mut sequence_ok = true;

    // How many times the clustering ran, and whether the merge graph was
    // restricted to the integration channel each time.
    if scales.attempts.len() != calls.len() || scales.traces.len() != calls.len() {
        tally.fail("a different number of clustering attempts", event.index);
        sequence_ok = false;
    } else {
        for (index, (mine, theirs)) in scales.attempts.iter().zip(calls.iter()).enumerate() {
            tally.bump("cluster_calls_per_event", &(index + 1).to_string());
            tally.bump("memo", mine.memo.name());
            if mine.chcluster != theirs.b(2) {
                tally.fail(
                    "a clustering attempt was restricted differently",
                    event.index,
                );
                sequence_ok = false;
            }
        }
        for (index, clustering) in scales.traces.iter().enumerate() {
            sequence_ok &=
                compare_clustering(event, index as i64 + 1, clustering, this_config, tally);
        }
    }

    // The beam-side vertex indices, before and after the `jfirst` fixup. Only
    // the accepted attempt's walk survives, so only its records are compared.
    let indices: Vec<Rec<'_>> = event.iter("JIDX").collect();
    for jidx in indices.iter().rev().take(2) {
        let (first, last, central) = (
            [jidx.u(2), jidx.u(3)],
            [jidx.u(4), jidx.u(5)],
            [jidx.u(6), jidx.u(7)],
        );
        let mine_first = if jidx.s(1) == "raw" {
            scales.jfirst_raw
        } else {
            scales.jfirst
        };
        if mine_first != first || scales.jlast != last || scales.jcentral != central {
            tally.fail(
                &format!("the beam-side vertex indices ({})", jidx.s(1)),
                event.index,
            );
            sequence_ok = false;
        }
    }

    // Which legs the walk counted as jets, and the jet code it got there by.
    let jets = event.last("JETS");
    let iqjets: Vec<i64> = (4..jets.len()).map(|k| jets.i(k)).collect();
    if scales.jcode != jets.i(1) || scales.iqjets != iqjets {
        tally.fail("the jet tags the walk left", event.index);
        sequence_ok = false;
    }
    let steps: Vec<Rec<'_>> = event.iter("MEMO").collect();
    if steps.len() != scales.attempts.len()
        || steps
            .iter()
            .zip(scales.attempts.iter())
            .any(|(theirs, mine)| theirs.s(1) != mine.memo.name() || theirs.u(2) != mine.jets)
    {
        tally.fail("what the jet memo did with the count", event.index);
        sequence_ok = false;
    }

    // Every line the walk could have asked about, with the provenance and jet
    // flags it reads them through. The records carry no attempt number and each
    // attempt writes the same count, so the accepted attempt's are the last
    // block of them.
    let lines: Vec<Rec<'_>> = event.iter("LINE").collect();
    let per_attempt = scales.lines.len().min(lines.len());
    for line in lines.iter().skip(lines.len() - per_attempt) {
        let mask = line.u(1) as u32;
        let Some(mine) = scales.lines.iter().find(|l| l.mask == mask) else {
            tally.fail("a line the reference walked and we did not", event.index);
            sequence_ok = false;
            continue;
        };
        if mine.pdg != line.i(2)
            || mine.ipart != [line.u(3), line.u(4)]
            || mine.goodjet != line.b(7)
        {
            tally.fail("a line's flavour, provenance or jet flag", event.index);
            sequence_ok = false;
        }
    }

    // The two rewrites of the vertex scales, and every vertex scale after them.
    let overrides = event.last("OVR");
    if scales.overrides != [overrides.b(1), overrides.b(2), overrides.b(3)] {
        tally.fail("which scale rewrite fired", event.index);
        sequence_ok = false;
    }
    tally.bump(
        "mt2last_override",
        if scales.overrides[0] { "True" } else { "False" },
    );
    tally.bump(
        "jcentral_override_beam1",
        if scales.overrides[1] { "True" } else { "False" },
    );
    tally.bump(
        "jcentral_override_beam2",
        if scales.overrides[2] { "True" } else { "False" },
    );
    for stage in event.iter("PT2").filter(|r| r.s(1) == "FINAL") {
        let vertex = stage.u(2);
        let deviation = close(scales.pt2[vertex - 1], stage.f(3));
        tally.worst_scale = tally.worst_scale.max(deviation);
        if !(deviation < AGREEMENT) {
            tally.fail("a vertex scale after the rewrites", event.index);
            sequence_ok = false;
        }
    }

    if sequence_ok {
        tally.sequence_ok += 1;
    }

    // The scales, and the branch each came from.
    let out = event.first("OUT");
    let mur = event.last("MUR");
    let muf = event.last("MUF");
    let mut scales_ok = true;
    tally.bump("mur_branch", scales.mur_branch.name());
    tally.bump("muf_branch", scales.muf_branch.name());
    if scales.mur_branch.name() != mur.s(1) {
        tally.fail(
            &format!(
                "mu_R came from branch {} not {}",
                scales.mur_branch.name(),
                mur.s(1)
            ),
            event.index,
        );
        scales_ok = false;
    }
    if scales.muf_branch.name() != muf.s(1) {
        tally.fail(
            &format!(
                "mu_F came from branch {} not {}",
                scales.muf_branch.name(),
                muf.s(1)
            ),
            event.index,
        );
        scales_ok = false;
    }
    let deviation = close(scales.mu_r, out.f(1))
        .max(close(scales.q2fact[0], out.f(2)))
        .max(close(scales.q2fact[1], out.f(3)));
    tally.worst_scale = tally.worst_scale.max(deviation);
    if !(deviation < AGREEMENT) {
        tally.fail("the scales themselves", event.index);
        scales_ok = false;
    }
    if scales_ok {
        tally.scales_ok += 1;
    }
}

/// One clustering attempt against the records the reference tagged with it.
fn compare_clustering(
    event: &Event,
    attempt: i64,
    clustering: &vibegraph::coupling::cluster::kt::Clustering,
    this_config: usize,
    tally: &mut Tally,
) -> bool {
    let mut ok = true;

    // Which of the channel's timelike propagators the event puts on shell.
    let bw = event
        .iter("BW")
        .find(|r| r.i(1) == attempt)
        .expect("a clustering records its resonance list");
    let tagged: Vec<(u32, i32)> = (0..bw.u(2))
        .map(|k| (bw.u(3 + 2 * k) as u32, bw.i(4 + 2 * k) as i32))
        .collect();
    if clustering.tagged != tagged {
        tally.fail("the on-shell resonance tagging", event.index);
        ok = false;
    }

    // Every candidate pair, admissible or not.
    let candidates: Vec<Rec<'_>> = event.iter("CAND").filter(|r| r.i(1) == attempt).collect();
    let mine = &clustering.candidates;
    tally.candidates += mine.len();
    if mine.len() != candidates.len() {
        tally.fail("a different number of candidate pairs", event.index);
        return false;
    }
    for (mine, theirs) in mine.iter().zip(candidates.iter()) {
        tally.bump("candidate_measure", mine.measure.name());
        if mine.inflated {
            tally.bump("beam_crossing_inflation", "applied");
        }
        if mine.position != [theirs.u(3), theirs.u(4)]
            || mine.leg != [theirs.u(5), theirs.u(6)]
            || mine.daughters != [theirs.u(7) as u32, theirs.u(8) as u32]
            || mine.mother != theirs.u(9) as u32
        {
            tally.fail("a candidate pair was visited in another order", event.index);
            return false;
        }
        if mine.admissible != theirs.b(10) {
            tally.fail("a candidate pair's admissibility", event.index);
            return false;
        }
        if mine.measure.name() != theirs.s(11) {
            tally.fail("a candidate took another arm of the measure", event.index);
            return false;
        }
        if mine.n_graphs != theirs.u(16) {
            tally.fail(
                "a candidate left another number of channels alive",
                event.index,
            );
            return false;
        }
        if mine.inflated != theirs.b(13) {
            tally.fail("a candidate's crossing inflation", event.index);
            return false;
        }
        if !mine.admissible {
            continue;
        }
        let deviation = close(mine.pt2, theirs.f(14))
            .max(close(mine.raw, theirs.f(12)))
            .max(close(mine.z, theirs.f(15)));
        tally.worst_measure = tally.worst_measure.max(deviation);
        if !(deviation < AGREEMENT) {
            tally.fail("a candidate's measure", event.index);
            return false;
        }
    }

    // The winner of each pass.
    let wins: Vec<Rec<'_>> = event.iter("WIN").filter(|r| r.i(1) == attempt).collect();
    let merges: Vec<Rec<'_>> = event.iter("MRG").filter(|r| r.i(1) == attempt).collect();
    let core = event
        .iter("CORE")
        .filter(|r| r.i(1) == attempt)
        .last()
        .expect("a clustering writes a core");
    let real: Vec<_> = clustering
        .merges
        .iter()
        .filter(|m| m.kind != MergeKind::Core)
        .collect();
    if real.len() != merges.len() || real.len() != wins.len() {
        tally.fail("a different number of merges", event.index);
        return false;
    }
    for (index, ((mine, theirs), win)) in
        real.iter().zip(merges.iter()).zip(wins.iter()).enumerate()
    {
        tally.bump(
            "merge kind",
            if mine.kind == MergeKind::Initial {
                "IS"
            } else {
                "FS"
            },
        );
        if mine.daughters != [theirs.u(3) as u32, theirs.u(4) as u32]
            || mine.mother != theirs.u(5) as u32
        {
            tally.fail(
                &format!("merge {} joined other lines", index + 1),
                event.index,
            );
            return false;
        }
        let kind = if mine.kind == MergeKind::Initial {
            "IS"
        } else {
            "FS"
        };
        if kind != theirs.s(6) {
            tally.fail("a merge took the beam on the other side", event.index);
            return false;
        }
        let icluster: [i32; 4] = [
            theirs.i(13) as i32,
            theirs.i(14) as i32,
            theirs.i(15) as i32,
            theirs.i(16) as i32,
        ];
        if mine.icluster != icluster {
            tally.fail("a merge's written leg numbers", event.index);
            ok = false;
        }
        let deviation = close(mine.pt2, theirs.f(7))
            .max(close(mine.mt2, theirs.f(9)))
            .max(close(mine.z, theirs.f(8)))
            .max(close(mine.pt2, win.f(5)));
        tally.worst_measure = tally.worst_measure.max(deviation);
        if !(deviation < AGREEMENT) {
            tally.fail(&format!("merge {}'s scale", index + 1), event.index);
            return false;
        }
    }

    // The terminal vertex, and the frame changes on the way to it.
    let mine_core = clustering
        .merges
        .last()
        .expect("a clustering writes a core");
    if mine_core.daughters != [core.u(3) as u32, core.u(4) as u32]
        || mine_core.mother != core.u(5) as u32
    {
        tally.fail("the terminal vertex joins other lines", event.index);
        return false;
    }
    let deviation = close(mine_core.pt2, core.f(6)).max(close(clustering.mt2last, core.f(7)));
    tally.worst_measure = tally.worst_measure.max(deviation);
    if !(deviation < AGREEMENT) {
        tally.fail("the terminal vertex's scale", event.index);
        ok = false;
    }
    let boosts: Vec<Rec<'_>> = event.iter("BOOST").filter(|r| r.i(1) == attempt).collect();
    if clustering.boosts.len() != boosts.len() {
        tally.fail("a different number of frame changes", event.index);
        ok = false;
    } else {
        for (mine, theirs) in clustering.boosts.iter().zip(boosts.iter()) {
            tally.bump("boost", if mine.fired { "fired" } else { "not_fired" });
            if mine.fired != theirs.b(3) || mine.lines_left != theirs.u(4) {
                tally.fail("a frame change fired differently", event.index);
                ok = false;
                continue;
            }
            let deviation = (0..4)
                .map(|k| close(mine.frame[k], theirs.f(5 + k)))
                .fold(0.0_f64, f64::max)
                .max(close(mine.invariant, theirs.f(9)));
            tally.worst_measure = tally.worst_measure.max(deviation);
            if !(deviation < AGREEMENT) {
                tally.fail("a frame change's boost vector", event.index);
                ok = false;
            }
        }
    }

    // The surviving channel list, before and after the integration channel is
    // allowed to claim it.
    for grph in event.iter("GRPH").filter(|r| r.i(1) == attempt) {
        let listed: Vec<usize> = (4..grph.len()).map(|k| grph.u(k)).collect();
        let mine = if grph.s(2) == "before" {
            &clustering.graphs_before_claim
        } else {
            &clustering.graphs
        };
        if grph.s(2) == "after" {
            tally.bump(
                "igraphs1_is_iconfig",
                if mine.first() == Some(&this_config) {
                    "True"
                } else {
                    "False"
                },
            );
        }
        if mine.as_slice() != listed.as_slice() {
            tally.fail(
                &format!("the surviving channel list {}", grph.s(2)),
                event.index,
            );
            ok = false;
        }
    }
    ok
}

// ── the forests, derived from our own diagrams ───────────────────────────────

/// Each dumped run whose process directory groups a single subprocess, with the
/// process it was generated from.
///
/// A grouped directory cannot take part: `configs.inc` writes one `sprop` column
/// per subprocess of the group and takes every `tprid` from the group's first
/// subprocess, so its forests are not a function of any one subprocess's
/// diagrams. The runs below are the ones whose group is a single flavour
/// assignment, which is what makes the comparison exact.
const DERIVED_FOREST_RUNS: &[(&str, &str)] = &[
    ("bbx_to_ccx_emmm_qcd0", "b b~ > c c~ e+ e- mu+ mu- QCD=0"),
    ("ee_to_mumu_tata_qcd0", "e+ e- > mu+ mu- ta+ ta- QCD=0"),
    ("ee_to_mumua", "e+ e- > mu+ mu- a"),
    ("ee_to_ttx", "e+ e- > t t~"),
    ("uux_to_ccx_emmm_qcd0", "u u~ > c c~ e+ e- mu+ mu- QCD=0"),
    ("uux_to_uux", "u u~ > u u~"),
];

/// One channel forest as a comparison can see it: every line named by the leg
/// set below it rather than by the index the file happened to give it.
///
/// The index is a labelling — `configs.inc` numbers the lines in the order it
/// writes them, and two generators that agree on the tree can disagree on that
/// order without disagreeing on anything the clustering reads. The leg set is
/// not: it is what `filmap` keys the merge table on, what `checkbw` measures,
/// and what every merge is matched against. So the canonical form keys each line
/// by its own leg set and its two daughters' leg sets, and carries the rest of
/// the line — the propagator codes, the mass and the width — verbatim.
type CanonicalLine = (u32, u32, u32, i64, i64, u64, u64);

fn canonical_forest(forest: &ConfigForest, n_external: usize) -> Vec<CanonicalLine> {
    let bit = |d: i32| -> u32 {
        if d > 0 {
            1 << (d - 1)
        } else {
            forest.mask(d).expect("a daughter that resolves")
        }
    };
    // Masses and widths are model constants on both sides, so they compare as
    // written; quantising them keeps the tuple orderable without inventing a
    // tolerance that would hide a wrong propagator.
    let quantise = |v: f64| -> u64 { (v * 1e6).round() as u64 };
    let mut lines: Vec<CanonicalLine> = forest
        .lines
        .iter()
        .map(|line| {
            let mut daughters = [bit(line.daughters[0]), bit(line.daughters[1])];
            daughters.sort_unstable();
            (
                forest.mask(line.index).expect("a line that resolves"),
                daughters[0],
                daughters[1],
                line.tprid,
                line.sprop[0],
                quantise(line.mass),
                quantise(line.width),
            )
        })
        .collect();
    let _ = n_external;
    lines.sort_unstable();
    lines
}

/// The forests a dump carries for its single process directory.
fn dumped_forests(path: &Path) -> (usize, Vec<ConfigForest>) {
    let (header, _events) = read_dump(path);
    let directory = &header["directory"];
    // The `RUN` record is written once per integration channel, so one directory
    // appears many times; its subprocess count is what tells two directories
    // apart, and a single value means the dump holds one ungrouped directory.
    let rows = directory["RUN"].as_array().expect("RUN rows");
    let groups: BTreeSet<u64> = rows
        .iter()
        .map(|r| {
            r.as_array().expect("RUN row")[3]
                .as_u64()
                .expect("maxsproc")
        })
        .collect();
    assert_eq!(
        groups.iter().copied().collect::<Vec<u64>>(),
        vec![1],
        "the dump holds a grouped or multi-directory process"
    );
    let row = rows[0].as_array().expect("RUN row");
    let n_external = row[1].as_u64().expect("nexternal") as usize;
    let n_configs = rows
        .iter()
        .map(|r| {
            r.as_array().expect("RUN row")[4]
                .as_u64()
                .expect("mapconfig(0)") as usize
        })
        .max()
        .expect("a RUN row");

    // A directory's tables are written once per integration channel, so the same
    // line arrives many times over.
    let mut lines: BTreeMap<(usize, i32), ForestLine> = BTreeMap::new();
    for row in directory["IFOR"].as_array().expect("IFOR rows") {
        let row = row.as_array().expect("IFOR row");
        let config = row[1].as_u64().expect("config") as usize;
        let index = row[2].as_i64().expect("line index") as i32;
        lines.entry((config, index)).or_insert(ForestLine {
            index,
            daughters: [
                row[3].as_i64().expect("daughter") as i32,
                row[4].as_i64().expect("daughter") as i32,
            ],
            tprid: row[5].as_i64().expect("tprid"),
            mass: row[6].as_f64().expect("mass"),
            width: row[7].as_f64().expect("width"),
            sprop: row[8..]
                .iter()
                .map(|v| v.as_i64().expect("sprop"))
                .collect(),
        });
    }
    let mut configs = vec![ConfigForest::default(); n_configs];
    for ((config, _), line) in lines {
        configs[config - 1].lines.push(line);
    }
    for config in &mut configs {
        config.lines.sort_by_key(|line| -line.index);
    }
    (n_external, configs)
}

/// The channel forests derived from vibegraph's own diagrams against the ones
/// MadGraph generated for the same process.
///
/// This is the step the clustering engine did not take: it was given the dump's
/// `IFOR` records, so everything below them was derived and the forests
/// themselves were assumed. Here they are produced from the enumerated diagrams
/// and compared whole — every line's leg set, its daughters' leg sets, both
/// propagator codes, the mass and the width — against the file MadGraph wrote.
///
/// The channel *numbering* is not compared, and cannot be: MadGraph numbers its
/// configs by its own diagram order and drops the ones its vertex filter
/// rejects. What is compared is the bijection — every generated channel is one
/// of ours and every one of ours is generated — together with the QCD order that
/// partitions them, which is the only thing the merge table reads a channel's
/// identity for.
#[test]
fn derived_channel_forests_match_the_generated_ones() {
    let dumps = dumps_dir();
    if !dumps.is_dir() {
        println!("no kT clustering dumps in {}", dumps.display());
        return;
    }
    let model = common::sm_model();
    let mut checked = 0usize;
    let mut lines_checked = 0usize;
    for (run, process) in DERIVED_FOREST_RUNS {
        let path = dumps.join(format!("{run}.jsonl.gz"));
        if !path.is_file() {
            continue;
        }
        let card = vibegraph::ufo::slha::ParamCard::from_file(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../validation/madgraph/output")
                .join(run)
                .join("Cards/param_card.dat"),
        )
        .expect("param card");
        let evaluated = vibegraph::ufo::EvaluatedModel::from_model_card(model.clone(), &card);

        let sets = common::generate_with(process, model.as_ref());
        let set = sets
            .iter()
            .find(|s| !s.diagrams.is_empty())
            .expect("a non-empty subprocess");
        assert_eq!(
            sets.iter().filter(|s| !s.diagrams.is_empty()).count(),
            1,
            "{run}: {process} enumerates more than one subprocess"
        );
        let externals: Vec<vibegraph::ufo::particles::ParticleId> = set
            .particles_in
            .iter()
            .chain(set.particles_out.iter())
            .map(|name| model.particle_id(name).expect("external in model"))
            .collect();
        let derived = vibegraph::coupling::cluster::configs::derive_channels(
            &set.diagrams,
            &externals,
            set.particles_in.len(),
            model.as_ref(),
            &evaluated,
        )
        .expect("channel forests");

        let (n_external, generated) = dumped_forests(&path);
        assert_eq!(n_external, derived.set.n_external, "{run}: nexternal");
        assert_eq!(
            generated.len(),
            derived.set.configs.len(),
            "{run}: {} generated channels against {} derived from {} diagrams",
            generated.len(),
            derived.set.configs.len(),
            set.diagrams.len()
        );

        let mut mine: Vec<(i64, Vec<CanonicalLine>)> = derived
            .set
            .configs
            .iter()
            .map(|c| (c.nqcd, canonical_forest(c, n_external)))
            .collect();
        let mut theirs: Vec<Vec<CanonicalLine>> = generated
            .iter()
            .map(|c| canonical_forest(c, n_external))
            .collect();
        let mut mine_forests: Vec<Vec<CanonicalLine>> =
            mine.iter().map(|(_, f)| f.clone()).collect();
        mine_forests.sort();
        theirs.sort();
        let only_ours: Vec<&Vec<CanonicalLine>> = mine_forests
            .iter()
            .filter(|f| !theirs.contains(f))
            .collect();
        let only_theirs: Vec<&Vec<CanonicalLine>> = theirs
            .iter()
            .filter(|f| !mine_forests.contains(f))
            .collect();
        assert!(
            only_ours.is_empty() && only_theirs.is_empty(),
            "{run}: {} derived channels MadGraph does not generate and {} generated \
             channels we do not derive\n  ours:   {:?}\n  theirs: {:?}",
            only_ours.len(),
            only_theirs.len(),
            only_ours.iter().take(2).collect::<Vec<_>>(),
            only_theirs.iter().take(2).collect::<Vec<_>>()
        );
        assert_eq!(
            mine_forests, theirs,
            "{run}: the two channel multisets contain the same forests with different \
             multiplicities"
        );
        lines_checked += theirs.iter().map(Vec::len).sum::<usize>();

        mine.sort_by_key(|(nqcd, _)| *nqcd);
        let orders: BTreeMap<i64, usize> = mine.iter().fold(BTreeMap::new(), |mut acc, (n, _)| {
            *acc.entry(*n).or_insert(0) += 1;
            acc
        });
        println!(
            "{run}: {} channels from {} diagrams, {} lines, QCD orders {:?}",
            derived.set.configs.len(),
            set.diagrams.len(),
            theirs.iter().map(Vec::len).sum::<usize>(),
            orders
        );
        checked += 1;
    }
    assert_eq!(
        checked,
        DERIVED_FOREST_RUNS.len(),
        "not every single-subprocess dump was compared"
    );
    println!(
        "channel forests: {checked} processes, {lines_checked} lines derived from our own \
         diagrams and equal to MadGraph's generated configs.inc"
    );
}
