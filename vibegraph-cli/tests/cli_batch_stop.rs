//! How `integrate` ends: the stop reason its result carries, and the way
//! SIGTERM and SIGINT stop a run outside the display, against real child
//! processes.
//!
//! Needs only the interned SM model, so it runs in the default suite.

#![cfg(unix)]

mod common;

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde_json::Value;
use vibegraph::artifact::IntegrateArtifact;

const EBEAM: f64 = 45.6;

fn cards(dir: &Path) -> (PathBuf, PathBuf) {
    let proc_card = dir.join("proc_card.dat");
    let run_card = dir.join("run_card.dat");
    std::fs::write(&proc_card, "import model sm\ngenerate e+ e- > mu+ mu-\n").unwrap();
    std::fs::write(&run_card, common::fixed_beams(EBEAM, EBEAM)).unwrap();
    (proc_card, run_card)
}

fn integrate(dir: &Path, extra: &[&str]) -> Command {
    let (proc_card, run_card) = cards(dir);
    let mut cmd = common::vibegraph(dir, dir);
    cmd.arg("integrate")
        .arg(proc_card)
        .arg("--run-card")
        .arg(run_card)
        .arg("--out")
        .arg(dir.join("out"))
        .args(["-j", "1"])
        .args(extra);
    cmd
}

fn result_of(stdout: &str) -> Value {
    let last = stdout.lines().last().expect("a final object");
    let object: Value = serde_json::from_str(last).expect("the last line is JSON");
    assert_eq!(object["kind"], "result", "{last}");
    object
}

/// A run to completion under `extra`, its `result` object.
fn completed(extra: &[&str]) -> Value {
    let tmp = tempfile::tempdir().unwrap();
    let output = integrate(tmp.path(), extra)
        .arg("--output-ldjson")
        .output()
        .expect("spawn vibegraph");
    assert!(
        output.status.success(),
        "{}{}",
        common::stdout_of(&output),
        common::stderr_of(&output)
    );
    result_of(&common::stdout_of(&output))
}

#[test]
fn a_fixed_budget_run_says_so() {
    let r = completed(&["--fixed-budget", "--neval", "4000", "--niter", "4"]);
    assert_eq!(r["stop_reason"], "fixed_budget");
    assert_eq!(r["target_rel"], Value::Null);
    let quoted = r["rel_err_quoted"].as_f64().unwrap();
    assert_eq!(
        quoted,
        r["sigma_err"].as_f64().unwrap() / r["sigma"].as_f64().unwrap()
    );
    assert!(r["rel_err_scaled"].as_f64().unwrap() >= quoted * (1.0 - 1e-12));
}

#[test]
fn a_met_target_says_so_and_reports_the_error_it_reached() {
    let r = completed(&["--target-rel", "0.2", "--neval", "4000"]);
    assert_eq!(r["stop_reason"], "target_met");
    assert_eq!(r["target_rel"].as_f64(), Some(0.2));
    assert!(r["rel_err_scaled"].as_f64().unwrap() <= 0.2, "{r}");
}

#[test]
fn an_unmet_target_at_the_iteration_cap_says_so() {
    let r = completed(&[
        "--target-rel",
        "1e-9",
        "--neval",
        "2000",
        "--min-iters",
        "3",
        "--max-iters",
        "4",
    ]);
    assert_eq!(r["stop_reason"], "max_iters");
    assert!(r["rel_err_scaled"].as_f64().unwrap() > 1e-9);
}

#[test]
fn an_unmet_target_at_the_evaluation_cap_says_so() {
    let r = completed(&[
        "--target-rel",
        "1e-9",
        "--neval",
        "2000",
        "--min-iters",
        "3",
        "--max-points",
        "20000",
    ]);
    assert_eq!(r["stop_reason"], "max_evals");
}

/// A child with its stdout read on a thread, line by line.
struct Run {
    child: Child,
    lines: mpsc::Receiver<String>,
    seen: Vec<String>,
}

impl Run {
    fn start(mut cmd: Command) -> Run {
        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn vibegraph");
        let (tx, lines) = mpsc::channel();
        let stdout = child.stdout.take().unwrap();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        Run {
            child,
            lines,
            seen: Vec::new(),
        }
    }

    /// Read lines until one satisfies `done`, within a minute.
    fn until(&mut self, done: impl Fn(&Value) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let line = self
                .lines
                .recv_timeout(left)
                .unwrap_or_else(|e| panic!("{e}; saw {:?}", self.seen));
            let hit = serde_json::from_str::<Value>(&line).is_ok_and(|v| done(&v));
            self.seen.push(line);
            if hit {
                return;
            }
        }
    }

    /// Past the warm-up: an iteration that enters the combination has finished.
    fn until_sampling_kept(&mut self) {
        self.until(|v| {
            v["kind"] == "progress"
                && v["fields"]["stage"] == "vegas"
                && v["fields"]["done"].as_u64().unwrap_or(0) >= 4
        });
    }

    fn signal(&self, name: &str) {
        let status = Command::new("kill")
            .arg(format!("-{name}"))
            .arg(self.child.id().to_string())
            .status()
            .expect("kill");
        assert!(status.success());
    }

    /// Everything the child wrote after what was read so far, and its status.
    fn finish(mut self) -> (std::process::ExitStatus, Vec<String>) {
        let status = self.child.wait().expect("wait");
        self.seen.extend(self.lines.try_iter());
        while let Ok(line) = self.lines.recv_timeout(Duration::from_millis(200)) {
            self.seen.push(line);
        }
        (status, self.seen)
    }
}

/// A run that would take far longer than the test, by iterations.
fn endless(dir: &Path, neval: &str, extra: &[&str]) -> Command {
    let mut cmd = integrate(
        dir,
        &[
            "--fixed-budget",
            "--neval",
            neval,
            "--niter",
            "1000000",
            "--output-ldjson",
        ],
    );
    cmd.args(extra);
    cmd
}

fn artifact_of(dir: &Path) -> IntegrateArtifact {
    IntegrateArtifact::read_from_path(&dir.join("out/grid.bin.zst")).expect("the artifact")
}

/// The first SIGTERM or SIGINT ends the run the way `q` does: the iteration in
/// flight finishes, the artifact is banked and the stream ends on a result that
/// says why.
#[test]
fn the_first_term_or_int_banks_the_artifact_and_ends_the_stream_with_a_result() {
    for signal in ["TERM", "INT"] {
        let tmp = tempfile::tempdir().unwrap();
        let mut run = Run::start(endless(tmp.path(), "4000", &[]));
        run.until_sampling_kept();
        run.signal(signal);
        let (status, lines) = run.finish();
        let stdout = lines.join("\n");
        assert!(status.success(), "SIG{signal}: {status:?}\n{stdout}");
        let result = result_of(&stdout);
        assert_eq!(result["stop_reason"], "stopped", "SIG{signal}");
        assert_eq!(
            result["rel_err_quoted"].as_f64().map(f64::is_finite),
            Some(true)
        );
        let artifact = artifact_of(tmp.path());
        assert_eq!(result["sigma"].as_f64(), Some(artifact.sigma_pb));
        assert!(artifact.niter < 1_000_000, "{}", artifact.niter);
    }
}

/// Plain text mode stops the same way: the result line is printed and the
/// artifact banked.
#[test]
fn the_first_term_stops_a_plain_text_run_too() {
    let tmp = tempfile::tempdir().unwrap();
    let mut child = integrate(
        tmp.path(),
        &[
            "--fixed-budget",
            "--neval",
            "4000",
            "--niter",
            "1000000",
            "--no-tui",
        ],
    )
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("spawn vibegraph");
    let mut stderr = BufReader::new(child.stderr.take().unwrap());
    let mut seen = String::new();
    loop {
        let mut line = String::new();
        let n = stderr.read_line(&mut line).expect("read stderr");
        assert!(n > 0, "the run ended before sampling:\n{seen}");
        seen.push_str(&line);
        if line.contains("points per iteration") {
            break;
        }
    }
    // Iterations of this size take milliseconds, so a second is far past the
    // warm-up.
    std::thread::sleep(Duration::from_secs(1));
    let status = Command::new("kill")
        .arg("-TERM")
        .arg(child.id().to_string())
        .status()
        .unwrap();
    assert!(status.success());
    let mut rest = String::new();
    std::io::Read::read_to_string(&mut stderr, &mut rest).unwrap();
    let output = child.wait_with_output().expect("wait");
    assert!(output.status.success(), "{:?}\n{seen}{rest}", output.status);
    let stdout = common::stdout_of(&output);
    assert!(stdout.starts_with("σ = "), "{stdout}");
    assert!(stdout.contains("wrote "), "{stdout}");
    let artifact = artifact_of(tmp.path());
    assert!(artifact.niter < 1_000_000);
}

/// A second signal while the run finishes its iteration takes the default
/// action: the process dies of the signal, and no artifact is banked.
#[test]
fn a_second_signal_takes_the_default_action() {
    let tmp = tempfile::tempdir().unwrap();
    // An iteration long enough that the second signal lands inside it.
    let mut run = Run::start(endless(tmp.path(), "20000000", &[]));
    run.until(|v| v["kind"] == "progress" && v["fields"]["stage"] == "vegas");
    run.signal("TERM");
    std::thread::sleep(Duration::from_millis(100));
    run.signal("TERM");
    let (status, _) = run.finish();
    use std::os::unix::process::ExitStatusExt;
    assert_eq!(status.signal(), Some(15), "{status:?}");
    assert!(!tmp.path().join("out/grid.bin.zst").exists());
}

/// A stop that arrives before any iteration past the warm-up is kept has
/// measured nothing: the stream ends in a refusal rather than a result, and no
/// artifact is written.
#[test]
fn a_stop_before_any_iteration_is_kept_ends_in_a_refusal() {
    let tmp = tempfile::tempdir().unwrap();
    let mut run = Run::start(endless(tmp.path(), "10000000", &[]));
    run.until(|v| v["kind"] == "progress" && v["fields"]["stage"] == "vegas");
    run.signal("TERM");
    let (status, lines) = run.finish();
    assert_eq!(status.code(), Some(1), "{status:?}");
    let last: Value = serde_json::from_str(lines.last().expect("a final object")).unwrap();
    assert_eq!(last["kind"], "refusal", "{last}");
    assert_eq!(last["error_kind"], "support");
    assert!(
        last["message"]
            .as_str()
            .unwrap()
            .contains("stopped before any iteration was kept"),
        "{last}"
    );
    assert!(!tmp.path().join("out/grid.bin.zst").exists());
}
