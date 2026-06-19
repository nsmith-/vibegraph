//! Runtime amplitude evaluator with compiled AST.
//!
//! This module provides `AmplitudeEvaluator`, which compiles a `DiagramSet` once
//! and then evaluates amplitudes rapidly for any phase-space point and helicity
//! configuration.
//!
//! The evaluator consists of:
//! - `ast.rs` — AST data structures (DiagramAst, EvalStep, WaveformSlot, descriptors)
//! - `compile.rs` — Compile phase: DiagramView + UFOModel → DiagramAst (leg/prop extraction)
//! - `topo_sort.rs` — Topological ordering: dependency graph → evaluation steps
//! - `dispatch.rs` — Vertex dispatch: VertexInfo + slots → result
//! - `run.rs` — Evaluation phase: DiagramAst × momenta × helicities → amplitude

mod ast;
mod compile;
mod root_diagram;
mod root_lorentz;
mod run;
mod tree;
mod waveform_slot;

pub use ast::DiagramAst;
pub use compile::{compile_diagram_ast, CompileError};
pub use run::AmplitudeEvaluator;

/// Errors that can occur during AST compilation or evaluation.
#[derive(Debug, Clone)]
pub enum EvalError {
    /// Compilation error
    Compile(CompileError),
    /// Error during evaluation (e.g., invalid slot access)
    Runtime(String),
}

impl From<CompileError> for EvalError {
    fn from(e: CompileError) -> Self {
        EvalError::Compile(e)
    }
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::Compile(e) => write!(f, "Compilation error: {:?}", e),
            EvalError::Runtime(s) => write!(f, "Runtime error: {}", s),
        }
    }
}

impl std::error::Error for EvalError {}
