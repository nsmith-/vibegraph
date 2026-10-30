//! The peripheral spine without a fiducial scale, at the level of σ.
//!
//! `u c > e+ e- u c QCD=0` at fixed `√ŝ = 1 TeV` under invariant-mass cuts only
//! (`ptj = ptl = 0`), so [`Cuts::spacelike_floor`] is zero and every one of its
//! ten channels is peripheral. The production integrator — `FixedBeamIntegrand`,
//! its α-adapted multichannel map and per-channel VEGAS, as `vibegraph integrate`
//! runs a fixed budget — is run on independent seeds and read under the seed
//! policy: the seeds must agree with their own quoted errors (`χ²/dof` inside a
//! band) and scatter by less than a relative bound.
//!
//! The structural tests in `diagram_channel.rs` pin that the spine is built.
//! This one pins what the spine is for. Without it the channels fall back to
//! all-timelike trees, which sample the `1/t` poles of the two quark lines
//! nowhere; VEGAS then misses regions of small measure, and the symptom is the
//! heavy-tailed one: seeds that disagree far beyond their quoted errors.
//!
//! What it cannot see: a bias every seed shares. The seeds are compared with
//! each other, not with an independent σ — flat RAMBO does not converge on this
//! process, under VEGAS or without it — so a map that is consistently wrong
//! everywhere passes. The walk/density identity in `diagram_channel.rs` is the
//! check on that.
//!
//! Banked for cost (a few minutes of integration), not for dependencies: it
//! reads only the interned Standard Model.
//!
//!     cargo test -p vibegraph-lib --profile release-debug \
//!         --features extended-validation --test spine_sigma -- --nocapture

mod common;

use vibegraph::cuts::Cuts;
use vibegraph::diagrams::diagram::Diagram;
use vibegraph::hadronic::{
    channel_diagrams, compile_subprocesses, initial_spin_color_average, process_external_legs,
    FixedBeamIntegrand, FixedBeams,
};
use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
use vibegraph::phasespace::diagram_channel::DiagramChannel;
use vibegraph::phasespace::GEV2_TO_PB;
use vibegraph::runcard::RunCard;
use vibegraph::ufo::EvaluatedModel;

use common::seeds::{combine, SeedMean};

/// Two quark lines exchanging electroweak bosons, the lepton pair radiated off
/// them or produced between them: every diagram is peripheral.
const PROCESS: &str = "u c > e+ e- u c QCD=0";

/// No single-leg `pT` cut, so no fiducial scale; the invariant-mass cuts of the
/// CMS VBF/EWK Drell–Yan cards. The default `|η_j| < 5` and `ΔR > 0.4` stay on
/// and keep the massless quarks off their beams.
const CARD: &str =
    "0 = lpp1\n0 = lpp2\n500 = ebeam1\n500 = ebeam2\n0 = ptj\n0 = ptl\n300 = mmjj\n50 = mmll\n";

/// VEGAS points per iteration and iterations, a seed.
const NEVAL: usize = 200_000;
const NITER: usize = 8;

/// The α-survey's points per iteration and iterations.
const SURVEY: (usize, usize) = (30_000, 6);

/// The gated seeds.
const SEEDS: [u64; 16] = [
    11, 22, 33, 44, 55, 66, 77, 88, 99, 110, 121, 132, 143, 154, 165, 176,
];

/// The band the seeds' `χ²/dof` about their unweighted mean must fall in.
///
/// Over 40 calibration seeds (1001–1040) a seed scatters by 0.91 of its quoted
/// error; the 16 gated seeds read 1.06, and an earlier gate of eight of them
/// 1.55. `χ²₁₅/15` lies in 0.23–2.5 at the 0.1% and 99.9% quantiles, so the
/// band leaves room for quoted errors up to about 1.25× too small. Its lower
/// edge catches seeds that are not independent, its upper edge a region the
/// seeds find only sometimes. With the spine withheld the gated seeds read
/// 5.7, but sixteen drawn from 40 such seeds read a median of 2.4: the
/// no-spine estimator quotes errors nearly as wide as its scatter, so this
/// bound alone does not separate the two.
const CHI2_PER_DOF_BAND: (f64, f64) = (0.15, 4.0);

/// The bound on the seeds' spread (their standard deviation over `√n`) over
/// their unweighted mean.
///
/// A seed scatters by 3.3e-3 relative over the 40 calibration seeds and by
/// 5.4e-3 over the first eight gated ones, so sixteen are expected at
/// 0.8–1.4e-3; the gated sixteen read 1.05e-3. With the spine withheld a seed
/// scatters by 0.13 relative, sixteen drawn from 40 such seeds read 7.5e-3 at
/// the 0.1% quantile, and the gated sixteen read 3.7e-2.
const REL_SPREAD_MAX: f64 = 3.0e-3;

/// The process compiled once, integrated per seed.
struct Setup {
    evaluated: EvaluatedModel,
    diagrams: Vec<Diagram>,
    evals: Vec<AmplitudeEvaluator>,
    cuts: Cuts,
    beams: FixedBeams,
    final_masses: Vec<f64>,
    avg: f64,
}

impl Setup {
    fn new() -> Self {
        let model = common::sm_model();
        let evaluated = EvaluatedModel::from_model(model.clone());
        let sets = common::generate(PROCESS);
        let evals = compile_subprocesses(&sets, &model, &evaluated).expect("compile");
        let rep = &evals[0];
        let legs = process_external_legs(rep, &model, &evaluated);
        let card = RunCard::parse(CARD).expect("card parses");
        let cuts = Cuts::compile(&card, &legs).expect("cuts compile");
        let beams = FixedBeams::from_run_card(&card, &legs);
        let final_masses: Vec<f64> = rep.external_particles()[rep.n_in()..]
            .iter()
            .map(|&id| evaluated.mass(id))
            .collect();
        let avg = initial_spin_color_average(rep, &model, &evaluated);
        let diagrams: Vec<Diagram> = sets
            .iter()
            .flat_map(|s| s.diagrams.iter().cloned())
            .collect();
        Setup {
            evaluated,
            diagrams,
            evals,
            cuts,
            beams,
            final_masses,
            avg,
        }
    }

    /// The channels the multichannel map builds, and how many of them are spines.
    fn census(&self) -> (usize, usize) {
        let reps = channel_diagrams(&self.diagrams, &self.evaluated);
        let spines = reps
            .iter()
            .filter(|d| {
                !DiagramChannel::<f64>::from_diagram_regulated(
                    d,
                    &self.evaluated,
                    self.beams.sqrt_s(),
                    self.cuts.spacelike_floor(),
                )
                .spine_poles()
                .is_empty()
            })
            .count();
        (reps.len(), spines)
    }

    /// One seed's σ and its quoted error, in pb.
    fn integrate(&self, seed: u64) -> (f64, f64) {
        let bounds: Vec<BoundAmplitude<f64>> = self
            .evals
            .iter()
            .map(|e| BoundAmplitude::bind(e, &self.evaluated))
            .collect();
        let amps: Vec<&BoundAmplitude<f64>> = bounds.iter().collect();
        let mut integ = FixedBeamIntegrand::new(
            amps,
            &self.cuts,
            self.beams,
            self.final_masses.clone(),
            self.avg,
        );
        integ
            .use_multichannel(&self.diagrams, &self.evaluated, SURVEY.0, SURVEY.1, seed)
            .expect("the process has diagrams");
        let r = integ.adapt_grids(NEVAL, NITER, seed).1;
        (r.integral * GEV2_TO_PB, r.std_dev * GEV2_TO_PB)
    }

    fn sweep(&self, seeds: &[u64]) -> SeedMean {
        let runs: Vec<(f64, f64)> = seeds
            .iter()
            .map(|&seed| {
                let (sigma, err) = self.integrate(seed);
                eprintln!("  seed {seed}: σ = {sigma:.6e} ± {err:.2e} pb");
                (sigma, err)
            })
            .collect();
        combine(&runs)
    }
}

/// The gate: the production integrator's seeds agree with their own quoted
/// errors and with each other.
#[test]
fn the_no_scale_spine_gives_seeds_that_agree_on_a_mass_cut_peripheral_process() {
    let setup = Setup::new();
    assert_eq!(
        setup.cuts.spacelike_floor(),
        0.0,
        "the card must leave no fiducial scale"
    );
    let (channels, spines) = setup.census();
    eprintln!(
        "{PROCESS} at √ŝ = {} GeV, {NEVAL} × {NITER} a seed: {spines} peripheral channels of {channels}",
        setup.beams.sqrt_s()
    );
    let start = std::time::Instant::now();
    let sweep = setup.sweep(&SEEDS);
    let rel_spread = sweep.spread / sweep.mean;
    eprintln!(
        "  {} seeds: mean {:.6e} pb, spread {:.2e} (rel {rel_spread:.2e}), seed χ²/dof {:.2}, {:.0} s",
        sweep.n,
        sweep.mean,
        sweep.spread,
        sweep.chi2_per_dof,
        start.elapsed().as_secs_f64()
    );
    assert!(
        rel_spread <= REL_SPREAD_MAX,
        "the seeds' relative spread {rel_spread:.2e} exceeds {REL_SPREAD_MAX:.1e}"
    );
    assert!(
        (CHI2_PER_DOF_BAND.0..=CHI2_PER_DOF_BAND.1).contains(&sweep.chi2_per_dof),
        "the seeds scatter at χ²/dof {:.2} against their own errors, outside {CHI2_PER_DOF_BAND:?}",
        sweep.chi2_per_dof
    );
}

/// The calibration sweep the gate's bands are read from: every seed's σ, for a
/// seed set wider than the gate's, printed as one line each.
#[test]
#[ignore]
fn probe_the_seed_scatter_of_the_no_scale_spine() {
    let setup = Setup::new();
    let (channels, spines) = setup.census();
    eprintln!(
        "{PROCESS} at √ŝ = {} GeV, {NEVAL} × {NITER} a seed: {spines} peripheral channels of {channels}",
        setup.beams.sqrt_s()
    );
    let seeds: Vec<u64> = (1..=40).map(|k| 1000 + k).collect();
    let start = std::time::Instant::now();
    let sweep = setup.sweep(&seeds);
    eprintln!(
        "  {} seeds: mean {:.6e} pb, spread {:.2e} (rel {:.2e}), seed χ²/dof {:.2}, {:.0} s",
        sweep.n,
        sweep.mean,
        sweep.spread,
        sweep.spread / sweep.mean,
        sweep.chi2_per_dof,
        start.elapsed().as_secs_f64()
    );
}
