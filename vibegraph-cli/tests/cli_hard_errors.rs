//! Three card surfaces that are out of the v0.1 restricted scope must refuse
//! at the CLI boundary with the same message the underlying parser raises,
//! not just from a unit test that never goes through `main`.

use std::path::Path;
use std::process::{Command, Output};

/// The binary, with `VIBEGRAPH_HOME` pointed at a temp dir so `cache_root()`
/// never touches the real `$HOME`.
fn vibegraph(cwd: &Path, home: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_vibegraph"));
    cmd.current_dir(cwd)
        .env("VIBEGRAPH_HOME", home)
        .env_remove("VIBEGRAPH_NO_NETWORK")
        .env_remove("VIBEGRAPH_PDF_DIR")
        .env_remove("VIBEGRAPH_UFO_DIR");
    cmd
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn cli_polarized_beam_card_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();

    let proc_card = cwd.path().join("proc_card.dat");
    std::fs::write(&proc_card, "import model sm\ngenerate e+ e- > mu+ mu-\n").unwrap();

    let run_card = cwd.path().join("run_card.dat");
    std::fs::write(
        &run_card,
        "  0 = lpp1\n  0 = lpp2\n  45.6 = ebeam1\n  45.6 = ebeam2\n  1.0 = polbeam1\n",
    )
    .unwrap();

    let output = vibegraph(cwd.path(), home.path())
        .arg("--no-network")
        .arg("integrate")
        .arg(&proc_card)
        .arg("--run-card")
        .arg(&run_card)
        .arg("--out")
        .arg(out.path())
        .args(["--fixed-budget", "--neval", "1000", "--niter", "2"])
        .output()
        .expect("spawn vibegraph");

    assert!(
        !output.status.success(),
        "a polarized beam card must be refused"
    );
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("polbeam1") && stderr.contains("beam polarisation"),
        "got:\n{stderr}"
    );
}

#[test]
fn cli_decay_chain_proc_card_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();

    let proc_card = cwd.path().join("proc_card.dat");
    std::fs::write(
        &proc_card,
        "import model sm\ngenerate p p > t t~, t > w+ b\n",
    )
    .unwrap();

    let output = vibegraph(cwd.path(), home.path())
        .arg("--no-network")
        .arg("integrate")
        .arg(&proc_card)
        .arg("--out")
        .arg(out.path())
        .args(["--fixed-budget", "--neval", "1000", "--niter", "2"])
        .output()
        .expect("spawn vibegraph");

    assert!(
        !output.status.success(),
        "a decay-chain proc card must be refused"
    );
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("decay-chain process syntax is not supported"),
        "got:\n{stderr}"
    );
}

#[test]
fn cli_propagators_py_model_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    let ufo_root = tempfile::tempdir().unwrap();

    let model_dir = ufo_root.path().join("fixturemodel");
    std::fs::create_dir_all(&model_dir).unwrap();
    for name in [
        "particles.py",
        "lorentz.py",
        "couplings.py",
        "parameters.py",
        "vertices.py",
        "propagators.py",
    ] {
        std::fs::write(model_dir.join(name), "").unwrap();
    }

    let proc_card = cwd.path().join("proc_card.dat");
    std::fs::write(
        &proc_card,
        "import model fixturemodel\ngenerate e+ e- > mu+ mu-\n",
    )
    .unwrap();

    let output = vibegraph(cwd.path(), home.path())
        .arg("--no-network")
        .arg("integrate")
        .arg(&proc_card)
        .arg("--ufo-dir")
        .arg(ufo_root.path())
        .arg("--out")
        .arg(out.path())
        .args(["--fixed-budget", "--neval", "1000", "--niter", "2"])
        .output()
        .expect("spawn vibegraph");

    assert!(
        !output.status.success(),
        "a model directory carrying propagators.py must be refused"
    );
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("custom UFO propagators are not supported"),
        "got:\n{stderr}"
    );
}
