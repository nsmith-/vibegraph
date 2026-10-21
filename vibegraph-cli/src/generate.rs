//! `vibegraph generate` — turn a converged `vibegraph integrate` run into an
//! unweighted event sample, serialised as a Les Houches Event File.
//!
//! The grid artifact is the run: it carries the trained per-channel VEGAS grids,
//! the channel selection weights they were trained under, the integration's run
//! card, the process string and the identity of the model behind it. What it
//! does **not** carry is the compiled amplitude, so the proc card has to be
//! supplied again — and is then checked against the artifact rather than
//! trusted. A grid trained on one process, or on the same process in another
//! model, and replayed against another samples a perfectly plausible-looking
//! wrong distribution, so the mismatch is refused instead of being taken as new
//! input.
//!
//! The sampling itself is [`vibegraph::generate::generate`]; this command
//! gathers its inputs: the artifact, the model and PDF assets, the generation
//! settings the artifact recorded with `--nevents` applied, and the options of
//! how many events to write, where, and under which weight strategy.

use std::path::PathBuf;

use clap::{Args, ValueEnum};
use vibegraph::artifact::IntegrateArtifact;
use vibegraph::config::GlobalConfig;
use vibegraph::diagrams::{ParsingOptions, SupportedCard};
use vibegraph::generate::{
    generate, GenerateAssets, GenerateOptions, GenerateSettings, Strategy as SampleStrategy,
};
use vibegraph::reweight::card::ReweightCard;
use vibegraph::reweight::engine::ReweightOptions;
use vibegraph::reweight::{resolve, resolve_couplings, Launch};
use vibegraph::runcard::{BeamMode, RunCard};
use vibegraph::ufo::identity::ModelIdentity;
use vibegraph::ufo::{EvaluatedModel, UFOModel};
use vibegraph::unweight::{MaxRule, ScanBudget, DEFAULT_EXCESS_SHARE};

use crate::error::{err, CliError};
use crate::integrate::{
    forbidden_onshell, load_pdf_set, load_run_card, process_string, NO_PDF, PDF_MEMBER,
};
use crate::network::NetworkPolicy;
use crate::parallel::ParallelArgs;
use crate::tui;
use vibegraph::cache::pinned::DEFAULT_PDF_SET;

/// Default output file.
const DEFAULT_OUTPUT: &str = "events.lhe";

/// Default `--scan-points`. Measured on the 24-channel `p p > l+ l- j` grids, a
/// flat per-channel budget and the integration's own allocation sit on the same
/// trade-off curve between the summed maximum (which sets acceptance) and the
/// cross-section share above the maxima, so the allocation that costs nothing to
/// compute is the default. [`ScanBudget`] carries what the curve is.
const DEFAULT_SCAN_POINTS: &str = "share";

/// How the accept/reject weights become the file's events.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum Strategy {
    /// Hold the sample, write the weights (`IDWTUP = -4`, or `-3` under the run card's
    /// `event_norm = sum` or `unity`).
    Buffer,
    /// Stream, writing each event `floor(w) + Bernoulli(frac(w))` times at unit
    /// weight (`IDWTUP = +3`).
    StochasticRounding,
}

impl From<Strategy> for SampleStrategy {
    fn from(strategy: Strategy) -> Self {
        match strategy {
            Strategy::Buffer => SampleStrategy::Buffer,
            Strategy::StochasticRounding => SampleStrategy::StochasticRounding,
        }
    }
}

#[derive(Args, Debug)]
pub(crate) struct GenerateArgs {
    /// Grid artifact from a completed `vibegraph integrate` run.
    pub(crate) artifact: PathBuf,

    /// The same process card the artifact was integrated from; `-` reads the
    /// card from stdin.
    pub(crate) proc_card: PathBuf,

    /// The same run card the artifact was integrated with; absent → MadGraph LO
    /// defaults. A card that differs from the banked one is refused.
    #[arg(long)]
    pub(crate) run_card: Option<PathBuf>,

    /// Directory containing the proc card's UFO model directory; defaults to
    /// `$VIBEGRAPH_UFO_DIR`, then the `~/.vibegraph` cache, then the current
    /// directory. Unused for the built-in Standard Model.
    #[arg(long)]
    pub(crate) ufo_dir: Option<PathBuf>,

    /// Events to write; defaults to the run card's `nevents`.
    #[arg(long)]
    pub(crate) nevents: Option<usize>,

    /// Output Les Houches file.
    #[arg(short, long, default_value = DEFAULT_OUTPUT)]
    pub(crate) out: PathBuf,

    /// Overwrite an existing event file.
    #[arg(long)]
    pub(crate) force: bool,

    /// Weight strategy.
    #[arg(long, value_enum, default_value_t = Strategy::Buffer)]
    pub(crate) strategy: Strategy,

    /// RNG seed for the generation. Same seed, same sample.
    #[arg(long, default_value_t = 20_260_728)]
    pub(crate) seed: u64,

    /// LHAPDF set name (proton beams only). A set other than the one the artifact
    /// was integrated with is refused.
    #[arg(long, default_value = DEFAULT_PDF_SET)]
    pub(crate) pdf_set: String,

    /// Directory containing `<pdf-set>/`; defaults to `$VIBEGRAPH_PDF_DIR`, then
    /// the `~/.vibegraph` cache (offering to download the set if absent), then
    /// `validation/pdf` under the current directory.
    #[arg(long)]
    pub(crate) pdf_dir: Option<PathBuf>,

    /// Points the frozen `w_max` scan spends on *each* channel, or `share` to give
    /// every channel the integration's own per-channel budget instead. What the
    /// budget buys depends on `--max-truncation`: at zero a larger budget raises
    /// the maxima, trading acceptance for fewer events above them; under a
    /// truncation it sharpens the quantile the maxima converge to.
    #[arg(
        long,
        value_name = "N|share",
        default_value = DEFAULT_SCAN_POINTS,
        value_parser = parse_scan_budget
    )]
    pub(crate) scan_points: ScanBudget,

    /// Share of each channel's scanned cross section its maximum may leave above
    /// itself. `0` takes the largest weight the scan saw instead; a larger share
    /// lowers the maxima, accepting more trials and giving more of the accepted
    /// events a weight above one.
    #[arg(
        long,
        value_name = "SHARE",
        default_value_t = DEFAULT_EXCESS_SHARE,
        value_parser = parse_excess_share
    )]
    pub(crate) max_truncation: f64,

    /// MadGraph reweight card: every event also carries its weight under each
    /// `launch` block's parameters, as an LHEF `<rwgt>` block.
    #[arg(long)]
    pub(crate) reweight_card: Option<PathBuf>,

    /// Evaluate every reweighting hypothesis directly, never through a
    /// polynomial in the couplings. Slower; for cross-checking.
    #[arg(
        long,
        requires = "reweight_card",
        conflicts_with = "reweight_couplings"
    )]
    pub(crate) reweight_exact: bool,

    /// Track the amplitude as a polynomial jointly in these external parameters
    /// (comma-separated names), so every hypothesis costs a quadratic form rather
    /// than an amplitude evaluation: an event costs one evaluation per monomial of
    /// the amplitude — `1 + n` for `n` couplings entering at most once per
    /// diagram — whatever the number of hypotheses. The card may then move only
    /// these, and each must enter every subprocess polynomially.
    #[arg(
        long,
        value_delimiter = ',',
        value_name = "NAME,...",
        requires = "reweight_card"
    )]
    pub(crate) reweight_couplings: Option<Vec<String>>,

    #[command(flatten)]
    pub(crate) parallel: ParallelArgs,
}

/// `--scan-points` accepts a per-channel count or the word that ties the scan to
/// the integration's own per-channel allocation.
fn parse_scan_budget(raw: &str) -> Result<ScanBudget, String> {
    if raw == "share" {
        return Ok(ScanBudget::IntegrationShare);
    }
    match raw.parse::<usize>() {
        Ok(0) => Err("--scan-points needs at least one point per channel".to_string()),
        Ok(n) => Ok(ScanBudget::PerChannel(n)),
        Err(_) => Err(format!("expected a point count or `share`, got `{raw}`")),
    }
}

/// `--max-truncation` is a share of a channel's own scanned cross section, so a
/// value of one or more would ask for every scanned point to sit above the
/// maximum and is a refusal rather than a clamp.
fn parse_excess_share(raw: &str) -> Result<f64, String> {
    let share: f64 = raw
        .parse()
        .map_err(|_| format!("expected a share in [0, 1), got `{raw}`"))?;
    match MaxRule::truncated(share) {
        Some(_) => Ok(share),
        None => Err(format!("expected a share in [0, 1), got `{raw}`")),
    }
}

/// The rule the run reads its maxima off. The share is validated by the parser,
/// so a run that reached here has one.
fn max_rule(args: &GenerateArgs) -> MaxRule {
    MaxRule::truncated(args.max_truncation).expect("--max-truncation is a share below one")
}

/// One way the cards handed to a generation run differ from the ones that trained
/// the grid.
#[derive(Debug, PartialEq)]
pub(crate) struct CardMismatch {
    pub(crate) what: String,
    pub(crate) banked: String,
    pub(crate) given: String,
}

/// Every difference between the inputs this run was given and the ones banked in
/// the artifact: the model, the process, and every run-card parameter.
///
/// The comparison is exact and covers every run-card parameter, including the ones
/// no physics reads: the card is a fingerprint of the run, an unexplained
/// difference in it means the artifact and the cards came from different places,
/// and deciding case by case which differences are harmless is how a mismatched
/// grid gets sampled anyway. Float parameters are compared for equality rather
/// than within a tolerance for the same reason — both sides are the same parser's
/// output, so any difference is a real one.
///
/// The model is compared twice over. Its `import model` label catches the case a
/// reader would recognise (`sm` versus `sm-no_b_mass` under an identical `generate`
/// line, which the process string alone spells the same way); its digest catches
/// the case no label can see, a model whose assets changed underneath an unchanged
/// name. A differing label already implies a differing digest, so only the label is
/// reported then — the digest check is what has teeth when the labels agree.
pub(crate) fn card_mismatches(
    artifact: &IntegrateArtifact,
    model: &ModelIdentity,
    process: &str,
    run_card: &RunCard,
) -> Vec<CardMismatch> {
    let mut out = Vec::new();
    if artifact.model.label() != model.label() {
        out.push(CardMismatch {
            what: "model".to_string(),
            banked: artifact.model.label(),
            given: model.label(),
        });
    } else if artifact.model.digest != model.digest {
        out.push(CardMismatch {
            what: format!("model `{}` contents", model.label()),
            banked: artifact.model.digest.clone(),
            given: model.digest.clone(),
        });
    }
    if artifact.process != process {
        out.push(CardMismatch {
            what: "process".to_string(),
            banked: artifact.process.clone(),
            given: process.to_string(),
        });
    }
    let banked: std::collections::BTreeMap<&str, String> = artifact
        .run_card
        .iter()
        .map(|(k, v)| (k, format!("{v:?}")))
        .collect();
    let given: std::collections::BTreeMap<&str, String> = run_card
        .iter()
        .map(|(k, v)| (k, format!("{v:?}")))
        .collect();
    for (name, value) in &banked {
        let theirs = given.get(name);
        if theirs != Some(value) {
            out.push(CardMismatch {
                what: format!("run card `{name}`"),
                banked: value.clone(),
                given: theirs.cloned().unwrap_or_else(|| "<absent>".to_string()),
            });
        }
    }
    for (name, value) in &given {
        if !banked.contains_key(name) {
            out.push(CardMismatch {
                what: format!("run card `{name}`"),
                banked: "<absent>".to_string(),
                given: value.clone(),
            });
        }
    }
    out
}

/// The differences between the parton distributions this run would read and the
/// ones the artifact was integrated with.
///
/// The run card pins the LHAPDF *id*, but the set a run actually loads is named
/// by a flag, so an artifact and a command line can agree on every card and still
/// disagree about which tabulation the luminosities came from. Nothing downstream
/// notices: the grids replay, the events come out, and every weight is taken
/// against a different parton distribution than the one that trained them.
pub(crate) fn pdf_mismatches(
    artifact: &IntegrateArtifact,
    pdf_set: &str,
    pdf_member: u32,
) -> Vec<CardMismatch> {
    let mut out = Vec::new();
    if artifact.pdf_set != pdf_set {
        out.push(CardMismatch {
            what: "PDF set".to_string(),
            banked: artifact.pdf_set.clone(),
            given: pdf_set.to_string(),
        });
    }
    if artifact.pdf_member != pdf_member {
        out.push(CardMismatch {
            what: "PDF member".to_string(),
            banked: artifact.pdf_member.to_string(),
            given: pdf_member.to_string(),
        });
    }
    out
}

fn refuse_on_mismatch(mismatches: &[CardMismatch]) -> Result<(), CliError> {
    if mismatches.is_empty() {
        return Ok(());
    }
    let mut msg = String::from(
        "the inputs do not match the ones the grid was trained on; \
         re-integrate, or pass the cards that produced this artifact\n",
    );
    for m in mismatches {
        msg.push_str(&format!(
            "  {}: artifact has {}, this run has {}\n",
            m.what, m.banked, m.given
        ));
    }
    Err(err(msg))
}

/// Every difference between the generation settings of the card handed to this
/// run and the ones the artifact recorded, each named by its run-card field.
pub(crate) fn settings_mismatches(
    artifact: &IntegrateArtifact,
    given: &GenerateSettings,
) -> Vec<CardMismatch> {
    fn fields(settings: &GenerateSettings) -> Vec<(String, String)> {
        let mut out = vec![
            ("nevents".to_string(), settings.nevents.to_string()),
            (
                "event_norm".to_string(),
                settings.event_norm.as_str().to_string(),
            ),
            (
                "use_syst".to_string(),
                settings.systematics.enabled.to_string(),
            ),
            ("clusinfo".to_string(), settings.clustering_info.to_string()),
        ];
        out.extend(settings.systematics.fields.iter().cloned());
        out
    }
    let given = fields(given);
    fields(&artifact.generate)
        .into_iter()
        .filter_map(|(name, banked)| {
            let theirs = given
                .iter()
                .find(|(n, _)| *n == name)
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "<absent>".to_string());
            (theirs != banked).then(|| CardMismatch {
                what: format!("run card `{name}`"),
                banked,
                given: theirs,
            })
        })
        .collect()
}

/// The reweight card's hypotheses and how to evaluate them, resolved against the
/// model, or `None` when the run has no card. Everything the card cannot honour
/// is refused here, before any subprocess is compiled.
fn reweight_launches(
    args: &GenerateArgs,
    parsed: &SupportedCard,
    model: &UFOModel,
) -> Result<Option<(Vec<Launch>, ReweightOptions)>, CliError> {
    let Some(path) = &args.reweight_card else {
        return Ok(None);
    };
    let text = std::fs::read_to_string(path)
        .map_err(|e| err(format!("cannot read {}: {e}", path.display())))?;
    let card: ReweightCard = text
        .parse()
        .map_err(|e| err(format!("{}: {e}", path.display())))?;
    let forbidden = forbidden_onshell(parsed, model)?;
    if !forbidden.is_empty() {
        return Err(err(format!(
            "{}: {}",
            path.display(),
            vibegraph::reweight::ReweightError::ForbiddenSChannel { ids: forbidden }
        )));
    }
    let launches = resolve(&card, model).map_err(|e| err(format!("{}: {e}", path.display())))?;
    let couplings = args
        .reweight_couplings
        .as_deref()
        .map(|names| resolve_couplings(model, names))
        .transpose()
        .map_err(|e| err(format!("--reweight-couplings: {e}")))?;
    let options = ReweightOptions {
        exact: args.reweight_exact,
        couplings,
    };
    Ok(Some((launches, options)))
}

pub(crate) fn run(args: &GenerateArgs, network: NetworkPolicy) -> Result<(), CliError> {
    args.parallel.install().map_err(err)?;
    if !args.force && args.out.exists() {
        return Err(err(format!(
            "{} already exists (pass --force to overwrite)",
            args.out.display()
        )));
    }

    let opts = ParsingOptions::default();
    let parsed = crate::read_proc_card(&args.proc_card, &opts)?;
    let process = process_string(&parsed)?;

    let config = GlobalConfig {
        ufo_search_path: crate::assets::resolve_ufo_search_path(
            parsed.model.as_ref(),
            args.ufo_dir.as_deref(),
        )
        .map_err(err)?,
        restrict_path_override: None,
        run_card_path: args.run_card.clone(),
    };
    let (model, model_id) = config
        .load_ufo_with_identity(&parsed.model)
        .map_err(|e| err(format!("failed to load model: {e}")))?;
    // The run card is checked before the artifact is opened, as `integrate` checks
    // it before enumerating: a card the generator must refuse is the first error
    // whatever is wrong with the artifact.
    let (rc, card_settings) = load_run_card(&config, &parsed)?.split();
    let artifact = IntegrateArtifact::read_from_path(&args.artifact)
        .map_err(|e| err(format!("cannot read {}: {e}", args.artifact.display())))?;

    tui::state::describe_model(
        &model_id.label(),
        &model_id.digest,
        model.particles.len(),
        model.vertices.len(),
        model.couplings.len(),
    );
    tui::state::describe_process(&process);
    tui::state::note_channels(artifact.channels.len());

    let hadronic = !parsed.is_decay() && rc.beam_mode() == BeamMode::Proton;
    let mut mismatches = card_mismatches(&artifact, &model_id, &process, &rc);
    mismatches.extend(settings_mismatches(&artifact, &card_settings));
    // A fixed-energy run reads no parton distributions, and the artifact says so;
    // the flag is only meaningful on the hadronic path.
    let pdf_set = if hadronic { &args.pdf_set } else { NO_PDF };
    mismatches.extend(pdf_mismatches(&artifact, pdf_set, PDF_MEMBER));
    refuse_on_mismatch(&mismatches)?;

    let evaluated = EvaluatedModel::from_model(model.clone());
    let reweight = reweight_launches(args, &parsed, &model)?;
    let mut settings = artifact.generate.clone();
    if let Some(nevents) = args.nevents {
        settings.nevents = nevents as u64;
    }
    if settings.nevents == 0 {
        return Err(err(match args.nevents {
            Some(_) => "no events requested: --nevents is 0".to_string(),
            None => format!(
                "no events requested: --nevents is absent and the run card recorded in {} \
                 asks for nevents = 0",
                args.artifact.display(),
            ),
        }));
    }

    let set = if hadronic {
        Some(load_pdf_set(&args.pdf_set, args.pdf_dir.as_ref(), network)?)
    } else {
        None
    };
    let member = set
        .as_ref()
        .map(|set| {
            set.member(PDF_MEMBER)
                .map_err(|e| err(format!("cannot load PDF member {PDF_MEMBER}: {e}")))
        })
        .transpose()?;
    let assets = GenerateAssets {
        card: &parsed,
        model: &model,
        evaluated: &evaluated,
        pdf: set.as_ref().zip(member.as_ref()),
    };
    let options = GenerateOptions {
        seed: args.seed,
        strategy: args.strategy.into(),
        scan_points: args.scan_points,
        max_rule: max_rule(args),
        reweight,
        enumeration: args.parallel.enumeration(),
        out: args.out.clone(),
        artifact_label: args.artifact.display().to_string(),
        on_scan: Some(&tui::state::note_unweighting_efficiency),
    };
    generate(&artifact, &assets, &settings, &options).map_err(|e| err(e.to_string()))?;
    // The command's result: the path a caller pipes this to learn, at any
    // verbosity. Everything else the run had to say went to the notice stream.
    tui::result_line(format_args!("wrote {}", args.out.display()));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use vibegraph::artifact::{ChannelGrid, ChannelKey, FORMAT_VERSION};
    use vibegraph::phasespace::maps::MapChoices;
    use vibegraph::ufo::sm::SMRestrict;
    use vibegraph::vegas::{GridRule, VegasGrid};

    fn sm() -> ModelIdentity {
        ModelIdentity::interned_sm(SMRestrict::Default)
    }

    fn artifact(process: &str, run_card: RunCard) -> IntegrateArtifact {
        IntegrateArtifact {
            format_version: FORMAT_VERSION,
            process: process.to_string(),
            proc_card: format!("import model sm\ngenerate {process}\n"),
            model: sm(),
            pdf_set: "none".to_string(),
            pdf_member: 0,
            mu_f: 0.0,
            sqrt_s_had: 91.2,
            neval: 1000,
            niter: 2,
            seed: 1,
            run_card,
            generate: RunCard::default().split().1,
            channels: vec![ChannelGrid {
                key: ChannelKey::Whole,
                alpha: 1.0,
                neval: 1000,
                grid: VegasGrid::new(2, 16, 1.5),
                sigma_pb: 1.0,
                sigma_err_pb: 0.01,
                chi2_per_dof: 1.0,
                sampler: None,
            }],
            maps: MapChoices::LEGACY,
            grid_rule: Some(GridRule::UNBOUNDED),
            sigma_pb: 1.0,
            sigma_err_pb: 0.01,
            chi2_per_dof: 1.0,
        }
    }

    fn card(text: &str) -> RunCard {
        RunCard::parse(text).expect("run card").split().0
    }

    const BASE_CARD: &str = "  0 = lpp1\n  0 = lpp2\n  45.6 = ebeam1\n  45.6 = ebeam2\n";

    /// The refusal is the point, so it is the refusal that is tested: a matching
    /// pair passing proves nothing on its own, since a check that always passes
    /// also passes that.
    #[test]
    fn a_changed_run_card_parameter_is_refused() {
        let banked = artifact("e+ e- > mu+ mu-", card(BASE_CARD));
        // The matching case first, so a check that refuses everything is not
        // mistaken for one that works.
        assert!(card_mismatches(&banked, &sm(), "e+ e- > mu+ mu-", &card(BASE_CARD)).is_empty());

        let moved = card("  0 = lpp1\n  0 = lpp2\n  100.0 = ebeam1\n  45.6 = ebeam2\n");
        let found = card_mismatches(&banked, &sm(), "e+ e- > mu+ mu-", &moved);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].what, "run card `ebeam1`");
        assert!(refuse_on_mismatch(&found).is_err());
    }

    /// The gap this check exists for: `import model sm-no_b_mass` writes the same
    /// `generate` line and the same run card, so nothing else in the comparison
    /// moves — only the model does.
    #[test]
    fn a_different_restrict_variant_is_refused() {
        let banked = artifact("e+ e- > mu+ mu-", card(BASE_CARD));
        let other = ModelIdentity::interned_sm(SMRestrict::NoBMass);
        let found = card_mismatches(&banked, &other, "e+ e- > mu+ mu-", &card(BASE_CARD));
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].what, "model");
        assert_eq!(found[0].banked, "sm-default");
        assert_eq!(found[0].given, "sm-no_b_mass");
        assert!(refuse_on_mismatch(&found).is_err());
    }

    /// The case the name provably cannot see: same model, same restrict-card name,
    /// different card contents. Only the digest separates these, so this is the
    /// test that fails if the digest comparison is dropped.
    #[test]
    fn a_restrict_card_that_changed_under_its_name_is_refused() {
        let banked = artifact("e+ e- > mu+ mu-", card(BASE_CARD));
        let mut altered = sm();
        assert_eq!(altered.label(), banked.model.label());
        altered.digest = vibegraph::ufo::identity::digest_bytes(b"a different restrict card");

        let found = card_mismatches(&banked, &altered, "e+ e- > mu+ mu-", &card(BASE_CARD));
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].what, "model `sm-default` contents");
        assert_ne!(found[0].banked, found[0].given);
        assert!(refuse_on_mismatch(&found).is_err());
    }

    /// A cut threshold moves no beam and no scale, but it changes which points the
    /// grid was trained on — so it has to be refused just as loudly.
    #[test]
    fn a_changed_cut_is_refused_too() {
        let banked = artifact("e+ e- > mu+ mu-", card(BASE_CARD));
        let recut = card(&format!("{BASE_CARD}  25.0 = ptl\n"));
        let found = card_mismatches(&banked, &sm(), "e+ e- > mu+ mu-", &recut);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].what, "run card `ptl`");
    }

    #[test]
    fn a_different_process_is_refused() {
        let banked = artifact("e+ e- > mu+ mu-", card(BASE_CARD));
        let found = card_mismatches(&banked, &sm(), "e+ e- > ta+ ta-", &card(BASE_CARD));
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].what, "process");
        assert_eq!(found[0].banked, "e+ e- > mu+ mu-");
        assert_eq!(found[0].given, "e+ e- > ta+ ta-");
    }

    /// An absent `--run-card` resolves to the MadGraph LO defaults, which are a
    /// proton run: an artifact trained on a fixed-energy card must not silently
    /// accept it.
    #[test]
    fn omitting_the_run_card_does_not_pass_as_a_match() {
        let banked = artifact("e+ e- > mu+ mu-", card(BASE_CARD));
        let found = card_mismatches(
            &banked,
            &sm(),
            "e+ e- > mu+ mu-",
            &RunCard::default().split().0,
        );
        assert!(
            found.iter().any(|m| m.what == "run card `lpp1`"),
            "{found:?}"
        );
    }

    /// A generation setting a card moves is refused by its run-card name, the
    /// same card is not, and `--nevents` alone never reaches the comparison.
    #[test]
    fn a_changed_generation_setting_is_refused() {
        let banked = artifact("e+ e- > mu+ mu-", card(BASE_CARD));
        let same = RunCard::parse(BASE_CARD).unwrap().split().1;
        assert!(settings_mismatches(&banked, &same).is_empty());
        let moved = RunCard::parse(&format!("{BASE_CARD}  sum = event_norm\n"))
            .unwrap()
            .split()
            .1;
        let found = settings_mismatches(&banked, &moved);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].what, "run card `event_norm`");
        assert_eq!(
            (found[0].banked.as_str(), found[0].given.as_str()),
            ("average", "sum")
        );
        let syst = RunCard::parse(&format!("{BASE_CARD}  '0.5 1' = sys_scalefact\n"))
            .unwrap()
            .split()
            .1;
        let found = settings_mismatches(&banked, &syst);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].what, "run card `sys_scalefact`");
    }
}
