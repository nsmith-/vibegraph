//! The worker count a run takes when `-j` is absent, against the real binary
//! under a CPU affinity mask and `OMP_NUM_THREADS`.
//!
//! Linux only: the mask is set with `taskset`. The cgroup CPU quota is read by
//! the standard library alongside the mask and is not exercised here, since
//! setting one needs privileges a test cannot assume; the unit tests in
//! `parallel.rs` cover how the count is combined with the environment.

#![cfg(target_os = "linux")]

mod common;

use std::process::{Command, Output};

fn run(mask: Option<&str>, env: &[(&str, &str)], extra: &[&str]) -> Output {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    std::fs::write(
        dir.join("proc_card.dat"),
        "import model sm\ngenerate e+ e- > mu+ mu-\n",
    )
    .unwrap();
    std::fs::write(dir.join("run_card.dat"), common::fixed_beams(45.6, 45.6)).unwrap();
    let mut cmd = match mask {
        Some(mask) => {
            let mut cmd = Command::new("taskset");
            cmd.args(["-c", mask, env!("CARGO_BIN_EXE_vibegraph")]);
            cmd
        }
        None => Command::new(env!("CARGO_BIN_EXE_vibegraph")),
    };
    cmd.current_dir(dir)
        .env("VIBEGRAPH_HOME", dir)
        .env_remove("OMP_NUM_THREADS")
        .env_remove("RAYON_NUM_THREADS")
        .envs(env.iter().copied())
        .args(["integrate", "proc_card.dat", "--run-card", "run_card.dat"])
        .args(["--check", "--no-tui"])
        .args(extra);
    let output = cmd.output().expect("spawn (is taskset installed?)");
    assert!(output.status.success(), "{}", common::stderr_of(&output));
    output
}

fn threads_line(output: &Output) -> Option<String> {
    common::stderr_of(output)
        .lines()
        .find(|l| l.contains("threads:"))
        .map(str::to_string)
}

#[test]
fn the_affinity_mask_sets_the_default() {
    let line = threads_line(&run(Some("0"), &[], &[])).expect("a threads line");
    assert!(
        line.contains("threads: 1 (from the CPU affinity mask"),
        "{line}"
    );
    let two = threads_line(&run(Some("0,1"), &[], &[])).expect("a threads line");
    assert!(
        two.contains("threads: 2 (from the CPU affinity mask"),
        "{two}"
    );
}

#[test]
fn omp_num_threads_below_the_mask_wins() {
    let line = threads_line(&run(Some("0,1"), &[("OMP_NUM_THREADS", "1")], &[])).unwrap();
    assert!(line.contains("threads: 1 (from OMP_NUM_THREADS)"), "{line}");
}

#[test]
fn omp_num_threads_above_the_mask_does_not_widen_it() {
    let line = threads_line(&run(Some("0"), &[("OMP_NUM_THREADS", "64")], &[])).unwrap();
    assert!(
        line.contains("threads: 1 (from the CPU affinity mask"),
        "{line}"
    );
}

#[test]
fn an_explicit_count_is_not_reported_as_a_default() {
    let output = run(Some("0"), &[("OMP_NUM_THREADS", "1")], &["-j", "2"]);
    assert_eq!(threads_line(&output), None);
}
