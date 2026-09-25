//! Forbidden on-shell s-channels (`$ A`) as MadEvent integrates them.
//!
//! `$ A` keeps every diagram (`diagram_generation.py:781` only marks the
//! propagator `onshell = False`), and `export_v4.py:5879` writes the mark into
//! `decayBW.inc` as `gForceBW(i, iconfig) = 2` for each s-channel line of each
//! integration configuration. `cut_bw` (`Template/LO/SubProcesses/myamp.f:76`)
//! then rejects a phase-space point *in the configuration MadEvent is
//! integrating* when one of that configuration's marked lines is on its
//! Breit–Wigner window:
//!
//! * only s-channel lines are visited (`myamp.f:117`: the walk stops at the first
//!   line attached to an incoming leg, and a configuration lists its s-channel
//!   lines first);
//! * only lines with a positive width (`myamp.f:123`), which is then floored at
//!   `M · small_width_treatment` (`myamp.f:132`);
//! * on-shell means `|√p² − M| < bwcutoff · Γ` and `Γ/M < 0.1` (`myamp.f:136`);
//! * the rejection happens only when `sde_strat == 1` (`myamp.f:142`);
//! * the first marked line on its window rejects the point, so a configuration
//!   with two marked lines is rejected when either is on-shell.
//!
//! `passcuts` calls `cut_bw` (`cuts.f:509`), so a rejected point contributes
//! nothing to that configuration's integral. Configuration `c`'s integrand is
//! `|M|² · AMP2_c / Σ_d AMP2_d` (`matrix_madevent_group_v4.inc:213`, the
//! `MULTI_CHANNEL` block at `sde_strat = 1`), summed over every configuration
//! the process has. The integral MadEvent reports is therefore that of
//!
//! ```text
//! F(x) = |M|²(x) · Σ_c AMP2_c(x) (1 − V_c(x)) / Σ_d AMP2_d(x)
//! ```
//!
//! with `V_c(x)` the rejection above. That is a function of the point alone, so
//! any unbiased sampler integrates it; this module supplies the factor
//! [`OnShellVeto::factor`] and the flags `V_c`.
//!
//! A configuration here is one of the evaluator's `AMP2` accumulators, which is
//! MadGraph's (`get_amp2_lines`, grouped as `IdentifyConfigTag` groups them), and
//! its lines are read off the accumulator's first diagram, the representative
//! `configs.inc` writes the forest of. `$` is a property of the process line and
//! MadEvent marks each subprocess's own diagrams, so each subprocess gets its own
//! veto; flavour groups sharing one matrix element are required to share the
//! marking too ([`check_group_members`]).

use std::collections::BTreeMap;

use thiserror::Error;

use crate::diagrams::diagram::Diagram;
use crate::diagrams::schannel::oriented_s_channel_id;
use crate::diagrams::DiagramSet;
use crate::helas::eval::AmplitudeEvaluator;
use crate::helas::repr::lorentz::LorentzVector;
use crate::proton::FlavorGroups;
use crate::runcard::RunCard;
use crate::ufo::{EvaluatedModel, UFOModel};

type V = LorentzVector<f64>;

/// `cut_bw`'s ceiling on `Γ/M` for a line that is not forced on-shell.
const MAX_WIDTH_OVER_MASS: f64 = 0.1;

#[derive(Debug, Error, PartialEq)]
pub enum OnShellVetoError {
    /// `cut_bw` applies the veto only at `sde_strat == 1`, and MadGraph writes
    /// that value into the run card of every process with a `$`
    /// (`banner.py:5055`). A card that moves it would make `$` do nothing.
    #[error(
        "the run card sets sde_strategy = {0}; with a '$' restriction MadEvent vetoes the \
         on-shell window only at sde_strategy = 1, the value MadGraph writes for such a \
         process"
    )]
    SdeStrategy(i64),
    /// Two subprocesses share one compiled matrix element but not the same
    /// marked propagators, so no single veto describes both.
    #[error(
        "'{a}' and '{b}' share a matrix element but differ in which propagators the '$' \
         restriction marks"
    )]
    GroupMarkingDiffers { a: String, b: String },
}

/// The run card's Breit–Wigner window for the veto.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BwWindow {
    /// `bwcutoff`: the window's half-width in units of the width.
    pub bwcutoff: f64,
    /// `small_width_treatment`: the floor on `Γ/M`.
    pub small_width_treatment: f64,
}

impl BwWindow {
    /// The window a run card describes, refusing a card on which MadEvent would
    /// not veto at all.
    pub fn from_run_card(card: &RunCard) -> Result<Self, OnShellVetoError> {
        let sde = card.int("SDE_strategy");
        if sde != 1 {
            return Err(OnShellVetoError::SdeStrategy(sde));
        }
        Ok(BwWindow {
            bwcutoff: card.float("bwcutoff"),
            small_width_treatment: card.float("small_width_treatment"),
        })
    }

    /// `cut_bw`'s on-shell test for a line of invariant mass `m` whose particle
    /// has pole mass `mass` and width `width` (`myamp.f:123`–`:139`, the branch
    /// for a line not forced on-shell). A line with no width is never on-shell.
    pub fn on_shell(&self, m: f64, mass: f64, width: f64) -> bool {
        match self.half_width(mass, width) {
            Some(half) => (m - mass.abs()).abs() < half,
            None => false,
        }
    }

    /// `bwcutoff · Γ` for a line that can be on-shell at all, `None` for one that
    /// cannot: no width, no mass, or `Γ/M` at or above the ceiling after the
    /// width floor.
    fn half_width(&self, mass: f64, width: f64) -> Option<f64> {
        let mass = mass.abs();
        if width <= 0.0 || mass <= 0.0 {
            return None;
        }
        let width = width.max(mass * self.small_width_treatment);
        (width / mass < MAX_WIDTH_OVER_MASS).then_some(self.bwcutoff * width)
    }
}

/// One marked line: the outgoing legs whose momenta sum to its momentum, and
/// its window.
#[derive(Debug, Clone, PartialEq)]
struct MarkedLine {
    legs: Vec<usize>,
    mass: f64,
    half_width: f64,
}

/// The veto of one subprocess: per integration configuration, the marked lines
/// that can reach their window.
#[derive(Debug, Clone)]
pub struct OnShellVeto {
    configs: Vec<Vec<MarkedLine>>,
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
) -> impl Iterator<Item = &'d crate::diagrams::diagram::Prop> + 'd {
    diagram.props.iter().filter(move |prop| {
        oriented_s_channel_id(prop, diagram, model).is_some_and(|id| forbidden.contains(&id))
    })
}

impl OnShellVeto {
    /// The veto `forbidden` puts on `eval`, compiled from `diagrams`, or `None`
    /// when no configuration carries a marked line that can reach its window —
    /// every point then keeps its full weight and nothing is evaluated per
    /// point.
    pub fn new(
        eval: &AmplitudeEvaluator,
        diagrams: &[Diagram],
        forbidden: &[i64],
        evaluated: &EvaluatedModel,
        window: BwWindow,
    ) -> Option<Self> {
        if forbidden.is_empty() {
            return None;
        }
        let model = evaluated.model();
        let mut configs = Vec::with_capacity(eval.n_configs());
        let mut start = 0;
        for &span in eval.config_amp_counts() {
            let representative = &diagrams[eval.config_amp_diagrams()[start]];
            start += span;
            let lines = marked_lines(representative, forbidden, model)
                .filter_map(|prop| {
                    let mass = evaluated.mass(prop.particle);
                    let half_width = window.half_width(mass, evaluated.width(prop.particle))?;
                    Some(MarkedLine {
                        legs: final_side_legs(&prop.momentum, representative.n_in),
                        mass: mass.abs(),
                        half_width,
                    })
                })
                .collect::<Vec<_>>();
            configs.push(lines);
        }
        configs
            .iter()
            .any(|c| !c.is_empty())
            .then_some(OnShellVeto { configs })
    }

    /// The number of integration configurations the flags cover.
    pub fn n_configs(&self) -> usize {
        self.configs.len()
    }

    /// Set `vetoed[c]` to `V_c` at the external momenta `momenta` (incoming
    /// first, any frame), and return whether any configuration is vetoed.
    ///
    /// # Panics
    ///
    /// If `vetoed` is not one flag per configuration.
    pub fn mark(&self, momenta: &[V], vetoed: &mut [bool]) -> bool {
        assert_eq!(
            vetoed.len(),
            self.configs.len(),
            "one veto flag per integration configuration"
        );
        let mut any = false;
        for (flag, lines) in vetoed.iter_mut().zip(&self.configs) {
            *flag = lines.iter().any(|line| {
                let mut p = [0.0; 4];
                for &leg in &line.legs {
                    let k = momenta[leg];
                    p[0] += k.e();
                    p[1] += k.px();
                    p[2] += k.py();
                    p[3] += k.pz();
                }
                let m = (p[0] * p[0] - p[1] * p[1] - p[2] * p[2] - p[3] * p[3])
                    .max(0.0)
                    .sqrt();
                (m - line.mass).abs() < line.half_width
            });
            any |= *flag;
        }
        any
    }

    /// `Σ_c AMP2_c (1 − V_c) / Σ_d AMP2_d`: the share of the matrix element the
    /// configurations left standing carry. Where every `AMP2` vanishes MadEvent
    /// leaves the matrix element unweighted (`XTOT = 0`), and so does this.
    pub fn factor(amp2: &[f64], vetoed: &[bool]) -> f64 {
        let total: f64 = amp2.iter().sum();
        if total == 0.0 {
            return 1.0;
        }
        let kept: f64 = amp2
            .iter()
            .zip(vetoed)
            .filter(|(_, &v)| !v)
            .map(|(a, _)| a)
            .sum();
        kept / total
    }

    /// Zero the entries of `weights` whose configuration is vetoed, so a draw
    /// over configurations lands only where MadEvent's point would have been
    /// kept.
    pub fn mask(weights: &mut [f64], vetoed: &[bool]) {
        for (w, &v) in weights.iter_mut().zip(vetoed) {
            if v {
                *w = 0.0;
            }
        }
    }
}

/// One veto per subprocess, pairing each non-empty set of `sets` with the
/// evaluator compiled from it (in order, as `compile_subprocesses` pairs them).
/// Empty when `forbidden` is: an unrestricted card reads nothing from the run
/// card here.
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
    let window = BwWindow::from_run_card(card)?;
    let nonempty = sets.iter().filter(|s| !s.diagrams.is_empty());
    Ok(nonempty
        .zip(evals)
        .map(|(set, eval)| OnShellVeto::new(eval, &set.diagrams, forbidden, evaluated, window))
        .collect())
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
/// diagrams and so one veto; MadEvent marks each subprocess's own diagrams, and
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
    let window = BwWindow::from_run_card(card)?;
    check_group_members(groups, markings, forbidden, evaluated.model())?;
    Ok(groups
        .groups()
        .iter()
        .map(|g| OnShellVeto::new(g.evaluator(), g.diagrams(), forbidden, evaluated, window))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::hadronic::compile_class;
    use crate::helas::eval::BoundAmplitude;
    use crate::proton::derive_flavor_groups;
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

    fn m2(eval: &AmplitudeEvaluator, evaluated: &EvaluatedModel, p: &[V]) -> f64 {
        let bound = BoundAmplitude::<f64>::bind(eval, evaluated);
        bound.eval_m2(p, &mut bound.scratch_space())
    }

    fn amp2(eval: &AmplitudeEvaluator, evaluated: &EvaluatedModel, p: &[V]) -> Vec<f64> {
        let bound = BoundAmplitude::<f64>::bind(eval, evaluated);
        let mut out = vec![0.0; eval.n_configs()];
        bound.eval_amp2(p, &mut bound.scratch_space(), &mut out);
        out
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
    /// `z` with velocity `beta`, decaying at polar angle `cos` and azimuth `phi`
    /// in its rest frame.
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

    /// `e+ e- → (μ+ μ-) (τ+ τ-)` at `sqrt_s` with the two pair masses given: the
    /// muon pair along `+z`, the tau pair along `−z`.
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

    fn invariant_mass(ps: &[V]) -> f64 {
        let p = ps.iter().fold([0.0; 4], |a, k| {
            [a[0] + k.e(), a[1] + k.px(), a[2] + k.py(), a[3] + k.pz()]
        });
        (p[0] * p[0] - p[1] * p[1] - p[2] * p[2] - p[3] * p[3]).sqrt()
    }

    fn particle<'m>(model: &'m UFOModel, name: &str) -> &'m crate::ufo::particles::Particle {
        model
            .particles
            .values()
            .find(|p| p.name.eq_ignore_ascii_case(name))
            .unwrap_or_else(|| panic!("no particle {name}"))
    }

    fn pdg(model: &UFOModel, name: &str) -> i64 {
        particle(model, name).pdg_code
    }

    fn pole(evaluated: &EvaluatedModel, name: &str) -> f64 {
        let id = evaluated
            .model()
            .particles
            .values()
            .position(|p| p.name.eq_ignore_ascii_case(name))
            .map(crate::ufo::particles::ParticleId::from)
            .unwrap();
        evaluated.mass(id)
    }

    #[test]
    fn the_window_is_madevents() {
        // Inside and outside `|m − M| < bwcutoff · Γ`, both sides of the pole.
        assert!(WINDOW.on_shell(91.188 + 36.5, 91.188, 2.44));
        assert!(WINDOW.on_shell(91.188 - 36.5, 91.188, 2.44));
        assert!(!WINDOW.on_shell(91.188 + 36.7, 91.188, 2.44));
        assert!(!WINDOW.on_shell(91.188 - 36.7, 91.188, 2.44));
        // No width: never a Breit–Wigner line.
        assert!(!WINDOW.on_shell(91.188, 91.188, 0.0));
        // Γ/M at or above 0.1 is never on-shell for a line `$` marks.
        assert!(WINDOW.on_shell(100.0, 100.0, 9.99));
        assert!(!WINDOW.on_shell(100.0, 100.0, 10.0));
        // A tiny width is floored at M · small_width_treatment first.
        assert!(WINDOW.on_shell(100.0 + 14e-4, 100.0, 1e-12));
        assert!(!WINDOW.on_shell(100.0 + 16e-4, 100.0, 1e-12));
    }

    #[test]
    fn a_card_away_from_sde_strategy_one_is_refused() {
        let card = RunCard::parse("  2 = sde_strategy\n").unwrap();
        assert_eq!(
            BwWindow::from_run_card(&card),
            Err(OnShellVetoError::SdeStrategy(2))
        );
        let card = RunCard::parse("  3.5 = bwcutoff\n").unwrap();
        assert_eq!(BwWindow::from_run_card(&card).unwrap().bwcutoff, 3.5);
    }

    /// On the `Z` pole every point is inside the `Z` window, so
    /// `e+ e- > mu+ mu- $ z` keeps only the photon configuration's share of
    /// `|M|²`. The share is checked against two separately compiled amplitudes,
    /// `/ z` (the photon diagram alone) and `/ a` (the `Z` alone), so a veto on
    /// the wrong configuration, or a flag landing on the wrong accumulator, reads
    /// the `Z`'s share instead.
    #[test]
    fn on_the_z_pole_only_the_photon_share_is_kept() {
        let evaluated = EvaluatedModel::from_model(sm_model(SMRestrict::Default));
        let model = evaluated.model();
        let mz = pole(&evaluated, "z");
        let (set, eval) = compiled("e+ e- > mu+ mu-", &evaluated);
        let (_, photon) = compiled("e+ e- > mu+ mu- / z", &evaluated);
        let (_, z) = compiled("e+ e- > mu+ mu- / a", &evaluated);
        let veto = OnShellVeto::new(&eval, &set.diagrams, &[pdg(model, "z")], &evaluated, WINDOW)
            .expect("the Z configuration carries the marked line");

        for cos in [-0.8, -0.1, 0.4, 0.9] {
            let p = two_to_two(mz, cos);
            let mut flags = vec![false; veto.n_configs()];
            assert!(veto.mark(&p, &mut flags));
            assert_eq!(flags.iter().filter(|&&f| f).count(), 1, "{flags:?}");
            let kept = OnShellVeto::factor(&amp2(&eval, &evaluated, &p), &flags);
            let (a, zz) = (m2(&photon, &evaluated, &p), m2(&z, &evaluated, &p));
            let expected = a / (a + zz);
            assert!(
                (kept - expected).abs() < 1e-12 * expected,
                "cos {cos}: kept share {kept}, photon share {expected}"
            );
            assert!(expected < 0.01, "on the pole the Z dominates");
        }

        // Away from the window nothing is vetoed and the factor is not needed.
        let mut flags = vec![false; veto.n_configs()];
        assert!(!veto.mark(&two_to_two(200.0, 0.3), &mut flags));
        assert!(flags.iter().all(|&f| !f));
        // At 100 GeV the window (±15 Γ_Z) still covers the point.
        assert!(veto.mark(&two_to_two(100.0, 0.3), &mut flags));
        let narrow = BwWindow {
            bwcutoff: 3.0,
            ..WINDOW
        };
        let veto =
            OnShellVeto::new(&eval, &set.diagrams, &[pdg(model, "z")], &evaluated, narrow).unwrap();
        assert!(!veto.mark(&two_to_two(100.0, 0.3), &mut flags));
    }

    /// A process no configuration of which carries the named particle as an
    /// s-channel, and a particle named against its orientation, veto nothing.
    #[test]
    fn a_configuration_without_the_line_is_never_vetoed() {
        let evaluated = EvaluatedModel::from_model(sm_model(SMRestrict::Default));
        let model = evaluated.model();
        let (set, eval) = compiled("e+ e- > mu+ mu-", &evaluated);
        assert!(OnShellVeto::new(
            &eval,
            &set.diagrams,
            &[pdg(model, "w+")],
            &evaluated,
            WINDOW
        )
        .is_none());
        // `u d~ > e+ ve` carries a W+ flowing to the final state; `$ w-` names the
        // other orientation and is MadGraph's no-op.
        let (set, eval) = compiled("u d~ > e+ ve", &evaluated);
        assert!(OnShellVeto::new(
            &eval,
            &set.diagrams,
            &[pdg(model, "w+")],
            &evaluated,
            WINDOW
        )
        .is_some());
        assert!(OnShellVeto::new(
            &eval,
            &set.diagrams,
            &[pdg(model, "w-")],
            &evaluated,
            WINDOW
        )
        .is_none());
        // An empty list is no veto at all.
        assert!(OnShellVeto::new(&eval, &set.diagrams, &[], &evaluated, WINDOW).is_none());
    }

    /// With `Γ_Z/M_Z` raised to 0.1 the `Z` line is never on-shell for `$`, just
    /// below it the veto is live.
    #[test]
    fn a_wide_resonance_is_never_vetoed() {
        let model = sm_model(SMRestrict::Default);
        let mz = 91.188;
        let with_width = |width: f64| {
            let card: ParamCard = format!("DECAY 23 {width}\n").parse().unwrap();
            EvaluatedModel::from_model_card(Arc::clone(&model), &card)
        };
        let wide = with_width(0.1 * mz * 1.0001);
        let (set, eval) = compiled("e+ e- > mu+ mu-", &wide);
        assert!(
            OnShellVeto::new(&eval, &set.diagrams, &[pdg(&model, "z")], &wide, WINDOW).is_none()
        );
        let narrow = with_width(0.1 * mz * 0.9999);
        let (set, eval) = compiled("e+ e- > mu+ mu-", &narrow);
        assert!(
            OnShellVeto::new(&eval, &set.diagrams, &[pdg(&model, "z")], &narrow, WINDOW).is_some()
        );
    }

    /// `e+ e- > mu+ mu- ta+ ta- $ z`: configurations carry zero, one or two
    /// marked `Z` lines. At a point with the muon pair on the `Z` pole and the tau
    /// pair far off it, exactly the configurations with a `Z` line over the muon
    /// pair are vetoed — including those whose second, tau-pair `Z` is off-shell,
    /// since either marked line rejects the point — and swapping which pair is
    /// on-shell swaps the set.
    #[test]
    fn either_of_two_marked_lines_vetoes_its_configuration() {
        let evaluated = EvaluatedModel::from_model(sm_model(SMRestrict::Default));
        let model = evaluated.model();
        let z = pdg(model, "z");
        let mz = pole(&evaluated, "z");
        let m_tau = pole(&evaluated, "ta-");
        let (set, eval) = compiled("e+ e- > mu+ mu- ta+ ta-", &evaluated);
        let veto = OnShellVeto::new(&eval, &set.diagrams, &[z], &evaluated, WINDOW).unwrap();

        // Which configurations carry a Z line over which pair, read off the
        // representative diagrams independently of the veto's own bookkeeping.
        let mut start = 0;
        let mut over = Vec::new();
        for &span in eval.config_amp_counts() {
            let d = &set.diagrams[eval.config_amp_diagrams()[start]];
            start += span;
            let pairs: Vec<Vec<usize>> = marked_lines(d, &[z], model)
                .map(|p| final_side_legs(&p.momentum, d.n_in))
                .collect();
            over.push((pairs.contains(&vec![2, 3]), pairs.contains(&vec![4, 5])));
        }
        assert!(
            over.iter().any(|&(mu, ta)| mu && ta),
            "some configuration carries two marked Z lines"
        );
        assert!(over.iter().any(|&(mu, ta)| !mu && !ta));

        let mut flags = vec![false; veto.n_configs()];
        let p = four_leptons(250.0, mz, 40.0, m_tau);
        assert!((invariant_mass(&p[2..4]) - mz).abs() < 1e-9);
        assert!((invariant_mass(&p[4..6]) - 40.0).abs() < 1e-9);
        assert!(veto.mark(&p, &mut flags));
        let expected: Vec<bool> = over.iter().map(|&(mu, _)| mu).collect();
        assert_eq!(flags, expected);

        let p = four_leptons(250.0, 40.0, mz + 10.0, m_tau);
        assert!(veto.mark(&p, &mut flags));
        let expected: Vec<bool> = over.iter().map(|&(_, ta)| ta).collect();
        assert_eq!(flags, expected);

        let p = four_leptons(250.0, 40.0, 150.0, m_tau);
        assert!(!veto.mark(&p, &mut flags));

        // The kept share is the unvetoed configurations' AMP2 over all of them.
        let p = four_leptons(250.0, mz, 40.0, m_tau);
        veto.mark(&p, &mut flags);
        let a = amp2(&eval, &evaluated, &p);
        let kept: f64 = a
            .iter()
            .zip(&flags)
            .filter(|(_, &f)| !f)
            .map(|(x, _)| x)
            .sum();
        let total: f64 = a.iter().sum();
        let factor = OnShellVeto::factor(&a, &flags);
        assert!((factor - kept / total).abs() < 1e-15);
        assert!(factor > 0.0 && factor < 1.0);
    }

    /// One incoming particle: every line is an s-channel oriented away from the
    /// decaying particle, and `t > b e+ ve $ w+` loses its one configuration on
    /// the W window.
    #[test]
    fn a_decay_loses_its_resonant_configuration() {
        let evaluated = EvaluatedModel::from_model(sm_model(SMRestrict::Default));
        let model = evaluated.model();
        let mt = pole(&evaluated, "t");
        let mw = pole(&evaluated, "w+");
        let (set, eval) = compiled("t > b e+ ve", &evaluated);
        let veto = OnShellVeto::new(
            &eval,
            &set.diagrams,
            &[pdg(model, "w+")],
            &evaluated,
            WINDOW,
        )
        .unwrap();
        assert!(OnShellVeto::new(
            &eval,
            &set.diagrams,
            &[pdg(model, "w-")],
            &evaluated,
            WINDOW
        )
        .is_none());
        let at = |m_ev: f64| {
            // The top at rest; the b recoils against the (e+ ve) system of mass m_ev.
            let p = ((mt * mt - m_ev * m_ev) / (2.0 * mt)).max(0.0);
            let beta = p / (p * p + m_ev * m_ev).sqrt();
            let [e, v] = pair(m_ev, 0.0, beta, 0.2, 1.0);
            vec![V::new(mt, 0.0, 0.0, 0.0), V::new(p, 0.0, 0.0, -p), e, v]
        };
        let mut flags = vec![false; veto.n_configs()];
        let p = at(mw + 1.0);
        assert!(veto.mark(&p, &mut flags));
        assert_eq!(
            OnShellVeto::factor(&amp2(&eval, &evaluated, &p), &flags),
            0.0
        );
        assert!(!veto.mark(&at(40.0), &mut flags));
    }

    /// A flavour group's members share the representative's marking on
    /// `p p > e+ e- $ z`; a member marked differently is refused.
    #[test]
    fn group_members_must_share_the_marking() {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(Arc::clone(&model));
        let z = [pdg(&model, "z")];
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
