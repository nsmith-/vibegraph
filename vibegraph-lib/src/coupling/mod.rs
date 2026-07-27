//! Running couplings: the scale dependence MadGraph applies to the strong
//! coupling between the model's parameter card and the matrix element.
//!
//! [`alphas`] is the renormalisation-group evolution itself — a port of
//! MadGraph's `Source/alfas_functions.f` — together with the rule that fixes its
//! two inputs (`αs(M_Z)` and the loop order) from a run card and a parameter
//! card.
//!
//! [`scales`] is the other half: the scale the coupling is evaluated *at*, and
//! the per-beam factorisation scale the same run-card prescription fixes with it.

//! [`topology`] supplies the one input [`scales`] does not take from the run
//! card — the facts about a process's diagrams that MadGraph's clustering
//! consults — by reading them off the enumerated diagrams.

pub mod alphas;
pub mod scales;
pub mod topology;
