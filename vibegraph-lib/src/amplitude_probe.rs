//! The matrix element an integration was computed with, recorded at a few fixed
//! phase-space points per subprocess, so a later replay can tell whether the
//! running build still computes the same |M|².
//!
//! An integrate artifact's cross section describes the model evaluated at the
//! banked parameters *as compiled by the integrating binary*. A compiler fix, a
//! change to how internal parameters or couplings are derived, or simply running
//! the artifact under another build can move |M|² with no change to the
//! artifact's encoding, so no format version sees it. The probe does: each
//! diagram set is compiled and evaluated at [`PROBE_POINTS`] banked momenta
//! under the bound parameter card, and the replay recomputes them and refuses a
//! relative deviation above [`PROBE_REL_TOL`].
//!
//! The momenta are banked with the values. A replay therefore depends on
//! neither the RNG nor the point construction below staying the same; they
//! decide only which points a new integration banks.
//!
//! **Points.** Partonic centre of mass, the two incoming legs along ±z (the
//! frame a helicity-pruned evaluator requires), or the decaying particle at rest
//! for a `1 → n` set. A scattering set is probed at
//! `√ŝ = max(`[`PROBE_THRESHOLD_MULTIPLE`]` × threshold, `[`PROBE_MIN_SQRT_S`]`)`,
//! the threshold being the larger of the incoming and outgoing mass sums, so
//! every point sits well above threshold and away from resonance peaks of the
//! model's lighter states; a decay is probed at the parent's mass. The outgoing
//! momenta are massive RAMBO from a fixed-seed ChaCha8 stream, and a draw is
//! kept only when every pair of external momenta has `2 pᵢ·pⱼ` of at least
//! [`PROBE_MIN_PAIR`]` × ŝ`: no soft or collinear configuration, where |M|² is
//! large and badly conditioned, and no exactly symmetric one.
//!
//! **Couplings.** The probe evaluates at the parameter card's own couplings,
//! including its `α_s`. A change to the running of `α_s` with the event's scale,
//! to the cuts or to the phase-space maps is outside what it sees.
//!
//! **Blind spots.** A change to |M|² that leaves every probed point unmoved: one
//! confined to regions the four generic points miss (a single helicity that
//! vanishes at all of them, a numerically unstable corner). A one-body final
//! state (`2 → 1`) has no point to probe and records none; such a subprocess is
//! never integrated.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::diagrams::DiagramSet;
use crate::hadronic::{compile_class, HadronicError};
use crate::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
use crate::helas::repr::lorentz::LorentzVector;
use crate::phasespace::rambo::rambo_massive;
use crate::ufo::{EvaluatedModel, UFOModel};

/// Points banked per diagram set.
pub const PROBE_POINTS: usize = 4;

/// The largest relative deviation between a banked and a recomputed |M|² a
/// replay accepts.
///
/// About five orders of magnitude above the last-ulp differences between system
/// libms on the summed |M|², and far below any convention or coupling error. A
/// reduced-precision evaluator would need its own value, set from its measured
/// error.
pub const PROBE_REL_TOL: f64 = 1e-10;

/// `√ŝ` of a scattering set's probe points as a multiple of its mass threshold.
pub const PROBE_THRESHOLD_MULTIPLE: f64 = 3.3;

/// The smallest `√ŝ` (GeV) a scattering set is probed at, for a set whose mass
/// threshold is small or zero.
pub const PROBE_MIN_SQRT_S: f64 = 1234.5;

/// The smallest `2 pᵢ·pⱼ / ŝ` over every pair of external momenta a probe point
/// may have.
pub const PROBE_MIN_PAIR: f64 = 1e-3;

/// The seed of the RAMBO stream a set's probe points are drawn from.
const PROBE_SEED: u64 = 0x9E0B_E5E7;

/// Draws a set may spend finding [`PROBE_POINTS`] points that pass
/// [`PROBE_MIN_PAIR`].
const MAX_DRAWS: usize = 10_000;

/// One banked point: the external momenta, incoming first, each `[E, px, py,
/// pz]` in GeV, and the colour- and helicity-summed |M|² there (not averaged).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProbePoint {
    pub momenta: Vec<[f64; 4]>,
    pub m2: f64,
}

/// One diagram set's banked points, in draw order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetProbe {
    pub points: Vec<ProbePoint>,
}

#[derive(Debug, Error)]
pub enum ProbeError {
    #[error("cannot compile {label} for the amplitude probe: {source}")]
    Compile {
        label: String,
        source: HadronicError,
    },
    #[error("cannot place amplitude probe points for {label}: {why}")]
    NoPoints { label: String, why: String },
    #[error(
        "the artifact banks amplitude probes for {banked} diagram sets and holds {sets}; the \
         file is corrupt or was edited (integrate again)"
    )]
    SetCount { banked: usize, sets: usize },
    #[error(
        "the amplitude probe of set {set} ({label}), point {point}, has {got} momenta where the \
         set has {want} external legs; the file is corrupt or was edited (integrate again)"
    )]
    Legs {
        set: usize,
        label: String,
        point: usize,
        got: usize,
        want: usize,
    },
    #[error(
        "|M|² of set {set} ({label}) at amplitude probe point {point} is {computed:e} in this \
         build and {banked:e} in the artifact, a relative deviation of {deviation:e} against a \
         tolerance of {PROBE_REL_TOL:e}: this build computes a different matrix element from \
         the one the cross section and grids were integrated with (a compiler, coupling or \
         parameter change since); integrate again with this build"
    )]
    Moved {
        set: usize,
        label: String,
        point: usize,
        banked: f64,
        computed: f64,
        deviation: f64,
    },
}

/// `|a − b| / max(|a|, |b|)`, zero when both are zero. Any non-finite operand
/// that is not bit-identical to the other deviates infinitely.
pub fn relative_deviation(a: f64, b: f64) -> f64 {
    if a.to_bits() == b.to_bits() {
        return 0.0;
    }
    if !a.is_finite() || !b.is_finite() {
        return f64::INFINITY;
    }
    let scale = a.abs().max(b.abs());
    if scale == 0.0 {
        0.0
    } else {
        (a - b).abs() / scale
    }
}

fn vector(p: &[f64; 4]) -> LorentzVector<f64> {
    LorentzVector::new(p[0], p[1], p[2], p[3])
}

/// The external momenta of `PROBE_POINTS` points for a process with
/// `n_in` incoming legs of the given `masses` (incoming first), or why there are
/// none: empty for a one-body final state.
fn probe_momenta(n_in: usize, masses: &[f64]) -> Result<Vec<Vec<[f64; 4]>>, String> {
    let (m_in, m_out) = masses.split_at(n_in);
    if m_out.len() < 2 {
        return Ok(Vec::new());
    }
    let out_sum: f64 = m_out.iter().sum();
    let (sqrt_s, incoming) = match m_in {
        [parent] => {
            if *parent <= out_sum {
                return Err(format!(
                    "the decaying particle's mass {parent} GeV is not above its products' \
                     {out_sum} GeV"
                ));
            }
            (*parent, vec![[*parent, 0.0, 0.0, 0.0]])
        }
        [m1, m2] => {
            let threshold = (m1 + m2).max(out_sum);
            let sqrt_s = (PROBE_THRESHOLD_MULTIPLE * threshold).max(PROBE_MIN_SQRT_S);
            let s = sqrt_s * sqrt_s;
            let e1 = (s + m1 * m1 - m2 * m2) / (2.0 * sqrt_s);
            let pz = (e1 * e1 - m1 * m1).max(0.0).sqrt();
            (
                sqrt_s,
                vec![[e1, 0.0, 0.0, pz], [sqrt_s - e1, 0.0, 0.0, -pz]],
            )
        }
        _ => return Err(format!("{} incoming legs", m_in.len())),
    };
    let s = sqrt_s * sqrt_s;
    let mut rng = ChaCha8Rng::seed_from_u64(PROBE_SEED);
    let mut points = Vec::with_capacity(PROBE_POINTS);
    for _ in 0..MAX_DRAWS {
        let mut point = incoming.clone();
        point.extend(
            rambo_massive(sqrt_s, m_out, &mut rng)
                .into_iter()
                .map(|p| [p.e(), p.px(), p.py(), p.pz()]),
        );
        if generic(&point, s) {
            points.push(point);
            if points.len() == PROBE_POINTS {
                return Ok(points);
            }
        }
    }
    Err(format!(
        "{MAX_DRAWS} draws at sqrt(s) = {sqrt_s} GeV gave {} of {PROBE_POINTS} points with every \
         2 p_i.p_j above {PROBE_MIN_PAIR} s",
        points.len()
    ))
}

/// Whether every pair of the point's momenta has `2 pᵢ·pⱼ ≥ PROBE_MIN_PAIR · s`.
fn generic(point: &[[f64; 4]], s: f64) -> bool {
    let dot = |a: &[f64; 4], b: &[f64; 4]| a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3];
    point.iter().enumerate().all(|(i, a)| {
        point[i + 1..]
            .iter()
            .all(|b| 2.0 * dot(a, b) >= PROBE_MIN_PAIR * s)
    })
}

/// |M|² of a compiled set at each of `points`.
fn evaluate(
    evaluator: &AmplitudeEvaluator,
    evaluated: &EvaluatedModel,
    points: &[Vec<[f64; 4]>],
) -> Vec<f64> {
    let bound = BoundAmplitude::<f64>::bind(evaluator, evaluated);
    let mut scratch = bound.scratch_space();
    points
        .iter()
        .map(|p| {
            let momenta: Vec<LorentzVector<f64>> = p.iter().map(vector).collect();
            bound.eval_m2(&momenta, &mut scratch)
        })
        .collect()
}

fn compile(
    set: &DiagramSet,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
) -> Result<AmplitudeEvaluator, ProbeError> {
    compile_class(set, model, evaluated).map_err(|source| ProbeError::Compile {
        label: set.label(),
        source,
    })
}

/// Compile every set of `sets` and bank |M|² at its probe points under
/// `evaluated`, one [`SetProbe`] per set in set order.
pub fn measure(
    sets: &[DiagramSet],
    model: &UFOModel,
    evaluated: &EvaluatedModel,
) -> Result<Vec<SetProbe>, ProbeError> {
    sets.iter()
        .map(|set| {
            let evaluator = compile(set, model, evaluated)?;
            let masses: Vec<f64> = evaluator
                .external_particles()
                .iter()
                .map(|&id| evaluated.mass(id))
                .collect();
            let momenta =
                probe_momenta(evaluator.n_in(), &masses).map_err(|why| ProbeError::NoPoints {
                    label: set.label(),
                    why,
                })?;
            let m2 = evaluate(&evaluator, evaluated, &momenta);
            Ok(SetProbe {
                points: momenta
                    .into_iter()
                    .zip(m2)
                    .map(|(momenta, m2)| ProbePoint { momenta, m2 })
                    .collect(),
            })
        })
        .collect()
}

/// Recompute every banked point of `banked` with `m2_of(set index, momenta)` and
/// refuse the first whose relative deviation exceeds [`PROBE_REL_TOL`].
pub(crate) fn check_with(
    banked: &[SetProbe],
    sets: &[DiagramSet],
    mut m2_of: impl FnMut(usize, &[Vec<[f64; 4]>]) -> Result<Vec<f64>, ProbeError>,
) -> Result<(), ProbeError> {
    if banked.len() != sets.len() {
        return Err(ProbeError::SetCount {
            banked: banked.len(),
            sets: sets.len(),
        });
    }
    for (k, (probe, set)) in banked.iter().zip(sets).enumerate() {
        if probe.points.is_empty() {
            continue;
        }
        let want = set.particles_in.len() + set.particles_out.len();
        if let Some((point, p)) = probe
            .points
            .iter()
            .enumerate()
            .find(|(_, p)| p.momenta.len() != want)
        {
            return Err(ProbeError::Legs {
                set: k,
                label: set.label(),
                point,
                got: p.momenta.len(),
                want,
            });
        }
        let momenta: Vec<Vec<[f64; 4]>> = probe.points.iter().map(|p| p.momenta.clone()).collect();
        let computed = m2_of(k, &momenta)?;
        for (point, (p, &m2)) in probe.points.iter().zip(&computed).enumerate() {
            let deviation = relative_deviation(p.m2, m2);
            // Written so that a NaN deviation refuses too.
            if !(deviation <= PROBE_REL_TOL) {
                return Err(ProbeError::Moved {
                    set: k,
                    label: set.label(),
                    point,
                    banked: p.m2,
                    computed: m2,
                    deviation,
                });
            }
        }
    }
    Ok(())
}

/// Recompile every set of `sets` and refuse a banked point whose |M|² this build
/// no longer reproduces to [`PROBE_REL_TOL`] under `evaluated`.
pub fn check(
    banked: &[SetProbe],
    sets: &[DiagramSet],
    model: &UFOModel,
    evaluated: &EvaluatedModel,
) -> Result<(), ProbeError> {
    check_with(banked, sets, |k, points| {
        let evaluator = compile(&sets[k], model, evaluated)?;
        Ok(evaluate(&evaluator, evaluated, points))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::ufo::bind::bind_card;
    use crate::ufo::loaded::LoadedModel;
    use crate::ufo::slha::ParamCard;
    use crate::ufo::sm::SMRestrict;

    struct Fixture {
        sets: Vec<DiagramSet>,
        model: LoadedModel,
        evaluated: EvaluatedModel,
    }

    fn fixture(card: &str) -> Fixture {
        let parsed = parse_proc_card(card, &ParsingOptions::default()).unwrap();
        let model = LoadedModel::interned_sm(SMRestrict::Default);
        let sets = generate_from_proc_card(&parsed, &model.model).unwrap();
        let evaluated = bind_card(model.model.clone(), &ParamCard::default())
            .unwrap()
            .evaluated;
        Fixture {
            sets,
            model,
            evaluated,
        }
    }

    /// Two subprocesses, one massive final state, gluons and quarks: the sets
    /// carry several helicities and colour flows each.
    const CARD: &str = "generate u u~ > t t~ g\nadd process g g > t t~ g\n";

    fn probe(f: &Fixture) -> Vec<SetProbe> {
        measure(&f.sets, &f.model.model, &f.evaluated).unwrap()
    }

    /// Every set gets four points: partonic centre of mass with the beams on
    /// ±z, momentum conserved and every leg on its shell, every pair separated
    /// by the genericity bound, at the documented `√ŝ`, with a finite, positive
    /// |M|².
    ///
    /// Cannot detect a point that passes these checks and still sits somewhere
    /// |M|² is degenerate.
    #[test]
    fn probe_points_are_on_shell_partonic_and_generic() {
        let f = fixture(CARD);
        let probes = probe(&f);
        assert_eq!(probes.len(), f.sets.len());
        assert_eq!(probes.len(), 2);
        let mt = f.evaluated.mass(f.model.model.particle_id("t").unwrap());
        let sqrt_s = (PROBE_THRESHOLD_MULTIPLE * 2.0 * mt).max(PROBE_MIN_SQRT_S);
        for (probe, set) in probes.iter().zip(&f.sets) {
            assert_eq!(probe.points.len(), PROBE_POINTS);
            let masses: Vec<f64> = set
                .particles_in
                .iter()
                .chain(&set.particles_out)
                .map(|n| f.evaluated.mass(f.model.model.particle_id(n).unwrap()))
                .collect();
            for p in &probe.points {
                assert!(p.m2.is_finite() && p.m2 > 0.0, "{}", p.m2);
                let [a, b] = [p.momenta[0], p.momenta[1]];
                assert_eq!([a[1], a[2], b[1], b[2]], [0.0; 4]);
                assert!(a[3] > 0.0 && b[3] < 0.0);
                for mu in 0..4 {
                    let total_in = a[mu] + b[mu];
                    let total_out: f64 = p.momenta[2..].iter().map(|q| q[mu]).sum();
                    assert!((total_in - total_out).abs() < 1e-9 * sqrt_s, "{mu}");
                }
                assert!((a[0] + b[0] - sqrt_s).abs() < 1e-9 * sqrt_s);
                for (q, m) in p.momenta.iter().zip(&masses) {
                    let m2 = q[0] * q[0] - q[1] * q[1] - q[2] * q[2] - q[3] * q[3];
                    assert!((m2 - m * m).abs() < 1e-6 * sqrt_s * sqrt_s, "{m2} vs {m}");
                }
                assert!(generic(&p.momenta, sqrt_s * sqrt_s));
            }
            assert_ne!(probe.points[0].momenta, probe.points[1].momenta);
        }
    }

    /// The probe is a function of the sets and the bound card alone: measuring
    /// twice gives the same momenta and the same |M|², bit for bit.
    #[test]
    fn measuring_twice_banks_the_same_probe() {
        let f = fixture(CARD);
        let a = probe(&f);
        let b = probe(&f);
        assert_eq!(a, b);
    }

    /// A decay is probed at rest at the parent's mass.
    #[test]
    fn a_decay_is_probed_at_rest() {
        let f = fixture("generate t > b e+ ve\n");
        let probes = probe(&f);
        let mt = f.evaluated.mass(f.model.model.particle_id("t").unwrap());
        for p in &probes[0].points {
            assert_eq!(p.momenta[0], [mt, 0.0, 0.0, 0.0]);
            assert!(p.m2.is_finite() && p.m2 > 0.0);
        }
    }

    /// The probe accepts what this build computes, and refuses a banked value
    /// moved by a relative 1e-8 while accepting one moved by 1e-13, naming the
    /// set, the point and the deviation.
    ///
    /// Cannot detect a change in the evaluator that moves no probed point.
    #[test]
    fn a_moved_banked_value_is_refused_and_ulp_noise_is_not() {
        let f = fixture(CARD);
        let banked = probe(&f);
        check(&banked, &f.sets, &f.model.model, &f.evaluated).unwrap();

        let mut tiny = banked.clone();
        tiny[1].points[2].m2 *= 1.0 + 1e-13;
        assert_ne!(tiny, banked);
        check(&tiny, &f.sets, &f.model.model, &f.evaluated).unwrap();

        let mut moved = banked.clone();
        moved[1].points[2].m2 *= 1.0 + 1e-8;
        let err = check(&moved, &f.sets, &f.model.model, &f.evaluated)
            .expect_err("a 1e-8 move must be refused");
        let ProbeError::Moved {
            set,
            point,
            deviation,
            ..
        } = &err
        else {
            panic!("{err}");
        };
        assert_eq!((*set, *point), (1, 2));
        assert!((deviation / 1e-8 - 1.0).abs() < 1e-3, "{deviation}");
        let message = err.to_string();
        assert!(message.contains("g g > t t~ g"), "{message}");
        assert!(message.contains("point 2"), "{message}");
        assert!(message.contains("integrate again"), "{message}");
    }

    /// A build whose matrix element moved by a relative 1e-9 is refused at the
    /// first set: the recomputation, not the banked value, is perturbed here.
    #[test]
    fn a_recomputed_matrix_element_moved_by_1e_9_is_refused() {
        let f = fixture(CARD);
        let banked = probe(&f);
        let moved = |k: usize, points: &[Vec<[f64; 4]>]| {
            let evaluator = compile(&f.sets[k], &f.model.model, &f.evaluated)?;
            Ok(evaluate(&evaluator, &f.evaluated, points)
                .into_iter()
                .map(|m2| m2 * (1.0 + 1e-9))
                .collect())
        };
        let err = check_with(&banked, &f.sets, moved).expect_err("a 1e-9 move must be refused");
        assert!(
            matches!(
                err,
                ProbeError::Moved {
                    set: 0,
                    point: 0,
                    ..
                }
            ),
            "{err}"
        );
    }

    /// A probe list that does not match the sets, by count or by legs, is
    /// refused rather than compared positionally.
    #[test]
    fn a_misshapen_probe_is_refused() {
        let f = fixture(CARD);
        let banked = probe(&f);
        let short = &banked[..1];
        assert!(matches!(
            check(short, &f.sets, &f.model.model, &f.evaluated),
            Err(ProbeError::SetCount { banked: 1, sets: 2 })
        ));
        let mut legs = banked.clone();
        legs[0].points[3].momenta.pop();
        assert!(matches!(
            check(&legs, &f.sets, &f.model.model, &f.evaluated),
            Err(ProbeError::Legs {
                set: 0,
                point: 3,
                got: 4,
                want: 5,
                ..
            })
        ));
    }

    #[test]
    fn the_relative_deviation_is_symmetric_and_refuses_nan() {
        assert_eq!(relative_deviation(0.0, 0.0), 0.0);
        assert_eq!(relative_deviation(2.0, 1.0), relative_deviation(1.0, 2.0));
        assert_eq!(relative_deviation(2.0, 1.0), 0.5);
        assert_eq!(relative_deviation(f64::NAN, 1.0), f64::INFINITY);
        assert_eq!(relative_deviation(1.0, 0.0), 1.0);
    }
}
