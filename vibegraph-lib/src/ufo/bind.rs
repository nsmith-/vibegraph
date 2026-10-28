//! Binding a restricted model to a user's SLHA parameter card.
//!
//! [`EvaluatedModel::from_model_card`] takes each external parameter's value
//! from the card where it has one and the restricted default otherwise, and
//! ignores every other entry. That is the evaluation; [`bind_card`] is the
//! check around it that a run at a card must pass first, and the record of
//! what the card moved:
//!
//! * `DECAY <pdg> Auto` is refused: MadGraph computes such a width at launch
//!   (MadWidth) and writes the number into the card it runs from, which is the
//!   card to give here.
//! * A non-zero value for a parameter the restriction fixed to zero is refused:
//!   the restriction also pruned the vertices that parameter feeds, so the
//!   model could not honour it.
//! * Moving one parameter of a group the restriction merged
//!   ([`ParameterSet::merged`](super::parameters::ParameterSet)) without the
//!   others is refused: MadGraph evaluates the others as the group's first.
//! * A `MASS` or `DECAY` entry for a particle whose mass or width is not an
//!   external parameter (MW in the SM, or a massless particle's `ZERO`) is
//!   ignored, as MadGraph's Fortran ignores it; one that disagrees with the
//!   computed value is reported in [`BoundCard::ignored`].
//! * Any other entry the model has no parameter for is refused, except the
//!   `QNUMBERS` blocks that declare a new state's quantum numbers.

use std::sync::Arc;

use thiserror::Error;

use super::parameters::{ParamNature, ParameterSet};
use super::slha::ParamCard;
use super::{EvaluatedModel, UFOModel};
use crate::artifact::MovedParam;

/// How far a card's entry for a computed mass or width may sit from the value
/// the model computes, relative, before it is reported. MadGraph writes those
/// entries with `%e`, seven significant digits, so its own card sits within
/// `5e-7` of the computed value.
const DEPENDENT_TOLERANCE: f64 = 1e-6;

/// Why a parameter card cannot be bound to a model.
#[derive(Clone, Debug, PartialEq, Error)]
pub enum CardError {
    #[error(
        "the card sets the width of particle {pdg} to Auto, which MadGraph computes at launch; \
         computing Auto widths is not supported yet, so give the width as a number, or use the \
         param_card.dat MadGraph wrote for the run (a gridpack's Cards/param_card.dat), where \
         it has been computed"
    )]
    AutoWidth { pdg: i32 },
    #[error(
        "the card sets {name} (block {block}, code {code}) to {value:e}, but the model's \
         restriction fixes it to zero and pruned the vertices it feeds; use a restriction that \
         keeps {name} free, or leave it at zero"
    )]
    RevivesZeroed {
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
         there"
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
}

/// A card entry for a mass or width the model computes, which the binding
/// ignored because its value disagrees with the computed one.
#[derive(Clone, Debug, PartialEq)]
pub struct IgnoredEntry {
    /// The parameter the model computes the value from (`MW`, `ZERO`).
    pub param: String,
    /// `MASS` or `DECAY`.
    pub block: &'static str,
    pub pdg: i32,
    /// The card's value.
    pub card: f64,
    /// The value the run uses.
    pub computed: f64,
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
}

fn code_text(code: &[i32]) -> String {
    code.iter()
        .map(i32::to_string)
        .collect::<Vec<_>>()
        .join(" ")
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
    /// The card's value, or the default where the card has none or the
    /// restriction fixed the parameter to zero.
    value: f64,
}

/// Every external parameter of `params` as `card` binds it, in model order:
/// the value [`EvaluatedModel::from_model_card`] evaluates it at.
fn bound_externals<'a>(
    params: &'a ParameterSet,
    card: &'a ParamCard,
) -> impl Iterator<Item = Bound<'a>> {
    params
        .externals
        .iter()
        .filter_map(move |p| match &p.nature {
            ParamNature::External {
                default_value,
                lha_block,
                lha_code,
            } => Some(Bound {
                name: &p.name,
                block: lha_block,
                code: lha_code,
                default: *default_value,
                value: if params.zeros.contains(&p.name) {
                    *default_value
                } else {
                    card.get(lha_block, lha_code).unwrap_or(*default_value)
                },
            }),
            ParamNature::Internal { .. } => None,
        })
}

/// Refuse a card whose values break a relation the restriction merged
/// ([`ParameterSet::merged`]): every member must stay its factor times the
/// group's first.
fn check_merged(params: &ParameterSet, card: &ParamCard) -> Result<(), CardError> {
    let bound: Vec<Bound<'_>> = bound_externals(params, card).collect();
    let find = |name: &str| {
        bound
            .iter()
            .find(|b| b.name == name)
            .expect("a merged parameter is an external of the model")
    };
    for group in &params.merged {
        let first = find(&group[0].0);
        for (member, factor) in &group[1..] {
            let member = find(member);
            if member.value != factor * first.value {
                return Err(CardError::SplitsMerged {
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
    Ok(())
}

/// Bind `model` to `card`, refusing what the model cannot honour (see the
/// module documentation), and record what the card moved.
///
/// # Errors
///
/// [`CardError`] for the first entry refused, entries taken in block then code
/// order, after any `Auto` width.
pub fn bind_card(model: Arc<UFOModel>, card: &ParamCard) -> Result<BoundCard, CardError> {
    if let Some(&pdg) = card.auto_widths().first() {
        return Err(CardError::AutoWidth { pdg });
    }

    let mut dependents: Vec<(&'static str, i32, String, f64)> = Vec::new();
    for (block, code, value) in card.entries() {
        if !value.is_finite() {
            return Err(CardError::NonFinite {
                block: block.to_string(),
                code: code_text(code),
                value,
            });
        }
        if let Some((name, _)) = external_at(&model, block, code) {
            if model.params.zeros.contains(name) && value != 0.0 {
                return Err(CardError::RevivesZeroed {
                    name: name.to_string(),
                    block: block.to_string(),
                    code: code_text(code),
                    value,
                });
            }
            continue;
        }
        if block == "qnumbers" {
            continue;
        }
        let dependent = match (block, code) {
            ("mass", &[pdg]) => particle_param(&model, block, pdg).map(|p| ("MASS", pdg, p)),
            ("decay", &[pdg]) => particle_param(&model, block, pdg).map(|p| ("DECAY", pdg, p)),
            _ => None,
        };
        match dependent {
            Some((block, pdg, param)) => dependents.push((block, pdg, param.to_string(), value)),
            None => {
                return Err(CardError::Unknown {
                    block: block.to_string(),
                    code: code_text(code),
                    value,
                })
            }
        }
    }

    let moved = bound_externals(&model.params, card)
        .filter(|b| b.value != b.default)
        .map(|b| MovedParam {
            name: b.name.to_string(),
            block: b.block.to_string(),
            code: b.code.to_vec(),
            value: b.value,
        })
        .collect();
    check_merged(&model.params, card)?;

    let evaluated = EvaluatedModel::from_model_card(model, card);
    let ignored = dependents
        .into_iter()
        .filter_map(|(block, pdg, param, value)| {
            let computed = evaluated.param_values.get(&param).map_or(0.0, |v| v.re);
            let off = (value - computed).abs() > DEPENDENT_TOLERANCE * computed.abs();
            off.then_some(IgnoredEntry {
                param,
                block,
                pdg,
                card: value,
                computed,
            })
        })
        .collect();

    Ok(BoundCard {
        evaluated,
        moved,
        ignored,
    })
}

/// Bind `model` at the parameters an integration recorded: each one's value,
/// every other external at its restricted default. Gives the same value for
/// every parameter and coupling as the [`bind_card`] that recorded them.
///
/// # Errors
///
/// [`CardError::NotRecordable`] for a recorded parameter that is not a free
/// external of `model` at the recorded block and code.
pub fn bind_recorded(
    model: Arc<UFOModel>,
    params: &[MovedParam],
) -> Result<EvaluatedModel, CardError> {
    let mut card = ParamCard::default();
    for p in params {
        let free = matches!(
            external_at(&model, &p.block, &p.code),
            Some((name, _)) if name == p.name && !model.params.zeros.contains(name)
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
    Ok(EvaluatedModel::from_model_card(model, &card))
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
        let rebound = bind_recorded(sm(), &bound.moved).unwrap();
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

    #[test]
    fn an_auto_width_is_refused() {
        let err = bind("DECAY 25 Auto\n").err().unwrap();
        assert_eq!(err, CardError::AutoWidth { pdg: 25 });
        let text = err.to_string();
        assert!(
            text.contains("not supported yet") && text.contains("give the width"),
            "{text}"
        );
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

    #[test]
    fn an_entry_the_model_lacks_is_refused() {
        for text in [
            "Block NOSUCH\n 1 2.0\n",
            "Block MASS\n 99 2.0\n",
            "Block SMINPUTS\n 9 2.0\n",
            "DECAY 99 1.0\n",
        ] {
            assert!(
                matches!(bind(text), Err(CardError::Unknown { .. })),
                "{text:?}"
            );
        }
        // A new state's quantum numbers are not parameters.
        assert!(bind("Block QNUMBERS 9000005 # x\n 1 0\n 2 1\n").is_ok());
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
            .map(|e| (e.param.as_str(), e.block, e.pdg, e.card))
            .collect();
        assert_eq!(
            reported,
            vec![
                ("ZERO", "DECAY", 22, 1.0),
                ("ZERO", "MASS", 1, 0.005),
                ("MW", "MASS", 24, 80.0)
            ]
        );
        let mw = &bound.ignored[2];
        assert_eq!(mw.computed, defaults.param_values["MW"].re);
    }

    /// A model whose restriction gives two parameters of one block the same
    /// default, and a third the opposite one, merges them as MadGraph does;
    /// a card must keep the relation.
    #[test]
    fn splitting_a_merged_pair_is_refused() {
        let src = r"
a = Parameter(name = 'a', nature = 'external', type = 'real', value = 0.3, lhablock = 'NP', lhacode = [ 1 ])
b = Parameter(name = 'b', nature = 'external', type = 'real', value = 0.4, lhablock = 'NP', lhacode = [ 2 ])
c = Parameter(name = 'c', nature = 'external', type = 'real', value = 0.5, lhablock = 'NP', lhacode = [ 3 ])
d = Parameter(name = 'd', nature = 'external', type = 'real', value = 0.6, lhablock = 'OTHER', lhacode = [ 1 ])
w = Parameter(name = 'w', nature = 'external', type = 'real', value = 0.7, lhablock = 'DECAY', lhacode = [ 5 ])
v = Parameter(name = 'v', nature = 'external', type = 'real', value = 0.8, lhablock = 'DECAY', lhacode = [ 6 ])
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
        let check = |text: &str| check_merged(&params, &card(text));

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
                    bind_recorded(sm(), &moved),
                    Err(CardError::NotRecordable { .. })
                ),
                "{name}"
            );
        }
    }
}
