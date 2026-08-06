//! Locating the data a run needs — PDF sets and UFO models — and deciding, once,
//! whether anything missing may be downloaded.
//!
//! Both kinds resolve the same way (`--flag` → environment → `~/.vibegraph` →
//! repo-local dev fallback), which is [`cache::resolve::locate`]'s job. What
//! belongs here is the part that talks to the user: which step is allowed to
//! reach the network, what it says before it does, and what it says instead when
//! it may not.
//!
//! # Only one path can download
//!
//! A [`HttpFetch`] is constructed in exactly one place in this file: immediately
//! after [`network::confirm`] returned [`Consent::Granted`]. Every other call
//! into the cache layer passes [`RefusingFetch`], so a resolution step that was
//! not supposed to fetch cannot fetch even if its "is it already cached?"
//! predicate is wrong. That structure, not the predicate, is what makes an
//! unattended run safe.

use std::path::{Path, PathBuf};

use vibegraph::cache::pinned::{ensure_pdf_set, pdf_set_is_cached, pinned_pdf_set, PinnedPdfSet};
use vibegraph::cache::resolve::{locate, locate_from_env, Source};
use vibegraph::cache::store::{Fetch, RefusingFetch};
use vibegraph::cache::AssetKind;
use vibegraph::diagrams::ModelImport;

use crate::fetch::HttpFetch;
use crate::network::{self, Consent, Download, NetworkPolicy};

/// Environment override for the asset cache root, otherwise `~/.vibegraph`.
pub const CACHE_ROOT_VAR: &str = "VIBEGRAPH_HOME";

/// Repo-local PDF data, tried last. Relative to the working directory, so it
/// only ever resolves inside a dev checkout and is simply absent for a user
/// running an installed binary.
const DEV_PDF_FALLBACK: &str = "validation/pdf";

/// UFO models have historically been read from `<cwd>/<name>/`, which stays the
/// last resort so a checkout with a model unpacked beside it keeps working.
const DEV_UFO_FALLBACK: &str = ".";

/// The cache root: `$VIBEGRAPH_HOME` if set, else `~/.vibegraph`.
pub fn cache_root() -> Option<PathBuf> {
    std::env::var_os(CACHE_ROOT_VAR)
        .map(PathBuf::from)
        .or_else(vibegraph::cache::default_cache_root)
}

fn no_home(flag: &str) -> String {
    format!(
        "cannot locate a home directory for the asset cache; set ${CACHE_ROOT_VAR} or pass {flag}"
    )
}

/// Resolve the directory holding `<pdf_set>.info` and its members, downloading
/// the set into the cache if the resolution order turns up nothing and the user
/// agrees to it.
///
/// A set reached through `--pdf-dir`, `$VIBEGRAPH_PDF_DIR`, or the dev fallback
/// is used as found and never checksum-checked: those name data the caller is
/// pointing at deliberately, which is also what keeps sets this build has no pin
/// for usable. The cache step is the one that decides on the user's behalf what
/// a bare set name means, so there the compiled-in pin is authoritative — an
/// entry pinned to anything else is refetched rather than trusted.
pub fn resolve_pdf_set_dir(
    pdf_set: &str,
    pdf_dir: Option<&Path>,
    policy: NetworkPolicy,
) -> Result<PathBuf, String> {
    let root = cache_root().ok_or_else(|| no_home("--pdf-dir"))?;
    let located = locate_from_env(
        AssetKind::Pdf,
        pdf_set,
        pdf_dir,
        Some(&root),
        Some(Path::new(DEV_PDF_FALLBACK)),
    )
    .ok_or_else(|| "cannot resolve the PDF cache root".to_string())?;

    match located.source {
        Source::Flag | Source::Env if located.found => return Ok(located.dir),
        Source::Flag | Source::Env => {
            return Err(format!(
                "PDF set {pdf_set} not found at {}",
                located.dir.display()
            ))
        }
        Source::DevFallback => return Ok(located.dir),
        Source::Cache => {}
    }

    let Some(pin) = pinned_pdf_set(pdf_set) else {
        if located.found {
            return Ok(located.dir);
        }
        return Err(format!(
            "PDF set {pdf_set} is not present locally and this build has no download pin for it; \
             fetch it yourself and point --pdf-dir / ${} at the directory containing it",
            AssetKind::Pdf.env_var()
        ));
    };

    let fetcher: Box<dyn Fetch> = if pdf_set_is_cached(&root, pdf_set) {
        // Already there: nothing to authorise, and nothing that may fetch.
        Box::new(RefusingFetch)
    } else {
        match ask_for(pin, &located.dir, policy) {
            Consent::Granted => Box::new(HttpFetch::default()),
            Consent::Refused(why) => return Err(why),
        }
    };

    let ensured = ensure_pdf_set(&root, pdf_set, fetcher.as_ref())
        .map_err(|e| format!("cannot obtain PDF set {pdf_set}: {e}"))?;
    if ensured.fetched {
        tracing::info!("PDF set cached at {}", ensured.dir.display());
    }
    Ok(ensured.dir)
}

fn ask_for(pin: &PinnedPdfSet, destination: &Path, policy: NetworkPolicy) -> Consent {
    let what = format!("PDF set {}", pin.name);
    let destination = destination.display().to_string();
    network::confirm(
        policy,
        &Download {
            what: &what,
            url: pin.url,
            bytes: pin.archive_bytes,
            sha256: pin.sha256,
            destination: &destination,
        },
    )
}

/// Resolve the directory that a proc card's `import model` should be read from,
/// returning the *search path* — the parent a model name is joined onto — since
/// that is what [`vibegraph::config::GlobalConfig`] takes.
///
/// `None` for the import means the interned Standard Model, which is compiled
/// into the binary and needs no directory at all; the caller passes the returned
/// path through regardless, so it must be harmless when nothing reads it.
///
/// # Models are never downloaded
///
/// There is no model counterpart to the PDF pin table, and deliberately so.
/// Resolving a model *name* to an archive needs an index that maps names to
/// URLs, and FeynRules — where UFO models are actually published — has none: its
/// model database is a wiki, one page per model, with hand-attached files whose
/// names follow no rule (`<model>.tar.gz` next to `<model>_UFO.tar.gz` next to
/// `<model>_UFO.tar.2.gz` on the same page, only one of which is a UFO
/// directory). A name-derived URL is therefore not merely unverified, it is
/// wrong in two ways at once: it 404s for most models, and for some it succeeds
/// and hands back FeynRules Mathematica sources instead of a UFO. So a model
/// that is not on disk is an error naming where to put one, not a download.
pub fn resolve_ufo_search_path(
    import: Option<&ModelImport>,
    ufo_dir: Option<&Path>,
) -> Result<PathBuf, String> {
    let Some(import) = import else {
        return Ok(PathBuf::from(DEV_UFO_FALLBACK));
    };
    if import.name == "sm" {
        return Ok(PathBuf::from(DEV_UFO_FALLBACK));
    }
    let root = cache_root().ok_or_else(|| no_home("--ufo-dir"))?;
    let env = std::env::var_os(AssetKind::Ufo.env_var());
    ufo_search_path(&import.name, ufo_dir, env.as_deref(), &root)
}

/// The resolution and the error text behind [`resolve_ufo_search_path`], with
/// every input a parameter so both are exercisable without mutating process
/// state (the pattern [`vibegraph::cache::resolve::locate`] establishes).
fn ufo_search_path(
    name: &str,
    flag: Option<&Path>,
    env: Option<&std::ffi::OsStr>,
    root: &Path,
) -> Result<PathBuf, String> {
    let located = locate(
        AssetKind::Ufo,
        name,
        flag,
        env,
        root,
        Some(Path::new(DEV_UFO_FALLBACK)),
    );
    if located.found {
        return search_path_of(&located.dir);
    }

    match located.source {
        Source::Flag | Source::Env => Err(format!(
            "model `{name}` not found at {}",
            located.dir.display()
        )),
        _ => Err(format!(
            "model `{name}` was not found, and vibegraph does not download UFO models \
             (FeynRules publishes no per-model index a name could be resolved through, so there \
             is no URL this build could pin).\n\
             Download the model's UFO archive yourself and unpack it so that \
             `{}/particles.py` exists, or point --ufo-dir / ${} at the directory containing it.",
            located.dir.display(),
            AssetKind::Ufo.env_var(),
        )),
    }
}

/// The parent a model directory sits in — what a search path is.
fn search_path_of(model_dir: &Path) -> Result<PathBuf, String> {
    model_dir
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| format!("{} has no parent directory", model_dir.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn import(name: &str) -> ModelImport {
        ModelImport {
            name: name.to_string(),
            restrict_variant: None,
        }
    }

    /// The interned SM resolves without consulting the cache at all, so a
    /// Standard-Model run never depends on `$HOME` being readable.
    #[test]
    fn the_interned_sm_needs_no_directory() {
        assert_eq!(
            resolve_ufo_search_path(None, None).unwrap(),
            PathBuf::from(DEV_UFO_FALLBACK)
        );
        assert_eq!(
            resolve_ufo_search_path(Some(&import("sm")), None).unwrap(),
            PathBuf::from(DEV_UFO_FALLBACK)
        );
    }

    /// A model directory is handed back as its *parent*, since that is what the
    /// loader joins the name onto — get this wrong and the model resolves to
    /// `<dir>/<name>/<name>`. Checked on the flag and on the cache step, whose
    /// directories are built differently.
    #[test]
    fn a_located_model_becomes_its_parent_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let flag_base = tmp.path().join("elsewhere");
        std::fs::create_dir_all(flag_base.join("MyModel")).unwrap();
        assert_eq!(
            ufo_search_path("MyModel", Some(&flag_base), None, tmp.path()).unwrap(),
            flag_base
        );

        let cache_base = tmp.path().join("ufo");
        std::fs::create_dir_all(cache_base.join("MyModel")).unwrap();
        assert_eq!(
            ufo_search_path("MyModel", None, None, tmp.path()).unwrap(),
            cache_base
        );
    }

    /// The missing-model error has to be actionable on its own terms: it names
    /// the cache path to unpack into, the environment variable and the flag,
    /// and says plainly that no download will be attempted. This is the whole
    /// UFO half of the first-run experience, so its text is the deliverable.
    #[test]
    fn a_missing_model_explains_how_to_install_it_by_hand() {
        let tmp = tempfile::tempdir().unwrap();
        let err = ufo_search_path("NoSuchModel", None, None, tmp.path()).unwrap_err();
        assert!(err.contains("NoSuchModel"), "{err}");
        assert!(err.contains("does not download UFO models"), "{err}");
        assert!(err.contains("particles.py"), "{err}");
        assert!(err.contains(AssetKind::Ufo.env_var()), "{err}");
        assert!(err.contains("--ufo-dir"), "{err}");
        assert!(
            err.contains(
                &tmp.path()
                    .join("ufo")
                    .join("NoSuchModel")
                    .display()
                    .to_string()
            ),
            "the cache path to unpack into must appear: {err}"
        );
    }

    /// An explicit path that is wrong fails naming *that* path, rather than
    /// falling through to a cache entry the user did not ask for.
    #[test]
    fn an_explicit_path_that_misses_names_itself() {
        let tmp = tempfile::tempdir().unwrap();
        let flag_base = tmp.path().join("nowhere");
        let err = ufo_search_path("MyModel", Some(&flag_base), None, tmp.path()).unwrap_err();
        assert!(
            err.contains(&flag_base.join("MyModel").display().to_string()),
            "{err}"
        );
        assert!(!err.contains("does not download"), "{err}");
    }

    /// The environment variable is honoured, and the flag outranks it.
    #[test]
    fn the_flag_outranks_the_environment_variable() {
        let tmp = tempfile::tempdir().unwrap();
        let from_env = tmp.path().join("from-env");
        let from_flag = tmp.path().join("from-flag");
        std::fs::create_dir_all(from_env.join("MyModel")).unwrap();
        std::fs::create_dir_all(from_flag.join("MyModel")).unwrap();

        let env = std::ffi::OsString::from(&from_env);
        assert_eq!(
            ufo_search_path("MyModel", None, Some(&env), tmp.path()).unwrap(),
            from_env
        );
        assert_eq!(
            ufo_search_path("MyModel", Some(&from_flag), Some(&env), tmp.path()).unwrap(),
            from_flag
        );
    }
}
