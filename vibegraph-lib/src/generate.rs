//! Event generation from a completed integration: the trained per-channel VEGAS
//! grids of an [`IntegrateArtifact`], unweighted into a Les Houches Event File.
//!
//! [`generate`] takes the artifact, the assets it was integrated against (the
//! parsed proc card, the model, and on proton beams the PDF set), the
//! [`GenerateSettings`] of this generation and the [`GenerateOptions`] of how to
//! draw it. It reads no run card: every integration setting comes from the
//! artifact's own record of the card, the same one the integrand read at
//! integration, so the integrand rebuilt here is the one the grids were trained
//! on. A caller that keeps a grid and draws from it several times changes only
//! the settings and the options between calls.
//!
//! A reweighting request adds, to every event, its weight under each of the
//! card's alternative parameter hypotheses (see [`crate::reweight`]); the sample
//! itself, drawn under the model's own parameters, is unchanged by it.

mod audit;
mod source;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn};

use crate::artifact::IntegrateArtifact;
use crate::cuts::{Cuts, ForcedResonances};
use crate::diagrams::{
    forbidden_onshell_ids, generate_from_proc_card_in, DiagramSet, EnumerationPool, SupportedCard,
};
use crate::hadronic::{
    compile_subprocesses, initial_spin_color_average, process_external_legs,
    refuse_polarized_frame, FixedBeamIntegrand, InitialState, Observable,
};
use crate::helas::eval::BoundAmplitude;
use crate::lhef::build::SubprocessRecord;
use crate::lhef::emit::{
    Buffer, EmitPlan, EmitSummary, EventSource, PartSigma, StochasticRounding, UnweightStrategy,
};
use crate::lhef::resonance::{member_line_pdg, SubprocessResonances};
use crate::lhef::write::{generator_element, mg_run_card, InputsRead};
use crate::multiplicity::{
    multiplicity_groups, multiplicity_maps, refuse_mixed_multiplicity, MultiplicityGroups,
    MultiplicitySum,
};
use crate::onshell::subprocess_vetoes;
use crate::pdf::{PdfMember, PdfSet};
use crate::phasespace::maps::MapOptions;
use crate::proton::{BeamOrdering, FlavorGroups, ProtonBeams, ProtonIntegrand};
use crate::reweight::engine::{ReweightOptions, ReweightPlan, SubprocessSummary};
use crate::reweight::Launch;
use crate::runcard::{BeamMode, EventNorm};
use crate::ufo::identity::ModelIdentity;
use crate::ufo::{EvaluatedModel, UFOModel};
use crate::unweight::{MaxRule, ScanBudget, UnweightStats, Unweighter};

use source::{
    ordering_slot, report_resonances, FlavorRecords, PartRecords, ProtonSampleSource, SampleSource,
};

/// Seed offsets, so the weight scan, the event generation and a strategy's own
/// draws never share a stream with each other or with the integration.
const SCAN_SEED_OFFSET: u64 = 0x5CA7_0000;
const GEN_SEED_OFFSET: u64 = 0xE7E7_0000;
const ROUNDING_SEED_OFFSET: u64 = 0x524E_4400;
/// Trials one event may cost before the source gives up. Sized well above the
/// reciprocal of the worst unweighting efficiency measured on any gated process
/// (~3e-2), so only a genuinely stuck sampler reaches it.
const MAX_TRIALS_PER_EVENT: usize = 5_000_000;

/// What a generation writes, as opposed to what the grids were integrated
/// under: the run-card fields that reach only the event budget or the written
/// record ([`crate::runcard::RunCard::split`]).
///
/// An integration records these for replay, and nothing in it reads them, so a
/// generation may change any of them without re-integrating.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenerateSettings {
    /// Events to write.
    pub nevents: u64,
    /// How the written event weights are normalised.
    pub event_norm: EventNorm,
    /// The systematics request, carried to the record verbatim.
    pub systematics: Systematics,
    /// Whether a matched event carries its clustering record (`clusinfo`). No
    /// such block is written yet, so it reaches only the `<MGRunCard>` record.
    pub clustering_info: bool,
}

/// A request for post-hoc systematics variation weights, as the card spells it.
///
/// Nothing here evaluates the variations; the request is carried so the written
/// record says what was asked for.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Systematics {
    /// Whether variation weights are requested (`use_syst`).
    pub enabled: bool,
    /// The program, its arguments and the variations, each as its run-card name
    /// and the card's text for it, in the run card's order.
    pub fields: Vec<(String, String)>,
}

/// How the accept/reject weights become the file's events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strategy {
    /// Hold the sample, write the weights (`IDWTUP = -4`, or `-3` under
    /// `event_norm = sum` or `unity`).
    Buffer,
    /// Stream, writing each event `floor(w) + Bernoulli(frac(w))` times at unit
    /// weight (`IDWTUP = +3`).
    StochasticRounding,
}

/// How one generation draws its sample.
pub struct GenerateOptions<'a> {
    /// The generation seed. Same artifact, settings and seed, same sample.
    pub seed: u64,
    /// How the accepted points become the file's events.
    pub strategy: Strategy,
    /// Points the frozen `w_max` scan spends on each channel.
    pub scan_points: ScanBudget,
    /// The rule each channel's maximum is read off its scan with.
    pub max_rule: MaxRule,
    /// The reweighting hypotheses, resolved against the model, and how to
    /// evaluate them; `None` writes no `<rwgt>` block.
    pub reweight: Option<(Vec<Launch>, ReweightOptions)>,
    /// The pool the process is enumerated on.
    pub enumeration: EnumerationPool,
    /// The event file to write. A file whose reweighting fails its audit is
    /// removed.
    pub out: PathBuf,
    /// How the file's provenance header names the artifact.
    pub artifact_label: String,
    /// Told the unweighting efficiency the frozen scan predicts, before any
    /// event is drawn.
    pub on_scan: Option<&'a (dyn Fn(f64) + Sync)>,
}

/// What an artifact was integrated against and a generation reads again: the
/// parsed proc card, the model, and on proton beams the parton densities.
#[derive(Clone, Copy)]
pub struct GenerateAssets<'a> {
    /// The artifact's recorded proc card, as this build parses it.
    pub card: &'a SupportedCard,
    /// The model that card names, at the digest the artifact records.
    pub model: &'a UFOModel,
    /// `model` evaluated at its own parameters.
    pub evaluated: &'a EvaluatedModel,
    /// The PDF set and the member the artifact was integrated with; required
    /// where [`reads_parton_densities`], unread otherwise.
    pub pdf: Option<(&'a PdfSet, &'a PdfMember)>,
    /// What was read to build these, written to the event file's header.
    pub inputs: &'a InputsRead,
}

/// Why a generation stopped.
///
/// The refusals a front end may want to phrase in its own terms (a flag name,
/// say) are variants of their own; everything else is a message for whoever ran
/// the generation.
#[derive(Debug, Error)]
pub enum GenerateError {
    /// The stochastic-rounding strategy was asked for an `event_norm` other than
    /// `average`: it writes unit weights under `IDWTUP = +3`, which honours no
    /// other normalisation.
    #[error(
        "the stochastic-rounding strategy writes unit weights (IDWTUP = +3) and cannot honour \
         event_norm = {}; use the buffer strategy",
        .norm.as_str()
    )]
    RoundingOffAverage { norm: EventNorm },
    /// The stochastic-rounding strategy was asked to write a sum over several
    /// final-state multiplicities, whose parts it cannot each normalise to their
    /// own integrated cross section.
    #[error(
        "the stochastic-rounding strategy cannot normalise a sum over {parts} final-state \
         multiplicities to each one's integrated cross section; use the buffer strategy"
    )]
    RoundingOnMixedMultiplicity { parts: usize },
    /// The artifact's recorded proc card, parsed by this build, does not spell
    /// the process the artifact records.
    #[error(
        "the artifact's proc card parses to `{reparsed}`, but the artifact records the process \
         `{recorded}`: this build reads the card differently from the one that integrated it; \
         re-integrate"
    )]
    ProcessReparsed { recorded: String, reparsed: String },
    /// The model the recorded proc card resolves to is not the one the artifact
    /// was integrated against.
    #[error(
        "model {label} at {location} has digest {got}; the grid was integrated against {want}: \
         its files changed after vibegraph integrate ran; re-integrate, or restore them"
    )]
    ModelChanged {
        label: String,
        location: String,
        got: String,
        want: String,
    },
    /// A proton artifact's recorded collider energy is not the `2√(E₁E₂)` its
    /// recorded beams give: its grids were trained on the frames of another
    /// collision energy.
    #[error(
        "this artifact records a collider energy sqrt(s) = {recorded} GeV, and its run card's \
         proton beams of {} and {} GeV collide at sqrt(s) = 2*sqrt(ebeam1*ebeam2) = {rebuilt} \
         GeV; its grids were trained at a different collision energy. Re-integrate with the \
         current build",
        .beams[0],
        .beams[1]
    )]
    ColliderEnergyChanged {
        recorded: f64,
        rebuilt: f64,
        beams: [f64; 2],
    },
    /// Any other refusal, as a message for whoever ran the generation.
    #[error("{0}")]
    Other(String),
}

fn err(msg: impl Into<String>) -> GenerateError {
    GenerateError::Other(msg.into())
}

/// Refuse an artifact whose recorded proc card no longer parses, in this build,
/// to the process the artifact records. `reparsed` is the process string of the
/// recorded card as this build parses it.
pub fn check_process(artifact: &IntegrateArtifact, reparsed: &str) -> Result<(), GenerateError> {
    if artifact.process == reparsed {
        return Ok(());
    }
    Err(GenerateError::ProcessReparsed {
        recorded: artifact.process.clone(),
        reparsed: reparsed.to_string(),
    })
}

/// Refuse a model whose digest is not the one the artifact recorded. `found` is
/// the identity of the model the recorded proc card resolved to, and `location`
/// says where it was read from, for the message. The interned Standard Model is
/// checked the same way, so a build whose compiled-in model differs from the
/// integrating build's is refused too.
pub fn check_model(
    artifact: &IntegrateArtifact,
    found: &ModelIdentity,
    location: &str,
) -> Result<(), GenerateError> {
    if artifact.model.digest == found.digest {
        return Ok(());
    }
    Err(GenerateError::ModelChanged {
        label: found.label(),
        location: location.to_string(),
        got: found.digest.clone(),
        want: artifact.model.digest.clone(),
    })
}

/// Refuse a proton artifact whose recorded collider energy is not the one its
/// recorded run card's beams give, `2√(E₁E₂)`, to the bit.
///
/// An artifact that recorded `E₁ + E₂` at unequal beams had its grids trained
/// on the frames that energy implies, and replaying them under the current
/// frames would sample a plausible-looking wrong distribution. At equal beams
/// the two expressions agree to the bit, so such an artifact passes. A
/// generation that convolves with no parton densities (fixed beams, a decay)
/// is not checked: its frames never read the recorded `sqrt_s_had`.
pub fn check_collider_energy(
    artifact: &IntegrateArtifact,
    card: &SupportedCard,
) -> Result<(), GenerateError> {
    if !reads_parton_densities(artifact, card) {
        return Ok(());
    }
    let beams = ProtonBeams::from_run_card(&artifact.run_card);
    let rebuilt = beams.sqrt_s();
    if artifact.sqrt_s_had.to_bits() == rebuilt.to_bits() {
        return Ok(());
    }
    Err(GenerateError::ColliderEnergyChanged {
        recorded: artifact.sqrt_s_had,
        rebuilt,
        beams: beams.energies(),
    })
}

/// Draw the sample `settings` and `options` describe from `artifact`'s grids
/// and write it to [`GenerateOptions::out`].
///
/// The integrand is rebuilt from `assets` under the artifact's recorded run card
/// and phase-space maps, and the artifact's channel selection weights are
/// installed rather than re-surveyed; a process whose channels do not match the
/// banked grids, position by position, is refused, and so is a proton artifact
/// whose recorded collider energy its recorded beams no longer give
/// ([`check_collider_energy`]).
pub fn generate(
    artifact: &IntegrateArtifact,
    assets: &GenerateAssets<'_>,
    settings: &GenerateSettings,
    options: &GenerateOptions<'_>,
) -> Result<EmitSummary, GenerateError> {
    refuse_rounding_off_average(options.strategy, settings.event_norm)?;
    check_collider_energy(artifact, assets.card)?;
    if reads_parton_densities(artifact, assets.card) {
        let Some((set, pdf)) = assets.pdf else {
            return Err(err(
                "the artifact was integrated at proton beams, and no PDF set was given to \
                 generate with",
            ));
        };
        generate_proton_sample(artifact, assets, settings, options, set, pdf)
    } else {
        generate_sample(artifact, assets, settings, options)
    }
}

/// Whether a generation from `artifact`, whose recorded proc card parses to
/// `card`, convolves with parton densities, and so needs
/// [`GenerateAssets::pdf`]: a scattering integrated at proton beams. A decay is
/// at rest whatever the card's beams.
pub fn reads_parton_densities(artifact: &IntegrateArtifact, card: &SupportedCard) -> bool {
    !card.is_decay() && artifact.run_card.beam_mode() == BeamMode::Proton
}

/// Stochastic rounding writes unit weights, so on a sum over several
/// multiplicities each part's share of the file would be the sample's own rather
/// than its integration's. The emitter refuses it too; refusing here does so
/// before the scan and the draw.
fn refuse_rounding_on_mixed_multiplicity(
    strategy: Strategy,
    parts: usize,
) -> Result<(), GenerateError> {
    if parts <= 1 || !matches!(strategy, Strategy::StochasticRounding) {
        return Ok(());
    }
    Err(GenerateError::RoundingOnMixedMultiplicity { parts })
}

/// Stochastic rounding writes unit weights under `IDWTUP = +3`, which is no
/// `event_norm` but its own, so a request for `sum` or `unity` has no file to be
/// written. The emitter refuses it too; refusing here does so before the scan
/// and the draw.
fn refuse_rounding_off_average(strategy: Strategy, norm: EventNorm) -> Result<(), GenerateError> {
    if !matches!(strategy, Strategy::StochasticRounding) || StochasticRounding::honours(norm) {
        return Ok(());
    }
    Err(GenerateError::RoundingOffAverage { norm })
}

/// Build the fixed-beam or decay integrand the artifact describes, unweight
/// against its grids, and write the file.
fn generate_sample(
    artifact: &IntegrateArtifact,
    assets: &GenerateAssets<'_>,
    settings: &GenerateSettings,
    options: &GenerateOptions<'_>,
) -> Result<EmitSummary, GenerateError> {
    let GenerateAssets {
        card: parsed,
        model,
        evaluated,
        ..
    } = *assets;
    let rc = &artifact.run_card;
    let nevents = settings.nevents as usize;
    let launches = options.reweight.clone();
    let sets = generate_from_proc_card_in(parsed, model, options.enumeration)
        .map_err(|e| err(format!("failed to enumerate process: {e}")))?;
    refuse_mixed_multiplicity(&sets).map_err(err)?;
    let evals = compile_subprocesses(&sets, model, evaluated)
        .map_err(|e| err(format!("failed to compile subprocesses: {e}")))?;
    let vetoes = subprocess_vetoes(
        &sets,
        &evals,
        &forbidden_onshell(parsed, model)?,
        evaluated,
        rc,
    )
    .map_err(|e| err(e.to_string()))?;
    refuse_polarized_frame(rc, &evals, model).map_err(|e| err(e.to_string()))?;
    let bounds: Vec<_> = evals
        .iter()
        .map(|e| BoundAmplitude::<f64>::bind(e, evaluated))
        .collect();

    let rep = &evals[0];
    let legs = process_external_legs(rep, model, evaluated);
    let initial = InitialState::from_run_card(rc, &legs).map_err(|e| err(e.to_string()))?;
    let diagrams: Vec<_> = sets
        .iter()
        .flat_map(|s| s.diagrams.iter().cloned())
        .collect();
    let cuts = Cuts::compile_with(rc, &legs, &ForcedResonances::of(&diagrams, evaluated))
        .map_err(|e| err(format!("failed to compile cuts: {e}")))?;
    let final_masses: Vec<f64> = rep.external_particles()[rep.n_in()..]
        .iter()
        .map(|&id| evaluated.mass(id))
        .collect();
    let spin_color_avg = initial_spin_color_average(rep, model, evaluated);

    // The subprocesses `compile_subprocesses` kept, in the evaluators' order.
    let compiled_sets: Vec<&DiagramSet> = sets.iter().filter(|s| !s.diagrams.is_empty()).collect();
    let process_of: Vec<i32> = compiled_sets
        .iter()
        .map(|s| s.diagrams[0].provenance.process as i32)
        .collect();
    let reweight_plan = launches
        .map(|l| {
            reweight_plan(
                &compiled_sets,
                &forbidden_onshell(parsed, model)?,
                model,
                evaluated,
                l,
            )
        })
        .transpose()?;
    let resonances: Vec<SubprocessResonances> = if has_decay_chains(parsed) {
        compiled_sets
            .iter()
            .zip(&evals)
            .map(|(set, eval)| {
                SubprocessResonances::new(
                    eval,
                    &set.diagrams,
                    model,
                    evaluated,
                    rc.float("bwcutoff"),
                )
            })
            .collect::<Result<_, _>>()
            .map_err(|e| err(format!("cannot resolve the resonances: {e}")))?
    } else {
        Vec::new()
    };

    let amps: Vec<&BoundAmplitude<f64>> = bounds.iter().collect();
    let mut integ = FixedBeamIntegrand::new(amps, &cuts, initial, final_masses, spin_color_avg);
    integ.use_onshell_veto(&vetoes, evaluated);
    integ.use_resonances(&resonances);
    // The grids were trained under the artifact's maps; the channels are rebuilt
    // under exactly those, whatever the rule would choose today.
    integ.set_map_options(MapOptions::fixed(artifact.maps));
    integ
        .use_running_coupling(&diagrams, model, evaluated, rc)
        .map_err(|e| err(format!("run card scale prescription: {e}")))?;

    let alphas: Vec<f64> = artifact.channels.iter().map(|c| c.alpha).collect();
    check_alphas(&alphas)?;
    match integ.use_multichannel_with_alphas(&diagrams, evaluated, &alphas) {
        Some(Err(built)) => {
            return Err(err(format!(
                "the artifact banks {} channel grids but this process enumerates {built} \
                 diagrams: the grids were trained on a different process",
                alphas.len()
            )))
        }
        Some(Ok(())) => {}
        None => {
            if alphas.len() != 1 {
                return Err(err(
                    "the artifact banks several channel grids but the process enumerates no \
                     diagrams to map them onto",
                ));
            }
        }
    }
    for (j, channel) in artifact.channels.iter().enumerate() {
        if channel.grid.ndim() != integ.channel_grid_ndim() {
            return Err(err(format!(
                "channel {j}'s banked grid is over {} coordinates, this process's channels over \
                 {}: the grids were trained on a different process",
                channel.grid.ndim(),
                integ.channel_grid_ndim()
            )));
        }
    }

    let records: Vec<SubprocessRecord> = evals
        .iter()
        .map(|e| SubprocessRecord::new(e, model, evaluated))
        .collect::<Result<_, _>>()
        .map_err(|e| err(format!("cannot build a subprocess record: {e}")))?;
    let (beam_pdg, beam_energy) = match initial {
        InitialState::Beams(_) => (beam_pdg(&records)?, [rc.ebeam1, rc.ebeam2]),
        InitialState::Decay(decay) => (decay_pdg(&records)?, [decay.mass(), 0.0]),
    };
    let observable = initial.observable();

    let rule = options.max_rule;
    let scan = Unweighter::scan_with(
        &integ,
        artifact
            .channels
            .iter()
            .map(|c| (&c.grid, options.scan_points.draws_for(c.neval))),
        options.seed ^ SCAN_SEED_OFFSET,
        rule,
    );
    report_scan(
        &scan,
        artifact,
        options.scan_points,
        rule,
        observable,
        options.on_scan,
    );

    let alpha_qed = evaluated
        .param_values
        .get("aEW")
        .map(|v| v.re)
        .unwrap_or(0.0);

    let mut source = SampleSource::new(
        &integ,
        &records,
        scan,
        options.seed ^ GEN_SEED_OFFSET,
        rc,
        alpha_qed,
        &process_of,
        &resonances,
        reweight_plan.as_ref().map(ReweightPlan::bind),
    );

    let strategy = weight_strategy(options);
    let plan = EmitPlan {
        nevents,
        sigma_pb: artifact.sigma_pb,
        sigma_err_pb: artifact.sigma_err_pb,
        // One final-state multiplicity: the whole file is normalised to the
        // integration.
        parts: Vec::new(),
        beam_pdg,
        beam_energy,
        // No parton densities on a fixed-energy or a decay run; `PDFSUP` is
        // still MadEvent's, read off the card's `pdlabel`.
        pdf_group: [0, 0],
        pdf_set: [rc.pdfsup(); 2],
        process_ids: process_ids(parsed),
        trailer: vec![generator_element(
            "vibegraph",
            env!("CARGO_PKG_VERSION"),
            "",
        )],
        header_blocks: vec![assets.inputs.header_block()],
        header: Some(file_header(
            options,
            artifact,
            strategy.as_ref(),
            observable,
        )),
        reweights: declared_reweights(reweight_plan.as_ref()),
        event_norm: settings.event_norm,
    };

    let summary = emit_to(options, &mut source, &plan, strategy.as_ref())?;
    report_resonances(source.tally, summary.drawn);
    if reweight_plan.is_some() {
        source.audit.finish(&options.out)?;
    }
    report(
        artifact,
        source.stats(),
        source.sigma_pb(),
        &summary,
        strategy.as_ref(),
        settings.event_norm,
        observable,
    );
    Ok(summary)
}

/// Compile and plan the reweighting of `sets`, the run's concrete subprocesses in
/// the indexing its event source reports, and say what each event will cost.
fn reweight_plan(
    sets: &[&DiagramSet],
    forbidden_onshell: &[i64],
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    (launches, options): (Vec<Launch>, ReweightOptions),
) -> Result<ReweightPlan, GenerateError> {
    let n = launches.len();
    let plan = ReweightPlan::new(sets, forbidden_onshell, model, evaluated, launches, options)
        .map_err(|e| err(format!("reweighting: {e}")))?;
    report_reweighting(n, &plan.summary());
    Ok(plan)
}

/// One line per distinct subprocess plan: how many subprocesses share it, how
/// many amplitude evaluations an event costs, and what serves the hypotheses.
fn report_reweighting(hypotheses: usize, summary: &[SubprocessSummary]) {
    let mut lines: Vec<(String, usize)> = Vec::new();
    for sub in summary {
        let mut parts: Vec<String> = sub
            .polynomial
            .iter()
            .map(|g| {
                format!(
                    "{} as a polynomial in {} ({} amplitude monomials)",
                    g.hypotheses,
                    g.params.join(","),
                    g.terms
                )
            })
            .collect();
        if sub.exact > 0 {
            parts.push(format!("{} evaluated directly", sub.exact));
        }
        if sub.unchanged > 0 {
            parts.push(format!("{} not moving |M|^2", sub.unchanged));
        }
        let line = format!(
            "{} evaluations per event: {}",
            sub.evaluations(),
            parts.join(", ")
        );
        match lines.iter_mut().find(|(l, _)| *l == line) {
            Some((_, count)) => *count += 1,
            None => lines.push((line, 1)),
        }
    }
    info!("reweight: {hypotheses} hypotheses");
    for (line, count) in lines {
        info!("reweight: {count} subprocess(es), {line}");
    }
}

/// The `(id, description)` of every hypothesis the file declares.
fn declared_reweights(plan: Option<&ReweightPlan>) -> Vec<(String, String)> {
    plan.map(|p| {
        p.launches()
            .iter()
            .map(|l| (l.id.clone(), l.info.clone()))
            .collect()
    })
    .unwrap_or_default()
}

/// What the frozen scan cost and what it found, then the channels it never
/// reached.
///
/// The draw count and the rule are both printed because between them they set the
/// maxima every overweight number the run goes on to print is measured against.
fn report_scan(
    scan: &Unweighter,
    artifact: &IntegrateArtifact,
    budget: ScanBudget,
    rule: MaxRule,
    observable: Observable,
    on_scan: Option<&(dyn Fn(f64) + Sync)>,
) {
    let draws = budget.total_draws(artifact.channels.iter().map(|c| c.neval));
    let per_channel = match budget {
        ScanBudget::PerChannel(n) => format!("{n} per channel"),
        ScanBudget::IntegrationShare => {
            "each channel's share of the integration budget".to_string()
        }
    };
    let unit = observable.per_natural_unit();
    let efficiency = artifact.sigma_pb / (scan.total_w_max() * unit);
    if let Some(observe) = on_scan {
        observe(efficiency);
    }
    info!(
        "scan:     {draws} points over {} channels ({per_channel}), sum w_max {:.6e}, \
         predicted efficiency {efficiency:.4e}",
        artifact.channels.len(),
        scan.total_w_max() * unit,
    );
    match rule {
        MaxRule::Extremum => info!("maxima:   the largest weight each channel's scan saw"),
        MaxRule::Truncated { excess_share } => info!(
            "maxima:   {:.3}% of each channel's scanned cross section left above them, \
             summing to {:.4} of the summed largest weight",
            100.0 * excess_share,
            scan.total_w_max() / scan.total_w_peak()
        ),
    }
    warn_on_empty_channels(scan, artifact);
}

/// A channel the frozen scan never reached is never drawn from, so its share of the
/// banked cross section is simply missing from the sample. That is a property of the
/// run worth saying out loud rather than a failure.
fn warn_on_empty_channels(scan: &Unweighter, artifact: &IntegrateArtifact) {
    let empty = scan.empty_channels();
    if !empty.is_empty() {
        warn!(
            "{} of {} channels produced no point in the weight scan and will never be drawn from \
             ({empty:?}); their share of the banked cross section is missing from the sample",
            empty.len(),
            artifact.channels.len()
        );
    }
}

fn weight_strategy(options: &GenerateOptions<'_>) -> Box<dyn UnweightStrategy> {
    match options.strategy {
        Strategy::Buffer => Box::new(Buffer),
        Strategy::StochasticRounding => {
            Box::new(StochasticRounding::new(options.seed ^ ROUNDING_SEED_OFFSET))
        }
    }
}

/// The provenance block the file carries in its `<header>`.
fn file_header(
    options: &GenerateOptions<'_>,
    artifact: &IntegrateArtifact,
    strategy: &dyn UnweightStrategy,
    observable: Observable,
) -> String {
    let quantity = match observable {
        Observable::CrossSection => "sigma",
        // MadEvent's decay-run convention: `XSECUP` and `XWGTUP` carry the
        // width, in GeV, where a scattering run's carry picobarns.
        Observable::PartialWidth => "width (XSECUP and XWGTUP in GeV)",
    };
    format!(
        "process {}\nartifact {}\nintegration {quantity} {:.6e} +- {:.6e} {}\nseed {}\n{}",
        artifact.process,
        options.artifact_label,
        artifact.sigma_pb,
        artifact.sigma_err_pb,
        observable.unit(),
        options.seed,
        strategy.describe(),
    )
}

fn emit_to(
    options: &GenerateOptions<'_>,
    source: &mut dyn EventSource,
    plan: &EmitPlan,
    strategy: &dyn UnweightStrategy,
) -> Result<EmitSummary, GenerateError> {
    let file = std::fs::File::create(&options.out)
        .map_err(|e| err(format!("cannot create {}: {e}", options.out.display())))?;
    let mut sink = std::io::BufWriter::new(file);
    strategy
        .emit(source, plan, &mut sink)
        .map_err(|e| err(e.to_string()))
}

/// Resolve the record layer's view of every flavour a group's events can be
/// labelled with.
///
/// One compiled amplitude serves the whole group, so each member is that
/// amplitude's record relabelled: the PDG codes, the colour reps and the colour-flow
/// table are the member's own — a group may join subprocesses that route their colour
/// lines between different pairs of legs — and an exchanged beam ordering additionally
/// trades the two incoming legs of every per-leg field.
fn flavor_records(
    groups: &FlavorGroups,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
) -> Result<FlavorRecords, GenerateError> {
    let mut out = Vec::with_capacity(groups.groups().len());
    for group in groups.groups() {
        let base = SubprocessRecord::new(group.evaluator(), model, evaluated)
            .map_err(|e| err(format!("cannot build a subprocess record: {e}")))?;
        let mut members = Vec::with_capacity(group.members().len());
        for (i, member) in group.members().iter().enumerate() {
            let mut slots = [None, None];
            for ordering in [BeamOrdering::Direct, BeamOrdering::Exchanged] {
                if ordering == BeamOrdering::Exchanged && !member.has_mirror() {
                    continue;
                }
                let (pdg, order) = group.event_legs(i, ordering);
                let legs = group.event_leg_colors(i, ordering);
                slots[ordering_slot(ordering)] = Some(
                    base.relabelled(&order, &pdg, &legs, &member.flows)
                        .map_err(|e| err(format!("cannot relabel {pdg:?}: {e}")))?,
                );
            }
            members.push(slots);
        }
        out.push(members);
    }
    Ok(out)
}

/// The artifact's channels, matched to this process's **position by position**.
///
/// A hadronic channel is a `(group, diagram)` pair, or the pairs sharing one map
/// — of one multiplicity, on a card of several — and the grids are banked in the integrand's own channel
/// order, so the same set of keys in another order would install every grid on
/// the wrong channel — a run that samples a perfectly plausible wrong
/// distribution with nothing else to show for it. Comparing counts, or comparing
/// the keys as a set, would not see it.
fn check_channel_keys(
    artifact: &IntegrateArtifact,
    integ: &MultiplicitySum<'_>,
) -> Result<(), GenerateError> {
    let derived = integ.channel_keys();
    if artifact.channels.len() != derived.len() {
        return Err(err(format!(
            "the artifact banks {} channel grids but this process has {} (group, diagram) \
             channels: the grids were trained on a different process",
            artifact.channels.len(),
            derived.len()
        )));
    }
    for (j, (banked, want)) in artifact.channels.iter().zip(&derived).enumerate() {
        if banked.key != *want {
            return Err(err(format!(
                "the artifact's channel {j} is {:?} but this process's channel {j} is {want:?}: \
                 the grids were trained on a different process, or on the same one in a \
                 different channel order",
                banked.key
            )));
        }
        if banked.grid.ndim() != integ.channel_grid_ndim(j) {
            return Err(err(format!(
                "channel {j}'s banked grid is over {} coordinates, this process's channel over \
                 {}: the grids were trained on a different process",
                banked.grid.ndim(),
                integ.channel_grid_ndim(j)
            )));
        }
    }
    Ok(())
}

/// Rebuild the hadronic integrand the artifact describes, unweight against its
/// frozen grids, and write the file.
///
/// Every physics input is taken from the artifact's recorded run card and the
/// assets it was integrated against; the channel selection weights are
/// *installed* from the artifact rather than re-surveyed, because a survey
/// reproduces them only by accident and `alpha_j` enters every channel's weight.
fn generate_proton_sample(
    artifact: &IntegrateArtifact,
    assets: &GenerateAssets<'_>,
    settings: &GenerateSettings,
    options: &GenerateOptions<'_>,
    set: &PdfSet,
    pdf: &PdfMember,
) -> Result<EmitSummary, GenerateError> {
    let GenerateAssets {
        card: parsed,
        model,
        evaluated,
        ..
    } = *assets;
    let rc = &artifact.run_card;
    let nevents = settings.nevents as usize;
    let launches = options.reweight.clone();
    let beams = ProtonBeams::from_run_card(rc);
    let sqrt_s_had = beams.sqrt_s();

    let MultiplicityGroups { groups, vetoes } =
        multiplicity_groups(parsed, model, evaluated, rc, options.enumeration).map_err(err)?;
    refuse_rounding_on_mixed_multiplicity(options.strategy, groups.len())?;
    let mut parts_records = Vec::with_capacity(groups.len());
    // Under matching every record lists the resonances the clustering found on
    // their Breit–Wigner, whatever the card's decays.
    let matched = rc.int("ickkw") > 0;
    for part_groups in &groups {
        let (resonances, member_pdgs) = if has_decay_chains(parsed) || matched {
            group_resonances(part_groups, model, evaluated, rc.float("bwcutoff"))?
        } else {
            (Vec::new(), Vec::new())
        };
        parts_records.push(PartRecords {
            records: flavor_records(part_groups, model, evaluated)?,
            resonances,
            member_pdgs,
        });
    }
    let amps: Vec<Vec<BoundAmplitude<f64>>> = groups
        .iter()
        .map(|part| {
            part.groups()
                .iter()
                .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), evaluated))
                .collect()
        })
        .collect();

    // The grids were trained under the artifact's maps; the channels and the `τ`
    // draw are rebuilt under exactly those, whatever the rule would choose today.
    let maps = multiplicity_maps(
        &groups,
        evaluated,
        sqrt_s_had,
        MapOptions::fixed(artifact.maps),
    );
    let mut parts = Vec::with_capacity(groups.len());
    for (((part_groups, part_amps), part_vetoes), records) in
        groups.iter().zip(&amps).zip(&vetoes).zip(&parts_records)
    {
        let mut integ = ProtonIntegrand::new_with_maps(
            part_groups,
            part_amps,
            evaluated,
            pdf,
            beams,
            rc.dsqrt_q2fact1,
            maps,
        )
        .map_err(|e| err(format!("failed to build the hadronic integrand: {e}")))?;
        integ.use_onshell_veto(part_vetoes, evaluated);
        integ
            .use_run_card_scales(model, evaluated, rc, Some(&set.info.alpha_s))
            .map_err(|e| err(format!("run card scale prescription: {e}")))?;
        if has_decay_chains(parsed) {
            integ.use_resonances(&records.resonances);
        }
        parts.push(integ);
    }
    let mut integ = MultiplicitySum::new(parts);

    check_channel_keys(artifact, &integ)?;
    let alphas: Vec<f64> = artifact.channels.iter().map(|c| c.alpha).collect();
    for k in 0..groups.len() {
        check_alphas(&alphas[integ.offsets()[k]..integ.offsets()[k + 1]])?;
    }
    integ.set_channel_alphas(alphas);

    // Every member of every part, in the order `member_offsets` indexes them.
    let reweight_plan = launches
        .map(|l| {
            let members: Vec<&DiagramSet> = groups
                .iter()
                .flat_map(FlavorGroups::groups)
                .flat_map(|g| (0..g.members().len()).map(|i| g.member_diagram_set(i)))
                .collect();
            reweight_plan(
                &members,
                &forbidden_onshell(parsed, model)?,
                model,
                evaluated,
                l,
            )
        })
        .transpose()?;
    let beam_pdg = PROTON_BEAM_PDG;

    let rule = options.max_rule;
    let scan = Unweighter::scan_with(
        &integ,
        artifact
            .channels
            .iter()
            .map(|c| (&c.grid, options.scan_points.draws_for(c.neval))),
        options.seed ^ SCAN_SEED_OFFSET,
        rule,
    );
    report_scan(
        &scan,
        artifact,
        options.scan_points,
        rule,
        Observable::CrossSection,
        options.on_scan,
    );

    let alpha_qed = evaluated
        .param_values
        .get("aEW")
        .map(|v| v.re)
        .unwrap_or(0.0);
    let mut source = ProtonSampleSource::new(
        &integ,
        &parts_records,
        scan,
        options.seed ^ GEN_SEED_OFFSET,
        alpha_qed,
        reweight_plan.as_ref().map(ReweightPlan::bind),
    );

    let strategy = weight_strategy(options);
    let plan = EmitPlan {
        nevents,
        sigma_pb: artifact.sigma_pb,
        sigma_err_pb: artifact.sigma_err_pb,
        parts: part_sigmas(artifact, integ.offsets()),
        beam_pdg,
        beam_energy: [rc.ebeam1, rc.ebeam2],
        // MadGraph writes the LHAPDF id in `PDFSUP` and leaves `PDFGUP` at zero,
        // the accord's "the set is named by its LHAPDF id alone" spelling.
        pdf_group: [0, 0],
        pdf_set: [rc.pdfsup(); 2],
        process_ids: process_ids(parsed),
        trailer: vec![generator_element(
            "vibegraph",
            env!("CARGO_PKG_VERSION"),
            "",
        )],
        // The card a shower's MLM matching reads its parameters from, then
        // what the weights were computed from.
        header_blocks: if matched {
            vec![
                mg_run_card(rc, settings, options.seed),
                assets.inputs.header_block(),
            ]
        } else {
            vec![assets.inputs.header_block()]
        },
        header: Some(file_header(
            options,
            artifact,
            strategy.as_ref(),
            Observable::CrossSection,
        )),
        reweights: declared_reweights(reweight_plan.as_ref()),
        event_norm: settings.event_norm,
    };

    let summary = emit_to(options, &mut source, &plan, strategy.as_ref())?;
    report_resonances(source.tally, summary.drawn);
    if reweight_plan.is_some() {
        source.audit.finish(&options.out)?;
    }
    report(
        artifact,
        source.unweighter.stats(),
        source.sigma_pb(),
        &summary,
        strategy.as_ref(),
        settings.event_norm,
        Observable::CrossSection,
    );
    Ok(summary)
}

/// Each final-state multiplicity's integrated cross section, from its channels'
/// banked terms (`offsets[k]..offsets[k + 1]` in the artifact's channel order),
/// for a sum over several; empty for one, whose part is the whole integration.
fn part_sigmas(artifact: &IntegrateArtifact, offsets: &[usize]) -> Vec<PartSigma> {
    if offsets.len() <= 2 {
        return Vec::new();
    }
    offsets
        .windows(2)
        .map(|range| {
            let channels = &artifact.channels[range[0]..range[1]];
            PartSigma {
                sigma_pb: channels.iter().map(|c| c.sigma_pb).sum(),
                sigma_err_pb: channels
                    .iter()
                    .map(|c| c.sigma_err_pb * c.sigma_err_pb)
                    .sum::<f64>()
                    .sqrt(),
            }
        })
        .collect()
}

/// Every flavour group's resonance lines, and the PDG code each line carries in
/// each of the group's members (`[group][member][config][line]`).
///
/// A group's lines are its representative's; a member's line is the same
/// flavour or its antiparticle ([`member_line_pdg`]), resolved here once so a
/// member whose resonance neither describes is refused before any event is drawn.
#[allow(clippy::type_complexity)]
fn group_resonances(
    groups: &FlavorGroups,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    bwcutoff: f64,
) -> Result<(Vec<SubprocessResonances>, Vec<Vec<Vec<Vec<i32>>>>), GenerateError> {
    let mut tables = Vec::with_capacity(groups.groups().len());
    let mut pdgs = Vec::with_capacity(groups.groups().len());
    for group in groups.groups() {
        let table = SubprocessResonances::new(
            group.evaluator(),
            group.diagrams(),
            model,
            evaluated,
            bwcutoff,
        )
        .map_err(|e| err(format!("cannot resolve the resonances: {e}")))?;
        let mut members = Vec::with_capacity(group.members().len());
        for member in group.members() {
            let mut configs = Vec::with_capacity(group.evaluator().n_configs());
            for config in 0..group.evaluator().n_configs() {
                let lines = table
                    .lines(config)
                    .iter()
                    .map(|line| {
                        member_line_pdg(line, table.outgoing(), &member.outgoing, model).ok_or_else(
                            || {
                                err(format!(
                                    "the resonance {} of {} has no flavour in the subprocess \
                                     with outgoing {:?} that shares its matrix element",
                                    line.pdg,
                                    group.diagram_set().label(),
                                    member.outgoing
                                ))
                            },
                        )
                    })
                    .collect::<Result<Vec<i32>, _>>()?;
                configs.push(lines);
            }
            members.push(configs);
        }
        tables.push(table);
        pdgs.push(members);
    }
    Ok((tables, pdgs))
}

/// `IDBMUP` for a hadron-collider run: a proton on each side, the one hadron
/// beam pair [`BeamMode::Proton`] stands for (the run card admits it only as
/// `lpp1 = lpp2 = 1`).
const PROTON_BEAM_PDG: [i32; 2] = [2212, 2212];

/// `IDBMUP` for a fixed-beam run: the incoming legs' PDG codes, which every
/// subprocess sharing one `<init>` block has to agree on.
fn beam_pdg(records: &[SubprocessRecord]) -> Result<[i32; 2], GenerateError> {
    let first = &records[0];
    if first.n_in() != 2 {
        return Err(err(format!(
            "a Les Houches beam pair needs two incoming legs, this process has {}",
            first.n_in()
        )));
    }
    let beams = [first.pdg()[0], first.pdg()[1]];
    for record in records {
        if record.pdg()[..2] != beams {
            return Err(err(
                "the subprocesses do not share an initial state, so one <init> block cannot \
                 describe them",
            ));
        }
    }
    Ok(beams)
}

/// `IDBMUP` for a decay run: the decaying particle's PDG code and `0`, as MadEvent
/// writes it — the second beam slot is empty, since there is no second incoming
/// particle.
fn decay_pdg(records: &[SubprocessRecord]) -> Result<[i32; 2], GenerateError> {
    let mother = records[0].pdg()[0];
    if records
        .iter()
        .any(|r| r.n_in() != 1 || r.pdg()[0] != mother)
    {
        return Err(err(
            "the subprocesses do not share one decaying particle, so one <init> block cannot \
             describe them",
        ));
    }
    Ok([mother, 0])
}

/// The banked channel weights have to be a usable selection distribution before
/// they are installed, since the combiner asserts rather than reports.
fn check_alphas(alphas: &[f64]) -> Result<(), GenerateError> {
    if alphas.is_empty() {
        return Err(err("the artifact banks no channel grids"));
    }
    let sum: f64 = alphas.iter().sum();
    if !alphas.iter().all(|&a| a > 0.0) || (sum - 1.0).abs() >= 1e-9 {
        return Err(err(format!(
            "the artifact's channel weights are not a normalised selection distribution \
             (sum {sum}); regenerate it with `vibegraph integrate`"
        )));
    }
    Ok(())
}

fn report(
    artifact: &IntegrateArtifact,
    stats: &UnweightStats,
    sample_sigma: f64,
    summary: &EmitSummary,
    strategy: &dyn UnweightStrategy,
    norm: EventNorm,
    observable: Observable,
) {
    let (symbol, unit) = (observable.symbol(), observable.unit());
    info!("process:  {}", artifact.process);
    info!("strategy: {}", strategy.describe());
    info!(
        "sampling: {} events from {} accepted points in {} trials (efficiency {:.4e})",
        summary.written,
        summary.drawn,
        stats.trials,
        stats.efficiency()
    );
    info!(
        "overweight: rate {:.3e}, cross-section share {:.3e}, largest w/w_max {:.3}",
        stats.overweight_fraction(),
        stats.overweight_weight_share(),
        stats.ratio_max
    );
    info!(
        "{symbol}:        {sample_sigma:.6} ± {:.6} {unit} from the sample vs {:.6} ± {:.6} \
         {unit} from the integration ({:+.3}%, pull {:+.2})",
        summary.sample_sigma_err_pb,
        artifact.sigma_pb,
        artifact.sigma_err_pb,
        100.0 * (sample_sigma / artifact.sigma_pb - 1.0),
        (sample_sigma - artifact.sigma_pb)
            / summary
                .sample_sigma_err_pb
                .hypot(artifact.sigma_err_pb)
                .max(f64::MIN_POSITIVE)
    );
    info!(
        "file:     IDWTUP = {}, XSECUP = {:.6e} {unit}, XMAXUP = {:.6e}",
        strategy.weight_strategy(norm).as_i32(),
        summary.xsec_pb,
        summary.xmax
    );
}

/// The card's forbidden on-shell s-channels (`$`) as PDG codes, empty for a card
/// without them.
fn forbidden_onshell(parsed: &SupportedCard, model: &UFOModel) -> Result<Vec<i64>, GenerateError> {
    forbidden_onshell_ids(parsed, model).map_err(|e| err(e.to_string()))
}

/// Whether any process line of the card carries decays, which is when an event
/// record lists its resonances.
fn has_decay_chains(parsed: &SupportedCard) -> bool {
    parsed.processes.iter().any(|p| !p.decays.is_empty())
}

/// The `LPRUP` of every process the card declares: its process numbers, each
/// once, in increasing order.
fn process_ids(parsed: &SupportedCard) -> Vec<i32> {
    let mut ids: Vec<i32> = parsed.processes.iter().map(|p| p.id as i32).collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

#[cfg(test)]
mod tests {
    use super::audit::ReweightAudit;
    use super::*;
    use crate::artifact::{ChannelGrid, ChannelKey, FORMAT_VERSION};
    use crate::hadronic::FixedBeams;
    use crate::phasespace::maps::MapChoices;
    use crate::reweight::resolve;
    use crate::runcard::RunCard;
    use crate::ufo::identity::ModelIdentity;
    use crate::vegas::{GridRule, VegasGrid};

    const BASE_CARD: &str = "  0 = lpp1\n  0 = lpp2\n  45.6 = ebeam1\n  45.6 = ebeam2\n";

    /// A file the reweighting audit refuses is removed, and a clean audit leaves
    /// it in place.
    #[test]
    fn a_refused_reweighting_removes_its_file() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("events.lhe");
        let launches = vec![Launch {
            id: "zero".to_string(),
            info: String::new(),
            values: vec![("aEWM1".to_string(), 0.0)],
        }];
        let record = crate::lhef::record::LheEvent {
            process_id: 1,
            weight: 1.0,
            scale: 0.0,
            alpha_qed: 0.0,
            alpha_qcd: 0.0,
            particles: Vec::new(),
            trailer: Vec::new(),
            source: None,
        };

        std::fs::write(&out, "<LesHouchesEvents>").unwrap();
        let mut clean = ReweightAudit::default();
        clean.check(
            (0, BeamOrdering::Direct),
            1.0,
            1.0,
            &[],
            &record,
            (&[2.0], &launches),
        );
        clean.finish(&out).unwrap();
        assert!(out.exists());

        let mut refused = ReweightAudit::default();
        refused.check(
            (0, BeamOrdering::Direct),
            1.0,
            1.0,
            &[],
            &record,
            (&[f64::NAN], &launches),
        );
        let why = refused.finish(&out).unwrap_err().to_string();
        assert!(
            why.contains("non-finite") && why.contains("`zero`"),
            "{why}"
        );
        assert!(!out.exists(), "{why}");
    }

    /// Stochastic rounding is refused on a sum over multiplicities and only there.
    #[test]
    fn stochastic_rounding_is_refused_on_a_sum_over_multiplicities() {
        assert!(refuse_rounding_on_mixed_multiplicity(Strategy::StochasticRounding, 1).is_ok());
        assert!(refuse_rounding_on_mixed_multiplicity(Strategy::Buffer, 3).is_ok());
        assert!(refuse_rounding_on_mixed_multiplicity(Strategy::StochasticRounding, 3).is_err());
    }

    #[test]
    fn channel_weights_must_be_a_selection_distribution() {
        assert!(check_alphas(&[0.25, 0.75]).is_ok());
        assert!(check_alphas(&[]).is_err());
        assert!(check_alphas(&[0.25, 0.25]).is_err());
        assert!(check_alphas(&[-0.5, 1.5]).is_err());
    }

    /// [`EventSource::restart`] promises the identical sequence follows, and a
    /// two-pass strategy would be silently wrong if it did not. Nothing here
    /// consumes it yet, so this is where the promise is kept honest: the sequence
    /// is drawn, the source restarted, and the sequence drawn again — records and
    /// weights, not just counts, since a restart that reseeded only the accept
    /// stream would still produce the right number of events.
    #[test]
    fn restarting_the_source_replays_the_same_events() {
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use crate::ufo::sm::{sm_model, SMRestrict};
        use crate::vegas::VegasGrid;

        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let opts = ParsingOptions::default();
        let parsed = parse_proc_card("generate e+ e- > mu+ mu-", &opts).unwrap();
        let sets = generate_from_proc_card(&parsed, &model).unwrap();
        let evals = compile_subprocesses(&sets, &model, &evaluated).unwrap();
        let bounds: Vec<_> = evals
            .iter()
            .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
            .collect();
        let rep = &evals[0];
        let rc = RunCard::parse(BASE_CARD).unwrap();
        let cuts = Cuts::compile(&rc, &process_external_legs(rep, &model, &evaluated)).unwrap();
        let masses: Vec<f64> = rep.external_particles()[rep.n_in()..]
            .iter()
            .map(|&id| evaluated.mass(id))
            .collect();
        let avg = initial_spin_color_average(rep, &model, &evaluated);
        let diagrams: Vec<_> = sets
            .iter()
            .flat_map(|s| s.diagrams.iter().cloned())
            .collect();

        let legs = process_external_legs(rep, &model, &evaluated);
        let mut integ = FixedBeamIntegrand::new(
            bounds.iter().collect(),
            &cuts,
            FixedBeams::from_run_card(&rc, &legs),
            masses,
            avg,
        );
        let n = diagrams.len();
        let alphas = vec![1.0 / n as f64; n];
        integ
            .use_multichannel_with_alphas(&diagrams, &evaluated, &alphas)
            .unwrap()
            .unwrap();

        let records: Vec<SubprocessRecord> = evals
            .iter()
            .map(|e| SubprocessRecord::new(e, &model, &evaluated).unwrap())
            .collect();
        let grids: Vec<VegasGrid> = (0..n)
            .map(|_| VegasGrid::new(integ.channel_grid_ndim(), 16, 0.0))
            .collect();
        let scan = Unweighter::scan(&integ, grids.iter().map(|g| (g, 2_000)), 3);

        let process_of = vec![1; records.len()];
        let launches = resolve(
            &"launch\n set MZ 90.0\nlaunch\n set DECAY 23 3.0\nlaunch\n"
                .parse()
                .unwrap(),
            &model,
        )
        .unwrap();
        let compiled: Vec<&DiagramSet> = sets.iter().filter(|s| !s.diagrams.is_empty()).collect();
        let plan = ReweightPlan::new(
            &compiled,
            &[],
            &model,
            &evaluated,
            launches,
            ReweightOptions::default(),
        )
        .unwrap();
        let mut source = SampleSource::new(
            &integ,
            &records,
            scan,
            11,
            &rc,
            0.0075,
            &process_of,
            &[],
            Some(plan.bind()),
        );
        let first: Vec<_> = (0..40)
            .map(|_| source.next_event().expect("an event"))
            .map(|e| (e.record, e.weight, e.reweights))
            .collect();
        for (_, _, reweights) in &first {
            assert_eq!(reweights.len(), 3);
            assert!(reweights[0] > 0.0 && reweights[0] != 1.0, "{reweights:?}");
            assert!(reweights[1] > 0.0 && reweights[1] != 1.0, "{reweights:?}");
            assert_eq!(
                reweights[2], 1.0,
                "a launch changing nothing is the event itself"
            );
        }
        let trials = source.stats().trials;
        let sigma = source.sigma_pb();
        assert!(trials > 40 && sigma > 0.0, "the pass produced nothing");

        source.restart();
        assert_eq!(source.stats().trials, 0, "the accumulators did not reset");
        let again: Vec<_> = (0..40)
            .map(|_| source.next_event().expect("an event"))
            .map(|e| (e.record, e.weight, e.reweights))
            .collect();
        assert_eq!(first, again, "a restarted source drew a different sample");
        // The accumulators went back with the stream, so a second pass reports the
        // sample it produced rather than both passes at once.
        assert_eq!(source.stats().trials, trials);
        assert_eq!(source.sigma_pb().to_bits(), sigma.to_bits());
    }

    const UUBAR_EE: &str = "import model sm\ngenerate u u~ > e+ e-\n";

    /// A one-channel artifact of `u u~ > e+ e-` on the default (proton) card,
    /// against the interned default-restriction SM.
    fn proton_artifact() -> IntegrateArtifact {
        use crate::ufo::sm::SMRestrict;
        let text = UUBAR_EE;
        let (run_card, settings) = RunCard::default().split();
        IntegrateArtifact {
            format_version: FORMAT_VERSION,
            process: "u u~ > e+ e-".to_string(),
            proc_card: text.to_string(),
            model: ModelIdentity::interned_sm(SMRestrict::Default),
            inputs: crate::artifact::RunInputs::with_pdf(crate::artifact::PdfInput::Lhapdf {
                lhaid: 247000,
                set: "NNPDF23_lo_as_0130_qed".to_string(),
                member: 0,
                files: Vec::new(),
            }),
            mu_f: 0.0,
            sqrt_s_had: 13000.0,
            neval: 1000,
            niter: 1,
            seed: 1,
            run_card,
            generate: settings.clone(),
            channels: vec![ChannelGrid {
                key: ChannelKey::GroupChannel {
                    group: 0,
                    channel: 0,
                },
                alpha: 1.0,
                neval: 1000,
                grid: VegasGrid::new(4, 16, 1.5),
                sigma_pb: 1.0,
                sigma_err_pb: 0.01,
                chi2_per_dof: 1.0,
                sampler: None,
            }],
            sigma_pb: 1.0,
            sigma_err_pb: 0.01,
            chi2_per_dof: 1.0,
            maps: MapChoices::LEGACY,
            grid_rule: Some(GridRule::UNBOUNDED),
        }
    }

    /// The two stochastic-rounding refusals are phrased in the library's terms:
    /// they name the strategy and the setting, and no command-line flag.
    #[test]
    fn the_rounding_refusals_name_no_flag() {
        let off_average = refuse_rounding_off_average(Strategy::StochasticRounding, EventNorm::Sum)
            .unwrap_err()
            .to_string();
        assert!(
            off_average.contains("stochastic-rounding strategy")
                && off_average.contains("event_norm = sum")
                && off_average.contains("use the buffer strategy"),
            "{off_average}"
        );
        let mixed = refuse_rounding_on_mixed_multiplicity(Strategy::StochasticRounding, 2)
            .unwrap_err()
            .to_string();
        assert!(mixed.contains("a sum over 2 final-state"), "{mixed}");
        for why in [&off_average, &mixed] {
            assert!(!why.contains("--"), "a flag name in the library: {why}");
        }
    }

    /// The recorded process is checked against the recorded proc card as this
    /// build parses it: the same string passes, any other is refused naming both.
    #[test]
    fn a_proc_card_that_reparses_to_another_process_is_refused() {
        let artifact = proton_artifact();
        assert!(check_process(&artifact, "u u~ > e+ e-").is_ok());
        let why = check_process(&artifact, "u u~ > mu+ mu-")
            .unwrap_err()
            .to_string();
        assert!(
            why.contains("`u u~ > mu+ mu-`")
                && why.contains("`u u~ > e+ e-`")
                && why.contains("re-integrate"),
            "{why}"
        );
    }

    /// The model is checked by digest alone. The interned SM at the recorded
    /// restriction passes; another restriction, and the same label over
    /// different contents, are both refused, naming the model, where it was
    /// read and both digests.
    #[test]
    fn a_model_whose_digest_moved_is_refused() {
        use crate::ufo::sm::SMRestrict;
        let artifact = proton_artifact();
        let same = ModelIdentity::interned_sm(SMRestrict::Default);
        assert!(check_model(&artifact, &same, "(built in)").is_ok());

        let other = ModelIdentity::interned_sm(SMRestrict::NoBMass);
        let why = check_model(&artifact, &other, "(built in)")
            .unwrap_err()
            .to_string();
        assert!(
            why.starts_with("model sm-no_b_mass at (built in) has digest"),
            "{why}"
        );

        let mut altered = same.clone();
        altered.digest = crate::ufo::identity::digest_bytes(b"a different restrict card");
        let why = check_model(&artifact, &altered, "/models/sm")
            .unwrap_err()
            .to_string();
        assert_eq!(
            why,
            format!(
                "model sm-default at /models/sm has digest {}; the grid was integrated against \
                 {}: its files changed after vibegraph integrate ran; re-integrate, or restore \
                 them",
                altered.digest, artifact.model.digest
            )
        );
    }

    /// A proton artifact that recorded `E₁ + E₂` at unequal beams is refused,
    /// naming both energies; one that recorded `2√(E₁E₂)` passes, and so does an
    /// equal-beam artifact that recorded `E₁ + E₂`, which is the same number. A
    /// decay of the same artifact is not checked.
    #[test]
    fn a_proton_artifact_at_a_stale_collider_energy_is_refused() {
        use crate::diagrams::{parse_proc_card, ParsingOptions};

        let parsed = parse_proc_card(UUBAR_EE, &ParsingOptions::default()).unwrap();
        let mut unequal = proton_artifact();
        unequal.run_card.ebeam1 = 6500.0;
        unequal.run_card.ebeam2 = 4000.0;

        unequal.sqrt_s_had = 6500.0 + 4000.0;
        let why = check_collider_energy(&unequal, &parsed)
            .unwrap_err()
            .to_string();
        assert!(
            why.contains("sqrt(s) = 10500 GeV")
                && why.contains("beams of 6500 and 4000 GeV")
                && why.contains("= 10198.03902718557 GeV"),
            "{why}"
        );
        let decay = parse_proc_card(
            "import model sm\ngenerate t > w+ b\n",
            &ParsingOptions::default(),
        )
        .unwrap();
        assert!(check_collider_energy(&unequal, &decay).is_ok());

        unequal.sqrt_s_had = 2.0 * (6500.0_f64 * 4000.0).sqrt();
        assert!(check_collider_energy(&unequal, &parsed).is_ok());

        let mut equal = proton_artifact();
        equal.run_card.ebeam1 = 6500.0;
        equal.run_card.ebeam2 = 6500.0;
        equal.sqrt_s_had = 6500.0 + 6500.0;
        assert!(check_collider_energy(&equal, &parsed).is_ok());
    }

    /// An artifact integrated at proton beams cannot be replayed without the
    /// parton densities it was integrated with, nor at a collider energy its
    /// beams no longer give: the entry point refuses each before anything is
    /// enumerated, and writes no file.
    #[test]
    fn a_proton_artifact_without_a_pdf_set_is_refused() {
        use crate::diagrams::{parse_proc_card, ParsingOptions};
        use crate::ufo::sm::{sm_model, SMRestrict};

        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let parsed = parse_proc_card(UUBAR_EE, &ParsingOptions::default()).unwrap();
        let artifact = proton_artifact();
        let settings = artifact.generate.clone();
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("events.lhe");
        let inputs = InputsRead {
            pdf: None,
            grid_sha256: String::new(),
        };
        let assets = GenerateAssets {
            card: &parsed,
            model: &model,
            evaluated: &evaluated,
            pdf: None,
            inputs: &inputs,
        };
        let options = GenerateOptions {
            seed: 1,
            strategy: Strategy::Buffer,
            scan_points: ScanBudget::PerChannel(10),
            max_rule: MaxRule::Extremum,
            reweight: None,
            enumeration: EnumerationPool::default(),
            out: out.clone(),
            artifact_label: "grid.bin.zst".to_string(),
            on_scan: None,
        };
        let why = generate(&artifact, &assets, &settings, &options)
            .expect_err("no parton densities")
            .to_string();
        assert!(why.contains("no PDF set"), "{why}");
        assert!(!out.exists());

        // The entry point checks the recorded collider energy itself, whatever
        // its caller checked, and before it asks for the parton densities.
        let mut stale = artifact.clone();
        stale.run_card.ebeam1 = 6500.0;
        stale.run_card.ebeam2 = 4000.0;
        stale.sqrt_s_had = 6500.0 + 4000.0;
        let why = generate(&stale, &assets, &settings, &options).expect_err("a stale sqrt(s)");
        assert!(
            matches!(why, GenerateError::ColliderEnergyChanged { .. }),
            "{why}"
        );
        assert!(!out.exists());
    }
}
