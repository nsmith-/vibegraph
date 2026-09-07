//! `eval_m2` microbenchmark across process sizes 2→2 … 2→6 (all-massless externals
//! so plain massless RAMBO provides the kinematics), including colored NCOLOR=2/6 2→2s
//! that exercise the CF-weighted multi-flow path. The per-process before/after
//! yardstick for evaluator changes.
//!
//! The `forward` benchmark is the scalar `eval_m2`; `lanes{N}` runs the SIMD
//! lane-batched [`eval_m2_lanes`] with `F = NumericArray<f64, N>` over the same
//! points, chunked `N` at a time. Comparing `lanes{N}` to `forward` at equal
//! per-point work measures the SIMD speedup and the best width `N` for the host.
//!
//! `set_alpha_s` moves the same amplitude to as many strong couplings as `forward`
//! evaluates points, so the price of a per-event renormalisation scale reads off as the
//! ratio of the two bars.
//!
//! Run: `cargo bench -p vibegraph-lib --bench eval_strategies`

use criterion::{criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion};
use rand::rngs::StdRng;
use rand::SeedableRng;

use numeric_array::generic_array::typenum::Const;
use numeric_array::generic_array::IntoArrayLength;
use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
use vibegraph::helas::eval::{
    eval_m2_lanes, eval_m2_lanes_packed, pack_lane_points, AmplitudeEvaluator, BoundAmplitude,
    LaneField, ScaleAwareAmplitude,
};
use vibegraph::helas::repr::Real;
use vibegraph::helas::LorentzVector;
use vibegraph::phasespace::rambo_massless;
use vibegraph::ufo::sm::{sm_model, SMRestrict};
use vibegraph::ufo::EvaluatedModel;

/// Register a `lanes{N}` benchmark: rebind onto an `N`-wide lane pack once, then sum
/// `eval_m2_lanes` over the points chunked `N` at a time (points count is a multiple
/// of every swept `N`).
fn bench_lanes<const N: usize>(
    group: &mut BenchmarkGroup<'_, criterion::measurement::WallTime>,
    name: &str,
    amp: &BoundAmplitude<'_, f64>,
    points: &[Vec<LorentzVector<f64>>],
) where
    Const<N>: IntoArrayLength,
    LaneField<N>: Real,
{
    let lane_amp = amp.broadcast_lanes::<N>();
    let mut scratch = lane_amp.scratch_space();
    group.bench_with_input(
        BenchmarkId::new(format!("lanes{N}"), name),
        points,
        |b, pts| {
            b.iter(|| {
                let mut acc = 0.0;
                for chunk in pts.as_chunks::<N>().0 {
                    let refs: [&[LorentzVector<f64>]; N] =
                        std::array::from_fn(|k| chunk[k].as_slice());
                    acc += eval_m2_lanes(&lane_amp, &refs, &mut scratch)
                        .iter()
                        .sum::<f64>();
                }
                acc
            })
        },
    );
}

/// `lanes{N}` with the AoS→SoA transpose hoisted out of the timed region: the same
/// evaluation over the same points, entered through [`eval_m2_lanes_packed`] on
/// momenta packed once up front. Against the `lanes{N}` bar it prices the transpose —
/// which `eval_m2_lanes` performs, and heap-allocates, once per chunk.
fn bench_lanes_prepacked<const N: usize>(
    group: &mut BenchmarkGroup<'_, criterion::measurement::WallTime>,
    name: &str,
    amp: &BoundAmplitude<'_, f64>,
    points: &[Vec<LorentzVector<f64>>],
) where
    Const<N>: IntoArrayLength,
    LaneField<N>: Real,
{
    let lane_amp = amp.broadcast_lanes::<N>();
    let mut scratch = lane_amp.scratch_space();
    let packed: Vec<Vec<LorentzVector<LaneField<N>>>> = points
        .as_chunks::<N>()
        .0
        .iter()
        .map(|chunk| {
            let refs: [&[LorentzVector<f64>]; N] = std::array::from_fn(|k| chunk[k].as_slice());
            pack_lane_points(&refs)
        })
        .collect();
    group.bench_with_input(
        BenchmarkId::new(format!("lanes{N}_prepacked"), name),
        &packed,
        |b, chunks| {
            b.iter(|| {
                let mut acc = 0.0;
                for momenta in chunks {
                    acc += eval_m2_lanes_packed(&lane_amp, momenta, &mut scratch)
                        .iter()
                        .sum::<f64>();
                }
                acc
            })
        },
    );
}

/// The processes to benchmark: every `validation/manifest.toml` row that
/// carries an `mg_amplitude` table, in the manifest's own row order — the
/// same registry `validation/madgraph/gen_amplitude.py` compiles MATRIX1
/// timings from, so this bench and `mg_timings.json` cover exactly the same
/// set. `amplitude_oracle.rs` carries no such list of its own: it reads every
/// file under `validation/madgraph/amplitudes/`, so it needs no synchronising.
#[derive(serde::Deserialize)]
struct ManifestProcess {
    key: String,
    mg_amplitude: Option<MgAmplitude>,
}

#[derive(serde::Deserialize)]
struct MgAmplitude {
    process: String,
}

fn manifest_processes() -> Vec<(String, String)> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/manifest.toml");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    #[derive(serde::Deserialize)]
    struct Manifest {
        #[serde(rename = "process")]
        processes: Vec<ManifestProcess>,
    }
    let manifest: Manifest =
        toml::from_str(&text).unwrap_or_else(|e| panic!("{} does not parse: {e}", path.display()));
    manifest
        .processes
        .into_iter()
        .filter_map(|p| p.mg_amplitude.map(|mg| (p.key, mg.process)))
        .collect()
}

/// Extra `name=card` process rows, semicolon-separated, from
/// `VIBEGRAPH_BENCH_EXTRA_PROCESSES`. Lets a study measure processes the manifest
/// does not carry an `mg_amplitude` table for, without editing the manifest — the
/// row set `scripts/mg_perf_compare.sh` joins against stays exactly
/// [`manifest_processes`]'s output unless the variable is set.
fn extra_processes() -> Vec<(String, String)> {
    let Ok(spec) = std::env::var("VIBEGRAPH_BENCH_EXTRA_PROCESSES") else {
        return Vec::new();
    };
    spec.split(';')
        .filter(|s| !s.trim().is_empty())
        .map(|entry| {
            let (name, card) = entry
                .split_once('=')
                .expect("extra process entry must be name=card");
            (name.trim().to_string(), card.trim().to_string())
        })
        .collect()
}

fn bench_eval_m2(c: &mut Criterion) {
    let model = sm_model(SMRestrict::Default);
    let evaluated = EvaluatedModel::from_model(model.clone());
    let opts = ParsingOptions::default();
    let mut rng = StdRng::seed_from_u64(0xBE7C4);
    let sqrt_s = 500.0;

    let mut all: Vec<(String, String)> = manifest_processes();
    all.extend(extra_processes());

    let mut group = c.benchmark_group("eval_m2");
    group.sample_size(10);
    for (name, process) in &all {
        let pc = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
        let sets = generate_from_proc_card(&pc, &model).unwrap();
        let mut eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
        // MG's timed MATRIX1 is its helicity-recycled code with the helicity filter
        // baked in; prune here so the comparison is like-for-like.
        eval.prune_zero_helicities(&evaluated);
        let fwd = BoundAmplitude::<f64>::bind(&eval, &evaluated);

        let points: Vec<Vec<LorentzVector<f64>>> = (0..16)
            .map(|_| {
                let mut p = vec![
                    LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, sqrt_s / 2.0),
                    LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, -sqrt_s / 2.0),
                ];
                p.extend(rambo_massless(sqrt_s, eval.n_ext() - 2, &mut rng));
                p
            })
            .collect();

        let mut scratch = fwd.scratch_space();
        group.bench_with_input(BenchmarkId::new("forward", name), &points, |b, pts| {
            b.iter(|| {
                pts.iter()
                    .map(|p| fwd.eval_m2(p, &mut scratch))
                    .sum::<f64>()
            })
        });

        bench_lanes::<2>(&mut group, name, &fwd, &points);
        bench_lanes::<4>(&mut group, name, &fwd, &points);
        bench_lanes::<8>(&mut group, name, &fwd, &points);

        bench_lanes_prepacked::<2>(&mut group, name, &fwd, &points);
        bench_lanes_prepacked::<4>(&mut group, name, &fwd, &points);
        bench_lanes_prepacked::<8>(&mut group, name, &fwd, &points);

        // Moving the amplitude to a new strong coupling, over as many couplings as
        // `forward` evaluates points, so the per-event price of a dynamic
        // renormalisation scale is the ratio of the two bars.
        let mut scaled = ScaleAwareAmplitude::<f64>::new(&eval, &evaluated);
        let couplings: Vec<f64> = (0..points.len()).map(|k| 0.08 + 0.01 * k as f64).collect();
        group.bench_with_input(
            BenchmarkId::new("set_alpha_s", name),
            &couplings,
            |b, cs| {
                b.iter(|| {
                    for &c in cs {
                        scaled.set_alpha_s(c);
                    }
                })
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_eval_m2);
criterion_main!(benches);
