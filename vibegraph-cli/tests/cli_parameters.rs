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
    assert_eq!(
        classes("free").len() + classes("zero").len() + classes("internal").len(),
        report["parameters"].as_array().unwrap().len(),
        "the massless restriction ties and fixes nothing"
    );
    assert_eq!(report["parameters"].as_array().unwrap().len(), 219);

    let card =
        std::fs::read_to_string(tmp.path().join(result["param_card_path"].as_str().unwrap()))
            .unwrap();
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
    let mut sorted = entering.clone();
    sorted.sort_unstable();
    assert_eq!(
        sorted,
        [
            "Gf",
            "LambdaSMEFT",
            "MW",
            "MZ",
            "cHDD",
            "cHQ1",
            "cHQ3",
            "cHWB",
            "cHe",
            "cHl1",
            "cHl3",
            "cHt",
            "cQe",
            "cQl1",
            "cQl3",
            "cll1",
            "ctBRe",
            "ctWRe",
            "cte",
            "ctl"
        ]
    );
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
        &std::fs::read_to_string(
            tmp2.path()
                .join(enumerated["parameters_path"].as_str().unwrap()),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(a, standalone["report"]);
    assert_eq!(
        std::fs::read_to_string(
            tmp2.path()
                .join(enumerated["param_card_path"].as_str().unwrap())
        )
        .unwrap(),
        std::fs::read_to_string(
            tmp.path()
                .join(standalone["param_card_path"].as_str().unwrap())
        )
        .unwrap()
    );
}

/// The `one` and `merged` classes, which no shipped restriction produces: a
/// copy of the model whose restriction ties `cW` to `cG` and sets `cH` to one.
#[test]
fn a_restriction_that_ties_and_fixes_parameters() {
    let tmp = tempfile::tempdir().unwrap();
    let models = tmp.path().join("ufo");
    let model = models.join(MODEL);
    std::fs::create_dir_all(&model).unwrap();
    let source = ufo_dir().join(MODEL);
    for entry in std::fs::read_dir(&source).unwrap() {
        let path = entry.unwrap().path();
        std::os::unix::fs::symlink(&path, model.join(path.file_name().unwrap())).unwrap();
    }
    let card = std::fs::read_to_string(source.join("restrict_massless.dat")).unwrap();
    let edited = card
        .replace("    2 0.300000e+00  # cW", "    2 0.200000e+00  # cW")
        .replace("    3 0.400000e+00  # cH", "    3 1.000000e+00  # cH");
    assert_ne!(card, edited);
    std::fs::write(model.join("restrict_tied.dat"), edited).unwrap();

    std::fs::write(
        tmp.path().join("proc_card.dat"),
        format!("import model {MODEL}-tied\n"),
    )
    .unwrap();
    let output = common::vibegraph(tmp.path(), tmp.path())
        .arg("parameters")
        .arg("proc_card.dat")
        .args(["--ufo-dir".as_ref(), models.as_os_str()])
        .args(["--out", "out", "--output-ldjson"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", common::stdout_of(&output));
    let stdout = common::stdout_of(&output);
    let result: Value = serde_json::from_str(stdout.lines().last().unwrap()).unwrap();
    let report = &result["report"];

    let ch = record(report, "cH");
    assert_eq!(ch["class"], json!({"kind": "one"}));
    assert_eq!(ch["default"], 1.0);
    assert_eq!(ch["reweight"]["verdict"], "fixed_one");
    assert!(ch["constraints"]
        .as_array()
        .unwrap()
        .contains(&json!({"kind": "fixed", "value": 1.0})));

    let cw = record(report, "cW");
    assert_eq!(
        cw["class"],
        json!({"kind": "merged", "into": "cG", "factor": 1.0})
    );
    assert_eq!(cw["reweight"]["verdict"], "merged");
    let cg = record(report, "cG");
    assert_eq!(cg["class"], json!({"kind": "free"}));
    assert!(cg["constraints"].as_array().unwrap().contains(&json!(
        {"kind": "leads_group", "members": [{"name": "cW", "factor": 1.0}]}
    )));
}
