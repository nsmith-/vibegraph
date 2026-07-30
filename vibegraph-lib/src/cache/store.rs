//! Fetching and checksum-pinning a named asset into `<cache_root>/<kind>/<name>/`.
//!
//! Network I/O is not performed here — [`Fetch::fetch`] is the seam a caller
//! implements to actually retrieve archive bytes (HTTP, a staged local copy,
//! or [`FixedFetch`] in a test). Everything downstream of that byte buffer —
//! `.tar.gz` extraction, checksumming, and atomically publishing the result —
//! is this module's job.
//!
//! Both asset kinds are fetched as a `.tar.gz`, matching
//! `validation/pdf/fetch.sh`'s existing LHAPDF pattern and the shape UFO
//! models are normally distributed in. A tarball whose only top-level entry
//! is a directory (the common "wraps everything in `<name>/`" packaging, e.g.
//! LHAPDF's own sets) is unwrapped, so the cached entry is always
//! `<name>/<payload>`, never `<name>/<name>/<payload>`.

use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use tar::Archive;

use crate::ufo::identity::digest_bytes;
use crate::ufo::{UFOModel, UfoError};

use super::AssetKind;

/// A source of an asset's raw archive bytes. The only seam through which
/// this module reaches outside the local filesystem; implementations choose
/// how — or whether — to perform network I/O. `U4` (the CLI's first-run UX)
/// owns *when* this is called: the fetch prompt and `--no-network` refusal
/// both live upstream of this trait, not behind it.
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
    #[error("fetched UFO archive did not parse as a model: {0}")]
    UfoParse(#[from] UfoError),
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
pub fn lhapdf_download_url(set_name: &str) -> String {
    format!("https://lhapdfsets.web.cern.ch/current/{set_name}.tar.gz")
}

/// The LHAPDF `pdfsets.index` listing all set names the data server carries —
/// what a caller would fetch (through [`Fetch`]) to validate a set name or
/// offer suggestions before attempting the set's own download.
pub const LHAPDF_INDEX_URL: &str = "https://lhapdfsets.web.cern.ch/current/pdfsets.index";

// There is no UFO counterpart to `lhapdf_download_url`, and it is not an
// omission. UFO models are published on the FeynRules wiki, one page per model,
// with hand-attached files: `/raw-attachment/wiki/<page>/<file>`, where the page
// is not the model name and the file follows no rule — the 2HDM page alone
// carries `2HDM.tar.gz`, `2HDM_UFO.tar.gz` and `2HDM_UFO.tar.2.gz`, of which
// only the middle one is a UFO directory and the first is FeynRules Mathematica
// source. A URL derived from a model name is therefore not merely unverified: it
// 404s for most names, and for some it succeeds and returns the wrong kind of
// archive. [`cache_ufo_model`] below takes a URL from its caller and is ready
// for a real one; deriving that URL from a name is what nothing can do today.

/// A cached entry after a successful fetch: its directory and the checksum
/// pinned alongside it ([`PIN_FILENAME`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cached {
    pub dir: PathBuf,
    pub checksum: String,
}

/// Sidecar file inside a cached entry's directory holding its pinned
/// checksum, hidden so it never collides with a real asset file name.
pub const PIN_FILENAME: &str = ".vibegraph-checksum";

/// The checksum pinned for an already-cached entry, if it was written by
/// [`cache_pdf_set`]/[`cache_ufo_model`] (a `--flag`/env/dev-fallback entry
/// found through [`super::resolve::locate`] was not necessarily fetched by
/// this module and may have no pin at all).
pub fn read_pin(dir: &Path) -> Option<String> {
    std::fs::read_to_string(dir.join(PIN_FILENAME))
        .ok()
        .map(|s| s.trim().to_string())
}

fn write_pin(dir: &Path, checksum: &str) -> Result<(), StoreError> {
    std::fs::write(dir.join(PIN_FILENAME), checksum).map_err(|e| io_err(dir, e))
}

/// Extract `bytes` (a `.tar.gz`) into a fresh staging directory under
/// `<cache_root>/<kind>/`, and return the directory actually holding the
/// payload — the staging directory itself, or its sole top-level
/// subdirectory if the archive wrapped everything in one.
fn extract_to_staging(
    cache_root: &Path,
    kind: AssetKind,
    name: &str,
    bytes: &[u8],
) -> Result<PathBuf, StoreError> {
    let kind_dir = cache_root.join(kind.cache_subdir());
    std::fs::create_dir_all(&kind_dir).map_err(|e| io_err(&kind_dir, e))?;

    let staging = kind_dir.join(format!(".staging-{name}-{}", std::process::id()));
    if staging.exists() {
        std::fs::remove_dir_all(&staging).map_err(|e| io_err(&staging, e))?;
    }
    std::fs::create_dir_all(&staging).map_err(|e| io_err(&staging, e))?;

    let decoder = GzDecoder::new(bytes);
    let mut archive = Archive::new(decoder);
    archive.unpack(&staging).map_err(|e| StoreError::Extract {
        dir: staging.display().to_string(),
        source: e,
    })?;

    let entries: Vec<_> = std::fs::read_dir(&staging)
        .map_err(|e| io_err(&staging, e))?
        .filter_map(|e| e.ok())
        .collect();
    if let [only] = entries.as_slice() {
        if only.path().is_dir() {
            return Ok(only.path());
        }
    }
    Ok(staging)
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
pub fn cache_pdf_set(
    cache_root: &Path,
    name: &str,
    url: &str,
    fetch: &dyn Fetch,
) -> Result<Cached, StoreError> {
    let bytes = fetch.fetch(url)?;
    let checksum = digest_bytes(&bytes);
    let staging = cache_root
        .join(AssetKind::Pdf.cache_subdir())
        .join(format!(".staging-{name}-{}", std::process::id()));
    let payload = extract_to_staging(cache_root, AssetKind::Pdf, name, &bytes)?;
    write_pin(&payload, &checksum)?;
    let dir = cache_root.join(AssetKind::Pdf.cache_subdir()).join(name);
    publish(staging, payload, &dir)?;
    Ok(Cached { dir, checksum })
}

/// Fetch and cache a UFO model. The pinned checksum is the existing
/// [`model_digest`](crate::ufo::identity::model_digest) of the model the
/// archive parses to under its default restriction (the one a bare `import
/// model <name>` resolves to) — not a hash of the archive bytes. Two UFO
/// tarballs differing only in comments, file order, or packaging pin
/// identically; that is the reason to reuse the model digest here rather
/// than hash bytes as [`cache_pdf_set`] does.
pub fn cache_ufo_model(
    cache_root: &Path,
    name: &str,
    url: &str,
    fetch: &dyn Fetch,
) -> Result<Cached, StoreError> {
    let bytes = fetch.fetch(url)?;
    let staging = cache_root
        .join(AssetKind::Ufo.cache_subdir())
        .join(format!(".staging-{name}-{}", std::process::id()));
    let payload = extract_to_staging(cache_root, AssetKind::Ufo, name, &bytes)?;
    let (_, checksum) = UFOModel::load_with_digest(&payload, None)?;
    write_pin(&payload, &checksum)?;
    let dir = cache_root.join(AssetKind::Ufo.cache_subdir()).join(name);
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
    /// filesystem — the `--no-network` refusal path U4 will build on.
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

    /// The UFO path pins the existing model digest, not a hash of the
    /// archive bytes — two archives that parse to the same restricted model
    /// pin identically even though their bytes (and even file layout) differ.
    /// Uses a minimal but syntactically valid empty UFO model (no particles,
    /// no vertices) purely to exercise the parse → digest → pin pipeline;
    /// nothing about a real physics model is asserted here (that is
    /// `vibegraph-lib/src/ufo`'s own test surface).
    #[test]
    fn ufo_checksum_is_the_model_digest_and_survives_repackaging() {
        let empty_ufo_files: &[(&str, &[u8])] = &[
            ("particles.py", b"# no particles\n" as &[u8]),
            ("lorentz.py", b"# no lorentz structures\n" as &[u8]),
            ("couplings.py", b"# no couplings\n" as &[u8]),
            ("parameters.py", b"# no parameters\n" as &[u8]),
            ("vertices.py", b"# no vertices\n" as &[u8]),
        ];

        let cache_root_a = scratch("ufo-digest-root-a");
        let wrapped = build_tar_gz_wrapped("toy_model", empty_ufo_files);
        let cached_a = cache_ufo_model(&cache_root_a, "toy_model", "u", &FixedFetch(wrapped))
            .expect("cache_ufo_model (wrapped archive)");

        let cache_root_b = scratch("ufo-digest-root-b");
        let flat = build_tar_gz_flat(empty_ufo_files);
        let cached_b = cache_ufo_model(&cache_root_b, "toy_model", "u", &FixedFetch(flat))
            .expect("cache_ufo_model (flat archive)");

        assert_eq!(
            cached_a.checksum, cached_b.checksum,
            "same model content through different packaging must pin identically"
        );

        let (_, recomputed) = UFOModel::load_with_digest(&cached_a.dir, None)
            .expect("cached directory reloads as a valid UFO model");
        assert_eq!(
            recomputed, cached_a.checksum,
            "the pinned checksum must be exactly what GlobalConfig would recompute on load"
        );
        assert_eq!(
            read_pin(&cached_a.dir).as_deref(),
            Some(cached_a.checksum.as_str())
        );
    }

    /// A UFO archive that fails to parse is never published under its final
    /// name — the staging directory absorbs the failure.
    #[test]
    fn unparseable_ufo_archive_is_not_published() {
        let cache_root = scratch("ufo-bad-root");
        let archive = build_tar_gz_wrapped(
            "broken_model",
            &[("particles.py", b"not even close to python(((" as &[u8])],
        );
        let err = cache_ufo_model(&cache_root, "broken_model", "u", &FixedFetch(archive));
        // Missing required source files (lorentz.py etc.) is itself an error
        // even before the malformed particles.py would be — either way this
        // must not produce a cached entry.
        assert!(err.is_err(), "{err:?}");
        assert!(!cache_root.join("ufo").join("broken_model").exists());
    }
}
