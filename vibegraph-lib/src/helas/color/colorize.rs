//! Colorize a subprocess: diagram color strings → color basis + exact CF matrix.
//!
//! This is the compile-time bridge from parsed UFO color factors
//! ([`crate::ufo::color`]) to the symbolic algebra ([`super`]), walking the
//! owned [`Diagram`]s of one subprocess and producing
//!
//! - a **color basis**: the distinct simplified color structures (one per
//!   "flow"), each recording the `(diagram, color-index chain, exact
//!   coefficient)` contributions that build its JAMP;
//! - the exact **color-factor matrix** `CF_{ff'} = ⟨f | f'*⟩` evaluated at
//!   `Nc = 3`, as rationals.
//!
//! Both mirror MadGraph's `color_amp.py` (`ColorBasis`/`ColorMatrix`); the
//! floating-point evaluator never sees a color index.
//!
//! ## The walk (per diagram)
//!
//! The owned [`Diagram`] already stores each vertex's rays in UFO
//! interaction-slot order, which is exactly the order the vertex color-string
//! indices `1..n` refer to. So the substitution is purely positional:
//!
//! - a slot holding an external [`Ray::Leg`] takes the (positive, free) leg
//!   number as its color index;
//! - a slot holding a [`Ray::Prop`] takes one fresh negative *summed* index per
//!   propagator — both end-vertices reference the same [`PropIdx`] and so share
//!   the index, which is the implicit δ that glues the propagator.
//!
//! **Output-leg conjugation.** MadGraph's `colorize` conjugates the color rep
//! of a non-final vertex's output leg (`get_anti_color`), because the
//! propagator's far end sees the antiparticle. That conjugation is pure
//! bookkeeping in MadGraph: the color *string* contraction is driven entirely
//! by the shared summed index, not by the rep. Here both ends of a propagator
//! reference the same [`PropIdx`] and receive the same summed index; the 3/3̄
//! pairing is carried by the interaction color strings' slot structure (a
//! fundamental index `i` at one end, an antifundamental index `j` at the
//! other). We assert the two endpoints' reps are mutually conjugate as a
//! cross-check that ray order matches interaction-slot order.
//!
//! A vertex with `k` used color structures expands the accumulated product `k`
//! ways; the per-diagram "color-index chain" records the chosen structure
//! index for every vertex (in [`VtxIdx`] order). A colorless vertex has the
//! single structure `0`; a fully colorless diagram reduces to `ColorOne`.

use std::collections::HashMap;

use indexmap::IndexMap;
use num_rational::Ratio;

use crate::diagrams::diagram::{Diagram, PropIdx, Ray, VtxIdx};
use crate::helas::repr::color::ColorRep;
use crate::ufo::color::{ColorAtom, ColorExpr};
use crate::ufo::UFOModel;

use super::coeff::ColorCoeff;
use super::factor::{ColorFactor, ColorString, ImmutableString};
use super::tensor::{ColorAlgebraError, ColorTensor, Idx};

/// The number of colors SU(3) is evaluated at.
const NC: i64 = 3;

/// One contribution to a color flow's JAMP: an amplitude picked out by
/// `(diagram, chain)` weighted by the exact color coefficient that survived
/// simplification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contribution {
    /// Index of the contributing diagram within the subprocess.
    pub diagram: usize,
    /// The color-index chain (chosen color structure per vertex, in [`VtxIdx`]
    /// order) selecting this amplitude.
    pub chain: Vec<u8>,
    /// Exact coefficient `q · i^imag · Nc^power` of this contribution.
    pub coeff: ColorCoeff,
}

/// One element of the color basis (one "flow"): a simplified color structure
/// and every `(diagram, chain)` contribution that lands on it.
#[derive(Clone, Debug)]
pub struct BasisElement {
    /// The immutable color structure that is this flow's basis key.
    pub structure: ImmutableString,
    /// Contributions summed into this flow's JAMP.
    pub contributions: Vec<Contribution>,
}

/// The color factorization of one subprocess: a basis of flows in MadGraph's
/// canonical (sorted) order, plus the exact color-factor matrix over them.
#[derive(Clone, Debug)]
pub struct ColorBasis {
    /// Basis elements ("flows") in sorted-key order (matching MadGraph's JAMP
    /// order, `sorted(ColorBasis.keys())`).
    pub elements: Vec<BasisElement>,
    /// `NCOLOR² = elements.len()²` exact color factors, row-major:
    /// `cf_matrix[i * ncolor + j] = CF_{ij}`.
    pub cf_matrix: Vec<Ratio<i64>>,
}

impl ColorBasis {
    /// The number of color flows (NCOLOR).
    pub fn ncolor(&self) -> usize {
        self.elements.len()
    }

    /// The exact color factor `CF_{ij}`.
    pub fn cf(&self, i: usize, j: usize) -> Ratio<i64> {
        self.cf_matrix[i * self.ncolor() + j]
    }
}

/// The color rep at a `(vertex, slot)` of a diagram, read from the vertex's
/// interaction particle list. Sextets (and any non-SU(3)-tree rep) yield
/// [`ColorAlgebraError::Unsupported`].
fn slot_rep(
    model: &UFOModel,
    diagram: &Diagram,
    vtx: VtxIdx,
    slot: usize,
) -> Result<ColorRep, ColorAlgebraError> {
    let interaction = diagram.vertices[vtx.0].interaction;
    let particle = model.vertex_def(interaction).particles[slot];
    let color = model.particle(particle).color;
    ColorRep::from_ufo(color)
        .ok_or_else(|| ColorAlgebraError::Unsupported(format!("color rep {color}")))
}

/// The color structure indices actually used by a vertex: the distinct
/// `color_idx` appearing as the first component of a `(color_idx, lorentz_idx)`
/// coupling key, sorted ascending.
fn used_color_indices(model: &UFOModel, vtx: &crate::diagrams::diagram::Vertex) -> Vec<usize> {
    let mut idxs: Vec<usize> = model
        .vertex_def(vtx.interaction)
        .couplings
        .keys()
        .map(|&(c, _)| c)
        .collect();
    idxs.sort_unstable();
    idxs.dedup();
    idxs
}

/// Substitute one UFO color-string index against the slot map and the
/// per-vertex internal-index table. Positive indices are 1-based interaction
/// slots; negative indices are the vertex's own summed indices, each mapped to
/// a fresh global summed index.
fn map_index(
    idx: i32,
    slot_index: &[Idx],
    internal: &mut HashMap<i32, Idx>,
    counter: &mut Idx,
) -> Idx {
    if idx > 0 {
        slot_index[(idx - 1) as usize]
    } else {
        *internal.entry(idx).or_insert_with(|| {
            let v = *counter;
            *counter -= 1;
            v
        })
    }
}

/// Convert one vertex's chosen [`ColorExpr`] into algebra tensors, substituting
/// slot indices from `slot_index` and allocating fresh summed indices (from
/// `counter`) for the expression's internal negative indices. Returns the
/// integer coefficient (the `2` from an octet `Identity`, otherwise `1`).
///
/// **The `T` index pair is transposed on substitution, and the baryonic
/// `Epsilon`/`EpsilonBar` are exchanged.** In a UFO color string `T(a…,i,j)` the
/// slot `i` holds the interaction's `3` field and `j` its `3̄`
/// ([`check_slot_reps`] pins that). MadGraph instead indexes a `T`'s
/// fundamental slot by the leg that carries a `3` index in the all-outgoing
/// crossing, and feyngraph presents every leg in the all-incoming crossing — the
/// opposite arrow — so the leg standing in a `3` slot is exactly the one
/// MadGraph puts in the antifundamental position. Reading the pair straight
/// through would complex-conjugate the color string: invisible to the CF matrix
/// and to the purely-rational T-chain contributions, but it flips the sign of
/// the imaginary `f → trace` coefficients (corrupting the relative sign between
/// `f`-derived and T-chain structures, e.g. `g g > t t~`) and it splits a basis
/// that mixes both readings into a structure and its conjugate (`u u~ > t t~`
/// with a four-quark contact, where the singlet contact and the gluon exchange
/// then land on two unrelated keys). Adjoint indices are self-conjugate and pass
/// through untouched, so pure-gluon vertices are unaffected.
///
/// The sextet atoms cross the same way: a `K6` stands on a **6** slot with two
/// `3̄` slots and a `K6Bar` on a `6̄` with two `3`, so under the crossing each
/// becomes the other, and a `T6(i,j)` transposes exactly as a `T` does.
///
/// The same crossing is what exchanges the two baryonic invariants. An
/// `Epsilon` stands on three `3` slots and an `EpsilonBar` on three `3̄` slots,
/// so under the crossing every one of those legs is the opposite kind to the one
/// MadGraph reads and the whole tensor becomes its conjugate. Reading them
/// straight through is not invisible: the epsilon–epsilon-bar contraction
/// produces `T` products, and taking them unconjugated while every `T` in the
/// same basis is conjugated splits the basis into a structure and its transpose
/// — the `p3 r3 > p3 r3` diquark row would then land its two diagrams on
/// unrelated keys instead of on MadGraph's `T(3,1) T(4,2)` / `T(3,2) T(4,1)`.
fn convert_expr(
    expr: &ColorExpr,
    slot_index: &[Idx],
    counter: &mut Idx,
) -> Result<(i64, Vec<ColorTensor>), ColorAlgebraError> {
    let mut internal: HashMap<i32, Idx> = HashMap::new();
    let mut map = |idx: i32| map_index(idx, slot_index, &mut internal, counter);
    let tensors = expr
        .atoms
        .iter()
        .map(|atom| match atom {
            ColorAtom::T(adj, i, j) => {
                let adj: Vec<Idx> = adj.iter().map(|&a| map(a)).collect();
                Ok(ColorTensor::T(adj, map(*j), map(*i)))
            }
            ColorAtom::Tr(adj) => Ok(ColorTensor::Tr(adj.iter().map(|&a| map(a)).collect())),
            ColorAtom::F(a, b, c) => Ok(ColorTensor::F(map(*a), map(*b), map(*c))),
            ColorAtom::D(a, b, c) => Ok(ColorTensor::D(map(*a), map(*b), map(*c))),
            ColorAtom::Epsilon(a, b, c) => Ok(ColorTensor::EpsilonBar(map(*a), map(*b), map(*c))),
            ColorAtom::EpsilonBar(a, b, c) => Ok(ColorTensor::Epsilon(map(*a), map(*b), map(*c))),
            ColorAtom::K6(m, i, j) => Ok(ColorTensor::K6Bar(map(*m), map(*i), map(*j))),
            ColorAtom::K6Bar(m, i, j) => Ok(ColorTensor::K6(map(*m), map(*i), map(*j))),
            ColorAtom::T6(adj, i, j) if adj.is_empty() => Ok(ColorTensor::T6(map(*j), map(*i))),
            ColorAtom::T6(..) => Err(ColorAlgebraError::Unsupported(format!(
                "sextet generator '{atom}' in '{expr}': only the sextet delta T6(i,j) is \
                 reduced, the generator's expansion into K6 T K6Bar is not"
            ))),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((expr.coeff, tensors))
}

/// The per-slot color index for every vertex of a diagram, plus the base value
/// for fresh vertex-internal summed indices (one below the most negative
/// propagator index). External legs get their (positive) leg number; each
/// propagator gets one shared negative summed index.
///
/// The mapping is purely positional: feyngraph presents a vertex's rays in UFO
/// interaction-slot order, which is the order the color-string indices `1..n`
/// refer to. The `3`/`3̄` crossing between feyngraph's slot order and
/// MadGraph's `T` index convention is undone per tensor in [`convert_expr`],
/// not by permuting slots.
fn slot_indices(diagram: &Diagram) -> (Vec<Vec<Idx>>, Idx) {
    // One summed index per propagator, deterministic in PropIdx order.
    let prop_index = |p: PropIdx| -(p.0 as Idx + 1);
    let slots = diagram
        .vertices
        .iter()
        .map(|v| {
            v.rays
                .iter()
                .map(|ray| match ray {
                    Ray::Leg(li) => li.0 as Idx + 1,
                    Ray::Prop { prop, .. } => prop_index(*prop),
                })
                .collect()
        })
        .collect();
    let internal_base = -(diagram.props.len() as Idx + 1);
    (slots, internal_base)
}

/// Cross-check the index convention [`convert_expr`] crosses under. In every
/// color structure the diagram uses, each atom must stand on the reps its own
/// crossing assumes: a `T(a…,i,j)` on a `3` and a `3̄` in that order, an
/// `Epsilon` on three `3`s and an `EpsilonBar` on three `3̄`s, a `K6` on a `6`
/// with two `3̄`s and a `K6Bar` on a `6̄` with two `3`s, and a sextet delta
/// `T6(i,j)` on a `6`/`6̄` pair.
///
/// A summed index in any of those positions, or a slot whose rep is not the one
/// the atom implies, means the vertex is written under a convention this engine
/// does not read, so it is rejected rather than crossed into a wrong answer.
fn check_slot_reps(model: &UFOModel, diagram: &Diagram) -> Result<(), ColorAlgebraError> {
    for (vi, vertex) in diagram.vertices.iter().enumerate() {
        let structures = &model.vertex_def(vertex.interaction).color;
        for expr in used_color_indices(model, vertex)
            .into_iter()
            .map(|c| &structures[c])
        {
            let rep = |slot: i32| -> Option<ColorRep> {
                let s = usize::try_from(slot - 1).ok()?;
                (s < vertex.rays.len())
                    .then(|| slot_rep(model, diagram, VtxIdx(vi), s).ok())
                    .flatten()
            };
            let require = |slots: &[i32], want: ColorRep, what: &str| {
                slots
                    .iter()
                    .all(|&s| rep(s) == Some(want))
                    .then_some(())
                    .ok_or_else(|| {
                        ColorAlgebraError::Unsupported(format!(
                            "{what} in '{expr}' does not stand on {} {want:?} slot(s)",
                            slots.len()
                        ))
                    })
            };
            for atom in &expr.atoms {
                match atom {
                    ColorAtom::T(_, i, j) => {
                        if rep(*i) != Some(ColorRep::Triplet)
                            || rep(*j) != Some(ColorRep::AntiTriplet)
                        {
                            return Err(ColorAlgebraError::Unsupported(format!(
                                "T({i},{j}) in '{expr}' is not a 3/3̄ slot pair"
                            )));
                        }
                    }
                    ColorAtom::Epsilon(a, b, c) => require(
                        &[*a, *b, *c],
                        ColorRep::Triplet,
                        &format!("Epsilon({a},{b},{c})"),
                    )?,
                    ColorAtom::EpsilonBar(a, b, c) => require(
                        &[*a, *b, *c],
                        ColorRep::AntiTriplet,
                        &format!("EpsilonBar({a},{b},{c})"),
                    )?,
                    ColorAtom::K6(m, i, j) => {
                        if rep(*m) != Some(ColorRep::Sextet) {
                            return Err(ColorAlgebraError::Unsupported(format!(
                                "K6({m},{i},{j}) in '{expr}' does not lead with a 6 slot"
                            )));
                        }
                        require(
                            &[*i, *j],
                            ColorRep::AntiTriplet,
                            &format!("K6({m},{i},{j})"),
                        )?;
                    }
                    ColorAtom::K6Bar(m, i, j) => {
                        if rep(*m) != Some(ColorRep::AntiSextet) {
                            return Err(ColorAlgebraError::Unsupported(format!(
                                "K6Bar({m},{i},{j}) in '{expr}' does not lead with a 6̄ slot"
                            )));
                        }
                        require(&[*i, *j], ColorRep::Triplet, &format!("K6Bar({m},{i},{j})"))?;
                    }
                    ColorAtom::T6(adj, i, j) if adj.is_empty() => {
                        if rep(*i) != Some(ColorRep::Sextet)
                            || rep(*j) != Some(ColorRep::AntiSextet)
                        {
                            return Err(ColorAlgebraError::Unsupported(format!(
                                "T6({i},{j}) in '{expr}' is not a 6/6̄ slot pair"
                            )));
                        }
                    }
                    ColorAtom::Tr(_) | ColorAtom::F(..) | ColorAtom::D(..) | ColorAtom::T6(..) => {}
                }
            }
        }
    }
    Ok(())
}

/// Cross-check that each propagator's two endpoints carry mutually-conjugate
/// color reps (e.g. `3`/`3̄`, `8`/`8`). A mismatch means ray order does not match
/// interaction-slot order — a construction bug, not a model error.
fn check_propagator_reps(model: &UFOModel, diagram: &Diagram) -> Result<(), ColorAlgebraError> {
    for prop in &diagram.props {
        let [(va, sa), (vb, sb)] = prop.endpoints;
        let rep_a = slot_rep(model, diagram, va, sa.0)?;
        let rep_b = slot_rep(model, diagram, vb, sb.0)?;
        assert_eq!(
            rep_a.anti(),
            rep_b,
            "propagator endpoints carry non-conjugate color reps ({rep_a:?} vs {rep_b:?}); \
             ray order likely disagrees with interaction-slot order"
        );
    }
    Ok(())
}

/// Build every `(chain, color string)` for one diagram: the Cartesian product
/// over vertices of their used color structures, each product multiplied out
/// with substituted indices (before simplification).
fn colorize_diagram(
    model: &UFOModel,
    diagram: &Diagram,
) -> Result<Vec<(Vec<u8>, ColorString)>, ColorAlgebraError> {
    check_propagator_reps(model, diagram)?;

    check_slot_reps(model, diagram)?;

    let (slots, internal_base) = slot_indices(diagram);
    let used: Vec<Vec<usize>> = diagram
        .vertices
        .iter()
        .map(|v| used_color_indices(model, v))
        .collect();

    // Cartesian product of the per-vertex used color-structure indices.
    let mut chains: Vec<Vec<u8>> = vec![Vec::new()];
    for choices in &used {
        chains = chains
            .into_iter()
            .flat_map(|prefix| {
                choices.iter().map(move |&c| {
                    let mut next = prefix.clone();
                    next.push(c as u8);
                    next
                })
            })
            .collect();
    }

    let mut out = Vec::with_capacity(chains.len());
    for chain in chains {
        let mut counter = internal_base;
        let mut coeff = ColorCoeff::one();
        let mut tensors = Vec::new();
        for (vi, &color_idx) in chain.iter().enumerate() {
            let vertex = &diagram.vertices[vi];
            let expr = &model.vertex_def(vertex.interaction).color[color_idx as usize];
            let (int_coeff, ts) = convert_expr(expr, &slots[vi], &mut counter)?;
            coeff = coeff.mul(&ColorCoeff::rational(int_coeff, 1));
            tensors.extend(ts);
        }
        out.push((chain, ColorString { coeff, tensors }));
    }
    Ok(out)
}

/// Relabel the summed (twice-appearing negative) indices of `struct2` to sit
/// below the smallest index of `struct1`, avoiding collisions when the two are
/// contracted. Ports MadGraph's `ColorMatrix.fix_summed_indices`.
fn fix_summed_indices(s1: &ImmutableString, s2: &ImmutableString) -> ImmutableString {
    let all_indices = |s: &ImmutableString| -> Vec<Idx> {
        s.iter()
            .flat_map(|(_, idxs)| idxs.iter().copied())
            .collect()
    };
    let list1 = all_indices(s1);
    let list2 = all_indices(s2);
    let mut min_index = list1.iter().min().map_or(-1, |m| m - 1);

    let mut repl: HashMap<Idx, Idx> = HashMap::new();
    for &idx in &list2 {
        if list2.iter().filter(|&&x| x == idx).count() == 2 && !repl.contains_key(&idx) {
            repl.insert(idx, min_index);
            min_index -= 1;
        }
    }

    s2.iter()
        .map(|(kind, idxs)| {
            let mapped = idxs
                .iter()
                .map(|i| repl.get(i).copied().unwrap_or(*i))
                .collect();
            (*kind, mapped)
        })
        .collect()
}

/// One color-matrix entry `CF_{ij} = ⟨struct1 | struct2*⟩` evaluated at
/// `Nc = 3`: multiply `struct1` by the conjugate of the (index-fixed)
/// `struct2`, `full_simplify`, and sum the scalar result. The result is real;
/// the imaginary parts must cancel.
fn cf_entry(s1: &ImmutableString, s2: &ImmutableString) -> Ratio<i64> {
    let fixed2 = fix_summed_indices(s1, s2);
    let str1 = ColorString::from_immutable(s1);
    let str2 = ColorString::from_immutable(&fixed2).conj();

    let mut tensors = str1.tensors;
    tensors.extend(str2.tensors);
    let product = ColorString {
        coeff: str1.coeff.mul(&str2.coeff),
        tensors,
    };
    let simplified = ColorFactor(vec![product]).full_simplify();

    let mut real = Ratio::from_integer(0);
    let mut imag = Ratio::from_integer(0);
    for cs in &simplified.0 {
        assert!(
            cs.tensors.is_empty(),
            "color matrix entry did not reduce to a scalar: {:?}",
            cs.tensors
        );
        if cs.coeff.imag {
            imag += cs.coeff.eval_nc(NC);
        } else {
            real += cs.coeff.eval_nc(NC);
        }
    }
    assert_eq!(
        imag,
        Ratio::from_integer(0),
        "color matrix entry has a non-vanishing imaginary part"
    );
    real
}

/// Colorize a whole subprocess: walk every diagram, accumulate the color basis
/// (deterministically ordered by sorted immutable key, matching MadGraph's JAMP
/// order), and build the exact `NCOLOR²` color-factor matrix.
pub fn colorize_process(
    model: &UFOModel,
    diagrams: &[Diagram],
) -> Result<ColorBasis, ColorAlgebraError> {
    let mut basis: IndexMap<ImmutableString, Vec<Contribution>> = IndexMap::new();

    for (diag_idx, diagram) in diagrams.iter().enumerate() {
        for (chain, string) in colorize_diagram(model, diagram)? {
            let simplified = ColorFactor(vec![string]).full_simplify();
            for cs in &simplified.0 {
                let key = cs.to_immutable();
                basis.entry(key).or_default().push(Contribution {
                    diagram: diag_idx,
                    chain: chain.clone(),
                    coeff: cs.coeff,
                });
            }
        }
    }

    // MadGraph orders flows by sorted immutable key (ColorMatrix.build_matrix
    // and get_color_amplitudes both iterate `sorted(keys)`); match it so JAMP
    // ordering lines up with the Fortran reference downstream.
    let mut keys: Vec<ImmutableString> = basis.keys().cloned().collect();
    keys.sort();

    let ncolor = keys.len();
    let mut cf_matrix = vec![Ratio::from_integer(0); ncolor * ncolor];
    for (i, ki) in keys.iter().enumerate() {
        for (j, kj) in keys.iter().enumerate() {
            if j < i {
                cf_matrix[i * ncolor + j] = cf_matrix[j * ncolor + i];
            } else {
                cf_matrix[i * ncolor + j] = cf_entry(ki, kj);
            }
        }
    }

    let elements = keys
        .into_iter()
        .map(|structure| {
            let contributions = basis.swap_remove(&structure).unwrap_or_default();
            BasisElement {
                structure,
                contributions,
            }
        })
        .collect();

    Ok(ColorBasis {
        elements,
        cf_matrix,
    })
}

#[cfg(test)]
mod tests {
    use super::super::tensor::TensorKind;
    use super::*;

    /// A `T(i,j)` immutable structure over the given fundamental/antifundamental
    /// external indices.
    fn t(i: Idx, j: Idx) -> (TensorKind, Vec<Idx>) {
        (TensorKind::T, vec![i, j])
    }

    /// The color-flow diagonal for a single quark line is `Nc`.
    #[test]
    fn cf_single_quark_line_is_nc() {
        // ⟨T(1,2)|T(1,2)*⟩ = Tr() = Nc = 3.
        let s: ImmutableString = vec![t(1, 2)];
        assert_eq!(cf_entry(&s, &s), Ratio::from_integer(3));
    }

    /// Two independent quark lines give `Nc²`.
    #[test]
    fn cf_two_quark_lines_is_nc_squared() {
        let s: ImmutableString = vec![t(1, 2), t(3, 4)];
        assert_eq!(cf_entry(&s, &s), Ratio::from_integer(9));
    }

    /// The two color flows of a `qq̄ → qq̄`-type basis interfere as `Nc` off the
    /// diagonal: ⟨T(1,2)T(3,4) | (T(1,4)T(3,2))*⟩ = Nc.
    #[test]
    fn cf_offdiagonal_two_flows_is_nc() {
        let a: ImmutableString = vec![t(1, 2), t(3, 4)];
        let b: ImmutableString = vec![t(1, 4), t(3, 2)];
        assert_eq!(cf_entry(&a, &b), Ratio::from_integer(3));
    }

    /// The colorless flow has `CF = 1`.
    #[test]
    fn cf_colorless_is_one() {
        let one: ImmutableString = vec![(TensorKind::One, vec![])];
        assert_eq!(cf_entry(&one, &one), Ratio::from_integer(1));
    }

    /// `fix_summed_indices` pushes struct2's summed index below struct1's min.
    #[test]
    fn fix_summed_indices_relabels_below_min() {
        // struct1 has summed index -1; struct2 also has a summed index -1 that
        // must be relabelled to avoid a collision.
        let s1: ImmutableString = vec![(TensorKind::T, vec![1, -1, -1, 2])];
        let s2: ImmutableString = vec![(TensorKind::T, vec![3, -1, -1, 4])];
        let fixed = fix_summed_indices(&s1, &s2);
        // min index in s1 is -1, so the new summed index is -2.
        assert_eq!(fixed, vec![(TensorKind::T, vec![3, -2, -2, 4])]);
    }
}
