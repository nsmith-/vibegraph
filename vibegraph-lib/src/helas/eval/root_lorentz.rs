//! Vertex dispatch: resolve LorentzTerm into rooted contraction trees at compile time.
//!
//! Each LorentzTerm is a tensor network, we root it with an output fixed by `result_leg_idx`.
//! The rooted descriptor (RootedNode) specifies the concrete primitive and its input/output
//! orientation so that eval walks resolved nodes

use crate::{
    helas::eval::tree::Tree,
    ufo::lorentz::{LorentzOp, LorentzTerm},
};

/// A single LorentzTerm, already rooted at the output leg and ready to eval.
#[derive(Clone, Debug)]
pub struct RootedTerm {
    /// Coefficient carried from the UFO LorentzTerm.coeff (real, since couplings are stored separately).
    pub coeff: f64,
    /// Resolved primitive with output fiber fixed.
    pub tree: LorentzEvalTree,
}

impl std::fmt::Display for RootedTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}*{}", self.coeff, self.tree)
    }
}

/// Errors from rooting a vertex's Lorentz structure into a contraction tree.
#[derive(Clone, Debug, thiserror::Error)]
pub enum RootLorentzError {
    /// The vertex structure contains unsupported ops (Sigma, Epsilon, C) or too many free indices.
    #[error("unsupported vertex: {0}")]
    UnsupportedVertex(String),
    /// The structure is syntactically invalid (e.g., mismatched indices).
    #[error("invalid structure: {0}")]
    InvalidStructure(String),
}

/// Descriptor for one term in the vertex lorentz structure, with dispatch info and resolved rooted node.
///
/// Index labels follow the convention:
/// - a,b for scalars
/// - i,j for fermions (i=row, j=col)
/// - mu, nu, ... for vectors
#[derive(Clone, Debug, PartialEq)]
pub enum LorentzEvalNode {
    /// Leg index (0-indexed)
    Leg(usize),
    /// 2-fermion in, vector out
    GammaVout { i: usize, j: usize },
    /// vector+in-flowing fermion in, in-flowing fermion out
    GammaIout { mu: usize, j: usize },
    /// vector+out-flowing fermion in, out-flowing fermion out
    GammaOout { mu: usize, i: usize },
    /// fermion left projection
    ProjM { i: usize },
    /// fermion right projection
    ProjP { i: usize },
    /// Left chiral amplitude
    ProjMAmp { i: usize, j: usize },
    /// Right chiral amplitude
    ProjPAmp { i: usize, j: usize },
    /// contract two vector indices → scalar (`g_{μν} V^μ W^ν`)
    Metric { mu: usize, nu: usize },
    /// metric with one free index → vector: the off-shell vector current of a
    /// `Metric(out, v)` structure (e.g. the VVS/HVV vertex rooted at a vector
    /// leg). Raises the output index on the partner vector `v`; cf. ALOHA
    /// `VVS1P1N_1`. Output type: vector.
    MetricVout { v: usize },
    /// Handle the implicit product over the disconnected structures.
    /// At most one child can be non-scalar (which then implies the output type)
    Mul { children: Vec<usize> },
    /// 4-momentum of leg `leg` (0-indexed) as a vector at a free Lorentz index
    P { leg: usize },
    /// Full scalar bilinear ψ̄_i δ ψ_j (Identity amplitude contraction)
    IdentityAmp { i: usize, j: usize },
    // TODO: Sigma, Epsilon
}

impl LorentzEvalNode {
    pub fn children(&self) -> Vec<usize> {
        match self {
            LorentzEvalNode::Leg(_) => vec![],
            LorentzEvalNode::GammaVout { i, j } => vec![*i, *j],
            LorentzEvalNode::GammaIout { mu, j } => vec![*mu, *j],
            LorentzEvalNode::GammaOout { mu, i } => vec![*mu, *i],
            LorentzEvalNode::ProjM { i } | LorentzEvalNode::ProjP { i } => vec![*i],
            LorentzEvalNode::ProjMAmp { i, j } | LorentzEvalNode::ProjPAmp { i, j } => vec![*i, *j],
            LorentzEvalNode::Metric { mu, nu } => vec![*mu, *nu],
            LorentzEvalNode::MetricVout { v } => vec![*v],
            LorentzEvalNode::Mul { children } => children.clone(),
            LorentzEvalNode::P { .. } => vec![],
            LorentzEvalNode::IdentityAmp { i, j } => vec![*i, *j],
        }
    }

    fn render(&self, body: String) -> String {
        use LorentzEvalNode::*;
        match self {
            Leg(i) => format!("Leg({})", i), // leaf node
            GammaVout { .. } => format!("GammaVout({})", body),
            GammaIout { .. } => format!("GammaIout({})", body),
            GammaOout { .. } => format!("GammaOout({})", body),
            ProjM { .. } => format!("ProjM({})", body),
            ProjP { .. } => format!("ProjP({})", body),
            ProjMAmp { .. } => format!("ProjMAmp({})", body),
            ProjPAmp { .. } => format!("ProjPAmp({})", body),
            Metric { .. } => format!("Metric({})", body),
            MetricVout { .. } => format!("MetricVout({})", body),
            Mul { .. } => format!("ScalarProduct({})", body),
            P { .. } => format!("P({})", body),
            IdentityAmp { .. } => format!("IdentityAmp({})", body),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LorentzEvalTree {
    nodes: Vec<LorentzEvalNode>,
    root: Option<usize>,
}

impl Tree for LorentzEvalTree {
    type Item = LorentzEvalNode;
    type NodeId = usize;

    fn children(&self, node: Self::NodeId) -> impl Iterator<Item = usize> {
        self.value(node).children().into_iter()
    }

    fn value(&self, node: Self::NodeId) -> &Self::Item {
        &self.nodes[node]
    }

    fn root(&self) -> Self::NodeId {
        self.root.expect("LorentzEvalTree has no root node")
    }

    fn iter(&self) -> impl Iterator<Item = Self::NodeId> {
        0..self.nodes.len()
    }
}

impl LorentzEvalTree {
    /// The value at the root node. Used by tests to assert the rooted primitive.
    #[cfg(test)]
    pub fn root_value(&self) -> &LorentzEvalNode {
        self.value(self.root())
    }

    fn add_node(&mut self, node: LorentzEvalNode) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        idx
    }

    fn build_child(
        &mut self,
        term: &LorentzTerm,
        idx: isize,
        visited_ops: &mut Vec<usize>,
    ) -> Result<usize, RootLorentzError> {
        // Find an operator that involves this index and has not been visited
        let Some((iop, op)) = term.ops.iter().enumerate().find(|&(i, op)| {
            (op.involves_spinor(idx) || op.involves_vector(idx))
                && visited_ops.iter().all(|&j| i != j)
        }) else {
            // No operator involves this index
            if idx >= 0 {
                // This is a scalar leaf node
                return Ok(self.add_node(LorentzEvalNode::Leg(idx as usize)));
            } else {
                return Err(RootLorentzError::InvalidStructure(format!(
                    "Free index {} has no operator in term",
                    idx
                )));
            }
        };

        visited_ops.push(iop);

        match op {
            LorentzOp::Gamma { mu, i, j } => {
                if *mu == idx {
                    let child_i = self.build_child(term, *i, visited_ops)?;
                    let child_j = self.build_child(term, *j, visited_ops)?;
                    Ok(self.add_node(LorentzEvalNode::GammaVout {
                        i: child_i,
                        j: child_j,
                    }))
                } else if *i == idx {
                    let child_mu = self.build_child(term, *mu, visited_ops)?;
                    let child_j = self.build_child(term, *j, visited_ops)?;
                    Ok(self.add_node(LorentzEvalNode::GammaIout {
                        mu: child_mu,
                        j: child_j,
                    }))
                } else if *j == idx {
                    let child_mu = self.build_child(term, *mu, visited_ops)?;
                    let child_i = self.build_child(term, *i, visited_ops)?;
                    Ok(self.add_node(LorentzEvalNode::GammaOout {
                        mu: child_mu,
                        i: child_i,
                    }))
                } else {
                    unreachable!("Gamma op should involve idx {}", idx);
                }
            }
            LorentzOp::ProjM { i, j } => {
                if *i == idx {
                    let child = self.build_child(term, *j, visited_ops)?;
                    Ok(self.add_node(LorentzEvalNode::ProjM { i: child }))
                } else if *j == idx {
                    let child = self.build_child(term, *i, visited_ops)?;
                    Ok(self.add_node(LorentzEvalNode::ProjM { i: child }))
                } else {
                    unreachable!("ProjM op should involve idx {}", idx);
                }
            }
            LorentzOp::ProjP { i, j } => {
                if *i == idx {
                    let child = self.build_child(term, *j, visited_ops)?;
                    Ok(self.add_node(LorentzEvalNode::ProjP { i: child }))
                } else if *j == idx {
                    let child = self.build_child(term, *i, visited_ops)?;
                    Ok(self.add_node(LorentzEvalNode::ProjP { i: child }))
                } else {
                    unreachable!("ProjP op should involve idx {}", idx);
                }
            }
            LorentzOp::Metric { mu, nu } => {
                if *mu == idx {
                    let child = self.build_child(term, *nu, visited_ops)?;
                    Ok(self.add_node(LorentzEvalNode::MetricVout { v: child }))
                } else if *nu == idx {
                    let child = self.build_child(term, *mu, visited_ops)?;
                    Ok(self.add_node(LorentzEvalNode::MetricVout { v: child }))
                } else {
                    unreachable!("Metric op should involve idx {}", idx);
                }
            }
            LorentzOp::P { mu, leg } => {
                // mu == idx (guaranteed by involves_vector fix); leg is the momentum source particle
                assert_eq!(*mu, idx, "Momentum index mismatch");
                Ok(self.add_node(LorentzEvalNode::P { leg: *leg as usize }))
            }
            LorentzOp::Identity { i, j } => {
                if *i == idx {
                    self.build_child(term, *j, visited_ops)
                } else if *j == idx {
                    self.build_child(term, *i, visited_ops)
                } else {
                    unreachable!("Identity op should involve idx {}", idx);
                }
            }
            LorentzOp::Sigma { .. } => Err(RootLorentzError::UnsupportedVertex(
                "Sigma tensors are deferred to future work".to_string(),
            )),
            LorentzOp::Epsilon { .. } => Err(RootLorentzError::UnsupportedVertex(
                "Epsilon tensors are deferred to future work".to_string(),
            )),
            LorentzOp::C { .. } => Err(RootLorentzError::UnsupportedVertex(
                "Charge conjugation is deferred to future work".to_string(),
            )),
        }
    }

    /// Build a LorentzEvalTree rooted at the specified leg index.
    ///
    /// This turns the undirected tensor network of the LorentzTerm into a
    /// directed tree with a known output. If the leg index is None, the
    /// tree is rooted at an amplitude (scalar sink) and an arbitrary leg
    /// is chosen for routing.
    ///
    /// Note: idx is 0-indexed
    pub fn build_at_leg(
        term: &LorentzTerm,
        spins: &[i32],
        idx: Option<usize>,
    ) -> Result<Self, RootLorentzError> {
        let mut tree = LorentzEvalTree {
            nodes: vec![],
            root: None,
        };
        let mut visited_ops = Vec::new(); // LorentzOp is so small that Vec is probably better than HashSet
        let mut term_roots = Vec::new();

        // If idx is specified, build the tree rooted at that leg
        if let Some(root_leg) = idx {
            let node_idx = tree.build_child(term, root_leg as isize, &mut visited_ops)?;
            // If no operator connects to this leg, build_child returns a trivial Leg leaf.
            // Pop it — the disconnected structures collected below are the actual output.
            let is_trivial_leaf =
                matches!(tree.nodes[node_idx], LorentzEvalNode::Leg(i) if i == root_leg);
            if is_trivial_leaf {
                tree.nodes.pop();
            } else {
                term_roots.push(node_idx);
            }
        }

        // find all the remaining scalar roots
        while visited_ops.len() < term.ops.len() {
            // find an unvisited op
            let Some((iop, op)) = term
                .ops
                .iter()
                .enumerate()
                .find(|&(i, _)| visited_ops.iter().all(|&j| i != j))
            else {
                break; // no more unvisited ops
            };
            // pick a leg index from this op that we know how to contract and return a scalar
            let term = match op {
                LorentzOp::Gamma { mu, .. } => {
                    // route through a vector leg, which can always be contracted with a metric to return a scalar
                    // two-pass: one for the gamma and one for the leg
                    let v_in = tree.build_child(term, *mu, &mut visited_ops)?;
                    let v_out = tree.build_child(term, *mu, &mut visited_ops)?;
                    tree.add_node(LorentzEvalNode::Metric {
                        mu: v_in,
                        nu: v_out,
                    })
                }
                LorentzOp::ProjM { i, j } => {
                    visited_ops.push(iop);
                    let child_i = tree.build_child(term, *i, &mut visited_ops)?;
                    let child_j = tree.build_child(term, *j, &mut visited_ops)?;
                    tree.add_node(LorentzEvalNode::ProjMAmp {
                        i: child_i,
                        j: child_j,
                    })
                }
                LorentzOp::ProjP { i, j } => {
                    visited_ops.push(iop);
                    let child_i = tree.build_child(term, *i, &mut visited_ops)?;
                    let child_j = tree.build_child(term, *j, &mut visited_ops)?;
                    tree.add_node(LorentzEvalNode::ProjPAmp {
                        i: child_i,
                        j: child_j,
                    })
                }
                LorentzOp::Metric { mu, nu } => {
                    visited_ops.push(iop);
                    let child_mu = tree.build_child(term, *mu, &mut visited_ops)?;
                    let child_nu = tree.build_child(term, *nu, &mut visited_ops)?;
                    tree.add_node(LorentzEvalNode::Metric {
                        mu: child_mu,
                        nu: child_nu,
                    })
                }
                LorentzOp::P { mu, .. } => {
                    // p^μ contracted with the vector leg at the same index
                    let p_node = tree.build_child(term, *mu, &mut visited_ops)?;
                    let leg_node = tree.build_child(term, *mu, &mut visited_ops)?;
                    tree.add_node(LorentzEvalNode::Metric {
                        mu: p_node,
                        nu: leg_node,
                    })
                }
                LorentzOp::Identity { i, j } => {
                    visited_ops.push(iop);
                    let child_i = tree.build_child(term, *i, &mut visited_ops)?;
                    let child_j = tree.build_child(term, *j, &mut visited_ops)?;
                    tree.add_node(LorentzEvalNode::IdentityAmp {
                        i: child_i,
                        j: child_j,
                    })
                }
                _ => {
                    todo!(
                        "Routing for remaining ops not yet implemented in tree builder: {:?}",
                        op
                    );
                }
            };
            term_roots.push(term);
        }

        // Find any scalar legs not connected to any operator and add them as scalar roots.
        // Skip the output leg (excluded_leg) — it is not an input for off-shell currents.
        for (ileg, spin) in spins.iter().enumerate() {
            if *spin == 1 {
                if Some(ileg) == idx {
                    continue; // output leg — not an input
                }
                if !tree
                    .nodes
                    .iter()
                    .any(|node| matches!(node, LorentzEvalNode::Leg(i) if *i == ileg))
                {
                    term_roots.push(tree.add_node(LorentzEvalNode::Leg(ileg)));
                }
            }
        }

        // take scalar product
        let root = if term_roots.len() == 1 {
            term_roots[0]
        } else {
            tree.add_node(LorentzEvalNode::Mul {
                children: term_roots,
            })
        };

        tree.root = Some(root);

        // The output leg is never referenced by an off-shell current, so its
        // position is a hole in the input-leg numbering. Compact the leg references
        // by dropping that hole: every input leg above `out` shifts down by one, so
        // `Leg(i)`/`P{leg}` index directly into the caller's gap-free input list
        // (vertex legs in order, output omitted) with no per-eval reindexing.
        if let Some(out) = idx {
            for node in &mut tree.nodes {
                match node {
                    LorentzEvalNode::Leg(i) if *i > out => *i -= 1,
                    LorentzEvalNode::P { leg } if *leg > out => *leg -= 1,
                    _ => {}
                }
            }
        }

        Ok(tree)
    }

    fn render_expression(&self) -> String {
        self.fold_recursive(
            &|node, acc| node.render(acc),
            &|acc, r| if acc.is_empty() { r } else { acc + "," + &r },
            String::new(),
            self.root.expect("should have a root node"),
        )
    }
}

impl std::fmt::Display for LorentzEvalTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.render_expression())
    }
}

/// Resolve a single LorentzTerm into a rooted primitive with the output leg fixed.
///
/// # Arguments
/// * `term` — The UFO LorentzTerm to resolve.
/// * `spins` — Spin codes [1, 2, 3] for each leg
/// * `result_leg_idx` — The output leg (0-indexed), or `None` for amplitude (scalar sink).
///
/// # Returns
/// A `RootedTerm` ready for evaluation, or a `RootLorentzError`.
pub fn root_term(
    term: &crate::ufo::lorentz::LorentzTerm,
    spins: &[i32],
    result_leg_idx: Option<usize>,
) -> Result<RootedTerm, RootLorentzError> {
    Ok(RootedTerm {
        coeff: term.coeff,
        tree: LorentzEvalTree::build_at_leg(term, spins, result_leg_idx)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ufo::lorentz::LorentzTerm;

    #[test]
    fn test_vvs_rooted_at_vector_leg_is_a_vector_current() {
        // VVS1: Metric(1,2), spins [3,3,1] (Z, Z, H). Rooting at a *vector* leg
        // (the off-shell Z current of an HZZ vertex) must raise that index via
        // the metric: the current is the OTHER vector leg × the scalar leg — a
        // vector, NOT a scalar contraction. It must never reference its own output
        // leg as an input; since `build_at_leg` compacts leg references over the
        // removed output, that means every `Leg(i)` must land in the gap-free input
        // range `0..n_legs-1`. Regression for the VVS Leg/placeholder crash.
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::Metric { mu: 0, nu: 1 }],
        };
        let spins = vec![3, 3, 1];
        let n_inputs = spins.len() - 1; // gap-free input legs (output removed)

        let max_leg = |t: &LorentzEvalTree| {
            t.nodes
                .iter()
                .filter_map(|n| match n {
                    LorentzEvalNode::Leg(i) => Some(*i),
                    _ => None,
                })
                .max()
        };

        // Output = vector leg 0: current = (other vector) × (scalar), both compacted
        // into 0..n_inputs.
        let t0 = LorentzEvalTree::build_at_leg(&term, &spins, Some(0)).unwrap();
        assert!(
            max_leg(&t0).is_some_and(|m| m < n_inputs),
            "VVS rooted at leg 0 must only index the gap-free inputs: {t0:?}"
        );
        assert!(
            matches!(t0.root_value(), LorentzEvalNode::Mul { .. }),
            "VVS rooted at a vector leg must yield a vector (scalar×vector), got {:?}",
            t0.root_value()
        );

        // Output = vector leg 2 (idx 1): same invariant.
        let t1 = LorentzEvalTree::build_at_leg(&term, &spins, Some(1)).unwrap();
        assert!(
            max_leg(&t1).is_some_and(|m| m < n_inputs),
            "VVS rooted at leg 1 must only index the gap-free inputs: {t1:?}"
        );

        // Output = scalar leg 3 (idx 2): unchanged — a Metric contraction → scalar H.
        let t2 = LorentzEvalTree::build_at_leg(&term, &spins, Some(2)).unwrap();
        assert!(
            matches!(t2.root_value(), LorentzEvalNode::Metric { .. }),
            "VVS rooted at the scalar leg must contract the two vectors: {:?}",
            t2.root_value()
        );
    }

    #[test]
    fn test_root_ffv1_photon_current_at_leg3() {
        // FFV1: Gamma(3,2,1) rooted at vector leg 3 → SpinorCurrent{Both,row=2,col=1}
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::Gamma { mu: 2, i: 1, j: 0 }],
        };
        let spins = vec![2, 2, 3]; // e+, e-, photon
        let result = root_term(&term, &spins, Some(2)).unwrap();
        assert_eq!(result.coeff, 1.0);
        assert_eq!(
            result.tree,
            LorentzEvalTree {
                nodes: vec![
                    LorentzEvalNode::Leg(1),
                    LorentzEvalNode::Leg(0),
                    LorentzEvalNode::GammaVout { i: 0, j: 1 },
                ],
                root: Some(2)
            }
        );
    }

    #[test]
    fn test_root_ffv1_amplitude_at_sink() {
        // FFV1 rooted at amplitude (scalar sink) → ScalarProduct of the vector output and itself
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::Gamma { mu: 2, i: 1, j: 0 }],
        };
        let spins = vec![2, 2, 3];
        let result = root_term(&term, &spins, None).unwrap();
        assert_eq!(result.coeff, 1.0);
        // When rooted at amplitude, builds the Gamma structure and contracts with a vector leg
        assert_eq!(
            result.tree,
            LorentzEvalTree {
                nodes: vec![
                    LorentzEvalNode::Leg(1),
                    LorentzEvalNode::Leg(0),
                    LorentzEvalNode::GammaVout { i: 0, j: 1 },
                    LorentzEvalNode::Leg(2),
                    LorentzEvalNode::Metric { mu: 2, nu: 3 },
                ],
                root: Some(4)
            }
        )
    }

    #[test]
    fn test_root_ffv2_left_chiral() {
        // FFV2: Gamma(3,2,-1)*ProjM(-1,1) → left-chiral fermion rooted at amplitude
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![
                LorentzOp::Gamma { mu: 2, i: 1, j: -1 },
                LorentzOp::ProjM { i: -1, j: 0 },
            ],
        };
        let spins = vec![2, 2, 3];
        let result = root_term(&term, &spins, None).unwrap();
        assert_eq!(result.coeff, 1.0);
        // The tree routes through ProjM → Gamma
        assert_eq!(
            result.tree,
            LorentzEvalTree {
                nodes: vec![
                    LorentzEvalNode::Leg(1),
                    LorentzEvalNode::Leg(0),
                    LorentzEvalNode::ProjM { i: 1 },
                    LorentzEvalNode::GammaVout { i: 0, j: 2 },
                    LorentzEvalNode::Leg(2),
                    LorentzEvalNode::Metric { mu: 3, nu: 4 },
                ],
                root: Some(5)
            }
        );
    }

    #[test]
    fn test_root_ffs_yukawa() {
        // FFS1: ProjM(2,1) rooted at amplitude → ScalarProduct (bilinear)
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::ProjM { i: 1, j: 0 }],
        };
        let spins = vec![2, 2, 1];
        let result = root_term(&term, &spins, None).unwrap();
        assert_eq!(
            result.tree,
            LorentzEvalTree {
                nodes: vec![
                    LorentzEvalNode::Leg(1),
                    LorentzEvalNode::Leg(0),
                    LorentzEvalNode::ProjMAmp { i: 0, j: 1 },
                    LorentzEvalNode::Leg(2),
                    LorentzEvalNode::Mul {
                        children: vec![2, 3]
                    },
                ],
                root: Some(4)
            }
        )
    }

    #[test]
    fn test_root_ffs_off_shell_scalar() {
        // FFS1: ProjM(2,1) rooted at scalar leg 3 → just the bilinear, no Leg(2)
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::ProjM { i: 1, j: 0 }],
        };
        let spins = vec![2, 2, 1];
        let result = root_term(&term, &spins, Some(2)).unwrap();
        assert_eq!(
            result.tree,
            LorentzEvalTree {
                nodes: vec![
                    LorentzEvalNode::Leg(1),
                    LorentzEvalNode::Leg(0),
                    LorentzEvalNode::ProjMAmp { i: 0, j: 1 },
                ],
                root: Some(2)
            }
        )
    }

    #[test]
    fn test_root_sigma_unsupported() {
        // Sigma should trigger UnsupportedVertex when it's on the path to the root.
        // Root at leg 1 (fermion) where Sigma(1,2,2,1) is present
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::Sigma {
                mu: 0,
                nu: 1,
                i: 1,
                j: 0,
            }],
        };
        let spins = vec![2, 2, 3];
        let result = root_term(&term, &spins, Some(0)); // root at leg 1 (0-indexed as 0)
        assert!(matches!(
            result,
            Err(RootLorentzError::UnsupportedVertex(_))
        ));
    }

    #[test]
    fn test_root_vvs_metric() {
        // VVS1: Metric(1,2) rooted at amplitude → Metric contraction
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::Metric { mu: 0, nu: 1 }],
        };
        let spins = vec![3, 3, 1];
        let result = root_term(&term, &spins, None).unwrap();
        assert_eq!(result.coeff, 1.0);
        // When rooted at amplitude with 2 vector legs, uses Metric to contract them
        assert_eq!(
            result.tree,
            LorentzEvalTree {
                nodes: vec![
                    LorentzEvalNode::Leg(0),
                    LorentzEvalNode::Leg(1),
                    LorentzEvalNode::Metric { mu: 0, nu: 1 },
                    LorentzEvalNode::Leg(2),
                    LorentzEvalNode::Mul {
                        children: vec![2, 3]
                    },
                ],
                root: Some(4)
            }
        )
    }

    #[test]
    fn test_root_sss_scalar() {
        // SSS1: Empty ops (all scalars) rooted at amplitude → ScalarProduct
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![],
        };
        let spins = vec![1, 1, 1];
        let result = root_term(&term, &spins, None).unwrap();
        assert_eq!(
            result.tree,
            LorentzEvalTree {
                nodes: vec![
                    LorentzEvalNode::Leg(0),
                    LorentzEvalNode::Leg(1),
                    LorentzEvalNode::Leg(2),
                    LorentzEvalNode::Mul {
                        children: vec![0, 1, 2]
                    },
                ],
                root: Some(3)
            }
        )
    }
}
