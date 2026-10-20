//! A model's particles answer to MadGraph's default names by PDG code, as
//! `import model` leaves them, and to the model's own names under
//! `import model X --modelname`.
//!
//! `vibegraph_toy_names_UFO` calls the muon `m-`/`m+` (as `loop_sm` does) and
//! the up quark `up`/`up~`. Each check runs the card the way the CLI does:
//! check the card, load the model its `import model` names, enumerate.

use std::path::PathBuf;
use std::sync::Arc;

use vibegraph::config::GlobalConfig;
use vibegraph::diagrams::parse_proc_card_ast;
use vibegraph::diagrams::resolve::{resolve_card, ResolveError};
use vibegraph::diagrams::{check_supported, generate_from_proc_card, SupportedCard};
use vibegraph::ufo::UFOModel;

const IMPORT: &str = "import model vibegraph_toy_names_UFO";
const IMPORT_KEEP: &str = "import model vibegraph_toy_names_UFO --modelname";

fn config() -> GlobalConfig {
    GlobalConfig {
        ufo_search_path: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../validation/ufo"),
        restrict_path_override: None,
        run_card_path: None,
    }
}

/// The checked card and the model its `import model` line loads.
fn load(card: &str) -> (SupportedCard, Arc<UFOModel>) {
    let checked = check_supported(&parse_proc_card_ast(card).unwrap()).unwrap();
    let (model, _) = config().load_ufo_with_identity(&checked.model).unwrap();
    (checked, model)
}

/// Each subprocess's label and diagram count.
fn generate(card: &str) -> Vec<(String, usize)> {
    let (checked, model) = load(card);
    generate_from_proc_card(&checked, &model)
        .unwrap()
        .iter()
        .map(|set| (set.label(), set.diagrams.len()))
        .collect()
}

fn resolve(card: &str) -> Result<Vec<Vec<i64>>, ResolveError> {
    let (_, model) = load(card);
    let resolved = resolve_card(&parse_proc_card_ast(card).unwrap(), &model)?;
    Ok(resolved[0].legs.iter().map(|l| l.ids.clone()).collect())
}

/// The default names resolve, in legs, labels and restriction lists alike, and
/// the enumerated subprocess still carries the model's own names.
#[test]
fn default_names_resolve() {
    assert_eq!(
        resolve(&format!("{IMPORT}\ngenerate u u~ > mu+ mu-")).unwrap(),
        [[2], [-2], [-13], [13]]
    );
    let expected = vec![("up up~ > m+ m-".to_owned(), 1)];
    assert_eq!(
        generate(&format!("{IMPORT}\ngenerate u u~ > mu+ mu-")),
        expected
    );
    assert_eq!(
        generate(&format!(
            "{IMPORT}\ndefine uu = u u~\ngenerate uu uu > mu+ mu- / mu- $ u\n"
        )),
        expected
    );
}

/// The model's own name for a renamed particle is refused, as in MadGraph, and
/// the refusal names the default that replaced it.
#[test]
fn renamed_model_names_are_refused() {
    for (process, name, default) in [
        ("up up~ > mu+ mu-", "up", "u"),
        ("u u~ > m+ m-", "m+", "mu+"),
        ("u u~ > mu+ mu- / m-", "m-", "mu-"),
    ] {
        assert_eq!(
            resolve(&format!("{IMPORT}\ngenerate {process}")),
            Err(ResolveError::RenamedParticle {
                name: name.into(),
                default: default.into(),
            }),
            "{process}"
        );
    }
}

/// `--modelname` switches the renaming off: the model's names resolve, the
/// defaults do not, and the subprocess and its PDG codes are the same.
#[test]
fn modelname_keeps_the_models_names() {
    assert_eq!(
        resolve(&format!("{IMPORT_KEEP}\ngenerate up up~ > m+ m-")).unwrap(),
        resolve(&format!("{IMPORT}\ngenerate u u~ > mu+ mu-")).unwrap()
    );
    assert_eq!(
        generate(&format!("{IMPORT_KEEP}\ngenerate up up~ > m+ m-")),
        [("up up~ > m+ m-".to_owned(), 1)]
    );
    assert_eq!(
        resolve(&format!("{IMPORT_KEEP}\ngenerate u u~ > mu+ mu-")),
        Err(ResolveError::UnknownParticle("u".into()))
    );
}
