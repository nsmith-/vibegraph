//! End-to-end gate on `vibegraph generate`: a converged integration, replayed
//! into an unweighted Les Houches file, under each weight strategy.
//!
//! Needs no PDF set and no banked MadGraph data — only the interned SM model — so
//! it runs in the default test suite. One `vibegraph integrate` run produces the
//! artifact every case below replays, so the grids, the channel weights and the
//! cross section are shared and the only thing varying is the sampling phase.
//!
//! # What this gate checks, and what it provably cannot
//!
//! It reads the emitted files back with this crate's own `lhef::parse`. Our reader
//! and our writer share their assumptions, so a **self-consistently wrong format**
//! is invisible here: a file both agree on but a shower rejects would pass. What
//! carries the format evidence is `validate_lhef`'s byte-for-byte round trip of
//! MadGraph's own banked event files, and — still owed — a run through a real
//! shower.
//!
//! It is also blind to everything the integrand gets wrong: a wrong matrix
//! element, cut or sampler is replayed faithfully into the file and agrees with
//! itself. `amplitude_oracle`, `validate_sigma` and `validate_unweighting` cover
//! those. And it says nothing about an event's discrete labels — helicity and
//! colour-flow selection move no weight, so a mislabelled event is invisible to
//! every comparison here (`color_flow_tags_oracle` and `validate_lhef` cover them).
//!
//! Finally, the rounding *rule* is not pinned here. Almost every event of this
//! sample carries weight exactly `1`, where every plausible rounding rule agrees;
//! the rule is pinned on controlled weights by the `lhef::emit` unit tests.
//!
//! # Why the two strategies are compared on *different* seeds
//!
//! Given the same seed the source hands both strategies the identical accepted
//! events, and the stochastic-rounding file is then the buffered one with a few
//! duplicates — comparing them would be very nearly a tautology. The statistical
//! comparison below therefore runs them on independent seeds, so agreement is
//! evidence that the two randomisations describe the same distribution rather than
//! evidence that they share a stream. Reproducibility is checked separately, where
//! sharing a seed is exactly the point.

mod common;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use vibegraph::artifact::IntegrateArtifact;
use vibegraph::lhef::emit::sample_estimate_in;
use vibegraph::lhef::parse::LheFile;
use vibegraph::lhef::record::{LheEvent, WeightStrategy, STATUS_INCOMING, STATUS_OUTGOING};

/// A process with a resonance in reach of the beams, so the multichannel sampler
/// has something to resolve and the artifact banks more than one channel.
const PROCESS: &str = "e+ e- > mu+ mu-";
const EBEAM: f64 = 45.6;

const NEVAL: &str = "20000";
const NITER: &str = "4";
/// Events in the σ comparison. The cross section an accept/reject sample recovers
/// has a relative error of at most `1/√N`, so this sets its resolution at ~0.7%.
const NEVENTS: usize = 20_000;
/// Events where only the file's structure is under test.
const FEW_EVENTS: usize = 600;

const SEED_A: &str = "20260728";
const SEED_B: &str = "981221";

/// How far the sample's σ may sit from the integration's, in units of `1/√N`.
const SIGMA_LIMIT_IN_MC_ERRORS: f64 = 4.0;
/// Shape comparison between the two strategies.
const NBINS: usize = 10;
const MIN_BIN_EVENTS: usize = 50;
const SHAPE_CHI2_LIMIT: f64 = 3.0;
const SHAPE_PULL_LIMIT: f64 = 4.0;

struct Run {
    _tmp: tempfile::TempDir,
    dir: PathBuf,
    artifact_path: PathBuf,
    artifact: IntegrateArtifact,
}

fn write_run_card(path: &Path, ebeam1: f64, extra: &str) {
    std::fs::write(
        path,
        format!("{}{extra}", common::fixed_beams(ebeam1, EBEAM)),
    )
    .unwrap();
}

/// The one `vibegraph integrate` run every case in this file replays.
fn run() -> &'static Run {
    static RUN: OnceLock<Run> = OnceLock::new();
    RUN.get_or_init(integrate)
}

fn integrate() -> Run {
    integrate_with("")
}

/// The integration of the same card with `event_norm` set to `norm`, run once
/// per setting for every case that needs it. `average` is the card default, so
/// it is [`run`]'s own.
fn run_with_event_norm(norm: &str) -> &'static Run {
    static SUM: OnceLock<Run> = OnceLock::new();
    static UNITY: OnceLock<Run> = OnceLock::new();
    match norm {
        "average" => run(),
        "sum" => SUM.get_or_init(|| integrate_with("  sum = event_norm\n")),
        "unity" => UNITY.get_or_init(|| integrate_with("  unity = event_norm\n")),
        other => panic!("no such event_norm: {other}"),
    }
}

/// An integration whose run card carries `extra` card lines after the beams.
fn integrate_with(extra: &str) -> Run {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().to_path_buf();
    let proc_card = dir.join("proc_card.dat");
    let run_card = dir.join("run_card.dat");
    std::fs::write(&proc_card, format!("import model sm\ngenerate {PROCESS}\n")).unwrap();
    write_run_card(&run_card, EBEAM, extra);

    let out = dir.join("out");
    let status = common::vibegraph_ambient()
        .arg("integrate")
        .arg(&proc_card)
        .arg("--run-card")
        .arg(&run_card)
        .arg("--out")
        .arg(&out)
        .args(["--neval", NEVAL, "--niter", NITER])
        .status()
        .expect("spawn vibegraph");
    assert!(status.success(), "vibegraph integrate exited non-zero");

    let artifact_path = out.join("grid.bin.zst");
    let artifact = IntegrateArtifact::read_from_path(&artifact_path).expect("reload artifact");
    assert_eq!(artifact.process, PROCESS);
    Run {
        _tmp: tmp,
        dir,
        artifact_path,
        artifact,
    }
}

impl Run {
    fn generate_cmd(&self, strategy: &str, seed: &str, nevents: usize, output: &Path) -> Command {
        let mut cmd = common::vibegraph_ambient();
        cmd.arg("generate")
            .arg(&self.artifact_path)
            .arg("--strategy")
            .arg(strategy)
            .arg("--seed")
            .arg(seed)
            .arg("--nevents")
            .arg(nevents.to_string())
            .arg("-o")
            .arg(output);
        cmd
    }

    /// Generate into `name` and read the file back with our own parser.
    fn generate(
        &self,
        strategy: &str,
        seed: &str,
        nevents: usize,
        name: &str,
    ) -> (LheFile, String) {
        let output = self.dir.join(name);
        let out = self
            .generate_cmd(strategy, seed, nevents, &output)
            .output()
            .expect("spawn vibegraph");
        assert!(
            out.status.success(),
            "vibegraph generate ({strategy}) failed:\n{}",
            common::stderr_of(&out)
        );
        eprint!("{}", common::stdout_of(&out));
        let text = std::fs::read_to_string(&output).expect("read the event file");
        let file = LheFile::parse(&text).expect("our own file parses");
        (file, text)
    }

    /// Generate with extra flags appended, returning the file's bytes and the
    /// run's notices, which the command reports on stderr.
    fn generate_with(&self, extra: &[&str], seed: &str, name: &str) -> (String, String) {
        let output = self.dir.join(name);
        let mut cmd = self.generate_cmd("buffer", seed, FEW_EVENTS, &output);
        cmd.args(extra);
        let out = cmd.output().expect("spawn vibegraph");
        let notices = common::stderr_of(&out);
        assert!(
            out.status.success(),
            "vibegraph generate {extra:?} failed:\n{notices}"
        );
        eprint!("{notices}");
        (
            std::fs::read_to_string(&output).expect("read the event file"),
            notices,
        )
    }

    /// `generate` on `artifact` with only the flags that every run needs, and
    /// `extra` after them.
    fn generate_on(&self, artifact: &Path, extra: &[&str], name: &str) -> Output {
        common::vibegraph_ambient()
            .arg("generate")
            .arg(artifact)
            .args(["--seed", SEED_A, "--nevents", "10", "-o"])
            .arg(self.dir.join(name))
            .args(extra)
            .output()
            .expect("spawn vibegraph")
    }

    /// A copy of this run's artifact, changed by `edit` and written to `name`.
    fn edited_artifact(&self, name: &str, edit: impl FnOnce(&mut IntegrateArtifact)) -> PathBuf {
        let mut artifact = self.artifact.clone();
        edit(&mut artifact);
        let path = self.dir.join(name);
        artifact
            .write_to_path(&path, true)
            .expect("write the edited artifact");
        path
    }
}

/// `cosθ` of the first outgoing leg against the beam axis — the only shape a
/// fixed-energy `2 → 2` sample has.
fn cos_theta(event: &LheEvent) -> f64 {
    let [_, px, py, pz] = event.particles[2].momentum;
    let p = (px * px + py * py + pz * pz).sqrt();
    if p > 0.0 {
        (pz / p).clamp(-1.0, 1.0)
    } else {
        0.0
    }
}

/// The cross section each written event carries, in picobarns, whichever weight
/// convention the file declares.
fn event_content(file: &LheFile) -> Vec<f64> {
    let n = file.events.len() as f64;
    match file.init.weight_strategy {
        // σ is the mean of the weights, so an event's share of it is its weight
        // over the sample size.
        WeightStrategy::MeanCrossSectionPb => file.events.iter().map(|e| e.weight / n).collect(),
        // Unit weights: σ is `XSECUP`, split evenly.
        WeightStrategy::UnitWeight => vec![file.init.processes[0].xsec_pb / n; file.events.len()],
        other => panic!("unexpected IDWTUP {other:?}"),
    }
}

/// `(σ, Δσ, count)` per bin of `cosθ`, the per-event contents summed in quadrature
/// for the error.
///
/// That error is the sample's own, and under stochastic rounding it is a slight
/// under-estimate: an event written twice contributes two perfectly correlated
/// entries. The duplicated fraction on this process is of order `1e-4`, far below
/// the band the comparison uses.
fn binned(file: &LheFile) -> (Vec<f64>, Vec<f64>, Vec<usize>) {
    let content = event_content(file);
    let mut sigma = vec![0.0; NBINS];
    let mut var = [0.0; NBINS];
    let mut counts = vec![0usize; NBINS];
    for (event, c) in file.events.iter().zip(&content) {
        let bin = ((((cos_theta(event) + 1.0) / 2.0) * NBINS as f64) as usize).min(NBINS - 1);
        sigma[bin] += c;
        var[bin] += c * c;
        counts[bin] += 1;
    }
    (sigma, var.iter().map(|v| v.sqrt()).collect(), counts)
}

/// Every event of a fixed-beam `2 → 2` record has the same shape, and a file whose
/// events do not is not one a shower can read.
fn check_record_shape(file: &LheFile, n_ext: usize, n_in: usize) {
    assert_eq!(file.init.processes.len(), 1);
    for (index, event) in file.events.iter().enumerate() {
        assert_eq!(event.nup(), n_ext, "event {index}: NUP");
        assert_eq!(
            event.process_id, file.init.processes[0].id,
            "event {index}: IDPRUP"
        );
        for (leg, particle) in event.particles.iter().enumerate() {
            let incoming = leg < n_in;
            assert_eq!(
                particle.status,
                if incoming {
                    STATUS_INCOMING
                } else {
                    STATUS_OUTGOING
                },
                "event {index} leg {leg}: ISTUP"
            );
        }
        assert!(event.scale > 0.0, "event {index}: SCALUP");
        assert!(event.weight > 0.0, "event {index}: XWGTUP");
        assert!(event.alpha_qed > 0.0, "event {index}: AQEDUP");
    }
}

#[test]
fn generate_writes_a_sample_that_reproduces_the_integrated_cross_section() {
    let run = run();
    let sigma = run.artifact.sigma_pb;
    assert!(sigma > 0.0);
    // The σ band: the accept/reject sample's own `1/√N`, plus the integration's.
    let band =
        SIGMA_LIMIT_IN_MC_ERRORS / (NEVENTS as f64).sqrt() + run.artifact.sigma_err_pb / sigma;

    let (buffered, buffered_text) = run.generate("buffer", SEED_A, NEVENTS, "buffered.lhe");
    let (rounded, _) = run.generate("stochastic-rounding", SEED_B, NEVENTS, "rounded.lhe");
    check_record_shape(&buffered, 4, 2);
    check_record_shape(&rounded, 4, 2);

    // -- buffered: IDWTUP = -4, weights in pb, σ their mean --------------------
    assert_eq!(
        buffered.init.weight_strategy,
        WeightStrategy::MeanCrossSectionPb
    );
    assert_eq!(buffered.events.len(), NEVENTS);
    let mean: f64 =
        buffered.events.iter().map(|e| e.weight).sum::<f64>() / buffered.events.len() as f64;
    let declared = buffered.init.processes[0].xsec_pb;
    assert!(
        (mean / declared - 1.0).abs() < 1e-6,
        "IDWTUP = -4 promises sigma is the mean weight: {mean:.6e} vs XSECUP {declared:.6e}"
    );
    let largest = buffered.events.iter().map(|e| e.weight).fold(0.0, f64::max);
    assert!(
        buffered.init.processes[0].xmax >= largest * (1.0 - 1e-6),
        "XMAXUP {:.6e} does not bound the largest weight written {largest:.6e}",
        buffered.init.processes[0].xmax
    );
    // The writer normalises the file to the integration, so the mean weight is
    // the integration's σ by construction; what measures the accept/reject pass
    // is the sample's own estimate before that, which the header records.
    assert!(
        (declared / sigma - 1.0).abs() < 1e-6,
        "XSECUP {declared:.6e} is not the integration's {sigma:.6e}"
    );
    let (sample, sample_err) =
        sample_estimate_in(&buffered_text).expect("the header records the sample's estimate");
    let deviation = sample / sigma - 1.0;
    eprintln!(
        "buffered:  sigma(sample, before normalisation) = {sample:.6e} ± {sample_err:.2e} pb vs \
         integration {sigma:.6e} +- {:.2e} pb ({:+.3}%, band +-{:.3}%)",
        run.artifact.sigma_err_pb,
        100.0 * deviation,
        100.0 * band
    );
    assert!(
        deviation.abs() < band,
        "the buffered sample's own sigma is {:+.3}% off the integration",
        100.0 * deviation
    );

    // -- stochastic rounding: IDWTUP = +3, unit weights ------------------------
    //
    // `XSECUP` is carried from the integration, so recovering σ from this file is
    // exact by construction — that is the error class this half cannot detect, and
    // it is why the buffered file above is where the σ comparison has teeth. What
    // this half must show is that unit weights did not cost the sample its shape.
    assert_eq!(rounded.init.weight_strategy, WeightStrategy::UnitWeight);
    assert_eq!(rounded.init.processes[0].xmax, 1.0);
    assert!(rounded.events.iter().all(|e| e.weight == 1.0));
    assert!(
        rounded.events.len() >= NEVENTS,
        "an event's copies are never truncated, so the file is never short"
    );
    let declared_rounded = rounded.init.processes[0].xsec_pb;
    assert!(
        (declared_rounded / sigma - 1.0).abs() < 1e-6,
        "XSECUP {declared_rounded:.6e} is not the integration's {sigma:.6e}"
    );

    // -- the two strategies describe the same distribution ---------------------
    let (sigma_b, err_b, counts_b) = binned(&buffered);
    let (sigma_r, err_r, counts_r) = binned(&rounded);
    let total_b: f64 = sigma_b.iter().sum();
    let total_r: f64 = sigma_r.iter().sum();
    let mut chi2 = 0.0;
    let mut dof = 0usize;
    let mut worst = 0.0f64;
    for bin in 0..NBINS {
        if counts_b[bin] < MIN_BIN_EVENTS || counts_r[bin] < MIN_BIN_EVENTS {
            continue;
        }
        let combined = (err_b[bin] * err_b[bin] + err_r[bin] * err_r[bin]).sqrt();
        let pull = (sigma_b[bin] - sigma_r[bin]) / combined;
        chi2 += pull * pull;
        dof += 1;
        worst = worst.max(pull.abs());
    }
    assert!(
        dof >= NBINS / 2,
        "too few populated bins ({dof}) to compare"
    );
    let chi2_dof = chi2 / dof as f64;
    eprintln!(
        "shape:     sigma(buffered) = {total_b:.6e} pb, sigma(rounded) = {total_r:.6e} pb; \
         cos(theta) chi2/dof = {chi2_dof:.3} over {dof} bins, worst pull {worst:.2}"
    );
    assert!(
        chi2_dof < SHAPE_CHI2_LIMIT && worst < SHAPE_PULL_LIMIT,
        "the two strategies disagree on the cos(theta) shape"
    );
    // Both totals are the integration's by construction — the buffered file is
    // normalised to it and the rounded one carries it in `XSECUP` — so the
    // strategies are compared on the shape above, not on the total.
    assert!((total_b / sigma - 1.0).abs() < 1e-6 && (total_r / sigma - 1.0).abs() < 1e-6);
}

/// Same seed, same file; a different seed, a different one. Both strategies.
#[test]
fn a_seed_reproduces_the_sample() {
    let run = run();
    for strategy in ["buffer", "stochastic-rounding"] {
        let (_, first) = run.generate(strategy, SEED_A, FEW_EVENTS, &format!("{strategy}-1.lhe"));
        let (_, again) = run.generate(strategy, SEED_A, FEW_EVENTS, &format!("{strategy}-2.lhe"));
        // The header carries the artifact path and the seed, identical here, so
        // the whole file must be.
        assert_eq!(first, again, "{strategy} is not reproducible from its seed");
        let (_, other) = run.generate(strategy, SEED_B, FEW_EVENTS, &format!("{strategy}-3.lhe"));
        assert_ne!(first, other, "{strategy} ignores its seed");
    }
}

/// The summed maximum a run reports, out of its `scan:` notice.
fn sum_w_max(notices: &str) -> f64 {
    notices
        .lines()
        .find_map(|l| l.split("scan:").nth(1))
        .and_then(|l| l.split("sum w_max ").nth(1))
        .and_then(|l| l.split(',').next())
        .and_then(|v| v.trim().parse::<f64>().ok())
        .unwrap_or_else(|| panic!("no scan line in:\n{notices}"))
}

/// The `w_max` scan's budget is a knob of its own, and its default is the
/// integration's per-channel allocation — so the default run and an explicit
/// `--scan-points share` must be the *same bytes*, and a scan on a different
/// budget must be a different sample.
///
/// The negative control matters more than the equality here: the maxima only
/// enter the file through the channel-selection weights and the acceptance
/// probability, so a flag that silently did nothing would pass the first
/// assertion and every other test in this file. It is taken under
/// `--max-truncation 0`, where a shorter scan provably cannot find a larger
/// maximum in any channel; the truncating rule reads a quantile of the scan
/// instead of its extremum, and a quantile is not monotone in the sample size.
#[test]
fn the_scan_budget_defaults_to_the_integration_allocation_and_is_otherwise_live() {
    let run = run();
    let (default_file, _) = run.generate_with(&[], SEED_A, "scan-default.lhe");
    let (share_file, _) = run.generate_with(&["--scan-points", "share"], SEED_A, "scan-share.lhe");
    assert_eq!(
        default_file, share_file,
        "`--scan-points share` must be the default, byte for byte"
    );

    let (other_file, _) = run.generate_with(&["--scan-points", "250"], SEED_A, "scan-250.lhe");
    assert_ne!(
        default_file, other_file,
        "a different scan budget produced an identical sample: the flag is inert"
    );

    let (_, long_out) = run.generate_with(&["--max-truncation", "0"], SEED_A, "scan-extremum.lhe");
    let (_, short_out) = run.generate_with(
        &["--max-truncation", "0", "--scan-points", "250"],
        SEED_A,
        "scan-extremum-250.lhe",
    );
    assert!(
        sum_w_max(&short_out) < sum_w_max(&long_out),
        "250 points per channel gave sum w_max {:.6e}, the integration's own allocation {:.6e}",
        sum_w_max(&short_out),
        sum_w_max(&long_out)
    );
}

/// The rule that reads the maxima off the same scan is the other knob, and its
/// default is MadGraph's 1% truncation.
///
/// Allowing a larger share above the maxima can only walk each channel's ladder
/// further down, so the summed maximum is non-increasing in the share and strictly
/// below the extremum the same scan saw — the direction that says the flag reached
/// the maxima rather than some other part of the run. The maxima set both the
/// channel-selection weights and the acceptance probability, so a rule that
/// changed nothing would leave the file byte-identical.
#[test]
fn the_maximum_rule_defaults_to_madgraphs_truncation_and_is_otherwise_live() {
    let run = run();
    let (default_file, default_out) = run.generate_with(&[], SEED_A, "trunc-default.lhe");
    let (same_file, _) = run.generate_with(&["--max-truncation", "0.01"], SEED_A, "trunc-1pct.lhe");
    assert_eq!(
        default_file, same_file,
        "`--max-truncation 0.01` must be the default, byte for byte"
    );

    let (extremum_file, extremum_out) =
        run.generate_with(&["--max-truncation", "0"], SEED_A, "trunc-0.lhe");
    assert_ne!(
        default_file, extremum_file,
        "the extremum rule produced an identical sample: the flag is inert"
    );

    let (_, wide_out) = run.generate_with(&["--max-truncation", "0.05"], SEED_A, "trunc-5pct.lhe");
    let (extremum, default, wide) = (
        sum_w_max(&extremum_out),
        sum_w_max(&default_out),
        sum_w_max(&wide_out),
    );
    assert!(
        wide <= default && default < extremum,
        "sum w_max must fall as the allowed share grows: \
         5% {wide:.6e}, 1% {default:.6e}, extremum {extremum:.6e}"
    );
}

/// A truncation share outside `[0, 1)` is a refusal: at one and above, the whole
/// scanned cross section is allowed above the maximum and the ladder has no rung
/// left to stand on.
#[test]
fn a_truncation_share_outside_the_unit_interval_is_refused() {
    let run = run();
    for bad in ["1", "1.5", "-0.01", "most"] {
        let mut cmd = run.generate_cmd("buffer", SEED_A, 10, &run.dir.join("never.lhe"));
        // `=` keeps a negative share a value, so clap hands it to the share's
        // own parser rather than reading it as an unknown flag.
        cmd.arg(format!("--max-truncation={bad}"));
        expect_refusal(
            cmd.output().expect("spawn vibegraph"),
            &format!("--max-truncation {bad}"),
            &format!("invalid value '{bad}' for '--max-truncation"),
        );
    }
}

/// The per-channel scans run in parallel, and each is a function of its own two
/// streams and of nothing another channel touches — so the maxima, and with them
/// every accepted event, must not depend on how many threads found them.
#[test]
fn the_scan_does_not_depend_on_the_thread_count() {
    let run = run();
    let (one, _) = run.generate_with(&["-j", "1"], SEED_A, "threads-1.lhe");
    let (many, _) = run.generate_with(&["-j", "16"], SEED_A, "threads-16.lhe");
    assert_eq!(
        one, many,
        "`generate -j 1` and `-j 16` wrote different samples"
    );
}

/// A scan budget that is not a positive count is a refusal, not a fallback: the
/// maxima it would produce set every event weight in the file.
#[test]
fn a_scan_budget_that_is_not_a_positive_count_is_refused() {
    let run = run();
    for bad in ["0", "half", "-5"] {
        let mut cmd = run.generate_cmd("buffer", SEED_A, 10, &run.dir.join("never.lhe"));
        cmd.arg(format!("--scan-points={bad}"));
        expect_refusal(
            cmd.output().expect("spawn vibegraph"),
            &format!("--scan-points {bad}"),
            &format!("invalid value '{bad}' for '--scan-points"),
        );
    }
}

/// `out` is a refusal of `because` that says so: a non-zero exit whose stderr
/// carries `names` — the flag, the field or the condition refused — and no
/// panic, which would also exit non-zero without refusing anything.
#[track_caller]
fn expect_refusal(out: Output, because: &str, names: &str) {
    let stderr = common::stderr_of(&out);
    assert!(
        !out.status.success(),
        "generate accepted {because}; it must refuse\nstdout: {}",
        common::stdout_of(&out)
    );
    assert!(
        !stderr.contains("panicked"),
        "generate panicked on {because} instead of refusing it:\n{stderr}"
    );
    assert!(
        stderr.contains(names),
        "the refusal of {because} does not say {names:?}:\n{stderr}"
    );
}

/// `generate` takes no card and no PDF choice: the artifact recorded both. A
/// proc card, a run card, a PDF set or a PDF directory handed to it is an
/// argument error, refused before anything is read.
#[test]
fn a_card_or_a_pdf_choice_passed_to_generate_is_an_argument_error() {
    let run = run();
    let proc_card = run.dir.join("proc_card.dat");
    let run_card = run.dir.join("run_card.dat");
    let cases: [(&str, Vec<String>); 4] = [
        (
            "a positional proc card",
            vec![proc_card.display().to_string()],
        ),
        (
            "--run-card",
            vec!["--run-card".to_string(), run_card.display().to_string()],
        ),
        (
            "--pdf-set",
            vec![
                "--pdf-set".to_string(),
                "NNPDF23_lo_as_0130_qed".to_string(),
            ],
        ),
        (
            "--pdf-dir",
            vec!["--pdf-dir".to_string(), run.dir.display().to_string()],
        ),
    ];
    for (what, extra) in cases {
        let extra: Vec<&str> = extra.iter().map(String::as_str).collect();
        let out = run.generate_on(&run.artifact_path, &extra, "never.lhe");
        expect_refusal(out, what, "unexpected argument");
        assert!(!run.dir.join("never.lhe").exists(), "{what} wrote a file");
    }
}

/// A missing `--seed` is refused, saying why: two jobs on one grid under one
/// seed would write the same events.
#[test]
fn a_missing_seed_is_refused() {
    let run = run();
    let out = common::vibegraph_ambient()
        .arg("generate")
        .arg(&run.artifact_path)
        .args(["--nevents", "10", "-o"])
        .arg(run.dir.join("unseeded.lhe"))
        .output()
        .expect("spawn vibegraph");
    expect_refusal(
        out,
        "a generation without --seed",
        "each generation job needs its own seed",
    );
    assert!(!run.dir.join("unseeded.lhe").exists());
}

/// The recorded proc card is parsed again and must spell the recorded
/// process. The artifact is `run()`'s own with only the card's text changed,
/// as a build that parsed the card differently would see it.
#[test]
fn a_recorded_proc_card_that_parses_to_another_process_is_refused() {
    let run = run();
    let edited = run.edited_artifact("reparsed.bin.zst", |a| {
        a.proc_card = "import model sm\ngenerate e+ e- > ta+ ta-\n".to_string();
    });
    let out = run.generate_on(&edited, &[], "never.lhe");
    let stderr = common::stderr_of(&out);
    expect_refusal(
        out,
        "a recorded proc card for another process",
        "parses to `e+ e- > ta+ ta-`",
    );
    assert!(stderr.contains("re-integrate"), "{stderr}");
}

/// The model the recorded proc card resolves to is checked by digest against
/// the artifact's, the interned SM included. The artifact is `run()`'s own
/// with only the recorded digest changed, so the digest is all that can refuse
/// it.
#[test]
fn a_model_whose_digest_moved_is_refused() {
    let run = run();
    let edited = run.edited_artifact("moved-model.bin.zst", |a| {
        a.model.digest = vibegraph::ufo::identity::digest_bytes(b"another model");
    });
    let out = run.generate_on(&edited, &[], "never.lhe");
    let stderr = common::stderr_of(&out);
    expect_refusal(
        out,
        "a model whose digest moved",
        "model sm-default at (built in) has digest",
    );
    assert!(
        stderr.contains("its files changed after vibegraph integrate ran"),
        "{stderr}"
    );
}

/// An existing output file is not clobbered.
#[test]
fn an_existing_event_file_is_left_alone() {
    let run = run();
    let (_, written) = run.generate("buffer", SEED_A, FEW_EVENTS, "existing.lhe");
    expect_refusal(
        run.generate_cmd("buffer", SEED_A, 10, &run.dir.join("existing.lhe"))
            .output()
            .expect("spawn"),
        "an existing output file without --force",
        "already exists (pass --force to overwrite)",
    );
    let untouched = std::fs::read_to_string(run.dir.join("existing.lhe")).expect("read back");
    assert_eq!(
        untouched, written,
        "the refused run must leave the file alone"
    );
}

/// An artifact written before version 13 records no proc card for a
/// generation to replay, so `generate` refuses it by its version and says to
/// re-integrate. The file is `run()`'s own payload with its version stamped
/// down to 12: everything after the version decodes, so only the version can be
/// what refuses it.
#[test]
fn a_version_12_artifact_is_refused() {
    let run = run();
    let mut raw = zstd::decode_all(
        std::fs::read(&run.artifact_path)
            .expect("read the artifact")
            .as_slice(),
    )
    .expect("decompress the artifact");
    raw[..4].copy_from_slice(&12u32.to_le_bytes());
    let stale = run.dir.join("v12.bin.zst");
    std::fs::write(&stale, zstd::encode_all(raw.as_slice(), 19).unwrap()).unwrap();

    let out = run.generate_on(&stale, &[], "never.lhe");
    let stderr = common::stderr_of(&out);
    expect_refusal(out, "a version-12 artifact", "artifact format version 12");
    assert!(
        stderr.contains("regenerate it with `vibegraph integrate`"),
        "the refusal does not say to re-integrate:\n{stderr}"
    );
}

/// One sample per `event_norm`, each from an integration of the same card plus
/// that line (`average` being the card default): the written weights follow MadGraph's rule for the setting,
/// and nothing else about the run moves.
///
/// MadGraph's rule is `MultiEventFile.unweight` (`lhe_parser.py:1225-1239`) and
/// `EventFile.unweight` (`lhe_parser.py:519-527`): `sum` writes `IDWTUP = -3` and
/// weights σ/N, `average` `IDWTUP = -4` and weights σ, `unity` `IDWTUP = -3` and
/// weights 1. This file keeps an overweight event above the rest where
/// MadGraph's flattens it, so the weights are compared as ratios to the
/// `average` sample's own rather than as constants.
///
/// What this cannot see: it reads the files with this crate's own parser, so a
/// weight convention both ends share is invisible; the MadGraph rule it pins
/// is a reading of the source, not a banked MadGraph file written under `sum`.
#[test]
fn event_norm_changes_the_written_weights_and_nothing_else() {
    let norms = ["average", "sum", "unity"];
    let runs: Vec<&Run> = norms.iter().map(|norm| run_with_event_norm(norm)).collect();
    let files: Vec<(LheFile, String)> = runs
        .iter()
        .zip(norms)
        .map(|(run, norm)| run.generate("buffer", SEED_A, FEW_EVENTS, &format!("{norm}.lhe")))
        .collect();
    let (average, sum, unity) = (&files[0].0, &files[1].0, &files[2].0);
    let n = FEW_EVENTS as f64;

    let sigma = runs[0].artifact.sigma_pb;
    for run in &runs {
        assert_eq!(
            run.artifact.sigma_pb, sigma,
            "the integration moved with event_norm"
        );
    }

    assert_eq!(
        average.init.weight_strategy,
        WeightStrategy::MeanCrossSectionPb
    );
    assert_eq!(sum.init.weight_strategy, WeightStrategy::SumCrossSectionPb);
    assert_eq!(
        unity.init.weight_strategy,
        WeightStrategy::SumCrossSectionPb
    );
    for file in [average, sum, unity] {
        let declared = file.init.processes[0].xsec_pb;
        assert!(
            (declared / sigma - 1.0).abs() < 1e-6,
            "XSECUP {declared:.6e} is not the integration's {sigma:.6e}"
        );
        check_record_shape(file, 4, 2);
    }

    let mean: f64 = average.events.iter().map(|e| e.weight).sum::<f64>() / n;
    let total: f64 = sum.events.iter().map(|e| e.weight).sum();
    assert!((mean / sigma - 1.0).abs() < 1e-6, "average: {mean:.6e}");
    assert!((total / sigma - 1.0).abs() < 1e-6, "sum: {total:.6e}");
    assert!(unity.events.iter().all(|e| e.weight == 1.0));
    assert_eq!(unity.init.processes[0].xmax, 1.0);

    // The same seed draws the same events: only the weights differ.
    for (index, ((a, s), u)) in average
        .events
        .iter()
        .zip(&sum.events)
        .zip(&unity.events)
        .enumerate()
    {
        for other in [s, u] {
            assert_eq!(
                a.particles.iter().map(|p| p.momentum).collect::<Vec<_>>(),
                other
                    .particles
                    .iter()
                    .map(|p| p.momentum)
                    .collect::<Vec<_>>(),
                "event {index}: the kinematics moved with event_norm"
            );
        }
        assert!(
            (s.weight * n / a.weight - 1.0).abs() < 1e-5,
            "event {index}: sum weight {:.6e} is not the average weight {:.6e} over {n}",
            s.weight,
            a.weight
        );
    }
}

/// `--event-norm` overrides the recorded `event_norm` and moves only the
/// weights: the `average` artifact under `--event-norm sum` writes the events,
/// `<init>` and weights the `sum` artifact writes under its own recorded value.
/// The two integrations are the same (`event_norm` trains nothing), so the
/// files differ only in the header line naming the artifact.
#[test]
fn the_event_norm_flag_overrides_the_recorded_value() {
    let recorded = run_with_event_norm("sum");
    let (from_record, _) = recorded.generate("buffer", SEED_A, FEW_EVENTS, "recorded-sum.lhe");
    let (from_flag, _) = run().generate_with(&["--event-norm", "sum"], SEED_A, "flag-sum.lhe");
    let from_flag = LheFile::parse(&from_flag).expect("our own file parses");
    assert_eq!(
        from_flag.init.weight_strategy,
        WeightStrategy::SumCrossSectionPb
    );
    assert_eq!(from_flag.init, from_record.init);
    assert_eq!(from_flag.events.len(), from_record.events.len());
    for (index, (a, b)) in from_flag.events.iter().zip(&from_record.events).enumerate() {
        assert_eq!(a, b, "event {index} moved between the flag and the record");
    }
}

/// Stochastic rounding's `IDWTUP = +3` unit-weight file is no `sum` or `unity`
/// file, so the combination is refused before anything is drawn.
#[test]
fn stochastic_rounding_refuses_an_event_norm_off_average() {
    let run = run_with_event_norm("sum");
    let out = run
        .generate_cmd(
            "stochastic-rounding",
            SEED_A,
            10,
            &run.dir.join("refused.lhe"),
        )
        .output()
        .expect("spawn vibegraph");
    expect_refusal(
        out,
        "unit weights under event_norm = sum",
        "--strategy stochastic-rounding writes unit weights",
    );
}
