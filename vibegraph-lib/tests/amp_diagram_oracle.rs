//! Per-diagram amplitude oracle: validate vibegraph's individual diagram
//! amplitudes against MadGraph's own `AMP(1:NGRAPHS)`, complex and element-wise,
//! for every single-flow process in `validation/madgraph/amp_reference.json`.
//!
//! # Why this test exists
//!
//! `validate_helas_mg` compares one real number per phase-space point: the
//! colour-contracted, helicity-summed |M|². That number cannot see a relative
//! sign or phase between two diagrams whose interference is small at the sampled
//! points, cannot see a permutation of the helicity assignment that leaves the
//! sum invariant, and cannot see a per-diagram error compensated by a colour
//! coefficient. The finest linear object MadGraph exposes for a single-flow
//! process is `AMP(i)` per helicity, so that is what this compares.
//!
//! # What is compared, and why it is `c_i · AMP(i)`
//!
//! MadGraph's coherent amplitude for a single-flow process is
//! `JAMP(1) = Σ_i c_i · AMP(i)`, and MadGraph puts the relative sign between an
//! annihilation diagram and its exchange partner into `c_i` — `e+ e- > e+ e-`
//! carries `c = (−1, −1, +1, +1)`. vibegraph puts that sign in the diagram root
//! (`fermi_sign`) and keeps colour coefficients of +1. Neither convention is
//! observable on its own; what is observable is the product, so the comparable
//! per-diagram object is `c_i · AMP(i)` — the diagram's contribution to the
//! amplitude — and the coefficients are banked with the amplitudes.
//!
//! # What is asserted
//!
//! Per process, against banked MadGraph values (see `gen_amp_reference.py`):
//!
//! 1. The helicity-combination set agrees with MadGraph's `NHEL` table, and the
//!    diagram count agrees with `NGRAPHS`.
//! 2. A **single** complex constant `G`, fitted once per process over every
//!    (point, helicity, diagram) entry, satisfies
//!    `A_i^vg(p, h) = G · c_j · AMP_j^mg(p, h)` for all of them, with `j` the
//!    diagram pairing below. One constant for the whole process is what makes
//!    this sensitive: a relative sign between two diagrams, a helicity-dependent
//!    phase, or a momentum-dependent one all have nowhere to hide.
//! 3. `|G| = 1` — a uniform rescaling is the one deviation the fit absorbs with
//!    zero residual — and `Re G = 0`. The second half is a convention claim with
//!    teeth: vibegraph roots its diagrams with one factor of `i` MadGraph's
//!    `AMP()` leaves out, so the constant is `±i` and nothing else. An arbitrary
//!    phase would satisfy assertion 2 just as well and would mean the two sides
//!    differ by something other than that single factor.
//! 4. The coherent amplitude follows the *same* constant:
//!    `M^vg(p, h) = G · JAMP_1^mg(p, h)`. Assertion 2 pins the diagram roots
//!    against the weighted MadGraph terms; this pins that vibegraph's own colour
//!    coefficients reassemble them into the amplitude the way MadGraph's do.
//!
//! # Known blind spots
//!
//! - A phase common to every diagram is absorbed into `G` by construction. It is
//!   genuine convention freedom: MadGraph itself chooses the overall sign of
//!   `JAMP(1)` per colour structure (`T(1,5,2)` with `c = +1` for `g u`,
//!   `T(1,2,5)` with `c = −1` for `g u~`), and |M|² = CF·|JAMP|² cannot see it.
//!   Assertion 3 narrows the freedom to a sign but cannot remove it.
//! - The diagram pairing is banked, not searched for. Two independent
//!   enumeration orders have no reason to agree, so a disagreement is recorded in
//!   [`MG_DIAGRAM_ORDER`] rather than re-derived per run: a reordering on either
//!   side then fails here instead of being silently re-matched. The pairing is
//!   massively over-determined by the data it has to reproduce (every helicity,
//!   every point, one constant), so banking it is not fitting it.
//! - Multi-flow processes are not banked: their diagram roots are not scalars,
//!   and the corresponding fine-grained object is `JAMP(1:NCOLOR)`, covered by
//!   `color_jamp_oracle`.
//!
//! Run:
//!   cargo test -p vibegraph-lib --features extended-validation \
//!              --test amp_diagram_oracle
//!
//! Prerequisites (the param cards live in the gitignored MG output tree):
//!   pixi run -e madgraph build-amplitude

mod common;

use libtest_mimic::{Arguments, Failed, Trial};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use vibegraph::diagrams::DiagramSet;
use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
use vibegraph::helas::repr::C;
use vibegraph::helas::LorentzVector;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::EvaluatedModel;

/// Relative tolerance on the element-wise per-diagram comparison, normalised to
/// the largest reference term in the process.
///
/// Both sides evaluate the same HELAS kernels in f64 from the same param card, so
/// the difference is accumulated rounding over a handful of operations; the
/// observed residuals sit at 1e-15. The margin is for kernel reassociation, not
/// for a convention; it matches `color_jamp_oracle`'s.
const AMP_REL_TOL: f64 = 1e-12;

/// MadGraph graph index of each vibegraph diagram, for processes whose two
/// enumeration orders differ. Absent processes pair by the identity.
///
/// The orders agree for every process here except `e+ e- > ta+ ta- H`, where
/// vibegraph emits the two `H`-off-a-τ-leg diagrams before the `Z → Z H` one that
/// MadGraph puts first. Nothing derives one order from the other, so the
/// disagreement is data.
const MG_DIAGRAM_ORDER: &[(&str, &[usize])] = &[("ee_to_tatah", &[3, 4, 1, 2, 0])];

fn reference_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/amp_reference.json")
}

fn mg_output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/output")
}

/// One banked process: MadGraph's diagram count, colour coefficients, helicity
/// table, and per-point `AMP()` / `JAMP(1)` tables.
struct MgAmps {
    process: String,
    n_graphs: usize,
    coefficients: Vec<C<f64>>,
    helicities: Vec<Vec<i32>>,
    /// `points[k] = (momenta, amps[hel][graph], jamp[hel])`
    points: Vec<(Vec<LorentzVector<f64>>, Vec<Vec<C<f64>>>, Vec<C<f64>>)>,
}

fn complex_list(v: &serde_json::Value) -> Vec<C<f64>> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|z| {
            let c = z.as_array().unwrap();
            C::new(c[0].as_f64().unwrap(), c[1].as_f64().unwrap())
        })
        .collect()
}

fn parse_reference(json: &serde_json::Value, name: &str) -> MgAmps {
    let entry = &json["processes"][name];
    let helicities = entry["helicities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| {
            h.as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i32)
                .collect()
        })
        .collect();
    let points = entry["points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|pt| {
            let momenta = pt["momenta"]
                .as_array()
                .unwrap()
                .iter()
                .map(|m| {
                    let c: Vec<f64> = m
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_f64().unwrap())
                        .collect();
                    LorentzVector::new(c[0], c[1], c[2], c[3])
                })
                .collect();
            let amps = pt["amps"]
                .as_array()
                .unwrap()
                .iter()
                .map(complex_list)
                .collect();
            // Single-flow by construction, so JAMP is one number per helicity.
            let jamp = pt["jamps"]
                .as_array()
                .unwrap()
                .iter()
                .map(|per_hel| complex_list(per_hel)[0])
                .collect();
            (momenta, amps, jamp)
        })
        .collect();
    MgAmps {
        process: entry["process"].as_str().unwrap().to_owned(),
        n_graphs: entry["n_graphs"].as_u64().unwrap() as usize,
        coefficients: complex_list(&entry["jamp_coefficients"]),
        helicities,
        points,
    }
}

/// Per-diagram breakdown printed when the element-wise comparison fails: each
/// vibegraph diagram's own best-fit constant with its residual under it, and the
/// normalised overlap against every MadGraph term. A constant that is unit and a
/// residual that is at rounding on every row means the diagrams are individually
/// right and only their relative weighting is wrong; a single off-diagonal
/// overlap of 1 names the diagram this one should have been paired with.
fn report_breakdown(name: &str, vg: &[Vec<Vec<C<f64>>>], mg: &[Vec<Vec<C<f64>>>], scale: f64) {
    let n = mg.first().and_then(|p| p.first()).map_or(0, Vec::len);
    eprintln!("[{name}] per-diagram breakdown (vibegraph row vs MadGraph term):");
    for i in 0..n {
        let (mut num, mut den) = (C::new(0.0, 0.0), 0.0f64);
        for (pi, per_point) in mg.iter().enumerate() {
            for (hi, per_hel) in per_point.iter().enumerate() {
                num += per_hel[i].conj() * vg[pi][hi][i];
                den += per_hel[i].norm_sqr();
            }
        }
        let g = if den > 0.0 {
            num / den
        } else {
            C::new(0.0, 0.0)
        };
        let mut res = 0.0f64;
        let mut overlaps = String::new();
        for j in 0..n {
            let (mut o, mut a, mut b) = (C::new(0.0, 0.0), 0.0f64, 0.0f64);
            for (pi, per_point) in mg.iter().enumerate() {
                for (hi, per_hel) in per_point.iter().enumerate() {
                    if j == i {
                        res = res.max((vg[pi][hi][i] - g * per_hel[i]).norm() / scale);
                    }
                    o += per_hel[j].conj() * vg[pi][hi][i];
                    a += vg[pi][hi][i].norm_sqr();
                    b += per_hel[j].norm_sqr();
                }
            }
            overlaps.push_str(&format!(" {:.4}", o.norm() / (a * b).sqrt().max(1e-300)));
        }
        eprintln!(
            "  diagram {i}: own constant |g|={:.6} arg={:+7.2}° residual={res:.2e} \
             | overlaps{overlaps}",
            g.norm(),
            g.arg().to_degrees()
        );
    }
}

fn run_trial(name: String, reference: MgAmps) -> Result<(), Failed> {
    let model = common::sm_model();
    let card = std::fs::read_to_string(mg_output_dir().join(&name).join("Cards/param_card.dat"))
        .map_err(|e| format!("param_card.dat for {name}: {e}"))?
        .parse::<ParamCard>()
        .map_err(|e| format!("param_card.dat for {name}: {e:?}"))?;
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &card);

    let sets = common::generate_with(&reference.process, model.as_ref());
    let set = sets
        .first()
        .ok_or_else(|| format!("no diagram set for '{}'", reference.process))?;
    if set.diagrams.len() != reference.n_graphs {
        return Err(format!(
            "[{name}] diagram count: vibegraph {} vs MadGraph NGRAPHS {}",
            set.diagrams.len(),
            reference.n_graphs
        )
        .into());
    }

    let full = AmplitudeEvaluator::compile(set, model.as_ref())?;
    if full.n_flows() != 1 {
        return Err(format!(
            "[{name}] NCOLOR={} — the per-diagram reference banks single-flow \
             processes only",
            full.n_flows()
        )
        .into());
    }

    // The helicity sum both sides run over must be the same set, or the
    // element-wise comparison would silently skip combinations.
    let ours: BTreeSet<Vec<i32>> = full.helicities().iter().map(|h| h.to_vec()).collect();
    let theirs: BTreeSet<Vec<i32>> = reference.helicities.iter().cloned().collect();
    if ours != theirs {
        return Err(format!(
            "[{name}] helicity sets differ: vibegraph {} combos, MadGraph {}",
            ours.len(),
            theirs.len()
        )
        .into());
    }

    let order: Vec<usize> = MG_DIAGRAM_ORDER
        .iter()
        .find(|(p, _)| *p == name)
        .map(|(_, o)| o.to_vec())
        .unwrap_or_else(|| (0..reference.n_graphs).collect());
    if order.len() != reference.n_graphs
        || order.iter().collect::<BTreeSet<_>>().len() != order.len()
    {
        return Err(format!(
            "[{name}] MG_DIAGRAM_ORDER is not a permutation of {} indices",
            reference.n_graphs
        )
        .into());
    }

    // MadGraph's per-diagram *contribution* to the amplitude, in vibegraph's
    // diagram order: mg[point][hel][vibegraph diagram] = c_j · AMP_j.
    let mg: Vec<Vec<Vec<C<f64>>>> = reference
        .points
        .iter()
        .map(|(_, amps, _)| {
            amps.iter()
                .map(|per_hel| {
                    order
                        .iter()
                        .map(|&j| reference.coefficients[j] * per_hel[j])
                        .collect()
                })
                .collect()
        })
        .collect();

    // One evaluator per diagram: a single-diagram `DiagramSet` compiles the same
    // rooted tree the full set gives that diagram — the rooting and its fermion
    // sign are properties of the diagram — so its amplitude root is the diagram's
    // contribution up to the process-wide convention constant fitted below.
    let per_diagram: Vec<AmplitudeEvaluator> = set
        .diagrams
        .iter()
        .map(|d| {
            AmplitudeEvaluator::compile(
                &DiagramSet {
                    particles_in: set.particles_in.clone(),
                    particles_out: set.particles_out.clone(),
                    diagrams: vec![d.clone()],
                },
                model.as_ref(),
            )
        })
        .collect::<Result<_, _>>()?;

    // vg[point][hel][diagram], evaluated at MadGraph's own helicity vectors.
    let vg: Vec<Vec<Vec<C<f64>>>> = reference
        .points
        .iter()
        .map(|(momenta, _, _)| {
            reference
                .helicities
                .iter()
                .map(|hel| {
                    per_diagram
                        .iter()
                        .map(|ev| {
                            let bound = BoundAmplitude::<f64>::bind(ev, &evaluated);
                            let mut scratch = bound.scratch_space();
                            bound.eval_amplitude(momenta, hel, &mut scratch)
                        })
                        .collect()
                })
                .collect()
        })
        .collect();

    // One constant for the whole process, least-squares over every
    // (point, helicity, diagram) entry: G = Σ conj(T^mg)·A^vg / Σ |T^mg|².
    // Fitting it globally rather than per diagram is what makes the residual
    // below sensitive to a relative per-diagram phase.
    let mut scale = 0.0f64;
    let mut num = C::new(0.0, 0.0);
    let mut den = 0.0f64;
    for (pi, per_point) in mg.iter().enumerate() {
        for (hi, per_hel) in per_point.iter().enumerate() {
            for (di, term) in per_hel.iter().enumerate() {
                scale = scale.max(term.norm());
                num += term.conj() * vg[pi][hi][di];
                den += term.norm_sqr();
            }
        }
    }
    if den == 0.0 {
        return Err(format!("[{name}] reference has no non-zero AMP").into());
    }
    let g = num / den;

    let mut worst = 0.0f64;
    let mut worst_where = String::new();
    for (pi, per_point) in mg.iter().enumerate() {
        for (hi, per_hel) in per_point.iter().enumerate() {
            for (di, term) in per_hel.iter().enumerate() {
                let expect = g * *term;
                let dev = (vg[pi][hi][di] - expect).norm() / scale;
                if dev > worst {
                    worst = dev;
                    worst_where = format!(
                        "point {pi}, hel {:?}, diagram {di} (MadGraph graph {}): \
                         vibegraph {:?} vs G·MadGraph {expect:?}",
                        reference.helicities[hi],
                        order[di] + 1,
                        vg[pi][hi][di]
                    );
                }
            }
        }
    }
    if worst > AMP_REL_TOL {
        report_breakdown(&name, &vg, &mg, scale);
        return Err(format!(
            "[{name}] per-diagram amplitudes disagree with MadGraph beyond a single \
             global phase (max element-wise deviation {worst:.3e}) at {worst_where}"
        )
        .into());
    }

    let mag_dev = (g.norm() - 1.0).abs();
    if mag_dev > AMP_REL_TOL {
        return Err(format!(
            "[{name}] the vibegraph↔MadGraph diagram constant is not a pure phase: \
             |G| = {:.17} (deviation {mag_dev:.3e})",
            g.norm()
        )
        .into());
    }
    if g.re.abs() > AMP_REL_TOL {
        return Err(format!(
            "[{name}] the vibegraph↔MadGraph diagram constant is not ±i: G = {g:?}. \
             The two sides are expected to differ by exactly the one factor of i \
             vibegraph's diagram roots carry and MadGraph's AMP() does not"
        )
        .into());
    }

    // The coherent amplitude under the *same* constant: vibegraph's own colour
    // coefficients must reassemble its diagram roots the way MadGraph's do.
    let bound = BoundAmplitude::<f64>::bind(&full, &evaluated);
    let mut scratch = bound.scratch_space();
    let coherent_scale = reference
        .points
        .iter()
        .flat_map(|(_, _, jamp)| jamp.iter())
        .fold(0.0f64, |m, z| m.max(z.norm()));
    let mut worst_coherent = 0.0f64;
    let mut worst_coherent_where = String::new();
    for (pi, (momenta, _, jamp)) in reference.points.iter().enumerate() {
        for (hi, hel) in reference.helicities.iter().enumerate() {
            let expect = g * jamp[hi];
            let got = bound.eval_amplitude(momenta, hel, &mut scratch);
            let dev = (got - expect).norm() / coherent_scale;
            if dev > worst_coherent {
                worst_coherent = dev;
                worst_coherent_where =
                    format!("point {pi}, hel {hel:?}: vibegraph {got:?} vs G·MadGraph {expect:?}");
            }
        }
    }
    if worst_coherent > AMP_REL_TOL {
        return Err(format!(
            "[{name}] the coherent amplitude does not follow the per-diagram constant \
             (max deviation {worst_coherent:.3e}) at {worst_coherent_where} — vibegraph's \
             colour coefficients weight the diagrams differently from MadGraph's"
        )
        .into());
    }

    println!(
        "  [{name}] '{}' NGRAPHS={}: per-diagram max_rel={worst:.2e}, coherent \
         max_rel={worst_coherent:.2e} (G = {:+.0}i, |G|-1 = {mag_dev:.1e})",
        reference.process,
        reference.n_graphs,
        g.im.signum(),
    );
    Ok(())
}

fn main() {
    let args = Arguments::from_args();

    let path = reference_path();
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("cannot read {}: {e}", path.display());
            eprintln!("Run: pixi run -e madgraph generate-amp-reference");
            libtest_mimic::run(&args, vec![]).exit();
        }
    };
    let json: serde_json::Value = serde_json::from_str(&text).expect("amp_reference.json");
    let mut names: Vec<String> = json["processes"]
        .as_object()
        .expect("processes map")
        .keys()
        .cloned()
        .collect();
    names.sort();

    let trials: Vec<Trial> = names
        .into_iter()
        .map(|name| {
            let reference = parse_reference(&json, &name);
            Trial::test(name.clone(), move || run_trial(name, reference))
        })
        .collect();

    libtest_mimic::run(&args, trials).exit();
}
