//! Diagram-driven phase-space channel: extraction from the `Prop` chain.
//!
//! Complements the in-crate unit tests (which drive the sampler off explicit
//! topologies) by building channels from real enumerated diagrams and checking
//! that the propagator chain yields on-shell, momentum-conserving kinematics and
//! that the s-channel resonance / t-channel metadata a resonance-aware map will
//! consume is populated from the model.

mod common;

use vibegraph::diagrams::diagram::Diagram;
use vibegraph::helas::LorentzVector;
use vibegraph::phasespace::rng::SubStream;
use vibegraph::phasespace::{Channel, DiagramChannel, MultiChannel, PhaseSpaceMap, RamboChannel};
use vibegraph::ufo::EvaluatedModel;

fn total(momenta: &[LorentzVector<f64>]) -> [f64; 4] {
    momenta.iter().fold([0.0; 4], |a, p| {
        [a[0] + p.e(), a[1] + p.px(), a[2] + p.py(), a[3] + p.pz()]
    })
}

/// Outgoing-leg masses of a diagram, read from the model — the same masses
/// `DiagramChannel::from_diagram` puts on the tree leaves.
fn out_masses(d: &Diagram, model: &EvaluatedModel) -> Vec<f64> {
    let n_out = d.n_ext() - d.n_in;
    (0..n_out)
        .map(|slot| model.mass(d.legs[d.n_in + slot].particle))
        .collect()
}

fn assert_valid(ch: &DiagramChannel<f64>, sqrt_s: f64, masses: &[f64], seed: u64) {
    let n_out = masses.len();
    assert_eq!(ch.ndim(), 3 * n_out - 4);
    let mut stream = SubStream::from_stream(seed, 3);
    for _ in 0..300 {
        let u = stream.uniforms::<f64>(ch.ndim());
        let pt = ch.sample(&u);
        assert_eq!(pt.momenta.len(), n_out);
        let tot = total(&pt.momenta);
        assert!(
            (tot[0] - sqrt_s).abs() < 1e-6 * sqrt_s,
            "energy not conserved: {} vs {sqrt_s}",
            tot[0]
        );
        for c in &tot[1..] {
            assert!(c.abs() < 1e-6 * sqrt_s, "3-momentum not conserved: {c}");
        }
        for (p, &m) in pt.momenta.iter().zip(masses) {
            assert!(
                (p.m2() - m * m).abs() < 1e-5 * sqrt_s * sqrt_s + 1e-5,
                "off shell: m² = {} want {}",
                p.m2(),
                m * m
            );
            assert!(p.e() > 0.0 && p.e().is_finite());
        }
        assert!(pt.weight > 0.0 && pt.weight.is_finite());
        assert_eq!(ch.density(&pt.momenta), 1.0 / pt.weight);
    }
}

/// Channels built from real diagrams of a spread of processes emit valid points.
#[test]
fn diagram_channels_are_on_shell_and_conserving() {
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let sqrt_s = 500.0;

    let processes = [
        "e+ e- > mu+ mu-",
        "u u~ > u u~",
        "e+ e- > mu+ mu- ta+ ta-",
        "u u~ > d d~ g",
    ];
    for process in processes {
        let sets = common::generate(process);
        assert!(!sets.is_empty(), "no diagrams for {process}");
        let diagrams: &[Diagram] = &sets[0].diagrams;
        assert!(!diagrams.is_empty(), "empty diagram set for {process}");
        for (i, d) in diagrams.iter().enumerate() {
            let masses = out_masses(d, &evaluated);
            let ch = DiagramChannel::from_diagram(d, &evaluated, sqrt_s);
            assert_valid(&ch, sqrt_s, &masses, 0xC0DE + i as u64);
        }
    }
}

/// The s-channel resonance metadata is read off the propagator chain: every mass
/// a diagram's channel records must be the model mass of one of that diagram's own
/// timelike internal lines, and across `e+ e- > mu+ mu- ta+ ta-` the Z-pole
/// subsystem (a µ⁺µ⁻ or τ⁺τ⁻ pair off a Z) must appear. A subsystem read off the
/// wrong legs, or attached to the wrong propagator particle, would break this.
#[test]
fn resonance_metadata_matches_propagator_masses() {
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let mz = evaluated.mass(model.particle_id("Z").expect("Z in model"));
    assert!(mz > 0.0, "expected a massive Z");

    let sets = common::generate("e+ e- > mu+ mu- ta+ ta-");
    let diagrams = &sets[0].diagrams;
    let mut saw_z = false;
    for d in diagrams {
        // The masses the diagram's timelike internal lines can supply.
        let prop_masses: Vec<f64> = d.props.iter().map(|p| evaluated.mass(p.particle)).collect();
        let ch = DiagramChannel::<f64>::from_diagram(d, &evaluated, 500.0);
        for r in ch.resonances() {
            assert!(
                prop_masses.iter().any(|&m| (m - r.mass).abs() < 1e-9),
                "resonance mass {} is not any propagator mass in the diagram",
                r.mass
            );
            saw_z |= (r.mass - mz).abs() < 1e-6;
        }
    }
    assert!(saw_z, "no Z-pole subsystem found among the diagrams");
}

/// The t-channel metadata is populated for a process with spacelike exchange:
/// `u u~ > u u~` has gluon t-channel diagrams, so at least one diagram records a
/// spacelike line.
#[test]
fn t_channel_metadata_populated_for_spacelike_exchange() {
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());

    let sets = common::generate("u u~ > u u~");
    let diagrams = &sets[0].diagrams;
    let any_t = diagrams.iter().any(|d| {
        !DiagramChannel::<f64>::from_diagram(d, &evaluated, 500.0)
            .t_channels()
            .is_empty()
    });
    assert!(any_t, "no t-channel line recorded for u u~ > u u~");
}

/// Invariant mass² of the outgoing pair `(i, j)`.
fn s_pair(p: &[LorentzVector<f64>], i: usize, j: usize) -> f64 {
    let (a, b) = (&p[i], &p[j]);
    let e = a.e() + b.e();
    let px = a.px() + b.px();
    let py = a.py() + b.py();
    let pz = a.pz() + b.pz();
    e * e - px * px - py * py - pz * pz
}

/// Monte-Carlo mean and per-point estimator variance of `weight·f` over a map.
fn mc_estimate(
    map: &dyn PhaseSpaceMap<f64>,
    seed: u64,
    stream: u64,
    n: usize,
    f: impl Fn(&[LorentzVector<f64>]) -> f64,
) -> (f64, f64) {
    let mut s = SubStream::from_stream(seed, stream);
    let ndim = map.ndim();
    let (mut sum, mut sum_sq) = (0.0, 0.0);
    for _ in 0..n {
        let u = s.uniforms::<f64>(ndim);
        let pt = map.sample(&u);
        let v = pt.weight * f(&pt.momenta);
        sum += v;
        sum_sq += v * v;
    }
    let mean = sum / n as f64;
    let var = (sum_sq / n as f64 - mean * mean).max(0.0);
    (mean, var)
}

/// A [`MultiChannel`] combiner assembled from every diagram of a real process
/// integrates unbiasedly — it agrees with flat RAMBO on the phase-space volume `V_n`
/// — and resolves *both* the µ⁺µ⁻ and the τ⁺τ⁻ Z-pole its diagram channels resonate
/// on at variance strictly below flat RAMBO at fixed `N`. The τ pole is the payload:
/// its s-channel line is stored on the both-beam complement side of feyngraph's
/// momentum routing, so before the routing-aware relabel no channel resonated on
/// `s(τ⁺τ⁻)` and this probe could not beat flat RAMBO. This exercises the combiner
/// over heterogeneous `from_diagram` channels, not just controlled topologies.
#[test]
fn multichannel_over_real_diagrams_unbiased_and_resonant() {
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let z = model.particle_id("Z").expect("Z in model");
    let (mz, gz) = (evaluated.mass(z), evaluated.width(z));
    assert!(mz > 0.0 && gz > 0.0, "expected a massive, finite-width Z");
    let sqrt_s = 500.0;

    let sets = common::generate("e+ e- > mu+ mu- ta+ ta-");
    let diagrams = &sets[0].diagrams;
    let masses = out_masses(&diagrams[0], &evaluated);
    let n_out = masses.len();

    let channels: Vec<Box<dyn Channel<f64>>> = diagrams
        .iter()
        .map(|d| {
            Box::new(DiagramChannel::from_diagram(d, &evaluated, sqrt_s)) as Box<dyn Channel<f64>>
        })
        .collect();
    let n_ch = channels.len();
    let multi = MultiChannel::uniform(channels);
    assert_eq!(multi.ndim(), 1 + (3 * n_out - 4));

    let flat = RamboChannel::new(sqrt_s, masses.clone());
    let n = 400_000;

    // Unbiased volume: the combiner and flat RAMBO agree on V_n.
    let (v_m, var_vm) = mc_estimate(&multi, 0xA11, 41, n, |_| 1.0);
    let (v_f, var_vf) = mc_estimate(&flat, 0xA12, 43, n, |_| 1.0);
    let (e_m, e_f) = ((var_vm / n as f64).sqrt(), (var_vf / n as f64).sqrt());
    let ev = (e_m * e_m + e_f * e_f).sqrt();
    eprintln!(
        "real V_n ({n_ch} channels): multi {v_m:.6e} ± {e_m:.2e} vs flat {v_f:.6e} ± {e_f:.2e}"
    );
    assert!(
        (v_m - v_f).abs() < 6.0 * ev,
        "combiner V_n {v_m:.6e} disagrees with flat RAMBO {v_f:.6e}"
    );

    // Resolves both the µ⁺µ⁻ (outgoing slots 0,1) and τ⁺τ⁻ (slots 2,3) Z poles
    // below flat RAMBO. The τ pair rides the non-prefix (both-beam) s-channel line
    // that only resonates after the routing-aware relabel.
    let (m2, mg) = (mz * mz, mz * gz);
    let bw = move |s: f64| 1.0 / ((s - m2).powi(2) + mg * mg);
    for (name, i, j, seed) in [("µµ", 0usize, 1usize, 0xB11u64), ("ττ", 2, 3, 0xB21)] {
        let probe = move |p: &[LorentzVector<f64>]| bw(s_pair(p, i, j));
        let (sig_m, var_m) = mc_estimate(&multi, seed, 45, n, probe);
        let (sig_f, var_f) = mc_estimate(&flat, seed + 1, 47, n, probe);
        let err = ((var_m + var_f) / n as f64).sqrt();
        eprintln!(
            "real {name} Z-pole: multi {sig_m:.6e} (var {var_m:.3e}) vs flat {sig_f:.6e} \
             (var {var_f:.3e}), variance ratio {:.1}×",
            var_f / var_m
        );
        assert!(
            (sig_m - sig_f).abs() < 6.0 * err,
            "{name}: combiner σ {sig_m:.6e} disagrees with flat {sig_f:.6e} on the Z pole"
        );
        assert!(
            var_m < var_f,
            "{name}: combiner variance {var_m:.3e} not below flat RAMBO {var_f:.3e}"
        );
    }
}
