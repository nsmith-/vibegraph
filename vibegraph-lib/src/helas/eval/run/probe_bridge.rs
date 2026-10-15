//! The per-diagram probes against the production evaluator.
//!
//! A per-diagram probe ([`eval_single_diagram`]) lowers one `DiagramEval` with the
//! test-only [`lower::lower`](super::super::lower::lower) and walks it with
//! [`run_forward_slot`](super::run_forward_slot). The production evaluator runs
//! `lower_flows` → `optimize` → `Folded` → the typed `Program` → [`fill_arenas`](super::fill_arenas),
//! over the helicity-expanded arena on the hot path. The two share the rooting and the
//! kernels, and nothing after that. This bridge compares them, as complex values, at
//! every helicity combination of generic on-shell points:
//!
//! - **per amplitude**: each `(diagram, colour chain)` amplitude the production
//!   program reads out (its configuration amplitudes, from both the helicity-expanded
//!   and the base program) times its symmetry factor and Fermi sign, against the
//!   probe of the same `DiagramEval`;
//! - **per flow**: each production JAMP (helicity-expanded and base) against the
//!   colour-weighted sum of the probes of the amplitudes the colour basis puts in it;
//! - **|M|²**: `eval_m2` against the CF contraction of the probe JAMPs.
//!
//! Both sides evaluate the same `DiagramSet` — one labelling of the external legs — and
//! the same `DiagramEval`s, so a convention that moves with the labelling moves both
//! sides together and cannot show here. The bridge is blind, by construction, to
//! anything the two paths share: rooting, the convention signs read at the anchor, the
//! external wavefunctions and the kernels themselves. MadGraph's per-diagram `AMP()`
//! is the oracle for those.
//!
//! Contact diagrams (a vertex of four or more legs) carry no configuration, so the
//! production program has no per-amplitude readout for them; they are compared only
//! inside the flow sums.

use std::collections::HashMap;
use std::sync::Arc;

use num_complex::Complex64;
use num_traits::ToPrimitive;
use rand::rngs::StdRng;
use rand::SeedableRng;

use super::super::compile::{root_amplitudes, Configurations};
use super::super::layout::Program;
use super::super::root_diagram::{compile_diagram_ast, DiagramEval};
use super::*;
use crate::diagrams::diagram::FermionLine;
use crate::diagrams::{generate_from_proc_card, parse_proc_card, DiagramSet, ParsingOptions};
use crate::helas::color::colorize::Contribution;
use crate::helas::color::{colorize_process, ColorBasis};
use crate::phasespace::rambo_massive;
use crate::ufo::sm::{sm_model, SMRestrict};
use crate::ufo::UFOModel;

/// Largest residual admitted at every level, relative to the largest probe amplitude
/// at the same helicity combination. The two paths associate the same products and
/// sums differently (common-subexpression elimination, n-ary sums, constant folding),
/// so they agree to rounding, not to the bit.
const REL_TOL: f64 = 1e-12;

/// The processes the bridge runs, chosen for the features named against each. Every
/// feature [`Features`] counts must be covered by at least one of them.
const PROCESSES: &[&str] = &[
    // Single flow, one crossed lepton line without a propagator.
    "e+ e- > mu+ mu-",
    // Initial-initial line through an internal electron.
    "e+ e- > a a",
    // Internal neutrino next to VVV-anchored s-channels.
    "e+ e- > w+ w-",
    // Crossed lepton line through an internal neutrino, a colourless vector at leg 0.
    "w+ w- > e+ e-",
    // Mixed (initial-final) line through an internal electron.
    "e- a > e- a",
    // Two flows; a crossed quark line through an internal quark; a VVV vertex.
    "g g > u u~",
    // Three flows, the four-gluon contact vertex with several colour chains.
    "g g > g g",
    // Identical fermions: two diagrams of opposite Fermi sign, two flows.
    "u u > u u",
    // The VVVV contact next to s- and t-channel boson exchanges and the Higgs.
    "w+ w- > w+ w-",
    // A massive crossed top line through FFV and FFS vertices.
    "z z > t t~",
    // Five legs, six flows, a massive top propagator and the four-gluon contact.
    "g g > t t~ g",
];

/// Structural features of the diagrams compared, counted over every process.
#[derive(Default, Debug)]
struct Features {
    /// Diagrams with a fermion line carrying an internal propagator.
    internal_fermion: usize,
    /// Diagrams with a crossed (final-final) fermion line carrying a propagator.
    crossed_line_with_propagator: usize,
    /// Diagrams with a mixed (initial-final) fermion line carrying a propagator.
    mixed_line_with_propagator: usize,
    /// Diagrams with a vertex of four or more legs.
    contact: usize,
    /// Processes with more than one colour flow.
    multi_flow: usize,
    /// Processes whose diagrams carry both relative Fermi signs.
    relative_fermi_sign: usize,
    /// Amplitudes with a colour chain other than all-zero.
    nonzero_chain: usize,
}

/// Largest residual met at each level, relative to the per-combination probe scale.
#[derive(Default, Debug, Clone, Copy)]
struct Residuals {
    amp_hel: f64,
    amp_base: f64,
    flow_hel: f64,
    flow_base: f64,
    m2: f64,
    /// Comparisons made at the per-amplitude levels and the per-flow levels.
    n_amp: usize,
    n_flow: usize,
}

fn generate(model: &Arc<UFOModel>, process: &str) -> DiagramSet {
    let card = parse_proc_card(&format!("generate {process}"), &ParsingOptions::default())
        .expect("parse the process");
    let mut sets = generate_from_proc_card(&card, model).expect("generate the process");
    assert_eq!(sets.len(), 1, "{process}: expected one subprocess");
    sets.pop().unwrap()
}

/// Two generic on-shell partonic-CM points per energy scale, at two scales: beams
/// along ±z, the outgoing legs from seeded massive RAMBO.
fn points(eval: &AmplitudeEvaluator, evaluated: &EvaluatedModel) -> Vec<Vec<LorentzVector<f64>>> {
    let masses: Vec<f64> = eval
        .external_particles()
        .iter()
        .map(|&pid| evaluated.mass(pid))
        .collect();
    let (m_in, m_out) = masses.split_at(eval.n_in());
    let threshold = m_in
        .iter()
        .sum::<f64>()
        .max(m_out.iter().sum::<f64>())
        .max(10.0);
    let mut rng = StdRng::seed_from_u64(0xB41D_6E5);
    let mut out = Vec::new();
    for scale in [2.9, 7.3] {
        let sqrt_s = scale * threshold;
        let s = sqrt_s * sqrt_s;
        let e1 = (s + m_in[0] * m_in[0] - m_in[1] * m_in[1]) / (2.0 * sqrt_s);
        let pz = (e1 * e1 - m_in[0] * m_in[0]).max(0.0).sqrt();
        for _ in 0..2 {
            let mut p = vec![
                LorentzVector::new(e1, 0.0, 0.0, pz),
                LorentzVector::new(sqrt_s - e1, 0.0, 0.0, -pz),
            ];
            p.extend(rambo_massive(sqrt_s, m_out, &mut rng));
            out.push(p);
        }
    }
    out
}

fn count_features(set: &DiagramSet, model: &UFOModel, basis: &ColorBasis, f: &mut Features) {
    let mut signs = [false; 2];
    for d in &set.diagrams {
        let lines = d.fermion_lines(model);
        let carries = |l: &FermionLine| l.vertices.len() > 1;
        let final_end = |l: &FermionLine, i: usize| l.legs[i].0 >= d.n_in;
        f.internal_fermion += usize::from(lines.iter().any(carries));
        f.crossed_line_with_propagator += usize::from(
            lines
                .iter()
                .any(|l| carries(l) && final_end(l, 0) && final_end(l, 1)),
        );
        f.mixed_line_with_propagator += usize::from(
            lines
                .iter()
                .any(|l| carries(l) && final_end(l, 0) != final_end(l, 1)),
        );
        f.contact += usize::from(d.vertices.iter().any(|v| v.rays.len() >= 4));
        signs[usize::from(d.sign > 0)] = true;
    }
    f.multi_flow += usize::from(basis.ncolor() > 1);
    f.relative_fermi_sign += usize::from(signs[0] && signs[1]);
    f.nonzero_chain += basis
        .elements
        .iter()
        .flat_map(|e| &e.contributions)
        .filter(|c| c.chain.iter().any(|&k| k != 0))
        .count();
}

/// A colour coefficient evaluated at `Nc = 3`, as the production lowering applies it.
fn colour_coeff(c: &Contribution) -> Complex64 {
    let q = c.coeff.eval_nc(3).to_f64().expect("rational in f64 range");
    if c.coeff.imag {
        Complex64::new(0.0, q)
    } else {
        Complex64::new(q, 0.0)
    }
}

/// One process, compiled once, with the probe side's view of the same diagrams.
struct Case<'a> {
    process: &'a str,
    set: DiagramSet,
    basis: ColorBasis,
    evals: HashMap<(usize, Vec<u8>), DiagramEval>,
    /// The `(diagram, chain)` keys of the configuration amplitudes, in the production
    /// readout order.
    config_amps: Vec<(usize, Vec<u8>)>,
    /// Every key of `evals`, sorted.
    keys: Vec<(usize, Vec<u8>)>,
    /// The diagrams' chain-zero rootings, which the colour-free per-diagram probes use.
    colour_free: Vec<DiagramEval>,
    /// The colour-factor matrix as `f64`, from the evaluator's exact rationals.
    cf: Vec<f64>,
}

impl<'a> Case<'a> {
    fn new(
        model: &Arc<UFOModel>,
        eval: &AmplitudeEvaluator,
        process: &'a str,
        set: DiagramSet,
        features: &mut Features,
    ) -> Self {
        let basis = colorize_process(model, &set.diagrams).expect("colour basis");
        let evals = root_amplitudes(&basis, &set, model).expect("root the amplitudes");
        let configs = Configurations::group(&set, model, &evals);
        let config_amps: Vec<(usize, Vec<u8>)> = configs.amps.into_iter().flatten().collect();
        count_features(&set, model, &basis, features);

        // The production readout order is the one the evaluator was compiled with.
        assert_eq!(basis.ncolor(), eval.n_flows(), "{process}: flow count");
        assert_eq!(
            config_amps.iter().map(|(d, _)| *d).collect::<Vec<_>>(),
            eval.config_amp_diagrams(),
            "{process}: configuration amplitude order"
        );
        let colour_free = compile_diagram_ast(&set, model).expect("colour-free rooting");
        let mut keys: Vec<_> = evals.keys().cloned().collect();
        keys.sort();
        let cf = eval
            .cf_matrix()
            .iter()
            .map(|r| r.to_f64().expect("rational in f64 range"))
            .collect();
        Case {
            process,
            set,
            basis,
            evals,
            config_amps,
            keys,
            colour_free,
            cf,
        }
    }

    /// The probe amplitude of every `(diagram, chain)` at one helicity combination.
    fn probe_amplitudes(
        &self,
        p: &[LorentzVector<f64>],
        hel: &[i32],
        evaluated: &EvaluatedModel,
    ) -> HashMap<&(usize, Vec<u8>), C<f64>> {
        self.keys
            .iter()
            .map(|k| (k, eval_single_diagram(&self.evals[k], p, hel, evaluated)))
            .collect()
    }

    /// The colour-free probe of each diagram equals its chain-zero amplitude to the bit.
    fn check_colour_free(
        &self,
        probe: &HashMap<&(usize, Vec<u8>), C<f64>>,
        p: &[LorentzVector<f64>],
        hel: &[i32],
        evaluated: &EvaluatedModel,
    ) {
        for (d, diagram) in self.colour_free.iter().enumerate() {
            let key = (d, vec![0u8; self.set.diagrams[d].vertices.len()]);
            if let Some(&want) = probe.get(&key) {
                let got = eval_single_diagram(diagram, p, hel, evaluated);
                assert!(
                    got == want,
                    "{}: diagram {d} hel {hel:?}: colour-free probe {got:e} differs from \
                     its chain-zero amplitude {want:e}",
                    self.process
                );
            }
        }
    }

    /// Each flow's probe JAMP: the colour-weighted sum of its contributions' probes.
    fn probe_jamps(&self, probe: &HashMap<&(usize, Vec<u8>), C<f64>>) -> Vec<C<f64>> {
        self.basis
            .elements
            .iter()
            .map(|elem| {
                elem.contributions
                    .iter()
                    .map(|c| colour_coeff(c) * probe[&(c.diagram, c.chain.clone())])
                    .sum()
            })
            .collect()
    }

    /// The terms `Σ_j J_j CF_ji conj(J_i)`, one per flow `i`, whose sum is the
    /// colour-summed |M|² of one helicity combination.
    fn contract<'b>(&'b self, jamps: &'b [C<f64>]) -> impl Iterator<Item = f64> + 'b {
        let n = jamps.len();
        (0..n).map(move |i| {
            let ztemp: C<f64> = (0..n).map(|j| jamps[j] * self.cf[j * n + i]).sum();
            (ztemp * jamps[i].conj()).re
        })
    }
}

/// The helicity-expanded program's configuration amplitudes, one row per helicity
/// combination, read off the arenas of the last `fill_for` pass.
fn expanded_amp_rows(
    bound: &BoundAmplitude<'_, f64>,
    program: &Program,
    scratch: &ScratchSpace<f64>,
) -> Vec<Vec<C<f64>>> {
    let n = (program.n_amps as usize).max(1);
    program
        .amp_locs
        .chunks_exact(n)
        .zip(program.amp_weights.chunks_exact(n))
        .map(|(locs, ws)| {
            locs.iter()
                .zip(ws)
                .map(|(&l, &w)| bound.weighted(w, scratch.scalars[l as usize]))
                .collect()
        })
        .collect()
}

/// Assert `got` agrees with `want` to [`REL_TOL`] of `scale`, recording the residual.
fn check_level(what: &str, level: &str, got: C<f64>, want: C<f64>, scale: f64, worst: &mut f64) {
    let r = (got - want).norm() / scale;
    *worst = worst.max(r);
    assert!(
        r <= REL_TOL,
        "{what}: {level} {got:e} vs probe {want:e} (rel {r:.2e})"
    );
}

/// Run the bridge on one process, asserting every level and returning the largest
/// residuals met.
fn bridge(
    model: &Arc<UFOModel>,
    evaluated: &EvaluatedModel,
    process: &str,
    features: &mut Features,
) -> Residuals {
    let set = generate(model, process);
    let eval = AmplitudeEvaluator::compile(&set, model).expect("compile");
    let case = Case::new(model, &eval, process, set, features);
    let bound = BoundAmplitude::<f64>::bind(&eval, evaluated);

    let n_flows = eval.n_flows();
    let helicities = eval.helicities();
    let folded_hel = eval.folded_hel();
    let program_hel = folded_hel.program();
    let n_amps = program_hel.n_amps as usize;
    assert_eq!(
        n_amps,
        case.config_amps.len(),
        "{process}: amplitude row length"
    );
    assert_eq!(
        program_hel.amp_locs.len(),
        helicities.len() * n_amps,
        "{process}: one amplitude row per helicity combination"
    );

    let pts = points(&eval, evaluated);
    let mut res = Residuals::default();
    let mut scratch = bound.scratch_space();
    let mut hel_jamps = Vec::new();
    for p in &pts {
        bound.eval_hel_jamps(p, &mut scratch, &mut hel_jamps);
        assert_eq!(
            hel_jamps.len(),
            helicities.len() * n_flows,
            "{process}: one JAMP row per helicity combination"
        );
        // `eval_hel_jamps` leaves the arenas filled; the pass refills them identically.
        bound.fill_for(folded_hel, p, &mut scratch);
        let amp_rows = expanded_amp_rows(&bound, program_hel, &scratch);
        assert_eq!(
            amp_rows.len(),
            helicities.len(),
            "{process}: amplitude rows"
        );

        let mut m2_probe = 0.0;
        for (h, hel) in helicities.iter().enumerate() {
            let probe = case.probe_amplitudes(p, hel, evaluated);
            // A diagram scale, not the flow's value, so a cancelling flow is not lost. A
            // helicity-forbidden combination (every probe zero) is held to `REL_TOL`
            // absolutely.
            let scale = match probe.values().map(|a| a.norm()).fold(0.0, f64::max) {
                s if s > 0.0 => s,
                _ => 1.0,
            };
            case.check_colour_free(&probe, p, hel, evaluated);

            let base_amps = bound.run_config_amps(p, hel, &mut scratch);
            for (i, key) in case.config_amps.iter().enumerate() {
                let e = &case.evals[key];
                let sf = e.symmetry_factor * f64::from(e.fermi_sign);
                let what = format!("{process}: amplitude {key:?} hel {hel:?}");
                for (level, got, worst) in [
                    (
                        "helicity-expanded program",
                        amp_rows[h][i],
                        &mut res.amp_hel,
                    ),
                    ("base program", base_amps[i], &mut res.amp_base),
                ] {
                    check_level(&what, level, got * sf, probe[key], scale, worst);
                    res.n_amp += 1;
                }
            }

            let probe_jamps = case.probe_jamps(&probe);
            let base_jamps = if n_flows == 1 {
                vec![bound.eval_amplitude(p, hel, &mut scratch)]
            } else {
                bound.run_flows(p, hel, &mut scratch)
            };
            for (f, &want) in probe_jamps.iter().enumerate() {
                let what = format!("{process}: flow {f} hel {hel:?}");
                for (level, got, worst) in [
                    (
                        "helicity-expanded JAMP",
                        hel_jamps[h * n_flows + f],
                        &mut res.flow_hel,
                    ),
                    ("base JAMP", base_jamps[f], &mut res.flow_base),
                ] {
                    check_level(&what, level, got, want, scale, worst);
                    res.n_flow += 1;
                }
            }
            for term in case.contract(&probe_jamps) {
                m2_probe += term;
            }
        }
        let m2 = bound.eval_m2(p, &mut scratch);
        assert!(m2_probe > 0.0, "{process}: probe |M|² {m2_probe:e}");
        let r = (m2 - m2_probe).abs() / m2_probe;
        res.m2 = res.m2.max(r);
        assert!(
            r <= REL_TOL,
            "{process}: eval_m2 {m2:e} vs probe contraction {m2_probe:e} (rel {r:.2e})"
        );
    }
    // Every comparison the loops should make was made: a truncated loop is not green.
    assert_eq!(
        res.n_amp,
        2 * pts.len() * helicities.len() * n_amps,
        "{process}: amplitude comparisons"
    );
    assert_eq!(
        res.n_flow,
        2 * pts.len() * helicities.len() * n_flows,
        "{process}: flow comparisons"
    );
    res
}

/// The per-diagram probes equal the production evaluator, per amplitude, per flow and
/// per helicity, as complex values, on processes covering internal, crossed and mixed
/// fermion lines, several colour flows, contact vertices and relative Fermi signs.
#[test]
fn per_diagram_probes_match_the_production_program() {
    let model = sm_model(SMRestrict::Default);
    let evaluated = EvaluatedModel::from_model(model.clone());
    let mut features = Features::default();
    println!(
        "{:<16} {:>9} {:>9} {:>9} {:>9} {:>9} {:>6} {:>6}",
        "process", "amp_hel", "amp_base", "flow_hel", "flow_base", "m2", "n_amp", "n_flow"
    );
    for process in PROCESSES {
        let r = bridge(&model, &evaluated, process, &mut features);
        println!(
            "{process:<16} {:>9.1e} {:>9.1e} {:>9.1e} {:>9.1e} {:>9.1e} {:>6} {:>6}",
            r.amp_hel, r.amp_base, r.flow_hel, r.flow_base, r.m2, r.n_amp, r.n_flow
        );
        assert!(r.n_flow > 0, "{process}: no flow compared");
    }
    println!("{features:?}");
    let Features {
        internal_fermion,
        crossed_line_with_propagator,
        mixed_line_with_propagator,
        contact,
        multi_flow,
        relative_fermi_sign,
        nonzero_chain,
    } = features;
    for (name, n) in [
        ("internal fermion", internal_fermion),
        (
            "crossed line with a propagator",
            crossed_line_with_propagator,
        ),
        ("mixed line with a propagator", mixed_line_with_propagator),
        ("contact vertex", contact),
        ("multi-flow process", multi_flow),
        ("relative Fermi sign", relative_fermi_sign),
        ("nonzero colour chain", nonzero_chain),
    ] {
        assert!(n > 0, "no compared diagram covers: {name}");
    }
}
