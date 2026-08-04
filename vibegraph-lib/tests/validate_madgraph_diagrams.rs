//! vibegraph's diagram enumeration against MadGraph's own, one named test case
//! per process via `libtest-mimic`.
//!
//! The reference is the committed `validation/madgraph/diagrams.json`: MadGraph's
//! `NGRAPHS` for each P-class's representative subprocess, summed. Committing the
//! counts is what lets this run against a checkout that has never built a process
//! directory; regenerating them is
//!
//! ```sh
//! pixi run generate-references refs
//! ```
//!
//! Each row names a `.mg5` script under `scripts/`, whose `generate` line is
//! parsed and enumerated here, then counted the way MadGraph counts: one
//! representative per (initial type class, final type class) group, since
//! MadGraph collapses flavour-equivalent subprocesses into one.
//!
//! The work area's per-process files carry the configs.inc topologies as well;
//! when one is present beside its process directory this prints it next to
//! vibegraph's own routing, which is a debugging aid and asserts nothing.

mod common;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use common::report::{DiagramsRow, Stopwatch};
use libtest_mimic::{Arguments, Failed, Trial};
use vibegraph::diagrams::{self, generate_from_proc_card, DiagramSet, ParsingOptions};
use vibegraph::ufo::sm::{sm_model, SMRestrict};
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

/// One row of the committed count reference.
#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct DiagramCounts {
    total_diagrams: u32,
    diagrams_by_subprocess: HashMap<String, u32>,
}

#[derive(Debug, serde::Deserialize)]
struct DiagramReference {
    processes: HashMap<String, DiagramCounts>,
}

/// The work area's per-process file, which adds the configs.inc topologies to
/// the committed counts.
#[derive(Debug, serde::Deserialize)]
struct DiagramTopologies {
    #[serde(default)]
    topologies_by_subprocess: HashMap<String, Vec<MgDiagram>>,
}

/// Rows whose count difference is understood, reported and not enforced.
///
/// `g g > g g`: MadGraph writes the four-gluon contact term as three separate
/// graphs, one per colour structure (`VVVV1_0`, `VVVV3_0`, `VVVV4_0` into
/// `AMP(1..3)`), and vibegraph writes it as one diagram whose vertex carries all
/// three structures — so 3 exchange + 3 contact against 3 exchange + 1 contact.
/// The physics of the row is pinned at a finer level than a count: the per-flow
/// amplitude gate over the same process agrees with MadGraph to 1e-13, which no
/// difference in diagram *content* could survive.
const INFORMATIONAL_ROWS: &[(&str, &str)] = &[(
    "gg_to_gg",
    "MadGraph counts the four-gluon vertex once per colour structure",
)];

fn print_madgraph_topologies(key: &str, data: &DiagramTopologies) {
    println!("=== MadGraph topologies: {key} ===");
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

fn madgraph_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph")
}

/// The committed counts, in `.mg5` script-name order.
fn madgraph_reference() -> Vec<(String, DiagramCounts)> {
    let path = madgraph_dir().join("diagrams.json");
    let content =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let reference: DiagramReference = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("cannot parse {}: {e}", path.display()));
    let mut rows: Vec<_> = reference.processes.into_iter().collect();
    rows.sort_by(|(a, _), (b, _)| a.cmp(b));
    rows
}

/// The work area's topology file for a process, when the work area is present.
fn work_area_topologies(key: &str) -> Option<DiagramTopologies> {
    let path = madgraph_dir().join(format!("output/{key}.json"));
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
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
fn print_diagram_topologies(process_str: &str, sets: &[DiagramSet], model: &UFOModel) {
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
        for diagram in &set.diagrams {
            global_idx += 1;
            let prop_strs: Vec<String> = diagram
                .props
                .iter()
                .map(|p| {
                    let mom_str = p
                        .momentum
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
                    let particle = model.particle(p.particle);
                    format!(
                        "{}(pdg={},mom={})",
                        particle.name, particle.pdg_code, mom_str
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

fn run_trial(key: &str, mg_counts: &DiagramCounts) -> Result<(), Failed> {
    let clock = Stopwatch::start();
    let script_path = madgraph_dir().join(format!("scripts/{key}.mg5"));
    let script_content = fs::read_to_string(&script_path)
        .map_err(|e| Failed::from(format!("cannot read {}: {e}", script_path.display())))?;

    let opts = ParsingOptions::default();
    let card = diagrams::parse_proc_card(&script_content, &opts)
        .map_err(|e| Failed::from(format!("cannot parse .mg5 script: {e}")))?;

    // Extract first process from the card (should be 'generate' line)
    let process_str = card
        .processes
        .first()
        .ok_or("no 'generate' line in .mg5 script")?
        .to_string();

    let model = sm_model(SMRestrict::Default);
    let sets = generate_from_proc_card(&card, &model)
        .map_err(|e| Failed::from(format!("diagram generation failed: {e}")))?;
    let total_count: u32 = sets.iter().map(|s| s.diagrams.len() as u32).sum();
    let unique_topology_count = count_mg_style_topologies(&sets, &model);

    if let Some(topologies) = work_area_topologies(key) {
        print_madgraph_topologies(key, &topologies);
    }
    print_diagram_topologies(&process_str, &sets, &model);
    println!(
        "  vibegraph: {total_count} total diagrams ({unique_topology_count} unique topologies)"
    );
    let informational = INFORMATIONAL_ROWS.iter().find(|(row, _)| *row == key);
    let mut row = DiagramsRow::new(
        key,
        &process_str,
        if informational.is_some() {
            "info"
        } else {
            "gate"
        },
    );
    row.ours = unique_topology_count;
    row.theirs = mg_counts.total_diagrams;
    row.ours_all_subprocesses = total_count;
    row.note = informational.map(|(_, why)| (*why).to_string());

    if unique_topology_count != mg_counts.total_diagrams {
        let report = format!(
            "vibegraph: {unique_topology_count} unique topologies, MG5 reference: {}",
            mg_counts.total_diagrams
        );
        match informational {
            Some((_, why)) => println!("  informational: {report} — {why}"),
            None => {
                row.status = "fail";
                row.note = Some(report.clone());
                row.duration_s = Some(clock.seconds());
                row.write();
                return Err(report.into());
            }
        }
    }
    row.duration_s = Some(clock.seconds());
    row.write();
    Ok(())
}

/// The committed `diagrams.json`'s keys must be exactly the manifest's
/// `diagrams = hermetic` rows — no more (a stale count for a row the manifest
/// no longer gates), no fewer (a gated row the committed file silently lost).
/// Both files are committed, so this stays hermetic.
fn diagrams_json_covers_exactly_the_hermetic_rows() -> Result<(), Failed> {
    let committed: std::collections::BTreeSet<String> = madgraph_reference()
        .into_iter()
        .map(|(key, _)| key)
        .collect();
    let declared = common::manifest::hermetic_diagram_rows();

    let missing_from_committed: Vec<_> = declared.difference(&committed).collect();
    let extra_in_committed: Vec<_> = committed.difference(&declared).collect();
    if !missing_from_committed.is_empty() || !extra_in_committed.is_empty() {
        return Err(format!(
            "diagrams.json disagrees with the manifest's hermetic diagrams rows: \
             missing from diagrams.json {missing_from_committed:?}, \
             present in diagrams.json but not declared hermetic {extra_in_committed:?}"
        )
        .into());
    }
    Ok(())
}

fn main() {
    let args = Arguments::from_args();

    // Each trial runs sequentially; rayon parallelism only adds lock overhead here.
    rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build_global()
        .unwrap();

    let mut trials: Vec<Trial> = madgraph_reference()
        .into_iter()
        .map(|(key, counts)| Trial::test(key.clone(), move || run_trial(&key, &counts)))
        .collect();
    assert!(!trials.is_empty(), "the committed reference has no rows");
    trials.push(Trial::test(
        "diagrams_json_covers_exactly_the_hermetic_rows",
        diagrams_json_covers_exactly_the_hermetic_rows,
    ));

    libtest_mimic::run(&args, trials).exit();
}
