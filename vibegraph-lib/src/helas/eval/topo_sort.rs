//! Topological ordering for diagram compilation.
//!
//! This module converts an undirected diagram graph into a directed acyclic graph (DAG)
//! by choosing an arbitrary root vertex and directing all edges toward it. This transforms
//! the diagram structure into an evaluation tree where child vertices feed their
//! off-shell currents upward to parents.
//!
//! The key insight: an undirected Feynman diagram has no intrinsic evaluation order,
//! but choosing a root makes it a tree. We do this by starting at any external vertex
//! and recursively processing unvisited neighbors. Each vertex becomes an evaluation step
//! that reads inputs from its children and produces either an off-shell current (if not root)
//! or the final amplitude (if root).

use std::collections::HashSet;

use feyngraph::diagram::view::{DiagramView, LegView, VertexView};
use itertools::Either;

use crate::helas::eval::ast::{ExtLegInfo, PropInfo, VertexInfo};
use crate::ufo::UFOModel;

use super::ast::{DiagramAst, EvalStep};
use super::compile::CompileError;

/// Topological sorting context for a single diagram.
struct TopoContext<'a> {
    /// Model for looking up particle and vertex details not available from feyngraph
    model: &'a UFOModel,
    /// Next available slot for internal propagators
    next_slot: usize,
    /// Steps to emit (in order)
    steps: Vec<EvalStep>,
    /// Set of processed vertices
    processed_vertices: HashSet<usize>,
}

impl<'a> TopoContext<'a> {
    /// Create a new context for topological ordering.
    fn new(model: &'a UFOModel) -> Self {
        TopoContext {
            model: model,
            next_slot: 0,
            steps: Vec::new(),
            processed_vertices: HashSet::new(),
        }
    }

    fn next_slot(&mut self) -> usize {
        let slot = self.next_slot;
        self.next_slot += 1;
        slot
    }

    fn make_externalwf(&mut self, leg: LegView) -> EvalStep {
        let particle = leg.particle();
        let particle_id = self
            .model
            .particle_id(particle.name())
            .expect("particle not found");

        EvalStep::ExternalWf {
            info: ExtLegInfo {
                id: particle_id,
                leg_idx: leg.index(),
            },
            output_slot: self.next_slot(),
        }
    }

    /// Recursively walk the diagram tree from a root vertex.
    ///
    /// Process all propagators attached to `vtx`. For each:
    /// - If external leg: emit ExternalWf step and store input slot
    /// - If internal (unvisited): recurse to the other vertex and store its output
    /// - If internal (visited): skip (came from here via result_leg_idx)
    ///
    /// Then emit the appropriate step for this vertex:
    /// - If not root (result_leg_idx is Some): emit OffShellCurrent + Propagate
    /// - If root (result_leg_idx is None): emit ContractAmplitude
    ///
    /// Returns the step that produces this vertex's output (OffShellCurrent's Propagate
    /// for non-root, or ContractAmplitude for root). This return value is accumulated
    /// into parent vertices' input slots.
    fn walk_vertex(&mut self, vtx: &VertexView, result_leg_idx: Option<usize>) -> EvalStep {
        self.processed_vertices.insert(vtx.id());
        let mut slots = vec![];
        for (idx, prop) in vtx.propagators_ordered().enumerate() {
            let is_upstream = result_leg_idx.map_or(false, |ir| ir == idx);
            match (is_upstream, prop) {
                (false, Either::Left(leg)) => {
                    let step = self.make_externalwf(leg);
                    slots.push(step.output_slot());
                    self.steps.push(step);
                }
                (false, Either::Right(prop)) => {
                    prop.vertices().enumerate().for_each(|(vidx, next_vtx)| {
                        // Either it is the vertex we are at or the next one the propagator goes to
                        if !self.processed_vertices.contains(&next_vtx.id()) {
                            let this_prop = prop.ray_index_ordered(vidx);
                            let step = self.walk_vertex(&next_vtx, Some(this_prop));
                            slots.push(step.output_slot());
                            self.steps.push(step);
                        }
                    });
                }
                (true, Either::Left(_)) => {
                    panic!("An external leg cannot be the result leg for a vertex");
                }
                (true, Either::Right(_)) => {
                    // This is the propagator we came from
                }
            }
        }

        let model_vertex_id = self
            .model
            .vertex_id(vtx.interaction().name())
            .expect("no vertex in UFO");

        // After processing all connections, emit the appropriate step for this vertex
        if let Some(result_leg_idx) = result_leg_idx {
            // This is an internal vertex, so push OffShellCurrent and return Propagate step
            let contraction = EvalStep::OffShellCurrent {
                info: VertexInfo::from_ufo(self.model, model_vertex_id, Some(result_leg_idx)),
                input_slots: slots,
                output_slot: self.next_slot(),
            };
            let contraction_output_slot = contraction.output_slot();
            self.steps.push(contraction);
            // return propagation step
            EvalStep::Propagate {
                info: PropInfo {
                    id: self.model.vertex_def(model_vertex_id).particles[result_leg_idx],
                    momentum_coeffs: vec![], // TODO: determine momentum coefficients from diagram structure
                },
                input_slot: contraction_output_slot,
                output_slot: self.next_slot(),
            }
        } else {
            // This is the top vertex, so emit ContractAmplitude step
            let step = EvalStep::ContractAmplitude {
                info: VertexInfo::from_ufo(self.model, model_vertex_id, None),
                input_slots: slots,
                output_slot: self.next_slot(),
            };
            self.steps.push(step.clone());
            step
        }
    }
}

/// Compile a single diagram into an evaluable AST.
///
/// Recursively walk from an arbitrary root vertex, building a directed evaluation tree.
/// External legs produce slots 0..n_ext (immediately available). Internal vertices are
/// processed depth-first as children feed their outputs upward. The root vertex emits
/// the final ContractAmplitude step.
///
/// # Arguments
/// * `view` — DiagramView with vertices and propagators
/// * `model` — UFO model for vertex/particle/coupling lookups
///
/// # Returns
/// A DiagramAst with steps in evaluation order, ready to run against phase-space points.
pub fn compile_single_diagram(
    view: &DiagramView,
    model: &UFOModel,
) -> Result<DiagramAst, CompileError> {
    let n_ext = view.legs().count();

    let mut ctx = TopoContext::new(model);

    // Choose an arbitrary root vertex (the first one) and walk the tree from there.
    // External legs are handled as we encounter them, and internal propagators
    // trigger recursive walks to unvisited vertices. The final step emitted
    // (from the root) is a ContractAmplitude.
    let start_vertex = view.vertex(0);

    let amplitude_slot = match ctx.walk_vertex(&start_vertex, None) {
        EvalStep::ContractAmplitude {
            output_slot: result_slot,
            ..
        } => result_slot,
        _ => {
            return Err(CompileError::TopologyError(
                "Topological sort did not produce a ContractAmplitude step".to_string(),
            ));
        }
    };
    Ok(DiagramAst {
        n_ext,
        n_slots: ctx.next_slot,
        steps: ctx.steps,
        amplitude_slot,
        symmetry_factor: 1.0 / view.symmetry_factor() as f64,
        fermi_sign: view.sign(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::tests::generate;

    #[test]
    fn test_walk() {
        let model = crate::diagrams::tests::sm_model();
        let sets = generate("e+ e- > mu+ mu-");
        for set in sets {
            for view in set.diagrams.views() {
                println!("Testing diagram {}", view);
                let ast = compile_single_diagram(&view, model).expect("compilation failed");
                println!("Generated AST: {:#?}", ast);
            }
        }
    }
}
