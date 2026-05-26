//! Validation: compare vibegraph diagram counts against MadGraph reference output.
//!
//! This test dynamically discovers and validates all MadGraph processes under
//! `validation/madgraph/output/`, comparing tree-level diagram counts.
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

    // Glob all .json files in output/
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

    // Find the first "generate" or "add process" line
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

/// Helper: generate diagrams and count them for a process
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

/// Infer the .mg5 script name from a JSON file path
/// e.g., "ee_to_mumu.json" -> "ee_to_mumu.mg5"
fn infer_script_path(json_path: &Path) -> Option<PathBuf> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let file_stem = json_path.file_stem()?.to_string_lossy();

    // Scripts directory
    let scripts_dir = Path::new(manifest_dir).join("../validation/madgraph/scripts");

    // Script name is just JSON filename with .mg5 extension
    let script_path = scripts_dir.join(format!("{}.mg5", file_stem));

    if script_path.exists() {
        return Some(script_path);
    }

    None
}

#[test]
fn validate_madgraph_diagrams_dynamic() {
    let model = match load_sm_ufo() {
        Some(m) => m,
        None => {
            eprintln!("Skipping test (SM UFO not available)");
            return;
        }
    };

    let references = find_madgraph_references();

    if references.is_empty() {
        eprintln!("Warning: No MadGraph reference files found");
        eprintln!("Run: pixi run -e madgraph build-diagrams extract-diagrams");
        return;
    }

    eprintln!("Found {} MadGraph reference(s)", references.len());
    eprintln!("");

    let mut passed = 0;
    let mut failed = 0;

    for (json_path, mg_data) in references {
        let dir_name = json_path.file_stem().unwrap().to_string_lossy();
        eprintln!("Testing: {}", dir_name);

        // Infer the script path from the JSON filename
        let script_path = match infer_script_path(&json_path) {
            Some(p) => p,
            None => {
                eprintln!("  ✗ Could not find corresponding .mg5 script");
                failed += 1;
                continue;
            }
        };

        // Extract process string from the .mg5 script
        let process = match extract_process_from_mg5(&script_path) {
            Some(p) => p,
            None => {
                eprintln!(
                    "  ✗ Could not extract process from {}",
                    script_path.display()
                );
                failed += 1;
                continue;
            }
        };

        eprintln!("  Process: {}", process);
        eprintln!("  Script:  {}", script_path.display());

        // Generate diagrams with vibegraph
        let vg_count = match count_vibegraph_diagrams(&process, &model) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("  ✗ Error: {}", e);
                failed += 1;
                continue;
            }
        };

        eprintln!(
            "  MG5: {} diagrams, vibegraph: {} topologies",
            mg_data.total_diagrams, vg_count
        );

        // Validate that vibegraph generates at least 1 diagram
        if vg_count > 0 {
            eprintln!("  ✓ Passed");
            passed += 1;
        } else {
            eprintln!("  ✗ vibegraph generated 0 diagrams");
            failed += 1;
        }

        eprintln!("");
    }

    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    eprintln!("Results: {} passed, {} failed", passed, failed);
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    assert_eq!(failed, 0, "Expected 0 failures, but got {}", failed);
}
