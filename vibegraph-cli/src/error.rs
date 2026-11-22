//! The failure every command reports: one message, and the kind of thing that
//! failed. The text modes print the message; `--output-ldjson` carries both in
//! its `refusal` object.

use serde::Serialize;
use vibegraph::artifact::ArtifactError;
use vibegraph::cache::lhapdf::LookupError;
use vibegraph::cache::pinned::EnsureError;
use vibegraph::cache::store::StoreError;
use vibegraph::diagrams::artifact::DiagramArtifactError;
use vibegraph::generate::GenerateError;
use vibegraph::pdf::select::{PdfSelectError, SelectFailure};
use vibegraph::systematics::plan::ResolveError;

/// What a refusal is about, as a short name a tool can branch on without
/// reading the message.
///
/// The names are part of the `--output-ldjson` schema
/// ([`crate::output::SCHEMA_VERSION`]): a kind is never renamed or given another
/// meaning, and adding or removing one bumps the schema version. The message is
/// for people and carries no such promise.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ErrorKind {
    /// The flags, or a combination of them, cannot be honoured.
    Usage,
    /// A proc, run or param card cannot be read or parsed, or asks for
    /// something this build refuses.
    Card,
    /// The model cannot be located, loaded or bound.
    Model,
    /// The process cannot be enumerated or compiled, or has a shape the
    /// integrator refuses.
    Process,
    /// The parton densities cannot be located, read or matched to the card.
    Pdf,
    /// An asset was not downloaded: the network policy refused it, or the
    /// download failed.
    Network,
    /// The integration finished without a result its points support: no point
    /// carried weight, too few did, or no iteration was kept.
    Support,
    /// A saved artifact cannot be read, is of another format version, or
    /// disagrees with the build or the inputs reading it.
    Artifact,
    /// A file or directory the command reads or writes, other than a card or
    /// an artifact, cannot be used.
    Io,
    /// The run failed after its inputs were accepted, for a reason none of the
    /// other kinds names.
    Run,
}

impl ErrorKind {
    /// Every kind, in the order the schema documents them.
    #[cfg(test)]
    pub(crate) const ALL: [ErrorKind; 10] = [
        ErrorKind::Usage,
        ErrorKind::Card,
        ErrorKind::Model,
        ErrorKind::Process,
        ErrorKind::Pdf,
        ErrorKind::Network,
        ErrorKind::Support,
        ErrorKind::Artifact,
        ErrorKind::Io,
        ErrorKind::Run,
    ];
}

/// Why an `artifact`-kind refusal happened, as a stable name a tool can branch
/// on, with the numbers or digests that name it.
///
/// Carried in the refusal's `reason` object, tagged by `kind`. A reason is never
/// renamed or given another meaning; a new one is an added value of an existing
/// field, so it keeps the schema version.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum ArtifactReason {
    /// The diagrams were enumerated by another enumeration version than this
    /// build's: integrate again from the cards.
    EnumerationVersion { found: u32, expected: u32 },
    /// The file's format version is not one this build reads.
    FormatVersion {
        found: u32,
        oldest: u32,
        expected: u32,
    },
    /// The embedded model does not digest to the recorded one: the file is
    /// corrupt or was edited.
    ModelDigest { recorded: String, embedded: String },
    /// The diagram sets do not digest to the recorded digest: the file is
    /// corrupt or was edited.
    DiagramSetDigest { recorded: String, computed: String },
    /// The collider energy the cards now give is not the one the artifact was
    /// integrated at.
    ColliderEnergy {
        recorded: f64,
        rebuilt: f64,
        beams: [f64; 2],
    },
    /// The recorded proc card parses to another process in this build.
    ProcessReparsed { recorded: String, reparsed: String },
    /// The file could not be read or decoded.
    Unreadable,
    /// Any other disagreement; the message says which.
    Other,
}

impl From<&ArtifactError> for ArtifactReason {
    fn from(e: &ArtifactError) -> Self {
        match e {
            ArtifactError::UnsupportedVersion {
                found,
                oldest,
                expected,
            } => ArtifactReason::FormatVersion {
                found: *found,
                oldest: *oldest,
                expected: *expected,
            },
            ArtifactError::EnumerationVersion { found, expected } => {
                ArtifactReason::EnumerationVersion {
                    found: *found,
                    expected: *expected,
                }
            }
            ArtifactError::ModelDigestMismatch {
                recorded, embedded, ..
            } => ArtifactReason::ModelDigest {
                recorded: recorded.clone(),
                embedded: embedded.clone(),
            },
            ArtifactError::DiagramDigestMismatch { recorded, computed } => {
                ArtifactReason::DiagramSetDigest {
                    recorded: recorded.clone(),
                    computed: computed.clone(),
                }
            }
            ArtifactError::Io { .. } | ArtifactError::Decode(_) | ArtifactError::Zstd(_) => {
                ArtifactReason::Unreadable
            }
            _ => ArtifactReason::Other,
        }
    }
}

impl From<&DiagramArtifactError> for ArtifactReason {
    fn from(e: &DiagramArtifactError) -> Self {
        match e {
            DiagramArtifactError::UnsupportedVersion {
                found,
                oldest,
                expected,
            } => ArtifactReason::FormatVersion {
                found: *found,
                oldest: *oldest,
                expected: *expected,
            },
            DiagramArtifactError::EnumerationVersion { found, expected } => {
                ArtifactReason::EnumerationVersion {
                    found: *found,
                    expected: *expected,
                }
            }
            DiagramArtifactError::ModelDigestMismatch {
                recorded, embedded, ..
            } => ArtifactReason::ModelDigest {
                recorded: recorded.clone(),
                embedded: embedded.clone(),
            },
            DiagramArtifactError::DiagramDigestMismatch { recorded, computed } => {
                ArtifactReason::DiagramSetDigest {
                    recorded: recorded.clone(),
                    computed: computed.clone(),
                }
            }
            DiagramArtifactError::Io { .. }
            | DiagramArtifactError::Decode(_)
            | DiagramArtifactError::Zstd(_) => ArtifactReason::Unreadable,
            _ => ArtifactReason::Other,
        }
    }
}

impl From<&GenerateError> for ArtifactReason {
    fn from(e: &GenerateError) -> Self {
        match e {
            GenerateError::ColliderEnergyChanged {
                recorded,
                rebuilt,
                beams,
            } => ArtifactReason::ColliderEnergy {
                recorded: *recorded,
                rebuilt: *rebuilt,
                beams: *beams,
            },
            GenerateError::ProcessReparsed { recorded, reparsed } => {
                ArtifactReason::ProcessReparsed {
                    recorded: recorded.clone(),
                    reparsed: reparsed.clone(),
                }
            }
            _ => ArtifactReason::Other,
        }
    }
}

/// A command's failure, already phrased for the user: what went wrong and,
/// where there is one, the file or flag it concerns.
#[derive(Debug)]
pub(crate) struct CliError {
    kind: ErrorKind,
    message: String,
    /// Set on every [`ErrorKind::Artifact`] failure, and on no other.
    reason: Option<ArtifactReason>,
}

impl CliError {
    pub(crate) fn reason(&self) -> Option<&ArtifactReason> {
        self.reason.as_ref()
    }

    /// This failure with `reason` as the cause of its artifact refusal.
    pub(crate) fn with_reason(mut self, reason: impl Into<ArtifactReason>) -> Self {
        debug_assert_eq!(self.kind, ErrorKind::Artifact);
        self.reason = Some(reason.into());
        self
    }

    pub(crate) fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CliError {}

/// A [`CliError`] of `kind` carrying `msg`.
pub(crate) fn err(kind: ErrorKind, msg: impl Into<String>) -> CliError {
    CliError {
        kind,
        message: msg.into(),
        reason: (kind == ErrorKind::Artifact).then_some(ArtifactReason::Other),
    }
}

/// The kind of a failure to find or read the parton densities: a download the
/// network policy refused or that failed is [`ErrorKind::Network`], a label the
/// run card cannot have is [`ErrorKind::Card`], and the rest are the set's own.
pub(crate) fn pdf_error_kind(e: &PdfSelectError) -> ErrorKind {
    fn lookup(e: &LookupError) -> ErrorKind {
        match e {
            LookupError::Refused(_) | LookupError::IndexFetch { .. } => ErrorKind::Network,
            LookupError::Obtain {
                source: EnsureError::ChecksumMismatch(_) | EnsureError::Store(StoreError::Fetch(_)),
                ..
            } => ErrorKind::Network,
            _ => ErrorKind::Pdf,
        }
    }
    match e.kind() {
        SelectFailure::Label(_) => ErrorKind::Card,
        SelectFailure::Lookup(source) | SelectFailure::IndexUnavailable { source, .. } => {
            lookup(source)
        }
        _ => ErrorKind::Pdf,
    }
}

/// The kind of a failure to resolve a systematics request's PDF sets: downloads
/// the network policy did not allow are [`ErrorKind::Network`], and the rest
/// are the sets' own.
pub(crate) fn systematics_error_kind(e: &ResolveError) -> ErrorKind {
    match e {
        ResolveError::Declined { .. } => ErrorKind::Network,
        _ => ErrorKind::Pdf,
    }
}

#[cfg(test)]
mod tests {
    use super::{systematics_error_kind, ErrorKind};
    use vibegraph::systematics::plan::ResolveError;

    /// A systematics set that would need a download nobody allowed is a network
    /// refusal, like the run's own set; one that cannot be read is the set's.
    #[test]
    fn a_declined_systematics_download_is_a_network_refusal() {
        let declined = ResolveError::Declined {
            count: 1,
            what: "NNPDF31_nnlo_as_0118".to_string(),
            why: "the network is off".to_string(),
        };
        assert_eq!(systematics_error_kind(&declined), ErrorKind::Network);
        let member = ResolveError::Member {
            set: "NNPDF31_nnlo_as_0118".to_string(),
            member: 3,
            why: "unreadable".to_string(),
        };
        assert_eq!(systematics_error_kind(&member), ErrorKind::Pdf);
    }

    /// The kinds' wire names are the schema's: pinned name for name, so a
    /// renamed variant fails here rather than in a consumer.
    #[test]
    fn every_kind_serializes_to_its_documented_name() {
        let names: Vec<String> = ErrorKind::ALL
            .iter()
            .map(|k| {
                serde_json::to_value(k)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string()
            })
            .collect();
        assert_eq!(
            names,
            [
                "usage", "card", "model", "process", "pdf", "network", "support", "artifact", "io",
                "run"
            ]
        );
    }
}
