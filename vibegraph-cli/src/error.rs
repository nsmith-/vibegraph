//! The failure every command reports: one message, printed to stderr by the
//! binary's top-level handler before it exits non-zero.

/// A command's failure, already phrased for the user: what went wrong and,
/// where there is one, the file or flag it concerns.
#[derive(Debug)]
pub(crate) struct CliError(String);

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CliError {}

/// A [`CliError`] carrying `msg`.
pub(crate) fn err(msg: impl Into<String>) -> CliError {
    CliError(msg.into())
}
