//! Names to particles: a parsed card read against a model, as MadGraph reads it.
//!
//! Two consumers. Enumeration needs every leg as a list of model particle
//! names ([`leg_names`], [`forbidden_propagator_names`]). The parser oracle needs the
//! whole of MadGraph's `ProcessDefinition` — PDG codes, polarization states,
//! the coupling-order dictionaries MadGraph derives from the constraints as
//! written — so that a card parsed here can be compared field by field with
//! the same card parsed by MadGraph ([`resolve_card`]).
//!
//! Names match the model case-insensitively after an exact match fails, which
//! is what MadGraph does for every model whose particle names do not differ by
//! case alone (it lowercases the line; the models' names are lowercase).

use std::collections::BTreeMap;

use thiserror::Error;

use super::alias::AliasTable;
use super::parse::{CouplingOp, LegParticle, LegState, ProcCardAst, ProcessDefinition};
use crate::ufo::particles::Particle;
use crate::ufo::UFOModel;

/// A name or code in a card that the model does not resolve, or a
/// combination MadGraph refuses once the model is known.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ResolveError {
    #[error("no particle '{0}' in the model")]
    UnknownParticle(String),
    #[error("no particle with PDG code {0} in the model")]
    UnknownPdg(i64),
    #[error(
        "'{0}' is both a model particle and a repeat count followed by a particle name; \
         this generator does not guess which one is meant"
    )]
    AmbiguousRepeat(String),
    #[error("multiparticle label '{0}' is also the name of a model particle")]
    LabelIsParticle(String),
    #[error("'{0}' is defined through an or-multiparticle, which a define cannot contain")]
    NestedOrMultiparticle(String),
    #[error("a required s-channel names the same particle twice: '> A A >' is not '> A >'")]
    DuplicateRequired,
    #[error("coupling order '{name}' is not an order of this model (its orders: {known})")]
    UnknownOrder { name: String, known: String },
    #[error("at most one negative coupling-order constraint can be given")]
    NegativeOrders,
    #[error("amplitude constraints '==' and '>' are tree-level only")]
    ConstrainedOrdersBeyondTree,
    #[error("the model defines no loop perturbation of '{0}'")]
    NotALoopModel(String),
    #[error("polarization '{pol}' of '{leg}': {why}")]
    Polarization {
        leg: String,
        pol: String,
        why: &'static str,
    },
    /// MadGraph's `check_polarization`: a final-state particle appears both
    /// polarized and otherwise (or with overlapping polarizations), as in
    /// `p p > z{T} z`. MadGraph asks whether to continue and a batch run
    /// answers no.
    #[error(
        "'{0}' carries a polarization that overlaps another final-state leg of the same \
         particle; MadGraph calls this syntax ambiguous ('p p > z{{T}} z') and does not \
         guarantee its symmetry factor"
    )]
    AmbiguousPolarization(String),
    #[error("decay processes cannot carry {0}")]
    DecayConstraint(&'static str),
    #[error("processes with different numbers of initial particles cannot be combined")]
    MixedInitialStates,
}

// ── Model lookups ─────────────────────────────────────────────────────────────

/// The model particle a name denotes: exact first, then case-insensitive.
pub fn particle_by_name<'m>(model: &'m UFOModel, name: &str) -> Option<&'m Particle> {
    model.particles.get(name).or_else(|| {
        model
            .particles
            .values()
            .find(|p| p.name.eq_ignore_ascii_case(name))
    })
}

/// The model particle with a PDG code.
pub fn particle_by_pdg(model: &UFOModel, code: i64) -> Option<&Particle> {
    model.particles.values().find(|p| p.pdg_code == code)
}

/// A label member or restriction entry as a model particle: a name, or an
/// integer PDG code.
fn member_particle<'m>(model: &'m UFOModel, member: &str) -> Result<&'m Particle, ResolveError> {
    if let Some(p) = particle_by_name(model, member) {
        return Ok(p);
    }
    match member.parse::<i64>() {
        Ok(code) => particle_by_pdg(model, code).ok_or(ResolveError::UnknownPdg(code)),
        Err(_) => Err(ResolveError::UnknownParticle(member.to_owned())),
    }
}

/// The particles a leg may be, in the label's member order.
fn leg_particles<'m>(
    model: &'m UFOModel,
    particle: &LegParticle,
    token: &str,
    aliases: &AliasTable,
) -> Result<Vec<&'m Particle>, ResolveError> {
    match particle {
        LegParticle::Label(label) => {
            if particle_by_name(model, label).is_some() {
                return Err(ResolveError::LabelIsParticle(label.clone()));
            }
            aliases
                .expand_name(label)
                .iter()
                .map(|m| member_particle(model, m))
                .collect()
        }
        LegParticle::Pdg(code) => Ok(vec![
            particle_by_pdg(model, *code).ok_or(ResolveError::UnknownPdg(*code))?
        ]),
        LegParticle::Name(name) => {
            if token != name && particle_by_name(model, token).is_some() {
                return Err(ResolveError::AmbiguousRepeat(token.to_owned()));
            }
            Ok(vec![particle_by_name(model, name).ok_or_else(|| {
                ResolveError::UnknownParticle(name.clone())
            })?])
        }
    }
}

/// The model particle names a leg may be, in the label's member order.
pub fn leg_names(
    model: &UFOModel,
    particle: &LegParticle,
    token: &str,
    aliases: &AliasTable,
) -> Result<Vec<String>, ResolveError> {
    Ok(leg_particles(model, particle, token, aliases)?
        .into_iter()
        .map(|p| p.name.clone())
        .collect())
}

/// A `/` list as the model particle names no propagator may carry: labels
/// expanded, and each particle's antiparticle added, since MadGraph forbids a
/// propagator by |PDG code| — in either orientation.
pub fn forbidden_propagator_names(
    model: &UFOModel,
    names: &[String],
    aliases: &AliasTable,
) -> Result<Vec<String>, ResolveError> {
    let mut out: Vec<String> = Vec::new();
    for p in restriction_particles(model, names, aliases)? {
        for name in [&p.name, &p.antiname] {
            if !out.contains(name) {
                out.push(name.clone());
            }
        }
    }
    Ok(out)
}

/// `extract_particle_ids` for a restriction list: a particle name first, then a
/// label, then a PDG code.
fn restriction_particles<'m>(
    model: &'m UFOModel,
    names: &[String],
    aliases: &AliasTable,
) -> Result<Vec<&'m Particle>, ResolveError> {
    let mut out = Vec::new();
    for name in names {
        if let Some(p) = particle_by_name(model, name) {
            out.push(p);
        } else if aliases.is_label(name) {
            for m in aliases.expand_name(name) {
                out.push(member_particle(model, &m)?);
            }
        } else {
            out.push(member_particle(model, name)?);
        }
    }
    Ok(out)
}

/// A card's labels as they stand under `model`.
///
/// MadGraph rewrites `p` and `j` on every `import model`
/// (`add_default_multiparticles`, `madgraph_interface.py:6042`), starting with
/// the `import model sm` it runs on start-up: when the model's b quark
/// (PDG 5) is massless and the label lacks it, `b b~` is appended (the
/// five-flavour scheme); when it is massive and the label has it, it is
/// removed. The photon is removed as well, since no model here perturbs in
/// QED. A model without a PDG-5 particle leaves the labels alone. Only a
/// label as it stood at the import is rewritten: `define p = ...` after the
/// import is taken as written, while a label defined through `p` after it
/// sees the rewritten `p`.
pub fn model_aliases(aliases: &AliasTable, model: &UFOModel) -> AliasTable {
    aliases.replay(&mut |table| rewrite_default_multiparticles(table, model))
}

fn rewrite_default_multiparticles(table: &mut AliasTable, model: &UFOModel) {
    let Some(b) = particle_by_pdg(model, 5) else {
        return;
    };
    let massless = b.mass_param == "ZERO" || model.params.zeros.contains(&b.mass_param);
    let pdg = |member: &str| member_particle(model, member).ok().map(|p| p.pdg_code);
    for label in ["p", "j"] {
        let Some(members) = table.plain_members(label) else {
            continue;
        };
        let mut members = members.to_vec();
        let remove = |members: &mut Vec<String>, code: i64| {
            if let Some(i) = members.iter().position(|m| pdg(m) == Some(code)) {
                members.remove(i);
            }
        };
        if members.iter().any(|m| pdg(m) == Some(5)) {
            if !massless {
                remove(&mut members, 5);
                remove(&mut members, -5);
            }
        } else if massless {
            members.push(b.name.clone());
            members.push(b.antiname.clone());
        }
        remove(&mut members, 22);
        table.rewrite_plain(label, members);
    }
}

/// The required s-channels of a definition as MadGraph's or-list of and-lists
/// of PDG codes.
pub fn required_s_channel_ids(
    groups: &[Vec<String>],
    aliases: &AliasTable,
    model: &UFOModel,
) -> Result<Vec<Vec<i64>>, ResolveError> {
    required_ids(groups, aliases, model)
}

/// A `$$` list as PDG codes, as written: MadGraph compares them with the
/// oriented s-channel id, so `$$ t` does not forbid an s-channel `t~`.
pub fn forbidden_s_channel_ids(
    names: &[String],
    aliases: &AliasTable,
    model: &UFOModel,
) -> Result<Vec<i64>, ResolveError> {
    let mut out: Vec<i64> = Vec::new();
    for p in restriction_particles(model, names, aliases)? {
        if !out.contains(&p.pdg_code) {
            out.push(p.pdg_code);
        }
    }
    Ok(out)
}

/// The model's coupling-order names.
pub fn check_order_name(model: &UFOModel, name: &str) -> Result<(), ResolveError> {
    if model.order_hierarchy.contains_key(name) {
        return Ok(());
    }
    Err(ResolveError::UnknownOrder {
        name: name.to_owned(),
        known: model
            .order_hierarchy
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .join(", "),
    })
}

// ── MadGraph's ProcessDefinition ──────────────────────────────────────────────

/// One leg of a [`ResolvedProcess`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedLeg {
    /// The PDG codes the leg may be, in the label's member order.
    pub ids: Vec<i64>,
    /// `true` for the final state, as MadGraph stores it.
    pub state: bool,
    /// MadGraph's helicity codes (`-1`, `0`, `1`, `99` for `{A}`, ...).
    pub polarization: Vec<i64>,
    pub tagged: bool,
}

/// MadGraph's `ProcessDefinition`, field for field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedProcess {
    pub id: u32,
    pub legs: Vec<ResolvedLeg>,
    pub required_s_channels: Vec<Vec<i64>>,
    pub forbidden_particles: Vec<i64>,
    pub forbidden_s_channels: Vec<i64>,
    pub forbidden_onsh_s_channels: Vec<i64>,
    pub orders: BTreeMap<String, i64>,
    pub squared_orders: BTreeMap<String, i64>,
    pub sqorders_types: BTreeMap<String, String>,
    pub constrained_orders: BTreeMap<String, (i64, String)>,
    pub overall_orders: BTreeMap<String, i64>,
    pub perturbation_couplings: Vec<String>,
    pub nlo_mode: String,
    pub has_born: bool,
    pub decay_chains: Vec<ResolvedProcess>,
}

/// Every process of a card, as MadGraph holds it after running the card.
pub fn resolve_card(
    ast: &ProcCardAst,
    model: &UFOModel,
) -> Result<Vec<ResolvedProcess>, ResolveError> {
    let mut out: Vec<ResolvedProcess> = Vec::new();
    let mut n_initial: Option<usize> = None;
    for p in ast.processes() {
        let def = &p.line.definition;
        let aliases = model_aliases(&p.aliases, model);
        let resolved = resolve_definition(def, Some(p.id), &aliases, model)?;
        let legs: Vec<(bool, Vec<i64>, Vec<i64>)> = resolved
            .legs
            .iter()
            .map(|l| (l.state, l.ids.clone(), l.polarization.clone()))
            .collect();
        if !polarizations_unambiguous(&legs) {
            return Err(ResolveError::AmbiguousPolarization(p.line.text.clone()));
        }
        if def.decay_chains.is_empty() {
            check_negative_orders(&resolved)?;
        } else {
            check_decay_constraints(&resolved, true)?;
        }
        let n = def.initial().count();
        if n_initial.is_some_and(|m| m != n) {
            return Err(ResolveError::MixedInitialStates);
        }
        n_initial = Some(n);
        out.push(resolved);
    }
    Ok(out)
}

fn check_negative_orders(p: &ResolvedProcess) -> Result<(), ResolveError> {
    let negatives = p
        .orders
        .values()
        .chain(p.squared_orders.values())
        .filter(|v| **v < 0)
        .count();
    if negatives > 1 {
        return Err(ResolveError::NegativeOrders);
    }
    Ok(())
}

/// `do_add`'s checks on a decay-chain line.
fn check_decay_constraints(p: &ResolvedProcess, core: bool) -> Result<(), ResolveError> {
    if !core && !p.perturbation_couplings.is_empty() {
        return Err(ResolveError::DecayConstraint("perturbation orders"));
    }
    if !p.squared_orders.is_empty() {
        return Err(ResolveError::DecayConstraint("squared-order constraints"));
    }
    if p.orders.values().any(|v| *v < 0) {
        return Err(ResolveError::DecayConstraint("negative coupling orders"));
    }
    for d in &p.decay_chains {
        check_decay_constraints(d, false)?;
    }
    Ok(())
}

/// One definition, recursively. `id` is the process number of a top-level
/// definition; a decay without its own `@N` is number 0, as in MadGraph.
pub fn resolve_definition(
    def: &ProcessDefinition,
    id: Option<u32>,
    aliases: &AliasTable,
    model: &UFOModel,
) -> Result<ResolvedProcess, ResolveError> {
    let mut legs = Vec::new();
    for leg in &def.legs {
        let particles = leg_particles(model, &leg.particle, &leg.token, aliases)?;
        let polarization = match &leg.polarization {
            Some(pol) => polarization_codes(&leg.token, pol, &particles)?,
            None => Vec::new(),
        };
        legs.push(ResolvedLeg {
            ids: particles.iter().map(|p| p.pdg_code).collect(),
            state: leg.state == LegState::Final,
            polarization,
            tagged: leg.tagged,
        });
    }

    let (nlo_mode, has_born, perturbation) = loop_mode(def, model)?;
    let orders = mg_orders(def, model, !perturbation.is_empty(), &nlo_mode)?;

    let ids = |names: &[String]| -> Result<Vec<i64>, ResolveError> {
        let mut out: Vec<i64> = Vec::new();
        for p in restriction_particles(model, names, aliases)? {
            if !out.contains(&p.pdg_code) {
                out.push(p.pdg_code);
            }
        }
        Ok(out)
    };

    let decay_chains = def
        .decay_chains
        .iter()
        .map(|d| resolve_definition(d, None, aliases, model))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ResolvedProcess {
        id: def.tag.or(id).unwrap_or(0),
        legs,
        required_s_channels: required_ids(&def.required_s_channels, aliases, model)?,
        // `Process.set` stores forbidden particles by |PDG code|: a forbidden
        // propagator is forbidden in either orientation.
        forbidden_particles: ids(&def.forbidden_particles)?
            .into_iter()
            .map(i64::abs)
            .collect(),
        forbidden_s_channels: ids(&def.forbidden_s_channels)?,
        forbidden_onsh_s_channels: ids(&def.forbidden_onsh_s_channels)?,
        orders: orders.orders,
        squared_orders: orders.squared,
        sqorders_types: orders.squared_types,
        constrained_orders: orders.constrained,
        overall_orders: def.overall_orders.iter().cloned().collect(),
        perturbation_couplings: perturbation,
        nlo_mode,
        has_born,
        decay_chains,
    })
}

/// `extract_particle_ids(..., crash_on_duplication=True)` on the required
/// s-channels: each `|` alternative is the product over its names, a plain
/// label contributing all its members at once and an or-label one of its
/// alternatives.
fn required_ids(
    groups: &[Vec<String>],
    aliases: &AliasTable,
    model: &UFOModel,
) -> Result<Vec<Vec<i64>>, ResolveError> {
    let mut out = Vec::new();
    for group in groups {
        let mut partial: Vec<Vec<i64>> = vec![Vec::new()];
        for name in group {
            let choices: Vec<Vec<i64>> = if let Some(p) = particle_by_name(model, name) {
                vec![vec![p.pdg_code]]
            } else if let Some(alternatives) = aliases.or_groups(name) {
                alternatives
                    .iter()
                    .map(|alt| {
                        alt.iter()
                            .map(|m| {
                                if aliases.is_label(m) {
                                    return Err(ResolveError::NestedOrMultiparticle(name.clone()));
                                }
                                member_particle(model, m).map(|p| p.pdg_code)
                            })
                            .collect()
                    })
                    .collect::<Result<_, _>>()?
            } else if aliases.is_label(name) {
                vec![aliases
                    .expand_name(name)
                    .iter()
                    .map(|m| member_particle(model, m).map(|p| p.pdg_code))
                    .collect::<Result<_, _>>()?]
            } else {
                vec![vec![member_particle(model, name)?.pdg_code]]
            };
            partial = partial
                .iter()
                .flat_map(|head| {
                    choices.iter().map(move |c| {
                        let mut v = head.clone();
                        v.extend(c);
                        v
                    })
                })
                .collect();
        }
        out.extend(partial);
    }
    for list in &out {
        let mut seen = list.clone();
        seen.sort_unstable();
        seen.dedup();
        if seen.len() != list.len() {
            return Err(ResolveError::DuplicateRequired);
        }
    }
    Ok(out)
}

/// MadGraph's `ProcessDefinition.check_polarization`
/// (`base_objects.py:3869`), over `(final state, PDG codes, helicity codes)`
/// per leg: `false` when a final-state particle is reachable from two legs
/// whose polarizations are not either identical lists or disjoint, an
/// unpolarized leg counting as every helicity. Initial-state legs are not
/// looked at.
pub fn polarizations_unambiguous(legs: &[(bool, Vec<i64>, Vec<i64>)]) -> bool {
    let all: Vec<i64> = (-3..=3).collect();
    let mut seen: BTreeMap<i64, Vec<Vec<i64>>> = BTreeMap::new();
    for (state, ids, pol) in legs {
        if !state {
            continue;
        }
        for id in ids {
            match seen.get_mut(id) {
                None => {
                    let first = if pol.is_empty() {
                        all.clone()
                    } else {
                        pol.clone()
                    };
                    seen.insert(*id, vec![first]);
                }
                Some(lists) if pol.is_empty() => {
                    if *lists != [all.clone()] {
                        return false;
                    }
                }
                Some(lists) => {
                    if lists.contains(pol) {
                        continue;
                    }
                    if pol.iter().any(|h| lists.iter().any(|l| l.contains(h))) {
                        return false;
                    }
                    lists.push(pol.clone());
                }
            }
        }
    }
    true
}

/// A polarized leg's helicity codes, read against the particles the leg may
/// be.
pub fn leg_polarization_codes(
    model: &UFOModel,
    particle: &LegParticle,
    token: &str,
    pol: &str,
    aliases: &AliasTable,
) -> Result<Vec<i64>, ResolveError> {
    let particles = leg_particles(model, particle, token, aliases)?;
    polarization_codes(token, pol, &particles)
}

/// `extract_process`'s polarization reader, which needs the particle's spin
/// and mass.
fn polarization_codes(
    token: &str,
    pol: &str,
    particles: &[&Particle],
) -> Result<Vec<i64>, ResolveError> {
    let err = |why| ResolveError::Polarization {
        leg: token.to_owned(),
        pol: pol.to_owned(),
        why,
    };
    let mut spins: Vec<i32> = particles.iter().map(|p| p.spin).collect();
    spins.dedup();
    if spins.len() > 1 {
        return Err(err(
            "a multiparticle with several spins cannot be polarized",
        ));
    }
    let spin = spins[0];
    let vector = spin == 3;
    let chars: Vec<char> = pol.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        i += 1;
        match c {
            ',' => {}
            'T' | 't' if vector => out.extend([1, -1]),
            'T' | 't' => return Err(err("'T' is for spin-one particles")),
            'L' | 'l' => out.push(-1),
            'R' | 'r' => out.push(1),
            'A' | 'a' | 'G' | 'g' | 'H' | 'h' | 'Q' | 'q' | 'W' | 'w' | 'S' | 's' => {
                if !vector {
                    return Err(err("propagator codes are for spin-one particles"));
                }
                out.push(match c.to_ascii_uppercase() {
                    'A' => 99,
                    'G' => 4,
                    'H' => 5,
                    'Q' => 6,
                    'W' => 7,
                    _ => 9,
                });
            }
            '+' | '-' => {
                let sign = if c == '+' { 1 } else { -1 };
                match chars.get(i).and_then(|d| d.to_digit(10)) {
                    Some(d) => {
                        if d > 3 {
                            return Err(err("helicities run from -3 to 3"));
                        }
                        out.push(sign * d as i64);
                        i += 1;
                    }
                    None => out.push(sign),
                }
            }
            '0' if spin == 1 || spin == 2 => {
                return Err(err("'0' (longitudinal) is not a scalar or fermion state"))
            }
            d if d.is_ascii_digit() => {
                let d = d.to_digit(10).expect("an ASCII digit") as i64;
                if d > 3 {
                    return Err(err("helicities run from -3 to 3"));
                }
                out.push(d);
            }
            _ => return Err(err("not a polarization code")),
        }
    }
    Ok(out)
}

/// MadGraph's `NLO_mode`, `has_born` and perturbation orders for a `[...]`.
fn loop_mode(
    def: &ProcessDefinition,
    model: &UFOModel,
) -> Result<(String, bool, Vec<String>), ResolveError> {
    let Some(spec) = &def.loop_spec else {
        return Ok(("tree".to_owned(), true, Vec::new()));
    };
    let (mut mode, has_born) = match spec.option.as_deref() {
        None => ("all", true),
        Some("sqrvirt") => ("virt", false),
        Some("noborn") => ("noborn", false),
        Some(o) => (o, true),
    };
    let joined = spec.orders.join(" ");
    let mut orders = spec.orders.clone();
    if joined.eq_ignore_ascii_case("all") || joined.eq_ignore_ascii_case("loonly") {
        if joined.eq_ignore_ascii_case("loonly") {
            mode = "LOonly";
        }
        // The models this crate loads declare no loop perturbations.
        orders = Vec::new();
    }
    for o in &orders {
        if o != "WEIGHTED" {
            check_order_name(model, o)?;
        }
    }
    if mode == "tree" {
        orders.clear();
    }
    if !orders.is_empty() && mode != "real" && mode != "LOonly" {
        return Err(ResolveError::NotALoopModel(orders.join(" ")));
    }
    Ok((mode.to_owned(), has_born, orders))
}

struct MgOrders {
    orders: BTreeMap<String, i64>,
    squared: BTreeMap<String, i64>,
    squared_types: BTreeMap<String, String>,
    constrained: BTreeMap<String, (i64, String)>,
}

/// `extract_process`'s coupling-order dictionaries, built from the constraints
/// in the order MadGraph strips them (right to left), with its aliases (`EW`
/// for `QED`, `aEW` for twice `QED^2`, `aS` for twice `QCD^2`).
fn mg_orders(
    def: &ProcessDefinition,
    model: &UFOModel,
    perturbed: bool,
    nlo_mode: &str,
) -> Result<MgOrders, ResolveError> {
    let known = |n: &str| model.order_hierarchy.contains_key(n);
    let mut alias: BTreeMap<&str, (&str, bool)> = BTreeMap::new();
    if known("EW") {
        if !known("QED") {
            alias.insert("QED", ("EW", false));
            alias.insert("QED^2", ("EW^2", false));
        }
        if !known("aEW") {
            alias.insert("aEW", ("EW^2", true));
        }
    } else if known("QED") {
        alias.insert("EW", ("QED", false));
        alias.insert("EW^2", ("QED^2", false));
        if !known("aEW") {
            alias.insert("aEW", ("QED^2", true));
        }
    }
    if known("QCD") && !known("aS") {
        alias.insert("aS", ("QCD^2", true));
    }

    let mut m = MgOrders {
        orders: BTreeMap::new(),
        squared: BTreeMap::new(),
        squared_types: BTreeMap::new(),
        constrained: BTreeMap::new(),
    };
    let op_str = |op: CouplingOp| op.to_string();
    for c in def.orders.iter().rev() {
        let written = if c.squared {
            format!("{}^2", c.name)
        } else {
            c.name.clone()
        };
        let (name, value) = match alias.get(written.as_str()) {
            Some((to, doubled)) => (to.to_string(), if *doubled { 2 * c.value } else { c.value }),
            None => (written.clone(), c.value),
        };
        if let Some(base) = name.strip_suffix("^2") {
            if base != "WEIGHTED" {
                check_order_name(model, base)?;
            }
            let ty = match c.op {
                CouplingOp::Eq => "<=".to_owned(),
                op => op_str(op),
            };
            m.squared.insert(base.to_owned(), value);
            m.squared_types.insert(base.to_owned(), ty);
        } else {
            if name != "WEIGHTED" {
                check_order_name(model, &name)?;
            }
            // MadGraph validates the aliased name and then records the one written.
            let (name, value) = (c.name.clone(), c.value);
            match c.op {
                CouplingOp::Eq | CouplingOp::Le => {
                    m.orders.insert(name, value);
                }
                CouplingOp::ExactEq => {
                    m.constrained.insert(name.clone(), (value, "==".to_owned()));
                    if !def.chain_core && !m.squared.contains_key(&name) {
                        m.squared.insert(name.clone(), 2 * value);
                        m.squared_types.insert(name.clone(), "==".to_owned());
                    }
                    m.orders.insert(name, value);
                }
                CouplingOp::Gt => {
                    m.constrained.insert(name.clone(), (value, ">".to_owned()));
                    if !def.chain_core && !m.squared.contains_key(&name) {
                        m.squared.insert(name.clone(), 2 * value);
                        m.squared_types.insert(name, ">".to_owned());
                    }
                }
            }
        }
    }
    if !m.constrained.is_empty() && nlo_mode != "tree" {
        return Err(ResolveError::ConstrainedOrdersBeyondTree);
    }
    if m.orders.is_empty() && !m.squared.is_empty() && !perturbed {
        for (k, v) in &m.squared {
            let ty = &m.squared_types[k];
            m.orders
                .insert(k.clone(), if *v >= 0 && ty != ">" { *v } else { 99 });
        }
    }
    if m.squared.values().filter(|v| **v < 0).count() > 1 {
        return Err(ResolveError::NegativeOrders);
    }
    Ok(m)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::parse::parse_proc_card_ast;
    use crate::ufo::sm::{sm_model, SMRestrict};

    fn resolve(card: &str) -> Result<Vec<ResolvedProcess>, ResolveError> {
        let model = sm_model(SMRestrict::Default);
        resolve_card(&parse_proc_card_ast(card).unwrap(), &model)
    }

    #[test]
    fn pdg_codes_and_names_resolve_alike() {
        let a = resolve("generate 11 -11 > 21 21").unwrap();
        let b = resolve("generate e- e+ > g g").unwrap();
        assert_eq!(a[0].legs, b[0].legs);
        assert_eq!(a[0].legs[0].ids, [11]);
        assert_eq!(a[0].legs[2].ids, [21]);
    }

    #[test]
    fn required_s_channels_expand_or_of_and() {
        let r = resolve("define v = z | a\ngenerate e+ e- > v h > mu+ mu- h").unwrap();
        assert_eq!(r[0].required_s_channels, [vec![23, 25], vec![22, 25]]);
        let r = resolve("generate e+ e- > l+ > e+ e-").unwrap();
        assert_eq!(r[0].required_s_channels, [vec![-11, -13]]);
        assert!(resolve("generate e+ e- > z z > mu+ mu-").is_err());
    }

    #[test]
    fn exact_orders_imply_squared_orders_except_in_a_chain_core() {
        let r = resolve("generate e+ e- > mu+ mu- QED==2").unwrap();
        assert_eq!(r[0].squared_orders["QED"], 4);
        assert_eq!(r[0].constrained_orders["QED"], (2, "==".to_owned()));
    }

    #[test]
    fn unknown_names_and_codes_are_errors() {
        assert!(matches!(
            resolve("generate e+ e- > mu+ foo"),
            Err(ResolveError::UnknownParticle(_))
        ));
        assert!(matches!(
            resolve("generate e+ e- > 99 -99"),
            Err(ResolveError::UnknownPdg(99))
        ));
        assert!(matches!(
            resolve("generate e+ e- > mu+ mu- FOO=2"),
            Err(ResolveError::UnknownOrder { .. })
        ));
    }

    #[test]
    fn polarizations_read_as_madgraph_codes() {
        let r = resolve("generate e+ e- > w+{0} w-{T}").unwrap();
        assert_eq!(r[0].legs[2].polarization, [0]);
        assert_eq!(r[0].legs[3].polarization, [1, -1]);
        assert!(resolve("generate e+ e- > mu+{0} mu-").is_err());
    }
}
