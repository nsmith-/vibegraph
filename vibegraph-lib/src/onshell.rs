//! Forbidden on-shell s-channels (`$ A`) as MadEvent integrates them.
//!
//! `$ A` keeps every diagram. `diagram_generation.py:781` marks each s-channel
//! line of `A` (named as it flows towards the final state) `onshell = False`,
//! and MadGraph then acts on the mark twice:
//!
//! 1. **In the amplitude.** `helas_call_writers.py:1184` gives a marked
//!    propagator's wavefunction ALOHA's `P1D` form ("D is for $ syntax ->
//!    offshell propagator only"), which multiplies the propagator by
//!    `THETA_FUNCTIONR((p² − (M − c·Γ)²)(p² − (M + c·Γ)²) ≥ 0)` with
//!    `c = bwcutoff` and `Γ` the fake width `fk_W = max(|W|, |M ·
//!    small_width_treatment|)` (`export_v4.py:4820`; zero where `W` is zero). The
//!    line is removed from every diagram carrying it wherever `√p²` is strictly
//!    inside `(M − c·Γ, M + c·Γ)` (`(c·Γ − M, M + c·Γ)` once `c·Γ > M`, where the
//!    lower root of the theta's argument turns over), whatever `Γ/M`.
//! 2. **In the phase space.** `export_v4.py:5879` writes the mark into
//!    `decayBW.inc` as `gForceBW = 2`, and `cut_bw` (`myamp.f:142`) rejects a point
//!    in a configuration whose own marked line is on the same window, at
//!    `sde_strat = 1` and `Γ/M < 0.1`. That configuration's `AMP2` is already zero
//!    there, since its diagrams carry the zeroed line, so its single-diagram-enhanced
//!    share `AMP2_c / Σ AMP2` was zero anyway: the configurations that remain carry
//!    the whole matrix element, and the rejection moves no cross section.
//!
//! The integral MadEvent reports is therefore that of `|M'|²`, the matrix element
//! with every marked line zeroed on its window. Here `|M'|²` is evaluated exactly:
//! at a point the marked lines on their windows form a *pattern*, and the
//! amplitude of a pattern is compiled from the diagrams that carry none of its
//! lines. The configuration a point's scale is clustered in, and the one its
//! colour flow is drawn in, follow `|M'|²`'s own `AMP2`: `AMP2` with the
//! configurations that carry a zeroed line removed.
//!
//! `$` is a property of the process line and MadGraph marks each subprocess's
//! own diagrams, so each subprocess gets its own veto; flavour groups sharing one
//! matrix element are required to share the marking too ([`check_group_members`]).

use std::collections::BTreeMap;

use thiserror::Error;

use crate::diagrams::diagram::{Diagram, Prop};
use crate::diagrams::schannel::oriented_s_channel_id;
use crate::diagrams::DiagramSet;
use crate::hadronic::{compile_class, HadronicError};
use crate::helas::eval::AmplitudeEvaluator;
use crate::helas::repr::lorentz::LorentzVector;
use crate::proton::FlavorGroups;
use crate::runcard::RunCard;
use crate::ufo::{EvaluatedModel, UFOModel};

type V = LorentzVector<f64>;

/// The most distinct marked lines one subprocess may carry: every subset of them
/// is compiled into an amplitude of its own.
pub const MAX_MARKED_LINES: usize = 6;

#[derive(Debug, Error)]
pub enum OnShellVetoError {
    #[error(
        "'{process}' carries {lines} distinct propagators the '$' restriction marks; each \
         subset of them needs an amplitude of its own, and more than {MAX_MARKED_LINES} are \
         not supported"
    )]
    TooManyLines { process: String, lines: usize },
    #[error("compiling the amplitude with the marked propagators removed: {0}")]
    Compile(#[from] HadronicError),
    /// A colour flow of an amplitude with diagrams removed that the whole
    /// amplitude's basis does not carry exactly once, so an event drawn in it has
    /// no label in the record's flow table.
    #[error(
        "'{process}': a colour flow of the amplitude with marked propagators removed is {why}"
    )]
    FlowMap { process: String, why: &'static str },
    /// Two subprocesses share one compiled matrix element but not the same
    /// marked propagators, so no single set of amplitudes describes both.
    #[error(
        "'{a}' and '{b}' share a matrix element but differ in which propagators the '$' \
         restriction marks"
    )]
    GroupMarkingDiffers { a: String, b: String },
}

/// The run card's window for the marked propagators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BwWindow {
    /// `bwcutoff`: the window's half-width in units of the width.
    pub bwcutoff: f64,
    /// `small_width_treatment`: the floor on `|Γ/M|`.
    pub small_width_treatment: f64,
}

impl BwWindow {
    pub fn from_run_card(card: &RunCard) -> Self {
        BwWindow {
            bwcutoff: card.float("bwcutoff"),
            small_width_treatment: card.float("small_width_treatment"),
        }
    }

    /// `fk_W`: the width a propagator is evaluated with, floored at
    /// `|M · small_width_treatment|` unless it is zero.
    pub fn fake_width(&self, mass: f64, width: f64) -> f64 {
        if width == 0.0 {
            return 0.0;
        }
        width.abs().max((mass * self.small_width_treatment).abs())
    }

    /// Whether a marked line of pole mass `mass` and width `width` is zeroed at
    /// virtuality `p2`: where the `P1D` propagator's theta function reads `0`.
    pub fn zeroes(&self, p2: f64, mass: f64, width: f64) -> bool {
        let half = self.bwcutoff * self.fake_width(mass, width);
        (p2 - (mass - half).powi(2)) * (p2 - (mass + half).powi(2)) < 0.0
    }
}

/// One marked line: the outgoing legs whose momenta sum to its momentum, and
/// the pole it is zeroed around.
#[derive(Debug, Clone, PartialEq)]
struct MarkedLine {
    legs: Vec<usize>,
    mass: f64,
    width: f64,
}

/// The external legs, all outgoing, whose momenta sum to an s-channel line's.
///
/// A line's momentum is fixed only up to `Σ p_in − Σ p_out`, and the two
/// representatives carry the incoming legs or not. The outgoing legs on the
/// side of the line away from the initial state are the one form independent of
/// that choice and of the sign convention of the coefficients: the legs with a
/// nonzero coefficient where no incoming leg has one, and the others otherwise.
fn final_side_legs(momentum: &[i8], n_in: usize) -> Vec<usize> {
    let carries_initial = momentum[..n_in].iter().any(|&c| c != 0);
    (n_in..momentum.len())
        .filter(|&i| (momentum[i] != 0) != carries_initial)
        .collect()
}

/// The marked s-channel lines of `diagram`: those whose oriented id is in
/// `forbidden`.
fn marked_lines<'d>(
    diagram: &'d Diagram,
    forbidden: &'d [i64],
    model: &'d UFOModel,
) -> impl Iterator<Item = &'d Prop> + 'd {
    diagram.props.iter().filter(move |prop| {
        oriented_s_channel_id(prop, diagram, model).is_some_and(|id| forbidden.contains(&id))
    })
}

/// The veto of one subprocess: its distinct marked lines, and for every nonempty
/// subset of them the amplitude over the diagrams carrying none.
pub struct OnShellVeto {
    window: BwWindow,
    lines: Vec<MarkedLine>,
    /// Per configuration of the whole amplitude, the lines its representative
    /// diagram carries, as a bit set over `lines`.
    config_lines: Vec<u32>,
    /// Indexed by pattern (a bit set over `lines`; entry `0` is unused): the
    /// amplitude over the diagrams carrying none of the pattern's lines, `None`
    /// where no diagram is left, with its colour flows' indices in the whole
    /// amplitude's basis.
    patterns: Vec<Option<(AmplitudeEvaluator, Vec<usize>)>>,
}

impl std::fmt::Debug for OnShellVeto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OnShellVeto")
            .field("lines", &self.lines)
            .field("config_lines", &self.config_lines)
            .finish_non_exhaustive()
    }
}

impl OnShellVeto {
    /// The veto `forbidden` puts on `eval`, compiled from `set`, or `None` when
    /// no diagram carries a marked line.
    pub fn new(
        set: &DiagramSet,
        eval: &AmplitudeEvaluator,
        forbidden: &[i64],
        evaluated: &EvaluatedModel,
        window: BwWindow,
    ) -> Result<Option<Self>, OnShellVetoError> {
        if forbidden.is_empty() {
            return Ok(None);
        }
        let model = evaluated.model();
        let mut lines: Vec<MarkedLine> = Vec::new();
        let mut diagram_lines = Vec::with_capacity(set.diagrams.len());
        for d in &set.diagrams {
            let mut bits = 0u64;
            for prop in marked_lines(d, forbidden, model) {
                let line = MarkedLine {
                    legs: final_side_legs(&prop.momentum, d.n_in),
                    mass: evaluated.mass(prop.particle),
                    width: evaluated.width(prop.particle),
                };
                let i = lines.iter().position(|l| *l == line).unwrap_or_else(|| {
                    lines.push(line);
                    lines.len() - 1
                });
                bits |= 1u64.checked_shl(i as u32).unwrap_or(0);
            }
            diagram_lines.push(bits);
        }
        if lines.is_empty() {
            return Ok(None);
        }
        if lines.len() > MAX_MARKED_LINES {
            return Err(OnShellVetoError::TooManyLines {
                process: set.label(),
                lines: lines.len(),
            });
        }
        let diagram_lines: Vec<u32> = diagram_lines.into_iter().map(|b| b as u32).collect();

        let mut start = 0;
        let config_lines = eval
            .config_amp_counts()
            .iter()
            .map(|&span| {
                let representative = eval.config_amp_diagrams()[start];
                start += span;
                diagram_lines[representative]
            })
            .collect();

        let full_flows = eval.color_flow_tags();
        let mut patterns = vec![None];
        for pattern in 1u32..(1 << lines.len()) {
            let kept: Vec<Diagram> = set
                .diagrams
                .iter()
                .zip(&diagram_lines)
                .filter(|(_, &bits)| bits & pattern == 0)
                .map(|(d, _)| d.clone())
                .collect();
            if kept.is_empty() {
                patterns.push(None);
                continue;
            }
            let subset = DiagramSet {
                particles_in: set.particles_in.clone(),
                particles_out: set.particles_out.clone(),
                polarizations: set.polarizations.clone(),
                diagrams: kept,
            };
            let part = compile_class(&subset, model, evaluated)?;
            let tags = part.color_flow_tags();
            let flow_map = (0..tags.n_flows())
                .map(|f| {
                    let mut hits =
                        (0..full_flows.n_flows()).filter(|&g| full_flows.flow(g) == tags.flow(f));
                    match (hits.next(), hits.next()) {
                        (Some(g), None) => Ok(g),
                        (None, _) => Err("absent from the whole amplitude's basis"),
                        (Some(_), Some(_)) => Err("ambiguous in the whole amplitude's basis"),
                    }
                })
                .collect::<Result<Vec<usize>, _>>()
                .map_err(|why| OnShellVetoError::FlowMap {
                    process: set.label(),
                    why,
                })?;
            patterns.push(Some((part, flow_map)));
        }
        Ok(Some(OnShellVeto {
            window,
            lines,
            config_lines,
            patterns,
        }))
    }

    /// The number of configurations of the whole amplitude.
    pub fn n_configs(&self) -> usize {
        self.config_lines.len()
    }

    /// The number of distinct marked lines.
    pub fn n_lines(&self) -> usize {
        self.lines.len()
    }

    /// The marked lines zeroed at the external momenta `momenta` (incoming first,
    /// any frame), as a bit set; `0` where every marked line is off its window.
    pub fn pattern(&self, momenta: &[V]) -> u32 {
        let mut pattern = 0;
        for (i, line) in self.lines.iter().enumerate() {
            let mut p = [0.0; 4];
            for &leg in &line.legs {
                let k = momenta[leg];
                p[0] += k.e();
                p[1] += k.px();
                p[2] += k.py();
                p[3] += k.pz();
            }
            let p2 = p[0] * p[0] - p[1] * p[1] - p[2] * p[2] - p[3] * p[3];
            if self.window.zeroes(p2, line.mass, line.width) {
                pattern |= 1 << i;
            }
        }
        pattern
    }

    /// The amplitude of a nonzero `pattern`, with its flows' indices in the
    /// whole amplitude's basis, or `None` where no diagram survives it.
    ///
    /// # Panics
    ///
    /// If `pattern` is zero or not a pattern of this veto.
    pub fn amplitude(&self, pattern: u32) -> Option<(&AmplitudeEvaluator, &[usize])> {
        assert!(pattern != 0, "pattern 0 is the whole amplitude");
        self.patterns[pattern as usize]
            .as_ref()
            .map(|(eval, flows)| (eval, flows.as_slice()))
    }

    /// Every pattern's amplitude, `None` where no diagram survives; index `0` is
    /// the whole amplitude, which this veto does not hold, and is `None`.
    pub fn amplitudes(&self) -> impl Iterator<Item = Option<&AmplitudeEvaluator>> {
        self.patterns
            .iter()
            .map(|p| p.as_ref().map(|(eval, _)| eval))
    }

    /// Zero the entries of `weights` (one per configuration of the whole
    /// amplitude) whose configuration carries a line of `pattern`: their `AMP2`
    /// in `|M'|²` is zero.
    pub fn mask(&self, pattern: u32, weights: &mut [f64]) {
        for (w, &bits) in weights.iter_mut().zip(&self.config_lines) {
            if bits & pattern != 0 {
                *w = 0.0;
            }
        }
    }
}

/// One veto per subprocess, pairing each non-empty set of `sets` with the
/// evaluator compiled from it (in order, as `compile_subprocesses` pairs them).
/// Empty when `forbidden` is: an unrestricted card compiles nothing here.
pub fn subprocess_vetoes(
    sets: &[DiagramSet],
    evals: &[AmplitudeEvaluator],
    forbidden: &[i64],
    evaluated: &EvaluatedModel,
    card: &RunCard,
) -> Result<Vec<Option<OnShellVeto>>, OnShellVetoError> {
    if forbidden.is_empty() {
        return Ok(Vec::new());
    }
    let window = BwWindow::from_run_card(card);
    sets.iter()
        .filter(|s| !s.diagrams.is_empty())
        .zip(evals)
        .map(|(set, eval)| OnShellVeto::new(set, eval, forbidden, evaluated, window))
        .collect()
}

/// What `$` marks on one subprocess, independent of diagram order: per diagram,
/// its marked lines as (outgoing legs, mass parameter, width parameter), and
/// the diagrams as a sorted multiset.
type Marking = Vec<Vec<(Vec<usize>, String, String)>>;

fn marking(diagrams: &[Diagram], forbidden: &[i64], model: &UFOModel) -> Marking {
    let mut out: Marking = diagrams
        .iter()
        .map(|d| {
            let mut lines: Vec<_> = marked_lines(d, forbidden, model)
                .map(|prop| {
                    let particle = model.particle(prop.particle);
                    (
                        final_side_legs(&prop.momentum, d.n_in),
                        particle.mass_param.clone(),
                        particle.width_param.clone(),
                    )
                })
                .collect();
            lines.sort();
            lines
        })
        .collect();
    out.sort();
    out
}

/// The key a subprocess is found under: its incoming and outgoing PDG codes.
type SubprocessKey = (Vec<i64>, Vec<i64>);

fn set_key(set: &DiagramSet, model: &UFOModel) -> SubprocessKey {
    let pdg = |names: &[String]| {
        names
            .iter()
            .map(|n| model.particles.get(n.as_str()).map_or(0, |p| p.pdg_code))
            .collect()
    };
    (pdg(&set.particles_in), pdg(&set.particles_out))
}

/// The marking of every subprocess of an enumeration, taken before the
/// enumeration is partitioned into flavour groups (which keep only each group's
/// representative diagrams).
pub fn subprocess_markings(
    sets: &[DiagramSet],
    forbidden: &[i64],
    model: &UFOModel,
) -> BTreeMap<SubprocessKey, (String, Marking)> {
    sets.iter()
        .filter(|s| !s.diagrams.is_empty())
        .map(|s| {
            (
                set_key(s, model),
                (s.label(), marking(&s.diagrams, forbidden, model)),
            )
        })
        .collect()
}

/// Refuse a flavour group whose members `$` marks differently from its
/// representative. The group's members share one evaluator, one set of
/// diagrams and so one set of amplitudes with marked lines removed; MadEvent marks each subprocess's own diagrams, and
/// only a group whose members agree on the marking is described by one veto.
/// That fails only where the `$` list is not closed under the relabelling the
/// group makes — a member whose propagator is the antiparticle of the
/// representative's, say, under `$ w+` alone.
pub fn check_group_members(
    groups: &FlavorGroups,
    markings: &BTreeMap<SubprocessKey, (String, Marking)>,
    forbidden: &[i64],
    model: &UFOModel,
) -> Result<(), OnShellVetoError> {
    for g in groups.groups() {
        let rep = marking(g.diagrams(), forbidden, model);
        for member in g.members() {
            let key = (
                member.incoming.iter().map(|&c| i64::from(c)).collect(),
                member.outgoing.iter().map(|&c| i64::from(c)).collect(),
            );
            if let Some((label, own)) = markings.get(&key) {
                if *own != rep {
                    return Err(OnShellVetoError::GroupMarkingDiffers {
                        a: g.diagram_set().label(),
                        b: label.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

/// One veto per flavour group, in group order, each built on the group's
/// representative. Empty when `forbidden` is.
pub fn group_vetoes(
    groups: &FlavorGroups,
    markings: &BTreeMap<SubprocessKey, (String, Marking)>,
    forbidden: &[i64],
    evaluated: &EvaluatedModel,
    card: &RunCard,
) -> Result<Vec<Option<OnShellVeto>>, OnShellVetoError> {
    if forbidden.is_empty() {
        return Ok(Vec::new());
    }
    let window = BwWindow::from_run_card(card);
    check_group_members(groups, markings, forbidden, evaluated.model())?;
    groups
        .groups()
        .iter()
        .map(|g| OnShellVeto::new(g.diagram_set(), g.evaluator(), forbidden, evaluated, window))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::helas::eval::BoundAmplitude;
    use crate::proton::derive_flavor_groups;
    use crate::ufo::particles::{Particle, ParticleId};
    use crate::ufo::slha::ParamCard;
    use crate::ufo::sm::{sm_model, SMRestrict};
    use std::sync::Arc;

    const WINDOW: BwWindow = BwWindow {
        bwcutoff: 15.0,
        small_width_treatment: 1e-6,
    };

    fn sets(process: &str, model: &UFOModel) -> Vec<DiagramSet> {
        let card =
            parse_proc_card(&format!("generate {process}"), &ParsingOptions::default()).unwrap();
        generate_from_proc_card(&card, model)
            .unwrap()
            .into_iter()
            .filter(|s| !s.diagrams.is_empty())
            .collect()
    }

    fn compiled(process: &str, evaluated: &EvaluatedModel) -> (DiagramSet, AmplitudeEvaluator) {
        let model = evaluated.model();
        let set = sets(process, model).remove(0);
        let eval = compile_class(&set, model, evaluated).unwrap();
        (set, eval)
    }

    fn veto(
        process: &str,
        forbidden: &[&str],
        evaluated: &EvaluatedModel,
        window: BwWindow,
    ) -> Option<OnShellVeto> {
        let (set, eval) = compiled(process, evaluated);
        let ids: Vec<i64> = forbidden
            .iter()
            .map(|n| particle(evaluated.model(), n).pdg_code)
            .collect();
        OnShellVeto::new(&set, &eval, &ids, evaluated, window).unwrap()
    }

    fn m2(eval: &AmplitudeEvaluator, evaluated: &EvaluatedModel, p: &[V]) -> f64 {
        let bound = BoundAmplitude::<f64>::bind(eval, evaluated);
        bound.eval_m2(p, &mut bound.scratch_space())
    }

    fn particle<'m>(model: &'m UFOModel, name: &str) -> &'m Particle {
        model
            .particles
            .values()
            .find(|p| p.name.eq_ignore_ascii_case(name))
            .unwrap_or_else(|| panic!("no particle {name}"))
    }

    fn pole(evaluated: &EvaluatedModel, name: &str) -> f64 {
        let id = evaluated
            .model()
            .particles
            .values()
            .position(|p| p.name.eq_ignore_ascii_case(name))
            .map(ParticleId::from)
            .unwrap();
        evaluated.mass(id)
    }

    /// `a b → c d` in the centre of mass, massless, beams along ±z.
    fn two_to_two(sqrt_s: f64, cos: f64) -> Vec<V> {
        let e = sqrt_s / 2.0;
        let sin = (1.0 - cos * cos).sqrt();
        vec![
            V::new(e, 0.0, 0.0, e),
            V::new(e, 0.0, 0.0, -e),
            V::new(e, e * sin, 0.0, e * cos),
            V::new(e, -e * sin, 0.0, -e * cos),
        ]
    }

    fn boost_z(p: V, beta: f64) -> V {
        let gamma = 1.0 / (1.0 - beta * beta).sqrt();
        V::new(
            gamma * (p.e() + beta * p.pz()),
            p.px(),
            p.py(),
            gamma * (p.pz() + beta * p.e()),
        )
    }

    /// Two daughters of mass `m` each from a parent of mass `mass` moving along
    /// `z` with velocity `beta`, at rest-frame polar angle `cos` and azimuth `phi`.
    fn pair(mass: f64, m: f64, beta: f64, cos: f64, phi: f64) -> [V; 2] {
        let q = (mass * mass / 4.0 - m * m).sqrt();
        let sin = (1.0 - cos * cos).sqrt();
        let (x, y, z) = (q * sin * phi.cos(), q * sin * phi.sin(), q * cos);
        let e = mass / 2.0;
        [
            boost_z(V::new(e, x, y, z), beta),
            boost_z(V::new(e, -x, -y, -z), beta),
        ]
    }

    /// `e+ e- → (μ+ μ-) (τ+ τ-)` with the two pair masses given: the muon pair
    /// along `+z`, the tau pair along `−z`.
    fn four_leptons(sqrt_s: f64, m_mu: f64, m_ta: f64, m_tau: f64) -> Vec<V> {
        let s = sqrt_s * sqrt_s;
        let lambda = (s - (m_mu + m_ta).powi(2)) * (s - (m_mu - m_ta).powi(2));
        let p = lambda.sqrt() / (2.0 * sqrt_s);
        let beta = |m: f64| p / (p * p + m * m).sqrt();
        let [mu_p, mu_m] = pair(m_mu, 0.0, beta(m_mu), 0.3, 0.4);
        let [ta_p, ta_m] = pair(m_ta, m_tau, -beta(m_ta), -0.6, 2.1);
        let e = sqrt_s / 2.0;
        vec![
            V::new(e, 0.0, 0.0, e),
            V::new(e, 0.0, 0.0, -e),
            mu_p,
            mu_m,
            ta_p,
            ta_m,
        ]
    }

    #[test]
    fn the_window_is_the_p1d_propagators() {
        let z = |m: f64| m * m;
        // Inside and outside `|√p² − M| < bwcutoff · Γ` on both sides of the pole.
        assert!(WINDOW.zeroes(z(91.188 + 36.5), 91.188, 2.44));
        assert!(WINDOW.zeroes(z(91.188 - 36.5), 91.188, 2.44));
        assert!(!WINDOW.zeroes(z(91.188 + 36.7), 91.188, 2.44));
        assert!(!WINDOW.zeroes(z(91.188 - 36.7), 91.188, 2.44));
        // A line with no width is never zeroed, not even on its pole.
        assert!(!WINDOW.zeroes(z(91.188), 91.188, 0.0));
        // No Γ/M condition: the propagator's theta function has none, unlike
        // cut_bw's phase-space rejection.
        assert!(WINDOW.zeroes(z(100.0), 100.0, 12.0));
        // Once bwcutoff · Γ exceeds M the theta function's lower root is
        // bwcutoff · Γ − M, not zero: the zeroed interval is (650, 850) here.
        assert!(!WINDOW.zeroes(z(100.0), 100.0, 50.0));
        assert!(WINDOW.zeroes(z(700.0), 100.0, 50.0));
        // A tiny width is floored at M · small_width_treatment first.
        assert!(WINDOW.zeroes(z(100.0 + 14e-4), 100.0, 1e-12));
        assert!(!WINDOW.zeroes(z(100.0 + 16e-4), 100.0, 1e-12));
        let card = RunCard::parse("  3.5 = bwcutoff\n").unwrap();
        assert_eq!(BwWindow::from_run_card(&card).bwcutoff, 3.5);
    }

    /// Inside the `Z` window `e+ e- > mu+ mu- $ z` is the photon diagram alone:
    /// the zeroed amplitude is checked against `/ z`, compiled separately, so a
    /// line zeroed on the wrong diagram, or a factor that reweights rather than
    /// removes, shows. The `Z` configuration, and only it, leaves the `AMP2`
    /// draws.
    #[test]
    fn inside_the_z_window_only_the_photon_diagram_is_left() {
        let evaluated = EvaluatedModel::from_model(sm_model(SMRestrict::Default));
        let mz = pole(&evaluated, "z");
        let veto = veto("e+ e- > mu+ mu-", &["z"], &evaluated, WINDOW).unwrap();
        let (_, photon) = compiled("e+ e- > mu+ mu- / z", &evaluated);
        let (_, whole) = compiled("e+ e- > mu+ mu-", &evaluated);
        assert_eq!(veto.n_lines(), 1);
        for sqrt_s in [mz, 100.0] {
            for cos in [-0.8, -0.1, 0.4, 0.9] {
                let p = two_to_two(sqrt_s, cos);
                let pattern = veto.pattern(&p);
                assert_eq!(pattern, 1);
                let (part, flows) = veto.amplitude(pattern).unwrap();
                assert_eq!(flows, [0]);
                let (a, b) = (m2(part, &evaluated, &p), m2(&photon, &evaluated, &p));
                assert!(
                    (a - b).abs() < 1e-12 * b,
                    "√s {sqrt_s} cos {cos}: {a} vs {b}"
                );
                assert!(a < m2(&whole, &evaluated, &p));
            }
        }
        let mut weights = vec![1.0; veto.n_configs()];
        veto.mask(1, &mut weights);
        assert_eq!(weights.iter().filter(|&&w| w == 0.0).count(), 1);

        // Outside the window nothing is zeroed.
        assert_eq!(veto.pattern(&two_to_two(200.0, 0.3)), 0);
        let narrow = BwWindow {
            bwcutoff: 3.0,
            ..WINDOW
        };
        let veto = self::veto("e+ e- > mu+ mu-", &["z"], &evaluated, narrow).unwrap();
        assert_eq!(veto.pattern(&two_to_two(100.0, 0.3)), 0);
        assert_eq!(veto.pattern(&two_to_two(95.0, 0.3)), 1);
    }

    /// A particle no diagram carries as an s-channel, or one named against its
    /// orientation, marks nothing.
    #[test]
    fn a_process_without_the_line_marks_nothing() {
        let evaluated = EvaluatedModel::from_model(sm_model(SMRestrict::Default));
        assert!(veto("e+ e- > mu+ mu-", &["w+"], &evaluated, WINDOW).is_none());
        assert!(veto("e+ e- > mu+ mu-", &[], &evaluated, WINDOW).is_none());
        // `u d~ > e+ ve` carries a W+ flowing to the final state; `$ w-` names the
        // other orientation and is MadGraph's no-op.
        assert!(veto("u d~ > e+ ve", &["w+"], &evaluated, WINDOW).is_some());
        assert!(veto("u d~ > e+ ve", &["w-"], &evaluated, WINDOW).is_none());
    }

    /// With `Γ_Z` raised past `M_Z/10` the line is still zeroed on its window:
    /// the amplitude's theta function knows no `Γ/M` rule.
    #[test]
    fn a_wide_resonance_is_still_zeroed() {
        let model = sm_model(SMRestrict::Default);
        let card: ParamCard = "DECAY 23 12.0\n".parse().unwrap();
        let wide = EvaluatedModel::from_model_card(Arc::clone(&model), &card);
        let veto = veto("e+ e- > mu+ mu-", &["z"], &wide, WINDOW).unwrap();
        assert_eq!(veto.pattern(&two_to_two(91.188 + 150.0, 0.2)), 1);
        assert_eq!(veto.pattern(&two_to_two(91.188 + 190.0, 0.2)), 0);
    }

    /// `e+ e- > mu+ mu- ta+ ta- $ z`: diagrams carry up to two marked `Z` lines.
    /// Each line is zeroed on its own window, a configuration with two marked
    /// lines leaves the draws when either is zeroed, and every compiled pattern
    /// amplitude is the sum over exactly the diagrams carrying none of its lines.
    #[test]
    fn two_marked_lines_are_zeroed_independently() {
        let evaluated = EvaluatedModel::from_model(sm_model(SMRestrict::Default));
        let model = evaluated.model();
        let mz = pole(&evaluated, "z");
        let m_tau = pole(&evaluated, "ta-");
        let (set, eval) = compiled("e+ e- > mu+ mu- ta+ ta-", &evaluated);
        let z = [particle(model, "z").pdg_code];
        let veto = OnShellVeto::new(&set, &eval, &z, &evaluated, WINDOW)
            .unwrap()
            .unwrap();
        let bit_of = |legs: &[usize]| {
            veto.lines
                .iter()
                .position(|l| l.legs == legs)
                .map(|i| 1u32 << i)
                .unwrap_or_else(|| panic!("no marked line over {legs:?}"))
        };
        let (mu, ta) = (bit_of(&[2, 3]), bit_of(&[4, 5]));

        let p = four_leptons(250.0, mz, 40.0, m_tau);
        assert_eq!(veto.pattern(&p) & (mu | ta), mu);
        let p = four_leptons(250.0, 40.0, mz + 10.0, m_tau);
        assert_eq!(veto.pattern(&p) & (mu | ta), ta);
        let p = four_leptons(250.0, mz, mz, m_tau);
        assert_eq!(veto.pattern(&p) & (mu | ta), mu | ta);
        assert_eq!(
            veto.pattern(&four_leptons(250.0, 40.0, 150.0, m_tau)) & (mu | ta),
            0
        );

        // The configurations the draws lose, read off the diagrams independently.
        let mut start = 0;
        let mut carries = Vec::new();
        for &span in eval.config_amp_counts() {
            let d = &set.diagrams[eval.config_amp_diagrams()[start]];
            start += span;
            let over: Vec<Vec<usize>> = marked_lines(d, &z, model)
                .map(|p| final_side_legs(&p.momentum, d.n_in))
                .collect();
            carries.push((over.contains(&vec![2, 3]), over.contains(&vec![4, 5])));
        }
        assert!(
            carries.iter().any(|&(a, b)| a && b),
            "a configuration with two marked lines"
        );
        let mut weights = vec![1.0; veto.n_configs()];
        veto.mask(mu, &mut weights);
        let expected: Vec<f64> = carries
            .iter()
            .map(|&(a, _)| if a { 0.0 } else { 1.0 })
            .collect();
        assert_eq!(weights, expected);

        // Each pattern amplitude is the one compiled from the diagrams it keeps.
        let p = four_leptons(250.0, mz, mz, m_tau);
        for pattern in [mu, ta, mu | ta] {
            let kept: Vec<Diagram> = set
                .diagrams
                .iter()
                .filter(|d| {
                    marked_lines(d, &z, model).all(|l| {
                        let legs = final_side_legs(&l.momentum, d.n_in);
                        bit_of(&legs) & pattern == 0
                    })
                })
                .cloned()
                .collect();
            assert!(kept.len() < set.diagrams.len());
            let subset = DiagramSet {
                particles_in: set.particles_in.clone(),
                particles_out: set.particles_out.clone(),
                polarizations: set.polarizations.clone(),
                diagrams: kept,
            };
            let reference = compile_class(&subset, model, &evaluated).unwrap();
            let (part, _) = veto.amplitude(pattern).unwrap();
            let (a, b) = (m2(part, &evaluated, &p), m2(&reference, &evaluated, &p));
            assert!(
                (a - b).abs() <= 1e-12 * b.abs(),
                "pattern {pattern:b}: {a} vs {b}"
            );
        }
    }

    /// One incoming particle: every line is an s-channel oriented away from the
    /// decaying particle, and `t > b e+ ve $ w+` has nothing left on the W window.
    #[test]
    fn a_decay_has_nothing_left_on_the_window() {
        let evaluated = EvaluatedModel::from_model(sm_model(SMRestrict::Default));
        let mt = pole(&evaluated, "t");
        let mw = pole(&evaluated, "w+");
        let veto = veto("t > b e+ ve", &["w+"], &evaluated, WINDOW).unwrap();
        assert!(self::veto("t > b e+ ve", &["w-"], &evaluated, WINDOW).is_none());
        let at = |m_ev: f64| {
            let p = (mt * mt - m_ev * m_ev) / (2.0 * mt);
            let beta = p / (p * p + m_ev * m_ev).sqrt();
            let [e, v] = pair(m_ev, 0.0, beta, 0.2, 1.0);
            vec![V::new(mt, 0.0, 0.0, 0.0), V::new(p, 0.0, 0.0, -p), e, v]
        };
        assert_eq!(veto.pattern(&at(mw + 1.0)), 1);
        assert!(veto.amplitude(1).is_none());
        assert_eq!(veto.pattern(&at(40.0)), 0);
    }

    /// A flavour group's members share the representative's marking on
    /// `p p > e+ e- $ z`; a member marked differently is refused.
    #[test]
    fn group_members_must_share_the_marking() {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(Arc::clone(&model));
        let z = [particle(&model, "z").pdg_code];
        let all = sets("p p > e+ e-", &model);
        let mut markings = subprocess_markings(&all, &z, &model);
        let groups = derive_flavor_groups(all, &model, &evaluated, &RunCard::default()).unwrap();
        assert!(groups.groups().iter().any(|g| g.members().len() > 1));
        check_group_members(&groups, &markings, &z, &model).unwrap();
        let vetoes = group_vetoes(&groups, &markings, &z, &evaluated, &RunCard::default()).unwrap();
        assert!(vetoes.iter().all(Option::is_some));

        let member = groups
            .groups()
            .iter()
            .flat_map(|g| g.members().iter().skip(1))
            .next()
            .unwrap();
        let key = (
            member.incoming.iter().map(|&c| i64::from(c)).collect(),
            member.outgoing.iter().map(|&c| i64::from(c)).collect(),
        );
        markings.get_mut(&key).unwrap().1.clear();
        assert!(matches!(
            check_group_members(&groups, &markings, &z, &model),
            Err(OnShellVetoError::GroupMarkingDiffers { .. })
        ));
    }
}
