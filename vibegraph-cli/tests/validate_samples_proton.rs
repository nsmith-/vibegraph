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

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use flate2::read::MultiGzDecoder;
use vibegraph::lhef::observables::Labelling;
use vibegraph::lhef::parse::LheFile;
use vibegraph::validation::samples::{compare, labelling_for, EventSample, Spectrum};

#[path = "../../vibegraph-lib/tests/common/report.rs"]
mod report;

#[path = "../../vibegraph-lib/tests/common/manifest.rs"]
mod manifest;

#[path = "../../vibegraph-lib/tests/common/leshouche.rs"]
mod leshouche;

use report::{CategoryCount, Chi2Cell, KsCell, SamplesRow, SeedSample};

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
}

/// One per-event scan: `(name, unit, run)`, where `run` returns `(bad, total)`.
type Scan = (&'static str, &'static str, fn(&Row, &EventSample) -> (usize, usize));

fn validation_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph")
}


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
        let integrate = Command::new(env!("CARGO_BIN_EXE_vibegraph"))
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
                INTEGRATION_SEED,
            ])
            .output()
            .expect("spawn vibegraph");
        assert!(
            integrate.status.success(),
            "vibegraph integrate failed:\n{}",
            String::from_utf8_lossy(&integrate.stderr)
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
        let generate = Command::new(env!("CARGO_BIN_EXE_vibegraph"))
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
            String::from_utf8_lossy(&generate.stderr)
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

/// One row: integrate the banked cards once, then generate and compare a sample
/// per seed.
///
/// `mode` is `gate` for a row whose columns agree and `info` for one carrying a
/// recorded disagreement: the measurement is taken and reported either way, and
/// the mode says only whether a column below the floor fails the suite. A row is
/// demoted with a note saying what was measured and where the fix is tracked —
/// never by widening the floor.
fn check_row(row_spec: &Row) {
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

    let mut row = SamplesRow::new(key, process, mode);
    row.variant = variant.map(str::to_string);
    row.p_floor = P_FLOOR;
    row.mg_events = mg.len();
    row.sigma_mg_pb = mg.sigma_pb;
    row.labelling = "coarse";
    let mut failures: Vec<String> = Vec::new();

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

        let labelling = labelling_for(&ours, &mg);
        let found = compare(&ours, &mg, labelling);
        let worst = found.worst_ks().expect("a comparable observable");
        eprintln!(
            "  seed {seed:#010x} | {} events (n_eff {:.0}) | worst KS: {} p {:.3e} (D {:.4})",
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
        for cell in &found.ks {
            if cell.p < P_FLOOR {
                failures.push(format!(
                    "seed {seed:#010x} KS {} p {:.3e} (D {:.4}) below the {P_FLOOR:.0e} floor",
                    cell.observable, cell.p, cell.d
                ));
            }
        }
        for cell in &found.chi2 {
            if cell.p < P_FLOOR {
                failures.push(format!(
                    "seed {seed:#010x} chi2 {} p {:.3e} ({:.1}/{} dof) below the \
                     {P_FLOOR:.0e} floor",
                    cell.column, cell.p, cell.chi2, cell.dof
                ));
            }
        }

        row.constant_observables = found.constant.clone();
        row.single_category = found
            .single_category
            .iter()
            .map(|c| (*c).to_string())
            .collect();
        row.per_seed.push(SeedSample {
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
        });
    }

    row.finish();
    eprintln!(
        "  min KS p {:.3e} ({}), min chi2 p {:.3e} ({}) over {} seeds",
        row.min_ks_p,
        row.worst_ks_observable,
        row.min_chi2_p,
        row.worst_chi2_column,
        GEN_SEEDS.len()
    );
    row.status = match mode {
        "gate" => {
            if failures.is_empty() {
                "pass"
            } else {
                "fail"
            }
        }
        _ => "info",
    };
    row.write();

    if mode != "gate" {
        if !failures.is_empty() {
            eprintln!("  [{run}] measured, not enforced:\n{failures:#?}");
        }
        return;
    }
    assert!(failures.is_empty(), "[{run}] samples gate:\n{failures:#?}");
}

#[test]
fn generated_proton_events_agree_with_madgraphs_banked_ones() {
    check_row(&Row {
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
    });
}

/// The scale-moved twin of the row above: same generate line, same budget, the
/// run's own dynamical-scale card instead of the fixed one. The two run cards
/// differ only in the three `fixed_*_scale` booleans, so this cell is what the
/// per-point `AMP2`-drawn configuration change actually exercises end to end —
/// a sample assembled by the binary rather than a scalar cross section.
#[test]
fn generated_llj_dyn_events_agree_with_madgraphs_banked_ones() {
    check_row(&Row {
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
    });
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
    check_row(&Row {
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
    });
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
        let mg = banked_sample(row_spec.run, row_spec.events);
        let mut theirs = Spectrum::new(MLL_EDGES);
        theirs.fill_from(&mg, "m(e+,e-)", Labelling::Fine);

        let generator = Generator::integrate(row_spec);
        let mut ours = Spectrum::new(MLL_EDGES);
        let mut sigma_sum = 0.0;
        for &seed in &GEN_SEEDS {
            let sample = generator.sample(seed);
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
    check_row(&Row {
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
    });
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
    check_row(&Row {
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
    });
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
    check_row(&Row {
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
    });
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
    check_row(&Row {
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
    });
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
    check_row(&Row {
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
    });
}
