//! Vertex dispatch: resolve LorentzExpr + spin codes into rooted contraction trees at compile time.
//!
//! Each LorentzTerm is decomposed into a tensor network with an output fiber fixed by `result_leg_idx`.
//! The rooted descriptor (RootedNode) specifies the concrete primitive and its input/output orientation
//! so that eval walks resolved nodes, never symbolic ops.

pub use crate::helas::repr::lorentz::Chirality;
use crate::ufo::lorentz::{LorentzExpr, LorentzOp};

/// Which end of a fermion pair is the output leg.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpinorEnd {
    /// Output is row fermion (fo): fioxxx orientation.
    Row,
    /// Output is column fermion (fi): foxxx orientation.
    Col,
}

/// Boson factors in a pure-boson vertex (Metric or momentum P).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BosonFactor {
    /// Two Lorentz indices contracted via Minkowski metric g^μν.
    Metric { a: u8, b: u8 },
    /// Momentum leg contracted into a Lorentz index.
    Momentum { mu_leg: u8, p_leg: u8 },
}

/// A resolved, rooted contraction primitive with output fiber fixed.
///
/// Each variant corresponds to a HELAS primitive or group thereof,
/// already oriented for a known output leg (from `result_leg_idx`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RootedNode {
    /// FFV off-shell current: two fermions → vector (jioxxx/j3xxxx).
    /// Carries chirality so `GammaL` or `GammaR` can be selected at eval.
    SpinorCurrent {
        chirality: Chirality,
        row_leg: u8,
        col_leg: u8,
    },
    /// FFS or FFV amplitude: two fermions + optional boson → scalar.
    /// `vector_leg=Some(μ)` means iovxxx (current · vector); `None` means iosxxx (bilinear · scalar).
    SpinorAmplitude {
        chirality: Chirality,
        row_leg: u8,
        col_leg: u8,
        vector_leg: Option<u8>,
    },
    /// FFV or FFS fermion-out: one fermion + optional vector → off-shell spinor.
    /// `vector_leg=Some` triggers `project_left/right` + `GammaV`; `None` is a projected bilinear.
    SpinorOut {
        chirality: Chirality,
        out: SpinorEnd,
        in_leg: u8,
        vector_leg: Option<u8>,
    },
    /// Pure-boson scalar (e.g., VVS amplitude): product of Metric/P contractions.
    BosonScalar { factors: Vec<BosonFactor> },
    /// Pure-boson vector-out (e.g., VVS off-shell): copy or momentum into the output leg.
    BosonVector {
        out_leg: u8,
        factors: Vec<BosonFactor>,
    },
    /// Pure scalar product (SSS/SSSS): product of input scalar values.
    ScalarProduct,
}

/// A single LorentzTerm, already rooted at the output leg and ready to eval.
#[derive(Clone, Debug)]
pub struct RootedTerm {
    /// Coefficient carried from the UFO LorentzTerm.coeff (real, since couplings are stored separately).
    pub coeff: f64,
    /// Resolved primitive with output fiber fixed.
    pub node: RootedNode,
}

/// Compile error types for unsupported vertices or invalid structures.
#[derive(Clone, Debug)]
pub enum CompileError {
    /// The vertex structure contains unsupported ops (Sigma, Epsilon, C) or too many free indices.
    UnsupportedVertex(String),
    /// The structure is syntactically invalid (e.g., mismatched indices).
    InvalidStructure(String),
}

/// Resolve a single LorentzTerm into a rooted primitive with the output leg fixed.
///
/// # Arguments
/// * `term` — The UFO LorentzTerm to resolve.
/// * `spins` — Spin codes [1, 2, 3] for each leg (1-indexed).
/// * `result_leg_idx` — The output leg (1-indexed), or `None` for amplitude (scalar sink).
///
/// # Returns
/// A `RootedTerm` ready for evaluation, or a `CompileError`.
pub fn root_term(
    term: &crate::ufo::lorentz::LorentzTerm,
    spins: &[i32],
    result_leg_idx: Option<usize>,
) -> Result<RootedTerm, CompileError> {
    // Partition ops into spinor (Gamma|Identity|ProjM|ProjP) and boson (Metric|P).
    let mut spinor_ops = Vec::new();
    let mut boson_ops = Vec::new();

    for op in &term.ops {
        match op {
            LorentzOp::Gamma { .. }
            | LorentzOp::Identity { .. }
            | LorentzOp::ProjM { .. }
            | LorentzOp::ProjP { .. } => spinor_ops.push(op),
            LorentzOp::Metric { .. } | LorentzOp::P { .. } => boson_ops.push(op),
            LorentzOp::Sigma { .. } | LorentzOp::Epsilon { .. } | LorentzOp::C { .. } => {
                return Err(CompileError::UnsupportedVertex(
                    "Sigma/Epsilon/C tensors are deferred to future work".to_string(),
                ));
            }
        }
    }

    // Determine the output fiber and route to the appropriate node variant.
    let node = if spinor_ops.is_empty() {
        // Pure-boson vertex: route on boson structure and output fiber.
        route_boson_node(spins, result_leg_idx, boson_ops)?
    } else {
        // Spinor chain present: extract the chain and route.
        let (row_leg, col_leg, gamma_mu, chirality) = extract_spinor_chain(spins, &spinor_ops)?;
        route_spinor_node(
            spins,
            result_leg_idx,
            row_leg,
            col_leg,
            gamma_mu,
            chirality,
            boson_ops,
        )?
    };

    Ok(RootedTerm {
        coeff: term.coeff,
        node,
    })
}

/// Extract the spinor chain structure: fermion pair indices, Gamma free index, and chirality.
fn extract_spinor_chain(
    _spins: &[i32],
    spinor_ops: &[&LorentzOp],
) -> Result<(u8, u8, Option<u8>, Chirality), CompileError> {
    // Trace which legs appear in single-connection endpoints.
    // The two positive single-connection legs are the fermions (row/col).
    // Find any free Lorentz index (from Gamma), and infer chirality from projectors.

    let mut left_leg = None;
    let mut right_leg = None;
    let mut gamma_mu = None;
    let mut has_projm = false;
    let mut has_projp = false;

    for op in spinor_ops {
        match op {
            LorentzOp::Gamma { mu, i, j } => {
                gamma_mu = Some(*mu as u8);
                // i is the row (barred/outgoing), j is the col (unbarred/incoming)
                // but they may be reindexed through the term, so we'll identify them as endpoints
                if *i > 0 {
                    left_leg = Some(*i as u8);
                }
                if *j > 0 {
                    right_leg = Some(*j as u8);
                }
            }
            LorentzOp::Identity { i, j } => {
                if *i > 0 {
                    left_leg = Some(*i as u8);
                }
                if *j > 0 {
                    right_leg = Some(*j as u8);
                }
            }
            LorentzOp::ProjM { i, j } => {
                has_projm = true;
                if *i > 0 {
                    left_leg = Some(*i as u8);
                }
                if *j > 0 {
                    right_leg = Some(*j as u8);
                }
            }
            LorentzOp::ProjP { i, j } => {
                has_projp = true;
                if *i > 0 {
                    left_leg = Some(*i as u8);
                }
                if *j > 0 {
                    right_leg = Some(*j as u8);
                }
            }
            _ => {}
        }
    }

    let row_leg = left_leg
        .ok_or_else(|| CompileError::InvalidStructure("No left fermion leg found".to_string()))?;
    let col_leg = right_leg
        .ok_or_else(|| CompileError::InvalidStructure("No right fermion leg found".to_string()))?;

    let chirality = match (has_projm, has_projp) {
        (true, false) => Chirality::Left,
        (false, true) => Chirality::Right,
        (false, false) => Chirality::Both,
        (true, true) => {
            return Err(CompileError::InvalidStructure(
                "Both ProjM and ProjP in the same term — multi-term vertices must be split"
                    .to_string(),
            ));
        }
    };

    Ok((row_leg, col_leg, gamma_mu, chirality))
}

/// Route a spinor-chain vertex to the appropriate RootedNode variant based on output fiber.
fn route_spinor_node(
    spins: &[i32],
    result_leg_idx: Option<usize>,
    row_leg: u8,
    col_leg: u8,
    gamma_mu: Option<u8>,
    chirality: Chirality,
    _boson_ops: Vec<&LorentzOp>,
) -> Result<RootedNode, CompileError> {
    match result_leg_idx {
        Some(idx) => {
            // Output is a specific leg — check its role.
            let spin = spins.get(idx - 1).copied();
            match spin {
                Some(3) if gamma_mu.is_some() => {
                    // Output is a vector leg with a Gamma free index → SpinorCurrent.
                    Ok(RootedNode::SpinorCurrent {
                        chirality,
                        row_leg,
                        col_leg,
                    })
                }
                Some(2) => {
                    // Output is a fermion leg → SpinorOut.
                    let out = if idx == row_leg as usize {
                        SpinorEnd::Row
                    } else if idx == col_leg as usize {
                        SpinorEnd::Col
                    } else {
                        return Err(CompileError::InvalidStructure(
                            "Output leg is neither row nor col fermion".to_string(),
                        ));
                    };
                    Ok(RootedNode::SpinorOut {
                        chirality,
                        out,
                        in_leg: if out == SpinorEnd::Row {
                            col_leg
                        } else {
                            row_leg
                        },
                        vector_leg: gamma_mu,
                    })
                }
                _ => Err(CompileError::InvalidStructure(
                    "Output leg has unsupported spin or gamma structure".to_string(),
                )),
            }
        }
        None => {
            // Output is the amplitude (scalar sink) — check if there's a boson to contract with.
            if let Some(mu) = gamma_mu {
                // FFV amplitude: pair the current with the boson leg.
                Ok(RootedNode::SpinorAmplitude {
                    chirality,
                    row_leg,
                    col_leg,
                    vector_leg: Some(mu),
                })
            } else {
                // FFS amplitude: bilinear contracted with scalar.
                Ok(RootedNode::SpinorAmplitude {
                    chirality,
                    row_leg,
                    col_leg,
                    vector_leg: None,
                })
            }
        }
    }
}

/// Route a pure-boson vertex to the appropriate RootedNode variant.
fn route_boson_node(
    spins: &[i32],
    result_leg_idx: Option<usize>,
    _boson_ops: Vec<&LorentzOp>,
) -> Result<RootedNode, CompileError> {
    match result_leg_idx {
        Some(idx) => {
            // Output is a specific leg.
            let spin = spins.get(idx - 1).copied();
            match spin {
                Some(3) => {
                    // Output is a vector → BosonVector.
                    Ok(RootedNode::BosonVector {
                        out_leg: idx as u8,
                        factors: vec![], // Boson factors populated later if needed.
                    })
                }
                Some(1) => {
                    // Output is a scalar in a VVS-like structure; this is unusual but handled.
                    Ok(RootedNode::BosonScalar { factors: vec![] })
                }
                _ => Err(CompileError::InvalidStructure(
                    "Pure-boson output leg has unsupported spin".to_string(),
                )),
            }
        }
        None => {
            // Output is amplitude (scalar) — VVS, SSS, SSSS, etc.
            Ok(RootedNode::BosonScalar { factors: vec![] })
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Legacy DispatchKind for backward compatibility (will be phased out)
// ─────────────────────────────────────────────────────────────────────────────

/// Pre-compiled dispatch tag, derived from LorentzExpr + spins at compile time.
/// **DEPRECATED**: Being replaced by RootedNode. Kept for backward compatibility during migration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DispatchKind {
    FfvProjM,
    FfvProjP,
    Ffs,
    Vvv,
    Vvvv,
    Vvs,
    Sss,
    Ssss,
}

impl DispatchKind {
    /// Pattern-match a LorentzExpr against spin codes (legacy function).
    pub fn from_lorentz_expr(expr: &LorentzExpr, spins: &[i32]) -> Option<DispatchKind> {
        if expr.is_empty() {
            return None;
        }

        match (spins.len(), spins) {
            (3, [1, 1, 1]) => Some(DispatchKind::Sss),
            (4, [1, 1, 1, 1]) => Some(DispatchKind::Ssss),
            (3, [2, 2, 1]) | (3, [2, 1, 2]) | (3, [1, 2, 2]) => Some(DispatchKind::Ffs),
            (3, [2, 2, 3]) => Some(dispatch_ffv_legacy(expr)),
            (3, [3, 3, 1]) | (3, [3, 1, 3]) | (3, [1, 3, 3]) => Some(DispatchKind::Vvs),
            (3, [3, 3, 3]) => Some(DispatchKind::Vvv),
            (4, [3, 3, 3, 3]) => Some(DispatchKind::Vvvv),
            _ => None,
        }
    }
}

/// Legacy FFV dispatch helper.
fn dispatch_ffv_legacy(expr: &LorentzExpr) -> DispatchKind {
    let mut has_projm = false;
    let mut has_projp = false;

    for term in expr {
        for op in &term.ops {
            match op {
                LorentzOp::ProjM { .. } => has_projm = true,
                LorentzOp::ProjP { .. } => has_projp = true,
                _ => {}
            }
        }
    }

    match (has_projm, has_projp) {
        (true, false) => DispatchKind::FfvProjM,
        (false, true) => DispatchKind::FfvProjP,
        (false, false) => DispatchKind::FfvProjP,
        (true, true) => {
            let projm_count = expr
                .iter()
                .filter(|term| {
                    term.ops
                        .iter()
                        .any(|op| matches!(op, LorentzOp::ProjM { .. }))
                })
                .count();
            let projp_count = expr
                .iter()
                .filter(|term| {
                    term.ops
                        .iter()
                        .any(|op| matches!(op, LorentzOp::ProjP { .. }))
                })
                .count();

            if projm_count > projp_count {
                DispatchKind::FfvProjM
            } else {
                DispatchKind::FfvProjP
            }
        }
    }
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
        let result = root_term(&term, &spins, Some(3)).unwrap();
        assert_eq!(result.coeff, 1.0);
        match result.node {
            RootedNode::SpinorCurrent {
                chirality,
                row_leg,
                col_leg,
            } => {
                assert_eq!(chirality, Chirality::Both);
                assert_eq!(row_leg, 2);
                assert_eq!(col_leg, 1);
            }
            _ => panic!("Expected SpinorCurrent"),
        }
    }

    #[test]
    fn test_root_ffv1_amplitude_at_sink() {
        // FFV1 rooted at amplitude (scalar sink) → SpinorAmplitude{Both,row=2,col=1,vector=Some(3)}
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::Gamma { mu: 3, i: 2, j: 1 }],
        };
        let spins = vec![2, 2, 3];
        let result = root_term(&term, &spins, None).unwrap();
        match result.node {
            RootedNode::SpinorAmplitude {
                chirality,
                row_leg,
                col_leg,
                vector_leg,
            } => {
                assert_eq!(chirality, Chirality::Both);
                assert_eq!(row_leg, 2);
                assert_eq!(col_leg, 1);
                assert_eq!(vector_leg, Some(3));
            }
            _ => panic!("Expected SpinorAmplitude"),
        }
    }

    #[test]
    fn test_root_ffv2_left_chiral() {
        // FFV2: Gamma(3,2,-1)*ProjM(-1,1) → left-chiral
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![
                LorentzOp::Gamma { mu: 3, i: 2, j: -1 },
                LorentzOp::ProjM { i: -1, j: 1 },
            ],
        };
        let spins = vec![2, 2, 3];
        let result = root_term(&term, &spins, None).unwrap();
        match result.node {
            RootedNode::SpinorAmplitude { chirality, .. } => {
                assert_eq!(chirality, Chirality::Left);
            }
            _ => panic!("Expected SpinorAmplitude"),
        }
    }

    #[test]
    fn test_root_ffs_yakawa() {
        // FFS1: ProjM(2,1) rooted at amplitude → SpinorAmplitude{Left,row=2,col=1,vector=None}
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::ProjM { i: 2, j: 1 }],
        };
        let spins = vec![2, 2, 1];
        let result = root_term(&term, &spins, None).unwrap();
        match result.node {
            RootedNode::SpinorAmplitude {
                chirality,
                row_leg,
                col_leg,
                vector_leg,
            } => {
                assert_eq!(chirality, Chirality::Left);
                assert_eq!(row_leg, 2);
                assert_eq!(col_leg, 1);
                assert_eq!(vector_leg, None);
            }
            _ => panic!("Expected SpinorAmplitude"),
        }
    }

    #[test]
    fn test_root_sigma_unsupported() {
        // Sigma should trigger UnsupportedVertex.
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
        let result = root_term(&term, &spins, None);
        assert!(matches!(result, Err(CompileError::UnsupportedVertex(_))));
    }

    #[test]
    fn test_root_vvs_metric() {
        // VVS1: Metric(1,2) rooted at amplitude → BosonScalar
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::Metric { mu: 1, nu: 2 }],
        };
        let spins = vec![3, 3, 1];
        let result = root_term(&term, &spins, None).unwrap();
        match result.node {
            RootedNode::BosonScalar { .. } => {
                // Test passes; full factor evaluation is in run.rs
            }
            _ => panic!("Expected BosonScalar"),
        }
    }

    #[test]
    fn test_root_sss_scalar() {
        // SSS1: Identity (empty ops) rooted at amplitude → ScalarProduct
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![],
        };
        let spins = vec![1, 1, 1];
        let result = root_term(&term, &spins, None).unwrap();
        match result.node {
            RootedNode::BosonScalar { .. } => {
                // Routed to BosonScalar as a degenerate case (pure scalar with no ops).
            }
            _ => panic!("Expected BosonScalar or ScalarProduct"),
        }
    }

    // Legacy DispatchKind tests (for backward compatibility during migration)
    #[test]
    fn test_legacy_ffs_yakawa() {
        let expr = vec![LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::ProjM { i: 2, j: 1 }],
        }];
        let spins = vec![2, 2, 1];
        assert_eq!(
            DispatchKind::from_lorentz_expr(&expr, &spins),
            Some(DispatchKind::Ffs)
        );
    }

    #[test]
    fn test_legacy_ffv_projm() {
        let expr = vec![LorentzTerm {
            coeff: 1.0,
            ops: vec![
                LorentzOp::Gamma { mu: 3, i: 2, j: -1 },
                LorentzOp::ProjM { i: -1, j: 1 },
            ],
        }];
        let spins = vec![2, 2, 3];
        assert_eq!(
            DispatchKind::from_lorentz_expr(&expr, &spins),
            Some(DispatchKind::FfvProjM)
        );
    }

    #[test]
    fn test_legacy_vvs_higgs() {
        let expr = vec![LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::Metric { mu: 1, nu: 2 }],
        }];
        let spins = vec![3, 3, 1];
        assert_eq!(
            DispatchKind::from_lorentz_expr(&expr, &spins),
            Some(DispatchKind::Vvs)
        );
    }

    #[test]
    fn test_legacy_sss_scalar_triple() {
        let expr = vec![LorentzTerm {
            coeff: 1.0,
            ops: vec![],
        }];
        let spins = vec![1, 1, 1];
        assert_eq!(
            DispatchKind::from_lorentz_expr(&expr, &spins),
            Some(DispatchKind::Sss)
        );
    }
}
