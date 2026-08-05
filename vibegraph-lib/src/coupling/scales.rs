//! Per-event renormalisation and factorisation scales, as MadGraph chooses them.
//!
//! # Two scales, three numbers
//!
//! A run card asks for one renormalisation scale `μR` and, separately, a
//! factorisation scale *per beam*: `Source/run.inc` carries `q2fact(1)` and
//! `q2fact(2)`, and `SubProcesses/setscales.f` guards them with independent
//! `fixed_fac_scale1` / `fixed_fac_scale2` flags. [`EventScales`] therefore
//! reports three numbers, not two.
//!
//! # Where the numbers come from
//!
//! `SubProcesses/cuts.f:1220` calls `set_ren_scale` only when
//! `fixed_ren_scale` is false, and `set_fac_scale` only when at least one beam is
//! dynamic — so a fixed scale is the run card's value untouched, and in
//! particular **`scalefact` never multiplies a fixed scale**. It multiplies
//! `set_ren_scale`'s result once, and `set_fac_scale` then squares that already
//! multiplied value into `q2fact`, so a dynamic `μF` carries exactly one factor
//! too. The one exception is the clustering branch, where `reweight.f` applies it
//! itself; see [`ScaleChoice::scales`].
//!
//! `dynamical_scale_choice` 1–5 are the closed forms of `setscales.f`
//! ([`DynamicalChoice`]). Choice `0` is a user-edited Fortran function and has no
//! meaning here. The **default, `-1`, is not in that file at all**: it is the
//! clustering scale of `SubProcesses/reweight.f:551 setclscales`, which runs the
//! event through the kT clustering of `SubProcesses/cluster.f` and reads the
//! scale off the resulting 2 → 2 core. That path lives in
//! [`cluster`](super::cluster) and is reached through
//! [`ScaleChoice::cluster_scales`], which the caller supplies the process's
//! channel forests to.
//!
//! # Refusal, not fallback
//!
//! Every configuration outside the implemented set returns [`ScaleError`]. A
//! wrong scale is a smooth shift of the cross section with no other symptom, so a
//! plausible guess — `√ŝ` for an unreached branch, say — is strictly worse than a
//! stopped run.

use thiserror::Error;

use crate::coupling::cluster::graph::{ChannelSet, ColorTable, MergeTable};
use crate::coupling::cluster::kt::{Channel, ClusterSettings};
use crate::coupling::cluster::setclscales::{setclscales, JetMemo, ScaleRefusal, ScaleSettings};
use crate::runcard::RunCard;

/// The scales for one event: MadGraph's `scale` and `sqrt(q2fact(1:2))`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EventScales {
    /// Renormalisation scale `μR`, the argument of the running coupling.
    pub mu_r: f64,
    /// Factorisation scale per beam, `mu_f[0]` for beam 1.
    pub mu_f: [f64; 2],
}

/// MadGraph's `dynamical_scale_choice`, for the values that name a scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DynamicalChoice {
    /// `-1`: cluster the event to a 2 → 2 core and take that core's scale.
    Clustered,
    /// `1`: total transverse energy of the final state, `Σ E_T`.
    TotalTransverseEnergy,
    /// `2`: sum of final-state transverse masses, `Σ √((E + p_z)(E − p_z))`.
    SumTransverseMass,
    /// `3`: half the sum of final-state transverse masses.
    HalfSumTransverseMass,
    /// `4`: partonic energy `√ŝ`.
    PartonicEnergy,
    /// `5`: invariant mass of the first incoming leg, for decay processes.
    DecayingMass,
}

impl DynamicalChoice {
    /// The run-card integer, or `None` for `0` (a user-edited Fortran function)
    /// and for values `setscales.f` stops on.
    pub fn from_i64(choice: i64) -> Option<Self> {
        match choice {
            -1 => Some(DynamicalChoice::Clustered),
            1 => Some(DynamicalChoice::TotalTransverseEnergy),
            2 => Some(DynamicalChoice::SumTransverseMass),
            3 => Some(DynamicalChoice::HalfSumTransverseMass),
            4 => Some(DynamicalChoice::PartonicEnergy),
            5 => Some(DynamicalChoice::DecayingMass),
            _ => None,
        }
    }

    pub fn as_i64(self) -> i64 {
        match self {
            DynamicalChoice::Clustered => -1,
            DynamicalChoice::TotalTransverseEnergy => 1,
            DynamicalChoice::SumTransverseMass => 2,
            DynamicalChoice::HalfSumTransverseMass => 3,
            DynamicalChoice::PartonicEnergy => 4,
            DynamicalChoice::DecayingMass => 5,
        }
    }
}

/// One event's kinematics, in the frame the scale is defined in.
#[derive(Clone, Copy, Debug)]
pub struct ScaleEvent<'a> {
    /// Incoming momenta `[E, px, py, pz]`, in the lab frame — the clustering's
    /// beam measure is a transverse mass about the beam axis.
    pub incoming: [[f64; 4]; 2],
    /// Outgoing momenta, matrix-element level — resonances that a Les Houches
    /// record lists as intermediate are not among them.
    pub outgoing: &'a [[f64; 4]],
}

/// The channel data the clustering path reads, alongside one event's momenta.
///
/// The cluster scale is not a function of the momenta and the process alone: the
/// merge table is selected by the integration channel's QCD order, the resonance
/// tagging reads that channel's own timelike lines, and the jet-count memo is
/// keyed on it. A caller that sampled a channel names it.
#[derive(Clone, Copy, Debug)]
pub struct ClusterInput<'a> {
    /// The process's channel forests, as
    /// [`configs`](super::cluster::configs) derives them from its diagrams.
    pub set: &'a ChannelSet,
    pub colors: &'a ColorTable,
    /// The channel being integrated, from `1`.
    pub this_config: usize,
    /// The subprocess of the group, from `1`.
    pub iproc: usize,
    /// The merge tables for `this_config`'s coupling order, when the caller
    /// keeps them. They are a function of the channel set and that order alone,
    /// so a caller clustering many events builds them once; `None` builds them
    /// from `set` on each call.
    pub tables: Option<&'a [MergeTable]>,
}

#[derive(Debug, Error, PartialEq)]
pub enum ScaleError {
    #[error(
        "run card selects dynamical_scale_choice = {choice}: setscales.f implements 0 as a \
         user-edited Fortran function and stops on every other value"
    )]
    UnsupportedChoice { choice: i64 },
    #[error(
        "run card selects ickkw = {ickkw}, xqcut = {xqcut}: MadGraph then runs its clustering \
         even behind a closed-form scale choice, and multiplies q2fact by scalefact a second \
         time on the way through"
    )]
    UnsupportedMatching { ickkw: i64, xqcut: f64 },
    #[error("scalefact must be positive, got {scalefact}")]
    NonPositiveScaleFact { scalefact: f64 },
    #[error("fixed {name} must be positive, got {value}")]
    NonPositiveFixedScale { name: &'static str, value: f64 },
    #[error(
        "dynamical_scale_choice = -1 is a clustering scale: the caller must supply the process's \
         channel forests and the integration channel through ScaleChoice::cluster_scales"
    )]
    MissingChannels,
    #[error("the event carries no cluster scale: {0:?}")]
    Clustering(ScaleRefusal),
    #[error(
        "dynamical_scale_choice = -1 with one factorisation scale fixed and the other dynamic: \
         reweight.f then fills the dynamic one from the fixed one in a way no reference run \
         exercises"
    )]
    MixedFixedFactorisationScales,
    #[error("scale choice {choice} needs at least {needs} outgoing momenta, got {got}")]
    TooFewOutgoing {
        choice: i64,
        needs: usize,
        got: usize,
    },
    #[error("scale choice {choice} gives a non-positive scale {value} on these momenta")]
    DegenerateKinematics { choice: i64, value: f64 },
    #[error(
        "run card selects dynamical_scale_choice = {choice}: the integration path evaluates the \
         clustered scale only, and the closed forms for 1-5 are computed nowhere a cross section \
         reads them"
    )]
    UnhonouredScaleChoice { choice: i64 },
}

/// A run card's scale prescription, compiled once.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScaleChoice {
    choice: DynamicalChoice,
    /// `Some(μR)` when `fixed_ren_scale` is set.
    fixed_ren: Option<f64>,
    /// `Some(μF)` per beam when that beam's `fixed_fac_scale` is set.
    fixed_fac: [Option<f64>; 2],
    scalefact: f64,
    /// A parton density on each beam, which is what the clustering's measures
    /// switch on and what puts the factorisation floor under that beam.
    beam_has_pdf: [bool; 2],
    /// The run-card constants the clustering itself reads.
    cluster: ClusterSettings,
    /// `xmtcentral`, the floor `setclscales` puts under the central vertex.
    xmtc: f64,
    pdfwgt: bool,
}

impl ScaleChoice {
    /// Compile the scale prescription from a run card.
    ///
    /// `fixed_fac_scale1` / `fixed_fac_scale2` take precedence over the older
    /// single `fixed_fac_scale`, which fills in for whichever of them the card
    /// leaves alone (`banner.py:post_set_fixed_fac_scale`). MadGraph tracks which
    /// names the card actually set; a card that sets all three inconsistently
    /// gets a warning there and the per-beam values, while the disjunction below
    /// would keep the single flag — the only combination where the two readings
    /// differ, and one MadGraph itself calls out.
    pub fn from_run_card(card: &RunCard) -> Result<Self, ScaleError> {
        let compiled = Self::compile(card)?;
        // The choice is only read where a scale is derived from the event: both
        // `scales` and `cluster_scales` short-circuit on `is_fully_fixed` before
        // reaching it, so on a card that fixes every scale the value provably
        // cannot change a number and refusing it would be a refusal the code
        // cannot justify. Everywhere else the closed forms for 1-5 are computed
        // nowhere a cross section reads them, and a scale prescription with no
        // oracle behind it is refused rather than approximated.
        if !compiled.is_fully_fixed() && compiled.choice != DynamicalChoice::Clustered {
            return Err(ScaleError::UnhonouredScaleChoice {
                choice: compiled.choice.as_i64(),
            });
        }
        Ok(compiled)
    }

    /// [`from_run_card`](Self::from_run_card) without its refusal of the
    /// closed-form scale choices. Those formulas are transcribed from
    /// `setscales.f` and keep their unit tests through here, so the arithmetic
    /// stays pinned for whoever wires them to a cross section; no run card
    /// reaches them.
    fn compile(card: &RunCard) -> Result<Self, ScaleError> {
        let choice_int = card.int("dynamical_scale_choice");
        let choice = DynamicalChoice::from_i64(choice_int)
            .ok_or(ScaleError::UnsupportedChoice { choice: choice_int })?;

        let ickkw = card.int("ickkw");
        let xqcut = card.float("xqcut");
        if ickkw != 0 || xqcut > 0.0 {
            return Err(ScaleError::UnsupportedMatching { ickkw, xqcut });
        }

        let scalefact = card.float("scalefact");
        if !(scalefact > 0.0) {
            return Err(ScaleError::NonPositiveScaleFact { scalefact });
        }

        let fixed_ren = card
            .fixed_ren_scale
            .then_some(card.scale)
            .map(|value| positive("scale", value))
            .transpose()?;

        let fac_flags = [
            card.fixed_fac_scale || card.get("fixed_fac_scale1").expect("known").as_bool(),
            card.fixed_fac_scale || card.get("fixed_fac_scale2").expect("known").as_bool(),
        ];
        let fac_values = [card.dsqrt_q2fact1, card.dsqrt_q2fact2];
        let names = ["dsqrt_q2fact1", "dsqrt_q2fact2"];
        let mut fixed_fac = [None, None];
        for beam in 0..2 {
            if fac_flags[beam] {
                fixed_fac[beam] = Some(positive(names[beam], fac_values[beam])?);
            }
        }

        let beam_has_pdf = [card.lpp1 != 0, card.lpp2 != 0];
        Ok(ScaleChoice {
            choice,
            fixed_ren,
            fixed_fac,
            scalefact,
            beam_has_pdf,
            cluster: ClusterSettings {
                hadronic: beam_has_pdf[0] || beam_has_pdf[1],
                d_parameter: card.float("d"),
                bwcutoff: card.float("bwcutoff"),
                small_width_treatment: 1e-6,
            },
            xmtc: card.float("xmtcentral"),
            pdfwgt: card.get("pdfwgt").expect("known").as_bool(),
        })
    }

    /// The clustering's own run-card constants, for a caller that drives
    /// [`setclscales`](crate::coupling::cluster::setclscales::setclscales)
    /// itself.
    pub fn cluster_settings(&self) -> &ClusterSettings {
        &self.cluster
    }

    pub fn choice(&self) -> DynamicalChoice {
        self.choice
    }

    pub fn scalefact(&self) -> f64 {
        self.scalefact
    }

    /// Every scale is a run-card constant, so no event kinematics are consulted.
    /// A fixed-beam run can hoist the coupling out of its integration loop on the
    /// strength of this.
    pub fn is_fully_fixed(&self) -> bool {
        self.fixed_ren.is_some() && self.fixed_fac.iter().all(Option::is_some)
    }

    /// A [`ClusterInput`] must be supplied with each event, through
    /// [`ScaleChoice::cluster_scales`].
    pub fn needs_channels(&self) -> bool {
        self.choice == DynamicalChoice::Clustered && !self.is_fully_fixed()
    }

    /// The scales for one event.
    ///
    /// The dynamic value is computed once and shared: `set_fac_scale` calls
    /// `set_ren_scale` for it (`setscales.f:181`), so a run with a fixed `μR` and
    /// a dynamic `μF` still evaluates the dynamical choice — the fixed value
    /// simply does not come from it.
    pub fn scales(&self, event: &ScaleEvent<'_>) -> Result<EventScales, ScaleError> {
        if self.is_fully_fixed() {
            return Ok(EventScales {
                mu_r: self.fixed_ren.expect("fully fixed"),
                mu_f: [
                    self.fixed_fac[0].expect("fully fixed"),
                    self.fixed_fac[1].expect("fully fixed"),
                ],
            });
        }

        let dynamic = match self.choice {
            DynamicalChoice::Clustered => return Err(ScaleError::MissingChannels),
            other => {
                let mu = self.scalefact * self.closed_form(other, event)?;
                Dynamic {
                    mu_r: mu,
                    mu_f: [mu, mu],
                }
            }
        };

        Ok(EventScales {
            mu_r: self.fixed_ren.unwrap_or(dynamic.mu_r),
            mu_f: [
                self.fixed_fac[0].unwrap_or(dynamic.mu_f[0]),
                self.fixed_fac[1].unwrap_or(dynamic.mu_f[1]),
            ],
        })
    }

    /// The scales `reweight.f`'s `setclscales` reads off the clustered event.
    ///
    /// This is the whole of `dynamical_scale_choice = -1`: the event is
    /// clustered down to a `2 → 2` core against the merge graph the channel
    /// forests imply, and the scales are read off the vertices a colour line
    /// passes through. Every branch of the two scale formulas, the two rewrites
    /// before them, and the one power of `scalefact` each of the three results
    /// carries live in
    /// [`setclscales`](crate::coupling::cluster::setclscales::setclscales); what
    /// is here is the run card's side of the call and the squared-to-linear
    /// conversion of the factorisation scales.
    ///
    /// The jet memo starts empty on every event. MadGraph keeps it per process
    /// directory across a whole run, so its first event of a channel is the one
    /// that fills it; starting empty reproduces exactly that event's behaviour
    /// and makes the scale a function of the event rather than of the order
    /// events were generated in.
    pub fn cluster_scales(
        &self,
        event: &ScaleEvent<'_>,
        input: &ClusterInput<'_>,
    ) -> Result<EventScales, ScaleError> {
        if self.fixed_fac[0].is_some() != self.fixed_fac[1].is_some() {
            return Err(ScaleError::MixedFixedFactorisationScales);
        }
        if self.is_fully_fixed() {
            return self.scales(event);
        }
        let mut p: Vec<[f64; 4]> = Vec::with_capacity(input.set.n_external);
        p.extend_from_slice(&event.incoming);
        p.extend_from_slice(event.outgoing);
        if p.len() != input.set.n_external {
            return Err(ScaleError::TooFewOutgoing {
                choice: -1,
                needs: input.set.n_external - 2,
                got: event.outgoing.len(),
            });
        }
        let settings = ScaleSettings {
            scalefact: self.scalefact,
            fixed_ren: self.fixed_ren.is_some(),
            fixed_fac: [self.fixed_fac[0].is_some(), self.fixed_fac[1].is_some()],
            beam_has_pdf: self.beam_has_pdf,
            // A card with matching switched on is refused when the prescription
            // is compiled, so the clustering never sees one here.
            ickkw: 0,
            xqcut: 0.0,
            xmtc: self.xmtc,
            pdfwgt: self.pdfwgt,
        };
        let built;
        let tables = match input.tables {
            Some(tables) => tables,
            None => {
                built = input.set.merge_tables(input.this_config);
                &built
            }
        };
        let channel = Channel {
            set: input.set,
            table: &tables[input.iproc - 1],
            colors: input.colors,
            this_config: input.this_config,
            iproc: input.iproc,
        };
        let incoming = (
            self.fixed_ren.unwrap_or(0.0),
            [
                self.fixed_fac[0].map_or(0.0, |mu| mu * mu),
                self.fixed_fac[1].map_or(0.0, |mu| mu * mu),
            ],
        );
        let scales = setclscales(
            &channel,
            &self.cluster,
            &settings,
            &p,
            &mut JetMemo::default(),
            false,
            &[],
            incoming,
            false,
        )
        .map_err(ScaleError::Clustering)?;
        Ok(EventScales {
            mu_r: scales.mu_r,
            mu_f: [scales.q2fact[0].sqrt(), scales.q2fact[1].sqrt()],
        })
    }

    /// `set_ren_scale` for `dynamical_scale_choice` 1–5, before `scalefact`.
    fn closed_form(
        &self,
        choice: DynamicalChoice,
        event: &ScaleEvent<'_>,
    ) -> Result<f64, ScaleError> {
        let out = event.outgoing;
        let value = match choice {
            DynamicalChoice::TotalTransverseEnergy => out.iter().map(transverse_energy).sum(),
            DynamicalChoice::SumTransverseMass => out.iter().map(transverse_mass).sum(),
            DynamicalChoice::HalfSumTransverseMass => {
                out.iter().map(transverse_mass).sum::<f64>() / 2.0
            }
            DynamicalChoice::PartonicEnergy => {
                let total = add(&event.incoming[0], &event.incoming[1]);
                mg_dot(&total, &total).max(0.0).sqrt()
            }
            DynamicalChoice::DecayingMass => {
                let p = &event.incoming[0];
                mg_dot(p, p).max(0.0).sqrt()
            }
            DynamicalChoice::Clustered => unreachable!("handled by the caller"),
        };
        if !(value > 0.0) {
            return Err(ScaleError::DegenerateKinematics {
                choice: choice.as_i64(),
                value,
            });
        }
        Ok(value)
    }
}

struct Dynamic {
    mu_r: f64,
    mu_f: [f64; 2],
}

fn positive(name: &'static str, value: f64) -> Result<f64, ScaleError> {
    if value > 0.0 {
        Ok(value)
    } else {
        Err(ScaleError::NonPositiveFixedScale { name, value })
    }
}

/// `et` (`Source/kin_functions.f:186`): `E · p_T / |p⃗|`, zero for a leg with no
/// transverse momentum.
fn transverse_energy(p: &[f64; 4]) -> f64 {
    let pt = (p[1] * p[1] + p[2] * p[2]).sqrt();
    if pt > 0.0 {
        p[0] * pt / (pt * pt + p[3] * p[3]).sqrt()
    } else {
        0.0
    }
}

/// `√((E + p_z)(E − p_z))`, spelled as `setscales.f` spells it: the factored form
/// keeps the cancellation out of the difference of squares.
fn transverse_mass(p: &[f64; 4]) -> f64 {
    ((p[0] + p[3]) * (p[0] - p[3])).max(0.0).sqrt()
}

fn add(a: &[f64; 4], b: &[f64; 4]) -> [f64; 4] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}

/// `dot` (`Source/kin_functions.f:588`), including the clamp it applies when the
/// Minkowski product is small against the Euclidean one: a would-be massless leg
/// whose components leave a `1e-7` relative residue is returned as exactly
/// massless rather than as that residue.
fn mg_dot(p1: &[f64; 4], p2: &[f64; 4]) -> f64 {
    let dot = p1[0] * p2[0] - p1[1] * p2[1] - p1[2] * p2[2] - p1[3] * p2[3];
    if dot.abs() < 1e-6 {
        let euclidean = (p1[0] * p2[0] + p1[1] * p2[1] + p1[2] * p2[2] + p1[3] * p2[3])
            .max(f64::from(1e-99f32));
        if dot / euclidean < 1e-6 {
            return 0.0;
        }
    }
    dot
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coupling::cluster::configs::derive_channels;
    use crate::coupling::cluster::graph::MergeTablesByOrder;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::ufo::particles::ParticleId;
    use crate::ufo::sm::{sm_model, SMRestrict};
    use crate::ufo::EvaluatedModel;

    fn card(extra: &str) -> RunCard {
        RunCard::parse(extra).expect("run card")
    }

    /// MadGraph's default beams are protons; the fixed-beam cases below have to
    /// say so, since the clustering's measures switch on it.
    fn partonic_card(extra: &str) -> RunCard {
        card(&format!("0 = lpp1\n0 = lpp2\n{extra}"))
    }

    fn event(outgoing: &[[f64; 4]]) -> ScaleEvent<'_> {
        ScaleEvent {
            incoming: [[250.0, 0.0, 0.0, 250.0], [250.0, 0.0, 0.0, -250.0]],
            outgoing,
        }
    }

    /// The channel forests and colour table of a fully concrete process, which
    /// is what the clustering branch has to be handed.
    struct Process {
        derived: crate::coupling::cluster::configs::DerivedChannels,
        colors: ColorTable,
    }

    fn process(spec: &str) -> Process {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let parsed = parse_proc_card(&format!("generate {spec}"), &ParsingOptions::default())
            .expect("proc card parses");
        let sets = generate_from_proc_card(&parsed, model.as_ref()).expect("enumerate");
        let set = sets
            .iter()
            .find(|s| !s.diagrams.is_empty())
            .expect("a non-empty subprocess");
        let externals: Vec<ParticleId> = set
            .particles_in
            .iter()
            .chain(set.particles_out.iter())
            .map(|name| model.particle_id(name).expect("external in model"))
            .collect();
        Process {
            derived: derive_channels(
                &set.diagrams,
                &externals,
                set.particles_in.len(),
                model.as_ref(),
                &evaluated,
            )
            .expect("channel forests"),
            colors: ColorTable::new(
                model
                    .particles
                    .values()
                    .map(|p| (p.pdg_code, p.color))
                    .collect::<Vec<(i64, i32)>>(),
                4,
            ),
        }
    }

    impl Process {
        fn input(&self, config: usize) -> ClusterInput<'_> {
            ClusterInput {
                set: &self.derived.set,
                colors: &self.colors,
                this_config: config,
                iproc: 1,
                tables: None,
            }
        }
    }

    /// On the processes the clustering actually runs on, the hoisted tables are
    /// the ones the per-event build produces, channel by channel.
    ///
    /// The keying itself — one table set per coupling order rather than per
    /// channel — is pinned in `graph.rs` on a set that carries two orders; every
    /// process here carries one, which is the common case and not the
    /// interesting one.
    #[test]
    fn hoisted_merge_tables_reproduce_the_per_event_build() {
        for spec in ["u u~ > u u~", "e+ e- > e+ e-", "g g > g g", "u u~ > e+ e- g"] {
            let proc = process(spec);
            let set = &proc.derived.set;
            let hoisted = MergeTablesByOrder::build(set);
            for config in 1..=set.configs.len() {
                assert_eq!(
                    hoisted.of(config),
                    set.merge_tables(config).as_slice(),
                    "{spec} channel {config}"
                );
            }
        }
    }

    /// The cluster scales themselves are unmoved by where the tables came from.
    #[test]
    fn the_hoisted_tables_leave_the_cluster_scale_alone() {
        let choice = ScaleChoice::from_run_card(&partonic_card("")).expect("compiled");
        let out = [[250.0, 30.0, 0.0, -100.0], [250.0, -30.0, 0.0, 100.0]];
        let quarks = process("u u~ > u u~");
        let hoisted = MergeTablesByOrder::build(&quarks.derived.set);
        for config in 1..=quarks.derived.set.configs.len() {
            let mut input = quarks.input(config);
            let built = choice.cluster_scales(&event(&out), &input);
            input.tables = Some(hoisted.of(config));
            assert_eq!(choice.cluster_scales(&event(&out), &input), built);
        }
    }

    /// A fixed scale is the card's value, and `scalefact` does not touch it —
    /// `cuts.f` never calls `set_ren_scale` on that path.
    #[test]
    fn scalefact_leaves_fixed_scales_alone() {
        let choice = ScaleChoice::from_run_card(&card(
            "True = fixed_ren_scale\nTrue = fixed_fac_scale\n\
             80.0 = scale\n70.0 = dsqrt_q2fact1\n60.0 = dsqrt_q2fact2\n2.0 = scalefact\n",
        ))
        .expect("compiled");
        assert!(choice.is_fully_fixed());
        assert!(!choice.needs_channels());
        let out = [[125.0, 10.0, 0.0, 124.0], [125.0, -10.0, 0.0, -124.0]];
        let scales = choice.scales(&event(&out)).expect("scales");
        assert_eq!(scales.mu_r, 80.0);
        assert_eq!(scales.mu_f, [70.0, 60.0]);
    }

    /// `scalefact` multiplies a dynamic scale exactly once, and the factorisation
    /// scale inherits it through `tempscale**2` rather than picking up a second.
    ///
    /// Built through [`ScaleChoice::compile`] because `from_run_card` refuses a
    /// closed-form choice on a card that does not fix every scale; this pins the
    /// transcribed arithmetic, not a reachable configuration.
    #[test]
    fn scalefact_multiplies_a_dynamic_scale_once() {
        let choice = ScaleChoice::compile(&card("3 = dynamical_scale_choice\n2.0 = scalefact"))
            .expect("compiled");
        let out = [[125.0, 0.0, 0.0, 0.0], [125.0, 0.0, 0.0, 0.0]];
        let scales = choice.scales(&event(&out)).expect("scales");
        assert_eq!(scales.mu_r, 2.0 * 125.0);
        assert_eq!(scales.mu_f, [250.0, 250.0]);
    }

    /// A fixed `μR` with a dynamic `μF`: `set_fac_scale` calls `set_ren_scale`
    /// for its own value regardless of `fixed_ren_scale`.
    ///
    /// Built through [`ScaleChoice::compile`] for the same reason as the test
    /// above — a card selecting choice 4 without fixing every scale is refused.
    #[test]
    fn a_fixed_renormalisation_scale_leaves_the_factorisation_scale_dynamic() {
        let choice = ScaleChoice::compile(&card(
            "True = fixed_ren_scale\n91.188 = scale\n4 = dynamical_scale_choice\n",
        ))
        .expect("compiled");
        let out = [[250.0, 0.0, 0.0, 0.0], [250.0, 0.0, 0.0, 0.0]];
        let scales = choice.scales(&event(&out)).expect("scales");
        assert_eq!(scales.mu_r, 91.188);
        assert_eq!(scales.mu_f, [500.0, 500.0]);
    }

    /// The closed-form scale choices are refused where they would be read, and
    /// only there.
    ///
    /// `ScaleChoice::closed_form` computes all five, and nothing on the
    /// integration path evaluates them: every per-event scale goes through the
    /// clustering. A prescription with no reference run behind it would produce a
    /// plausible, smooth, wrong cross section with nothing to notice it by, so it
    /// is named rather than approximated.
    ///
    /// The `fully_fixed` half is the accuracy of the gate, not leniency: both
    /// scale entry points short-circuit on `is_fully_fixed` before the choice is
    /// read, so there the value cannot reach a number.
    ///
    /// This cannot say whether the closed forms are *correct* — nothing here
    /// claims they are, which is the reason for the refusal.
    #[test]
    fn an_unhonoured_scale_choice_is_refused() {
        for choice in [1, 2, 3, 4, 5] {
            let text = format!("{choice} = dynamical_scale_choice");
            assert_eq!(
                ScaleChoice::from_run_card(&card(&text)),
                Err(ScaleError::UnhonouredScaleChoice { choice }),
                "choice {choice} on a dynamical card"
            );

            // The same choice where every scale is a run-card constant is
            // accepted, and yields those constants.
            let fixed = format!(
                "{choice} = dynamical_scale_choice\nTrue = fixed_ren_scale\n80.0 = scale\n\
                 True = fixed_fac_scale\n70.0 = dsqrt_q2fact1\n60.0 = dsqrt_q2fact2\n"
            );
            let compiled = ScaleChoice::from_run_card(&card(&fixed))
                .unwrap_or_else(|e| panic!("choice {choice} on a fully fixed card: {e}"));
            assert!(compiled.is_fully_fixed());
            let out = [[125.0, 10.0, 0.0, 124.0], [125.0, -10.0, 0.0, -124.0]];
            let scales = compiled.scales(&event(&out)).expect("scales");
            assert_eq!(scales.mu_r, 80.0);
            assert_eq!(scales.mu_f, [70.0, 60.0]);
        }

        // The clustered default is never touched by the gate.
        assert_eq!(
            ScaleChoice::from_run_card(&card("-1 = dynamical_scale_choice"))
                .expect("the clustering default is honoured")
                .choice(),
            DynamicalChoice::Clustered
        );
        assert_eq!(
            ScaleChoice::from_run_card(&card(""))
                .expect("an empty card is the clustering default")
                .choice(),
            DynamicalChoice::Clustered
        );
    }

    /// The unimplemented choices are named, not approximated.
    #[test]
    fn unsupported_choices_are_refused() {
        for choice in [0, 6, -2] {
            let text = format!("{choice} = dynamical_scale_choice");
            assert_eq!(
                ScaleChoice::from_run_card(&card(&text)),
                Err(ScaleError::UnsupportedChoice { choice })
            );
        }
        assert_eq!(
            ScaleChoice::from_run_card(&card("1 = ickkw")),
            Err(ScaleError::UnsupportedMatching {
                ickkw: 1,
                xqcut: 0.0
            })
        );
    }

    /// `-1` reached through the generic entry point is an error, not a guess:
    /// the clustering needs the process's channels and that call site has none.
    #[test]
    fn the_clustering_branch_demands_channels() {
        let choice = ScaleChoice::from_run_card(&card("")).expect("compiled");
        assert_eq!(choice.choice(), DynamicalChoice::Clustered);
        assert!(choice.needs_channels());
        let out = [[125.0, 10.0, 0.0, 124.0], [125.0, -10.0, 0.0, -124.0]];
        assert_eq!(
            choice.scales(&event(&out)),
            Err(ScaleError::MissingChannels)
        );
    }

    /// `cluster.f` inflates a beam–leg candidate whose legs point in opposite
    /// directions by `1 + 1e-6`, so that a leg following the beam it came from
    /// wins an otherwise exact tie. It reaches the scale only when a colour line
    /// runs from beam to beam and every allowed candidate is crossed — which is
    /// what `u ū → u ū` does, and what note 22 measured as its `250.0001` row.
    #[test]
    fn the_cluster_tie_break_moves_the_scale() {
        let choice = ScaleChoice::from_run_card(&partonic_card("")).expect("compiled");
        let out = [[250.0, 30.0, 0.0, -100.0], [250.0, -30.0, 0.0, 100.0]];
        let quarks = process("u u~ > u u~");
        let crossed = choice
            .cluster_scales(&event(&out), &quarks.input(1))
            .expect("scales");
        assert!(
            (crossed.mu_r / (250.0 * (1.0f64 + 1e-6).sqrt()) - 1.0).abs() < 1e-12,
            "{}",
            crossed.mu_r
        );
        assert_eq!(crossed.mu_f, [crossed.mu_r, crossed.mu_r]);

        // Uncrossed, the inflation never fires and the scale is the beams' own.
        let aligned = [[250.0, 30.0, 0.0, 100.0], [250.0, -30.0, 0.0, -100.0]];
        let straight = choice
            .cluster_scales(&event(&aligned), &quarks.input(1))
            .expect("scales");
        assert_eq!(straight.mu_r, 250.0);
        assert!(crossed.mu_r > straight.mu_r);
    }

    /// The negative control: with colourless beams the colour line stops before
    /// it reaches them, so the inflated candidate drops out of the scale
    /// altogether and the same crossed kinematics gives the uninflated value.
    #[test]
    fn colourless_beams_keep_the_tie_break_out_of_the_scale() {
        let choice = ScaleChoice::from_run_card(&partonic_card("")).expect("compiled");
        let out = [[250.0, 30.0, 0.0, -100.0], [250.0, -30.0, 0.0, 100.0]];
        let leptons = process("e+ e- > e+ e-");
        let scales = choice
            .cluster_scales(&event(&out), &leptons.input(1))
            .expect("scales");
        assert_eq!(scales.mu_r, 250.0);
    }

    /// The beam measure has no beam to measure against without a parton density,
    /// and says so by switching from transverse mass to energy — the reason a
    /// fixed-beam 2 → 2 sits at a constant scale while the same process at a
    /// hadron collider does not.
    #[test]
    fn the_beam_measure_follows_the_beam_configuration() {
        let out = [[250.0, 30.0, 0.0, 200.0], [250.0, -30.0, 0.0, -200.0]];
        let gluons = process("g g > g g");
        let partonic = ScaleChoice::from_run_card(&partonic_card("")).expect("compiled");
        assert_eq!(
            partonic
                .cluster_scales(&event(&out), &gluons.input(1))
                .expect("scales")
                .mu_r,
            250.0
        );
        let hadronic = ScaleChoice::from_run_card(&card("1 = lpp1\n1 = lpp2")).expect("compiled");
        let mt2 = (250.0f64 + 200.0) * (250.0 - 200.0);
        assert_eq!(
            hadronic
                .cluster_scales(&event(&out), &gluons.input(1))
                .expect("scales")
                .mu_r,
            mt2.sqrt()
        );
    }

    /// On 3.7.1 every reachable branch multiplies `μR` and each `μF` by exactly
    /// one power of `scalefact`. 3.5.7 built beam 2's
    /// factorisation scale from an already-scaled beam 1 in the branch where no
    /// colour line reaches the beams, and picked the factor up twice there; this
    /// is that branch, and it now carries one power like every other.
    #[test]
    fn scalefact_reaches_every_scale_exactly_once() {
        let one = ScaleChoice::from_run_card(&partonic_card("")).expect("compiled");
        let three =
            ScaleChoice::from_run_card(&partonic_card("3.0 = scalefact")).expect("compiled");
        let out = [[250.0, 10.0, 0.0, 0.0], [250.0, -10.0, 0.0, 0.0]];
        // A colour-singlet final state off colourless beams: `jcentral` is zero
        // on both, which is the branch the two versions disagreed in.
        let leptons = process("e+ e- > mu+ mu-");
        let base = one
            .cluster_scales(&event(&out), &leptons.input(1))
            .expect("scales");
        let scaled = three
            .cluster_scales(&event(&out), &leptons.input(1))
            .expect("scales");
        assert_eq!(scaled.mu_r, 3.0 * base.mu_r);
        assert_eq!(scaled.mu_f[0], 3.0 * base.mu_f[0]);
        assert_eq!(scaled.mu_f[1], 3.0 * base.mu_f[1]);
        assert_eq!(scaled.mu_f[0], scaled.mu_f[1]);
    }

    /// A single `fixed_fac_scale` fills both beams; the per-beam names override it.
    #[test]
    fn per_beam_factorisation_flags_override_the_single_one() {
        let both = ScaleChoice::from_run_card(&card(
            "True = fixed_fac_scale\n30.0 = dsqrt_q2fact1\n40.0 = dsqrt_q2fact2",
        ))
        .expect("compiled");
        assert_eq!(both.fixed_fac, [Some(30.0), Some(40.0)]);
        let beam1 = ScaleChoice::from_run_card(&card(
            "True = fixed_fac_scale1\n30.0 = dsqrt_q2fact1\n40.0 = dsqrt_q2fact2",
        ))
        .expect("compiled");
        assert_eq!(beam1.fixed_fac, [Some(30.0), None]);
    }

    /// One beam fixed and the other dynamic reaches a corner of `reweight.f`
    /// where beam 2 never picks up `scalefact` at all; it is refused rather than
    /// guessed, and no banked run exercises it.
    #[test]
    fn a_half_fixed_factorisation_scale_is_refused_under_clustering() {
        let choice = ScaleChoice::from_run_card(&partonic_card(
            "True = fixed_fac_scale1\n30.0 = dsqrt_q2fact1\n-1 = dynamical_scale_choice",
        ))
        .expect("compiled");
        let out = [[250.0, 10.0, 0.0, 0.0], [250.0, -10.0, 0.0, 0.0]];
        let leptons = process("e+ e- > mu+ mu-");
        assert_eq!(
            choice.cluster_scales(&event(&out), &leptons.input(1)),
            Err(ScaleError::MixedFixedFactorisationScales)
        );
    }
}
