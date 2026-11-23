//! Which ratio `generate --reweight-card` writes: the card's `change helicity`
//! line, `--reweight-helicity`, and without either MadGraph's default, the
//! ratio in each event's own helicity.
//!
//! Hermetic: a fixed-energy `e+ e- > mu+ mu-` grid integrated here. The Z mass
//! moves the left- and right-handed couplings apart, so the two ratios differ
//! event by event while sharing the events. What the values are is the
//! MadGraph-reweight oracle's business (`reweight_mg_oracle`); this pins which
//! one each spelling selects, that the run's audit checked every event's
//! per-helicity denominator against the integrand's, and the refusal of a flag
//! the card contradicts.

mod common;

use std::path::{Path, PathBuf};

use common::{fixed_beams, stderr_of, vibegraph};

const LAUNCHES: &str = "launch --rwgt_name=mz\n set mass 23 89.0\nlaunch --rwgt_name=same\n";

/// Each event's `<event>` block up to its weights, and its `mz` weight.
fn events(path: &Path) -> Vec<(String, f64)> {
    let text = std::fs::read_to_string(path).unwrap();
    text.split("<event>")
        .skip(1)
        .map(|body| {
            let (kinematics, rest) = body.split_once("<rwgt>").expect("reweighted events");
            let wgt = rest
                .split("<wgt id='mz'>")
                .nth(1)
                .and_then(|w| w.split("</wgt>").next())
                .expect("an mz weight");
            (kinematics.to_string(), wgt.trim().parse().unwrap())
        })
        .collect()
}

struct Run {
    status: bool,
    log: String,
    lhe: PathBuf,
}

fn generate(dir: &Path, grid: &Path, name: &str, card: &str, flag: Option<&str>) -> Run {
    let rw = dir.join(format!("{name}.dat"));
    std::fs::write(&rw, card).unwrap();
    let lhe = dir.join(format!("{name}.lhe"));
    let mut cmd = vibegraph(dir, dir);
    cmd.arg("--no-network")
        .arg("generate")
        .arg(grid)
        .args(["--seed", "11", "--nevents", "200"])
        .arg("--reweight-card")
        .arg(&rw)
        .arg("-o")
        .arg(&lhe);
    if let Some(flag) = flag {
        cmd.args(["--reweight-helicity", flag]);
    }
    let out = cmd.output().expect("spawn vibegraph");
    Run {
        status: out.status.success(),
        log: stderr_of(&out),
        lhe,
    }
}

#[test]
fn cli_reweight_helicity_follows_the_card_and_the_flag() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    std::fs::write(
        dir.join("proc_card.dat"),
        "import model sm\ngenerate e+ e- > mu+ mu-\n",
    )
    .unwrap();
    std::fs::write(dir.join("run_card.dat"), fixed_beams(45.0, 45.0)).unwrap();
    let integrate = vibegraph(dir, dir)
        .arg("--no-network")
        .arg("integrate")
        .arg(dir.join("proc_card.dat"))
        .arg("--run-card")
        .arg(dir.join("run_card.dat"))
        .arg("--out")
        .arg(dir.join("grid"))
        .args(["--fixed-budget", "--neval", "4000", "--niter", "2"])
        .output()
        .expect("spawn vibegraph");
    assert!(integrate.status.success(), "{}", stderr_of(&integrate));
    let grid = dir.join("grid").join("grid.bin.zst");

    let summed = format!("change helicity False\n{LAUNCHES}");
    let event = format!("change helicity True\n{LAUNCHES}");
    let runs = [
        ("default", LAUNCHES, None, true),
        ("true", event.as_str(), None, true),
        ("false", summed.as_str(), None, false),
        ("flag_summed", LAUNCHES, Some("summed"), false),
        ("flag_event", LAUNCHES, Some("event"), true),
        ("agreeing", summed.as_str(), Some("summed"), false),
    ];
    let mut files = Vec::new();
    for (name, card, flag, per_event) in runs {
        let run = generate(dir, &grid, name, card, flag);
        assert!(run.status, "{name}:\n{}", run.log);
        let want = if per_event {
            "each ratio in each event's own helicity"
        } else {
            "each ratio helicity-summed"
        };
        assert!(run.log.contains(want), "{name}: not `{want}`:\n{}", run.log);
        let audit = run
            .log
            .lines()
            .find(|l| l.contains("events weighted in their own helicity"));
        if per_event {
            let audit = audit.unwrap_or_else(|| panic!("{name}: no audit:\n{}", run.log));
            assert!(
                audit.contains("200 events weighted")
                    && audit.contains("beyond 1e-9: 0;")
                    && audit.contains("refused: 0;")
                    && audit.contains("different mothers: 0"),
                "{name}: {audit}"
            );
        } else {
            assert!(audit.is_none(), "{name}: {audit:?}");
        }
        files.push((name, per_event, events(&run.lhe)));
    }
    // Every run holds the same events; the weights are one of two sets.
    let (_, _, reference) = &files[0];
    let (_, _, other) = &files[2];
    let mut differ = 0;
    for (a, b) in reference.iter().zip(other) {
        assert_eq!(a.0, b.0, "the two ratios share the events");
        differ += usize::from(a.1 != b.1);
    }
    assert!(
        differ * 2 > reference.len(),
        "the two ratios agree on {} of {} events",
        reference.len() - differ,
        reference.len()
    );
    for (name, per_event, got) in &files {
        let want = if *per_event { reference } else { other };
        assert_eq!(got, want, "{name}");
    }

    // A flag the card's own line contradicts is refused, naming both.
    for (card, flag) in [(&summed, "event"), (&event, "summed")] {
        let run = generate(dir, &grid, "contradicted", card, Some(flag));
        assert!(!run.status, "{flag} over the card's line:\n{}", run.log);
        assert!(
            run.log.contains("--reweight-helicity") && run.log.contains("change helicity"),
            "{}",
            run.log
        );
        assert!(!run.lhe.exists());
    }
}

/// A decay-chain card whose records list identical outgoing particles under
/// different mothers (two Z's, each to `mu+ mu-`) reaches the per-event ratio by
/// default, with no `change helicity` line, and is refused with both ways back
/// to the helicity-summed ratio named; the same card with
/// `change helicity False` runs.
#[test]
fn cli_a_default_per_event_refusal_names_both_remedies() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    std::fs::write(
        dir.join("proc_card.dat"),
        "import model sm\ngenerate e+ e- > z z, z > mu+ mu-\n",
    )
    .unwrap();
    std::fs::write(dir.join("run_card.dat"), fixed_beams(125.0, 125.0)).unwrap();
    let integrate = vibegraph(dir, dir)
        .arg("--no-network")
        .arg("integrate")
        .arg(dir.join("proc_card.dat"))
        .arg("--run-card")
        .arg(dir.join("run_card.dat"))
        .arg("--out")
        .arg(dir.join("grid"))
        .args(["--fixed-budget", "--neval", "4000", "--niter", "2"])
        .output()
        .expect("spawn vibegraph");
    assert!(integrate.status.success(), "{}", stderr_of(&integrate));
    let grid = dir.join("grid").join("grid.bin.zst");

    let refused = generate(dir, &grid, "default", LAUNCHES, None);
    assert!(
        !refused.status,
        "the default per-event ratio ran:\n{}",
        refused.log
    );
    assert!(
        refused.log.contains("different mothers")
            && refused
                .log
                .contains("add `change helicity False` to the reweight card")
            && refused.log.contains("`--reweight-helicity summed`"),
        "the refusal does not name both remedies:\n{}",
        refused.log
    );
    assert!(!refused.lhe.exists(), "a refused file is removed");

    let summed = generate(
        dir,
        &grid,
        "summed",
        &format!("change helicity False\n{LAUNCHES}"),
        None,
    );
    assert!(summed.status, "{}", summed.log);
    assert!(
        summed.log.contains("each ratio helicity-summed"),
        "{}",
        summed.log
    );
    let flag = generate(dir, &grid, "flag", LAUNCHES, Some("summed"));
    assert!(flag.status, "{}", flag.log);
    assert_eq!(
        std::fs::read(&summed.lhe).unwrap(),
        std::fs::read(&flag.lhe).unwrap(),
        "the line and the flag ask for the same file"
    );
}
