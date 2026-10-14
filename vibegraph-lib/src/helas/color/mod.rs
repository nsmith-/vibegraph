//! # `color` — symbolic SU(3) color algebra
//!
//! A compile-time (F-independent) engine for the tree-level SU(3) color
//! algebra, ported from MadGraph's `color_algebra.py`. It factors color out of
//! kinematics exactly the way MadGraph does: color structures are reduced
//! symbolically to a basis of generalized traces/deltas with **exact rational**
//! coefficients, so the floating-point runtime never carries a color index.
//!
//! ## Vocabulary
//!
//! - [`ColorCoeff`](coeff::ColorCoeff) — the scalar prefactor
//!   `q · i^imag · Nc^nc_power`, exact rational, checked arithmetic.
//! - [`ColorTensor`](tensor::ColorTensor) — a generalized color object: `T`,
//!   `Tr`, `f`, `d`, `One`.
//! - [`ColorString`](factor::ColorString) — a coefficient times a product of
//!   tensors.
//! - [`ColorFactor`](factor::ColorFactor) — a sum of color strings;
//!   [`ColorFactor::full_simplify`](factor::ColorFactor::full_simplify) reduces
//!   it to canonical form.
//!
//! The reduction is a fixed-point iteration of the SU(3) rewrite rules (`f`/`d`
//! to traces, `T`-chain merge, `T` closing to a trace, the three Fierz
//! variants, trace values, and conjugation).
//! [`ColorString::to_immutable`](factor::ColorString::to_immutable) gives the
//! basis key used downstream to assemble the color basis and the color matrix.
//!
//! [`flow_tags`] reads the resulting basis keys back as color *lines*, giving the
//! `(color, anticolor)` label pair per external leg that a Les Houches event
//! record carries.

pub(crate) mod coeff;
pub(crate) mod colorize;
pub(crate) mod factor;
pub mod flow_tags;
pub(crate) mod tensor;

pub use colorize::{colorize_process, ColorBasis};
pub use factor::ImmutableString;
pub use flow_tags::LeadingColorFlows;
pub(crate) use tensor::ColorAlgebraError;
pub use tensor::TensorKind;

#[cfg(test)]
mod tests;
