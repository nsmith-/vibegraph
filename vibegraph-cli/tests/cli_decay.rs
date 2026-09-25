//! End-to-end gate on `1 → n` decay processes: `vibegraph integrate` on a decay
//! proc card measures a partial width, and `vibegraph generate` writes its events
//! in MadEvent's decay-run convention.
//!
//! The reference is MadEvent itself, run on the same proc card with its own
//! decay run card (`validation/madgraph/decay_width_reference.json`, written by
//! `gen_decay_widths.sh`) — no banked bundle, no PDF set, only the interned SM,
//! so it runs in the default suite.
//!
//! # What the width rows can and cannot see
//!
//! A width is an integral, so it is blind to anything that moves no weight — a
//! wrong helicity or colour label on an event, say. Each row is read over
//! [`SEEDS`] of this side against its reference: the pull of the seed mean, the χ²/dof of this side's seeds about their own mean
//! (a sampler that misses a region is confidently wrong on every seed at once, and
//! then scatters far less than it quotes, or far more), and each seed's own pull.
//!
//! The reference is the exact width where one is known and MadEvent's
//! otherwise. Three rows have one: `t > w+ b` and `z > e+ e-` from the SM UFO's
//! analytic `decays.py` (`sm_decay_widths.json`), and `h > e+ e- mu+ mu-` from
//! its factorisation into two off-shell `Z` lines, integrated by quadrature to
//! `1e-9` (`decay_width_exact.json`). On those rows MadEvent is compared against
//! the same exact value and reported, not gated: on `h > e+ e- mu+ mu-` its ten
//! 10k-event seeds sit 0.14% below it, 2.2 standard errors of their own mean,
//! scattering by 1.33× the error each quotes, and three 100k-event seeds sit
//! 0.05% below — MadEvent converging on the exact width with budget, where this
//! side's seeds sit within about one standard error at every budget measured.
//!
//! Each mean carries the larger of its quoted error and its seed spread over
//! `√n`, because the two differ where it matters. Thirty seeds of this side at
//! the budget one seed here spends measured the `h > e+ e- mu+ mu-` scatter at
//! 1.08× its quoted error.
//! Two rows carry run-card cuts, which MadEvent applies in the rest frame; a cut
//! read in any other frame, or one of MadEvent's `nincoming = 1` special cases
//! missed, moves those rows by far more than the few per mille they resolve.
//!
//! # What the sample row can and cannot see
//!
//! The file's convention — `IDBMUP`, `EBMUP`, the mother at rest with status
//! `-1`, the products' `(1, 0)` mother pointers, `SCALUP` and `AQCDUP` — is
//! compared field by field against what MadEvent writes for the same decay. The
//! shape is compared in two invariant masses binned from MadEvent's own events.
//! MadEvent additionally writes the intermediate `W` as a status-2 record between
//! the top and its leptons; this generator writes no intermediate records for any
//! process, so that record is not compared.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use serde::Deserialize;
use vibegraph::lhef::parse::LheFile;
use vibegraph::lhef::record::{LheEvent, STATUS_INCOMING, STATUS_OUTGOING};

/// Integration seeds per row.
const SEEDS: [u64; 10] = [11, 22, 33, 44, 55, 66, 77, 88, 99, 110];
/// Per-seed relative target: each seed then resolves the width to about the
/// accuracy of one MadEvent run, and the five-seed mean to about half of it.
const TARGET_REL: &str = "2e-3";

/// The seed mean against MadEvent's, in combined standard errors.
const MEAN_PULL_LIMIT: f64 = 3.0;
/// Any one seed against MadEvent's mean.
const SEED_PULL_LIMIT: f64 = 4.0;
/// This side's seeds about their own mean. Nine degrees of freedom put the 0.1%
/// and 99.9% points of χ²/dof at 0.15 and 3.1; below the lower bound a run would
/// be quoting errors several times wider than its own scatter.
const SEED_CHI2_RANGE: (f64, f64) = (0.15, 3.1);

#[derive(Deserialize)]
struct Reference {
    mg_version: String,
    rows: std::collections::BTreeMap<String, Row>,
    t_bev_sample: Sample,
}

#[derive(Deserialize)]
struct Row {
    process: String,
    run_card: Option<String>,
    runs: Vec<MgRun>,
}

#[derive(Deserialize)]
struct MgRun {
    width_gev: f64,
    err_gev: f64,
}

#[derive(Deserialize)]
struct Sample {
    events: usize,
    init: Vec<String>,
    edges: std::collections::BTreeMap<String, Vec<f64>>,
    counts: std::collections::BTreeMap<String, Vec<u64>>,
}

/// The exact widths the gate prefers over MadEvent's, keyed by process with its
/// daughters sorted.
fn exact_widths() -> std::collections::BTreeMap<String, f64> {
    #[derive(Deserialize)]
    struct TwoBody {
        parent: String,
        daughters: Vec<String>,
        width_gev: f64,
    }
    #[derive(Deserialize)]
    struct Analytic {
        widths: Vec<TwoBody>,
    }
    #[derive(Deserialize)]
    struct Quadrature {
        widths: std::collections::BTreeMap<String, f64>,
    }
    let read = |name: &str| {
        let path = madgraph_dir().join(name);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
    };
    let analytic: Analytic = serde_json::from_str(&read("sm_decay_widths.json")).unwrap();
    let quadrature: Quadrature = serde_json::from_str(&read("decay_width_exact.json")).unwrap();
    let mut out: std::collections::BTreeMap<String, f64> = analytic
        .widths
        .iter()
        .map(|w| {
            let parts: Vec<&str> = std::iter::once(w.parent.as_str())
                .chain(std::iter::once(">"))
                .chain(w.daughters.iter().map(String::as_str))
                .collect();
            (process_key(&parts.join(" ")), w.width_gev)
        })
        .collect();
    out.extend(
        quadrature
            .widths
            .into_iter()
            .map(|(process, width)| (process_key(&process), width)),
    );
    out
}

/// A process spelled with its daughters in sorted order, case folded.
fn process_key(process: &str) -> String {
    let lower = process.to_lowercase();
    let (mother, daughters) = lower.split_once('>').expect("a process has a '>'");
    let mut daughters: Vec<&str> = daughters.split_whitespace().collect();
    daughters.sort_unstable();
    format!("{} > {}", mother.trim(), daughters.join(" "))
}

fn madgraph_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph")
}

fn reference() -> &'static Reference {
    static REFERENCE: OnceLock<Reference> = OnceLock::new();
    REFERENCE.get_or_init(|| {
        let path = madgraph_dir().join("decay_width_reference.json");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        serde_json::from_str(&text).expect("decay_width_reference.json parses")
    })
}

/// `(value, error)` read off a command's result line, `Γ = <v> ± <e> GeV`.
fn parse_width(stdout: &str) -> (f64, f64) {
    let line = stdout
        .lines()
        .find(|l| l.starts_with("Γ = "))
        .unwrap_or_else(|| panic!("no width line in:\n{stdout}"));
    let fields: Vec<&str> = line.split_whitespace().collect();
    assert_eq!(fields[fields.len() - 1], "GeV", "{line}");
    (fields[2].parse().unwrap(), fields[4].parse().unwrap())
}

fn proc_card(dir: &Path, process: &str) -> PathBuf {
    let path = dir.join(format!("{}.dat", process.replace([' ', '>'], "_")));
    std::fs::write(&path, format!("import model sm\ngenerate {process}\n")).unwrap();
    path
}

fn integrate(
    dir: &Path,
    card: &Path,
    run_card: Option<&Path>,
    seed: u64,
    out: &Path,
) -> (f64, f64) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_vibegraph"));
    cmd.arg("integrate")
        .arg(card)
        .arg("--out")
        .arg(out)
        .arg("--force")
        .args(["--target-rel", TARGET_REL, "--seed", &seed.to_string()])
        .current_dir(dir);
    if let Some(path) = run_card {
        cmd.arg("--run-card").arg(path);
    }
    let output = cmd.output().expect("spawn vibegraph");
    assert!(
        output.status.success(),
        "vibegraph integrate {} failed:\n{}",
        card.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    parse_width(&String::from_utf8_lossy(&output.stdout))
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

#[test]
fn partial_widths_match_madevent_over_seeds() {
    let reference = reference();
    let exact = exact_widths();
    let tmp = tempfile::tempdir().unwrap();
    let mut failures = Vec::new();
    println!(
        "MadEvent {} decay widths against vibegraph, {} seeds each",
        reference.mg_version,
        SEEDS.len()
    );
    for (name, row) in &reference.rows {
        let card = proc_card(tmp.path(), &row.process);
        let run_card = row.run_card.as_ref().map(|c| madgraph_dir().join(c));
        let ours: Vec<(f64, f64)> = SEEDS
            .iter()
            .map(|&seed| {
                integrate(
                    tmp.path(),
                    &card,
                    run_card.as_deref(),
                    seed,
                    &tmp.path().join(name),
                )
            })
            .collect();
        let mg: Vec<(f64, f64)> = row.runs.iter().map(|r| (r.width_gev, r.err_gev)).collect();
        let (mg_mean, mg_err) = seed_mean(&mg);
        let (our_mean, our_err) = seed_mean(&ours);
        let chi2 = chi2_per_dof(&ours);
        // Rows with an exact width are gated against it, with no reference error;
        // cut rows have none and are gated against MadEvent.
        let exact = row
            .run_card
            .is_none()
            .then(|| exact.get(&process_key(&row.process)));
        let (reference_width, reference_err, against) = match exact.flatten() {
            Some(&width) => (width, 0.0, "exact"),
            None => (mg_mean, mg_err, "MG"),
        };
        let pull = (our_mean - reference_width) / (our_err.powi(2) + reference_err.powi(2)).sqrt();
        let worst_seed = ours
            .iter()
            .map(|(v, e)| ((v - reference_width) / (e * e + reference_err.powi(2)).sqrt()).abs())
            .fold(0.0f64, f64::max);
        if against == "exact" {
            println!(
                "{name:<11} exact {reference_width:.6e}: MG {:+.2} of its own error from it \
                 (reported, not gated)",
                (mg_mean - reference_width) / mg_err
            );
        }
        println!(
            "{name:<11} {:<20} MG {mg_mean:.6e} ± {mg_err:.2e} ({} seeds, χ²/dof {:.2}, spread \
             {:.2e}) | ours {our_mean:.6e} ± {our_err:.2e} (χ²/dof {chi2:.2}, spread {:.2e}) | \
             pull against {against} {pull:+.2}, worst seed {worst_seed:.2}",
            row.process,
            mg.len(),
            chi2_per_dof(&mg),
            spread(&mg),
            spread(&ours),
        );
        for (v, e) in &ours {
            println!("            seed value {v:.6e} ± {e:.2e}");
        }
        if pull.abs() > MEAN_PULL_LIMIT
            || worst_seed > SEED_PULL_LIMIT
            || !(SEED_CHI2_RANGE.0..SEED_CHI2_RANGE.1).contains(&chi2)
        {
            failures.push(name.clone());
        }
    }
    assert!(failures.is_empty(), "rows outside the gate: {failures:?}");
}

/// The event record of every file this gate writes: the mother at rest first,
/// then the products pointing at it alone.
fn check_decay_record(event: &LheEvent, mass: f64) {
    let mother = &event.particles[0];
    assert_eq!(mother.pdg, 6);
    assert_eq!(mother.status, STATUS_INCOMING);
    assert_eq!(mother.mothers, [0, 0]);
    assert_eq!(mother.momentum, [mass, 0.0, 0.0, 0.0]);
    let mut total = [0.0f64; 4];
    for product in &event.particles[1..] {
        assert_eq!(product.status, STATUS_OUTGOING);
        assert_eq!(
            product.mothers,
            [1, 0],
            "MadEvent's decay-run mother pointers"
        );
        for (t, p) in total.iter_mut().zip(product.momentum) {
            *t += p;
        }
    }
    for (t, m) in total.iter().zip(mother.momentum) {
        assert!((t - m).abs() < 1e-8 * mass, "momentum balance: {total:?}");
    }
}

fn invariant_mass(ps: &[[f64; 4]]) -> f64 {
    let s = ps.iter().fold([0.0; 4], |mut acc, p| {
        for i in 0..4 {
            acc[i] += p[i];
        }
        acc
    });
    (s[0] * s[0] - s[1] * s[1] - s[2] * s[2] - s[3] * s[3])
        .max(0.0)
        .sqrt()
}

#[test]
fn a_top_decay_sample_follows_madevents_convention_and_shape() {
    let reference = reference();
    let sample = &reference.t_bev_sample;
    let tmp = tempfile::tempdir().unwrap();
    let card = proc_card(tmp.path(), "t > b e+ ve");
    let out = tmp.path().join("t_bev");
    let (width, _) = integrate(tmp.path(), &card, None, 7, &out);

    let lhe = tmp.path().join("t_bev.lhe");
    let output = Command::new(env!("CARGO_BIN_EXE_vibegraph"))
        .arg("generate")
        .arg(out.join("grid.bin.zst"))
        .arg(&card)
        .args(["--strategy", "stochastic-rounding", "--seed", "20260925"])
        .args(["--nevents", &sample.events.to_string()])
        .arg("-o")
        .arg(&lhe)
        .output()
        .expect("spawn vibegraph");
    assert!(
        output.status.success(),
        "vibegraph generate failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let file = LheFile::parse(&std::fs::read_to_string(&lhe).unwrap()).expect("parses");
    let check = Command::new(env!("CARGO_BIN_EXE_vibegraph"))
        .arg("check-events")
        .arg(&lhe)
        .output()
        .expect("spawn vibegraph");
    assert!(
        check.status.success(),
        "check-events rejected the decay sample:\n{}",
        String::from_utf8_lossy(&check.stderr)
    );

    // <init>: MadEvent's `6 0 1.730000e+02 0.000000e+00 0 0 <lhaid> <lhaid> -4 1`
    // on its first line. PDFSUP is the one field left out: MadEvent writes the run
    // card's `lhaid` there even on runs with no parton densities, where this
    // generator writes 0 for a decay as for every fixed-energy run.
    let mg_beams: Vec<f64> = sample.init[0]
        .split_whitespace()
        .map(|f| f.parse().unwrap())
        .collect();
    assert_eq!(file.init.beam_pdg, [mg_beams[0] as i32, mg_beams[1] as i32]);
    assert_eq!(file.init.beam_energy, [mg_beams[2], mg_beams[3]]);
    assert_eq!(
        file.init.pdf_group,
        [mg_beams[4] as i32, mg_beams[5] as i32]
    );
    // XSECUP is the width in GeV, as MadEvent's is: within the sample's own
    // accept/reject resolution of the integrated width.
    let xsecup = file.init.processes[0].xsec_pb;
    assert!(
        (xsecup / width - 1.0).abs() < 4.0 / (sample.events as f64).sqrt(),
        "XSECUP {xsecup} against the integrated width {width} GeV"
    );
    let mg_xsec: f64 = sample.init[1]
        .split_whitespace()
        .next()
        .unwrap()
        .parse()
        .unwrap();
    assert!(
        (xsecup / mg_xsec - 1.0).abs() < 0.03,
        "XSECUP {xsecup}, MadEvent's {mg_xsec}"
    );

    for event in &file.events {
        check_decay_record(event, mg_beams[2]);
        // SCALUP is the larger fixed factorisation scale, AQCDUP αs at the top
        // mass — MadEvent's `setcuts.f` fixes both for one incoming particle.
        assert_eq!(event.scale, 91.188);
        assert!(
            (event.alpha_qcd - 0.1076279).abs() < 5e-7,
            "AQCDUP {}",
            event.alpha_qcd
        );
    }

    let mut chi2_total = 0.0;
    let mut dof = 0;
    for (observable, edges) in &sample.edges {
        let mg_counts = &sample.counts[observable];
        let mut ours = vec![0u64; edges.len() - 1];
        for event in &file.events {
            let find = |pdg: i32| {
                event
                    .particles
                    .iter()
                    .find(|p| p.pdg == pdg && p.status == STATUS_OUTGOING)
                    .unwrap()
                    .momentum
            };
            let value = match observable.as_str() {
                "m_eve" => invariant_mass(&[find(-11), find(12)]),
                "m_be" => invariant_mass(&[find(5), find(-11)]),
                other => panic!("unknown observable {other}"),
            };
            if let Some(bin) = edges.windows(2).position(|w| w[0] <= value && value < w[1]) {
                ours[bin] += 1;
            }
        }
        let (n, m) = (
            ours.iter().sum::<u64>() as f64,
            mg_counts.iter().sum::<u64>() as f64,
        );
        let mut chi2 = 0.0;
        let mut bins = 0;
        for (i, (&a, &b)) in ours.iter().zip(mg_counts).enumerate() {
            if a + b == 0 {
                continue;
            }
            let (pa, pb) = (a as f64 / n, b as f64 / m);
            let pull = (pa - pb) / (a as f64 / (n * n) + b as f64 / (m * m)).sqrt();
            println!(
                "{observable} [{:>6.1}, {:>6.1}): ours {a:>5}, MadEvent {b:>5}, pull {pull:+.2}",
                edges[i],
                edges[i + 1]
            );
            assert!(pull.abs() < 4.0, "{observable} bin {i}: pull {pull}");
            chi2 += pull * pull;
            bins += 1;
        }
        println!(
            "{observable}: χ²/dof = {:.2} over {} bins",
            chi2 / (bins - 1) as f64,
            bins
        );
        chi2_total += chi2;
        dof += bins - 1;
    }
    let chi2_per_dof = chi2_total / dof as f64;
    assert!(
        chi2_per_dof < 2.0,
        "shape χ²/dof {chi2_per_dof:.2} over {dof} dof"
    );
}
