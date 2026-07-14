//! Instruction-node width sensitivity harness.
//!
//! Times `BoundAmplitude::eval_m2` over every process with a MadGraph reference CSV,
//! reporting amortized ns/eval with a per-process spread across repeated timing
//! blocks. The self-reported `size_of::<Node<Const>>()` labels each run, so the same
//! harness measures the current node, padded variants, and the packed node without
//! any code change — the build feature (or the packed `Const`) sets the width.
//!
//! Run (single build/variant), pinning to one core for stable numbers:
//!   cargo test -p vibegraph-lib --profile profiling --features extended-validation \
//!     --test instruction_size_bench -- --test-threads=1 --nocapture
//!
//! Prints machine-readable `RESULT` lines consumed by the A0 aggregation.

mod common;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude, Const, Node};
use vibegraph::helas::LorentzVector;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::EvaluatedModel;

/// One evaluated phase-space point: external momenta (incoming then outgoing).
struct AmpPoint {
    momenta: Vec<LorentzVector<f64>>,
}

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

fn process_name(p: &Path) -> String {
    let stem = p
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    stem.strip_suffix("_amplitude").unwrap_or(&stem).to_owned()
}

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
            header_skipped = true;
        } else {
            let n = n_ext.expect("missing '# n_ext:' header before data rows");
            let cols: Vec<f64> = line
                .split(',')
                .map(|c| c.trim().parse().expect("bad number in row"))
                .collect();
            let momenta = (0..n)
                .map(|i| {
                    let b = 1 + 4 * i;
                    LorentzVector::new(cols[b], cols[b + 1], cols[b + 2], cols[b + 3])
                })
                .collect();
            points.push(AmpPoint { momenta });
        }
    }
    (process_str, points)
}

/// One timing block: amortize `eval_m2` over the point set until either the eval
/// budget or the time budget is met; return ns/eval.
fn time_block(bound: &BoundAmplitude<f64>, points: &[AmpPoint], min_evals: usize) -> f64 {
    const MAX_TIME: Duration = Duration::from_millis(200);
    let mut scratch = bound.scratch_space();
    let mut n_evals = 0usize;
    let mut acc = 0.0f64;
    let t0 = Instant::now();
    'outer: loop {
        for pt in points {
            acc += bound.eval_m2(&pt.momenta, &mut scratch);
            n_evals += 1;
            if n_evals >= min_evals && t0.elapsed() >= MAX_TIME {
                break 'outer;
            }
        }
    }
    let elapsed = t0.elapsed();
    std::hint::black_box(acc);
    elapsed.as_nanos() as f64 / n_evals as f64
}

fn median(xs: &mut [f64]) -> f64 {
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = xs.len();
    if n % 2 == 1 {
        xs[n / 2]
    } else {
        0.5 * (xs[n / 2 - 1] + xs[n / 2])
    }
}

#[test]
fn bench_instruction_size() {
    let node_bytes = std::mem::size_of::<Node<Const>>();
    let const_bytes = std::mem::size_of::<Const>();
    eprintln!("SIZE node_const={node_bytes} const={const_bytes}");

    // Repetitions per process; taken as separate timing blocks so the min is a
    // low-noise estimator and the spread quantifies contention from sibling load.
    const REPS: usize = 11;
    // Warmup evals discarded before timing, to page in scratch and prime caches.
    const MIN_EVALS: usize = 64;

    let model = common::sm_model();

    let mut csvs = find_amplitude_csvs();
    assert!(!csvs.is_empty(), "no amplitude CSVs found");
    csvs.sort_by_key(|p| process_name(p));

    for csv in csvs {
        let name = process_name(&csv);
        let (process_str, points) = read_csv(&csv);
        if process_str.is_empty() || points.is_empty() {
            continue;
        }
        let sets = common::generate(&process_str);
        if sets.is_empty() {
            continue;
        }
        let card_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../validation/madgraph/output")
            .join(&name)
            .join("Cards/param_card.dat");
        let card = std::fs::read_to_string(&card_path)
            .ok()
            .and_then(|s| s.parse::<ParamCard>().ok())
            .unwrap_or_else(|| "".parse::<ParamCard>().unwrap());
        let evaluated = EvaluatedModel::from_model_card(model.clone(), &card);

        let Ok(evaluator) = AmplitudeEvaluator::compile(&sets[0], &model) else {
            continue;
        };
        let bound = BoundAmplitude::<f64>::bind(&evaluator, &evaluated);

        // Warmup block (discarded).
        let _ = time_block(&bound, &points, MIN_EVALS);

        let mut samples: Vec<f64> = (0..REPS)
            .map(|_| time_block(&bound, &points, MIN_EVALS))
            .collect();
        let min = samples.iter().cloned().fold(f64::INFINITY, f64::min);
        let med = median(&mut samples);
        let spread_pct = 100.0 * (med - min) / min;

        eprintln!(
            "RESULT node={node_bytes} proc={name} min={min:.1} median={med:.1} \
             spread_pct={spread_pct:.1} nflows={}",
            evaluator.n_flows()
        );
    }
}
