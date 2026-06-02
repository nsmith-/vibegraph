//! Diagram compilation: DiagramSet + UFOModel → DiagramAst(s)
//!
//! This module implements the compile-time phase of the amplitude evaluator:
//! converting feyngraph DiagramView objects into optimized DiagramAst structures
//! that can be efficiently evaluated at runtime against phase-space points.

use crate::diagrams::DiagramSet;
use crate::ufo::UFOModel;

use super::ast::DiagramAst;

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
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::VertexNotFound(s) => write!(f, "Vertex not found: {}", s),
            CompileError::LorentzNotFound(s) => write!(f, "Lorentz structure not found: {}", s),
            CompileError::CouplingNotFound(s) => write!(f, "Coupling not found: {}", s),
            CompileError::TopologyError(s) => write!(f, "Topology error: {}", s),
            CompileError::UnsupportedVertex(s) => write!(f, "Unsupported vertex: {}", s),
        }
    }
}

impl std::error::Error for CompileError {}

/// Compile all diagrams from a DiagramSet into optimized ASTs.
///
/// # Algorithm
///
/// For each diagram in the set:
///
/// 1. **Extract external legs** via `view.legs()`:
///    - Determine spin, mass, and charge from UFOModel
///    - Create ExtLegInfo for each (incoming legs first 0..n_in, then outgoing n_in..n_ext)
///
/// 2. **Extract internal propagators** via `view.propagators()`:
///    - For each propagator, determine which external legs contribute to its momentum
///    - Create PropInfo with momentum_coeffs (±1 for each external leg inflow)
///
/// 3. **Perform topological ordering**:
///    - Build a dependency graph: vertex → (set of propagators it depends on)
///    - Mark all external legs as "available" (slot 0..n_ext)
///    - Iteratively find a vertex where all but one attached propagators are available
///    - Emit `OffShellCurrent` step for that vertex (reads from input_slots, writes to output_slot)
///    - Mark the output propagator as available
///    - Repeat until no new vertices found
///    - The final vertex (with all legs available) emits a `ContractAmplitude` step
///
/// 4. **Allocate slots**:
///    - Slots 0..n_ext: external wavefunctions
///    - Slots n_ext..: internal propagators (in the order they become available)
///
/// 5. **Emit steps** in topological order into the DiagramAst
///
/// # Arguments
/// * `set` — The diagram set to compile
/// * `model` — The UFO model (needed for particle/vertex/coupling lookups)
///
/// # Returns
/// Vector of compiled `DiagramAst`, one per diagram in the set
pub fn compile_diagram_ast(
    set: &DiagramSet,
    model: &UFOModel,
) -> Result<Vec<DiagramAst>, CompileError> {
    // TODO: Implement compilation algorithm
    //
    // Pseudo-code structure:
    // ```rust
    // let mut asts = Vec::new();
    // for view in set.diagrams.views() {
    //     let n_in = set.particles_in.len();
    //     let n_out = set.particles_out.len();
    //     let n_ext = n_in + n_out;
    //
    //     // Step 1: Extract external legs
    //     let mut ext_legs = Vec::new();
    //     for (leg_idx, leg_view) in view.legs().enumerate() {
    //         let particle = leg_view.particle();
    //         let particle_id = model.particle_id(particle.name())?;
    //         let particle_def = model.particle_def(particle_id);
    //         ext_legs.push(ExtLegInfo {
    //             leg_idx,
    //             spin: particle_def.spin,
    //             mass: particle_def.mass,
    //             is_incoming: leg_idx < n_in,
    //             particle_name: particle.name().to_string(),
    //         });
    //     }
    //
    //     // Step 2: Extract propagators
    //     let mut props = Vec::new();
    //     for prop_view in view.propagators() {
    //         let particle = prop_view.particle();
    //         let particle_id = model.particle_id(particle.name())?;
    //         let particle_def = model.particle_def(particle_id);
    //         // TODO: Compute momentum_coeffs by analyzing which external legs feed this propagator
    //         props.push(PropInfo {
    //             spin: particle_def.spin,
    //             mass: particle_def.mass,
    //             width: particle_def.width,
    //             momentum_coeffs: vec![/* TODO */],
    //         });
    //     }
    //
    //     // Step 3-5: Topological ordering + slot allocation + step emission
    //     // (Implementation pending)
    //
    //     asts.push(DiagramAst::new(
    //         n_ext,
    //         n_slots,
    //         steps,
    //         amplitude_slot,
    //         symmetry_factor,
    //         fermi_sign,
    //     ));
    // }
    // Ok(asts)
    // ```
    //
    Err(CompileError::TopologyError(
        "compile_diagram_ast: implementation pending feyngraph API integration".to_string(),
    ))
}
