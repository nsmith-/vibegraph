//! Runtime amplitude evaluator with a compiled, interned AST.
//!
//! `AmplitudeEvaluator` compiles a `DiagramSet` once and then evaluates amplitudes
//! rapidly for any phase-space point and helicity configuration.
//!
//! Pipeline:
//! - `root_diagram.rs` / `root_lorentz.rs` — pass 1+2: `DiagramView` → rooted
//!   `DiagramEvalTree` (symbolic, model-bound; node payloads in `diagram_eval.rs`).
//! - `op.rs` — the unified node language (`Op` + `Sym`/`Const` leaves).
//! - `ast.rs` — the unified `Ast<T>` arena (CSR children, s-expr I/O).
//! - `lower.rs` — pass 3a: inline all `DiagramEval`s into one `Ast<Sym>` (+ `optimize`
//!   no-op, the egglog hook).
//! - `fold.rs` — pass 3b: intern couplings/masses/widths/coeffs into deduped pool specs
//!   → card-independent `Ast<Const>`.
//! - `compile.rs` — orchestrates passes 1–3 into a card-independent `AmplitudeEvaluator`;
//!   `bind`ing an `EvaluatedModel` resolves the pools into a runtime `BoundAmplitude`.
//! - `run.rs` — `BoundAmplitude`: single forward-pass evaluator over the folded arena.

mod ast;
mod compile;
mod diagram_eval;
mod fold;
mod lower;
mod op;
mod root_diagram;
mod root_lorentz;
mod run;
// `Tree` is used by every arena; `Linearized`/`linearize`/`max_depth` are retained for
// the test-only Lorentz cross-check path and future passes.
#[allow(dead_code)]
mod tree;
mod waveform_slot;

pub use ast::{Ast, ParseAstError};
pub use compile::{compile_diagram_ast, AmplitudeEvaluator, CompileError, EvalError};
pub use op::{Const, Node, Op, Sym};
pub use root_diagram::RootDiagramError;
pub use root_lorentz::RootLorentzError;
pub use run::BoundAmplitude;
