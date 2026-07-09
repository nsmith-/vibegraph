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

use super::ast::{Ast, AstBuilder};
use super::diagram_eval::VertexInfo;
use super::op::{NodeId, Op, Sym};
use super::root_diagram::{DiagramEval, DiagramEvalTree, EvalNode, EvalNodeId};
use super::root_lorentz::{LorentzEvalNode, LorentzEvalTree};
use super::tree::Tree;

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

/// Structure-optimization pass (egglog hook). No-op for now.
pub fn optimize(ast: Ast<Sym>) -> Ast<Sym> {
    ast
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
