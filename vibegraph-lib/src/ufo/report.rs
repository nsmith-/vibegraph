//! A restricted model's external parameters as data: what each one is, what a
//! parameter card may set it to, whether a reweight hypothesis may move it, and,
//! given a process, which of them reach its diagrams. Also the default
//! `param_card.dat` the restriction implies.
//!
//! Every decision here is read from the model's own state — the restriction's
//! zero, one and merged sets, the binder's value checks ([`super::bind`]) and the
//! reweight classification ([`crate::reweight`]) — so a reader of the report
//! holds no copy of those rules.
//!
//! The record is meant to grow by added fields: a later field on
//! [`ParameterRecord`] (a computed width and how it was computed, say) does not
//! reshape the existing ones.

use std::collections::BTreeSet;
use std::sync::Arc;

use serde::Serialize;

use super::parameters::{ParamNature, ParameterSet};
use super::slha::code_text;
use super::{EvaluatedModel, UFOModel};
use crate::diagrams::DiagramSet;
use crate::reweight::{check_movable, poly::PolyAnalysis, resolve_couplings, ReweightError};

/// What the restriction made of a parameter.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ParameterClass {
    /// A parameter a card and a hypothesis may set.
    Free,
    /// Fixed to zero by the restriction, which also removed the vertices it feeds.
    Zero,
    /// Left at exactly one by the restriction, which fixes it there.
    One,
    /// Tied by the restriction to another parameter: the value is `factor` times
    /// `into`'s.
    Merged { into: String, factor: f64 },
    /// Computed from the external parameters; it has no value of its own to set.
    /// Listed only where a `MASS` or `DECAY` entry of a card addresses it.
    Internal,
}

/// A particle whose mass or width a parameter is.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ParticleRole {
    /// The particle's positive PDG code.
    pub pdg: i64,
    pub name: String,
    /// `mass` or `width`.
    pub role: &'static str,
    /// Whether the particle is an external leg of some diagram of the process:
    /// `null` when no process was given.
    pub external_in_process: Option<bool>,
}

/// A condition the card binder holds a value to.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Constraint {
    /// The value is a finite number.
    Finite,
    /// The value is not below `value` (a mass is not negative unless it is a
    /// Majorana fermion's).
    Minimum { value: f64 },
    /// The width is a number: `DECAY <pdg> Auto` is refused.
    NotAuto,
    /// The value is `value`: a restriction's zero or one.
    Fixed { value: f64 },
    /// The value is `factor` times `to`'s.
    TiedTo { to: String, factor: f64 },
    /// Other parameters are tied to this one, each by its factor; they move only
    /// together with it.
    LeadsGroup { members: Vec<GroupMember> },
}

/// A member of a merged group and its factor against the group's first.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GroupMember {
    pub name: String,
    pub factor: f64,
}

/// Whether a reweight hypothesis may move a parameter.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReweightVerdict {
    Movable,
    /// Fixed to zero by the restriction.
    Locked,
    /// Fixed to one by the restriction.
    FixedOne,
    /// Tied to others by the restriction; a hypothesis moves the whole group.
    Merged,
    /// Moves the strong coupling, which each event takes from the run's scale.
    StrongCoupling,
    /// Moves the mass of an external particle of the process. Needs a process.
    ExternalMass,
    /// Not an external parameter.
    NotApplicable,
}

/// Whether the process's amplitude is provably a polynomial in the parameter.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolynomialStatus {
    /// No process was given.
    NoProcess,
    /// The parameter reaches no coupling, mass or width of the process.
    Independent,
    Polynomial,
    NotPolynomial,
    NotApplicable,
}

/// How the parameter enters the process's amplitude as a polynomial.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PolynomialEntry {
    pub status: PolynomialStatus,
    /// The highest power of the parameter in any subprocess's amplitude;
    /// `null` unless `status` is `polynomial`.
    pub max_degree: Option<u8>,
}

/// The reweight view of a parameter.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ReweightEntry {
    pub verdict: ReweightVerdict,
    /// Whether the parameter may be named among a reweighting's couplings.
    pub usable_as_coupling: bool,
    pub polynomial: PolynomialEntry,
}

/// One parameter of a card: an external parameter, or the internal one a `MASS`
/// or `DECAY` entry addresses.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ParameterRecord {
    pub name: String,
    /// The card block, as the model spells it.
    pub block: String,
    pub code: Vec<i32>,
    /// The value the restricted model takes with no card: the restriction's,
    /// or, for an internal parameter, the one computed from them.
    pub default: f64,
    pub class: ParameterClass,
    /// The particles this parameter is the mass or width of.
    pub particles: Vec<ParticleRole>,
    pub constraints: Vec<Constraint>,
    pub reweight: ReweightEntry,
    /// How the parameter reaches the process's diagrams, from `couplings` (some
    /// vertex coupling depends on it) and `masses_widths` (the mass or width of
    /// a leg or propagator does); `null` when no process was given.
    pub enters: Option<Vec<&'static str>>,
}

/// The report: every parameter, and with a process the parameters entering its
/// couplings.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ParameterReport {
    pub parameters: Vec<ParameterRecord>,
    /// The external parameters, in model order, that enter at least one
    /// diagram's couplings; `null` when no process was given. A parameter the
    /// restriction fixed to zero is not listed: the vertices it fed were
    /// removed, and where a coupling survives it sees the zero.
    pub entering: Option<Vec<String>>,
}

fn particles_of(
    model: &UFOModel,
    param: &str,
    externals: Option<&BTreeSet<i64>>,
) -> Vec<ParticleRole> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    let mut particles: Vec<_> = model
        .particles
        .values()
        .filter(|p| p.pdg_code > 0 && !p.is_unphysical())
        .collect();
    particles.sort_by_key(|p| p.pdg_code);
    for p in particles {
        for (role, name) in [("mass", &p.mass_param), ("width", &p.width_param)] {
            if name == param && seen.insert((p.pdg_code, role)) {
                out.push(ParticleRole {
                    pdg: p.pdg_code,
                    name: p.name.clone(),
                    role,
                    external_in_process: externals.map(|e| e.contains(&p.pdg_code)),
                });
            }
        }
    }
    out
}

/// The parameter names a set of diagrams reads: the direct dependencies of
/// every vertex coupling, and the mass and width of every leg and propagator.
struct Reads {
    couplings: BTreeSet<String>,
    masses_widths: BTreeSet<String>,
    external_pdgs: BTreeSet<i64>,
}

fn reads(model: &UFOModel, sets: &[DiagramSet]) -> Reads {
    let mut r = Reads {
        couplings: BTreeSet::new(),
        masses_widths: BTreeSet::new(),
        external_pdgs: BTreeSet::new(),
    };
    for set in sets {
        for d in &set.diagrams {
            for leg in &d.legs {
                r.external_pdgs
                    .insert(model.particle(leg.particle).pdg_code.abs());
            }
            let particles = d
                .legs
                .iter()
                .map(|l| l.particle)
                .chain(d.props.iter().map(|p| p.particle));
            for pid in particles {
                let p = model.particle(pid);
                r.masses_widths.insert(p.mass_param.clone());
                r.masses_widths.insert(p.width_param.clone());
            }
            for vertex in &d.vertices {
                for &key in model.vertex_def(vertex.interaction).couplings.keys() {
                    let coupling = model.bound_coupling(vertex.interaction, key);
                    r.couplings
                        .extend(model.coupling_def(coupling).deps.iter().cloned());
                }
            }
        }
    }
    r
}

/// Whether the external parameter `name` is one of `read`, or one `read` is
/// computed from.
fn reaches(params: &ParameterSet, name: &str, read: &BTreeSet<String>) -> bool {
    if params.zeros.contains(name) {
        return false;
    }
    read.contains(name) || params.dependents(name).iter().any(|d| read.contains(d))
}

fn class_of(params: &ParameterSet, name: &str) -> ParameterClass {
    if params.zeros.contains(name) {
        return ParameterClass::Zero;
    }
    if params.ones.contains(name) {
        return ParameterClass::One;
    }
    for group in &params.merged {
        if group[0].0 != name {
            if let Some((_, factor)) = group.iter().find(|(n, _)| n == name) {
                return ParameterClass::Merged {
                    into: group[0].0.clone(),
                    factor: *factor,
                };
            }
        }
    }
    ParameterClass::Free
}

fn constraints_of(
    params: &ParameterSet,
    name: &str,
    class: &ParameterClass,
    particles: &[ParticleRole],
    model: &UFOModel,
) -> Vec<Constraint> {
    let mut out = vec![Constraint::Finite];
    let majorana = |pdg: i64| {
        model
            .particles
            .values()
            .any(|p| p.pdg_code == pdg && p.spin.abs() == 2 && p.is_self_conjugate)
    };
    if particles
        .iter()
        .any(|p| p.role == "mass" && !majorana(p.pdg))
    {
        out.push(Constraint::Minimum { value: 0.0 });
    }
    if particles.iter().any(|p| p.role == "width") {
        out.push(Constraint::NotAuto);
    }
    match class {
        ParameterClass::Zero => out.push(Constraint::Fixed { value: 0.0 }),
        ParameterClass::One => out.push(Constraint::Fixed { value: 1.0 }),
        ParameterClass::Merged { into, factor } => out.push(Constraint::TiedTo {
            to: into.clone(),
            factor: *factor,
        }),
        ParameterClass::Free | ParameterClass::Internal => {}
    }
    if let Some(group) = params.merged.iter().find(|g| g[0].0 == name) {
        out.push(Constraint::LeadsGroup {
            members: group[1..]
                .iter()
                .map(|(n, f)| GroupMember {
                    name: n.clone(),
                    factor: *f,
                })
                .collect(),
        });
    }
    out
}

fn verdict_of(
    model: &UFOModel,
    name: &str,
    class: &ParameterClass,
    externals: Option<&BTreeSet<i64>>,
) -> ReweightVerdict {
    match check_movable(model, name.to_string(), 0) {
        Err(ReweightError::Locked { .. }) => return ReweightVerdict::Locked,
        Err(ReweightError::StrongCoupling { .. }) => return ReweightVerdict::StrongCoupling,
        _ => {}
    }
    if matches!(class, ParameterClass::One) {
        return ReweightVerdict::FixedOne;
    }
    if let Some(externals) = externals {
        let mut moved = model.params.dependents(name);
        moved.insert(name.to_string());
        if model
            .particles
            .values()
            .any(|p| externals.contains(&p.pdg_code.abs()) && moved.contains(&p.mass_param))
        {
            return ReweightVerdict::ExternalMass;
        }
    }
    if matches!(class, ParameterClass::Merged { .. })
        || model.params.merged.iter().any(|g| g[0].0 == name)
    {
        return ReweightVerdict::Merged;
    }
    ReweightVerdict::Movable
}

fn polynomial_of(
    model: &UFOModel,
    name: &str,
    sets: Option<&[DiagramSet]>,
    enters: bool,
) -> PolynomialEntry {
    let (status, max_degree) = match sets {
        None => (PolynomialStatus::NoProcess, None),
        Some(_) if !enters => (PolynomialStatus::Independent, None),
        Some(sets) => {
            let mut analysis = PolyAnalysis::new(model, &[name]);
            let mut degree = 0u8;
            let mut ok = true;
            for set in sets {
                match analysis.amplitude_support(&set.diagrams) {
                    Some(support) => degree = degree.max(support.max_exponent(0)),
                    None => {
                        ok = false;
                        break;
                    }
                }
            }
            if ok {
                (PolynomialStatus::Polynomial, Some(degree))
            } else {
                (PolynomialStatus::NotPolynomial, None)
            }
        }
    };
    PolynomialEntry { status, max_degree }
}

/// The report for `model`, and with `sets` (the process's enumerated
/// subprocesses) the parameters that reach its diagrams.
///
/// Records come in the model's external order, then the internal masses and
/// widths a `MASS` or `DECAY` entry addresses, in PDG order.
pub fn describe_parameters(model: &Arc<UFOModel>, sets: Option<&[DiagramSet]>) -> ParameterReport {
    let params = &model.params;
    let read = sets.map(|s| reads(model, s));
    let externals = read.as_ref().map(|r| &r.external_pdgs);
    let mut records = Vec::new();
    let mut entering = Vec::new();

    for p in &params.externals {
        let ParamNature::External {
            default_value,
            lha_block,
            lha_code,
        } = &p.nature
        else {
            continue;
        };
        let class = class_of(params, &p.name);
        let particles = particles_of(model, &p.name, externals);
        let in_couplings = read
            .as_ref()
            .is_some_and(|r| reaches(params, &p.name, &r.couplings));
        let in_kinematics = read
            .as_ref()
            .is_some_and(|r| reaches(params, &p.name, &r.masses_widths));
        if in_couplings {
            entering.push(p.name.clone());
        }
        let enters = read.as_ref().map(|_| {
            let mut via = Vec::new();
            if in_couplings {
                via.push("couplings");
            }
            if in_kinematics {
                via.push("masses_widths");
            }
            via
        });
        let verdict = verdict_of(model, &p.name, &class, externals);
        records.push(ParameterRecord {
            name: p.name.clone(),
            block: lha_block.clone(),
            code: lha_code.clone(),
            default: *default_value,
            constraints: constraints_of(params, &p.name, &class, &particles, model),
            reweight: ReweightEntry {
                verdict,
                usable_as_coupling: resolve_couplings(model, std::slice::from_ref(&p.name)).is_ok(),
                polynomial: polynomial_of(model, &p.name, sets, in_couplings || in_kinematics),
            },
            class,
            particles,
            enters,
        });
    }

    let evaluated = EvaluatedModel::from_model(model.clone());
    let mut internal_seen: BTreeSet<String> = BTreeSet::new();
    let mut physical: Vec<_> = model
        .particles
        .values()
        .filter(|p| p.pdg_code > 0 && !p.is_unphysical())
        .collect();
    physical.sort_by_key(|p| p.pdg_code);
    for p in physical {
        for (block, name) in [("MASS", &p.mass_param), ("DECAY", &p.width_param)] {
            if name == "ZERO"
                || params.externals.iter().any(|e| &e.name == name)
                || !internal_seen.insert(format!("{block}:{name}"))
            {
                continue;
            }
            let particles = particles_of(model, name, externals);
            records.push(ParameterRecord {
                name: name.clone(),
                block: block.to_string(),
                code: vec![p.pdg_code as i32],
                default: evaluated.param_values.get(name).map_or(0.0, |v| v.re),
                class: ParameterClass::Internal,
                particles,
                constraints: vec![],
                reweight: ReweightEntry {
                    verdict: ReweightVerdict::NotApplicable,
                    usable_as_coupling: false,
                    polynomial: PolynomialEntry {
                        status: PolynomialStatus::NotApplicable,
                        max_degree: None,
                    },
                },
                enters: None,
            });
        }
    }

    ParameterReport {
        parameters: records,
        entering: read.map(|_| entering),
    }
}

/// The default `param_card.dat` of the restricted model: every external
/// parameter at its restricted default, in blocks in the model's order, then
/// `DECAY` lines for the widths, with the computed masses and widths a card
/// also carries. Binding it reproduces the model's own defaults.
pub fn default_param_card(model: &Arc<UFOModel>) -> String {
    let report = describe_parameters(model, None);
    let mut blocks: Vec<(&str, Vec<&ParameterRecord>)> = Vec::new();
    let mut decays: Vec<&ParameterRecord> = Vec::new();
    for r in &report.parameters {
        if r.block.eq_ignore_ascii_case("decay") {
            decays.push(r);
            continue;
        }
        match blocks
            .iter_mut()
            .find(|(b, _)| b.eq_ignore_ascii_case(&r.block))
        {
            Some((_, rows)) => rows.push(r),
            None => blocks.push((&r.block, vec![r])),
        }
    }
    let mut out = String::from("# param_card.dat of the restricted model, written by vibegraph\n");
    for (block, rows) in blocks {
        out.push_str(&format!("Block {block}\n"));
        for r in rows {
            out.push_str(&format!(
                "    {} {:e} # {}\n",
                code_text(&r.code),
                r.default,
                r.name
            ));
        }
    }
    for r in decays {
        out.push_str(&format!(
            "DECAY {} {:e} # {}\n",
            code_text(&r.code),
            r.default,
            r.name
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::reweight::resolve;
    use crate::ufo::bind::bind_card;
    use crate::ufo::slha::ParamCard;

    fn smeft() -> (std::path::PathBuf, Arc<UFOModel>) {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../validation/ufo/SMEFTsim_topU3l_MwScheme_UFO");
        let model = UFOModel::load(&dir, Some(&dir.join("restrict_massless.dat"))).unwrap();
        (dir, model)
    }

    fn sets(process: &str, model: &UFOModel) -> Vec<DiagramSet> {
        let card =
            parse_proc_card(&format!("generate {process}\n"), &ParsingOptions::default()).unwrap();
        generate_from_proc_card(&card, model).unwrap()
    }

    fn record<'a>(report: &'a ParameterReport, name: &str) -> &'a ParameterRecord {
        report
            .parameters
            .iter()
            .find(|r| r.name == name)
            .unwrap_or_else(|| panic!("no record for {name}"))
    }

    #[test]
    fn every_class_has_its_record() {
        let (_, model) = smeft();
        let report = describe_parameters(&model, None);
        let mc = record(&report, "MC");
        assert_eq!(mc.class, ParameterClass::Zero);
        assert_eq!(mc.default, 0.0);
        assert!(mc.constraints.contains(&Constraint::Fixed { value: 0.0 }));
        assert_eq!(mc.reweight.verdict, ReweightVerdict::Locked);
        assert_eq!(mc.particles[0].pdg, 4);
        assert_eq!(mc.particles[0].external_in_process, None);

        let mt = record(&report, "MT");
        assert_eq!(mt.class, ParameterClass::Free);
        assert_eq!(mt.default, 172.76);
        assert!(mt.constraints.contains(&Constraint::Minimum { value: 0.0 }));
        assert_eq!(mt.reweight.verdict, ReweightVerdict::Movable);

        let wt = record(&report, "WT");
        assert!(wt.constraints.contains(&Constraint::NotAuto));
        assert_eq!(wt.block, "DECAY");

        for (class, verdict) in [
            (
                std::mem::discriminant(&ParameterClass::One),
                ReweightVerdict::FixedOne,
            ),
            (
                std::mem::discriminant(&ParameterClass::Merged {
                    into: String::new(),
                    factor: 0.0,
                }),
                ReweightVerdict::Merged,
            ),
        ] {
            let found: Vec<_> = report
                .parameters
                .iter()
                .filter(|r| std::mem::discriminant(&r.class) == class)
                .collect();
            assert!(!found.is_empty(), "no parameter of class {class:?}");
            assert!(found.iter().all(|r| r.reweight.verdict == verdict));
        }
        for r in &report.parameters {
            if let ParameterClass::Merged { into, factor } = &r.class {
                assert!(r.constraints.contains(&Constraint::TiedTo {
                    to: into.clone(),
                    factor: *factor
                }));
                let lead = record(&report, into);
                assert_eq!(lead.class, ParameterClass::Free);
                assert!(lead
                    .constraints
                    .iter()
                    .any(|c| matches!(c, Constraint::LeadsGroup { .. })));
            }
        }
    }

    #[test]
    fn internal_masses_are_listed_with_their_computed_value() {
        let model = crate::ufo::sm::sm_model(crate::ufo::sm::SMRestrict::Default);
        let report = describe_parameters(&model, None);
        let mw = record(&report, "MW");
        assert_eq!(mw.class, ParameterClass::Internal);
        assert_eq!((mw.block.as_str(), mw.code.as_slice()), ("MASS", &[24][..]));
        assert!((mw.default - 80.4).abs() < 1.0, "{}", mw.default);
        assert_eq!(mw.reweight.verdict, ReweightVerdict::NotApplicable);
    }

    /// The default card agrees with the restriction card's value for every
    /// external parameter the card gives, and binding it moves nothing.
    #[test]
    fn default_card_matches_the_restriction_card() {
        let (dir, model) = smeft();
        let restriction = ParamCard::from_file(&dir.join("restrict_massless.dat")).unwrap();
        let card_text = default_param_card(&model);
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("param_card.dat");
        std::fs::write(&path, &card_text).unwrap();
        let card = ParamCard::from_file(&path).unwrap();
        let report = describe_parameters(&model, None);
        let mut compared = 0;
        for r in report
            .parameters
            .iter()
            .filter(|r| r.class != ParameterClass::Internal)
        {
            assert_eq!(card.get(&r.block, &r.code), Some(r.default), "{}", r.name);
            if let Some(v) = restriction.get(&r.block, &r.code) {
                let stands_for = crate::ufo::parameters::MADGRAPH_FREE_VALUES
                    .iter()
                    .find(|(spelling, _)| *spelling == v)
                    .map_or(v, |&(_, s)| s);
                assert_eq!(r.default, stands_for, "{} against the restriction", r.name);
                compared += 1;
            }
        }
        assert!(compared > 100, "only {compared} compared");
        let bound = bind_card(model.clone(), &card).unwrap();
        assert!(bound.moved.is_empty(), "{:?}", bound.moved);
        assert!(bound.ignored.is_empty(), "{:?}", bound.ignored);
        let base = EvaluatedModel::from_model(model.clone());
        for (name, v) in &base.param_values {
            assert_eq!(bound.evaluated.param_values[name], *v, "{name}");
        }
    }

    /// The verdict is the one `resolve` reaches for a move of the parameter
    /// alone to a value that is neither its default nor one.
    #[test]
    fn verdicts_agree_with_the_reweight_resolver() {
        let (_, model) = smeft();
        let report = describe_parameters(&model, None);
        for r in report
            .parameters
            .iter()
            .filter(|r| r.class != ParameterClass::Internal)
        {
            let outcome = resolve(
                &format!("launch\n set {} 2.7\n", r.name).parse().unwrap(),
                &model,
            );
            let expected = match outcome {
                Ok(_) => ReweightVerdict::Movable,
                Err(ReweightError::Locked { .. }) => ReweightVerdict::Locked,
                Err(ReweightError::FixedOne { .. }) => ReweightVerdict::FixedOne,
                Err(ReweightError::SplitsMerged { .. }) => ReweightVerdict::Merged,
                Err(ReweightError::StrongCoupling { .. }) => ReweightVerdict::StrongCoupling,
                Err(e) => panic!("{}: {e}", r.name),
            };
            assert_eq!(r.reweight.verdict, expected, "{}", r.name);
        }
    }

    #[test]
    fn the_entering_list_follows_the_couplings() {
        let (_, model) = smeft();
        let process = sets("e+ e- > t t~ NP<=1", &model);
        let report = describe_parameters(&model, Some(&process));
        let entering = report.entering.as_ref().unwrap();
        println!("{entering:?}");
        let has = |n: &str| entering.iter().any(|e| e == n);
        assert!(has("cHW") || has("cHWB") || has("cHl3"), "{entering:?}");
        assert!(!entering.iter().any(|n| n == "MC"));
        assert!(entering.iter().all(|n| record(&report, n)
            .enters
            .as_ref()
            .unwrap()
            .contains(&"couplings")));
    }
}
