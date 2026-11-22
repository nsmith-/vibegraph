//! Tree-level two-body decay widths, as a parameter card's `DECAY <pdg> Auto`
//! asks for them.
//!
//! MadGraph turns an `Auto` width into a computation when a run starts
//! (`common_run_interface.py`, `static_check_param_card`, which calls
//! `compute_widths <pdg>...` with MadWidth's defaults: `--body_decay=4.0025`,
//! `--precision_channel=0.01`, `--min_br` the decimal part of `body_decay` over
//! five). Its two-body widths are tree level: from the model's `decays.py`
//! formulas where the model ships them (`madgraph_interface.py`,
//! `do_compute_widths`), and otherwise from MadWidth's tree-level matrix elements.
//! This module computes the same widths from the model's own vertices and the
//! amplitude evaluation every process uses:
//!
//! * The channels of a particle are every pair of daughters a three-point
//!   vertex couples it to, the daughters being the antiparticles of the
//!   vertex's other legs (UFO vertices list every leg incoming). A channel is
//!   open when the parent's mass exceeds its daughters' strictly, as MadGraph
//!   skips one whose mass difference is not positive.
//! * An open channel's width is `Σ|M|² · p* / (8π M²)` with `Σ|M|²` summed over
//!   every helicity and colour at one rest-frame point, averaged over the
//!   parent's spin and colour states ([`initial_spin_color_average`]) and
//!   divided by the daughters' identical-particle factor
//!   ([`outgoing_symmetry_factor`]). An unpolarised two-body decay's summed
//!   `|M|²` is isotropic, so one point is the whole integral.
//! * A coloured parent's partial width below [`QCD_SCALE_FLOOR`] is set to zero,
//!   as MadGraph sets it ("lower than QCD scale"), and listed as such.
//! * The total is the sum; the width a run binds is that total written as
//!   MadGraph writes it into the card it runs from (`%e`, seven significant
//!   digits, `update_width_in_param_card`).
//!
//! What is not computed, and is reported or refused rather than left out
//! silently:
//!
//! * **Beyond two bodies.** MadWidth's default (`body_decay=4.0025`) goes on to
//!   three- and four-body modes through an off-shell daughter of a closed
//!   two-body channel when its estimate puts them above 0.25% of the width. This
//!   does not reproduce that estimate: a particle with any three-body mode
//!   through a closed channel that its vertices and masses allow is refused,
//!   naming the modes ([`AutoWidthError::BeyondTwoBody`]). The rule is
//!   conservative: it refuses where MadGraph's estimate adds nothing (the SM Z,
//!   whose `Z > W+ e- ve~` through `W-` is open but estimated negligible), and
//!   it cannot tell which of the listed modes MadGraph would add.
//! * **The running of `α_s`.** MadGraph evaluates the couplings at `α_s` run to
//!   the parent's mass; a channel whose couplings depend on `α_s` is refused
//!   ([`AutoWidthError::RunsWithAlphaS`]).
//! * **Widths in couplings.** MadGraph sets every `Auto` width to one while it
//!   computes them, so a width cannot feed another's computation; a channel
//!   whose couplings or masses depend on an `Auto` width is refused
//!   ([`AutoWidthError::DependsOnAutoWidth`]).
//! * **Loop-induced modes** (`h > a a`, `h > g g` in a model without the
//!   effective vertex) have no tree-level vertex and are no channel. A decay
//!   table the card gives below the `Auto` line that names one, or any mode with
//!   more than two daughters, is refused ([`AutoWidthError::ModeNotComputed`]):
//!   MadGraph replaces that table with the one it computes.

use std::collections::{BTreeSet, HashSet};
use std::f64::consts::PI;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info_span;

use crate::artifact::{DecayChannel, DecayTable};
use crate::diagrams::{enumerate_lines_in, parse_proc_card, EnumerationPool, ParsingOptions};
use crate::hadronic::{compile_class, initial_spin_color_average, outgoing_symmetry_factor};
use crate::helas::eval::BoundAmplitude;
use crate::helas::repr::lorentz::LorentzVector;
use crate::lhef::write::c_exponential;
use crate::ufo::particles::ParticleId;
use crate::ufo::{EvaluatedModel, UFOModel};

/// The method [`ComputedWidth::method`] names.
pub const METHOD: &str = "tree_level_two_body";

/// The partial width (GeV) below which MadGraph sets a coloured particle's
/// channel to zero (`do_compute_widths`, `madgraph_interface.py`).
pub const QCD_SCALE_FLOOR: f64 = 0.1;

/// One daughter of a channel: its PDG code and the name a process card writes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Daughter {
    pub pdg: i32,
    pub name: String,
}

/// One two-body channel and its partial width in GeV.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PartialWidth {
    /// The two daughters, by descending PDG code.
    pub daughters: Vec<Daughter>,
    pub width: f64,
}

/// A two-body channel the parent is too light for.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClosedChannel {
    pub daughters: Vec<Daughter>,
}

/// A three-body mode reached through an off-shell daughter of a closed
/// two-body channel: MadWidth's default may add it, and this width does not.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OmittedMode {
    /// The three daughters, by descending PDG code.
    pub daughters: Vec<Daughter>,
    /// The off-shell particle the mode goes through.
    pub through: Daughter,
}

/// A width computed for `DECAY <pdg> Auto`: the channels, the total and what
/// it leaves out.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComputedWidth {
    pub pdg: i32,
    pub particle: String,
    /// The width parameter the run binds the result to.
    pub param: String,
    /// [`METHOD`].
    pub method: String,
    /// The sum of [`channels`](Self::channels), GeV.
    pub total: f64,
    /// The value the run binds: [`total`](Self::total) to seven significant
    /// digits, as MadGraph writes a computed width into the card it runs from.
    pub bound: f64,
    /// The open channels, by descending width.
    pub channels: Vec<PartialWidth>,
    /// Open channels of a coloured parent whose width is below
    /// [`QCD_SCALE_FLOOR`], set to zero and left out of the total as MadGraph
    /// leaves them out.
    pub below_qcd_scale: Vec<PartialWidth>,
    pub closed: Vec<ClosedChannel>,
    /// Three-body modes through an off-shell daughter of a closed channel.
    /// Empty on every width a card binds: [`compute_width`] refuses a particle
    /// that has one; [`two_body_width`] lists them.
    pub not_included: Vec<OmittedMode>,
}

impl ComputedWidth {
    /// The decay table MadGraph writes below the computed `DECAY` line: each
    /// channel with a non-zero width, by descending branching ratio, the ratio
    /// to the seven digits MadGraph writes. Empty when the width is zero.
    pub fn decay_table(&self) -> DecayTable {
        let channels = if self.total > 0.0 {
            self.channels
                .iter()
                .filter(|c| c.width > 0.0)
                .map(|c| DecayChannel {
                    branching_ratio: madgraph_rounded(c.width / self.total),
                    daughters: c.daughters.iter().map(|d| d.pdg).collect(),
                })
                .collect()
        } else {
            Vec::new()
        };
        DecayTable {
            pdg: self.pdg,
            channels,
        }
    }
}

/// Why an `Auto` width is not computed.
#[derive(Clone, Debug, PartialEq, Error)]
pub enum AutoWidthError {
    #[error("{pdg} is not the code of a physical particle of the model")]
    UnknownParticle { pdg: i32 },
    #[error(
        "the width of {particle} is {param}, which is not a free parameter of the model ({why}), \
         so there is no width to set"
    )]
    NotFree {
        particle: String,
        param: String,
        why: &'static str,
    },
    #[error(
        "the coupling of {channel} depends on alpha_s, which MadGraph runs to the parent's mass \
         before it computes the width, and this does not; give the width as a number"
    )]
    RunsWithAlphaS { channel: String },
    #[error(
        "{channel} depends on the width {width}, which the card also sets to Auto; MadGraph \
         computes every Auto width with the others set to one, so give one of them as a number"
    )]
    DependsOnAutoWidth { channel: String, width: String },
    #[error(
        "{particle} has three-body modes through closed two-body channels ({modes}); vibegraph \
         computes two-body widths only, and MadGraph's default may include such modes, so give \
         the width as a number"
    )]
    BeyondTwoBody { particle: String, modes: String },
    #[error(
        "the card's decay table below the Auto line lists {mode}, which {why}; MadGraph replaces \
         the table with the channels it computes, so drop the row or give the width as a number"
    )]
    ModeNotComputed { mode: String, why: &'static str },
    #[error("{channel} cannot be evaluated: {message}")]
    Channel { channel: String, message: String },
}

/// The name a process card writes for particle `id`.
fn card_name(model: &UFOModel, id: ParticleId) -> String {
    let ufo = &model.particle(id).name;
    model
        .particle_names()
        .renamed_to(ufo)
        .unwrap_or(ufo)
        .to_owned()
}

fn daughter(model: &UFOModel, id: ParticleId) -> Daughter {
    Daughter {
        pdg: model.particle(id).pdg_code as i32,
        name: card_name(model, id),
    }
}

/// The antiparticle of `id`: itself for a self-conjugate particle.
fn anti(model: &UFOModel, id: ParticleId) -> ParticleId {
    let p = model.particle(id);
    if p.is_self_conjugate {
        return id;
    }
    model
        .particles
        .values()
        .position(|q| q.pdg_code == -p.pdg_code)
        .map_or(id, ParticleId::from)
}

/// The physical particle with PDG code `pdg`.
fn particle_with_code(model: &UFOModel, pdg: i32) -> Option<ParticleId> {
    model
        .particles
        .values()
        .position(|p| p.pdg_code == i64::from(pdg) && !p.is_unphysical())
        .map(ParticleId::from)
}

/// `daughters` by descending PDG code.
fn ordered(model: &UFOModel, mut daughters: Vec<ParticleId>) -> Vec<ParticleId> {
    daughters.sort_by_key(|&d| std::cmp::Reverse(model.particle(d).pdg_code));
    daughters
}

/// One two-body channel of a parent: its daughters by descending PDG code and
/// every parameter its vertices' couplings read directly.
struct Channel {
    daughters: [ParticleId; 2],
    coupling_deps: BTreeSet<String>,
}

/// The two-body channels of `parent` in vertex order: each unordered pair of
/// daughters some physical three-point vertex couples it to, once.
fn two_body_channels(model: &UFOModel, parent: ParticleId) -> Vec<Channel> {
    let mut channels: Vec<Channel> = Vec::new();
    for vertex in model.vertices.values() {
        if vertex.particles.len() != 3 || !vertex.is_physical(&model.particles) {
            continue;
        }
        let Some(at) = vertex.particles.iter().position(|&p| p == parent) else {
            continue;
        };
        let others: Vec<ParticleId> = vertex
            .particles
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != at)
            .map(|(_, &p)| anti(model, p))
            .collect();
        let pair = ordered(model, others);
        let daughters = [pair[0], pair[1]];
        let deps = vertex
            .couplings
            .values()
            .flat_map(|&c| model.coupling_def(c).deps.iter().cloned());
        match channels.iter_mut().find(|c| c.daughters == daughters) {
            Some(c) => c.coupling_deps.extend(deps),
            None => channels.push(Channel {
                daughters,
                coupling_deps: deps.collect(),
            }),
        }
    }
    channels
}

fn mode_text(model: &UFOModel, parent: ParticleId, daughters: &[ParticleId]) -> String {
    let names: Vec<String> = daughters.iter().map(|&d| card_name(model, d)).collect();
    format!("{} > {}", card_name(model, parent), names.join(" "))
}

/// The three-body modes reached through an off-shell daughter of one of
/// `closed`: the off-shell daughter `X` splits through a two-body channel of
/// its own, and the parent is heavier than the three. A massless emission off
/// the particle's own line (`t > t g`, `X > X a`) is a radiative correction to
/// a two-body width, not a mode, and is left out, as MadWidth leaves it out.
/// Once per final state, in channel order.
fn three_body_modes(
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    parent: ParticleId,
    closed: &[[ParticleId; 2]],
) -> Vec<(Vec<ParticleId>, ParticleId)> {
    let mass = |id: ParticleId| evaluated.mass(id).abs();
    let m_parent = mass(parent);
    let parent_anti = anti(model, parent);
    let mut seen: HashSet<Vec<i64>> = HashSet::new();
    let mut modes = Vec::new();
    for pair in closed {
        for (off, other) in [(pair[0], pair[1]), (pair[1], pair[0])] {
            if (off == parent || off == parent_anti) && mass(other) == 0.0 {
                continue;
            }
            for split in two_body_channels(model, off) {
                let [a, b] = split.daughters;
                if (a == off && mass(b) == 0.0) || (b == off && mass(a) == 0.0) {
                    continue;
                }
                if !(m_parent - mass(a) - mass(b) - mass(other) > 0.0) {
                    continue;
                }
                let three = ordered(model, vec![a, b, other]);
                let key: Vec<i64> = three.iter().map(|&d| model.particle(d).pdg_code).collect();
                if seen.insert(key) {
                    modes.push((three, off));
                }
            }
        }
    }
    modes
}

/// The partial width (GeV) of the one subprocess `process` generates:
/// `Σ|M|²` over every helicity and colour at the parent at rest and the
/// daughters back to back, times the parent's spin-colour average and the
/// daughters' identical-particle factor, times `p* / (8π M²)`.
fn channel_width(
    model: &Arc<UFOModel>,
    evaluated: &EvaluatedModel,
    process: &str,
) -> Result<f64, String> {
    let card = parse_proc_card(&format!("generate {process}\n"), &ParsingOptions::default())
        .map_err(|e| e.to_string())?;
    // Enumeration and compilation report each subprocess at `info`; a width's
    // channels are reported once, by the caller, so theirs are not shown.
    let quiet = tracing::subscriber::NoSubscriber::default();
    let eval = tracing::subscriber::with_default(quiet, || {
        let lines =
            enumerate_lines_in(&card, model, EnumerationPool::Serial).map_err(|e| e.to_string())?;
        let [set] = lines.sets.as_slice() else {
            return Err(format!(
                "its generation gives {} subprocesses where one was expected",
                lines.sets.len()
            ));
        };
        let eval = compile_class(set, model, evaluated).map_err(|e| e.to_string())?;
        Ok(eval)
    })?;
    let masses: Vec<f64> = eval
        .external_particles()
        .iter()
        .map(|&id| evaluated.mass(id).abs())
        .collect();
    let [m, m1, m2] = masses[..] else {
        return Err(format!("it has {} external legs", masses.len()));
    };
    let lambda = (m * m - (m1 + m2).powi(2)) * (m * m - (m1 - m2).powi(2));
    let p_star = lambda.max(0.0).sqrt() / (2.0 * m);
    // Any direction off the axes: the helicity-summed |M|² does not depend on it.
    let (cos_theta, phi) = (0.3f64, 0.7f64);
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    let dir = [sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta];
    let e1 = (m * m + m1 * m1 - m2 * m2) / (2.0 * m);
    let momenta = [
        LorentzVector::new(m, 0.0, 0.0, 0.0),
        LorentzVector::new(e1, p_star * dir[0], p_star * dir[1], p_star * dir[2]),
        LorentzVector::new(m - e1, -p_star * dir[0], -p_star * dir[1], -p_star * dir[2]),
    ];
    let bound = BoundAmplitude::<f64>::bind(&eval, evaluated);
    let mut scratch = bound.scratch_space();
    let m2_sum = bound.eval_m2(&momenta, &mut scratch);
    let weight =
        initial_spin_color_average(&eval, model, evaluated) * outgoing_symmetry_factor(&eval);
    let width = m2_sum * weight * p_star / (8.0 * PI * m * m);
    if !width.is_finite() || width < 0.0 {
        return Err(format!("its partial width evaluates to {width}"));
    }
    Ok(width)
}

/// `value` as MadGraph's `%e` writes it: seven significant digits.
pub fn madgraph_rounded(value: f64) -> f64 {
    format!("{value:.6e}")
        .parse()
        .expect("a formatted finite number parses")
}

/// The width `DECAY <pdg> Auto` binds: [`two_body_width`], refused when the
/// particle has a mode beyond two bodies ([`AutoWidthError::BeyondTwoBody`]),
/// so the result's [`not_included`](ComputedWidth::not_included) is empty.
///
/// # Errors
///
/// [`AutoWidthError`] for a width this cannot compute as MadGraph would.
pub fn compute_width(
    model: &Arc<UFOModel>,
    evaluated: &EvaluatedModel,
    pdg: i32,
    auto_params: &BTreeSet<String>,
) -> Result<ComputedWidth, AutoWidthError> {
    let width = two_body_width(model, evaluated, pdg, auto_params)?;
    if width.not_included.is_empty() {
        return Ok(width);
    }
    let modes: Vec<String> = width
        .not_included
        .iter()
        .map(|m| {
            let names: Vec<&str> = m.daughters.iter().map(|d| d.name.as_str()).collect();
            format!(
                "{} > {} (through {})",
                width.particle,
                names.join(" "),
                m.through.name
            )
        })
        .collect();
    Err(AutoWidthError::BeyondTwoBody {
        particle: width.particle,
        modes: modes.join(", "),
    })
}

/// The tree-level two-body width of the particle with PDG code `pdg` under
/// `evaluated`, whose `Auto` widths `auto_params` are not yet known, with the
/// three-body modes through its closed channels listed in
/// [`not_included`](ComputedWidth::not_included) rather than refused.
///
/// # Errors
///
/// [`AutoWidthError`] for a two-body width this cannot compute as MadGraph
/// would.
pub fn two_body_width(
    model: &Arc<UFOModel>,
    evaluated: &EvaluatedModel,
    pdg: i32,
    auto_params: &BTreeSet<String>,
) -> Result<ComputedWidth, AutoWidthError> {
    let parent = particle_with_code(model, pdg)
        .filter(|_| pdg > 0)
        .ok_or(AutoWidthError::UnknownParticle { pdg })?;
    let particle = model.particle(parent);
    let _span = info_span!("auto_width", particle = %particle.name).entered();
    let m_parent = evaluated.mass(parent).abs();
    let mass = |id: ParticleId| evaluated.mass(id).abs();

    let strong: HashSet<String> = {
        let mut s = model.params.dependents("aS");
        s.insert("aS".to_owned());
        s
    };
    let width_driven: Vec<(String, HashSet<String>)> = auto_params
        .iter()
        .map(|w| {
            let mut s = model.params.dependents(w);
            s.insert(w.clone());
            (w.clone(), s)
        })
        .collect();

    let mut channels = Vec::new();
    let mut below = Vec::new();
    let mut closed = Vec::new();
    for channel in two_body_channels(model, parent) {
        let [a, b] = channel.daughters;
        let daughters = vec![daughter(model, a), daughter(model, b)];
        if !(m_parent - mass(a) - mass(b) > 0.0) {
            closed.push(channel.daughters);
            continue;
        }
        let process = mode_text(model, parent, &channel.daughters);
        if channel.coupling_deps.iter().any(|d| strong.contains(d)) {
            return Err(AutoWidthError::RunsWithAlphaS { channel: process });
        }
        let masses = [parent, a, b].map(|id| model.particle(id).mass_param.clone());
        for (width, driven) in &width_driven {
            if channel.coupling_deps.iter().any(|d| driven.contains(d))
                || masses.iter().any(|m| driven.contains(m))
            {
                return Err(AutoWidthError::DependsOnAutoWidth {
                    channel: process,
                    width: width.clone(),
                });
            }
        }
        let width = channel_width(model, evaluated, &process).map_err(|message| {
            AutoWidthError::Channel {
                channel: process.clone(),
                message,
            }
        })?;
        let partial = PartialWidth { daughters, width };
        if particle.color.abs() != 1 && width > 0.0 && width < QCD_SCALE_FLOOR {
            below.push(partial);
        } else {
            channels.push(partial);
        }
    }

    let modes = three_body_modes(model, evaluated, parent, &closed);

    channels.sort_by(|x, y| y.width.total_cmp(&x.width));
    let total: f64 = channels.iter().map(|c| c.width).sum();
    Ok(ComputedWidth {
        pdg,
        particle: card_name(model, parent),
        param: particle.width_param.clone(),
        method: METHOD.to_owned(),
        total,
        bound: madgraph_rounded(total),
        channels,
        below_qcd_scale: below,
        closed: closed
            .iter()
            .map(|&[a, b]| ClosedChannel {
                daughters: vec![daughter(model, a), daughter(model, b)],
            })
            .collect(),
        not_included: modes
            .into_iter()
            .map(|(d, through)| OmittedMode {
                daughters: d.into_iter().map(|id| daughter(model, id)).collect(),
                through: daughter(model, through),
            })
            .collect(),
    })
}

/// Refuse a row of `table`, given below `computed`'s `Auto` line, that names a
/// mode the computation has no channel for: one beyond two bodies, or a
/// two-body one no tree-level vertex gives.
pub(crate) fn check_given_table(
    model: &UFOModel,
    computed: &ComputedWidth,
    table: &DecayTable,
) -> Result<(), AutoWidthError> {
    let known: Vec<Vec<i32>> = computed
        .channels
        .iter()
        .chain(&computed.below_qcd_scale)
        .map(|c| c.daughters.iter().map(|d| d.pdg).collect::<Vec<_>>())
        .chain(
            computed
                .closed
                .iter()
                .map(|c| c.daughters.iter().map(|d| d.pdg).collect()),
        )
        .map(|mut d: Vec<i32>| {
            d.sort_unstable();
            d
        })
        .collect();
    for row in &table.channels {
        let mut key = row.daughters.clone();
        key.sort_unstable();
        if row.daughters.len() == 2 && known.contains(&key) {
            continue;
        }
        let codes: Vec<String> = row.daughters.iter().map(i32::to_string).collect();
        let name = |pdg: i32| {
            particle_with_code(model, pdg)
                .map(|id| card_name(model, id))
                .unwrap_or_else(|| pdg.to_string())
        };
        let names: Vec<String> = row.daughters.iter().map(|&d| name(d)).collect();
        let mode = format!(
            "{} > {} ({})",
            computed.particle,
            names.join(" "),
            codes.join(" ")
        );
        let why = if row.daughters.len() == 2 {
            "no tree-level vertex of the model gives (a loop-induced mode needs a vertex the model lacks)"
        } else {
            "has more than two daughters, and only two-body channels are computed"
        };
        return Err(AutoWidthError::ModeNotComputed { mode, why });
    }
    Ok(())
}

/// Whether `line` is a row of a decay table: its first field is a number.
fn is_table_row(line: &str) -> bool {
    line.split_whitespace()
        .next()
        .is_some_and(|f| !f.starts_with('#') && f.parse::<f64>().is_ok())
}

/// `text`, a parameter card, with each `DECAY <pdg> Auto` line of `computed`
/// written as MadGraph writes the card it runs from: the computed width on the
/// `DECAY` line, with a comment saying it was computed and how, the table the
/// card gave below it replaced by the computed one (each row's partial width as
/// a comment). Binding the result
/// gives the same parameters and tables as binding `text`.
pub fn resolved_card_text(text: &str, computed: &[ComputedWidth]) -> String {
    let mut out = String::new();
    let mut dropping = false;
    for line in text.lines() {
        let fields: Vec<&str> = line
            .split('#')
            .next()
            .unwrap_or("")
            .split_whitespace()
            .collect();
        let width = match fields.as_slice() {
            [decay, code, value]
                if decay.eq_ignore_ascii_case("decay") && value.eq_ignore_ascii_case("auto") =>
            {
                code.parse::<i32>()
                    .ok()
                    .and_then(|pdg| computed.iter().find(|w| w.pdg == pdg))
            }
            _ => None,
        };
        if let Some(width) = width {
            out.push_str(&width_lines(width));
            dropping = true;
            continue;
        }
        if dropping && is_table_row(line) {
            continue;
        }
        if !line.trim().is_empty() && !line.trim_start().starts_with('#') {
            dropping = false;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// The `DECAY` line and table of one computed width, as
/// [`resolved_card_text`] writes them.
fn width_lines(width: &ComputedWidth) -> String {
    let mut text = format!(
        "DECAY  {}   {} # {} Auto: {} width computed by vibegraph {}, total {:e} GeV\n",
        width.pdg,
        c_exponential(width.bound, 6, false, 0),
        width.param,
        width.method,
        env!("CARGO_PKG_VERSION"),
        width.total,
    );
    let table = width.decay_table();
    if !table.channels.is_empty() {
        text.push_str("#  BR             NDA  ID1    ID2   ...\n");
        for channel in &table.channels {
            let daughters: Vec<String> = channel.daughters.iter().map(i32::to_string).collect();
            text.push_str(&format!(
                "   {}   {}    {} # {:e}\n",
                c_exponential(channel.branching_ratio, 6, false, 0),
                channel.daughters.len(),
                daughters.join("  "),
                channel.branching_ratio * width.total,
            ));
        }
    }
    for c in &width.below_qcd_scale {
        let names: Vec<&str> = c.daughters.iter().map(|d| d.name.as_str()).collect();
        text.push_str(&format!(
            "#  set to zero, below {QCD_SCALE_FLOOR} GeV for a coloured particle: {} > {} ({:e} GeV)\n",
            width.particle,
            names.join(" "),
            c.width
        ));
    }
    text
}
