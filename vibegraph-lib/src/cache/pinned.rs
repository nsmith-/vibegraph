//! Compiled-in download pins for the PDF sets this build knows how to fetch.
//!
//! A pin is a set name, the archive URL it is fetched from, and the SHA-256
//! that archive must hash to. It lives in this table rather than in a config
//! file so that a user cannot end up silently running against different data
//! than the build was validated against: changing the data a set name resolves
//! to requires changing this source and rebuilding.
//!
//! PDF sets are fetched on first use rather than shipped inside the binary.
//! Size is not the reason — only member 0 is ever consumed, which packs to a
//! few hundred kilobytes. The reason is that no redistribution grant is
//! published for these sets: the LHAPDF archives carry no license file, the
//! `.info` metadata format has no license field, and LHAPDF's own GPLv3 covers
//! its source code rather than the third-party grids it serves. Fetching from
//! the upstream data server leaves distribution where its authors put it.
//! Shipping only member 0 would be a partial redistribution, not an exemption
//! from that.
//!
//! [`ensure_pdf_set`] layers verification on top of [`store::cache_pdf_set`] by
//! wrapping the caller's [`Fetch`] in a [`VerifiedFetch`], so an archive whose
//! bytes do not match the pin is rejected *before* the storage layer extracts
//! or publishes anything.

use std::cell::RefCell;
use std::path::{Path, PathBuf};

use crate::ufo::identity::digest_bytes;

use super::store::{self, Fetch, FetchError, StoreError};
use super::AssetKind;

/// The PDF set used when a run does not name one: MG5's LO default `nn23lo1`,
/// LHAPDF ID 247000.
pub const DEFAULT_PDF_SET: &str = "NNPDF23_lo_as_0130_qed";

/// One set's download pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinnedPdfSet {
    pub name: &'static str,
    /// Archive URL. Stored rather than derived so a set served from somewhere
    /// other than the LHAPDF data server can be pinned without special-casing;
    /// a test asserts the entries that *are* LHAPDF-served agree with
    /// [`store::lhapdf_download_url`].
    pub url: &'static str,
    /// Lowercase hex SHA-256 of the archive at `url`, in the same form
    /// [`digest_bytes`] produces.
    pub sha256: &'static str,
    /// Archive size, so a caller can tell a user what a fetch will cost before
    /// starting it.
    pub archive_bytes: u64,
}

/// Every set this build can fetch by name.
pub const PINNED_PDF_SETS: &[PinnedPdfSet] = &[PinnedPdfSet {
    name: DEFAULT_PDF_SET,
    url: "https://lhapdfsets.web.cern.ch/current/NNPDF23_lo_as_0130_qed.tar.gz",
    sha256: "60d3c1df1c31e5840f91f4217163ae30a256b9291a5adc894882e86607ef5d63",
    archive_bytes: 27_625_668,
}];

/// The pin for `name`, or `None` if this build has none.
pub fn pinned_pdf_set(name: &str) -> Option<&'static PinnedPdfSet> {
    PINNED_PDF_SETS.iter().find(|set| set.name == name)
}

/// Comma-separated pinned set names, for an error listing what *is* available.
fn pinned_names() -> String {
    PINNED_PDF_SETS
        .iter()
        .map(|s| s.name)
        .collect::<Vec<_>>()
        .join(", ")
}

/// A fetched archive that did not hash to its pin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChecksumMismatch {
    pub url: String,
    pub expected: String,
    pub actual: String,
}

#[derive(Debug, thiserror::Error)]
pub enum EnsureError {
    #[error("PDF set {name} has no download pin in this build (pinned sets: {known})")]
    Unpinned { name: String, known: String },
    #[error(
        "the archive fetched from {} does not match this build's pinned checksum \
         (expected sha256 {}, got {}); nothing was cached",
        .0.url, .0.expected, .0.actual
    )]
    ChecksumMismatch(ChecksumMismatch),
    #[error(transparent)]
    Store(#[from] StoreError),
}

/// A [`Fetch`] that checks what an inner `Fetch` returned against an expected
/// SHA-256, turning a mismatch into a fetch failure.
///
/// Interposing at this seam rather than checking after the fact is what makes
/// a mismatched archive un-publishable: [`store::cache_pdf_set`] never sees the
/// bytes, so its "a failed fetch writes nothing" guarantee covers a corrupted
/// or substituted download as well as an unreachable server.
pub struct VerifiedFetch<'a> {
    inner: &'a dyn Fetch,
    expected_sha256: &'a str,
    mismatch: RefCell<Option<ChecksumMismatch>>,
}

impl<'a> VerifiedFetch<'a> {
    pub fn new(inner: &'a dyn Fetch, expected_sha256: &'a str) -> Self {
        Self {
            inner,
            expected_sha256,
            mismatch: RefCell::new(None),
        }
    }

    /// The mismatch this wrapper rejected, if it rejected one. Lets a caller
    /// distinguish "the download was wrong" from "the download failed" after
    /// the error has been flattened into [`StoreError::Fetch`].
    pub fn mismatch(&self) -> Option<ChecksumMismatch> {
        self.mismatch.borrow().clone()
    }
}

impl Fetch for VerifiedFetch<'_> {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let bytes = self.inner.fetch(url)?;
        let actual = digest_bytes(&bytes);
        if actual != self.expected_sha256 {
            let mismatch = ChecksumMismatch {
                url: url.to_string(),
                expected: self.expected_sha256.to_string(),
                actual,
            };
            let message = format!(
                "checksum mismatch for {url}: expected sha256 {}, got {}",
                mismatch.expected, mismatch.actual
            );
            *self.mismatch.borrow_mut() = Some(mismatch);
            return Err(FetchError(message));
        }
        Ok(bytes)
    }
}

/// Where a PDF set lives once cached.
pub fn pdf_cache_dir(cache_root: &Path, name: &str) -> PathBuf {
    cache_root.join(AssetKind::Pdf.cache_subdir()).join(name)
}

/// A PDF set that is now present in the cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ensured {
    pub dir: PathBuf,
    pub checksum: String,
    /// `false` if the cache already held an entry pinned to this checksum, so a
    /// caller can report (or prompt about) only the fetches that actually happen.
    pub fetched: bool,
}

/// Whether the cache already holds `name` pinned to `expected_sha256`.
///
/// An entry whose sidecar pin is absent or different is *not* current: it was
/// either left by an interrupted write or fetched by a build pinning different
/// data, and in both cases the compiled-in pin is the authority.
fn is_current(cache_root: &Path, name: &str, expected_sha256: &str) -> bool {
    let dir = pdf_cache_dir(cache_root, name);
    dir.is_dir() && store::read_pin(&dir).as_deref() == Some(expected_sha256)
}

/// Whether [`ensure_pdf_set`] would return without fetching anything.
pub fn pdf_set_is_cached(cache_root: &Path, name: &str) -> bool {
    pinned_pdf_set(name).is_some_and(|pin| is_current(cache_root, name, pin.sha256))
}

/// Make `name` available under `cache_root`, fetching it only if the cache does
/// not already hold it pinned to this build's checksum.
pub fn ensure_pdf_set(
    cache_root: &Path,
    name: &str,
    fetch: &dyn Fetch,
) -> Result<Ensured, EnsureError> {
    let pin = pinned_pdf_set(name).ok_or_else(|| EnsureError::Unpinned {
        name: name.to_string(),
        known: pinned_names(),
    })?;
    ensure_pdf_set_pinned(cache_root, name, pin.url, pin.sha256, fetch)
}

/// [`ensure_pdf_set`] against an explicitly supplied pin rather than the
/// compiled-in table.
pub fn ensure_pdf_set_pinned(
    cache_root: &Path,
    name: &str,
    url: &str,
    sha256: &str,
    fetch: &dyn Fetch,
) -> Result<Ensured, EnsureError> {
    if is_current(cache_root, name, sha256) {
        return Ok(Ensured {
            dir: pdf_cache_dir(cache_root, name),
            checksum: sha256.to_string(),
            fetched: false,
        });
    }
    let verified = VerifiedFetch::new(fetch, sha256);
    match store::cache_pdf_set(cache_root, name, url, &verified) {
        Ok(cached) => Ok(Ensured {
            dir: cached.dir,
            checksum: cached.checksum,
            fetched: true,
        }),
        Err(err) => Err(match verified.mismatch() {
            Some(mismatch) => EnsureError::ChecksumMismatch(mismatch),
            None => EnsureError::Store(err),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn scratch(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "vibegraph-cache-pinned-test-{label}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A `.tar.gz` wrapping `files` in one top-level directory, the packaging
    /// LHAPDF's own set archives use.
    fn tar_gz(wrapper: &str, files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        for (path, content) in files {
            let mut header = tar::Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, format!("{wrapper}/{path}"), *content)
                .unwrap();
        }
        let tar_bytes = builder.into_inner().unwrap();
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&tar_bytes).unwrap();
        encoder.finish().unwrap()
    }

    fn sample_archive() -> Vec<u8> {
        tar_gz(
            "TestSet",
            &[("TestSet.info", b"SetDesc: \"test\"\n" as &[u8])],
        )
    }

    /// A [`Fetch`] recording how many times it was called, to distinguish
    /// "served from cache" from "fetched again".
    struct CountingFetch {
        bytes: Vec<u8>,
        calls: RefCell<usize>,
    }

    impl CountingFetch {
        fn new(bytes: Vec<u8>) -> Self {
            Self {
                bytes,
                calls: RefCell::new(0),
            }
        }
        fn calls(&self) -> usize {
            *self.calls.borrow()
        }
    }

    impl Fetch for CountingFetch {
        fn fetch(&self, _url: &str) -> Result<Vec<u8>, FetchError> {
            *self.calls.borrow_mut() += 1;
            Ok(self.bytes.clone())
        }
    }

    #[test]
    fn the_default_set_is_pinned() {
        let pin = pinned_pdf_set(DEFAULT_PDF_SET).expect("default set must have a download pin");
        assert_eq!(pin.name, DEFAULT_PDF_SET);
        assert!(pin.archive_bytes > 0);
    }

    /// Every pin's checksum is a well-formed lowercase-hex SHA-256, and every
    /// set served by the LHAPDF data server agrees with the URL pattern
    /// [`store::lhapdf_download_url`] builds — so the stored URL and the
    /// pattern cannot drift apart unnoticed.
    #[test]
    fn pins_are_well_formed_and_agree_with_the_lhapdf_url_pattern() {
        for pin in PINNED_PDF_SETS {
            assert_eq!(
                pin.sha256.len(),
                64,
                "{}: sha256 must be 64 hex characters",
                pin.name
            );
            assert!(
                pin.sha256
                    .chars()
                    .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
                "{}: sha256 must be lowercase hex, matching digest_bytes",
                pin.name
            );
            if pin.url.starts_with("https://lhapdfsets.web.cern.ch/") {
                assert_eq!(
                    pin.url,
                    store::lhapdf_download_url(pin.name),
                    "{}: pinned URL diverged from the LHAPDF download pattern",
                    pin.name
                );
            }
        }
    }

    #[test]
    fn an_unpinned_set_name_is_refused_without_fetching() {
        let cache_root = scratch("unpinned");
        let fetch = CountingFetch::new(sample_archive());
        let err = ensure_pdf_set(&cache_root, "NoSuchSet_v99", &fetch).unwrap_err();
        assert!(matches!(err, EnsureError::Unpinned { .. }), "{err:?}");
        assert_eq!(
            fetch.calls(),
            0,
            "an unpinned name must not reach the network"
        );
    }

    #[test]
    fn an_archive_matching_its_pin_is_published() {
        let cache_root = scratch("match");
        let archive = sample_archive();
        let sha = digest_bytes(&archive);

        let ensured = ensure_pdf_set_pinned(
            &cache_root,
            "TestSet",
            "https://example.invalid/TestSet.tar.gz",
            &sha,
            &store::FixedFetch(archive),
        )
        .expect("a matching archive must be published");

        assert!(ensured.fetched);
        assert_eq!(ensured.checksum, sha);
        assert!(ensured.dir.join("TestSet.info").is_file());
        assert_eq!(store::read_pin(&ensured.dir).as_deref(), Some(sha.as_str()));
    }

    /// The security-relevant case: bytes that do not hash to the pin are
    /// rejected as a checksum mismatch specifically (not as a generic fetch
    /// failure), and nothing is left in the cache for `locate` to find.
    #[test]
    fn an_archive_failing_its_pin_is_rejected_and_nothing_is_published() {
        let cache_root = scratch("tamper");
        let pinned_sha = digest_bytes(b"the archive this build was validated against");
        let substituted = sample_archive();

        let err = ensure_pdf_set_pinned(
            &cache_root,
            "TestSet",
            "https://example.invalid/TestSet.tar.gz",
            &pinned_sha,
            &store::FixedFetch(substituted.clone()),
        )
        .unwrap_err();

        match err {
            EnsureError::ChecksumMismatch(m) => {
                assert_eq!(m.expected, pinned_sha);
                assert_eq!(m.actual, digest_bytes(&substituted));
            }
            other => panic!("expected a checksum mismatch, got {other:?}"),
        }
        assert!(
            !pdf_cache_dir(&cache_root, "TestSet").exists(),
            "a mismatched archive must never be published"
        );
    }

    /// A fetch that fails outright stays a store error, so the mismatch class
    /// above is not just swallowing every failure.
    #[test]
    fn a_failing_fetch_is_not_reported_as_a_checksum_mismatch() {
        let cache_root = scratch("refuse");
        let err = ensure_pdf_set_pinned(
            &cache_root,
            "TestSet",
            "https://example.invalid/TestSet.tar.gz",
            &digest_bytes(b"whatever"),
            &store::RefusingFetch,
        )
        .unwrap_err();
        assert!(
            matches!(err, EnsureError::Store(StoreError::Fetch(_))),
            "{err:?}"
        );
    }

    #[test]
    fn a_cached_set_is_served_without_refetching() {
        let cache_root = scratch("reuse");
        let archive = sample_archive();
        let sha = digest_bytes(&archive);
        let fetch = CountingFetch::new(archive);

        let first =
            ensure_pdf_set_pinned(&cache_root, "TestSet", "u", &sha, &fetch).expect("first");
        assert!(first.fetched);
        assert_eq!(fetch.calls(), 1);
        assert!(pdf_set_is_cached_with(&cache_root, "TestSet", &sha));

        let second =
            ensure_pdf_set_pinned(&cache_root, "TestSet", "u", &sha, &fetch).expect("second");
        assert!(!second.fetched, "a cached set must not be refetched");
        assert_eq!(fetch.calls(), 1, "no second fetch may reach the network");
        assert_eq!(second.dir, first.dir);
    }

    /// An entry cached under a different pin than this build's is stale, not
    /// reusable: the compiled-in pin decides what the name resolves to.
    #[test]
    fn an_entry_pinned_to_another_checksum_is_replaced() {
        let cache_root = scratch("stale");
        let old = tar_gz("TestSet", &[("only_in_old.dat", b"old" as &[u8])]);
        let old_sha = digest_bytes(&old);
        ensure_pdf_set_pinned(
            &cache_root,
            "TestSet",
            "u",
            &old_sha,
            &store::FixedFetch(old),
        )
        .expect("seed the cache under the old pin");

        let new = tar_gz("TestSet", &[("only_in_new.dat", b"new" as &[u8])]);
        let new_sha = digest_bytes(&new);
        assert_ne!(old_sha, new_sha);
        assert!(!pdf_set_is_cached_with(&cache_root, "TestSet", &new_sha));

        let fetch = CountingFetch::new(new);
        let ensured = ensure_pdf_set_pinned(&cache_root, "TestSet", "u", &new_sha, &fetch)
            .expect("the stale entry must be replaced");

        assert!(ensured.fetched);
        assert_eq!(fetch.calls(), 1);
        assert!(ensured.dir.join("only_in_new.dat").is_file());
        assert!(!ensured.dir.join("only_in_old.dat").exists());
    }

    /// An entry with no pin sidecar at all — an interrupted or hand-made
    /// directory — is not mistaken for a cached set.
    #[test]
    fn an_unpinned_directory_in_the_cache_is_not_treated_as_cached() {
        let cache_root = scratch("nopin");
        let dir = pdf_cache_dir(&cache_root, "TestSet");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("TestSet.info"), b"hand-placed").unwrap();

        assert!(!pdf_set_is_cached_with(
            &cache_root,
            "TestSet",
            &digest_bytes(b"anything")
        ));
    }

    fn pdf_set_is_cached_with(cache_root: &Path, name: &str, sha256: &str) -> bool {
        is_current(cache_root, name, sha256)
    }
}
