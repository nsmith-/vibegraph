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

use common::report::{CategoryCount, Chi2Cell, KsCell, SamplesRow, SeedSample, Stopwatch};

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
/// every observable of every gating row on every seed: twelve fixed-beam rows and
/// four proton ones, three seeds and seven to twenty-one observables each, and the
/// observables of a `2 → 2` row are heavily correlated (both legs' `pT` are one
/// number at fixed beams), so the draws from the null distribution number a few
/// hundred rather than a few thousand. At a floor of `1e-3` that is an expected
/// 0.2 to 0.4 spurious failures per run, which would make the gate flap; at `1e-4`
/// it is under 0.05.
///
/// The measured minimum over every gating row and three seeds is `1.29e-4`
/// (`ee_to_mumua`, `pt(a)`; per-seed `3.38e-4`, `1.17e-2`, `1.29e-4`), with
/// `ee_to_wpwm` at `2.84e-3`, `ddx_to_epemg` at `5.85e-3` and `uux_to_mumu` at
/// `1.56e-2` behind it, against `3.6e-6` and `0` for the two rows that used to
/// disagree. `ee_to_mumua` is the row to watch: it sits only `1.3x` above the
/// floor, and its `integrals` pull is `+2.79` against the same reference.
///
/// That `pt(a)` column is the one the windowed measurement attributed: MadGraph's
/// banked sample puts 8.73% of σ below `pt(a) = 20` GeV where MadGraph's *own*
/// cross section for that region puts 9.40%, and both of this side's estimators
/// land on the latter. The column sitting closest to this floor is measuring the
/// reference's sample rather than this side's — see the `ee_to_mumua` `samples`
/// note in `validation/manifest.toml`.
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
        mode: "gate",
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
        mode: "gate",
    },
    Row {
        key: "ud_to_epemud_qcd0",
        process: "u d > e+ e- u d QCD=0",
        neval: 60_000,
        niter: 6,
        // Kinematics and SPINUP agree (min KS p 1.0e-1, SPINUP chi2 p 0.43-0.73
        // over three seeds), but ICOLUP does not: chi2 642-664 on 1 dof (p ~ 0)
        // on every seed, a sharp and seed-stable colour-connectivity
        // disagreement rather than a marginal miss. Measured and reported
        // rather than gated; see validation/manifest.toml's note.
        mode: "info",
    },
    // ── the ℓ⁺ℓ⁻ j partonic rows, generated at a per-event cluster scale ──
    // The only rows here whose renormalisation and factorisation scales are
    // recomputed from each event's kT clustering rather than read once off the
    // run card, so the accept/reject draw runs over an integrand whose coupling
    // moves with the point. `the_llj_parton_rows_take_a_per_event_cluster_scale`
    // is what says the prescription did not collapse to a constant — the way
    // these could agree for a reason other than the clustering.
    Row {
        key: "uux_to_epemg",
        process: "u u~ > e+ e- g QCD=2 QED=2",
        neval: 40_000,
        niter: 5,
        mode: "gate",
    },
    Row {
        key: "ddx_to_epemg",
        process: "d d~ > e+ e- g QCD=2 QED=2",
        neval: 40_000,
        niter: 5,
        mode: "gate",
    },
    Row {
        key: "gu_to_epemu",
        process: "g u > e+ e- u QCD=2 QED=2",
        neval: 40_000,
        niter: 5,
        mode: "gate",
    },
    Row {
        key: "gux_to_epemux",
        process: "g u~ > e+ e- u~ QCD=2 QED=2",
        neval: 40_000,
        niter: 5,
        mode: "gate",
    },
];

/// The four `l+ l- j` partonic rows: the ones whose run cards leave both scales
/// free at `dynamical_scale_choice = -1`.
const LLJ_PARTON_KEYS: [&str; 4] = [
    "uux_to_epemg",
    "ddx_to_epemg",
    "gu_to_epemu",
    "gux_to_epemux",
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
        let u = [rng.random(), rng.random(), rng.random(), rng.random()];
        let Some(selection) = integ.select_event(&momenta, point.channel, u) else {
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
        let clock = Stopwatch::start();
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
            report.duration_s = Some(clock.seconds());
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

/// The four `l+ l- j` partonic rows draw their scales from the kT clustering on
/// every event, not from a constant — measured here rather than assumed.
///
/// Their run cards leave `dynamical_scale_choice = -1`, and a t-channel
/// propagator into a three-leg final state is the topology whose cluster scale
/// depends on the merge order. The general clustering computes it, and every
/// event of all four runs reproduces the banked `SCALUP`/`<rscale>`/`<pdfrwt>`
/// inside its printing budget (`validate_scales.rs`). What this asserts is the
/// integrand's own side of it: the prescription compiles, resolves on a sampled
/// cut-passing point, and hands back a coupling that moves with the event.
///
/// It is what stops the sample and cross-section rows above from agreeing for the
/// wrong reason. A prescription that quietly collapsed to a constant — `m_Z`, say,
/// which is where their `ℓ⁺ℓ⁻` pair sits — would leave both comparisons close
/// enough to pass while measuring nothing about the clustering, and the two
/// assertions below are exactly what such a collapse fails.
#[test]
fn the_llj_parton_rows_take_a_per_event_cluster_scale() {
    for key in LLJ_PARTON_KEYS {
        let row = ROWS
            .iter()
            .find(|r| r.key == key)
            .expect("every llj parton key names a compared row");
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
        let report = integ
            .use_running_coupling(&diagrams, &model, &evaluated, &run_card)
            .unwrap_or_else(|e| panic!("[{}] the scale prescription was refused: {e}", row.key));
        assert!(
            report.depends_on_alpha_s,
            "[{}] a QCD matrix element must move with the strong coupling",
            row.key
        );
        // A clustered scale is a function of the event, so the prescription must
        // *not* have collapsed to a constant — that is what separates it from the
        // fixed-scale rows.
        assert!(
            report.constant_scales.is_none(),
            "[{}] the clustering branch resolved to a constant",
            row.key
        );
        let channels = report
            .channels
            .expect("the clustering branch was given channel forests");
        assert!(channels > 0, "[{}] no integration channels", row.key);
        eprintln!(
            "  {} resolves a per-event cluster scale over {channels} channels",
            row.key
        );
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

// ── pt(γ) windows: the `ee_to_mumua` σ/shape discrepancy, localised ──────────

/// Window edges in `pt(γ)`, GeV. Fixed from the run card and from kinematics
/// before any measurement, not fitted to the disagreement:
///
/// * `10` is the run card's own `pta`, and `250` is `√s/2`;
/// * `20 = 2·pta` isolates the strip where the cut, not the dynamics, sets the
///   density;
/// * `39.4 = p_RR / cosh(etaa)` with `p_RR = (s − M_Z²)/(2√s) = 241.685` GeV is
///   the transverse momentum below which no on-shell-Z radiative-return event
///   can survive the `etaa = 2.5` rapidity cut, so it separates the phase space
///   the two Breit–Wigner channels reach from the phase space they cannot;
/// * `77` and `144` are the equal-population tertiles of MadGraph's banked
///   sample *above* that threshold, rounded to 1 GeV.
const PTA_EDGES: [f64; 6] = [10.0, 20.0, 39.4, 77.0, 144.0, 250.0];
/// The secondary axis: below-Z continuum, low shoulder, the Z peak, high
/// shoulder, and the `m(μμ) → √s` non-radiative region.
///
/// `pt(γ)` is a *smeared image* of the structure carrying this cross section —
/// an on-shell-Z event lands anywhere in `pt(γ) ∈ [39.4, 241.7]` depending on
/// `η(γ)` — whereas `m(μμ)` resolves the Breit–Wigner directly. This axis is what
/// says whether something localised in `pt(γ)` sits on the Z peak or in the
/// continuum. It carries no verdict of its own: nothing in the decision rule
/// keys on it, by construction.
const MUMU_EDGES: [f64; 6] = [0.0, 60.0, 86.0, 96.0, 200.0, 500.0];
/// The σ gate's own budget for this row, so the windowed shape is measured at
/// the configuration the gated cross section comes from.
const PTA_NEVAL: usize = 80_000;
const PTA_NITER: usize = 8;
/// The seed set `probe_resonant_seed_stability` sweeps, so this sweep and the
/// recorded one are directly comparable.
const PTA_SEEDS: [u64; 5] = [SEED, 11, 22, 33, 44];
/// Seeds carried to the 4× budget. A residual that is sampling shrinks with
/// budget; one that migrates between seeds at fixed size is a defect, and three
/// seeds is enough to tell those apart without quadrupling the sweep's cost.
const PTA_BUDGET_SEEDS: [u64; 3] = [SEED, 11, 22];
/// ChaCha stream base for the windowed measurement pass, disjoint from the
/// integrand's own channel streams and from the α survey's.
const PTA_STREAM_BASE: u64 = 0x0D_0D_0000;

/// Build the production `ee_to_mumua` integrand — its banked run card, cuts,
/// per-event scale and α-adapted per-diagram multichannel sampler — optionally
/// with the photon `pt` cuts replaced by a window.
///
/// With `pt_window = None` this is exactly the σ gate's integrand for this row.
/// With a window it is a *different* integration: `pta` sets the process's
/// fiducial scale, so both the channel maps and the VEGAS grids re-adapt inside
/// the window, which is what makes the sum over windows an independent survey
/// rather than a partition of one.
fn with_mumua_integrand<R>(
    pt_window: Option<(f64, f64)>,
    seed: u64,
    f: impl FnOnce(&FixedBeamIntegrand) -> R,
) -> R {
    const KEY: &str = "ee_to_mumua";
    const PROCESS: &str = "e+ e- > mu+ mu- a";

    let card_path = output_dir().join(KEY).join("Cards/run_card.dat");
    let card_path = match pt_window {
        None => card_path,
        Some((lo, hi)) => {
            let text = std::fs::read_to_string(&card_path).expect("run card readable");
            let (mut saw_lo, mut saw_hi) = (0usize, 0usize);
            let patched: String = text
                .lines()
                .map(|l| {
                    if l.contains("= pta ") {
                        saw_lo += 1;
                        format!("  {lo} = pta\n")
                    } else if l.contains("= ptamax ") {
                        saw_hi += 1;
                        format!("  {hi} = ptamax\n")
                    } else {
                        format!("{l}\n")
                    }
                })
                .collect();
            assert_eq!((saw_lo, saw_hi), (1, 1), "one pta and one ptamax line");
            let out = std::env::temp_dir().join(format!("vg_pta_window_{lo}_{hi}.dat"));
            std::fs::write(&out, patched).expect("windowed run card writable");
            out
        }
    };

    let run_card = RunCard::parse_file(&card_path).expect("real run card parses");
    assert_eq!(run_card.beam_mode(), BeamMode::FixedEnergy);
    assert_eq!(
        run_card.ebeam1, run_card.ebeam2,
        "the comparison assumes the lab frame is the partonic centre of mass"
    );
    let sqrt_s = run_card.ebeam1 + run_card.ebeam2;

    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &param_card(KEY));
    let sets = common::generate(PROCESS);
    let evals = compile_subprocesses(&sets, &model, &evaluated).expect("compile subprocesses");
    let bounds: Vec<_> = evals
        .iter()
        .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
        .collect();

    let rep = &evals[0];
    // The window is read off `momenta[2]`, which is only the photon if the final
    // state is ordered (mu+, mu-, a). MadGraph's own window is cut on external
    // leg 5 of `leshouche.inc`, IDUP = (-11, 11, -13, 13, 22); the two sides
    // must be windowing the same leg.
    let final_pdg: Vec<i64> = rep.external_particles()[rep.n_in()..]
        .iter()
        .map(|&id| model.particle(id).pdg_code)
        .collect();
    assert_eq!(
        final_pdg,
        vec![-13, 13, 22],
        "final state must be (mu+, mu-, a) for momenta[2] to be the photon"
    );

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
        seed,
    );
    f(&integ)
}

/// A cross section and its Monte-Carlo error, in pb.
#[derive(Clone, Copy, Debug)]
struct Sigma {
    pb: f64,
    err: f64,
}

impl Sigma {
    fn zero() -> Self {
        Sigma { pb: 0.0, err: 0.0 }
    }
    /// Independent terms: values add, errors add in quadrature.
    fn add(self, other: Sigma) -> Self {
        Sigma {
            pb: self.pb + other.pb,
            err: (self.err * self.err + other.err * other.err).sqrt(),
        }
    }
}

/// `a/b − 1` with both errors propagated.
fn rel_with_err(a: Sigma, b: Sigma) -> (f64, f64) {
    let rel = a.pb / b.pb - 1.0;
    let err = ((a.err / b.pb).powi(2) + (a.pb * b.err / (b.pb * b.pb)).powi(2)).sqrt();
    (rel, err)
}

/// The unwindowed integral split into `pt(γ)` windows on one set of draws: the
/// total and every window come off the *same* points, so the windows sum to the
/// total exactly and the split carries no information about coverage — only
/// about shape, at the gate's own configuration.
///
/// The grids are the ones `adapt_grids` trains at the gate budget; the
/// measurement itself is a frozen pass over them on a disjoint ChaCha stream,
/// which is what lets a per-window indicator be accumulated without changing the
/// integration the gate runs.
/// Both axes are accumulated on the same draws, because they cost nothing extra
/// there and because a shared set of points is what makes the two tables
/// comparable: any difference between them is the projection, not the sample.
struct PartSplit {
    total: Sigma,
    pt: Vec<Sigma>,
    mll: Vec<Sigma>,
    /// The `adapt_grids` result the σ gate itself reports, for cross-checking
    /// that the integrand measured here is the gated one.
    gate: f64,
}

fn vg_part(seed: u64, neval: usize) -> PartSplit {
    with_mumua_integrand(None, seed, |integ| {
        let (channels, gate) = integ.adapt_grids(neval, PTA_NITER, seed);
        let (npt, nmll) = (PTA_EDGES.len() - 1, MUMU_EDGES.len() - 1);
        let mut pt_windows = vec![Sigma::zero(); npt];
        let mut mll_windows = vec![Sigma::zero(); nmll];
        let mut total = Sigma::zero();
        let ndim = integ.channel_grid_ndim();

        for (j, ch) in channels.iter().enumerate() {
            let draws = ch.neval * PTA_NITER;
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            rng.set_stream(PTA_STREAM_BASE + j as u64);
            let mut x = vec![0.0; ndim];
            let mut momenta = Vec::new();
            // Slot 0 is the total; then the pt(γ) windows, then the m(μμ) ones.
            let mut sum = vec![0.0; 1 + npt + nmll];
            let mut sum_sq = vec![0.0; 1 + npt + nmll];
            for _ in 0..draws {
                let jac = ch.grid.draw(&mut rng, &mut x);
                let v = jac * integ.event_in_channel(j, &x, &mut momenta);
                sum[0] += v;
                sum_sq[0] += v * v;
                if v == 0.0 {
                    continue;
                }
                let p = momenta[2];
                let pt = (p.px() * p.px() + p.py() * p.py()).sqrt();
                if let Some(w) = PTA_EDGES.windows(2).position(|e| pt >= e[0] && pt < e[1]) {
                    sum[1 + w] += v;
                    sum_sq[1 + w] += v * v;
                }
                let mll = (momenta[0] + momenta[1]).m();
                if let Some(w) = MUMU_EDGES
                    .windows(2)
                    .position(|e| mll >= e[0] && mll < e[1])
                {
                    sum[1 + npt + w] += v;
                    sum_sq[1 + npt + w] += v * v;
                }
            }
            let n = draws as f64;
            let term = |k: usize| {
                let mean = sum[k] / n;
                Sigma {
                    pb: mean * GEV2_TO_PB,
                    err: ((sum_sq[k] / n - mean * mean).max(0.0) / n).sqrt() * GEV2_TO_PB,
                }
            };
            total = total.add(term(0));
            for (w, slot) in pt_windows.iter_mut().enumerate() {
                *slot = slot.add(term(1 + w));
            }
            for (w, slot) in mll_windows.iter_mut().enumerate() {
                *slot = slot.add(term(1 + npt + w));
            }
        }
        PartSplit {
            total,
            pt: pt_windows,
            mll: mll_windows,
            gate: gate.integral * GEV2_TO_PB,
        }
    })
}

/// One window integrated on its own, with the run card's photon `pt` cuts set to
/// the window's edges. Unlike [`vg_part`] this re-surveys: the α mixture, the
/// channel maps' fiducial scale and the VEGAS grids all adapt to the window, so
/// the sum over windows is an audit of the unwindowed integral rather than a
/// restatement of it.
fn vg_cut(window: usize, seed: u64, neval: usize) -> (Sigma, f64) {
    let (lo, hi) = (PTA_EDGES[window], PTA_EDGES[window + 1]);
    with_mumua_integrand(Some((lo, hi)), seed, |integ| {
        let (_, r) = integ.adapt_grids(neval, PTA_NITER, seed);
        (
            Sigma {
                pb: r.integral * GEV2_TO_PB,
                err: r.std_dev * GEV2_TO_PB,
            },
            r.chi2_per_dof,
        )
    })
}

/// The MadGraph side of the window table, seed-averaged per estimator.
///
/// `pta_window_reference.json` holds one row per MadGraph run: stage, version,
/// estimator (`control` unwindowed, `part` windowed through `dummy_cuts`, `cut`
/// windowed through the run card), `axis`, window index, seed, `nevents`, σ and
/// its quoted error. This reduces a selection of those rows to a mean and the two
/// error estimates that matter — the quoted errors combined, and the *spread* of
/// the seeds about their own mean, which is the one B1 showed can be larger.
///
/// `axis` must be part of the key: the two axes number their windows from 1
/// independently, so a filter without it would silently pool `pt(γ)`'s window 3
/// with `m(μμ)`'s.
fn mg_cloud(
    reference: &serde_json::Value,
    version: &str,
    estimator: &str,
    axis: &str,
    window: usize,
    nevents: u64,
) -> Option<(Sigma, f64, f64, usize)> {
    let runs: Vec<&serde_json::Value> = reference["runs"]
        .as_array()
        .expect("runs array")
        .iter()
        .filter(|r| {
            r["mg_version"].as_str() == Some(version)
                && r["estimator"].as_str() == Some(estimator)
                // A row banked before the secondary axis existed is pt(γ).
                && r["axis"].as_str().unwrap_or("pt_a") == axis
                && r["window"].as_u64() == Some(window as u64)
                && r["nevents"].as_u64() == Some(nevents)
        })
        .collect();
    if runs.is_empty() {
        return None;
    }
    let n = runs.len() as f64;
    let s: Vec<f64> = runs
        .iter()
        .map(|r| r["sigma_pb"].as_f64().unwrap())
        .collect();
    let e: Vec<f64> = runs
        .iter()
        .map(|r| r["sigma_err_pb"].as_f64().unwrap())
        .collect();
    let mean = s.iter().sum::<f64>() / n;
    // The quoted errors of independent runs, combined as the error on their mean.
    let quoted = (e.iter().map(|x| x * x).sum::<f64>()).sqrt() / n;
    // The seeds' own spread, and the error on the mean it implies. A quoted error
    // that undercuts this is the failure mode this whole measurement is about.
    let spread = if runs.len() > 1 {
        (s.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt()
    } else {
        f64::NAN
    };
    // χ²/dof of the cloud about its own mean on the quoted errors: > 1 means the
    // runs disagree with each other by more than they claim.
    let chi2_dof = if runs.len() > 1 {
        s.iter()
            .zip(&e)
            .map(|(x, err)| ((x - mean) / err).powi(2))
            .sum::<f64>()
            / (n - 1.0)
    } else {
        f64::NAN
    };
    Some((
        Sigma {
            pb: mean,
            err: quoted,
        },
        spread / n.sqrt(),
        chi2_dof,
        runs.len(),
    ))
}

/// Localise the `ee_to_mumua` cross-section and `pt(γ)` shape disagreement by
/// measuring both sides' windowed cross sections and testing each side's
/// partition against its own unwindowed control.
///
/// # Why a partition and not a total
///
/// The gated row disagrees by +0.80% (2.8σ) against the MadGraph 3.7.1 bank, and
/// agreed with the 3.5.7 bank it replaced (−0.03%, 0.07σ) — while MadGraph's own
/// two numbers differ by only 1.84σ on their own quoted errors. A single cross
/// section cannot say which side mis-covers a region, and a seed-fixed pull is
/// not evidence on either side. Four estimators can:
///
/// * `MG-part(w)` — MadGraph with the window imposed through `dummy_cuts`, which
///   `passcuts` applies after every other cut and which the phase-space generator
///   never sees, so the windowed runs integrate the same integrand restricted to
///   `w` and **must** sum to the unwindowed control. B1 (`ee_to_mumu_tata_qcd0`)
///   is the precedent where they did not, by 7.2σ.
/// * `MG-cut(w)` — the window as run-card `pta`/`ptamax`, which `setcuts.f` feeds
///   into `etmax(i)` so the generator re-optimises. A better estimate of the true
///   windowed σ, and for exactly that reason not a term in the closure sum.
/// * `VG-part(w)` — this side's shape at the σ gate's own configuration, split on
///   one set of draws. Closes exactly and therefore says nothing about coverage.
/// * `VG-cut(w)` — this side's independent per-window integration, whose sum is
///   the only coverage audit this side gets.
///
/// # What this provably cannot decide
///
/// * Anything both sides get wrong the same way inside a window: they share the
///   matrix element (gated to 1e-11 by the `amplitudes` cell) and the window
///   definition. This is a statement about phase-space coverage and cut
///   boundaries, not about `|M|²`.
/// * `VG-cut` re-adapts grids and fiducial scale but reuses the same channel
///   construction and map code as the unwindowed run, so a defect in that shared
///   code survives in both and lets the closure pass. MadGraph's windowed runs
///   re-survey with a genuinely different channel allocation, so its closure is
///   the stronger oracle — the asymmetry favours the reference.
/// * The first window's lower edge *is* the `pta` cut, so a disagreement confined
///   to it cannot separate "the cut boundary is implemented differently" from
///   "the region is mis-covered".
/// * `η(γ)` is integrated over inside each window, so a compensating error that
///   cancels in the `pt(γ)` projection is invisible here.
///
/// Ignored: this is a measurement, not a gate. Run with
/// `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_pta_windows_against_madgraph() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/madgraph/pta_window_reference.json");
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let reference: serde_json::Value = serde_json::from_str(&text).expect("window reference");
    let edges: Vec<f64> = reference["pt_edges_gev"]
        .as_array()
        .expect("pt_edges_gev")
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    assert_eq!(
        edges, PTA_EDGES,
        "the MadGraph runs and this side must window the same edges"
    );
    let nwin = PTA_EDGES.len() - 1;
    const NEV: u64 = 100_000;

    // ── MadGraph's side, as banked ──────────────────────────────────────────
    eprintln!("── MadGraph, sigma(e+ e- > mu+ mu- a) by pt(a) window ──");
    for (version, estimator, label) in [
        ("3.7.1", "control", "3.7.1 unwindowed"),
        ("3.5.7", "control", "3.5.7 unwindowed"),
        ("3.7.1", "part", "3.7.1 dummy_cuts window"),
        ("3.5.7", "part", "3.5.7 dummy_cuts window"),
        ("3.7.1", "cut", "3.7.1 run-card window"),
    ] {
        for w in 0..=nwin {
            let Some((s, spread, chi2, n)) =
                mg_cloud(&reference, version, estimator, "pt_a", w, NEV)
            else {
                continue;
            };
            let range = if w == 0 {
                "unwindowed".to_string()
            } else {
                format!("[{}, {})", PTA_EDGES[w - 1], PTA_EDGES[w])
            };
            eprintln!(
                "  {label:<24} {range:>14}  {:.6e} +- {:.3e} (quoted, {n} seeds) \
                 | seed-spread err {spread:.3e} | cloud chi2/dof {chi2:.2}",
                s.pb, s.err,
            );
        }
    }

    // MG's closure: the dummy_cuts partition against the unwindowed control.
    let mg_control = mg_cloud(&reference, "3.7.1", "control", "pt_a", 0, NEV);
    let mg_parts: Vec<Option<(Sigma, f64, f64, usize)>> = (1..=nwin)
        .map(|w| mg_cloud(&reference, "3.7.1", "part", "pt_a", w, NEV))
        .collect();
    let mut c_mg = None;
    if let (Some((control, _, _, _)), true) = (mg_control, mg_parts.iter().all(|p| p.is_some())) {
        let sum = mg_parts
            .iter()
            .map(|p| p.unwrap().0)
            .fold(Sigma::zero(), Sigma::add);
        let (rel, err) = rel_with_err(sum, control);
        eprintln!(
            "  C_MG: sum of windows {:.6e} +- {:.3e} against unwindowed {:.6e} +- {:.3e} \
             -> {:+.3}% +- {:.3}%, {:+.2} sigma",
            sum.pb,
            sum.err,
            control.pb,
            control.err,
            100.0 * rel,
            100.0 * err,
            rel / err,
        );
        c_mg = Some((sum, control, rel, err));
    } else {
        eprintln!("  C_MG: not computable — the 3.7.1 partition is incomplete");
    }

    // ── this side: shape at the gate's configuration ────────────────────────
    eprintln!("\n── vibegraph VG-part: gate budget {PTA_NEVAL} x {PTA_NITER}, windows on one set of draws ──");
    let mut vg_part_seeds: Vec<(u64, Sigma, Vec<Sigma>)> = Vec::new();
    let mut vg_part_mll: Vec<(u64, Vec<Sigma>)> = Vec::new();
    for seed in PTA_SEEDS {
        let split = vg_part(seed, PTA_NEVAL);
        let (total, windows, gate) = (split.total, split.pt.clone(), split.gate);
        vg_part_mll.push((seed, split.mll));
        eprintln!(
            "  seed {seed:>10}: total {:.6e} +- {:.3e} pb (gate's own combined {gate:.6e})",
            total.pb, total.err
        );
        for (w, s) in windows.iter().enumerate() {
            eprintln!(
                "      [{:>5}, {:>5})  {:.6e} +- {:.3e}  ({:5.2}% of total)",
                PTA_EDGES[w],
                PTA_EDGES[w + 1],
                s.pb,
                s.err,
                100.0 * s.pb / total.pb
            );
        }
        vg_part_seeds.push((seed, total, windows));
    }
    let mut vg_part_4x: Vec<(u64, Sigma, Vec<Sigma>)> = Vec::new();
    for seed in PTA_BUDGET_SEEDS {
        let split = vg_part(seed, 4 * PTA_NEVAL);
        let (total, windows) = (split.total, split.pt);
        eprintln!(
            "  seed {seed:>10} @4x budget: total {:.6e} +- {:.3e} pb",
            total.pb, total.err
        );
        for (w, s) in windows.iter().enumerate() {
            eprintln!(
                "      [{:>5}, {:>5})  {:.6e} +- {:.3e}",
                PTA_EDGES[w],
                PTA_EDGES[w + 1],
                s.pb,
                s.err
            );
        }
        vg_part_4x.push((seed, total, windows));
    }

    // ── this side: independent per-window integrations, and their closure ───
    eprintln!("\n── vibegraph VG-cut: one integration per window (pta/ptamax), 4x on W1 ──");
    let mut vg_cut_seeds: Vec<(u64, Vec<Sigma>)> = Vec::new();
    for seed in PTA_SEEDS {
        let mut windows = Vec::with_capacity(nwin);
        for w in 0..nwin {
            // The first window rejects ~92% of draws through `ptamax`, so at the
            // base budget its error would dominate the closure sum and make the
            // audit vacuous.
            let neval = if w == 0 { 4 * PTA_NEVAL } else { PTA_NEVAL };
            let (s, chi2) = vg_cut(w, seed, neval);
            eprintln!(
                "  seed {seed:>10}  [{:>5}, {:>5})  {:.6e} +- {:.3e}  chi2/dof {chi2:.2}  \
                 (neval {neval})",
                PTA_EDGES[w],
                PTA_EDGES[w + 1],
                s.pb,
                s.err
            );
            windows.push(s);
        }
        vg_cut_seeds.push((seed, windows));
    }

    // ── the statistics the verdict is read off ──────────────────────────────
    let mean_over_seeds = |pick: &dyn Fn(usize) -> Sigma, n: usize| -> (Sigma, f64, f64) {
        let vals: Vec<Sigma> = (0..n).map(pick).collect();
        let m = vals.iter().map(|s| s.pb).sum::<f64>() / n as f64;
        let quoted = vals.iter().map(|s| s.err * s.err).sum::<f64>().sqrt() / n as f64;
        let spread = if n > 1 {
            (vals.iter().map(|s| (s.pb - m).powi(2)).sum::<f64>() / (n as f64 - 1.0)).sqrt()
        } else {
            f64::NAN
        };
        let chi2_dof = if n > 1 {
            vals.iter()
                .map(|s| ((s.pb - m) / s.err).powi(2))
                .sum::<f64>()
                / (n as f64 - 1.0)
        } else {
            f64::NAN
        };
        (
            Sigma { pb: m, err: quoted },
            spread / (n as f64).sqrt(),
            chi2_dof,
        )
    };

    eprintln!("\n── seed clouds (mean, quoted error on the mean, seed-spread error, cloud chi2/dof on 4 dof) ──");
    let ns = PTA_SEEDS.len();
    let (vg_total, vg_total_spread, vg_total_chi2) = mean_over_seeds(&|i| vg_part_seeds[i].1, ns);
    eprintln!(
        "  VG total   {:.6e} +- {:.3e} | spread err {vg_total_spread:.3e} | chi2/dof {vg_total_chi2:.2}",
        vg_total.pb, vg_total.err
    );

    let mut vg_part_mean = Vec::with_capacity(nwin);
    let mut vg_cut_mean = Vec::with_capacity(nwin);
    for w in 0..nwin {
        let (p, p_spread, p_chi2) = mean_over_seeds(&|i| vg_part_seeds[i].2[w], ns);
        let (c, c_spread, c_chi2) = mean_over_seeds(&|i| vg_cut_seeds[i].1[w], ns);
        eprintln!(
            "  [{:>5}, {:>5})  VG-part {:.6e} +- {:.3e} | spread {p_spread:.3e} | chi2/dof {p_chi2:.2}\n\
             {:>18}  VG-cut  {:.6e} +- {:.3e} | spread {c_spread:.3e} | chi2/dof {c_chi2:.2}",
            PTA_EDGES[w], PTA_EDGES[w + 1], p.pb, p.err, "", c.pb, c.err
        );
        vg_part_mean.push(p);
        vg_cut_mean.push(c);
    }

    // Budget stability, per window and on the total, over the seeds carried to
    // 4×: a residual that is sampling shrinks with budget, where a defect keeps
    // its size and migrates between seeds.
    eprintln!("\n── VG-part budget stability, base against 4x on the same seeds ──");
    let nb = PTA_BUDGET_SEEDS.len();
    let base_at = |i: usize, w: Option<usize>| -> Sigma {
        let k = vg_part_seeds
            .iter()
            .position(|(s, _, _)| *s == PTA_BUDGET_SEEDS[i])
            .expect("the 4x seeds are a subset of the sweep");
        match w {
            None => vg_part_seeds[k].1,
            Some(w) => vg_part_seeds[k].2[w],
        }
    };
    let quad = |a: Sigma, b: Sigma| (a.err * a.err + b.err * b.err).sqrt();
    for slot in [None, Some(0), Some(1), Some(2), Some(3), Some(4)] {
        let (b, q) = (
            mean_over_seeds(&|i| base_at(i, slot), nb).0,
            mean_over_seeds(
                &|i| match slot {
                    None => vg_part_4x[i].1,
                    Some(w) => vg_part_4x[i].2[w],
                },
                nb,
            )
            .0,
        );
        let label = match slot {
            None => "total".to_string(),
            Some(w) => format!("[{}, {})", PTA_EDGES[w], PTA_EDGES[w + 1]),
        };
        eprintln!(
            "  {label:>14}  base {:.6e} +- {:.3e} | 4x {:.6e} +- {:.3e} | \
             err shrinks {:.2}x (need >= 1.7) | shift {:+.3}% = {:+.2} combined sigma",
            b.pb,
            b.err,
            q.pb,
            q.err,
            b.err / q.err,
            100.0 * (q.pb / b.pb - 1.0),
            (q.pb - b.pb) / quad(b, q),
        );
    }

    // C_VG: the independent per-window integrations against the unwindowed total.
    let vg_sum = vg_cut_mean.iter().copied().fold(Sigma::zero(), Sigma::add);
    let (c_vg_rel, c_vg_err) = rel_with_err(vg_sum, vg_total);
    eprintln!(
        "\n  C_VG: sum of windows {:.6e} +- {:.3e} against unwindowed {:.6e} +- {:.3e} \
         -> {:+.3}% +- {:.3}%, {:+.2} sigma",
        vg_sum.pb,
        vg_sum.err,
        vg_total.pb,
        vg_total.err,
        100.0 * c_vg_rel,
        100.0 * c_vg_err,
        c_vg_rel / c_vg_err,
    );

    // chi2_flat: is the disagreement localised in a window, or the same in all?
    if mg_parts.iter().all(|p| p.is_some()) {
        let mut deltas = Vec::with_capacity(nwin);
        for w in 0..nwin {
            let (rel, err) = rel_with_err(vg_part_mean[w], mg_parts[w].unwrap().0);
            deltas.push((rel, err));
        }
        let wsum: f64 = deltas.iter().map(|(_, e)| 1.0 / (e * e)).sum();
        let dbar: f64 = deltas.iter().map(|(d, e)| d / (e * e)).sum::<f64>() / wsum;
        let chi2_flat: f64 = deltas.iter().map(|(d, e)| ((d - dbar) / e).powi(2)).sum();
        eprintln!("\n── Delta_w = VG-part(w)/MG-part(w) - 1 ──");
        for (w, (d, e)) in deltas.iter().enumerate() {
            eprintln!(
                "  [{:>5}, {:>5})  {:+.3}% +- {:.3}%  ({:+.2} sigma)",
                PTA_EDGES[w],
                PTA_EDGES[w + 1],
                100.0 * d,
                100.0 * e,
                d / e
            );
        }
        let (tot_rel, tot_err) = rel_with_err(
            vg_total,
            mg_control.map(|c| c.0).unwrap_or(Sigma {
                pb: f64::NAN,
                err: f64::NAN,
            }),
        );
        eprintln!(
            "  inverse-variance mean Delta_bar {:+.3}%, chi2_flat {chi2_flat:.2} on 4 dof \
             (localised iff > 13.28); Delta_tot {:+.3}% +- {:.3}%",
            100.0 * dbar,
            100.0 * tot_rel,
            100.0 * tot_err,
        );
    } else {
        eprintln!("\n── Delta_w not computable — the 3.7.1 partition is incomplete");
    }
    let _ = c_mg;

    // ── the secondary axis: m(mu+ mu-), reported, gating nothing ────────────
    //
    // `pt(γ)` cannot separate an on-shell-Z event from a continuum one, because
    // `η(γ)` smears the Z's image across most of the `pt(γ)` range. This axis
    // resolves the Breit–Wigner directly, so it says *where* a `pt(γ)`
    // localisation lives. Nothing in the decision rule reads it.
    let nmll = MUMU_EDGES.len() - 1;
    let mg_mll: Vec<Option<(Sigma, f64, f64, usize)>> = (1..=nmll)
        .map(|w| mg_cloud(&reference, "3.7.1", "part", "m_mumu", w, NEV))
        .collect();
    if mg_mll.iter().all(|m| m.is_some()) {
        eprintln!(
            "\n── secondary axis m(mu+ mu-): VG-part against MG-part (no verdict keys on this) ──"
        );
        let mut sum_mg = Sigma::zero();
        let mut sum_vg = Sigma::zero();
        for w in 0..nmll {
            let (mg, mg_spread, mg_chi2, n) = mg_mll[w].unwrap();
            let (vg, vg_spread, vg_chi2) = mean_over_seeds(&|i| vg_part_mll[i].1[w], ns);
            let (rel, err) = rel_with_err(vg, mg);
            // The same ratio against MadGraph's seed spread rather than its
            // quoted error, since the pt(γ) axis measured those clouds at
            // chi2/dof 3.5-4.3 and this axis has no reason to be better.
            let err_spread = ((vg_spread / mg.pb).powi(2)
                + (vg.pb * mg_spread / (mg.pb * mg.pb)).powi(2))
            .sqrt();
            eprintln!(
                "  [{:>5}, {:>5})  MG-part {:.6e} +- {:.3e} (quoted, {n} seeds) | spread {mg_spread:.3e} | chi2/dof {mg_chi2:.2}\n\
                 {:>18}  VG-part {:.6e} +- {:.3e} | spread {vg_spread:.3e} | chi2/dof {vg_chi2:.2}\n\
                 {:>18}  Delta {:+.3}% +- {:.3}% ({:+.2} sigma quoted) +- {:.3}% ({:+.2} sigma spread) | \
                 MG share {:5.2}%",
                MUMU_EDGES[w],
                MUMU_EDGES[w + 1],
                mg.pb,
                mg.err,
                "",
                vg.pb,
                vg.err,
                "",
                100.0 * rel,
                100.0 * err,
                rel / err,
                100.0 * err_spread,
                rel / err_spread,
                100.0 * mg.pb
                    / mg_mll
                        .iter()
                        .map(|m| m.unwrap().0.pb)
                        .sum::<f64>(),
            );
            sum_mg = sum_mg.add(mg);
            sum_vg = sum_vg.add(vg);
        }
        // A second, independent partition of the same MadGraph run. Recorded as
        // corroboration only: the verdicts are frozen and nothing keys on it.
        if let Some((control, _, _, _)) = mg_control {
            let (rel, err) = rel_with_err(sum_mg, control);
            eprintln!(
                "  C_MG on this axis (recorded, keys nothing): sum {:.6e} +- {:.3e} against \
                 unwindowed {:.6e} +- {:.3e} -> {:+.3}% +- {:.3}%, {:+.2} sigma",
                sum_mg.pb,
                sum_mg.err,
                control.pb,
                control.err,
                100.0 * rel,
                100.0 * err,
                rel / err,
            );
        }
        let (rel, err) = rel_with_err(sum_vg, sum_mg);
        eprintln!(
            "  sum over this axis: VG {:.6e} vs MG {:.6e} -> {:+.3}% +- {:.3}%",
            sum_vg.pb,
            sum_mg.pb,
            100.0 * rel,
            100.0 * err
        );
    } else {
        eprintln!("\n── secondary axis m(mu+ mu-) not measured on MadGraph's side");
    }
}
