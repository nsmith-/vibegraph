//! Diagram compilation: DiagramSet + UFOModel → DiagramEval(s)
//!
//! This module implements the compile-time phase of the amplitude evaluator:
//! converting feyngraph DiagramView objects into evaluable DiagramEval trees
//! that can be efficiently evaluated at runtime against phase-space points.

use crate::diagrams::DiagramSet;
use crate::ufo::UFOModel;

use super::ast::DiagramEval;
use super::root_diagram;

/// Errors during AST compilation.
#[derive(Debug, Clone)]
pub enum CompileError {
    /// Vertex not found in model
    VertexNotFound(String),
    /// Lorentz structure not found
    LorentzNotFound(String),
    /// Coupling not found
    CouplingNotFound(String),
    /// Invalid topological ordering (e.g., cyclic dependencies)
    TopologyError(String),
    /// Unsupported vertex type or coupling
    UnsupportedVertex(String),
    /// Particle not found in model
    ParticleNotFound(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::VertexNotFound(s) => write!(f, "Vertex not found: {}", s),
            CompileError::LorentzNotFound(s) => write!(f, "Lorentz structure not found: {}", s),
            CompileError::CouplingNotFound(s) => write!(f, "Coupling not found: {}", s),
            CompileError::TopologyError(s) => write!(f, "Topology error: {}", s),
            CompileError::UnsupportedVertex(s) => write!(f, "Unsupported vertex: {}", s),
            CompileError::ParticleNotFound(s) => write!(f, "Particle not found: {}", s),
        }
    }
}

impl std::error::Error for CompileError {}

/// Compile all diagrams from a DiagramSet into optimized ASTs.
///
/// For each diagram, recursively walks from an arbitrary root vertex to build
/// a directed evaluation tree. External legs become immediate slots (0..n_ext).
/// Internal vertices are processed depth-first, with each non-root vertex emitting
/// an OffShellCurrent + Propagate pair, and the root emitting a ContractAmplitude.
///
/// # Arguments
/// * `set` — The diagram set to compile
/// * `model` — The UFO model (needed for particle/vertex/coupling lookups)
///
/// # Returns
/// Vector of compiled `DiagramEval`, one per diagram in the set
pub fn compile_diagram_ast(
    set: &DiagramSet,
    model: &UFOModel,
) -> Result<Vec<DiagramEval>, CompileError> {
    set.diagrams
        .views()
        .map(|view| root_diagram::compile_single_diagram(&view, model))
        .collect()
}
