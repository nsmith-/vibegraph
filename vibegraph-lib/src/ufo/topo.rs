//! Utilities for building feyngraph Models from vibegraph's parsed UFO data.
//!
//! This module provides functionality to construct a feyngraph Model from vibegraph's parsed UFO data.

use std::collections::{BTreeSet, HashSet};

use crate::ufo::{
    couplings::Coupling,
    lorentz::{LorentzOp, LorentzStructure, LorentzTerm},
    particles::Particle,
    vertices::Vertex,
};
use feyngraph::model::{LineStyle, Model as TopoModel, Statistic};
use indexmap::IndexMap;
use rustc_hash::FxHashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TopoError {
    #[error("Error building feyngraph model: {0}")]
    BuildError(String),
    #[error("FeynGraph model evaluation error: {0}")]
    FeynGraph(#[from] feyngraph::model::ModelError),
}

/// Helper function to determine feyngraph LineStyle
///
/// Follows particle rule in feyngraph/src/model/ufo_parser.rs
fn map_line_style(particle: &Particle) -> LineStyle {
    match particle.line_style {
        Some(ref style) => match style.as_str() {
            "dashed" => LineStyle::Dashed,
            "dotted" => LineStyle::Dotted,
            "straight" => LineStyle::Straight,
            "wavy" => LineStyle::Wavy,
            "curly" => LineStyle::Curly,
            "scurly" => LineStyle::Scurly,
            "swavy" => LineStyle::Swavy,
            "double" => LineStyle::Double,
            _ => LineStyle::None,
        },
        None => {
            match (particle.spin, particle.color) {
                (1, 1) => LineStyle::Dashed,   // Scalar boson
                (2, _) => LineStyle::Straight, // Fermion
                (3, 1) => LineStyle::Wavy,     // Vector boson
                (3, 8) => LineStyle::Curly,    // Gluon-like boson
                _ => LineStyle::None,
            }
        }
    }
}

/// Helper function to determine feyngraph Statistic from spin.
fn spin_to_statistic(spin: i32) -> Statistic {
    // 2s+1
    match spin.rem_euclid(2) {
        1 => Statistic::Bose,
        0 => Statistic::Fermi,
        _ => unreachable!(),
    }
}

/// Build a feyngraph Model from vibegraph's parsed UFO data.
///
/// Uses feyngraph's mutation API to construct the model without re-parsing the UFO.
pub fn build_feyngraph_model(
    particles: &IndexMap<String, Particle>,
    lorentz: &IndexMap<String, LorentzStructure>,
    couplings: &IndexMap<String, Coupling>,
    vertices: &IndexMap<String, Vertex>,
) -> Result<TopoModel, TopoError> {
    let mut model_builder = TopoModel::empty();

    // Unitary gauge: Goldstone bosons and ghosts never appear in tree-level
    // diagrams (MadGraph excludes them the same way); dropping the particles
    // (and below, their vertices) keeps them out of propagator assignment.
    let is_unphysical = |p: &Particle| p.is_goldstone || p.ghost_number != 0;

    // Add all particles but skip antiparticles since feyngraph's add_particle
    // automatically adds the antiparticle. If we add both, diagram generation
    // misbehaves
    let mut seen_anti = HashSet::new();
    for particle in particles.values() {
        if is_unphysical(particle) {
            continue;
        }
        if seen_anti.contains(&particle.name) {
            continue; // skip if we've already added the antiparticle
        }
        seen_anti.insert(&particle.antiname);
        model_builder.add_particle(
            particle.name.clone(),
            particle.antiname.clone(),
            (particle.spin - 1) as isize, // feyngraph uses 2s for spin
            particle.color as isize,
            particle.pdg_code as isize,
            particle.texname.clone(),
            particle.antitexname.clone(),
            map_line_style(particle),
            spin_to_statistic(particle.spin),
        );
    }

    for (vertex_name, vertex) in vertices {
        if vertex
            .particles
            .iter()
            .any(|&pid| is_unphysical(&particles[pid]))
        {
            continue;
        }
        if vertex.lorentz.is_empty() {
            return Err(TopoError::BuildError(format!(
                "Vertex '{}' has no Lorentz structures defined",
                vertex_name
            )));
        }

        // Collect particle names for this vertex
        let particle_names: Vec<String> = vertex
            .particles
            .iter()
            .map(|&pid| particles[pid].name.clone())
            .collect();

        // One feyngraph vertex per fermion pairing the vertex's structures read. A
        // vertex whose structures disagree describes two different fermion-line
        // topologies, and feyngraph carries one spin map per vertex, so the topologies
        // are separated here and rejoined nowhere: the diagram records which group it
        // used (`Diagram::from_view`) and the evaluator sums only that group's
        // structures.
        let groups = flow_groups(vertex, lorentz);

        // The coupling orders of the interaction: one feyngraph vertex per UFO
        // vertex, and a UFO vertex is split so that all of its couplings carry the
        // same order tuple (`ufo::split_vertices_by_coupling_order`), so the tuple
        // is read off any one of them. Reading them all and asserting agreement is
        // what keeps a future loader change from silently reintroducing the union
        // that made an SM photon current read as `NP = 1`.
        let mut coupling_orders: FxHashMap<String, usize> = FxHashMap::default();
        for (n, &coupling_id) in vertex.couplings.values().enumerate() {
            let orders = &couplings[coupling_id].orders;
            if n == 0 {
                coupling_orders = orders.iter().map(|(k, &v)| (k.clone(), v)).collect();
            } else if orders.len() != coupling_orders.len()
                || orders
                    .iter()
                    .any(|(k, v)| coupling_orders.get(k) != Some(v))
            {
                return Err(TopoError::BuildError(format!(
                    "Vertex '{vertex_name}' mixes coupling orders across its couplings"
                )));
            }
        }

        // the vertex should have been pruned already
        if coupling_orders.is_empty() {
            return Err(TopoError::BuildError(format!(
                "Vertex '{}' has no coupling orders defined",
                vertex_name
            )));
        }

        for (g, group) in groups.iter().enumerate() {
            let name = if groups.len() == 1 {
                vertex_name.clone()
            } else {
                format!("{vertex_name}@{g}")
            };
            model_builder.add_vertex(
                name,
                particle_names.clone(),
                group.spin_map.clone(),
                coupling_orders.clone(),
            )?;
        }
    }

    Ok(model_builder)
}

// ───────────────────────── Fermion flow of a Lorentz structure ─────────────────────────

/// The spinor pairing a Lorentz structure defines, oriented: `(ket leg, bra leg)`
/// pairs, 0-indexed over the vertex's legs.
///
/// A vertex with four fermion legs can be contracted two ways — `(1,2)(3,4)` or
/// `(1,4)(2,3)` in UFO's 1-based numbering — and which one a structure uses is a
/// property of *that structure*, not of the vertex: 80 of SMEFTsim's interactions
/// carry both. The pairing decides which external legs share a fermion line, so it
/// decides the diagram's fermion-line topology and every sign read off it.
pub type FermionFlow = Vec<(usize, usize)>;

/// The fermion flow MadGraph reads from a Lorentz structure
/// (`aloha/aloha_fct.py::get_fermion_flow`), or `None` if the structure's spinor
/// index graph does not resolve into one oriented line per fermion pair.
///
/// The walk is MadGraph's: every spinor operator contributes its index pair as a
/// left (row) → right (column) link, and a fermion line is the chain of links joining
/// two external spinor indices. The chain is followed forwards through `link` and
/// backwards through `rlink`, never revisiting an index, and the pair is oriented by
/// which end terminates on a row (the bra) and which on a column (the ket). Every term
/// of a sum must read the same flow, which is how a structure that mixes pairings
/// inside one expression is rejected rather than silently taking the first term's.
pub fn fermion_flow(structure: &LorentzStructure) -> Option<FermionFlow> {
    let fermion_legs: Vec<isize> = structure
        .spins
        .iter()
        .enumerate()
        .filter(|(_, &s)| s == 2)
        .map(|(i, _)| i as isize)
        .collect();
    if fermion_legs.is_empty() {
        return Some(Vec::new());
    }
    let mut flow: Option<FermionFlow> = None;
    for term in &structure.expr {
        let term_flow = term_fermion_flow(term, &fermion_legs)?;
        match &flow {
            None => flow = Some(term_flow),
            Some(seen) if *seen == term_flow => {}
            Some(_) => return None,
        }
    }
    flow
}

/// [`fermion_flow`] for one term of the sum.
fn term_fermion_flow(term: &LorentzTerm, fermion_legs: &[isize]) -> Option<FermionFlow> {
    let mut link: FxHashMap<isize, isize> = FxHashMap::default();
    let mut rlink: FxHashMap<isize, isize> = FxHashMap::default();
    for op in &term.ops {
        let (row, col) = match op {
            LorentzOp::Gamma { i, j, .. }
            | LorentzOp::Sigma { i, j, .. }
            | LorentzOp::Identity { i, j }
            | LorentzOp::ProjM { i, j }
            | LorentzOp::ProjP { i, j }
            | LorentzOp::Gamma5 { i, j }
            | LorentzOp::C { i, j } => (*i, *j),
            _ => continue,
        };
        // A spinor index carried twice on the same side is not a fermion line.
        if link.insert(row, col).is_some() || rlink.insert(col, row).is_some() {
            return None;
        }
    }

    let mut flow: FermionFlow = Vec::new();
    for &start in fermion_legs {
        let start_leg = start as usize;
        if flow.iter().any(|&(k, b)| k == start_leg || b == start_leg) {
            continue;
        }
        let mut walked = vec![start];
        let mut pos = start;
        loop {
            let step = link
                .get(&pos)
                .filter(|next| !walked.contains(next))
                .or_else(|| rlink.get(&pos).filter(|next| !walked.contains(next)))
                .copied();
            match step {
                Some(next) => {
                    pos = next;
                    walked.push(pos);
                }
                // The chain ends: whichever end sits on a row is the bra.
                None => {
                    if link.contains_key(&pos) && rlink.contains_key(&start) {
                        flow.push((start as usize, pos as usize));
                    } else if rlink.contains_key(&pos) && link.contains_key(&start) {
                        flow.push((pos as usize, start as usize));
                    } else {
                        return None;
                    }
                    break;
                }
            }
        }
    }
    (flow.len() == fermion_legs.len() / 2).then_some(flow)
}

/// The sign MadGraph attaches to a four-fermion structure's coupling for reading its
/// spinor pairing in an order other than the reference `(1,2)(3,4)`
/// (`models/import_ufo.py::UFOMG5Converter.get_sign_flow`).
///
/// MadGraph builds every four-fermion amplitude with the interaction's leading fermion
/// legs paired consecutively, and absorbs the difference between that reference and the
/// structure's actual pairing into the parity of the permutation
/// `(ket₁, bra₁, ket₂, bra₂, …)` — the pairs taken in ascending order of their ket leg.
/// Fewer than four fermions leaves nothing to permute, so the sign is `+1` there by
/// construction, matching MadGraph's `nb_fermion < 4` early return.
///
/// This engine never multiplies the sign in. It builds each diagram on the fermion
/// lines the structure's own pairing defines, so the parity is already carried by the
/// diagram's Fermi sign — MadGraph's factorization into "canonical pairing × coupling
/// sign" and this one differ by nothing observable, which is what `ee_to_mumu_4f`'s
/// per-diagram gate measures. The function is here as the statement of MadGraph's
/// convention, checked structure by structure against the model in
/// `madgraph_fermion_flow_of_smeftsims_four_fermion_structures`.
pub fn permutation_sign(flow: &FermionFlow, fermion_legs: &[usize]) -> i8 {
    if fermion_legs.len() < 4 {
        return 1;
    }
    let mut order: Vec<usize> = Vec::with_capacity(fermion_legs.len());
    for &leg in fermion_legs {
        if let Some(&(ket, bra)) = flow.iter().find(|&&(ket, _)| ket == leg) {
            order.push(ket);
            order.push(bra);
        }
    }
    let inversions = (0..order.len())
        .flat_map(|k| (k + 1..order.len()).map(move |l| (k, l)))
        .filter(|&(k, l)| order[l] < order[k])
        .count();
    if inversions % 2 == 0 {
        1
    } else {
        -1
    }
}

/// One group of a vertex's Lorentz structures: those that pair its fermion legs the
/// same way, and therefore describe the same fermion-line topology.
#[derive(Clone, Debug, PartialEq)]
pub struct FlowGroup {
    /// feyngraph's spin map for this pairing: `spin_map[i]` is the leg `i` shares a
    /// fermion line with, and `i` itself for a leg with no spinor index. This is the
    /// group's identity — two structures belong together exactly when their maps agree.
    pub spin_map: Vec<isize>,
    /// The same pairing oriented as MadGraph reads it, `(ket leg, bra leg)` (see
    /// [`fermion_flow`]). Empty when the structures' spinor index graph does not
    /// resolve into oriented lines, which leaves [`spin_map`](Self::spin_map) — the
    /// unoriented pairing — as all that is known.
    pub flow: FermionFlow,
    /// Positions in the vertex's `lorentz` list carrying this pairing, ascending.
    pub lorentz: Vec<usize>,
}

impl FlowGroup {
    /// The leg this group pairs `leg` with, or `leg` itself for a non-fermion leg.
    pub fn partner(&self, leg: usize) -> usize {
        self.spin_map[leg] as usize
    }
}

/// A vertex's referenced Lorentz structures partitioned by the fermion pairing they
/// read, in the order the pairings first appear.
///
/// Returns a single group for every vertex whose structures agree — which is every
/// vertex of the Standard Model and every vertex with fewer than four fermion legs.
/// Structures are keyed on the unoriented pairing each one's own `spin_map` records,
/// and [`fermion_flow`] supplies the orientation MadGraph's permutation sign is defined
/// on; the two are computed independently from the same expression, so
/// `flow_and_spin_map_agree_on_smeftsim` can hold one against the other.
pub fn flow_groups(
    vertex: &Vertex,
    lorentz: &IndexMap<String, LorentzStructure>,
) -> Vec<FlowGroup> {
    let referenced: BTreeSet<usize> = vertex.couplings.keys().map(|&(_, l)| l).collect();
    let mut groups: Vec<FlowGroup> = Vec::new();
    for l in referenced {
        let structure = &lorentz[vertex.lorentz[l]];
        match groups.iter_mut().find(|g| g.spin_map == structure.spin_map) {
            Some(group) => group.lorentz.push(l),
            None => groups.push(FlowGroup {
                spin_map: structure.spin_map.clone(),
                flow: fermion_flow(structure).unwrap_or_default(),
                lorentz: vec![l],
            }),
        }
    }
    groups
}

/// The `<vertex name>@<group>` suffix [`build_feyngraph_model`] gives the feyngraph
/// vertices of a UFO vertex whose structures read more than one fermion pairing, split
/// back into the UFO vertex name and the [`flow_groups`] index.
///
/// A vertex with one group keeps its bare name, so an unsuffixed name is group 0.
pub fn split_flow_group(name: &str) -> (&str, usize) {
    match name.rsplit_once('@') {
        Some((base, group)) => match group.parse::<usize>() {
            Ok(g) => (base, g),
            Err(_) => (name, 0),
        },
        None => (name, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ufo::{sm::sm_model, sm::SMRestrict, ParsedModel};
    use std::path::PathBuf;

    fn smeftsim_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../validation/ufo/SMEFTsim_topU3l_MwScheme_UFO")
    }

    /// [`fermion_flow`] and [`permutation_sign`] on every four-fermion structure of
    /// SMEFTsim, against the flows and signs MadGraph reads from the same file.
    ///
    /// Four of the rows are readable straight out of MadGraph's own generated
    /// `matrix1_orig.f` for the `ee_to_mumu_4f` row, which calls
    /// `FFFF4_0(..., GC_60, ...)` for a `(1,2)(3,4)` structure and
    /// `FFFF110_0(..., -GC_54, ...)` / `FFFF15_0(..., -GC_29, ...)` — MadGraph's merged
    /// `FFFF14 + FFFF16` routine and `FFFF15` — for the `(1,4)(2,3)` ones, with the
    /// minus signs `get_sign_flow` prefixes. (The restricted model defines `GC_29`,
    /// `GC_54` and `GC_60` with the same sign the UFO gives them, so those minus signs
    /// are the permutation sign and not MadGraph's own merging of equal-up-to-sign
    /// couplings, which is what turns `GC_4` into `-GC_3` a few lines further down the
    /// same file.)
    ///
    /// The pairing count corrects §1.2 of the sprint note, which read 14 / 7 off the
    /// index labels: by MadGraph's walk it is fifteen `(1,2)(3,4)` structures and six
    /// `(1,4)(2,3)`, because `FFFF13` and `FFFF16` write their gamma chains crossed
    /// (`Gamma(-1,2,-3)*Gamma(-1,4,-2)`) and the pairing only appears once the chain
    /// through the projectors is followed.
    #[test]
    fn madgraph_fermion_flow_of_smeftsims_four_fermion_structures() {
        const PAIRED_1234: &[&str] = &[
            "FFFF2", "FFFF4", "FFFF5", "FFFF6", "FFFF7", "FFFF8", "FFFF9", "FFFF11", "FFFF12",
            "FFFF13", "FFFF17", "FFFF18", "FFFF19", "FFFF20", "FFFF21",
        ];
        const PAIRED_1423: &[&str] = &["FFFF1", "FFFF3", "FFFF10", "FFFF14", "FFFF15", "FFFF16"];

        let parsed = ParsedModel::parse(&smeftsim_dir()).expect("parse SMEFTsim");
        let mut seen = 0;
        for structure in parsed.lorentz.values() {
            if structure.spins != [2, 2, 2, 2] {
                continue;
            }
            seen += 1;
            let flow = fermion_flow(structure)
                .unwrap_or_else(|| panic!("{} has no fermion flow", structure.name));
            let sign = permutation_sign(&flow, &[0, 1, 2, 3]);
            let name = structure.name.as_str();
            if PAIRED_1234.contains(&name) {
                assert_eq!(flow, vec![(0, 1), (2, 3)], "{name} {}", structure.structure);
                assert_eq!(sign, 1, "{name}");
            } else if PAIRED_1423.contains(&name) {
                assert_eq!(flow, vec![(0, 3), (2, 1)], "{name} {}", structure.structure);
                assert_eq!(sign, -1, "{name}");
            } else {
                panic!("{name} is in neither pairing list: {}", structure.structure);
            }
        }
        assert_eq!(seen, PAIRED_1234.len() + PAIRED_1423.len());
    }

    /// The oriented flow and the `spin_map` the parser traces independently agree on
    /// which legs share a fermion line, for every structure of SMEFTsim that has one.
    ///
    /// Two implementations reading the same expression: one walks the spinor index graph
    /// MadGraph's way and keeps the row/column orientation, the other traces
    /// contractions in `ufo::lorentz::compute_spin_map` and keeps only the pairing.
    /// [`flow_groups`] keys on the second and takes the orientation from the first, so a
    /// disagreement would give a group an orientation that is not its own.
    #[test]
    fn flow_and_spin_map_agree_on_smeftsim() {
        let parsed = ParsedModel::parse(&smeftsim_dir()).expect("parse SMEFTsim");
        let mut checked = 0;
        for structure in parsed.lorentz.values() {
            let Some(flow) = fermion_flow(structure) else {
                continue;
            };
            for &(ket, bra) in &flow {
                assert_eq!(
                    structure.spin_map[ket], bra as isize,
                    "{}: {}",
                    structure.name, structure.structure
                );
                assert_eq!(structure.spin_map[bra], ket as isize);
                checked += 1;
            }
        }
        assert!(checked >= 60, "only {checked} fermion lines checked");
    }

    /// No Standard-Model vertex splits: every one reads a single pairing, so the
    /// feyngraph model built here is the vertex set it was before the split existed.
    /// The 19 MG-validated processes staying bit-for-bit is the other half of this
    /// statement; this half says why they do.
    #[test]
    fn standard_model_vertices_have_one_fermion_flow_group() {
        let model = sm_model(SMRestrict::Default);
        for (name, vertex) in &model.vertices {
            let groups = flow_groups(vertex, &model.lorentz);
            assert_eq!(groups.len(), 1, "vertex '{name}' split into {groups:?}");
        }
    }

    /// The four-lepton vertex `e+ e- > mu+ mu-` reaches carries both pairings, and they
    /// are different fermion-line topologies: with the vertex's particles
    /// `[mu+, e-, e+, mu-]`, one pairing joins the two muons and the two electrons, the
    /// other joins each muon to an electron. Taking the first structure's map for the
    /// whole vertex would put some structures on the wrong lines, and the wrongness is
    /// invisible in the particle content — both readings pair a fermion with an
    /// antifermion.
    ///
    /// The two splits are also measured against each other. Splitting by coupling-order
    /// tuple (MadGraph's `add_interaction`) already separates *this* vertex's pairings,
    /// because its structures carry different Wilson coefficients; the same-flavour
    /// vertices are where it does not, and there this split is the only thing that does.
    #[test]
    fn the_four_lepton_vertex_carries_both_pairings() {
        let parsed = ParsedModel::parse(&smeftsim_dir()).expect("parse SMEFTsim");
        let names = |v: &Vertex| -> Vec<String> {
            v.particles
                .iter()
                .map(|&p| parsed.particles[p].name.clone())
                .collect()
        };

        // The row's vertex, as MadGraph's order-tuple split leaves it: several
        // interactions, between them both pairings.
        let mut across_splits: Vec<Vec<isize>> = parsed
            .vertices
            .values()
            .filter(|v| names(v) == ["mu+", "e-", "e+", "mu-"])
            .flat_map(|v| flow_groups(v, &parsed.lorentz))
            .map(|g| g.spin_map)
            .collect();
        across_splits.sort();
        across_splits.dedup();
        assert_eq!(
            across_splits,
            vec![vec![1, 0, 3, 2], vec![3, 2, 1, 0]],
            "the mu/e four-lepton vertex"
        );

        // The same-flavour vertices, where one interaction carries both pairings and
        // this split is what separates them.
        let mixed: Vec<&String> = parsed
            .vertices
            .iter()
            .filter(|(_, v)| flow_groups(v, &parsed.lorentz).len() > 1)
            .map(|(name, _)| name)
            .collect();
        assert_eq!(
            mixed.len(),
            80,
            "interactions carrying more than one pairing"
        );
        let same_flavour = parsed
            .vertices
            .iter()
            .find(|(name, _)| *name == mixed[0])
            .map(|(_, v)| names(v))
            .unwrap();
        assert_eq!(same_flavour[0], same_flavour[2], "{same_flavour:?}");
        assert_eq!(same_flavour[1], same_flavour[3], "{same_flavour:?}");
    }
}
