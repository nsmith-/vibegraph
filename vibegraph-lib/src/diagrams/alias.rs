//! Multiparticle labels: MadGraph's defaults plus the card's `define` commands.
//!
//! Mirrors MadGraph5's multiparticle table: a label names a list of particles
//! (`p`, `j`, `l+`), or, when defined with `|`, a list of alternative lists (an
//! or-multiparticle, which only a required s-channel may use). Labels are
//! matched case-insensitively, as MadGraph does for every model whose particle
//! names do not differ by case alone.

use std::collections::HashMap;

use super::parse::MultiparticleDef;

/// A table mapping labels to their members, names or PDG codes as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasTable {
    plain: HashMap<String, Vec<String>>,
    or: HashMap<String, Vec<Vec<String>>>,
}

impl AliasTable {
    /// Default SM multiparticle aliases from `input/multiparticles_default.txt`.
    pub fn default_sm() -> Self {
        let proton: Vec<String> = ["g", "u", "c", "d", "s", "u~", "c~", "d~", "s~"]
            .map(String::from)
            .to_vec();
        let mut plain = HashMap::new();
        plain.insert("p".into(), proton.clone());
        plain.insert("j".into(), proton);
        plain.insert("l+".into(), vec!["e+".into(), "mu+".into()]);
        plain.insert("l-".into(), vec!["e-".into(), "mu-".into()]);
        plain.insert("vl".into(), vec!["ve".into(), "vm".into(), "vt".into()]);
        plain.insert("vl~".into(), vec!["ve~".into(), "vm~".into(), "vt~".into()]);
        AliasTable {
            plain,
            or: HashMap::new(),
        }
    }

    /// Build from `default_sm()` plus a list of `define` commands (applied in order).
    pub fn from_defines(defines: &[MultiparticleDef]) -> Self {
        let mut table = Self::default_sm();
        for def in defines {
            table.apply(def);
        }
        table
    }

    /// Apply one `define`: its members are expanded through the labels defined
    /// so far, and the `/` exclusions removed. A later `define` of the same label
    /// replaces the earlier one.
    pub fn apply(&mut self, def: &MultiparticleDef) {
        let key = def.alias.to_lowercase();
        let expand = |names: &[String]| -> Vec<String> {
            names.iter().flat_map(|n| self.expand_name(n)).collect()
        };
        if def.is_or() {
            let groups = def.groups.iter().map(|g| expand(g)).collect();
            self.plain.remove(&key);
            self.or.insert(key, groups);
        } else {
            let mut members = expand(&def.groups[0]);
            if !def.except.is_empty() {
                let excluded = expand(&def.except);
                members.retain(|p| !excluded.contains(p));
            }
            self.or.remove(&key);
            self.plain.insert(key, members);
        }
    }

    /// Insert or overwrite a plain label.
    pub fn insert(&mut self, alias: String, particles: Vec<String>) {
        let key = alias.to_lowercase();
        self.or.remove(&key);
        self.plain.insert(key, particles);
    }

    /// Whether `name` is a label of either kind.
    pub fn is_label(&self, name: &str) -> bool {
        let key = name.to_lowercase();
        self.plain.contains_key(&key) || self.or.contains_key(&key)
    }

    /// Whether `name` is an or-multiparticle label.
    pub fn is_or_label(&self, name: &str) -> bool {
        self.or.contains_key(&name.to_lowercase())
    }

    /// The alternatives of an or-multiparticle label.
    pub fn or_groups(&self, name: &str) -> Option<&[Vec<String>]> {
        self.or.get(&name.to_lowercase()).map(Vec::as_slice)
    }

    /// Every label, plain and or.
    pub fn labels(&self) -> impl Iterator<Item = &str> {
        self.plain.keys().chain(self.or.keys()).map(String::as_str)
    }

    /// Expand a single name: the members of a plain label, or the name itself.
    pub fn expand_name(&self, name: &str) -> Vec<String> {
        self.plain
            .get(&name.to_lowercase())
            .cloned()
            .unwrap_or_else(|| vec![name.to_owned()])
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::parse::parse_define_line;

    #[test]
    fn define_expands_through_earlier_labels_and_subtracts() {
        let table = AliasTable::from_defines(&[parse_define_line("q = p / g").unwrap()]);
        let q = table.expand_name("q");
        assert_eq!(q.len(), 8);
        assert!(!q.contains(&"g".to_owned()));
    }

    #[test]
    fn labels_match_case_insensitively() {
        let table = AliasTable::from_defines(&[parse_define_line("MyP = u d").unwrap()]);
        assert_eq!(table.expand_name("myp"), ["u", "d"]);
        assert_eq!(table.expand_name("P").len(), 9);
    }

    #[test]
    fn or_labels_are_kept_apart() {
        let table = AliasTable::from_defines(&[parse_define_line("v = z | a").unwrap()]);
        assert!(table.is_or_label("v"));
        assert_eq!(
            table.or_groups("v").unwrap(),
            [vec!["z".to_owned()], vec!["a".to_owned()]]
        );
        let table = AliasTable::from_defines(&[
            parse_define_line("v = z | a").unwrap(),
            parse_define_line("v = z a").unwrap(),
        ]);
        assert!(!table.is_or_label("v"));
    }
}
