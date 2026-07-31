//! `vibegraph integrate` obtaining its PDF set through the `~/.vibegraph` cache
//! rather than a dev checkout.
//!
//! Every test runs the binary with its working directory set to an empty
//! temporary directory, so the repo-local `validation/pdf` dev fallback cannot
//! resolve — what a user running an installed binary sees. That is what lets
//! these tests tell a working cache path from a dev fallback masking a broken
//! one.
//!
//! Nothing here reaches the network or the real `~/.vibegraph`: the cache root
//! is redirected with `$VIBEGRAPH_HOME` and downloads are refused outright with
//! `$VIBEGRAPH_NO_NETWORK`, so a regression that made resolution fall through to
//! a fetch fails these tests rather than quietly pulling 27 MB from CERN.

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
/// directory), or `None` when the dev checkout has no fetched set to copy from.
///
/// Renaming is what keeps this test offline: under its own name the default set
/// is pinned, and an entry that does not match the compiled-in checksum is
/// correctly treated as stale and refetched.
fn local_set_archive() -> Option<Vec<u8>> {
    let src = dev_pdf_dir().join(DEFAULT_PDF_SET);
    let info = std::fs::read(src.join(format!("{DEFAULT_PDF_SET}.info"))).ok()?;
    let member = std::fs::read(src.join(format!("{DEFAULT_PDF_SET}_0000.dat"))).ok()?;

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
    Some(encoder.finish().unwrap())
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
    let Some(archive) = local_set_archive() else {
        eprintln!(
            "skipped: no PDF set at {} to build a cache entry from \
             (run `pixi run -e madgraph fetch-pdf`)",
            dev_pdf_dir().display()
        );
        return;
    };

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

/// The default set, absent from the cache, with downloads refused: the refusal
/// must name the exact URL, size, and checksum a fetch would use — the material
/// an interactive prompt needs, and proof the fetch path was reached rather than
/// silently skipped.
#[test]
fn a_missing_pinned_set_reports_what_it_would_download() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();

    let output = integrate_from_bare_cwd(cwd.path(), home.path(), out.path(), &[]);
    assert!(
        !output.status.success(),
        "a missing set with downloads refused must fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    for expected in [
        DEFAULT_PDF_SET,
        "https://lhapdfsets.web.cern.ch/current/NNPDF23_lo_as_0130_qed.tar.gz",
        "60d3c1df1c31e5840f91f4217163ae30a256b9291a5adc894882e86607ef5d63",
        "VIBEGRAPH_NO_NETWORK",
    ] {
        assert!(
            stderr.contains(expected),
            "refusal should mention {expected}, got:\n{stderr}"
        );
    }
    assert!(
        !home.path().join("pdf").join(DEFAULT_PDF_SET).exists(),
        "a refused fetch must not leave a cache entry"
    );
}

/// A set name this build has no pin for, with nothing on disk to satisfy it,
/// fails with an actionable message instead of guessing at a URL.
#[test]
fn an_unpinned_missing_set_is_refused_without_reaching_the_network() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();

    let output = integrate_from_bare_cwd(
        cwd.path(),
        home.path(),
        out.path(),
        &["--pdf-set", "NoSuchSet_v99"],
    );
    assert!(
        !output.status.success(),
        "an unpinned missing set must fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no download pin"),
        "error should say the set is unpinned, got:\n{stderr}"
    );
    assert!(
        stderr.contains("--pdf-dir"),
        "error should name the flag that would fix it, got:\n{stderr}"
    );
}

/// An explicit `--pdf-dir` still wins over the cache, so wiring the cache in did
/// not change what a dev checkout or a CI job pointing at its own data gets.
#[test]
fn an_explicit_pdf_dir_still_takes_precedence() {
    if !dev_pdf_dir().join(DEFAULT_PDF_SET).is_dir() {
        eprintln!("skipped: no fetched PDF set to point --pdf-dir at");
        return;
    }
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

/// The compiled-in pin still describes what the LHAPDF data server actually
/// serves. Ignored by default because it is the one check here that must reach
/// the network; run it deliberately after any pin change:
///
///     cargo test -p vibegraph --test cli_pdf_cache -- --ignored
#[test]
#[ignore = "downloads the pinned archive from the LHAPDF data server"]
fn pinned_checksums_match_the_upstream_archives() {
    use vibegraph::cache::pinned::PINNED_PDF_SETS;
    use vibegraph::cache::store::Fetch;

    // The binary's own HTTP client is not reachable from an integration test,
    // so this uses the same crate the binary does.
    struct Http;
    impl Fetch for Http {
        fn fetch(&self, url: &str) -> Result<Vec<u8>, vibegraph::cache::store::FetchError> {
            let mut r = ureq::get(url)
                .call()
                .map_err(|e| vibegraph::cache::store::FetchError(format!("GET {url}: {e}")))?;
            r.body_mut()
                .with_config()
                .limit(256 * 1024 * 1024)
                .read_to_vec()
                .map_err(|e| vibegraph::cache::store::FetchError(format!("read {url}: {e}")))
        }
    }

    for pin in PINNED_PDF_SETS {
        let bytes = Http
            .fetch(pin.url)
            .unwrap_or_else(|e| panic!("{}: {e}", pin.name));
        assert_eq!(
            bytes.len() as u64,
            pin.archive_bytes,
            "{}: pinned archive_bytes is stale",
            pin.name
        );
        assert_eq!(
            digest_bytes(&bytes),
            pin.sha256,
            "{}: upstream archive no longer matches the pinned checksum",
            pin.name
        );
    }
}
