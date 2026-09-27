//! The `samples` category for decay chains and for cards with several process
//! numbers: the shipped binary's own event files against MadEvent's.
//!
//! The reference is `validation/madgraph/decay_chain_events_reference.json`,
//! written by `gen_decay_chain_events.sh` and `summarise_decay_chain_events.py`:
//! per row, five MadEvent runs of 10k unweighted events, pooled into the
//! quantities compared here. This side integrates the row's card once and
//! writes its own sample with `vibegraph generate`, then reads it back through
//! the same summary.
//!
//! Gated behind `extended-validation`: the hadronic rows need the fetched PDF
//! set.
//!
//!     cargo test -p vibegraph --features extended-validation \
//!         --test cli_decay_chain_events -- --nocapture
//!
//! # What is compared
//!
//! * **The resonance records.** Every event's status-2 records as a tree,
//!   order- and position-free (`6[24[-11,12],5]`), and the frequency of each
//!   tree: a forced top is always written, a free `W` inside it only when its
//!   invariant mass lands inside its own window, so the trees carry the window
//!   rule. The records' positions are not compared: MadEvent orders siblings by
//!   its configuration tag, which differs between configurations of one
//!   subprocess.
//! * **Their fields.** Each record's momentum is its daughters' sum and its
//!   `ICOLUP` what its daughters leave open, on both sides and on every record;
//!   its `SPINUP` is 9; its mass distribution inside the window is binned
//!   against MadEvent's.
//! * **`SCALUP` and `AQCDUP`** at the default dynamical scale, twice: binned
//!   against MadEvent's own sample (informational, see below), and replayed
//!   event by event — MadEvent's own momenta handed to this crate's scale
//!   prescription, whose clustering must return MadEvent's `SCALUP` in one of
//!   the event's configurations and `αs` of it must be MadEvent's `AQCDUP`.
//!   This crate's own events are replayed the same way, on the decayed
//!   `p p > t t~` and on `p p > l+ l- j` at the dynamical scale, and each
//!   `SCALUP` has to come from a configuration of the flavour group the event
//!   is labelled with.
//! * **The process split** of a two-`@N` card: one `<init>` entry per process
//!   number, each event's `IDPRUP`, and the share of events per process against
//!   MadEvent's own per-process `XSECUP`.
//!
//! # What it provably cannot detect
//!
//! * Correlations between columns: each column is a marginal.
//! * The record order: see above.
//! * Which configuration MadEvent integrated an event in: the replay accepts a
//!   `SCALUP` any configuration of the event's subprocess reproduces.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;
use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card};
use vibegraph::hadronic::SampledChannel;
use vibegraph::helas::eval::BoundAmplitude;
use vibegraph::lhef::parse::LheFile;
use vibegraph::lhef::record::{LheEvent, STATUS_INCOMING, STATUS_INTERMEDIATE, STATUS_OUTGOING};
use vibegraph::pdf::PdfSet;
use vibegraph::phasespace::maps::MapOptions;
use vibegraph::proton::{derive_flavor_groups, ProtonIntegrand};
use vibegraph::runcard::RunCard;
use vibegraph::stats::{chi2_homogeneity, effective_counts, Chi2Test};
use vibegraph::ufo::sm::{sm_model, SMRestrict};
use vibegraph::ufo::EvaluatedModel;

/// The PDF set every hadronic row was generated with.
const PDF_SET: &str = "NNPDF23_lo_as_0130_qed";
/// Integration seed, and the generation seeds each row's sample pools.
const INTEGRATION_SEED: &str = "20260926";
const GEN_SEEDS: [u64; 2] = [0xE1_0001, 0xE1_0002];
/// Events per generation seed.
const EVENTS_PER_SEED: usize = 20_000;
/// The p-value a gated column must clear: the floor the other `samples` gates
/// use, chosen there from the same kind of trial count (a handful of rows, a
/// dozen columns each).
const P_FLOOR: f64 = 1e-4;

#[derive(Deserialize)]
struct Reference {
    mg_version: String,
    rows: BTreeMap<String, Row>,
}

#[derive(Deserialize)]
struct Row {
    lines: String,
    run_card: String,
    runs: Vec<MgRun>,
    init: Vec<String>,
    events: u64,
    structures: BTreeMap<String, u64>,
    spins: BTreeMap<String, u64>,
    processes: BTreeMap<String, u64>,
    histograms: BTreeMap<String, Histogram>,
    status2_records: u64,
    color_mismatches: u64,
    momentum_mismatches: u64,
    #[serde(default)]
    replay: Vec<ReplayEvent>,
}

#[derive(Deserialize)]
struct MgRun {
    sigma_pb: f64,
    err_pb: f64,
}

#[derive(Deserialize, Clone)]
struct Histogram {
    edges: Vec<f64>,
    counts: Vec<u64>,
    outside: u64,
}

#[derive(Deserialize)]
struct ReplayEvent {
    /// `[pdg, E, px, py, pz]` per leg.
    incoming: Vec<[f64; 5]>,
    outgoing: Vec<[f64; 5]>,
    scalup: f64,
    aqcdup: f64,
}

fn madgraph_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph")
}

fn pdf_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/pdf")
}

fn reference() -> Reference {
    let path = madgraph_dir().join("decay_chain_events_reference.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    serde_json::from_str(&text).expect("decay_chain_events_reference.json parses")
}

fn row<'r>(reference: &'r Reference, name: &str) -> &'r Row {
    reference
        .rows
        .get(name)
        .unwrap_or_else(|| panic!("the reference has no row {name}"))
}

/// The proc card a row's `lines` spell (`;` separates card lines).
fn proc_card(dir: &Path, row: &Row) -> PathBuf {
    let card = dir.join("proc_card.dat");
    let body: String = row.lines.split(';').map(|l| format!("{l}\n")).collect();
    std::fs::write(&card, format!("import model sm\n{body}")).unwrap();
    card
}

fn vibegraph(args: &[&std::ffi::OsStr]) -> std::process::Output {
    let output = Command::new(env!("CARGO_BIN_EXE_vibegraph"))
        .args(args)
        .output()
        .expect("spawn vibegraph");
    assert!(
        output.status.success(),
        "vibegraph {:?} failed:\n{}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// `(σ, error)` of one integration of the row's card at `seed`, and the
/// artifact it wrote.
fn integrate(
    dir: &Path,
    card: &Path,
    run_card: &Path,
    seed: &str,
    target: &str,
) -> (f64, f64, PathBuf) {
    let out = dir.join(format!("run_{seed}"));
    vibegraph(&[
        "integrate".as_ref(),
        card.as_os_str(),
        "--run-card".as_ref(),
        run_card.as_os_str(),
        "--pdf-dir".as_ref(),
        pdf_dir().as_os_str(),
        "--out".as_ref(),
        out.as_os_str(),
        "--force".as_ref(),
        "--target-rel".as_ref(),
        target.as_ref(),
        "--seed".as_ref(),
        seed.as_ref(),
    ]);
    let artifact = out.join("grid.bin.zst");
    let banked =
        vibegraph::artifact::IntegrateArtifact::read_from_path(&artifact).expect("artifact");
    (banked.sigma_pb, banked.sigma_err_pb, artifact)
}

/// One sample per generation seed off `artifact`, each checked by
/// `check-events`, parsed.
fn samples(
    dir: &Path,
    card: &Path,
    run_card: &Path,
    artifact: &Path,
    strategy: &str,
) -> Vec<LheFile> {
    GEN_SEEDS
        .iter()
        .map(|seed| {
            let lhe = dir.join(format!("events_{strategy}_{seed:x}.lhe"));
            vibegraph(&[
                "generate".as_ref(),
                artifact.as_os_str(),
                card.as_os_str(),
                "--run-card".as_ref(),
                run_card.as_os_str(),
                "--pdf-dir".as_ref(),
                pdf_dir().as_os_str(),
                "--seed".as_ref(),
                seed.to_string().as_ref(),
                "--nevents".as_ref(),
                EVENTS_PER_SEED.to_string().as_ref(),
                "--strategy".as_ref(),
                strategy.as_ref(),
                "-o".as_ref(),
                lhe.as_os_str(),
            ]);
            vibegraph(&["check-events".as_ref(), lhe.as_os_str()]);
            LheFile::parse(&std::fs::read_to_string(&lhe).unwrap()).expect("our own file parses")
        })
        .collect()
}

// ── The summary, as summarise_decay_chain_events.py reads MadEvent's files ──

/// 1-based positions of the records naming `index` as their first mother.
fn daughters(event: &LheEvent, index: usize) -> Vec<usize> {
    event
        .particles
        .iter()
        .enumerate()
        .filter(|(_, p)| p.status != STATUS_INCOMING && p.mothers[0] as usize == index)
        .map(|(i, _)| i + 1)
        .collect()
}

fn tree(event: &LheEvent, index: usize) -> String {
    let p = &event.particles[index - 1];
    if p.status != STATUS_INTERMEDIATE {
        return p.pdg.to_string();
    }
    let mut below: Vec<String> = daughters(event, index)
        .into_iter()
        .map(|d| tree(event, d))
        .collect();
    below.sort();
    format!("{}[{}]", p.pdg, below.join(","))
}

/// The event's resonance trees, sorted, then the outgoing legs no record claims.
fn structure(event: &LheEvent) -> String {
    let from_beams = |m: i32| m == 0 || event.particles[m as usize - 1].status == STATUS_INCOMING;
    let mut tops = Vec::new();
    let mut loose = Vec::new();
    for (i, p) in event.particles.iter().enumerate() {
        if p.status == STATUS_INTERMEDIATE && from_beams(p.mothers[0]) {
            tops.push(tree(event, i + 1));
        } else if p.status == STATUS_OUTGOING && from_beams(p.mothers[0]) {
            loose.push(format!(".{}", p.pdg));
        }
    }
    tops.sort();
    loose.sort();
    tops.extend(loose);
    tops.join(" ")
}

/// Whether record `index`'s `ICOLUP` is what its daughters leave open.
fn color_ok(event: &LheEvent, index: usize) -> bool {
    let mut open_c = Vec::new();
    let mut open_a = Vec::new();
    for d in daughters(event, index) {
        let [c, a] = event.particles[d - 1].color;
        if c != 0 {
            open_c.push(c);
        }
        if a != 0 {
            open_a.push(a);
        }
    }
    let free_c: Vec<i32> = open_c
        .iter()
        .copied()
        .filter(|c| !open_a.contains(c))
        .collect();
    let free_a: Vec<i32> = open_a
        .iter()
        .copied()
        .filter(|a| !open_c.contains(a))
        .collect();
    let want = [
        free_c.first().copied().unwrap_or(0),
        free_a.first().copied().unwrap_or(0),
    ];
    event.particles[index - 1].color == want && free_c.len() <= 1 && free_a.len() <= 1
}

/// Whether record `index`'s momentum is its daughters' sum.
fn momentum_ok(event: &LheEvent, index: usize) -> bool {
    let p = &event.particles[index - 1];
    let mut sum = [0.0; 4];
    for d in daughters(event, index) {
        for (k, s) in sum.iter_mut().enumerate() {
            *s += event.particles[d - 1].momentum[k];
        }
    }
    (0..4).all(|k| (sum[k] - p.momentum[k]).abs() <= 1e-7 * p.momentum[0].abs().max(1.0))
}

fn status2_of(
    event: &LheEvent,
    pdg: i32,
) -> impl Iterator<Item = &vibegraph::lhef::record::LheParticle> {
    event
        .particles
        .iter()
        .filter(move |p| p.status == STATUS_INTERMEDIATE && p.pdg == pdg)
}

fn pair_mass(event: &LheEvent, a: i32, b: i32) -> Vec<f64> {
    let legs: Vec<_> = event
        .particles
        .iter()
        .filter(|p| p.status == STATUS_OUTGOING && (p.pdg == a || p.pdg == b))
        .collect();
    if legs.len() != 2 {
        return Vec::new();
    }
    let s: Vec<f64> = (0..4)
        .map(|k| legs[0].momentum[k] + legs[1].momentum[k])
        .collect();
    vec![(s[0] * s[0] - s[1] * s[1] - s[2] * s[2] - s[3] * s[3])
        .max(0.0)
        .sqrt()]
}

/// The observable a histogram of the reference is named for.
fn observable(name: &str, event: &LheEvent) -> Vec<f64> {
    let mass = |pdg| status2_of(event, pdg).map(|p| p.mass).collect();
    match name {
        "m_t" | "m_tbar" | "m_wp" | "m_wm" | "m_z" => mass(match name {
            "m_t" => 6,
            "m_tbar" => -6,
            "m_wp" => 24,
            "m_wm" => -24,
            _ => 23,
        }),
        "pt_t" => status2_of(event, 6)
            .map(|p| p.momentum[1].hypot(p.momentum[2]))
            .collect(),
        "y_t" => status2_of(event, 6)
            .map(|p| 0.5 * ((p.momentum[0] + p.momentum[3]) / (p.momentum[0] - p.momentum[3])).ln())
            .collect(),
        "scalup" => vec![event.scale],
        "aqcdup" => vec![event.alpha_qcd],
        "m_ee" => pair_mass(event, -11, 11),
        "m_mumu" => pair_mass(event, -13, 13),
        other => panic!("no observable {other}"),
    }
}

/// A weighted count: the summed event weights and their squares.
#[derive(Clone, Copy, Default, Debug)]
struct Weighted {
    w: f64,
    w2: f64,
}

impl Weighted {
    fn add(&mut self, w: f64) {
        self.w += w;
        self.w2 += w * w;
    }
}

/// This side's sample, every count weighted by the event's `XWGTUP` over the
/// file's mean: a buffered file keeps the overweight tail as weights, so an
/// unweighted count would read that tail short.
#[derive(Default)]
struct Summary {
    events: u64,
    structures: BTreeMap<String, Weighted>,
    spins: BTreeMap<String, Weighted>,
    processes: BTreeMap<String, Weighted>,
    histograms: BTreeMap<String, (Vec<Weighted>, u64)>,
    records: u64,
    color_bad: u64,
    momentum_bad: u64,
}

fn summarise(files: &[LheFile], row: &Row) -> Summary {
    let mut s = Summary::default();
    for (name, h) in &row.histograms {
        s.histograms
            .insert(name.clone(), (vec![Weighted::default(); h.counts.len()], 0));
    }
    for file in files {
        let mean = file.events.iter().map(|e| e.weight).sum::<f64>() / file.events.len() as f64;
        for event in &file.events {
            let w = event.weight / mean;
            s.events += 1;
            s.structures.entry(structure(event)).or_default().add(w);
            s.processes
                .entry(event.process_id.to_string())
                .or_default()
                .add(w);
            for (i, p) in event.particles.iter().enumerate() {
                s.spins
                    .entry(format!("{} {} {}", p.status, p.pdg, p.spin))
                    .or_default()
                    .add(w);
                if p.status == STATUS_INTERMEDIATE {
                    s.records += 1;
                    s.color_bad += u64::from(!color_ok(event, i + 1));
                    s.momentum_bad += u64::from(!momentum_ok(event, i + 1));
                }
            }
            for (name, h) in &row.histograms {
                let entry = s.histograms.get_mut(name).unwrap();
                for v in observable(name, event) {
                    match h.edges.windows(2).position(|w| w[0] <= v && v < w[1]) {
                        Some(b) => entry.0[b].add(w),
                        None => entry.1 += 1,
                    }
                }
            }
        }
    }
    s
}

/// A column's comparison: the χ² homogeneity test of this side's weighted bins
/// against MadEvent's unweighted ones — this side's counts deflated to their
/// effective size ([`effective_counts`]) — and the largest single-bin pull, each
/// pull's variance `Σw²/W² + m/M²`, for diagnosis.
struct Column {
    test: Chi2Test,
    worst_pull: f64,
}

fn homogeneity(ours: &[Weighted], theirs: &[u64]) -> Column {
    let sum_w: Vec<f64> = ours.iter().map(|c| c.w).collect();
    let sum_w2: Vec<f64> = ours.iter().map(|c| c.w2).collect();
    let mg: Vec<f64> = theirs.iter().map(|&c| c as f64).collect();
    // A column with a single populated category on both sides has nothing to
    // compare and passes as an equality.
    let test = chi2_homogeneity(&effective_counts(&sum_w, &sum_w2), &mg).unwrap_or(Chi2Test {
        chi2: 0.0,
        dof: 0,
        p: 1.0,
        categories: 1,
        pooled: 0,
        pooled_share: 0.0,
    });
    let n: f64 = sum_w.iter().sum();
    let m: f64 = mg.iter().sum();
    let worst_pull = ours
        .iter()
        .zip(&mg)
        .filter(|(a, &b)| a.w + b > 0.0)
        .map(|(a, &b)| ((a.w / n - b / m) / (a.w2 / (n * n) + b / (m * m)).sqrt()).abs())
        .fold(0.0f64, f64::max);
    Column { test, worst_pull }
}

/// [`homogeneity`] over two keyed counts, missing keys counting zero.
fn categorical(ours: &BTreeMap<String, Weighted>, theirs: &BTreeMap<String, u64>) -> Column {
    let keys: BTreeSet<&String> = ours.keys().chain(theirs.keys()).collect();
    let a: Vec<Weighted> = keys
        .iter()
        .map(|k| ours.get(*k).copied().unwrap_or_default())
        .collect();
    let b: Vec<u64> = keys
        .iter()
        .map(|k| theirs.get(*k).copied().unwrap_or(0))
        .collect();
    homogeneity(&a, &b)
}

/// Print one column, and record a failure where a gated one fails.
fn column(name: &str, c: Column, gated: bool, failures: &mut Vec<String>) {
    let verdict = if !gated {
        "informational"
    } else if c.test.p < P_FLOOR {
        failures.push(format!(
            "{name}: χ² {:.1} over {} dof, p {:.2e}",
            c.test.chi2, c.test.dof, c.test.p
        ));
        "FAIL"
    } else {
        "ok"
    };
    println!(
        "  {name:<14} χ² {:>6.1} over {:>2} dof, p {:.3e}, worst pull {:>5.2}  {verdict}",
        c.test.chi2, c.test.dof, c.test.p, c.worst_pull
    );
}

/// The shared record checks of every decay-chain row.
fn compare_records(name: &str, row: &Row, ours: &Summary, ungated: &[&str]) -> Vec<String> {
    let mut failures = Vec::new();
    println!(
        "{name}: ours {} events, MadEvent {} ({} status-2 records ours, {} MadEvent)",
        ours.events, row.events, ours.records, row.status2_records
    );
    assert_eq!(
        row.color_mismatches, 0,
        "MadEvent's own records break the colour rule"
    );
    assert_eq!(
        row.momentum_mismatches, 0,
        "MadEvent's own records break the momentum rule"
    );
    if ours.color_bad + ours.momentum_bad > 0 {
        failures.push(format!(
            "{} records whose ICOLUP, and {} whose momentum, is not their daughters'",
            ours.color_bad, ours.momentum_bad
        ));
    }
    println!("  resonance trees (ours / MadEvent):");
    let keys: BTreeSet<&String> = ours
        .structures
        .keys()
        .chain(row.structures.keys())
        .collect();
    for k in &keys {
        println!(
            "    {:>8.1} / {:>6}  {k}",
            ours.structures.get(*k).copied().unwrap_or_default().w,
            row.structures.get(*k).copied().unwrap_or(0)
        );
    }
    for k in ours.structures.keys() {
        if !row.structures.contains_key(k) {
            failures.push(format!("a resonance tree MadEvent never writes: {k}"));
        }
    }
    column(
        "trees",
        categorical(&ours.structures, &row.structures),
        !ungated.contains(&"trees"),
        &mut failures,
    );
    // Every intermediate's helicity is summed over.
    for (key, _) in ours.spins.iter().filter(|(k, _)| k.starts_with("2 ")) {
        if !key.ends_with(" 9") {
            failures.push(format!("an intermediate SPINUP other than 9: {key}"));
        }
    }
    let ours_out: BTreeMap<String, Weighted> = ours
        .spins
        .iter()
        .filter(|(k, _)| k.starts_with("1 "))
        .map(|(k, v)| (k.clone(), *v))
        .collect();
    let mg_out: BTreeMap<String, u64> = row
        .spins
        .iter()
        .filter(|(k, _)| k.starts_with("1 "))
        .map(|(k, v)| (k.clone(), *v))
        .collect();
    column(
        "SPINUP",
        categorical(&ours_out, &mg_out),
        true,
        &mut failures,
    );
    for (obs, h) in &row.histograms {
        let (counts, outside) = &ours.histograms[obs];
        let gated = !ungated.contains(&obs.as_str());
        column(obs, homogeneity(counts, &h.counts), gated, &mut failures);
        if obs.starts_with("m_") && *outside > 0 && gated {
            failures.push(format!("{outside} {obs} values outside the window"));
        }
        let _ = h.outside;
    }
    failures
}

/// The status-2 records, the colour and momentum rules, the trees, `SPINUP` and
/// the resonance line shapes of `p p > t t~, t > b e+ ve, t~ > b~ mu- vm~` at
/// 13 TeV on MadGraph's default dynamical scale — the forced tops and the free
/// `W` lines inside them.
///
/// `SCALUP` and `AQCDUP` are measured and not enforced: a hadronic point's scale
/// is clustered in the flavour group the sampler drew it in, and an event is
/// labelled with the group its luminosity-weighted `|M|²` draws, so an event of
/// one group can carry the other's clustering. MadEvent integrates each group
/// on its own and never does. On `p p > t t~` the two clusterings agree, since
/// both tops sit at their pole mass; with the tops' virtualities apart they
/// separate (`q q̄` takes the geometric mean of the two transverse masses, `g g`
/// the larger), which these two columns see.
/// [`madevents_own_events_replay_through_the_scale_prescription`] is the
/// per-event statement that the clustering itself is MadEvent's.
#[test]
fn a_hadronic_decay_chain_sample_matches_madevents_records() {
    let reference = reference();
    let row = row(&reference, "pp_ttx_lep_dyn");
    let tmp = tempfile::tempdir().unwrap();
    let card = proc_card(tmp.path(), row);
    let run_card = madgraph_dir().join(&row.run_card);
    let (_, _, artifact) = integrate(tmp.path(), &card, &run_card, INTEGRATION_SEED, "3e-3");
    let files = samples(tmp.path(), &card, &run_card, &artifact, "buffer");
    // `<init>`: MadEvent's beams, energies and PDF ids. `IDWTUP` is not compared:
    // this card leaves `event_norm` at MadGraph's system default `sum` (`-3`)
    // where `generate` writes the mean (`-4`).
    let mg: Vec<&str> = row.init[0].split_whitespace().collect();
    let init = &files[0].init;
    assert_eq!(init.beam_pdg, [2212, 2212]);
    assert_eq!(
        init.pdf_set,
        [mg[6].parse::<i32>().unwrap(), mg[7].parse().unwrap()]
    );
    assert_eq!(init.processes.len(), 1);
    let ours = summarise(&files, row);
    let failures = compare_records("pp_ttx_lep_dyn", row, &ours, &["scalup", "aqcdup"]);
    assert!(failures.is_empty(), "{failures:#?}");
}

/// The nested `e+ e- > t t~, (t > w+ b, w+ > e+ ve), t~ > w- b~` at 500 GeV:
/// a forced resonance inside a forced resonance, whose mothers name their
/// parent twice, and an undecayed `W-` that stays an outgoing leg.
#[test]
fn a_nested_decay_chain_sample_matches_madevents_records() {
    let reference = reference();
    let row = row(&reference, "ttx_nested");
    let tmp = tempfile::tempdir().unwrap();
    let card = proc_card(tmp.path(), row);
    let run_card = madgraph_dir().join(&row.run_card);
    let (_, _, artifact) = integrate(tmp.path(), &card, &run_card, INTEGRATION_SEED, "3e-3");
    let files = samples(tmp.path(), &card, &run_card, &artifact, "buffer");
    let ours = summarise(&files, row);
    let failures = compare_records("ttx_nested", row, &ours, &[]);
    assert!(failures.is_empty(), "{failures:#?}");
}

/// `e+ e- > z z, z > e+ e-`: which pairing of the four leptons the two `Z`
/// records carry.
///
/// MadGraph generates one pairing and writes its two `Z`s; this side keeps both
/// pairings (the 2026-09-26 decision) and writes the pairing of the
/// configuration the event's colour flow is drawn in, among those whose forced
/// windows the event is inside — so every record is a `Z` on its window, as
/// MadEvent's are. The line shape is compared informationally: the kept
/// interference between pairings moves the sample by the few per mille the
/// cross-section row measures.
#[test]
fn identical_decays_write_a_pairing_inside_its_windows() {
    let reference = reference();
    let row = row(&reference, "zz_ee");
    let tmp = tempfile::tempdir().unwrap();
    let card = proc_card(tmp.path(), row);
    let run_card = madgraph_dir().join(&row.run_card);
    let (_, _, artifact) = integrate(tmp.path(), &card, &run_card, INTEGRATION_SEED, "3e-3");
    let files = samples(tmp.path(), &card, &run_card, &artifact, "buffer");
    let ours = summarise(&files, row);
    let failures = compare_records("zz_ee", row, &ours, &["m_z"]);
    assert!(failures.is_empty(), "{failures:#?}");
}

/// `p p > e+ e- / z @1` and `add process p p > mu+ mu- / a @2` on the Drell–Yan
/// window card: one `<init>` entry per process number, each event's `IDPRUP`
/// its own line's, and the split of the sample between them against
/// MadEvent's per-process cross sections.
///
/// The resonance trees are not compared here: MadEvent writes the `Z` of the
/// `@2` line as a status-2 record inside its Breit–Wigner window, as its
/// `addmothers` does for any s-channel line of a plain process, and `generate`
/// writes intermediate records for decay chains only.
#[test]
fn two_process_numbers_split_the_sample_as_madevent_does() {
    let reference = reference();
    let row = row(&reference, "dy_two_procs");
    let tmp = tempfile::tempdir().unwrap();
    let card = proc_card(tmp.path(), row);
    let run_card = madgraph_dir().join(&row.run_card);
    let (sigma, sigma_err, artifact) =
        integrate(tmp.path(), &card, &run_card, INTEGRATION_SEED, "2e-3");
    let mut failures = Vec::new();
    for strategy in ["buffer", "stochastic-rounding"] {
        let files = samples(tmp.path(), &card, &run_card, &artifact, strategy);
        let ids: Vec<i32> = files[0].init.processes.iter().map(|p| p.id).collect();
        assert_eq!(ids, [1, 2], "one <init> entry per process number");
        // MadEvent's per-process cross sections, pooled over its seeds from each
        // run's own split of the same `<init>` lines.
        let mg_share = row.processes["1"] as f64 / row.events as f64;
        let mut ours_1 = 0.0f64;
        let mut ours_n = 0.0f64;
        for file in &files {
            let xsec: Vec<f64> = file.init.processes.iter().map(|p| p.xsec_pb).collect();
            let share_init = xsec[0] / (xsec[0] + xsec[1]);
            let mean = file.events.iter().map(|e| e.weight).sum::<f64>() / file.events.len() as f64;
            for event in &file.events {
                let w = event.weight / mean;
                ours_n += w;
                let leptons: Vec<i32> = event
                    .particles
                    .iter()
                    .filter(|p| p.status == STATUS_OUTGOING)
                    .map(|p| p.pdg.abs())
                    .collect();
                let want = if event.process_id == 1 { 11 } else { 13 };
                if leptons != [want, want] {
                    failures.push(format!(
                        "an event of process {} carries {leptons:?}",
                        event.process_id
                    ));
                    break;
                }
                if event.process_id == 1 {
                    ours_1 += w;
                }
            }
            // XSECUP_1 / Σ XSECUP is the file's own share of process 1: its
            // weight's share of the file's.
            let w1: f64 = file
                .events
                .iter()
                .filter(|e| e.process_id == 1)
                .map(|e| e.weight)
                .sum();
            let share_file = w1 / file.events.iter().map(|e| e.weight).sum::<f64>();
            let summed: f64 = xsec.iter().sum();
            println!(
                "{strategy}: XSECUP {:.4} + {:.4} = {summed:.4} pb (integration {sigma:.4} ± \
                 {sigma_err:.4}), share of @1 in <init> {share_init:.5}, in the events \
                 {share_file:.5}",
                xsec[0], xsec[1]
            );
            // Both strategies declare the share their own file carries, to the
            // seven digits `XSECUP` is printed with.
            if (share_init - share_file).abs() > 1e-6 {
                failures.push(format!(
                    "{strategy}: <init> share {share_init} is not the file's {share_file}"
                ));
            }
        }
        let share = ours_1 / ours_n;
        let err = (share * (1.0 - share) / ours_n
            + mg_share * (1.0 - mg_share) / row.events as f64)
            .sqrt();
        let pull = (share - mg_share) / err;
        println!(
            "{strategy}: share of @1 ours {share:.5} over {ours_n} events, MadEvent {mg_share:.5} \
             over {}: pull {pull:+.2}",
            row.events
        );
        if pull.abs() > 4.0 {
            failures.push(format!("{strategy}: @1 share pull {pull:+.2}"));
        }
        let ours = summarise(&files, row);
        for (obs, h) in &row.histograms {
            column(
                obs,
                homogeneity(&ours.histograms[obs].0, &h.counts),
                true,
                &mut failures,
            );
        }
    }
    let mg_sigma: f64 = row.runs.iter().map(|r| r.sigma_pb).sum::<f64>() / row.runs.len() as f64;
    println!("integration {sigma:.4} ± {sigma_err:.4} pb, MadEvent {mg_sigma:.4} pb");
    assert!(failures.is_empty(), "{failures:#?}");
}

/// `e+ e- > mu+ mu- z $ z, z > e+ e-`: a forbidden on-shell `Z` on the core of
/// a decay chain against MadEvent, five seeds each.
///
/// The `$` marks the core's own s-channel `Z` (the one into `mu+ mu-`) and
/// neither the forced `Z` nor any line inside a decay: MadGraph marks the core
/// amplitude before the decays are attached. Read the other way the forced `Z`
/// would be zeroed on exactly its own window and the cross section would vanish.
#[test]
fn a_forbidden_onshell_line_on_a_chain_core_matches_madevent() {
    let reference = reference();
    let row = row(&reference, "veto_chain");
    let tmp = tempfile::tempdir().unwrap();
    let card = proc_card(tmp.path(), row);
    let run_card = madgraph_dir().join(&row.run_card);
    let ours: Vec<(f64, f64)> = ["11", "22", "33", "44", "55"]
        .iter()
        .map(|seed| {
            let (s, e, _) = integrate(tmp.path(), &card, &run_card, seed, "2e-3");
            (s, e)
        })
        .collect();
    let mg: Vec<(f64, f64)> = row.runs.iter().map(|r| (r.sigma_pb, r.err_pb)).collect();
    let mean = |v: &[(f64, f64)]| -> (f64, f64) {
        let n = v.len() as f64;
        let m = v.iter().map(|x| x.0).sum::<f64>() / n;
        let spread = (v.iter().map(|x| (x.0 - m).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
        let quoted = (v.iter().map(|x| x.1 * x.1).sum::<f64>()).sqrt() / n;
        (m, quoted.max(spread / n.sqrt()))
    };
    let (om, oe) = mean(&ours);
    let (mm, me) = mean(&mg);
    let pull = (om - mm) / (oe * oe + me * me).sqrt();
    println!(
        "MadEvent {} {}: {mm:.6e} ± {me:.1e}; ours {om:.6e} ± {oe:.1e}; ours/MG − 1 = \
         {:+.2e}; pull {pull:+.2}",
        reference.mg_version,
        row.lines,
        om / mm - 1.0
    );
    for (s, e) in &ours {
        println!("  seed {s:.6e} ± {e:.1e}");
    }
    assert!(pull.abs() < 3.0, "pull {pull:+.2}");
}

/// MadEvent's own decay-chain events through this crate's scale prescription.
///
/// For each of the reference's replay events (the first 300 of one MadEvent
/// run of the `pp_ttx_lep_dyn` card), the event's momenta are handed to the
/// clustering of every configuration of its flavour group: one of them has to
/// return the event's `SCALUP` (the larger factorisation scale), and `αs` of
/// that configuration's `μR` from the PDF set's own running has to be its
/// `AQCDUP`. The decay products are clustered into their tops through the
/// forced Breit–Wigner lines first, so what is compared is the clustering of
/// the `t t̄` core at the tops' own virtualities.
///
/// The tolerances are the printed precision: `SCALUP` and `AQCDUP` carry seven
/// significant digits, and `AQCDUP` MadGraph's `π` truncation, `1.7e-8`
/// relative.
#[test]
fn madevents_own_events_replay_through_the_scale_prescription() {
    let reference = reference();
    let row = row(&reference, "pp_ttx_lep_dyn");
    assert!(
        !row.replay.is_empty(),
        "the reference carries no replay events"
    );
    let replay = replay_scales(
        &row.lines,
        &madgraph_dir().join(&row.run_card),
        &row.replay,
        AlphaSPrinting::MadEvent,
    );
    println!(
        "{} MadEvent events replayed: {} reproduce SCALUP in one of their configurations \
         (worst relative {:.2e}), AQCDUP worst |Δ| {:.2e}",
        replay.events,
        replay.events - replay.missed.len(),
        replay.worst_scale,
        replay.worst_alpha,
    );
    assert!(
        replay.missed.is_empty(),
        "events whose SCALUP no configuration reproduces: {:?}",
        replay.missed
    );
    // Half a unit in the seventh printed digit of a value near 0.1, with a
    // hundredth of it for the last-ulp noise of both runnings.
    assert!(
        replay.worst_alpha < 5.05e-8,
        "AQCDUP off by {:.2e}",
        replay.worst_alpha
    );
}

/// How an event file's `AQCDUP` relates to `αs(μR)`.
#[derive(Clone, Copy, PartialEq)]
enum AlphaSPrinting {
    /// MadEvent's: `αs(μR)` times `π / 3.1415926`, from `AQCDUP`'s `G²/(4·3.1415926)`.
    MadEvent,
    /// This crate's: `αs(μR)` itself.
    Exact,
}

/// What replaying a sample's scales through the prescription found.
struct ScaleReplay {
    events: usize,
    /// Events whose `SCALUP` no configuration of their own flavour group
    /// reproduces.
    missed: Vec<usize>,
    /// Of those, the ones some *other* group's configuration reproduces — the
    /// signature of a scale clustered in the wrong group's merge graph.
    elsewhere: usize,
    /// Events per flavour group, and of those how many were missed.
    per_group: Vec<(usize, usize)>,
    worst_scale: f64,
    worst_alpha: f64,
}

/// Replay each event's momenta through the card's scale prescription in every
/// configuration of the flavour group its flavours belong to: one configuration
/// has to return the event's `SCALUP` to `1e-6` relative, and `αs` of that
/// configuration's `μR` (as `printing` says the file writes it) is compared
/// with its `AQCDUP`.
///
/// The group is named by the event's own flavours, which is the label the file
/// carries; the configuration is not in the file, so any of the group's is
/// accepted. An event whose beams carry the group's partons exchanged is read
/// in the group's own flavour order, rotated by π about the x axis with its
/// beams swapped — the orientation MadEvent clusters a mirrored point in.
fn replay_scales(
    lines: &str,
    run_card: &Path,
    events: &[ReplayEvent],
    printing: AlphaSPrinting,
) -> ScaleReplay {
    let card = format!("import model sm\n{}\n", lines.replace(';', "\n"));
    let parsed = parse_proc_card(&card, &Default::default()).expect("card");
    let model = sm_model(SMRestrict::Default);
    let evaluated = EvaluatedModel::from_model(model.clone());
    let rc = RunCard::parse(&std::fs::read_to_string(run_card).unwrap()).unwrap();
    let sets = generate_from_proc_card(&parsed, &model).expect("diagrams");
    let groups = derive_flavor_groups(sets, &model, &evaluated, &rc).expect("groups");
    let amps: Vec<_> = groups
        .groups()
        .iter()
        .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), &evaluated))
        .collect();
    let set = PdfSet::load(&pdf_dir().join(PDF_SET), PDF_SET).expect("PDF set");
    let pdf = set.member(0).expect("member 0");
    let mut integ = ProtonIntegrand::new_with_maps(
        &groups,
        &amps,
        &evaluated,
        &pdf,
        rc.ebeam1 + rc.ebeam2,
        rc.dsqrt_q2fact1,
        MapOptions::default(),
    )
    .expect("integrand");
    integ
        .use_run_card_scales(&model, &evaluated, &rc, Some(&set.info.alpha_s))
        .expect("scales");
    let source = integ.scale_source();
    let alpha_s = source.alpha_s().expect("a running coupling");
    #[allow(clippy::approx_constant)]
    const TRUNCATED_PI: f64 = 3.1415926;
    let n_groups = groups.groups().len();
    // The scale in a group's configuration that reproduces `scalup`, if any.
    let hit = |group: usize, incoming: &[[f64; 4]], outgoing: &[[f64; 4]], scalup: f64| {
        let n_configs = groups.groups()[group].evaluator().n_configs();
        (0..n_configs).find_map(|c| {
            let scales = source
                .scales(
                    [incoming[0], incoming[1]],
                    outgoing,
                    SampledChannel { group, channel: c },
                )
                .ok()?;
            let rel = (scales.mu_f[0].max(scales.mu_f[1]) / scalup - 1.0).abs();
            (rel < 1e-6).then_some((rel, scales.mu_r))
        })
    };
    let mut replay = ScaleReplay {
        events: events.len(),
        missed: Vec::new(),
        elsewhere: 0,
        per_group: vec![(0, 0); n_groups],
        worst_scale: 0.0,
        worst_alpha: 0.0,
    };
    for (k, event) in events.iter().enumerate() {
        let incoming: Vec<[f64; 4]> = event
            .incoming
            .iter()
            .map(|p| [p[1], p[2], p[3], p[4]])
            .collect();
        let outgoing: Vec<[f64; 4]> = event
            .outgoing
            .iter()
            .map(|p| [p[1], p[2], p[3], p[4]])
            .collect();
        let pdg_in = [event.incoming[0][0] as i32, event.incoming[1][0] as i32];
        let pdg_out: Vec<i32> = event.outgoing.iter().map(|p| p[0] as i32).collect();
        let (group, exchanged) = groups
            .groups()
            .iter()
            .enumerate()
            .find_map(|(gi, g)| {
                g.members().iter().find_map(|m| {
                    if m.outgoing != pdg_out {
                        None
                    } else if m.incoming == pdg_in {
                        Some((gi, false))
                    } else if m.incoming == [pdg_in[1], pdg_in[0]] {
                        Some((gi, true))
                    } else {
                        None
                    }
                })
            })
            .expect("the event's subprocess is one of the card's");
        // An event whose beams carry the member's partons the other way round is
        // clustered as MadEvent clusters a mirrored point: rotated by π about x
        // with the beams exchanged, so the group's own flavour order reads it.
        let (incoming, outgoing) = if exchanged {
            let rotate = |p: &[f64; 4]| [p[0], p[1], -p[2], -p[3]];
            (
                vec![rotate(&incoming[1]), rotate(&incoming[0])],
                outgoing.iter().map(rotate).collect(),
            )
        } else {
            (incoming, outgoing)
        };
        replay.per_group[group].0 += 1;
        match hit(group, &incoming, &outgoing, event.scalup) {
            Some((rel, mu_r)) => {
                replay.worst_scale = replay.worst_scale.max(rel);
                let aqcdup = match printing {
                    AlphaSPrinting::MadEvent => {
                        alpha_s.eval(mu_r) * std::f64::consts::PI / TRUNCATED_PI
                    }
                    AlphaSPrinting::Exact => alpha_s.eval(mu_r),
                };
                replay.worst_alpha = replay.worst_alpha.max((aqcdup - event.aqcdup).abs());
            }
            None => {
                replay.missed.push(k);
                replay.per_group[group].1 += 1;
                if (0..n_groups)
                    .any(|g| g != group && hit(g, &incoming, &outgoing, event.scalup).is_some())
                {
                    replay.elsewhere += 1;
                }
            }
        }
    }
    replay
}

/// The replay form of one of this crate's own event files: the incoming and
/// outgoing legs in record order, and the event's `SCALUP` and `AQCDUP`.
fn replay_events(files: &[LheFile]) -> Vec<ReplayEvent> {
    let leg = |p: &vibegraph::lhef::record::LheParticle| {
        [
            f64::from(p.pdg),
            p.momentum[0],
            p.momentum[1],
            p.momentum[2],
            p.momentum[3],
        ]
    };
    files
        .iter()
        .flat_map(|f| &f.events)
        .map(|e| ReplayEvent {
            incoming: e
                .particles
                .iter()
                .filter(|p| p.status == STATUS_INCOMING)
                .map(leg)
                .collect(),
            outgoing: e
                .particles
                .iter()
                .filter(|p| p.status == STATUS_OUTGOING)
                .map(leg)
                .collect(),
            scalup: e.scale,
            aqcdup: e.alpha_qcd,
        })
        .collect()
}

/// Events this crate generates per card for [`our_own_events_replay_in_their_own_flavour_group`].
const OWN_REPLAY_EVENTS: usize = 2_000;

/// This crate's own generated events through its own scale prescription: every
/// event's `SCALUP` has to be reproduced by a configuration of the flavour
/// group the event is labelled with, and `AQCDUP` has to be `αs` of that
/// configuration's `μR`.
///
/// This is the per-event statement that a point's scale is clustered in the
/// group whose matrix element its term evaluates, and that the record carries
/// that group's scales — MadEvent's rule, which integrates each subprocess group
/// on its own and clusters in its configurations. Two cards whose groups
/// cluster apart: the decayed `p p > t t~` (`q q̄` takes √(m_T(t)·m_T(t̄)),
/// `g g` the larger m_T) and `p p > l+ l- j` at the dynamical scale (six groups,
/// `q q̄ → ℓℓg` against `q g → ℓℓq`). A scale taken in the sampler's group
/// instead reads 253 of 400 on the first card.
///
/// What it cannot see: which configuration inside the group was used, so a
/// configuration draw with the wrong weights passes (the `samples` columns and
/// the σ rows are what see that); the mirrored ordering, whose term takes the
/// same scale as the direct one by construction on both sides; and an event
/// where two groups cluster to the same scale, which is most of `p p > t t~`
/// near threshold.
#[test]
fn our_own_events_replay_in_their_own_flavour_group() {
    let reference = reference();
    let ttx = row(&reference, "pp_ttx_lep_dyn");
    let llj_card = madgraph_dir().join("output/pp_to_llj_dyn/Cards/run_card.dat");
    if !llj_card.is_file() {
        vibegraph::validation::require(
            "our_own_events_replay_in_their_own_flavour_group",
            "a banked run directory",
            "pp_to_llj_dyn",
        );
    }
    let cards: [(&str, String, PathBuf); 2] = [
        (
            "pp_ttx_lep_dyn",
            ttx.lines.clone(),
            madgraph_dir().join(&ttx.run_card),
        ),
        (
            "pp_to_llj_dyn",
            "generate p p > l+ l- j QCD=2 QED=2".to_string(),
            llj_card,
        ),
    ];
    let mut failures = Vec::new();
    for (name, lines, run_card) in cards {
        let tmp = tempfile::tempdir().unwrap();
        let card = tmp.path().join("proc_card.dat");
        let body: String = lines.split(';').map(|l| format!("{l}\n")).collect();
        std::fs::write(&card, format!("import model sm\n{body}")).unwrap();
        let (_, _, artifact) = integrate(tmp.path(), &card, &run_card, INTEGRATION_SEED, "1e-2");
        let lhe = tmp.path().join("events.lhe");
        vibegraph(&[
            "generate".as_ref(),
            artifact.as_os_str(),
            card.as_os_str(),
            "--run-card".as_ref(),
            run_card.as_os_str(),
            "--pdf-dir".as_ref(),
            pdf_dir().as_os_str(),
            "--seed".as_ref(),
            GEN_SEEDS[0].to_string().as_ref(),
            "--nevents".as_ref(),
            OWN_REPLAY_EVENTS.to_string().as_ref(),
            "--strategy".as_ref(),
            "buffer".as_ref(),
            "-o".as_ref(),
            lhe.as_os_str(),
        ]);
        let file =
            LheFile::parse(&std::fs::read_to_string(&lhe).unwrap()).expect("our own file parses");
        let events = replay_events(std::slice::from_ref(&file));
        let replay = replay_scales(&lines, &run_card, &events, AlphaSPrinting::Exact);
        println!(
            "[{name}] {} of our events replayed: {} reproduce SCALUP in their own group \
             (worst relative {:.2e}), {} of the rest in another group's; AQCDUP worst |Δ| \
             {:.2e}; per group (events, missed): {:?}",
            replay.events,
            replay.events - replay.missed.len(),
            replay.worst_scale,
            replay.elsewhere,
            replay.worst_alpha,
            replay.per_group,
        );
        if replay.events != OWN_REPLAY_EVENTS {
            failures.push(format!("[{name}] {} events written", replay.events));
        }
        if !replay.missed.is_empty() {
            failures.push(format!(
                "[{name}] {} events whose SCALUP no configuration of their own group \
                 reproduces ({} of them another group's): first {:?}",
                replay.missed.len(),
                replay.elsewhere,
                &replay.missed[..replay.missed.len().min(10)]
            ));
        }
        // Half a unit in the ninth printed digit of a value near 0.1, doubled
        // for the momenta's own printed rounding reaching μR.
        if replay.worst_alpha >= 1e-9 {
            failures.push(format!("[{name}] AQCDUP off by {:.2e}", replay.worst_alpha));
        }
        // More than one group has to have been exercised, or the statement is
        // vacuous.
        let populated = replay.per_group.iter().filter(|(n, _)| *n > 0).count();
        if populated < 2 {
            failures.push(format!(
                "[{name}] only {populated} flavour group carries events"
            ));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}
