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
#[cfg(test)]
use crate::diagrams::DiagramSet;
use crate::helas::eval::diagram_eval::{vertex_flow_group, ExtLegInfo, PropInfo, VertexInfo};
use crate::helas::eval::tree::Tree;
use crate::helas::repr::numbers::Charge;
use crate::ufo::particles::ParticleId;
use crate::ufo::topo::FlowGroup;
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
    /// the output-leg position omitted. `vtx` is the diagram vertex index, used to
    /// select this vertex's color structure from the per-diagram color-index chain.
    Vertex {
        vertex: VertexId,
        vtx: VtxIdx,
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
            vtx,
            result,
            children,
        }))
    }
}

// ───────────────────────────── Pass 2: evaluable tree ─────────────────────────────

/// Node id into a [`DiagramEvalTree`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EvalNodeId(usize);

impl EvalNodeId {
    /// Reference a node by its position in a hand-built node list (tests only).
    #[cfg(test)]
    pub(crate) fn new(idx: usize) -> Self {
        EvalNodeId(idx)
    }
}

/// A node in the evaluable diagram tree, typed by what it produces.
#[derive(Clone, Debug)]
pub(crate) enum EvalNode {
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
        /// Fermion lines this vertex *closes*, as pairs of positions in `children`.
        /// A vertex closes every fermion line whose two legs are both inputs here;
        /// the line continuing into the output leg is not among them, so a fermion
        /// current's continuing input is exactly the fermion child no pair names.
        fermion_pairs: Vec<(usize, usize)>,
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
        /// Fermion lines this vertex closes (see
        /// [`OffShellCurrent`](EvalNode::OffShellCurrent)); at the amplitude root
        /// every fermion leg is an input, so every line the vertex touches is here.
        fermion_pairs: Vec<(usize, usize)>,
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
pub(crate) struct DiagramEvalTree {
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
        chain: &[u8],
    ) -> Result<Self, RootLorentzError> {
        let mut baker = Baker {
            raw,
            diagram,
            model,
            n_in,
            uncross,
            chain,
            nodes: Vec::with_capacity(raw.nodes.len()),
        };
        let (root, _) = baker.bake_node(raw.root)?;
        Ok(DiagramEvalTree {
            nodes: baker.nodes,
            root,
        })
    }
}

/// The per-diagram inputs [`DiagramEvalTree::bake`] threads through its recursion, and
/// the node list it fills.
struct Baker<'a> {
    raw: &'a RawDiagramTree,
    diagram: &'a Diagram,
    model: &'a UFOModel,
    n_in: usize,
    uncross: &'a HashSet<LegIdx>,
    chain: &'a [u8],
    nodes: Vec<EvalNode>,
}

impl Baker<'_> {
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
        &mut self,
        id: RawNodeId,
    ) -> Result<(EvalNodeId, Option<LegAdjoint>), RootLorentzError> {
        let (raw, diagram, model, n_in) = (self.raw, self.diagram, self.model, self.n_in);
        match raw.value(id) {
            RawNode::Leg {
                particle,
                leg_idx,
                charge,
                spin,
                incoming,
            } => {
                let (info, bind) = external_leg(
                    model,
                    *particle,
                    *leg_idx,
                    *charge,
                    *spin,
                    *incoming,
                    self.uncross.contains(leg_idx),
                );
                Ok((
                    DiagramEvalTree::add(&mut self.nodes, EvalNode::External(info)),
                    bind,
                ))
            }
            RawNode::Vertex {
                vertex,
                vtx,
                result,
                children,
            } => {
                let baked: Vec<(EvalNodeId, Option<LegAdjoint>)> = children
                    .iter()
                    .map(|&c| self.bake_node(c))
                    .collect::<Result<Vec<_>, _>>()?;
                let child_ids: Vec<EvalNodeId> = baked.iter().map(|(id, _)| *id).collect();
                let color_idx = self.chain[vtx.0] as usize;
                let flow_group = diagram.vertex(*vtx).flow_group;
                let group = vertex_flow_group(model, *vertex, flow_group);
                match result {
                    Some(rl) => {
                        // Internal vertex: off-shell current rooted at the output leg,
                        // wrapped by the propagator on that leg.
                        let ri = rl.slot;
                        let prop_id = model.vertex_def(*vertex).particles[ri.0];
                        // The current keeps the binding of the fermion input the
                        // vertex's pairing continues into the output leg iff that leg
                        // is itself a fermion; a bosonic output carries none. Reading
                        // the pairing rather than taking the first fermion child is
                        // what keeps a four-fermion current (three fermion inputs, two
                        // of them closing a line here) on the right line. The bindings
                        // are passed into the Lorentz rooting so it picks the in/out
                        // gamma routine and detects reversed/crossed pairs.
                        let bind = (model.particle(prop_id).spin == 2)
                            .then(|| {
                                let cont = input_position(group.partner(ri.0), ri.0);
                                baked[cont].1
                            })
                            .flatten();
                        // Per-leg bindings in vertex-leg order: children with the
                        // output's binding spliced in at its position, so the Lorentz
                        // rooting can compare each leg to its UFO slot.
                        let mut flows: Vec<Option<LegAdjoint>> =
                            baked.iter().map(|(_, f)| *f).collect();
                        flows.insert(ri.0, bind);
                        let info = VertexInfo::from_ufo(
                            model,
                            *vertex,
                            color_idx,
                            flow_group,
                            Some(ri.0),
                            &flows,
                        )?;
                        let adjoint = bind.map(|lf| lf.adjoint);
                        let current = DiagramEvalTree::add(
                            &mut self.nodes,
                            EvalNode::OffShellCurrent {
                                info,
                                adjoint,
                                children: child_ids,
                                fermion_pairs: closed_fermion_pairs(&group, Some(ri.0)),
                            },
                        );
                        Ok((
                            DiagramEvalTree::add(
                                &mut self.nodes,
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
                        let info = VertexInfo::from_ufo(
                            model, *vertex, color_idx, flow_group, None, &flows,
                        )?;
                        Ok((
                            DiagramEvalTree::add(
                                &mut self.nodes,
                                EvalNode::ContractAmplitude {
                                    info,
                                    children: child_ids,
                                    fermion_pairs: closed_fermion_pairs(&group, None),
                                },
                            ),
                            None,
                        ))
                    }
                }
            }
        }
    }
}

impl DiagramEvalTree {
    fn render_expression(&self) -> String {
        self.fold_recursive(
            &|node, acc| node.render(acc),
            &|acc, r| if acc.is_empty() { r } else { acc + ", " + &r },
            String::new(),
            self.root,
        )
    }

    /// The diagram's runtime `reversed`-bilinear parity at this rooting: the product over
    /// its vertices of each vertex's
    /// [`reversed_sign`](super::diagram_eval::VertexInfo::reversed_sign), the parity the
    /// runtime `resolve_bra_ket` applies at the fermion→vector sinks of this tree. It
    /// depends on the rooting (a fermion→vector sink under one rooting is a
    /// fermion-continuing current under another), so [`compile_single_diagram`] folds it
    /// into `fermi_sign` to cancel the runtime one, and the diagram carries the parity of
    /// [`convention_sign`] instead, which no rooting changes.
    fn live_reversed_parity(&self) -> Result<i8, RootLorentzError> {
        let mut sign = 1i8;
        for id in self.iter() {
            let info = match self.value(id) {
                EvalNode::OffShellCurrent { info, .. }
                | EvalNode::ContractAmplitude { info, .. } => info,
                _ => continue,
            };
            sign *= info.reversed_sign()?;
        }
        Ok(sign)
    }
}

impl std::fmt::Display for DiagramEvalTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.render_expression())
    }
}

// ───────────────────── Fermion-line sign from the baked adjoint ─────────────────────
//
// The fermion-line sign `fermi_sign` carries is a property of the diagram
// (`Diagram::fermion_line_sign`, folded into `Diagram::sign`). The derivation below reads
// the same sign off a rooted evaluation tree instead — from the spinor adjoint the
// rooting baked into it — and is the independent cross-check debug builds hold every
// compiled diagram to.

/// One half of a fermion line, as seen descending from the vertex it closes at.
#[derive(Clone, Copy)]
struct FermionHalf {
    /// Whether the external leg the descent terminates at is incoming.
    incoming: bool,
    /// Internal fermion propagators crossed.
    props: usize,
}

/// Descend the fermion line from `node` (a fermion child of a pair-sink) to its
/// terminal external leg. Follows the continuing fermion (the lone `Some`-adjoint
/// child) through each off-shell current; a `Propagate` is exactly one internal fermion
/// propagator.
fn descend_fermion_line(tree: &DiagramEvalTree, node: EvalNodeId) -> FermionHalf {
    match tree.value(node) {
        EvalNode::External(info) => FermionHalf {
            incoming: info.incoming,
            props: 0,
        },
        EvalNode::Propagate { child, .. } => {
            let half = descend_fermion_line(tree, *child);
            FermionHalf {
                props: half.props + 1,
                ..half
            }
        }
        EvalNode::OffShellCurrent {
            children,
            fermion_pairs,
            ..
        } => {
            // The continuing input is the fermion child no closed pair names: at a
            // four-fermion current two of the three fermion inputs close a line here.
            let cont = children
                .iter()
                .enumerate()
                .find(|&(pos, &c)| {
                    tree.value(c).out_adjoint().is_some()
                        && !fermion_pairs.iter().any(|&(a, b)| a == pos || b == pos)
                })
                .map(|(_, &c)| c)
                .expect("a fermion off-shell current has a continuing fermion input");
            descend_fermion_line(tree, cont)
        }
        EvalNode::ContractAmplitude { .. } => {
            unreachable!("the amplitude root is never reached while descending a fermion line")
        }
    }
}

/// The fermion-line sign of [`Diagram::fermion_line_sign`], derived from a rooted tree's
/// baked spinor adjoint rather than from the diagram's lines.
///
/// A fermion line closes at the one vertex node where both of its legs are inputs: the
/// root contraction, a two-fermion current rooted at its boson leg, or a four-fermion
/// current rooted at a leg of its *other* line — which outputs a fermion and still
/// closes this one. Every node's closed pairs are read, so every line is counted once;
/// descending both halves from where it closes recovers the line's ends, its internal
/// propagators, which is all the sign reads.
pub(super) fn spine_sign_from_flow(tree: &DiagramEvalTree) -> i8 {
    let mut sign = 1i8;
    for id in tree.iter() {
        let (children, fermion_pairs) = match tree.value(id) {
            EvalNode::ContractAmplitude {
                children,
                fermion_pairs,
                ..
            }
            | EvalNode::OffShellCurrent {
                children,
                fermion_pairs,
                ..
            } => (children, fermion_pairs),
            _ => continue,
        };
        // Every line the vertex closes, one at a two-fermion sink and up to two at a
        // four-fermion one; a line's two ends are named by the vertex's own pairing,
        // not by their order in the child list.
        for &(a, b) in fermion_pairs {
            let half_a = descend_fermion_line(tree, children[a]);
            let half_b = descend_fermion_line(tree, children[b]);
            let crossed = !half_a.incoming && !half_b.incoming;
            // A crossed line takes its single −1 whatever it is built from; an
            // uncrossed one takes a −1 per internal propagator, whatever its vertices.
            let flip = crossed || (half_a.props + half_b.props) % 2 == 1;
            if flip {
                sign = -sign;
            }
        }
    }
    sign
}

// ─────────────────────── Mixed-line (initial↔final) uncrossing ───────────────────────

/// The child position a vertex slot occupies in a vertex rooted at `out_slot`: children
/// are the vertex's legs in slot order with the output leg's position removed.
fn input_position(slot: usize, out_slot: usize) -> usize {
    if slot < out_slot {
        slot
    } else {
        slot - 1
    }
}

/// The fermion lines a vertex closes, as pairs of positions in its input-child list:
/// every pair of the vertex's fermion flow whose two legs are both inputs. Rooted at
/// a fermion leg, the line through `out_slot` is open and is left out.
fn closed_fermion_pairs(group: &FlowGroup, out_slot: Option<usize>) -> Vec<(usize, usize)> {
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    for (leg, &partner) in group.spin_map.iter().enumerate() {
        let partner = partner as usize;
        if partner <= leg || Some(leg) == out_slot || Some(partner) == out_slot {
            continue;
        }
        match out_slot {
            Some(out) => pairs.push((input_position(leg, out), input_position(partner, out))),
            None => pairs.push((leg, partner)),
        }
    }
    pairs
}

/// Final-state legs whose fermion line connects to an *initial-state* leg (e.g. the
/// Bhabha t-channel electron line). Such legs must be typed by their physical
/// particle/adjoint — matching the reference HELAS externals — rather than by the
/// crossed (all-incoming) identity feyngraph reports: the crossed representation
/// C-conjugates the whole bilinear chain, which is only an identity when *both*
/// endpoints of the line conjugate together, i.e. for final–final pairs.
///
/// A property of the diagram's lines ([`Diagram::fermion_lines`]), so the live tree and
/// the convention sign ([`external_bindings`]) read the same set at every rooting.
fn mixed_line_final_legs(diagram: &Diagram, model: &UFOModel) -> HashSet<LegIdx> {
    let n_in = diagram.n_in;
    diagram
        .fermion_lines(model)
        .into_iter()
        .filter_map(|line| {
            let [a, b] = line.legs;
            match (a.0 < n_in, b.0 < n_in) {
                (true, false) => Some(b),
                (false, true) => Some(a),
                _ => None,
            }
        })
        .collect()
}

/// The external wavefunction a leg is built as, and the spinor binding it carries
/// (`None` for a boson). A final-state leg of [`mixed_line_final_legs`] (`uncrossed`)
/// is typed by its physical particle; every other leg keeps the identity diagram
/// enumeration gives it, which puts a final-state fermion on a crossed line.
fn external_leg(
    model: &UFOModel,
    particle: ParticleId,
    leg_idx: LegIdx,
    charge: Charge,
    spin: i32,
    incoming: bool,
    uncrossed: bool,
) -> (ExtLegInfo, Option<LegAdjoint>) {
    let (id, charge) = if uncrossed {
        let anti = model
            .particle_id(&model.particle(particle).antiname)
            .expect("antiparticle exists in model");
        (anti, charge.anti())
    } else {
        (particle, charge)
    };
    let info = ExtLegInfo {
        id,
        leg_idx: leg_idx.0,
        charge,
        spin,
        incoming,
    };
    let bind = info.adjoint().map(|adjoint| LegAdjoint {
        adjoint,
        crossed: !incoming && !uncrossed,
    });
    (info, bind)
}

// ───────────────────────────── Rooting entry point ─────────────────────────────

/// Number of external legs directly attached to a vertex.
fn ext_leg_count(vertex: &crate::diagrams::diagram::Vertex) -> usize {
    vertex
        .rays
        .iter()
        .filter(|r| matches!(r, Ray::Leg(_)))
        .count()
}

/// The canonical production root: the vertex with the fewest directly-attached external
/// legs, ties broken toward the lowest vertex index.
///
/// Rooting a diagram at a low-degree-in-externals vertex leaves the tree's off-shell
/// currents shared across more diagrams: a deep, few-external root keeps the sub-currents
/// closer to the canonical `(edge, direction)` signatures cross-diagram CSE deduplicates,
/// whereas rooting at a high-external hub duplicates them. The amplitude is
/// rooting-invariant (`rooting_soundness`), so this only reshapes the shared arena, not the
/// physics.
fn canonical_root(diagram: &Diagram) -> VtxIdx {
    let mut best_vi = 0usize;
    let mut best_c = ext_leg_count(&diagram.vertices[0]);
    for (vi, v) in diagram.vertices.iter().enumerate().skip(1) {
        let c = ext_leg_count(v);
        if c < best_c {
            best_c = c;
            best_vi = vi;
        }
    }
    VtxIdx(best_vi)
}

/// The root vertex [`root_tree`] walks from.
///
/// The override hook below exists only for the rooting-soundness harness, so it
/// is compiled only into test builds.
#[cfg(not(test))]
fn choose_root(diagram: &Diagram) -> VtxIdx {
    canonical_root(diagram)
}

/// A replacement for [`canonical_root`], installed per thread.
#[cfg(test)]
pub(crate) type RootChooser = Box<dyn Fn(&Diagram) -> VtxIdx>;

#[cfg(test)]
thread_local! {
    static ROOT_OVERRIDE: std::cell::RefCell<Option<RootChooser>> =
        const { std::cell::RefCell::new(None) };
}

/// Install a per-diagram root-vertex chooser consulted by [`root_tree`] on the current
/// thread. Lets a soundness harness re-root diagrams without touching the production
/// walk. With no override installed, rooting falls back to [`canonical_root`].
#[cfg(test)]
pub(crate) fn set_root_override(f: RootChooser) {
    ROOT_OVERRIDE.with(|c| *c.borrow_mut() = Some(f));
}

/// Remove any installed root chooser, restoring the [`canonical_root`] default.
#[cfg(test)]
pub(crate) fn clear_root_override() {
    ROOT_OVERRIDE.with(|c| *c.borrow_mut() = None);
}

#[cfg(test)]
fn choose_root(diagram: &Diagram) -> VtxIdx {
    ROOT_OVERRIDE
        .with(|c| c.borrow().as_ref().map(|f| f(diagram)))
        .unwrap_or_else(|| canonical_root(diagram))
}

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
/// * `chain` — the color-index chain: the chosen color structure per vertex, in
///   [`VtxIdx`] order (as recorded by `colorize`). Selects which color structure of
///   each vertex the rooted Lorentz/coupling terms are built from.
pub(super) fn root_tree(
    diagram: &Diagram,
    model: &UFOModel,
    chain: &[u8],
) -> Result<DiagramEvalTree, CompileError> {
    // Walk the tree from the chosen root vertex ([`canonical_root`]); a test-only override
    // may select another vertex to probe rooting soundness.
    root_tree_at(diagram, model, chain, choose_root(diagram))
}

/// Root a diagram at an explicit vertex (bypassing [`choose_root`]).
pub(super) fn root_tree_at(
    diagram: &Diagram,
    model: &UFOModel,
    chain: &[u8],
    root: VtxIdx,
) -> Result<DiagramEvalTree, CompileError> {
    let mut builder = RawBuilder::new(diagram);
    let raw_root = builder.walk_vertex(root, None)?;
    let raw = RawDiagramTree {
        nodes: builder.nodes,
        root: raw_root,
    };

    let uncross = mixed_line_final_legs(diagram, model);
    Ok(DiagramEvalTree::bake(
        &raw,
        diagram,
        model,
        diagram.n_in,
        &uncross,
        chain,
    )?)
}

// ───────────────────────────── Per-diagram artifact ─────────────────────────────

/// A compiled representation of a single Feynman diagram.
///
/// Built once from an owned [`Diagram`] + `UFOModel`. The diagram is a rooted
/// [`DiagramEvalTree`]: external legs are leaves, internal vertices are off-shell
/// currents wrapped by propagators, and the root contracts into the scalar amplitude.
#[derive(Clone, Debug)]
pub(crate) struct DiagramEval {
    /// Number of external legs (determines array indexing for momenta)
    pub(crate) n_ext: usize,
    /// Rooted evaluation tree for this diagram
    pub(crate) tree: DiagramEvalTree,
    /// Symmetry factor: 1 / (vertex_sym × propagator_sym)
    pub(crate) symmetry_factor: f64,
    /// ±1: the diagram's relative Fermi sign ([`Diagram::sign`]) times its
    /// [`convention_sign`], with the live tree's reversed-bilinear parity cancelled
    /// (see [`compile_single_diagram`])
    pub(crate) fermi_sign: i8,
}

impl DiagramEval {
    /// Assemble a single diagram from a hand-specified node list (tests only).
    ///
    /// The last node is the root; children reference earlier nodes by index (see
    /// [`EvalNodeId::new`]). Symmetry factor and Fermi sign are trivial (1, +1), so the
    /// reconstructed amplitude is exactly the rooted contraction of the given nodes —
    /// used to drive single-vertex primitives through the generic `run_forward_slot` pass.
    #[cfg(test)]
    pub(crate) fn from_nodes(n_ext: usize, nodes: Vec<EvalNode>) -> Self {
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
    #[cfg(all(test, feature = "extended-validation"))]
    pub(crate) fn propagator_particles(&self) -> impl Iterator<Item = ParticleId> + '_ {
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

/// True iff `interaction` is a Yang-Mills triple-vector (VVV) vertex: a **three-leg**
/// all-vector vertex whose Lorentz structure carries a momentum (`P`) factor.
///
/// The leg count is part of the predicate, not a consequence of the momentum test: a
/// contact of four or more vectors *can* carry momenta (SMEFTsim's `VVVV2`/`VVVV3` and
/// the five-vector structures of `O_W` all do), and such a vertex is a contact, not a
/// triple-vector vertex.
fn is_yang_mills_vvv(model: &UFOModel, interaction: VertexId) -> bool {
    use crate::ufo::lorentz::LorentzOp;
    let def = model.vertex_def(interaction);
    def.particles.len() == 3
        && def.particles.iter().all(|&p| model.particle(p).spin == 3)
        && def.lorentz.iter().any(|&lid| {
            model
                .lorentz_struct(lid)
                .expr
                .iter()
                .any(|t| t.ops.iter().any(|op| matches!(op, LorentzOp::P { .. })))
        })
}

/// Whether any leg of `interaction` carries colour (an antiparticle stores its
/// representation negated, so a singlet is `±1`).
fn is_coloured(model: &UFOModel, interaction: VertexId) -> bool {
    model
        .vertex_def(interaction)
        .particles
        .iter()
        .any(|&p| model.particle(p).color.abs() != 1)
}

/// The spinor binding each external leg's wavefunction carries (`None` for a boson):
/// its adjoint, and whether its fermion line is crossed — the binding the live tree
/// gives the leg ([`external_leg`]).
fn external_bindings(diagram: &Diagram, model: &UFOModel) -> Vec<Option<LegAdjoint>> {
    let uncross = mixed_line_final_legs(diagram, model);
    diagram
        .legs
        .iter()
        .map(|leg| {
            external_leg(
                model,
                leg.particle,
                leg.leg_idx,
                leg.charge,
                leg.spin,
                leg.incoming,
                uncross.contains(&leg.leg_idx),
            )
            .1
        })
        .collect()
}

/// The external leg a fermion line reaches leaving vertex `v` through ray slot `slot`.
fn line_end(diagram: &Diagram, model: &UFOModel, v: VtxIdx, slot: usize) -> LegIdx {
    let (mut v, mut slot) = (v, slot);
    loop {
        match diagram.vertex(v).rays[slot] {
            Ray::Leg(l) => return l,
            Ray::Prop { prop, end } => {
                let (w, RaySlot(s)) = diagram.prop(prop).endpoints[1 - end];
                let w_vertex = diagram.vertex(w);
                let group = vertex_flow_group(model, w_vertex.interaction, w_vertex.flow_group);
                (v, slot) = (w, group.partner(s));
            }
        }
    }
}

/// The diagram's convention sign: the product of a constant per internal line and a
/// sign per vertex, each read off the graph alone, so it is the same at every rooting
/// and under every labelling of the legs.
///
/// The currents the kernels build carry no convention sign, and with one they are
/// rooting-invariant tensors. MadGraph's `AMP()` is rooting-invariant too: ALOHA
/// evaluates every vertex as `COUP · L` and closes the amplitude with `−i`, whichever
/// vertex that is, and gives each propagator a fixed phase. So the two differ by a
/// product of local constants, one per line and one per vertex, and nothing in it
/// refers to a root:
///
/// * [`propagator_phase_sign`]: the phase of each propagator against ALOHA's;
/// * [`vertex_root_signs`]: each vertex's reading of its fermion bilinears;
/// * [`triple_gluon_sign`]: the slot exchange the triple-gluon vertex, alone of the
///   triple-vector vertices, does not get from the crossing.
///
/// With [`Diagram::sign`] (the external fermions' pairing parity and the crossed-line
/// reordering) that is the whole of `fermi_sign`.
fn convention_sign(diagram: &Diagram, model: &UFOModel, chain: &[u8]) -> Result<i8, CompileError> {
    Ok(propagator_phase_sign(diagram, model)
        * vertex_root_signs(diagram, model, chain)?
        * triple_gluon_sign(diagram, model))
}

/// `−1` per internal scalar line and per internal fermion line of a crossed fermion
/// line (both ends final-state).
///
/// ALOHA's off-shell currents carry `+i` on a fermion or scalar propagator
/// (`i(q̸ + m)/D`, `i/D`) and `−i` on a vector one (`−i(g − qq/m²)/D`); every
/// propagator here carries `−i` ([`propagate_core`](super::kernel::propagate_core)). So
/// every internal fermion and scalar line takes `−1` against MadGraph and a vector line
/// none. The fermion lines of a line with an initial-state end take theirs as the
/// `(−1)^P` of [`Diagram::fermion_line_sign`], which leaves a crossed line's to this
/// factor.
fn propagator_phase_sign(diagram: &Diagram, model: &UFOModel) -> i8 {
    let scalar_lines = diagram
        .props
        .iter()
        .filter(|p| model.particle(p.particle).spin == 1)
        .count();
    let crossed_fermion_lines: usize = diagram
        .fermion_lines(model)
        .iter()
        .filter(|line| line.legs.iter().all(|l| l.0 >= diagram.n_in))
        .map(|line| line.vertices.len() - 1)
        .sum();
    if (scalar_lines + crossed_fermion_lines).is_multiple_of(2) {
        1
    } else {
        -1
    }
}

/// The product over vertices of each vertex's sign read with the vertex as the
/// amplitude: every leg an input, each fermion slot bound to the spinor of the
/// external leg its line reaches through that slot.
///
/// A vertex reads each fermion bilinear with the spinors its lines bind to its slots.
/// Where a bilinear's row slot is bound to a ket, the bilinear is read against its UFO
/// order, `ψ̄₁Γψ₂ = ψ̄₂(CΓᵀC⁻¹)ψ₁`, and `CΓᵀC⁻¹ = −Γ` for `γ^μ` and `σ^{μν}` while
/// `1`, `γ⁵` and `P_χ` keep their sign: that is the vertex's reversed-bilinear parity
/// ([`VertexInfo::reversed_sign`]) with every leg an input, a function of which
/// external leg each fermion slot leads to and so of the graph. A four-fermion
/// structure evaluated through the tensor path
/// ([`VertexInfo::tensor`](super::diagram_eval::VertexInfo::tensor)) cuts its index
/// cycle at one line and reads the reversals inside the kernel; it carries the sign
/// that path gives its amplitude root instead
/// ([`VertexInfo::build_sign`](super::diagram_eval::VertexInfo::build_sign)).
fn vertex_root_signs(
    diagram: &Diagram,
    model: &UFOModel,
    chain: &[u8],
) -> Result<i8, CompileError> {
    let bindings = external_bindings(diagram, model);
    let mut sign = 1i8;
    for (v, vertex) in diagram.vertices.iter().enumerate() {
        let def = model.vertex_def(vertex.interaction);
        let flows: Vec<Option<LegAdjoint>> = (0..vertex.rays.len())
            .map(|slot| {
                (model.particle(def.particles[slot]).spin == 2)
                    .then(|| bindings[line_end(diagram, model, VtxIdx(v), slot).0])
                    .flatten()
            })
            .collect();
        let info = VertexInfo::from_ufo(
            model,
            vertex.interaction,
            chain[v] as usize,
            vertex.flow_group,
            None,
            &flows,
        )?;
        sign *= info.reversed_sign()?;
        if info.tensor() {
            sign *= info.build_sign()?;
        }
    }
    Ok(sign)
}

/// `−1` per triple-gluon vertex: the slot exchange every other triple-vector vertex
/// carries in its kernel.
///
/// feyngraph reads every interaction slot incoming and MadGraph reads it outgoing, so
/// each leg stands in its antiparticle's slot of the UFO vertex, and the two slots of
/// every conjugate pair hold each other's legs against MadGraph's reading. At a fermion
/// vertex that exchange is undone, in the colour by `convert_expr`'s transpose of each
/// `T` and in the bilinear by [`vertex_root_signs`]. At a `γW⁺W⁻` or `ZW⁺W⁻` vertex
/// nothing undoes it: the `W⁺` and `W⁻` slots are exchanged, and `VVV1`, like every
/// triple-vector structure the SM and SMEFTsim put on a W pair, is odd under that
/// exchange, so the kernel reads `−1` against MadGraph's `COUP · L`. The other factors
/// of [`convention_sign`] are set against kernels that carry it: `w+ w- > u u~` and
/// `g g > u u~` carry the same [`Diagram::sign`], propagator phases and bilinear
/// parities diagram for diagram, as do `e+ e- > w+ w-` and `u u~ > g g`, and the
/// colourless one of each pair agrees with MadGraph with no vertex sign.
///
/// A gluon is its own antiparticle, so the crossing exchanges none of the triple-gluon
/// vertex's slots. Its rays stand in MadGraph's slot order up to a permutation its
/// colour `f(1,2,3)` follows, since both are read positionally; its coupling `GC_10`
/// is MadGraph's; and its colour coefficients are MadGraph's JAMP coefficients term by
/// term, because `f` passes through `convert_expr` untouched and `f_to_traces` is
/// MadGraph's expansion. So the `−1` is not in the colour factor or the coupling: it
/// is the slot exchange the kernel does not get. The vertices counted are the coloured
/// ones; by the same account a colourless triple-vector vertex of three self-conjugate
/// legs would take it too, and no model in reach has one.
///
/// Only its ratio to the quark–gluon vertex is observable: a process's external gluons
/// fix the parity of the two counts together, so `−1` per quark–gluon vertex is the
/// same convention up to a sign common to the process. Without it the diagrams through
/// a triple-gluon vertex come out with the wrong sign against those with a quark line
/// in its place: every MadGraph row that has both (`u u~ > g g g`, `u g > u g`,
/// `g g > u u~`, `g g > t t~` and the `t t~ + jet` rows) disagrees. The factors are
/// pinned one by one against MadGraph's `AMP()` in `helas::eval::triple_gluon`.
pub(super) fn triple_gluon_sign(diagram: &Diagram, model: &UFOModel) -> i8 {
    let n = diagram
        .vertices
        .iter()
        .filter(|v| is_yang_mills_vvv(model, v.interaction) && is_coloured(model, v.interaction))
        .count();
    if n.is_multiple_of(2) {
        1
    } else {
        -1
    }
}

/// Compile a single diagram into an evaluable [`DiagramEval`].
///
/// Roots the diagram into its evaluation tree (topology + Lorentz structures) and
/// attaches the per-diagram metadata: external-leg count, symmetry factor, and
/// `fermi_sign` — the diagram's own relative Fermi sign ([`Diagram::sign`]) times its
/// [`convention_sign`], both functions of the graph alone, so the product is the same
/// whichever vertex the tree is rooted at and however the legs are labelled.
pub(super) fn compile_single_diagram(
    diagram: &Diagram,
    model: &UFOModel,
    chain: &[u8],
) -> Result<DiagramEval, CompileError> {
    let tree = root_tree(diagram, model, chain)?;
    debug_assert_eq!(
        spine_sign_from_flow(&tree),
        diagram.fermion_line_sign(model),
        "the rooted tree and the diagram's lines disagree on the fermion-line sign"
    );
    // The runtime `resolve_bra_ket` applies the live tree's reversed-bilinear parity
    // (`tree.live_reversed_parity()`); multiplying by it cancels that, and the
    // diagram carries the rooting-free parity of `convention_sign` instead.
    let fermi_sign =
        diagram.sign * convention_sign(diagram, model, chain)? * tree.live_reversed_parity()?;
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
#[cfg(test)]
pub(crate) fn compile_diagram_ast(
    set: &DiagramSet,
    model: &UFOModel,
) -> Result<Vec<DiagramEval>, CompileError> {
    set.diagrams
        .iter()
        .map(|diagram| {
            let chain = vec![0u8; diagram.vertices.len()];
            compile_single_diagram(diagram, model, &chain)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, DiagramSet, ParsingOptions};
    use crate::ufo::sm::{sm_model, SMRestrict};

    fn generate(process: &str) -> Vec<DiagramSet> {
        let opts = ParsingOptions::default();
        let card = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
        generate_from_proc_card(&card, &sm_model(SMRestrict::Default)).unwrap()
    }

    #[test]
    fn test_walk() {
        let model = sm_model(SMRestrict::Default);
        let sets = generate("e+ e- > mu+ mu-");
        for set in sets {
            for (d, diagram) in set.diagrams.iter().enumerate() {
                println!("Testing diagram {d}");
                let chain = vec![0u8; diagram.vertices.len()];
                let tree = root_tree(diagram, &model, &chain).expect("rooting failed");
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
    #[ignore = "diagnostic dump; run with --nocapture"]
    fn probe_vertex_leg_binding() {
        let model = sm_model(SMRestrict::Default);
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
                let chain = vec![0u8; diagram.vertices.len()];
                let tree = root_tree(diagram, &model, &chain).expect("rooting failed");
                println!("  baked: {tree}");
            }
        }
    }

    /// Each factor of the convention sign fires on a named process, so a refactor or an
    /// enumeration change that stops reaching one fails here instead of leaving it
    /// unexercised beside its siblings. The factors' values are pinned against
    /// MadGraph by the amplitude oracles; this is the non-vacuity half.
    #[test]
    fn named_processes_exercise_every_convention_factor() {
        let sm = sm_model(SMRestrict::Default);
        let firing = |model: &UFOModel, process: &str, factor: &dyn Fn(&Diagram) -> i8| {
            let opts = ParsingOptions::default();
            let card = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
            generate_from_proc_card(&card, model)
                .unwrap()
                .iter()
                .flat_map(|set| set.diagrams.iter())
                .filter(|d| factor(d) < 0)
                .count()
        };
        let chain0 = |d: &Diagram| vec![0u8; d.vertices.len()];
        // An internal Higgs line: the s-channel Higgs of the triple-Higgs annihilation.
        assert!(firing(&sm, "b b~ > h h", &|d| propagator_phase_sign(d, &sm)) > 0);
        // A fermion propagator on a crossed line: the Higgs emitted off the final-state
        // tau pair.
        assert!(firing(&sm, "e+ e- > ta+ ta- h", &|d| propagator_phase_sign(d, &sm)) > 0);
        // A bilinear read against its UFO order: the initial electron line.
        assert!(
            firing(&sm, "e+ e- > mu+ mu-", &|d| vertex_root_signs(
                d,
                &sm,
                &chain0(d)
            )
            .unwrap())
                > 0
        );
        // The triple-gluon vertex: the s-channel gluon of `g g > t t~`.
        assert!(firing(&sm, "g g > t t~", &|d| triple_gluon_sign(d, &sm)) > 0);
        // A tensor four-fermion contact as the amplitude, on a crossed line.
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let toy = crate::ufo::UFOModel::load(
            &root.join("../validation/ufo/vibegraph_toy_UFO"),
            Some(&root.join("../validation/ufo/vibegraph_toy_UFO/restrict_tensor.dat")),
        )
        .expect("load the toy model under its tensor card");
        assert!(
            firing(&toy, "lt~ lt > qt qt~ NP<=1 NPGG<=1", &|d| {
                let tensor_contact = d.vertices.iter().any(|v| {
                    let def = toy.vertex_def(v.interaction);
                    def.particles
                        .iter()
                        .filter(|&&p| toy.particle(p).spin == 2)
                        .count()
                        == 4
                });
                if tensor_contact {
                    vertex_root_signs(d, &toy, &chain0(d)).unwrap()
                } else {
                    1
                }
            }) > 0
        );
    }

    /// The diagram's fermion-line sign, read off its lines, must agree with the same sign
    /// derived from the baked spinor adjoint of a rooted tree — at every rooting, so the
    /// sign is shown to be a property of the diagram and not of where it is rooted —
    /// across processes with and without off-shell fermion lines.
    #[test]
    fn fermion_line_sign_matches_the_rooted_tree_derivation() {
        let model = sm_model(SMRestrict::Default);
        let processes = [
            "e+ e- > mu+ mu-",
            "e+ e- > e+ e-",
            "u u~ > d d~",
            "e+ e- > mu+ mu- ta+ ta-",
            "u d > e+ e- u d QCD=0",
            "g g > t t~",
            // An all-Yukawa quark line, and the same topology with a gauge vertex on it.
            "b b~ > h h",
            "b b~ > a h",
        ];
        let mut flipped_total = 0;
        for process in processes {
            for set in generate(process) {
                for (i, diagram) in set.diagrams.iter().enumerate() {
                    let chain = vec![0u8; diagram.vertices.len()];
                    let from_lines = diagram.fermion_line_sign(&model);
                    for root in 0..diagram.vertices.len() {
                        let tree = root_tree_at(diagram, &model, &chain, VtxIdx(root))
                            .expect("rooting failed");
                        let from_tree = spine_sign_from_flow(&tree);
                        assert_eq!(
                            from_tree, from_lines,
                            "fermion-line sign mismatch in `{process}` diagram {i} rooted \
                             at vertex {root}: tree={from_tree} lines={from_lines}"
                        );
                    }
                    if from_lines < 0 {
                        flipped_total += 1;
                    }
                }
            }
        }
        // The e+e-→μ+μ-τ+τ- e-line class (8 diagrams) must actually exercise a flip, so
        // the agreement above is not vacuous.
        assert!(
            flipped_total >= 8,
            "expected at least the 8 e-line flips, saw {flipped_total}"
        );
    }

    /// A fermion line takes the per-propagator sign whatever its vertices are: an
    /// all-Yukawa line exactly as one carrying a photon.
    ///
    /// `b b~ > h h` is the Standard Model's own all-Yukawa line: the two exchange
    /// diagrams run the `b` line through two `b b~ H` vertices and one internal
    /// propagator, the triple-Higgs annihilation diagram through one vertex and none.
    /// The exchange diagrams take the −1; the annihilation diagram takes its −1 from its
    /// internal Higgs line instead ([`propagator_phase_sign`]), so the two still
    /// interfere at the sign MadGraph gives them — which the `bbx_to_hh` standalone
    /// table checks end to end. `b b~ > a h` is the gauge control: the same
    /// one-propagator topology with a photon vertex on the line.
    #[test]
    fn every_line_takes_the_propagator_sign() {
        let model = sm_model(SMRestrict::Default);
        let spine = |process: &str| -> Vec<(usize, i8)> {
            generate(process)
                .iter()
                .flat_map(|set| set.diagrams.clone())
                .map(|diagram| {
                    let fermion_props = diagram
                        .props
                        .iter()
                        .filter(|p| model.particle(p.particle).spin.abs() == 2)
                        .count();
                    (fermion_props, diagram.fermion_line_sign(&model))
                })
                .collect()
        };
        for process in ["b b~ > h h", "b b~ > a h"] {
            let lines = spine(process);
            assert!(
                lines.iter().any(|&(props, _)| props == 1),
                "{process} must contribute a one-propagator line, got {lines:?}"
            );
            assert!(
                lines
                    .iter()
                    .all(|&(props, sign)| sign == if props % 2 == 1 { -1 } else { 1 }),
                "{process}: every line takes the per-propagator flip, got {lines:?}"
            );
        }
    }

    /// The per-propagator flip fires on a *mixed* (initial↔final) fermion line, not only
    /// on an initial–initial one, and a *crossed* (final–final) line takes its single −1
    /// regardless of how many propagators it carries.
    ///
    /// `u d > e+ e- u d QCD=0` is the discriminating topology: every one of its 35
    /// diagrams has exactly one internal fermion propagator except the two with a triple-
    /// gauge vertex, and the propagator sits either on a mixed quark line (24 diagrams)
    /// or on the crossed lepton line (9). The two classes must therefore come out with
    /// *opposite* spine signs. `g g > t t~` supplies the negative control: its s-channel top
    /// line carries no propagator and its t/u-channel lines carry one, and all three must
    /// come out the same.
    ///
    /// What this cannot see: a sign common to all diagrams of a process (absorbed by the
    /// per-configuration phase fit), and anything outside the spine channel — the gate on
    /// `ud_to_epemud_qcd0` in `tests/amplitude_oracle.rs` is what pins the absolute
    /// per-diagram values.
    #[test]
    fn spine_sign_separates_mixed_line_and_crossed_line_propagators() {
        let model = sm_model(SMRestrict::Default);

        let mut mixed_prop = Vec::new();
        let mut crossed_prop = Vec::new();
        let mut no_prop = Vec::new();
        for set in generate("u d > e+ e- u d QCD=0") {
            let n_in = set.diagrams[0].n_in;
            for diagram in &set.diagrams {
                let sign = diagram.fermion_line_sign(&model);
                // Classify by where the diagram's internal fermion propagator sits: one
                // propagator per vertex a line passes through beyond the first.
                let (mut on_mixed, mut on_crossed) = (0usize, 0usize);
                for line in diagram.fermion_lines(&model) {
                    let n_props = line.vertices.len() - 1;
                    if line.legs.iter().all(|l| l.0 >= n_in) {
                        on_crossed += n_props;
                    } else {
                        on_mixed += n_props;
                    }
                }
                match (on_mixed, on_crossed) {
                    (0, 0) => no_prop.push(sign),
                    (1, 0) => mixed_prop.push(sign),
                    (0, 1) => crossed_prop.push(sign),
                    other => panic!("unexpected propagator placement {other:?}"),
                }
            }
        }
        assert_eq!(
            (mixed_prop.len(), crossed_prop.len(), no_prop.len()),
            (24, 9, 2),
            "u d > e+ e- u d QCD=0 must split 24 mixed-line / 9 crossed-line / 2 \
             propagator-free diagrams"
        );
        // Both classes carry one crossed lepton line, so the crossed −1 is common and the
        // classes must differ by exactly the mixed-line propagator flip.
        assert!(
            mixed_prop.iter().all(|&s| s == mixed_prop[0]),
            "mixed-line-propagator diagrams must share a spine sign, got {mixed_prop:?}"
        );
        assert!(
            crossed_prop
                .iter()
                .chain(&no_prop)
                .all(|&s| s == no_prop[0]),
            "crossed-line-propagator and propagator-free diagrams must share a spine \
             sign, got {crossed_prop:?} / {no_prop:?}"
        );
        assert_eq!(
            mixed_prop[0], -no_prop[0],
            "a propagator on a mixed line must flip the spine sign relative to one on a \
             crossed line"
        );

        // Negative control: a crossed line's −1 does not count propagators.
        let ttx: Vec<i8> = generate("g g > t t~")
            .iter()
            .flat_map(|set| set.diagrams.iter())
            .map(|d| d.fermion_line_sign(&model))
            .collect();
        assert_eq!(
            ttx,
            vec![-1, -1, -1],
            "g g > t t~ must carry one crossed-line −1 per diagram regardless of the \
             top-line propagator"
        );
    }

    /// The two pairings of the four-lepton contact vertex give a process two different
    /// fermion-line topologies, and the rooting reads each diagram's own.
    ///
    /// With `e+ e- > mu+ mu-`'s legs `[e+ in, e- in, mu- out, mu+ out]`, the
    /// `(1,4)(2,3)` structures join the two muons and the two electrons — a *crossed*
    /// line (both ends final) and an initial–initial one — while the `(1,2)(3,4)`
    /// structure joins each muon to an electron, two *mixed* lines whose final legs must
    /// be typed by their physical wavefunction rather than the all-incoming one. So the
    /// same four external legs are built differently in the two diagrams, and each
    /// carries the spine sign its own topology implies: one crossed line is a −1, two
    /// mixed propagator-free lines are not.
    ///
    /// This is the falsifier for reading the pairing per structure. Before it, every
    /// structure of a vertex took the first one's map, so contact diagrams were built on
    /// the wrong lines — with the same particle content, the same diagram count and the
    /// same colour, which is why nothing but the amplitude could see it.
    ///
    /// `e+ e- > e+ e-` is the second half. There one interaction carries both pairings,
    /// so splitting the *interaction* by coupling order (which is what separates them in
    /// the two-flavour process) does not, and this split is the only thing that does.
    /// Which pairing gives which topology is the other way round there, because the
    /// enumeration binds the identical electrons to the vertex's slots in a different
    /// order — which is exactly why the topology has to be read off the diagram rather
    /// than assumed from the structure.
    #[test]
    fn four_fermion_pairings_type_a_diagram_s_external_legs() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let model = crate::ufo::UFOModel::load(
            &root.join("../validation/ufo/SMEFTsim_topU3l_MwScheme_UFO"),
            Some(&root.join("../validation/madgraph/cards/smeft/restrict_vg_c4l.dat")),
        )
        .expect("load SMEFTsim under vg_c4l");
        let opts = ParsingOptions::default();

        // Per contact diagram: the vertex's pairing, the final legs restored to their
        // physical wavefunction, and the diagram's spine sign.
        let topologies = |process: &str| -> Vec<(Vec<isize>, Vec<usize>, i8)> {
            let card = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
            let sets = generate_from_proc_card(&card, &model).unwrap();
            let mut seen: Vec<(Vec<isize>, Vec<usize>, i8)> = Vec::new();
            for set in &sets {
                for diagram in &set.diagrams {
                    // The contact diagrams are the single-vertex ones.
                    if diagram.vertices.len() != 1 {
                        continue;
                    }
                    let vtx = &diagram.vertices[0];
                    let group = vertex_flow_group(&model, vtx.interaction, vtx.flow_group);
                    let mut uncross: Vec<usize> = mixed_line_final_legs(diagram, &model)
                        .into_iter()
                        .map(|l| l.0)
                        .collect();
                    uncross.sort();
                    let tree = root_tree_at(diagram, &model, &[0u8], VtxIdx(0)).unwrap();
                    seen.push((group.spin_map.clone(), uncross, spine_sign_from_flow(&tree)));
                }
            }
            seen.sort();
            seen.dedup();
            seen
        };

        assert_eq!(
            topologies("e+ e- > mu+ mu- NP<=1"),
            vec![
                // (1,2)(3,4): mu+ with e-, e+ with mu- — two mixed lines, both final
                // legs restored to their physical wavefunction, no crossed −1.
                (vec![1, 0, 3, 2], vec![2, 3], 1),
                // (1,4)(2,3): the muon line is crossed and the electron line is
                // initial–initial, so nothing is uncrossed and the crossed line is a −1.
                (vec![3, 2, 1, 0], vec![], -1),
            ]
        );
        assert_eq!(
            topologies("e+ e- > e+ e- NP<=1"),
            vec![
                (vec![1, 0, 3, 2], vec![], -1),
                (vec![3, 2, 1, 0], vec![2, 3], 1),
            ]
        );
    }
    /// The triple-gluon sign is a property of the *three*-vector vertex, and a
    /// momentum-bearing contact of four or more vectors is not one.
    ///
    /// An all-vector contact can carry momenta — `O_G` gives SMEFTsim four- and
    /// five-gluon structures built from momenta and metrics — so "all vector legs and
    /// some `P`" does not separate the triple vertex from the contact, and counted as a
    /// triple-gluon vertex such a contact would take its sign.
    ///
    /// Both halves are checked: the predicate itself, and that the model really does
    /// carry the momentum-bearing contacts that make the leg count do work — a check on
    /// a model whose only contact were the pure-metric Standard-Model one would pass
    /// with or without the arity test.
    #[test]
    fn a_momentum_bearing_vector_contact_is_not_a_triple_vector_vertex() {
        use crate::ufo::lorentz::LorentzOp;

        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let model = crate::ufo::UFOModel::load(
            &root.join("../validation/ufo/SMEFTsim_topU3l_MwScheme_UFO"),
            Some(&root.join("../validation/madgraph/cards/smeft/restrict_vg_cG.dat")),
        )
        .expect("load SMEFTsim under vg_cG");

        let mut momentum_bearing_by_legs: std::collections::BTreeMap<usize, usize> =
            Default::default();
        for (name, vertex) in &model.vertices {
            let id = model.vertex_id(name).expect("every vertex has an id");
            let all_vector = vertex
                .particles
                .iter()
                .all(|&p| model.particle(p).spin == 3);
            if !all_vector {
                assert!(
                    !is_yang_mills_vvv(&model, id),
                    "vertex '{name}' has a non-vector leg"
                );
                continue;
            }
            let has_momentum = vertex.lorentz.iter().any(|&lid| {
                model
                    .lorentz_struct(lid)
                    .expr
                    .iter()
                    .any(|t| t.ops.iter().any(|op| matches!(op, LorentzOp::P { .. })))
            });
            if has_momentum {
                *momentum_bearing_by_legs
                    .entry(vertex.particles.len())
                    .or_default() += 1;
            }
            assert_eq!(
                is_yang_mills_vvv(&model, id),
                has_momentum && vertex.particles.len() == 3,
                "vertex '{name}' with {} vector legs, momentum-bearing {has_momentum}",
                vertex.particles.len()
            );
        }
        assert!(
            momentum_bearing_by_legs.get(&4).copied().unwrap_or(0) > 0
                && momentum_bearing_by_legs.get(&5).copied().unwrap_or(0) > 0,
            "this card carries no momentum-bearing vector contact, so the leg count \
             does no work here: {momentum_bearing_by_legs:?}"
        );
    }
}
