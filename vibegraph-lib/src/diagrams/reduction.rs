//! MadGraph's coupling-order budget, spent vertex by vertex in the order its
//! diagram generation reduces a tree.
//!
//! `Amplitude.reduce_leglist` (`diagram_generation.py:958-1046`) builds a diagram
//! from its external legs inwards, a step at a time, and `reduce_orders`
//! (`:1048-1083`) subtracts each new vertex's coupling orders, and its WEIGHTED,
//! from what the process allows, dropping the diagram the moment any bound goes
//! below zero. With non-negative orders the running sum never exceeds the total,
//! so that is the same as bounding the diagram's total. A negative power (the
//! `QED: -1` of SMEFTsim's vev insertions) lowers the running sum, and a diagram
//! whose total is within the bound is dropped when the vertex carrying it comes
//! after a prefix that is not.
//!
//! The order of reduction is a property of the tree:
//!
//! * a combination needs at least one leg made in the step before
//!   (`LegList.can_combine_to_1`, `base_objects.py:2250-2256`), and a leg left
//!   uncombined loses that mark (`merge_comb_legs`, `:1224-1231`), so each vertex is
//!   made at the step one past the latest of its inputs: its height above the
//!   external legs;
//! * the last vertex joins what is left and needs two legs made in the step before
//!   (`can_combine_to_0`, `base_objects.py:2258-2279`), or is the identity joining
//!   the last two (`ParticleList.generate_ref_dict`, `:579-589`), which carries no
//!   orders. So the root is the tree's centre: the vertex whose two highest branches
//!   are equally high, or the propagator whose two sides are. A decay's root is its
//!   initial leg, which only the closing identity may join (`:2267-2274`);
//! * within a step, the vertices are checked in the order of the legs they replace,
//!   which stay sorted by the smallest external leg number below them
//!   (`combine_legs`, `diagram_generation.py:1114-1153`; `merge_comb_legs`, `:1195`).

use std::collections::BTreeMap;

use super::diagram::{Diagram, Ray};
use crate::ufo::UFOModel;

/// A neighbour of a vertex: an external leg, or another vertex through a propagator.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Node {
    Leg(usize),
    Vertex(usize),
}

/// What the generation can see of a tree.
struct Tree {
    neighbours: Vec<Vec<Node>>,
    /// The vertex each external leg attaches to.
    leg_vertex: Vec<usize>,
}

impl Tree {
    fn of(diagram: &Diagram) -> Self {
        let mut neighbours = vec![Vec::new(); diagram.vertices.len()];
        let mut leg_vertex = vec![usize::MAX; diagram.legs.len()];
        for (v, vertex) in diagram.vertices.iter().enumerate() {
            for ray in &vertex.rays {
                match *ray {
                    Ray::Leg(leg) => {
                        neighbours[v].push(Node::Leg(leg.0));
                        leg_vertex[leg.0] = v;
                    }
                    Ray::Prop { prop, end } => {
                        let other = diagram.props[prop.0].endpoints[1 - end].0 .0;
                        neighbours[v].push(Node::Vertex(other));
                    }
                }
            }
        }
        Tree {
            neighbours,
            leg_vertex,
        }
    }

    /// The height of the branch at `to` seen from vertex `from`: 0 for an external
    /// leg, else one more than the highest branch beyond it.
    fn height(&self, from: usize, to: Node) -> usize {
        match to {
            Node::Leg(_) => 0,
            Node::Vertex(u) => {
                1 + self.neighbours[u]
                    .iter()
                    .filter(|&&n| n != Node::Vertex(from))
                    .map(|&n| self.height(u, n))
                    .max()
                    .unwrap_or(0)
            }
        }
    }

    /// The smallest external leg in the branch at `to` seen from `from`.
    fn min_leg(&self, from: usize, to: Node) -> usize {
        match to {
            Node::Leg(l) => l,
            Node::Vertex(u) => self.neighbours[u]
                .iter()
                .filter(|&&n| n != Node::Vertex(from))
                .map(|&n| self.min_leg(u, n))
                .min()
                .unwrap_or(usize::MAX),
        }
    }

    /// Each vertex with the vertex its output leg goes to, `None` for the root vertex,
    /// as MadGraph roots the tree.
    fn parents(&self, decay: bool) -> Option<Vec<Option<usize>>> {
        let n = self.neighbours.len();
        let root: (usize, Option<usize>) = if decay {
            // The initial leg's vertex makes the leg the closing identity joins.
            (*self.leg_vertex.first()?, Some(usize::MAX))
        } else {
            let mut found = None;
            for v in 0..n {
                let mut h: Vec<usize> = self.neighbours[v]
                    .iter()
                    .map(|&nb| self.height(v, nb))
                    .collect();
                h.sort_unstable_by(|a, b| b.cmp(a));
                if h.len() >= 2 && h[0] == h[1] {
                    found = Some((v, None));
                    break;
                }
                for &nb in &self.neighbours[v] {
                    if let Node::Vertex(u) = nb {
                        if v < u && self.height(v, nb) == self.height(u, Node::Vertex(v)) {
                            found = Some((v, Some(u)));
                        }
                    }
                }
                if found.is_some() {
                    break;
                }
            }
            found?
        };
        let mut parent = vec![None; n];
        let mut stack: Vec<(usize, Option<usize>)> = vec![(root.0, root.1)];
        if let Some(u) = root.1.filter(|&u| u != usize::MAX) {
            stack.push((u, Some(root.0)));
        }
        while let Some((v, p)) = stack.pop() {
            parent[v] = p;
            for &nb in &self.neighbours[v] {
                if let Node::Vertex(u) = nb {
                    if Some(u) != p {
                        stack.push((u, Some(v)));
                    }
                }
            }
        }
        Some(
            parent
                .into_iter()
                .map(|p| p.filter(|&p| p != usize::MAX))
                .collect(),
        )
    }
}

/// The diagram's vertices in the order MadGraph's generation adds them, or `None`
/// for a graph that is not a tree it can root.
fn reduction_order(diagram: &Diagram) -> Option<Vec<usize>> {
    let tree = Tree::of(diagram);
    let parents = tree.parents(diagram.n_in == 1)?;
    let mut keyed: Vec<(usize, usize, usize)> = Vec::with_capacity(parents.len());
    let mut root = None;
    for (v, parent) in parents.iter().enumerate() {
        match *parent {
            Some(p) => keyed.push((
                tree.height(p, Node::Vertex(v)),
                tree.min_leg(p, Node::Vertex(v)),
                v,
            )),
            // A decay's initial-leg vertex has no parent vertex either: it is made
            // last, at its height, ahead of the closing identity.
            None if diagram.n_in == 1 => keyed.push((usize::MAX, 0, v)),
            None => root = Some(v),
        }
    }
    keyed.sort_unstable();
    let mut order: Vec<usize> = keyed.into_iter().map(|(_, _, v)| v).collect();
    order.extend(root);
    Some(order)
}

/// Whether MadGraph's `reduce_orders` keeps `diagram` under `bounds` (order name to
/// its `<=` bound; a negative bound bounds nothing) and an optional WEIGHTED bound:
/// every running sum, in [`reduction_order`], stays within its bound.
pub(crate) fn within_running_budget(
    diagram: &Diagram,
    model: &UFOModel,
    bounds: &BTreeMap<String, i64>,
    max_weighted: Option<i64>,
) -> bool {
    let Some(order) = reduction_order(diagram) else {
        return true;
    };
    let mut spent: BTreeMap<&str, i64> = BTreeMap::new();
    let mut weighted = 0i64;
    for v in order {
        let def = model.vertex_def(diagram.vertices[v].interaction);
        let Some(&coupling) = def.couplings.values().next() else {
            continue;
        };
        for (name, &n) in &model.coupling_def(coupling).orders {
            let total = spent.entry(name.as_str()).or_default();
            *total += n;
            if bounds.get(name).is_some_and(|&b| b >= 0 && *total > b) {
                return false;
            }
            weighted += n * i64::from(model.order_hierarchy.get(name).copied().unwrap_or(0));
        }
        if max_weighted.is_some_and(|b| b >= 0 && weighted > b) {
            return false;
        }
    }
    true
}
