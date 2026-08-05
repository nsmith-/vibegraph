//! Flavour decomposition of a hadron-collider process: which of its enumerated
//! subprocesses share one matrix element, and which parton-distribution
//! luminosity each of them carries.
//!
//! A proc card written with beam multiparticles (`p p > l+ l- j`) expands into
//! many concrete subprocesses, most of which are the *same* partonic calculation
//! with a different flavour label on the incoming legs. Compiling and evaluating
//! each of them separately would be arithmetic wasted on a distinction the matrix
//! element does not make. [`derive_flavor_groups`] partitions them into
//! [`FlavorGroup`]s — one compiled amplitude, one phase-space map, one cut filter
//! per group — and gives each group the summed luminosity of the flavours inside
//! it.
//!
//! # The grouping rule
//!
//! Two subprocesses join a group when their `|M|²` agree, to
//! [`GROUP_REL_TOL`], at a shared set of probe phase-space points spanning
//! several partonic energies. Nothing is hand-listed: the coupling classes
//! (up-type vs down-type), the generation copies (`u`/`c`, `d`/`s`), the lepton
//! flavours (`e`/`mu`) and the separation of `q g` from `q̄ g` all fall out of
//! the measurement. Grouping is refused unless the members also share the
//! *outgoing pole masses* (so one phase-space map serves the group), an equal
//! [`Cuts`] filter (so one cut indicator does) and an equal colour basis (so an
//! event's colour flow can be read off the representative), and unless distinct
//! groups separate by more than [`GROUP_SEPARATION_MIN`] — a partition resting
//! on a knife edge is a measurement that failed, not a decomposition.
//!
//! The extra requirements are there because `|M|²` is a sum: it is blind to a
//! global phase, and it would not move if two members' colour bases differed by a
//! relabelling. `|M|²` equality alone is therefore not enough to license reusing
//! one member's colour flow, cuts or phase-space map for another.
//!
//! # Both beam orderings
//!
//! Diagram enumeration emits **one ordering per unordered initial state**: `g u`
//! is generated, `u g` is not. Both are physical — the parton distributions of
//! the two beams are evaluated at different momentum fractions — so the missing
//! ordering must be restored, and it is not free. The identity is
//!
//! ```text
//! |M_{b a}(p₁, p₂, q)|² = |M_{a b}(p₁, p₂, R q)|²,   R: (E, pₓ, p_y, p_z) ↦ (E, pₓ, −p_y, −p_z)
//! ```
//!
//! `R` is the rotation by π about the x axis, which maps a partonic-CM beam
//! momentum onto the other beam's (`R p₁ = p₂`); rotating the mirrored
//! configuration by it puts the beams back where the amplitude expects them and
//! leaves the outgoing legs reflected. So a group contributes
//!
//! ```text
//! xf_a(x₁)·xf_b(x₂)·|M(q)|²  +  xf_b(x₁)·xf_a(x₂)·|M(R q)|²
//! ```
//!
//! under one cut indicator: `R` is an argument to the matrix element, not a
//! change to the event, whose final state stays `q`. [`FlavorGroup::mirror_into`]
//! builds the reflected argument and [`FlavorGroup::luminosity`] returns the two
//! sums. An initial state with `a == b` has only one ordering and contributes no
//! mirror term.
//!
//! # Identical outgoing particles
//!
//! The symmetry factor `1/Π_s n_s!` belongs to a subprocess's outgoing multiset,
//! which the grouping rule does not constrain: members share `|M|²` and outgoing
//! *masses*, and `p p → j j` puts `g g → g g` (`1/2`) and `q q̄ → q q̄` (`1`) in
//! different groups with the same mass list `[0, 0]`. Every member therefore
//! carries its own factor ([`Subprocess::symmetry_factor`]) into the luminosity
//! sum ([`FlavorGroup::symmetry_weighted_luminosity`]), which is where the sum over
//! subprocesses can still tell them apart.
//!
//! # The integrand
//!
//! [`ProtonIntegrand`] convolves the decomposition with parton distributions over
//! a `(τ, y)` outer map and a per-diagram multichannel inner map, and presents the
//! result as a [`ChannelIntegrand`] so per-channel VEGAS banking, frozen-grid scans
//! and accept/reject need no hadronic special case. Its master formula, frames and
//! change of variables are documented on the type.

use std::cell::{Cell, RefCell};
use std::f64::consts::PI;
use std::sync::atomic::{AtomicU64, Ordering};

use rand::SeedableRng;
use rayon::prelude::*;
use thiserror::Error;
use thread_local::ThreadLocal;

use crate::artifact::ChannelSampler;
use crate::coupling::alphas::AlphaSSource;
use crate::coupling::scales::{EventScales, ScaleError};
use crate::cuts::{CutError, Cuts, ExternalLeg};
use crate::diagrams::diagram::Diagram;
use crate::budget::{integrate_channels, BlockAllocation, Budget, ConvergenceReport};
use crate::diagrams::DiagramSet;
use crate::hadronic::{
    boost_z, compile_class, compile_scale_source, components, constant_scale_report,
    initial_spin_color_average, make_subs_scale_aware, process_external_legs, BoundSubprocess,
    ChannelIntegration, EventScaleSource, HadronicError, PointScales, RunningCouplingReport,
    SampledChannel, SubprocessProto, SCALE_PROBE_DRAWS, SCALE_PROBE_SEED, VEGAS_ALPHA_MAPPED,
};
use crate::helas::color::flow_tags::{ColorFlowTags, LegColor};
use crate::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
use crate::helas::repr::color::ColorRep;
use crate::helas::repr::lorentz::LorentzVector;
use crate::pdf::grid::AlphaSInfo;
use crate::pdf::{flavor_slot, FlavorRow, PdfMember, FLAVOR_SLOTS};
use crate::phasespace::rng::{SubStream, SCALE_DRAW_STREAM_BASE};
use crate::phasespace::{
    identical_particle_factor, kleiss_pittau_step, AlphaAdaptation, DiagramChannel, PhaseSpaceMap,
    RamboChannel, ScaledChannel, ScaledMultiChannel, GEV2_TO_PB,
};
use crate::runcard::RunCard;
use crate::select::select_index;
use crate::ufo::{EvaluatedModel, UFOModel};
use crate::unweight::ChannelIntegrand;
use crate::vegas::{IterationCombination, VegasResult};

type V = LorentzVector<f64>;

/// Relative agreement at which two subprocesses' `|M|²` count as one group's.
///
/// Members of a group are the *same* arithmetic on differently-labelled legs, so
/// the observed agreement is exact (measured: bit-for-bit over every `p p → ℓℓj`
/// group). The bound is loose against that and still eleven orders below the
/// closest measured separation between distinct groups.
pub const GROUP_REL_TOL: f64 = 1e-10;

/// Minimum relative separation between two distinct groups at their best-separated
/// probe point.
///
/// Without it a partition could be produced by two subprocesses landing either
/// side of [`GROUP_REL_TOL`] by rounding, which is not a physical distinction.
/// It is a floor and not evidence of a margin: the margin is measured instead by
/// `group_members_agree_where_the_partition_was_not_measured`, where the closest
/// pair of `p p → ℓℓj` groups separates by `0.74` at points the partition was not
/// fitted on — six orders above this floor, and asserted there to stay above
/// `0.1`.
pub const GROUP_SEPARATION_MIN: f64 = 1e-6;

/// Probe points drawn at each of [`probe_energies`]' three energies.
const PROBE_POINTS_PER_ENERGY: usize = 4;

/// RNG stream the probe draws from, distinct from any integration stream.
const PROBE_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

/// One concrete external flavour assignment inside a group: the subprocess an
/// event of this group is finally labelled with.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Subprocess {
    /// PDG codes of the two incoming partons, in the enumerated beam order.
    pub incoming: [i32; 2],
    /// PDG codes of the outgoing legs, in the group's shared leg order.
    pub outgoing: Vec<i32>,
    /// SU(3) rep of every leg, in the group's shared leg order (incoming first),
    /// read off *this* member's own compiled amplitude.
    ///
    /// A group's members share a matrix element but not necessarily a colour rep —
    /// a quark and an antiquark share `|M|²`, mass list, cut filter and colour-factor
    /// matrix — and the record layer needs the member's own reps to check the table
    /// it is about to write.
    pub colors: Vec<ColorRep>,
    /// This member's **own** per-flow `ICOLUP` tags, reordered into the group
    /// representative's flow indexing by [`flow_permutation`].
    ///
    /// Nothing downstream sees the permutation: the configuration draw, the
    /// `ICOLAMP` mask and the flow draw all happen in the representative's indexing,
    /// and labelling an event is a plain index into this table.
    pub flows: ColorFlowTags,
    /// `flow_permutation[f]` is the flow of this member's own colour basis that
    /// corresponds to flow `f` of the representative's. Kept for the tests and for
    /// failure messages; production reads [`Self::flows`], which has it applied.
    pub flow_permutation: Vec<usize>,
}

impl Subprocess {
    /// Whether the two beams carry different partons, and so whether the mirrored
    /// ordering is a second physical initial state rather than the same one.
    pub fn has_mirror(&self) -> bool {
        self.incoming[0] != self.incoming[1]
    }

    /// This subprocess's identical-particle symmetry factor `1/Π_s n_s!`, from its
    /// own outgoing flavours ([`identical_particle_factor`]).
    ///
    /// Read from the concrete assignment rather than from the group's
    /// representative, because the outgoing multiset is what the factor counts and
    /// nothing in the grouping rule holds it fixed across members.
    pub fn symmetry_factor(&self) -> f64 {
        identical_particle_factor(&self.outgoing)
    }
}

/// Which of a member's two beam orderings an event was assigned to.
///
/// The enumeration produces one ordering of each unordered initial state, so the
/// other is reached through the mirror identity rather than through a second
/// compiled subprocess.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BeamOrdering {
    /// The enumerated ordering: beam 1 carries [`Subprocess::incoming`]`[0]`, and
    /// the shared matrix element is evaluated at the point as drawn.
    Direct,
    /// The exchanged ordering: beam 1 carries [`Subprocess::incoming`]`[1]`, and
    /// the shared matrix element is evaluated at the rotated argument
    /// ([`FlavorGroup::mirror_into`]).
    Exchanged,
}

/// A set of subprocesses sharing one matrix element, one phase-space map and one
/// cut filter, differing only in the parton-distribution luminosity they carry.
pub struct FlavorGroup {
    representative: DiagramSet,
    evaluator: AmplitudeEvaluator,
    legs: Vec<ExternalLeg>,
    cuts: Cuts,
    members: Vec<Subprocess>,
    /// Per member, parallel to `members`: where its beam partons sit in a
    /// flavour row.
    member_slots: Vec<BeamSlots>,
    spin_color_avg: f64,
}

/// One member's luminosity addressing: the [`crate::pdf::flavor_slot`] of each
/// beam parton, resolved once at setup, and whether the two differ — a member
/// whose beams carry the same parton has no mirrored ordering.
#[derive(Debug, Clone, Copy)]
struct BeamSlots {
    slots: [usize; 2],
    mirrored: bool,
}

/// The two per-beam flavour rows one phase-space point reads: `x·f` at
/// `(x₁, μ²_F1)` and at `(x₂, μ²_F2)`, every flavour at once. Every subprocess
/// summed over the point reads these same two, whatever its beam flavours.
pub fn beam_rows(pdf: &PdfMember, x1: f64, x2: f64, mu_f: [f64; 2]) -> [FlavorRow; 2] {
    let mut rows = [[0.0; FLAVOR_SLOTS]; 2];
    pdf.xfx_all(x1, mu_f[0] * mu_f[0], &mut rows[0]);
    pdf.xfx_all(x2, mu_f[1] * mu_f[1], &mut rows[1]);
    rows
}

impl FlavorGroup {
    /// The compiled, helicity-pruned amplitude every member shares.
    pub fn evaluator(&self) -> &AmplitudeEvaluator {
        &self.evaluator
    }

    /// The representative subprocess's diagrams — the input the per-diagram
    /// phase-space channels are derived from.
    pub fn diagrams(&self) -> &[Diagram] {
        &self.representative.diagrams
    }

    /// The representative subprocess's enumerated particle names.
    pub fn diagram_set(&self) -> &DiagramSet {
        &self.representative
    }

    /// External legs in process order (incoming first), for the representative.
    pub fn external_legs(&self) -> &[ExternalLeg] {
        &self.legs
    }

    /// The cut filter every member compiles to.
    pub fn cuts(&self) -> &Cuts {
        &self.cuts
    }

    /// Concrete flavour assignments summed into this group.
    pub fn members(&self) -> &[Subprocess] {
        &self.members
    }

    /// `1 / Π_a (n_spin · n_colour)` over the incoming legs.
    pub fn spin_color_average(&self) -> f64 {
        self.spin_color_avg
    }

    /// Outgoing pole masses in leg order — the phase-space map's targets.
    pub fn final_masses(&self) -> Vec<f64> {
        self.legs[self.n_in()..].iter().map(|l| l.mass).collect()
    }

    /// Number of incoming legs; two for every hadronic subprocess.
    pub fn n_in(&self) -> usize {
        self.evaluator.n_in()
    }

    /// Whether any member has distinct beam partons, i.e. whether the mirrored
    /// ordering carries luminosity at all. A group of identical-parton initial
    /// states (`g g`) has one ordering and no mirror term to evaluate.
    pub fn has_mirror(&self) -> bool {
        self.members.iter().any(Subprocess::has_mirror)
    }

    /// The argument the shared `|M|²` takes for the mirrored beam ordering:
    /// the beams untouched, the outgoing legs rotated by π about the x axis.
    ///
    /// The rotation maps each beam momentum onto the other's, so evaluating the
    /// group's amplitude here is evaluating the subprocess with its two incoming
    /// flavours exchanged. `out` is overwritten and may be a reused buffer.
    pub fn mirror_into(&self, momenta: &[V], out: &mut Vec<V>) {
        out.clear();
        out.extend_from_slice(&momenta[..self.n_in()]);
        out.extend(
            momenta[self.n_in()..]
                .iter()
                .map(|p| V::new(p.e(), p.px(), -p.py(), -p.pz())),
        );
    }

    /// Summed `x·f` luminosity of the group, as `[direct, mirror]`:
    ///
    /// ```text
    /// direct = Σ_members xf_a(x₁, μ_F1) · xf_b(x₂, μ_F2)
    /// mirror = Σ_members xf_b(x₁, μ_F1) · xf_a(x₂, μ_F2)     (a ≠ b only)
    /// ```
    ///
    /// `direct` weights `|M(q)|²` and `mirror` weights `|M(R q)|²`
    /// ([`mirror_into`](Self::mirror_into)). `mu_f` is per beam in beam order,
    /// since MadGraph's `q2fact(1)` and `q2fact(2)` need not coincide.
    ///
    /// A member whose beams carry the same parton has a single ordering, so it
    /// contributes to `direct` only — counting it twice would double that
    /// subprocess's cross section.
    ///
    /// This reads the densities itself. Every group of a process evaluates them
    /// at the same two points, so a caller summing over groups reads the two
    /// beam rows once with [`beam_rows`] and takes
    /// [`luminosity_rows`](Self::luminosity_rows) instead.
    pub fn luminosity(&self, pdf: &PdfMember, x1: f64, x2: f64, mu_f: [f64; 2]) -> [f64; 2] {
        let [f1, f2] = beam_rows(pdf, x1, x2, mu_f);
        self.luminosity_rows(&f1, &f2)
    }

    /// [`luminosity`](Self::luminosity) off the two beam flavour rows directly.
    pub fn luminosity_rows(&self, f1: &FlavorRow, f2: &FlavorRow) -> [f64; 2] {
        let mut sums = [0.0; 2];
        for i in 0..self.members.len() {
            let m = self.member_luminosity_rows(i, f1, f2);
            sums[0] += m[0];
            sums[1] += m[1];
        }
        sums
    }

    /// [`luminosity`](Self::luminosity) with each member weighted by its own
    /// identical-particle symmetry factor — the combination the cross section takes.
    ///
    /// ```text
    /// Σ_members S_i · xf_a(x₁, μ_F1) · xf_b(x₂, μ_F2)
    /// ```
    ///
    /// The members share `|M|²`, so their `S_i` cannot be pulled out in front of the
    /// group unless they happen to agree: a group is a statement about the matrix
    /// element, not about the outgoing multiset.
    pub fn symmetry_weighted_luminosity(
        &self,
        pdf: &PdfMember,
        x1: f64,
        x2: f64,
        mu_f: [f64; 2],
    ) -> [f64; 2] {
        let [f1, f2] = beam_rows(pdf, x1, x2, mu_f);
        self.symmetry_weighted_luminosity_rows(&f1, &f2)
    }

    /// [`symmetry_weighted_luminosity`](Self::symmetry_weighted_luminosity) off
    /// the two beam flavour rows directly.
    pub fn symmetry_weighted_luminosity_rows(&self, f1: &FlavorRow, f2: &FlavorRow) -> [f64; 2] {
        let mut sums = [0.0; 2];
        for (i, member) in self.members.iter().enumerate() {
            let s = member.symmetry_factor();
            let m = self.member_luminosity_rows(i, f1, f2);
            sums[0] += s * m[0];
            sums[1] += s * m[1];
        }
        sums
    }

    /// One member's `[direct, mirror]` luminosity — the share that decides which
    /// concrete flavour an accepted event of this group is labelled with.
    pub fn member_luminosity(
        &self,
        member: usize,
        pdf: &PdfMember,
        x1: f64,
        x2: f64,
        mu_f: [f64; 2],
    ) -> [f64; 2] {
        let [f1, f2] = beam_rows(pdf, x1, x2, mu_f);
        self.member_luminosity_rows(member, &f1, &f2)
    }

    /// [`member_luminosity`](Self::member_luminosity) off the two beam flavour
    /// rows directly: two array reads per ordering, at slots resolved when the
    /// group was built.
    pub fn member_luminosity_rows(
        &self,
        member: usize,
        f1: &FlavorRow,
        f2: &FlavorRow,
    ) -> [f64; 2] {
        let m = self.member_slots[member];
        let [a, b] = m.slots;
        let direct = f1[a] * f2[b];
        let mirror = if m.mirrored { f1[b] * f2[a] } else { 0.0 };
        [direct, mirror]
    }

    /// The external legs one member carries under one beam ordering, as an event
    /// record sees them: the PDG code of every leg in **physical** order (beam 1
    /// first), and the permutation `order[physical] = representative` that carries
    /// the shared amplitude's per-leg data onto those legs.
    ///
    /// Under [`BeamOrdering::Exchanged`] the two beams trade places. The mirrored
    /// term evaluates the shared amplitude at the rotated argument
    /// ([`mirror_into`](Self::mirror_into)), and that rotation maps each beam
    /// momentum onto the other's, so everything the representative says about its
    /// leg 0 — colour lines, helicity, mass — describes the event's *second* beam.
    /// The rotation is a proper one, so a helicity is carried across unchanged.
    /// The outgoing legs are untouched: the mirror is an argument to the matrix
    /// element, not a second final state.
    pub fn event_legs(&self, member: usize, ordering: BeamOrdering) -> (Vec<i32>, Vec<usize>) {
        let m = &self.members[member];
        let mut order: Vec<usize> = (0..self.legs.len()).collect();
        let mut incoming = m.incoming;
        if ordering == BeamOrdering::Exchanged {
            order.swap(0, 1);
            incoming.swap(0, 1);
        }
        let pdg = incoming
            .iter()
            .chain(m.outgoing.iter())
            .copied()
            .collect::<Vec<i32>>();
        (pdg, order)
    }

    /// The colour reps one member carries under one beam ordering, in the same
    /// **physical** leg order [`event_legs`](Self::event_legs) reports its codes in.
    ///
    /// These are the member's own reps, not the group representative's; the two
    /// differ wherever a group joins subprocesses that conjugate some of their legs.
    /// `incoming` is positional, since the beams are the first two legs whichever
    /// parton is on them.
    pub fn event_leg_colors(&self, member: usize, ordering: BeamOrdering) -> Vec<LegColor> {
        let mut colors = self.members[member].colors.clone();
        if ordering == BeamOrdering::Exchanged {
            colors.swap(0, 1);
        }
        let n_in = self.n_in();
        colors
            .into_iter()
            .enumerate()
            .map(|(leg, rep)| LegColor {
                rep,
                incoming: leg < n_in,
            })
            .collect()
    }
}

/// The flavour decomposition of one hadronic process.
pub struct FlavorGroups {
    groups: Vec<FlavorGroup>,
}

impl FlavorGroups {
    pub fn groups(&self) -> &[FlavorGroup] {
        &self.groups
    }

    /// Total number of concrete subprocesses summed, over all groups — the count
    /// the enumeration produced.
    pub fn subprocess_count(&self) -> usize {
        self.groups.iter().map(|g| g.members.len()).sum()
    }
}

#[derive(Debug, Error)]
pub enum ProtonError {
    #[error(transparent)]
    Hadronic(#[from] HadronicError),
    #[error("cut compilation failed: {0}")]
    Cut(#[from] CutError),
    #[error("the enumeration produced no subprocess with diagrams")]
    NoSubprocess,
    #[error("subprocess {process} has {n_in} incoming legs; a beam decomposition needs 2")]
    NotTwoIncoming { process: String, n_in: usize },
    #[error(
        "subprocess {process} has an incoming leg of mass {mass} GeV; a parton distribution \
         supplies massless partons and the probe puts the beams on the light cone"
    )]
    MassiveInitialState { process: String, mass: f64 },
    #[error(
        "subprocesses {a} and {b} have different outgoing masses, so no single phase-space map \
         covers the sum"
    )]
    UnequalFinalMasses { a: String, b: String },
    #[error(
        "subprocesses {a} and {b} share a matrix element but compile to different cut filters, \
         so no single cut indicator covers the sum"
    )]
    CutIndicatorDiffers { a: String, b: String },
    #[error(
        "subprocesses {a} and {b} share a matrix element but not a colour basis, so the flow an \
         event of {b} is labelled with cannot be read off {a}"
    )]
    ColorStructureDiffers { a: String, b: String },
    #[error(
        "subprocesses {a} and {b} share a matrix element, but their colour flows cannot be paired \
         up: {reason}. An event of {b} is labelled with a flow drawn in {a}'s indexing, so without \
         that pairing the label would be a guess"
    )]
    ColorFlowPairing { a: String, b: String, reason: String },
    #[error(
        "the groups represented by {a} and {b} separate by only {rel:.3e} at their best-separated \
         probe point; a partition that close is rounding, not a coupling distinction"
    )]
    DegenerateGroups { a: String, b: String, rel: f64 },
    #[error(
        "groups {a} and {b} compile to different cut filters, so the sum has no single phase-space \
         indicator; a process whose groups are cut differently needs one indicator per group"
    )]
    GroupCutsDiffer { a: usize, b: usize },
    #[error(
        "{amps} bound amplitudes were supplied for {groups} flavour groups, in an order that must \
         match one for one"
    )]
    AmplitudeCount { amps: usize, groups: usize },
    #[error(
        "the bound amplitude at position {index} was not built from group {index}'s evaluator, so \
         the luminosity of one group would weight another's matrix element"
    )]
    AmplitudeMismatch { index: usize },
}

/// Partonic energies the grouping probe measures at, spread over two decades so
/// a coincidence at one energy cannot survive.
///
/// The rungs are scaled by the outgoing pole masses, with a floor for the
/// massless case, and every rung is pushed above the final state's own threshold
/// so a heavy final state is probed above it rather than at an energy RAMBO
/// cannot fill. Two rungs are there for a specific blind spot each:
///
/// * the lowest, a fifth of the base, because the integrator routinely visits
///   `ŝ` below the electroweak scale, and a pair of subprocesses agreeing over a
///   ladder that starts above it while differing below would be merged silently;
/// * `resonance`, the `s`-channel pole the model supplies, because nothing else
///   in the ladder is deliberately placed on one, and a propagator on its pole is
///   where two subprocesses' weak content separates most sharply.
///
/// Rungs that collide after the threshold clamp collapse to one, so a heavy final
/// state costs fewer points rather than repeating a point.
fn probe_energies(final_masses: &[f64], resonance: Option<f64>) -> Vec<f64> {
    let threshold: f64 = final_masses.iter().sum();
    let base = threshold.max(100.0);
    let floor = 1.2 * threshold;
    let mut rungs: Vec<f64> = [
        Some(0.2 * base),
        resonance,
        Some(3.0 * base),
        Some(5.0 * base),
        Some(13.0 * base),
    ]
    .into_iter()
    .flatten()
    .map(|e| e.max(floor))
    .collect();
    rungs.sort_by(f64::total_cmp);
    rungs.dedup_by(|a, b| (*a - *b).abs() <= 1e-9 * a.abs().max(b.abs()));
    rungs
}

/// Partonic-CM probe points: massless beams along ±z and a flat RAMBO draw over
/// the outgoing legs, at each of [`probe_energies`].
fn probe_momenta(final_masses: &[f64], resonance: Option<f64>, seed: u64) -> Vec<Vec<V>> {
    let mut points = Vec::new();
    for (i, sqrt_s) in probe_energies(final_masses, resonance)
        .into_iter()
        .enumerate()
    {
        let rambo = RamboChannel::<f64>::new(sqrt_s, final_masses.to_vec());
        let mut stream = SubStream::from_stream(seed, i as u64);
        for _ in 0..PROBE_POINTS_PER_ENERGY {
            let u = stream.uniforms::<f64>(rambo.ndim());
            let drawn = rambo.sample(&u);
            let e = sqrt_s / 2.0;
            let mut momenta = vec![V::new(e, 0.0, 0.0, e), V::new(e, 0.0, 0.0, -e)];
            momenta.extend(drawn.momenta.iter().cloned());
            points.push(momenta);
        }
    }
    points
}

/// The `s`-channel pole the probe ladder places a rung on: the model's `Z` mass,
/// or nothing if the model has no `Z` or leaves it massless.
fn s_channel_resonance(model: &UFOModel, evaluated: &EvaluatedModel) -> Option<f64> {
    let mass = evaluated.mass(model.particle_id("Z")?);
    (mass > 0.0).then_some(mass)
}

/// Worst relative disagreement between two probe traces.
fn worst_rel(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).abs() / x.abs().max(y.abs()).max(f64::MIN_POSITIVE))
        .fold(0.0f64, f64::max)
}

/// `u u~ > e+ e- g`-style label for an enumerated subprocess.
fn label(set: &DiagramSet) -> String {
    format!(
        "{} > {}",
        set.particles_in.join(" "),
        set.particles_out.join(" ")
    )
}

/// Partition a hadronic enumeration into flavour groups.
///
/// `sets` are the `DiagramSet`s of one proc card (empty ones ignored), `card` the
/// run card whose cuts every member must compile to alike. See the module
/// documentation for the rule and for what it refuses.
pub fn derive_flavor_groups(
    sets: Vec<DiagramSet>,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    card: &RunCard,
) -> Result<FlavorGroups, ProtonError> {
    let sets: Vec<DiagramSet> = sets
        .into_iter()
        .filter(|s| !s.diagrams.is_empty())
        .collect();
    if sets.is_empty() {
        return Err(ProtonError::NoSubprocess);
    }

    let labels: Vec<String> = sets.iter().map(label).collect();

    let mut compiled = Vec::with_capacity(sets.len());
    for (set, process) in sets.iter().zip(&labels) {
        let evaluator = compile_class(set, model, evaluated)?;
        if evaluator.n_in() != 2 {
            return Err(ProtonError::NotTwoIncoming {
                process: process.clone(),
                n_in: evaluator.n_in(),
            });
        }
        let legs = process_external_legs(&evaluator, model, evaluated);
        if let Some(leg) = legs[..2].iter().find(|l| l.mass != 0.0) {
            return Err(ProtonError::MassiveInitialState {
                process: process.clone(),
                mass: leg.mass,
            });
        }
        let cuts = Cuts::compile(card, &legs)?;
        compiled.push((evaluator, legs, cuts));
    }

    let final_masses: Vec<f64> = compiled[0].1[2..].iter().map(|l| l.mass).collect();
    for (i, (_, legs, _)) in compiled.iter().enumerate().skip(1) {
        let masses: Vec<f64> = legs[2..].iter().map(|l| l.mass).collect();
        if masses != final_masses {
            return Err(ProtonError::UnequalFinalMasses {
                a: labels[0].clone(),
                b: labels[i].clone(),
            });
        }
    }

    let points = probe_momenta(
        &final_masses,
        s_channel_resonance(model, evaluated),
        PROBE_SEED,
    );
    let traces: Vec<Vec<f64>> = compiled
        .iter()
        .map(|(evaluator, ..)| {
            let bound = BoundAmplitude::<f64>::bind(evaluator, evaluated);
            let mut scratch = bound.scratch_space();
            points
                .iter()
                .map(|k| bound.eval_m2(k, &mut scratch))
                .collect()
        })
        .collect();

    let mut partition: Vec<Vec<usize>> = Vec::new();
    for i in 0..compiled.len() {
        match partition
            .iter_mut()
            .find(|g| worst_rel(&traces[g[0]], &traces[i]) < GROUP_REL_TOL)
        {
            Some(group) => group.push(i),
            None => partition.push(vec![i]),
        }
    }

    for (a, ga) in partition.iter().enumerate() {
        for gb in &partition[a + 1..] {
            let rel = worst_rel(&traces[ga[0]], &traces[gb[0]]);
            if rel <= GROUP_SEPARATION_MIN {
                return Err(ProtonError::DegenerateGroups {
                    a: labels[ga[0]].clone(),
                    b: labels[gb[0]].clone(),
                    rel,
                });
            }
        }
    }

    let mut sets: Vec<Option<DiagramSet>> = sets.into_iter().map(Some).collect();
    let mut compiled: Vec<Option<_>> = compiled.into_iter().map(Some).collect();
    let mut groups = Vec::with_capacity(partition.len());
    for indices in &partition {
        let head = indices[0];
        let (head_eval, _, head_cuts) = compiled[head].as_ref().expect("group head unclaimed");
        for &i in &indices[1..] {
            let (member_eval, _, member_cuts) = compiled[i].as_ref().expect("member unclaimed");
            if member_cuts != head_cuts {
                return Err(ProtonError::CutIndicatorDiffers {
                    a: labels[head].clone(),
                    b: labels[i].clone(),
                });
            }
            // An event's flow is drawn in the representative's indexing, and each
            // member's own table is reindexed into it below. That reindexing needs a
            // bijection between the two bases, which needs them to have the same
            // number of flows and the same colour-factor matrix. `|M|²` on its own
            // does not imply either: it is a sum over the basis and would not move
            // if two members' bases differed by a relabelling.
            if member_eval.n_flows() != head_eval.n_flows()
                || member_eval.cf_matrix() != head_eval.cf_matrix()
            {
                return Err(ProtonError::ColorStructureDiffers {
                    a: labels[head].clone(),
                    b: labels[i].clone(),
                });
            }
        }
        let members = indices
            .iter()
            .map(|&i| {
                let (evaluator, legs, _) = compiled[i].as_ref().expect("member unclaimed");
                // The representative is a member of its own group, and there its
                // table *is* the one an event was drawn from, so there is no
                // correspondence to establish and the fingerprint is not consulted.
                // The condition is an identity of objects — the same compiled
                // subprocess — not an equality of process strings, leg reps or
                // fingerprints, which is what keeps this a scoping of the question
                // rather than an answer to it. Between two distinct subprocesses the
                // refusal below stands unconditionally.
                let flow_permutation = if i == head {
                    (0..evaluator.n_flows()).collect()
                } else {
                    flow_permutation(head_eval, evaluator).map_err(|reason| {
                        ProtonError::ColorFlowPairing {
                            a: labels[head].clone(),
                            b: labels[i].clone(),
                            reason,
                        }
                    })?
                };
                // The member's own table, put into the representative's flow
                // indexing once, so no consumer downstream has to know about the
                // permutation.
                let own = evaluator.color_flow_tags();
                let flows = own
                    .reindexed(&flow_permutation)
                    .expect("flow_permutation is a permutation of this basis");
                Ok(Subprocess {
                    incoming: [legs[0].pdg, legs[1].pdg],
                    outgoing: legs[2..].iter().map(|l| l.pdg).collect(),
                    colors: evaluator.external_colors().iter().map(|l| l.rep).collect(),
                    flows,
                    flow_permutation,
                })
            })
            .collect::<Result<Vec<_>, ProtonError>>()?;
        let (evaluator, legs, cuts) = compiled[head].take().expect("group head unclaimed");
        let spin_color_avg = initial_spin_color_average(&evaluator, model, evaluated);
        let member_slots = members
            .iter()
            .map(|m| {
                let [a, b] = m.incoming;
                let slot = |pdg: i32| {
                    flavor_slot(pdg).unwrap_or_else(|| {
                        panic!("initial-state parton {pdg} is not a tabulated parton density")
                    })
                };
                BeamSlots {
                    slots: [slot(a), slot(b)],
                    mirrored: a != b,
                }
            })
            .collect();
        groups.push(FlavorGroup {
            representative: sets[head].take().expect("group head unclaimed"),
            evaluator,
            legs,
            cuts,
            members,
            member_slots,
            spin_color_avg,
        });
    }

    Ok(FlavorGroups { groups })
}

/// Pair up two subprocesses' colour flows: `π[f]` is the flow of `member`'s basis
/// built from the same contributions as flow `f` of `representative`'s.
///
/// Two subprocesses that share a matrix element carry the same set of colour
/// structures, but their bases are sorted by key and the keys differ, so the same
/// flow generally sits at a different index in each. Matching is on the structural
/// fingerprint — which diagram, through which colour-index chain, lands on the flow
/// at which power of `Nc` and with what rational magnitude — rather than on numbers
/// that agree at a probe point, because that is a statement about the colour algebra
/// and holds at every point by construction.
///
/// **Ambiguity is refused, never broken.** If either basis has two flows with equal
/// fingerprints, or no bijection exists, or more than one does, this returns an error
/// and the group is refused at setup. A wrong `π` is a silently wrong colour label on
/// every event of that member; a refusal is loud and cheap.
fn flow_permutation(
    representative: &AmplitudeEvaluator,
    member: &AmplitudeEvaluator,
) -> Result<Vec<usize>, String> {
    let ours = representative.flow_fingerprints();
    let theirs = member.flow_fingerprints();
    if ours.len() != theirs.len() {
        return Err(format!(
            "{} flows against {}",
            ours.len(),
            theirs.len()
        ));
    }
    for (side, keys) in [("the representative", ours), ("the member", theirs)] {
        for a in 0..keys.len() {
            if let Some(b) = (a + 1..keys.len()).find(|&b| keys[a] == keys[b]) {
                return Err(format!(
                    "flows {} and {} of {side}'s own basis have the same contributions, so no \
                     fingerprint can tell them apart",
                    a + 1,
                    b + 1
                ));
            }
        }
    }
    let mut pi = Vec::with_capacity(ours.len());
    for (f, key) in ours.iter().enumerate() {
        let hits: Vec<usize> = theirs
            .iter()
            .enumerate()
            .filter(|(_, other)| *other == key)
            .map(|(g, _)| g)
            .collect();
        match hits[..] {
            [g] => pi.push(g),
            [] => {
                return Err(format!(
                    "flow {} of the representative has no counterpart in the member's basis",
                    f + 1
                ))
            }
            _ => {
                return Err(format!(
                    "flow {} of the representative matches {} of the member's flows",
                    f + 1,
                    hits.len()
                ))
            }
        }
    }
    Ok(pi)
}

/// Unit-hypercube coordinates the `(τ, y)` outer map consumes. They are *prepended*
/// to each channel's own coordinates, so a channel grid is over
/// `OUTER_NDIM + channel_ndim`.
const OUTER_NDIM: usize = 2;

/// RNG substream the channel-weight survey draws on, kept distinct from the
/// per-channel integration streams so the survey and the integral neither share nor
/// correlate their sequences.
const ADAPT_STREAM: u64 = 0xA1FA_9110;

/// Survey points one rayon task evaluates before its partial variance shares are
/// reduced ([`ProtonIntegrand::survey_variance`]).
///
/// Fixed, rather than derived from the pool size as the per-iteration VEGAS chunking
/// is, because the survey reduces per-chunk partial sums: the chunk boundaries fix
/// the summation order, so the survey is thread-count independent only while the
/// split itself is. Sized so the smallest survey a run takes (10k points) still hands
/// a 16-thread pool several chunks each, while the per-chunk cost — two generator
/// seeks and one `n_channels` accumulator — stays far below the points in it.
const SURVEY_CHUNK: usize = 128;

/// Which diagram of which flavour group a sampling channel was derived from.
///
/// The channels of every group are pooled into one mixture, so a channel is
/// identified by the pair rather than by a diagram index alone — the key a banked
/// per-channel grid is stored under.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelId {
    pub group: usize,
    pub diagram: usize,
}

/// An accepted point, reconstructed from the coordinates it was drawn at.
///
/// The two frames are both kept because both are load-bearing: the record carries
/// the lab-frame momenta, while the labels are read off diagonals of the matrix
/// element, which lives in the partonic CM.
#[derive(Clone, Debug)]
pub struct ProtonEvent {
    /// The channel term's value here, the number
    /// [`value_in_channel`](ProtonIntegrand::value_in_channel) returns at the same
    /// coordinates.
    pub weight: f64,
    /// Beam momentum fractions `(x₁, x₂)`.
    pub x: [f64; 2],
    /// The scales the matrix element was evaluated at.
    pub scales: EventScales,
    /// Lab-frame external momenta, beams first — what the event record reports.
    pub lab: Vec<V>,
    /// Partonic-CM external momenta, beams first — the frame `|M|²` is taken in.
    pub cm: Vec<V>,
}

/// The discrete labels an accepted hadronic event carries besides its momenta.
///
/// Every one of them is summed over in the cross section, so none is a sampling
/// channel: they are read off accumulators after the fact, to fill in a record.
/// The concrete flavour is the one label with no fixed-beam counterpart — a
/// hadronic group is a sum over flavours, and which of them an event is labelled
/// with is decided by their parton-luminosity shares at the event's own `(x₁, x₂)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProtonSelection {
    /// Index into [`FlavorGroups::groups`].
    pub group: usize,
    /// Index into that group's [`FlavorGroup::members`].
    pub member: usize,
    /// Which beam carries which of the member's two partons.
    pub ordering: BeamOrdering,
    /// The helicity of each external leg, in the **physical** leg order — the same
    /// order [`FlavorGroup::event_legs`] returns codes in, so an exchanged
    /// ordering's beam helicities are already swapped.
    pub helicity: Vec<i32>,
    /// Index into the group **representative's** colour-flow basis — the indexing
    /// the draw is made in, off the representative's `JAMP2`.
    ///
    /// The members' bases are *not* equal: two subprocesses can share a matrix
    /// element and a colour-factor matrix while routing their colour lines between
    /// different pairs of legs. Each member's own table is reindexed into this
    /// indexing once at group construction, by the permutation stored on
    /// [`Subprocess::flow_permutation`], so the index is meaningful for every member
    /// without their bases agreeing. A beam exchange permutes the legs of a flow
    /// rather than the flows, and so does not touch it either.
    pub flow: usize,
}

/// A VEGAS point's outer coordinates, mapped to the partonic system.
#[derive(Clone, Copy, Debug)]
pub struct OuterPoint {
    /// Beam-1 momentum fraction.
    pub x1: f64,
    /// Beam-2 momentum fraction.
    pub x2: f64,
    /// The partonic collision energy `√ŝ = √(x₁x₂ s)`.
    pub sqrt_shat: f64,
    /// `dτ dy / (du₀ du₁)` with the `1/τ` from `f = (x·f)/x` on both legs already
    /// divided out.
    pub jac: f64,
}

/// A ready-to-integrate hadronic cross section for an arbitrary flavour-decomposed
/// process at proton beams (`lpp = 1`), sampled by a per-diagram multichannel map at
/// each event's own partonic energy.
///
/// # Master formula
///
/// ```text
/// σ = ∫ dτ dy dΦ_n  Σ_g avg_g · [ L_g^direct(x₁,x₂,μF) |M_g(q)|²
///                               + L_g^mirror(x₁,x₂,μF) |M_g(Rq)|² ] · Θ_cuts(q) / (2ŝ)
/// ```
///
/// summed over the [`FlavorGroup`]s of the process, with `L^direct`/`L^mirror` the
/// group's two beam orderings summed over its members, each member weighted by its
/// own identical-particle symmetry factor
/// ([`FlavorGroup::symmetry_weighted_luminosity`]), and `R` the mirror map
/// ([`FlavorGroup::mirror_into`]). There is **one** cut indicator, on the
/// unreflected final state: the mirror is an argument to the matrix element, not a
/// second event.
///
/// # Change of variables
///
/// The outer two coordinates are `(τ, y)` rather than `(x₁, x₂)`: a dilepton mass
/// window is a one-dimensional bound on `τ` rather than a thin diagonal band in
/// `(x₁, x₂)`, and VEGAS resolves the former far better.
///
/// ```text
/// τ = τ_min^(1−u₀)   (ln τ uniform),   dτ/du₀ = τ · ln(1/τ_min)
/// y = (2u₁ − 1)·y_max,  y_max = ½ ln(1/τ),   dy/du₁ = 2·y_max
/// ```
///
/// with `τ = ŝ/s = x₁x₂`, `y = ½ ln(x₁/x₂)` and `τ_min = ŝ_min/s` from
/// [`Cuts::shat_min`]. Since the grids return `x·f(x)` and `x₁x₂ = τ` matches the
/// `dτ` Jacobian, the `1/x₁x₂` in `f = (x·f)/x` cancels the `τ` and the luminosity is
/// built directly from `x·f` products, leaving the bare `ln(1/τ_min)·2·y_max`.
///
/// The remaining `3n−4` coordinates are the multichannel's, evaluated at the event's
/// own `√ŝ = √(τ s)` ([`ScaledMultiChannel`]) — the channel trees are `√ŝ`-independent
/// structures, so nothing is rebuilt per point.
///
/// # Frames
///
/// The matrix element is evaluated in the **partonic CM** with the beams along ±z,
/// the frame the helicity-pruned [`BoundAmplitude::eval_m2`] requires and the frame
/// the channel maps generate in. The cut filter and the scale prescription operate in
/// the **lab frame**, so the outgoing momenta are boosted along z by the partonic
/// rapidity `y` first.
pub struct ProtonIntegrand<'a> {
    groups: &'a FlavorGroups,
    subs: Vec<SubprocessProto<'a>>,
    pdf: &'a PdfMember,
    /// The one cut filter every group compiles to.
    cuts: &'a Cuts,
    combiner: ScaledMultiChannel<f64>,
    channel_ids: Vec<ChannelId>,
    /// What the composition rule chose for each channel, read off the channel as
    /// it was built and kept because the built channels are type-erased behind
    /// [`ScaledChannel`] afterwards.
    channel_samplers: Vec<ChannelSampler>,
    /// Total hadronic invariant `s = (E₁+E₂)²` (head-on beams).
    s_had: f64,
    sqrt_s_had: f64,
    /// Lower support of the logarithmic `τ = ŝ/s` map, `ŝ_min/s`.
    tau_min: f64,
    ln_inv_tau_min: f64,
    /// The `(2π)^{4−3n}` measure factor.
    lips_2pi: f64,
    scales: EventScaleSource,
    /// One evaluation context per thread that has evaluated a point, forked from
    /// `subs` on first use. See [`ProtonScratch`].
    scratch: ThreadLocal<ProtonScratch<'a>>,
    vegas_alpha: f64,
    /// Length the configuration draw's `AMP2` buffer is sized to — the widest
    /// group's configuration count — and zero where no draw runs.
    amp2_len: usize,
    /// The coupling the configuration draw forms `AMP2` at, so the drawn
    /// configuration is a function of the momenta and not of whatever scale the
    /// previous point left bound. `None` where no draw runs.
    amp2_alpha_s: Option<f64>,
    /// Points whose `AMP2` carried no probability at all, where the draw kept the
    /// sampling channel instead.
    scale_draw_fallbacks: AtomicU64,
}

/// One thread's private half of a [`ProtonIntegrand`].
///
/// Everything a point evaluation writes lives here, so the integrand itself is
/// immutable while an integration runs and can be shared across a rayon pool.
/// Nothing in it carries information from one point to the next that changes a
/// value: the amplitudes rescale from the card's own `αs` rather than from the
/// previous point's, and the coupling memo returns what a recomputation would.
/// A point's value is therefore the same whichever thread takes it.
struct ProtonScratch<'a> {
    /// This thread's own amplitudes ([`SubprocessProto::bind`]).
    subs: Vec<BoundSubprocess<'a>>,
    scale_buf: RefCell<Vec<[f64; 4]>>,
    cm_buf: RefCell<Vec<V>>,
    lab_buf: RefCell<Vec<V>>,
    mirror_buf: RefCell<Vec<V>>,
    /// The last `(μR, αs(μR))` pair, so a repeated scale does not repeat the
    /// coupling lookup.
    last_coupling: Cell<(f64, f64)>,
    /// Reused `AMP2` buffer for the configuration draw, sized to the widest
    /// group's configuration count.
    amp2_buf: RefCell<Vec<f64>>,
}

impl<'a> ProtonIntegrand<'a> {
    /// Build the integrand over a process's flavour decomposition.
    ///
    /// * `groups` — the decomposition ([`derive_flavor_groups`]).
    /// * `amps` — one bound amplitude per group, **in group order**, each bound from
    ///   that group's own [`FlavorGroup::evaluator`]. The pairing is checked by
    ///   identity rather than trusted: crossing it would weight one group's matrix
    ///   element with another's luminosity, which no cross-section-level check
    ///   separates from a coupling error.
    /// * `model` — the evaluated model the channel trees read pole masses and widths
    ///   from.
    /// * `sqrt_s_had` — total collider energy `√s = E₁ + E₂`.
    /// * `mu_f` — a constant scale on both beams, replaced by the run card's
    ///   prescription by [`use_run_card_scales`](Self::use_run_card_scales).
    ///
    /// Every group's diagrams contribute a channel and all of them are pooled into
    /// one mixture, so a peak one group's own diagrams do not cover — the mirrored
    /// `g q` configuration above all — is still covered by another group's.
    ///
    /// The peripheral channels are floored at [`Cuts::spacelike_floor`], the scale
    /// the process's own transverse-momentum cuts imply. The floor is passed because
    /// a final state of more than two legs has no peripheral spine without one; it
    /// also reshapes a `2 → 2` process's spine, which leaves the estimator unbiased
    /// but does change the map a banked run was taken with.
    pub fn new(
        groups: &'a FlavorGroups,
        amps: &'a [BoundAmplitude<'a, f64>],
        model: &EvaluatedModel,
        pdf: &'a PdfMember,
        sqrt_s_had: f64,
        mu_f: f64,
    ) -> Result<Self, ProtonError> {
        Self::build(groups, amps, model, pdf, sqrt_s_had, mu_f, true)
    }

    /// [`new`](Self::new) with the peripheral channels' fiducial transfer bound
    /// dropped, leaving only the regulating pole floor.
    ///
    /// Production always bounds. This builds the map the bound narrows, on the same
    /// channels, so what the bound is worth can be measured on a real integration
    /// rather than argued from the channel's own variance.
    pub fn new_unbounded(
        groups: &'a FlavorGroups,
        amps: &'a [BoundAmplitude<'a, f64>],
        model: &EvaluatedModel,
        pdf: &'a PdfMember,
        sqrt_s_had: f64,
        mu_f: f64,
    ) -> Result<Self, ProtonError> {
        Self::build(groups, amps, model, pdf, sqrt_s_had, mu_f, false)
    }

    #[allow(clippy::too_many_arguments)]
    fn build(
        groups: &'a FlavorGroups,
        amps: &'a [BoundAmplitude<'a, f64>],
        model: &EvaluatedModel,
        pdf: &'a PdfMember,
        sqrt_s_had: f64,
        mu_f: f64,
        bound_transfer: bool,
    ) -> Result<Self, ProtonError> {
        if amps.len() != groups.groups().len() {
            return Err(ProtonError::AmplitudeCount {
                amps: amps.len(),
                groups: groups.groups().len(),
            });
        }
        for (i, (g, amp)) in groups.groups().iter().zip(amps).enumerate() {
            if !std::ptr::eq(amp.evaluator(), g.evaluator()) {
                return Err(ProtonError::AmplitudeMismatch { index: i });
            }
        }
        let cuts = groups.groups()[0].cuts();
        for (i, g) in groups.groups().iter().enumerate().skip(1) {
            if g.cuts() != cuts {
                return Err(ProtonError::GroupCutsDiffer { a: 0, b: i });
            }
        }

        let floor = cuts.spacelike_floor();
        let mut channels: Vec<Box<dyn ScaledChannel<f64>>> = Vec::new();
        let mut channel_ids = Vec::new();
        let mut channel_samplers = Vec::new();
        for (gi, g) in groups.groups().iter().enumerate() {
            for (di, d) in g.diagrams().iter().enumerate() {
                // The baked-in energy is unread through `ScaledChannel`, which takes
                // the event's own; the collider energy is the well-formed value to
                // leave it at.
                let channel = DiagramChannel::from_diagram_regulated(d, model, sqrt_s_had, floor);
                let channel = if bound_transfer {
                    channel
                } else {
                    channel.without_transfer_bound()
                };
                channel_samplers.push(ChannelSampler::of(&channel));
                channels.push(Box::new(channel));
                channel_ids.push(ChannelId {
                    group: gi,
                    diagram: di,
                });
            }
        }

        let n_out = groups.groups()[0].final_masses().len();
        let s_had = sqrt_s_had * sqrt_s_had;
        let tau_min = cuts.shat_min() / s_had;
        Ok(ProtonIntegrand {
            subs: amps.iter().map(SubprocessProto::fixed).collect(),
            groups,
            pdf,
            cuts,
            combiner: ScaledMultiChannel::uniform(channels),
            channel_ids,
            channel_samplers,
            s_had,
            sqrt_s_had,
            tau_min,
            ln_inv_tau_min: (1.0 / tau_min).ln(),
            lips_2pi: (2.0 * PI).powi(4 - 3 * n_out as i32),
            scales: EventScaleSource::constant(mu_f),
            scratch: ThreadLocal::new(),
            amp2_len: 0,
            amp2_alpha_s: None,
            scale_draw_fallbacks: AtomicU64::new(0),
            vegas_alpha: VEGAS_ALPHA_MAPPED,
        })
    }

    /// This thread's evaluation context, forked from the integrand's own
    /// subprocesses the first time the thread evaluates a point.
    ///
    /// Setup takes `&mut self` and so runs before any of these exist; a fork
    /// therefore starts from the fully configured amplitudes.
    fn scratch(&self) -> &ProtonScratch<'a> {
        self.scratch.get_or(|| {
            let n_out = self.groups.groups()[0].final_masses().len();
            ProtonScratch {
                subs: self.subs.iter().map(SubprocessProto::bind).collect(),
                scale_buf: RefCell::new(Vec::with_capacity(n_out)),
                cm_buf: RefCell::new(Vec::with_capacity(2 + n_out)),
                lab_buf: RefCell::new(Vec::with_capacity(2 + n_out)),
                mirror_buf: RefCell::new(Vec::with_capacity(2 + n_out)),
                last_coupling: Cell::new((f64::NAN, f64::NAN)),
                amp2_buf: RefCell::new(vec![0.0; self.amp2_len]),
            }
        })
    }

    /// Discard every thread's evaluation context, so the next point forks a fresh
    /// one from the integrand's own subprocesses.
    ///
    /// Setup that changes what a subprocess evaluates — installing a scale
    /// prescription, moving the coupling — must run this, or a thread that has
    /// already evaluated a point would keep the amplitudes it forked before.
    fn reset_scratch(&mut self) {
        self.scratch.clear();
    }

    /// Take the renormalisation and per-beam factorisation scales — and the strong
    /// coupling `μR` implies — from the run card instead of the constant `μF`
    /// [`new`](Self::new) was given.
    ///
    /// `alpha_s` is the `AlphaS_*` metadata of the set the beams read their densities
    /// from. A card whose `pdlabel` delegates `αs` to LHAPDF is resolved from that
    /// tabulation, and refuses rather than falling back to a beta-function solve the
    /// densities were not fitted with.
    ///
    /// The channel forests the `-1` scale clusters against are derived per flavour
    /// group, from that group's own diagrams and external flavours — the groups of
    /// one process do not share a merge graph, and a sampling channel belongs to
    /// exactly one of them. A prescription that actually reads them is resolved
    /// once here on a sampled, cut-passing point, so a clustering this crate
    /// refuses stops the run at setup rather than at the first VEGAS point.
    pub fn use_run_card_scales(
        &mut self,
        model: &UFOModel,
        evaluated: &'a EvaluatedModel,
        card: &RunCard,
        alpha_s: Option<&AlphaSInfo>,
    ) -> Result<RunningCouplingReport, ProtonError> {
        let awareness = make_subs_scale_aware(&mut self.subs, evaluated);
        // Unlike a fixed-beam run, the factorisation scale has a consumer whatever
        // the matrix element is made of, so the prescription is compiled even when
        // nothing moves with the strong coupling.
        let subprocesses: Vec<(&AmplitudeEvaluator, &[Diagram])> = self
            .groups
            .groups()
            .iter()
            .map(|g| (g.evaluator(), g.diagrams()))
            .collect();
        let source = compile_scale_source(
            &subprocesses,
            model,
            evaluated,
            card,
            alpha_s,
            awareness.depends_on_alpha_s,
        )?;
        // Every pooled sampling channel has to name a channel of its own group's
        // forests: that pairing is the whole of how a drawn channel reaches the
        // cluster scale, and it is an index into a set built elsewhere.
        if let Some(sets) = source.channels() {
            assert_eq!(sets.len(), self.groups.groups().len());
            for id in &self.channel_ids {
                assert!(
                    id.diagram < sets[id.group].diagram_count(),
                    "sampling channel (group {}, diagram {}) has no forest in a set of {}",
                    id.group,
                    id.diagram,
                    sets[id.group].diagram_count()
                );
            }
        }
        if source.constant_scales().is_none() {
            self.probe_scale(&source)?;
        }
        let report = constant_scale_report(&mut self.subs, Some(&source), awareness);
        if source.draws_configuration() {
            let widest = self
                .groups
                .groups()
                .iter()
                .map(|g| g.evaluator().n_configs())
                .max()
                .unwrap_or(0);
            self.amp2_len = widest;
            self.amp2_alpha_s = report.alpha_s_ref;
        }
        self.scales = source;
        self.reset_scratch();
        Ok(report)
    }

    /// Resolve the scale on the first cut-passing point of a fixed pseudo-random
    /// draw, so a refusal that would otherwise surface mid-integration surfaces at
    /// setup.
    /// A point below the factorisation floor is not a refusal — it is one ordinary
    /// zero-weight point — so the probe steps over it and keeps drawing. Only the
    /// degenerate case is reported: cut-passing points exist and *every* one of
    /// them was vetoed, which makes the cross section zero by construction and is
    /// worth saying before the integration spends anything on it. A run whose
    /// support is partly below the floor integrates normally, as it does in
    /// MadGraph.
    fn probe_scale(&self, source: &EventScaleSource) -> Result<(), ProtonError> {
        use rand::Rng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(SCALE_PROBE_SEED);
        let ndim = self.channel_grid_ndim();
        let mut any_passed_cuts = false;
        for _ in 0..SCALE_PROBE_DRAWS {
            let u: Vec<f64> = (0..ndim).map(|_| rng.random::<f64>()).collect();
            let m = self.map_point(&u);
            let pt = self
                .combiner
                .sample_channel_at(0, m.sqrt_shat, &u[OUTER_NDIM..]);
            let sc = self.scratch();
            self.build_frames(sc, &m, &pt.momenta);
            let lab = sc.lab_buf.borrow();
            if self.cuts.pass(&lab) {
                any_passed_cuts = true;
                let resolved = self
                    .scales_of(sc, source, &lab, self.sampled_channel(0))
                    .map_err(HadronicError::from)?;
                if let PointScales::Scales(_) = resolved {
                    return Ok(());
                }
            }
        }
        if any_passed_cuts {
            return Err(HadronicError::FactorisationScaleBelowFloor.into());
        }
        // No draw passed the cuts, which says something about the cuts and
        // nothing about the scale.
        Ok(())
    }

    /// Lower support `ŝ_min/s` of the logarithmic τ map.
    pub fn tau_min(&self) -> f64 {
        self.tau_min
    }

    /// The spacelike-pole floor (GeV²) the peripheral channels were built with, from
    /// the process's own cuts.
    pub fn spacelike_floor(&self) -> f64 {
        self.cuts.spacelike_floor()
    }

    /// What the rule-based composition chose for each sampling channel, in channel
    /// order — the map kind and the propagator poles that shaped it.
    pub fn channel_samplers(&self) -> &[ChannelSampler] {
        &self.channel_samplers
    }

    /// Which diagram of which group each sampling channel came from, in channel
    /// order — the key a per-channel grid is banked under.
    pub fn channel_ids(&self) -> &[ChannelId] {
        &self.channel_ids
    }

    /// The partonic system the outer coordinates `u[0], u[1]` map to — the `(τ, y)`
    /// map every channel term shares, with its Jacobian.
    ///
    /// Exposed so a pointwise oracle can drive the production map at pinned
    /// coordinates instead of reimplementing it; the integration reads the same
    /// function.
    pub fn outer_point(&self, u: &[f64]) -> OuterPoint {
        self.map_point(u)
    }

    /// The channels the integral is split across: one per diagram of every group,
    /// pooled into a single mixture.
    pub fn channel_count(&self) -> usize {
        self.combiner.channels().len()
    }

    /// The current channel selection weights, in channel order.
    pub fn channel_alphas(&self) -> Vec<f64> {
        self.combiner.alphas().to_vec()
    }

    /// Install selection weights taken from a completed adaptation instead of
    /// re-surveyed — the exactness a replayed run needs, since `αⱼ` enters every
    /// channel's weight.
    ///
    /// # Panics
    ///
    /// If `alphas` is not one normalised set of positive weights per channel.
    pub fn set_channel_alphas(&mut self, alphas: Vec<f64>) {
        self.combiner.set_alphas(alphas);
    }

    /// The coordinates one channel's grid is built over: the two outer `(τ, y)`
    /// coordinates followed by the channel's own `3n − 4`.
    pub fn channel_grid_ndim(&self) -> usize {
        OUTER_NDIM + self.combiner.channel_ndim()
    }

    /// The uniforms a point carries beyond its channel's grid coordinates: one for
    /// the scale prescription's configuration draw where that draw is live, and
    /// none otherwise.
    ///
    /// It is not a grid coordinate — the grids stay over
    /// [`channel_grid_ndim`](Self::channel_grid_ndim) exactly.
    pub fn scale_draw_ndim(&self) -> usize {
        usize::from(self.scales.draws_configuration())
    }

    /// The full length of the coordinate slice a point is evaluated at: its
    /// channel's grid coordinates followed by
    /// [`scale_draw_ndim`](Self::scale_draw_ndim) trailing uniforms.
    pub fn point_ndim(&self) -> usize {
        self.channel_grid_ndim() + self.scale_draw_ndim()
    }

    /// Split a point's coordinates into the ones its maps consume and the trailing
    /// ones the scale prescription does.
    ///
    /// # Panics
    ///
    /// If `u` is not [`point_ndim`](Self::point_ndim) long. A driver that handed
    /// over only the grid's coordinates would silently drop the scale draw and
    /// evaluate every point in the sampler's own channel, which is a difference no
    /// cross section announces.
    fn split_point<'u>(&self, u: &'u [f64]) -> (&'u [f64], &'u [f64]) {
        let grid_ndim = self.channel_grid_ndim();
        assert_eq!(
            u.len(),
            grid_ndim + self.scale_draw_ndim(),
            "a point is {} map coordinates and {} scale-draw uniforms",
            grid_ndim,
            self.scale_draw_ndim()
        );
        u.split_at(grid_ndim)
    }

    /// The strong coupling's source, once a run card installed one.
    pub fn alpha_s_source(&self) -> Option<&AlphaSSource> {
        self.scales.alpha_s()
    }

    /// The compiled per-event scale prescription this integrand runs.
    pub fn scale_source(&self) -> &EventScaleSource {
        &self.scales
    }

    /// Map a VEGAS point's outer coordinates to the partonic system.
    fn map_point(&self, u: &[f64]) -> OuterPoint {
        let tau = self.tau_min.powf(1.0 - u[0]);
        let sqrt_tau = tau.sqrt();
        let y_max = -0.5 * tau.ln();
        let y = (2.0 * u[1] - 1.0) * y_max;
        OuterPoint {
            x1: sqrt_tau * y.exp(),
            x2: sqrt_tau * (-y).exp(),
            sqrt_shat: (tau * self.s_had).sqrt(),
            jac: self.ln_inv_tau_min * 2.0 * y_max,
        }
    }

    /// Fill the partonic-CM and lab-frame external momenta of one point: beams first,
    /// then the outgoing legs.
    fn build_frames(&self, sc: &ProtonScratch<'a>, m: &OuterPoint, out: &[V]) {
        let e_cm = m.sqrt_shat / 2.0;
        let mut cm = sc.cm_buf.borrow_mut();
        cm.clear();
        cm.push(V::new(e_cm, 0.0, 0.0, e_cm));
        cm.push(V::new(e_cm, 0.0, 0.0, -e_cm));
        cm.extend_from_slice(out);

        let e_beam = self.sqrt_s_had / 2.0;
        let beta = (m.x1 - m.x2) / (m.x1 + m.x2);
        let mut lab = sc.lab_buf.borrow_mut();
        lab.clear();
        lab.push(V::new(m.x1 * e_beam, 0.0, 0.0, m.x1 * e_beam));
        lab.push(V::new(m.x2 * e_beam, 0.0, 0.0, -m.x2 * e_beam));
        lab.extend(out.iter().map(|p| boost_z(*p, beta)));
    }

    /// The integrand at one point with the phase-space map's own weight left out:
    /// the `(τ, y)` Jacobian, the flux, the `2π` measure and the luminosity-weighted
    /// sum over groups. Zero where the cuts reject the lab-frame configuration or no
    /// group carries luminosity.
    fn shape(
        &self,
        sc: &ProtonScratch<'a>,
        m: &OuterPoint,
        out: &[V],
        channel: SampledChannel,
        scale_u: &[f64],
    ) -> (f64, SampledChannel) {
        self.build_frames(sc, m, out);
        let cm = sc.cm_buf.borrow();
        {
            let lab = sc.lab_buf.borrow();
            if !self.cuts.pass(&lab) {
                return (0.0, channel);
            }
        }
        // The configuration the scale is clustered in, drawn from this point's own
        // squared amplitudes where the card's enhancement weight is that and
        // nothing else. The group is the sampler's: a configuration draw names one
        // configuration *inside* a group's forests and says nothing about which
        // group's forests to use.
        let channel = self.scale_channel(sc, &cm, channel, scale_u);
        // A point whose factorisation scale fell below the floor carries no
        // weight, and returning here is before both of the things that follow:
        // the coupling is not moved for a point that contributes nothing, and the
        // parton densities are not queried below roughly their own grid's lowest
        // tabulated `Q`, which is most of why the floor sits where it does.
        let Some(scales) = self.event_scales_in(sc, channel) else {
            return (0.0, channel);
        };
        self.apply_scale(sc, scales.mu_r);

        // Two density readings for the whole point: every group reads `x·f` at
        // the same two `(x, μ²_F)`, and differs only in which flavours it picks
        // out of them.
        let [f1, f2] = beam_rows(self.pdf, m.x1, m.x2, scales.mu_f);

        let mut acc = 0.0;
        let mut mirror = sc.mirror_buf.borrow_mut();
        for (g, sub) in self.groups.groups().iter().zip(&sc.subs) {
            let [direct, reflected] = g.symmetry_weighted_luminosity_rows(&f1, &f2);
            let mut term = 0.0;
            if direct != 0.0 {
                term += direct * sub.eval_m2(&cm);
            }
            // Zero for a group whose beams carry one parton ([`FlavorGroup::has_mirror`]),
            // so such a group costs one matrix element per point rather than two.
            if reflected != 0.0 {
                g.mirror_into(&cm, &mut mirror);
                term += reflected * sub.eval_m2(&mirror);
            }
            acc += g.spin_color_average() * term;
        }
        if acc == 0.0 {
            return (0.0, channel);
        }
        let flux = 1.0 / (2.0 * m.sqrt_shat * m.sqrt_shat);
        (m.jac * flux * self.lips_2pi * acc, channel)
    }

    /// Which of the sampled group's channels names the integration configuration
    /// this point's scale is clustered in.
    ///
    /// Without the draw it is the sampling channel the point came from. With it,
    /// the configuration is drawn `∝ AMP2_c(p)` from the group's own squared
    /// amplitudes and named back through *its own diagram*, the common ground
    /// between the evaluator's configuration order and the channel forests'.
    /// `AMP2` is formed at the coupling the amplitudes were bound at, so the drawn
    /// configuration is a function of the momenta and not of evaluation history.
    ///
    /// The momenta are the direct ordering's. A group's mirrored term is evaluated
    /// at the same scale as its direct one, so there is one draw per point and not
    /// one per ordering.
    fn scale_channel(
        &self,
        sc: &ProtonScratch<'a>,
        cm: &[V],
        channel: SampledChannel,
        scale_u: &[f64],
    ) -> SampledChannel {
        let [v] = scale_u else { return channel };
        let sub = &sc.subs[channel.group];
        if let Some(alpha_s) = self.amp2_alpha_s {
            sub.set_alpha_s(alpha_s);
        }
        let eval = self.groups.groups()[channel.group].evaluator();
        let mut buf = sc.amp2_buf.borrow_mut();
        let amp2 = &mut buf[..eval.n_configs()];
        sub.eval_amp2(cm, amp2);
        match select_index(amp2, *v) {
            Some(c) => SampledChannel {
                group: channel.group,
                diagram: eval.config_diagrams()[c],
            },
            // Every diagram amplitude vanished here, so the coherent sum does too
            // and this point carries no weight whichever channel names its scale.
            None => {
                self.scale_draw_fallbacks.fetch_add(1, Ordering::Relaxed);
                channel
            }
        }
    }

    /// Points on which the configuration draw found no probability and kept the
    /// sampling channel. Expected to be zero on a run that produces anything.
    pub fn scale_draw_fallbacks(&self) -> u64 {
        self.scale_draw_fallbacks.load(Ordering::Relaxed)
    }

    /// The scales at the lab-frame point currently in the frame buffers, in the
    /// sampling channel that drew it, or `None` where the point fell below the
    /// factorisation floor and so carries no weight.
    ///
    /// Every *other* scale error still stops the run, with the message it had
    /// before: those say the prescription does not apply to this process, which
    /// no amount of sampling fixes.
    fn event_scales_in(
        &self,
        sc: &ProtonScratch<'a>,
        channel: SampledChannel,
    ) -> Option<EventScales> {
        if let Some(fixed) = self.scales.constant_scales() {
            return Some(fixed);
        }
        let lab = sc.lab_buf.borrow();
        match self
            .scales_of(sc, &self.scales, &lab, channel)
            .unwrap_or_else(|e| panic!("per-event scale on a sampled point: {e}"))
        {
            PointScales::Scales(scales) => Some(scales),
            PointScales::Vetoed => None,
        }
    }

    /// The pooled sampling channel `j` as the scale prescription names it: the
    /// flavour group it was built for, and its diagram inside that group.
    fn sampled_channel(&self, j: usize) -> SampledChannel {
        let id = self.channel_ids[j];
        SampledChannel {
            group: id.group,
            diagram: id.diagram,
        }
    }

    fn scales_of(
        &self,
        sc: &ProtonScratch<'a>,
        source: &EventScaleSource,
        lab: &[V],
        channel: SampledChannel,
    ) -> Result<PointScales, ScaleError> {
        let mut buf = sc.scale_buf.borrow_mut();
        buf.clear();
        buf.extend(lab[2..].iter().map(components));
        source.point_scales([components(&lab[0]), components(&lab[1])], &buf, channel)
    }

    /// Move every group's amplitude to the coupling `mu_r` implies. A constant
    /// prescription was applied once at installation and a matrix element with no
    /// strong coupling has none to move, so both return without touching the pools.
    fn apply_scale(&self, sc: &ProtonScratch<'a>, mu_r: f64) {
        if self.scales.constant_scales().is_some() {
            return;
        }
        let Some(source) = self.scales.alpha_s() else {
            return;
        };
        let (last_mu_r, last_alpha_s) = sc.last_coupling.get();
        let alpha_s = if mu_r == last_mu_r {
            last_alpha_s
        } else {
            let alpha_s = source.eval(mu_r);
            sc.last_coupling.set((mu_r, alpha_s));
            alpha_s
        };
        for sub in &sc.subs {
            sub.set_alpha_s(alpha_s);
        }
    }

    /// The `channel`-th term of the channel-split estimator at
    /// `u ∈ [0,1]^channel_grid_ndim`, in natural units (GeV⁻²): the outer map takes
    /// `u[0..2]`, the channel's own map the rest, and the point is weighted by
    /// `αⱼ/g` so the terms' integrals sum to the cross section.
    ///
    /// # Panics
    ///
    /// If `channel` is not a channel index.
    pub fn value_in_channel(&self, channel: usize, u: &[f64]) -> f64 {
        let (grid_u, scale_u) = self.split_point(u);
        let m = self.map_point(grid_u);
        let point = self
            .combiner
            .sample_channel_at(channel, m.sqrt_shat, &grid_u[OUTER_NDIM..]);
        let (shape, _) = self.shape(
            self.scratch(),
            &m,
            &point.momenta,
            self.sampled_channel(channel),
            scale_u,
        );
        if shape == 0.0 {
            return 0.0;
        }
        shape * point.weight
    }

    /// [`value_in_channel`](Self::value_in_channel) with the point kept: the two
    /// frames, the beam momentum fractions and the scales the matrix element ran
    /// at, for a point that carries weight.
    ///
    /// An accept/reject pass needs all of that only for the points it keeps, so the
    /// trial loop runs on `value_in_channel` and reconstructs an accepted point
    /// through this — the same map at the same `u`, hence the same weight. `None`
    /// where the cuts reject the point or no group carries luminosity, which is
    /// where the trial would have carried no weight either.
    pub fn event_in_channel(&self, channel: usize, u: &[f64]) -> Option<ProtonEvent> {
        let (grid_u, scale_u) = self.split_point(u);
        let m = self.map_point(grid_u);
        let point = self
            .combiner
            .sample_channel_at(channel, m.sqrt_shat, &grid_u[OUTER_NDIM..]);
        let sc = self.scratch();
        let (shape, drawn) = self.shape(
            sc,
            &m,
            &point.momenta,
            self.sampled_channel(channel),
            scale_u,
        );
        if shape == 0.0 {
            return None;
        }
        Some(ProtonEvent {
            weight: shape * point.weight,
            x: [m.x1, m.x2],
            // `shape` returned nonzero, so this point was not vetoed.
            scales: self
                .event_scales_in(sc, drawn)
                .expect("a point carrying weight has scales"),
            lab: sc.lab_buf.borrow().clone(),
            cm: sc.cm_buf.borrow().clone(),
        })
    }

    /// Fill in an accepted event's discrete labels: which flavour group produced
    /// it, which concrete flavour assignment inside that group, which way round the
    /// beams carry it, which helicity combination and which colour flow.
    ///
    /// The five uniforms are consumed in that order. The draws are nested rather
    /// than independent, and each is proportional to the term of the point's own
    /// value that the label names:
    ///
    /// * the group `∝ avg_g · (L_g^direct |M_g(q)|² + L_g^mirror |M_g(Rq)|²)`, with
    ///   the two luminosities symmetry-weighted as in the cross section;
    /// * the `(flavour, beam ordering)` pair inside it `∝ S_i · L_i^o · |M(q or Rq)|²`,
    ///   which at fixed ordering is the member's share of the group's summed
    ///   parton luminosity at this event's `(x₁, x₂)`, times its own
    ///   identical-particle factor — the whole of what distinguishes one member of a
    ///   group from another, since they share the matrix element exactly;
    /// * the helicity `∝ |M_c|²`, then the colour flow through
    ///   [`AmplitudeEvaluator::select_color_flow`] — the integration configuration
    ///   `∝ AMP2(d)` and the flow `∝ JAMP2(i)` inside that configuration's
    ///   admitted set — all evaluated at the argument the drawn ordering implies,
    ///   as on a fixed-beam run.
    ///
    /// All of them are selections, not sampling channels: the cross section sums
    /// over every one, and this reads accumulators that sum already contains. A
    /// caller may skip it entirely and integrate the same number.
    ///
    /// `None` when the point carries no weight, where no label is defined.
    pub fn select_event(&self, event: &ProtonEvent, u: [f64; 5]) -> Option<ProtonSelection> {
        let sc = self.scratch();
        // The diagonals are read at the event's own coupling, the one its |M|² was
        // taken at.
        self.apply_scale(sc, event.scales.mu_r);
        let mut mirror = Vec::with_capacity(event.cm.len());

        let [f1, f2] = beam_rows(self.pdf, event.x[0], event.x[1], event.scales.mu_f);

        let mut m2 = Vec::with_capacity(sc.subs.len());
        let mut terms = Vec::with_capacity(sc.subs.len());
        for (g, sub) in self.groups.groups().iter().zip(&sc.subs) {
            let lumi = g.symmetry_weighted_luminosity_rows(&f1, &f2);
            let direct = if lumi[0] != 0.0 {
                sub.eval_m2(&event.cm)
            } else {
                0.0
            };
            let reflected = if lumi[1] != 0.0 {
                g.mirror_into(&event.cm, &mut mirror);
                sub.eval_m2(&mirror)
            } else {
                0.0
            };
            m2.push([direct, reflected]);
            terms.push(g.spin_color_average() * (lumi[0] * direct + lumi[1] * reflected));
        }
        let group = select_index(&terms, u[0])?;
        let g = &self.groups.groups()[group];

        // One categorical draw over the group's `(member, ordering)` terms: the
        // matrix element is common to the members, so within an ordering this is the
        // luminosity share times the member's own identical-particle factor.
        let weights: Vec<f64> = g
            .members()
            .iter()
            .enumerate()
            .flat_map(|(i, member)| {
                let s = member.symmetry_factor();
                let lumi = g.member_luminosity_rows(i, &f1, &f2);
                [s * lumi[0] * m2[group][0], s * lumi[1] * m2[group][1]]
            })
            .collect();
        let picked = select_index(&weights, u[1])?;
        let (member, ordering) = (
            picked / 2,
            if picked % 2 == 0 {
                BeamOrdering::Direct
            } else {
                BeamOrdering::Exchanged
            },
        );

        let argument = match ordering {
            BeamOrdering::Direct => &event.cm,
            BeamOrdering::Exchanged => {
                g.mirror_into(&event.cm, &mut mirror);
                &mirror
            }
        };
        let sub = &sc.subs[group];
        let eval = sub.evaluator();
        let mut hel_m2 = vec![0.0; eval.helicities().len()];
        let mut amp2 = vec![0.0; eval.n_configs()];
        let mut jamp2 = vec![0.0; eval.n_flows()];
        sub.eval_diagonals(argument, &mut hel_m2, &mut amp2, &mut jamp2);
        let drawn = eval.select_helicity(&hel_m2, u[2])?;
        let (_, order) = g.event_legs(member, ordering);

        Some(ProtonSelection {
            group,
            member,
            ordering,
            helicity: order.iter().map(|&leg| drawn[leg]).collect(),
            flow: eval.select_color_flow(&amp2, &jamp2, [u[3], u[4]])?,
        })
    }

    /// The uniforms the whole map consumes as one mixture: the two outer coordinates,
    /// one channel-selection coordinate, and the channel's own `3n − 4`.
    pub fn vegas_ndim(&self) -> usize {
        self.channel_grid_ndim() + 1 + self.scale_draw_ndim()
    }

    /// The integrand at `u ∈ [0,1]^vegas_ndim` drawn through the mixture rather than
    /// one frozen channel, in natural units (GeV⁻²): `u[2]` selects the channel and
    /// the point is weighted by `1/g`.
    ///
    /// Its integral is the same cross section the channel-split terms sum to; this is
    /// the undivided form, and the estimator whose variance the selection weights are
    /// adapted to minimise.
    pub fn value(&self, u: &[f64]) -> f64 {
        let m = self.map_point(u);
        let j = self.combiner.select(u[OUTER_NDIM]);
        let point = self
            .combiner
            .sample_channel_at(j, m.sqrt_shat, &u[OUTER_NDIM + 1..]);
        let (shape, _) = self.shape(
            self.scratch(),
            &m,
            &point.momenta,
            self.sampled_channel(j),
            &u[self.channel_grid_ndim() + 1..],
        );
        if shape == 0.0 {
            return 0.0;
        }
        // `sample_channel_at` weights by `αⱼ/g`, and the mixture that drew this point
        // has density `g`.
        shape * point.weight / self.combiner.alphas()[j]
    }

    /// Refine the channel selection weights toward the variance-minimising mixture,
    /// jointly over the `(group, diagram)` channel space.
    ///
    /// This is [`MultiChannel::adapt_alphas`](crate::phasespace::MultiChannel::adapt_alphas)'
    /// survey→refine loop driven from outside the combiner, because the integrand —
    /// not the combiner — owns the `(τ, y)` coordinates and so owns the energy each
    /// draw is made at. Each survey draws `n_survey` points from the *current*
    /// mixture over the full hypercube (outer coordinates, a channel-selection
    /// coordinate, then the channel's own), estimates every channel's variance share
    /// `Wⱼ = E_g[(f/g)²·gⱼ/g]`, and reallocates by [`kleiss_pittau_step`].
    ///
    /// The surveyed `f` is the whole integrand shape — the `(τ, y)` Jacobian, the
    /// flux, the cut and the luminosity-weighted sum over groups — so weight flows to
    /// the channels that carry variance *of the hadronic integral*, not of a partonic
    /// one at some representative energy.
    pub fn adapt_alphas(
        &mut self,
        seed: u64,
        n_survey: usize,
        n_iter: usize,
        damping: f64,
    ) -> AlphaAdaptation<f64> {
        let mut trajectory = vec![self.channel_alphas()];
        let mut variance_shares = vec![0.0; self.channel_count()];
        for it in 0..n_iter {
            let w = self.survey_variance(seed, ADAPT_STREAM + it as u64, n_survey);
            variance_shares = w.clone();
            let Some(raw) = kleiss_pittau_step(self.combiner.alphas(), &w, damping) else {
                break;
            };
            self.combiner.set_alphas(raw.clone());
            trajectory.push(raw);
        }
        AlphaAdaptation {
            trajectory,
            variance_shares,
        }
    }

    /// One survey pass: every channel's variance share `Wⱼ = E_g[(f/g)²·gⱼ/g]` under
    /// the current mixture. Each drawn point informs every channel, so the estimate
    /// is low-variance in the channels it does not draw from.
    ///
    /// The point loop runs in one rayon region split into fixed-size chunks. Both
    /// substreams are addressed by the point's index within the survey, so a chunk
    /// seeks straight to its own first point and draws exactly the points it would
    /// have drawn in sequence; what the split changes is the summation, which is
    /// reduced from per-chunk partials in chunk order. Carrying every point's whole
    /// `n_channels` density row out to a single sequential reduction — the stronger
    /// contract [`crate::vegas::VegasGrid::adapt_parallel_seeded`] holds — would cost
    /// `n_survey × n_channels` doubles, which on a several-hundred-channel process is
    /// hundreds of megabytes, so the partials are summed per chunk instead. That
    /// makes [`SURVEY_CHUNK`] part of the answer and the thread count not.
    fn survey_variance(&self, seed: u64, stream: u64, n_survey: usize) -> Vec<f64> {
        let n = self.channel_count();
        let ndim = self.channel_grid_ndim() + 1;
        let scale_ndim = self.scale_draw_ndim();
        let nchunks = n_survey.div_ceil(SURVEY_CHUNK);
        let partials: Vec<Vec<f64>> = (0..nchunks)
            .into_par_iter()
            .map(|chunk| {
                let first = chunk * SURVEY_CHUNK;
                let points = (n_survey - first).min(SURVEY_CHUNK);
                let mut s = SubStream::new(seed, stream, (first * ndim) as u64);
                // The scale draw's uniforms come off a stream of their own, so the
                // survey's own point sequence is what it would be with no draw
                // installed.
                let mut scale_draw = SubStream::new(
                    seed,
                    SCALE_DRAW_STREAM_BASE + stream,
                    (first * scale_ndim) as u64,
                );
                let sc = self.scratch();
                let mut w = vec![0.0; n];
                let mut scale_u = vec![0.0; scale_ndim];
                for _ in 0..points {
                    let u = s.uniforms::<f64>(ndim);
                    let m = self.map_point(&u);
                    let j = self.combiner.select(u[OUTER_NDIM]);
                    let point =
                        self.combiner
                            .sample_channel_at(j, m.sqrt_shat, &u[OUTER_NDIM + 1..]);
                    // `sample_channel_at` weights by `αⱼ/g`; the mixture that actually
                    // drew this point has density `g`, so the mixture estimator is `f/g`.
                    let g = self.combiner.alphas()[j] / point.weight;
                    scale_draw.fill_uniforms(&mut scale_u);
                    let est = self
                        .shape(sc, &m, &point.momenta, self.sampled_channel(j), &scale_u)
                        .0
                        / g;
                    if est == 0.0 {
                        continue;
                    }
                    let est2 = est * est;
                    for (wj, ch) in w.iter_mut().zip(self.combiner.channels()) {
                        *wj += est2 * ch.density_at(m.sqrt_shat, &point.momenta) / g;
                    }
                }
                w
            })
            .collect();
        let mut w = vec![0.0; n];
        for partial in &partials {
            for (wj, pj) in w.iter_mut().zip(partial) {
                *wj += pj;
            }
        }
        let inv = 1.0 / n_survey as f64;
        for wj in &mut w {
            *wj *= inv;
        }
        w
    }

    /// Run one VEGAS adaptation per channel, returning each channel's trained grid and
    /// term alongside their sum — the primitive a banked integration serialises.
    ///
    /// Channel `j` is integrated over its own `channel_grid_ndim` coordinates with the
    /// channel frozen, on a budget of `αⱼ · neval` per iteration and its own RNG
    /// substream, exactly as the fixed-beam path does.
    pub fn adapt_grids(
        &self,
        neval: usize,
        niter: usize,
        seed: u64,
    ) -> (Vec<ChannelIntegration>, VegasResult) {
        let (per_channel, total, _) = integrate_channels(
            self,
            &self.channel_alphas(),
            self.vegas_alpha,
            IterationCombination::default(),
            Budget::Fixed { neval, niter },
            BlockAllocation::ByAlpha,
            seed,
        );
        (per_channel, total)
    }

    /// Integrate under an arbitrary [`Budget`] and channel-allocation rule,
    /// reporting what was spent alongside the terms.
    ///
    /// [`adapt_grids`](Self::adapt_grids) is the [`Budget::Fixed`],
    /// [`BlockAllocation::ByAlpha`] case of this.
    pub fn adapt_grids_budget(
        &self,
        budget: Budget,
        allocation: BlockAllocation,
        seed: u64,
    ) -> (Vec<ChannelIntegration>, VegasResult, ConvergenceReport) {
        integrate_channels(
            self,
            &self.channel_alphas(),
            self.vegas_alpha,
            IterationCombination::default(),
            budget,
            allocation,
            seed,
        )
    }

    /// Integrate the cross section with VEGAS, returning `(σ, Δσ)` in picobarns.
    pub fn integrate(&self, neval: usize, niter: usize, seed: u64) -> (f64, f64) {
        let result = self.adapt_grids(neval, niter, seed).1;
        (result.integral * GEV2_TO_PB, result.std_dev * GEV2_TO_PB)
    }
}

/// The seam a frozen-grid scan and an accept/reject pass drive this integrand
/// through — the same one the fixed-beam path exposes, so neither is rebuilt for
/// hadronic beams.
impl ChannelIntegrand for ProtonIntegrand<'_> {
    fn channel_count(&self) -> usize {
        ProtonIntegrand::channel_count(self)
    }

    fn channel_grid_ndim(&self) -> usize {
        ProtonIntegrand::channel_grid_ndim(self)
    }

    fn scale_draw_ndim(&self) -> usize {
        ProtonIntegrand::scale_draw_ndim(self)
    }

    fn value_in_channel(&self, channel: usize, u: &[f64]) -> f64 {
        ProtonIntegrand::value_in_channel(self, channel, u)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::SamplerTopology;
    use crate::hadronic::{
        channel_neval, CHANNEL_STREAM_BASE, VEGAS_NBINS,
    };
    use crate::vegas::VegasGrid;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::lhef::build::SubprocessRecord;
    use crate::pdf::grid::SubGrid;
    use crate::ufo::sm::{sm_model, SMRestrict};
    use std::collections::BTreeSet;
    use std::sync::Arc;

    /// The multi-subprocess hadronic process the grouping rule is measured on:
    /// two beam multiparticles, two coupling classes, and a jet that is a gluon
    /// in one arrangement and a quark in the other.
    const LLJ: &str = "p p > l+ l- j QCD=2 QED=2";

    /// The hadronic process whose flavour groups join subprocesses across all three
    /// colour-rep relations: equal (`u u > u u` with `c c > c c`), globally conjugate
    /// (`g u > g u` with `g u~ > g u~`) and partially conjugate — the crossing class,
    /// `u c > u c` with `u c~ > u c~`, which no slot operation relates.
    const JJ: &str = "p p > j j";

    fn model() -> Arc<UFOModel> {
        sm_model(SMRestrict::Default)
    }

    fn enumerate(process: &str, model: &UFOModel) -> Vec<DiagramSet> {
        let opts = ParsingOptions::default();
        let card = parse_proc_card(&format!("generate {process}"), &opts).expect("proc card");
        generate_from_proc_card(&card, model).expect("enumeration")
    }

    fn derive(process: &str, model: &UFOModel, evaluated: &EvaluatedModel) -> FlavorGroups {
        derive_flavor_groups(
            enumerate(process, model),
            model,
            evaluated,
            &RunCard::default(),
        )
        .expect("flavour groups")
    }

    /// Probe points at energies and on a stream the partition was *not* derived
    /// from, so a within-group agreement measured here is a prediction.
    ///
    /// These points serve the partition — that a subprocess matches exactly one
    /// group and that no two groups agree — and the mirror *identity*, both of
    /// which are energy-independent claims. The mirror term's *visibility* is
    /// not, so it is measured on a ladder of its own against
    /// `mirror_visibility_floor` rather than on these three energies.
    fn fresh_points(final_masses: &[f64]) -> Vec<Vec<V>> {
        let mut points = Vec::new();
        for (i, sqrt_s) in [220.0f64, 740.0, 2100.0].into_iter().enumerate() {
            let rambo = RamboChannel::<f64>::new(sqrt_s, final_masses.to_vec());
            let mut stream = SubStream::from_stream(0x0FF5_E7ED, i as u64);
            for _ in 0..12 {
                let u = stream.uniforms::<f64>(rambo.ndim());
                let drawn = rambo.sample(&u);
                let e = sqrt_s / 2.0;
                let mut momenta = vec![V::new(e, 0.0, 0.0, e), V::new(e, 0.0, 0.0, -e)];
                momenta.extend(drawn.momenta.iter().cloned());
                points.push(momenta);
            }
        }
        points
    }

    fn m2_trace(
        evaluator: &AmplitudeEvaluator,
        evaluated: &EvaluatedModel,
        points: &[Vec<V>],
    ) -> Vec<f64> {
        let bound = BoundAmplitude::<f64>::bind(evaluator, evaluated);
        let mut scratch = bound.scratch_space();
        points
            .iter()
            .map(|k| bound.eval_m2(k, &mut scratch))
            .collect()
    }

    /// A synthetic parton distribution whose `x·f` differs per flavour and whose
    /// `x` shape differs per flavour too, so a luminosity that swapped the two
    /// beam orderings or dropped one of them produces a different number. A
    /// common `x` shape would make the two orderings equal and the check vacuous.
    fn probe_pdf() -> PdfMember {
        let flavors = vec![-4, -3, -2, -1, 1, 2, 3, 4, 21];
        let x = vec![1e-7, 1.0];
        let q2 = vec![1.0, 1e8];
        let mut xf = Vec::new();
        for ix in 0..x.len() {
            for iq in 0..q2.len() {
                for ifl in 0..flavors.len() {
                    let shape = 1.0 + ix as f64 * 0.1 * (ifl + 1) as f64;
                    xf.push(0.01 * (ifl + 1) as f64 * shape * (1.0 + 0.5 * iq as f64));
                }
            }
        }
        PdfMember::from_subgrids(vec![SubGrid { x, q2, flavors, xf }])
    }

    /// [`probe_pdf`] restricted to `alive`: every other flavour is absent from the
    /// grid and so carries exactly zero, which switches off the flavour groups whose
    /// initial states need it while leaving their sampling channels in the mixture.
    fn probe_pdf_restricted(alive: &[i32]) -> PdfMember {
        let flavors = alive.to_vec();
        let x = vec![1e-7, 1.0];
        let q2 = vec![1.0, 1e8];
        let mut xf = Vec::new();
        for ix in 0..x.len() {
            for iq in 0..q2.len() {
                for ifl in 0..flavors.len() {
                    let shape = 1.0 + ix as f64 * 0.1 * (ifl + 1) as f64;
                    xf.push(0.01 * (ifl + 1) as f64 * shape * (1.0 + 0.5 * iq as f64));
                }
            }
        }
        PdfMember::from_subgrids(vec![SubGrid { x, q2, flavors, xf }])
    }

    /// A probe distribution whose `x` shape differs *sharply* per flavour, so which
    /// beam a momentum fraction belongs to changes the luminosity shares by a wide
    /// margin. [`probe_pdf`] is flat enough in `x` that exchanging `x₁` and `x₂`
    /// barely moves them, which would leave the beam orientation unpinned.
    fn beam_asymmetric_pdf() -> PdfMember {
        let flavors = vec![-4, -3, -2, -1, 1, 2, 3, 4, 21];
        let x = vec![1e-7, 1e-4, 1e-2, 1.0];
        let q2 = vec![1.0, 1e8];
        let mut xf = Vec::new();
        for ix in 0..x.len() {
            for iq in 0..q2.len() {
                for ifl in 0..flavors.len() {
                    let slope = (1.0 + ix as f64).powi(ifl as i32 + 1);
                    xf.push(0.01 * (ifl + 1) as f64 * slope * (1.0 + 0.5 * iq as f64));
                }
            }
        }
        PdfMember::from_subgrids(vec![SubGrid { x, q2, flavors, xf }])
    }

    /// The ladder's own shape, which no grouping outcome pins: the partition would
    /// still come out right if a rung were silently dropped or left below its
    /// final state's threshold.
    #[test]
    fn the_probe_ladder_reaches_below_the_electroweak_scale_and_onto_the_pole() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let m_z = s_channel_resonance(&m, &evaluated).expect("the SM has a massive Z");

        // A massless final state: five distinct rungs over two decades, one of
        // them the pole and one of them below the electroweak scale.
        let massless = probe_energies(&[0.0, 0.0, 0.0], Some(m_z));
        assert_eq!(massless.len(), 5);
        assert!(massless[0] < 91.0, "no rung below the electroweak scale");
        assert!(
            massless.contains(&m_z),
            "the pole is not a rung: {massless:?}"
        );
        assert!(
            massless.last().unwrap() / massless[0] > 50.0,
            "ladder too narrow"
        );

        // A heavy final state: every rung sits above its threshold, and the ones
        // that would not collapse onto the floor instead of repeating a point.
        let m_t = evaluated.mass(m.particle_id("t").expect("t in the SM"));
        let heavy = probe_energies(&[m_t, m_t], Some(m_z));
        assert!(
            heavy.iter().all(|&e| e > 2.0 * m_t),
            "a rung below the t t~ threshold: {heavy:?}"
        );
        assert!(
            heavy.windows(2).all(|w| w[1] > w[0]),
            "rungs repeat: {heavy:?}"
        );
    }

    #[test]
    fn llj_partitions_into_six_groups_of_four_concrete_subprocesses() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let groups = derive(LLJ, &m, &evaluated);

        assert_eq!(groups.subprocess_count(), 24);
        assert_eq!(groups.groups().len(), 6);

        // Each group is one coupling class × one colour arrangement: two quark
        // generations × two lepton flavours, all with the same `|M|²`.
        let mut initial_states: BTreeSet<[i32; 2]> = BTreeSet::new();
        for g in groups.groups() {
            assert_eq!(g.members().len(), 4);
            let pairs: BTreeSet<[i32; 2]> = g.members().iter().map(|s| s.incoming).collect();
            assert_eq!(pairs.len(), 2, "a group should hold two quark generations");
            // Two lepton flavours per initial state; `q q̄` groups share the
            // outgoing gluon, so it is the whole assignment that must be distinct.
            let distinct: BTreeSet<&Subprocess> = g.members().iter().collect();
            assert_eq!(distinct.len(), 4);
            initial_states.extend(pairs);
        }
        // Eight gluon-quark initial states and four annihilations, one ordering each.
        assert_eq!(initial_states.len(), 12);

        let averages: BTreeSet<String> = groups
            .groups()
            .iter()
            .map(|g| format!("{:.6}", g.spin_color_average()))
            .collect();
        let expected: BTreeSet<String> = [1.0 / 96.0, 1.0 / 36.0]
            .iter()
            .map(|v| format!("{v:.6}"))
            .collect();
        assert_eq!(averages, expected);
    }

    /// The partition predicts, rather than merely records: members agree at points
    /// it was not fitted on, and distinct groups disagree by a wide margin there.
    #[test]
    fn group_members_agree_where_the_partition_was_not_measured() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let groups = derive(LLJ, &m, &evaluated);
        let points = fresh_points(&groups.groups()[0].final_masses());

        let representatives: Vec<Vec<f64>> = groups
            .groups()
            .iter()
            .map(|g| m2_trace(g.evaluator(), &evaluated, &points))
            .collect();

        // Every enumerated subprocess is re-derived from the card and matched
        // against the partition, so both halves are checked: that it agrees with
        // the group it was put in, and that the group claims it as a member.
        let mut worst_within = 0.0f64;
        let mut matched = 0usize;
        for set in enumerate(LLJ, &m).iter().filter(|s| !s.diagrams.is_empty()) {
            let evaluator = compile_class(set, &m, &evaluated).expect("subprocess compiles");
            let legs = process_external_legs(&evaluator, &m, &evaluated);
            let flows = evaluator.color_flow_tags().clone();
            let flow_permutation = (0..evaluator.n_flows()).collect();
            let subprocess = Subprocess {
                incoming: [legs[0].pdg, legs[1].pdg],
                outgoing: legs[2..].iter().map(|l| l.pdg).collect(),
                colors: evaluator.external_colors().iter().map(|l| l.rep).collect(),
                flows,
                flow_permutation,
            };
            let trace = m2_trace(&evaluator, &evaluated, &points);
            let hits: Vec<usize> = (0..groups.groups().len())
                .filter(|&i| worst_rel(&representatives[i], &trace) < 1e-12)
                .collect();
            assert_eq!(
                hits.len(),
                1,
                "{} matches {} groups at fresh points",
                label(set),
                hits.len()
            );
            assert!(
                groups.groups()[hits[0]].members().contains(&subprocess),
                "{} agrees with a group that does not list it",
                label(set)
            );
            worst_within = worst_within.max(worst_rel(&representatives[hits[0]], &trace));
            matched += 1;
        }
        assert_eq!(matched, groups.subprocess_count());

        let mut min_cross = f64::INFINITY;
        for (i, a) in representatives.iter().enumerate() {
            for b in &representatives[i + 1..] {
                min_cross = min_cross.min(worst_rel(a, b));
            }
        }
        eprintln!("within-group {worst_within:.3e}, cross-group {min_cross:.3e}");
        assert!(
            min_cross > 0.1,
            "two groups differ by only {min_cross:.3e}; the partition is not a coupling distinction"
        );
    }

    /// The mirror identity `|M_{b a}(q)|² = |M_{a b}(R q)|²`, against explicitly
    /// enumerated mirrored subprocesses.
    ///
    /// This is the term the enumeration does not produce: only one ordering of
    /// each unordered initial state exists, and its parton luminosity is not the
    /// other's. The identity control is the load-bearing half — evaluating the
    /// representative at the *unreflected* point is what dropping the mirror
    /// amounts to, and it has to be wrong by enough to be seen.
    ///
    /// How much is enough is a function of `s-hat`, not a constant, and it is not
    /// a statement about the *weakest* point: the visibility vanishes wherever
    /// the two orderings happen to agree, so a minimum over random draws is a
    /// property of the sample size — it falls by a decade from 36 draws to 512 at
    /// every energy. The control is therefore stated on the tenth percentile,
    /// against `mirror_visibility_floor`, over a ladder that reaches well below
    /// the electroweak scale.
    ///
    /// What this cannot see: `|M|²` is invariant under a further reflection in the
    /// `xz` plane, so the sign of `p_y` in `R` is unpinned — asserted below as a
    /// measured fact rather than left as an assumption. What `R` must do is
    /// reverse `p_z`, and that is what the identity control pins.
    #[test]
    fn the_mirrored_beam_ordering_needs_the_reflected_matrix_element() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let groups = derive(LLJ, &m, &evaluated);
        let points = fresh_points(&groups.groups()[0].final_masses());

        let (mut worst_mirror, mut worst_py) = (0.0f64, 0.0f64);
        let pairs = mirror_pairs(&m, &evaluated, &groups);
        for (g, mirror_eval) in &pairs {
            let bound = BoundAmplitude::<f64>::bind(g.evaluator(), &evaluated);
            let mut scratch = bound.scratch_space();
            let mirror_bound = BoundAmplitude::<f64>::bind(mirror_eval, &evaluated);
            let mut mirror_scratch = mirror_bound.scratch_space();

            let mut reflected = Vec::new();
            for k in &points {
                let target = mirror_bound.eval_m2(k, &mut mirror_scratch);
                let rel = |x: f64| (x - target).abs() / target.abs().max(f64::MIN_POSITIVE);

                g.mirror_into(k, &mut reflected);
                worst_mirror = worst_mirror.max(rel(bound.eval_m2(&reflected, &mut scratch)));

                let py_flipped: Vec<V> = k
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        if i < g.n_in() {
                            *p
                        } else {
                            V::new(p.e(), p.px(), -p.py(), p.pz())
                        }
                    })
                    .collect();
                let direct = bound.eval_m2(k, &mut scratch);
                let flipped = bound.eval_m2(&py_flipped, &mut scratch);
                worst_py =
                    worst_py.max((direct - flipped).abs() / direct.abs().max(f64::MIN_POSITIVE));
            }
        }
        eprintln!(
            "mirror identity worst {worst_mirror:.3e}; an xz reflection alone moves |M|² by \
             {worst_py:.3e}"
        );
        // The bound is set by the probe points, not by the identity. One of the 36
        // RAMBO draws is 8e-10 off the light cone and 2e-12 off momentum
        // conservation, where the two independently compiled programs route the
        // gauge-dependent parts differently and land 5.4e-13 apart; every other
        // point agrees to 4.4e-15. Nothing downstream sees this: the integrand
        // evaluates *one* program at both arguments.
        assert!(
            worst_mirror < 1e-11,
            "the reflected representative disagrees with the mirrored subprocess by \
             {worst_mirror:.3e}"
        );
        assert!(
            worst_py < 1e-12,
            "|M|² moved by {worst_py:.3e} under a reflection in the xz plane, so the sign of p_y \
             in the mirror map is pinned after all and this test's blind spot is misstated"
        );

        // What a dropped mirror would cost, as a function of s-hat rather than as
        // one number: at each rung, nine draws in ten move by more than the floor.
        let masses = groups.groups()[0].final_masses();
        for sqrt_s in [25.0f64, 65.0, 150.0, 400.0, 1200.0] {
            let rels = mirror_visibility(&evaluated, &pairs, &masses, sqrt_s, 32, 0x0FF5_E7ED);
            let p10 = rels[((rels.len() as f64 - 1.0) * 0.10) as usize];
            let floor = mirror_visibility_floor(sqrt_s);
            eprintln!(
                "  sqrt(s-hat) {sqrt_s:7.1}: tenth-percentile visibility {p10:.3e}, \
                 floor {floor:.3e} ({:.2}x)",
                p10 / floor
            );
            assert!(
                p10 > floor,
                "at sqrt(s-hat) = {sqrt_s} nine draws in ten move |M|² by only {p10:.3e}, under \
                 the measured floor {floor:.3e}; a dropped mirror would not be visible there"
            );
        }
    }

    /// Every group paired with the compiled matrix element of its mirrored beam
    /// ordering, asserted to be the same process.
    fn mirror_pairs<'a>(
        m: &UFOModel,
        evaluated: &EvaluatedModel,
        groups: &'a FlavorGroups,
    ) -> Vec<(&'a FlavorGroup, AmplitudeEvaluator)> {
        groups
            .groups()
            .iter()
            .map(|g| {
                let set = g.diagram_set();
                let swapped = format!(
                    "{} {} > {} QCD=2 QED=2",
                    set.particles_in[1],
                    set.particles_in[0],
                    set.particles_out.join(" ")
                );
                let mirror_set = enumerate(&swapped, m)
                    .into_iter()
                    .find(|s| !s.diagrams.is_empty())
                    .expect("the mirrored ordering enumerates");
                assert_eq!(
                    mirror_set.diagrams.len(),
                    set.diagrams.len(),
                    "{swapped} is not the same process as {}",
                    label(set)
                );
                let mirror_eval =
                    compile_class(&mirror_set, m, evaluated).expect("mirror compiles");
                (g, mirror_eval)
            })
            .collect()
    }

    /// How far `|M|²` moves when the mirrored ordering's matrix element is
    /// replaced by the representative's at the *unreflected* point — what
    /// dropping the mirror term amounts to — over `npts` RAMBO draws at
    /// `sqrt_s`, for every group. Returned sorted.
    fn mirror_visibility(
        evaluated: &EvaluatedModel,
        pairs: &[(&FlavorGroup, AmplitudeEvaluator)],
        masses: &[f64],
        sqrt_s: f64,
        npts: usize,
        stream_seed: u64,
    ) -> Vec<f64> {
        let rambo = RamboChannel::<f64>::new(sqrt_s, masses.to_vec());
        let mut rels = Vec::with_capacity(npts * pairs.len());
        for seed in 0..npts as u64 {
            let mut stream = SubStream::from_stream(stream_seed, seed);
            let u = stream.uniforms::<f64>(rambo.ndim());
            let drawn = rambo.sample(&u);
            let e = sqrt_s / 2.0;
            let mut k = vec![V::new(e, 0.0, 0.0, e), V::new(e, 0.0, 0.0, -e)];
            k.extend(drawn.momenta.iter().cloned());
            for (g, mirror_eval) in pairs {
                let bound = BoundAmplitude::<f64>::bind(g.evaluator(), evaluated);
                let mut scratch = bound.scratch_space();
                let mirror_bound = BoundAmplitude::<f64>::bind(mirror_eval, evaluated);
                let mut mirror_scratch = mirror_bound.scratch_space();
                let target = mirror_bound.eval_m2(&k, &mut mirror_scratch);
                let direct = bound.eval_m2(&k, &mut scratch);
                rels.push((direct - target).abs() / target.abs().max(f64::MIN_POSITIVE));
            }
        }
        rels.sort_by(|a, b| a.partial_cmp(b).unwrap());
        rels
    }

    /// A lower bound on the tenth percentile of `mirror_visibility` at `sqrt_s`.
    ///
    /// The mirror term is the beam-direction asymmetry of `p p > l+ l- j`, and
    /// the ladder in `probe_mirror_visibility_ladder` measures it growing like
    /// `s-hat` while `s-hat` sits below the electroweak scale and saturating
    /// above it — the shape of a `gamma*/Z` core whose forward-backward asymmetry
    /// is set by `s-hat / m_Z²`. Fitting that shape to the measured plateau and
    /// halving it gives this floor, which sits between 1.58 and 4.86 times under
    /// every point of that ladder from 25 GeV to 4 TeV, over three independent
    /// streams and two sample sizes.
    fn mirror_visibility_floor(sqrt_s: f64) -> f64 {
        const M_Z2: f64 = 91.188 * 91.188;
        let s = sqrt_s * sqrt_s;
        0.076 * s / (s + M_Z2)
    }

    /// The ladder `mirror_visibility_floor` is fitted to, at sample sizes and
    /// stream seeds the gate does not use, so the floor stands on a measurement
    /// rather than on the one draw it is asserted against. Tenth percentiles:
    ///
    /// ```text
    /// npts stream          25       65      150      400     1200     4000
    ///   32 0x0ff5e7ed   2.3e-2   9.3e-2   1.0e-1   2.9e-1   2.5e-1   2.5e-1
    ///   32 0xdeadbeef   1.6e-2   7.8e-2   8.8e-2   1.8e-1   1.9e-1   1.9e-1
    ///   32 0x12345678   1.7e-2   5.9e-2   1.7e-1   2.7e-1   2.9e-1   2.8e-1
    ///  512 0x0ff5e7ed   1.3e-2   5.9e-2   1.0e-1   1.8e-1   1.9e-1   1.9e-1
    ///  512 0xdeadbeef   1.4e-2   6.0e-2   9.4e-2   1.5e-1   1.6e-1   1.7e-1
    ///  512 0x12345678   1.4e-2   6.7e-2   1.2e-1   1.9e-1   2.1e-1   2.1e-1
    /// ```
    ///
    /// It also shows why the bound is a percentile and not a minimum: the
    /// smallest visibility over the same draws falls by a decade going from 32
    /// points to 512 at every energy, because the two orderings agree exactly
    /// wherever the configuration happens to be symmetric and a larger sample
    /// gets closer to one. A minimum measures the sample, not the physics.
    #[test]
    #[ignore]
    fn probe_mirror_visibility_ladder() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let groups = derive(LLJ, &m, &evaluated);
        let masses = groups.groups()[0].final_masses();
        let pairs = mirror_pairs(&m, &evaluated, &groups);
        for npts in [32usize, 512] {
            for stream_seed in [0x0FF5_E7EDu64, 0xDEAD_BEEF, 0x1234_5678] {
                print!("npts {npts:4} stream {stream_seed:#011x}:");
                for sqrt_s in [10.0f64, 25.0, 65.0, 150.0, 400.0, 1200.0, 4000.0] {
                    let rels =
                        mirror_visibility(&evaluated, &pairs, &masses, sqrt_s, npts, stream_seed);
                    let p10 = rels[((rels.len() as f64 - 1.0) * 0.10) as usize];
                    print!(
                        "  {sqrt_s:.0}: p10 {p10:.2e} min {:.2e} ({:.2}x floor)",
                        rels[0],
                        p10 / mirror_visibility_floor(sqrt_s)
                    );
                }
                println!();
            }
        }
    }

    /// A decomposition whose groups carry *different* identical-particle factors is
    /// accepted, and each group's luminosity is weighted by its own members' factors.
    ///
    /// `u ū → g g` (`1/2`) and `u ū → d d̄` (`1`) share the outgoing mass list
    /// `[0, 0]` and a cut filter, so nothing below the outgoing flavours can tell
    /// their factors apart — the shape `p p > j j` is made of.
    #[test]
    fn groups_with_different_symmetry_factors_are_accepted_and_weighted_apart() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let mut sets = enumerate("u u~ > g g", &m);
        sets.extend(enumerate("u u~ > d d~", &m));
        let groups = derive_flavor_groups(sets, &m, &evaluated, &RunCard::default())
            .expect("a mixed-factor decomposition is accepted");
        assert_eq!(groups.groups().len(), 2);

        let pdf = probe_pdf();
        let (x1, x2, mu_f) = (0.12, 0.31, [MU_F, MU_F]);
        let mut factors = Vec::new();
        for g in groups.groups() {
            let raw = g.luminosity(&pdf, x1, x2, mu_f);
            let weighted = g.symmetry_weighted_luminosity(&pdf, x1, x2, mu_f);
            let expected: f64 = g.members()[0].symmetry_factor();
            assert!(
                g.members().iter().all(|s| s.symmetry_factor() == expected),
                "a group whose members disagree needs the per-member sum, not this check"
            );
            assert!(raw[0] > 0.0, "a group carries no direct luminosity");
            for k in 0..2 {
                assert_eq!(weighted[k], expected * raw[k]);
            }
            factors.push(expected);
        }
        factors.sort_by(f64::total_cmp);
        assert_eq!(factors, vec![0.5, 1.0]);
    }

    /// Every member of `p p > l+ l- j` carries factor one, so its enforced cross
    /// section is the row that would move if the weighting were applied where it
    /// does not belong.
    #[test]
    fn a_final_state_of_distinct_species_is_left_alone() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let groups = derive(LLJ, &m, &evaluated);
        let pdf = probe_pdf();
        for g in groups.groups() {
            assert!(g.members().iter().all(|s| s.symmetry_factor() == 1.0));
            let raw = g.luminosity(&pdf, 0.12, 0.31, [MU_F, MU_F]);
            let weighted = g.symmetry_weighted_luminosity(&pdf, 0.12, 0.31, [MU_F, MU_F]);
            assert_eq!(raw, weighted);
        }
    }

    /// The flavour partition of `p p > e+ e-` lands on the two Z/gamma* coupling
    /// classes — up-type `{u, c}` and down-type `{d, s}` — and each group's two
    /// beam orderings sum to that class's parton luminosity.
    #[test]
    fn drell_yan_classes_and_their_two_orderings_are_reproduced() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let groups = derive("p p > e+ e-", &m, &evaluated);
        assert_eq!(groups.groups().len(), 2);

        let pdf = probe_pdf();
        let (x1, x2, mu_f) = (0.03, 0.007, [91.188, 91.188]);
        let q2 = [mu_f[0] * mu_f[0], mu_f[1] * mu_f[1]];
        for g in groups.groups() {
            let quarks: Vec<i32> = g
                .members()
                .iter()
                .map(|s| s.incoming[0].abs())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            assert!(
                quarks == vec![2, 4] || quarks == vec![1, 3],
                "classes: {quarks:?}"
            );

            let expected: f64 = quarks
                .iter()
                .map(|&q| {
                    pdf.xfx_q2(q, x1, q2[0]) * pdf.xfx_q2(-q, x2, q2[1])
                        + pdf.xfx_q2(-q, x1, q2[0]) * pdf.xfx_q2(q, x2, q2[1])
                })
                .sum();
            let [direct, mirror] = g.luminosity(&pdf, x1, x2, mu_f);
            let rel = (direct + mirror - expected).abs() / expected;
            assert!(rel < 1e-14, "class luminosity off by {rel:.3e}");
            assert!(
                (direct - mirror).abs() / direct > 1e-3,
                "the two orderings coincide here, so this test cannot see a dropped one"
            );
        }
    }

    /// A `g g` initial state is one ordering, not two: its mirror carries no
    /// luminosity, while a `q q̄` group's does and differs from the direct term.
    #[test]
    fn an_identical_parton_initial_state_carries_a_single_ordering() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let groups = derive("p p > t t~ QED=0", &m, &evaluated);
        assert_eq!(groups.groups().len(), 2);
        assert_eq!(groups.subprocess_count(), 5);

        let pdf = probe_pdf();
        let (x1, x2, mu_f) = (0.05, 0.004, [173.0, 173.0]);
        let mut seen_gg = false;
        let mut bases = Vec::new();
        for g in groups.groups() {
            bases.push((g.evaluator().n_flows(), g.evaluator().cf_matrix().to_vec()));
            let [direct, mirror] = g.luminosity(&pdf, x1, x2, mu_f);
            if g.members()[0].incoming == [21, 21] {
                seen_gg = true;
                assert_eq!(g.members().len(), 1);
                assert!(!g.has_mirror());
                assert_eq!(mirror, 0.0);
                assert!((g.spin_color_average() - 1.0 / 256.0).abs() < 1e-15);
            } else {
                // At `QED = 0` every quark annihilation is the same colour and
                // coupling structure, so all four flavours are one group.
                assert_eq!(g.members().len(), 4);
                assert!(g.has_mirror());
                assert!(mirror > 0.0 && (direct - mirror).abs() / direct > 1e-3);
                assert!((g.spin_color_average() - 1.0 / 36.0).abs() < 1e-15);
            }
        }
        assert!(seen_gg, "no gluon-fusion group in p p > t t~");
        // The within-group colour-basis requirement is checked against a quantity
        // that separates real processes: gluon fusion and quark annihilation reach
        // the same final state through different colour bases. It does not fire
        // inside any group here, where the members differ by generation label alone.
        assert_ne!(bases[0], bases[1]);
    }

    /// A quark and its antiquark against a gluon are *different* groups, and the
    /// pointwise criterion is what says so.
    ///
    /// Their partonic cross sections agree within Monte-Carlo error — the banked
    /// `√ŝ = 500` runs give `0.11812 ± 0.00022 pb` for `g u > e+ e- u` against
    /// `0.11816 ± 0.00026 pb` for `g u~ > e+ e- u~` — so a criterion built on σ̂
    /// would have merged them, summing the antiquark's luminosity against the
    /// quark's matrix element. Pointwise `|M|²` separates them by more than 10%,
    /// which is also why they need separate colour structures downstream.
    #[test]
    fn a_quark_and_its_antiquark_against_a_gluon_do_not_share_a_matrix_element() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let groups = derive(LLJ, &m, &evaluated);
        let points = fresh_points(&groups.groups()[0].final_masses());

        let find = |beams: [i32; 2]| {
            groups
                .groups()
                .iter()
                .position(|g| g.members().iter().any(|s| s.incoming == beams))
                .unwrap_or_else(|| panic!("no group holds {beams:?}"))
        };
        let mut worst = 0.0f64;
        for quark in [2, 1] {
            let (q, qbar) = (find([21, quark]), find([21, -quark]));
            assert_ne!(q, qbar, "g q and g q̄ were merged into one group");
            let rel = worst_rel(
                &m2_trace(groups.groups()[q].evaluator(), &evaluated, &points),
                &m2_trace(groups.groups()[qbar].evaluator(), &evaluated, &points),
            );
            worst = worst.max(rel);
            assert!(
                rel > 0.1,
                "g {quark} and g -{quark} differ by only {rel:.3e}"
            );
        }
        eprintln!("g q vs g q̄ pointwise |M|² separation up to {worst:.3e}");
    }

    /// The shared-cut-filter requirement has teeth: a b-quark jet is a different
    /// filter from a light-quark jet at `maxjetflavor = 4`, where MadGraph cuts it
    /// with `ptb` instead of `ptj`.
    #[test]
    fn a_group_sharing_one_cut_filter_is_a_real_requirement() {
        let legs = |jet: i32, mass: f64| {
            vec![
                ExternalLeg::incoming(21, 0.0),
                ExternalLeg::incoming(jet, 0.0),
                ExternalLeg::outgoing(-11, 0.0),
                ExternalLeg::outgoing(11, 0.0),
                ExternalLeg::outgoing(jet, mass),
            ]
        };
        let mut card = RunCard::default();
        assert_eq!(card.maxjetflavor, 4);
        let light = Cuts::compile(&card, &legs(2, 0.0)).expect("light-jet cuts");
        let bottom = Cuts::compile(&card, &legs(5, 4.7)).expect("b-jet cuts");
        assert_ne!(
            light, bottom,
            "a b leg compiles to the same filter as a light jet, so the group check is vacuous"
        );

        card.maxjetflavor = 5;
        let bottom_as_jet = Cuts::compile(&card, &legs(5, 4.7)).expect("b-as-jet cuts");
        assert_eq!(
            light, bottom_as_jet,
            "the difference is the jet classification, not the leg's mass"
        );
    }

    /// The banked fixed-scale `p p > l+ l- j` configuration: 13 TeV proton beams,
    /// every scale fixed at `m_Z`, and the cuts the reference run was taken with.
    fn llj_card() -> RunCard {
        RunCard::parse(
            "  1 = lpp1\n  1 = lpp2\n  6500.0 = ebeam1\n  6500.0 = ebeam2\n\
             \x20 lhapdf = pdlabel\n  247000 = lhaid\n\
             \x20 True = fixed_ren_scale\n  True = fixed_fac_scale1\n  True = fixed_fac_scale2\n\
             \x20 91.188 = scale\n  91.188 = dsqrt_q2fact1\n  91.188 = dsqrt_q2fact2\n\
             \x20 -1 = dynamical_scale_choice\n\
             \x20 20.0 = ptj\n  10.0 = ptl\n  5.0 = etaj\n  2.5 = etal\n\
             \x20 0.4 = drll\n  0.4 = drjl\n  50.0 = mmll\n  4 = maxjetflavor\n",
        )
        .expect("run card")
    }

    const SQRT_S_HAD: f64 = 13000.0;
    const MU_F: f64 = 91.188;

    /// `αs` knots bracketing `m_Z`, so a card that delegates the coupling to the set
    /// resolves without the fetched grid.
    fn probe_alpha_s() -> AlphaSInfo {
        AlphaSInfo {
            mz: 91.1876,
            order_qcd: 1,
            kind: "ipol".to_string(),
            qs: vec![1.0, 91.1876, 109.8541, 10000.0],
            vals: vec![0.4, 0.1300028, 0.1262725, 0.08],
            lambda4: 0.0,
            lambda5: 0.0,
        }
    }

    /// A `√ŝ`-independent channel set built exactly as the integrand builds it — the
    /// oracle's own copy of the phase-space map, so the point and weight it compares
    /// against are reproduced rather than read out of the integrand.
    fn rebuild_channels(
        groups: &FlavorGroups,
        model: &EvaluatedModel,
        floor: f64,
    ) -> ScaledMultiChannel<f64> {
        let mut channels: Vec<Box<dyn ScaledChannel<f64>>> = Vec::new();
        for g in groups.groups() {
            for d in g.diagrams() {
                channels.push(Box::new(DiagramChannel::from_diagram_regulated(
                    d, model, SQRT_S_HAD, floor,
                )));
            }
        }
        ScaledMultiChannel::uniform(channels)
    }

    fn bind_all<'a>(
        groups: &'a FlavorGroups,
        evaluated: &'a EvaluatedModel,
    ) -> Vec<BoundAmplitude<'a, f64>> {
        groups
            .groups()
            .iter()
            .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), evaluated))
            .collect()
    }

    /// Every diagram of every group becomes a channel, they are pooled into one
    /// mixture, and the peripheral ones are regulated at the scale the cuts imply.
    ///
    /// The scale is what *builds* those channels: at scale zero the same diagrams
    /// give an all-timelike tree, which is why it is passed rather than defaulted.
    /// It regulates them two ways: the token pole floor the propagator map draws
    /// against, which the banked summary records and this checks, and the transverse
    /// bound on the transfer, whose effect on the drawn points the channel's own
    /// tests measure.
    #[test]
    fn every_diagram_of_every_group_becomes_a_floored_channel() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let card = llj_card();
        let groups = derive_flavor_groups(enumerate(LLJ, &m), &m, &evaluated, &card)
            .expect("flavour groups");
        let amps = bind_all(&groups, &evaluated);
        let pdf = probe_pdf();
        let integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
            .expect("integrand");

        assert_eq!(integ.channel_count(), 24);
        assert_eq!(integ.channel_grid_ndim(), 2 + 5);
        let ids: Vec<ChannelId> = integ.channel_ids().to_vec();
        for (g, group) in groups.groups().iter().enumerate() {
            for d in 0..group.diagrams().len() {
                assert!(ids.contains(&ChannelId {
                    group: g,
                    diagram: d
                }));
            }
        }
        assert_eq!(integ.spacelike_floor(), 400.0);

        let (mut floored, mut unfloored) = (0, 0);
        for g in groups.groups() {
            for d in g.diagrams() {
                let with = DiagramChannel::<f64>::from_diagram_regulated(
                    d,
                    &evaluated,
                    SQRT_S_HAD,
                    integ.spacelike_floor(),
                );
                let without =
                    DiagramChannel::<f64>::from_diagram_regulated(d, &evaluated, SQRT_S_HAD, 0.0);
                assert!(
                    without.spine_poles().is_empty(),
                    "a three-body spine was built without a floor"
                );
                match with.spine_poles().as_slice() {
                    [] => unfloored += 1,
                    poles => {
                        assert_eq!(poles, [0.4], "an llj diagram carries one spacelike line");
                        floored += 1;
                    }
                }
            }
        }
        eprintln!("{floored} peripheral channels, {unfloored} all-timelike");
        assert!(
            floored > 0,
            "no channel carries a peripheral spine, so the floor is doing nothing here"
        );

        // The banked summary reports what the composition actually chose. Compared
        // against the channels re-derived above rather than against a hand-written
        // expectation, so it cannot pass by both sides agreeing on a default: the
        // topology, the floored pole and the propagator poles all have to line up
        // channel by channel.
        let samplers = integ.channel_samplers();
        assert_eq!(samplers.len(), integ.channel_count());
        let mut spines = 0;
        let mut resonant = 0;
        let mut k = 0;
        for g in groups.groups() {
            for d in g.diagrams() {
                let built = DiagramChannel::<f64>::from_diagram_regulated(
                    d,
                    &evaluated,
                    SQRT_S_HAD,
                    integ.spacelike_floor(),
                );
                assert_eq!(samplers[k], ChannelSampler::of(&built), "channel {k}");
                match samplers[k].topology {
                    SamplerTopology::Spine => {
                        assert_eq!(samplers[k].spine_poles_gev2, vec![0.4]);
                        assert_eq!(samplers[k].t_channels.len(), 1);
                        spines += 1;
                    }
                    SamplerTopology::Timelike => {
                        assert!(samplers[k].spine_poles_gev2.is_empty());
                        assert!(samplers[k].t_channels.is_empty());
                    }
                }
                // Every llj channel draws one dilepton invariant, on the photon
                // pole or on the Z's; a summary that lost the widths would report
                // the Z as massless.
                assert_eq!(samplers[k].resonances.len(), 1, "channel {k}");
                let pole = samplers[k].resonances[0];
                if pole.mass != 0.0 {
                    assert_eq!(pole.mass, evaluated.mass(z_id(&m)));
                    assert_eq!(pole.width, evaluated.width(z_id(&m)));
                    resonant += 1;
                } else {
                    assert_eq!(pole.width, 0.0);
                }
                k += 1;
            }
        }
        assert_eq!(spines, floored);
        assert!(
            resonant > 0 && resonant < samplers.len(),
            "the summary reports the same pole on every channel, so it cannot be \
             distinguishing the Z exchange from the photon"
        );
    }

    /// The model's own particle id for the Z, so the summary's poles are compared
    /// against the model rather than against transcribed numbers.
    fn z_id(m: &UFOModel) -> crate::ufo::particles::ParticleId {
        m.particle_id("Z").expect("the model has a Z")
    }

    /// The integrand's value at a point, against an assembly built from the process
    /// data alone.
    ///
    /// The oracle re-derives the `(τ, y)` map, both frames, the flux and the `2π`
    /// measure, and — decisively — takes the mirrored beam ordering from an
    /// **explicitly enumerated** `b a > …` subprocess evaluated at the *unreflected*
    /// point, with each member's `x·f` product formed directly from the parton
    /// distribution. So it shares neither [`FlavorGroup::luminosity`] nor
    /// [`FlavorGroup::mirror_into`] with the integrand: a dropped or mis-argued mirror
    /// term, a swapped beam ordering, or a lost spin/colour average all move it.
    ///
    /// What it cannot see: the phase-space weight, which it takes from its own copy of
    /// the same channel construction, and anything the cut filter and the parton
    /// distribution agree on being wrong about.
    ///
    /// The two sides multiply the same factors in different orders over a sum whose
    /// mirror term can carry the whole of a group's contribution, so they agree only
    /// to reassociation noise: worst `2.9e-13` over a twelve-seed sweep of the point
    /// stream, the largest of them from configurations sitting on the transverse
    /// cut edge. The bound sits a few times above that and many orders below any
    /// structural defect — a dropped mirror term or a lost average moves the ratio
    /// by a finite fraction, not by an ulp.
    #[test]
    fn a_point_reproduces_an_independently_assembled_integrand() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let card = llj_card();
        let groups = derive_flavor_groups(enumerate(LLJ, &m), &m, &evaluated, &card)
            .expect("flavour groups");
        let amps = bind_all(&groups, &evaluated);
        let pdf = probe_pdf();
        let integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
            .expect("integrand");

        // The mirrored ordering of each group, compiled from its own proc card.
        let mirrored: Vec<BoundAmplitude<f64>> = Vec::new();
        let mirror_evals: Vec<AmplitudeEvaluator> = groups
            .groups()
            .iter()
            .map(|g| {
                let set = g.diagram_set();
                let swapped = format!(
                    "{} {} > {} QCD=2 QED=2",
                    set.particles_in[1],
                    set.particles_in[0],
                    set.particles_out.join(" ")
                );
                let mirror_set = enumerate(&swapped, &m)
                    .into_iter()
                    .find(|s| !s.diagrams.is_empty())
                    .expect("the mirrored ordering enumerates");
                compile_class(&mirror_set, &m, &evaluated).expect("mirror compiles")
            })
            .collect();
        drop(mirrored);
        let mirror_bound: Vec<BoundAmplitude<f64>> = mirror_evals
            .iter()
            .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
            .collect();
        let direct_bound = bind_all(&groups, &evaluated);

        let combiner = rebuild_channels(&groups, &evaluated, 400.0);
        let cuts = groups.groups()[0].cuts();
        let lips_2pi = (2.0 * PI).powi(4 - 3 * 3);
        let tau_min = cuts.shat_min() / (SQRT_S_HAD * SQRT_S_HAD);
        let q2 = [MU_F * MU_F, MU_F * MU_F];

        let mut stream = SubStream::from_stream(0xBEEF_0D1E, 7);
        let (mut worst, mut checked, mut rejected, mut mirror_weight) = (0.0f64, 0, 0, 0.0f64);
        for trial in 0..400 {
            let u = stream.uniforms::<f64>(integ.point_ndim());
            let channel = trial % integ.channel_count();

            let tau = tau_min.powf(1.0 - u[0]);
            let y_max = -0.5 * tau.ln();
            let y = (2.0 * u[1] - 1.0) * y_max;
            let (x1, x2) = (tau.sqrt() * y.exp(), tau.sqrt() * (-y).exp());
            let sqrt_shat = (tau * SQRT_S_HAD * SQRT_S_HAD).sqrt();
            let jac = (1.0 / tau_min).ln() * 2.0 * y_max;

            let point = combiner.sample_channel_at(channel, sqrt_shat, &u[2..]);
            let e_cm = sqrt_shat / 2.0;
            let mut cm = vec![V::new(e_cm, 0.0, 0.0, e_cm), V::new(e_cm, 0.0, 0.0, -e_cm)];
            cm.extend_from_slice(&point.momenta);
            let e_beam = SQRT_S_HAD / 2.0;
            let beta = (x1 - x2) / (x1 + x2);
            let mut lab = vec![
                V::new(x1 * e_beam, 0.0, 0.0, x1 * e_beam),
                V::new(x2 * e_beam, 0.0, 0.0, -x2 * e_beam),
            ];
            lab.extend(point.momenta.iter().map(|p| boost_z(*p, beta)));

            let expected = if !cuts.pass(&lab) {
                rejected += 1;
                0.0
            } else {
                let mut acc = 0.0;
                for (gi, g) in groups.groups().iter().enumerate() {
                    let mut scratch = direct_bound[gi].scratch_space();
                    let m2_direct = direct_bound[gi].eval_m2(&cm, &mut scratch);
                    let mut mirror_scratch = mirror_bound[gi].scratch_space();
                    let m2_mirror = mirror_bound[gi].eval_m2(&cm, &mut mirror_scratch);
                    let mut term = 0.0;
                    for s in g.members() {
                        let [a, b] = s.incoming;
                        term += pdf.xfx_q2(a, x1, q2[0]) * pdf.xfx_q2(b, x2, q2[1]) * m2_direct;
                        if a != b {
                            let w = pdf.xfx_q2(b, x1, q2[0]) * pdf.xfx_q2(a, x2, q2[1]);
                            term += w * m2_mirror;
                            mirror_weight = mirror_weight
                                .max(w * m2_mirror / (term).abs().max(f64::MIN_POSITIVE));
                        }
                    }
                    acc += g.spin_color_average() * term;
                }
                let flux = 1.0 / (2.0 * sqrt_shat * sqrt_shat);
                jac * flux * lips_2pi * acc * point.weight
            };

            let got = integ.value_in_channel(channel, &u);
            let rel = (got - expected).abs() / expected.abs().max(f64::MIN_POSITIVE);
            worst = worst.max(rel);
            checked += 1;
        }
        eprintln!(
            "pointwise worst {worst:.3e} over {checked} points ({rejected} cut-rejected); \
             the mirror carries up to {mirror_weight:.3} of a group's term"
        );
        assert!(
            checked - rejected > 30,
            "only {} of {checked} points landed inside the cuts",
            checked - rejected
        );
        assert!(
            rejected > 0,
            "no point was cut-rejected, so the zero branch is unchecked"
        );
        assert!(
            mirror_weight > 0.1,
            "the mirror term is worth only {mirror_weight:.3e} of a group's contribution, so \
             this oracle could not see it dropped"
        );
        assert!(worst < 2e-12, "pointwise disagreement {worst:.3e}");
    }

    /// A bound amplitude paired with the wrong group is refused. Crossing the pairing
    /// weights one group's matrix element with another's luminosity — a smooth shift
    /// of the cross section with no other symptom — so the check is by pointer
    /// identity, not by shape.
    #[test]
    fn a_crossed_group_amplitude_pairing_is_refused() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let card = llj_card();
        let groups = derive_flavor_groups(enumerate(LLJ, &m), &m, &evaluated, &card)
            .expect("flavour groups");
        let pdf = probe_pdf();

        let mut crossed: Vec<BoundAmplitude<f64>> = bind_all(&groups, &evaluated);
        crossed.swap(0, 1);
        let err = match ProtonIntegrand::new(&groups, &crossed, &evaluated, &pdf, SQRT_S_HAD, MU_F)
        {
            Ok(_) => panic!("a crossed amplitude pairing must be refused"),
            Err(e) => e,
        };
        assert!(
            matches!(err, ProtonError::AmplitudeMismatch { index: 0 }),
            "unexpected refusal: {err}"
        );

        let short = &bind_all(&groups, &evaluated)[..2];
        assert!(matches!(
            ProtonIntegrand::new(&groups, short, &evaluated, &pdf, SQRT_S_HAD, MU_F),
            Err(ProtonError::AmplitudeCount { amps: 2, groups: 6 })
        ));
    }

    /// The parallel channel integration reproduces the sequential one it replaced,
    /// on a process whose configuration draw carries weight.
    ///
    /// `p p > l+ l- j` under a clustering renormalisation scale is the case the
    /// fixed-beam sequential reference cannot cover: the drawn configuration picks
    /// the merge sequence the scale is read from, so the trailing uniform moves
    /// `|M|²` — checked here rather than assumed, since a `2 → 2` draws the same
    /// uniform and changes nothing. Both substreams are therefore live, and the
    /// reference consumes each point by point exactly as a serial integrator
    /// would, written out rather than shared with the implementation so the two
    /// cannot drift together.
    #[test]
    fn the_parallel_integration_reproduces_a_sequential_one() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let card = RunCard::parse(
            "  1 = lpp1\n  1 = lpp2\n  6500.0 = ebeam1\n  6500.0 = ebeam2\n\
             \x20 lhapdf = pdlabel\n  247000 = lhaid\n  -1 = dynamical_scale_choice\n\
             \x20 False = fixed_ren_scale\n  False = fixed_fac_scale1\n\
             \x20 False = fixed_fac_scale2\n\
             \x20 20.0 = ptj\n  10.0 = ptl\n  5.0 = etaj\n  2.5 = etal\n\
             \x20 0.4 = drll\n  0.4 = drjl\n  50.0 = mmll\n  4 = maxjetflavor\n",
        )
        .expect("dynamical run card");
        let groups = derive_flavor_groups(enumerate(LLJ, &m), &m, &evaluated, &card)
            .expect("flavour groups");
        let amps = bind_all(&groups, &evaluated);
        let pdf = probe_pdf();
        let info = probe_alpha_s();
        let mut integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
            .expect("integrand");
        integ
            .use_run_card_scales(&m, &evaluated, &card, Some(&info))
            .expect("the dynamical prescription compiles");

        // The configuration draw has to reach the value, or the substream it runs
        // on could be addressed any way at all and this test would still pass.
        assert_eq!(integ.scale_draw_ndim(), 1, "no configuration draw installed");
        let ndim = integ.point_ndim();
        let mut probes = 0;
        let mut moved = 0;
        let mut s = SubStream::from_stream(0x5EED_D1, 4);
        for _ in 0..400 {
            let mut u = s.uniforms::<f64>(ndim);
            u[ndim - 1] = 0.02;
            let a = integ.value_in_channel(0, &u);
            if a == 0.0 {
                continue;
            }
            probes += 1;
            u[ndim - 1] = 0.98;
            moved += usize::from(a.to_bits() != integ.value_in_channel(0, &u).to_bits());
        }
        assert!(probes > 0, "every probe was cut away, so nothing was compared");
        assert!(
            moved > 0,
            "{probes} probes carried weight and the configuration draw moved none of them"
        );

        let (seed, neval, niter) = (0x5EED_D2, 2_000, 3);
        let (got, _) = integ.adapt_grids(neval, niter, seed);

        let grid_ndim = integ.channel_grid_ndim();
        let point_ndim = integ.point_ndim();
        let alphas = integ.channel_alphas();
        for (j, &alpha) in alphas.iter().enumerate() {
            let n_j = if alphas.len() == 1 {
                neval
            } else {
                channel_neval(alpha, neval)
            };
            let mut grid = VegasGrid::new(grid_ndim, VEGAS_NBINS, VEGAS_ALPHA_MAPPED);
            let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
            rng.set_stream(CHANNEL_STREAM_BASE + j as u64);
            rng.set_word_pos(0);
            let mut scale_draw = SubStream::from_stream(seed, SCALE_DRAW_STREAM_BASE + j as u64);
            let mut point = vec![0.0; point_ndim];
            let want = grid.adapt(
                |u| {
                    point[..grid_ndim].copy_from_slice(u);
                    scale_draw.fill_uniforms(&mut point[grid_ndim..]);
                    integ.value_in_channel(j, &point)
                },
                n_j,
                niter,
                &mut rng,
            );
            assert_eq!(
                got[j].result.integral.to_bits(),
                want.integral.to_bits(),
                "channel {j} term"
            );
            for (dim, (a, b)) in got[j].grid.xi().iter().zip(grid.xi()).enumerate() {
                assert_eq!(a, b, "channel {j} grid edges of dim {dim}");
            }
        }
    }

    /// The fixed-scale card resolves to constants and reads `αs` from the set's own
    /// tabulation; the dynamical card of the same process resolves too, over a set
    /// whose table stops below the collider.
    ///
    /// That combination is the ordinary one rather than a corner: this set
    /// tabulates `αs` to 10 TeV and a per-event scale on a 13 TeV collider can
    /// exceed it, on some events and not on all. Past its last knot LHAPDF holds
    /// the coupling at the last tabulated value, so the reading is defined
    /// wherever the clustering can land, and a run does not depend on which of its
    /// events happen to cross the top of the table.
    #[test]
    fn a_dynamical_scale_resolves_where_the_table_stops_below_the_collider() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let card = llj_card();
        let groups = derive_flavor_groups(enumerate(LLJ, &m), &m, &evaluated, &card)
            .expect("flavour groups");
        let amps = bind_all(&groups, &evaluated);
        let pdf = probe_pdf();
        let info = probe_alpha_s();

        let mut integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
            .expect("integrand");
        let report = integ
            .use_run_card_scales(&m, &evaluated, &card, Some(&info))
            .expect("the fixed prescription compiles");
        let constant = report
            .constant_scales
            .expect("the reference card fixes every scale");
        assert_eq!((constant.mu_r, constant.mu_f), (MU_F, [MU_F, MU_F]));
        assert!(
            report.depends_on_alpha_s,
            "a QCD=2 matrix element must move with the strong coupling"
        );
        // The coupling is the grid's, not the parameter card's: the card's own
        // `pdlabel` is what routes it there, and a beta-function solve is refused.
        let grid_alpha_s = report.constant_alpha_s.expect("a coupling was installed");
        assert!((grid_alpha_s - 0.1300028).abs() < 1e-6, "{grid_alpha_s}");
        assert!(
            (grid_alpha_s - report.alpha_s_ref.expect("bound coupling")).abs() > 1e-4,
            "the grid coupling coincides with the parameter card's, so this cannot tell \
             which source was used"
        );
        assert!(
            integ
                .use_run_card_scales(&m, &evaluated, &card, None)
                .is_err(),
            "a card whose alpha_s lives in the set must refuse when the set is withheld"
        );

        let mut text = String::new();
        for (name, value) in [
            ("fixed_ren_scale", "False"),
            ("fixed_fac_scale1", "False"),
            ("fixed_fac_scale2", "False"),
        ] {
            text.push_str(&format!("  {value} = {name}\n"));
        }
        let dynamical = RunCard::parse(&format!(
            "{}{text}",
            "  1 = lpp1\n  1 = lpp2\n  6500.0 = ebeam1\n  6500.0 = ebeam2\n\
             \x20 lhapdf = pdlabel\n  247000 = lhaid\n  -1 = dynamical_scale_choice\n\
             \x20 20.0 = ptj\n  10.0 = ptl\n  5.0 = etaj\n  2.5 = etal\n\
             \x20 0.4 = drll\n  0.4 = drjl\n  50.0 = mmll\n  4 = maxjetflavor\n"
        ))
        .expect("dynamical run card");
        let mut integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
            .expect("integrand");
        let report = integ
            .use_run_card_scales(&m, &evaluated, &dynamical, Some(&info))
            .expect("the dynamical prescription compiles");
        assert!(
            report.constant_scales.is_none(),
            "the dynamical card must read the event: {report:?}"
        );
        let grid = integ
            .alpha_s_source()
            .and_then(AlphaSSource::grid)
            .expect("the set's own tabulation");

        let collider = dynamical.ebeam1 + dynamical.ebeam2;
        let (_, q_max) = grid.q_range();
        assert!(
            q_max < collider,
            "the fixture's table reaches {q_max}, so it no longer stops below the \
             {collider} GeV collider this asserts about"
        );
        let last = *info.vals.last().expect("tabulated values");
        assert_eq!(grid.eval(q_max), last);
        for q in [q_max * 1.000_001, 0.5 * (q_max + collider), collider] {
            assert_eq!(grid.eval(q), last, "at Q = {q}");
        }
    }

    /// The banked `p p > l+ l- j` configuration with `μF` scaled by `scalefact`,
    /// for the tests that exercise the factorisation floor. `scalefact` multiplies
    /// every factorisation scale uniformly, so it moves the whole distribution
    /// down against a fixed floor without touching the cuts, the flavours or the
    /// phase-space map — which is what makes the `scalefact = 1` control a
    /// controlled comparison rather than a different run.
    fn scaled_llj_card(scalefact: &str) -> RunCard {
        RunCard::parse(&format!(
            "  1 = lpp1\n  1 = lpp2\n  6500.0 = ebeam1\n  6500.0 = ebeam2\n\
             \x20 lhapdf = pdlabel\n  247000 = lhaid\n\
             \x20 -1 = dynamical_scale_choice\n  {scalefact} = scalefact\n\
             \x20 20.0 = ptj\n  10.0 = ptl\n  5.0 = etaj\n  2.5 = etal\n\
             \x20 0.4 = drll\n  0.4 = drjl\n  50.0 = mmll\n  4 = maxjetflavor\n"
        ))
        .expect("run card")
    }

    /// `reweight.f` answers a point whose factorisation scale fell below 2 GeV by
    /// zeroing its weight and carrying on (`reweight.f:1907-1908`). This is that
    /// behaviour reaching the integrand: the vetoed points evaluate to exactly
    /// `0.0`, and the run does not stop.
    ///
    /// The `scalefact = 1` control is part of the same test and has to be — on its
    /// own a zero cannot be told from a point the cuts rejected or one where the
    /// densities returned nothing. What identifies the veto is that the *same*
    /// points carry weight when the scale is not scaled down.
    ///
    /// It cannot see a veto firing too often on an ordinary card; the banked
    /// factorisation-floor margin and the unmoved reference cross sections cover
    /// that direction.
    #[test]
    fn a_sub_threshold_factorisation_scale_gives_zero_weight() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let groups = derive_flavor_groups(enumerate(LLJ, &m), &m, &evaluated, &llj_card())
            .expect("flavour groups");
        let amps = bind_all(&groups, &evaluated);
        let pdf = probe_pdf();
        let info = probe_alpha_s();
        let build = |scalefact: &str| {
            let mut integ =
                ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
                    .expect("integrand");
            integ
                .use_run_card_scales(&m, &evaluated, &scaled_llj_card(scalefact), Some(&info))
                .expect("a partly sub-threshold card still compiles");
            integ
        };
        let control = build("1.0");
        let vetoed = build("1e-3");

        let mut stream = SubStream::from_stream(0x00C2_A111, 3);
        let (mut carried, mut dropped) = (0usize, 0usize);
        for i in 0..300 {
            let channel = i % control.channel_count();
            let u = stream.uniforms::<f64>(control.point_ndim());
            if control.value_in_channel(channel, &u) == 0.0 {
                continue;
            }
            carried += 1;
            if vetoed.value_in_channel(channel, &u) == 0.0 {
                dropped += 1;
            }
        }
        assert!(
            carried > 0,
            "no point carried weight at scalefact = 1, so there was nothing to veto"
        );
        assert!(
            dropped > 0,
            "{carried} points carry weight at scalefact = 1 and none was vetoed at 1e-3"
        );
        eprintln!(
            "factorisation floor: {dropped} of {carried} weight-carrying points give exactly 0.0 \
             once mu_F is scaled below the floor"
        );
    }

    /// The veto reaches generation, not only the integral. `event_in_channel`
    /// evaluates `shape` first and returns `None` on a zero, so a sample and the
    /// integral it came from drop the same points — which is what keeps a
    /// sample's scales consistent with its own cross section.
    #[test]
    fn a_vetoed_point_is_dropped_from_generation_too() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let groups = derive_flavor_groups(enumerate(LLJ, &m), &m, &evaluated, &llj_card())
            .expect("flavour groups");
        let amps = bind_all(&groups, &evaluated);
        let pdf = probe_pdf();
        let info = probe_alpha_s();
        let build = |scalefact: &str| {
            let mut integ =
                ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
                    .expect("integrand");
            integ
                .use_run_card_scales(&m, &evaluated, &scaled_llj_card(scalefact), Some(&info))
                .expect("a partly sub-threshold card still compiles");
            integ
        };
        let control = build("1.0");
        let vetoed = build("1e-3");

        let mut stream = SubStream::from_stream(0x00C2_B222, 3);
        let mut dropped = 0usize;
        for i in 0..300 {
            let channel = i % control.channel_count();
            let u = stream.uniforms::<f64>(control.point_ndim());
            if control.event_in_channel(channel, &u).is_none() {
                continue;
            }
            let generated = vetoed.event_in_channel(channel, &u).is_some();
            let integrated = vetoed.value_in_channel(channel, &u) != 0.0;
            assert_eq!(
                generated, integrated,
                "generation and integration disagree about a point in channel {channel}"
            );
            if !generated {
                dropped += 1;
            }
        }
        assert!(
            dropped > 0,
            "no weight-carrying point was dropped, so generation was never exercised on a veto"
        );
        eprintln!(
            "factorisation floor: {dropped} points dropped from generation as well as from the \
             integral"
        );
    }

    /// A card whose support is *entirely* below the floor is refused at setup
    /// rather than integrated to zero. The scale probe already draws 64 points to
    /// surface a refusal before the integration starts; it now steps over vetoed
    /// draws instead of reporting the first one, so a partly vetoed card passes
    /// it and an entirely vetoed one does not.
    ///
    /// This is deliberately not a per-event counter: a partly vetoed run is
    /// legitimate physics with a correct cross section, so only the degenerate
    /// case is actionable — and it is actionable before anything is spent on it.
    #[test]
    fn a_wholly_sub_threshold_card_is_refused_at_setup() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let groups = derive_flavor_groups(enumerate(LLJ, &m), &m, &evaluated, &llj_card())
            .expect("flavour groups");
        let amps = bind_all(&groups, &evaluated);
        let pdf = probe_pdf();
        let info = probe_alpha_s();

        let mut integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
            .expect("integrand");
        let err = integ
            .use_run_card_scales(&m, &evaluated, &scaled_llj_card("1e-4"), Some(&info))
            .expect_err("a card with no support above the floor must be refused");
        assert!(
            matches!(
                err,
                ProtonError::Hadronic(HadronicError::FactorisationScaleBelowFloor)
            ),
            "refused for the wrong reason: {err}"
        );
        eprintln!("factorisation floor, wholly sub-threshold card: {err}");

        // One decade up there is support left, and the same probe accepts it —
        // so the refusal tracks the support and not merely a small scalefact.
        let mut integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
            .expect("integrand");
        integ
            .use_run_card_scales(&m, &evaluated, &scaled_llj_card("1e-3"), Some(&info))
            .expect("a partly sub-threshold card integrates normally");
    }

    /// The `ŝ` window the cuts *hint* is looser than the one they impose, and the
    /// integrand's `τ` map is built on the hint. Both halves matter: a hint above the
    /// true threshold would cut real phase space, and one far below it would leave the
    /// grid to find the ridge alone.
    ///
    /// The measurement below is the second half — what the looseness actually costs,
    /// and that every draw it admits below threshold is rejected by the cuts rather
    /// than contributing.
    #[test]
    fn the_tau_map_is_loose_below_the_true_threshold_by_a_measured_margin() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let card = llj_card();
        let groups = derive_flavor_groups(enumerate(LLJ, &m), &m, &evaluated, &card)
            .expect("flavour groups");
        let amps = bind_all(&groups, &evaluated);
        let pdf = probe_pdf();
        let integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
            .expect("integrand");

        // A jet above `ptj` recoiling against a dilepton pair above `mmll` needs
        // `√ŝ ≥ √(mmll² + ptj²) + ptj`.
        let (mmll, ptj) = (50.0f64, 20.0f64);
        let shat_true = ((mmll * mmll + ptj * ptj).sqrt() + ptj).powi(2);
        let shat_hint = integ.tau_min() * SQRT_S_HAD * SQRT_S_HAD;
        assert!(
            shat_hint <= shat_true,
            "the hint {shat_hint} is above the true threshold {shat_true}, so the map cuts \
             physical phase space"
        );
        eprintln!(
            "shat_min hint {shat_hint:.1} vs true {shat_true:.1}: loose by {:.2}x, \
             {:.2} of ln(1/tau_min) = {:.2}",
            shat_true / shat_hint,
            (shat_true / shat_hint).ln(),
            (1.0 / integ.tau_min()).ln()
        );

        let mut stream = SubStream::from_stream(0x7A0_1111, 3);
        let (mut below, mut n) = (0usize, 4000usize);
        for i in 0..n {
            let u = stream.uniforms::<f64>(integ.point_ndim());
            let tau = integ.tau_min().powf(1.0 - u[0]);
            if tau * SQRT_S_HAD * SQRT_S_HAD < shat_true {
                below += 1;
                let v = integ.value_in_channel(i % integ.channel_count(), &u);
                assert_eq!(
                    v, 0.0,
                    "a point below the true threshold carried weight {v}"
                );
            }
        }
        n = n.max(1);
        let share = below as f64 / n as f64;
        eprintln!(
            "{:.1}% of tau draws land below the true threshold",
            100.0 * share
        );
        assert!(
            share < 0.1,
            "{:.1}% of draws are wasted below threshold; the hint is too loose to leave alone",
            100.0 * share
        );
    }

    /// A plain-Monte-Carlo estimate of `Σⱼ ∫ duᵢ value_in_channel(j, ·)` with the two
    /// outer coordinates frozen, and its standard error.
    fn inner_integral(
        integ: &ProtonIntegrand<'_>,
        outer: [f64; 2],
        n: usize,
        seed: u64,
    ) -> [f64; 2] {
        let ndim = integ.point_ndim();
        let (mut total, mut var) = (0.0, 0.0);
        for j in 0..integ.channel_count() {
            let mut stream = SubStream::from_stream(seed, j as u64);
            let (mut sum, mut sum2) = (0.0, 0.0);
            for _ in 0..n {
                let mut u = vec![outer[0], outer[1]];
                u.extend(stream.uniforms::<f64>(ndim - 2));
                let v = integ.value_in_channel(j, &u);
                sum += v;
                sum2 += v * v;
            }
            let mean = sum / n as f64;
            total += mean;
            var += (sum2 / n as f64 - mean * mean).max(0.0) / n as f64;
        }
        [total, var.sqrt()]
    }

    /// At a frozen partonic energy and zero rapidity the integrand is the partonic
    /// cross section of every group times that group's parton luminosity — the
    /// statement that the PDF layer contributes exactly a factor of luminosity.
    ///
    /// Zero rapidity is what makes the comparison exact: the lab frame then coincides
    /// with the partonic CM, so the two sides apply one cut filter to the same
    /// configuration. The partonic side is [`FixedBeamIntegrand`], sampled through its
    /// *own* map (all-timelike per-diagram channels at fixed `√ŝ`) rather than this
    /// integrand's floored spines, so the flux, the `2π` measure, the spin/colour
    /// average and the identical-particle factor are compared across two independent
    /// phase-space maps.
    ///
    /// What it cannot see: the rapidity boost, since it is switched off, and the
    /// mirrored matrix element's *argument* — at `y = 0` the two beam orderings carry
    /// equal luminosity and `∫|M(Rq)|²Θ(q)dΦ = ∫|M(q)|²Θ(q)dΦ`, so a mirror evaluated
    /// at the wrong point would still integrate here. The pointwise oracle is what
    /// pins the argument.
    #[test]
    fn at_fixed_energy_the_integrand_is_the_partonic_cross_section_times_luminosity() {
        use crate::hadronic::FixedBeamIntegrand;

        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let card = llj_card();
        let groups = derive_flavor_groups(enumerate(LLJ, &m), &m, &evaluated, &card)
            .expect("flavour groups");
        let amps = bind_all(&groups, &evaluated);
        let pdf = probe_pdf();
        let integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
            .expect("integrand");

        let cuts = groups.groups()[0].cuts();
        let masses = groups.groups()[0].final_masses();
        // Two energies, so the `1/(2ŝ)` flux and the `√ŝ` dependence of the map are
        // pinned as a shape and not only as one normalisation.
        for sqrt_shat in [200.0f64, 500.0] {
            let tau = sqrt_shat * sqrt_shat / (SQRT_S_HAD * SQRT_S_HAD);
            let u0 = 1.0 - tau.ln() / integ.tau_min().ln();
            let y_max = -0.5 * tau.ln();
            let jac = (1.0 / integ.tau_min()).ln() * 2.0 * y_max;
            let x = tau.sqrt();

            let seed = 0x5115_0001 + sqrt_shat as u64;
            let [hadronic, hadronic_err] = inner_integral(&integ, [u0, 0.5], 3000, seed);

            let (mut partonic, mut partonic_var) = (0.0, 0.0);
            for (g, amp) in groups.groups().iter().zip(&amps) {
                let mut fixed = FixedBeamIntegrand::new(
                    vec![amp],
                    cuts,
                    sqrt_shat,
                    masses.clone(),
                    g.spin_color_average(),
                );
                fixed.use_multichannel(g.diagrams(), &evaluated, 3000, 4, 0xA55E_7000);
                let (sigma, err) = fixed.integrate(20_000, 5, seed + 1);
                let [direct, mirror] = g.luminosity(&pdf, x, x, [MU_F, MU_F]);
                // Both orderings weight the same partonic cross section: `R` is a
                // rotation about the beam-perpendicular x axis, so it preserves the
                // measure and every observable this filter cuts on.
                let lum = direct + mirror;
                partonic += lum * sigma / GEV2_TO_PB;
                partonic_var += (lum * err / GEV2_TO_PB).powi(2);
            }
            let expected = jac * partonic;
            let expected_err = jac * partonic_var.sqrt();
            let combined = (hadronic_err * hadronic_err + expected_err * expected_err).sqrt();
            let pull = (hadronic - expected) / combined;
            let rel = (hadronic - expected).abs() / expected;
            eprintln!(
                "sqrt(shat) = {sqrt_shat}: hadronic {hadronic:.6e} ± {hadronic_err:.1e}, \
                 luminosity × partonic {expected:.6e} ± {expected_err:.1e}, rel {rel:.3e}, \
                 pull {pull:.2}"
            );
            assert!(
                hadronic > 0.0 && expected > 0.0,
                "one side vanished at {sqrt_shat}: {hadronic} vs {expected}"
            );
            // Both bounds are above a measured four-seed sweep (worst 1.31% and 2.73
            // combined errors) and far below anything a lost factor could produce: the
            // smallest normalisation this test exists to catch is a factor of two.
            // Raising the partonic budget five-fold brings the same seeds to
            // 0.04%-0.46%, so the residual is where the two Monte Carlos have
            // converged to, not a disagreement between them.
            assert!(
                rel < 0.03,
                "the two sides differ by {rel:.3e} at {sqrt_shat}"
            );
            assert!(
                pull.abs() < 4.0,
                "the two sides differ by {pull:.2} combined standard errors at {sqrt_shat}"
            );
        }
    }

    /// The same fixed-energy oracle on a decomposition whose groups need *different*
    /// identical-particle factors: `u ū → g g` at `1/2` beside `d d̄ → u ū` at `1`,
    /// with one outgoing mass list and one cut filter, so nothing but the outgoing
    /// flavours separates them.
    ///
    /// The two groups are measured one at a time, by a probe distribution that
    /// supports only one of them — the other's luminosity is then exactly zero while
    /// its channels keep sampling, so the mixture is the mixed one throughout and the
    /// measurement is still of a group inside it. That isolation is what makes the
    /// test fire: had the other group's factor been used, the reference would be off
    /// by a factor of two rather than by a share of the total, and the assertion
    /// below states that distance explicitly.
    ///
    /// The partonic side computes each group's `σ̂` through [`FixedBeamIntegrand`],
    /// which reads the factor off that group's own amplitude, so the comparison is
    /// between two independent readings of the factor and not one reading checked
    /// against itself.
    #[test]
    fn at_fixed_energy_each_group_carries_its_own_symmetry_factor() {
        use crate::hadronic::FixedBeamIntegrand;

        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let card = llj_card();
        let mut sets = enumerate("u u~ > g g", &m);
        sets.extend(enumerate("d d~ > u u~", &m));
        let groups =
            derive_flavor_groups(sets, &m, &evaluated, &card).expect("mixed-factor groups");
        assert_eq!(groups.groups().len(), 2);
        let amps = bind_all(&groups, &evaluated);
        let factors: Vec<f64> = groups
            .groups()
            .iter()
            .map(|g| g.members()[0].symmetry_factor())
            .collect();
        assert_eq!(factors.iter().copied().fold(0.0, f64::max), 1.0);
        assert_eq!(factors.iter().copied().fold(1.0, f64::min), 0.5);

        let cuts = groups.groups()[0].cuts();
        let masses = groups.groups()[0].final_masses();
        let sqrt_shat = 500.0f64;
        let tau = sqrt_shat * sqrt_shat / (SQRT_S_HAD * SQRT_S_HAD);
        let x = tau.sqrt();

        for (i, alive) in [vec![2, -2], vec![1, -1]].into_iter().enumerate() {
            let pdf = probe_pdf_restricted(&alive);
            let integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
                .expect("integrand");
            let u0 = 1.0 - tau.ln() / integ.tau_min().ln();
            let y_max = -0.5 * tau.ln();
            let jac = (1.0 / integ.tau_min()).ln() * 2.0 * y_max;

            let seed = 0x5111_0002 + i as u64;
            let [hadronic, hadronic_err] = inner_integral(&integ, [u0, 0.5], 3000, seed);

            let (mut partonic, mut partonic_var) = (0.0, 0.0);
            let mut live = Vec::new();
            for (gi, (g, amp)) in groups.groups().iter().zip(&amps).enumerate() {
                let [direct, mirror] = g.luminosity(&pdf, x, x, [MU_F, MU_F]);
                let lum = direct + mirror;
                if lum == 0.0 {
                    continue;
                }
                live.push(gi);
                let mut fixed = FixedBeamIntegrand::new(
                    vec![amp],
                    cuts,
                    sqrt_shat,
                    masses.clone(),
                    g.spin_color_average(),
                );
                fixed.use_multichannel(g.diagrams(), &evaluated, 3000, 4, 0xA55E_7000);
                let (sigma, err) = fixed.integrate(20_000, 5, seed + 1);
                partonic += lum * sigma / GEV2_TO_PB;
                partonic_var += (lum * err / GEV2_TO_PB).powi(2);
            }
            assert_eq!(live.len(), 1, "the probe distribution lit up {live:?}");
            let other = factors[1 - live[0]];
            let mine = factors[live[0]];

            let expected = jac * partonic;
            let expected_err = jac * partonic_var.sqrt();
            let combined = (hadronic_err * hadronic_err + expected_err * expected_err).sqrt();
            let rel = (hadronic - expected).abs() / expected;
            let pull = (hadronic - expected) / combined;
            let wrong = expected * other / mine;
            let control = (wrong - expected).abs() / expected;
            eprintln!(
                "group {} (factor {mine}): hadronic {hadronic:.6e} ± {hadronic_err:.1e}, \
                 luminosity × partonic {expected:.6e} ± {expected_err:.1e}, rel {rel:.3e}, \
                 pull {pull:.2}; the other group's factor would sit {control:.3} away",
                live[0]
            );
            assert!(
                control > 0.4,
                "the two factors are only {control:.3} apart here"
            );
            assert!(rel < 0.03, "the two sides differ by {rel:.3e}");
            assert!(
                pull.abs() < 4.0,
                "the two sides differ by {pull:.2} combined standard errors"
            );
        }
    }

    /// The mixture estimator's mean and standard error over `n` draws — the quantity
    /// the selection weights exist to make cheap.
    fn mixture_estimate(integ: &ProtonIntegrand<'_>, n: usize, seed: u64) -> [f64; 2] {
        let mut stream = SubStream::from_stream(seed, 5);
        let (mut sum, mut sum2) = (0.0, 0.0);
        for _ in 0..n {
            let u = stream.uniforms::<f64>(integ.vegas_ndim());
            let v = integ.value(&u);
            sum += v;
            sum2 += v * v;
        }
        let mean = sum / n as f64;
        [
            mean,
            ((sum2 / n as f64 - mean * mean).max(0.0) / n as f64).sqrt(),
        ]
    }

    /// The channel weights adapt jointly over the whole `(group, diagram)` space, and
    /// the adaptation moves only the sampling: the same integral, at lower variance.
    ///
    /// Both sides are measured on the *mixture* estimator at one fixed sample count,
    /// which is what the Kleiss-Pittau reallocation minimises. A per-channel VEGAS
    /// comparison would not answer the same question: the budget split `αⱼ · neval`
    /// and the `MIN_CHANNEL_NEVAL` floor make the two runs draw different numbers of
    /// points, and each grid then refines a different conditional density.
    #[test]
    fn joint_alpha_adaptation_lowers_the_variance_without_moving_the_integral() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let card = llj_card();
        let groups = derive_flavor_groups(enumerate(LLJ, &m), &m, &evaluated, &card)
            .expect("flavour groups");
        let amps = bind_all(&groups, &evaluated);
        let pdf = probe_pdf();
        let mut integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
            .expect("integrand");

        let [uniform, uniform_err] = mixture_estimate(&integ, 30_000, 0x9E77_0001);
        let report = integ.adapt_alphas(0xADAB_7000, 4000, 6, 0.5);
        let [adapted, adapted_err] = mixture_estimate(&integ, 30_000, 0x9E77_0001);

        let alphas = integ.channel_alphas();
        assert_eq!(alphas.len(), 24);
        assert_eq!(alphas.len(), report.variance_shares.len());
        let spread = alphas.iter().fold(0.0f64, |a, &x| a.max(x))
            / alphas.iter().fold(f64::INFINITY, |a, &x| a.min(x));
        eprintln!(
            "alpha spread {spread:.1}x after {} surveys; mixture estimate \
             {uniform:.5e} ± {uniform_err:.2e} (uniform) vs {adapted:.5e} ± {adapted_err:.2e} \
             (adapted), error ratio {:.2}",
            report.trajectory.len() - 1,
            adapted_err / uniform_err
        );
        assert!(
            spread > 10.0,
            "the weights stayed within {spread:.2}x of each other, so this run does not \
             show the adaptation doing anything"
        );
        assert!(
            (alphas.iter().sum::<f64>() - 1.0).abs() < 1e-12,
            "the selection weights stopped being a distribution"
        );
        let combined = (uniform_err * uniform_err + adapted_err * adapted_err).sqrt();
        let pull = (adapted - uniform) / combined;
        assert!(
            pull.abs() < 4.0,
            "the adaptation moved the integral by {pull:.2} combined standard errors: \
             {uniform} vs {adapted}"
        );
        assert!(
            adapted_err < uniform_err,
            "the adapted mixture is noisier ({adapted_err:.2e}) than the uniform one \
             ({uniform_err:.2e})"
        );
    }
    /// A helper for the event-side tests: the integrand, its groups and their
    /// amplitudes, kept alive together.
    fn llj_setup(m: &Arc<UFOModel>, evaluated: &EvaluatedModel) -> (FlavorGroups, PdfMember) {
        let card = llj_card();
        let groups =
            derive_flavor_groups(enumerate(LLJ, m), m, evaluated, &card).expect("flavour groups");
        (groups, probe_pdf())
    }

    /// The first cut-passing point a fixed stream produces that `wanted` accepts,
    /// with the coordinates it was drawn at.
    fn find_event(
        integ: &ProtonIntegrand<'_>,
        seed: u64,
        wanted: impl Fn(&ProtonEvent) -> bool,
    ) -> (usize, Vec<f64>, ProtonEvent) {
        let mut stream = SubStream::from_stream(seed, 11);
        for trial in 0..40_000 {
            let u = stream.uniforms::<f64>(integ.point_ndim());
            let channel = trial % integ.channel_count();
            if let Some(event) = integ.event_in_channel(channel, &u) {
                if wanted(&event) {
                    return (channel, u, event);
                }
            }
        }
        panic!("no point inside the cuts met the condition");
    }

    /// An accepted point has to come back as the point its weight was taken at, in
    /// both frames: the trial loop runs on `value_in_channel` and only the kept
    /// points are reconstructed, so a reconstruction at a different point would
    /// write events that carry another point's weight.
    #[test]
    fn an_accepted_point_reconstructs_the_value_it_was_drawn_at() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let (groups, pdf) = llj_setup(&m, &evaluated);
        let amps = bind_all(&groups, &evaluated);
        let integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
            .expect("integrand");

        let e_beam = SQRT_S_HAD / 2.0;
        let mut stream = SubStream::from_stream(0x4E4E_7000, 3);
        let (mut kept, mut empty, mut worst_balance, mut worst_shat) = (0, 0, 0.0f64, 0.0f64);
        for trial in 0..400 {
            let u = stream.uniforms::<f64>(integ.point_ndim());
            let channel = trial % integ.channel_count();
            let value = integ.value_in_channel(channel, &u);
            let Some(event) = integ.event_in_channel(channel, &u) else {
                assert_eq!(value, 0.0, "a point with weight was not reconstructed");
                empty += 1;
                continue;
            };
            kept += 1;
            assert_eq!(
                event.weight.to_bits(),
                value.to_bits(),
                "the reconstructed point carries a different weight"
            );

            // The beams are the collider's, at this point's own momentum fractions,
            // head-on along the axis.
            for (beam, (x, sign)) in event.lab[..2]
                .iter()
                .zip([(event.x[0], 1.0), (event.x[1], -1.0)])
            {
                assert!((beam.e() - x * e_beam).abs() < 1e-9 * e_beam);
                assert!((beam.pz() - sign * x * e_beam).abs() < 1e-9 * e_beam);
                assert_eq!([beam.px(), beam.py()], [0.0, 0.0]);
            }
            for component in 0..4 {
                let of = |p: &V| [p.e(), p.px(), p.py(), p.pz()][component];
                let balance: f64 = event.lab[..2].iter().map(of).sum::<f64>()
                    - event.lab[2..].iter().map(of).sum::<f64>();
                worst_balance = worst_balance.max(balance.abs() / (event.x[0] * e_beam));
            }
            // The other frame is the partonic CM of the same event: back-to-back
            // beams carrying the whole of `ŝ = x₁ x₂ s`.
            let shat = 4.0 * event.cm[0].e() * event.cm[0].e();
            worst_shat = worst_shat
                .max((shat / (event.x[0] * event.x[1] * SQRT_S_HAD * SQRT_S_HAD) - 1.0).abs());
            assert!((event.cm[0].pz() + event.cm[1].pz()).abs() < 1e-9 * event.cm[0].e());
        }
        eprintln!(
            "{kept} reconstructed points ({empty} carrying no weight); lab balance \
             {worst_balance:.2e} of a beam, partonic invariant {worst_shat:.2e}"
        );
        assert!(kept > 30 && empty > 0, "{kept} kept, {empty} empty");
        assert!(worst_balance < 1e-12 && worst_shat < 1e-12);
    }

    /// The per-event step with no fixed-beam counterpart: **which concrete flavour**
    /// inside a group an event is labelled with.
    ///
    /// The members of a group share their matrix element exactly, so at a fixed beam
    /// ordering the whole of what separates them is their parton luminosity at the
    /// event's own `(x₁, x₂)` — and that is what the draw has to follow. The oracle
    /// forms each member's `x·f` product straight from the parton distribution, so it
    /// shares no code with [`FlavorGroup::member_luminosity`].
    ///
    /// The uniform is *swept* rather than sampled, so the measured shares are the
    /// rule's own and carry no Monte Carlo error: the only residual is the sweep's
    /// grid, and every margin below is quoted in units of it.
    ///
    /// What it cannot see: which of `x₁`, `x₂` belongs to which beam is pinned only as
    /// far as the probe distribution separates them, so the margin against the
    /// exchanged assignment is measured and asserted rather than assumed. It also says
    /// nothing about *realised* frequencies over a generated sample against MadGraph's
    /// — that is the deferred validation pass's content.
    #[test]
    fn a_members_share_of_the_draw_is_its_share_of_the_parton_luminosity() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let (groups, _) = llj_setup(&m, &evaluated);
        let pdf = beam_asymmetric_pdf();
        let amps = bind_all(&groups, &evaluated);
        let integ = ProtonIntegrand::new(&groups, &amps, &evaluated, &pdf, SQRT_S_HAD, MU_F)
            .expect("integrand");
        // A point well off central rapidity, so the two beams sit at different
        // momentum fractions and the shares can tell them apart at all.
        let (_, _, event) = find_event(&integ, 0x51A2_E000, |e| e.x[0] > 5.0 * e.x[1]);
        let q2 = [MU_F * MU_F; 2];
        const SWEEP: usize = 4_001;

        // A `u₀` landing in each group, so the conditional draw below is taken at a
        // known group. The group index rises with `u₀` (it is read off a cumulative
        // distribution), so each group's interval is found by bisection rather than
        // by a sweep fine enough to land inside the smallest of them.
        let group_at = |u0: f64| {
            integ
                .select_event(&event, [u0, 0.5, 0.5, 0.5, 0.5])
                .expect("an accepted point has labels")
                .group
        };
        let mut entry: Vec<Option<f64>> = Vec::with_capacity(groups.groups().len());
        let mut floor = 0.0f64;
        for gi in 0..groups.groups().len() {
            if group_at(floor) != gi {
                entry.push(None);
                continue;
            }
            let (mut inside, mut outside) = (floor, 1.0);
            for _ in 0..60 {
                let mid = 0.5 * (inside + outside);
                if group_at(mid) <= gi {
                    inside = mid;
                } else {
                    outside = mid;
                }
            }
            entry.push(Some(0.5 * (floor + inside)));
            floor = outside;
        }

        let (mut worst, mut worst_uniform, mut swap_margin, mut orderings) =
            (0.0f64, f64::INFINITY, 0.0f64, [0usize; 2]);
        for (gi, g) in groups.groups().iter().enumerate() {
            let Some(u0) = entry[gi] else {
                continue;
            };
            let mut counts = vec![[0usize; 2]; g.members().len()];
            for k in 0..SWEEP {
                let u1 = (k as f64 + 0.5) / SWEEP as f64;
                let s = integ
                    .select_event(&event, [u0, u1, 0.5, 0.5, 0.5])
                    .expect("an accepted point has labels");
                assert_eq!(s.group, gi, "the group draw moved with the flavour draw");
                counts[s.member][usize::from(s.ordering == BeamOrdering::Exchanged)] += 1;
            }

            // The luminosity share, formed from the parton distribution directly, and
            // the same shares with the two beams exchanged — the assignment this test
            // has to be able to rule out.
            let share = |swapped: bool, ordering: usize| -> Vec<f64> {
                let (xa, xb) = if swapped {
                    (event.x[1], event.x[0])
                } else {
                    (event.x[0], event.x[1])
                };
                g.members()
                    .iter()
                    .map(|s| {
                        let [a, b] = if ordering == 0 {
                            s.incoming
                        } else {
                            [s.incoming[1], s.incoming[0]]
                        };
                        if ordering == 1 && s.incoming[0] == s.incoming[1] {
                            return 0.0;
                        }
                        pdf.xfx_q2(a, xa, q2[0]) * pdf.xfx_q2(b, xb, q2[1])
                    })
                    .collect()
            };

            for ordering in 0..2 {
                let drawn: usize = counts.iter().map(|c| c[ordering]).sum();
                if drawn == 0 {
                    continue;
                }
                orderings[ordering] += 1;
                let expected = share(false, ordering);
                let total: f64 = expected.iter().sum();
                let swapped = share(true, ordering);
                let swapped_total: f64 = swapped.iter().sum();
                let uniform = 1.0 / expected.len() as f64;
                // The sweep resolves each cell to one step, and the counts are
                // renormalised within their ordering, so a cell's share is known to
                // about `(1 + cells)/drawn`. Everything below is measured against that
                // resolution rather than against a flat number, so an ordering the
                // luminosities make rare is held to its own accuracy.
                let resolution = (1.0 + expected.len() as f64) / drawn as f64;
                for (i, want) in expected.iter().map(|w| w / total).enumerate() {
                    let got = counts[i][ordering] as f64 / drawn as f64;
                    worst = worst.max((got - want).abs() / resolution);
                    worst_uniform = worst_uniform.min((want - uniform).abs() / resolution);
                    swap_margin =
                        swap_margin.max((swapped[i] / swapped_total - want).abs() / resolution);
                }
            }
        }
        eprintln!(
            "flavour draw: worst deviation from the luminosity share {worst:.2} of the sweep's \
             own resolution; a uniform draw would be off by at least {worst_uniform:.1} of it, an \
             exchanged-beam one by up to {swap_margin:.1}; {} groups drew both orderings",
            orderings[1]
        );
        assert!(
            worst < 1.0,
            "the flavour draw is not the luminosity share: {worst:.2} resolutions off"
        );
        assert_eq!(
            orderings,
            [groups.groups().len(); 2],
            "every group must be reached and must draw both beam orderings"
        );
        assert!(
            worst_uniform > 5.0,
            "the probe luminosities are within {worst_uniform:.2} resolutions of uniform, so this \
             test could not tell the rule from a uniform draw"
        );
        assert!(
            swap_margin > 5.0,
            "exchanging the two beams moves the shares by at most {swap_margin:.2} resolutions, \
             so this test cannot see which `x` belongs to which beam"
        );
    }

    /// The exchanged beam ordering carries the *same* per-leg record fields with the
    /// two incoming legs traded, and nothing else moved.
    ///
    /// The claim is that the mirror identity extends from `|M|²` to the accumulators an
    /// event record is filled from: the rotation `R` maps each beam momentum onto the
    /// other's, so the representative's leg 0 describes the event's *second* beam.
    /// That is a convention claim about helicity labels and colour lines, which no
    /// cross-section-level check can see, so it is pinned against the mirrored
    /// subprocess compiled from **its own** proc card.
    ///
    /// What it cannot see: `p p → ℓℓj` has one colour flow per subprocess, so the flow
    /// *index* is unpermutable here and only the per-leg tags of that one flow are
    /// compared. A process whose mirrored basis reordered its flows would need the flow
    /// map pinned too.
    #[test]
    fn an_exchanged_ordering_relabels_the_beams_of_every_per_leg_field() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let (groups, _) = llj_setup(&m, &evaluated);
        let points = fresh_points(&groups.groups()[0].final_masses());

        let (mut worst_hel, mut worst_jamp, mut flows) = (0.0f64, 0.0f64, 0);
        for g in groups.groups() {
            let set = g.diagram_set();
            let swapped = format!(
                "{} {} > {} QCD=2 QED=2",
                set.particles_in[1],
                set.particles_in[0],
                set.particles_out.join(" ")
            );
            let mirror_set = enumerate(&swapped, &m)
                .into_iter()
                .find(|s| !s.diagrams.is_empty())
                .expect("the mirrored ordering enumerates");
            let mirror_eval = compile_class(&mirror_set, &m, &evaluated).expect("mirror compiles");

            // The permutation the record layer applies, taken from the group rather
            // than rebuilt, so this measures the mapping the generator uses.
            let member = g
                .members()
                .iter()
                .position(Subprocess::has_mirror)
                .expect("a mirrored ordering exists");
            let (pdg, order) = g.event_legs(member, BeamOrdering::Exchanged);
            assert_eq!(
                pdg[..2],
                [
                    g.members()[member].incoming[1],
                    g.members()[member].incoming[0]
                ],
                "the exchanged ordering did not trade the beam flavours"
            );

            let rep = BoundAmplitude::<f64>::bind(g.evaluator(), &evaluated);
            let mut rep_scratch = rep.scratch_space();
            let mirror = BoundAmplitude::<f64>::bind(&mirror_eval, &evaluated);
            let mut mirror_scratch = mirror.scratch_space();

            // The colour lines of the exchanged ordering are the representative's on
            // permuted legs, and they must be the mirrored subprocess's own.
            let permuted = g
                .evaluator()
                .color_flow_tags()
                .permuted(&order)
                .expect("the beam exchange is a permutation");
            assert_eq!(
                permuted.n_flows(),
                mirror_eval.n_flows(),
                "{swapped} has a different number of colour flows"
            );
            for f in 0..permuted.n_flows() {
                assert_eq!(
                    connectivity(permuted.flow(f)),
                    connectivity(mirror_eval.color_flow_tags().flow(f)),
                    "{swapped}: flow {f}\u{2019}s colour lines are not the exchanged ordering\u{2019}s"
                );
                flows += 1;
            }

            let mut reflected = Vec::new();
            for k in &points {
                g.mirror_into(k, &mut reflected);
                let mut rep_hel = vec![0.0; g.evaluator().helicities().len()];
                let mut rep_jamp = vec![0.0; g.evaluator().n_flows()];
                rep.eval_hel_m2(&reflected, &mut rep_scratch, &mut rep_hel);
                rep.eval_jamp2(&reflected, &mut rep_scratch, &mut rep_jamp);

                let mut mirror_hel = vec![0.0; mirror_eval.helicities().len()];
                let mut mirror_jamp = vec![0.0; mirror_eval.n_flows()];
                mirror.eval_hel_m2(k, &mut mirror_scratch, &mut mirror_hel);
                mirror.eval_jamp2(k, &mut mirror_scratch, &mut mirror_jamp);

                let scale: f64 = mirror_hel.iter().sum();
                for (c, combination) in g.evaluator().helicities().iter().enumerate() {
                    let physical: Vec<i32> = order.iter().map(|&leg| combination[leg]).collect();
                    let target = mirror_eval
                        .helicities()
                        .iter()
                        .position(|h| *h == physical)
                        .map(|i| mirror_hel[i])
                        .unwrap_or(0.0);
                    worst_hel = worst_hel.max((rep_hel[c] - target).abs() / scale);
                }
                for (f, &jamp) in mirror_jamp.iter().enumerate() {
                    worst_jamp = worst_jamp.max((rep_jamp[f] - jamp).abs() / scale);
                }
            }
        }
        eprintln!(
            "exchanged ordering: per-helicity {worst_hel:.2e}, per-flow {worst_jamp:.2e} relative \
             to the summed |M|\u{b2}, over {flows} colour flows"
        );
        // The bound is the probe points\u{2019}, as for the `|M|\u{b2}` mirror identity: one of the
        // RAMBO draws is far enough off the light cone that two independently compiled
        // programs route the gauge-dependent parts differently.
        assert!(
            worst_hel < 1e-11,
            "per-helicity disagreement {worst_hel:.2e}"
        );
        assert!(worst_jamp < 1e-11, "per-flow disagreement {worst_jamp:.2e}");
    }

    /// How a member's colour reps relate to its group representative's.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum RepClass {
        /// Every leg carries the representative's own rep.
        Identity,
        /// Every leg carries the conjugate, and at least one actually differs.
        GlobalConjugate,
        /// Some legs conjugated and some not — no slot operation relates the tables.
        Crossing,
    }

    fn rep_class(representative: &[ColorRep], member: &[ColorRep]) -> RepClass {
        if representative == member {
            RepClass::Identity
        } else if representative
            .iter()
            .zip(member)
            .all(|(r, m)| r.anti() == *m)
        {
            RepClass::GlobalConjugate
        } else {
            RepClass::Crossing
        }
    }

    /// Every enumerated subprocess of `process`, compiled, keyed by its PDG
    /// assignment — the member's *own* amplitude, in the enumeration's own leg order.
    fn compiled_by_assignment(
        process: &str,
        m: &Arc<UFOModel>,
        evaluated: &EvaluatedModel,
    ) -> std::collections::BTreeMap<Vec<i32>, AmplitudeEvaluator> {
        let mut out = std::collections::BTreeMap::new();
        for set in enumerate(process, m).iter().filter(|s| !s.diagrams.is_empty()) {
            let evaluator = compile_class(set, m, evaluated).expect("subprocess compiles");
            let legs = process_external_legs(&evaluator, m, evaluated);
            out.insert(legs.iter().map(|l| l.pdg).collect(), evaluator);
        }
        out
    }

    /// Every member of every `p p → j j` group carries **its own** subprocess's
    /// colour flows, under both beam orderings.
    ///
    /// The record layer serves a whole group off one compiled amplitude, and the
    /// colour flows are the one thing that cannot come from it: two subprocesses can
    /// share a matrix element, a mass list and a colour-factor matrix while routing
    /// their colour lines between different pairs of legs. So each member's table is
    /// its own, put into the representative's flow indexing once by a permutation
    /// matched on the colour algebra rather than on numbers. This asserts the whole
    /// of that against the member compiled from its own enumerated diagrams.
    ///
    /// Then, per class, the statement that class alone makes. The sharpest is the
    /// crossing class's: its members' tags are **neither** the representative's nor
    /// the representative's conjugate, at any index — so no slot operation, global or
    /// per-leg, could have produced them, and a table transformed from the
    /// representative's cannot serve them. `conjugated` survives as the independent
    /// oracle for the global-conjugate class, so that class is checked twice by two
    /// derivations with different failure modes.
    ///
    /// What it cannot see: an error shared by the member's compilation and the
    /// representative's — the 73 `leshouche.inc` trials of `color_flow_tags_oracle`
    /// are what exclude that, and the two have no common blind spot.
    #[test]
    fn every_member_carries_its_own_subprocesss_colour_flows() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let groups = derive(JJ, &m, &evaluated);
        let owned = compiled_by_assignment(JJ, &m, &evaluated);

        let mut counts = [0usize; 3];
        let (mut non_identity_pi, mut identity_pi_rep_differs) = (0usize, 0usize);
        let mut checked = 0usize;
        for group in groups.groups() {
            let rep_colors: Vec<ColorRep> = group
                .evaluator()
                .external_colors()
                .iter()
                .map(|l| l.rep)
                .collect();
            let rep_tags = group.evaluator().color_flow_tags();
            let rep_conj = rep_tags.conjugated();
            let base = SubprocessRecord::new(group.evaluator(), &m, &evaluated).expect("record");

            for (i, member) in group.members().iter().enumerate() {
                let direct: Vec<i32> = member
                    .incoming
                    .iter()
                    .chain(member.outgoing.iter())
                    .copied()
                    .collect();
                let own = &owned[&direct];
                let pi = &member.flow_permutation;
                let class = rep_class(&rep_colors, &member.colors);
                counts[class as usize] += 1;
                if pi.iter().enumerate().any(|(f, &g)| f != g) {
                    non_identity_pi += 1;
                } else if class != RepClass::Identity {
                    identity_pi_rep_differs += 1;
                }

                for ordering in [BeamOrdering::Direct, BeamOrdering::Exchanged] {
                    if ordering == BeamOrdering::Exchanged && !member.has_mirror() {
                        continue;
                    }
                    let (pdg, order) = group.event_legs(i, ordering);
                    let legs = group.event_leg_colors(i, ordering);
                    let record = base
                        .relabelled(&order, &pdg, &legs, &member.flows)
                        .unwrap_or_else(|e| panic!("{pdg:?} {ordering:?}: {e}"));

                    for f in 0..record.n_flows() {
                        // The record's table is the member's own, at `π(f)`, with the
                        // beam exchange applied to the legs and nothing else.
                        let want = own
                            .color_flow_tags()
                            .flow(pi[f])
                            .iter()
                            .enumerate()
                            .map(|(leg, _)| own.color_flow_tags().flow(pi[f])[order[leg]])
                            .collect::<Vec<_>>();
                        assert_eq!(
                            connectivity(record.flows().flow(f)),
                            connectivity(&want),
                            "{pdg:?} {ordering:?}: record flow {f} is not the member's own \
                             flow {} on these legs",
                            pi[f]
                        );
                        checked += 1;
                    }
                }

                // Per class, on the direct ordering, where the leg order is shared.
                for f in 0..rep_tags.n_flows() {
                    let ours = connectivity(member.flows.flow(f));
                    match class {
                        RepClass::Identity => {
                            assert_eq!(pi[f], f, "{direct:?}: identity class with a permuted flow");
                            assert_eq!(
                                ours,
                                connectivity(rep_tags.flow(f)),
                                "{direct:?}: identity class disagrees with the representative"
                            );
                        }
                        RepClass::GlobalConjugate => {
                            assert_eq!(
                                ours,
                                connectivity(rep_conj.flow(f)),
                                "{direct:?}: conjugate class is not the representative's \
                                 conjugated table at flow {f}"
                            );
                        }
                        RepClass::Crossing => {
                            let matches_rep = (0..rep_tags.n_flows())
                                .any(|g| ours == connectivity(rep_tags.flow(g)));
                            let matches_conj = (0..rep_conj.n_flows())
                                .any(|g| ours == connectivity(rep_conj.flow(g)));
                            assert!(
                                !matches_rep && !matches_conj,
                                "{direct:?}: a crossing member's flow {f} is the representative's \
                                 table (or its conjugate) at some index, so a slot transformation \
                                 would have sufficed after all"
                            );
                        }
                    }
                }
            }
        }
        eprintln!(
            "per-member colour flows: {checked} (member, ordering, flow) tables checked; \
             classes identity/conjugate/crossing = {}/{}/{}; {non_identity_pi} members carry a \
             non-identity flow permutation, {identity_pi_rep_differs} carry the identity while \
             differing from the representative in rep",
            counts[0], counts[1], counts[2]
        );
        assert!(counts[0] > 0, "no identity-class member");
        assert!(counts[1] > 0, "no global-conjugate member");
        assert!(counts[2] > 0, "no crossing-class member");
        assert!(non_identity_pi > 0, "every flow permutation was the identity");
        assert!(
            identity_pi_rep_differs > 0,
            "every rep-differing member permuted its flows, so this cannot tell the permutation \
             from the rep relation"
        );
    }

    /// The flow permutation carries the leading-colour mask, elementwise.
    ///
    /// `ICOLAMP` is reused across a group unchanged: the configuration draw, the mask
    /// and the flow draw all happen in the representative's indexing, and only the
    /// final label is the member's. That is sound only if `π` maps a diagram's
    /// reached flows onto the same diagram's reached flows, which it does by
    /// construction — `π` is matched on `(diagram, chain, nc_power)` and `reached` is
    /// a predicate on exactly those. Asserted rather than argued, because if it ever
    /// fails the mask has to be translated too and the labels would otherwise be
    /// right at the wrong frequencies.
    ///
    /// What it cannot see: whether the mask is the right mask — that is
    /// `validate_unweighting`'s job.
    #[test]
    fn the_flow_permutation_carries_the_leading_colour_mask() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let groups = derive(JJ, &m, &evaluated);
        let owned = compiled_by_assignment(JJ, &m, &evaluated);

        let (mut rows, mut restrictive, mut permuted) = (0usize, 0usize, 0usize);
        for group in groups.groups() {
            let rep = group.evaluator().leading_color_flows();
            for member in group.members() {
                let direct: Vec<i32> = member
                    .incoming
                    .iter()
                    .chain(member.outgoing.iter())
                    .copied()
                    .collect();
                let theirs = owned[&direct].leading_color_flows();
                let pi = &member.flow_permutation;
                if pi.iter().enumerate().any(|(f, &g)| f != g) {
                    permuted += 1;
                }
                assert_eq!(rep.n_diagrams(), theirs.n_diagrams(), "{direct:?}: diagram count");
                for d in 0..rep.n_diagrams() {
                    let (ours, other) = (rep.reached_by(d), theirs.reached_by(d));
                    assert_eq!(ours.len(), other.len(), "{direct:?}: diagram {d} row width");
                    if ours.iter().any(|&r| !r) {
                        restrictive += 1;
                    }
                    for (f, &reached) in ours.iter().enumerate() {
                        assert_eq!(
                            reached, other[pi[f]],
                            "{direct:?}: diagram {d} reaches flow {f} = {reached} in the \
                             representative but {} at the member's flow {}",
                            other[pi[f]], pi[f]
                        );
                    }
                    rows += 1;
                }
            }
        }
        eprintln!(
            "ICOLAMP invariance: {rows} diagram rows compared, {restrictive} of them actually \
             restricting, over {permuted} members with a non-identity flow permutation"
        );
        assert!(restrictive > 0, "every mask row was all-true, so the identity is trivial");
        assert!(permuted > 0, "every flow permutation was the identity");
    }

    /// The exchanged beam ordering is a leg permutation of the direct one, and
    /// nothing else.
    ///
    /// The mirrored term evaluates the shared amplitude at the rotated argument, so
    /// everything the direct record says about its leg 0 describes the event's second
    /// beam. This pins that at the record layer without compiling a beam-swapped
    /// process — a second compilation in a non-canonical leg order is not a
    /// trustworthy oracle at `NCOLOR > 1`, and it is not needed: the exchange is a
    /// permutation applied to an already-resolved table.
    ///
    /// What it cannot see: whether the direct ordering's table is right at all —
    /// that is [`every_member_carries_its_own_subprocesss_colour_flows`]'s job.
    #[test]
    fn the_exchanged_ordering_is_a_leg_permutation_of_the_direct_one() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let groups = derive(JJ, &m, &evaluated);

        let mut compared = 0usize;
        for group in groups.groups() {
            let base = SubprocessRecord::new(group.evaluator(), &m, &evaluated).expect("record");
            for (i, member) in group.members().iter().enumerate() {
                if !member.has_mirror() {
                    continue;
                }
                let (pdg_d, order_d) = group.event_legs(i, BeamOrdering::Direct);
                let legs_d = group.event_leg_colors(i, BeamOrdering::Direct);
                let direct = base
                    .relabelled(&order_d, &pdg_d, &legs_d, &member.flows)
                    .expect("direct record");
                let (pdg_x, order_x) = group.event_legs(i, BeamOrdering::Exchanged);
                let legs_x = group.event_leg_colors(i, BeamOrdering::Exchanged);
                let exchanged = base
                    .relabelled(&order_x, &pdg_x, &legs_x, &member.flows)
                    .expect("exchanged record");

                let mut swap: Vec<usize> = (0..pdg_d.len()).collect();
                swap.swap(0, 1);
                assert_eq!(
                    exchanged.flows(),
                    &direct.flows().permuted(&swap).expect("beam swap"),
                    "{pdg_d:?}: the exchanged tags are not the direct ones with the beams traded"
                );
                assert_eq!(
                    exchanged.legs(),
                    {
                        let mut want = legs_d.clone();
                        want.swap(0, 1);
                        // The `incoming` flag is positional, so it does not travel.
                        for (leg, l) in want.iter_mut().enumerate() {
                            l.incoming = leg < group.n_in();
                        }
                        &want.clone()[..]
                    },
                    "{pdg_d:?}: the exchanged leg reps are not the direct ones with the beams \
                     traded"
                );
                assert_eq!(exchanged.pdg()[0], pdg_d[1]);
                assert_eq!(exchanged.pdg()[1], pdg_d[0]);
                compared += 1;
            }
        }
        eprintln!("beam exchange: {compared} mirrored members compared against their direct record");
        assert!(compared > 0, "no member carried a mirrored ordering");
    }

    /// The flow fingerprint identifies a flow uniquely, and the permutation it picks
    /// is the one the amplitudes agree with.
    ///
    /// The pairing is a structural decision — which diagram, through which colour
    /// chain, lands on the flow at which power of `Nc` — and this is its numeric
    /// cross-check, never its basis. `JAMP2` cannot be the decision: `g g > g g`'s
    /// colour orderings come in reversal pairs whose `|JAMP|²` agree *exactly* at
    /// every point, so a squared oracle provably cannot separate them. Where such a
    /// block exists the check weakens to "the block maps onto a block of equal
    /// multiset", which is all any numeric oracle can say there.
    ///
    /// Distinctness is required of a basis only where that basis is paired against a
    /// *different* subprocess's. A group whose only member is its representative
    /// pairs a basis with itself, where the identity is the answer by construction;
    /// `g g > g g` is that case here, and its reversal-degenerate flows are exactly
    /// the block the JAMP2 rule above covers.
    #[test]
    fn the_flow_fingerprint_identifies_a_flow_uniquely() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let (mut bases, mut degenerate_blocks, mut asserted) = (0usize, 0usize, 0usize);
        let mut worst = 0.0f64;
        let mut exempt: Vec<String> = Vec::new();

        for process in [JJ, LLJ] {
            let groups = derive(process, &m, &evaluated);
            let owned = compiled_by_assignment(process, &m, &evaluated);
            // A basis has to be pairwise distinct only where it is actually paired
            // against a *different* subprocess's, which is every basis in a group of
            // more than one member. A single-member group pairs its representative
            // with itself, where the identity is the answer by construction and no
            // fingerprint is consulted; `g g > g g`'s three reversal-degenerate pairs
            // are the one such basis here, and the block is covered instead by the
            // JAMP2 degeneracy rule below.
            for group in groups.groups() {
                if group.members().len() < 2 {
                    exempt.push(format!("{:?}", group.members()[0].incoming));
                    continue;
                }
                for member in group.members() {
                    let direct: Vec<i32> = member
                        .incoming
                        .iter()
                        .chain(member.outgoing.iter())
                        .copied()
                        .collect();
                    let keys = owned[&direct].flow_fingerprints();
                    for a in 0..keys.len() {
                        for b in a + 1..keys.len() {
                            assert_ne!(
                                keys[a], keys[b],
                                "{direct:?}: flows {a} and {b} share a fingerprint, and this \
                                 basis is paired against a different subprocess's, so the \
                                 pairing would not be unique"
                            );
                        }
                    }
                    bases += 1;
                }
            }

            for group in groups.groups() {
                let points = fresh_points(&group.final_masses());
                let rep = BoundAmplitude::<f64>::bind(group.evaluator(), &evaluated);
                let mut rep_scratch = rep.scratch_space();
                let rep_keys = group.evaluator().flow_fingerprints();
                let n_flows = group.evaluator().n_flows();

                for member in group.members() {
                    let direct: Vec<i32> = member
                        .incoming
                        .iter()
                        .chain(member.outgoing.iter())
                        .copied()
                        .collect();
                    let own = &owned[&direct];
                    let pi = &member.flow_permutation;
                    for f in 0..n_flows {
                        assert_eq!(
                            rep_keys[f],
                            own.flow_fingerprints()[pi[f]],
                            "{direct:?}: flow {f}'s fingerprint is not its image's"
                        );
                    }

                    let bound = BoundAmplitude::<f64>::bind(own, &evaluated);
                    let mut scratch = bound.scratch_space();
                    let mut rep_j = Vec::with_capacity(points.len());
                    let mut own_j = Vec::with_capacity(points.len());
                    let mut scales = Vec::with_capacity(points.len());
                    for k in &points {
                        let mut a = vec![0.0; n_flows];
                        let mut b = vec![0.0; n_flows];
                        rep.eval_jamp2(k, &mut rep_scratch, &mut a);
                        bound.eval_jamp2(k, &mut scratch, &mut b);
                        scales.push(b.iter().sum::<f64>().max(f64::MIN_POSITIVE));
                        rep_j.push(a);
                        own_j.push(b);
                    }
                    // A flow is "separated" when some probe point tells it apart from
                    // every other flow of the representative's basis by more than the
                    // stated margin; only then does a value comparison identify it.
                    for f in 0..n_flows {
                        let separated = (0..n_flows).filter(|&g| g != f).all(|g| {
                            (0..points.len()).any(|p| {
                                (rep_j[p][f] - rep_j[p][g]).abs() / scales[p] > JAMP_SEPARATION_MIN
                            })
                        });
                        if separated {
                            for p in 0..points.len() {
                                worst = worst
                                    .max((rep_j[p][f] - own_j[p][pi[f]]).abs() / scales[p]);
                            }
                            asserted += 1;
                        } else {
                            // The degenerate block: assert the multisets match instead.
                            degenerate_blocks += 1;
                            for p in 0..points.len() {
                                let mut ours: Vec<f64> = rep_j[p].clone();
                                let mut theirs: Vec<f64> =
                                    (0..n_flows).map(|g| own_j[p][pi[g]]).collect();
                                ours.sort_by(|a, b| a.partial_cmp(b).unwrap());
                                theirs.sort_by(|a, b| a.partial_cmp(b).unwrap());
                                for (a, b) in ours.iter().zip(&theirs) {
                                    worst = worst.max((a - b).abs() / scales[p]);
                                }
                            }
                        }
                    }
                }
            }
        }
        eprintln!(
            "flow fingerprints: {bases} paired bases with pairwise-distinct keys, {} \
             self-paired bases exempt ({exempt:?}); JAMP2 identity \
             asserted directly on {asserted} separated flows and by multiset on \
             {degenerate_blocks} degenerate ones, worst {worst:.2e} of the summed |M|\u{b2}",
            exempt.len()
        );
        assert!(asserted > 0, "no flow was separated enough to compare by value");
        assert!(worst < 1e-11, "JAMP2 disagreement {worst:.2e} across the permutation");
    }

    /// How far apart two flows must get, relative to the summed `|M|²` at some probe
    /// point, before a value comparison can identify one of them.
    const JAMP_SEPARATION_MIN: f64 = 1e-3;

    /// The colour lines a set of tags induces: the `(leg, slot)` endpoints sharing a
    /// label, with the label itself discarded because any consistent relabelling is
    /// the same event.
    fn connectivity(tags: &[[u32; 2]]) -> BTreeSet<Vec<(usize, usize)>> {
        let mut lines: std::collections::BTreeMap<u32, Vec<(usize, usize)>> =
            std::collections::BTreeMap::new();
        for (leg, pair) in tags.iter().enumerate() {
            for (slot, &label) in pair.iter().enumerate() {
                if label != 0 {
                    lines.entry(label).or_default().push((leg, slot));
                }
            }
        }
        lines.into_values().collect()
    }
}

/// Both integrands are shared, not copied, across a parallel integration: the
/// closure a VEGAS chunk runs holds `&integrand` and must be `Sync`. That is a
/// property of the *type*, so it is asserted on the type — a field that
/// reintroduced interior mutability outside the per-thread scratch would fail
/// here rather than at the call site that eventually needed it.
#[cfg(test)]
mod thread_safety {
    fn assert_sync<T: Sync>() {}

    #[test]
    fn the_integrands_are_shareable() {
        assert_sync::<super::ProtonIntegrand<'static>>();
        assert_sync::<crate::hadronic::FixedBeamIntegrand<'static>>();
    }
}
