//! Build a feyngraph `DiagramSelector` from a `ConcreteProcess`.

use std::sync::Arc;

use feyngraph::diagram::view::DiagramView;
use feyngraph::DiagramSelector;

use super::check::AmplitudeOrder;
use super::parse::CouplingOp;
use crate::ufo::topo::NegativeOrders;

/// Maximum coupling power considered when translating a `>` constraint.
///
/// feyngraph has no built-in upper-unbounded selector; we enumerate powers
/// up to this limit. 20 is well beyond any physical LO tree-level process.
const MAX_COUPLING_POWER: usize = 20;

/// A fully concrete process: every leg a model particle name.
#[derive(Debug, Clone)]
pub(crate) struct ConcreteProcess {
    pub(crate) initial: Vec<String>,
    pub(crate) final_state: Vec<String>,
    /// Particles that may not appear as a propagator.
    pub(crate) forbidden_particles: Vec<String>,
    pub(crate) orders: Vec<AmplitudeOrder>,
}

/// Translate a `ConcreteProcess` into a feyngraph `DiagramSelector`.
///
/// The s-channel restrictions are not selector constraints. `>` and `$$`
/// filter the converted [`super::Diagram`]s ([`super::schannel`]), whose
/// propagators carry the signed external-momentum combination that decides
/// whether a line is an s-channel; `$` keeps every diagram and vetoes a window
/// of the integration ([`crate::onshell`]).
///
/// An order some interaction carries a negative power of is bounded on the
/// diagram's signed sum rather than through feyngraph's coupling powers, which
/// count only the non-negative parts ([`NegativeOrders`]).
pub(crate) fn build_selector(proc: &ConcreteProcess, negative: &NegativeOrders) -> DiagramSelector {
    let mut sel = DiagramSelector::new();

    // Forbidden propagator species: `/ Z` → zero Z propagators in the diagram.
    for name in &proc.forbidden_particles {
        sel.select_propagator_count(name, 0);
    }

    // Coupling order constraints.
    for c in &proc.orders {
        if negative.carries(&c.name) {
            apply_signed_constraint(&mut sel, c, negative);
        } else {
            apply_coupling_constraint(&mut sel, c);
        }
    }

    sel
}

/// `c` on the diagram's signed power of its order, with the comparisons
/// [`apply_coupling_constraint`] reads: an upper bound for `=` and `<=` (none
/// when negative, as MadGraph's `reduce_orders` ignores a negative bound,
/// `diagram_generation.py:1055-1072`), equality for `==`, a strict lower bound
/// for `>`.
fn apply_signed_constraint(
    sel: &mut DiagramSelector,
    c: &AmplitudeOrder,
    negative: &NegativeOrders,
) {
    let name = c.name.clone();
    let v = c.value;
    let negative = negative.clone();
    let power = move |view: &DiagramView| view.order(&name) as i64 + negative.power(view, &name);
    let keep: Arc<dyn Fn(&DiagramView) -> bool + Send + Sync> = match c.op {
        CouplingOp::Eq | CouplingOp::Le if v < 0 => return,
        CouplingOp::Eq | CouplingOp::Le => Arc::new(move |view| power(view) <= v),
        CouplingOp::ExactEq => Arc::new(move |view| power(view) == v),
        CouplingOp::Gt => Arc::new(move |view| power(view) > v),
    };
    sel.add_custom_function(keep);
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
        let _ = build_selector(&proc, &NegativeOrders::default());
    }

    #[test]
    fn test_coupling_le() {
        let proc = concrete(vec![], vec![constraint("QCD", CouplingOp::Le, 2)]);
        let _ = build_selector(&proc, &NegativeOrders::default());
    }

    #[test]
    fn test_coupling_exact_eq() {
        let proc = concrete(vec![], vec![constraint("QED", CouplingOp::ExactEq, 4)]);
        let _ = build_selector(&proc, &NegativeOrders::default());
    }

    #[test]
    fn test_forbidden_propagator() {
        let proc = concrete(vec!["Z"], vec![]);
        let _ = build_selector(&proc, &NegativeOrders::default());
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
        let _ = build_selector(&proc, &NegativeOrders::default());
    }

    #[test]
    fn test_negative_value_skipped_gracefully() {
        // Negative coupling powers are physically nonsensical; build_selector should
        // not panic — it will produce an empty power list for these cases.
        let proc = concrete(vec![], vec![constraint("QCD", CouplingOp::ExactEq, -1)]);
        let _ = build_selector(&proc, &NegativeOrders::default());
    }
}
