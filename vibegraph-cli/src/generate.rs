//! `vibegraph generate` — turn a converged `vibegraph integrate` run into an
//! unweighted event sample, serialised as a Les Houches Event File.
//!
//! The grid artifact is the run: it carries the trained per-channel VEGAS grids,
//! the channel selection weights they were trained under, the resolved run card,
//! the process string and the identity of the model behind it. What it does
//! **not** carry is the compiled amplitude, so the proc card has to be supplied
//! again — and is then checked against the artifact rather than trusted. A grid
//! trained on one process, or on the same process in another model, and replayed
//! against another samples a perfectly plausible-looking wrong distribution, so
//! the mismatch is refused instead of being taken as new input.
//!
//! The physics inputs are read from the artifact and never re-taken as flags: the
//! only knobs here are how many events to write, where to write them, and which
//! weight strategy to write them under.

use std::path::PathBuf;

use clap::{Args, ValueEnum};
use vibegraph::artifact::{ChannelKey, IntegrateArtifact};
use vibegraph::config::GlobalConfig;
use vibegraph::cuts::Cuts;
use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card_file, ParsingOptions};
use vibegraph::hadronic::{
    compile_subprocesses, initial_spin_color_average, process_external_legs, FixedBeamIntegrand,
};
use vibegraph::helas::eval::BoundAmplitude;
use vibegraph::helas::repr::lorentz::LorentzVector;
use vibegraph::lhef::build::{scalup, EventHeader, SubprocessRecord};
use vibegraph::lhef::emit::{
    Buffer, EmitPlan, EmitSummary, EventSource, StochasticRounding, UnweightStrategy, WeightedEvent,
};
use vibegraph::lhef::write::generator_element;
use vibegraph::pdf::{PdfMember, PdfSet};
use vibegraph::phasespace::GEV2_TO_PB;
use vibegraph::proton::{
    derive_flavor_groups, BeamOrdering, FlavorGroups, ProtonIntegrand, ProtonSelection,
};
use vibegraph::runcard::{BeamMode, RunCard};
use vibegraph::ufo::identity::ModelIdentity;
use vibegraph::ufo::{EvaluatedModel, UFOModel};
use vibegraph::unweight::{UnweightStats, Unweighter};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::integrate::{load_pdf_set, process_string, IntegrateError, NO_PDF, PDF_MEMBER};
use crate::network::NetworkPolicy;
use vibegraph::cache::pinned::DEFAULT_PDF_SET;

type V = LorentzVector<f64>;

/// Default output file.
const DEFAULT_OUTPUT: &str = "events.lhe";
/// `LPRUP` of the single `<init>` process entry every event refers back to.
const PROCESS_ID: i32 = 1;
/// Seed offsets, so the weight scan, the event generation and a strategy's own
/// draws never share a stream with each other or with the integration.
const SCAN_SEED_OFFSET: u64 = 0x5CA7_0000;
const GEN_SEED_OFFSET: u64 = 0xE7E7_0000;
const ROUNDING_SEED_OFFSET: u64 = 0x524E_4400;
/// Trials one event may cost before the source gives up. Sized well above the
/// reciprocal of the worst unweighting efficiency measured on any gated process
/// (~3e-2), so only a genuinely stuck sampler reaches it.
const MAX_TRIALS_PER_EVENT: usize = 5_000_000;

/// How the accept/reject weights become the file's events.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Strategy {
    /// Hold the sample, write the weights (`IDWTUP = -4`).
    Buffer,
    /// Stream, writing each event `floor(w) + Bernoulli(frac(w))` times at unit
    /// weight (`IDWTUP = +3`).
    StochasticRounding,
}

#[derive(Args, Debug)]
pub struct GenerateArgs {
    /// Grid artifact from a completed `vibegraph integrate` run.
    pub artifact: PathBuf,

    /// The same process card the artifact was integrated from.
    pub proc_card: PathBuf,

    /// The same run card the artifact was integrated with; absent → MadGraph LO
    /// defaults. A card that differs from the banked one is refused.
    #[arg(long)]
    pub run_card: Option<PathBuf>,

    /// Directory containing the proc card's UFO model directory; defaults to
    /// `$VIBEGRAPH_UFO_DIR`, then the `~/.vibegraph` cache, then the current
    /// directory. Unused for the built-in Standard Model.
    #[arg(long)]
    pub ufo_dir: Option<PathBuf>,

    /// Events to write; defaults to the run card's `nevents`.
    #[arg(long)]
    pub nevents: Option<usize>,

    /// Output Les Houches file.
    #[arg(short, long, default_value = DEFAULT_OUTPUT)]
    pub out: PathBuf,

    /// Overwrite an existing event file.
    #[arg(long)]
    pub force: bool,

    /// Weight strategy.
    #[arg(long, value_enum, default_value_t = Strategy::Buffer)]
    pub strategy: Strategy,

    /// RNG seed for the generation. Same seed, same sample.
    #[arg(long, default_value_t = 20_260_728)]
    pub seed: u64,

    /// LHAPDF set name (proton beams only). A set other than the one the artifact
    /// was integrated with is refused.
    #[arg(long, default_value = DEFAULT_PDF_SET)]
    pub pdf_set: String,

    /// Directory containing `<pdf-set>/`; defaults to `$VIBEGRAPH_PDF_DIR`, then
    /// the `~/.vibegraph` cache (offering to download the set if absent), then
    /// `validation/pdf` under the current directory.
    #[arg(long)]
    pub pdf_dir: Option<PathBuf>,
}

fn err(msg: impl Into<String>) -> IntegrateError {
    IntegrateError::Message(msg.into())
}

/// One way the cards handed to a generation run differ from the ones that trained
/// the grid.
#[derive(Debug, PartialEq)]
pub struct CardMismatch {
    pub what: String,
    pub banked: String,
    pub given: String,
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
pub fn card_mismatches(
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
pub fn pdf_mismatches(
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

fn refuse_on_mismatch(mismatches: &[CardMismatch]) -> Result<(), IntegrateError> {
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

/// The accept/reject pass as a replayable source of events.
///
/// Each accepted point is turned straight into a record: the momenta come back
/// from the same channel map at the same `u` the trial was accepted on, and the
/// discrete labels are selected off the diagonals at that point.
struct SampleSource<'a> {
    integrand: &'a FixedBeamIntegrand<'a>,
    records: &'a [SubprocessRecord],
    /// The unweighter as the scan left it, kept so a restart returns the pass to
    /// its initial state rather than continuing with accumulated statistics.
    pristine: Unweighter,
    unweighter: Unweighter,
    rng: ChaCha8Rng,
    seed: u64,
    /// `μF` from the run card, used when nothing in the matrix element moves with
    /// `αs` and so no per-event scale prescription was installed.
    static_scale: f64,
    alpha_qed: f64,
    momenta: Vec<V>,
}

impl<'a> SampleSource<'a> {
    fn new(
        integrand: &'a FixedBeamIntegrand<'a>,
        records: &'a [SubprocessRecord],
        unweighter: Unweighter,
        seed: u64,
        static_scale: f64,
        alpha_qed: f64,
    ) -> Self {
        SampleSource {
            integrand,
            records,
            pristine: unweighter.clone(),
            unweighter,
            rng: ChaCha8Rng::seed_from_u64(seed),
            seed,
            static_scale,
            alpha_qed,
            momenta: Vec::new(),
        }
    }

    fn stats(&self) -> &vibegraph::unweight::UnweightStats {
        self.unweighter.stats()
    }
}

impl EventSource for SampleSource<'_> {
    fn next_event(&mut self) -> Option<WeightedEvent> {
        let point =
            self.unweighter
                .next_event(self.integrand, &mut self.rng, MAX_TRIALS_PER_EVENT)?;
        self.integrand
            .event_in_channel(point.channel, &point.u, &mut self.momenta);
        let selection = self.integrand.select_event(
            &self.momenta,
            point.channel,
            [
                self.rng.random(),
                self.rng.random(),
                self.rng.random(),
                self.rng.random(),
            ],
        )?;
        let externals: Vec<[f64; 4]> = self
            .integrand
            .beams()
            .iter()
            .chain(self.momenta.iter())
            .map(|p| [p.e(), p.px(), p.py(), p.pz()])
            .collect();
        // The scales the matrix element itself ran at, so the record reports the
        // run rather than a second prescription compiled off the same card.
        let (scale, alpha_qcd) = match self
            .integrand
            .event_scales_at(&self.momenta, point.channel, &point.u)
        {
            Some(Ok(scales)) => {
                let alpha_s = self
                    .integrand
                    .alpha_s_source()
                    .map(|r| r.eval(scales.mu_r))
                    .unwrap_or(0.0);
                (scalup(&scales), alpha_s)
            }
            // A point the scale prescription rejects has no scale to report, so the
            // source stops rather than inventing one, and the strategy reports a
            // sample it could not fill.
            Some(Err(_)) => return None,
            None => (self.static_scale, 0.0),
        };
        let header = EventHeader {
            process_id: PROCESS_ID,
            // The strategy imposes the file's weight convention; this slot is
            // overwritten before the record is written.
            weight: 0.0,
            scale,
            alpha_qed: self.alpha_qed,
            alpha_qcd,
        };
        let record = self.records[selection.subprocess]
            .event(&externals, &selection.helicity, selection.flow, header)
            .ok()?;
        Some(WeightedEvent {
            record,
            weight: point.weight,
        })
    }

    fn restart(&mut self) {
        self.unweighter = self.pristine.clone();
        self.rng = ChaCha8Rng::seed_from_u64(self.seed);
    }

    fn sigma_pb(&self) -> f64 {
        self.unweighter.sigma_from_events() * GEV2_TO_PB
    }
}

pub fn run(args: &GenerateArgs, network: NetworkPolicy) -> Result<(), IntegrateError> {
    if !args.force && args.out.exists() {
        return Err(err(format!(
            "{} already exists (pass --force to overwrite)",
            args.out.display()
        )));
    }

    let artifact = IntegrateArtifact::read_from_path(&args.artifact)
        .map_err(|e| err(format!("cannot read {}: {e}", args.artifact.display())))?;

    let opts = ParsingOptions::default();
    let parsed = parse_proc_card_file(&args.proc_card, &opts)
        .map_err(|e| err(format!("failed to parse proc card: {e}")))?;
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
    let rc = config
        .load_run_card()
        .map_err(|e| err(format!("failed to load run card: {e}")))?;

    let hadronic = rc.beam_mode() == BeamMode::Proton;
    let mut mismatches = card_mismatches(&artifact, &model_id, &process, &rc);
    // A fixed-energy run reads no parton distributions, and the artifact says so;
    // the flag is only meaningful on the hadronic path.
    let pdf_set = if hadronic { &args.pdf_set } else { NO_PDF };
    mismatches.extend(pdf_mismatches(&artifact, pdf_set, PDF_MEMBER));
    refuse_on_mismatch(&mismatches)?;

    let evaluated = EvaluatedModel::from_model(model.clone());
    let nevents = args
        .nevents
        .unwrap_or(artifact.run_card.nevents.max(0) as usize);
    if nevents == 0 {
        return Err(err("no events requested"));
    }

    if hadronic {
        let set = load_pdf_set(&args.pdf_set, args.pdf_dir.as_ref(), network)?;
        let pdf = set
            .member(PDF_MEMBER)
            .map_err(|e| err(format!("cannot load PDF member {PDF_MEMBER}: {e}")))?;
        generate_proton_sample(
            args, &artifact, &parsed, &model, &evaluated, &rc, nevents, &set, &pdf,
        )?;
    } else {
        generate_sample(args, &artifact, &parsed, &model, &evaluated, &rc, nevents)?;
    }
    println!("wrote {}", args.out.display());
    Ok(())
}

/// Build the integrand the artifact describes, unweight against its grids, and
/// write the file.
#[allow(clippy::too_many_arguments)]
fn generate_sample(
    args: &GenerateArgs,
    artifact: &IntegrateArtifact,
    parsed: &vibegraph::diagrams::ParsedProcCard,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    rc: &RunCard,
    nevents: usize,
) -> Result<EmitSummary, IntegrateError> {
    let sqrt_s = rc.ebeam1 + rc.ebeam2;

    let sets = generate_from_proc_card(parsed, model)
        .map_err(|e| err(format!("failed to enumerate process: {e}")))?;
    let evals = compile_subprocesses(&sets, model, evaluated)
        .map_err(|e| err(format!("failed to compile subprocesses: {e}")))?;
    let bounds: Vec<_> = evals
        .iter()
        .map(|e| BoundAmplitude::<f64>::bind(e, evaluated))
        .collect();

    let rep = &evals[0];
    let legs = process_external_legs(rep, model, evaluated);
    let cuts = Cuts::compile(rc, &legs).map_err(|e| err(format!("failed to compile cuts: {e}")))?;
    let final_masses: Vec<f64> = rep.external_particles()[rep.n_in()..]
        .iter()
        .map(|&id| evaluated.mass(id))
        .collect();
    let spin_color_avg = initial_spin_color_average(rep, model, evaluated);
    let diagrams: Vec<_> = sets
        .iter()
        .flat_map(|s| s.diagrams.iter().cloned())
        .collect();

    let amps: Vec<&BoundAmplitude<f64>> = bounds.iter().collect();
    let mut integ = FixedBeamIntegrand::new(amps, &cuts, sqrt_s, final_masses, spin_color_avg);
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
    let beam_pdg = beam_pdg(&records)?;

    let scan = Unweighter::scan(
        &integ,
        artifact.channels.iter().map(|c| (&c.grid, c.neval)),
        args.seed ^ SCAN_SEED_OFFSET,
    );
    warn_on_empty_channels(&scan, artifact);

    // A model with no strong coupling installs no per-event scale prescription, and
    // no cross section depended on a factorisation scale; the run card's own is
    // then what the record reports.
    let static_scale = rc.dsqrt_q2fact1.max(rc.dsqrt_q2fact2);
    let alpha_qed = evaluated
        .param_values
        .get("aEW")
        .map(|v| v.re)
        .unwrap_or(0.0);

    let mut source = SampleSource::new(
        &integ,
        &records,
        scan,
        args.seed ^ GEN_SEED_OFFSET,
        static_scale,
        alpha_qed,
    );

    let strategy = weight_strategy(args);
    let plan = EmitPlan {
        nevents,
        sigma_pb: artifact.sigma_pb,
        sigma_err_pb: artifact.sigma_err_pb,
        beam_pdg,
        beam_energy: [sqrt_s / 2.0, sqrt_s / 2.0],
        // No parton densities on a fixed-energy run, so both beams report none.
        pdf_group: [0, 0],
        pdf_set: [0, 0],
        process_id: PROCESS_ID,
        trailer: vec![generator_element(
            "vibegraph",
            env!("CARGO_PKG_VERSION"),
            "",
        )],
        header: Some(file_header(args, artifact, strategy.as_ref())),
    };

    let summary = emit_to(args, &mut source, &plan, strategy.as_ref())?;
    report(
        artifact,
        source.stats(),
        source.sigma_pb(),
        &summary,
        strategy.as_ref(),
    );
    Ok(summary)
}

/// A channel the frozen scan never reached is never drawn from, so its share of the
/// banked cross section is simply missing from the sample. That is a property of the
/// run worth saying out loud rather than a failure.
fn warn_on_empty_channels(scan: &Unweighter, artifact: &IntegrateArtifact) {
    let empty = scan.empty_channels();
    if !empty.is_empty() {
        eprintln!(
            "warning: {} of {} channels produced no point in the weight scan and will never be \
             drawn from ({empty:?}); their share of the banked cross section is missing from the \
             sample",
            empty.len(),
            artifact.channels.len()
        );
    }
}

fn weight_strategy(args: &GenerateArgs) -> Box<dyn UnweightStrategy> {
    match args.strategy {
        Strategy::Buffer => Box::new(Buffer),
        Strategy::StochasticRounding => {
            Box::new(StochasticRounding::new(args.seed ^ ROUNDING_SEED_OFFSET))
        }
    }
}

/// The provenance block the file carries in its `<header>`.
fn file_header(
    args: &GenerateArgs,
    artifact: &IntegrateArtifact,
    strategy: &dyn UnweightStrategy,
) -> String {
    format!(
        "process {}\nartifact {}\nintegration sigma {:.6e} +- {:.6e} pb\nseed {}\n{}",
        artifact.process,
        args.artifact.display(),
        artifact.sigma_pb,
        artifact.sigma_err_pb,
        args.seed,
        strategy.describe(),
    )
}

fn emit_to(
    args: &GenerateArgs,
    source: &mut dyn EventSource,
    plan: &EmitPlan,
    strategy: &dyn UnweightStrategy,
) -> Result<EmitSummary, IntegrateError> {
    let file = std::fs::File::create(&args.out)
        .map_err(|e| err(format!("cannot create {}: {e}", args.out.display())))?;
    let mut sink = std::io::BufWriter::new(file);
    strategy
        .emit(source, plan, &mut sink)
        .map_err(|e| err(e.to_string()))
}

/// Every concrete flavour assignment of every group, under both beam orderings, as
/// the record layer sees it: `[group][member][ordering]`, with the exchanged slot
/// absent where the two beams carry the same parton and there is only one ordering.
type FlavorRecords = Vec<Vec<[Option<SubprocessRecord>; 2]>>;

/// The record slot an ordering occupies.
fn ordering_slot(ordering: BeamOrdering) -> usize {
    usize::from(ordering == BeamOrdering::Exchanged)
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
) -> Result<FlavorRecords, IntegrateError> {
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
/// A hadronic channel is a `(group, diagram)` pair and the grids are banked in the
/// integrand's own channel order, so the same set of keys in another order would
/// install every grid on the wrong channel — a run that samples a perfectly
/// plausible wrong distribution with nothing else to show for it. Comparing counts,
/// or comparing the keys as a set, would not see it.
fn check_channel_keys(
    artifact: &IntegrateArtifact,
    integ: &ProtonIntegrand<'_>,
) -> Result<(), IntegrateError> {
    let derived: Vec<ChannelKey> = integ
        .channel_ids()
        .iter()
        .map(|id| ChannelKey::GroupDiagram {
            group: id.group,
            diagram: id.diagram,
        })
        .collect();
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
        if banked.grid.ndim() != integ.channel_grid_ndim() {
            return Err(err(format!(
                "channel {j}'s banked grid is over {} coordinates, this process's channels over \
                 {}: the grids were trained on a different process",
                banked.grid.ndim(),
                integ.channel_grid_ndim()
            )));
        }
    }
    Ok(())
}

/// The accept/reject pass over a hadronic integrand as a replayable source.
///
/// The difference from a fixed-beam run is entirely in what an accepted point
/// carries: its own beam momentum fractions, its own scales, and a concrete flavour
/// assignment drawn from the parton luminosities at those fractions.
struct ProtonSampleSource<'a> {
    integrand: &'a ProtonIntegrand<'a>,
    records: &'a FlavorRecords,
    pristine: Unweighter,
    unweighter: Unweighter,
    rng: ChaCha8Rng,
    seed: u64,
    alpha_qed: f64,
}

impl<'a> ProtonSampleSource<'a> {
    fn new(
        integrand: &'a ProtonIntegrand<'a>,
        records: &'a FlavorRecords,
        unweighter: Unweighter,
        seed: u64,
        alpha_qed: f64,
    ) -> Self {
        ProtonSampleSource {
            integrand,
            records,
            pristine: unweighter.clone(),
            unweighter,
            rng: ChaCha8Rng::seed_from_u64(seed),
            seed,
            alpha_qed,
        }
    }

    fn record(&self, selection: &ProtonSelection) -> Option<&SubprocessRecord> {
        self.records[selection.group][selection.member][ordering_slot(selection.ordering)].as_ref()
    }
}

impl EventSource for ProtonSampleSource<'_> {
    fn next_event(&mut self) -> Option<WeightedEvent> {
        let point =
            self.unweighter
                .next_event(self.integrand, &mut self.rng, MAX_TRIALS_PER_EVENT)?;
        let event = self.integrand.event_in_channel(point.channel, &point.u)?;
        let selection = self.integrand.select_event(
            &event,
            [
                self.rng.random(),
                self.rng.random(),
                self.rng.random(),
                self.rng.random(),
                self.rng.random(),
            ],
        )?;
        let externals: Vec<[f64; 4]> = event
            .lab
            .iter()
            .map(|p| [p.e(), p.px(), p.py(), p.pz()])
            .collect();
        // The coupling the matrix element itself ran at, off the PDF set's own
        // tabulation — the one the densities were fitted with.
        let alpha_qcd = self
            .integrand
            .alpha_s_source()
            .map(|source| source.eval(event.scales.mu_r))
            .unwrap_or(0.0);
        let header = EventHeader {
            process_id: PROCESS_ID,
            // The strategy imposes the file's weight convention; this slot is
            // overwritten before the record is written.
            weight: 0.0,
            scale: scalup(&event.scales),
            alpha_qed: self.alpha_qed,
            alpha_qcd,
        };
        let record = self
            .record(&selection)?
            .event(&externals, &selection.helicity, selection.flow, header)
            .ok()?;
        Some(WeightedEvent {
            record,
            weight: point.weight,
        })
    }

    fn restart(&mut self) {
        self.unweighter = self.pristine.clone();
        self.rng = ChaCha8Rng::seed_from_u64(self.seed);
    }

    fn sigma_pb(&self) -> f64 {
        self.unweighter.sigma_from_events() * GEV2_TO_PB
    }
}

/// Rebuild the hadronic integrand the artifact describes, unweight against its
/// frozen grids, and write the file.
///
/// Every physics input is taken from the cards the artifact was checked against;
/// the channel selection weights are *installed* from the artifact rather than
/// re-surveyed, because a survey reproduces them only by accident and `alpha_j`
/// enters every channel's weight.
#[allow(clippy::too_many_arguments)]
fn generate_proton_sample(
    args: &GenerateArgs,
    artifact: &IntegrateArtifact,
    parsed: &vibegraph::diagrams::ParsedProcCard,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    rc: &RunCard,
    nevents: usize,
    set: &PdfSet,
    pdf: &PdfMember,
) -> Result<EmitSummary, IntegrateError> {
    let sqrt_s_had = rc.ebeam1 + rc.ebeam2;

    let sets = generate_from_proc_card(parsed, model)
        .map_err(|e| err(format!("failed to enumerate process: {e}")))?;
    let groups = derive_flavor_groups(sets, model, evaluated, rc)
        .map_err(|e| err(format!("failed to decompose into flavour groups: {e}")))?;
    let amps: Vec<BoundAmplitude<f64>> = groups
        .groups()
        .iter()
        .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), evaluated))
        .collect();

    let mut integ =
        ProtonIntegrand::new(&groups, &amps, evaluated, pdf, sqrt_s_had, rc.dsqrt_q2fact1)
            .map_err(|e| err(format!("failed to build the hadronic integrand: {e}")))?;
    integ
        .use_run_card_scales(model, evaluated, rc, Some(&set.info.alpha_s))
        .map_err(|e| err(format!("run card scale prescription: {e}")))?;

    check_channel_keys(artifact, &integ)?;
    let alphas: Vec<f64> = artifact.channels.iter().map(|c| c.alpha).collect();
    check_alphas(&alphas)?;
    integ.set_channel_alphas(alphas);

    let records = flavor_records(&groups, model, evaluated)?;
    let beam_pdg = hadron_beam_pdg(rc)?;

    let scan = Unweighter::scan(
        &integ,
        artifact.channels.iter().map(|c| (&c.grid, c.neval)),
        args.seed ^ SCAN_SEED_OFFSET,
    );
    warn_on_empty_channels(&scan, artifact);

    let alpha_qed = evaluated
        .param_values
        .get("aEW")
        .map(|v| v.re)
        .unwrap_or(0.0);
    let mut source = ProtonSampleSource::new(
        &integ,
        &records,
        scan,
        args.seed ^ GEN_SEED_OFFSET,
        alpha_qed,
    );

    let strategy = weight_strategy(args);
    let plan = EmitPlan {
        nevents,
        sigma_pb: artifact.sigma_pb,
        sigma_err_pb: artifact.sigma_err_pb,
        beam_pdg,
        beam_energy: [rc.ebeam1, rc.ebeam2],
        // MadGraph writes the LHAPDF id in `PDFSUP` and leaves `PDFGUP` at zero,
        // the accord's "the set is named by its LHAPDF id alone" spelling.
        pdf_group: [0, 0],
        pdf_set: [rc.lhaid as i32; 2],
        process_id: PROCESS_ID,
        trailer: vec![generator_element(
            "vibegraph",
            env!("CARGO_PKG_VERSION"),
            "",
        )],
        header: Some(file_header(args, artifact, strategy.as_ref())),
    };

    let summary = emit_to(args, &mut source, &plan, strategy.as_ref())?;
    report(
        artifact,
        source.unweighter.stats(),
        source.sigma_pb(),
        &summary,
        strategy.as_ref(),
    );
    Ok(summary)
}

/// `IDBMUP` for a hadron-collider run, from the run card's beam labels.
fn hadron_beam_pdg(rc: &RunCard) -> Result<[i32; 2], IntegrateError> {
    let mut out = [0i32; 2];
    for (slot, lpp) in out.iter_mut().zip([rc.lpp1, rc.lpp2]) {
        *slot = match lpp {
            1 => 2212,
            -1 => -2212,
            other => {
                return Err(err(format!(
                    "beam label lpp = {other} is not a proton beam; event generation covers \
                     lpp = 0 (fixed-energy partons) and lpp = ±1 (protons)"
                )))
            }
        };
    }
    Ok(out)
}

/// `IDBMUP` for a fixed-beam run: the incoming legs' PDG codes, which every
/// subprocess sharing one `<init>` block has to agree on.
fn beam_pdg(records: &[SubprocessRecord]) -> Result<[i32; 2], IntegrateError> {
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

/// The banked channel weights have to be a usable selection distribution before
/// they are installed, since the combiner asserts rather than reports.
fn check_alphas(alphas: &[f64]) -> Result<(), IntegrateError> {
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
) {
    println!("process:  {}", artifact.process);
    println!("strategy: {}", strategy.describe());
    println!(
        "sampling: {} events from {} accepted points in {} trials (efficiency {:.4e})",
        summary.written,
        summary.drawn,
        stats.trials,
        stats.efficiency()
    );
    println!(
        "overweight: rate {:.3e}, cross-section share {:.3e}, largest w/w_max {:.3}",
        stats.overweight_fraction(),
        stats.overweight_weight_share(),
        stats.ratio_max
    );
    println!(
        "σ:        {sample_sigma:.6} pb from the sample vs {:.6} ± {:.6} pb from the integration \
         ({:+.3}%)",
        artifact.sigma_pb,
        artifact.sigma_err_pb,
        100.0 * (sample_sigma / artifact.sigma_pb - 1.0)
    );
    println!(
        "file:     IDWTUP = {}, XSECUP = {:.6e} pb, XMAXUP = {:.6e}",
        strategy.weight_strategy().as_i32(),
        summary.xsec_pb,
        summary.xmax
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use vibegraph::artifact::{ChannelGrid, ChannelKey, FORMAT_VERSION};
    use vibegraph::ufo::sm::SMRestrict;
    use vibegraph::vegas::VegasGrid;

    fn sm() -> ModelIdentity {
        ModelIdentity::interned_sm(SMRestrict::Default)
    }

    fn artifact(process: &str, run_card: RunCard) -> IntegrateArtifact {
        IntegrateArtifact {
            format_version: FORMAT_VERSION,
            process: process.to_string(),
            model: sm(),
            pdf_set: "none".to_string(),
            pdf_member: 0,
            mu_f: 0.0,
            sqrt_s_had: 91.2,
            neval: 1000,
            niter: 2,
            seed: 1,
            run_card,
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
            sigma_pb: 1.0,
            sigma_err_pb: 0.01,
            chi2_per_dof: 1.0,
        }
    }

    fn card(text: &str) -> RunCard {
        RunCard::parse(text).expect("run card")
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
        let found = card_mismatches(&banked, &sm(), "e+ e- > mu+ mu-", &RunCard::default());
        assert!(
            found.iter().any(|m| m.what == "run card `lpp1`"),
            "{found:?}"
        );
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
        use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use vibegraph::ufo::sm::{sm_model, SMRestrict};
        use vibegraph::vegas::VegasGrid;

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

        let mut integ = FixedBeamIntegrand::new(
            bounds.iter().collect(),
            &cuts,
            rc.ebeam1 + rc.ebeam2,
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

        let mut source = SampleSource::new(&integ, &records, scan, 11, 91.188, 0.0075);
        let first: Vec<_> = (0..40)
            .map(|_| source.next_event().expect("an event"))
            .map(|e| (e.record, e.weight))
            .collect();
        let trials = source.stats().trials;
        let sigma = source.sigma_pb();
        assert!(trials > 40 && sigma > 0.0, "the pass produced nothing");

        source.restart();
        assert_eq!(source.stats().trials, 0, "the accumulators did not reset");
        let again: Vec<_> = (0..40)
            .map(|_| source.next_event().expect("an event"))
            .map(|e| (e.record, e.weight))
            .collect();
        assert_eq!(first, again, "a restarted source drew a different sample");
        // The accumulators went back with the stream, so a second pass reports the
        // sample it produced rather than both passes at once.
        assert_eq!(source.stats().trials, trials);
        assert_eq!(source.sigma_pb().to_bits(), sigma.to_bits());
    }
}
