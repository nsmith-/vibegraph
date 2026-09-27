//! Event-by-event reweighting to alternative model parameters (MadGraph's
//! `reweight_card.dat`).
//!
//! Each `launch` block of a card is one alternative hypothesis: the generated
//! model with some external parameters moved. An event generated under the card's
//! own parameters is carried to the hypothesis by the ratio
//! `|M|²_new / |M|²_old` of its own subprocess at its own momenta and strong
//! coupling; the phase-space density, the parton densities and the scales are
//! unchanged, so everything else in the event weight cancels.
//!
//! The layers:
//!
//! * [`card`] reads the card as data.
//! * [`resolve`] names each change as an external parameter of the model and
//!   refuses what reweighting cannot honour.
//! * [`poly`] proves, symbolically, which powers of one parameter a subprocess's
//!   `|M|²` is a polynomial in.
//! * [`engine`] turns that into a per-event evaluation plan.
//!
//! # Two ways to evaluate a hypothesis
//!
//! The **exact path** binds the compiled amplitude to the hypothesis's own
//! parameter values once, before the first event, and evaluates `|M|²` once per
//! event per hypothesis.
//!
//! The **polynomial path** serves every hypothesis that moves the same single
//! parameter `P`, when `P` enters the process only through its couplings and each
//! of them is a polynomial in it: then `|M|²(P) = Σ_k q_k Pᵏ` for `k ≤ D`, with `D`
//! read off the diagrams. The terms of each power are collected per event by
//! evaluating `|M|²` at `D + 1` fixed nodes, after which any number of hypotheses
//! along `P` cost one `(D + 1)`-term dot product each. The nodes are the
//! Chebyshev–Lobatto points of the interval the hypotheses span, and the dot
//! product is barycentric Lagrange interpolation — the numerically stable way to
//! read a polynomial off its values — with the weights for each hypothesis
//! precomputed. An effective-field-theory coefficient entering linearly has
//! `D = 2`: three amplitude evaluations per event serve an arbitrarily fine scan.
//!
//! Which path a hypothesis takes is a cost decision per subprocess and per
//! parameter; both give the same `|M|²` to within rounding, and the tests hold
//! them to it.

pub mod card;
pub mod engine;
pub mod poly;

use std::collections::HashSet;

use crate::ufo::parameters::ParamNature;
use crate::ufo::UFOModel;

use self::card::{Change, ReweightCard};

/// One resolved hypothesis.
#[derive(Clone, Debug, PartialEq)]
pub struct Launch {
    /// The weight's id in the event file (`<wgt id='…'>`).
    pub id: String,
    /// Its description in the file's `<initrwgt>` block.
    pub info: String,
    /// The external parameters it moves and their new values, each parameter once,
    /// in the order the card first names them.
    pub values: Vec<(String, f64)>,
}

/// Why a reweight card cannot be applied to a run.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum ReweightError {
    #[error("{0}")]
    Card(#[from] card::CardError),
    #[error("line {line}: the model has no external parameter {what}")]
    UnknownParameter { line: usize, what: String },
    #[error(
        "line {line}: `{name}` is an internal parameter, computed from the external ones; \
         reweight in the external parameters it depends on"
    )]
    InternalParameter { line: usize, name: String },
    #[error(
        "line {line}: `{name}` is fixed to zero by the model's restriction, which removed \
         the vertices it feeds; generate with a restriction that keeps it non-zero"
    )]
    Locked { line: usize, name: String },
    #[error(
        "line {line}: `{name}` moves the strong coupling, which each event takes from the \
         run's scale choice; that is a scale variation, not a parameter reweight"
    )]
    StrongCoupling { line: usize, name: String },
    #[error("two launch blocks are both named `{0}`")]
    DuplicateName(String),
    #[error("`{0}` cannot name a weight: use letters, digits, `_`, `-` and `.`")]
    BadName(String),
    #[error(
        "launch `{launch}` moves `{param}`, the mass of the external {particle}: the \
         events' momenta are on the old mass shell"
    )]
    ExternalMass {
        launch: String,
        param: String,
        particle: String,
    },
    #[error(
        "the process forbids an s-channel on shell (`$`), whose amplitude depends on where \
         each event sits relative to the veto windows; it cannot be reweighted"
    )]
    ForbiddenSChannel,
    #[error("failed to compile a subprocess for reweighting: {0}")]
    Compile(String),
}

/// Name every change of the card as an external parameter of `model`, and refuse
/// the changes reweighting cannot honour.
pub fn resolve(card: &ReweightCard, model: &UFOModel) -> Result<Vec<Launch>, ReweightError> {
    let strong: HashSet<String> = {
        let mut s = model.params.dependents("aS");
        s.insert("aS".to_string());
        s
    };
    let mut launches: Vec<Launch> = Vec::with_capacity(card.launches.len());
    let mut ids: HashSet<String> = HashSet::new();
    for (k, spec) in card.launches.iter().enumerate() {
        let id = spec
            .name
            .clone()
            .unwrap_or_else(|| format!("rwgt_{}", k + 1));
        if id.is_empty()
            || !id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
        {
            return Err(ReweightError::BadName(id));
        }
        if !ids.insert(id.clone()) {
            return Err(ReweightError::DuplicateName(id));
        }
        let mut values: Vec<(String, f64)> = Vec::new();
        for change in &spec.changes {
            let name = external_name(model, change)?;
            let line = change.line();
            if model.params.zeros.contains(&name) {
                return Err(ReweightError::Locked { line, name });
            }
            if strong.contains(&name)
                || model
                    .params
                    .dependents(&name)
                    .iter()
                    .any(|d| strong.contains(d))
            {
                return Err(ReweightError::StrongCoupling { line, name });
            }
            match values.iter_mut().find(|(n, _)| *n == name) {
                Some(entry) => entry.1 = change.value(),
                None => values.push((name, change.value())),
            }
        }
        let info = spec.info.clone().unwrap_or_else(|| {
            if spec.changes.is_empty() {
                return "no change".to_string();
            }
            spec.changes
                .iter()
                .map(Change::to_string)
                .collect::<Vec<_>>()
                .join("; ")
        });
        launches.push(Launch { id, info, values });
    }
    Ok(launches)
}

/// The external parameter a change addresses.
fn external_name(model: &UFOModel, change: &Change) -> Result<String, ReweightError> {
    let params = &model.params;
    match change {
        Change::Lha {
            block, code, line, ..
        } => params
            .externals
            .iter()
            .find(|p| {
                matches!(&p.nature, ParamNature::External { lha_block, lha_code, .. }
                    if lha_block.eq_ignore_ascii_case(block) && lha_code == code)
            })
            .map(|p| p.name.clone())
            .ok_or_else(|| ReweightError::UnknownParameter {
                line: *line,
                what: format!(
                    "at block {block} code {}",
                    code.iter()
                        .map(i32::to_string)
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
            }),
        Change::Name { name, line, .. } => {
            if params.externals.iter().any(|p| p.name == *name) {
                return Ok(name.clone());
            }
            if params.internals.iter().any(|p| p.name == *name) {
                return Err(ReweightError::InternalParameter {
                    line: *line,
                    name: name.clone(),
                });
            }
            // MadGraph matches names case-insensitively; so does this, when the
            // match is unambiguous.
            let folded: Vec<&str> = params
                .externals
                .iter()
                .filter(|p| p.name.eq_ignore_ascii_case(name))
                .map(|p| p.name.as_str())
                .collect();
            match folded.as_slice() {
                [only] => Ok(only.to_string()),
                _ => Err(ReweightError::UnknownParameter {
                    line: *line,
                    what: format!("named `{name}`"),
                }),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ufo::sm::{sm_model, SMRestrict};

    fn launches(text: &str) -> Result<Vec<Launch>, ReweightError> {
        let model = sm_model(SMRestrict::Default);
        resolve(&text.parse()?, &model)
    }

    #[test]
    fn lha_addresses_and_names_resolve_to_the_same_parameter() {
        let got = launches(
            "launch\n set yukawa 6 150\nlaunch --rwgt_name=b\n set YMT 160\n set ymt 170\n",
        )
        .unwrap();
        assert_eq!(got[0].id, "rwgt_1");
        assert_eq!(got[0].values, vec![("ymt".to_string(), 150.0)]);
        assert_eq!(got[0].info, "set param_card yukawa 6 150");
        // The last of two changes to one parameter wins.
        assert_eq!(got[1].id, "b");
        assert_eq!(got[1].values, vec![("ymt".to_string(), 170.0)]);
    }

    #[test]
    fn unusable_parameters_are_refused() {
        assert!(matches!(
            launches("launch\n set nosuch 1\n"),
            Err(ReweightError::UnknownParameter { line: 2, .. })
        ));
        assert!(matches!(
            launches("launch\n set yukawa 99 1\n"),
            Err(ReweightError::UnknownParameter { line: 2, .. })
        ));
        assert!(matches!(
            launches("launch\n set MW 80\n"),
            Err(ReweightError::InternalParameter { .. })
        ));
        assert!(matches!(
            launches("launch\n set aS 0.12\n"),
            Err(ReweightError::StrongCoupling { .. })
        ));
        assert!(matches!(
            launches("launch\n set sminputs 3 0.12\n"),
            Err(ReweightError::StrongCoupling { .. })
        ));
        // The default restriction zeroes the light-quark Yukawas.
        let model = sm_model(SMRestrict::Default);
        let locked = model
            .params
            .zeros
            .iter()
            .next()
            .expect("a locked parameter");
        assert!(matches!(
            resolve(
                &format!("launch\n set {locked} 1\n").parse().unwrap(),
                &model
            ),
            Err(ReweightError::Locked { .. })
        ));
    }

    #[test]
    fn weight_ids_are_unique_and_attribute_safe() {
        assert!(matches!(
            launches("launch --rwgt_name=a\nlaunch --rwgt_name=a\n"),
            Err(ReweightError::DuplicateName(_))
        ));
        assert!(matches!(
            launches("launch --rwgt_name=a'b\n"),
            Err(ReweightError::BadName(_))
        ));
    }
}
