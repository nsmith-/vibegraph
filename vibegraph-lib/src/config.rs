//! Global configuration: resolves a proc-card `import model` directive to a
//! loaded [`UFOModel`].
//!
//! For the Standard Model this returns the interned [`sm_model`](crate::ufo::sm::sm_model) (no filesystem
//! access); for other models it falls back to reading a UFO directory under
//! [`GlobalConfig::ufo_search_path`]. Choosing that directory (`--ufo-dir`, the
//! environment, the `~/.vibegraph` cache) is the caller's job; a UFO model is
//! never downloaded.

use std::collections::{BTreeSet, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use thiserror::Error;
use tracing::{debug, info, info_span, trace};

use crate::diagrams::check::SupportedProcess;
use crate::diagrams::{
    generate_from_proc_card_in, DiagramError, DiagramSet, EnumerationPool, ModelImport,
    SupportedCard,
};
use crate::helas::color::colorize::colorize_process;
use crate::helas::color::tensor::ColorAlgebraError;
use crate::progress;
use crate::runcard::process_defaults::ProcessFacts;
use crate::runcard::{RunCard, RunCardError};
use crate::ufo::identity::ModelIdentity;
use crate::ufo::loaded::LoadedModel;
use crate::ufo::sm::SMRestrict;
use crate::ufo::{UFOModel, UfoError};

/// Report a resolved model and hand it straight back.
///
/// The digest rides on the headline because it is the only part of the report a
/// second run can compare: two models with the same label and different contents
/// differ nowhere else in what is said about them.
fn describe(loaded: LoadedModel) -> LoadedModel {
    let (model, identity) = (&loaded.model, &loaded.identity);
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
    loaded
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
    pub fn load_ufo_with_identity(
        &self,
        spec: &Option<ModelImport>,
    ) -> Result<(Arc<UFOModel>, ModelIdentity), UfoError> {
        self.load_model(spec)
            .map(|loaded| (loaded.model, loaded.identity))
    }

    /// Resolve an `import model` directive to a [`LoadedModel`]: the model, its
    /// identity, the restricted parsed model both come from, and its particle
    /// naming.
    ///
    /// Loading and identifying share this one resolution so the digest can never
    /// describe a different model than the one returned.
    pub fn load_model(&self, spec: &Option<ModelImport>) -> Result<LoadedModel, UfoError> {
        let _span = info_span!("ufo_load").entered();
        progress::step(progress::stage::UFO_LOAD, 0, Some(1));
        let Some(import) = spec else {
            return Ok(describe(LoadedModel::interned_sm(SMRestrict::Default)));
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
            return Ok(describe(LoadedModel::interned_sm(restrict)));
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
        let label = import.restrict_variant.as_deref().unwrap_or("default");
        let loaded = LoadedModel::from_dir(
            &dir,
            restrict.as_deref(),
            import.particle_naming,
            &import.name,
            label,
        )?;
        Ok(describe(loaded))
    }

    /// Resolve the run card for the processes of `parsed`: parse
    /// [`run_card_path`](Self::run_card_path) if set, and otherwise build
    /// MadGraph's default card for them, as its `output` writes it.
    ///
    /// For a `1 → n` decay that is [`RunCard::decay_default`] (every cut
    /// removed). For a scattering process it is [`RunCard::scattering_default`]
    /// over what MadGraph reads off its matrix elements, which this reads off
    /// an enumeration of each process line under `model` on `pool`
    /// ([`process_facts`]): a subprocess with no diagram is not one MadGraph
    /// generates, so it does not count.
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
        if let Some(particle) = massless_coloured_scalar(model) {
            return Err(LoadRunCardError::FixedScaleModel { particle });
        }
        let facts = process_facts(parsed, model, pool)?;
        Ok(RunCard::scattering_default(&facts)?)
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
    /// The default card reads the colour basis of every subprocess, and one
    /// could not be built.
    #[error("cannot build the colour basis the default card reads: {0}")]
    Colour(#[from] ColorAlgebraError),
    /// A model with a massless coloured scalar. MadGraph marks it `fix_scale`
    /// (`import_ufo.py:576-579`): its default card fixes both scales and turns
    /// matching off, and `launch` refuses any card that runs a scale
    /// (`common_run_interface.py:6563`). That card is not built here.
    #[error(
        "the model has the massless coloured scalar {particle}, for which MadGraph's default run \
         card fixes the renormalisation and factorisation scales and its launch refuses running \
         ones; that default is not built here, so pass a run card with fixed_ren_scale = T and \
         fixed_fac_scale = T"
    )]
    FixedScaleModel { particle: String },
}

/// A massless coloured scalar of `model`, MadGraph's `colored_scalar`
/// (`import_ufo.py:1330-1333`): spin 0, not a colour singlet, zero mass.
fn massless_coloured_scalar(model: &UFOModel) -> Option<String> {
    model
        .particles
        .values()
        .find(|p| p.spin == 1 && p.color != 1 && p.mass_param.eq_ignore_ascii_case("ZERO"))
        .map(|p| p.name.clone())
}

/// What MadGraph's `create_default_for_process` reads off the processes of
/// `parsed` ([`ProcessFacts`]), from each process line enumerated on its own:
/// MadGraph's matrix elements are those subprocesses, and a decay-chain line's
/// first process is its core, enumerated again without the decays.
fn process_facts(
    parsed: &SupportedCard,
    model: &UFOModel,
    pool: EnumerationPool,
) -> Result<ProcessFacts, LoadRunCardError> {
    let pdg = |name: &String| {
        let id = model
            .particle_id(name)
            .expect("an enumerated leg is a model particle");
        model.particle(id).pdg_code
    };
    let enumerate = |process: SupportedProcess| -> Result<Vec<DiagramSet>, DiagramError> {
        let card = SupportedCard {
            model: parsed.model.clone(),
            processes: vec![process],
        };
        Ok(generate_from_proc_card_in(&card, model, pool)?
            .into_iter()
            .filter(|set| !set.diagrams.is_empty())
            .collect())
    };

    let mut facts = ProcessFacts::default();
    let mut sets = Vec::new();
    for process in &parsed.processes {
        let line = enumerate(process.clone())?;
        let core = if process.decays.is_empty() {
            None
        } else {
            Some(enumerate(SupportedProcess {
                decays: Vec::new(),
                ..process.clone()
            })?)
        };
        for set in core.as_ref().unwrap_or(&line) {
            facts.core_legs.push(
                set.particles_in
                    .iter()
                    .chain(&set.particles_out)
                    .map(pdg)
                    .collect(),
            );
        }
        sets.extend(line);
    }
    for set in &sets {
        for (beam, name) in facts.beams.iter_mut().zip(&set.particles_in) {
            beam.insert(pdg(name));
        }
    }

    let mut coloured: Vec<i64> = model
        .particles
        .values()
        .filter(|p| p.color > 1 && p.spin > 0)
        .map(|p| p.pdg_code.abs())
        .collect();
    coloured.sort_unstable();
    coloured.dedup();
    facts.coloured_pdgs = coloured;

    facts.non_qcd_gluon_emission = uses_non_qcd_gluon_emission(&sets, model);
    facts.onshell_veto = parsed
        .processes
        .iter()
        .any(|p| !p.forbidden_onshell_s_channels.is_empty());
    let qcd_free = parsed
        .processes
        .iter()
        .all(|p| p.orders.iter().any(|o| o.name == "QCD" && o.value == 0));
    facts.qcd_free_single_color = qcd_free && single_color(&sets, model)?;
    Ok(facts)
}

/// Whether some subprocess uses a coupling of a vertex that emits a gluon among
/// coloured particles only, with no QCD coupling order: MadGraph's test for the
/// `MLM` limitation of a matrix element (`export_v4.py:4779-4791`), on the
/// vertices `import_ufo.py:1836-1842` counts.
fn uses_non_qcd_gluon_emission(sets: &[DiagramSet], model: &UFOModel) -> bool {
    let used: HashSet<_> = sets
        .iter()
        .flat_map(|set| &set.diagrams)
        .flat_map(|diagram| &diagram.vertices)
        .flat_map(|vertex| model.vertex_def(vertex.interaction).couplings.values())
        .copied()
        .collect();
    model.vertices.values().any(|vertex| {
        let particles: Vec<_> = vertex
            .particles
            .iter()
            .map(|&id| model.particle(id))
            .collect();
        particles.iter().any(|p| p.pdg_code == 21)
            && particles.iter().all(|p| p.color != 1)
            && vertex.couplings.values().any(|&coupling| {
                !model.coupling_def(coupling).orders.contains_key("QCD") && used.contains(&coupling)
            })
    })
}

/// Whether the colour basis of every subprocess is at a single power of `Nc`:
/// MadGraph's `single_color`, which `export_v4.py:1340` clears on any colour
/// flow below the basis's leading power.
fn single_color(sets: &[DiagramSet], model: &UFOModel) -> Result<bool, ColorAlgebraError> {
    for set in sets {
        let basis = colorize_process(model, &set.diagrams)?;
        let powers: BTreeSet<_> = basis
            .elements
            .iter()
            .flat_map(|e| &e.contributions)
            .map(|c| c.coeff.nc_power)
            .collect();
        if powers.len() > 1 {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use crate::runcard::ParamValue;
    use crate::ufo::sm::sm_model;

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

    /// The card MadGraph 3.7.1's `output` wrote for a process, committed under
    /// `tests/data/mg_default_run_cards`.
    fn madgraph_card(name: &str) -> RunCard {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/mg_default_run_cards")
            .join(format!("{name}.dat"));
        RunCard::parse_file(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    /// Every parameter on which two cards differ, as `(name, ours, theirs)`,
    /// both as resolved and as an event file records them. The PDF fields are
    /// left out: the written card names MadGraph's built-in `nn23lo1` because
    /// that MadGraph has no LHAPDF, and this crate's template names the LHAPDF
    /// copy of the same set, 247000.
    fn differences(ours: &RunCard, theirs: &RunCard) -> Vec<(String, String, String)> {
        const PDF: [&str; 4] = ["pdlabel", "pdlabel1", "pdlabel2", "lhaid"];
        let table = |rc: &RunCard| -> BTreeMap<String, String> {
            rc.iter()
                .map(|(k, v)| (k.to_string(), format!("{v:?}")))
                .chain(
                    rc.banner_values()
                        .map(|(k, v)| (format!("banner:{k}"), format!("{v:?}"))),
                )
                .collect()
        };
        let (a, b) = (table(ours), table(theirs));
        a.keys()
            .chain(b.keys())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter(|k| !PDF.contains(&k.trim_start_matches("banner:")))
            .filter(|k| a.get(*k) != b.get(*k))
            .map(|k| {
                let get = |t: &BTreeMap<String, String>| t.get(k).cloned().unwrap_or_default();
                (k.clone(), get(&a), get(&b))
            })
            .collect()
    }

    /// With no run card, `p p > e+ e-` plus `add process p p > e+ e- j` gets
    /// MadGraph's matched default: the second line differs from the first by a
    /// jet, so `create_default_for_process` turns MLM on. The whole card is
    /// compared with the one MadGraph wrote, so a rule that moved any other
    /// field would show here too; the matching fields are pinned by value as
    /// well, against the numbers that card holds. `ptj` resolves to `xqcut`
    /// under `auto_ptj_mjj`, and the event-file record keeps the card's 20.
    #[test]
    fn absent_card_on_mixed_jet_multiplicities_is_madgraphs_matched_card() {
        let rc = run_card_for(
            &config(),
            "import model sm\ngenerate p p > e+ e-\nadd process p p > e+ e- j\n",
        );
        let madgraph = madgraph_card("pp_ee_add_pp_ee_j");
        assert_eq!(differences(&rc, &madgraph), []);
        assert_eq!(rc.int("ickkw"), 1);
        assert_eq!(rc.float("xqcut"), 30.0);
        assert_eq!(rc.float("drjl"), 0.0);
        assert_eq!(rc.float("drjj"), 0.0);
        assert_eq!(rc.float("ptj"), 30.0);
        assert_eq!(rc.int("dynamical_scale_choice"), -1);
        assert_eq!(flavours(&rc), (4, 4));
        assert_eq!(
            rc.get("systematics_arguments").unwrap(),
            &ParamValue::Opaque(
                "['--mur=0.5,1,2', '--muf=0.5,1,2', '--pdf=errorset', '--alps=0.5,1,2']".into()
            )
        );
        assert_eq!(
            rc.get("sys_alpsfact").unwrap(),
            &ParamValue::Str("None".into()),
            "sys_alpsfact is hidden and not written, so MadEvent reads the template's"
        );
        let (_, settings) = rc.split().unwrap();
        let request = settings.systematics.request.expect("use_syst = T");
        assert_eq!(request.alps, [0.5, 1.0, 2.0]);
    }

    /// With no run card, `e+ e- > mu+ mu-` gets MadGraph's lepton beams:
    /// `lpp = 0`, 500 GeV each, systematics off. The whole card is compared with
    /// the one MadGraph wrote.
    #[test]
    fn absent_card_on_lepton_beams_is_madgraphs_lepton_card() {
        let rc = run_card_for(&config(), "import model sm\ngenerate e+ e- > mu+ mu-\n");
        assert_eq!(differences(&rc, &madgraph_card("ee_mumu")), []);
        assert_eq!((rc.lpp1, rc.lpp2), (0, 0));
        assert_eq!((rc.ebeam1, rc.ebeam2), (500.0, 500.0));
        assert_eq!(rc.get("use_syst"), Some(&ParamValue::Bool(false)));
        assert_eq!(rc.int("ickkw"), 0);
    }

    /// A model whose gluon vertices include ones without a QCD order (SMEFTsim's
    /// chromomagnetic operators) is MadGraph's `MLM` limitation once a
    /// subprocess uses one: the matching rule still fires, then `ickkw` goes
    /// back to 0 and the CKKW scale `-1` to 3, while `xqcut` and `--alps` stay.
    #[test]
    fn absent_card_under_the_mlm_limitation_is_madgraphs_card() {
        let cfg = GlobalConfig {
            ufo_search_path: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../validation/ufo"),
            ..config()
        };
        let rc = run_card_for(
            &cfg,
            "import model SMEFTsim_topU3l_MwScheme_UFO-massless\n\
             generate g g > t t~ NP<=1\nadd process g g > t t~ g NP<=1\n",
        );
        assert_eq!(
            differences(&rc, &madgraph_card("smeft_gg_tt_add_gg_tt_g")),
            []
        );
        assert_eq!(rc.int("ickkw"), 0);
        assert_eq!(rc.float("xqcut"), 30.0);
        assert_eq!(rc.int("dynamical_scale_choice"), 3);
    }

    /// The load error of an absent card for `proc_card`.
    fn default_card_error(proc_card: &str) -> String {
        let parsed = crate::diagrams::parse_proc_card(proc_card, &Default::default()).unwrap();
        let cfg = config();
        let model = cfg.load_ufo(&parsed.model).unwrap();
        cfg.load_run_card(&parsed, &model, EnumerationPool::Serial)
            .expect_err(proc_card)
            .to_string()
    }

    /// Beams MadGraph writes and this crate does not run are refused by the
    /// rule a hand-written card meets, on the values MadGraph's card holds
    /// (`lpp1 = 0, lpp2 = 1` for `e- p` and `ve p`, `-3, 3` for W beams, a
    /// polarized neutrino beam, `pdlabel1 = eva` beside a proton).
    #[test]
    fn absent_card_on_unsupported_beams_is_refused_by_name() {
        for (proc_card, expected) in [
            ("generate e- p > e- j\n", "lpp1=0, lpp2=1"),
            ("generate ve p > e- j\n", "lpp1=0, lpp2=1"),
            ("generate w+ w- > h\n", "lpp1=-3, lpp2=3"),
            ("generate ve ve~ > e+ e-\n", "polbeam1"),
            ("generate w+ d > u g\n", "eva"),
        ] {
            let message = default_card_error(&format!("import model sm\n{proc_card}"));
            assert!(message.contains(expected), "{proc_card}: {message}");
        }
    }

    /// `sde_strategy` as MadGraph's `output` wrote it for each card (local
    /// MadGraph 3.7.1): 2 when every line bounds QCD at 0 over a single colour
    /// flow, unless the final state is all leptons from partonic beams or a
    /// line has `$`; a line with no QCD bound, or one on QED only, stays at 1.
    #[test]
    fn absent_card_sde_strategy_follows_madgraph() {
        for (process, sde) in [
            ("e+ e- > mu+ mu- QCD=0", 2),
            ("e+ e- > u u~ QCD=0", 2),
            ("e+ e- > u u~ u u~ QCD=0", 2),
            ("u u~ > z QCD=0", 2),
            ("p p > e+ e- QCD=0", 1),
            ("e+ e- > u u~ $ z QCD=0", 1),
            ("e+ e- > u u~ QED=2", 1),
            ("e+ e- > mu+ mu-", 1),
            ("p p > e+ e-", 1),
        ] {
            let rc = run_card_for(&config(), &format!("import model sm\ngenerate {process}\n"));
            assert_eq!(rc.int("SDE_strategy"), sde, "{process}");
        }
    }
}
