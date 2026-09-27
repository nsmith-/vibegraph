//! The per-event reweighting plan and its evaluation.
//!
//! [`ReweightPlan`] owns one compiled amplitude per subprocess and decides, per
//! subprocess, which hypotheses take the polynomial path and which the exact path
//! (see the [module docs](super)). [`Reweighter`] is the plan bound to numeric
//! constant pools: mutable, single-threaded state whose
//! [`ratios`](Reweighter::ratios) is the per-event inner loop.
//!
//! # Why the plan compiles its own amplitudes
//!
//! A generation's amplitudes drop the helicity combinations and diagram
//! contributions that vanish at the card's parameters. A hypothesis that switches
//! on a coupling the card leaves at zero revives exactly those, so the plan compiles
//! each subprocess again and prunes it at a *generic* parameter point instead —
//! every parameter any hypothesis moves set to an unremarkable value at once. A
//! contribution that vanishes there is a function of the parameters that vanishes
//! at a generic point, which is one that vanishes identically; everything any
//! hypothesis can reach survives.

use std::collections::BTreeMap;

use crate::diagrams::diagram::OnShell;
use crate::diagrams::DiagramSet;
use crate::helas::eval::{AmplitudeEvaluator, ScaleAwareAmplitude, ScratchSpace};
use crate::helas::repr::lorentz::LorentzVector;
use crate::ufo::{EvaluatedModel, UFOModel};

use super::poly::{squared, PolyAnalysis};
use super::{Launch, ReweightError};

/// How the plan may evaluate hypotheses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReweightOptions {
    /// Allow the polynomial path. Off, every hypothesis takes the exact path.
    pub polynomial: bool,
}

impl Default for ReweightOptions {
    fn default() -> Self {
        ReweightOptions { polynomial: true }
    }
}

/// Where one interpolation node's `|M|²` comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NodeSource {
    /// The node is the card's own value, whose `|M|²` every event evaluates anyway.
    Base,
    /// A bound amplitude of its own.
    Slot(usize),
}

/// The hypotheses of one subprocess that move the same single parameter, served
/// by one polynomial.
#[derive(Clone, Debug)]
struct PolynomialGroup {
    param: String,
    /// Degree of `|M|²` in the parameter.
    degree: u32,
    nodes: Vec<NodeSource>,
    /// Per hypothesis: its launch index and its Lagrange weights over `nodes`.
    launches: Vec<(usize, Box<[f64]>)>,
}

/// One subprocess's evaluation plan.
struct SubPlan {
    evaluator: AmplitudeEvaluator,
    /// Parameter points to bind the amplitude at; slot 0 is the card's own.
    slots: Vec<EvaluatedModel>,
    groups: Vec<PolynomialGroup>,
    /// Hypotheses on the exact path: launch index and slot.
    exact: Vec<(usize, usize)>,
}

/// How one subprocess's hypotheses are evaluated, for a run's log.
#[derive(Clone, Debug, PartialEq)]
pub struct SubprocessSummary {
    /// Per polynomial group: the parameter, the degree of `|M|²` in it, the
    /// amplitude evaluations it costs per event beyond the card's own, and the
    /// hypotheses it serves.
    pub polynomial: Vec<(String, u32, usize, usize)>,
    /// Hypotheses on the exact path, one evaluation each.
    pub exact: usize,
    /// Hypotheses that do not move this subprocess's `|M|²` at all.
    pub unchanged: usize,
}

impl SubprocessSummary {
    /// Amplitude evaluations per event, the card's own included.
    pub fn evaluations(&self) -> usize {
        1 + self.exact + self.polynomial.iter().map(|g| g.2).sum::<usize>()
    }
}

/// The compiled, planned reweighting of a run's subprocesses.
pub struct ReweightPlan {
    launches: Vec<Launch>,
    subs: Vec<SubPlan>,
}

impl ReweightPlan {
    /// Plan the reweighting of each subprocess in `sets` (in the caller's
    /// subprocess indexing) to each of `launches`, relative to `base`, the
    /// parameters the events were generated under.
    pub fn new(
        sets: &[&DiagramSet],
        model: &UFOModel,
        base: &EvaluatedModel,
        launches: Vec<Launch>,
        options: ReweightOptions,
    ) -> Result<Self, ReweightError> {
        let generic = generic_point(base, &launches);
        let mut subs = Vec::with_capacity(sets.len());
        for set in sets {
            if set
                .diagrams
                .iter()
                .any(|d| d.props.iter().any(|p| p.onshell == OnShell::Forbidden))
            {
                return Err(ReweightError::ForbiddenSChannel);
            }
            let mut evaluator = AmplitudeEvaluator::compile(set, model)
                .map_err(|e| ReweightError::Compile(e.to_string()))?;
            check_external_masses(&evaluator, model, &launches)?;
            evaluator.prune_zero_helicities(&generic);
            subs.push(plan_subprocess(
                evaluator, set, model, base, &launches, options,
            ));
        }
        Ok(ReweightPlan { launches, subs })
    }

    pub fn launches(&self) -> &[Launch] {
        &self.launches
    }

    /// How each subprocess's hypotheses are evaluated.
    pub fn summary(&self) -> Vec<SubprocessSummary> {
        self.subs
            .iter()
            .map(|sub| {
                let served =
                    sub.exact.len() + sub.groups.iter().map(|g| g.launches.len()).sum::<usize>();
                SubprocessSummary {
                    polynomial: sub
                        .groups
                        .iter()
                        .map(|g| {
                            let evaluations = g
                                .nodes
                                .iter()
                                .filter(|n| matches!(n, NodeSource::Slot(_)))
                                .count();
                            (g.param.clone(), g.degree, evaluations, g.launches.len())
                        })
                        .collect(),
                    exact: sub.exact.len(),
                    unchanged: self.launches.len() - served,
                }
            })
            .collect()
    }

    /// Bind every subprocess's amplitude at every parameter point the plan needs.
    pub fn bind(&self) -> Reweighter<'_> {
        let subs = self
            .subs
            .iter()
            .map(|sub| {
                let amps: Vec<ScaleAwareAmplitude<'_, f64>> = sub
                    .slots
                    .iter()
                    .map(|model| ScaleAwareAmplitude::new(&sub.evaluator, model))
                    .collect();
                let scratch = amps[0].scratch_space();
                BoundSub {
                    amps,
                    scratch,
                    values: Vec::new(),
                }
            })
            .collect();
        Reweighter { plan: self, subs }
    }
}

/// One subprocess's amplitudes, bound.
struct BoundSub<'p> {
    amps: Vec<ScaleAwareAmplitude<'p, f64>>,
    scratch: ScratchSpace<f64>,
    /// Per-event node values, reused across events.
    values: Vec<f64>,
}

/// A [`ReweightPlan`] bound to numeric pools: the per-event evaluator.
pub struct Reweighter<'p> {
    plan: &'p ReweightPlan,
    subs: Vec<BoundSub<'p>>,
}

impl Reweighter<'_> {
    /// The hypotheses, in the order [`ratios`](Self::ratios) reports them.
    pub fn launches(&self) -> &[Launch] {
        &self.plan.launches
    }

    /// `|M|²_new / |M|²_old` of subprocess `sub` at `momenta` for every hypothesis,
    /// written to `out` in launch order.
    ///
    /// `momenta` are the subprocess's own external momenta in its leg order, in the
    /// partonic centre of mass with the beams along ±z (the frame the generation
    /// evaluates in). `alpha_s` is the strong coupling the event's matrix element
    /// ran at, or `None` where nothing in it runs. An event whose card-point `|M|²`
    /// is zero — which an accepted event never is — gets zero ratios.
    pub fn ratios(
        &mut self,
        sub: usize,
        momenta: &[LorentzVector<f64>],
        alpha_s: Option<f64>,
        out: &mut Vec<f64>,
    ) {
        let plan = &self.plan.subs[sub];
        let bound = &mut self.subs[sub];
        out.clear();
        out.resize(self.plan.launches.len(), 1.0);

        if let Some(alpha_s) = alpha_s.filter(|&a| a > 0.0) {
            for amp in &mut bound.amps {
                amp.set_alpha_s(alpha_s);
            }
        }
        let m0 = bound.amps[0].eval_m2(momenta, &mut bound.scratch);
        let ratio = |m: f64| if m0 > 0.0 { m / m0 } else { 0.0 };

        for group in &plan.groups {
            bound.values.clear();
            for node in &group.nodes {
                let value = match *node {
                    NodeSource::Base => m0,
                    NodeSource::Slot(s) => bound.amps[s].eval_m2(momenta, &mut bound.scratch),
                };
                bound.values.push(value);
            }
            for (launch, weights) in &group.launches {
                let m: f64 = weights.iter().zip(&bound.values).map(|(w, v)| w * v).sum();
                out[*launch] = ratio(m);
            }
        }
        for &(launch, slot) in &plan.exact {
            out[launch] = ratio(bound.amps[slot].eval_m2(momenta, &mut bound.scratch));
        }
    }
}

/// `base` with every parameter any hypothesis moves set to an unremarkable value.
fn generic_point(base: &EvaluatedModel, launches: &[Launch]) -> EvaluatedModel {
    let mut generic = base.clone();
    let mut seen: Vec<&str> = Vec::new();
    for (name, value) in launches.iter().flat_map(|l| l.values.iter()) {
        if seen.contains(&name.as_str()) {
            continue;
        }
        seen.push(name);
        let v0 = base.param_values[name].re;
        // Away from the card's value, from zero and from every hypothesis's
        // round numbers, without leaving their scale.
        let scale = v0.abs().max(value.abs()).max(1.0);
        let offset = 0.291_7 + 0.017 * seen.len() as f64;
        generic.recompute(name, (v0 + offset * scale).into());
    }
    generic
}

/// Refuse a hypothesis that moves the mass of one of the subprocess's own legs.
fn check_external_masses(
    evaluator: &AmplitudeEvaluator,
    model: &UFOModel,
    launches: &[Launch],
) -> Result<(), ReweightError> {
    for launch in launches {
        for (name, _) in &launch.values {
            let mut moved = model.params.dependents(name);
            moved.insert(name.clone());
            for &pid in evaluator.external_particles() {
                let particle = model.particle(pid);
                if moved.contains(&particle.mass_param) {
                    return Err(ReweightError::ExternalMass {
                        launch: launch.id.clone(),
                        param: particle.mass_param.clone(),
                        particle: particle.name.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

/// Decide, for one subprocess, which hypotheses the polynomial path serves.
fn plan_subprocess(
    evaluator: AmplitudeEvaluator,
    set: &DiagramSet,
    model: &UFOModel,
    base: &EvaluatedModel,
    launches: &[Launch],
    options: ReweightOptions,
) -> SubPlan {
    let mut slots = vec![base.clone()];
    let mut groups = Vec::new();
    let mut exact = Vec::new();

    // Hypotheses moving exactly one parameter, grouped by it.
    let mut by_param: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (k, launch) in launches.iter().enumerate() {
        match launch.values.as_slice() {
            [] => {}
            [(name, _)] => by_param.entry(name.as_str()).or_default().push(k),
            _ => exact.push(k),
        }
    }

    for (param, members) in by_param {
        let degree = PolyAnalysis::new(model, param)
            .amplitude_support(&set.diagrams)
            .and_then(squared)
            .and_then(|s| s.degree());
        let v0 = base.param_values[param].re;
        match degree {
            // The parameter does not enter this subprocess's `|M|²`.
            Some(0) => {}
            Some(degree) if options.polynomial => {
                match polynomial_group(param, degree, v0, &members, launches) {
                    Some((group, node_values)) => {
                        let mut group = group;
                        for (node, value) in group.nodes.iter_mut().zip(node_values) {
                            if let NodeSource::Slot(slot) = node {
                                let mut point = base.clone();
                                point.recompute(param, value.into());
                                *slot = slots.len();
                                slots.push(point);
                            }
                        }
                        groups.push(group);
                    }
                    None => exact.extend(members),
                }
            }
            _ => exact.extend(members),
        }
    }

    exact.sort_unstable();
    let exact = exact
        .into_iter()
        .map(|k| {
            let mut point = base.clone();
            for (name, value) in &launches[k].values {
                point.recompute(name, (*value).into());
            }
            slots.push(point);
            (k, slots.len() - 1)
        })
        .collect();

    SubPlan {
        evaluator,
        slots,
        groups,
        exact,
    }
}

/// The polynomial group for the hypotheses `members` along `param`, with its node
/// values, or `None` when evaluating the hypotheses directly is no dearer.
///
/// Slots are left for the caller to number: every `NodeSource::Slot` carries `0`.
fn polynomial_group(
    param: &str,
    degree: u32,
    v0: f64,
    members: &[usize],
    launches: &[Launch],
) -> Option<(PolynomialGroup, Vec<f64>)> {
    let value_of = |k: usize| launches[k].values[0].1;
    let lo = members.iter().map(|&k| value_of(k)).fold(v0, f64::min);
    let hi = members.iter().map(|&k| value_of(k)).fold(v0, f64::max);
    if !(hi > lo) {
        return None;
    }

    let nodes = lobatto_nodes(lo, hi, degree as usize);
    let sources: Vec<NodeSource> = nodes
        .iter()
        .map(|&t| {
            if t == v0 {
                NodeSource::Base
            } else {
                NodeSource::Slot(0)
            }
        })
        .collect();
    let cost = sources
        .iter()
        .filter(|s| matches!(s, NodeSource::Slot(_)))
        .count();
    let mut distinct: Vec<f64> = members
        .iter()
        .map(|&k| value_of(k))
        .filter(|&v| v != v0)
        .collect();
    distinct.sort_by(f64::total_cmp);
    distinct.dedup();
    if cost >= distinct.len() {
        return None;
    }

    let weights = lobatto_weights(degree as usize);
    let launches = members
        .iter()
        .map(|&k| (k, lagrange_basis(&nodes, &weights, value_of(k))))
        .collect();
    Some((
        PolynomialGroup {
            param: param.to_string(),
            degree,
            nodes: sources,
            launches,
        },
        nodes,
    ))
}

/// The `n + 1` Chebyshev–Lobatto points of `[lo, hi]`, `lo` and `hi` exactly
/// among them.
fn lobatto_nodes(lo: f64, hi: f64, n: usize) -> Vec<f64> {
    (0..=n)
        .map(|j| {
            if j == 0 {
                lo
            } else if j == n {
                hi
            } else {
                let x = (std::f64::consts::PI * j as f64 / n as f64).cos();
                lo + (hi - lo) * (1.0 - x) / 2.0
            }
        })
        .collect()
}

/// Barycentric weights of the Chebyshev–Lobatto points: `(−1)ʲ`, halved at the
/// ends. An affine map of the interval scales every weight alike, which the
/// barycentric formula divides out.
fn lobatto_weights(n: usize) -> Vec<f64> {
    (0..=n)
        .map(|j| {
            let sign = if j % 2 == 0 { 1.0 } else { -1.0 };
            if j == 0 || j == n {
                0.5 * sign
            } else {
                sign
            }
        })
        .collect()
}

/// The Lagrange basis over `nodes` evaluated at `t`: the weights that read the
/// interpolating polynomial's value at `t` off its node values.
fn lagrange_basis(nodes: &[f64], weights: &[f64], t: f64) -> Box<[f64]> {
    if let Some(j) = nodes.iter().position(|&x| x == t) {
        let mut unit = vec![0.0; nodes.len()];
        unit[j] = 1.0;
        return unit.into_boxed_slice();
    }
    let terms: Vec<f64> = nodes
        .iter()
        .zip(weights)
        .map(|(&x, &w)| w / (t - x))
        .collect();
    let total: f64 = terms.iter().sum();
    terms.iter().map(|&c| c / total).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::helas::eval::BoundAmplitude;
    use crate::phasespace::rambo_massive;
    use crate::reweight::card::ReweightCard;
    use crate::reweight::resolve;
    use crate::ufo::sm::{sm_model, SMRestrict};
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use std::sync::Arc;

    type V = LorentzVector<f64>;

    fn sets(process: &str, model: &UFOModel) -> Vec<DiagramSet> {
        let card =
            parse_proc_card(&format!("generate {process}"), &ParsingOptions::default()).unwrap();
        generate_from_proc_card(&card, model)
            .unwrap()
            .into_iter()
            .filter(|s| !s.diagrams.is_empty())
            .collect()
    }

    /// Partonic-CM points for a 2 → n process with massless beams.
    fn points(eval: &AmplitudeEvaluator, evaluated: &EvaluatedModel, sqrt_s: f64) -> Vec<Vec<V>> {
        let masses: Vec<f64> = eval.external_particles()[2..]
            .iter()
            .map(|&p| evaluated.mass(p))
            .collect();
        let mut rng = StdRng::seed_from_u64(0x2E_3E16);
        (0..8)
            .map(|_| {
                let e = sqrt_s / 2.0;
                let mut p = vec![V::new(e, 0.0, 0.0, e), V::new(e, 0.0, 0.0, -e)];
                p.extend(rambo_massive(sqrt_s, &masses, &mut rng));
                p
            })
            .collect()
    }

    fn plan(
        process: &str,
        card: &str,
        options: ReweightOptions,
    ) -> (Arc<UFOModel>, EvaluatedModel, Vec<DiagramSet>, ReweightPlan) {
        let model = sm_model(SMRestrict::Default);
        let base = EvaluatedModel::from_model(model.clone());
        let sets = sets(process, &model);
        let card: ReweightCard = card.parse().unwrap();
        let launches = resolve(&card, &model).unwrap();
        let refs: Vec<&DiagramSet> = sets.iter().collect();
        let plan = ReweightPlan::new(&refs, &model, &base, launches, options).unwrap();
        (model, base, sets, plan)
    }

    /// The oracle both paths answer to: the unpruned amplitude bound directly at the
    /// hypothesis's parameters, over the one bound at the card's.
    fn direct_ratio(
        set: &DiagramSet,
        model: &UFOModel,
        base: &EvaluatedModel,
        launch: &Launch,
        momenta: &[V],
    ) -> f64 {
        let eval = AmplitudeEvaluator::compile(set, model).unwrap();
        let mut point = base.clone();
        for (name, value) in &launch.values {
            point.recompute(name, (*value).into());
        }
        let old = BoundAmplitude::<f64>::bind(&eval, base);
        let new = BoundAmplitude::<f64>::bind(&eval, &point);
        let mut scratch = old.scratch_space();
        new.eval_m2(momenta, &mut scratch) / old.eval_m2(momenta, &mut scratch)
    }

    const YMT_SCAN: &str = "\
launch --rwgt_name=y0
 set ymt 0
launch --rwgt_name=y1
 set ymt 100
launch --rwgt_name=y2
 set ymt 150
launch --rwgt_name=y3
 set ymt 200
launch --rwgt_name=y4
 set ymt 350
";

    /// `e+ e- > t t~ h` is `a + b·ymt` (Higgsstrahlung plus top-Yukawa radiation),
    /// so `|M|²` is quadratic in `ymt` with an interference term: the polynomial
    /// path must reproduce the direct evaluation at every hypothesis.
    #[test]
    fn the_polynomial_path_reproduces_direct_evaluation() {
        let (model, base, sets, plan) =
            plan("e+ e- > t t~ h", YMT_SCAN, ReweightOptions::default());
        let summary = plan.summary();
        assert_eq!(summary[0].polynomial.len(), 1, "{summary:?}");
        let (param, degree, cost, served) = &summary[0].polynomial[0];
        assert_eq!((param.as_str(), *degree, *served), ("ymt", 2, 5));
        // Three Lobatto nodes over [0, 350], none of them the card's 173.
        assert_eq!(*cost, 3);
        assert_eq!(summary[0].exact, 0);

        let mut rw = plan.bind();
        let mut out = Vec::new();
        let eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
        let mut spread = (f64::INFINITY, 0.0f64);
        for p in points(&eval, &base, 1000.0) {
            rw.ratios(0, &p, None, &mut out);
            for (launch, &got) in plan.launches().iter().zip(&out) {
                let want = direct_ratio(&sets[0], &model, &base, launch, &p);
                spread = (spread.0.min(want), spread.1.max(want));
                assert!(
                    (got - want).abs() <= 1e-11 * want.abs().max(1.0),
                    "{}: {got} vs {want}",
                    launch.id
                );
            }
        }
        // The hypotheses really do move |M|², and not by one common factor.
        assert!(spread.1 / spread.0 > 1.5, "{spread:?}");
    }

    /// The degree is load-bearing: a straight line through two of the nodes misses
    /// the quadratic term by far more than rounding.
    #[test]
    fn a_lower_degree_would_be_wrong() {
        let model = sm_model(SMRestrict::Default);
        let base = EvaluatedModel::from_model(model.clone());
        let set = &sets("e+ e- > t t~ h", &model)[0];
        let eval = AmplitudeEvaluator::compile(set, &model).unwrap();
        let p = &points(&eval, &base, 1000.0)[0];
        let at = |v: f64| {
            let mut point = base.clone();
            point.recompute("ymt", v.into());
            let bound = BoundAmplitude::<f64>::bind(&eval, &point);
            let mut scratch = bound.scratch_space();
            bound.eval_m2(p, &mut scratch)
        };
        let (m0, m1, m2) = (at(0.0), at(175.0), at(350.0));
        let linear = 0.5 * (m0 + m2);
        assert!(
            (linear - m1).abs() > 1e-3 * m1,
            "the quadratic term is too small for this check: {m0} {m1} {m2}"
        );
    }

    /// With the polynomial path off, every hypothesis is evaluated directly and
    /// agrees with the oracle; with it on, the two paths agree with each other.
    #[test]
    fn the_exact_path_agrees_and_the_paths_agree_with_each_other() {
        let (model, base, sets, exact) = plan(
            "e+ e- > t t~ h",
            YMT_SCAN,
            ReweightOptions { polynomial: false },
        );
        assert_eq!(exact.summary()[0].exact, 5);
        let (_, _, _, fast) = plan("e+ e- > t t~ h", YMT_SCAN, ReweightOptions::default());
        let eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
        let (mut a, mut b) = (exact.bind(), fast.bind());
        let (mut ra, mut rb) = (Vec::new(), Vec::new());
        for p in points(&eval, &base, 800.0) {
            a.ratios(0, &p, None, &mut ra);
            b.ratios(0, &p, None, &mut rb);
            for ((launch, &x), &y) in exact.launches().iter().zip(&ra).zip(&rb) {
                let want = direct_ratio(&sets[0], &model, &base, launch, &p);
                assert!((x - want).abs() <= 1e-12 * want.max(1.0), "{x} vs {want}");
                assert!((x - y).abs() <= 1e-11 * x.max(1.0), "{x} vs {y}");
            }
        }
    }

    /// Few hypotheses along a parameter are cheaper evaluated directly, and a
    /// hypothesis moving a parameter the process does not see changes nothing.
    #[test]
    fn the_plan_picks_the_cheaper_path() {
        let (_, _, _, few) = plan(
            "e+ e- > t t~ h",
            "launch\n set ymt 150\nlaunch\n set ymt 200\n",
            ReweightOptions::default(),
        );
        let s = &few.summary()[0];
        assert!(s.polynomial.is_empty());
        assert_eq!(s.exact, 2);

        let (_, base, sets, blind) = plan(
            "e+ e- > mu+ mu-",
            "launch\n set ymt 150\nlaunch\n set ymt 200\nlaunch\n set ymt 1\n",
            ReweightOptions::default(),
        );
        let s = &blind.summary()[0];
        assert_eq!((s.unchanged, s.exact, s.evaluations()), (3, 0, 1));
        let model = sm_model(SMRestrict::Default);
        let eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
        let mut rw = blind.bind();
        let mut out = Vec::new();
        rw.ratios(0, &points(&eval, &base, 300.0)[0], None, &mut out);
        assert_eq!(out, vec![1.0; 3]);
    }

    /// A parameter that is not a polynomial coupling — the Z mass, which is also a
    /// propagator pole — and a launch moving two parameters take the exact path.
    #[test]
    fn non_polynomial_and_multi_parameter_hypotheses_are_exact() {
        let card = "\
launch --rwgt_name=mz
 set MZ 91.0
launch --rwgt_name=both
 set ymt 150
 set ymtau 2.0
launch --rwgt_name=mz2
 set MZ 92.0
launch --rwgt_name=mz3
 set MZ 93.0
launch --rwgt_name=mz4
 set MZ 94.0
";
        let (model, base, sets, plan) = plan("e+ e- > t t~ h", card, ReweightOptions::default());
        let s = &plan.summary()[0];
        assert!(s.polynomial.is_empty(), "{s:?}");
        assert_eq!(s.exact, 5);
        let eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
        let mut rw = plan.bind();
        let mut out = Vec::new();
        for p in points(&eval, &base, 900.0).iter().take(3) {
            rw.ratios(0, p, None, &mut out);
            for (launch, &got) in plan.launches().iter().zip(&out) {
                let want = direct_ratio(&sets[0], &model, &base, launch, p);
                assert!((got - want).abs() <= 1e-12 * want.max(1.0), "{}", launch.id);
            }
        }
    }

    /// A hypothesis that switches on a coupling the card leaves at zero revives
    /// contributions the card's own pruning would drop: the plan must prune at a
    /// generic point, not at the card. At `ymt = 0` every diagram of
    /// `e+ e- > t t~ h` radiating the Higgs off the top is exactly zero, and the
    /// zero-amplitude pass removes them.
    #[test]
    fn pruning_keeps_what_a_hypothesis_revives() {
        let model = sm_model(SMRestrict::Default);
        let mut base = EvaluatedModel::from_model(model.clone());
        base.recompute("ymt", 0.0.into());
        let sets = sets("e+ e- > t t~ h", &model);
        let launches = resolve(&YMT_SCAN.parse().unwrap(), &model).unwrap();
        let refs: Vec<&DiagramSet> = sets.iter().collect();
        let plan = ReweightPlan::new(
            &refs,
            &model,
            &base,
            launches.clone(),
            ReweightOptions::default(),
        )
        .unwrap();
        let eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
        let mut rw = plan.bind();
        let mut out = Vec::new();
        for p in points(&eval, &base, 1000.0).iter().take(3) {
            rw.ratios(0, p, None, &mut out);
            for (launch, &got) in launches.iter().zip(&out) {
                let want = direct_ratio(&sets[0], &model, &base, launch, p);
                assert!(
                    (got - want).abs() <= 1e-10 * want.max(1.0),
                    "{}: {got} vs {want}",
                    launch.id
                );
            }
        }

        // The check is load-bearing: pruned at the card, the amplitude has lost the
        // Yukawa diagrams and no longer moves with `ymt`.
        let mut at_card = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
        at_card.prune_zero_helicities(&base);
        let (before, after) = at_card.zeroamp_node_reduction();
        assert!(after < before, "the card's pruning removed nothing");
        let mut point = base.clone();
        point.recompute("ymt", 173.0.into());
        let p = &points(&eval, &base, 1000.0)[0];
        let pruned = BoundAmplitude::<f64>::bind(&at_card, &point);
        let mut scratch = pruned.scratch_space();
        let lost = pruned.eval_m2(p, &mut scratch);
        let full = BoundAmplitude::<f64>::bind(&eval, &point).eval_m2(p, &mut scratch);
        assert!(
            (lost - full).abs() > 1e-3 * full,
            "pruning at the card kept the Yukawa diagrams: {lost} vs {full}"
        );
    }

    #[test]
    fn a_hypothesis_moving_an_external_mass_is_refused() {
        let model = sm_model(SMRestrict::Default);
        let base = EvaluatedModel::from_model(model.clone());
        let sets = sets("e+ e- > t t~", &model);
        let launches = resolve(&"launch\n set MT 170\n".parse().unwrap(), &model).unwrap();
        let refs: Vec<&DiagramSet> = sets.iter().collect();
        assert!(matches!(
            ReweightPlan::new(&refs, &model, &base, launches, ReweightOptions::default()),
            Err(ReweightError::ExternalMass { .. })
        ));
    }

    /// The per-event strong coupling moves the card-point and hypothesis amplitudes
    /// together, and the ratio is the one at that coupling.
    #[test]
    fn ratios_are_taken_at_the_events_strong_coupling() {
        let card = "launch\n set ymt 0\nlaunch\n set ymt 120\nlaunch\n set ymt 250\nlaunch\n set ymt 300\n";
        let (model, base, sets, plan) = plan("g g > t t~ h", card, ReweightOptions::default());
        let eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
        let p = &points(&eval, &base, 900.0)[0];
        let mut rw = plan.bind();
        let mut out = Vec::new();
        let alpha_s = 0.09;
        rw.ratios(0, p, Some(alpha_s), &mut out);
        let mut at_scale = base.clone();
        at_scale.set_alpha_s(alpha_s);
        for (launch, &got) in plan.launches().iter().zip(&out) {
            let want = direct_ratio(&sets[0], &model, &at_scale, launch, p);
            assert!(
                (got - want).abs() <= 1e-10 * want.max(1.0),
                "{got} vs {want}"
            );
        }
    }

    #[test]
    fn lagrange_weights_reproduce_polynomials_exactly_at_the_nodes_and_between() {
        let nodes = lobatto_nodes(-1.0, 3.0, 4);
        let weights = lobatto_weights(4);
        let f = |x: f64| 2.0 - x + 0.5 * x.powi(3) - 0.25 * x.powi(4);
        let values: Vec<f64> = nodes.iter().map(|&x| f(x)).collect();
        for t in [-1.0, -0.3, 0.0, 1.1, 2.5, 3.0] {
            let l = lagrange_basis(&nodes, &weights, t);
            let got: f64 = l.iter().zip(&values).map(|(a, b)| a * b).sum();
            assert!((got - f(t)).abs() < 1e-13, "{t}: {got} vs {}", f(t));
        }
    }
}
