//! `vibegraph inspect --ranking` end to end on the binary: an `enumerate` run,
//! an integration started from its diagram artifact, and the ranking paired
//! with the enumerate run's `diagrams.json`.
//!
//! What is pinned here:
//!
//! * configurations are ranked by decreasing σ_c, their shares and the
//!   unattributed weight sum to one, and every id the ranking names is one
//!   `diagrams.json` names, with the same configuration membership and the
//!   same drawings, each of which exists on disk;
//! * the numbers are the ones `inspect --output-ldjson` reports;
//! * a `diagrams.json` of another card is refused, naming both digests, and
//!   the ranking file is left as it was; an existing ranking is not
//!   overwritten without `--force`;
//! * at proton beams the flavour groups are the ones the integration banked,
//!   which are the ones `enumerate`'s preview forms under the same run card.
//!
//! What it cannot see: whether σ_c itself is right (the configuration-share
//! oracle against MadEvent covers that), and a card whose integrated
//! configurations are numbered differently from `config_groups` (a whole
//! configuration with no colour-referenced amplitude); none of the cards here
//! has one.
//!
//! Hermetic: the built-in SM, fixed-energy beams or the committed PDF set.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Output;

use common::{fixed_beams, stderr_of, PdfDataPath};
use serde_json::Value;

/// Five-gluon scattering: 25 diagrams in several configurations, with
/// four-gluon contact diagrams that carry none.
const GLUON_CARD: &str = "import model sm\ngenerate g g > g g g\n";
const OTHER_CARD: &str = "import model sm\ngenerate e+ e- > mu+ mu-\n";
/// Light flavours that group pairwise (`u` with `c`, `d` with `s`).
#[cfg(feature = "extended-validation")]
const PROTON_CARD: &str =
    "import model sm\ndefine p = g u c d s u~ c~ d~ s~\ndefine j = p\ngenerate p p > e+ e- j\n";

fn validation(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../validation")
        .join(name)
}

fn assert_ok(output: &Output, what: &str) {
    assert!(
        output.status.success(),
        "{what} failed:\n{}",
        stderr_of(output)
    );
}

struct Run {
    dir: tempfile::TempDir,
}

impl Run {
    fn new() -> Self {
        Run {
            dir: tempfile::tempdir().unwrap(),
        }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.path().join(name)
    }

    fn vibegraph(&self) -> std::process::Command {
        let mut cmd = common::vibegraph(self.dir.path(), self.dir.path());
        cmd.pdf_data_path(validation("pdf"));
        cmd
    }

    /// `enumerate` `card` into `out`.
    fn enumerate(&self, card: &str, out: &str, extra: &[&str]) {
        let card_path = self.path(&format!("{out}.dat"));
        std::fs::write(&card_path, card).unwrap();
        let output = self
            .vibegraph()
            .arg("enumerate")
            .arg(&card_path)
            .arg("--out")
            .arg(self.path(out))
            .args(extra)
            .output()
            .unwrap();
        assert_ok(&output, "enumerate");
    }

    /// `integrate` the diagram artifact in `enumerated` into `out`.
    fn integrate(&self, enumerated: &str, run_card: &Path, out: &str) -> PathBuf {
        let output = self
            .vibegraph()
            .arg("--no-network")
            .arg("integrate")
            .arg(self.path(enumerated).join("diagrams.bin.zst"))
            .arg("--run-card")
            .arg(run_card)
            .arg("--out")
            .arg(self.path(out))
            .args(["--fixed-budget", "--neval", "4000", "--niter", "3"])
            .args(["--seed", "17", "-q"])
            .output()
            .unwrap();
        assert_ok(&output, "integrate");
        self.path(out).join("grid.bin.zst")
    }

    fn fixed_run_card(&self) -> PathBuf {
        let path = self.path("run_card.dat");
        std::fs::write(&path, fixed_beams(500.0, 500.0)).unwrap();
        path
    }

    fn inspect(&self, grid: &Path, extra: &[&str]) -> Output {
        self.vibegraph()
            .arg("inspect")
            .arg(grid)
            .args(extra)
            .output()
            .unwrap()
    }

    fn json(&self, name: &str) -> Value {
        serde_json::from_str(&std::fs::read_to_string(self.path(name)).unwrap()).unwrap()
    }
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap_or_else(|| panic!("not an array: {v}"))
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| panic!("not a number: {v}"))
}

/// `diagrams.json`, indexed: every diagram's configuration and drawing, every
/// configuration's diagrams (as `s<k>.c<j>`), every set's contact diagrams.
struct Summary {
    diagrams: BTreeMap<String, (Option<String>, Option<String>)>,
    configurations: BTreeMap<String, Vec<String>>,
    contact: BTreeSet<String>,
}

fn index(summary: &Value) -> Summary {
    let mut out = Summary {
        diagrams: BTreeMap::new(),
        configurations: BTreeMap::new(),
        contact: BTreeSet::new(),
    };
    for set in summary["sets"].as_array().unwrap() {
        let k = set["id"].as_str().unwrap();
        for c in set["configurations"].as_array().unwrap() {
            let id = format!("{k}.{}", c["id"].as_str().unwrap());
            out.configurations.insert(id, strings(&c["diagrams"]));
        }
        out.contact.extend(strings(&set["contact"]));
        for d in set["diagram_list"].as_array().unwrap() {
            out.diagrams.insert(
                d["id"].as_str().unwrap().to_string(),
                (
                    d["configuration"].as_str().map(|c| format!("{k}.{c}")),
                    d["svg"].as_str().map(str::to_string),
                ),
            );
        }
    }
    out
}

/// Every check that holds of any ranking paired with the `diagrams.json` it
/// was given.
fn check_paired(ranking: &Value, summary: &Value, enumerated: &Path) {
    assert_eq!(ranking["schema_version"], 1);
    for key in ["process", "model", "diagram_set_digest"] {
        assert_eq!(ranking[key], summary[key], "{key}");
    }
    assert_eq!(ranking["drawings"]["paired"], true);
    let idx = index(summary);

    // Ranked by σ_c, descending, ranks counting from 1.
    let configs = ranking["configurations"].as_array().unwrap();
    assert!(configs.len() > 1, "a single configuration ranks nothing");
    let sigma = f(&ranking["sigma"]);
    let mut cumulative = 0.0;
    for (i, c) in configs.iter().enumerate() {
        assert_eq!(c["rank"].as_u64(), Some(i as u64 + 1));
        if i > 0 {
            assert!(
                f(&configs[i - 1]["sigma"]) >= f(&c["sigma"]),
                "rank {i} above rank {}: {} < {}",
                i + 1,
                configs[i - 1]["sigma"],
                c["sigma"]
            );
        }
        assert_eq!(f(&c["share"]), f(&c["sigma"]) / sigma);
        cumulative += f(&c["share"]);
        assert!((f(&c["cumulative_share"]) - cumulative).abs() <= 1e-15);

        // The configuration is one `diagrams.json` names, holding these diagrams.
        let id = c["id"].as_str().unwrap();
        let theirs = idx
            .configurations
            .get(id)
            .unwrap_or_else(|| panic!("{id} is no configuration of diagrams.json"));
        let ours = strings(&c["diagrams"]);
        assert!(!ours.is_empty());
        for d in &ours {
            assert!(theirs.contains(d), "{id}: {d} not among {theirs:?}");
            assert_eq!(idx.diagrams[d].0.as_deref(), Some(id), "{d}");
        }
        assert!(id.starts_with(&format!("{}.", c["set"].as_str().unwrap())));
        assert!(!c["channels"].as_array().unwrap().is_empty(), "{id}");
        for m in c["members"].as_array().unwrap() {
            let mid = m["configuration"].as_str();
            assert!(mid.is_some(), "{m}");
            for d in strings(&m["diagrams"]) {
                assert_eq!(idx.diagrams[&d].0.as_deref(), mid, "{d}");
            }
        }
    }

    // The shares and the unattributed weight sum to one.
    let mut total = cumulative;
    for u in ranking["unranked"].as_array().unwrap() {
        match u["reason"].as_str().unwrap() {
            "no_configuration" => total += f(&u["share"]),
            "contact" => {
                for d in strings(&u["diagrams"]) {
                    assert!(idx.contact.contains(&d), "{d} is no contact diagram");
                }
            }
            "colour_null" => {
                for d in strings(&u["diagrams"]) {
                    assert!(idx.diagrams[&d].0.is_some(), "{d}");
                }
            }
            other => panic!("unknown reason {other}"),
        }
    }
    assert!((total - 1.0).abs() <= 1e-12, "the shares sum to {total}");

    // Every channel member is a diagram of diagrams.json, in the configuration
    // it names.
    for ch in ranking["channels"].as_array().unwrap() {
        for m in ch["members"].as_array().unwrap() {
            for d in strings(&m["diagrams"]) {
                assert_eq!(
                    idx.diagrams[&d].0.as_deref(),
                    m["configuration"].as_str(),
                    "{d}"
                );
            }
        }
    }

    // The diagram list is diagrams.json's, drawing for drawing, and every
    // drawing exists.
    let listed = ranking["diagrams"].as_array().unwrap();
    assert_eq!(listed.len(), idx.diagrams.len());
    let mut drawn = 0;
    for d in listed {
        let id = d["id"].as_str().unwrap();
        let (configuration, svg) = &idx.diagrams[id];
        assert_eq!(
            d["configuration"].as_str(),
            configuration.as_deref(),
            "{id}"
        );
        assert_eq!(d["svg"].as_str(), svg.as_deref(), "{id}");
        assert!(d["coupling_orders"]
            .as_object()
            .is_some_and(|o| !o.is_empty()));
        if let Some(svg) = d["svg"].as_str() {
            assert!(enumerated.join(svg).is_file(), "{svg} is not on disk");
            drawn += 1;
        }
    }
    assert!(drawn > 0);
}

#[test]
fn the_ranking_pairs_with_the_enumerate_run_it_came_from() {
    let run = Run::new();
    // Not every diagram drawn, so the ranking must carry diagrams.json's nulls.
    run.enumerate(GLUON_CARD, "enum", &["--svg", "first=20"]);
    let grid = run.integrate("enum", &run.fixed_run_card(), "int");
    let out = run.path("ranking.json");
    let inspected = run.inspect(
        &grid,
        &[
            "--ranking",
            out.to_str().unwrap(),
            "--diagrams",
            run.path("enum").to_str().unwrap(),
        ],
    );
    assert_ok(&inspected, "inspect --ranking");
    let ranking = run.json("ranking.json");
    let summary = run.json("enum/diagrams.json");
    check_paired(&ranking, &summary, &run.path("enum"));
    assert_eq!(ranking["beams"], "fixed_energy");
    assert_eq!(ranking["weight"], "amp2");
    assert!(ranking["diagrams"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d["svg"].is_null()));
    assert!(ranking["unranked"]
        .as_array()
        .unwrap()
        .iter()
        .any(|u| u["reason"] == "contact"));

    // The same ranking, through `diagrams.json` named directly, or on stdout.
    let piped = run.inspect(
        &grid,
        &[
            "--ranking",
            "-",
            "--diagrams",
            run.path("enum/diagrams.json").to_str().unwrap(),
        ],
    );
    assert_ok(&piped, "inspect --ranking -");
    let mut piped: Value = serde_json::from_slice(&piped.stdout).unwrap();
    let mut file = ranking.clone();
    piped["drawings"]["summary"] = Value::Null;
    file["drawings"]["summary"] = Value::Null;
    assert_eq!(piped, file);

    // Unpaired, every diagram carries the conventional path.
    let unpaired = run.inspect(&grid, &["--ranking", "-"]);
    assert_ok(&unpaired, "inspect --ranking - (unpaired)");
    let unpaired: Value = serde_json::from_slice(&unpaired.stdout).unwrap();
    assert_eq!(unpaired["drawings"]["paired"], false);
    for d in unpaired["diagrams"].as_array().unwrap() {
        let (id, set) = (d["id"].as_str().unwrap(), d["set"].as_str().unwrap());
        assert_eq!(
            d["svg"].as_str(),
            Some(format!("svg/{set}/{id}.svg").as_str())
        );
    }
    assert_eq!(unpaired["configurations"], ranking["configurations"]);
}

/// The ranking's numbers are the ones `inspect --output-ldjson` reports: each
/// configuration's σ, error and share, the unattributed weight, and each
/// channel's.
#[test]
fn the_ranking_carries_the_shares_inspect_reports() {
    let run = Run::new();
    run.enumerate(GLUON_CARD, "enum", &["--svg", "none"]);
    let grid = run.integrate("enum", &run.fixed_run_card(), "int");
    let ranked = run.inspect(&grid, &["--ranking", "-"]);
    assert_ok(&ranked, "inspect --ranking -");
    let ranking: Value = serde_json::from_slice(&ranked.stdout).unwrap();
    let reported = run.inspect(&grid, &["--output-ldjson"]);
    assert_ok(&reported, "inspect --output-ldjson");
    let result: Value = common::stdout_of(&reported)
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .find(|v| v["kind"] == "result")
        .expect("a result object");

    for key in ["sigma", "sigma_err", "chi2_per_dof", "seed", "unit"] {
        assert_eq!(ranking[key], result[key], "{key}");
    }
    let mut matched = 0;
    for c in result["configurations"].as_array().unwrap() {
        let set = format!("s{}", c["set"]);
        let diagrams: Vec<String> = c["diagrams"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| format!("{set}.d{d}"))
            .collect();
        let entry = if c["configuration"].is_null() {
            ranking["unranked"]
                .as_array()
                .unwrap()
                .iter()
                .find(|u| u["set"] == set.as_str() && u["reason"] == "no_configuration")
        } else {
            ranking["configurations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["set"] == set.as_str() && strings(&r["diagrams"]) == diagrams)
        }
        .unwrap_or_else(|| panic!("{c} is not in the ranking"));
        for key in ["sigma", "sigma_err", "share"] {
            assert_eq!(entry[key], c[key], "{key} of {c}");
        }
        matched += 1;
    }
    assert_eq!(
        matched,
        ranking["configurations"].as_array().unwrap().len() + 1,
        "one evaluator: every ranked configuration and its unattributed entry"
    );
    let channels = ranking["channels"].as_array().unwrap();
    let reported = result["channels"].as_array().unwrap();
    assert_eq!(channels.len(), reported.len());
    for (ours, theirs) in channels.iter().zip(reported) {
        for key in [
            "key",
            "alpha",
            "neval",
            "sigma",
            "sigma_err",
            "chi2_per_dof",
        ] {
            assert_eq!(ours[key], theirs[key], "{key}");
        }
    }
}

/// A `diagrams.json` of another card is refused before anything is written:
/// a new ranking file is not created, and an existing one is left byte for
/// byte, even under `--force`.
#[test]
fn a_summary_of_other_diagrams_is_refused() {
    let run = Run::new();
    run.enumerate(GLUON_CARD, "enum", &["--svg", "none"]);
    run.enumerate(OTHER_CARD, "other", &["--svg", "none"]);
    let grid = run.integrate("enum", &run.fixed_run_card(), "int");
    let ours = run.json("enum/diagrams.json")["diagram_set_digest"]
        .as_str()
        .unwrap()
        .to_string();
    let theirs = run.json("other/diagrams.json")["diagram_set_digest"]
        .as_str()
        .unwrap()
        .to_string();
    assert_ne!(ours, theirs);

    let out = run.path("ranking.json");
    let other = run.path("other");
    let args = [
        "--ranking",
        out.to_str().unwrap(),
        "--diagrams",
        other.to_str().unwrap(),
    ];
    let refused = run.inspect(&grid, &args);
    assert!(!refused.status.success(), "a foreign summary was accepted");
    let stderr = stderr_of(&refused);
    assert!(
        stderr.contains(&ours) && stderr.contains(&theirs),
        "the refusal names neither digest:\n{stderr}"
    );
    assert!(stderr.contains("enumerate"), "no remedy:\n{stderr}");
    assert!(!out.exists(), "a refused ranking was written");

    std::fs::write(&out, "earlier\n").unwrap();
    let forced = run.inspect(&grid, &[&args[..], &["--force"]].concat());
    assert!(!forced.status.success());
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "earlier\n");

    // An existing ranking is kept without --force, and replaced with it.
    let kept = run.inspect(&grid, &["--ranking", out.to_str().unwrap()]);
    assert!(!kept.status.success());
    assert!(stderr_of(&kept).contains("--force"));
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "earlier\n");
    let replaced = run.inspect(&grid, &["--ranking", out.to_str().unwrap(), "--force"]);
    assert_ok(&replaced, "inspect --ranking --force");
    assert_eq!(run.json("ranking.json")["schema_version"], 1);

    // stdout carries the ldjson stream, so it cannot carry the ranking too.
    let clash = run.inspect(&grid, &["--ranking", "-", "--output-ldjson"]);
    assert!(!clash.status.success());
}

/// Under `--output-ldjson` the result object nests the ranking beside its path.
#[test]
fn the_ldjson_result_nests_the_ranking() {
    let run = Run::new();
    run.enumerate(GLUON_CARD, "enum", &["--svg", "none"]);
    let grid = run.integrate("enum", &run.fixed_run_card(), "int");
    let out = run.path("ranking.json");
    let output = run.inspect(
        &grid,
        &["--ranking", out.to_str().unwrap(), "--output-ldjson"],
    );
    assert_ok(&output, "inspect --ranking --output-ldjson");
    let result: Value = common::stdout_of(&output)
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .find(|v| v["kind"] == "result")
        .expect("a result object");
    assert_eq!(result["ranking_path"].as_str(), out.to_str());
    assert_eq!(result["ranking"], run.json("ranking.json"));
}

/// At proton beams each ranked configuration is a flavour group's, and the
/// groups are the ones `enumerate` previews under the same run card. It reads
/// the NNPDF23 set, which only the extended-validation layer fetches.
#[cfg(feature = "extended-validation")]
#[test]
fn proton_beam_sets_carry_their_flavour_groups() {
    let run = Run::new();
    let run_card = validation("madgraph/config_share_pp_eej_run_card.dat");
    run.enumerate(
        PROTON_CARD,
        "enum",
        &["--svg", "first=1", "--run-card", run_card.to_str().unwrap()],
    );
    let grid = run.integrate("enum", &run_card, "int");
    let out = run.path("ranking.json");
    let inspected = run.inspect(
        &grid,
        &[
            "--ranking",
            out.to_str().unwrap(),
            "--diagrams",
            run.path("enum").to_str().unwrap(),
        ],
    );
    assert_ok(&inspected, "inspect --ranking");
    let ranking = run.json("ranking.json");
    let summary = run.json("enum/diagrams.json");
    check_paired(&ranking, &summary, &run.path("enum"));
    assert_eq!(ranking["beams"], "proton");

    let preview = summary["flavour_groups"]["groups"].as_array().unwrap();
    let groups = ranking["flavour_groups"].as_array().unwrap();
    assert_eq!(groups.len(), preview.len());
    for (ours, theirs) in groups.iter().zip(preview) {
        assert_eq!(ours["id"], theirs["id"]);
        assert_eq!(ours["representative"], theirs["representative"]);
        let members: Vec<&Value> = ours["members"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| &m["set"])
            .collect();
        let previewed: Vec<&Value> = theirs["members"].as_array().unwrap().iter().collect();
        assert_eq!(members, previewed);
    }
    assert!(
        groups
            .iter()
            .any(|g| g["members"].as_array().unwrap().len() > 1),
        "no flavour group has more than one member"
    );
    for set in ranking["sets"].as_array().unwrap() {
        let group = set["flavour_group"].as_str().expect("every set is grouped");
        let g = groups.iter().find(|g| g["id"] == group).unwrap();
        assert!(g["members"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["set"] == set["id"]));
        assert_eq!(
            set["integrated"].as_bool(),
            Some(g["representative"] == set["id"])
        );
    }
    // A group's configuration names each member's own configuration.
    let grouped = ranking["configurations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["members"].as_array().unwrap().len() > 1)
        .expect("a configuration of a group of several sets");
    let sets: BTreeSet<&str> = grouped["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["set"].as_str().unwrap())
        .collect();
    assert!(sets.len() > 1, "{grouped}");
}
