//! `vibegraph generate` — turn a converged `vibegraph integrate` run into an
//! unweighted event sample, serialised as a Les Houches Event File.
//!
//! The grid artifact is the run, like a gridpack: it carries the trained
//! per-channel VEGAS grids, the channel selection weights they were trained
//! under, the proc card and run card the integration read, the PDF set it
//! convolved with, and the identity of the model behind it. So `generate` takes
//! no card: it re-parses the recorded proc card, re-reads the model and the PDF
//! set as assets, and checks what it read against the artifact's record rather
//! than trusting it — a recorded card that this build parses to another
//! process, or a model whose files changed since the integration, would sample
//! a plausible-looking wrong distribution, so each is refused.
//!
//! What `generate` does take is what only generation introduces: the event
//! count, the generation seed, the output, the weight strategy and its
//! options, the event normalisation, and a reweight card.
//!
//! The sampling itself is [`vibegraph::generate::generate`]; this command
//! gathers its inputs: the artifact, the model and PDF assets, the generation
//! settings the artifact recorded with `--nevents` and `--event-norm` applied,
//! and the options of how many events to write, where, and under which weight
//! strategy.

use std::path::PathBuf;

use clap::{Args, ValueEnum};
use vibegraph::artifact::IntegrateArtifact;
use vibegraph::config::GlobalConfig;
use vibegraph::diagrams::{parse_proc_card, ParsingOptions, SupportedCard};
use vibegraph::generate::{
    check_collider_energy, check_model, check_process, generate, reads_parton_densities,
    GenerateAssets, GenerateError, GenerateOptions, Strategy as SampleStrategy,
};
use vibegraph::reweight::card::ReweightCard;
use vibegraph::reweight::engine::ReweightOptions;
use vibegraph::reweight::{resolve, resolve_couplings, Launch};
use vibegraph::runcard::EventNorm;
use vibegraph::ufo::{EvaluatedModel, UFOModel};
use vibegraph::unweight::{MaxRule, ScanBudget, DEFAULT_EXCESS_SHARE};

use crate::error::{err, CliError};
use crate::integrate::{forbidden_onshell, load_pdf_set, process_string};
use crate::network::NetworkPolicy;
use crate::parallel::ParallelArgs;
use crate::tui;

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
    /// Hold the sample, write the weights (`IDWTUP = -4`, or `-3` under
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

/// How the written event weights are normalised, as the run card's
/// `event_norm` spells it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum EventNormArg {
    /// The weights' mean is the cross section (`IDWTUP = -4`).
    Average,
    /// The weights' sum is the cross section (`IDWTUP = -3`).
    Sum,
    /// Every weight is one (`IDWTUP = -3`).
    Unity,
}

impl From<EventNormArg> for EventNorm {
    fn from(norm: EventNormArg) -> Self {
        match norm {
            EventNormArg::Average => EventNorm::Average,
            EventNormArg::Sum => EventNorm::Sum,
            EventNormArg::Unity => EventNorm::Unity,
        }
    }
}

#[derive(Args, Debug)]
pub(crate) struct GenerateArgs {
    /// Grid artifact from a completed `vibegraph integrate` run.
    pub(crate) artifact: PathBuf,

    /// RNG seed for the generation; required. Same artifact, settings and seed,
    /// same sample, so each generation job from one grid needs its own seed.
    #[arg(long)]
    pub(crate) seed: Option<u64>,

    /// Events to write; defaults to the `nevents` the artifact recorded.
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

    /// How the written weights are normalised; defaults to the `event_norm` the
    /// artifact recorded. It moves the weights and the `<init>` block, not the
    /// events drawn.
    #[arg(long, value_enum)]
    pub(crate) event_norm: Option<EventNormArg>,

    /// Directory containing the recorded proc card's UFO model directory;
    /// defaults to `$VIBEGRAPH_UFO_DIR`, then the `~/.vibegraph` cache, then the
    /// current directory. Unused for the built-in Standard Model. The model
    /// found must be the one the artifact was integrated against.
    #[arg(long)]
    pub(crate) ufo_dir: Option<PathBuf>,

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

/// `--seed` has no default: two jobs drawing from one grid under one seed write
/// the same events, and a batch of jobs that all fell back to a default would
/// do so without anyone noticing.
fn generation_seed(args: &GenerateArgs) -> Result<u64, CliError> {
    args.seed.ok_or_else(|| {
        err(
            "generate needs --seed: each generation job needs its own seed, since two jobs \
             drawing from one grid with the same seed write the same events",
        )
    })
}

/// Where the model the recorded proc card names was read from, for a refusal
/// to point at.
fn model_location(parsed: &SupportedCard, config: &GlobalConfig) -> String {
    match &parsed.model {
        Some(import) if import.name != "sm" => config
            .ufo_search_path
            .join(&import.name)
            .display()
            .to_string(),
        _ => "(built in)".to_string(),
    }
}

/// The library's refusal, in this command's terms: a refusal a flag can fix
/// names the flag.
fn cli_error(e: GenerateError) -> CliError {
    match e {
        GenerateError::RoundingOffAverage { norm } => err(format!(
            "--strategy stochastic-rounding writes unit weights (IDWTUP = +3) and cannot honour \
             event_norm = {}; use --strategy buffer, or --event-norm average",
            norm.as_str()
        )),
        GenerateError::RoundingOnMixedMultiplicity { parts } => err(format!(
            "--strategy stochastic-rounding cannot normalise a sum over {parts} final-state \
             multiplicities to each one's integrated cross section; use --strategy buffer"
        )),
        other => err(other.to_string()),
    }
}

pub(crate) fn run(args: &GenerateArgs, network: NetworkPolicy) -> Result<(), CliError> {
    let seed = generation_seed(args)?;
    args.parallel.install().map_err(err)?;
    if !args.force && args.out.exists() {
        return Err(err(format!(
            "{} already exists (pass --force to overwrite)",
            args.out.display()
        )));
    }

    let artifact = IntegrateArtifact::read_from_path(&args.artifact)
        .map_err(|e| err(format!("cannot read {}: {e}", args.artifact.display())))?;
    let parsed = parse_proc_card(&artifact.proc_card, &ParsingOptions::default()).map_err(|e| {
        err(format!(
            "cannot parse the proc card recorded in {}: {e}; re-integrate",
            args.artifact.display()
        ))
    })?;
    check_process(&artifact, &process_string(&parsed)?).map_err(cli_error)?;

    let config = GlobalConfig {
        ufo_search_path: crate::assets::resolve_ufo_search_path(
            parsed.model.as_ref(),
            args.ufo_dir.as_deref(),
        )
        .map_err(err)?,
        restrict_path_override: None,
        run_card_path: None,
    };
    let (model, model_id) = config
        .load_ufo_with_identity(&parsed.model)
        .map_err(|e| err(format!("failed to load model: {e}")))?;
    check_model(&artifact, &model_id, &model_location(&parsed, &config)).map_err(cli_error)?;
    // The library refuses a stale collider energy too; checking it here does so
    // before the PDF set is read, which may mean a download.
    check_collider_energy(&artifact, &parsed).map_err(cli_error)?;

    tui::state::describe_model(
        &model_id.label(),
        &model_id.digest,
        model.particles.len(),
        model.vertices.len(),
        model.couplings.len(),
    );
    tui::state::describe_process(&artifact.process);
    tui::state::note_channels(artifact.channels.len());

    let mut settings = artifact.generate.clone();
    if let Some(nevents) = args.nevents {
        settings.nevents = nevents as u64;
    }
    if let Some(norm) = args.event_norm {
        settings.event_norm = norm.into();
    }
    if settings.nevents == 0 {
        return Err(err(match args.nevents {
            Some(_) => "no events requested: --nevents is 0".to_string(),
            None => format!(
                "no events requested: --nevents is absent and {} recorded nevents = 0",
                args.artifact.display(),
            ),
        }));
    }

    let evaluated = EvaluatedModel::from_model(model.clone());
    let reweight = reweight_launches(args, &parsed, &model)?;

    let set = if reads_parton_densities(&artifact, &parsed) {
        Some(load_pdf_set(&artifact.pdf_set, None, network)?)
    } else {
        None
    };
    let member = set
        .as_ref()
        .map(|set| {
            set.member(artifact.pdf_member).map_err(|e| {
                err(format!(
                    "cannot load PDF member {} of {}: {e}",
                    artifact.pdf_member, artifact.pdf_set
                ))
            })
        })
        .transpose()?;
    let assets = GenerateAssets {
        card: &parsed,
        model: &model,
        evaluated: &evaluated,
        pdf: set.as_ref().zip(member.as_ref()),
    };
    let options = GenerateOptions {
        seed,
        strategy: args.strategy.into(),
        scan_points: args.scan_points,
        max_rule: max_rule(args),
        reweight,
        enumeration: args.parallel.enumeration(),
        out: args.out.clone(),
        artifact_label: args.artifact.display().to_string(),
        on_scan: Some(&tui::state::note_unweighting_efficiency),
    };
    generate(&artifact, &assets, &settings, &options).map_err(cli_error)?;
    // The command's result: the path a caller pipes this to learn, at any
    // verbosity. Everything else the run had to say went to the notice stream.
    tui::result_line(format_args!("wrote {}", args.out.display()));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The CLI names the flag that fixes a rounding refusal; the library's own
    /// text names none.
    #[test]
    fn a_rounding_refusal_names_the_strategy_flag() {
        let why = cli_error(GenerateError::RoundingOffAverage {
            norm: EventNorm::Sum,
        })
        .to_string();
        assert!(
            why.starts_with("--strategy stochastic-rounding writes unit weights")
                && why.contains("event_norm = sum")
                && why.contains("--event-norm average"),
            "{why}"
        );
        let why = cli_error(GenerateError::RoundingOnMixedMultiplicity { parts: 2 }).to_string();
        assert!(
            why.starts_with("--strategy stochastic-rounding cannot normalise a sum over 2"),
            "{why}"
        );
    }
}
