//! Majorana fermions and fermion-number-violating vertices are refused through
//! `vibegraph integrate`, with an error naming the particle or vertex, before
//! any amplitude is evaluated.
//!
//! The two models are refusal fixtures committed under `validation/ufo/`:
//! `vibegraph_toy_majorana_UFO` (a Majorana `nt` beside a Dirac `lt`) and
//! `vibegraph_toy_fnv_UFO` (a diquark-like scalar `dq` coupling the Dirac
//! fermions `at` and `bt` with no charge-conjugation operator). Each model also
//! has a Dirac-only process, which must still integrate: the refusal is a
//! property of a process's diagrams, not of the model it is drawn from.
//!
//! Hermetic: the models are committed to this repository and no PDF set is
//! needed (`lpp = 0`).

use std::path::{Path, PathBuf};
use std::process::Output;

fn ufo_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../validation/ufo")
}

/// `vibegraph integrate` on `generate <process>` in `model`, at a budget that is
/// a smoke run when the process is not refused.
fn integrate(tmp: &Path, model: &str, process: &str) -> Output {
    let proc_path = tmp.join("proc_card.dat");
    std::fs::write(
        &proc_path,
        format!("import model {model}\ngenerate {process}\n"),
    )
    .unwrap();
    let run_path = tmp.join("run_card.dat");
    std::fs::write(
        &run_path,
        "  0 = lpp1\n  0 = lpp2\n  250.0 = ebeam1\n  250.0 = ebeam2\n",
    )
    .unwrap();
    std::process::Command::new(env!("CARGO_BIN_EXE_vibegraph"))
        .env("VIBEGRAPH_HOME", tmp.join("home"))
        .env_remove("VIBEGRAPH_UFO_DIR")
        .arg("--no-network")
        .arg("integrate")
        .arg(&proc_path)
        .args(["--ufo-dir".as_ref(), ufo_dir().as_os_str()])
        .arg("--run-card")
        .arg(&run_path)
        .arg("--out")
        .arg(tmp.join("out"))
        .arg("--fixed-budget")
        .args(["--neval", "2000", "--niter", "2"])
        .output()
        .expect("spawn vibegraph")
}

/// The process is refused, cleanly, with every one of `needles` in the error.
fn assert_refused(model: &str, process: &str, needles: &[&str]) {
    let tmp = tempfile::tempdir().unwrap();
    let output = integrate(tmp.path(), model, process);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "'{process}' in {model} must be refused"
    );
    assert!(
        !stderr.contains("panicked"),
        "'{process}' in {model} panicked instead of refusing:\n{stderr}"
    );
    for needle in needles {
        assert!(
            stderr.contains(needle),
            "'{process}' in {model}: the refusal does not say {needle:?}:\n{stderr}"
        );
    }
    assert!(
        !tmp.path().join("out").join("grid.bin.zst").exists(),
        "a refused process left an artifact"
    );
}

fn assert_integrates(model: &str, process: &str) {
    let tmp = tempfile::tempdir().unwrap();
    let output = integrate(tmp.path(), model, process);
    assert!(
        output.status.success(),
        "'{process}' in {model} must integrate:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Two Majorana fermions in the final state, reached through an s-channel vector
/// and a t- and u-channel scalar: every diagram carries `nt` on two external legs.
#[test]
fn a_majorana_fermion_on_an_external_leg_is_refused() {
    assert_refused(
        "vibegraph_toy_majorana_UFO",
        "lt~ lt > nt nt",
        &[
            "particle 'nt' is a Majorana fermion",
            "an external leg",
            "charge conjugation",
        ],
    );
}

/// Every external leg is a Dirac lepton or a scalar; the Majorana fermion is the
/// t-channel propagator only.
#[test]
fn a_majorana_fermion_on_a_propagator_only_is_refused() {
    assert_refused(
        "vibegraph_toy_majorana_UFO",
        "lt~ lt > sc sc~",
        &[
            "particle 'nt' is a Majorana fermion",
            "an internal propagator",
        ],
    );
}

/// Two incoming particles annihilate into the diquark-like scalar, whose vertex
/// pairs them on one fermion line.
#[test]
fn a_fermion_number_violating_vertex_is_refused() {
    assert_refused(
        "vibegraph_toy_fnv_UFO",
        "at bt > at bt",
        &[
            "vertex 'V_atbtdq",
            "on one fermion line as two incoming",
            "fermion-number-violating",
        ],
    );
}

/// The control for both refusals: a Dirac-only process of each model, whose
/// diagrams reach neither the Majorana fermion nor the diquark vertex.
#[test]
fn dirac_only_processes_of_the_same_models_integrate() {
    assert_integrates("vibegraph_toy_majorana_UFO", "lt~ lt > lt~ lt");
    assert_integrates("vibegraph_toy_fnv_UFO", "at~ at > at~ at");
}
