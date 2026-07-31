//! PDF resolution exercised against a real fetched set.
//!
//! The refusal paths — a missing pinned set, an unpinned name — need no data and
//! stay in `cli_pdf_cache.rs`. What needs a set is the other half: that a cache
//! entry actually serves an `integrate` run, and that an explicit `--pdf-dir`
//! still wins over the cache. Both run the binary from an empty working
//! directory so the repo-local dev fallback cannot resolve, which is what lets
//! them tell a working cache path from a dev fallback masking a broken one.
//!
//! Nothing here reaches the network: `$VIBEGRAPH_NO_NETWORK` refuses downloads
//! outright, and the cache entry is published from the fetched set's own bytes
//! under a name no build pins.
//!
//!     pixi run fetch-pdf
//!     cargo test -p vibegraph --features extended-validation \
//!         --test cli_pdf_cache_fetched

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use vibegraph::cache::pinned::{ensure_pdf_set_pinned, DEFAULT_PDF_SET};
use vibegraph::cache::store::FixedFetch;
use vibegraph::ufo::identity::digest_bytes;

/// A set name no build pins, so resolving it can never reach for a URL.
const LOCAL_SET: &str = "VibegraphTestSet";

fn validation_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph")
}

fn dev_pdf_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/pdf")
}

/// The member-0 files `integrate` reads, renamed to `LOCAL_SET` and packed as a
/// `.tar.gz` in the layout LHAPDF ships (everything under one top-level
/// directory).
///
/// Renaming is what keeps this test offline: under its own name the default set
/// is pinned, and an entry that does not match the compiled-in checksum is
/// correctly treated as stale and refetched.
fn local_set_archive() -> Vec<u8> {
    let src = dev_pdf_dir().join(DEFAULT_PDF_SET);
    let read = |name: String| {
        std::fs::read(src.join(&name)).unwrap_or_else(|e| {
            panic!(
                "{}: {e} (run `pixi run fetch-pdf`)",
                src.join(&name).display()
            )
        })
    };
    let info = read(format!("{DEFAULT_PDF_SET}.info"));
    let member = read(format!("{DEFAULT_PDF_SET}_0000.dat"));

    let mut builder = tar::Builder::new(Vec::new());
    for (name, bytes) in [
        (format!("{LOCAL_SET}.info"), info),
        (format!("{LOCAL_SET}_0000.dat"), member),
    ] {
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

/// Run `integrate` from an empty working directory, with the cache root
/// redirected, downloads refused, and every other PDF resolution route removed.
fn integrate_from_bare_cwd(
    cwd: &Path,
    cache_root: &Path,
    out: &Path,
    extra: &[&str],
) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vibegraph"))
        .current_dir(cwd)
        .arg("integrate")
        .arg(validation_dir().join("dy13_proc_card.dat"))
        .arg("--run-card")
        .arg(validation_dir().join("dy13_default_run_card.dat"))
        .arg("--out")
        .arg(out)
        .args(["--neval", "2000", "--niter", "2"])
        .args(extra)
        .env("VIBEGRAPH_HOME", cache_root)
        .env("VIBEGRAPH_NO_NETWORK", "1")
        .env_remove("VIBEGRAPH_PDF_DIR")
        .output()
        .expect("spawn vibegraph")
}

/// The acceptance case: with no dev checkout reachable and no `--pdf-dir`, a set
/// sitting in `~/.vibegraph/pdf/` is what serves the run.
///
/// The cache is populated through `ensure_pdf_set_pinned` with in-memory archive
/// bytes, so the entry under test was produced by the real
/// fetch → verify → extract → publish path rather than hand-assembled.
#[test]
fn a_cached_pdf_set_serves_integrate_with_no_dev_checkout() {
    let archive = local_set_archive();

    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();

    let sha = digest_bytes(&archive);
    let ensured = ensure_pdf_set_pinned(
        home.path(),
        LOCAL_SET,
        "https://example.invalid/set.tar.gz",
        &sha,
        &FixedFetch(archive),
    )
    .expect("publish the set into the cache");
    assert!(ensured.fetched);
    assert_eq!(
        ensured.dir,
        home.path().join("pdf").join(LOCAL_SET),
        "the cache entry must land where resolution looks for it"
    );

    // The dev fallback is a relative path, so an empty working directory is
    // exactly the "installed binary, no checkout" situation.
    assert!(
        !cwd.path().join("validation/pdf").exists(),
        "the working directory must not offer a dev fallback"
    );

    let output = integrate_from_bare_cwd(
        cwd.path(),
        home.path(),
        out.path(),
        &["--pdf-set", LOCAL_SET],
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "integrate failed with a populated cache:\n{stderr}"
    );
    assert!(
        out.path().join("grid.bin.zst").is_file(),
        "integrate produced no grid artifact:\n{stderr}"
    );
}

/// An explicit `--pdf-dir` still wins over the cache, so wiring the cache in did
/// not change what a dev checkout or a CI job pointing at its own data gets.
#[test]
fn an_explicit_pdf_dir_still_takes_precedence() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();

    let output = integrate_from_bare_cwd(
        cwd.path(),
        home.path(),
        out.path(),
        &["--pdf-dir", dev_pdf_dir().to_str().unwrap()],
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "--pdf-dir run failed:\n{stderr}");
    assert!(
        !home.path().join("pdf").exists(),
        "an explicit --pdf-dir must not populate the cache:\n{stderr}"
    );
}
