//! Pass 3a: inline the pass-1+2 `DiagramEval`s into one unified [`Ast<Sym>`].
//!
//! Each diagram's `DiagramEvalTree` and the per-vertex `LorentzEvalTree`s are flattened
//! into a single arena over the whole amplitude:
//! - external legs → `External` (with a `Mass` child),
//! - propagators → `Propagate(current, Mass, Width)`,
//! - a vertex's `coupling · Σ_k coeff_k · structure_k` → `Mul`/`Add` over `Coupling`/
//!   `Coeff` leaves and the inlined Lorentz structure; chiral-pair FFV structures
//!   (`Gamma·ProjM` / `Gamma·ProjP` variants of one shape) fuse into a single `Ffv*`
//!   node with the per-chirality effective couplings as scalar operands,
//! - the Lorentz tree's `Leg(i)`/`P{leg}` → the (shared) lowered input child / `PMom`,
//! - per-diagram `symmetry_factor · fermi_sign` → a `Mul` with a `Coeff` leaf,
//! - the whole amplitude → one `Add` over the diagram roots.
//!
//! Input currents are lowered once and referenced by id, so a current feeding several
//! summed terms is shared (a DAG), not duplicated.

use std::collections::HashMap;

use indexmap::IndexMap;

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

/// A fusable chiral FFV site in one rooted structure tree: a single `ProjM`/`ProjP`
/// node adjacent to a `Gamma*` node. Two vertex terms whose trees are identical
/// except for the projector tag form a chiral pair and fuse into one `Ffv*` node.
struct ChiralSite {
    /// The `ProjM`/`ProjP` node index.
    proj: usize,
    /// The adjacent `Gamma*` node index.
    gamma: usize,
    /// `true` when the projector wraps the Gamma's output (`ProjX(GammaIout(..))`),
    /// `false` when it sits on the Gamma's fermion input (`GammaIout(.., ProjX(..))`).
    outer: bool,
    /// Normalized inner chirality: `true` = this term is the left-handed member.
    /// An outer projector flips (`ProjX ∘ slash = slash ∘ ProjX̄` — the Weyl slash
    /// maps chiral storage blocks crosswise, so the identity is exact).
    left: bool,
    /// The tree rendered with the projector tag replaced by a hole — the grouping
    /// key: equal keys ⇒ identical contraction shape modulo chirality.
    key: String,
}

/// Analyze one rooted structure tree for a fusable chiral FFV site.
fn chiral_gamma_site(tree: &LorentzEvalTree) -> Option<ChiralSite> {
    use LorentzEvalNode as L;
    let projs: Vec<usize> = tree
        .iter()
        .filter(|&n| matches!(tree.value(n), L::ProjM { .. } | L::ProjP { .. }))
        .collect();
    let [proj] = projs[..] else {
        return None;
    };
    let proj_is_left = matches!(tree.value(proj), L::ProjM { .. });

    // The fused kernels put the projected fermion at the second operand
    // (`GammaVout`'s `j`; the continuing fermion of `GammaIout`/`GammaOout`).
    // A projector on `GammaVout`'s `i` position stays generic.
    if tree
        .iter()
        .any(|n| matches!(tree.value(n), L::GammaVout { i, .. } if *i == proj))
    {
        return None;
    }
    let inner_parent = tree.iter().find(|&n| match tree.value(n) {
        L::GammaVout { j, .. } => *j == proj,
        L::GammaIout { j, .. } => *j == proj,
        L::GammaOout { i, .. } => *i == proj,
        _ => false,
    });
    let proj_child = tree.value(proj).children()[0];
    let (gamma, outer, left) = if let Some(g) = inner_parent {
        (g, false, proj_is_left)
    } else if matches!(
        tree.value(proj_child),
        L::GammaIout { .. } | L::GammaOout { .. }
    ) {
        (proj_child, true, !proj_is_left)
    } else {
        return None;
    };
    let key = render_hole(tree, tree.root(), proj);
    Some(ChiralSite {
        proj,
        gamma,
        outer,
        left,
        key,
    })
}

/// Canonical rendering of a rooted structure tree with the projector node at `proj`
/// replaced by a hole (its subtree still rendered), so a chiral pair's two trees
/// produce equal strings.
fn render_hole(lt: &LorentzEvalTree, n: usize, proj: usize) -> String {
    use LorentzEvalNode as L;
    let node = lt.value(n);
    let head = if n == proj {
        "Proj?".to_string()
    } else {
        match node {
            L::Leg(i) => format!("Leg{i}"),
            L::P { leg } => format!("P{leg}"),
            L::POut => "POut".to_string(),
            L::GammaVout { .. } => "GammaVout".to_string(),
            L::GammaIout { .. } => "GammaIout".to_string(),
            L::GammaOout { .. } => "GammaOout".to_string(),
            L::ProjM { .. } => "ProjM".to_string(),
            L::ProjP { .. } => "ProjP".to_string(),
            L::ProjMAmp { .. } => "ProjMAmp".to_string(),
            L::ProjPAmp { .. } => "ProjPAmp".to_string(),
            L::Metric { .. } => "Metric".to_string(),
            L::MetricNegI { .. } => "MetricNegI".to_string(),
            L::MetricVout { .. } => "MetricVout".to_string(),
            L::LowerVout { .. } => "LowerVout".to_string(),
            L::Mul { .. } => "Mul".to_string(),
            L::IdentityAmp { .. } => "IdentityAmp".to_string(),
        }
    };
    let kids = node
        .children()
        .iter()
        .map(|&c| render_hole(lt, c, proj))
        .collect::<Vec<_>>()
        .join(" ");
    if kids.is_empty() {
        head
    } else {
        format!("({head} {kids})")
    }
}

/// The per-chirality effective coupling of a fused group side:
/// `Σ_members coupling·coeff` as a scalar sub-graph.
fn chiral_coupling_sum(
    b: &mut AstBuilder<Sym>,
    info: &VertexInfo,
    members: &[(usize, usize)],
) -> NodeId {
    let parts = members
        .iter()
        .map(|&(ti, ri)| {
            let coupling = b.add(
                Op::Coupling,
                Sym::Coupling(info.terms[ti].coupling_id),
                vec![],
            );
            let coeff = b.add(
                Op::Coeff,
                Sym::Coeff(info.terms[ti].terms[ri].coeff),
                vec![],
            );
            b.add(Op::Mul, Sym::None, vec![coupling, coeff])
        })
        .collect();
    sum_or_single(b, parts)
}

/// Lower `Σ_terms coupling · (Σ_k coeff_k · structure_k)` against the gap-free input
/// currents `inputs` (vertex legs in order, output omitted).
///
/// Chiral-pair FFV structures (same contraction shape, `ProjM` vs `ProjP` tag) are
/// fused: each pair collapses to one `Ffv*` node carrying the per-chirality
/// effective couplings as scalar operands (`g_L·left + g_R·right` — distributivity
/// over the shared structure shape). Everything else lowers generically.
fn lower_vertex(info: &VertexInfo, inputs: &[NodeId], b: &mut AstBuilder<Sym>) -> NodeId {
    let sites: Vec<Vec<Option<ChiralSite>>> = info
        .terms
        .iter()
        .map(|vt| {
            vt.terms
                .iter()
                .map(|rt| chiral_gamma_site(&rt.tree))
                .collect()
        })
        .collect();

    let mut groups: IndexMap<&str, Vec<(usize, usize)>> = IndexMap::new();
    for (ti, row) in sites.iter().enumerate() {
        for (ri, site) in row.iter().enumerate() {
            if let Some(s) = site {
                groups.entry(s.key.as_str()).or_default().push((ti, ri));
            }
        }
    }

    let mut consumed: Vec<Vec<bool>> = sites.iter().map(|row| vec![false; row.len()]).collect();
    let mut fused_nodes = Vec::new();
    for members in groups.values() {
        let site_of = |&(ti, ri): &(usize, usize)| sites[ti][ri].as_ref().unwrap();
        let s0 = site_of(&members[0]);
        // A pair needs both chiralities, and (paranoia; equal keys imply it) the
        // same site anchoring in every member's tree.
        let fusable = members.iter().any(|m| site_of(m).left)
            && members.iter().any(|m| !site_of(m).left)
            && members.iter().all(|m| {
                let s = site_of(m);
                (s.proj, s.gamma, s.outer) == (s0.proj, s0.gamma, s0.outer)
            });
        if !fusable {
            continue;
        }
        let side = |left: bool| -> Vec<(usize, usize)> {
            members
                .iter()
                .copied()
                .filter(|m| site_of(m).left == left)
                .collect()
        };
        let gl = chiral_coupling_sum(b, info, &side(true));
        let gr = chiral_coupling_sum(b, info, &side(false));
        let (t0, r0) = members[0];
        let s0 = sites[t0][r0].as_ref().unwrap();
        let tree = &info.terms[t0].terms[r0].tree;
        let fuse = FuseCtx {
            proj: s0.proj,
            gamma: s0.gamma,
            outer: s0.outer,
            gl,
            gr,
        };
        fused_nodes.push(lower_lorentz(tree, tree.root(), inputs, b, Some(&fuse)));
        for &(ti, ri) in members {
            consumed[ti][ri] = true;
        }
    }

    let mut term_nodes = Vec::with_capacity(info.terms.len());
    for (ti, vt) in info.terms.iter().enumerate() {
        let mut structures = Vec::with_capacity(vt.terms.len());
        for (ri, rt) in vt.terms.iter().enumerate() {
            if consumed[ti][ri] {
                continue;
            }
            let coeff = b.add(Op::Coeff, Sym::Coeff(rt.coeff), vec![]);
            let structure = lower_lorentz(&rt.tree, rt.tree.root(), inputs, b, None);
            structures.push(b.add(Op::Mul, Sym::None, vec![coeff, structure]));
        }
        if structures.is_empty() {
            continue;
        }
        let coupling = b.add(Op::Coupling, Sym::Coupling(vt.coupling_id), vec![]);
        let summed = sum_or_single(b, structures);
        term_nodes.push(b.add(Op::Mul, Sym::None, vec![coupling, summed]));
    }
    term_nodes.extend(fused_nodes);
    sum_or_single(b, term_nodes)
}

/// A chiral-pair fusion order for one [`lower_lorentz`] walk: when the recursion
/// reaches the site anchor (the projector for an outer site, the Gamma otherwise),
/// it emits one fused `Ffv*` node — operands `[a, f, gl, gr]`, projector level
/// skipped — instead of the generic `Gamma`/`Proj` nodes.
struct FuseCtx {
    proj: usize,
    gamma: usize,
    outer: bool,
    gl: NodeId,
    gr: NodeId,
}

/// Inline one rooted `LorentzEvalTree`. `Leg(i)` resolves to the shared `inputs[i]`
/// node; every other node maps to a unified `Op`, recursing on the Lorentz tree's
/// child node indices. With a `fuse` order, the chiral site emits its fused node
/// (see [`FuseCtx`]).
fn lower_lorentz(
    lt: &LorentzEvalTree,
    n: usize,
    inputs: &[NodeId],
    b: &mut AstBuilder<Sym>,
    fuse: Option<&FuseCtx>,
) -> NodeId {
    use LorentzEvalNode as L;
    let rec = |child: usize, b: &mut AstBuilder<Sym>| lower_lorentz(lt, child, inputs, b, fuse);
    if let Some(f) = fuse {
        let anchor = if f.outer { f.proj } else { f.gamma };
        if n == anchor {
            // The projected fermion operand: for an inner site, the projector's
            // child (the projection folds into the fused kernel); for an outer
            // site, the Gamma's own fermion child.
            let unwrap_proj = |p: usize| lt.value(p).children()[0];
            let (op, a_idx, f_idx) = match (f.outer, lt.value(f.gamma)) {
                (false, &L::GammaVout { i, j }) => (Op::FfvVout, i, unwrap_proj(j)),
                (false, &L::GammaIout { mu, j }) => (Op::FfvIout, mu, unwrap_proj(j)),
                (false, &L::GammaOout { mu, i }) => (Op::FfvOout, mu, unwrap_proj(i)),
                (true, &L::GammaIout { mu, j }) => (Op::FfvIout, mu, j),
                (true, &L::GammaOout { mu, i }) => (Op::FfvOout, mu, i),
                (outer, other) => unreachable!("bad chiral site: outer={outer}, {other:?}"),
            };
            let a = rec(a_idx, b);
            let c = rec(f_idx, b);
            return b.add(op, Sym::None, vec![a, c, f.gl, f.gr]);
        }
    }
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
