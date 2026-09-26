//! End-to-end gate on decay-chain cross sections: `vibegraph integrate` on a
//! decay-chain proc card (`e+ e- > t t~, t > w+ b, t~ > w- b~`) against MadEvent
//! on the same proc card and run card.
//!
//! The reference is `validation/madgraph/decay_chain_sigma_reference.json`,
//! written by `gen_decay_chain_sigma.sh`: one MadEvent run per seed. The gated
//! quantity is MadEvent's decay-chain cross section itself — every forced
//! resonance held within `bwcutoff` widths of its pole — not σ × BR.
//!
//! Gated behind `extended-validation`: the hadronic row needs the fetched PDF
//! set, and the sweep integrates every row over [`SEEDS`].
//!
//!     cargo test -p vibegraph --profile release-debug --features extended-validation \
//!         --test cli_decay_chain -- --nocapture
//!
//! # What the rows can and cannot see
//!
//! A cross section is blind to anything that moves no weight. What these rows do
//! see: the window itself (a window of the wrong half-width, or on the wrong legs,
//! moves σ by the Breit–Wigner tail it adds or removes, several per cent at
//! `bwcutoff = 15`), a missed region of the resonance-mapped channels, and — on
//! the `cut_decays = T` row — whether the decay products receive the lepton cuts
//! (at `F` the same cuts move nothing). The seed sweep reads the pull of the
//! seed mean, the χ²/dof of this side's seeds about their own mean, and each
//! seed's own pull, since a sampler that misses a region is confidently wrong
//! on every seed at once.
//!
//! `zz_ee` (`e+ e- > z z, z > e+ e-`) has identical particles across its decays.
//! MadGraph keeps one pairing of the four leptons and divides by an
//! identical-decay-chain factor; this generator keeps both pairings, their
//! interference and the final state's own identical-particle factor. The row is
//! printed, not gated.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;
use vibegraph::artifact::IntegrateArtifact;

/// Integration seeds per row.
const SEEDS: [u64; 10] = [11, 22, 33, 44, 55, 66, 77, 88, 99, 110];
/// Per-seed relative target, about one MadEvent run's precision.
const TARGET_REL: &str = "2e-3";
/// The seed mean against MadEvent's, in combined standard errors.
const MEAN_PULL_LIMIT: f64 = 3.0;
/// Any one seed against MadEvent's mean.
const SEED_PULL_LIMIT: f64 = 4.0;
/// This side's seeds about their own mean: the 0.1% and 99.9% points of χ²/dof
/// at nine degrees of freedom.
const SEED_CHI2_RANGE: (f64, f64) = (0.15, 3.1);
/// Rows reported against MadEvent without a gate.
const INFORMATIONAL: &[&str] = &["zz_ee"];

#[derive(Deserialize)]
struct Reference {
    mg_version: String,
    rows: BTreeMap<String, Row>,
}

#[derive(Deserialize)]
struct Row {
    process: String,
    run_card: String,
    runs: Vec<MgRun>,
}

#[derive(Deserialize)]
struct MgRun {
    sigma_pb: f64,
    err_pb: f64,
}

fn madgraph_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph")
}

fn pdf_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/pdf")
}

fn reference() -> Reference {
    let path = madgraph_dir().join("decay_chain_sigma_reference.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    serde_json::from_str(&text).expect("decay_chain_sigma_reference.json parses")
}

/// `(σ, error)` in pb of one seed, read from the artifact rather than the
/// printed line, whose fixed six decimals do not resolve a femtobarn row.
fn integrate(dir: &Path, card: &Path, run_card: &Path, seed: u64) -> (f64, f64) {
    let out = dir.join(format!("run_{seed}"));
    let output = Command::new(env!("CARGO_BIN_EXE_vibegraph"))
        .arg("integrate")
        .arg(card)
        .arg("--run-card")
        .arg(run_card)
        .arg("--pdf-dir")
        .arg(pdf_dir())
        .arg("--out")
        .arg(&out)
        .arg("--force")
        .args(["--target-rel", TARGET_REL, "--seed", &seed.to_string()])
        .current_dir(dir)
        .output()
        .expect("spawn vibegraph");
    assert!(
        output.status.success(),
        "vibegraph integrate {} failed:\n{}",
        card.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    let artifact =
        IntegrateArtifact::read_from_path(&out.join("grid.bin.zst")).expect("reload artifact");
    (artifact.sigma_pb, artifact.sigma_err_pb)
}

/// Inverse-variance mean and its error.
fn weighted_mean(values: &[(f64, f64)]) -> (f64, f64) {
    let w: f64 = values.iter().map(|(_, e)| 1.0 / (e * e)).sum();
    let m = values.iter().map(|(v, e)| v / (e * e)).sum::<f64>() / w;
    (m, 1.0 / w.sqrt())
}

/// The seeds' sample standard deviation.
fn spread(values: &[(f64, f64)]) -> f64 {
    let n = values.len() as f64;
    let m = values.iter().map(|(v, _)| v).sum::<f64>() / n;
    (values.iter().map(|(v, _)| (v - m).powi(2)).sum::<f64>() / (n - 1.0)).sqrt()
}

/// The inverse-variance mean, with an error no smaller than the seed spread
/// allows.
fn seed_mean(values: &[(f64, f64)]) -> (f64, f64) {
    let (m, quoted) = weighted_mean(values);
    (m, quoted.max(spread(values) / (values.len() as f64).sqrt()))
}

/// χ²/dof of `values` about their inverse-variance mean.
fn chi2_per_dof(values: &[(f64, f64)]) -> f64 {
    let (m, _) = weighted_mean(values);
    let chi2: f64 = values.iter().map(|(v, e)| ((v - m) / e).powi(2)).sum();
    chi2 / (values.len() - 1) as f64
}

#[test]
fn decay_chain_cross_sections_match_madevent_over_seeds() {
    let reference = reference();
    let mut failures = Vec::new();
    println!(
        "MadEvent {} decay-chain cross sections against vibegraph, {} seeds each",
        reference.mg_version,
        SEEDS.len()
    );
    for (name, row) in &reference.rows {
        let tmp = tempfile::tempdir().unwrap();
        let card = tmp.path().join("proc_card.dat");
        std::fs::write(
            &card,
            format!("import model sm\ngenerate {}\n", row.process),
        )
        .unwrap();
        let run_card = madgraph_dir().join(&row.run_card);
        let ours: Vec<(f64, f64)> = SEEDS
            .iter()
            .map(|&seed| integrate(tmp.path(), &card, &run_card, seed))
            .collect();
        let mg: Vec<(f64, f64)> = row.runs.iter().map(|r| (r.sigma_pb, r.err_pb)).collect();
        let (mg_mean, mg_err) = seed_mean(&mg);
        let (our_mean, our_err) = seed_mean(&ours);
        let chi2 = chi2_per_dof(&ours);
        let pull = (our_mean - mg_mean) / (our_err.powi(2) + mg_err.powi(2)).sqrt();
        let worst_seed = ours
            .iter()
            .map(|(v, e)| ((v - mg_mean) / (e * e + mg_err * mg_err).sqrt()).abs())
            .fold(0.0f64, f64::max);
        let gated = !INFORMATIONAL.contains(&name.as_str());
        println!(
            "{name:<11} {:<52} MG {mg_mean:.6e} ± {mg_err:.2e} ({} seeds, χ²/dof {:.2}) | \
             ours {our_mean:.6e} ± {our_err:.2e} (χ²/dof {chi2:.2}) | ours/MG − 1 = {:+.3e} \
             ± {:.1e} | pull {pull:+.2}, worst seed {worst_seed:.2}{}",
            row.process,
            mg.len(),
            chi2_per_dof(&mg),
            our_mean / mg_mean - 1.0,
            (our_err.powi(2) + mg_err.powi(2)).sqrt() / mg_mean,
            if gated { "" } else { " (informational)" },
        );
        for (v, e) in &ours {
            println!("            seed {v:.6e} ± {e:.2e}");
        }
        if gated
            && (pull.abs() > MEAN_PULL_LIMIT
                || worst_seed > SEED_PULL_LIMIT
                || !(SEED_CHI2_RANGE.0..SEED_CHI2_RANGE.1).contains(&chi2))
        {
            failures.push(name.clone());
        }
    }
    assert!(failures.is_empty(), "rows outside the gate: {failures:?}");
}
