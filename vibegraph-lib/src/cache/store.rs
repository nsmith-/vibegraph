//! Fetching and checksum-pinning a named PDF set into `<cache_root>/pdf/<name>/`.
//!
//! Network I/O is not performed here — [`Fetch::fetch`] is the seam a caller
//! implements to actually retrieve archive bytes (HTTP, a staged local copy,
//! or [`FixedFetch`] in a test). Everything downstream of that byte buffer —
//! `.tar.gz` extraction, checksumming, and atomically publishing the result —
//! is this module's job.
//!
//! A set is fetched as a `.tar.gz`, the LHAPDF pattern
//! `validation/pdf/fetch.sh` also fetches. A tarball whose only top-level entry
//! is a directory (the common "wraps everything in `<name>/`" packaging, e.g.
//! LHAPDF's own sets) is unwrapped, so the cached entry is always
//! `<name>/<payload>`, never `<name>/<name>/<payload>`.

use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use tar::Archive;

use crate::ufo::identity::digest_bytes;

use super::AssetKind;

/// A source of an asset's raw archive bytes. The only seam through which
/// this module reaches outside the local filesystem; implementations choose
/// how — or whether — to perform network I/O. The CLI owns *when* this is
/// called: the fetch prompt and `--no-network` refusal both live upstream of
/// this trait, not behind it.
pub trait Fetch {
    /// Fetch `url`'s raw bytes (the `.tar.gz` archive).
    fn fetch(&self, url: &str) -> Result<Vec<u8>, FetchError>;
}

/// An opaque fetch failure. This module does not know or care whether the
/// cause was a network error, an HTTP status, or a stub in a test — the
/// caller's [`Fetch`] impl already turned it into a message.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct FetchError(pub String);

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Fetch(#[from] FetchError),
    #[error("failed to extract archive into {dir}: {source}")]
    Extract { dir: String, source: std::io::Error },
    #[error("cache directory error at {dir}: {source}")]
    Io { dir: String, source: std::io::Error },
}

fn io_err(dir: &Path, source: std::io::Error) -> StoreError {
    StoreError::Io {
        dir: dir.display().to_string(),
        source,
    }
}

/// The LHAPDF data-server URL for a set's `.tar.gz` — exactly the pattern
/// `validation/pdf/fetch.sh` already fetches by hand.
#[cfg(any(test, doc))]
pub(crate) fn lhapdf_download_url(set_name: &str) -> String {
    format!("https://lhapdfsets.web.cern.ch/current/{set_name}.tar.gz")
}

/// A cached entry after a successful fetch: its directory and the checksum
/// pinned alongside it ([`PIN_FILENAME`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Cached {
    pub(crate) dir: PathBuf,
    pub(crate) checksum: String,
}

/// Sidecar file inside a cached entry's directory holding its pinned
/// checksum, hidden so it never collides with a real asset file name.
pub(crate) const PIN_FILENAME: &str = ".vibegraph-checksum";

/// The checksum pinned for an already-cached entry, if it was written by
/// [`cache_pdf_set`] (a `--flag`/env/dev-fallback entry
/// found through [`super::resolve::locate`] was not necessarily fetched by
/// this module and may have no pin at all).
pub(crate) fn read_pin(dir: &Path) -> Option<String> {
    std::fs::read_to_string(dir.join(PIN_FILENAME))
        .ok()
        .map(|s| s.trim().to_string())
}

fn write_pin(dir: &Path, checksum: &str) -> Result<(), StoreError> {
    std::fs::write(dir.join(PIN_FILENAME), checksum).map_err(|e| io_err(dir, e))
}

/// The staging directory an entry of `kind` named `name` is extracted into
/// before it is published: a hidden sibling of the final entry, private to this
/// process.
fn staging_dir(cache_root: &Path, kind: AssetKind, name: &str) -> PathBuf {
    kind.entry_dir(
        cache_root,
        &format!(".staging-{name}-{}", std::process::id()),
    )
}

/// Extract `bytes` (a `.tar.gz`) into a fresh `staging` directory, and return
/// the directory actually holding the payload — `staging` itself, or its sole
/// top-level subdirectory if the archive wrapped everything in one.
fn extract_to_staging(staging: &Path, bytes: &[u8]) -> Result<PathBuf, StoreError> {
    if staging.exists() {
        std::fs::remove_dir_all(staging).map_err(|e| io_err(staging, e))?;
    }
    std::fs::create_dir_all(staging).map_err(|e| io_err(staging, e))?;

    let decoder = GzDecoder::new(bytes);
    let mut archive = Archive::new(decoder);
    archive.unpack(staging).map_err(|e| StoreError::Extract {
        dir: staging.display().to_string(),
        source: e,
    })?;

    let entries: Vec<_> = std::fs::read_dir(staging)
        .map_err(|e| io_err(staging, e))?
        .filter_map(|e| e.ok())
        .collect();
    if let [only] = entries.as_slice() {
        if only.path().is_dir() {
            return Ok(only.path());
        }
    }
    Ok(staging.to_path_buf())
}

/// Publish `payload` (produced by [`extract_to_staging`]) as `<final_dir>`,
/// replacing any prior contents, then remove the now-empty staging wrapper if
/// the payload lived nested inside one.
fn publish(staging: PathBuf, payload: PathBuf, final_dir: &Path) -> Result<(), StoreError> {
    if final_dir.exists() {
        std::fs::remove_dir_all(final_dir).map_err(|e| io_err(final_dir, e))?;
    }
    if let Some(parent) = final_dir.parent() {
        std::fs::create_dir_all(parent).map_err(|e| io_err(parent, e))?;
    }
    std::fs::rename(&payload, final_dir).map_err(|e| io_err(final_dir, e))?;
    if staging != payload && staging.exists() {
        std::fs::remove_dir_all(&staging).map_err(|e| io_err(&staging, e))?;
    }
    Ok(())
}

/// Fetch and cache a PDF set. The pinned checksum is SHA-256 of the fetched
/// `.tar.gz` bytes, computed before extraction — the archive as retrieved,
/// not a hash of whatever files happened to land on disk.
pub(crate) fn cache_pdf_set(
    cache_root: &Path,
    name: &str,
    url: &str,
    fetch: &dyn Fetch,
) -> Result<Cached, StoreError> {
    let bytes = fetch.fetch(url)?;
    let checksum = digest_bytes(&bytes);
    let staging = staging_dir(cache_root, AssetKind::Pdf, name);
    let payload = extract_to_staging(&staging, &bytes)?;
    write_pin(&payload, &checksum)?;
    let dir = AssetKind::Pdf.entry_dir(cache_root, name);
    publish(staging, payload, &dir)?;
    Ok(Cached { dir, checksum })
}

/// A [`Fetch`] that ignores the URL and always returns the same bytes.
/// Mainly useful for tests exercising the storage layer without real network
/// access — a stub for the `Fetch` seam, the same role
/// [`crate::pdf::PdfMember::from_subgrids`] plays for in-memory PDF grids.
pub struct FixedFetch(pub Vec<u8>);

impl Fetch for FixedFetch {
    fn fetch(&self, _url: &str) -> Result<Vec<u8>, FetchError> {
        Ok(self.0.clone())
    }
}

/// A [`Fetch`] that always fails, for exercising `--no-network`-style
/// refusal paths without a stray real request.
pub struct RefusingFetch;

impl Fetch for RefusingFetch {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        Err(FetchError(format!(
            "network fetch disabled (would fetch {url})"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn scratch(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "vibegraph-cache-store-test-{label}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Build a `.tar.gz` in memory with one top-level file per `(path,
    /// content)` pair, no wrapping directory.
    fn build_tar_gz_flat(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        for (path, content) in files {
            let mut header = tar::Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append_data(&mut header, path, *content).unwrap();
        }
        let tar_bytes = builder.into_inner().unwrap();
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&tar_bytes).unwrap();
        encoder.finish().unwrap()
    }

    /// Like [`build_tar_gz_flat`], but every path is prefixed with
    /// `wrapper/`, the "everything under one top-level directory" packaging
    /// LHAPDF's own tarballs use.
    fn build_tar_gz_wrapped(wrapper: &str, files: &[(&str, &[u8])]) -> Vec<u8> {
        let prefixed: Vec<(String, &[u8])> = files
            .iter()
            .map(|(path, content)| (format!("{wrapper}/{path}"), *content))
            .collect();
        let refs: Vec<(&str, &[u8])> = prefixed.iter().map(|(p, c)| (p.as_str(), *c)).collect();
        build_tar_gz_flat(&refs)
    }

    #[test]
    fn pdf_checksum_is_sha256_of_the_archive_bytes() {
        let cache_root = scratch("pdf-checksum-root");
        let archive = build_tar_gz_wrapped(
            "TestSet",
            &[("TestSet.info", b"SetDesc: \"test\"\n" as &[u8])],
        );
        let expected = digest_bytes(&archive);

        let cached = cache_pdf_set(
            &cache_root,
            "TestSet",
            "https://example.invalid/TestSet.tar.gz",
            &FixedFetch(archive),
        )
        .expect("cache_pdf_set");

        assert_eq!(cached.checksum, expected);
        assert_eq!(cached.dir, cache_root.join("pdf").join("TestSet"));
        assert!(cached.dir.join("TestSet.info").is_file());
        assert_eq!(read_pin(&cached.dir).as_deref(), Some(expected.as_str()));
    }

    /// A tarball with no single wrapping directory (multiple top-level
    /// entries) is stored as-is, not misidentified as wrapped.
    #[test]
    fn unwrapped_archive_with_multiple_top_level_entries_is_stored_flat() {
        let cache_root = scratch("pdf-flat-root");
        let archive = build_tar_gz_flat(&[
            ("TestSet.info", b"a" as &[u8]),
            ("TestSet_0000.dat", b"b" as &[u8]),
        ]);

        let cached = cache_pdf_set(
            &cache_root,
            "TestSet",
            "https://example.invalid/TestSet.tar.gz",
            &FixedFetch(archive),
        )
        .expect("cache_pdf_set");

        assert!(cached.dir.join("TestSet.info").is_file());
        assert!(cached.dir.join("TestSet_0000.dat").is_file());
    }

    /// Fetch failure propagates as [`StoreError::Fetch`] without touching the
    /// filesystem — what the CLI's `--no-network` refusal path rests on.
    #[test]
    fn fetch_failure_propagates_without_writing_anything() {
        let cache_root = scratch("pdf-refuse-root");
        let err = cache_pdf_set(
            &cache_root,
            "TestSet",
            "https://example.invalid/TestSet.tar.gz",
            &RefusingFetch,
        )
        .unwrap_err();
        assert!(matches!(err, StoreError::Fetch(_)), "{err:?}");
        assert!(!cache_root.join("pdf").join("TestSet").exists());
    }

    /// A second fetch of the same name overwrites the first entry cleanly
    /// (no leftover staging directories, no stale files from the old
    /// content) rather than merging or refusing.
    #[test]
    fn recaching_an_existing_name_replaces_it() {
        let cache_root = scratch("pdf-recache-root");
        let first = build_tar_gz_wrapped("TestSet", &[("only_in_v1.dat", b"v1" as &[u8])]);
        cache_pdf_set(&cache_root, "TestSet", "u", &FixedFetch(first)).unwrap();

        let second = build_tar_gz_wrapped("TestSet", &[("only_in_v2.dat", b"v2" as &[u8])]);
        let cached = cache_pdf_set(&cache_root, "TestSet", "u", &FixedFetch(second)).unwrap();

        assert!(!cached.dir.join("only_in_v1.dat").exists());
        assert!(cached.dir.join("only_in_v2.dat").is_file());
    }
}
