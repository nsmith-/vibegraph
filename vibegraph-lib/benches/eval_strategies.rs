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
//! Run: `cargo bench -p vibegraph-lib --bench eval_strategies`

use criterion::{criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion};
use rand::rngs::StdRng;
use rand::SeedableRng;

use numeric_array::generic_array::typenum::Const;
use numeric_array::generic_array::IntoArrayLength;
use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
use vibegraph::helas::eval::{eval_m2_lanes, AmplitudeEvaluator, BoundAmplitude, LaneField};
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
                for chunk in pts.chunks_exact(N) {
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

const PROCESSES: [(&str, &str); 7] = [
    ("ee_to_mumu", "e+ e- > mu+ mu-"),
    ("ee_to_ee", "e+ e- > e+ e-"),
    ("uux_to_uux", "u u~ > u u~"),
    ("gg_to_gg", "g g > g g"),
    ("ee_to_mumua", "e+ e- > mu+ mu- a"),
    ("ee_to_mumu_tata_qcd0", "e+ e- > mu+ mu- ta+ ta- QCD=0"),
    ("uux_to_ccx_emmm_qcd0", "u u~ > c c~ e+ e- mu+ mu- QCD=0"),
];

fn bench_eval_m2(c: &mut Criterion) {
    let model = sm_model(SMRestrict::Default);
    let evaluated = EvaluatedModel::from_model(model.clone());
    let opts = ParsingOptions::default();
    let mut rng = StdRng::seed_from_u64(0xBE7C4);
    let sqrt_s = 500.0;

    let mut group = c.benchmark_group("eval_m2");
    group.sample_size(10);
    for (name, process) in PROCESSES {
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
    }
    group.finish();
}

criterion_group!(benches, bench_eval_m2);
criterion_main!(benches);
