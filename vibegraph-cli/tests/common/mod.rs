//! What the CLI integration tests share: the binary with no route to data a test
//! did not set up, its stderr, the fixed-energy run cards and the MadGraph
//! validation inputs.
//!
//! Each test binary compiles this module whole and calls part of it, so every
//! helper that some binary leaves unused carries its own `allow(dead_code)`; an
//! item added without one is reported by the first binary that does not call it.

use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The binary in `cwd`, with `VIBEGRAPH_HOME` pointed at `home` so `cache_root()`
/// never touches the real `$HOME`, and none of the environment's overrides of
/// the network policy, the PDF directory or the UFO directory.
#[allow(dead_code)]
pub fn vibegraph(cwd: &Path, home: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_vibegraph"));
    cmd.current_dir(cwd)
        .env("VIBEGRAPH_HOME", home)
        .env_remove("VIBEGRAPH_NO_NETWORK")
        .env_remove("VIBEGRAPH_PDF_DIR")
        .env_remove("VIBEGRAPH_UFO_DIR");
    cmd
}

/// [`vibegraph`] with `ALL_PROXY` pointed at `127.0.0.1:1`, where nothing
/// listens, and `NO_PROXY`/`no_proxy` removed. For tests that run under the
/// default network policy and must still never download: a fetch that a refusal
/// failed to stop fails in milliseconds instead of reaching the network.
#[allow(dead_code)]
pub fn vibegraph_behind_dead_proxy(cwd: &Path, home: &Path) -> Command {
    let mut cmd = vibegraph(cwd, home);
    cmd.env("ALL_PROXY", "http://127.0.0.1:1")
        .env_remove("NO_PROXY")
        .env_remove("no_proxy");
    cmd
}

#[allow(dead_code)]
pub fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// The run-card lines for fixed-energy (`lpp = 0`) beams of the given energies;
/// every other parameter keeps MadGraph's default.
#[allow(dead_code)]
pub fn fixed_beams(ebeam1: impl Display, ebeam2: impl Display) -> String {
    format!("  0 = lpp1\n  0 = lpp2\n  {ebeam1} = ebeam1\n  {ebeam2} = ebeam2\n")
}

/// The directory holding MadGraph's cards and references for the Drell-Yan runs.
#[allow(dead_code)]
pub fn validation_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph")
}

/// `integrate` on the Drell-Yan cards from `cwd`, with the cache root at
/// `cache_root`, downloads refused (`VIBEGRAPH_NO_NETWORK=1`) and every other
/// PDF resolution route removed.
#[allow(dead_code)]
pub fn integrate_from_bare_cwd(
    cwd: &Path,
    cache_root: &Path,
    out: &Path,
    extra: &[&str],
) -> Output {
    vibegraph(cwd, cache_root)
        .env("VIBEGRAPH_NO_NETWORK", "1")
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
