//! The choices a phase-space map leaves open, and how a run settles them.
//!
//! Every choice here is a different parametrisation of the same phase space:
//! the estimator is unbiased under any of them, and what moves is the variance
//! of the weight — the evaluations a run needs to reach an accuracy. Each is a
//! map MadEvent applies, or one this code applies that MadEvent leaves to its
//! adaptive grid. [`MapOptions`] is what a caller
//! asks for, each choice either named or left to the rule that reads the
//! process; [`MapChoices`] is what a run actually integrated under, banked in
//! the artifact so an event sample is drawn from the maps its grids were
//! trained on rather than from whatever the current default happens to be.
//!
//! The rules in [`MapOptions::resolve`] are measurements, not opinions: each
//! `Auto` picks the option that measured best on the process class the rule
//! recognises, and falls back to the option every gated row was banked under
//! where no measurement exists. A rule with no measurement behind it is not
//! written down as a rule.

use serde::{Deserialize, Serialize};

use crate::cuts::{forced_lines, Cuts};
use crate::diagrams::diagram::Diagram;
use crate::ufo::EvaluatedModel;

use super::diagram_channel::{AngleShape, DiagramChannel};

/// How a 2-body split of the decay tree draws its decay angle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitAngle {
    /// Flat in `cos θ` and `φ` against the collision-CM axes — the map every
    /// gated row was banked under, and MadEvent's.
    Isotropic,
    /// Flat in `cos θ*` from the parent's direction of flight, confined to the
    /// angles at which both daughters clear their cut-implied energy floors, on
    /// every split whose parent moves ([`AngleShape::Windowed`]).
    Windowed,
    /// The soft-shaped map `∝ 1/(E₁E₂)` ([`AngleShape::Soft`]) on the splits with a
    /// single massless vector daughter — a gluon or photon emission, where a
    /// splitting kernel is soft-singular — and isotropic elsewhere.
    SoftEmission,
    /// The soft-shaped map on every split whose parent moves.
    SoftAll,
}

/// How a hadronic run draws `τ = ŝ/s` above the cut-implied `τ_min`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TauMap {
    /// `τ = τ_min^(1−u)`, density `∝ 1/τ` — the map every gated hadronic row was
    /// banked under.
    Log,
    /// Density `∝ 1/τ²`, MadEvent's `transpole(pole = −2)` for a hadronic run with
    /// no resonance spanning the whole final state.
    InverseSquare,
}

/// The order a peripheral chain's rungs are drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RungOrder {
    /// Outward from beam 0, the nesting of the diagram's spacelike lines.
    Derived,
    /// The same rungs in reverse, so the chain draws its transfers against the
    /// running remainder from the other end.
    Reversed,
}

/// What a caller asks of the maps: each choice named, or `None` for the rule in
/// [`MapOptions::resolve`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapOptions {
    pub split_angle: Option<SplitAngle>,
    pub tau: Option<TauMap>,
    pub rung_order: Option<RungOrder>,
}

/// The maps a run integrates under, every choice settled. Banked in the
/// artifact; a generator rebuilds its channels from these and nothing else.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapChoices {
    pub split_angle: SplitAngle,
    pub tau: TauMap,
    pub rung_order: RungOrder,
}

/// What the rule needs to know about a process to settle the choices: read off
/// the channels a decomposition builds, not off the process string.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProcessShape {
    /// Splits with a single massless vector daughter whose parent can move, summed
    /// over the channel set. Zero means [`SplitAngle::SoftEmission`] shapes nothing.
    pub soft_emission_splits: usize,
    /// Every split whose parent can move, summed over the channel set: every split
    /// but the root of an all-timelike tree. Zero means no angular map shapes
    /// anything, as on every `2 → 2` process.
    pub moving_splits: usize,
    /// The longest peripheral chain any channel draws.
    pub max_rungs: usize,
    /// Whether any channel's diagram carries a finite-width s-channel resonance
    /// spanning the whole final state, whose peak then sits in the `τ` draw.
    pub whole_state_resonance: bool,
}

impl ProcessShape {
    /// Read the shape off the channels `diagrams` decompose into, built as
    /// [`MapChoices::channel`] builds them but before any map option is applied.
    pub fn of<'d>(
        diagrams: impl IntoIterator<Item = &'d Diagram>,
        model: &EvaluatedModel,
        sqrt_s: f64,
        cuts: &Cuts,
    ) -> Self {
        let floor = cuts.spacelike_floor();
        let mut shape = ProcessShape::default();
        for d in diagrams {
            let channel = DiagramChannel::<f64>::from_diagram_regulated(d, model, sqrt_s, floor);
            let emitters = DiagramChannel::<f64>::massless_vector_slots(d, model);
            shape.soft_emission_splits +=
                channel.splits_selected(&DiagramChannel::<f64>::soft_emission_rule(emitters));
            shape.moving_splits += channel.splits_selected(&|_, _| true);
            shape.max_rungs = shape.max_rungs.max(channel.rung_count());
            shape.whole_state_resonance |=
                DiagramChannel::<f64>::has_whole_state_resonance(d, model);
        }
        shape
    }
}

impl MapChoices {
    /// The maps every artifact written before the choices were banked was
    /// integrated under, and the fallback of every rule with no measurement
    /// behind it.
    pub const LEGACY: MapChoices = MapChoices {
        split_angle: SplitAngle::Isotropic,
        tau: TauMap::Log,
        rung_order: RungOrder::Derived,
    };

    /// One diagram's phase-space channel under these choices: the regulated
    /// decomposition with the cut-implied timelike floors, each line a decay chain
    /// forces on shell confined to its `bwcutoff` window, the chosen angular map
    /// on the splits it selects, and the chosen rung order. The one place a channel
    /// is built for integration, so an integrator and the generator replaying its
    /// grids cannot disagree about the map.
    pub fn channel(
        &self,
        diagram: &Diagram,
        model: &EvaluatedModel,
        sqrt_s: f64,
        cuts: &Cuts,
    ) -> DiagramChannel<f64> {
        let channel = DiagramChannel::<f64>::from_diagram_regulated(
            diagram,
            model,
            sqrt_s,
            cuts.spacelike_floor(),
        )
        .with_timelike_floors(&|slots| cuts.timelike_floor(slots));
        let forced = forced_lines(diagram, model);
        let channel = if forced.is_empty() {
            channel
        } else {
            let bwcutoff = cuts.bwcutoff();
            channel.with_forced_windows(&|slots| {
                let line = forced.iter().find(|l| l.slots == slots)?;
                let (lo, hi) = line.mass_window(bwcutoff)?;
                Some((lo.max(0.0).powi(2), hi * hi))
            })
        };
        let energy_floor = |slots: u64| cuts.energy_floor(slots);
        let emitters = DiagramChannel::<f64>::massless_vector_slots(diagram, model);
        let emission = DiagramChannel::<f64>::soft_emission_rule(emitters);
        let channel = match self.split_angle {
            SplitAngle::Isotropic => channel,
            SplitAngle::Windowed => {
                channel.with_split_angles(&|_, _| Some(AngleShape::Windowed), &energy_floor)
            }
            SplitAngle::SoftEmission => channel.with_split_angles(
                &|l, r| emission(l, r).then_some(AngleShape::Soft),
                &energy_floor,
            ),
            SplitAngle::SoftAll => {
                channel.with_split_angles(&|_, _| Some(AngleShape::Soft), &energy_floor)
            }
        };
        match self.rung_order {
            RungOrder::Derived => channel,
            RungOrder::Reversed => {
                let order: Vec<usize> = (0..channel.rung_count()).rev().collect();
                channel.with_rung_order(&order)
            }
        }
    }

    /// One line naming each choice and whether the rule or the caller made it.
    pub fn describe(&self, asked: &MapOptions) -> String {
        let how = |named: bool| if named { "" } else { " (auto)" };
        format!(
            "split-angle {}{}, tau {}{}, rung-order {}{}",
            self.split_angle.name(),
            how(asked.split_angle.is_some()),
            self.tau.name(),
            how(asked.tau.is_some()),
            self.rung_order.name(),
            how(asked.rung_order.is_some()),
        )
    }
}

impl MapOptions {
    /// Every choice named: what a generator asks for, replaying an artifact.
    pub fn fixed(choices: MapChoices) -> Self {
        MapOptions {
            split_angle: Some(choices.split_angle),
            tau: Some(choices.tau),
            rung_order: Some(choices.rung_order),
        }
    }

    /// Settle every unnamed choice by the rule for `shape`.
    ///
    /// Measured as evaluations the convergence stop needs to reach a χ²-scaled
    /// 0.1%, twenty seeds or more per arm, as a ratio to the map it replaces:
    ///
    /// * `split_angle`: [`SplitAngle::SoftEmission`] where some split has a single
    ///   gluon or photon daughter, else [`SplitAngle::Isotropic`]. `u u~ > g g g`
    ///   0.67 of isotropic; inert — bit-identical draws — everywhere else, which is
    ///   every gated row but `e+ e- > mu+ mu- a`. [`SplitAngle::SoftAll`], the
    ///   shape on every split whose parent moves, measures better still where it
    ///   differs: `p p > l+ l- j` 0.50 ± 0.02, `g u > e+ e- u` 0.85 ± 0.02 (160
    ///   seeds), `g g > g u u~` 1.02 ± 0.09. It is not the rule yet because
    ///   `pp_to_llj_dyn`'s five-seed scatter guard reads 4.17 against its 4.0
    ///   under it, while forty seeds at the gate's own configuration read
    ///   `χ²/dof` 0.91 under either map — a decision about that cell, not the map.
    /// * `tau`: [`TauMap::Log`]. MadEvent's rule — `1/τ²` unless a finite-width
    ///   resonance spans the whole final state, which
    ///   [`ProcessShape::whole_state_resonance`] detects — measures better where it
    ///   differs: `p p > j j` 0.77 ± 0.03, `p p > b b~` 0.65 ± 0.05 in error² ×
    ///   evaluations (it stops on the iteration floor), Drell–Yan 1.06 ± 0.05 the
    ///   other way, `p p > l+ l- j` neutral. It is not the rule yet because the
    ///   dijet event sample's flavour χ² against MadGraph's banked one crosses its
    ///   gate floor on one seed under it, while two samples of our own, one per
    ///   map, agree at χ² 40.5 / 47 — a decision about that cell, not the map.
    /// * `rung_order`: [`RungOrder::Derived`]. The reverse reads 1.01 ± 0.02 on
    ///   `u u~ > g g g`, the one row whose two-rung ladders it moves.
    pub fn resolve(&self, shape: &ProcessShape) -> MapChoices {
        MapChoices {
            split_angle: self
                .split_angle
                .unwrap_or(if shape.soft_emission_splits > 0 {
                    SplitAngle::SoftEmission
                } else {
                    SplitAngle::Isotropic
                }),
            tau: self.tau.unwrap_or(TauMap::Log),
            rung_order: self.rung_order.unwrap_or(RungOrder::Derived),
        }
    }
}

impl SplitAngle {
    /// The flag spelling of the choice.
    pub fn name(self) -> &'static str {
        match self {
            SplitAngle::Isotropic => "isotropic",
            SplitAngle::Windowed => "windowed",
            SplitAngle::SoftEmission => "soft-emission",
            SplitAngle::SoftAll => "soft-all",
        }
    }
}

impl TauMap {
    /// The flag spelling of the choice.
    pub fn name(self) -> &'static str {
        match self {
            TauMap::Log => "log",
            TauMap::InverseSquare => "inverse-square",
        }
    }
}

impl RungOrder {
    /// The flag spelling of the choice.
    pub fn name(self) -> &'static str {
        match self {
            RungOrder::Derived => "derived",
            RungOrder::Reversed => "reversed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rule_shapes_only_where_a_split_can_take_it() {
        let auto = MapOptions::default();
        let two_to_two = ProcessShape::default();
        assert_eq!(auto.resolve(&two_to_two), MapChoices::LEGACY);
        let resonant = ProcessShape {
            whole_state_resonance: true,
            ..ProcessShape::default()
        };
        assert_eq!(auto.resolve(&resonant), MapChoices::LEGACY);
        let with = ProcessShape {
            soft_emission_splits: 2,
            moving_splits: 4,
            max_rungs: 1,
            whole_state_resonance: false,
        };
        assert_eq!(auto.resolve(&with).split_angle, SplitAngle::SoftEmission);
        let pair_only = ProcessShape {
            moving_splits: 2,
            ..ProcessShape::default()
        };
        assert_eq!(auto.resolve(&pair_only).split_angle, SplitAngle::Isotropic);
        assert_eq!(auto.resolve(&with).rung_order, RungOrder::Derived);
    }

    /// The rule reads real processes the way its documentation says: Drell–Yan, a
    /// dijet process and a lepton pair recoiling against a gluon all keep the
    /// legacy maps (the gluon is a leg of the root split, whose parent is at rest,
    /// or of a rung), while a gluon emitted from a moving system is shaped. The shape
    /// behind the pending `τ` rule is pinned too: only Drell–Yan carries a
    /// resonance spanning its final state.
    #[test]
    fn the_rule_reads_real_processes() {
        use crate::cuts::{Cuts, ExternalLeg};
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use crate::runcard::RunCard;
        use crate::ufo::sm::{sm_model, SMRestrict};

        let m = sm_model(SMRestrict::Default);
        let ev = EvaluatedModel::from_model(m.clone());
        let resolve = |process: &str, legs: &[(i32, bool)]| -> (MapChoices, bool) {
            let card = parse_proc_card(&format!("generate {process}"), &ParsingOptions::default())
                .unwrap();
            let diagrams: Vec<Diagram> = generate_from_proc_card(&card, &m)
                .unwrap()
                .into_iter()
                .flat_map(|s| s.diagrams)
                .collect();
            let legs: Vec<ExternalLeg> = legs
                .iter()
                .map(|&(pdg, incoming)| {
                    if incoming {
                        ExternalLeg::incoming(pdg, 0.0)
                    } else {
                        ExternalLeg::outgoing(pdg, 0.0)
                    }
                })
                .collect();
            let cuts = Cuts::compile(&RunCard::default(), &legs).unwrap();
            let shape = ProcessShape::of(&diagrams, &ev, 500.0, &cuts);
            (
                MapOptions::default().resolve(&shape),
                shape.whole_state_resonance,
            )
        };
        let dy = resolve(
            "u u~ > e+ e-",
            &[(2, true), (-2, true), (-11, false), (11, false)],
        );
        assert_eq!(dy, (MapChoices::LEGACY, true));
        let jj = resolve(
            "u u~ > d d~",
            &[(2, true), (-2, true), (1, false), (-1, false)],
        );
        assert_eq!(jj, (MapChoices::LEGACY, false));
        let llj = resolve(
            "u u~ > e+ e- g",
            &[
                (2, true),
                (-2, true),
                (-11, false),
                (11, false),
                (21, false),
            ],
        );
        assert_eq!(llj, (MapChoices::LEGACY, false));
        let ggg = resolve(
            "u u~ > g g g",
            &[(2, true), (-2, true), (21, false), (21, false), (21, false)],
        );
        assert_eq!(ggg.0.split_angle, SplitAngle::SoftEmission);
    }

    #[test]
    fn a_named_choice_overrides_the_rule_and_fixed_names_every_choice() {
        let shape = ProcessShape {
            soft_emission_splits: 2,
            moving_splits: 4,
            max_rungs: 2,
            whole_state_resonance: false,
        };
        let asked = MapOptions {
            split_angle: Some(SplitAngle::Isotropic),
            tau: Some(TauMap::InverseSquare),
            rung_order: None,
        };
        let choices = asked.resolve(&shape);
        assert_eq!(
            choices,
            MapChoices {
                split_angle: SplitAngle::Isotropic,
                tau: TauMap::InverseSquare,
                rung_order: RungOrder::Derived,
            }
        );
        assert_eq!(
            MapOptions::fixed(choices).resolve(&ProcessShape::default()),
            choices
        );
        assert_eq!(
            choices.describe(&asked),
            "split-angle isotropic, tau inverse-square, rung-order derived (auto)"
        );
    }
}
