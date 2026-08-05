//! Runtime amplitude evaluator with a compiled, interned AST.
//!
//! `AmplitudeEvaluator` compiles a `DiagramSet` once and then evaluates amplitudes
//! rapidly for any phase-space point and helicity configuration.
//!
//! Pipeline (a one-way DAG — every module below depends only on earlier ones):
//! - `error.rs` — the eval-pass error tree (`RootDiagramError`/`RootLorentzError` →
//!   `CompileError` → `EvalError`); a leaf so the passes depend on it one-way.
//! - `root_diagram.rs` / `root_lorentz.rs` — pass 1+2: `DiagramView` → rooted
//!   `DiagramEvalTree` (symbolic, model-bound; node payloads in `diagram_eval.rs`).
//!   `root_diagram.rs` also owns the per-diagram `DiagramEval` + `compile_diagram_ast`.
//! - `op.rs` — the unified node language (`Op` + `Sym`/`Const` leaves).
//! - `ast.rs` — the unified `Ast<T>` arena (CSR children, s-expr I/O).
//! - `lower.rs` — pass 3a: inline all `DiagramEval`s into one binary-arity `Ast<Sym>`
//!   (+ `optimize`: sum re-flattening and hash-cons CSE, with the egglog stage to
//!   slot in before them).
//! - `fold.rs` — pass 3b: intern couplings/masses/widths/coeffs into deduped pool specs
//!   → card-independent `Ast<Const>`.
//! - `compile.rs` — orchestrates passes 1–3 into a card-independent `AmplitudeEvaluator`.
//! - `run.rs` — `BoundAmplitude`: `BoundAmplitude::bind` resolves an `EvaluatedModel`
//!   into the pools; a single forward pass evaluates the folded arena.
//! - `kernel.rs` — the Lorentz-primitive eval kernels `run::apply` dispatches to
//!   (one `pub(crate)` fn per Lorentz `Op`, named for it).

// Static per-node analysis (output type, constness, momentum id, helicity support)
// over a lowered arena. The runtime cross-checks its predictions against every computed
// slot; the full annotation surface is consumed by the not-yet-landed layout/recycling
// passes and the egraph schema encoder.
#[allow(dead_code)]
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
// Lorentz-primitive eval kernels (one `pub(crate)` fn per Lorentz `Op`, named for it);
// the `run::apply` dispatch is `kernel::<op>(children)`.
mod kernel;
// SIMD lane batching: `F = NumericArray<f64, N>` runs one `eval_m2` pass over N
// phase-space points. See the module doc for the lane-uniformity contract.
mod lanes;
mod layout;
mod lower;
mod op;
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
mod root_diagram;
mod root_lorentz;
#[cfg(test)]
mod rooting_soundness;
mod run;
// Alternative topological execution orders for the compiled instruction stream, and the
// structural metrics that judge them. A study hook: the order production emits lives
// with the lowering in `layout.rs`, and this module exists only under `cfg(test)` or the
// `eval-schedule-study` feature.
#[cfg(any(test, feature = "eval-schedule-study"))]
#[cfg_attr(not(test), allow(dead_code))]
mod schedule;
// `Tree` is used by every arena; `Linearized`/`linearize`/`max_depth` are generic
// traversal utilities no current pass needs.
#[allow(dead_code)]
mod tree;
mod waveform_slot;

/// Internal kernels, slot type, and typed random-slot generators, re-exported for
/// the kernel-granularity microbenches in `benches/`. Feature-gated and hidden:
/// not a public API surface.
#[cfg(feature = "bench-internals")]
#[doc(hidden)]
pub mod bench_internals {
    pub use super::kernel::{
        ffv_iout, ffv_oout, ffv_vout, gamma_iout, gamma_oout, gamma_vout, metric, proj_m, proj_p,
        propagate_core,
    };
    pub use super::prop_harness::{
        rand_bra, rand_c, rand_ket, rand_vector, seeded_rng, slots_approx_eq,
    };
    pub use super::run::mul_apply;
    pub use super::waveform_slot::WaveformSlot;
}

pub use ast::{Ast, ParseAstError};
pub use compile::AmplitudeEvaluator;
pub use error::{CompileError, EvalError, RootDiagramError};
pub use lanes::LaneField;
pub use op::{Const, ConstKind, Node, Op, Sym};
pub use rescale::{PoolTagCensus, RescaleFallback, ScaleAwareAmplitude};
pub use root_diagram::compile_diagram_ast;
pub use root_lorentz::RootLorentzError;
pub use run::{
    eval_m2_lanes, eval_m2_lanes_packed, pack_lane_points, BoundAmplitude, ScratchSpace,
};
