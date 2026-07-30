//! What a user meets the first time they run a binary that has none of the data
//! it needs, and what the acceptance path checks once it does.
//!
//! # These tests must never download anything
//!
//! Some of them deliberately exercise the *default* policy — no
//! `$VIBEGRAPH_NO_NETWORK`, no `--yes` — because "an unattended run refuses"
//! is the property under test, and setting the kill switch would test the kill
//! switch instead. Two things keep that honest:
//!
//! * the assertion itself: a refusal is a non-zero exit with a specific message,
//!   so a regression that downloaded the set would fail the test rather than
//!   pass it quietly;
//! * `$ALL_PROXY` pointed at a closed port, which `ureq` reads from the
//!   environment. If the refusal ever stops working, the fetch underneath it
//!   fails in milliseconds against `127.0.0.1` instead of pulling 27 MB from
//!   CERN.
//!
//! The working directory is an empty temporary one throughout, so the repo's own
//! `validation/pdf` cannot resolve — what a user running an installed binary
//! sees, and the only way to tell a working cache path from a dev fallback
//! masking a broken one.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use vibegraph::cache::pinned::DEFAULT_PDF_SET;
use vibegraph::lhef::parse::LheFile;
use vibegraph::lhef::write::LheWriter;

/// A proxy nothing listens on: the belt under the braces described above.
const DEAD_PROXY: &str = "http://127.0.0.1:1";

fn validation_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph")
}

/// The binary, with every route to data the test did not set up removed.
fn vibegraph(cwd: &Path, cache_root: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_vibegraph"));
    cmd.current_dir(cwd)
        .env("VIBEGRAPH_HOME", cache_root)
        .env("ALL_PROXY", DEAD_PROXY)
        .env_remove("NO_PROXY")
        .env_remove("no_proxy")
        .env_remove("VIBEGRAPH_NO_NETWORK")
        .env_remove("VIBEGRAPH_PDF_DIR")
        .env_remove("VIBEGRAPH_UFO_DIR");
    cmd
}

/// A hadronic run, which needs the PDF set that is not there.
fn integrate_drell_yan(cwd: &Path, cache_root: &Path, out: &Path, extra: &[&str]) -> Output {
    vibegraph(cwd, cache_root)
        .arg("integrate")
        .arg(validation_dir().join("dy13_proc_card.dat"))
        .arg("--run-card")
        .arg(validation_dir().join("dy13_default_run_card.dat"))
        .arg("--out")
        .arg(out)
        .args(["--neval", "2000", "--niter", "2"])
        .args(extra)
        .output()
        .expect("spawn vibegraph")
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// The CI-safety property, end to end: a process with no terminal, under the
/// *default* policy, refuses rather than downloading — and says which flag would
/// have allowed it.
#[test]
fn an_unattended_run_refuses_the_download_and_names_the_consent_flag() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();

    let output = integrate_drell_yan(cwd.path(), home.path(), out.path(), &[]);
    assert!(
        !output.status.success(),
        "a missing set with no terminal to ask on must fail"
    );
    let stderr = stderr_of(&output);
    for expected in [
        DEFAULT_PDF_SET,
        "https://lhapdfsets.web.cern.ch/current/NNPDF23_lo_as_0130_qed.tar.gz",
        "60d3c1df1c31e5840f91f4217163ae30a256b9291a5adc894882e86607ef5d63",
        "no terminal",
        "--yes",
    ] {
        assert!(
            stderr.contains(expected),
            "refusal should mention {expected}, got:\n{stderr}"
        );
    }
    assert!(
        !home.path().join("pdf").join(DEFAULT_PDF_SET).exists(),
        "a refused fetch must not leave a cache entry"
    );
}

/// The refusal names the cache directory the archive would land in, which is the
/// same directory a user unpacking it by hand has to use. Without that the
/// message tells them what to download but not where to put it.
#[test]
fn the_refusal_names_the_directory_to_install_into() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();

    let stderr = stderr_of(&integrate_drell_yan(
        cwd.path(),
        home.path(),
        out.path(),
        &[],
    ));
    let destination = home.path().join("pdf").join(DEFAULT_PDF_SET);
    assert!(
        stderr.contains(&destination.display().to_string()),
        "refusal should name {}, got:\n{stderr}",
        destination.display()
    );
}

/// Consent given on the command line does not override an environment that
/// forbids downloads, and the refusal names the environment variable rather
/// than the flag the user did pass.
#[test]
fn the_kill_switch_outranks_explicit_consent() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();

    let output = vibegraph(cwd.path(), home.path())
        .env("VIBEGRAPH_NO_NETWORK", "1")
        .arg("integrate")
        .arg(validation_dir().join("dy13_proc_card.dat"))
        .arg("--run-card")
        .arg(validation_dir().join("dy13_default_run_card.dat"))
        .arg("--out")
        .arg(out.path())
        .args(["--neval", "2000", "--niter", "2", "--yes"])
        .output()
        .expect("spawn vibegraph");

    assert!(
        !output.status.success(),
        "--yes must not defeat the kill switch"
    );
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("VIBEGRAPH_NO_NETWORK"),
        "refusal should name the variable that produced it, got:\n{stderr}"
    );
}

/// A model that is not the interned SM and is not on disk fails with an
/// explanation, not a download attempt against a guessed URL.
#[test]
fn an_unresolvable_model_explains_that_models_are_never_downloaded() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    let proc_card = cwd.path().join("proc_card.dat");
    std::fs::write(
        &proc_card,
        "import model NoSuchModel\ngenerate e+ e- > mu+ mu-\n",
    )
    .unwrap();

    let output = vibegraph(cwd.path(), home.path())
        .arg("integrate")
        .arg(&proc_card)
        .arg("--out")
        .arg(out.path())
        .args(["--neval", "1000", "--niter", "2"])
        .output()
        .expect("spawn vibegraph");

    assert!(!output.status.success(), "a missing model must fail");
    let stderr = stderr_of(&output);
    for expected in [
        "NoSuchModel",
        "does not download UFO models",
        "particles.py",
        "--ufo-dir",
        "VIBEGRAPH_UFO_DIR",
    ] {
        assert!(
            stderr.contains(expected),
            "error should mention {expected}, got:\n{stderr}"
        );
    }
}

/// The acceptance path's last step, on the binary rather than through the
/// library: an emitted sample re-read and checked by the shipped `check-events`.
///
/// A fixed-energy process stands in for the hadronic one because event
/// generation for proton beams is not wired yet; the checks `check-events`
/// applies do not depend on which.
#[test]
fn a_generated_sample_passes_check_events_and_a_damaged_one_does_not() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let proc_card = cwd.path().join("proc_card.dat");
    let run_card = cwd.path().join("run_card.dat");
    std::fs::write(&proc_card, "import model sm\ngenerate e+ e- > mu+ mu-\n").unwrap();
    std::fs::write(
        &run_card,
        "  0 = lpp1\n  0 = lpp2\n  45.6 = ebeam1\n  45.6 = ebeam2\n",
    )
    .unwrap();

    let out = cwd.path().join("run");
    let integrated = vibegraph(cwd.path(), home.path())
        .arg("integrate")
        .arg(&proc_card)
        .arg("--run-card")
        .arg(&run_card)
        .arg("--out")
        .arg(&out)
        .args(["--neval", "8000", "--niter", "3"])
        .output()
        .expect("spawn vibegraph");
    assert!(
        integrated.status.success(),
        "integrate failed:\n{}",
        stderr_of(&integrated)
    );

    let events = cwd.path().join("events.lhe");
    let generated = vibegraph(cwd.path(), home.path())
        .arg("generate")
        .arg(out.join("grid.bin.zst"))
        .arg(&proc_card)
        .arg("--run-card")
        .arg(&run_card)
        .args(["--nevents", "200", "--seed", "7"])
        .arg("-o")
        .arg(&events)
        .output()
        .expect("spawn vibegraph");
    assert!(
        generated.status.success(),
        "generate failed:\n{}",
        stderr_of(&generated)
    );

    let checked = vibegraph(cwd.path(), home.path())
        .arg("check-events")
        .arg(&events)
        .args(["--min-events", "200"])
        .output()
        .expect("spawn vibegraph");
    assert!(
        checked.status.success(),
        "check-events rejected a sample this binary just wrote:\n{}",
        stderr_of(&checked)
    );

    // A checker that accepts everything would pass the assertion above just as
    // well, so the same file is re-emitted with one leg's `pz` displaced by a
    // GeV — enough to break both momentum balance and that leg's mass shell.
    let damaged = cwd.path().join("damaged.lhe");
    displace_one_leg(&events, &damaged);
    let rejected = vibegraph(cwd.path(), home.path())
        .arg("check-events")
        .arg(&damaged)
        .output()
        .expect("spawn vibegraph");
    assert!(
        !rejected.status.success(),
        "check-events accepted a file with a displaced leg"
    );
    let stderr = stderr_of(&rejected);
    assert!(
        stderr.contains("does not balance"),
        "the rejection should say what failed, got:\n{stderr}"
    );
}

/// Re-emit `source` with the third leg of its first event moved along `z`.
fn displace_one_leg(source: &Path, destination: &Path) {
    let text = std::fs::read_to_string(source).expect("read the sample");
    let mut file = LheFile::parse(&text).expect("our own file parses");
    file.events[0].particles[2].momentum[3] += 1.0;

    let out = std::fs::File::create(destination).expect("create the damaged file");
    let mut writer = LheWriter::begin(std::io::BufWriter::new(out), &file.init, None)
        .expect("write the init block");
    for event in &file.events {
        writer.write_event(event).expect("write an event");
    }
    writer.finish().expect("close the file");
}
