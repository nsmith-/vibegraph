//! `git describe` for the version string, shared by the build script and the
//! test that pins its arguments.

use std::path::Path;
use std::process::Command;

/// Only `v*` tags name a release. Other tags in the repository (the
/// reference-data bundles, `refdata-*`) are newer than the release tags and
/// would otherwise win `git describe --tags`.
pub const DESCRIBE_ARGS: [&str; 5] = ["describe", "--tags", "--match", "v*", "--always"];

/// The newest `v*` tag reachable from `HEAD` with its distance and abbreviated
/// hash (or just the hash when no such tag is reachable), with a `-dirty`
/// suffix if the working tree has uncommitted changes. `None` when there is no
/// git checkout to describe. `dir` is the directory to describe in; `None`
/// uses the current directory.
pub fn git_describe(dir: Option<&Path>) -> Option<String> {
    let mut command = Command::new("git");
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    let output = command
        .args(DESCRIBE_ARGS)
        .arg("--dirty=-dirty")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let described = String::from_utf8(output.stdout).ok()?;
    let described = described.trim();
    if described.is_empty() {
        None
    } else {
        Some(described.to_string())
    }
}
