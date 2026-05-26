//! Validation: compare vibegraph diagram counts against MadGraph reference output.
//!
//! Each discovered process appears as a separate named test case via `libtest-mimic`.
//!
//! ## Prerequisites
//!
//! Run the build pipeline:
//!
//! ```sh
//! pixi run -e madgraph build-diagrams
//! pixi run -e madgraph extract-diagrams
//! ```
//!
//! This generates:
//! - Process directories under `validation/madgraph/output/`
//! - JSON file for each process: `output/DIR_NAME.json`
//!
//! ## Test Discovery
//!
//! Tests are generated dynamically via glob:
//! - Find all `*.json` files in `output/`
//! - For each JSON, infer the corresponding `.mg5` script
//! - Parse the script to extract the process string
//! - Generate diagrams with vibegraph
//! - Validate against MadGraph reference counts

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use libtest_mimic::{Arguments, Failed, Trial};
use vibegraph::diagrams::{self, ParsingOptions};
use vibegraph::ufo::UFOModel;

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct DiagramData {
    process: String,
    total_diagrams: u32,
    diagrams_by_subprocess: HashMap<String, u32>,
}

/// Find all MadGraph reference JSON files
fn find_madgraph_references() -> Vec<(PathBuf, DiagramData)> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output_dir = Path::new(manifest_dir).join("../validation/madgraph/output");

    if !output_dir.exists() {
        eprintln!(
            "MadGraph output directory not found: {}",
            output_dir.display()
        );
        eprintln!("Run: pixi run -e madgraph build-diagrams extract-diagrams");
        return Vec::new();
    }

    let mut results = Vec::new();

    if let Ok(entries) = fs::read_dir(&output_dir) {
        let mut json_files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .map(|e| e.path())
            .collect();

        json_files.sort();

        for json_path in json_files {
            if let Ok(content) = fs::read_to_string(&json_path) {
                if let Ok(data) = serde_json::from_str::<DiagramData>(&content) {
                    results.push((json_path, data));
                }
            }
        }
    }

    results
}

/// Extract process string from .mg5 script file
fn extract_process_from_mg5(script_path: &Path) -> Option<String> {
    let content = fs::read_to_string(script_path).ok()?;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("generate ") {
            return Some(trimmed[9..].trim().to_string());
        }

        if trimmed.starts_with("add process ") {
            return Some(trimmed[12..].trim().to_string());
        }
    }

    None
}

/// Load the SM UFO model from the submodule
fn load_sm_ufo() -> Option<UFOModel> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let ufo_path = Path::new(manifest_dir).join("../research/refs/mg5amcnlo/models/sm");

    if !ufo_path.exists() {
        eprintln!("SM UFO model not found: {}", ufo_path.display());
        return None;
    }

    UFOModel::load(&ufo_path).ok()
}

/// Generate diagrams and count them for a process
fn count_vibegraph_diagrams(process_str: &str, model: &UFOModel) -> Result<u32, String> {
    let opts = ParsingOptions::default();
    let spec = diagrams::parse_process_string(process_str, &opts)
        .map_err(|e| format!("Parse error: {e}"))?;

    let aliases = diagrams::AliasTable::default_sm();
    let sets = diagrams::generate_from_process_spec(&spec, model, &aliases)
        .map_err(|e| format!("Generation error: {e}"))?;

    let total: u32 = sets.iter().map(|s| s.diagrams.len() as u32).sum();

    Ok(total)
}

/// Infer the .mg5 script path from a JSON file path
fn infer_script_path(json_path: &Path) -> Option<PathBuf> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let file_stem = json_path.file_stem()?.to_string_lossy();
    let scripts_dir = Path::new(manifest_dir).join("../validation/madgraph/scripts");
    let script_path = scripts_dir.join(format!("{}.mg5", file_stem));

    if script_path.exists() {
        Some(script_path)
    } else {
        None
    }
}

fn run_trial(json_path: &Path, mg_data: &DiagramData) -> Result<(), Failed> {
    let script_path = infer_script_path(json_path).ok_or("no corresponding .mg5 script")?;
    let process =
        extract_process_from_mg5(&script_path).ok_or("no 'generate' line in .mg5 script")?;
    let model = load_sm_ufo().ok_or("SM UFO model not found")?;
    let count = count_vibegraph_diagrams(&process, &model).map_err(Failed::from)?;
    if count == 0 {
        return Err(format!(
            "vibegraph: 0 diagrams (MG5 reference: {})",
            mg_data.total_diagrams
        )
        .into());
    }
    Ok(())
}

fn make_trial(json_path: PathBuf, mg_data: DiagramData) -> Option<Trial> {
    let name = json_path.file_stem()?.to_string_lossy().into_owned();
    Some(Trial::test(name, move || run_trial(&json_path, &mg_data)))
}

fn main() {
    let args = Arguments::from_args();

    // Each trial runs sequentially; rayon parallelism only adds lock overhead here.
    rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build_global()
        .unwrap();

    let references = find_madgraph_references();
    if references.is_empty() {
        eprintln!("No MadGraph reference files found");
        eprintln!("Run: pixi run -e madgraph build-diagrams extract-diagrams");
        libtest_mimic::run(&args, vec![]).exit();
    }

    let trials: Vec<Trial> = references
        .into_iter()
        .filter_map(|(path, data)| make_trial(path, data))
        .collect();

    libtest_mimic::run(&args, trials).exit();
}
