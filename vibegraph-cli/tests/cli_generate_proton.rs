//! End-to-end gate on `vibegraph generate` at proton beams: the banked
//! fixed-scale `p p > l+ l- j` cards integrated, replayed into an unweighted Les
//! Houches file, and read back.
//!
//! Gated behind `extended-validation`; needs the fetched PDF set and the banked
//! MadGraph run:
//!
//!     pixi run -e madgraph fetch-pdf
//!     pixi run -e madgraph build-diagrams
//!     cargo test -p vibegraph --features extended-validation --test cli_generate_proton
//!
//! # What this gate checks, and what it provably cannot
//!
//! The file is read back with this crate's own `lhef::parse`, so — exactly as for
//! the fixed-energy gate — a **self-consistently wrong format** is invisible here:
//! our reader and our writer share their assumptions. What carries the format
//! evidence is `validate_lhef`'s byte-for-byte round trip of MadGraph's own banked
//! files, which now includes this very run.
//!
//! What *is* compared against MadGraph is the content the hadronic path introduced
//! and the fixed-energy one never had:
//!
//! * **which flavours an event may be labelled with** — every emitted `IDUP` row
//!   must be one of the 24 subprocesses MadGraph's own `leshouche.inc` lists, or
//!   that subprocess with its two beams exchanged, which is the ordering the
//!   enumeration does not produce and the mirror term supplies;
//! * **the colour lines on a coloured initial state** — every emitted event's
//!   connectivity must be one MadGraph's own events exhibit for the same
//!   arrangement of gluon, quark, antiquark and leptons.
//!
//! Neither says anything about *how often* each flavour is drawn. The rule behind
//! the frequencies is pinned in `proton`'s unit tests; comparing realised
//! frequencies against MadGraph's sample is a distribution-level question and is
//! deliberately left to a later validation pass, together with a run through a real
//! shower.
//!
//! It is also blind, as every self-comparison is, to anything the integrand gets
//! wrong: a wrong matrix element, cut or sampler is replayed faithfully into the
//! file and agrees with itself. `validate_hadronic` and `amplitude_oracle`
//! cover those.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use vibegraph::artifact::{ChannelKey, IntegrateArtifact};
use vibegraph::lhef::parse::LheFile;
use vibegraph::lhef::record::{LheEvent, WeightStrategy, STATUS_INCOMING, STATUS_OUTGOING};

/// The banked run this replays: MadGraph's own cards, its own PDF set, and its own
/// event file as the flavour and colour oracle.
const RUN: &str = "pp_to_llj_fixed";
const PROCESS: &str = "p p > l+ l- j QCD=2 QED=2";
const PDF_SET: &str = "NNPDF23_lo_as_0130_qed";

/// The run card fixes both scales at `m_Z`, so every event reports the same
/// `SCALUP` and the same `AQCDUP`.
const SCALE: f64 = 91.188;
/// How far `AQCDUP` may sit from `αs(m_Z)` off the set's own tabulation.
///
/// The bound is MadGraph's own printing budget for the field, not a chosen number:
/// `%e` gives seven significant digits, so half a unit in the last one is `5e-8`
/// absolute on a coupling of `0.13`, and anything inside that is invisible in
/// MadGraph's file. The observed gap is `1.1e-7` relative — `validate_alphas`
/// reports the same number as `0.281` of this budget for this run — and comes from
/// the two interpolations of the same `αs` grid. (MadGraph's field additionally
/// carries a `+1.7e-8` bias from a `π` truncated to eight digits in `unwgt.f`,
/// which the reference below undoes.)
const ALPHA_S_TOLERANCE: f64 = 4e-7;

/// Integration budget, chosen by a scan rather than for the wall clock alone.
///
/// Two separate things degrade below it. The per-channel `w_max` a frozen scan
/// finds is an extremum estimate on that channel's own share of the budget, and the
/// cross section it leaves *above* itself falls with the budget: 3.2% at 30 000,
/// 1.5% at 100 000, 0.8% here, 0.5% at 600 000 — where a gated fixed-energy process
/// shows 3e-4. Those events are kept at a weight above one, so the estimator stays
/// unbiased and what the tail costs is the sample's spread. Separately, the banked
/// σ itself is still rising at 100 000 (see the cross-section bound below), so a
/// comparison against it there measures the integration's convergence, not the
/// sampling.
const NEVAL: &str = "300000";
const NITER: &str = "8";
const NEVENTS: usize = 20_000;
const SEED: &str = "20260731";

/// External legs of `p p → ℓ⁺ℓ⁻ j`.
const N_EXT: usize = 5;
const N_IN: usize = 2;

/// How far the sample's own cross section may sit from the integration's,
/// relatively.
///
/// The bound is a measurement, not a `1/√N`: over five seeds at this budget the
/// deviations are `{−0.36, −0.14, −0.62, +0.47, +0.29}%`, and the spread is set by
/// the events above their channel's `w_max`, not by the event count. The same sweep
/// at `neval = 100 000` gives `{−0.19, +2.29, +1.67, +1.37, +1.11}%` — four of five
/// on the *same side*, because the sample's estimator is a single pass over the
/// frozen grids and so does not inherit VEGAS's `1/σ²` combination of iterations,
/// which at that budget still has the banked σ about 1% low. This comparison is
/// therefore also a read on the integration's convergence, and the bound sits above
/// the converged spread and below what an unconverged budget produces.
const SIGMA_MAX_REL: f64 = 0.015;

fn output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/output")
}

fn pdf_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/pdf")
}

fn run_dir() -> PathBuf {
    output_dir().join(RUN)
}

fn banked_present() -> bool {
    run_dir().join("Cards/run_card.dat").is_file() && pdf_dir().join(PDF_SET).is_dir()
}

struct Run {
    _tmp: tempfile::TempDir,
    dir: PathBuf,
    proc_card: PathBuf,
    run_card: PathBuf,
    artifact_path: PathBuf,
    artifact: IntegrateArtifact,
}

/// The one `vibegraph integrate` run every case here replays.
fn integrated() -> &'static Run {
    static ONCE: OnceLock<Run> = OnceLock::new();
    ONCE.get_or_init(integrate)
}

fn integrate() -> Run {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().to_path_buf();
    let proc_card = dir.join("proc_card.dat");
    std::fs::write(&proc_card, format!("import model sm\ngenerate {PROCESS}\n")).unwrap();
    // MadGraph's own card, byte for byte, so both generators read one file.
    let run_card = run_dir().join("Cards/run_card.dat");

    let out = dir.join("out");
    let output = Command::new(env!("CARGO_BIN_EXE_vibegraph"))
        .arg("integrate")
        .arg(&proc_card)
        .arg("--run-card")
        .arg(&run_card)
        .arg("--out")
        .arg(&out)
        .arg("--pdf-dir")
        .arg(pdf_dir())
        .args(["--neval", NEVAL, "--niter", NITER, "--seed", SEED])
        .output()
        .expect("spawn vibegraph");
    assert!(
        output.status.success(),
        "vibegraph integrate failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    eprint!("{}", String::from_utf8_lossy(&output.stdout));

    let artifact_path = out.join("grid.bin.zst");
    let artifact = IntegrateArtifact::read_from_path(&artifact_path).expect("reload artifact");
    assert_eq!(artifact.pdf_set, PDF_SET);
    assert!(
        artifact
            .channels
            .iter()
            .all(|c| matches!(c.key, ChannelKey::GroupDiagram { .. })),
        "the hadronic path must bank (group, diagram) channels"
    );
    Run {
        _tmp: tmp,
        dir,
        proc_card,
        run_card,
        artifact_path,
        artifact,
    }
}

impl Run {
    fn generate_cmd(&self, name: &str) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_vibegraph"));
        cmd.arg("generate")
            .arg(&self.artifact_path)
            .arg(&self.proc_card)
            .arg("--run-card")
            .arg(&self.run_card)
            .arg("--pdf-dir")
            .arg(pdf_dir())
            .arg("--seed")
            .arg(SEED)
            .arg("-o")
            .arg(self.dir.join(name))
            .arg("--force");
        cmd
    }

    fn generate(&self, nevents: usize, name: &str) -> LheFile {
        let out = self
            .generate_cmd(name)
            .arg("--nevents")
            .arg(nevents.to_string())
            .output()
            .expect("spawn vibegraph");
        assert!(
            out.status.success(),
            "vibegraph generate failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        eprint!("{}", String::from_utf8_lossy(&out.stdout));
        let text = std::fs::read_to_string(self.dir.join(name)).expect("read the event file");
        LheFile::parse(&text).expect("our own file parses")
    }
}

/// MadGraph's own banked events for this run.
fn banked_events() -> &'static LheFile {
    static ONCE: OnceLock<LheFile> = OnceLock::new();
    ONCE.get_or_init(|| {
        let lhe = run_dir().join("Events/run_01/unweighted_events.lhe.gz");
        let out = Command::new("gzip")
            .args(["-dc", lhe.to_str().unwrap()])
            .output()
            .expect("gzip -dc");
        assert!(out.status.success(), "gzip failed on {}", lhe.display());
        let text = String::from_utf8(out.stdout).expect("the banked files are ASCII");
        LheFile::parse(&text).expect("MadGraph's own file parses")
    })
}

/// Every `IDUP` row MadGraph's `leshouche.inc` lists for this run, in both beam
/// orderings — the complete set of external flavour assignments an event of this
/// process may carry.
///
/// This is read from the generated Fortran rather than from MadGraph's event
/// sample, so a flavour its 10 000 events happen not to contain is still in the
/// oracle and a flavour it could never produce is still out of it.
fn allowed_flavors() -> &'static BTreeSet<Vec<i32>> {
    static ONCE: OnceLock<BTreeSet<Vec<i32>>> = OnceLock::new();
    ONCE.get_or_init(|| {
        let mut out = BTreeSet::new();
        for entry in std::fs::read_dir(run_dir().join("SubProcesses")).expect("SubProcesses") {
            let dir = entry.expect("dir entry").path();
            let file = dir.join("leshouche.inc");
            if !file.is_file() {
                continue;
            }
            let text = std::fs::read_to_string(&file).expect("leshouche.inc");
            for line in text.lines() {
                let Some(rest) = line.trim().strip_prefix("DATA (IDUP(") else {
                    continue;
                };
                let Some(body) = rest.split('/').nth(1) else {
                    continue;
                };
                let row: Vec<i32> = body
                    .split(',')
                    .filter_map(|f| f.trim().parse::<i32>().ok())
                    .collect();
                assert_eq!(
                    row.len(),
                    N_EXT,
                    "unexpected IDUP row in {}",
                    file.display()
                );
                let mut exchanged = row.clone();
                exchanged.swap(0, 1);
                out.insert(row);
                out.insert(exchanged);
            }
        }
        assert!(
            !out.is_empty(),
            "no IDUP rows found under {}",
            run_dir().join("SubProcesses").display()
        );
        out
    })
}

/// A PDG code reduced to the role it plays in the colour flow. Two events with the
/// same roles on the same legs must carry the same colour connectivity, whatever
/// generation or lepton flavour they happen to be.
fn role(pdg: i32) -> i32 {
    match pdg {
        21 => 21,
        q if (1..=6).contains(&q) => 1,
        q if (-6..=-1).contains(&q) => -1,
        _ => 0,
    }
}

/// The partonic invariant of an event, from its own incoming momenta.
fn shat(event: &LheEvent) -> f64 {
    let mut sum = [0.0f64; 4];
    for p in &event.particles[..N_IN] {
        for (acc, v) in sum.iter_mut().zip(p.momentum) {
            *acc += v;
        }
    }
    sum[0] * sum[0] - sum[1] * sum[1] - sum[2] * sum[2] - sum[3] * sum[3]
}

fn roles(event: &LheEvent) -> Vec<i32> {
    event.particles.iter().map(|p| role(p.pdg)).collect()
}

/// The colour lines an event carries: the `(leg, slot)` endpoints sharing a label,
/// with the label itself discarded because any consistent relabelling is the same
/// event.
fn connectivity(event: &LheEvent) -> Vec<Vec<(usize, usize)>> {
    event.color_connectivity()
}

/// One event's flavour roles paired with its colour connectivity.
type ColourPattern = (Vec<i32>, Vec<Vec<(usize, usize)>>);

/// `(roles, connectivity)` pairs MadGraph's own events exhibit.
fn banked_colour_patterns() -> &'static BTreeSet<ColourPattern> {
    static ONCE: OnceLock<BTreeSet<ColourPattern>> = OnceLock::new();
    ONCE.get_or_init(|| {
        banked_events()
            .events
            .iter()
            .map(|e| (roles(e), connectivity(e)))
            .collect()
    })
}

/// The initial states MadGraph's own sample contains, as `(roles[0], roles[1])`.
fn banked_initial_roles() -> BTreeSet<(i32, i32)> {
    banked_events()
        .events
        .iter()
        .map(|e| {
            let r = roles(e);
            (r[0], r[1])
        })
        .collect()
}

/// The whole gate: generate, read back, and check every event against MadGraph's
/// own flavour table and colour patterns.
#[test]
fn generated_proton_events_are_coherent_and_madgraph_labelled() {
    if !banked_present() {
        vibegraph::validation::require(
            "generated_proton_events_are_coherent_and_madgraph_labelled",
            "the banked MadGraph run and the fetched PDF set",
            RUN,
        );
    }
    let run = integrated();
    let file = run.generate(NEVENTS, "events.lhe");
    assert_eq!(file.events.len(), NEVENTS);

    // The `<init>` block is the hadronic one: proton beams at the run card's own
    // energies, and the LHAPDF id in `PDFSUP`, as MadGraph writes it.
    let rc = &run.artifact.run_card;
    assert_eq!(file.init.beam_pdg, [2212, 2212]);
    assert_eq!(file.init.beam_energy, [rc.ebeam1, rc.ebeam2]);
    assert_eq!(file.init.pdf_group, [0, 0]);
    assert_eq!(file.init.pdf_set, [rc.lhaid as i32; 2]);
    assert_eq!(
        file.init.weight_strategy,
        WeightStrategy::MeanCrossSectionPb
    );
    assert_eq!(banked_events().init.beam_pdg, file.init.beam_pdg);
    assert_eq!(banked_events().init.pdf_set, file.init.pdf_set);

    let e_beam = [rc.ebeam1, rc.ebeam2];
    // Every tolerance below is relative to the event's own scale, not to a fixed
    // one: a hadronic event's momenta run from the lepton cuts to the beams, and the
    // format carries eleven significant digits whatever the magnitude.
    let allowed = allowed_flavors();
    let patterns = banked_colour_patterns();
    let mut seen_flavors: BTreeSet<Vec<i32>> = BTreeSet::new();
    let mut seen_initial: BTreeSet<(i32, i32)> = BTreeSet::new();
    let (mut worst_balance, mut worst_mass, mut worst_alpha_s) = (0.0f64, 0.0f64, 0.0f64);
    let mut weight_sum = 0.0f64;
    let mut weight_sq = 0.0f64;

    for (index, event) in file.events.iter().enumerate() {
        assert_eq!(event.nup(), N_EXT, "event {index}: NUP");
        weight_sum += event.weight;
        weight_sq += event.weight * event.weight;

        // Both scales are fixed by the card, so every event reports the same ones,
        // and `AQCDUP` is the coupling off the PDF set's own tabulation.
        assert_eq!(event.scale, SCALE, "event {index}: SCALUP");
        worst_alpha_s = worst_alpha_s.max((event.alpha_qcd / banked_alpha_s() - 1.0).abs());

        let flavors: Vec<i32> = event.particles.iter().map(|p| p.pdg).collect();
        assert!(
            allowed.contains(&flavors),
            "event {index}: the flavour assignment {flavors:?} is not one MadGraph's \
             leshouche.inc lists for this process, in either beam ordering"
        );
        seen_flavors.insert(flavors);
        let r = roles(event);
        seen_initial.insert((r[0], r[1]));
        assert!(
            patterns.contains(&(r.clone(), connectivity(event))),
            "event {index}: colour lines {:?} on legs {r:?} are a pattern MadGraph's own events \
             for this run never carry",
            connectivity(event)
        );

        for (leg, p) in event.particles.iter().enumerate() {
            let incoming = leg < N_IN;
            assert_eq!(
                p.status,
                if incoming {
                    STATUS_INCOMING
                } else {
                    STATUS_OUTGOING
                },
                "event {index} leg {leg}: ISTUP"
            );
            assert_eq!(
                p.mothers,
                if incoming { [0, 0] } else { [1, N_IN as i32] },
                "event {index} leg {leg}: MOTHUP"
            );
            let [en, px, py, pz] = p.momentum;
            if incoming {
                // A beam parton runs down the axis carrying a fraction of its own
                // beam's energy; a writer that put the partonic-CM momenta here, or
                // crossed the two beams, fails on one of these.
                assert_eq!(
                    [px, py],
                    [0.0, 0.0],
                    "event {index} leg {leg}: beam is off axis"
                );
                let sign = if leg == 0 { 1.0 } else { -1.0 };
                assert!(
                    (pz - sign * en).abs() < 1e-9 * en,
                    "event {index} leg {leg}: beam parton is not on its own side of the axis"
                );
                assert!(
                    en > 0.0 && en <= e_beam[leg],
                    "event {index} leg {leg}: energy {en} outside (0, {}]",
                    e_beam[leg]
                );
            }
            let m2 = en * en - px * px - py * py - pz * pz;
            worst_mass = worst_mass.max((m2 - p.mass * p.mass).abs() / shat(event));
        }

        let e_in: f64 = event.particles[..N_IN].iter().map(|p| p.momentum[0]).sum();
        for component in 0..4 {
            let balance: f64 = event
                .particles
                .iter()
                .enumerate()
                .map(|(leg, p)| {
                    let v = p.momentum[component];
                    if leg < N_IN {
                        v
                    } else {
                        -v
                    }
                })
                .sum();
            worst_balance = worst_balance.max(balance.abs() / e_in);
        }
    }

    // Both beam orderings have to reach the file. Only one of each unordered
    // initial state is enumerated, so a sample carrying only those would mean the
    // mirrored ordering never reached an event record — which no cross section can
    // see, since both orderings are summed into the same number.
    let banked_initial = banked_initial_roles();
    assert_eq!(
        seen_initial, banked_initial,
        "the sample's initial-state arrangements differ from MadGraph's"
    );

    let mean = weight_sum / NEVENTS as f64;
    let variance = weight_sq / NEVENTS as f64 - mean * mean;
    let sample_error = (variance / NEVENTS as f64).sqrt();
    let declared = file.init.processes[0].xsec_pb;
    let integrated = run.artifact.sigma_pb;
    let rel = mean / integrated - 1.0;

    eprintln!(
        "-- {RUN} -- {NEVENTS} events, {} distinct flavour assignments over {} initial-state \
         arrangements\n  \
         sigma(sample) = {mean:.4} ± {sample_error:.4} pb vs integration {integrated:.4} ± \
         {:.4} pb ({:+.3}%)\n  \
         momentum balance <= {worst_balance:.2e} of the incoming energy, |p^2 - m^2| <= \
         {worst_mass:.2e} of s-hat, AQCDUP within {worst_alpha_s:.2e} of the grid",
        seen_flavors.len(),
        seen_initial.len(),
        run.artifact.sigma_err_pb,
        100.0 * rel,
    );

    assert!(worst_balance < 1e-9, "momenta do not balance");
    assert!(worst_mass < 1e-8, "legs are not on shell");
    assert!(
        worst_alpha_s < ALPHA_S_TOLERANCE,
        "AQCDUP is not the PDF grid's coupling"
    );
    // `IDWTUP = -4` says the cross section is the mean of the event weights, and the
    // buffered writer declares the sample's own.
    assert!(
        (mean / declared - 1.0).abs() < 1e-6,
        "mean XWGTUP {mean:.6e} vs declared XSECUP {declared:.6e}"
    );
    assert!(
        rel.abs() < SIGMA_MAX_REL,
        "the sample's cross section is {:.3}% from the integration's",
        100.0 * rel
    );
    // 24 subprocesses, and the mirrored ordering of each initial state whose two
    // partons differ, is what the decomposition sums; a sample that reached only
    // some of them would still integrate to the same number.
    assert!(
        seen_flavors.len() >= 24,
        "only {} distinct flavour assignments reached the file",
        seen_flavors.len()
    );
}

/// `αs(m_Z)` as MadGraph's own banked events report it, with its `π` truncation
/// undone — the value `AQCDUP` should carry.
fn banked_alpha_s() -> f64 {
    static ONCE: OnceLock<f64> = OnceLock::new();
    #[allow(clippy::approx_constant)]
    const TRUNCATED_PI: f64 = 3.1415926;
    *ONCE.get_or_init(|| {
        let alpha = banked_events().events[0].alpha_qcd;
        assert!(alpha > 0.0, "the banked events carry no AQCDUP");
        alpha * TRUNCATED_PI / std::f64::consts::PI
    })
}

/// A `generate` run reads the parton distributions by *name*, and the run card
/// pins only the LHAPDF id, so an artifact and a command line can agree on every
/// card and still disagree about which tabulation trained the grids. The refusal is
/// what closes that, and the matching case is exercised by the gate above.
#[test]
fn a_different_pdf_set_is_refused() {
    if !banked_present() {
        vibegraph::validation::require(
            "a_different_pdf_set_is_refused",
            "the banked MadGraph run and the fetched PDF set",
            RUN,
        );
    }
    let run = integrated();
    let out = run
        .generate_cmd("wrong-pdf.lhe")
        .arg("--nevents")
        .arg("10")
        .arg("--pdf-set")
        .arg("CT14lo")
        .output()
        .expect("spawn vibegraph");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "a foreign PDF set was accepted");
    assert!(
        stderr.contains("PDF set") && stderr.contains(PDF_SET) && stderr.contains("CT14lo"),
        "the refusal does not name the two sets:\n{stderr}"
    );
}

/// A dynamical scale on this process stays refused, in both places a user can meet
/// one — but no longer for the same reason.
///
/// `generate` never reaches the scale prescription: the card it is handed does not
/// match the one that trained the grids. `integrate` now computes the clustering
/// scale, and meets the *next* limit instead: the run card's PDF set tabulates
/// `αs` to 10 TeV while a per-event scale on a 13 TeV collider can exceed it.
/// LHAPDF extrapolates past its own table and this crate does not, so the run
/// stops at setup naming the range rather than evaluating off the end of it on
/// whichever events happen to reach past it.
///
/// The card is the banked one with its three `fixed_*_scale` switches turned off,
/// so nothing but the scale prescription differs and the refusals cannot be coming
/// from something else about it.
#[test]
fn a_dynamical_scale_card_is_still_refused() {
    if !banked_present() {
        vibegraph::validation::require(
            "a_dynamical_scale_card_is_still_refused",
            "the banked MadGraph run and the fetched PDF set",
            RUN,
        );
    }
    let run = integrated();
    let dynamical = run.dir.join("dynamical_run_card.dat");
    std::fs::write(&dynamical, dynamical_card()).expect("write the dynamical card");

    let out = Command::new(env!("CARGO_BIN_EXE_vibegraph"))
        .arg("generate")
        .arg(&run.artifact_path)
        .arg(&run.proc_card)
        .arg("--run-card")
        .arg(&dynamical)
        .arg("--pdf-dir")
        .arg(pdf_dir())
        .arg("--nevents")
        .arg("10")
        .arg("-o")
        .arg(run.dir.join("dynamical.lhe"))
        .arg("--force")
        .output()
        .expect("spawn vibegraph");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "a dynamical-scale card was accepted");
    assert!(
        stderr.contains("run card `fixed_ren_scale`"),
        "the refusal does not name the switch that was turned off:\n{stderr}"
    );

    // And the card that would have to produce such an artifact is itself refused,
    // with the missing capability spelled out rather than silently approximated.
    let out: Output = Command::new(env!("CARGO_BIN_EXE_vibegraph"))
        .arg("integrate")
        .arg(&run.proc_card)
        .arg("--run-card")
        .arg(&dynamical)
        .arg("--out")
        .arg(run.dir.join("dynamical-out"))
        .arg("--pdf-dir")
        .arg(pdf_dir())
        .args(["--neval", "2000", "--niter", "2"])
        .output()
        .expect("spawn vibegraph");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "a dynamical-scale card was integrated"
    );
    assert!(
        stderr.contains("alpha_s") && stderr.contains("10000"),
        "the integration's refusal does not name the tabulated alpha_s range:\n{stderr}"
    );
}

/// The banked run card with its three fixed-scale switches turned off, and nothing
/// else touched.
fn dynamical_card() -> String {
    let text = std::fs::read_to_string(run_dir().join("Cards/run_card.dat")).expect("run card");
    let switches = ["fixed_ren_scale", "fixed_fac_scale1", "fixed_fac_scale2"];
    let mut turned_off = 0;
    let out: String =
        text.lines()
            .map(|line| {
                match switches.iter().find(|name| {
                    line.contains(&format!("= {name}")) && !line.trim().starts_with('#')
                }) {
                    Some(name) => {
                        turned_off += 1;
                        format!("  False = {name}\n")
                    }
                    None => format!("{line}\n"),
                }
            })
            .collect();
    assert_eq!(
        turned_off,
        switches.len(),
        "the banked run card no longer spells its fixed-scale switches as expected"
    );
    out
}
