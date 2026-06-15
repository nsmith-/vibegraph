#![allow(dead_code)]

use std::sync::OnceLock;
use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, DiagramSet, ParsingOptions};
use vibegraph::ufo::UFOModel;

pub fn ufo_models_dir() -> std::path::PathBuf {
    let manifest = env!("CARGO_MANIFEST_DIR");
    std::path::Path::new(manifest).join("../research/refs/mg5amcnlo/models")
}

pub fn ufo_path(model: &str) -> std::path::PathBuf {
    ufo_models_dir().join(model)
}

static SM_MODEL: OnceLock<UFOModel> = OnceLock::new();

pub fn sm_model() -> &'static UFOModel {
    SM_MODEL.get_or_init(|| {
        UFOModel::load(&ufo_path("sm"), None)
            .expect("SM UFO not found — run: git submodule update --init --recursive")
    })
}

static SM_LEPTON_MASSES_MODEL: OnceLock<UFOModel> = OnceLock::new();

/// SM loaded with the `lepton_masses` restriction (`import model sm-lepton_masses`),
/// which keeps Me/MM/MTA non-zero — unlike `restrict_default`, which locks them to
/// zero. Use this when a test needs settable, physical lepton masses.
pub fn sm_lepton_masses_model() -> &'static UFOModel {
    SM_LEPTON_MASSES_MODEL.get_or_init(|| {
        let path = ufo_path("sm");
        let restrict = path.join("restrict_lepton_masses.dat");
        UFOModel::load(&path, Some(&restrict))
            .expect("SM UFO not found — run: git submodule update --init --recursive")
    })
}

pub fn generate(process: &str) -> Vec<DiagramSet> {
    generate_with(process, sm_model())
}

pub fn generate_with(process: &str, model: &UFOModel) -> Vec<DiagramSet> {
    let opts = ParsingOptions::default();
    let card = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
    generate_from_proc_card(&card, model).unwrap()
}
