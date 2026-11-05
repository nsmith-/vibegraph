//! The `--output-ldjson` stream's schema, pinned end to end on the binary:
//! every `kind`, the schema version on every line, the final objects of
//! `integrate`, `generate` and `inspect`, and the refusal object.
//!
//! Needs only the interned SM model, so it runs in the default suite. The
//! physics of the numbers is not under test here — a wrong σ is carried into
//! the stream faithfully — only that the stream carries exactly what the
//! artifact and the event file record, at full precision.

mod common;

use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::OnceLock;

use serde_json::Value;
use vibegraph::artifact::{ChannelKey, IntegrateArtifact, FORMAT_VERSION};

const SCHEMA_VERSION: u64 = 1;
const PROCESS: &str = "e+ e- > mu+ mu-";
const EBEAM: f64 = 45.6;

/// Every line of `stdout`, parsed: each must be a JSON object carrying `kind`
/// and the schema version, or the test fails naming the line.
fn objects(output: &Output) -> Vec<Value> {
    let stdout = common::stdout_of(output);
    stdout
        .lines()
        .map(|line| {
            let value: Value = serde_json::from_str(line)
                .unwrap_or_else(|e| panic!("not a JSON line ({e}): {line}"));
            assert!(value.is_object(), "not an object: {line}");
            assert_eq!(
                value["schema_version"].as_u64(),
                Some(SCHEMA_VERSION),
                "{line}"
            );
            let kind = value["kind"].as_str().unwrap_or_else(|| panic!("{line}"));
            assert!(
                ["log", "progress", "result", "refusal"].contains(&kind),
                "unknown kind {kind}: {line}"
            );
            value
        })
        .collect()
}

fn of_kind<'a>(objects: &'a [Value], kind: &str) -> Vec<&'a Value> {
    objects.iter().filter(|o| o["kind"] == kind).collect()
}

/// How many `log` objects carry exactly `message`.
fn logged(objects: &[Value], message: &str) -> usize {
    of_kind(objects, "log")
        .iter()
        .filter(|l| l["fields"]["message"] == message)
        .count()
}

/// The one final object of a run's stream, which must be its last line.
fn final_object<'a>(objects: &'a [Value], kind: &str) -> &'a Value {
    let finals: Vec<&Value> = objects
        .iter()
        .filter(|o| o["kind"] == "result" || o["kind"] == "refusal")
        .collect();
    assert_eq!(finals.len(), 1, "one final object: {finals:?}");
    assert_eq!(finals[0]["kind"], kind, "{:?}", finals[0]);
    assert_eq!(
        objects.last(),
        Some(finals[0]),
        "the final object is the last line"
    );
    finals[0]
}

struct Integrated {
    _tmp: tempfile::TempDir,
    dir: PathBuf,
    artifact_path: PathBuf,
    artifact: IntegrateArtifact,
    output: Output,
}

fn cards(dir: &Path) -> (PathBuf, PathBuf) {
    let proc_card = dir.join("proc_card.dat");
    let run_card = dir.join("run_card.dat");
    std::fs::write(&proc_card, format!("import model sm\ngenerate {PROCESS}\n")).unwrap();
    std::fs::write(&run_card, common::fixed_beams(EBEAM, EBEAM)).unwrap();
    (proc_card, run_card)
}

fn integrate_ldjson(extra: &[&str]) -> Integrated {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().to_path_buf();
    let (proc_card, run_card) = cards(&dir);
    let out = dir.join("out");
    let output = common::vibegraph(&dir, &dir)
        .arg("integrate")
        .arg(&proc_card)
        .arg("--run-card")
        .arg(&run_card)
        .arg("--out")
        .arg(&out)
        .args(["--neval", "4000", "--niter", "4", "--output-ldjson"])
        .args(extra)
        .output()
        .expect("spawn vibegraph");
    assert!(
        output.status.success(),
        "integrate failed:\n{}{}",
        common::stdout_of(&output),
        common::stderr_of(&output)
    );
    let artifact_path = out.join("grid.bin.zst");
    let artifact = IntegrateArtifact::read_from_path(&artifact_path).expect("reload artifact");
    Integrated {
        _tmp: tmp,
        dir,
        artifact_path,
        artifact,
        output,
    }
}

/// The run every test reading a default-level stream shares.
fn integrated() -> &'static Integrated {
    static RUN: OnceLock<Integrated> = OnceLock::new();
    RUN.get_or_init(|| integrate_ldjson(&[]))
}

/// The stream is all objects, on stdout, in the three event kinds plus one
/// result; the text modes' result lines and their stderr commentary are gone.
#[test]
fn integrate_streams_logs_progress_and_one_result() {
    let run = integrated();
    let objects = objects(&run.output);
    let logs = of_kind(&objects, "log");
    let progress = of_kind(&objects, "progress");
    assert!(!logs.is_empty() && !progress.is_empty(), "{objects:?}");
    final_object(&objects, "result");
    assert_eq!(
        common::stderr_of(&run.output),
        "",
        "the stream owns the run's whole output"
    );

    // A log object is the JSON formatter's event: level, target, and the text
    // in `fields.message`.
    let process_line = logs
        .iter()
        .find(|l| {
            l["fields"]["message"]
                .as_str()
                .is_some_and(|m| m.starts_with("process:"))
        })
        .expect("the process line is logged");
    assert_eq!(process_line["level"], "INFO");
    assert!(process_line["target"]
        .as_str()
        .unwrap()
        .starts_with("vibegraph"));

    // A progress object carries the progress field contract under `fields`.
    for p in &progress {
        assert_eq!(p["target"], "vibegraph::progress", "{p}");
        assert!(p["fields"]["stage"].is_string(), "{p}");
        assert!(p["fields"]["done"].is_u64(), "{p}");
    }
    let vegas: Vec<&&Value> = progress
        .iter()
        .filter(|p| p["fields"]["stage"] == "vegas" && p["fields"]["sigma"].is_number())
        .collect();
    assert!(!vegas.is_empty(), "no VEGAS iteration was reported");
}

/// The progress objects are measurements, not commentary: a quiet run keeps
/// every one of them and drops the info-level log objects.
#[test]
fn progress_is_streamed_whatever_the_level() {
    let quiet = integrate_ldjson(&["-q"]);
    let objects = objects(&quiet.output);
    assert!(
        of_kind(&objects, "log")
            .iter()
            .all(|l| l["level"] == "WARN" || l["level"] == "ERROR"),
        "-q streamed an info line"
    );
    assert!(!of_kind(&objects, "progress").is_empty());
    final_object(&objects, "result");
}

/// The result object is the artifact's record, value for value and bit for
/// bit, in the documented fields.
#[test]
fn the_integrate_result_carries_the_recorded_values() {
    let run = integrated();
    let objects = objects(&run.output);
    let result = final_object(&objects, "result");
    let a = &run.artifact;

    assert_eq!(result["process"], PROCESS);
    assert_eq!(result["model"]["digest"], a.model.digest.as_str());
    assert_eq!(result["observable"], "cross_section");
    assert_eq!(result["unit"], "pb");
    assert_eq!(
        result["sigma"].as_f64().unwrap().to_bits(),
        a.sigma_pb.to_bits()
    );
    assert_eq!(
        result["sigma_err"].as_f64().unwrap().to_bits(),
        a.sigma_err_pb.to_bits()
    );
    match a.chi2_per_dof {
        Some(chi2) => assert_eq!(result["chi2_per_dof"].as_f64(), Some(chi2)),
        None => assert!(result["chi2_per_dof"].is_null()),
    }
    assert_eq!(result["mu_f"], serde_json::json!({"type": "no_densities"}));
    assert_eq!(result["seed"].as_u64(), Some(a.seed));
    assert_eq!(result["multiplicities"], serde_json::json!([]));
    assert_eq!(
        result["artifact"],
        run.artifact_path.display().to_string().as_str()
    );
    assert_eq!(
        result["format_version"].as_u64(),
        Some(u64::from(FORMAT_VERSION))
    );

    let channels = result["channels"].as_array().unwrap();
    assert_eq!(channels.len(), a.channels.len());
    assert!(channels.len() > 1, "a multichannel run");
    for (c, banked) in channels.iter().zip(&a.channels) {
        let ChannelKey::Channel { channel } = banked.key else {
            panic!(
                "a fixed-energy channel is keyed by its index: {:?}",
                banked.key
            );
        };
        assert_eq!(
            c["key"],
            serde_json::json!({"type": "channel", "channel": channel})
        );
        assert_eq!(c["alpha"].as_f64(), Some(banked.alpha));
        assert_eq!(c["neval"].as_u64(), Some(banked.neval as u64));
        assert_eq!(c["sigma"].as_f64(), Some(banked.sigma_pb));
        assert_eq!(c["sigma_err"].as_f64(), Some(banked.sigma_err_pb));
        assert_eq!(c["chi2_per_dof"].as_f64(), banked.chi2_per_dof);
    }
}

/// `inspect --output-ldjson` of the artifact a run wrote is the object the run
/// emitted, whole.
#[test]
fn inspect_emits_the_object_integrate_emitted() {
    let run = integrated();
    let emitted = final_object(&objects(&run.output), "result").clone();
    let inspected = common::vibegraph(&run.dir, &run.dir)
        .arg("inspect")
        .arg(&run.artifact_path)
        .arg("--output-ldjson")
        .output()
        .expect("spawn vibegraph");
    assert!(
        inspected.status.success(),
        "{}",
        common::stderr_of(&inspected)
    );
    let streamed = objects(&inspected);
    assert_eq!(final_object(&streamed, "result"), &emitted);

    // The card `integrate` read is announced once; the copy `inspect` parses
    // back out of the artifact is not a card read, and is not announced.
    let announced = format!("generate {PROCESS} @1");
    assert_eq!(logged(&objects(&run.output), &announced), 1);
    assert_eq!(logged(&streamed, &announced), 0, "{streamed:?}");
}

/// Without the flag, `inspect` prints the recorded results as text: the result
/// line `integrate` printed, and the record's provenance.
#[test]
fn inspect_prints_the_recorded_results_as_text() {
    let run = integrated();
    let inspected = common::vibegraph(&run.dir, &run.dir)
        .arg("inspect")
        .arg(&run.artifact_path)
        .output()
        .expect("spawn vibegraph");
    assert!(
        inspected.status.success(),
        "{}",
        common::stderr_of(&inspected)
    );
    let stdout = common::stdout_of(&inspected);
    let line = common::result_line(&stdout);
    assert_eq!(line.symbol, "σ");
    let a = &run.artifact;
    assert!(
        (line.value - a.sigma_pb).abs() <= a.sigma_err_pb,
        "{} against {} ± {}",
        line.line,
        a.sigma_pb,
        a.sigma_err_pb
    );
    assert!(stdout.contains(&format!("process:  {PROCESS}")), "{stdout}");
    assert!(stdout.contains("scale:    no parton densities"), "{stdout}");
    assert!(
        stdout.contains(&format!(
            "artifact: {} (format version {FORMAT_VERSION})",
            run.artifact_path.display()
        )),
        "{stdout}"
    );
    assert_eq!(
        stdout.lines().filter(|l| l.starts_with("channel ")).count(),
        a.channels.len(),
        "{stdout}"
    );
}

/// `generate`'s final object: what was written, where, and the sample's own σ
/// with its pull against the integration.
#[test]
fn the_generate_result_carries_the_sample_and_its_pull() {
    let run = integrated();
    let events = run.dir.join("events.lhe");
    let generated = common::vibegraph(&run.dir, &run.dir)
        .arg("generate")
        .arg(&run.artifact_path)
        .args(["--seed", "7", "--nevents", "500", "--output-ldjson", "-o"])
        .arg(&events)
        .output()
        .expect("spawn vibegraph");
    assert!(
        generated.status.success(),
        "{}",
        common::stdout_of(&generated)
    );
    let objects = objects(&generated);
    assert!(!of_kind(&objects, "log").is_empty());
    assert_eq!(logged(&objects, &format!("generate {PROCESS} @1")), 0);
    assert!(!of_kind(&objects, "progress").is_empty());
    let result = final_object(&objects, "result");
    assert!(events.exists());

    assert_eq!(result["events_written"].as_u64(), Some(500));
    assert!(result["events_drawn"].as_u64().unwrap() <= 500);
    assert_eq!(result["output"], events.display().to_string().as_str());
    assert_eq!(
        result["artifact"],
        run.artifact_path.display().to_string().as_str()
    );
    assert_eq!(result["observable"], "cross_section");
    assert_eq!(result["unit"], "pb");
    assert_eq!(
        result["integration_sigma"].as_f64(),
        Some(run.artifact.sigma_pb)
    );
    assert_eq!(
        result["integration_sigma_err"].as_f64(),
        Some(run.artifact.sigma_err_pb)
    );
    let sample = result["sample_sigma"].as_f64().unwrap();
    let sample_err = result["sample_sigma_err"].as_f64().unwrap();
    let pull = result["pull"].as_f64().unwrap();
    assert!(sample > 0.0 && sample_err > 0.0, "{result}");
    let expected = (sample - run.artifact.sigma_pb) / sample_err.hypot(run.artifact.sigma_err_pb);
    assert!(
        (pull - expected).abs() <= 1e-12 * expected.abs().max(1.0),
        "{result}"
    );

    // The file's own record of the sample's estimate is the one the object
    // carries.
    let text = std::fs::read_to_string(&events).unwrap();
    assert!(
        text.contains(&format!("{sample:.6e} +- {sample_err:.6e}")),
        "the file does not record {sample:.6e} +- {sample_err:.6e}"
    );
}

/// A refusal is the stream's final object, with its stable kind and the
/// message the text modes print, and nothing goes to stderr.
#[test]
fn a_refusal_is_the_final_object_with_its_kind() {
    let tmp = tempfile::tempdir().unwrap();
    let missing = tmp.path().join("no_such_card.dat");
    let output = common::vibegraph(tmp.path(), tmp.path())
        .arg("integrate")
        .arg(&missing)
        .arg("--output-ldjson")
        .output()
        .expect("spawn vibegraph");
    assert!(!output.status.success());
    let objects = objects(&output);
    let refusal = final_object(&objects, "refusal");
    assert_eq!(refusal["error_kind"], "card");
    assert!(
        refusal["message"]
            .as_str()
            .unwrap()
            .starts_with(&format!("cannot read the proc card {}", missing.display())),
        "{refusal}"
    );
    assert_eq!(common::stderr_of(&output), "");
}

/// The refusals of a few distinct kinds, each where it is built: a flag
/// (`usage`), a refused download (`network`), and an artifact of no readable
/// version (`artifact`).
#[test]
fn refusals_carry_the_kind_of_what_failed() {
    let run = integrated();
    let no_seed = common::vibegraph(&run.dir, &run.dir)
        .arg("generate")
        .arg(&run.artifact_path)
        .arg("--output-ldjson")
        .output()
        .expect("spawn vibegraph");
    let refusal = final_object(&objects(&no_seed), "refusal").clone();
    assert_eq!(refusal["error_kind"], "usage", "{refusal}");
    assert!(refusal["message"]
        .as_str()
        .unwrap()
        .contains("generate needs --seed"));

    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    let refused =
        common::integrate_from_bare_cwd(cwd.path(), home.path(), out.path(), &["--output-ldjson"]);
    assert!(!refused.status.success());
    let refusal = final_object(&objects(&refused), "refusal").clone();
    assert_eq!(refusal["error_kind"], "network", "{refusal}");
    assert!(refusal["message"]
        .as_str()
        .unwrap()
        .contains("VIBEGRAPH_NO_NETWORK"));

    let garbage = run.dir.join("garbage.bin.zst");
    std::fs::write(&garbage, b"not an artifact").unwrap();
    let unreadable = common::vibegraph(&run.dir, &run.dir)
        .arg("inspect")
        .arg(&garbage)
        .arg("--output-ldjson")
        .output()
        .expect("spawn vibegraph");
    assert!(!unreadable.status.success());
    let refusal = final_object(&objects(&unreadable), "refusal").clone();
    assert_eq!(refusal["error_kind"], "artifact", "{refusal}");
}

/// `--output-ldjson` with `--tui` asks for two displays of one run, and
/// `check-events` has no object to stream: both are usage errors, refused
/// before anything runs.
#[test]
fn the_flag_is_refused_with_tui_and_by_check_events() {
    let tmp = tempfile::tempdir().unwrap();
    let (proc_card, run_card) = cards(tmp.path());
    let both = common::vibegraph(tmp.path(), tmp.path())
        .arg("integrate")
        .arg(&proc_card)
        .arg("--run-card")
        .arg(&run_card)
        .args(["--output-ldjson", "--tui"])
        .output()
        .expect("spawn vibegraph");
    assert_eq!(both.status.code(), Some(2), "{}", common::stderr_of(&both));
    let stderr = common::stderr_of(&both);
    assert!(stderr.contains("cannot be used with"), "{stderr}");
    assert!(!tmp.path().join("out").exists());

    let check = common::vibegraph(tmp.path(), tmp.path())
        .args(["check-events", "events.lhe", "--output-ldjson"])
        .output()
        .expect("spawn vibegraph");
    assert_eq!(
        check.status.code(),
        Some(2),
        "{}",
        common::stderr_of(&check)
    );
    let stderr = common::stderr_of(&check);
    assert!(
        stderr.contains("`check-events` prints a text report and has no --output-ldjson form"),
        "{stderr}"
    );
    assert_eq!(common::stdout_of(&check), "");
}
