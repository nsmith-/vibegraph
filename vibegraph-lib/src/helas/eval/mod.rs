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
//! - `lower.rs` — pass 3a: inline all `DiagramEval`s into one `Ast<Sym>` (+ `optimize`
//!   no-op, the egglog hook).
//! - `fold.rs` — pass 3b: intern couplings/masses/widths/coeffs into deduped pool specs
//!   → card-independent `Ast<Const>`.
//! - `compile.rs` — orchestrates passes 1–3 into a card-independent `AmplitudeEvaluator`.
//! - `run.rs` — `BoundAmplitude`: `BoundAmplitude::bind` resolves an `EvaluatedModel`
//!   into the pools; a single forward pass evaluates the folded arena.
//! - `kernel.rs` — the Lorentz-primitive eval kernels `run::apply` dispatches to
//!   (one `pub(crate)` fn per Lorentz `Op`, named for it).

mod ast;
mod compile;
mod diagram_eval;
mod error;
mod fold;
// Lorentz-primitive eval kernels (one `pub(crate)` fn per Lorentz `Op`, named for it);
// the `run::apply` dispatch is `kernel::<op>(children)`.
mod kernel;
mod lower;
mod op;
// Reusable property-test harness: typed random-input generators + a "compare two
// kernels on the same random inputs" driver, for kernel-equivalence tests. Also
// compiled (without the test driver) for the `bench-internals` microbench facade.
#[cfg(any(test, feature = "bench-internals"))]
#[cfg_attr(not(test), allow(dead_code))]
mod prop_harness;
mod root_diagram;
mod root_lorentz;
mod run;
// Post-order stack-machine evaluation strategy (memoizes only shared DAG nodes),
// benchmarked against `run`'s memoize-all forward scan.
mod run_stack;
// `Tree` is used by every arena; `Linearized`/`linearize`/`max_depth` are retained for
// future passes (e.g. benchmarking the forward scan against a stack walk).
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
pub use op::{Const, Node, Op, Sym};
pub use root_diagram::compile_diagram_ast;
pub use root_lorentz::RootLorentzError;
pub use run::{BoundAmplitude, ScratchSpace};
pub use run_stack::BoundAmplitudeStack;
