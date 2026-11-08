//! A panic on a worker thread ends the run at once, with the panic's message on
//! stderr and a non-zero exit, in text and ldjson modes.
//!
//! The panic is injected by the parallel adaptation's fault point, which only
//! debug builds compile (`VIBEGRAPH_FAULT_WORKER_PANIC`, in `vegas.rs`), so the
//! test is compiled under the same condition. The fault panics in the first
//! task of an iteration and stalls every other task, so a run that waits for its
//! sibling tasks before unwinding outlives the bound by far.
#![cfg(debug_assertions)]

mod common;

use std::time::{Duration, Instant};

use common::{fixed_beams, stderr_of, vibegraph};

/// How long each non-panicking task of the iteration stalls.
const STALL_MS: u64 = 2000;

/// The wall time a run may take from spawn to exit: well under the time its
/// stalled siblings would hold it (about 30 tasks of `STALL_MS` on two threads).
const BOUND: Duration = Duration::from_secs(10);

fn run_with_injected_panic(extra: &[&str]) {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let proc_card = cwd.path().join("proc_card.dat");
    std::fs::write(&proc_card, "import model sm\ngenerate e+ e- > mu+ mu-\n").unwrap();
    let run_card = cwd.path().join("run_card.dat");
    std::fs::write(&run_card, fixed_beams(45.6, 45.6)).unwrap();

    let start = Instant::now();
    let output = vibegraph(cwd.path(), home.path())
        .env("VIBEGRAPH_FAULT_WORKER_PANIC", STALL_MS.to_string())
        .arg("--no-network")
        .arg("integrate")
        .arg(&proc_card)
        .arg("--run-card")
        .arg(&run_card)
        .arg("--out")
        .arg(cwd.path().join("out"))
        .args([
            "--fixed-budget",
            "--neval",
            "20000",
            "--niter",
            "2",
            "-j",
            "2",
        ])
        .args(extra)
        .output()
        .expect("spawn vibegraph");
    let elapsed = start.elapsed();

    let stderr = stderr_of(&output);
    assert!(!output.status.success(), "a worker panic must fail the run");
    assert_eq!(output.status.code(), Some(101), "stderr:\n{stderr}");
    assert!(
        stderr.contains("injected worker panic"),
        "the panic's message must reach stderr; got:\n{stderr}"
    );
    assert!(
        elapsed < BOUND,
        "the run outlived its worker's panic by {elapsed:?}; stderr:\n{stderr}"
    );
}

#[test]
fn a_worker_panic_ends_a_text_run_at_once() {
    run_with_injected_panic(&[]);
}

#[test]
fn a_worker_panic_ends_an_ldjson_run_at_once() {
    run_with_injected_panic(&["--output-ldjson"]);
}
