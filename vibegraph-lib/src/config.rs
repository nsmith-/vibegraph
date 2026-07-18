//! Global configuration: resolves a proc-card `import model` directive to a
//! loaded [`UFOModel`].
//!
//! For the Standard Model this returns the interned [`sm_model`] (no filesystem
//! access); for other models it falls back to reading a UFO directory under
//! [`GlobalConfig::ufo_search_path`].
//!
//! TODO: future work will support more sophisticated model resolution, including handling
//! downloading and caching in a user local directory.

use std::path::PathBuf;
use std::sync::Arc;

use crate::diagrams::ModelImport;
use crate::runcard::{RunCard, RunCardError};
use crate::ufo::sm::{sm_model, SMRestrict};
use crate::ufo::{UFOModel, UfoError};

/// Coordinates model loading for the CLI: maps a parsed `import model` spec to a
/// concrete [`UFOModel`].
#[derive(Debug, Clone)]
pub struct GlobalConfig {
    /// Directory searched for non-SM UFO models (`<ufo_search_path>/<name>/`).
    pub ufo_search_path: PathBuf,
    /// Explicit restrict-card path, overriding a directive's `-<variant>` suffix
    /// and the default `restrict_default.dat` discovery (non-SM models only).
    pub restrict_path_override: Option<PathBuf>,
    /// Optional MadGraph `run_card.dat`; absent → MadGraph LO defaults.
    pub run_card_path: Option<PathBuf>,
}

impl GlobalConfig {
    /// Resolve an `import model` directive to a loaded model.
    ///
    /// - Absent directive → the interned SM default (`import model sm`).
    /// - `sm` (with optional `-<variant>` suffix) → the interned SM variant.
    /// - Any other model name → loaded from `ufo_search_path/<name>/`.
    pub fn load_ufo(&self, spec: &Option<ModelImport>) -> Result<Arc<UFOModel>, UfoError> {
        let Some(import) = spec else {
            return Ok(sm_model(SMRestrict::Default));
        };

        if import.name == "sm" {
            let restrict =
                SMRestrict::from_suffix(import.restrict_variant.as_deref()).ok_or_else(|| {
                    UfoError::Io {
                        file: format!("sm-{}", import.restrict_variant.as_deref().unwrap_or("")),
                        cause: std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            format!(
                                "unknown SM restrict variant '{}'",
                                import.restrict_variant.as_deref().unwrap_or("")
                            ),
                        ),
                    }
                })?;
            return Ok(sm_model(restrict));
        }

        // Non-SM model: read from the UFO search path.
        let dir = self.ufo_search_path.join(&import.name);
        let restrict = match &self.restrict_path_override {
            Some(path) => Some(path.clone()),
            None => import
                .restrict_variant
                .as_ref()
                .map(|v| dir.join(format!("restrict_{v}.dat"))),
        };
        UFOModel::load(&dir, restrict.as_deref())
    }

    /// Resolve the run card: parse [`run_card_path`](Self::run_card_path) if set,
    /// otherwise return the MadGraph LO defaults (an empty card).
    pub fn load_run_card(&self) -> Result<RunCard, RunCardError> {
        match &self.run_card_path {
            Some(path) => RunCard::parse_file(path),
            None => Ok(RunCard::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> GlobalConfig {
        GlobalConfig {
            ufo_search_path: PathBuf::from("/nonexistent"),
            restrict_path_override: None,
            run_card_path: None,
        }
    }

    #[test]
    fn none_and_sm_return_interned_default() {
        let cfg = config();
        let a = cfg.load_ufo(&None).unwrap();
        let b = cfg
            .load_ufo(&Some(ModelImport {
                name: "sm".into(),
                restrict_variant: None,
            }))
            .unwrap();
        // Same interned Arc for the default SM in both paths.
        assert!(Arc::ptr_eq(&a, &b));
        assert!(Arc::ptr_eq(&a, &sm_model(SMRestrict::Default)));
    }

    #[test]
    fn sm_variant_suffix_maps_to_interned() {
        let cfg = config();
        let m = cfg
            .load_ufo(&Some(ModelImport {
                name: "sm".into(),
                restrict_variant: Some("lepton_masses".into()),
            }))
            .unwrap();
        assert!(Arc::ptr_eq(&m, &sm_model(SMRestrict::LeptonMasses)));
    }

    #[test]
    fn load_run_card_defaults_when_absent() {
        let cfg = config();
        let rc = cfg.load_run_card().unwrap();
        // An absent card yields the MadGraph LO defaults.
        assert_eq!(rc.ebeam1, 6500.0);
        assert_eq!(rc.float("ptl"), 10.0);
    }

    #[test]
    fn unknown_sm_variant_errors() {
        let cfg = config();
        let r = cfg.load_ufo(&Some(ModelImport {
            name: "sm".into(),
            restrict_variant: Some("bogus".into()),
        }));
        assert!(r.is_err());
    }
}
