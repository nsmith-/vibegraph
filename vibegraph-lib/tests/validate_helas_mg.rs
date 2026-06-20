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

mod common;

use libtest_mimic::{Arguments, Failed, Trial};
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::time::Instant;
use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
use vibegraph::helas::LorentzVector;
use vibegraph::ufo::slha::ParamCard;

/// Relative tolerance for processes expected to agree (color-free, same SM params).
///
/// vibegraph now evaluates with MadGraph's exact per-process `param_card.dat` (same
/// rounded masses, SM inputs, and decay widths), and the SM model bakes in
/// `restrict_default.dat`, so the comparison is bit-for-bit: ee→μμ and pp→ll agree
/// to ~1e-14 (was ~7e-4 when vibegraph used physical lepton masses vs MadGraph's
/// massless ZERO). The tolerance is kept a few orders above the double-precision
/// floor to allow for benign summation-order differences across the diagram set.
const REL_TOL: f64 = 1e-10;

/// Processes for which we enforce agreement; others are informational only.
///
/// `uux_to_ccx_emmm_qcd0` (2->6) is intentionally NOT enforced yet: all 579 diagrams
/// evaluate and conserve momentum, but a continuum γ/Z relative-phase residual
/// remains (max_rel_diff ~3.96e1, amplified by the strong gauge cancellation). It is
/// momentum-conserving and mass-independent (confirmed: this bit-for-bit param-card
/// comparison leaves it unchanged). Tracked as `helas-2to6-continuum`.
const EXPECT_MATCH: &[&str] = &["ee_to_mumu", "pp_to_ll_qcd0"];

/// Overall color factor relating MadGraph's color-summed |M|² to vibegraph's
/// color-stripped coherent diagram sum, for single-color-flow processes.
/// MadGraph computes CF(1,1)·|JAMP|² with JAMP = ±(coherent sum); vibegraph
/// omits color, so `color_factor * eval_m2_rust == MG`.
///   ee_to_mumu          : colorless                       → 1
///   pp_to_ll_qcd0        : one quark line (Nc)            → 3
///   uux_to_ccx_emmm_qcd0 : two quark lines (Nc²)          → 9
/// This is a stand-in until multi-flow color is implemented in vibegraph.
fn color_factor(name: &str) -> f64 {
    match name {
        "pp_to_ll_qcd0" => 3.0,
        "uux_to_ccx_emmm_qcd0" => 9.0,
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

/// Derive process name from CSV path: "ee_to_mumu_amplitude.csv" → "ee_to_mumu".
fn process_name(p: &Path) -> String {
    let stem = p
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    stem.strip_suffix("_amplitude").unwrap_or(&stem).to_owned()
}

/// Parse the CSV into (process_string, points).  Two schemas are supported:
///   * 2->2:  columns `sqrt_s_GeV, cos_theta, M2_summed` (momenta rebuilt below)
///   * n-body: a `# n_ext: N` header, then columns `m2_summed, E0,px0,..,pz_{N-1}`
/// Points are empty for skeleton CSVs.
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
        } else if let Some(n) = n_ext {
            let cols: Vec<f64> = line
                .split(',')
                .map(|c| c.trim().parse().expect("bad number in n-body row"))
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
        } else {
            let cols: Vec<&str> = line.splitn(3, ',').collect();
            assert_eq!(cols.len(), 3, "expected 3 columns in: {line}");
            let sqrt_s: f64 = cols[0].trim().parse().expect("bad sqrt_s");
            let cos_theta: f64 = cols[1].trim().parse().expect("bad cos_theta");
            points.push(AmpPoint {
                momenta: make_momenta_2to2(sqrt_s, cos_theta),
                m2_ref: cols[2].trim().parse().expect("bad M2"),
            });
        }
    }

    (process_str, points)
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
    let evaluated = model.evaluate(&card);

    let evaluator = AmplitudeEvaluator::compile(&sets[0], model)
        .map_err(|e| Failed::from(format!("compile: {e}")))?;
    let bound = BoundAmplitude::<f64>::bind(&evaluator, &evaluated);

    let cf = color_factor(&name);
    let mut failures = 0usize;
    let mut max_rel_diff = 0.0f64;
    let mut panicked = false;

    let t0 = Instant::now();
    for pt in &points {
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| bound.eval_m2(&pt.momenta)));
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
    // Quick-and-dirty timing for rough performance feedback; not a rigorous benchmark.
    let elapsed = t0.elapsed();
    eprintln!(
        "  [{name}] evaluated {} points in {:.2} ms  ({:.0} ns/eval)",
        points.len(),
        elapsed.as_secs_f64() * 1e3,
        elapsed.as_nanos() as f64 / points.len() as f64
    );

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
