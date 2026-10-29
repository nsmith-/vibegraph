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
//! recognises, and keeps the [`MapChoices::LEGACY`] option where no measurement
//! favours another. A rule with no measurement behind it is not
//! written down as a rule.

use serde::{Deserialize, Serialize};

use crate::cuts::{forced_lines, Cuts};
use crate::diagrams::diagram::Diagram;
use crate::ufo::EvaluatedModel;

use super::diagram_channel::{AngleShape, DiagramChannel};

/// How a 2-body split of the decay tree draws its decay angle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitAngle {
    /// Flat in `cos θ` and `φ` against the collision-CM axes: MadEvent's map, and
    /// that of every artifact written before the choices were banked.
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
    /// `τ = τ_min^(1−u)`, density `∝ 1/τ`: the rule's choice where a resonance
    /// spans the whole final state, and the map of every artifact written before
    /// the choices were banked.
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

/// The order a split whose two daughters both carry a finite-width resonance
/// draws their invariants in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PairOrder {
    /// The left daughter first, on its Breit–Wigner, and the right over the range
    /// the left leaves: the map of every artifact written before the choice was
    /// banked. Below the two poles' sum (`H → ZZ*`) it covers only the half of
    /// the parent's peak with the left daughter on shell.
    Ordered,
    /// Either daughter first with probability ½, the density the mean of the
    /// two orders' ([`DiagramChannel::with_symmetric_pairs`]).
    Symmetric,
}

/// What a caller asks of the maps: each choice named, or `None` for the rule in
/// [`MapOptions::resolve`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapOptions {
    pub split_angle: Option<SplitAngle>,
    pub tau: Option<TauMap>,
    pub rung_order: Option<RungOrder>,
    pub pair_order: Option<PairOrder>,
}

/// The maps a run integrates under, every choice settled. Banked in the
/// artifact; a generator rebuilds its channels from these and nothing else.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapChoices {
    pub(crate) split_angle: SplitAngle,
    pub(crate) tau: TauMap,
    pub(crate) rung_order: RungOrder,
    pub(crate) pair_order: PairOrder,
}

/// What the rule needs to know about a process to settle the choices: read off
/// the channels a decomposition builds, not off the process string.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProcessShape {
    /// Splits with a single massless vector daughter whose parent can move, summed
    /// over the channel set. Zero means [`SplitAngle::SoftEmission`] shapes nothing.
    pub(crate) soft_emission_splits: usize,
    /// The longest peripheral chain any channel draws.
    pub(crate) max_rungs: usize,
    /// Splits whose two daughters both carry a finite-width resonance, summed
    /// over the channel set: the splits [`PairOrder`] decides.
    pub(crate) resonant_pair_splits: usize,
    /// Whether any channel's diagram carries a finite-width s-channel resonance
    /// spanning the whole final state, whose peak then sits in the `τ` draw.
    pub(crate) whole_state_resonance: bool,
}

impl ProcessShape {
    /// Read the shape off the channels `diagrams` decompose into, built as
    /// [`MapChoices::channel`] builds them but before any map option is applied.
    pub(crate) fn of<'d>(
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
            shape.max_rungs = shape.max_rungs.max(channel.rung_count());
            shape.resonant_pair_splits += channel.resonant_pair_splits();
            shape.whole_state_resonance |=
                DiagramChannel::<f64>::has_whole_state_resonance(d, model);
        }
        shape
    }
}

impl MapChoices {
    /// The maps every artifact written before the choices were banked was
    /// integrated under.
    pub const LEGACY: MapChoices = MapChoices {
        split_angle: SplitAngle::Isotropic,
        tau: TauMap::Log,
        rung_order: RungOrder::Derived,
        pair_order: PairOrder::Ordered,
    };

    /// One diagram's phase-space channel under these choices: the regulated
    /// decomposition with the cut-implied timelike floors, each line a decay chain
    /// forces on shell confined to its `bwcutoff` window, the chosen angular map
    /// on the splits it selects, the chosen rung order and the chosen pair order.
    /// The one place a channel
    /// is built for integration, so an integrator and the generator replaying its
    /// grids cannot disagree about the map.
    pub(crate) fn channel(
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
        let channel = match self.rung_order {
            RungOrder::Derived => channel,
            RungOrder::Reversed => {
                let order: Vec<usize> = (0..channel.rung_count()).rev().collect();
                channel.with_rung_order(&order)
            }
        };
        match self.pair_order {
            PairOrder::Ordered => channel,
            PairOrder::Symmetric => channel.with_symmetric_pairs(),
        }
    }

    /// One line naming each choice and whether the rule or the caller made it.
    pub(crate) fn describe(&self, asked: &MapOptions) -> String {
        let how = |named: bool| if named { "" } else { " (auto)" };
        format!(
            "split-angle {}{}, tau {}{}, rung-order {}{}, pair-order {}{}",
            self.split_angle.name(),
            how(asked.split_angle.is_some()),
            self.tau.name(),
            how(asked.tau.is_some()),
            self.rung_order.name(),
            how(asked.rung_order.is_some()),
            self.pair_order.name(),
            how(asked.pair_order.is_some()),
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
            pair_order: Some(choices.pair_order),
        }
    }

    /// Settle every unnamed choice by the rule for `shape`.
    ///
    /// Measured as evaluations the convergence stop needs to reach a χ²-scaled
    /// 0.1%, twenty seeds or more per arm, as a ratio to the map it replaces:
    ///
    /// * `split_angle`: [`SplitAngle::SoftAll`], the soft-shaped map on every
    ///   split whose parent moves. `p p > l+ l- j` 0.50 ± 0.02 of isotropic,
    ///   `g u > e+ e- u` 0.85 ± 0.02 (160 seeds), `g g > g u u~` 1.02 ± 0.09 of
    ///   [`SplitAngle::SoftEmission`]; on `u u~ > g g g`, where every moving split
    ///   has a gluon daughter, it is the soft-emission map, 0.67 of isotropic. A
    ///   `2 → 2` process has no moving split, so every choice draws the same points.
    /// * `tau`: MadEvent's rule, [`TauMap::InverseSquare`] unless a finite-width
    ///   resonance spans the whole final state
    ///   ([`ProcessShape::whole_state_resonance`]), where its peak sits in the `τ`
    ///   draw and [`TauMap::Log`] keeps it. `p p > j j` 0.77 ± 0.03,
    ///   `p p > b b~` 0.65 ± 0.05 in error² × evaluations (it stops on the
    ///   iteration floor); Drell–Yan reads 1.06 ± 0.05 the other way, which the
    ///   resonance exception keeps; `p p > l+ l- j` is neutral, 1.03 ± 0.05. The
    ///   rule reads the whole process, not each channel as MadEvent does: one
    ///   diagram with a whole-state resonance keeps every channel on the log map,
    ///   so `p p > b b~ QCD=2`, whose electroweak diagrams carry `Z` and `h`
    ///   propagators spanning the pair, stays on it.
    /// * `rung_order`: [`RungOrder::Derived`]. The reverse reads 1.01 ± 0.02 on
    ///   `u u~ > g g g`, the one row whose two-rung ladders it moves.
    /// * `pair_order`: [`PairOrder::Symmetric`]. The ordered draw leaves half of
    ///   an `H → ZZ*` peak in a Breit–Wigner tail; on `b b~ > c c~ e+ e- mu+ mu-`
    ///   points there carried most of the largest single-point weights of the
    ///   20-seed budget ladder. Where both daughters can sit on their poles the
    ///   two orders' densities nearly coincide and the choice moves little.
    ///
    /// The ratios are evaluations measured on the M3 Max host off the 2026-09-23
    /// code, twenty seeds per arm at the banked rows' own cards, with the map
    /// asked for by name. The gate cells were re-read under these rules on
    /// 2026-10-17: `pp_to_jj`'s event sample at minimum `p` 5.6e-2 against its
    /// 1e-4 floor, and the five-seed scatter guards of `pp_to_llj_dyn` and
    /// `pp_to_llj_fixed` at `χ²/dof` 1.03 and 1.39 against their 4.0, the latter
    /// at 0.92 over eighty seeds under either map.
    pub fn resolve(&self, shape: &ProcessShape) -> MapChoices {
        MapChoices {
            split_angle: self.split_angle.unwrap_or(SplitAngle::SoftAll),
            tau: self.tau.unwrap_or(if shape.whole_state_resonance {
                TauMap::Log
            } else {
                TauMap::InverseSquare
            }),
            rung_order: self.rung_order.unwrap_or(RungOrder::Derived),
            pair_order: self.pair_order.unwrap_or(PairOrder::Symmetric),
        }
    }
}

impl SplitAngle {
    /// The flag spelling of the choice.
    pub(crate) fn name(self) -> &'static str {
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
    pub(crate) fn name(self) -> &'static str {
        match self {
            TauMap::Log => "log",
            TauMap::InverseSquare => "inverse-square",
        }
    }
}

impl RungOrder {
    /// The flag spelling of the choice.
    pub(crate) fn name(self) -> &'static str {
        match self {
            RungOrder::Derived => "derived",
            RungOrder::Reversed => "reversed",
        }
    }
}

impl PairOrder {
    /// The flag spelling of the choice.
    pub(crate) fn name(self) -> &'static str {
        match self {
            PairOrder::Ordered => "ordered",
            PairOrder::Symmetric => "symmetric",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rule_reads_the_whole_state_resonance_and_shapes_every_split() {
        let auto = MapOptions::default();
        let flipped = MapChoices {
            split_angle: SplitAngle::SoftAll,
            tau: TauMap::InverseSquare,
            rung_order: RungOrder::Derived,
            pair_order: PairOrder::Symmetric,
        };
        assert_eq!(auto.resolve(&ProcessShape::default()), flipped);
        let resonant = ProcessShape {
            whole_state_resonance: true,
            ..ProcessShape::default()
        };
        assert_eq!(
            auto.resolve(&resonant),
            MapChoices {
                tau: TauMap::Log,
                ..flipped
            }
        );
        let with = ProcessShape {
            soft_emission_splits: 2,
            max_rungs: 1,
            resonant_pair_splits: 3,
            whole_state_resonance: false,
        };
        assert_eq!(auto.resolve(&with), flipped);
    }

    /// The rule reads real processes the way its documentation says: only
    /// Drell–Yan carries a resonance spanning its final state and keeps the log
    /// `τ` map, and every process takes the soft-shaped angle whether or not it has
    /// a single-gluon split (`u u~ > g g g` has; the lepton pair recoiling against a
    /// gluon has none, its gluon a leg of the root split or of a rung).
    #[test]
    fn the_rule_reads_real_processes() {
        use crate::cuts::{Cuts, ExternalLeg};
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use crate::runcard::RunCard;
        use crate::ufo::sm::{sm_model, SMRestrict};

        let m = sm_model(SMRestrict::Default);
        let ev = EvaluatedModel::from_model(m.clone());
        let resolve = |process: &str, legs: &[(i32, bool)]| -> (MapChoices, ProcessShape) {
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
            (MapOptions::default().resolve(&shape), shape)
        };
        let choices = |tau| MapChoices {
            split_angle: SplitAngle::SoftAll,
            tau,
            rung_order: RungOrder::Derived,
            pair_order: PairOrder::Symmetric,
        };
        let dy = resolve(
            "u u~ > e+ e-",
            &[(2, true), (-2, true), (-11, false), (11, false)],
        );
        assert_eq!(dy.0, choices(TauMap::Log));
        assert!(dy.1.whole_state_resonance);
        let jj = resolve(
            "u u~ > d d~",
            &[(2, true), (-2, true), (1, false), (-1, false)],
        );
        assert_eq!(jj.0, choices(TauMap::InverseSquare));
        assert!(!jj.1.whole_state_resonance);
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
        assert_eq!(llj.0, choices(TauMap::InverseSquare));
        assert_eq!(
            (llj.1.whole_state_resonance, llj.1.soft_emission_splits),
            (false, 0)
        );
        let ggg = resolve(
            "u u~ > g g g",
            &[(2, true), (-2, true), (21, false), (21, false), (21, false)],
        );
        assert_eq!(ggg.0, choices(TauMap::InverseSquare));
        assert!(ggg.1.soft_emission_splits > 0);
        assert_eq!(dy.1.resonant_pair_splits, 0);
        // The s-channel `γ*/Z → W⁺W⁻` diagrams: two W pairs under one split.
        let four = resolve(
            "u u~ > e+ ve mu- vm~",
            &[
                (2, true),
                (-2, true),
                (-11, false),
                (12, false),
                (13, false),
                (-14, false),
            ],
        );
        assert!(four.1.resonant_pair_splits > 0);
    }

    #[test]
    fn a_named_choice_overrides_the_rule_and_fixed_names_every_choice() {
        let shape = ProcessShape {
            soft_emission_splits: 2,
            max_rungs: 2,
            resonant_pair_splits: 1,
            whole_state_resonance: false,
        };
        let asked = MapOptions {
            split_angle: Some(SplitAngle::Isotropic),
            tau: Some(TauMap::InverseSquare),
            rung_order: None,
            pair_order: Some(PairOrder::Ordered),
        };
        let choices = asked.resolve(&shape);
        assert_eq!(
            choices,
            MapChoices {
                split_angle: SplitAngle::Isotropic,
                tau: TauMap::InverseSquare,
                rung_order: RungOrder::Derived,
                pair_order: PairOrder::Ordered,
            }
        );
        assert_eq!(
            MapOptions::fixed(choices).resolve(&ProcessShape::default()),
            choices
        );
        assert_eq!(
            choices.describe(&asked),
            "split-angle isotropic, tau inverse-square, rung-order derived (auto), pair-order \
             ordered"
        );
    }
}
