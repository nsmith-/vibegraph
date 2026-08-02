//! MadGraph's kT clustering, and the scales it reads off the result.
//!
//! `dynamical_scale_choice = -1` — the run-card default — names no closed form
//! at all. `setscales.f` returns zero for both scales, and `reweight.f`'s
//! `setclscales` fills them by clustering the event's external momenta down to a
//! `2 → 2` core and reading the scale off the vertices a colour line passes
//! through. This module is that path.
//!
//! The three pieces are separable, and each is a different kind of statement.
//!
//! * [`graph`] is combinatorics: which sets of external legs the process's
//!   integration channels let the clustering combine, derived from the channel
//!   forests. It sees no momenta.
//! * [`kt`] is the clustering: measures, tie-break, merge order, and the frame
//!   changes an initial-state merge makes. It sees no colour.
//! * [`setclscales`] is the walk: which vertices carry the scales, and the two
//!   rewrites applied before the geometric means are taken.
//!
//! The scale is not a function of an event's momenta alone. It depends on the
//! integration channel through three separate routes — the coupling-order filter
//! on the merge table, the resonance tagging, and the jet-count memo — so a
//! replay that does not know which channel produced an event cannot in general
//! reproduce its scale.

pub mod graph;
pub mod kt;
pub mod setclscales;
