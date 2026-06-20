//! The eval-pass error tree.
//!
//! A leaf module so the rooting/compile passes can depend on it one-way: the two
//! rooting passes ([`RootDiagramError`] from walking the topology, [`RootLorentzError`]
//! from rooting a vertex's Lorentz structure) aggregate into [`CompileError`], and the
//! process-level [`EvalError`] wraps that plus the model lookups done while compiling.

use super::root_lorentz::RootLorentzError;

/// Errors from Pass 1: walking the diagram topology and interning model ids.
#[derive(Clone, Debug, thiserror::Error)]
pub enum RootDiagramError {
    /// A leg's particle name is absent from the UFO model.
    #[error("particle not found in model: {0}")]
    ParticleNotFound(String),
    /// A vertex's interaction name is absent from the UFO model.
    #[error("vertex not found in model: {0}")]
    VertexNotFound(String),
    /// feyngraph's is_anti flag disagrees with the model's pdg-code sign.
    #[error(
        "antiparticle flag mismatch for {name}: feyngraph is_anti={is_anti}, model pdg_code={pdg}"
    )]
    AntiparticleMismatch {
        name: String,
        is_anti: bool,
        pdg: i64,
    },
    /// The output (result) leg of a vertex resolved to an external leg, which has no
    /// off-shell continuation.
    #[error("an external leg cannot be the result leg of a vertex")]
    ExternalLegAsResult,
}

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

/// Errors while building an [`AmplitudeEvaluator`](super::compile::AmplitudeEvaluator)
/// from a process.
///
/// Holds the model-parameter lookups performed at that layer (particle ids, spins,
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
