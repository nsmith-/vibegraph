//! `integrate --check`: everything `integrate` does before its first sampling
//! point, ending in `ok` or in the refusal the full run would have emitted.
//!
//! Needs only the interned SM model, so it runs in the default suite. The
//! parity cases run each card twice, with and without the flag, and compare
//! the two refusal objects whole, so a refusal that moved to a different place
//! in the pipeline, or a message that drifted between the two paths, fails.
//! What they cannot see is a refusal that only appears once points are drawn
//! (a degenerate-point census, a support check): `--check` is blind to those by
//! construction.

mod common;

use std::path::Path;
use std::process::Output;

use serde_json::Value;

const FIXED_BEAMS: (f64, f64) = (45.6, 45.6);

struct Case<'a> {
    name: &'a str,
    proc_card: &'a str,
    run_card_extra: &'a str,
    extra_args: &'a [&'a str],
}

fn run(dir: &Path, case: &Case<'_>, check: bool) -> Output {
    std::fs::write(dir.join("proc_card.dat"), case.proc_card).unwrap();
    std::fs::write(
        dir.join("run_card.dat"),
        format!(
            "{}{}",
            common::fixed_beams(FIXED_BEAMS.0, FIXED_BEAMS.1),
            case.run_card_extra
        ),
    )
    .unwrap();
    let mut cmd = common::vibegraph(dir, dir);
    cmd.arg("--no-network")
        .arg("integrate")
        .arg(dir.join("proc_card.dat"))
        .arg("--run-card")
        .arg(dir.join("run_card.dat"))
        .arg("--out")
        .arg(dir.join("out"))
        .args(["--neval", "1000", "-j", "1"])
        .args(case.extra_args)
        .arg("--output-ldjson");
    if check {
        cmd.arg("--check");
    }
    cmd.output().expect("spawn vibegraph")
}

fn last_object(output: &Output) -> Value {
    let stdout = common::stdout_of(output);
    let line = stdout.lines().last().expect("an object on stdout");
    serde_json::from_str(line).unwrap_or_else(|e| panic!("{e}: {line}"))
}

fn objects(output: &Output) -> Vec<Value> {
    common::stdout_of(output)
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

/// A card refused with the same `error_kind` and `message` whether or not the
/// run was asked to stop before sampling.
#[test]
fn a_refusal_is_the_same_object_with_and_without_the_flag() {
    let cases = [
        Case {
            name: "an unimplemented cut",
            proc_card: "import model sm\ngenerate e+ e- > mu+ mu-\n",
            run_card_extra: "  30 = ht2min\n",
            extra_args: &[],
        },
        Case {
            name: "a mixed multiplicity at fixed-energy beams",
            proc_card: "import model sm\ngenerate e+ e- > mu+ mu-\nadd process e+ e- > mu+ mu- a\n",
            run_card_extra: "",
            extra_args: &[],
        },
        Case {
            name: "a forbidden propagator on a decay",
            proc_card: "import model sm\ngenerate e+ e- > z h, h > e+ e- mu+ mu- $ z\n",
            run_card_extra: "",
            extra_args: &[],
        },
        Case {
            name: "a bad target",
            proc_card: "import model sm\ngenerate e+ e- > mu+ mu-\n",
            run_card_extra: "",
            extra_args: &["--target-rel", "2"],
        },
    ];
    for case in &cases {
        let tmp = tempfile::tempdir().unwrap();
        let full = run(tmp.path(), case, false);
        let checked = run(tmp.path(), case, true);
        assert!(
            !full.status.success(),
            "{}: the full run succeeded",
            case.name
        );
        assert!(
            !checked.status.success(),
            "{}: --check passed a card the run refuses",
            case.name
        );
        let (full, checked) = (last_object(&full), last_object(&checked));
        assert_eq!(full["kind"], "refusal", "{}: {full}", case.name);
        assert_eq!(checked["kind"], "refusal", "{}: {checked}", case.name);
        assert_eq!(full, checked, "{}", case.name);
    }
}

/// The three refusal kinds the parity cases span, so the comparison is not
/// three copies of one path.
#[test]
fn the_parity_cases_span_distinct_refusal_kinds() {
    let tmp = tempfile::tempdir().unwrap();
    let kinds: Vec<String> = [
        (
            "import model sm\ngenerate e+ e- > mu+ mu-\n",
            "  30 = ht2min\n",
            &[][..],
        ),
        (
            "import model sm\ngenerate e+ e- > mu+ mu-\nadd process e+ e- > mu+ mu- a\n",
            "",
            &[][..],
        ),
        (
            "import model sm\ngenerate e+ e- > mu+ mu-\n",
            "",
            &["--target-rel", "2"][..],
        ),
    ]
    .iter()
    .map(|(proc_card, extra, args)| {
        let case = Case {
            name: "",
            proc_card,
            run_card_extra: extra,
            extra_args: args,
        };
        last_object(&run(tmp.path(), &case, true))["error_kind"]
            .as_str()
            .unwrap()
            .to_string()
    })
    .collect();
    let mut distinct = kinds.clone();
    distinct.sort();
    distinct.dedup();
    assert_eq!(distinct.len(), 3, "{kinds:?}");
}

/// A card that passes ends in `ok`, draws no point and writes no artifact.
#[test]
fn a_clean_card_reports_ok_without_sampling_or_an_artifact() {
    let tmp = tempfile::tempdir().unwrap();
    let case = Case {
        name: "clean",
        proc_card: "import model sm\ngenerate e+ e- > mu+ mu-\n",
        run_card_extra: "",
        extra_args: &[],
    };
    let output = run(tmp.path(), &case, true);
    assert!(output.status.success(), "{}", common::stderr_of(&output));
    let all = objects(&output);
    let result = all.last().unwrap();
    assert_eq!(result["kind"], "result");
    assert_eq!(result["check"], "ok");
    assert_eq!(result["process"], "e+ e- > mu+ mu-");
    for object in &all {
        let stage = object["fields"]["stage"].as_str().unwrap_or("");
        assert!(
            !["alpha_survey", "vegas"].contains(&stage),
            "sampled under --check: {object}"
        );
    }
    assert!(
        !tmp.path().join("out").exists(),
        "an artifact directory was made"
    );
}

/// In text mode the check says so on standard output.
#[test]
fn the_text_mode_prints_check_ok() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("proc_card.dat"),
        "import model sm\ngenerate e+ e- > mu+ mu-\n",
    )
    .unwrap();
    std::fs::write(
        tmp.path().join("run_card.dat"),
        common::fixed_beams(45.6, 45.6),
    )
    .unwrap();
    let output = common::vibegraph(tmp.path(), tmp.path())
        .args([
            "integrate",
            "proc_card.dat",
            "--run-card",
            "run_card.dat",
            "--check",
        ])
        .args(["--no-tui", "-j", "1"])
        .output()
        .expect("spawn vibegraph");
    assert!(output.status.success(), "{}", common::stderr_of(&output));
    assert_eq!(common::stdout_of(&output), "check ok: e+ e- > mu+ mu-\n");
}
