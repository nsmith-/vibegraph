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

use crate::diagrams::diagram::{Diagram, Leg, LegIdx, PropIdx, Ray, RaySlot, VtxIdx};
use crate::diagrams::DiagramSet;
use crate::helas::eval::diagram_eval::{ExtLegInfo, PropInfo, VertexInfo};
use crate::helas::eval::tree::Tree;
use crate::helas::repr::numbers::Charge;
use crate::ufo::particles::ParticleId;
use crate::ufo::vertices::VertexId;
use crate::ufo::UFOModel;

use super::error::{CompileError, RootDiagramError};
use super::root_lorentz::{Adjoint, LegAdjoint, RootLorentzError};

// ───────────────────────────── Pass 1: raw topology tree ─────────────────────────────

/// Node id into a [`RawDiagramTree`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RawNodeId(usize);

/// The output (continuation) leg of a non-root vertex: the ray slot the result
/// propagator occupies at this vertex, plus that propagator's index so its baked
/// momentum can be read when typing the propagator node.
#[derive(Clone, Copy, Debug)]
struct ResultLeg {
    slot: RaySlot,
    prop: PropIdx,
}

/// A node in the raw rooted-diagram tree: pure topology with interned model ids,
/// before Lorentz rooting or wavefunction construction.
#[derive(Clone, Debug)]
enum RawNode {
    /// External leg (tree leaf). `incoming` is the baked momentum-adjoint direction.
    Leg {
        particle: ParticleId,
        leg_idx: LegIdx,
        charge: Charge,
        spin: i32,
        incoming: bool,
    },
    /// A vertex. `result` is the output (continuation) leg for a non-root vertex, or
    /// `None` for the root. `children` are the input nodes in vertex-leg order, with
    /// the output-leg position omitted.
    Vertex {
        vertex: VertexId,
        result: Option<ResultLeg>,
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
    diagram: &'a Diagram,
    nodes: Vec<RawNode>,
    processed_vertices: HashSet<VtxIdx>,
}

impl<'a> RawBuilder<'a> {
    fn new(diagram: &'a Diagram) -> Self {
        RawBuilder {
            diagram,
            nodes: Vec::new(),
            processed_vertices: HashSet::new(),
        }
    }

    fn add(&mut self, node: RawNode) -> RawNodeId {
        let id = RawNodeId(self.nodes.len());
        self.nodes.push(node);
        id
    }

    fn make_leg(&mut self, leg: &Leg) -> RawNodeId {
        // The owned diagram already resolved the particle and baked charge/spin/adjoint at
        // the module boundary (`Diagram::from_view`), so this is a pure structural copy.
        self.add(RawNode::Leg {
            particle: leg.particle,
            leg_idx: leg.leg_idx,
            charge: leg.charge,
            spin: leg.spin,
            incoming: leg.incoming,
        })
    }

    /// Recursively walk the diagram tree from a root vertex.
    ///
    /// Process all rays attached to vertex `vtx`. For each:
    /// - external leg: emit a `Leg` child
    /// - internal (unvisited): recurse to the other vertex and keep its node as a child
    /// - internal (visited): skip — this is the output leg we came from
    ///
    /// `children` collects the input nodes in vertex-leg order with the output-leg
    /// position omitted; `result` records that ray slot and its propagator so the second
    /// pass can root the vertex's Lorentz structure there (the rooted tree's `Leg(i)`
    /// references are then compacted to index this gap-free child list directly) and read
    /// the propagator's baked momentum.
    fn walk_vertex(
        &mut self,
        vtx: VtxIdx,
        result: Option<ResultLeg>,
    ) -> Result<RawNodeId, RootDiagramError> {
        self.processed_vertices.insert(vtx);
        let mut children = vec![];
        for (idx, ray) in self
            .diagram
            .vertex(vtx)
            .rays
            .clone()
            .into_iter()
            .enumerate()
        {
            let is_upstream = result.is_some_and(|r| r.slot.0 == idx);
            match (is_upstream, ray) {
                (false, Ray::Leg(li)) => {
                    let leg = self.diagram.leg(li).clone();
                    children.push(self.make_leg(&leg));
                }
                (false, Ray::Prop { prop, end }) => {
                    // The propagator's other endpoint is the next vertex; recurse into it
                    // (unless already processed — that end is where we came from), rooting
                    // its Lorentz structure at the ray slot the line occupies there and
                    // carrying `prop` so the child can read its momentum.
                    let (next_vtx, next_slot) = self.diagram.prop(prop).endpoints[1 - end];
                    if !self.processed_vertices.contains(&next_vtx) {
                        let result = ResultLeg {
                            slot: next_slot,
                            prop,
                        };
                        children.push(self.walk_vertex(next_vtx, Some(result))?);
                    }
                }
                (true, Ray::Leg(_)) => {
                    return Err(RootDiagramError::ExternalLegAsResult);
                }
                (true, Ray::Prop { .. }) => {
                    // The output (result) leg — the propagator we came from. It has
                    // no input wavefunction, so it contributes no child; the gap is
                    // tracked by `result`.
                }
            }
        }

        Ok(self.add(RawNode::Vertex {
            vertex: self.diagram.vertex(vtx).interaction,
            result,
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
    /// compile time, see `LorentzEvalTree::build_at_leg`). `adjoint` is the spinor adjoint of
    /// the output current (`Some` iff the output leg is a fermion), inherited from the
    /// continuing fermion input.
    OffShellCurrent {
        info: VertexInfo,
        adjoint: Option<Adjoint>,
        children: Vec<EvalNodeId>,
    },
    /// Propagator applied to its single child off-shell current. `adjoint` matches the
    /// current it wraps (a propagator preserves fermion adjoint).
    Propagate {
        info: PropInfo,
        adjoint: Option<Adjoint>,
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

    /// Spinor adjoint of the wavefunction this node outputs (`None` for bosonic / scalar
    /// outputs).
    fn out_adjoint(&self) -> Option<Adjoint> {
        match self {
            EvalNode::External(info) => info.adjoint(),
            EvalNode::OffShellCurrent { adjoint, .. } => *adjoint,
            EvalNode::Propagate { adjoint, .. } => *adjoint,
            EvalNode::ContractAmplitude { .. } => None,
        }
    }

    fn render(&self, body: String) -> String {
        match self {
            EvalNode::External(info) => {
                format!("ExternalWf{}({})", adjoint_tag(info.adjoint()), info)
            }
            EvalNode::OffShellCurrent { info, adjoint, .. } => {
                format!(
                    "OffShellCurrent{}({}; {})",
                    adjoint_tag(*adjoint),
                    info,
                    body
                )
            }
            EvalNode::Propagate { info, adjoint, .. } => {
                format!(
                    "Propagate{}({:?}; {})",
                    adjoint_tag(*adjoint),
                    info.id,
                    body
                )
            }
            EvalNode::ContractAmplitude { info, .. } => {
                format!("ContractAmplitude({}; {})", info, body)
            }
        }
    }
}

/// Render a baked adjoint as a bracketed tag (`[ket]`/`[bra]`), or empty for a bosonic /
/// scalar node.
fn adjoint_tag(adjoint: Option<Adjoint>) -> String {
    adjoint.map(|f| format!("[{f}]")).unwrap_or_default()
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
    /// incoming externals, used to flag each leg's adjoint direction; `uncross` lists
    /// the final-state legs to type physically (see [`mixed_line_final_legs`]).
    fn bake(
        raw: &RawDiagramTree,
        diagram: &Diagram,
        model: &UFOModel,
        n_in: usize,
        uncross: &HashSet<LegIdx>,
    ) -> Result<Self, RootLorentzError> {
        let mut nodes = Vec::with_capacity(raw.nodes.len());
        let (root, _) = Self::bake_node(raw, raw.root, diagram, model, n_in, uncross, &mut nodes)?;
        Ok(DiagramEvalTree { nodes, root })
    }

    /// Bake one node, returning its id and the spinor binding of the wavefunction it
    /// produces (`None` for bosonic / scalar-amplitude outputs). The binding is resolved
    /// bottom-up: external legs from their charge/direction (with `crossed = !incoming`,
    /// since diagram enumeration presents outgoing legs in the all-incoming convention —
    /// except legs in `uncross`, restored to their physical particle/adjoint because their
    /// line partner is an initial-state leg), and an off-shell fermion current (plus the
    /// propagator on it) inherits the binding of its continuing fermion input. Each leg's
    /// `incoming` flag and each propagator's t-channel classification are read off the
    /// baked momentum (`Leg.incoming`, `Prop::is_spacelike`).
    fn bake_node(
        raw: &RawDiagramTree,
        id: RawNodeId,
        diagram: &Diagram,
        model: &UFOModel,
        n_in: usize,
        uncross: &HashSet<LegIdx>,
        nodes: &mut Vec<EvalNode>,
    ) -> Result<(EvalNodeId, Option<LegAdjoint>), RootLorentzError> {
        match raw.value(id) {
            RawNode::Leg {
                particle,
                leg_idx,
                charge,
                spin,
                incoming,
            } => {
                let uncrossed = uncross.contains(leg_idx);
                let (id, charge) = if uncrossed {
                    let anti = model
                        .particle_id(&model.particle(*particle).antiname)
                        .expect("antiparticle exists in model");
                    (anti, charge.anti())
                } else {
                    (*particle, *charge)
                };
                let info = ExtLegInfo {
                    id,
                    leg_idx: leg_idx.0,
                    charge,
                    spin: *spin,
                    incoming: *incoming,
                };
                let bind = info.adjoint().map(|adjoint| LegAdjoint {
                    adjoint,
                    crossed: !info.incoming && !uncrossed,
                });
                Ok((Self::add(nodes, EvalNode::External(info)), bind))
            }
            RawNode::Vertex {
                vertex,
                result,
                children,
            } => {
                let baked: Vec<(EvalNodeId, Option<LegAdjoint>)> = children
                    .iter()
                    .map(|&c| Self::bake_node(raw, c, diagram, model, n_in, uncross, nodes))
                    .collect::<Result<Vec<_>, _>>()?;
                let child_ids: Vec<EvalNodeId> = baked.iter().map(|(id, _)| *id).collect();
                match result {
                    Some(rl) => {
                        // Internal vertex: off-shell current rooted at the output leg,
                        // wrapped by the propagator on that leg.
                        let ri = rl.slot;
                        let prop_id = model.vertex_def(*vertex).particles[ri.0];
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
                        let mut flows: Vec<Option<LegAdjoint>> =
                            baked.iter().map(|(_, f)| *f).collect();
                        flows.insert(ri.0, bind);
                        let info = VertexInfo::from_ufo(model, *vertex, Some(ri.0), &flows)?;
                        let adjoint = bind.map(|lf| lf.adjoint);
                        let current = Self::add(
                            nodes,
                            EvalNode::OffShellCurrent {
                                info,
                                adjoint,
                                children: child_ids,
                            },
                        );
                        Ok((
                            Self::add(
                                nodes,
                                EvalNode::Propagate {
                                    info: PropInfo {
                                        id: prop_id,
                                        // Spacelike (t-channel) iff exactly one beam
                                        // flows through this line — read off its baked
                                        // momentum.
                                        t_channel: diagram.prop(rl.prop).is_spacelike(n_in),
                                    },
                                    adjoint,
                                    child: current,
                                },
                            ),
                            bind,
                        ))
                    }
                    None => {
                        // Root vertex: contract all legs into the scalar amplitude — a
                        // scalar sink, so no fermion output adjoint; every leg is a child.
                        let flows: Vec<Option<LegAdjoint>> =
                            baked.iter().map(|(_, f)| *f).collect();
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
// The production spine sign is derived from the baked spinor adjoint
// ([`spine_sign_from_flow`]). The independent `spin_map`-tracing implementation below
// is retained as the cross-check oracle for `spine_sign_from_flow_matches_heuristic`.

/// Trace the fermion line that enters `start_vtx` at ordered ray slot `in_ray`,
/// following spinor connectivity until it reaches an external leg.
///
/// Connectivity comes from *our* recomputed `spin_map` (UFOModel `LorentzStructure`),
/// indexed by the vertex's ordered ray slots — which are exactly the owned diagram's
/// `Vertex.rays`, since `Diagram::from_view` records them in interaction-slot order.
/// Returns the external leg index where the line terminates and whether it passed
/// through at least one internal propagator (i.e. is an off-shell spine).
#[cfg(test)]
fn trace_fermion_line(
    diagram: &Diagram,
    model: &UFOModel,
    start_vtx: VtxIdx,
    in_ray: RaySlot,
) -> (LegIdx, bool) {
    let mut vtx = start_vtx;
    let mut in_ray = in_ray;
    let mut passed_internal = false;
    // Tree diagrams terminate; the bound only guards against pathological loops.
    for _ in 0..1024 {
        let vertex = diagram.vertex(vtx);
        let lid = model.vertex_def(vertex.interaction).lorentz[0];
        let out_ray = model.lorentz_struct(lid).spin_map[in_ray.0] as usize;
        match vertex.rays[out_ray] {
            Ray::Leg(li) => return (li, passed_internal),
            Ray::Prop { prop, end } => {
                let (next_vtx, next_slot) = diagram.prop(prop).endpoints[1 - end];
                passed_internal = true;
                vtx = next_vtx;
                in_ray = next_slot;
            }
        }
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
fn initial_state_spine_sign(diagram: &Diagram, model: &UFOModel) -> i8 {
    let n_in = diagram.n_in;
    let mut visited: HashSet<LegIdx> = HashSet::new();
    let mut sign: i8 = 1;
    for leg in &diagram.legs {
        let li = leg.leg_idx;
        // A Dirac fermion leg (UFO spin code 2); mirrors the rest of this file.
        if leg.spin.abs() != 2 || !visited.insert(li) {
            continue;
        }
        let (attach_vtx, attach_slot) = diagram.leg_attachment(li);
        let (other, passed_internal) = trace_fermion_line(diagram, model, attach_vtx, attach_slot);
        visited.insert(other);
        if passed_internal && li.0 < n_in && other.0 < n_in {
            sign = -sign;
        }
        // Crossed (final–final) line: one −1 each, mirroring spine_sign_from_flow.
        if li.0 >= n_in && other.0 >= n_in {
            sign = -sign;
        }
    }
    sign
}

// ──────────────────────── Spine sign from baked adjoint ────────────────────────

/// Descend the fermion line from `node` (a fermion child of a pair-sink) to its
/// terminal external leg, reporting how many internal fermion propagators the descent
/// crossed. Follows the continuing fermion (the lone `Some`-adjoint child) through each
/// off-shell current; a `Propagate` is exactly one internal fermion propagator.
fn descend_fermion_line(tree: &DiagramEvalTree, node: EvalNodeId) -> (bool, usize) {
    match tree.value(node) {
        EvalNode::External(info) => (info.incoming, 0),
        EvalNode::Propagate { child, .. } => {
            let (incoming, n) = descend_fermion_line(tree, *child);
            (incoming, n + 1)
        }
        EvalNode::OffShellCurrent { children, .. } => {
            let cont = children
                .iter()
                .copied()
                .find(|&c| tree.value(c).out_adjoint().is_some())
                .expect("a fermion off-shell current has a continuing fermion input");
            descend_fermion_line(tree, cont)
        }
        EvalNode::ContractAmplitude { .. } => {
            unreachable!("the amplitude root is never reached while descending a fermion line")
        }
    }
}

/// Derive the fermion-line sign corrections purely from the baked spinor adjoint, using
/// only the rooted evaluation tree we already build — no second graph walk. (The
/// `spin_map`-tracing `initial_state_spine_sign` is kept as a test oracle and proven
/// equivalent by `spine_sign_from_flow_matches_heuristic`.)
///
/// A fermion line terminates at any vertex node that outputs a non-fermion yet has two
/// fermion children: an FFV/FFS current rooted at its boson leg, or the root
/// contraction. Each fermion line meets exactly one such sink, so every line is
/// counted once. Two per-line flips on top of feyngraph's permutation sign:
///
/// * **Initial spine**: a line joining two *incoming* legs through an ODD number of
///   internal fermion propagators — MadGraph's crossing sign for an initial-state
///   fermion pair carried as an off-shell spine, one −1 per reversed propagator (a
///   2-propagator initial spine flips twice = no net sign; pinned by the uux 2→6
///   per-diagram oracle, validation/madgraph/compare_amps.py).
/// * **Crossed line**: a line joining two *final-state* legs, evaluated in the
///   crossed (conjugate-wavefunction) representation, takes one −1 — the operator
///   reordering of the conjugated pair. Invisible while every diagram of a process
///   has the same crossed-line count (uniform sign); exposed and pinned by Bhabha,
///   where the s-channel has one crossed line and the t-channel none
///   (validation/madgraph/compare_amps.py, ee_to_ee).
pub(super) fn spine_sign_from_flow(tree: &DiagramEvalTree) -> i8 {
    let mut sign = 1i8;
    for id in tree.iter() {
        let node = tree.value(id);
        let is_sink = matches!(node, EvalNode::ContractAmplitude { .. })
            || matches!(node, EvalNode::OffShellCurrent { adjoint: None, .. });
        if !is_sink {
            continue;
        }
        let fermions: Vec<EvalNodeId> = node
            .children()
            .into_iter()
            .filter(|&c| tree.value(c).out_adjoint().is_some())
            .collect();
        // SM vertices pair fermions, so a sink has 0 or 2 fermion legs (one line).
        if let [a, b] = fermions[..] {
            let (inc_a, n_props_a) = descend_fermion_line(tree, a);
            let (inc_b, n_props_b) = descend_fermion_line(tree, b);
            if inc_a && inc_b && (n_props_a + n_props_b) % 2 == 1 {
                sign = -sign;
            }
            if !inc_a && !inc_b {
                sign = -sign;
            }
        }
    }
    sign
}

// ─────────────────────── Mixed-line (initial↔final) uncrossing ───────────────────────

/// Walk one raw subtree collecting closed fermion-line endpoint pairs into `pairs`,
/// returning the subtree's open fermion end (the external leg whose line continues
/// through this subtree's output), if any. A line closes at any vertex whose output
/// is not a fermion (or at the root contraction) yet has two fermion inputs.
fn collect_fermion_pairs(
    raw: &RawDiagramTree,
    model: &UFOModel,
    id: RawNodeId,
    pairs: &mut Vec<(LegIdx, LegIdx)>,
) -> Option<LegIdx> {
    match raw.value(id) {
        RawNode::Leg { leg_idx, spin, .. } => (spin.abs() == 2).then_some(*leg_idx),
        RawNode::Vertex {
            vertex,
            result,
            children,
        } => {
            let mut ends: Vec<LegIdx> = children
                .iter()
                .filter_map(|&c| collect_fermion_pairs(raw, model, c, pairs))
                .collect();
            let out_is_fermion = result.is_some_and(|rl| {
                let pid = model.vertex_def(*vertex).particles[rl.slot.0];
                model.particle(pid).spin.abs() == 2
            });
            if out_is_fermion {
                assert_eq!(ends.len(), 1, "a fermion current has one continuing input");
                ends.pop()
            } else {
                match ends[..] {
                    [] => None,
                    [a, b] => {
                        pairs.push((a, b));
                        None
                    }
                    _ => panic!("SM vertices pair fermions: 0 or 2 fermion legs per sink"),
                }
            }
        }
    }
}

/// Final-state legs whose fermion line connects to an *initial-state* leg (e.g. the
/// Bhabha t-channel electron line). Such legs must be typed by their physical
/// particle/adjoint — matching the reference HELAS externals — rather than by the
/// crossed (all-incoming) identity feyngraph reports: the crossed representation
/// C-conjugates the whole bilinear chain, which is only an identity when *both*
/// endpoints of the line conjugate together, i.e. for final–final pairs.
fn mixed_line_final_legs(raw: &RawDiagramTree, model: &UFOModel, n_in: usize) -> HashSet<LegIdx> {
    let mut pairs = Vec::new();
    let open = collect_fermion_pairs(raw, model, raw.root(), &mut pairs);
    assert!(open.is_none(), "all fermion lines close at some sink");
    pairs
        .into_iter()
        .filter_map(|(a, b)| match (a.0 < n_in, b.0 < n_in) {
            (true, false) => Some(b),
            (false, true) => Some(a),
            _ => None,
        })
        .collect()
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
/// * `diagram` — the owned, convention-baked diagram
/// * `model` — UFO model for vertex/particle/coupling lookups
pub(super) fn root_tree(
    diagram: &Diagram,
    model: &UFOModel,
) -> Result<DiagramEvalTree, CompileError> {
    // Choose an arbitrary root vertex (the first one) and walk the tree from there.
    let mut builder = RawBuilder::new(diagram);
    let raw_root = builder.walk_vertex(VtxIdx(0), None)?;
    let raw = RawDiagramTree {
        nodes: builder.nodes,
        root: raw_root,
    };

    let n_in = diagram.n_in;
    let uncross = mixed_line_final_legs(&raw, model, n_in);
    Ok(DiagramEvalTree::bake(&raw, diagram, model, n_in, &uncross)?)
}

// ───────────────────────────── Per-diagram artifact ─────────────────────────────

/// A compiled representation of a single Feynman diagram.
///
/// Built once from an owned [`Diagram`] + `UFOModel`. The diagram is a rooted
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
/// fermion-adjoint sign, including the initial-state spine correction derived from the
/// baked spinor adjoint via [`spine_sign_from_flow`]).
fn compile_single_diagram(
    diagram: &Diagram,
    model: &UFOModel,
) -> Result<DiagramEval, CompileError> {
    let tree = root_tree(diagram, model)?;
    let fermi_sign = diagram.sign * spine_sign_from_flow(&tree);
    Ok(DiagramEval {
        n_ext: diagram.n_ext(),
        tree,
        symmetry_factor: 1.0 / diagram.symmetry_factor as f64,
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
        .iter()
        .map(|diagram| compile_single_diagram(diagram, model))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, DiagramSet, ParsingOptions};
    use crate::ufo::sm::{sm_model as interned_sm, SMRestrict};
    use crate::ufo::UFOModel;
    use std::sync::{Arc, OnceLock};

    static SM_MODEL: OnceLock<Arc<UFOModel>> = OnceLock::new();
    fn sm_model() -> &'static UFOModel {
        SM_MODEL.get_or_init(|| interned_sm(SMRestrict::Default))
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
            for (d, diagram) in set.diagrams.iter().enumerate() {
                println!("Testing diagram {d}");
                let tree = root_tree(diagram, model).expect("rooting failed");
                println!("Generated tree: {tree}");
            }
        }
    }

    /// Instrumentation dump: per-vertex UFO slot order vs the actual bound rays, for
    /// every ee→μμττ diagram. Prints, for each vertex: interaction name, UFO particle
    /// slots, and each ray in slot order (external leg index + particle + charge, or
    /// internal propagator particle + momentum), so slot↔ray binding and adjoint alignment
    /// can be read off directly.
    ///
    /// Run: cargo test -p vibegraph-lib --lib \
    ///        helas::eval::root_diagram::tests::probe_vertex_leg_binding -- --ignored --nocapture
    #[test]
    #[ignore]
    fn probe_vertex_leg_binding() {
        let model = sm_model();
        let sets = generate("e+ e- > mu+ mu- ta+ ta- QCD=0");
        for set in &sets {
            for (d, diagram) in set.diagrams.iter().enumerate() {
                println!("── diagram {d} ──");
                for leg in &diagram.legs {
                    println!(
                        "  ext leg {}: {} ({}, incoming={})",
                        leg.leg_idx.0,
                        model.particle(leg.particle).name,
                        leg.charge,
                        leg.incoming
                    );
                }
                for (vi, vtx) in diagram.vertices.iter().enumerate() {
                    let slots: Vec<String> = model
                        .vertex_def(vtx.interaction)
                        .particles
                        .iter()
                        .map(|pid| model.particle(*pid).name.clone())
                        .collect();
                    let interaction = &model.vertex_def(vtx.interaction).name;
                    println!("  vertex {vi}: {interaction} slots={slots:?}");
                    for (slot, ray) in vtx.rays.iter().enumerate() {
                        match ray {
                            Ray::Leg(li) => {
                                let leg = diagram.leg(*li);
                                println!(
                                    "    ray {slot} (slot {}): EXT leg {} {} ({})",
                                    slots[slot],
                                    li.0,
                                    model.particle(leg.particle).name,
                                    leg.charge
                                );
                            }
                            Ray::Prop { prop, end } => {
                                let p = diagram.prop(*prop);
                                println!(
                                    "    ray {slot} (slot {}): PROP {} (end={end}, mom={:?})",
                                    slots[slot],
                                    model.particle(p.particle).name,
                                    p.momentum
                                );
                            }
                        }
                    }
                }
                let tree = root_tree(diagram, model).expect("rooting failed");
                println!("  baked: {tree}");
            }
        }
    }

    /// The adjoint-derived spine sign must agree with the `spin_map`-tracing heuristic for
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
                for (i, diagram) in set.diagrams.iter().enumerate() {
                    let tree = root_tree(diagram, model).expect("rooting failed");
                    let from_flow = spine_sign_from_flow(&tree);
                    let heuristic = initial_state_spine_sign(diagram, model);
                    assert_eq!(
                        from_flow, heuristic,
                        "spine sign mismatch in `{process}` diagram {i}: \
                         adjoint={from_flow} heuristic={heuristic}"
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
