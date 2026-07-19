//! `vibegraph integrate` — compute the hadronic Drell–Yan cross section and
//! persist the adapted VEGAS grid for a later sampling phase.
//!
//! The process is fixed to `p p → e⁺ e⁻` this release (full proc-card option
//! coverage is a separate work item); the proc card supplies the `import model`
//! directive and is checked to describe that process. Beam energy and the fixed
//! factorization scale are read from the run card, so the same card file drives
//! both this integration and a MadGraph reference run.

use std::path::{Path, PathBuf};

use clap::Args;
use vibegraph::artifact::{IntegrateArtifact, FORMAT_VERSION};
use vibegraph::config::GlobalConfig;
use vibegraph::cuts::Cuts;
use vibegraph::diagrams::{parse_proc_card_file, ParsingOptions};
use vibegraph::hadronic::{compile_class, dy_external_legs, dy_flavor_classes, DrellYanIntegrand};
use vibegraph::helas::eval::BoundAmplitude;
use vibegraph::pdf::PdfSet;
use vibegraph::phasespace::GEV2_TO_PB;
use vibegraph::runcard::RunCard;
use vibegraph::ufo::EvaluatedModel;

/// Default LHAPDF set — the only one wired through the pipeline this release
/// (MG5's LO default `nn23lo1`, lhaid 247000).
const DEFAULT_PDF_SET: &str = "NNPDF23_lo_as_0130_qed";
/// PDF member index (central value; error members are not consumed at LO).
const PDF_MEMBER: u32 = 0;
/// Artifact filename written inside the output directory.
const GRID_FILENAME: &str = "grid.bin.zst";

#[derive(Args, Debug)]
pub struct IntegrateArgs {
    /// Process card selecting the model (the process is Drell–Yan `p p > e+ e-`).
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

    /// LHAPDF set name.
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

/// Verify the proc card describes the supported Drell–Yan process and return its
/// canonical string for the artifact metadata.
fn dy_process_string(proc_card: &Path) -> Result<String, IntegrateError> {
    let opts = ParsingOptions::default();
    let card = parse_proc_card_file(proc_card, &opts).map_err(|e| {
        err(format!(
            "failed to parse proc card {}: {e}",
            proc_card.display()
        ))
    })?;
    let spec = card
        .processes
        .first()
        .ok_or_else(|| err(format!("proc card {} has no process", proc_card.display())))?;

    let initial: Vec<String> = spec
        .initial
        .iter()
        .flat_map(|leg| std::iter::repeat(leg.name.to_lowercase()).take(leg.count.max(1)))
        .collect();
    let final_state: Vec<String> = spec
        .final_state
        .iter()
        .flat_map(|leg| std::iter::repeat(leg.name.to_lowercase()).take(leg.count.max(1)))
        .collect();

    let is_dy = initial == ["p", "p"] && final_state == ["e+", "e-"];
    if !is_dy {
        return Err(err(format!(
            "`integrate` supports only Drell–Yan `p p > e+ e-` this release; \
             proc card describes `{spec}`"
        )));
    }
    Ok(format!("{spec}"))
}

/// Factorization scale μF (GeV) from the run card. Only a fixed scale is
/// supported; a dynamical/running scale is rejected.
fn factorization_scale(rc: &RunCard) -> Result<f64, IntegrateError> {
    if !rc.fixed_fac_scale {
        return Err(err(
            "only a fixed factorization scale is supported (set `fixed_fac_scale = True`); \
             dynamical scale choices are not implemented",
        ));
    }
    Ok(rc.dsqrt_q2fact1)
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

    let process = dy_process_string(&args.proc_card)?;

    // Model resolution reuses the proc card's `import model` directive.
    let opts = ParsingOptions::default();
    let parsed = parse_proc_card_file(&args.proc_card, &opts)
        .map_err(|e| err(format!("failed to parse proc card: {e}")))?;
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
    let mu_f = factorization_scale(&rc)?;
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

    // Assemble the Drell–Yan integrand: two coupling-class amplitudes bound to
    // the model, the compiled cuts, and the PDF luminosity.
    let evaluated = EvaluatedModel::from_model(model.clone());
    let fc = dy_flavor_classes(&model)
        .map_err(|e| err(format!("failed to classify Drell–Yan flavors: {e}")))?;
    let up = compile_class(&fc.up_set, &model, &evaluated)
        .map_err(|e| err(format!("failed to compile up-type class: {e}")))?;
    let down = compile_class(&fc.down_set, &model, &evaluated)
        .map_err(|e| err(format!("failed to compile down-type class: {e}")))?;
    let b_up = BoundAmplitude::<f64>::bind(&up, &evaluated);
    let b_down = BoundAmplitude::<f64>::bind(&down, &evaluated);

    let cuts = Cuts::compile(&rc, &dy_external_legs(2))
        .map_err(|e| err(format!("failed to compile cuts: {e}")))?;

    let integ = DrellYanIntegrand::new(
        &b_up,
        &b_down,
        &pdf,
        &cuts,
        fc.up_flavors,
        fc.down_flavors,
        sqrt_s_had,
        mu_f,
    );

    let (grid, result) = integ.adapt_grid(args.neval, args.niter, args.seed);
    let sigma_pb = result.integral * GEV2_TO_PB;
    let sigma_err_pb = result.std_dev * GEV2_TO_PB;

    println!("process:  {process}");
    println!("PDF set:  {} (member {PDF_MEMBER})", args.pdf_set);
    println!("√s:       {sqrt_s_had} GeV,  μF = {mu_f} GeV");
    println!(
        "VEGAS:    {} evals × {} iters, seed {} (χ²/dof = {:.3})",
        args.neval, args.niter, args.seed, result.chi2_per_dof
    );
    println!("σ = {sigma_pb:.6} ± {sigma_err_pb:.6} pb");

    let artifact = IntegrateArtifact {
        format_version: FORMAT_VERSION,
        process,
        pdf_set: args.pdf_set.clone(),
        pdf_member: PDF_MEMBER,
        mu_f,
        sqrt_s_had,
        neval: args.neval,
        niter: args.niter,
        seed: args.seed,
        run_card: rc,
        grid,
        sigma_pb,
        sigma_err_pb,
        chi2_per_dof: result.chi2_per_dof,
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
