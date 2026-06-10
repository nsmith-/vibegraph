//! Vertex dispatch: resolve LorentzExpr + spin codes into rooted contraction trees at compile time.
//!
//! Each LorentzTerm is decomposed into a tensor network with an output fiber fixed by `result_leg_idx`.
//! The rooted descriptor (RootedNode) specifies the concrete primitive and its input/output orientation
//! so that eval walks resolved nodes, never symbolic ops.

pub use crate::helas::repr::lorentz::Chirality;
use crate::ufo::lorentz::{LorentzOp, LorentzTerm};

/// A single LorentzTerm, already rooted at the output leg and ready to eval.
#[derive(Clone, Debug)]
pub struct RootedTerm {
    /// Coefficient carried from the UFO LorentzTerm.coeff (real, since couplings are stored separately).
    pub coeff: f64,
    /// Resolved primitive with output fiber fixed.
    pub tree: LorentzEvalTree,
}

/// Compile error types for unsupported vertices or invalid structures.
#[derive(Clone, Debug)]
pub enum CompileError {
    /// The vertex structure contains unsupported ops (Sigma, Epsilon, C) or too many free indices.
    UnsupportedVertex(String),
    /// The structure is syntactically invalid (e.g., mismatched indices).
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
    /// Leg index according to LorentzOp conventions (1-indexed)
    Leg(i32),
    /// 2-fermion in, vector out
    GammaVout { i: usize, j: usize },
    /// vector+fermion in, fermion out
    GammaIout { mu: usize, j: usize },
    /// vector+fermion in, fermion out (TODO: is this distinct from Gamma_Iout or can it be unified with a flag?)
    GammaJout { mu: usize, i: usize },
    /// fermion left projection
    ProjM { i: usize },
    /// fermion right projection
    ProjP { i: usize },
    /// Left chiral amplitude
    ProjMAmp { i: usize, j: usize },
    /// Right chiral amplitude
    ProjPAmp { i: usize, j: usize },
    /// contract two vector indices
    Metric { mu: usize, nu: usize },
    /// Handle the implicit product over the disconnected structures.
    /// At most one child can be non-scalar (which then implies the output type)
    ScalarProduct { children: Vec<usize> },
    // TODO: rest of LorentzOps (Identity, P, Sigma, Epsilon)
}

impl LorentzEvalNode {
    pub fn children(&self) -> Vec<usize> {
        match self {
            LorentzEvalNode::Leg(_) => vec![],
            LorentzEvalNode::GammaVout { i, j } => vec![*i, *j],
            LorentzEvalNode::GammaIout { mu, j } => vec![*mu, *j],
            LorentzEvalNode::GammaJout { mu, i } => vec![*mu, *i],
            LorentzEvalNode::ProjM { i } | LorentzEvalNode::ProjP { i } => vec![*i],
            LorentzEvalNode::ProjMAmp { i, j } | LorentzEvalNode::ProjPAmp { i, j } => vec![*i, *j],
            LorentzEvalNode::Metric { mu, nu } => vec![*mu, *nu],
            LorentzEvalNode::ScalarProduct { children } => children.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LorentzEvalTree {
    nodes: Vec<LorentzEvalNode>,
    root: Option<usize>,
}

impl LorentzEvalTree {
    fn add_node(&mut self, node: LorentzEvalNode) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        idx
    }

    fn build_child(
        &mut self,
        term: &LorentzTerm,
        idx: i32,
        visited_ops: &mut Vec<usize>,
    ) -> Result<usize, CompileError> {
        // Find an operator that involves this index and has not been visited
        let Some((iop, op)) = term.ops.iter().enumerate().find(|&(i, op)| {
            (op.involves_spinor(idx) || op.involves_vector(idx))
                && visited_ops.iter().all(|&j| i != j)
        }) else {
            // No operator involves this index
            if idx > 0 {
                // This is a scalar leaf node
                return Ok(self.add_node(LorentzEvalNode::Leg(idx)));
            } else {
                return Err(CompileError::InvalidStructure(format!(
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
                    Ok(self.add_node(LorentzEvalNode::GammaJout {
                        mu: child_mu,
                        i: child_i,
                    }))
                } else {
                    unreachable!("Gamma op should involve idx {}", idx);
                }
            }
            LorentzOp::ProjM { i, j } => {
                if *i > 0 && *j > 0 {
                    // Contracts to a scalar
                    let child_i = self.build_child(term, *i, visited_ops)?;
                    let child_j = self.build_child(term, *j, visited_ops)?;
                    Ok(self.add_node(LorentzEvalNode::ProjMAmp {
                        i: child_i,
                        j: child_j,
                    }))
                } else {
                    // Projects a fermion
                    let k = if *i == idx {
                        *j
                    } else {
                        assert_eq!(*j, idx);
                        *i
                    };
                    let child = self.build_child(term, k, visited_ops)?;
                    Ok(self.add_node(LorentzEvalNode::ProjM { i: child }))
                }
            }
            LorentzOp::ProjP { i, j } => {
                if *i > 0 && *j > 0 {
                    // Contracts to a scalar
                    let child_i = self.build_child(term, *i, visited_ops)?;
                    let child_j = self.build_child(term, *j, visited_ops)?;
                    Ok(self.add_node(LorentzEvalNode::ProjPAmp {
                        i: child_i,
                        j: child_j,
                    }))
                } else {
                    // Projects a fermion
                    let k = if *i == idx {
                        *j
                    } else {
                        assert_eq!(*j, idx);
                        *i
                    };
                    let child = self.build_child(term, k, visited_ops)?;
                    Ok(self.add_node(LorentzEvalNode::ProjP { i: child }))
                }
            }
            LorentzOp::Metric { mu, nu } => {
                let child_mu = self.build_child(term, *mu, visited_ops)?;
                let child_nu = self.build_child(term, *nu, visited_ops)?;
                Ok(self.add_node(LorentzEvalNode::Metric {
                    mu: child_mu,
                    nu: child_nu,
                }))
            }
            LorentzOp::Sigma { .. } => Err(CompileError::UnsupportedVertex(
                "Sigma tensors are deferred to future work".to_string(),
            )),
            LorentzOp::Epsilon { .. } => Err(CompileError::UnsupportedVertex(
                "Epsilon tensors are deferred to future work".to_string(),
            )),
            LorentzOp::C { .. } => Err(CompileError::UnsupportedVertex(
                "Charge conjugation is deferred to future work".to_string(),
            )),
            LorentzOp::P { .. } | LorentzOp::Identity { .. } => {
                todo!("P and Identity operators not yet implemented in tree builder")
            }
        }
    }

    /// Get the node at the specified index.
    pub fn node(&self, idx: usize) -> &LorentzEvalNode {
        &self.nodes[idx]
    }

    /// Get the root node
    pub fn root(&self) -> &LorentzEvalNode {
        self.root
            .map(|idx| self.node(idx))
            .expect("The only public constructor is build_at_leg, and it always sets a root node")
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
    ) -> Result<Self, CompileError> {
        let mut tree = LorentzEvalTree {
            nodes: vec![],
            root: None,
        };
        let mut visited_ops = Vec::new(); // LorentzOp is so small that Vec is probably better than HashSet
        let mut term_roots = Vec::new();

        // If idx is specified, build the tree rooted at that leg
        if let Some(idx) = idx {
            term_roots.push(tree.build_child(term, (idx as i32) + 1, &mut visited_ops)?);
        };

        // find all the remaining scalar roots
        while visited_ops.len() < term.ops.len() {
            // find an unvisited op
            let Some((_, op)) = term
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
                LorentzOp::ProjM { i, j } | LorentzOp::ProjP { i, j } if *i > 0 && *j > 0 => {
                    // upstream will handle the contraction
                    tree.build_child(term, *i, &mut visited_ops)?
                }
                LorentzOp::Metric { mu, nu } if *mu > 0 && *nu > 0 => {
                    // upstream will handle the contraction
                    tree.build_child(term, *mu, &mut visited_ops)?
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

        // Find any scalar legs not connected to any operator and add them as scalar roots
        for (ileg, spin) in spins.iter().enumerate() {
            if *spin == 1 {
                let idx = (ileg as i32) + 1;
                if !tree
                    .nodes
                    .iter()
                    .any(|node| matches!(node, LorentzEvalNode::Leg(i) if *i == idx))
                {
                    term_roots.push(tree.add_node(LorentzEvalNode::Leg(idx)));
                }
            }
        }

        // take scalar product
        let root = if term_roots.len() == 1 {
            term_roots[0]
        } else {
            tree.add_node(LorentzEvalNode::ScalarProduct {
                children: term_roots,
            })
        };

        tree.root = Some(root);
        Ok(tree)
    }
}

/// Resolve a single LorentzTerm into a rooted primitive with the output leg fixed.
///
/// # Arguments
/// * `term` — The UFO LorentzTerm to resolve.
/// * `spins` — Spin codes [1, 2, 3] for each leg (1-indexed).
/// * `result_leg_idx` — The output leg (0-indexed), or `None` for amplitude (scalar sink).
///
/// # Returns
/// A `RootedTerm` ready for evaluation, or a `CompileError`.
pub fn root_term(
    term: &crate::ufo::lorentz::LorentzTerm,
    spins: &[i32],
    result_leg_idx: Option<usize>,
) -> Result<RootedTerm, CompileError> {
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
    fn test_root_ffv1_photon_current_at_leg3() {
        // FFV1: Gamma(3,2,1) rooted at vector leg 3 → SpinorCurrent{Both,row=2,col=1}
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::Gamma { mu: 3, i: 2, j: 1 }],
        };
        let spins = vec![2, 2, 3]; // e+, e-, photon
        let result = root_term(&term, &spins, Some(2)).unwrap();
        assert_eq!(result.coeff, 1.0);
        assert_eq!(
            result.tree,
            LorentzEvalTree {
                nodes: vec![
                    LorentzEvalNode::Leg(2),
                    LorentzEvalNode::Leg(1),
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
            ops: vec![LorentzOp::Gamma { mu: 3, i: 2, j: 1 }],
        };
        let spins = vec![2, 2, 3];
        let result = root_term(&term, &spins, None).unwrap();
        assert_eq!(result.coeff, 1.0);
        // When rooted at amplitude, builds the Gamma structure and contracts with a vector leg
        assert_eq!(
            result.tree,
            LorentzEvalTree {
                nodes: vec![
                    LorentzEvalNode::Leg(2),
                    LorentzEvalNode::Leg(1),
                    LorentzEvalNode::GammaVout { i: 0, j: 1 },
                    LorentzEvalNode::Leg(3),
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
                LorentzOp::Gamma { mu: 3, i: 2, j: -1 },
                LorentzOp::ProjM { i: -1, j: 1 },
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
                    LorentzEvalNode::Leg(2),
                    LorentzEvalNode::Leg(1),
                    LorentzEvalNode::ProjM { i: 1 },
                    LorentzEvalNode::GammaVout { i: 0, j: 2 },
                    LorentzEvalNode::Leg(3),
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
            ops: vec![LorentzOp::ProjM { i: 2, j: 1 }],
        };
        let spins = vec![2, 2, 1];
        let result = root_term(&term, &spins, None).unwrap();
        assert_eq!(
            result.tree,
            LorentzEvalTree {
                nodes: vec![
                    LorentzEvalNode::Leg(2),
                    LorentzEvalNode::Leg(1),
                    LorentzEvalNode::ProjMAmp { i: 0, j: 1 },
                    LorentzEvalNode::Leg(3),
                    LorentzEvalNode::ScalarProduct {
                        children: vec![2, 3]
                    },
                ],
                root: Some(4)
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
                mu: 1,
                nu: 2,
                i: 2,
                j: 1,
            }],
        };
        let spins = vec![2, 2, 3];
        let result = root_term(&term, &spins, Some(0)); // root at leg 1 (0-indexed as 0)
        assert!(matches!(result, Err(CompileError::UnsupportedVertex(_))));
    }

    #[test]
    fn test_root_vvs_metric() {
        // VVS1: Metric(1,2) rooted at amplitude → Metric contraction
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::Metric { mu: 1, nu: 2 }],
        };
        let spins = vec![3, 3, 1];
        let result = root_term(&term, &spins, None).unwrap();
        assert_eq!(result.coeff, 1.0);
        // When rooted at amplitude with 2 vector legs, uses Metric to contract them
        assert_eq!(
            result.tree,
            LorentzEvalTree {
                nodes: vec![
                    LorentzEvalNode::Leg(1),
                    LorentzEvalNode::Leg(2),
                    LorentzEvalNode::Metric { mu: 0, nu: 1 },
                    LorentzEvalNode::Leg(3),
                    LorentzEvalNode::ScalarProduct {
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
                    LorentzEvalNode::Leg(1),
                    LorentzEvalNode::Leg(2),
                    LorentzEvalNode::Leg(3),
                    LorentzEvalNode::ScalarProduct {
                        children: vec![0, 1, 2]
                    },
                ],
                root: Some(3)
            }
        )
    }
}
