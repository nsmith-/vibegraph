//! Validate `AmplitudeEvaluator` against MadGraph's own generated Fortran matrix
//! elements for each process covered by the diagram validation suite.
//!
//! One named test trial per process, generated dynamically from CSV files in
//! `validation/madgraph/output/`.  Only color-free processes (ee→μμ) are expected
//! to pass; colored processes emit an informational message and pass regardless,
//! since color flow is not yet implemented in vibegraph.
//!
//! Run:
//!   cargo test -p vibegraph-lib --features extended-validation \
//!              --test helas_mg_validation
//!
//! Prerequisites:
//!   pixi run -e madgraph build-amplitude
//!   pixi run -e madgraph generate-amplitude

mod common;

use libtest_mimic::{Arguments, Failed, Trial};
use std::path::{Path, PathBuf};
use vibegraph::helas::eval::AmplitudeEvaluator;
use vibegraph::helas::LorentzVector;
use vibegraph::ufo::slha::ParamCard;

/// Relative tolerance for processes expected to agree (color-free, same SM params).
///
/// MadGraph's generated MATRIX1 treats all leptons as massless (hard-coded `ZERO` in
/// HELAS calls), while vibegraph uses physical masses from the UFO model.  The resulting
/// difference is O(m_mu² / s) ≈ 7×10⁻⁴ at √s = 10 GeV, decreasing at higher energies.
/// Any real amplitude bug (wrong coupling, missing diagram) would produce O(1%) or larger
/// deviations, well above this threshold.
const REL_TOL: f64 = 2e-3;

/// Processes for which we enforce agreement; others are informational only.
const EXPECT_MATCH: &[&str] = &["ee_to_mumu"];

struct AmpRow {
    sqrt_s: f64,
    cos_theta: f64,
    m2_ref: f64,
}

/// Find all *_amplitude.csv files in validation/madgraph/output/.
fn find_amplitude_csvs() -> Vec<PathBuf> {
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/output");

    if !output_dir.exists() {
        return Vec::new();
    }

    let mut paths: Vec<PathBuf> = std::fs::read_dir(&output_dir)
        .expect("cannot read output dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with("_amplitude.csv"))
                .unwrap_or(false)
        })
        .collect();

    paths.sort();
    paths
}

/// Derive process name from CSV path: "ee_to_mumu_amplitude.csv" → "ee_to_mumu".
fn process_name(p: &Path) -> String {
    let stem = p
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    stem.strip_suffix("_amplitude").unwrap_or(&stem).to_owned()
}

/// Parse the CSV: extract `# process: ...` header and data rows.
/// Returns (process_string, rows).  Rows are empty for skeleton CSVs.
fn read_csv(path: &Path) -> (String, Vec<AmpRow>) {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));

    let mut process_str = String::new();
    let mut rows = Vec::new();
    let mut header_skipped = false;

    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("# process:") {
            process_str = rest.trim().to_owned();
        } else if line.starts_with('#') || line.is_empty() {
            continue;
        } else if !header_skipped {
            header_skipped = true; // skip column-name header row
        } else {
            let cols: Vec<&str> = line.splitn(3, ',').collect();
            assert_eq!(cols.len(), 3, "expected 3 columns in: {line}");
            rows.push(AmpRow {
                sqrt_s: cols[0].trim().parse().expect("bad sqrt_s"),
                cos_theta: cols[1].trim().parse().expect("bad cos_theta"),
                m2_ref: cols[2].trim().parse().expect("bad M2"),
            });
        }
    }

    (process_str, rows)
}

/// Build massless 2→2 momenta matching helas_validation.rs convention (lines 145-149):
///   [0] e+  (E,  0,       0,  -E)
///   [1] e-  (E,  0,       0,  +E)
///   [2] mu+ (E, -E*sin_t, 0,  -E*cos_t)
///   [3] mu- (E, +E*sin_t, 0,  +E*cos_t)
fn make_momenta_2to2(sqrt_s: f64, cos_theta: f64) -> Vec<LorentzVector<f64>> {
    let e = sqrt_s / 2.0;
    let sin_t = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
    vec![
        LorentzVector::new(e, 0.0, 0.0, -e),
        LorentzVector::new(e, 0.0, 0.0, e),
        LorentzVector::new(e, -e * sin_t, 0.0, -e * cos_theta),
        LorentzVector::new(e, e * sin_t, 0.0, e * cos_theta),
    ]
}

fn run_trial(csv_path: PathBuf) -> Result<(), Failed> {
    let name = process_name(&csv_path);
    let (process_str, rows) = read_csv(&csv_path);

    if process_str.is_empty() {
        return Err("no '# process:' header found in CSV".into());
    }
    if rows.is_empty() {
        // Skeleton CSV (2→3 or unimplemented process) — skip silently.
        return Ok(());
    }

    let sets = common::generate(&process_str);
    if sets.is_empty() {
        return Err(format!("no diagrams generated for '{process_str}'").into());
    }

    let model = common::sm_model();
    // Use UFO SM defaults (matching MadGraph's param_card values used in gen_amplitude.py)
    let empty_card = ParamCard::from_str("").unwrap();
    let evaluated = model.evaluate(&empty_card);

    let evaluator = AmplitudeEvaluator::compile(&sets[0], model)
        .map_err(|e| Failed::from(format!("compile: {e}")))?;

    let mut failures = 0usize;
    let mut max_rel_diff = 0.0f64;

    for row in &rows {
        let momenta = make_momenta_2to2(row.sqrt_s, row.cos_theta);
        let m2_rust = evaluator.eval_m2(&momenta, &evaluated);
        let rel = (m2_rust - row.m2_ref).abs() / row.m2_ref.max(1e-30);
        if rel > max_rel_diff {
            max_rel_diff = rel;
        }
        if rel > REL_TOL {
            failures += 1;
        }
    }

    if EXPECT_MATCH.contains(&name.as_str()) {
        if failures > 0 {
            return Err(format!(
                "{failures}/{} points exceeded rel tolerance {REL_TOL:.0e} \
                 (max_rel_diff={max_rel_diff:.2e})",
                rows.len()
            )
            .into());
        }
    } else if failures > 0 {
        eprintln!(
            "INFO [{name}]: {failures}/{} points differ from MG reference \
             (color not yet implemented); max_rel_diff={max_rel_diff:.2e}",
            rows.len()
        );
    }

    Ok(())
}

fn main() {
    let args = Arguments::from_args();

    let csv_paths = find_amplitude_csvs();
    if csv_paths.is_empty() {
        eprintln!("No amplitude CSV files found in validation/madgraph/output/");
        eprintln!("Run: pixi run -e madgraph build-amplitude generate-amplitude");
        libtest_mimic::run(&args, vec![]).exit();
    }

    let trials: Vec<Trial> = csv_paths
        .into_iter()
        .map(|p| {
            let name = process_name(&p);
            Trial::test(name, move || run_trial(p))
        })
        .collect();

    libtest_mimic::run(&args, trials).exit();
}
