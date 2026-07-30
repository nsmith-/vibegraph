//! `~/.vibegraph` asset cache: resolving and storing named UFO models and PDF
//! sets on the machine of a user with no dev checkout.
//!
//! Two kinds of asset share one layout, `<cache_root>/{ufo,pdf}/<name>/`, and
//! one resolution order ([`resolve::locate`]): an explicit path, an
//! environment variable, the cache, then a repo-local dev fallback. What
//! differs between the two kinds is how an entry is checksum-pinned once
//! fetched — a UFO model by the existing [`crate::ufo::identity::model_digest`]
//! computed over its parsed form, a PDF set by the SHA-256 of the archive it
//! was fetched as ([`store`]) — because a UFO model's digest is already the
//! project's identity for "this is the same model", and pinning archive bytes
//! for it would flag a re-packaged but semantically identical tarball as a
//! different model.
//!
//! Network I/O is deliberately absent from this module: [`store::Fetch`] is
//! the seam a caller supplies bytes through, so the interaction policy around
//! *when* to fetch (prompting, a `--no-network` refusal) lives with the
//! caller, not here.

pub mod resolve;
pub mod store;

use std::path::PathBuf;

/// The two asset kinds the cache stores, each under its own subdirectory and
/// resolved through its own environment variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    Ufo,
    Pdf,
}

impl AssetKind {
    /// Subdirectory of the cache root holding this kind's entries.
    pub fn cache_subdir(self) -> &'static str {
        match self {
            AssetKind::Ufo => "ufo",
            AssetKind::Pdf => "pdf",
        }
    }

    /// Environment variable naming this kind's resolution-order override
    /// directory. `VIBEGRAPH_PDF_DIR` predates this cache; `VIBEGRAPH_UFO_DIR`
    /// is its UFO counterpart, introduced alongside it.
    pub fn env_var(self) -> &'static str {
        match self {
            AssetKind::Ufo => "VIBEGRAPH_UFO_DIR",
            AssetKind::Pdf => "VIBEGRAPH_PDF_DIR",
        }
    }
}

/// The default cache root, `~/.vibegraph`, or `None` if the platform has no
/// resolvable home directory.
///
/// Nothing in this crate's own tests calls this — a test that touched the
/// real home directory would corrupt a developer's machine state or another
/// test's run. Every resolution/store entry point instead takes the cache
/// root as an explicit parameter; this function exists only so a caller
/// outside this crate (the CLI) has one place to compute the real default
/// rather than re-deriving `dirs::home_dir().join(".vibegraph")` itself.
pub fn default_cache_root() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".vibegraph"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_kind_subdirs_and_env_vars_are_distinct() {
        assert_ne!(AssetKind::Ufo.cache_subdir(), AssetKind::Pdf.cache_subdir());
        assert_ne!(AssetKind::Ufo.env_var(), AssetKind::Pdf.env_var());
        assert_eq!(AssetKind::Pdf.env_var(), "VIBEGRAPH_PDF_DIR");
        assert_eq!(AssetKind::Ufo.env_var(), "VIBEGRAPH_UFO_DIR");
    }
}
