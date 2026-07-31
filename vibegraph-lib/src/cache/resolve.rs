//! Resolution order for a named cache asset, independent of how — or whether
//! — the asset was ever fetched.
//!
//! The order is fixed and the same for both asset kinds: an explicit
//! `--flag` path, then the kind's environment variable, then the
//! `~/.vibegraph` cache, then a repo-local dev fallback (`validation/pdf`,
//! for a checked-out dev tree that already has data fetched). The first two
//! steps are trusted unconditionally — a wrong explicit path surfaces as a
//! normal "asset not found" error one level up, not a silent fallthrough to
//! the next step, matching the pre-cache `--pdf-dir`/`VIBEGRAPH_PDF_DIR`
//! behavior this generalizes. The last two steps are existence-probed so the
//! order can actually choose between them.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::AssetKind;

/// Which resolution step produced a [`Located`]'s directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// An explicit `--flag` path.
    Flag,
    /// The kind's environment variable.
    Env,
    /// `<cache_root>/<kind>/<name>/`.
    Cache,
    /// The repo-local dev fallback directory.
    DevFallback,
}

/// Where a named asset was found — or, if [`found`](Located::found) is
/// `false`, where the cache step would place it, so a caller that goes on to
/// fetch the asset has a write target without recomputing this resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Located {
    pub dir: PathBuf,
    pub source: Source,
    pub found: bool,
}

/// Resolve `name`'s directory under `kind`, walking the precedence order
/// above. `flag` and `env` are base directories containing `<name>/` (the
/// same shape as today's `--pdf-dir`/`VIBEGRAPH_PDF_DIR`); `cache_root` is
/// the `~/.vibegraph` directory itself (its `<kind>/` subdirectory is
/// appended here); `dev_fallback`, when given, is also a base directory.
///
/// Nothing here reads the process environment or the real home directory —
/// every input is a parameter, so resolution order is fully exercisable
/// offline without touching global state.
pub fn locate(
    kind: AssetKind,
    name: &str,
    flag: Option<&Path>,
    env: Option<&OsStr>,
    cache_root: &Path,
    dev_fallback: Option<&Path>,
) -> Located {
    if let Some(flag) = flag {
        let dir = flag.join(name);
        let found = dir.is_dir();
        return Located {
            dir,
            source: Source::Flag,
            found,
        };
    }
    if let Some(env) = env {
        let dir = PathBuf::from(env).join(name);
        let found = dir.is_dir();
        return Located {
            dir,
            source: Source::Env,
            found,
        };
    }
    let cache_dir = cache_root.join(kind.cache_subdir()).join(name);
    if cache_dir.is_dir() {
        return Located {
            dir: cache_dir,
            source: Source::Cache,
            found: true,
        };
    }
    if let Some(dev_fallback) = dev_fallback {
        let dev_dir = dev_fallback.join(name);
        if dev_dir.is_dir() {
            return Located {
                dir: dev_dir,
                source: Source::DevFallback,
                found: true,
            };
        }
    }
    Located {
        dir: cache_dir,
        source: Source::Cache,
        found: false,
    }
}

/// [`locate`] using the process's real `VIBEGRAPH_UFO_DIR`/`VIBEGRAPH_PDF_DIR`
/// environment variable, so the boilerplate of reading it exists once. The
/// cache root still has to be supplied explicitly — `None` falls back to
/// [`super::default_cache_root`], `Some` overrides it (the injection point a
/// caller's own tests need); this function reads the environment regardless,
/// so it is not exercised by this crate's own tests, only [`locate`] is.
pub fn locate_from_env(
    kind: AssetKind,
    name: &str,
    flag: Option<&Path>,
    cache_root_override: Option<&Path>,
    dev_fallback: Option<&Path>,
) -> Option<Located> {
    let cache_root = match cache_root_override {
        Some(p) => p.to_path_buf(),
        None => super::default_cache_root()?,
    };
    let env = std::env::var_os(kind.env_var());
    Some(locate(
        kind,
        name,
        flag,
        env.as_deref(),
        &cache_root,
        dev_fallback,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "vibegraph-cache-resolve-test-{label}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn make_entry(base: &Path, name: &str) -> PathBuf {
        let dir = base.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// An explicit `--flag` wins even when the cache and dev fallback both
    /// have the name — the property the whole precedence order rests on.
    #[test]
    fn flag_wins_over_everything() {
        let flag_base = scratch("flag-base");
        let cache_root = scratch("flag-cache-root");
        let dev = scratch("flag-dev");
        let expected = make_entry(&flag_base, "Foo");
        make_entry(&cache_root.join("pdf"), "Foo");
        make_entry(&dev, "Foo");

        let env_base = scratch("flag-env-base");
        make_entry(&env_base, "Foo");
        let env_os = env_base.as_os_str();

        let got = locate(
            AssetKind::Pdf,
            "Foo",
            Some(&flag_base),
            Some(env_os),
            &cache_root,
            Some(&dev),
        );
        assert_eq!(got.source, Source::Flag);
        assert!(got.found);
        assert_eq!(got.dir, expected);
    }

    /// Env wins over the cache and dev fallback when no flag is given.
    #[test]
    fn env_wins_when_flag_absent() {
        let env_base = scratch("env-base");
        let cache_root = scratch("env-cache-root");
        let dev = scratch("env-dev");
        let expected = make_entry(&env_base, "Foo");
        make_entry(&cache_root.join("pdf"), "Foo");
        make_entry(&dev, "Foo");

        let got = locate(
            AssetKind::Pdf,
            "Foo",
            None,
            Some(env_base.as_os_str()),
            &cache_root,
            Some(&dev),
        );
        assert_eq!(got.source, Source::Env);
        assert!(got.found);
        assert_eq!(got.dir, expected);
    }

    /// The cache wins over the dev fallback when neither flag nor env is set.
    #[test]
    fn cache_wins_over_dev_fallback() {
        let cache_root = scratch("cache-wins-root");
        let dev = scratch("cache-wins-dev");
        let expected = make_entry(&cache_root.join("ufo"), "sm-extra");
        make_entry(&dev, "sm-extra");

        let got = locate(
            AssetKind::Ufo,
            "sm-extra",
            None,
            None,
            &cache_root,
            Some(&dev),
        );
        assert_eq!(got.source, Source::Cache);
        assert!(got.found);
        assert_eq!(got.dir, expected);
    }

    /// The dev fallback is used only once the cache has nothing — the
    /// property that makes it a fallback rather than an equal alternative.
    #[test]
    fn dev_fallback_used_only_when_cache_misses() {
        let cache_root = scratch("dev-fallback-root");
        let dev = scratch("dev-fallback-dir");
        let expected = make_entry(&dev, "NNPDF23_lo_as_0130_qed");

        let got = locate(
            AssetKind::Pdf,
            "NNPDF23_lo_as_0130_qed",
            None,
            None,
            &cache_root,
            Some(&dev),
        );
        assert_eq!(got.source, Source::DevFallback);
        assert!(got.found);
        assert_eq!(got.dir, expected);
    }

    /// Nothing found anywhere: resolution reports the cache write target a
    /// fetch should use, marked not-found rather than silently picking one of
    /// the probed directories.
    #[test]
    fn nothing_found_reports_the_cache_write_target() {
        let cache_root = scratch("nothing-root");
        let dev = scratch("nothing-dev");

        let got = locate(
            AssetKind::Pdf,
            "NoSuchSet",
            None,
            None,
            &cache_root,
            Some(&dev),
        );
        assert_eq!(got.source, Source::Cache);
        assert!(!got.found);
        assert_eq!(got.dir, cache_root.join("pdf").join("NoSuchSet"));
    }

    /// A flag/env path that does not actually exist on disk is still
    /// returned (trusted unconditionally, per the module doc) but reported
    /// as not found — a caller can tell "explicit but wrong" apart from
    /// "nothing configured".
    #[test]
    fn flag_not_on_disk_is_reported_as_not_found() {
        let cache_root = scratch("flag-missing-root");
        let missing_base = scratch("flag-missing-base");
        // Note: no entry created under missing_base/DoesNotExist.

        let got = locate(
            AssetKind::Ufo,
            "DoesNotExist",
            Some(&missing_base),
            None,
            &cache_root,
            None,
        );
        assert_eq!(got.source, Source::Flag);
        assert!(!got.found);
        assert_eq!(got.dir, missing_base.join("DoesNotExist"));
    }

    /// No dev fallback configured, cache misses: falls straight through to
    /// the not-found cache write target, without panicking on the absent
    /// fallback.
    #[test]
    fn missing_dev_fallback_is_skipped_cleanly() {
        let cache_root = scratch("no-dev-root");
        let got = locate(AssetKind::Ufo, "Foo", None, None, &cache_root, None);
        assert_eq!(got.source, Source::Cache);
        assert!(!got.found);
    }
}
