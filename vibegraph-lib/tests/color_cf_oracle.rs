//! CF-matrix oracle: validate vibegraph's exact color-factor matrix against
//! MadGraph's own generated `matrix1_orig.f` for every process under
//! `validation/madgraph/output/`.
//!
//! For each `SubProcesses/P*/matrix1_orig.f` this parses the concrete process
//! (`C     Process:` header), `NCOLOR`, `NGRAPHS`, and the `DATA (CF(I,J)…)`
//! color-matrix block, then runs vibegraph's diagram enumeration +
//! `colorize_process` for the same process and asserts NCOLOR and the full CF
//! matrix agree. MadGraph prints the exact rationals as decimals, so the CF
//! comparison is a float compare against our `Ratio<i64>` cast to `f64` at a
//! tight relative tolerance.
//!
//! This validates the color pipeline (colorize walk → basis → CF matrix)
//! independently of the amplitude evaluator: NCOLOR=1 processes cover the
//! `CF ∈ {1,3,9}` scalar cases, and the QCD `q q~`/`g g` processes are genuine
//! NCOLOR=2 checks.
//!
//! Run:
//!   cargo test -p vibegraph-lib --features extended-validation \
//!              --test color_cf_oracle
//!
//! Prerequisites (regenerates the gitignored MG output):
//!   pixi run -e madgraph build-diagrams

mod common;

use libtest_mimic::{Arguments, Failed, Trial};
use std::path::{Path, PathBuf};

use vibegraph::diagrams::{Diagram, DiagramSet};
use vibegraph::helas::color::{colorize_process, ColorBasis};
use vibegraph::helas::color::{ImmutableString, TensorKind};

/// Relative tolerance for the MadGraph decimal ↔ our-rational CF comparison.
/// MadGraph prints each rational to 16 significant digits, so the only error is
/// the last-digit rounding of both sides to the nearest `f64`.
const CF_REL_TOL: f64 = 1e-14;

/// A parsed `matrix1_orig.f`: the concrete process, its color-matrix dimensions,
/// and the reference CF matrix (row-major, `cf[i*ncolor + j]`) plus the per-flow
/// basis-structure comment strings MadGraph emits after each `DATA (CF...)`.
struct MgReference {
    process: String,
    ncolor: usize,
    ngraphs: usize,
    cf: Vec<f64>,
    /// One MadGraph basis-structure label per flow, e.g. `"T(2,1) T(3,4)"`
    /// (from the `C     1 T(2,1) T(3,4)` comment following each CF column).
    structures: Vec<String>,
}

/// Find every `SubProcesses/P*/matrix1_orig.f` under the MG output tree.
fn find_matrix_files() -> Vec<PathBuf> {
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/output");
    let mut out = Vec::new();
    collect_matrix_files(&output_dir, &mut out);
    out.sort();
    out
}

fn collect_matrix_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            collect_matrix_files(&path, out);
        } else if path.file_name().and_then(|n| n.to_str()) == Some("matrix1_orig.f") {
            out.push(path);
        }
    }
}

/// A short, stable trial name from the process directory and P-subprocess, e.g.
/// `pp_to_bb_qcd2/P1_qq_bbx`.
fn trial_name(matrix_path: &Path) -> String {
    let sub = matrix_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("P?");
    let proc = matrix_path
        .ancestors()
        .nth(3)
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("?");
    format!("{proc}/{sub}")
}

/// Join Fortran fixed-form continuation lines: a physical line whose first
/// non-blank character is `$` (column-6 continuation marker used by the MG
/// exporter) is appended to the running logical line.
fn logical_lines(content: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for raw in content.lines() {
        let trimmed = raw.trim_start();
        if let Some(rest) = trimmed.strip_prefix('$') {
            if let Some(last) = out.last_mut() {
                last.push_str(rest);
                continue;
            }
        }
        out.push(raw.to_string());
    }
    out
}

/// Parse the first `PARAMETER (... NAME=VALUE ...)` integer for `name`.
fn parse_param(lines: &[String], name: &str) -> Option<usize> {
    let needle = format!("{name}=");
    for line in lines {
        if let Some(pos) = line.find(&needle) {
            let after = &line[pos + needle.len()..];
            let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(v) = digits.parse() {
                return Some(v);
            }
        }
    }
    None
}

/// Convert one Fortran real literal (`5.333333333333333D+00`) to `f64`.
fn parse_fortran_real(tok: &str) -> Option<f64> {
    let t = tok.trim().replace(['D', 'd'], "E");
    t.parse().ok()
}

/// Parse a `matrix1_orig.f` into an [`MgReference`].
fn parse_matrix_file(path: &Path) -> Result<MgReference, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("read {path:?}: {e}"))?;
    let lines = logical_lines(&content);

    let process = content
        .lines()
        .find_map(|l| l.split_once("Process:").map(|(_, p)| p.trim().to_string()))
        .ok_or("no 'C     Process:' header")?;

    let ncolor = parse_param(&lines, "NCOLOR").ok_or("no NCOLOR parameter")?;
    let ngraphs = parse_param(&lines, "NGRAPHS").ok_or("no NGRAPHS parameter")?;

    // Parse the CF DATA block. Each column J is one logical line
    //   DATA (CF(I,  J),I=  1,  NCOLOR) /v1, v2, .../
    // optionally followed by a `C     <coeff> <structure>` comment giving the
    // basis structure of flow J.
    let mut cf = vec![f64::NAN; ncolor * ncolor];
    let mut structures = vec![String::new(); ncolor];
    let mut current_col: Option<usize> = None;
    for line in &lines {
        if let Some(rest) = line.trim_start().strip_prefix("DATA (CF(I,") {
            // rest = "  J),I=  1,  N) /v1 ,v2 .../"
            let col: usize = rest
                .split(')')
                .next()
                .and_then(|s| s.trim().parse().ok())
                .ok_or_else(|| format!("bad CF column in: {line}"))?;
            let body = line
                .split_once('/')
                .and_then(|(_, r)| r.rsplit_once('/').map(|(v, _)| v))
                .ok_or_else(|| format!("no /.../ in CF line: {line}"))?;
            let vals: Vec<f64> = body
                .split(',')
                .map(|t| t.trim())
                .filter(|t| !t.is_empty())
                .map(|t| parse_fortran_real(t).ok_or_else(|| format!("bad CF value '{t}'")))
                .collect::<Result<_, _>>()?;
            if vals.len() != ncolor {
                return Err(format!(
                    "CF column {col} has {} values, expected NCOLOR={ncolor}",
                    vals.len()
                ));
            }
            // DATA (CF(I,J),I=1,N) fills column J: CF(I,J) for I=1..N.
            for (i, v) in vals.into_iter().enumerate() {
                cf[i * ncolor + (col - 1)] = v;
            }
            current_col = Some(col - 1);
        } else if let Some(col) = current_col {
            // The structure comment immediately follows the DATA line.
            if let Some(rest) = line.trim_start().strip_prefix('C') {
                let rest = rest.trim();
                // Format: "<coeff> <structure...>", e.g. "1 T(2,1) T(3,4)".
                if let Some((_, structure)) = rest.split_once(char::is_whitespace) {
                    structures[col] = normalize_structure(structure);
                }
            }
            current_col = None;
        }
    }

    if cf.iter().any(|v| v.is_nan()) {
        return Err("CF matrix has unfilled entries".into());
    }
    Ok(MgReference {
        process,
        ncolor,
        ngraphs,
        cf,
        structures,
    })
}

/// Normalise a color-structure label for comparison: drop whitespace so
/// `"T(2,1) T(3,4)"` ↔ `"T(2,1)T(3,4)"`.
fn normalize_structure(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Render one of our basis elements in MadGraph's structure notation
/// (`T(2,1)T(3,4)`, `Tr(1,2,3)`, `ColorOne()`), for the ordering cross-check.
fn render_structure(structure: &ImmutableString) -> String {
    if structure.iter().all(|(k, _)| *k == TensorKind::One) {
        return "ColorOne()".to_string();
    }
    let mut out = String::new();
    for (kind, idxs) in structure {
        if *kind == TensorKind::One {
            continue;
        }
        let name = match kind {
            TensorKind::T => "T",
            TensorKind::Tr => "Tr",
            TensorKind::F => "f",
            TensorKind::D => "d",
            TensorKind::One => unreachable!(),
        };
        let args: Vec<String> = idxs.iter().map(|i| i.to_string()).collect();
        out.push_str(name);
        out.push('(');
        out.push_str(&args.join(","));
        out.push(')');
    }
    out
}

/// Strip MadGraph process-string cruft vibegraph's grammar does not need: the
/// `@N` process tag and the derived `WEIGHTED<=N` order (physical diagrams are
/// fixed by the particles + the QCD/QED constraints, which are kept).
fn clean_process(raw: &str) -> String {
    raw.split_whitespace()
        .filter(|tok| !tok.starts_with('@') && !tok.to_uppercase().starts_with("WEIGHTED"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Enumerate `process` and colorize its single concrete subprocess.
fn colorize(process: &str) -> Result<ColorBasis, String> {
    let sets: Vec<DiagramSet> = common::generate(process);
    let with_diagrams: Vec<&Vec<Diagram>> = sets
        .iter()
        .filter(|s| !s.diagrams.is_empty())
        .map(|s| &s.diagrams)
        .collect();
    if with_diagrams.len() != 1 {
        return Err(format!(
            "expected exactly one non-empty subprocess for '{process}', got {}",
            with_diagrams.len()
        ));
    }
    let model = common::sm_model();
    colorize_process(&model, with_diagrams[0]).map_err(|e| format!("colorize: {e}"))
}

fn run_trial(matrix_path: PathBuf) -> Result<(), Failed> {
    let mg = parse_matrix_file(&matrix_path)?;
    let process = clean_process(&mg.process);

    let cb = colorize(&process)?;

    // NCOLOR must match exactly.
    if cb.ncolor() != mg.ncolor {
        return Err(format!(
            "NCOLOR mismatch for '{process}': vibegraph {} vs MG {}",
            cb.ncolor(),
            mg.ncolor
        )
        .into());
    }

    // Full CF matrix within tolerance.
    let n = mg.ncolor;
    let mut max_rel = 0.0f64;
    for i in 0..n {
        for j in 0..n {
            let ours = ratio_to_f64(cb.cf(i, j));
            let theirs = mg.cf[i * n + j];
            let rel = (ours - theirs).abs() / theirs.abs().max(1.0);
            if rel > max_rel {
                max_rel = rel;
            }
            if rel > CF_REL_TOL {
                return Err(format!(
                    "CF[{i}][{j}] mismatch for '{process}': vibegraph {ours} vs MG {theirs} \
                     (rel {rel:.2e} > {CF_REL_TOL:.0e})"
                )
                .into());
            }
        }
    }

    // Ordering cross-check (report-only): compare our sorted basis structures to
    // MadGraph's per-flow structure comments. A mismatch is a finding, not a
    // failure — the summed-index labelling differs, so this only catches a real
    // row/column permutation on external-index structures.
    let ordering_note = check_ordering(&cb, &mg);

    // NGRAPHS (report-only): the CF oracle is not the diagram-count gate.
    let ngraphs_note = if cb.elements.iter().flat_map(|e| &e.contributions).count() == 0 {
        String::new()
    } else {
        let max_diag = cb
            .elements
            .iter()
            .flat_map(|e| e.contributions.iter().map(|c| c.diagram))
            .max()
            .map(|d| d + 1)
            .unwrap_or(0);
        if max_diag != mg.ngraphs {
            format!(
                " | NGRAPHS: MG {} vs vibegraph max-diag {max_diag}",
                mg.ngraphs
            )
        } else {
            format!(" | NGRAPHS {}", mg.ngraphs)
        }
    };

    println!(
        "  [{}] '{process}' NCOLOR={n} CF max_rel={max_rel:.2e}{ngraphs_note}{ordering_note}",
        trial_name(&matrix_path)
    );
    Ok(())
}

/// Cast an exact rational to `f64` the same way binding will.
fn ratio_to_f64(r: num_rational::Ratio<i64>) -> f64 {
    *r.numer() as f64 / *r.denom() as f64
}

/// Compare our basis order to MadGraph's structure comments; return a note
/// string (empty if they agree or the labels are not comparable).
fn check_ordering(cb: &ColorBasis, mg: &MgReference) -> String {
    if mg.structures.iter().any(|s| s.is_empty()) {
        return String::new();
    }
    let ours: Vec<String> = cb
        .elements
        .iter()
        .map(|e| render_structure(&e.structure))
        .collect();
    let mg_norm: Vec<String> = mg
        .structures
        .iter()
        .map(|s| normalize_structure(s))
        .collect();
    if ours == mg_norm {
        String::new()
    } else {
        format!(" | ORDER-DIFF ours={ours:?} mg={mg_norm:?}")
    }
}

fn main() {
    let args = Arguments::from_args();

    let matrix_files = find_matrix_files();
    if matrix_files.is_empty() {
        eprintln!("No matrix1_orig.f files found in validation/madgraph/output/");
        eprintln!("Run: pixi run -e madgraph build-diagrams");
        libtest_mimic::run(&args, vec![]).exit();
    }

    let trials: Vec<Trial> = matrix_files
        .into_iter()
        .map(|p| {
            let name = trial_name(&p);
            Trial::test(name, move || run_trial(p))
        })
        .collect();

    libtest_mimic::run(&args, trials).exit();
}
