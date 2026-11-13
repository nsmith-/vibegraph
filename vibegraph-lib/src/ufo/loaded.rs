//! A loaded model together with the restricted [`ParsedModel`] it was built from,
//! the names its particles answer to, and its [`ModelIdentity`].
//!
//! Every way of obtaining a model for a run ([`LoadedModel::from_dir`],
//! [`LoadedModel::interned_sm`], [`LoadedModel::from_restricted`]) ends in one
//! builder, [`build_restricted`], over one restricted `ParsedModel`, and the
//! identity's digest is [`model_digest`] of that same value. So the model, the
//! digest and the parsed model a caller holds can never describe different
//! physics, and an artifact that embeds the parsed model and its naming rebuilds
//! exactly the model the run used.

use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::identity::{model_digest, ModelIdentity};
use super::names::ParticleNaming;
use super::sm::{sm_model, sm_restricted_parsed, SMRestrict};
use super::{ParsedModel, UFOModel, UfoError};

/// What [`UFOModel::with_particle_naming`] reads: which names the particles
/// answer to, and the model's name for MadGraph's renaming (its directory's base
/// name, which selects the MSSM's `h1` rule).
///
/// It is not part of the model digest: a name cannot change the physics. It does
/// change which names a proc card may write, so it is recorded wherever a model
/// is rebuilt from its parsed form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelNaming {
    pub naming: ParticleNaming,
    pub model_name: String,
}

impl ModelNaming {
    /// The interned SM's naming: MadGraph's defaults under the name `sm`.
    pub fn interned_sm() -> Self {
        ModelNaming {
            naming: ParticleNaming::MadGraphDefaults,
            model_name: "sm".to_string(),
        }
    }
}

/// A model ready for a run, and everything needed to bank and rebuild it.
#[derive(Clone)]
pub struct LoadedModel {
    pub model: Arc<UFOModel>,
    pub identity: ModelIdentity,
    /// The model with its restriction applied: the value [`identity`](Self::identity)'s
    /// digest is computed over and [`model`](Self::model) is built from.
    pub parsed: Arc<ParsedModel>,
    pub naming: ModelNaming,
}

/// Build the model a restricted [`ParsedModel`] describes, its particles named
/// per `naming`. The restriction is already baked in, so this only builds the
/// topology and the name table.
pub(crate) fn build_restricted(
    parsed: &ParsedModel,
    naming: &ModelNaming,
) -> Result<UFOModel, UfoError> {
    parsed
        .clone()
        .into_model(None)?
        .with_particle_naming(naming.naming, &naming.model_name)
}

impl LoadedModel {
    /// The model a restricted `parsed` describes, identified as
    /// `import model <name>-<restrict>` with the digest of `parsed` itself.
    pub fn from_restricted(
        parsed: ParsedModel,
        naming: ModelNaming,
        name: &str,
        restrict: &str,
    ) -> Result<Self, UfoError> {
        let digest = model_digest(&parsed);
        let model = build_restricted(&parsed, &naming)?;
        Ok(LoadedModel {
            model: Arc::new(model),
            identity: ModelIdentity::from_loaded(name, restrict, digest),
            parsed: Arc::new(parsed),
            naming,
        })
    }

    /// The model of the UFO directory `path` under `restrict_card` (see
    /// [`ParsedModel::load_restricted`] for the card's resolution), its particles
    /// named per `naming` with the directory's base name as the model's name.
    pub fn from_dir(
        path: &Path,
        restrict_card: Option<&Path>,
        naming: ParticleNaming,
        name: &str,
        restrict: &str,
    ) -> Result<Self, UfoError> {
        let parsed = ParsedModel::load_restricted(path, restrict_card)?;
        let model_name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        Self::from_restricted(parsed, ModelNaming { naming, model_name }, name, restrict)
    }

    /// An interned SM variant: the cached [`sm_model`], the cached restricted
    /// parsed model it was built from, and [`ModelIdentity::interned_sm`].
    pub fn interned_sm(restrict: SMRestrict) -> Self {
        LoadedModel {
            model: sm_model(restrict),
            identity: ModelIdentity::interned_sm(restrict),
            parsed: sm_restricted_parsed(restrict),
            naming: ModelNaming::interned_sm(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The interned SM's identity digest is the digest of the parsed model it
    /// carries, and rebuilding from that parsed model and naming gives a model
    /// with the same particles, vertices and couplings in the same order as the
    /// cached one. Cannot detect a difference confined to fields of `UFOModel`
    /// not compared here (parameter values, topology internals).
    #[test]
    fn interned_sm_parts_agree() {
        for variant in SMRestrict::ALL {
            let loaded = LoadedModel::interned_sm(variant);
            assert_eq!(loaded.identity.digest, model_digest(&loaded.parsed));
            let rebuilt = build_restricted(&loaded.parsed, &loaded.naming).unwrap();
            assert!(rebuilt.particles.keys().eq(loaded.model.particles.keys()));
            assert!(rebuilt.vertices.keys().eq(loaded.model.vertices.keys()));
            assert!(rebuilt.couplings.keys().eq(loaded.model.couplings.keys()));
            assert_eq!(rebuilt.particle_names(), loaded.model.particle_names());
        }
    }
}
