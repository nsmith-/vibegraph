//! `vibegraph integrate` — compute a leading-order cross section for an
//! MG-validated process and persist the adapted VEGAS grids for a later sampling
//! phase.
//!
//! The process, model, beams, scales, and cuts are driven by the proc card
//! (`import model` + `generate`) and the run card, so the same card files drive
//! both this integration and a MadGraph reference run. Two beam modes are
//! supported, selected by the run card's `lpp1`/`lpp2`:
//!
//! * `lpp = 1` (proton beams) — an arbitrary process, PDF-convolved over a
//!   `(τ, y)` outer map with a per-diagram multichannel inner map pooled across
//!   the process's flavour groups, one VEGAS grid per `(group, diagram)` channel.
//! * `lpp = 0` (fixed-energy partonic beams) — an arbitrary process with no PDF
//!   convolution over any final multiplicity, sampled by a resonance-aware
//!   per-diagram multichannel map whose integral is split channel by channel, one
//!   VEGAS grid each.

use std::path::PathBuf;

use clap::Args;
use tracing::{info, warn};
use vibegraph::artifact::{
    ChannelGrid, ChannelKey, ChannelSampler, IntegrateArtifact, FORMAT_VERSION,
};
use vibegraph::config::GlobalConfig;
use vibegraph::cuts::Cuts;
use vibegraph::diagrams::{
    generate_from_proc_card_in, parse_proc_card_file, ParsedProcCard, ParsingOptions,
};
use vibegraph::hadronic::{
    compile_subprocesses, initial_spin_color_average, process_external_legs, ChannelIntegration,
    FixedBeamIntegrand, RunningCouplingReport,
};
use vibegraph::helas::eval::BoundAmplitude;
use vibegraph::pdf::{PdfMember, PdfSet};
use vibegraph::phasespace::GEV2_TO_PB;
use vibegraph::proton::{derive_flavor_groups, ProtonIntegrand};
use vibegraph::runcard::{BeamMode, RunCard};
use vibegraph::ufo::{EvaluatedModel, UFOModel};
use vibegraph::vegas::VegasResult;

use vibegraph::budget::{BlockAllocation, Budget, ConvergenceReport, StopReason};

use crate::assets;
use crate::network::NetworkPolicy;
use crate::parallel::ParallelArgs;
use crate::tui;
use vibegraph::cache::pinned::DEFAULT_PDF_SET;

/// PDF member index (central value; error members are not consumed at LO).
pub(crate) const PDF_MEMBER: u32 = 0;
/// Sentinel `pdf_set` recorded in the artifact for a no-PDF (fixed-energy) run.
pub(crate) const NO_PDF: &str = "none";
/// Artifact filename written inside the output directory.
const GRID_FILENAME: &str = "grid.bin.zst";

/// Points per survey iteration when α-adapting the multichannel combiner — the
/// hadronic one and the fixed-energy one alike — clamped to `[MIN, MAX]` around
/// the integration budget.
///
/// The cap is measured rather than asserted. Over `n_survey ∈ {10 000, 40 000,
/// 160 000, 640 000}` at [`ADAPT_ITERS`] iterations and [`ADAPT_DAMPING`], each
/// rung's converged α driving five independent integration seeds at the row's own
/// budget, the five-seed cross section is flat across the whole sixty-four-fold
/// range: inside `0.6%` of the banked MadGraph value at every rung on the 579- and
/// 615-channel `2 → 6` rows bar the single-draw excursion below, and inside
/// `0.04%` on the 24-channel `p p > l+ l- j`, against a five-seed `sd/σ` of
/// `0.21%` within a rung at the cap and the validation layer's own `0.8%` median
/// run-to-run spread. Points above the cap buy nothing the cross section can see.
///
/// Why they cannot: α itself is not resolved at any budget in that range. Surveyed
/// from three independent seeds at one budget, the 579-channel row's converged α
/// vectors sit `L1 0.15`–`0.74` apart out of a possible `2` — as far apart as the
/// budgets sit from each other — and one such draw moves that rung's five-seed σ
/// to `1.0%` where its two siblings read `0.05%` and `0.18%`. What limits a
/// several-hundred-channel split is the iteration count, not the points each
/// iteration spends; σ is insensitive to which draw it runs under, which is what
/// makes the cap free.
///
/// The floor is not free. `10 000` is measurably worse than `40 000` on both wide
/// rows — five-seed `sd/σ` of `0.52%` and `0.39%` against `0.21%` and `0.21%`,
/// a gap wider than the survey-seed noise on the means — so it is what keeps the
/// channel split from being the dominant noise in a small `--neval` run.
const MIN_ADAPT_SURVEY: usize = 10_000;
const MAX_ADAPT_SURVEY: usize = 40_000;
/// Survey→refine iterations for the α-adaptation.
const ADAPT_ITERS: usize = 6;
/// Kleiss–Pittau exponent the α-reallocation is damped by.
const ADAPT_DAMPING: f64 = 0.5;

/// Points the α-survey spends per iteration for an integration budget of
/// `neval`, warning when the clamp means `neval` did not decide it.
fn survey_points(neval: usize) -> usize {
    let n = neval.clamp(MIN_ADAPT_SURVEY, MAX_ADAPT_SURVEY);
    if n != neval {
        let direction = if n > neval { "raised" } else { "capped" };
        warn!(
            "α-survey neval {direction} from {neval} to {n} \
             (survey range {MIN_ADAPT_SURVEY}–{MAX_ADAPT_SURVEY})"
        );
    }
    n
}

/// Relative uncertainty a run converges to when no budget mode is asked for.
///
/// Below the Monte-Carlo error of every banked MadGraph reference this command is
/// compared against (0.048% on `p p > e+ e-`, 0.43% on `p p > l+ l- j`), so a
/// default run resolves more finely than the reference — and stops there rather
/// than spending points no comparison can see. Reaching it costs a `p p > l+ l- j`
/// run about what MadGraph spends for its own accuracy and a `p p > e+ e-` run
/// several times less.
const DEFAULT_TARGET_REL: f64 = 1.0e-3;

/// How the per-iteration budget is split across the phase-space channels.
#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Allocation {
    /// `Nⱼ ∝ αⱼ`, the split the channel weights imply.
    ByAlpha,
    /// Neyman: `Nⱼ ∝ αⱼ σⱼ`, driven by the variance each channel measures.
    Neyman,
}

impl From<Allocation> for BlockAllocation {
    fn from(a: Allocation) -> Self {
        match a {
            Allocation::ByAlpha => BlockAllocation::ByAlpha,
            Allocation::Neyman => BlockAllocation::Neyman,
        }
    }
}

#[derive(Args, Debug)]
pub struct IntegrateArgs {
    /// Process card selecting the model and process (`import model` + `generate`).
    pub proc_card: PathBuf,

    /// MadGraph `run_card.dat`; absent → MadGraph LO defaults.
    #[arg(long)]
    pub run_card: Option<PathBuf>,

    /// Output directory for the grid artifact (`<out>/grid.bin.zst`).
    #[arg(long, default_value = ".")]
    pub out: PathBuf,

    /// Overwrite an existing artifact.
    #[arg(long)]
    pub force: bool,

    /// LHAPDF set name (proton beams only).
    #[arg(long, default_value = DEFAULT_PDF_SET)]
    pub pdf_set: String,

    /// Directory containing `<pdf-set>/`; defaults to `$VIBEGRAPH_PDF_DIR`, then
    /// the `~/.vibegraph` cache (offering to download the set if absent), then
    /// `validation/pdf` under the current directory.
    #[arg(long)]
    pub pdf_dir: Option<PathBuf>,

    /// Directory containing the proc card's UFO model directory; defaults to
    /// `$VIBEGRAPH_UFO_DIR`, then the `~/.vibegraph` cache, then the current
    /// directory. Unused for the built-in Standard Model.
    #[arg(long)]
    pub ufo_dir: Option<PathBuf>,

    /// VEGAS evaluations per adaptation iteration.
    ///
    /// Split across the channels by their selection weights, but every channel
    /// draws at least 512 points so none stops covering its own region: a
    /// process with many channels spends `512 × channels` per iteration however
    /// small this is. Also sets the α-survey's points per iteration, clamped to
    /// `[10000, 40000]`.
    #[arg(long, default_value_t = 120_000)]
    pub neval: usize,

    /// VEGAS adaptation iterations under `--fixed-budget`. The default
    /// convergence mode decides its own iteration count, capped by
    /// `--max-iters`.
    #[arg(long, default_value_t = 12)]
    pub niter: usize,

    /// Integrate until σ's relative uncertainty reaches this.
    ///
    /// The uncertainty the stop reads is the quoted one widened by each
    /// channel's own `√max(1, χ²/dof)`, so a run whose iterations disagree by
    /// more than their error bars keeps going. `--neval` sets the points an
    /// iteration spends; how many iterations run is what the target decides,
    /// bounded by `--min-iters`, `--max-iters` and `--max-points`.
    #[arg(long, value_name = "REL", default_value_t = DEFAULT_TARGET_REL, conflicts_with = "fixed_budget")]
    pub target_rel: f64,

    /// Spend a fixed `--neval × --niter` and stop, instead of converging to
    /// `--target-rel`.
    ///
    /// The mode a banked run is reproducible under: a fixed budget draws the
    /// same points from the same seed every time, where a convergence run's
    /// length depends on the variance it measures.
    #[arg(long)]
    pub fixed_budget: bool,

    /// Iterations that must run before the convergence target may stop a run.
    #[arg(long, default_value_t = 6)]
    pub min_iters: usize,

    /// Iteration cap for `--target-rel`.
    ///
    /// A safety bound, not a budget: a run that reaches its target stops there,
    /// so the cap only ever binds on a run whose iterations keep disagreeing.
    /// Sized so that the widest gated process needing ~156 iterations at the
    /// default `--neval` converges with headroom; `--max-points` still bounds
    /// the total spend.
    #[arg(long, default_value_t = 500)]
    pub max_iters: usize,

    /// Evaluation cap for `--target-rel`, over all channels and iterations.
    #[arg(long, default_value_t = 400_000_000)]
    pub max_points: u64,

    /// How the per-iteration budget is split across channels.
    ///
    /// Defaults to `by-alpha` under `--fixed-budget` — the split a banked run is
    /// reproducible under — and to `neyman` under a convergence target, where it
    /// reaches the same accuracy for roughly half the evaluations on a wide
    /// multichannel process (`p p > l+ l- j`, 24 channels, 8 seeds: 2.94M
    /// evaluations against 6.41M at a 0.179% target) and makes no measurable
    /// difference on a narrow one (`p p > e+ e-`, 4 channels).
    #[arg(long, value_enum)]
    pub allocate: Option<Allocation>,

    /// RNG seed for the integration.
    #[arg(long, default_value_t = 20_260_719)]
    pub seed: u64,

    #[command(flatten)]
    pub parallel: ParallelArgs,
}

/// The failure surface of the `integrate` command. Displayed to stderr by the
/// binary's top-level handler.
#[derive(Debug)]
pub enum IntegrateError {
    Message(String),
}

impl std::fmt::Display for IntegrateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntegrateError::Message(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for IntegrateError {}

fn err(msg: impl Into<String>) -> IntegrateError {
    IntegrateError::Message(msg.into())
}

impl IntegrateArgs {
    /// The channel-allocation rule, defaulted by mode where the flag is absent.
    fn allocation(&self) -> BlockAllocation {
        match self.allocate {
            Some(a) => a.into(),
            None if self.fixed_budget => BlockAllocation::ByAlpha,
            None => BlockAllocation::Neyman,
        }
    }

    /// What this invocation asks the integrator to spend: however many
    /// iterations it takes to reach `--target-rel`, or the fixed
    /// `--neval × --niter` of `--fixed-budget`.
    fn budget(&self) -> Result<Budget, IntegrateError> {
        if self.fixed_budget {
            return Ok(Budget::Fixed {
                neval: self.neval,
                niter: self.niter,
            });
        }
        if !(self.target_rel > 0.0 && self.target_rel < 1.0) {
            return Err(err(format!(
                "--target-rel must be a relative uncertainty in (0, 1); got {}",
                self.target_rel
            )));
        }
        Ok(Budget::Target {
            target_rel: self.target_rel,
            neval: self.neval,
            min_iters: self.min_iters,
            max_iters: self.max_iters,
            max_points: self.max_points,
        })
    }
}

/// Locate and load a PDF set by name, which may mean asking to download it.
pub(crate) fn load_pdf_set(
    name: &str,
    pdf_dir: Option<&PathBuf>,
    network: NetworkPolicy,
) -> Result<PdfSet, IntegrateError> {
    let set_dir =
        assets::resolve_pdf_set_dir(name, pdf_dir.map(|p| p.as_path()), network).map_err(err)?;
    PdfSet::load(&set_dir, name).map_err(|e| {
        err(format!(
            "cannot load PDF set {name} from {}: {e}",
            set_dir.display()
        ))
    })
}

/// The canonical string of the proc card's first process, for artifact metadata.
pub fn process_string(parsed: &ParsedProcCard) -> Result<String, IntegrateError> {
    let spec = parsed
        .processes
        .first()
        .ok_or_else(|| err("proc card has no process"))?;
    Ok(format!("{spec}"))
}

/// The factorization scale the run artifact records: the run card's constant when
/// it fixes one on both beams, and `0` when the scale is chosen per event and no
/// single number describes the run.
fn recorded_mu_f(report: &RunningCouplingReport) -> f64 {
    match report.constant_scales {
        Some(scales) if scales.mu_f[0] == scales.mu_f[1] => scales.mu_f[0],
        _ => 0.0,
    }
}

/// The metadata printed and banked for a completed run, independent of beam mode.
struct RunOutput {
    process: String,
    pdf_set: String,
    mu_f: f64,
    sqrt_s: f64,
    /// One trained grid per phase-space channel, in channel order.
    channels: Vec<ChannelGrid>,
    result: VegasResult,
    convergence: ConvergenceReport,
}

/// Convert one channel's integration into the artifact record, with its term's
/// integral and error in picobarns and the composition its map was built by.
fn bank_channel(
    key: ChannelKey,
    c: &ChannelIntegration,
    sampler: Option<ChannelSampler>,
) -> ChannelGrid {
    ChannelGrid {
        key,
        alpha: c.alpha,
        neval: c.neval,
        grid: c.grid.clone(),
        sigma_pb: c.result.integral * GEV2_TO_PB,
        sigma_err_pb: c.result.std_dev * GEV2_TO_PB,
        chi2_per_dof: c.result.chi2_per_dof,
        sampler,
    }
}

pub fn run(args: &IntegrateArgs, network: NetworkPolicy) -> Result<(), IntegrateError> {
    args.parallel.install().map_err(err)?;
    // Refuse to clobber an existing artifact before spending the integration.
    let out_path = args.out.join(GRID_FILENAME);
    if !args.force && out_path.exists() {
        return Err(err(format!(
            "{} already exists (pass --force to overwrite)",
            out_path.display()
        )));
    }

    let opts = ParsingOptions::default();
    let parsed = parse_proc_card_file(&args.proc_card, &opts)
        .map_err(|e| err(format!("failed to parse proc card: {e}")))?;
    let process = process_string(&parsed)?;

    let config = GlobalConfig {
        ufo_search_path: assets::resolve_ufo_search_path(
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
    tui::state::describe_model(
        &model_id.label(),
        &model_id.digest,
        model.particles.len(),
        model.vertices.len(),
        model.couplings.len(),
    );
    tui::state::describe_process(&process);

    let evaluated = EvaluatedModel::from_model(model.clone());

    let output = match rc.beam_mode() {
        BeamMode::Proton => {
            integrate_proton(args, &parsed, &model, &evaluated, &rc, process, network)?
        }
        BeamMode::FixedEnergy => {
            integrate_fixed_energy(args, &parsed, &model, &evaluated, &rc, process)?
        }
    };

    // A run stopped before its warm-up finished kept no iteration, so its terms
    // are combined over nothing and its grids were never refined past the ones
    // the discard exists to throw away. There is no cross section to print and
    // no artifact worth banking; refusing is what keeps a stopped run from
    // reading downstream as a completed one.
    if output.convergence.kept_iterations == 0 {
        return Err(err(
            "stopped before any iteration was kept: nothing was measured, and no artifact \
             was written",
        ));
    }

    let sigma_pb = output.result.integral * GEV2_TO_PB;
    let sigma_err_pb = output.result.std_dev * GEV2_TO_PB;

    info!("process:  {}", output.process);
    info!("model:    {} ({})", model_id.label(), model_id.digest);
    info!("PDF set:  {} (member {PDF_MEMBER})", output.pdf_set);
    info!("√s:       {} GeV,  μF = {} GeV", output.sqrt_s, output.mu_f);
    let conv = &output.convergence;
    info!(
        "VEGAS:    {} evals over {} iters, seed {} (χ²/dof = {:.3})",
        conv.points, conv.iterations, args.seed, output.result.chi2_per_dof
    );
    if output.channels.len() > 1 {
        let total_neval: usize = output.channels.iter().map(|c| c.neval).sum();
        info!(
            "channels: {} grids, {} evals in the last iteration, allocated {}",
            output.channels.len(),
            total_neval,
            match args.allocation() {
                BlockAllocation::ByAlpha => "by α",
                BlockAllocation::Neyman => "by αⱼσⱼ (Neyman)",
            }
        );
    }
    if let Some(target) = conv.target_rel {
        // Price the caps at what an iteration really costs rather than at
        // `--neval`: on a process with more channels than the per-channel floor
        // leaves room for, the two differ by a factor, and both caps then bind
        // somewhere the flags on the command line do not suggest.
        if conv.points_per_iteration > args.neval {
            info!(
                "budget:   an iteration spends {} evaluations against the {} asked for, so \
                 --max-points {} allows {} iterations and --max-iters {} would cost {}",
                conv.points_per_iteration,
                args.neval,
                args.max_points,
                args.max_points / conv.points_per_iteration.max(1) as u64,
                args.max_iters,
                args.max_iters as u64 * conv.points_per_iteration as u64,
            );
        }
        info!(
            "target:   {:.4}% relative, {} after {} iterations and {} evaluations \
             (quoted {:.4}%, χ²-scaled {:.4}%)",
            100.0 * target,
            match conv.stop {
                StopReason::TargetMet => "met",
                StopReason::MaxIters => "GAVE UP on the iteration cap",
                StopReason::MaxPoints => "GAVE UP on the evaluation cap",
                StopReason::Aborted => "ABANDONED at the operator's request",
                StopReason::Budget => "unreachable",
            },
            conv.iterations,
            conv.points,
            100.0 * conv.achieved_rel,
            100.0 * conv.scaled_rel,
        );
    }
    // The command's result, and the reason `stdout` carries nothing else: a
    // caller pipes this to read the cross section, at any verbosity.
    tui::result_line(format_args!("σ = {sigma_pb:.6} ± {sigma_err_pb:.6} pb"));

    let artifact = IntegrateArtifact {
        format_version: FORMAT_VERSION,
        process: output.process,
        model: model_id,
        pdf_set: output.pdf_set,
        pdf_member: PDF_MEMBER,
        mu_f: output.mu_f,
        sqrt_s_had: output.sqrt_s,
        neval: args.neval,
        niter: output.convergence.iterations,
        seed: args.seed,
        run_card: rc,
        channels: output.channels,
        sigma_pb,
        sigma_err_pb,
        chi2_per_dof: output.result.chi2_per_dof,
    };

    std::fs::create_dir_all(&args.out).map_err(|e| {
        err(format!(
            "cannot create output directory {}: {e}",
            args.out.display()
        ))
    })?;
    artifact
        .write_to_path(&out_path, args.force)
        .map_err(|e| err(e.to_string()))?;
    tui::result_line(format_args!("wrote {}", out_path.display()));

    Ok(())
}

/// Proton beams (`lpp = 1`): load the PDF set and integrate over the flavour
/// decomposition.
fn integrate_proton(
    args: &IntegrateArgs,
    parsed: &ParsedProcCard,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    rc: &RunCard,
    process: String,
    network: NetworkPolicy,
) -> Result<RunOutput, IntegrateError> {
    let set = load_pdf_set(&args.pdf_set, args.pdf_dir.as_ref(), network)?;
    let pdf = set
        .member(PDF_MEMBER)
        .map_err(|e| err(format!("cannot load PDF member {PDF_MEMBER}: {e}")))?;

    integrate_hadronic(args, parsed, model, evaluated, rc, &set, &pdf, process)
}

/// Proton beams over an arbitrary process (`lpp = 1`): the proc card's enumeration
/// is partitioned into flavour groups, each group's summed parton-distribution
/// luminosity multiplying one compiled matrix element, and the pooled per-diagram
/// channels of every group are integrated one grid at a time.
///
/// The channel selection weights are adapted on the *hadronic* mixture before the
/// grids are trained, so the survey sees the integrand the integration will and
/// weight goes to the channels carrying variance of the cross section rather than
/// of a partonic one at some representative energy.
// Every argument is an independently-loaded piece of the run's setup, and the
// two callers hold them separately; bundling them into a struct here would only
// move the same list one level out.
#[allow(clippy::too_many_arguments)]
fn integrate_hadronic(
    args: &IntegrateArgs,
    parsed: &ParsedProcCard,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    rc: &RunCard,
    set: &PdfSet,
    pdf: &PdfMember,
    process: String,
) -> Result<RunOutput, IntegrateError> {
    let sqrt_s_had = rc.ebeam1 + rc.ebeam2;

    let sets = generate_from_proc_card_in(parsed, model, args.parallel.enumeration())
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
    // Both scales and the strong coupling come from the run card; the coupling is
    // the PDF set's own tabulation, which is what the densities were fitted with.
    let scale_report = integ
        .use_run_card_scales(model, evaluated, rc, Some(&set.info.alpha_s))
        .map_err(|e| err(format!("run card scale prescription: {e}")))?;

    tui::state::note_channels(integ.channel_ids().len());

    let n_survey = survey_points(args.neval);
    integ.adapt_alphas(args.seed, n_survey, ADAPT_ITERS, ADAPT_DAMPING);

    let (per_channel, result, convergence) = integ.adapt_grids_budget(
        args.budget()?,
        args.allocation(),
        args.seed,
        &tui::stop_signal(),
    );
    let channels = integ
        .channel_ids()
        .iter()
        .zip(integ.channel_samplers())
        .zip(&per_channel)
        .map(|((id, sampler), c)| {
            bank_channel(
                ChannelKey::GroupDiagram {
                    group: id.group,
                    diagram: id.diagram,
                },
                c,
                Some(sampler.clone()),
            )
        })
        .collect();
    Ok(RunOutput {
        process,
        pdf_set: args.pdf_set.clone(),
        mu_f: recorded_mu_f(&scale_report),
        sqrt_s: sqrt_s_had,
        channels,
        result,
        convergence,
    })
}

/// Fixed-energy partonic beams (`lpp = 0`): resonance-aware multichannel integration
/// with no PDF. The subprocess(es) and their external state are generated from the
/// caller's proc card, and a per-diagram [`MultiChannel`] combiner — α-adapted to the
/// process's own `Σ|M|²` — replaces flat RAMBO as the VEGAS integrand map so narrow
/// Breit–Wigner peaks converge (unbiased, same σ̂; flat RAMBO under-samples them).
fn integrate_fixed_energy(
    args: &IntegrateArgs,
    parsed: &ParsedProcCard,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    rc: &RunCard,
    process: String,
) -> Result<RunOutput, IntegrateError> {
    let sqrt_s = rc.ebeam1 + rc.ebeam2;

    let sets = generate_from_proc_card_in(parsed, model, args.parallel.enumeration())
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
    // The strong coupling follows the run card's per-event renormalisation scale.
    // Installed before the α-adaptation so the survey sees the same integrand the
    // integration will.
    integ
        .use_running_coupling(&diagrams, model, evaluated, rc)
        .map_err(|e| err(format!("run card scale prescription: {e}")))?;
    tui::state::note_channels(diagrams.len());
    let n_survey = survey_points(args.neval);
    integ.use_multichannel(&diagrams, evaluated, n_survey, ADAPT_ITERS, args.seed);

    let (per_channel, result, convergence) = integ.adapt_grids_budget(
        args.budget()?,
        args.allocation(),
        args.seed,
        &tui::stop_signal(),
    );
    Ok(RunOutput {
        process,
        pdf_set: NO_PDF.to_string(),
        // No parton distributions, so no factorisation scale enters the integral.
        mu_f: 0.0,
        sqrt_s,
        channels: per_channel
            .iter()
            .enumerate()
            .map(|(j, c)| {
                bank_channel(
                    ChannelKey::Diagram { diagram: j },
                    c,
                    integ.channel_samplers().get(j).cloned(),
                )
            })
            .collect(),
        result,
        convergence,
    })
}
