//! Where an LHAPDF set, and LHAPDF's `pdfsets.index`, are read from.
//!
//! Both are looked for through the same tiers, first hit wins:
//!
//! 1. each entry of the LHAPDF data path ([`super::resolve::pdf_data_path`]);
//! 2. the CVMFS LHAPDF area ([`CVMFS_LHAPDF_SETS`]), when it is mounted;
//! 3. the `~/.vibegraph` cache;
//! 4. an HTTPS download from the LHAPDF data server, into the cache.
//!
//! Tiers 1 and 2 are used as found. The cache decides what a set name means on
//! the user's behalf, so there a set this build pins is current only if its
//! sidecar records the pinned archive checksum, and a set it does not pin only
//! if the sidecar exists at all, which marks an extraction that completed.
//!
//! # Consent stays with the caller
//!
//! No function here reaches the network itself. A download is described as a
//! [`DownloadRequest`] and handed to the caller's [`FetchFactory`], which
//! answers with a [`Fetch`] to read the bytes through or with the reason it
//! refuses. Asking the user, and constructing anything that can open a socket,
//! is therefore the factory's business alone, and a lookup that a local tier
//! satisfies never calls it.

use std::path::{Path, PathBuf};

use super::index::{SetIndexTable, INDEX_FILENAME};
use super::pinned::{ensure_pdf_set_pinned, pinned_pdf_set, EnsureError};
use super::store::{self, read_pin, Fetch, FetchError};
use super::AssetKind;

/// The CVMFS directory LHAPDF's sets are published under, one unpacked
/// `<set>/` per set beside its `<set>.tar.gz`, with `pdfsets.index` at the top.
pub const CVMFS_LHAPDF_SETS: &str = "/cvmfs/sft.cern.ch/lcg/external/lhapdfsets/current";

/// The LHAPDF data server's directory of set archives and the current index.
pub const LHAPDF_SETS_URL: &str = "https://lhapdfsets.web.cern.ch/current";

/// The tier a set or an index was found in. Named, never a path, wherever a
/// record leaves the machine it was made on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// An entry of `$LHAPDF_DATA_PATH`.
    LhapdfDataPath,
    /// An entry of `$LHAPATH`, read because `$LHAPDF_DATA_PATH` is unset.
    Lhapath,
    Cvmfs,
    Cache,
    /// Fetched into the cache by this lookup.
    Download,
}

impl Tier {
    pub fn name(self) -> &'static str {
        match self {
            Tier::LhapdfDataPath => "lhapdf_data_path",
            Tier::Lhapath => "lhapath",
            Tier::Cvmfs => "cvmfs",
            Tier::Cache => "cache",
            Tier::Download => "download",
        }
    }
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// The local places a lookup reads, every one a parameter so that a test can
/// lay out each tier in a temporary directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tiers {
    /// The LHAPDF data path, in order.
    pub lhapdf_data_path: Vec<PathBuf>,
    /// Which variable [`lhapdf_data_path`](Self::lhapdf_data_path) came from:
    /// [`Tier::LhapdfDataPath`] or [`Tier::Lhapath`].
    pub data_path_tier: Tier,
    /// The CVMFS LHAPDF directory, `None` where it is not mounted.
    pub cvmfs: Option<PathBuf>,
    /// The `~/.vibegraph` directory; sets and the fetched index live under its
    /// `pdf/`.
    pub cache_root: PathBuf,
}

impl Tiers {
    /// The tiers of this process: the LHAPDF data path from `$LHAPDF_DATA_PATH`
    /// (or `$LHAPATH` when it is unset), and [`CVMFS_LHAPDF_SETS`] if it is a
    /// directory.
    pub fn from_env(cache_root: PathBuf) -> Self {
        let data = std::env::var_os(AssetKind::Pdf.env_var());
        let lhapath = std::env::var_os(super::LHAPATH_VAR);
        let data_path_tier = if data.is_none() && lhapath.is_some() {
            Tier::Lhapath
        } else {
            Tier::LhapdfDataPath
        };
        let cvmfs = Path::new(CVMFS_LHAPDF_SETS);
        Tiers {
            lhapdf_data_path: super::resolve::pdf_data_path(data.as_deref(), lhapath.as_deref()),
            data_path_tier,
            cvmfs: cvmfs.is_dir().then(|| cvmfs.to_path_buf()),
            cache_root,
        }
    }

    /// `<cache_root>/pdf/`, where fetched sets and the fetched index live.
    pub fn cache_pdf_dir(&self) -> PathBuf {
        self.cache_root.join(AssetKind::Pdf.cache_subdir())
    }

    /// The local copies of `pdfsets.index` that exist, in tier order.
    pub fn local_indices(&self) -> Vec<(PathBuf, Tier)> {
        let data_path = self
            .lhapdf_data_path
            .iter()
            .map(|entry| (entry.join(INDEX_FILENAME), self.data_path_tier));
        let cvmfs = self
            .cvmfs
            .iter()
            .map(|root| (root.join(INDEX_FILENAME), Tier::Cvmfs));
        let cache = std::iter::once((self.cache_pdf_dir().join(INDEX_FILENAME), Tier::Cache));
        data_path
            .chain(cvmfs)
            .chain(cache)
            .filter(|(path, _)| path.is_file())
            .collect()
    }
}

/// A download a lookup needs, in the terms a consent prompt shows. A set this
/// build does not pin has no size or checksum to state in advance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadRequest {
    /// What the bytes are, phrased for a sentence: `"PDF set NNPDF23_…"`.
    pub what: String,
    pub url: String,
    pub bytes: Option<u64>,
    pub sha256: Option<String>,
    /// Where the download lands, which is also where a user who fetches it
    /// themselves should put it.
    pub destination: PathBuf,
}

/// The caller's answer to a [`DownloadRequest`]: something to fetch through,
/// or the reason it may not, which a lookup reports verbatim.
pub type FetchFactory<'a> = dyn Fn(&DownloadRequest) -> Result<Box<dyn Fetch>, String> + 'a;

/// A set directory and the tier it was found in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundSet {
    pub dir: PathBuf,
    pub tier: Tier,
}

#[derive(Debug, thiserror::Error)]
pub enum LookupError {
    /// The fetch factory refused a download; its message, verbatim.
    #[error("{0}")]
    Refused(String),
    #[error("`{name}` is not an LHAPDF set name")]
    NotASetName { name: String },
    #[error("cannot obtain PDF set {name}: {source}")]
    Obtain { name: String, source: EnsureError },
    #[error("cannot fetch the LHAPDF set index from {url}: {source}")]
    IndexFetch { url: String, source: FetchError },
    #[error("cannot write {path}: {source}")]
    Write {
        path: String,
        source: std::io::Error,
    },
}

/// Whether `name` can name a set directory. CVMFS keeps `@DEPRECATED/`,
/// `@UNVALIDATED/` and a `<set>.tar.gz` beside each `<set>/` in the same
/// directory as the sets, and none of those is a set.
fn is_set_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('@')
        && !name.starts_with('.')
        && !name.ends_with(".tar.gz")
        && !name.contains(['/', '\\'])
}

/// Whether the cache holds `name` as a lookup may use it: at its pin when this
/// build pins it, otherwise with the sidecar a completed extraction leaves.
pub(crate) fn cached_set_is_current(cache_root: &Path, name: &str) -> bool {
    let dir = AssetKind::Pdf.entry_dir(cache_root, name);
    if !dir.is_dir() {
        return false;
    }
    match pinned_pdf_set(name) {
        Some(pin) => read_pin(&dir).as_deref() == Some(pin.sha256),
        None => read_pin(&dir).is_some(),
    }
}

/// Find set `name` through the tiers, downloading it into the cache through
/// `fetch` only when no local tier has it.
pub fn locate_set(
    name: &str,
    tiers: &Tiers,
    fetch: &FetchFactory<'_>,
) -> Result<FoundSet, LookupError> {
    if !is_set_name(name) {
        return Err(LookupError::NotASetName {
            name: name.to_string(),
        });
    }
    if let Some(dir) = tiers
        .lhapdf_data_path
        .iter()
        .map(|entry| entry.join(name))
        .find(|dir| dir.is_dir())
    {
        return Ok(FoundSet {
            dir,
            tier: tiers.data_path_tier,
        });
    }
    if let Some(dir) = tiers
        .cvmfs
        .as_ref()
        .map(|root| root.join(name))
        .filter(|dir| dir.is_dir())
    {
        return Ok(FoundSet {
            dir,
            tier: Tier::Cvmfs,
        });
    }
    let destination = AssetKind::Pdf.entry_dir(&tiers.cache_root, name);
    if cached_set_is_current(&tiers.cache_root, name) {
        return Ok(FoundSet {
            dir: destination,
            tier: Tier::Cache,
        });
    }

    let pin = pinned_pdf_set(name);
    let url = match pin {
        Some(pin) => pin.url.to_string(),
        None => store::lhapdf_download_url(name),
    };
    let request = DownloadRequest {
        what: format!("PDF set {name}"),
        url: url.clone(),
        bytes: pin.map(|p| p.archive_bytes),
        sha256: pin.map(|p| p.sha256.to_string()),
        destination,
    };
    let fetcher = fetch(&request).map_err(LookupError::Refused)?;
    let obtain = |source| LookupError::Obtain {
        name: name.to_string(),
        source,
    };
    let dir = match pin {
        Some(pin) => {
            ensure_pdf_set_pinned(&tiers.cache_root, name, &url, pin.sha256, fetcher.as_ref())
                .map_err(obtain)?
                .dir
        }
        None => {
            store::cache_pdf_set(&tiers.cache_root, name, &url, fetcher.as_ref())
                .map_err(|e| obtain(EnsureError::Store(e)))?
                .dir
        }
    };
    tracing::info!("PDF set {name} cached at {}", dir.display());
    Ok(FoundSet {
        dir,
        tier: Tier::Download,
    })
}

/// Fetch the current `pdfsets.index` into the cache through `fetch`, replacing
/// any copy there, and return where it was written.
pub fn fetch_index(tiers: &Tiers, fetch: &FetchFactory<'_>) -> Result<PathBuf, LookupError> {
    let destination = tiers.cache_pdf_dir().join(INDEX_FILENAME);
    let url = format!("{LHAPDF_SETS_URL}/{INDEX_FILENAME}");
    let request = DownloadRequest {
        what: format!("the LHAPDF set index {INDEX_FILENAME}"),
        url: url.clone(),
        bytes: None,
        sha256: None,
        destination: destination.clone(),
    };
    let fetcher = fetch(&request).map_err(LookupError::Refused)?;
    let bytes = fetcher
        .fetch(&url)
        .map_err(|source| LookupError::IndexFetch {
            url: url.clone(),
            source,
        })?;
    let write = |path: &Path, source| LookupError::Write {
        path: path.display().to_string(),
        source,
    };
    let dir = tiers.cache_pdf_dir();
    std::fs::create_dir_all(&dir).map_err(|e| write(&dir, e))?;
    let staging = dir.join(format!(".{INDEX_FILENAME}-{}", std::process::id()));
    std::fs::write(&staging, &bytes).map_err(|e| write(&staging, e))?;
    std::fs::rename(&staging, &destination).map_err(|e| write(&destination, e))?;
    Ok(destination)
}

/// Read and parse one index file.
pub(crate) fn read_index(path: &Path) -> Result<SetIndexTable, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    SetIndexTable::parse(&text, path).map_err(|e| e.to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::cache::pinned::DEFAULT_PDF_SET;
    use crate::ufo::identity::digest_bytes;
    use std::cell::RefCell;
    use std::collections::BTreeMap;
    use std::io::Write;
    use std::rc::Rc;

    /// An in-memory server: URL → bytes. A URL it does not hold fails.
    #[derive(Clone, Default)]
    pub(crate) struct MapFetch {
        pub(crate) files: BTreeMap<String, Vec<u8>>,
    }

    impl Fetch for MapFetch {
        fn fetch(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            self.files
                .get(url)
                .cloned()
                .ok_or_else(|| FetchError(format!("404 {url}")))
        }
    }

    /// A fetch factory that records every request it was asked about and either
    /// grants each with `server` or refuses each.
    pub(crate) struct Factory {
        pub(crate) server: MapFetch,
        pub(crate) grant: bool,
        pub(crate) asked: Rc<RefCell<Vec<DownloadRequest>>>,
    }

    impl Factory {
        pub(crate) fn granting(server: MapFetch) -> Self {
            Factory {
                server,
                grant: true,
                asked: Rc::default(),
            }
        }

        pub(crate) fn refusing() -> Self {
            Factory {
                server: MapFetch::default(),
                grant: false,
                asked: Rc::default(),
            }
        }

        pub(crate) fn callback(
            &self,
        ) -> impl Fn(&DownloadRequest) -> Result<Box<dyn Fetch>, String> {
            let server = self.server.clone();
            let grant = self.grant;
            let asked = Rc::clone(&self.asked);
            move |request: &DownloadRequest| {
                asked.borrow_mut().push(request.clone());
                if grant {
                    Ok(Box::new(server.clone()) as Box<dyn Fetch>)
                } else {
                    Err(format!("refused: {}", request.what))
                }
            }
        }

        pub(crate) fn asked(&self) -> Vec<DownloadRequest> {
            self.asked.borrow().clone()
        }
    }

    /// A `.tar.gz` of `files` under one top-level `<wrapper>/`, as LHAPDF
    /// packs a set.
    pub(crate) fn tar_gz(wrapper: &str, files: &[(String, Vec<u8>)]) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        for (path, content) in files {
            let mut header = tar::Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, format!("{wrapper}/{path}"), content.as_slice())
                .unwrap();
        }
        let tar_bytes = builder.into_inner().unwrap();
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(&tar_bytes).unwrap();
        encoder.finish().unwrap()
    }

    pub(crate) fn tiers(root: &Path) -> Tiers {
        Tiers {
            lhapdf_data_path: Vec::new(),
            data_path_tier: Tier::LhapdfDataPath,
            cvmfs: None,
            cache_root: root.join("home"),
        }
    }

    fn set_dir(base: &Path, name: &str) -> PathBuf {
        let dir = base.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{name}.info")), name).unwrap();
        dir
    }

    fn set_url(name: &str) -> String {
        store::lhapdf_download_url(name)
    }

    fn archive(name: &str) -> Vec<u8> {
        tar_gz(name, &[(format!("{name}.info"), b"archived".to_vec())])
    }

    /// Each local tier alone serves the set, and each shadows every later one;
    /// the fetch factory is never asked while a local tier has the set.
    #[test]
    fn each_local_tier_serves_the_set_and_shadows_the_next() {
        let root = tempfile::tempdir().unwrap();
        let mut t = tiers(root.path());
        let data = root.path().join("data");
        let cvmfs = root.path().join("cvmfs");
        let factory = Factory::refusing();
        let fetch = factory.callback();

        let cached = AssetKind::Pdf.entry_dir(&t.cache_root, "Set");
        std::fs::create_dir_all(&cached).unwrap();
        std::fs::write(cached.join(store::PIN_FILENAME), "whatever").unwrap();
        let found = locate_set("Set", &t, &fetch).unwrap();
        assert_eq!((found.dir, found.tier), (cached, Tier::Cache));

        let in_cvmfs = set_dir(&cvmfs, "Set");
        t.cvmfs = Some(cvmfs.clone());
        let found = locate_set("Set", &t, &fetch).unwrap();
        assert_eq!((found.dir, found.tier), (in_cvmfs, Tier::Cvmfs));

        let in_data = set_dir(&data, "Set");
        t.lhapdf_data_path = vec![root.path().join("empty"), data.clone()];
        let found = locate_set("Set", &t, &fetch).unwrap();
        assert_eq!(
            (found.dir, found.tier),
            (in_data.clone(), Tier::LhapdfDataPath)
        );

        t.data_path_tier = Tier::Lhapath;
        let found = locate_set("Set", &t, &fetch).unwrap();
        assert_eq!((found.dir, found.tier), (in_data, Tier::Lhapath));

        assert!(factory.asked().is_empty(), "{:?}", factory.asked());
    }

    /// A CVMFS-shaped directory: the set only under `@DEPRECATED/` or only as
    /// its `.tar.gz` twin is not found there, and the names CVMFS keeps beside
    /// the sets are not set names at all.
    #[test]
    fn cvmfs_deprecated_and_tarball_entries_are_not_sets() {
        let root = tempfile::tempdir().unwrap();
        let cvmfs = root.path().join("cvmfs");
        set_dir(&cvmfs.join("@DEPRECATED"), "Old");
        set_dir(&cvmfs.join("@UNVALIDATED"), "New");
        std::fs::write(cvmfs.join("Packed.tar.gz"), b"gz").unwrap();
        let mut t = tiers(root.path());
        t.cvmfs = Some(cvmfs.clone());
        let factory = Factory::refusing();
        let fetch = factory.callback();

        for name in ["Old", "New", "Packed"] {
            let err = locate_set(name, &t, &fetch).unwrap_err();
            assert!(matches!(err, LookupError::Refused(_)), "{name}: {err}");
        }
        for name in ["@DEPRECATED", "@UNVALIDATED", "Packed.tar.gz", "a/b", ""] {
            let err = locate_set(name, &t, &fetch).unwrap_err();
            assert!(
                matches!(err, LookupError::NotASetName { .. }),
                "{name}: {err}"
            );
        }
        assert_eq!(factory.asked().len(), 3);
    }

    /// An unpinned set downloads from the LHAPDF server with consent: the
    /// request carries no size or checksum, the archive is published with its
    /// sidecar, and a second lookup finds it in the cache without asking.
    #[test]
    fn an_unpinned_set_downloads_once_and_is_then_cached() {
        let root = tempfile::tempdir().unwrap();
        let t = tiers(root.path());
        let mut server = MapFetch::default();
        let bytes = archive("Unpinned");
        server.files.insert(set_url("Unpinned"), bytes.clone());
        let factory = Factory::granting(server);
        let fetch = factory.callback();

        let found = locate_set("Unpinned", &t, &fetch).unwrap();
        assert_eq!(found.tier, Tier::Download);
        assert!(found.dir.join("Unpinned.info").is_file());
        assert_eq!(read_pin(&found.dir), Some(digest_bytes(&bytes)));
        let asked = factory.asked();
        assert_eq!(asked.len(), 1);
        assert_eq!(asked[0].url, set_url("Unpinned"));
        assert_eq!((asked[0].bytes, asked[0].sha256.as_deref()), (None, None));
        assert_eq!(asked[0].destination, found.dir);

        let again = locate_set("Unpinned", &t, &fetch).unwrap();
        assert_eq!((again.dir, again.tier), (found.dir, Tier::Cache));
        assert_eq!(factory.asked().len(), 1);
    }

    /// An unpinned cache directory without the sidecar is an interrupted
    /// extraction, not a set: it is fetched again.
    #[test]
    fn an_unpinned_cache_entry_without_its_sidecar_is_refetched() {
        let root = tempfile::tempdir().unwrap();
        let t = tiers(root.path());
        set_dir(&t.cache_pdf_dir(), "Unpinned");
        let mut server = MapFetch::default();
        server
            .files
            .insert(set_url("Unpinned"), archive("Unpinned"));
        let factory = Factory::granting(server);
        let found = locate_set("Unpinned", &t, &factory.callback()).unwrap();
        assert_eq!(found.tier, Tier::Download);
        assert_eq!(
            std::fs::read(found.dir.join("Unpinned.info")).unwrap(),
            b"archived"
        );
    }

    /// A refused download is reported in the factory's words and leaves
    /// nothing in the cache.
    #[test]
    fn a_refused_download_reports_the_refusal_and_writes_nothing() {
        let root = tempfile::tempdir().unwrap();
        let t = tiers(root.path());
        let factory = Factory::refusing();
        let err = locate_set("Unpinned", &t, &factory.callback()).unwrap_err();
        assert_eq!(err.to_string(), "refused: PDF set Unpinned");
        assert!(!t.cache_pdf_dir().join("Unpinned").exists());
    }

    /// The pinned set: its request states the pin's URL, size and checksum; a
    /// cache entry at the pin is used without asking, one at another checksum
    /// is stale and refetched, and an archive that does not hash to the pin is
    /// refused with nothing published.
    #[test]
    fn a_pinned_set_keeps_its_digest() {
        let pin = pinned_pdf_set(DEFAULT_PDF_SET).unwrap();
        let root = tempfile::tempdir().unwrap();
        let t = tiers(root.path());

        let mut server = MapFetch::default();
        server
            .files
            .insert(pin.url.to_string(), archive(DEFAULT_PDF_SET));
        let factory = Factory::granting(server);
        let err = locate_set(DEFAULT_PDF_SET, &t, &factory.callback()).unwrap_err();
        assert!(
            matches!(
                err,
                LookupError::Obtain {
                    source: EnsureError::ChecksumMismatch(_),
                    ..
                }
            ),
            "{err}"
        );
        assert!(!t.cache_pdf_dir().join(DEFAULT_PDF_SET).exists());
        let asked = factory.asked();
        assert_eq!(asked[0].url, pin.url);
        assert_eq!(asked[0].bytes, Some(pin.archive_bytes));
        assert_eq!(asked[0].sha256.as_deref(), Some(pin.sha256));

        let cached = set_dir(&t.cache_pdf_dir(), DEFAULT_PDF_SET);
        std::fs::write(cached.join(store::PIN_FILENAME), pin.sha256).unwrap();
        let factory = Factory::refusing();
        let found = locate_set(DEFAULT_PDF_SET, &t, &factory.callback()).unwrap();
        assert_eq!((found.dir, found.tier), (cached.clone(), Tier::Cache));
        assert!(factory.asked().is_empty());

        std::fs::write(cached.join(store::PIN_FILENAME), "stale").unwrap();
        let err = locate_set(DEFAULT_PDF_SET, &t, &factory.callback()).unwrap_err();
        assert!(matches!(err, LookupError::Refused(_)), "{err}");
        assert_eq!(factory.asked().len(), 1);
    }

    /// The index is fetched with consent into the cache, replacing a stale copy,
    /// and its request states neither size nor checksum.
    #[test]
    fn the_index_is_fetched_into_the_cache_replacing_a_stale_copy() {
        let root = tempfile::tempdir().unwrap();
        let t = tiers(root.path());
        std::fs::create_dir_all(t.cache_pdf_dir()).unwrap();
        std::fs::write(t.cache_pdf_dir().join(INDEX_FILENAME), "1 Stale 1\n").unwrap();
        let url = format!("{LHAPDF_SETS_URL}/{INDEX_FILENAME}");
        let mut server = MapFetch::default();
        server.files.insert(url.clone(), b"2 Fresh 1\n".to_vec());
        let factory = Factory::granting(server);

        let path = fetch_index(&t, &factory.callback()).unwrap();
        assert_eq!(path, t.cache_pdf_dir().join(INDEX_FILENAME));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "2 Fresh 1\n");
        let asked = factory.asked();
        assert_eq!(asked.len(), 1);
        assert_eq!(asked[0].url, url);
        assert_eq!((asked[0].bytes, asked[0].sha256.as_deref()), (None, None));

        let err = fetch_index(&t, &Factory::refusing().callback()).unwrap_err();
        assert!(matches!(err, LookupError::Refused(_)), "{err}");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "2 Fresh 1\n");
    }

    /// Local indices are listed in tier order, only where a file exists.
    #[test]
    fn local_indices_follow_the_tier_order() {
        let root = tempfile::tempdir().unwrap();
        let mut t = tiers(root.path());
        let (a, b, cvmfs) = (
            root.path().join("a"),
            root.path().join("b"),
            root.path().join("cvmfs"),
        );
        for dir in [&b, &cvmfs, &t.cache_pdf_dir()] {
            std::fs::create_dir_all(dir).unwrap();
            std::fs::write(dir.join(INDEX_FILENAME), "").unwrap();
        }
        t.lhapdf_data_path = vec![a, b.clone()];
        t.cvmfs = Some(cvmfs.clone());
        assert_eq!(
            t.local_indices(),
            vec![
                (b.join(INDEX_FILENAME), Tier::LhapdfDataPath),
                (cvmfs.join(INDEX_FILENAME), Tier::Cvmfs),
                (t.cache_pdf_dir().join(INDEX_FILENAME), Tier::Cache),
            ]
        );
    }
}
