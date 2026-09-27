//! Multi-gluon amplitudes against the Parke–Taylor formula, colour flow by colour
//! flow.
//!
//! At tree level the n-gluon amplitude decomposes exactly into single traces,
//! `M = Σ_σ Tr(T^{σ1}…T^{σn}) A(σ)`, and for the maximally helicity-violating
//! configurations every colour-ordered partial amplitude is known in closed form:
//! with outgoing helicities `i⁻ j⁻` and every other gluon `+`,
//! `A(σ) = c · ⟨ij⟩⁴ / (⟨σ1σ2⟩⟨σ2σ3⟩…⟨σnσ1⟩)`, and its parity image with square
//! brackets when exactly two gluons are `+`. The configurations with fewer than
//! two gluons of either helicity vanish.
//!
//! Each compiled colour flow is one trace, read off the flow's own colour-line
//! tags, so this compares the evaluator's per-flow amplitudes to the formula
//! without MadGraph. The test is written to be blind to the conventions the
//! formula does not fix, and to nothing else it can see:
//!
//! * For one point and one helicity configuration, `J_σ · ⟨σ1σ2⟩…⟨σnσ1⟩` must be
//!   the same complex number for every flow (compared as `J_σ` against that fitted
//!   constant over the cyclic product, relative to the largest flow of any
//!   configuration at the point: a configuration suppressed by a soft gluon of the
//!   wrong helicity carries the rounding of the point's scale, not of its own). A spinor's phase and scale enter
//!   every cyclic product twice, so this ratio is independent of how the external
//!   polarisation vectors and the spinors are phased (and of the spinors' scale);
//!   the overall phase of the configuration is not compared.
//! * `|J_σ| · |⟨σ1σ2⟩…⟨σnσ1⟩| / |⟨ij⟩|⁴` must be one number over every MHV
//!   configuration at every point: the relative normalisation of helicity
//!   configurations and of kinematic points (its deviation weighted by the
//!   configuration's share of the point's largest flow, for the same reason).
//!
//! What it cannot see: a global complex constant (the couplings, `i`, the trace
//! normalisation); a per-configuration phase; any configuration with three or
//! more gluons of each helicity (six gluons' NMHV amplitudes have no one-line
//! formula and are not compared); and an error proportional to the Parke–Taylor
//! amplitude in every flow at once. A wrong sign on the four-gluon contact is not
//! such an error: the contact diagrams enter different traces with different
//! weights, so it moves the ratio between flows.
//!
//! Needs nothing but the interned SM model, so it runs on a bare clone.

mod common;

use common::{generate_with, sm_model};
use rand::SeedableRng;
use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
use vibegraph::helas::repr::C;
use vibegraph::helas::LorentzVector;
use vibegraph::phasespace::rambo_massless;
use vibegraph::ufo::EvaluatedModel;

const SQRT_S: f64 = 500.0;

/// Relative agreement required between flows (and between configurations for
/// the normalisation). The evaluator's own rounding over a few hundred diagrams
/// sits near 1e-13; a defect in one diagram class moves it by O(1).
const REL_TOL: f64 = 1e-10;

/// `v` rescaled to Euclidean norm `√(2|k⁰|)`, which makes `|⟨ij⟩|² = |s_ij|` for the
/// spinors below whatever their phase.
fn normalised(v: [C<f64>; 2], k: [f64; 4]) -> [C<f64>; 2] {
    let n = (v[0].norm_sqr() + v[1].norm_sqr()).sqrt();
    let s = (2.0 * k[0].abs()).sqrt() / n;
    [v[0] * s, v[1] * s]
}

/// The holomorphic spinor `λ` of a null momentum, up to phase: a non-vanishing
/// column of `p·σ = [[p⁰+p³, p¹−ip²], [p¹+ip², p⁰−p³]]`.
fn angle_spinor(k: [f64; 4]) -> [C<f64>; 2] {
    let plus = k[0] + k[3];
    let minus = k[0] - k[3];
    let col1 = [C::new(plus, 0.0), C::new(k[1], k[2])];
    let col2 = [C::new(k[1], -k[2]), C::new(minus, 0.0)];
    normalised(
        if plus.abs() >= minus.abs() {
            col1
        } else {
            col2
        },
        k,
    )
}

/// The antiholomorphic spinor `λ̃`, up to phase: a non-vanishing row of `p·σ`.
fn square_spinor(k: [f64; 4]) -> [C<f64>; 2] {
    let plus = k[0] + k[3];
    let minus = k[0] - k[3];
    let row1 = [C::new(plus, 0.0), C::new(k[1], -k[2])];
    let row2 = [C::new(k[1], k[2]), C::new(minus, 0.0)];
    normalised(
        if plus.abs() >= minus.abs() {
            row1
        } else {
            row2
        },
        k,
    )
}

fn bracket(a: [C<f64>; 2], b: [C<f64>; 2]) -> C<f64> {
    a[0] * b[1] - a[1] * b[0]
}

/// The cyclic order of the single trace a flow carries, from its `(colour,
/// anticolour)` tags: in the all-outgoing picture (an incoming leg's two slots
/// exchanged) each gluon's anticolour line is the next gluon's colour line.
fn trace_order(tags: &[[u32; 2]], n_in: usize) -> Vec<usize> {
    let out: Vec<[u32; 2]> = tags
        .iter()
        .enumerate()
        .map(|(i, t)| if i < n_in { [t[1], t[0]] } else { *t })
        .collect();
    let mut order = vec![0usize];
    while order.len() < out.len() {
        let last = out[*order.last().unwrap()];
        let next = (0..out.len())
            .find(|&j| out[j][0] == last[1])
            .expect("every anticolour line ends on a colour slot");
        assert!(
            !order.contains(&next),
            "flow {tags:?} is not a single trace"
        );
        order.push(next);
    }
    assert_eq!(
        out[*order.last().unwrap()][1],
        out[0][0],
        "flow {tags:?} does not close into one trace"
    );
    order
}

struct Summary {
    mhv: usize,
    vanishing: usize,
    worst_flow: f64,
    worst_norm: f64,
}

fn check_gluons(n: usize, n_points: usize, seed: u64) -> Summary {
    let model = sm_model();
    let process = format!("g g > {}", vec!["g"; n - 2].join(" "));
    let set = generate_with(&process, model.as_ref()).remove(0);
    let evaluated = EvaluatedModel::from_model(model.clone());
    let evaluator = AmplitudeEvaluator::compile(&set, model.as_ref()).unwrap();
    let bound = BoundAmplitude::<f64>::bind(&evaluator, &evaluated);
    let mut scratch = bound.scratch_space();

    let tags = evaluator.color_flow_tags();
    let orders: Vec<Vec<usize>> = (0..evaluator.n_flows())
        .map(|f| trace_order(tags.flow(f), 2))
        .collect();
    let expected_flows: usize = (1..n).product();
    assert_eq!(
        orders.len(),
        expected_flows,
        "{process}: (n−1)! single traces"
    );

    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
    let mut norm: Option<f64> = None;
    let mut summary = Summary {
        mhv: 0,
        vanishing: 0,
        worst_flow: 0.0,
        worst_norm: 0.0,
    };
    for point in 0..n_points {
        let e = SQRT_S / 2.0;
        let mut momenta = vec![
            LorentzVector::new(e, 0.0, 0.0, e),
            LorentzVector::new(e, 0.0, 0.0, -e),
        ];
        momenta.extend(rambo_massless(SQRT_S, n - 2, &mut rng));
        // All outgoing: the incoming momenta and helicities are crossed.
        let k: Vec<[f64; 4]> = momenta
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let a = [p.e(), p.px(), p.py(), p.pz()];
                if i < 2 {
                    a.map(|x| -x)
                } else {
                    a
                }
            })
            .collect();
        let angle: Vec<_> = k.iter().map(|&p| angle_spinor(p)).collect();
        let square: Vec<_> = k.iter().map(|&p| square_spinor(p)).collect();

        let mut scale = 0.0f64;
        let mut per_config = Vec::new();
        for hel in evaluator.helicities() {
            let flows = bound.run_flows(&momenta, hel, &mut scratch);
            scale = scale.max(flows.iter().map(|z| z.norm()).fold(0.0, f64::max));
            per_config.push((hel.clone(), flows));
        }
        for (hel, flows) in per_config {
            let out: Vec<i32> = hel
                .iter()
                .enumerate()
                .map(|(i, &h)| if i < 2 { -h } else { h })
                .collect();
            let minus: Vec<usize> = (0..n).filter(|&i| out[i] < 0).collect();
            let plus: Vec<usize> = (0..n).filter(|&i| out[i] > 0).collect();
            let (spinors, pair) = if minus.len() == 2 {
                (&angle, [minus[0], minus[1]])
            } else if plus.len() == 2 {
                (&square, [plus[0], plus[1]])
            } else {
                if minus.len() < 2 || plus.len() < 2 {
                    summary.vanishing += 1;
                    let worst = flows.iter().map(|z| z.norm()).fold(0.0, f64::max) / scale;
                    assert!(
                        worst < REL_TOL,
                        "{process}: helicities {hel:?} must vanish, a flow is {worst:.3e} of \
                         the largest"
                    );
                }
                continue;
            };
            summary.mhv += 1;
            let numerator = bracket(spinors[pair[0]], spinors[pair[1]]).norm().powi(4);
            // Parke–Taylor shape of each flow, 1/(⟨σ1σ2⟩…⟨σnσ1⟩), and the one constant
            // that best maps it onto the flows.
            let shapes: Vec<C<f64>> = orders
                .iter()
                .map(|order| {
                    let cyclic = (0..n)
                        .map(|a| bracket(spinors[order[a]], spinors[order[(a + 1) % n]]))
                        .fold(C::new(1.0, 0.0), |acc, b| acc * b);
                    C::new(1.0, 0.0) / cyclic
                })
                .collect();
            let (num, den) = shapes
                .iter()
                .zip(&flows)
                .fold((C::new(0.0, 0.0), 0.0), |(n_, d_), (y, j)| {
                    (n_ + y.conj() * j, d_ + y.norm_sqr())
                });
            let reference = num / den;
            for (f, (y, j)) in shapes.iter().zip(&flows).enumerate() {
                let dev = (j - reference * y).norm() / scale;
                summary.worst_flow = summary.worst_flow.max(dev);
                assert!(
                    dev < REL_TOL,
                    "{process}: point {point}, helicities {hel:?}, flow {f} (trace {:?}) departs from \
                     Parke–Taylor by {dev:.3e} of the point's largest flow",
                    orders[f]
                );
            }
            let this_norm = reference.norm() / numerator;
            let n0 = *norm.get_or_insert(this_norm);
            // In units of the point's largest flow, as the flow comparison above.
            let largest = flows.iter().map(|z| z.norm()).fold(0.0, f64::max);
            let dev = (this_norm - n0).abs() / n0 * largest / scale;
            summary.worst_norm = summary.worst_norm.max(dev);
            assert!(
                dev < REL_TOL,
                "{process}: helicities {hel:?}: |A|·|cyclic|/|⟨ij⟩|⁴ = {this_norm:e} against \
                 {n0:e} at the first MHV configuration ({dev:.3e})"
            );
        }
    }
    summary
}

#[test]
fn five_gluon_flows_are_parke_taylor() {
    let s = check_gluons(5, 4, 5);
    // 20 MHV and anti-MHV configurations and 12 vanishing ones per point.
    assert_eq!((s.mhv, s.vanishing), (4 * 20, 4 * 12));
    println!(
        "g g > g g g: {} MHV configurations, worst flow {:.3e}, worst normalisation {:.3e}",
        s.mhv, s.worst_flow, s.worst_norm
    );
}

#[test]
fn six_gluon_flows_are_parke_taylor() {
    let s = check_gluons(6, 2, 6);
    // 15 MHV + 15 anti-MHV, and 1 + 6 + 6 + 1 vanishing, per point.
    assert_eq!((s.mhv, s.vanishing), (2 * 30, 2 * 14));
    println!(
        "g g > g g g g: {} MHV configurations, worst flow {:.3e}, worst normalisation {:.3e}",
        s.mhv, s.worst_flow, s.worst_norm
    );
}
