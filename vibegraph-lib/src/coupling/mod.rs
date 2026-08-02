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
//!
//! [`cluster`] is where the run card's own default lives:
//! `dynamical_scale_choice = -1` names no closed form at all, and the scale
//! comes out of the kT clustering of `cluster.f` — from the merge graph a
//! process's integration channels imply, through to `μR` and the two `μF`.

pub mod alphas;
pub mod cluster;
pub mod scales;
