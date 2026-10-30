//! σ at a moved parameter point: `integrate --param-card` against a MadGraph run
//! made at the same card.
//!
//! Two rows, each a fresh MadGraph run at its card, banked as independent seeds
//! by `validation/madgraph/bank_seeded_sigma.sh`:
//!
//! * `pp_to_ll_mz95` — `p p > e+ e-` on the m_ll ∈ [60, 120] GeV card
//!   (`dy13_mmll_run_card.dat`) at `MZ = 95 GeV`
//!   (`dy13_mz95_param_card.dat`, MadGraph's own SM card with that one line
//!   changed); seed record `dy13_mmll_60_120_mz95`. The Z peak sits inside the
//!   window, so σ moves by about +16% against the default card.
//! * `ee_to_tth_ymt250` — `e+ e- > t t~ h` at √s = 1 TeV with `ymt = 250 GeV`
//!   and `MT` left at 173 GeV (`scripts/ee_to_tth_ymt250.mg5`); seed record
//!   `ee_to_tth_ymt250`, run card and param card those of the banked run. σ
//!   about doubles against the default.
//!
//! Each side reads five seeds under the seed policy: their unweighted mean with
//! `max(quoted, spread)` (`common/seeds.rs`), the seeds' χ²/dof bounded on its
//! own. Both rows gate as the Drell-Yan σ cells do: `|pull| < 3` and
//! `|rel| < 0.005` together, and the vibegraph seeds' χ²/dof below 4.
//!
//! Both rows are banked: a seed record or a card missing from this checkout is
//! an incomplete environment, and the gate fails naming it
//! ([`vibegraph::validation::require`]).
//!
//! # Beside the gate
//!
//! `ttbar_higgs_at_ymt_250_against_the_reweight_oracle_ratio` reports, never
//! enforces, vibegraph's σ(ymt = 250) / σ(default) on the reweight oracle's card
//! against MadGraph's per-event reweight ratio ⟨w₂₅₀ / XWGTUP⟩ over the
//! oracle's events (`reweight_mg_reference.json`). It runs whether or not the
//! `ee_to_tth_ymt250` row is banked.
//!
//! # What these gates cannot see
//!
//! A σ is blind to a mis-sampled region of small measure, which the seeds'
//! scatter and not the pull guards, and to any parameter the process does not
//! depend on. Each card moves one parameter that the process is strongly
//! sensitive to (`MZ` through the Z propagator and, via `MW`, the weak mixing;
//! `ymt` through the `t t~ h` vertex alone), so a run that bound the card
//! wrongly, or not at all, is far outside the band. MadGraph's restriction
//! fuses couplings that are equal at the restriction point
//! (`detect_identical_couplings`); `validation/madgraph/check_coupling_fusion.py`
//! checks, with MadGraph's own code, that no fused pair of the SM default
//! restriction splits at either card, so the two sides evaluate the same
//! couplings.

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

use report::{IntegralsRow, SeedResult, Stopwatch};
use seeds::{combine, SeedMean, SeedValue};

impl SeedValue for SeedResult {
    fn value_and_err(&self) -> (f64, f64) {
        (self.sigma_pb, self.sigma_err_pb)
    }
}

/// The integration budget per seed: `validate_hadronic`'s Drell-Yan σ gate's,
/// at which one Drell-Yan seed quotes about 0.1% and one `t t~ h` seed about
/// 0.05%.
const NEVAL: &str = "120000";
const NITER: &str = "12";
const SEEDS: [u64; 5] = [20261031, 20261032, 20261033, 20261034, 20261035];

const MAX_PULL: f64 = 3.0;
const MAX_REL: f64 = 0.005;
const MAX_CHI2_PER_DOF: f64 = 4.0;

fn output_dir() -> PathBuf {
    validation_dir().join("output")
}

fn pdf_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/pdf")
}

/// One moved-card row.
struct MovedRow {
    /// The manifest row.
    key: &'static str,
    /// The seed record's directory under `output/seeded_sigma/`.
    record: &'static str,
    process: &'static str,
    /// The parameter the card moves, and its value there.
    moved: (&'static str, f64),
}

const DRELL_YAN_MZ95: MovedRow = MovedRow {
    key: "pp_to_ll_mz95",
    record: "dy13_mmll_60_120_mz95",
    process: "p p > e+ e-",
    moved: ("MZ", 95.0),
};

const TTH_YMT250: MovedRow = MovedRow {
    key: "ee_to_tth_ymt250",
    record: "ee_to_tth_ymt250",
    process: "e+ e- > t t~ h",
    moved: ("ymt", 250.0),
};

/// `integrate` `process` on `run_card` (and `param_card`, where given) at
/// `seed`, at [`NEVAL`] × [`NITER`], and read its artifact.
fn integrate(
    process: &str,
    run_card: &Path,
    param_card: Option<&Path>,
    seed: u64,
) -> IntegrateArtifact {
    let tmp = tempfile::tempdir().expect("temporary directory");
    let proc_card = tmp.path().join("proc_card.dat");
    std::fs::write(&proc_card, format!("import model sm\ngenerate {process}\n"))
        .expect("write the proc card");
    let out = tmp.path().join("out");
    let mut cmd = common::vibegraph_ambient();
    cmd.arg("integrate")
        .arg(&proc_card)
        .arg("--run-card")
        .arg(run_card)
        .arg("--out")
        .arg(&out)
        .pdf_data_path(pdf_dir())
        .args(["--fixed-budget", "--neval", NEVAL, "--niter", NITER])
        .args(["--seed", &seed.to_string()]);
    if let Some(card) = param_card {
        cmd.arg("--param-card").arg(card);
    }
    let output = cmd.output().expect("spawn vibegraph");
    assert!(
        output.status.success(),
        "vibegraph integrate failed:\n{}",
        common::stderr_of(&output)
    );
    IntegrateArtifact::read_from_path(&out.join("grid.bin.zst")).expect("read the artifact")
}

/// [`SEEDS`]' `(σ, Δσ)`, each run checked to have recorded `moved` among the
/// parameters its card moved.
fn seeds(
    process: &str,
    run_card: &Path,
    param_card: Option<&Path>,
    moved: Option<(&str, f64)>,
) -> Vec<SeedResult> {
    SEEDS
        .iter()
        .map(|&seed| {
            let artifact = integrate(process, run_card, param_card, seed);
            match moved {
                Some((name, value)) => assert!(
                    artifact
                        .inputs
                        .params
                        .iter()
                        .any(|p| p.name == name && p.value == value),
                    "seed {seed}: the artifact records {:?}, not {name} = {value}",
                    artifact.inputs.params
                ),
                None => assert!(
                    artifact.inputs.params.is_empty(),
                    "seed {seed}: a card-less run records {:?}",
                    artifact.inputs.params
                ),
            }
            eprintln!(
                "  seed {seed}: σ = {:.7e} ± {:.2e} pb (χ²/dof {:.2})",
                artifact.sigma_pb, artifact.sigma_err_pb, artifact.chi2_per_dof
            );
            SeedResult {
                seed,
                sigma_pb: artifact.sigma_pb,
                sigma_err_pb: artifact.sigma_err_pb,
            }
        })
        .collect()
}

/// The row's MadGraph reference, its banked seed record.
fn reference(row: &MovedRow) -> seed_record::SeedReference {
    seed_record::reference(&output_dir(), row.record).unwrap_or_else(|| {
        vibegraph::validation::require(
            "validate_param_card_sigma",
            "the row's seed record",
            format!(
                "output/{}/{}/{}",
                seed_record::RECORD_DIR,
                row.record,
                seed_record::RECORD_FILE
            ),
        )
    })
}

/// Integrate the row at its card, compare with its seed record, write the
/// `integrals` cell, and gate.
fn check(
    row: &MovedRow,
    reference: &seed_record::SeedReference,
    run_card: &Path,
    param_card: &Path,
) {
    let clock = Stopwatch::start();
    eprintln!("[{}] {}", row.key, reference.describe(row.record));
    let runs = seeds(row.process, run_card, Some(param_card), Some(row.moved));
    let SeedMean {
        mean,
        err,
        chi2_per_dof: chi2,
        ..
    } = combine(&runs);
    let (mg, mg_err) = (reference.mean(), reference.err());
    let pull = (mean - mg) / (err * err + mg_err * mg_err).sqrt();
    let rel = mean / mg - 1.0;
    let ok = pull.abs() < MAX_PULL && rel.abs() < MAX_REL && chi2 < MAX_CHI2_PER_DOF;
    eprintln!(
        "[{}] GATE vibegraph σ = {mean:.7e} ± {err:.2e} pb ({} seeds, χ²/dof = {chi2:.2}) | \
         MG σ = {mg:.7e} ± {mg_err:.2e} pb | pull = {pull:+.2} | rel = {rel:+.5}",
        row.key,
        runs.len()
    );

    let mut cell = IntegralsRow::new(row.key, row.process, "gate");
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
    cell.note = Some(format!(
        "integrate --param-card {} ({} = {}); {}",
        param_card.display(),
        row.moved.0,
        row.moved.1,
        reference.describe(row.record)
    ));
    cell.duration_s = Some(clock.seconds());
    cell.write();

    assert!(
        pull.abs() < MAX_PULL && rel.abs() < MAX_REL,
        "[{}] σ disagreement: vibegraph {mean:.7e} ± {err:.2e} against MadGraph \
         {mg:.7e} ± {mg_err:.2e} pb, pull {pull:+.2}, rel {rel:+.5}",
        row.key
    );
    assert!(
        chi2 < MAX_CHI2_PER_DOF,
        "[{}] the seeds scatter by more than they claim: χ²/dof = {chi2:.2} over {runs:?}",
        row.key
    );
}

/// `p p > e+ e-` in the m_ll window at `MZ = 95 GeV` against MadGraph's run at
/// the same card. The card is the committed one the bank installs, so the run
/// card and param card are the same files on both sides.
#[test]
fn drell_yan_at_mz_95_matches_madgraphs_run_at_the_card() {
    let row = DRELL_YAN_MZ95;
    let reference = reference(&row);
    check(
        &row,
        &reference,
        &validation_dir().join("dy13_mmll_run_card.dat"),
        &validation_dir().join("dy13_mz95_param_card.dat"),
    );
}

/// `e+ e- > t t~ h` at `ymt = 250 GeV` against MadGraph's run at the same
/// card: the banked run's own `Cards/run_card.dat` and `Cards/param_card.dat`.
#[test]
fn ttbar_higgs_at_ymt_250_matches_madgraphs_run_at_the_card() {
    let row = TTH_YMT250;
    let reference = reference(&row);
    let cards = output_dir().join(row.record).join("Cards");
    let run_card = cards.join("run_card.dat");
    let param_card = cards.join("param_card.dat");
    for card in [&run_card, &param_card] {
        if !card.is_file() {
            vibegraph::validation::require(
                "validate_param_card_sigma",
                "the banked run's cards",
                card.display().to_string(),
            );
        }
    }
    check(&row, &reference, &run_card, &param_card);
}

/// MadGraph's reweight ratio ⟨w₂₅₀ / XWGTUP⟩ over the reweight oracle's
/// `ee_tth_ymt` events, with its standard error.
fn reweight_oracle_ratio() -> (f64, f64, usize) {
    let path = validation_dir().join("reweight_mg_reference.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let reference: serde_json::Value = serde_json::from_str(&text).expect("the oracle parses");
    let events = reference["rows"]["ee_tth_ymt"]["events"]
        .as_array()
        .expect("the oracle's ee_tth_ymt row has events");
    let ratios: Vec<f64> = events
        .iter()
        .map(|e| {
            let w = e["madgraph"]["ymt250"].as_f64().expect("a ymt250 weight");
            let w0 = e["xwgtup"].as_f64().expect("an XWGTUP");
            w / w0
        })
        .collect();
    let n = ratios.len() as f64;
    let mean = ratios.iter().sum::<f64>() / n;
    let variance = ratios.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
    (mean, (variance / n).sqrt(), ratios.len())
}

/// vibegraph's σ(ymt = 250) / σ(default) on the reweight oracle's card,
/// against MadGraph's per-event reweight ratio: **reported, not enforced**.
///
/// The oracle's events were integrated at a fixed budget and its own σ₀ sits
/// below a converged one, so its absolute reweighted σ is not a reference; the
/// ratio is. It is a different estimator from the gate's (one MadGraph sample
/// reweighted, against a fresh run at the card), so the two meet on one point
/// from two directions. The known-wrong comparison, σ(250) against σ(default),
/// is printed beside it.
#[test]
fn ttbar_higgs_at_ymt_250_against_the_reweight_oracle_ratio() {
    let run_card = validation_dir().join("onshell_ee_run_card.dat");
    let tmp = tempfile::tempdir().expect("temporary directory");
    let card = tmp.path().join("ymt250_param_card.dat");
    std::fs::write(&card, "Block YUKAWA\n    6 2.500000e+02 # ymt\n").expect("write the card");

    eprintln!("[ee_to_tth_ymt250 / reweight oracle] default card:");
    let default = combine(&seeds(TTH_YMT250.process, &run_card, None, None));
    eprintln!("[ee_to_tth_ymt250 / reweight oracle] ymt = 250:");
    let moved = combine(&seeds(
        TTH_YMT250.process,
        &run_card,
        Some(&card),
        Some(TTH_YMT250.moved),
    ));
    let ratio = moved.mean / default.mean;
    let ratio_err =
        ratio * ((moved.err / moved.mean).powi(2) + (default.err / default.mean).powi(2)).sqrt();
    let (oracle, oracle_err, events) = reweight_oracle_ratio();
    let pull = (ratio - oracle) / (ratio_err * ratio_err + oracle_err * oracle_err).sqrt();
    eprintln!(
        "[ee_to_tth_ymt250] INFO σ(default) = {:.7e} ± {:.2e} pb (χ²/dof {:.2}), \
         σ(ymt=250) = {:.7e} ± {:.2e} pb (χ²/dof {:.2}), {} seeds each",
        default.mean,
        default.err,
        default.chi2_per_dof,
        moved.mean,
        moved.err,
        moved.chi2_per_dof,
        SEEDS.len()
    );
    eprintln!(
        "[ee_to_tth_ymt250] INFO vibegraph σ(250)/σ(default) = {ratio:.5} ± {ratio_err:.5} | \
         MadGraph reweight ⟨w250/XWGTUP⟩ = {oracle:.5} ± {oracle_err:.5} over {events} events | \
         pull = {pull:+.2}"
    );
    eprintln!(
        "[ee_to_tth_ymt250] INFO known-wrong: σ(250) against σ(default), rel = {:+.4}",
        ratio - 1.0
    );
}
