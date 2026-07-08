//! Benchmark AmplitudeEvaluator::eval_m2 on N random ee→μμ kinematic points.
//!
//! Run (release-optimised, same as cargo bench default):
//!   cargo bench -p vibegraph-lib --bench amplitude_eval
//!
//! Compare to MadGraph Fortran MATRIX1:
//!   pixi run -e madgraph generate-amplitude

use std::time::Instant;
use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
use vibegraph::helas::{
    eval::{AmplitudeEvaluator, BoundAmplitude},
    LorentzVector,
};
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::sm::{sm_model, SMRestrict};

const N: usize = 10_000;

fn make_momenta(sqrt_s: f64, cos_theta: f64) -> Vec<LorentzVector<f64>> {
    let e = sqrt_s / 2.0;
    let sin_t = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
    vec![
        LorentzVector::new(e, 0.0, 0.0, -e),
        LorentzVector::new(e, 0.0, 0.0, e),
        LorentzVector::new(e, -e * sin_t, 0.0, -e * cos_theta),
        LorentzVector::new(e, e * sin_t, 0.0, e * cos_theta),
    ]
}

fn main() {
    let model = sm_model(SMRestrict::Default);

    let opts = ParsingOptions::default();
    let card = parse_proc_card("generate e+ e- > mu+ mu-", &opts).unwrap();
    let sets = generate_from_proc_card(&card, &model).unwrap();

    let empty_card = "".parse::<ParamCard>().unwrap();
    let evaluated = model.evaluate(&empty_card);
    let evaluator = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
    let bound = BoundAmplitude::<f64>::bind(&evaluator, &evaluated);

    // Fixed-seed random kinematic points, same range as gen_amplitude.py
    use rand::{Rng, SeedableRng};
    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
    let batch: Vec<Vec<LorentzVector<f64>>> = (0..N)
        .map(|_| {
            let r1: f64 = rng.random();
            let r2: f64 = rng.random();
            make_momenta(10.0 + r1 * 190.0, -0.9 + r2 * 1.8)
        })
        .collect();

    // Warm-up: one call to trigger any lazy initialisation
    let _ = bound.eval_m2(&batch[0]);

    let t0 = Instant::now();
    for momenta in &batch {
        let _ = bound.eval_m2(momenta);
    }
    let elapsed = t0.elapsed();

    println!(
        "AmplitudeEvaluator (ee->mumu): {N} evals in {:.2} ms  ({:.0} ns/eval)",
        elapsed.as_secs_f64() * 1e3,
        elapsed.as_nanos() as f64 / N as f64,
    );
}
