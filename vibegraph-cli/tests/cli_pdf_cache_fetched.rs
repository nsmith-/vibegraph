//! PDF resolution exercised against a real fetched set.
//!
//! The refusal paths — a missing pinned set, an unpinned name — need no data and
//! stay in `cli_pdf_cache.rs`. What needs a set is the other half: that a cache
//! entry actually serves an `integrate` run, and that `$LHAPDF_DATA_PATH`
//! serves one ahead of the cache. Each runs the binary from an empty working
//! directory with no PDF variable set unless it sets one, so only the route
//! under test can resolve the set.
//!
//! Nothing here reaches the network: `$VIBEGRAPH_NO_NETWORK` refuses downloads
//! outright, and the cache entry is published from the fetched set's own bytes
//! under a name and an `lhaid` no build pins and no LHAPDF index lists, so the
//! run card reaches it only through the `pdfsets.index` the test writes.
//!
//!     pixi run fetch-pdf
//!     cargo test -p vibegraph --features extended-validation \
//!         --test cli_pdf_cache_fetched

mod common;

use std::io::Write;
use std::path::{Path, PathBuf};

use common::{validation_dir, PdfDataPath};

use vibegraph::cache::pinned::{ensure_pdf_set_pinned, DEFAULT_PDF_SET};
use vibegraph::cache::store::FixedFetch;
use vibegraph::ufo::identity::digest_bytes;

/// A set name no build pins, so resolving it can never reach for a URL.
const LOCAL_SET: &str = "VibegraphTestSet";

/// The `lhaid` of [`LOCAL_SET`]: in no set of LHAPDF's own index (it lies past
/// the last member of `DSSV14pol`, 960000, and below `JAM25pion_nlonll`,
/// 1000000), so only the index line [`local_index`] writes resolves it.
const LOCAL_LHAID: i64 = 999990;

/// A `pdfsets.index` listing [`LOCAL_SET`] at [`LOCAL_LHAID`].
fn local_index() -> String {
    format!("{LOCAL_LHAID} {LOCAL_SET} 1\n")
}

/// The Drell-Yan run card with its `lhaid` moved to [`LOCAL_LHAID`], in `dir`.
fn local_run_card(dir: &Path) -> PathBuf {
    let text = std::fs::read_to_string(validation_dir().join("dy13_default_run_card.dat"))
        .unwrap()
        .replace("247000  = lhaid", &format!("{LOCAL_LHAID}  = lhaid"));
    assert!(text.contains(&format!("{LOCAL_LHAID}  = lhaid")));
    let path = dir.join("run_card.dat");
    std::fs::write(&path, text).unwrap();
    path
}

/// `integrate` on the Drell-Yan process and [`local_run_card`], from `cwd`, with
/// the cache root at `home` and downloads refused.
fn integrate_local(cwd: &Path, home: &Path, out: &Path) -> std::process::Command {
    let mut cmd = common::vibegraph(cwd, home);
    cmd.env("VIBEGRAPH_NO_NETWORK", "1")
        .arg("integrate")
        .arg(validation_dir().join("dy13_proc_card.dat"))
        .arg("--run-card")
        .arg(local_run_card(cwd))
        .arg("--out")
        .arg(out)
        .args(["--neval", "2000", "--niter", "2"]);
    cmd
}

fn dev_pdf_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/pdf")
}

/// The member-0 files `integrate` reads, renamed to [`LOCAL_SET`] with their
/// `SetIndex` moved to [`LOCAL_LHAID`]: `(file name, bytes)`.
///
/// Renaming is what keeps these tests offline: under its own name the default
/// set is pinned, and an entry that does not match the compiled-in checksum is
/// correctly treated as stale and refetched. `with_set_index = false` drops the
/// `SetIndex` line, which the lookup refuses.
fn local_set_files(with_set_index: bool) -> Vec<(String, Vec<u8>)> {
    let src = dev_pdf_dir().join(DEFAULT_PDF_SET);
    let read = |name: String| {
        std::fs::read(src.join(&name)).unwrap_or_else(|e| {
            panic!(
                "{}: {e} (run `pixi run fetch-pdf`)",
                src.join(&name).display()
            )
        })
    };
    let info = String::from_utf8(read(format!("{DEFAULT_PDF_SET}.info"))).unwrap();
    let info: String = info
        .lines()
        .filter_map(|line| match line.trim_start().strip_prefix("SetIndex:") {
            Some(_) if with_set_index => Some(format!("SetIndex: {LOCAL_LHAID}\n")),
            Some(_) => None,
            None => Some(format!("{line}\n")),
        })
        .collect();
    assert_eq!(
        info.contains(&format!("SetIndex: {LOCAL_LHAID}")),
        with_set_index
    );
    let member = read(format!("{DEFAULT_PDF_SET}_0000.dat"));
    vec![
        (format!("{LOCAL_SET}.info"), info.into_bytes()),
        (format!("{LOCAL_SET}_0000.dat"), member),
    ]
}

/// [`local_set_files`] packed as a `.tar.gz` in the layout LHAPDF ships
/// (everything under one top-level directory).
fn local_set_archive(with_set_index: bool) -> Vec<u8> {
    let mut builder = tar::Builder::new(Vec::new());
    for (name, bytes) in local_set_files(with_set_index) {
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, format!("{LOCAL_SET}/{name}"), bytes.as_slice())
            .unwrap();
    }
    let tar_bytes = builder.into_inner().unwrap();
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(&tar_bytes).unwrap();
    encoder.finish().unwrap()
}

/// Publish [`local_set_archive`] into the cache under `home` through
/// `ensure_pdf_set_pinned`, so the entry was produced by the real
/// fetch → verify → extract → publish path rather than hand-assembled.
fn publish_local_set(home: &Path, with_set_index: bool) {
    let archive = local_set_archive(with_set_index);
    let sha = digest_bytes(&archive);
    let ensured = ensure_pdf_set_pinned(
        home,
        LOCAL_SET,
        "https://example.invalid/set.tar.gz",
        &sha,
        &FixedFetch(archive),
    )
    .expect("publish the set into the cache");
    assert!(ensured.fetched);
    assert_eq!(
        ensured.dir,
        home.join("pdf").join(LOCAL_SET),
        "the cache entry must land where resolution looks for it"
    );
}

/// The acceptance case: with no dev checkout reachable and no PDF variable set,
/// a set sitting in `~/.vibegraph/pdf/`, listed by the index there, is what
/// serves the run card's `lhaid`.
#[test]
fn a_cached_pdf_set_serves_integrate_with_no_dev_checkout() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    publish_local_set(home.path(), true);
    std::fs::write(home.path().join("pdf/pdfsets.index"), local_index()).unwrap();

    let output = integrate_local(cwd.path(), home.path(), out.path())
        .output()
        .expect("spawn vibegraph");
    let stderr = common::stderr_of(&output);
    assert!(
        output.status.success(),
        "integrate failed with a populated cache:\n{stderr}"
    );
    assert!(
        out.path().join("grid.bin.zst").is_file(),
        "integrate produced no grid artifact:\n{stderr}"
    );
}

/// `$LHAPDF_DATA_PATH` wins over the cache: with the same set in both, the
/// copy on the data path serves the run and the cached one is never read. The
/// cached copy is one the lookup refuses (its `.info` has no `SetIndex`), so a
/// run that read it would fail.
#[test]
fn lhapdf_data_path_takes_precedence_over_the_cache() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    let data = tempfile::tempdir().unwrap();
    publish_local_set(home.path(), false);
    std::fs::write(home.path().join("pdf/pdfsets.index"), local_index()).unwrap();
    let set_dir = data.path().join(LOCAL_SET);
    std::fs::create_dir(&set_dir).unwrap();
    for (name, bytes) in local_set_files(true) {
        std::fs::write(set_dir.join(name), bytes).unwrap();
    }
    std::fs::write(data.path().join("pdfsets.index"), local_index()).unwrap();

    let output = integrate_local(cwd.path(), home.path(), out.path())
        .pdf_data_path(data.path())
        .output()
        .expect("spawn vibegraph");
    let stderr = common::stderr_of(&output);
    assert!(
        output.status.success(),
        "the set on $LHAPDF_DATA_PATH did not serve the run:\n{stderr}"
    );
}

/// `$LHAPDF_DATA_PATH` naming the fetched set's directory serves the run as
/// found, and leaves the cache untouched.
#[test]
fn lhapdf_data_path_serves_a_run_without_touching_the_cache() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();

    let output = common::vibegraph(cwd.path(), home.path())
        .env("VIBEGRAPH_NO_NETWORK", "1")
        .pdf_data_path(dev_pdf_dir())
        .arg("integrate")
        .arg(validation_dir().join("dy13_proc_card.dat"))
        .arg("--run-card")
        .arg(validation_dir().join("dy13_default_run_card.dat"))
        .arg("--out")
        .arg(out.path())
        .args(["--neval", "2000", "--niter", "2"])
        .output()
        .expect("spawn vibegraph");
    let stderr = common::stderr_of(&output);
    assert!(
        output.status.success(),
        "a run on $LHAPDF_DATA_PATH failed:\n{stderr}"
    );
    assert!(
        !home.path().join("pdf").exists(),
        "a set on $LHAPDF_DATA_PATH must not populate the cache:\n{stderr}"
    );
}
