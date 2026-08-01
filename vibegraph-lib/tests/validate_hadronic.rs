//! Extended validation: hadronic LO cross sections against banked MadGraph5
//! reference runs, assembled from the PDF luminosity, the compiled amplitude, the
//! run-card cuts, and VEGAS, and driven by the *same* run-card files, PDF set
//! (NNPDF23_lo_as_0130_qed / lhaid 247000) and fixed scale μF = m_Z MadGraph was
//! given.
//!
//! Every cross section here runs through the **general** hadronic path
//! ([`vibegraph::proton`]): the process's flavour-group decomposition convolved
//! over `(τ, y)` with a per-diagram multichannel inner map, one VEGAS grid per
//! `(group, diagram)` channel. There is no per-process integrand.
//!
//! Four reference runs are enforced: two σ(pp → e⁺e⁻) cards — default lepton
//! cuts, and the m_ll ∈ [60,120] window — σ(pp → ℓ⁺ℓ⁻ j) against the banked
//! `pp_to_llj_fixed` run, the one row with a coloured initial state, a three-body
//! final state, a jet cut and a strong coupling, and σ(pp → b b̄) against
//! `pp_to_bb_fixed`, the one whose hard process has no electroweak core at all
//! and whose `ŝ` floor therefore comes from a transverse cut rather than from a
//! lepton. Each row is measured over several seeds, because VEGAS's `1/σ²`
//! iteration combination reports an under-sampled region confidently.
//!
//! A pointwise integrand oracle pins the factors of the hadronic assembly — the
//! `(τ, y)` map and its Jacobian, the per-group parton luminosity, the group
//! amplitudes and the cut indicator — at fixed `(x₁, x₂, cosθ)` points against an
//! independent Python computation (`validation/madgraph/gen_dy_oracle.py`).
//!
//! Each gate writes its measurement to `target/validation-report/integrals/`
//! (see `common::report`), so the report table is assembled from what ran.
//!
//! Gated behind `extended-validation`; the σ tests need the fetched PDF set, the
//! banked reference JSON and the banked `pp_to_llj_fixed` run:
//!
//!     pixi run -e madgraph validate-hadronic
//!
//! Run it under `--profile release-debug` if invoking cargo directly: the ℓℓj
//! sweep is minutes optimised and hours unoptimised.

mod common;

use std::path::{Path, PathBuf};

use common::report::{ChannelSummary, IntegralsRow, SeedResult};
use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
use vibegraph::helas::eval::BoundAmplitude;
use vibegraph::helas::repr::lorentz::LorentzVector;
use vibegraph::pdf::{PdfMember, PdfSet};
use vibegraph::proton::{derive_flavor_groups, FlavorGroups, ProtonIntegrand};
use vibegraph::runcard::RunCard;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::{EvaluatedModel, UFOModel};

type V = LorentzVector<f64>;

const MU_F: f64 = 91.1880;
const SQRT_S_HAD: f64 = 13000.0;
const PDF_SET: &str = "NNPDF23_lo_as_0130_qed";

/// The Drell–Yan process both banked run cards were generated for.
const DY_PROCESS: &str = "p p > e+ e-";

/// The process of the banked `pp_to_llj_fixed` run, spelled as its `.mg5` script
/// spells it — the coupling orders included, since they set the diagram content.
const LLJ_PROCESS: &str = "p p > l+ l- j QCD=2 QED=2";

/// VEGAS budget of the Drell–Yan rows, per seed.
const DY_NEVAL: usize = 120_000;
const DY_NITER: usize = 12;
/// Points per survey iteration, and iterations, of the Drell–Yan channel-weight
/// adaptation.
const DY_ADAPT_SURVEY: usize = 20_000;
const DY_ADAPT_ITERS: usize = 6;
/// Independent seeds each Drell–Yan row is measured on. Three rather than one for
/// the reason the ℓℓj sweep gives below; the row is cheap enough to afford them.
const DY_SEEDS: &[u64] = &[20260719, 20260720, 20260721];

/// Independent seeds the ℓℓj cross section is measured on. Several runs rather
/// than one because VEGAS's `1/σ²` iteration combination reports an under-sampled
/// region as a confident number, which one seed cannot distinguish from a
/// converged one.
///
/// Three seeds here, not the five of the oracle-layer sweep: three already
/// separates a seed-unstable coverage defect from a converged estimate, and the
/// full sweep with its budget ladder is what the deeper layer is for.
const LLJ_SEEDS: &[u64] = &[20260730, 20260731, 20260732];
/// Points per survey iteration, and iterations, of the channel-weight adaptation.
const LLJ_ADAPT_SURVEY: usize = 8_000;
const LLJ_ADAPT_ITERS: usize = 5;
/// VEGAS budget per seed, chosen from a measured budget scan and not from cost
/// alone. The estimator approaches its limit **from below** — the unadapted early
/// iterations enter the `1/σ²` combination with underestimated variances — so the
/// sweep mean rises with `neval` per iteration: `418.5` at 60 000, `421.7` at
/// 150 000, `422.9` here, `423.5` at 600 000, each step about half the last. Below
/// 150 000 the residual exceeds MadGraph's own error and the sweep would be
/// measuring this crate's convergence rather than an agreement.
///
/// The per-channel allocation floors at 512 points a channel, so the 24 pooled
/// `(group, diagram)` channels spend at least 12 288 evaluations an iteration
/// whatever this says.
const LLJ_NEVAL: usize = 300_000;
const LLJ_NITER: usize = 10;
/// Largest relative distance from the banked MadGraph σ the ℓℓj sweep may show.
///
/// Above MadGraph's own `0.43%` Monte-Carlo error, which is the floor: no
/// agreement tighter than the reference's precision is meaningful. Below the
/// `1.0%` an under-converged budget produces, which is what it exists to catch.
/// The whole measured budget family — `0.28%`, `0.00%`, `0.16%` at 150 000,
/// 300 000 and 600 000 — sits inside it, so it is not a bound around one number.
const LLJ_MAX_REL: f64 = 0.005;
/// Scatter the estimates are allowed about their own mean, in units of their
/// quoted errors. Measured over the same budget family: `1.55`, `0.47`, `1.90`,
/// `0.37`.
const LLJ_MAX_CHI2_PER_DOF: f64 = 4.0;

/// Largest relative distance from the banked MadGraph σ a Drell–Yan row may show.
///
/// The rows sit far inside it: `+0.02%` on the default card and `−0.10%` on the
/// m_ll window, over three seeds, against per-seed spreads of `0.17%` and
/// `0.22%`. The bound is disjoined with a three-standard-deviation pull because
/// the Monte-Carlo errors here are small enough (`~0.06%` on the mean) that a
/// pull alone would be the tighter of the two and would be reading floating-point
/// reproducibility across machines rather than an agreement.
const DY_MAX_REL: f64 = 0.01;
/// Scatter the seeds are allowed about their own mean, in units of their quoted
/// errors — the guard the scalar pull cannot be: a run that missed a region
/// reports a small integral *and* a small error. Measured `0.74` and `1.19`.
const DY_MAX_CHI2_PER_DOF: f64 = 4.0;

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

/// The flavour decomposition of one process under one run card.
fn groups_for(
    process: &str,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    rc: &RunCard,
) -> FlavorGroups {
    let opts = ParsingOptions::default();
    let proc_card = parse_proc_card(&format!("generate {process}"), &opts).expect("proc card");
    let sets = generate_from_proc_card(&proc_card, model).expect("enumeration");
    derive_flavor_groups(sets, model, evaluated, rc).expect("flavour groups")
}

/// What the rule-based channel composition chose, one entry per channel — the
/// summary the report reprints beside the cross section.
fn subsampler_summary(integ: &ProtonIntegrand<'_>) -> Vec<ChannelSummary> {
    integ
        .channel_ids()
        .iter()
        .zip(integ.channel_samplers())
        .map(|(id, s)| ChannelSummary {
            channel: format!("group {} diagram {}", id.group, id.diagram),
            sampler: s.clone(),
        })
        .collect()
}

/// One seed's `(σ, Δσ)` in pb through the general path, with the run card's own
/// scale prescription installed and the channel weights adapted on that seed's
/// own survey substream.
///
/// `expect_alpha_s` says whether the process's matrix element must carry the
/// strong coupling. It is asserted rather than observed: a QCD process whose
/// amplitude had lost its gluon vertex would integrate to a plausible number.
#[allow(clippy::too_many_arguments)]
fn run_seed(
    groups: &FlavorGroups,
    amps: &[BoundAmplitude<'_, f64>],
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    set: &PdfSet,
    pdf: &PdfMember,
    rc: &RunCard,
    budget: (usize, usize, usize, usize),
    seed: u64,
    expect_alpha_s: bool,
    summary: &mut Vec<ChannelSummary>,
) -> (f64, f64) {
    let (survey, adapt_iters, neval, niter) = budget;
    let mut integ = ProtonIntegrand::new(groups, amps, evaluated, pdf, SQRT_S_HAD, MU_F)
        .expect("hadronic integrand");
    let report = integ
        .use_run_card_scales(model, evaluated, rc, Some(&set.info.alpha_s))
        .expect("run card scale prescription compiles");
    let constant = report.constant_scales.unwrap_or_else(|| {
        panic!("the reference run card no longer fixes both scales: {report:?}")
    });
    assert_eq!(
        (constant.mu_r, constant.mu_f),
        (MU_F, [MU_F, MU_F]),
        "the reference run card no longer fixes both scales at m_Z"
    );
    assert_eq!(
        report.depends_on_alpha_s, expect_alpha_s,
        "the matrix element's dependence on the strong coupling is not what this \
         process's diagram content implies"
    );
    if summary.is_empty() {
        *summary = subsampler_summary(&integ);
    }
    integ.adapt_alphas(seed, survey, adapt_iters, 0.5);
    integ.integrate(neval, niter, seed)
}

/// The inverse-variance mean of independent seeds, its error, and the scatter of
/// the seeds about that mean in units of their own quoted errors.
///
/// The scatter and not the error is what shows a missed region: a run that misses
/// one reports a small integral *and* a small variance, which the mean alone
/// cannot distinguish from convergence.
fn combine_seeds(runs: &[SeedResult]) -> (f64, f64, f64) {
    let inv_var: f64 = runs
        .iter()
        .map(|r| 1.0 / (r.sigma_err_pb * r.sigma_err_pb))
        .sum();
    let mean: f64 = runs
        .iter()
        .map(|r| r.sigma_pb / (r.sigma_err_pb * r.sigma_err_pb))
        .sum::<f64>()
        / inv_var;
    let mean_err = inv_var.sqrt().recip();
    let dof = runs.len().saturating_sub(1).max(1) as f64;
    let chi2: f64 = runs
        .iter()
        .map(|r| ((r.sigma_pb - mean) / r.sigma_err_pb).powi(2))
        .sum::<f64>()
        / dof;
    (mean, mean_err, chi2)
}

/// σ(p p → e⁺e⁻) through the general hadronic path against a banked MadGraph run.
///
/// What it proves: the whole assembled chain — the `(τ, y)` map and its Jacobian,
/// the `x·f` luminosity, the flux and `2π` measure, the cut filter in the lab
/// frame, the flavour partition and both beam orderings — reproduces a measured
/// hadronic cross section, on real parton distributions and a real run card.
///
/// What it cannot see: anything specific to a coloured initial state or a
/// three-body final state. Drell–Yan has no gluon-initiated group, no peripheral
/// channel, no strong coupling and no jet cut, so the spacelike floor, the grid
/// `αs` and the three-body spine are all untouched by it — those are the ℓℓj
/// row's. Being a single scalar it is also blind to a mis-sampled region of small
/// measure, which the seed sweep and not the pull is what guards.
fn check_dy_run(run: &str, card: &str) {
    let (mg, mg_err) = banked(run).expect("banked Drell-Yan reference");
    let card_path = validation_dir().join(card);
    let rc = RunCard::parse_file(&card_path).expect("parse run card");

    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let groups = groups_for(DY_PROCESS, &model, &evaluated, &rc);
    let set = load_pdf_set();
    let pdf = set.member(0).expect("PDF member 0");
    let amps: Vec<BoundAmplitude<f64>> = groups
        .groups()
        .iter()
        .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), &evaluated))
        .collect();

    let mut summary = Vec::new();
    let mut runs = Vec::new();
    for &seed in DY_SEEDS {
        let (sigma, err) = run_seed(
            &groups,
            &amps,
            &model,
            &evaluated,
            &set,
            &pdf,
            &rc,
            (DY_ADAPT_SURVEY, DY_ADAPT_ITERS, DY_NEVAL, DY_NITER),
            seed,
            // Drell-Yan at this order carries no strong coupling; a matrix element
            // that did would need one, and `pdlabel = lhapdf` refuses to supply it.
            false,
            &mut summary,
        );
        eprintln!(
            "[{run} seed {seed}] vibegraph σ = {sigma:.3} ± {err:.3} pb | rel = {:+.4}",
            sigma / mg - 1.0
        );
        runs.push(SeedResult {
            seed,
            sigma_pb: sigma,
            sigma_err_pb: err,
        });
    }

    let (mean, mean_err, chi2) = combine_seeds(&runs);
    let combined = (mean_err * mean_err + mg_err * mg_err).sqrt();
    let pull = (mean - mg) / combined;
    let rel = mean / mg - 1.0;
    eprintln!(
        "[{run}] GATE vibegraph σ = {mean:.3} ± {mean_err:.3} pb ({} seeds, \
         χ²/dof = {chi2:.2}) | MG σ = {mg:.3} ± {mg_err:.3} pb | \
         pull = {pull:+.2} | rel = {rel:+.4}",
        runs.len()
    );

    let ok = pull.abs() < 3.0 || rel.abs() < DY_MAX_REL;
    let mut row = IntegralsRow::new("pp_to_ll", DY_PROCESS, "gate").with_variant(run);
    row.status = if ok && chi2 < DY_MAX_CHI2_PER_DOF {
        "pass"
    } else {
        "fail"
    };
    row.sigma_vg_pb = mean;
    row.sigma_vg_err_pb = mean_err;
    row.sigma_mg_pb = mg;
    row.sigma_mg_err_pb = mg_err;
    row.pull = pull;
    row.rel = rel;
    row.chi2_dof = chi2;
    row.seeds = runs.iter().map(|r| r.seed).collect();
    row.per_seed = runs.clone();
    row.neval = DY_NEVAL;
    row.niter = DY_NITER;
    row.subsampler = summary;
    row.write();

    assert!(
        ok,
        "[{run}] σ disagreement: vibegraph {mean:.3}±{mean_err:.3} vs \
         MG {mg:.3}±{mg_err:.3} pb, pull = {pull:+.2}, rel = {rel:+.4}"
    );
    assert!(
        chi2 < DY_MAX_CHI2_PER_DOF,
        "[{run}] the seeds scatter by more than they claim: χ²/dof = {chi2:.2} over {runs:?}"
    );
}

#[test]
fn sigma_default_cuts_vs_mg() {
    check_dy_run("default", "dy13_default_run_card.dat");
}

#[test]
fn sigma_mmll_window_vs_mg() {
    check_dy_run("mmll_60_120", "dy13_mmll_run_card.dat");
}

/// Partonic-CM external momenta `[q, q̄, e⁺, e⁻]` of a back-to-back dilepton
/// configuration at `(√ŝ, cosθ)`, beams along ±z and the azimuth fixed — the
/// configuration `gen_dy_oracle.py` evaluates MadGraph's standalone matrix
/// element at. Built here rather than taken from the library, so the two sides of
/// the comparison construct the point independently.
fn dilepton_cm(sqrt_shat: f64, cos_theta: f64) -> Vec<V> {
    let half = sqrt_shat / 2.0;
    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
    vec![
        V::new(half, 0.0, 0.0, half),
        V::new(half, 0.0, 0.0, -half),
        V::new(half, half * sin_theta, 0.0, half * cos_theta),
        V::new(half, -half * sin_theta, 0.0, -half * cos_theta),
    ]
}

/// The same configuration in the lab frame, the frame the cut filter reads: the
/// beams at their momentum fractions and the leptons boosted along z by the
/// parton system's rapidity.
fn dilepton_lab(cm: &[V], x1: f64, x2: f64) -> Vec<V> {
    let beta = (x1 - x2) / (x1 + x2);
    let gamma = 1.0 / (1.0 - beta * beta).sqrt();
    let boost = |p: &V| {
        V::new(
            gamma * (p.e() + beta * p.pz()),
            p.px(),
            p.py(),
            gamma * (p.pz() + beta * p.e()),
        )
    };
    let e_beam = SQRT_S_HAD / 2.0;
    vec![
        V::new(x1 * e_beam, 0.0, 0.0, x1 * e_beam),
        V::new(x2 * e_beam, 0.0, 0.0, -x2 * e_beam),
        boost(&cm[2]),
        boost(&cm[3]),
    ]
}

/// Pointwise integrand oracle: at ~10 pinned `(x₁, x₂, cosθ)` points (including
/// two straddling the pT_ℓ = 10 GeV cut boundary), the factors of vibegraph's
/// hadronic assembly must match an independent Python oracle (LHAPDF `xfxQ2` ×
/// MadGraph standalone |M|²) to ≤ 1e-9 relative. Regenerate the oracle with
/// `pixi run -e madgraph generate-dy-oracle`.
///
/// The factors compared are the ones that belong to the *physics* of the point
/// and not to a phase-space map: the `(τ, y)` outer map's `x₁, x₂, √ŝ` and its
/// Jacobian, each flavour group's summed parton luminosity over both beam
/// orderings, each group's `|M(q)|²` at the point, and the cut indicator on the
/// lab-frame configuration.
///
/// What it deliberately does not compare is the assembled integrand value. The
/// oracle's `phat` and `value` columns carry the flat-`cosθ` two-body measure the
/// bespoke Drell–Yan map used; the general path draws the same physical point
/// through a per-diagram multichannel whose weight is a sampling density, so the
/// two assembled values differ by exactly that ratio and agreeing on it would
/// test the ratio rather than the integrand. The two paths' assembled values meet
/// after integration, which is what the σ gates above are.
///
/// It is also blind to anything the mirrored beam ordering does pointwise: the
/// oracle sums both orderings' luminosities against a single `|M(q)|²`, which is
/// right only once the polar angle is integrated over, so `lum` here is checked
/// against the summed pair and `m2` against the unreflected argument.
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

    let rc = RunCard::parse_file(&validation_dir().join("dy13_default_run_card.dat")).unwrap();
    let groups = groups_for(DY_PROCESS, &model, &evaluated, &rc);
    let set = load_pdf_set();
    let pdf = set.member(0).expect("PDF member 0");
    let amps: Vec<BoundAmplitude<f64>> = groups
        .groups()
        .iter()
        .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), &evaluated))
        .collect();
    let integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
        .expect("hadronic integrand");

    // The oracle's two coupling classes, located by the flavour content of the
    // groups rather than by group order, so a reordering of the decomposition
    // cannot silently swap which |M|² is compared against which.
    let is_up = |g: &vibegraph::proton::FlavorGroup| {
        g.members()
            .iter()
            .all(|m| matches!(m.incoming[0].abs(), 2 | 4))
    };
    let up = groups
        .groups()
        .iter()
        .position(is_up)
        .expect("up-type group");
    let down = groups
        .groups()
        .iter()
        .position(|g| !is_up(g))
        .expect("down-type group");
    assert_eq!(
        groups.groups().len(),
        2,
        "Drell-Yan decomposes into two coupling classes"
    );

    let mut scratch: Vec<_> = amps.iter().map(|a| a.scratch_space()).collect();

    const TOL: f64 = 1e-9;
    let rel = |a: f64, b: f64| (a - b).abs() / b.abs().max(1e-30);
    let mut worst = 0.0f64;

    assert!(
        rel(integ.tau_min(), oracle["tau_min"].as_f64().unwrap()) <= TOL,
        "the tau map's lower support moved: {} vs oracle {}",
        integ.tau_min(),
        oracle["tau_min"]
    );

    for (i, p) in points.iter().enumerate() {
        let u: Vec<f64> = p["u"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        let g = |k: &str| p[k].as_f64().unwrap();

        // The outer map takes the oracle's first two coordinates; its third is the
        // bespoke map's `cosθ = 2u₃ − 1`, which the oracle also banks directly.
        let outer = integ.outer_point(&u);
        let cos_theta = g("cos_theta");
        let cm = dilepton_cm(outer.sqrt_shat, cos_theta);
        let lab = dilepton_lab(&cm, outer.x1, outer.x2);

        let lum = |gi: usize| {
            let [direct, mirror] =
                groups.groups()[gi].luminosity(&pdf, outer.x1, outer.x2, [MU_F, MU_F]);
            direct + mirror
        };
        let m2 = |gi: usize, scratch: &mut Vec<_>| amps[gi].eval_m2(&cm, &mut scratch[gi]);

        assert_eq!(
            groups.groups()[0].cuts().pass(&lab),
            p["pass"].as_bool().unwrap(),
            "cut indicator, point {i}"
        );
        for (name, got, want) in [
            ("x1", outer.x1, g("x1")),
            ("x2", outer.x2, g("x2")),
            ("sqrt_shat", outer.sqrt_shat, g("sqrt_shat")),
            ("jac", outer.jac, g("jac")),
            ("lum_up", lum(up), g("lum_up")),
            ("lum_down", lum(down), g("lum_down")),
            ("m2_up", m2(up, &mut scratch), g("m2_up")),
            ("m2_down", m2(down, &mut scratch), g("m2_down")),
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
/// This is the only cross section here for a process with a coloured initial
/// state, a three-body final state, a jet cut and a strong coupling — everything
/// the Drell–Yan rows above are blind to. The comparison is against MadGraph's
/// own number for the same cards: the same proc card content, the same run card
/// file, the same PDF set and the same fixed scales.
///
/// **Several seeds, not one.** VEGAS combines its iterations by `1/σ²`, so a run
/// that under-samples a region reports a confidently wrong integral with a small
/// error rather than a large one, and a single seed agreeing is then not
/// evidence. The runs are compared individually and through their inverse-variance
/// mean, and it is the mean the gate is on.
///
/// What it cannot see: anything the cross section integrates over. A per-diagram
/// phase, a colour-flow relabelling and a helicity-by-helicity error all leave
/// `Σ|M|²` and hence σ alone — those are pinned at the amplitude level by
/// `amplitude_oracle`. It also cannot separate the
/// phase-space map from the matrix element: a map whose weight and density were
/// both wrong by one factor would integrate correctly.
#[test]
fn sigma_llj_fixed_scale_vs_mg() {
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
    let groups = groups_for(LLJ_PROCESS, &model, &evaluated, &rc);
    let set = load_pdf_set();
    let pdf = set.member(0).expect("PDF member 0");
    let amps: Vec<BoundAmplitude<f64>> = groups
        .groups()
        .iter()
        .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), &evaluated))
        .collect();

    let mut summary = Vec::new();
    let mut runs: Vec<SeedResult> = Vec::new();
    for &seed in LLJ_SEEDS {
        let (sigma, err) = run_seed(
            &groups,
            &amps,
            &model,
            &evaluated,
            &set,
            &pdf,
            &rc,
            (LLJ_ADAPT_SURVEY, LLJ_ADAPT_ITERS, LLJ_NEVAL, LLJ_NITER),
            seed,
            // A QCD ℓℓj matrix element must carry the strong coupling; one that
            // did not would be missing its gluon vertex.
            true,
            &mut summary,
        );
        eprintln!(
            "[llj_fixed seed {seed}] vibegraph σ = {sigma:.3} ± {err:.3} pb | \
             rel = {:+.4} | pull = {:+.2}",
            sigma / mg - 1.0,
            (sigma - mg) / (err * err + mg_err * mg_err).sqrt()
        );
        runs.push(SeedResult {
            seed,
            sigma_pb: sigma,
            sigma_err_pb: err,
        });
    }

    let (mean, mean_err, chi2) = combine_seeds(&runs);
    let combined = (mean_err * mean_err + mg_err * mg_err).sqrt();
    let pull = (mean - mg) / combined;
    let rel = mean / mg - 1.0;
    eprintln!(
        "[llj_fixed] GATE vibegraph σ = {mean:.3} ± {mean_err:.3} pb ({} seeds, \
         χ²/dof = {chi2:.2}) | MG σ = {mg:.3} ± {mg_err:.3} pb | \
         pull = {pull:+.2} | rel = {rel:+.4}",
        runs.len()
    );

    let ok = pull.abs() < 3.0 && rel.abs() < LLJ_MAX_REL && chi2 < LLJ_MAX_CHI2_PER_DOF;
    let mut row = IntegralsRow::new("pp_to_llj_fixed", LLJ_PROCESS, "gate");
    row.status = if ok { "pass" } else { "fail" };
    row.sigma_vg_pb = mean;
    row.sigma_vg_err_pb = mean_err;
    row.sigma_mg_pb = mg;
    row.sigma_mg_err_pb = mg_err;
    row.pull = pull;
    row.rel = rel;
    row.chi2_dof = chi2;
    row.seeds = runs.iter().map(|r| r.seed).collect();
    row.per_seed = runs.clone();
    row.neval = LLJ_NEVAL;
    row.niter = LLJ_NITER;
    row.subsampler = summary;
    row.note = Some(
        "three seeds at 300k in this layer; the full seed sweep and the budget \
         ladder are oracle-layer"
            .to_string(),
    );
    row.write();

    assert!(
        pull.abs() < 3.0,
        "[llj_fixed] σ disagreement: vibegraph {mean:.3}±{mean_err:.3} vs \
         MG {mg:.3}±{mg_err:.3} pb, pull = {pull:+.2}"
    );
    assert!(
        rel.abs() < LLJ_MAX_REL,
        "[llj_fixed] σ disagreement: vibegraph {mean:.3}±{mean_err:.3} vs \
         MG {mg:.3}±{mg_err:.3} pb, rel = {rel:+.4} > {LLJ_MAX_REL}"
    );
    assert!(
        chi2 < LLJ_MAX_CHI2_PER_DOF,
        "[llj_fixed] the seeds scatter by more than they claim: \
         χ²/dof = {chi2:.2} over {runs:?}"
    );
}

/// The process of the banked `pp_to_bb_fixed` run, spelled as its `.mg5` script
/// spells it.
const BB_PROCESS: &str = "p p > b b~ QCD=2";

/// Independent seeds the `b b̄` cross section is measured on, for the reason the
/// ℓℓj sweep gives.
const BB_SEEDS: &[u64] = &[20260801, 20260802, 20260803];
/// Points per survey iteration, and iterations, of the channel-weight adaptation.
const BB_ADAPT_SURVEY: usize = 8_000;
const BB_ADAPT_ITERS: usize = 5;
/// VEGAS budget per seed, taken from a measured ladder rather than from cost.
/// The three-seed mean is flat in the budget — `−0.07%`, `+0.04%`, `−0.01%`,
/// `−0.03%`, `−0.00%` at 75 000, 150 000, 300 000, 600 000 and 1 200 000 points
/// an iteration — so this row is converged well below the budget it runs at, and
/// the seed sweep is measuring an agreement rather than this crate's convergence.
/// The ℓℓj row, whose estimator approaches its limit from below, is the reason
/// that is checked rather than assumed.
const BB_NEVAL: usize = 300_000;
const BB_NITER: usize = 10;
/// Largest relative distance from the banked MadGraph σ the `b b̄` sweep may show.
///
/// MadGraph's own Monte-Carlo error on this run is `0.16%`, which is the floor —
/// no agreement tighter than the reference's precision is meaningful — and the
/// combined error of the two sides is `0.17%`, so this is the three-standard-
/// deviation distance. The whole measured budget family sits inside `0.07%`.
const BB_MAX_REL: f64 = 0.005;
/// Scatter the seeds are allowed about their own mean, in units of their own
/// quoted errors — the guard the scalar pull cannot be, since a run that missed a
/// region reports a small integral *and* a small error. Measured `0.48`, `1.67`,
/// `0.51`, `0.96`, `0.91` over the same budget ladder.
const BB_MAX_CHI2_PER_DOF: f64 = 4.0;

/// The `ŝ` floor the banked `pp_to_bb_fixed` card implies, against MadGraph's own
/// `setcuts.f` arithmetic for the same card.
///
/// The `(τ, y)` map draws `τ` logarithmically between `τ_min` and 1, so it needs
/// a positive lower bound on `ŝ`; a final state with no lepton in it is the case
/// that has to come from somewhere other than the lepton cuts.
///
/// `setcuts.f:574-600` walks the b-class legs accumulating
/// `smin_p = Σ max(eb, ptb, 0)` and takes `max(smin_p**2, −Σ mb**2, 0)`; line 707
/// then raises the result to `max(smin, (Σ pmass)**2, dsqrt_shat**2)`. At
/// `ptb = 20`, `mb = 4.7` and `mmbb = dsqrt_shat = 0` that is
/// `max(40², (2·4.7)², 0) = 1600`, and 1600 is the number this row must not
/// exceed: the banked run integrated `τ` from `1600/s` upwards, so a higher floor
/// would clip phase space its cross section covers.
///
/// The flavour census is printed alongside because it is what this row was banked
/// for: the first group whose mirrored members carry a large share of the cross
/// section.
#[test]
fn bb_fixed_shat_floor_matches_madgraphs_own() {
    let run_dir = validation_dir().join("output/pp_to_bb_fixed");
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
    let groups = groups_for(BB_PROCESS, &model, &evaluated, &rc);
    let mirrored = groups.groups().iter().filter(|g| g.has_mirror()).count();
    eprintln!(
        "[bb_fixed] banked MG σ = {mg:.1} ± {mg_err:.1} pb | {} flavour groups, \
         {mirrored} mirrored, {} subprocesses",
        groups.groups().len(),
        groups.subprocess_count()
    );

    let masses = groups.groups()[0].final_masses();
    let ptb = rc.float("ptb");
    let mg_smin = (2.0 * ptb).powi(2).max(masses.iter().sum::<f64>().powi(2));
    let cuts = groups.groups()[0].cuts();
    let shat_min = cuts.shat_min();
    let tau_min = shat_min / (SQRT_S_HAD * SQRT_S_HAD);
    eprintln!(
        "[bb_fixed] ptb = {ptb}, m_b = {:?} => shat_min = {shat_min} GeV^2 \
         (setcuts.f: {mg_smin}), ln(1/tau_min) = {:.3}",
        masses,
        (1.0 / tau_min).ln()
    );

    assert_eq!(
        shat_min, mg_smin,
        "[bb_fixed] the floor this crate derives is not the one MadGraph's \
         setcuts.f computes for the same card"
    );
    assert!(
        (1.0 / tau_min).ln().is_finite(),
        "[bb_fixed] ln(1/tau_min) is not finite, so the (tau, y) map is unusable"
    );
}

/// σ(p p → b b̄) at a fixed scale through the general hadronic path, against the
/// banked `pp_to_bb_fixed` MadGraph run.
///
/// The only cross section here whose hard process carries no electroweak core at
/// all: three flavour groups, two of them mirrored, a gluon-initiated group and a
/// massive final state. It is what makes the `ŝ` floor above a live gate rather
/// than an arithmetic identity — a floor above the true threshold would clip
/// phase space and show up as a low cross section.
///
/// **Several seeds, not one**, for the reason the ℓℓj sweep gives: VEGAS's `1/σ²`
/// iteration combination reports an under-sampled region as a confident number.
///
/// What it cannot see: everything σ integrates over — per-diagram phases,
/// colour-flow relabellings, helicity-by-helicity errors — and, being a single
/// scalar, a mis-sampled region of small measure, which the seed scatter and not
/// the pull is what guards.
#[test]
fn sigma_bb_fixed_scale_vs_mg() {
    let run_dir = validation_dir().join("output/pp_to_bb_fixed");
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
    let groups = groups_for(BB_PROCESS, &model, &evaluated, &rc);
    let set = load_pdf_set();
    let pdf = set.member(0).expect("PDF member 0");
    let amps: Vec<BoundAmplitude<f64>> = groups
        .groups()
        .iter()
        .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), &evaluated))
        .collect();

    let mut summary = Vec::new();
    let mut runs: Vec<SeedResult> = Vec::new();
    for &seed in BB_SEEDS {
        let (sigma, err) = run_seed(
            &groups,
            &amps,
            &model,
            &evaluated,
            &set,
            &pdf,
            &rc,
            (BB_ADAPT_SURVEY, BB_ADAPT_ITERS, BB_NEVAL, BB_NITER),
            seed,
            // A b b̄ matrix element at QCD=2 must carry the strong coupling.
            true,
            &mut summary,
        );
        eprintln!(
            "[bb_fixed seed {seed}] vibegraph σ = {sigma:.1} ± {err:.1} pb | \
             rel = {:+.4} | pull = {:+.2}",
            sigma / mg - 1.0,
            (sigma - mg) / (err * err + mg_err * mg_err).sqrt()
        );
        runs.push(SeedResult {
            seed,
            sigma_pb: sigma,
            sigma_err_pb: err,
        });
    }

    let (mean, mean_err, chi2) = combine_seeds(&runs);
    let combined = (mean_err * mean_err + mg_err * mg_err).sqrt();
    let pull = (mean - mg) / combined;
    let rel = mean / mg - 1.0;
    eprintln!(
        "[bb_fixed] GATE vibegraph σ = {mean:.1} ± {mean_err:.1} pb ({} seeds, \
         χ²/dof = {chi2:.2}) | MG σ = {mg:.1} ± {mg_err:.1} pb | \
         pull = {pull:+.2} | rel = {rel:+.4}",
        runs.len()
    );

    let ok = pull.abs() < 3.0 && rel.abs() < BB_MAX_REL && chi2 < BB_MAX_CHI2_PER_DOF;
    let mut row = IntegralsRow::new("pp_to_bb_fixed", BB_PROCESS, "gate");
    row.status = if ok { "pass" } else { "fail" };
    row.sigma_vg_pb = mean;
    row.sigma_vg_err_pb = mean_err;
    row.sigma_mg_pb = mg;
    row.sigma_mg_err_pb = mg_err;
    row.pull = pull;
    row.rel = rel;
    row.chi2_dof = chi2;
    row.seeds = runs.iter().map(|r| r.seed).collect();
    row.per_seed = runs.clone();
    row.neval = BB_NEVAL;
    row.niter = BB_NITER;
    row.subsampler = summary;
    row.note = Some(
        "three seeds at 300k in this layer; the mean is flat across a 75k to 1.2M \
         budget ladder"
            .to_string(),
    );
    row.write();

    assert!(
        pull.abs() < 3.0 && rel.abs() < BB_MAX_REL,
        "[bb_fixed] σ disagreement: vibegraph {mean:.1}±{mean_err:.1} vs \
         MG {mg:.1}±{mg_err:.1} pb, pull = {pull:+.2}, rel = {rel:+.4}"
    );
    assert!(
        chi2 < BB_MAX_CHI2_PER_DOF,
        "[bb_fixed] the seeds scatter by more than they claim: \
         χ²/dof = {chi2:.2} over {runs:?}"
    );
}
