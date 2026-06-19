//! Root a diagram at a given vertex
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
use crate::helas::repr::numbers::Charge;
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

        let model_particle = self.model.particle(particle_id);
        // Check that feyngraph's is_anti flag is consistent with the UFO model.
        // Use pdg_code < 0 (not charge sign) because up-type quarks have positive
        // charge yet are particles (is_anti=false), breaking the charge-based check.
        assert_eq!(
            particle.is_anti(),
            model_particle.pdg_code < 0,
            "Antiparticle mismatch for {}: feyngraph is_anti={} but UFOModel pdg_code={}",
            particle.name(),
            particle.is_anti(),
            model_particle.pdg_code
        );

        EvalStep::ExternalWf {
            info: ExtLegInfo {
                id: particle_id,
                leg_idx: leg.index(),
                charge: match particle.is_anti() {
                    true => Charge::Antiparticle,
                    false => Charge::Particle,
                },
                spin: model_particle.spin,
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
                    // This is the result (output) leg — the propagator we came from.
                    // It has no input wavefunction yet, but we must keep `slots`
                    // aligned with the vertex's leg ordering so the rooted Lorentz
                    // structure's 1-based Leg(i) references resolve correctly even
                    // when the output is not the last leg (e.g. an off-shell fermion
                    // current). This placeholder is never read: an off-shell current
                    // never references its own output leg as an input.
                    slots.push(usize::MAX);
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

/// Trace the fermion line that enters `start_vtx` at ordered ray `in_ray`,
/// following spinor connectivity until it reaches an external leg.
///
/// Connectivity comes from *our* recomputed `spin_map` (UFOModel
/// `LorentzStructure`), indexed by the vertex's ordered rays — which align with
/// feyngraph's `propagators_ordered` because we feed feyngraph that same particle
/// ordering. Returns the external leg index where the line terminates and whether
/// it passed through at least one internal propagator (i.e. is an off-shell spine).
fn trace_fermion_line(
    view: &DiagramView,
    model: &UFOModel,
    start_vtx_id: usize,
    in_ray: usize,
) -> (usize, bool) {
    let mut vtx_id = start_vtx_id;
    let mut in_ray = in_ray;
    let mut passed_internal = false;
    // Tree diagrams terminate; the bound only guards against pathological loops.
    for _ in 0..1024 {
        let vtx = view.vertex(vtx_id);
        let vid = model
            .vertex_id(vtx.interaction().name())
            .expect("vertex in UFO");
        let lid = model.vertex_def(vid).lorentz[0];
        let out_ray = model.lorentz_struct(lid).spin_map[in_ray] as usize;
        // Extract Copy values so the `propagators_ordered` borrow of `vtx` is
        // released before the next iteration rebinds it from `view`.
        let next: (usize, usize) = match vtx
            .propagators_ordered()
            .nth(out_ray)
            .expect("spin_map ray index in range")
        {
            Either::Left(leg) => return (leg.index(), passed_internal),
            Either::Right(prop) => {
                let n = if prop.vertex(0).id() == vtx_id { 1 } else { 0 };
                (prop.vertex(n).id(), prop.ray_index_ordered(n))
            }
        };
        passed_internal = true;
        vtx_id = next.0;
        in_ray = next.1;
    }
    panic!("fermion line trace did not terminate");
}

/// Relative fermion sign that feyngraph's connectivity-based `view.sign()` omits.
///
/// MadGraph assigns a relative −1 to a diagram whose off-shell fermion **spine**
/// is the line joining the two **initial-state** fermions: in MadGraph's
/// all-outgoing convention that pair is crossed, and chaining it as a propagator
/// (rather than closing it at a single vertex) picks up the crossing sign. Across
/// such diagrams `view.sign()` is uniform, so we apply the correction here.
///
/// Detected structurally by tracing each external fermion line: flip when a line
/// connects two incoming legs through at least one internal propagator. Validated
/// against MadGraph per-diagram amplitudes for e+e-→μ+μ-τ+τ- and
/// u u~→c c~ e+e- μ+μ- (QCD=0).
fn initial_state_spine_sign(view: &DiagramView, model: &UFOModel) -> i8 {
    let n_in = view.incoming().count();
    let mut visited: HashSet<usize> = HashSet::new();
    let mut sign: i8 = 1;
    for leg in view.incoming().chain(view.outgoing()) {
        let li = leg.index();
        if !leg.particle().is_fermi() || !visited.insert(li) {
            continue;
        }
        let (other, passed_internal) =
            trace_fermion_line(view, model, leg.vertex().id(), leg.ray_index_ordered());
        visited.insert(other);
        if passed_internal && li < n_in && other < n_in {
            sign = -sign;
        }
    }
    sign
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
        fermi_sign: view.sign() * initial_state_spine_sign(view, model),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, DiagramSet, ParsingOptions};
    use crate::ufo::UFOModel;
    use std::sync::OnceLock;

    static SM_MODEL: OnceLock<UFOModel> = OnceLock::new();
    fn sm_model() -> &'static UFOModel {
        SM_MODEL.get_or_init(|| {
            let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
            let path = std::path::Path::new(&manifest).join("../research/refs/mg5amcnlo/models/sm");
            UFOModel::load(&path, None).expect("SM UFO not found")
        })
    }
    fn generate(process: &str) -> Vec<DiagramSet> {
        let opts = ParsingOptions::default();
        let card = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
        generate_from_proc_card(&card, sm_model()).unwrap()
    }

    #[test]
    fn test_walk() {
        let model = sm_model();
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
