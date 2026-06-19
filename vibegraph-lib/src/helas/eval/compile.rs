//! Diagram compilation: DiagramSet + UFOModel → DiagramEval(s)
//!
//! This module implements the compile-time phase of the amplitude evaluator:
//! converting feyngraph DiagramView objects into evaluable DiagramEval trees
//! that can be efficiently evaluated at runtime against phase-space points.

use feyngraph::diagram::view::DiagramView;

use crate::diagrams::DiagramSet;
use crate::ufo::UFOModel;

use super::ast::DiagramEval;
use super::root_diagram::{self, RootDiagramError};
use super::root_lorentz::RootLorentzError;

/// Errors during diagram rooting (the compile phase).
///
/// The two rooting passes each contribute a subtype: [`RootDiagramError`] from
/// walking the topology, and [`RootLorentzError`] from rooting each vertex's
/// Lorentz structure.
#[derive(Debug, Clone, thiserror::Error)]
pub enum CompileError {
    /// Pass 1: walking the diagram topology and interning model ids.
    #[error(transparent)]
    RootDiagram(#[from] RootDiagramError),
    /// Pass 2: rooting a vertex's Lorentz structure into a contraction tree.
    #[error(transparent)]
    RootVertex(#[from] RootLorentzError),
}

/// Compile a single diagram into an evaluable [`DiagramEval`].
///
/// Roots the diagram into its evaluation tree (topology + Lorentz structures) and
/// attaches the per-diagram metadata (external-leg count, symmetry factor, and the
/// fermion-flow sign, including the initial-state spine correction).
fn compile_single_diagram(
    view: &DiagramView,
    model: &UFOModel,
) -> Result<DiagramEval, CompileError> {
    Ok(DiagramEval {
        n_ext: view.legs().count(),
        tree: root_diagram::root_tree(view, model)?,
        symmetry_factor: 1.0 / view.symmetry_factor() as f64,
        fermi_sign: view.sign() * root_diagram::initial_state_spine_sign(view, model),
    })
}

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
        .map(|view| compile_single_diagram(&view, model))
        .collect()
}
