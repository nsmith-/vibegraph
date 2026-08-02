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
        chain: &[u8],
    ) -> Result<Self, RootLorentzError> {
        let mut nodes = Vec::with_capacity(raw.nodes.len());
        let (root, _) = Self::bake_node(
            raw, raw.root, diagram, model, n_in, uncross, chain, &mut nodes,
        )?;
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
    #[allow(clippy::too_many_arguments)]
    fn bake_node(
        raw: &RawDiagramTree,
        id: RawNodeId,
        diagram: &Diagram,
        model: &UFOModel,
        n_in: usize,
        uncross: &HashSet<LegIdx>,
        chain: &[u8],
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
                vtx,
                result,
                children,
            } => {
                let baked: Vec<(EvalNodeId, Option<LegAdjoint>)> = children
                    .iter()
                    .map(|&c| Self::bake_node(raw, c, diagram, model, n_in, uncross, chain, nodes))
                    .collect::<Result<Vec<_>, _>>()?;
                let child_ids: Vec<EvalNodeId> = baked.iter().map(|(id, _)| *id).collect();
                let color_idx = chain[vtx.0] as usize;
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
                        let info =
                            VertexInfo::from_ufo(model, *vertex, color_idx, Some(ri.0), &flows)?;
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
                        let info = VertexInfo::from_ufo(model, *vertex, color_idx, None, &flows)?;
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

    /// The diagram's rooting-convention sign: the product over its vertices of each
    /// vertex's [`build_sign`](super::diagram_eval::VertexInfo::build_sign) (VVS
    /// `pure_metric`, FFS scalar-sink, crossed-pair). Each vertex's sign depends on which
    /// leg the rooting made its output, so this is only rooting-invariant when read off a
    /// tree built at the canonical `VtxIdx(0)` rooting — which [`compile_single_diagram`]
    /// does, folding the result into `fermi_sign` so the honest (sign-free) currents stay
    /// root-invariant. Mirrors [`yang_mills_vvv_sign`], which carries the VVV vertex sign
    /// the same way.
    pub(super) fn build_convention_sign(&self) -> i8 {
        let mut sign = 1i8;
        for id in self.iter() {
            let info = match self.value(id) {
                EvalNode::OffShellCurrent { info, .. }
                | EvalNode::ContractAmplitude { info, .. } => info,
                _ => continue,
            };
            sign *= info.build_sign();
        }
        sign
    }

    /// The diagram's runtime `reversed`-bilinear parity: the product over its vertices of
    /// each vertex's [`reversed_sign`](super::diagram_eval::VertexInfo::reversed_sign).
    /// Like [`build_convention_sign`](Self::build_convention_sign), it depends on the
    /// rooting (a fermion→vector sink under one rooting is a fermion-continuing current
    /// under another), so [`compile_single_diagram`] folds `P_canonical · P_live` into
    /// `fermi_sign`: `P_live` cancels the parity the runtime `resolve_bra_ket` actually
    /// applies on the live tree, and `P_canonical` reinstates the canonical one. When the
    /// live rooting coincides with the canonical `VtxIdx(0)` (every diagram whose chosen
    /// root is vertex 0) the two are equal and the factor is `+1`; when they differ this
    /// correction is what keeps the re-rooted amplitude equal to the canonical one.
    pub(super) fn reversed_convention_sign(&self) -> i8 {
        let mut sign = 1i8;
        for id in self.iter() {
            let info = match self.value(id) {
                EvalNode::OffShellCurrent { info, .. }
                | EvalNode::ContractAmplitude { info, .. } => info,
                _ => continue,
            };
            sign *= info.reversed_sign();
        }
        sign
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
/// Returns the external leg index where the line terminates and how many internal
/// propagators the trace crossed.
#[cfg(test)]
fn trace_fermion_line(
    diagram: &Diagram,
    model: &UFOModel,
    start_vtx: VtxIdx,
    in_ray: RaySlot,
) -> (LegIdx, usize) {
    let mut vtx = start_vtx;
    let mut in_ray = in_ray;
    let mut n_props = 0usize;
    // Tree diagrams terminate; the bound only guards against pathological loops.
    for _ in 0..1024 {
        let vertex = diagram.vertex(vtx);
        let lid = model.vertex_def(vertex.interaction).lorentz[0];
        let out_ray = model.lorentz_struct(lid).spin_map[in_ray.0] as usize;
        match vertex.rays[out_ray] {
            Ray::Leg(li) => return (li, n_props),
            Ray::Prop { prop, end } => {
                let (next_vtx, next_slot) = diagram.prop(prop).endpoints[1 - end];
                n_props += 1;
                vtx = next_vtx;
                in_ray = next_slot;
            }
        }
    }
    panic!("fermion line trace did not terminate");
}

/// Relative fermion sign that feyngraph's connectivity-based `view.sign()` omits.
///
/// Independent `spin_map`-tracing oracle for [`spine_sign_from_flow`]; see there for
/// the derivation. Detected structurally by tracing each external fermion line: one
/// −1 per internal propagator on a line with at least one initial-state endpoint, and
/// one −1 per final–final line.
#[cfg(test)]
fn reversed_line_propagator_sign(diagram: &Diagram, model: &UFOModel) -> i8 {
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
        let (other, n_props) = trace_fermion_line(diagram, model, attach_vtx, attach_slot);
        visited.insert(other);
        let crossed = li.0 >= n_in && other.0 >= n_in;
        if !crossed && n_props % 2 == 1 {
            sign = -sign;
        }
        if crossed {
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
/// `spin_map`-tracing `reversed_line_propagator_sign` is kept as a test oracle and proven
/// equivalent by `spine_sign_from_flow_matches_heuristic`.)
///
/// A fermion line terminates at any vertex node that outputs a non-fermion yet has two
/// fermion children: an FFV/FFS current rooted at its boson leg, or the root
/// contraction. Each fermion line meets exactly one such sink, so every line is
/// counted once.
///
/// The two flips below both come from the same place: which UFO slot each external
/// wavefunction is bound to. Diagram enumeration binds slots in the *all-incoming*
/// identity, so an outgoing leg is bound to its antiparticle's slot; the reference
/// HELAS bookkeeping binds them in the all-outgoing identity, so an *incoming* leg is
/// bound to its antiparticle's slot. A vertex slot pairs with a definite spinor adjoint
/// (the pair-first slot takes the ket, the pair-second the bra), so the two bindings
/// disagree exactly on the legs neither convention crosses the same way:
///
/// * A line with at least one **initial-state** endpoint is bound against its own
///   arrow at every vertex (the initial leg always, and a mixed line's final leg
///   because [`mixed_line_final_legs`] restores its physical wavefunction while the
///   slot stays the all-incoming one). Reading a bilinear against its slot arrow
///   replaces each vertex structure by `C Γᵀ C⁻¹`, which for `Γ = γ^μ P_χ` is
///   `−γ^μ P_χ̄`: the chirality flip is applied per vertex by
///   [`chiral_correction`](super::root_lorentz), and one of the `V` minus signs is
///   supplied by [`reversed_convention_sign`](DiagramEvalTree::reversed_convention_sign)
///   at the line's single vector-rooted sink. The remaining `V − 1` — one per internal
///   fermion propagator on the line — are this flip. Pinned by the uux 2→6 per-diagram
///   oracle for the initial–initial case and by `u d > e+ e- u d QCD=0`, whose 35
///   diagrams split 24/11 on whether a *mixed* quark line carries the propagator, for
///   the mixed case.
/// * A **crossed line** — both endpoints final-state, kept in the all-incoming
///   (conjugate-wavefunction) representation — is bound *along* its arrow at every
///   vertex, so it takes no per-propagator factor; its single −1 is the operator
///   reordering of the conjugated pair relative to the reference's physical pair.
///   Invisible while every diagram of a process has the same crossed-line count
///   (uniform sign); exposed and pinned by Bhabha, where the s-channel has one crossed
///   line and the t-channel none, and its propagator-independence by `g g > t t~`,
///   whose s-channel top line carries no propagator and whose t/u-channel lines carry
///   one.
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
            let crossed = !inc_a && !inc_b;
            if !crossed && (n_props_a + n_props_b) % 2 == 1 {
                sign = -sign;
            }
            if crossed {
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
            ..
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
/// currents shared across more diagrams: a deep, few-external anchor keeps the sub-currents
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

/// Root a diagram at an explicit vertex (bypassing [`choose_root`]). Used to build the
/// canonical `VtxIdx(0)` tree for the rooting-invariant convention sign
/// ([`DiagramEvalTree::build_convention_sign`], [`spine_sign_from_flow`]) whenever the
/// live evaluation tree is rooted elsewhere — either by [`canonical_root`] in production
/// or by a soundness-harness override.
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

    let n_in = diagram.n_in;
    let uncross = mixed_line_final_legs(&raw, model, n_in);
    Ok(DiagramEvalTree::bake(
        &raw, diagram, model, n_in, &uncross, chain,
    )?)
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

/// True iff `interaction` is a Yang-Mills triple-vector (VVV) vertex: an all-vector
/// vertex whose Lorentz structure carries a momentum (`P`) factor. Its rooted vector
/// current ([`super::root_lorentz`]) is now built honestly (`+V^μ`), so relative to
/// MadGraph it needs a −1 at every rooting where the vertex is a *source* (off-shell
/// vector current), supplied rooting-invariantly by [`yang_mills_vvv_sign`]. The
/// 4-vector contact (VVVV) is all-vector but momentum-free, so it is excluded — its
/// −1 is the pure-metric vertex factor already applied symmetrically in both source
/// and sink modes.
fn is_yang_mills_vvv(model: &UFOModel, interaction: VertexId) -> bool {
    use crate::ufo::lorentz::LorentzOp;
    let def = model.vertex_def(interaction);
    def.particles.iter().all(|&p| model.particle(p).spin == 3)
        && def.lorentz.iter().any(|&lid| {
            model
                .lorentz_struct(lid)
                .expr
                .iter()
                .any(|t| t.ops.iter().any(|op| matches!(op, LorentzOp::P { .. })))
        })
}

/// The rooting-invariant sign a diagram picks up from its Yang-Mills (VVV) vertices.
///
/// The honest vector current is root-invariant, but a VVV vertex needs a −1 relative
/// to it whenever it sits at a vector *output* (source) leg rather than the amplitude
/// sink. The convention reference roots at `VtxIdx(0)`, so exactly the VVV vertices at
/// indices `1..` are sources; each contributes a −1. Deriving the sign from the *fixed*
/// diagram (vertex 0 is the canonical root) rather than the live evaluation rooting decouples
/// it from the root choice: the honest current handles the tensor contraction root-
/// invariantly and this scalar carries the antisymmetric-vertex sign, so their product
/// reproduces the `VtxIdx(0)` amplitude bit-for-bit for every re-rooting.
fn yang_mills_vvv_sign(diagram: &Diagram, model: &UFOModel) -> i8 {
    let sources = diagram
        .vertices
        .iter()
        .skip(1)
        .filter(|v| is_yang_mills_vvv(model, v.interaction))
        .count();
    if sources % 2 == 0 {
        1
    } else {
        -1
    }
}

/// Compile a single diagram into an evaluable [`DiagramEval`].
///
/// Roots the diagram into its evaluation tree (topology + Lorentz structures) and
/// attaches the per-diagram metadata (external-leg count, symmetry factor, and the
/// fermion-adjoint sign, including the initial-state spine correction derived from the
/// baked spinor adjoint via [`spine_sign_from_flow`], plus the Yang-Mills VVV vertex
/// sign from [`yang_mills_vvv_sign`]).
pub(super) fn compile_single_diagram(
    diagram: &Diagram,
    model: &UFOModel,
    chain: &[u8],
) -> Result<DiagramEval, CompileError> {
    let tree = root_tree(diagram, model, chain)?;
    // The rooting-convention signs (`build_convention_sign`, `spine_sign_from_flow`) depend
    // on the output-leg orientation the rooting chose, but the honest currents do not. To
    // keep the amplitude root-invariant, read those signs off the *canonical* `VtxIdx(0)`
    // tree rather than the live evaluation tree. The live tree coincides with the canonical
    // one for every diagram whose chosen root ([`canonical_root`]) is vertex 0 (all 2→2
    // processes, whose vertices tie on external-leg count); it diverges when the chosen
    // root is elsewhere, and then the separate canonical tree carries the signs.
    let canonical_owned;
    let canonical = if choose_root(diagram) == VtxIdx(0) {
        &tree
    } else {
        canonical_owned = root_tree_at(diagram, model, chain, VtxIdx(0))?;
        &canonical_owned
    };
    // The runtime `resolve_bra_ket` applies the live tree's reversed-bilinear parity
    // (`tree.reversed_convention_sign()`); multiplying by it cancels that and by the
    // canonical parity reinstates the rooting-invariant one. When the live tree is the
    // canonical one the product is `+1` and the runtime sign is left untouched; otherwise
    // it re-expresses the live parity in the canonical frame.
    let fermi_sign = diagram.sign
        * spine_sign_from_flow(canonical)
        * yang_mills_vvv_sign(diagram, model)
        * canonical.build_convention_sign()
        * canonical.reversed_convention_sign()
        * tree.reversed_convention_sign();
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
    #[ignore]
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

    /// The Yang-Mills VVV sign fires only where a triple-vector vertex is a source: a
    /// process with a VVV vertex off the canonical root exercises a −1, a VVV-free
    /// process is uniformly +1, and a process whose canonical root *is* the VVV (all
    /// its VVVs at index 0) stays +1. Guards `is_yang_mills_vvv`'s detection and the
    /// index-`1..` source count that make the fix bit-exact at production rooting.
    #[test]
    fn yang_mills_vvv_sign_fires_only_for_source_vvv() {
        let model = sm_model(SMRestrict::Default);

        // e+ e- > W+ W-: the s-channel γ/Z→WW diagram has the VVV as a non-root source.
        let signs: Vec<i8> = generate("e+ e- > W+ W-")
            .iter()
            .flat_map(|set| set.diagrams.iter())
            .map(|d| yang_mills_vvv_sign(d, &model))
            .collect();
        assert!(
            signs.contains(&-1),
            "e+ e- > W+ W- must exercise a −1 from a VVV source, got {signs:?}"
        );

        // e+ e- > mu+ mu-: no VVV vertex anywhere → uniformly +1.
        for set in generate("e+ e- > mu+ mu-") {
            for d in &set.diagrams {
                assert_eq!(
                    yang_mills_vvv_sign(d, &model),
                    1,
                    "VVV-free process must stay +1"
                );
            }
        }
        // The all-vector-but-momentum-free 4-gluon contact (VVVV) must not be counted
        // as a Yang-Mills VVV; that it isn't is pinned bit-for-bit by g g > g g in
        // `tests/amplitude_oracle.rs` (miscounting it would flip that amplitude).
    }

    /// Number of diagrams of `process` whose canonical `VtxIdx(0)` tree fires each
    /// rooting-convention sign channel. Rooting at the canonical vertex is what
    /// production uses and what the `fermi_sign` lift reads, so a channel that fires
    /// here is a channel [`compile_single_diagram`] genuinely carries.
    fn channel_counts(model: &UFOModel, process: &str) -> (usize, usize, usize, usize) {
        let (mut vvv, mut spine, mut build, mut reversed) = (0, 0, 0, 0);
        for set in generate(process) {
            for diagram in &set.diagrams {
                let chain = vec![0u8; diagram.vertices.len()];
                let t = root_tree_at(diagram, model, &chain, VtxIdx(0)).unwrap();
                vvv += (yang_mills_vvv_sign(diagram, model) < 0) as usize;
                spine += (spine_sign_from_flow(&t) < 0) as usize;
                build += (t.build_convention_sign() < 0) as usize;
                reversed += (t.reversed_convention_sign() < 0) as usize;
            }
        }
        (vvv, spine, build, reversed)
    }

    /// Coverage map for the rooting-convention sign channels lifted into `fermi_sign`
    /// (`research/notes/19` §V5): each channel is exercised by a *named* process, so a
    /// refactor or enumeration change that silently stops exercising a branch fails here
    /// rather than rotting undetected against its still-exercised sibling — the failure
    /// mode that produced the `g g > g g` VVVV phase bug (note 16 §6).
    ///
    /// This is the non-vacuity half of the map; the deeper per-channel properties live in
    /// dedicated tests — [`yang_mills_vvv_sign_fires_only_for_source_vvv`] (VVV `σ_V`,
    /// with the +1-uniform and VVVV-not-counted arms),
    /// [`spine_sign_from_flow_matches_heuristic`] (spine sign vs the `spin_map` oracle)
    /// and [`spine_sign_separates_mixed_line_and_crossed_line_propagators`] (the spine
    /// sign's two arms, mixed-line-per-propagator vs crossed-line-once).
    /// The one sub-branch no default-suite process reaches — the VVS pure-metric −1 with
    /// the *scalar* leg as output (H produced from two vectors, only in the 2→6 H
    /// classes) — is pinned at the primitive level by
    /// `root_lorentz::tests::test_root_vvs_metric_scalar_out` and bit-for-bit by
    /// `u u~/b b~ > … QCD=0` in `tests/amplitude_oracle.rs`.
    #[test]
    fn mg_guard_processes_exercise_every_convention_channel() {
        let model = sm_model(SMRestrict::Default);

        // VVV σ_V: the s-channel γ/Z → W+W- vertex is a non-root vector source.
        assert!(
            channel_counts(&model, "e+ e- > W+ W-").0 > 0,
            "e+ e- > W+ W- must exercise the Yang-Mills VVV source sign"
        );
        // Spine sign: Bhabha's s-channel has one crossed (final-final) fermion line.
        assert!(
            channel_counts(&model, "e+ e- > e+ e-").1 > 0,
            "e+ e- > e+ e- must exercise the crossed-line spine sign"
        );
        // Build-convention −1, VVVV pure-metric arm: g g > g g has no fermion or scalar
        // externals, so its 4-gluon contact is the *only* source of a build sign — this
        // is the single-process branch of the note-16 §6 bug.
        assert!(
            channel_counts(&model, "g g > g g").2 > 0,
            "g g > g g must exercise the VVVV pure-metric build sign"
        );
        // Build-convention −1, FFS/crossed scalar-sink arm: e+ e- > ta+ ta- H has no
        // pure-vector vertex, so its build sign comes only from the τ-Yukawa scalar
        // bilinear (ProjM/ProjP scalar-sink + the crossed-τ standalone projector).
        assert!(
            channel_counts(&model, "e+ e- > ta+ ta- H").2 > 0,
            "e+ e- > ta+ ta- H must exercise the scalar-bilinear build sign"
        );
        // Reversed-bilinear parity: e+ e- > mu+ mu- has a reversed FFV bilinear, so the
        // per-diagram `reversed_convention_sign` is non-trivial — i.e. the runtime
        // `resolve_bra_ket` parity cancellation folded into `fermi_sign` is load-bearing,
        // not a global no-op. (Pure-gauge g g > g g / g g > t t~ never reverse a
        // bilinear, so this channel would go unexercised without a fermion process.)
        assert!(
            channel_counts(&model, "e+ e- > mu+ mu-").3 > 0,
            "e+ e- > mu+ mu- must exercise the reversed-bilinear parity channel"
        );
    }

    /// The adjoint-derived spine sign must agree with the `spin_map`-tracing heuristic for
    /// every diagram, across processes with and without off-shell fermion spines.
    #[test]
    fn spine_sign_from_flow_matches_heuristic() {
        let model = sm_model(SMRestrict::Default);
        let processes = [
            "e+ e- > mu+ mu-",
            "e+ e- > e+ e-",
            "u u~ > d d~",
            "e+ e- > mu+ mu- ta+ ta-",
            "u d > e+ e- u d QCD=0",
        ];
        let mut flipped_total = 0;
        for process in processes {
            for set in generate(process) {
                for (i, diagram) in set.diagrams.iter().enumerate() {
                    let chain = vec![0u8; diagram.vertices.len()];
                    let tree = root_tree(diagram, &model, &chain).expect("rooting failed");
                    let from_flow = spine_sign_from_flow(&tree);
                    let heuristic = reversed_line_propagator_sign(diagram, &model);
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

    /// The per-propagator flip fires on a *mixed* (initial↔final) fermion line, not only
    /// on an initial–initial one, and a *crossed* (final–final) line takes its single −1
    /// regardless of how many propagators it carries.
    ///
    /// `u d > e+ e- u d QCD=0` is the discriminating topology: every one of its 35
    /// diagrams has exactly one internal fermion propagator except the two with a triple-
    /// gauge vertex, and the propagator sits either on a mixed quark line (24 diagrams)
    /// or on the crossed lepton line (9). The two classes must therefore come out with
    /// *opposite* spine signs — which is the relative sign between MadGraph graphs
    /// 1-8/17 and the rest. `g g > t t~` supplies the negative control: its s-channel top
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
                let chain = vec![0u8; diagram.vertices.len()];
                let tree = root_tree_at(diagram, &model, &chain, VtxIdx(0)).unwrap();
                let sign = spine_sign_from_flow(&tree);
                // Classify by where the diagram's internal fermion propagator sits.
                let mut visited: HashSet<LegIdx> = HashSet::new();
                let (mut on_mixed, mut on_crossed) = (0usize, 0usize);
                for leg in &diagram.legs {
                    let li = leg.leg_idx;
                    if leg.spin.abs() != 2 || !visited.insert(li) {
                        continue;
                    }
                    let (attach_vtx, attach_slot) = diagram.leg_attachment(li);
                    let (other, n_props) =
                        trace_fermion_line(diagram, &model, attach_vtx, attach_slot);
                    visited.insert(other);
                    if li.0 >= n_in && other.0 >= n_in {
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
            .map(|d| {
                let chain = vec![0u8; d.vertices.len()];
                spine_sign_from_flow(&root_tree_at(d, &model, &chain, VtxIdx(0)).unwrap())
            })
            .collect();
        assert_eq!(
            ttx,
            vec![-1, -1, -1],
            "g g > t t~ must carry one crossed-line −1 per diagram regardless of the \
             top-line propagator"
        );
    }
}
