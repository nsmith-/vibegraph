//! Resolves the version `vibegraph --version` reports.
//!
//! A release binary is built from a git checkout at a tag, so `git describe`
//! gives the exact tag; a plain `cargo build` off a commit with no tag yet gives
//! the abbreviated hash instead. Neither is available from a source tarball with
//! no `.git` directory (a GitHub "Source code" download, for instance), so that
//! case falls back to the crate's own `Cargo.toml` version rather than leaving
//! the binary unable to build.

use std::path::PathBuf;
use std::process::Command;

fn main() {
    let version = git_describe().unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
    println!("cargo:rustc-env=VIBEGRAPH_VERSION={version}");

    // Best-effort: rebuild when the checked-out ref moves, so a local build
    // picks up a new tag without touching any tracked source file. Silently
    // skipped outside a git checkout, where there is nothing to watch.
    if let Some(git_dir) = git_common_dir() {
        println!("cargo:rerun-if-changed={}", git_dir.join("HEAD").display());
        println!("cargo:rerun-if-changed={}", git_dir.join("refs").display());
    }
}

/// The tag (or, absent one, the abbreviated commit hash) at `HEAD`, with a
/// `-dirty` suffix if the working tree has uncommitted changes. `None` when
/// there is no git checkout to describe (no `git` binary, or the source tree
/// is not a repository at all).
fn git_describe() -> Option<String> {
    let output = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty=-dirty"])
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

/// The directory `git` itself would watch for ref changes (`.git`, or the
/// worktree-specific directory `.git` points at), or `None` outside a checkout.
fn git_common_dir() -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-common-dir"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let dir = String::from_utf8(output.stdout).ok()?;
    Some(PathBuf::from(dir.trim()))
}
