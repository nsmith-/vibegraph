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
//! The same files carry a second, finer comparison: MadGraph's own
//! `JAMP(i) = … AMP(j)` lines give the colour coefficient every amplitude enters
//! every flow with, which is the *decomposition* into the basis rather than the
//! basis itself. `CF` is a Gram matrix and is blind to it — it is invariant under
//! a uniform transpose of the basis keys, and says nothing at all about how a
//! vertex's several colour structures distribute over the flows — so a wrong
//! coefficient on one structure of a multi-structure vertex (the four-gluon
//! contact, with three) survives a perfect `CF` match. [`check_jamp`] closes
//! that: it compares the per-amplitude coefficient columns graph by graph, with
//! each graph's structures kept in order.
//!
//! Each run is colorized under the model `validation/manifest.toml` records for
//! its row, not under the interned Standard Model: two rows can carry the same
//! process string and differ only in which vertices their restrict card leaves
//! standing. Enforcement follows the row's `amplitudes` cell — the colour basis
//! is a factor of the amplitude rather than a category of its own — so a row
//! that cell declares informational has its comparison run and printed and is
//! not asserted.
//!
//! Run:
//!   cargo test -p vibegraph-lib --features extended-validation \
//!              --test color_cf_oracle
//!
//! Prerequisites (regenerates the gitignored MG output):
//!   pixi run -e madgraph build-diagrams

mod common;

use libtest_mimic::{Arguments, Failed, Trial};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use vibegraph::diagrams::{Diagram, DiagramSet};
use vibegraph::helas::color::{colorize_process, ColorBasis};
use vibegraph::helas::color::{ImmutableString, TensorKind};
use vibegraph::ufo::UFOModel;

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
    /// `jamp[flow][graph]`: the colour coefficient MadGraph's own
    /// `JAMP(flow) = Σ_g c AMP(g)` lines multiply each amplitude by.
    jamp: Vec<Vec<Cx>>,
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

    // Parse the CF DATA block. MadGraph emits it two ways, and which one a file
    // carries is a property of the version that wrote it — see `parse_packed_cf`
    // for the newer one. The square form is one logical line per column J,
    //   DATA (CF(I,  J),I=  1,  NCOLOR) /v1, v2, .../
    // optionally followed by a `C     <coeff> <structure>` comment giving the
    // basis structure of flow J.
    if lines
        .iter()
        .any(|l| l.trim_start().starts_with("DATA (CF(I),"))
    {
        let (cf, structures) = parse_packed_cf(&lines, ncolor)?;
        let jamp = parse_jamp_block(&lines, ncolor, ngraphs)?;
        return Ok(MgReference {
            process,
            ncolor,
            ngraphs,
            cf,
            structures,
            jamp,
        });
    }

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
    let jamp = parse_jamp_block(&lines, ncolor, ngraphs)?;
    Ok(MgReference {
        process,
        ncolor,
        ngraphs,
        cf,
        structures,
        jamp,
    })
}

/// Parse the packed color-matrix form, and return the same square `cf` and
/// `structures` the caller builds from the other one.
///
/// Newer MadGraph emits the matrix as integers over one common denominator,
/// storing only the upper triangle:
///
/// ```text
/// INTEGER CF(NCOLOR*(NCOLOR+1)/2)
/// DATA DENOM/6/
/// DATA (CF(I),I=  1,  6) /19,-4,-4,-4,-4,8/
/// ```
///
/// and contracts it with a single running index,
///
/// ```text
/// DO I = 1, NCOLOR
///   DO J = I, NCOLOR
///     CF_INDEX = CF_INDEX + 1
///     ZTEMP = ZTEMP + CF(CF_INDEX)*JAMP(J,M)
/// ```
///
/// so entry `(I,J)` appears once for the unordered pair rather than twice.
/// `MATRIX1` is `REAL*8`, which takes the real part of that sum, and
/// `Re[c·a·conj(b)]` equals `Re[c·b·conj(a)]` for real `c` — so an off-diagonal
/// entry has to carry **twice** the symmetric matrix's value to reproduce the
/// same `|M|²`. The square form is therefore
///
/// ```text
/// CF(I,I) = packed / DENOM        CF(I,J) = CF(J,I) = packed / (2 DENOM)
/// ```
///
/// Confirmed against the older form on three processes spanning `NCOLOR` 1, 2
/// and 6 and `DENOM` 1, 3 and 6, element for element; `packed_gg_ttx_matches_the_square_form`
/// pins the `NCOLOR = 2` case here.
fn parse_packed_cf(lines: &[String], ncolor: usize) -> Result<(Vec<f64>, Vec<String>), String> {
    let denom = lines
        .iter()
        .find_map(|l| {
            l.trim_start()
                .strip_prefix("DATA DENOM/")?
                .split('/')
                .next()?
                .trim()
                .parse::<f64>()
                .ok()
        })
        .ok_or("packed CF block with no DATA DENOM")?;
    if denom == 0.0 {
        return Err("DENOM is zero".into());
    }

    let mut packed: Vec<f64> = Vec::new();
    let mut structures = vec![String::new(); ncolor];
    let mut pending = false;
    let mut row = 0usize;
    for line in lines {
        if line.trim_start().starts_with("DATA (CF(I),") {
            let body = line
                .split_once('/')
                .and_then(|(_, r)| r.rsplit_once('/').map(|(v, _)| v))
                .ok_or_else(|| format!("no /.../ in CF line: {line}"))?;
            for tok in body.split(',').map(str::trim).filter(|t| !t.is_empty()) {
                packed
                    .push(parse_fortran_real(tok).ok_or_else(|| format!("bad CF value '{tok}'"))?);
            }
            pending = true;
        } else if pending {
            // Each `DATA` line is one row of the triangle, and the structure
            // comment after it names that row's flow.
            if let Some(rest) = line.trim_start().strip_prefix('C') {
                if let Some((_, structure)) = rest.trim().split_once(char::is_whitespace) {
                    if row < ncolor {
                        structures[row] = normalize_structure(structure);
                    }
                }
            }
            row += 1;
            pending = false;
        }
    }

    let expected = ncolor * (ncolor + 1) / 2;
    if packed.len() != expected {
        return Err(format!(
            "packed CF has {} values, expected NCOLOR*(NCOLOR+1)/2 = {expected}",
            packed.len()
        ));
    }

    let mut cf = vec![f64::NAN; ncolor * ncolor];
    let mut k = 0;
    for i in 0..ncolor {
        for j in i..ncolor {
            let v = if i == j {
                packed[k] / denom
            } else {
                packed[k] / (2.0 * denom)
            };
            cf[i * ncolor + j] = v;
            cf[j * ncolor + i] = v;
            k += 1;
        }
    }
    Ok((cf, structures))
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
            TensorKind::Epsilon => "Epsilon",
            TensorKind::EpsilonBar => "EpsilonBar",
            TensorKind::K6 => "K6",
            TensorKind::K6Bar => "K6Bar",
            TensorKind::T6 => "T6",
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

/// Enumerate `process` under `model` and colorize its single concrete subprocess.
fn colorize(process: &str, model: &UFOModel) -> Result<ColorBasis, String> {
    let sets: Vec<DiagramSet> = common::generate_with(process, model);
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
    colorize_process(model, with_diagrams[0]).map_err(|e| format!("colorize: {e}"))
}

/// Rows whose `amplitudes` cell is informational but whose **colour**
/// comparison is enforced anyway, each with the measurement that earned it.
///
/// The default rule below reads enforcement off the row's `amplitudes` cell,
/// because a colour basis is a factor of an amplitude rather than a category of
/// its own. That is the right default for a row nothing yet evaluates, and the
/// wrong one for a row whose colour layer has been measured exact while its
/// amplitude is still under construction — leaving it reported would let the
/// colour result regress silently while the amplitude cell explains why nobody
/// noticed. A row listed here must pass; the list is not an exemption from
/// anything, it is a promotion.
const COLOUR_ENFORCED_INFO_ROWS: &[(&str, &str)] = &[(
    "gg_to_gg_cg",
    "CF max_rel = 0 at NCOLOR 9 and all 27 JAMP colour columns exact with no rephasing, \
     including the nine four-gluon contact structures; the row's residual against \
     MadGraph is in the amplitudes, not in the colour decomposition",
)];

/// A row's cell is asserted or only reported, and the comparison itself is the
/// same either way, so it is run first and its outcome decided on after.
fn run_trial(matrix_path: PathBuf) -> Result<(), Failed> {
    let key = common::row_key_of(&matrix_path);
    let name = trial_name(&matrix_path);
    // An enforced row's panic stays a panic; a reported one is part of what is
    // being reported.
    let enforced = common::amplitudes_enforced(&key)
        || COLOUR_ENFORCED_INFO_ROWS.iter().any(|(k, _)| *k == key);
    let outcome = if enforced {
        compare(&matrix_path)
    } else {
        common::catching_panics(|| compare(&matrix_path))
    };
    match outcome {
        Ok(line) => {
            println!("  [{name}] {line}");
            Ok(())
        }
        Err(message) if !enforced => {
            println!("  [{name}] reported, not enforced: {message}");
            Ok(())
        }
        Err(message) => Err(message.into()),
    }
}

/// Compare one generated subprocess's colour-factor matrix to ours, returning
/// the line that describes the agreement.
fn compare(matrix_path: &Path) -> Result<String, String> {
    let content =
        std::fs::read_to_string(matrix_path).map_err(|e| format!("read {matrix_path:?}: {e}"))?;
    let lines = logical_lines(&content);
    let mg = parse_matrix_file(matrix_path)?;
    let process = clean_process(&mg.process);

    // The row's own model under its own restrict card: two rows can share a
    // process string and differ only in which vertices their card leaves
    // standing, and a colour basis built from the wrong one is not a comparison.
    let model = common::model_for_row(&common::row_key_of(matrix_path))?;
    let cb = colorize(&process, &model)?;

    // NCOLOR must match exactly.
    if cb.ncolor() != mg.ncolor {
        return Err(format!(
            "NCOLOR mismatch for '{process}': vibegraph {} vs MG {}",
            cb.ncolor(),
            mg.ncolor
        ));
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
                ));
            }
        }
    }

    // The colour coefficients each amplitude enters `JAMP` with, against
    // MadGraph's own JAMP lines — the level below the CF matrix.
    let jamp_note = check_jamp(&cb, &mg, &lines)?;

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

    Ok(format!(
        "'{process}' NCOLOR={n} CF max_rel={max_rel:.2e}{jamp_note}{ngraphs_note}{ordering_note}"
    ))
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

/// The packed color-matrix block MadGraph 3.7.1 writes for `g g > t t~`,
/// verbatim.
const PACKED_GG_TTX: &str = "\
      DATA DENOM/3/
      DATA (CF(I),I=  1,  2) /16,-4/
C     1 T(1,2,3,4)
      DATA (CF(I),I=  3,  3) /16/
C     1 T(2,1,3,4)
";

/// The square block MadGraph 3.5.7 writes for the same process, which is the
/// matrix the packed one has to reproduce.
const SQUARE_GG_TTX: [f64; 4] = [
    5.333333333333333,
    -0.6666666666666666,
    -0.6666666666666666,
    5.333333333333333,
];

/// Pin the packed→square mapping against the other form's own numbers.
///
/// The live sweep below only reaches the packed form on `NCOLOR = 1` process
/// directories, where the diagonal-only mapping is trivially right and the factor
/// of two on off-diagonal entries is never exercised. This is the smallest case
/// that exercises it.
fn packed_cf_form_matches_the_square_form() -> Result<(), Failed> {
    let (cf, structures) = parse_packed_cf(&logical_lines(PACKED_GG_TTX), 2)?;
    for (k, (got, want)) in cf.iter().zip(&SQUARE_GG_TTX).enumerate() {
        let rel = (got - want).abs() / want.abs();
        if rel > CF_REL_TOL {
            return Err(format!(
                "packed CF[{}][{}] = {got} against the square form's {want} (rel {rel:.2e})",
                k / 2,
                k % 2
            )
            .into());
        }
    }
    if structures != ["T(1,2,3,4)", "T(2,1,3,4)"] {
        return Err(format!("packed CF structures came out as {structures:?}").into());
    }
    Ok(())
}

fn main() {
    let args = Arguments::from_args();

    let matrix_files = find_matrix_files();
    if matrix_files.is_empty() {
        eprintln!("No matrix1_orig.f files found in validation/madgraph/output/");
        eprintln!("Run: pixi run -e madgraph build-diagrams");
        libtest_mimic::run(&args, vec![]).exit();
    }

    let mut trials: Vec<Trial> = matrix_files
        .into_iter()
        .map(|p| {
            let name = trial_name(&p);
            Trial::test(name, move || run_trial(p))
        })
        .collect();
    trials.push(Trial::test(
        "packed-cf-form/gg_ttx",
        packed_cf_form_matches_the_square_form,
    ));

    libtest_mimic::run(&args, trials).exit();
}

// ── JAMP colour-coefficient oracle ───────────────────────────────────────────

/// Relative tolerance on the JAMP colour coefficients. Both sides are exact
/// rationals; MadGraph prints them as 16-digit decimals, so the only error is
/// the last-digit rounding of both sides to the nearest `f64`.
const JAMP_REL_TOL: f64 = 1e-14;

/// A complex colour coefficient, `(re, im)`.
type Cx = (f64, f64);

fn cx_add(a: Cx, b: Cx) -> Cx {
    (a.0 + b.0, a.1 + b.1)
}

fn cx_mul(a: Cx, b: Cx) -> Cx {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}

fn cx_abs(a: Cx) -> f64 {
    a.0.hypot(a.1)
}

/// Parse one Fortran scalar literal as it appears inside a JAMP term's
/// parenthesised coefficient: either a real (`-5.0D-01`) or a complex pair
/// (`(0.0D+00,1.0D+00)`).
fn parse_fortran_scalar(tok: &str) -> Option<Cx> {
    let t = tok.trim();
    if let Some(inner) = t.strip_prefix('(').and_then(|r| r.strip_suffix(')')) {
        let (re, im) = inner.split_once(',')?;
        return Some((parse_fortran_real(re)?, parse_fortran_real(im)?));
    }
    Some((parse_fortran_real(t)?, 0.0))
}

/// Split off the matching `)` of a leading `(`; returns `(inside, rest)`.
fn balanced(s: &str) -> Option<(&str, &str)> {
    let bytes = s.as_bytes();
    if bytes.first() != Some(&b'(') {
        return None;
    }
    let mut depth = 0usize;
    for (k, b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some((&s[1..k], &s[k + 1..]));
                }
            }
            _ => {}
        }
    }
    None
}

/// Parse one right-hand side of a `JAMP`/`TMP_JAMP` assignment into a vector of
/// coefficients over `AMP(1..ngraphs)`, resolving `TMP_JAMP` references against
/// the ones already parsed.
///
/// The grammar MadGraph's exporter emits is a signed sum of terms, each an
/// optional parenthesised scalar times either `AMP(n)` or `TMP_JAMP(n)`.
fn parse_jamp_rhs(
    rhs: &str,
    tmp: &HashMap<usize, Vec<Cx>>,
    ngraphs: usize,
) -> Result<Vec<Cx>, String> {
    let mut out = vec![(0.0, 0.0); ngraphs];
    let mut rest: &str = rhs;
    while !rest.is_empty() {
        let mut sign = 1.0;
        if let Some(r) = rest.strip_prefix('+') {
            rest = r;
        } else if let Some(r) = rest.strip_prefix('-') {
            sign = -1.0;
            rest = r;
        }
        let mut coeff = (sign, 0.0);
        if rest.starts_with('(') {
            let (inside, after) =
                balanced(rest).ok_or_else(|| format!("unbalanced '(' in {rhs}"))?;
            let scalar =
                parse_fortran_scalar(inside).ok_or_else(|| format!("bad scalar '{inside}'"))?;
            coeff = cx_mul(coeff, scalar);
            rest = after
                .strip_prefix('*')
                .ok_or_else(|| format!("coefficient not followed by '*' in {rhs}"))?;
        }
        let (name, after) = rest
            .split_once('(')
            .ok_or_else(|| format!("term without an operand in {rhs}"))?;
        let (index, after) = after
            .split_once(')')
            .ok_or_else(|| format!("unterminated index in {rhs}"))?;
        let index: usize = index
            .parse()
            .map_err(|_| format!("non-numeric index '{index}' in {rhs}"))?;
        match name {
            "AMP" => {
                if index == 0 || index > ngraphs {
                    return Err(format!("AMP({index}) out of range 1..{ngraphs}"));
                }
                out[index - 1] = cx_add(out[index - 1], coeff);
            }
            "TMP_JAMP" => {
                let src = tmp
                    .get(&index)
                    .ok_or_else(|| format!("TMP_JAMP({index}) used before it is defined"))?;
                for (o, s) in out.iter_mut().zip(src) {
                    *o = cx_add(*o, cx_mul(coeff, *s));
                }
            }
            other => return Err(format!("unknown operand '{other}' in {rhs}")),
        }
        rest = after;
    }
    Ok(out)
}

/// Parse the `JAMP(f,1) = …` block into `jamp[flow][graph]`, the exact colour
/// coefficient MadGraph's generated code multiplies each `AMP()` by.
fn parse_jamp_block(
    lines: &[String],
    ncolor: usize,
    ngraphs: usize,
) -> Result<Vec<Vec<Cx>>, String> {
    let mut tmp: HashMap<usize, Vec<Cx>> = HashMap::new();
    let mut jamp: Vec<Option<Vec<Cx>>> = vec![None; ncolor];
    for line in lines {
        // One logical statement, comment stripped and whitespace removed: the
        // exporter's continuation lines split numeric literals mid-token.
        let stmt: String = line
            .split('!')
            .next()
            .unwrap_or("")
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        let Some((lhs, rhs)) = stmt.split_once('=') else {
            continue;
        };
        if let Some(idx) = lhs
            .strip_prefix("TMP_JAMP(")
            .and_then(|r| r.strip_suffix(')'))
        {
            let idx: usize = idx
                .parse()
                .map_err(|_| format!("bad TMP_JAMP index: {lhs}"))?;
            let value = parse_jamp_rhs(rhs, &tmp, ngraphs)?;
            tmp.insert(idx, value);
        } else if let Some(idx) = lhs
            .strip_prefix("JAMP(")
            .and_then(|r| r.strip_suffix(",1)"))
        {
            let Ok(idx) = idx.parse::<usize>() else {
                continue; // `JAMP(:,:) = (0D0,0D0)`
            };
            if idx == 0 || idx > ncolor {
                return Err(format!("JAMP({idx},1) out of range 1..{ncolor}"));
            }
            jamp[idx - 1] = Some(parse_jamp_rhs(rhs, &tmp, ngraphs)?);
        }
    }
    jamp.into_iter()
        .enumerate()
        .map(|(f, row)| row.ok_or_else(|| format!("no JAMP({},1) assignment", f + 1)))
        .collect()
}

/// Our own JAMP decomposition: one column of colour coefficients over the flows
/// per amplitude, an amplitude being one `(diagram, colour-index chain)` pair —
/// the same object MadGraph writes as one `AMP()`.
fn our_jamp(cb: &ColorBasis) -> Vec<((usize, Vec<u8>), Vec<Cx>)> {
    let mut keys: BTreeSet<(usize, Vec<u8>)> = BTreeSet::new();
    for el in &cb.elements {
        for c in &el.contributions {
            keys.insert((c.diagram, c.chain.clone()));
        }
    }
    let keys: Vec<(usize, Vec<u8>)> = keys.into_iter().collect();
    let mut columns = vec![vec![(0.0, 0.0); cb.ncolor()]; keys.len()];
    for (f, el) in cb.elements.iter().enumerate() {
        for c in &el.contributions {
            let a = keys
                .iter()
                .position(|k| k.0 == c.diagram && k.1 == c.chain)
                .expect("key collected above");
            let q = ratio_to_f64(c.coeff.eval_nc(3));
            let value = if c.coeff.imag { (0.0, q) } else { (q, 0.0) };
            columns[a][f] = cx_add(columns[a][f], value);
        }
    }
    keys.into_iter().zip(columns).collect()
}

/// The unit factor a *graph's* colour-coefficient columns are collectively defined
/// up to: MadGraph folds the diagram's fermion factor into the coefficient where
/// vibegraph carries it in the diagram root, so a graph is fixed only up to one
/// overall unit. Return every column of the graph divided by the phase of the first
/// non-zero entry of its first non-zero column, together with that phase.
///
/// The unit is **per graph, not per column**: a vertex's several colour structures
/// (the four-gluon contact has three) write several `AMP()` of the same graph, and
/// they share whatever convention factor separates the two sides. Normalising each
/// column on its own would absorb an independent sign per structure — the freedom a
/// per-structure sign error hides in.
fn normalise_group(cols: &[Vec<Cx>]) -> Option<(Cx, Vec<Vec<Cx>>)> {
    let scale = cols
        .iter()
        .flatten()
        .map(|c| cx_abs(*c))
        .fold(0.0f64, f64::max);
    if scale == 0.0 {
        return None;
    }
    let lead = *cols
        .iter()
        .flatten()
        .find(|c| cx_abs(**c) > 1e-12 * scale)?;
    let unit = (lead.0 / cx_abs(lead), lead.1 / cx_abs(lead));
    let conj = (unit.0, -unit.1);
    Some((
        unit,
        cols.iter()
            .map(|col| col.iter().map(|c| cx_mul(*c, conj)).collect())
            .collect(),
    ))
}

/// Sort key that orders normalised columns deterministically for the multiset
/// comparison (they are compared numerically afterwards).
fn column_key(col: &[Cx]) -> Vec<(i64, i64)> {
    col.iter()
        .map(|c| ((c.0 * 1e9).round() as i64, (c.1 * 1e9).round() as i64))
        .collect()
}

/// MadGraph's grouping of `AMP()` indices by the graph that produced them, read
/// from its own `C     Amplitude(s) for diagram number N` comments.
///
/// A vertex with several colour structures makes one graph write several
/// amplitudes — the four-gluon contact writes three — so this is what turns the
/// per-amplitude comparison into a per-*structure* one: the amplitudes of one
/// graph are compared as an ordered tuple, in the order MadGraph emits them.
fn mg_amp_groups(lines: &[String]) -> Vec<Vec<usize>> {
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for line in lines {
        let t = line.trim_start();
        if t.starts_with("JAMP(") || t.contains("JAMP(:,:)") {
            break;
        }
        if t.starts_with('C') && t.contains("Amplitude(s) for diagram number") {
            groups.push(Vec::new());
            continue;
        }
        if !t.starts_with("CALL") {
            continue;
        }
        let Some(group) = groups.last_mut() else {
            continue;
        };
        let mut rest = t;
        while let Some((_, after)) = rest.split_once("AMP(") {
            let Some((idx, tail)) = after.split_once(')') else {
                break;
            };
            if let Ok(i) = idx.trim().parse::<usize>() {
                group.push(i - 1);
            }
            rest = tail;
        }
    }
    groups
}

/// Compare our per-amplitude colour-coefficient columns to MadGraph's own
/// `JAMP(i) = … AMP(j)` lines, and return the note describing the agreement.
///
/// This is the level below the CF matrix: `CF` is a Gram matrix over the basis
/// and so cannot see how a diagram's colour structure *decomposes into* it, and
/// the flow tags see only each basis key's connectivity. The decomposition is
/// what multiplies the amplitudes into `JAMP`, and a wrong coefficient on one
/// structure of a multi-structure vertex — the four-gluon contact is the case
/// with three of them — shows up here and nowhere else in the colour layer.
///
/// The comparison is mapping-free: nothing derives MadGraph's graph order from
/// ours, so graphs are matched as a multiset rather than paired by index
/// (`amplitude_oracle`'s banked `MG_DIAGRAM_ORDER` is what pins the pairing).
/// What is compared per graph is the **ordered tuple** of its amplitudes'
/// columns under a *single* unit for the whole graph ([`normalise_group`]), so
/// neither a permutation of a vertex's colour structures nor a sign on one of them
/// survives.
///
/// The per-graph units are then held to each other. MadGraph's own freedom here is
/// its fermion factor, which is real, and every other difference between the two
/// sides' coefficients is a property of the colour conventions and so common to the
/// whole subprocess. So each graph's unit must be `±1` times the subprocess's modal
/// unit; a graph that needs a factor of `i` of its own is a colour-convention defect
/// and fails. How many graphs carry the `−1` is reported — that number is MadGraph's
/// fermion-factor pattern, read out of its JAMP lines.
fn check_jamp(cb: &ColorBasis, mg: &MgReference, lines: &[String]) -> Result<String, String> {
    // Ours, grouped by diagram: one group per graph, its columns in colour-index
    // chain order.
    let ours = our_jamp(cb);
    let mut mine: Vec<(Vec<Vec<(i64, i64)>>, Cx, Vec<Vec<Cx>>)> = Vec::new();
    for (_diagram, columns) in group_by_diagram(&ours) {
        let cols: Vec<Vec<Cx>> = columns.into_iter().map(|(_, col)| col).collect();
        if let Some((unit, norm)) = normalise_group(&cols) {
            mine.push((norm.iter().map(|c| column_key(c)).collect(), unit, norm));
        }
    }

    // MadGraph's, grouped by its own `Amplitude(s) for diagram number` comments.
    let groups = mg_amp_groups(lines);
    if groups.iter().map(Vec::len).sum::<usize>() != mg.ngraphs {
        return Err(format!(
            "MadGraph's amplitude comments cover {} of {} graphs",
            groups.iter().map(Vec::len).sum::<usize>(),
            mg.ngraphs
        ));
    }
    let mut theirs: Vec<(Vec<Vec<(i64, i64)>>, Cx, Vec<Vec<Cx>>)> = Vec::new();
    for group in &groups {
        let cols: Vec<Vec<Cx>> = group
            .iter()
            .map(|&g| (0..mg.ncolor).map(|f| mg.jamp[f][g]).collect())
            .collect();
        if let Some((unit, norm)) = normalise_group(&cols) {
            theirs.push((norm.iter().map(|c| column_key(c)).collect(), unit, norm));
        }
    }

    if mine.len() != theirs.len() {
        return Err(format!(
            "colour-carrying graph count: vibegraph {} vs MadGraph {} (NGRAPHS {})",
            mine.len(),
            theirs.len(),
            mg.ngraphs
        ));
    }
    mine.sort_by(|a, b| a.0.cmp(&b.0));
    theirs.sort_by(|a, b| a.0.cmp(&b.0));

    let mut max_rel = 0.0f64;
    let mut columns = 0usize;
    let mut units: Vec<Cx> = Vec::new();
    for (g, (a, b)) in mine.iter().zip(&theirs).enumerate() {
        if a.2.len() != b.2.len() {
            return Err(format!(
                "graph {g}: vibegraph writes {} colour structures, MadGraph {}",
                a.2.len(),
                b.2.len()
            ));
        }
        units.push(cx_mul(a.1, (b.1 .0, -b.1 .1)));
        for (k, (ours, mg_col)) in a.2.iter().zip(&b.2).enumerate() {
            columns += 1;
            for (f, (x, y)) in ours.iter().zip(mg_col).enumerate() {
                let diff = cx_abs((x.0 - y.0, x.1 - y.1));
                let rel = diff / cx_abs(*y).max(1.0);
                max_rel = max_rel.max(rel);
                if rel > JAMP_REL_TOL {
                    return Err(format!(
                        "JAMP coefficient of graph {g} structure {k} on flow {f}: vibegraph \
                         ({}, {}) vs MadGraph ({}, {}) (rel {rel:.2e} > {JAMP_REL_TOL:.0e})",
                        x.0, x.1, y.0, y.1
                    ));
                }
            }
        }
    }

    // The modal unit is the subprocess's colour-convention factor; every graph must
    // sit at ±1 of it, which is all MadGraph's real fermion factor can produce.
    let flipped = graph_unit_flips(&units)?;
    Ok(format!(
        " | JAMP {columns} columns over {} graphs max_rel={max_rel:.2e} ({flipped} sign-flipped)",
        mine.len()
    ))
}

/// The number of graphs whose unit is the negative of the subprocess's modal unit,
/// after checking that the mode is a fourth root of unity and that every graph's
/// unit is real relative to it.
fn graph_unit_flips(units: &[Cx]) -> Result<usize, String> {
    let Some(&modal) = units.first() else {
        return Ok(0);
    };
    let quarter = [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)];
    if !quarter
        .iter()
        .any(|q| cx_abs((modal.0 - q.0, modal.1 - q.1)) < 1e-9)
    {
        return Err(format!(
            "the subprocess's colour coefficients lead with the unit ({}, {}), not a \
             fourth root of unity",
            modal.0, modal.1
        ));
    }
    // Take the majority of the two candidate modes (`modal` and `−modal`) so a
    // process whose first graph is the flipped one still reports the small count.
    let mut same = 0usize;
    let mut flipped = 0usize;
    for (g, &u) in units.iter().enumerate() {
        let ratio = cx_mul(u, (modal.0, -modal.1));
        if cx_abs((ratio.0 - 1.0, ratio.1)) < 1e-9 {
            same += 1;
        } else if cx_abs((ratio.0 + 1.0, ratio.1)) < 1e-9 {
            flipped += 1;
        } else {
            return Err(format!(
                "graph {g}'s colour coefficients need the unit ({}, {}) relative to the \
                 subprocess's, which is not ±1: MadGraph's own freedom here is a real \
                 fermion factor, so a complex one is a colour-convention defect",
                ratio.0, ratio.1
            ));
        }
    }
    Ok(same.min(flipped))
}

/// Split [`our_jamp`]'s per-amplitude columns into per-diagram groups, keeping
/// each diagram's colour-index chains in ascending order.
#[allow(clippy::type_complexity)]
fn group_by_diagram(
    amplitudes: &[((usize, Vec<u8>), Vec<Cx>)],
) -> Vec<(usize, Vec<(Vec<u8>, Vec<Cx>)>)> {
    let mut out: Vec<(usize, Vec<(Vec<u8>, Vec<Cx>)>)> = Vec::new();
    for ((diagram, chain), col) in amplitudes {
        match out.last_mut() {
            Some((d, group)) if d == diagram => group.push((chain.clone(), col.clone())),
            _ => out.push((*diagram, vec![(chain.clone(), col.clone())])),
        }
    }
    out
}
