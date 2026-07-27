//! Colour-flow tag oracle: validate the derived Les Houches `ICOLUP` table
//! against MadGraph's own `leshouche.inc` for every generated subprocess.
//!
//! For each `SubProcesses/P*/leshouche.inc` this parses MadGraph's
//! `ICOLUP(1|2, leg, iflow, isproc)` rows of the first subprocess — the one
//! `matrix1_orig.f`'s `C     Process:` header names, and the one whose colour
//! basis vibegraph reproduces — then compiles the same process and compares its
//! [`ColorFlowTags`] flow by flow.
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

use vibegraph::diagrams::DiagramSet;
use vibegraph::helas::eval::AmplitudeEvaluator;

/// A colour line as the comparison sees it: the two `(leg, slot)` endpoints it
/// joins, sorted, with the label discarded.
type Line = [(usize, usize); 2];

/// Find every `SubProcesses/P*/leshouche.inc` under the MG output tree.
fn find_leshouche_files() -> Vec<PathBuf> {
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/output");
    let mut out = Vec::new();
    collect(&output_dir, &mut out);
    out.sort();
    out
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.file_name().and_then(|n| n.to_str()) == Some("leshouche.inc")
            && path.parent().is_some_and(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with('P'))
            })
        {
            out.push(path);
        }
    }
}

/// A short, stable trial name, e.g. `pp_to_bb/P1_gg_bbx`.
fn trial_name(path: &Path) -> String {
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
    format!("{proc}/{sub}")
}

/// Read the concrete process from the sibling `matrix1_orig.f`, dropping the
/// MadGraph-only `@N` tag and derived `WEIGHTED` order.
fn process_of(leshouche: &Path) -> Result<String, String> {
    let matrix = leshouche
        .parent()
        .ok_or("no subprocess directory")?
        .join("matrix1_orig.f");
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

/// Parse the `ICOLUP` rows of subprocess 1 into `[flow][leg] = [colour, anti]`.
///
/// Rows read `DATA (ICOLUP(k,I,iflow,isproc),I=1, N)/v1,…,vN/` with `k = 1` the
/// colour slot and `k = 2` the anticolour slot.
fn parse_leshouche(path: &Path) -> Result<Vec<Vec<[u32; 2]>>, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("read {path:?}: {e}"))?;
    let mut flows: Vec<Vec<[u32; 2]>> = Vec::new();
    for line in content.lines() {
        let Some(rest) = line.trim_start().strip_prefix("DATA (ICOLUP(") else {
            continue;
        };
        let (args, tail) = rest
            .split_once(')')
            .ok_or_else(|| format!("unterminated ICOLUP subscript: {line}"))?;
        let subscripts: Vec<&str> = args.split(',').map(str::trim).collect();
        if subscripts.len() != 4 {
            return Err(format!("unexpected ICOLUP rank: {line}"));
        }
        let slot: usize = subscripts[0]
            .parse::<usize>()
            .map_err(|_| format!("bad ICOLUP slot: {line}"))?;
        let flow: usize = subscripts[2]
            .parse::<usize>()
            .map_err(|_| format!("bad ICOLUP flow: {line}"))?;
        let subproc: usize = subscripts[3]
            .parse::<usize>()
            .map_err(|_| format!("bad ICOLUP subprocess: {line}"))?;
        if subproc != 1 {
            continue;
        }
        let body = tail
            .split_once('/')
            .and_then(|(_, r)| r.rsplit_once('/').map(|(v, _)| v))
            .ok_or_else(|| format!("no /…/ in ICOLUP line: {line}"))?;
        let values: Vec<u32> = body
            .split(',')
            .map(|t| {
                t.trim()
                    .parse::<u32>()
                    .map_err(|_| format!("bad ICOLUP value '{t}'"))
            })
            .collect::<Result<_, _>>()?;
        if flows.len() < flow {
            flows.resize(flow, Vec::new());
        }
        let row = &mut flows[flow - 1];
        if row.is_empty() {
            row.resize(values.len(), [0; 2]);
        }
        if row.len() != values.len() {
            return Err(format!("ICOLUP leg count changes within a flow: {line}"));
        }
        for (leg, v) in values.into_iter().enumerate() {
            row[leg][slot - 1] = v;
        }
    }
    if flows.iter().any(|f| f.is_empty()) {
        return Err("gap in the ICOLUP flow index".into());
    }
    Ok(flows)
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

fn run_trial(leshouche: PathBuf) -> Result<(), Failed> {
    let process = process_of(&leshouche)?;
    let mg = parse_leshouche(&leshouche)?;
    let eval = compile(&process)?;
    let tags = eval.color_flow_tags();

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
        trial_name(&leshouche),
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

    let files = find_leshouche_files();
    if files.is_empty() {
        eprintln!("No SubProcesses/P*/leshouche.inc found in validation/madgraph/output/");
        eprintln!("Run: pixi run -e madgraph build-diagrams");
        libtest_mimic::run(&args, vec![]).exit();
    }

    let trials: Vec<Trial> = files
        .into_iter()
        .map(|p| {
            let name = trial_name(&p);
            Trial::test(name, move || run_trial(p))
        })
        .collect();

    libtest_mimic::run(&args, trials).exit();
}
