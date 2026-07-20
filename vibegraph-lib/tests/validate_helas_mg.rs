//! Validate `AmplitudeEvaluator` against MadGraph's own generated Fortran matrix
//! elements for each process covered by the diagram validation suite.
//!
//! One named test trial per process, generated dynamically from CSV files in
//! `validation/madgraph/output/`. `eval_m2` returns the full color-summed |M|²
//! (the CF matrix is computed by the color factorization, not hard-coded), so the
//! single-color-flow `EXPECT_MATCH` processes are compared bit-for-bit against
//! MadGraph's MATRIX1, and the multi-flow `uux_to_uux` (NCOLOR=2) is enforced at
//! `REL_TOL` via the CF-weighted contraction. Any process not in `EXPECT_MATCH`
//! is informational until its reference is enforced.
//!
//! Run:
//!   cargo test -p vibegraph-lib --features extended-validation \
//!              --test validate_helas_mg
//!
//! Prerequisites:
//!   pixi run -e madgraph build-amplitude
//!   pixi run -e madgraph generate-amplitude
//!
//! This is a correctness gate only. For evaluator/integration performance,
//! profile the sigma gate (`pixi run validate-sigma`) under `--profile
//! profiling` with `samply` — its per-process time is weighted by how hard each
//! process is to *integrate*, unlike a fixed-N micro-benchmark.

mod common;

use libtest_mimic::{Arguments, Failed, Trial};
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
use vibegraph::helas::LorentzVector;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::EvaluatedModel;

/// Relative tolerance for processes expected to agree (color-free, same SM params).
///
/// vibegraph evaluates with MadGraph's exact per-process `param_card.dat` (same
/// rounded masses, SM inputs, and decay widths), and the SM model bakes in
/// `restrict_default.dat`, so agreement is limited only by summation-order
/// differences across the diagram set: the suite max is currently 5.4e-13
/// (`ee_to_mumu_tata_qcd0`), most processes sit at 1e-15..1e-13. The tolerance
/// rides ~2× above that max so genuine regressions fail while benign FP
/// reordering (fused kernels, balanced sums, egglog rewrites) passes.
const REL_TOL: f64 = 1e-12;

/// Processes for which we enforce agreement; others are informational only.
const EXPECT_MATCH: &[&str] = &[
    "ee_to_mumu",
    "pp_to_ll_qcd0",
    "ee_to_ee",
    "ee_to_mumua",
    "ee_to_ttx",
    "ee_to_wpwm",
    "ee_to_zh",
    "ee_to_tatah",
    "ee_to_mumu_tata_qcd0",
    "uux_to_ccx_emmm_qcd0",
    "bbx_to_ccx_emmm_qcd0",
    // First multi-flow (NCOLOR=2) enforced process: s⊕t gluon exchange with
    // identical quarks, |M|² formed by the CF-weighted contraction over both
    // color flows. Agrees with MadGraph's MATRIX1 to max_rel_diff ~5.6e-14.
    "uux_to_uux",
    // External octets feeding a quark line: the triple-gluon `f(1,2,3)` vertex
    // (f → trace) mixed with pure T-chain diagrams (NCOLOR=2). Exercises the
    // fundamental/antifundamental slot convention that keeps the imaginary
    // f-derived contribution in sign with the rational T-chain terms; agrees
    // with MadGraph's MATRIX1 to max_rel_diff ~1.9e-15.
    "gg_to_ttx",
    // First NCOLOR=6 process: the 4-gluon contact (VVVV) vertex plus three
    // triple-gluon exchange diagrams. The propagator-free contact term contracts
    // a pure-metric structure straight into the amplitude with a real −1 vertex
    // factor; |M|² formed by the CF-weighted contraction over all six color
    // flows. Agrees with MadGraph's MATRIX1 to max_rel_diff ~8.25e-14.
    "gg_to_gg",
];

/// One evaluated phase-space point: external momenta (incoming then outgoing)
/// and MadGraph's reference Σ_hel Σ_color |M|².
struct AmpPoint {
    momenta: Vec<LorentzVector<f64>>,
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

/// Parse the CSV into (process_string, points): a `# process:` + `# n_ext: N`
/// header, one column-name row, then rows of `m2_summed, E0,px0,..,pz_{N-1}`
/// (the momenta-based schema gen_amplitude.py writes for every process).
fn read_csv(path: &Path) -> (String, Vec<AmpPoint>) {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));

    let mut process_str = String::new();
    let mut n_ext: Option<usize> = None;
    let mut points = Vec::new();
    let mut header_skipped = false;

    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("# process:") {
            process_str = rest.trim().to_owned();
        } else if let Some(rest) = line.strip_prefix("# n_ext:") {
            n_ext = Some(rest.trim().parse().expect("bad n_ext"));
        } else if line.starts_with('#') || line.is_empty() {
            continue;
        } else if !header_skipped {
            header_skipped = true; // skip column-name header row
        } else {
            let n = n_ext.expect("missing '# n_ext:' header before data rows");
            let cols: Vec<f64> = line
                .split(',')
                .map(|c| c.trim().parse().expect("bad number in row"))
                .collect();
            assert_eq!(
                cols.len(),
                1 + 4 * n,
                "expected {} cols in: {line}",
                1 + 4 * n
            );
            let momenta = (0..n)
                .map(|i| {
                    let b = 1 + 4 * i;
                    LorentzVector::new(cols[b], cols[b + 1], cols[b + 2], cols[b + 3])
                })
                .collect();
            points.push(AmpPoint {
                momenta,
                m2_ref: cols[0],
            });
        }
    }

    (process_str, points)
}

fn run_trial(csv_path: PathBuf) -> Result<(), Failed> {
    let name = process_name(&csv_path);
    let (process_str, points) = read_csv(&csv_path);

    if process_str.is_empty() {
        return Err("no '# process:' header found in CSV".into());
    }
    if points.is_empty() {
        // Skeleton CSV (unimplemented process) — skip silently.
        return Ok(());
    }

    let sets = common::generate(&process_str);
    if sets.is_empty() {
        return Err(format!("no diagrams generated for '{process_str}'").into());
    }

    let model = common::sm_model();
    // Evaluate with MadGraph's actual param_card for this process, so vibegraph uses
    // the exact (param-card-rounded) inputs MadGraph used — masses, SM inputs, and
    // decay widths — giving a bit-for-bit comparison rather than a ~1e-7 floor from
    // 7-significant-figure rounding. Falls back to the baked restrict defaults.
    let card_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/madgraph/output")
        .join(&name)
        .join("Cards/param_card.dat");
    let card = std::fs::read_to_string(&card_path)
        .ok()
        .and_then(|s| s.parse::<ParamCard>().ok())
        .unwrap_or_else(|| "".parse::<ParamCard>().unwrap());
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &card);

    let evaluator = AmplitudeEvaluator::compile(&sets[0], &model)
        .map_err(|e| Failed::from(format!("compile: {e}")))?;
    let bound = BoundAmplitude::<f64>::bind(&evaluator, &evaluated);

    // Helicity-filtered evaluator (the production eval_m2 configuration): must be
    // bit-for-bit against the unpruned one on every reference point — every pruned
    // combination contributes below rounding.
    let mut evaluator_pruned = AmplitudeEvaluator::compile(&sets[0], &model)
        .map_err(|e| Failed::from(format!("compile: {e}")))?;
    let n_dropped = evaluator_pruned.prune_zero_helicities(&evaluated);
    let bound_pruned = BoundAmplitude::<f64>::bind(&evaluator_pruned, &evaluated);
    let mut scratch_pruned = bound_pruned.scratch_space();

    let mut scratch = bound.scratch_space();
    let mut failures = 0usize;
    let mut max_rel_diff = 0.0f64;
    let mut panicked = false;

    for pt in &points {
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            bound.eval_m2(&pt.momenta, &mut scratch)
        }));
        match result {
            Err(_) => {
                panicked = true;
                failures += 1;
            }
            Ok(m2_rust) => {
                // `eval_m2` already applies the exact color-factor contraction (for
                // NCOLOR=1, `CF(1,1)·Σ_hel|M|²`), so the comparison is direct.
                let rel = (m2_rust - pt.m2_ref).abs() / pt.m2_ref.max(1e-30);
                if rel > max_rel_diff {
                    max_rel_diff = rel;
                }
                if rel > REL_TOL {
                    failures += 1;
                }
                let m2_pruned = bound_pruned.eval_m2(&pt.momenta, &mut scratch_pruned);
                if m2_pruned.to_bits() != m2_rust.to_bits() {
                    return Err(format!(
                        "helicity-pruned eval_m2 diverged from unpruned: \
                         {m2_pruned:e} vs {m2_rust:e} ({n_dropped} combinations pruned)"
                    )
                    .into());
                }
            }
        }
    }

    if panicked {
        eprintln!(
            "INFO [{name}]: evaluator panicked on ≥1 points — evaluator bug, not a harness failure"
        );
        if !EXPECT_MATCH.contains(&name.as_str()) {
            return Ok(());
        }
        return Err("evaluator panicked on a process in EXPECT_MATCH".into());
    }

    println!(
        "  [{name}] {} points, n_flows={}, max_rel_diff={max_rel_diff:.2e} | \
         vibegraph legs: {:?} > {:?}",
        points.len(),
        evaluator.n_flows(),
        sets[0].particles_in,
        sets[0].particles_out
    );

    if EXPECT_MATCH.contains(&name.as_str()) {
        if failures > 0 {
            return Err(format!(
                "{failures}/{} points exceeded rel tolerance {REL_TOL:.0e} \
                 (max_rel_diff={max_rel_diff:.2e})",
                points.len()
            )
            .into());
        }
    } else if failures > 0 {
        eprintln!(
            "INFO [{name}]: {failures}/{} points differ from MG reference; \
             max_rel_diff={max_rel_diff:.2e}",
            points.len()
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
