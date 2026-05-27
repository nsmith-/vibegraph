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
use vibegraph::diagrams::{self, generate_from_proc_card, DiagramSet, ParsingOptions};
use vibegraph::ufo::UFOModel;

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct MgCluster {
    cluster: i32,
    legs: Vec<i32>,
    sprop: Vec<i32>,
    tprop: i32,
}

#[derive(Debug, serde::Deserialize)]
struct MgDiagram {
    diagram_id: u32,
    clusters: Vec<MgCluster>,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct DiagramData {
    process: String,
    total_diagrams: u32,
    diagrams_by_subprocess: HashMap<String, u32>,
    #[serde(default)]
    topologies_by_subprocess: HashMap<String, Vec<MgDiagram>>,
}

fn print_madgraph_topologies(data: &DiagramData) {
    eprintln!("=== MadGraph topologies: {} ===", data.process);
    let mut subprocesses: Vec<_> = data.topologies_by_subprocess.iter().collect();
    subprocesses.sort_by_key(|(name, _)| name.as_str());
    for (subprocess, diagrams) in subprocesses {
        if diagrams.is_empty() {
            continue;
        }
        eprintln!("  subprocess: {subprocess}");
        for diag in diagrams {
            let cluster_strs: Vec<String> = diag
                .clusters
                .iter()
                .map(|c| {
                    let legs_str = c
                        .legs
                        .iter()
                        .map(|l| {
                            if *l < 0 {
                                format!("cluster{l}")
                            } else {
                                format!("leg{l}")
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(",");
                    if c.tprop != 0 {
                        format!("[{legs_str}]→t-chan(pdg={})", c.tprop)
                    } else {
                        format!("[{legs_str}]→s-chan(pdg={:?})", c.sprop)
                    }
                })
                .collect();
            eprintln!(
                "    diagram {:2}: {}",
                diag.diagram_id,
                cluster_strs.join(", ")
            );
        }
    }
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

/// Default UFO search path for test harness. Skipped if not found.
fn ufo_search_path() -> Result<PathBuf, &'static str> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let ufo_path = Path::new(manifest_dir).join("../research/refs/mg5amcnlo/models");

    match ufo_path.exists() {
        true => Ok(ufo_path),
        false => Err(
            "UFO models directory not found. Please clone the madgraph submodule:
    git submodule update --init --recursive",
        ),
    }
}

/// Compute a topology fingerprint: for each diagram, collect and sort the
/// propagator PDG codes, then return the list of these sorted PDG lists.
fn compute_fingerprint(set: &DiagramSet) -> Vec<Vec<i32>> {
    let mut fingerprint = Vec::new();

    for diagram in set.diagrams.views() {
        let mut pdg_codes: Vec<i32> = diagram
            .propagators()
            .map(|prop| prop.particle().pdg() as i32)
            .collect();
        pdg_codes.sort();
        fingerprint.push(pdg_codes);
    }

    fingerprint.sort();
    fingerprint
}

/// Count unique topologies across diagram sets.
/// Two diagram sets are considered the same topology if they have identical
/// propagator PDG signatures. Returns the count of one representative per topology.
fn count_unique_topologies(sets: &[DiagramSet]) -> u32 {
    use std::collections::HashMap;

    let mut topology_groups: HashMap<Vec<Vec<i32>>, u32> = HashMap::new();

    for set in sets {
        let fingerprint = compute_fingerprint(set);
        // Count the first representative of each topology
        topology_groups
            .entry(fingerprint)
            .or_insert_with(|| set.diagrams.len() as u32);
    }

    topology_groups.values().sum()
}

/// Print the topology (propagator particles + momentum routing) for each diagram.
///
/// Momentum routing uses the convention from feyngraph: entry i is the coefficient of
/// the i-th external momentum (0-indexed: legs 1..n_in, then n_in+1..n_ext outgoing).
/// Outgoing leg momenta already have their sign flipped, so the vector reads as the
/// sum of incoming momenta flowing into the propagator.
fn print_diagram_topologies(process_str: &str, sets: &[DiagramSet]) {
    eprintln!("\n=== vibegraph topologies: {process_str} ===");
    let mut global_idx = 0usize;
    for set in sets {
        if set.diagrams.is_empty() {
            continue;
        }
        eprintln!(
            "  subprocess: {} > {}",
            set.particles_in.join(" "),
            set.particles_out.join(" ")
        );
        for view in set.diagrams.views() {
            global_idx += 1;
            let prop_strs: Vec<String> = view
                .propagators()
                .map(|p| {
                    let mom = p.momentum();
                    let mom_str = mom
                        .iter()
                        .enumerate()
                        .filter(|(_, &c)| c != 0)
                        .map(|(i, &c)| match c {
                            1 => format!("p{}", i + 1),
                            -1 => format!("-p{}", i + 1),
                            _ => format!("{c}*p{}", i + 1),
                        })
                        .collect::<Vec<_>>()
                        .join("+");
                    format!(
                        "{}(pdg={},mom={})",
                        p.particle().name(),
                        p.particle().pdg(),
                        mom_str
                    )
                })
                .collect();
            if prop_strs.is_empty() {
                eprintln!("    diagram {global_idx:2}: <no internal propagators>");
            } else {
                eprintln!("    diagram {global_idx:2}: {}", prop_strs.join(", "));
            }
        }
    }
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
    let script_content = fs::read_to_string(&script_path)
        .map_err(|e| Failed::from(format!("cannot read .mg5 script: {e}")))?;

    let opts = ParsingOptions::default();
    let card = diagrams::parse_proc_card(&script_content, &opts)
        .map_err(|e| Failed::from(format!("cannot parse .mg5 script: {e}")))?;

    // Extract first process from the card (should be 'generate' line)
    let process_str = card
        .processes
        .first()
        .ok_or("no 'generate' line in .mg5 script")?
        .to_string();

    let model = UFOModel::load(&ufo_search_path()?.join("sm"), None)?;
    let sets = generate_from_proc_card(&card, &model)
        .map_err(|e| Failed::from(format!("diagram generation failed: {e}")))?;
    let total_count: u32 = sets.iter().map(|s| s.diagrams.len() as u32).sum();
    let unique_topology_count = count_unique_topologies(&sets);

    print_madgraph_topologies(mg_data);
    print_diagram_topologies(&process_str, &sets);
    eprintln!(
        "  vibegraph: {total_count} total diagrams ({unique_topology_count} unique topologies)"
    );
    if unique_topology_count != mg_data.total_diagrams {
        return Err(format!(
            "vibegraph: {unique_topology_count} unique topologies, MG5 reference: {}",
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
