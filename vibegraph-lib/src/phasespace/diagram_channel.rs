//! Per-diagram phase-space channel: a recursive 2-body-decomposition of the
//! `n`-body final state read off a [`Diagram`]'s propagator chain.
//!
//! A tree Feynman diagram organises its final state into a nested set of
//! subsystems. Each timelike (s-channel) internal line bounds one subsystem — the
//! final-state legs on the beam-free side of the cut it makes — and those
//! subsystems form a laminar family, i.e. a tree. Reading that subsystem off the
//! stored momentum takes care: feyngraph's momentum routing eliminates the
//! highest-indexed external, so the beam-free side is the complementary final
//! state whenever the stored coefficients happen to carry both beams (see
//! [`subsystem_mask`]). [`DiagramChannel`] turns that tree into a chain of 2-body decays: the
//! total system `(√ŝ, 0, 0, 0)` splits into two daughters, each daughter with a
//! fixed mass (a single outgoing particle) or a sampled invariant mass (a
//! composite subsystem), and each composite daughter recurses.
//!
//! A timelike (s-channel) invariant whose subsystem carries a finite-width pole is
//! drawn through the Breit–Wigner tan-substitution `s = m² + mΓ·tan θ`, so the
//! sampling density concentrates on the resonance as `1/((s−m²)²+(mΓ)²)`; a
//! subsystem with no pole (or a zero-width/massless one) keeps the flat draw over
//! its kinematic range. The importance map for a spacelike (t-channel) transfer is
//! a separate concern — such lines drive no node in this all-timelike decay tree
//! and are kept as metadata only. Each node records the propagator particle's mass
//! and width, so the resonance-aware draw of a chosen invariant slots in without
//! changing the tree.
//!
//! The weight is the exact product of the 2-body LIPS factors `R_2 = π|p*|/√s`
//! and the flat invariant-range measures, so a flat Monte-Carlo average of
//! `weight · f` estimates `∫ dR_n f` over the same invariant volume `R_n` that
//! flat RAMBO integrates — the channel is a different parametrisation of the same
//! phase space.

use std::collections::BTreeMap;

use crate::diagrams::diagram::Diagram;
use crate::helas::repr::lorentz::LorentzVector;
use crate::helas::repr::Real;
use crate::ufo::EvaluatedModel;

use super::channel::{Channel, PhaseSpaceMap, PhaseSpacePoint};

/// The propagator pole a subsystem's invariant sits on: the timelike line's mass
/// and width, driving the Breit–Wigner importance map for that invariant.
#[derive(Clone, Copy, Debug)]
pub struct Resonance<F: Real> {
    pub mass: F,
    pub width: F,
}

/// A spacelike (t-channel) internal line: its propagator mass and width, kept for
/// a later t-channel importance map. Its invariant is a momentum transfer, not a
/// subsystem mass, so it drives no node in the flat decay tree here.
#[derive(Clone, Copy, Debug)]
pub struct TChannel<F: Real> {
    pub mass: F,
    pub width: F,
}

/// A node of the decay tree: either a single outgoing particle of fixed mass, or
/// a 2-body split of a composite system.
#[derive(Clone, Debug)]
enum Node<F: Real> {
    Leaf { slot: usize, mass: F },
    Branch(Box<Branch<F>>),
}

#[derive(Clone, Debug)]
struct Branch<F: Real> {
    left: Node<F>,
    right: Node<F>,
    /// Sum of the subtree's leaf masses — the minimal invariant mass of the
    /// system this branch decays.
    mu: F,
    /// The s-channel propagator whose pole sits on this branch's invariant, if the
    /// diagram has one. `None` for the root (invariant fixed at √ŝ) and for the
    /// auxiliary branches introduced when a vertex has more than two subsystems.
    resonance: Option<Resonance<F>>,
}

impl<F: Real> Node<F> {
    fn mu(&self) -> F {
        match self {
            Node::Leaf { mass, .. } => *mass,
            Node::Branch(b) => b.mu,
        }
    }
}

/// A single-diagram phase-space channel on a fixed `√ŝ` and outgoing-mass set.
#[derive(Clone, Debug)]
pub struct DiagramChannel<F: Real> {
    sqrt_s: F,
    n_out: usize,
    root: Branch<F>,
    t_channels: Vec<TChannel<F>>,
}

impl<F: Real> DiagramChannel<F> {
    /// Build the channel from a diagram's propagator chain at CM energy `sqrt_s`.
    ///
    /// Outgoing-leg masses and each internal line's mass/width are read from
    /// `model`. Only meaningful for a `2 → n` process; the beams are externals
    /// `0..n_in`.
    pub fn from_diagram(diagram: &Diagram, model: &EvaluatedModel, sqrt_s: F) -> Self {
        let n_in = diagram.n_in;
        let n_ext = diagram.n_ext();
        let n_out = n_ext - n_in;
        assert!(n_out >= 2, "a 2-body decomposition needs at least two legs");

        let masses: Vec<F> = (0..n_out)
            .map(|slot| {
                let particle = diagram.legs[n_in + slot].particle;
                cast(model.mass(particle))
            })
            .collect();

        // Subsystems bounded by timelike, beam-free internal lines, plus the
        // spacelike lines kept aside for a later t-channel map.
        let mut resonances: BTreeMap<u64, Resonance<F>> = BTreeMap::new();
        let mut t_channels = Vec::new();
        for prop in &diagram.props {
            if prop.is_spacelike(n_in) {
                t_channels.push(TChannel {
                    mass: cast(model.mass(prop.particle)),
                    width: cast(model.width(prop.particle)),
                });
                continue;
            }
            if let Some(mask) = subsystem_mask(&prop.momentum, n_in, n_ext) {
                resonances.entry(mask).or_insert(Resonance {
                    mass: cast(model.mass(prop.particle)),
                    width: cast(model.width(prop.particle)),
                });
            }
        }

        let subsystems: Vec<u64> = resonances.keys().copied().collect();
        let root = build_root(n_out, &masses, &subsystems, &resonances);
        DiagramChannel {
            sqrt_s,
            n_out,
            root,
            t_channels,
        }
    }

    /// Build a channel directly from an explicit subsystem list — the same tree
    /// construction as [`from_diagram`](Self::from_diagram) without a diagram, for
    /// exercising a controlled topology. Each entry of `subsystems` is a set of
    /// outgoing-leg slots (`0..masses.len()`) that share an s-channel line.
    pub fn from_topology(sqrt_s: F, masses: Vec<F>, subsystems: &[Vec<usize>]) -> Self {
        let n_out = masses.len();
        assert!(n_out >= 2, "a 2-body decomposition needs at least two legs");
        let masks: Vec<u64> = subsystems
            .iter()
            .map(|s| s.iter().fold(0u64, |m, &i| m | (1 << i)))
            .collect();
        let root = build_root(n_out, &masses, &masks, &BTreeMap::new());
        DiagramChannel {
            sqrt_s,
            n_out,
            root,
            t_channels: Vec::new(),
        }
    }

    /// Build a channel from an explicit subsystem list, attaching an optional
    /// [`Resonance`] to each subsystem so its invariant is Breit–Wigner-mapped — the
    /// same tree as [`from_topology`](Self::from_topology) but with resonance-aware
    /// invariant draws, for exercising the pole map on a controlled topology.
    pub fn from_topology_resonant(
        sqrt_s: F,
        masses: Vec<F>,
        subsystems: &[(Vec<usize>, Option<Resonance<F>>)],
    ) -> Self {
        let n_out = masses.len();
        assert!(n_out >= 2, "a 2-body decomposition needs at least two legs");
        let mut masks = Vec::with_capacity(subsystems.len());
        let mut resonances: BTreeMap<u64, Resonance<F>> = BTreeMap::new();
        for (slots, res) in subsystems {
            let mask = slots.iter().fold(0u64, |m, &i| m | (1 << i));
            masks.push(mask);
            if let Some(r) = res {
                resonances.insert(mask, *r);
            }
        }
        let root = build_root(n_out, &masses, &masks, &resonances);
        DiagramChannel {
            sqrt_s,
            n_out,
            root,
            t_channels: Vec::new(),
        }
    }

    /// Number of outgoing momenta the channel produces.
    pub fn n_out(&self) -> usize {
        self.n_out
    }

    /// The s-channel propagator poles the sampled invariants sit on, for a
    /// resonance-aware invariant map.
    pub fn resonances(&self) -> Vec<Resonance<F>> {
        let mut out = Vec::new();
        collect_resonances(&self.root, &mut out);
        out
    }

    /// The spacelike (t-channel) lines of the diagram, for a t-channel importance
    /// map.
    pub fn t_channels(&self) -> &[TChannel<F>] {
        &self.t_channels
    }
}

impl<F: Real> PhaseSpaceMap<F> for DiagramChannel<F> {
    fn ndim(&self) -> usize {
        3 * self.n_out - 4
    }

    fn sample(&self, u: &[F]) -> PhaseSpacePoint<F> {
        let s = self.sqrt_s * self.sqrt_s;
        let total = LorentzVector::new(self.sqrt_s, F::zero(), F::zero(), F::zero());
        let mut slots: Vec<Option<LorentzVector<F>>> = vec![None; self.n_out];
        let mut cursor = 0usize;
        sample_branch(&self.root, s, total, u, &mut cursor, &mut slots);
        let momenta: Vec<LorentzVector<F>> = slots
            .into_iter()
            .map(|m| m.expect("every outgoing slot is filled"))
            .collect();
        // Reciprocal of the density at the realised configuration: exactly the
        // phase-space Jacobian this walk carried.
        let weight = F::one() / self.density(&momenta);
        PhaseSpacePoint { momenta, weight }
    }
}

impl<F: Real> Channel<F> for DiagramChannel<F> {
    fn density(&self, momenta: &[LorentzVector<F>]) -> F {
        let s = self.sqrt_s * self.sqrt_s;
        F::one() / branch_jacobian(&self.root, s, momenta)
    }
}

// ── Tree construction ────────────────────────────────────────────────────────

/// The set of outgoing-leg slots a timelike (s-channel) internal line bounds, as a
/// bitmask over `0..n_out`. `None` for a spacelike (t-channel) transfer, for the
/// s-channel core (whose subsystem is the whole final state), and for a line that
/// bounds a single leg rather than a composite subsystem.
///
/// The stored `momentum` is not a plain "which externals sit on this side"
/// indicator: feyngraph routes momenta by eliminating the highest-indexed
/// external through global conservation, so a stored coefficient vector is the
/// signed combination for the cut side *away from* that external. Its raw beam
/// coefficients are therefore gauge-dependent and cannot be read as "is this the
/// beam side". The convention-robust classifier is the beam content of the cut: a
/// genuine final-state s-channel subsystem is the side of the cut carrying *no*
/// beam. That zero-beam side is the stored side when the stored coefficients touch
/// no beam, and the complementary final-state set when they touch every beam (the
/// non-prefix case the elimination produces). A cut whose two sides each carry a
/// beam is a spacelike momentum transfer and bounds no subsystem here.
fn subsystem_mask(momentum: &[i8], n_in: usize, n_ext: usize) -> Option<u64> {
    let n_out = n_ext - n_in;
    let beams = momentum[..n_in].iter().filter(|&&c| c != 0).count();
    let mut stored = 0u64;
    for i in n_in..n_ext {
        if momentum[i] != 0 {
            stored |= 1 << (i - n_in);
        }
    }
    let full = if n_out >= 64 {
        u64::MAX
    } else {
        (1u64 << n_out) - 1
    };
    let subsystem = if beams == 0 {
        stored
    } else if beams == n_in {
        full & !stored
    } else {
        return None;
    };
    let count = subsystem.count_ones() as usize;
    if count >= 2 && count < n_out {
        Some(subsystem)
    } else {
        None
    }
}

fn build_root<F: Real>(
    n_out: usize,
    masses: &[F],
    subsystems: &[u64],
    resonances: &BTreeMap<u64, Resonance<F>>,
) -> Branch<F> {
    let universe = if n_out >= 64 {
        u64::MAX
    } else {
        (1u64 << n_out) - 1
    };
    match build_node(universe, masses, subsystems, resonances) {
        Node::Branch(b) => *b,
        Node::Leaf { .. } => unreachable!("the full outgoing set has at least two legs"),
    }
}

/// Build the subtree spanning the outgoing-leg set `mask`.
fn build_node<F: Real>(
    mask: u64,
    masses: &[F],
    subsystems: &[u64],
    resonances: &BTreeMap<u64, Resonance<F>>,
) -> Node<F> {
    if mask.count_ones() == 1 {
        let slot = mask.trailing_zeros() as usize;
        return Node::Leaf {
            slot,
            mass: masses[slot],
        };
    }

    // Children: the maximal candidate sets strictly inside `mask`. Candidates are
    // the diagram subsystems plus every singleton; a set is maximal when no other
    // candidate strictly inside `mask` contains it. For a tree diagram these
    // partition `mask`.
    let mut candidates: Vec<u64> = subsystems
        .iter()
        .copied()
        .filter(|&s| s != mask && s & mask == s)
        .collect();
    for i in 0..64 {
        let bit = 1u64 << i;
        if mask & bit != 0 {
            candidates.push(bit);
        }
    }
    let mut children: Vec<u64> = candidates
        .iter()
        .copied()
        .filter(|&c| {
            !candidates
                .iter()
                .any(|&other| other != c && other != mask && c & other == c)
        })
        .collect();
    children.sort_unstable();
    children.dedup();

    let child_nodes: Vec<Node<F>> = children
        .iter()
        .map(|&c| build_node(c, masses, subsystems, resonances))
        .collect();

    let resonance = resonances.get(&mask).copied();
    binarize(child_nodes, resonance)
}

/// Fold a vertex's children into a binary chain. A 2-body vertex maps directly; a
/// higher vertex becomes a right-leaning caterpillar whose interior branches carry
/// an auxiliary invariant (no propagator pole). The outermost branch carries the
/// subsystem's own resonance, since its invariant is the subsystem mass.
fn binarize<F: Real>(mut children: Vec<Node<F>>, resonance: Option<Resonance<F>>) -> Node<F> {
    assert!(
        children.len() >= 2,
        "a composite subsystem splits in two or more"
    );
    let left = children.remove(0);
    let right = if children.len() == 1 {
        children.remove(0)
    } else {
        binarize(children, None)
    };
    let mu = left.mu() + right.mu();
    Node::Branch(Box::new(Branch {
        left,
        right,
        mu,
        resonance,
    }))
}

fn collect_resonances<F: Real>(branch: &Branch<F>, out: &mut Vec<Resonance<F>>) {
    if let Some(r) = branch.resonance {
        out.push(r);
    }
    if let Node::Branch(b) = &branch.left {
        collect_resonances(b, out);
    }
    if let Node::Branch(b) = &branch.right {
        collect_resonances(b, out);
    }
}

// ── Sampling & Jacobian ──────────────────────────────────────────────────────

/// Källén function `λ(a,b,c) = a²+b²+c²−2(ab+bc+ca)`.
fn kallen<F: Real>(a: F, b: F, c: F) -> F {
    a * a + b * b + c * c - (F::one() + F::one()) * (a * b + b * c + c * a)
}

/// CM momentum magnitude of a 2-body split of invariant `s` into masses² `sl`,`sr`.
fn p_star<F: Real>(s: F, sqrt_s: F, sl: F, sr: F) -> F {
    let two = F::one() + F::one();
    kallen(s, sl, sr).max(F::zero()).sqrt() / (two * sqrt_s)
}

/// The 2-body LIPS factor `R_2 = π|p*|/√s`.
fn r2_factor<F: Real>(s: F, sqrt_s: F, sl: F, sr: F) -> F {
    if sqrt_s > F::zero() {
        F::PI() * p_star(s, sqrt_s, sl, sr) / sqrt_s
    } else {
        F::zero()
    }
}

/// Boost a rest-frame vector into the CM frame of a system with lab momentum
/// `p_lab`, guarding the `β = p⃗/E` division against a degenerate (`E → 0` or
/// numerically superluminal) subsystem — where the vector being boosted is already
/// zero, so no boost is needed.
fn safe_boost<F: Real>(v: LorentzVector<F>, p_lab: LorentzVector<F>) -> LorentzVector<F> {
    let e = p_lab.e();
    if e <= F::zero() {
        return v;
    }
    let beta = [p_lab.px() / e, p_lab.py() / e, p_lab.pz() / e];
    let b2 = beta[0] * beta[0] + beta[1] * beta[1] + beta[2] * beta[2];
    if b2 >= F::one() {
        return v;
    }
    v.boost(beta)
}

/// The Breit–Wigner scale `(m², mΓ)` a resonance imposes on its invariant draw,
/// or `None` when the pole cannot shape the draw — no resonance, or a
/// zero-width/massless pole (`mΓ ≤ 0`), for which the flat draw stands.
fn bw_scale<F: Real>(res: Option<Resonance<F>>) -> Option<(F, F)> {
    let r = res?;
    let mg = r.mass * r.width;
    if mg > F::zero() {
        Some((r.mass * r.mass, mg))
    } else {
        None
    }
}

/// Map `x ∈ [0,1]` to an invariant `s ∈ [lo, hi]`. A finite-width pole importance-
/// samples the relativistic Breit–Wigner via `s = m² + mΓ·tan θ`, with `θ` uniform
/// over `[atan((lo−m²)/mΓ), atan((hi−m²)/mΓ)]`; otherwise the draw is flat.
fn draw_invariant<F: Real>(lo: F, hi: F, res: Option<Resonance<F>>, x: F) -> F {
    match bw_scale(res) {
        Some((m2, mg)) => {
            let theta_lo = ((lo - m2) / mg).atan();
            let theta_hi = ((hi - m2) / mg).atan();
            let theta = theta_lo + (theta_hi - theta_lo) * x;
            m2 + mg * theta.tan()
        }
        None => lo + (hi - lo) * x,
    }
}

/// The invariant-draw measure `ds/dx` at the realised `s`: the flat range length
/// `hi − lo`, or, for the Breit–Wigner map, `[(s−m²)²/(mΓ) + mΓ]·(θ_hi−θ_lo)` — the
/// exact `ds/dθ · dθ/dx`, whose reciprocal is a sampling density `∝ BW(s)`.
fn invariant_measure<F: Real>(lo: F, hi: F, res: Option<Resonance<F>>, s: F) -> F {
    match bw_scale(res) {
        Some((m2, mg)) => {
            let theta_lo = ((lo - m2) / mg).atan();
            let theta_hi = ((hi - m2) / mg).atan();
            let d = s - m2;
            (d * d / mg + mg) * (theta_hi - theta_lo)
        }
        None => hi - lo,
    }
}

/// Draw the invariants and angles of one 2-body split and recurse into composite
/// daughters. `s` is the (fixed) invariant mass² of the system this branch
/// decays; `p_lab` is its four-momentum in the CM frame.
fn sample_branch<F: Real>(
    branch: &Branch<F>,
    s: F,
    p_lab: LorentzVector<F>,
    u: &[F],
    cursor: &mut usize,
    slots: &mut [Option<LorentzVector<F>>],
) {
    let two = F::one() + F::one();
    let sqrt_s = s.sqrt();
    let mu_l = branch.left.mu();
    let mu_r = branch.right.mu();

    let sl = match &branch.left {
        Node::Leaf { mass, .. } => *mass * *mass,
        Node::Branch(b) => {
            let lo = mu_l * mu_l;
            let hi = (sqrt_s - mu_r).powi(2);
            let x = u[*cursor];
            *cursor += 1;
            draw_invariant(lo, hi, b.resonance, x)
        }
    };
    let sqrt_sl = sl.sqrt();
    let sr = match &branch.right {
        Node::Leaf { mass, .. } => *mass * *mass,
        Node::Branch(b) => {
            let lo = mu_r * mu_r;
            let hi = (sqrt_s - sqrt_sl).powi(2);
            let x = u[*cursor];
            *cursor += 1;
            draw_invariant(lo, hi, b.resonance, x)
        }
    };

    let cos = two * u[*cursor] - F::one();
    *cursor += 1;
    let phi = two * F::PI() * u[*cursor];
    *cursor += 1;
    let sin = (F::one() - cos * cos).max(F::zero()).sqrt();
    let (dx, dy, dz) = (sin * phi.cos(), sin * phi.sin(), cos);

    // A subsystem invariant sampled at its degenerate lower edge (`√s → 0`, or a
    // sibling taking all the energy) makes the rest-frame split and the boost
    // singular. Such points sit at the phase-space boundary where the upstream
    // `p*` — and thus the weight — already vanishes, so it suffices to keep the
    // momenta finite: the split contributes nothing to the integral.
    let (e_l, e_r, pstar) = if sqrt_s > F::zero() {
        let two_sqrt_s = two * sqrt_s;
        (
            (s + sl - sr) / two_sqrt_s,
            (s + sr - sl) / two_sqrt_s,
            p_star(s, sqrt_s, sl, sr),
        )
    } else {
        (F::zero(), F::zero(), F::zero())
    };
    let pl_rest = LorentzVector::new(e_l, pstar * dx, pstar * dy, pstar * dz);
    let pr_rest = LorentzVector::new(e_r, -pstar * dx, -pstar * dy, -pstar * dz);

    let pl = safe_boost(pl_rest, p_lab);
    let pr = safe_boost(pr_rest, p_lab);

    match &branch.left {
        Node::Leaf { slot, .. } => slots[*slot] = Some(pl),
        Node::Branch(b) => sample_branch(b, sl, pl, u, cursor, slots),
    }
    match &branch.right {
        Node::Leaf { slot, .. } => slots[*slot] = Some(pr),
        Node::Branch(b) => sample_branch(b, sr, pr, u, cursor, slots),
    }
}

/// Total four-momentum of a node's subtree, summed over its leaves.
fn subtree_momentum<F: Real>(node: &Node<F>, momenta: &[LorentzVector<F>]) -> LorentzVector<F> {
    match node {
        Node::Leaf { slot, .. } => momenta[*slot],
        Node::Branch(b) => {
            let l = subtree_momentum(&b.left, momenta);
            let r = subtree_momentum(&b.right, momenta);
            LorentzVector::new(
                l.e() + r.e(),
                l.px() + r.px(),
                l.py() + r.py(),
                l.pz() + r.pz(),
            )
        }
    }
}

fn node_invariant<F: Real>(node: &Node<F>, momenta: &[LorentzVector<F>]) -> F {
    match node {
        Node::Leaf { mass, .. } => *mass * *mass,
        // A composite subsystem is timelike; clamp away the tiny negative `m²` a
        // near-threshold configuration can pick up so `√s` stays real.
        Node::Branch(_) => subtree_momentum(node, momenta).m2().max(F::zero()),
    }
}

/// The product of 2-body LIPS factors and flat invariant-range measures for the
/// subtree rooted at `branch`, evaluated at `momenta`. Its reciprocal is the
/// channel density; the sampler's weight is the reciprocal of the density, so the
/// two are exact inverses at any generated point.
fn branch_jacobian<F: Real>(branch: &Branch<F>, s: F, momenta: &[LorentzVector<F>]) -> F {
    let sqrt_s = s.sqrt();
    let mu_l = branch.left.mu();
    let mu_r = branch.right.mu();
    let sl = node_invariant(&branch.left, momenta);
    let sqrt_sl = sl.sqrt();
    let sr = node_invariant(&branch.right, momenta);

    let mut f = F::one();
    if let Node::Branch(b) = &branch.left {
        let lo = mu_l * mu_l;
        let hi = (sqrt_s - mu_r).powi(2);
        f = f * invariant_measure(lo, hi, b.resonance, sl);
    }
    if let Node::Branch(b) = &branch.right {
        let lo = mu_r * mu_r;
        let hi = (sqrt_s - sqrt_sl).powi(2);
        f = f * invariant_measure(lo, hi, b.resonance, sr);
    }
    f = f * r2_factor(s, sqrt_s, sl, sr);
    if let Node::Branch(b) = &branch.left {
        f = f * branch_jacobian(b, sl, momenta);
    }
    if let Node::Branch(b) = &branch.right {
        f = f * branch_jacobian(b, sr, momenta);
    }
    f
}

fn cast<F: Real>(x: f64) -> F {
    F::from(x).expect("mass/width fits the scalar field")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::diagram::{Diagram, LegIdx, Ray};
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::phasespace::rambo::massless_volume;
    use crate::phasespace::rng::SubStream;
    use crate::phasespace::RamboChannel;
    use crate::ufo::sm::{sm_model, SMRestrict};
    use crate::ufo::EvaluatedModel;

    /// A spread of topologies for the volume and kinematics checks, each as
    /// `(√ŝ, masses, subsystems)`.
    fn topologies() -> Vec<(f64, Vec<f64>, Vec<Vec<usize>>)> {
        vec![
            // 2→2, no internal subsystem.
            (500.0, vec![0.0, 0.0], vec![]),
            // 2→3 caterpillar: {1,2} pair off the total.
            (500.0, vec![0.0, 0.0, 0.0], vec![vec![1, 2]]),
            // 2→4 balanced: {0,1} and {2,3}.
            (600.0, vec![0.0; 4], vec![vec![0, 1], vec![2, 3]]),
            // 2→4 caterpillar: {1,2,3} then {2,3}.
            (600.0, vec![0.0; 4], vec![vec![1, 2, 3], vec![2, 3]]),
            // 2→5 nested.
            (
                700.0,
                vec![0.0; 5],
                vec![vec![1, 2, 3, 4], vec![3, 4], vec![1, 2]],
            ),
            // 2→6 two W-like pairs plus a nested pair.
            (
                800.0,
                vec![0.0; 6],
                vec![vec![0, 1], vec![2, 3, 4, 5], vec![2, 3], vec![4, 5]],
            ),
            // Massive 2→4: a heavy pair recoiling against two light legs.
            (600.0, vec![80.4, 80.4, 5.0, 5.0], vec![vec![0, 1]]),
        ]
    }

    fn total<F: Real>(momenta: &[LorentzVector<F>]) -> [F; 4] {
        momenta.iter().fold([F::zero(); 4], |a, p| {
            [a[0] + p.e(), a[1] + p.px(), a[2] + p.py(), a[3] + p.pz()]
        })
    }

    /// Every channel emits on-shell, momentum-conserving points across seeds and
    /// topologies.
    #[test]
    fn on_shell_and_conserving_fuzz() {
        let mut stream = SubStream::from_stream(0xD1A6, 2);
        for (sqrt_s, masses, subs) in topologies() {
            let ch = DiagramChannel::from_topology(sqrt_s, masses.clone(), &subs);
            assert_eq!(ch.ndim(), 3 * masses.len() - 4);
            for _ in 0..200 {
                let u = stream.uniforms::<f64>(ch.ndim());
                let pt = ch.sample(&u);
                assert_eq!(pt.momenta.len(), masses.len());
                let tot = total(&pt.momenta);
                assert!(
                    (tot[0] - sqrt_s).abs() < 1e-7 * sqrt_s,
                    "energy not conserved: {} vs {sqrt_s}",
                    tot[0]
                );
                for c in &tot[1..] {
                    assert!(c.abs() < 1e-7 * sqrt_s, "3-momentum not conserved: {c}");
                }
                for (p, &m) in pt.momenta.iter().zip(&masses) {
                    let scale = sqrt_s * sqrt_s;
                    assert!(
                        (p.m2() - m * m).abs() < 1e-6 * scale + 1e-6,
                        "off shell: m² = {} want {}",
                        p.m2(),
                        m * m
                    );
                    assert!(p.e() > 0.0 && p.e().is_finite());
                }
                assert!(pt.weight > 0.0 && pt.weight.is_finite());
            }
        }
    }

    /// The channel density is the exact reciprocal of the weight it assigns to a
    /// point it generated.
    #[test]
    fn density_is_reciprocal_weight() {
        let mut stream = SubStream::from_stream(0xD1A7, 5);
        for (sqrt_s, masses, subs) in topologies() {
            let ch = DiagramChannel::from_topology(sqrt_s, masses.clone(), &subs);
            for _ in 0..50 {
                let u = stream.uniforms::<f64>(ch.ndim());
                let pt = ch.sample(&u);
                assert_eq!(ch.density(&pt.momenta), 1.0 / pt.weight);
            }
        }
    }

    /// Monte-Carlo estimate of a channel's flat-weight average, with its standard
    /// error.
    fn mc_volume(ch: &DiagramChannel<f64>, seed: u64, n: usize) -> (f64, f64) {
        let mut stream = SubStream::from_stream(seed, 11);
        let ndim = ch.ndim();
        let mut sum = 0.0;
        let mut sum_sq = 0.0;
        for _ in 0..n {
            let u = stream.uniforms::<f64>(ndim);
            let w = ch.sample(&u).weight;
            sum += w;
            sum_sq += w * w;
        }
        let mean = sum / n as f64;
        let var = (sum_sq / n as f64 - mean * mean).max(0.0);
        (mean, (var / n as f64).sqrt())
    }

    /// The flat channel Jacobian reproduces the analytic massless phase-space
    /// volume `V_n` for 2→2 up to 2→6 topologies: the channel is a different
    /// parametrisation of the same invariant volume, so its flat integral matches.
    #[test]
    fn flat_volume_matches_massless_v_n() {
        for (sqrt_s, masses, subs) in topologies() {
            if masses.iter().any(|&m| m != 0.0) {
                continue;
            }
            let n = masses.len();
            let ch = DiagramChannel::from_topology(sqrt_s, masses.clone(), &subs);
            let (mean, err) = mc_volume(&ch, 0xF00D, 400_000);
            let analytic: f64 = massless_volume(sqrt_s, n);
            // A 2→2 channel has a constant weight (zero variance); fall back to a
            // tight relative bound there and use the MC error bar otherwise.
            let tol = (5.0 * err).max(1e-9 * analytic);
            eprintln!(
                "n={n} sub={subs:?}: channel V_n = {mean:.6e} ± {err:.2e}, \
                 analytic {analytic:.6e}, diff {:.2e}",
                (mean - analytic).abs()
            );
            assert!(
                (mean - analytic).abs() < tol,
                "n={n}: channel V_n {mean:.6e} ± {err:.2e} vs analytic {analytic:.6e}"
            );
        }
    }

    /// Known-wrong-tripping cross-check: the channel and flat RAMBO estimate the
    /// same volume `V_n` on the same masses, so their flat-MC estimates must agree
    /// within the combined MC error. A wrong invariant ordering or 2-body Jacobian
    /// would separate them.
    #[test]
    fn flat_volume_matches_flat_rambo() {
        for (sqrt_s, masses, subs) in topologies() {
            let n = masses.len();
            let ch = DiagramChannel::from_topology(sqrt_s, masses.clone(), &subs);
            let (m_ch, e_ch) = mc_volume(&ch, 0xBEEF, 400_000);

            let rambo = RamboChannel::new(sqrt_s, masses.clone());
            let mut stream = SubStream::from_stream(0xBEE5, 7);
            let mut sum = 0.0;
            let mut sum_sq = 0.0;
            let np = 400_000;
            for _ in 0..np {
                let u = stream.uniforms::<f64>(4 * n);
                let w = rambo.sample(&u).weight;
                sum += w;
                sum_sq += w * w;
            }
            let m_rb = sum / np as f64;
            let var = (sum_sq / np as f64 - m_rb * m_rb).max(0.0);
            let e_rb = (var / np as f64).sqrt();

            let err = (e_ch * e_ch + e_rb * e_rb).sqrt();
            let tol = (5.0 * err).max(1e-9 * m_rb);
            eprintln!(
                "n={n} sub={subs:?}: channel {m_ch:.6e} ± {e_ch:.2e} vs \
                 RAMBO {m_rb:.6e} ± {e_rb:.2e} (diff {:.2e})",
                (m_ch - m_rb).abs()
            );
            assert!(
                (m_ch - m_rb).abs() < tol,
                "n={n}: channel V_n {m_ch:.6e} disagrees with flat RAMBO {m_rb:.6e}"
            );
        }
    }

    /// Z-boson pole parameters used across the resonance tests.
    const M_Z: f64 = 91.1876;
    const G_Z: f64 = 2.4952;

    fn z_resonance() -> Resonance<f64> {
        Resonance {
            mass: M_Z,
            width: G_Z,
        }
    }

    /// Invariant mass² of the outgoing pair `(0, 1)`.
    fn s01(momenta: &[LorentzVector<f64>]) -> f64 {
        let (a, b) = (&momenta[0], &momenta[1]);
        let e = a.e() + b.e();
        let px = a.px() + b.px();
        let py = a.py() + b.py();
        let pz = a.pz() + b.pz();
        e * e - px * px - py * py - pz * pz
    }

    /// The Breit–Wigner draw is measure-preserving: averaging its `ds/dx` over
    /// uniform `x` reproduces the flat range integral `∫ ds = hi − lo`, since the
    /// Jacobian exactly cancels the sampling density. A wrong `ds/dθ` misses it.
    #[test]
    fn bw_map_is_measure_preserving() {
        let res = Some(z_resonance());
        let (lo, hi) = (0.0_f64, 250_000.0);
        let mut stream = SubStream::from_stream(0xB011, 9);
        let n = 400_000;
        let (mut sum, mut sum_sq) = (0.0, 0.0);
        for _ in 0..n {
            let x = stream.uniforms::<f64>(1)[0];
            let s = draw_invariant(lo, hi, res, x);
            let w = invariant_measure(lo, hi, res, s);
            sum += w;
            sum_sq += w * w;
        }
        let mean = sum / n as f64;
        let err = ((sum_sq / n as f64 - mean * mean).max(0.0) / n as f64).sqrt();
        eprintln!("BW ∫ds = {mean:.6e} ± {err:.2e}, want {:.6e}", hi - lo);
        assert!(
            (mean - (hi - lo)).abs() < 5.0 * err,
            "BW measure not preserving: ∫ds = {mean:.6e} ± {err:.2e} vs {:.6e}",
            hi - lo
        );
    }

    /// The Breit–Wigner map turns `∫ ds · BW(s)` into a constant integrand: with
    /// the exact `ds/dθ`, `measure(s)·BW(s)` equals the analytic
    /// `∫BW ds = (θ_hi−θ_lo)/mΓ` at every `x`, so the estimator has zero variance.
    /// A wrong Jacobian breaks the constancy — the sharpest pin on `ds/dθ`.
    #[test]
    fn bw_map_zero_variance_on_bw_integrand() {
        let res = Some(z_resonance());
        let (lo, hi) = (100.0_f64, 250_000.0);
        let (m2, mg) = (M_Z * M_Z, M_Z * G_Z);
        let bw = |s: f64| 1.0 / ((s - m2).powi(2) + mg * mg);
        let analytic = (((hi - m2) / mg).atan() - ((lo - m2) / mg).atan()) / mg;
        for k in 0..=40 {
            let x = k as f64 / 40.0;
            let s = draw_invariant(lo, hi, res, x);
            let est = invariant_measure(lo, hi, res, s) * bw(s);
            assert!(
                (est - analytic).abs() < 1e-12 * analytic,
                "measure·BW = {est:.12e} not constant at analytic ∫BW = {analytic:.12e}"
            );
        }
    }

    /// Importance sampling reshapes variance, not volume: a channel with
    /// Breit–Wigner invariant maps installed still integrates to the massless
    /// phase-space volume `V_n`. A draw/measure mismatch would bias this.
    #[test]
    fn resonant_channel_volume_still_v_n() {
        let z = Some(z_resonance());
        let cases: Vec<(f64, Vec<f64>, Vec<(Vec<usize>, Option<Resonance<f64>>)>)> = vec![
            (500.0, vec![0.0; 3], vec![(vec![0, 1], z)]),
            (
                600.0,
                vec![0.0; 4],
                vec![(vec![0, 1], z), (vec![2, 3], None)],
            ),
            // A Breit–Wigner on the *second* pair — the non-prefix subsystem the
            // routing-aware classifier recovers — must be volume-neutral too.
            (
                600.0,
                vec![0.0; 4],
                vec![(vec![0, 1], None), (vec![2, 3], z)],
            ),
            (
                700.0,
                vec![0.0; 5],
                vec![
                    (vec![1, 2, 3, 4], None),
                    (vec![3, 4], z),
                    (vec![1, 2], None),
                ],
            ),
        ];
        for (sqrt_s, masses, subs) in cases {
            let n = masses.len();
            let ch = DiagramChannel::from_topology_resonant(sqrt_s, masses.clone(), &subs);
            let (mean, err) = mc_volume(&ch, 0xF11D, 2_000_000);
            let analytic: f64 = massless_volume(sqrt_s, n);
            let tol = (6.0 * err).max(1e-9 * analytic);
            eprintln!(
                "n={n}: resonant V_n = {mean:.6e} ± {err:.2e}, analytic {analytic:.6e}, \
                 diff {:.2e}",
                (mean - analytic).abs()
            );
            assert!(
                (mean - analytic).abs() < tol,
                "n={n}: resonant channel V_n {mean:.6e} ± {err:.2e} vs analytic {analytic:.6e}"
            );
        }
    }

    /// Monte-Carlo estimate of `∫ dΦ_n f` and the sample variance of the per-point
    /// estimator `weight·f`, for a resonant integrand `f`.
    fn mc_integrand(
        ch: &DiagramChannel<f64>,
        seed: u64,
        n: usize,
        f: impl Fn(&[LorentzVector<f64>]) -> f64,
    ) -> (f64, f64) {
        let mut stream = SubStream::from_stream(seed, 13);
        let ndim = ch.ndim();
        let (mut sum, mut sum_sq) = (0.0, 0.0);
        for _ in 0..n {
            let u = stream.uniforms::<f64>(ndim);
            let pt = ch.sample(&u);
            let v = pt.weight * f(&pt.momenta);
            sum += v;
            sum_sq += v * v;
        }
        let mean = sum / n as f64;
        let var = (sum_sq / n as f64 - mean * mean).max(0.0);
        (mean, var)
    }

    /// On a Z-pole process (`2 → 3` with a `{0,1}` resonant pair recoiling against a
    /// massless leg), the Breit–Wigner channel and flat RAMBO estimate the same
    /// resonant cross section, but the BW channel's per-point variance is far below
    /// flat RAMBO's at fixed `N` — the point of the importance map.
    #[test]
    fn z_pole_lower_variance_than_flat_rambo() {
        let (m2, mg) = (M_Z * M_Z, M_Z * G_Z);
        let bw = |p: &[LorentzVector<f64>]| 1.0 / ((s01(p) - m2).powi(2) + mg * mg);
        let sqrt_s = 500.0;
        let masses = vec![0.0; 3];

        let ch = DiagramChannel::from_topology_resonant(
            sqrt_s,
            masses.clone(),
            &[(vec![0, 1], Some(z_resonance()))],
        );
        let rb = DiagramChannel::from_topology(sqrt_s, masses.clone(), &[vec![0, 1]]);

        let n = 400_000;
        let (mean_ch, var_ch) = mc_integrand(&ch, 0x2110, n, bw);
        let (mean_rb, var_rb) = mc_integrand(&rb, 0x2111, n, bw);

        let err = ((var_ch + var_rb) / n as f64).sqrt();
        eprintln!(
            "Z-pole σ: BW {mean_ch:.6e} (var {var_ch:.3e}) vs flat {mean_rb:.6e} \
             (var {var_rb:.3e}); var ratio {:.2e}",
            var_ch / var_rb
        );
        assert!(
            (mean_ch - mean_rb).abs() < 6.0 * err,
            "resonant σ disagrees: BW {mean_ch:.6e} vs flat {mean_rb:.6e} (err {err:.2e})"
        );
        assert!(
            var_ch < var_rb,
            "BW variance {var_ch:.3e} not below flat RAMBO variance {var_rb:.3e}"
        );
    }

    /// The sampled invariant-mass distribution of the resonant pair reproduces the
    /// analytic Breit–Wigner line shape. The differential cross section of a
    /// resonant integrand factorises as `dσ/ds ∝ (ŝ − s)·BW(s)` (the two-body
    /// phase-space factor times the pole); binning `weight·BW` from the channel must
    /// track that curve. A mis-sampled pole would distort the line shape even where
    /// σ stays unchanged.
    #[test]
    fn z_pole_histogram_matches_breit_wigner() {
        let (m2, mg) = (M_Z * M_Z, M_Z * G_Z);
        let bw = |p: &[LorentzVector<f64>]| 1.0 / ((s01(p) - m2).powi(2) + mg * mg);
        let sqrt_s = 500.0;
        let s_hat = sqrt_s * sqrt_s;
        let masses = vec![0.0; 3];
        let ch = DiagramChannel::from_topology_resonant(
            sqrt_s,
            masses.clone(),
            &[(vec![0, 1], Some(z_resonance()))],
        );

        let window = 30.0 * mg;
        let (win_lo, win_hi) = (m2 - window, m2 + window);
        let nbins = 24usize;
        let bin_w = (win_hi - win_lo) / nbins as f64;
        let mut hist = vec![0.0_f64; nbins];
        let mut hist_sq = vec![0.0_f64; nbins];
        let mut count = vec![0usize; nbins];

        let mut stream = SubStream::from_stream(0x2112, 17);
        let n = 2_000_000;
        for _ in 0..n {
            let u = stream.uniforms::<f64>(ch.ndim());
            let pt = ch.sample(&u);
            let s = s01(&pt.momenta);
            if s < win_lo || s >= win_hi {
                continue;
            }
            let k = ((s - win_lo) / bin_w) as usize;
            let v = pt.weight * bw(&pt.momenta);
            hist[k] += v;
            hist_sq[k] += v * v;
            count[k] += 1;
        }

        // Analytic antiderivative of `(ŝ − s)·BW(s)`:
        //   A(s) = (ŝ−m²)/mΓ · atan((s−m²)/mΓ) − ½·ln((s−m²)²+(mΓ)²).
        let anti = |s: f64| {
            (s_hat - m2) / mg * ((s - m2) / mg).atan() - 0.5 * ((s - m2).powi(2) + mg * mg).ln()
        };

        let mut mc: Vec<f64> = Vec::new();
        let mut mc_err: Vec<f64> = Vec::new();
        let mut an: Vec<f64> = Vec::new();
        for k in 0..nbins {
            if count[k] < 200 {
                continue;
            }
            let mean = hist[k] / n as f64;
            let err = ((hist_sq[k] / n as f64 - mean * mean).max(0.0) / n as f64).sqrt();
            let lo = win_lo + k as f64 * bin_w;
            mc.push(mean);
            mc_err.push(err);
            an.push(anti(lo + bin_w) - anti(lo));
        }
        assert!(mc.len() >= 12, "too few populated bins: {}", mc.len());

        let s_mc: f64 = mc.iter().sum();
        let s_an: f64 = an.iter().sum();
        let mut chi2 = 0.0;
        for i in 0..mc.len() {
            let p_mc = mc[i] / s_mc;
            let p_an = an[i] / s_an;
            let e = (mc_err[i] / s_mc).max(1e-12);
            chi2 += ((p_mc - p_an) / e).powi(2);
        }
        let dof = mc.len() as f64;
        eprintln!(
            "Z-pole line shape: {} bins, χ²/dof = {:.2}",
            mc.len(),
            chi2 / dof
        );
        assert!(
            chi2 / dof < 3.0,
            "sampled line shape departs from analytic BW: χ²/dof = {:.2}",
            chi2 / dof
        );
    }

    // ── Diagram classification against the momentum-routing convention ──────────

    /// The externals attached to the connected component of `prop[cut]`'s first
    /// endpoint after `prop[cut]` is removed — an independent, momentum-free
    /// derivation of one side of the cut, from the vertex/propagator adjacency
    /// alone. Cross-checking it against the stored-momentum reading catches a
    /// future feyngraph routing-convention change in either derivation.
    fn cut_side_externals(d: &Diagram, cut: usize) -> Vec<usize> {
        let nv = d.vertices.len();
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); nv];
        for (pi, p) in d.props.iter().enumerate() {
            if pi == cut {
                continue;
            }
            let (a, b) = (p.endpoints[0].0, p.endpoints[1].0);
            adj[a.0].push(b.0);
            adj[b.0].push(a.0);
        }
        let start = d.props[cut].endpoints[0].0 .0;
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
    }

    /// Classify `prop[cut]` from the graph cut alone: the outgoing-slot subsystem
    /// it bounds (the beam-free side, under the same `2 ≤ count < n_out` guard),
    /// together with whether the cut splits the two beams (a spacelike transfer).
    fn classify_by_cut(d: &Diagram, cut: usize) -> (Option<u64>, bool) {
        let n_in = d.n_in;
        let n_ext = d.n_ext();
        let n_out = n_ext - n_in;
        let side_a = cut_side_externals(d, cut);
        let mut in_a = vec![false; n_ext];
        for &e in &side_a {
            in_a[e] = true;
        }
        let beams_a = (0..n_in).filter(|&e| in_a[e]).count();
        let splits_beams = beams_a != 0 && beams_a != n_in;
        if splits_beams {
            return (None, true);
        }
        let mut mask = 0u64;
        let mut count = 0usize;
        for e in n_in..n_ext {
            // The beam-free side is `side_a` when it holds no beam, else its
            // complement; either way, take its outgoing legs.
            let on_zero_beam_side = if beams_a == 0 { in_a[e] } else { !in_a[e] };
            if on_zero_beam_side {
                mask |= 1 << (e - n_in);
                count += 1;
            }
        }
        let subsystem = if count >= 2 && count < n_out {
            Some(mask)
        } else {
            None
        };
        (subsystem, false)
    }

    /// The τ⁺τ⁻ s-channel line in `e+ e- > mu+ mu- ta+ ta-` is stored on the
    /// both-beam complement side (feyngraph eliminates the highest external, τ⁺),
    /// so the routing-aware classifier must relabel it onto the τ outgoing slots
    /// `{2,3}` and drive a Z Breit–Wigner node there. This pins the convention
    /// directly: it fails if feyngraph's routing changes or the relabel regresses,
    /// with no reliance on a cross section.
    #[test]
    fn tautau_non_prefix_s_channel_recovered() {
        let m = sm_model(SMRestrict::Default);
        let ev = EvaluatedModel::from_model(m.clone());
        let z_mass = ev.mass(m.particle_id("Z").expect("SM model has a Z"));
        let opts = ParsingOptions::default();
        let card = parse_proc_card("generate e+ e- > mu+ mu- ta+ ta-", &opts).unwrap();
        let sets = generate_from_proc_card(&card, &m).unwrap();
        let set = sets.into_iter().find(|s| !s.diagrams.is_empty()).unwrap();
        assert_eq!(set.particles_out, vec!["mu+", "mu-", "ta+", "ta-"]);

        // Legs: 0=e+ 1=e- 2=mu- 3=mu+ 4=ta- 5=ta+; the τ pair is externals {4,5}
        // = outgoing slots {2,3}. External 5 (τ⁺) is the eliminated one, so a Z on
        // the τ pair is stored on the complementary {0,1,2,3} side with both beams.
        const TAU_MASK: u64 = (1 << 2) | (1 << 3);
        let mut z_tau_diag = None;
        for d in &set.diagrams {
            let n_in = d.n_in;
            let n_ext = d.n_ext();
            for p in &d.props {
                if m.particle(p.particle).pdg_code != 23 {
                    continue;
                }
                if subsystem_mask(&p.momentum, n_in, n_ext) != Some(TAU_MASK) {
                    continue;
                }
                // The stored routing: both beams present, τ slots zero.
                assert_eq!(
                    p.momentum,
                    vec![1, 1, -1, -1, 0, 0],
                    "unexpected stored routing for the τ-pair Z line"
                );
                assert_eq!(
                    p.momentum[..n_in].iter().filter(|&&c| c != 0).count(),
                    2,
                    "τ-pair s-channel line must carry both beams in the stored routing"
                );
                assert!(
                    !p.is_spacelike(n_in),
                    "τ-pair s-channel line must not be classed spacelike"
                );
                z_tau_diag = Some(d);
            }
        }
        let d = z_tau_diag.expect("no τ-pair Z line recovered onto outgoing slots {2,3}");

        // The recovered subsystem installs a Z Breit–Wigner node in the built channel.
        assert!(
            (z_mass - 91.1876).abs() < 0.5,
            "model Z mass {z_mass} not ~M_Z"
        );
        let ch = DiagramChannel::<f64>::from_diagram(d, &ev, 500.0);
        assert!(
            ch.resonances()
                .iter()
                .any(|r| (r.mass - z_mass).abs() < 1e-9),
            "built channel has no Z resonance on the recovered τ-pair subsystem"
        );
    }

    /// The stored-momentum classifier agrees with the independent graph cut on
    /// every propagator of a spread of processes — including genuine t-channels
    /// (Bhabha, `u u~ > u u~`) and the non-prefix s-channel of the τ-pair process.
    /// Two derivations of the same partition pin the routing convention from both
    /// sides.
    #[test]
    fn subsystem_classification_matches_graph_cut() {
        let m = sm_model(SMRestrict::Default);
        let opts = ParsingOptions::default();
        for proc in ["e+ e- > mu+ mu- ta+ ta-", "e+ e- > e+ e-", "u u~ > u u~"] {
            let card = parse_proc_card(&format!("generate {proc}"), &opts).unwrap();
            let sets = generate_from_proc_card(&card, &m).unwrap();
            let mut checked = 0usize;
            let mut spacelike_seen = false;
            for set in &sets {
                for d in &set.diagrams {
                    let n_in = d.n_in;
                    let n_ext = d.n_ext();
                    for (pi, p) in d.props.iter().enumerate() {
                        let via_mom = subsystem_mask(&p.momentum, n_in, n_ext);
                        let (via_cut, splits_beams) = classify_by_cut(d, pi);
                        assert_eq!(
                            via_mom, via_cut,
                            "{proc}: prop {pi} subsystem disagrees (mom {:?})",
                            p.momentum
                        );
                        assert_eq!(
                            p.is_spacelike(n_in),
                            splits_beams,
                            "{proc}: prop {pi} spacelike flag disagrees (mom {:?})",
                            p.momentum
                        );
                        spacelike_seen |= splits_beams;
                        checked += 1;
                    }
                }
            }
            assert!(checked > 0, "{proc}: no propagators exercised");
            if proc != "e+ e- > mu+ mu- ta+ ta-" {
                assert!(spacelike_seen, "{proc}: expected a t-channel propagator");
            }
        }
    }
}
