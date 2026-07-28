//! Les Houches output gate: does the writer produce the format, and does the
//! generator fill it with a coherent event?
//!
//! # What can and cannot be checked against MadGraph here
//!
//! vibegraph does not share MadGraph's random number generator, so our events are
//! not its events and no per-event comparison of *content* against a banked
//! `unweighted_events.lhe.gz` is possible — it would be comparing unrelated
//! points. Statistical agreement between the two samples is a distribution-level
//! question and is deliberately left to a later validation pass.
//!
//! What MadGraph can still serve as, with no shared sampling at all, is a
//! **format oracle**. [`banked_files_round_trip_byte_for_byte`] parses every
//! banked run's `<init>` and every one of its `<event>` blocks into this crate's
//! record types, writes them back out, and requires the bytes to be identical.
//! That pins the whole layout at once — field order, column widths, the exponent
//! spelling, the `px py pz E` permutation, the sign on a negative zero — against
//! a file 10k events long that a real shower reads.
//!
//! # The error classes this gate provably cannot detect
//!
//! * **Anything about which event we generated.** The round-trip re-emits values
//!   MadGraph chose, so it is blind to every physics field being filled with the
//!   wrong number; and the end-to-end test compares our record against our own
//!   generator, so it is blind to a wrong matrix element, cut or sampler. Those
//!   are covered by `validate_helas_mg`, `validate_sigma` and
//!   `validate_unweighting`.
//! * **The colour-line integers.** Only the connectivity they induce is physical,
//!   and 4 of MadGraph's 24 banked subprocesses relabel the same connectivity, so
//!   nothing here or in `color_flow_tags_oracle` compares labels.
//! * **Which helicity an event should have carried.** `SPINUP` is a selection off
//!   a diagonal accumulator; the end-to-end test checks only that it is one of the
//!   subprocess's surviving combinations.
//! * **`SCALUP` as `μF` rather than `μR`.** Every process whose clustering this
//!   crate computes has the two equal, so no banked file separates them. The
//!   distinction is pinned by a unit test in `lhef::build` built with them apart,
//!   and measured on MadGraph's `2 → 6` runs by `validate_scales`.
//!
//! Runs only when the gitignored MadGraph `output/` tree is present.

use std::path::{Path, PathBuf};
use std::process::Command;

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use vibegraph::cuts::Cuts;
use vibegraph::hadronic::{
    compile_subprocesses, initial_spin_color_average, process_external_legs, FixedBeamIntegrand,
};
use vibegraph::helas::eval::BoundAmplitude;
use vibegraph::lhef::build::{scalup, EventHeader, SubprocessRecord, WeightNormalisation};
use vibegraph::lhef::parse::LheFile;
use vibegraph::lhef::record::{
    LheEvent, LheInit, LheProcess, WeightStrategy, STATUS_INCOMING, STATUS_OUTGOING,
};
use vibegraph::lhef::write::{generator_element, write_event, write_init, LheWriter};
use vibegraph::phasespace::GEV2_TO_PB;
use vibegraph::runcard::{BeamMode, RunCard};
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::EvaluatedModel;
use vibegraph::unweight::Unweighter;

mod common;

fn output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/output")
}

/// Every banked run directory carrying an unweighted event file.
fn banked_runs() -> Vec<(String, PathBuf)> {
    let mut runs: Vec<(String, PathBuf)> = std::fs::read_dir(output_dir())
        .expect("MadGraph output directory")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if !path
                .join("Events/run_01/unweighted_events.lhe.gz")
                .is_file()
            {
                return None;
            }
            Some((path.file_name()?.to_string_lossy().into_owned(), path))
        })
        .collect();
    runs.sort();
    runs
}

fn banked_text(run: &Path) -> String {
    let lhe = run.join("Events/run_01/unweighted_events.lhe.gz");
    let out = Command::new("gzip")
        .args(["-dc", lhe.to_str().unwrap()])
        .output()
        .expect("gzip -dc");
    assert!(out.status.success(), "gzip failed on {}", lhe.display());
    String::from_utf8(out.stdout).expect("the banked files are ASCII")
}

/// The part of a banked file this crate models: everything from `<init>` up to
/// the closing root tag, which is the `<init>` block followed by the events.
fn record_span(text: &str) -> String {
    text.lines()
        .skip_while(|l| l.trim() != "<init>")
        .take_while(|l| l.trim() != "</LesHouchesEvents>")
        .fold(String::new(), |mut acc, line| {
            acc.push_str(line);
            acc.push('\n');
            acc
        })
}

fn serialise(file: &LheFile) -> String {
    let mut out = Vec::new();
    write_init(&mut out, &file.init).expect("write");
    for event in &file.events {
        write_event(&mut out, event).expect("write");
    }
    String::from_utf8(out).expect("ASCII")
}

/// MadGraph as the format oracle: its own bytes, through our record types, back
/// to the same bytes.
#[test]
fn banked_files_round_trip_byte_for_byte() {
    if !output_dir().exists() {
        eprintln!(
            "skipping: {} absent (run `pixi run -e madgraph build-diagrams`)",
            output_dir().display()
        );
        return;
    }
    let runs = banked_runs();
    assert!(!runs.is_empty(), "no banked runs with an event file");

    let mut total_events = 0usize;
    let mut total_particles = 0usize;
    for (name, run) in &runs {
        let text = banked_text(run);
        let file = LheFile::parse(&text).unwrap_or_else(|e| panic!("{name}: {e}"));
        let expected = record_span(&text);
        let rendered = serialise(&file);
        if rendered != expected {
            let (line, (got, want)) = rendered
                .lines()
                .zip(expected.lines())
                .enumerate()
                .find(|(_, (a, b))| a != b)
                .map(|(i, pair)| (i + 1, pair))
                .unwrap_or((0, ("<length differs>", "<length differs>")));
            panic!(
                "{name}: re-serialised block {line} of the record span differs\n  \
                 wrote    {got:?}\n  MadGraph {want:?}"
            );
        }
        let particles: usize = file.events.iter().map(|e| e.nup()).sum();
        println!(
            "{name}: {} events, {particles} legs, {} process entries -- byte-identical",
            file.events.len(),
            file.init.processes.len()
        );
        total_events += file.events.len();
        total_particles += particles;
    }
    println!(
        "LHE format: {total_events} events / {total_particles} particle lines across {} banked \
         runs re-serialise byte-for-byte",
        runs.len()
    );
}

/// The round-trip above is only evidence if it is sensitive to the fields a
/// writer can plausibly get wrong. Each mutation below is a convention error that
/// no `|M|²`-level gate can see, and each must break the comparison.
#[test]
fn the_round_trip_is_sensitive_to_every_convention_sensitive_field() {
    if !output_dir().exists() {
        eprintln!("skipping: no banked MadGraph output");
        return;
    }
    // A gluon-initiated process, so every leg carries colour in both slots and a
    // slot swap is visible on every line.
    let run = output_dir().join("gg_to_ttx");
    if !run.exists() {
        eprintln!("skipping: gg_to_ttx not banked");
        return;
    }
    let text = banked_text(&run);
    let file = LheFile::parse(&text).expect("parse");
    let expected = record_span(&text);
    assert_eq!(serialise(&file), expected, "the unmutated file must match");

    let mutations: Vec<(&str, Box<dyn Fn(&mut LheEvent)>)> = vec![
        (
            "MOTHUP dropped on the outgoing legs",
            Box::new(|e: &mut LheEvent| {
                for p in e
                    .particles
                    .iter_mut()
                    .filter(|p| p.status == STATUS_OUTGOING)
                {
                    p.mothers = [0, 0];
                }
            }),
        ),
        (
            "MOTHUP order swapped",
            Box::new(|e: &mut LheEvent| {
                for p in e
                    .particles
                    .iter_mut()
                    .filter(|p| p.status == STATUS_OUTGOING)
                {
                    p.mothers.swap(0, 1);
                }
            }),
        ),
        (
            "ISTUP sign flipped on the incoming legs",
            Box::new(|e: &mut LheEvent| {
                for p in e
                    .particles
                    .iter_mut()
                    .filter(|p| p.status == STATUS_INCOMING)
                {
                    p.status = STATUS_OUTGOING;
                }
            }),
        ),
        (
            "ICOLUP slots 1 and 2 exchanged",
            Box::new(|e: &mut LheEvent| {
                for p in e.particles.iter_mut() {
                    p.color.swap(0, 1);
                }
            }),
        ),
        (
            "incoming momenta crossed to all-outgoing",
            Box::new(|e: &mut LheEvent| {
                for p in e
                    .particles
                    .iter_mut()
                    .filter(|p| p.status == STATUS_INCOMING)
                {
                    for c in p.momentum.iter_mut() {
                        *c = -*c;
                    }
                }
            }),
        ),
        (
            "momentum components rotated (px py pz E read as E px py pz)",
            Box::new(|e: &mut LheEvent| {
                for p in e.particles.iter_mut() {
                    p.momentum = [p.momentum[1], p.momentum[2], p.momentum[3], p.momentum[0]];
                }
            }),
        ),
        (
            "mass replaced by the momentum's own invariant",
            Box::new(|e: &mut LheEvent| {
                for p in e.particles.iter_mut() {
                    let [en, px, py, pz] = p.momentum;
                    p.mass = (en * en - px * px - py * py - pz * pz).max(0.0).sqrt();
                }
            }),
        ),
        (
            "SPINUP zeroed",
            Box::new(|e: &mut LheEvent| {
                for p in e.particles.iter_mut() {
                    p.spin = 0.0;
                }
            }),
        ),
    ];

    for (what, mutate) in mutations {
        let mut mutated = file.clone();
        for event in mutated.events.iter_mut() {
            mutate(event);
        }
        assert_ne!(
            serialise(&mutated),
            expected,
            "the round-trip is blind to: {what}"
        );
        println!("  round-trip detects: {what}");
    }
}

/// One process to generate events for.
struct Row {
    dir: &'static str,
    process: &'static str,
    neval: usize,
    niter: usize,
}

/// A colourless `2 → 2` and a gluon-initiated coloured one, so the record layer
/// is exercised with and without colour lines, and with and without a strong
/// coupling to put in `AQCDUP`.
const ROWS: &[Row] = &[
    Row {
        dir: "ee_to_mumu",
        process: "e+ e- > mu+ mu-",
        neval: 12_000,
        niter: 4,
    },
    Row {
        dir: "gg_to_ttx",
        process: "g g > t t~",
        neval: 15_000,
        niter: 4,
    },
];

const SEED: u64 = 20_260_728;
const SCAN_SEED: u64 = 0x5CA7_1EF0;
const GEN_SEED: u64 = 0xE7E7_1EF0;
const TRIALS: usize = 40_000;
const MULTICHANNEL_SURVEY: usize = 20_000;
const MULTICHANNEL_ITERS: usize = 5;
/// How much of a value survives the format, per field. The `<init>` block prints
/// its cross sections with C's bare `%e` — seven significant digits — while an
/// event's `XWGTUP` gets eight, so a file is a lossy record of the run that wrote
/// it and nothing read back out of one agrees with what went in more closely.
const INIT_PRECISION: f64 = 1e-6;
const WEIGHT_PRECISION: f64 = 1e-7;

fn param_card(dir: &str) -> ParamCard {
    let path = output_dir().join(dir).join("Cards/param_card.dat");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.parse::<ParamCard>().ok())
        .expect("param card")
}

/// Generate an unweighted sample through the production path, write it as a Les
/// Houches file, read it back, and check the record against what the generator
/// said the event was.
#[test]
fn generated_events_serialise_into_a_coherent_file() {
    if !output_dir().exists() {
        eprintln!("skipping: no banked MadGraph output");
        return;
    }
    for row in ROWS {
        if !output_dir().join(row.dir).exists() {
            eprintln!("-- {} -- skipped: not banked", row.dir);
            continue;
        }
        generate_and_check(row);
    }
}

fn generate_and_check(row: &Row) {
    let run_card = RunCard::parse_file(&output_dir().join(row.dir).join("Cards/run_card.dat"))
        .expect("run card");
    assert_eq!(run_card.beam_mode(), BeamMode::FixedEnergy);
    let sqrt_s = run_card.ebeam1 + run_card.ebeam2;
    let params = param_card(row.dir);
    let alpha_qed = 1.0 / params.get("sminputs", &[1]).expect("aEWM1 in SMINPUTS");

    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &params);
    let sets = common::generate(row.process);
    let evals = compile_subprocesses(&sets, &model, &evaluated).expect("compile subprocesses");
    let bounds: Vec<_> = evals
        .iter()
        .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
        .collect();
    let rep = &evals[0];
    let legs = process_external_legs(rep, &model, &evaluated);
    let cuts = Cuts::compile(&run_card, &legs).expect("cuts compile");
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
        sqrt_s,
        final_masses.clone(),
        spin_color_avg,
    );
    integ
        .use_running_coupling(&diagrams, &model, &evaluated, &run_card)
        .expect("scale prescription compiles");
    integ.use_multichannel(
        &diagrams,
        &evaluated,
        MULTICHANNEL_SURVEY,
        MULTICHANNEL_ITERS,
        SEED,
    );

    // The record layer's view of each subprocess: PDG codes, masses, flow tags.
    let records: Vec<SubprocessRecord> = evals
        .iter()
        .map(|e| SubprocessRecord::new(e, &model, &evaluated).expect("subprocess record"))
        .collect();

    let (channels, result) = integ.adapt_grids(row.neval, row.niter, SEED);
    let sigma_pb = result.integral * GEV2_TO_PB;
    let mut uw = Unweighter::scan(
        &integ,
        channels.iter().map(|c| (&c.grid, c.neval)),
        SCAN_SEED,
    );

    // Generate first, so the `<init>` block reports the sample it actually
    // describes rather than a prediction of it.
    let mut rng = ChaCha8Rng::seed_from_u64(GEN_SEED);
    let mut momenta = Vec::new();
    let mut generated: Vec<(usize, Vec<i32>, usize, Vec<[f64; 4]>, f64, f64, f64)> = Vec::new();
    for _ in 0..TRIALS {
        let Some(point) = uw.trial(&integ, &mut rng) else {
            continue;
        };
        integ.event_in_channel(point.channel, &point.u, &mut momenta);
        let Some(selection) =
            integ.select_event(&momenta, [rng.random(), rng.random(), rng.random()])
        else {
            panic!("an accepted point carries weight, so its labels are defined");
        };
        let externals: Vec<[f64; 4]> = integ
            .beams()
            .iter()
            .chain(momenta.iter())
            .map(|p| [p.e(), p.px(), p.py(), p.pz()])
            .collect();
        // The scale the matrix element itself ran at, when one was installed; a
        // process with no strong coupling has none, and the run card's own
        // factorisation scale stands in.
        let (scale, alpha_qcd) = match integ.event_scales(&momenta) {
            Some(scales) => {
                let scales = scales.expect("the scale prescription accepts a sampled point");
                let alpha_s = integ
                    .running_alpha_s()
                    .map(|r| r.eval(scales.mu_r))
                    .expect("a scale-aware run has a running coupling");
                (scalup(&scales), alpha_s)
            }
            None => (run_card.dsqrt_q2fact1.max(run_card.dsqrt_q2fact2), 0.0),
        };
        generated.push((
            selection.subprocess,
            selection.helicity,
            selection.flow,
            externals,
            point.weight,
            scale,
            alpha_qcd,
        ));
    }
    assert!(
        generated.len() > 200,
        "{}: only {} events in {TRIALS} trials",
        row.dir,
        generated.len()
    );

    let stats = uw.stats().clone();
    let sigma_events = uw.total_w_max() * GEV2_TO_PB * stats.event_weight_sum / TRIALS as f64;
    let normalisation = WeightNormalisation::new(sigma_events, stats.mean_event_weight());
    let max_weight = generated.iter().map(|g| g.4).fold(0.0f64, f64::max);
    let init = LheInit {
        beam_pdg: [records[0].pdg()[0], records[0].pdg()[1]],
        beam_energy: [sqrt_s / 2.0, sqrt_s / 2.0],
        pdf_group: [0, 0],
        pdf_set: [0, 0],
        weight_strategy: WeightStrategy::MeanCrossSectionPb,
        processes: vec![LheProcess {
            xsec_pb: sigma_events,
            xerr_pb: result.std_dev * GEV2_TO_PB,
            xmax: normalisation.xwgtup(max_weight),
            id: 1,
        }],
        trailer: vec![generator_element("vibegraph", "0.1", "")],
    };

    let mut out = Vec::new();
    let mut writer =
        LheWriter::begin(&mut out, &init, Some(&format!("process {}", row.process))).expect("open");
    for (sub, helicity, flow, externals, weight, scale, alpha_qcd) in &generated {
        let header = EventHeader {
            process_id: 1,
            weight: normalisation.xwgtup(*weight),
            scale: *scale,
            alpha_qed,
            alpha_qcd: *alpha_qcd,
        };
        let event = records[*sub]
            .event(externals, helicity, *flow, header)
            .expect("record");
        writer.write_event(&event).expect("write");
    }
    writer.finish().expect("close");
    let text = String::from_utf8(out).expect("ASCII");

    // Our own output goes back through the reader that reads MadGraph's files,
    // and out again to the same bytes.
    //
    // The reals do not come back bit-identical and are not meant to: the format
    // prints cross sections and weights to eight significant digits and momenta to
    // eleven, so a file is a lossy record of the run that made it. What has to
    // hold is that re-emitting what was read reproduces the file, which is the
    // property a consumer depends on.
    let parsed = LheFile::parse(&text).expect("our own file parses");
    assert_eq!(parsed.events.len(), generated.len());
    assert_eq!(record_span(&text), serialise(&parsed));
    assert_eq!(parsed.init.beam_pdg, init.beam_pdg);
    assert_eq!(parsed.init.pdf_group, init.pdf_group);
    assert_eq!(parsed.init.pdf_set, init.pdf_set);
    assert_eq!(parsed.init.weight_strategy, init.weight_strategy);
    assert_eq!(parsed.init.trailer, init.trailer);
    assert_eq!(parsed.init.processes.len(), 1);
    assert_eq!(parsed.init.processes[0].id, 1);
    for (read, wrote) in [
        (parsed.init.beam_energy[0], init.beam_energy[0]),
        (parsed.init.beam_energy[1], init.beam_energy[1]),
        (parsed.init.processes[0].xsec_pb, sigma_events),
        (parsed.init.processes[0].xmax, init.processes[0].xmax),
    ] {
        assert!(
            (read / wrote - 1.0).abs() < INIT_PRECISION,
            "<init> field read back as {read:.10e}, written as {wrote:.10e}"
        );
    }

    let n_ext = records[0].n_ext();
    let n_in = records[0].n_in();
    let mut worst_conservation = 0.0f64;
    let mut worst_mass = 0.0f64;
    let mut weight_sum = 0.0f64;
    for (index, event) in parsed.events.iter().enumerate() {
        let (sub, helicity, flow, ..) = &generated[index];
        assert_eq!(event.nup(), n_ext, "event {index}: NUP");
        weight_sum += event.weight;

        for (leg, p) in event.particles.iter().enumerate() {
            let incoming = leg < n_in;
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
                if incoming { [0, 0] } else { [1, n_in as i32] },
                "event {index} leg {leg}: MOTHUP"
            );
            assert_eq!(p.lifetime, 0.0);
            assert!(
                helicity.contains(&(p.spin as i32)),
                "event {index} leg {leg}: SPINUP {} is not a selected helicity",
                p.spin
            );
            let [en, px, py, pz] = p.momentum;
            let m2 = en * en - px * px - py * py - pz * pz;
            worst_mass = worst_mass.max((m2 - p.mass * p.mass).abs() / (sqrt_s * sqrt_s));
        }

        // Four-momentum conservation, over the physical momenta the record
        // carries: a writer that crossed the incoming legs would fail here even
        // though every column would still parse.
        for component in 0..4 {
            let balance: f64 = event
                .particles
                .iter()
                .enumerate()
                .map(|(leg, p)| {
                    if leg < n_in {
                        p.momentum[component]
                    } else {
                        -p.momentum[component]
                    }
                })
                .sum();
            worst_conservation = worst_conservation.max(balance.abs() / sqrt_s);
        }

        // The colour lines are the selected flow's, and no other.
        let selected = records[*sub]
            .event(
                &generated[index].3,
                helicity,
                *flow,
                EventHeader {
                    process_id: 1,
                    weight: 1.0,
                    scale: 1.0,
                    alpha_qed: 0.0,
                    alpha_qcd: 0.0,
                },
            )
            .expect("record");
        assert_eq!(
            event.color_connectivity(),
            selected.color_connectivity(),
            "event {index}: colour connectivity is not flow {flow}'s"
        );
        assert!(event.scale > 0.0, "event {index}: SCALUP");
    }

    // `IDWTUP = -4` says the cross section is the mean of the event weights.
    let mean_weight = weight_sum / parsed.events.len() as f64;
    let deviation = (mean_weight / sigma_events - 1.0).abs();
    assert!(
        deviation < WEIGHT_PRECISION,
        "{}: mean XWGTUP {mean_weight:.6e} vs sigma {sigma_events:.6e}",
        row.dir
    );

    println!(
        "-- {} -- {} events written ({} bytes), sigma(events) = {sigma_events:.6e} pb \
         (VEGAS {sigma_pb:.6e} pb)\n  \
         momentum balance <= {worst_conservation:.2e} of sqrt(s), \
         |p^2 - m^2| <= {worst_mass:.2e} of s, mean XWGTUP matches sigma to {deviation:.1e}",
        row.dir,
        parsed.events.len(),
        text.len(),
    );
    // Both bounds are set by the momentum columns' eleven significant digits, not
    // by the generator: a component of order `sqrt(s)` loses its last digit to the
    // format, and an invariant formed from two of them loses one more.
    assert!(
        worst_conservation < 1e-9,
        "{}: momenta do not balance",
        row.dir
    );
    assert!(worst_mass < 1e-8, "{}: legs are not on shell", row.dir);
}
