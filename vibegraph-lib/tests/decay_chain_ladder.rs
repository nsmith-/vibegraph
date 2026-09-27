//! The decay-chain sampler ladder: how the cost of an unweighted event grows with
//! the number of final-state legs a decay chain adds.
//!
//! Each rung is integrated to a fixed relative accuracy, its channel maxima are
//! scanned on the trained grids, and unweighted events are drawn by the same
//! accept/reject pass `vibegraph generate` runs (no file is written: a decay-chain
//! event file waits on its status-2 resonance records). For every rung it prints
//!
//! - the final-state leg count, channels and subprocesses;
//! - the cost of one integrand point and of the squared matrix element alone at
//!   that point, so the phase-space and bookkeeping share can be read off;
//! - the unweighting efficiency and the wall time per unweighted event;
//! - the integration's evaluations and wall time to the target accuracy.
//!
//! Each decayed rung sits beside its undecayed core, so the question it answers
//! is whether decaying the resonances in the matrix element costs more than the
//! extra matrix-element work: if the Breit–Wigner-mapped resonances keep the
//! sampler's efficiency near the core's, the per-event cost grows as the
//! matrix element does and no decay-after-generation step is needed.
//!
//! A measurement, not a gate: it asserts only that every rung produced events.
//!
//!     cargo test -p vibegraph-lib --profile release-debug --features extended-validation \
//!         --test decay_chain_ladder -- --ignored --nocapture
//!
//! `LADDER_ROWS=zz,zz_emu` runs a subset; `LADDER_EVENTS` sets the event count.

use std::path::{Path, PathBuf};
use std::time::Instant;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use vibegraph::budget::{BlockAllocation, Budget, StopSignal};
use vibegraph::cuts::{Cuts, ForcedResonances};
use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, DiagramSet, ParsingOptions};
use vibegraph::hadronic::{
    compile_subprocesses, initial_spin_color_average, process_external_legs, ChannelIntegration,
    FixedBeamIntegrand, FixedBeams,
};
use vibegraph::helas::eval::BoundAmplitude;
use vibegraph::helas::LorentzVector;
use vibegraph::pdf::PdfSet;
use vibegraph::phasespace::{rambo_massive, MapOptions};
use vibegraph::proton::{derive_flavor_groups, ProtonIntegrand};
use vibegraph::runcard::RunCard;
use vibegraph::ufo::sm::{sm_model, SMRestrict};
use vibegraph::ufo::{EvaluatedModel, UFOModel};
use vibegraph::unweight::{ChannelIntegrand, MaxRule, ScanBudget, UnweightStats, Unweighter};
use vibegraph::vegas::VegasResult;

type V = LorentzVector<f64>;

const PDF_SET: &str = "NNPDF23_lo_as_0130_qed";
const SEED: u64 = 20_260_926;
/// Relative accuracy every rung is integrated to before its maxima are scanned.
const TARGET_REL: f64 = 5e-3;
const NEVAL: usize = 120_000;
/// Points the matrix-element and integrand timings average over.
const TIMING_POINTS: usize = 20_000;
/// The survey the α-adaptation spends, as `vibegraph integrate` spends it.
const SURVEY: usize = 40_000;

fn madgraph_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph")
}

fn pdf_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/pdf")
}

struct Rung {
    name: &'static str,
    process: &'static str,
    run_card: &'static str,
}

const RUNGS: &[Rung] = &[
    Rung {
        name: "zz",
        process: "e+ e- > z z",
        run_card: "decay_chain_ee500_run_card.dat",
    },
    Rung {
        name: "zz_emu",
        process: "e+ e- > z z, z > e+ e-, z > mu+ mu-",
        run_card: "decay_chain_ee500_run_card.dat",
    },
    Rung {
        name: "ttx",
        process: "p p > t t~",
        run_card: "decay_chain_pp13_run_card.dat",
    },
    Rung {
        name: "ttx_lep",
        process: "p p > t t~, t > b e+ ve, t~ > b~ mu- vm~",
        run_card: "decay_chain_pp13_run_card.dat",
    },
    Rung {
        name: "ttx_semilep",
        process: "p p > t t~, (t > w+ b, w+ > l+ vl), t~ > b~ j j",
        run_card: "decay_chain_pp13_run_card.dat",
    },
    Rung {
        name: "ttxhh",
        process: "p p > t t~ h h",
        run_card: "decay_chain_pp13_run_card.dat",
    },
    Rung {
        name: "ttxhh_dec",
        process: "p p > t t~ h h, (t > w+ b, w+ > j j), (t~ > w- b~, w- > l- vl~)",
        run_card: "decay_chain_pp13_run_card.dat",
    },
];

/// What one rung measured.
struct Measured {
    n_final: usize,
    subprocesses: usize,
    channels: usize,
    diagrams: usize,
    integral: VegasResult,
    integration_points: u64,
    integration_s: f64,
    /// One integrand point, phase space and cuts included (ns).
    point_ns: f64,
    /// The squared matrix element alone, summed over the subprocesses a point
    /// evaluates (ns).
    me_ns: f64,
    stats: UnweightStats,
    events: usize,
    generation_s: f64,
    scan_s: f64,
}

fn sets_of(process: &str, model: &UFOModel) -> Vec<DiagramSet> {
    let card = parse_proc_card(
        &format!("import model sm\ngenerate {process}\n"),
        &ParsingOptions::default(),
    )
    .unwrap();
    generate_from_proc_card(&card, model)
        .unwrap()
        .into_iter()
        .filter(|s| !s.diagrams.is_empty())
        .collect()
}

/// Mean time of `f` over `TIMING_POINTS` calls, in ns.
fn time_ns(mut f: impl FnMut(usize)) -> f64 {
    let start = Instant::now();
    for i in 0..TIMING_POINTS {
        f(i);
    }
    start.elapsed().as_secs_f64() * 1e9 / TIMING_POINTS as f64
}

/// Centre-of-mass momenta at `sqrt_s`, beams along `±z`, outgoing flat.
fn cm_points(sqrt_s: f64, masses: &[f64], n: usize) -> Vec<Vec<V>> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..n)
        .map(|_| {
            let e = sqrt_s / 2.0;
            let mut p = vec![V::new(e, 0.0, 0.0, e), V::new(e, 0.0, 0.0, -e)];
            p.extend(rambo_massive(sqrt_s, masses, &mut rng));
            p
        })
        .collect()
}

/// A uniform point for channel `j`'s integrand.
fn uniforms(rng: &mut ChaCha8Rng, n: usize) -> Vec<f64> {
    use rand::Rng;
    (0..n).map(|_| rng.random::<f64>()).collect()
}

fn budget() -> Budget {
    Budget::Target {
        target_rel: TARGET_REL,
        neval: NEVAL,
        min_iters: 6,
        max_iters: 60,
        max_points: 200_000_000,
    }
}

/// Scan the trained grids and draw `events` unweighted events.
fn unweight<I: ChannelIntegrand + Sync>(
    integ: &I,
    per_channel: &[ChannelIntegration],
    events: usize,
) -> (UnweightStats, usize, f64, f64) {
    let start = Instant::now();
    let mut unweighter = Unweighter::scan_with(
        integ,
        per_channel
            .iter()
            .map(|c| (&c.grid, ScanBudget::IntegrationShare.draws_for(c.neval))),
        SEED ^ 0x5CA4,
        MaxRule::default(),
    );
    let scan_s = start.elapsed().as_secs_f64();
    let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ 0x6E4);
    let start = Instant::now();
    let mut accepted = 0;
    while accepted < events {
        if unweighter.next_event(integ, &mut rng, 10_000_000).is_none() {
            break;
        }
        accepted += 1;
    }
    let generation_s = start.elapsed().as_secs_f64();
    (unweighter.stats().clone(), accepted, scan_s, generation_s)
}

fn measure_fixed(
    rung: &Rung,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    events: usize,
) -> Measured {
    let rc = RunCard::parse(&std::fs::read_to_string(madgraph_dir().join(rung.run_card)).unwrap())
        .unwrap();
    let sets = sets_of(rung.process, model);
    let evals = compile_subprocesses(&sets, model, evaluated).unwrap();
    let bounds: Vec<_> = evals
        .iter()
        .map(|e| BoundAmplitude::<f64>::bind(e, evaluated))
        .collect();
    let rep = &evals[0];
    let legs = process_external_legs(rep, model, evaluated);
    let diagrams: Vec<_> = sets
        .iter()
        .flat_map(|s| s.diagrams.iter().cloned())
        .collect();
    let cuts = Cuts::compile_with(&rc, &legs, &ForcedResonances::of(&diagrams, evaluated)).unwrap();
    let final_masses: Vec<f64> = legs.iter().filter(|l| l.is_final).map(|l| l.mass).collect();
    let spin_color_avg = initial_spin_color_average(rep, model, evaluated);
    let initial = FixedBeams::from_run_card(&rc, &legs);
    let amps: Vec<&BoundAmplitude<f64>> = bounds.iter().collect();
    let mut integ =
        FixedBeamIntegrand::new(amps, &cuts, initial, final_masses.clone(), spin_color_avg);
    integ.set_map_options(MapOptions::default());
    integ
        .use_running_coupling(&diagrams, model, evaluated, &rc)
        .unwrap();
    integ.use_multichannel(&diagrams, evaluated, SURVEY, 6, SEED);

    let start = Instant::now();
    let (per_channel, integral, conv) =
        integ.adapt_grids_budget(budget(), BlockAllocation::Neyman, SEED, &StopSignal::new());
    let integration_s = start.elapsed().as_secs_f64();

    let points = cm_points(initial.sqrt_s(), &final_masses, TIMING_POINTS);
    let mut scratch: Vec<_> = bounds.iter().map(|b| b.scratch_space()).collect();
    let me_ns = time_ns(|i| {
        for (b, s) in bounds.iter().zip(scratch.iter_mut()) {
            std::hint::black_box(b.eval_m2(&points[i], s));
        }
    });
    let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ 0x77);
    let ndim = integ.channel_grid_ndim() + integ.scale_draw_ndim();
    let us: Vec<Vec<f64>> = (0..TIMING_POINTS)
        .map(|_| uniforms(&mut rng, ndim))
        .collect();
    let n_channels = integ.channel_count();
    let point_ns = time_ns(|i| {
        std::hint::black_box(integ.value_in_channel(i % n_channels, &us[i]));
    });

    let (stats, accepted, scan_s, generation_s) = unweight(&integ, &per_channel, events);
    Measured {
        n_final: legs.iter().filter(|l| l.is_final).count(),
        subprocesses: evals.len(),
        channels: n_channels,
        diagrams: diagrams.len(),
        integral,
        integration_points: conv.points,
        integration_s,
        point_ns,
        me_ns,
        stats,
        events: accepted,
        generation_s,
        scan_s,
    }
}

fn measure_proton(
    rung: &Rung,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    set: &PdfSet,
    events: usize,
) -> Measured {
    let rc = RunCard::parse(&std::fs::read_to_string(madgraph_dir().join(rung.run_card)).unwrap())
        .unwrap();
    let pdf = set.member(0).unwrap();
    let sets = sets_of(rung.process, model);
    let diagrams: usize = sets.iter().map(|s| s.diagrams.len()).sum();
    let groups = derive_flavor_groups(sets, model, evaluated, &rc).unwrap();
    let amps: Vec<BoundAmplitude<f64>> = groups
        .groups()
        .iter()
        .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), evaluated))
        .collect();
    let mut integ = ProtonIntegrand::new_with_maps(
        &groups,
        &amps,
        evaluated,
        &pdf,
        rc.ebeam1 + rc.ebeam2,
        rc.dsqrt_q2fact1,
        MapOptions::default(),
    )
    .unwrap();
    integ
        .use_run_card_scales(model, evaluated, &rc, Some(&set.info.alpha_s))
        .unwrap();
    integ.adapt_alphas(SEED, SURVEY, 6, 0.5);

    let start = Instant::now();
    let (per_channel, integral, conv) =
        integ.adapt_grids_budget(budget(), BlockAllocation::Neyman, SEED, &StopSignal::new());
    let integration_s = start.elapsed().as_secs_f64();

    let final_masses: Vec<f64> = groups.groups()[0].evaluator().external_particles()[2..]
        .iter()
        .map(|&id| evaluated.mass(id))
        .collect();
    let sqrt_s = final_masses.iter().sum::<f64>() + 300.0;
    let points = cm_points(sqrt_s, &final_masses, TIMING_POINTS);
    let mut scratch: Vec<_> = amps.iter().map(|b| b.scratch_space()).collect();
    let me_ns = time_ns(|i| {
        for (b, s) in amps.iter().zip(scratch.iter_mut()) {
            std::hint::black_box(b.eval_m2(&points[i], s));
        }
    });
    let mut rng = ChaCha8Rng::seed_from_u64(SEED ^ 0x77);
    let ndim = integ.channel_grid_ndim() + integ.scale_draw_ndim();
    let us: Vec<Vec<f64>> = (0..TIMING_POINTS)
        .map(|_| uniforms(&mut rng, ndim))
        .collect();
    let n_channels = integ.channel_count();
    let point_ns = time_ns(|i| {
        std::hint::black_box(integ.value_in_channel(i % n_channels, &us[i]));
    });

    let (stats, accepted, scan_s, generation_s) = unweight(&integ, &per_channel, events);
    Measured {
        n_final: final_masses.len(),
        subprocesses: groups.groups().len(),
        channels: n_channels,
        diagrams,
        integral,
        integration_points: conv.points,
        integration_s,
        point_ns,
        me_ns,
        stats,
        events: accepted,
        generation_s,
        scan_s,
    }
}

#[test]
#[ignore = "a measurement over minutes of integration; run with --ignored --nocapture"]
fn decay_chain_sampler_ladder() {
    let model = sm_model(SMRestrict::Default);
    let evaluated = EvaluatedModel::from_model(model.clone());
    let set = PdfSet::load(&pdf_dir().join(PDF_SET), PDF_SET).expect("the fetched PDF set");
    let events: usize = std::env::var("LADDER_EVENTS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5_000);
    let wanted: Option<Vec<String>> = std::env::var("LADDER_ROWS")
        .ok()
        .map(|v| v.split(',').map(str::to_owned).collect());
    println!(
        "{:<12} {:>3} {:>4} {:>5} {:>6} {:>13} {:>9} {:>8} {:>8} {:>9} {:>9} {:>9} {:>10}",
        "rung",
        "n_f",
        "sub",
        "chan",
        "diag",
        "σ (pb)",
        "rel err",
        "pt (ns)",
        "ME (ns)",
        "int (s)",
        "ε_unw",
        "ms/event",
        "trials/ev"
    );
    for rung in RUNGS {
        if let Some(w) = &wanted {
            if !w.iter().any(|n| n == rung.name) {
                continue;
            }
        }
        let proton = rung.run_card.contains("pp13");
        let m = if proton {
            measure_proton(rung, &model, &evaluated, &set, events)
        } else {
            measure_fixed(rung, &model, &evaluated, events)
        };
        let unit = vibegraph::phasespace::GEV2_TO_PB;
        let eff = m.stats.efficiency();
        println!(
            "{:<12} {:>3} {:>4} {:>5} {:>6} {:>13.6e} {:>9.2e} {:>8.0} {:>8.0} {:>9.1} {:>9.3e} {:>9.3} {:>10.1}",
            rung.name,
            m.n_final,
            m.subprocesses,
            m.channels,
            m.diagrams,
            m.integral.integral * unit,
            m.integral.std_dev / m.integral.integral,
            m.point_ns,
            m.me_ns,
            m.integration_s,
            eff,
            1e3 * m.generation_s / m.events.max(1) as f64,
            m.stats.trials as f64 / m.events.max(1) as f64,
        );
        println!(
            "             {} integration points, scan {:.1} s, {} events in {:.1} s, overweight \
             fraction {:.2e}, excess share {:.2e}",
            m.integration_points,
            m.scan_s,
            m.events,
            m.generation_s,
            m.stats.overweight_fraction(),
            m.stats.excess_share(),
        );
        assert!(m.events > 0, "{}: no event accepted", rung.name);
    }
}
