//! Diagram-driven phase-space channel: extraction from the `Prop` chain.
//!
//! Complements the in-crate unit tests (which drive the sampler off explicit
//! topologies) by building channels from real enumerated diagrams and checking
//! that the propagator chain yields on-shell, momentum-conserving kinematics and
//! that the s-channel resonance / t-channel metadata a resonance-aware map will
//! consume is populated from the model.
//!
//! The last group measures what a peripheral (t-channel spine) map would do for a
//! `p p → l+ l- j` subprocess, whose single spacelike line sits in a three-body
//! final state rather than the two-body one the spine was introduced for. Built by
//! hand from each diagram's own cut, such a map integrates that diagram's peaked
//! structure far better than anything currently derived for it — but only once its
//! spacelike pole is regulated: with the model's massless quark exchange the map is
//! numerically degenerate at the collinear edge and comes out biased, which is
//! asserted here rather than left to be discovered downstream.

mod common;

use vibegraph::diagrams::diagram::Diagram;
use vibegraph::helas::LorentzVector;
use vibegraph::phasespace::rng::SubStream;
use vibegraph::phasespace::{
    Channel, DiagramChannel, MultiChannel, PhaseSpaceMap, RamboChannel, Resonance,
};
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

/// Assert a channel's points are physical and its two weight computations agree,
/// returning the worst relative disagreement seen so a caller can report it.
fn assert_valid(ch: &DiagramChannel<f64>, sqrt_s: f64, masses: &[f64], seed: u64) -> f64 {
    let n_out = masses.len();
    assert_eq!(ch.ndim(), 3 * n_out - 4);
    let mut stream = SubStream::from_stream(seed, 3);
    let mut worst = 0.0f64;
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
        // `sample` accumulates this weight from the invariants it drew; `density`
        // rebuilds it from the momenta they produced. The two are separate
        // computations, so their agreement does see a sampling density that
        // differs from the weighting one — the defect an unfloored peripheral
        // rung carries. It remains a pointwise check: how well the density
        // *matches the integrand* is a question only an integrated quantity
        // answers (`V_n` against flat RAMBO, or a seed sweep).
        let recip = 1.0 / ch.density(&pt.momenta);
        let rel = (pt.weight - recip).abs() / recip;
        worst = worst.max(rel);
        assert!(
            rel < WALK_DENSITY_TOL,
            "walk weight {} vs 1/density {recip} (rel {rel:.3e})",
            pt.weight
        );
    }
    worst
}

/// Bound on the relative gap between the weight [`DiagramChannel`]'s walk
/// accumulates and the one its density reconstructs from the realised momenta.
///
/// The two multiply the same factors in different orders and rebuild each
/// invariant from different inputs, so they agree only to rounding: worst measured
/// 7.1e-9 over every diagram-derived channel here, and 1.2e-8 on a floored llj
/// spine, both from near-threshold configurations where the Källén functions cancel
/// hardest. The bound sits above those and some twelve orders below the mismatch it
/// exists to catch — an unfloored spine reaches 4e4 on the same measure.
const WALK_DENSITY_TOL: f64 = 1e-7;

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
    let mut worst = 0.0f64;
    for process in processes {
        let sets = common::generate(process);
        assert!(!sets.is_empty(), "no diagrams for {process}");
        let diagrams: &[Diagram] = &sets[0].diagrams;
        assert!(!diagrams.is_empty(), "empty diagram set for {process}");
        for (i, d) in diagrams.iter().enumerate() {
            let masses = out_masses(d, &evaluated);
            let ch = DiagramChannel::from_diagram(d, &evaluated, sqrt_s);
            worst = worst.max(assert_valid(&ch, sqrt_s, &masses, 0xC0DE + i as u64));
        }
    }
    eprintln!("walk weight vs 1/density over every diagram channel: worst {worst:.3e} relative");
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

// ── Spacelike lines in a three-body final state ──────────────────────────────

/// The `p p > l+ l- j` subprocess classes, as one concrete flavour assignment each.
const LLJ_SUBPROCESSES: [&str; 2] = ["u u~ > e+ e- g QCD=2 QED=2", "g u > e+ e- u QCD=2 QED=2"];

/// The outgoing-slot split a spacelike line induces, read the way the channel
/// derivation reads it: the emitted subsystem is the outgoing legs sharing beam
/// `0`'s side of the cut. `momentum` sign-decorates the externals on one side, so
/// only the nonzero pattern matters.
fn spine_partition(momentum: &[i8], n_in: usize, n_ext: usize) -> (Vec<usize>, Vec<usize>) {
    let stored: Vec<usize> = (n_in..n_ext)
        .filter(|&i| momentum[i] != 0)
        .map(|i| i - n_in)
        .collect();
    let emitted = if momentum[0] != 0 {
        stored
    } else {
        (0..n_ext - n_in).filter(|s| !stored.contains(s)).collect()
    };
    let recoil = (0..n_ext - n_in)
        .filter(|s| !emitted.contains(s))
        .collect::<Vec<_>>();
    (emitted, recoil)
}

/// One `llj` diagram reduced to the facts a peripheral map would be built from,
/// alongside the channel the derivation actually produces for it today.
struct SpacelikeCut {
    emitted: Vec<usize>,
    recoil: Vec<usize>,
    /// Mass of the spacelike propagator.
    t_mass: f64,
    /// Resonance on whichever side is the lepton pair, if the diagram has one.
    lepton_pair_resonance: Option<Resonance<f64>>,
    /// What `from_diagram` builds for this diagram.
    derived: DiagramChannel<f64>,
}

/// Every single-spacelike-line diagram of `process`, with its cut.
fn spacelike_cuts(
    process: &str,
    model: &EvaluatedModel,
    sqrt_s: f64,
) -> (Vec<SpacelikeCut>, usize, Vec<f64>) {
    let sets = common::generate(process);
    let diagrams = &sets[0].diagrams;
    let masses = out_masses(&diagrams[0], model);
    let mut cuts = Vec::new();
    for d in diagrams {
        let n_ext = d.n_ext();
        let spacelike: Vec<_> = d
            .props
            .iter()
            .filter(|p| p.is_spacelike(d.n_in))
            .collect::<Vec<_>>();
        if spacelike.len() != 1 {
            continue;
        }
        let (emitted, recoil) = spine_partition(&spacelike[0].momentum, d.n_in, n_ext);
        // The lepton pair is the two outgoing leptons; its timelike line is the
        // only resonance a `2 -> 3` llj diagram carries.
        let derived = DiagramChannel::<f64>::from_diagram(d, model, sqrt_s);
        cuts.push(SpacelikeCut {
            emitted,
            recoil,
            t_mass: model.mass(spacelike[0].particle),
            lepton_pair_resonance: derived.resonances().first().copied(),
            derived,
        });
    }
    (cuts, diagrams.len(), masses)
}

/// Both `llj` subprocess classes put a *single* spacelike line into a three-body
/// final state, and that line always separates the lepton pair from the jet.
///
/// This is the shape the peripheral map would have to handle, and it is one step
/// beyond the single spacelike line in a `2 → 2` the [`DiagramChannel`] spine was
/// built for: one of the two sides is now a composite subsystem carrying its own
/// invariant, not a single on-shell leg.
#[test]
fn llj_diagrams_cut_a_three_body_final_state_with_one_spacelike_line() {
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let z = model.particle_id("Z").expect("Z in model");
    let mz = evaluated.mass(z);

    for process in LLJ_SUBPROCESSES {
        let (cuts, n_diagrams, masses) = spacelike_cuts(process, &evaluated, 500.0);
        assert_eq!(
            masses.len(),
            3,
            "{process}: expected a three-body final state"
        );
        assert!(
            masses.iter().all(|&m| m == 0.0),
            "{process}: every outgoing leg should be massless here"
        );
        assert!(
            !cuts.is_empty(),
            "{process}: no diagram carries a spacelike line"
        );
        let mut resonant = 0usize;
        for cut in &cuts {
            let mut both: Vec<usize> = cut.emitted.iter().chain(&cut.recoil).copied().collect();
            both.sort_unstable();
            assert_eq!(both, vec![0, 1, 2], "{process}: cut does not partition");
            // Outgoing slots 0 and 1 are the leptons in both process strings.
            let pair_side = if cut.emitted.contains(&0) {
                &cut.emitted
            } else {
                &cut.recoil
            };
            let mut pair = pair_side.clone();
            pair.sort_unstable();
            assert_eq!(
                pair,
                vec![0, 1],
                "{process}: the spacelike line does not separate the lepton pair from the jet"
            );
            assert_eq!(
                cut.t_mass, 0.0,
                "{process}: the spacelike line is expected to be a massless quark"
            );
            if let Some(r) = cut.lepton_pair_resonance {
                if (r.mass - mz).abs() < 1e-6 {
                    resonant += 1;
                }
            }
        }
        assert!(
            resonant > 0,
            "{process}: no Z-pole subsystem alongside the spacelike line"
        );
        eprintln!(
            "{process}: {}/{n_diagrams} diagrams carry exactly one spacelike line, \
             {resonant} of them alongside the Z pole; cuts (emitted|recoil) = {:?}",
            cuts.len(),
            cuts.iter()
                .map(|c| (c.emitted.clone(), c.recoil.clone()))
                .collect::<Vec<_>>()
        );
    }
}

/// Build the peripheral map an `llj` cut implies, with the lepton pair's Z pole on
/// whichever side carries it, and `t_mass` standing in for the spacelike line's
/// mass so the pole can be regulated independently of the model value.
fn build_spine(
    sqrt_s: f64,
    masses: &[f64],
    cut: &SpacelikeCut,
    t_mass: f64,
) -> DiagramChannel<f64> {
    let pair_is_emitted = cut.emitted.contains(&0);
    let res = cut.lepton_pair_resonance;
    DiagramChannel::from_topology_tchannel(
        sqrt_s,
        [0.0, 0.0],
        masses.to_vec(),
        (
            cut.emitted.clone(),
            pair_is_emitted.then_some(res).flatten(),
        ),
        (
            cut.recoil.clone(),
            (!pair_is_emitted).then_some(res).flatten(),
        ),
        t_mass,
    )
}

/// The upper edge of the spacelike transfer, spelled exactly as the peripheral
/// kinematics spell it: `t_max = m_a² + s₁ − 2·E_a·E₁ + 2·k·p*`, each factor built
/// from the Källén function the same way.
fn t_max_as_computed(s: f64, s1: f64, s2: f64) -> f64 {
    fn kallen(a: f64, b: f64, c: f64) -> f64 {
        a * a + b * b + c * c - 2.0 * (a * b + b * c + c * a)
    }
    let sqrt_s = s.sqrt();
    let inv = 1.0 / (2.0 * sqrt_s);
    let ea = s * inv;
    let e1 = (s + s1 - s2) * inv;
    let k = kallen(s, 0.0, 0.0).max(0.0).sqrt() * inv;
    let pstar = kallen(s, s1, s2).max(0.0).sqrt() * inv;
    (s1 - 2.0 * ea * e1) + 2.0 * k * pstar
}

/// With a massless spacelike line and a massless subsystem on one side of it, the
/// transfer's upper edge sits *exactly* on the pole: `t_max = m² = 0`. Analytically
/// that is a clean statement; numerically it is a difference of two large equal
/// quantities, and the two are built from different expressions, so it lands on
/// either side of zero at the rounding scale.
///
/// This decides whether the peripheral draw importance-samples the propagator or
/// falls back to flat — a choice that therefore turns over on floating-point noise
/// rather than on kinematics. Both signs really occur, which is what makes
/// [`the_unregulated_three_body_spine_is_biased_at_the_collinear_edge`] a defect and
/// not a tolerance question.
#[test]
fn a_massless_spacelike_pole_puts_the_transfer_edge_on_rounding_noise() {
    let s = 250_000.0;
    let (mut negative, mut positive, mut exact) = (0usize, 0usize, 0usize);
    let mut worst = 0.0f64;
    // The recoil invariants a run actually sees are whatever the invariant draw
    // returns, so the sweep uses arbitrary values rather than tidy fractions of
    // `s` — the latter cancel exactly and would hide the effect.
    let n = 20_000;
    let mut stream = SubStream::from_stream(0x7EDBE, 5);
    let draws = stream.uniforms::<f64>(n);
    for &x in &draws {
        let s2 = s * x;
        let t_max = t_max_as_computed(s, 0.0, s2);
        worst = worst.max(t_max.abs());
        match t_max.partial_cmp(&0.0).expect("finite") {
            std::cmp::Ordering::Less => negative += 1,
            std::cmp::Ordering::Greater => positive += 1,
            std::cmp::Ordering::Equal => exact += 1,
        }
    }
    eprintln!(
        "massless t edge over {n} recoil invariants: {negative} below zero, {positive} above, \
         {exact} exactly zero; worst |t_max| = {worst:.3e} (s = {s})"
    );
    assert!(
        negative > 0 && positive > 0,
        "the transfer edge no longer straddles zero, so the pole/flat decision may have \
         stopped depending on rounding"
    );
    assert!(
        worst < 1e-9 * s,
        "|t_max| reached {worst:.3e}, far more than rounding: the edge is no longer the \
         analytic zero this argument assumes"
    );
    // Over the same draws, a spacelike line given a mass well above that noise
    // keeps the edge strictly below the pole, which is what makes the pole usable.
    let m2 = 25.0;
    for &x in &draws {
        assert!(
            m2 - t_max_as_computed(s, 0.0, s * x) > 0.0,
            "a massive spacelike line should keep the whole window below its pole"
        );
    }
}

/// A three-body peripheral spine built from an `llj` cut, with its spacelike pole
/// regulated above the rounding scale, is a valid map: on-shell, conserving, and
/// integrating the flat volume `V_3` to the same number as flat RAMBO.
///
/// Nothing in the spine machinery assumes the `2 → 2` shape it was introduced for —
/// either side may be a composite subsystem with its own invariant and decay tree,
/// and the dimension count works out. What it does assume is a usable pole.
#[test]
fn a_regulated_three_body_spine_from_an_llj_cut_is_a_valid_map() {
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let sqrt_s = 500.0;
    let regulator = sqrt_s / 100.0;

    let mut checked = 0usize;
    for process in LLJ_SUBPROCESSES {
        let (cuts, _, masses) = spacelike_cuts(process, &evaluated, sqrt_s);
        for (i, cut) in cuts.iter().enumerate() {
            let spine = build_spine(sqrt_s, &masses, cut, regulator);
            assert_valid(&spine, sqrt_s, &masses, 0x5D1CE + i as u64);

            let flat = RamboChannel::new(sqrt_s, masses.clone());
            let n = 200_000;
            let (v_s, var_s) = mc_estimate(&spine, 0xC01 + i as u64, 61, n, |_| 1.0);
            let (v_f, var_f) = mc_estimate(&flat, 0xC02 + i as u64, 63, n, |_| 1.0);
            let err = ((var_s + var_f) / n as f64).sqrt();
            eprintln!(
                "{process} cut {i} (t pole regulated at {regulator} GeV): V_3 spine {v_s:.6e} \
                 vs flat {v_f:.6e} (+-{err:.1e})"
            );
            assert!(
                (v_s - v_f).abs() < 6.0 * err,
                "{process} cut {i}: spine V_3 {v_s:.6e} disagrees with flat RAMBO {v_f:.6e}"
            );
            checked += 1;
        }
    }
    assert!(checked > 0, "no llj cut was exercised");
}

/// An unregulated three-body spine samples from a density its own
/// [`Channel::density`] does not describe — and *that*, not the walk, is what a
/// multichannel combiner weights by.
///
/// The mechanism is
/// [`a_massless_spacelike_pole_puts_the_transfer_edge_on_rounding_noise`]: on the
/// draws where the edge lands just below zero the propagator map switches on with a
/// span of some thirty e-folds reaching down to `|t| ~ 1e-11`, while `density`
/// re-derives the transfer from the momenta with a cancellation error of the same
/// size. The walk knows which `t` it drew, so a standalone draw weighted by its own
/// accumulated Jacobian stays consistent; `density` evaluated at the realised
/// momenta does not, and a combiner discards the walk weight in favour of
/// `αⱼ / Σₖ αₖ gₖ`, built from `density` alone.
///
/// So the two halves are asserted separately: the per-point gap between the two
/// weightings blows up to O(1) or worse unregulated and collapses to rounding once
/// floored, and the floor is what the combiner path therefore requires. That the
/// floored map is otherwise a faithful one is
/// [`a_regulated_three_body_spine_from_an_llj_cut_is_a_valid_map`].
#[test]
fn an_unregulated_three_body_spine_breaks_the_density_a_combiner_weights_by() {
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let sqrt_s = 500.0;
    let regulator = sqrt_s / 100.0;

    let mut worst_unregulated = 0.0f64;
    let mut worst_regulated = 0.0f64;
    let mut compared = 0usize;
    for process in LLJ_SUBPROCESSES {
        let (cuts, _, masses) = spacelike_cuts(process, &evaluated, sqrt_s);
        for (i, cut) in cuts.iter().enumerate() {
            assert_eq!(
                cut.t_mass, 0.0,
                "{process} cut {i}: expected a massless line"
            );
            // The largest relative gap between the weight the walk accumulated and
            // the one `density` reconstructs, over a fixed draw sequence.
            let gap = |floor: f64, seed: u64| -> f64 {
                let spine = build_spine(sqrt_s, &masses, cut, floor);
                let mut stream = SubStream::from_stream(seed, 67);
                let mut worst = 0.0f64;
                for _ in 0..50_000 {
                    let u = stream.uniforms::<f64>(spine.ndim());
                    let pt = spine.sample(&u);
                    let recip = 1.0 / spine.density(&pt.momenta);
                    if recip > 0.0 && recip.is_finite() {
                        worst = worst.max((pt.weight - recip).abs() / recip);
                    }
                }
                worst
            };
            let bad = gap(0.0, 0xDEF0 + i as u64);
            let good = gap(regulator, 0xDEF0 + i as u64);
            eprintln!(
                "{process} cut {i}: walk-vs-density gap {bad:.2e} unregulated, \
                 {good:.2e} floored at {regulator} GeV"
            );
            assert!(
                bad > 1e-3,
                "{process} cut {i}: the unregulated spine's two weightings now agree to \
                 {bad:.2e} — if the transfer edge stopped straddling the pole, this test has \
                 lost its subject"
            );
            assert!(
                good < WALK_DENSITY_TOL,
                "{process} cut {i}: the floored spine's weightings disagree by {good:.2e}"
            );
            worst_unregulated = worst_unregulated.max(bad);
            worst_regulated = worst_regulated.max(good);
            compared += 1;
        }
    }
    assert!(compared > 0, "no llj cut was exercised");
    eprintln!(
        "over {compared} llj cuts: worst walk-vs-density gap {worst_unregulated:.2e} \
         unregulated, {worst_regulated:.2e} floored"
    );
}

/// The consequence of the gap above, at the contract a combiner actually rests on:
/// an *unregulated* three-body spine returns a non-positive [`Channel::density`] at
/// points it generated itself, and a floored one never does.
///
/// A combiner weights every point by `αⱼ / Σₖ αₖ gₖ` with each `gₖ` read from
/// `density`, so a channel that reports zero density where it just placed a point
/// does not merely mis-normalise the mixture — it puts a zero in the denominator.
/// "Multichannel is unbiased under a bad map" does not cover a map that breaks the
/// density contract, which is the sense in which the unfloored spine is wrong
/// rather than bad. The floored map's V_3 through the combiner is checked against
/// flat RAMBO here too, since the unregulated one cannot be run at all.
#[test]
fn an_unregulated_spine_breaks_the_positive_density_contract() {
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let sqrt_s = 500.0;
    let regulator = sqrt_s / 100.0;
    let n = 200_000;

    let (cuts, _, masses) = spacelike_cuts(LLJ_SUBPROCESSES[0], &evaluated, sqrt_s);

    // How often a spine's own density comes out non-positive or non-finite on the
    // points it generated.
    let degenerate = |floor: f64, seed: u64| -> usize {
        let mut bad = 0usize;
        for (i, cut) in cuts.iter().enumerate() {
            let spine = build_spine(sqrt_s, &masses, cut, floor);
            let mut stream = SubStream::from_stream(seed + i as u64, 73);
            for _ in 0..n {
                let u = stream.uniforms::<f64>(spine.ndim());
                let pt = spine.sample(&u);
                let g = spine.density(&pt.momenta);
                if !(g > 0.0) || !g.is_finite() {
                    bad += 1;
                }
            }
        }
        bad
    };

    let bad_unregulated = degenerate(0.0, 0xB1A5);
    let bad_floored = degenerate(regulator, 0xB1A6);
    let drawn = n * cuts.len();
    eprintln!(
        "self-density over {drawn} points on {} llj spines: {bad_unregulated} non-positive \
         unregulated, {bad_floored} floored at {regulator} GeV",
        cuts.len()
    );
    assert!(
        bad_unregulated > 0,
        "the unregulated spine no longer breaks its own density contract — if the spacelike \
         draw grew a floor of its own, retire this test and enforce the agreement instead"
    );
    assert_eq!(
        bad_floored, 0,
        "the floored spine reported a non-positive density on a point it generated"
    );

    // With the contract restored, the combiner runs and reproduces the volume.
    let channels: Vec<Box<dyn Channel<f64>>> = cuts
        .iter()
        .map(|c| Box::new(build_spine(sqrt_s, &masses, c, regulator)) as Box<dyn Channel<f64>>)
        .collect();
    let combiner = MultiChannel::uniform(channels);
    let flat = RamboChannel::new(sqrt_s, masses.clone());
    let (reference, var_ref) = mc_estimate(&flat, 0xB1A7, 75, n, |_| 1.0);
    let (v_good, var_good) = mc_estimate(&combiner, 0xB1A8, 77, n, |_| 1.0);
    let err = ((var_good + var_ref) / n as f64).sqrt();
    eprintln!("floored combiner V_3 = {v_good:.6e}, flat RAMBO {reference:.6e} ± {err:.2e}");
    assert!(
        (v_good - reference).abs() < 6.0 * err,
        "floored combiner V_3 {v_good:.6e} vs flat RAMBO {reference:.6e} ± {err:.2e}"
    );
}

/// Does a three-body peripheral map earn its place? On a toy integrand carrying the
/// two structures an `llj` matrix element has — the lepton pair's Z pole and the
/// spacelike propagator — the regulated spine is compared against the all-timelike
/// channel the derivation builds for the very same diagram, and against flat RAMBO.
///
/// The comparison is a seed sweep, not a single run, because a single run cannot
/// tell a converged estimate from an under-covered one: a map that misses the
/// peripheral region reports a small integral *and* a small variance, and looks
/// perfectly stable from the inside. What separates them is whether independent
/// seeds land within the error each of them claims. The spine's do; the other two
/// maps' do not, and they disagree with the spine by factors, so the assertion here
/// is *not* that the three agree — it is that only the spine is self-consistent.
#[test]
fn a_regulated_three_body_spine_beats_the_all_timelike_channel_on_a_peripheral_integrand() {
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let z = model.particle_id("Z").expect("Z in model");
    let (mz, gz) = (evaluated.mass(z), evaluated.width(z));
    let sqrt_s = 500.0;
    let regulator = sqrt_s / 100.0;
    let m0_2 = regulator * regulator;
    // Beam 0 along +z in the partonic CM, as the peripheral map anchors it.
    let beam0 = [sqrt_s / 2.0, 0.0, 0.0, sqrt_s / 2.0];

    let (m2, mg) = (mz * mz, mz * gz);
    let mut compared = 0usize;
    let mut worst_alternative_pull = 0.0f64;
    let mut worst_miss = 1.0f64;
    for process in LLJ_SUBPROCESSES {
        let (cuts, _, masses) = spacelike_cuts(process, &evaluated, sqrt_s);
        for (i, cut) in cuts.iter().enumerate() {
            let emitted = cut.emitted.clone();
            let probe = |p: &[LorentzVector<f64>]| {
                let s_ll = s_pair(p, 0, 1);
                let [mut e, mut px, mut py, mut pz] = beam0;
                for &slot in &emitted {
                    e -= p[slot].e();
                    px -= p[slot].px();
                    py -= p[slot].py();
                    pz -= p[slot].pz();
                }
                let t = e * e - px * px - py * py - pz * pz;
                1.0 / (((s_ll - m2).powi(2) + mg * mg) * (m0_2 - t).powi(2))
            };

            let spine = build_spine(sqrt_s, &masses, cut, regulator);
            let flat = RamboChannel::new(sqrt_s, masses.clone());
            let maps: [(&str, &dyn PhaseSpaceMap<f64>); 3] = [
                ("spine", &spine),
                ("all-timelike", &cut.derived),
                ("flat", &flat),
            ];
            let n = 60_000;

            let mut pulls = Vec::new();
            let mut variances = Vec::new();
            for (label, map) in maps {
                let runs: Vec<(f64, f64)> = (0..5)
                    .map(|seed| {
                        mc_estimate(map, 0xD00 + 16 * seed + i as u64, 71 + seed, n, &probe)
                    })
                    .collect();
                let mean = runs.iter().map(|r| r.0).sum::<f64>() / runs.len() as f64;
                let var = runs.iter().map(|r| r.1).sum::<f64>() / runs.len() as f64;
                let err = (var / n as f64).sqrt();
                // The largest deviation any one seed shows, in units of the error
                // that seed's own run claims.
                let pull = runs
                    .iter()
                    .map(|r| (r.0 - mean).abs() / (r.1 / n as f64).sqrt())
                    .fold(0.0f64, f64::max);
                eprintln!(
                    "{process} cut {i} {label}: {mean:.6e} +- {err:.1e} over 5 seeds, \
                     worst seed pull {pull:.1}, per-point variance {var:.2e}"
                );
                pulls.push((label, pull));
                variances.push((label, var, mean));
            }

            let spine_pull = pulls[0].1;
            assert!(
                spine_pull < 5.0,
                "{process} cut {i}: the spine's seeds scatter by {spine_pull:.1} of their own \
                 claimed error, so it is not covering the integrand either"
            );
            let (_, spine_var, spine_mean) = variances[0];
            for (label, var, mean) in &variances[1..] {
                assert!(
                    spine_var < *var,
                    "{process} cut {i}: the peripheral map does not reduce variance against \
                     {label} ({spine_var:.2e} vs {var:.2e})"
                );
                worst_miss = worst_miss.max((mean / spine_mean).max(spine_mean / mean));
            }
            worst_alternative_pull = pulls[1..]
                .iter()
                .fold(worst_alternative_pull, |w, (_, p)| w.max(*p));
            compared += 1;
        }
    }
    assert!(compared > 0, "no llj cut was compared");
    // Neither alternative is merely slower here: on at least one cut each of them
    // is off by a factor while reporting an error that does not admit it. That is
    // the failure a single-seed run would have accepted as an answer.
    eprintln!(
        "peripheral probe over {compared} cuts: worst alternative-map seed pull \
         {worst_alternative_pull:.1}, worst factor away from the spine {worst_miss:.2}x"
    );
    assert!(
        worst_alternative_pull > 5.0 && worst_miss > 2.0,
        "no alternative map under-covers any more (worst pull {worst_alternative_pull:.1}, \
         worst factor {worst_miss:.2}) — the comparison has lost its control"
    );
}
