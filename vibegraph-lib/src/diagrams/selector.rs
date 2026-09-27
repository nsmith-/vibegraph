//! Build a feyngraph `DiagramSelector` from a `ConcreteProcess`.

use feyngraph::DiagramSelector;

use super::check::AmplitudeOrder;
use super::parse::CouplingOp;

/// Maximum coupling power considered when translating a `>` constraint.
///
/// feyngraph has no built-in upper-unbounded selector; we enumerate powers
/// up to this limit. 20 is well beyond any physical LO tree-level process.
const MAX_COUPLING_POWER: usize = 20;

/// A fully concrete process: every leg a model particle name.
#[derive(Debug, Clone)]
pub struct ConcreteProcess {
    pub initial: Vec<String>,
    pub final_state: Vec<String>,
    /// Particles that may not appear as a propagator.
    pub forbidden_particles: Vec<String>,
    pub orders: Vec<AmplitudeOrder>,
}

/// Translate a `ConcreteProcess` into a feyngraph `DiagramSelector`.
///
/// The s-channel restrictions (`>`, `$$`, `$`) never reach this point: they
/// are refused before enumeration. When they are supported they filter
/// converted [`super::Diagram`]s, whose propagators carry the signed
/// external-momentum combination that decides whether a line is an s-channel.
pub fn build_selector(proc: &ConcreteProcess) -> DiagramSelector {
    let mut sel = DiagramSelector::new();

    // Forbidden propagator species: `/ Z` → zero Z propagators in the diagram.
    for name in &proc.forbidden_particles {
        sel.select_propagator_count(name, 0);
    }

    // Coupling order constraints.
    for c in &proc.orders {
        apply_coupling_constraint(&mut sel, c);
    }

    sel
}

fn apply_coupling_constraint(sel: &mut DiagramSelector, c: &AmplitudeOrder) {
    let name = c.name.as_str();
    let v = c.value;

    match c.op {
        // `=` on amplitude orders is an upper bound, as MadGraph reads it.
        CouplingOp::Eq | CouplingOp::Le => {
            if v >= 0 {
                let powers: Vec<usize> = (0..=(v as usize)).collect();
                sel.select_coupling_power_list(name, powers);
            }
        }
        CouplingOp::ExactEq => {
            if v >= 0 {
                sel.select_coupling_power(name, v as usize);
            }
        }
        CouplingOp::Gt => {
            let start = (v + 1).max(0) as usize;
            let powers: Vec<usize> = (start..=MAX_COUPLING_POWER).collect();
            sel.select_coupling_power_list(name, powers);
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::parse::CouplingOp;

    // DiagramSelector fields are pub(crate) in feyngraph so we can't inspect them
    // directly. These tests are smoke tests verifying build_selector completes
    // without panic. Behavioral verification is covered by integration tests in mod.rs.

    fn concrete(forbidden: Vec<&str>, orders: Vec<AmplitudeOrder>) -> ConcreteProcess {
        ConcreteProcess {
            initial: vec!["e+".into(), "e-".into()],
            final_state: vec!["mu+".into(), "mu-".into()],
            forbidden_particles: forbidden.into_iter().map(String::from).collect(),
            orders,
        }
    }

    fn constraint(name: &str, op: CouplingOp, value: i64) -> AmplitudeOrder {
        AmplitudeOrder {
            name: name.into(),
            op,
            value,
        }
    }

    #[test]
    fn test_no_constraints() {
        let proc = concrete(vec![], vec![]);
        let _ = build_selector(&proc);
    }

    #[test]
    fn test_coupling_le() {
        let proc = concrete(vec![], vec![constraint("QCD", CouplingOp::Le, 2)]);
        let _ = build_selector(&proc);
    }

    #[test]
    fn test_coupling_exact_eq() {
        let proc = concrete(vec![], vec![constraint("QED", CouplingOp::ExactEq, 4)]);
        let _ = build_selector(&proc);
    }

    #[test]
    fn test_forbidden_propagator() {
        let proc = concrete(vec!["Z"], vec![]);
        let _ = build_selector(&proc);
    }

    #[test]
    fn test_multiple_constraints() {
        let proc = concrete(
            vec!["t"],
            vec![
                constraint("QCD", CouplingOp::Le, 2),
                constraint("QED", CouplingOp::ExactEq, 0),
            ],
        );
        let _ = build_selector(&proc);
    }

    #[test]
    fn test_negative_value_skipped_gracefully() {
        // Negative coupling powers are physically nonsensical; build_selector should
        // not panic — it will produce an empty power list for these cases.
        let proc = concrete(vec![], vec![constraint("QCD", CouplingOp::ExactEq, -1)]);
        let _ = build_selector(&proc);
    }
}
