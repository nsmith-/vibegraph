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
//! scale off the resulting 2 → 2 core. Reproducing that clustering in general is
//! a project of its own; what is implemented here is the set of topologies in
//! which it collapses to a closed form, declared by the caller as a
//! [`ClusterTopology`] and refused otherwise.
//!
//! # Refusal, not fallback
//!
//! Every configuration outside the implemented set returns [`ScaleError`]. A
//! wrong scale is a smooth shift of the cross section with no other symptom, so a
//! plausible guess — `√ŝ` for an unimplemented clustering, say — is strictly
//! worse than a stopped run.

use thiserror::Error;

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

/// The facts about a process's topology that MadGraph's clustering consults.
///
/// `cluster.f` chooses which pair of legs to merge by minimising a jet measure
/// over the pairs *some diagram allows* (`findmt`), and `setclscales` then walks
/// the resulting tree asking `isqcd` of the lines it passes and `isjet` of the
/// legs. Those four questions are the whole of what the closed forms below
/// depend on, so they are declared rather than guessed: a run whose topology is
/// misdeclared fails the comparison against MadGraph, where a run whose topology
/// was inferred from PDG codes alone could silently be one branch out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClusterTopology {
    /// How the beams reach the final state in the process's diagrams, which is
    /// what decides the shape of the clustering tree.
    pub beam_connections: BeamConnections,
    /// The incoming legs carry colour (`isqcd` of the beam flavours). This is
    /// what decides whether a colour line runs from beam to beam, and with it
    /// which vertex `setclscales` reads the scale off.
    pub coloured_beams: bool,
    /// The propagator the final state clusters onto carries colour. Consulted
    /// only for an s-channel-only two-body process, where it selects between the
    /// s-channel mass and the geometric mean of the transverse masses.
    pub coloured_central_line: bool,
    /// Every outgoing leg counts as a jet (`isjet`: `|pdg| ≤ maxjetflavor`, or a
    /// gluon). Consulted only where the clustering's tie-break below actually
    /// changes the answer.
    pub jet_legs: bool,
}

/// Whether any diagram carries a t-channel propagator from a beam into the final
/// state.
///
/// This is what fixes the *shape* of the clustering tree, and it is a statement
/// about the diagrams rather than about vertices: a beam does not have to meet an
/// external leg, only a subtree. In `e⁺e⁻ → μ⁺μ⁻τ⁺τ⁻`, no vertex joins an
/// electron to a muon, yet the `ZZ` diagram hangs both `Z`s off the electron
/// line, so the clustering does reach an initial-state merge — two steps in — and
/// the final state never collapses to one propagator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BeamConnections {
    /// Every diagram is an s-channel tree: the final state clusters among itself
    /// all the way down, whatever its multiplicity, leaving a single propagator
    /// joining the beams.
    SChannelOnly,
    /// Some diagram carries a t-channel propagator. For a two-body final state
    /// `two_body_pairs[b][l]` says which `(beam, leg)` pairs a diagram lets merge,
    /// which is all the clustering can do there. For a longer final state the
    /// tree depends on the merge order and there is no closed form; the mask is
    /// then not consulted.
    TChannel { two_body_pairs: [[bool; 2]; 2] },
}

/// One event's kinematics, plus the topology declaration the clustering branch
/// needs.
#[derive(Clone, Copy, Debug)]
pub struct ScaleEvent<'a> {
    /// Incoming momenta `[E, px, py, pz]`, in the frame the scale is defined in
    /// (the lab frame: `djb` measures transverse mass against the beam axis).
    pub incoming: [[f64; 4]; 2],
    /// Outgoing momenta, matrix-element level — resonances that a Les Houches
    /// record lists as intermediate are not among them.
    pub outgoing: &'a [[f64; 4]],
    /// Required by [`DynamicalChoice::Clustered`] unless every scale is fixed.
    pub topology: Option<ClusterTopology>,
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
        "dynamical_scale_choice = -1 is a clustering scale: the caller must declare the \
         topology facts the clustering consults (ClusterTopology)"
    )]
    MissingTopology,
    #[error(
        "dynamical_scale_choice = -1 outside the topologies whose cluster scale collapses to a \
         closed form ({reason}); the general case needs the kT clustering of cluster.f"
    )]
    ClusteringNotDegenerate { reason: String },
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
    /// A PDF on either beam, which is what `djb` switches its measure on.
    hadronic_beams: bool,
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

        Ok(ScaleChoice {
            choice,
            fixed_ren,
            fixed_fac,
            scalefact,
            hadronic_beams: card.lpp1 != 0 || card.lpp2 != 0,
        })
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

    /// A [`ClusterTopology`] must be supplied with each event.
    pub fn needs_topology(&self) -> bool {
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
            DynamicalChoice::Clustered => {
                if self.fixed_fac[0].is_some() != self.fixed_fac[1].is_some() {
                    return Err(ScaleError::MixedFixedFactorisationScales);
                }
                let clustered = self.clustered(event)?;
                let mu = self.scalefact * clustered.q2.sqrt();
                Dynamic {
                    mu_r: mu,
                    mu_f: [
                        mu,
                        if clustered.beam2_from_beam1 {
                            self.scalefact * mu
                        } else {
                            mu
                        },
                    ],
                }
            }
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

    /// MadGraph's beam jet measure `djb` (`Source/kin_functions.f:397`): the
    /// squared scale one leg carries with respect to the beams.
    ///
    /// With a PDF on either beam this is the transverse mass squared. With
    /// neither there is no beam direction worth measuring against — the routine
    /// says so in a commented-out error — and it returns `E²`, which is why a
    /// fixed-beam 2 → 2 has a constant cluster scale while the same process at a
    /// hadron collider does not.
    fn djb(&self, p: &[f64; 4]) -> f64 {
        if self.hadronic_beams {
            (p[0] - p[3]) * (p[0] + p[3])
        } else {
            p[0].max(0.0) * p[0].max(0.0)
        }
    }

    /// The clustering scale of `setclscales`, for the topologies where it
    /// collapses to a closed form.
    ///
    /// Two families reach one, and both end up reading a single squared scale off
    /// the clustering's central vertex.
    ///
    /// **A t-channel reaches a two-body final state.** The beam–leg measure
    /// `djb(leg)` always beats the final-state measure there — `dj` carries a
    /// `2(cosh Δη − cos Δφ)` factor that is at least `4` for a back-to-back pair,
    /// and a Breit-Wigner mother is measured by the pair mass instead, which is
    /// larger still — so a beam–leg pair merges first and the leftover leg carries
    /// the central scale. Requiring the two legs to share a `djb` is what makes
    /// the rest independent of *which* leg merged, and of where `setclscales`
    /// decides the colour line stops.
    ///
    /// **Every diagram is an s-channel tree.** The final state merges among
    /// itself down to one propagator joining the beams, at any multiplicity. A
    /// colour-charged propagator triggers the `mt2last` override at
    /// `reweight.f:1012`, which replaces the propagator's own scale by the
    /// geometric mean of the daughters' transverse masses; otherwise the
    /// propagator's `djb` stands.
    ///
    /// The `1 + 1e-6` in the first family is `cluster.f`'s tie-break between
    /// beam–leg candidates that are otherwise numerically equal: a candidate
    /// whose legs point in opposite directions is inflated so the same-direction
    /// one wins. It survives into the scale only when a colour line reaches both
    /// beams, and then only in the seventh digit — which is exactly why it is
    /// modelled rather than rounded away, since ten of `u ū → u ū`'s ten thousand
    /// banked events sit on it.
    fn clustered(&self, event: &ScaleEvent<'_>) -> Result<Clustered, ScaleError> {
        let topology = event.topology.ok_or(ScaleError::MissingTopology)?;
        let out = event.outgoing;
        if out.len() < 2 {
            return Err(ScaleError::TooFewOutgoing {
                choice: -1,
                needs: 2,
                got: out.len(),
            });
        }
        let refuse = |reason: String| Err(ScaleError::ClusteringNotDegenerate { reason });

        if let BeamConnections::TChannel { two_body_pairs } = topology.beam_connections {
            if out.len() != 2 {
                return refuse(format!(
                    "a t-channel propagator into a final state of {} legs: the clustering tree \
                     then depends on the merge order",
                    out.len()
                ));
            }
            let beam_leg: Vec<(usize, usize)> = (0..2)
                .flat_map(|beam| (0..2).map(move |leg| (beam, leg)))
                .filter(|&(beam, leg)| two_body_pairs[beam][leg])
                .collect();
            if beam_leg.is_empty() {
                return refuse(
                    "a t-channel propagator was declared but no beam-leg pair with it".to_string(),
                );
            }
            let d = [self.djb(&out[0]), self.djb(&out[1])];
            if !(d[0] > 0.0)
                || !(d[1] > 0.0)
                || (d[0] - d[1]).abs() > BALANCE_HEADROOM * (d[0] + d[1])
            {
                return refuse(format!(
                    "the two outgoing legs carry different beam measures, djb = {d:?}: which leg \
                     merges first, and where the colour line stops, then both change the answer"
                ));
            }
            // Any candidate pointing the same way as its beam wins the tie-break
            // outright, so the inflation only bites when every allowed pair is
            // crossed.
            let crossed = beam_leg.iter().all(|&(beam, leg)| {
                fortran_sign(event.incoming[beam][3]) != fortran_sign(out[leg][3])
            });
            if crossed && topology.coloured_beams && !topology.jet_legs {
                return refuse(
                    "the clustering's tie-break inflates every allowed beam-leg candidate, and \
                     with a non-jet leg it reaches the scale through a different power than any \
                     reference run pins"
                        .to_string(),
                );
            }
            let mut q2 = (d[0] * d[1]).sqrt();
            // With colourless beams the colour line never reaches them, so
            // setclscales reads the leftover leg's own measure and the inflated
            // candidate drops out entirely.
            if crossed && topology.coloured_beams {
                q2 *= TIE_BREAK;
            }
            return Ok(Clustered {
                q2,
                beam2_from_beam1: !topology.coloured_beams,
            });
        }

        if topology.coloured_central_line && !topology.coloured_beams {
            return refuse(
                "a colour-charged central propagator between colourless beams".to_string(),
            );
        }
        if out.len() != 2 {
            if topology.coloured_beams {
                return refuse(format!(
                    "an s-channel tree into {} legs with colour reaching the beams: setclscales \
                     then reads the scale off a vertex chosen by isjet, which no reference run \
                     pins",
                    out.len()
                ));
            }
            let total = out.iter().fold([0.0; 4], |acc, p| add(&acc, p));
            return Ok(Clustered {
                q2: self.djb(&total),
                beam2_from_beam1: true,
            });
        }

        let d = [self.djb(&out[0]), self.djb(&out[1])];
        if topology.coloured_central_line {
            let mt2last = (d[0] * d[1]).sqrt();
            if !(mt2last > MT2LAST_FLOOR) {
                return refuse(format!(
                    "the geometric mean of the outgoing transverse masses, {mt2last}, is below \
                     reweight.f's {MT2LAST_FLOOR} floor, so the override that would set the \
                     central scale does not fire"
                ));
            }
            return Ok(Clustered {
                q2: mt2last,
                beam2_from_beam1: false,
            });
        }
        let total = add(&out[0], &out[1]);
        Ok(Clustered {
            q2: self.djb(&total),
            beam2_from_beam1: !topology.coloured_beams,
        })
    }
}

/// `cluster.f`'s inflation of a beam–leg candidate whose legs point in opposite
/// directions, which it uses to prefer clustering an outgoing leg onto the beam
/// it followed.
const TIE_BREAK: f64 = 1.0 + 1e-6;

/// `reweight.f`'s floor on the geometric mean of transverse masses, below which
/// the central vertex keeps the propagator's own scale.
const MT2LAST_FLOOR: f64 = 4.0;

/// How far the two outgoing beam measures of a t-channel 2 → 2 may differ before
/// the topology is treated as misdeclared.
///
/// This is a check on the declaration, not a tolerance on the scale. The two legs
/// of a 2 → 2 carry exactly opposite transverse momenta, so equal masses give
/// exactly equal measures and the clustering's two possible routes — merge a beam
/// with a leg, or merge the legs with each other — return the same number; unequal
/// masses split them, and the answer then turns on `isjet` and on which leg
/// merged. The headroom is there because a *replay* of printed momenta forms
/// `(E − p_z)(E + p_z)` from eleven digits that cancel down to six or seven, which
/// alone reaches `1e-6`. It is orders of magnitude tighter than any real mass
/// splitting, and what survives it moves the scale only as its eighth root.
const BALANCE_HEADROOM: f64 = 1e-4;

/// The clustering's outcome: one squared scale, plus how `reweight.f` fills the
/// second beam from it.
struct Clustered {
    /// `pt2ijcl` at the vertex both scales are read from.
    q2: f64,
    /// With no colour line reaching the beams, `reweight.f:1215` sets
    /// `q2fact(2) = scalefact² · q2fact(1)` from a `q2fact(1)` that already
    /// carries `scalefact²` — so beam 2's scale picks the factor up twice. It is
    /// invisible in every banked run, all of which have `scalefact = 1`, and is
    /// reproduced rather than tidied because a run with `scalefact ≠ 1` would
    /// otherwise disagree with MadGraph on beam 2 alone.
    beam2_from_beam1: bool,
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

/// Fortran's `sign(1d0, x)`, which is `+1` at zero.
fn fortran_sign(x: f64) -> f64 {
    if x >= 0.0 {
        1.0
    } else {
        -1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(extra: &str) -> RunCard {
        RunCard::parse(extra).expect("run card")
    }

    /// MadGraph's default beams are protons; the fixed-beam cases below have to
    /// say so, since the beam measure switches on it.
    fn partonic_card(extra: &str) -> RunCard {
        card(&format!("0 = lpp1\n0 = lpp2\n{extra}"))
    }

    fn event<'a>(outgoing: &'a [[f64; 4]], topology: Option<ClusterTopology>) -> ScaleEvent<'a> {
        ScaleEvent {
            incoming: [[250.0, 0.0, 0.0, 250.0], [250.0, 0.0, 0.0, -250.0]],
            outgoing,
            topology,
        }
    }

    /// A coloured 2 -> 2 whose diagrams let either leg follow either beam, all
    /// legs jets: `g g -> g g`.
    const T_CHANNEL_JETS: ClusterTopology = ClusterTopology {
        beam_connections: BeamConnections::TChannel {
            two_body_pairs: [[true, true], [true, true]],
        },
        coloured_beams: true,
        coloured_central_line: true,
        jet_legs: true,
    };

    /// A colourless s-channel process: `e+ e- -> mu+ mu-`.
    const SINGLET: ClusterTopology = ClusterTopology {
        beam_connections: BeamConnections::SChannelOnly,
        coloured_beams: false,
        coloured_central_line: false,
        jet_legs: false,
    };

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
        assert!(!choice.needs_topology());
        let out = [[125.0, 10.0, 0.0, 124.0], [125.0, -10.0, 0.0, -124.0]];
        let scales = choice.scales(&event(&out, None)).expect("scales");
        assert_eq!(scales.mu_r, 80.0);
        assert_eq!(scales.mu_f, [70.0, 60.0]);
    }

    /// `scalefact` multiplies a dynamic scale exactly once, and the factorisation
    /// scale inherits it through `tempscale**2` rather than picking up a second.
    #[test]
    fn scalefact_multiplies_a_dynamic_scale_once() {
        let choice =
            ScaleChoice::from_run_card(&card("3 = dynamical_scale_choice\n2.0 = scalefact"))
                .expect("compiled");
        let out = [[125.0, 0.0, 0.0, 0.0], [125.0, 0.0, 0.0, 0.0]];
        let scales = choice.scales(&event(&out, None)).expect("scales");
        assert_eq!(scales.mu_r, 2.0 * 125.0);
        assert_eq!(scales.mu_f, [250.0, 250.0]);
    }

    /// A fixed `μR` with a dynamic `μF` is a real configuration: `set_fac_scale`
    /// calls `set_ren_scale` for its own value regardless of `fixed_ren_scale`.
    #[test]
    fn a_fixed_renormalisation_scale_leaves_the_factorisation_scale_dynamic() {
        let choice = ScaleChoice::from_run_card(&card(
            "True = fixed_ren_scale\n91.188 = scale\n4 = dynamical_scale_choice\n",
        ))
        .expect("compiled");
        let out = [[250.0, 0.0, 0.0, 0.0], [250.0, 0.0, 0.0, 0.0]];
        let scales = choice.scales(&event(&out, None)).expect("scales");
        assert_eq!(scales.mu_r, 91.188);
        assert_eq!(scales.mu_f, [500.0, 500.0]);
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

    /// `-1` without a topology declaration is an error, not a guess.
    #[test]
    fn the_clustering_branch_demands_a_topology() {
        let choice = ScaleChoice::from_run_card(&card("")).expect("compiled");
        assert_eq!(choice.choice(), DynamicalChoice::Clustered);
        assert!(choice.needs_topology());
        let out = [[125.0, 10.0, 0.0, 124.0], [125.0, -10.0, 0.0, -124.0]];
        assert_eq!(
            choice.scales(&event(&out, None)),
            Err(ScaleError::MissingTopology)
        );
    }

    /// Legs with unequal beam measures leave the closed form behind: the answer
    /// would depend on which leg merged first and on `isjet`.
    #[test]
    fn an_unbalanced_two_body_clustering_is_refused() {
        let choice = ScaleChoice::from_run_card(&card("1 = lpp1\n1 = lpp2")).expect("compiled");
        let out = [[125.0, 10.0, 0.0, 100.0], [125.0, -10.0, 0.0, -60.0]];
        let err = choice
            .scales(&event(&out, Some(T_CHANNEL_JETS)))
            .expect_err("refused");
        assert!(matches!(err, ScaleError::ClusteringNotDegenerate { .. }));
    }

    /// The tie-break is a real branch, not a rounding artefact: crossing both
    /// allowed beam–leg pairs moves the scale by half of `1e-6`.
    #[test]
    fn the_cluster_tie_break_moves_the_scale() {
        let choice = ScaleChoice::from_run_card(&partonic_card("")).expect("compiled");
        let out = [[250.0, 30.0, 0.0, -100.0], [250.0, -30.0, 0.0, 100.0]];
        let locked = ClusterTopology {
            beam_connections: BeamConnections::TChannel {
                two_body_pairs: [[true, false], [false, true]],
            },
            ..T_CHANNEL_JETS
        };
        let crossed = choice.scales(&event(&out, Some(locked))).expect("scales");
        let free = choice
            .scales(&event(&out, Some(T_CHANNEL_JETS)))
            .expect("scales");
        assert_eq!(free.mu_r, 250.0);
        // The inflation lands on the squared scale, as it does in `cluster.f`.
        assert_eq!(crossed.mu_r, (62500.0 * TIE_BREAK).sqrt());
        assert!(crossed.mu_r > free.mu_r);

        // Colourless beams stop the colour line before it reaches them, and the
        // inflated candidate then drops out of the scale altogether.
        let colourless = ClusterTopology {
            coloured_beams: false,
            coloured_central_line: false,
            ..locked
        };
        assert_eq!(
            choice
                .scales(&event(&out, Some(colourless)))
                .expect("scales")
                .mu_r,
            250.0
        );
    }

    /// With no colour line reaching the beams, beam 2's factorisation scale
    /// carries `scalefact` twice. Unexercised by any reference run — every one has
    /// `scalefact = 1` — and asserted so that reproducing `reweight.f` here stays
    /// a deliberate choice.
    #[test]
    fn a_colourless_beam_line_scales_the_second_beam_twice() {
        let choice =
            ScaleChoice::from_run_card(&partonic_card("3.0 = scalefact")).expect("compiled");
        let out = [[250.0, 10.0, 0.0, 0.0], [250.0, -10.0, 0.0, 0.0]];
        let scales = choice.scales(&event(&out, Some(SINGLET))).expect("scales");
        assert_eq!(scales.mu_r, 3.0 * 500.0);
        assert_eq!(scales.mu_f, [3.0 * 500.0, 9.0 * 500.0]);
    }

    /// The beam measure has no beam to measure against without a PDF, and says so
    /// by switching from transverse mass to energy — the reason a fixed-beam
    /// 2 → 2 sits at a constant scale.
    #[test]
    fn the_beam_measure_follows_the_beam_configuration() {
        let out = [[250.0, 30.0, 0.0, 200.0], [250.0, -30.0, 0.0, -200.0]];
        let partonic = ScaleChoice::from_run_card(&partonic_card("")).expect("compiled");
        assert_eq!(
            partonic
                .scales(&event(&out, Some(T_CHANNEL_JETS)))
                .expect("scales")
                .mu_r,
            250.0
        );
        let hadronic = ScaleChoice::from_run_card(&card("1 = lpp1\n1 = lpp2")).expect("compiled");
        let mt2 = (250.0f64 + 200.0) * (250.0 - 200.0);
        assert_eq!(
            hadronic
                .scales(&event(&out, Some(T_CHANNEL_JETS)))
                .expect("scales")
                .mu_r,
            mt2.sqrt()
        );
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

    /// One beam fixed and the other dynamic reaches a corner of `reweight.f` that
    /// fills the dynamic scale from the fixed one; it is refused rather than
    /// guessed.
    #[test]
    fn a_half_fixed_factorisation_scale_is_refused_under_clustering() {
        let choice = ScaleChoice::from_run_card(&card(
            "0 = lpp1\n0 = lpp2\nTrue = fixed_fac_scale1\n30.0 = dsqrt_q2fact1\n-1 = dynamical_scale_choice",
        ))
        .expect("compiled");
        let out = [[250.0, 10.0, 0.0, 0.0], [250.0, -10.0, 0.0, 0.0]];
        assert_eq!(
            choice.scales(&event(&out, Some(SINGLET))),
            Err(ScaleError::MixedFixedFactorisationScales)
        );
    }
}
