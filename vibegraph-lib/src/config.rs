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

use tracing::{debug, info, info_span, trace};

use crate::diagrams::ModelImport;
use crate::progress;
use crate::runcard::{RunCard, RunCardError};
use crate::ufo::identity::ModelIdentity;
use crate::ufo::sm::{sm_model, SMRestrict};
use crate::ufo::{UFOModel, UfoError};

/// Report a resolved model and hand it straight back.
///
/// The digest rides on the headline because it is the only part of the report a
/// second run can compare: two models with the same label and different contents
/// differ nowhere else in what is said about them.
fn describe(model: Arc<UFOModel>, identity: ModelIdentity) -> (Arc<UFOModel>, ModelIdentity) {
    info!("model {} ({})", identity.label(), identity.digest);
    debug!(
        particles = model.particles.len(),
        vertices = model.vertices.len(),
        couplings = model.couplings.len(),
        parameters = model.params.externals.len() + model.params.internals.len(),
        lorentz = model.lorentz.len(),
        "model contents"
    );
    let orders: Vec<String> = model
        .order_hierarchy
        .iter()
        .map(|(name, hierarchy)| format!("{name}={hierarchy}"))
        .collect();
    debug!("coupling-order hierarchy: {}", orders.join(" "));
    if tracing::enabled!(tracing::Level::TRACE) {
        for vertex in model.vertices.values() {
            let structures: Vec<&str> = vertex
                .lorentz
                .iter()
                .map(|&id| model.lorentz_struct(id).structure.as_str())
                .collect();
            trace!("vertex {}: {}", vertex.name, structures.join(" | "));
        }
    }
    progress::step(progress::stage::UFO_LOAD, 1, Some(1));
    (model, identity)
}

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
        self.load_ufo_with_identity(spec).map(|(model, _)| model)
    }

    /// Resolve an `import model` directive to a loaded model together with the
    /// [`ModelIdentity`] of the assets it was built from.
    ///
    /// Loading and identifying share this one resolution so the digest can never
    /// describe a different model than the one returned.
    pub fn load_ufo_with_identity(
        &self,
        spec: &Option<ModelImport>,
    ) -> Result<(Arc<UFOModel>, ModelIdentity), UfoError> {
        let _span = info_span!("ufo_load").entered();
        progress::step(progress::stage::UFO_LOAD, 0, Some(1));
        let Some(import) = spec else {
            return Ok(describe(
                sm_model(SMRestrict::Default),
                ModelIdentity::interned_sm(SMRestrict::Default),
            ));
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
            return Ok(describe(
                sm_model(restrict),
                ModelIdentity::interned_sm(restrict),
            ));
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
        let (model, digest) = UFOModel::load_with_digest(&dir, restrict.as_deref())?;
        let label = import.restrict_variant.as_deref().unwrap_or("default");
        let identity = ModelIdentity::from_loaded(&import.name, label, digest);
        Ok(describe(model, identity))
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

    /// The identity handed back describes the variant that was actually resolved,
    /// down to the restrict card's bytes — the property a later run's refusal
    /// rests on. It cannot detect a digest that is wrong in the *same* way on both
    /// sides of a comparison; `ufo::identity` pins the digest's inputs instead.
    #[test]
    fn identity_follows_the_resolved_variant() {
        let cfg = config();
        let (_, default) = cfg.load_ufo_with_identity(&None).unwrap();
        let (_, no_b) = cfg
            .load_ufo_with_identity(&Some(ModelImport {
                name: "sm".into(),
                restrict_variant: Some("no_b_mass".into()),
            }))
            .unwrap();
        assert_eq!(default.label(), "sm-default");
        assert_eq!(no_b.label(), "sm-no_b_mass");
        assert_ne!(default.digest, no_b.digest);
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
