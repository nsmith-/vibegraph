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

use vibegraph::cuts::{Cuts, ExternalLeg};
use vibegraph::diagrams::diagram::{Diagram, LegIdx, Ray};
use vibegraph::helas::LorentzVector;
use vibegraph::phasespace::rng::SubStream;
use vibegraph::phasespace::{
    Channel, DiagramChannel, MultiChannel, PhaseSpaceMap, RamboChannel, Resonance, RungSpec,
};
use vibegraph::runcard::RunCard;
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

/// The upper edge of the spacelike transfer, `t_max = m_a² + s₁ − 2·E_a·E₁ + 2·k·p*`,
/// built from a Källén function evaluated in one of two algebraically equal ways:
/// the expanded `a²+b²+c²−2(ab+bc+ca)`, or the grouped `(a−b−c)² − 4bc` the
/// peripheral kinematics uses.
fn t_max_as_computed(s: f64, s1: f64, s2: f64, grouped: bool) -> f64 {
    let kallen = |a: f64, b: f64, c: f64| {
        if grouped {
            let d = a - b - c;
            d * d - 4.0 * b * c
        } else {
            a * a + b * b + c * c - 2.0 * (a * b + b * c + c * a)
        }
    };
    let sqrt_s = s.sqrt();
    let inv = 1.0 / (2.0 * sqrt_s);
    let ea = s * inv;
    let e1 = (s + s1 - s2) * inv;
    let k = kallen(s, 0.0, 0.0).max(0.0).sqrt() * inv;
    let pstar = kallen(s, s1, s2).max(0.0).sqrt() * inv;
    (s1 - 2.0 * ea * e1) + 2.0 * k * pstar
}

/// With a massless spacelike line and a massless subsystem on one side of it, the
/// transfer's upper edge sits *exactly* on the pole, `t_max = m² = 0`. Whether the
/// arithmetic reproduces that, or lands on either side of zero at the rounding
/// scale, is decided entirely by how the Källén function is grouped — and the edge
/// is what decides whether the peripheral draw importance-samples the propagator or
/// falls back to flat.
///
/// The expanded `a²+b²+c²−2(ab+bc+ca)` reaches the answer by cancelling terms of
/// order `s²` against each other. In the soft-emission corner, where the recoil
/// carries almost all the invariant mass, the true value is many orders below
/// those terms and only a few digits survive: the edge then straddles zero, and
/// the pole/flat decision turns over on floating-point noise rather than on
/// kinematics. The grouped `(a−b−c)² − 4bc` cancels once, at `a−b−c`, and returns
/// the analytic zero exactly. Both are checked here over the same draws, so the
/// grouping the peripheral kinematics uses is pinned rather than assumed.
#[test]
fn only_the_grouped_kallen_puts_the_massless_transfer_edge_on_its_analytic_zero() {
    let s = 250_000.0;
    let (mut negative, mut positive, mut exact) = (0usize, 0usize, 0usize);
    let mut worst_expanded = 0.0f64;
    let mut worst_grouped = 0.0f64;
    // The recoil invariants a run actually sees are whatever the invariant draw
    // returns, so the sweep uses arbitrary values rather than tidy fractions of
    // `s` — the latter cancel exactly and would hide the effect.
    let n = 20_000;
    let mut stream = SubStream::from_stream(0x7EDBE, 5);
    let draws = stream.uniforms::<f64>(n);
    for &x in &draws {
        let s2 = s * x;
        let t_max = t_max_as_computed(s, 0.0, s2, false);
        worst_expanded = worst_expanded.max(t_max.abs());
        worst_grouped = worst_grouped.max(t_max_as_computed(s, 0.0, s2, true).abs());
        match t_max.partial_cmp(&0.0).expect("finite") {
            std::cmp::Ordering::Less => negative += 1,
            std::cmp::Ordering::Greater => positive += 1,
            std::cmp::Ordering::Equal => exact += 1,
        }
    }
    eprintln!(
        "massless t edge over {n} recoil invariants, expanded Källén: {negative} below zero, \
         {positive} above, {exact} exactly zero, worst |t_max| = {worst_expanded:.3e}; \
         grouped Källén worst |t_max| = {worst_grouped:.3e} (s = {s})"
    );
    assert!(
        negative > 0 && positive > 0,
        "the expanded form's edge no longer straddles zero, so this sweep has lost the \
         conditioning failure it exists to exhibit"
    );
    assert!(
        worst_expanded > 1e-13 * s,
        "the expanded form reached only {worst_expanded:.3e}, so the two groupings are no \
         longer distinguishable here"
    );
    assert_eq!(
        worst_grouped, 0.0,
        "the grouped form put the massless transfer edge at {worst_grouped:.3e} rather than \
         on its analytic zero"
    );
    // Over the same draws, a spacelike line given a mass well above that noise
    // keeps the edge strictly below the pole, which is what makes the pole usable.
    let m2 = 25.0;
    for &x in &draws {
        assert!(
            m2 - t_max_as_computed(s, 0.0, s * x, true) > 0.0,
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

/// An unregulated three-body spine can sample from a density its own
/// [`Channel::density`] does not describe — and *that*, not the walk, is what a
/// multichannel combiner weights by.
///
/// The mechanism is the massless transfer edge. On a cut where the edge lands just
/// below zero the propagator map switches on with a span of some thirty e-folds
/// reaching down to `|t| ~ 1e-11`, while `density` re-derives the transfer from the
/// momenta with a cancellation error of the same size. The walk knows which `t` it
/// drew, so a standalone draw weighted by its own accumulated Jacobian stays
/// consistent; `density` evaluated at the realised momenta does not, and a combiner
/// discards the walk weight in favour of `αⱼ / Σₖ αₖ gₖ`, built from `density`
/// alone.
///
/// Which cuts reach the edge is decided by which side of the rung carries a *drawn*
/// invariant. When the emitted subsystem's invariant is fixed — a single on-shell
/// leg — the edge is the grouped Källén function's exact zero and the flat fallback
/// fires deterministically; when the emitted side is the composite one, the edge is
/// a cancellation of the drawn invariant against `ŝ` and straddles zero on rounding.
/// So the defect is asserted where it survives and the agreement is required
/// everywhere it is floored, rather than assuming every cut behaves alike.
///
/// That the floored map is otherwise a faithful one is
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
            let bare = gap(0.0, 0xDEF0 + i as u64);
            let floored = gap(regulator, 0xDEF0 + i as u64);
            eprintln!(
                "{process} cut {i}: walk-vs-density gap {bare:.2e} unfloored, \
                 {floored:.2e} floored at {regulator} GeV"
            );
            assert!(
                floored < WALK_DENSITY_TOL,
                "{process} cut {i}: the floored spine's weightings disagree by {floored:.2e}"
            );
            worst_unregulated = worst_unregulated.max(bare);
            worst_regulated = worst_regulated.max(floored);
            compared += 1;
        }
    }
    assert!(compared > 0, "no llj cut was exercised");
    eprintln!(
        "over {compared} llj cuts: worst walk-vs-density gap {worst_unregulated:.2e} \
         unfloored, {worst_regulated:.2e} floored"
    );
    assert!(
        worst_unregulated > 1e-3,
        "no unfloored cut's two weightings disagree any more (worst \
         {worst_unregulated:.2e}) — if the transfer edge stopped straddling the pole \
         everywhere, this test has lost its subject"
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
         unfloored, {bad_floored} floored at {regulator} GeV",
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
                    .map(|seed| mc_estimate(map, 0xD00 + 16 * seed + i as u64, 71 + seed, n, probe))
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

/// A ladder's spacelike lines are **totally ordered by inclusion** of the outgoing
/// legs they leave on beam `0`'s side, so the chain of rungs — and with it which
/// blob is emitted at which rung, against which running momentum transfer — is a
/// property of the diagram and not a choice.
///
/// This is the hypothesis every ordered-rung decomposition rests on: sorting the
/// spacelike lines by the size of that side is the same as sorting them along the
/// chain only if the sides nest, and the blob emitted at rung `i` is well defined
/// only as the difference `S_i \ S_{i-1}`. Both fail loudly here if the sides ever
/// come out incomparable or equal.
///
/// The check is kept from being vacuous by requiring genuine ladders to be
/// present: a process whose diagrams all carried at most one spacelike line would
/// satisfy "the sides nest" for free.
#[test]
fn spacelike_lines_of_a_diagram_nest_into_an_ordered_rung_chain() {
    // The electroweak ladders: one, two and three spacelike lines in one process.
    let processes = [
        "u d > e+ e- u d QCD=0",
        "u u~ > e+ e- u u~ QCD=0",
        "u u~ > u u~",
        LLJ_SUBPROCESSES[0],
    ];
    let mut rungs_seen = std::collections::BTreeMap::<usize, usize>::new();
    let mut example = String::new();
    for process in processes {
        let sets = common::generate(process);
        for d in &sets[0].diagrams {
            let n_ext = d.n_ext();
            let mut sides: Vec<Vec<usize>> = d
                .props
                .iter()
                .filter(|p| p.is_spacelike(d.n_in))
                .map(|p| spine_partition(&p.momentum, d.n_in, n_ext).0)
                .collect();
            sides.sort_by_key(|s| s.len());
            *rungs_seen.entry(sides.len()).or_default() += 1;
            for w in sides.windows(2) {
                assert!(
                    w[0].len() < w[1].len(),
                    "{process}: two spacelike lines leave the same number of legs on \
                     beam 0's side ({:?} and {:?}), so sorting by size does not order \
                     the chain",
                    w[0],
                    w[1]
                );
                assert!(
                    w[0].iter().all(|s| w[1].contains(s)),
                    "{process}: spacelike sides {:?} and {:?} are incomparable, so the \
                     lines do not form a chain and no rung ordering exists",
                    w[0],
                    w[1]
                );
            }
            if sides.len() == 3 && example.is_empty() {
                let blobs: Vec<Vec<usize>> = (0..sides.len())
                    .map(|i| {
                        sides[i]
                            .iter()
                            .copied()
                            .filter(|s| i == 0 || !sides[i - 1].contains(s))
                            .collect()
                    })
                    .collect();
                let recoil: Vec<usize> = (0..n_ext - d.n_in)
                    .filter(|s| !sides[sides.len() - 1].contains(s))
                    .collect();
                example = format!("{process}: rung blobs {blobs:?}, recoil {recoil:?}");
            }
        }
    }
    eprintln!("spacelike-line count over the surveyed diagrams: {rungs_seen:?}");
    eprintln!("a three-rung chain: {example}");
    let ladders: usize = rungs_seen
        .iter()
        .filter(|(&k, _)| k >= 2)
        .map(|(_, &v)| v)
        .sum();
    assert!(
        ladders > 0,
        "no diagram carries two spacelike lines, so the nesting property is vacuous"
    );
    assert!(
        rungs_seen.contains_key(&3),
        "no three-rung ladder was surveyed — a chain of three is where sorting by \
         size could first disagree with sorting along the chain"
    );
}

/// How a fiducially cut process wants its massless spacelike pole treated: the
/// pole floored to the cut scale over the whole kinematic window, or kept bare
/// with the window itself bounded by that scale.
///
/// The two differ only over `|t| < pT_min²`, which the cuts reject anyway — the
/// floored map spends draws there and weights them zero, the bounded map does not
/// go there at all and reports density zero if asked. What that is worth, and
/// whether the narrowed support costs anything in agreement, is measured rather
/// than argued: three maps on the same `llj` cuts, the same peaked integrand and
/// the same seeds.
///
/// The integrand carries the two structures an `llj` matrix element has, with the
/// spacelike propagator left **massless** (`1/t²`, the singular thing the question
/// is about) and the run card's own cut indicator in front, which is what makes it
/// integrable. The baseline is the all-timelike channel the derivation builds for
/// these diagrams when no floor is supplied — past two outgoing legs that is what
/// "the map falls back flat" means concretely, since no spine is built at all.
///
/// Read as a measurement, not a gate: the assertions pin only that all three maps
/// agree on the integral (so the narrowed support loses nothing here) and that the
/// narrowing is actually in force (so the comparison is not vacuous).
#[test]
fn probe_fiducial_t_max_against_the_floored_pole_on_llj_cuts() {
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let z = model.particle_id("Z").expect("Z in model");
    let (mz, gz) = (evaluated.mass(z), evaluated.width(z));
    let (m2, mg) = (mz * mz, mz * gz);
    let sqrt_s = 500.0;
    let beam0 = [sqrt_s / 2.0, 0.0, 0.0, sqrt_s / 2.0];
    let beam1 = [sqrt_s / 2.0, 0.0, 0.0, -sqrt_s / 2.0];

    // The scale the banked llj card implies: ptj = 20 -> 400 GeV^2.
    let legs = vec![
        ExternalLeg::incoming(2, 0.0),
        ExternalLeg::incoming(-2, 0.0),
        ExternalLeg::outgoing(-11, 0.0),
        ExternalLeg::outgoing(11, 0.0),
        ExternalLeg::outgoing(21, 0.0),
    ];
    let cuts = Cuts::compile(&RunCard::default(), &legs).expect("llj cuts compile");
    let scale = cuts.spacelike_floor();
    assert_eq!(scale, 400.0);
    // The bounded arm keeps a pole three orders below the bound — negligible against
    // it, so the draw is the bare `1/|t|` the bound exists to expose, while a
    // configuration whose window the bound cannot narrow still gets a well-posed map
    // instead of the unregulated one whose density is documented as inconsistent.
    let safe_floor = scale / 1000.0;

    let n = 200_000;
    let seeds = 5u64;
    let mut compared = 0usize;
    let mut var_ratio_cb = Vec::new();
    let mut eff_gain = Vec::new();

    for process in LLJ_SUBPROCESSES {
        let (cutlist, _, masses) = spacelike_cuts(process, &evaluated, sqrt_s);
        for (i, cut) in cutlist.iter().enumerate() {
            let emitted = cut.emitted.clone();
            let transfer = move |p: &[LorentzVector<f64>]| {
                let [mut e, mut px, mut py, mut pz] = beam0;
                for &slot in &emitted {
                    e -= p[slot].e();
                    px -= p[slot].px();
                    py -= p[slot].py();
                    pz -= p[slot].pz();
                }
                e * e - px * px - py * py - pz * pz
            };
            let pass = |p: &[LorentzVector<f64>]| {
                let mut ext = Vec::with_capacity(2 + p.len());
                ext.push(LorentzVector::new(beam0[0], beam0[1], beam0[2], beam0[3]));
                ext.push(LorentzVector::new(beam1[0], beam1[1], beam1[2], beam1[3]));
                ext.extend_from_slice(p);
                cuts.pass(&ext)
            };
            let probe = |p: &[LorentzVector<f64>]| {
                if !pass(p) {
                    return 0.0;
                }
                let t = transfer(p);
                let s_ll = s_pair(p, 0, 1);
                1.0 / (((s_ll - m2).powi(2) + mg * mg) * t * t)
            };

            let floored = build_spine(sqrt_s, &masses, cut, scale.sqrt());
            let bounded =
                build_spine(sqrt_s, &masses, cut, safe_floor.sqrt()).with_fiducial_t_max(-scale);
            let maps: [(&str, &dyn PhaseSpaceMap<f64>); 3] = [
                ("all-timelike", &cut.derived),
                ("floored-pole", &floored),
                ("bounded-tmax", &bounded),
            ];

            let mut arms = Vec::new();
            for (label, map) in maps {
                let runs: Vec<(f64, f64)> = (0..seeds)
                    .map(|s| mc_estimate(map, 0xD3A0 + 16 * s + i as u64, 91 + s, n, probe))
                    .collect();
                let mean = runs.iter().map(|r| r.0).sum::<f64>() / seeds as f64;
                let var = runs.iter().map(|r| r.1).sum::<f64>() / seeds as f64;
                let err = (var / n as f64 / seeds as f64).sqrt();
                let pull = runs
                    .iter()
                    .map(|r| (r.0 - mean).abs() / (r.1 / n as f64).sqrt())
                    .fold(0.0f64, f64::max);
                // Cut efficiency: the share of draws the map spends inside the
                // fiducial region, which is the mechanism the bound acts through.
                let mut stream = SubStream::from_stream(0xEFF0 + i as u64, 7);
                let mut kept = 0usize;
                let probes = 40_000;
                for _ in 0..probes {
                    let u = stream.uniforms::<f64>(map.ndim());
                    if pass(&map.sample(&u).momenta) {
                        kept += 1;
                    }
                }
                let eff = kept as f64 / probes as f64;
                eprintln!(
                    "  {process} cut {i} {label:>13}: I = {mean:.6e} ± {err:.2e} \
                     ({seeds} seeds × {n}), per-point var {var:.3e}, worst pull {pull:.2}, \
                     cut efficiency {eff:.4}"
                );
                arms.push((label, mean, var, err, pull, eff));
            }

            let (_, i_time, var_time, e_time, _, _) = arms[0];
            let (_, i_floor, var_floor, e_floor, _, eff_floor) = arms[1];
            let (_, i_bound, var_bound, e_bound, _, eff_bound) = arms[2];
            eprintln!(
                "  {process} cut {i} SUMMARY: var(all-timelike)/var(floored) = {:.2}×, \
                 var(floored)/var(bounded) = {:.3}×, efficiency {eff_floor:.4} → {eff_bound:.4} \
                 ({:.2}×)",
                var_time / var_floor,
                var_floor / var_bound,
                eff_bound / eff_floor
            );
            var_ratio_cb.push(var_floor / var_bound);
            eff_gain.push(eff_bound / eff_floor);

            // Unbiasedness: narrowing the support must not move the integral. This
            // is the soundness half — a bound that cut into the surviving region
            // would show up here as a low estimate, not as a variance win.
            let err_fb = (e_floor * e_floor + e_bound * e_bound).sqrt();
            assert!(
                (i_floor - i_bound).abs() < 6.0 * err_fb,
                "{process} cut {i}: bounding t_max moved the integral \
                 ({i_bound:.6e} vs floored {i_floor:.6e}, err {err_fb:.2e})"
            );
            let err_tb = (e_time * e_time + e_bound * e_bound).sqrt();
            assert!(
                i_time > 0.0 && err_tb > 0.0,
                "{process} cut {i}: the all-timelike baseline produced nothing ({i_time:.3e})"
            );
            compared += 1;
        }
    }

    assert!(compared > 0, "no llj cut was compared");

    // Union coverage: a channel set whose members each renounce part of phase space
    // is unbiased only if between them they still reach everywhere the integrand
    // lives. Measured on the sharpest available integrand — the cut indicator
    // itself — over a combiner built from *only* bounded spines, so no unrestricted
    // channel can paper over a hole. Flat RAMBO covers everything by construction
    // and is the reference.
    let mut bounded_only: Vec<Box<dyn Channel<f64>>> = Vec::new();
    let mut masses_all = Vec::new();
    for process in LLJ_SUBPROCESSES {
        let (cutlist, _, masses) = spacelike_cuts(process, &evaluated, sqrt_s);
        masses_all = masses.clone();
        for cut in &cutlist {
            bounded_only.push(Box::new(
                build_spine(sqrt_s, &masses, cut, safe_floor.sqrt()).with_fiducial_t_max(-scale),
            ));
        }
    }
    let indicator = |p: &[LorentzVector<f64>]| {
        let mut ext = Vec::with_capacity(2 + p.len());
        ext.push(LorentzVector::new(beam0[0], beam0[1], beam0[2], beam0[3]));
        ext.push(LorentzVector::new(beam1[0], beam1[1], beam1[2], beam1[3]));
        ext.extend_from_slice(p);
        if cuts.pass(&ext) {
            1.0
        } else {
            0.0
        }
    };
    let _ = bounded_only;
    let flat = RamboChannel::new(sqrt_s, masses_all.clone());
    let nc = 400_000;
    let (v_f, var_f) = mc_estimate(&flat, 0xB0DF, 103, nc, indicator);

    // A ladder of bounds, from the scale the cuts provably imply up past the point
    // where the bound must start renouncing surviving phase space. It answers two
    // questions at once: how loose `pT_min²` is as a bound on the transfer, and
    // whether the coverage check can fire at all.
    let mut broke_at = None;
    for (k, mult) in [1.0f64, 10.0, 100.0, 250.0, 375.0, 500.0]
        .iter()
        .enumerate()
    {
        let cap = mult * scale;
        let mut set: Vec<Box<dyn Channel<f64>>> = Vec::new();
        for process in LLJ_SUBPROCESSES {
            let (cutlist, _, masses) = spacelike_cuts(process, &evaluated, sqrt_s);
            for cut in &cutlist {
                set.push(Box::new(
                    build_spine(sqrt_s, &masses, cut, safe_floor.sqrt()).with_fiducial_t_max(-cap),
                ));
            }
        }
        let (v_b, var_b) = mc_estimate(
            &MultiChannel::uniform(set),
            0xB0DE + k as u64,
            101 + k as u64,
            nc,
            indicator,
        );
        let err = ((var_b + var_f) / nc as f64).sqrt();
        let pull = (v_b - v_f).abs() / err;
        eprintln!(
            "fiducial-volume coverage @ |t| ≥ {cap:>8.0} GeV² ({:.3}·ŝ): bounded-spine \
             combiner {v_b:.6e} vs flat RAMBO {v_f:.6e} (± {err:.2e}), {pull:.1}σ",
            cap / (sqrt_s * sqrt_s)
        );
        if pull > 5.0 && broke_at.is_none() {
            broke_at = Some(cap);
        }
        if (mult - 1.0).abs() < 1e-12 {
            // The bound the design would actually install must lose nothing.
            assert!(
                pull < 5.0,
                "the bounded-spine channel set misses part of the cut-surviving region \
                 already at the cut scale: {v_b:.6e} vs flat RAMBO {v_f:.6e} (± {err:.2e})"
            );
        }
    }
    eprintln!(
        "coverage first breaks at |t| ≥ {:?} GeV²; the cut scale itself is {scale} GeV²",
        broke_at
    );
    assert!(
        broke_at.is_some(),
        "no bound on the ladder renounced any surviving phase space — the coverage \
         check cannot fire, so its passing at the cut scale means nothing"
    );

    let worst_eff = eff_gain.iter().cloned().fold(f64::INFINITY, f64::min);
    let best_var = var_ratio_cb.iter().cloned().fold(0.0f64, f64::max);
    eprintln!(
        "over {compared} llj cuts: smallest efficiency gain from bounding t_max \
         {worst_eff:.3}×, largest variance gain {best_var:.3}×"
    );
    // The comparison has content only if the bound is actually in force: a bounded
    // map that spent the same share of its draws inside the cuts would be the
    // floored map under another name.
    assert!(
        worst_eff > 1.0,
        "bounding t_max changed no map's cut efficiency — the arms are the same map"
    );
}

// ── The floor a run card implies, and what a zero one must leave alone ───────

/// Where the regulator's scale comes from: the process's own cuts, not the model.
///
/// The banked `p p > l+ l- j` card holds every jet above `ptj = 20`, and a
/// peripheral rung that produces a leg at transverse momentum `pT` transfers at
/// least `pT²`, so `|t| ≳ 400 GeV²` — ten orders above the `4e-8` cancellation
/// noise the unregulated edge sits on
/// ([`a_massless_spacelike_pole_puts_the_transfer_edge_on_rounding_noise`]).
#[test]
fn the_llj_run_card_supplies_the_scale_the_spacelike_pole_is_floored_at() {
    let legs = vec![
        ExternalLeg::incoming(2, 0.0),
        ExternalLeg::incoming(-2, 0.0),
        ExternalLeg::outgoing(-11, 0.0),
        ExternalLeg::outgoing(11, 0.0),
        ExternalLeg::outgoing(21, 0.0),
    ];
    let cuts = Cuts::compile(&RunCard::default(), &legs).expect("llj cuts compile");
    assert_eq!(cuts.spacelike_floor(), 400.0);
    let noise = 4e-8;
    assert!(
        cuts.spacelike_floor() / noise >= 1e10,
        "the floor no longer clears the transfer's cancellation noise by orders"
    );
}

/// A zero floor is the identity map on the channel derivation.
///
/// Every `lpp = 0` caller reaches the derivation through
/// [`DiagramChannel::from_diagram`], which supplies floor `0`, so a partonic run's
/// channels have to be bit-for-bit what they were before a regulator existed — the
/// banked `σ̂` artifacts depend on it. Asserted on the sampled momenta and the
/// densities rather than on the constructor arguments, since those are what a run
/// actually consumes.
///
/// The second half is what keeps the first from being vacuous: with the floor the
/// `llj` run card implies, the same derivation produces *different* channels. Note
/// that this holds for `2 → 2` spacelike diagrams too — the floor raises their pole
/// as well, so it is a per-process input and not a global constant.
#[test]
fn a_zero_spacelike_floor_leaves_every_channel_bit_identical() {
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let sqrt_s = 500.0;
    let floor = 400.0;

    let processes = [
        "e+ e- > mu+ mu-",
        "u u~ > u u~",
        "u u~ > d d~ g",
        LLJ_SUBPROCESSES[0],
        LLJ_SUBPROCESSES[1],
    ];

    let draw = |ch: &DiagramChannel<f64>, seed: u64| -> Vec<(Vec<[u64; 4]>, u64, u64)> {
        let mut stream = SubStream::from_stream(seed, 5);
        (0..200)
            .map(|_| {
                let u = stream.uniforms::<f64>(ch.ndim());
                let pt = ch.sample(&u);
                let bits: Vec<[u64; 4]> = pt
                    .momenta
                    .iter()
                    .map(|p| {
                        [
                            p.e().to_bits(),
                            p.px().to_bits(),
                            p.py().to_bits(),
                            p.pz().to_bits(),
                        ]
                    })
                    .collect();
                let density = ch.density(&pt.momenta).to_bits();
                (bits, pt.weight.to_bits(), density)
            })
            .collect()
    };

    let mut compared = 0usize;
    let mut moved_by_floor = 0usize;
    for process in processes {
        let sets = common::generate(process);
        for (i, d) in sets[0].diagrams.iter().enumerate() {
            let seed = 0x510E + i as u64;
            let plain = DiagramChannel::<f64>::from_diagram(d, &evaluated, sqrt_s);
            let zero = DiagramChannel::<f64>::from_diagram_regulated(d, &evaluated, sqrt_s, 0.0);
            assert_eq!(
                draw(&plain, seed),
                draw(&zero, seed),
                "{process} diagram {i}: a zero floor moved the channel"
            );
            let floored =
                DiagramChannel::<f64>::from_diagram_regulated(d, &evaluated, sqrt_s, floor);
            if draw(&floored, seed) != draw(&plain, seed) {
                moved_by_floor += 1;
            }
            compared += 1;
        }
    }
    eprintln!(
        "{compared} diagram channels bit-identical at floor 0; {moved_by_floor} move when floored \
         at {floor} GeV²"
    );
    assert!(
        moved_by_floor > 0,
        "no channel responds to the floor at all — the identity above is vacuous"
    );
}

// ── The ordered peripheral chain ─────────────────────────────────────────────

/// The reference process for the ordered peripheral chain: one concrete flavour
/// assignment of `p p > e+ e- j j` at `QCD = 0`, at fixed partonic beams.
///
/// 35 diagrams splitting `12 / 14 / 9` over one, two and three spacelike lines, so
/// the whole ladder spectrum is present — including the multiperipheral topology
/// where the two leptons leave the chain at *different* rungs, which a single-rung
/// spine cannot express at all. Its rungs are asymmetric: the blobs are a jet, a
/// lepton and a lepton pair, and the poles mix massless lines with `m_W` and `m_Z`,
/// which is what keeps a swapped ordering from being the same map.
const LADDER_PROCESS: &str = "u d > e+ e- u d QCD=0";
const LADDER_SQRT_S: f64 = 500.0;

/// One diagram's peripheral chain, derived here rather than asked of the channel:
/// the spacelike sides `S_1 ⊂ … ⊂ S_r` sorted by size, the blobs `B_i = S_i \ S_{i-1}`
/// they imply, what is left over, and each rung's *bare* propagator mass².
#[derive(Clone, Debug)]
struct RungChain {
    sides: Vec<Vec<usize>>,
    blobs: Vec<Vec<usize>>,
    recoil: Vec<usize>,
    poles: Vec<f64>,
}

fn rung_chain(d: &Diagram, model: &EvaluatedModel) -> RungChain {
    let n_out = d.n_ext() - d.n_in;
    let mut lines: Vec<(Vec<usize>, f64)> = d
        .props
        .iter()
        .filter(|p| p.is_spacelike(d.n_in))
        .map(|p| {
            let m = model.mass(p.particle);
            (spine_partition(&p.momentum, d.n_in, d.n_ext()).0, m * m)
        })
        .collect();
    lines.sort_by_key(|(s, _)| s.len());
    let sides: Vec<Vec<usize>> = lines.iter().map(|(s, _)| s.clone()).collect();
    let poles: Vec<f64> = lines.iter().map(|(_, m)| *m).collect();
    let blobs: Vec<Vec<usize>> = (0..sides.len())
        .map(|i| {
            sides[i]
                .iter()
                .copied()
                .filter(|s| i == 0 || !sides[i - 1].contains(s))
                .collect()
        })
        .collect();
    let recoil: Vec<usize> = (0..n_out)
        .filter(|s| sides.last().is_none_or(|l| !l.contains(s)))
        .collect();
    RungChain {
        sides,
        blobs,
        recoil,
        poles,
    }
}

/// The outgoing-leg slots on beam `0`'s side of propagator `cut`, derived by
/// deleting that propagator from the diagram's vertex graph and taking the connected
/// component beam `0` lands in — an independent route to the same partition, reading
/// the graph rather than the stored momentum routing.
fn beam0_side_by_graph_cut(d: &Diagram, cut: usize) -> Vec<usize> {
    let nv = d.vertices.len();
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); nv];
    for (pi, p) in d.props.iter().enumerate() {
        if pi == cut {
            continue;
        }
        adj[p.endpoints[0].0 .0].push(p.endpoints[1].0 .0);
        adj[p.endpoints[1].0 .0].push(p.endpoints[0].0 .0);
    }
    let side = |start: usize| -> Vec<usize> {
        let mut seen = vec![false; nv];
        let mut stack = vec![start];
        seen[start] = true;
        while let Some(v) = stack.pop() {
            for &w in &adj[v] {
                if !seen[w] {
                    seen[w] = true;
                    stack.push(w);
                }
            }
        }
        let mut ext = Vec::new();
        for (vi, vtx) in d.vertices.iter().enumerate() {
            if !seen[vi] {
                continue;
            }
            for ray in &vtx.rays {
                if let Ray::Leg(LegIdx(li)) = ray {
                    ext.push(*li);
                }
            }
        }
        ext.sort_unstable();
        ext
    };
    let a = side(d.props[cut].endpoints[0].0 .0);
    let with_beam0 = if a.contains(&0) {
        a
    } else {
        side(d.props[cut].endpoints[1].0 .0)
    };
    with_beam0
        .into_iter()
        .filter(|&l| l >= d.n_in)
        .map(|l| l - d.n_in)
        .collect()
}

/// The running momentum transfers `t_i = (p_a − Σ_{S_i} p)²` of a chain, computed
/// from the chain's own prefixes and never asked of the channel.
fn chain_transfers(
    chain: &RungChain,
    beam0: LorentzVector<f64>,
    p: &[LorentzVector<f64>],
) -> Vec<f64> {
    chain
        .sides
        .iter()
        .map(|side| {
            let (mut e, mut px, mut py, mut pz) = (beam0.e(), beam0.px(), beam0.py(), beam0.pz());
            for &s in side {
                e -= p[s].e();
                px -= p[s].px();
                py -= p[s].py();
                pz -= p[s].pz();
            }
            e * e - px * px - py * py - pz * pz
        })
        .collect()
}

fn ladder_beams() -> [LorentzVector<f64>; 2] {
    let h = LADDER_SQRT_S / 2.0;
    [
        LorentzVector::new(h, 0.0, 0.0, h),
        LorentzVector::new(h, 0.0, 0.0, -h),
    ]
}

/// The run card's cuts for [`LADDER_PROCESS`], and the fiducial scale they imply.
fn ladder_cuts() -> Cuts {
    let legs = vec![
        ExternalLeg::incoming(2, 0.0),
        ExternalLeg::incoming(1, 0.0),
        ExternalLeg::outgoing(-11, 0.0),
        ExternalLeg::outgoing(11, 0.0),
        ExternalLeg::outgoing(2, 0.0),
        ExternalLeg::outgoing(1, 0.0),
    ];
    Cuts::compile(&RunCard::default(), &legs).expect("the reference process's cuts compile")
}

/// Every diagram of the reference process with the chain it implies.
fn ladder_diagrams(model: &EvaluatedModel) -> Vec<(Diagram, RungChain)> {
    let sets = common::generate(LADDER_PROCESS);
    assert_eq!(sets.len(), 1, "the reference process is one subprocess");
    sets[0]
        .diagrams
        .iter()
        .map(|d| {
            let chain = rung_chain(d, model);
            (d.clone(), chain)
        })
        .collect()
}

/// The chain the ordering test is specified at: two rungs, the lepton pair emitted
/// first and a single jet second, with poles of different masses.
///
/// Selected by shape rather than by index, so a change in enumeration order moves
/// the test rather than silently retargeting it. The asymmetry is the precondition:
/// two rungs carrying the same pole and interchangeable blobs would make the swapped
/// chain the *same* map, and the ordering question would have no content.
fn ordering_chain(model: &EvaluatedModel) -> (Diagram, RungChain) {
    ladder_diagrams(model)
        .into_iter()
        .find(|(_, c)| {
            c.blobs.len() == 2
                && c.blobs[0] == vec![0, 1]
                && c.blobs[1].len() == 1
                && c.poles[0] != c.poles[1]
        })
        .expect("the reference process carries an asymmetric two-rung chain")
}

/// The spacelike lines of every diagram of the reference process nest into an
/// ordered chain *and* that chain is the one an independent graph cut gives.
///
/// The chain is read off the stored momentum routing — feyngraph eliminates the
/// highest-indexed external, so a stored coefficient vector is the signed
/// combination for one side of the cut, and which side that is takes care. Deleting
/// the propagator from the vertex graph and taking beam `0`'s connected component
/// asks the same question of the topology instead. The two agreeing means a routing
/// convention change trips one derivation or the other rather than quietly moving
/// every rung's blob.
#[test]
fn the_rung_chain_agrees_with_an_independent_graph_cut() {
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let mut rungs_seen = std::collections::BTreeMap::<usize, usize>::new();
    let mut compared = 0usize;
    for (d, chain) in ladder_diagrams(&evaluated) {
        *rungs_seen.entry(chain.sides.len()).or_default() += 1;
        let spacelike: Vec<usize> = (0..d.props.len())
            .filter(|&pi| d.props[pi].is_spacelike(d.n_in))
            .collect();
        let mut by_graph: Vec<Vec<usize>> = spacelike
            .iter()
            .map(|&pi| beam0_side_by_graph_cut(&d, pi))
            .collect();
        by_graph.sort_by_key(|s| s.len());
        assert_eq!(
            by_graph, chain.sides,
            "the routed and graph-cut derivations of the rung chain disagree"
        );
        // Blobs and recoil partition the final state, in chain order.
        let mut covered: Vec<usize> = chain.blobs.concat();
        covered.extend(&chain.recoil);
        covered.sort_unstable();
        assert_eq!(
            covered,
            (0..4).collect::<Vec<_>>(),
            "the blobs and recoil do not partition the final state"
        );
        assert!(!chain.recoil.is_empty(), "the chain left nothing behind");
        compared += 1;
    }
    eprintln!("{LADDER_PROCESS}: spacelike-line count over {compared} diagrams: {rungs_seen:?}");
    assert!(
        rungs_seen.contains_key(&3),
        "no three-rung ladder in the reference process, so the cross-check is weak"
    );
}

/// Every chain the reference process implies is a valid map: it consumes exactly its
/// own dimension, emits on-shell momentum-conserving points, and the weight its walk
/// accumulated is the reciprocal of the density rebuilt from the realised momenta.
///
/// The last is the instrument that sees a wrong inter-rung rotation. The walk builds
/// rung `i > 1` in the CM of what the previous rung left behind, with `q_{i-1}` as
/// its polar axis; the density never enters that frame at all, and reads every
/// invariant back off the momenta. A rotation that put a rung's emission at the
/// wrong angle would leave the drawn `t_i` and the reconstructed one disagreeing,
/// which is exactly this comparison.
#[test]
fn every_ladder_chain_is_a_valid_map() {
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let scale = ladder_cuts().spacelike_floor();
    let masses = vec![0.0; 4];
    let mut worst = 0.0f64;
    let mut multi = 0usize;
    for (i, (d, chain)) in ladder_diagrams(&evaluated).into_iter().enumerate() {
        let ch = DiagramChannel::<f64>::from_diagram_ladder(&d, &evaluated, LADDER_SQRT_S, scale);
        assert_eq!(
            ch.spine_poles().len(),
            chain.sides.len(),
            "diagram {i}: the channel built {} rungs for a {}-line ladder",
            ch.spine_poles().len(),
            chain.sides.len()
        );
        if chain.sides.len() > 1 {
            multi += 1;
        }
        worst = worst.max(assert_valid(
            &ch,
            LADDER_SQRT_S,
            &masses,
            0x1ADD0 + i as u64,
        ));
    }
    eprintln!(
        "{LADDER_PROCESS}: walk weight vs 1/density over every derived chain \
         ({multi} of them multi-rung): worst {worst:.3e} relative"
    );
    assert!(multi > 0, "no multi-rung chain was exercised");
    assert!(
        worst < WALK_DENSITY_TOL,
        "a chain's two weightings disagree by {worst:.3e}"
    );
}

/// Importance sampling reshapes variance, not volume: each chain integrates exactly
/// the volume of the region it draws from, and between them the chains reach
/// everywhere the process's own cuts accept.
///
/// The comparison cannot simply be against `V_4`, because a rung bounded at the
/// fiducial scale deliberately *renounces* part of phase space and must report
/// density zero there. So the reference is `V_4` restricted to the chain's own
/// support, measured with flat RAMBO using the chain's density as the indicator:
/// two entirely different parametrisations of the same region, one drawing it and
/// one rejecting into it. A wrong Jacobian moves the first and not the second.
///
/// Both halves are stated as what they are. The volume agreement is a check on the
/// Jacobian and is **blind to the rung ordering** by construction — the multiset
/// `{t_i}` a configuration carries is the same read from either end of the ladder —
/// so it is recorded here so nobody later mistakes it for confirmation that the
/// ordering is right. The coverage half is the other side of a narrowed support: a
/// channel set whose members each renounce a piece is unbiased only if between them
/// they still reach everywhere the integrand lives, and the sharpest available
/// integrand for that is the cut indicator itself.
#[test]
fn ladder_chains_integrate_their_own_support_and_cover_the_fiducial_region() {
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let cuts = ladder_cuts();
    let scale = cuts.spacelike_floor();
    assert_eq!(scale, 400.0, "the reference card's fiducial scale");
    let masses = vec![0.0; 4];
    let n = 200_000;

    let flat = RamboChannel::new(LADDER_SQRT_S, masses.clone());
    let (v_full, _) = mc_estimate(&flat, 0x1ADD_10, 21, n, |_| 1.0);

    let diagrams = ladder_diagrams(&evaluated);
    let channels: Vec<DiagramChannel<f64>> = diagrams
        .iter()
        .map(|(d, _)| {
            DiagramChannel::<f64>::from_diagram_ladder(d, &evaluated, LADDER_SQRT_S, scale)
        })
        .collect();

    let mut checked = 0usize;
    let mut worst_pull = 0.0f64;
    let mut smallest_support = 1.0f64;
    for (i, (_, chain)) in diagrams.iter().enumerate() {
        if chain.sides.len() < 2 {
            continue;
        }
        let ch = &channels[i];
        let (drawn, var_drawn) = mc_estimate(ch, 0x1ADD_20 + i as u64, 23, n, |_| 1.0);
        let (rejected, var_rej) = mc_estimate(&flat, 0x1ADD_40 + i as u64, 27, n, |p| {
            if ch.density(p) > 0.0 {
                1.0
            } else {
                0.0
            }
        });
        let err = ((var_drawn + var_rej) / n as f64).sqrt();
        let pull = (drawn - rejected) / err;
        worst_pull = worst_pull.max(pull.abs());
        smallest_support = smallest_support.min(rejected / v_full);
        eprintln!(
            "  chain {i} (r = {}): drawn {drawn:.6e} vs rejected-into {rejected:.6e} ± {err:.2e}, \
             pull {pull:+.2}; support is {:.4} of V_4",
            chain.sides.len(),
            rejected / v_full
        );
        assert!(
            pull.abs() < 5.0,
            "chain {i} draws a volume {drawn:.6e} its own support does not have \
             ({rejected:.6e} ± {err:.2e})"
        );
        checked += 1;
    }
    assert!(checked > 0, "no multi-rung chain was exercised");
    eprintln!(
        "over {checked} multi-rung chains: worst volume pull {worst_pull:.2}, \
         narrowest support {smallest_support:.4} of V_4 (flat RAMBO V_4 = {v_full:.6e})"
    );
    assert!(
        smallest_support < 0.999,
        "no chain's support is narrowed at all, so the support-restricted comparison \
         is the plain volume check in disguise"
    );

    // Coverage: over the whole channel set, the region no channel reaches must hold
    // nothing the cuts accept. Measured with flat RAMBO on the cut indicator, once
    // over everything the cuts keep and once over what they keep *and* some channel
    // can draw.
    let beams = ladder_beams();
    let pass = |p: &[LorentzVector<f64>]| {
        let mut ext = vec![beams[0], beams[1]];
        ext.extend_from_slice(p);
        cuts.pass(&ext)
    };
    let covered = |p: &[LorentzVector<f64>]| channels.iter().any(|c| c.density(p) > 0.0);
    let (fiducial, var_f) =
        mc_estimate(&flat, 0x1ADD_50, 29, n, |p| if pass(p) { 1.0 } else { 0.0 });
    let (reachable, var_r) = mc_estimate(&flat, 0x1ADD_50, 29, n, |p| {
        if pass(p) && covered(p) {
            1.0
        } else {
            0.0
        }
    });
    // The same draws, so the difference is a strict subset count and its own error
    // is what matters, not the two quoted separately.
    let err = ((var_f + var_r) / n as f64).sqrt();
    eprintln!(
        "fiducial volume {fiducial:.6e}, of it reachable by some chain {reachable:.6e} \
         ({:.6} of it), combined error {err:.2e}",
        reachable / fiducial
    );
    assert_eq!(
        fiducial, reachable,
        "the chain set leaves part of the fiducial region unreachable, so its densities \
         cannot normalise a combiner over it"
    );
    assert!(
        fiducial > 0.0 && fiducial < v_full,
        "the cut indicator is not selecting a proper subregion, so coverage is vacuous"
    );
}

/// Logarithmic bins the per-rung transfer projections are read in, spanning the
/// fiducial window `[|t|_cut, ŝ]` a bounded rung draws over.
const T_ORD_BINS: usize = 12;
/// Draws each coverage and precision figure is measured over.
const T_ORD_DRAWS: usize = 400_000;
/// Least share of the raw draws a bin must hold for the rung it projects to count
/// as importance-sampled.
///
/// A rung drawn against its own propagator is uniform in `ln|t|` within each
/// event's window, so its occupancy is spread across the fiducial window rather
/// than piled at the wide end. Measured on the reference process's asymmetric
/// two-rung chain: the derived ordering holds at least `1.80%` in every bin of both
/// rungs, while the reversed one falls to `0.91%` in rung 1 — and stays at `2.62%`
/// in rung 2, which is the ordering-blindness the mechanism predicts. The bound sits
/// between them with half again as much margin either way.
const T_ORD_MIN_BIN_SHARE: f64 = 0.012;

/// What one map does to the probe integrand: per-rung bin occupancies, per-bin
/// precision, and the seed-swept integral.
struct OrderingReport {
    /// `[rung][bin]` raw draw counts over `ln|t_i|` across the fiducial window.
    occupancy: Vec<[usize; T_ORD_BINS]>,
    /// Worst per-bin relative error of `∫f` restricted to a bin, over all rungs.
    worst_bin_rel_err: f64,
    /// Inverse-variance mean of `∫f` over the seed sweep, with its error, `χ²/dof`
    /// and the worst single-seed pull.
    integral: f64,
    integral_err: f64,
    chi2_per_dof: f64,
    worst_pull: f64,
}

impl OrderingReport {
    fn share(&self, rung: usize, bin: usize) -> f64 {
        self.occupancy[rung][bin] as f64 / T_ORD_DRAWS as f64
    }

    fn thinnest_bin(&self, rung: usize) -> f64 {
        (0..T_ORD_BINS)
            .map(|b| self.share(rung, b))
            .fold(f64::INFINITY, f64::min)
    }

    /// Rungs whose thinnest bin falls below the coverage share — the rungs this map
    /// is not importance-sampling.
    fn starved_rungs(&self) -> Vec<usize> {
        (0..self.occupancy.len())
            .filter(|&i| self.thinnest_bin(i) < T_ORD_MIN_BIN_SHARE)
            .collect()
    }
}

/// Measure a map against a chain's own probe integrand.
///
/// The transfers come from the `S_i` prefixes of the *diagram*, not from anything
/// the map reports, so a map that orders its rungs differently is still judged on
/// the diagram's structure.
fn ordering_report(
    map: &dyn PhaseSpaceMap<f64>,
    chain: &RungChain,
    cuts: &Cuts,
    mz2: f64,
    mg: f64,
    seed: u64,
) -> OrderingReport {
    let beams = ladder_beams();
    let scale = cuts.spacelike_floor();
    let r = chain.sides.len();
    // The probe carries the structures the diagram's matrix element has: the lepton
    // pair's Z line shape and each rung's spacelike propagator. A pole lighter than
    // the fiducial scale is held at it — below that scale the cuts reject anyway,
    // and a bare massless `1/t²` is not integrable against a cut set that lets the
    // two jets balance each other rather than the beam.
    let probe = |p: &[LorentzVector<f64>]| -> f64 {
        let mut ext = vec![beams[0], beams[1]];
        ext.extend_from_slice(p);
        if !cuts.pass(&ext) {
            return 0.0;
        }
        let ts = chain_transfers(chain, beams[0], p);
        let s_ll = s_pair(p, 0, 1);
        let mut f = 1.0 / ((s_ll - mz2).powi(2) + mg * mg);
        for (i, &t) in ts.iter().enumerate() {
            f /= (chain.poles[i].max(scale) - t).powi(2);
        }
        f
    };

    let lo = scale.ln();
    let hi = (LADDER_SQRT_S * LADDER_SQRT_S).ln();
    let bin_of = |t: f64| -> Option<usize> {
        let l = t.abs().ln();
        if !(l >= lo) || !(l < hi) {
            return None;
        }
        Some((((l - lo) / (hi - lo)) * T_ORD_BINS as f64) as usize)
    };

    let mut occupancy = vec![[0usize; T_ORD_BINS]; r];
    let mut bin_sum = vec![[0.0f64; T_ORD_BINS]; r];
    let mut bin_sq = vec![[0.0f64; T_ORD_BINS]; r];
    let mut stream = SubStream::from_stream(seed, 31);
    for _ in 0..T_ORD_DRAWS {
        let u = stream.uniforms::<f64>(map.ndim());
        let pt = map.sample(&u);
        let ts = chain_transfers(chain, beams[0], &pt.momenta);
        let v = pt.weight * probe(&pt.momenta);
        for (i, &t) in ts.iter().enumerate() {
            if let Some(b) = bin_of(t) {
                occupancy[i][b] += 1;
                bin_sum[i][b] += v;
                bin_sq[i][b] += v * v;
            }
        }
    }
    let n = T_ORD_DRAWS as f64;
    let mut worst_bin_rel_err = 0.0f64;
    for i in 0..r {
        for b in 0..T_ORD_BINS {
            let mean = bin_sum[i][b] / n;
            if !(mean > 0.0) {
                worst_bin_rel_err = f64::INFINITY;
                continue;
            }
            let var = (bin_sq[i][b] / n - mean * mean).max(0.0);
            worst_bin_rel_err = worst_bin_rel_err.max((var / n).sqrt() / mean);
        }
    }

    // Seed stability: a map that misses a region reports a small integral *and* a
    // small variance, so only independent seeds landing within the errors they each
    // claim separates a converged estimate from an under-covered one.
    let seeds = 5u64;
    let per_seed = T_ORD_DRAWS / 4;
    let runs: Vec<(f64, f64)> = (0..seeds)
        .map(|k| mc_estimate(map, seed ^ (0x5EED_0000 + k), 33 + k, per_seed, probe))
        .collect();
    let weights: Vec<f64> = runs
        .iter()
        .map(|r| per_seed as f64 / r.1.max(f64::MIN_POSITIVE))
        .collect();
    let wsum: f64 = weights.iter().sum();
    let integral = runs.iter().zip(&weights).map(|(r, w)| r.0 * w).sum::<f64>() / wsum;
    let chi2: f64 = runs
        .iter()
        .zip(&weights)
        .map(|(r, w)| w * (r.0 - integral).powi(2))
        .sum();
    let worst_pull = runs
        .iter()
        .map(|r| (r.0 - integral).abs() / (r.1 / per_seed as f64).sqrt())
        .fold(0.0f64, f64::max);
    OrderingReport {
        occupancy,
        worst_bin_rel_err,
        integral,
        integral_err: (1.0 / wsum).sqrt(),
        chi2_per_dof: chi2 / (seeds as f64 - 1.0),
        worst_pull,
    }
}

fn describe_ordering(label: &str, rep: &OrderingReport) {
    for (i, occ) in rep.occupancy.iter().enumerate() {
        eprintln!(
            "  {label} rung {i}: ln|t| occupancy {occ:?}, thinnest bin {:.4}",
            rep.thinnest_bin(i)
        );
    }
    eprintln!(
        "  {label}: I = {:.5e} ± {:.2e} ({:.1}%), worst per-bin rel err {:.3}, χ²/dof {:.2}, \
         worst seed pull {:.2}, starved rungs {:?}",
        rep.integral,
        rep.integral_err,
        100.0 * rep.integral_err / rep.integral,
        rep.worst_bin_rel_err,
        rep.chi2_per_dof,
        rep.worst_pull,
        rep.starved_rungs()
    );
}

/// The rung ordering is a property of the map, not of a configuration, so no
/// invariant-level check can see a wrong one: `V_n`, `σ`, and any histogram of the
/// *volume* in `t_i` are identical between a chain and its reverse, because the
/// multiset `{t_i}` a configuration carries is the same read from either end of the
/// ladder. What a wrong ordering costs is importance sampling — both orderings
/// integrate `dΦ` correctly, only the right one concentrates its draws where the
/// diagram's propagators peak.
///
/// So this is a coverage test on a peaked integrand, not an agreement test on a
/// volume. On the reference process's asymmetric two-rung chain, at fixed `√ŝ` with
/// the run card's cuts:
///
/// * **per-rung coverage** — the raw draws, binned by `ln|t_i|` over the fiducial
///   window, hold at least [`T_ORD_MIN_BIN_SHARE`] in every bin. A rung that is not
///   being importance-sampled piles its draws at the wide end and thins out toward
///   the fiducial edge.
/// * **per-bin precision** — restricted to each bin the estimator of `∫f` is
///   non-empty, and the integral as a whole holds its relative error under 10%.
/// * **seed stability** — five independent seeds agree within the errors they claim.
///   This is the guard a scalar cannot be: a map that misses a region reports a
///   small integral *and* a small variance, and looks perfectly stable from inside.
///
/// Volume neutrality is the fourth leg and lives in
/// [`ladder_chains_integrate_their_own_support_and_cover_the_fiducial_region`]; it
/// checks the Jacobian, not the ordering, and is named here so nobody mistakes it
/// for confirmation.
///
/// And the control that makes the group mean anything: the same channel with its
/// rungs **reversed** must fail. Note what a swap of a two-rung chain actually
/// changes — `t_2 = (p_a − p_{B_1} − p_{B_2})²` is `p_a` minus every emitted blob
/// and is untouched, and only `t_1` moves, from `(p_a − p_{B_1})²` to
/// `(p_a − p_{B_2})²`, which is not a propagator of the diagram at all. So the
/// firing is expected in rung 1's projection specifically and nowhere else, and both
/// halves of that prediction are asserted rather than only the convenient one.
///
/// What this provably cannot detect. **A symmetric chain**: two rungs carrying the
/// same pole and kinematically interchangeable blobs make the reversed map the
/// *same* map, and the test has no content — which is why the chain is selected for
/// asymmetric blobs and distinct poles, and why
/// [`a_swapped_chain_and_an_anchor_flip_are_different_maps`] measures the density
/// gap rather than assuming it. **Anything a positive density cannot see**: a global
/// phase, an amplitude sign, a colour-flow index. **An error common to both
/// orderings** — a wrong per-rung Jacobian, a wrong anchor beam applied to every
/// rung, a misread blob content — which are volume and reciprocity errors owned by
/// the support-volume and walk-vs-density comparisons, and would leave this test
/// green. **A wrong ordering whose coverage happens to survive**: the test fires on
/// starvation, so at a `√ŝ` where every `t_i` window overlaps heavily a wrong
/// ordering could stay adequate, which is why it is specified at a stated `√ŝ` and
/// cut configuration and why the margin is printed and asserted rather than assumed
/// to be large. **Whether the chain belongs to this diagram**, which is
/// [`the_rung_chain_agrees_with_an_independent_graph_cut`]'s business.
#[test]
fn the_rung_ordering_test_fires_on_a_swapped_chain() {
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let z = model.particle_id("Z").expect("Z in model");
    let (mz, gz) = (evaluated.mass(z), evaluated.width(z));
    let (mz2, mg) = (mz * mz, mz * gz);
    let cuts = ladder_cuts();
    let scale = cuts.spacelike_floor();
    let (d, chain) = ordering_chain(&evaluated);
    eprintln!(
        "ordering chain: blobs {:?}, recoil {:?}, bare poles {:?}",
        chain.blobs, chain.recoil, chain.poles
    );
    assert_eq!(chain.blobs.len(), 2, "the ordering chain has two rungs");

    let build = || DiagramChannel::<f64>::from_diagram_ladder(&d, &evaluated, LADDER_SQRT_S, scale);
    let good = ordering_report(&build(), &chain, &cuts, mz2, mg, 0xC0DE_0DDE);
    describe_ordering("derived", &good);
    let bad = ordering_report(
        &build().with_rung_order(&[1, 0]),
        &chain,
        &cuts,
        mz2,
        mg,
        0xC0DE_0DDE,
    );
    describe_ordering("reversed", &bad);

    // T-ORD-1..3 on the chain the diagram implies.
    assert!(
        good.starved_rungs().is_empty(),
        "the derived chain starves rungs {:?}: thinnest bins {:?}",
        good.starved_rungs(),
        (0..chain.blobs.len())
            .map(|i| good.thinnest_bin(i))
            .collect::<Vec<_>>()
    );
    assert!(
        good.worst_bin_rel_err.is_finite(),
        "a bin of the derived chain's projection holds no estimator at all"
    );
    assert!(
        good.integral_err / good.integral < 0.10,
        "the derived chain's integral carries {:.1}% error",
        100.0 * good.integral_err / good.integral
    );
    assert!(
        good.chi2_per_dof < 4.0 && good.worst_pull < 5.0,
        "the derived chain's seeds scatter by more than they claim: χ²/dof {:.2}, \
         worst pull {:.2}",
        good.chi2_per_dof,
        good.worst_pull
    );

    // NEG-A: the same measurement on the reversed chain has to fire, in rung 1.
    assert_eq!(
        bad.starved_rungs(),
        vec![0],
        "reversing the rungs was expected to starve rung 1's projection and only that \
         one — thinnest bins {:?} against a share of {T_ORD_MIN_BIN_SHARE}",
        (0..chain.blobs.len())
            .map(|i| bad.thinnest_bin(i))
            .collect::<Vec<_>>()
    );
    let margin = T_ORD_MIN_BIN_SHARE / bad.thinnest_bin(0);
    eprintln!(
        "NEG-A: the reversed chain holds {:.4} of its draws in rung 1's thinnest bin \
         against the derived chain's {:.4}, a factor {:.2} below the coverage share",
        bad.thinnest_bin(0),
        good.thinnest_bin(0),
        margin
    );
    assert!(
        margin > 1.2,
        "the reversed chain misses the coverage share by only {margin:.2}, too little \
         to read as a firing rather than as noise"
    );
    // The multiperipheral chain, where a single-rung spine has nothing to say at
    // all. Three rungs admit no single coverage share that separates the two
    // orderings — the last rung's projection is ordering-blind and thin under both —
    // so the statement there is relative: the derived ordering out-populates its
    // reversal in the rung the swap moves.
    let (d3, chain3) = ladder_diagrams(&evaluated)
        .into_iter()
        .find(|(_, c)| c.blobs.len() == 3)
        .expect("the reference process carries a three-rung chain");
    eprintln!(
        "multiperipheral chain: blobs {:?}, recoil {:?}, bare poles {:?}",
        chain3.blobs, chain3.recoil, chain3.poles
    );
    let build3 =
        || DiagramChannel::<f64>::from_diagram_ladder(&d3, &evaluated, LADDER_SQRT_S, scale);
    let good3 = ordering_report(&build3(), &chain3, &cuts, mz2, mg, 0xC0DE_0DD3);
    describe_ordering("derived-3", &good3);
    let bad3 = ordering_report(
        &build3().with_rung_order(&[2, 1, 0]),
        &chain3,
        &cuts,
        mz2,
        mg,
        0xC0DE_0DD3,
    );
    describe_ordering("reversed-3", &bad3);
    for rung in [0usize, 1] {
        let ratio = good3.thinnest_bin(rung) / bad3.thinnest_bin(rung);
        eprintln!(
            "  multiperipheral rung {rung}: derived {:.4} vs reversed {:.4}, ratio {ratio:.2}",
            good3.thinnest_bin(rung),
            bad3.thinnest_bin(rung)
        );
        assert!(
            ratio > 2.0,
            "reversing a three-rung chain left rung {rung}'s coverage within a factor \
             {ratio:.2} of the derived ordering's"
        );
    }
    assert!(
        (good3.thinnest_bin(2) - bad3.thinnest_bin(2)).abs() < 1e-12,
        "reversing a three-rung chain moved the last rung's projection, which is `p_a` \
         minus every emitted blob and cannot depend on their order"
    );
}

/// The chain read from the other beam: the same ladder, anchored at beam `1`.
///
/// Complementing every side and re-sorting turns `S_1 ⊂ … ⊂ S_r` into
/// `full\S_r ⊂ … ⊂ full\S_1`, so the blobs come out reversed with the old recoil
/// leading and the old first blob left over. It is a *different map* — a different
/// blob is the recoil, and each rung's transfer is measured against the other beam —
/// not a relabelling of the same one.
fn anchor_flipped(
    chain: &RungChain,
    derived: &DiagramChannel<f64>,
    scale: f64,
) -> DiagramChannel<f64> {
    let composite: Vec<&Vec<usize>> = chain
        .blobs
        .iter()
        .chain(std::iter::once(&chain.recoil))
        .filter(|b| b.len() > 1)
        .collect();
    assert!(
        composite.len() <= 1,
        "the flip reads its one composite blob's pole off the channel, so a chain with \
         more than one would need the resonances matched up by mask"
    );
    let pole = derived.resonances().first().copied();
    let res_of = |slots: &[usize]| if slots.len() > 1 { pole } else { None };
    let mut rungs: Vec<RungSpec<f64>> = Vec::with_capacity(chain.blobs.len());
    rungs.push((
        chain.recoil.clone(),
        res_of(&chain.recoil),
        chain.poles[chain.poles.len() - 1]
            .max(scale / 1000.0)
            .sqrt(),
    ));
    for i in (1..chain.blobs.len()).rev() {
        rungs.push((
            chain.blobs[i].clone(),
            res_of(&chain.blobs[i]),
            chain.poles[i - 1].max(scale / 1000.0).sqrt(),
        ));
    }
    DiagramChannel::from_topology_ladder(
        LADDER_SQRT_S,
        [0.0, 0.0],
        vec![0.0; 4],
        &rungs,
        (chain.blobs[0].clone(), res_of(&chain.blobs[0])),
    )
    .with_fiducial_t_max(-scale)
}

/// The precondition the ordering test rests on, and the convention it pins.
///
/// **NEG-C, the precondition.** If a chain and its reversal assigned the same
/// density to the same configuration, the ordering question would be moot *and* the
/// coverage test would have no content. That is not hypothetical — permutation
/// channels of `g g > g g` were once found collapsing onto a common map at spacelike
/// floor zero — so the gap is measured, at the regulator a real run gives the
/// channels rather than at zero, for exactly that reason.
///
/// **NEG-B, the anchor.** Anchoring at beam `1` reads the same ladder from the other
/// end. It is a different map, not a relabelling: a different blob becomes the
/// recoil and every transfer is measured against the other beam. Asserting that its
/// density differs pins that beam `0` is a *choice* the chain derivation makes, so a
/// derivation that quietly anchored at the other beam would not pass unnoticed.
#[test]
fn a_swapped_chain_and_an_anchor_flip_are_different_maps() {
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let scale = ladder_cuts().spacelike_floor();
    let n = 400;
    let mut stream = SubStream::from_stream(0x1ADD_C0, 41);
    let mut checked = 0usize;
    let mut worst_swap_agreement = 1.0f64;
    let mut worst_flip_agreement = 1.0f64;
    for (i, (d, chain)) in ladder_diagrams(&evaluated).into_iter().enumerate() {
        let r = chain.blobs.len();
        if r < 2 {
            continue;
        }
        let derived =
            DiagramChannel::<f64>::from_diagram_ladder(&d, &evaluated, LADDER_SQRT_S, scale);
        let reversed =
            DiagramChannel::<f64>::from_diagram_ladder(&d, &evaluated, LADDER_SQRT_S, scale)
                .with_rung_order(&(0..r).rev().collect::<Vec<_>>());
        let flipped = anchor_flipped(&chain, &derived, scale);

        let (mut swap_apart, mut flip_apart, mut drawn) = (0usize, 0usize, 0usize);
        for _ in 0..n {
            let u = stream.uniforms::<f64>(derived.ndim());
            let pt = derived.sample(&u);
            let g = derived.density(&pt.momenta);
            let gap = |other: f64| (g - other).abs() > 1e-6 * g.max(other);
            if gap(reversed.density(&pt.momenta)) {
                swap_apart += 1;
            }
            if gap(flipped.density(&pt.momenta)) {
                flip_apart += 1;
            }
            drawn += 1;
        }
        let swap_share = swap_apart as f64 / drawn as f64;
        let flip_share = flip_apart as f64 / drawn as f64;
        worst_swap_agreement = worst_swap_agreement.min(swap_share);
        worst_flip_agreement = worst_flip_agreement.min(flip_share);
        assert!(
            swap_share > 0.5,
            "chain {i}: the reversed chain's density matches the derived one's to rounding \
             on {:.0}% of its own points, so the two are the same map and the ordering \
             test has no content there",
            100.0 * (1.0 - swap_share)
        );
        assert!(
            flip_share > 0.5,
            "chain {i}: the beam-1-anchored chain's density matches the beam-0 one's to \
             rounding on {:.0}% of its points, so the anchor is not a choice the \
             derivation is making",
            100.0 * (1.0 - flip_share)
        );
        checked += 1;
    }
    eprintln!(
        "over {checked} multi-rung chains: the reversed chain's density differs on at least \
         {:.3} of drawn points, the anchor-flipped one on at least {:.3}",
        worst_swap_agreement, worst_flip_agreement
    );
    assert!(checked > 0, "no multi-rung chain was exercised");
}

/// The foreign-configuration density contract, at a rung chain.
///
/// A combiner weights every point by `αⱼ / Σₖ αₖ gₖ` gathered from *every* channel
/// at the *same* configuration, so a chain has to report the density it assigns to
/// an arbitrary on-shell momentum-conserving point, not only to points it generated.
/// A chain makes a new way to get that wrong available, because it has an ordering
/// and a configuration does not: a point whose `t_2` is smaller than its `t_1`, or
/// whose blobs sit nowhere near this chain's peaks, is a perfectly ordinary point
/// that another channel drew. **The rung order is a property of the map, not a
/// constraint on the configuration** — a chain that refused such a point, or returned
/// zero or `NaN` at it, would bias every other channel's estimate through the shared
/// `Σₖ αₖ gₖ`.
///
/// Checked here over points drawn from every channel of the reference process, from
/// the reversed chains, and from flat RAMBO:
///
/// * **totality** — every density is finite and non-negative, at every configuration,
///   including the ones ordered against the chain's own peaks.
/// * **positivity where the chain has support** — a configuration whose every running
///   transfer clears the fiducial scale is inside the chain's windows by construction,
///   and gets a strictly positive density.
/// * **support honesty** — outside a deliberately narrowed window the density is
///   *exactly* zero and never a small positive number, since the estimator is
///   unbiased only when each `gⱼ` is the true pushforward density of channel `j`
///   everywhere.
/// * **degeneracy** — a threshold or lightlike configuration returns zero rather than
///   `NaN`, which the flat-RAMBO and near-threshold draws reach.
///
/// Reciprocity, the contract's first clause, is
/// [`every_ladder_chain_is_a_valid_map`]; frame independence is
/// [`the_chain_density_reads_only_invariants`].
#[test]
fn the_chain_density_contract_holds_at_foreign_configurations() {
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let scale = ladder_cuts().spacelike_floor();
    let beams = ladder_beams();
    let diagrams = ladder_diagrams(&evaluated);
    let channels: Vec<DiagramChannel<f64>> = diagrams
        .iter()
        .map(|(d, _)| {
            DiagramChannel::<f64>::from_diagram_ladder(d, &evaluated, LADDER_SQRT_S, scale)
        })
        .collect();

    // Sources: every channel of the process, every reversed chain, and flat RAMBO —
    // the last reaching the threshold and near-collinear corners no peaked map dwells
    // on.
    let mut sources: Vec<Box<dyn PhaseSpaceMap<f64>>> = Vec::new();
    for (i, (_, chain)) in diagrams.iter().enumerate() {
        sources.push(Box::new(channels[i].clone()));
        if chain.blobs.len() > 1 {
            sources.push(Box::new(
                channels[i]
                    .clone()
                    .with_rung_order(&(0..chain.blobs.len()).rev().collect::<Vec<_>>()),
            ));
        }
    }
    sources.push(Box::new(RamboChannel::new(LADDER_SQRT_S, vec![0.0; 4])));

    let mut stream = SubStream::from_stream(0x1ADD_F0, 43);
    let mut evaluated_at = 0usize;
    let mut zeros = 0usize;
    let mut against_the_ordering = 0usize;
    let mut inside_support = 0usize;
    for src in &sources {
        for _ in 0..120 {
            let u = stream.uniforms::<f64>(src.ndim());
            let p = src.sample(&u).momenta;
            for (i, (_, chain)) in diagrams.iter().enumerate() {
                let g = channels[i].density(&p);
                assert!(
                    g.is_finite() && g >= 0.0,
                    "chain {i} reported density {g} at a configuration it did not generate"
                );
                evaluated_at += 1;
                let ts = chain_transfers(chain, beams[0], &p);
                // A configuration whose transfers run the other way from the chain's
                // own peaks is an ordinary point another channel drew, not one this
                // chain may refuse.
                if ts.len() > 1 && ts[1].abs() < ts[0].abs() {
                    against_the_ordering += 1;
                    assert!(
                        g.is_finite() && g >= 0.0,
                        "chain {i} refused a configuration ordered against its own rungs"
                    );
                }
                if ts.iter().all(|&t| t <= -scale) {
                    inside_support += 1;
                    assert!(
                        g > 0.0,
                        "chain {i} reported density {g} at a configuration inside its own \
                         windows: transfers {ts:?}"
                    );
                }
                if g == 0.0 {
                    zeros += 1;
                    assert!(
                        ts.iter().any(|&t| t > -scale),
                        "chain {i} renounced a configuration every one of whose transfers \
                         clears the fiducial scale: {ts:?}"
                    );
                }
            }
        }
    }
    eprintln!(
        "density contract over {evaluated_at} chain × configuration pairs from {} sources: \
         {zeros} exact zeros (all above the fiducial bound), {inside_support} inside the \
         chain's own windows, {against_the_ordering} ordered against the chain's rungs",
        sources.len()
    );
    assert!(
        against_the_ordering > 0,
        "no configuration ran against a chain's own ordering, so the clause that matters \
         most for a chain is untested"
    );
    assert!(
        zeros > 0,
        "no configuration fell outside a narrowed window, so support honesty is untested"
    );
    assert!(
        inside_support > 0,
        "no configuration landed inside a chain's own windows, so positivity is untested"
    );
}

/// The chain's density reads invariants only, so no frame the sampler worked in
/// leaks into it.
///
/// The sampler builds rung `i > 1` in the CM of what the previous rung left behind,
/// with `q_{i-1}` as its polar axis — a transverse frame that exists nowhere in the
/// configuration. Rotating an event about the beam axis leaves every invariant the
/// density reads untouched (`t_i`, the blob invariants, the running remainders) while
/// moving every momentum, so the density must come back identical. Rotating about a
/// transverse axis instead moves the configuration relative to the *beams*, which are
/// part of the channel's data, and must move the density — otherwise the first check
/// would pass for a density that reads nothing at all.
#[test]
fn the_chain_density_reads_only_invariants() {
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let scale = ladder_cuts().spacelike_floor();
    let spin_z = |p: LorentzVector<f64>, a: f64| {
        LorentzVector::new(
            p.e(),
            p.px() * a.cos() - p.py() * a.sin(),
            p.px() * a.sin() + p.py() * a.cos(),
            p.pz(),
        )
    };
    let tilt_x = |p: LorentzVector<f64>, a: f64| {
        LorentzVector::new(
            p.e(),
            p.px(),
            p.py() * a.cos() - p.pz() * a.sin(),
            p.py() * a.sin() + p.pz() * a.cos(),
        )
    };
    let mut stream = SubStream::from_stream(0x1ADD_A0, 45);
    let mut worst = 0.0f64;
    let mut moved = 0usize;
    let mut checked = 0usize;
    for (i, (d, chain)) in ladder_diagrams(&evaluated).into_iter().enumerate() {
        if chain.blobs.len() < 2 {
            continue;
        }
        let ch = DiagramChannel::<f64>::from_diagram_ladder(&d, &evaluated, LADDER_SQRT_S, scale);
        for k in 0..60 {
            let u = stream.uniforms::<f64>(ch.ndim());
            let p = ch.sample(&u).momenta;
            let g = ch.density(&p);
            if !(g > 0.0) {
                continue;
            }
            let angle = 0.3 + 0.1 * k as f64;
            let spun: Vec<LorentzVector<f64>> = p.iter().map(|q| spin_z(*q, angle)).collect();
            let rel = (ch.density(&spun) - g).abs() / g;
            worst = worst.max(rel);
            assert!(
                rel < 1e-9,
                "chain {i}: rotating an event about the beam axis moved its density by \
                 {rel:.3e}, so the density is reading a frame rather than invariants"
            );
            let tilted: Vec<LorentzVector<f64>> = p.iter().map(|q| tilt_x(*q, angle)).collect();
            let tilted_g = ch.density(&tilted);
            if (tilted_g - g).abs() > 1e-6 * g.max(tilted_g) {
                moved += 1;
            }
            checked += 1;
        }
    }
    eprintln!(
        "azimuthal invariance over {checked} configurations: worst relative change \
         {worst:.3e}; a transverse tilt moved the density on {moved} of them"
    );
    assert!(checked > 0, "no multi-rung chain was exercised");
    assert!(
        moved as f64 > 0.9 * checked as f64,
        "a transverse tilt left the density where it was on {} of {checked} configurations, \
         so the azimuthal check above could be passing for a density that reads nothing",
        checked - moved
    );
}

/// What the chain is worth, measured beside the map production actually uses.
///
/// A ladder diagram still falls back to the all-timelike tree in production, so this
/// runs the two side by side on the diagram's own peaked structure — the lepton
/// pair's Z line shape and every rung's spacelike propagator, under the run card's
/// cuts — over independent seeds.
///
/// A seed sweep and not a single run, because a single run cannot tell a converged
/// estimate from an under-covered one: a map that misses the peripheral region
/// reports a small integral *and* a small variance and looks perfectly stable from
/// the inside. That is exactly what the all-timelike map does here, so the assertion
/// is emphatically **not** that the two agree. It is that the chain is
/// self-consistent across seeds and never the noisier of the two, and that the
/// fallback still visibly fails to keep up — a comparison whose known-wrong arm
/// stopped being wrong would have lost its subject, and the day the chain goes into
/// production this is the number that has to move.
#[test]
fn the_ladder_chain_beside_the_all_timelike_fallback() {
    let model = common::sm_model();
    let evaluated = EvaluatedModel::from_model(model.clone());
    let z = model.particle_id("Z").expect("Z in model");
    let (mz, gz) = (evaluated.mass(z), evaluated.width(z));
    let (mz2, mg) = (mz * mz, mz * gz);
    let cuts = ladder_cuts();
    let scale = cuts.spacelike_floor();
    let beams = ladder_beams();
    let seeds = 5u64;
    let n = 100_000;

    let mut compared = 0usize;
    let mut disagreements = 0usize;
    let mut worst_ratio = 0.0f64;
    let mut best_ratio = f64::INFINITY;
    for (i, (d, chain)) in ladder_diagrams(&evaluated).into_iter().enumerate() {
        if chain.blobs.len() < 2 {
            continue;
        }
        let probe = |p: &[LorentzVector<f64>]| -> f64 {
            let mut ext = vec![beams[0], beams[1]];
            ext.extend_from_slice(p);
            if !cuts.pass(&ext) {
                return 0.0;
            }
            let ts = chain_transfers(&chain, beams[0], p);
            let s_ll = s_pair(p, 0, 1);
            let mut f = 1.0 / ((s_ll - mz2).powi(2) + mg * mg);
            for (k, &t) in ts.iter().enumerate() {
                f /= (chain.poles[k].max(scale) - t).powi(2);
            }
            f
        };
        // What production builds for a ladder today, and what the chain builds for it.
        let fallback =
            DiagramChannel::<f64>::from_diagram_regulated(&d, &evaluated, LADDER_SQRT_S, scale);
        assert!(
            fallback.spine_poles().is_empty(),
            "diagram {i} is a ladder, so the regulated derivation should leave it \
             all-timelike"
        );
        let chained =
            DiagramChannel::<f64>::from_diagram_ladder(&d, &evaluated, LADDER_SQRT_S, scale);

        let sweep = |map: &dyn PhaseSpaceMap<f64>, base: u64| -> (f64, f64, f64, f64) {
            let runs: Vec<(f64, f64)> = (0..seeds)
                .map(|k| mc_estimate(map, base + 64 * k + i as u64, 51 + k, n, probe))
                .collect();
            let w: Vec<f64> = runs
                .iter()
                .map(|r| n as f64 / r.1.max(f64::MIN_POSITIVE))
                .collect();
            let wsum: f64 = w.iter().sum();
            let mean = runs.iter().zip(&w).map(|(r, x)| r.0 * x).sum::<f64>() / wsum;
            let err = (1.0 / wsum).sqrt();
            let chi2 = runs
                .iter()
                .zip(&w)
                .map(|(r, x)| x * (r.0 - mean).powi(2))
                .sum::<f64>()
                / (seeds as f64 - 1.0);
            let var = runs.iter().map(|r| r.1).sum::<f64>() / seeds as f64;
            (mean, err, chi2, var)
        };
        let (i_time, e_time, chi_time, var_time) = sweep(&fallback, 0x1ADD_B0);
        let (i_chain, e_chain, chi_chain, var_chain) = sweep(&chained, 0x1ADD_B0);
        let ratio = var_time / var_chain;
        worst_ratio = worst_ratio.max(ratio);
        best_ratio = best_ratio.min(ratio);
        let pull = (i_chain - i_time) / (e_chain * e_chain + e_time * e_time).sqrt();
        eprintln!(
            "  chain {i} (r = {}): all-timelike {i_time:.4e} ± {e_time:.1e} (χ²/dof \
             {chi_time:.2}) vs chain {i_chain:.4e} ± {e_chain:.1e} (χ²/dof {chi_chain:.2}), \
             pull {pull:+.2}, variance ratio {ratio:.2}×",
            chain.blobs.len()
        );
        assert!(
            chi_chain < 4.0,
            "chain {i}'s seeds scatter by more than they claim: χ²/dof {chi_chain:.2}"
        );
        assert!(
            ratio > 1.0,
            "chain {i} carries {:.2}× the per-point variance of the all-timelike map it \
             is meant to replace",
            1.0 / ratio
        );
        if pull.abs() > 3.0 {
            disagreements += 1;
        }
        compared += 1;
    }
    eprintln!(
        "over {compared} ladder diagrams: the chain's per-point variance is between \
         {best_ratio:.2}× and {worst_ratio:.2}× below the all-timelike fallback's, and the \
         two integrals disagree by more than 3σ on {disagreements} of them"
    );
    assert!(compared > 0, "no ladder diagram was compared");
    assert!(
        disagreements > 0,
        "the all-timelike fallback now agrees with the chain everywhere, so this \
         comparison has stopped carrying a signal about what production is missing"
    );
}
