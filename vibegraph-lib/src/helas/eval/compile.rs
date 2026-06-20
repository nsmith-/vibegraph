//! Compilation: DiagramSet + UFOModel → a card-independent [`AmplitudeEvaluator`].
//!
//! This module orchestrates the compile-time phase of amplitude evaluation:
//! - pass 1+2: root each `DiagramView` into a [`DiagramEval`] (topology + Lorentz
//!   structures, still model-bound; see `root_diagram` / `root_lorentz`),
//! - pass 3a: [`lower`] inlines every diagram into one whole-amplitude `Ast<Sym>`,
//! - pass 3b: [`Folded::build`] interns the constants into a card-independent skeleton.
//!
//! The result is independent of both the parameter card and the scalar field `F`.
//! Resolving a card (and choosing `F`) happens in [`AmplitudeEvaluator::bind`], which
//! produces the runtime [`BoundAmplitude`](super::run::BoundAmplitude).

use std::collections::HashSet;

use feyngraph::diagram::view::DiagramView;
use num_traits::FromPrimitive;

use crate::diagrams::DiagramSet;
use crate::helas::repr::Real;
use crate::ufo::couplings::CouplingId;
use crate::ufo::particles::ParticleId;
use crate::ufo::{EvaluatedModel, UFOModel};

use super::fold::Folded;
use super::lower;
use super::root_diagram::{self, RootDiagramError};
use super::root_lorentz::RootLorentzError;
use super::run::BoundAmplitude;

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

/// Errors while building an [`AmplitudeEvaluator`] from a process.
///
/// Holds the model-parameter lookups performed at this layer (particle ids, spins,
/// external-leg counts) on top of the diagram-rooting [`CompileError`].
#[derive(Debug, Clone, thiserror::Error)]
pub enum EvalError {
    /// An external particle name is absent from the UFO model.
    #[error("particle not found in model: {0}")]
    ParticleNotFound(String),
    /// An external leg carries a spin code with no defined helicity states.
    #[error("unsupported external spin code: {0}")]
    UnsupportedSpin(i32),
    /// The process and the compiled AST disagree on the external-leg count.
    #[error("external-leg count mismatch: {0}")]
    TopologyError(String),
    /// Diagram rooting failed.
    #[error(transparent)]
    Compile(#[from] CompileError),
}

/// A compiled representation of a single Feynman diagram.
///
/// Built once from a `DiagramView` + `UFOModel`. The diagram is a rooted
/// [`DiagramEvalTree`](super::root_diagram::DiagramEvalTree): external legs are leaves,
/// internal vertices are off-shell currents wrapped by propagators, and the root
/// contracts into the scalar amplitude.
#[derive(Clone, Debug)]
pub struct DiagramEval {
    /// Number of external legs (determines array indexing for momenta)
    pub n_ext: usize,
    /// Rooted evaluation tree for this diagram
    pub tree: super::root_diagram::DiagramEvalTree,
    /// Symmetry factor: 1 / (vertex_sym × propagator_sym)
    pub symmetry_factor: f64,
    /// ±1 from the diagram's Fermi permutation sign
    pub fermi_sign: i8,
}

impl DiagramEval {
    /// Internal propagator particle ids appearing in this diagram (one per
    /// `Propagate` node). Used to characterize a diagram by its propagator content.
    #[cfg(test)]
    pub fn propagator_particles(&self) -> impl Iterator<Item = ParticleId> + '_ {
        use super::root_diagram::EvalNode;
        use super::tree::Tree;
        self.tree.iter().filter_map(|id| match self.tree.value(id) {
            EvalNode::Propagate { info, .. } => Some(info.id),
            _ => None,
        })
    }
}

impl std::fmt::Display for DiagramEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Diagram(external legs {}): {}", self.n_ext, self.tree)
    }
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

/// Compile all diagrams from a DiagramSet into rooted [`DiagramEval`]s.
///
/// For each diagram, recursively walks from an arbitrary root vertex to build a
/// directed evaluation tree. External legs become leaves; internal vertices emit an
/// off-shell current + propagator pair; the root emits the amplitude contraction.
pub fn compile_diagram_ast(
    set: &DiagramSet,
    model: &UFOModel,
) -> Result<Vec<DiagramEval>, CompileError> {
    set.diagrams
        .views()
        .map(|view| compile_single_diagram(&view, model))
        .collect()
}

/// Compiled amplitude evaluator for a whole process (card- and `F`-independent).
///
/// Built once into a [`Folded`] skeleton (pass 1+2 rooting → `lower` → `fold`).
/// [`AmplitudeEvaluator::bind`] resolves a `&EvaluatedModel` at a chosen scalar
/// precision `F` into a runtime [`BoundAmplitude`], so the same evaluator works with
/// any parameter card and any precision.
#[derive(Debug)]
pub struct AmplitudeEvaluator {
    /// Folded whole-amplitude AST + constant-pool specs.
    folded: Folded,
    /// Number of external particles
    n_ext: usize,
    /// Number of incoming external particles
    n_in: usize,
    /// Number of diagrams folded into the amplitude
    n_diagrams: usize,
    /// External particle ids in process order (incoming first, then outgoing)
    ext_particle_ids: Vec<ParticleId>,
    /// All valid helicity combinations (precomputed)
    helicities: Vec<Vec<i32>>,
}

impl AmplitudeEvaluator {
    /// Compile from a DiagramSet + UFO model (symbolic, no param card needed).
    pub fn compile(set: &DiagramSet, model: &UFOModel) -> Result<Self, EvalError> {
        let ext_particle_names = set
            .particles_in
            .iter()
            .chain(set.particles_out.iter())
            .cloned()
            .collect::<Vec<_>>();

        let ext_particle_ids = ext_particle_names
            .iter()
            .map(|name| {
                model
                    .particle_id(name)
                    .ok_or_else(|| EvalError::ParticleNotFound(name.clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let diagrams = compile_diagram_ast(set, model)?;
        let n_ext = ext_particle_ids.len();

        // Compile phase should preserve process external-leg count consistency.
        if let Some(ast) = diagrams.first() {
            if ast.n_ext != n_ext {
                return Err(EvalError::TopologyError(format!(
                    "process has {n_ext}, AST has {}",
                    ast.n_ext
                )));
            }
        }

        let helicity_states = ext_particle_ids
            .iter()
            .map(|&pid| helicity_states_for_spin(model.particle(pid).spin))
            .collect::<Result<Vec<_>, _>>()?;
        let helicities = cartesian_helicity_product(&helicity_states);

        // Pass 3: inline every diagram into one whole-amplitude AST (structure pass is
        // a no-op for now), then intern the constants into the folded skeleton.
        let n_diagrams = diagrams.len();
        let symbolic = lower::optimize(lower::lower(&diagrams));
        let folded = Folded::build(&symbolic);

        Ok(Self {
            folded,
            n_ext,
            n_in: set.particles_in.len(),
            n_diagrams,
            ext_particle_ids,
            helicities,
        })
    }

    /// Resolve a parameter card at scalar precision `F` into a runtime evaluator with
    /// all couplings/masses/widths baked into its constant pools.
    pub fn bind<F: Real + FromPrimitive>(
        &self,
        evaluated: &EvaluatedModel,
    ) -> BoundAmplitude<'_, F> {
        let (consts_c, consts_f) = self.folded.pools::<F>(evaluated);
        BoundAmplitude::new(self, consts_c, consts_f)
    }

    /// The folded whole-amplitude skeleton (arena + pool specs).
    pub(super) fn folded(&self) -> &Folded {
        &self.folded
    }

    /// Return the number of external legs.
    pub fn n_ext(&self) -> usize {
        self.n_ext
    }

    /// Return the number of incoming external legs.
    pub fn n_in(&self) -> usize {
        self.n_in
    }

    /// Return external particle ids in process order (incoming, then outgoing).
    pub fn external_particles(&self) -> &[ParticleId] {
        &self.ext_particle_ids
    }

    /// Return the number of compiled diagrams.
    pub fn n_diagrams(&self) -> usize {
        self.n_diagrams
    }

    /// Return the valid helicity combinations.
    pub fn helicities(&self) -> &[Vec<i32>] {
        &self.helicities
    }

    /// Return all coupling and particle ids needed to evaluate the amplitude.
    ///
    /// Can be used for prefetching from EvaluatedModel if desired.
    pub fn coupling_particle_ids(&self) -> (HashSet<CouplingId>, HashSet<ParticleId>) {
        (
            self.folded.coupling_ids().collect(),
            self.folded.particle_ids().collect(),
        )
    }
}

fn helicity_states_for_spin(spin_code: i32) -> Result<Vec<i32>, EvalError> {
    // UFO spin code convention is 2s+1 with negative values reserved for ghosts.
    match spin_code.abs() {
        1 => Ok(vec![0]),               // scalar
        2 => Ok(vec![-1, 1]),           // fermion
        3 => Ok(vec![-1, 0, 1]),        // vector
        5 => Ok(vec![-2, -1, 0, 1, 2]), // spin-2 (future-proof)
        other => Err(EvalError::UnsupportedSpin(other)),
    }
}

fn cartesian_helicity_product(states: &[Vec<i32>]) -> Vec<Vec<i32>> {
    let mut out = vec![Vec::new()];
    for leg_states in states {
        let mut next = Vec::with_capacity(out.len() * leg_states.len());
        for partial in &out {
            for &h in leg_states {
                let mut combo = partial.clone();
                combo.push(h);
                next.push(combo);
            }
        }
        out = next;
    }
    out
}
