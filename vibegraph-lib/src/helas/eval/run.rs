//! Runtime amplitude evaluation: DiagramAst × momenta × helicities → amplitude

use crate::diagrams::DiagramSet;
use crate::helas::repr::lorentz::LorentzVector;
use crate::helas::repr::{Real, C};
use crate::ufo::{EvaluatedModel, UFOModel};

use super::ast::DiagramAst;
use super::compile::CompileError;

/// Compiled amplitude evaluator for all diagrams of a process.
///
/// The AST is built once from `&UFOModel`; coupling values are resolved at eval time
/// from `&EvaluatedModel` so the same evaluator works with any param card.
pub struct AmplitudeEvaluator {
    /// One compiled AST per diagram
    diagram_asts: Vec<DiagramAst>,
    /// Number of external particles
    n_ext: usize,
    /// All valid helicity combinations (precomputed)
    helicities: Vec<Vec<i32>>,
}

impl AmplitudeEvaluator {
    /// Compile from a DiagramSet + UFO model (symbolic, no param card needed).
    ///
    /// # Arguments
    /// * `set` — The diagram set for the process
    /// * `model` — The UFO model (used for topology and particle properties)
    ///
    /// # Returns
    /// A compiled evaluator, or a compilation error.
    pub fn compile(set: &DiagramSet, model: &UFOModel) -> Result<Self, CompileError> {
        // TODO: Implement compilation
        // - Extract external legs from the diagram set
        // - Compile each diagram
        // - Generate valid helicity combinations
        Err(CompileError::TopologyError(
            "AmplitudeEvaluator::compile not yet implemented".to_string(),
        ))
    }

    /// Evaluate |M|² summed over all helicities.
    ///
    /// # Arguments
    /// * `momenta` — External 4-momenta [E, px, py, pz] in order:
    ///   incoming legs first, then outgoing legs.
    /// * `evaluated` — Coupling constants resolved from a param card
    ///
    /// # Returns
    /// Σ_{helicities} |M|² (summed, not averaged)
    pub fn eval_m2<F: Real>(&self, momenta: &[LorentzVector<F>], evaluated: &EvaluatedModel) -> F {
        // TODO: Implement evaluation loop
        // - Iterate over all valid helicity combinations
        // - For each combo, call eval_amplitude
        // - Accumulate |M|² summed over all helicities
        F::zero()
    }

    /// Evaluate the complex amplitude M for a single helicity configuration.
    ///
    /// # Arguments
    /// * `momenta` — External 4-momenta
    /// * `helicities` — Helicity configuration [nhel_1, nhel_2, ..., nhel_n]
    /// * `evaluated` — Coupling constants
    ///
    /// # Returns
    /// The complex amplitude M (sum of all diagrams with the given kinematics/helicities)
    pub fn eval_amplitude<F: Real>(
        &self,
        momenta: &[LorentzVector<F>],
        helicities: &[i32],
        evaluated: &EvaluatedModel,
    ) -> C<F> {
        // TODO: Implement amplitude evaluation
        // - Iterate over all diagrams
        // - For each diagram, evaluate its AST
        // - Coherently sum amplitudes (no squaring yet)
        C::new(F::zero(), F::zero())
    }

    /// Return the number of external legs.
    pub fn n_ext(&self) -> usize {
        self.n_ext
    }

    /// Return the number of compiled diagrams.
    pub fn n_diagrams(&self) -> usize {
        self.diagram_asts.len()
    }

    /// Return the valid helicity combinations.
    pub fn helicities(&self) -> &[Vec<i32>] {
        &self.helicities
    }
}
