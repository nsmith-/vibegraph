//! `vibegraph integrate` refusing to resolve a PDF set it cannot legitimately
//! obtain.
//!
//! Every test runs the binary with its working directory set to a temporary
//! directory and with `$LHAPDF_DATA_PATH`, `$LHAPATH` and `$VIBEGRAPH_PDF_DIR`
//! removed unless the test sets one, so nothing resolves a set but the route
//! under test — what a user running an installed binary sees.
//!
//! Nothing here reaches the network or the real `~/.vibegraph`: the cache root
//! is redirected with `$VIBEGRAPH_HOME` and downloads are refused outright with
//! `$VIBEGRAPH_NO_NETWORK`, so a regression that made resolution fall through to
//! a fetch fails these tests rather than quietly pulling 27 MB from CERN. That
//! also makes them need no data, so they run on a bare clone; the half that
//! needs a real fetched set is `cli_pdf_cache_fetched.rs`.

mod common;

use std::path::Path;
use std::process::Command;

use common::{integrate_from_bare_cwd, validation_dir};

use vibegraph::cache::pinned::DEFAULT_PDF_SET;
use vibegraph::ufo::identity::digest_bytes;

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
    let stderr = common::stderr_of(&output);
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

/// `$VIBEGRAPH_NO_NETWORK` refuses by being set, whatever its value, and the
/// binary and the validation fetch scripts (`validation/fetch_common.sh`) read
/// it by that one rule: `0` and the empty string refuse on both, so no value
/// stops one of them and lets the other download.
#[test]
fn the_no_network_variable_refuses_at_any_value_in_the_binary_and_the_fetch_scripts() {
    let fetch_common = Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/fetch_common.sh");
    // `vg_consent` answers without asking: consent is granted up front, so only
    // the refusal can make it say no.
    let script_consents = |value: Option<&str>| {
        let mut cmd = Command::new("bash");
        cmd.arg("-c")
            .arg(r#". "$1" && vg_consent "a test asset" "https://example.invalid/x""#)
            .arg("bash")
            .arg(&fetch_common)
            .env("VIBEGRAPH_FETCH_CONSENT", "1");
        match value {
            Some(v) => cmd.env("VIBEGRAPH_NO_NETWORK", v),
            None => cmd.env_remove("VIBEGRAPH_NO_NETWORK"),
        };
        cmd.output().expect("run bash").status.success()
    };
    assert!(
        script_consents(None),
        "with the variable unset and consent given, the fetch scripts must download"
    );

    for value in ["1", "0", ""] {
        assert!(
            !script_consents(Some(value)),
            "fetch_common.sh downloads under VIBEGRAPH_NO_NETWORK={value:?}"
        );

        let home = tempfile::tempdir().unwrap();
        let cwd = tempfile::tempdir().unwrap();
        let out = tempfile::tempdir().unwrap();
        let output = common::vibegraph(cwd.path(), home.path())
            .arg("integrate")
            .arg(validation_dir().join("dy13_proc_card.dat"))
            .arg("--run-card")
            .arg(validation_dir().join("dy13_default_run_card.dat"))
            .arg("--out")
            .arg(out.path())
            .arg("-y")
            .env("VIBEGRAPH_NO_NETWORK", value)
            .output()
            .expect("spawn vibegraph");
        let stderr = common::stderr_of(&output);
        assert!(
            !output.status.success()
                && stderr.contains("downloads are disabled by VIBEGRAPH_NO_NETWORK"),
            "the binary did not refuse under VIBEGRAPH_NO_NETWORK={value:?}:\n{stderr}"
        );
    }
}

/// A set name this build has no pin for, with nothing on disk to satisfy it, is
/// offered as a download from the LHAPDF server whose size and checksum the
/// refusal says are unknown, rather than guessed at; with downloads refused,
/// nothing is fetched or cached.
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
    let stderr = common::stderr_of(&output);
    for expected in [
        "PDF set NoSuchSet_v99 is not available locally",
        "https://lhapdfsets.web.cern.ch/current/NoSuchSet_v99.tar.gz",
        "size:    unknown (not pinned by this build)",
        "sha256:  unknown (not pinned by this build)",
        "downloads are disabled by VIBEGRAPH_NO_NETWORK",
    ] {
        assert!(
            stderr.contains(expected),
            "refusal should mention {expected:?}, got:\n{stderr}"
        );
    }
    assert!(!home.path().join("pdf").join("NoSuchSet_v99").exists());
}

/// A run card naming an `lhaid` no pinned set holds, with no
/// `pdfsets.index` anywhere and downloads refused, is refused naming the id and
/// the index download it would take to look it up.
#[test]
fn an_unindexed_lhaid_is_refused_naming_the_id() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    let card = cwd.path().join("run_card.dat");
    let text = std::fs::read_to_string(validation_dir().join("dy13_default_run_card.dat"))
        .unwrap()
        .replace("247000  = lhaid", "999990  = lhaid");
    assert!(text.contains("999990  = lhaid"));
    std::fs::write(&card, text).unwrap();

    let output = common::vibegraph(cwd.path(), home.path())
        .env("VIBEGRAPH_NO_NETWORK", "1")
        .arg("integrate")
        .arg(validation_dir().join("dy13_proc_card.dat"))
        .arg("--run-card")
        .arg(&card)
        .arg("--out")
        .arg(out.path())
        .args(["--neval", "2000", "--niter", "2"])
        .output()
        .expect("spawn vibegraph");
    assert!(!output.status.success());
    let stderr = common::stderr_of(&output);
    for expected in [
        "no LHAPDF set index is available locally to look lhaid 999990 up in",
        "https://lhapdfsets.web.cern.ch/current/pdfsets.index",
        "downloads are disabled by VIBEGRAPH_NO_NETWORK",
    ] {
        assert!(
            stderr.contains(expected),
            "refusal should mention {expected:?}, got:\n{stderr}"
        );
    }
}

/// A proton run card on a built-in `pdlabel` is refused before any set is
/// looked for, with the message that says to set `pdlabel = lhapdf`.
#[test]
fn a_built_in_pdlabel_is_refused_at_integrate() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    let card = cwd.path().join("run_card.dat");
    let text = std::fs::read_to_string(validation_dir().join("dy13_default_run_card.dat"))
        .unwrap()
        .replace("lhapdf  = pdlabel", "nn23lo1  = pdlabel");
    assert!(text.contains("nn23lo1  = pdlabel"));
    std::fs::write(&card, text).unwrap();

    let output = common::vibegraph(cwd.path(), home.path())
        .env("VIBEGRAPH_NO_NETWORK", "1")
        .arg("integrate")
        .arg(validation_dir().join("dy13_proc_card.dat"))
        .arg("--run-card")
        .arg(&card)
        .arg("--out")
        .arg(out.path())
        .output()
        .expect("spawn vibegraph");
    assert!(!output.status.success());
    let stderr = common::stderr_of(&output);
    assert!(
        stderr.contains("run card sets pdlabel = nn23lo1, MadGraph's built-in parameterisation")
            && stderr.contains("(247000 is NNPDF23_lo_as_0130_qed")
            && !stderr.contains("not available locally"),
        "{stderr}"
    );
}

/// A directory holding a placeholder `<DEFAULT_PDF_SET>/` that no parser
/// accepts. A run that resolves the set there fails loading it, naming the
/// directory, which is how a test tells where a set was resolved without
/// needing a real one.
fn placeholder_set(base: &Path) -> std::path::PathBuf {
    let dir = base.join(DEFAULT_PDF_SET);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join(format!("{DEFAULT_PDF_SET}.info")),
        "not an LHAPDF info file\n",
    )
    .unwrap();
    dir
}

/// The message of a run that resolved the set at `dir` and could not load it.
fn loaded_from(dir: &Path) -> String {
    format!(
        "cannot load PDF set {DEFAULT_PDF_SET} from {}",
        dir.display()
    )
}

/// `integrate` on the Drell-Yan cards from `cwd`, downloads refused, with
/// `env` set on top of the bare environment.
fn integrate_with_env(cwd: &Path, env: &[(&str, &std::ffi::OsStr)]) -> (bool, String) {
    let home = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    let mut cmd = common::vibegraph(cwd, home.path());
    cmd.env("VIBEGRAPH_NO_NETWORK", "1")
        .arg("integrate")
        .arg(validation_dir().join("dy13_proc_card.dat"))
        .arg("--run-card")
        .arg(validation_dir().join("dy13_default_run_card.dat"))
        .arg("--out")
        .arg(out.path())
        .args(["--fixed-budget", "--neval", "2000", "--niter", "2"]);
    for (name, value) in env {
        cmd.env(name, value);
    }
    let output = cmd.output().expect("spawn vibegraph");
    (output.status.success(), common::stderr_of(&output))
}

/// `$LHAPDF_DATA_PATH` is a `:` list read in order: an empty entry and an
/// entry without the set are passed over, and the first entry holding it is
/// where the set is read from.
#[test]
fn lhapdf_data_path_resolves_the_first_entry_holding_the_set() {
    let cwd = tempfile::tempdir().unwrap();
    let without = tempfile::tempdir().unwrap();
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let want = placeholder_set(first.path());
    placeholder_set(second.path());
    let value =
        std::env::join_paths([without.path(), Path::new(""), first.path(), second.path()]).unwrap();
    let (ok, stderr) = integrate_with_env(cwd.path(), &[("LHAPDF_DATA_PATH", &value)]);
    assert!(!ok, "a placeholder set loaded:\n{stderr}");
    assert!(stderr.contains(&loaded_from(&want)), "{stderr}");
}

/// `$LHAPATH` is read when `$LHAPDF_DATA_PATH` is unset, and not when it is
/// set, even to the empty string, as LHAPDF reads them.
#[test]
fn lhapath_is_read_only_when_lhapdf_data_path_is_unset() {
    let cwd = tempfile::tempdir().unwrap();
    let base = tempfile::tempdir().unwrap();
    let want = placeholder_set(base.path());
    let lhapath = base.path().as_os_str();

    let (ok, stderr) = integrate_with_env(cwd.path(), &[("LHAPATH", lhapath)]);
    assert!(!ok && stderr.contains(&loaded_from(&want)), "{stderr}");

    let empty = std::ffi::OsString::new();
    let (ok, stderr) = integrate_with_env(
        cwd.path(),
        &[("LHAPATH", lhapath), ("LHAPDF_DATA_PATH", &empty)],
    );
    assert!(!ok, "{stderr}");
    assert!(
        !stderr.contains("cannot load PDF set")
            && stderr.contains("downloads are disabled by VIBEGRAPH_NO_NETWORK"),
        "an empty $LHAPDF_DATA_PATH did not shadow $LHAPATH:\n{stderr}"
    );
}

/// `$VIBEGRAPH_PDF_DIR` is retired: a set reachable only through it is not
/// resolved, and the run warns once, naming `$LHAPDF_DATA_PATH`.
#[test]
fn vibegraph_pdf_dir_resolves_nothing_and_draws_a_warning() {
    let cwd = tempfile::tempdir().unwrap();
    let base = tempfile::tempdir().unwrap();
    placeholder_set(base.path());
    let (ok, stderr) = integrate_with_env(
        cwd.path(),
        &[("VIBEGRAPH_PDF_DIR", base.path().as_os_str())],
    );
    assert!(!ok, "{stderr}");
    assert!(
        !stderr.contains("cannot load PDF set")
            && stderr.contains("downloads are disabled by VIBEGRAPH_NO_NETWORK"),
        "$VIBEGRAPH_PDF_DIR still resolved the set:\n{stderr}"
    );
    let warnings: Vec<&str> = stderr
        .lines()
        .filter(|l| l.contains("$VIBEGRAPH_PDF_DIR is set"))
        .collect();
    assert_eq!(warnings.len(), 1, "{stderr}");
    assert!(warnings[0].contains("$LHAPDF_DATA_PATH"), "{stderr}");
}

/// A checkout holding `validation/pdf/<set>/`, run from its root with no
/// variable set, does not resolve the set there: nothing is relative to the
/// working directory, so the run reaches the download and refuses it.
#[test]
fn a_checkouts_validation_pdf_is_not_a_fallback() {
    let cwd = tempfile::tempdir().unwrap();
    placeholder_set(&cwd.path().join("validation/pdf"));
    let (ok, stderr) = integrate_with_env(cwd.path(), &[]);
    assert!(!ok, "{stderr}");
    assert!(
        !stderr.contains("cannot load PDF set")
            && stderr.contains("downloads are disabled by VIBEGRAPH_NO_NETWORK"),
        "the working directory's validation/pdf resolved the set:\n{stderr}"
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
