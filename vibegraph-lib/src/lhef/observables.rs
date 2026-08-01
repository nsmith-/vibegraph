//! Observables read off an event record, for comparing two samples of the same
//! process.
//!
//! Two generators agree on the physics of a process and on nothing about the
//! order they happen to list identical particles in, or which of two same-flavour
//! leptons they wrote first. So a comparison needs a *canonical* view of an
//! event — [`canonical`] — in which the external legs sit in an order derived
//! from the event itself, and every observable below is named after that order.
//! Two files then produce the same observable names for the same physics.
//!
//! # What is canonicalised and what deliberately is not
//!
//! The **final state** is reordered: by leg label, then by decreasing transverse
//! momentum inside a label. That is what makes `g g > g g` comparable at all, and
//! it is why a per-leg observable is a statement about "the harder gluon" rather
//! than about a slot in a file.
//!
//! The **incoming legs are left alone**. Beam 1 runs along `+z` and beam 2 along
//! `−z`, so which flavour sits on which beam is a physical property of the event,
//! not a convention: sorting the initial state would merge `u u~ > X` with
//! `u~ u > X` and hide exactly the mirrored beam ordering a hadron-collider group
//! has to get right. [`canonical`] asserts nothing about the beams; a caller that
//! wants the `pz` ordering checked should check it.
//!
//! Legs the record lists as intermediate resonances (`ISTUP = 2`) are dropped:
//! they are a bookkeeping choice of the writer — MadGraph lists the `Z` of
//! `e+ e- > mu+ mu-` and nothing of `u u~ > u u~` — and no external observable
//! depends on them.
//!
//! # Labels: fine and coarse
//!
//! A leg's label is its particle species ([`Labelling::Fine`]) or the class the
//! process definition grouped it into ([`Labelling::Coarse`]): `l+` for any
//! positively charged lepton, `j` for any light parton. A single concrete
//! subprocess has one final-state species multiset and can use fine labels; a
//! flavour group cannot — the same slot is a `g` in one event and a `u~` in the
//! next — and coarse labels are what keep its observable names stable. Choosing
//! between them is the caller's, because it depends on the sample, not on one
//! event.

use super::record::{LheEvent, LheParticle, STATUS_INCOMING, STATUS_OUTGOING};

/// How finely a leg's species is named.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Labelling {
    /// The species itself: `u`, `u~`, `g`, `e+`.
    Fine,
    /// The class a multiparticle label groups it into: `j`, `l+`, `v`.
    Coarse,
}

/// The label a PDG code carries under a labelling.
///
/// The coarse classes follow MadGraph's cut classes (`setcuts.f`): a light parton
/// is a jet, `b` and `t` stand apart because their masses do, and the charged
/// leptons and neutrinos are grouped by charge sign. Anything unrecognised keeps
/// its numeric code, so a new species is visible rather than silently merged.
pub fn leg_label(pdg: i32, labelling: Labelling) -> String {
    if labelling == Labelling::Fine {
        let name = match pdg.abs() {
            1 => "d",
            2 => "u",
            3 => "s",
            4 => "c",
            5 => "b",
            6 => "t",
            11 => "e",
            12 => "ve",
            13 => "mu",
            14 => "vm",
            15 => "ta",
            16 => "vt",
            21 => return "g".to_string(),
            22 => return "a".to_string(),
            23 => return "z".to_string(),
            24 => return if pdg > 0 { "w+" } else { "w-" }.to_string(),
            25 => return "h".to_string(),
            _ => return format!("pdg{pdg}"),
        };
        // Leptons carry a charge sign; quarks carry a bar.
        return match pdg.abs() {
            11..=16 => format!("{name}{}", if pdg > 0 { "-" } else { "+" }),
            _ => format!("{name}{}", if pdg > 0 { "" } else { "~" }),
        };
    }
    match pdg {
        21 => "j".to_string(),
        1..=4 | -4..=-1 => "j".to_string(),
        5 => "b".to_string(),
        -5 => "b~".to_string(),
        6 => "t".to_string(),
        -6 => "t~".to_string(),
        11 | 13 | 15 => "l-".to_string(),
        -11 | -13 | -15 => "l+".to_string(),
        12 | 14 | 16 => "v".to_string(),
        -12 | -14 | -16 => "v~".to_string(),
        22 => "a".to_string(),
        23 => "z".to_string(),
        24 => "w+".to_string(),
        -24 => "w-".to_string(),
        25 => "h".to_string(),
        other => format!("pdg{other}"),
    }
}

fn pt(p: &LheParticle) -> f64 {
    (p.momentum[1] * p.momentum[1] + p.momentum[2] * p.momentum[2]).sqrt()
}

fn rapidity(p: &LheParticle) -> f64 {
    let (e, pz) = (p.momentum[0], p.momentum[3]);
    let num = (e + pz).max(f64::MIN_POSITIVE);
    let den = (e - pz).max(f64::MIN_POSITIVE);
    0.5 * (num / den).ln()
}

fn cos_theta(p: &LheParticle) -> f64 {
    let m = &p.momentum;
    let mag = (m[1] * m[1] + m[2] * m[2] + m[3] * m[3]).sqrt();
    if mag > 0.0 {
        m[3] / mag
    } else {
        0.0
    }
}

fn mass(momenta: [[f64; 4]; 2]) -> f64 {
    let s: Vec<f64> = (0..4).map(|k| momenta[0][k] + momenta[1][k]).collect();
    let m2 = s[0] * s[0] - s[1] * s[1] - s[2] * s[2] - s[3] * s[3];
    if m2 > 0.0 {
        m2.sqrt()
    } else {
        0.0
    }
}

/// The same event with its final state in canonical order and its intermediate
/// resonances dropped. The incoming legs keep their record order and their
/// beams.
pub fn canonical(event: &LheEvent, labelling: Labelling) -> LheEvent {
    let mut particles: Vec<LheParticle> = event
        .particles
        .iter()
        .filter(|p| p.status == STATUS_INCOMING)
        .copied()
        .collect();
    let mut out: Vec<LheParticle> = event
        .particles
        .iter()
        .filter(|p| p.status == STATUS_OUTGOING)
        .copied()
        .collect();
    out.sort_by(|a, b| {
        leg_label(a.pdg, labelling)
            .cmp(&leg_label(b.pdg, labelling))
            .then_with(|| pt(b).total_cmp(&pt(a)))
    });
    particles.append(&mut out);
    LheEvent {
        particles,
        trailer: Vec::new(),
        // The legs are reordered, so the file's own lines no longer describe
        // this record.
        source: None,
        ..event.clone()
    }
}

/// The final-state legs of a canonical event, with the names every observable
/// below is built from: the label, suffixed by a 1-based rank when the label
/// occurs more than once.
pub fn final_state_names(event: &LheEvent, labelling: Labelling) -> Vec<String> {
    let labels: Vec<String> = event
        .particles
        .iter()
        .filter(|p| p.status == STATUS_OUTGOING)
        .map(|p| leg_label(p.pdg, labelling))
        .collect();
    let mut seen: Vec<(String, usize)> = Vec::new();
    let mut names = Vec::with_capacity(labels.len());
    for label in &labels {
        let count = labels.iter().filter(|l| *l == label).count();
        let rank = match seen.iter_mut().find(|(l, _)| l == label) {
            Some((_, n)) => {
                *n += 1;
                *n
            }
            None => {
                seen.push((label.clone(), 1));
                1
            }
        };
        names.push(if count > 1 {
            format!("{label}{rank}")
        } else {
            label.clone()
        });
    }
    names
}

/// The named continuous observables of one event, in a fixed order.
///
/// Per final-state leg: transverse momentum, rapidity and polar cosine. Per
/// unordered pair: invariant mass. The azimuth of the first leg, which nothing
/// else covers and which every correct sampler makes uniform. And, when the final
/// state holds exactly one opposite-sign same-labelled charged-lepton pair, that
/// pair's transverse momentum, rapidity and Collins–Soper polar cosine.
///
/// The event must already be [`canonical`]; the names come from
/// [`final_state_names`].
pub fn kinematics(event: &LheEvent, labelling: Labelling) -> Vec<(String, f64)> {
    let names = final_state_names(event, labelling);
    let legs: Vec<&LheParticle> = event
        .particles
        .iter()
        .filter(|p| p.status == STATUS_OUTGOING)
        .collect();
    let mut obs = Vec::new();
    for (name, leg) in names.iter().zip(&legs) {
        obs.push((format!("pt({name})"), pt(leg)));
        obs.push((format!("y({name})"), rapidity(leg)));
        obs.push((format!("cos({name})"), cos_theta(leg)));
    }
    if let Some(first) = legs.first() {
        let phi = first.momentum[2].atan2(first.momentum[1]);
        obs.push((format!("phi({})/pi", names[0]), phi / std::f64::consts::PI));
    }
    for i in 0..legs.len() {
        for j in (i + 1)..legs.len() {
            obs.push((
                format!("m({},{})", names[i], names[j]),
                mass([legs[i].momentum, legs[j].momentum]),
            ));
        }
    }
    if let Some((minus, plus)) = charged_lepton_pair(&legs) {
        let q: Vec<f64> = (0..4)
            .map(|k| minus.momentum[k] + plus.momentum[k])
            .collect();
        let qt = (q[1] * q[1] + q[2] * q[2]).sqrt();
        obs.push(("pt(ll)".to_string(), qt));
        obs.push((
            "y(ll)".to_string(),
            0.5 * ((q[0] + q[3]).max(f64::MIN_POSITIVE) / (q[0] - q[3]).max(f64::MIN_POSITIVE))
                .ln(),
        ));
        obs.push(("cs_cos(ll)".to_string(), collins_soper(minus, plus)));
    }
    obs
}

/// The one opposite-sign, same-coarse-label charged-lepton pair of a final state,
/// as `(ℓ⁻, ℓ⁺)`, or `None` when there is not exactly one.
fn charged_lepton_pair<'a>(legs: &[&'a LheParticle]) -> Option<(&'a LheParticle, &'a LheParticle)> {
    let minus: Vec<&LheParticle> = legs
        .iter()
        .copied()
        .filter(|p| matches!(p.pdg, 11 | 13 | 15))
        .collect();
    let plus: Vec<&LheParticle> = legs
        .iter()
        .copied()
        .filter(|p| matches!(p.pdg, -11 | -13 | -15))
        .collect();
    if minus.len() == 1 && plus.len() == 1 {
        Some((minus[0], plus[0]))
    } else {
        None
    }
}

/// The Collins–Soper polar cosine of a lepton pair.
///
/// The frame bisects the two beam directions in the pair's rest frame, which is
/// what makes the angle well defined once the pair carries transverse momentum
/// (Collins and Soper, *Phys. Rev.* **D16** (1977) 2219). In light-cone
/// components `p^± = (E ± p_z)/√2` the closed form is
///
/// ```text
/// cos θ_CS = 2 (p₁⁺p₂⁻ − p₁⁻p₂⁺) / (Q √(Q² + Q_T²)) · sign(Q_z)
/// ```
///
/// with `1 = ℓ⁻`, `2 = ℓ⁺`. The `sign(Q_z)` factor is the hadron-collider
/// convention that orients the axis along the boost of the pair, which is the
/// only way to name a "forward" direction when both beams are protons; at
/// `Q_z = 0` the sign is taken positive, an arbitrary but measure-zero choice.
pub fn collins_soper(minus: &LheParticle, plus: &LheParticle) -> f64 {
    let lc = |p: &LheParticle| {
        (
            (p.momentum[0] + p.momentum[3]) / std::f64::consts::SQRT_2,
            (p.momentum[0] - p.momentum[3]) / std::f64::consts::SQRT_2,
        )
    };
    let (m_plus_lc, m_minus_lc) = lc(minus);
    let (p_plus_lc, p_minus_lc) = lc(plus);
    let q: Vec<f64> = (0..4)
        .map(|k| minus.momentum[k] + plus.momentum[k])
        .collect();
    let q2 = q[0] * q[0] - q[1] * q[1] - q[2] * q[2] - q[3] * q[3];
    if q2 <= 0.0 {
        return 0.0;
    }
    let qq = q2.sqrt();
    let qt2 = q[1] * q[1] + q[2] * q[2];
    let denom = qq * (q2 + qt2).sqrt();
    if denom <= 0.0 {
        return 0.0;
    }
    let sign = if q[3] < 0.0 { -1.0 } else { 1.0 };
    2.0 * (m_plus_lc * p_minus_lc - m_minus_lc * p_plus_lc) / denom * sign
}

/// The event's flavour assignment: the PDG code of every leg of a canonical
/// event, incoming first.
pub fn flavour_key(event: &LheEvent) -> String {
    join(event.particles.iter().map(|p| p.pdg.to_string()))
}

/// The event's helicity assignment: `SPINUP` for every leg of a canonical event.
///
/// A helicity is written as a real and is `±1`, `0` for the longitudinal state of
/// a massive vector, or `9` for a leg whose helicity was summed over. It is
/// rendered here at one decimal so that the key is exact rather than
/// format-dependent.
pub fn helicity_key(event: &LheEvent) -> String {
    join(event.particles.iter().map(|p| format!("{:.1}", p.spin)))
}

/// The event's colour flow, as the partition of leg slots the colour labels
/// induce — blind to a relabelling, which carries no information, and to nothing
/// else.
pub fn colour_key(event: &LheEvent) -> String {
    join(
        event
            .color_connectivity()
            .iter()
            .map(|line| join(line.iter().map(|(leg, slot)| format!("{leg}.{slot}")))),
    )
}

fn join(parts: impl IntoIterator<Item = String>) -> String {
    parts.into_iter().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lhef::record::{LheEvent, LheParticle, STATUS_INTERMEDIATE};

    fn particle(pdg: i32, momentum: [f64; 4], status: i32) -> LheParticle {
        LheParticle {
            pdg,
            status,
            mothers: [0, 0],
            color: [0, 0],
            momentum,
            mass: 0.0,
            lifetime: 0.0,
            spin: 1.0,
        }
    }

    fn event(particles: Vec<LheParticle>) -> LheEvent {
        LheEvent {
            process_id: 1,
            weight: 1.0,
            scale: 91.2,
            alpha_qed: 0.0075,
            alpha_qcd: 0.118,
            particles,
            trailer: Vec::new(),
            source: None,
        }
    }

    /// Two events that differ only in the order two identical particles were
    /// written in are the same event, and must produce the same observables.
    #[test]
    fn identical_particles_are_ordered_by_transverse_momentum() {
        let soft = particle(21, [50.0, 10.0, 0.0, 48.0], STATUS_OUTGOING);
        let hard = particle(21, [50.0, 30.0, 0.0, 40.0], STATUS_OUTGOING);
        let beams = [
            particle(21, [50.0, 0.0, 0.0, 50.0], STATUS_INCOMING),
            particle(21, [50.0, 0.0, 0.0, -50.0], STATUS_INCOMING),
        ];
        let a = event(vec![beams[0], beams[1], soft, hard]);
        let b = event(vec![beams[0], beams[1], hard, soft]);
        let (ca, cb) = (
            canonical(&a, Labelling::Fine),
            canonical(&b, Labelling::Fine),
        );
        assert_eq!(
            final_state_names(&ca, Labelling::Fine),
            vec!["g1".to_string(), "g2".to_string()]
        );
        assert_eq!(
            kinematics(&ca, Labelling::Fine),
            kinematics(&cb, Labelling::Fine)
        );
        // g1 is the harder one whichever way the file listed them.
        let pt_g1 = kinematics(&ca, Labelling::Fine)
            .into_iter()
            .find(|(n, _)| n == "pt(g1)")
            .unwrap()
            .1;
        assert!((pt_g1 - 30.0).abs() < 1e-12);
    }

    /// Which beam a flavour sits on is physics, so the initial state is not
    /// reordered and two mirrored events are distinguishable.
    #[test]
    fn the_initial_state_keeps_its_beams() {
        let out = [
            particle(5, [50.0, 20.0, 0.0, 40.0], STATUS_OUTGOING),
            particle(-5, [50.0, -20.0, 0.0, -40.0], STATUS_OUTGOING),
        ];
        let forward = event(vec![
            particle(2, [50.0, 0.0, 0.0, 50.0], STATUS_INCOMING),
            particle(-2, [50.0, 0.0, 0.0, -50.0], STATUS_INCOMING),
            out[0],
            out[1],
        ]);
        let mirrored = event(vec![
            particle(-2, [50.0, 0.0, 0.0, 50.0], STATUS_INCOMING),
            particle(2, [50.0, 0.0, 0.0, -50.0], STATUS_INCOMING),
            out[0],
            out[1],
        ]);
        assert_ne!(
            flavour_key(&canonical(&forward, Labelling::Fine)),
            flavour_key(&canonical(&mirrored, Labelling::Fine))
        );
    }

    /// An intermediate resonance is a writer's bookkeeping and leaves no trace in
    /// the observables.
    #[test]
    fn intermediate_resonances_are_dropped() {
        let beams = [
            particle(-11, [50.0, 0.0, 0.0, 50.0], STATUS_INCOMING),
            particle(11, [50.0, 0.0, 0.0, -50.0], STATUS_INCOMING),
        ];
        let out = [
            particle(-13, [50.0, 30.0, 0.0, 40.0], STATUS_OUTGOING),
            particle(13, [50.0, -30.0, 0.0, -40.0], STATUS_OUTGOING),
        ];
        let bare = event(vec![beams[0], beams[1], out[0], out[1]]);
        let with_z = event(vec![
            beams[0],
            beams[1],
            particle(23, [100.0, 0.0, 0.0, 0.0], STATUS_INTERMEDIATE),
            out[0],
            out[1],
        ]);
        assert_eq!(
            kinematics(&canonical(&bare, Labelling::Fine), Labelling::Fine),
            kinematics(&canonical(&with_z, Labelling::Fine), Labelling::Fine)
        );
        assert_eq!(
            flavour_key(&canonical(&with_z, Labelling::Fine)),
            "-11 11 -13 13"
        );
    }

    /// Coarse labels merge what a multiparticle definition merged, so a group
    /// whose events carry different species still produces one set of names.
    #[test]
    fn coarse_labels_make_a_flavour_group_comparable() {
        let with_electrons = event(vec![
            particle(2, [50.0, 0.0, 0.0, 50.0], STATUS_INCOMING),
            particle(-2, [50.0, 0.0, 0.0, -50.0], STATUS_INCOMING),
            particle(-11, [50.0, 30.0, 0.0, 40.0], STATUS_OUTGOING),
            particle(11, [50.0, -30.0, 0.0, -40.0], STATUS_OUTGOING),
        ]);
        let with_muons = event(vec![
            particle(2, [50.0, 0.0, 0.0, 50.0], STATUS_INCOMING),
            particle(-2, [50.0, 0.0, 0.0, -50.0], STATUS_INCOMING),
            particle(-13, [50.0, 30.0, 0.0, 40.0], STATUS_OUTGOING),
            particle(13, [50.0, -30.0, 0.0, -40.0], STATUS_OUTGOING),
        ]);
        let names =
            |e: &LheEvent| final_state_names(&canonical(e, Labelling::Coarse), Labelling::Coarse);
        assert_eq!(names(&with_electrons), names(&with_muons));
        assert_eq!(
            names(&with_electrons),
            vec!["l+".to_string(), "l-".to_string()]
        );
        assert_ne!(
            final_state_names(
                &canonical(&with_electrons, Labelling::Fine),
                Labelling::Fine
            ),
            final_state_names(&canonical(&with_muons, Labelling::Fine), Labelling::Fine)
        );
    }

    /// At zero pair transverse momentum the Collins–Soper angle is the polar
    /// angle of the ℓ⁻ in the pair rest frame, so a back-to-back pair along the
    /// beam gives ±1 and a transverse one gives 0.
    #[test]
    fn collins_soper_reduces_to_the_rest_frame_polar_angle() {
        let along = collins_soper(
            &particle(13, [50.0, 0.0, 0.0, 50.0], STATUS_OUTGOING),
            &particle(-13, [50.0, 0.0, 0.0, -50.0], STATUS_OUTGOING),
        );
        assert!((along - 1.0).abs() < 1e-12, "cos = {along}");
        let against = collins_soper(
            &particle(13, [50.0, 0.0, 0.0, -50.0], STATUS_OUTGOING),
            &particle(-13, [50.0, 0.0, 0.0, 50.0], STATUS_OUTGOING),
        );
        assert!((against + 1.0).abs() < 1e-12, "cos = {against}");
        let transverse = collins_soper(
            &particle(13, [50.0, 50.0, 0.0, 0.0], STATUS_OUTGOING),
            &particle(-13, [50.0, -50.0, 0.0, 0.0], STATUS_OUTGOING),
        );
        assert!(transverse.abs() < 1e-12, "cos = {transverse}");
    }

    /// A boosted pair keeps its rest-frame angle, which is what the frame is for.
    #[test]
    fn collins_soper_is_invariant_under_a_boost_along_the_beam() {
        let boost = |p: LheParticle, rapidity: f64| {
            let (c, s) = (rapidity.cosh(), rapidity.sinh());
            let (e, pz) = (p.momentum[0], p.momentum[3]);
            LheParticle {
                momentum: [c * e + s * pz, p.momentum[1], p.momentum[2], s * e + c * pz],
                ..p
            }
        };
        let minus = particle(13, [50.0, 30.0, 0.0, 40.0], STATUS_OUTGOING);
        let plus = particle(-13, [50.0, -30.0, 0.0, -40.0], STATUS_OUTGOING);
        let rest = collins_soper(&minus, &plus);
        let boosted = collins_soper(&boost(minus, 0.7), &boost(plus, 0.7));
        assert!(
            (rest - boosted).abs() < 1e-12,
            "{rest} against {boosted} after a boost"
        );
    }

    /// The colour key is the connectivity, so a relabelling is invisible and a
    /// reconnection is not.
    #[test]
    fn the_colour_key_sees_connectivity_and_not_labels() {
        let with = |c1: [i32; 2], c2: [i32; 2]| {
            event(vec![
                LheParticle {
                    color: c1,
                    ..particle(21, [50.0, 0.0, 0.0, 50.0], STATUS_INCOMING)
                },
                LheParticle {
                    color: c2,
                    ..particle(21, [50.0, 0.0, 0.0, -50.0], STATUS_INCOMING)
                },
            ])
        };
        assert_eq!(
            colour_key(&with([501, 502], [502, 501])),
            colour_key(&with([601, 602], [602, 601]))
        );
        assert_ne!(
            colour_key(&with([501, 502], [502, 501])),
            colour_key(&with([501, 502], [501, 502]))
        );
    }
}
