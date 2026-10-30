//! The systematics refusals the command line reaches without a PDF set: the
//! ones at integrate, where the run card's request is read, and the ones at
//! generate that a `--systematics` override meets before any density is read.

mod common;

use std::path::{Path, PathBuf};

use common::fixed_beams;

fn write(dir: &Path, name: &str, text: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    path
}

/// `integrate` on `process` with `run_card`, returning its output.
fn integrate(dir: &Path, process: &str, run_card: Option<&str>) -> std::process::Output {
    let proc_card = write(
        dir,
        "proc_card.dat",
        &format!("import model sm\ngenerate {process}\n"),
    );
    let mut cmd = common::vibegraph_behind_dead_proxy(dir, dir);
    cmd.arg("integrate")
        .arg(&proc_card)
        .arg("--out")
        .arg(dir.join("out"))
        .args(["--fixed-budget", "--neval", "2000", "--niter", "2"]);
    if let Some(text) = run_card {
        cmd.arg("--run-card").arg(write(dir, "run_card.dat", text));
    }
    cmd.output().expect("spawn vibegraph")
}

fn generate_with(dir: &Path, arguments: &str) -> std::process::Output {
    common::vibegraph_behind_dead_proxy(dir, dir)
        .arg("generate")
        .arg(dir.join("out/grid.bin.zst"))
        .args(["--seed", "1", "--nevents", "10"])
        .arg(format!("--systematics={arguments}"))
        .arg("-o")
        .arg(dir.join("events.lhe"))
        .output()
        .expect("spawn vibegraph")
}

/// Refusal table: `use_syst = T` written on fixed-energy beams is refused at
/// integrate, before anything is integrated.
#[test]
fn use_syst_written_on_fixed_energy_beams_is_refused_at_integrate() {
    let tmp = tempfile::tempdir().unwrap();
    let card = format!("{}  True = use_syst\n", fixed_beams(45, 45));
    let output = integrate(tmp.path(), "e+ e- > mu+ mu-", Some(&card));
    let stderr = common::stderr_of(&output);
    assert!(!output.status.success());
    assert!(
        stderr.contains("use_syst = 'True'") && stderr.contains("fixed-energy beams"),
        "{stderr}"
    );
    assert!(!tmp.path().join("out/grid.bin.zst").exists());
}

/// Refusal table: a request the card's program refuses fails at integrate
/// on proton beams too, before the PDF set is read.
#[test]
fn a_refused_program_fails_integrate_before_the_pdf_is_read() {
    let tmp = tempfile::tempdir().unwrap();
    let output = integrate(
        tmp.path(),
        "u u~ > e+ e-",
        Some("  syscalc = systematics_program\n"),
    );
    let stderr = common::stderr_of(&output);
    assert!(!output.status.success());
    assert!(stderr.contains("SysCalc is MadGraph's retired"), "{stderr}");
    assert!(!stderr.contains("PDF set"), "{stderr}");
}

/// Refusal table: `generate --systematics` on events integrated without
/// `use_syst` (which fixed-energy beams resolve off) is refused, as
/// MadGraph's module refuses them.
#[test]
fn an_override_without_use_syst_is_refused_at_generate() {
    let tmp = tempfile::tempdir().unwrap();
    let output = integrate(tmp.path(), "e+ e- > mu+ mu-", Some(&fixed_beams(45, 45)));
    assert!(output.status.success(), "{}", common::stderr_of(&output));
    let output = generate_with(tmp.path(), "--mur=0.5,1,2");
    let stderr = common::stderr_of(&output);
    assert!(!output.status.success());
    assert!(
        stderr.contains("--systematics") && stderr.contains("use_syst"),
        "{stderr}"
    );
    assert!(!tmp.path().join("events.lhe").exists());
}

/// Refusal table: `generate --systematics` on a decay is refused, as
/// MadGraph refuses an explicit request on one.
#[test]
fn an_override_on_a_decay_is_refused_at_generate() {
    let tmp = tempfile::tempdir().unwrap();
    let output = integrate(tmp.path(), "z > e+ e-", None);
    assert!(output.status.success(), "{}", common::stderr_of(&output));
    let output = generate_with(tmp.path(), "--pdf=errorset");
    let stderr = common::stderr_of(&output);
    assert!(!output.status.success());
    assert!(stderr.contains("--systematics on a decay"), "{stderr}");
}
