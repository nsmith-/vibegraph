//! `vibegraph enumerate` end to end on the interned Standard Model: the three
//! outputs, how they refer to one another, the refusal to overwrite, the
//! drawing selection, the flavour-group preview and the configurations.
//!
//! Needs only the interned SM and cards committed under `validation/`, so it
//! runs in the default suite.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Output;

use serde_json::Value;
use vibegraph::diagrams::artifact::DiagramArtifact;
use vibegraph::helas::eval::config_groups;

/// Six subprocesses, each standing for both beam orderings.
const CARD: &str = "define p = g u d u~ d~\ndefine j = p\ngenerate p p > e+ e- j\n";

struct Run {
    dir: tempfile::TempDir,
}

impl Run {
    fn new(card: &str) -> Self {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("proc_card.dat"), card).unwrap();
        Run { dir }
    }

    fn out(&self) -> PathBuf {
        self.dir.path().join("out")
    }

    fn enumerate(&self, extra: &[&str]) -> Output {
        common::vibegraph(self.dir.path(), self.dir.path())
            .arg("enumerate")
            .arg("proc_card.dat")
            .arg("--out")
            .arg(self.out())
            .args(extra)
            .output()
            .expect("spawn vibegraph")
    }

    fn ok(&self, extra: &[&str]) -> Output {
        let output = self.enumerate(extra);
        assert!(
            output.status.success(),
            "enumerate {extra:?} failed:\n{}",
            common::stderr_of(&output)
        );
        output
    }

    fn summary(&self) -> Value {
        let text = std::fs::read_to_string(self.out().join("diagrams.json")).unwrap();
        serde_json::from_str(&text).unwrap()
    }
}

/// Every file under `dir`, relative to it, with its contents.
fn tree(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for entry in entries {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let bytes = std::fs::read(&path).unwrap();
                out.insert(path.strip_prefix(dir).unwrap().to_path_buf(), bytes);
            }
        }
    }
    out
}

/// The SVG files under `<out>/svg`, by path relative to `<out>`.
fn drawings(out: &Path) -> BTreeSet<String> {
    tree(&out.join("svg"))
        .into_keys()
        .map(|p| format!("svg/{}", p.display()))
        .collect()
}

fn sets(summary: &Value) -> &Vec<Value> {
    summary["sets"].as_array().unwrap()
}

/// The three outputs are written and agree with one another: the artifact reads
/// back with the digest the summary records and the same sets in the same order;
/// every diagram id is `<set>.d<j>` in enumeration order; every drawing the
/// summary names exists at `svg/<set>/<id>.svg`, and no other drawing does.
///
/// Cannot detect a drawing whose content is wrong (the drawing itself is pinned
/// by the library's feyngraph oracle), nor an artifact whose sets are wrong in
/// the same way as the summary's, since both are read from one enumeration.
#[test]
fn enumerate_writes_the_artifact_the_summary_and_the_drawings() {
    let run = Run::new(CARD);
    let output = run.ok(&[]);
    let stdout = common::stdout_of(&output);
    let out = run.out();
    assert!(stdout.contains("process:  p p > e+ e- j"), "{stdout}");
    assert!(
        stdout.contains("total:    24 diagrams in 6 subprocesses"),
        "{stdout}"
    );

    let (artifact, _) = DiagramArtifact::read_from_path(&out.join("diagrams.bin.zst")).unwrap();
    let summary = run.summary();
    assert_eq!(summary["schema_version"], 1);
    assert_eq!(summary["process"], artifact.process);
    assert_eq!(summary["diagram_set_digest"], artifact.diagram_set_digest);
    assert_eq!(summary["model"]["label"], "sm-default");
    assert_eq!(summary["model"]["digest"], artifact.model.digest);
    assert!(summary.get("flavour_groups").is_none());

    let sets = sets(&summary);
    assert_eq!(sets.len(), artifact.sets.len());
    let mut named = BTreeSet::new();
    for (k, (set, enumerated)) in sets.iter().zip(&artifact.sets).enumerate() {
        assert_eq!(set["id"], format!("s{k}"));
        assert_eq!(set["label"], enumerated.label());
        assert_eq!(set["mirrored"], enumerated.mirrored);
        assert!(set.get("flavour_group").is_none());
        assert_eq!(set["diagrams"], enumerated.diagrams.len());
        let list = set["diagram_list"].as_array().unwrap();
        assert_eq!(list.len(), enumerated.diagrams.len());
        for (j, d) in list.iter().enumerate() {
            let id = format!("s{k}.d{j}");
            assert_eq!(d["id"], id);
            assert_eq!(d["svg"], format!("svg/s{k}/{id}.svg"));
            named.insert(d["svg"].as_str().unwrap().to_string());
            assert_eq!(
                d["propagators"].as_array().unwrap().len(),
                enumerated.diagrams[j].props.len()
            );
        }
        let histogram: u64 = set["coupling_orders"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| h["diagrams"].as_u64().unwrap())
            .sum();
        assert_eq!(histogram as usize, enumerated.diagrams.len());
    }
    assert_eq!(named.len(), 24);
    assert_eq!(drawings(&out), named);
    for path in &named {
        let svg = std::fs::read_to_string(out.join(path)).unwrap();
        assert!(svg.contains("<svg"), "{path}");
    }
}

/// A second run into the same directory is refused before the card is read, and
/// leaves every file as it was: a card that does not even exist is refused for
/// the existing outputs, not for itself. Each of the three outputs alone is
/// enough to refuse. `--force` then writes over them.
///
/// Cannot detect a refusal that happens after enumeration but before writing
/// for a card that does exist, except through the missing-card run.
#[test]
fn an_existing_output_is_refused_before_anything_is_read() {
    let run = Run::new(CARD);
    run.ok(&[]);
    let out = run.out();
    let before = tree(&out);

    for card in ["proc_card.dat", "missing_card.dat"] {
        let output = common::vibegraph(run.dir.path(), run.dir.path())
            .args(["enumerate", card, "--out"])
            .arg(&out)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{card}");
        let stderr = common::stderr_of(&output);
        assert!(
            stderr.contains("diagrams.bin.zst already exists"),
            "{stderr}"
        );
        assert!(stderr.contains("--force"), "{stderr}");
        assert!(!stderr.contains("missing_card"), "{stderr}");
        assert_eq!(tree(&out), before);
    }

    for kept in ["diagrams.bin.zst", "diagrams.json", "svg"] {
        let other = Run::new(CARD);
        std::fs::create_dir_all(other.out()).unwrap();
        let path = other.out().join(kept);
        if kept == "svg" {
            std::fs::create_dir(&path).unwrap();
        } else {
            std::fs::write(&path, "earlier").unwrap();
        }
        let output = other.enumerate(&[]);
        assert!(!output.status.success(), "{kept}");
        assert!(
            common::stderr_of(&output).contains(&format!("{kept} already exists")),
            "{kept}: {}",
            common::stderr_of(&output)
        );
    }

    run.ok(&["--force"]);
    assert_eq!(
        tree(&out).keys().collect::<Vec<_>>(),
        before.keys().collect::<Vec<_>>()
    );
}

/// `--svg none` draws nothing, writes no `svg/` and records every drawing as
/// `null`; `--svg first=N` draws the first `N` of each set; `--force` removes the
/// drawings an earlier run wrote and nothing else under `svg/`.
#[test]
fn the_drawing_selection_is_honoured_and_recorded() {
    let run = Run::new(CARD);
    run.ok(&["--svg", "none"]);
    assert!(!run.out().join("svg").exists());
    for set in sets(&run.summary()) {
        for d in set["diagram_list"].as_array().unwrap() {
            assert!(d["svg"].is_null());
        }
    }

    let run = Run::new(CARD);
    run.ok(&[]);
    let notes = run.out().join("svg/s0/notes.txt");
    std::fs::write(&notes, "kept").unwrap();
    run.ok(&["--force", "--svg", "first=1"]);
    let mut expected = BTreeSet::new();
    for (k, set) in sets(&run.summary()).iter().enumerate() {
        for (j, d) in set["diagram_list"].as_array().unwrap().iter().enumerate() {
            if j < 1 {
                assert_eq!(d["svg"], format!("svg/s{k}/s{k}.d{j}.svg"));
                expected.insert(d["svg"].as_str().unwrap().to_string());
            } else {
                assert!(d["svg"].is_null());
            }
        }
    }
    expected.insert("svg/s0/notes.txt".to_string());
    assert_eq!(drawings(&run.out()), expected);

    let refused = run.enumerate(&["--force", "--svg", "some"]);
    assert!(!refused.status.success());
}

/// Flavour groups are reported only when a card is given. With the Drell-Yan
/// run card (proton beams), every set belongs to exactly one group, each group
/// lists its members by set id with its representative first, and the cards
/// used are recorded. With fixed-energy beams nothing is grouped.
///
/// Cannot detect a grouping that differs from `integrate`'s while calling the
/// same function: that would be a change to `multiplicity_groups` itself.
#[test]
fn flavour_groups_are_a_preview_under_the_cards_given() {
    let run_card = common::validation_dir().join("dy13_default_run_card.dat");
    let run = Run::new("define p = g u c d s u~ c~ d~ s~\ngenerate p p > e+ e-\n");
    run.ok(&["--run-card", run_card.to_str().unwrap()]);
    let summary = run.summary();
    let preview = &summary["flavour_groups"];
    assert_eq!(preview["beams"], "proton");
    assert_eq!(preview["run_card"], run_card.display().to_string());
    assert!(preview["param_card"].is_null());
    let groups = preview["groups"].as_array().unwrap();
    assert!(!groups.is_empty());
    let mut member_of = BTreeMap::new();
    for g in groups {
        let members = g["members"].as_array().unwrap();
        assert_eq!(g["representative"], members[0]);
        for m in members {
            assert!(member_of
                .insert(m.as_str().unwrap().to_string(), g["id"].clone())
                .is_none());
        }
    }
    for set in sets(&summary) {
        assert_eq!(set["flavour_group"], member_of[set["id"].as_str().unwrap()]);
    }
    // `u u~` and `c c~` share |M|², masses and cuts, and so do `d d~` and `s s~`:
    // two groups of two.
    let labels = |g: &Value| -> Vec<String> {
        g["members"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| {
                let k: usize = m.as_str().unwrap()[1..].parse().unwrap();
                sets(&summary)[k]["label"].as_str().unwrap().to_string()
            })
            .collect()
    };
    let grouped: Vec<Vec<String>> = groups.iter().map(labels).collect();
    assert_eq!(
        grouped,
        [
            vec!["u u~ > e+ e-", "c c~ > e+ e-"],
            vec!["d d~ > e+ e-", "s s~ > e+ e-"]
        ],
        "{grouped:?}"
    );

    let param_card = common::validation_dir().join("dy13_param_card.dat");
    let run = Run::new("define p = g u c d s u~ c~ d~ s~\ngenerate p p > e+ e-\n");
    run.ok(&["--param-card", param_card.to_str().unwrap()]);
    let with_params = run.summary();
    assert!(with_params["flavour_groups"]["run_card"].is_null());
    assert_eq!(
        with_params["flavour_groups"]["param_card"],
        param_card.display().to_string()
    );
    assert_eq!(with_params["flavour_groups"]["groups"], preview["groups"]);

    let fixed = tempfile::tempdir().unwrap();
    let fixed_card = fixed.path().join("run_card.dat");
    std::fs::write(&fixed_card, common::fixed_beams(45.6, 45.6)).unwrap();
    let run = Run::new("generate e+ e- > mu+ mu-\n");
    run.ok(&["--run-card", fixed_card.to_str().unwrap()]);
    let summary = run.summary();
    assert_eq!(summary["flavour_groups"]["beams"], "fixed_energy");
    assert_eq!(summary["flavour_groups"]["groups"], Value::Array(vec![]));
    assert!(sets(&summary)[0]["flavour_group"].is_null());
}

/// The configurations of `e+ e- > e+ e-` are `config_groups`' partition, by
/// diagram id, and have MadGraph's shape: three AMP2 accumulators, the
/// t-channel photon and Z sharing one (`validation/madgraph/amplitudes/ee_to_ee.json`
/// banks `[[0], [1], [2, 3]]`). Every diagram is in a configuration, or listed
/// as a contact.
///
/// Compares against MadGraph through the group sizes and the propagators of
/// the shared group only, not index by index: the two enumerations number the
/// diagrams differently, and the index-level comparison is the amplitude
/// oracle's.
#[test]
fn configurations_are_madgraphs_partition() {
    let run = Run::new("generate e+ e- > e+ e-\n");
    run.ok(&["--svg", "none"]);
    let summary = run.summary();
    let set = &sets(&summary)[0];
    let (artifact, model) =
        DiagramArtifact::read_from_path(&run.out().join("diagrams.bin.zst")).unwrap();
    let expected: Vec<Vec<String>> = config_groups(&artifact.sets[0].diagrams, &model.model)
        .iter()
        .map(|g| g.iter().map(|j| format!("s0.d{j}")).collect())
        .collect();
    let configurations: Vec<Vec<String>> = set["configurations"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(c, conf)| {
            assert_eq!(conf["id"], format!("c{c}"));
            conf["diagrams"]
                .as_array()
                .unwrap()
                .iter()
                .map(|d| d.as_str().unwrap().to_string())
                .collect()
        })
        .collect();
    assert_eq!(configurations, expected);
    assert!(set["contact"].as_array().unwrap().is_empty());

    let reference: Value = serde_json::from_str(
        &std::fs::read_to_string(common::validation_dir().join("amplitudes/ee_to_ee.json"))
            .unwrap(),
    )
    .unwrap();
    let mut theirs: Vec<usize> = reference["amp2_groups"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g.as_array().unwrap().len())
        .collect();
    let mut ours: Vec<usize> = configurations.iter().map(Vec::len).collect();
    theirs.sort_unstable();
    ours.sort_unstable();
    assert_eq!(ours, theirs);

    let list = set["diagram_list"].as_array().unwrap();
    for (c, members) in configurations.iter().enumerate() {
        for id in members {
            let j: usize = id.strip_prefix("s0.d").unwrap().parse().unwrap();
            assert_eq!(list[j]["configuration"], format!("c{c}"));
        }
    }
    let shared = configurations.iter().find(|c| c.len() == 2).unwrap();
    let mut particles: Vec<String> = shared
        .iter()
        .map(|id| {
            let j: usize = id.strip_prefix("s0.d").unwrap().parse().unwrap();
            list[j]["propagators"][0]["particle"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();
    particles.sort();
    assert_eq!(particles, ["Z", "a"]);
}

/// A diagram with a four-point vertex is a contact: in no configuration, listed
/// under `contact`, its `configuration` `null`.
#[test]
fn contact_diagrams_are_listed_apart() {
    let run = Run::new("generate g g > g g\n");
    run.ok(&["--svg", "none"]);
    let summary = run.summary();
    let set = &sets(&summary)[0];
    let contact = set["contact"].as_array().unwrap();
    assert_eq!(contact.len(), 1, "{set}");
    let j: usize = contact[0]
        .as_str()
        .unwrap()
        .strip_prefix("s0.d")
        .unwrap()
        .parse()
        .unwrap();
    let d = &set["diagram_list"][j];
    assert!(d["configuration"].is_null());
    assert!(d["propagators"].as_array().unwrap().is_empty());
    assert_eq!(set["configurations"].as_array().unwrap().len(), 3);
}

/// Under `--output-ldjson` the result object carries the paths written and the
/// summary, identical to `diagrams.json`.
#[test]
fn the_ldjson_result_is_the_summary() {
    let run = Run::new(CARD);
    let output = run.ok(&["--output-ldjson"]);
    let stdout = common::stdout_of(&output);
    let result: Value = stdout
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .find(|v| v["kind"] == "result")
        .unwrap_or_else(|| panic!("no result in {stdout}"));
    assert_eq!(result["summary"], run.summary());
    assert_eq!(result["drawings"], 24);
    assert_eq!(
        result["artifact"],
        run.out().join("diagrams.bin.zst").display().to_string()
    );
}
