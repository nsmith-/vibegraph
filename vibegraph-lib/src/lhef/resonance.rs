//! Which propagators an event record lists as intermediate resonances.
//!
//! MadEvent writes an `ISTUP = 2` record for a propagator of the event's
//! integration configuration (`ICONFIG`) when `cut_bw` (`myamp.f:76`) leaves it
//! flagged `OnBW`, and `addmothers` (`addmothers.f:253`) then gives it its
//! daughters' summed momentum, its virtuality as the mass, the colour its
//! daughters leave open, and the daughters' mother pointers. The flag is:
//!
//! * a timelike line of nonzero width whose invariant mass is inside
//!   `bwcutoff` widths of its pole, `|√p² − M| < bwcutoff·Γ` with `Γ` floored at
//!   `M·small_width_treatment` (`myamp.f:131`), **and** narrow, `Γ/M < 0.1`,
//!   unless a decay chain forces it on shell (`gForceBW = 1`, `myamp.f:136`);
//! * withdrawn from a line with a daughter of its own flavour: always when that
//!   daughter is an external leg, otherwise from whichever of the two a decay
//!   chain does not force, or, when neither is forced, from the one further
//!   from its pole (`myamp.f:146`–`:176`).
//!
//! A decay chain's forced lines always carry the flag inside their windows, and
//! a free line (a `W` inside `t > b e+ ve`) carries it whenever it lands inside
//! its own window, so the records of one subprocess differ from event to event.
//!
//! [`SubprocessResonances`] resolves every configuration's lines once, from the
//! diagram MadGraph writes the configuration from.

use crate::cuts::SMALL_WIDTH_TREATMENT;
use crate::diagrams::diagram::{Diagram, OnShell, PropIdx};
use crate::diagrams::schannel::oriented_s_channel_id;
use crate::helas::eval::AmplitudeEvaluator;
use crate::ufo::{EvaluatedModel, UFOModel};

use super::LhefError;

/// `cut_bw`'s bound on `Γ/M` for a line a decay chain does not force.
const NARROW_WIDTH_RATIO: f64 = 0.1;

/// One timelike propagator of a configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResonanceLine {
    /// The outgoing legs below the line, bit `k` naming the `k`-th outgoing leg.
    pub slots: u64,
    /// The particle as it flows towards the final state.
    pub pdg: i32,
    /// Its SU(3) representation in the UFO's code (`1`, `3`, `-3`, `8`, …).
    pub color: i32,
    pub mass: f64,
    /// MadGraph's `prwidth_tmp`: the width floored at `small_width_treatment`
    /// times the mass, `0` for a line without width.
    pub width: f64,
    /// Whether a decay chain forces the line on shell.
    pub forced: bool,
}

impl ResonanceLine {
    fn invariant_mass(&self, outgoing: &[[f64; 4]]) -> f64 {
        let p = sum_slots(self.slots, outgoing);
        (p[0] * p[0] - p[1] * p[1] - p[2] * p[2] - p[3] * p[3])
            .max(0.0)
            .sqrt()
    }

    /// Whether the line is inside its window, `|√p² − M| < bwcutoff·Γ`.
    fn in_window(&self, bwcutoff: f64, outgoing: &[[f64; 4]]) -> bool {
        (self.invariant_mass(outgoing) - self.mass).abs() < bwcutoff * self.width
    }
}

/// The sum of the outgoing momenta a slot mask names.
pub fn sum_slots(slots: u64, outgoing: &[[f64; 4]]) -> [f64; 4] {
    let mut p = [0.0; 4];
    for (k, q) in outgoing.iter().enumerate() {
        if slots & (1u64 << k) != 0 {
            for (sum, component) in p.iter_mut().zip(q) {
                *sum += component;
            }
        }
    }
    p
}

/// Every configuration's timelike lines, for one compiled subprocess.
#[derive(Clone, Debug)]
pub struct SubprocessResonances {
    /// Per configuration, its representative diagram's timelike lines, each
    /// line after every line below it.
    configs: Vec<Vec<ResonanceLine>>,
    /// The outgoing legs' PDG codes, for the flavour-daughter rule.
    outgoing: Vec<i32>,
    bwcutoff: f64,
    forced: bool,
}

impl SubprocessResonances {
    /// Resolve the lines of every configuration of `eval`, compiled from
    /// `diagrams`.
    pub fn new(
        eval: &AmplitudeEvaluator,
        diagrams: &[Diagram],
        model: &UFOModel,
        evaluated: &EvaluatedModel,
        bwcutoff: f64,
    ) -> Result<Self, LhefError> {
        let n_in = eval.n_in();
        let outgoing = eval.external_particles()[n_in..]
            .iter()
            .enumerate()
            .map(|(k, &id)| {
                let code = model.particle(id).pdg_code;
                i32::try_from(code).map_err(|_| LhefError::PdgOutOfRange {
                    leg: n_in + k,
                    pdg: code,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut configs = Vec::with_capacity(eval.n_configs());
        for config in 0..eval.n_configs() {
            let lines = match eval.config_diagram(config).and_then(|d| diagrams.get(d)) {
                Some(diagram) => timelike_lines(diagram, model, evaluated)?,
                None => Vec::new(),
            };
            configs.push(lines);
        }
        let forced = configs.iter().flatten().any(|l| l.forced);
        Ok(SubprocessResonances {
            configs,
            outgoing,
            bwcutoff,
            forced,
        })
    }

    /// Whether any configuration carries a line a decay chain forces.
    pub fn has_forced(&self) -> bool {
        self.forced
    }

    /// Configuration `config`'s timelike lines, children first.
    pub fn lines(&self, config: usize) -> &[ResonanceLine] {
        self.configs.get(config).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Whether every forced line of `config` is inside its window at these
    /// outgoing momenta: MadEvent's `cut_bw` on that configuration, which
    /// rejects the point in its channel otherwise.
    pub fn admits(&self, config: usize, outgoing: &[[f64; 4]]) -> bool {
        self.lines(config)
            .iter()
            .filter(|l| l.forced)
            .all(|l| l.in_window(self.bwcutoff, outgoing))
    }

    /// Zero the `AMP2` share of every configuration that does not
    /// [`admit`](Self::admits) the point, unless that would zero them all.
    ///
    /// MadEvent writes an event from a channel whose forced lines are all inside
    /// their windows, so the configuration an event record is drawn in must be
    /// one of those. Every configuration of a decay chain without identical
    /// particles across its decays forces the same lines and admits the same
    /// points, so this changes nothing there; with them, it keeps the pairing
    /// whose windows the point is inside.
    pub fn mask_unadmitted(&self, amp2: &mut [f64], outgoing: &[[f64; 4]]) {
        if !self.forced {
            return;
        }
        let admitted: Vec<bool> = (0..amp2.len()).map(|c| self.admits(c, outgoing)).collect();
        let kept: f64 = amp2
            .iter()
            .zip(&admitted)
            .filter(|(_, &ok)| ok)
            .map(|(w, _)| *w)
            .sum();
        if !(kept > 0.0) {
            return;
        }
        for (w, ok) in amp2.iter_mut().zip(admitted) {
            if !ok {
                *w = 0.0;
            }
        }
    }

    /// The outgoing legs' PDG codes, in the subprocess's own order.
    pub fn outgoing(&self) -> &[i32] {
        &self.outgoing
    }

    /// The lines of `config` MadEvent's `cut_bw` leaves flagged `OnBW` at these
    /// outgoing momenta, as indices into [`lines`](Self::lines), children first.
    pub fn on_shell(&self, config: usize, outgoing: &[[f64; 4]]) -> Vec<usize> {
        let lines = self.lines(config);
        let masses: Vec<f64> = lines.iter().map(|l| l.invariant_mass(outgoing)).collect();
        let mut flagged = vec![false; lines.len()];
        for (i, line) in lines.iter().enumerate() {
            let inside = (masses[i] - line.mass).abs() < self.bwcutoff * line.width
                && (line.width / line.mass < NARROW_WIDTH_RATIO || line.forced);
            if !inside {
                continue;
            }
            flagged[i] = true;
            // The flavour-daughter rule reads the last daughter carrying the line's
            // own flavour, as `cut_bw`'s loop over the two daughters leaves it.
            let mut same: Option<Daughter> = None;
            for daughter in daughters(lines, i) {
                let flavour = match daughter {
                    Daughter::Leg(k) => self.outgoing[k],
                    Daughter::Line(j) => lines[j].pdg,
                };
                if flavour == line.pdg {
                    same = Some(daughter);
                }
            }
            match same {
                None => {}
                Some(Daughter::Leg(_)) => flagged[i] = false,
                Some(Daughter::Line(j)) => {
                    if lines[j].forced {
                        flagged[i] = false;
                    } else if line.forced {
                        flagged[j] = false;
                    } else if (masses[i] - line.mass).abs() > (masses[j] - line.mass).abs() {
                        flagged[i] = false;
                    } else {
                        flagged[j] = false;
                    }
                }
            }
        }
        (0..lines.len()).filter(|&i| flagged[i]).collect()
    }
}

/// A direct daughter of a line: an outgoing leg, or another line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Daughter {
    Leg(usize),
    Line(usize),
}

/// The direct daughters of `lines[i]`: the largest lines strictly below it, and
/// the outgoing legs no such line covers.
fn daughters(lines: &[ResonanceLine], i: usize) -> Vec<Daughter> {
    let slots = lines[i].slots;
    let below: Vec<usize> = (0..lines.len())
        .filter(|&j| j != i && lines[j].slots & !slots == 0 && lines[j].slots != slots)
        .collect();
    let direct: Vec<usize> = below
        .iter()
        .copied()
        .filter(|&j| {
            !below.iter().any(|&k| {
                k != j && lines[j].slots & !lines[k].slots == 0 && lines[k].slots != lines[j].slots
            })
        })
        .collect();
    let covered = direct.iter().fold(0u64, |m, &j| m | lines[j].slots);
    let mut out: Vec<Daughter> = direct.into_iter().map(Daughter::Line).collect();
    out.extend(
        (0..64)
            .filter(|k| slots & !covered & (1u64 << k) != 0)
            .map(Daughter::Leg),
    );
    out
}

/// One diagram's timelike lines as a record reads them, each after every line
/// below it.
fn timelike_lines(
    diagram: &Diagram,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
) -> Result<Vec<ResonanceLine>, LhefError> {
    let n_in = diagram.n_in;
    let mut lines = Vec::new();
    for (i, prop) in diagram.props.iter().enumerate() {
        let Some(code) = oriented_s_channel_id(prop, diagram, model) else {
            continue;
        };
        let Some(side) = diagram.final_state_side(PropIdx(i)) else {
            continue;
        };
        let slots = side
            .iter()
            .filter(|l| l.0 >= n_in)
            .fold(0u64, |m, l| m | (1u64 << (l.0 - n_in)));
        let pdg = i32::try_from(code).map_err(|_| LhefError::PdgOutOfRange {
            leg: diagram.n_ext(),
            pdg: code,
        })?;
        // The representation of the particle as oriented: the conjugate of the
        // propagator's own slot particle's when the orientation flipped it. A
        // singlet or an octet is its own conjugate, whatever sign the model's
        // antiparticle entry carries.
        let particle = model.particle(prop.particle);
        let color = if matches!(particle.color.abs(), 1 | 8) {
            particle.color.abs()
        } else if code == particle.pdg_code {
            particle.color
        } else {
            -particle.color
        };
        let mass = evaluated.mass(prop.particle);
        let width = evaluated.width(prop.particle);
        lines.push(ResonanceLine {
            slots,
            pdg,
            color,
            mass,
            width: if width > 0.0 {
                width.max(mass * SMALL_WIDTH_TREATMENT)
            } else {
                0.0
            },
            forced: prop.onshell == OnShell::Forced,
        });
    }
    lines.sort_by_key(|l| l.slots.count_ones());
    Ok(lines)
}

/// The PDG code a line of a flavour group's representative carries in one of
/// the group's members, whose outgoing legs are `member` where the
/// representative's are `representative`.
///
/// A group joins subprocesses that share a matrix element, so a member's line is
/// the representative's line or its conjugate: the same flavour where the legs
/// below it carry the same charge, the antiparticle where they carry the opposite
/// one. `None` where neither holds, or where a neutral line that is not its own
/// antiparticle leaves the charge unable to tell them apart.
pub fn member_line_pdg(
    line: &ResonanceLine,
    representative: &[i32],
    member: &[i32],
    model: &UFOModel,
) -> Option<i32> {
    let below = |legs: &[i32]| -> Vec<i32> {
        let mut v: Vec<i32> = (0..legs.len())
            .filter(|k| line.slots & (1u64 << k) != 0)
            .map(|k| legs[k])
            .collect();
        v.sort_unstable();
        v
    };
    let rep = below(representative);
    let mem = below(member);
    if rep == mem {
        return Some(line.pdg);
    }
    let conjugate = conjugate_pdg(line.pdg, model)?;
    let mut negated: Vec<i32> = rep
        .iter()
        .map(|&p| conjugate_pdg(p, model).unwrap_or(p))
        .collect();
    negated.sort_unstable();
    if negated == mem {
        return Some(conjugate);
    }
    let charge = |legs: &[i32]| -> Option<f64> { legs.iter().map(|&p| pdg_charge(p, model)).sum() };
    let (qr, qm) = (charge(&rep)?, charge(&mem)?);
    if (qr - qm).abs() < 1e-9 && (conjugate == line.pdg || qr.abs() > 1e-9) {
        Some(line.pdg)
    } else if (qr + qm).abs() < 1e-9 && qr.abs() > 1e-9 {
        Some(conjugate)
    } else {
        None
    }
}

/// The antiparticle's PDG code, or the code itself for a self-conjugate one.
fn conjugate_pdg(pdg: i32, model: &UFOModel) -> Option<i32> {
    let particle = model
        .particles
        .values()
        .find(|p| p.pdg_code == i64::from(pdg.abs()))?;
    Some(if particle.is_self_conjugate {
        pdg
    } else {
        -pdg
    })
}

/// A PDG code's electric charge.
fn pdg_charge(pdg: i32, model: &UFOModel) -> Option<f64> {
    let particle = model
        .particles
        .values()
        .find(|p| p.pdg_code == i64::from(pdg.abs()))?;
    Some(if pdg < 0 {
        -particle.charge
    } else {
        particle.charge
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, DiagramSet};
    use crate::hadronic::compile_class;
    use crate::ufo::sm::{sm_model, SMRestrict};

    /// `p` boosted from its parent's rest frame into the frame where the parent
    /// carries `parent`.
    fn boost(p: [f64; 4], parent: [f64; 4]) -> [f64; 4] {
        let m =
            (parent[0].powi(2) - parent[1].powi(2) - parent[2].powi(2) - parent[3].powi(2)).sqrt();
        let b = [
            parent[1] / parent[0],
            parent[2] / parent[0],
            parent[3] / parent[0],
        ];
        let b2 = b[0] * b[0] + b[1] * b[1] + b[2] * b[2];
        let gamma = parent[0] / m;
        let bp = b[0] * p[1] + b[1] * p[2] + b[2] * p[3];
        let k = if b2 > 0.0 {
            (gamma - 1.0) * bp / b2
        } else {
            0.0
        };
        [
            gamma * (p[0] + bp),
            p[1] + (k + gamma * p[0]) * b[0],
            p[2] + (k + gamma * p[0]) * b[1],
            p[3] + (k + gamma * p[0]) * b[2],
        ]
    }

    /// The two daughters of `parent`, of masses `m1` and `m2`, back to back
    /// along `axis` in its rest frame.
    fn decay(parent: [f64; 4], m1: f64, m2: f64, axis: [f64; 3]) -> ([f64; 4], [f64; 4]) {
        let m =
            (parent[0].powi(2) - parent[1].powi(2) - parent[2].powi(2) - parent[3].powi(2)).sqrt();
        let q = ((m * m - (m1 + m2).powi(2)) * (m * m - (m1 - m2).powi(2))).sqrt() / (2.0 * m);
        let n = (axis[0].powi(2) + axis[1].powi(2) + axis[2].powi(2)).sqrt();
        let d = [axis[0] / n, axis[1] / n, axis[2] / n];
        let a = [(m1 * m1 + q * q).sqrt(), q * d[0], q * d[1], q * d[2]];
        let b = [(m2 * m2 + q * q).sqrt(), -q * d[0], -q * d[1], -q * d[2]];
        (boost(a, parent), boost(b, parent))
    }

    fn subprocess(card: &str) -> (DiagramSet, SubprocessResonances) {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let parsed = parse_proc_card(card, &Default::default()).expect("card");
        let set = generate_from_proc_card(&parsed, &model)
            .expect("diagrams")
            .into_iter()
            .find(|s| !s.diagrams.is_empty())
            .expect("a subprocess");
        let eval = compile_class(&set, &model, &evaluated).expect("compiles");
        let table = SubprocessResonances::new(&eval, &set.diagrams, &model, &evaluated, 15.0)
            .expect("lines");
        (set, table)
    }

    /// `e+ e- > b e+ ve b~ mu- vm~` through two tops at 500 GeV, the `W+` at
    /// `mw_plus` and the `W-` at the pole.
    fn top_pair(mw_plus: f64) -> Vec<[f64; 4]> {
        let (mt, mw, mb) = (173.4, 80.419002445756, 4.7);
        let (t, tb) = decay([500.0, 0.0, 0.0, 0.0], mt, mt, [0.3, 0.4, 0.8]);
        let (b, wp) = decay(t, mb, mw_plus, [0.1, -0.7, 0.2]);
        let (ep, ve) = decay(wp, 0.0, 0.0, [-0.5, 0.2, 0.6]);
        let (bb, wm) = decay(tb, mb, mw, [0.9, 0.1, -0.3]);
        let (mu, vm) = decay(wm, 0.0, 0.0, [0.2, 0.9, 0.1]);
        vec![b, ep, ve, bb, mu, vm]
    }

    /// The forced tops are flagged inside their windows and a free `W` only
    /// inside its own: the window rule MadEvent's `cut_bw` applies, which is what
    /// makes a free line's record come and go from event to event.
    #[test]
    fn a_free_line_is_flagged_only_inside_its_own_window() {
        let (_, table) =
            subprocess("import model sm\ngenerate e+ e- > t t~, t > b e+ ve, t~ > b~ mu- vm~\n");
        assert!(table.has_forced());
        let flagged = |momenta: &[[f64; 4]]| -> Vec<Vec<i32>> {
            (0..table.configs.len())
                .map(|c| {
                    let mut pdg: Vec<i32> = table
                        .on_shell(c, momenta)
                        .into_iter()
                        .map(|i| table.lines(c)[i].pdg)
                        .collect();
                    pdg.sort_unstable();
                    pdg
                })
                .collect()
        };
        let inside = top_pair(80.0);
        assert!((0..table.configs.len()).all(|c| table.admits(c, &inside)));
        for lines in flagged(&inside) {
            assert_eq!(lines, [-24, -6, 6, 24]);
        }
        // 45 GeV is outside the W's 15-width window, [49.7, 111.1] GeV.
        for lines in flagged(&top_pair(45.0)) {
            assert_eq!(lines, [-24, -6, 6]);
        }
        // The top lines carry the W below them and the b beside it.
        let top = table
            .lines(0)
            .iter()
            .find(|l| l.pdg == 6)
            .expect("a top line");
        assert_eq!(top.slots, 0b000111);
        assert_eq!(top.color, 3);
        let antitop = table
            .lines(0)
            .iter()
            .find(|l| l.pdg == -6)
            .expect("an antitop line");
        assert_eq!(
            antitop.color, -3,
            "the conjugate of the propagator's own slot particle"
        );
    }

    /// With identical particles across the decays, the configurations force
    /// different pairings; the ones whose windows the point is outside are
    /// withdrawn from the event's configuration draw.
    #[test]
    fn a_pairing_outside_its_windows_is_not_drawn() {
        let (set, table) = subprocess("import model sm\ngenerate e+ e- > z z, z > e+ e-\n");
        let mz = 91.188;
        let (z1, z2) = decay([500.0, 0.0, 0.0, 0.0], mz, mz, [0.2, 0.5, 0.7]);
        let (a1, a2) = decay(z1, 0.0, 0.0, [0.3, -0.2, 0.9]);
        let (b1, b2) = decay(z2, 0.0, 0.0, [-0.6, 0.4, 0.1]);
        // The legs in the subprocess's own order: e+ e- e+ e-, the first pair
        // from the first Z.
        let names = &set.particles_out;
        assert_eq!(names.iter().filter(|n| *n == "e+").count(), 2);
        let momenta = [a1, a2, b1, b2];
        let admitted: Vec<bool> = (0..table.configs.len())
            .map(|c| table.admits(c, &momenta))
            .collect();
        assert!(admitted.iter().any(|&a| a) && admitted.iter().any(|&a| !a));
        let mut amp2 = vec![1.0; table.configs.len()];
        table.mask_unadmitted(&mut amp2, &momenta);
        for (w, ok) in amp2.iter().zip(&admitted) {
            assert_eq!(*w, if *ok { 1.0 } else { 0.0 });
        }
        // Where no configuration admits the point the draw is left whole.
        let far = [a1, b1, a2, b2];
        if (0..table.configs.len()).all(|c| !table.admits(c, &far)) {
            let mut amp2 = vec![1.0; table.configs.len()];
            table.mask_unadmitted(&mut amp2, &far);
            assert!(amp2.iter().all(|&w| w == 1.0));
        }
    }

    fn line(slots: u64, pdg: i32, width: f64, forced: bool) -> ResonanceLine {
        ResonanceLine {
            slots,
            pdg,
            color: 1,
            mass: 91.188,
            width,
            forced,
        }
    }

    /// `cut_bw`'s flavour-daughter rule, on two nested `Z` lines both inside
    /// their windows: the one further from the pole loses unless a decay chain
    /// forces one of them, which then wins; a line decaying into an external leg
    /// of its own flavour is never flagged; and a broad line is flagged only
    /// when forced.
    #[test]
    fn the_flavour_daughter_rule_is_madevents() {
        // Legs: e+ e- (the inner Z at 91.0 GeV, 0.19 from the pole), then a
        // photon, which puts the outer Z at 91.99 GeV, 0.81 from it.
        let outgoing = [
            [45.5, 0.0, 0.0, 45.5],
            [45.5, 0.0, 0.0, -45.5],
            [1.0, 1.0, 0.0, 0.0],
        ];
        let table = |inner_forced: bool, outer_forced: bool| SubprocessResonances {
            configs: vec![vec![
                line(0b011, 23, 2.44, inner_forced),
                line(0b111, 23, 2.44, outer_forced),
            ]],
            outgoing: vec![-11, 11, 22],
            bwcutoff: 15.0,
            forced: inner_forced || outer_forced,
        };
        let flagged = |t: &SubprocessResonances| t.on_shell(0, &outgoing);
        // Neither forced: the outer one, further from the pole, loses.
        assert_eq!(flagged(&table(false, false)), [0]);
        assert_eq!(flagged(&table(false, true)), [1]);
        assert_eq!(flagged(&table(true, false)), [0]);
        let own_leg = SubprocessResonances {
            configs: vec![vec![line(0b011, 22, 2.44, false)]],
            outgoing: vec![22, 11, 22],
            bwcutoff: 15.0,
            forced: false,
        };
        assert!(own_leg.on_shell(0, &outgoing).is_empty());
        let broad = |forced| SubprocessResonances {
            configs: vec![vec![line(0b011, 23, 12.0, forced)]],
            outgoing: vec![-11, 11, 22],
            bwcutoff: 15.0,
            forced,
        };
        assert!(broad(false).on_shell(0, &outgoing).is_empty());
        assert_eq!(broad(true).on_shell(0, &outgoing), [0]);
    }

    /// A member of a flavour group names the representative's line, or its
    /// antiparticle where its legs below the line carry the opposite charge.
    #[test]
    fn a_member_line_keeps_or_conjugates_the_representatives_flavour() {
        let model = sm_model(SMRestrict::Default);
        let top = ResonanceLine {
            slots: 0b0111,
            pdg: 6,
            color: 3,
            mass: 173.0,
            width: 1.49,
            forced: true,
        };
        let rep = [5, -11, 12, 22];
        assert_eq!(member_line_pdg(&top, &rep, &rep, &model), Some(6));
        assert_eq!(
            member_line_pdg(&top, &rep, &[-5, 11, -12, 22], &model),
            Some(-6)
        );
        let w = ResonanceLine {
            slots: 0b0011,
            pdg: 24,
            color: 1,
            mass: 80.4,
            width: 2.05,
            forced: true,
        };
        assert_eq!(
            member_line_pdg(&w, &[2, -1, 21], &[4, -3, 21], &model),
            Some(24)
        );
        assert_eq!(
            member_line_pdg(&w, &[2, -1, 21], &[1, -2, 21], &model),
            Some(-24)
        );
        // Legs whose charge neither matches nor opposes the line's name no
        // flavour of it.
        assert_eq!(
            member_line_pdg(&w, &[2, -1, 21], &[2, -2, 21], &model),
            None
        );
    }
}
