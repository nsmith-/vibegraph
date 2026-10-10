//! Runtime amplitude evaluator with a compiled, interned AST.
//!
//! `AmplitudeEvaluator` compiles a `DiagramSet` once and then evaluates amplitudes
//! rapidly for any phase-space point and helicity configuration.
//!
//! Compile time, in pipeline order:
//! - `root_diagram.rs` / `root_lorentz.rs` — passes 1 and 2: a [`Diagram`](crate::diagrams::Diagram)
//!   → rooted `DiagramEvalTree` (symbolic, model-bound; node payloads in
//!   `diagram_eval.rs`). `root_diagram.rs` also owns the per-diagram `DiagramEval`.
//! - `lower.rs` — pass 3a: `lower_flows` inlines every colour-flow JAMP and
//!   configuration amplitude into one `Ast<Sym>` with binary `Add`/`Mul`, and
//!   `optimize` re-flattens the sums and hash-conses common subexpressions.
//! - `fold.rs` — pass 3b: intern couplings, masses, widths and coefficients into deduped
//!   pool specs and fold constant composites, giving the card-independent
//!   `Ast<Const>`; `Folded::expand_helicities` specializes it per helicity combination.
//! - `analysis.rs` — static per-node annotations of a folded arena (output class,
//!   constness, momentum id, helicity support).
//! - `layout.rs` — lowers a folded arena and its analysis to the typed instruction
//!   stream (`Program`), in op-blocked execution order, with liveness-recycled arena
//!   slots.
//! - `compile.rs` — orchestrates these into a card-independent `AmplitudeEvaluator`.
//!
//! Run time:
//! - `run.rs` — `BoundAmplitude`: `BoundAmplitude::bind` resolves an `EvaluatedModel`
//!   into the constant pools, and `fill_arenas` runs a `Program`'s instructions over
//!   per-class result arenas. The generic `apply` reduces one node over
//!   [`WaveformSlot`](waveform_slot::WaveformSlot)s; constant folding and the test-only
//!   reference pass use it.
//! - `kernel.rs` — the Lorentz-primitive kernels: the slot-level fns `apply`
//!   dispatches to, and the `*_bare` fns on momentum-stripped values that
//!   `fill_arenas` calls.
//! - `lane_field.rs` / `lanes.rs` — SIMD lane batching over phase-space points.
//! - `rescale.rs` — moves a bound amplitude's pools to another `alpha_s`.
//!
//! Shared vocabulary: `op.rs` (the node language, `Op` with `Sym`/`Const` leaves),
//! `ast.rs` (the CSR arena `Ast<T>` and its s-expression I/O), `tree.rs` (the `Tree`
//! trait), `waveform_slot.rs` and `error.rs` (the eval-pass error tree).
//!
//! The modules reference each other in both directions; the list above is the
//! pipeline's order, not an import order.

mod analysis;
mod ast;
mod compile;
mod diagram_eval;
// Skeleton of the egglog rewrite stage: round-trips `Ast<Sym>` through an e-graph.
// No rules yet (an identity pass). Parked, not wired into the `lower::optimize`
// pipeline — see the module doc for why.
#[allow(dead_code)]
mod egraph;
mod error;
mod fold;
mod kernel;
// SIMD lane batching: `F = LaneField<N>` runs one `eval_m2` pass over N
// phase-space points. See the module doc for the lane-uniformity contract.
mod lane_field;
mod lanes;
mod layout;
mod lower;
mod op;
// Per-event operation count (an op-counting scalar field) and arena-traffic census of
// the forward pass: the inputs to a roofline reading of the evaluator. Study
// instrumentation, test-only.
#[cfg(test)]
mod roofline;
// Per-event strong coupling: `ScaleAwareAmplitude` owns a bound amplitude's constant
// pools and moves them to another `alpha_s`, either by scaling the tagged powers of `G`
// or by re-evaluating the model.
mod rescale;
// Reusable property-test harness: typed random-input generators + a "compare two
// kernels on the same random inputs" driver, for kernel-equivalence tests. Also
// compiled (without the test driver) for the `bench-internals` microbench facade.
#[cfg(any(test, feature = "bench-internals"))]
#[cfg_attr(not(test), allow(dead_code))]
mod prop_harness;
#[cfg(test)]
mod renumbering;
mod root_diagram;
mod root_lorentz;
#[cfg(test)]
mod rooting_soundness;
mod run;
#[cfg(test)]
mod stitching;
// Alternative topological execution orders for the compiled instruction stream, and the
// structural metrics that judge them. A study hook: the order production emits lives
// with the lowering in `layout.rs`, and this module exists only under `cfg(test)` or the
// `eval-schedule-study` feature.
#[cfg(any(test, feature = "eval-schedule-study"))]
#[cfg_attr(not(test), allow(dead_code))]
mod schedule;
mod tree;
mod waveform_slot;

/// Internal kernels, slot type, and typed random-slot generators, re-exported for
/// the kernel-granularity microbenches in `benches/`. Feature-gated and hidden:
/// not a public API surface.
#[cfg(feature = "bench-internals")]
#[doc(hidden)]
pub mod bench_internals {
    pub use super::kernel::{ffv_iout, ffv_vout, gamma_iout, gamma_vout, proj_m, proj_p};
    pub use super::prop_harness::{
        rand_bra, rand_c, rand_ket, rand_vector, seeded_rng, slots_approx_eq,
    };
    pub use super::run::mul_apply;
    pub use super::waveform_slot::WaveformSlot;
}

/// Per-model op-coverage census: which evaluator primitives a model's gated
/// process list actually compiles to, and the two-way assertion over its
/// allowlist. Feature-gated because the banked non-SM instance lives in an
/// integration test; not a public API surface.
#[cfg(feature = "extended-validation")]
#[doc(hidden)]
pub mod op_census {
    pub use super::compile::assert_op_coverage_across;
    pub use super::op::Op;
}

pub use compile::{config_groups, AmplitudeEvaluator};
pub use lane_field::{LaneField, Lanes, SupportedLanes};
pub use rescale::ScaleAwareAmplitude;
pub use run::{
    eval_m2_lanes, eval_m2_lanes_packed, pack_lane_points, BoundAmplitude, ScratchSpace,
};
