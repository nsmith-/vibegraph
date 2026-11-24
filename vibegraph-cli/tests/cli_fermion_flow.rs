//! Majorana fermions and fermion-number-violating vertices integrate through
//! `vibegraph integrate`.
//!
//! The two models are committed under `validation/ufo/`:
//! `vibegraph_toy_majorana_UFO` (a Majorana `nt` beside a Dirac `lt`) and
//! `vibegraph_toy_fnv_UFO` (a diquark-like scalar `dq` coupling the Dirac fermions
//! `at` and `bt` with no charge-conjugation operator). Their amplitudes are compared
//! with MadGraph's per diagram in `vibegraph-lib/tests/standalone_jamps.rs`; these
//! tests hold the command-line path.
//!
//! Hermetic: the models are committed to this repository and no PDF set is
//! needed (`lpp = 0`).

use std::path::{Path, PathBuf};
use std::process::Output;

fn ufo_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../validation/ufo")
}

/// `vibegraph integrate` on `generate <process>` in `model`, at a smoke-run budget.
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

fn assert_integrates(model: &str, process: &str) {
    let tmp = tempfile::tempdir().unwrap();
    let output = integrate(tmp.path(), model, process);
    assert!(
        output.status.success(),
        "'{process}' in {model} must integrate:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Two Majorana fermions in the final state, reached through the s-channel vector
/// on the axial `nt nt vt` vertex and the t- and u-channel scalar.
#[test]
fn external_majorana_fermions_integrate() {
    assert_integrates("vibegraph_toy_majorana_UFO", "lt~ lt > nt nt");
}

/// Every external leg is a Dirac lepton or a scalar; the Majorana fermion is the
/// t-channel propagator only.
#[test]
fn a_majorana_propagator_integrates() {
    assert_integrates("vibegraph_toy_majorana_UFO", "lt~ lt > sc sc~");
}

/// Two incoming particles annihilate into the diquark-like scalar, whose vertex
/// pairs them on one fermion line, and its conjugate pairs two antiparticles.
#[test]
fn fermion_number_violating_vertices_integrate() {
    assert_integrates("vibegraph_toy_fnv_UFO", "at bt > at bt");
    assert_integrates("vibegraph_toy_fnv_UFO", "at~ bt~ > at~ bt~");
}

/// A Dirac-only process of each model, whose diagrams reach neither the Majorana
/// fermion nor the diquark vertex.
#[test]
fn dirac_only_processes_of_the_same_models_integrate() {
    assert_integrates("vibegraph_toy_majorana_UFO", "lt~ lt > lt~ lt");
    assert_integrates("vibegraph_toy_fnv_UFO", "at~ at > at~ at");
}
