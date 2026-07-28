//! Unweighting gate: does an accept/reject sample over the frozen per-channel
//! VEGAS grids reproduce the integration it came from?
//!
//! The integration path is the production one (`FixedBeamIntegrand` with the
//! per-event renormalisation scale and the α-adapted per-diagram multichannel
//! sampler, integrated channel by channel), driven with MadGraph's own run card
//! per process — the same setup `validate_sigma` uses. What this gate adds is the
//! phase *after* the integral: each channel's maximum weight is estimated by a
//! frozen scan on its own grid, events are drawn by accept/reject against it, and
//! the resulting unweighted sample is compared against the weighted estimator over
//! the same grids.
//!
//! # Why the comparison is internal
//!
//! The banked MadGraph σ is not the reference here: `validate_sigma` already gates
//! the integral against it, and an unweighting bug does not move the integral —
//! that is precisely what makes it dangerous. What an unweighting bug moves is
//! *which points are kept*. So this gate compares the unweighted sample against an
//! independent weighted estimator built from the same grids, at two levels:
//!
//! * **σ.** The cross section recovered from the kept events must match the
//!   weighted estimator and the VEGAS integral.
//! * **Distributions.** An invariant mass and an angle, binned. σ is a single
//!   scalar and is blind to a mis-sampled region of small measure; a wrong
//!   channel-selection rule, in particular, leaves σ correct and the shape wrong.
//!
//! The weighted reference deliberately uses a *different* channel-selection rule
//! from the generator (`∝ αⱼ` against the generator's `∝ w_maxⱼ`) and compensates
//! for it with the `1/qⱼ` weight, so it is unbiased whatever rule the generator
//! uses. That is what makes the shape comparison sensitive to the rule instead of
//! sharing its mistake.
//!
//! # The error classes this gate provably cannot detect
//!
//! It compares the event sample against the integrand it was drawn from, so it is
//! blind to everything wrong with that integrand — a wrong matrix element, flux,
//! or cut is reproduced faithfully on both sides. `validate_helas_mg` and
//! `validate_sigma` are what cover those. It is also blind to the *labels* an
//! event carries: helicity and colour-flow selection move no weight, so a
//! mislabelled event is invisible here by construction. Those are pinned by the
//! frequency and neutrality tests in the library, and by `color_flow_tags_oracle`.
//!
//! # Seed stability is part of the evidence
//!
//! A single generation seed proves little: an accept/reject pass that
//! systematically misses a region reports a σ that is wrong and stable. Every row
//! is generated on several independent seeds and the spread across them, not one
//! number, is what the σ comparison is made against.
//!
//! Runs only when the gitignored MadGraph `output/` tree is present (same contract
//! as `validate_sigma`); otherwise every process is skipped.

use std::path::{Path, PathBuf};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use vibegraph::cuts::Cuts;
use vibegraph::hadronic::{
    compile_subprocesses, initial_spin_color_average, process_external_legs, FixedBeamIntegrand,
};
use vibegraph::helas::eval::BoundAmplitude;
use vibegraph::helas::repr::lorentz::LorentzVector;
use vibegraph::phasespace::GEV2_TO_PB;
use vibegraph::runcard::{BeamMode, RunCard};
use vibegraph::select::select_index;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::EvaluatedModel;
use vibegraph::unweight::Unweighter;

mod common;

type V = LorentzVector<f64>;

/// α-adaptation budget for the multichannel combiner, matching the σ gate's.
const MULTICHANNEL_SURVEY: usize = 30_000;
const MULTICHANNEL_ITERS: usize = 6;

/// Integration seed, shared by every row so the grids under test are the ones the
/// σ gate would have produced.
const SEED: u64 = 20_260_719;
/// Seed for the per-channel weight scan, distinct from both the integration and
/// the generation.
const SCAN_SEED: u64 = 0x5CA7_0FF0;
/// Independent generation seeds. The spread across them is what the σ comparison
/// is made against.
const GEN_SEEDS: [u64; 4] = [0xE7E7_0001, 0xE7E7_0002, 0xE7E7_0003, 0xE7E7_0004];
/// Trials per generation seed. Fixing the trial count rather than the event count
/// bounds the runtime of a low-efficiency row.
const TRIALS_PER_SEED: usize = 150_000;
/// Trials in the weighted reference pass.
const REF_TRIALS: usize = 200_000;
/// Fewest events a seed must produce for its σ to be worth comparing.
const MIN_EVENTS: usize = 400;

/// Histogram bins per observable, and the fewest events in a bin for it to enter
/// the shape comparison.
const NBINS: usize = 16;
const MIN_BIN_EVENTS: f64 = 25.0;

/// Limits on the σ comparison: the seed-mean of the unweighted σ against the
/// VEGAS integral, in units of the combined error and as a relative deviation.
const SIGMA_PULL_LIMIT: f64 = 3.5;
const SIGMA_REL_LIMIT: f64 = 0.03;
/// Limits on the shape comparison, per observable.
const SHAPE_CHI2_LIMIT: f64 = 3.0;
const SHAPE_PULL_LIMIT: f64 = 5.0;

/// One process to exercise, with the integration budget its grids are built on.
struct Row {
    dir: &'static str,
    process: &'static str,
    neval: usize,
    niter: usize,
}

/// Processes chosen to span the shapes unweighting behaves differently on: a
/// t-channel-dominated QCD pair (one channel carries almost the whole summed
/// maximum), a gluon-initiated one with a massive final state, and two resonant
/// electroweak states whose channels are Breit-Wigner peaks.
const ROWS: &[Row] = &[
    Row {
        dir: "ee_to_mumu",
        process: "e+ e- > mu+ mu-",
        neval: 20_000,
        niter: 4,
    },
    Row {
        dir: "uux_to_uux",
        process: "u u~ > u u~",
        neval: 30_000,
        niter: 5,
    },
    Row {
        dir: "gg_to_ttx",
        process: "g g > t t~",
        neval: 30_000,
        niter: 5,
    },
    Row {
        dir: "ee_to_tatah",
        process: "e+ e- > ta+ ta- h",
        neval: 30_000,
        niter: 5,
    },
    Row {
        dir: "ee_to_mumua",
        process: "e+ e- > mu+ mu- a",
        neval: 40_000,
        niter: 5,
    },
];

fn output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/output")
}

fn param_card(dir: &str) -> ParamCard {
    let path = output_dir().join(dir).join("Cards/param_card.dat");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.parse::<ParamCard>().ok())
        .unwrap_or_else(|| "".parse::<ParamCard>().unwrap())
}

/// Build the production fixed-energy integrand for a banked process — its run
/// card, param card, cuts, per-event renormalisation scale and α-adapted
/// per-diagram multichannel sampler — and hand it to `f`.
fn with_integrand<R>(row: &Row, f: impl FnOnce(&FixedBeamIntegrand) -> R) -> R {
    let card_path = output_dir().join(row.dir).join("Cards/run_card.dat");
    let run_card = RunCard::parse_file(&card_path).expect("real run card parses");
    assert_eq!(run_card.beam_mode(), BeamMode::FixedEnergy);
    let sqrt_s = run_card.ebeam1 + run_card.ebeam2;

    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &param_card(row.dir));

    let sets = common::generate(row.process);
    let evals = compile_subprocesses(&sets, &model, &evaluated).expect("compile subprocesses");
    let bounds: Vec<_> = evals
        .iter()
        .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
        .collect();

    let rep = &evals[0];
    let legs = process_external_legs(rep, &model, &evaluated);
    let cuts = Cuts::compile(&run_card, &legs).expect("run card cuts compile");
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
    integ
        .use_running_coupling(&diagrams, &model, &evaluated, &run_card)
        .expect("run card scale prescription compiles");
    integ.use_multichannel(
        &diagrams,
        &evaluated,
        MULTICHANNEL_SURVEY,
        MULTICHANNEL_ITERS,
        SEED,
    );
    f(&integ)
}

/// One binned observable: what it is called and the range it is histogrammed over.
struct ObsSpec {
    name: &'static str,
    lo: f64,
    hi: f64,
}

/// The observables compared bin by bin, chosen to cover the coordinates the
/// channel maps reshape.
///
/// The first leg's polar and azimuthal angles are always compared; at a fixed
/// partonic energy they are the *complete* set of degrees of freedom of a `2 → 2`
/// final state, so for those processes the shape comparison is exhaustive rather
/// than indicative. Above `2 → 2` the leading pair's invariant mass is added: that
/// is the coordinate a Breit-Wigner channel maps, and so the one a mis-sampled
/// resonance shows up in.
fn obs_specs(n_out: usize, sqrt_s: f64) -> Vec<ObsSpec> {
    let mut specs = vec![
        ObsSpec {
            name: "cos(theta_1)",
            lo: -1.0,
            hi: 1.0,
        },
        ObsSpec {
            name: "phi_1/pi",
            lo: -1.0,
            hi: 1.0,
        },
    ];
    if n_out >= 3 {
        specs.push(ObsSpec {
            name: "m(1,2)",
            lo: 0.0,
            hi: sqrt_s,
        });
    }
    specs
}

fn observables(momenta: &[V]) -> Vec<f64> {
    let p = momenta[0];
    let mut v = vec![
        p.pz() / p.p3().max(f64::MIN_POSITIVE),
        p.py().atan2(p.px()) / std::f64::consts::PI,
    ];
    if momenta.len() >= 3 {
        v.push((momenta[0] + momenta[1]).m());
    }
    v
}

/// A fixed-range weighted histogram; out-of-range entries land in the end bins so
/// nothing silently leaves the comparison.
#[derive(Clone)]
struct Hist {
    lo: f64,
    hi: f64,
    sum: Vec<f64>,
    sum_sq: Vec<f64>,
}

impl Hist {
    fn new(lo: f64, hi: f64) -> Self {
        Hist {
            lo,
            hi,
            sum: vec![0.0; NBINS],
            sum_sq: vec![0.0; NBINS],
        }
    }

    fn fill(&mut self, x: f64, w: f64) {
        let t = (x - self.lo) / (self.hi - self.lo);
        let k = ((t * NBINS as f64).floor() as isize).clamp(0, NBINS as isize - 1) as usize;
        self.sum[k] += w;
        self.sum_sq[k] += w * w;
    }

    fn total(&self) -> f64 {
        self.sum.iter().sum()
    }
}

/// Compare two histograms as normalised shapes, returning `(chi2_per_dof,
/// max_abs_pull, bins_compared)`.
///
/// Each bin's fraction carries the error its own weight sum implies
/// (`sqrt(Σw²)/Σw` scaled to the fraction); bins the unweighted sample populated
/// too thinly to say anything about are dropped rather than compared at zero.
fn compare_shapes(a: &Hist, b: &Hist) -> (f64, f64, usize) {
    let (ta, tb) = (a.total(), b.total());
    let mut chi2 = 0.0;
    let mut worst = 0.0f64;
    let mut ndf = 0usize;
    for k in 0..NBINS {
        // The unweighted histogram's bin population, in effective events.
        let eff_a = if a.sum_sq[k] > 0.0 {
            a.sum[k] * a.sum[k] / a.sum_sq[k]
        } else {
            0.0
        };
        if eff_a < MIN_BIN_EVENTS || b.sum[k] <= 0.0 {
            continue;
        }
        let fa = a.sum[k] / ta;
        let fb = b.sum[k] / tb;
        let ea = a.sum_sq[k].sqrt() / ta;
        let eb = b.sum_sq[k].sqrt() / tb;
        let err = (ea * ea + eb * eb).sqrt();
        if !(err > 0.0) {
            continue;
        }
        let pull = (fa - fb) / err;
        chi2 += pull * pull;
        worst = worst.max(pull.abs());
        ndf += 1;
    }
    (chi2 / ndf.max(1) as f64, worst, ndf)
}

/// The weighted reference pass: draw the channel `∝ αⱼ` and weight by `1/qⱼ`, so
/// the estimator is unbiased independently of the rule the generator uses to pick
/// its channel. Returns `(sigma_pb, sigma_err_pb, histograms)`.
fn weighted_reference(
    integ: &FixedBeamIntegrand,
    grids: &[&vibegraph::vegas::VegasGrid],
    hists: Vec<Hist>,
    seed: u64,
) -> (f64, f64, Vec<Hist>) {
    let alphas = integ.channel_alphas();
    let total_alpha: f64 = alphas.iter().sum();
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut u = vec![0.0; integ.channel_grid_ndim()];
    let mut momenta = Vec::new();
    let mut hists = hists;
    let (mut sum, mut sum_sq) = (0.0f64, 0.0f64);
    for _ in 0..REF_TRIALS {
        let j = select_index(&alphas, rng.random::<f64>()).expect("some channel carries weight");
        let q = alphas[j] / total_alpha;
        let jac = grids[j].draw(&mut rng, &mut u);
        let w = jac * integ.event_in_channel(j, &u, &mut momenta) * GEV2_TO_PB / q;
        sum += w;
        sum_sq += w * w;
        if w > 0.0 {
            let x = observables(&momenta);
            for (h, v) in hists.iter_mut().zip(x) {
                h.fill(v, w);
            }
        }
    }
    let n = REF_TRIALS as f64;
    let mean = sum / n;
    let err = ((sum_sq / n - mean * mean) / n).max(0.0).sqrt();
    (mean, err, hists)
}

#[test]
fn unweighted_sample_reproduces_the_integration_it_came_from() {
    if !output_dir().exists() {
        eprintln!(
            "skipping: {} absent (run `pixi run -e madgraph build-diagrams`)",
            output_dir().display()
        );
        return;
    }

    let mut failures: Vec<String> = Vec::new();
    for row in ROWS {
        if !output_dir().join(row.dir).exists() {
            eprintln!("-- {} -- skipped: no banked MadGraph output", row.dir);
            continue;
        }
        with_integrand(row, |integ| {
            let (channels, result) = integ.adapt_grids(row.neval, row.niter, SEED);
            let sigma_vegas = result.integral * GEV2_TO_PB;
            let err_vegas = result.std_dev * GEV2_TO_PB;
            let grids: Vec<_> = channels.iter().map(|c| &c.grid).collect();

            // Each channel's maximum, scanned on its own grid with its own share of
            // the integration budget.
            let mut uw = Unweighter::scan(
                integ,
                channels.iter().map(|c| (&c.grid, c.neval)),
                SCAN_SEED,
            );
            let w_max_sum = uw.total_w_max() * GEV2_TO_PB;
            let predicted_eff = sigma_vegas / w_max_sum;
            let dir = row.dir;
            let empty = uw.empty_channels();
            eprintln!(
                "-- {dir} ({} channels) --\n  \
                 sigma(VEGAS) = {sigma_vegas:.6e} +- {err_vegas:.2e} pb, \
                 sum w_max = {w_max_sum:.4e} pb\n  \
                 predicted efficiency {predicted_eff:.3e}, \
                 largest channel {:.0}% of sum w_max, empty channels {empty:?}",
                channels.len(),
                100.0 * uw.largest_channel_share(),
            );

            let specs = obs_specs(integ.final_masses().len(), integ.beams()[0].e() * 2.0);
            let blank = || specs.iter().map(|s| Hist::new(s.lo, s.hi)).collect();

            // The weighted reference, on its own seed and its own channel rule.
            let (sigma_ref, err_ref, ref_hists) =
                weighted_reference(integ, &grids, blank(), 0xBEEF_0001);

            // Generation: one pass per seed, sharing the scanned maxima.
            let mut sigmas = Vec::new();
            let mut gen_hists: Vec<Hist> = blank();
            let mut momenta = Vec::new();
            for &seed in &GEN_SEEDS {
                let before = uw.stats().clone();
                let mut rng = ChaCha8Rng::seed_from_u64(seed);
                for _ in 0..TRIALS_PER_SEED {
                    if let Some(point) = uw.trial(integ, &mut rng) {
                        integ.event_in_channel(point.channel, &point.u, &mut momenta);
                        let x = observables(&momenta);
                        for (h, v) in gen_hists.iter_mut().zip(x) {
                            h.fill(v, point.weight);
                        }
                    }
                }
                let after = uw.stats();
                let events = after.accepted - before.accepted;
                let weight = after.event_weight_sum - before.event_weight_sum;
                let sigma = uw.total_w_max() * GEV2_TO_PB * weight / TRIALS_PER_SEED as f64;
                eprintln!(
                    "  seed {seed:#010x} | {events:>6} events, \
                     eff {:.3e} | sigma(events) = {sigma:.6e} pb",
                    events as f64 / TRIALS_PER_SEED as f64
                );
                if (events as usize) < MIN_EVENTS {
                    failures.push(format!(
                        "[{}] seed {seed:#010x} produced only {events} events",
                        row.dir
                    ));
                }
                sigmas.push(sigma);
            }

            let s = uw.stats();
            eprintln!(
                "  overall: eff {:.3e} (predicted {predicted_eff:.3e}) | \
                 overweight fraction {:.3e}, weight share {:.3e}, \
                 excess share {:.3e}, max w/w_max {:.3}\n  \
                 sigma(weighted ref) = {sigma_ref:.6e} +- {err_ref:.2e} pb",
                s.efficiency(),
                s.overweight_fraction(),
                s.overweight_weight_share(),
                s.excess_share(),
                s.ratio_max,
            );

            // σ from the events, as a seed mean with the spread as its error.
            let n = sigmas.len() as f64;
            let mean = sigmas.iter().sum::<f64>() / n;
            let var = sigmas.iter().map(|s| (s - mean) * (s - mean)).sum::<f64>() / (n - 1.0);
            let sem = (var / n).sqrt();
            let spread = sigmas
                .iter()
                .map(|s| (s / mean - 1.0).abs())
                .fold(0.0f64, f64::max);
            for (label, target, err) in [
                ("VEGAS", sigma_vegas, err_vegas),
                ("weighted ref", sigma_ref, err_ref),
            ] {
                let pull = (mean - target) / (sem * sem + err * err).sqrt();
                let rel = mean / target - 1.0;
                eprintln!(
                    "  sigma(events) = {mean:.6e} +- {sem:.2e} pb (seed spread {:.2}%) \
                     vs {label} -> pull {pull:+.2}, rel {:+.3}%",
                    100.0 * spread,
                    100.0 * rel
                );
                if pull.abs() > SIGMA_PULL_LIMIT || rel.abs() > SIGMA_REL_LIMIT {
                    failures.push(format!(
                        "[{}] sigma from events {mean:.6e} vs {label} {target:.6e}: \
                         pull {pull:+.2}, rel {:+.3}%",
                        row.dir,
                        100.0 * rel
                    ));
                }
            }

            for (k, spec) in specs.iter().enumerate() {
                let name = spec.name;
                let (chi2, worst, ndf) = compare_shapes(&gen_hists[k], &ref_hists[k]);
                eprintln!(
                    "  shape {name:<13} chi2/dof {chi2:.2} over {ndf} bins, \
                     worst pull {worst:.2}"
                );
                if ndf < 3 {
                    failures.push(format!(
                        "[{}] {name}: only {ndf} bins were populated enough to compare",
                        row.dir
                    ));
                } else if chi2 > SHAPE_CHI2_LIMIT || worst > SHAPE_PULL_LIMIT {
                    failures.push(format!(
                        "[{}] {name}: chi2/dof {chi2:.2} over {ndf} bins, worst pull {worst:.2}",
                        row.dir
                    ));
                }
            }
        });
    }

    assert!(
        failures.is_empty(),
        "unweighting gate failures:\n{failures:#?}"
    );
}
