//! Vertex dispatch: LorentzExpr + spin codes → HELAS routine selection.
use crate::ufo::lorentz::{LorentzExpr, LorentzOp};

/// Pre-compiled dispatch tag, derived from LorentzExpr + spins at compile time.
/// Eliminates symbolic evaluation on the hot path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DispatchKind {
    /// FFV with left-chiral projector (ProjM)
    FfvProjM,
    /// FFV with right-chiral projector (ProjP)
    FfvProjP,
    /// FFS Yukawa (scalar coupling)
    Ffs,
    /// VVV triple gauge
    Vvv,
    /// VVVV quartic gauge
    Vvvv,
    /// VVS (Higgs coupling)
    Vvs,
    /// SSS scalar triple
    Sss,
    /// SSSS scalar quartic
    Ssss,
}

/// Distinguish between FFV with left-chiral (ProjM) vs right-chiral (ProjP) projector.
///
/// # Returns
/// - `FfvProjM` if the expression contains `ProjM` without dominant `ProjP`
/// - `FfvProjP` if the expression contains `ProjP` without dominant `ProjM` or no projectors
fn dispatch_ffv(expr: &LorentzExpr) -> DispatchKind {
    // Count occurrences of ProjM and ProjP across all terms
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

    // Classify based on which projectors are present
    match (has_projm, has_projp) {
        (true, false) => DispatchKind::FfvProjM, // Only ProjM → left-chiral
        (false, true) => DispatchKind::FfvProjP, // Only ProjP → right-chiral
        (false, false) => DispatchKind::FfvProjP, // No projectors → default to right-chiral
        (true, true) => {
            // Both present: need to distinguish by term structure
            // If more terms have ProjM, it's left-chiral; otherwise right-chiral
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

impl DispatchKind {
    /// Pattern-match a LorentzExpr against spin codes to determine which HELAS routine to call.
    ///
    /// The dispatch is based on the operator set in the LorentzExpr and the spin signature.
    /// This function is called at compile time (once per vertex in the AST) to produce a
    /// `DispatchKind` tag. At runtime, the tag is used to select the appropriate HELAS routine.
    ///
    /// # Arguments
    /// * `expr` — The Lorentz structure expression (parsed from the UFO)
    /// * `spins` — UFO spin codes for each leg (1=scalar, 2=fermion, 3=vector)
    ///
    /// # Returns
    /// A `DispatchKind` tag, or `None` if the pattern is not recognized.
    pub fn from_lorentz_expr(expr: &LorentzExpr, spins: &[i32]) -> Option<DispatchKind> {
        if expr.is_empty() {
            return None;
        }

        // Classify by spin signature and operator content
        match (spins.len(), spins) {
            // Scalar vertices: SSS, SSSS
            (3, [1, 1, 1]) => Some(DispatchKind::Sss),
            (4, [1, 1, 1, 1]) => Some(DispatchKind::Ssss),

            // Fermion-fermion-scalar: FFS
            (3, [2, 2, 1]) | (3, [2, 1, 2]) | (3, [1, 2, 2]) => Some(DispatchKind::Ffs),

            // Fermion-fermion-vector: FFV with projector distinction
            (3, [2, 2, 3]) => {
                // Distinguish left-chiral (ProjM) from right-chiral (ProjP)
                Some(dispatch_ffv(expr))
            }

            // Vector-vector-scalar: VVS
            (3, [3, 3, 1]) | (3, [3, 1, 3]) | (3, [1, 3, 3]) => Some(DispatchKind::Vvs),

            // Vector-vector-vector: VVV
            (3, [3, 3, 3]) => Some(DispatchKind::Vvv),

            // Quartic gauge: VVVV
            (4, [3, 3, 3, 3]) => Some(DispatchKind::Vvvv),

            // Unrecognized pattern
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ufo::lorentz::LorentzTerm;

    use super::*;

    #[test]
    fn test_ffs_yakawa() {
        // FFS1: structure = 'ProjM(2,1)' — Yukawa with left chirality
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
    fn test_ffv_projm() {
        // FFV2: structure = 'Gamma(3,2,-1)*ProjM(-1,1)' — left-chiral
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
    fn test_ffv_projp() {
        // FFV3: 'Gamma(3,2,-1)*ProjM(-1,1) - 2*Gamma(3,2,-1)*ProjP(-1,1)'
        // Has both ProjM and ProjP; ProjP count = 1, ProjM count = 1 → depends on coefficients
        // For now, with equal counts, defaults to ProjP per the tie-break
        let expr = vec![
            LorentzTerm {
                coeff: 1.0,
                ops: vec![
                    LorentzOp::Gamma { mu: 3, i: 2, j: -1 },
                    LorentzOp::ProjM { i: -1, j: 1 },
                ],
            },
            LorentzTerm {
                coeff: -2.0,
                ops: vec![
                    LorentzOp::Gamma { mu: 3, i: 2, j: -1 },
                    LorentzOp::ProjP { i: -1, j: 1 },
                ],
            },
        ];
        let spins = vec![2, 2, 3];
        let result = DispatchKind::from_lorentz_expr(&expr, &spins);
        // Equal term counts → tie-break to ProjP
        assert_eq!(result, Some(DispatchKind::FfvProjP));
    }

    #[test]
    fn test_vvv_triple_gauge() {
        // VVV1: complex structure with Metric and P operators
        let expr = vec![LorentzTerm {
            coeff: 1.0,
            ops: vec![
                LorentzOp::P { mu: 3, leg: 1 },
                LorentzOp::Metric { mu: 1, nu: 2 },
            ],
        }];
        let spins = vec![3, 3, 3];
        assert_eq!(
            DispatchKind::from_lorentz_expr(&expr, &spins),
            Some(DispatchKind::Vvv)
        );
    }

    #[test]
    fn test_vvs_higgs() {
        // VVS1: structure = 'Metric(1,2)'
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
    fn test_sss_scalar_triple() {
        // SSS1: structure = '1' (just identity)
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

    #[test]
    fn test_ssss_scalar_quartic() {
        let expr = vec![LorentzTerm {
            coeff: 1.0,
            ops: vec![],
        }];
        let spins = vec![1, 1, 1, 1];
        assert_eq!(
            DispatchKind::from_lorentz_expr(&expr, &spins),
            Some(DispatchKind::Ssss)
        );
    }

    #[test]
    fn test_empty_expr() {
        let expr: Vec<LorentzTerm> = vec![];
        let spins = vec![2, 2, 3];
        assert_eq!(DispatchKind::from_lorentz_expr(&expr, &spins), None);
    }

    #[test]
    fn test_unknown_spin_signature() {
        let expr = vec![LorentzTerm {
            coeff: 1.0,
            ops: vec![],
        }];
        let spins = vec![5, 5, 5]; // Invalid spin codes
        assert_eq!(DispatchKind::from_lorentz_expr(&expr, &spins), None);
    }
}
