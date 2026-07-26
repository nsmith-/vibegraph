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
//! The per-point `validate_helas_mg` net is blind to everything *outside* the
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
//! This gate is also blind
//! to differences in *couplings evaluated outside the matrix element*: MadGraph
//! runs alpha_s to a per-event dynamical scale (`fixed_ren_scale = False`),
//! whereas this integration uses the fixed param-card alpha_s, so the sigma of an
//! alpha_s-dependent (QCD) process differs by the running even when its |M|^2 is
//! bit-exact. The bit-exact net remains the fine instrument; agreement here
//! confirms the *normalisation and averaging* of the electroweak cross sections.
//!
//! # Which processes are asserted
//!
//! The electroweak final states are gated with an assertion, now including the
//! sharply resonant `e+ e- > ta+ ta- h` and `e+ e- > mu+ mu- a`: the multichannel
//! sampler resolves the Z/gamma* Breit-Wigner peaks flat RAMBO under-sampled, so
//! they converge to the banked sigma and hold across independent RNG seeds.
//!
//! Three classes are integrated *informationally* (printed, not asserted). The QCD
//! processes differ from the banked value by the dynamical-scale alpha_s running
//! described above. `e+ e- > mu+ mu- ta+ ta-` samples stably but sits ~3.0% above
//! the banked sigma, an offset localised at low lepton-pair mass and not yet
//! reconciled (see its `Plan::Info` reason). The 2->6 states are not integrated at
//! all — their ~1 ms matrix-element cost over a 24-dim map makes a meaningful
//! integral prohibitively slow.
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
//! contract as `validate_helas_mg`); otherwise every process is skipped.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use vibegraph::cuts::Cuts;
use vibegraph::hadronic::{
    compile_subprocesses, initial_spin_color_average, process_external_legs, FixedBeamIntegrand,
    VEGAS_NBINS,
};
use vibegraph::helas::eval::BoundAmplitude;
use vibegraph::phasespace::GEV2_TO_PB;
use vibegraph::runcard::{BeamMode, RunCard};
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::EvaluatedModel;
use vibegraph::vegas::VegasGrid;

mod common;

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
/// whose Z/gamma* Breit-Wigner peaks flat RAMBO could not reach. Their budgets are
/// sized to bring `err_vg` near the banked MG error while keeping the default test
/// suite fast. QCD processes are
/// informational — MadGraph runs alpha_s to a per-event dynamical scale, so their
/// banked sigma differs from a fixed-alpha_s integral by the running (a difference
/// invisible to the bit-exact net, which compares |M|^2 at the fixed param-card
/// alpha_s). The 2->6 states cost ~1 ms per matrix-element evaluation over a
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
        // ── QCD, informational (dynamical-scale alpha_s mismatch) ───────────
        "gg_to_ttx" => Plan::Info {
            neval: 60_000,
            niter: 8,
            reason:
                "QCD: MG runs alpha_s to a dynamical scale; vibegraph uses fixed param-card alpha_s",
        },
        "gg_to_gg" => Plan::Info {
            neval: 40_000,
            niter: 6,
            reason: "QCD: dynamical-scale alpha_s running, plus a t-channel-peaked integrand",
        },
        "uux_to_uux" => Plan::Info {
            neval: 40_000,
            niter: 6,
            reason: "QCD: dynamical-scale alpha_s running, plus a t-channel-peaked integrand",
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
        // Stable but carrying an unexplained offset, so informational rather than
        // gated. The estimator itself is sound — five seeds agree within 0.6% of
        // each other with chi2/dof in 0.36-2.02 — but every seed sits ~3.0% *above*
        // the banked sigma (pull +7.9 to +9.5), an offset that no longer hides
        // inside the error bar now that the sampler converges.
        //
        // The offset is entirely localised at low lepton-pair mass: re-integrating
        // with `mmll = 20 GeV`, which truncates that region, agrees with MadGraph to
        // -0.1% (`probe_photon_pole_is_the_instability`). Its *sign* is what keeps
        // this open rather than filed as a known deficiency of this side — failing
        // to cover the photon pole would read low, not high. The leading hypothesis
        // is the reverse: MadEvent pre-shapes a massless s-channel invariant's grid
        // only down to `xo = 10/stot` (`set_peaks` in `myamp.f`, `setgrid` in
        // `dsample.f`), so it may under-sample below `m_ll ~ 3 GeV` where this
        // sampler now maps explicitly. That is a hypothesis, not a finding: the
        // floor sets MadGraph's initial grid density, not its support, so its own
        // adaptation can still reach the region. Gate this row once the low-m_ll
        // region is reconciled against MadGraph directly — a differential
        // `dsigma/dm_ll` comparison, not a scalar.
        "ee_to_mumu_tata_qcd0" => Plan::Info {
            neval: 100_000,
            niter: 8,
            reason: "stable across seeds (spread 0.6%) but +3.0% vs banked, localised at \
                     low m_ll (agrees to -0.1% with mmll = 20 GeV); the sign rules out \
                     under-coverage on this side",
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
    // Promote flat RAMBO to the resonance-aware per-diagram multichannel — the same
    // production sampler `vibegraph integrate` drives — so the narrow electroweak
    // peaks that flat RAMBO under-samples converge. α is adapted to this process's
    // own |M|² on a survey substream disjoint from the integration seed.
    let report = integ.use_multichannel(&diagrams, &evaluated, n_survey, n_adapt_iter, seed);
    let alphas = report
        .map(|r| r.trajectory.last().cloned().unwrap_or_default())
        .unwrap_or_default();
    let result = match vegas_alpha {
        None => integ.adapt_grid(neval, niter, seed).1,
        // Same run, but with the grid-damping exponent under the probe's control:
        // `alpha = 0` freezes the grid, reducing VEGAS to an iteration-averager
        // over the multichannel sampler alone.
        Some(a) => {
            let mut grid = VegasGrid::new(integ.vegas_ndim(), VEGAS_NBINS, a);
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            grid.adapt(|u| integ.value(u), neval, niter, &mut rng)
        }
    };
    (
        result.integral * GEV2_TO_PB,
        result.std_dev * GEV2_TO_PB,
        result.chi2_per_dof,
        alphas,
    )
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

/// `(pull, relative_deviation)` of a vibegraph integral against a banked value.
fn compare(sigma_vg: f64, err_vg: f64, banked: &BankedSigma) -> (f64, f64) {
    let denom = (err_vg * err_vg + banked.sigma_err_pb * banked.sigma_err_pb).sqrt();
    let pull = (sigma_vg - banked.sigma_pb) / denom;
    let rel = (sigma_vg / banked.sigma_pb - 1.0).abs();
    (pull, rel)
}

/// Drive the gate for one banked directory. `Ok(())` on pass, skip, or info;
/// `Err(reason)` only on a failed assertion.
fn gate_dir(dir: &str, banked: &BankedSigma) -> Result<(), String> {
    match plan_for(dir) {
        Plan::Skip(reason) => {
            eprintln!("[{dir}] SKIP ({reason})");
            Ok(())
        }
        Plan::Info {
            neval,
            niter,
            reason,
        } => {
            let (sigma_vg, err_vg, chi2) = integrate(dir, &banked.process, neval, niter, SEED);
            let (pull, rel) = compare(sigma_vg, err_vg, banked);
            eprintln!(
                "[{dir}] INFO vg = {sigma_vg:.6e} +- {err_vg:.3e} pb | MG = {:.6e} +- {:.3e} pb | \
                 pull = {pull:+.2} | rel = {rel:.2e} | chi2/dof = {chi2:.2} ({neval}x{niter})  <{reason}>",
                banked.sigma_pb, banked.sigma_err_pb
            );
            Ok(())
        }
        Plan::Gate {
            neval,
            niter,
            rel_tol,
        } => {
            let (sigma_vg, err_vg, chi2) = integrate(dir, &banked.process, neval, niter, SEED);
            let (pull, rel) = compare(sigma_vg, err_vg, banked);
            eprintln!(
                "[{dir}] GATE vg = {sigma_vg:.6e} +- {err_vg:.3e} pb | MG = {:.6e} +- {:.3e} pb | \
                 pull = {pull:+.2} | rel = {rel:.2e} | chi2/dof = {chi2:.2} ({neval}x{niter})",
                banked.sigma_pb, banked.sigma_err_pb
            );
            if pull.abs() > PULL_LIMIT {
                return Err(format!(
                    "[{dir}] |pull| = {:.2} exceeds {PULL_LIMIT} \
                     (vg {sigma_vg:.6e} +- {err_vg:.3e} vs MG {:.6e} +- {:.3e})",
                    pull.abs(),
                    banked.sigma_pb,
                    banked.sigma_err_pb
                ));
            }
            if rel > rel_tol {
                return Err(format!(
                    "[{dir}] relative deviation {rel:.2e} exceeds tol {rel_tol:.2e} \
                     (vg {sigma_vg:.6e} vs MG {:.6e})",
                    banked.sigma_pb
                ));
            }
            Ok(())
        }
    }
}

#[test]
fn sigma_gate_matches_madgraph() {
    let ref_path = reference_path();
    let text = std::fs::read_to_string(&ref_path)
        .unwrap_or_else(|e| panic!("missing sigma reference {}: {e}", ref_path.display()));
    let banked: BTreeMap<String, BankedSigma> =
        serde_json::from_str(&text).expect("sigma_reference.json parses");

    if !output_dir().exists() {
        eprintln!(
            "MadGraph output/ tree absent — sigma gate skipped (regenerate with \
             `pixi run extract-sigma` or copy the reference output tree)"
        );
        return;
    }

    let mut failures = Vec::new();
    let mut asserted = 0usize;
    for (dir, entry) in &banked {
        // A banked process whose run card is missing from output/ is skipped.
        if !output_dir().join(dir).join("Cards/run_card.dat").exists() {
            eprintln!("[{dir}] SKIP (run card absent from output/)");
            continue;
        }
        if matches!(plan_for(dir), Plan::Gate { .. }) {
            asserted += 1;
        }
        if let Err(e) = gate_dir(dir, entry) {
            failures.push(e);
        }
    }

    eprintln!("sigma gate: {asserted} process(es) asserted against banked MadGraph sigma");
    assert!(
        failures.is_empty(),
        "sigma gate failures:\n{}",
        failures.join("\n")
    );
}
