//! `integrate` refuses a cross section its points do not carry, and `generate`
//! refuses an artifact recorded at `σ = 0`.
//!
//! Three ways a run's points can fail to carry σ, one card each:
//!
//! * the cuts reject every point: `e+ e- > mu+ mu-` at `√s = 91.2 GeV`, where no
//!   muon can reach the `ptl = 50 GeV` the run card asks for;
//! * every point passes the cuts and `|M|²` is zero on all of them:
//!   `lt~ lt > vt vt` in `vibegraph_toy_UFO`, whose only coupling `gl` the param
//!   card sets to zero;
//! * a handful of points carry the weight: the same `e+ e- > mu+ mu-` with `ptl`
//!   a hair under the beam energy, which only muons within `2·10⁻⁴` of
//!   `cos θ = 0` pass, on a seed whose grid never finds that window and on one
//!   whose grid finds it late.
//!
//! Hermetic: the built-in SM and the committed toy model, fixed-energy beams.
//!
//! # What these tests cannot see
//!
//! They pin the refusal and the cause it names on cards whose cause is known by
//! construction. They cannot see whether the census counts the right points on a
//! proton-beam run (the hadronic integrand's count is pinned by its own library
//! tests), nor whether the nonzero-point threshold sits where it should: that is
//! a measurement against the banked rows, not something a test of one card
//! decides. The two too-few cases are pinned seeds of a chaotic adaptation:
//! another platform's last-ulp arithmetic can move them, though the margins
//! (an effective 1 and 5 points against a floor of 20) are wide.

mod common;

use std::path::{Path, PathBuf};
use std::process::Output;

use common::fixed_beams;
use vibegraph::artifact::IntegrateArtifact;

const SM_PROCESS: &str = "import model sm\ngenerate e+ e- > mu+ mu-\n";
const EBEAM: f64 = 45.6;

fn toy_ufo_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../validation/ufo")
}

/// `integrate` the card in `dir`, with `extra` flags, at a small fixed budget.
fn integrate(dir: &Path, proc_card: &str, run_card: &str, extra: &[&str]) -> Output {
    let seeded = extra.contains(&"--seed");
    let proc_path = dir.join("proc_card.dat");
    let run_path = dir.join("run_card.dat");
    std::fs::write(&proc_path, proc_card).unwrap();
    std::fs::write(&run_path, run_card).unwrap();
    common::vibegraph_ambient()
        .arg("integrate")
        .arg(&proc_path)
        .arg("--run-card")
        .arg(&run_path)
        .arg("--out")
        .arg(dir.join("out"))
        .args(["--fixed-budget", "--neval", "4000", "--niter", "4"])
        .args(if seeded {
            &[][..]
        } else {
            &["--seed", "20261104"][..]
        })
        .args(extra)
        .output()
        .expect("spawn vibegraph")
}

/// The run was refused, said every one of `needles`, and left no artifact.
fn assert_refused(dir: &Path, output: &Output, needles: &[&str]) {
    let stderr = common::stderr_of(output);
    assert!(
        !output.status.success(),
        "the run must be refused; it succeeded:\n{stderr}"
    );
    assert!(!stderr.contains("panicked"), "panicked:\n{stderr}");
    for needle in needles {
        assert!(
            stderr.contains(needle),
            "the refusal does not say {needle:?}:\n{stderr}"
        );
    }
    assert!(
        !dir.join("out").join("grid.bin.zst").exists(),
        "a refused run left an artifact"
    );
    assert!(
        !common::stdout_of(output).contains("σ ="),
        "a refused run printed a cross section"
    );
}

#[test]
fn cuts_that_reject_every_point_are_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let run_card = format!("{}  50.0 = ptl\n", fixed_beams(EBEAM, EBEAM));
    let output = integrate(tmp.path(), SM_PROCESS, &run_card, &[]);
    assert_refused(
        tmp.path(),
        &output,
        &[
            "has zero weight",
            "none passed the cuts",
            "no artifact was written",
        ],
    );
}

#[test]
fn a_coupling_zero_at_the_parameters_in_force_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let param_card = tmp.path().join("param_card.dat");
    std::fs::write(&param_card, "Block TOYCOUP\n    1 0.0 # gl\n").unwrap();
    let output = integrate(
        tmp.path(),
        "import model vibegraph_toy_UFO\ngenerate lt~ lt > vt vt\n",
        &fixed_beams(500.0, 500.0),
        &[
            "--ufo-dir",
            toy_ufo_dir().to_str().unwrap(),
            "--param-card",
            param_card.to_str().unwrap(),
        ],
    );
    assert_refused(
        tmp.path(),
        &output,
        &[
            "has zero weight",
            "passed the cuts",
            "|M|²",
            "zero at the parameters in force",
            "no artifact was written",
        ],
    );
    assert!(
        !common::stderr_of(&output).contains("none passed the cuts"),
        "every point passed the cuts here"
    );
}

/// The control for the zero-coupling card: the same process at the model's own
/// `gl` integrates.
#[test]
fn the_same_toy_process_at_its_default_coupling_integrates() {
    let tmp = tempfile::tempdir().unwrap();
    let output = integrate(
        tmp.path(),
        "import model vibegraph_toy_UFO\ngenerate lt~ lt > vt vt\n",
        &fixed_beams(500.0, 500.0),
        &["--ufo-dir", toy_ufo_dir().to_str().unwrap()],
    );
    assert!(output.status.success(), "{}", common::stderr_of(&output));
}

/// `ptl` this close under the beam energy passes only muons within
/// `√(2(1 − ptl/E)) ≈ 2·10⁻⁴` of `cos θ = 0`.
const TIGHT_PTL: &str = "45.599999";

/// The tight card at `seed`, refused as carried by too few effective points,
/// with `nonzero` points of weight among those σ averages.
fn assert_too_few(seed: &str, nonzero: &str) {
    let tmp = tempfile::tempdir().unwrap();
    let run_card = format!("{}  {TIGHT_PTL} = ptl\n", fixed_beams(EBEAM, EBEAM));
    let output = integrate(tmp.path(), SM_PROCESS, &run_card, &["--seed", seed]);
    assert_refused(
        tmp.path(),
        &output,
        &[
            "the cross section is carried by an effective",
            "below the 20 it is accepted on",
            &format!(": {nonzero} of the 8192 points it is averaged over have nonzero weight"),
            "no artifact was written",
        ],
    );
}

/// The grid never finds the window: one point of the kept iterations lands in
/// it, the shape of a σ quoted as `x ± x`.
#[test]
fn a_cross_section_carried_by_one_point_is_refused() {
    assert_too_few("20261104", "1");
}

/// The grid finds the window late: hundreds of kept points have weight, and a
/// handful carry nearly all of it. A count of nonzero points passes this run;
/// the effective sample size refuses it.
#[test]
fn a_cross_section_carried_by_a_few_of_many_nonzero_points_is_refused() {
    assert_too_few("5", "590");
}

#[test]
fn generate_refuses_an_artifact_recorded_at_zero_cross_section() {
    let tmp = tempfile::tempdir().unwrap();
    let output = integrate(tmp.path(), SM_PROCESS, &fixed_beams(EBEAM, EBEAM), &[]);
    assert!(output.status.success(), "{}", common::stderr_of(&output));
    let path = tmp.path().join("out").join("grid.bin.zst");
    let mut artifact = IntegrateArtifact::read_from_path(&path).expect("reload artifact");
    artifact.sigma_pb = 0.0;
    artifact.sigma_err_pb = 0.0;
    for c in &mut artifact.channels {
        c.sigma_pb = 0.0;
        c.sigma_err_pb = 0.0;
    }
    artifact
        .write_to_path(&path, true)
        .expect("rewrite artifact");

    let events = tmp.path().join("events.lhe");
    let output = common::vibegraph_ambient()
        .arg("generate")
        .arg(&path)
        .args(["--seed", "7", "--nevents", "100"])
        .arg("-o")
        .arg(&events)
        .output()
        .expect("spawn vibegraph");
    let stderr = common::stderr_of(&output);
    assert!(
        !output.status.success(),
        "generate must refuse a σ = 0 artifact:\n{stderr}"
    );
    assert!(!stderr.contains("panicked"), "panicked:\n{stderr}");
    assert!(stderr.contains("records σ = 0"), "{stderr}");
    assert!(!events.exists(), "a refused generate wrote events");
}
