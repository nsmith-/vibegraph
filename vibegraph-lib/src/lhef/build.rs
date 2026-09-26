//! Assembling an event record out of what the generator produces.
//!
//! An accepted point supplies the momenta and a weight; the per-event selections
//! supply a helicity combination and a colour flow; the run card's scale
//! prescription supplies `μR`, `μF` and the couplings they imply. Everything else
//! on the record — the PDG codes, the masses, the incoming/outgoing statuses, the
//! mother pointers and the colour-line labels — is fixed by the subprocess and is
//! resolved once, by [`SubprocessRecord`].

use crate::coupling::scales::EventScales;
use crate::helas::color::flow_tags::{ColorFlowTags, LegColor};
use crate::helas::eval::AmplitudeEvaluator;
use crate::ufo::{EvaluatedModel, UFOModel};

use super::record::{
    LheEvent, LheParticle, NO_COLOR_LINE, SPIN_UNKNOWN, STATUS_INCOMING, STATUS_INTERMEDIATE,
    STATUS_OUTGOING,
};
use super::LhefError;

/// The `SCALUP` field: the larger of the two factorisation scales.
///
/// The accord defines `SCALUP` as the scale the parton densities were evaluated
/// at, and MadGraph fills it as `sqrt(max(q2fact(1), q2fact(2)))` — the
/// factorisation scale on the beam that carries the larger one. It is **not** the
/// renormalisation scale. The two coincide whenever the scale prescription reads
/// both off the same vertex, which is every closed-form clustering this crate
/// computes, and that coincidence is why the field is so often read as `μR`; on a
/// process whose clustering splits them the reading is simply wrong, and
/// `validate_scales` measures the split on the banked `2 → 6` runs.
///
/// The renormalisation scale reaches the record through `AQCDUP` instead, as
/// `αs(μR)`.
pub fn scalup(scales: &EventScales) -> f64 {
    scales.mu_f[0].max(scales.mu_f[1])
}

/// The scalar fields of one `<event>` line.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EventHeader {
    /// `IDPRUP` — which `<init>` process entry the event belongs to.
    pub process_id: i32,
    /// `XWGTUP`, in the units the file's `IDWTUP` implies.
    pub weight: f64,
    /// `SCALUP` (see [`scalup`]).
    pub scale: f64,
    /// `AQEDUP`.
    pub alpha_qed: f64,
    /// `AQCDUP` — `αs(μR)`.
    ///
    /// MadGraph's own files carry `αs·(1 + 1.7e-8)` here, because `unwgt.f`
    /// divides by a π truncated to eight digits while the coupling was built from
    /// the full one. That is a defect of the field, not a convention of it, so
    /// this is the untruncated value.
    pub alpha_qcd: f64,
}

impl EventHeader {
    /// The header of an event evaluated at `scales`, with the couplings those
    /// scales imply.
    pub fn from_scales(
        process_id: i32,
        weight: f64,
        scales: &EventScales,
        alpha_qed: f64,
        alpha_qcd: f64,
    ) -> Self {
        EventHeader {
            process_id,
            weight,
            scale: scalup(scales),
            alpha_qed,
            alpha_qcd,
        }
    }
}

/// Turning the generator's dimensionless event weights into `XWGTUP` values.
///
/// Under [`WeightStrategy::MeanCrossSectionPb`](super::record::WeightStrategy),
/// the cross section is the **mean** of the event weights, so an unweighted
/// sample whose events mostly carry weight `1` needs each weight multiplied by
/// the cross section itself. Overweight events — the ones an accept/reject pass
/// keeps at a weight above one because they exceeded the estimated maximum — then
/// carry proportionally more, which is exactly what keeps the mean unbiased.
#[derive(Clone, Copy, Debug)]
pub struct WeightNormalisation {
    scale_pb: f64,
}

impl WeightNormalisation {
    /// From an accept/reject pass: the cross section it recovered and the mean of
    /// the weights its kept events carry (`1` when nothing went overweight).
    ///
    /// A non-positive mean weight leaves the normalisation at zero rather than
    /// producing infinities, since a sample with no events has no scale to set.
    pub fn new(sigma_pb: f64, mean_event_weight: f64) -> Self {
        let scale_pb = if mean_event_weight > 0.0 {
            sigma_pb / mean_event_weight
        } else {
            0.0
        };
        WeightNormalisation { scale_pb }
    }

    /// The `XWGTUP` of an event carrying the generator's weight `event_weight`.
    pub fn xwgtup(&self, event_weight: f64) -> f64 {
        self.scale_pb * event_weight
    }
}

/// A resonance an event record lists between the incoming and the outgoing legs
/// (`ISTUP = 2`), named by the outgoing legs it decays into.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Intermediate {
    /// `IDUP`.
    pub pdg: i32,
    /// Its SU(3) representation in the UFO's code: `1`, `3`, `-3` or `8`.
    pub color: i32,
    /// The outgoing legs below it, bit `k` naming the `k`-th outgoing leg in the
    /// record's own order.
    pub slots: u64,
}

/// Everything an event record needs about one subprocess, resolved once.
///
/// The external-leg order is the amplitude's own — incoming legs first — and is
/// the order the momenta, the helicity combination and the colour-flow table are
/// all indexed in, so no permutation is applied anywhere below.
#[derive(Clone, Debug)]
pub struct SubprocessRecord {
    /// PDG code per external leg.
    pdg: Vec<i32>,
    /// Pole mass per external leg, in GeV.
    mass: Vec<f64>,
    n_in: usize,
    /// The colour rep and direction of every leg *this record* describes — the reps
    /// [`SubprocessRecord::flows`] is checked against.
    legs: Vec<LegColor>,
    /// `(colour, anticolour)` line labels per leg, per flow.
    flows: ColorFlowTags,
}

impl SubprocessRecord {
    /// Resolve a compiled subprocess's PDG codes, masses and colour-flow table.
    pub fn new(
        evaluator: &AmplitudeEvaluator,
        model: &UFOModel,
        evaluated: &EvaluatedModel,
    ) -> Result<Self, LhefError> {
        let mut pdg = Vec::with_capacity(evaluator.n_ext());
        let mut mass = Vec::with_capacity(evaluator.n_ext());
        for (leg, &id) in evaluator.external_particles().iter().enumerate() {
            let code = model.particle(id).pdg_code;
            pdg.push(i32::try_from(code).map_err(|_| LhefError::PdgOutOfRange { leg, pdg: code })?);
            mass.push(evaluated.mass(id));
        }
        Ok(SubprocessRecord {
            pdg,
            mass,
            n_in: evaluator.n_in(),
            legs: evaluator.external_colors().to_vec(),
            flows: evaluator.color_flow_tags().clone(),
        })
    }

    /// The same compiled subprocess read on reordered, relabelled external legs:
    /// leg `i` of the result is leg `order[i]` of `self`, carrying PDG code
    /// `pdg[i]`.
    ///
    /// A hadron-collider event needs this because one compiled amplitude serves
    /// several concrete flavour assignments and both beam orderings. The pole masses
    /// travel with the legs and only the codes change; exchanging the two beams
    /// exchanges their momenta along with everything else the record says about them.
    ///
    /// **The colour flows do not travel with the legs at all.** `flows` is the
    /// member's *own* subprocess's table, already reordered into this one's flow
    /// indexing by the caller, and all that happens here is the beam-exchange
    /// permutation. Two subprocesses can share a matrix element, a mass list and a
    /// colour-factor matrix and still route their colour lines between different
    /// pairs of legs — a quark and an antiquark on the same leg carry conjugate SU(3)
    /// reps, and conjugating one end of a line moves it — so nothing derived from
    /// this record's own flows would be right for them.
    ///
    /// `legs` is the reps those legs carry, and the table is checked against them
    /// before it can reach an event: a line in a slot its leg's rep forbids is
    /// refused here rather than emitted.
    ///
    /// The incoming/outgoing split is `self`'s, so a permutation that moves a leg
    /// across it is refused.
    pub fn relabelled(
        &self,
        order: &[usize],
        pdg: &[i32],
        legs: &[LegColor],
        flows: &ColorFlowTags,
    ) -> Result<Self, LhefError> {
        let n_ext = self.n_ext();
        let well_formed = order.len() == n_ext
            && pdg.len() == n_ext
            && legs.len() == n_ext
            && order.iter().enumerate().all(|(i, &leg)| {
                leg < n_ext && (i < self.n_in) == (leg < self.n_in) && !order[..i].contains(&leg)
            });
        if !well_formed {
            return Err(LhefError::LegOrder {
                order: order.to_vec(),
                n_ext,
            });
        }
        if flows.n_ext() != n_ext || flows.n_flows() != self.n_flows() {
            return Err(LhefError::ColorFlowShape {
                n_ext,
                n_flows: self.n_flows(),
                got_ext: flows.n_ext(),
                got_flows: flows.n_flows(),
            });
        }
        let flows = flows.permuted(order).ok_or_else(|| LhefError::LegOrder {
            order: order.to_vec(),
            n_ext,
        })?;
        flows
            .check_legs(legs)
            .map_err(|e| LhefError::ColorFlowLegs(e.to_string()))?;
        Ok(SubprocessRecord {
            pdg: pdg.to_vec(),
            mass: order.iter().map(|&leg| self.mass[leg]).collect(),
            n_in: self.n_in,
            legs: legs.to_vec(),
            flows,
        })
    }

    /// The colour rep and direction of every leg, in this record's own order.
    pub fn legs(&self) -> &[LegColor] {
        &self.legs
    }

    /// This record's per-flow `(colour, anticolour)` tags.
    pub fn flows(&self) -> &ColorFlowTags {
        &self.flows
    }

    /// The number of external legs.
    pub fn n_ext(&self) -> usize {
        self.pdg.len()
    }

    /// The PDG code of every external leg, in process order — the incoming ones
    /// are also what an `<init>` block's `IDBMUP` reports for a fixed-beam run.
    pub fn pdg(&self) -> &[i32] {
        &self.pdg
    }

    /// The pole mass of every external leg, in process order.
    pub fn masses(&self) -> &[f64] {
        &self.mass
    }

    /// The number of incoming legs.
    pub fn n_in(&self) -> usize {
        self.n_in
    }

    /// The number of colour flows a record may select from.
    pub fn n_flows(&self) -> usize {
        self.flows.n_flows()
    }

    /// Build one `<event>` record.
    ///
    /// `momenta` are the physical four-momenta of every external leg in
    /// `[E, px, py, pz]`, incoming legs first and carrying their own signs — the
    /// second beam runs down the axis with `pz < 0`, and no all-outgoing crossing
    /// is applied. `helicity` is the selected combination, one entry per leg, and
    /// `flow` indexes the subprocess's colour-flow basis.
    pub fn event(
        &self,
        momenta: &[[f64; 4]],
        helicity: &[i32],
        flow: usize,
        header: EventHeader,
    ) -> Result<LheEvent, LhefError> {
        if momenta.len() != self.n_ext() {
            return Err(LhefError::MomentumCount {
                want: self.n_ext(),
                got: momenta.len(),
            });
        }
        if helicity.len() != self.n_ext() {
            return Err(LhefError::HelicityCount {
                want: self.n_ext(),
                got: helicity.len(),
            });
        }
        if flow >= self.n_flows() {
            return Err(LhefError::FlowOutOfRange {
                flow,
                n_flows: self.n_flows(),
            });
        }
        let tags = self.flows.flow(flow);
        // Every leg leaving the hard process descends from the whole initial
        // state, so its mother range spans the incoming legs; an incoming leg has
        // no mother in the record. A decay's products name the decaying particle
        // alone, `(1, 0)` rather than the range `(1, 1)`: MadEvent's `unwgt.f`
        // zeroes the second mother of every line whose first is `1` when
        // `nincoming = 1`.
        let outgoing_mothers = if self.n_in == 1 {
            [1, 0]
        } else {
            [1, self.n_in as i32]
        };
        let particles = (0..self.n_ext())
            .map(|leg| LheParticle {
                pdg: self.pdg[leg],
                status: if leg < self.n_in {
                    STATUS_INCOMING
                } else {
                    STATUS_OUTGOING
                },
                mothers: if leg < self.n_in {
                    [0, 0]
                } else {
                    outgoing_mothers
                },
                color: [tags[leg][0] as i32, tags[leg][1] as i32],
                momentum: momenta[leg],
                mass: self.mass[leg],
                // Tree-level external legs are stable as far as this record is
                // concerned; a lifetime belongs to a decay the shower performs.
                lifetime: 0.0,
                spin: f64::from(helicity[leg]),
            })
            .collect();
        Ok(LheEvent {
            process_id: header.process_id,
            weight: header.weight,
            scale: header.scale,
            alpha_qed: header.alpha_qed,
            alpha_qcd: header.alpha_qcd,
            particles,
            trailer: Vec::new(),
            source: None,
        })
    }

    /// [`event`](Self::event) with `intermediates` written as status-2 records,
    /// in MadEvent's layout (`addmothers.f`).
    ///
    /// * **Order.** The incoming legs, then the intermediates, then the outgoing
    ///   legs in their own order. The intermediates run parent before child, a
    ///   resonance's own sub-resonances directly after it, and siblings by their
    ///   lowest outgoing leg. MadEvent also writes every parent before its
    ///   children, but orders siblings by its configuration tag's sort, which
    ///   differs between configurations of one subprocess; positions carry no
    ///   physics, since every pointer names its target by position.
    /// * **Mothers.** A top-level intermediate descends from the whole initial
    ///   state, `(1, n_in)`, or `(1, 0)` on a decay (`unwgt.f:741`); a nested one
    ///   names its parent twice, `(k, k)`, as does every outgoing leg below an
    ///   intermediate, the innermost one. An outgoing leg below none keeps the
    ///   initial state.
    /// * **Momentum and mass.** The sum of the outgoing legs below it, and its
    ///   virtuality `√max(0, p²)` rather than its pole mass.
    /// * **Colour.** What its daughters leave open once every line one daughter
    ///   carries as colour and another as anticolour is contracted
    ///   (`elim_indices`, `addmothers.f:793`), which has to fit its own
    ///   representation: nothing for a singlet, one colour for a triplet, one
    ///   anticolour for an antitriplet, one of each for an octet.
    /// * **`SPINUP`** [`SPIN_UNKNOWN`]: an intermediate's helicity is summed over.
    ///
    /// An intermediate that shares its outgoing legs with another, or whose
    /// daughters' colour does not fit it, is refused rather than written.
    pub fn event_with_intermediates(
        &self,
        momenta: &[[f64; 4]],
        helicity: &[i32],
        flow: usize,
        header: EventHeader,
        intermediates: &[Intermediate],
    ) -> Result<LheEvent, LhefError> {
        let mut event = self.event(momenta, helicity, flow, header)?;
        if intermediates.is_empty() {
            return Ok(event);
        }
        let n_in = self.n_in;
        let n_res = intermediates.len();
        // The innermost intermediate strictly containing each one.
        let contains = |outer: u64, inner: u64| inner & !outer == 0 && inner != outer;
        let mut parent: Vec<Option<usize>> = vec![None; n_res];
        for (i, res) in intermediates.iter().enumerate() {
            for (j, other) in intermediates.iter().enumerate() {
                if i != j && other.slots == res.slots {
                    return Err(LhefError::IntermediateNesting {
                        a: res.pdg,
                        b: other.pdg,
                    });
                }
                if contains(other.slots, res.slots)
                    && parent[i].is_none_or(|p| {
                        intermediates[p].slots.count_ones() > other.slots.count_ones()
                    })
                {
                    parent[i] = Some(j);
                }
            }
        }
        // Parent before child, siblings by their lowest outgoing leg.
        let mut order: Vec<usize> = Vec::with_capacity(n_res);
        let children = |of: Option<usize>| -> Vec<usize> {
            let mut c: Vec<usize> = (0..n_res).filter(|&i| parent[i] == of).collect();
            c.sort_by_key(|&i| intermediates[i].slots.trailing_zeros());
            c
        };
        let mut stack: Vec<usize> = children(None).into_iter().rev().collect();
        while let Some(i) = stack.pop() {
            order.push(i);
            stack.extend(children(Some(i)).into_iter().rev());
        }
        // 1-based record position of each intermediate.
        let mut position = vec![0i32; n_res];
        for (at, &i) in order.iter().enumerate() {
            position[i] = (n_in + at + 1) as i32;
        }
        let top_mothers = if n_in == 1 { [1, 0] } else { [1, n_in as i32] };
        let outgoing = &event.particles[n_in..];
        // The innermost intermediate above each outgoing leg.
        let above = |leg: usize| -> Option<usize> {
            (0..n_res)
                .filter(|&i| intermediates[i].slots & (1u64 << leg) != 0)
                .min_by_key(|&i| intermediates[i].slots.count_ones())
        };
        // Colours, children before parents: larger masks contain smaller ones.
        let mut colors: Vec<[i32; 2]> = vec![[NO_COLOR_LINE; 2]; n_res];
        let mut by_size: Vec<usize> = (0..n_res).collect();
        by_size.sort_by_key(|&i| intermediates[i].slots.count_ones());
        for &i in &by_size {
            let mut open: Vec<[i32; 2]> = (0..n_res)
                .filter(|&j| parent[j] == Some(i))
                .map(|j| colors[j])
                .collect();
            open.extend(
                (0..outgoing.len())
                    .filter(|&leg| above(leg) == Some(i))
                    .map(|leg| outgoing[leg].color),
            );
            let labels_c: Vec<i32> = open
                .iter()
                .map(|c| c[0])
                .filter(|&c| c != NO_COLOR_LINE)
                .collect();
            let labels_a: Vec<i32> = open
                .iter()
                .map(|c| c[1])
                .filter(|&c| c != NO_COLOR_LINE)
                .collect();
            let free_c: Vec<i32> = labels_c
                .iter()
                .copied()
                .filter(|c| !labels_a.contains(c))
                .collect();
            let free_a: Vec<i32> = labels_a
                .iter()
                .copied()
                .filter(|a| !labels_c.contains(a))
                .collect();
            let res = &intermediates[i];
            let want = match res.color {
                1 => (0, 0),
                3 => (1, 0),
                -3 => (0, 1),
                8 => (1, 1),
                _ => (usize::MAX, usize::MAX),
            };
            if (free_c.len(), free_a.len()) != want {
                return Err(LhefError::IntermediateColor {
                    pdg: res.pdg,
                    rep: res.color,
                    colors: free_c.len(),
                    anticolors: free_a.len(),
                });
            }
            colors[i] = [
                free_c.first().copied().unwrap_or(NO_COLOR_LINE),
                free_a.first().copied().unwrap_or(NO_COLOR_LINE),
            ];
        }
        let mut records: Vec<LheParticle> = Vec::with_capacity(n_res);
        for &i in &order {
            let res = &intermediates[i];
            let mut p = [0.0; 4];
            for (leg, q) in outgoing.iter().enumerate() {
                if res.slots & (1u64 << leg) != 0 {
                    for (sum, component) in p.iter_mut().zip(q.momentum) {
                        *sum += component;
                    }
                }
            }
            let mothers = match parent[i] {
                Some(up) => [position[up]; 2],
                None => top_mothers,
            };
            records.push(LheParticle {
                pdg: res.pdg,
                status: STATUS_INTERMEDIATE,
                mothers,
                color: colors[i],
                momentum: p,
                mass: (p[0] * p[0] - p[1] * p[1] - p[2] * p[2] - p[3] * p[3])
                    .max(0.0)
                    .sqrt(),
                lifetime: 0.0,
                spin: SPIN_UNKNOWN,
            });
        }
        let mut finals: Vec<LheParticle> = event.particles.split_off(n_in);
        for (leg, particle) in finals.iter_mut().enumerate() {
            if let Some(i) = above(leg) {
                particle.mothers = [position[i]; 2];
            }
        }
        event.particles.extend(records);
        event.particles.extend(finals);
        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helas::color::colorize::{BasisElement, ColorBasis};
    use crate::helas::color::flow_tags::{color_flow_tags, LegColor};
    use crate::helas::color::tensor::TensorKind;
    use crate::helas::repr::color::ColorRep;
    use num_rational::Ratio;

    /// `g g > t t~` as the record layer sees it: one colour flow `T(1,2,3,4)`,
    /// two incoming gluons, an outgoing quark pair.
    fn ggttx() -> SubprocessRecord {
        let legs = [
            LegColor {
                rep: ColorRep::Octet,
                incoming: true,
            },
            LegColor {
                rep: ColorRep::Octet,
                incoming: true,
            },
            LegColor {
                rep: ColorRep::Triplet,
                incoming: false,
            },
            LegColor {
                rep: ColorRep::AntiTriplet,
                incoming: false,
            },
        ];
        let basis = ColorBasis {
            elements: vec![BasisElement {
                structure: vec![(TensorKind::T, vec![1, 2, 3, 4])],
                contributions: Vec::new(),
            }],
            cf_matrix: vec![Ratio::from_integer(1)],
        };
        SubprocessRecord {
            pdg: vec![21, 21, 6, -6],
            mass: vec![0.0, 0.0, 173.0, 173.0],
            n_in: 2,
            legs: legs.to_vec(),
            flows: color_flow_tags(&basis, &legs).expect("flow tags"),
        }
    }

    fn momenta() -> Vec<[f64; 4]> {
        vec![
            [250.0, 0.0, 0.0, 250.0],
            [250.0, 0.0, 0.0, -250.0],
            [250.0, 81.0, 18.0, -160.0],
            [250.0, -81.0, -18.0, 160.0],
        ]
    }

    fn header() -> EventHeader {
        EventHeader {
            process_id: 1,
            weight: 15.95319,
            scale: 250.0,
            alpha_qed: 0.007546771,
            alpha_qcd: 0.1113305,
        }
    }

    fn built() -> LheEvent {
        ggttx()
            .event(&momenta(), &[1, -1, 1, -1], 0, header())
            .expect("record")
    }

    /// The three conventions a record can get wrong without any `|M|²`-level gate
    /// noticing: which legs are incoming, what an outgoing leg's mothers are, and
    /// whether the incoming momenta were crossed.
    #[test]
    fn statuses_mothers_and_incoming_momenta_follow_the_leg_order() {
        let event = built();
        let statuses: Vec<i32> = event.particles.iter().map(|p| p.status).collect();
        assert_eq!(
            statuses,
            [
                STATUS_INCOMING,
                STATUS_INCOMING,
                STATUS_OUTGOING,
                STATUS_OUTGOING
            ]
        );
        let mothers: Vec<[i32; 2]> = event.particles.iter().map(|p| p.mothers).collect();
        assert_eq!(mothers, [[0, 0], [0, 0], [1, 2], [1, 2]]);

        // The record carries physical momenta: the second beam runs down the axis,
        // and the pair does not come out crossed to all-outgoing.
        assert_eq!(event.particles[0].momentum, [250.0, 0.0, 0.0, 250.0]);
        assert_eq!(event.particles[1].momentum, [250.0, 0.0, 0.0, -250.0]);
        assert_eq!(event.particles[2].momentum[1], 81.0);
        let total_e: f64 = event.particles[..2].iter().map(|p| p.momentum[0]).sum();
        let out_e: f64 = event.particles[2..].iter().map(|p| p.momentum[0]).sum();
        assert_eq!(total_e, out_e);
    }

    /// `ICOLUP` slot 1 is the physical colour and slot 2 the anticolour, whichever
    /// way the leg runs. On this flow the top takes a colour line and the antitop
    /// an anticolour line; a writer that filled the slots from the amplitude's
    /// all-outgoing index rep instead would put the antitop's line in slot 1.
    #[test]
    fn colour_lines_land_in_the_physical_slots() {
        let event = built();
        let colors: Vec<[i32; 2]> = event.particles.iter().map(|p| p.color).collect();
        assert_eq!(colors, [[501, 502], [502, 503], [501, 0], [0, 503]]);
        // Every line joins exactly two endpoints, and does so in the crossed
        // pattern the physical slots force: two legs on the same side of the
        // process are joined colour-to-anticolour, while a line running from an
        // initial-state leg to a final-state one keeps its slot, because crossing
        // the leg already conjugated the index once.
        for line in event.color_connectivity() {
            assert_eq!(line.len(), 2, "a colour line joins exactly two endpoints");
            let [(leg_a, slot_a), (leg_b, slot_b)] = line[..] else {
                unreachable!("checked just above")
            };
            let same_side = (leg_a < 2) == (leg_b < 2);
            assert_eq!(
                same_side,
                slot_a != slot_b,
                "line {line:?} does not respect the crossing rule"
            );
        }
    }

    #[test]
    fn masses_are_the_pole_masses_and_helicities_reach_spinup() {
        let event = built();
        let masses: Vec<f64> = event.particles.iter().map(|p| p.mass).collect();
        assert_eq!(masses, [0.0, 0.0, 173.0, 173.0]);
        let spins: Vec<f64> = event.particles.iter().map(|p| p.spin).collect();
        assert_eq!(spins, [1.0, -1.0, 1.0, -1.0]);
        assert!(event.particles.iter().all(|p| p.lifetime == 0.0));
    }

    #[test]
    fn a_mismatched_input_is_refused_rather_than_truncated() {
        let record = ggttx();
        let h = header();
        assert_eq!(
            record.event(&momenta()[..3], &[1, -1, 1, -1], 0, h),
            Err(LhefError::MomentumCount { want: 4, got: 3 })
        );
        assert_eq!(
            record.event(&momenta(), &[1, -1, 1], 0, h),
            Err(LhefError::HelicityCount { want: 4, got: 3 })
        );
        assert_eq!(
            record.event(&momenta(), &[1, -1, 1, -1], 1, h),
            Err(LhefError::FlowOutOfRange {
                flow: 1,
                n_flows: 1
            })
        );
    }

    /// `e+ e- > b e+ ve b~ mu- vm~` as a top-pair decay chain records it: one
    /// flow joining the `b` to the `b~`.
    fn ttx_chain() -> SubprocessRecord {
        let out = |rep| LegColor {
            rep,
            incoming: false,
        };
        let inc = LegColor {
            rep: ColorRep::Singlet,
            incoming: true,
        };
        let legs = [
            inc,
            inc,
            out(ColorRep::Triplet),
            out(ColorRep::Singlet),
            out(ColorRep::Singlet),
            out(ColorRep::AntiTriplet),
            out(ColorRep::Singlet),
            out(ColorRep::Singlet),
        ];
        let basis = ColorBasis {
            elements: vec![BasisElement {
                structure: vec![(TensorKind::T, vec![3, 6])],
                contributions: Vec::new(),
            }],
            cf_matrix: vec![Ratio::from_integer(3)],
        };
        SubprocessRecord {
            pdg: vec![-11, 11, 5, -11, 12, -5, 13, -14],
            mass: vec![0.0, 0.0, 4.7, 0.0, 0.0, 4.7, 0.0, 0.0],
            n_in: 2,
            legs: legs.to_vec(),
            flows: color_flow_tags(&basis, &legs).expect("flow tags"),
        }
    }

    fn ttx_momenta() -> Vec<[f64; 4]> {
        vec![
            [250.0, 0.0, 0.0, 250.0],
            [250.0, 0.0, 0.0, -250.0],
            [60.0, 10.0, 20.0, 57.0],
            [70.0, -30.0, 40.0, 40.0],
            [120.0, 50.0, 60.0, 80.0],
            [80.0, -20.0, -30.0, -70.0],
            [90.0, 10.0, -50.0, -70.0],
            [80.0, -20.0, -40.0, -37.0],
        ]
    }

    /// The layout MadEvent's `addmothers` writes a decay chain in: resonances
    /// between the incoming and the outgoing legs, a nested one and every leg
    /// below it naming its innermost resonance twice, a top-level one the whole
    /// initial state, the colour its daughters leave open, and their summed
    /// momentum with its own virtuality as the mass.
    #[test]
    fn intermediates_follow_madevents_layout() {
        let record = ttx_chain();
        let intermediates = [
            Intermediate {
                pdg: 24,
                color: 1,
                slots: 0b000110,
            },
            Intermediate {
                pdg: -6,
                color: -3,
                slots: 0b111000,
            },
            Intermediate {
                pdg: 6,
                color: 3,
                slots: 0b000111,
            },
            Intermediate {
                pdg: -24,
                color: 1,
                slots: 0b110000,
            },
        ];
        let helicity = [1, -1, -1, 1, -1, 1, -1, 1];
        let event = record
            .event_with_intermediates(&ttx_momenta(), &helicity, 0, header(), &intermediates)
            .expect("record");
        let rows: Vec<(i32, i32, [i32; 2], [i32; 2])> = event
            .particles
            .iter()
            .map(|p| (p.pdg, p.status, p.mothers, p.color))
            .collect();
        assert_eq!(
            rows,
            [
                (-11, STATUS_INCOMING, [0, 0], [0, 0]),
                (11, STATUS_INCOMING, [0, 0], [0, 0]),
                (6, STATUS_INTERMEDIATE, [1, 2], [501, 0]),
                (24, STATUS_INTERMEDIATE, [3, 3], [0, 0]),
                (-6, STATUS_INTERMEDIATE, [1, 2], [0, 501]),
                (-24, STATUS_INTERMEDIATE, [5, 5], [0, 0]),
                (5, STATUS_OUTGOING, [3, 3], [501, 0]),
                (-11, STATUS_OUTGOING, [4, 4], [0, 0]),
                (12, STATUS_OUTGOING, [4, 4], [0, 0]),
                (-5, STATUS_OUTGOING, [5, 5], [0, 501]),
                (13, STATUS_OUTGOING, [6, 6], [0, 0]),
                (-14, STATUS_OUTGOING, [6, 6], [0, 0]),
            ]
        );
        let p = ttx_momenta();
        let top = &event.particles[2];
        for (k, &component) in top.momentum.iter().enumerate() {
            assert_eq!(component, p[2][k] + p[3][k] + p[4][k]);
        }
        let m2 = top.momentum[0].powi(2)
            - top.momentum[1].powi(2)
            - top.momentum[2].powi(2)
            - top.momentum[3].powi(2);
        assert_eq!(top.mass, m2.sqrt());
        assert!(event.particles[2..6].iter().all(|p| p.spin == SPIN_UNKNOWN));
        // The outgoing legs keep their own helicities.
        let spins: Vec<f64> = event.particles[6..].iter().map(|p| p.spin).collect();
        assert_eq!(spins, [-1.0, 1.0, -1.0, 1.0, -1.0, 1.0]);
    }

    /// A decay at rest names its decaying particle alone, `(1, 0)`, on every
    /// line whose mother is the initial state — the intermediate included, as
    /// `unwgt.f:741` zeroes the second mother of each after `addmothers`.
    #[test]
    fn a_decays_intermediates_name_the_decaying_particle_alone() {
        let legs = [
            LegColor {
                rep: ColorRep::Triplet,
                incoming: true,
            },
            LegColor {
                rep: ColorRep::Triplet,
                incoming: false,
            },
            LegColor {
                rep: ColorRep::Singlet,
                incoming: false,
            },
            LegColor {
                rep: ColorRep::Singlet,
                incoming: false,
            },
        ];
        let basis = ColorBasis {
            elements: vec![BasisElement {
                structure: vec![(TensorKind::T, vec![2, 1])],
                contributions: Vec::new(),
            }],
            cf_matrix: vec![Ratio::from_integer(3)],
        };
        let record = SubprocessRecord {
            pdg: vec![6, 5, -11, 12],
            mass: vec![173.0, 4.7, 0.0, 0.0],
            n_in: 1,
            legs: legs.to_vec(),
            flows: color_flow_tags(&basis, &legs).expect("flow tags"),
        };
        let momenta = [
            [173.0, 0.0, 0.0, 0.0],
            [70.0, 0.0, 0.0, 69.8],
            [50.0, 0.0, 30.0, -40.0],
            [53.0, 0.0, -30.0, -29.8],
        ];
        let event = record
            .event_with_intermediates(
                &momenta,
                &[1, -1, 1, -1],
                0,
                header(),
                &[Intermediate {
                    pdg: 24,
                    color: 1,
                    slots: 0b110,
                }],
            )
            .expect("record");
        let mothers: Vec<[i32; 2]> = event.particles.iter().map(|p| p.mothers).collect();
        assert_eq!(mothers, [[0, 0], [1, 0], [1, 0], [2, 2], [2, 2]]);
    }

    /// An intermediate whose daughters leave a colour line its representation
    /// cannot carry is refused rather than written with a colour of its own
    /// making, and so is one that shares its legs with another.
    #[test]
    fn an_intermediate_that_does_not_fit_is_refused() {
        let record = ttx_chain();
        let lone_quark_singlet = Intermediate {
            pdg: 24,
            color: 1,
            slots: 0b000011,
        };
        assert!(matches!(
            record.event_with_intermediates(
                &ttx_momenta(),
                &[1; 8],
                0,
                header(),
                &[lone_quark_singlet]
            ),
            Err(LhefError::IntermediateColor { pdg: 24, .. })
        ));
        let twice = Intermediate {
            pdg: 6,
            color: 3,
            slots: 0b000111,
        };
        assert!(matches!(
            record.event_with_intermediates(&ttx_momenta(), &[1; 8], 0, header(), &[twice, twice]),
            Err(LhefError::IntermediateNesting { .. })
        ));
    }

    /// `SCALUP` is the larger factorisation scale. Every process whose clustering
    /// this crate computes has `μR = μF`, so only a case built with them apart can
    /// tell the two readings apart at all.
    #[test]
    fn scalup_is_the_factorisation_scale_not_the_renormalisation_one() {
        let scales = EventScales {
            mu_r: 91.188,
            mu_f: [200.0, 50.0],
        };
        assert_eq!(scalup(&scales), 200.0);
        assert_ne!(scalup(&scales), scales.mu_r);
        let head = EventHeader::from_scales(1, 1.0, &scales, 0.0075, 0.118);
        assert_eq!(head.scale, 200.0);
        assert_eq!(head.alpha_qcd, 0.118);
    }

    /// `AQCDUP` is `αs`, not MadGraph's `αs·π/3.1415926`. The bias is a sixth of
    /// the field's last printed digit, so the only way to state the choice is to
    /// assert the size of the difference.
    #[test]
    fn aqcdup_does_not_reproduce_the_truncated_pi() {
        let alpha_s = 0.1113305_f64;
        let head = EventHeader::from_scales(
            1,
            1.0,
            &EventScales {
                mu_r: 250.0,
                mu_f: [250.0; 2],
            },
            0.0075,
            alpha_s,
        );
        // The literal `unwgt.f` divides by, spelled out because being an
        // approximation of π is the whole point of it.
        #[allow(clippy::approx_constant)]
        const TRUNCATED_PI: f64 = 3.1415926;
        let madgraph = alpha_s * std::f64::consts::PI / TRUNCATED_PI;
        let relative = (madgraph - head.alpha_qcd) / alpha_s;
        assert!(
            (relative - 1.7e-8).abs() < 1e-9,
            "MadGraph's truncation is {relative:.3e} relative"
        );
        assert_eq!(head.alpha_qcd, alpha_s);
    }

    /// The mean of the emitted weights is the cross section, which is what
    /// `IDWTUP = -4` promises a consumer.
    #[test]
    fn weights_normalise_so_their_mean_is_the_cross_section() {
        let weights = [1.0, 1.0, 1.0, 1.0, 2.5, 1.0, 1.0, 1.7];
        let mean = weights.iter().sum::<f64>() / weights.len() as f64;
        let norm = WeightNormalisation::new(15.95319, mean);
        let emitted: f64 =
            weights.iter().map(|&w| norm.xwgtup(w)).sum::<f64>() / weights.len() as f64;
        assert!((emitted / 15.95319 - 1.0).abs() < 1e-14, "{emitted}");
        // An overweight event carries proportionally more, which is what keeps the
        // mean unbiased instead of truncating the tail away.
        assert!(norm.xwgtup(2.5) > norm.xwgtup(1.0));
        assert_eq!(WeightNormalisation::new(1.0, 0.0).xwgtup(1.0), 0.0);
    }
}
