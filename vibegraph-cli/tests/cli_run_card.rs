//! `vibegraph run-card`: the resolved-card report and the per-release table.
//!
//! The report's refusal is compared with the refusal `integrate --check` emits
//! for the same card, so a rule that moved in one and not the other fails. What
//! the comparison cannot see is a refusal that needs the process (the cut
//! compiler's, the phase-space guards): `--check` reaches those and the report
//! does not.

mod common;

use std::path::Path;
use std::process::Output;

use serde_json::Value;

const PROC: &str = "import model sm\ngenerate e+ e- > mu+ mu-\n";

fn objects(output: &Output) -> Vec<Value> {
    common::stdout_of(output)
        .lines()
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("{e}: {l}")))
        .collect()
}

fn run_card(dir: &Path, card: &str, extra: &[&str]) -> Output {
    std::fs::write(dir.join("run_card.dat"), card).unwrap();
    std::fs::write(dir.join("proc_card.dat"), PROC).unwrap();
    common::vibegraph(dir, dir)
        .arg("--no-network")
        .arg("run-card")
        .arg(dir.join("run_card.dat"))
        .args(extra)
        .arg("--output-ldjson")
        .output()
        .expect("spawn vibegraph")
}

fn result_of(output: &Output) -> Value {
    assert!(output.status.success(), "{}", common::stderr_of(output));
    let last = objects(output).pop().expect("an object on stdout");
    assert_eq!(last["kind"], "result", "{last}");
    last
}

fn integrate_check(dir: &Path, card: &str) -> Value {
    std::fs::write(dir.join("run_card.dat"), card).unwrap();
    std::fs::write(dir.join("proc_card.dat"), PROC).unwrap();
    let output = common::vibegraph(dir, dir)
        .arg("--no-network")
        .arg("integrate")
        .arg(dir.join("proc_card.dat"))
        .arg("--run-card")
        .arg(dir.join("run_card.dat"))
        .arg("--out")
        .arg(dir.join("out"))
        .args(["--neval", "1000", "-j", "1", "--check", "--output-ldjson"])
        .output()
        .expect("spawn vibegraph");
    objects(&output).pop().expect("an object on stdout")
}

fn key<'a>(result: &'a Value, name: &str) -> &'a Value {
    result["keys"]
        .as_array()
        .unwrap()
        .iter()
        .find(|k| k["name"] == name)
        .unwrap_or_else(|| panic!("no key {name}"))
}

fn fixed() -> String {
    common::fixed_beams(45.6, 45.6)
}

/// The sentinels MadGraph's older templates write get past the refusal in the
/// report and in `integrate --check`, and report as moved by the card.
#[test]
fn the_off_sentinels_pass_the_report_and_the_check() {
    let tmp = tempfile::tempdir().unwrap();
    let card = format!(
        "{}  -1 = ptj1min\n  -1 = ptj2min\n  -1 = ptj3min\n  -1 = ptj4min\n  -1 = ptl1min\n  \
         -1 = ptl2min\n  -1 = ptl3min\n  -1 = ptl4min\n  -1 = misset\n  -1 = ptheavy\n  \
         100000 = ht2max\n  100000 = ht3max\n  100000 = ht4max\n",
        fixed()
    );
    let result = result_of(&run_card(tmp.path(), &card, &[]));
    assert_eq!(result["refusal"], Value::Null, "{result}");
    for name in [
        "ptj1min", "ptj2min", "ptj3min", "ptj4min", "ptl1min", "ptl2min", "ptl3min", "ptl4min",
        "misset", "ptheavy", "ht2max", "ht3max", "ht4max",
    ] {
        let k = key(&result, name);
        assert_eq!(k["moved_by_card"], true, "{name}");
        assert_eq!(k["refusal"], Value::Null, "{name}");
        assert_eq!(k["status"], "refused", "{name}");
    }
    let checked = integrate_check(tmp.path(), &card);
    assert_eq!(checked["kind"], "result", "{checked}");
    assert_eq!(checked["check"], "ok");
}

/// A card that cuts, or carries a name this build does not read, is reported
/// with the refusal `integrate --check` emits.
#[test]
fn a_refused_card_reports_the_refusal_the_check_emits() {
    let tmp = tempfile::tempdir().unwrap();
    let cases = [
        ("cuts", format!("{}  30 = ptj1min\n", fixed()), "ptj1min"),
        ("cuts", format!("{}  50 = ht2max\n", fixed()), "ht2max"),
        ("cuts", format!("{}  30 = misset\n", fixed()), "misset"),
        (
            "parse",
            format!("{}  3 = nb_proton1\n", fixed()),
            "nb_proton1",
        ),
    ];
    for (stage, card, name) in &cases {
        let result = result_of(&run_card(tmp.path(), card, &[]));
        let refusal = &result["refusal"];
        assert_eq!(refusal["stage"], *stage, "{name}: {result}");
        let checked = integrate_check(tmp.path(), card);
        assert_eq!(checked["kind"], "refusal", "{name}: {checked}");
        let message = refusal["message"].as_str().unwrap();
        assert!(
            checked["message"].as_str().unwrap().contains(message),
            "{name}: report {message:?} vs check {checked}"
        );
        assert_eq!(key(&result, name)["refusal"], refusal["message"], "{name}");
    }
}

/// A line the parser does not read is listed with its line number, the other
/// parameters resolve, and the card's refusal is the parser's.
#[test]
fn an_unknown_key_is_listed_and_not_dropped() {
    let tmp = tempfile::tempdir().unwrap();
    let card = format!(
        "{}  $DEFAULT_PDF_SETS = lhaid\n  1 = surprise\n  25 = ptj\n",
        fixed()
    );
    let result = result_of(&run_card(tmp.path(), &card, &[]));
    let unknown = result["unknown_keys"].as_array().unwrap();
    assert_eq!(unknown.len(), 2, "{result}");
    assert_eq!(unknown[0]["name"], "lhaid");
    assert_eq!(unknown[0]["problem"], "placeholder");
    assert_eq!(unknown[1]["name"], "surprise");
    assert_eq!(unknown[1]["line"], 6);
    assert_eq!(result["refusal"]["stage"], "parse");
    assert_eq!(key(&result, "ptj")["resolved"], 25.0);
    let checked = integrate_check(tmp.path(), &card);
    assert!(checked["message"]
        .as_str()
        .unwrap()
        .contains(result["refusal"]["message"].as_str().unwrap()));
}

/// With a process the report lists what it moves from the template: a decay
/// removes the default cuts, and a card line overrides one.
#[test]
fn a_decay_reports_the_cuts_it_removes() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("decay.dat"),
        "import model sm\ngenerate z > e+ e-\n",
    )
    .unwrap();
    let output = common::vibegraph(tmp.path(), tmp.path())
        .arg("--no-network")
        .arg("run-card")
        .arg("--proc-card")
        .arg(tmp.path().join("decay.dat"))
        .arg("--output-ldjson")
        .output()
        .unwrap();
    let result = result_of(&output);
    assert_eq!(result["process"], "z > e+ e-");
    assert_eq!(result["run_card"], Value::Null);
    let ptl = result["process_defaults"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["name"] == "ptl")
        .unwrap_or_else(|| panic!("a decay removes ptl: {result}"));
    assert_eq!(ptl["process_default"], 0.0);
    assert_eq!(key(&result, "ptl")["resolved"], 0.0);
    assert_eq!(key(&result, "ptl")["moved_by_card"], false);
    assert_eq!(key(&result, "ptl")["moved_by_resolution"], true);
}

/// A run card and a process together report the card and the process's own
/// defaults.
#[test]
fn a_card_and_a_process_report_together() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("decay.dat"),
        "import model sm\ngenerate z > e+ e-\n",
    )
    .unwrap();
    let output = run_card(
        tmp.path(),
        "  5 = ptl\n",
        &[
            "--proc-card",
            tmp.path().join("decay.dat").to_str().unwrap(),
        ],
    );
    let result = result_of(&output);
    assert_eq!(result["process"], "z > e+ e-");
    let ptl = result["process_defaults"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["name"] == "ptl")
        .unwrap();
    assert_eq!(ptl["overridden_by_card"], true);
    assert_eq!(key(&result, "ptl")["resolved"], 5.0);
}

/// The table has one entry per parameter, with the release it describes, and
/// the plain mode prints the same object as indented JSON.
#[test]
fn the_table_lists_every_parameter() {
    let tmp = tempfile::tempdir().unwrap();
    let output = common::vibegraph(tmp.path(), tmp.path())
        .args(["run-card", "--table", "--output-ldjson"])
        .output()
        .unwrap();
    let result = result_of(&output);
    let parameters = result["parameters"].as_array().unwrap();
    assert_eq!(
        parameters.len(),
        vibegraph::runcard::report::classification_table().len()
    );
    for p in parameters {
        assert!(
            ["honoured", "ignored", "refused"].contains(&p["status"].as_str().unwrap()),
            "{p}"
        );
        assert!(!p["note"].as_str().unwrap().is_empty(), "{p}");
    }
    let by_name = |n: &str| parameters.iter().find(|p| p["name"] == n).unwrap();
    assert_eq!(by_name("ptj").status_of(), "honoured");
    assert_eq!(by_name("ptj1min").status_of(), "refused");
    assert!(by_name("ptj1min")["refused_values"]
        .as_str()
        .unwrap()
        .contains("at or below 0"));
    assert_eq!(by_name("nb_proton1").status_of(), "refused");
    assert_eq!(by_name("iseed").status_of(), "honoured");
    assert!(!result["version"].as_str().unwrap().is_empty());

    let text = common::vibegraph(tmp.path(), tmp.path())
        .args(["run-card", "--table"])
        .output()
        .unwrap();
    assert!(text.status.success());
    let printed: Value = serde_json::from_str(&common::stdout_of(&text)).unwrap();
    assert_eq!(printed["parameters"], result["parameters"]);
}

trait StatusOf {
    fn status_of(&self) -> &str;
}

impl StatusOf for Value {
    fn status_of(&self) -> &str {
        self["status"].as_str().unwrap()
    }
}

/// The command needs something to read, and the table takes no card.
#[test]
fn the_arguments_are_checked() {
    let tmp = tempfile::tempdir().unwrap();
    let none = common::vibegraph(tmp.path(), tmp.path())
        .arg("run-card")
        .output()
        .unwrap();
    assert!(!none.status.success());
    let both = common::vibegraph(tmp.path(), tmp.path())
        .args(["run-card", "--table", "run_card.dat"])
        .output()
        .unwrap();
    assert!(!both.status.success());
    let missing = common::vibegraph(tmp.path(), tmp.path())
        .args(["run-card", "absent.dat"])
        .output()
        .unwrap();
    assert!(!missing.status.success());
    assert!(common::stderr_of(&missing).contains("cannot read the run card"));
}
