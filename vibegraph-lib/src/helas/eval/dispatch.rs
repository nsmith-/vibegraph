//! Vertex dispatch: LorentzExpr + spin codes → HELAS routine selection.

use super::ast::DispatchKind;
use crate::ufo::lorentz::LorentzExpr;

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
pub fn dispatch_lorentz_expr(expr: &LorentzExpr, spins: &[i32]) -> Option<DispatchKind> {
    // TODO: Implement full pattern matching against LorentzExpr
    // For now, return None to indicate this needs implementation
    None
}

/// Classify a vertex by its spin signature and operator content.
///
/// # Example patterns
/// - Spins [2, 2, 3] + Gamma + ProjM → FFV with left-chiral coupling
/// - Spins [2, 2, 3] + Gamma + ProjP → FFV with right-chiral coupling
/// - Spins [2, 2, 1] + Identity → FFS (Yukawa)
/// - Spins [3, 3, 3] + Metric + P → VVV (triple gauge)
/// - Spins [1, 1, 1] → SSS (scalar triple)
///
/// Returns the dispatch kind, or None if unrecognized.
fn classify_vertex(expr: &LorentzExpr, spins: &[i32]) -> Option<DispatchKind> {
    // TODO: Implement classification logic
    None
}
