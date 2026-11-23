//! The `phase` records of `--output-ldjson`: each phase of `integrate` and
//! `generate` writes a start record when it is entered and an end record when it
//! is left, in order, on a monotone clock, and a run killed inside a phase leaves
//! that phase's start record in its output.
//!
//! The fixed-beam tests need only the interned SM model. The proton test needs
//! the fetched NNPDF set under `validation/pdf`, which only the banked layer has,
//! so it runs under `extended-validation`.

#![cfg(unix)]

mod common;

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

const EBEAM: f64 = 45.6;

const FIXED_BEAM_PHASES: [&str; 9] = [
    "input",
    "enumerate",
    "amplitude_probe",
    "compile",
    "channels",
    "alpha_survey",
    "vegas",
    "configurations",
    "bank",
];

#[cfg(feature = "extended-validation")]
const PROTON_PHASES: [&str; 10] = [
    "input",
    "enumerate",
    "amplitude_probe",
    "pdf",
    "compile",
    "channels",
    "alpha_survey",
    "vegas",
    "configurations",
    "bank",
];

fn fixed_beam_integrate(dir: &Path, extra: &[&str]) -> Command {
    let proc_card = dir.join("proc_card.dat");
    let run_card = dir.join("run_card.dat");
    std::fs::write(&proc_card, "import model sm\ngenerate e+ e- > mu+ mu-\n").unwrap();
    std::fs::write(&run_card, common::fixed_beams(EBEAM, EBEAM)).unwrap();
    let mut cmd = common::vibegraph(dir, dir);
    cmd.arg("integrate")
        .arg(proc_card)
        .arg("--run-card")
        .arg(run_card)
        .arg("--out")
        .arg(dir.join("out"))
        .args(["-j", "1", "--output-ldjson"])
        .args(extra);
    cmd
}

/// Every line of `stdout`, parsed.
fn parse_stream(stdout: &str) -> Vec<Value> {
    stdout
        .lines()
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("{e}: {l}")))
        .collect()
}

fn phases(objects: &[Value]) -> Vec<&Value> {
    objects.iter().filter(|o| o["kind"] == "phase").collect()
}

/// The records pair up start then end, phase by phase, in `expected` order, on
/// a clock that never runs backwards, each end carrying its own wall time; and
/// every record precedes the final object.
fn assert_paired_in_order(objects: &[Value], expected: &[&str]) {
    let records = phases(objects);
    assert_eq!(records.len(), 2 * expected.len(), "{records:#?}");
    let mut last = 0.0;
    for (pair, name) in records.chunks(2).zip(expected) {
        let (start, end) = (pair[0], pair[1]);
        assert_eq!(start["event"], "start", "{start}");
        assert_eq!(end["event"], "end", "{end}");
        assert_eq!(start["phase"], *name, "{start}");
        assert_eq!(end["phase"], *name, "{end}");
        assert_eq!(end["outcome"], "ok", "{end}");
        assert_eq!(start["schema_version"], 1);
        let t0 = start["elapsed_s"].as_f64().unwrap();
        let t1 = end["elapsed_s"].as_f64().unwrap();
        assert!(t0 >= last && t1 >= t0, "{start} {end}");
        let wall = end["wall_s"].as_f64().unwrap();
        assert!((wall - (t1 - t0)).abs() <= 1e-9 * t1.max(1.0), "{end}");
        assert!(end["sizes"].is_object(), "{end}");
        last = t1;
    }
    let final_at = objects
        .iter()
        .position(|o| o["kind"] == "result")
        .expect("a result");
    let last_phase = objects.iter().rposition(|o| o["kind"] == "phase").unwrap();
    assert!(last_phase < final_at);
}

fn size(objects: &[Value], phase: &str, key: &str) -> u64 {
    phases(objects)
        .into_iter()
        .find(|p| p["phase"] == phase && p["event"] == "end")
        .and_then(|p| p["sizes"][key].as_u64())
        .unwrap_or_else(|| panic!("{phase}.{key}"))
}

#[test]
fn a_fixed_beam_integration_streams_its_phases_in_order() {
    let tmp = tempfile::tempdir().unwrap();
    let output = fixed_beam_integrate(
        tmp.path(),
        &["--fixed-budget", "--neval", "4000", "--niter", "4"],
    )
    .output()
    .expect("spawn vibegraph");
    let stdout = common::stdout_of(&output);
    assert!(
        output.status.success(),
        "{stdout}{}",
        common::stderr_of(&output)
    );
    let objects = parse_stream(&stdout);
    assert_paired_in_order(&objects, &FIXED_BEAM_PHASES);
    // e+ e- > mu+ mu-: one subprocess, the photon and the Z.
    assert_eq!(size(&objects, "enumerate", "subprocesses"), 1);
    assert_eq!(size(&objects, "enumerate", "diagrams"), 2);
    assert_eq!(size(&objects, "channels", "channels"), 2);
    assert_eq!(size(&objects, "vegas", "iterations"), 4);

    // `generate` from the artifact the run wrote.
    let events = tmp.path().join("events.lhe");
    let output = common::vibegraph(tmp.path(), tmp.path())
        .arg("generate")
        .arg(tmp.path().join("out/grid.bin.zst"))
        .args(["--nevents", "50", "-o"])
        .arg(&events)
        .args(["--seed", "7", "-j", "1", "--output-ldjson"])
        .output()
        .expect("spawn vibegraph");
    let stdout = common::stdout_of(&output);
    assert!(
        output.status.success(),
        "{stdout}{}",
        common::stderr_of(&output)
    );
    let objects = parse_stream(&stdout);
    assert_paired_in_order(&objects, &["input", "pdf", "generate"]);
    assert_eq!(size(&objects, "generate", "events_written"), 50);
}

#[cfg(feature = "extended-validation")]
#[test]
fn a_proton_integration_streams_its_phases_in_order() {
    use common::PdfDataPath;

    let pdf_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/pdf");
    if !pdf_dir.join("NNPDF23_lo_as_0130_qed").is_dir() {
        vibegraph::validation::require(
            "a_proton_integration_streams_its_phases_in_order",
            "the NNPDF23_lo_as_0130_qed set",
            pdf_dir.display(),
        );
    }
    let tmp = tempfile::tempdir().unwrap();
    let proc_card = tmp.path().join("proc_card.dat");
    std::fs::write(&proc_card, "import model sm\ngenerate p p > e+ e-\n").unwrap();
    let output = common::vibegraph(tmp.path(), tmp.path())
        .pdf_data_path(&pdf_dir)
        .arg("integrate")
        .arg(&proc_card)
        .arg("--run-card")
        .arg(common::validation_dir().join("pp_to_llj_mlm_run_card.dat"))
        .args(["--out", "out", "--fixed-budget", "--no-network"])
        .args([
            "--neval",
            "2000",
            "--niter",
            "3",
            "-j",
            "1",
            "--output-ldjson",
        ])
        .output()
        .expect("spawn vibegraph");
    let stdout = common::stdout_of(&output);
    assert!(
        output.status.success(),
        "{stdout}{}",
        common::stderr_of(&output)
    );
    let objects = parse_stream(&stdout);
    assert_paired_in_order(&objects, &PROTON_PHASES);
    assert!(size(&objects, "compile", "flavour_groups") >= 1);
    assert!(
        size(&objects, "compile", "subprocesses") >= size(&objects, "compile", "flavour_groups")
    );
    assert!(size(&objects, "channels", "channels") >= 1);
}

/// `--check` ends the run after the channels are built, and the stream with it.
#[test]
fn a_check_run_stops_after_the_channels() {
    let tmp = tempfile::tempdir().unwrap();
    let output = fixed_beam_integrate(tmp.path(), &["--check"])
        .output()
        .expect("spawn vibegraph");
    let stdout = common::stdout_of(&output);
    assert!(output.status.success(), "{stdout}");
    let objects = parse_stream(&stdout);
    assert_paired_in_order(&objects, &FIXED_BEAM_PHASES[..5]);
}

/// A refusal inside a phase ends that phase as `failed`, before the refusal.
#[test]
fn a_refusal_ends_its_phase_as_failed() {
    let tmp = tempfile::tempdir().unwrap();
    let proc_card = tmp.path().join("proc_card.dat");
    std::fs::write(&proc_card, "import model sm\ngenerate e+ e- > e+ mu-\n").unwrap();
    let run_card = tmp.path().join("run_card.dat");
    std::fs::write(&run_card, common::fixed_beams(EBEAM, EBEAM)).unwrap();
    let output = common::vibegraph(tmp.path(), tmp.path())
        .arg("integrate")
        .arg(&proc_card)
        .arg("--run-card")
        .arg(&run_card)
        .args(["--out", "out", "--output-ldjson"])
        .output()
        .expect("spawn vibegraph");
    let stdout = common::stdout_of(&output);
    assert!(!output.status.success(), "{stdout}");
    let objects = parse_stream(&stdout);
    let records = phases(&objects);
    let last = records.last().expect("phase records");
    assert_eq!(last["event"], "end", "{last}");
    assert_eq!(last["outcome"], "failed", "{last}");
    let at = |kind: &str| objects.iter().rposition(|o| o["kind"] == kind).unwrap();
    assert!(at("phase") < at("refusal"));
}

/// SIGKILL runs nothing on the way out, so what the stream holds is what was
/// written before the signal: the start of the phase the run was killed in, and
/// no end for it.
#[test]
fn a_run_killed_in_a_phase_leaves_its_start_record_on_disk() {
    let tmp = tempfile::tempdir().unwrap();
    let log = tmp.path().join("stream.ldjson");
    let mut child = fixed_beam_integrate(
        tmp.path(),
        &["--fixed-budget", "--neval", "4000", "--niter", "1000000"],
    )
    .stdout(std::fs::File::create(&log).unwrap())
    .stderr(Stdio::null())
    .spawn()
    .expect("spawn vibegraph");

    let entered = |text: &str| {
        objects_so_far(text)
            .iter()
            .any(|o| o["kind"] == "phase" && o["phase"] == "vegas" && o["event"] == "start")
    };
    let deadline = Instant::now() + Duration::from_secs(60);
    while !entered(&std::fs::read_to_string(&log).unwrap()) {
        assert!(
            Instant::now() < deadline,
            "no vegas start within a minute:\n{}",
            std::fs::read_to_string(&log).unwrap()
        );
        assert!(child.try_wait().unwrap().is_none(), "the run ended early");
        std::thread::sleep(Duration::from_millis(20));
    }
    // Inside the phase: a further iteration is under way.
    std::thread::sleep(Duration::from_millis(200));
    child.kill().expect("SIGKILL");
    let status = child.wait().unwrap();
    use std::os::unix::process::ExitStatusExt;
    assert_eq!(status.signal(), Some(9), "{status:?}");

    let text = std::fs::read_to_string(&log).unwrap();
    let objects = objects_so_far(&text);
    let records = phases(&objects);
    let last = records.last().expect("phase records");
    assert_eq!(last["phase"], "vegas", "{last}");
    assert_eq!(last["event"], "start", "{last}");
    assert!(!objects
        .iter()
        .any(|o| o["kind"] == "result" || o["kind"] == "refusal"));
    // The phases before it all ended.
    assert_paired_prefix(&records[..records.len() - 1], &FIXED_BEAM_PHASES[..6]);
}

/// The complete lines of a stream that may end mid-line.
fn objects_so_far(text: &str) -> Vec<Value> {
    text.lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

fn assert_paired_prefix(records: &[&Value], expected: &[&str]) {
    assert_eq!(records.len(), 2 * expected.len(), "{records:#?}");
    for (pair, name) in records.chunks(2).zip(expected) {
        assert_eq!(pair[0]["phase"], *name);
        assert_eq!(pair[0]["event"], "start");
        assert_eq!(pair[1]["phase"], *name);
        assert_eq!(pair[1]["event"], "end");
    }
}
