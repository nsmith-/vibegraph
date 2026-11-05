//! What the CLI integration tests share: the binary with no route to data a test
//! did not set up (or, for tests that read the ambient configuration, the bare
//! binary), its stderr and stdout, the fixed-energy run cards and the MadGraph
//! validation inputs.
//!
//! Each test binary compiles this module whole and calls part of it, so every
//! helper that some binary leaves unused carries its own `allow(dead_code)`; an
//! item added without one is reported by the first binary that does not call it.

use std::ffi::OsStr;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The binary in `cwd`, with `VIBEGRAPH_HOME` pointed at `home` so `cache_root()`
/// never touches the real `$HOME`, and none of the environment's overrides of
/// the network policy, the PDF data path or the UFO directory.
#[allow(dead_code)]
pub fn vibegraph(cwd: &Path, home: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_vibegraph"));
    cmd.current_dir(cwd)
        .env("VIBEGRAPH_HOME", home)
        .env_remove("VIBEGRAPH_NO_NETWORK")
        .env_remove("LHAPDF_DATA_PATH")
        .env_remove("LHAPATH")
        .env_remove("VIBEGRAPH_PDF_DIR")
        .env_remove("VIBEGRAPH_UFO_DIR");
    cmd
}

/// The binary in the test process's own working directory and environment,
/// untouched. For tests that read the ambient configuration on purpose and name
/// their data explicitly (`--pdf-dir`, `$LHAPDF_DATA_PATH`); a test with no
/// such need uses [`vibegraph`].
#[allow(dead_code)]
pub fn vibegraph_ambient() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vibegraph"))
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

/// Pointing a command's PDF lookup at one directory, the way a user's LHAPDF
/// installation does: `$LHAPDF_DATA_PATH` set to it, and the other variables
/// that could resolve a set (`$LHAPATH`, the retired `$VIBEGRAPH_PDF_DIR`)
/// removed.
#[allow(dead_code)]
pub trait PdfDataPath {
    fn pdf_data_path(&mut self, dir: impl AsRef<OsStr>) -> &mut Self;
}

impl PdfDataPath for Command {
    fn pdf_data_path(&mut self, dir: impl AsRef<OsStr>) -> &mut Self {
        self.env("LHAPDF_DATA_PATH", dir)
            .env_remove("LHAPATH")
            .env_remove("VIBEGRAPH_PDF_DIR")
    }
}

#[allow(dead_code)]
pub fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[allow(dead_code)]
pub fn stdout_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
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

/// A measurement as `integrate` prints it, `σ = <v> ± <e> <prefix>b` or
/// `Γ = <v> ± <e> <prefix>eV`, read back.
#[allow(dead_code)]
#[derive(Debug)]
pub struct ResultLine<'a> {
    /// The whole line the measurement was read from.
    pub line: &'a str,
    /// `σ` or `Γ`.
    pub symbol: &'a str,
    /// The two numbers as printed, in the printed unit.
    pub value_text: &'a str,
    pub error_text: &'a str,
    /// The printed unit, prefix included (`ab`, `eV`).
    pub unit: &'a str,
    /// The value and its error in the unit the artifact records: pb for a cross
    /// section, GeV for a width.
    pub value: f64,
    pub error: f64,
}

/// The first result line of `stdout`. Panics, naming the line, on any other
/// shape or on a unit that is not an SI-prefixed barn (σ) or electronvolt (Γ).
#[allow(dead_code)]
pub fn result_line(stdout: &str) -> ResultLine<'_> {
    let line = stdout
        .lines()
        .find(|l| l.starts_with("σ = ") || l.starts_with("Γ = "))
        .unwrap_or_else(|| panic!("no result line in:\n{stdout}"));
    measurement(line, line)
}

/// The cross section a mixed-multiplicity run's summary gives the part with
/// `n_out` outgoing legs, `N outgoing legs (…): σ = <v> ± <e> <prefix>b over M
/// channels`, read back as [`result_line`] reads the total.
#[allow(dead_code)]
pub fn part_line(stderr: &str, n_out: usize) -> ResultLine<'_> {
    let line = stderr
        .lines()
        .find(|l| l.contains(&format!(" {n_out} outgoing legs (@")))
        .unwrap_or_else(|| panic!("no {n_out}-leg part line in:\n{stderr}"));
    let start = line
        .find("σ = ")
        .unwrap_or_else(|| panic!("{line:?} has no σ"));
    let end = line
        .rfind(" over ")
        .unwrap_or_else(|| panic!("{line:?} has no channel count"));
    measurement(line, &line[start..end])
}

/// `text`, `<symbol> = <value> ± <error> <unit>`, read off `line`.
fn measurement<'a>(line: &'a str, text: &'a str) -> ResultLine<'a> {
    let fields: Vec<&str> = text.split_whitespace().collect();
    let [symbol, "=", value_text, "±", error_text, unit] = fields[..] else {
        panic!("{line:?} is not `<symbol> = <value> ± <error> <unit>`");
    };
    // The exponent of the unit artifacts record (pb = 1e-12 b, GeV = 1e9 eV).
    let (base, recorded_exp) = match symbol {
        "σ" => ("b", -12),
        _ => ("eV", 9),
    };
    let prefix = unit
        .strip_suffix(base)
        .unwrap_or_else(|| panic!("{line:?}: {symbol} is not in a unit of {base}"));
    let prefix_exp = match prefix {
        "z" => -21,
        "a" => -18,
        "f" => -15,
        "p" => -12,
        "n" => -9,
        "\u{b5}" => -6,
        "m" => -3,
        "" => 0,
        "k" => 3,
        "M" => 6,
        "G" => 9,
        "T" => 12,
        other => panic!("{line:?}: unknown SI prefix {other:?}"),
    };
    // One multiplication or division by an exact power of ten.
    let shift = prefix_exp - recorded_exp;
    let in_recorded = |text: &str| -> f64 {
        let x: f64 = text
            .parse()
            .unwrap_or_else(|_| panic!("{line:?}: {text:?} is not a number"));
        if shift >= 0 {
            x * 10f64.powi(shift)
        } else {
            x / 10f64.powi(-shift)
        }
    };
    ResultLine {
        line,
        symbol,
        value_text,
        error_text,
        unit,
        value: in_recorded(value_text),
        error: in_recorded(error_text),
    }
}
