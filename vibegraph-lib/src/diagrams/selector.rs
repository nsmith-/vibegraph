//! Build a feyngraph `DiagramSelector` from a `ConcreteProcess`.

use feyngraph::DiagramSelector;

use super::alias::ConcreteProcess;
use super::parse::{CouplingConstraint, CouplingOp};

/// Maximum coupling power considered when translating `>=` or `>` constraints.
///
/// feyngraph has no built-in upper-unbounded selector; we enumerate powers
/// up to this limit. 20 is well beyond any physical LO tree-level process.
const MAX_COUPLING_POWER: usize = 20;

/// Translate a `ConcreteProcess` into a feyngraph `DiagramSelector`.
///
/// # S-channel filters
///
/// `required_s_channels`, `forbidden_s_channels`, and `forbidden_onsh_s_channels`
/// require checking whether a propagator's momentum is a sum of only initial-state
/// external momenta. feyngraph's `DiagramView` does not yet expose momentum-flow
/// information in a way that supports this check cleanly. These three fields are
/// currently **not** forwarded to the selector.
///
/// TODO(s-channel): implement via `selector.add_custom_function` once feyngraph
/// exposes propagator momentum-flow data through `DiagramView`.
pub fn build_selector(proc: &ConcreteProcess) -> DiagramSelector {
    let mut sel = DiagramSelector::new();

    // Forbidden propagator species: `/ Z` → zero Z propagators in the diagram.
    for name in &proc.forbidden_particles {
        sel.select_propagator_count(name, 0);
    }

    // Coupling order constraints.
    for c in &proc.coupling_constraints {
        apply_coupling_constraint(&mut sel, c);
    }

    sel
}

fn apply_coupling_constraint(sel: &mut DiagramSelector, c: &CouplingConstraint) {
    // MadGraph treats squared-order constraints differently, but at the diagram-
    // selection level we apply both the same way for LO tree-level generation.
    let name = c.name.as_str();
    let v = c.value;

    match c.op {
        // `=` on amplitude orders → treated as `<=` (MadGraph coerces this).
        CouplingOp::Eq | CouplingOp::Le => {
            if v >= 0 {
                let powers: Vec<usize> = (0..=(v as usize)).collect();
                sel.select_coupling_power_list(name, powers);
            }
        }
        CouplingOp::Lt => {
            if v > 0 {
                let powers: Vec<usize> = (0..v as usize).collect();
                sel.select_coupling_power_list(name, powers);
            }
        }
        // `==` and `===` → exact equality.
        CouplingOp::ExactEq | CouplingOp::StrictEq => {
            if v >= 0 {
                sel.select_coupling_power(name, v as usize);
            }
        }
        CouplingOp::Ge => {
            let start = v.max(0) as usize;
            let powers: Vec<usize> = (start..=MAX_COUPLING_POWER).collect();
            sel.select_coupling_power_list(name, powers);
        }
        CouplingOp::Gt => {
            let start = (v + 1).max(0) as usize;
            let powers: Vec<usize> = (start..=MAX_COUPLING_POWER).collect();
            sel.select_coupling_power_list(name, powers);
        }
        CouplingOp::Ne => {
            let excluded = if v >= 0 { Some(v as usize) } else { None };
            let powers: Vec<usize> = (0..=MAX_COUPLING_POWER)
                .filter(|&p| excluded != Some(p))
                .collect();
            sel.select_coupling_power_list(name, powers);
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::alias::ConcreteProcess;
    use crate::diagrams::parse::{CouplingConstraint, CouplingOp};

    // DiagramSelector fields are pub(crate) in feyngraph so we can't inspect them
    // directly. These tests are smoke tests verifying build_selector completes
    // without panic. Behavioral verification is covered by integration tests in mod.rs.

    fn concrete(forbidden: Vec<&str>, constraints: Vec<CouplingConstraint>) -> ConcreteProcess {
        ConcreteProcess {
            initial: vec!["e+".into(), "e-".into()],
            final_state: vec!["mu+".into(), "mu-".into()],
            forbidden_particles: forbidden.into_iter().map(String::from).collect(),
            forbidden_s_channels: vec![],
            forbidden_onsh_s_channels: vec![],
            required_s_channels: vec![],
            coupling_constraints: constraints,
        }
    }

    fn constraint(name: &str, op: CouplingOp, value: i64) -> CouplingConstraint {
        CouplingConstraint {
            name: name.into(),
            squared: false,
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
