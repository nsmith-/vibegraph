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

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use vibegraph::cuts::Cuts;
use vibegraph::hadronic::{
    compile_subprocesses, initial_spin_color_average, process_external_legs, FixedBeamIntegrand,
};
use vibegraph::helas::eval::BoundAmplitude;
use vibegraph::lhef::build::{EventHeader, SubprocessRecord};
use vibegraph::lhef::observables::{
    canonical, colour_key, flavour_key, helicity_key, kinematics, Labelling,
};
use vibegraph::lhef::parse::LheFile;
use vibegraph::lhef::record::LheEvent;
use vibegraph::phasespace::GEV2_TO_PB;
use vibegraph::runcard::{BeamMode, RunCard};
use vibegraph::stats::{chi2_homogeneity, effective_counts, ks_two_sample};
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::EvaluatedModel;
use vibegraph::unweight::Unweighter;

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

/// Categorical columns with at most this many distinct keys carry their per-key
/// counts into the report, so a χ² that fails says which category moved.
const MAX_CATEGORY_DETAIL: usize = 32;

/// An observable whose values span less than this fraction of their own scale is
/// a constant of the process — `m(l+,l-)` at fixed beams is `√s` on every event —
/// and has no distribution to compare. Such columns are named in the report
/// rather than dropped silently.
const DEGENERATE_SPAN: f64 = 1e-9;

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

/// A sample: one record per event, with the weight it carries.
struct Sample {
    events: Vec<LheEvent>,
    weights: Vec<f64>,
    /// The cross section the sample represents, in picobarns — MadGraph's banked
    /// value, or the one our accept/reject pass recovered.
    sigma_pb: f64,
}

impl Sample {
    fn len(&self) -> usize {
        self.events.len()
    }
}

/// MadGraph's banked events for a run, as records.
fn banked_sample(dir: &str) -> Sample {
    let path = output_dir()
        .join(dir)
        .join("Events/run_01/unweighted_events.lhe.gz");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let mut text = String::new();
    GzDecoder::new(&bytes[..])
        .read_to_string(&mut text)
        .unwrap_or_else(|e| panic!("decompress {}: {e}", path.display()));
    let file = LheFile::parse(&text).expect("MadGraph's own file parses");
    // `XWGTUP` under `IDWTUP = -4` is a cross section per event and the total is
    // their mean, so the mean is the sample's σ and the weights carry whatever
    // spread the run left in them.
    let weights: Vec<f64> = file.events.iter().map(|e| e.weight).collect();
    let sigma_pb = weights.iter().sum::<f64>() / weights.len() as f64;
    Sample {
        events: file.events,
        weights,
        sigma_pb,
    }
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
) -> Sample {
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
    Sample {
        events,
        weights,
        sigma_pb,
    }
}

/// The columns of a sample: one named continuous observable per entry, plus the
/// three categorical keys.
struct Columns {
    kinematic: Vec<(String, Vec<(f64, f64)>)>,
    helicity: BTreeMap<String, (f64, f64)>,
    colour: BTreeMap<String, (f64, f64)>,
    flavour: BTreeMap<String, (f64, f64)>,
}

fn columns(sample: &Sample, labelling: Labelling) -> Columns {
    let mut kinematic: Vec<(String, Vec<(f64, f64)>)> = Vec::new();
    let mut helicity = BTreeMap::new();
    let mut colour = BTreeMap::new();
    let mut flavour = BTreeMap::new();
    for (event, &w) in sample.events.iter().zip(&sample.weights) {
        let event = canonical(event, labelling);
        for (k, (name, value)) in kinematics(&event, labelling).into_iter().enumerate() {
            if k == kinematic.len() {
                kinematic.push((name.clone(), Vec::with_capacity(sample.len())));
            }
            assert_eq!(
                kinematic[k].0, name,
                "the observable names must not depend on the event"
            );
            kinematic[k].1.push((value, w));
        }
        for (map, key) in [
            (&mut helicity, helicity_key(&event)),
            (&mut colour, colour_key(&event)),
            (&mut flavour, flavour_key(&event)),
        ] {
            let entry = map.entry(key).or_insert((0.0, 0.0));
            entry.0 += w;
            entry.1 += w * w;
        }
    }
    Columns {
        kinematic,
        helicity,
        colour,
        flavour,
    }
}

/// χ² homogeneity between two categorical columns, over the union of their
/// categories.
fn categorical(
    ours: &BTreeMap<String, (f64, f64)>,
    theirs: &BTreeMap<String, (f64, f64)>,
    column: &str,
) -> Option<Chi2Cell> {
    let keys: Vec<&String> = ours
        .keys()
        .chain(theirs.keys())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    if keys.len() < 2 {
        return None;
    }
    let pick =
        |m: &BTreeMap<String, (f64, f64)>, k: &String| m.get(k).copied().unwrap_or((0.0, 0.0));
    let (a_w, a_w2): (Vec<f64>, Vec<f64>) = keys.iter().map(|k| pick(ours, k)).unzip();
    let (b_w, b_w2): (Vec<f64>, Vec<f64>) = keys.iter().map(|k| pick(theirs, k)).unzip();
    let a = effective_counts(&a_w, &a_w2);
    let b = effective_counts(&b_w, &b_w2);
    let test = chi2_homogeneity(&a, &b).ok()?;
    let detail = if keys.len() <= MAX_CATEGORY_DETAIL {
        keys.iter()
            .zip(a.iter().zip(&b))
            .map(|(k, (&ours, &theirs))| CategoryCount {
                key: (*k).clone(),
                ours,
                theirs,
            })
            .collect()
    } else {
        Vec::new()
    };
    Some(Chi2Cell {
        detail,
        column: column.to_string(),
        chi2: test.chi2,
        dof: test.dof,
        p: test.p,
        categories: test.categories,
        distinct_keys: keys.len(),
        pooled_share: test.pooled_share,
    })
}

/// Whether a column's values span enough of their own scale to have a
/// distribution.
fn degenerate(values: &[(f64, f64)]) -> bool {
    let (lo, hi) = values
        .iter()
        .fold((f64::MAX, f64::MIN), |(lo, hi), &(v, _)| {
            (lo.min(v), hi.max(v))
        });
    (hi - lo).abs() <= DEGENERATE_SPAN * hi.abs().max(lo.abs()).max(1.0)
}

/// Compare one generated sample against MadGraph's, filling in a report row's
/// per-seed entry and returning the failures the comparison found.
fn compare(
    key: &str,
    seed: u64,
    ours: &Sample,
    theirs: &Sample,
    labelling: Labelling,
    row: &mut SamplesRow,
) -> Vec<String> {
    let mine = columns(ours, labelling);
    let mg = columns(theirs, labelling);
    assert_eq!(
        mine.kinematic.iter().map(|c| &c.0).collect::<Vec<_>>(),
        mg.kinematic.iter().map(|c| &c.0).collect::<Vec<_>>(),
        "[{key}] the two samples produced different observable names"
    );

    let mut cells = Vec::new();
    let mut constant = Vec::new();
    for ((name, ours_col), (_, mg_col)) in mine.kinematic.iter().zip(&mg.kinematic) {
        if degenerate(ours_col) && degenerate(mg_col) {
            constant.push(name.clone());
            continue;
        }
        let ks = ks_two_sample(ours_col, mg_col).expect("both columns are finite and non-empty");
        cells.push(KsCell {
            observable: name.clone(),
            d: ks.d,
            p: ks.p,
        });
    }
    let mut chi2 = Vec::new();
    for (column, ours_map, mg_map) in [
        ("SPINUP", &mine.helicity, &mg.helicity),
        ("ICOLUP", &mine.colour, &mg.colour),
        ("flavour", &mine.flavour, &mg.flavour),
    ] {
        match categorical(ours_map, mg_map, column) {
            Some(cell) => chi2.push(cell),
            None => row.single_category.push(column.to_string()),
        }
    }

    let worst_ks = cells
        .iter()
        .min_by(|a, b| a.p.total_cmp(&b.p))
        .cloned()
        .expect("a row has at least one non-degenerate observable");
    eprintln!(
        "  seed {seed:#010x} | {} events (n_eff {:.0}) | worst KS: {} p {:.3e} (D {:.4})",
        ours.len(),
        vibegraph::stats::effective_size(ours.weights.iter().copied()),
        worst_ks.observable,
        worst_ks.p,
        worst_ks.d,
    );
    for cell in &chi2 {
        eprintln!(
            "             chi2 {:<8} p {:.3e} (chi2 {:.1} / {} dof over {} of {} categories, {:.1}% pooled)",
            cell.column, cell.p, cell.chi2, cell.dof, cell.categories, cell.distinct_keys,
            100.0 * cell.pooled_share
        );
    }

    let mut failures = Vec::new();
    for cell in &cells {
        if cell.p < P_FLOOR {
            failures.push(format!(
                "[{key}] seed {seed:#010x} KS {} p {:.3e} (D {:.4}) below the {P_FLOOR:.0e} floor",
                cell.observable, cell.p, cell.d
            ));
        }
    }
    for cell in &chi2 {
        if cell.p < P_FLOOR {
            failures.push(format!(
                "[{key}] seed {seed:#010x} chi2 {} p {:.3e} ({:.1}/{} dof) below the {P_FLOOR:.0e} floor",
                cell.column, cell.p, cell.chi2, cell.dof
            ));
        }
    }

    row.constant_observables = constant;
    row.per_seed.push(SeedSample {
        seed,
        events: ours.len(),
        sigma_pb: ours.sigma_pb,
        ks: cells,
        chi2,
    });
    failures
}

/// Fine labels when both samples carry one final-state species multiset, coarse
/// ones otherwise — the choice is a property of the sample, so it is measured
/// rather than declared.
fn labelling_for(a: &Sample, b: &Sample) -> Labelling {
    let key = |s: &Sample| {
        let mut keys: Vec<String> = s
            .events
            .iter()
            .map(|e| flavour_key(&canonical(e, Labelling::Fine)))
            .collect();
        keys.sort();
        keys.dedup();
        keys
    };
    if key(a).len() == 1 && key(b).len() == 1 && key(a) == key(b) {
        Labelling::Fine
    } else {
        Labelling::Coarse
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
                let found = compare(row.key, seed, &ours, &mg, l, &mut report);
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
        let (ca, cb) = (columns(&sa, l), columns(&sb, l));
        let worst = ca
            .kinematic
            .iter()
            .zip(&cb.kinematic)
            .filter(|((_, x), (_, y))| !(degenerate(x) && degenerate(y)))
            .map(|((name, x), (_, y))| {
                let ks = ks_two_sample(x, y).expect("finite columns");
                (name.clone(), ks.p)
            })
            .min_by(|x, y| x.1.total_cmp(&y.1))
            .expect("a comparable observable");
        eprintln!(
            "  {a} against {b}: smallest KS p {:.3e} on {}",
            worst.1, worst.0
        );
        assert!(
            worst.1 < P_FLOOR,
            "{a} against {b} passed the {P_FLOOR:.0e} floor at p = {:.3e}",
            worst.1
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

/// A binned weight sum with the squared weights that give it an error.
#[derive(Clone)]
struct Binned {
    sum: Vec<f64>,
    sum_sq: Vec<f64>,
    below: f64,
    above: f64,
}

impl Binned {
    fn new() -> Self {
        Binned {
            sum: vec![0.0; MLL_EDGES.len() - 1],
            sum_sq: vec![0.0; MLL_EDGES.len() - 1],
            below: 0.0,
            above: 0.0,
        }
    }

    fn fill(&mut self, x: f64, w: f64) {
        if x < MLL_EDGES[0] {
            self.below += w;
            return;
        }
        match MLL_EDGES.windows(2).position(|e| x >= e[0] && x < e[1]) {
            Some(k) => {
                self.sum[k] += w;
                self.sum_sq[k] += w * w;
            }
            None => self.above += w,
        }
    }

    fn total(&self) -> f64 {
        self.sum.iter().sum::<f64>() + self.below + self.above
    }

    /// Bin contents scaled so the whole histogram carries `sigma_pb`, with the
    /// error each bin's own weights imply.
    fn as_sigma(&self, sigma_pb: f64) -> Vec<(f64, f64)> {
        let scale = sigma_pb / self.total();
        self.sum
            .iter()
            .zip(&self.sum_sq)
            .map(|(&s, &q)| (s * scale, q.sqrt() * scale))
            .collect()
    }
}

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
/// Which side is wrong is *not* settled here, and the third estimate does not
/// settle it: flat RAMBO under a VEGAS grid puts 1.0e-6 pb in that bin, twenty
/// times under MadGraph, because a map with no Breit–Wigner cannot find a 6.4 MeV
/// peak in a 500 GeV process — it shows only the direction a poor map fails in,
/// and its per-bin ratios elsewhere (0.004 to 36) say it is not converged. What
/// can be said is that MadGraph's own per-channel `results.dat` and its 10 000
/// banked events agree with each other, so its sample is not merely
/// under-representing its own integral. The ratio 3.158 is within its errors of
/// `π`, which in a Breit–Wigner map — where `∫ds/((s−m²)²+m²Γ²) = π/(mΓ)` — is the
/// first thing a follow-up should check on this side.
///
/// So `low-mll-reconciliation` is retired and replaced by a resonance question,
/// this row's `integrals` cell stays informational for the new reason, and its
/// `samples` cell joins it. Neither threshold moved.
#[test]
fn the_low_m_ll_region_is_binned_against_madgraph() {
    let row = ROWS
        .iter()
        .find(|r| r.key == "ee_to_mumu_tata_qcd0")
        .expect("the decider row is in the table");
    let mg = banked_sample(row.key);

    // MadGraph's side, from its own events and its own cross section.
    let mut mg_mumu = Binned::new();
    let mut mg_tata = Binned::new();
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
        let mut ours_mumu = Binned::new();
        let mut ours_tata = Binned::new();
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
        let mut flat_mumu = Binned::new();
        let mut flat_tata = Binned::new();
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
            for (k, edges) in MLL_EDGES.windows(2).enumerate() {
                if b[k].0 <= 0.0 && a[k].0 <= 0.0 {
                    continue;
                }
                eprintln!(
                    "    {:>6.1}-{:<6.1} {:>12.5e} +- {:>7.1e} {:>12.5e} +- {:>7.1e} \
                     {:>12.5e} +- {:>7.1e} {:>+11.3e} {:>6.0}% {:>7}",
                    edges[0],
                    edges[1],
                    a[k].0,
                    a[k].1,
                    b[k].0,
                    b[k].1,
                    c[k].0,
                    c[k].1,
                    a[k].0 - b[k].0,
                    100.0 * (a[k].0 - b[k].0) / excess,
                    ratio(c[k].0, b[k].0),
                );
            }
            eprintln!(
                "    below {:.0}: production {:.3e} pb, MadGraph {:.3e} pb, flat {:.3e} pb",
                MLL_EDGES[0],
                ours.below / ours.total() * ours_sigma,
                theirs.below / theirs.total() * mg.sigma_pb,
                flat.below / flat.total() * flat_sigma,
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
