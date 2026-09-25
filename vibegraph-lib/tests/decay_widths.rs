//! Partial widths of `1 → n` decay processes, integrated through the same
//! fixed-initial-state integrand a scattering process uses, with the decaying
//! particle at rest ([`DecayAtRest`]).
//!
//! The oracle here is the SM UFO's own `decays.py`: the analytic two-body
//! partial widths MadGraph's `compute_widths` evaluates, evaluated by MadGraph's
//! `model_reader` on the default restriction's parameters and committed as
//! `validation/madgraph/sm_decay_widths.json`
//! (`validation/madgraph/dump_sm_decay_widths.py`). A two-body width is a
//! constant `|M|²` over a two-body phase space, so this pins the flux `1/(2M)`,
//! the spin×colour average of the decaying particle, the final-state colour sum
//! and the phase-space volume exactly — and cannot see anything that varies over
//! the phase space. The many-body widths are compared against MadEvent's own
//! integration in `vibegraph-cli/tests/cli_decay.rs`.

mod common;

use std::sync::Arc;

use vibegraph::cuts::Cuts;
use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
use vibegraph::hadronic::{
    compile_subprocesses, initial_spin_color_average, process_external_legs, DecayAtRest,
    FixedBeamIntegrand, Observable,
};
use vibegraph::helas::eval::BoundAmplitude;
use vibegraph::helas::repr::lorentz::LorentzVector;
use vibegraph::runcard::RunCard;
use vibegraph::ufo::{EvaluatedModel, UFOModel};

type V = LorentzVector<f64>;

/// One decay process compiled against the default-restricted SM, as `vibegraph
/// integrate` compiles it.
struct Decay {
    model: Arc<UFOModel>,
    evaluated: EvaluatedModel,
    evals: Vec<vibegraph::helas::eval::AmplitudeEvaluator>,
    diagrams: Vec<vibegraph::diagrams::Diagram>,
}

impl Decay {
    fn new(process: &str) -> Self {
        let model = common::sm_model();
        let evaluated = EvaluatedModel::from_model(model.clone());
        let card =
            parse_proc_card(&format!("generate {process}"), &ParsingOptions::default()).unwrap();
        let sets = generate_from_proc_card(&card, &model).unwrap();
        let evals = compile_subprocesses(&sets, &model, &evaluated).unwrap();
        let diagrams = sets.iter().flat_map(|s| s.diagrams.clone()).collect();
        Decay {
            model,
            evaluated,
            evals,
            diagrams,
        }
    }

    /// `(Γ, ΔΓ, χ²/dof)` in GeV under `card`, over `niter` iterations of `neval`
    /// points after an α-adaptation survey, as the CLI integrates.
    fn width(&self, card: &RunCard, neval: usize, niter: usize, seed: u64) -> (f64, f64, f64) {
        let rep = &self.evals[0];
        let legs = process_external_legs(rep, &self.model, &self.evaluated);
        let cuts = Cuts::compile(card, &legs).unwrap();
        let bounds: Vec<_> = self
            .evals
            .iter()
            .map(|e| BoundAmplitude::<f64>::bind(e, &self.evaluated))
            .collect();
        let final_masses: Vec<f64> = rep.external_particles()[1..]
            .iter()
            .map(|&id| self.evaluated.mass(id))
            .collect();
        let avg = initial_spin_color_average(rep, &self.model, &self.evaluated);
        let mut integ = FixedBeamIntegrand::new(
            bounds.iter().collect(),
            &cuts,
            DecayAtRest::from_legs(&legs).unwrap(),
            final_masses,
            avg,
        );
        assert_eq!(integ.observable(), Observable::PartialWidth);
        integ
            .use_running_coupling(&self.diagrams, &self.model, &self.evaluated, card)
            .unwrap();
        integ.use_multichannel(&self.diagrams, &self.evaluated, neval.min(40_000), 6, seed);
        let (_, result) = integ.adapt_grids(neval, niter, seed);
        (result.integral, result.std_dev, result.chi2_per_dof)
    }
}

#[derive(serde::Deserialize)]
struct TwoBody {
    parent: String,
    daughters: [String; 2],
    width_gev: f64,
}

/// Every kinematically open two-body channel of the SM's `decays.py`, as
/// MadGraph evaluates it on `restrict_default.dat`.
fn two_body_widths() -> Vec<TwoBody> {
    #[derive(serde::Deserialize)]
    struct File {
        widths: Vec<TwoBody>,
    }
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../validation/madgraph/sm_decay_widths.json"
    );
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let file: File = serde_json::from_str(&text).expect("sm_decay_widths.json parses");
    assert!(
        file.widths.len() >= 10,
        "the oracle lists every open channel"
    );
    file.widths
}

impl TwoBody {
    fn process(&self) -> String {
        format!(
            "{} > {} {}",
            self.parent, self.daughters[0], self.daughters[1]
        )
    }
}

/// Every open two-body channel of `decays.py` — a massive fermion (`t > W+ b`,
/// spin average 1/2, colour 1/3), massive vectors into leptons, neutrinos and
/// coloured quark pairs (`Z`, `W+`: average 1/3, final colour sum 3), and a
/// scalar into massive fermions (`H > b b~`, `H > ta- ta+`).
///
/// The two-body phase space is flat in the decay angle and the spin-summed
/// `|M|²` of an unpolarised decay is isotropic, so the integrand is a constant,
/// and one VEGAS iteration on its initial uniform grid averages that constant to
/// rounding: the gate is at `1e-10`, which a wrong flux, average, colour sum,
/// `(2π)` power or phase-space volume misses by orders of magnitude. One
/// iteration, because a refined grid is not uniform any more — its bins follow
/// the sampling noise of the first iteration — and a constant times a
/// non-uniform Jacobian has variance again, unbiased but no longer exact. What
/// this cannot see is any angular dependence, which is what the many-body rows
/// against MadEvent are for.
#[test]
fn two_body_widths_match_the_models_analytic_decays() {
    for channel in two_body_widths() {
        let process = channel.process();
        let decay = Decay::new(&process);
        let (width, err, _) = decay.width(&RunCard::decay_default(), 20_000, 1, 0xDECA_1);
        let rel = (width / channel.width_gev - 1.0).abs();
        println!(
            "{process:<14} Γ = {width:.10e} ± {err:.1e} GeV, decays.py {:.10e} (rel {rel:.1e})",
            channel.width_gev
        );
        assert!(
            rel < 1e-10,
            "{process}: Γ = {width} against decays.py {}",
            channel.width_gev
        );
    }
}

/// The same channels one level finer: `Σ_hel |M|²` at fixed rest-frame points,
/// against the analytic width inverted through the two-body phase space,
/// `Σ|M|² = Γ · 8π M² (2s+1) N_c / |p*|`.
///
/// Several orientations, because an unpolarised decay's summed `|M|²` does not
/// depend on the direction — a spurious one (an incoming wavefunction built
/// along a fixed axis, say) shows up as a spread here before it shows up in a
/// width. A two-body decay has one diagram and one colour flow, so this is the
/// finest level a squared amplitude offers; the amplitude's own phase is
/// unobservable.
#[test]
fn two_body_squared_amplitudes_match_the_models_analytic_decays() {
    for channel in two_body_widths() {
        let process = channel.process();
        let decay = Decay::new(&process);
        let eval = &decay.evals[0];
        let masses: Vec<f64> = eval
            .external_particles()
            .iter()
            .map(|&id| decay.evaluated.mass(id))
            .collect();
        let (m, m1, m2) = (masses[0], masses[1], masses[2]);
        let lambda = (m * m - (m1 + m2).powi(2)) * (m * m - (m1 - m2).powi(2));
        let p_star = lambda.sqrt() / (2.0 * m);
        let states = 1.0 / initial_spin_color_average(eval, &decay.model, &decay.evaluated);
        let expected = channel.width_gev * 8.0 * std::f64::consts::PI * m * m * states / p_star;

        let bound = BoundAmplitude::<f64>::bind(eval, &decay.evaluated);
        let mut scratch = bound.scratch_space();
        for (cos_theta, phi) in [(0.3f64, 0.7f64), (-0.91, 2.2), (0.999, 4.0), (0.0, 0.0)] {
            let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
            let dir = [sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta];
            let e1 = (m * m + m1 * m1 - m2 * m2) / (2.0 * m);
            let p = [
                V::new(m, 0.0, 0.0, 0.0),
                V::new(e1, p_star * dir[0], p_star * dir[1], p_star * dir[2]),
                V::new(m - e1, -p_star * dir[0], -p_star * dir[1], -p_star * dir[2]),
            ];
            let m2_sum = bound.eval_m2(&p, &mut scratch);
            let rel = (m2_sum / expected - 1.0).abs();
            assert!(
                rel < 1e-12,
                "{process} at cosθ = {cos_theta}, φ = {phi}: Σ|M|² = {m2_sum}, analytic {expected}"
            );
        }
    }
}

/// The many-body decays one level below their widths: `Σ|M|²` at fixed
/// rest-frame points against MadGraph's standalone `SMATRIX` for the same
/// process (`validation/madgraph/decay_amplitudes.json`,
/// `gen_decay_amplitudes.py`), on and off the resonances the diagrams carry.
///
/// `SMATRIX` divides by `IDEN`, the mother's spin and colour states times the
/// final state's identical-particle factor, which is exactly the factor this
/// crate's integrand multiplies `Σ|M|²` by: the comparison is of that product.
/// `z > e+ e- mu+ mu-` (eight diagrams, photon and Z exchange, a massive
/// vector mother) and `t > b e+ ve a` (four diagrams, the photon off every
/// charged line) are where interference would show a relative sign or a
/// mother-wavefunction convention the one-diagram rows cannot. What it cannot
/// see is the amplitude's overall phase, and anything past `|M|²`.
#[test]
fn many_body_squared_amplitudes_match_madgraph_standalone() {
    #[derive(serde::Deserialize)]
    struct Case {
        process: String,
        points: Vec<Vec<[f64; 4]>>,
        smatrix: Vec<f64>,
    }
    #[derive(serde::Deserialize)]
    struct File {
        cases: Vec<Case>,
    }
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../validation/madgraph/decay_amplitudes.json"
    );
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let file: File = serde_json::from_str(&text).expect("decay_amplitudes.json parses");
    for case in &file.cases {
        let decay = Decay::new(&case.process);
        let eval = &decay.evals[0];
        let avg = initial_spin_color_average(eval, &decay.model, &decay.evaluated);
        let bound = BoundAmplitude::<f64>::bind(eval, &decay.evaluated);
        let mut scratch = bound.scratch_space();
        let mut worst = 0.0f64;
        for (point, &expected) in case.points.iter().zip(&case.smatrix) {
            let p: Vec<V> = point
                .iter()
                .map(|&[e, x, y, z]| V::new(e, x, y, z))
                .collect();
            let ours = avg * bound.eval_m2(&p, &mut scratch);
            let rel = (ours / expected - 1.0).abs();
            worst = worst.max(rel);
            assert!(
                rel < 1e-9,
                "{}: Σ|M|²/IDEN = {ours:e}, MadGraph {expected:e} at {point:?}",
                case.process
            );
        }
        println!(
            "{:<18} {} points, worst relative difference {worst:.1e}",
            case.process,
            case.points.len()
        );
    }
}

/// Budget ladder behind the `h > e+ e- mu+ mu-` width row, and an unmapped
/// control: the resonance-aware map against VEGAS over flat RAMBO, which shapes
/// nothing and so cannot mis-cover the region where the first-drawn `Z` is the
/// off-shell one.
///
/// The row's one channel draws the `e+ e-` pair's invariant first, over its full
/// range, and the `mu+ mu-` pair's against what is left; with `M_H < 2 M_Z` one
/// pair is always off shell, and when it is the first-drawn one the peak the
/// integrand then has is the second's, reached only through the tail of the
/// first's Breit–Wigner. What decides whether that costs a bias or only variance
/// is the seed scatter against the quoted error at each rung, so each rung
/// prints both. The flat control scatters far wider than it quotes — narrow
/// resonances in a flat map — and reads as an oracle only where its seeds agree.
///
/// Too long for the default suite; run with
/// `cargo test -p vibegraph-lib --test decay_widths -- --ignored --nocapture`.
#[test]
#[ignore = "budget ladder, minutes"]
fn probe_h4l_width_ladder() {
    let decay = Decay::new("h > e+ e- mu+ mu-");
    let card = RunCard::decay_default();
    for (neval, niter, seeds) in [(120_000usize, 10usize, 10u64), (480_000, 20, 5)] {
        let runs: Vec<(f64, f64, f64)> = (0..seeds)
            .map(|s| decay.width(&card, neval, niter, 0xA11CE + s))
            .collect();
        report_ladder_rung(&format!("mapped {neval}×{niter}"), &runs);
    }
    let rep = &decay.evals[0];
    let legs = process_external_legs(rep, &decay.model, &decay.evaluated);
    let cuts = Cuts::compile(&card, &legs).unwrap();
    let bounds: Vec<_> = decay
        .evals
        .iter()
        .map(|e| BoundAmplitude::<f64>::bind(e, &decay.evaluated))
        .collect();
    let final_masses: Vec<f64> = rep.external_particles()[1..]
        .iter()
        .map(|&id| decay.evaluated.mass(id))
        .collect();
    let avg = initial_spin_color_average(rep, &decay.model, &decay.evaluated);
    let mut flat = FixedBeamIntegrand::new(
        bounds.iter().collect(),
        &cuts,
        DecayAtRest::from_legs(&legs).unwrap(),
        final_masses,
        avg,
    );
    flat.use_running_coupling(&decay.diagrams, &decay.model, &decay.evaluated, &card)
        .unwrap();
    let runs: Vec<(f64, f64, f64)> = (0..5u64)
        .map(|s| {
            let (_, r) = flat.adapt_grids(1_000_000, 20, 0xF1A7 + s);
            (r.integral, r.std_dev, r.chi2_per_dof)
        })
        .collect();
    report_ladder_rung("flat RAMBO 1000000×20", &runs);
}

fn report_ladder_rung(label: &str, runs: &[(f64, f64, f64)]) {
    let n = runs.len() as f64;
    let mean = runs.iter().map(|r| r.0).sum::<f64>() / n;
    let spread = (runs.iter().map(|r| (r.0 - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
    let quoted = runs.iter().map(|r| r.1).sum::<f64>() / n;
    println!(
        "{label}: mean {mean:.6e} ± {:.2e} (seed spread {spread:.2e}, mean quoted {quoted:.2e}) \
         over {} seeds; values {:?}",
        spread / n.sqrt(),
        runs.len(),
        runs.iter().map(|r| r.0).collect::<Vec<_>>()
    );
}
