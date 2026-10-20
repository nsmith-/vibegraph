//! A run card that activates a cut the generator parses but does not apply is
//! refused at the CLI boundary by naming the cut and its value, before the
//! process is enumerated.
//!
//! "Before" is observed through a process whose enumeration cannot succeed
//! (QED order 0 for a purely electroweak final state): were enumeration reached
//! first, its own error would reach stderr instead of the cut's. The test shows
//! the order of the two failures; it cannot show how long the refused run spent
//! before failing.

use std::path::Path;
use std::process::{Command, Output};

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

const BEAMS: &str = "  0 = lpp1\n  0 = lpp2\n  45.6 = ebeam1\n  45.6 = ebeam2\n";

fn integrate(proc_line: &str, cuts: &str) -> Output {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let proc_card = cwd.path().join("proc_card.dat");
    std::fs::write(
        &proc_card,
        format!("import model sm\ngenerate {proc_line}\n"),
    )
    .unwrap();
    let run_card = cwd.path().join("run_card.dat");
    std::fs::write(&run_card, format!("{BEAMS}{cuts}")).unwrap();
    vibegraph(cwd.path(), home.path())
        .arg("--no-network")
        .arg("integrate")
        .arg(&proc_card)
        .arg("--run-card")
        .arg(&run_card)
        .arg("--out")
        .arg(cwd.path().join("out"))
        .args(["--fixed-budget", "--neval", "1000", "--niter", "2"])
        .output()
        .expect("spawn vibegraph")
}

#[test]
fn integrate_refuses_an_unimplemented_cut_naming_it_and_its_value() {
    let output = integrate("e+ e- > mu+ mu-", "  30 = ptj1min\n");
    assert!(!output.status.success());
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("'ptj1min' (= 30"),
        "the refusal must name the cut and its value, got:\n{stderr}"
    );
    assert!(
        !stderr.contains("flavour groups") && !stderr.contains("cut compilation failed"),
        "got:\n{stderr}"
    );
}

#[test]
fn integrate_refuses_before_enumeration() {
    let process = "e+ e- > mu+ mu- QED=0";
    let control = integrate(process, "");
    assert!(!control.status.success());
    let control_stderr = stderr_of(&control);
    assert!(
        control_stderr.contains("enumerate") || control_stderr.contains("diagram"),
        "the control card must fail in enumeration, got:\n{control_stderr}"
    );

    let output = integrate(process, "  30 = ptj1min\n");
    assert!(!output.status.success());
    let stderr = stderr_of(&output);
    assert!(stderr.contains("'ptj1min' (= 30"), "got:\n{stderr}");
    assert!(
        !stderr.contains("enumerate"),
        "enumeration ran before the refusal:\n{stderr}"
    );
}

#[test]
fn integrate_lets_an_unbindable_maximum_through_and_refuses_a_bindable_one() {
    // Collider energy 91.2 GeV: a transverse momentum cannot exceed 45.6.
    let bindable = integrate("e+ e- > mu+ mu-", "  30 = missetmax\n");
    assert!(!bindable.status.success());
    assert!(
        stderr_of(&bindable).contains("'missetmax' (= 30"),
        "got:\n{}",
        stderr_of(&bindable)
    );

    let unbindable = integrate("e+ e- > mu+ mu-", "  1e5 = missetmax\n");
    assert!(
        unbindable.status.success(),
        "got:\n{}",
        stderr_of(&unbindable)
    );
}

#[test]
fn generate_refuses_an_unimplemented_cut_naming_it_and_its_value() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let proc_card = cwd.path().join("proc_card.dat");
    std::fs::write(&proc_card, "import model sm\ngenerate e+ e- > mu+ mu-\n").unwrap();
    let clean_card = cwd.path().join("clean_run_card.dat");
    std::fs::write(&clean_card, BEAMS).unwrap();
    let out = cwd.path().join("out");
    let integrated = vibegraph(cwd.path(), home.path())
        .arg("--no-network")
        .arg("integrate")
        .arg(&proc_card)
        .arg("--run-card")
        .arg(&clean_card)
        .arg("--out")
        .arg(&out)
        .args(["--fixed-budget", "--neval", "1000", "--niter", "2"])
        .output()
        .expect("spawn vibegraph");
    assert!(
        integrated.status.success(),
        "got:\n{}",
        stderr_of(&integrated)
    );

    let cut_card = cwd.path().join("cut_run_card.dat");
    std::fs::write(&cut_card, format!("{BEAMS}  30 = ptj1min\n")).unwrap();
    let artifact = std::fs::read_dir(&out)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.is_file())
        .expect("an artifact");
    let output = vibegraph(cwd.path(), home.path())
        .arg("--no-network")
        .arg("generate")
        .arg(&artifact)
        .arg(&proc_card)
        .arg("--run-card")
        .arg(&cut_card)
        .args(["--nevents", "10", "-o"])
        .arg(cwd.path().join("events.lhe"))
        .output()
        .expect("spawn vibegraph");
    assert!(!output.status.success());
    let stderr = stderr_of(&output);
    assert!(stderr.contains("'ptj1min' (= 30"), "got:\n{stderr}");
    assert!(
        !stderr.contains("flavour groups") && !stderr.contains("cut compilation failed"),
        "got:\n{stderr}"
    );
}
