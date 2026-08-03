#![allow(dead_code)]

pub mod leshouche;
pub mod manifest;
pub mod pdfset;
pub mod report;

use std::sync::Arc;

use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, DiagramSet, ParsingOptions};
use vibegraph::ufo::sm::{sm_model as interned_sm, SMRestrict};
use vibegraph::ufo::UFOModel;

pub fn ufo_models_dir() -> std::path::PathBuf {
    let manifest = env!("CARGO_MANIFEST_DIR");
    std::path::Path::new(manifest).join("../research/refs/mg5amcnlo/models")
}

pub fn ufo_path(model: &str) -> std::path::PathBuf {
    ufo_models_dir().join(model)
}

pub fn sm_model() -> Arc<UFOModel> {
    interned_sm(SMRestrict::Default)
}

/// SM loaded with the `lepton_masses` restriction (`import model sm-lepton_masses`),
/// which keeps Me/MM/MTA non-zero — unlike `restrict_default`, which locks them to
/// zero. Use this when a test needs settable, physical lepton masses.
pub fn sm_lepton_masses_model() -> Arc<UFOModel> {
    interned_sm(SMRestrict::LeptonMasses)
}

pub fn generate(process: &str) -> Vec<DiagramSet> {
    generate_with(process, sm_model().as_ref())
}

pub fn generate_with(process: &str, model: &UFOModel) -> Vec<DiagramSet> {
    let opts = ParsingOptions::default();
    let card = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
    generate_from_proc_card(&card, model).unwrap()
}
