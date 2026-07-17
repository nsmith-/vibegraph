//! `eval_m2` microbenchmark across process sizes 2→2 … 2→6 (all-massless externals
//! so plain massless RAMBO provides the kinematics), including colored NCOLOR=2/6 2→2s
//! that exercise the CF-weighted multi-flow path. The per-process before/after
//! yardstick for evaluator changes.
//!
//! Run: `cargo bench -p vibegraph-lib --bench eval_strategies`

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::rngs::StdRng;
use rand::SeedableRng;

use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
use vibegraph::helas::LorentzVector;
use vibegraph::phasespace::rambo_massless;
use vibegraph::ufo::sm::{sm_model, SMRestrict};
use vibegraph::ufo::EvaluatedModel;

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
    }
    group.finish();
}

criterion_group!(benches, bench_eval_m2);
criterion_main!(benches);
