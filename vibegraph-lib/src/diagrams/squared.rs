//! Squared-order constraints: MadGraph's selection of the terms of |M|² by their
//! coupling order.
//!
//! A squared-order constraint (`NP^2==1`, `QCD^2<=2`) bounds the order of a term
//! `A_a A_b*` of |M|² — the sum of the two diagrams' orders — and not the order of
//! any one diagram. MadGraph honours one in two places, and this module mirrors
//! both:
//!
//! 1. **Diagram generation** (`apply_squared_order_constraints`,
//!    `diagram_generation.py:856-899`) drops every diagram that cannot pair with
//!    any diagram of the list to make a term the constraints accept, repeating
//!    until nothing more drops, and then resolves at most one negative constraint
//!    (`QED^2<=-1`, "up to the next-to-leading power") into a positive target
//!    computed from the surviving diagrams ([`SquaredRequest::filter`]). Dropping
//!    only diagrams none of whose terms are selected does not move |M|²; it moves
//!    the diagram list, and so the integration configurations.
//! 2. **The matrix element** (`set_chosen_SO_index`, `export_v4.py:1171-1203`, and
//!    the `CHOSEN_SO_CONFIGS` mask of `matrix_madevent_group_v4.inc:359-372`): the
//!    JAMPs are split by the diagrams' orders in the constrained names (the
//!    process's `split_orders`, `madgraph_interface.py:5264`), and |M|² sums the
//!    colour contraction `JAMP(i,M)·CF·JAMP(j,N)*` over only the pairs of order
//!    classes `(M, N)` whose summed orders the constraints accept
//!    ([`SquaredOrders::selected_pairs`]).
//!
//! What reaches the evaluator is a [`SquaredOrders`] on the subprocess's
//! [`DiagramSet`]: the constraints with any negative target
//! already resolved, which is what the selection at evaluation time reads.

use serde::{Deserialize, Serialize};

use super::check::{SquaredOrder, SupportedCard};
use super::parse::CouplingOp;
use super::resolve::{check_order_name, ResolveError};
use super::{Diagram, DiagramError, DiagramSet};
use crate::ufo::UFOModel;

/// The order name MadGraph's `WEIGHTED` is: every coupling order weighted by its
/// hierarchy.
const WEIGHTED: &str = "WEIGHTED";

/// The comparison a squared-order constraint makes. MadGraph reads a squared
/// `=` as `<=` (`madgraph_interface.py:4936-4939`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SquaredOp {
    /// `<=` (and `=`): the term's order is at most the value.
    #[serde(rename = "<=")]
    Le,
    /// `==`: the term's order is exactly the value.
    #[serde(rename = "==")]
    Eq,
    /// `>`: the term's order exceeds the value.
    #[serde(rename = ">")]
    Gt,
}

impl SquaredOp {
    fn of(op: CouplingOp) -> Self {
        match op {
            CouplingOp::Eq | CouplingOp::Le => SquaredOp::Le,
            CouplingOp::ExactEq => SquaredOp::Eq,
            CouplingOp::Gt => SquaredOp::Gt,
        }
    }

    /// `pass_squared_order_constraints`' comparison (`base_objects.py:2671-2674`).
    fn accepts(self, order: i64, value: i64) -> bool {
        match self {
            SquaredOp::Le => order <= value,
            SquaredOp::Eq => order == value,
            SquaredOp::Gt => order > value,
        }
    }
}

impl std::fmt::Display for SquaredOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SquaredOp::Le => "<=",
            SquaredOp::Eq => "==",
            SquaredOp::Gt => ">",
        })
    }
}

/// One squared-order constraint on a model order, aliases resolved.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SquaredConstraint {
    /// The model's order name, or `WEIGHTED`.
    pub order: String,
    pub op: SquaredOp,
    /// The bound on the summed order of a term `A_a A_b*`.
    pub value: i64,
}

impl std::fmt::Display for SquaredConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}^2{}{}", self.order, self.op, self.value)
    }
}

/// The squared-order selection of one subprocess: which terms `A_a A_b*` of
/// |M|² enter it, read off the two diagrams' orders in the constrained names.
///
/// Every value is non-negative: a negative constraint has been resolved into
/// its positive target against the subprocess's own diagrams.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SquaredOrders {
    /// One constraint per constrained order, in MadGraph's `split_orders` order:
    /// `WEIGHTED` first, the rest by hierarchy, then by name.
    pub constraints: Vec<SquaredConstraint>,
}

impl SquaredOrders {
    /// A diagram's orders in the constrained names, in constraint order: its
    /// summed signed powers, `WEIGHTED` being each order's power times its
    /// hierarchy (`Diagram.calculate_orders`, `base_objects.py:2638-2657`). An
    /// order the diagram does not carry is zero (`Diagram.get_order`).
    fn diagram_orders(&self, diagram: &Diagram, model: &UFOModel) -> Vec<i64> {
        orders_of(&self.constraints, diagram, model)
    }

    /// Whether the term of two diagrams (or classes) of these orders is selected.
    fn accepts(&self, a: &[i64], b: &[i64]) -> bool {
        self.constraints
            .iter()
            .zip(a.iter().zip(b))
            .all(|(c, (&x, &y))| c.op.accepts(x + y, c.value))
    }

    /// The diagrams' order classes: per diagram, the index of its class, and the
    /// distinct order vectors in ascending order. The JAMPs are split one per
    /// class, as MadGraph's `get_split_orders_mapping` splits them.
    pub(crate) fn classes(
        &self,
        diagrams: &[Diagram],
        model: &UFOModel,
    ) -> (Vec<usize>, Vec<Vec<i64>>) {
        let orders: Vec<Vec<i64>> = diagrams
            .iter()
            .map(|d| self.diagram_orders(d, model))
            .collect();
        let mut keys = orders.clone();
        keys.sort();
        keys.dedup();
        let class = orders
            .iter()
            .map(|o| keys.binary_search(o).expect("every order vector is a key"))
            .collect();
        (class, keys)
    }

    /// The class pairs whose terms enter |M|², row-major over `keys`
    /// (`selected[a * n + b]`). Symmetric, since a term's order is a sum.
    pub(crate) fn selected_pairs(&self, keys: &[Vec<i64>]) -> Vec<bool> {
        let n = keys.len();
        let mut out = vec![false; n * n];
        for a in 0..n {
            for b in 0..n {
                out[a * n + b] = self.accepts(&keys[a], &keys[b]);
            }
        }
        out
    }

    /// Whether the selection can make |M|² negative: it is a sum of squared
    /// moduli only when the selected pairs are every pair of some set of classes,
    /// `|Σ_{a∈T} A_a|²`. Any other selection keeps an interference term without
    /// the squares that bound it, and its sign is the kinematics'.
    pub(crate) fn is_signed(&self, keys: &[Vec<i64>]) -> bool {
        let n = keys.len();
        let selected = self.selected_pairs(keys);
        let in_t: Vec<bool> = (0..n).map(|a| selected[a * n + a]).collect();
        (0..n).any(|a| (0..n).any(|b| selected[a * n + b] != (in_t[a] && in_t[b])))
    }

    /// [`is_signed`](Self::is_signed) over the classes of `diagrams`.
    pub(crate) fn is_signed_on(&self, diagrams: &[Diagram], model: &UFOModel) -> bool {
        let (_, keys) = self.classes(diagrams, model);
        self.is_signed(&keys)
    }
}

impl std::fmt::Display for SquaredOrders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parts: Vec<String> = self.constraints.iter().map(|c| c.to_string()).collect();
        f.write_str(&parts.join(" "))
    }
}

/// The orders of `diagram` in the names `constraints` constrains.
fn orders_of(constraints: &[SquaredConstraint], diagram: &Diagram, model: &UFOModel) -> Vec<i64> {
    let orders = diagram.coupling_orders(model);
    constraints
        .iter()
        .map(|c| {
            if c.order == WEIGHTED {
                orders
                    .iter()
                    .map(|(name, &n)| {
                        n * i64::from(model.order_hierarchy.get(name).copied().unwrap_or(0))
                    })
                    .sum()
            } else {
                orders.get(&c.order).copied().unwrap_or(0)
            }
        })
        .collect()
}

/// A process line's squared-order constraints against a model, before any
/// subprocess's diagrams are known: a negative value is still the request
/// "up to `−value − 1` powers past the lowest".
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SquaredRequest {
    constraints: Vec<SquaredConstraint>,
}

impl SquaredRequest {
    /// The written constraints as MadGraph's `extract_process` stores them
    /// (`madgraph_interface.py:4893-4940`): aliases resolved against the model
    /// (`aEW=n` is `QED^2=2n`, or `EW^2` on a model whose order is `EW`; `aS=n`
    /// is `QCD^2=2n`; `QED^2` and `EW^2` stand for each other), `=` read as
    /// `<=`, and the leftmost constraint on an order winning, as MadGraph's
    /// right-to-left strip of the line leaves it. `None` when there is none.
    pub(crate) fn resolve(
        written: &[SquaredOrder],
        model: &UFOModel,
    ) -> Result<Option<Self>, ResolveError> {
        if written.is_empty() {
            return Ok(None);
        }
        let known = |n: &str| model.order_hierarchy.contains_key(n);
        let mut constraints: Vec<SquaredConstraint> = Vec::new();
        for o in written.iter().rev() {
            let (order, value) = if o.alias {
                if known(&o.name) {
                    return Err(ResolveError::AliasIsAModelOrder(o.name.clone()));
                }
                // MadGraph prefers `EW` for `aEW` whenever the model defines it.
                let base = match o.name.as_str() {
                    "aEW" if known("EW") => "EW",
                    "aEW" => "QED",
                    _ => "QCD",
                };
                (base.to_owned(), 2 * o.value)
            } else {
                let base = match o.name.as_str() {
                    "QED" if !known("QED") && known("EW") => "EW",
                    "EW" if !known("EW") && known("QED") => "QED",
                    name => name,
                };
                (base.to_owned(), o.value)
            };
            if order != WEIGHTED {
                check_order_name(model, &order)?;
            }
            let c = SquaredConstraint {
                order,
                op: SquaredOp::of(o.op),
                value,
            };
            match constraints.iter_mut().find(|k| k.order == c.order) {
                Some(k) => *k = c,
                None => constraints.push(c),
            }
        }
        if constraints.iter().filter(|c| c.value < 0).count() > 1 {
            return Err(ResolveError::NegativeOrders);
        }
        let rank = |name: &str| {
            if name == WEIGHTED {
                0
            } else {
                model.order_hierarchy.get(name).copied().unwrap_or(0)
            }
        };
        constraints.sort_by(|a, b| {
            (rank(&a.order), a.order.as_str()).cmp(&(rank(&b.order), b.order.as_str()))
        });
        Ok(Some(SquaredRequest { constraints }))
    }

    /// The constraints, aliases resolved, a negative value still unresolved.
    pub(crate) fn constraints(&self) -> &[SquaredConstraint] {
        &self.constraints
    }

    /// The amplitude-level bounds MadGraph assumes when a line has squared orders
    /// and no amplitude order of its own (`madgraph_interface.py:4987-4999`): each
    /// constrained order bounded above by its squared value, or left unbounded
    /// (MadGraph's 99) where the value is negative or the constraint is `>`.
    /// Pairs `(order, bound)`.
    pub(crate) fn implied_amplitude_bounds(&self) -> Vec<(String, Option<i64>)> {
        self.constraints
            .iter()
            .map(|c| {
                let bound = (c.value >= 0 && c.op != SquaredOp::Gt).then_some(c.value);
                (c.order.clone(), bound)
            })
            .collect()
    }

    /// `apply_squared_order_constraints` on one subprocess's diagrams
    /// (`diagram_generation.py:856-899`): drop every diagram that pairs with no
    /// diagram of the list into a term the positive constraints accept, until
    /// nothing more drops; then resolve the negative constraint, if any, into its
    /// target `min + min + 2(−value − 1)` over the survivors' orders
    /// (`base_objects.py:2863-2880`) and drop once more against it.
    ///
    /// Returns the selection the subprocess's |M|² is formed with.
    pub(crate) fn filter(&self, diagrams: &mut Vec<Diagram>, model: &UFOModel) -> SquaredOrders {
        let positive: Vec<bool> = self.constraints.iter().map(|c| c.value >= 0).collect();
        let passes = |a: &[i64], b: &[i64], only: Option<usize>| {
            self.constraints
                .iter()
                .enumerate()
                .filter(|(i, _)| match only {
                    Some(k) => *i == k,
                    None => positive[*i],
                })
                .all(|(i, c)| c.op.accepts(a[i] + b[i], c.value))
        };
        let mut orders: Vec<Vec<i64>> = diagrams
            .iter()
            .map(|d| orders_of(&self.constraints, d, model))
            .collect();
        let keep_against = |orders: &[Vec<i64>], only: Option<usize>| -> Vec<bool> {
            orders
                .iter()
                .map(|a| orders.iter().any(|b| passes(a, b, only)))
                .collect()
        };
        let retain = |diagrams: &mut Vec<Diagram>, orders: &mut Vec<Vec<i64>>, keep: &[bool]| {
            let mut k = keep.iter();
            diagrams.retain(|_| *k.next().expect("one flag per diagram"));
            let mut k = keep.iter();
            orders.retain(|_| *k.next().expect("one flag per diagram"));
        };
        loop {
            let keep = keep_against(&orders, None);
            if keep.iter().all(|&k| k) {
                break;
            }
            retain(diagrams, &mut orders, &keep);
        }
        let mut constraints = self.constraints.clone();
        if let Some(k) = constraints.iter().position(|c| c.value < 0) {
            // `apply_negative_sq_order` takes the minimum over the list it filters
            // and over the list it pairs against, which are the same list here.
            let lowest = orders.iter().map(|o| o[k]).min().unwrap_or(0);
            constraints[k].value = 2 * lowest + 2 * (-constraints[k].value - 1);
            let c = &constraints[k];
            let keep: Vec<bool> = orders
                .iter()
                .map(|a| orders.iter().any(|b| c.op.accepts(a[k] + b[k], c.value)))
                .collect();
            retain(diagrams, &mut orders, &keep);
        }
        SquaredOrders { constraints }
    }
}

/// Refuse a card two of whose process lines share a process number and differ
/// in their squared-order constraints ([`DiagramError::MixedSquaredOrders`]):
/// [`annotate_sets`] finds a set's line by the number alone.
pub(crate) fn refuse_mixed_squared_orders(card: &SupportedCard) -> Result<(), DiagramError> {
    for (i, a) in card.processes.iter().enumerate() {
        for b in &card.processes[i + 1..] {
            if a.id == b.id && a.squared_orders != b.squared_orders {
                return Err(DiagramError::MixedSquaredOrders {
                    a: a.to_string(),
                    b: b.to_string(),
                });
            }
        }
    }
    Ok(())
}

/// Derive each set's squared-order selection again from the card line its
/// diagrams were enumerated from (their [`Provenance::process`]), as an artifact
/// reader must: the selection is not serialized
/// ([`DiagramSet::squared_orders`]).
///
/// The recorded diagrams are what enumeration kept, so the line's filter must
/// keep every one of them; a set it would not leave unchanged is refused
/// ([`DiagramError::SquaredOrderMismatch`]) rather than given a selection its
/// diagrams were not filtered with.
///
/// [`Provenance::process`]: super::diagram::Provenance
pub(crate) fn annotate_sets(
    sets: &mut [DiagramSet],
    card: &SupportedCard,
    model: &UFOModel,
) -> Result<(), DiagramError> {
    refuse_mixed_squared_orders(card)?;
    for set in sets.iter_mut() {
        set.squared_orders = None;
        let Some(first) = set.diagrams.first() else {
            continue;
        };
        let line = card
            .processes
            .iter()
            .find(|p| p.id == first.provenance.process);
        let Some(line) = line else {
            continue;
        };
        let Some(request) = SquaredRequest::resolve(&line.squared_orders, model)? else {
            continue;
        };
        let mut kept = set.diagrams.clone();
        let selection = request.filter(&mut kept, model);
        if kept.len() != set.diagrams.len() {
            return Err(DiagramError::SquaredOrderMismatch {
                process: set.label(),
            });
        }
        set.squared_orders = Some(selection);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sel(constraints: &[(&str, SquaredOp, i64)]) -> SquaredOrders {
        SquaredOrders {
            constraints: constraints
                .iter()
                .map(|&(order, op, value)| SquaredConstraint {
                    order: order.into(),
                    op,
                    value,
                })
                .collect(),
        }
    }

    /// `NP^2==1` over classes NP = 0, 1, 2 selects exactly the SM × NP¹ terms, in
    /// both orders, and is signed; `NP^2==2` selects NP¹ × NP¹ and SM × NP², also
    /// signed; `NP^2<=2` over NP = 0, 1 selects every pair, which is a square.
    #[test]
    fn the_pair_selection_follows_the_summed_order() {
        let keys = vec![vec![0], vec![1], vec![2]];
        let one = sel(&[("NP", SquaredOp::Eq, 1)]);
        assert_eq!(
            one.selected_pairs(&keys),
            [false, true, false, true, false, false, false, false, false]
        );
        assert!(one.is_signed(&keys));
        let two = sel(&[("NP", SquaredOp::Eq, 2)]);
        assert_eq!(
            two.selected_pairs(&keys),
            [false, false, true, false, true, false, true, false, false]
        );
        assert!(two.is_signed(&keys));
        let le = sel(&[("NP", SquaredOp::Le, 2)]);
        assert!(!le.is_signed(&keys[..2]));
        // A pure square of one class is not signed either.
        let pure = sel(&[("QCD", SquaredOp::Eq, 4)]);
        assert!(!pure.is_signed(&[vec![0], vec![2]]));
        // `>` is strict.
        let gt = sel(&[("QCD", SquaredOp::Gt, 2)]);
        assert_eq!(
            gt.selected_pairs(&[vec![0], vec![2]]),
            [false, false, false, true]
        );
    }

    /// Several constraints select a pair only when every one of them accepts it.
    #[test]
    fn several_constraints_must_all_accept() {
        let s = sel(&[("QED", SquaredOp::Le, 4), ("QCD", SquaredOp::Eq, 2)]);
        // (QED, QCD): (2, 0) and (0, 2).
        let keys = vec![vec![0, 2], vec![2, 0]];
        assert_eq!(s.selected_pairs(&keys), [false, true, true, false]);
    }
}
