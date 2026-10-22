//! The `samples` category at proton beams: the shipped binary's own unweighted
//! events against MadGraph's banked ones.
//!
//! The fixed-beam rows are compared in `validate_samples` inside the library. A
//! hadron-collider row cannot be: the event a flavour group emits is assembled by
//! the `generate` command — the flavour drawn from the parton luminosities, the
//! record relabelled onto that flavour, both beam orderings — and that assembly
//! lives in the binary. So these rows run the binary: one `integrate`, then one
//! `generate` per seed off the frozen grids, and the emitted `.lhe` is read back
//! and compared through the same
//! [`validation::samples`](vibegraph::validation::samples) machinery the library
//! rows use.
//!
//! Five rows: `pp_to_llj_fixed`, a three-body final state with an electroweak
//! core; its scale-moved twin `pp_to_llj_dyn`, same generate line on the
//! dynamical-scale card; `pp_to_bb_fixed`, a two-body one with no electroweak
//! core; `pp_to_ll`, measured once per committed Drell-Yan card; and
//! `pp_to_jj`, the purely hadronic dijet row.
//!
//! # What this adds over the fixed-beam rows
//!
//! The **flavour-group frequencies**. Which subprocess an event is labelled with
//! is drawn `∝ luminosity × σ̂`, a rule `cli_generate_proton` checks only for
//! *admissibility* — every emitted flavour assignment is one MadGraph's
//! `leshouche.inc` lists — and never for *frequency*. The χ² homogeneity on the
//! flavour column is the first comparison of the realised populations against
//! MadGraph's own sample.
//!
//! And the **incoming legs**, which here are not constants of the process: each
//! beam's energy is a momentum fraction times the run card's, drawn from the
//! parton luminosities, so its column is a weighted KS against MadGraph's own
//! and is the first comparison of the realised `x` spectra. The fixed-beam rows
//! compare the same legs as an equality, since there they are constants.
//!
//! And, on the Drell-Yan pair, **`dσ/dm_ll` in absolute picobarns** down to the
//! card's own threshold ([`the_drell_yan_mass_spectrum_is_binned_against_madgraph`]):
//! the per-row columns below are shape statements, and the σ gates elsewhere are
//! scalars, so a differential normalisation is neither's.
//!
//! # What it provably cannot detect
//!
//! Everything the library rows cannot (correlations between columns, a
//! discrepancy confined to a small tail), plus one of its own: each comparison is
//! against a *single* banked MadGraph run, so a flavour carrying well under a
//! per-mille of the cross section is pooled into the χ²'s residual category rather
//! than compared on its own.

mod common;

use std::io::Read;
use std::path::{Path, PathBuf};

use common::validation_dir;

use flate2::read::MultiGzDecoder;
use vibegraph::lhef::observables::Labelling;
use vibegraph::lhef::parse::LheFile;
use vibegraph::lhef::record::{LheEvent, STATUS_INCOMING, STATUS_OUTGOING};
use vibegraph::runcard::RunCard;
use vibegraph::validation::samples::{compare, labelling_for, Comparison, EventSample, Spectrum};

#[path = "../../vibegraph-lib/tests/common/report.rs"]
mod report;

#[path = "../../vibegraph-lib/tests/common/manifest.rs"]
mod manifest;

#[path = "../../vibegraph-lib/tests/common/leshouche.rs"]
mod leshouche;

use report::{CategoryCount, Chi2Cell, FieldCell, KsCell, SamplesRow, SeedSample, Stopwatch};

/// The PDF set both banked runs were generated with.
const PDF_SET: &str = "NNPDF23_lo_as_0130_qed";

/// Integration budget, the one `cli_generate_proton` measured the sample's own
/// cross section to be converged at, and the one the `b b~` sigma row's budget
/// ladder is flat across.
const NEVAL: &str = "300000";
const NITER: &str = "8";
/// The Drell-Yan budget, the one `validate_hadronic`'s σ gate takes for the same
/// two cards.
const DY_NEVAL: &str = "120000";
const DY_NITER: &str = "12";
const INTEGRATION_SEED: &str = "20260731";
/// Events per generation seed, against MadGraph's banked 10 000.
const NEVENTS: usize = 20_000;
/// Independent generation seeds, replayed off the one set of frozen grids.
const GEN_SEEDS: [u64; 3] = [0x5A_4D_1001, 0x5A_4D_1002, 0x5A_4D_1003];

/// The p-value a column must clear, the same floor the library rows use and
/// chosen there from the same kind of trial count.
const P_FLOOR: f64 = 1e-4;

fn output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/output")
}

fn pdf_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/pdf")
}

fn run_dir(run: &str) -> PathBuf {
    output_dir().join(run)
}

/// MadGraph's banked events for this run, from the named `Events/` subdirectory —
/// MadGraph names it after the run tag it was launched with.
fn banked_sample(run: &str, events: &str) -> EventSample {
    let path = run_dir(run).join(format!("Events/{events}/unweighted_events.lhe.gz"));
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let mut text = String::new();
    // MadGraph concatenates the per-channel event files, so a banked `.lhe.gz` is
    // a *multi-member* gzip stream and a single-member reader stops silently at
    // the first member's end — with a perfectly parseable prefix of the run.
    MultiGzDecoder::new(&bytes[..])
        .read_to_string(&mut text)
        .unwrap_or_else(|e| panic!("decompress {}: {e}", path.display()));
    assert!(
        text.trim_end().ends_with("</LesHouchesEvents>"),
        "{} decompressed to a truncated document",
        path.display()
    );
    EventSample::from_lhe(LheFile::parse(&text).expect("MadGraph's own file parses"))
}

/// One measured cell: which banked run it compares against, which manifest row it
/// is filed under, and the budget the comparison stands on.
struct Row {
    /// The MadGraph work-area directory whose banked run supplies the reference.
    run: &'static str,
    /// The `Events/` subdirectory inside it.
    events: &'static str,
    /// The `validation/manifest.toml` process key the cell belongs to.
    key: &'static str,
    /// Which measurement of that key this is, where a row has more than one.
    variant: Option<&'static str>,
    process: &'static str,
    /// A committed card under `validation/madgraph/`, or `None` to take the run's
    /// own `Cards/run_card.dat`.
    run_card: Option<&'static str>,
    neval: &'static str,
    niter: &'static str,
    mode: &'static str,
    /// Per-event scans run over each generated sample in addition to the
    /// distribution comparison. Each returns how many units it judged and how many
    /// failed, so a clean reading is a recorded measurement rather than silence.
    scans: &'static [Scan],
    /// A registered deviation the banked reference carries, or `None`.
    deviation: Option<&'static MirroredCutDeviation>,
}

/// One per-event scan: `(name, unit, run)`, where `run` returns `(bad, total)`.
type Scan = (
    &'static str,
    &'static str,
    fn(&Row, &EventSample) -> (usize, usize),
);

/// Every leg's `ICOLUP` slots are the ones its own colour rep allows.
///
/// Reference-free: it reads the emitted record against the Les Houches convention
/// itself — a quark's line belongs in `ICOLUP(1)`, an antiquark's in `ICOLUP(2)`, a
/// gluon fills both and a colourless leg neither — so it is a statement about our
/// file with nothing on the other side of it. That is what makes it able to localise
/// a defect rather than only detect one: it once read 4 758 of 80 000 legs, every one
/// an antiquark carrying its line in the colour slot.
///
/// What it cannot detect: a legal but wrong flow, or a wrong flow *frequency*. The
/// pattern-membership scan below and the `ICOLUP` chi-squared are what cover those.
fn scan_slot_legality(_row: &Row, sample: &EventSample) -> (usize, usize) {
    let mut bad = 0;
    let mut total = 0;
    for event in &sample.events {
        total += event.particles.len();
        bad += leshouche::illegal_slots(event).len();
    }
    (bad, total)
}

/// Every emitted event's `(roles, connectivity)` is one MadGraph's own tables admit,
/// at zero tolerance.
///
/// The admissible set is built from `leshouche.inc` — every `P*` directory, every
/// `isproc`, every flavour row, every colour flow, both beam orderings — rather than
/// from MadGraph's 10 000-event sample. A sample-derived set is missing whatever its
/// draws happened not to reach, and an honest event carrying a rare flow would fail
/// against it.
///
/// What it cannot detect: frequencies. Every event could carry the same legal
/// pattern and pass.
fn scan_colour_patterns(row: &Row, sample: &EventSample) -> (usize, usize) {
    let allowed = leshouche::allowed_patterns(&run_dir(row.run)).expect("leshouche tables");
    let mut bad = 0;
    for event in &sample.events {
        let pattern = (leshouche::roles(event), leshouche::connectivity(event));
        if !allowed.contains(&pattern) {
            bad += 1;
        }
    }
    (bad, sample.events.len())
}

/// MadGraph's defect on the unequal-beam row, kept as a registered deviation
/// (user, 2026-10-22): grouped MadEvent 3.7.1 runs `passcuts` once per point,
/// on the sampled momenta at the sampled `cm_rap` (`dsample.f:181`), before
/// `select_grouping` picks the beam ordering (`dsample.f:222`); the mirror
/// flips `CM_RAP` only afterwards (`super_auto_dsig_group_v4.inc:815-829`) and
/// its own `PASSCUTS` (`:835`) is commented out. A mirrored event — the
/// antiquark on the `+z` beam here — is therefore admitted when its leptons'
/// lab rapidities lie within `etal` of `ln(E₁/E₂)` rather than of zero, and the
/// written sample carries leptons up to `etal + ln(E₁/E₂)`. This crate keeps
/// the correct cut.
///
/// The defect moves which mirrored events MadGraph's sample holds and nothing
/// else, so the comparison splits three ways:
/// - **reported**: every full-sample column the admitted region can correlate
///   with — the leptons' rapidities and polar angles, the incoming `E` and `pz`
///   through `x₁, x₂`, the transverse momenta, the pair mass and `cs_cos(ll)`
///   through the acceptance's dependence on them, and `flavour`, `ICOLUP` and
///   `SPINUP` through the direct-to-mirrored share;
/// - **gated** on the full sample: the columns in [`gated`](Self::gated), each
///   with why the defect cannot move it;
/// - **gated** on every column: the direct-ordering sub-sample (the quark on the
///   `+z` beam), which MadEvent cuts on its own lab frame, so the defect cannot
///   reach it on either side.
///
/// The reference is read before any of that: its direct events must all pass
/// the cut, its mirrored ones must all lie in the displaced window, and some of
/// them must lie outside the cut — the defect present, as registered, in the
/// sample the reported columns are excused against
/// ([`measure_reference`](Self::measure_reference)).
struct MirroredCutDeviation {
    /// What the log calls it.
    name: &'static str,
    /// The full-sample columns that stay gated, each with why the defect cannot
    /// move it. An entry ending in `(` names every column it prefixes.
    gated: &'static [(&'static str, &'static str)],
    /// The report variant the direct-ordering sub-sample is written under.
    untouched_variant: &'static str,
    /// That variant's note.
    untouched_note: &'static str,
}

const MADEVENT_MIRRORED_CUT: MirroredCutDeviation = MirroredCutDeviation {
    name: "MadEvent mirrored-ordering cut",
    gated: &[
        (
            "phi(",
            "the integrand, the cuts and the displaced window are all invariant under \
             rotations about the beam axis, so the azimuth is uniform whichever events \
             the defect admits",
        ),
        (
            "beam1 m",
            "a parton's mass is a constant of the record, whichever event carries it",
        ),
        (
            "beam2 m",
            "a parton's mass is a constant of the record, whichever event carries it",
        ),
        (
            "SCALUP",
            "the card fixes the scales, so every admitted event carries the same value",
        ),
        (
            "AQCDUP",
            "the card fixes the scales, so every admitted event carries the same value",
        ),
    ],
    untouched_variant: "direct_ordering",
    untouched_note: "the direct ordering alone (the quark on the +z beam), which MadEvent \
        3.7.1 cuts on its own lab frame, so the registered mirrored-ordering deviation cannot \
        reach it on either side: every column gated",
};

/// What [`MirroredCutDeviation::measure_reference`] read off MadGraph's sample.
struct MirroredCutWindow {
    /// The card's `etal`, and `ln(E₁/E₂)`, the displacement of the window
    /// MadEvent admits mirrored events in.
    etal: f64,
    shift: f64,
    /// MadGraph's direct and mirrored events, and the mirrored ones carrying a
    /// lepton outside the cut.
    direct: usize,
    mirrored: usize,
    mirrored_outside: usize,
    /// The largest lepton `|η|` on a mirrored event.
    max_eta: f64,
}

/// How far a printed lepton's `η` may sit past a cut edge and count as on it:
/// eleven printed digits of momentum resolve `η` far below this.
const ETA_EDGE: f64 = 1e-6;

/// Whether an event is in the direct ordering: the parton on the `+z` beam is a
/// quark.
fn quark_on_the_plus_beam(event: &LheEvent) -> bool {
    let plus: Vec<i32> = event
        .particles
        .iter()
        .filter(|p| p.status == STATUS_INCOMING && p.momentum[3] > 0.0)
        .map(|p| p.pdg)
        .collect();
    let &[pdg] = plus.as_slice() else {
        panic!("an event without exactly one incoming parton along +z: {plus:?}")
    };
    assert!(
        (1..=6).contains(&pdg.abs()),
        "a Drell-Yan beam parton {pdg} is not a quark"
    );
    pdg > 0
}

/// The charged leptons' pseudorapidities.
fn lepton_etas(event: &LheEvent) -> impl Iterator<Item = f64> + '_ {
    event
        .particles
        .iter()
        .filter(|p| p.status == STATUS_OUTGOING && matches!(p.pdg.abs(), 11 | 13 | 15))
        .map(|p| {
            let [_, px, py, pz] = p.momentum;
            let mag = (px * px + py * py + pz * pz).sqrt();
            0.5 * ((mag + pz) / (mag - pz)).ln()
        })
}

impl MirroredCutDeviation {
    /// Whether a full-sample column stays gated under the deviation.
    fn gates(&self, column: &str) -> bool {
        self.gated.iter().any(|(name, _)| {
            if name.ends_with('(') {
                column.starts_with(name)
            } else {
                column == *name
            }
        })
    }

    /// The direct-ordering events of `sample`, carrying their share of its
    /// cross section.
    fn untouched_part(&self, sample: &EventSample) -> EventSample {
        let total: f64 = sample.weights.iter().sum();
        let (events, weights): (Vec<LheEvent>, Vec<f64>) = sample
            .events
            .iter()
            .zip(&sample.weights)
            .filter(|(event, _)| quark_on_the_plus_beam(event))
            .map(|(event, &w)| (event.clone(), w))
            .unzip();
        let kept: f64 = weights.iter().sum();
        assert!(
            !events.is_empty(),
            "no direct-ordering event in a sample of {}",
            sample.len()
        );
        EventSample {
            events,
            weights,
            sigma_pb: sample.sigma_pb * kept / total,
        }
    }

    /// Read the defect off MadGraph's own sample, failing where it is not the
    /// deviation registered: a direct event outside the cut, a mirrored one
    /// outside the displaced window, or no mirrored event outside the cut at all.
    fn measure_reference(&self, row: &Row, mg: &EventSample) -> MirroredCutWindow {
        let card = match row.run_card {
            Some(name) => validation_dir().join(name),
            None => run_dir(row.run).join("Cards/run_card.dat"),
        };
        let rc = RunCard::parse_file(&card).expect("the row's run card parses");
        let etal = rc.float("etal");
        let shift = (rc.ebeam1 / rc.ebeam2).ln();
        let mut window = MirroredCutWindow {
            etal,
            shift,
            direct: 0,
            mirrored: 0,
            mirrored_outside: 0,
            max_eta: 0.0,
        };
        let (mut direct_outside, mut outside_window) = (0, 0);
        for event in &mg.events {
            let etas: Vec<f64> = lepton_etas(event).collect();
            assert_eq!(
                etas.len(),
                2,
                "a Drell-Yan event with {} leptons",
                etas.len()
            );
            let outside = etas.iter().any(|eta| eta.abs() > etal + ETA_EDGE);
            if quark_on_the_plus_beam(event) {
                window.direct += 1;
                direct_outside += usize::from(outside);
            } else {
                window.mirrored += 1;
                window.mirrored_outside += usize::from(outside);
                outside_window +=
                    usize::from(etas.iter().any(|eta| (eta - shift).abs() > etal + ETA_EDGE));
                window.max_eta = etas.iter().fold(window.max_eta, |m, e| m.max(e.abs()));
            }
        }
        eprintln!(
            "  {} on MadGraph's sample: {direct_outside} / {} direct events with a lepton \
             outside |eta| <= {etal}; {} / {} mirrored ones, all but {outside_window} inside \
             [{:.3}, {:.3}], largest |eta| {:.4}",
            self.name,
            window.direct,
            window.mirrored_outside,
            window.mirrored,
            shift - etal,
            shift + etal,
            window.max_eta,
        );
        assert_eq!(
            direct_outside, 0,
            "MadGraph's direct-ordering events leave the cut: the direct ordering is not the \
             one MadEvent cuts on its own lab frame, so its sub-sample is not untouched"
        );
        assert_eq!(
            outside_window, 0,
            "MadGraph's mirrored events leave the window ln(E1/E2) +- etal: the defect is not \
             the one registered"
        );
        assert!(
            window.mirrored_outside > 0,
            "no mirrored MadGraph event carries a lepton outside the cut: the registered \
             deviation is absent from the reference, so nothing excuses the reported columns"
        );
        window
    }

    /// Our events with a lepton outside the cut, out of all of them.
    fn outside_the_cut(window: &MirroredCutWindow, sample: &EventSample) -> (usize, usize) {
        let bad = sample
            .events
            .iter()
            .filter(|event| lepton_etas(event).any(|eta| eta.abs() > window.etal + ETA_EDGE))
            .count();
        (bad, sample.len())
    }

    /// The full-sample row's note: the deviation, its size in MadGraph's sample,
    /// the gated columns with their arguments, and every reported disagreement.
    fn note(&self, window: &MirroredCutWindow, reported: &[String]) -> String {
        let gated: Vec<String> = self
            .gated
            .iter()
            .map(|(name, why)| {
                let shown = if name.ends_with('(') {
                    format!("{name}...)")
                } else {
                    name.to_string()
                };
                format!("{shown}: {why}")
            })
            .collect();
        format!(
            "registered deviation (user, 2026-10-22): MadEvent 3.7.1 cuts the mirrored beam \
             ordering on the configuration it sampled (dsample.f:181 before \
             super_auto_dsig_group_v4.inc:815-829), which displaces its window by ln(E1/E2) = \
             {:.4}. MadGraph's sample: {} of {} mirrored events carry a lepton outside \
             |eta| <= {}, every one inside [{:.3}, {:.3}] (largest |eta| {:.4}); 0 of {} direct \
             events do. Gated on the full sample, the defect cannot move them: {}. Every other \
             column is reported, not gated; below the floor: {}. The direct-ordering \
             sub-sample ({}) is gated on every column.",
            window.shift,
            window.mirrored_outside,
            window.mirrored,
            window.etal,
            window.shift - window.etal,
            window.shift + window.etal,
            window.max_eta,
            window.direct,
            gated.join("; "),
            if reported.is_empty() {
                "none".to_string()
            } else {
                reported.join("; ")
            },
            self.untouched_variant,
        )
    }
}

/// The dijet row's scans: slot legality, then pattern membership.
const DIJET_SCANS: &[Scan] = &[
    ("slot legality", "legs", scan_slot_legality),
    ("colour patterns", "events", scan_colour_patterns),
];

/// A row's generation side: one integration off its cards, then a sample per seed
/// replayed from the frozen grids the integration wrote.
struct Generator {
    /// Held for its `Drop`: everything below lives inside it.
    _tmp: tempfile::TempDir,
    dir: PathBuf,
    proc_card: PathBuf,
    run_card: PathBuf,
    artifact: PathBuf,
}

impl Generator {
    /// Integrate the row's process against its card, at the row's budget.
    fn integrate(row_spec: &Row) -> Self {
        Self::integrate_with(row_spec, INTEGRATION_SEED, &[])
    }

    /// [`integrate`](Self::integrate) at another integration seed, with `extra`
    /// appended to the `integrate` command line (a map option, say).
    fn integrate_with(row_spec: &Row, seed: &str, extra: &[&str]) -> Self {
        let tmp = tempfile::tempdir().expect("temporary directory");
        let dir = tmp.path().to_path_buf();
        let proc_card = dir.join("proc_card.dat");
        std::fs::write(
            &proc_card,
            format!("import model sm\ngenerate {}\n", row_spec.process),
        )
        .expect("write the proc card");
        // One card for both generators: either MadGraph's own copy in the run
        // directory, or — where the run was driven from a committed card — that
        // card, which is the artifact the rest of the layer pins.
        let run_card = match row_spec.run_card {
            Some(name) => validation_dir().join(name),
            None => run_dir(row_spec.run).join("Cards/run_card.dat"),
        };

        let out = dir.join("out");
        let integrate = common::vibegraph_ambient()
            .arg("integrate")
            .arg(&proc_card)
            .arg("--run-card")
            .arg(&run_card)
            .arg("--out")
            .arg(&out)
            .arg("--pdf-dir")
            .arg(pdf_dir())
            .args([
                "--neval",
                row_spec.neval,
                "--niter",
                row_spec.niter,
                "--seed",
                seed,
            ])
            .args(extra)
            .output()
            .expect("spawn vibegraph");
        assert!(
            integrate.status.success(),
            "vibegraph integrate failed:\n{}",
            common::stderr_of(&integrate)
        );
        Generator {
            artifact: out.join("grid.bin.zst"),
            _tmp: tmp,
            dir,
            proc_card,
            run_card,
        }
    }

    /// One unweighted sample from those grids, at `seed`.
    fn sample(&self, seed: u64) -> EventSample {
        let file = self.dir.join(format!("events_{seed:#x}.lhe"));
        let generate = common::vibegraph_ambient()
            .arg("generate")
            .arg(&self.artifact)
            .arg(&self.proc_card)
            .arg("--run-card")
            .arg(&self.run_card)
            .arg("--pdf-dir")
            .arg(pdf_dir())
            .arg("--seed")
            .arg(seed.to_string())
            .arg("--nevents")
            .arg(NEVENTS.to_string())
            .arg("-o")
            .arg(&file)
            .arg("--force")
            .output()
            .expect("spawn vibegraph");
        assert!(
            generate.status.success(),
            "vibegraph generate failed:\n{}",
            common::stderr_of(&generate)
        );
        let text = std::fs::read_to_string(&file).expect("read the emitted event file");
        let ours = EventSample::from_lhe(LheFile::parse(&text).expect("our own file parses"));
        assert_eq!(
            ours.len(),
            NEVENTS,
            "the generator emitted {} of {NEVENTS} events",
            ours.len()
        );
        ours
    }
}

// The row specifications the gates below take, one const per cell. Named rather
// than written inline at each call so the seed-headroom probe measures the same
// specifications the gates do rather than a copy of them; what each row is *for*
// is on its own test.
const LLJ_FIXED_ROW: Row = Row {
    run: "pp_to_llj_fixed",
    events: "run_01",
    key: "pp_to_llj_fixed",
    variant: None,
    process: "p p > l+ l- j QCD=2 QED=2",
    run_card: None,
    neval: NEVAL,
    niter: NITER,
    mode: "gate",
    scans: &[],
    deviation: None,
};

const LLJ_DYN_ROW: Row = Row {
    run: "pp_to_llj_dyn",
    events: "run_01",
    key: "pp_to_llj_dyn",
    variant: None,
    process: "p p > l+ l- j QCD=2 QED=2",
    run_card: None,
    neval: NEVAL,
    niter: NITER,
    mode: "gate",
    scans: &[],
    deviation: None,
};

const BB_FIXED_ROW: Row = Row {
    run: "pp_to_bb_fixed",
    events: "run_01",
    key: "pp_to_bb_fixed",
    variant: None,
    process: "p p > b b~ QCD=2",
    run_card: None,
    neval: NEVAL,
    niter: NITER,
    mode: "gate",
    scans: &[],
    deviation: None,
};

const JJ_ROW: Row = Row {
    run: "pp_to_jj",
    events: "run_01",
    key: "pp_to_jj",
    variant: None,
    process: "p p > j j",
    run_card: None,
    neval: NEVAL,
    niter: NITER,
    mode: "gate",
    scans: DIJET_SCANS,
    deviation: None,
};

const BB_ROW: Row = Row {
    run: "pp_to_bb",
    events: "run_01",
    key: "pp_to_bb",
    variant: None,
    process: "p p > b b~",
    run_card: None,
    neval: NEVAL,
    niter: NITER,
    mode: "gate",
    scans: &[],
    deviation: None,
};

const BB_QCD2_ROW: Row = Row {
    run: "pp_to_bb_qcd2",
    events: "run_01",
    key: "pp_to_bb_qcd2",
    variant: None,
    process: "p p > b b~ QCD=2",
    run_card: None,
    neval: NEVAL,
    niter: NITER,
    mode: "gate",
    scans: &[],
    deviation: None,
};

const LLJ_ROW: Row = Row {
    run: "pp_to_llj",
    events: "run_01",
    key: "pp_to_llj",
    variant: None,
    process: "p p > l+ l- j",
    run_card: None,
    neval: NEVAL,
    niter: NITER,
    mode: "gate",
    scans: &[],
    deviation: None,
};

const SCALEFACT2_ROW: Row = Row {
    run: "pp_to_ll_scalefact2",
    events: "run_01",
    key: "pp_to_ll_scalefact2",
    variant: None,
    process: "p p > l+ l-",
    run_card: None,
    neval: NEVAL,
    niter: NITER,
    mode: "gate",
    scans: &[],
    deviation: None,
};

/// Every gate row above, in the order the tests declare them — the set
/// `probe_proton_samples_p_floor_headroom` sweeps.
const ALL_GATE_ROWS: &[&Row] = &[
    &LLJ_FIXED_ROW,
    &LLJ_DYN_ROW,
    &BB_FIXED_ROW,
    &JJ_ROW,
    &BB_ROW,
    &BB_QCD2_ROW,
    &LLJ_ROW,
    &SCALEFACT2_ROW,
    &DY_ROWS[0],
    &DY_ROWS[1],
    &DY_ROWS[2],
];

/// One row: integrate the banked cards once, then generate and compare a sample
/// per seed.
///
/// `mode` is `gate` for a row whose columns agree and `info` for one carrying a
/// recorded disagreement: the measurement is taken and reported either way, and
/// the mode says only whether a column below the floor fails the suite. A row is
/// demoted with a note saying what was measured and where the fix is tracked —
/// never by widening the floor.
fn check_row(row_spec: &Row) {
    let clock = Stopwatch::start();
    let Row {
        run,
        key,
        variant,
        process,
        mode,
        ..
    } = *row_spec;
    let mg = banked_sample(run, row_spec.events);
    let generator = Generator::integrate(row_spec);

    eprintln!(
        "-- {run} ({} banked events, sigma {:.4} pb, PDF set {PDF_SET}) --",
        mg.len(),
        mg.sigma_pb
    );

    // Both scans are instruments, so both are read on MadGraph's own banked sample
    // first: one that cannot fail on the reference is not measuring anything.
    for (name, unit, scan) in row_spec.scans {
        let (bad, total) = scan(row_spec, &mg);
        eprintln!("  scan {name} on MadGraph's banked sample: {bad} / {total} {unit}");
        assert_eq!(
            bad, 0,
            "[{run}] the {name} scan reports {bad} of {total} {unit} on MadGraph's own events, \
             so it is not a statement about ours"
        );
    }
    let window = row_spec
        .deviation
        .map(|deviation| deviation.measure_reference(row_spec, &mg));

    let mut row = SamplesRow::new(key, process, mode);
    row.variant = variant.map(str::to_string);
    row.p_floor = P_FLOOR;
    row.mg_events = mg.len();
    row.sigma_mg_pb = mg.sigma_pb;
    row.labelling = "coarse";
    let mut failures: Vec<String> = Vec::new();
    let mut reported: Vec<String> = Vec::new();
    // The sub-sample a registered deviation cannot reach, compared on both sides
    // and gated on every column.
    let mut untouched = row_spec.deviation.map(|deviation| {
        let mg_part = deviation.untouched_part(&mg);
        let mut part =
            SamplesRow::new(key, process, mode).with_variant(deviation.untouched_variant);
        part.p_floor = P_FLOOR;
        part.mg_events = mg_part.len();
        part.sigma_mg_pb = mg_part.sigma_pb;
        part.labelling = "coarse";
        (deviation, mg_part, part)
    });

    for &seed in &GEN_SEEDS {
        let ours = generator.sample(seed);

        for (name, unit, scan) in row_spec.scans {
            let (bad, total) = scan(row_spec, &ours);
            eprintln!("  seed {seed:#010x} | scan {name}: {bad} / {total} {unit}");
            if bad != 0 {
                failures.push(format!(
                    "seed {seed:#010x} scan {name}: {bad} of {total} {unit}"
                ));
            }
        }
        if let (Some(deviation), Some(window)) = (row_spec.deviation, &window) {
            let (bad, total) = MirroredCutDeviation::outside_the_cut(window, &ours);
            eprintln!(
                "  seed {seed:#010x} | {}: {bad} / {total} events with a lepton outside \
                 |eta| <= {}",
                deviation.name, window.etal
            );
            if bad != 0 {
                failures.push(format!(
                    "seed {seed:#010x}: {bad} of {total} events with a lepton outside the cut"
                ));
            }
        }

        let labelling = labelling_for(&ours, &mg);
        let found = compare(&ours, &mg, labelling);
        print_comparison(seed, "", &ours, &found);
        judge(
            &found,
            &format!("seed {seed:#010x}"),
            |column| row_spec.deviation.is_none_or(|d| d.gates(column)),
            &mut failures,
            &mut reported,
        );
        row.constant_observables = found.constant.clone();
        row.single_category = found
            .single_category
            .iter()
            .map(|c| (*c).to_string())
            .collect();
        row.per_seed.push(seed_sample(seed, &ours, &found));

        if let Some((deviation, mg_part, part)) = untouched.as_mut() {
            let ours_part = deviation.untouched_part(&ours);
            let found = compare(&ours_part, mg_part, labelling_for(&ours_part, mg_part));
            print_comparison(seed, deviation.untouched_variant, &ours_part, &found);
            judge(
                &found,
                &format!("seed {seed:#010x} {}", deviation.untouched_variant),
                |_| true,
                &mut failures,
                &mut reported,
            );
            part.constant_observables = found.constant.clone();
            part.single_category = found
                .single_category
                .iter()
                .map(|c| (*c).to_string())
                .collect();
            part.per_seed.push(seed_sample(seed, &ours_part, &found));
        }
    }

    row.finish();
    eprintln!(
        "  min KS p {:.3e} ({}), min chi2 p {:.3e} ({}), worst incoming {} \
         (dev {:.4e} against tol {:.2e}, min beam KS p {:.3e}) over {} seeds",
        row.min_ks_p,
        row.worst_ks_observable,
        row.min_chi2_p,
        row.worst_chi2_column,
        row.worst_beam_field,
        row.max_beam_dev,
        row.beam_tol,
        row.min_beam_ks_p,
        GEN_SEEDS.len()
    );
    eprintln!(
        "  worst reported {} (dev {:.4e} against tol {:.2e}, min scale KS p {:.3e})",
        row.worst_scale_field, row.max_scale_dev, row.scale_tol, row.min_scale_ks_p,
    );
    let status = match mode {
        "gate" => {
            if failures.is_empty() {
                "pass"
            } else {
                "fail"
            }
        }
        _ => "info",
    };
    if let (Some(deviation), Some(window)) = (row_spec.deviation, &window) {
        let note = deviation.note(window, &reported);
        eprintln!("  INFO {note}");
        row.note = Some(note);
    }
    row.status = status;
    row.duration_s = Some(clock.seconds());
    row.write();
    if let Some((deviation, _, mut part)) = untouched {
        part.finish();
        part.note = Some(deviation.untouched_note.to_string());
        part.status = status;
        part.duration_s = row.duration_s;
        part.write();
    }

    if mode != "gate" {
        if !failures.is_empty() {
            eprintln!("  [{run}] measured, not enforced:\n{failures:#?}");
        }
        return;
    }
    assert!(failures.is_empty(), "[{run}] samples gate:\n{failures:#?}");
}

/// One generated sample's comparison as log lines, `scope` naming the
/// sub-sample it is of, or empty for the whole one.
fn print_comparison(seed: u64, scope: &str, ours: &EventSample, found: &Comparison) {
    let worst = found.worst_ks().expect("a comparable observable");
    eprintln!(
        "  seed {seed:#010x}{}{scope} | {} events (n_eff {:.0}) | worst KS: {} p {:.3e} (D {:.4})",
        if scope.is_empty() { "" } else { " " },
        ours.len(),
        ours.effective_size(),
        worst.observable,
        worst.p,
        worst.d,
    );
    for cell in &found.chi2 {
        eprintln!(
            "             chi2 {:<8} p {:.3e} (chi2 {:.1} / {} dof over {} of {} categories, \
             {:.1}% pooled)",
            cell.column,
            cell.p,
            cell.chi2,
            cell.dof,
            cell.categories,
            cell.distinct_keys,
            100.0 * cell.pooled_share
        );
    }
    for cell in found.beams.iter().chain(&found.scales) {
        eprintln!("             {}", cell.describe());
    }
}

/// Every column of `found` below the floor or outside its printed precision, as
/// a failure where `gated` names the column and as a reported line where it does
/// not. `scope` heads each line.
fn judge(
    found: &Comparison,
    scope: &str,
    gated: impl Fn(&str) -> bool,
    failures: &mut Vec<String>,
    reported: &mut Vec<String>,
) {
    let mut file = |column: &str, line: String| {
        if gated(column) {
            failures.push(format!("{scope} {line}"));
        } else {
            reported.push(format!("{scope} {line}"));
        }
    };
    for cell in &found.ks {
        if cell.p < P_FLOOR {
            file(
                &cell.observable,
                format!(
                    "KS {} p {:.3e} (D {:.4}) below the {P_FLOOR:.0e} floor",
                    cell.observable, cell.p, cell.d
                ),
            );
        }
    }
    for cell in &found.chi2 {
        if cell.p < P_FLOOR {
            file(
                cell.column,
                format!(
                    "chi2 {} p {:.3e} ({:.1}/{} dof) below the {P_FLOOR:.0e} floor",
                    cell.column, cell.p, cell.chi2, cell.dof
                ),
            );
        }
    }
    for (what, columns) in [("incoming", &found.beams), ("reported", &found.scales)] {
        for cell in columns {
            if let Some(line) = cell.disagreement(what, P_FLOOR) {
                file(&cell.field, line);
            }
        }
    }
}

/// One seed's cells as the report records them.
fn seed_sample(seed: u64, ours: &EventSample, found: &Comparison) -> SeedSample {
    SeedSample {
        seed,
        events: ours.len(),
        sigma_pb: ours.sigma_pb,
        ks: found
            .ks
            .iter()
            .map(|c| KsCell {
                observable: c.observable.clone(),
                d: c.d,
                p: c.p,
            })
            .collect(),
        chi2: found
            .chi2
            .iter()
            .map(|c| Chi2Cell {
                column: c.column.to_string(),
                chi2: c.chi2,
                dof: c.dof,
                p: c.p,
                categories: c.categories,
                distinct_keys: c.distinct_keys,
                pooled_share: c.pooled_share,
                detail: c
                    .detail
                    .iter()
                    .map(|(key, ours, theirs)| CategoryCount {
                        key: key.clone(),
                        ours: *ours,
                        theirs: *theirs,
                    })
                    .collect(),
            })
            .collect(),
        beams: found.beams.iter().map(FieldCell::of).collect(),
        scales: found.scales.iter().map(FieldCell::of).collect(),
    }
}

#[test]
fn generated_proton_events_agree_with_madgraphs_banked_ones() {
    check_row(&LLJ_FIXED_ROW);
}

/// The scale-moved twin of the row above: same generate line, same budget, the
/// run's own dynamical-scale card instead of the fixed one. The two run cards
/// differ only in the three `fixed_*_scale` booleans, so this cell is what the
/// per-point `AMP2`-drawn configuration change actually exercises end to end —
/// a sample assembled by the binary rather than a scalar cross section.
#[test]
fn generated_llj_dyn_events_agree_with_madgraphs_banked_ones() {
    check_row(&LLJ_DYN_ROW);
}

/// The same comparison on a purely hadronic final state: no lepton column to
/// carry the kinematics, a gluon-initiated group, and the flavour draw spread
/// over three groups of which two are mirrored.
///
/// This row is the sharper check on the colour draw: the s-channel
/// gluon-splitting configuration admits *both* leading-colour flows, so the
/// realised `ICOLUP` frequencies exercise the per-configuration `AMP2` weights
/// themselves and not merely the `ICOLAMP` mask — the two sub-percent flows
/// land at MadGraph's `0.07%`/`0.08%` only if the configuration draw carries
/// the right shares. Chi-squared over five degrees of freedom clears the floor
/// on every seed, alongside the kinematic KS, helicity and flavour-group
/// columns.
#[test]
fn generated_b_quark_events_agree_with_madgraphs_banked_ones() {
    check_row(&BB_FIXED_ROW);
}

/// The two Drell-Yan cells of the `pp_to_ll` row: the committed `dy13` cards,
/// whose MadGraph runs bank a cross section and a sample out of one invocation,
/// so the reference this compares against and the one `validate_hadronic` gates
/// σ on are the same run.
///
/// The pair is one row measured twice rather than two rows: the default card has
/// no `m_ll` window and so runs the spectrum down to where the lepton cuts stop
/// it, and `mmll_60_120` is the same process with the window on. What the second
/// adds is the cut path — an event sample generated under a cut both sides apply
/// is the only place the `mmll`/`mmllmax` implementation is compared
/// event-by-event rather than through a scalar.
const DY_ROWS: &[Row] = &[
    Row {
        run: "dy13_default",
        events: "run_default",
        key: "pp_to_ll",
        variant: Some("default"),
        process: "p p > e+ e-",
        run_card: Some("dy13_default_run_card.dat"),
        neval: DY_NEVAL,
        niter: DY_NITER,
        mode: "gate",
        scans: &[],
        deviation: None,
    },
    Row {
        run: "dy13_mmll_60_120",
        events: "run_mmll_60_120",
        key: "pp_to_ll",
        variant: Some("mmll_60_120"),
        process: "p p > e+ e-",
        run_card: Some("dy13_mmll_run_card.dat"),
        neval: DY_NEVAL,
        niter: DY_NITER,
        mode: "gate",
        scans: &[],
        deviation: None,
    },
    // Own manifest row rather than a variant of `pp_to_ll`: the beams are the
    // only difference from the default card, and the cells it fills are the ones
    // no equal-beam row can see.
    Row {
        run: "dy13_beams_6500_4000",
        events: "run_beams_6500_4000",
        key: "pp_to_ll_beams_6500_4000",
        variant: None,
        process: "p p > e+ e-",
        run_card: Some("dy_beams_6500_4000_run_card.dat"),
        neval: DY_NEVAL,
        niter: DY_NITER,
        mode: "gate",
        scans: &[],
        deviation: Some(&MADEVENT_MIRRORED_CUT),
    },
];

#[test]
fn generated_drell_yan_events_agree_with_madgraphs_banked_ones() {
    check_row(&DY_ROWS[0]);
}

#[test]
fn generated_drell_yan_events_in_the_mass_window_agree_with_madgraphs_banked_ones() {
    check_row(&DY_ROWS[1]);
}

/// Proton beams of 6500 and 4000 GeV, the one banked row whose beams differ: the
/// events carry the lab boost and the per-beam incoming energies that no
/// equal-beam row can distinguish from the energy-sum form.
///
/// Reports `NOT YET BANKED` and writes no cell while the manifest marks the row
/// `planned` and its MadGraph run is absent; once the row is banked the gate runs,
/// and once the manifest status is flipped an absent run fails.
#[test]
fn generated_unequal_beam_drell_yan_events_agree_with_madgraphs_banked_ones() {
    if not_yet_banked(&DY_ROWS[2]) {
        return;
    }
    check_row(&DY_ROWS[2]);
}

/// [`GEN_SEEDS`] extended to AGENTS.md's five for the headroom census below. The
/// first three are [`GEN_SEEDS`] itself, so the probe's reading contains the
/// gate's own.
const HEADROOM_GEN_SEEDS: [u64; 5] = [
    0x5A_4D_1001,
    0x5A_4D_1002,
    0x5A_4D_1003,
    0x5A_4D_1004,
    0x5A_4D_1005,
];

/// [`P_FLOOR`]'s headroom on the proton rows over five generation seeds rather
/// than the gate's three.
///
/// The gate statistic is the smallest `p` over rows × seeds × columns. A minimum
/// is the order statistic that moves most when the sample grows, so what the two
/// extra seeds measure is how much of the gate's margin over the floor is a
/// property of the comparison and how much is of how few draws it takes.
///
/// The floor itself is a designed false-positive rate against a trial count, not
/// a tolerance: more seeds mean more draws from the null and so a lower expected
/// minimum. The reading is therefore *how far* the minimum falls, and a fall
/// roughly in proportion to the extra draws is the null behaving. Run with
/// `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_proton_samples_p_floor_headroom() {
    let mut three: Vec<(f64, String)> = Vec::new();
    let mut five: Vec<(f64, String)> = Vec::new();
    for row_spec in ALL_GATE_ROWS {
        if not_yet_banked(row_spec) {
            continue;
        }
        if !run_present("probe_proton_samples_p_floor_headroom", row_spec.run) {
            continue;
        }
        let mg = banked_sample(row_spec.run, row_spec.events);
        let generator = Generator::integrate(row_spec);
        let mut row_three = (f64::INFINITY, String::new());
        let mut row_five = (f64::INFINITY, String::new());
        for (i, &seed) in HEADROOM_GEN_SEEDS.iter().enumerate() {
            let ours = generator.sample(seed);
            let found = compare(&ours, &mg, labelling_for(&ours, &mg));
            let mut worst = (f64::INFINITY, String::new());
            for cell in &found.ks {
                if cell.p < worst.0 {
                    worst = (cell.p, format!("KS {}", cell.observable));
                }
            }
            for cell in &found.chi2 {
                if cell.p < worst.0 {
                    worst = (cell.p, format!("chi2 {}", cell.column));
                }
            }
            eprintln!(
                "  [{}] seed {seed:#010x} ({} events): worst p {:.3e} on {}",
                row_spec.run,
                ours.len(),
                worst.0,
                worst.1
            );
            if i < GEN_SEEDS.len() && worst.0 < row_three.0 {
                row_three = (worst.0, format!("seed {seed:#010x} {}", worst.1));
            }
            if worst.0 < row_five.0 {
                row_five = (worst.0, format!("seed {seed:#010x} {}", worst.1));
            }
        }
        eprintln!(
            "  ROW {:<22} mode {:<4} | 3-seed min p {:.3e} ({:.1}x floor) | 5-seed min p \
             {:.3e} ({:.1}x floor) on {}",
            row_spec.run,
            row_spec.mode,
            row_three.0,
            row_three.0 / P_FLOOR,
            row_five.0,
            row_five.0 / P_FLOOR,
            row_five.1
        );
        if row_spec.mode == "gate" {
            three.push((row_three.0, format!("{} {}", row_spec.run, row_three.1)));
            five.push((row_five.0, format!("{} {}", row_spec.run, row_five.1)));
        }
    }
    let pick = |v: &[(f64, String)]| {
        v.iter()
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .expect("a gating row")
            .clone()
    };
    let (p3, w3) = pick(&three);
    let (p5, w5) = pick(&five);
    eprintln!(
        "\nHEADROOM samples_proton P_FLOOR {P_FLOOR:.0e} | 3 seeds over {} gating rows: min p \
         {p3:.3e} ({:.1}x) on {w3} | 5 seeds: min p {p5:.3e} ({:.1}x) on {w5}",
        three.len(),
        p3 / P_FLOOR,
        p5 / P_FLOOR,
    );
}

/// The dijet flavour calibration's integration seeds: the gate's own grid, then
/// an independent one, so a frozen grid's own draw is not read as its map's.
const JJ_CALIBRATION_GRIDS: [&str; 2] = [INTEGRATION_SEED, "20260741"];

/// Generation seeds per calibration grid: the gate's own on the gate's grid, and
/// seeds of their own on the second.
///
/// The second grid cannot reuse the first's seeds. A generation seed fixes the
/// whole trial stream — channel, point and acceptance uniforms — and two grids
/// trained on the same cards differ only slightly, so the same seed replayed off
/// both draws largely the same events: the two samples are correlated, and a χ²
/// between them reads below its null.
const JJ_CALIBRATION_GEN_SEEDS: [[u64; 5]; 2] = [
    HEADROOM_GEN_SEEDS,
    [
        0x5A_4D_2001,
        0x5A_4D_2002,
        0x5A_4D_2003,
        0x5A_4D_2004,
        0x5A_4D_2005,
    ],
];

/// A second, independent grid pair for the calibration's null, with generation
/// seeds of their own: the null of [`JJ_CALIBRATION_GRIDS`] reads each grid's
/// samples against the other's, so one pair's readings share its two grids, and
/// a second pair is the independent repeat.
const JJ_SECOND_PAIR_GRIDS: [&str; 2] = ["20260751", "20260761"];

/// Generation seeds for [`JJ_SECOND_PAIR_GRIDS`], disjoint from every other
/// calibration grid's.
const JJ_SECOND_PAIR_GEN_SEEDS: [[u64; 5]; 2] = [
    [
        0x5A_4D_3001,
        0x5A_4D_3002,
        0x5A_4D_3003,
        0x5A_4D_3004,
        0x5A_4D_3005,
    ],
    [
        0x5A_4D_4001,
        0x5A_4D_4002,
        0x5A_4D_4003,
        0x5A_4D_4004,
        0x5A_4D_4005,
    ],
];

/// The seed of the permutations the calibration splits samples with.
const JJ_SPLIT_SEED: u64 = 0x5A_4D_5E11;

/// Random halvings of MadGraph's banked dijet sample in its null.
const MG_HALF_SPLITS: usize = 1000;

/// The `pp_to_jj` samples cell's χ² columns, and the null they would read under,
/// on the `τ` map `map`.
///
/// Two grids — [`JJ_CALIBRATION_GRIDS`] — each generating five samples of
/// [`NEVENTS`] at its own [`JJ_CALIBRATION_GEN_SEEDS`], all under
/// `--map-tau map`.
/// Against MadGraph's banked sample it records every seed's χ² columns (the
/// gate's statistic) and the pooled sample of the first three and of all five
/// seeds (one χ² each). Then the null, where the two sides cannot differ: each
/// grid's samples, singly and pooled, against random 10 000-event halves of the
/// *other* grid's samples, the reference size the gate compares against. Every
/// reading is one `JJREC` line.
///
/// What it cannot detect: anything common to both grids — the null is this
/// side's own, so it calibrates the statistic, not this side's physics.
///
/// Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_jj_flavour_chi2_calibration_log() {
    jj_flavour_calibration("log", JJ_CALIBRATION_GRIDS, JJ_CALIBRATION_GEN_SEEDS);
}

/// [`probe_jj_flavour_chi2_calibration_log`] under MadEvent's `τ` rule.
#[test]
#[ignore]
fn probe_jj_flavour_chi2_calibration_inverse_square() {
    jj_flavour_calibration(
        "inverse-square",
        JJ_CALIBRATION_GRIDS,
        JJ_CALIBRATION_GEN_SEEDS,
    );
}

/// [`probe_jj_flavour_chi2_calibration_log`] on [`JJ_SECOND_PAIR_GRIDS`]: the
/// grid-against-grid null repeated on two grids that share nothing with the
/// first pair.
#[test]
#[ignore]
fn probe_jj_flavour_chi2_second_grid_pair_log() {
    jj_flavour_calibration("log", JJ_SECOND_PAIR_GRIDS, JJ_SECOND_PAIR_GEN_SEEDS);
}

/// [`probe_jj_flavour_chi2_second_grid_pair_log`] under MadEvent's `τ` rule.
#[test]
#[ignore]
fn probe_jj_flavour_chi2_second_grid_pair_inverse_square() {
    jj_flavour_calibration(
        "inverse-square",
        JJ_SECOND_PAIR_GRIDS,
        JJ_SECOND_PAIR_GEN_SEEDS,
    );
}

/// MadGraph's banked dijet sample against itself: [`MG_HALF_SPLITS`] random
/// halvings, each half of 5 000 compared with the other, one `JJREC` line each.
///
/// A random halving of one sample is a draw from the null of the two-sample χ²
/// conditioned on the sample's own composition, so its distribution is what the
/// statistic reads where the two sides cannot differ, with no generator of ours
/// in it. The file's own event order is MadGraph's per-channel concatenation, so
/// a split along it would compare channels, not halves.
///
/// Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_jj_flavour_chi2_mg_half_splits() {
    use rand::seq::SliceRandom;
    use rand::SeedableRng;

    if !run_present("probe_jj_flavour_chi2_mg_half_splits", JJ_ROW.run) {
        return;
    }
    let mg = banked_sample(JJ_ROW.run, JJ_ROW.events);
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(JJ_SPLIT_SEED);
    let mut order: Vec<usize> = (0..mg.len()).collect();
    for split in 0..MG_HALF_SPLITS {
        order.shuffle(&mut rng);
        let (a, b) = order.split_at(mg.len() / 2);
        let (a, b) = (subsample(&mg, a), subsample(&mg, b));
        let found = compare(&a, &b, labelling_for(&a, &b));
        print_chi2_records(
            "mg",
            "null-mg",
            &format!("mg-half-{split}"),
            "mg-half",
            &found,
            a.len(),
            b.len(),
        );
    }
}

/// The calibration run behind the map probes above, on the grid pair `grids`
/// with generation seeds `gen_seeds`.
fn jj_flavour_calibration(map: &str, grids: [&str; 2], gen_seeds: [[u64; 5]; 2]) {
    use rand::seq::SliceRandom;
    use rand::SeedableRng;

    if !run_present("probe_jj_flavour_chi2_calibration", JJ_ROW.run) {
        return;
    }
    let mg = banked_sample(JJ_ROW.run, JJ_ROW.events);
    let samples: Vec<Vec<EventSample>> = grids
        .iter()
        .zip(&gen_seeds)
        .map(|(&grid, seeds)| {
            let clock = Stopwatch::start();
            let generator = Generator::integrate_with(&JJ_ROW, grid, &["--map-tau", map]);
            eprintln!(
                "  [{map}] grid {grid} integrated in {:.0}s",
                clock.seconds()
            );
            seeds
                .iter()
                .map(|&seed| {
                    let clock = Stopwatch::start();
                    let ours = generator.sample(seed);
                    eprintln!(
                        "  [{map}] grid {grid} seed {seed:#010x}: {} events (n_eff {:.0}) in \
                         {:.0}s",
                        ours.len(),
                        ours.effective_size(),
                        clock.seconds()
                    );
                    ours
                })
                .collect()
        })
        .collect();

    for (g, grid) in grids.iter().enumerate() {
        for (sample, seed) in samples[g].iter().zip(gen_seeds[g]) {
            let found = compare(sample, &mg, labelling_for(sample, &mg));
            print_chi2_records(
                map,
                "mg",
                grid,
                &format!("{seed:#010x}"),
                &found,
                sample.len(),
                mg.len(),
            );
        }
        for k in [GEN_SEEDS.len(), HEADROOM_GEN_SEEDS.len()] {
            let pool = pooled(&samples[g][..k]);
            let found = compare(&pool, &mg, labelling_for(&pool, &mg));
            print_chi2_records(
                map,
                "mg",
                grid,
                &format!("pool{k}"),
                &found,
                pool.len(),
                mg.len(),
            );
        }
    }

    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(JJ_SPLIT_SEED);
    for (g, grid) in grids.iter().enumerate() {
        let other = 1 - g;
        for (r, reference) in samples[other].iter().enumerate() {
            let mut order: Vec<usize> = (0..reference.len()).collect();
            order.shuffle(&mut rng);
            for (h, half) in order.chunks(reference.len() / 2).enumerate() {
                let half = subsample(reference, half);
                let tag = format!("null-{}-{r}-{h}", grids[other]);
                for (sample, seed) in samples[g].iter().zip(gen_seeds[g]) {
                    let found = compare(sample, &half, labelling_for(sample, &half));
                    print_chi2_records(
                        map,
                        &tag,
                        grid,
                        &format!("{seed:#010x}"),
                        &found,
                        sample.len(),
                        half.len(),
                    );
                }
                for k in [GEN_SEEDS.len(), HEADROOM_GEN_SEEDS.len()] {
                    let pool = pooled(&samples[g][..k]);
                    let found = compare(&pool, &half, labelling_for(&pool, &half));
                    print_chi2_records(
                        map,
                        &tag,
                        grid,
                        &format!("pool{k}"),
                        &found,
                        pool.len(),
                        half.len(),
                    );
                }
            }
        }
    }
}

/// One `JJREC` line per χ² column of `found`, and one for its smallest KS `p`.
fn print_chi2_records(
    map: &str,
    reference: &str,
    grid: &str,
    sample: &str,
    found: &vibegraph::validation::samples::Comparison,
    n_ours: usize,
    n_reference: usize,
) {
    for cell in &found.chi2 {
        eprintln!(
            "JJREC map={map} ref={reference} grid={grid} sample={sample} column={} \
             chi2={:.3} dof={} p={:.4e} pooled_share={:.4} n={n_ours} n_ref={n_reference}",
            cell.column, cell.chi2, cell.dof, cell.p, cell.pooled_share
        );
    }
    if let Some(worst) = found.worst_ks() {
        eprintln!(
            "JJREC map={map} ref={reference} grid={grid} sample={sample} column=KS:{} \
             chi2=0 dof=0 p={:.4e} pooled_share=0 n={n_ours} n_ref={n_reference}",
            worst.observable, worst.p
        );
    }
}

/// The events of `sample` at `indices`, with their weights.
fn subsample(sample: &EventSample, indices: &[usize]) -> EventSample {
    EventSample {
        events: indices.iter().map(|&i| sample.events[i].clone()).collect(),
        weights: indices.iter().map(|&i| sample.weights[i]).collect(),
        sigma_pb: sample.sigma_pb,
    }
}

/// Samples generated off one grid, as one sample.
fn pooled(samples: &[EventSample]) -> EventSample {
    EventSample {
        events: samples.iter().flat_map(|s| s.events.clone()).collect(),
        weights: samples.iter().flat_map(|s| s.weights.clone()).collect(),
        sigma_pb: samples.iter().map(|s| s.sigma_pb).sum::<f64>() / samples.len() as f64,
    }
}

/// Bin edges for `dσ/dm_ll`, in GeV.
///
/// The first is the threshold the default card's own cuts impose, and it is
/// exact rather than approximate: at this order the pair recoils against nothing,
/// so both leptons carry the same `pt` and `m_ll = 2 pt / sin θ ≥ 2 ptl = 20 GeV`.
/// The `drll` cut never binds under it — a back-to-back pair has `Δφ = π`. Widths
/// follow how fast the spectrum moves: 2 GeV across the Z, decade-wide in the
/// tail.
const MLL_EDGES: &[f64] = &[
    20.0, 25.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 86.0, 88.0, 90.0, 92.0, 94.0, 96.0, 100.0,
    110.0, 120.0, 150.0, 200.0, 300.0, 500.0, 1000.0,
];

/// A bin carrying less than this share of MadGraph's own sample is reported but
/// not judged: the reference's `√n` there is not a measurement of anything.
const SPECTRUM_MIN_SHARE: f64 = 1e-4;

/// How far a bin may sit from MadGraph's in units of the two errors combined.
///
/// The comparison is over about 25 judged bins across two cards, so the largest
/// of ~50 standard normal draws is typically near 2.6 and exceeds 4 about one run
/// in 6000. The threshold is set on that trial count rather than on the observed
/// deviations, and the suite reports every bin's pull so a drift shows as the
/// column moving rather than as one bin crossing.
const SPECTRUM_MAX_PULL: f64 = 4.0;

/// `dσ/dm_ll` in absolute picobarns, ours against MadGraph's own events, on both
/// Drell-Yan cards.
///
/// # What this adds over the row's KS and χ² columns
///
/// Normalisation, differentially. [`compare`]'s KS test on `m(e+,e-)` is a
/// statement about the two cumulative distributions and is blind to both samples'
/// cross sections, so a spectrum uniformly low by a few percent passes it; the σ
/// gate in `validate_hadronic` is the opposite, one scalar that a compensating
/// pair of errors passes. Binning each side against its *own* cross section
/// closes both: a bin's picobarns are wrong if either the shape or the
/// normalisation is.
///
/// The low-mass end is the part worth having. Between the threshold and the Z the
/// integrand is the photon pole, falling by two orders of magnitude across the
/// range and carrying the part of the cross section a map tuned to the Z is most
/// likely to under-cover — and it is exactly the region the two cards differ
/// over, since the window card removes it entirely. So the pair separates "the
/// spectrum is right" from "the cut is right", and the threshold check below
/// separates both from "the cut is in the right place".
///
/// # What it provably cannot detect
///
/// Structure narrower than a bin, and any error that leaves `m_ll` alone: the
/// spectrum is one projection, and a rotation of the final state within a mass
/// bin moves nothing here. The row's own KS columns cover the other observables,
/// and neither sees a correlation between two of them.
#[test]
fn the_drell_yan_mass_spectrum_is_binned_against_madgraph() {
    let mut failures: Vec<String> = Vec::new();
    for row_spec in DY_ROWS {
        if not_yet_banked(row_spec) {
            continue;
        }
        // A row whose reference carries a registered deviation is binned on the
        // sub-sample the deviation cannot reach, in that sub-sample's own share of
        // the cross section: the whole spectrum is off by the deviation itself.
        let restrict = |sample: EventSample| match row_spec.deviation {
            Some(deviation) => deviation.untouched_part(&sample),
            None => sample,
        };
        if let Some(deviation) = row_spec.deviation {
            eprintln!(
                "  [{}] {}: binned on the {} sub-sample only",
                row_spec.run, deviation.name, deviation.untouched_variant
            );
        }
        let mg = restrict(banked_sample(row_spec.run, row_spec.events));
        let mut theirs = Spectrum::new(MLL_EDGES);
        theirs.fill_from(&mg, "m(e+,e-)", Labelling::Fine);

        let generator = Generator::integrate(row_spec);
        let mut ours = Spectrum::new(MLL_EDGES);
        let mut sigma_sum = 0.0;
        for &seed in &GEN_SEEDS {
            let sample = restrict(generator.sample(seed));
            sigma_sum += sample.sigma_pb;
            ours.fill_from(&sample, "m(e+,e-)", Labelling::Fine);
        }
        let our_sigma = sigma_sum / GEN_SEEDS.len() as f64;

        let a = ours.as_sigma(our_sigma);
        let b = theirs.as_sigma(mg.sigma_pb);
        let (mg_below, mg_above) = theirs.outside();
        let (our_below, our_above) = ours.outside();
        eprintln!(
            "-- dsigma/dm_ll: {} ({} MadGraph events at {:.3} pb, {} of ours at {:.3} pb) --\n  \
             {:>13} {:>21} {:>21} {:>12} {:>8} {:>7}",
            row_spec.run,
            mg.len(),
            mg.sigma_pb,
            GEN_SEEDS.len() * NEVENTS,
            our_sigma,
            "bin",
            "ours",
            "MadGraph",
            "ours-MG",
            "rel",
            "pull",
        );
        for (ours_bin, mg_bin) in a.iter().zip(&b) {
            if ours_bin.sigma_pb <= 0.0 && mg_bin.sigma_pb <= 0.0 {
                continue;
            }
            let combined =
                (ours_bin.err_pb * ours_bin.err_pb + mg_bin.err_pb * mg_bin.err_pb).sqrt();
            let pull = if combined > 0.0 {
                (ours_bin.sigma_pb - mg_bin.sigma_pb) / combined
            } else {
                0.0
            };
            let share = mg_bin.sigma_pb / mg.sigma_pb;
            let judged = share >= SPECTRUM_MIN_SHARE;
            eprintln!(
                "    {:>6.0}-{:<6.0} {:>12.5e} +- {:>6.1e} {:>12.5e} +- {:>6.1e} {:>+12.4e} \
                 {:>+7.2}% {:>+7.2}{}",
                ours_bin.low,
                ours_bin.high,
                ours_bin.sigma_pb,
                ours_bin.err_pb,
                mg_bin.sigma_pb,
                mg_bin.err_pb,
                ours_bin.sigma_pb - mg_bin.sigma_pb,
                100.0 * (ours_bin.sigma_pb / mg_bin.sigma_pb - 1.0),
                pull,
                if judged { "" } else { "  (not judged)" },
            );
            if judged && pull.abs() > SPECTRUM_MAX_PULL {
                failures.push(format!(
                    "[{}] m_ll in [{:.0}, {:.0}]: {:.5e} against MadGraph {:.5e} pb, \
                     pull {pull:+.2} beyond {SPECTRUM_MAX_PULL:.0}",
                    row_spec.run, ours_bin.low, ours_bin.high, ours_bin.sigma_pb, mg_bin.sigma_pb,
                ));
            }
        }
        eprintln!(
            "    outside the edges: ours {:.3e} below / {:.3e} above, \
             MadGraph {:.3e} / {:.3e} (fractions of each sample)",
            our_below / ours.total().max(f64::MIN_POSITIVE),
            our_above / ours.total().max(f64::MIN_POSITIVE),
            mg_below / theirs.total().max(f64::MIN_POSITIVE),
            mg_above / theirs.total().max(f64::MIN_POSITIVE),
        );
        // Zero is the right tolerance here rather than a small fraction: the
        // threshold is a consequence of the pt cut and not a cut of its own, so an
        // event below it needs a lepton below `ptl`, which neither generator's
        // cuts admit. A sampler that placed one there would be generating outside
        // the region it was asked for.
        if mg_below != 0.0 || our_below != 0.0 {
            failures.push(format!(
                "[{}] weight below the {:.0} GeV threshold: ours {our_below:.3e}, \
                 MadGraph {mg_below:.3e}",
                row_spec.run, MLL_EDGES[0],
            ));
        }
    }
    assert!(failures.is_empty(), "dsigma/dm_ll:\n{failures:#?}");
}

/// Whether this checkout has a row's banked run, given what the manifest says the
/// pinned bundle carries.
///
/// The two halves are each other's control: a row marked `bundled = false` may be
/// absent from a fetching checkout, and one the bundle carries may not be. The
/// library's `validate_sigma` states the same rule over its own reference; this is
/// it at the binary's rows.
fn run_present(gate: &str, run: &str) -> bool {
    if run_dir(run).join("Cards/run_card.dat").is_file() {
        return true;
    }
    if manifest::unbundled_rows().contains(run) {
        eprintln!(
            "[{run}] AWAITING BUNDLE (the manifest marks this row bundled = false and this \
             checkout does not have its run, so no cell is written for it)"
        );
        return false;
    }
    vibegraph::validation::require(gate, "a banked run directory", run);
}

/// Whether a row is declared `planned` in the manifest and its MadGraph run is
/// not in this checkout, so there is nothing yet to compare against.
///
/// A banked run is always compared, and an absent run of a row that is not
/// planned still fails in the caller: the declaration, not the absence, is what
/// permits skipping.
fn not_yet_banked(row_spec: &Row) -> bool {
    if run_dir(row_spec.run).join("Cards/run_card.dat").is_file() {
        return false;
    }
    if manifest::planned_rows().contains(row_spec.key) {
        eprintln!(
            "[{}] NOT YET BANKED (the manifest marks row {} planned and this checkout has no \
             run for it, so no cell is written)",
            row_spec.run, row_spec.key
        );
        return true;
    }
    false
}

/// The `p p > j j` sample against MadGraph's banked one — **measured and
/// recorded, not enforced**, on one column.
///
/// The event side of the canonical leading-order QCD row: a purely hadronic
/// two-jet final state whose flavour column spans five MadGraph subprocess
/// directories and 65 concrete assignments, generated through the shipped binary
/// at the run card's own dynamical scale.
///
/// **It is compared by distribution and never by bytes.** MadGraph regenerates a
/// single-group run's events bit-identically, but this run's five subprocess
/// groups make the unweighting draw sensitive to job scheduling, so a re-run of
/// the same card yields a different — equally valid — sample. The banked one is
/// the reference and only its distributions are statements about it.
///
/// **What the `ICOLUP` column measures here.** Each member of a flavour group
/// carries its **own** subprocess's colour-flow table, reordered once into the
/// representative's flow indexing at group construction. Two subprocesses can share
/// a matrix element, a mass list and a colour-factor matrix while routing their
/// colour lines between different pairs of legs — conjugating one end of a line
/// moves it — so a table derived from the representative is not the member's, and
/// this is the first row whose groups mix them: `g q > g q` with `g q̄ > g q̄`,
/// `q q > q q` with `q̄ q̄ > q̄ q̄`, and `q q\' > q q\'` with `q q̄\' > q q̄\'`.
///
/// Three statements at increasing strength, all enforced: every leg's occupied
/// slots are the ones its own colour rep allows, judged against the Les Houches
/// convention with no reference at all; every event's `(roles, connectivity)` is one
/// MadGraph's `leshouche.inc` tables admit, at zero tolerance; and the realised
/// `ICOLUP` frequencies agree with MadGraph's own sample. Each is blind to the next
/// one's failure — legality cannot see a legal-but-wrong flow, membership cannot see
/// a frequency.
///
/// What it is blind to: correlations between columns, a discrepancy confined to a
/// small tail, and the *ordering* of the outgoing legs within an assignment —
/// `canonical()` sorts them by class label and then by `pT`, so leg order enters
/// the flavour key only through `pT`.
#[test]
fn generated_dijet_events_agree_with_madgraphs_banked_ones() {
    if !run_present(
        "generated_dijet_events_agree_with_madgraphs_banked_ones",
        "pp_to_jj",
    ) {
        return;
    }
    check_row(&JJ_ROW);
}

// ───────────── the rows re-carded off MadGraph's internal parton densities ──────
//
// The sample halves of the four runs re-carded from `pdlabel = nn23lo1` onto
// `lhaid = 247000`. Each is the uncut member of a pair whose cut member gates
// above: `pp_to_bb` / `pp_to_bb_qcd2` against `pp_to_bb_fixed` at `ptb = 0`
// rather than 20, `pp_to_llj` against `pp_to_llj_dyn` at `mmll = 0` rather than
// 50, and `pp_to_ll_scalefact2` at `mmll = 0` with the per-event scales doubled.
//
// Every column of all four clears the `1e-4` p-value floor on all three seeds
// with nothing to report, at p-values in the same family as the enforced rows'
// — worst KS `9.9e-3` (pt(ℓ⁺), ℓℓj) and worst χ² `1.6e-1` (flavour, b b̄) — so
// they are enforced rather than reported.

/// σ's uncut `b b̄` counterpart at MadGraph's default coupling orders.
#[test]
fn generated_bb_recarded_events_agree_with_madgraphs_banked_ones() {
    if !run_present(
        "generated_bb_recarded_events_agree_with_madgraphs_banked_ones",
        "pp_to_bb",
    ) {
        return;
    }
    check_row(&BB_ROW);
}

/// The explicit-order spelling of the row above, on its own banked run.
#[test]
fn generated_bb_qcd2_recarded_events_agree_with_madgraphs_banked_ones() {
    if !run_present(
        "generated_bb_qcd2_recarded_events_agree_with_madgraphs_banked_ones",
        "pp_to_bb_qcd2",
    ) {
        return;
    }
    check_row(&BB_QCD2_ROW);
}

/// `pp_to_llj_dyn`'s card with `mmll` back at 0 — the low-mass lepton-pair
/// region the enforced twin is carded away from.
#[test]
fn generated_llj_recarded_events_agree_with_madgraphs_banked_ones() {
    if !run_present(
        "generated_llj_recarded_events_agree_with_madgraphs_banked_ones",
        "pp_to_llj",
    ) {
        return;
    }
    check_row(&LLJ_ROW);
}

/// Drell-Yan with the event-by-event scales doubled: the only banked sample
/// whose `scalefact` is not 1, so it is where a sample carries the run card's
/// scale factor rather than only a cross section doing so.
#[test]
fn generated_scalefact2_events_agree_with_madgraphs_banked_ones() {
    if !run_present(
        "generated_scalefact2_events_agree_with_madgraphs_banked_ones",
        "pp_to_ll_scalefact2",
    ) {
        return;
    }
    check_row(&SCALEFACT2_ROW);
}
