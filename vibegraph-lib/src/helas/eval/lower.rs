//! Pass 3a: inline the pass-1+2 `DiagramEval`s into one unified [`Ast<Sym>`].
//!
//! Each diagram's `DiagramEvalTree` and the per-vertex `LorentzEvalTree`s are flattened
//! into a single arena over the whole amplitude:
//! - external legs → `External` (with a `Mass` child),
//! - propagators → `Propagate(current, Mass, Width)`,
//! - a vertex's `coupling · Σ_k coeff_k · structure_k` → `Mul`/`Add` over `Coupling`/
//!   `Coeff` leaves and the inlined Lorentz structure,
//! - the Lorentz tree's `Leg(i)`/`P{leg}` → the (shared) lowered input child / `PMom`,
//! - per-diagram `symmetry_factor · fermi_sign` → a `Mul` with a `Coeff` leaf,
//! - the whole amplitude → one `Add` over the diagram roots.
//!
//! Input currents are lowered once and referenced by id, so a current feeding several
//! summed terms is shared (a DAG), not duplicated.

use std::collections::HashMap;

use super::ast::{Ast, AstBuilder};
use super::diagram_eval::VertexInfo;
use super::op::{NodeId, Op, Sym};
use super::root_diagram::{DiagramEval, DiagramEvalTree, EvalNode, EvalNodeId};
use super::root_lorentz::{LorentzEvalNode, LorentzEvalTree};
use super::tree::Tree;
use crate::ufo::couplings::CouplingId;
use crate::ufo::particles::ParticleId;

/// Inline every diagram into a single whole-amplitude [`Ast<Sym>`].
pub fn lower(diagrams: &[DiagramEval]) -> Ast<Sym> {
    let mut b = AstBuilder::new();
    let mut diagram_roots = Vec::with_capacity(diagrams.len());
    for d in diagrams {
        let amp = lower_diagram_node(&d.tree, d.tree.root(), &mut b);
        // Fold the per-diagram symmetry factor and Fermi sign into a real coefficient.
        let factor = d.symmetry_factor * (d.fermi_sign as f64);
        let coeff = b.add(Op::Coeff, Sym::Coeff(factor), vec![]);
        diagram_roots.push(b.add(Op::Mul, Sym::None, vec![coeff, amp]));
    }
    let root = sum_or_single(&mut b, diagram_roots);
    b.finish(root)
}

/// Structure-optimization pass. Today this is common-subexpression elimination;
/// the planned egglog rewrite stage will run *before* it (egglog extraction yields
/// a minimal tree, not a minimal DAG, so CSE stays as the tree→DAG post-process).
pub fn optimize(ast: Ast<Sym>) -> Ast<Sym> {
    let deduped = cse(&ast);
    log::debug!(
        "optimize: {} nodes → {} after CSE",
        ast.len(),
        deduped.len()
    );
    deduped
}

/// Hashable identity of a node's leaf payload (`f64` coeffs compare by bit pattern,
/// so two `Coeff` nodes merge only when they are the same IEEE value).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum LeafKey {
    Coupling(CouplingId),
    Particle(ParticleId),
    Coeff(u64),
    Ext {
        leg_idx: usize,
        spin: i32,
        sign: i32,
        incoming: bool,
    },
    None,
}

fn leaf_key(leaf: &Sym) -> LeafKey {
    match *leaf {
        Sym::Coupling(id) => LeafKey::Coupling(id),
        Sym::Particle(id) => LeafKey::Particle(id),
        Sym::Coeff(c) => LeafKey::Coeff(c.to_bits()),
        Sym::Ext {
            leg_idx,
            spin,
            charge,
            incoming,
        } => LeafKey::Ext {
            leg_idx,
            spin,
            sign: charge.sign(),
            incoming,
        },
        Sym::None => LeafKey::None,
    }
}

/// Hash-cons the arena into a DAG: one forward scan (children precede parents, so
/// each node's children are already remapped) interning every node by
/// `(op, leaf, remapped children)`. Structurally identical subtrees collapse to one
/// node; child order is preserved, so evaluation results — and their floating-point
/// operation order — are unchanged.
fn cse(ast: &Ast<Sym>) -> Ast<Sym> {
    let mut b = AstBuilder::new();
    let mut interned: HashMap<(Op, LeafKey, Vec<NodeId>), NodeId> = HashMap::new();
    let mut remap: Vec<NodeId> = Vec::with_capacity(ast.len());
    for id in ast.iter() {
        let node = ast.value(id);
        let children: Vec<NodeId> = ast
            .children_ids(id)
            .iter()
            .map(|&c| remap[c as usize])
            .collect();
        let new_id = *interned
            .entry((node.op, leaf_key(&node.leaf), children.clone()))
            .or_insert_with(|| b.add(node.op, node.leaf, children));
        remap.push(new_id);
    }
    b.finish(remap[ast.root() as usize])
}

fn sum_or_single(b: &mut AstBuilder<Sym>, mut nodes: Vec<NodeId>) -> NodeId {
    match nodes.len() {
        1 => nodes.pop().unwrap(),
        _ => b.add(Op::Add, Sym::None, nodes),
    }
}

fn lower_diagram_node(tree: &DiagramEvalTree, id: EvalNodeId, b: &mut AstBuilder<Sym>) -> NodeId {
    match tree.value(id) {
        EvalNode::External(info) => {
            let mass = b.add(Op::Mass, Sym::Particle(info.id), vec![]);
            b.add(
                Op::External,
                Sym::Ext {
                    leg_idx: info.leg_idx,
                    spin: info.spin,
                    charge: info.charge,
                    incoming: info.incoming,
                },
                vec![mass],
            )
        }
        EvalNode::OffShellCurrent { info, children, .. } => {
            let inputs = lower_children(tree, children, b);
            lower_vertex(info, &inputs, b)
        }
        EvalNode::Propagate { info, child, .. } => {
            let current = lower_diagram_node(tree, *child, b);
            let mass = b.add(Op::Mass, Sym::Particle(info.id), vec![]);
            // t-channel (spacelike) propagators can never resonate: MadGraph
            // passes ZERO width for them, and we bake the same zero in here.
            let width = if info.t_channel {
                b.add(Op::Coeff, Sym::Coeff(0.0), vec![])
            } else {
                b.add(Op::Width, Sym::Particle(info.id), vec![])
            };
            b.add(Op::Propagate, Sym::None, vec![current, mass, width])
        }
        EvalNode::ContractAmplitude { info, children } => {
            let inputs = lower_children(tree, children, b);
            lower_vertex(info, &inputs, b)
        }
    }
}

fn lower_children(
    tree: &DiagramEvalTree,
    children: &[EvalNodeId],
    b: &mut AstBuilder<Sym>,
) -> Vec<NodeId> {
    children
        .iter()
        .map(|&c| lower_diagram_node(tree, c, b))
        .collect()
}

/// Lower `Σ_terms coupling · (Σ_k coeff_k · structure_k)` against the gap-free input
/// currents `inputs` (vertex legs in order, output omitted).
fn lower_vertex(info: &VertexInfo, inputs: &[NodeId], b: &mut AstBuilder<Sym>) -> NodeId {
    let mut term_nodes = Vec::with_capacity(info.terms.len());
    for vt in &info.terms {
        let coupling = b.add(Op::Coupling, Sym::Coupling(vt.coupling_id), vec![]);
        let mut structures = Vec::with_capacity(vt.terms.len());
        for rt in &vt.terms {
            let coeff = b.add(Op::Coeff, Sym::Coeff(rt.coeff), vec![]);
            let structure = lower_lorentz(&rt.tree, rt.tree.root(), inputs, b);
            structures.push(b.add(Op::Mul, Sym::None, vec![coeff, structure]));
        }
        let summed = sum_or_single(b, structures);
        term_nodes.push(b.add(Op::Mul, Sym::None, vec![coupling, summed]));
    }
    sum_or_single(b, term_nodes)
}

/// Inline one rooted `LorentzEvalTree`. `Leg(i)` resolves to the shared `inputs[i]`
/// node; every other node maps to a unified `Op`, recursing on the Lorentz tree's
/// child node indices.
fn lower_lorentz(
    lt: &LorentzEvalTree,
    n: usize,
    inputs: &[NodeId],
    b: &mut AstBuilder<Sym>,
) -> NodeId {
    use LorentzEvalNode as L;
    let rec = |child: usize, b: &mut AstBuilder<Sym>| lower_lorentz(lt, child, inputs, b);
    match *lt.value(n) {
        L::Leg(i) => inputs[i],
        L::P { leg } => b.add(Op::PMom, Sym::None, vec![inputs[leg]]),
        L::POut => b.add(Op::PMomOut, Sym::None, inputs.to_vec()),
        L::GammaVout { i, j } => {
            let a = rec(i, b);
            let c = rec(j, b);
            b.add(Op::GammaVout, Sym::None, vec![a, c])
        }
        L::GammaIout { mu, j } => {
            let a = rec(mu, b);
            let c = rec(j, b);
            b.add(Op::GammaIout, Sym::None, vec![a, c])
        }
        L::GammaOout { mu, i } => {
            let a = rec(mu, b);
            let c = rec(i, b);
            b.add(Op::GammaOout, Sym::None, vec![a, c])
        }
        L::ProjM { i } => {
            let a = rec(i, b);
            b.add(Op::ProjM, Sym::None, vec![a])
        }
        L::ProjP { i } => {
            let a = rec(i, b);
            b.add(Op::ProjP, Sym::None, vec![a])
        }
        L::ProjMAmp { i, j } => {
            let a = rec(i, b);
            let c = rec(j, b);
            b.add(Op::ProjMAmp, Sym::None, vec![a, c])
        }
        L::ProjPAmp { i, j } => {
            let a = rec(i, b);
            let c = rec(j, b);
            b.add(Op::ProjPAmp, Sym::None, vec![a, c])
        }
        L::Metric { mu, nu } => {
            let a = rec(mu, b);
            let c = rec(nu, b);
            b.add(Op::Metric, Sym::None, vec![a, c])
        }
        L::MetricNegI { mu, nu } => {
            let a = rec(mu, b);
            let c = rec(nu, b);
            b.add(Op::MetricNegI, Sym::None, vec![a, c])
        }
        L::MetricVout { v } => {
            let a = rec(v, b);
            b.add(Op::MetricVout, Sym::None, vec![a])
        }
        L::LowerVout { v } => {
            let a = rec(v, b);
            b.add(Op::LowerVout, Sym::None, vec![a])
        }
        L::IdentityAmp { i, j } => {
            let a = rec(i, b);
            let c = rec(j, b);
            b.add(Op::IdentityAmp, Sym::None, vec![a, c])
        }
        L::Mul { ref children } => {
            let cs: Vec<NodeId> = children.iter().map(|&c| rec(c, b)).collect();
            b.add(Op::Mul, Sym::None, cs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Structurally identical subtrees collapse to shared nodes, and the rendered
    /// s-expression — which re-expands shared subtrees per parent — is unchanged.
    #[test]
    fn cse_merges_identical_subtrees() {
        let mut b = AstBuilder::new();
        let c1 = b.add(
            Op::Coupling,
            Sym::Coupling(CouplingId::from(5usize)),
            vec![],
        );
        let m1 = b.add(Op::Mul, Sym::None, vec![c1]);
        let c2 = b.add(
            Op::Coupling,
            Sym::Coupling(CouplingId::from(5usize)),
            vec![],
        );
        let m2 = b.add(Op::Mul, Sym::None, vec![c2]);
        let root = b.add(Op::Add, Sym::None, vec![m1, m2]);
        let ast = b.finish(root);
        let rendered = ast.to_string();

        let opt = optimize(ast);
        assert_eq!(opt.len(), 3, "coupling, mul, add");
        assert_eq!(opt.to_string(), rendered);
        let kids = opt.children_ids(opt.root());
        assert_eq!(kids[0], kids[1], "both Add children share one node");
    }

    /// Nodes differing in leaf value or child order must not merge.
    #[test]
    fn cse_keeps_distinct_nodes() {
        let mut b = AstBuilder::new();
        let a = b.add(Op::Coeff, Sym::Coeff(1.0), vec![]);
        let c = b.add(Op::Coeff, Sym::Coeff(2.0), vec![]);
        let m1 = b.add(Op::Mul, Sym::None, vec![a, c]);
        let m2 = b.add(Op::Mul, Sym::None, vec![c, a]);
        let root = b.add(Op::Add, Sym::None, vec![m1, m2]);
        let ast = b.finish(root);
        let rendered = ast.to_string();

        let opt = optimize(ast);
        assert_eq!(opt.len(), 5, "child order distinguishes the two Muls");
        assert_eq!(opt.to_string(), rendered);
    }
}
