//! The names a model's particles answer to in a proc card.
//!
//! `import model X` in MadGraph renames every particle whose PDG code is listed
//! in `input/particles_name_default.txt` to the name listed there
//! (`Model.pass_particles_name_in_mg_default`, `madgraph/core/base_objects.py`),
//! unless the import carries `-modelname` or `--modelname`
//! (`madgraph_interface.py`, `check_import` and `do_import`). The renaming is
//! in place: afterwards the default name resolves and the model's own name, if
//! it differed, does not, so `m+` on `loop_sm` is refused and `mu+` accepted.
//!
//! Here the renaming builds a lookup table, [`ParticleNames`], from the names a
//! card may write to the model's particles. The particles keep the model's own
//! names, so every name the library prints downstream of resolution (process
//! labels, diagram dumps, error messages about enumerated subprocesses) is the
//! UFO's; only what a card may write follows MadGraph.

use std::collections::HashSet;
use std::sync::OnceLock;

use indexmap::IndexMap;
use thiserror::Error;

/// `input/particles_name_default.txt` of MadGraph5_aMC@NLO, byte for byte.
const DEFAULT_NAMES_FILE: &str = include_str!("mg_input/particles_name_default.txt");

/// Which names a model's particles answer to.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ParticleNaming {
    /// MadGraph's default names by PDG code, as `import model X` applies them.
    #[default]
    MadGraphDefaults,
    /// The model's own names, as `import model X --modelname` keeps them.
    Model,
}

/// A default name MadGraph cannot give because another particle already has it.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error(
    "particle '{holder}' (PDG {holder_pdg}) already has the name '{name}', MadGraph's \
     default name for PDG {pdg}; import the model with --modelname to keep the model's \
     own particle names"
)]
pub struct DefaultNameConflict {
    pub name: String,
    pub pdg: i64,
    pub holder: String,
    pub holder_pdg: i64,
}

/// MadGraph's default names in file order, as `Model.load_default_name` reads
/// them: a line whose first non-blank character is `#` is a comment, any other
/// line that does not split into exactly two fields is skipped, and the name is
/// lowercased. A repeated code keeps its first position and its last name.
pub(crate) fn madgraph_default_names() -> &'static IndexMap<i64, String> {
    static TABLE: OnceLock<IndexMap<i64, String>> = OnceLock::new();
    TABLE.get_or_init(|| parse_default_names(DEFAULT_NAMES_FILE))
}

fn parse_default_names(text: &str) -> IndexMap<i64, String> {
    let mut table = IndexMap::new();
    for line in text.lines() {
        let line = line.trim_start();
        if line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        let [code, name] = fields[..] else {
            continue;
        };
        let code: i64 = code
            .parse()
            .expect("particles_name_default.txt: a PDG code is an integer");
        table.insert(code, name.to_lowercase());
    }
    table
}

/// The names a card may write for a model's particles, each mapped to the
/// particle's own (UFO) name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticleNames {
    /// Accepted name → the particle's UFO name, in the model's particle order.
    accepted: IndexMap<String, String>,
    /// UFO name → the default name that replaced it, for each particle renamed.
    renamed: IndexMap<String, String>,
}

impl ParticleNames {
    /// The model's own names, unchanged.
    pub(crate) fn of_model<'a>(particles: impl IntoIterator<Item = (&'a str, i64)>) -> Self {
        ParticleNames {
            accepted: particles
                .into_iter()
                .map(|(name, _)| (name.to_owned(), name.to_owned()))
                .collect(),
            renamed: IndexMap::new(),
        }
    }

    /// The names `import model` leaves a model's particles with.
    ///
    /// `particles` lists every particle and antiparticle once, by UFO name and
    /// PDG code (an antiparticle carrying the negated code); `model_name` is the
    /// model directory's base name, which MadGraph uses as the model's name.
    ///
    /// The renaming replays `pass_particles_name_in_mg_default` step by step, in
    /// the table's order: a particle whose current name differs from its default
    /// takes the default, unless some particle's current name already is that
    /// default, which MadGraph refuses. Its MSSM rule follows: a model named
    /// `mssm` or `mssm-*` calls PDG 25 `h1`, without a check.
    ///
    /// MadGraph lowercases a model's names before renaming when they stay
    /// distinct lowercased (`import_ufo.py`, `case_sensitive`). That cannot
    /// change the outcome here: such a model has no particle whose exact name is
    /// another's lowercased one, so a name differing from its default by case
    /// alone (`Z` for `z`) takes it unopposed, and resolution ignores case.
    pub(crate) fn madgraph<'a>(
        particles: impl IntoIterator<Item = (&'a str, i64)>,
        model_name: &str,
    ) -> Result<Self, DefaultNameConflict> {
        let entries: Vec<(&str, i64)> = particles.into_iter().collect();
        let mut current: Vec<String> = entries.iter().map(|(n, _)| (*n).to_owned()).collect();

        for (&pdg, default) in madgraph_default_names() {
            let Some(i) = entries.iter().position(|&(_, code)| code == pdg) else {
                continue;
            };
            if current[i] == *default {
                continue;
            }
            if let Some(j) = current.iter().position(|n| n == default) {
                return Err(DefaultNameConflict {
                    name: default.clone(),
                    pdg,
                    holder: entries[j].0.to_owned(),
                    holder_pdg: entries[j].1,
                });
            }
            current[i] = default.clone();
        }
        if model_name == "mssm" || model_name.starts_with("mssm-") {
            if let Some(i) = entries.iter().position(|&(_, code)| code == 25) {
                current[i] = "h1".to_owned();
            }
        }

        let mut names = ParticleNames {
            accepted: IndexMap::new(),
            renamed: IndexMap::new(),
        };
        for (&(ufo, _), name) in entries.iter().zip(current) {
            if !name.eq_ignore_ascii_case(ufo) {
                names.renamed.insert(ufo.to_owned(), name.clone());
            }
            names.accepted.insert(name, ufo.to_owned());
        }
        Ok(names)
    }

    /// The names under `naming`.
    pub(crate) fn with_naming<'a>(
        particles: impl IntoIterator<Item = (&'a str, i64)>,
        naming: ParticleNaming,
        model_name: &str,
    ) -> Result<Self, DefaultNameConflict> {
        match naming {
            ParticleNaming::MadGraphDefaults => Self::madgraph(particles, model_name),
            ParticleNaming::Model => Ok(Self::of_model(particles)),
        }
    }

    /// The UFO name of the particle `name` denotes: an exact match first, then
    /// one ignoring ASCII case.
    pub fn resolve(&self, name: &str) -> Option<&str> {
        lookup(&self.accepted, name)
    }

    /// The name that replaced `ufo_name`, if the model's own name was renamed
    /// away (matched as [`resolve`](Self::resolve) matches).
    pub fn renamed_to(&self, ufo_name: &str) -> Option<&str> {
        lookup(&self.renamed, ufo_name)
    }

    /// Whether the table covers exactly the particles `particles` lists (by UFO
    /// name), as it does when built from them.
    pub(crate) fn is_built_from<'a>(
        &self,
        particles: impl IntoIterator<Item = (&'a str, i64)>,
    ) -> bool {
        let covered: HashSet<&str> = self.accepted.values().map(String::as_str).collect();
        let listed: HashSet<&str> = particles.into_iter().map(|(name, _)| name).collect();
        covered == listed
    }
}

/// `map[key]`, else the value of the first key equal to `key` ignoring ASCII case.
fn lookup<'m>(map: &'m IndexMap<String, String>, key: &str) -> Option<&'m str> {
    map.get(key)
        .or_else(|| {
            map.iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(key))
                .map(|(_, v)| v)
        })
        .map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ufo::identity::digest_bytes;
    use crate::ufo::sm::{sm_model, SMRestrict};

    /// The vendored table is MadGraph's file at the pinned commit, unchanged.
    #[test]
    fn vendored_table_is_the_pinned_file() {
        assert_eq!(
            digest_bytes(DEFAULT_NAMES_FILE.as_bytes()),
            "8b4f80eb8f7d31fb3a75ef9a718f937f1ec5044964230efd63859a3ad1719e64"
        );
    }

    /// What `load_default_name` reads out of it.
    #[test]
    fn parsed_table() {
        let table = madgraph_default_names();
        assert_eq!(table.len(), 84);
        let first: Vec<(i64, &str)> = table
            .iter()
            .take(4)
            .map(|(&c, n)| (c, n.as_str()))
            .collect();
        assert_eq!(first, [(1, "d"), (-1, "d~"), (2, "u"), (-2, "u~")]);
        for (code, name) in [
            (13, "mu-"),
            (-13, "mu+"),
            (22, "a"),
            (23, "z"),
            (-24, "w-"),
            (-1000037, "x2-"),
        ] {
            assert_eq!(table.get(&code).map(String::as_str), Some(name), "{code}");
        }
        assert!(!table.contains_key(&25), "PDG 25 has no default name");
        assert_eq!(table.last(), Some((&-1000037, &"x2-".to_owned())));
    }

    #[test]
    fn parse_rules() {
        let table =
            parse_default_names("  # comment\n 6 T\n7 t extra\n\n-5 b~\n5 x\n-6 T~\n5 bb\n");
        let entries: Vec<(i64, &str)> = table.iter().map(|(&c, n)| (c, n.as_str())).collect();
        assert_eq!(entries, [(6, "t"), (-5, "b~"), (5, "bb"), (-6, "t~")]);
    }

    const LOOP_SM_LIKE: &[(&str, i64)] = &[
        ("m-", 13),
        ("m+", -13),
        ("e-", 11),
        ("e+", -11),
        ("a", 22),
        ("Z", 23),
    ];

    #[test]
    fn default_names_replace_the_models() {
        let names = ParticleNames::madgraph(LOOP_SM_LIKE.iter().copied(), "loop_sm").unwrap();
        assert_eq!(names.resolve("mu+"), Some("m+"));
        assert_eq!(names.resolve("mu-"), Some("m-"));
        assert_eq!(names.resolve("m+"), None);
        assert_eq!(names.renamed_to("m+"), Some("mu+"));
        assert_eq!(names.resolve("e-"), Some("e-"));
        assert_eq!(names.renamed_to("e-"), None);
        assert_eq!(names.resolve("z"), Some("Z"));
        assert_eq!(names.resolve("Z"), Some("Z"));
        assert_eq!(names.renamed_to("Z"), None, "a case change is not a rename");
    }

    #[test]
    fn model_naming_keeps_the_models() {
        let names = ParticleNames::with_naming(
            LOOP_SM_LIKE.iter().copied(),
            ParticleNaming::Model,
            "loop_sm",
        )
        .unwrap();
        assert_eq!(names.resolve("m+"), Some("m+"));
        assert_eq!(names.resolve("mu+"), None);
        assert_eq!(names.renamed_to("m+"), None);
    }

    /// `check_name_free`: a default name another particle already holds is an
    /// import error, and the error names the holder.
    #[test]
    fn a_taken_default_name_is_refused() {
        let err =
            ParticleNames::madgraph([("m-", 13), ("m+", -13), ("mu+", 9000013)], "x").unwrap_err();
        assert_eq!(
            err,
            DefaultNameConflict {
                name: "mu+".into(),
                pdg: -13,
                holder: "mu+".into(),
                holder_pdg: 9000013,
            }
        );
    }

    /// The check runs against names as they stand mid-renaming: a name freed by
    /// an earlier rename can be taken by a later one.
    #[test]
    fn renaming_is_sequential() {
        let names = ParticleNames::madgraph([("u", 1), ("x", 2)], "x").unwrap();
        assert_eq!(names.resolve("d"), Some("u"));
        assert_eq!(names.resolve("u"), Some("x"));
        assert_eq!(names.renamed_to("x"), Some("u"));
    }

    /// A model particle whose name is the default of a code the model lacks
    /// keeps it.
    #[test]
    fn an_absent_code_frees_its_name() {
        let names = ParticleNames::madgraph([("h2", 9000035)], "x").unwrap();
        assert_eq!(names.resolve("h2"), Some("h2"));
    }

    #[test]
    fn mssm_calls_the_light_higgs_h1() {
        for model in ["mssm", "mssm-full"] {
            let names = ParticleNames::madgraph([("h01", 25)], model).unwrap();
            assert_eq!(names.resolve("h1"), Some("h01"), "{model}");
        }
        for model in ["MSSM_SLHA2", "mssm_slha2"] {
            let names = ParticleNames::madgraph([("h01", 25)], model).unwrap();
            assert_eq!(names.resolve("h01"), Some("h01"), "{model}");
            assert_eq!(names.resolve("h1"), None, "{model}");
        }
    }

    /// The Standard Model's names already are MadGraph's defaults up to case
    /// (`Z`, `W+`, `H`), which MadGraph folds and resolution ignores, so every
    /// name resolves as it did before the renaming, and `--modelname` changes
    /// nothing on it.
    #[test]
    fn the_sm_is_unchanged() {
        for restrict in SMRestrict::ALL {
            let sm = sm_model(restrict);
            let pairs = || sm.particles.values().map(|p| (p.name.as_str(), p.pdg_code));
            let renamed = ParticleNames::madgraph(pairs(), "sm").unwrap();
            let own = ParticleNames::of_model(pairs());
            assert!(renamed.renamed.is_empty(), "{restrict:?}");
            let candidates = sm
                .particles
                .keys()
                .chain(madgraph_default_names().values())
                .flat_map(|n| [n.clone(), n.to_lowercase(), n.to_uppercase()]);
            for name in candidates {
                assert_eq!(renamed.resolve(&name), own.resolve(&name), "{name}");
            }
            assert_eq!(sm.particle_names().resolve("Z"), Some("Z"), "{restrict:?}");
        }
    }

    /// The table matches a particle set exactly: a particle added, one removed or
    /// one replaced is seen.
    #[test]
    fn the_table_knows_which_particles_it_was_built_from() {
        let names = ParticleNames::madgraph(LOOP_SM_LIKE.iter().copied(), "loop_sm").unwrap();
        assert!(names.is_built_from(LOOP_SM_LIKE.iter().copied()));
        assert!(!names.is_built_from(LOOP_SM_LIKE[1..].iter().copied()));
        let extra = LOOP_SM_LIKE.iter().copied().chain([("h", 25)]);
        assert!(!names.is_built_from(extra));
        let swapped = LOOP_SM_LIKE.iter().copied().skip(1).chain([("zz", 13)]);
        assert!(
            !names.is_built_from(swapped),
            "same count, another particle"
        );
    }

    /// Reading the names of a model whose particles changed after the table was
    /// built panics in debug builds, and rebuilding the table clears it.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "call with_particle_naming")]
    fn a_stale_table_is_caught_at_the_lookup() {
        let mut model = (*sm_model(SMRestrict::ALL[0])).clone();
        let mut twin = model.particles["a"].clone();
        twin.name = "a2".to_owned();
        twin.antiname = "a2".to_owned();
        model.particles.insert("a2".to_owned(), twin);
        let _ = model.particle_names();
    }
}
