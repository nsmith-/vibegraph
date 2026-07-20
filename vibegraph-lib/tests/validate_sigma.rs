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
//! to *mis-sampled regions of small measure*. Flat RAMBO carries no importance
//! map, so a narrow feature — a Breit-Wigner resonance, or a soft/collinear
//! enhancement left after cuts — that contributes little to the total can be
//! systematically under-sampled without moving sigma beyond its Monte-Carlo
//! error. It is also blind to differences in *couplings evaluated outside the
//! matrix element*: MadGraph runs alpha_s to a per-event dynamical scale
//! (`fixed_ren_scale = False`), whereas this integration uses the fixed
//! param-card alpha_s, so the sigma of an alpha_s-dependent (QCD) process differs
//! by the running even when its |M|^2 is bit-exact. The bit-exact net remains the
//! fine instrument; agreement here confirms the *normalisation and averaging* of
//! the smooth electroweak cross sections.
//!
//! # Which processes are asserted
//!
//! Only the smooth, non-resonant electroweak final states are gated with an
//! assertion — the ones flat RAMBO at fixed alpha_s can reproduce to a few
//! percent. The QCD processes are integrated *informationally* (printed, not
//! asserted): their sigma legitimately differs from the banked value by the
//! dynamical-scale alpha_s running described above. The sharply resonant
//! electroweak multi-body final states and the 2->6 states are not integrated:
//! flat RAMBO cannot sample their resonant peaks (the small-measure blind spot),
//! and the 2->6 matrix-element cost makes a meaningful integral prohibitively
//! slow. Every process is still driven through the same run-card-pinned setup, so
//! the cut compiler and beam handling are exercised for all of them.
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

use vibegraph::cuts::Cuts;
use vibegraph::hadronic::{
    compile_subprocesses, initial_spin_color_average, process_external_legs, FixedBeamIntegrand,
};
use vibegraph::helas::eval::BoundAmplitude;
use vibegraph::phasespace::GEV2_TO_PB;
use vibegraph::runcard::{BeamMode, RunCard};
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::EvaluatedModel;

mod common;

/// A pull magnitude above this fails the gate. The banked runs and the vibegraph
/// integral are independent Monte-Carlo estimates, so a few-sigma spread is
/// expected; 3.5 leaves headroom over the nominal 3-sigma target without
/// admitting a genuine normalisation error (which shows up as a many-sigma pull
/// once the budget makes `err_vg` small).
const PULL_LIMIT: f64 = 3.5;

/// Fixed RNG seed — makes the integral (and hence the pull) reproducible.
const SEED: u64 = 20_260_719;

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
/// Gated (asserted) processes are the smooth electroweak final states; their
/// budgets are sized to bring `err_vg` near the banked MG error while keeping the
/// default test suite fast. QCD processes are informational — MadGraph runs
/// alpha_s to a per-event dynamical scale, so their banked sigma differs from a
/// fixed-alpha_s integral by the running (a difference invisible to the bit-exact
/// net, which compares |M|^2 at the fixed param-card alpha_s). The resonant
/// electroweak multi-body states carry sharp Breit-Wigner peaks (Z/gamma* in the
/// lepton-pair mass) that flat RAMBO under-samples, and the 2->6 states cost
/// ~1 ms per matrix-element evaluation over a 24-dim map — neither is integrated.
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
        // ── sharply resonant electroweak, not integrated ────────────────────
        "ee_to_mumua" => Plan::Skip(
            "resonant: Z/gamma* peak plus soft/collinear photon — flat RAMBO under-samples",
        ),
        "ee_to_tatah" => Plan::Skip(
            "resonant: on-shell Z (Z->ta ta) peak in the 3-body map — flat RAMBO under-samples",
        ),
        "ee_to_mumu_tata_qcd0" => Plan::Skip(
            "resonant: double Z/gamma* peaks in the 4-body map — flat RAMBO under-samples",
        ),
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
fn integrate(dir: &str, process: &str, neval: usize, niter: usize) -> (f64, f64, f64) {
    let run_card = RunCard::parse_file(&output_dir().join(dir).join("Cards/run_card.dat"))
        .expect("real run card parses");
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

    let amps: Vec<&BoundAmplitude<f64>> = bounds.iter().collect();
    let integ = FixedBeamIntegrand::new(amps, &cuts, sqrt_s, final_masses, spin_color_avg);
    let result = integ.adapt_grid(neval, niter, SEED).1;
    (
        result.integral * GEV2_TO_PB,
        result.std_dev * GEV2_TO_PB,
        result.chi2_per_dof,
    )
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
            let (sigma_vg, err_vg, chi2) = integrate(dir, &banked.process, neval, niter);
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
            let (sigma_vg, err_vg, chi2) = integrate(dir, &banked.process, neval, niter);
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
