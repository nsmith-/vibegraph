//! Root a diagram at a given vertex
//!
//! This module converts an undirected diagram graph into a directed acyclic graph (DAG)
//! by choosing an arbitrary root vertex and directing all edges toward it. This transforms
//! the diagram structure into an evaluation tree where child vertices feed their
//! off-shell currents upward to parents.
//!
//! The key insight: an undirected Feynman diagram has no intrinsic evaluation order,
//! but choosing a root makes it a tree. We do this by starting at any external vertex
//! and recursively processing unvisited neighbors.
//!
//! Rooting is a two-pass walk:
//! 1. [`RawDiagramTree`] — pure topology with model ids interned ([`RawNode`]). No
//!    Lorentz rooting, no wavefunctions.
//! 2. [`DiagramEvalTree`] — the evaluable tree ([`EvalNode`]): each vertex's Lorentz
//!    structure is rooted at its output leg, and nodes are typed by what they
//!    produce (external wavefunction, off-shell current, propagator, amplitude).

use std::collections::HashSet;

use feyngraph::diagram::view::{DiagramView, LegView, VertexView};
use itertools::Either;

use crate::helas::eval::ast::{ExtLegInfo, PropInfo, VertexInfo};
use crate::helas::eval::tree::Tree;
use crate::helas::repr::numbers::Charge;
use crate::ufo::particles::ParticleId;
use crate::ufo::vertices::VertexId;
use crate::ufo::UFOModel;

use super::compile::CompileError;
use super::root_lorentz::RootLorentzError;

/// Errors from Pass 1: walking the diagram topology and interning model ids.
#[derive(Clone, Debug, thiserror::Error)]
pub enum RootDiagramError {
    /// A leg's particle name is absent from the UFO model.
    #[error("particle not found in model: {0}")]
    ParticleNotFound(String),
    /// A vertex's interaction name is absent from the UFO model.
    #[error("vertex not found in model: {0}")]
    VertexNotFound(String),
    /// feyngraph's is_anti flag disagrees with the model's pdg-code sign.
    #[error(
        "antiparticle flag mismatch for {name}: feyngraph is_anti={is_anti}, model pdg_code={pdg}"
    )]
    AntiparticleMismatch {
        name: String,
        is_anti: bool,
        pdg: i64,
    },
    /// The output (result) leg of a vertex resolved to an external leg, which has no
    /// off-shell continuation.
    #[error("an external leg cannot be the result leg of a vertex")]
    ExternalLegAsResult,
}

// ───────────────────────────── Pass 1: raw topology tree ─────────────────────────────

/// Node id into a [`RawDiagramTree`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RawNodeId(usize);

/// A node in the raw rooted-diagram tree: pure topology with interned model ids,
/// before Lorentz rooting or wavefunction construction.
#[derive(Clone, Debug)]
enum RawNode {
    /// External leg (tree leaf).
    Leg {
        particle: ParticleId,
        leg_idx: usize,
        charge: Charge,
        spin: i32,
    },
    /// A vertex. `result_leg_idx` is the output (continuation) leg for a non-root
    /// vertex, or `None` for the root. `children` are the input nodes in vertex-leg
    /// order, with the output-leg position omitted.
    Vertex {
        vertex: VertexId,
        result_leg_idx: Option<usize>,
        children: Vec<RawNodeId>,
    },
}

impl RawNode {
    fn children(&self) -> Vec<RawNodeId> {
        match self {
            RawNode::Leg { .. } => vec![],
            RawNode::Vertex { children, .. } => children.clone(),
        }
    }
}

/// Pure-topology rooted tree produced by the first walk.
struct RawDiagramTree {
    nodes: Vec<RawNode>,
    root: RawNodeId,
}

impl Tree for RawDiagramTree {
    type Item = RawNode;
    type NodeId = RawNodeId;

    fn children(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.value(node).children().into_iter()
    }

    fn value(&self, node: Self::NodeId) -> &Self::Item {
        &self.nodes[node.0]
    }

    fn root(&self) -> Self::NodeId {
        self.root
    }

    fn iter(&self) -> impl Iterator<Item = Self::NodeId> {
        (0..self.nodes.len()).map(RawNodeId)
    }
}

/// Builder for the raw topology tree.
struct RawBuilder<'a> {
    model: &'a UFOModel,
    nodes: Vec<RawNode>,
    processed_vertices: HashSet<usize>,
}

impl<'a> RawBuilder<'a> {
    fn new(model: &'a UFOModel) -> Self {
        RawBuilder {
            model,
            nodes: Vec::new(),
            processed_vertices: HashSet::new(),
        }
    }

    fn add(&mut self, node: RawNode) -> RawNodeId {
        let id = RawNodeId(self.nodes.len());
        self.nodes.push(node);
        id
    }

    fn make_leg(&mut self, leg: LegView) -> Result<RawNodeId, RootDiagramError> {
        let particle = leg.particle();
        let particle_id = self
            .model
            .particle_id(particle.name())
            .ok_or_else(|| RootDiagramError::ParticleNotFound(particle.name().to_string()))?;

        let model_particle = self.model.particle(particle_id);
        // Check that feyngraph's is_anti flag is consistent with the UFO model.
        // Use pdg_code < 0 (not charge sign) because up-type quarks have positive
        // charge yet are particles (is_anti=false), breaking the charge-based check.
        if particle.is_anti() != (model_particle.pdg_code < 0) {
            return Err(RootDiagramError::AntiparticleMismatch {
                name: particle.name().to_string(),
                is_anti: particle.is_anti(),
                pdg: model_particle.pdg_code,
            });
        }

        Ok(self.add(RawNode::Leg {
            particle: particle_id,
            leg_idx: leg.index(),
            charge: match particle.is_anti() {
                true => Charge::Antiparticle,
                false => Charge::Particle,
            },
            spin: model_particle.spin,
        }))
    }

    /// Recursively walk the diagram tree from a root vertex.
    ///
    /// Process all propagators attached to `vtx`. For each:
    /// - external leg: emit a `Leg` child
    /// - internal (unvisited): recurse to the other vertex and keep its node as a child
    /// - internal (visited): skip — this is the output leg we came from
    ///
    /// `children` collects the input nodes in vertex-leg order with the output-leg
    /// position omitted; `result_leg_idx` records that position so the second pass
    /// can root the vertex's Lorentz structure there (the rooted tree's `Leg(i)`
    /// references are then compacted to index this gap-free child list directly).
    fn walk_vertex(
        &mut self,
        vtx: &VertexView,
        result_leg_idx: Option<usize>,
    ) -> Result<RawNodeId, RootDiagramError> {
        self.processed_vertices.insert(vtx.id());
        let mut children = vec![];
        for (idx, prop) in vtx.propagators_ordered().enumerate() {
            let is_upstream = result_leg_idx.is_some_and(|ir| ir == idx);
            match (is_upstream, prop) {
                (false, Either::Left(leg)) => {
                    children.push(self.make_leg(leg)?);
                }
                (false, Either::Right(prop)) => {
                    for (vidx, next_vtx) in prop.vertices().enumerate() {
                        // Either it is the vertex we are at or the next one the propagator goes to
                        if !self.processed_vertices.contains(&next_vtx.id()) {
                            let this_prop = prop.ray_index_ordered(vidx);
                            children.push(self.walk_vertex(&next_vtx, Some(this_prop))?);
                        }
                    }
                }
                (true, Either::Left(_)) => {
                    return Err(RootDiagramError::ExternalLegAsResult);
                }
                (true, Either::Right(_)) => {
                    // The output (result) leg — the propagator we came from. It has
                    // no input wavefunction, so it contributes no child; the gap is
                    // tracked by `result_leg_idx`.
                }
            }
        }

        let vertex = self
            .model
            .vertex_id(vtx.interaction().name())
            .ok_or_else(|| {
                RootDiagramError::VertexNotFound(vtx.interaction().name().to_string())
            })?;

        Ok(self.add(RawNode::Vertex {
            vertex,
            result_leg_idx,
            children,
        }))
    }
}

// ───────────────────────────── Pass 2: evaluable tree ─────────────────────────────

/// Node id into a [`DiagramEvalTree`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvalNodeId(usize);

/// A node in the evaluable diagram tree, typed by what it produces.
#[derive(Clone, Debug)]
pub enum EvalNode {
    /// External wavefunction (leaf): built from momentum + helicity at eval time.
    External(ExtLegInfo),
    /// Off-shell current: apply the vertex to its input children. `children` are in
    /// vertex-leg order with the output position omitted, and the vertex's rooted
    /// Lorentz tree indexes them directly (its leg references were compacted at
    /// compile time, see `LorentzEvalTree::build_at_leg`).
    OffShellCurrent {
        info: VertexInfo,
        children: Vec<EvalNodeId>,
    },
    /// Propagator applied to its single child off-shell current.
    Propagate { info: PropInfo, child: EvalNodeId },
    /// Root vertex: contract all children into the scalar amplitude.
    ContractAmplitude {
        info: VertexInfo,
        children: Vec<EvalNodeId>,
    },
}

impl EvalNode {
    fn children(&self) -> Vec<EvalNodeId> {
        match self {
            EvalNode::External(_) => vec![],
            EvalNode::OffShellCurrent { children, .. } => children.clone(),
            EvalNode::Propagate { child, .. } => vec![*child],
            EvalNode::ContractAmplitude { children, .. } => children.clone(),
        }
    }

    fn render(&self, body: String) -> String {
        match self {
            EvalNode::External(info) => format!("ExternalWf({})", info),
            EvalNode::OffShellCurrent { info, .. } => {
                format!("OffShellCurrent({}; {})", info, body)
            }
            EvalNode::Propagate { info, .. } => {
                format!("Propagate({:?}; {})", info.id, body)
            }
            EvalNode::ContractAmplitude { info, .. } => {
                format!("ContractAmplitude({}; {})", info, body)
            }
        }
    }
}

/// The evaluable rooted tree for a single diagram (second-pass output).
#[derive(Clone, Debug)]
pub struct DiagramEvalTree {
    nodes: Vec<EvalNode>,
    root: EvalNodeId,
}

impl Tree for DiagramEvalTree {
    type Item = EvalNode;
    type NodeId = EvalNodeId;

    fn children(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.value(node).children().into_iter()
    }

    fn value(&self, node: Self::NodeId) -> &Self::Item {
        &self.nodes[node.0]
    }

    fn root(&self) -> Self::NodeId {
        self.root
    }

    fn iter(&self) -> impl Iterator<Item = Self::NodeId> {
        (0..self.nodes.len()).map(EvalNodeId)
    }
}

impl DiagramEvalTree {
    fn add(nodes: &mut Vec<EvalNode>, node: EvalNode) -> EvalNodeId {
        let id = EvalNodeId(nodes.len());
        nodes.push(node);
        id
    }

    /// Bake a raw topology tree into the evaluable tree: root each vertex's Lorentz
    /// structure and type each node by what it produces.
    fn bake(raw: &RawDiagramTree, model: &UFOModel) -> Result<Self, RootLorentzError> {
        let mut nodes = Vec::with_capacity(raw.nodes.len());
        let root = Self::bake_node(raw, raw.root, model, &mut nodes)?;
        Ok(DiagramEvalTree { nodes, root })
    }

    fn bake_node(
        raw: &RawDiagramTree,
        id: RawNodeId,
        model: &UFOModel,
        nodes: &mut Vec<EvalNode>,
    ) -> Result<EvalNodeId, RootLorentzError> {
        match raw.value(id) {
            RawNode::Leg {
                particle,
                leg_idx,
                charge,
                spin,
            } => Ok(Self::add(
                nodes,
                EvalNode::External(ExtLegInfo {
                    id: *particle,
                    leg_idx: *leg_idx,
                    charge: *charge,
                    spin: *spin,
                }),
            )),
            RawNode::Vertex {
                vertex,
                result_leg_idx,
                children,
            } => {
                let baked: Vec<EvalNodeId> = children
                    .iter()
                    .map(|&c| Self::bake_node(raw, c, model, nodes))
                    .collect::<Result<Vec<_>, _>>()?;
                match result_leg_idx {
                    Some(ri) => {
                        // Internal vertex: off-shell current rooted at the output leg,
                        // wrapped by the propagator on that leg.
                        let info = VertexInfo::from_ufo(model, *vertex, Some(*ri))?;
                        let prop_id = model.vertex_def(*vertex).particles[*ri];
                        let current = Self::add(
                            nodes,
                            EvalNode::OffShellCurrent {
                                info,
                                children: baked,
                            },
                        );
                        Ok(Self::add(
                            nodes,
                            EvalNode::Propagate {
                                info: PropInfo { id: prop_id },
                                child: current,
                            },
                        ))
                    }
                    None => {
                        // Root vertex: contract all legs into the scalar amplitude.
                        let info = VertexInfo::from_ufo(model, *vertex, None)?;
                        Ok(Self::add(
                            nodes,
                            EvalNode::ContractAmplitude {
                                info,
                                children: baked,
                            },
                        ))
                    }
                }
            }
        }
    }

    fn render_expression(&self) -> String {
        self.fold_recursive(
            &|node, acc| node.render(acc),
            &|acc, r| if acc.is_empty() { r } else { acc + ", " + &r },
            String::new(),
            self.root,
        )
    }
}

impl std::fmt::Display for DiagramEvalTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.render_expression())
    }
}

// ───────────────────────────── Fermion-line sign ─────────────────────────────

/// Trace the fermion line that enters `start_vtx` at ordered ray `in_ray`,
/// following spinor connectivity until it reaches an external leg.
///
/// Connectivity comes from *our* recomputed `spin_map` (UFOModel
/// `LorentzStructure`), indexed by the vertex's ordered rays — which align with
/// feyngraph's `propagators_ordered` because we feed feyngraph that same particle
/// ordering. Returns the external leg index where the line terminates and whether
/// it passed through at least one internal propagator (i.e. is an off-shell spine).
fn trace_fermion_line(
    view: &DiagramView,
    model: &UFOModel,
    start_vtx_id: usize,
    in_ray: usize,
) -> (usize, bool) {
    let mut vtx_id = start_vtx_id;
    let mut in_ray = in_ray;
    let mut passed_internal = false;
    // Tree diagrams terminate; the bound only guards against pathological loops.
    for _ in 0..1024 {
        let vtx = view.vertex(vtx_id);
        let vid = model
            .vertex_id(vtx.interaction().name())
            .expect("vertex in UFO");
        let lid = model.vertex_def(vid).lorentz[0];
        let out_ray = model.lorentz_struct(lid).spin_map[in_ray] as usize;
        // Extract Copy values so the `propagators_ordered` borrow of `vtx` is
        // released before the next iteration rebinds it from `view`.
        let next: (usize, usize) = match vtx
            .propagators_ordered()
            .nth(out_ray)
            .expect("spin_map ray index in range")
        {
            Either::Left(leg) => return (leg.index(), passed_internal),
            Either::Right(prop) => {
                let n = if prop.vertex(0).id() == vtx_id { 1 } else { 0 };
                (prop.vertex(n).id(), prop.ray_index_ordered(n))
            }
        };
        passed_internal = true;
        vtx_id = next.0;
        in_ray = next.1;
    }
    panic!("fermion line trace did not terminate");
}

/// Relative fermion sign that feyngraph's connectivity-based `view.sign()` omits.
///
/// MadGraph assigns a relative −1 to a diagram whose off-shell fermion **spine**
/// is the line joining the two **initial-state** fermions: in MadGraph's
/// all-outgoing convention that pair is crossed, and chaining it as a propagator
/// (rather than closing it at a single vertex) picks up the crossing sign. Across
/// such diagrams `view.sign()` is uniform, so we apply the correction here.
///
/// Detected structurally by tracing each external fermion line: flip when a line
/// connects two incoming legs through at least one internal propagator. Validated
/// against MadGraph per-diagram amplitudes for e+e-→μ+μ-τ+τ- and
/// u u~→c c~ e+e- μ+μ- (QCD=0).
pub(super) fn initial_state_spine_sign(view: &DiagramView, model: &UFOModel) -> i8 {
    let n_in = view.incoming().count();
    let mut visited: HashSet<usize> = HashSet::new();
    let mut sign: i8 = 1;
    for leg in view.incoming().chain(view.outgoing()) {
        let li = leg.index();
        if !leg.particle().is_fermi() || !visited.insert(li) {
            continue;
        }
        let (other, passed_internal) =
            trace_fermion_line(view, model, leg.vertex().id(), leg.ray_index_ordered());
        visited.insert(other);
        if passed_internal && li < n_in && other < n_in {
            sign = -sign;
        }
    }
    sign
}

// ───────────────────────────── Rooting entry point ─────────────────────────────

/// Root a diagram into an evaluable tree.
///
/// Walk from an arbitrary root vertex to build the raw topology tree (Pass 1), then
/// bake it into the evaluable [`DiagramEvalTree`] (Pass 2: Lorentz structures rooted
/// at output legs, nodes typed by produced wavefunction). Both pass errors surface
/// through the [`CompileError`] umbrella.
///
/// # Arguments
/// * `view` — DiagramView with vertices and propagators
/// * `model` — UFO model for vertex/particle/coupling lookups
pub(super) fn root_tree(
    view: &DiagramView,
    model: &UFOModel,
) -> Result<DiagramEvalTree, CompileError> {
    // Choose an arbitrary root vertex (the first one) and walk the tree from there.
    let mut builder = RawBuilder::new(model);
    let raw_root = builder.walk_vertex(&view.vertex(0), None)?;
    let raw = RawDiagramTree {
        nodes: builder.nodes,
        root: raw_root,
    };

    Ok(DiagramEvalTree::bake(&raw, model)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, DiagramSet, ParsingOptions};
    use crate::ufo::UFOModel;
    use std::sync::OnceLock;

    static SM_MODEL: OnceLock<UFOModel> = OnceLock::new();
    fn sm_model() -> &'static UFOModel {
        SM_MODEL.get_or_init(|| {
            let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
            let path = std::path::Path::new(&manifest).join("../research/refs/mg5amcnlo/models/sm");
            UFOModel::load(&path, None).expect("SM UFO not found")
        })
    }
    fn generate(process: &str) -> Vec<DiagramSet> {
        let opts = ParsingOptions::default();
        let card = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
        generate_from_proc_card(&card, sm_model()).unwrap()
    }

    #[test]
    fn test_walk() {
        let model = sm_model();
        let sets = generate("e+ e- > mu+ mu-");
        for set in sets {
            for view in set.diagrams.views() {
                println!("Testing diagram {}", view);
                let tree = root_tree(&view, model).expect("rooting failed");
                println!("Generated tree: {}", tree);
            }
        }
    }
}
