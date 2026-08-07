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
//! lepton. Each row is measured over several seeds, because an under-sampled
//! region is reported confidently: a run that misses one returns a small
//! integral *and* a small error, which no single seed's pull can see.
//!
//! A fifth is measured and **not** enforced: σ(pp → ℓ⁺ℓ⁻ j) against
//! `pp_to_llj_dyn`, whose card is the enforced ℓℓj one with its three
//! `fixed_*_scale` switches turned off, so the kT-clustered per-event scale is
//! the only moving part in the chain. It carries a diagnosed disagreement; see
//! [`sigma_llj_dynamical_scale_vs_mg`].
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

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use common::report::{ChannelSummary, IntegralsRow, SeedResult, Stopwatch};
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
/// than one because an under-sampled region is reported as a confident number,
/// which one seed cannot distinguish from a converged one.
///
/// Five, and the same five the oracle-layer budget ladders sweep, so the scatter
/// this gate forms is the statistic [`LLJ_MAX_CHI2_PER_DOF`] is calibrated on
/// rather than a three-seed cousin of it. χ²/dof over three seeds carries two
/// degrees of freedom, which is wide enough that a converged row can post an
/// alarming value and a genuinely scattered one an unremarkable value.
const LLJ_SEEDS: &[u64] = &[20260730, 20260731, 20260732, 20260733, 20260734];
/// Points per survey iteration, and iterations, of the channel-weight adaptation.
const LLJ_ADAPT_SURVEY: usize = 8_000;
const LLJ_ADAPT_ITERS: usize = 5;
/// VEGAS budget per seed, chosen from a measured budget scan and not from cost
/// alone: the rung both ℓℓj rows have stopped moving on, over five seeds a rung.
///
/// `probe_llj_fixed_budget_ladder` reads `423.81` / `424.09` / `424.24` /
/// `423.94` pb at `75k` / `150k` / `300k` / `600k` (χ²/dof `1.66`, `1.03`,
/// `1.11`, `0.40`) against MadGraph's `423.84 ± 1.52`, and
/// `probe_llj_dyn_budget_ladder` reads `416.23` / `416.27` / `416.26` /
/// `416.13` pb (χ²/dof `2.59`, `0.66`, `0.75`, `0.29`) against its
/// `415.42 ± 1.36`. Both ladders are flat across an eightfold budget —
/// `0.10 %` and `0.04 %` end to end, against the references' own `0.36 %` and
/// `0.33 %` — so this is a rung where the estimator has converged rather than
/// the cheapest one that happens to agree.
///
/// `150 000` rather than `75 000`, even though the reference's precision alone
/// would license the cut: the two ladders are flat at both, but `75k` carries the
/// largest inter-seed scatter on either row (χ²/dof `1.66` and `2.59` against
/// `≤ 1.11` everywhere above it, the dynamical row's driven by one seed `2.6`
/// standard deviations high), which is the rung a three-seed gate would be
/// reading. Seed scatter is the floor a precision argument does not get to cross.
///
/// The per-channel allocation floors at 512 points a channel, so the 24 pooled
/// `(group, diagram)` channels spend at least 12 288 evaluations an iteration
/// whatever this says.
const LLJ_NEVAL: usize = 150_000;
const LLJ_NITER: usize = 10;
/// Largest relative distance from the banked MadGraph σ the ℓℓj sweep may show.
///
/// Above MadGraph's own `0.36%` Monte-Carlo error on this run, which is the
/// floor: no agreement tighter than the reference's precision is meaningful.
/// Below the `1.0%` an under-converged budget produces, which is what it exists
/// to catch. The whole measured budget family — `−0.01%`, `+0.06%`, `+0.09%`,
/// `+0.02%` over five seeds at 75 000 to 600 000 — sits an order inside it, so it
/// is not a bound around one number.
const LLJ_MAX_REL: f64 = 0.005;
/// The same bound for the *dynamical*-scale row, and now set at the same thing.
///
/// This row's residual used to be a systematic: the cluster scale was read in the
/// channel the sampler drew the point in, so σ depended on the channel partition
/// it was integrated with, and the row read `−0.68%`. Each point's integration
/// configuration is now drawn from its own squared amplitudes, and what is left
/// is the reference's own error: `+0.21%` over five seeds at this budget, at
/// `χ²/dof 0.66`.
///
/// `0.005` is MadGraph's own `0.33%` on this run — the floor no agreement can be
/// tighter than — with headroom. The budget ladder is what says the row is
/// converged here rather than merely close: `416.23`, `416.27`, `416.26`,
/// `416.13` pb over five seeds at `75k`, `150k`, `300k` and `600k`
/// (`probe_llj_dyn_budget_ladder`) against MadGraph's `415.42 ± 1.36` — a
/// `0.04%` end-to-end span across an eightfold budget, an eighth of the
/// reference's own error, with no direction to it.
const LLJ_DYN_MAX_REL: f64 = 0.005;
/// Scatter the estimates are allowed about their own mean, in units of their
/// quoted errors. Measured over the same budget family, worst rung first:
/// `2.59`, `1.66`, `1.11`, `1.03`, `0.75`, `0.66`, `0.40`, `0.29` across both
/// ℓℓj ladders.
///
/// Both gates that read this form it over [`LLJ_SEEDS`], which is the same
/// five-seed sweep those ladder rungs used, so the bound and the statistic it
/// bounds have the same number of degrees of freedom behind them.
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
    run_seed_mapped(
        groups,
        amps,
        model,
        evaluated,
        set,
        pdf,
        rc,
        budget,
        seed,
        expect_alpha_s,
        summary,
        true,
    )
}

/// [`run_seed`] with the peripheral channels' fiducial transfer bound under the
/// caller's control, so what the bound is worth can be read off two runs that
/// differ in nothing else.
#[allow(clippy::too_many_arguments)]
fn run_seed_mapped(
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
    bound_transfer: bool,
) -> (f64, f64) {
    run_seed_shaped(
        groups,
        amps,
        model,
        evaluated,
        set,
        pdf,
        rc,
        budget,
        seed,
        expect_alpha_s,
        summary,
        bound_transfer,
        ScaleShape::FixedAtMz,
    )
}

/// What a run card's scale prescription must resolve to.
///
/// Asserted rather than observed. The fixed-scale and dynamical cards of the same
/// process differ in three booleans and in nothing else, so one read as the other
/// integrates a plausible cross section at the wrong scale. MadGraph's own two
/// numbers for `p p → ℓ⁺ℓ⁻ j` differ by `1.75%` — `415.42` against `422.84` —
/// which is three times the fixed-scale row's tolerance and well outside either
/// run's Monte-Carlo error, so the confusion would not be a small one.
enum ScaleShape {
    /// Both scales pinned at [`MU_F`], applied once rather than per point.
    FixedAtMz,
    /// Recomputed on every event by the kT clustering, over channel forests
    /// derived from the process's own diagrams.
    PerEvent,
}

/// [`run_seed_mapped`] with the scale prescription's expected shape under the
/// caller's control.
#[allow(clippy::too_many_arguments)]
fn run_seed_shaped(
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
    bound_transfer: bool,
    shape: ScaleShape,
) -> (f64, f64) {
    let (survey, adapt_iters, neval, niter) = budget;
    let build = if bound_transfer {
        ProtonIntegrand::new
    } else {
        ProtonIntegrand::new_unbounded
    };
    let mut integ =
        build(groups, amps, evaluated, pdf, SQRT_S_HAD, MU_F).expect("hadronic integrand");
    let report = integ
        .use_run_card_scales(model, evaluated, rc, Some(&set.info.alpha_s))
        .expect("run card scale prescription compiles");
    match shape {
        ScaleShape::FixedAtMz => {
            let constant = report.constant_scales.unwrap_or_else(|| {
                panic!("the reference run card no longer fixes both scales: {report:?}")
            });
            assert_eq!(
                (constant.mu_r, constant.mu_f),
                (MU_F, [MU_F, MU_F]),
                "the reference run card no longer fixes both scales at m_Z"
            );
        }
        ScaleShape::PerEvent => {
            assert!(
                report.constant_scales.is_none(),
                "the dynamical run card collapsed to a constant scale: {report:?}"
            );
            let channels = report
                .channels
                .expect("the clustering branch was given no channel forests");
            assert!(channels > 0, "the clustering branch has no channels");
        }
    }
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

/// The unweighted mean of independent seeds, its error, and the scatter of the
/// seeds about that mean in units of their own quoted errors.
///
/// Seeds run equal budgets, so weighting by `1/σ²` here is the same bias Lepage's
/// theorem forbids for VEGAS's own iteration combination: a seed whose variance
/// estimate came out low by chance is double-counted, both in the mean and in
/// its own weight. The unweighted mean and `err = √(Σᵢ σᵢ²)/n` carry no such bias.
///
/// The scatter and not the error is what shows a missed region: a run that misses
/// one reports a small integral *and* a small variance, which the mean alone
/// cannot distinguish from convergence.
fn combine_seeds(runs: &[SeedResult]) -> (f64, f64, f64) {
    let n = runs.len() as f64;
    let mean: f64 = runs.iter().map(|r| r.sigma_pb).sum::<f64>() / n;
    let var_sum: f64 = runs.iter().map(|r| r.sigma_err_pb * r.sigma_err_pb).sum();
    let mean_err = var_sum.sqrt() / n;
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
    let clock = Stopwatch::start();
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
    row.duration_s = Some(clock.seconds());
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
/// **Several seeds, not one.** A run that under-samples a region reports a
/// confidently wrong integral with a small error rather than a large one — the
/// missed region is missing from the variance as much as from the integral — so
/// a single seed agreeing is not evidence. The runs are compared individually
/// and through their inverse-variance mean, and it is the mean the gate is on.
///
/// What it cannot see: anything the cross section integrates over. A per-diagram
/// phase, a colour-flow relabelling and a helicity-by-helicity error all leave
/// `Σ|M|²` and hence σ alone — those are pinned at the amplitude level by
/// `amplitude_oracle`. It also cannot separate the
/// phase-space map from the matrix element: a map whose weight and density were
/// both wrong by one factor would integrate correctly.
/// What the fiducial transfer bound is worth on the one enforced row whose map it
/// narrows.
///
/// The peripheral channels of `p p → ℓ⁺ℓ⁻ j` draw their momentum transfer over the
/// window `t ≤ −pT_min²` rather than up to the collinear edge, which is where the
/// jet cut stops accepting anyway. Both maps are unbiased estimators of the same
/// `σ̂` — the bound narrows support the cuts already reject — so the comparison
/// reads two things: the quoted error per seed, which is what the bound buys, and
/// the agreement of the two means, which is what says the narrowing renounced
/// nothing the integrand lives on.
///
/// The same seeds and the same budget on both arms, so the difference is the map
/// and nothing else. Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_fiducial_bound_on_llj_fixed() {
    let run_dir = validation_dir().join("output/pp_to_llj_fixed");
    let rc = RunCard::parse_file(&run_dir.join("Cards/run_card.dat")).expect("banked run card");
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

    eprintln!("── pp_to_llj_fixed: the fiducial transfer bound (MG {mg:.3} ± {mg_err:.3} pb) ──");
    for bound in [true, false] {
        let mut summary = Vec::new();
        let mut runs: Vec<SeedResult> = Vec::new();
        for &seed in LLJ_SEEDS {
            let (sigma, err) = run_seed_mapped(
                &groups,
                &amps,
                &model,
                &evaluated,
                &set,
                &pdf,
                &rc,
                (LLJ_ADAPT_SURVEY, LLJ_ADAPT_ITERS, LLJ_NEVAL, LLJ_NITER),
                seed,
                true,
                &mut summary,
                bound,
            );
            runs.push(SeedResult {
                seed,
                sigma_pb: sigma,
                sigma_err_pb: err,
            });
        }
        let (mean, mean_err, chi2) = combine_seeds(&runs);
        let pull = (mean - mg) / (mean_err * mean_err + mg_err * mg_err).sqrt();
        eprintln!(
            "  bound {:>3}: σ = {mean:.4} ± {mean_err:.4} pb (χ²/dof {chi2:.2}) | rel {:+.4} | \
             pull {pull:+.2} | per seed {}",
            if bound { "on" } else { "off" },
            mean / mg - 1.0,
            runs.iter()
                .map(|r| format!("{:.3}±{:.3}", r.sigma_pb, r.sigma_err_pb))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
}

#[test]
fn sigma_llj_fixed_scale_vs_mg() {
    let clock = Stopwatch::start();
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
        "three seeds at 150k in this layer; the full seed sweep and the budget \
         ladder are oracle-layer"
            .to_string(),
    );
    row.duration_s = Some(clock.seconds());
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

/// Seed sweep and budget ladder for the fixed-scale ℓℓj row — what says its
/// gate budget is a rung the estimator has stopped moving on rather than the
/// cheapest one that happens to agree.
///
/// The dynamical row has the same ladder next door
/// ([`probe_llj_dyn_budget_ladder`]); this one is its control, with the
/// per-event scale prescription out of the integrand. Five seeds a rung, so a
/// rung's χ²/dof is a statement about the estimator rather than about one seed.
/// Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_llj_fixed_budget_ladder() {
    let run_dir = validation_dir().join("output/pp_to_llj_fixed");
    let rc = RunCard::parse_file(&run_dir.join("Cards/run_card.dat")).expect("banked run card");
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

    eprintln!("── pp_to_llj_fixed: MG {mg:.3} ± {mg_err:.3} pb ──");
    for neval in [75_000usize, 150_000, 300_000, 600_000] {
        let mut summary = Vec::new();
        let mut runs: Vec<SeedResult> = Vec::new();
        for &seed in &[20260730u64, 20260731, 20260732, 20260733, 20260734] {
            let (sigma, err) = run_seed(
                &groups,
                &amps,
                &model,
                &evaluated,
                &set,
                &pdf,
                &rc,
                (LLJ_ADAPT_SURVEY, LLJ_ADAPT_ITERS, neval, LLJ_NITER),
                seed,
                true,
                &mut summary,
            );
            runs.push(SeedResult {
                seed,
                sigma_pb: sigma,
                sigma_err_pb: err,
            });
        }
        let (mean, mean_err, chi2) = combine_seeds(&runs);
        let pull = (mean - mg) / (mean_err * mean_err + mg_err * mg_err).sqrt();
        eprintln!(
            "  neval {neval:>7}: σ = {mean:.4} ± {mean_err:.4} pb (χ²/dof {chi2:.2}) | \
             rel {:+.4} | pull {pull:+.2} | per seed {}",
            mean / mg - 1.0,
            runs.iter()
                .map(|r| format!("{:.2}±{:.2}", r.sigma_pb, r.sigma_err_pb))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
}

/// The accepted-point channel floor on the hadronic sampler: what it costs, what
/// coverage it realises, and the cross section it drives.
///
/// `p p > l+ l- j` is the narrow-split control for the same measurement the σ
/// gate's `probe_accepted_point_floor` takes on the `2 -> 6` rows: 24 channels
/// rather than hundreds, and an acceptance the draw-performance work measured at
/// 23.8% untrained. The five gate seeds at the gate budget, so the σ printed is
/// the statistic the row is enforced on. Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_accepted_point_floor_hadronic() {
    use vibegraph::budget::{BlockAllocation, Budget, StopSignal};
    use vibegraph::phasespace::GEV2_TO_PB;

    let run_dir = validation_dir().join("output/pp_to_llj_fixed");
    let rc = RunCard::parse_file(&run_dir.join("Cards/run_card.dat")).expect("banked run card");
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

    eprintln!("-- pp_to_llj_fixed: MG {mg:.3} +- {mg_err:.3} pb, {LLJ_NEVAL} x {LLJ_NITER} --");
    let mut runs: Vec<SeedResult> = Vec::new();
    for &seed in LLJ_SEEDS {
        let mut integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
            .expect("hadronic integrand");
        integ
            .use_run_card_scales(&model, &evaluated, &rc, Some(&set.info.alpha_s))
            .expect("run card scale prescription compiles");
        integ.adapt_alphas(seed, LLJ_ADAPT_SURVEY, LLJ_ADAPT_ITERS, 0.5);
        let (_, result, spend) = integ.adapt_grids_budget(
            Budget::Fixed {
                neval: LLJ_NEVAL,
                niter: LLJ_NITER,
            },
            BlockAllocation::ByAlpha,
            seed,
            &StopSignal::default(),
        );
        let sigma_pb = result.integral * GEV2_TO_PB;
        let sigma_err_pb = result.std_dev * GEV2_TO_PB;
        eprintln!(
            "  seed {seed:>10}: sigma {sigma_pb:.4} +- {sigma_err_pb:.4} pb | rel {:+.4} \
             | chi2/dof {:6.2}",
            sigma_pb / mg - 1.0,
            result.chi2_per_dof,
        );
        eprintln!("    {}", common::floor_coverage_line(&spend));
        runs.push(SeedResult {
            seed,
            sigma_pb,
            sigma_err_pb,
        });
    }
    let (mean, mean_err, chi2) = combine_seeds(&runs);
    let sd = (runs
        .iter()
        .map(|r| (r.sigma_pb - mean).powi(2))
        .sum::<f64>()
        / (runs.len() - 1) as f64)
        .sqrt();
    eprintln!(
        "  -> 5-seed sigma {mean:.4} +- {mean_err:.4} pb (chi2/dof {chi2:.2}) | rel {:+.4} \
         | seed sd/sigma {:.4}",
        mean / mg - 1.0,
        sd / mean,
    );
}

/// What the α-survey budget buys on the hadronic sampler: the converged
/// selection weights, and the cross section they drive, as a function of
/// `n_survey`.
///
/// The fixed-energy sibling of this measurement is `probe_alpha_survey_budget` in
/// the σ gate; both estimators are the same Kleiss–Pittau one over separate code,
/// and the pair is what says a verdict about the survey budget is about the
/// estimator rather than about one implementation.
///
/// α is surveyed once per rung at a single seed and then held while five
/// independent seeds integrate under it — deliberately unlike the gate, which
/// re-surveys per seed. Holding α is what makes the seed spread a measurement of
/// the *integration* noise the rungs must be compared against, instead of folding
/// the survey's own noise back into it.
///
/// The second row carries no banked σ, so it contributes α stability only: it is
/// the several-hundred-channel hadronic case, where the survey cap binds hardest.
/// Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_alpha_survey_budget_hadronic() {
    use vibegraph::budget::{BlockAllocation, Budget, StopSignal};
    use vibegraph::phasespace::GEV2_TO_PB;

    const SURVEYS: [usize; 4] = [10_000, 40_000, 160_000, 640_000];
    const SURVEY_ITERS: usize = 6;
    const SURVEY_SEED: u64 = 20260730;

    let run_dir = validation_dir().join("output/pp_to_llj_fixed");
    let rc = RunCard::parse_file(&run_dir.join("Cards/run_card.dat")).expect("banked run card");
    let (mg, mg_err) = banked_llj_sigma(&run_dir);
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let set = load_pdf_set();
    let pdf = set.member(0).expect("PDF member 0");

    for (label, process, banked) in [
        ("pp_to_llj_fixed", LLJ_PROCESS, Some((mg, mg_err))),
        ("pp_to_lljj", "p p > l+ l- j j", None),
    ] {
        let groups = groups_for(process, &model, &evaluated, &rc);
        let amps: Vec<BoundAmplitude<f64>> = groups
            .groups()
            .iter()
            .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), &evaluated))
            .collect();
        match banked {
            Some((mg, mg_err)) => eprintln!(
                "── {label} ({process}): MG {mg:.3} ± {mg_err:.3} pb, \
                 driven at {LLJ_NEVAL} × {LLJ_NITER} ──"
            ),
            None => eprintln!("── {label} ({process}): no banked σ, α stability only ──"),
        }
        let mut converged: Vec<(usize, Vec<f64>)> = Vec::new();
        for n_survey in SURVEYS {
            let clock = Stopwatch::start();
            let mut integ =
                ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
                    .expect("hadronic integrand");
            integ
                .use_run_card_scales(&model, &evaluated, &rc, Some(&set.info.alpha_s))
                .expect("run card scale prescription compiles");
            let adaptation = integ.adapt_alphas(SURVEY_SEED, n_survey, SURVEY_ITERS, 0.5);
            let steps: Vec<String> = adaptation
                .trajectory
                .windows(2)
                .map(|w| format!("{:.2e}", l1_distance(&w[0], &w[1])))
                .collect();
            eprintln!(
                "  n_survey {n_survey:>7} ({:>4.0} s, {} channels): α steps (L1) {}",
                clock.seconds(),
                integ.channel_count(),
                steps.join(" "),
            );
            if let Some((mg, _)) = banked {
                let mut runs: Vec<SeedResult> = Vec::new();
                for &seed in LLJ_SEEDS {
                    let (_, result, spend) = integ.adapt_grids_budget(
                        Budget::Fixed {
                            neval: LLJ_NEVAL,
                            niter: LLJ_NITER,
                        },
                        BlockAllocation::ByAlpha,
                        seed,
                        &StopSignal::default(),
                    );
                    let sigma_pb = result.integral * GEV2_TO_PB;
                    let sigma_err_pb = result.std_dev * GEV2_TO_PB;
                    eprintln!(
                        "      seed {seed:>10}: σ {sigma_pb:.4} ± {sigma_err_pb:.4} pb \
                         | rel {:+.4} | χ²/dof {:6.2} | achieved_rel {:.5} \
                         | scaled_rel {:.5} (×{:.1})",
                        sigma_pb / mg - 1.0,
                        result.chi2_per_dof,
                        spend.achieved_rel,
                        spend.scaled_rel,
                        spend.scaled_rel / spend.achieved_rel,
                    );
                    runs.push(SeedResult {
                        seed,
                        sigma_pb,
                        sigma_err_pb,
                    });
                }
                let (mean, mean_err, chi2) = combine_seeds(&runs);
                let sd = (runs
                    .iter()
                    .map(|r| (r.sigma_pb - mean).powi(2))
                    .sum::<f64>()
                    / (runs.len() - 1) as f64)
                    .sqrt();
                let lo = runs.iter().map(|r| r.sigma_pb).fold(f64::MAX, f64::min);
                let hi = runs.iter().map(|r| r.sigma_pb).fold(f64::MIN, f64::max);
                eprintln!(
                    "    → 5-seed σ {mean:.4} ± {mean_err:.4} pb (χ²/dof {chi2:.2}) \
                     | rel {:+.4} | seed sd/σ {:.4} | full spread {:.4}",
                    mean / mg - 1.0,
                    sd / mean,
                    (hi - lo) / mean,
                );
            }
            converged.push((n_survey, integ.channel_alphas()));
        }
        report_alpha_stability(&converged);
    }
}

/// `Σⱼ |aⱼ − bⱼ|` between two selection-weight vectors: both sum to one, so this
/// is twice the total variation between the mixtures they define.
fn l1_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| (x - y).abs()).sum()
}

/// Each survey budget's converged α against the highest budget's, which is the
/// closest thing to the fixed point the experiment measures.
///
/// Three readings, because a wide split hides movement three ways: the L1
/// distance is the whole mixture's, `worst αⱼ ratio` is the largest *relative*
/// move over the channels that carry weight — a channel at `10⁻³` moving by half
/// is invisible in L1 — and the leading-channel shares say whether the mixture's
/// concentration itself moved.
fn report_alpha_stability(converged: &[(usize, Vec<f64>)]) {
    let Some((top_n, top)) = converged.last() else {
        return;
    };
    for (n, a) in converged {
        let carrying = |x: &f64| *x > 1e-6;
        let worst = a
            .iter()
            .zip(top)
            .filter(|(x, y)| carrying(x) || carrying(y))
            .map(|(x, y)| (x / y).max(y / x))
            .fold(0.0_f64, f64::max);
        let mut sorted = a.clone();
        sorted.sort_by(|p, q| q.partial_cmp(p).expect("selection weights are numbers"));
        eprintln!(
            "  α({n:>7}) vs α({top_n}): L1 {:.4e} | worst αⱼ ratio {worst:8.2} \
             | top-1 {:.4} | top-10 {:.4}",
            l1_distance(a, top),
            sorted[0],
            sorted.iter().take(10).sum::<f64>(),
        );
    }
}

/// The run whose card is `pp_to_llj_fixed`'s with the three `fixed_*_scale`
/// switches turned off, and nothing else changed.
const LLJ_DYN_RUN: &str = "pp_to_llj_dyn";

/// The Drell–Yan run the draw-cost probe prices alongside the `ℓ⁺ℓ⁻ j` one.
const DY_DRAW_RUN: &str = "pp_to_ll";

/// Whether this checkout has the dynamical run, given that the manifest declares
/// it absent from the pinned bundle.
///
/// The two halves are each other's control: a row the bundle does not carry may
/// be missing, and one it does carry may not. Without the first, a fetching
/// checkout fails on a run nothing promised it; without the second, an incomplete
/// work area passes silently.
fn dyn_run_present(gate: &str, run: &str) -> bool {
    if validation_dir()
        .join("output")
        .join(run)
        .join("Cards/run_card.dat")
        .is_file()
    {
        return true;
    }
    if common::manifest::unbundled_rows().contains(run) {
        eprintln!(
            "[{run}] AWAITING BUNDLE (the manifest marks this row bundled = false and this \
             checkout does not have its run, so no cell is written for it)"
        );
        return false;
    }
    vibegraph::validation::require(gate, "a banked run directory", run);
}

/// σ(p p → ℓ⁺ℓ⁻ j) at MadGraph's *dynamical* scale choice, against the banked
/// `pp_to_llj_dyn` run.
///
/// The one row where the scale prescription is the only thing under test. The two
/// run cards differ in the three `fixed_*_scale` booleans and in nothing else —
/// same process, same beams, same cuts, same PDF set, same param card — and the
/// fixed-scale row above is enforced at `0.5%` against its own reference. So a
/// disagreement here is the kT clustering's `μR` and per-beam `μF`, the coupling
/// read at them and the densities read at them, and cannot be the amplitudes, the
/// flavour decomposition, the phase-space map or the cuts: those are held fixed
/// by a passing row.
///
/// Each point is clustered in the integration configuration drawn from its own
/// squared amplitudes, inside the flavour group that produced it — the groups of
/// this process do not share a merge graph, so which group is asked matters as
/// much as which configuration. The row has read three numbers under three rules:
/// `−3.05%` while every point was clustered in channel 1, `−0.68%` while the
/// scale was read in the channel the *sampler* drew the point in, and `+0.25%`
/// now. The budget ladder in [`probe_llj_dyn_budget_ladder`] gives `416.23`,
/// `416.27`, `416.26`, `416.13` pb over five seeds at `75k`, `150k`, `300k` and
/// `600k` against MadGraph's `415.42 ± 1.36` — flat to `0.04%` across an
/// eightfold budget, so what is left is the reference's own error and not a
/// budget the row has yet to spend.
///
/// **The channel partition is what the middle number was, and it is gone.** Once
/// the scale reads the integration channel, σ is no longer independent of the
/// multichannel selection weights: `αⱼ` decides which scale a region of phase
/// space is evaluated at, not only how often it is visited. Drawing the
/// configuration per point removes `αⱼ` from that decision, and
/// `validate_sigma`'s `probe_channel_partition_moves_sigma` measures the
/// difference directly on the partonic rows: `1.5%` between this crate's
/// converged and uniform partitions before, `1.9e-3` and `1.5e-3` after, against
/// a `1.6e-3` Monte-Carlo error.
///
/// The **pull is asserted**. It was reported and not asserted while the residual
/// was a systematic of about `0.6%`, whose pull grows without bound as this
/// side's budget rises; five seeds now sit at `χ²/dof 0.65` with the mean
/// `0.11σ` from the reference, which is what the statistic is for.
///
/// What it cannot see: everything a scalar integrates over. The per-event scale
/// enters σ through an average, so a clustering that got individual events wrong
/// while preserving that average would pass — which is why `validate_scales`
/// replays every banked event's `SCALUP`, `<rscale>` and `<pdfrwt>` against
/// MadGraph's own printed values, and why this row is the end-to-end statement
/// rather than the fine one.
#[test]
fn sigma_llj_dynamical_scale_vs_mg() {
    if !dyn_run_present("sigma_llj_dynamical_scale_vs_mg", LLJ_DYN_RUN) {
        return;
    }
    let clock = Stopwatch::start();
    let run_dir = validation_dir().join("output").join(LLJ_DYN_RUN);
    let rc = RunCard::parse_file(&run_dir.join("Cards/run_card.dat")).expect("banked run card");
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
        let (sigma, err) = run_seed_shaped(
            &groups,
            &amps,
            &model,
            &evaluated,
            &set,
            &pdf,
            &rc,
            (LLJ_ADAPT_SURVEY, LLJ_ADAPT_ITERS, LLJ_NEVAL, LLJ_NITER),
            seed,
            true,
            &mut summary,
            true,
            ScaleShape::PerEvent,
        );
        eprintln!(
            "[llj_dyn seed {seed}] vibegraph σ = {sigma:.3} ± {err:.3} pb | \
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
        "[llj_dyn] GATE vibegraph σ = {mean:.3} ± {mean_err:.3} pb ({} seeds, \
         χ²/dof = {chi2:.2}) | MG σ = {mg:.3} ± {mg_err:.3} pb | \
         pull = {pull:+.2} | rel = {rel:+.4}",
        runs.len()
    );

    let ok = rel.abs() < LLJ_DYN_MAX_REL && chi2 < LLJ_MAX_CHI2_PER_DOF;
    let mut row = IntegralsRow::new(LLJ_DYN_RUN, LLJ_PROCESS, "gate");
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
        "each point clustered in the integration configuration drawn from its own \
         AMP2, inside the flavour group that produced it. Three seeds at 150k here; \
         the five-seed budget ladder that says this is converged rather than \
         under-sampled is oracle-layer, and gives 416.23, 416.27, 416.26, 416.13 pb \
         at 75k, 150k, 300k and 600k against MadGraph's 415.42 +- 1.36 -- flat to 0.04% \
         across an eightfold budget, at chi2/dof 2.59, 0.66, 0.75, 0.29, the 75k rung's \
         scatter being why the gate does not run there. rel_tol 0.005 is the \
         reference's own 0.33% with headroom; the pull is asserted, the channel-partition systematic that made it \
         the wrong statistic having been retired with the draw"
            .to_string(),
    );
    row.duration_s = Some(clock.seconds());
    row.write();

    assert!(
        rel.abs() < LLJ_DYN_MAX_REL,
        "[llj_dyn] σ disagreement: vibegraph {mean:.3}±{mean_err:.3} vs \
         MG {mg:.3}±{mg_err:.3} pb, rel = {rel:+.4} > {LLJ_DYN_MAX_REL}"
    );
    assert!(
        pull.abs() < 3.0,
        "[llj_dyn] σ pull: vibegraph {mean:.3}±{mean_err:.3} vs \
         MG {mg:.3}±{mg_err:.3} pb, pull = {pull:+.2}"
    );
    assert!(
        chi2 < LLJ_MAX_CHI2_PER_DOF,
        "[llj_dyn] the seeds scatter by more than they claim: \
         χ²/dof = {chi2:.2} over {runs:?}"
    );
}

/// Seed sweep and budget ladder for the dynamical ℓℓj row, the evidence its
/// recorded disagreement rests on.
///
/// Budget convergence has to be established here rather than assumed, because
/// the dynamical row's integrand carries a per-event coupling the fixed-scale
/// one does not: a scale prescription wrong in a phase-space-dependent way would
/// produce a bias that does *not* shrink with budget, which is exactly what the
/// ladder separates from one that does.
///
/// Both axes, five seeds each: a seed sweep is the floor and budget convergence
/// is the second. Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_llj_dyn_budget_ladder() {
    if !dyn_run_present("probe_llj_dyn_budget_ladder", LLJ_DYN_RUN) {
        return;
    }
    let run_dir = validation_dir().join("output").join(LLJ_DYN_RUN);
    let rc = RunCard::parse_file(&run_dir.join("Cards/run_card.dat")).expect("banked run card");
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

    eprintln!("── {LLJ_DYN_RUN}: MG {mg:.3} ± {mg_err:.3} pb ──");
    for neval in [75_000usize, 150_000, 300_000, 600_000] {
        let mut summary = Vec::new();
        let mut runs: Vec<SeedResult> = Vec::new();
        for &seed in &[20260730u64, 20260731, 20260732, 20260733, 20260734] {
            let (sigma, err) = run_seed_shaped(
                &groups,
                &amps,
                &model,
                &evaluated,
                &set,
                &pdf,
                &rc,
                (LLJ_ADAPT_SURVEY, LLJ_ADAPT_ITERS, neval, LLJ_NITER),
                seed,
                true,
                &mut summary,
                true,
                ScaleShape::PerEvent,
            );
            runs.push(SeedResult {
                seed,
                sigma_pb: sigma,
                sigma_err_pb: err,
            });
        }
        let (mean, mean_err, chi2) = combine_seeds(&runs);
        let pull = (mean - mg) / (mean_err * mean_err + mg_err * mg_err).sqrt();
        eprintln!(
            "  neval {neval:>7}: σ = {mean:.4} ± {mean_err:.4} pb (χ²/dof {chi2:.2}) | \
             rel {:+.4} | pull {pull:+.2} | per seed {}",
            mean / mg - 1.0,
            runs.iter()
                .map(|r| format!("{:.2}±{:.2}", r.sigma_pb, r.sigma_err_pb))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
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
/// VEGAS budget per seed, taken from a measured ladder rather than from cost, and
/// sized against the reference's own precision: no agreement finer than
/// MadGraph's Monte-Carlo error on this run is visible to the comparison, so
/// points spent past it buy the gate nothing.
///
/// Five seeds a rung (`probe_bb_budget_ladder`) read `+0.15%`, `+0.16%`, `+0.11%`,
/// `+0.14%` at 75 000, 150 000, 300 000 and 600 000 points an iteration, at χ²/dof
/// `0.63`, `0.79`, `0.68`, `0.06` — an `0.05%` span with no direction to it across
/// an eightfold budget, and no rung whose seeds scatter by more than they claim.
/// So this row is converged at the rung it runs at, and the seed sweep is
/// measuring an agreement rather than this crate's convergence. The ℓℓj rows,
/// which spread the same budget over 24 pooled channels and so reach a converged
/// rung much later, are the reason that is checked rather than assumed.
///
/// At `75 000` three seeds leave this side's error at `0.076%` against the banked
/// run's own `0.182%`, so the combined error of the two sides is still dominated
/// by the reference.
const BB_NEVAL: usize = 75_000;
const BB_NITER: usize = 10;
/// Largest relative distance from the banked MadGraph σ the `b b̄` sweep may show.
///
/// MadGraph's own Monte-Carlo error on the banked run is `0.182%`, which is the
/// floor — no agreement tighter than the reference's precision is meaningful —
/// and this side's is `0.076%` at the gate budget, so `0.005` is two and a half
/// times the two sides' combined `0.197%`. The whole measured budget family sits
/// inside `0.16%`.
const BB_MAX_REL: f64 = 0.005;
/// Scatter the seeds are allowed about their own mean, in units of their own
/// quoted errors — the guard the scalar pull cannot be, since a run that missed a
/// region reports a small integral *and* a small error. Measured `0.63`, `0.79`,
/// `0.68`, `0.06` over five seeds a rung of the same budget ladder.
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
/// **Several seeds, not one**, for the reason the ℓℓj sweep gives: an
/// under-sampled region is reported as a confident number.
///
/// What it cannot see: everything σ integrates over — per-diagram phases,
/// colour-flow relabellings, helicity-by-helicity errors — and, being a single
/// scalar, a mis-sampled region of small measure, which the seed scatter and not
/// the pull is what guards.
#[test]
fn sigma_bb_fixed_scale_vs_mg() {
    let clock = Stopwatch::start();
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
        "three seeds at 75k in this layer; the five-seed 75k to 600k ladder \
         (probe_bb_budget_ladder) reads +0.15% / +0.16% / +0.11% / +0.14% at \
         chi2/dof 0.63 / 0.79 / 0.68 / 0.06, an 0.05% span with no direction to \
         it, so the gate runs at the lowest rung the ladder resolves as converged"
            .to_string(),
    );
    row.duration_s = Some(clock.seconds());
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

/// The budget ladder behind [`BB_NEVAL`], five seeds a rung.
///
/// The gate reads three seeds at one budget, and that pair of numbers cannot
/// separate an agreement from an estimator still moving: VEGAS's inverse-variance
/// iteration combination reports an under-sampled region as a confident value,
/// error bar included. What tells the two apart is whether σ moves with the
/// budget, so the rungs are printed rather than summarised, and five seeds a rung
/// makes each rung's χ²/dof a statement about the estimator rather than about one
/// seed.
///
/// Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_bb_budget_ladder() {
    let run_dir = validation_dir().join("output/pp_to_bb_fixed");
    let rc = RunCard::parse_file(&run_dir.join("Cards/run_card.dat")).expect("banked run card");
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

    eprintln!("── pp_to_bb_fixed: MG {mg:.6e} ± {mg_err:.3e} pb ──");
    for neval in [75_000usize, 150_000, 300_000, 600_000] {
        let mut summary = Vec::new();
        let mut runs: Vec<SeedResult> = Vec::new();
        for &seed in &[20260801u64, 20260802, 20260803, 20260804, 20260805] {
            let (sigma, err) = run_seed(
                &groups,
                &amps,
                &model,
                &evaluated,
                &set,
                &pdf,
                &rc,
                (BB_ADAPT_SURVEY, BB_ADAPT_ITERS, neval, BB_NITER),
                seed,
                true,
                &mut summary,
            );
            runs.push(SeedResult {
                seed,
                sigma_pb: sigma,
                sigma_err_pb: err,
            });
        }
        let (mean, mean_err, chi2) = combine_seeds(&runs);
        let pull = (mean - mg) / (mean_err * mean_err + mg_err * mg_err).sqrt();
        eprintln!(
            "  neval {neval:>7}: σ = {mean:.6e} ± {mean_err:.3e} pb (χ²/dof {chi2:.2}) | \
             rel {:+.4} | pull {pull:+.2} | per seed {}",
            mean / mg - 1.0,
            runs.iter()
                .map(|r| format!("{:.5e}", r.sigma_pb))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
}

// ──────────────────────────── p p > j j ─────────────────────────────────────

/// The process of the banked `pp_to_jj` run, spelled as its `.mg5` script spells it.
const JJ_PROCESS: &str = "p p > j j";

/// The banked run directory.
const JJ_RUN: &str = "pp_to_jj";

/// Independent seeds the `j j` cross section is measured on, for the reason the
/// ℓℓj sweep gives.
const JJ_SEEDS: &[u64] = &[20260810, 20260811, 20260812];
/// Points per survey iteration, and iterations, of the channel-weight adaptation.
const JJ_ADAPT_SURVEY: usize = 8_000;
const JJ_ADAPT_ITERS: usize = 5;
/// VEGAS budget per seed, taken from a measured ladder rather than from cost, and
/// sized against the reference's own precision rather than below it: the gate's
/// resolving power is `√(σ_ours² + σ_MG²)`, so once this side's error is well
/// inside MadGraph's `0.22 %` on this run, further points buy the comparison
/// nothing it can see.
///
/// Over five seeds at 75 000, 150 000, 300 000 and 600 000 points an iteration
/// (`probe_jj_budget_ladder`) this row reads `+0.33 %`, `+0.26 %`, `+0.25 %`,
/// `+0.30 %` at χ²/dof `1.40`, `0.44`, `0.82`, `1.21` — an `0.08 %` span with no
/// direction to it, a third of the reference's own Monte-Carlo error, and no rung
/// whose seeds scatter by more than they claim. A `2 → 2` final state gives every
/// channel far more points per iteration than the 24-channel ℓℓj rows get, which
/// is why this row is flat at budgets where those are still climbing.
///
/// `75 000` is the ladder's lowest rung. Three seeds there leave this side's error
/// at `0.081 %` against the reference's `0.217 %`, so the combined error is still
/// the reference's and the row is compared at the precision the bank was written
/// with.
const JJ_NEVAL: usize = 75_000;
const JJ_NITER: usize = 10;
/// Relative agreement this row is held to.
///
/// Set from the reference's own Monte-Carlo error — `1.4726e6` on `6.7885e8 pb`,
/// `0.22 %` — and not from the channel-partition band the `ℓℓj` rows carry: a
/// `2 → 2` final state gives the clustering no merge to choose, so σ here is a
/// function of the momenta alone and `probe_jj_channel_partition` measures the
/// partition gap at `1.0e-3` against its own `9.6e-4` Monte Carlo. What is left
/// is Monte Carlo, so the pull is asserted too. Measured `+0.33 %` / `+0.26 %` /
/// `+0.25 %` / `+0.30 %` over a `75 000`–`600 000` ladder at five seeds a rung.
const JJ_MAX_REL: f64 = 0.005;
/// Scatter the seeds are allowed about their own mean, in units of their quoted
/// errors — the guard the scalar pull cannot be. The gate budget is the ladder's
/// worst rung and measures `1.40` over five seeds there, against `0.44`, `0.82`
/// and `1.21` at the three rungs above it.
const JJ_MAX_CHI2_PER_DOF: f64 = 4.0;

/// MadGraph's own concrete subprocesses for a banked run, one entry per
/// `leshouche.inc` `IDUP` record, as `(incoming, outgoing)` PDG codes.
///
/// `leshouche.inc` is the file the colour-flow dictionary is already checked
/// against, and it is the only place a run states which concrete flavour
/// assignments its cross section is a sum over. Read from the banked run rather
/// than from a committed list, so the reference cannot drift from the run it
/// describes.
fn madgraph_subprocesses(run: &str, n_in: usize, n_out: usize) -> BTreeSet<(Vec<i32>, Vec<i32>)> {
    let dir = validation_dir()
        .join("output")
        .join(run)
        .join("SubProcesses");
    let mut found = BTreeSet::new();
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.is_dir()
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with('P'))
        })
        .collect();
    dirs.sort();
    assert!(
        !dirs.is_empty(),
        "no P* subprocess directory under {}",
        dir.display()
    );
    for d in dirs {
        let path = d.join("leshouche.inc");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        for line in text.lines() {
            let Some(rest) = line.split_once("DATA (IDUP(") else {
                continue;
            };
            let Some((_, payload)) = rest.1.split_once(")/") else {
                continue;
            };
            let codes: Vec<i32> = payload
                .trim_end()
                .trim_end_matches('/')
                .split(',')
                .map(|f| {
                    f.trim()
                        .parse::<i32>()
                        .expect("an IDUP field is an integer")
                })
                .collect();
            assert_eq!(
                codes.len(),
                n_in + n_out,
                "{} lists an IDUP record of {} legs",
                path.display(),
                codes.len()
            );
            let mut incoming = codes[..n_in].to_vec();
            incoming.sort_unstable();
            found.insert((incoming, codes[n_in..].to_vec()));
        }
    }
    found
}

/// The outgoing-leg orderings MadGraph's banked sample actually emits: how many
/// distinct flavour assignments it carries, how many of those have their outgoing
/// swap emitted too, and — over the assignments whose two outgoing flavours differ
/// — how often the first outgoing leg is the more forward one and how often it is
/// not.
fn banked_outgoing_orderings(run: &str) -> (usize, usize, (usize, usize)) {
    use flate2::read::MultiGzDecoder;
    use std::io::Read;
    use vibegraph::lhef::parse::LheFile;

    let path = validation_dir()
        .join("output")
        .join(run)
        .join("Events/run_01/unweighted_events.lhe.gz");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let mut text = String::new();
    MultiGzDecoder::new(&bytes[..])
        .read_to_string(&mut text)
        .unwrap_or_else(|e| panic!("decompress {}: {e}", path.display()));
    let events = LheFile::parse(&text)
        .expect("MadGraph's own file parses")
        .events;

    let mut patterns: BTreeSet<Vec<i32>> = BTreeSet::new();
    let mut ordered = (0usize, 0usize);
    for ev in &events {
        let codes: Vec<i32> = ev.particles.iter().map(|p| p.pdg).collect();
        assert_eq!(codes.len(), 4, "a 2 -> 2 record carries four legs");
        patterns.insert(codes.clone());
        if codes[2] == codes[3] {
            continue;
        }
        let eta = |p: &vibegraph::lhef::record::LheParticle| {
            let pt = p.momentum[0].hypot(p.momentum[1]);
            if pt > 0.0 {
                (p.momentum[2] / pt).asinh()
            } else {
                0.0
            }
        };
        if eta(&ev.particles[2]) > eta(&ev.particles[3]) {
            ordered.0 += 1;
        } else {
            ordered.1 += 1;
        }
    }
    let swapped = patterns
        .iter()
        .filter(|c| c[2] != c[3] && patterns.contains(&vec![c[0], c[1], c[3], c[2]]))
        .count();
    (patterns.len(), swapped, ordered)
}

/// This crate's concrete subprocesses for a flavour decomposition, in the same
/// `(incoming, outgoing)` shape, with the incoming pair sorted so the two sides
/// agree about a beam ordering neither of them enumerates twice.
fn our_subprocesses(groups: &FlavorGroups) -> Vec<(Vec<i32>, Vec<i32>)> {
    let mut out = Vec::new();
    for g in groups.groups() {
        for m in g.members() {
            let mut incoming = m.incoming.to_vec();
            incoming.sort_unstable();
            out.push((incoming, m.outgoing.clone()));
        }
    }
    out
}

/// The concrete subprocess set of `p p > j j`, against the one MadGraph's own
/// `leshouche.inc` declares: **set equality, at zero tolerance.**
///
/// `p p > j j` is the one banked row whose process card repeats a multiparticle
/// label in the final state, so it is the row that exercises the enumeration's
/// unordered-outgoing rule: a concrete subprocess is identified by the unordered
/// content of each side, and `g u > g u` and `g u > u g` are one subprocess, not
/// two. They have to be, because both would then be summed over the same
/// labelled `dΦ₂` — whose polar angle runs over the whole sphere — and the second
/// is the first relabelled, so enumerating both adds its term twice.
///
/// Three things are asserted, and none of them is implied by the other two:
///
/// - **MadGraph itself lists one representative per unordered assignment.** Of
///   its 65, the 52 whose two outgoing flavours differ each have an outgoing swap
///   that `leshouche.inc` could have listed and does not. Without this the set
///   equality below could hold for the trivial reason that there was never a
///   choice to make.
/// - **The two sets are equal**, so nothing is missing and nothing is surplus —
///   and, since each entry carries its outgoing legs in the order that side
///   enumerated them, the surviving representative is in **MadGraph's own
///   outgoing order** rather than merely being one of the two.
/// - **MadGraph's own 10 000 events say the premise from the other side**: no
///   emitted flavour assignment has its outgoing swap also emitted, and inside a
///   single assignment the two outgoing legs are found in both relative
///   orderings. A single representative covering the whole sphere is what that
///   looks like; two distinct subprocesses is not.
///
/// The negative control is a card whose final-state slots draw on *disjoint*
/// alias sets, where the rule can collapse nothing and must not.
///
/// What it cannot see: whether the *diagrams* of a listed subprocess agree, and
/// anything about the cross section — a set comparison is blind to both. The σ
/// consequence is [`sigma_jj_dynamical_scale_vs_mg`], and its convergence
/// [`probe_jj_budget_ladder`].
#[test]
fn jj_subprocesses_are_madgraphs_own() {
    if !dyn_run_present("jj_subprocesses_are_madgraphs_own", JJ_RUN) {
        return;
    }
    let run_dir = validation_dir().join("output").join(JJ_RUN);
    let rc = RunCard::parse_file(&run_dir.join("Cards/run_card.dat")).expect("banked run card");
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let groups = groups_for(JJ_PROCESS, &model, &evaluated, &rc);

    let mg = madgraph_subprocesses(JJ_RUN, 2, 2);
    let ours = our_subprocesses(&groups);
    let ours_set: BTreeSet<(Vec<i32>, Vec<i32>)> = ours.iter().cloned().collect();
    assert_eq!(
        ours.len(),
        ours_set.len(),
        "the flavour decomposition lists a concrete subprocess twice"
    );

    // MadGraph's own side of the rule, so the equality below is not trivially
    // satisfiable: every assignment with unequal outgoing flavours is one whose
    // swap `leshouche.inc` could have carried, and none of them is there.
    let mg_unequal_outgoing = mg.iter().filter(|(_, out)| out[0] != out[1]).count();
    let mg_with_swap = mg
        .iter()
        .filter(|(inc, out)| out[0] != out[1] && mg.contains(&(inc.clone(), vec![out[1], out[0]])))
        .count();

    let missing: Vec<_> = mg.difference(&ours_set).cloned().collect();
    let surplus: Vec<_> = ours_set.difference(&mg).cloned().collect();
    eprintln!(
        "[{JJ_RUN}] concrete subprocesses: MadGraph {} ({} with unequal outgoing flavours, \
         {} of those with the swap also listed), this side {} — {} missing, {} surplus",
        mg.len(),
        mg_unequal_outgoing,
        mg_with_swap,
        ours_set.len(),
        missing.len(),
        surplus.len(),
    );

    assert!(
        mg_unequal_outgoing > 0,
        "no MadGraph assignment has unequal outgoing flavours, so this run cannot \
         exercise the unordered-outgoing rule at all"
    );
    assert_eq!(
        mg_with_swap, 0,
        "MadGraph's own leshouche.inc lists both outgoing orderings of an assignment, \
         so it does not keep one representative per unordered assignment after all"
    );
    assert!(
        missing.is_empty(),
        "MadGraph sums over {} concrete subprocess(es) this side does not enumerate: {missing:?}",
        missing.len()
    );
    assert!(
        surplus.is_empty(),
        "this side enumerates {} concrete subprocess(es) MadGraph does not sum over: {surplus:?}",
        surplus.len()
    );
    // Redundant with the two differences taken together, and kept because it is
    // the statement the row rests on: the representative this side keeps is the
    // one MadGraph writes, legs in the same order, so an event record built on
    // its legs needs no reordering.
    assert_eq!(
        ours_set, mg,
        "the concrete subprocess sets differ, so this side does not sum over MadGraph's \
         own assignments in MadGraph's own outgoing order"
    );

    // The premise, read off MadGraph's own events rather than off the reading of
    // `dΦ₂`: if the two orderings were distinct subprocesses there, both would
    // appear in the sample, and the region the second one would cover would be
    // missing from the first. Neither is so — no emitted assignment's outgoing
    // swap is also emitted, and inside a single assignment the two outgoing legs
    // are found in both relative orderings.
    let (patterns, swapped_pairs, both_orderings) = banked_outgoing_orderings(JJ_RUN);
    eprintln!(
        "[{JJ_RUN}] MadGraph's banked sample: {patterns} distinct emitted flavour \
         assignments, {swapped_pairs} of them with their outgoing swap also emitted; \
         inside the unequal-outgoing ones the legs are ordered {} / {} either way",
        both_orderings.0, both_orderings.1,
    );
    assert_eq!(
        swapped_pairs, 0,
        "MadGraph emits both outgoing orderings of a flavour assignment, so the two are \
         distinct subprocesses there after all"
    );
    assert!(
        both_orderings.0 > 0 && both_orderings.1 > 0,
        "MadGraph's events put the two outgoing legs in one relative order only, so its \
         single representative does not visibly cover the other"
    );

    // The negative control: a card whose final-state slots draw on *disjoint*
    // alias sets cannot produce two ordered assignments with the same multiset, so
    // the unordered-outgoing rule has nothing to collapse there and must leave the
    // count alone. Applying the same key on top of the enumerated sets is how that
    // is measured — it drops nothing. Without it, the rule could be silently
    // merging assignments across every process rather than only where a label
    // repeats.
    let control_dir = validation_dir().join("output/pp_to_llj_fixed");
    let control_rc =
        RunCard::parse_file(&control_dir.join("Cards/run_card.dat")).expect("banked run card");
    let opts = ParsingOptions::default();
    let card = parse_proc_card(&format!("generate {LLJ_PROCESS}"), &opts).expect("proc card");
    let control_sets = generate_from_proc_card(&card, &model).expect("enumeration");
    let enumerated = control_sets.len();
    let mut seen: BTreeSet<(Vec<String>, Vec<String>)> = BTreeSet::new();
    let collapsed = control_sets
        .iter()
        .filter(|s| {
            let mut key = (s.particles_in.clone(), s.particles_out.clone());
            key.0.sort();
            key.1.sort();
            seen.insert(key)
        })
        .count();
    let control_groups = derive_flavor_groups(control_sets, &model, &evaluated, &control_rc)
        .expect("flavour groups");
    let control_members: usize = control_groups
        .groups()
        .iter()
        .map(|g| g.members().len())
        .sum();
    eprintln!(
        "[pp_to_llj_fixed] control: {enumerated} enumerated sets, {collapsed} surviving the \
         same key, {control_members} of them carrying diagrams"
    );
    assert_eq!(
        collapsed, enumerated,
        "two enumerated sets of a process whose final-state slots draw on disjoint alias \
         sets share an unordered final state, so the rule is merging more than a repeated \
         label"
    );
}

/// Seed sweep and budget ladder for the `j j` row — the two axes the gate's
/// tolerance is set from.
///
/// Neither axis alone is evidence: a seed sweep cannot see a bias shared by every
/// seed (an under-sampled region is reported *confidently*), and a single budget
/// cannot tell a converged estimator from one still climbing.
///
/// What it cannot see: whether the estimator is asymptotic. The climb across an
/// eightfold budget is smaller than the reference's own Monte-Carlo error, so the
/// ladder resolves the row as converged *at the scale the comparison is made at*
/// and no finer — which is why the gate's `rel_tol` is set at that scale rather
/// than at the ladder's residual.
///
/// Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_jj_budget_ladder() {
    if !dyn_run_present("probe_jj_budget_ladder", JJ_RUN) {
        return;
    }
    let run_dir = validation_dir().join("output").join(JJ_RUN);
    let rc = RunCard::parse_file(&run_dir.join("Cards/run_card.dat")).expect("banked run card");
    let (mg, mg_err) = banked_llj_sigma(&run_dir);
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let set = load_pdf_set();
    let pdf = set.member(0).expect("PDF member 0");
    let seeds = [20260810u64, 20260811, 20260812, 20260813, 20260814];

    eprintln!("── {JJ_RUN}: MG {mg:.6e} ± {mg_err:.3e} pb ──");
    let groups = groups_for(JJ_PROCESS, &model, &evaluated, &rc);
    let amps: Vec<BoundAmplitude<f64>> = groups
        .groups()
        .iter()
        .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), &evaluated))
        .collect();
    for &neval in &[75_000usize, 150_000, 300_000, 600_000] {
        let mut summary = Vec::new();
        let mut runs: Vec<SeedResult> = Vec::new();
        for &seed in &seeds {
            let (sigma, err) = run_seed_shaped(
                &groups,
                &amps,
                &model,
                &evaluated,
                &set,
                &pdf,
                &rc,
                (JJ_ADAPT_SURVEY, JJ_ADAPT_ITERS, neval, JJ_NITER),
                seed,
                true,
                &mut summary,
                true,
                ScaleShape::PerEvent,
            );
            runs.push(SeedResult {
                seed,
                sigma_pb: sigma,
                sigma_err_pb: err,
            });
        }
        let (mean, mean_err, chi2) = combine_seeds(&runs);
        let pull = (mean - mg) / (mean_err * mean_err + mg_err * mg_err).sqrt();
        eprintln!(
            "  neval {neval:>7}: σ = {mean:.6e} ± {mean_err:.3e} pb \
             (χ²/dof {chi2:.2}) | rel {:+.4} | pull {pull:+.2} | per seed {}",
            mean / mg - 1.0,
            runs.iter()
                .map(|r| format!("{:.5e}", r.sigma_pb))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
}

/// The channel-partition ambiguity on `p p → j j`, measured rather than inherited.
///
/// Once the cluster scale is read in the channel a point was drawn in, σ stops
/// being independent of the multichannel selection weights `αⱼ`: they decide
/// *which* scale a region of phase space is evaluated at, not merely how often it
/// is visited. The size of that is a property of the process, so it is measured
/// here rather than inherited from the partonic rows — and it comes out at this
/// row's own Monte-Carlo error, which is why `JJ_MAX_REL` is set from the
/// reference's error and not from a partition band.
///
/// It is small here for a structural reason rather than by luck: the cluster
/// scale depends on the integration channel only through which merge sequence the
/// channel's forest admits, and a `2 → 2` final state offers no merge to choose —
/// the clustering's terminal core is the event itself, so `μR` and both `μF` are
/// functions of the momenta alone.
///
/// Both arms are measured, at the converged `αⱼ` and at uniform `αⱼ`
/// (`n_adapt_iter = 0`) with everything else held, one seed each.
///
/// **`pp_to_llj_fixed` is the negative control**, on the same hadronic path with
/// the same instrument: its card fixes all three scales, so its integrand is a
/// function of the momenta alone and its two partitions must agree to Monte-Carlo
/// error. Without it, a gap measured here could be the uniform-`α` arm's own
/// variance rather than the scale reading the channel.
///
/// What it cannot see: MadEvent's partition, which is neither of these — single-
/// diagram enhancement weights channel `c` by `AMP2_c(p)/Σ AMP2`, a function of
/// the point rather than a constant, so it is not reachable from any choice of
/// `αⱼ`. The gap below brackets the ambiguity; it does not locate MadGraph inside
/// it.
///
/// Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_jj_channel_partition() {
    if !dyn_run_present("probe_jj_channel_partition", JJ_RUN) {
        return;
    }
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let set = load_pdf_set();
    let pdf = set.member(0).expect("PDF member 0");

    let jj_dir = validation_dir().join("output").join(JJ_RUN);
    let jj_rc = RunCard::parse_file(&jj_dir.join("Cards/run_card.dat")).expect("banked run card");
    let (jj_mg, _) = banked_llj_sigma(&jj_dir);
    let control_dir = validation_dir().join("output/pp_to_llj_fixed");
    let control_rc =
        RunCard::parse_file(&control_dir.join("Cards/run_card.dat")).expect("banked run card");
    let (control_mg, _) = banked_llj_sigma(&control_dir);

    struct Arm<'a> {
        label: &'a str,
        groups: FlavorGroups,
        rc: &'a RunCard,
        mg: f64,
        shape: ScaleShape,
    }
    let arms = vec![
        Arm {
            label: "j j",
            groups: groups_for(JJ_PROCESS, &model, &evaluated, &jj_rc),
            rc: &jj_rc,
            mg: jj_mg,
            shape: ScaleShape::PerEvent,
        },
        Arm {
            label: "llj_fixed (control)",
            groups: groups_for(LLJ_PROCESS, &model, &evaluated, &control_rc),
            rc: &control_rc,
            mg: control_mg,
            shape: ScaleShape::FixedAtMz,
        },
    ];

    for arm in &arms {
        let amps: Vec<BoundAmplitude<f64>> = arm
            .groups
            .groups()
            .iter()
            .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), &evaluated))
            .collect();
        let mut measured = Vec::new();
        for adapt_iters in [JJ_ADAPT_ITERS, 0] {
            let mut summary = Vec::new();
            measured.push(run_seed_shaped(
                &arm.groups,
                &amps,
                &model,
                &evaluated,
                &set,
                &pdf,
                arm.rc,
                (JJ_ADAPT_SURVEY, adapt_iters, JJ_NEVAL, JJ_NITER),
                JJ_SEEDS[0],
                true,
                &mut summary,
                true,
                match arm.shape {
                    ScaleShape::PerEvent => ScaleShape::PerEvent,
                    ScaleShape::FixedAtMz => ScaleShape::FixedAtMz,
                },
            ));
        }
        let (adapted, err_a) = measured[0];
        let (uniform, err_u) = measured[1];
        eprintln!(
            "  {:<28}: adapted α {adapted:.6e} ± {err_a:.2e} | uniform α {uniform:.6e} ± \
             {err_u:.2e} | partition gap {:+.3e} (Monte-Carlo {:.1e}) | MG rel adapted {:+.3e} \
             uniform {:+.3e}",
            arm.label,
            uniform / adapted - 1.0,
            (err_a * err_a + err_u * err_u).sqrt() / adapted,
            adapted / arm.mg - 1.0,
            uniform / arm.mg - 1.0,
        );
    }
}

/// σ(p p → j j) at MadGraph's default dynamical scale, against the banked
/// `pp_to_jj` run.
///
/// The canonical leading-order QCD process on MadGraph's shipped run-card
/// defaults, and the row that needs the whole chain at once: flavour groups whose
/// members carry *unequal* identical-particle symmetry factors, a per-event kT
/// cluster scale in the integration channel each point was drawn in, and a
/// multichannel over a mixed subprocess set.
///
/// The set it sums over is MadGraph's own, entry for entry, and
/// [`jj_subprocesses_are_madgraphs_own`] asserts that at zero tolerance against
/// the run's `leshouche.inc` — this is the row where the enumeration's
/// unordered-outgoing rule matters, and a scalar cross section is far too coarse
/// to see a subprocess counted twice as anything but a normalisation.
///
/// `JJ_MAX_REL` is the reference's own `0.22 %` Monte-Carlo error with headroom,
/// and the **pull is asserted** rather than reported: the residual here is Monte
/// Carlo rather than a systematic of fixed size, and the arithmetic says a budget
/// increase cannot drive the pull up — MadGraph's error on this run is `1.47e6 pb`
/// against this side's `1.9e5` at the gate budget, so the combined error is
/// essentially the reference's.
///
/// What the number can and cannot see: σ is a
/// scalar, so a clustering that got individual events wrong while preserving their
/// average would pass it. That is what `validate_scales` replays this run's 10 000
/// events for, field by field against MadGraph's own printed `SCALUP` and
/// `<rscale>`.
#[test]
fn sigma_jj_dynamical_scale_vs_mg() {
    if !dyn_run_present("sigma_jj_dynamical_scale_vs_mg", JJ_RUN) {
        return;
    }
    let clock = Stopwatch::start();
    let run_dir = validation_dir().join("output").join(JJ_RUN);
    let rc = RunCard::parse_file(&run_dir.join("Cards/run_card.dat")).expect("banked run card");
    let (mg, mg_err) = banked_llj_sigma(&run_dir);

    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let groups = groups_for(JJ_PROCESS, &model, &evaluated, &rc);
    let set = load_pdf_set();
    let pdf = set.member(0).expect("PDF member 0");
    let amps: Vec<BoundAmplitude<f64>> = groups
        .groups()
        .iter()
        .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), &evaluated))
        .collect();

    let mut summary = Vec::new();
    let mut runs: Vec<SeedResult> = Vec::new();
    for &seed in JJ_SEEDS {
        let (sigma, err) = run_seed_shaped(
            &groups,
            &amps,
            &model,
            &evaluated,
            &set,
            &pdf,
            &rc,
            (JJ_ADAPT_SURVEY, JJ_ADAPT_ITERS, JJ_NEVAL, JJ_NITER),
            seed,
            // Every diagram of `p p > j j` is a strong vertex.
            true,
            &mut summary,
            true,
            ScaleShape::PerEvent,
        );
        eprintln!(
            "[jj seed {seed}] vibegraph σ = {sigma:.6e} ± {err:.3e} pb | rel = {:+.4}",
            sigma / mg - 1.0,
        );
        runs.push(SeedResult {
            seed,
            sigma_pb: sigma,
            sigma_err_pb: err,
        });
    }

    let (mean, mean_err, chi2) = combine_seeds(&runs);
    let pull = (mean - mg) / (mean_err * mean_err + mg_err * mg_err).sqrt();
    let rel = mean / mg - 1.0;
    eprintln!(
        "[jj] GATE vibegraph σ = {mean:.6e} ± {mean_err:.3e} pb ({} seeds, χ²/dof = {chi2:.2}) | \
         MG σ = {mg:.6e} ± {mg_err:.3e} pb | pull = {pull:+.2} | rel = {rel:+.4}",
        runs.len()
    );

    let ok = pull.abs() < 3.0 && rel.abs() < JJ_MAX_REL && chi2 < JJ_MAX_CHI2_PER_DOF;
    let mut row = IntegralsRow::new(JJ_RUN, JJ_PROCESS, "gate");
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
    row.neval = JJ_NEVAL;
    row.niter = JJ_NITER;
    row.subsampler = summary;
    row.note = Some(
        "the canonical leading-order QCD row: a per-event kT cluster scale read in the \
         channel each point was drawn in, over a flavour decomposition whose members \
         carry unequal identical-particle symmetry factors. Its 65 concrete \
         subprocesses are MadGraph's own, asserted entry for entry against the run's \
         leshouche.inc by jj_subprocesses_are_madgraphs_own. rel_tol is the reference's \
         own 0.22% Monte-Carlo error with headroom, and the pull is asserted: the \
         partition ambiguity that sets the llj tolerances is 1.0e-3 here, at its own \
         Monte Carlo, because a 2 -> 2 final state gives the clustering no merge to \
         choose. Three seeds at 75k, the lowest rung of the oracle-layer five-seed \
         ladder, which reads +0.33% / +0.26% / +0.25% / +0.30% at chi2/dof \
         1.40 / 0.44 / 0.82 / 1.21 over 75k to 600k -- flat, and with this side's \
         three-seed error at 0.081% against the reference's 0.217% the comparison is \
         still made at the bank's own precision"
            .to_string(),
    );
    row.duration_s = Some(clock.seconds());
    row.write();

    assert!(
        pull.abs() < 3.0 && rel.abs() < JJ_MAX_REL,
        "[jj] σ disagreement: vibegraph {mean:.6e}±{mean_err:.3e} vs \
         MG {mg:.6e}±{mg_err:.3e} pb, rel = {rel:+.4} (bound {JJ_MAX_REL}), \
         pull = {pull:+.2}"
    );
    assert!(
        chi2 < JJ_MAX_CHI2_PER_DOF,
        "[jj] the seeds scatter by more than they claim: χ²/dof = {chi2:.2} over {runs:?}"
    );
}

// ─────────────────── the configuration-dependence census ─────────────────────

/// How far a hadronic row's scales can move if the integration configuration
/// they are clustered in changes — and how much of that movement is the flavour
/// *group* rather than the configuration inside it.
///
/// A hadronic sampling channel is a `(group, diagram)` pair, and both halves
/// reach the scale: the group selects which merge graph a point is clustered
/// against, the diagram selects which configuration inside it. The two are
/// separately measurable at one point and this reports both:
///
/// * **within-group** — the worst spread over the configurations of a single
///   group, maximised over groups. This is what a rule for choosing the
///   configuration could move.
/// * **across-group** — the worst spread over every `(group, configuration)`
///   pair at the same momenta. It is bounded below by the within-group number,
///   and the gap between the two is the part no configuration rule can reach.
///
/// The clustering is rebuilt here from the run card and the groups' own diagrams
/// rather than read off the integrand, which has no accessor for it. That
/// reconstruction is not assumed: on every point the value it gives in the
/// channel the point was drawn in is compared against the scales the integrand
/// itself recorded on that point, and a mismatch fails the probe. Without that
/// check the numbers below would be a measurement of a second implementation.
///
/// What this cannot see: configuration dependence confined to a region of phase
/// space the drawn points miss, and any effect of the *mirror* ordering, which
/// is evaluated at the same scale as the direct one.
///
/// Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_cluster_scale_spread_over_configurations() {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use vibegraph::coupling::cluster::configs::derive_channels;
    use vibegraph::coupling::cluster::graph::ColorTable;
    use vibegraph::coupling::scales::{ClusterInput, ScaleChoice, ScaleEvent};

    /// Cut-passing points per row.
    const POINTS: usize = 64;

    for (run, process) in [(LLJ_DYN_RUN, LLJ_PROCESS), (JJ_RUN, JJ_PROCESS)] {
        if !dyn_run_present("probe_cluster_scale_spread_over_configurations", run) {
            continue;
        }
        let run_dir = validation_dir().join("output").join(run);
        let rc = RunCard::parse_file(&run_dir.join("Cards/run_card.dat")).expect("banked run card");
        let model = common::sm_model();
        let evaluated = EvaluatedModel::from_model(model.clone());
        let groups = groups_for(process, &model, &evaluated, &rc);
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
        assert!(
            report.constant_scales.is_none(),
            "[{run}] this row's card does not leave the scale to the clustering"
        );

        let choice = ScaleChoice::from_run_card(&rc).expect("the card's prescription compiles");
        let colors = ColorTable::new(
            model
                .particles
                .values()
                .map(|p| (p.pdg_code, p.color))
                .collect::<Vec<(i64, i32)>>(),
            rc.maxjetflavor,
        );
        let derived: Vec<_> = groups
            .groups()
            .iter()
            .map(|g| {
                derive_channels(
                    g.diagrams(),
                    g.evaluator().external_particles(),
                    g.evaluator().n_in(),
                    &model,
                    &evaluated,
                )
                .expect("the group's channel forests derive")
            })
            .collect();

        let ndim = integ.point_ndim();
        let mut rng = ChaCha8Rng::seed_from_u64(0x5CA1_E5_C4);
        let (mut within, mut across) = ([0.0f64; 3], [0.0f64; 3]);
        let mut per_group = vec![0.0f64; derived.len()];
        let (mut kept, mut tries) = (0usize, 0usize);
        let mut checked = 0usize;
        while kept < POINTS && tries < 400 * POINTS {
            tries += 1;
            let u: Vec<f64> = (0..ndim)
                .map(|_| rand::Rng::random::<f64>(&mut rng))
                .collect();
            let Some(ev) = integ.event_in_channel(0, &u) else {
                continue;
            };
            kept += 1;
            let incoming = [comps(&ev.lab[0]), comps(&ev.lab[1])];
            let outgoing: Vec<[f64; 4]> = ev.lab[2..].iter().map(comps).collect();
            let event = ScaleEvent {
                incoming,
                outgoing: &outgoing,
            };
            let (mut all_lo, mut all_hi) = ([f64::INFINITY; 3], [0.0f64; 3]);
            for (g, d) in derived.iter().enumerate() {
                let (mut lo, mut hi) = ([f64::INFINITY; 3], [0.0f64; 3]);
                for config in 1..=d.set.configs.len() {
                    let input = ClusterInput {
                        set: &d.set,
                        colors: &colors,
                        this_config: config,
                        iproc: 1,
                        tables: None,
                    };
                    let s = choice
                        .cluster_scales(&event, &input)
                        .expect("the prescription accepts a cut-passing point");
                    // The reconstruction is only worth reading if it reproduces
                    // what the integrand itself evaluated this point at.
                    let drawn = integ.channel_ids()[0];
                    if g == drawn.group && d.config_of_diagram[drawn.diagram].unwrap_or(1) == config
                    {
                        assert_eq!(
                            (s.mu_r, s.mu_f),
                            (ev.scales.mu_r, ev.scales.mu_f),
                            "[{run}] the rebuilt clustering disagrees with the integrand's \
                             own scales in the channel the point was drawn in"
                        );
                        checked += 1;
                    }
                    for (k, v) in [s.mu_r, s.mu_f[0], s.mu_f[1]].into_iter().enumerate() {
                        lo[k] = lo[k].min(v);
                        hi[k] = hi[k].max(v);
                        all_lo[k] = all_lo[k].min(v);
                        all_hi[k] = all_hi[k].max(v);
                    }
                }
                per_group[g] = per_group[g].max(hi[0] / lo[0] - 1.0);
                for k in 0..3 {
                    within[k] = within[k].max(hi[k] / lo[k] - 1.0);
                }
            }
            for k in 0..3 {
                across[k] = across[k].max(all_hi[k] / all_lo[k] - 1.0);
            }
        }
        assert!(
            checked == kept,
            "[{run}] the drawn channel's configuration was not reached on every point"
        );
        let configs: Vec<usize> = derived.iter().map(|d| d.set.configs.len()).collect();
        println!(
            "{run}: {} groups, configs {configs:?} over {kept} points | \
             within-group worst spread mu_R {:.6e} mu_F1 {:.6e} mu_F2 {:.6e} | \
             across-group worst spread mu_R {:.6e} mu_F1 {:.6e} mu_F2 {:.6e}",
            derived.len(),
            within[0],
            within[1],
            within[2],
            across[0],
            across[1],
            across[2]
        );
        println!(
            "{run}: per-group worst mu_R spread {}",
            per_group
                .iter()
                .enumerate()
                .map(|(g, s)| format!("g{g} {s:.6e}"))
                .collect::<Vec<_>>()
                .join(" | ")
        );
    }
}

/// Components in the `[E, px, py, pz]` layout the scale prescription reads.
fn comps(p: &V) -> [f64; 4] {
    [p.e(), p.px(), p.py(), p.pz()]
}

// ───────────── the rows re-carded off MadGraph's internal parton densities ──────
//
// `pp_to_bb`, `pp_to_bb_qcd2`, `pp_to_llj` and `pp_to_ll_scalefact2` were banked
// on `pdlabel = nn23lo1`, MadGraph's own parameterisation rather than an LHAPDF6
// grid this crate can read, so a cross section against them measured the parton
// densities and not the process. Re-carded onto `lhaid = 247000` they became
// measurable, and these are the first measurements: both sides convolve the same
// set, and what is left is the integrand.
//
// Each of the four is the *uncut* member of a pair whose cut member already
// gates. `pp_to_bb` / `pp_to_bb_qcd2` carry `ptb = 0` where `pp_to_bb_fixed`
// carries 20, so their ŝ floor is the b masses alone — `(2 m_b)² = 88 GeV²`
// against 1600 — and their σ is four orders larger. `pp_to_llj` is
// `pp_to_llj_dyn`'s card with `mmll` back at 0, the low-mass region the gating
// twin was carded away from. `pp_to_ll_scalefact2` is Drell-Yan at `mmll = 0`
// with the event-by-event scales doubled. So these rows reach phase space no
// enforced row does, which is what makes them worth measuring and what the
// tolerance discussion on each has to answer for.

/// The four re-carded runs, each with the process its `.mg5` script generates.
///
/// The spelling matters: `pp_to_bb` and `pp_to_llj` are the default-order
/// halves of their order-constraint pairs, so they are generated without the
/// explicit `QCD=` / `QED=` their twins carry.
/// Each entry is `(run, process, neval, ladder_note)`. The budget is per row and
/// comes from [`probe_recarded_budget_ladder`] rather than from cost, floored by
/// the rung each row's ladder stops moving on and sized against the reference's
/// own error above that floor — the gate resolves at `√(σ_ours² + σ_MG²)`, so
/// points spent past the reference's precision are invisible to the comparison.
/// `ladder_note` is what [`measure_recarded_sigma`] writes into the row's report
/// cell, and it is per row rather than one shared sentence: three of the four
/// ladders are flat at the rung the gate runs at, but `pp_to_llj`'s is not, and a
/// shared "the rung this row's ladder is flat at" sentence would misdescribe it.
const RECARDED_ROWS: &[(&str, &str, usize, &str)] = &[
    // Flat across the whole ladder — `+0.07%`, `−0.01%`, `+0.02%`, `−0.02%` at
    // 75k, 150k, 300k and 600k, with χ²/dof 0.55, 1.00, 0.09, 0.19 — so the
    // lowest rung is the converged one. Three seeds there leave this side's
    // error at `0.044%` against the reference's `0.071%`.
    (
        "pp_to_bb",
        "p p > b b~",
        75_000,
        "the rung this row's budget ladder is flat at",
    ),
    // The explicit-order twin, and flat on the same ladder: `+0.03%`, `+0.04%`,
    // `+0.00%`, `+0.00%` at χ²/dof 0.23, 1.03, 0.89, 0.15.
    (
        "pp_to_bb_qcd2",
        "p p > b b~ QCD=2",
        75_000,
        "the rung this row's budget ladder is flat at",
    ),
    // `mmll = 0` opens the low lepton-pair-mass region, the hardest budget on
    // any row here, and the only ladder that still moves in one direction: five
    // seeds a rung read `+0.04%`, `+0.07%`, `+0.11%`, `+0.21%` at 75k, 150k,
    // 300k and 600k, with χ²/dof 1.49, 0.65, 0.33, 0.46. The rise is monotone
    // over the whole eightfold range and is not a `75k` artefact, so no cut is
    // licensed here; `150k` is kept because the `75k` rung's error is inflated
    // to `0.29%` by a single seed (χ²/dof 1.49 against ≤0.65 above it), which is
    // the rung a three-seed gate would be reading.
    (
        "pp_to_llj",
        "p p > l+ l- j",
        150_000,
        "not a rung this row's budget ladder is flat at -- it still climbs \
         monotonically over the whole 75k-600k range (span 0.17%), and this \
         is the lowest rung whose 75k-inflated single-seed error does not \
         dominate a three-seed read; the 0.17% span is unresolvable against \
         the reference's own 0.33% error, which is why the climb is a \
         recorded residual rather than a reason to cut further",
    ),
    // Flat: `−0.03%`, `+0.05%`, `−0.02%`, `−0.04%` at χ²/dof 0.82, 0.65, 0.65,
    // 0.21, with this side's three-seed error `0.084%` against the reference's
    // `0.198%` at the lowest rung.
    (
        "pp_to_ll_scalefact2",
        "p p > l+ l-",
        75_000,
        "the rung this row's budget ladder is flat at",
    ),
];

/// Independent seeds these rows are measured on, for the reason the ℓℓj sweep
/// gives: a single seed's error bar is not a measurement of a VEGAS estimator.
const RECARDED_SEEDS: &[u64] = &[20260901, 20260902, 20260903];
/// Points per survey iteration, and iterations, of the channel-weight adaptation.
const RECARDED_ADAPT_SURVEY: usize = 8_000;
const RECARDED_ADAPT_ITERS: usize = 5;
const RECARDED_NITER: usize = 10;

/// Relative agreement these rows are held to.
///
/// Set from the references' own Monte-Carlo errors — `0.071%` on both `b b̄`
/// runs, `0.33%` on `ℓℓj`, `0.198%` on the `scalefact` Drell-Yan run — since no
/// agreement tighter than the reference's precision is meaningful. This is the
/// loosest of those with headroom; the measured distances are an order inside
/// it, and so is every rung of every row's budget ladder — the widest is `ℓℓj`'s
/// `+0.21%` at 600k.
const RECARDED_MAX_REL: f64 = 0.005;
/// Scatter the seeds are allowed about their own mean, in units of their own
/// quoted errors — the guard the scalar pull cannot be, since a run that missed
/// a region reports a small integral *and* a small error. Measured `0.09` to
/// `1.49` over five seeds a rung across the whole ladder, worst on `ℓℓj` at 75k.
const RECARDED_MAX_CHI2_PER_DOF: f64 = 4.0;

/// Gate one re-carded row's σ over [`RECARDED_SEEDS`] and write its cell.
///
/// The pull is asserted alongside the relative distance, and the arithmetic is
/// what makes that safe: MadGraph's own error on each of these runs is three to
/// nine times this side's at the gate budget, so the combined error is
/// essentially the reference's and no budget spent here drives the pull up.
///
/// What the cell cannot see is everything a scalar integrates over — the
/// per-event scale enters σ through an average, so a clustering that got
/// individual events wrong while preserving that average would pass. That is
/// what `validate_scales` replays all four runs' 10000 events for, field by
/// field, and what `validate_kt_cluster` reproduces the merge sequences behind
/// for two of them.
fn measure_recarded_sigma(run: &str, process: &str, neval: usize, ladder_note: &str) {
    if !dyn_run_present("recarded_rows_sigma_vs_mg", run) {
        return;
    }
    let clock = Stopwatch::start();
    let run_dir = validation_dir().join("output").join(run);
    let rc = RunCard::parse_file(&run_dir.join("Cards/run_card.dat")).expect("banked run card");
    let (mg, mg_err) = banked_llj_sigma(&run_dir);

    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let groups = groups_for(process, &model, &evaluated, &rc);
    let set = load_pdf_set();
    let pdf = set.member(0).expect("PDF member 0");
    let amps: Vec<BoundAmplitude<f64>> = groups
        .groups()
        .iter()
        .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), &evaluated))
        .collect();

    // Drell-Yan at QED-only orders has no strong vertex to find; every other
    // row here carries one.
    let expect_alpha_s = process != "p p > l+ l-";

    let mut summary = Vec::new();
    let mut runs: Vec<SeedResult> = Vec::new();
    for &seed in RECARDED_SEEDS {
        let (sigma, err) = run_seed_shaped(
            &groups,
            &amps,
            &model,
            &evaluated,
            &set,
            &pdf,
            &rc,
            (
                RECARDED_ADAPT_SURVEY,
                RECARDED_ADAPT_ITERS,
                neval,
                RECARDED_NITER,
            ),
            seed,
            expect_alpha_s,
            &mut summary,
            true,
            ScaleShape::PerEvent,
        );
        eprintln!(
            "[{run} seed {seed}] vibegraph σ = {sigma:.6e} ± {err:.3e} pb | rel = {:+.4}",
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
        "[{run}] GATE vibegraph σ = {mean:.6e} ± {mean_err:.3e} pb ({} seeds, χ²/dof = {chi2:.2}) \
         | MG σ = {mg:.6e} ± {mg_err:.3e} pb | pull = {pull:+.2} | rel = {rel:+.4}",
        runs.len()
    );

    let ok = pull.abs() < 3.0 && rel.abs() < RECARDED_MAX_REL && chi2 < RECARDED_MAX_CHI2_PER_DOF;
    let mut row = IntegralsRow::new(run, process, "gate");
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
    row.neval = neval;
    row.niter = RECARDED_NITER;
    row.subsampler = summary;
    row.note = Some(format!(
        "three seeds at {neval} points an iteration, {ladder_note}"
    ));
    row.duration_s = Some(clock.seconds());
    row.write();

    assert!(
        pull.abs() < 3.0 && rel.abs() < RECARDED_MAX_REL,
        "[{run}] σ disagreement: vibegraph {mean:.6e}±{mean_err:.3e} vs \
         MG {mg:.6e}±{mg_err:.3e} pb, pull = {pull:+.2}, rel = {rel:+.4}"
    );
    assert!(
        chi2 < RECARDED_MAX_CHI2_PER_DOF,
        "[{run}] the seeds scatter by more than they claim: χ²/dof = {chi2:.2} over {runs:?}"
    );
}

/// σ(p p → b b̄) at MadGraph's default coupling orders, with `ptb = 0` so the
/// `ŝ` floor is `(2 m_b)² = 88 GeV²` against `pp_to_bb_fixed`'s 1600 — four
/// orders more cross section, drawn from phase space no enforced row reaches.
#[test]
fn sigma_bb_recarded_vs_mg() {
    let (run, process, neval, ladder_note) = RECARDED_ROWS[0];
    measure_recarded_sigma(run, process, neval, ladder_note);
}

/// The explicit-`QCD=2` spelling of the row above, on its own banked run: the
/// pair is what pins order-constraint semantics at the cross-section level.
#[test]
fn sigma_bb_qcd2_recarded_vs_mg() {
    let (run, process, neval, ladder_note) = RECARDED_ROWS[1];
    measure_recarded_sigma(run, process, neval, ladder_note);
}

/// `pp_to_llj_dyn`'s card with `mmll` back at 0 — the low lepton-pair-mass
/// region the enforced twin is carded away from, at the budget its own ladder
/// says the estimator has stopped climbing at.
#[test]
fn sigma_llj_recarded_vs_mg() {
    let (run, process, neval, ladder_note) = RECARDED_ROWS[2];
    measure_recarded_sigma(run, process, neval, ladder_note);
}

/// The only banked row whose `scalefact` is not 1: Drell-Yan with every
/// event-by-event scale doubled, so the run card's scale factor reaches σ.
#[test]
fn sigma_ll_scalefact2_recarded_vs_mg() {
    let (run, process, neval, ladder_note) = RECARDED_ROWS[3];
    measure_recarded_sigma(run, process, neval, ladder_note);
}

/// The budget ladder behind the four re-carded rows' tolerances.
///
/// A seed sweep alone cannot separate agreement from this crate's convergence:
/// VEGAS's inverse-variance combination turns a region it under-samples into a
/// confidently wrong σ, and mutually consistent seeds have been collectively
/// low before. What tells the two apart is whether the estimator moves with the
/// budget. Five seeds a rung over an eightfold range, printed per rung so a
/// residual that shrinks and one that does not are distinguishable.
///
/// Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_recarded_budget_ladder() {
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let set = load_pdf_set();
    let pdf = set.member(0).expect("PDF member 0");

    for (run, process, _, _) in RECARDED_ROWS {
        if !dyn_run_present("probe_recarded_budget_ladder", run) {
            continue;
        }
        let run_dir = validation_dir().join("output").join(run);
        let rc = RunCard::parse_file(&run_dir.join("Cards/run_card.dat")).expect("banked run card");
        let (mg, mg_err) = banked_llj_sigma(&run_dir);
        let groups = groups_for(process, &model, &evaluated, &rc);
        let amps: Vec<BoundAmplitude<f64>> = groups
            .groups()
            .iter()
            .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), &evaluated))
            .collect();
        let expect_alpha_s = *process != "p p > l+ l-";

        eprintln!("── {run}: MG {mg:.6e} ± {mg_err:.3e} pb ──");
        for neval in [75_000usize, 150_000, 300_000, 600_000] {
            let mut summary = Vec::new();
            let mut runs: Vec<SeedResult> = Vec::new();
            for &seed in &[20260901u64, 20260902, 20260903, 20260904, 20260905] {
                let (sigma, err) = run_seed_shaped(
                    &groups,
                    &amps,
                    &model,
                    &evaluated,
                    &set,
                    &pdf,
                    &rc,
                    (
                        RECARDED_ADAPT_SURVEY,
                        RECARDED_ADAPT_ITERS,
                        neval,
                        RECARDED_NITER,
                    ),
                    seed,
                    expect_alpha_s,
                    &mut summary,
                    true,
                    ScaleShape::PerEvent,
                );
                runs.push(SeedResult {
                    seed,
                    sigma_pb: sigma,
                    sigma_err_pb: err,
                });
            }
            let (mean, mean_err, chi2) = combine_seeds(&runs);
            let pull = (mean - mg) / (mean_err * mean_err + mg_err * mg_err).sqrt();
            eprintln!(
                "  neval {neval:>7}: σ = {mean:.6e} ± {mean_err:.3e} pb (χ²/dof {chi2:.2}) | \
                 rel {:+.4} | pull {pull:+.2}",
                mean / mg - 1.0
            );
        }
    }
}

/// Whether the re-carded `p p > l+ l- j` estimator is still moving above the
/// budget its gate runs at, and what it is moving with.
///
/// The four-rung ladder above reads that row rising monotonically over `75k`
/// to `600k` at a size the reference's own `0.33%` cannot resolve, so it
/// cannot say whether the estimator converges to something and where. This
/// extends the same measurement by two more doublings and adds two controls
/// that the σ ladder alone does not separate.
///
/// **Arms.** `pp_to_llj` is the `mmll = 0` card and `pp_to_llj_dyn` is the same
/// card with `mmll = 50`; nothing else differs. Running both over the same
/// rungs turns "the drift lives below the lepton-pair mass cut" into a
/// controlled comparison rather than a reading of one column.
///
/// **The equal-kept controls.** A rung moves `neval` and the kept sample size
/// together, so a drift in the combined estimate cannot be attributed between
/// them. `150k × 34`, `300k × 18` and `600k × 10` all keep `4.8M` points past
/// the two warm-up iterations while the per-iteration budget spans `4×`: an
/// estimator limited by its total sample agrees across the three, one limited
/// by the grid an iteration is drawn against does not.
///
/// The α survey is `8000 × 5` at every rung and is addressed by the seed
/// alone, so the channel weights are identical along a row and the survey is
/// not on the list of things a rung changes.
///
/// Two errors are printed per rung: the quoted `√(Σσᵢ²)/n`, and the seeds' own
/// RMS about their mean over `√n`. Neither is the estimator's real spread at
/// five seeds — `probe_llj_seed_ensemble` measures that over forty — and the
/// scatter is the further off of the two, so both are printed rather than one.
///
/// Reads, on the `mmll = 0` card against MadGraph's `504.630 ± 1.674 pb`:
/// `504.506` / `505.184` / `505.977` / `505.849` / `505.828` at `150k` through
/// `2.4M`, so the rise the four-rung ladder shows stops by `600k` and the next
/// two doublings move nothing. The three equal-kept rungs read `505.977`
/// (`600k × 10`), `505.964` (`150k × 34`) and `505.931` (`300k × 18`) — `0.05
/// pb` apart across a `4×` span in per-iteration budget, so what the estimator
/// is limited by at the low rungs is its total sample and not the grid an
/// iteration draws against. The `mmll = 50` arm steps by the same relative
/// amount (`415.456` → `416.361` → `416.186`), so the step is not the low
/// lepton-pair-mass region that card cuts away.
///
/// Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_llj_deep_budget_ladder() {
    const SEEDS: &[u64] = &[20260901u64, 20260902, 20260903, 20260904, 20260905];
    const RUNGS: &[(usize, usize)] = &[
        (150_000, 10),
        (300_000, 10),
        (600_000, 10),
        (1_200_000, 10),
        (2_400_000, 10),
        (150_000, 34),
        (300_000, 18),
    ];
    const ARMS: &[(&str, &str, &[(usize, usize)])] = &[
        ("pp_to_llj", "p p > l+ l- j", RUNGS),
        (
            LLJ_DYN_RUN,
            LLJ_PROCESS,
            &[(150_000, 10), (600_000, 10), (2_400_000, 10)],
        ),
    ];

    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let set = load_pdf_set();
    let pdf = set.member(0).expect("PDF member 0");

    for &(run, process, rungs) in ARMS {
        if !dyn_run_present("probe_llj_deep_budget_ladder", run) {
            continue;
        }
        let run_dir = validation_dir().join("output").join(run);
        let rc = RunCard::parse_file(&run_dir.join("Cards/run_card.dat")).expect("banked run card");
        let (mg, mg_err) = banked_llj_sigma(&run_dir);
        let groups = groups_for(process, &model, &evaluated, &rc);
        let amps: Vec<BoundAmplitude<f64>> = groups
            .groups()
            .iter()
            .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), &evaluated))
            .collect();

        eprintln!(
            "── {run}: MG {mg:.6e} ± {mg_err:.3e} pb, mmll = {} ──",
            rc.float("mmll")
        );
        let mut base: Option<(f64, f64)> = None;
        for &(neval, niter) in rungs {
            let clock = Stopwatch::start();
            let mut summary = Vec::new();
            let mut runs: Vec<SeedResult> = Vec::new();
            for &seed in SEEDS {
                let (sigma, err) = run_seed_shaped(
                    &groups,
                    &amps,
                    &model,
                    &evaluated,
                    &set,
                    &pdf,
                    &rc,
                    (RECARDED_ADAPT_SURVEY, RECARDED_ADAPT_ITERS, neval, niter),
                    seed,
                    true,
                    &mut summary,
                    true,
                    ScaleShape::PerEvent,
                );
                runs.push(SeedResult {
                    seed,
                    sigma_pb: sigma,
                    sigma_err_pb: err,
                });
            }
            let (mean, mean_err, chi2) = combine_seeds(&runs);
            let n = runs.len() as f64;
            let scatter = (runs
                .iter()
                .map(|r| (r.sigma_pb - mean).powi(2))
                .sum::<f64>()
                / (n - 1.0))
                .sqrt()
                / n.sqrt();
            let pull = (mean - mg) / (mean_err * mean_err + mg_err * mg_err).sqrt();
            let kept = (niter - 2) * neval;
            let drift = base.map(|(b, be)| {
                (
                    mean - b,
                    (mean - b) / (mean_err * mean_err + be * be).sqrt(),
                )
            });
            if base.is_none() {
                base = Some((mean, mean_err));
            }
            eprintln!(
                "  {neval:>7} x {niter:<2} (kept {:>5.1}M): σ = {mean:.4} ± {mean_err:.4} pb \
                 [scatter ±{scatter:.4}] (χ²/dof {chi2:.2}) | rel {:+.4} | pull {pull:+.2}{} | \
                 {:.0} s",
                kept as f64 / 1e6,
                mean / mg - 1.0,
                drift
                    .map(|(d, s)| format!(" | vs base {d:+.3} pb = {s:+.2} sd"))
                    .unwrap_or_default(),
                clock.seconds(),
            );
            eprintln!(
                "            per seed {}",
                runs.iter()
                    .map(|r| format!("{:.2}±{:.2}", r.sigma_pb, r.sigma_err_pb))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
    }
}

/// Whether the re-carded `p p > l+ l- j` estimator's low-budget deficit is a
/// bias in its expectation or a skew in its distribution.
///
/// Its ladder reads a rung `0.26%` below where the same estimator settles once
/// the sample is `10×` larger. Five seeds cannot say which of two things that
/// is. An estimator whose *expectation* moves with the budget is missing part
/// of the integrand at the small one; an estimator whose expectation is right
/// but whose distribution is right-skewed puts most seeds below its own mean,
/// and a five-seed average of it reads low without anything being missed. The
/// two want different answers — the first wants budget, the second wants
/// seeds — and they are told apart by the shape of the seed distribution, not
/// by its first five draws.
///
/// So: forty independent seeds at each of three rungs, reporting the mean
/// against the median and the skewness of each. Equal expectations with a
/// right skew is the second; means that differ by the ladder's own step is the
/// first.
///
/// The forty are also split into eight disjoint quintets and each quintet's
/// own three-rung ladder is read, because a five-seed ladder is what the gate
/// budgets are set from: how many of the eight climb says directly whether a
/// climb is a property of the estimator or of a seed set.
///
/// Both `ℓ⁺ℓ⁻ j` cards are measured. The `mmll = 50` one is the enforced row,
/// and its own five-seed ladder steps by the same relative amount, so what the
/// ensemble says about the estimator has to be said about it too.
///
/// Reads, on the `mmll = 0` card: `505.902 ± 0.218` / `505.853 ± 0.127` /
/// `505.709 ± 0.070 pb` at `150k` / `300k` / `600k`, flat to `0.84` standard
/// errors over the whole range and drifting *down* rather than up — the
/// estimator's expectation does not move with the budget, and the four-rung
/// ladder's step is not one. What does move is the spread: per-seed `sd`
/// `1.378` / `0.803` / `0.443 pb`, so a five-seed mean at `150k` has a real
/// error of `0.616 pb`, twice what five seeds' own scatter estimates it at.
/// One of the eight quintets reproduces the four-rung ladder's step almost
/// digit for digit (`504.307` → `505.120` → `505.790`, `+0.294%` against that
/// ladder's `+0.291%`); three step monotonically down. The `mmll = 50` card
/// reads `416.257 ± 0.114 pb` at `150k`, which is already where its own
/// `2.4M` rung sits.
///
/// Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_llj_seed_ensemble() {
    const SEEDS: usize = 40;
    const ARMS: &[(&str, &str, &[(usize, usize)])] = &[
        (
            "pp_to_llj",
            "p p > l+ l- j",
            &[(150_000, 10), (300_000, 10), (600_000, 10)],
        ),
        (
            LLJ_DYN_RUN,
            LLJ_PROCESS,
            &[(150_000, 10), (300_000, 10), (600_000, 10)],
        ),
    ];

    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let set = load_pdf_set();
    let pdf = set.member(0).expect("PDF member 0");
    for &(run, process, rungs) in ARMS {
        if !dyn_run_present("probe_llj_seed_ensemble", run) {
            continue;
        }
        seed_ensemble_of(run, process, rungs, SEEDS, &model, &evaluated, &set, &pdf);
    }
}

/// One card's seed ensemble, printed. Split out so the two arms run under
/// identical code.
#[expect(clippy::too_many_arguments)]
fn seed_ensemble_of(
    run: &str,
    process: &str,
    rungs: &[(usize, usize)],
    seeds: usize,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    set: &PdfSet,
    pdf: &PdfMember,
) {
    let run_dir = validation_dir().join("output").join(run);
    let rc = RunCard::parse_file(&run_dir.join("Cards/run_card.dat")).expect("banked run card");
    let (mg, mg_err) = banked_llj_sigma(&run_dir);
    let groups = groups_for(process, model, evaluated, &rc);
    let amps: Vec<BoundAmplitude<f64>> = groups
        .groups()
        .iter()
        .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), evaluated))
        .collect();

    eprintln!("── {run}: MG {mg:.6e} ± {mg_err:.3e} pb, {seeds} seeds a rung ──");
    let mut per_rung: Vec<Vec<f64>> = Vec::new();
    for &(neval, niter) in rungs {
        let clock = Stopwatch::start();
        let mut summary = Vec::new();
        let mut sigmas: Vec<f64> = Vec::new();
        for k in 0..seeds {
            let (sigma, _) = run_seed_shaped(
                &groups,
                &amps,
                model,
                evaluated,
                set,
                pdf,
                &rc,
                (RECARDED_ADAPT_SURVEY, RECARDED_ADAPT_ITERS, neval, niter),
                20_261_000 + k as u64,
                true,
                &mut summary,
                true,
                ScaleShape::PerEvent,
            );
            sigmas.push(sigma);
        }
        let n = sigmas.len() as f64;
        let mean = sigmas.iter().sum::<f64>() / n;
        let var = sigmas.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / (n - 1.0);
        let sd = var.sqrt();
        let skew = sigmas
            .iter()
            .map(|s| ((s - mean) / sd).powi(3))
            .sum::<f64>()
            * n
            / ((n - 1.0) * (n - 2.0));
        let mut sorted = sigmas.clone();
        sorted.sort_by(f64::total_cmp);
        let median = 0.5 * (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]);
        eprintln!(
            "  {neval:>7} x {niter:<2}: mean {mean:.4} ± {:.4} pb | median {median:.4} \
             (mean − median {:+.4}) | sd {sd:.4} | skew {skew:+.2} | \
             range [{:.2}, {:.2}] | rel {:+.4} | {:.0} s",
            sd / n.sqrt(),
            mean - median,
            sorted[0],
            sorted[sorted.len() - 1],
            mean / mg - 1.0,
            clock.seconds(),
        );
        per_rung.push(sigmas);
    }

    eprintln!("  disjoint five-seed ladders over the same rungs:");
    let mut monotone = 0usize;
    for q in 0..seeds / 5 {
        let means: Vec<f64> = per_rung
            .iter()
            .map(|s| s[5 * q..5 * q + 5].iter().sum::<f64>() / 5.0)
            .collect();
        let up = means.windows(2).all(|w| w[1] > w[0]);
        let down = means.windows(2).all(|w| w[1] < w[0]);
        monotone += usize::from(up);
        eprintln!(
            "    seeds {}..{}: {} | span {:+.3} pb ({:+.3}%) | {}",
            20_261_000 + 5 * q,
            20_261_004 + 5 * q,
            means
                .iter()
                .map(|m| format!("{m:.3}"))
                .collect::<Vec<_>>()
                .join(" -> "),
            means[means.len() - 1] - means[0],
            100.0 * (means[means.len() - 1] / means[0] - 1.0),
            if up {
                "monotone up"
            } else if down {
                "monotone down"
            } else {
                "not monotone"
            },
        );
    }
    eprintln!("  {monotone} of {} quintets climb monotonically", seeds / 5);
}

/// What the per-point configuration draw costs on a live-draw row.
///
/// The dynamical-scale rows draw the integration configuration their scale is
/// clustered in from the point's own squared amplitudes, which is one `eval_amp2`
/// and one `set_alpha_s` per point on top of everything the fixed-scale path
/// already does. The card's `sde_strategy` is the switch that decides whether the
/// enhancement weight is the squared amplitude, and it is the *only* thing the
/// two integrands here differ in: same process, same cuts, same clustering, same
/// running coupling, same points. So the gap between them is the draw and nothing
/// else — unlike a dynamical-against-fixed comparison, which also carries the kT
/// clustering and the per-point coupling move.
///
/// Points the cuts reject return before the draw, so the cost is charged on the
/// surviving fraction; the ratio reported is against the same points' total, which
/// is the number an integration budget is spent on. Run with
/// `--ignored --nocapture`, and on an otherwise idle machine.
///
/// Two rows are priced, because the draw does not cost the same on them. On `ℓ⁺ℓ⁻ j`
/// the drawn configuration sets the event's renormalisation scale, so the coupling
/// moves between the draw's `AMP2` and the matrix element and the second evaluation
/// starts from nothing. Drell–Yan at this order carries no strong coupling, so nothing
/// between the two moves and the matrix element reads currents the draw already
/// computed.
#[test]
#[ignore]
fn probe_scale_draw_cost() {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::time::Instant;
    if !dyn_run_present("probe_scale_draw_cost", LLJ_DYN_RUN) {
        return;
    }
    for (run, process) in [(LLJ_DYN_RUN, LLJ_PROCESS), (DY_DRAW_RUN, DY_PROCESS)] {
        let run_dir = validation_dir().join("output").join(run);
        let text = std::fs::read_to_string(run_dir.join("Cards/run_card.dat")).expect("run card");
        let live = RunCard::parse(&text).expect("banked run card parses");
        let off = RunCard::parse(&without_amp2_weights(&text)).expect("patched run card parses");

        let model = common::sm_model();
        let evaluated = EvaluatedModel::from_model(model.clone());
        let groups = groups_for(process, &model, &evaluated, &live);
        let set = load_pdf_set();
        let pdf = set.member(0).expect("PDF member 0");
        let amps: Vec<BoundAmplitude<f64>> = groups
            .groups()
            .iter()
            .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), &evaluated))
            .collect();

        let build = |rc: &RunCard| {
            let mut integ =
                ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
                    .expect("integrand");
            integ
                .use_run_card_scales(&model, &evaluated, rc, Some(&set.info.alpha_s))
                .expect("scale prescription compiles");
            integ.adapt_alphas(LLJ_SEEDS[0], LLJ_ADAPT_SURVEY, LLJ_ADAPT_ITERS, 0.5);
            integ
        };
        let with_draw = build(&live);
        let without_draw = build(&off);
        assert_eq!(
            (with_draw.scale_draw_ndim(), without_draw.scale_draw_ndim()),
            (1, 0),
            "[{run}] the two integrands must differ in the configuration draw and only there"
        );

        let ndim = with_draw.vegas_ndim();
        let mut rng = ChaCha8Rng::seed_from_u64(LLJ_SEEDS[0]);
        let points: Vec<Vec<f64>> = (0..PROBE_POINTS)
            .map(|_| {
                (0..ndim)
                    .map(|_| rand::Rng::random::<f64>(&mut rng))
                    .collect()
            })
            .collect();

        let time = |integ: &ProtonIntegrand<'_>| {
            let take = integ.vegas_ndim();
            let start = Instant::now();
            let mut acc = 0.0;
            for u in &points {
                acc += integ.value(&u[..take]);
            }
            std::hint::black_box(acc);
            start.elapsed().as_secs_f64() / points.len() as f64 * 1e9
        };
        time(&with_draw);
        time(&without_draw);
        let ns_on = time(&with_draw);
        let ns_off = time(&without_draw);
        eprintln!(
            "[{run}] {ns_off:8.1} ns/point without the configuration draw | \
             {ns_on:8.1} ns/point with it | draw {:+.1} ns ({:+.2}% of the per-point budget) \
             over {PROBE_POINTS} points",
            ns_on - ns_off,
            (1.0 - ns_off / ns_on) * 100.0,
        );
    }
}

/// How many points a per-point cost probe averages over.
const PROBE_POINTS: usize = 20_000;

/// The same run card with its enhancement weight taken off the squared amplitude,
/// which is what turns the per-point configuration draw off.
fn without_amp2_weights(text: &str) -> String {
    text.lines()
        .map(|l| {
            if l.contains("= sde_strategy") {
                "  2 = sde_strategy\n".to_string()
            } else {
                format!("{l}\n")
            }
        })
        .collect()
}

/// Where `p p > l+ l- j`'s sampling variance actually lives.
///
/// The row costs several times MadGraph's evaluations for the same accuracy and
/// its σ ladder climbs monotonically with budget, and both are one statement:
/// the weight distribution has a tail heavy enough that the sample mean and the
/// empirical variance both converge slowly and from below. Knowing the tail
/// index is not knowing where it comes from, and the fix — if there is one — is
/// a map, so this decomposes the variance by channel and by the kinematics of
/// the points that carry it.
///
/// Reads, per channel: the σ share, the variance share, the fraction of the
/// channel's own second moment carried by its heaviest 0.1% of draws, and a Hill
/// tail index over the top 1%. Then the same second moment binned in the
/// variables the maps are built out of — the lepton-pair mass the timelike pole
/// sits on, the jet `pT` the spacelike ones are cut at, and the partonic `√ŝ`
/// the shared outer map draws.
///
/// Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_llj_weight_tail_regions() {
    const SEED: u64 = 20_260_719;
    const DRAWS_PER_CHANNEL: usize = 200_000;

    // The two cards differ in `mmll` and in nothing else — 0 against 50 GeV — so
    // running both turns "the tail lives at low lepton-pair mass" from a reading
    // of one table into a controlled comparison.
    for run in ["pp_to_llj", "pp_to_llj_dyn"] {
        weight_tail_of(run, SEED, DRAWS_PER_CHANNEL);
    }
}

/// One run card's weight-tail decomposition, printed. Split out of the probe so
/// the two cards run under identical code.
fn weight_tail_of(run: &str, seed: u64, draws_per_channel: usize) {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use vibegraph::phasespace::rng::{SubStream, SCALE_DRAW_STREAM_BASE};

    if !dyn_run_present("probe_llj_weight_tail_regions", run) {
        return;
    }
    let run_dir = validation_dir().join("output").join(run);
    let rc = RunCard::parse_file(&run_dir.join("Cards/run_card.dat")).expect("banked run card");
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let groups = groups_for("p p > l+ l- j", &model, &evaluated, &rc);
    let set = load_pdf_set();
    let pdf = set.member(0).expect("PDF member 0");
    let amps: Vec<BoundAmplitude<f64>> = groups
        .groups()
        .iter()
        .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), &evaluated))
        .collect();
    let mut integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
        .expect("hadronic integrand");
    integ
        .use_run_card_scales(&model, &evaluated, &rc, Some(&set.info.alpha_s))
        .expect("run card scale prescription compiles");
    integ.adapt_alphas(seed, LLJ_ADAPT_SURVEY, LLJ_ADAPT_ITERS, 0.5);
    let (channels, total) = integ.adapt_grids(LLJ_NEVAL, LLJ_NITER, seed);
    eprintln!(
        "── {run}: trained at {LLJ_NEVAL} x {LLJ_NITER}, σ̂ = {:.6e} ± {:.3e} (χ²/dof {:.2}), \
         τ_min = {:.3e}, {} channels ──",
        total.integral,
        total.std_dev,
        total.chi2_per_dof,
        integ.tau_min(),
        channels.len(),
    );

    let ndim = integ.channel_grid_ndim();
    let scale_ndim = integ.scale_draw_ndim();

    // What a *fresh* grid accepts, against what the trained one does (the
    // `nonzero` column below). A rejected point short-circuits before the matrix
    // element, so the average point gets more expensive exactly as the sampler
    // gets better at finding the fiducial region — which is the shape of the
    // per-iteration ns/eval climb.
    {
        const ACCEPT_DRAWS: usize = 100_000;
        let mut u = vec![0.0f64; ndim + scale_ndim];
        let fresh = vibegraph::vegas::VegasGrid::new(ndim, 64, 1.5);
        let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0xACCE);
        let mut trailing = SubStream::from_stream(seed ^ 0xACCE, SCALE_DRAW_STREAM_BASE);
        let mut passed = 0usize;
        for k in 0..ACCEPT_DRAWS {
            let j = k % channels.len();
            fresh.draw(&mut rng, &mut u[..ndim]);
            trailing.fill_uniforms(&mut u[ndim..]);
            if integ.event_in_channel(j, &u).is_some() {
                passed += 1;
            }
        }
        eprintln!(
            "  cut acceptance on an untrained grid: {:.1}% over {ACCEPT_DRAWS} draws",
            100.0 * passed as f64 / ACCEPT_DRAWS as f64
        );
    }

    // The bins the maps are built out of. `m_ll` is where the timelike poles are
    // — the Z's Breit-Wigner and the photon's zero-width log map — `pT(j)` is
    // where the spacelike ones are cut, and `√ŝ` is the one variable no channel
    // maps: the shared logarithmic `(τ, y)` draw sets it before a channel is
    // consulted.
    let mll_edges = [0.0, 5.0, 10.0, 20.0, 40.0, 70.0, 88.0, 94.0, 120.0, 200.0];
    let ptj_edges = [20.0, 25.0, 35.0, 50.0, 80.0, 150.0, 400.0];
    let shat_edges = [0.0, 150.0, 250.0, 400.0, 700.0, 1500.0, 4000.0];
    let bin = |edges: &[f64], v: f64| edges.iter().rposition(|&e| v >= e).unwrap_or(0);
    let mut mll_m2 = vec![0.0f64; mll_edges.len()];
    let mut ptj_m2 = vec![0.0f64; ptj_edges.len()];
    let mut shat_m2 = vec![0.0f64; shat_edges.len()];
    let mut mll_sum = vec![0.0f64; mll_edges.len()];
    let mut ptj_sum = vec![0.0f64; ptj_edges.len()];
    let mut shat_sum = vec![0.0f64; shat_edges.len()];

    struct Row {
        j: usize,
        sum: f64,
        m2: f64,
        nonzero: usize,
        top_share: f64,
        hill: f64,
        peak: (f64, f64, f64, f64),
    }
    let mut rows: Vec<Row> = Vec::new();
    let mut u = vec![0.0f64; ndim + scale_ndim];

    for (j, ch) in channels.iter().enumerate() {
        let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0x7A11);
        rng.set_stream(j as u64);
        let mut trailing = SubStream::from_stream(seed ^ 0x7A11, SCALE_DRAW_STREAM_BASE + j as u64);
        let mut weights: Vec<f64> = Vec::new();
        let mut sum = 0.0f64;
        let mut m2 = 0.0f64;
        let mut peak = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
        for _ in 0..draws_per_channel {
            let jac = ch.grid.draw(&mut rng, &mut u[..ndim]);
            trailing.fill_uniforms(&mut u[ndim..]);
            let Some(ev) = integ.event_in_channel(j, &u) else {
                continue;
            };
            let w = jac * ev.weight;
            if !(w > 0.0) {
                continue;
            }
            sum += w;
            m2 += w * w;
            weights.push(w);
            let ll = ev.lab[2] + ev.lab[3];
            let mll = ll.m2().max(0.0).sqrt();
            let jet = ev.lab[4];
            let ptj = (jet.px() * jet.px() + jet.py() * jet.py()).sqrt();
            let shat = (SQRT_S_HAD * SQRT_S_HAD * ev.x[0] * ev.x[1]).sqrt();
            mll_m2[bin(&mll_edges, mll)] += w * w;
            ptj_m2[bin(&ptj_edges, ptj)] += w * w;
            shat_m2[bin(&shat_edges, shat)] += w * w;
            mll_sum[bin(&mll_edges, mll)] += w;
            ptj_sum[bin(&ptj_edges, ptj)] += w;
            shat_sum[bin(&shat_edges, shat)] += w;
            if w * w > peak.0 {
                peak = (w * w, mll, ptj, shat);
            }
        }
        weights.sort_by(f64::total_cmp);
        let n = weights.len();
        let top = n / 1000;
        let top_share = if top > 0 {
            weights[n - top..].iter().map(|w| w * w).sum::<f64>() / m2
        } else {
            f64::NAN
        };
        // Hill's estimator on the top 1%: `α = (k / Σ ln(w_(n-i)/w_(n-k)))`, the
        // maximum-likelihood tail index of a Pareto upper tail. Below 2 the
        // variance does not exist and no error bar on this channel means
        // anything; near 2 it exists but converges arbitrarily slowly.
        let k = (n / 100).max(1);
        let hill = if n > 100 && weights[n - k] > 0.0 {
            let s: f64 = weights[n - k..]
                .iter()
                .map(|w| (w / weights[n - k]).ln())
                .sum();
            k as f64 / s
        } else {
            f64::NAN
        };
        rows.push(Row {
            j,
            sum,
            m2,
            nonzero: n,
            top_share,
            hill,
            peak,
        });
    }

    let sum_all: f64 = rows.iter().map(|r| r.sum).sum();
    let m2_all: f64 = rows.iter().map(|r| r.m2).sum();
    eprintln!(
        "\n  ch |  σ share | var share | nonzero |  top 0.1% of M2 | Hill α | peak at \
         (m_ll, pT_j, √ŝ)"
    );
    let mut order: Vec<&Row> = rows.iter().collect();
    order.sort_by(|a, b| b.m2.total_cmp(&a.m2));
    for r in order {
        eprintln!(
            "  {:>2} | {:>7.3}% | {:>8.3}% | {:>7} | {:>14.1}% | {:>6.2} | {:>7.1} {:>7.1} {:>8.1}",
            r.j,
            100.0 * r.sum / sum_all,
            100.0 * r.m2 / m2_all,
            r.nonzero,
            100.0 * r.top_share,
            r.hill,
            r.peak.1,
            r.peak.2,
            r.peak.3,
        );
    }

    let table = |name: &str, edges: &[f64], m2: &[f64], sums: &[f64]| {
        let m2_tot: f64 = m2.iter().sum();
        let s_tot: f64 = sums.iter().sum();
        eprintln!("\n  {name}: σ share and variance share by bin");
        for (i, &e) in edges.iter().enumerate() {
            let hi = edges.get(i + 1).copied().unwrap_or(f64::INFINITY);
            eprintln!(
                "    [{e:>7.1}, {hi:>7.1}) : σ {:>7.3}%  var {:>7.3}%  (var/σ ratio {:>6.2})",
                100.0 * sums[i] / s_tot,
                100.0 * m2[i] / m2_tot,
                (m2[i] / m2_tot) / (sums[i] / s_tot).max(1.0e-30),
            );
        }
    };
    table("m_ll [GeV]", &mll_edges, &mll_m2, &mll_sum);
    table("pT(j) [GeV]", &ptj_edges, &ptj_m2, &ptj_sum);
    table("√ŝ [GeV]", &shat_edges, &shat_m2, &shat_sum);
}
