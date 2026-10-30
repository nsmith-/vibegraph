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
//! banked by `validation/madgraph/bank_seeded_sigma.sh`.
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
//! While the manifest marks the row `planned` and no seed record exists, the
//! test integrates this side, prints `AWAITING THE BANK` with vibegraph's
//! σ, and writes no cell. Once the record exists it gates; a missing record on
//! a row no longer planned fails ([`vibegraph::validation::require`]).
//!
//! Long tier: `#[ignore]`, run by `pixi run validate-ewk-lljj-sigma`.
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
//! compare the cards themselves.

mod common;
use common::PdfDataPath;

use std::path::{Path, PathBuf};

use common::validation_dir;
use vibegraph::artifact::IntegrateArtifact;

#[path = "../../vibegraph-lib/tests/common/seeds.rs"]
mod seeds;

#[path = "../../vibegraph-lib/tests/common/seed_record.rs"]
mod seed_record;

#[path = "../../vibegraph-lib/tests/common/report.rs"]
mod report;

#[path = "../../vibegraph-lib/tests/common/manifest.rs"]
mod manifest;

use report::{IntegralsRow, SeedResult, Stopwatch};
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

/// The row's seed record, or `None` (after saying so) while the manifest
/// declares the row planned and this checkout has none.
fn reference() -> Option<seed_record::SeedReference> {
    if let Some(reference) = seed_record::reference(&output_dir(), ROW) {
        return Some(reference);
    }
    if manifest::planned_rows().contains(ROW) {
        return None;
    }
    vibegraph::validation::require(
        "validate_ewk_lljj_sigma",
        "the row's seed record",
        format!(
            "output/{}/{ROW}/{}",
            seed_record::RECORD_DIR,
            seed_record::RECORD_FILE
        ),
    )
}

/// `p p > e+ e- j j QCD=0` with no pT cut against MadEvent's seed record.
#[test]
#[ignore = "long tier: pixi run validate-ewk-lljj-sigma"]
fn ewk_epemjj_without_pt_cuts_matches_madevents_seeds() {
    let proc_card = proc_card_text();
    let reference = reference();
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
                artifact.chi2_per_dof,
                seed_clock.seconds()
            );
            SeedResult {
                seed,
                sigma_pb: artifact.sigma_pb,
                sigma_err_pb: artifact.sigma_err_pb,
            }
        })
        .collect();
    let SeedMean {
        mean,
        err,
        quoted,
        spread,
        chi2_per_dof: chi2,
        ..
    } = combine(&runs);
    let side = format!(
        "vibegraph σ = {mean:.7e} ± {err:.2e} pb ({} seeds, quoted {quoted:.2e}, spread {spread:.2e}, \
         χ²/dof = {chi2:.2}), {:.0} s",
        runs.len(),
        clock.seconds()
    );

    let Some(reference) = reference else {
        eprintln!(
            "[{ROW}] AWAITING THE BANK (the manifest marks row {ROW} planned and this checkout has \
             no seed record output/{}/{ROW}, so nothing is compared and no cell is written) | {side}",
            seed_record::RECORD_DIR
        );
        return;
    };

    let (mg, mg_err) = (reference.mean(), reference.err());
    let pull = (mean - mg) / (err * err + mg_err * mg_err).sqrt();
    let rel = mean / mg - 1.0;
    let chi2_ok = chi2 > CHI2_PER_DOF_BAND.0 && chi2 < CHI2_PER_DOF_BAND.1;
    let ok = pull.abs() < MAX_PULL && rel.abs() < MAX_REL && chi2_ok;
    eprintln!(
        "[{ROW}] GATE {side} | MG σ = {mg:.7e} ± {mg_err:.2e} pb | pull = {pull:+.2} | rel = {rel:+.5}"
    );

    let mut cell = IntegralsRow::new(ROW, "p p > e+ e- j j QCD=0", "gate");
    cell.status = if ok { "pass" } else { "fail" };
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
        pull.abs() < MAX_PULL && rel.abs() < MAX_REL,
        "[{ROW}] σ disagreement: vibegraph {mean:.7e} ± {err:.2e} against MadGraph \
         {mg:.7e} ± {mg_err:.2e} pb, pull {pull:+.2}, rel {rel:+.5}"
    );
    assert!(
        chi2_ok,
        "[{ROW}] the seeds' χ²/dof {chi2:.2} lies outside {CHI2_PER_DOF_BAND:?} over {runs:?}"
    );
}
