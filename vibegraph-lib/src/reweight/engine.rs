//! The per-event reweighting plan and its evaluation.
//!
//! [`ReweightPlan`] owns one compiled amplitude per subprocess and decides, per
//! subprocess, which hypotheses take the polynomial path and which the exact path
//! (see the [module docs](super)). [`Reweighter`] is the plan bound to numeric
//! constant pools: mutable, single-threaded state whose
//! [`ratios`](Reweighter::ratios) is the per-event inner loop.
//!
//! # The polynomial path, concretely
//!
//! A subprocess's amplitude is `A(P) = Σ_μ μ(P)·a_μ` over the monomials `μ` its
//! [`Support`] proves — per helicity combination and colour flow, with `a_μ` the
//! amplitude of that coupling class. Its value at `K = |Support|` well-chosen
//! parameter nodes `P_j` determines every `a_μ`: `A(P_j) = Σ_μ V_jμ a_μ` with
//! `V_jμ = μ(P_j)`. Then for any hypothesis `P_L`,
//! `A(P_L) = Σ_j u_Lj A(P_j)` with `u_L = V⁻ᵀ μ(P_L)`, a real vector fixed before
//! the first event, and
//!
//! ```text
//! |M(P_L)|² = u_Lᵀ R u_L,   R_jk = Re Σ_hel Σ_fg CF_fg conj(J_f(P_j)) J_g(P_k)
//! ```
//!
//! so an event costs `K − 1` amplitude evaluations beyond the card's own (the card
//! point is always node 0) and one `K × K` matrix, and each hypothesis a quadratic
//! form in it. `R` in the node basis is the monomial Gram matrix `Σ conj(a_μ)·a_ν`
//! in another basis; the class amplitudes themselves never need forming.
//!
//! The nodes are picked for conditioning, not placed by hand: a Chebyshev grid over
//! the box the hypotheses span, from which a column-pivoted QR takes the `K − 1`
//! points that, after the card's own, best span the monomial basis.
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

use std::collections::{BTreeMap, BTreeSet};

use nalgebra::{DMatrix, DVector};
use num_complex::Complex64;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::diagrams::diagram::OnShell;
use crate::diagrams::DiagramSet;
use crate::helas::eval::{AmplitudeEvaluator, ScaleAwareAmplitude, ScratchSpace};
use crate::helas::repr::lorentz::LorentzVector;
use crate::ufo::{EvaluatedModel, UFOModel};

use super::poly::{PolyAnalysis, Support};
use super::{Launch, ReweightError};

/// Largest condition number of the node system accepted. Past it the node values'
/// rounding would be amplified into the weights beyond what the ratios can carry.
const MAX_CONDITION: f64 = 1e10;
/// Candidate nodes drawn per basis monomial when the Chebyshev grid is larger.
const CANDIDATES_PER_TERM: usize = 16;
const CANDIDATE_SEED: u64 = 0x2E_16_4D;

/// How the plan may evaluate hypotheses.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReweightOptions {
    /// Every hypothesis takes the exact path.
    pub exact: bool,
    /// Serve every hypothesis by one polynomial per subprocess jointly in these
    /// parameters: each hypothesis may move only them, and each must enter every
    /// subprocess polynomially. `None` groups hypotheses by the single parameter
    /// they move, where that is cheaper than evaluating them.
    pub couplings: Option<Vec<String>>,
}

/// Where one node's amplitude comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NodeSource {
    /// The card's own point, whose amplitude every event evaluates anyway.
    Base,
    /// A bound amplitude of its own.
    Slot(usize),
}

/// Hypotheses of one subprocess served by one polynomial.
#[derive(Clone, Debug)]
struct PolynomialGroup {
    params: Vec<String>,
    nodes: Vec<NodeSource>,
    /// Per hypothesis: its launch index and its node weights `u_L`.
    launches: Vec<(usize, Box<[f64]>)>,
}

/// One subprocess's evaluation plan.
struct SubPlan {
    evaluator: AmplitudeEvaluator,
    /// The subprocess's external PDG codes, in its own leg order.
    pdgs: Vec<i32>,
    /// Parameter points to bind the amplitude at; slot 0 is the card's own.
    slots: Vec<EvaluatedModel>,
    groups: Vec<PolynomialGroup>,
    /// Hypotheses on the exact path: launch index and slot.
    exact: Vec<(usize, usize)>,
}

/// One polynomial group of a subprocess, for a run's log.
#[derive(Clone, Debug, PartialEq)]
pub struct PolynomialSummary {
    pub params: Vec<String>,
    /// Monomials of the amplitude: the number of nodes.
    pub terms: usize,
    /// Amplitude evaluations per event beyond the card's own.
    pub evaluations: usize,
    pub hypotheses: usize,
}

/// How one subprocess's hypotheses are evaluated, for a run's log.
#[derive(Clone, Debug, PartialEq)]
pub struct SubprocessSummary {
    pub polynomial: Vec<PolynomialSummary>,
    /// Hypotheses on the exact path, one evaluation each.
    pub exact: usize,
    /// Hypotheses that do not move this subprocess's `|M|²` at all.
    pub unchanged: usize,
}

impl SubprocessSummary {
    /// Amplitude evaluations per event, the card's own included.
    pub fn evaluations(&self) -> usize {
        1 + self.exact + self.polynomial.iter().map(|g| g.evaluations).sum::<usize>()
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
        if let Some(couplings) = &options.couplings {
            for launch in &launches {
                if let Some((param, _)) = launch.values.iter().find(|(p, _)| !couplings.contains(p))
                {
                    return Err(ReweightError::OutsideCouplings {
                        launch: launch.id.clone(),
                        param: param.clone(),
                    });
                }
            }
        }
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
            let pdgs = evaluator
                .external_particles()
                .iter()
                .map(|&pid| model.particle(pid).pdg_code as i32)
                .collect();
            let mut sub = plan_subprocess(evaluator, set, model, base, &launches, &options)?;
            sub.pdgs = pdgs;
            subs.push(sub);
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
                        .map(|g| PolynomialSummary {
                            params: g.params.clone(),
                            terms: g.nodes.len(),
                            evaluations: g
                                .nodes
                                .iter()
                                .filter(|n| matches!(n, NodeSource::Slot(_)))
                                .count(),
                            hypotheses: g.launches.len(),
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
                    jamps: Vec::new(),
                    contracted: Vec::new(),
                    gram: Vec::new(),
                }
            })
            .collect();
        Reweighter { plan: self, subs }
    }
}

/// One subprocess's amplitudes, bound, with per-event buffers.
struct BoundSub<'p> {
    amps: Vec<ScaleAwareAmplitude<'p, f64>>,
    scratch: ScratchSpace<f64>,
    /// Per node, the JAMPs of every helicity combination.
    jamps: Vec<Vec<Complex64>>,
    /// Per node, the JAMPs contracted with the colour matrix.
    contracted: Vec<Vec<Complex64>>,
    /// The node Gram matrix `R`, row-major.
    gram: Vec<f64>,
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

    /// The external PDG codes of subprocess `sub`, in the leg order
    /// [`ratios`](Self::ratios) takes its momenta in.
    pub fn pdgs(&self, sub: usize) -> &[i32] {
        &self.plan.subs[sub].pdgs
    }

    /// `|M|²_new / |M|²_old` of subprocess `sub` at `momenta` for every hypothesis,
    /// written to `out` in launch order; returns `|M|²_old`, the card-point
    /// `|M|²` the ratios are taken against.
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
    ) -> f64 {
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
        let ratio = |m: f64, m0: f64| if m0 > 0.0 { m / m0 } else { 0.0 };

        for group in &plan.groups {
            bound.node_gram(&group.nodes, momenta);
            let k = group.nodes.len();
            let r = &bound.gram;
            // The card point is node 0, so its |M|² is `R₀₀`: the denominator
            // comes out of the same contraction as every numerator.
            let r00 = r[0];
            for (launch, u) in &group.launches {
                let mut m = 0.0;
                for (j, &uj) in u.iter().enumerate() {
                    let row = &r[j * k..(j + 1) * k];
                    let ru: f64 = row.iter().zip(u.iter()).map(|(a, b)| a * b).sum();
                    m += uj * ru;
                }
                out[*launch] = ratio(m, r00);
            }
        }
        for &(launch, slot) in &plan.exact {
            out[launch] = ratio(bound.amps[slot].eval_m2(momenta, &mut bound.scratch), m0);
        }
        m0
    }
}

impl BoundSub<'_> {
    /// Fill `gram` with `R_jk = Re Σ_hel Σ_fg CF_fg conj(J_f(P_j)) J_g(P_k)` over
    /// `nodes` at `momenta`.
    fn node_gram(&mut self, nodes: &[NodeSource], momenta: &[LorentzVector<f64>]) {
        let k = nodes.len();
        self.jamps.resize_with(k, Vec::new);
        self.contracted.resize_with(k, Vec::new);
        for (j, node) in nodes.iter().enumerate() {
            let slot = match *node {
                NodeSource::Base => 0,
                NodeSource::Slot(s) => s,
            };
            self.amps[slot].amplitude().eval_hel_jamps(
                momenta,
                &mut self.scratch,
                &mut self.jamps[j],
            );
        }
        let cf = self.amps[0].amplitude().cf_weights();
        let n = self.amps[0].amplitude().evaluator().n_flows();
        // `T_j = CF · J_j` per helicity combination, the contraction `eval_m2`
        // forms as its `ZTEMP`.
        for (jamps, t) in self.jamps.iter().zip(self.contracted.iter_mut()) {
            t.clear();
            for combo in jamps.chunks_exact(n) {
                for i in 0..n {
                    let mut z = Complex64::new(0.0, 0.0);
                    for (jj, a) in combo.iter().enumerate() {
                        z += a * cf[jj * n + i];
                    }
                    t.push(z);
                }
            }
        }
        self.gram.clear();
        self.gram.resize(k * k, 0.0);
        for a in 0..k {
            for b in a..k {
                let value: f64 = self.contracted[a]
                    .iter()
                    .zip(&self.jamps[b])
                    .map(|(t, j)| (t * j.conj()).re)
                    .sum();
                self.gram[a * k + b] = value;
                self.gram[b * k + a] = value;
            }
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

fn process_label(set: &DiagramSet) -> String {
    format!(
        "{} > {}",
        set.particles_in.join(" "),
        set.particles_out.join(" ")
    )
}

/// What one polynomial group of hypotheses becomes for one subprocess.
enum GroupPlan {
    /// The hypotheses leave this subprocess's `|M|²` where it is.
    Unchanged,
    /// Evaluating the hypotheses directly is no dearer.
    Exact,
    /// The group, and each slot node's full parameter point.
    Polynomial(PolynomialGroup, Vec<Vec<(String, f64)>>),
}

/// Decide, for one subprocess, which hypotheses the polynomial path serves.
fn plan_subprocess(
    evaluator: AmplitudeEvaluator,
    set: &DiagramSet,
    model: &UFOModel,
    base: &EvaluatedModel,
    launches: &[Launch],
    options: &ReweightOptions,
) -> Result<SubPlan, ReweightError> {
    let mut slots = vec![base.clone()];
    let mut groups = Vec::new();
    let mut exact = Vec::new();

    // `(parameters, members, forced)`: with an explicit coupling set, one group
    // over all of it that must be served; otherwise one per single parameter,
    // served where cheaper.
    let mut specs: Vec<(Vec<String>, Vec<usize>, bool)> = Vec::new();
    let moving = |k: usize| !launches[k].values.is_empty();
    if options.exact {
        exact.extend((0..launches.len()).filter(|&k| moving(k)));
    } else if let Some(couplings) = &options.couplings {
        specs.push((
            couplings.clone(),
            (0..launches.len()).filter(|&k| moving(k)).collect(),
            true,
        ));
    } else {
        let mut by_param: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
        for (k, launch) in launches.iter().enumerate() {
            match launch.values.as_slice() {
                [] => {}
                [(name, _)] => by_param.entry(name.as_str()).or_default().push(k),
                _ => exact.push(k),
            }
        }
        specs.extend(
            by_param
                .into_iter()
                .map(|(p, members)| (vec![p.to_string()], members, false)),
        );
    }

    for (params, members, forced) in specs {
        let names: Vec<&str> = params.iter().map(String::as_str).collect();
        let support = PolyAnalysis::new(model, &names).amplitude_support(&set.diagrams);
        let plan = match support {
            None if forced => {
                return Err(ReweightError::NotPolynomial {
                    couplings: params.join(","),
                    process: process_label(set),
                })
            }
            None => GroupPlan::Exact,
            Some(s) if s.is_constant() => GroupPlan::Unchanged,
            Some(s) => polynomial_group(&params, &s, base, &members, launches, !forced).map_err(
                |reason| ReweightError::NodeSystem {
                    process: process_label(set),
                    reason,
                },
            )?,
        };
        match plan {
            GroupPlan::Unchanged => {}
            GroupPlan::Exact => exact.extend(members),
            GroupPlan::Polynomial(mut group, points) => {
                let mut points = points.into_iter();
                for node in group.nodes.iter_mut() {
                    if let NodeSource::Slot(slot) = node {
                        let mut model = base.clone();
                        for (name, value) in points.next().expect("one point per slot node") {
                            model.recompute(&name, value.into());
                        }
                        *slot = slots.len();
                        slots.push(model);
                    }
                }
                groups.push(group);
            }
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

    Ok(SubPlan {
        evaluator,
        pdgs: Vec::new(),
        slots,
        groups,
        exact,
    })
}

/// The variables a group's polynomial actually runs over, and the affine map
/// `x = (P − c)/h` into the unit box its monomials are evaluated in.
struct Coordinates {
    /// Indices into the group's parameter list.
    active: Vec<usize>,
    center: Vec<f64>,
    half: Vec<f64>,
    lo: Vec<f64>,
    hi: Vec<f64>,
}

impl Coordinates {
    fn scaled(&self, point: &[f64]) -> Vec<f64> {
        self.active
            .iter()
            .enumerate()
            .map(|(a, &i)| (point[i] - self.center[a]) / self.half[a])
            .collect()
    }
}

fn monomial(x: &[f64], exponents: &[u8]) -> f64 {
    x.iter()
        .zip(exponents)
        .map(|(v, &e)| v.powi(i32::from(e)))
        .product()
}

/// The polynomial group serving `members` jointly in `params`, whose amplitude
/// carries the monomials `support`.
///
/// With `gate`, a group that would cost as many evaluations as its hypotheses is
/// declined ([`GroupPlan::Exact`]). The error is a node system that cannot be
/// solved stably, which only an explicitly requested group reports.
fn polynomial_group(
    params: &[String],
    support: &Support,
    base: &EvaluatedModel,
    members: &[usize],
    launches: &[Launch],
    gate: bool,
) -> Result<GroupPlan, String> {
    let v0: Vec<f64> = params.iter().map(|p| base.param_values[p].re).collect();
    let point_of = |k: usize| -> Vec<f64> {
        let mut p = v0.clone();
        for (name, value) in &launches[k].values {
            if let Some(i) = params.iter().position(|q| q == name) {
                p[i] = *value;
            }
        }
        p
    };
    let points: Vec<Vec<f64>> = members.iter().map(|&k| point_of(k)).collect();

    // A parameter the amplitude does not carry, or one no hypothesis moves off the
    // card, is not a variable of this polynomial.
    let mut coords = Coordinates {
        active: Vec::new(),
        center: Vec::new(),
        half: Vec::new(),
        lo: Vec::new(),
        hi: Vec::new(),
    };
    for i in 0..params.len() {
        let lo = points.iter().map(|p| p[i]).fold(v0[i], f64::min);
        let hi = points.iter().map(|p| p[i]).fold(v0[i], f64::max);
        if support.max_exponent(i) == 0 || !(hi > lo) {
            continue;
        }
        // Centred at zero where the range reaches it, which keeps the monomials
        // the analysis proved; elsewhere at the middle, which needs every lower
        // power too.
        let center = if lo <= 0.0 && 0.0 <= hi {
            0.0
        } else {
            0.5 * (lo + hi)
        };
        coords.active.push(i);
        coords.center.push(center);
        coords
            .half
            .push((lo - center).abs().max((hi - center).abs()));
        coords.lo.push(lo);
        coords.hi.push(hi);
    }
    if coords.active.is_empty() {
        return Ok(GroupPlan::Unchanged);
    }

    let basis = monomial_basis(support, &coords);
    let k = basis.len();

    if gate {
        let base_x = coords.scaled(&v0);
        let mut distinct: Vec<Vec<u64>> = points
            .iter()
            .map(|p| coords.scaled(p))
            .filter(|x| *x != base_x)
            .map(|x| x.iter().map(|v| v.to_bits()).collect())
            .collect();
        distinct.sort();
        distinct.dedup();
        if k > distinct.len() {
            return Ok(GroupPlan::Exact);
        }
    }

    let candidates = candidate_nodes(&coords, &basis, &v0);
    let chosen = select_nodes(&candidates, &basis, &coords)?;

    let v = DMatrix::from_fn(k, k, |row, col| {
        monomial(&coords.scaled(&candidates[chosen[row]]), &basis[col])
    });
    let singular = v.clone().svd(false, false).singular_values;
    let condition = singular.max() / singular.min();
    if !(condition <= MAX_CONDITION) {
        return Err(format!(
            "the {k} interpolation nodes are ill-conditioned (condition number {condition:.1e})"
        ));
    }
    let vt = v.transpose().full_piv_lu();
    let weights = members
        .iter()
        .zip(&points)
        .map(|(&launch, p)| {
            let x = coords.scaled(p);
            let rhs = DVector::from_iterator(k, basis.iter().map(|e| monomial(&x, e)));
            let u = vt
                .solve(&rhs)
                .ok_or_else(|| "the interpolation nodes are singular".to_string())?;
            Ok((launch, u.iter().copied().collect()))
        })
        .collect::<Result<Vec<_>, String>>()?;

    let nodes = (0..k)
        .map(|j| {
            if j == 0 {
                NodeSource::Base
            } else {
                NodeSource::Slot(0)
            }
        })
        .collect();
    let node_points = chosen[1..]
        .iter()
        .map(|&c| {
            coords
                .active
                .iter()
                .map(|&i| (params[i].clone(), candidates[c][i]))
                .collect()
        })
        .collect();
    Ok(GroupPlan::Polynomial(
        PolynomialGroup {
            params: coords.active.iter().map(|&i| params[i].clone()).collect(),
            nodes,
            launches: weights,
        },
        node_points,
    ))
}

/// The monomials of the amplitude in the group's own coordinates: the support
/// projected onto the active variables and, along each variable whose centre is
/// not zero, closed downwards — `(c + h·x)ⁿ` carries every lower power of `x`.
fn monomial_basis(support: &Support, coords: &Coordinates) -> Vec<Vec<u8>> {
    let mut basis: BTreeSet<Vec<u8>> = support
        .terms()
        .map(|t| coords.active.iter().map(|&i| t[i]).collect())
        .collect();
    for (a, &c) in coords.center.iter().enumerate() {
        if c == 0.0 {
            continue;
        }
        let mut closed = BTreeSet::new();
        for t in &basis {
            for e in 0..=t[a] {
                let mut lower = t.clone();
                lower[a] = e;
                closed.insert(lower);
            }
        }
        basis = closed;
    }
    basis.into_iter().collect()
}

/// Candidate nodes, full parameter points: the card's first, then the tensor grid
/// of Chebyshev–Lobatto points along each active variable (as many as its degree
/// needs), or a seeded sample of that grid when it is much larger than the basis.
fn candidate_nodes(coords: &Coordinates, basis: &[Vec<u8>], v0: &[f64]) -> Vec<Vec<f64>> {
    let axes: Vec<Vec<f64>> = (0..coords.active.len())
        .map(|a| {
            let degree = basis.iter().map(|t| t[a]).max().unwrap_or(0).max(1) as usize;
            let (lo, hi) = (coords.lo[a], coords.hi[a]);
            (0..=degree)
                .map(|j| {
                    let x = (std::f64::consts::PI * j as f64 / degree as f64).cos();
                    lo + (hi - lo) * (1.0 - x) / 2.0
                })
                .collect()
        })
        .collect();
    let at = |index: &[usize]| -> Vec<f64> {
        let mut p = v0.to_vec();
        for (a, &i) in coords.active.iter().enumerate() {
            p[i] = axes[a][index[a]];
        }
        p
    };
    let cap = (CANDIDATES_PER_TERM * basis.len()).max(64);
    let grid: Option<usize> = axes
        .iter()
        .try_fold(1usize, |n, axis| n.checked_mul(axis.len()));
    let mut out = vec![v0.to_vec()];
    match grid {
        Some(n) if n <= cap => {
            let mut index = vec![0; axes.len()];
            for _ in 0..n {
                out.push(at(&index));
                for (a, i) in index.iter_mut().enumerate() {
                    *i += 1;
                    if *i < axes[a].len() {
                        break;
                    }
                    *i = 0;
                }
            }
        }
        _ => {
            let mut rng = ChaCha8Rng::seed_from_u64(CANDIDATE_SEED);
            for _ in 0..cap {
                let index: Vec<usize> = axes
                    .iter()
                    .map(|axis| rng.random_range(0..axis.len()))
                    .collect();
                out.push(at(&index));
            }
        }
    }
    out
}

/// Pick the card point and the `K − 1` candidates that, after it, best span the
/// monomial basis: a column-pivoted QR of the candidates' monomial vectors with
/// the card's projected out, whose pivot order is the greedy choice.
fn select_nodes(
    candidates: &[Vec<f64>],
    basis: &[Vec<u8>],
    coords: &Coordinates,
) -> Result<Vec<usize>, String> {
    let k = basis.len();
    let rows: Vec<Vec<f64>> = candidates
        .iter()
        .map(|p| {
            let x = coords.scaled(p);
            basis.iter().map(|e| monomial(&x, e)).collect()
        })
        .collect();
    let norm = rows[0].iter().map(|v| v * v).sum::<f64>().sqrt();
    if !(norm > 0.0) {
        return Err("every monomial of the amplitude vanishes at the card's parameters".into());
    }
    let q: Vec<f64> = rows[0].iter().map(|v| v / norm).collect();
    let others = candidates.len() - 1;
    let residual = DMatrix::from_fn(k, others, |r, c| {
        let row = &rows[c + 1];
        let dot: f64 = row.iter().zip(&q).map(|(a, b)| a * b).sum();
        row[r] - dot * q[r]
    });
    let qr = residual.col_piv_qr();
    let mut order = DMatrix::from_fn(1, others, |_, c| c as f64);
    qr.p().permute_columns(&mut order);
    let mut chosen = vec![0];
    chosen.extend(order.iter().take(k - 1).map(|&c| c as usize + 1));
    if chosen.len() < k {
        return Err(format!(
            "{} candidate nodes cannot determine {k} monomials",
            candidates.len()
        ));
    }
    Ok(chosen)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::helas::eval::BoundAmplitude;
    use crate::phasespace::rambo_massive;
    use crate::reweight::card::ReweightCard;
    use crate::reweight::{resolve, resolve_couplings};
    use crate::ufo::sm::{sm_model, SMRestrict};
    use rand::rngs::StdRng;
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

    /// Partonic-CM points for a 2 → n process, beams along ±z on their mass shells.
    fn points(eval: &AmplitudeEvaluator, evaluated: &EvaluatedModel, sqrt_s: f64) -> Vec<Vec<V>> {
        let mass = |p| evaluated.mass(p);
        let ext = eval.external_particles();
        let masses: Vec<f64> = ext[2..].iter().map(|&p| mass(p)).collect();
        let (m1, m2) = (mass(ext[0]), mass(ext[1]));
        let s = sqrt_s * sqrt_s;
        let e1 = (s + m1 * m1 - m2 * m2) / (2.0 * sqrt_s);
        let pz = (e1 * e1 - m1 * m1).sqrt();
        let mut rng = StdRng::seed_from_u64(0x2E_3E16);
        (0..8)
            .map(|_| {
                let mut p = vec![V::new(e1, 0.0, 0.0, pz), V::new(sqrt_s - e1, 0.0, 0.0, -pz)];
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

    fn joint(names: &[&str]) -> ReweightOptions {
        let model = sm_model(SMRestrict::Default);
        let names: Vec<String> = names.iter().map(|n| n.to_string()).collect();
        ReweightOptions {
            exact: false,
            couplings: Some(resolve_couplings(&model, &names).unwrap()),
        }
    }

    /// `at`'s external parameters with `moved` applied, evaluated afresh from a
    /// parameter card: no `recompute` on the way, so the oracle does not share the
    /// plan's parameter propagation.
    fn fresh(at: &EvaluatedModel, moved: &[(String, f64)]) -> EvaluatedModel {
        let model = at.model().clone();
        let mut card = String::new();
        for p in &model.params.externals {
            let crate::ufo::parameters::ParamNature::External {
                lha_block,
                lha_code,
                ..
            } = &p.nature
            else {
                continue;
            };
            let value = moved
                .iter()
                .rev()
                .find(|(n, _)| *n == p.name)
                .map_or(at.param_values[&p.name].re, |(_, v)| *v);
            let code: Vec<String> = lha_code.iter().map(i32::to_string).collect();
            card += &format!("BLOCK {lha_block}\n {} {value:.17e}\n", code.join(" "));
        }
        EvaluatedModel::from_model_card(model, &card.parse().unwrap())
    }

    /// The oracle every path answers to: the unpruned amplitude bound directly at
    /// the hypothesis's parameters, over the one bound at the card's, both
    /// evaluated afresh ([`fresh`]).
    fn direct_ratio(
        set: &DiagramSet,
        model: &UFOModel,
        base: &EvaluatedModel,
        launch: &Launch,
        momenta: &[V],
    ) -> f64 {
        let eval = AmplitudeEvaluator::compile(set, model).unwrap();
        let card = fresh(base, &[]);
        let point = fresh(base, &launch.values);
        let old = BoundAmplitude::<f64>::bind(&eval, &card);
        let new = BoundAmplitude::<f64>::bind(&eval, &point);
        let mut scratch = old.scratch_space();
        new.eval_m2(momenta, &mut scratch) / old.eval_m2(momenta, &mut scratch)
    }

    /// Every hypothesis's ratio at every point against [`direct_ratio`], to `tol`
    /// relative to the larger of the ratio and one; returns the ratios' spread.
    fn check_against_direct(
        process: &str,
        sqrt_s: f64,
        base: &EvaluatedModel,
        plan: &ReweightPlan,
        alpha_s: Option<f64>,
        tol: f64,
    ) -> (f64, f64) {
        let model = base.model().clone();
        let set = &sets(process, &model)[0];
        let eval = AmplitudeEvaluator::compile(set, &model).unwrap();
        let mut at = base.clone();
        if let Some(a) = alpha_s {
            at.set_alpha_s(a);
        }
        let mut rw = plan.bind();
        let mut out = Vec::new();
        let mut spread = (f64::INFINITY, 0.0f64);
        for p in points(&eval, base, sqrt_s).iter().take(4) {
            rw.ratios(0, p, alpha_s, &mut out);
            for (launch, &got) in plan.launches().iter().zip(&out) {
                let want = direct_ratio(set, &model, &at, launch, p);
                spread = (spread.0.min(want), spread.1.max(want));
                assert!(
                    (got - want).abs() <= tol * want.abs().max(1.0),
                    "{process} {}: {got} vs {want}",
                    launch.id
                );
            }
        }
        spread
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

    /// `e+ e- > t t~ h` is `a + b·ymt` (Higgsstrahlung plus top-Yukawa radiation):
    /// two amplitude classes, so one evaluation beyond the card's serves the scan.
    #[test]
    fn the_polynomial_path_reproduces_direct_evaluation() {
        let (_, base, _, plan) = plan("e+ e- > t t~ h", YMT_SCAN, ReweightOptions::default());
        let summary = plan.summary();
        assert_eq!(
            summary[0].polynomial,
            [PolynomialSummary {
                params: vec!["ymt".into()],
                terms: 2,
                evaluations: 1,
                hypotheses: 5
            }]
        );
        assert_eq!((summary[0].exact, summary[0].evaluations()), (0, 2));
        let spread = check_against_direct("e+ e- > t t~ h", 1000.0, &base, &plan, None, 1e-11);
        // The hypotheses really do move |M|², and not by one common factor.
        assert!(spread.1 / spread.0 > 1.5, "{spread:?}");
    }

    /// The Gram contraction of the JAMPs is `eval_m2`'s own: at the card's point,
    /// `R₀₀` must be `|M|²`. On a two-flow process, so the colour matrix enters.
    /// Blind to a transpose of the colour matrix, which is symmetric.
    #[test]
    fn the_gram_matrix_reproduces_the_matrix_element() {
        let (model, base, sets, plan) = plan(
            "g g > t t~ h",
            "launch\n set ymt 0\nlaunch\n set ymt 120\nlaunch\n set ymt 300\n",
            ReweightOptions::default(),
        );
        assert_eq!(plan.subs[0].evaluator.n_flows(), 2);
        let eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
        let mut rw = plan.bind();
        let group = &plan.subs[0].groups[0];
        for p in points(&eval, &base, 900.0).iter().take(3) {
            let bound = &mut rw.subs[0];
            let m0 = bound.amps[0].eval_m2(p, &mut bound.scratch);
            bound.node_gram(&group.nodes, p);
            let r00 = bound.gram[0];
            assert!((r00 - m0).abs() <= 1e-13 * m0, "{r00} vs {m0}");
        }
    }

    /// Finer than `|M|²`: the class amplitudes the node solve implies rebuild the
    /// complex JAMP of every helicity combination and colour flow at every
    /// hypothesis, against a direct evaluation there. `u u~ > t t~ h` with both
    /// coupling orders has two flows and two classes: the Yukawa class (QCD and
    /// electroweak diagrams) reaches both flows, and the constant one —
    /// Higgsstrahlung off an s-channel Z, colour-singlet — one of them, which the
    /// Yukawa class shares. A node or flow mix-up in the decomposition is then an
    /// O(1) miss on some entry here while `|M|²` could still agree.
    ///
    /// Blind to anything the two sides share: the evaluator and its helicity
    /// list (the amplitude itself is held to MadGraph by the amplitude oracles),
    /// and the colour matrix, which this never contracts.
    #[test]
    fn class_amplitudes_rebuild_every_helicity_and_flow() {
        let card = "launch\n set ymt 0\nlaunch\n set ymt 100\nlaunch\n set ymt 250\nlaunch\n set ymt 400\n";
        let process = "u u~ > t t~ h QCD=2 QED=3";
        let (model, base, sets, plan) = plan(process, card, joint(&["ymt"]));
        let sub = &plan.subs[0];
        let n = sub.evaluator.n_flows();
        assert_eq!(n, 2);
        let group = &sub.groups[0];
        assert_eq!(group.nodes.len(), 2, "classes 1 and ymt");
        let ymt = |m: &EvaluatedModel| m.param_values["ymt"].re;
        let node_ymt: Vec<f64> = group
            .nodes
            .iter()
            .map(|node| match *node {
                NodeSource::Base => ymt(&sub.slots[0]),
                NodeSource::Slot(s) => ymt(&sub.slots[s]),
            })
            .collect();
        assert!(node_ymt[0] != node_ymt[1]);

        let eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
        let mut rw = plan.bind();
        let mut direct = Vec::new();
        let mut populated = [vec![false; n], vec![false; n]];
        for p in points(&eval, &base, 1000.0).iter().take(3) {
            let bound = &mut rw.subs[0];
            bound.node_gram(&group.nodes, p);
            let (j0, j1) = (bound.jamps[0].clone(), bound.jamps[1].clone());
            // `J(y) = a + y·b`, read off the two nodes.
            let b: Vec<Complex64> = j0
                .iter()
                .zip(&j1)
                .map(|(x0, x1)| (x1 - x0) / (node_ymt[1] - node_ymt[0]))
                .collect();
            let a: Vec<Complex64> = j0
                .iter()
                .zip(&b)
                .map(|(x0, bb)| x0 - bb * node_ymt[0])
                .collect();
            let scale = j0.iter().chain(&j1).map(|z| z.norm()).fold(0.0, f64::max);
            for (k, (aa, bb)) in a.iter().zip(&b).enumerate() {
                populated[0][k % n] |= aa.norm() > 1e-6 * scale;
                populated[1][k % n] |= bb.norm() > 1e-6 * scale;
            }
            for (launch, u) in &group.launches {
                let values = &plan.launches[*launch].values;
                let y = values[0].1;
                let point = fresh(&base, values);
                let bound_direct = BoundAmplitude::<f64>::bind(&sub.evaluator, &point);
                let mut scratch = bound_direct.scratch_space();
                bound_direct.eval_hel_jamps(p, &mut scratch, &mut direct);
                assert_eq!(direct.len(), a.len());
                for (k, want) in direct.iter().enumerate() {
                    let classes = a[k] + b[k] * y;
                    let nodes = j0[k] * u[0] + j1[k] * u[1];
                    for (what, got) in [("classes", classes), ("node weights", nodes)] {
                        assert!(
                            (got - want).norm() <= 1e-11 * scale,
                            "ymt {y}, helicity combination {}, flow {}: {what} give {got} \
                             against {want}",
                            k / n,
                            k % n
                        );
                    }
                }
            }
        }
        let reached = |c: usize| populated[c].iter().filter(|&&f| f).count();
        assert_eq!(
            (reached(0), reached(1)),
            (1, 2),
            "flows reached: {populated:?}"
        );
        assert!(
            (0..n).any(|f| populated[0][f] && populated[1][f]),
            "the classes share no flow: {populated:?}"
        );
    }

    /// The case the polynomial path exists for: SMEFTsim with one insertion per
    /// diagram and a basis grid in three coefficients — a top dipole (`ctWRe`, a new
    /// Lorentz structure), `cHt` (on the Standard Model's own `Z t t~` vertex) and
    /// `cHWB` (the input-scheme shift, on both fermion lines). `K = 1 + 3` monomials
    /// serve all eleven hypotheses, against direct evaluation.
    #[test]
    fn a_smeft_basis_grid_takes_one_evaluation_per_coefficient() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../validation/ufo/SMEFTsim_topU3l_MwScheme_UFO");
        let model = UFOModel::load(&dir, Some(&dir.join("restrict_massless.dat"))).unwrap();
        let base = EvaluatedModel::from_model(model.clone());
        let process = "e+ e- > t t~ NP<=1";
        let sets = sets(process, &model);
        let mut card = String::new();
        for (a, b, c) in [
            (0, 0, 0),
            (1, 0, 0),
            (0, 1, 0),
            (0, 0, 1),
            (1, 1, 0),
            (1, 0, 1),
            (0, 1, 1),
            (2, 0, 0),
            (0, 2, 0),
            (0, 0, 2),
            (-1, 1, 1),
        ] {
            card += &format!(
                "launch\n set ctWRe {a}\n set cHt {b}\n set cHWB {}\n",
                0.5 * f64::from(c)
            );
        }
        let launches = resolve(&card.parse().unwrap(), &model).unwrap();
        let names: Vec<String> = ["ctWRe", "cHt", "cHWB"].map(String::from).to_vec();
        let options = ReweightOptions {
            exact: false,
            couplings: Some(resolve_couplings(&model, &names).unwrap()),
        };
        let refs: Vec<&DiagramSet> = sets.iter().collect();
        let plan = ReweightPlan::new(&refs, &model, &base, launches, options).unwrap();
        let g = &plan.summary()[0].polynomial[0];
        assert_eq!((g.terms, g.evaluations, g.hypotheses), (4, 3, 11), "{g:?}");

        // No diagram spans several classes here. SMEFTsim puts `cHt` beside the
        // Standard Model's own couplings on one `Z t t~` vertex, but gives every
        // coefficient its own coupling order (`NPcHt`, `NPctW`, ...), so the vertex
        // splits into one interaction per order, as MadGraph splits it. A coupling
        // that is itself `a + b·c` would make a diagram span two classes; this
        // model has none.
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        let mut analysis = PolyAnalysis::new(&model, &names);
        let spanning = sets[0]
            .diagrams
            .iter()
            .filter(|d| {
                analysis
                    .amplitude_support(std::slice::from_ref(*d))
                    .is_some_and(|s| s.terms().count() > 1)
            })
            .count();
        assert_eq!((sets[0].diagrams.len(), spanning), (36, 0));

        let spread = check_against_direct(process, 500.0, &base, &plan, None, 1e-9);
        assert!(spread.1 / spread.0 > 1.2, "{spread:?}");
    }

    /// The node count is load-bearing: the ymt scan's `|M|²` has a quadratic
    /// term a straight line through the ends misses by far more than rounding.
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

    #[test]
    fn the_exact_path_agrees_with_direct_evaluation() {
        let (_, base, _, exact) = plan(
            "e+ e- > t t~ h",
            YMT_SCAN,
            ReweightOptions {
                exact: true,
                couplings: None,
            },
        );
        assert_eq!(exact.summary()[0].exact, 5);
        check_against_direct("e+ e- > t t~ h", 800.0, &base, &exact, None, 1e-12);
    }

    /// One hypothesis along a parameter is cheaper evaluated directly, and a
    /// hypothesis moving a parameter the process does not see changes nothing.
    #[test]
    fn the_plan_picks_the_cheaper_path() {
        let (_, _, _, one) = plan(
            "e+ e- > t t~ h",
            "launch\n set ymt 150\n",
            ReweightOptions::default(),
        );
        let s = &one.summary()[0];
        assert!(s.polynomial.is_empty());
        assert_eq!(s.exact, 1);

        let (model, base, sets, blind) = plan(
            "e+ e- > mu+ mu-",
            "launch\n set ymt 150\nlaunch\n set ymt 200\nlaunch\n set ymt 1\n",
            ReweightOptions::default(),
        );
        let s = &blind.summary()[0];
        assert_eq!((s.unchanged, s.exact, s.evaluations()), (3, 0, 1));
        let eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
        let mut rw = blind.bind();
        let mut out = Vec::new();
        rw.ratios(0, &points(&eval, &base, 300.0)[0], None, &mut out);
        assert_eq!(out, vec![1.0; 3]);
    }

    /// Without an explicit coupling set, a parameter that is not a polynomial
    /// coupling — the Z mass, also a propagator pole — and a launch moving two
    /// parameters take the exact path.
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
launch --rwgt_name=aew
 set aEWM1 130
";
        let (_, base, _, plan) = plan("e+ e- > t t~ h", card, ReweightOptions::default());
        let s = &plan.summary()[0];
        assert!(s.polynomial.is_empty(), "{s:?}");
        assert_eq!(s.exact, 5);
        check_against_direct("e+ e- > t t~ h", 900.0, &base, &plan, None, 1e-12);
    }

    /// A basis-point grid over two couplings, the way an effective-theory scan is
    /// laid out, served jointly. `ta+ ta- > t t~ h` radiates the Higgs off either
    /// fermion line, and an s-channel Higgs joins the two Yukawas in one diagram,
    /// so the amplitude carries `1`, `ymt`, `ymtau` and `ymt·ymtau` — and `ymb`,
    /// named in the set, enters nowhere.
    #[test]
    fn a_joint_coupling_grid_reproduces_direct_evaluation() {
        let mut card = String::new();
        for (i, ymt) in [0.0, 100.0, 250.0].iter().enumerate() {
            for (j, ymtau) in [0.0, 1.777, 5.0].iter().enumerate() {
                card +=
                    &format!("launch --rwgt_name=g{i}{j}\n set ymt {ymt}\n set ymtau {ymtau}\n");
            }
        }
        card += "launch --rwgt_name=only_tau\n set ymtau 3.0\nlaunch --rwgt_name=nothing\n";
        let (_, base, _, plan) = plan("ta+ ta- > t t~ h", &card, joint(&["ymt", "ymtau", "ymb"]));
        let s = &plan.summary()[0];
        assert_eq!(s.exact, 0);
        assert_eq!(s.polynomial.len(), 1);
        let g = &s.polynomial[0];
        assert_eq!(g.params, ["ymt", "ymtau"], "ymb is not a variable here");
        assert!(g.terms >= 4, "{g:?}");
        assert_eq!((g.evaluations, g.hypotheses), (g.terms - 1, 10));
        let spread = check_against_direct("ta+ ta- > t t~ h", 1200.0, &base, &plan, None, 1e-9);
        assert!(spread.1 / spread.0 > 2.0, "{spread:?}");
    }

    /// Ranges that do not reach zero are centred elsewhere, which needs the lower
    /// powers the support alone does not list; the result must not change.
    #[test]
    fn an_off_centre_range_is_closed_downwards() {
        let card = "launch\n set ymt 150\nlaunch\n set ymt 200\nlaunch\n set ymt 260\nlaunch\n set ymt 330\n";
        let (_, base, _, plan) = plan("e+ e- > t t~ h", card, joint(&["ymt"]));
        let g = &plan.summary()[0].polynomial[0];
        assert_eq!((g.terms, g.evaluations), (2, 1));
        check_against_direct("e+ e- > t t~ h", 1000.0, &base, &plan, None, 1e-11);

        let coords = Coordinates {
            active: vec![0, 1],
            center: vec![0.0, 5.0],
            half: vec![1.0, 1.0],
            lo: vec![-1.0, 4.0],
            hi: vec![1.0, 6.0],
        };
        let mut support = Support::empty(2);
        for t in [[2u8, 0u8], [0, 2]] {
            let mut one = Support::constant(2);
            for (i, &e) in t.iter().enumerate() {
                for _ in 0..e {
                    one = one.product(&Support::variable(2, i)).unwrap();
                }
            }
            support = support.union(&one);
        }
        assert_eq!(
            monomial_basis(&support, &coords),
            [vec![0, 0], vec![0, 1], vec![0, 2], vec![2, 0]]
        );
    }

    #[test]
    fn an_explicit_coupling_set_is_enforced() {
        let model = sm_model(SMRestrict::Default);
        let base = EvaluatedModel::from_model(model.clone());
        let sets = sets("e+ e- > mu+ mu-", &model);
        let refs: Vec<&DiagramSet> = sets.iter().collect();
        let launches = |text: &str| resolve(&text.parse().unwrap(), &model).unwrap();

        // A launch moving something outside the set.
        assert!(matches!(
            ReweightPlan::new(
                &refs,
                &model,
                &base,
                launches("launch\n set ymt 1\n set ymtau 2\n"),
                joint(&["ymt"])
            ),
            Err(ReweightError::OutsideCouplings { param, .. }) if param == "ymtau"
        ));
        // A coupling that is not polynomial in the process.
        assert!(matches!(
            ReweightPlan::new(
                &refs,
                &model,
                &base,
                launches("launch\n set MZ 90\n"),
                joint(&["MZ"])
            ),
            Err(ReweightError::NotPolynomial { .. })
        ));
        assert!(matches!(
            resolve_couplings(&model, &["aS".to_string()]),
            Err(ReweightError::BadCoupling { .. })
        ));
        assert!(matches!(
            resolve_couplings(&model, &["nosuch".to_string()]),
            Err(ReweightError::BadCoupling { .. })
        ));
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
        let plan =
            ReweightPlan::new(&refs, &model, &base, launches, ReweightOptions::default()).unwrap();
        check_against_direct("e+ e- > t t~ h", 1000.0, &base, &plan, None, 1e-10);

        // The check is load-bearing: pruned at the card, the amplitude has lost the
        // Yukawa diagrams and no longer moves with `ymt`.
        let eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
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

    /// The per-event strong coupling moves every node amplitude together, and the
    /// ratio is the one at that coupling.
    #[test]
    fn ratios_are_taken_at_the_events_strong_coupling() {
        let card = "launch\n set ymt 0\nlaunch\n set ymt 120\nlaunch\n set ymt 250\nlaunch\n set ymt 300\n";
        let (_, base, _, plan) = plan("g g > t t~ h", card, ReweightOptions::default());
        assert_eq!(plan.summary()[0].polynomial.len(), 1);
        check_against_direct("g g > t t~ h", 900.0, &base, &plan, Some(0.09), 1e-10);
    }
}
