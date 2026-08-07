#![allow(dead_code)]

pub mod leshouche;
pub mod manifest;
pub mod pdfset;
pub mod report;

use std::sync::Arc;

use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, DiagramSet, ParsingOptions};
use vibegraph::ufo::sm::{sm_model as interned_sm, SMRestrict};
use vibegraph::ufo::UFOModel;

pub fn ufo_models_dir() -> std::path::PathBuf {
    let manifest = env!("CARGO_MANIFEST_DIR");
    std::path::Path::new(manifest).join("../research/refs/mg5amcnlo/models")
}

pub fn ufo_path(model: &str) -> std::path::PathBuf {
    ufo_models_dir().join(model)
}

pub fn sm_model() -> Arc<UFOModel> {
    interned_sm(SMRestrict::Default)
}

/// SM loaded with the `lepton_masses` restriction (`import model sm-lepton_masses`),
/// which keeps Me/MM/MTA non-zero — unlike `restrict_default`, which locks them to
/// zero. Use this when a test needs settable, physical lepton masses.
pub fn sm_lepton_masses_model() -> Arc<UFOModel> {
    interned_sm(SMRestrict::LeptonMasses)
}

pub fn generate(process: &str) -> Vec<DiagramSet> {
    generate_with(process, sm_model().as_ref())
}

pub fn generate_with(process: &str, model: &UFOModel) -> Vec<DiagramSet> {
    let opts = ParsingOptions::default();
    let card = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
    generate_from_proc_card(&card, model).unwrap()
}

/// The accepted-point floor as a run realised it, one line.
///
/// The floor promises coverage in accepted points and is paid for in drawn ones,
/// so both are printed: the per-channel acceptance the correction read, the spend
/// it bought at, the spend an uncapped correction would have asked for, and the
/// coverage and zero-variance-iteration counts that say whether the promise was
/// kept.
pub fn floor_coverage_line(spend: &vibegraph::budget::ConvergenceReport) -> String {
    use vibegraph::budget::{MAX_FLOOR_ACCEPTANCE_SCALE, MIN_CHANNEL_NEVAL};

    let n = spend.channel_points.len();
    let acceptance: Vec<f64> = spend
        .channel_accepted
        .iter()
        .zip(&spend.channel_points)
        .map(|(&a, &p)| if p > 0 { a as f64 / p as f64 } else { 0.0 })
        .collect();
    let mut sorted = acceptance.clone();
    sorted.sort_by(f64::total_cmp);
    let q = |p: f64| sorted[(((sorted.len() - 1) as f64) * p).round() as usize];
    let dead = acceptance.iter().filter(|&&a| a == 0.0).count();
    let uncapped: f64 = acceptance
        .iter()
        .filter(|&&a| a > 0.0)
        .map(|&a| (MIN_CHANNEL_NEVAL as f64 / a).ceil())
        .sum::<f64>()
        + (dead * MIN_CHANNEL_NEVAL * 1000) as f64;
    format!(
        "{n} channels | acceptance min {:.4} p10 {:.4} p50 {:.4} p90 {:.4} max {:.4} \
         | zero-acceptance {dead} | capped (<1/{MAX_FLOOR_ACCEPTANCE_SCALE}) {} \
         | floor spend {MIN_CHANNEL_NEVAL}×n {} → realised {}/iter (uncapped floors would ask {:.3e}) \
         | min accepted/channel/iter {} | zero-variance kept iters {} \
         | points {} | achieved_rel {:.5} scaled_rel {:.5}",
        q(0.0),
        q(0.10),
        q(0.50),
        q(0.90),
        q(1.0),
        spend.floor_capped_channels,
        n * MIN_CHANNEL_NEVAL,
        spend.points_per_iteration,
        uncapped,
        spend.min_channel_accepted,
        spend.zero_variance_iterations,
        spend.points,
        spend.achieved_rel,
        spend.scaled_rel,
    )
}
