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
//! sampling density concentrates on the resonance as `1/((s−m²)²+(mΓ)²)`. A
//! *zero-width* pole at or below the kinematic floor — the massless `γ*` of a
//! lepton-pair subsystem, above all — instead contributes a `1/(s−m²)²` rise
//! toward that floor, with no width to regulate it; its invariant is drawn
//! logarithmically in `t = s − m²` down to a floor, the two-piece map of
//! [`log_scale`]. A subsystem with no pole at all keeps the flat draw over its
//! kinematic range. Each node records the propagator particle's mass and width, so
//! the resonance-aware draw of a chosen invariant slots in without changing the
//! tree.
//!
//! Leaving that zero-width rise on a flat draw is not a mere inefficiency: the
//! estimator acquires a tail heavy enough that a run either misses the region
//! (collapsing `σ̂`) or catches it (inflating `σ̂`), and because VEGAS combines its
//! iterations by `1/σ²`, the iterations that miss report a small integral *and* a
//! small variance and go on to dominate the result. The failure is therefore
//! silent — a confidently wrong cross section, not a visibly noisy one.
//!
//! A spacelike (t-channel) line is peripheral, not a subsystem mass: it carries a
//! momentum transfer `t = (p_beam − p_emitted)² ≤ 0`. A diagram with a single
//! spacelike line is decomposed as a [`Spine`] instead of an all-timelike tree — a
//! top-level peripheral emission off one beam whose polar angle is fixed by `t`
//! (only the azimuth is free), with the emitted and recoil subsystems recursing
//! into the same 2-body-decay machinery. The transfer is importance-sampled through
//! the logarithmic substitution `t = m² − (m²−t_min)·exp(−x·N)` (density
//! `∝ 1/(m²−t)`), and a spacelike line carries no width. Genuine
//! multi-spacelike-line (ladder) topologies are not yet given a spine; their
//! spacelike lines are kept as metadata only.
//!
//! # Regulating the spacelike pole
//!
//! With a *massless* exchanged line and a massless subsystem on one side, the
//! transfer's upper edge sits analytically on the pole (`t_max = m² = 0`) but is
//! computed as a cancelling difference of two large quantities built from different
//! expressions, so it lands on either side of zero at the rounding scale. A draw
//! that then switches the propagator map on reaches `|t| ~ 1e-11` while the density
//! recomputes `t` from the momenta with a cancellation error of the same size:
//! sampling density and weighting density describe different maps, and the
//! estimator is *biased*, not merely noisy. In a `2 → 2` final state both
//! invariants are fixed constants and the cancellation is exact, so the edge is a
//! hard zero and the flat fallback fires deterministically; the degeneracy needs a
//! *drawn* invariant on one side, which first appears at three outgoing legs.
//!
//! [`DiagramChannel::from_diagram_regulated`] therefore takes a **floor** on the
//! pole location, `t_mass² ← max(m², floor)`. The floor moves only the sampling
//! density — `draw_t` and `t_measure` read the same `t_mass2`, so the estimator is
//! unbiased for any non-negative value — and a floor far above the cancellation
//! scale is what makes the propagator draw well posed. A spine is built for a
//! final state of more than two legs only when such a floor is supplied.
//!
//! The weight is the exact product of the 2-body LIPS factors `R_2 = π|p*|/√s`
//! and each invariant's draw measure `ds/dx`, so a flat Monte-Carlo average of
//! `weight · f` estimates `∫ dR_n f` over the same invariant volume `R_n` that
//! flat RAMBO integrates — the channel is a different parametrisation of the same
//! phase space. [`PhaseSpaceMap::sample`] accumulates that product **as it walks**,
//! independently of [`Channel::density`], which recomputes it from the realised
//! momenta; the two agreeing is then a real check on the map rather than an
//! identity the code arranged.
//!
//! # Energy
//!
//! The tree is a `√ŝ`-independent structure: masks, masses, resonances and the
//! spacelike pole do not move with the collision energy. The fixed-energy
//! [`PhaseSpaceMap`]/[`Channel`] impls use the `sqrt_s` a channel was built at;
//! [`ScaledChannel`] takes it per draw, which is what lets one channel set serve a
//! hadronic run whose `ŝ = τ s` changes every event.

use std::collections::BTreeMap;

use crate::diagrams::diagram::Diagram;
use crate::helas::repr::lorentz::LorentzVector;
use crate::helas::repr::Real;
use crate::ufo::EvaluatedModel;

use super::channel::{Channel, PhaseSpaceMap, PhaseSpacePoint, ScaledChannel};

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

/// A t-channel spine: the peripheral emission a single spacelike line drives.
///
/// One beam (the `emitted` subsystem's anchor, beam `0` here) emits the `emitted`
/// subsystem against the spacelike propagator of mass² `t_mass2`; the `recoil`
/// subsystem balances it. The polar angle of the emission is fixed by the sampled
/// momentum transfer `t`, so only its azimuth is free. Each subsystem is a
/// [`Node`] that recurses into the timelike 2-body-decay machinery unchanged. This
/// carries one spacelike rung; a ladder of several spacelike lines would extend it
/// to an ordered chain of rungs.
#[derive(Clone, Debug)]
struct Spine<F: Real> {
    emitted: Node<F>,
    recoil: Node<F>,
    /// The spacelike propagator's mass²; its width is zero by construction.
    t_mass2: F,
}

/// The top-level structure of a channel: an all-timelike decay tree, or a
/// peripheral t-channel spine with timelike subsystems hanging off it.
#[derive(Clone, Debug)]
enum ChannelTopology<F: Real> {
    Timelike(Branch<F>),
    Spine(Spine<F>),
}

/// A single-diagram phase-space channel on one outgoing-mass set.
///
/// `sqrt_s` is the energy the fixed-energy [`PhaseSpaceMap`]/[`Channel`] impls
/// draw at; [`ScaledChannel`] takes the energy per draw instead and leaves this
/// field unread.
#[derive(Clone, Debug)]
pub struct DiagramChannel<F: Real> {
    sqrt_s: F,
    n_out: usize,
    /// Pole masses of the two incoming legs, from which the CM beam momenta (beam
    /// `0` along `+z`) — the reference for a spacelike line's momentum transfer
    /// `t` — are rebuilt at whatever energy a draw is made at.
    beam_masses: [F; 2],
    topology: ChannelTopology<F>,
    t_channels: Vec<TChannel<F>>,
}

impl<F: Real> DiagramChannel<F> {
    /// Build the channel from a diagram's propagator chain at CM energy `sqrt_s`,
    /// with an unregulated spacelike pole.
    ///
    /// Outgoing-leg masses and each internal line's mass/width are read from
    /// `model`. Only meaningful for a `2 → n` process; the beams are externals
    /// `0..n_in`.
    pub fn from_diagram(diagram: &Diagram, model: &EvaluatedModel, sqrt_s: F) -> Self {
        Self::from_diagram_regulated(diagram, model, sqrt_s, F::zero())
    }

    /// [`from_diagram`](Self::from_diagram) with the spacelike propagator's pole
    /// floored at `spacelike_floor` (GeV²), so a massless exchanged line still
    /// gives a well-posed peripheral draw.
    ///
    /// The floor enters `draw_t` and `t_measure` alike, so it reshapes the sampling
    /// density and nothing else. Without one, a spacelike line inside a final state
    /// of more than two legs is left to the all-timelike tree: the module docs
    /// record why an unfloored three-body spine is biased rather than merely noisy.
    pub fn from_diagram_regulated(
        diagram: &Diagram,
        model: &EvaluatedModel,
        sqrt_s: F,
        spacelike_floor: F,
    ) -> Self {
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
        let beam_masses = [
            cast(model.mass(diagram.legs[0].particle)),
            cast(model.mass(diagram.legs[1].particle)),
        ];

        // Timelike subsystems drive the decay tree; spacelike lines are peripheral.
        let mut resonances: BTreeMap<u64, Resonance<F>> = BTreeMap::new();
        let mut t_channels = Vec::new();
        let mut spacelike: Vec<usize> = Vec::new();
        for (pi, prop) in diagram.props.iter().enumerate() {
            if prop.is_spacelike(n_in) {
                spacelike.push(pi);
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
        // A single spacelike line is peripheral: it is decomposed as a t-channel
        // spine rather than an all-timelike tree. Past two outgoing legs the
        // transfer's upper edge is a cancelling difference of drawn invariants, so
        // the spine is built only when a floor keeps the pole clear of it. Ladder
        // topologies (several spacelike lines) stay all-timelike for now.
        let regulated = spacelike_floor > F::zero();
        let topology = if spacelike.len() == 1 && (n_out == 2 || regulated) {
            let prop = &diagram.props[spacelike[0]];
            let (emitted_mask, recoil_mask) = spine_partition(&prop.momentum, n_in, n_ext);
            ChannelTopology::Spine(Spine {
                emitted: build_node(emitted_mask, &masses, &subsystems, &resonances),
                recoil: build_node(recoil_mask, &masses, &subsystems, &resonances),
                t_mass2: {
                    let m: F = cast(model.mass(prop.particle));
                    (m * m).max(spacelike_floor)
                },
            })
        } else {
            ChannelTopology::Timelike(build_root(n_out, &masses, &subsystems, &resonances))
        };
        DiagramChannel {
            sqrt_s,
            n_out,
            beam_masses,
            topology,
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
            beam_masses: [F::zero(), F::zero()],
            topology: ChannelTopology::Timelike(root),
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
            beam_masses: [F::zero(), F::zero()],
            topology: ChannelTopology::Timelike(root),
            t_channels: Vec::new(),
        }
    }

    /// Build a single-rung t-channel spine directly, without a diagram, for
    /// exercising the peripheral kinematics on a controlled topology. `beam_masses`
    /// are the two incoming masses; `emitted` and `recoil` are disjoint outgoing-leg
    /// slot sets partitioning `0..masses.len()` across the spacelike cut (`emitted`
    /// anchored to beam `0`), each optionally carrying a [`Resonance`] on its own
    /// invariant. `t_mass` is the spacelike propagator mass (width zero).
    pub fn from_topology_tchannel(
        sqrt_s: F,
        beam_masses: [F; 2],
        masses: Vec<F>,
        emitted: (Vec<usize>, Option<Resonance<F>>),
        recoil: (Vec<usize>, Option<Resonance<F>>),
        t_mass: F,
    ) -> Self {
        let n_out = masses.len();
        assert!(n_out >= 2, "a 2-body decomposition needs at least two legs");
        let mut resonances: BTreeMap<u64, Resonance<F>> = BTreeMap::new();
        let mut masks = Vec::new();
        let mut mask_of = |slots: &[usize], res: Option<Resonance<F>>| -> u64 {
            let mask = slots.iter().fold(0u64, |m, &i| m | (1 << i));
            if let Some(r) = res {
                resonances.insert(mask, r);
            }
            masks.push(mask);
            mask
        };
        let emitted_mask = mask_of(&emitted.0, emitted.1);
        let recoil_mask = mask_of(&recoil.0, recoil.1);
        assert_eq!(
            emitted_mask | recoil_mask,
            (1u64 << n_out) - 1,
            "emitted and recoil slots must partition the final state"
        );
        assert_eq!(
            emitted_mask & recoil_mask,
            0,
            "emitted and recoil slots must be disjoint"
        );
        let subsystems: Vec<u64> = resonances.keys().copied().collect();
        let spine = Spine {
            emitted: build_node(emitted_mask, &masses, &subsystems, &resonances),
            recoil: build_node(recoil_mask, &masses, &subsystems, &resonances),
            t_mass2: t_mass * t_mass,
        };
        DiagramChannel {
            sqrt_s,
            n_out,
            beam_masses,
            topology: ChannelTopology::Spine(spine),
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
        match &self.topology {
            ChannelTopology::Timelike(root) => collect_resonances(root, &mut out),
            ChannelTopology::Spine(spine) => {
                collect_node_resonances(&spine.emitted, &mut out);
                collect_node_resonances(&spine.recoil, &mut out);
            }
        }
        out
    }

    /// The spacelike (t-channel) lines of the diagram, for a t-channel importance
    /// map.
    pub fn t_channels(&self) -> &[TChannel<F>] {
        &self.t_channels
    }

    /// The pole location `t_mass²` of the peripheral rung, or `None` for an
    /// all-timelike tree — the floor a regulated channel actually installed.
    pub fn spine_pole(&self) -> Option<F> {
        match &self.topology {
            ChannelTopology::Timelike(_) => None,
            ChannelTopology::Spine(spine) => Some(spine.t_mass2),
        }
    }

    /// The CM beam momenta at energy `sqrt_s`, beam `0` along `+z`.
    fn beams_at(&self, sqrt_s: F) -> [LorentzVector<F>; 2] {
        beam_momenta(sqrt_s, self.beam_masses[0], self.beam_masses[1])
    }
}

impl<F: Real> ScaledChannel<F> for DiagramChannel<F> {
    fn sample_at(&self, sqrt_s: F, u: &[F]) -> PhaseSpacePoint<F> {
        let s = sqrt_s * sqrt_s;
        let total = LorentzVector::new(sqrt_s, F::zero(), F::zero(), F::zero());
        let mut slots: Vec<Option<LorentzVector<F>>> = vec![None; self.n_out];
        let mut cursor = 0usize;
        // Accumulated as the walk draws, from the drawn invariants rather than from
        // the momenta they produce — an independent computation of the same
        // Jacobian `density_at` reconstructs, so the two agreeing is a check.
        let mut weight = F::one();
        match &self.topology {
            ChannelTopology::Timelike(root) => {
                sample_branch(root, s, total, u, &mut cursor, &mut slots, &mut weight)
            }
            ChannelTopology::Spine(spine) => sample_spine(
                spine,
                s,
                &self.beams_at(sqrt_s),
                total,
                u,
                &mut cursor,
                &mut slots,
                &mut weight,
            ),
        }
        let momenta: Vec<LorentzVector<F>> = slots
            .into_iter()
            .map(|m| m.expect("every outgoing slot is filled"))
            .collect();
        PhaseSpacePoint { momenta, weight }
    }

    fn density_at(&self, sqrt_s: F, momenta: &[LorentzVector<F>]) -> F {
        let s = sqrt_s * sqrt_s;
        let jac = match &self.topology {
            ChannelTopology::Timelike(root) => branch_jacobian(root, s, momenta),
            ChannelTopology::Spine(spine) => {
                spine_jacobian(spine, s, &self.beams_at(sqrt_s), momenta)
            }
        };
        F::one() / jac
    }
}

impl<F: Real> PhaseSpaceMap<F> for DiagramChannel<F> {
    fn ndim(&self) -> usize {
        3 * self.n_out - 4
    }

    fn sample(&self, u: &[F]) -> PhaseSpacePoint<F> {
        self.sample_at(self.sqrt_s, u)
    }
}

impl<F: Real> Channel<F> for DiagramChannel<F> {
    fn density(&self, momenta: &[LorentzVector<F>]) -> F {
        self.density_at(self.sqrt_s, momenta)
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
    for (bit, &c) in momentum[n_in..n_ext].iter().enumerate() {
        if c != 0 {
            stored |= 1 << bit;
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

fn collect_node_resonances<F: Real>(node: &Node<F>, out: &mut Vec<Resonance<F>>) {
    if let Node::Branch(b) = node {
        collect_resonances(b, out);
    }
}

/// The two incoming beam four-momenta in the CM frame at `sqrt_s` for beam masses
/// `ma`, `mb`: beam `0` along `+z`, beam `1` along `−z`, both on shell.
fn beam_momenta<F: Real>(sqrt_s: F, ma: F, mb: F) -> [LorentzVector<F>; 2] {
    let two = F::one() + F::one();
    let s = sqrt_s * sqrt_s;
    let (ma2, mb2) = (ma * ma, mb * mb);
    let e_a = (s + ma2 - mb2) / (two * sqrt_s);
    let e_b = (s + mb2 - ma2) / (two * sqrt_s);
    let k = kallen(s, ma2, mb2).max(F::zero()).sqrt() / (two * sqrt_s);
    [
        LorentzVector::new(e_a, F::zero(), F::zero(), k),
        LorentzVector::new(e_b, F::zero(), F::zero(), -k),
    ]
}

/// Split the outgoing legs across a spacelike line into `(emitted, recoil)` masks,
/// `emitted` on beam `0`'s side. The stored `momentum` marks the externals on one
/// side of the cut (feyngraph's routing sign-decorates them, so only the nonzero
/// pattern is read); a spacelike line carries exactly one beam on that side. The
/// emitted subsystem is the outgoing legs sharing beam `0`'s side.
fn spine_partition(momentum: &[i8], n_in: usize, n_ext: usize) -> (u64, u64) {
    let n_out = n_ext - n_in;
    let full = (1u64 << n_out) - 1;
    let stored_has_beam0 = momentum[0] != 0;
    let mut stored_out = 0u64;
    for (bit, &c) in momentum[n_in..n_ext].iter().enumerate() {
        if c != 0 {
            stored_out |= 1 << bit;
        }
    }
    let emitted = if stored_has_beam0 {
        stored_out
    } else {
        full & !stored_out
    };
    (emitted, full & !emitted)
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
/// or `None` when the line carries no finite-width pole (`mΓ ≤ 0`) — a zero-width
/// or massless line, whose rise is handled by [`log_scale`] instead.
fn bw_scale<F: Real>(res: Option<Resonance<F>>) -> Option<(F, F)> {
    let r = res?;
    let mg = r.mass * r.width;
    if mg > F::zero() {
        Some((r.mass * r.mass, mg))
    } else {
        None
    }
}

/// Absolute floor (GeV²) on the shifted invariant `t = s − m²` below which a
/// zero-width pole's `1/t` rise is no longer chased, mirroring the `10/stot` term
/// of MadEvent's `set_peaks` (`myamp.f`). A massless pole has no kinematic lower
/// edge of its own, so without a floor the log map has no normalisable lower
/// limit.
const LOG_MAP_FLOOR_GEV2: f64 = 10.0;
/// Fraction of the draw spent covering `[t_lo, t0]` — the region below the log
/// map's floor — linearly, so the map keeps full support and the estimator stays
/// unbiased. MadEvent reserves the same tenth of its grid bins (`ngd = ng − 0.9·ng`
/// in `setgrid`, `dsample.f`) to reach down past `xo`.
const LOG_MAP_TAIL_FRACTION: f64 = 0.1;

/// The logarithmic map a **zero-width** timelike line imposes on its invariant
/// draw, in the shifted variable `t = s − m²`.
#[derive(Clone, Copy, Debug)]
struct LogMap<F: Real> {
    /// Pole location `m²`; the draw is logarithmic in `t = s − m²`.
    m2: F,
    /// Kinematic lower edge `lo − m²`.
    t_lo: F,
    /// Where the logarithmic piece starts: `max(t_lo, floor)`.
    t0: F,
    /// Upper edge `hi − m²`.
    t_hi: F,
    /// Share of `x` spent covering `[t_lo, t0]` linearly. Zero when the kinematic
    /// edge already sits at or above the floor, in which case there is no
    /// sub-floor region and the logarithmic piece takes the whole draw — the
    /// linear piece would otherwise be a zero-width interval carrying zero
    /// measure, collapsing the weight.
    frac: F,
}

/// The [`LogMap`] parameters a **zero-width** timelike line imposes on its
/// invariant draw, or `None` when the map does not apply.
///
/// A zero-width propagator contributes `1/(s − m²)²` to `|M|²`, which rises without
/// bound toward the kinematic edge `s → lo`. A flat draw over `[lo, hi]` therefore
/// samples an integrand whose variance is dominated by the edge — the estimator
/// acquires a heavy tail, and a run either misses the spike (collapsing `σ̂`) or
/// catches it (inflating `σ̂`). Sampling `∝ 1/t` — uniform in `ln t` — flattens
/// that rise.
///
/// The pole must sit at or below the kinematic floor (`m² ≤ lo`) for `1/(s − m²)`
/// to be a monotone rise toward `lo`; a zero-width pole *inside* the range is a
/// genuine singularity this map does not regulate, and the flat draw stands.
fn log_scale<F: Real>(lo: F, hi: F, res: Option<Resonance<F>>) -> Option<LogMap<F>> {
    let r = res?;
    if r.mass * r.width > F::zero() {
        return None;
    }
    let m2 = r.mass * r.mass;
    if m2 > lo {
        return None;
    }
    let t_lo = lo - m2;
    let t_hi = hi - m2;
    // MadEvent's floor is `min(10/stot, stot/50, 0.5)` in units of `stot`; the
    // second term keeps the floor inside a range narrower than the absolute one.
    let floor = cast::<F>(LOG_MAP_FLOOR_GEV2).min(t_hi / cast(50.0));
    let t0 = t_lo.max(floor);
    // Negated comparisons, so a NaN bound falls back to the flat draw rather than
    // reaching `ln`/`powf` — the positive form would let NaN through.
    if !(t_hi > t0) || !(t0 > F::zero()) {
        return None;
    }
    let frac = if t0 > t_lo {
        cast(LOG_MAP_TAIL_FRACTION)
    } else {
        F::zero()
    };
    Some(LogMap {
        m2,
        t_lo,
        t0,
        t_hi,
        frac,
    })
}

/// Map `x ∈ [0,1]` to an invariant `s ∈ [lo, hi]`.
///
/// A finite-width pole importance-samples the relativistic Breit–Wigner via
/// `s = m² + mΓ·tan θ`, with `θ` uniform over `[atan((lo−m²)/mΓ), atan((hi−m²)/mΓ)]`.
/// A zero-width pole instead gets the two-piece map of [`log_scale`]: the last
/// `1 − LOG_MAP_TAIL_FRACTION` of `x` is uniform in `ln t` over `[t0, t_hi]`,
/// importance-sampling the `1/t` rise, and the leading fraction covers the
/// remaining `[t_lo, t0]` linearly so the map still reaches the kinematic edge.
/// With neither, the draw is flat.
fn draw_invariant<F: Real>(lo: F, hi: F, res: Option<Resonance<F>>, x: F) -> F {
    if let Some((m2, mg)) = bw_scale(res) {
        let theta_lo = ((lo - m2) / mg).atan();
        let theta_hi = ((hi - m2) / mg).atan();
        let theta = theta_lo + (theta_hi - theta_lo) * x;
        return m2 + mg * theta.tan();
    }
    if let Some(m) = log_scale(lo, hi, res) {
        return if m.frac > F::zero() && x < m.frac {
            m.m2 + m.t_lo + (m.t0 - m.t_lo) * (x / m.frac)
        } else {
            let y = (x - m.frac) / (F::one() - m.frac);
            m.m2 + m.t0 * (m.t_hi / m.t0).powf(y)
        };
    }
    lo + (hi - lo) * x
}

/// The invariant-draw measure `ds/dx` at the realised `s`, whose reciprocal is the
/// sampling density: the flat range length `hi − lo`; for the Breit–Wigner map
/// `[(s−m²)²/(mΓ) + mΓ]·(θ_hi−θ_lo)`, the exact `ds/dθ · dθ/dx`; and for the
/// zero-width log map the piecewise `(t0 − t_lo)/frac` below the floor and
/// `t·ln(t_hi/t0)/(1 − frac)` above it — a density `∝ 1/t` over the rise.
fn invariant_measure<F: Real>(lo: F, hi: F, res: Option<Resonance<F>>, s: F) -> F {
    if let Some((m2, mg)) = bw_scale(res) {
        let theta_lo = ((lo - m2) / mg).atan();
        let theta_hi = ((hi - m2) / mg).atan();
        let d = s - m2;
        return (d * d / mg + mg) * (theta_hi - theta_lo);
    }
    if let Some(m) = log_scale(lo, hi, res) {
        let t = s - m.m2;
        return if m.frac > F::zero() && t < m.t0 {
            (m.t0 - m.t_lo) / m.frac
        } else {
            t * (m.t_hi / m.t0).ln() / (F::one() - m.frac)
        };
    }
    hi - lo
}

/// Draw the invariants and angles of one 2-body split and recurse into composite
/// daughters. `s` is the (fixed) invariant mass² of the system this branch
/// decays; `p_lab` is its four-momentum in the CM frame. `weight` accumulates the
/// draw measures and LIPS factors of the walk.
#[allow(clippy::too_many_arguments)]
fn sample_branch<F: Real>(
    branch: &Branch<F>,
    s: F,
    p_lab: LorentzVector<F>,
    u: &[F],
    cursor: &mut usize,
    slots: &mut [Option<LorentzVector<F>>],
    weight: &mut F,
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
            let s = draw_invariant(lo, hi, b.resonance, x);
            *weight = *weight * invariant_measure(lo, hi, b.resonance, s);
            s
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
            let s = draw_invariant(lo, hi, b.resonance, x);
            *weight = *weight * invariant_measure(lo, hi, b.resonance, s);
            s
        }
    };
    *weight = *weight * r2_factor(s, sqrt_s, sl, sr);

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
        Node::Branch(b) => sample_branch(b, sl, pl, u, cursor, slots, weight),
    }
    match &branch.right {
        Node::Leaf { slot, .. } => slots[*slot] = Some(pr),
        Node::Branch(b) => sample_branch(b, sr, pr, u, cursor, slots, weight),
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

// ── T-channel spine ──────────────────────────────────────────────────────────

/// Whether the propagator `1/(t − m²)` can shape the draw over `[t_min, t_max]`:
/// both endpoints strictly below the pole `m²`, and a non-degenerate window. A
/// massless line whose window reaches the collinear edge (`t_max = m²`) cannot —
/// the density would diverge — so the draw falls back to flat, exactly as a
/// zero-width timelike pole keeps the flat invariant draw.
fn t_pole_shapes<F: Real>(a: F, b: F) -> bool {
    a > F::zero() && b > F::zero() && a != b
}

/// Map `x ∈ [0,1]` to a spacelike momentum transfer `t ∈ [t_min, t_max]` (both
/// `≤ 0`) importance-sampling the propagator `1/(t − m²)`: density `∝ 1/(m² − t)`,
/// realised by `t = m² − (m²−t_min)·exp(−x·N)` with `N = ln[(m²−t_min)/(m²−t_max)]`.
/// A spacelike line has no width, so `m²` is its bare mass². When the pole cannot
/// shape the draw (a massless line at the collinear edge `t_max = m²`, or a
/// threshold-degenerate window) the draw is flat over `[t_min, t_max]`, which makes
/// the spine reduce to the isotropic 2-body split there.
fn draw_t<F: Real>(t_min: F, t_max: F, t_mass2: F, x: F) -> F {
    let a = t_mass2 - t_min;
    let b = t_mass2 - t_max;
    if t_pole_shapes(a, b) {
        let n = (a / b).ln();
        t_mass2 - a * (-x * n).exp()
    } else {
        t_min + (t_max - t_min) * x
    }
}

/// The t-draw measure `dt/dx` at the realised `t`: `N·(m² − t)` with
/// `N = ln[(m²−t_min)/(m²−t_max)]`, whose reciprocal is the sampling density
/// `∝ 1/(m² − t)`; or the flat range `t_max − t_min` where the pole cannot shape
/// the draw. Its reciprocal is the spine's sampling density in `t`.
fn t_measure<F: Real>(t_min: F, t_max: F, t_mass2: F, t: F) -> F {
    let a = t_mass2 - t_min;
    let b = t_mass2 - t_max;
    if t_pole_shapes(a, b) {
        let n = (a / b).ln();
        n * (t_mass2 - t)
    } else {
        t_max - t_min
    }
}

/// The peripheral 2-body kinematics of a system of invariant `s` scattering beam
/// masses² `ma2`, `mb2` into subsystems of invariant `s1` (emitted, beam-`0` side)
/// and `s2` (recoil).
struct TKin<F: Real> {
    /// Bounds of the spacelike transfer `t`, both `≤ 0`, at `cosθ = ∓1`.
    t_min: F,
    t_max: F,
    /// `t` at `cosθ = 0`: `t = center + 2·k·p*·cosθ`.
    center: F,
    /// Beam and emitted-subsystem CM momentum magnitudes.
    k: F,
    pstar: F,
    /// Emitted / recoil CM energies.
    e1: F,
    e2: F,
}

fn t_kinematics<F: Real>(s: F, ma2: F, mb2: F, s1: F, s2: F) -> TKin<F> {
    let two = F::one() + F::one();
    let sqrt_s = s.sqrt();
    let inv = F::one() / (two * sqrt_s);
    let ea = (s + ma2 - mb2) * inv;
    let e1 = (s + s1 - s2) * inv;
    let e2 = (s + s2 - s1) * inv;
    let k = kallen(s, ma2, mb2).max(F::zero()).sqrt() * inv;
    let pstar = kallen(s, s1, s2).max(F::zero()).sqrt() * inv;
    // t = ma² + s1 − 2(Ea·E1 − k·p*·cosθ); the beam is along +z, so cosθ is the
    // emitted subsystem's polar angle from beam 0.
    let center = ma2 + s1 - two * ea * e1;
    let span = two * k * pstar;
    TKin {
        t_min: center - span,
        t_max: center + span,
        center,
        k,
        pstar,
        e1,
        e2,
    }
}

/// The peripheral 2-body factor `π·(dt/dx)/(4√s·k)` a spine rung contributes: the
/// solid-angle LIPS `R_2` reparametrised from `(cosθ, φ)` to `(t, φ)` via
/// `dcosθ = dt/(2k·p*)`, with the azimuth's `2π` folded in and `p*` cancelling.
fn peripheral_factor<F: Real>(s: F, k: F, t_measure_val: F) -> F {
    let sqrt_s = s.sqrt();
    let four = F::from(4).expect("4 fits the scalar field");
    if k > F::zero() && sqrt_s > F::zero() {
        F::PI() * t_measure_val / (four * sqrt_s * k)
    } else {
        F::zero()
    }
}

/// The invariant `(pa − p1)²` — the momentum transfer between a beam and an emitted
/// subsystem, evaluated frame-independently so the spine density is well-defined on
/// a configuration the channel did not itself generate.
fn transfer_invariant<F: Real>(pa: LorentzVector<F>, p1: LorentzVector<F>) -> F {
    let de = pa.e() - p1.e();
    let dx = pa.px() - p1.px();
    let dy = pa.py() - p1.py();
    let dz = pa.pz() - p1.pz();
    de * de - dx * dx - dy * dy - dz * dz
}

/// Draw the emitted/recoil invariants, the transfer `t`, and the azimuth of one
/// peripheral emission, then recurse into each subsystem's decay tree. `s` is the
/// scattering system's invariant mass²; `p_lab` its CM four-momentum; `beams` the
/// incoming beams in that frame (beam `0` along `+z`).
#[allow(clippy::too_many_arguments)]
fn sample_spine<F: Real>(
    spine: &Spine<F>,
    s: F,
    beams: &[LorentzVector<F>; 2],
    p_lab: LorentzVector<F>,
    u: &[F],
    cursor: &mut usize,
    slots: &mut [Option<LorentzVector<F>>],
    weight: &mut F,
) {
    let two = F::one() + F::one();
    let sqrt_s = s.sqrt();
    let mu_e = spine.emitted.mu();
    let mu_r = spine.recoil.mu();

    let s1 = match &spine.emitted {
        Node::Leaf { mass, .. } => *mass * *mass,
        Node::Branch(b) => {
            let lo = mu_e * mu_e;
            let hi = (sqrt_s - mu_r).powi(2);
            let x = u[*cursor];
            *cursor += 1;
            let s = draw_invariant(lo, hi, b.resonance, x);
            *weight = *weight * invariant_measure(lo, hi, b.resonance, s);
            s
        }
    };
    let sqrt_s1 = s1.sqrt();
    let s2 = match &spine.recoil {
        Node::Leaf { mass, .. } => *mass * *mass,
        Node::Branch(b) => {
            let lo = mu_r * mu_r;
            let hi = (sqrt_s - sqrt_s1).powi(2);
            let x = u[*cursor];
            *cursor += 1;
            let s = draw_invariant(lo, hi, b.resonance, x);
            *weight = *weight * invariant_measure(lo, hi, b.resonance, s);
            s
        }
    };

    let tk = t_kinematics(s, beams[0].m2(), beams[1].m2(), s1, s2);
    let t = draw_t(tk.t_min, tk.t_max, spine.t_mass2, u[*cursor]);
    *cursor += 1;
    *weight = *weight * peripheral_factor(s, tk.k, t_measure(tk.t_min, tk.t_max, spine.t_mass2, t));
    let phi = two * F::PI() * u[*cursor];
    *cursor += 1;

    let span = two * tk.k * tk.pstar;
    let cos = if span > F::zero() {
        ((t - tk.center) / span).max(-F::one()).min(F::one())
    } else {
        F::zero()
    };
    let sin = (F::one() - cos * cos).max(F::zero()).sqrt();
    let (dx, dy, dz) = (sin * phi.cos(), sin * phi.sin(), cos);
    let pstar = tk.pstar;
    let pe_rest = LorentzVector::new(tk.e1, pstar * dx, pstar * dy, pstar * dz);
    let pr_rest = LorentzVector::new(tk.e2, -pstar * dx, -pstar * dy, -pstar * dz);

    let pe = safe_boost(pe_rest, p_lab);
    let pr = safe_boost(pr_rest, p_lab);

    match &spine.emitted {
        Node::Leaf { slot, .. } => slots[*slot] = Some(pe),
        Node::Branch(b) => sample_branch(b, s1, pe, u, cursor, slots, weight),
    }
    match &spine.recoil {
        Node::Leaf { slot, .. } => slots[*slot] = Some(pr),
        Node::Branch(b) => sample_branch(b, s2, pr, u, cursor, slots, weight),
    }
}

/// The spine's phase-space Jacobian at an arbitrary configuration: the emitted /
/// recoil invariant measures, the peripheral `t` factor (with `t` recomputed as the
/// beam-`0`-to-emitted transfer), and each subsystem's own decay-tree Jacobian.
fn spine_jacobian<F: Real>(
    spine: &Spine<F>,
    s: F,
    beams: &[LorentzVector<F>; 2],
    momenta: &[LorentzVector<F>],
) -> F {
    let sqrt_s = s.sqrt();
    let mu_e = spine.emitted.mu();
    let mu_r = spine.recoil.mu();
    let s1 = node_invariant(&spine.emitted, momenta);
    let sqrt_s1 = s1.sqrt();
    let s2 = node_invariant(&spine.recoil, momenta);

    let tk = t_kinematics(s, beams[0].m2(), beams[1].m2(), s1, s2);
    let p_emitted = subtree_momentum(&spine.emitted, momenta);
    let t = transfer_invariant(beams[0], p_emitted);

    let mut f = F::one();
    if let Node::Branch(b) = &spine.emitted {
        let lo = mu_e * mu_e;
        let hi = (sqrt_s - mu_r).powi(2);
        f = f * invariant_measure(lo, hi, b.resonance, s1);
    }
    if let Node::Branch(b) = &spine.recoil {
        let lo = mu_r * mu_r;
        let hi = (sqrt_s - sqrt_s1).powi(2);
        f = f * invariant_measure(lo, hi, b.resonance, s2);
    }
    f = f * peripheral_factor(s, tk.k, t_measure(tk.t_min, tk.t_max, spine.t_mass2, t));
    if let Node::Branch(b) = &spine.emitted {
        f = f * branch_jacobian(b, s1, momenta);
    }
    if let Node::Branch(b) = &spine.recoil {
        f = f * branch_jacobian(b, s2, momenta);
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

    /// The weight the walk accumulated and the density reconstructed from the
    /// realised momenta are reciprocal — a check with content, because the two are
    /// computed from different inputs.
    ///
    /// `sample` multiplies each draw measure and LIPS factor in as it draws, from
    /// the invariants it *drew*; `density` re-derives every one of them from the
    /// momenta the walk produced. A walk that sampled from one density and weighted
    /// by another — the failure a floorless spacelike pole produces, where the two
    /// disagree by orders of magnitude — separates them here. Agreement is not
    /// exact: the two arithmetic paths multiply the same factors in different
    /// orders, and the deepest tree here (`2 → 6`) accumulates ~1e-12 of relative
    /// reordering noise. The bound sits above that and some nine orders below the
    /// defect class it exists to catch, so it distinguishes a structural mismatch
    /// from rounding without being a fitted number.
    #[test]
    fn density_is_reciprocal_weight() {
        let mut stream = SubStream::from_stream(0xD1A7, 5);
        let mut worst = 0.0f64;
        for (sqrt_s, masses, subs) in topologies() {
            let ch = DiagramChannel::from_topology(sqrt_s, masses.clone(), &subs);
            for _ in 0..50 {
                let u = stream.uniforms::<f64>(ch.ndim());
                let pt = ch.sample(&u);
                let recip = 1.0 / ch.density(&pt.momenta);
                assert!(pt.weight > 0.0 && pt.weight.is_finite());
                let rel = (pt.weight - recip).abs() / recip;
                worst = worst.max(rel);
                assert!(
                    rel < 1e-9,
                    "walk weight {} vs 1/density {recip} (rel {rel:.3e})",
                    pt.weight
                );
            }
        }
        eprintln!("walk-weight vs density: worst relative disagreement {worst:.3e}");
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

    /// A massless zero-width pole (the `gamma* -> l+ l-` case) at the kinematic
    /// edge `lo = 0`. Its invariant is the pair mass², drawn over the full range.
    fn photon_resonance() -> Resonance<f64> {
        Resonance {
            mass: 0.0,
            width: 0.0,
        }
    }

    /// The zero-width log map is measure-preserving, exactly as the Breit–Wigner
    /// map is: averaging `ds/dx` over uniform `x` must reproduce `∫ ds = hi − lo`.
    /// This is what pins the *two-piece* Jacobian — a map that silently dropped the
    /// sub-floor linear piece, or mismatched the `1/(1−frac)` normalisation on the
    /// log piece, would bias every cross section using it.
    #[test]
    fn log_map_is_measure_preserving() {
        let res = Some(photon_resonance());
        let (lo, hi) = (0.0_f64, 250_000.0);
        let mut stream = SubStream::from_stream(0xF07E, 3);
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
        eprintln!("log ∫ds = {mean:.6e} ± {err:.2e}, want {:.6e}", hi - lo);
        assert!(
            (mean - (hi - lo)).abs() < 5.0 * err,
            "log measure not preserving: ∫ds = {mean:.6e} ± {err:.2e} vs {:.6e}",
            hi - lo
        );
    }

    /// Above its floor the log map turns the `1/s` rise of a massless pole into a
    /// *constant* integrand — `measure(s)/s = ln(t_hi/t0)/(1−frac)` at every `x` —
    /// so the estimator has zero variance there. This is the sharpest pin on
    /// `ds/dx`, and the property the flat draw lacked: against `1/s` a flat draw's
    /// estimator spans the full dynamic range of the propagator.
    #[test]
    fn log_map_zero_variance_on_one_over_s() {
        let res = Some(photon_resonance());
        let (lo, hi) = (0.0_f64, 250_000.0);
        let m = log_scale(lo, hi, res).expect("massless pole takes the log map");
        let analytic = (m.t_hi / m.t0).ln() / (1.0 - LOG_MAP_TAIL_FRACTION);
        // Sample strictly above the floor fraction, where the log piece applies.
        for k in 0..=40 {
            let x = LOG_MAP_TAIL_FRACTION + (1.0 - LOG_MAP_TAIL_FRACTION) * k as f64 / 40.0;
            let s = draw_invariant(lo, hi, res, x);
            let est = invariant_measure(lo, hi, res, s) / s;
            assert!(
                (est - analytic).abs() < 1e-12 * analytic,
                "measure/s = {est:.12e} not constant at analytic {analytic:.12e}"
            );
        }
    }

    /// A kinematic edge already at or above the floor leaves no sub-floor region,
    /// so the logarithmic piece must take the whole draw. Were the linear piece
    /// still allotted its share of `x`, that share would map onto the zero-width
    /// interval `[t_lo, t0]` and carry zero measure — every point drawn there gets
    /// an infinite weight, and any channel evaluating its density at such a point
    /// reports zero. Both are caught here: the measure stays finite and positive,
    /// and the draw stays inside `[lo, hi]`, right across the `x` range the linear
    /// piece would otherwise have claimed.
    #[test]
    fn log_map_without_subfloor_region_stays_finite() {
        // `lo` above the 10 GeV^2 floor: `t_lo = lo > floor`, so `frac = 0`.
        let (lo, hi) = (400.0_f64, 250_000.0);
        let res = Some(photon_resonance());
        let m = log_scale(lo, hi, res).expect("massless pole above the floor still log-maps");
        assert_eq!(m.frac, 0.0, "no sub-floor region means no linear piece");
        assert_eq!(m.t0, m.t_lo, "the log piece starts at the kinematic edge");
        for k in 0..=100 {
            let x = k as f64 / 100.0;
            let s = draw_invariant(lo, hi, res, x);
            let w = invariant_measure(lo, hi, res, s);
            assert!(
                (lo..=hi).contains(&s),
                "draw left the range at x = {x}: s = {s:.6e}"
            );
            assert!(
                w > 0.0 && w.is_finite(),
                "measure not finite and positive at x = {x}: s = {s:.6e}, w = {w:.6e}"
            );
        }
    }

    /// The log map only claims the draw when the pole sits at or below the
    /// kinematic floor and the range is wide enough to hold the floor. A
    /// finite-width pole must still route to the Breit–Wigner branch, and a
    /// zero-width pole *inside* the range must fall back to the flat draw rather
    /// than take a log map whose shifted lower limit would be negative.
    #[test]
    fn log_map_claims_only_zero_width_poles_below_threshold() {
        let hi = 250_000.0_f64;
        assert!(
            log_scale(0.0, hi, Some(z_resonance())).is_none(),
            "a finite-width pole belongs to the Breit-Wigner branch"
        );
        assert!(
            log_scale(0.0, hi, None).is_none(),
            "a line with no pole keeps the flat draw"
        );
        // Zero-width pole above the kinematic floor: singularity inside the range.
        let heavy = Resonance {
            mass: 100.0,
            width: 0.0,
        };
        assert!(
            log_scale(0.0, hi, Some(heavy)).is_none(),
            "a zero-width pole inside the range is not regulated by the log map"
        );
        assert!(
            log_scale(100.0 * 100.0, hi, Some(heavy)).is_some(),
            "the same pole at the kinematic floor does take the log map"
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
        // (√s, outgoing masses, one (subsystem, resonance) list per channel).
        type Case = (f64, Vec<f64>, Vec<(Vec<usize>, Option<Resonance<f64>>)>);
        let cases: Vec<Case> = vec![
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

    // ── T-channel spine ─────────────────────────────────────────────────────────

    /// A massive-beam t-channel window with both endpoints strictly below zero,
    /// the regime where the peripheral map is well-defined (a massless beam pins
    /// `t_max = 0`, the collinear edge).
    fn t_window() -> (f64, f64, f64) {
        // √s = 500, equal 80 GeV beams, massless final state: t_max < 0.
        let tk = t_kinematics(500.0 * 500.0, 80.0 * 80.0, 80.0 * 80.0, 0.0, 0.0);
        (tk.t_min, tk.t_max, 0.0)
    }

    /// The t-draw is measure-preserving: averaging `dt/dx` over uniform `x`
    /// reproduces the flat range integral `∫ dt = t_max − t_min`, since the
    /// Jacobian exactly cancels the sampling density. A wrong `dt/dx` misses it.
    #[test]
    fn t_map_is_measure_preserving() {
        let (t_min, t_max, m2) = t_window();
        assert!(t_max < 0.0 && t_min < t_max, "window must be spacelike");
        let mut stream = SubStream::from_stream(0x7C01, 41);
        let n = 400_000;
        let (mut sum, mut sum_sq) = (0.0, 0.0);
        for _ in 0..n {
            let x = stream.uniforms::<f64>(1)[0];
            let t = draw_t(t_min, t_max, m2, x);
            assert!(t <= 0.0, "sampled t = {t} not spacelike");
            let w = t_measure(t_min, t_max, m2, t);
            sum += w;
            sum_sq += w * w;
        }
        let mean = sum / n as f64;
        let err = ((sum_sq / n as f64 - mean * mean).max(0.0) / n as f64).sqrt();
        let want = t_max - t_min;
        eprintln!("t-map ∫dt = {mean:.6e} ± {err:.2e}, want {want:.6e}");
        assert!(
            (mean - want).abs() < 5.0 * err,
            "t measure not preserving: ∫dt = {mean:.6e} ± {err:.2e} vs {want:.6e}"
        );
    }

    /// With the exact `dt/dx`, `measure(t)·(1/(t−m²))` is a constant at every `x`:
    /// the map's density is exactly `∝ 1/(m²−t)`, so an estimator of `∫ dt/(t−m²)`
    /// has zero variance. A wrong Jacobian breaks the constancy — the sharpest pin
    /// on `dt/dx`, tested for a massless and a massive spacelike propagator.
    #[test]
    fn t_map_zero_variance_on_propagator() {
        let (t_min, t_max, _) = t_window();
        for m2 in [0.0_f64, 91.1876 * 91.1876] {
            let analytic = -((m2 - t_min) / (m2 - t_max)).ln();
            for k in 0..=40 {
                let x = k as f64 / 40.0;
                let t = draw_t(t_min, t_max, m2, x);
                let est = t_measure(t_min, t_max, m2, t) * (1.0 / (t - m2));
                assert!(
                    (est - analytic).abs() < 1e-10 * analytic.abs(),
                    "measure·1/(t−m²) = {est:.12e} not constant at {analytic:.12e} (m²={m2})"
                );
            }
        }
    }

    /// Beam mass moves the spacelike bounds: a massless beam pins `t_max = 0` (the
    /// collinear edge), while a massive initial state pushes `t_max` strictly below
    /// zero — the boundary condition note-07 2.9.3 flags as the classic wrong
    /// default. Both bounds stay `≤ 0`.
    #[test]
    fn t_bounds_include_initial_state_mass() {
        let s = 500.0_f64 * 500.0;
        let massless = t_kinematics(s, 0.0, 0.0, 0.0, 0.0);
        assert!(
            massless.t_max.abs() < 1e-6,
            "massless beam should pin t_max = 0, got {}",
            massless.t_max
        );
        let massive = t_kinematics(s, 80.0 * 80.0, 80.0 * 80.0, 0.0, 0.0);
        assert!(
            massive.t_max < -1.0,
            "massive initial state must push t_max below 0, got {}",
            massive.t_max
        );
        assert!(massive.t_min < massive.t_max && massive.t_max <= 0.0);
    }

    /// Threshold kinematics: as `√s → (m₁+m₂)` the emitted momentum `p*` vanishes,
    /// so the whole spacelike window collapses to a point. A map that ignored the
    /// threshold would keep sampling a finite range and bias the integral.
    #[test]
    fn t_channel_threshold_window_collapses() {
        let (m1, m2) = (40.0_f64, 40.0);
        let (s1, s2) = (m1 * m1, m2 * m2);
        // Light beams so √s can approach the final-state threshold from above.
        let wide = t_kinematics(300.0 * 300.0, 0.0, 0.0, s1, s2);
        let near = t_kinematics((m1 + m2 + 0.01).powi(2), 0.0, 0.0, s1, s2);
        assert!(
            near.pstar < 1e-2 * wide.pstar,
            "p* did not collapse near threshold: {} vs {}",
            near.pstar,
            wide.pstar
        );
        let (wide_w, near_w) = (wide.t_max - wide.t_min, near.t_max - near.t_min);
        assert!(
            near_w < 1e-2 * wide_w,
            "spacelike window {near_w:.3e} did not collapse near threshold (wide {wide_w:.3e})"
        );
    }

    /// A single-rung spine at √s = 500 with 80 GeV beams and massless final legs.
    fn spine_channel() -> DiagramChannel<f64> {
        DiagramChannel::from_topology_tchannel(
            500.0,
            [80.0, 80.0],
            vec![0.0, 0.0],
            (vec![0], None),
            (vec![1], None),
            0.0,
        )
    }

    /// Every generated spine point is on shell, momentum-conserving, and its
    /// reconstructed transfer `t = (p_beam − p_emitted)²` is spacelike (`≤ 0`,
    /// note-07 2.8.0). A positive `t` would signal a broken bound or map.
    #[test]
    fn spine_on_shell_and_spacelike() {
        let ch = spine_channel();
        assert_eq!(ch.ndim(), 2);
        let beams = beam_momenta::<f64>(500.0, 80.0, 80.0);
        let mut stream = SubStream::from_stream(0x7C02, 43);
        for _ in 0..5000 {
            let u = stream.uniforms::<f64>(ch.ndim());
            let pt = ch.sample(&u);
            let tot = total(&pt.momenta);
            assert!((tot[0] - 500.0).abs() < 1e-6 * 500.0, "energy: {}", tot[0]);
            for c in &tot[1..] {
                assert!(c.abs() < 1e-6 * 500.0, "3-momentum: {c}");
            }
            for p in &pt.momenta {
                assert!(p.m2().abs() < 1e-4, "off shell: m² = {}", p.m2());
            }
            let t = transfer_invariant(beams[0], pt.momenta[0]);
            assert!(t <= 1e-6, "reconstructed t = {t} not spacelike");
            assert!(pt.weight > 0.0 && pt.weight.is_finite());
        }
    }

    /// Importance sampling reshapes variance, not volume: the peripheral spine still
    /// integrates to the analytic massless phase-space volume `V_2`, independent of
    /// the beam mass that shapes its `t` window.
    #[test]
    fn t_channel_volume_still_v2() {
        let ch = spine_channel();
        let (mean, err) = mc_volume(&ch, 0x7C03, 400_000);
        let analytic = massless_volume(500.0, 2);
        eprintln!(
            "spine V_2 = {mean:.6e} ± {err:.2e}, analytic {analytic:.6e}, diff {:.2e}",
            (mean - analytic).abs()
        );
        // A 2→2 spine has a single sampled invariant `t`; its weight varies, so use
        // the MC error bar with a floor.
        let tol = (6.0 * err).max(1e-9 * analytic);
        assert!(
            (mean - analytic).abs() < tol,
            "spine V_2 {mean:.6e} ± {err:.2e} vs analytic {analytic:.6e}"
        );
    }

    /// The spine's walk-accumulated weight is the reciprocal of the density read
    /// back off the momenta, and that density stays finite and non-negative on a
    /// foreign on-shell configuration (here a flat-RAMBO point) — the contract the
    /// multichannel combiner relies on.
    ///
    /// The first half is the peripheral rung's share of
    /// [`density_is_reciprocal_weight`]: `sample` accumulates `dt/dx` at the `t` it
    /// drew, `density` recomputes `t` from the emitted subsystem's momenta. A rung
    /// whose draw and weight describe different maps fails here.
    #[test]
    fn spine_density_reciprocal_and_foreign() {
        let ch = spine_channel();
        let mut stream = SubStream::from_stream(0x7C04, 45);
        for _ in 0..2000 {
            let u = stream.uniforms::<f64>(ch.ndim());
            let pt = ch.sample(&u);
            let recip = 1.0 / pt.weight;
            assert!(
                (ch.density(&pt.momenta) - recip).abs() <= 1e-9 * recip,
                "spine density {} not reciprocal of weight {}",
                ch.density(&pt.momenta),
                pt.weight
            );
        }
        // Foreign on-shell points the spine did not generate: density stays finite,
        // non-negative, and reads the same transfer off the momenta.
        let rambo = RamboChannel::new(500.0, vec![0.0, 0.0]);
        let mut fstream = SubStream::from_stream(0x7C05, 47);
        for _ in 0..2000 {
            let u = fstream.uniforms::<f64>(rambo.ndim());
            let pt = rambo.sample(&u);
            let g = ch.density(&pt.momenta);
            assert!(
                g.is_finite() && g >= 0.0,
                "foreign spine density {g} invalid"
            );
        }
    }

    /// The transfer of the emitted subsystem, evaluated frame-independently from the
    /// final momenta, matches the transfer against the *recoil* side by momentum
    /// conservation: `(p_a − p_emitted)² = (p_recoil − p_b)²`. This pins that the
    /// emitted blob is paired with beam 0 (not beam 1 — that would read the crossed
    /// `u`-channel invariant), the note-07 2.9.0 ordering hazard.
    #[test]
    fn spine_transfer_pairs_emitted_with_beam0() {
        let ch = spine_channel();
        let beams = beam_momenta::<f64>(500.0, 80.0, 80.0);
        let mut stream = SubStream::from_stream(0x7C06, 49);
        for _ in 0..2000 {
            let u = stream.uniforms::<f64>(ch.ndim());
            let pt = ch.sample(&u);
            let t_beam0 = transfer_invariant(beams[0], pt.momenta[0]);
            let t_beam1 = transfer_invariant(beams[1], pt.momenta[1]);
            assert!(
                (t_beam0 - t_beam1).abs() < 1e-6 * t_beam0.abs().max(1.0),
                "emitted/recoil transfer mismatch: {t_beam0} vs {t_beam1}"
            );
        }
    }

    /// Ordering firing test (note-07 2.9.0): the peripheral map is forward-peaked
    /// along the anchoring beam, so the emitted subsystem drifts toward beam 0
    /// (`⟨p_z⟩ > 0`) and the recoil toward beam 1 (`⟨p_z⟩ < 0`). Swapping which leg
    /// the rung emits flips both signs — a silent swap of the rung's emitted/recoil
    /// assignment changes the physics, so the test fails under the wrong ordering.
    #[test]
    fn spine_emitted_is_forward_biased() {
        let forward = |ch: &DiagramChannel<f64>, seed: u64| -> (f64, f64) {
            let mut stream = SubStream::from_stream(seed, 51);
            let (mut pz0, mut pz1) = (0.0, 0.0);
            let n = 200_000;
            for _ in 0..n {
                let u = stream.uniforms::<f64>(ch.ndim());
                let pt = ch.sample(&u);
                pz0 += pt.momenta[0].pz();
                pz1 += pt.momenta[1].pz();
            }
            (pz0 / n as f64, pz1 / n as f64)
        };
        let (pz0, pz1) = forward(&spine_channel(), 0x7C07);
        eprintln!("emitted ⟨pz⟩ = {pz0:.3}, recoil ⟨pz⟩ = {pz1:.3}");
        assert!(
            pz0 > 1.0,
            "emitted subsystem not forward-biased: ⟨pz⟩ = {pz0}"
        );
        assert!(
            pz1 < -1.0,
            "recoil subsystem not backward-biased: ⟨pz⟩ = {pz1}"
        );

        // Swapping the emitted leg flips the bias — the ordering is load-bearing.
        let swapped = DiagramChannel::from_topology_tchannel(
            500.0,
            [80.0, 80.0],
            vec![0.0, 0.0],
            (vec![1], None),
            (vec![0], None),
            0.0,
        );
        let (spz0, spz1) = forward(&swapped, 0x7C08);
        assert!(
            spz0 < -1.0 && spz1 > 1.0,
            "swapped ordering did not flip the forward bias: {spz0}, {spz1}"
        );
    }

    /// The payoff: on a forward-peaked t-channel integrand `1/(t−m_t²)²`, the
    /// peripheral spine's per-point variance is far below flat RAMBO's at fixed `N`,
    /// while both agree on the integral. Per AGENTS.md this variance win does *not*
    /// confirm the ordering convention — the firing tests above do that.
    #[test]
    fn t_channel_beats_flat_rambo_variance() {
        let beams = beam_momenta::<f64>(500.0, 80.0, 80.0);
        let m_t2 = 0.0_f64;
        let integrand = move |p: &[LorentzVector<f64>]| {
            let t = transfer_invariant(beams[0], p[0]);
            1.0 / (t - m_t2).powi(2)
        };
        let ch = spine_channel();
        let rb = RamboChannel::new(500.0, vec![0.0, 0.0]);

        let n = 400_000;
        let (mean_ch, var_ch) = mc_integrand(&ch, 0x7C09, n, integrand);
        let (mean_rb, var_rb) = {
            let mut stream = SubStream::from_stream(0x7C0A, 53);
            let (mut sum, mut sum_sq) = (0.0, 0.0);
            for _ in 0..n {
                let u = stream.uniforms::<f64>(rb.ndim());
                let pt = rb.sample(&u);
                let v = pt.weight * integrand(&pt.momenta);
                sum += v;
                sum_sq += v * v;
            }
            let mean = sum / n as f64;
            (mean, (sum_sq / n as f64 - mean * mean).max(0.0))
        };
        let err = ((var_ch + var_rb) / n as f64).sqrt();
        eprintln!(
            "t-channel σ: spine {mean_ch:.6e} (var {var_ch:.3e}) vs flat {mean_rb:.6e} \
             (var {var_rb:.3e}); var ratio {:.2e}",
            var_rb / var_ch
        );
        assert!(
            (mean_ch - mean_rb).abs() < 6.0 * err,
            "t-channel σ disagrees: spine {mean_ch:.6e} vs flat {mean_rb:.6e} (err {err:.2e})"
        );
        assert!(
            var_ch < var_rb,
            "spine variance {var_ch:.3e} not below flat RAMBO {var_rb:.3e}"
        );
    }

    /// `from_diagram` builds a t-channel spine for a real 2→2 process (`u u~ > u u~`,
    /// t-channel gluon), and the emitted subsystem it pairs with beam 0 matches the
    /// independent graph cut of the spacelike line. This pins the peripheral
    /// pairing directly off the diagram, with no reliance on a cross section.
    #[test]
    fn spine_built_for_real_t_channel_process() {
        let m = sm_model(SMRestrict::Default);
        let ev = EvaluatedModel::from_model(m.clone());
        let opts = ParsingOptions::default();
        let card = parse_proc_card("generate u u~ > u u~", &opts).unwrap();
        let sets = generate_from_proc_card(&card, &m).unwrap();
        let mut spines = 0usize;
        for set in &sets {
            for d in &set.diagrams {
                let n_in = d.n_in;
                let n_ext = d.n_ext();
                let spacelike: Vec<usize> = (0..d.props.len())
                    .filter(|&pi| d.props[pi].is_spacelike(n_in))
                    .collect();
                if spacelike.len() != 1 || n_ext - n_in != 2 {
                    continue;
                }
                // The momentum-based emitted mask must match the graph cut's
                // beam-0-side outgoing legs.
                let (emitted, _) = spine_partition(&d.props[spacelike[0]].momentum, n_in, n_ext);
                let side0 = cut_side_externals(d, spacelike[0]);
                let has_beam0 = side0.contains(&0);
                let mut want = 0u64;
                for e in n_in..n_ext {
                    let on_beam0_side = if has_beam0 {
                        side0.contains(&e)
                    } else {
                        !side0.contains(&e)
                    };
                    if on_beam0_side {
                        want |= 1 << (e - n_in);
                    }
                }
                assert_eq!(
                    emitted, want,
                    "emitted mask disagrees with graph cut (mom {:?})",
                    d.props[spacelike[0]].momentum
                );

                let ch = DiagramChannel::<f64>::from_diagram(d, &ev, 500.0);
                assert_eq!(ch.ndim(), 2, "a 2→2 spine channel has ndim 2");
                let beams = beam_momenta::<f64>(500.0, 0.0, 0.0);
                let mut stream = SubStream::from_stream(0x7C0B, 55);
                for _ in 0..500 {
                    let u = stream.uniforms::<f64>(ch.ndim());
                    let pt = ch.sample(&u);
                    let t = transfer_invariant(beams[0], pt.momenta[0]);
                    assert!(t <= 1e-6, "u u~ spine t = {t} not spacelike");
                }
                spines += 1;
            }
        }
        assert!(spines > 0, "no t-channel spine built for u u~ > u u~");
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
        for (bit, &in_side_a) in in_a[n_in..n_ext].iter().enumerate() {
            // The beam-free side is `side_a` when it holds no beam, else its
            // complement; either way, take its outgoing legs.
            let on_zero_beam_side = if beams_a == 0 { in_side_a } else { !in_side_a };
            if on_zero_beam_side {
                mask |= 1 << bit;
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
