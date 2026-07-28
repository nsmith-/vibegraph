//! Per-flow JAMP oracle: validate vibegraph's colour-flow partial amplitudes —
//! and the `JAMP2` weights derived from them — against MadGraph's own
//! `JAMP(1:NCOLOR)`, element-wise and complex.
//!
//! # Why this test exists
//!
//! `validate_helas_mg` gates on the CF-contracted |M|², which sums the flows
//! away. It therefore cannot see a per-flow phase, a per-flow normalisation, or a
//! permutation belonging to the CF matrix's automorphism group — and the
//! `g g > g g` CF matrix has a large one, since it is invariant under the
//! trace-reversal pairing `(1,6)(2,4)(3,5)`. Any of those three would leave |M|²
//! and σ exact while skewing `JAMP2(i) = Σ_hel |JAMPᵢ|²`, the categorical weight
//! a per-event colour-flow selection draws from. This test closes that blind spot
//! at the finest linear level MadGraph exposes.
//!
//! Together with its two siblings the colour basis is pinned end to end:
//! `color_cf_oracle` pins the CF matrix and the basis ordering, and
//! `color_flow_tags_oracle` pins each flow's colour-line connectivity against
//! `leshouche.inc`; this pins the *amplitude* that sits in each of those slots.
//!
//! # What is asserted
//!
//! Per process, against `validation/madgraph/jamp_reference.json` (banked
//! MadGraph `JAMP()` values, see `gen_jamp_reference.py`):
//!
//! 1. `n_flows` and the helicity-combination set agree with MadGraph's.
//! 2. A **single** complex constant `g` — fitted once per process from the
//!    largest-magnitude reference entry, never per flow and never per point —
//!    satisfies `JAMPᵢ^vg(p, h) = g · JAMPᵢ^mg(p, h)` for every flow, helicity
//!    and phase-space point, under the **identity** flow pairing. A per-flow
//!    phase, a per-flow rescale or a permutation all break this.
//! 3. `|g| = 1`. Without this a uniform rescaling would be absorbed into the fit
//!    and would silently rescale every `JAMP2`. (A uniform *phase* is genuine
//!    convention freedom: vibegraph roots its diagrams with an `i` placement
//!    MadGraph's `AMP()` leaves out, so `g = ±i` here. It cancels in `JAMP2` and
//!    in |M|², which is precisely why it must not be allowed to hide anything.)
//! 4. `eval_jamp2` reproduces `Σ_hel |JAMPᵢ^mg|²` flow by flow. This is the
//!    selection weight itself, checked against MadGraph directly rather than
//!    against vibegraph's own JAMPs, so it does not inherit assertion 2's fit.
//!
//! # Known blind spot
//!
//! For `g g > g g`, flows related by trace reversal carry *identical* JAMPs
//! (`J₁ = J₆`, `J₂ = J₄`, `J₃ = J₅`, an exact consequence of the four-gluon
//! amplitude being real-coefficient in the trace basis). Swapping such a pair is
//! invisible here, to `JAMP2`, and to |M|². It is not invisible to
//! `color_flow_tags_oracle`, which compares the two orientations' colour-line
//! endpoints — so the pair ambiguity is covered there, not here.
//!
//! Run:
//!   cargo test -p vibegraph-lib --features extended-validation \
//!              --test color_jamp_oracle
//!
//! Prerequisites (the param cards live in the gitignored MG output tree):
//!   pixi run -e madgraph build-amplitude

mod common;

use libtest_mimic::{Arguments, Failed, Trial};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
use vibegraph::helas::repr::C;
use vibegraph::helas::LorentzVector;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::EvaluatedModel;

/// Relative tolerance on the element-wise JAMP comparison and on `JAMP2`.
///
/// Both sides evaluate the same HELAS kernels in f64 from the same param card, so
/// the difference is accumulated rounding over a handful of operations; the
/// observed residuals sit at 1e-16 relative. The margin here is for kernel
/// reassociation (e.g. an evaluator lowering change), not for a convention.
const JAMP_REL_TOL: f64 = 1e-12;

fn reference_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/jamp_reference.json")
}

fn mg_output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/output")
}

/// One banked process: MadGraph's flow count, helicity set, and per-point
/// `JAMP()` table.
struct MgJamps {
    process: String,
    n_flows: usize,
    structures: Vec<String>,
    helicities: Vec<Vec<i32>>,
    /// `points[k] = (momenta, jamps[hel][flow])`
    points: Vec<(Vec<LorentzVector<f64>>, Vec<Vec<(f64, f64)>>)>,
}

fn parse_reference(json: &serde_json::Value, name: &str) -> MgJamps {
    let entry = &json["processes"][name];
    let helicities: Vec<Vec<i32>> = entry["helicities"]
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
            let jamps = pt["jamps"]
                .as_array()
                .unwrap()
                .iter()
                .map(|per_hel| {
                    per_hel
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|z| {
                            let c = z.as_array().unwrap();
                            (c[0].as_f64().unwrap(), c[1].as_f64().unwrap())
                        })
                        .collect()
                })
                .collect();
            (momenta, jamps)
        })
        .collect();
    MgJamps {
        process: entry["process"].as_str().unwrap().to_owned(),
        n_flows: entry["n_flows"].as_u64().unwrap() as usize,
        structures: entry["flow_structures"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|s| s.as_str().unwrap().to_owned())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        helicities,
        points,
    }
}

fn run_trial(name: String, reference: MgJamps) -> Result<(), Failed> {
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
    let eval = AmplitudeEvaluator::compile(set, model.as_ref())?;
    let bound = BoundAmplitude::<f64>::bind(&eval, &evaluated);
    let mut scratch = bound.scratch_space();

    let n = eval.n_flows();
    if n != reference.n_flows {
        return Err(format!(
            "[{name}] NCOLOR mismatch: vibegraph {n} vs MadGraph {}",
            reference.n_flows
        )
        .into());
    }

    // The helicity sum both sides run over must be the same set, or the JAMP2
    // comparison below would compare different sums and the element-wise
    // comparison would silently skip combinations.
    let ours: BTreeSet<Vec<i32>> = eval.helicities().iter().map(|h| h.to_vec()).collect();
    let theirs: BTreeSet<Vec<i32>> = reference.helicities.iter().cloned().collect();
    if ours != theirs {
        return Err(format!(
            "[{name}] helicity sets differ: vibegraph {} combos, MadGraph {}",
            ours.len(),
            theirs.len()
        )
        .into());
    }

    // vg[point][hel][flow], evaluated at MadGraph's own helicity vectors.
    let vg: Vec<Vec<Vec<C<f64>>>> = reference
        .points
        .iter()
        .map(|(momenta, _)| {
            reference
                .helicities
                .iter()
                .map(|hel| bound.run_flows(momenta, hel, &mut scratch))
                .collect()
        })
        .collect();

    // One constant for the whole process, least-squares over every
    // (point, helicity, flow) entry: g = Σ conj(J^mg)·J^vg / Σ |J^mg|². Fitting it
    // globally rather than per flow or per point is what makes the residual below
    // sensitive to per-flow structure — the fit has nowhere to hide it.
    let mut scale = 0.0f64;
    let mut num = C::new(0.0, 0.0);
    let mut den = 0.0f64;
    for (pi, (_, jamps)) in reference.points.iter().enumerate() {
        for (hi, per_hel) in jamps.iter().enumerate() {
            for (fi, &(re, im)) in per_hel.iter().enumerate() {
                let mg = C::new(re, im);
                scale = scale.max(mg.norm());
                num += mg.conj() * vg[pi][hi][fi];
                den += mg.norm_sqr();
            }
        }
    }
    if den == 0.0 {
        return Err(format!("[{name}] reference has no non-zero JAMP").into());
    }
    let g = num / den;

    // Element-wise, identity flow pairing, one constant `g` for the process.
    let mut worst_elem = 0.0f64;
    let mut worst_where = String::new();
    for (pi, (_, jamps)) in reference.points.iter().enumerate() {
        for (hi, per_hel) in jamps.iter().enumerate() {
            for (fi, &(re, im)) in per_hel.iter().enumerate() {
                let expect = g * C::new(re, im);
                let dev = (vg[pi][hi][fi] - expect).norm() / scale;
                if dev > worst_elem {
                    worst_elem = dev;
                    let structure = reference
                        .structures
                        .get(fi)
                        .map(String::as_str)
                        .unwrap_or("?");
                    worst_where = format!(
                        "point {pi}, hel {:?}, flow {fi} [{structure}]: \
                         vibegraph {:?} vs g·MadGraph {expect:?}",
                        reference.helicities[hi], vg[pi][hi][fi]
                    );
                }
            }
        }
    }
    if worst_elem > JAMP_REL_TOL {
        return Err(format!(
            "[{name}] per-flow JAMPs disagree with MadGraph beyond a single global \
             phase (max element-wise deviation {worst_elem:.3e}) at {worst_where}"
        )
        .into());
    }

    // A uniform rescaling is the one deviation the fit above absorbs with zero
    // residual, and it multiplies every JAMP2 weight by |g|².
    let mag_dev = (g.norm() - 1.0).abs();
    if mag_dev > JAMP_REL_TOL {
        return Err(format!(
            "[{name}] the vibegraph↔MadGraph JAMP constant is not a pure phase: \
             |g| = {:.17} (deviation {mag_dev:.3e}); every JAMP2 weight is rescaled by |g|²",
            g.norm()
        )
        .into());
    }

    // The selection weight itself, against MadGraph's JAMPs rather than ours.
    let mut worst_jamp2 = 0.0f64;
    let mut worst_flow = 0usize;
    for (momenta, jamps) in &reference.points {
        let mut mg_jamp2 = vec![0.0f64; n];
        for per_hel in jamps {
            for (acc, &(re, im)) in mg_jamp2.iter_mut().zip(per_hel) {
                *acc += re * re + im * im;
            }
        }
        let mut ours = vec![0.0f64; n];
        bound.eval_jamp2(momenta, &mut scratch, &mut ours);
        let norm = mg_jamp2.iter().cloned().fold(0.0f64, f64::max);
        for (fi, (a, b)) in ours.iter().zip(&mg_jamp2).enumerate() {
            let dev = (a - b).abs() / norm;
            if dev > worst_jamp2 {
                worst_jamp2 = dev;
                worst_flow = fi;
            }
        }
    }
    if worst_jamp2 > JAMP_REL_TOL {
        return Err(format!(
            "[{name}] eval_jamp2 disagrees with Σ_hel |MadGraph JAMP|²: max relative \
             deviation {worst_jamp2:.3e} on flow {worst_flow}"
        )
        .into());
    }

    println!(
        "  [{name}] '{}' NCOLOR={n}: JAMP element-wise max_rel={worst_elem:.2e} \
         (global phase {:+.1}°, |g|-1 = {mag_dev:.1e}), JAMP2 max_rel={worst_jamp2:.2e}",
        reference.process,
        g.arg().to_degrees(),
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
            eprintln!("Run: pixi run -e madgraph python validation/madgraph/gen_jamp_reference.py");
            libtest_mimic::run(&args, vec![]).exit();
        }
    };
    let json: serde_json::Value = serde_json::from_str(&text).expect("jamp_reference.json");
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
