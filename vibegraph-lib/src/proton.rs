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
//! # What this module does not decide
//!
//! The identical-particle symmetry factor is *asserted* to be one per subprocess
//! rather than carried, because a summed matrix element has no single owner for
//! it: the factor belongs to the final state, and a sum over subprocesses with
//! different final states would need it per term.

use thiserror::Error;

use crate::cuts::{CutError, Cuts, ExternalLeg};
use crate::diagrams::diagram::Diagram;
use crate::diagrams::DiagramSet;
use crate::hadronic::{
    compile_class, final_state_symmetry_factor, initial_spin_color_average, process_external_legs,
    HadronicError,
};
use crate::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
use crate::helas::repr::lorentz::LorentzVector;
use crate::pdf::PdfMember;
use crate::phasespace::rng::SubStream;
use crate::phasespace::{PhaseSpaceMap, RamboChannel};
use crate::runcard::RunCard;
use crate::ufo::{EvaluatedModel, UFOModel};

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
/// Measured worst separation over `p p → ℓℓj`: `0.69`.
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
}

impl Subprocess {
    /// Whether the two beams carry different partons, and so whether the mirrored
    /// ordering is a second physical initial state rather than the same one.
    pub fn has_mirror(&self) -> bool {
        self.incoming[0] != self.incoming[1]
    }
}

/// A set of subprocesses sharing one matrix element, one phase-space map and one
/// cut filter, differing only in the parton-distribution luminosity they carry.
pub struct FlavorGroup {
    representative: DiagramSet,
    evaluator: AmplitudeEvaluator,
    legs: Vec<ExternalLeg>,
    cuts: Cuts,
    members: Vec<Subprocess>,
    spin_color_avg: f64,
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
    pub fn luminosity(&self, pdf: &PdfMember, x1: f64, x2: f64, mu_f: [f64; 2]) -> [f64; 2] {
        let mut sums = [0.0; 2];
        for i in 0..self.members.len() {
            let m = self.member_luminosity(i, pdf, x1, x2, mu_f);
            sums[0] += m[0];
            sums[1] += m[1];
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
        let q2 = [mu_f[0] * mu_f[0], mu_f[1] * mu_f[1]];
        let [a, b] = self.members[member].incoming;
        let direct = pdf.xfx_q2(a, x1, q2[0]) * pdf.xfx_q2(b, x2, q2[1]);
        let mirror = if a == b {
            0.0
        } else {
            pdf.xfx_q2(b, x1, q2[0]) * pdf.xfx_q2(a, x2, q2[1])
        };
        [direct, mirror]
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
        "subprocess {process} carries an identical-particle symmetry factor {factor}, not 1; a \
         matrix element summed over subprocesses has no single owner for a final-state factor, \
         so each term would have to carry its own"
    )]
    IdenticalFinalState { process: String, factor: f64 },
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
        "the groups represented by {a} and {b} separate by only {rel:.3e} at their best-separated \
         probe point; a partition that close is rounding, not a coupling distinction"
    )]
    DegenerateGroups { a: String, b: String, rel: f64 },
}

/// Partonic energies the grouping probe measures at, spread over more than a
/// decade so a coincidence at one energy cannot survive.
///
/// Scaled by the outgoing pole masses so a heavy final state is probed above its
/// threshold, with a floor for the massless case.
fn probe_energies(final_masses: &[f64]) -> [f64; 3] {
    let base = final_masses.iter().sum::<f64>().max(100.0);
    [3.0 * base, 5.0 * base, 13.0 * base]
}

/// Partonic-CM probe points: massless beams along ±z and a flat RAMBO draw over
/// the outgoing legs, at each of [`probe_energies`].
fn probe_momenta(final_masses: &[f64], seed: u64) -> Vec<Vec<V>> {
    let mut points = Vec::new();
    for (i, sqrt_s) in probe_energies(final_masses).into_iter().enumerate() {
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
        let factor = final_state_symmetry_factor(&evaluator);
        if factor != 1.0 {
            return Err(ProtonError::IdenticalFinalState {
                process: process.clone(),
                factor,
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

    let points = probe_momenta(&final_masses, PROBE_SEED);
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
            // An event is labelled with its group representative's colour-flow
            // basis, so members must share it. `|M|²` on its own does not imply
            // they do: it is a sum over the basis and would not move if two
            // members' bases differed by a relabelling.
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
                let legs = &compiled[i].as_ref().expect("member unclaimed").1;
                Subprocess {
                    incoming: [legs[0].pdg, legs[1].pdg],
                    outgoing: legs[2..].iter().map(|l| l.pdg).collect(),
                }
            })
            .collect();
        let (evaluator, legs, cuts) = compiled[head].take().expect("group head unclaimed");
        let spin_color_avg = initial_spin_color_average(&evaluator, model, evaluated);
        groups.push(FlavorGroup {
            representative: sets[head].take().expect("group head unclaimed"),
            evaluator,
            legs,
            cuts,
            members,
            spin_color_avg,
        });
    }

    Ok(FlavorGroups { groups })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::pdf::grid::SubGrid;
    use crate::ufo::sm::{sm_model, SMRestrict};
    use std::collections::BTreeSet;
    use std::sync::Arc;

    /// The multi-subprocess hadronic process the grouping rule is measured on:
    /// two beam multiparticles, two coupling classes, and a jet that is a gluon
    /// in one arrangement and a quark in the other.
    const LLJ: &str = "p p > l+ l- j QCD=2 QED=2";

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
            let subprocess = Subprocess {
                incoming: [legs[0].pdg, legs[1].pdg],
                outgoing: legs[2..].iter().map(|l| l.pdg).collect(),
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
    /// representative at the *unreflected* point (what dropping the mirror
    /// amounts to) is wrong at every probe point, by between 0.2% and a factor of
    /// 200.
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

        let (mut worst_mirror, mut min_identity, mut worst_py) = (0.0f64, f64::INFINITY, 0.0f64);
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
            assert_eq!(
                mirror_set.diagrams.len(),
                set.diagrams.len(),
                "{swapped} is not the same process as {}",
                label(set)
            );
            let mirror_eval = compile_class(&mirror_set, &m, &evaluated).expect("mirror compiles");

            let bound = BoundAmplitude::<f64>::bind(g.evaluator(), &evaluated);
            let mut scratch = bound.scratch_space();
            let mirror_bound = BoundAmplitude::<f64>::bind(&mirror_eval, &evaluated);
            let mut mirror_scratch = mirror_bound.scratch_space();

            let mut reflected = Vec::new();
            for k in &points {
                let target = mirror_bound.eval_m2(k, &mut mirror_scratch);
                let rel = |x: f64| (x - target).abs() / target.abs().max(f64::MIN_POSITIVE);

                g.mirror_into(k, &mut reflected);
                worst_mirror = worst_mirror.max(rel(bound.eval_m2(&reflected, &mut scratch)));

                min_identity = min_identity.min(rel(bound.eval_m2(k, &mut scratch)));

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
            "mirror identity worst {worst_mirror:.3e}; dropping it costs at least \
             {min_identity:.3e}; an xz reflection alone moves |M|² by {worst_py:.3e}"
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
            min_identity > 1e-3,
            "the mirror term is worth only {min_identity:.3e} at its weakest point; a dropped \
             mirror would not be visible here"
        );
        assert!(
            worst_py < 1e-12,
            "|M|² moved by {worst_py:.3e} under a reflection in the xz plane, so the sign of p_y \
             in the mirror map is pinned after all and this test's blind spot is misstated"
        );
    }

    /// One group per identical-particle-free subprocess is a premise, not a
    /// convention: a repeated outgoing species is refused rather than silently
    /// dropped into a summed matrix element with no owner for its `1/n!`.
    #[test]
    fn a_repeated_outgoing_species_is_refused() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        for g in derive(LLJ, &m, &evaluated).groups() {
            assert_eq!(final_state_symmetry_factor(g.evaluator()), 1.0);
        }
        let err = match derive_flavor_groups(
            enumerate("u u~ > g g", &m),
            &m,
            &evaluated,
            &RunCard::default(),
        ) {
            Ok(_) => panic!("two outgoing gluons must be refused"),
            Err(e) => e,
        };
        assert!(
            matches!(err, ProtonError::IdenticalFinalState { factor, .. } if factor == 0.5),
            "unexpected refusal: {err}"
        );
    }

    /// The generalisation lands on Drell–Yan's hand-derived classes, and its two
    /// orderings sum to the luminosity `DrellYanIntegrand` documents.
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
}
