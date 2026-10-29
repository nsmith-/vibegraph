//! Scale, PDF and αs variation weights, as MadGraph's systematics module
//! (`madgraph/various/systematics.py`) writes them, computed in-process.
//!
//! A variation's weight is `XWGTUP × W(variation) / W(nominal)`, where `W` is
//! the module's `get_lo_wgt` evaluated on an event's `<mgrwt>` record and its
//! written momenta. [`weight::lo_weight`] is that function over one
//! [`record::LoRecord`], so the same function writes this crate's weights and
//! is gated against MadGraph's on MadGraph's own events.
//!
//! * [`request`]: the typed request, from either card entry point or a
//!   command line, with MadGraph's defaults and refusals.
//! * [`record`]: the `<mgrwt>` record, filled as MadEvent fills it (from
//!   `rewgt` on a matched event), written and parsed.
//! * [`weight`]: `W` term by term.
//! * [`header`]: the variation list and the `<initrwgt>` block.
//! * [`plan`]: the request resolved against its PDF sets, and the writer that
//!   gives an event its weights.
//! * [`format`]: Python's `str(float)` and Fortran's `Ew.d`.

pub mod format;
pub mod header;
pub mod plan;
pub mod record;
pub mod request;
pub mod weight;
