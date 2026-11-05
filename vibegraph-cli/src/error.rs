//! The failure every command reports: one message, and the kind of thing that
//! failed. The text modes print the message; `--output-ldjson` carries both in
//! its `refusal` object.

use serde::Serialize;
use vibegraph::cache::lhapdf::LookupError;
use vibegraph::cache::pinned::EnsureError;
use vibegraph::cache::store::StoreError;
use vibegraph::pdf::select::{PdfSelectError, SelectFailure};

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

/// A command's failure, already phrased for the user: what went wrong and,
/// where there is one, the file or flag it concerns.
#[derive(Debug)]
pub(crate) struct CliError {
    kind: ErrorKind,
    message: String,
}

impl CliError {
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

#[cfg(test)]
mod tests {
    use super::ErrorKind;

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
