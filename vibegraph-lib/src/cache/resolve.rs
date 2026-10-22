//! Resolution order for a named cache asset, independent of how — or whether
//! — the asset was ever fetched.
//!
//! A UFO model ([`locate`]) is found through an explicit `--flag` path, then
//! its environment variable, then the `~/.vibegraph` cache, then a dev
//! fallback directory. The first two steps are trusted unconditionally — a
//! wrong explicit path surfaces as a normal "asset not found" error one level
//! up, not a silent fallthrough to the next step. The last two steps are
//! existence-probed so the order can actually choose between them.
//!
//! A PDF set ([`locate_pdf_set`]) is found as LHAPDF finds one: an explicit
//! `--flag` path (trusted, as above), then each entry of the LHAPDF data path
//! ([`pdf_data_path`]) in order, then the cache. A data-path entry that lacks
//! the set is passed over, so a list naming several installations resolves
//! the first that holds it. Nothing is relative to the working directory.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::AssetKind;

/// Which resolution step produced a [`Located`]'s directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// An explicit `--flag` path.
    Flag,
    /// The kind's environment variable (a UFO model's one directory).
    Env,
    /// An entry of the LHAPDF data path ([`pdf_data_path`]).
    DataPath,
    /// `<cache_root>/<kind>/<name>/`.
    Cache,
    /// The dev fallback directory.
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
/// shape of `--ufo-dir` and `VIBEGRAPH_UFO_DIR`); `cache_root` is
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
    let cache_dir = kind.entry_dir(cache_root, name);
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

/// The LHAPDF data path, parsed as LHAPDF 6.5.3 parses it (`Paths.cc`,
/// `paths()`): `lhapdf_data_path` (the value of `$LHAPDF_DATA_PATH`) when it is
/// set, even to the empty string, and `lhapath` (`$LHAPATH`) only when it is
/// not; the value is split as a path list (`:` on Unix) and empty entries are
/// skipped.
///
/// LHAPDF also appends its install prefix unless the value ends in `::`. There
/// is no LHAPDF installation here to append, so neither the prefix nor the
/// `::` marker means anything.
pub fn pdf_data_path(lhapdf_data_path: Option<&OsStr>, lhapath: Option<&OsStr>) -> Vec<PathBuf> {
    let Some(value) = lhapdf_data_path.or(lhapath) else {
        return Vec::new();
    };
    std::env::split_paths(value)
        .filter(|entry| !entry.as_os_str().is_empty())
        .collect()
}

/// Resolve PDF set `name`'s directory: `flag/<name>/` when a flag path is
/// given, trusted whether or not it exists; else the first `<entry>/<name>/`
/// that exists, over `data_path` in order; else `<cache_root>/pdf/<name>/`,
/// marked found or not, so a caller that goes on to fetch the set has its
/// write target.
pub fn locate_pdf_set(
    name: &str,
    flag: Option<&Path>,
    data_path: &[PathBuf],
    cache_root: &Path,
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
    if let Some(dir) = data_path
        .iter()
        .map(|entry| entry.join(name))
        .find(|dir| dir.is_dir())
    {
        return Located {
            dir,
            source: Source::DataPath,
            found: true,
        };
    }
    let dir = AssetKind::Pdf.entry_dir(cache_root, name);
    let found = dir.is_dir();
    Located {
        dir,
        source: Source::Cache,
        found,
    }
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
        make_entry(&cache_root.join("ufo"), "Foo");
        make_entry(&dev, "Foo");

        let env_base = scratch("flag-env-base");
        make_entry(&env_base, "Foo");
        let env_os = env_base.as_os_str();

        let got = locate(
            AssetKind::Ufo,
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
        make_entry(&cache_root.join("ufo"), "Foo");
        make_entry(&dev, "Foo");

        let got = locate(
            AssetKind::Ufo,
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
            AssetKind::Ufo,
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
            AssetKind::Ufo,
            "NoSuchSet",
            None,
            None,
            &cache_root,
            Some(&dev),
        );
        assert_eq!(got.source, Source::Cache);
        assert!(!got.found);
        assert_eq!(got.dir, cache_root.join("ufo").join("NoSuchSet"));
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

    fn os(text: &str) -> std::ffi::OsString {
        std::ffi::OsString::from(text)
    }

    /// The variable is a `:` list, read in order, with empty entries skipped.
    #[test]
    fn the_data_path_is_a_colon_list_without_empty_entries() {
        let value = os("/a/one::/b/two:");
        assert_eq!(
            pdf_data_path(Some(&value), None),
            vec![PathBuf::from("/a/one"), PathBuf::from("/b/two")]
        );
    }

    /// `LHAPATH` is read only when `LHAPDF_DATA_PATH` is unset; set to the
    /// empty string, `LHAPDF_DATA_PATH` still shadows it, as in LHAPDF.
    #[test]
    fn lhapath_is_read_only_when_lhapdf_data_path_is_unset() {
        let lhapath = os("/from/lhapath");
        assert_eq!(
            pdf_data_path(None, Some(&lhapath)),
            vec![PathBuf::from("/from/lhapath")]
        );
        let data = os("/from/data");
        assert_eq!(
            pdf_data_path(Some(&data), Some(&lhapath)),
            vec![PathBuf::from("/from/data")]
        );
        let empty = os("");
        assert!(pdf_data_path(Some(&empty), Some(&lhapath)).is_empty());
        assert!(pdf_data_path(None, None).is_empty());
    }

    /// The first data-path entry holding the set wins; an entry without it is
    /// passed over rather than refused, and the flag outranks the whole list.
    #[test]
    fn the_first_data_path_entry_holding_the_set_wins() {
        let empty = scratch("dp-empty");
        let first = scratch("dp-first");
        let second = scratch("dp-second");
        let cache_root = scratch("dp-cache");
        make_entry(&cache_root.join("pdf"), "Set");
        let want = make_entry(&first, "Set");
        make_entry(&second, "Set");
        let path = [empty.clone(), first.clone(), second.clone()];

        let got = locate_pdf_set("Set", None, &path, &cache_root);
        assert_eq!((got.source, got.found), (Source::DataPath, true));
        assert_eq!(got.dir, want);

        let flag = scratch("dp-flag");
        let got = locate_pdf_set("Set", Some(&flag), &path, &cache_root);
        assert_eq!((got.source, got.found), (Source::Flag, false));
        assert_eq!(got.dir, flag.join("Set"));
    }

    /// A data path that lacks the set falls through to the cache, found or not.
    #[test]
    fn a_data_path_without_the_set_falls_through_to_the_cache() {
        let empty = scratch("dp-miss");
        let cache_root = scratch("dp-miss-cache");
        let got = locate_pdf_set("Set", None, std::slice::from_ref(&empty), &cache_root);
        assert_eq!((got.source, got.found), (Source::Cache, false));
        assert_eq!(got.dir, cache_root.join("pdf").join("Set"));

        let cached = make_entry(&cache_root.join("pdf"), "Set");
        let got = locate_pdf_set("Set", None, &[empty], &cache_root);
        assert_eq!((got.source, got.found), (Source::Cache, true));
        assert_eq!(got.dir, cached);
    }
}
