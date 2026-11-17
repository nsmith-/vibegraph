//! `enumerate --output-ldjson`'s progress: the total known before the walk,
//! and the line and running counts beside it.
//!
//! Needs only the interned SM model, so it runs in the default suite. It reads
//! the stream's own fields and the summary the run wrote, so it checks that the
//! progress agrees with the result, not that either is the right set of
//! diagrams.

mod common;

use serde_json::Value;

fn enumerate_ldjson(card: &str) -> Vec<Value> {
    let tmp = tempfile::tempdir().unwrap();
    let card_path = tmp.path().join("proc_card.dat");
    std::fs::write(&card_path, card).unwrap();
    let output = common::vibegraph(tmp.path(), tmp.path())
        .arg("enumerate")
        .arg(&card_path)
        .args(["--out", "enum", "-j", "1", "--output-ldjson"])
        .output()
        .expect("spawn vibegraph");
    assert!(output.status.success(), "{}", common::stderr_of(&output));
    common::stdout_of(&output)
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

#[test]
fn enumerate_progress_carries_a_total_the_line_and_the_running_diagram_count() {
    let objects =
        enumerate_ldjson("import model sm\ngenerate p p > e+ e-\nadd process p p > e+ e- j\n");
    let progress: Vec<&Value> = objects
        .iter()
        .filter(|o| o["kind"] == "progress" && o["fields"]["stage"] == "enumerate")
        .map(|o| &o["fields"])
        .collect();
    assert!(!progress.is_empty());
    for f in &progress {
        let (done, total) = (f["done"].as_u64().unwrap(), f["total"].as_u64().unwrap());
        assert!(done <= total, "{f}");
        assert_eq!(f["lines"], 2, "{f}");
        assert!([1, 2].contains(&f["line"].as_u64().unwrap()), "{f}");
    }
    // Each line's walk starts at zero and ends with every candidate considered.
    for line in [1u64, 2] {
        let walk: Vec<&&Value> = progress
            .iter()
            .filter(|f| f["line"].as_u64() == Some(line))
            .collect();
        assert_eq!(walk.first().unwrap()["done"], 0, "line {line}");
        let last = walk.last().unwrap();
        assert_eq!(last["done"], last["total"], "line {line}");
        assert!(last["total"].as_u64().unwrap() > 1, "line {line}");
    }
    // The running diagram count only grows within the card, and ends at the
    // summary's total.
    let diagrams: Vec<u64> = progress
        .iter()
        .map(|f| f["diagrams"].as_u64().unwrap())
        .collect();
    assert!(diagrams.windows(2).all(|w| w[0] <= w[1]), "{diagrams:?}");
    let result = objects.last().unwrap();
    assert_eq!(result["kind"], "result");
    let summary_total: u64 = result["summary"]["sets"]
        .as_array()
        .expect("the summary lists the sets")
        .iter()
        .map(|s| s["diagrams"].as_u64().unwrap())
        .sum();
    assert_eq!(*diagrams.last().unwrap(), summary_total);
}
