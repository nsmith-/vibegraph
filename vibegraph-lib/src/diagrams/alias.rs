//! Multiparticle alias table and Cartesian-product expansion.
//!
//! Mirrors MadGraph5's `MultiProcess` alias-expansion logic: each aliased leg
//! independently expands to its member particles; the Cartesian product over all
//! legs yields the set of concrete processes that are submitted to feyngraph.

use std::collections::HashMap;

use itertools::Itertools;

use super::parse::{CouplingConstraint, MultiparticleDef, ParticleLeg, ProcessSpec};

/// A table mapping alias names to lists of concrete particle names.
///
/// Built from the default SM multiparticle aliases plus any `define` commands
/// from the proc_card.
#[derive(Debug, Clone)]
pub struct AliasTable(HashMap<String, Vec<String>>);

impl AliasTable {
    /// Default SM multiparticle aliases from `input/multiparticles_default.txt`.
    pub fn default_sm() -> Self {
        let mut map = HashMap::new();
        let proton = || {
            vec![
                "g".into(),
                "u".into(),
                "c".into(),
                "d".into(),
                "s".into(),
                "u~".into(),
                "c~".into(),
                "d~".into(),
                "s~".into(),
            ]
        };
        map.insert("p".into(), proton());
        map.insert("j".into(), proton());
        map.insert("l+".into(), vec!["e+".into(), "mu+".into()]);
        map.insert("l-".into(), vec!["e-".into(), "mu-".into()]);
        map.insert("vl".into(), vec!["ve".into(), "vm".into(), "vt".into()]);
        map.insert("vl~".into(), vec!["ve~".into(), "vm~".into(), "vt~".into()]);
        AliasTable(map)
    }

    /// Build from `default_sm()` plus a list of `define` commands (applied in order).
    pub fn from_defines(defines: &[MultiparticleDef]) -> Self {
        let mut table = Self::default_sm();

        for def in defines {
            // Expand each RHS particle through the *current* table (one level of recursion).
            let mut expanded: Vec<String> = def
                .particles
                .iter()
                .flat_map(|p| table.expand_name(p))
                .collect();

            // Apply the `/ except` subtraction.
            if !def.except.is_empty() {
                let excluded: Vec<String> = def
                    .except
                    .iter()
                    .flat_map(|p| table.expand_name(p))
                    .collect();
                expanded.retain(|p| !excluded.contains(p));
            }

            table.0.insert(def.alias.clone(), expanded);
        }

        table
    }

    /// Insert or overwrite an alias entry.
    pub fn insert(&mut self, alias: String, particles: Vec<String>) {
        self.0.insert(alias, particles);
    }

    /// Expand a single name: returns the alias members if known, or a
    /// single-element slice containing the name itself if it's a concrete particle.
    pub fn expand_name<'a>(&'a self, name: &'a str) -> Vec<String> {
        self.0
            .get(name)
            .cloned()
            .unwrap_or_else(|| vec![name.to_owned()])
    }
}

/// A fully concrete process: all leg names are model particle names (no aliases).
#[derive(Debug, Clone)]
pub struct ConcreteProcess {
    pub initial: Vec<String>,
    pub final_state: Vec<String>,
    pub forbidden_particles: Vec<String>,
    pub forbidden_s_channels: Vec<String>,
    pub forbidden_onsh_s_channels: Vec<String>,
    pub required_s_channels: Vec<String>,
    pub coupling_constraints: Vec<CouplingConstraint>,
}

/// Expand all aliased legs in `spec` using `table`, returning one
/// `ConcreteProcess` per concrete particle assignment.
///
/// Each leg independently expands to its alias members; the Cartesian product
/// of all per-leg expansions is computed with `itertools::multi_cartesian_product`.
/// For `p p > e+ e-` this yields up to 9 × 9 = 81 concrete processes.
pub fn expand_process<'a>(
    spec: &'a ProcessSpec,
    table: &'a AliasTable,
) -> impl Iterator<Item = ConcreteProcess> + use<'a> {
    // Build per-leg option-lists for initial and final state.
    let initial_options: Vec<Vec<String>> = spec
        .initial
        .iter()
        .map(|leg| expand_leg(leg, table))
        .collect();
    let final_options: Vec<Vec<String>> = spec
        .final_state
        .iter()
        .map(|leg| expand_leg(leg, table))
        .collect();

    // Cartesian product over all leg slots.
    let initial_combos: Vec<Vec<String>> = initial_options
        .into_iter()
        .multi_cartesian_product()
        .collect();
    let final_combos: Vec<Vec<String>> = final_options
        .into_iter()
        .multi_cartesian_product()
        .collect();

    // Expand restriction name lists.
    let forbidden_particles = expand_name_list(&spec.forbidden_particles, table);
    let forbidden_s_channels = expand_name_list(&spec.forbidden_s_channels, table);
    let forbidden_onsh_s_channels = expand_name_list(&spec.forbidden_onsh_s_channels, table);
    let required_s_channels = expand_name_list(&spec.required_s_channels, table);

    // Combine: one ConcreteProcess per (initial_combo × final_combo) pair.
    itertools::iproduct!(initial_combos, final_combos).map(move |(init, fin)| ConcreteProcess {
        initial: init,
        final_state: fin,
        forbidden_particles: forbidden_particles.clone(),
        forbidden_s_channels: forbidden_s_channels.clone(),
        forbidden_onsh_s_channels: forbidden_onsh_s_channels.clone(),
        required_s_channels: required_s_channels.clone(),
        coupling_constraints: spec.coupling_constraints.clone(),
    })
}

fn expand_leg(leg: &ParticleLeg, table: &AliasTable) -> Vec<String> {
    // `count` is always 1 here because parse_leg_list already flattened duplication.
    table.expand_name(&leg.name)
}

fn expand_name_list(names: &[String], table: &AliasTable) -> Vec<String> {
    names.iter().flat_map(|n| table.expand_name(n)).collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::parse::{parse_process_string, ParsingOptions};

    fn opts() -> ParsingOptions {
        ParsingOptions {
            allow_forbidden_s_channels: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_concrete_particle_no_expansion() {
        let spec = parse_process_string("e+ e- > mu+ mu-", &opts()).unwrap();
        let table = AliasTable::default_sm();
        let concrete = expand_process(&spec, &table).collect::<Vec<_>>();
        // No aliases → exactly 1 combination.
        assert_eq!(concrete.len(), 1);
        assert_eq!(concrete[0].initial, vec!["e+", "e-"]);
        assert_eq!(concrete[0].final_state, vec!["mu+", "mu-"]);
    }

    #[test]
    fn test_p_alias_expansion() {
        // p p > e+ e-: p expands to 9 particles → 9 × 9 = 81 combos.
        let spec = parse_process_string("p p > e+ e-", &opts()).unwrap();
        let table = AliasTable::default_sm();
        let concrete = expand_process(&spec, &table).collect::<Vec<_>>();
        assert_eq!(concrete.len(), 81);
        // Every initial particle should be a member of p.
        let p_members: Vec<_> = vec!["g", "u", "c", "d", "s", "u~", "c~", "d~", "s~"];
        for c in &concrete {
            assert!(p_members.contains(&c.initial[0].as_str()));
            assert!(p_members.contains(&c.initial[1].as_str()));
        }
    }

    #[test]
    fn test_define_override() {
        let defs = vec![crate::diagrams::parse::MultiparticleDef {
            alias: "myp".into(),
            particles: vec!["u".into(), "d".into()],
            except: vec![],
        }];
        let table = AliasTable::from_defines(&defs);
        let spec = parse_process_string("myp > e+ e-", &opts()).unwrap();
        let concrete = expand_process(&spec, &table).collect::<Vec<_>>();
        // myp expands to [u, d] → 2 combos.
        assert_eq!(concrete.len(), 2);
    }

    #[test]
    fn test_define_with_except() {
        let defs = vec![crate::diagrams::parse::MultiparticleDef {
            alias: "q".into(),
            particles: vec!["p".into()], // p = g u c d s u~ c~ d~ s~
            except: vec!["g".into()],
        }];
        let table = AliasTable::from_defines(&defs);
        // `q` should be p minus g → 8 particles.
        assert_eq!(table.expand_name("q").len(), 8);
        assert!(!table.expand_name("q").contains(&"g".to_owned()));
    }

    #[test]
    fn test_forbidden_particles_expanded() {
        let spec = parse_process_string("p p > e+ e- / p", &opts()).unwrap();
        let table = AliasTable::default_sm();
        let concrete = expand_process(&spec, &table).collect::<Vec<_>>();
        // Forbidden particles should be the 9 members of p.
        assert_eq!(concrete[0].forbidden_particles.len(), 9);
    }
}
