//! `vibegraph integrate` — compute a leading-order cross section for an
//! MG-validated process and persist the adapted VEGAS grid for a later sampling
//! phase.
//!
//! The process, model, beams, scales, and cuts are driven by the proc card
//! (`import model` + `generate`) and the run card, so the same card files drive
//! both this integration and a MadGraph reference run. Two beam modes are
//! supported, selected by the run card's `lpp1`/`lpp2`:
//!
//! * `lpp = 1` (proton beams) — the hadronic Drell–Yan `p p → e⁺ e⁻` process,
//!   PDF-convolved over a `(τ, y) × cosθ` map.
//! * `lpp = 0` (fixed-energy partonic beams) — an arbitrary process with no PDF
//!   convolution and a flat-RAMBO phase-space map over any final multiplicity.

use std::path::PathBuf;

use clap::Args;
use vibegraph::artifact::{IntegrateArtifact, FORMAT_VERSION};
use vibegraph::config::GlobalConfig;
use vibegraph::cuts::Cuts;
use vibegraph::diagrams::{
    generate_from_proc_card, parse_proc_card_file, ParsedProcCard, ParsingOptions,
};
use vibegraph::hadronic::{
    compile_class, compile_subprocesses, dy_flavor_classes, generate_dy_subprocesses,
    initial_spin_color_average, process_external_legs, DrellYanIntegrand, FixedBeamIntegrand,
    RunningCouplingReport,
};
use vibegraph::helas::eval::BoundAmplitude;
use vibegraph::pdf::PdfSet;
use vibegraph::phasespace::GEV2_TO_PB;
use vibegraph::runcard::{BeamMode, RunCard};
use vibegraph::ufo::{EvaluatedModel, UFOModel};
use vibegraph::vegas::{VegasGrid, VegasResult};

/// Default LHAPDF set — the one wired through the hadronic pipeline (MG5's LO
/// default `nn23lo1`, lhaid 247000).
const DEFAULT_PDF_SET: &str = "NNPDF23_lo_as_0130_qed";
/// PDF member index (central value; error members are not consumed at LO).
const PDF_MEMBER: u32 = 0;
/// Sentinel `pdf_set` recorded in the artifact for a no-PDF (fixed-energy) run.
const NO_PDF: &str = "none";
/// Artifact filename written inside the output directory.
const GRID_FILENAME: &str = "grid.bin.zst";

/// Points per survey iteration when α-adapting the fixed-energy multichannel
/// combiner, clamped to `[MIN, MAX]` around the integration budget: enough to
/// resolve each channel's variance share, capped so the one-off survey stays cheap.
const MIN_ADAPT_SURVEY: usize = 10_000;
const MAX_ADAPT_SURVEY: usize = 40_000;
/// Survey→refine iterations for the α-adaptation.
const ADAPT_ITERS: usize = 6;

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
    /// `validation/pdf` under the current directory.
    #[arg(long)]
    pub pdf_dir: Option<PathBuf>,

    /// VEGAS evaluations per adaptation iteration.
    #[arg(long, default_value_t = 120_000)]
    pub neval: usize,

    /// VEGAS adaptation iterations.
    #[arg(long, default_value_t = 12)]
    pub niter: usize,

    /// RNG seed for the integration.
    #[arg(long, default_value_t = 20_260_719)]
    pub seed: u64,
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

/// Resolve the directory holding `<pdf_set>/<pdf_set>.info`.
fn resolve_pdf_dir(args: &IntegrateArgs) -> PathBuf {
    if let Some(dir) = &args.pdf_dir {
        return dir.clone();
    }
    if let Some(env_dir) = std::env::var_os("VIBEGRAPH_PDF_DIR") {
        return PathBuf::from(env_dir);
    }
    PathBuf::from("validation/pdf")
}

/// The canonical string of the proc card's first process, for artifact metadata.
fn process_string(parsed: &ParsedProcCard) -> Result<String, IntegrateError> {
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
    grid: VegasGrid,
    result: VegasResult,
}

pub fn run(args: &IntegrateArgs) -> Result<(), IntegrateError> {
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
        ufo_search_path: PathBuf::from("."),
        restrict_path_override: None,
        run_card_path: args.run_card.clone(),
    };
    let model = config
        .load_ufo(&parsed.model)
        .map_err(|e| err(format!("failed to load model: {e}")))?;
    let rc = config
        .load_run_card()
        .map_err(|e| err(format!("failed to load run card: {e}")))?;

    let evaluated = EvaluatedModel::from_model(model.clone());

    let output = match rc.beam_mode() {
        BeamMode::Proton => integrate_proton(args, &parsed, &model, &evaluated, &rc, process)?,
        BeamMode::FixedEnergy => {
            integrate_fixed_energy(args, &parsed, &model, &evaluated, &rc, process)?
        }
    };

    let sigma_pb = output.result.integral * GEV2_TO_PB;
    let sigma_err_pb = output.result.std_dev * GEV2_TO_PB;

    println!("process:  {}", output.process);
    println!("PDF set:  {} (member {PDF_MEMBER})", output.pdf_set);
    println!("√s:       {} GeV,  μF = {} GeV", output.sqrt_s, output.mu_f);
    println!(
        "VEGAS:    {} evals × {} iters, seed {} (χ²/dof = {:.3})",
        args.neval, args.niter, args.seed, output.result.chi2_per_dof
    );
    println!("σ = {sigma_pb:.6} ± {sigma_err_pb:.6} pb");

    let artifact = IntegrateArtifact {
        format_version: FORMAT_VERSION,
        process: output.process,
        pdf_set: output.pdf_set,
        pdf_member: PDF_MEMBER,
        mu_f: output.mu_f,
        sqrt_s_had: output.sqrt_s,
        neval: args.neval,
        niter: args.niter,
        seed: args.seed,
        run_card: rc,
        grid: output.grid,
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
    println!("wrote {}", out_path.display());

    Ok(())
}

/// Hadronic Drell–Yan (`lpp = 1`): PDF-convolved `(τ, y) × cosθ` integration. The
/// subprocesses are generated from the caller's proc card and partitioned into
/// the up/down coupling classes; a non-Drell–Yan process is rejected there.
fn integrate_proton(
    args: &IntegrateArgs,
    _parsed: &ParsedProcCard,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    rc: &RunCard,
    process: String,
) -> Result<RunOutput, IntegrateError> {
    let sqrt_s_had = rc.ebeam1 + rc.ebeam2;

    // Locate and load the PDF set.
    let pdf_dir = resolve_pdf_dir(args);
    let set_dir = pdf_dir.join(&args.pdf_set);
    let set = PdfSet::load(&set_dir, &args.pdf_set).map_err(|e| {
        err(format!(
            "cannot load PDF set {} from {}: {e}\n\
             fetch it with `pixi run -e madgraph fetch-pdf` \
             or point --pdf-dir / $VIBEGRAPH_PDF_DIR at the data directory",
            args.pdf_set,
            set_dir.display()
        ))
    })?;
    let pdf = set
        .member(PDF_MEMBER)
        .map_err(|e| err(format!("cannot load PDF member {PDF_MEMBER}: {e}")))?;

    let sets =
        generate_dy_subprocesses(model).map_err(|e| err(format!("failed to enumerate: {e}")))?;
    let fc = dy_flavor_classes(sets, model).map_err(|e| {
        err(format!(
            "proton beams currently support only Drell–Yan `p p > e+ e-`: {e}"
        ))
    })?;
    let up = compile_class(&fc.up_set, model, evaluated)
        .map_err(|e| err(format!("failed to compile up-type class: {e}")))?;
    let down = compile_class(&fc.down_set, model, evaluated)
        .map_err(|e| err(format!("failed to compile down-type class: {e}")))?;
    let b_up = BoundAmplitude::<f64>::bind(&up, evaluated);
    let b_down = BoundAmplitude::<f64>::bind(&down, evaluated);

    let cuts = Cuts::compile(rc, &process_external_legs(&up, model, evaluated))
        .map_err(|e| err(format!("failed to compile cuts: {e}")))?;

    let spin_color_avg = initial_spin_color_average(&up, model, evaluated);
    let mut integ = DrellYanIntegrand::new(
        &b_up,
        &b_down,
        &pdf,
        &cuts,
        fc.up_flavors,
        fc.down_flavors,
        sqrt_s_had,
        rc.dsqrt_q2fact1,
        spin_color_avg,
    );
    // Both scales come from the run card: a fixed card leaves them constant, a
    // dynamic one has the parton distributions read per event and per beam.
    let scale_report = integ
        .use_run_card_scales(&fc.up_set.diagrams, model, evaluated, rc)
        .map_err(|e| err(format!("run card scale prescription: {e}")))?;

    let (grid, result) = integ.adapt_grid(args.neval, args.niter, args.seed);
    Ok(RunOutput {
        process,
        pdf_set: args.pdf_set.clone(),
        mu_f: recorded_mu_f(&scale_report),
        sqrt_s: sqrt_s_had,
        grid,
        result,
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
    // The strong coupling follows the run card's per-event renormalisation scale.
    // Installed before the α-adaptation so the survey sees the same integrand the
    // integration will.
    integ
        .use_running_coupling(&diagrams, model, evaluated, rc)
        .map_err(|e| err(format!("run card scale prescription: {e}")))?;
    let n_survey = args.neval.clamp(MIN_ADAPT_SURVEY, MAX_ADAPT_SURVEY);
    integ.use_multichannel(&diagrams, evaluated, n_survey, ADAPT_ITERS, args.seed);

    let (grid, result) = integ.adapt_grid(args.neval, args.niter, args.seed);
    Ok(RunOutput {
        process,
        pdf_set: NO_PDF.to_string(),
        // No parton distributions, so no factorisation scale enters the integral.
        mu_f: 0.0,
        sqrt_s,
        grid,
        result,
    })
}
