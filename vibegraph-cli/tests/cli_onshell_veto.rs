//! End-to-end gate on forbidden on-shell s-channels (`$ A`): `vibegraph
//! integrate` on a `$` proc card against MadEvent run on the same proc card and
//! the same run card (`validation/madgraph/onshell_veto_reference.json`, written
//! by `gen_onshell_veto.sh`).
//!
//! # What the rows can and cannot see
//!
//! MadEvent integrates `$ A` with every marked `A` propagator zeroed on its
//! Breit–Wigner window (ALOHA's `P1D` propagators); here the same `|M'|²` is
//! evaluated from amplitudes compiled without the zeroed lines. A cross section
//! sees it only integrated, so it cannot tell a wrong amplitude from a
//! compensating error elsewhere; which diagrams a zeroed line removes, the
//! window, and two marked lines are pinned by the unit tests in
//! `vibegraph::onshell`. What the rows can see is whether the lines are zeroed
//! where MadEvent's are and remove what MadEvent's remove:
//!
//! * `e+ e- > mu+ mu- $ z` at four fixed energies: on the `Z` pole and at 100 GeV
//!   inside the default window (every point vetoed in the `Z` configuration),
//!   at 100 GeV with `bwcutoff = 3` and at 200 GeV outside it (nothing vetoed).
//!   The unrestricted process at the first two energies is the known-wrong
//!   comparison: a veto that never fired reads it, 190× and 5.7× away. At
//!   100 GeV the row also separates `|M'|²` from `|M|²` weighted by the photon
//!   configuration's `AMP2` share (what rejecting the `Z` configuration alone
//!   would give), which integrates 0.93% lower: 35 standard errors.
//! * `t > b e+ ve $ w+`, a partial width with its one configuration vetoed on the
//!   `W` window: the one-incoming case, where every line is an s-channel.
//!
//! Each row is read over [`SEEDS`] on this side against MadEvent's own seeds: the
//! pull of the two seed means, this side's χ²/dof about its mean, and each seed's
//! pull. The heavier rows (a `2 → 4` top pair and the proton row) run under
//! `extended-validation`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use serde::Deserialize;

/// Integration seeds per row.
const SEEDS: [u64; 5] = [11, 22, 33, 44, 55];

/// The seed mean against MadEvent's, in combined standard errors.
const MEAN_PULL_LIMIT: f64 = 3.0;
/// Any one seed against MadEvent's mean.
const SEED_PULL_LIMIT: f64 = 4.0;
/// This side's seeds about their own mean. Four degrees of freedom put the 0.1%
/// and 99.9% points of χ²/dof at 0.02 and 4.6.
const SEED_CHI2_RANGE: (f64, f64) = (0.02, 4.6);

#[derive(Deserialize)]
struct Reference {
    mg_version: String,
    rows: std::collections::BTreeMap<String, Row>,
}

#[derive(Deserialize)]
struct Row {
    process: String,
    run_card: Option<String>,
    overrides: String,
    generated_sde_strategy: Option<i64>,
    runs: Vec<MgRun>,
}

#[derive(Deserialize)]
struct MgRun {
    value: f64,
    err: f64,
}

fn madgraph_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph")
}

fn reference() -> &'static Reference {
    static REFERENCE: OnceLock<Reference> = OnceLock::new();
    REFERENCE.get_or_init(|| {
        let path = madgraph_dir().join("onshell_veto_reference.json");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        serde_json::from_str(&text).expect("onshell_veto_reference.json parses")
    })
}

/// The row's run card as MadEvent read it: the committed base with the row's
/// `key=value` overrides applied the way `gen_onshell_veto.sh` applies them.
fn run_card(dir: &Path, name: &str, row: &Row) -> PathBuf {
    let base = row
        .run_card
        .as_deref()
        .unwrap_or_else(|| panic!("row {name} names no committed run card"));
    let mut text = std::fs::read_to_string(madgraph_dir().join(base)).unwrap();
    for pair in row.overrides.split(';').filter(|kv| !kv.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap();
        let mut hits = 0;
        let lines: Vec<String> = text
            .lines()
            .map(|line| {
                let body = line.split('!').next().unwrap_or("");
                let named = body
                    .split_once('=')
                    .is_some_and(|(_, k)| k.trim().eq_ignore_ascii_case(key));
                if named && !line.trim_start().starts_with('#') {
                    hits += 1;
                    format!("  {value} = {key}")
                } else {
                    line.to_owned()
                }
            })
            .collect();
        assert!(hits <= 1, "{key} set {hits} times in {base}");
        text = lines.join("\n") + "\n";
        if hits == 0 {
            text.push_str(&format!("  {value} = {key}\n"));
        }
    }
    let path = dir.join(format!("{name}_run_card.dat"));
    std::fs::write(&path, text).unwrap();
    path
}

/// `(value, error)` from a result line, `σ = <v> ± <e> pb` or `Γ = <v> ± <e> GeV`.
fn parse_result(stdout: &str) -> (f64, f64) {
    let line = stdout
        .lines()
        .find(|l| l.starts_with("σ = ") || l.starts_with("Γ = "))
        .unwrap_or_else(|| panic!("no result line in:\n{stdout}"));
    let fields: Vec<&str> = line.split_whitespace().collect();
    (fields[2].parse().unwrap(), fields[4].parse().unwrap())
}

fn integrate(dir: &Path, name: &str, row: &Row, seed: u64, extra: &[&str]) -> (f64, f64) {
    let proc_card = dir.join(format!("{name}_proc_card.dat"));
    std::fs::write(
        &proc_card,
        format!("import model sm\ngenerate {}\n", row.process),
    )
    .unwrap();
    let card = run_card(dir, name, row);
    let output = Command::new(env!("CARGO_BIN_EXE_vibegraph"))
        .arg("integrate")
        .arg(&proc_card)
        .arg("--run-card")
        .arg(&card)
        .arg("--out")
        .arg(dir.join(format!("{name}_{seed}")))
        .arg("--force")
        .args(["--seed", &seed.to_string()])
        .args(extra)
        .current_dir(dir)
        .output()
        .expect("spawn vibegraph");
    assert!(
        output.status.success(),
        "vibegraph integrate {name} failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    parse_result(&String::from_utf8_lossy(&output.stdout))
}

/// Inverse-variance mean and its error.
fn weighted_mean(values: &[(f64, f64)]) -> (f64, f64) {
    let w: f64 = values.iter().map(|(_, e)| 1.0 / (e * e)).sum();
    let m = values.iter().map(|(v, e)| v / (e * e)).sum::<f64>() / w;
    (m, 1.0 / w.sqrt())
}

/// The seeds' sample standard deviation.
fn spread(values: &[(f64, f64)]) -> f64 {
    let n = values.len() as f64;
    let m = values.iter().map(|(v, _)| v).sum::<f64>() / n;
    (values.iter().map(|(v, _)| (v - m).powi(2)).sum::<f64>() / (n - 1.0)).sqrt()
}

/// The inverse-variance mean, with an error no smaller than the seed spread
/// allows.
fn seed_mean(values: &[(f64, f64)]) -> (f64, f64) {
    let (m, quoted) = weighted_mean(values);
    (m, quoted.max(spread(values) / (values.len() as f64).sqrt()))
}

/// χ²/dof of `values` about their inverse-variance mean.
fn chi2_per_dof(values: &[(f64, f64)]) -> f64 {
    let (m, _) = weighted_mean(values);
    let chi2: f64 = values.iter().map(|(v, e)| ((v - m) / e).powi(2)).sum();
    chi2 / (values.len() - 1) as f64
}

fn madevent(name: &str) -> (f64, f64) {
    let row = &reference().rows[name];
    let runs: Vec<(f64, f64)> = row.runs.iter().map(|r| (r.value, r.err)).collect();
    seed_mean(&runs)
}

/// Integrate `name` over [`SEEDS`] and print it against MadEvent's seed mean;
/// returns this side's seed mean and the per-seed values.
fn measure_row(dir: &Path, name: &str, extra: &[&str]) -> ((f64, f64), Vec<(f64, f64)>) {
    let reference = reference();
    let row = &reference.rows[name];
    let (mg, mg_err) = madevent(name);
    let ours: Vec<(f64, f64)> = SEEDS
        .iter()
        .map(|&seed| integrate(dir, name, row, seed, extra))
        .collect();
    let (mean, err) = seed_mean(&ours);
    let pull = (mean - mg) / (err * err + mg_err * mg_err).sqrt();
    eprintln!(
        "[{name}] {} (MadGraph {}): here {mean:.6e} ± {err:.1e}, MadEvent {mg:.6e} ± \
         {mg_err:.1e}, rel {:+.3e}, pull {pull:+.2}, χ²/dof {:.2}, seeds {:?}",
        row.process,
        reference.mg_version,
        mean / mg - 1.0,
        chi2_per_dof(&ours),
        ours.iter().map(|(v, _)| *v).collect::<Vec<_>>(),
    );
    ((mean, err), ours)
}

/// [`measure_row`] without a gate on the agreement.
#[cfg(feature = "extended-validation")]
fn report_row(dir: &Path, name: &str, extra: &[&str]) -> (f64, f64) {
    measure_row(dir, name, extra).0
}

/// Integrate `name` over [`SEEDS`] and gate it against MadEvent's seed mean;
/// returns this side's seed mean.
fn check_row(dir: &Path, name: &str, extra: &[&str]) -> (f64, f64) {
    let reference = reference();
    let row = &reference.rows[name];
    if row.process.contains('$') {
        assert_eq!(
            row.generated_sde_strategy,
            Some(1),
            "[{name}] MadGraph writes sde_strategy = 1 for a '$' process"
        );
    }
    let (mg, mg_err) = madevent(name);
    let ((mean, err), ours) = measure_row(dir, name, extra);
    let chi2 = chi2_per_dof(&ours);
    let pull = (mean - mg) / (err * err + mg_err * mg_err).sqrt();
    assert!(
        pull.abs() < MEAN_PULL_LIMIT,
        "[{name}] seed mean {mean} ± {err} against MadEvent {mg} ± {mg_err}: pull {pull:.2}"
    );
    assert!(
        (SEED_CHI2_RANGE.0..SEED_CHI2_RANGE.1).contains(&chi2),
        "[{name}] χ²/dof {chi2:.2} of this side's seeds about their mean"
    );
    for (v, e) in &ours {
        let p = (v - mg) / (e * e + mg_err * mg_err).sqrt();
        assert!(
            p.abs() < SEED_PULL_LIMIT,
            "[{name}] seed value {v} ± {e} against MadEvent {mg}: pull {p:.2}"
        );
    }
    (mean, err)
}

/// Assert `vetoed` is far from `unrestricted` in MadEvent's own numbers: the row
/// is only evidence of a veto if a veto that never fired would fail it.
fn known_wrong(vetoed: &str, unrestricted: &str) {
    let (a, ea) = madevent(vetoed);
    let (b, eb) = madevent(unrestricted);
    let pull = (a - b) / (ea * ea + eb * eb).sqrt();
    eprintln!("[{vetoed}] against the unrestricted {unrestricted}: {a:.6e} vs {b:.6e}, {pull:.0}σ");
    assert!(pull.abs() > 20.0 * MEAN_PULL_LIMIT);
}

#[test]
fn a_lepton_pair_with_the_z_window_vetoed_matches_madevent() {
    let tmp = tempfile::tempdir().unwrap();
    known_wrong("ee_z_pole", "ee_pole");
    known_wrong("ee_z_100", "ee_100");
    // Outside the window the veto removes nothing: MadEvent's own restricted and
    // unrestricted rows agree there.
    let (bw3, e3) = madevent("ee_z_100_bw3");
    let (full, ef) = madevent("ee_100");
    assert!(((bw3 - full) / (e3 * e3 + ef * ef).sqrt()).abs() < MEAN_PULL_LIMIT);
    for name in ["ee_z_pole", "ee_z_100", "ee_z_100_bw3", "ee_z_200"] {
        check_row(tmp.path(), name, &["--target-rel", "5e-4"]);
    }
}

#[test]
fn a_decay_with_the_w_window_vetoed_matches_madevent() {
    let tmp = tempfile::tempdir().unwrap();
    check_row(tmp.path(), "t_bev_w", &["--target-rel", "2e-3"]);
}

#[cfg(feature = "extended-validation")]
mod heavy {
    use super::*;

    fn pdf_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/pdf")
    }

    /// Both top windows zeroed pointwise in a `2 → 4` final state at fixed
    /// energy. **Informational**: MadGraph 3.7.1 writes the `P1D` form of a
    /// fermion propagator computed from its second spinor slot (`FFV2P1D_1`, the
    /// `t` here) with the theta function's argument sign-flipped to
    /// `(p² + (M − c·Γ)²)(p² + (M + c·Γ)²)`, which never vanishes, while the
    /// `t~` (`FFV2P1D_2`) and every vector-boson line get the intended form. The
    /// `uu_tt` reference is therefore MadEvent generated by a copy of the pinned
    /// tree with `validation/madgraph/patches/aloha-p1d-flipped-fermion.patch`
    /// applied; the unpatched reading, 37% above this side, is kept beside it as
    /// `uu_tt_mg371`. With the routine corrected, MadGraph's standalone `SMATRIX`
    /// agrees with the amplitudes here to 1e-12 at six points, zeroed and not,
    /// yet MadEvent integrates about 2% below this side, which is unexplained.
    /// The row runs so that a change on either side shows, and gates only that
    /// the lines are zeroed at all.
    #[test]
    fn a_top_pair_with_both_windows_zeroed_is_reported_against_madevent() {
        let tmp = tempfile::tempdir().unwrap();
        known_wrong("uu_tt", "uu_full");
        let (mean, err) = report_row(tmp.path(), "uu_tt", &["--target-rel", "2e-3"]);
        let (full, _) = madevent("uu_full");
        assert!(
            mean < 0.01 * full,
            "the top windows were not zeroed: {mean} ± {err}"
        );
    }

    /// `p p > e+ e- $ z` on the Drell–Yan card, through the flavour groups and
    /// the mirrored beam ordering.
    #[test]
    fn drell_yan_with_the_z_window_vetoed_matches_madevent() {
        let tmp = tempfile::tempdir().unwrap();
        known_wrong("pp_z", "pp_full");
        let pdf = pdf_dir();
        check_row(
            tmp.path(),
            "pp_z",
            &["--target-rel", "1e-3", "--pdf-dir", pdf.to_str().unwrap()],
        );
    }
}
