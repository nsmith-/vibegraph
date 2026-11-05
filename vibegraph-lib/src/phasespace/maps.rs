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

/// How a hadronic run draws `τ = ŝ/s` over `[τ_min, τ_max]`: from the
/// cut-implied floor up to one, or to the ceiling a decay-chain window on the
/// whole final state sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TauMap {
    /// `τ = τ_min^(1−u) τ_max^u`, density `∝ 1/τ`: the rule's choice where a resonance
    /// spans the whole final state, and the map of every artifact written before
    /// the choices were banked.
    Log,
    /// Density `∝ 1/τ²`, MadEvent's `transpole(pole = −2)` for a hadronic run with
    /// no resonance spanning the whole final state.
    InverseSquare,
    /// The channel-averaged density of MadEvent's per-channel `τ` maps: each
    /// `(group, diagram)` channel whose diagram carries a finite-width resonance
    /// spanning the whole final state, with its pole inside `[τ_min, τ_max]`,
    /// contributes a Breit–Wigner in `τ` at that pole (MadEvent's
    /// `transpole(pole = M²/s, width = MΓ/s)`, at the model's own width), and
    /// every other channel contributes the [`TauMap::Log`] density. Each channel
    /// weighs `1/N`. With no pole in range it is [`TauMap::Log`] bit for bit.
    ///
    /// The rule's choice where such a resonance is narrow
    /// ([`ProcessShape::narrow_whole_state_resonance`]): a peak the log map gives
    /// almost none of its unit interval, which a VEGAS grid on that coordinate
    /// cannot find.
    BreitWigner,
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
    /// Whether one of those resonances has its pole inside the `τ` range and a
    /// peak narrower than [`NARROW_TAU_SHARE`] of the log map's unit interval
    /// ([`narrow_in_tau`]).
    pub(crate) narrow_whole_state_resonance: bool,
}

/// The share of the logarithmic `τ` map's unit interval below which a
/// whole-state resonance's peak counts as narrow: there, fewer than one draw in a
/// thousand lands within a width of the pole.
///
/// The measured rows bracket it. The `Z` of Drell–Yan sits at about `5e-3`
/// (`2Γ/M = 0.055` over `ln(τ_max/τ_min) ≈ 11` at `m_ll > 50 GeV`, 13 TeV) and the
/// log map resolves it, as it does a 1 MeV `Vz` at 3.5 TeV confined to its decay
/// chain's `bwcutoff` window (about `1e-2` of the bounded range). The same `Vz`
/// without the window, in `p p > h z`, sits at `7e-8`: under the log map five
/// seeds at 100k × 8 scatter by 17% (147640 to 224658 pb), at in-run `χ²/dof` up
/// to 52.
pub(crate) const NARROW_TAU_SHARE: f64 = 1e-3;

/// Whether a whole-state resonance of `mass` and `width` is narrow in the `τ`
/// draw over `[tau_min, tau_max]` at hadronic invariant `s`: its pole `M²/s` lies
/// strictly inside the range, and its peak's share of the logarithmic map's unit
/// interval, `(2Γ/M)/ln(τ_max/τ_min)` to first order in `Γ/M`, is below
/// [`NARROW_TAU_SHARE`]. A range with no positive floor has no log map and
/// nothing is narrow in it.
pub(crate) fn narrow_in_tau(mass: f64, width: f64, s: f64, tau_min: f64, tau_max: f64) -> bool {
    tau_min > 0.0
        && tau_max > tau_min
        && pole_in_tau(mass, s, tau_min, tau_max)
        && 2.0 * width / mass / (tau_max / tau_min).ln() < NARROW_TAU_SHARE
}

/// Whether a pole of `mass` lies strictly inside the `τ` range at hadronic
/// invariant `s`: `τ_min < M²/s < τ_max`.
pub(crate) fn pole_in_tau(mass: f64, s: f64, tau_min: f64, tau_max: f64) -> bool {
    let pole = mass * mass / s;
    pole > tau_min && pole < tau_max
}

/// The `[τ_min, τ_max]` a hadronic run's `τ` map draws over at hadronic
/// invariant `s`: from the cut-implied `ŝ_min/s` up to one, or to `ŝ_max/s`
/// where a decay-chain window on the whole final state bounds `ŝ`
/// ([`Cuts::shat_max`]), and never below the floor. At `τ_max = 1` the bounded
/// logarithmic and inverse-square maps are their unbounded forms bit for bit
/// (`1^u = 1`, `1/1 = 1`).
pub(crate) fn tau_range(cuts: &Cuts, s: f64) -> (f64, f64) {
    let tau_min = cuts.shat_min() / s;
    (tau_min, (cuts.shat_max() / s).min(1.0).max(tau_min))
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
        let s = sqrt_s * sqrt_s;
        let (tau_min, tau_max) = tau_range(cuts, s);
        let mut shape = ProcessShape::default();
        for d in diagrams {
            let channel = DiagramChannel::<f64>::from_diagram_regulated(d, model, sqrt_s, floor);
            let emitters = DiagramChannel::<f64>::massless_vector_slots(d, model);
            shape.soft_emission_splits +=
                channel.splits_selected(&DiagramChannel::<f64>::soft_emission_rule(emitters));
            shape.max_rungs = shape.max_rungs.max(channel.rung_count());
            shape.resonant_pair_splits += channel.resonant_pair_splits();
            if let Some((mass, width)) = DiagramChannel::<f64>::whole_state_resonance(d, model) {
                shape.whole_state_resonance = true;
                shape.narrow_whole_state_resonance |=
                    narrow_in_tau(mass, width, s, tau_min, tau_max);
            }
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
    ///   propagators spanning the pair, stays on it. Where such a peak is too
    ///   narrow for the log map to find
    ///   ([`ProcessShape::narrow_whole_state_resonance`], [`NARROW_TAU_SHARE`]),
    ///   the rule takes [`TauMap::BreitWigner`]: `p p > h z` through a 1 MeV
    ///   `Vz` at 3.5 TeV, at 100k × 8, scatters by 17% over five seeds under the
    ///   log map; under the Breit–Wigner mixture 25 seeds read 185698 ± 23 pb at
    ///   seed `χ²/dof` 0.78, 3.2e-4 below MadEvent 3.7.1, inside the bound its
    ///   small-width treatment sets.
    /// * `rung_order`: [`RungOrder::Derived`]. The reverse reads 1.01 ± 0.02 on
    ///   `u u~ > g g g`, the one row whose two-rung ladders it moves.
    /// * `pair_order`: [`PairOrder::Symmetric`]. The ordered draw leaves half of
    ///   an `H → ZZ*` peak in a Breit–Wigner tail. On `b b~ > c c~ e+ e- mu+ mu-`
    ///   (20 paired seeds, 600k and 1.2M evaluations, 8 iterations) points there
    ///   carried most of the largest single-point weights, and the per-seed
    ///   spread rose with budget, ×1.23. Under the symmetric order it falls,
    ///   ×0.78 (1.888e-9 pb at 1.2M against 3.330e-9), with no point on that half
    ///   above 1e-3 of its iteration. Where both daughters can sit on their poles
    ///   the two orders' densities nearly coincide and the choice moves little.
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
            tau: self.tau.unwrap_or(if shape.narrow_whole_state_resonance {
                TauMap::BreitWigner
            } else if shape.whole_state_resonance {
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
            TauMap::BreitWigner => "breit-wigner",
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
            narrow_whole_state_resonance: false,
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

    /// A narrow whole-state resonance takes the Breit–Wigner mixture, ahead of the
    /// log map a wide one keeps, and a named choice still overrides it.
    #[test]
    fn a_narrow_whole_state_resonance_takes_the_breit_wigner_mixture() {
        let narrow = ProcessShape {
            whole_state_resonance: true,
            narrow_whole_state_resonance: true,
            ..ProcessShape::default()
        };
        assert_eq!(
            MapOptions::default().resolve(&narrow).tau,
            TauMap::BreitWigner
        );
        let asked = MapOptions {
            tau: Some(TauMap::Log),
            ..MapOptions::default()
        };
        assert_eq!(asked.resolve(&narrow).tau, TauMap::Log);
        assert_eq!(TauMap::BreitWigner.name(), "breit-wigner");
    }

    /// The narrowness test reads the pole's position and the peak's share of the
    /// log map's unit interval, on both sides of [`NARROW_TAU_SHARE`].
    #[test]
    fn narrowness_is_the_peak_s_share_of_the_log_map_inside_the_range() {
        let s = 13000.0f64.powi(2);
        let (tau_min, tau_max) = (50.0f64.powi(2) / s, 1.0);
        let l = (tau_max / tau_min).ln();
        // The Z of Drell–Yan: wide in the draw.
        assert!(!narrow_in_tau(91.1876, 2.4952, s, tau_min, tau_max));
        // Widths straddling the threshold at M = 3500 GeV.
        let edge = NARROW_TAU_SHARE * l * 3500.0 / 2.0;
        assert!(narrow_in_tau(3500.0, 0.99 * edge, s, tau_min, tau_max));
        assert!(!narrow_in_tau(3500.0, 1.01 * edge, s, tau_min, tau_max));
        assert!(narrow_in_tau(3500.0, 1e-3, s, tau_min, tau_max));
        // A pole outside the range, below it or above it, is not in the draw.
        assert!(!narrow_in_tau(40.0, 1e-3, s, tau_min, tau_max));
        assert!(!narrow_in_tau(
            3500.0,
            1e-3,
            s,
            tau_min,
            3000.0f64.powi(2) / s
        ));
        // No positive floor, no log map.
        assert!(!narrow_in_tau(3500.0, 1e-3, s, 0.0, tau_max));
    }

    /// On a real process the rule reads the model's own width: Drell–Yan at the
    /// Standard Model's `Γ_Z` keeps the log map, and the same process with a
    /// 1 keV `Z` takes the Breit–Wigner mixture.
    #[test]
    fn the_rule_reads_the_width_of_a_real_resonance() {
        use crate::cuts::{Cuts, ExternalLeg};
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use crate::runcard::RunCard;
        use crate::ufo::slha::ParamCard;
        use crate::ufo::sm::{sm_model, SMRestrict};

        let m = sm_model(SMRestrict::Default);
        let card = parse_proc_card("generate u u~ > e+ e-", &ParsingOptions::default()).unwrap();
        let diagrams: Vec<Diagram> = generate_from_proc_card(&card, &m)
            .unwrap()
            .into_iter()
            .flat_map(|s| s.diagrams)
            .collect();
        let legs = [
            ExternalLeg::incoming(2, 0.0),
            ExternalLeg::incoming(-2, 0.0),
            ExternalLeg::outgoing(-11, 0.0),
            ExternalLeg::outgoing(11, 0.0),
        ];
        let rc = RunCard::parse("  50.0 = mmll\n").expect("run card");
        let cuts = Cuts::compile(&rc, &legs).unwrap();
        let shape_at = |ev: &EvaluatedModel| ProcessShape::of(&diagrams, ev, 13000.0, &cuts);
        let wide = shape_at(&EvaluatedModel::from_model(m.clone()));
        assert!(wide.whole_state_resonance && !wide.narrow_whole_state_resonance);
        assert_eq!(MapOptions::default().resolve(&wide).tau, TauMap::Log);
        let pc: ParamCard = "DECAY 23 1e-6\n".parse().unwrap();
        let narrow =
            shape_at(&EvaluatedModel::from_model_card(m.clone(), &pc).expect("the card binds"));
        assert!(narrow.narrow_whole_state_resonance);
        assert_eq!(
            MapOptions::default().resolve(&narrow).tau,
            TauMap::BreitWigner
        );
    }

    #[test]
    fn a_named_choice_overrides_the_rule_and_fixed_names_every_choice() {
        let shape = ProcessShape {
            soft_emission_splits: 2,
            max_rungs: 2,
            resonant_pair_splits: 1,
            whole_state_resonance: false,
            narrow_whole_state_resonance: false,
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
