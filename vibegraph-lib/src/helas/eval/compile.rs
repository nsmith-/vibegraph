//! Compilation: DiagramSet + UFOModel → a card-independent [`AmplitudeEvaluator`].
//!
//! This module orchestrates the compile-time phase of amplitude evaluation:
//! - pass 1+2: root each `DiagramView` into a [`DiagramEval`] (topology + Lorentz
//!   structures, still model-bound; see `root_diagram` / `root_lorentz`),
//! - pass 3a: [`lower`] inlines every diagram into one whole-amplitude `Ast<Sym>`,
//! - pass 3b: [`Folded::build`] interns the constants into a card-independent skeleton.
//!
//! The result is independent of both the parameter card and the scalar field `F`.
//! Resolving a card (and choosing `F`) happens in
//! [`BoundAmplitude::bind`](super::run::BoundAmplitude::bind), which produces the
//! runtime [`BoundAmplitude`](super::run::BoundAmplitude).

use std::collections::HashSet;

use crate::diagrams::DiagramSet;
use crate::ufo::couplings::CouplingId;
use crate::ufo::particles::ParticleId;
use crate::ufo::UFOModel;

use super::error::EvalError;
use super::fold::Folded;
use super::lower;
use super::root_diagram::compile_diagram_ast;

/// Compiled amplitude evaluator for a whole process (card- and `F`-independent).
///
/// Built once into a [`Folded`] skeleton (pass 1+2 rooting → `lower` → `fold`).
/// [`BoundAmplitude::bind`](super::run::BoundAmplitude::bind) resolves a
/// `&EvaluatedModel` at a chosen scalar precision `F` into a runtime
/// [`BoundAmplitude`](super::run::BoundAmplitude), so the same evaluator works with any
/// parameter card and any precision.
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
