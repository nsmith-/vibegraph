//! Leg-relabelling invariance of the per-diagram amplitudes.
//!
//! Listing a process's legs in another order within the initial and within the final
//! state changes no physics: each diagram of the relabelled process is a diagram of
//! the original, and evaluated at the permuted momenta and helicities it has the
//! same value. MadGraph's per-graph `AMP()` has that property graph by graph, and a
//! fermion-sign convention fixed by the leg order can change at most one sign shared
//! by every diagram (the order the external fermions are paired in). So for each
//! relabelled pair, every diagram of the relabelled process must equal its partner up
//! to one sign common to the whole process.
//!
//! The comparison needs no reference: it pits the evaluator against itself. It is the
//! falsifier for any convention sign that depends on which vertex holds a particular
//! leg (leg 0, say) rather than on the graph alone. It cannot see a convention that is
//! a wrong function of the graph — that changes both labellings alike and is the
//! amplitude oracles' to catch — nor a dependence on a vertex that no relabelling
//! moves.

use std::collections::HashMap;
use std::sync::Arc;

use num_complex::Complex;

use super::compile::AmplitudeEvaluator;
use super::renumbering::{generate, load, point, single_diagram_amplitudes_at};
use crate::diagrams::diagram::{Diagram, Ray, VtxIdx};
use crate::helas::LorentzVector;
use crate::ufo::sm::{sm_model, SMRestrict};
use crate::ufo::{EvaluatedModel, UFOModel};

/// A process, the same process with its legs listed in another order, and the
/// permutation between them: leg `i` of the relabelled process is leg `perm[i]` of the
/// original. Legs move only within the initial and within the final state.
struct Pair {
    /// `None` for the Standard Model, else a model directory and one of its restrict
    /// cards, as `renumbering::load` resolves them.
    model: Option<(&'static str, &'static str)>,
    original: &'static str,
    relabelled: &'static str,
    perm: &'static [usize],
}

const SMEFT: &str = "validation/ufo/SMEFTsim_topU3l_MwScheme_UFO";
const TOY: &str = "validation/ufo/vibegraph_toy_UFO";

/// The relabelled pairs. The first five move the vertex leg 0 attaches to between
/// vertex classes: a vector-boson vertex and a two-fermion vertex (`w+ e- > z ve`), a
/// two-fermion vector vertex and a Yukawa vertex on a final-state quark pair
/// (`w+ h > t b~`, `z h > t t~`), and in O_W's processes the photon–Z–Higgs vertex
/// between producing the Higgs and producing a vector. The rest swap legs where the
/// vertex classes holding leg 0 do not change, or change between a scalar and a
/// fermion vertex.
const PAIRS: &[Pair] = &[
    Pair {
        model: None,
        original: "w+ e- > z ve",
        relabelled: "e- w+ > ve z",
        perm: &[1, 0, 3, 2],
    },
    Pair {
        model: None,
        original: "w+ h > t b~",
        relabelled: "h w+ > t b~",
        perm: &[1, 0, 2, 3],
    },
    Pair {
        model: None,
        original: "z h > t t~",
        relabelled: "h z > t t~",
        perm: &[1, 0, 2, 3],
    },
    Pair {
        model: Some((SMEFT, "vg_cW")),
        original: "w+ w- > w+ w- z NP<=1",
        relabelled: "w- w+ > w- w+ z NP<=1",
        perm: &[1, 0, 3, 2, 4],
    },
    Pair {
        model: Some((SMEFT, "vg_cW")),
        original: "a z > w+ w- NP<=1",
        relabelled: "z a > w- w+ NP<=1",
        perm: &[1, 0, 3, 2],
    },
    Pair {
        model: None,
        original: "w+ w- > e+ e-",
        relabelled: "w- w+ > e- e+",
        perm: &[1, 0, 3, 2],
    },
    Pair {
        model: None,
        original: "e+ e- > w+ w-",
        relabelled: "e- e+ > w- w+",
        perm: &[1, 0, 3, 2],
    },
    Pair {
        model: None,
        original: "w+ a > e+ ve a",
        relabelled: "a w+ > ve e+ a",
        perm: &[1, 0, 3, 2, 4],
    },
    Pair {
        model: None,
        original: "e+ e- > ta+ ta- h",
        relabelled: "e- e+ > ta- ta+ h",
        perm: &[1, 0, 3, 2, 4],
    },
    Pair {
        model: None,
        original: "u u~ > z g",
        relabelled: "u~ u > g z",
        perm: &[1, 0, 3, 2],
    },
    Pair {
        model: None,
        original: "h h > t t~",
        relabelled: "h h > t~ t",
        perm: &[0, 1, 3, 2],
    },
    Pair {
        model: Some((TOY, "dscalar")),
        original: "st lt > st lt NP<=2 NPD<=1",
        relabelled: "lt st > lt st NP<=2 NPD<=1",
        perm: &[1, 0, 3, 2],
    },
];

/// Pairs whose per-diagram amplitudes are known to change with the labelling, with
/// the original process's diagrams that flip against the rest (the minority: a sign
/// shared by every diagram is invisible). Two-way: a listed pair must flip exactly
/// these diagrams, and every other pair none.
const KNOWN_RELABELLING_DEPENDENT: &[(&str, &[usize])] = &[];

/// Tolerance on a diagram's deviation from `±1` times its partner, relative to its
/// largest value. Both labellings evaluate the same currents; only the rooting and
/// the association order of momentum sums can differ.
const REL_TOL: f64 = 1e-10;

/// Diagrams below this fraction of the process's largest diagram are not compared:
/// a vanishing coupling (a massless fermion's Yukawa) leaves no sign to read.
const VANISHING: f64 = 1e-12;

/// The external legs on the far side of each of `v`'s rays, as original leg indices
/// (`relabel` maps this diagram's legs to the original's).
fn ray_leg_sets(d: &Diagram, v: usize, relabel: &[usize]) -> Vec<Vec<usize>> {
    let mut sets: Vec<Vec<usize>> = d.vertices[v]
        .rays
        .iter()
        .map(|ray| {
            let mut legs: Vec<usize> = match *ray {
                Ray::Leg(l) => vec![relabel[l.0]],
                // `v` sits at `endpoints[end]`; the far side is the other endpoint's.
                Ray::Prop { prop, end } => d
                    .legs_on_start_side(prop)
                    .iter()
                    .enumerate()
                    .filter(|&(_, &on_start)| on_start == (end == 1))
                    .map(|(l, _)| relabel[l])
                    .collect(),
            };
            legs.sort_unstable();
            legs
        })
        .collect();
    sets.sort();
    sets
}

/// A description of the graph that no labelling or numbering changes: each vertex's
/// interaction, fermion-flow group and the external legs behind each of its rays, in
/// the original process's leg labels.
fn signature(d: &Diagram, relabel: &[usize]) -> Vec<(usize, usize, Vec<Vec<usize>>)> {
    let mut sig: Vec<_> = (0..d.vertices.len())
        .map(|v| {
            let vertex = d.vertex(VtxIdx(v));
            (
                vertex.interaction.0,
                vertex.flow_group,
                ray_leg_sets(d, v, relabel),
            )
        })
        .collect();
    sig.sort();
    sig
}

/// The original process's diagrams whose sign under relabelling differs from the
/// majority's, with the number of diagrams compared.
fn relabelling_flips(pair: &Pair, model: &Arc<UFOModel>) -> (Vec<usize>, usize) {
    let evaluated = EvaluatedModel::from_model(model.clone());
    let mut originals = generate(pair.original, model);
    let mut relabelled = generate(pair.relabelled, model);
    assert_eq!(
        (originals.len(), relabelled.len()),
        (1, 1),
        "{}",
        pair.original
    );
    let (set1, set2) = (originals.remove(0), relabelled.remove(0));
    let n = set1.diagrams[0].n_ext();
    assert_eq!(pair.perm.len(), n);
    let identity: Vec<usize> = (0..n).collect();

    let mut by_signature: HashMap<_, usize> = HashMap::new();
    for (i, d) in set1.diagrams.iter().enumerate() {
        let previous = by_signature.insert(signature(d, &identity), i);
        assert!(
            previous.is_none(),
            "{}: two diagrams share a signature",
            pair.original
        );
    }
    assert_eq!(
        set1.diagrams.len(),
        set2.diagrams.len(),
        "{}: the relabelled process has another diagram count",
        pair.original
    );
    let partner: Vec<usize> = set2
        .diagrams
        .iter()
        .map(|d| {
            *by_signature
                .get(&signature(d, pair.perm))
                .unwrap_or_else(|| panic!("{}: a relabelled diagram has no partner", pair.original))
        })
        .collect();

    let momenta1 = point(&set1, &evaluated);
    let momenta2: Vec<LorentzVector<f64>> = pair.perm.iter().map(|&j| momenta1[j]).collect();
    let evaluator1 = AmplitudeEvaluator::compile(&set1, model).unwrap();
    assert_eq!(
        evaluator1.n_flows(),
        1,
        "{}: relabelled pairs are single-flow",
        pair.original
    );
    let helicities1: Vec<Vec<i32>> = evaluator1.helicities().to_vec();
    let helicities2: Vec<Vec<i32>> = helicities1
        .iter()
        .map(|h| pair.perm.iter().map(|&j| h[j]).collect())
        .collect();

    let values1: Vec<Vec<Complex<f64>>> = set1
        .diagrams
        .iter()
        .map(|d| {
            single_diagram_amplitudes_at(
                &set1,
                d,
                model,
                &evaluated,
                &momenta1,
                Some(helicities1.as_slice()),
            )
        })
        .collect();
    let largest = values1
        .iter()
        .flatten()
        .map(|c| c.norm())
        .fold(0.0, f64::max);
    let mut signs: Vec<(usize, f64)> = Vec::new();
    for (j, d2) in set2.diagrams.iter().enumerate() {
        let i = partner[j];
        let a = &values1[i];
        let scale = a.iter().map(|c| c.norm()).fold(0.0, f64::max);
        if scale < VANISHING * largest {
            continue;
        }
        let b = single_diagram_amplitudes_at(
            &set2,
            d2,
            model,
            &evaluated,
            &momenta2,
            Some(helicities2.as_slice()),
        );
        let k = (0..a.len())
            .max_by(|&x, &y| a[x].norm().total_cmp(&a[y].norm()))
            .unwrap();
        let sign = if (b[k] + a[k]).norm() < (b[k] - a[k]).norm() {
            -1.0
        } else {
            1.0
        };
        let worst = a
            .iter()
            .zip(&b)
            .map(|(a, b)| (b - sign * a).norm())
            .fold(0.0, f64::max);
        assert!(
            worst <= REL_TOL * scale,
            "{}: diagram {i} is not ±1 times its relabelled partner ({:.3e} of its scale)",
            pair.original,
            worst / scale
        );
        signs.push((i, sign));
    }
    // One compared diagram, or none, has no other to flip against.
    assert!(
        signs.len() >= 2,
        "{}: {} diagrams compared, too few to show a relative sign",
        pair.original,
        signs.len()
    );
    let minus = signs.iter().filter(|(_, s)| *s < 0.0).count();
    let majority = if 2 * minus > signs.len() { -1.0 } else { 1.0 };
    let mut flips: Vec<usize> = signs
        .iter()
        .filter(|(_, s)| *s != majority)
        .map(|(i, _)| *i)
        .collect();
    flips.sort_unstable();
    (flips, signs.len())
}

/// Every diagram of each relabelled pair equals its partner up to one sign shared by
/// the whole process; [`KNOWN_RELABELLING_DEPENDENT`] lists the measured exceptions.
#[test]
fn relabelling_preserves_per_diagram_amplitudes() {
    let mut models = HashMap::new();
    let sm = sm_model(SMRestrict::Default);
    let mut failures = Vec::new();
    for pair in PAIRS {
        let model = match pair.model {
            None => sm.clone(),
            Some((dir, card)) => load(&mut models, dir, Some(card)),
        };
        let (flips, compared) = relabelling_flips(pair, &model);
        let known = KNOWN_RELABELLING_DEPENDENT
            .iter()
            .find(|(p, _)| *p == pair.original)
            .map(|(_, d)| d.to_vec())
            .unwrap_or_default();
        println!(
            "{} / {}: {compared} diagrams compared, flipped against the rest: {flips:?}",
            pair.original, pair.relabelled
        );
        if flips != known {
            failures.push(format!(
                "{} / {}: diagrams {flips:?} flip under relabelling, {known:?} expected",
                pair.original, pair.relabelled
            ));
        }
    }
    for (p, _) in KNOWN_RELABELLING_DEPENDENT {
        assert!(
            PAIRS.iter().any(|pair| pair.original == *p),
            "{p} is listed but not a relabelled pair"
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
