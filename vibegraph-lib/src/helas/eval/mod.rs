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

mod ast;
mod compile;
mod diagram_eval;
mod error;
mod fold;
mod lower;
mod op;
mod root_diagram;
mod root_lorentz;
mod run;
// `Tree` is used by every arena; `Linearized`/`linearize`/`max_depth` are retained for
// future passes (e.g. benchmarking the forward scan against a stack walk).
#[allow(dead_code)]
mod tree;
mod waveform_slot;

pub use ast::{Ast, ParseAstError};
pub use compile::AmplitudeEvaluator;
pub use error::{CompileError, EvalError, RootDiagramError};
pub use op::{Const, Node, Op, Sym};
pub use root_diagram::compile_diagram_ast;
pub use root_lorentz::RootLorentzError;
pub use run::BoundAmplitude;
