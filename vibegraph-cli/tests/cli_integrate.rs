//! End-to-end test of `vibegraph integrate` on the Drell–Yan pp→e⁺e⁻ proc card
//! + reference run cards: a cold-start run must reproduce the banked MadGraph
//! reference σ (the H7 gate), and the persisted grid must reload and drive a
//! frozen sampling pass that reproduces the adapted estimate.
//!
//! Gated behind `extended-validation`; needs the fetched PDF set and the banked
//! reference JSON:
//!
//!     pixi run -e madgraph fetch-pdf
//!     pixi run -e madgraph generate-hadronic-sigma
//!     cargo test -p vibegraph --features extended-validation --test cli_integrate

use std::path::{Path, PathBuf};
use std::process::Command;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use vibegraph::artifact::IntegrateArtifact;
use vibegraph::cuts::Cuts;
use vibegraph::hadronic::{
    compile_class, dy_external_legs, dy_flavor_classes, generate_dy_subprocesses,
    initial_spin_color_average, DrellYanIntegrand,
};
use vibegraph::helas::eval::BoundAmplitude;
use vibegraph::pdf::{PdfMember, PdfSet};
use vibegraph::phasespace::GEV2_TO_PB;
use vibegraph::runcard::RunCard;
use vibegraph::ufo::sm::{sm_model, SMRestrict};
use vibegraph::ufo::EvaluatedModel;

const PDF_SET: &str = "NNPDF23_lo_as_0130_qed";

fn validation_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph")
}

fn pdf_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/pdf")
}

fn load_pdf() -> PdfMember {
    let dir = pdf_dir().join(PDF_SET);
    let set = PdfSet::load(&dir, PDF_SET).unwrap_or_else(|e| {
        panic!(
            "cannot load PDF set {PDF_SET} from {}: {e}\n run `pixi run -e madgraph fetch-pdf`",
            dir.display()
        )
    });
    set.member(0).expect("PDF member 0")
}

/// Banked MG σ ± Δσ for one run, or `None` if the reference JSON is absent.
fn banked(run: &str) -> Option<(f64, f64)> {
    let path = validation_dir().join("hadronic_sigma_reference.json");
    let text = std::fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    let r = v.get(run)?;
    Some((
        r.get("sigma_pb")?.as_f64()?,
        r.get("sigma_err_pb")?.as_f64()?,
    ))
}

fn run_cli(out_dir: &Path, run_card: &str) -> IntegrateArtifact {
    let status = Command::new(env!("CARGO_BIN_EXE_vibegraph"))
        .arg("integrate")
        .arg(validation_dir().join("dy13_proc_card.dat"))
        .arg("--run-card")
        .arg(validation_dir().join(run_card))
        .arg("--pdf-dir")
        .arg(pdf_dir())
        .arg("--out")
        .arg(out_dir)
        .status()
        .expect("spawn vibegraph");
    assert!(status.success(), "vibegraph integrate exited non-zero");
    IntegrateArtifact::read_from_path(&out_dir.join("grid.bin.zst")).expect("reload artifact")
}

fn check_run(test: &str, run: &str, run_card: &str) {
    let Some((mg, mg_err)) = banked(run) else {
        return vibegraph::validation::skip(test, "banked hadronic sigma reference", run);
    };

    let tmp = tempfile::tempdir().unwrap();
    let artifact = run_cli(tmp.path(), run_card);

    // (1) Cold-start σ from the artifact reproduces the banked MadGraph σ.
    let combined = (artifact.sigma_err_pb.powi(2) + mg_err * mg_err).sqrt();
    let delta = (artifact.sigma_pb - mg).abs();
    let rel = delta / mg;
    eprintln!(
        "[{run}] CLI σ = {:.3} ± {:.3} pb | MG σ = {mg:.3} ± {mg_err:.3} pb | \
         Δ = {delta:.3} pb ({:.1}σ), rel = {rel:.4}",
        artifact.sigma_pb,
        artifact.sigma_err_pb,
        delta / combined
    );
    assert!(
        delta < 3.0 * combined || rel < 0.01,
        "[{run}] CLI σ {:.3} disagrees with MG {mg:.3} pb ({:.1}σ)",
        artifact.sigma_pb,
        delta / combined
    );

    // (2) Metadata round-trips through the artifact.
    assert_eq!(artifact.process, "p p > e+ e-");
    assert_eq!(artifact.pdf_set, PDF_SET);
    assert_eq!(artifact.sqrt_s_had, 13000.0);
    assert_eq!(artifact.mu_f, 91.1880);

    // (3) The reloaded grid drives a frozen sampling pass that reproduces the
    // adapted estimate within the single-pass MC error (the distributed-phase
    // primitive against the persisted grid).
    let rc = RunCard::parse_file(&validation_dir().join(run_card)).unwrap();
    let model = sm_model(SMRestrict::Default);
    let evaluated = EvaluatedModel::from_model(model.clone());
    let fc = dy_flavor_classes(generate_dy_subprocesses(&model).unwrap(), &model).unwrap();
    let up = compile_class(&fc.up_set, &model, &evaluated).unwrap();
    let down = compile_class(&fc.down_set, &model, &evaluated).unwrap();
    let b_up = BoundAmplitude::<f64>::bind(&up, &evaluated);
    let b_down = BoundAmplitude::<f64>::bind(&down, &evaluated);
    let cuts = Cuts::compile(&rc, &dy_external_legs(2)).unwrap();
    let pdf = load_pdf();
    let spin_color_avg = initial_spin_color_average(&up, &model, &evaluated);
    let integ = DrellYanIntegrand::new(
        &b_up,
        &b_down,
        &pdf,
        &cuts,
        fc.up_flavors,
        fc.down_flavors,
        artifact.sqrt_s_had,
        artifact.mu_f,
        spin_color_avg,
    );

    let mut rng = ChaCha8Rng::seed_from_u64(0xF202E0);
    // The Drell–Yan map is not split across channels, so the artifact banks one
    // grid and the frozen pass replays it directly.
    let grid = artifact
        .sole_grid()
        .unwrap_or_else(|| panic!("[{run}] Drell–Yan artifact banked more than one grid"));
    let frozen = grid.sample_frozen(|u| integ.value(u), 200_000, &mut rng);
    let sigma_frozen = frozen.integral * GEV2_TO_PB;
    let err_frozen = frozen.std_dev * GEV2_TO_PB;
    let d = (sigma_frozen - artifact.sigma_pb).abs();
    let comb = (err_frozen * err_frozen + artifact.sigma_err_pb.powi(2)).sqrt();
    eprintln!(
        "[{run}] frozen-grid σ = {sigma_frozen:.3} ± {err_frozen:.3} pb vs adapted \
         {:.3} pb (Δ = {d:.3} pb, {:.1}σ)",
        artifact.sigma_pb,
        d / comb
    );
    assert!(
        d < 4.0 * comb,
        "[{run}] frozen-grid σ {sigma_frozen:.3} disagrees with adapted {:.3} pb ({:.1}σ)",
        artifact.sigma_pb,
        d / comb
    );
}

#[test]
fn integrate_default_cuts_reproduces_h7_sigma() {
    check_run(
        "integrate_default_cuts_reproduces_h7_sigma",
        "default",
        "dy13_default_run_card.dat",
    );
}

#[test]
fn integrate_mmll_window_reproduces_h7_sigma() {
    check_run(
        "integrate_mmll_window_reproduces_h7_sigma",
        "mmll_60_120",
        "dy13_mmll_run_card.dat",
    );
}

/// The artifact refuses to overwrite an existing grid unless `--force` is given.
#[test]
fn integrate_refuses_overwrite_without_force() {
    let tmp = tempfile::tempdir().unwrap();
    let proc_card = validation_dir().join("dy13_proc_card.dat");
    let run_card = validation_dir().join("dy13_default_run_card.dat");

    let mut base = Command::new(env!("CARGO_BIN_EXE_vibegraph"));
    base.arg("integrate")
        .arg(&proc_card)
        .arg("--run-card")
        .arg(&run_card)
        .arg("--pdf-dir")
        .arg(pdf_dir())
        .arg("--out")
        .arg(tmp.path())
        // Keep it cheap: a single tiny iteration is enough to write the artifact.
        .arg("--neval")
        .arg("200")
        .arg("--niter")
        .arg("1");

    assert!(base.status().expect("spawn").success(), "first run");

    // Second run without --force must fail (and not clobber).
    let refuse = base.status().expect("spawn");
    assert!(!refuse.success(), "overwrite without --force must fail");

    // With --force it succeeds.
    let forced = base.arg("--force").status().expect("spawn");
    assert!(forced.success(), "forced overwrite must succeed");
}
