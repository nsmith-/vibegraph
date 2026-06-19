//! Runtime amplitude evaluator with compiled AST.
//!
//! This module provides `AmplitudeEvaluator`, which compiles a `DiagramSet` once
//! and then evaluates amplitudes rapidly for any phase-space point and helicity
//! configuration.
//!
//! The evaluator consists of:
//! - `ast.rs` — Data structures (DiagramEval, VertexInfo, leg/prop descriptors)
//! - `compile.rs` — Compile phase: DiagramSet + UFOModel → DiagramEval
//! - `root_diagram.rs` — Two-pass diagram rooting: DiagramView → DiagramEvalTree
//! - `root_lorentz.rs` — Vertex dispatch: LorentzTerm → rooted LorentzEvalTree
//! - `tree.rs` — Generic tree trait + linearization onto a stack machine
//! - `run.rs` — Evaluation phase: DiagramEval × momenta × helicities → amplitude

mod ast;
mod compile;
mod root_diagram;
mod root_lorentz;
mod run;

// TODO(Step 4): drop once the remaining tree helpers (max_depth, len) gain users.
#[allow(dead_code)]
mod tree;

mod waveform_slot;

pub use compile::{compile_diagram_ast, CompileError};
pub use root_diagram::RootDiagramError;
pub use root_lorentz::RootLorentzError;
pub use run::{AmplitudeEvaluator, EvalError};
