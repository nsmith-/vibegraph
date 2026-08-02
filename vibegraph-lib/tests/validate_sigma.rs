//! Cross-section-level (sigma) validation gate against banked MadGraph runs.
//!
//! For each fixed-energy (`lpp = 0`) MG-validated process, this drives the
//! library integration path (`FixedBeamIntegrand`, the engine behind
//! `vibegraph integrate`) with the *exact* `run_card.dat` MadGraph used, so beams
//! and cuts are pinned identically on both sides by construction, and compares
//! the integrated sigma against the banked MG value
//! (`validation/madgraph/sigma_reference.json`).
//!
//! # What this gate covers that the bit-exact net cannot
//!
//! The per-point `amplitude_oracle` net is blind to everything *outside* the
//! matrix element: the flux factor `1/(2 s-hat)`, the initial-state spin/colour
//! average, identical-particle and phase-space symmetry factors, the cut filter,
//! and the beam/phase-space measure. A wrong constant in any of those leaves the
//! per-point |M|^2 bit-exact yet moves the cross section. This gate is the coarse
//! instrument that sees those.
//!
//! # The error classes this gate provably cannot detect
//!
//! sigma-agreement is a **weak oracle**: the integral is a single scalar, blind
//! to *mis-sampled regions of small measure*. The narrow features flat RAMBO
//! cannot reach — a Breit-Wigner resonance, or a soft/collinear enhancement left
//! after cuts — are now importance-mapped by the per-diagram multichannel sampler
//! (`FixedBeamIntegrand::use_multichannel`, the production `vibegraph integrate`
//! path), so the resonant electroweak states converge and are gated too.
//!
//! The integral is taken channel by channel (`FixedBeamIntegrand::adapt_grids`):
//! each channel is integrated over its own coordinates with its own VEGAS grid, on
//! a share `alpha_j` of the sample budget, and the terms are summed with their
//! errors in quadrature. That is what the production `vibegraph integrate` does, so
//! it is what this gate measures.
//!
//! Being unbiased *in expectation* is not the same as being safe at finite N, and
//! this gate must not be read as if it were. An under-covered region does not
//! merely inflate the variance: iterations that miss it report a small integral
//! **and** a small variance, and VEGAS combines iterations by `1/sigma^2`, so the
//! misses dominate the weighted mean and shrink the quoted error with them. The
//! observed failure mode is a sigma 25x low quoted to 5% — a *confident* wrong
//! answer that a single-seed pull reads as a mild few-sigma miss. What defends
//! against it is the seed sweep below plus the distribution-level gates in the
//! phase-space unit tests, never this scalar on its own.
//!
//! The coupling *is* now part of what this gate compares. Each process is driven
//! at the strong coupling its run card's per-event renormalisation scale implies
//! (`FixedBeamIntegrand::use_running_coupling`), the way MadGraph does, rather
//! than at the param card's alpha_s, so a QCD cross section is compared on the
//! same footing as an electroweak one. That closes what used to be this gate's
//! largest blind spot and is what lets the QCD rows be asserted at all.
//!
//! # Which processes are asserted
//!
//! The electroweak final states are gated with an assertion, including the
//! sharply resonant `e+ e- > ta+ ta- h`, `e+ e- > mu+ mu- a` and
//! `e+ e- > mu+ mu- ta+ ta-`: the multichannel sampler resolves the Z/gamma* and
//! Higgs Breit-Wigner peaks flat RAMBO under-sampled, so they converge to the
//! banked sigma and hold across independent RNG seeds. The three QCD processes
//! are gated too, now that they run alpha_s to the same scale MadGraph does.
//!
//! The 2->6 states are the class that stays out: they are not integrated at all,
//! their ~1 ms matrix-element cost over a 24-dim map making a meaningful integral
//! prohibitively slow.
//!
//! Every process is driven through the same run-card-pinned setup, so the cut
//! compiler and beam handling are exercised for all of them.
//!
//! # Seed stability is part of the gate's evidence
//!
//! A fixed-seed pull is only meaningful if neighbouring seeds agree. A sampler
//! that occasionally misses a narrow region produces a *confidently wrong* sigma —
//! iterations that miss the region report both a small integral and a small
//! variance, and VEGAS's `1/sigma^2` iteration combination then locks onto them.
//! The `probe_resonant_seed_stability` ignored test sweeps five seeds per resonant
//! row so that failure mode is visible before a row is trusted as a hard gate; the
//! companion probes (`probe_alpha_collapse`, `probe_photon_pole_is_the_instability`,
//! `probe_vegas_iteration_path`, `probe_grid_adaptation_is_the_residue`) separate
//! its possible causes.
//!
//! # Statistical gate
//!
//! With a fixed RNG seed and fixed sampling order the integral is deterministic,
//! so the gate is reproducible (not flaky). The primary check on an asserted
//! process is the standard pull
//!
//! ```text
//! pull = (sigma_vg - sigma_MG) / sqrt(err_vg^2 + err_MG^2)
//! ```
//!
//! required to satisfy `|pull| <= PULL_LIMIT`, backed by a per-process
//! relative-tolerance bound.
//!
//! Runs only when the gitignored MadGraph `output/` tree is present (same
//! contract as `amplitude_oracle`); otherwise every process is skipped.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Deserialize;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use vibegraph::cuts::Cuts;
use vibegraph::hadronic::{
    compile_subprocesses, initial_spin_color_average, process_external_legs, FixedBeamIntegrand,
};
use vibegraph::helas::eval::BoundAmplitude;
use vibegraph::helas::repr::lorentz::LorentzVector;
use vibegraph::phasespace::GEV2_TO_PB;
use vibegraph::runcard::{BeamMode, RunCard};
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::EvaluatedModel;

mod common;

use common::report::{ChannelSummary, IntegralsRow, SeedResult};

/// A pull magnitude above this fails the gate. The banked runs and the vibegraph
/// integral are independent Monte-Carlo estimates, so a few-sigma spread is
/// expected; 3.5 leaves headroom over the nominal 3-sigma target without
/// admitting a genuine normalisation error (which shows up as a many-sigma pull
/// once the budget makes `err_vg` small).
const PULL_LIMIT: f64 = 3.5;

/// Fixed RNG seed — makes the integral (and hence the pull) reproducible.
const SEED: u64 = 20_260_719;

/// Points per survey iteration and number of survey→refine iterations when
/// α-adapting the per-diagram multichannel combiner to each process's `Σ|M|²`.
const MULTICHANNEL_SURVEY: usize = 30_000;
const MULTICHANNEL_ITERS: usize = 6;

/// How a process is exercised by the gate.
enum Plan {
    /// Integrate and assert `|pull| <= PULL_LIMIT` and
    /// `|sigma_vg / sigma_MG - 1| <= rel_tol`.
    Gate {
        neval: usize,
        niter: usize,
        rel_tol: f64,
    },
    /// Integrate and print sigma/pull without asserting: the banked sigma
    /// legitimately differs (recorded reason), so it is a live informational
    /// comparison rather than a pass/fail check.
    ///
    /// The middle rung of the enforcement ladder — a row whose disagreement is
    /// measured and recorded rather than absorbed into a widened `rel_tol`, and
    /// the arm a demotion lands on.
    Info {
        neval: usize,
        niter: usize,
        reason: &'static str,
    },
    /// Not integrated, with a recorded reason (printed, never a failure).
    Skip(&'static str),
}

/// The per-process evaluation plan, keyed by the MadGraph `output/` directory
/// name.
///
/// Gated (asserted) processes are the electroweak final states — the smooth ones
/// and, via the multichannel sampler, the resonant `ee_to_tatah` and `ee_to_mumua`,
/// whose Z/gamma* Breit-Wigner peaks flat RAMBO could not reach — together with the
/// three QCD processes, which are now driven at the run card's own per-event
/// renormalisation scale rather than at the param card's alpha_s. Their budgets are
/// sized to bring `err_vg` near the banked MG error while keeping the default test
/// suite fast. The 2->6 states cost ~1 ms per matrix-element evaluation over a
/// 24-dim map, too slow to integrate meaningfully.
fn plan_for(dir: &str) -> Plan {
    match dir {
        // ── smooth electroweak, asserted ────────────────────────────────────
        // Colored initial state, purely electroweak (alpha_s-independent) s-channel.
        "uux_to_mumu" => Plan::Gate {
            neval: 30_000,
            niter: 6,
            rel_tol: 0.02,
        },
        "ee_to_mumu" => Plan::Gate {
            neval: 30_000,
            niter: 6,
            rel_tol: 0.02,
        },
        "ee_to_ttx" => Plan::Gate {
            neval: 30_000,
            niter: 6,
            rel_tol: 0.02,
        },
        "ee_to_zh" => Plan::Gate {
            neval: 30_000,
            niter: 6,
            rel_tol: 0.02,
        },
        "ee_to_wpwm" => Plan::Gate {
            neval: 40_000,
            niter: 8,
            rel_tol: 0.03,
        },
        "ee_to_ee" => Plan::Gate {
            neval: 80_000,
            niter: 10,
            rel_tol: 0.04,
        },
        // ── QCD, asserted at the run card's own renormalisation scale ───────
        // These three read alpha_s at the scale MadGraph's clustering picks for
        // them, which for a fixed-beam 2 -> 2 is sqrt(s-hat)/2 on every event.
        //
        // Their tolerances are set by the t-channel-peaked integrand, not by the
        // coupling. Over five seeds at these budgets (`probe_qcd_seed_stability`)
        // `gg_to_ttx` holds |pull| <= 0.68 and |rel| <= 8.6e-4, `gg_to_gg` |pull| <=
        // 1.24 and |rel| <= 1.4e-3, and `uux_to_uux` |pull| <= 0.93 and |rel| <=
        // 1.1e-3. Quadrupling the budget shrinks the sweep means (`gg_to_gg` +7.7e-4
        // -> +4.1e-4, `uux_to_uux` +1.9e-4 -> +1.5e-4), which is what says the
        // residual is sampling and not a defect: a bug makes the failure migrate
        // between seeds rather than shrink.
        //
        // The two coloured 2 -> 2 rows carry a jet cut, so their peripheral channels
        // draw the momentum transfer over the fiducial window rather than flat to the
        // collinear edge. That is what removed `uux_to_uux`'s standing negative mean
        // (-0.30% over the same five seeds when every transfer was drawn flat, and
        // not shrinking with budget) and cut the per-point variance with it: the
        // quoted error at the gate budget fell 2.4x on `uux_to_uux` and 2.6x on
        // `gg_to_gg`.
        "gg_to_ttx" => Plan::Gate {
            neval: 60_000,
            niter: 8,
            rel_tol: 0.02,
        },
        "gg_to_gg" => Plan::Gate {
            neval: 40_000,
            niter: 6,
            rel_tol: 0.03,
        },
        "uux_to_uux" => Plan::Gate {
            neval: 40_000,
            niter: 6,
            rel_tol: 0.02,
        },
        // ── sharply resonant electroweak, asserted via the multichannel sampler ──
        // The per-diagram Breit–Wigner combiner resolves the Z/γ* peaks flat RAMBO
        // under-samples, so these converge and are gated against the banked σ. Both
        // hold |pull| <= 1.8 with chi2/dof ≈ 1 across five independent RNG seeds
        // (`probe_resonant_seed_stability`), so the fixed-seed assertion below is a
        // representative draw and not a seed that happens to land well.
        "ee_to_tatah" => Plan::Gate {
            neval: 60_000,
            niter: 8,
            rel_tol: 0.02,
        },
        "ee_to_mumua" => Plan::Gate {
            neval: 80_000,
            niter: 8,
            rel_tol: 0.03,
        },
        // The narrowest resonance under gate: a 6.4 MeV `h -> tau tau` peak inside
        // a 500 GeV range, carrying 5% of the cross section. It reaches the pole
        // through the resonant channel's own mapping, and the tolerance is set by
        // the seed spread of that channel (0.45% over five seeds,
        // `probe_resonant_seed_stability`) rather than by the reference's error.
        //
        // The row is the reason the references are banked with MadGraph 3.7.1.
        // MadGraph 3.5.7 computes the `sde_strategy = 2` channel weight from
        // `(t - Mass)*(t + Mass)` where `t` is already an invariant mass squared
        // (`get_channel_cut`, `genps.f`), so the expression never vanishes on a
        // pole: the resonant channel is given alpha 1.9e-3 instead of 1 - 1.2e-7
        // there, and the 24 non-resonant channels, whose maps cannot resolve
        // 6.4 MeV, are left to find the peak. A 3.5.7 run therefore reports
        // 1.3373e-3 pb, 2.2% low and 7.2 sigma from the sum of its *own* windowed
        // and complementary cross sections. 3.7.1 uses `t - Mass**2` and reports
        // 1.3725e-3 pb, which is where this side already was.
        "ee_to_mumu_tata_qcd0" => Plan::Gate {
            neval: 100_000,
            niter: 8,
            rel_tol: 0.02,
        },
        // ── the multi-rung spine reference, asserted ────────────────────────
        // The one banked row whose diagrams carry a ladder of spacelike lines:
        // 35 channels splitting 12 / 14 / 9 over one, two and three of them, so
        // every channel is a peripheral chain and this cross section is the
        // chain's own end-to-end number — no all-timelike member is there to
        // carry it. QCD = 0 at lpp = 0 with all three scales fixed at m_Z keeps
        // it clear of the strong coupling and the PDFs entirely.
        //
        // The tolerance is the seed spread, not the reference's error. Over five
        // seeds (`probe_resonant_seed_stability`) the row holds |pull| <= 0.96
        // and |rel| <= 2.6e-3 with chi2/dof in 1.00-1.39. Quadrupling the budget
        // does not move it: rel -5.9e-4 / +1.3e-4 / -1.3e-3 at one, two and four
        // times, pulls -0.22 / +0.05 / -0.51, scattering inside a band the
        // reference's own 0.24% error sets and no budget can shrink. A defect
        // would migrate between seeds at fixed size instead.
        "ud_to_epemud_qcd0" => Plan::Gate {
            neval: 120_000,
            niter: 8,
            rel_tol: 0.01,
        },
        // ── llj partonic subprocesses, at the kT-clustered per-event scale ──
        // All four run cards leave both scales free at
        // `dynamical_scale_choice = -1`, so every point carries its own `αs`, and
        // these are the only rows here where the coupling moves under the
        // sampler. Their `σ` is linear in it.
        //
        // The two annihilation rows agree and are asserted; the two rows with a
        // gluon beam do not, and the difference between the pairs is a measured
        // property of the scale rather than of the amplitudes — which are gated
        // at `4e-14` on all four. The scale is a function of the event **and** of
        // the integration channel, and the integrand names channel 1 on every
        // point because the sampled channel is not plumbed through to the scale
        // prescription. On `uux_to_epemg` and `ddx_to_epemg` no banked event needs
        // any other channel; on `gu_to_epemu` and `gux_to_epemux` 7204 and 7231 of
        // 10 000 do. `probe_first_channel_cost_in_alpha_s` prices that in the
        // coupling itself, against MadGraph's own per-event `AQCDUP`: `-2e-9` on
        // the two asserted here, and `-5.540e-2` / `-5.557e-2` on the two below,
        // which are their sigma deviations to two digits.
        //
        // Over five seeds at this budget and at four times it
        // (`probe_llj_parton_seed_stability`) the two asserted rows hold
        // |rel| <= 3.9e-3 and 5.6e-3 with means +2.8e-3 / +4.3e-3, both flat
        // across the ladder and inside twice the banked run's own 0.21% and 0.20%
        // Monte-Carlo error. `rel_tol` is set at 0.01 by that spread rather than
        // by the reference's error, which is the tighter of the two here.
        "uux_to_epemg" | "ddx_to_epemg" => Plan::Gate {
            neval: 60_000,
            niter: 8,
            rel_tol: 0.01,
        },
        // A gluon beam: 5.5% low, on every seed and unmoved by four times the
        // budget, so it is not sampling. Measured and reported rather than
        // absorbed into a widened tolerance — the fix is to give the scale
        // prescription the channel the point was drawn in, which is an integrand
        // change and not a tolerance.
        "gu_to_epemu" | "gux_to_epemux" => Plan::Info {
            neval: 60_000,
            niter: 8,
            reason: "the cluster scale is taken in integration channel 1 on every point; \
                     7204 and 7231 of these runs' 10000 banked events need another channel, \
                     and sigma is 5.5% low, seed-stable and flat in the budget",
        },
        // ── 2->6, not integrated ────────────────────────────────────────────
        "uux_to_ccx_emmm_qcd0" | "bbx_to_ccx_emmm_qcd0" => {
            Plan::Skip("2->6 final state: 24-dim flat RAMBO at ~1 ms/eval is too slow to gate")
        }
        _ => Plan::Skip("no evaluation plan for this directory"),
    }
}

#[derive(Deserialize)]
struct BankedSigma {
    process: String,
    sigma_pb: f64,
    sigma_err_pb: f64,
}

/// Whether a banked run directory is on this machine, and — when it is not —
/// whether the manifest says it is allowed to be.
///
/// A gate that cannot find a run it names is normally looking at an incomplete
/// environment, and [`vibegraph::validation::require`] is what says so. A row the
/// manifest marks `bundled = false` is the declared exception: its artifacts live
/// in a local work area and are deliberately not in the pinned bundle yet, so a
/// checkout that fetched the bundle and does not have the run has a complete
/// environment with respect to what the bundle promises. Such a row is passed
/// over — the report renders the cell as awaiting the bundle — while a missing
/// bundled row still fails.
enum RunPresence {
    /// The run directory and its cards are on this machine.
    Present,
    /// Absent, and the manifest declares the bundle does not carry it.
    AwaitingBundle,
    /// Absent, and the bundle is supposed to carry it.
    Missing,
}

fn run_presence(dir: &str, unbundled: &BTreeSet<String>) -> RunPresence {
    if output_dir().join(dir).join("Cards/run_card.dat").exists() {
        RunPresence::Present
    } else if unbundled.contains(dir) {
        RunPresence::AwaitingBundle
    } else {
        RunPresence::Missing
    }
}

fn output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/output")
}

fn reference_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/sigma_reference.json")
}

/// Load MadGraph's exact param card for a run if present, else the interned SM
/// restrict defaults (a ~1e-7 relative shift in |M|^2, negligible against the
/// Monte-Carlo error of this gate).
fn param_card(dir: &str) -> ParamCard {
    let path = output_dir().join(dir).join("Cards/param_card.dat");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.parse::<ParamCard>().ok())
        .unwrap_or_else(|| "".parse::<ParamCard>().unwrap())
}

/// Integrate one process and return `(sigma_pb, err_pb, chi2_per_dof)`.
fn integrate(dir: &str, process: &str, neval: usize, niter: usize, seed: u64) -> (f64, f64, f64) {
    let (s, e, c, _) = integrate_with(
        dir,
        process,
        neval,
        niter,
        seed,
        MULTICHANNEL_SURVEY,
        MULTICHANNEL_ITERS,
    );
    (s, e, c)
}

/// [`integrate`] with the composition the multichannel map was built by, one
/// entry per sampling channel — the summary the report reprints beside the cross
/// section.
fn integrate_reported(
    dir: &str,
    process: &str,
    neval: usize,
    niter: usize,
    seed: u64,
) -> (f64, f64, f64, Vec<ChannelSummary>) {
    with_integrand(
        dir,
        process,
        seed,
        MULTICHANNEL_SURVEY,
        MULTICHANNEL_ITERS,
        None,
        |integ, _| {
            let result = integ.adapt_grids(neval, niter, seed).1;
            let summary = integ
                .channel_samplers()
                .iter()
                .enumerate()
                .map(|(j, s)| ChannelSummary {
                    channel: format!("diagram {j}"),
                    sampler: s.clone(),
                })
                .collect();
            (
                result.integral * GEV2_TO_PB,
                result.std_dev * GEV2_TO_PB,
                result.chi2_per_dof,
                summary,
            )
        },
    )
}

/// `integrate` with the α-adaptation budget exposed, additionally returning the
/// converged α vector. `n_adapt_iter = 0` leaves the combiner at uniform α.
fn integrate_with(
    dir: &str,
    process: &str,
    neval: usize,
    niter: usize,
    seed: u64,
    n_survey: usize,
    n_adapt_iter: usize,
) -> (f64, f64, f64, Vec<f64>) {
    integrate_probe(
        dir,
        process,
        neval,
        niter,
        seed,
        n_survey,
        n_adapt_iter,
        None,
        None,
    )
}

/// `integrate_with`, optionally overriding the run card's `mmll` (minimum
/// same-flavour lepton-pair invariant mass) before the cuts are compiled. The
/// override is a *diagnostic* — it changes the physics, so the result no longer
/// compares to the banked MG value; it exists to test whether the low-mass
/// photon pole is what destabilises a process.
#[allow(clippy::too_many_arguments)]
fn integrate_probe(
    dir: &str,
    process: &str,
    neval: usize,
    niter: usize,
    seed: u64,
    n_survey: usize,
    n_adapt_iter: usize,
    mmll_override: Option<f64>,
    vegas_alpha: Option<f64>,
) -> (f64, f64, f64, Vec<f64>) {
    with_integrand(
        dir,
        process,
        seed,
        n_survey,
        n_adapt_iter,
        mmll_override,
        |integ, alphas| {
            let result = match vegas_alpha {
                None => integ.adapt_grids(neval, niter, seed).1,
                // Same run, but with the grid-damping exponent under the probe's
                // control: `alpha = 0` freezes the grids, reducing VEGAS to an
                // iteration-averager over the multichannel sampler alone.
                Some(a) => integ.adapt_grids_with(neval, niter, seed, a).1,
            };
            (
                result.integral * GEV2_TO_PB,
                result.std_dev * GEV2_TO_PB,
                result.chi2_per_dof,
                alphas.to_vec(),
            )
        },
    )
}

/// Build the fixed-energy integrand for a banked process — its run card, param
/// card, cuts, amplitudes, per-event renormalisation scale and the α-adapted
/// per-diagram multichannel sampler — and hand it to `f` along with the converged
/// α vector. Everything the integrand borrows lives for the duration of the call,
/// so a caller can drive the same fully-built integrand more than once.
///
/// `mmll_override` patches the run card's minimum same-flavour lepton-pair mass
/// before the cuts are compiled. It is a *diagnostic*: it changes the physics, so
/// a result taken under it no longer compares to the banked MadGraph value.
fn with_integrand<R>(
    dir: &str,
    process: &str,
    seed: u64,
    n_survey: usize,
    n_adapt_iter: usize,
    mmll_override: Option<f64>,
    f: impl FnOnce(&FixedBeamIntegrand, &[f64]) -> R,
) -> R {
    let card_path = output_dir().join(dir).join("Cards/run_card.dat");
    let card_path = match mmll_override {
        None => card_path,
        Some(mmll) => {
            let text = std::fs::read_to_string(&card_path).expect("run card readable");
            let patched: String = text
                .lines()
                .map(|l| {
                    if l.contains("= mmll ") {
                        format!("  {mmll} = mmll ! patched by probe\n")
                    } else {
                        format!("{l}\n")
                    }
                })
                .collect();
            let out = std::env::temp_dir().join(format!("vg_probe_{dir}_{mmll}.dat"));
            std::fs::write(&out, patched).expect("probe run card writable");
            out
        }
    };
    let run_card = RunCard::parse_file(&card_path).expect("real run card parses");
    assert_eq!(
        run_card.beam_mode(),
        BeamMode::FixedEnergy,
        "[{dir}] banked as fixed-energy but run card is not lpp=0"
    );
    let sqrt_s = run_card.ebeam1 + run_card.ebeam2;

    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &param_card(dir));

    let sets = common::generate(process);
    let evals =
        compile_subprocesses(&sets, &model, &evaluated).expect("compile fixed-energy subprocesses");
    let bounds: Vec<_> = evals
        .iter()
        .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
        .collect();

    let rep = &evals[0];
    let legs = process_external_legs(rep, &model, &evaluated);
    let cuts = Cuts::compile(&run_card, &legs)
        .unwrap_or_else(|e| panic!("[{dir}] run card activates a cut vibegraph cannot apply: {e}"));
    let final_masses: Vec<f64> = rep.external_particles()[rep.n_in()..]
        .iter()
        .map(|&id| evaluated.mass(id))
        .collect();
    let spin_color_avg = initial_spin_color_average(rep, &model, &evaluated);

    let diagrams: Vec<_> = sets
        .iter()
        .flat_map(|s| s.diagrams.iter().cloned())
        .collect();

    let amps: Vec<&BoundAmplitude<f64>> = bounds.iter().collect();
    let mut integ = FixedBeamIntegrand::new(amps, &cuts, sqrt_s, final_masses, spin_color_avg);
    // Evaluate alpha_s at the run card's own per-event renormalisation scale, the way
    // MadGraph does, rather than at the param card's value. Installed before the
    // multichannel adaptation so the alpha survey sees the integrand the integration
    // will see. The topology the clustering scale consults is derived from the
    // diagrams, so a process needing the general kT clustering stops here rather than
    // integrating at a plausible wrong scale.
    let scale_report = integ
        .use_running_coupling(&diagrams, &model, &evaluated, &run_card)
        .unwrap_or_else(|e| panic!("[{dir}] cannot run alpha_s to the run card's scale: {e}"));
    assert!(
        scale_report.fallbacks.is_empty(),
        "[{dir}] a subprocess must re-evaluate the model per scale change: {:?}",
        scale_report.fallbacks
    );
    // Promote flat RAMBO to the resonance-aware per-diagram multichannel — the same
    // production sampler `vibegraph integrate` drives — so the narrow electroweak
    // peaks that flat RAMBO under-samples converge. α is adapted to this process's
    // own |M|² on a survey substream disjoint from the integration seed.
    let report = integ.use_multichannel(&diagrams, &evaluated, n_survey, n_adapt_iter, seed);
    let alphas = report
        .map(|r| r.trajectory.last().cloned().unwrap_or_default())
        .unwrap_or_default();
    f(&integ, &alphas)
}

/// Seed-stability sweep for the resonant multichannel rows: integrate each across
/// several RNG seeds and report the per-seed pull, so a seed-unstable coverage
/// defect (a channel set that occasionally misses the dominant region, collapsing
/// σ̂) is visible before a row is trusted as a hard gate. A stable row keeps
/// `|pull| ≲ few` across seeds; an unstable one swings wildly (see
/// `ee_to_mumu_tata_qcd0`, whose worst observed seed lands ~25× low). Run with
/// `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_resonant_seed_stability() {
    let ref_path = reference_path();
    let text = std::fs::read_to_string(&ref_path).unwrap();
    let banked: BTreeMap<String, BankedSigma> = serde_json::from_str(&text).unwrap();
    let seeds = [SEED, 11, 22, 33, 44];
    for (dir, neval, niter) in [
        ("ee_to_tatah", 60_000usize, 8usize),
        ("ee_to_mumua", 80_000, 8),
        ("ee_to_mumu_tata_qcd0", 100_000, 8),
        ("ud_to_epemud_qcd0", 120_000, 8),
    ] {
        let e = &banked[dir];
        eprintln!(
            "── {dir} (MG {:.6e} ± {:.2e}) ──",
            e.sigma_pb, e.sigma_err_pb
        );
        for seed in seeds {
            let (s, err, chi2) = integrate(dir, &e.process, neval, niter, seed);
            let pull = (s - e.sigma_pb) / (err * err + e.sigma_err_pb * e.sigma_err_pb).sqrt();
            eprintln!(
                "  seed {seed:>10}: vg {s:.6e} ± {err:.3e} | pull {pull:+8.2} | \
                 rel {:+.2e} | chi2/dof {chi2:.2}",
                s / e.sigma_pb - 1.0,
            );
        }
        // The second axis: a residual that is sampling shrinks with budget, where a
        // defect migrates between seeds at fixed size.
        for scale in [2usize, 4] {
            let (s, err, chi2) = integrate(dir, &e.process, neval * scale, niter, SEED);
            eprintln!(
                "  budget x{scale}: vg {s:.6e} ± {err:.3e} | rel {:+.2e} | chi2/dof {chi2:.2}",
                s / e.sigma_pb - 1.0,
            );
        }
    }
}

/// The processes the per-channel-grid studies below sweep: a spread of channel
/// counts over resonant and non-resonant integrands, at their gate budgets.
const GRID_STUDY_ROWS: [(&str, usize, usize); 5] = [
    ("ee_to_mumua", 80_000, 8),
    ("ee_to_tatah", 60_000, 8),
    ("ee_to_mumu_tata_qcd0", 100_000, 8),
    ("gg_to_ttx", 60_000, 8),
    ("uux_to_uux", 40_000, 6),
];

/// The figure of merit for the per-channel grid split: **error squared times CPU**
/// at a fixed budget, not time per point.
///
/// One VEGAS grid over the channel mixture and one grid per channel estimate the
/// same integral at (nearly) the same per-point cost, so what separates them is how
/// much variance each buys per second. The two arrangements are run back to back on
/// the *same* fully-built integrand and the same alpha, across seeds, and the quoted
/// ratio is `(dsigma^2 * T)_shared / (dsigma^2 * T)_per-channel` — above 1 means
/// splitting wins.
///
/// The split also spends a little more than the nominal budget: each channel's
/// share `alpha_j * neval` is floored so no channel goes unsampled, so the
/// evaluation counts are reported rather than assumed equal. Run with
/// `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_per_channel_grid_variance_cpu() {
    use std::time::Instant;

    let text = std::fs::read_to_string(reference_path()).unwrap();
    let banked: BTreeMap<String, BankedSigma> = serde_json::from_str(&text).unwrap();
    let seeds = [SEED, 11, 22, 33];

    for (dir, neval, niter) in GRID_STUDY_ROWS {
        let e = &banked[dir];
        eprintln!(
            "-- {dir} (MG {:.6e} +- {:.2e}, {neval}x{niter}) --",
            e.sigma_pb, e.sigma_err_pb
        );
        let mut fom_ratios = Vec::new();
        for seed in seeds {
            with_integrand(
                dir,
                &e.process,
                seed,
                MULTICHANNEL_SURVEY,
                MULTICHANNEL_ITERS,
                None,
                |integ, _alphas| {
                    let t0 = Instant::now();
                    let (_, shared) = integ.adapt_grid(neval, niter, seed);
                    let t_shared = t0.elapsed().as_secs_f64();

                    let t1 = Instant::now();
                    let (channels, split) = integ.adapt_grids(neval, niter, seed);
                    let t_split = t1.elapsed().as_secs_f64();

                    let evals_shared = neval * niter;
                    let evals_split: usize =
                        channels.iter().map(|c| c.neval).sum::<usize>() * niter;
                    let (s_shared, d_shared) =
                        (shared.integral * GEV2_TO_PB, shared.std_dev * GEV2_TO_PB);
                    let (s_split, d_split) =
                        (split.integral * GEV2_TO_PB, split.std_dev * GEV2_TO_PB);
                    let fom_shared = d_shared * d_shared * t_shared;
                    let fom_split = d_split * d_split * t_split;
                    fom_ratios.push(fom_shared / fom_split);
                    eprintln!(
                        "  seed {seed:>10} | shared   sigma {s_shared:.6e} +- {d_shared:.3e} \
                         ({t_shared:.2} s, {evals_shared} evals, chi2/dof {:.2})",
                        shared.chi2_per_dof
                    );
                    eprintln!(
                        "  {:>15} | {:>3} chan sigma {s_split:.6e} +- {d_split:.3e} \
                         ({t_split:.2} s, {evals_split} evals, chi2/dof {:.2})",
                        "",
                        channels.len(),
                        split.chi2_per_dof
                    );
                    eprintln!(
                        "  {:>15} | err ratio {:.3}x | err^2*CPU {fom_shared:.3e} vs \
                         {fom_split:.3e} -> {:.2}x better split",
                        "",
                        d_shared / d_split,
                        fom_shared / fom_split
                    );
                },
            );
        }
        let mean: f64 = fom_ratios.iter().sum::<f64>() / fom_ratios.len() as f64;
        eprintln!(
            "  [{dir}] mean err^2*CPU improvement over {} seeds: {mean:.2}x",
            fom_ratios.len()
        );
    }
}

/// The unweighting normalisation each arrangement hands to an accept/reject pass:
/// a single global `w_max` over the channel mixture versus one `w_max` per channel.
///
/// With events drawn from channel `j` in proportion to `sigma_j` and unweighted
/// against that channel's own maximum, the overall efficiency is
/// `sigma / sum_j w_max_j`, against `sigma / w_max` for the single grid — so the
/// ratio `w_max / sum_j w_max_j` *is* the efficiency change, and the spread of the
/// `w_max_j` says how much of the mixture's maximum was set by one channel. Both
/// maxima are estimated by frozen sampling against the adapted grids, with each
/// channel drawing its production share of the points. Run with
/// `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_unweighting_weight_max() {
    // Frozen draws used to estimate a maximum weight.
    const WMAX_DRAWS: usize = 400_000;
    // Floor on one channel's share of them, so a low-alpha channel's maximum is
    // still estimated from a usable sample.
    const MIN_CHANNEL_DRAWS: usize = 2_000;
    // Seed for the frozen weight scan, distinct from the integration seed.
    const WMAX_SEED: u64 = 0x5CA7_0FF0;

    let text = std::fs::read_to_string(reference_path()).unwrap();
    let banked: BTreeMap<String, BankedSigma> = serde_json::from_str(&text).unwrap();

    for (dir, neval, niter) in GRID_STUDY_ROWS {
        let e = &banked[dir];
        with_integrand(
            dir,
            &e.process,
            SEED,
            MULTICHANNEL_SURVEY,
            MULTICHANNEL_ITERS,
            None,
            |integ, _alphas| {
                // One grid over the mixture: the maximum is taken over every
                // channel's points at once, so the worst channel sets it.
                let (grid, shared) = integ.adapt_grid(neval, niter, SEED);
                let mut rng = ChaCha8Rng::seed_from_u64(WMAX_SEED);
                let mut x = vec![0.0; grid.ndim()];
                let mut w_max_shared = 0.0f64;
                for _ in 0..WMAX_DRAWS {
                    let jac = grid.draw(&mut rng, &mut x);
                    w_max_shared = w_max_shared.max(jac * integ.value(&x) * GEV2_TO_PB);
                }

                // One grid per channel: each channel is unweighted against its own
                // maximum, and the efficiency is set by their sum.
                let (channels, split) = integ.adapt_grids(neval, niter, SEED);
                let total_neval: usize = channels.iter().map(|c| c.neval).sum();
                let mut w_max_each = Vec::with_capacity(channels.len());
                for (j, ch) in channels.iter().enumerate() {
                    let draws = (WMAX_DRAWS * ch.neval / total_neval).max(MIN_CHANNEL_DRAWS);
                    let mut rng = ChaCha8Rng::seed_from_u64(WMAX_SEED);
                    rng.set_stream(1 + j as u64);
                    let mut x = vec![0.0; ch.grid.ndim()];
                    let mut w_max = 0.0f64;
                    for _ in 0..draws {
                        let jac = ch.grid.draw(&mut rng, &mut x);
                        w_max = w_max.max(jac * integ.value_in_channel(j, &x) * GEV2_TO_PB);
                    }
                    w_max_each.push(w_max);
                }

                let sigma_shared = shared.integral * GEV2_TO_PB;
                let sigma_split = split.integral * GEV2_TO_PB;
                let w_max_sum: f64 = w_max_each.iter().sum();
                let w_hi = w_max_each.iter().cloned().fold(0.0f64, f64::max);
                let w_lo = w_max_each
                    .iter()
                    .cloned()
                    .filter(|w| *w > 0.0)
                    .fold(f64::INFINITY, f64::min);
                let eff_shared = sigma_shared / w_max_shared;
                let eff_split = sigma_split / w_max_sum;
                eprintln!(
                    "-- {dir} ({} channels) --\n  \
                     global   w_max     = {w_max_shared:.4e} pb -> unweighting eff {eff_shared:.2e}\n  \
                     per-chan sum w_max = {w_max_sum:.4e} pb -> unweighting eff {eff_split:.2e} \
                     ({:.2}x the global)\n  \
                     per-channel w_max spread: max {w_hi:.4e}, min {w_lo:.4e} (ratio {:.1e}), \
                     largest channel is {:.0}% of the sum",
                    channels.len(),
                    eff_split / eff_shared,
                    w_hi / w_lo.max(f64::MIN_POSITIVE),
                    100.0 * w_hi / w_max_sum,
                );
            },
        );
    }
}

/// What the per-event scale costs, against the matrix element it sits in front of.
///
/// The scale prescription runs on every phase-space point the cuts admit, inside
/// the VEGAS loop, so "negligible" is a measurement rather than a design claim. The
/// same integrand is timed with and without the wiring installed, over one fixed
/// set of uniforms, so the difference is the marshalling, the cluster scale, the
/// `alpha_s` solve and the constant-pool move together. Run with
/// `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_scale_cost() {
    use std::time::Instant;
    for (dir, process) in [
        ("gg_to_gg", "g g > g g"),
        ("gg_to_ttx", "g g > t t~"),
        ("uux_to_uux", "u u~ > u u~"),
    ] {
        let card_path = output_dir().join(dir).join("Cards/run_card.dat");
        let run_card = RunCard::parse_file(&card_path).expect("run card parses");
        let sqrt_s = run_card.ebeam1 + run_card.ebeam2;
        let model = common::sm_model();
        let evaluated = EvaluatedModel::from_model_card(model.clone(), &param_card(dir));
        let sets = common::generate(process);
        let evals = compile_subprocesses(&sets, &model, &evaluated).expect("compile");
        let bounds: Vec<_> = evals
            .iter()
            .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
            .collect();
        let rep = &evals[0];
        let legs = process_external_legs(rep, &model, &evaluated);
        let cuts = Cuts::compile(&run_card, &legs).expect("compile cuts");
        let final_masses: Vec<f64> = rep.external_particles()[rep.n_in()..]
            .iter()
            .map(|&id| evaluated.mass(id))
            .collect();
        let avg = initial_spin_color_average(rep, &model, &evaluated);
        let diagrams: Vec<_> = sets
            .iter()
            .flat_map(|s| s.diagrams.iter().cloned())
            .collect();

        let build = || {
            let amps: Vec<&BoundAmplitude<f64>> = bounds.iter().collect();
            FixedBeamIntegrand::new(amps, &cuts, sqrt_s, final_masses.clone(), avg)
        };
        let plain = build();
        let mut scaled = build();
        scaled
            .use_running_coupling(&diagrams, &model, &evaluated, &run_card)
            .expect("scale prescription compiles");

        let mut rng = ChaCha8Rng::seed_from_u64(SEED);
        let points: Vec<Vec<f64>> = (0..20_000)
            .map(|_| {
                (0..plain.vegas_ndim())
                    .map(|_| rand::Rng::random::<f64>(&mut rng))
                    .collect()
            })
            .collect();

        let time = |integ: &FixedBeamIntegrand| {
            let start = Instant::now();
            let mut acc = 0.0;
            for u in &points {
                acc += integ.value(u);
            }
            std::hint::black_box(acc);
            start.elapsed().as_secs_f64() / points.len() as f64 * 1e9
        };
        // One warm pass each so neither timing pays for a cold cache.
        time(&plain);
        time(&scaled);
        let ns_plain = time(&plain);
        let ns_scaled = time(&scaled);
        eprintln!(
            "[{dir}] {ns_plain:8.1} ns/point fixed coupling | {ns_scaled:8.1} ns/point at the \
             run card's scale | scale + rescale {:+.1} ns ({:+.2}%)",
            ns_scaled - ns_plain,
            (ns_scaled / ns_plain - 1.0) * 100.0,
        );
    }
}

/// Seed-stability sweep and budget ladder for the four `ℓ⁺ℓ⁻ j` partonic rows —
/// the evidence two of them are enforced on and two are not.
///
/// These are the first gated rows whose renormalisation and factorisation scales
/// are recomputed per event by the kT clustering, so the integrand is no longer a
/// smooth function of a fixed coupling: `αs(μR(p))` varies over the phase space
/// the sampler adapts to. A seed sweep alone cannot say whether the residual is
/// sampling — a bias that survives the budget is a defect — so both axes are
/// reported here, five seeds at the gate budget and the same five at four times
/// it. Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_llj_parton_seed_stability() {
    let ref_path = reference_path();
    let text = std::fs::read_to_string(&ref_path).unwrap();
    let banked: BTreeMap<String, BankedSigma> = serde_json::from_str(&text).unwrap();
    let seeds = [SEED, 11, 22, 33, 44];
    for dir in LLJ_PARTON_ROWS {
        // The budget the gate itself integrates at, so the `1x` rung is the row's
        // own cell and the `4x` rung is what says whether it is converged.
        let (neval, niter) = match plan_for(dir) {
            Plan::Gate { neval, niter, .. } | Plan::Info { neval, niter, .. } => (neval, niter),
            Plan::Skip(_) => continue,
        };
        let e = &banked[dir];
        eprintln!(
            "── {dir} (MG {:.6e} ± {:.2e}) ──",
            e.sigma_pb, e.sigma_err_pb
        );
        for budget in [1usize, 4] {
            let mut rels = Vec::new();
            for seed in seeds {
                let (s, err, chi2) = integrate(dir, &e.process, neval * budget, niter, seed);
                let pull = (s - e.sigma_pb) / (err * err + e.sigma_err_pb * e.sigma_err_pb).sqrt();
                rels.push(s / e.sigma_pb - 1.0);
                eprintln!(
                    "  {:>7} seed {seed:>10}: vg {s:.6e} ± {err:.3e} | pull {pull:+8.2} | \
                     rel {:+.2e} | chi2/dof {chi2:.2}",
                    format!("{}x", budget),
                    s / e.sigma_pb - 1.0,
                );
            }
            let mean = rels.iter().sum::<f64>() / rels.len() as f64;
            let worst = rels.iter().fold(0.0f64, |a, r| a.max(r.abs()));
            eprintln!("  {budget}x mean rel {mean:+.2e} | worst |rel| {worst:.2e}");
        }
    }
}

/// The four `ℓ⁺ℓ⁻ j` partonic rows, the ones whose scales the kT clustering
/// recomputes on every point.
const LLJ_PARTON_ROWS: [&str; 4] = [
    "uux_to_epemg",
    "ddx_to_epemg",
    "gu_to_epemu",
    "gux_to_epemux",
];

/// Seed-stability sweep for the three QCD rows, the evidence their hard gate rests
/// on.
///
/// A single-seed pull cannot distinguish a converged integral from a sampler that
/// occasionally misses a region: an iteration that misses reports a small integral
/// *and* a small variance, and VEGAS's `1/sigma^2` combination then lets those
/// iterations dominate — a confidently wrong sigma rather than a noisy one. The
/// diagnostic that separates the two is what happens when the budget grows: genuine
/// under-sampling shrinks with it, while a bug makes the failure migrate between
/// seeds. Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_qcd_seed_stability() {
    let ref_path = reference_path();
    let text = std::fs::read_to_string(&ref_path).unwrap();
    let banked: BTreeMap<String, BankedSigma> = serde_json::from_str(&text).unwrap();
    let seeds = [SEED, 11, 22, 33, 44];
    for (dir, neval, niter) in [
        ("gg_to_ttx", 60_000usize, 8usize),
        ("gg_to_gg", 40_000, 6),
        ("uux_to_uux", 40_000, 6),
    ] {
        let e = &banked[dir];
        eprintln!(
            "── {dir} (MG {:.6e} ± {:.2e}) ──",
            e.sigma_pb, e.sigma_err_pb
        );
        for budget in [1usize, 4] {
            for seed in seeds {
                let (s, err, chi2) = integrate(dir, &e.process, neval * budget, niter, seed);
                let pull = (s - e.sigma_pb) / (err * err + e.sigma_err_pb * e.sigma_err_pb).sqrt();
                eprintln!(
                    "  {:>7} seed {seed:>10}: vg {s:.6e} ± {err:.3e} | pull {pull:+8.2} | \
                     rel {:+.2e} | chi2/dof {chi2:.2}",
                    format!("{}x", budget),
                    s / e.sigma_pb - 1.0,
                );
            }
        }
    }
}

/// Are the per-diagram channel maps of a massless-propagator process distinct, and
/// does the α-adaptation move off uniform?
///
/// The standing finding this re-measures: with the spacelike transfer drawn flat —
/// which is what an unregulated massless line gives — every `u u~ → u u~` channel
/// density was bit-identical at every probed point, all four of `g g → g g` were,
/// and the Kleiss–Pittau reallocation therefore had nothing to reallocate. Reported
/// as the worst *pairwise* relative density difference over drawn points (a set
/// whose members coincide reads zero) beside the converged α, with `g g → t t~` —
/// whose t/u maps carry a 173 GeV pole and were never degenerate — as the control
/// that says the instrument can tell the two apart.
///
/// Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_channel_map_degeneracy() {
    use vibegraph::phasespace::rng::SubStream;
    use vibegraph::phasespace::{Channel, DiagramChannel, PhaseSpaceMap};

    let ref_path = reference_path();
    let text = std::fs::read_to_string(&ref_path).unwrap();
    let banked: BTreeMap<String, BankedSigma> = serde_json::from_str(&text).unwrap();
    let model = common::sm_model();
    for (dir, neval, niter) in [
        ("uux_to_uux", 40_000usize, 6usize),
        ("gg_to_gg", 40_000, 6),
        ("gg_to_ttx", 60_000, 8),
    ] {
        let entry = &banked[dir];
        let run_card = RunCard::parse_file(&output_dir().join(dir).join("Cards/run_card.dat"))
            .expect("banked run card parses");
        let sqrt_s = run_card.ebeam1 + run_card.ebeam2;
        let evaluated = EvaluatedModel::from_model_card(model.clone(), &param_card(dir));
        let sets = common::generate(&entry.process);
        let evals = compile_subprocesses(&sets, &model, &evaluated).expect("compile subprocesses");
        let rep = &evals[0];
        let legs = process_external_legs(rep, &model, &evaluated);
        let cuts = Cuts::compile(&run_card, &legs).expect("cuts compile");
        let floor = cuts.spacelike_floor();
        let diagrams: Vec<_> = sets
            .iter()
            .flat_map(|s| s.diagrams.iter().cloned())
            .collect();
        let channels: Vec<DiagramChannel<f64>> = diagrams
            .iter()
            .map(|d| DiagramChannel::from_diagram_regulated(d, &evaluated, sqrt_s, floor))
            .collect();

        let mut stream = SubStream::from_stream(SEED, 0xD1FF);
        let mut worst = 0.0f64;
        let mut coincident = 0usize;
        let mut pairs = 0usize;
        for _ in 0..2_000 {
            let u = stream.uniforms::<f64>(channels[0].ndim());
            let p = channels[0].sample(&u).momenta;
            let d: Vec<f64> = channels.iter().map(|c| c.density(&p)).collect();
            for i in 0..d.len() {
                for j in (i + 1)..d.len() {
                    pairs += 1;
                    let rel =
                        (d[i] - d[j]).abs() / d[i].abs().max(d[j].abs()).max(f64::MIN_POSITIVE);
                    if rel == 0.0 {
                        coincident += 1;
                    }
                    worst = worst.max(rel);
                }
            }
        }
        let (_, _, _, alphas) = integrate_with(
            dir,
            &entry.process,
            neval,
            niter,
            SEED,
            MULTICHANNEL_SURVEY,
            MULTICHANNEL_ITERS,
        );
        let spread = alphas.iter().cloned().fold(0.0f64, f64::max)
            - alphas.iter().cloned().fold(f64::INFINITY, f64::min);
        eprintln!(
            "  {dir:>12} floor {floor:>5.0} GeV², {} channels: worst pairwise density difference \
             {worst:.3e}, {coincident} of {pairs} pairs bit-identical; α {alphas:?} (spread \
             {spread:.3e} about uniform {:.4})",
            channels.len(),
            1.0 / channels.len() as f64,
        );
    }
}

/// Diagnose *why* `ee_to_mumu_tata_qcd0` is seed-unstable: is the estimator
/// merely under-sampled (more points would fix it), or is the α-adaptation
/// collapsing the channel mixture (more points would not)?
///
/// Each seed is run at a fixed integration budget under three α regimes —
/// un-adapted uniform α, the production survey budget, and a 10x survey — so the
/// two hypotheses separate: if σ tracks the survey budget the defect is
/// statistical, if uniform α is stable while adapted α collapses the defect is in
/// the reallocation. `n_dead` counts channels driven to the 1e-12 floor.
/// Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_alpha_collapse() {
    let ref_path = reference_path();
    let text = std::fs::read_to_string(&ref_path).unwrap();
    let banked: BTreeMap<String, BankedSigma> = serde_json::from_str(&text).unwrap();
    let dir = "ee_to_mumu_tata_qcd0";
    let e = &banked[dir];
    eprintln!(
        "── {dir} (MG {:.6e} ± {:.2e}) ──",
        e.sigma_pb, e.sigma_err_pb
    );
    for (label, n_survey, n_iter) in [
        ("uniform-alpha ", 0usize, 0usize),
        ("survey  30k x6", 30_000, 6),
        ("survey 300k x6", 300_000, 6),
    ] {
        for seed in [SEED, 11, 22, 33, 44] {
            let (s, err, chi2, alphas) =
                integrate_with(dir, &e.process, 100_000, 8, seed, n_survey, n_iter);
            let n_dead = alphas.iter().filter(|&&a| a < 1e-9).count();
            let a_max = alphas.iter().cloned().fold(0.0_f64, f64::max);
            eprintln!(
                "  {label} seed {seed:>10}: vg {s:.6e} | rel {:+.2e} | chi2/dof {chi2:9.2} \
                 | nch {:3} dead {n_dead:3} amax {a_max:.3e} | err {err:.2e}",
                s / e.sigma_pb - 1.0,
                alphas.len(),
            );
        }
    }
}

/// Test whether the low-mass photon pole is what destabilises
/// `ee_to_mumu_tata_qcd0`. The run card leaves `mmll = 0`, so the s-channel
/// `gamma* -> l+ l-` invariant is unbounded below and the integrand rises as
/// `1/s_ll^2` toward the edge — while `draw_invariant` gives a zero-width pole a
/// *flat* draw. Raising `mmll` truncates that rise without touching anything
/// else. If the seed instability disappears as `mmll` grows, the flat draw over
/// the photon pole is the cause. The absolute sigma is not comparable to MG here
/// (the cut is different by construction) — only the seed spread is the signal.
/// Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_photon_pole_is_the_instability() {
    let ref_path = reference_path();
    let text = std::fs::read_to_string(&ref_path).unwrap();
    let banked: BTreeMap<String, BankedSigma> = serde_json::from_str(&text).unwrap();
    let dir = "ee_to_mumu_tata_qcd0";
    let e = &banked[dir];
    eprintln!("── {dir}: seed spread vs. the mmll floor on the photon pole ──");
    for mmll in [0.0, 5.0, 20.0, 50.0] {
        let mut sigmas = Vec::new();
        let mut worst_chi2: f64 = 0.0;
        for seed in [SEED, 11, 22, 33, 44] {
            let (s, _, chi2, _) = integrate_probe(
                dir,
                &e.process,
                100_000,
                8,
                seed,
                MULTICHANNEL_SURVEY,
                MULTICHANNEL_ITERS,
                Some(mmll),
                None,
            );
            sigmas.push(s);
            worst_chi2 = worst_chi2.max(chi2);
        }
        let mean = sigmas.iter().sum::<f64>() / sigmas.len() as f64;
        let lo = sigmas.iter().cloned().fold(f64::INFINITY, f64::min);
        let hi = sigmas.iter().cloned().fold(0.0_f64, f64::max);
        eprintln!(
            "  mmll {mmll:5.1} GeV: mean {mean:.4e} | spread hi/lo {:8.2}x \
             | worst chi2/dof {worst_chi2:9.2}",
            hi / lo,
        );
    }
}

/// Walk the VEGAS iteration count for the seeds that collapse, to separate a
/// *sampling* failure from a *grid-adaptation* failure. Iterations are keyed by
/// substream, so running with `niter = 1..N` replays the same first `k`
/// iterations each time and the cumulative estimate exposes the path. A sampler
/// that simply misses a spike shows one bad iteration among good ones (large
/// chi2, estimate recovering); a grid collapsing into a corner shows the estimate
/// walking monotonically away as iterations accumulate. Run with
/// `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_vegas_iteration_path() {
    let ref_path = reference_path();
    let text = std::fs::read_to_string(&ref_path).unwrap();
    let banked: BTreeMap<String, BankedSigma> = serde_json::from_str(&text).unwrap();
    let dir = "ee_to_mumu_tata_qcd0";
    let e = &banked[dir];
    eprintln!("── {dir} (MG {:.6e}) ──", e.sigma_pb);
    for seed in [SEED, 11] {
        eprintln!("  seed {seed}:");
        for niter in 1..=8 {
            let (s, err, chi2, _) = integrate_with(
                dir,
                &e.process,
                100_000,
                niter,
                seed,
                MULTICHANNEL_SURVEY,
                MULTICHANNEL_ITERS,
            );
            eprintln!(
                "    niter {niter}: vg {s:.6e} ± {err:.3e} | rel {:+.2e} | chi2/dof {chi2:9.2}",
                s / e.sigma_pb - 1.0,
            );
        }
    }
}

/// Is the residual collapse VEGAS's grid adaptation, or the multichannel sampler?
///
/// `vegas_alpha` is the grid-damping exponent: at `0` the grid never refines, so
/// the run is a pure multichannel Monte Carlo over the α-adapted combiner with
/// VEGAS reduced to an iteration-averager. If the collapse disappears at
/// `alpha = 0` and returns as `alpha` grows, the defect is grid adaptation
/// (a single grid shared across channels whose coordinates mean different
/// invariants in each), not channel coverage. Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_grid_adaptation_is_the_residue() {
    let ref_path = reference_path();
    let text = std::fs::read_to_string(&ref_path).unwrap();
    let banked: BTreeMap<String, BankedSigma> = serde_json::from_str(&text).unwrap();
    let dir = "ee_to_mumu_tata_qcd0";
    let e = &banked[dir];
    eprintln!(
        "── {dir} (MG {:.6e}): sigma vs. the VEGAS damping exponent ──",
        e.sigma_pb
    );
    for vegas_alpha in [0.0, 0.5, 1.5] {
        for seed in [SEED, 11, 22, 33, 44] {
            let (s, err, chi2, _) = integrate_probe(
                dir,
                &e.process,
                100_000,
                8,
                seed,
                MULTICHANNEL_SURVEY,
                MULTICHANNEL_ITERS,
                None,
                Some(vegas_alpha),
            );
            eprintln!(
                "  vegas_alpha {vegas_alpha:3.1} seed {seed:>10}: vg {s:.6e} ± {err:.3e} \
                 | rel {:+.2e} | chi2/dof {chi2:9.2}",
                s / e.sigma_pb - 1.0,
            );
        }
    }
}

/// The signed `(pull, relative_deviation)` of a vibegraph integral against a
/// banked value. Signed, because a table of magnitudes hides whether a family of
/// rows leans the same way; the gate asserts on the magnitudes.
fn compare(sigma_vg: f64, err_vg: f64, banked: &BankedSigma) -> (f64, f64) {
    let denom = (err_vg * err_vg + banked.sigma_err_pb * banked.sigma_err_pb).sqrt();
    let pull = (sigma_vg - banked.sigma_pb) / denom;
    let rel = sigma_vg / banked.sigma_pb - 1.0;
    (pull, rel)
}

/// Drive the gate for one banked directory. `Ok(())` on pass, skip, or info;
/// `Err(reason)` only on a failed assertion.
///
/// Every non-skipped row writes its measurement to the report directory,
/// including a failing one: the report is what ran, not what passed.
fn gate_dir(dir: &str, banked: &BankedSigma) -> Result<(), String> {
    let (neval, niter, mode, reason, rel_tol) = match plan_for(dir) {
        Plan::Skip(reason) => {
            eprintln!("[{dir}] SKIP ({reason})");
            return Ok(());
        }
        Plan::Info {
            neval,
            niter,
            reason,
        } => (neval, niter, "info", Some(reason), None),
        Plan::Gate {
            neval,
            niter,
            rel_tol,
        } => (neval, niter, "gate", None, Some(rel_tol)),
    };

    let (sigma_vg, err_vg, chi2, subsampler) =
        integrate_reported(dir, &banked.process, neval, niter, SEED);
    let (pull, rel) = compare(sigma_vg, err_vg, banked);
    eprintln!(
        "[{dir}] {} vg = {sigma_vg:.6e} +- {err_vg:.3e} pb | MG = {:.6e} +- {:.3e} pb | \
         pull = {pull:+.2} | rel = {rel:+.2e} | chi2/dof = {chi2:.2} ({neval}x{niter}){}",
        mode.to_uppercase(),
        banked.sigma_pb,
        banked.sigma_err_pb,
        reason.map(|r| format!("  <{r}>")).unwrap_or_default()
    );

    let failure = rel_tol.and_then(|tol| {
        if pull.abs() > PULL_LIMIT {
            Some(format!(
                "[{dir}] |pull| = {:.2} exceeds {PULL_LIMIT} \
                 (vg {sigma_vg:.6e} +- {err_vg:.3e} vs MG {:.6e} +- {:.3e})",
                pull.abs(),
                banked.sigma_pb,
                banked.sigma_err_pb
            ))
        } else if rel.abs() > tol {
            Some(format!(
                "[{dir}] relative deviation {rel:+.2e} exceeds tol {tol:.2e} \
                 (vg {sigma_vg:.6e} vs MG {:.6e})",
                banked.sigma_pb
            ))
        } else {
            None
        }
    });

    let mut row = IntegralsRow::new(dir, &banked.process, mode);
    row.status = match (mode, &failure) {
        ("gate", None) => "pass",
        ("gate", Some(_)) => "fail",
        _ => "info",
    };
    row.sigma_vg_pb = sigma_vg;
    row.sigma_vg_err_pb = err_vg;
    row.sigma_mg_pb = banked.sigma_pb;
    row.sigma_mg_err_pb = banked.sigma_err_pb;
    row.pull = pull;
    row.rel = rel;
    row.chi2_dof = chi2;
    row.seeds = vec![SEED];
    row.per_seed = vec![SeedResult {
        seed: SEED,
        sigma_pb: sigma_vg,
        sigma_err_pb: err_vg,
    }];
    row.neval = neval;
    row.niter = niter;
    row.subsampler = subsampler;
    row.note = reason.map(str::to_string);
    row.write();

    match failure {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

/// A row the bundle does not carry may be absent; a row it carries may not.
///
/// The two halves are the whole of the declared-absent rule, and each is the
/// other's control: without the first, a checkout that fetched the pinned bundle
/// fails on a run the bundle never promised it, and without the second, a genuinely
/// incomplete environment passes silently. The gate's own iteration is what
/// consumes [`run_presence`], so it is exercised here directly rather than by
/// arranging a work area.
#[test]
fn a_row_the_bundle_does_not_carry_may_be_absent() {
    let unbundled = common::manifest::unbundled_rows();
    assert!(
        !unbundled.is_empty(),
        "no manifest row is marked bundled = false, so this rule has nothing to check \
         and the gate's tolerance is untested"
    );

    // Absent and declared absent: passed over.
    let declared = unbundled.iter().next().unwrap();
    assert!(
        matches!(
            run_presence("no-such-run-directory", &unbundled),
            RunPresence::Missing
        ),
        "a directory no manifest row names is not exempt from anything"
    );
    let absent_and_declared: BTreeSet<String> =
        std::iter::once("no-such-run-directory".to_string()).collect();
    assert!(
        matches!(
            run_presence("no-such-run-directory", &absent_and_declared),
            RunPresence::AwaitingBundle
        ),
        "a row the manifest marks bundled = false may be absent"
    );

    // Present is present whatever the manifest says about the bundle, so a machine
    // that has the run measures it rather than passing over it.
    if output_dir()
        .join(declared)
        .join("Cards/run_card.dat")
        .exists()
    {
        assert!(
            matches!(run_presence(declared, &unbundled), RunPresence::Present),
            "an unbundled row whose run is on this machine must still be measured"
        );
    }
}

/// The other half, as the failure it has to stay: a missing run the bundle *does*
/// carry is an incomplete environment and says so.
#[test]
#[should_panic(expected = "needs a banked run card")]
fn a_row_the_bundle_carries_may_not_be_absent() {
    let unbundled = common::manifest::unbundled_rows();
    match run_presence("uux_to_mumu_but_misspelt", &unbundled) {
        RunPresence::Missing => vibegraph::validation::require(
            "sigma_gate_matches_madgraph",
            "a banked run card",
            "uux_to_mumu_but_misspelt",
        ),
        other => panic!(
            "a bundled row's missing run classified as {}",
            match other {
                RunPresence::Present => "present",
                RunPresence::AwaitingBundle => "awaiting the bundle",
                RunPresence::Missing => unreachable!(),
            }
        ),
    }
}

/// Draws the coverage sweep is measured over. Flat, so every accepted point is an
/// independent chance for the bound to have renounced something the cuts keep.
const COVERAGE_DRAWS: usize = 100_000;

/// How far the control pushes each rung's transfer bound out beyond the scale the
/// cuts imply. Two orders: the bound ladder measured on this crate's ladder cuts
/// first loses fiducial phase space at a hundred times the cut scale, so a control
/// at that factor is the nearest one that has to fire.
const CONTROL_BOUND_FACTOR: f64 = 100.0;

/// Every gated process whose peripheral channels are bounded still reaches
/// everywhere its own cuts accept.
///
/// [`Cuts::spacelike_floor`] does two things to a peripheral channel. It floors the
/// propagator pole, which enters the draw and the density alike and so only
/// reshapes the sampling; and it bounds each rung's transfer at `t ≤ −floor`, which
/// narrows the channel's *support* — above the bound the density is an exact zero.
/// A set of narrowed channels integrates the right thing only if between them they
/// still cover everywhere the integrand lives, and the bound's `pT_min²` argument
/// is a proof only up to three outgoing legs. Past that it is a scale, so coverage
/// is measured here per process rather than inherited from the process it was first
/// measured on.
///
/// The measurement is on the sharpest available integrand, the cut indicator
/// itself, over flat RAMBO: every drawn point the cuts accept must be reachable by
/// some channel. A process whose set keeps an unbounded member — an s-channel or
/// contact diagram, whose tree spans the whole final state — passes that trivially,
/// so the count of points only a bounded channel reaches is reported beside it and
/// is what says whether the process constrains anything.
///
/// The control is the second half. The same channels with every bound pushed out by
/// [`CONTROL_BOUND_FACTOR`] have to *lose* accepted points somewhere, or the pass
/// above is a property of the cuts rather than of where the bound sits.
#[test]
fn every_bounded_channel_set_covers_its_own_fiducial_region() {
    use vibegraph::phasespace::rng::SubStream;
    use vibegraph::phasespace::{Channel, DiagramChannel, PhaseSpaceMap, RamboChannel};

    let ref_path = reference_path();
    let text = std::fs::read_to_string(&ref_path).expect("sigma reference readable");
    let banked: BTreeMap<String, BankedSigma> =
        serde_json::from_str(&text).expect("sigma_reference.json parses");

    let model = common::sm_model();
    let unbundled = common::manifest::unbundled_rows();
    let mut bounded_processes = 0usize;
    let mut control_fired = 0usize;
    let mut constraining = 0usize;
    let mut awaiting = 0usize;
    for (dir, entry) in &banked {
        // Every row this suite integrates, gated or informational — an unenforced
        // cross section is drawn through the same channel set and would inherit the
        // same bias if the bound renounced something the cuts keep.
        if matches!(plan_for(dir), Plan::Skip(_)) {
            continue;
        }
        match run_presence(dir, &unbundled) {
            RunPresence::Present => {}
            RunPresence::AwaitingBundle => {
                eprintln!("[{dir}] awaiting the bundle: no run on this machine, nothing to cover");
                awaiting += 1;
                continue;
            }
            RunPresence::Missing => vibegraph::validation::require(
                "every_bounded_channel_set_covers_its_own_fiducial_region",
                "a banked run card",
                dir,
            ),
        }
        let run_card = RunCard::parse_file(&output_dir().join(dir).join("Cards/run_card.dat"))
            .expect("banked run card parses");
        let sqrt_s = run_card.ebeam1 + run_card.ebeam2;
        let evaluated = EvaluatedModel::from_model_card(model.clone(), &param_card(dir));
        let sets = common::generate(&entry.process);
        let evals = compile_subprocesses(&sets, &model, &evaluated).expect("compile subprocesses");
        let rep = &evals[0];
        let legs = process_external_legs(rep, &model, &evaluated);
        let cuts = Cuts::compile(&run_card, &legs).expect("run card cuts compile");
        let floor = cuts.spacelike_floor();

        let particles = rep.external_particles();
        let n_in = rep.n_in();
        let masses: Vec<f64> = particles[n_in..]
            .iter()
            .map(|&id| evaluated.mass(id))
            .collect();
        let beams: Vec<LorentzVector<f64>> = (0..n_in)
            .map(|a| {
                let m = evaluated.mass(particles[a]);
                let e = sqrt_s / 2.0;
                let pz = (e * e - m * m).max(0.0).sqrt();
                LorentzVector::new(e, 0.0, 0.0, if a == 0 { pz } else { -pz })
            })
            .collect();

        let diagrams: Vec<_> = sets
            .iter()
            .flat_map(|s| s.diagrams.iter().cloned())
            .collect();
        let build = |cap: Option<f64>| -> Vec<DiagramChannel<f64>> {
            diagrams
                .iter()
                .map(|d| {
                    let ch = DiagramChannel::from_diagram_regulated(d, &evaluated, sqrt_s, floor);
                    match cap {
                        Some(t_max) => ch.with_fiducial_t_max(t_max),
                        None => ch,
                    }
                })
                .collect()
        };
        let channels = build(None);
        let peripheral = channels
            .iter()
            .filter(|c| !c.spine_poles().is_empty())
            .count();
        if floor == 0.0 || peripheral == 0 {
            eprintln!(
                "[{dir}] floor {floor} GeV², {peripheral} of {} channels peripheral: \
                 nothing bounded, nothing to cover",
                channels.len()
            );
            continue;
        }
        let widened = build(Some(-floor * CONTROL_BOUND_FACTOR));

        let flat = RamboChannel::new(sqrt_s, masses.clone());
        let mut stream = SubStream::from_stream(SEED, 0xC0FE);
        let mut ext = beams.clone();
        let (mut fiducial, mut reachable, mut only_bounded, mut reachable_wide) = (0, 0, 0, 0);
        for _ in 0..COVERAGE_DRAWS {
            let u = stream.uniforms::<f64>(flat.ndim());
            let p = flat.sample(&u).momenta;
            ext.truncate(n_in);
            ext.extend_from_slice(&p);
            if !cuts.pass(&ext) {
                continue;
            }
            fiducial += 1;
            let hit = |set: &[DiagramChannel<f64>]| set.iter().any(|c| c.density(&p) > 0.0);
            if hit(&channels) {
                reachable += 1;
            }
            if !channels
                .iter()
                .any(|c| c.spine_poles().is_empty() && c.density(&p) > 0.0)
            {
                only_bounded += 1;
            }
            if hit(&widened) {
                reachable_wide += 1;
            }
        }
        eprintln!(
            "[{dir}] floor {floor:.0} GeV², {peripheral}/{} channels peripheral: {fiducial} of \
             {COVERAGE_DRAWS} draws accepted, {reachable} reachable ({only_bounded} of them only \
             by a bounded channel); at {CONTROL_BOUND_FACTOR:.0}x the bound, {reachable_wide}",
            channels.len()
        );
        assert!(
            fiducial > 0 && fiducial < COVERAGE_DRAWS,
            "[{dir}] the cut indicator is not selecting a proper subregion, so coverage \
             here would be vacuous"
        );
        assert_eq!(
            fiducial,
            reachable,
            "[{dir}] the bounded channel set leaves {} accepted point(s) unreachable, so its \
             densities cannot normalise a combiner over the fiducial region",
            fiducial - reachable
        );
        bounded_processes += 1;
        if only_bounded > 0 {
            constraining += 1;
        }
        if reachable_wide < fiducial {
            control_fired += 1;
        }
    }

    eprintln!(
        "coverage: {bounded_processes} bounded process(es), {constraining} of them with points \
         no unbounded channel reaches, control fires on {control_fired}; {awaiting} row(s) \
         awaiting the bundle"
    );
    assert!(
        bounded_processes > 0,
        "no process builds a bounded peripheral channel, so this measures nothing"
    );
    // Coverage above is asserted per process regardless. What the two checks below
    // add is that the assertion is not vacuous — that some process has accepted
    // points only a *bounded* channel reaches, and that moving the bound would lose
    // them. Only a process whose whole channel set is peripheral shows that, and on
    // this reference set exactly one does. A checkout the bundle has not yet reached
    // that row on cannot demonstrate it, and is told so rather than passing quietly;
    // a checkout that has every row must.
    if constraining == 0 && awaiting > 0 {
        eprintln!(
            "coverage: no available process has points only a bounded channel reaches, so the \
             bound's non-vacuity rests on the {awaiting} row(s) this checkout does not have yet"
        );
        return;
    }
    assert!(
        constraining > 0,
        "every accepted point is reachable by an unbounded channel on every process, so the \
         coverage pass says nothing about the bound"
    );
    assert!(
        control_fired > 0,
        "pushing every bound out by {CONTROL_BOUND_FACTOR}x loses no accepted point anywhere, \
         so the coverage check cannot see where the bound sits"
    );
}

#[test]
fn sigma_gate_matches_madgraph() {
    let ref_path = reference_path();
    let text = std::fs::read_to_string(&ref_path)
        .unwrap_or_else(|e| panic!("missing sigma reference {}: {e}", ref_path.display()));
    let banked: BTreeMap<String, BankedSigma> =
        serde_json::from_str(&text).expect("sigma_reference.json parses");

    if !output_dir().exists() {
        vibegraph::validation::require(
            "sigma_gate_matches_madgraph",
            "the banked MadGraph work area",
            output_dir().display(),
        );
    }

    let unbundled = common::manifest::unbundled_rows();
    let mut failures = Vec::new();
    let mut asserted = 0usize;
    let mut awaiting = Vec::new();
    for (dir, entry) in &banked {
        match run_presence(dir, &unbundled) {
            RunPresence::Present => {}
            RunPresence::AwaitingBundle => {
                eprintln!(
                    "[{dir}] AWAITING BUNDLE (the manifest marks this row bundled = false and \
                     this checkout does not have its run, so no cell is written for it)"
                );
                awaiting.push(dir.as_str());
                continue;
            }
            RunPresence::Missing => vibegraph::validation::require(
                "sigma_gate_matches_madgraph",
                "a banked run card",
                dir,
            ),
        }
        if matches!(plan_for(dir), Plan::Gate { .. }) {
            asserted += 1;
        }
        if let Err(e) = gate_dir(dir, entry) {
            failures.push(e);
        }
    }

    eprintln!(
        "sigma gate: {asserted} process(es) asserted against banked MadGraph sigma, \
         {} awaiting the bundle ({})",
        awaiting.len(),
        if awaiting.is_empty() {
            "none".to_string()
        } else {
            awaiting.join(", ")
        }
    );
    // A row passed over above writes no cell, so a reference whose runs had all gone
    // missing would leave this gate asserting nothing at all and still passing.
    assert!(
        asserted > 0,
        "no banked run on this machine is asserted, so the sigma gate measured nothing"
    );
    assert!(
        failures.is_empty(),
        "sigma gate failures:\n{}",
        failures.join("\n")
    );
}
