//! Validation: compare vibegraph diagram counts against MadGraph reference output.
//!
//! Each discovered process appears as a separate named test case via `libtest-mimic`.
//!
//! This test is part of extended validation and only runs with the `extended-validation` feature:
//! ```sh
//! cargo test --test validate_madgraph_diagrams --features extended-validation
//! ```
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

mod common;

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
    println!("=== MadGraph topologies: {} ===", data.process);
    let mut subprocesses: Vec<_> = data.topologies_by_subprocess.iter().collect();
    subprocesses.sort_by_key(|(name, _)| name.as_str());
    for (subprocess, diagrams) in subprocesses {
        if diagrams.is_empty() {
            continue;
        }
        println!("  subprocess: {subprocess}");
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
            println!(
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

fn ufo_search_path() -> Result<PathBuf, &'static str> {
    let path = common::ufo_models_dir();
    match path.exists() {
        true => Ok(path),
        false => Err(
            "UFO models directory not found. Please clone the madgraph submodule:
    git submodule update --init --recursive",
        ),
    }
}

/// Map a PDG code to a coarse particle-type class.
///
/// MadGraph groups subprocesses by initial/final state particle type rather than specific
/// particle identity.  All light quarks and antiquarks (|pdg| 1–6) collapse to "quark";
/// leptons/antileptons (|pdg| 11–16) collapse to "lepton"; gluon stays "gluon".
/// This mirrors MadGraph's subprocess class naming convention (P1_gq_llq, P1_qq_llg, …).
fn particle_type_class(pdg: i64) -> &'static str {
    match pdg.unsigned_abs() {
        21 => "gluon",
        1..=6 => "quark",
        11..=16 => "lepton",
        22 => "photon",
        23..=25 => "weak_boson",
        _ => "other",
    }
}

/// Count diagrams using MadGraph-style subprocess class grouping.
///
/// Groups subprocess sets by (sorted initial particle type classes, sorted final particle
/// type classes), takes the diagram count of one representative per class, and sums.
/// All quarks/antiquarks of any flavor map to the same type class, reproducing MadGraph's
/// behaviour of collapsing flavor-equivalent subprocesses into one representative
/// (e.g. `g d > e+ e- d`, `g u > e+ e- u`, `g d~ > e+ e- d~` all belong to P1_gq_llq).
fn count_mg_style_topologies(sets: &[DiagramSet], model: &UFOModel) -> u32 {
    use std::collections::HashMap;

    // Build display-name → PDG lookup from the model's particle table.
    let mut name_to_pdg: HashMap<String, i64> = HashMap::new();
    for p in model.particles.values() {
        name_to_pdg.insert(p.name.clone(), p.pdg_code);
    }

    let classify = |name: &str| -> &'static str {
        let pdg = name_to_pdg.get(name).copied().unwrap_or(0);
        particle_type_class(pdg)
    };

    let mut groups: HashMap<(Vec<&'static str>, Vec<&'static str>), u32> = HashMap::new();

    for set in sets {
        if set.diagrams.is_empty() {
            continue;
        }
        let mut in_types: Vec<&'static str> =
            set.particles_in.iter().map(|n| classify(n)).collect();
        in_types.sort_unstable();
        let mut out_types: Vec<&'static str> =
            set.particles_out.iter().map(|n| classify(n)).collect();
        out_types.sort_unstable();

        groups
            .entry((in_types, out_types))
            .or_insert(set.diagrams.len() as u32);
    }

    groups.values().sum()
}

/// Print the topology (propagator particles + momentum routing) for each diagram.
///
/// Momentum routing uses the convention from feyngraph: entry i is the coefficient of
/// the i-th external momentum (0-indexed: legs 1..n_in, then n_in+1..n_ext outgoing).
/// Outgoing leg momenta already have their sign flipped, so the vector reads as the
/// sum of incoming momenta flowing into the propagator.
fn print_diagram_topologies(process_str: &str, sets: &[DiagramSet]) {
    println!("\n=== vibegraph topologies: {process_str} ===");
    let mut global_idx = 0usize;
    for set in sets {
        if set.diagrams.is_empty() {
            continue;
        }
        println!(
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
                println!("    diagram {global_idx:2}: <no internal propagators>");
            } else {
                println!("    diagram {global_idx:2}: {}", prop_strs.join(", "));
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
    let unique_topology_count = count_mg_style_topologies(&sets, &model);

    print_madgraph_topologies(mg_data);
    print_diagram_topologies(&process_str, &sets);
    println!(
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
