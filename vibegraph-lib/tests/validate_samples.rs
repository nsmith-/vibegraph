//! The `samples` category: our unweighted events against MadGraph's banked ones,
//! distribution by distribution.
//!
//! Every other per-process category compares a *number* — a diagram count, an
//! amplitude at a point, a cross section. This one compares the sample a
//! generator actually emits, which is the only place several things become
//! visible at all:
//!
//! * a cross section is one scalar and is blind to a region of small measure
//!   being mis-sampled, which is exactly what a wrong channel map or a wrong
//!   selection rule produces;
//! * `SPINUP` and `ICOLUP` move no weight, so a mislabelled event leaves σ and
//!   every shape untouched. The rules behind them (`∝ |M_hel|²`, `∝ JAMP2`) are
//!   pinned in the library's own tests and the flow tags against
//!   `leshouche.inc`, but until here nothing had compared the *realised*
//!   frequencies against MadGraph's realised frequencies.
//!
//! # How the two samples are made comparable
//!
//! Both sides are read as Les Houches records — MadGraph's from its banked
//! `unweighted_events.lhe.gz`, ours by building the same record type out of an
//! accepted point through [`SubprocessRecord`], the production assembly. From
//! there one code path serves both: [`observables::canonical`] puts the final
//! state in an order derived from the event, and the named kinematic and
//! categorical columns follow.
//!
//! The rows here are the fixed-beam ones, where `ebeam1 = ebeam2` makes the lab
//! frame the partonic centre of mass, so our momenta and MadGraph's live in the
//! same frame without a boost. That is asserted per row rather than assumed.
//!
//! # Weights
//!
//! Our sample is *nearly* unweighted: a point above its channel's scanned maximum
//! is kept at `w/w_max > 1`. Those events are the tail hardest to sample, so they
//! are carried into the comparison as weights ([`stats::ks_two_sample`] takes the
//! weighted empirical CDF) rather than rounded to one. MadGraph's `XWGTUP` is
//! carried the same way, so a run whose events do not all share a weight is
//! handled without a special case.
//!
//! # What this gate provably cannot detect
//!
//! * Anything both sides get right about *shape* and wrong about *normalisation*:
//!   every statistic here is computed on normalised distributions, and the
//!   absolute cross section is the `integrals` category's business.
//! * Correlations between columns. Each observable is compared on its own
//!   marginal, so two samples with identical marginals and different correlations
//!   pass.
//! * A discrepancy confined to a small tail. The KS statistic is a maximum CDF
//!   gap and is least sensitive exactly there — which is why the low-`m_ll`
//!   question is decided by a binned comparison
//!   ([`the_low_m_ll_region_is_binned_against_madgraph`]) and not by a p-value.

use std::io::Read;
use std::path::{Path, PathBuf};

use flate2::read::MultiGzDecoder;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use vibegraph::cuts::Cuts;
use vibegraph::hadronic::{
    compile_subprocesses, initial_spin_color_average, process_external_legs, FixedBeamIntegrand,
};
use vibegraph::helas::eval::BoundAmplitude;
use vibegraph::lhef::build::{EventHeader, SubprocessRecord};
use vibegraph::lhef::observables::{canonical, kinematics, Labelling};
use vibegraph::lhef::parse::LheFile;
use vibegraph::phasespace::diagram_channel::DiagramChannel;
use vibegraph::phasespace::GEV2_TO_PB;
use vibegraph::runcard::{BeamMode, RunCard};
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::EvaluatedModel;
use vibegraph::unweight::Unweighter;
use vibegraph::validation::samples::{compare, labelling_for, Chi2Column, EventSample, Spectrum};

mod common;

use common::report::{CategoryCount, Chi2Cell, KsCell, SamplesRow, SeedSample};

/// α-adaptation budget for the multichannel combiner, matching the σ gate's, so
/// the grids the events are drawn on are the ones that gate integrates over.
const MULTICHANNEL_SURVEY: usize = 30_000;
const MULTICHANNEL_ITERS: usize = 6;

/// Integration seed, shared by every row.
const SEED: u64 = 20_260_719;
/// Seed for the per-channel weight scan, distinct from the integration's and from
/// every generation seed.
const SCAN_SEED: u64 = 0x5CA7_0FF0;
/// Independent generation seeds. One seed is a single draw from the null
/// distribution of every statistic below and proves little on its own; the sweep
/// is what separates a shape that disagrees from a sample that was unlucky.
const GEN_SEEDS: [u64; 3] = [0x5A_4D_0001, 0x5A_4D_0002, 0x5A_4D_0003];
/// Events per generation seed, against MadGraph's banked 10 000.
const EVENTS_PER_SEED: usize = 20_000;
/// Trials a seed may spend before its event count is called short. Sized well
/// above `1/efficiency` for the least efficient row here.
const MAX_TRIALS_PER_EVENT: usize = 400;

/// The p-value a column must clear.
///
/// Chosen from the trial count, not from taste. A run takes the smallest p over
/// every observable of every gating row on every seed: ten rows, three seeds and
/// seven to twenty-one observables each, and the observables of a `2 → 2` row are
/// heavily correlated (both legs' `pT` are one number at fixed beams), so the
/// draws from the null distribution number a few hundred rather than a few
/// thousand. At a floor of `1e-3` that is an expected 0.2 to 0.4 spurious
/// failures per run, which would make the gate flap; at `1e-4` it is under 0.05.
///
/// The measured minimum over the gating rows and three seeds is `2.03e-3`
/// (`ee_to_tatah`, `m(ta+,ta-)`), with `ee_to_mumua` at `2.14e-3`, `ee_to_ee` at
/// `4.02e-3` and `uux_to_uux` at `6.67e-3` behind it — so agreement produces
/// minima an order of magnitude above the floor, while the two rows that disagree
/// produce `3.6e-6` and `0`. There is a wide gap to sit in, and this sits in it.
///
/// Never loosened after a failure: a column that falls below this is recorded,
/// the row is marked informational with the measurement in its note, and the
/// disagreement is filed — the threshold does not move.
const P_FLOOR: f64 = 1e-4;

/// One process to compare, with the integration budget its grids are built on.
struct Row {
    key: &'static str,
    process: &'static str,
    neval: usize,
    niter: usize,
    /// `gate` for a row whose columns agree, `info` for one carrying a recorded
    /// disagreement: the measurement is taken and reported either way, and the
    /// mode says whether a failure fails the suite. A row is demoted only with a
    /// note saying what was measured and where the fix is tracked — never by
    /// widening a threshold.
    mode: &'static str,
}

/// Every fixed-beam row whose `samples` cell the manifest puts in the banked
/// layer. The budgets follow the σ gate's: enough that the frozen grids the
/// events are drawn on are the ones that gate integrates over.
const ROWS: &[Row] = &[
    Row {
        key: "ee_to_mumu",
        process: "e+ e- > mu+ mu-",
        neval: 20_000,
        niter: 4,
        mode: "gate",
    },
    Row {
        key: "ee_to_ee",
        process: "e+ e- > e+ e-",
        neval: 30_000,
        niter: 5,
        mode: "gate",
    },
    Row {
        key: "ee_to_ttx",
        process: "e+ e- > t t~",
        neval: 20_000,
        niter: 4,
        mode: "gate",
    },
    Row {
        key: "ee_to_wpwm",
        process: "e+ e- > w+ w-",
        neval: 20_000,
        niter: 4,
        mode: "gate",
    },
    Row {
        key: "ee_to_zh",
        process: "e+ e- > z h",
        neval: 20_000,
        niter: 4,
        mode: "gate",
    },
    Row {
        key: "uux_to_mumu",
        process: "u u~ > mu+ mu-",
        neval: 20_000,
        niter: 4,
        mode: "gate",
    },
    Row {
        key: "uux_to_uux",
        process: "u u~ > u u~",
        neval: 30_000,
        niter: 5,
        mode: "info",
    },
    Row {
        key: "gg_to_ttx",
        process: "g g > t t~",
        neval: 30_000,
        niter: 5,
        mode: "gate",
    },
    Row {
        key: "gg_to_gg",
        process: "g g > g g",
        neval: 30_000,
        niter: 5,
        mode: "gate",
    },
    Row {
        key: "ee_to_mumua",
        process: "e+ e- > mu+ mu- a",
        neval: 40_000,
        niter: 5,
        mode: "gate",
    },
    Row {
        key: "ee_to_tatah",
        process: "e+ e- > ta+ ta- h",
        neval: 30_000,
        niter: 5,
        mode: "gate",
    },
    Row {
        key: "ee_to_mumu_tata_qcd0",
        process: "e+ e- > mu+ mu- ta+ ta- QCD=0",
        neval: 60_000,
        niter: 6,
        mode: "info",
    },
];

/// The four `l+ l- j` partonic rows, whose run cards leave both scales free.
/// Their `integrals` cells are blocked on `kt-clustering`; this list is what
/// [`the_llj_parton_rows_cannot_be_generated_either`] measures the same refusal
/// on, so their `samples` cells are blocked for a reason that was checked.
const REFUSED_ROWS: &[Row] = &[
    Row {
        key: "uux_to_epemg",
        process: "u u~ > e+ e- g QCD=2 QED=2",
        neval: 0,
        niter: 0,
        mode: "blocked",
    },
    Row {
        key: "ddx_to_epemg",
        process: "d d~ > e+ e- g QCD=2 QED=2",
        neval: 0,
        niter: 0,
        mode: "blocked",
    },
    Row {
        key: "gu_to_epemu",
        process: "g u > e+ e- u QCD=2 QED=2",
        neval: 0,
        niter: 0,
        mode: "blocked",
    },
    Row {
        key: "gux_to_epemux",
        process: "g u~ > e+ e- u~ QCD=2 QED=2",
        neval: 0,
        niter: 0,
        mode: "blocked",
    },
];

fn output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/output")
}

fn param_card(dir: &str) -> ParamCard {
    let path = output_dir().join(dir).join("Cards/param_card.dat");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.parse::<ParamCard>().ok())
        .unwrap_or_else(|| "".parse::<ParamCard>().unwrap())
}

/// MadGraph's banked events for a run, as a sample.
fn banked_sample(dir: &str) -> EventSample {
    let path = output_dir()
        .join(dir)
        .join("Events/run_01/unweighted_events.lhe.gz");
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

/// Build the production fixed-energy integrand for a banked process and hand it
/// to `f`, together with one record assembler per compiled subprocess.
fn with_integrand<R>(
    row: &Row,
    f: impl FnOnce(&FixedBeamIntegrand, &[SubprocessRecord], &RunCard) -> R,
) -> R {
    let card_path = output_dir().join(row.key).join("Cards/run_card.dat");
    let run_card = RunCard::parse_file(&card_path).expect("real run card parses");
    assert_eq!(run_card.beam_mode(), BeamMode::FixedEnergy);
    assert_eq!(
        run_card.ebeam1, run_card.ebeam2,
        "[{}] the comparison assumes the lab frame is the partonic centre of mass",
        row.key
    );
    let sqrt_s = run_card.ebeam1 + run_card.ebeam2;

    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &param_card(row.key));

    let sets = common::generate(row.process);
    let evals = compile_subprocesses(&sets, &model, &evaluated).expect("compile subprocesses");
    let bounds: Vec<_> = evals
        .iter()
        .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
        .collect();
    let records: Vec<SubprocessRecord> = evals
        .iter()
        .map(|e| SubprocessRecord::new(e, &model, &evaluated).expect("subprocess record"))
        .collect();

    let rep = &evals[0];
    let legs = process_external_legs(rep, &model, &evaluated);
    let cuts = Cuts::compile(&run_card, &legs).expect("run card cuts compile");
    let final_masses: Vec<f64> = rep.external_particles()[rep.n_in()..]
        .iter()
        .map(|&id| evaluated.mass(id))
        .collect();
    let spin_color_avg = initial_spin_color_average(rep, &model, &evaluated);
    let diagrams: Vec<_> = sets
        .iter()
        .flat_map(|s| s.diagrams.iter().cloned())
        .collect();

    let amps: Vec<&BoundAmplitude<f64>> = bounds.iter().collect();
    let mut integ = FixedBeamIntegrand::new(amps, &cuts, sqrt_s, final_masses, spin_color_avg);
    integ
        .use_running_coupling(&diagrams, &model, &evaluated, &run_card)
        .expect("run card scale prescription compiles");
    integ.use_multichannel(
        &diagrams,
        &evaluated,
        MULTICHANNEL_SURVEY,
        MULTICHANNEL_ITERS,
        SEED,
    );
    f(&integ, &records, &run_card)
}

/// Generate one seed's worth of events off frozen grids.
///
/// The header's scalar fields play no part in any observable here, so they are
/// left as NaN rather than filled with a plausible-looking number: this record is
/// a view of an event's legs, not a file's event.
fn generate(
    integ: &FixedBeamIntegrand,
    records: &[SubprocessRecord],
    uw: &mut Unweighter,
    seed: u64,
) -> EventSample {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut momenta = Vec::new();
    let mut events = Vec::with_capacity(EVENTS_PER_SEED);
    let mut weights = Vec::with_capacity(EVENTS_PER_SEED);
    let before = uw.stats().clone();
    let budget = EVENTS_PER_SEED * MAX_TRIALS_PER_EVENT;
    let mut trials = 0usize;
    while events.len() < EVENTS_PER_SEED && trials < budget {
        trials += 1;
        let Some(point) = uw.trial(integ, &mut rng) else {
            continue;
        };
        integ.event_in_channel(point.channel, &point.u, &mut momenta);
        let u = [rng.random(), rng.random(), rng.random()];
        let Some(selection) = integ.select_event(&momenta, u) else {
            continue;
        };
        let beams = integ.beams();
        let mut external: Vec<[f64; 4]> = vec![
            [beams[0].e(), beams[0].px(), beams[0].py(), beams[0].pz()],
            [beams[1].e(), beams[1].px(), beams[1].py(), beams[1].pz()],
        ];
        external.extend(momenta.iter().map(|p| [p.e(), p.px(), p.py(), p.pz()]));
        let header = EventHeader {
            process_id: 1,
            weight: point.weight,
            scale: f64::NAN,
            alpha_qed: f64::NAN,
            alpha_qcd: f64::NAN,
        };
        let event = records[selection.subprocess]
            .event(&external, &selection.helicity, selection.flow, header)
            .expect("an accepted point assembles into a record");
        events.push(event);
        weights.push(point.weight);
    }
    let after = uw.stats();
    let weight_sum = after.event_weight_sum - before.event_weight_sum;
    let sigma_pb = uw.total_w_max() * GEV2_TO_PB * weight_sum / trials as f64;
    EventSample {
        events,
        weights,
        sigma_pb,
    }
}

/// Compare one generated sample against MadGraph's, filling in a report row's
/// per-seed entry and returning the columns that fell below the floor.
fn compare_seed(
    key: &str,
    seed: u64,
    ours: &EventSample,
    theirs: &EventSample,
    labelling: Labelling,
    row: &mut SamplesRow,
) -> Vec<String> {
    let found = compare(ours, theirs, labelling);
    let worst = found
        .worst_ks()
        .expect("a row has one comparable observable");
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

    let mut below = Vec::new();
    for cell in &found.ks {
        if cell.p < P_FLOOR {
            below.push(format!(
                "[{key}] seed {seed:#010x} KS {} p {:.3e} (D {:.4}) below the {P_FLOOR:.0e} floor",
                cell.observable, cell.p, cell.d
            ));
        }
    }
    for cell in &found.chi2 {
        if cell.p < P_FLOOR {
            below.push(format!(
                "[{key}] seed {seed:#010x} chi2 {} p {:.3e} ({:.1}/{} dof) below the \
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
        chi2: found.chi2.iter().map(chi2_cell).collect(),
    });
    below
}

/// A shared comparison's χ² result as the report row records it.
fn chi2_cell(cell: &Chi2Column) -> Chi2Cell {
    Chi2Cell {
        column: cell.column.to_string(),
        chi2: cell.chi2,
        dof: cell.dof,
        p: cell.p,
        categories: cell.categories,
        distinct_keys: cell.distinct_keys,
        pooled_share: cell.pooled_share,
        detail: cell
            .detail
            .iter()
            .map(|(key, ours, theirs)| CategoryCount {
                key: key.clone(),
                ours: *ours,
                theirs: *theirs,
            })
            .collect(),
    }
}

#[test]
fn unweighted_samples_agree_with_madgraphs_banked_ones() {
    let mut failures: Vec<String> = Vec::new();
    // Columns below the floor on a row the manifest marks informational: reported
    // in full, never enforced, and tracked in the backlog instead.
    let mut informational: Vec<String> = Vec::new();
    for row in ROWS {
        let mg = banked_sample(row.key);
        eprintln!(
            "-- {} ({} banked events, sigma {:.6e} pb) --",
            row.key,
            mg.len(),
            mg.sigma_pb
        );
        with_integrand(row, |integ, records, _| {
            let (channels, _) = integ.adapt_grids(row.neval, row.niter, SEED);
            let mut uw = Unweighter::scan(
                integ,
                channels.iter().map(|c| (&c.grid, c.neval)),
                SCAN_SEED,
            );
            let mut report = SamplesRow::new(row.key, row.process, row.mode);
            report.p_floor = P_FLOOR;
            report.mg_events = mg.len();
            report.sigma_mg_pb = mg.sigma_pb;
            let mut labelling = None;
            for &seed in &GEN_SEEDS {
                let ours = generate(integ, records, &mut uw, seed);
                if ours.len() < EVENTS_PER_SEED {
                    failures.push(format!(
                        "[{}] seed {seed:#010x} produced {} of {EVENTS_PER_SEED} events",
                        row.key,
                        ours.len()
                    ));
                }
                let l = *labelling.get_or_insert_with(|| labelling_for(&ours, &mg));
                report.labelling = match l {
                    Labelling::Fine => "fine",
                    Labelling::Coarse => "coarse",
                };
                let found = compare_seed(row.key, seed, &ours, &mg, l, &mut report);
                if row.mode == "gate" {
                    failures.extend(found);
                } else {
                    informational.extend(found);
                }
            }
            report.finish();
            eprintln!(
                "  min KS p {:.3e}, min chi2 p {:.3e} over {} seeds",
                report.min_ks_p,
                report.min_chi2_p,
                GEN_SEEDS.len()
            );
            report.status = match row.mode {
                "gate" => {
                    if report.min_ks_p >= P_FLOOR && report.min_chi2_p >= P_FLOOR {
                        "pass"
                    } else {
                        "fail"
                    }
                }
                _ => "info",
            };
            report.write();
        });
    }
    if !informational.is_empty() {
        eprintln!(
            "informational rows below the floor (measured, not enforced):\n{informational:#?}"
        );
    }
    assert!(failures.is_empty(), "samples gate failures:\n{failures:#?}");
}

/// The mutation probe: the statistics have to *catch* something, or a run of
/// green cells says nothing.
///
/// Each pair below is two banked MadGraph samples of processes that share an
/// observable naming — same final-state classes, genuinely different kinematics.
/// Compared as if they were the same process, every pair must fall below the same
/// floor the gate above enforces, and by orders of magnitude: this is what makes
/// a passing row evidence rather than an artefact of a statistic that never
/// rejects.
#[test]
fn the_gate_rejects_a_sample_from_a_different_process() {
    let pairs = [
        ("ee_to_mumu", "uux_to_mumu"),
        ("ee_to_ttx", "gg_to_ttx"),
        ("uux_to_uux", "gg_to_gg"),
    ];
    for (a, b) in pairs {
        let (sa, sb) = (banked_sample(a), banked_sample(b));
        let l = labelling_for(&sa, &sb);
        let worst = compare(&sa, &sb, l)
            .worst_ks()
            .cloned()
            .expect("the two processes share a comparable observable");
        eprintln!(
            "  {a} against {b}: smallest KS p {:.3e} on {}",
            worst.p, worst.observable
        );
        assert!(
            worst.p < P_FLOOR,
            "{a} against {b} passed the {P_FLOOR:.0e} floor at p = {:.3e}",
            worst.p
        );
    }
}

/// The four `l+ l- j` partonic rows cannot be generated for the same reason they
/// cannot be integrated, and this measures it rather than assuming it.
///
/// Their run cards leave `dynamical_scale_choice = -1`, and a t-channel
/// propagator into a three-leg final state is the topology whose cluster scale
/// depends on the merge order — which `coupling::scales` refuses. Event
/// generation runs through the same integrand, so the refusal lands in the same
/// place, and their `samples` cells are blocked on `kt-clustering` alongside their
/// `integrals` cells.
#[test]
fn the_llj_parton_rows_cannot_be_generated_either() {
    for row in REFUSED_ROWS {
        let card_path = output_dir().join(row.key).join("Cards/run_card.dat");
        let run_card = RunCard::parse_file(&card_path).expect("real run card parses");
        let model = common::sm_model();
        let evaluated = EvaluatedModel::from_model_card(model.clone(), &param_card(row.key));
        let sets = common::generate(row.process);
        let evals = compile_subprocesses(&sets, &model, &evaluated).expect("compile subprocesses");
        let bounds: Vec<_> = evals
            .iter()
            .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
            .collect();
        let rep = &evals[0];
        let legs = process_external_legs(rep, &model, &evaluated);
        let cuts = Cuts::compile(&run_card, &legs).expect("run card cuts compile");
        let final_masses: Vec<f64> = rep.external_particles()[rep.n_in()..]
            .iter()
            .map(|&id| evaluated.mass(id))
            .collect();
        let spin_color_avg = initial_spin_color_average(rep, &model, &evaluated);
        let diagrams: Vec<_> = sets
            .iter()
            .flat_map(|s| s.diagrams.iter().cloned())
            .collect();
        let mut integ = FixedBeamIntegrand::new(
            bounds.iter().collect(),
            &cuts,
            run_card.ebeam1 + run_card.ebeam2,
            final_masses,
            spin_color_avg,
        );
        let refusal = integ.use_running_coupling(&diagrams, &model, &evaluated, &run_card);
        let message = match refusal {
            Ok(_) => panic!(
                "[{}] the scale prescription was accepted, so this row is no longer blocked",
                row.key
            ),
            Err(e) => e.to_string(),
        };
        eprintln!("  {} refuses generation: {message}", row.key);
    }
}

/// Bin edges for the `m_ll` decider, in GeV: fine where the photon pole lives and
/// the two samples part company, coarse where they agree.
const MLL_EDGES: &[f64] = &[
    4.0, 6.0, 8.0, 10.0, 12.0, 15.0, 20.0, 25.0, 30.0, 40.0, 60.0, 80.0, 88.0, 92.0, 100.0, 110.0,
    120.0, 124.0, 124.9, 125.1, 126.0, 130.0, 140.0, 150.0, 200.0, 250.0, 350.0, 500.0,
];

/// Budget of the independent flat-RAMBO estimate that decides which side of the
/// low-`m_ll` disagreement is wrong.
const FLAT_NEVAL: usize = 120_000;
const FLAT_NITER: usize = 8;
const FLAT_DRAWS: usize = 600_000;
const FLAT_SEED: u64 = 0x_F1A7_0001;

/// The `low-mll-reconciliation` decider: `dσ/dm_ll` down to threshold for
/// `e+ e- > mu+ mu- ta+ ta-`, in absolute picobarns, from three independent
/// estimates.
///
/// The standing discrepancy is that this row's σ sits `+2.2%` above the banked
/// MadGraph value, entirely below `m_ll ≈ 20 GeV`, and a scalar cross section
/// cannot say whether MadGraph under-covers that region or this sampler
/// over-weights it. Both sides' *samples* say where the difference sits, which
/// is necessary but still symmetric — so a third estimate breaks the tie:
///
/// * **the production sampler** — per-diagram multichannel maps, the one under
///   suspicion;
/// * **MadGraph's banked events**, scaled to MadGraph's own banked σ;
/// * **flat RAMBO under a VEGAS grid** — the same matrix element and the same
///   cuts, reached through a phase-space map that shares nothing with either of
///   the other two. It is a poor sampler for a pole and its errors say so, but it
///   is not *this* sampler and it is not MadGraph's.
///
/// # The verdict
///
/// **The premise is wrong.** Every `m_ll` bin below 20 GeV agrees within its own
/// errors, on both pairs. What carries the offset is a single 200 MeV bin at the
/// `h → τ⁺τ⁻` pole in `m(ta+,ta-)`:
///
/// ```text
/// m(ta+,ta-) in [124.9, 125.1]   production 7.137e-5 +- 1.3e-6 pb
///                                MadGraph   2.260e-5 +- 1.7e-6 pb
///                                difference +4.876e-5 pb = 159% of the +3.06e-5 pb offset
/// ```
///
/// a factor of 3.16 and a 22σ disagreement in one bin, around a resonance
/// 6.4 MeV wide. Nothing else in either spectrum deviates by more than a few
/// times its error, and the rest of the spectrum sits about 1.4% *below*
/// MadGraph, so the offset is a resonance question and not a threshold question.
///
/// The third estimate here does not settle *which* side is wrong: flat RAMBO
/// under a VEGAS grid puts 1.0e-6 pb in that bin, twenty times under MadGraph,
/// because a map with no Breit–Wigner cannot find a 6.4 MeV peak in a 500 GeV
/// process — it shows only the direction a poor map fails in, and its per-bin
/// ratios elsewhere (0.004 to 36) say it is not converged.
///
/// [`the_higgs_pole_window_is_measured_against_madgraph`] settles it, by asking
/// MadGraph for the same window directly: MadGraph's own windowed cross section
/// is 7.2077e-5 pb, so the deficit is in MadGraph's *unwindowed* integration and
/// not in this sampler. Both cells stay informational because the banked
/// reference they compare against is the run that carries the deficit.
#[test]
fn the_low_m_ll_region_is_binned_against_madgraph() {
    let row = ROWS
        .iter()
        .find(|r| r.key == "ee_to_mumu_tata_qcd0")
        .expect("the decider row is in the table");
    let mg = banked_sample(row.key);

    // MadGraph's side, from its own events and its own cross section.
    let mut mg_mumu = Spectrum::new(MLL_EDGES);
    let mut mg_tata = Spectrum::new(MLL_EDGES);
    for (event, &w) in mg.events.iter().zip(&mg.weights) {
        let event = canonical(event, Labelling::Fine);
        for (name, value) in kinematics(&event, Labelling::Fine) {
            match name.as_str() {
                "m(mu+,mu-)" => mg_mumu.fill(value, w),
                "m(ta+,ta-)" => mg_tata.fill(value, w),
                _ => {}
            }
        }
    }

    with_integrand(row, |integ, records, _| {
        // 1. The production sampler, pooled over the same seeds the gate uses.
        let (channels, _) = integ.adapt_grids(row.neval, row.niter, SEED);
        let mut uw = Unweighter::scan(
            integ,
            channels.iter().map(|c| (&c.grid, c.neval)),
            SCAN_SEED,
        );
        let mut ours_mumu = Spectrum::new(MLL_EDGES);
        let mut ours_tata = Spectrum::new(MLL_EDGES);
        let mut sigma_sum = 0.0;
        for &seed in &GEN_SEEDS {
            let sample = generate(integ, records, &mut uw, seed);
            sigma_sum += sample.sigma_pb;
            for (event, &w) in sample.events.iter().zip(&sample.weights) {
                let event = canonical(event, Labelling::Fine);
                for (name, value) in kinematics(&event, Labelling::Fine) {
                    match name.as_str() {
                        "m(mu+,mu-)" => ours_mumu.fill(value, w),
                        "m(ta+,ta-)" => ours_tata.fill(value, w),
                        _ => {}
                    }
                }
            }
        }
        let ours_sigma = sigma_sum / GEN_SEEDS.len() as f64;

        // 2. Flat RAMBO under a VEGAS grid: an unweighted sample is not needed, the
        // weighted estimator is the estimate.
        let (grid, result) = integ.adapt_grid(FLAT_NEVAL, FLAT_NITER, FLAT_SEED);
        let flat_sigma = result.integral * GEV2_TO_PB;
        let mut flat_mumu = Spectrum::new(MLL_EDGES);
        let mut flat_tata = Spectrum::new(MLL_EDGES);
        let mut rng = ChaCha8Rng::seed_from_u64(FLAT_SEED ^ 0xFFFF);
        let mut u = vec![0.0; integ.channel_grid_ndim()];
        let mut momenta = Vec::new();
        for _ in 0..FLAT_DRAWS {
            let jac = grid.draw(&mut rng, &mut u);
            let w = jac * integ.event_in_channel(0, &u, &mut momenta);
            if !(w > 0.0) {
                continue;
            }
            let m = |a: usize, b: usize| (momenta[a] + momenta[b]).m();
            flat_mumu.fill(m(0, 1), w);
            flat_tata.fill(m(2, 3), w);
        }
        eprintln!(
            "-- low-m_ll decider: {} --\n  \
             sigma: production sampler {ours_sigma:.6e} pb, MadGraph {:.6e} pb \
             (rel {:+.3}%), flat RAMBO + VEGAS {flat_sigma:.6e} +- {:.1e} pb (rel {:+.3}%)",
            row.key,
            mg.sigma_pb,
            100.0 * (ours_sigma / mg.sigma_pb - 1.0),
            result.std_dev * GEV2_TO_PB,
            100.0 * (flat_sigma / mg.sigma_pb - 1.0),
        );

        for (label, ours, theirs, flat) in [
            ("m(mu+,mu-)", &ours_mumu, &mg_mumu, &flat_mumu),
            ("m(ta+,ta-)", &ours_tata, &mg_tata, &flat_tata),
        ] {
            let a = ours.as_sigma(ours_sigma);
            let b = theirs.as_sigma(mg.sigma_pb);
            let c = flat.as_sigma(flat_sigma);
            let excess = ours_sigma - mg.sigma_pb;
            eprintln!(
                "  {label} [pb per bin; `share` is the bin's part of the \
                 {excess:+.3e} pb total excess]\n    \
                 {:>13} {:>22} {:>22} {:>22} {:>11} {:>7} {:>7}",
                "bin", "production", "MadGraph", "flat RAMBO", "prod-MG", "share", "flat/MG"
            );
            for ((prod, mgb), flatb) in a.iter().zip(&b).zip(&c) {
                if mgb.sigma_pb <= 0.0 && prod.sigma_pb <= 0.0 {
                    continue;
                }
                eprintln!(
                    "    {:>6.1}-{:<6.1} {:>12.5e} +- {:>7.1e} {:>12.5e} +- {:>7.1e} \
                     {:>12.5e} +- {:>7.1e} {:>+11.3e} {:>6.0}% {:>7}",
                    prod.low,
                    prod.high,
                    prod.sigma_pb,
                    prod.err_pb,
                    mgb.sigma_pb,
                    mgb.err_pb,
                    flatb.sigma_pb,
                    flatb.err_pb,
                    prod.sigma_pb - mgb.sigma_pb,
                    100.0 * (prod.sigma_pb - mgb.sigma_pb) / excess,
                    ratio(flatb.sigma_pb, mgb.sigma_pb),
                );
            }
            eprintln!(
                "    below {:.0}: production {:.3e} pb, MadGraph {:.3e} pb, flat {:.3e} pb",
                MLL_EDGES[0],
                ours.outside().0 / ours.total() * ours_sigma,
                theirs.outside().0 / theirs.total() * mg.sigma_pb,
                flat.outside().0 / flat.total() * flat_sigma,
            );
        }
    });
}

fn ratio(a: f64, b: f64) -> String {
    if b > 0.0 {
        format!("{:.3}", a / b)
    } else {
        "-".to_string()
    }
}

/// Draws for the windowed measurement. The single-channel estimator's error at
/// this budget is well under a percent, against a disagreement that was a factor
/// of three.
const HWINDOW_DRAWS: usize = 400_000;
const HWINDOW_SEED: u64 = 0x_B1_0000;
/// Agreement the windowed cross sections must show. Sized on what the check is
/// for — the banked disagreement it replaces was a factor 3.16 — and left well
/// above the two estimates' combined Monte-Carlo errors (0.4% and 0.5%) so a
/// budget change cannot make it flap.
const HWINDOW_TOLERANCE: f64 = 0.02;

/// `σ` over the 200 MeV window at the `h → τ⁺τ⁻` pole, on both sides, measured
/// directly rather than read off a histogram.
///
/// This is what decides the resonance question
/// [`the_low_m_ll_region_is_binned_against_madgraph`] exposed. Two histograms of
/// the same bin disagreeing is symmetric evidence; a cross section *of that bin*
/// from each generator is not.
///
/// # MadGraph's side
///
/// `validation/madgraph/gen_higgs_window.sh` runs the banked process three ways
/// with the banked run card — inside the window, outside it, and unwindowed over
/// three seeds — and banks the scalars in `higgs_window_reference.json`. The
/// window is imposed through `dummy_cuts`, because the only run-card cut that
/// constrains a lepton-pair mass (`mmll`/`mmllmax`) is applied by `setcuts.f` to
/// *every* same-flavour opposite-sign pair and would bite the muons too; the
/// script records how that was verified.
///
/// The three numbers do not close:
///
/// ```text
/// m(ta+,ta-) in  [124.9, 125.1]   7.2077e-5  +- 2.94e-7 pb
/// m(ta+,ta-) outside              1.2965e-3  +- 3.43e-6 pb
///                       sum       1.36858e-3 +- 3.44e-6 pb
/// unwindowed, three seeds         1.3380e-3, 1.3421e-3, 1.3322e-3 (banked: 1.3373e-3)
/// ```
///
/// MadGraph's own partition of its own phase space exceeds its own unwindowed
/// integral by 3.1e-5 pb, 7.2σ on its own quoted errors, and the excess is the
/// pole. The unwindowed run is the one that is wrong, and its quoted 0.2% error
/// does not cover a 2.3% miss.
///
/// # This side
///
/// Measured with the single Breit–Wigner channel of the one diagram carrying the
/// Higgs propagator, drawn flat in its own uniforms: no VEGAS grid, no α mixture,
/// no unweighting — the layers the production number goes through and the ones a
/// mis-covered resonance would hide in. It agrees with MadGraph's windowed run
/// (and, at higher cost, so do the 25-channel combiner and flat RAMBO).
///
/// # What this cannot detect
///
/// Anything both sides get wrong the same way inside the window: the two
/// estimates share the matrix element, which the `amplitudes` cell gates at the
/// pole to 1e-11, and share the window definition. It is a statement about phase
/// space coverage, not about `|M|²`.
#[test]
fn the_higgs_pole_window_is_measured_against_madgraph() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/madgraph/higgs_window_reference.json");
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let reference: serde_json::Value = serde_json::from_str(&text).expect("windowed reference");
    let get = |section: &str, field: &str| -> f64 {
        reference[section][field]
            .as_f64()
            .unwrap_or_else(|| panic!("{section}.{field} in {}", path.display()))
    };
    let (lo, hi) = (get("window", "m_tautau_lo"), get("window", "m_tautau_hi"));
    let (mg, mg_err) = (get("hwindow", "sigma_pb"), get("hwindow", "sigma_err_pb"));

    let row = ROWS
        .iter()
        .find(|r| r.key == "ee_to_mumu_tata_qcd0")
        .expect("the decider row is in the table");

    let card_path = output_dir().join(row.key).join("Cards/run_card.dat");
    let run_card = RunCard::parse_file(&card_path).expect("real run card parses");
    let sqrt_s = run_card.ebeam1 + run_card.ebeam2;
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &param_card(row.key));
    let sets = common::generate(row.process);
    let evals = compile_subprocesses(&sets, &model, &evaluated).expect("compile subprocesses");
    let bounds: Vec<_> = evals
        .iter()
        .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
        .collect();
    let rep = &evals[0];
    let legs = process_external_legs(rep, &model, &evaluated);
    let cuts = Cuts::compile(&run_card, &legs).expect("run card cuts compile");
    let final_masses: Vec<f64> = rep.external_particles()[rep.n_in()..]
        .iter()
        .map(|&id| evaluated.mass(id))
        .collect();
    let spin_color_avg = initial_spin_color_average(rep, &model, &evaluated);
    let diagrams: Vec<_> = sets
        .iter()
        .flat_map(|s| s.diagrams.iter().cloned())
        .collect();

    // The diagram whose propagator chain carries the pole the window is drawn
    // around, found by asking the channel builder rather than by index.
    let pole = 0.5 * (lo + hi);
    let resonant: Vec<_> = diagrams
        .iter()
        .filter(|d| {
            DiagramChannel::<f64>::from_diagram(d, &evaluated, sqrt_s)
                .resonances()
                .iter()
                .any(|r| (r.mass - pole).abs() < 1.0)
        })
        .cloned()
        .collect();
    assert_eq!(
        resonant.len(),
        1,
        "[{}] expected exactly one diagram with a pole at {pole} GeV",
        row.key
    );

    let amps: Vec<&BoundAmplitude<f64>> = bounds.iter().collect();
    let mut integ = FixedBeamIntegrand::new(amps, &cuts, sqrt_s, final_masses, spin_color_avg);
    integ
        .use_running_coupling(&diagrams, &model, &evaluated, &run_card)
        .expect("run card scale prescription compiles");
    integ.use_multichannel(&resonant, &evaluated, 2_000, 1, SEED);
    assert_eq!(integ.channel_count(), 1, "one channel, so no α mixture");

    let mut rng = ChaCha8Rng::seed_from_u64(HWINDOW_SEED);
    let mut u = vec![0.0; integ.channel_grid_ndim()];
    let mut momenta = Vec::new();
    let (mut sum, mut sum_sq, mut inside) = (0.0, 0.0, 0usize);
    for _ in 0..HWINDOW_DRAWS {
        for x in u.iter_mut() {
            *x = rng.random::<f64>();
        }
        let v = integ.event_in_channel(0, &u, &mut momenta);
        let m = (momenta[2] + momenta[3]).m();
        let w = if m >= lo && m < hi { v } else { 0.0 };
        sum += w;
        sum_sq += w * w;
        if w != 0.0 {
            inside += 1;
        }
    }
    let n = HWINDOW_DRAWS as f64;
    let mean = sum / n;
    let ours = mean * GEV2_TO_PB;
    let ours_err = ((sum_sq / n - mean * mean).max(0.0) / n).sqrt() * GEV2_TO_PB;

    let rel = ours / mg - 1.0;
    let pull = (ours - mg) / (ours_err * ours_err + mg_err * mg_err).sqrt();
    eprintln!(
        "-- h -> tau tau pole window, m(ta+,ta-) in [{lo}, {hi}] --\n  \
         vibegraph (single Breit-Wigner channel, {inside}/{HWINDOW_DRAWS} draws in window) \
         {ours:.5e} +- {ours_err:.2e} pb\n  \
         MadGraph  (dummy_cuts window, own run card)                          \
         {mg:.5e} +- {mg_err:.2e} pb\n  \
         rel {:+.3}%, pull {pull:+.2}\n  \
         MadGraph's window + complement = {:.6e} pb against its unwindowed {:.6e} pb \
         and the banked {:.6e} pb",
        100.0 * rel,
        get("sum_in_plus_out", "sigma_pb"),
        reference["control"][0]["sigma_pb"]
            .as_f64()
            .expect("control"),
        banked_sigma(row.key),
    );
    assert!(
        rel.abs() < HWINDOW_TOLERANCE,
        "[{}] windowed sigma {ours:.5e} pb against MadGraph's own {mg:.5e} pb: \
         rel {:+.3}%, outside {:.0}%",
        row.key,
        100.0 * rel,
        100.0 * HWINDOW_TOLERANCE
    );
}

/// The banked run's own cross section, for context in the windowed comparison.
fn banked_sigma(key: &str) -> f64 {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/sigma_reference.json");
    let text = std::fs::read_to_string(&path).expect("banked sigma reference");
    let doc: serde_json::Value =
        serde_json::from_str(&text).expect("banked sigma reference parses");
    doc[key]["sigma_pb"].as_f64().expect("banked sigma")
}
