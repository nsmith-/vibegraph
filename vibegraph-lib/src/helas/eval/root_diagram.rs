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

use crate::diagrams::DiagramSet;
use crate::helas::eval::diagram_eval::{ExtLegInfo, PropInfo, VertexInfo};
use crate::helas::eval::tree::Tree;
use crate::helas::repr::numbers::Charge;
use crate::ufo::particles::ParticleId;
use crate::ufo::vertices::VertexId;
use crate::ufo::UFOModel;

use super::error::{CompileError, RootDiagramError};
use super::root_lorentz::{Flow, LegFlow, RootLorentzError};

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

impl EvalNodeId {
    /// Reference a node by its position in a hand-built node list (tests only).
    #[cfg(test)]
    pub fn new(idx: usize) -> Self {
        EvalNodeId(idx)
    }
}

/// A node in the evaluable diagram tree, typed by what it produces.
#[derive(Clone, Debug)]
pub enum EvalNode {
    /// External wavefunction (leaf): built from momentum + helicity at eval time.
    External(ExtLegInfo),
    /// Off-shell current: apply the vertex to its input children. `children` are in
    /// vertex-leg order with the output position omitted, and the vertex's rooted
    /// Lorentz tree indexes them directly (its leg references were compacted at
    /// compile time, see `LorentzEvalTree::build_at_leg`). `flow` is the spinor flow of
    /// the output current (`Some` iff the output leg is a fermion), inherited from the
    /// continuing fermion input.
    OffShellCurrent {
        info: VertexInfo,
        flow: Option<Flow>,
        children: Vec<EvalNodeId>,
    },
    /// Propagator applied to its single child off-shell current. `flow` matches the
    /// current it wraps (a propagator preserves fermion flow).
    Propagate {
        info: PropInfo,
        flow: Option<Flow>,
        child: EvalNodeId,
    },
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

    /// Spinor flow of the wavefunction this node outputs (`None` for bosonic / scalar
    /// outputs).
    fn out_flow(&self) -> Option<Flow> {
        match self {
            EvalNode::External(info) => info.flow(),
            EvalNode::OffShellCurrent { flow, .. } => *flow,
            EvalNode::Propagate { flow, .. } => *flow,
            EvalNode::ContractAmplitude { .. } => None,
        }
    }

    fn render(&self, body: String) -> String {
        match self {
            EvalNode::External(info) => {
                format!("ExternalWf{}({})", flow_tag(info.flow()), info)
            }
            EvalNode::OffShellCurrent { info, flow, .. } => {
                format!("OffShellCurrent{}({}; {})", flow_tag(*flow), info, body)
            }
            EvalNode::Propagate { info, flow, .. } => {
                format!("Propagate{}({:?}; {})", flow_tag(*flow), info.id, body)
            }
            EvalNode::ContractAmplitude { info, .. } => {
                format!("ContractAmplitude({}; {})", info, body)
            }
        }
    }
}

/// Render a baked flow as a bracketed tag (`[ket]`/`[bra]`), or empty for a bosonic /
/// scalar node.
fn flow_tag(flow: Option<Flow>) -> String {
    flow.map(|f| format!("[{f}]")).unwrap_or_default()
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
    /// structure and type each node by what it produces. `n_in` is the number of
    /// incoming externals, used to flag each leg's flow direction.
    fn bake(raw: &RawDiagramTree, model: &UFOModel, n_in: usize) -> Result<Self, RootLorentzError> {
        let mut nodes = Vec::with_capacity(raw.nodes.len());
        let (root, _) = Self::bake_node(raw, raw.root, model, n_in, &mut nodes)?;
        Ok(DiagramEvalTree { nodes, root })
    }

    /// Bake one node, returning its id and the spinor binding of the wavefunction it
    /// produces (`None` for bosonic / scalar-amplitude outputs). The binding is
    /// resolved bottom-up: external legs from their charge/direction (with
    /// `crossed = !incoming`, since diagram enumeration presents outgoing legs in the
    /// all-incoming convention), and an off-shell fermion current (plus the propagator
    /// on it) inherits the binding of its continuing fermion input.
    fn bake_node(
        raw: &RawDiagramTree,
        id: RawNodeId,
        model: &UFOModel,
        n_in: usize,
        nodes: &mut Vec<EvalNode>,
    ) -> Result<(EvalNodeId, Option<LegFlow>), RootLorentzError> {
        match raw.value(id) {
            RawNode::Leg {
                particle,
                leg_idx,
                charge,
                spin,
            } => {
                let info = ExtLegInfo {
                    id: *particle,
                    leg_idx: *leg_idx,
                    charge: *charge,
                    spin: *spin,
                    incoming: *leg_idx < n_in,
                };
                let bind = info.flow().map(|flow| LegFlow {
                    flow,
                    crossed: !info.incoming,
                });
                Ok((Self::add(nodes, EvalNode::External(info)), bind))
            }
            RawNode::Vertex {
                vertex,
                result_leg_idx,
                children,
            } => {
                let baked: Vec<(EvalNodeId, Option<LegFlow>)> = children
                    .iter()
                    .map(|&c| Self::bake_node(raw, c, model, n_in, nodes))
                    .collect::<Result<Vec<_>, _>>()?;
                let child_ids: Vec<EvalNodeId> = baked.iter().map(|(id, _)| *id).collect();
                match result_leg_idx {
                    Some(ri) => {
                        // Internal vertex: off-shell current rooted at the output leg,
                        // wrapped by the propagator on that leg.
                        let prop_id = model.vertex_def(*vertex).particles[*ri];
                        // The current keeps the binding of its continuing fermion input
                        // (one such child for an FFV) iff the output leg is itself a
                        // fermion; a bosonic output carries none. The bindings are
                        // passed into the Lorentz rooting so it picks the in/out gamma
                        // routine and detects reversed/crossed pairs.
                        let bind = (model.particle(prop_id).spin == 2)
                            .then(|| baked.iter().find_map(|(_, f)| *f))
                            .flatten();
                        // Per-leg bindings in vertex-leg order: children with the
                        // output's binding spliced in at its position, so the Lorentz
                        // rooting can compare each leg to its UFO slot.
                        let mut flows: Vec<Option<LegFlow>> =
                            baked.iter().map(|(_, f)| *f).collect();
                        flows.insert(*ri, bind);
                        let info = VertexInfo::from_ufo(model, *vertex, Some(*ri), &flows)?;
                        let flow = bind.map(|lf| lf.flow);
                        let current = Self::add(
                            nodes,
                            EvalNode::OffShellCurrent {
                                info,
                                flow,
                                children: child_ids,
                            },
                        );
                        Ok((
                            Self::add(
                                nodes,
                                EvalNode::Propagate {
                                    info: PropInfo { id: prop_id },
                                    flow,
                                    child: current,
                                },
                            ),
                            bind,
                        ))
                    }
                    None => {
                        // Root vertex: contract all legs into the scalar amplitude — a
                        // scalar sink, so no fermion output flow; every leg is a child.
                        let flows: Vec<Option<LegFlow>> = baked.iter().map(|(_, f)| *f).collect();
                        let info = VertexInfo::from_ufo(model, *vertex, None, &flows)?;
                        Ok((
                            Self::add(
                                nodes,
                                EvalNode::ContractAmplitude {
                                    info,
                                    children: child_ids,
                                },
                            ),
                            None,
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

// ───────────────────────── Fermion-line sign (test oracle) ─────────────────────────
//
// The production spine sign is derived from the baked spinor flow
// ([`spine_sign_from_flow`]). The independent `spin_map`-tracing implementation below
// is retained as the cross-check oracle for `spine_sign_from_flow_matches_heuristic`.

/// Trace the fermion line that enters `start_vtx` at ordered ray `in_ray`,
/// following spinor connectivity until it reaches an external leg.
///
/// Connectivity comes from *our* recomputed `spin_map` (UFOModel
/// `LorentzStructure`), indexed by the vertex's ordered rays — which align with
/// feyngraph's `propagators_ordered` because we feed feyngraph that same particle
/// ordering. Returns the external leg index where the line terminates and whether
/// it passed through at least one internal propagator (i.e. is an off-shell spine).
#[cfg(test)]
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
#[cfg(test)]
fn initial_state_spine_sign(view: &DiagramView, model: &UFOModel) -> i8 {
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

// ──────────────────────── Spine sign from baked flow ────────────────────────

/// Descend the fermion line from `node` (a fermion child of a pair-sink) to its
/// terminal external leg, reporting whether the descent crossed an internal fermion
/// propagator. Follows the continuing fermion (the lone `Some`-flow child) through each
/// off-shell current; a `Propagate` is exactly one internal fermion propagator.
fn descend_fermion_line(tree: &DiagramEvalTree, node: EvalNodeId) -> (bool, bool) {
    match tree.value(node) {
        EvalNode::External(info) => (info.incoming, false),
        EvalNode::Propagate { child, .. } => {
            let (incoming, _) = descend_fermion_line(tree, *child);
            (incoming, true)
        }
        EvalNode::OffShellCurrent { children, .. } => {
            let cont = children
                .iter()
                .copied()
                .find(|&c| tree.value(c).out_flow().is_some())
                .expect("a fermion off-shell current has a continuing fermion input");
            descend_fermion_line(tree, cont)
        }
        EvalNode::ContractAmplitude { .. } => {
            unreachable!("the amplitude root is never reached while descending a fermion line")
        }
    }
}

/// Derive the initial-state spine sign purely from the baked spinor flow, using only
/// the rooted evaluation tree we already build — no second graph walk. (The
/// `spin_map`-tracing `initial_state_spine_sign` is kept as a test oracle and proven
/// equivalent by `spine_sign_from_flow_matches_heuristic`.)
///
/// A fermion line terminates at any vertex node that outputs a non-fermion yet has two
/// fermion children: an FFV/FFS current rooted at its boson leg, or the root
/// contraction. Descend both ends to their external legs and flip the diagram sign when
/// the line joins two incoming legs through at least one internal fermion propagator —
/// MadGraph's crossing sign for an initial-state fermion pair carried as an off-shell
/// spine. Each fermion line meets exactly one such sink, so every line is counted once.
pub(super) fn spine_sign_from_flow(tree: &DiagramEvalTree) -> i8 {
    let mut sign = 1i8;
    for id in tree.iter() {
        let node = tree.value(id);
        let is_sink = matches!(node, EvalNode::ContractAmplitude { .. })
            || matches!(node, EvalNode::OffShellCurrent { flow: None, .. });
        if !is_sink {
            continue;
        }
        let fermions: Vec<EvalNodeId> = node
            .children()
            .into_iter()
            .filter(|&c| tree.value(c).out_flow().is_some())
            .collect();
        // SM vertices pair fermions, so a sink has 0 or 2 fermion legs (one line).
        if let [a, b] = fermions[..] {
            let (inc_a, internal_a) = descend_fermion_line(tree, a);
            let (inc_b, internal_b) = descend_fermion_line(tree, b);
            if inc_a && inc_b && (internal_a || internal_b) {
                sign = -sign;
            }
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

    Ok(DiagramEvalTree::bake(&raw, model, view.incoming().count())?)
}

// ───────────────────────────── Per-diagram artifact ─────────────────────────────

/// A compiled representation of a single Feynman diagram.
///
/// Built once from a `DiagramView` + `UFOModel`. The diagram is a rooted
/// [`DiagramEvalTree`]: external legs are leaves, internal vertices are off-shell
/// currents wrapped by propagators, and the root contracts into the scalar amplitude.
#[derive(Clone, Debug)]
pub struct DiagramEval {
    /// Number of external legs (determines array indexing for momenta)
    pub n_ext: usize,
    /// Rooted evaluation tree for this diagram
    pub tree: DiagramEvalTree,
    /// Symmetry factor: 1 / (vertex_sym × propagator_sym)
    pub symmetry_factor: f64,
    /// ±1 from the diagram's Fermi permutation sign
    pub fermi_sign: i8,
}

impl DiagramEval {
    /// Assemble a single diagram from a hand-specified node list (tests only).
    ///
    /// The last node is the root; children reference earlier nodes by index (see
    /// [`EvalNodeId::new`]). Symmetry factor and Fermi sign are trivial (1, +1), so the
    /// reconstructed amplitude is exactly the rooted contraction of the given nodes —
    /// used to drive single-vertex primitives through the production `run_forward` path.
    #[cfg(test)]
    pub fn from_nodes(n_ext: usize, nodes: Vec<EvalNode>) -> Self {
        let root = EvalNodeId(nodes.len() - 1);
        DiagramEval {
            n_ext,
            tree: DiagramEvalTree { nodes, root },
            symmetry_factor: 1.0,
            fermi_sign: 1,
        }
    }

    /// Internal propagator particle ids appearing in this diagram (one per
    /// `Propagate` node). Used to characterize a diagram by its propagator content.
    #[cfg(test)]
    pub fn propagator_particles(&self) -> impl Iterator<Item = ParticleId> + '_ {
        self.tree.iter().filter_map(|id| match self.tree.value(id) {
            EvalNode::Propagate { info, .. } => Some(info.id),
            _ => None,
        })
    }
}

impl std::fmt::Display for DiagramEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Diagram(external legs {}): {}", self.n_ext, self.tree)
    }
}

/// Compile a single diagram into an evaluable [`DiagramEval`].
///
/// Roots the diagram into its evaluation tree (topology + Lorentz structures) and
/// attaches the per-diagram metadata (external-leg count, symmetry factor, and the
/// fermion-flow sign, including the initial-state spine correction derived from the
/// baked spinor flow via [`spine_sign_from_flow`]).
fn compile_single_diagram(
    view: &DiagramView,
    model: &UFOModel,
) -> Result<DiagramEval, CompileError> {
    let tree = root_tree(view, model)?;
    let fermi_sign = view.sign() * spine_sign_from_flow(&tree);
    Ok(DiagramEval {
        n_ext: view.legs().count(),
        tree,
        symmetry_factor: 1.0 / view.symmetry_factor() as f64,
        fermi_sign,
    })
}

/// Compile all diagrams from a DiagramSet into rooted [`DiagramEval`]s.
///
/// For each diagram, recursively walks from an arbitrary root vertex to build a
/// directed evaluation tree. External legs become leaves; internal vertices emit an
/// off-shell current + propagator pair; the root emits the amplitude contraction.
pub fn compile_diagram_ast(
    set: &DiagramSet,
    model: &UFOModel,
) -> Result<Vec<DiagramEval>, CompileError> {
    set.diagrams
        .views()
        .map(|view| compile_single_diagram(&view, model))
        .collect()
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

    /// Instrumentation dump (S18 fix item 2): per-vertex UFO slot order vs the actual
    /// bound children, for every ee→μμττ diagram. Prints, for each vertex:
    /// interaction name, UFO particle slots, result_leg_idx, and each
    /// `propagators_ordered` entry (external leg index + particle name + is_anti, or
    /// internal propagator particle), so slot↔leg binding and flow alignment can be
    /// read off directly.
    ///
    /// Run: cargo test -p vibegraph-lib --lib \
    ///        helas::eval::root_diagram::tests::probe_vertex_leg_binding -- --ignored --nocapture
    #[test]
    #[ignore]
    fn probe_vertex_leg_binding() {
        let model = sm_model();
        let sets = generate("e+ e- > mu+ mu- ta+ ta- QCD=0");
        for set in &sets {
            for (d, view) in set.diagrams.views().enumerate() {
                println!("── diagram {d} ──");
                for leg in view.legs() {
                    println!(
                        "  ext leg {}: {} (is_anti={})",
                        leg.index(),
                        leg.particle().name(),
                        leg.particle().is_anti()
                    );
                }
                for vi in 0..view.vertices().count() {
                    let vtx = view.vertex(vi);
                    let interaction = vtx.interaction().name().to_string();
                    let vid = model.vertex_id(&interaction).expect("vertex in UFO");
                    let slots: Vec<String> = model
                        .vertex_def(vid)
                        .particles
                        .iter()
                        .map(|pid| model.particle(*pid).name.clone())
                        .collect();
                    println!("  vertex {vi}: {interaction} slots={slots:?}");
                    for (ray, prop) in vtx.propagators_ordered().enumerate() {
                        match prop {
                            Either::Left(leg) => println!(
                                "    ray {ray} (slot {}): EXT leg {} {} (is_anti={})",
                                slots[ray],
                                leg.index(),
                                leg.particle().name(),
                                leg.particle().is_anti()
                            ),
                            Either::Right(p) => println!(
                                "    ray {ray} (slot {}): PROP {} [{}]",
                                slots[ray],
                                p.particle().name(),
                                p.momentum_str()
                            ),
                        }
                    }
                }
                let tree = root_tree(&view, model).expect("rooting failed");
                println!("  baked: {tree}");
            }
        }
    }

    /// The flow-derived spine sign must agree with the `spin_map`-tracing heuristic for
    /// every diagram, across processes with and without initial-state fermion spines.
    #[test]
    fn spine_sign_from_flow_matches_heuristic() {
        let model = sm_model();
        let processes = [
            "e+ e- > mu+ mu-",
            "e+ e- > e+ e-",
            "u u~ > d d~",
            "e+ e- > mu+ mu- ta+ ta-",
        ];
        let mut flipped_total = 0;
        for process in processes {
            for set in generate(process) {
                for (i, view) in set.diagrams.views().enumerate() {
                    let tree = root_tree(&view, model).expect("rooting failed");
                    let from_flow = spine_sign_from_flow(&tree);
                    let heuristic = initial_state_spine_sign(&view, model);
                    assert_eq!(
                        from_flow, heuristic,
                        "spine sign mismatch in `{process}` diagram {i}: \
                         flow={from_flow} heuristic={heuristic}"
                    );
                    if from_flow < 0 {
                        flipped_total += 1;
                    }
                }
            }
        }
        // The e+e-→μ+μ-τ+τ- e-spine class (8 diagrams) must actually exercise a flip,
        // so the agreement above is not vacuous.
        assert!(
            flipped_total >= 8,
            "expected at least the 8 e-spine flips, saw {flipped_total}"
        );
    }
}
