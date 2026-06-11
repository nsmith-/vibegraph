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

pub fn generate(process: &str) -> Vec<DiagramSet> {
    let opts = ParsingOptions::default();
    let card = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
    generate_from_proc_card(&card, sm_model()).unwrap()
}
