//! `~/.vibegraph` asset cache: resolving named UFO models and PDF sets, and
//! fetching PDF sets, on the machine of a user with no dev checkout.
//!
//! Both asset kinds share one layout, `<cache_root>/{ufo,pdf}/<name>/`
//! ([`AssetKind::entry_dir`]), and one resolution order ([`resolve::locate`]):
//! an explicit path, an environment variable, the cache, then a repo-local dev
//! fallback.
//!
//! Only PDF sets are fetched ([`store`]), each pinned by the SHA-256 of the
//! archive it was fetched as. A UFO model is resolved and never fetched, so one
//! in the cache was placed there by hand. No download URL can be derived from a
//! model name: UFO models are published on the FeynRules wiki as files attached
//! by hand, `/raw-attachment/wiki/<page>/<file>`, where the page is not the
//! model name and the file follows no rule. The 2HDM page alone carries
//! `2HDM.tar.gz`, `2HDM_UFO.tar.gz` and `2HDM_UFO.tar.2.gz`, of which only the
//! middle one is a UFO directory and the first is FeynRules Mathematica source.
//! Fetching UFO models would need a pinned URL per model, and should pin each by
//! its [`model_digest`](crate::ufo::identity::model_digest) over the parsed
//! model rather than by its archive bytes, so that a re-packaged but
//! semantically identical tarball is not taken for a different model.
//!
//! Network I/O is absent from this module: [`store::Fetch`] is the seam a
//! caller supplies bytes through, so the interaction policy around *when* to
//! fetch (prompting, a `--no-network` refusal) lives with the caller, not here.
//!
//! [`pinned`] adds the other half of that seam: which URL a known set name
//! downloads from and what its archive must hash to, compiled in rather than
//! configured.

pub mod pinned;
pub mod resolve;
pub mod store;

use std::path::{Path, PathBuf};

/// The two asset kinds the cache stores, each under its own subdirectory and
/// resolved through its own environment variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    Ufo,
    Pdf,
}

impl AssetKind {
    /// Subdirectory of the cache root holding this kind's entries.
    pub(crate) fn cache_subdir(self) -> &'static str {
        match self {
            AssetKind::Ufo => "ufo",
            AssetKind::Pdf => "pdf",
        }
    }

    /// `<cache_root>/<kind>/<name>/`: where an entry of this kind named `name`
    /// is cached.
    pub(crate) fn entry_dir(self, cache_root: &Path, name: &str) -> PathBuf {
        cache_root.join(self.cache_subdir()).join(name)
    }

    /// Environment variable naming this kind's resolution-order override
    /// directory.
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
