//! `vibegraph parameters` and the files `enumerate` writes beside
//! `diagrams.json`: the parameter record of the vendored SMEFTsim model, its
//! default param card and the parameters entering a process.
//!
//! Hermetic: the model is committed under `validation/`.

mod common;

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

const MODEL: &str = "SMEFTsim_topU3l_MwScheme_UFO";

fn ufo_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../validation/ufo")
}

fn run(tmp: &Path, command: &str, card: &str, extra: &[&str]) -> Value {
    std::fs::write(tmp.join("proc_card.dat"), card).unwrap();
    let output = common::vibegraph(tmp, tmp)
        .arg(command)
        .arg("proc_card.dat")
        .args(["--ufo-dir".as_ref(), ufo_dir().as_os_str()])
        .args(["--out", "out", "--output-ldjson"])
        .args(extra)
        .output()
        .expect("spawn vibegraph");
    let stdout = common::stdout_of(&output);
    assert!(
        output.status.success(),
        "{command} failed:\n{stdout}{}",
        common::stderr_of(&output)
    );
    let line = stdout.lines().last().unwrap();
    serde_json::from_str(line).unwrap()
}

fn record<'a>(report: &'a Value, name: &str) -> &'a Value {
    report["parameters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == name)
        .unwrap_or_else(|| panic!("no record for {name}"))
}

fn model_card() -> String {
    format!("import model {MODEL}-massless\n")
}

/// A model-only report: one parameter of each class, no process, no entering
/// list, and a default card that reads back at the restriction's values.
#[test]
fn a_model_only_report_pins_each_class() {
    let tmp = tempfile::tempdir().unwrap();
    let result = run(tmp.path(), "parameters", &model_card(), &[]);
    let report = &result["report"];
    assert_eq!(report["schema_version"], 1);
    assert!(report["process"].is_null());
    assert!(report["entering"].is_null());

    let mc = record(report, "MC");
    assert_eq!(mc["class"], json!({"kind": "zero"}));
    assert_eq!(mc["block"], "MASS");
    assert_eq!(mc["code"], json!([4]));
    assert_eq!(mc["reweight"]["verdict"], "locked");
    assert!(mc["enters"].is_null());
    assert_eq!(mc["reweight"]["polynomial"]["status"], "no_process");

    let mt = record(report, "MT");
    assert_eq!(mt["class"], json!({"kind": "free"}));
    assert_eq!(mt["default"], json!(172.76));
    assert_eq!(mt["particles"][0]["pdg"], 6);
    assert_eq!(mt["particles"][0]["role"], "mass");
    assert!(mt["particles"][0]["external_in_process"].is_null());
    assert!(mt["constraints"]
        .as_array()
        .unwrap()
        .contains(&json!({"kind": "minimum", "value": 0.0})));

    let classes = |kind: &str| -> Vec<&Value> {
        report["parameters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| p["class"]["kind"] == kind)
            .collect()
    };
    let one = classes("one");
    assert!(!one.is_empty());
    assert!(one.iter().all(|p| p["default"] == 1.0));
    assert!(one.iter().all(|p| p["reweight"]["verdict"] == "fixed_one"));
    let merged = classes("merged");
    assert!(!merged.is_empty());
    for m in &merged {
        let into = m["class"]["into"].as_str().unwrap();
        assert_eq!(record(report, into)["class"], json!({"kind": "free"}));
        assert_eq!(m["reweight"]["verdict"], "merged");
    }
    assert_eq!(
        classes("free").len()
            + one.len()
            + merged.len()
            + classes("zero").len()
            + classes("internal").len(),
        report["parameters"].as_array().unwrap().len()
    );

    let card = std::fs::read_to_string(result["param_card_path"].as_str().unwrap()).unwrap();
    assert!(card.contains("Block MASS"), "{card}");
    assert!(report["model"]["digest"].is_string());
}

/// The parameters entering a SMEFTsim process: the Wilson coefficients its
/// couplings carry, and not those of operators the process cannot reach.
#[test]
fn the_entering_list_of_a_smeft_process() {
    let tmp = tempfile::tempdir().unwrap();
    let card = format!("{}generate e+ e- > t t~ NP<=1\n", model_card());
    let result = run(tmp.path(), "parameters", &card, &[]);
    let report = &result["report"];
    assert_eq!(report["process"], "e+ e- > t t~ NP<=1");
    let entering: Vec<&str> = report["entering"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    println!("{entering:?}");
    assert!(!entering.is_empty());
    for name in &entering {
        let r = record(report, name);
        assert!(r["enters"]
            .as_array()
            .unwrap()
            .contains(&json!("couplings")));
    }
    let t = record(report, "MT");
    assert_eq!(t["particles"][0]["external_in_process"], true);
    assert_eq!(t["reweight"]["verdict"], "external_mass");
    assert!(!entering.contains(&"MC"));
}

/// `enumerate` writes the same two files beside `diagrams.json`.
#[test]
fn enumerate_writes_the_report_and_card() {
    let tmp = tempfile::tempdir().unwrap();
    let card = format!("{}generate e+ e- > t t~ NP<=1\n", model_card());
    let standalone = run(tmp.path(), "parameters", &card, &[]);
    let tmp2 = tempfile::tempdir().unwrap();
    let enumerated = run(tmp2.path(), "enumerate", &card, &[]);
    let a: Value = serde_json::from_str(
        &std::fs::read_to_string(enumerated["parameters_path"].as_str().unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(a, standalone["report"]);
    assert_eq!(
        std::fs::read_to_string(enumerated["param_card_path"].as_str().unwrap()).unwrap(),
        std::fs::read_to_string(standalone["param_card_path"].as_str().unwrap()).unwrap()
    );
}
