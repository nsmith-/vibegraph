//! Locating the data a run needs — PDF sets and UFO models — and deciding, once,
//! whether anything missing may be downloaded.
//!
//! A UFO model resolves `--ufo-dir` → `$VIBEGRAPH_UFO_DIR` → `~/.vibegraph` →
//! the working directory ([`locate`]); a PDF set, and LHAPDF's index, resolve
//! through the tiers of [`vibegraph::cache::lhapdf`], whose last is a download
//! this module is asked to authorise ([`pdf_fetch`]).
//! What belongs here is the part that talks to the user: which step is allowed
//! to reach the network, what it says before it does, and what it says instead
//! when it may not.
//!
//! # Only one path can download
//!
//! A [`HttpFetch`] is constructed in exactly one place in this file: immediately
//! after [`network::confirm`] returned [`Consent::Granted`], inside the fetch
//! factory [`pdf_fetch`] hands the library. The library asks that factory only
//! for a download no local tier can replace, and nothing else in the binary can
//! construct a fetch that reaches the network. That structure, not a predicate
//! on what is cached, is what makes an unattended run safe.

use std::path::{Path, PathBuf};
use std::sync::Once;

use vibegraph::cache::lhapdf::{DownloadRequest, Tiers};
use vibegraph::cache::resolve::{locate, Located, Source};
use vibegraph::cache::store::Fetch;
use vibegraph::cache::{AssetKind, LHAPATH_VAR, RETIRED_PDF_DIR_VAR};
use vibegraph::diagrams::ModelImport;

use crate::fetch::HttpFetch;
use crate::network::{self, Consent, Download, NetworkPolicy};

/// Environment override for the asset cache root, otherwise `~/.vibegraph`.
pub(crate) const CACHE_ROOT_VAR: &str = "VIBEGRAPH_HOME";

/// The last resort for a UFO model is `<cwd>/<name>/`, so a checkout with a model
/// unpacked beside it needs no flag.
const DEV_UFO_FALLBACK: &str = ".";

/// The cache root: `$VIBEGRAPH_HOME` if set, else `~/.vibegraph`.
pub(crate) fn cache_root() -> Option<PathBuf> {
    std::env::var_os(CACHE_ROOT_VAR)
        .map(PathBuf::from)
        .or_else(vibegraph::cache::default_cache_root)
}

/// An asset missing from the directory the user named, saying which of the two
/// ways of naming one did it — so the fix is to the flag or the variable that is
/// actually in force, not to the one the user is looking at.
fn not_found_where_named(what: &str, kind: AssetKind, flag: &str, located: &Located) -> String {
    let named_by = match located.source {
        Source::Flag => flag.to_string(),
        Source::Env => format!("${}", kind.env_var()),
        Source::Cache | Source::DevFallback => {
            unreachable!("only an explicit path is reported as one the user named")
        }
    };
    format!(
        "{what} not found at {}, the directory {named_by} names",
        located.dir.display()
    )
}

fn no_home(flag: &str) -> String {
    format!(
        "cannot locate a home directory for the asset cache; set ${CACHE_ROOT_VAR} or pass {flag}"
    )
}

/// The tiers a PDF set and LHAPDF's index are looked for through, from this
/// process's environment. `pdf_dir`, the directory `--pdf-dir` names, is read
/// ahead of the LHAPDF data path.
pub(crate) fn pdf_tiers(pdf_dir: Option<&Path>) -> Result<Tiers, String> {
    warn_on_retired_pdf_dir();
    let root = cache_root().ok_or_else(|| {
        format!(
            "cannot locate a home directory for the asset cache; set ${CACHE_ROOT_VAR}, or put \
             the PDF set's directory on ${}",
            AssetKind::Pdf.env_var()
        )
    })?;
    let mut tiers = Tiers::from_env(root);
    if let Some(dir) = pdf_dir {
        tiers.lhapdf_data_path.insert(0, dir.to_path_buf());
    }
    Ok(tiers)
}

/// The fetch factory a PDF lookup asks before any download: it puts the
/// request to the user under `policy`, and only on consent constructs the
/// fetch that reaches the network.
pub(crate) fn pdf_fetch(
    policy: NetworkPolicy,
) -> impl Fn(&DownloadRequest) -> Result<Box<dyn Fetch>, String> {
    move |request: &DownloadRequest| {
        let destination = request.destination.display().to_string();
        let download = Download {
            what: &request.what,
            url: &request.url,
            bytes: request.bytes,
            sha256: request.sha256.as_deref(),
            destination: &destination,
        };
        match network::confirm(policy, &download) {
            Consent::Granted => Ok(Box::new(HttpFetch::default()) as Box<dyn Fetch>),
            Consent::Refused(why) => Err(why),
        }
    }
}

/// One warning per process when `$VIBEGRAPH_PDF_DIR` is set: it is not read,
/// and an environment written for it would otherwise resolve a set somewhere
/// else, or download one, without saying why.
fn warn_on_retired_pdf_dir() {
    static WARNED: Once = Once::new();
    if std::env::var_os(RETIRED_PDF_DIR_VAR).is_none() {
        return;
    }
    WARNED.call_once(|| {
        tracing::warn!("{}", retired_pdf_dir_warning());
    });
}

fn retired_pdf_dir_warning() -> String {
    format!(
        "${RETIRED_PDF_DIR_VAR} is set, and vibegraph no longer reads it: PDF sets are found \
         through ${} (or ${LHAPATH_VAR} when it is unset), then the ~/.vibegraph cache; add the \
         directory to ${}",
        AssetKind::Pdf.env_var(),
        AssetKind::Pdf.env_var()
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
pub(crate) fn resolve_ufo_search_path(
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
        Source::Flag | Source::Env => Err(not_found_where_named(
            &format!("model `{name}`"),
            AssetKind::Ufo,
            "--ufo-dir",
            &located,
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
            particle_naming: Default::default(),
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

    /// A miss under the flag and a miss under the environment variable name
    /// different things to change, so their messages differ in exactly that.
    #[test]
    fn a_miss_names_the_flag_or_the_variable_that_chose_the_path() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().join("nowhere");
        let env = std::ffi::OsString::from(&base);
        let by_flag = ufo_search_path("MyModel", Some(&base), None, tmp.path()).unwrap_err();
        let by_env = ufo_search_path("MyModel", None, Some(&env), tmp.path()).unwrap_err();
        let var = format!("${}", AssetKind::Ufo.env_var());
        assert!(
            by_flag.ends_with("the directory --ufo-dir names"),
            "{by_flag}"
        );
        assert!(!by_flag.contains(&var), "{by_flag}");
        assert!(
            by_env.ends_with(&format!("the directory {var} names")),
            "{by_env}"
        );
        assert!(!by_env.contains("--ufo-dir"), "{by_env}");
    }

    /// The retired variable's warning names the variable that replaced it.
    #[test]
    fn the_retired_pdf_dir_warning_names_lhapdf_data_path() {
        let msg = retired_pdf_dir_warning();
        assert!(msg.starts_with("$VIBEGRAPH_PDF_DIR is set"), "{msg}");
        assert!(msg.contains("$LHAPDF_DATA_PATH"), "{msg}");
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
