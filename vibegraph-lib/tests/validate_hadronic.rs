//! Extended validation: hadronic LO cross sections against banked MadGraph5
//! reference runs, assembled from the PDF luminosity, the compiled amplitude, the
//! run-card cuts, and VEGAS, and driven by the *same* run-card files, PDF set
//! (NNPDF23_lo_as_0130_qed / lhaid 247000) and fixed scale μF = m_Z MadGraph was
//! given.
//!
//! Two σ(pp → e⁺e⁻) reference runs are enforced through the bespoke Drell–Yan
//! integrand ([`vibegraph::hadronic`]): default lepton cuts, and the
//! m_ll ∈ [60,120] window. Both must agree within combined Monte-Carlo error
//! (target < 1%).
//!
//! σ(pp → ℓ⁺ℓ⁻ j) is enforced through the **general** hadronic path
//! ([`vibegraph::proton`]) against the banked `pp_to_llj_fixed` run — the first
//! cross section here for a coloured initial state, a three-body final state, a jet
//! cut and a strong coupling. It is measured over five seeds, because VEGAS's
//! `1/σ²` iteration combination reports an under-sampled region confidently.
//!
//! The Drell–Yan default run is *also* taken through the general path as an
//! informational row, so the two treatments of the mirrored beam ordering keep
//! meeting on a process both paths can do. The enforced Drell–Yan rows are the
//! bespoke integrand's and stay there.
//!
//! A pointwise integrand oracle pins the PDF × flux × |M|² factors at fixed
//! `(x₁, x₂, cosθ)` points (including points just inside/outside a cut boundary)
//! against an independent Python computation (`validation/madgraph/gen_dy_oracle.py`).
//!
//! Gated behind `extended-validation`; the σ tests need the fetched PDF set, the
//! banked reference JSON and the banked `pp_to_llj_fixed` run:
//!
//!     pixi run -e madgraph validate-hadronic
//!
//! Run it under `--profile profiling` if invoking cargo directly: the five-seed llj
//! sweep is minutes optimised and hours unoptimised.

mod common;

use std::path::{Path, PathBuf};

use vibegraph::cuts::Cuts;
use vibegraph::hadronic::{
    compile_class, dy_external_legs, dy_flavor_classes, generate_dy_subprocesses,
    initial_spin_color_average, DrellYanIntegrand,
};
use vibegraph::helas::eval::BoundAmplitude;
use vibegraph::pdf::{PdfMember, PdfSet};
use vibegraph::runcard::RunCard;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::EvaluatedModel;

const MU_F: f64 = 91.1880;
const SQRT_S_HAD: f64 = 13000.0;
const PDF_SET: &str = "NNPDF23_lo_as_0130_qed";

/// The process of the banked `pp_to_llj_fixed` run, spelled as its `.mg5` script
/// spells it — the coupling orders included, since they set the diagram content.
const LLJ_PROCESS: &str = "p p > l+ l- j QCD=2 QED=2";

/// Independent seeds the ℓℓj cross section is measured on. Five runs rather than
/// one because VEGAS's `1/σ²` iteration combination reports an under-sampled
/// region as a confident number, which one seed cannot distinguish from a
/// converged one.
const LLJ_SEEDS: &[u64] = &[20260730, 20260731, 20260732, 20260733, 20260734];
/// Points per survey iteration, and iterations, of the channel-weight adaptation.
const LLJ_ADAPT_SURVEY: usize = 8_000;
const LLJ_ADAPT_ITERS: usize = 5;
/// VEGAS budget per seed, chosen from a measured budget scan and not from cost
/// alone. The estimator approaches its limit **from below** — the unadapted early
/// iterations enter the `1/σ²` combination with underestimated variances — so the
/// five-seed mean rises with `neval` per iteration: `418.5` at 60 000, `421.7` at
/// 150 000, `422.9` here, `423.5` at 600 000, each step about half the last. Below
/// 150 000 the residual exceeds MadGraph's own error and the sweep would be
/// measuring this crate's convergence rather than an agreement.
///
/// The per-channel allocation floors at 512 points a channel, so the 24 pooled
/// `(group, diagram)` channels spend at least 12 288 evaluations an iteration
/// whatever this says.
const LLJ_NEVAL: usize = 300_000;
const LLJ_NITER: usize = 10;
/// Largest relative distance from the banked MadGraph σ the sweep may show.
///
/// Above MadGraph's own `0.43%` Monte-Carlo error, which is the floor: no
/// agreement tighter than the reference's precision is meaningful. Below the
/// `1.0%` an under-converged budget produces, which is what it exists to catch.
/// The whole measured budget family — `0.28%`, `0.00%`, `0.16%` at 150 000,
/// 300 000 and 600 000 — sits inside it, so it is not a bound around one number.
const LLJ_MAX_REL: f64 = 0.005;
/// Scatter the five estimates are allowed about their own mean, in units of their
/// quoted errors. Measured over the same budget family: `1.55`, `0.47`, `1.90`,
/// `0.37`.
const LLJ_MAX_CHI2_PER_DOF: f64 = 4.0;

fn validation_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph")
}

fn load_pdf_set() -> PdfSet {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/pdf")
        .join(PDF_SET);
    PdfSet::load(&dir, PDF_SET).unwrap_or_else(|e| {
        panic!(
            "cannot load PDF set {PDF_SET} from {}: {e}\n\
             run `pixi run -e madgraph fetch-pdf`",
            dir.display()
        )
    })
}

fn load_pdf() -> PdfMember {
    load_pdf_set().member(0).expect("PDF member 0")
}

/// Run the full VEGAS integration for a given run card, returning (σ, Δσ) in pb.
fn run_sigma(run_card_path: &Path, neval: usize, niter: usize, seed: u64) -> (f64, f64) {
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let fc = dy_flavor_classes(
        generate_dy_subprocesses(&model).expect("generate DY"),
        &model,
    )
    .expect("classify DY");
    let up = compile_class(&fc.up_set, &model, &evaluated).expect("up class");
    let down = compile_class(&fc.down_set, &model, &evaluated).expect("down class");
    let b_up = BoundAmplitude::<f64>::bind(&up, &evaluated);
    let b_down = BoundAmplitude::<f64>::bind(&down, &evaluated);

    let rc = RunCard::parse_file(run_card_path).expect("parse run card");
    let cuts = Cuts::compile(&rc, &dy_external_legs(2)).expect("compile cuts");
    let pdf = load_pdf();

    let spin_color_avg = initial_spin_color_average(&up, &model, &evaluated);
    let mut integ = DrellYanIntegrand::new(
        &b_up,
        &b_down,
        &pdf,
        &cuts,
        fc.up_flavors,
        fc.down_flavors,
        SQRT_S_HAD,
        MU_F,
        spin_color_avg,
    );
    // Take both scales from the run card rather than from MU_F. Both reference
    // cards fix them at m_Z, so the prescription resolves to a constant and the
    // parton distributions are read exactly where they were before — the cross
    // section must not move. A card that freed either scale would change these
    // numbers, which is why the constancy is asserted here and not assumed.
    let report = integ
        .use_run_card_scales(&fc.up_set.diagrams, &model, &evaluated, &rc)
        .expect("run card scale prescription compiles");
    let constant = report
        .constant_scales
        .unwrap_or_else(|| panic!("reference run card no longer fixes both scales: {report:?}"));
    assert_eq!(
        (constant.mu_r, constant.mu_f),
        (MU_F, [MU_F, MU_F]),
        "reference run card no longer fixes both scales at m_Z"
    );
    assert!(
        !report.depends_on_alpha_s,
        "Drell-Yan at this order carries no strong coupling; a matrix element that \
         did would need one, and `pdlabel = lhapdf` refuses to supply it"
    );
    integ.integrate(neval, niter, seed)
}

/// MadGraph's combined `(σ, Δσ)` in pb for a banked `madevent` run: fields 1 and
/// 2 of `SubProcesses/results.dat`, the same pair `gen_hadronic_sigma.sh` banks
/// into the Drell–Yan reference JSON. Read from the run rather than copied into
/// a committed file, so the number cannot drift from the run it came out of.
fn banked_llj_sigma(run_dir: &Path) -> (f64, f64) {
    let path = run_dir.join("SubProcesses/results.dat");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "cannot read {}: {e}\n\
             bank the run with `pixi run -e madgraph build-diagrams`",
            path.display()
        )
    });
    let mut fields = text.split_whitespace();
    let parse = |f: Option<&str>| -> f64 {
        f.and_then(|s| s.replace(['E', 'D'], "e").parse::<f64>().ok())
            .unwrap_or_else(|| panic!("cannot parse a cross section from {}", path.display()))
    };
    (parse(fields.next()), parse(fields.next()))
}

/// Banked MG σ ± Δσ for one run, or `None` when the reference JSON is absent.
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

fn check_run(run: &str, card: &str) {
    let card_path = validation_dir().join(card);
    let (sigma, err) = run_sigma(&card_path, 120_000, 12, 20260719);
    match banked(run) {
        Some((mg, mg_err)) => {
            let combined = (err * err + mg_err * mg_err).sqrt();
            let delta = (sigma - mg).abs();
            let rel = delta / mg;
            eprintln!(
                "[{run}] vibegraph σ = {sigma:.3} ± {err:.3} pb | \
                 MG σ = {mg:.3} ± {mg_err:.3} pb | Δ = {delta:.3} pb \
                 ({} combined σ), rel = {rel:.4}",
                delta / combined
            );
            assert!(
                delta < 3.0 * combined || rel < 0.01,
                "[{run}] σ disagreement: vibegraph {sigma:.3}±{err:.3} vs MG {mg:.3}±{mg_err:.3} pb, \
                 Δ = {delta:.3} pb = {:.1}σ, rel = {rel:.4}",
                delta / combined
            );
        }
        None => {
            // Known-wrong informational mode until the MG reference is banked.
            eprintln!(
                "[{run}] INFO (no banked MG reference yet): vibegraph σ = {sigma:.3} ± {err:.3} pb"
            );
        }
    }
}

#[test]
fn sigma_default_cuts_vs_mg() {
    check_run("default", "dy13_default_run_card.dat");
}

#[test]
fn sigma_mmll_window_vs_mg() {
    check_run("mmll_60_120", "dy13_mmll_run_card.dat");
}

/// Drell–Yan through the **general** hadronic path — the flavour-group
/// decomposition convolved over `(τ, y)` with a per-diagram multichannel inner
/// map — against the same banked MadGraph reference the bespoke integrand is
/// gated on.
///
/// **Informational.** The enforced Drell–Yan rows above are the pipeline's
/// bit-reproducibility anchor and stay exactly where they are; this row exists so
/// that a general path under construction has a known-good end-to-end comparison
/// running against a process it can already do. Whether the bespoke path is later
/// retired in its favour is a separate decision.
///
/// What it proves: the whole assembled chain — the `(τ, y)` map and its Jacobian,
/// the `x·f` luminosity, the flux and `2π` measure, the cut filter in the lab
/// frame, the flavour partition and both beam orderings — reproduces a measured
/// hadronic cross section, on real parton distributions and a real run card.
///
/// What it cannot see: anything specific to a coloured initial state or a
/// three-body final state. Drell–Yan has no gluon-initiated group, no peripheral
/// channel, no strong coupling and no jet cut, so the spacelike floor, the grid
/// `αs` and the three-body spine are all untouched by it.
#[test]
fn sigma_default_cuts_through_the_general_path() {
    use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use vibegraph::proton::{derive_flavor_groups, ProtonIntegrand};

    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let rc = RunCard::parse_file(&validation_dir().join("dy13_default_run_card.dat"))
        .expect("parse run card");

    let opts = ParsingOptions::default();
    let proc_card = parse_proc_card("generate p p > e+ e-", &opts).expect("proc card");
    let sets = generate_from_proc_card(&proc_card, &model).expect("enumeration");
    let groups = derive_flavor_groups(sets, &model, &evaluated, &rc).expect("flavour groups");

    let set = load_pdf_set();
    let pdf = set.member(0).expect("PDF member 0");
    let amps: Vec<BoundAmplitude<f64>> = groups
        .groups()
        .iter()
        .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), &evaluated))
        .collect();
    let mut integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
        .expect("hadronic integrand");
    let report = integ
        .use_run_card_scales(&model, &evaluated, &rc, Some(&set.info.alpha_s))
        .expect("run card scale prescription compiles");
    let constant = report
        .constant_scales
        .unwrap_or_else(|| panic!("reference run card no longer fixes both scales: {report:?}"));
    assert_eq!(
        (constant.mu_r, constant.mu_f),
        (MU_F, [MU_F, MU_F]),
        "reference run card no longer fixes both scales at m_Z"
    );
    assert!(
        !report.depends_on_alpha_s,
        "Drell-Yan at this order carries no strong coupling"
    );

    integ.adapt_alphas(20260730, 8_000, 5, 0.5);
    let (sigma, err) = integ.integrate(40_000, 10, 20260719);

    let (mg, mg_err) = banked("default").expect("banked MG reference");
    let combined = (err * err + mg_err * mg_err).sqrt();
    let delta = sigma - mg;
    let rel = delta.abs() / mg;
    eprintln!(
        "[default/general-path] INFO vibegraph σ = {sigma:.3} ± {err:.3} pb | \
         MG σ = {mg:.3} ± {mg_err:.3} pb | Δ = {delta:.3} pb ({:.1} combined σ), \
         rel = {rel:.4}",
        delta / combined
    );
    assert!(
        rel < 0.02,
        "[default/general-path] σ disagreement: vibegraph {sigma:.3}±{err:.3} vs \
         MG {mg:.3}±{mg_err:.3} pb, rel = {rel:.4}"
    );
}

/// Pointwise integrand oracle: at ~10 pinned `(x₁, x₂, cosθ)` points
/// (including two straddling the pT_ℓ = 10 GeV cut boundary), every factor
/// of vibegraph's integrand — PDF luminosity, |M|², flux prefactor, the
/// (τ,y) Jacobian, the cut indicator, and their product — must match the
/// independent Python oracle (LHAPDF `xfxQ2` × MadGraph standalone |M|²)
/// to ≤ 1e-9 relative. Regenerate with `pixi run -e madgraph
/// generate-dy-oracle`.
#[test]
fn pointwise_integrand_oracle() {
    let oracle_path = validation_dir().join("dy_integrand_oracle.json");
    let text = std::fs::read_to_string(&oracle_path).unwrap_or_else(|e| {
        panic!(
            "missing {}: {e}\n run `pixi run -e madgraph generate-dy-oracle`",
            oracle_path.display()
        )
    });
    let oracle: serde_json::Value = serde_json::from_str(&text).unwrap();
    let points = oracle["points"].as_array().expect("points array");

    // Bind vibegraph with MadGraph's exact param card (committed alongside the
    // oracle) so the |M|² comparison is at rounding level, not the ~1e-3 param
    // floor.
    let model = common::sm_model();
    let card_path = validation_dir().join(
        oracle["param_card"]
            .as_str()
            .unwrap_or("dy13_param_card.dat"),
    );
    let card = std::fs::read_to_string(&card_path)
        .ok()
        .and_then(|s| s.parse::<ParamCard>().ok())
        .expect("parse committed param card");
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &card);

    let fc = dy_flavor_classes(
        generate_dy_subprocesses(&model).expect("generate DY"),
        &model,
    )
    .expect("classify DY");
    let up = compile_class(&fc.up_set, &model, &evaluated).expect("up class");
    let down = compile_class(&fc.down_set, &model, &evaluated).expect("down class");
    let b_up = BoundAmplitude::<f64>::bind(&up, &evaluated);
    let b_down = BoundAmplitude::<f64>::bind(&down, &evaluated);

    let rc = RunCard::parse_file(&validation_dir().join("dy13_default_run_card.dat")).unwrap();
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
        SQRT_S_HAD,
        MU_F,
        spin_color_avg,
    );

    const TOL: f64 = 1e-9;
    // A relative comparison with a small absolute floor for near-zero factors
    // (e.g. the integrand value at the far tail, ~1e-9 GeV⁻²).
    let rel = |a: f64, b: f64| (a - b).abs() / b.abs().max(1e-30);

    let mut worst = 0.0f64;
    for (i, p) in points.iter().enumerate() {
        let u: Vec<f64> = p["u"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        let f = integ.debug_factors(&u);
        let g = |k: &str| p[k].as_f64().unwrap();

        assert_eq!(
            f.pass,
            p["pass"].as_bool().unwrap(),
            "cut indicator, point {i}"
        );
        for (name, got, want) in [
            ("x1", f.x1, g("x1")),
            ("x2", f.x2, g("x2")),
            ("sqrt_shat", f.sqrt_shat, g("sqrt_shat")),
            ("lum_up", f.lum_up, g("lum_up")),
            ("lum_down", f.lum_down, g("lum_down")),
            ("m2_up", f.m2_up, g("m2_up")),
            ("m2_down", f.m2_down, g("m2_down")),
            ("phat", f.phat, g("phat")),
            ("jac", f.jac, g("jac")),
            ("value", f.value, g("value")),
        ] {
            let r = rel(got, want);
            worst = worst.max(r);
            assert!(
                r <= TOL,
                "point {i} factor '{name}': vibegraph {got:.12e} vs oracle {want:.12e}, \
                 rel = {r:.2e} > {TOL:.0e}"
            );
        }
    }
    eprintln!(
        "[pointwise oracle] {} points, worst rel = {worst:.2e}",
        points.len()
    );
}

/// σ(p p → ℓ⁺ℓ⁻ j) at a fixed scale, through the general hadronic path, against
/// the banked `pp_to_llj_fixed` MadGraph run.
///
/// This is the first cross section this crate computes for a process with a
/// coloured initial state, a three-body final state, a jet cut and a strong
/// coupling — everything the Drell–Yan rows above are blind to. The comparison
/// is against MadGraph's own number for the same cards: the same proc card
/// content, the same run card file, the same PDF set and the same fixed scales.
///
/// **Five seeds, not one.** VEGAS combines its iterations by `1/σ²`, so a run
/// that under-samples a region reports a confidently wrong integral with a small
/// error rather than a large one, and a single seed agreeing is then not
/// evidence. Five independent runs are compared individually and through their
/// inverse-variance mean, and it is the mean the gate is on.
///
/// What it cannot see: anything the cross section integrates over. A per-diagram
/// phase, a colour-flow relabelling and a helicity-by-helicity error all leave
/// `Σ|M|²` and hence σ alone — those are pinned at the amplitude level by
/// `validate_helas_mg` and `amp_diagram_oracle`. It also cannot separate the
/// phase-space map from the matrix element: a map whose weight and density were
/// both wrong by one factor would integrate correctly.
#[test]
fn sigma_llj_fixed_scale_vs_mg() {
    use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use vibegraph::proton::{derive_flavor_groups, ProtonIntegrand};

    let run_dir = validation_dir().join("output/pp_to_llj_fixed");
    let rc = RunCard::parse_file(&run_dir.join("Cards/run_card.dat")).unwrap_or_else(|e| {
        panic!(
            "cannot read the banked run card at {}: {e}\n\
             bank the run with `pixi run -e madgraph build-diagrams`",
            run_dir.display()
        )
    });
    let (mg, mg_err) = banked_llj_sigma(&run_dir);

    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let opts = ParsingOptions::default();
    let proc_card = parse_proc_card(&format!("generate {LLJ_PROCESS}"), &opts).expect("proc card");
    let sets = generate_from_proc_card(&proc_card, &model).expect("enumeration");
    let groups = derive_flavor_groups(sets, &model, &evaluated, &rc).expect("flavour groups");

    let set = load_pdf_set();
    let pdf = set.member(0).expect("PDF member 0");
    let amps: Vec<BoundAmplitude<f64>> = groups
        .groups()
        .iter()
        .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), &evaluated))
        .collect();

    let mut runs: Vec<(u64, f64, f64)> = Vec::new();
    for &seed in LLJ_SEEDS {
        let mut integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
            .expect("hadronic integrand");
        let report = integ
            .use_run_card_scales(&model, &evaluated, &rc, Some(&set.info.alpha_s))
            .expect("run card scale prescription compiles");
        let constant = report.constant_scales.unwrap_or_else(|| {
            panic!("the banked llj run card no longer fixes both scales: {report:?}")
        });
        assert_eq!(
            (constant.mu_r, constant.mu_f),
            (MU_F, [MU_F, MU_F]),
            "the banked llj run card no longer fixes both scales at m_Z"
        );
        assert!(
            report.depends_on_alpha_s,
            "a QCD ℓℓj matrix element must carry the strong coupling; one that did \
             not would be missing its gluon vertex"
        );

        integ.adapt_alphas(seed, LLJ_ADAPT_SURVEY, LLJ_ADAPT_ITERS, 0.5);
        let (sigma, err) = integ.integrate(LLJ_NEVAL, LLJ_NITER, seed);
        eprintln!(
            "[llj_fixed seed {seed}] vibegraph σ = {sigma:.3} ± {err:.3} pb | \
             rel = {:+.4} | pull = {:+.2}",
            (sigma - mg) / mg,
            (sigma - mg) / (err * err + mg_err * mg_err).sqrt()
        );
        runs.push((seed, sigma, err));
    }

    // Inverse-variance mean of the independent runs, the estimator with the
    // seed-to-seed scatter averaged out.
    let inv_var: f64 = runs.iter().map(|(_, _, e)| 1.0 / (e * e)).sum();
    let mean: f64 = runs.iter().map(|(_, s, e)| s / (e * e)).sum::<f64>() / inv_var;
    let mean_err = inv_var.sqrt().recip();
    // Scatter of the five estimates about their mean, in units of their own
    // quoted errors: a run that missed a region reports a small error, so the
    // scatter and not the error is what shows it.
    let chi2: f64 = runs
        .iter()
        .map(|(_, s, e)| ((s - mean) / e).powi(2))
        .sum::<f64>()
        / (runs.len() - 1) as f64;

    let combined = (mean_err * mean_err + mg_err * mg_err).sqrt();
    let delta = mean - mg;
    let rel = delta.abs() / mg;
    eprintln!(
        "[llj_fixed] vibegraph σ = {mean:.3} ± {mean_err:.3} pb ({} seeds, \
         χ²/dof = {chi2:.2}) | MG σ = {mg:.3} ± {mg_err:.3} pb | \
         Δ = {delta:.3} pb ({:.2} combined σ), rel = {rel:.4}",
        runs.len(),
        delta / combined
    );

    assert!(
        delta.abs() < 3.0 * combined,
        "[llj_fixed] σ disagreement: vibegraph {mean:.3}±{mean_err:.3} vs \
         MG {mg:.3}±{mg_err:.3} pb, Δ = {delta:.3} pb = {:.1}σ",
        delta / combined
    );
    assert!(
        rel < LLJ_MAX_REL,
        "[llj_fixed] σ disagreement: vibegraph {mean:.3}±{mean_err:.3} vs \
         MG {mg:.3}±{mg_err:.3} pb, rel = {rel:.4} > {LLJ_MAX_REL}"
    );
    assert!(
        chi2 < LLJ_MAX_CHI2_PER_DOF,
        "[llj_fixed] the five seeds scatter by more than they claim: \
         χ²/dof = {chi2:.2} over {runs:?}"
    );
}

/// Emit the informational dσ/dm_ℓℓ comparison table (committed, not gated):
/// vibegraph's Drell–Yan mass spectrum under default cuts, with the two
/// banked MadGraph σ values as integral anchors (full range and the
/// [60,120] window). Run explicitly to regenerate the committed artifact:
///
///   cargo test -p vibegraph-lib --features extended-validation \
///     --test validate_hadronic emit_dsigma_dmll -- --ignored --nocapture
#[test]
#[ignore = "writes the committed dσ/dm_ll artifact; run manually"]
fn emit_dsigma_dmll_table() {
    use std::fmt::Write as _;

    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let fc = dy_flavor_classes(generate_dy_subprocesses(&model).unwrap(), &model).unwrap();
    let up = compile_class(&fc.up_set, &model, &evaluated).unwrap();
    let down = compile_class(&fc.down_set, &model, &evaluated).unwrap();
    let b_up = BoundAmplitude::<f64>::bind(&up, &evaluated);
    let b_down = BoundAmplitude::<f64>::bind(&down, &evaluated);
    let rc = RunCard::parse_file(&validation_dir().join("dy13_default_run_card.dat")).unwrap();
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
        SQRT_S_HAD,
        MU_F,
        spin_color_avg,
    );

    let (m_lo, m_hi, nbins) = (20.0_f64, 200.0_f64, 36);
    let bin_w = (m_hi - m_lo) / nbins as f64;
    let dens = integ.dsigma_dmll(m_lo, m_hi, nbins, 8_000_000, 424242);

    let (mg_default, _) = banked("default").unwrap_or((f64::NAN, 0.0));
    let (mg_window, _) = banked("mmll_60_120").unwrap_or((f64::NAN, 0.0));
    let sig_window: f64 = dens
        .iter()
        .enumerate()
        .filter(|(i, _)| {
            let lo = m_lo + *i as f64 * bin_w;
            lo >= 60.0 && lo < 120.0
        })
        .map(|(_, d)| d * bin_w)
        .sum();
    let sig_20_200: f64 = dens.iter().map(|d| d * bin_w).sum();

    let mut out = String::new();
    writeln!(
        out,
        "# Drell–Yan dσ/dm_ℓℓ — vibegraph vs MadGraph (informational)\n"
    )
    .unwrap();
    writeln!(
        out,
        "pp → e⁺e⁻ at √s = 13 TeV, LO, NNPDF23_lo_as_0130_qed (μF = m_Z), \
         default cuts (pT_ℓ > 10 GeV, |η_ℓ| < 2.5). vibegraph spectrum from \
         8×10⁶ Monte-Carlo samples of the (τ,y,cosθ) integrand; MadGraph σ \
         values from `hadronic_sigma_reference.json` anchor the integral.\n"
    )
    .unwrap();
    writeln!(out, "| m_ℓℓ bin (GeV) | dσ/dm_ℓℓ (pb/GeV) | bin σ (pb) |").unwrap();
    writeln!(out, "|---|---|---|").unwrap();
    for (i, d) in dens.iter().enumerate() {
        let lo = m_lo + i as f64 * bin_w;
        writeln!(
            out,
            "| {lo:.0}–{:.0} | {d:.4} | {:.3} |",
            lo + bin_w,
            d * bin_w
        )
        .unwrap();
    }
    writeln!(
        out,
        "\n## Integral cross-checks (vibegraph vs banked MadGraph)\n"
    )
    .unwrap();
    writeln!(out, "| range | vibegraph σ (pb) | MadGraph σ (pb) | rel |").unwrap();
    writeln!(out, "|---|---|---|---|").unwrap();
    writeln!(
        out,
        "| m_ℓℓ ∈ [60,120] | {sig_window:.2} | {mg_window:.2} | {:.3} |",
        (sig_window - mg_window).abs() / mg_window
    )
    .unwrap();
    writeln!(
        out,
        "| m_ℓℓ ∈ [20,200] | {sig_20_200:.2} | (full-range MG {mg_default:.2}) | — |"
    )
    .unwrap();
    writeln!(
        out,
        "\n(The full MadGraph σ = {mg_default:.2} pb covers all m_ℓℓ ≥ 2·pT_ℓ, so it \
         exceeds the [20,200] vibegraph integral by the m_ℓℓ > 200 tail.)"
    )
    .unwrap();

    let path = validation_dir().join("dy_dsigma_dmll.md");
    std::fs::write(&path, out).unwrap();
    eprintln!(
        "wrote {} ; [60,120] vibegraph {sig_window:.2} vs MG {mg_window:.2} pb",
        path.display()
    );
}
