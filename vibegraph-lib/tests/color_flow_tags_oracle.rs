//! Colour-flow tag oracle: validate the derived Les Houches `ICOLUP` table
//! against MadGraph's own `leshouche.inc` for every generated subprocess.
//!
//! For each `SubProcesses/P*/leshouche.inc` this parses MadGraph's
//! `ICOLUP(1|2, leg, iflow, isproc)` rows of **every** subprocess the file
//! carries — `isproc N` is the process the sibling `matrix<N>_orig.f`'s
//! `C     Process:` header names — then compiles each of them and compares its
//! [`ColorFlowTags`] flow by flow. One trial per `(P* directory, isproc)`.
//!
//! Reading past `isproc 1` is what reaches the conjugate members: a directory
//! groups `g u > g u` with `g u~ > g u~`, or `u u > u u` with `u u~ > u u~` and
//! `u~ u~ > u~ u~`, and each of them has a table of its own. Each trial also
//! asserts that the compiled subprocess's PDG codes are that `isproc`'s
//! `IDUP(·, 1, isproc)` row, so "we compiled the process MadGraph named" is
//! checked rather than assumed.
//!
//! Two things this deliberately does **not** do:
//!
//! - It does not transcribe MadGraph's table. vibegraph derives each flow's tags
//!   from its own basis key (a `T`/`Tr` chain over the external colour indices);
//!   `leshouche.inc` is only the check. A transcription would agree with MadGraph
//!   by construction and could not detect a basis mislabelling.
//! - It does not compare the colour-line *integers*. Line labels are arbitrary —
//!   any consistent relabelling is the same event — so the comparison is of the
//!   induced connectivity: the set of `(leg, slot)` endpoint pairs that share a
//!   label. Label equality is reported separately, as information.
//!
//! The comparison is **element-wise, per flow index** for every process,
//! including the NCOLOR=6 `g g > g g`: vibegraph's sorted basis keys reproduce
//! MadGraph's per-flow structure comments one for one there (see the CF oracle's
//! ordering cross-check), so the flow indices are directly comparable. That makes
//! this test sensitive to a flow permutation, which |M|² — which contracts the
//! flows away — provably cannot see.
//!
//! Run:
//!   cargo test -p vibegraph-lib --features extended-validation \
//!              --test color_flow_tags_oracle
//!
//! Prerequisites (regenerates the gitignored MG output):
//!   pixi run -e madgraph build-diagrams

mod common;

use libtest_mimic::{Arguments, Failed, Trial};
use std::path::{Path, PathBuf};

use common::leshouche;
use vibegraph::diagrams::DiagramSet;
use vibegraph::helas::eval::AmplitudeEvaluator;

/// A colour line as the comparison sees it: the two `(leg, slot)` endpoints it
/// joins, sorted, with the label discarded.
type Line = [(usize, usize); 2];

/// A short, stable trial name, e.g. `pp_to_bb/P1_gg_bbx#1`.
fn trial_name(path: &Path, isproc: usize) -> String {
    let sub = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("P?");
    let proc = path
        .ancestors()
        .nth(3)
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("?");
    format!("{proc}/{sub}#{isproc}")
}

/// Read the concrete process from the sibling `matrix<isproc>_orig.f`, dropping
/// the MadGraph-only `@N` tag and derived `WEIGHTED` order.
///
/// A missing file is a failure, never a skip: an `isproc` with no matrix file
/// would silently drop a subprocess out of the comparison.
fn process_of(leshouche: &Path, isproc: usize) -> Result<String, String> {
    let matrix = leshouche
        .parent()
        .ok_or("no subprocess directory")?
        .join(format!("matrix{isproc}_orig.f"));
    let content = std::fs::read_to_string(&matrix).map_err(|e| format!("read {matrix:?}: {e}"))?;
    let raw = content
        .lines()
        .find_map(|l| l.split_once("Process:").map(|(_, p)| p.trim().to_string()))
        .ok_or("no 'C     Process:' header")?;
    Ok(raw
        .split_whitespace()
        .filter(|tok| !tok.starts_with('@') && !tok.to_uppercase().starts_with("WEIGHTED"))
        .collect::<Vec<_>>()
        .join(" "))
}

/// The colour lines a tag row induces: `(leg, slot)` endpoint pairs sharing a
/// label. Fails if a label appears other than exactly twice.
fn lines(flow: &[[u32; 2]]) -> Result<Vec<Line>, String> {
    let mut pairs: Vec<Line> = Vec::new();
    let mut open: Vec<(u32, (usize, usize))> = Vec::new();
    for (leg, tags) in flow.iter().enumerate() {
        for (slot, &label) in tags.iter().enumerate() {
            if label == 0 {
                continue;
            }
            match open.iter().position(|(l, _)| *l == label) {
                Some(pos) => {
                    let (_, other) = open.remove(pos);
                    let mut ends: Line = [other, (leg, slot)];
                    ends.sort();
                    pairs.push(ends);
                }
                None => open.push((label, (leg, slot))),
            }
        }
    }
    if !open.is_empty() {
        return Err(format!("colour labels appearing once: {open:?}"));
    }
    pairs.sort();
    Ok(pairs)
}

/// Render a flow's endpoint pairs for a failure message.
fn render(pairs: &[Line]) -> String {
    pairs
        .iter()
        .map(|[a, b]| {
            format!(
                "{}{}—{}{}",
                a.0 + 1,
                if a.1 == 0 { "c" } else { "a" },
                b.0 + 1,
                if b.1 == 0 { "c" } else { "a" }
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Compile `process`'s single concrete subprocess.
fn compile(process: &str) -> Result<AmplitudeEvaluator, String> {
    let sets: Vec<DiagramSet> = common::generate(process);
    let with_diagrams: Vec<&DiagramSet> = sets.iter().filter(|s| !s.diagrams.is_empty()).collect();
    if with_diagrams.len() != 1 {
        return Err(format!(
            "expected exactly one non-empty subprocess for '{process}', got {}",
            with_diagrams.len()
        ));
    }
    let model = common::sm_model();
    AmplitudeEvaluator::compile(with_diagrams[0], &model).map_err(|e| format!("compile: {e}"))
}

fn run_trial(path: PathBuf, isproc: usize) -> Result<(), Failed> {
    let process = process_of(&path, isproc)?;
    let subprocess = leshouche::parse(&path)?
        .into_iter()
        .find(|s| s.isproc == isproc)
        .ok_or_else(|| format!("isproc {isproc} vanished from {path:?}"))?;
    let mg = subprocess.flows;
    let eval = compile(&process)?;
    let tags = eval.color_flow_tags();

    // The compiled subprocess is the one MadGraph filed this table under, rather
    // than whatever the header string happened to parse into.
    let model = common::sm_model();
    let ours: Vec<i32> = eval
        .external_particles()
        .iter()
        .map(|&id| model.particle(id).pdg_code as i32)
        .collect();
    let theirs = &subprocess.idup[0];
    if &ours != theirs {
        return Err(format!(
            "'{process}' compiles to PDG codes {ours:?}, but MadGraph files isproc \
             {isproc} under IDUP {theirs:?}"
        )
        .into());
    }

    if tags.n_flows() != mg.len() {
        return Err(format!(
            "flow count mismatch for '{process}': vibegraph {} vs MG {}",
            tags.n_flows(),
            mg.len()
        )
        .into());
    }

    let mut labels_identical = true;
    for (f, mg_flow) in mg.iter().enumerate() {
        let ours = tags.flow(f);
        if ours.len() != mg_flow.len() {
            return Err(format!(
                "'{process}' flow {}: leg count {} vs MG {}",
                f + 1,
                ours.len(),
                mg_flow.len()
            )
            .into());
        }
        let ours_lines = lines(ours)?;
        let mg_lines = lines(mg_flow)?;
        if ours_lines != mg_lines {
            return Err(format!(
                "'{process}' flow {}: colour-line connectivity differs\n  \
                 vibegraph: {}\n  MadGraph:  {}\n  \
                 vibegraph tags: {ours:?}\n  MadGraph tags:  {mg_flow:?}",
                f + 1,
                render(&ours_lines),
                render(&mg_lines)
            )
            .into());
        }
        labels_identical &= ours == mg_flow.as_slice();
    }

    let n_lines = lines(tags.flow(0))?.len();
    println!(
        "  [{}] '{process}' NCOLOR={} lines/flow={n_lines} labels={}",
        trial_name(&path, isproc),
        tags.n_flows(),
        if labels_identical {
            "identical to MG"
        } else {
            "relabelled (connectivity equal)"
        }
    );
    Ok(())
}

fn main() {
    let args = Arguments::from_args();

    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/output");
    let files = leshouche::files_under(&output_dir);
    if files.is_empty() {
        eprintln!("No SubProcesses/P*/leshouche.inc found in validation/madgraph/output/");
        eprintln!("Run: pixi run -e madgraph build-diagrams");
        libtest_mimic::run(&args, vec![]).exit();
    }

    let mut trials: Vec<Trial> = Vec::new();
    for path in files {
        let subprocesses = leshouche::parse(&path)
            .unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
        for sub in subprocesses {
            let name = trial_name(&path, sub.isproc);
            let path = path.clone();
            let isproc = sub.isproc;
            trials.push(Trial::test(name, move || run_trial(path, isproc)));
        }
    }

    libtest_mimic::run(&args, trials).exit();
}
