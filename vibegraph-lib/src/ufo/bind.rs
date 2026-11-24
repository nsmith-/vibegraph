//! Binding a restricted model to a user's SLHA parameter card.
//!
//! [`EvaluatedModel::from_model_card`] takes each external parameter's value
//! from the card where it has one and the restricted default otherwise, and
//! ignores every other entry. That is the evaluation; [`bind_card`] is the
//! check around it that a run at a card must pass first, and the record of
//! what the card moved:
//!
//! * `DECAY <pdg> Auto` binds the width to its tree-level two-body value,
//!   computed from the model at the card's other parameters
//!   ([`crate::widths`]), as MadGraph computes it at launch (MadWidth) and
//!   writes the number into the card it runs from: the bound value is that
//!   number, to the seven digits MadGraph writes, and the table below the
//!   `DECAY` line is the computed channels' ([`BoundCard::auto_widths`]). A width
//!   that cannot be computed as MadGraph would is refused naming why
//!   ([`CardError::AutoWidth`]).
//! * A non-zero value for a parameter the restriction fixed to zero is refused:
//!   the restriction also pruned the vertices that parameter feeds, so the
//!   model could not honour it. A restriction zeroes parameters and keeps them
//!   in the model's parameter list, so a block or entry that only the full
//!   model defines is told from one no version of the model defines, and the
//!   refusal names the restriction ([`bind_card_under`]).
//! * A value other than one for a parameter the restriction leaves at exactly
//!   one is refused: MadGraph fixes such a parameter at import and its
//!   `param_card_rule.dat` holds every card to it.
//! * Moving one parameter of a group the restriction merged
//!   ([`ParameterSet::merged`](super::parameters::ParameterSet)) without the
//!   others is refused: MadGraph evaluates the others as the group's first.
//! * A `MASS` or `DECAY` entry for a particle whose mass or width is not an
//!   external parameter (MW in the SM, or a massless particle's `ZERO`) is
//!   ignored, as MadGraph's Fortran ignores it; one that disagrees with the
//!   computed value is reported in [`BoundCard::ignored`].
//! * An entry in a block no parameter of the model sits in, and a `MASS` or
//!   `DECAY` entry at a code no particle of the model has, is ignored and
//!   listed in [`BoundCard::ignored`]: MadGraph reads only the entries the
//!   model's `ident_card` names (`write_inc_file`, `check_param_card.py`), so
//!   a card written for a larger model binds; a decay table below such a
//!   `DECAY` line goes with it. The `QNUMBERS` blocks that
//!   declare a new state's quantum numbers are not parameters either.
//! * A code the model has no parameter for inside a block it does define is
//!   refused: a mistyped code would leave the parameter the card meant at its
//!   default.
//! * A decay table is carried, as written, to the event file's `<slha>`
//!   header ([`EvaluatedModel::decay_tables`]), where MadGraph puts the card
//!   for a shower to decay from. MadGraph checks neither a table's sum nor its
//!   daughters, and neither does this. A table below a `DECAY` line the header
//!   does not write (an antiparticle's code) is refused.
//! * A negative mass is refused on any particle but a Majorana fermion,
//!   whichever of the model's default, its restriction or the card gives it
//!   ([`CardError::NegativeMass`]): the phase-space, on-shell, cut and
//!   event-record code downstream of the amplitudes is not validated for one.
//!   A Majorana fermion's negative mass binds, and the evaluation negates the
//!   width at its code as MadGraph's Fortran does ([`EvaluatedModel`]). Its
//!   spinors and propagator read the signed mass; the MSSM neutralino `n3`
//!   (`Mneu3 < 0` in the default card) agrees with MadGraph's `AMP()` per diagram
//!   as an external leg and as a propagator (`standalone_jamps`'s
//!   `ee_to_n1n3_mssm` and `x1px1m_to_wpwm_mssm`, at zero widths).
//!
//! Every entry refused is reported at once ([`CardError::Several`]).

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::sync::Arc;

use thiserror::Error;

use super::parameters::{ParamNature, ParameterSet};
use super::slha::{code_text, ParamCard};
use super::{EvaluatedModel, UFOModel};
use crate::artifact::{DecayTable, MovedParam};
use crate::widths::{
    check_given_table, compute_width, decaying_particle, AutoWidthError, ComputedWidth,
};

/// How far a card's entry for a computed mass or width may sit from the value
/// the model computes, relative, before it is reported. MadGraph writes those
/// entries with `%e`, seven significant digits, so its own card sits within
/// `5e-7` of the computed value.
const DEPENDENT_TOLERANCE: f64 = 1e-6;

/// Why a parameter card cannot be bound to a model.
#[derive(Clone, Debug, PartialEq, Error)]
pub enum CardError {
    #[error(
        "the card sets the width of particle {pdg} to Auto, which cannot be computed: {reason}"
    )]
    AutoWidth { pdg: i32, reason: AutoWidthError },
    #[error(
        "the card sets the width of particle {pdg} to Auto, which only binding the card \
         (bind_card) computes; this evaluation reads numbers only"
    )]
    AutoWidthUnbound { pdg: i32 },
    #[error(
        "the card sets {name} (block {block}, code {code}) to {value:e}, but it was removed by \
         {restriction}, which fixes it to zero and pruned the vertices it feeds; use a \
         restriction that keeps {name} free, or leave it at zero"
    )]
    RevivesZeroed {
        name: String,
        block: String,
        code: String,
        value: f64,
        /// What removed it: `restrict_<x>.dat`, or the model's restriction
        /// when the caller does not name one.
        restriction: String,
    },
    #[error(
        "the card sets {name} (block {block}, code {code}) to {value:e}, but the model's \
         restriction fixes it to one, as MadGraph's restriction does, and MadGraph refuses a \
         card that moves it; use a restriction that keeps {name} free, or leave it at one"
    )]
    MovesFixedOne {
        name: String,
        block: String,
        code: String,
        value: f64,
    },
    #[error(
        "the card sets {member} = {member_value:e} and {first} = {first_value:e}, but the \
         model's restriction merged them in block {block} ({member} = {factor} × {first}, as \
         MadGraph evaluates it); move them together, or use a restriction that keeps them apart"
    )]
    SplitsMerged {
        block: String,
        first: String,
        first_value: f64,
        member: String,
        member_value: f64,
        factor: f64,
    },
    #[error(
        "the card sets block {block} code {code} to {value:e}, and the model has no parameter \
         there (block {block} is one the model defines, so a mistyped code is refused rather \
         than left at its default)"
    )]
    Unknown {
        block: String,
        code: String,
        value: f64,
    },
    #[error("the card sets block {block} code {code} to {value}, which is not a finite number")]
    NonFinite {
        block: String,
        code: String,
        value: f64,
    },
    #[error(
        "the recorded parameter {name} (block {block}, code {code}) is not a free external \
         parameter of this model"
    )]
    NotRecordable {
        name: String,
        block: String,
        code: String,
    },
    #[error(
        "the card gives a decay table below DECAY {pdg}, which is not the code of a physical \
         particle of the model; the event file's <slha> header lists each particle's decays \
         once, under its positive PDG code, so give the table there"
    )]
    DecayTableParticle { pdg: i32 },
    #[error(
        "the mass of particle {particle} (PDG {pdg}) is {param} = {value:e}, negative (from \
         the model's default, its restriction or the param card); a negative mass is accepted \
         only on a Majorana fermion, because the Breit-Wigner phase-space map, the on-shell \
         checks, the cuts and the LHEF resonance record are not validated for a negative mass, \
         so give the mass as positive"
    )]
    NegativeMass {
        particle: String,
        pdg: i64,
        param: String,
        value: f64,
    },
    /// More than one entry of the card is refused; each is listed.
    #[error("{}", several(.0))]
    Several(Vec<CardError>),
}

fn several(errors: &[CardError]) -> String {
    errors
        .iter()
        .map(CardError::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

/// `Ok` when nothing was refused, the one refusal, or [`CardError::Several`].
fn refuse_all(mut errors: Vec<CardError>) -> Result<(), CardError> {
    match errors.len() {
        0 => Ok(()),
        1 => Err(errors.remove(0)),
        _ => Err(CardError::Several(errors)),
    }
}

/// Why a card entry was ignored.
#[derive(Clone, Debug, PartialEq)]
pub enum IgnoredBecause {
    /// The model computes the value from `param` (`MW`, `ZERO`) and the card's
    /// disagrees with `computed`.
    Computed { param: String, computed: f64 },
    /// No parameter of the model sits at the entry: its block is none the model
    /// defines, or it is a `MASS` or `DECAY` entry at a code no particle has.
    NoParameter,
    /// The decay table below a `DECAY` line at a code no particle of the model
    /// has; [`IgnoredEntry::card`] is the sum of its branching ratios.
    NoParticleTable { channels: usize },
    /// A `DECAY` line at a code no particle of the model has, with its width
    /// given as `Auto`; [`IgnoredEntry::card`] is zero.
    NoParticleAuto,
}

/// A card entry the binding ignored.
#[derive(Clone, Debug, PartialEq)]
pub struct IgnoredEntry {
    /// The block as the card spells it, lower case (`MASS` and `DECAY` upper
    /// case for an entry at a particle).
    pub block: String,
    /// The entry's code, as [`code_text`] writes it.
    pub code: String,
    /// The card's value.
    pub card: f64,
    pub why: IgnoredBecause,
}

/// A model bound to a card: the evaluation, the parameters the card moved off
/// the restricted defaults, and the card entries it ignored.
#[derive(Clone)]
pub struct BoundCard {
    pub evaluated: EvaluatedModel,
    /// Every external parameter whose bound value differs from the restricted
    /// default, in the model's external order.
    pub moved: Vec<MovedParam>,
    pub ignored: Vec<IgnoredEntry>,
    /// The widths the card set to `Auto`, as computed, in card order. Each
    /// bound value is also in [`moved`](Self::moved) where it differs from the
    /// default, and its decay table in the evaluation's
    /// [`decay_tables`](EvaluatedModel::decay_tables).
    pub auto_widths: Vec<ComputedWidth>,
}

/// The external parameter at `block` (case-insensitive) and `code`, with its
/// restricted default.
fn external_at<'m>(model: &'m UFOModel, block: &str, code: &[i32]) -> Option<(&'m str, f64)> {
    model.params.externals.iter().find_map(|p| match &p.nature {
        ParamNature::External {
            default_value,
            lha_block,
            lha_code,
        } if lha_block.eq_ignore_ascii_case(block) && lha_code == code => {
            Some((p.name.as_str(), *default_value))
        }
        _ => None,
    })
}

/// The decay tables of `tables` whose particle the `<slha>` header writes no
/// `DECAY` line for: a code that is not positive, or not a physical particle of
/// `model`.
fn unwritable_tables(model: &UFOModel, tables: &[DecayTable]) -> Vec<CardError> {
    tables
        .iter()
        .filter(|t| {
            t.pdg <= 0
                || !model
                    .particles
                    .values()
                    .any(|p| p.pdg_code == i64::from(t.pdg) && !p.is_unphysical())
        })
        .map(|t| CardError::DecayTableParticle { pdg: t.pdg })
        .collect()
}

/// A [`CardError::NegativeMass`] for each physical particle of `evaluated`'s
/// model, other than a Majorana fermion, whose mass evaluates negative; one per
/// mass parameter, by PDG code.
fn negative_masses(evaluated: &EvaluatedModel) -> Vec<CardError> {
    let model = evaluated.model();
    let mut particles: Vec<_> = model
        .particles
        .values()
        .filter(|p| p.pdg_code > 0 && !p.is_unphysical())
        .filter(|p| !(p.spin.abs() == 2 && p.is_self_conjugate))
        .collect();
    particles.sort_by_key(|p| p.pdg_code);
    let mut seen: Vec<&str> = Vec::new();
    let mut errors = Vec::new();
    for p in particles {
        let value = evaluated
            .param_values
            .get(&p.mass_param)
            .map_or(0.0, |v| v.re);
        if value < 0.0 && !seen.contains(&p.mass_param.as_str()) {
            seen.push(&p.mass_param);
            errors.push(CardError::NegativeMass {
                particle: p.name.clone(),
                pdg: p.pdg_code,
                param: p.mass_param.clone(),
                value,
            });
        }
    }
    errors
}

/// The parameter that sets `pdg`'s mass (`block = "mass"`) or width
/// (`"decay"`), for a particle the model defines.
fn particle_param<'m>(model: &'m UFOModel, block: &str, pdg: i32) -> Option<&'m str> {
    let particle = model
        .particles
        .values()
        .find(|p| p.pdg_code == i64::from(pdg))?;
    match block {
        "mass" => Some(particle.mass_param.as_str()),
        "decay" => Some(particle.width_param.as_str()),
        _ => None,
    }
}

/// One external parameter as a card binds it.
struct Bound<'a> {
    name: &'a str,
    block: &'a str,
    code: &'a [i32],
    default: f64,
    /// The value [`EvaluatedModel::from_model_card`] evaluates it at
    /// ([`ParameterSet::bound_value`]).
    value: f64,
}

/// Every external parameter of `params` as `card` binds it, in model order.
fn bound_externals<'a>(
    params: &'a ParameterSet,
    card: &'a ParamCard,
) -> impl Iterator<Item = Bound<'a>> {
    params.externals.iter().filter_map(move |p| {
        let ParamNature::External {
            default_value,
            lha_block,
            lha_code,
        } = &p.nature
        else {
            return None;
        };
        Some(Bound {
            name: &p.name,
            block: lha_block,
            code: lha_code,
            default: *default_value,
            value: params.bound_value(p, card)?,
        })
    })
}

/// A [`CardError::SplitsMerged`] for each member of a group the restriction
/// merged ([`ParameterSet::merged`]) that `card` moves off its factor times the
/// group's first: a card may move a merged group only as a whole.
fn merge_splits(params: &ParameterSet, card: &ParamCard) -> Vec<CardError> {
    let bound: Vec<Bound<'_>> = bound_externals(params, card).collect();
    let find = |name: &str| {
        bound
            .iter()
            .find(|b| b.name == name)
            .expect("a merged parameter is an external of the model")
    };
    let mut errors = Vec::new();
    for group in &params.merged {
        let first = find(&group[0].0);
        for (member, factor) in &group[1..] {
            let member = find(member);
            if member.value != factor * first.value {
                errors.push(CardError::SplitsMerged {
                    block: first.block.to_string(),
                    first: first.name.to_string(),
                    first_value: first.value,
                    member: member.name.to_string(),
                    member_value: member.value,
                    factor: *factor,
                });
            }
        }
    }
    errors
}

/// A card entry set aside for the evaluation: one the model computes, or one it
/// has no parameter for.
enum Aside {
    Computed {
        block: &'static str,
        pdg: i32,
        param: String,
        value: f64,
    },
    NoParameter {
        block: String,
        code: String,
        value: f64,
    },
    Table {
        pdg: i32,
        channels: usize,
        branching: f64,
    },
    Auto {
        pdg: i32,
    },
}

/// Whether `model` has a physical particle with the positive code `|pdg|`.
fn has_particle(model: &UFOModel, pdg: i32) -> bool {
    model
        .particles
        .values()
        .any(|p| p.pdg_code == i64::from(pdg).abs() && !p.is_unphysical())
}

/// Whether some external parameter of `model` sits in `block`, case aside.
fn defines_block(model: &UFOModel, block: &str) -> bool {
    model.params.externals.iter().any(|p| {
        matches!(&p.nature, ParamNature::External { lha_block, .. }
            if lha_block.eq_ignore_ascii_case(block))
    })
}

/// Bind `model` to `card`, refusing what the model cannot honour (see the
/// module documentation), and record what the card moved. A refusal for a
/// parameter the restriction zeroed says "the model's restriction"; use
/// [`bind_card_under`] to name it.
///
/// # Errors
///
/// [`CardError`] for the entry refused, or [`CardError::Several`] listing every
/// one: `Auto` widths and decay tables in card order, then entries in block
/// then code order, then merged groups split. A card whose entries all bind is
/// then evaluated, and [`CardError::NegativeMass`] refuses each negative mass
/// off a Majorana fermion, listed together; so a card with both kinds of
/// problem reports its entries' first.
pub fn bind_card(model: Arc<UFOModel>, card: &ParamCard) -> Result<BoundCard, CardError> {
    bind_card_named(model, card, "the model's restriction".to_string())
}

/// [`bind_card`] for a model loaded under the restriction `restriction` (the
/// `<x>` of `restrict_<x>.dat`), which a refusal of a parameter it zeroed names.
///
/// # Errors
///
/// As [`bind_card`].
pub fn bind_card_under(
    model: Arc<UFOModel>,
    card: &ParamCard,
    restriction: &str,
) -> Result<BoundCard, CardError> {
    bind_card_named(model, card, format!("restrict_{restriction}.dat"))
}

fn bind_card_named(
    model: Arc<UFOModel>,
    card: &ParamCard,
    restriction: String,
) -> Result<BoundCard, CardError> {
    let mut errors: Vec<CardError> = Vec::new();
    // A `DECAY` line at a code no particle has, with its table or its `Auto`
    // width, is one ignored entry; the other tables are held to what the header
    // can write, and the other `Auto` widths are computed.
    let (absent, present): (Vec<&DecayTable>, Vec<&DecayTable>) = card
        .decay_tables()
        .iter()
        .partition(|t| !has_particle(&model, t.pdg));
    errors.extend(unwritable_tables(
        &model,
        &present.into_iter().cloned().collect::<Vec<_>>(),
    ));
    let mut aside: Vec<Aside> = Vec::new();
    for table in absent {
        aside.push(Aside::Table {
            pdg: table.pdg,
            channels: table.channels.len(),
            branching: table.channels.iter().map(|c| c.branching_ratio).sum(),
        });
    }
    for &pdg in card.auto_widths() {
        if !has_particle(&model, pdg) {
            aside.push(Aside::Auto { pdg });
        }
    }
    let card = &card.without_absent_decays(|pdg| !has_particle(&model, pdg));
    let auto_params = auto_width_params(&model, card, &mut errors);

    for (block, code, value) in card.entries() {
        if !value.is_finite() {
            errors.push(CardError::NonFinite {
                block: block.to_string(),
                code: code_text(code),
                value,
            });
            continue;
        }
        if let Some((name, _)) = external_at(&model, block, code) {
            if model.params.zeros.contains(name) && value != 0.0 {
                errors.push(CardError::RevivesZeroed {
                    name: name.to_string(),
                    block: block.to_string(),
                    code: code_text(code),
                    value,
                    restriction: restriction.clone(),
                });
            } else if model.params.ones.contains(name) && value != 1.0 {
                errors.push(CardError::MovesFixedOne {
                    name: name.to_string(),
                    block: block.to_string(),
                    code: code_text(code),
                    value,
                });
            }
            continue;
        }
        if block == "qnumbers" || block.starts_with("qnumbers ") {
            continue;
        }
        match (block, code) {
            ("mass" | "decay", &[pdg]) => {
                let label = if block == "mass" { "MASS" } else { "DECAY" };
                match particle_param(&model, block, pdg) {
                    Some(param) => aside.push(Aside::Computed {
                        block: label,
                        pdg,
                        param: param.to_string(),
                        value,
                    }),
                    None => aside.push(Aside::NoParameter {
                        block: label.to_string(),
                        code: code_text(code),
                        value,
                    }),
                }
            }
            _ if !defines_block(&model, block) => aside.push(Aside::NoParameter {
                block: block.to_string(),
                code: code_text(code),
                value,
            }),
            _ => errors.push(CardError::Unknown {
                block: block.to_string(),
                code: code_text(code),
                value,
            }),
        }
    }
    errors.extend(merge_splits(&model.params, card));
    refuse_all(errors)?;

    let (card, auto_widths) = resolve_auto_widths(&model, card, &auto_params)?;
    let card = card.as_ref();

    let moved = bound_externals(&model.params, card)
        .filter(|b| b.value != b.default)
        .map(|b| MovedParam {
            name: b.name.to_string(),
            block: b.block.to_string(),
            code: b.code.to_vec(),
            value: b.value,
        })
        .collect();

    let evaluated = EvaluatedModel::from_model_card(model, card)?;
    refuse_all(negative_masses(&evaluated))?;
    let ignored = aside
        .into_iter()
        .filter_map(|entry| match entry {
            Aside::Computed {
                block,
                pdg,
                param,
                value,
            } => {
                let computed = evaluated.param_values.get(&param).map_or(0.0, |v| v.re);
                let off = (value - computed).abs() > DEPENDENT_TOLERANCE * computed.abs();
                off.then_some(IgnoredEntry {
                    block: block.to_string(),
                    code: pdg.to_string(),
                    card: value,
                    why: IgnoredBecause::Computed { param, computed },
                })
            }
            Aside::NoParameter { block, code, value } => Some(IgnoredEntry {
                block,
                code,
                card: value,
                why: IgnoredBecause::NoParameter,
            }),
            Aside::Auto { pdg } => Some(IgnoredEntry {
                block: "DECAY".to_string(),
                code: pdg.to_string(),
                card: 0.0,
                why: IgnoredBecause::NoParticleAuto,
            }),
            Aside::Table {
                pdg,
                channels,
                branching,
            } => Some(IgnoredEntry {
                block: "DECAY".to_string(),
                code: pdg.to_string(),
                card: branching,
                why: IgnoredBecause::NoParticleTable { channels },
            }),
        })
        .collect();

    Ok(BoundCard {
        evaluated,
        moved,
        ignored,
        auto_widths,
    })
}

/// The width parameter of each code `card` sets to `Auto`, refusing a code that
/// is not a physical particle's or whose width is not a free parameter.
fn auto_width_params(
    model: &UFOModel,
    card: &ParamCard,
    errors: &mut Vec<CardError>,
) -> BTreeSet<String> {
    let mut params = BTreeSet::new();
    for &pdg in card.auto_widths() {
        let refuse = |reason| CardError::AutoWidth { pdg, reason };
        let particle = decaying_particle(model, pdg).map(|id| model.particle(id));
        let Some(particle) = particle else {
            errors.push(refuse(AutoWidthError::UnknownParticle { pdg }));
            continue;
        };
        let not_free = |why| {
            refuse(AutoWidthError::NotFree {
                particle: particle.name.clone(),
                param: particle.width_param.clone(),
                why,
            })
        };
        let name = match external_at(model, "decay", &[pdg]) {
            Some((name, _)) if name == particle.width_param => name,
            _ if particle.width_param == "ZERO" => {
                errors.push(not_free("the particle has no width parameter"));
                continue;
            }
            _ => {
                errors.push(not_free("the model computes it from other parameters"));
                continue;
            }
        };
        let why = if model.params.zeros.contains(name) {
            Some("the restriction fixes it to zero")
        } else if model.params.ones.contains(name) {
            Some("the restriction fixes it to one")
        } else if model
            .params
            .merged
            .iter()
            .any(|g| g.iter().any(|(n, _)| n == name))
        {
            Some("the restriction ties it to other parameters")
        } else {
            None
        };
        match why {
            Some(why) => errors.push(not_free(why)),
            None => {
                params.insert(name.to_string());
            }
        }
    }
    params
}

/// `card` with every `Auto` width replaced by its computed value and decay
/// table, and the computed widths; `card` itself when it has none.
///
/// The widths are computed at the card's other parameters with each `Auto`
/// width at its restricted default: no computed channel reads one
/// ([`AutoWidthError::DependsOnAutoWidth`]), so the value is a placeholder.
fn resolve_auto_widths<'c>(
    model: &Arc<UFOModel>,
    card: &'c ParamCard,
    auto_params: &BTreeSet<String>,
) -> Result<(Cow<'c, ParamCard>, Vec<ComputedWidth>), CardError> {
    if card.auto_widths().is_empty() {
        return Ok((Cow::Borrowed(card), Vec::new()));
    }
    let placeholders: Vec<(i32, f64)> = card
        .auto_widths()
        .iter()
        .map(|&pdg| {
            let default = external_at(model, "decay", &[pdg]).map_or(0.0, |(_, d)| d);
            (pdg, default)
        })
        .collect();
    let provisional =
        EvaluatedModel::from_model_card(model.clone(), &card.with_widths_set(&placeholders, &[]))?;
    refuse_all(negative_masses(&provisional))?;

    let mut errors = Vec::new();
    let mut computed = Vec::new();
    for &pdg in card.auto_widths() {
        let width =
            compute_width(model, &provisional, pdg, auto_params).and_then(|width| {
                match card.decay_tables().iter().find(|t| t.pdg == pdg) {
                    Some(table) => check_given_table(model, &width, table).map(|()| width),
                    None => Ok(width),
                }
            });
        match width {
            Ok(width) => computed.push(width),
            Err(reason) => errors.push(CardError::AutoWidth { pdg, reason }),
        }
    }
    refuse_all(errors)?;

    let values: Vec<(i32, f64)> = computed.iter().map(|w| (w.pdg, w.bound)).collect();
    let tables: Vec<DecayTable> = computed.iter().map(ComputedWidth::decay_table).collect();
    let resolved = card.with_widths_set(&values, &tables);
    refuse_all(merge_splits(&model.params, &resolved))?;
    Ok((Cow::Owned(resolved), computed))
}

/// Bind `model` at the parameters an integration recorded: each one's value,
/// every other external at its restricted default, and the recorded decay
/// tables carried. Gives the same value for every parameter and coupling, and
/// the same tables, as the [`bind_card`] that recorded them.
///
/// # Errors
///
/// [`CardError::NotRecordable`] for a recorded parameter that is not a free
/// external of `model` at the recorded block and code, and
/// [`CardError::DecayTableParticle`] for a table whose particle the header
/// writes no line for, and [`CardError::NegativeMass`] as [`bind_card`]
/// refuses it.
pub fn bind_recorded(
    model: Arc<UFOModel>,
    params: &[MovedParam],
    decays: &[DecayTable],
) -> Result<EvaluatedModel, CardError> {
    refuse_all(unwritable_tables(&model, decays))?;
    let mut card = ParamCard::default();
    for p in params {
        let free = matches!(
            external_at(&model, &p.block, &p.code),
            Some((name, _)) if name == p.name
                && !model.params.zeros.contains(name)
                && !model.params.ones.contains(name)
        );
        if !free {
            return Err(CardError::NotRecordable {
                name: p.name.clone(),
                block: p.block.clone(),
                code: code_text(&p.code),
            });
        }
        card.insert(&p.block, p.code.clone(), p.value);
    }
    for table in decays {
        card.push_decay_table(table.clone());
    }
    let evaluated = EvaluatedModel::from_model_card(model, &card)?;
    refuse_all(negative_masses(&evaluated))?;
    Ok(evaluated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ufo::parameters::parse_parameters;
    use crate::ufo::sm::{sm_model, SMRestrict};

    fn sm() -> Arc<UFOModel> {
        sm_model(SMRestrict::Default)
    }

    fn card(text: &str) -> ParamCard {
        text.parse().unwrap()
    }

    fn bind(text: &str) -> Result<BoundCard, CardError> {
        bind_card(sm(), &card(text))
    }

    /// Every parameter and every coupling of two evaluations, bit for bit.
    fn assert_same_point(a: &EvaluatedModel, b: &EvaluatedModel) {
        let mut names: Vec<&String> = a.param_values.keys().collect();
        names.sort();
        for name in names {
            let (x, y) = (a.param_values[name], b.param_values[name]);
            assert!(
                x.re.to_bits() == y.re.to_bits() && x.im.to_bits() == y.im.to_bits(),
                "{name}: {x} against {y}"
            );
        }
        for i in 0..a.model().couplings.len() {
            let id = crate::ufo::couplings::CouplingId::from(i);
            let (x, y) = (a.coupling(id), b.coupling(id));
            assert!(
                x.re.to_bits() == y.re.to_bits() && x.im.to_bits() == y.im.to_bits(),
                "coupling {i}: {x} against {y}"
            );
        }
    }

    /// No card, an empty card and a card restating the defaults move nothing,
    /// and evaluate at the defaults.
    #[test]
    fn a_card_at_the_defaults_moves_nothing() {
        let defaults = EvaluatedModel::from_model(sm());
        for text in [
            "",
            "Block MASS\n 23 9.118800e+01 # MZ\n 6 1.730000e+02 # MT\nDECAY 23 2.441404e+00\n",
        ] {
            let bound = bind(text).unwrap();
            assert!(bound.moved.is_empty(), "{text:?}: {:?}", bound.moved);
            assert!(bound.ignored.is_empty(), "{text:?}: {:?}", bound.ignored);
            assert_same_point(&bound.evaluated, &defaults);
        }
    }

    /// MadGraph's own card for the default SM restriction, as it writes it into
    /// a process directory, binds without a refusal or a warning: its computed
    /// entries (MW, the massless particles) agree with the model's to the
    /// digits MadGraph prints. It is not the defaults, though: MadGraph writes
    /// every value with `%e`, and the restriction's Higgs width,
    /// `6.38233934e-03`, has two digits more than that keeps, so the card MadGraph
    /// runs at moves WH to `6.382339e-03`, and so does this binding.
    #[test]
    fn madgraphs_own_card_moves_only_the_width_it_rounds() {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../validation/madgraph/dy13_param_card.dat"
        ))
        .unwrap();
        let bound = bind(&text).unwrap();
        assert!(bound.ignored.is_empty(), "{:?}", bound.ignored);
        assert_eq!(
            bound.moved,
            vec![MovedParam {
                name: "WH".to_string(),
                block: "DECAY".to_string(),
                code: vec![25],
                value: 6.382339e-03,
            }]
        );
    }

    /// A moved MZ is the one parameter recorded, at the card's value; the
    /// recorded point rebinds to the same value of every parameter and
    /// coupling, and MW, computed from MZ, moves with it.
    #[test]
    fn a_moved_parameter_is_recorded_and_rebinds_exactly() {
        let bound = bind("Block MASS\n 23 92.5\n").unwrap();
        assert_eq!(
            bound.moved,
            vec![MovedParam {
                name: "MZ".to_string(),
                block: "MASS".to_string(),
                code: vec![23],
                value: 92.5,
            }]
        );
        let rebound = bind_recorded(sm(), &bound.moved, &[]).unwrap();
        assert_same_point(&bound.evaluated, &rebound);
        let defaults = EvaluatedModel::from_model(sm());
        assert_ne!(
            rebound.param_values["MW"].re,
            defaults.param_values["MW"].re
        );
    }

    /// Two moves are recorded in the model's external order, not the card's.
    #[test]
    fn moves_are_recorded_in_the_models_external_order() {
        let bound = bind("Block YUKAWA\n 6 250\nBlock MASS\n 23 92.5\n").unwrap();
        let names: Vec<&str> = bound.moved.iter().map(|m| m.name.as_str()).collect();
        let model = sm();
        let order: Vec<&str> = model
            .params
            .externals
            .iter()
            .map(|p| p.name.as_str())
            .filter(|n| names.contains(n))
            .collect();
        assert_eq!(names, order);
        assert_eq!(names.len(), 2);
    }

    /// `DECAY 24 Auto` binds WW to the computed two-body width, to the digits
    /// MadGraph writes, records the move and the computed table, and gives the
    /// same evaluation, moves and table as the card with that number and table
    /// typed in.
    #[test]
    fn an_auto_width_binds_like_the_number_it_computes() {
        let bound = bind("DECAY 24 Auto\n").unwrap();
        let [width] = bound.auto_widths.as_slice() else {
            panic!("{:?}", bound.auto_widths)
        };
        assert_eq!(width.param, "WW");
        assert_eq!(width.channels.len(), 5);
        assert!(width.not_included.is_empty());
        assert!((width.bound / width.total - 1.0).abs() < 5e-7);
        let ww = bound.evaluated.param_values["WW"].re;
        assert_eq!(ww, width.bound);
        assert_eq!(
            bound.moved,
            vec![MovedParam {
                name: "WW".to_string(),
                block: "DECAY".to_string(),
                code: vec![24],
                value: width.bound,
            }]
        );
        let table = &bound.evaluated.decay_tables()[0];
        assert_eq!(table.pdg, 24);
        assert_eq!(table.channels.len(), 5);
        let sum: f64 = table.channels.iter().map(|c| c.branching_ratio).sum();
        assert!((sum - 1.0).abs() < 1e-5, "{sum}");

        let mut typed = format!("DECAY 24 {:e}\n", width.bound);
        for c in &table.channels {
            typed.push_str(&format!(
                " {:e} 2 {} {}\n",
                c.branching_ratio, c.daughters[0], c.daughters[1]
            ));
        }
        let by_hand = bind(&typed).unwrap();
        assert_same_point(&bound.evaluated, &by_hand.evaluated);
        assert_eq!(bound.moved, by_hand.moved);
        assert_eq!(
            bound.evaluated.decay_tables(),
            by_hand.evaluated.decay_tables()
        );

        let resolved = crate::widths::resolved_card_text("DECAY 24 Auto\n", &bound.auto_widths);
        let rebound = bind(&resolved).unwrap();
        assert!(rebound.auto_widths.is_empty(), "{resolved}");
        assert_same_point(&bound.evaluated, &rebound.evaluated);
        assert_eq!(bound.moved, rebound.moved);
        assert_eq!(
            bound.evaluated.decay_tables(),
            rebound.evaluated.decay_tables()
        );
    }

    /// Each `Auto` width that cannot be computed as MadGraph would is refused,
    /// naming why, and every one is listed at once.
    #[test]
    fn auto_widths_that_cannot_be_computed_are_refused_naming_why() {
        let why = |text: &str| match bind(text) {
            Err(CardError::AutoWidth { pdg, reason }) => (pdg, reason),
            other => panic!("{text:?}: {:?}", other.map(|b| b.auto_widths)),
        };
        assert!(matches!(
            why("DECAY 22 Auto\n"),
            (
                22,
                AutoWidthError::NotFree {
                    why: "the particle has no width parameter",
                    ..
                }
            )
        ));
        assert!(matches!(
            why("DECAY -24 Auto\n"),
            (-24, AutoWidthError::UnknownParticle { pdg: -24 })
        ));
        // A particle with a three-body mode through a closed two-body channel
        // is refused naming it: a top lighter than W + b (through W+), the Z
        // (Z > W+ e- ve~ through W-) and the Higgs boson (H > W+ e- ve~
        // through W-).
        for (card, pdg, mode) in [
            (
                "Block MASS\n 6 80.0\nDECAY 6 Auto\n",
                6,
                "t > ve b e+ (through W+)",
            ),
            ("DECAY 23 Auto\n", 23, "(through W-)"),
            ("DECAY 25 Auto\n", 25, "(through W-)"),
        ] {
            let (got, reason) = why(card);
            assert_eq!(got, pdg);
            let AutoWidthError::BeyondTwoBody { modes, .. } = &reason else {
                panic!("{reason:?}")
            };
            assert!(modes.contains(mode), "{modes}");
            let text = reason.to_string();
            assert!(
                text.contains("two-body widths only") && text.contains("may include such modes"),
                "{text}"
            );
        }
        // A table below the Auto line naming a mode no tree-level vertex gives,
        // or one beyond two bodies.
        for (row, says) in [("2 22 22", "loop-induced"), ("3 5 -5 21", "more than two")] {
            let (pdg, reason) = why(&format!("DECAY 24 Auto\n 1.0 {row}\n"));
            assert_eq!(pdg, 24);
            assert!(
                matches!(reason, AutoWidthError::ModeNotComputed { .. })
                    && reason.to_string().contains(says),
                "{row}: {reason}"
            );
        }
        match bind("DECAY 22 Auto\nDECAY -23 Auto\n") {
            Err(CardError::Several(all)) => assert_eq!(all.len(), 2),
            other => panic!("{:?}", other.map(|b| b.auto_widths)),
        }
    }

    /// A table the card gives below an `Auto` line, naming channels the
    /// computation has, is replaced by the computed one, as MadGraph replaces
    /// it, both when binding and in the card text a run banks; a particle with
    /// no channel open and none beyond two bodies binds to zero, positive zero
    /// as MadGraph writes it, as MadGraph's stable particle does.
    #[test]
    fn a_given_table_is_replaced_and_a_stable_particle_is_zero() {
        let given = "DECAY 24 Auto\n 0.5 2 12 -11\n 0.5 2 14 -13\n";
        let bound = bind(given).unwrap();
        let table = &bound.evaluated.decay_tables()[0];
        assert_eq!(table.channels.len(), 5);
        assert!(table.channels.iter().all(|c| c.branching_ratio < 0.5));
        let resolved = crate::widths::resolved_card_text(given, &bound.auto_widths);
        assert!(!resolved.contains(" 0.5 2 "), "{resolved}");
        let rebound = bind(&resolved).unwrap();
        assert_eq!(
            rebound.evaluated.decay_tables(),
            bound.evaluated.decay_tables(),
            "{resolved}"
        );

        let light = bind("Block MASS\n 25 1.0\nDECAY 25 Auto\n").unwrap();
        let width = &light.auto_widths[0];
        assert_eq!((width.total, width.bound), (0.0, 0.0));
        assert!(
            width.total.is_sign_positive() && width.bound.is_sign_positive(),
            "{width:?}"
        );
        let resolved = crate::widths::resolved_card_text("DECAY 25 Auto\n", &light.auto_widths);
        assert!(
            resolved.starts_with("DECAY  25   0.000000e+00 "),
            "{resolved}"
        );
        assert!(width.channels.is_empty() && width.not_included.is_empty());
        assert_eq!(light.evaluated.param_values["WH"].re, 0.0);
        assert!(light.evaluated.decay_tables().is_empty());
    }

    /// The default restriction fixes the charm mass to zero; a card giving it a
    /// mass is refused, and one restating zero is not.
    #[test]
    fn reviving_a_zeroed_parameter_is_refused() {
        let err = bind("Block MASS\n 4 1.27 # MC\n").err().unwrap();
        assert!(
            matches!(&err, CardError::RevivesZeroed { name, value, .. } if name == "MC" && *value == 1.27),
            "{err}"
        );
        assert!(bind("Block MASS\n 4 0.0 # MC\n").unwrap().moved.is_empty());
    }

    /// A code the model lacks inside a block it defines is refused, as a
    /// mistyped code would leave the parameter meant at its default; the block
    /// that holds it is one some parameter of the model sits in.
    #[test]
    fn an_unknown_code_in_a_defined_block_is_refused() {
        for text in ["Block SMINPUTS\n 9 2.0\n", "Block YUKAWA\n 99 2.0\n"] {
            assert!(
                matches!(bind(text), Err(CardError::Unknown { .. })),
                "{text:?}"
            );
        }
        // A new state's quantum numbers are not parameters.
        assert!(bind("Block QNUMBERS 9000005 # x\n 1 0\n 2 1\n").is_ok());
        assert!(
            bind("Block QNUMBERS 9000005 # x\n 1 0\nBlock QNUMBERS 9000006 # y\n 1 3\n").is_ok()
        );
    }

    /// `DECAY <code> Auto` at a code no particle has is one ignored entry, like
    /// any other `DECAY` line there (nothing is computed, no width is moved);
    /// at a code the model has it is computed as before, and the two together
    /// give the computed width and the one ignored entry.
    #[test]
    fn an_auto_width_at_an_absent_code_is_ignored_and_at_a_present_one_computed() {
        let only_absent = bind("DECAY 1000022 Auto\n").unwrap();
        assert!(only_absent.auto_widths.is_empty() && only_absent.moved.is_empty());
        assert_same_point(&only_absent.evaluated, &EvaluatedModel::from_model(sm()));
        assert_eq!(
            only_absent.ignored,
            [IgnoredEntry {
                block: "DECAY".to_string(),
                code: "1000022".to_string(),
                card: 0.0,
                why: IgnoredBecause::NoParticleAuto,
            }]
        );

        let both = bind("DECAY 1000022 Auto\n 1.0 2 11 -11\nDECAY 24 Auto\n").unwrap();
        let alone = bind("DECAY 24 Auto\n").unwrap();
        assert_eq!(both.auto_widths.len(), 1);
        assert_eq!(both.auto_widths[0].bound, alone.auto_widths[0].bound);
        assert_eq!(both.moved, alone.moved);
        assert_eq!(
            both.evaluated.decay_tables(),
            alone.evaluated.decay_tables()
        );
        assert!(both
            .ignored
            .iter()
            .any(|e| e.why == IgnoredBecause::NoParticleAuto && e.code == "1000022"));
        assert!(both
            .ignored
            .iter()
            .any(|e| matches!(e.why, IgnoredBecause::NoParticleTable { .. })));
    }

    /// A decay table below a `DECAY` line at a code no particle has is ignored
    /// with the line, listed once with its channel count, and not carried; a
    /// table at a code the model has is carried as before.
    #[test]
    fn a_decay_table_for_an_absent_particle_is_ignored() {
        let bound =
            bind("DECAY 1000022 1.0\n 0.6 2 11 -11\n 0.4 2 13 -13\nDECAY 6 1.5\n 1.0 2 5 24\n")
                .unwrap();
        assert_eq!(bound.evaluated.decay_tables().len(), 1);
        assert_eq!(bound.evaluated.decay_tables()[0].pdg, 6);
        let table = bound
            .ignored
            .iter()
            .find(|e| matches!(e.why, IgnoredBecause::NoParticleTable { .. }))
            .expect("the table is listed");
        assert_eq!(
            (table.block.as_str(), table.code.as_str()),
            ("DECAY", "1000022")
        );
        assert_eq!(table.why, IgnoredBecause::NoParticleTable { channels: 2 });
        assert!((table.card - 1.0).abs() < 1e-12);
        assert!(bound
            .ignored
            .iter()
            .any(|e| e.why == IgnoredBecause::NoParameter));
        // An antiparticle's code the model has is still refused.
        assert!(matches!(
            bind("DECAY -24 2.085\n 1.0 2 11 -12\n"),
            Err(CardError::DecayTableParticle { pdg: -24 })
        ));
    }

    /// A block no parameter of the model sits in, and a `MASS` or `DECAY` entry
    /// at a code no particle has, bind as the card without them and are listed.
    #[test]
    fn entries_with_no_parameter_are_ignored_and_listed() {
        let bound = bind(
            "Block NOSUCH\n 1 2.0\nBlock MASS\n 99 2.0\nBlock ALPHA\n -1.1e-1\nDECAY 99 1.0\n",
        )
        .unwrap();
        assert!(bound.moved.is_empty());
        assert_same_point(&bound.evaluated, &EvaluatedModel::from_model(sm()));
        let listed: Vec<(&str, &str, f64, &IgnoredBecause)> = bound
            .ignored
            .iter()
            .map(|e| (e.block.as_str(), e.code.as_str(), e.card, &e.why))
            .collect();
        let none = &IgnoredBecause::NoParameter;
        assert_eq!(
            listed,
            [
                ("alpha", "", -1.1e-1, none),
                ("DECAY", "99", 1.0, none),
                ("MASS", "99", 2.0, none),
                ("nosuch", "1", 2.0, none),
            ]
        );
    }

    #[test]
    fn a_non_finite_value_is_refused() {
        assert!(matches!(
            bind("Block MASS\n 23 nan\n"),
            Err(CardError::NonFinite { .. })
        ));
    }

    /// The W mass is computed from MZ, aEWM1 and Gf: a card value for it is
    /// ignored, reported with the computed value, and recorded nowhere; the
    /// massless particles' entries likewise.
    #[test]
    fn a_dependent_mass_is_ignored_and_reported() {
        let bound = bind("Block MASS\n 24 80.0\n 1 0.005\nDECAY 22 1.0\n").unwrap();
        assert!(bound.moved.is_empty());
        let defaults = EvaluatedModel::from_model(sm());
        assert_same_point(&bound.evaluated, &defaults);
        let reported: Vec<(&str, &str, i32, f64)> = bound
            .ignored
            .iter()
            .map(|e| match &e.why {
                IgnoredBecause::Computed { param, .. } => (
                    param.as_str(),
                    e.block.as_str(),
                    e.code.parse().unwrap(),
                    e.card,
                ),
                _ => panic!("{e:?}"),
            })
            .collect();
        assert_eq!(
            reported,
            vec![
                ("ZERO", "DECAY", 22, 1.0),
                ("ZERO", "MASS", 1, 0.005),
                ("MW", "MASS", 24, 80.0)
            ]
        );
        let IgnoredBecause::Computed { computed, .. } = &bound.ignored[2].why else {
            panic!("{:?}", bound.ignored[2])
        };
        assert_eq!(*computed, defaults.param_values["MW"].re);
    }

    /// A computed entry within `%e`'s rounding of the computed value (5e-7
    /// relative, the width of MadGraph's own entries) is not reported, and one
    /// beyond [`DEPENDENT_TOLERANCE`] is, on either side.
    #[test]
    fn a_dependent_entry_is_reported_beyond_the_cards_rounding() {
        let mw = EvaluatedModel::from_model(sm()).param_values["MW"].re;
        let reported = |rel: f64| {
            bind(&format!("Block MASS\n 24 {:e}\n", mw * (1.0 + rel)))
                .unwrap()
                .ignored
                .len()
        };
        for rel in [4e-7, -4e-7] {
            assert_eq!(reported(rel), 0, "MW off by {rel:e}");
        }
        for rel in [2e-6, -2e-6] {
            assert_eq!(reported(rel), 1, "MW off by {rel:e}");
        }
    }

    /// A model whose restriction gives two parameters of one block the same
    /// default, and a third the opposite one, merges them as MadGraph does;
    /// a card must keep the relation.
    #[test]
    fn splitting_a_merged_pair_is_refused() {
        let src = r"
a = Parameter(name = 'a', nature = 'external', type = 'real', value = 0.3, texname = 'a', lhablock = 'NP', lhacode = [ 1 ])
b = Parameter(name = 'b', nature = 'external', type = 'real', value = 0.4, texname = 'b', lhablock = 'NP', lhacode = [ 2 ])
c = Parameter(name = 'c', nature = 'external', type = 'real', value = 0.5, texname = 'c', lhablock = 'NP', lhacode = [ 3 ])
d = Parameter(name = 'd', nature = 'external', type = 'real', value = 0.6, texname = 'd', lhablock = 'OTHER', lhacode = [ 1 ])
w = Parameter(name = 'w', nature = 'external', type = 'real', value = 0.7, texname = 'w', lhablock = 'DECAY', lhacode = [ 5 ])
v = Parameter(name = 'v', nature = 'external', type = 'real', value = 0.8, texname = 'v', lhablock = 'DECAY', lhacode = [ 6 ])
";
        let mut params = parse_parameters(src).unwrap();
        params.apply_restrict(&card(
            "Block NP\n 1 0.25\n 2 0.25\n 3 -0.25\nBlock OTHER\n 1 0.25\nDECAY 5 0.25\nDECAY 6 0.25\n",
        ));
        assert_eq!(
            params.merged,
            vec![vec![
                ("a".to_string(), 1.0),
                ("b".to_string(), 1.0),
                ("c".to_string(), -1.0)
            ]]
        );
        let check = |text: &str| refuse_all(merge_splits(&params, &card(text)));

        let err = check("Block NP\n 2 0.5\n").err().unwrap();
        assert!(
            matches!(&err, CardError::SplitsMerged { first, member, .. } if first == "a" && member == "b"),
            "{err}"
        );
        let err = check("Block NP\n 1 0.5\n 2 0.5\n").err().unwrap();
        assert!(
            matches!(&err, CardError::SplitsMerged { member, factor, .. } if member == "c" && *factor == -1.0),
            "{err}"
        );
        assert!(check("Block NP\n 1 0.5\n 2 0.5\n 3 -0.5\n").is_ok());
        assert!(check("").is_ok());
        // Another block, and widths, are never merged.
        assert!(check("Block OTHER\n 1 0.5\nDECAY 5 0.5\n").is_ok());
    }

    /// `bind_card` applies the merge check: on the SM with MT and ymt (equal
    /// defaults) tied as a merged pair, moving one alone is refused and moving
    /// both is recorded.
    #[test]
    fn bind_card_refuses_a_split_merge() {
        let mut model = (*sm()).clone();
        model.params.merged = vec![vec![("MT".to_string(), 1.0), ("ymt".to_string(), 1.0)]];
        let model = Arc::new(model);
        assert!(matches!(
            bind_card(model.clone(), &card("Block YUKAWA\n 6 250\n")),
            Err(CardError::SplitsMerged { .. })
        ));
        let both = bind_card(model, &card("Block YUKAWA\n 6 250\nBlock MASS\n 6 250\n")).unwrap();
        assert_eq!(both.moved.len(), 2);
    }

    /// The card the `dy13_mmll_60_120_mz95` row is banked on, MadGraph's own
    /// with the `MZ` line set to 95 GeV, moves `MZ` and the width MadGraph
    /// rounds, and nothing else; its `MW` line (80.419, MadGraph's value at the
    /// default MZ) is ignored and reported against the 85.036 GeV the model
    /// computes at MZ = 95, as MadGraph's Fortran recomputes it.
    #[test]
    fn the_mz95_banking_card_moves_mz_and_the_rounded_width() {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../validation/madgraph/dy13_mz95_param_card.dat"
        ))
        .unwrap();
        let bound = bind(&text).unwrap();
        let moved: Vec<(&str, f64)> = bound
            .moved
            .iter()
            .map(|p| (p.name.as_str(), p.value))
            .collect();
        assert_eq!(moved, [("MZ", 95.0), ("WH", 6.382339e-03)]);
        let ignored: Vec<(&str, i32, f64)> = bound
            .ignored
            .iter()
            .map(|e| match &e.why {
                IgnoredBecause::Computed { param, .. } => {
                    (param.as_str(), e.code.parse().unwrap(), e.card)
                }
                _ => panic!("{e:?}"),
            })
            .collect();
        assert_eq!(ignored, [("MW", 24, 80.419)]);
        let IgnoredBecause::Computed { computed, .. } = &bound.ignored[0].why else {
            panic!("{:?}", bound.ignored[0])
        };
        assert!((computed - 85.03597559307168).abs() < 1e-9);
    }

    /// The toy model's `restrict_all` leaves the colour-octet width `WO8` at
    /// exactly one, which MadGraph's restriction fixes (a `one` rule in the
    /// process's `param_card_rule.dat`): a card that moves it is refused, one
    /// that restates it binds and records nothing, and a record that names it
    /// does not rebind.
    #[test]
    fn moving_a_parameter_the_restriction_fixes_to_one_is_refused() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/ufo");
        let toy = UFOModel::load(
            &root.join("vibegraph_toy_UFO"),
            Some(&root.join("vibegraph_toy_UFO/restrict_all.dat")),
        )
        .unwrap();
        assert!(toy.params.ones.contains("WO8"), "{:?}", toy.params.ones);

        let err = bind_card(toy.clone(), &card("DECAY 9000021 2.0\n")).err();
        assert!(
            matches!(&err, Some(CardError::MovesFixedOne { name, value, .. }) if name == "WO8" && *value == 2.0),
            "{err:?}"
        );
        let kept = bind_card(toy.clone(), &card("DECAY 9000021 1.000000e+00\n")).unwrap();
        assert!(kept.moved.is_empty(), "{:?}", kept.moved);
        // A width beside it that the restriction did not fix stays free.
        let moved = bind_card(toy.clone(), &card("DECAY 9000023 2.5\n")).unwrap();
        assert_eq!(moved.moved.len(), 1);

        let record = MovedParam {
            name: "WO8".to_string(),
            block: "DECAY".to_string(),
            code: vec![9000021],
            value: 2.0,
        };
        assert!(matches!(
            bind_recorded(toy, &[record], &[]),
            Err(CardError::NotRecordable { .. })
        ));
    }

    /// Only a restriction value of exactly one fixes a parameter: MadGraph's
    /// "keep this parameter" value `9.999999e-01` does not, and neither does a
    /// `parameters.py` default of one with no restriction applied.
    #[test]
    fn only_a_restricted_value_of_exactly_one_is_fixed() {
        let src = r"
a = Parameter(name = 'a', nature = 'external', type = 'real', value = 1.0, texname = 'a', lhablock = 'NP', lhacode = [ 1 ])
b = Parameter(name = 'b', nature = 'external', type = 'real', value = 0.3, texname = 'b', lhablock = 'NP', lhacode = [ 2 ])
c = Parameter(name = 'c', nature = 'external', type = 'real', value = 0.3, texname = 'c', lhablock = 'NP', lhacode = [ 3 ])
";
        let unrestricted = parse_parameters(src).unwrap();
        assert!(unrestricted.ones.is_empty());
        let mut params = unrestricted.clone();
        params.apply_restrict(&card("Block NP\n 2 1.0\n 3 9.999999e-01\n"));
        let ones: Vec<&str> = params.ones.iter().map(String::as_str).collect();
        assert_eq!(ones, ["a", "b"]);
    }

    /// No restriction of the interned SM leaves an external at one, so the
    /// refusal above never fires on an SM card.
    #[test]
    fn the_interned_sm_fixes_nothing_to_one() {
        for restrict in SMRestrict::ALL.iter().copied() {
            let model = sm_model(restrict);
            assert!(
                model.params.ones.is_empty(),
                "{restrict:?}: {:?}",
                model.params.ones
            );
        }
    }

    /// No restriction of the interned SM merges a parameter: its blocks hold no
    /// two equal defaults (MB and ymb are equal, but in different blocks).
    #[test]
    fn the_interned_sm_merges_nothing() {
        for restrict in SMRestrict::ALL.iter().copied() {
            let model = sm_model(restrict);
            assert!(
                model.params.merged.is_empty(),
                "{restrict:?}: {:?}",
                model.params.merged
            );
        }
    }

    /// A recorded parameter this model does not leave free is refused.
    #[test]
    fn a_recorded_parameter_the_model_lacks_is_refused() {
        for (name, block, code) in [("MC", "MASS", 4), ("MZ", "MASS", 24), ("XX", "MASS", 23)] {
            let moved = [MovedParam {
                name: name.to_string(),
                block: block.to_string(),
                code: vec![code],
                value: 1.0,
            }];
            assert!(
                matches!(
                    bind_recorded(sm(), &moved, &[]),
                    Err(CardError::NotRecordable { .. })
                ),
                "{name}"
            );
        }
    }

    /// A card's decay tables are carried on the evaluation and recorded tables
    /// rebind to the same tables and the same point; the width beside a table
    /// binds as it does without one.
    #[test]
    fn decay_tables_are_carried_and_rebind() {
        let bound =
            bind("DECAY 6 1.5\n 0.6 2 5 24\n 0.4 2 24 3\nDECAY 15 0.0\n 1.0 2 16 -24\n").unwrap();
        let tables = bound.evaluated.decay_tables().to_vec();
        assert_eq!(
            tables.iter().map(|t| t.pdg).collect::<Vec<_>>(),
            [6, 15],
            "{tables:?}"
        );
        let plain = bind("DECAY 6 1.5\n").unwrap();
        assert_eq!(bound.moved, plain.moved);
        assert_same_point(&bound.evaluated, &plain.evaluated);
        let rebound = bind_recorded(sm(), &bound.moved, &tables).unwrap();
        assert_eq!(rebound.decay_tables(), tables.as_slice());
        assert_same_point(&bound.evaluated, &rebound);
    }

    /// A recorded table whose particle the header has no line for is refused
    /// as a card's is.
    #[test]
    fn a_recorded_table_on_a_negative_code_is_refused() {
        let table = DecayTable {
            pdg: -24,
            channels: Vec::new(),
        };
        assert_eq!(
            bind_recorded(sm(), &[], &[table]).err(),
            Some(CardError::DecayTableParticle { pdg: -24 })
        );
    }

    /// A negative mass on a particle that is not a Majorana fermion is refused,
    /// naming the particle and the unvalidated sites, from a card and from a
    /// recorded point alike.
    #[test]
    fn a_negative_mass_off_a_majorana_fermion_is_refused() {
        let err = bind("Block MASS\n 23 -9.1188e1\n").err().unwrap();
        assert!(
            matches!(&err, CardError::NegativeMass { particle, pdg: 23, param, .. }
                if particle == "Z" && param == "MZ"),
            "{err:?}"
        );
        assert!(err.to_string().contains("not validated"), "{err}");
        let moved = MovedParam {
            name: "MZ".to_string(),
            block: "MASS".to_string(),
            code: vec![23],
            value: -91.188,
        };
        assert!(matches!(
            bind_recorded(sm(), &[moved], &[]),
            Err(CardError::NegativeMass { pdg: 23, .. })
        ));
    }

    /// Below the binding, the evaluation negates the width at a negative mass's
    /// code and nothing else, and keeps the card's value for the header.
    #[test]
    fn the_evaluation_negates_the_width_beside_a_negative_mass() {
        let evaluated =
            EvaluatedModel::from_model_card(sm(), &card("Block MASS\n 23 -9.1188e1\n")).unwrap();
        let defaults = EvaluatedModel::from_model(sm());
        let wz = defaults.param_values["WZ"].re;
        assert!(wz > 0.0);
        assert_eq!(evaluated.param_values["WZ"].re, -wz);
        assert_eq!(evaluated.card_value("WZ"), wz);
        for name in ["WW", "WT", "WH"] {
            assert_eq!(
                evaluated.param_values[name], defaults.param_values[name],
                "{name}"
            );
        }
    }
}
