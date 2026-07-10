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
//!              --test validate_helas_mg
//!
//! Prerequisites:
//!   pixi run -e madgraph build-amplitude
//!   pixi run -e madgraph generate-amplitude
//!
//! Each trial also reports an amortized evaluator timing next to MadGraph's
//! MATRIX1 timing (`output/mg_timings.json`, written by gen_amplitude.py).
//! Trials run concurrently by default and contend for cores — pass
//! `--test-threads=1` when the timing numbers matter.

mod common;

use libtest_mimic::{Arguments, Failed, Trial};
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::time::Instant;
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
];

/// Overall color factor relating MadGraph's color-summed |M|² to vibegraph's
/// color-stripped coherent diagram sum, for single-color-flow processes.
/// MadGraph computes CF(1,1)·|JAMP|² with JAMP = ±(coherent sum); vibegraph
/// omits color, so `color_factor * eval_m2_rust == MG`.
///   colorless (leptons/EW bosons only)                    → 1
///   one quark line (Nc): pp_to_ll_qcd0, ee_to_ttx         → 3
///   two quark lines (Nc²): uux/bbx 2→6                    → 9
/// This is a stand-in until multi-flow color is implemented in vibegraph.
fn color_factor(name: &str) -> f64 {
    match name {
        "pp_to_ll_qcd0" | "ee_to_ttx" => 3.0,
        "uux_to_ccx_emmm_qcd0" | "bbx_to_ccx_emmm_qcd0" => 9.0,
        _ => 1.0,
    }
}

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

/// MATRIX1 ns/eval per process, from the timing table gen_amplitude.py writes
/// alongside the CSVs. Empty if the table is absent (pre-timing reference data).
fn read_mg_timings() -> std::collections::HashMap<String, f64> {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/output/mg_timings.json");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Default::default();
    };
    let Ok(table) = serde_json::from_str::<serde_json::Value>(&content) else {
        return Default::default();
    };
    table
        .as_object()
        .map(|m| {
            m.iter()
                .filter_map(|(name, t)| Some((name.clone(), t.get("ns_per_eval")?.as_f64()?)))
                .collect()
        })
        .unwrap_or_default()
}

/// Amortized evaluator timing: repeat the CSV point set until the eval budget is
/// met, so the short validation samples still give a stable ns/eval. Returns
/// (evals performed, elapsed).
fn time_evaluator(
    bound: &BoundAmplitude<f64>,
    points: &[AmpPoint],
) -> (usize, std::time::Duration) {
    const TARGET_EVALS: usize = 2_000;
    const MAX_TIME: std::time::Duration = std::time::Duration::from_secs(1);
    let mut scratch = bound.scratch_space();
    let mut n_evals = 0usize;
    let mut acc = 0.0f64;
    let t0 = Instant::now();
    'outer: loop {
        for pt in points {
            acc += bound.eval_m2(&pt.momenta, &mut scratch);
            n_evals += 1;
            if n_evals >= TARGET_EVALS || t0.elapsed() >= MAX_TIME {
                break 'outer;
            }
        }
    }
    let elapsed = t0.elapsed();
    std::hint::black_box(acc);
    (n_evals, elapsed)
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

    let cf = color_factor(&name);
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
            Ok(raw) => {
                let m2_rust = cf * raw;
                let rel = (m2_rust - pt.m2_ref).abs() / pt.m2_ref.max(1e-30);
                if rel > max_rel_diff {
                    max_rel_diff = rel;
                }
                if rel > REL_TOL {
                    failures += 1;
                }
            }
        }
    }

    if !panicked {
        // Rough performance feedback vs MadGraph; see the header note on
        // `--test-threads=1` for meaningful numbers.
        let (n_evals, elapsed) = time_evaluator(&bound, &points);
        let rust_ns = elapsed.as_nanos() as f64 / n_evals as f64;
        match read_mg_timings().get(&name) {
            Some(mg_ns) => eprintln!(
                "  [{name}] timing: rust {rust_ns:.0} ns/eval | MG {mg_ns:.0} ns/eval | \
                 ratio {:.2}x  ({n_evals} evals)",
                rust_ns / mg_ns
            ),
            None => eprintln!(
                "  [{name}] timing: rust {rust_ns:.0} ns/eval | MG n/a  ({n_evals} evals)"
            ),
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
        "  [{name}] {} points, color_factor={cf}, max_rel_diff={max_rel_diff:.2e} | \
         vibegraph legs: {:?} > {:?}",
        points.len(),
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
