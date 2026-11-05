//! Global configuration: resolves a proc-card `import model` directive to a
//! loaded [`UFOModel`].
//!
//! For the Standard Model this returns the interned [`sm_model`] (no filesystem
//! access); for other models it falls back to reading a UFO directory under
//! [`GlobalConfig::ufo_search_path`]. Choosing that directory (`--ufo-dir`, the
//! environment, the `~/.vibegraph` cache) is the caller's job; a UFO model is
//! never downloaded.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

use thiserror::Error;
use tracing::{debug, info, info_span, trace};

use crate::diagrams::{
    generate_from_proc_card_in, DiagramError, DiagramSet, EnumerationPool, ModelImport,
    SupportedCard,
};
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
    /// Optional MadGraph `run_card.dat`; absent → MadGraph's default card for
    /// the process ([`load_run_card`](Self::load_run_card)).
    pub run_card_path: Option<PathBuf>,
}

impl GlobalConfig {
    /// Resolve an `import model` directive to a loaded model.
    ///
    /// - Absent directive → the interned SM default (`import model sm`).
    /// - `sm` (with optional `-<variant>` suffix) → the interned SM variant.
    /// - Any other model name → loaded from `ufo_search_path/<name>/`.
    #[cfg(test)]
    pub(crate) fn load_ufo(&self, spec: &Option<ModelImport>) -> Result<Arc<UFOModel>, UfoError> {
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

        // The SM's particle names are MadGraph's defaults, so `--modelname`
        // leaves the interned model as it is (`names::tests::the_sm_is_unchanged`).
        if import.name == "sm" {
            let restrict =
                SMRestrict::from_suffix(import.restrict_variant.as_deref()).ok_or_else(|| {
                    UfoError::UnknownSmRestrict {
                        variant: import.restrict_variant.clone().unwrap_or_default(),
                        known: SMRestrict::ALL
                            .iter()
                            .map(|v| v.suffix())
                            .collect::<Vec<_>>()
                            .join(", "),
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
        let (model, digest) =
            UFOModel::load_with_digest(&dir, restrict.as_deref(), import.particle_naming)?;
        let label = import.restrict_variant.as_deref().unwrap_or("default");
        let identity = ModelIdentity::from_loaded(&import.name, label, digest);
        Ok(describe(model, identity))
    }

    /// Resolve the run card for the processes of `parsed`: parse
    /// [`run_card_path`](Self::run_card_path) if set, and otherwise build
    /// MadGraph's default card for them, as its `output` writes it.
    ///
    /// For a `1 → n` decay that is [`RunCard::decay_default`] (every cut
    /// removed). For a scattering process it is [`RunCard::scattering_default`]
    /// over the incoming partons of the generated subprocesses, which MadGraph
    /// reads off its matrix elements and this reads off an enumeration of the
    /// card under `model` on `pool`: a subprocess with no diagram is not one
    /// MadGraph generates, so its partons do not count.
    pub fn load_run_card(
        &self,
        parsed: &SupportedCard,
        model: &UFOModel,
        pool: EnumerationPool,
    ) -> Result<RunCard, LoadRunCardError> {
        if let Some(path) = &self.run_card_path {
            return Ok(RunCard::parse_file(path)?);
        }
        if parsed.is_decay() {
            return Ok(RunCard::decay_default());
        }
        let sets = generate_from_proc_card_in(parsed, model, pool)?;
        Ok(RunCard::scattering_default(incoming_pdg_ids(&sets, model)))
    }
}

/// Why [`GlobalConfig::load_run_card`] has no card.
#[derive(Debug, Error)]
pub enum LoadRunCardError {
    #[error(transparent)]
    Card(#[from] RunCardError),
    /// The default card reads the generated subprocesses, and they could not be
    /// generated.
    #[error("cannot enumerate the processes the default card is built for: {0}")]
    Process(#[from] DiagramError),
}

/// The PDG ids of the incoming legs of every subprocess with a diagram.
fn incoming_pdg_ids(sets: &[DiagramSet], model: &UFOModel) -> BTreeSet<i64> {
    sets.iter()
        .filter(|set| !set.diagrams.is_empty())
        .flat_map(|set| &set.particles_in)
        .map(|name| {
            let id = model
                .particle_id(name)
                .expect("an enumerated leg is a model particle");
            model.particle(id).pdg_code
        })
        .collect()
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
                particle_naming: Default::default(),
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
                particle_naming: Default::default(),
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
                particle_naming: Default::default(),
            }))
            .unwrap();
        assert_eq!(default.label(), "sm-default");
        assert_eq!(no_b.label(), "sm-no_b_mass");
        assert_ne!(default.digest, no_b.digest);
    }

    /// The run card `cfg` resolves for `proc_card` under its own model.
    fn run_card_for(cfg: &GlobalConfig, proc_card: &str) -> RunCard {
        let parsed = crate::diagrams::parse_proc_card(proc_card, &Default::default()).unwrap();
        let model = cfg.load_ufo(&parsed.model).unwrap();
        cfg.load_run_card(&parsed, &model, EnumerationPool::Serial)
            .unwrap()
    }

    /// `(maxjetflavor, asrwgtflavor)` of a card.
    fn flavours(rc: &RunCard) -> (i64, i64) {
        (rc.maxjetflavor, rc.int("asrwgtflavor"))
    }

    #[test]
    fn load_run_card_defaults_when_absent() {
        let rc = run_card_for(&config(), "generate p p > e+ e-\n");
        // An absent card yields the MadGraph LO defaults.
        assert_eq!(rc.ebeam1, 6500.0);
        assert_eq!(rc.float("ptl"), 10.0);
    }

    /// With no run card, `maxjetflavor` is the heaviest incoming quark of a
    /// generated subprocess, at least 4, as MadGraph's
    /// `create_default_for_process` writes it: 5 once a massless b is in the
    /// proton, 4 in the four-flavour model. `asrwgtflavor` stays at the
    /// template's 5 in both, as in the card MadGraph writes and reads back.
    #[test]
    fn absent_card_takes_the_jet_flavours_from_the_beams() {
        let cfg = config();
        let five = run_card_for(&cfg, "import model sm-no_b_mass\ngenerate p p > j j\n");
        assert_eq!(flavours(&five), (5, 5));
        let four = run_card_for(&cfg, "import model sm\ngenerate p p > j j\n");
        assert_eq!(flavours(&four), (4, 5));
    }

    /// Only the subprocesses that have a diagram count. Under a diagonal CKM no
    /// b-initiated subprocess makes a single W+, so `p p > w+` stays at 4 in the
    /// five-flavour model though `p` holds the b: MadGraph reads the flavours
    /// off its matrix elements, not off the multiparticle label.
    #[test]
    fn absent_card_counts_only_generated_subprocesses() {
        let cfg = config();
        let w = run_card_for(&cfg, "import model sm-no_b_mass\ngenerate p p > w+\n");
        assert_eq!(flavours(&w), (4, 5));
        let bb = run_card_for(&cfg, "import model sm-no_b_mass\ngenerate b b~ > z\n");
        assert_eq!(flavours(&bb), (5, 5));
    }

    /// Incoming leptons are no partons, so the template's `maxjetflavor` stays;
    /// a decay keeps it too. So does a subprocess whose only incoming quarks
    /// are tops: MadGraph takes the heaviest beam quark only when some incoming
    /// parton is a quark up to b, a gluon or a photon.
    #[test]
    fn absent_card_without_incoming_partons_keeps_the_template_flavours() {
        let cfg = config();
        let ee = run_card_for(&cfg, "import model sm-no_b_mass\ngenerate e+ e- > b b~\n");
        assert_eq!(flavours(&ee), (4, 5));
        let decay = run_card_for(&cfg, "import model sm-no_b_mass\ngenerate z > b b~\n");
        assert_eq!(flavours(&decay), (4, 5));
        let tt = run_card_for(&cfg, "import model sm\ngenerate t t~ > z\n");
        assert_eq!(flavours(&tt), (4, 5));
    }

    /// A run card that is given wins whatever the process, its own 4 included,
    /// and a card silent on the two keeps the template's values.
    #[test]
    fn an_explicit_card_keeps_its_jet_flavours() {
        let dir = tempfile::tempdir().unwrap();
        let card = |name: &str, text: &str| {
            let path = dir.path().join(name);
            std::fs::write(&path, text).unwrap();
            GlobalConfig {
                run_card_path: Some(path),
                ..config()
            }
        };
        let five_flavour = "import model sm-no_b_mass\ngenerate p p > j j\n";
        let explicit = card("four.dat", "4 = maxjetflavor\n4 = asrwgtflavor\n");
        assert_eq!(flavours(&run_card_for(&explicit, five_flavour)), (4, 4));
        let silent = card("empty.dat", "");
        assert_eq!(flavours(&run_card_for(&silent, five_flavour)), (4, 5));
        let six = card("six.dat", "6 = maxjetflavor\n");
        let four_flavour = "import model sm\ngenerate p p > j j\n";
        assert_eq!(flavours(&run_card_for(&six, four_flavour)), (6, 5));
    }

    /// An unknown variant is refused as such, naming the variant asked for and
    /// every variant the built-in Standard Model has.
    #[test]
    fn unknown_sm_variant_names_the_known_ones() {
        let cfg = config();
        let err = cfg
            .load_ufo(&Some(ModelImport {
                name: "sm".into(),
                restrict_variant: Some("bogus".into()),
                particle_naming: Default::default(),
            }))
            .err()
            .expect("an unknown variant is refused");
        assert!(
            matches!(&err, UfoError::UnknownSmRestrict { variant, .. } if variant == "bogus"),
            "{err:?}"
        );
        let message = err.to_string();
        assert!(message.contains("'bogus'"), "{message}");
        for variant in SMRestrict::ALL {
            assert!(message.contains(variant.suffix()), "{message}");
        }
    }
}
