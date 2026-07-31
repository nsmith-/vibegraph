//! Assembling an event record out of what the generator produces.
//!
//! An accepted point supplies the momenta and a weight; the per-event selections
//! supply a helicity combination and a colour flow; the run card's scale
//! prescription supplies `μR`, `μF` and the couplings they imply. Everything else
//! on the record — the PDG codes, the masses, the incoming/outgoing statuses, the
//! mother pointers and the colour-line labels — is fixed by the subprocess and is
//! resolved once, by [`SubprocessRecord`].

use crate::coupling::scales::EventScales;
use crate::helas::eval::AmplitudeEvaluator;
use crate::ufo::{EvaluatedModel, UFOModel};

use super::record::{LheEvent, LheParticle, STATUS_INCOMING, STATUS_OUTGOING};
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
    /// `(colour, anticolour)` line labels per leg, per flow.
    flows: crate::helas::color::flow_tags::ColorFlowTags,
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
            flows: evaluator.color_flow_tags().clone(),
        })
    }

    /// The same compiled subprocess read on reordered, relabelled external legs:
    /// leg `i` of the result is leg `order[i]` of `self`, carrying PDG code
    /// `pdg[i]`.
    ///
    /// A hadron-collider event needs this because one compiled amplitude serves
    /// several concrete flavour assignments and both beam orderings. The colour
    /// flows and the pole masses travel with the legs and only the codes change:
    /// the flavours sharing an amplitude are the ones whose legs carry the same
    /// masses, and exchanging the two beams exchanges their momenta along with
    /// everything else the record says about them.
    ///
    /// The incoming/outgoing split is `self`'s, so a permutation that moves a leg
    /// across it is refused.
    pub fn relabelled(&self, order: &[usize], pdg: &[i32]) -> Result<Self, LhefError> {
        let n_ext = self.n_ext();
        let well_formed = order.len() == n_ext
            && pdg.len() == n_ext
            && order.iter().enumerate().all(|(i, &leg)| {
                leg < n_ext && (i < self.n_in) == (leg < self.n_in) && !order[..i].contains(&leg)
            });
        if !well_formed {
            return Err(LhefError::LegOrder {
                order: order.to_vec(),
                n_ext,
            });
        }
        let flows = self
            .flows
            .permuted(order)
            .ok_or_else(|| LhefError::LegOrder {
                order: order.to_vec(),
                n_ext,
            })?;
        Ok(SubprocessRecord {
            pdg: pdg.to_vec(),
            mass: order.iter().map(|&leg| self.mass[leg]).collect(),
            n_in: self.n_in,
            flows,
        })
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
        // no mother in the record.
        let outgoing_mothers = [1, self.n_in as i32];
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
        })
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
