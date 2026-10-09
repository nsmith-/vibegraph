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
//! * [`poly`] proves, symbolically, which monomials in a set of parameters a
//!   subprocess's amplitude is a polynomial in.
//! * [`engine`] turns that into a per-event evaluation plan.
//!
//! # Two ways to evaluate a hypothesis
//!
//! The **exact path** binds the compiled amplitude to the hypothesis's own
//! parameter values once, before the first event, and evaluates `|M|²` once per
//! event per hypothesis.
//!
//! The **polynomial path** collects the amplitude by coupling monomial. When the
//! parameters `P` enter a subprocess only through its couplings, each of them a
//! polynomial in `P`, the amplitude is `A(P) = Σ_μ μ(P)·a_μ` over the monomials
//! [`poly`] proves, per helicity combination and colour flow — the amplitude of
//! each coupling class. Every hypothesis is then a quadratic form in the classes,
//! `|M(P)|² = Σ_μν μ(P) ν(P) Re Σ conj(a_μ)·CF·a_ν`, so an event costs one
//! amplitude evaluation per monomial and each hypothesis a `K × K` quadratic form,
//! whatever their number. An effective-field-theory scan in `n` coefficients with
//! at most one insertion per diagram has `K = 1 + n`: the `n(n+1)/2`-point basis
//! grid that pins the quadratic form costs `n` evaluations per event beyond the
//! card's own. The class amplitudes are read off `K` evaluations at well-chosen
//! parameter nodes, so a coupling that shifts an existing vertex (`a + b·c`) is
//! split correctly between classes rather than assigned to one by its diagram;
//! see [`engine`] for the construction.
//!
//! Without an explicit coupling set, hypotheses moving one and the same parameter
//! are grouped by it where that is cheaper than evaluating them. With one
//! ([`engine::ReweightOptions::couplings`]), every hypothesis is served jointly, and
//! a card moving anything else, or a coupling that is not polynomial in some
//! subprocess, is refused. Both paths give the same `|M|²` to within rounding, and
//! the tests hold them to it.

pub mod card;
pub mod engine;
pub(crate) mod poly;

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
    #[error(
        "launch `{launch}` makes `{name}` non-finite ({value}), which the card's own \
         parameters keep finite; no weight can be taken there"
    )]
    NonFinite {
        launch: String,
        name: String,
        value: String,
    },
    #[error("failed to compile a subprocess for reweighting: {0}")]
    Compile(String),
    #[error("`{name}` cannot be a reweighting coupling: {reason}")]
    BadCoupling { name: String, reason: String },
    #[error(
        "launch `{launch}` moves `{param}`, which is not among the couplings the \
         polynomial is tracked in"
    )]
    OutsideCouplings { launch: String, param: String },
    #[error(
        "{process}: its amplitude is not provably a polynomial in {couplings} (a coupling \
         reaches them through a non-polynomial function, or a mass or width moves with \
         them)"
    )]
    NotPolynomial { couplings: String, process: String },
    #[error("{process}: {reason}")]
    NodeSystem { process: String, reason: String },
}

/// Name each of `names` as an external parameter of `model` that a polynomial can
/// be tracked in: the same checks a card's `set` line passes, each name once.
pub fn resolve_couplings(model: &UFOModel, names: &[String]) -> Result<Vec<String>, ReweightError> {
    let mut out: Vec<String> = Vec::new();
    for name in names {
        let change = Change::Name {
            name: name.clone(),
            value: 0.0,
            line: 0,
        };
        let resolved = external_name(model, &change)
            .and_then(|n| check_movable(model, n, 0))
            .map_err(|e| ReweightError::BadCoupling {
                name: name.clone(),
                reason: match e {
                    ReweightError::UnknownParameter { .. } => "no such external parameter".into(),
                    ReweightError::InternalParameter { .. } => "an internal parameter".into(),
                    ReweightError::Locked { .. } => "fixed to zero by the restriction".into(),
                    ReweightError::StrongCoupling { .. } => "it moves the strong coupling".into(),
                    other => other.to_string(),
                },
            })?;
        if !out.contains(&resolved) {
            out.push(resolved);
        }
    }
    Ok(out)
}

/// Refuse a parameter no hypothesis may move.
///
/// The strong coupling is refused only where the parameter moves `aS` itself. A
/// parameter that merely shares a dependent with it — SMEFTsim's `dWH`, the Higgs
/// width's linear shift, is a function of `aS` and of most Wilson coefficients —
/// is movable: the per-event coupling update recomputes `aS`'s dependents from
/// each bound point's own parameters, the hypothesis's included.
fn check_movable(model: &UFOModel, name: String, line: usize) -> Result<String, ReweightError> {
    if model.params.zeros.contains(&name) {
        return Err(ReweightError::Locked { line, name });
    }
    if name == "aS" || model.params.dependents(&name).contains("aS") {
        return Err(ReweightError::StrongCoupling { line, name });
    }
    Ok(name)
}

/// Name every change of the card as an external parameter of `model`, and refuse
/// the changes reweighting cannot honour.
pub fn resolve(card: &ReweightCard, model: &UFOModel) -> Result<Vec<Launch>, ReweightError> {
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
            let name = check_movable(model, external_name(model, change)?, change.line())?;
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

    /// A Wilson coefficient that shares a dependent with `aS` (SMEFTsim's `dWH`)
    /// does not move the strong coupling and is movable.
    #[test]
    fn a_parameter_sharing_a_dependent_with_the_strong_coupling_is_movable() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../validation/ufo/SMEFTsim_topU3l_MwScheme_UFO");
        let model = UFOModel::load(&dir, Some(&dir.join("restrict_massless.dat"))).unwrap();
        let shared: Vec<String> = model
            .params
            .dependents("cHWB")
            .intersection(&model.params.dependents("aS"))
            .cloned()
            .collect();
        assert!(!shared.is_empty(), "the check needs a shared dependent");
        let got = resolve(&"launch\n set cHWB 0.5\n".parse().unwrap(), &model).unwrap();
        assert_eq!(got[0].values, vec![("cHWB".to_string(), 0.5)]);
        assert!(matches!(
            resolve(&"launch\n set aS 0.1\n".parse().unwrap(), &model),
            Err(ReweightError::StrongCoupling { .. })
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
