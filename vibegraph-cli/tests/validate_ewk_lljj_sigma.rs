//! σ of electroweak `p p > e+ e- j j` with no pT cut, against MadEvent's seeds.
//!
//! Row `pp_to_epemjj_qcd0`: `p p > e+ e- j j QCD=0 QED=4 $ h` in the
//! five-flavour `sm-no_b_mass`, on the committed
//! `validation/madgraph/pp_to_epemjj_qcd0_run_card.dat` (13 TeV, lhaid 247000,
//! `ptj = ptl = 0`, no rapidity or ΔR cut, `mmjj = 4`, `mmll = 50`, the
//! kT-clustering scale with MadEvent's 2 GeV floor on μF). The process line
//! and the model are read from `validation/madgraph/scripts/pp_to_epemjj_qcd0.mg5`,
//! the script the MadGraph side is generated from, so the two sides cannot
//! name different processes. The MadGraph side is the row's seed record,
//! banked by `validation/madgraph/bank_seeded_sigma.sh` and read through the
//! same reader as every other seeded row (`common/seed_record.rs`): the seeds at
//! the record's budget, on the estimator the record names.
//!
//! This is the regime the peripheral (spine) channels exist for: with no pT or
//! rapidity cut, the t-channel photon and lepton lines reach down to the
//! `mmjj`, `mmll` and μF-floor boundaries, and the γγ → ℓℓ corner with soft
//! photon fractions carries a large share of the weight variance.
//!
//! # The gate
//!
//! Both sides are read under the seed policy (`common/seeds.rs`): the seeds'
//! unweighted mean with `max(quoted, spread / √n)`, and their χ²/dof as a
//! separate scatter check. The gate holds when
//!
//! * `|pull| < 3` against the record;
//! * `|rel| <` [`MAX_REL`], the scale this side's measured seed spread can
//!   resolve at the gate's budget;
//! * this side's seed χ²/dof lies inside [`CHI2_PER_DOF_BAND`]: above it the
//!   seeds scatter beyond what they quote (a heavy tail the grid has not
//!   learnt), below it they quote errors their scatter does not show.
//!
//! The checks are evaluated together ([`verdict`]) and every one that fails is
//! named, so a σ disagreement does not hide a scatter failure or the reverse.
//! A missing seed record fails before anything is integrated
//! ([`vibegraph::validation::require`]), as on the other seeded rows.
//!
//! The integration is the long tier: `#[ignore]`, run by
//! `pixi run validate-ewk-lljj-sigma`. The checks themselves also run in the
//! banked layer on a fixed five-seed reading ([`RECORDED_SEEDS`]): against the
//! banked record they pass, and against a record shifted by 3%, a band that
//! excludes the reading's χ²/dof, or a work area holding no record they fail.
//!
//! # What this gate cannot see
//!
//! A σ is blind to a mis-sampled region whose share of σ is below the gate's
//! resolution; the seed χ²/dof, read over five seeds, catches a region some
//! seeds find and others miss, not one that every seed misses. It is blind to
//! the partition of σ over subprocesses and channels (a per-subprocess
//! comparison against the record's `by_channel` values would see that), and to
//! the per-event scale (validate_scales pins the kT-clustering scale on other
//! rows). Both sides use the model's default parameter card; this gate does not
//! compare the cards themselves. The banked-layer checks on [`RECORDED_SEEDS`]
//! see the record and the verdict, never this side's integration.

mod common;
use common::PdfDataPath;

use std::path::{Path, PathBuf};

use common::validation_dir;
use vibegraph::artifact::IntegrateArtifact;
use vibegraph::vegas::Chi2PerDof;

#[path = "../../vibegraph-lib/tests/common/seeds.rs"]
mod seeds;

#[path = "../../vibegraph-lib/tests/common/seed_record.rs"]
mod seed_record;

#[path = "../../vibegraph-lib/tests/common/report.rs"]
mod report;

use report::{IntegralsRow, SeedResult, Stopwatch};
use seed_record::SeedReference;
use seeds::{combine, SeedMean, SeedValue};

impl SeedValue for SeedResult {
    fn value_and_err(&self) -> (f64, f64) {
        (self.sigma_pb, self.sigma_err_pb)
    }
}

const ROW: &str = "pp_to_epemjj_qcd0";

/// The integration budget per seed: about 0.8 million points over twelve
/// iterations (68 channels, most at the 512-point floor). On these seeds the
/// mean moves by +0.04% ± 0.4% between twelve and twenty-four iterations at
/// this `--neval`, where it falls by 2% between three and twelve; one seed
/// scatters by about 1%.
const NEVAL: &str = "40000";
const NITER: &str = "12";
const SEEDS: [u64; 5] = [20261031, 20261032, 20261033, 20261034, 20261035];

const MAX_PULL: f64 = 3.0;
/// About three times the combined error of two five-seed means at this side's
/// measured one-seed scatter of 1%, with MadEvent's at a similar size.
const MAX_REL: f64 = 0.02;
/// The 0.1% and 99.9% quantiles of χ²/dof on four degrees of freedom.
const CHI2_PER_DOF_BAND: (f64, f64) = (0.023, 4.617);

/// `(seed, σ, quoted error)` of [`SEEDS`] as the long-tier gate measured them
/// at [`NEVAL`] × [`NITER`], to the digits it prints (the row's manifest note
/// records the run). The banked-layer checks below read the verdict on these.
const RECORDED_SEEDS: [(u64, f64, f64); 5] = [
    (20261031, 3.6111418, 3.11e-2),
    (20261032, 3.5834316, 3.63e-2),
    (20261033, 3.5822541, 2.43e-2),
    (20261034, 3.5210295, 2.14e-2),
    (20261035, 3.6042065, 2.39e-2),
];

fn output_dir() -> PathBuf {
    validation_dir().join("output")
}

fn pdf_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/pdf")
}

/// The `import model` and process lines of the row's MadGraph script: every
/// non-comment line before its `output`.
fn proc_card_text() -> String {
    let path = validation_dir().join("scripts").join(format!("{ROW}.mg5"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .take_while(|l| !l.starts_with("output"))
        .collect();
    assert!(
        lines.iter().any(|l| l.starts_with("import model"))
            && lines.iter().any(|l| l.starts_with("generate")),
        "{} names no model and process before its output line: {lines:?}",
        path.display()
    );
    lines.join("\n") + "\n"
}

fn run_card() -> PathBuf {
    validation_dir().join(format!("{ROW}_run_card.dat"))
}

/// `integrate` the row at `seed`, at [`NEVAL`] × [`NITER`].
fn integrate(proc_card: &str, seed: u64) -> IntegrateArtifact {
    let tmp = tempfile::tempdir().expect("temporary directory");
    let card = tmp.path().join("proc_card.dat");
    std::fs::write(&card, proc_card).expect("write the proc card");
    let out = tmp.path().join("out");
    let output = common::vibegraph_ambient()
        .arg("integrate")
        .arg(&card)
        .arg("--run-card")
        .arg(run_card())
        .arg("--out")
        .arg(&out)
        .pdf_data_path(pdf_dir())
        .args(["--fixed-budget", "--neval", NEVAL, "--niter", NITER])
        .args(["--seed", &seed.to_string()])
        .output()
        .expect("spawn vibegraph");
    assert!(
        output.status.success(),
        "vibegraph integrate failed:\n{}",
        common::stderr_of(&output)
    );
    IntegrateArtifact::read_from_path(&out.join("grid.bin.zst")).expect("read the artifact")
}

/// The row's seed record under `output_dir`; a work area without one fails.
fn reference(output_dir: &Path) -> SeedReference {
    seed_record::reference(output_dir, ROW).unwrap_or_else(|| {
        vibegraph::validation::require(
            "validate_ewk_lljj_sigma",
            "the row's seed record",
            seed_record::record_path(output_dir, ROW).display(),
        )
    })
}

/// The gate's reading of this side against the record.
struct Verdict {
    pull: f64,
    rel: f64,
    /// One line per check that failed; empty when the gate holds.
    failures: Vec<String>,
}

/// Every check of the gate on `side` against `reference`, with the scatter
/// read against `band`.
fn verdict(side: &SeedMean, reference: &SeedReference, band: (f64, f64)) -> Verdict {
    let (mg, mg_err) = (reference.mean(), reference.err());
    let pull = side.pull(mg, mg_err);
    let rel = side.mean / mg - 1.0;
    let mut failures = Vec::new();
    if pull.abs() >= MAX_PULL {
        failures.push(format!(
            "|pull| = {:.2} is not below {MAX_PULL}",
            pull.abs()
        ));
    }
    if rel.abs() >= MAX_REL {
        failures.push(format!("|rel| = {:.5} is not below {MAX_REL}", rel.abs()));
    }
    let chi2 = side.chi2_per_dof;
    let inside = chi2 > band.0 && chi2 < band.1;
    if !inside {
        failures.push(format!(
            "this side's seed χ²/dof {chi2:.2} lies outside {band:?}"
        ));
    }
    Verdict {
        pull,
        rel,
        failures,
    }
}

/// `p p > e+ e- j j QCD=0` with no pT cut against MadEvent's seed record.
#[test]
#[ignore = "long tier: pixi run validate-ewk-lljj-sigma"]
fn ewk_epemjj_without_pt_cuts_matches_madevents_seeds() {
    let proc_card = proc_card_text();
    let reference = reference(&output_dir());
    let clock = Stopwatch::start();
    eprintln!(
        "[{ROW}] {} on {}, --fixed-budget --neval {NEVAL} --niter {NITER}, seeds {SEEDS:?}",
        proc_card.trim().replace('\n', "; "),
        run_card().display()
    );
    let runs: Vec<SeedResult> = SEEDS
        .iter()
        .map(|&seed| {
            let seed_clock = Stopwatch::start();
            let artifact = integrate(&proc_card, seed);
            eprintln!(
                "  seed {seed}: σ = {:.7e} ± {:.2e} pb (χ²/dof {:.2}), {:.0} s",
                artifact.sigma_pb,
                artifact.sigma_err_pb,
                Chi2PerDof(artifact.chi2_per_dof),
                seed_clock.seconds()
            );
            SeedResult {
                seed,
                sigma_pb: artifact.sigma_pb,
                sigma_err_pb: artifact.sigma_err_pb,
            }
        })
        .collect();
    let side = combine(&runs);
    let SeedMean {
        mean,
        err,
        quoted,
        spread,
        chi2_per_dof: chi2,
        ..
    } = side;
    let Verdict {
        pull,
        rel,
        failures,
    } = verdict(&side, &reference, CHI2_PER_DOF_BAND);
    let (mg, mg_err) = (reference.mean(), reference.err());
    eprintln!(
        "[{ROW}] GATE vibegraph σ = {mean:.7e} ± {err:.2e} pb ({} seeds, quoted {quoted:.2e}, \
         spread {spread:.2e}, χ²/dof = {chi2:.2}, band {CHI2_PER_DOF_BAND:?}), {:.0} s | \
         MG σ = {mg:.7e} ± {mg_err:.2e} pb | pull = {pull:+.2} | rel = {rel:+.5} | {}",
        runs.len(),
        clock.seconds(),
        if failures.is_empty() {
            "PASS".to_string()
        } else {
            format!("FAIL: {}", failures.join("; "))
        }
    );

    let mut cell = IntegralsRow::new(ROW, "p p > e+ e- j j QCD=0", "gate");
    cell.status = if failures.is_empty() { "pass" } else { "fail" };
    cell.sigma_vg_pb = mean;
    cell.sigma_vg_err_pb = err;
    cell.sigma_mg_pb = mg;
    cell.sigma_mg_err_pb = mg_err;
    cell.pull = pull;
    cell.rel = rel;
    cell.chi2_dof = chi2;
    cell.seeds = runs.iter().map(|r| r.seed).collect();
    cell.per_seed = runs.clone();
    cell.neval = NEVAL.parse().unwrap();
    cell.niter = NITER.parse().unwrap();
    cell.note = Some(reference.describe(ROW));
    cell.duration_s = Some(clock.seconds());
    cell.write();

    assert!(
        failures.is_empty(),
        "[{ROW}] vibegraph {mean:.7e} ± {err:.2e} (χ²/dof {chi2:.2}) against MadGraph \
         {mg:.7e} ± {mg_err:.2e} pb: {} over {runs:?}",
        failures.join("; ")
    );
}

fn recorded_side() -> SeedMean {
    let values: Vec<(f64, f64)> = RECORDED_SEEDS.iter().map(|&(_, v, e)| (v, e)).collect();
    combine(&values)
}

/// A copy of the banked record in a fresh work area with every seed's values
/// scaled by `factor`; the directory holds it under the reader's own layout.
fn scaled_record(factor: f64) -> tempfile::TempDir {
    let path = seed_record::record_path(&output_dir(), ROW);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        vibegraph::validation::require(
            "validate_ewk_lljj_sigma",
            "the row's seed record",
            format!("{}: {e}", path.display()),
        )
    });
    let mut record: serde_json::Value = serde_json::from_str(&text).expect("the record parses");
    let seeds = record["seeds"].as_array_mut().expect("the record's seeds");
    assert!(!seeds.is_empty());
    for seed in seeds {
        for key in [
            "reported_pb",
            "reported_err_pb",
            "equal_weight_pb",
            "equal_weight_err_pb",
        ] {
            let v = seed[key].as_f64().expect("a seed value");
            seed[key] = serde_json::json!(v * factor);
        }
    }
    let tmp = tempfile::tempdir().expect("temporary directory");
    let path = seed_record::record_path(tmp.path(), ROW);
    std::fs::create_dir_all(path.parent().unwrap()).expect("the record's directory");
    std::fs::write(&path, serde_json::to_string(&record).unwrap()).expect("write the record");
    tmp
}

/// The recorded reading passes against the banked record: the control the
/// failing cases below are read against.
#[test]
fn the_recorded_seeds_pass_against_the_banked_record() {
    let reference = reference(&output_dir());
    let v = verdict(&recorded_side(), &reference, CHI2_PER_DOF_BAND);
    eprintln!(
        "[{ROW}] recorded seeds: pull {:+.2}, rel {:+.5}, failures {:?}",
        v.pull, v.rel, v.failures
    );
    assert!(v.failures.is_empty(), "{:?}", v.failures);
}

/// A reference 3% away, either way, fails on σ alone.
#[test]
fn a_reference_shifted_by_three_percent_fails_on_sigma() {
    let banked = reference(&output_dir()).mean();
    for factor in [1.03, 0.97] {
        let area = scaled_record(factor);
        let shifted = reference(area.path());
        assert!(((shifted.mean() / banked) / factor - 1.0).abs() < 1e-12);
        let v = verdict(&recorded_side(), &shifted, CHI2_PER_DOF_BAND);
        eprintln!(
            "[{ROW}] reference x{factor}: pull {:+.2}, rel {:+.5}, failures {:?}",
            v.pull, v.rel, v.failures
        );
        assert!(
            v.failures.iter().any(|f| f.starts_with("|rel|"))
                && v.failures.iter().all(|f| !f.contains("χ²/dof")),
            "x{factor}: {:?}",
            v.failures
        );
    }
}

/// A band whose upper edge sits below the reading's χ²/dof fails on the
/// scatter alone.
#[test]
fn a_band_that_excludes_the_measured_scatter_fails_on_it() {
    let reference = reference(&output_dir());
    let side = recorded_side();
    let band = (CHI2_PER_DOF_BAND.0, side.chi2_per_dof / 2.0);
    let v = verdict(&side, &reference, band);
    eprintln!(
        "[{ROW}] band {band:?} on χ²/dof {:.2}: failures {:?}",
        side.chi2_per_dof, v.failures
    );
    assert_eq!(v.failures.len(), 1, "{:?}", v.failures);
    assert!(v.failures[0].contains("χ²/dof"), "{:?}", v.failures);
}

/// A work area without the record fails rather than passing on nothing.
#[test]
#[should_panic(expected = "needs the row's seed record")]
fn a_missing_record_fails() {
    let empty = tempfile::tempdir().expect("temporary directory");
    reference(empty.path());
}
