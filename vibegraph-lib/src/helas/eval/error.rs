//! The eval-pass error tree.
//!
//! The two rooting passes ([`RootDiagramError`] from walking the topology,
//! [`RootLorentzError`] from rooting a vertex's Lorentz structure) aggregate into
//! [`CompileError`], and the process-level [`EvalError`] wraps that plus the model
//! lookups done while compiling.

use super::root_lorentz::RootLorentzError;

/// Errors from Pass 1: walking the owned diagram topology into a rooted tree.
///
/// Particle/interaction resolution and the antiparticle-consistency check happen earlier,
/// at the module boundary ([`Diagram::from_view`](crate::diagrams::diagram::Diagram::from_view),
/// reported via [`ConvertError`](crate::diagrams::diagram::ConvertError)), so the only failure left
/// here is a structural one from the walk itself.
#[derive(Clone, Debug, thiserror::Error)]
pub enum RootDiagramError {
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
    /// A line of the process carries a particle whose spin has no wavefunction
    /// (external) or propagator (internal) here: spin 3/2 or spin 2.
    #[error(
        "particle '{particle}' has spin code {spin}, which has no {line} here \
         (spin 3/2 and spin 2 are not supported)"
    )]
    UnsupportedSpin {
        particle: String,
        spin: i32,
        line: SpinLine,
    },
    /// A polarized leg lists no helicity, or one that is not a state of its
    /// particle.
    #[error("external leg {leg} ({particle}) is polarized to {listed:?}, which are not its helicity states")]
    Polarization {
        leg: usize,
        particle: String,
        listed: Vec<i32>,
    },
    /// The process and the compiled AST disagree on the external-leg count.
    #[error("external-leg count mismatch: {0}")]
    TopologyError(String),
    /// Diagram rooting failed.
    #[error(transparent)]
    Compile(#[from] CompileError),
    /// Colorizing the subprocess (color basis + CF matrix) failed.
    #[error(transparent)]
    Color(#[from] crate::helas::color::ColorAlgebraError),
}

/// Which kind of line an [`EvalError::UnsupportedSpin`] particle sits on, named by
/// the object the evaluator would need for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinLine {
    /// An external leg, which needs a wavefunction.
    External,
    /// An internal line, which needs a propagator.
    Internal,
}

impl std::fmt::Display for SpinLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::External => "external wavefunction",
            Self::Internal => "propagator",
        })
    }
}
