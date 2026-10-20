//! What the CLI integration tests share: the binary with no route to data a test
//! did not set up, its stderr, and the fixed-energy run cards.
// Each test binary compiles this module whole and calls part of it.
#![allow(dead_code)]

use std::fmt::Display;
use std::path::Path;
use std::process::{Command, Output};

/// A proxy nothing listens on, so a fetch that a refusal failed to stop fails in
/// milliseconds against `127.0.0.1` instead of reaching the network.
pub const DEAD_PROXY: &str = "http://127.0.0.1:1";

/// The binary in `cwd`, with `VIBEGRAPH_HOME` pointed at `home` so `cache_root()`
/// never touches the real `$HOME`, and none of the environment's overrides of
/// the network policy, the PDF directory or the UFO directory.
pub fn vibegraph(cwd: &Path, home: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_vibegraph"));
    cmd.current_dir(cwd)
        .env("VIBEGRAPH_HOME", home)
        .env_remove("VIBEGRAPH_NO_NETWORK")
        .env_remove("VIBEGRAPH_PDF_DIR")
        .env_remove("VIBEGRAPH_UFO_DIR");
    cmd
}

/// [`vibegraph`] with every HTTP route through [`DEAD_PROXY`], for tests that run
/// under the default network policy and must still never download.
pub fn vibegraph_behind_dead_proxy(cwd: &Path, home: &Path) -> Command {
    let mut cmd = vibegraph(cwd, home);
    cmd.env("ALL_PROXY", DEAD_PROXY)
        .env_remove("NO_PROXY")
        .env_remove("no_proxy");
    cmd
}

pub fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// The run-card lines for fixed-energy (`lpp = 0`) beams of the given energies;
/// every other parameter keeps MadGraph's default.
pub fn fixed_beams(ebeam1: impl Display, ebeam2: impl Display) -> String {
    format!("  0 = lpp1\n  0 = lpp2\n  {ebeam1} = ebeam1\n  {ebeam2} = ebeam2\n")
}
