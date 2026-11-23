//! Where a `progress` object of `--output-ldjson` sits in the run: the group it
//! belongs to on a card that is integrated in several, and the run's position in
//! its phases.
//!
//! The proton runs read a parton-density set written here, a small smooth grid
//! under the name the default run card asks for, so nothing in this file needs
//! fetched data. The densities are not physical; the records are what is tested.

#![cfg(unix)]

mod common;

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::Value;

use common::PdfDataPath;

const EBEAM: f64 = 45.6;

fn parse_stream(stdout: &str) -> Vec<Value> {
    stdout
        .lines()
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("{e}: {l}")))
        .collect()
}

fn run(mut cmd: std::process::Command) -> Vec<Value> {
    let output = cmd.output().expect("spawn vibegraph");
    let stdout = common::stdout_of(&output);
    assert!(
        output.status.success(),
        "{stdout}{}",
        common::stderr_of(&output)
    );
    parse_stream(&stdout)
}

fn progress<'a>(objects: &'a [Value], stage: &str) -> Vec<&'a Value> {
    objects
        .iter()
        .filter(|o| o["kind"] == "progress" && o["fields"]["stage"] == stage)
        .collect()
}

fn fixed_beam_integrate(dir: &Path, extra: &[&str]) -> std::process::Command {
    let proc_card = dir.join("proc_card.dat");
    let run_card = dir.join("run_card.dat");
    std::fs::write(&proc_card, "import model sm\ngenerate e+ e- > mu+ mu-\n").unwrap();
    std::fs::write(&run_card, common::fixed_beams(EBEAM, EBEAM)).unwrap();
    let mut cmd = common::vibegraph(dir, dir);
    cmd.arg("integrate")
        .arg(proc_card)
        .arg("--run-card")
        .arg(run_card)
        .arg("--out")
        .arg(dir.join("out"))
        .args(["-j", "1", "--output-ldjson", "--fixed-budget"])
        .args(["--neval", "2000", "--niter", "2"])
        .args(extra);
    cmd
}

/// A proton-density set the default run card resolves (`lhaid` 247000), on a
/// grid of smooth, positive densities.
fn write_pdf_set(root: &Path) {
    const NAME: &str = "NNPDF23_lo_as_0130_qed";
    let dir = root.join(NAME);
    std::fs::create_dir_all(&dir).unwrap();
    let flavors: [i32; 14] = [-6, -5, -4, -3, -2, -1, 1, 2, 3, 4, 5, 6, 21, 22];
    let flavor_list = flavors.map(|f| f.to_string()).join(", ");
    let xs: Vec<f64> = (0..12)
        .map(|i| 10f64.powf(-9.0 + 9.0 * i as f64 / 11.0))
        .collect();
    let qs: Vec<f64> = (0..6).map(|i| 10f64.powf(4.0 * i as f64 / 5.0)).collect();
    let alpha_qs = qs
        .iter()
        .map(|q| format!("{q:e}"))
        .collect::<Vec<_>>()
        .join(", ");
    let alpha_vals = qs
        .iter()
        .map(|q| format!("{:e}", 0.5 / (1.0 + q.ln().max(0.0) * 0.6)))
        .collect::<Vec<_>>()
        .join(", ");
    std::fs::write(
        dir.join(format!("{NAME}.info")),
        format!(
            "SetDesc: \"smooth stand-in densities\"\nSetIndex: 247000\nFormat: lhagrid1\n\
             DataVersion: 1\nNumMembers: 1\nParticle: 2212\nFlavors: [{flavor_list}]\n\
             OrderQCD: 0\nFlavorScheme: variable\nNumFlavors: 6\nErrorType: replicas\n\
             XMin: 1e-09\nXMax: 1\nQMin: 1\nQMax: 10000\nMZ: 91.1876\nMUp: 0\nMDown: 0\n\
             MStrange: 0\nMCharm: 1.41421\nMBottom: 4.75\nMTop: 175\nAlphaS_MZ: 0.13\n\
             AlphaS_OrderQCD: 0\nAlphaS_Type: ipol\nAlphaS_Qs: [{alpha_qs}]\n\
             AlphaS_Vals: [{alpha_vals}]\nAlphaS_Lambda4: 0.27\nAlphaS_Lambda5: 0.17\n"
        ),
    )
    .unwrap();
    let row = |v: &[f64]| {
        v.iter()
            .map(|x| format!("{x:e}"))
            .collect::<Vec<_>>()
            .join(" ")
    };
    let mut grid = format!(
        "PdfType: central\nFormat: lhagrid1\n---\n{}\n{}\n{}\n",
        row(&xs),
        row(&qs),
        flavors.map(|f| f.to_string()).join(" ")
    );
    for &x in &xs {
        for &q in &qs {
            let shape = x.powf(0.3) * (1.0 - x).powi(3) * (1.0 + 0.05 * q.ln());
            let values: Vec<f64> = flavors
                .iter()
                .map(|&f| match f {
                    6 | -6 => 0.0,
                    21 => 3.0 * shape,
                    22 => 0.01 * shape,
                    f if f < 0 => 0.5 * shape,
                    _ => shape,
                })
                .collect();
            grid.push_str(&row(&values));
            grid.push('\n');
        }
    }
    grid.push_str("---\n");
    std::fs::write(dir.join(format!("{NAME}_0000.dat")), grid).unwrap();
}

const PROTON_RUN_CARD: &str =
    "  1 = lpp1\n  1 = lpp2\n  6500 = ebeam1\n  6500 = ebeam2\n  lhapdf = pdlabel\n  247000 = lhaid\n  20 = ptb\n";

/// `generate p p > e+ e-` and `add process p p > e+ e- j` on `sm-no_b_mass`:
/// two groups, one per final-state multiplicity.
fn proton_two_group_integrate(dir: &Path, extra: &[&str]) -> std::process::Command {
    let pdf = dir.join("pdf");
    write_pdf_set(&pdf);
    let proc_card = dir.join("proc_card.dat");
    let run_card = dir.join("run_card.dat");
    std::fs::write(
        &proc_card,
        "import model sm-no_b_mass\ngenerate p p > e+ e-\nadd process p p > e+ e- j\n",
    )
    .unwrap();
    std::fs::write(&run_card, PROTON_RUN_CARD).unwrap();
    let mut cmd = common::vibegraph(dir, dir);
    cmd.pdf_data_path(&pdf)
        .arg("integrate")
        .arg(proc_card)
        .arg("--run-card")
        .arg(run_card)
        .arg("--out")
        .arg(dir.join("out"))
        .args([
            "--fixed-budget",
            "--no-network",
            "-j",
            "1",
            "--output-ldjson",
        ])
        .args(["--neval", "1000", "--niter", "2"])
        .args(extra);
    cmd
}

/// The number of phase `start` records before position `at`.
fn phases_started(objects: &[Value], at: usize) -> u64 {
    objects[..at]
        .iter()
        .filter(|o| o["kind"] == "phase" && o["event"] == "start")
        .count() as u64
}

/// Every progress object's `phase` is the number of phases the stream has
/// started by that point, and its `phases`, once present, is the number the
/// whole stream goes on to start. Returns that number.
fn assert_position_matches_the_phase_records(objects: &[Value], complete: bool) -> u64 {
    let started = phases_started(objects, objects.len());
    let mut last_phase = 0;
    let mut phases_seen = BTreeSet::new();
    for (at, o) in objects.iter().enumerate() {
        if o["kind"] != "progress" {
            continue;
        }
        let fields = &o["fields"];
        if let Some(phase) = fields["phase"].as_u64() {
            assert_eq!(phase, phases_started(objects, at), "{o}");
            assert!(phase >= last_phase, "phase went backwards at {o}");
            last_phase = phase;
        } else {
            assert_eq!(phases_started(objects, at), 0, "{o}");
        }
        if let Some(phases) = fields["phases"].as_u64() {
            phases_seen.insert(phases);
        }
    }
    assert_eq!(phases_seen.len(), 1, "{phases_seen:?}");
    let phases = *phases_seen.first().unwrap();
    if complete {
        assert_eq!(phases, started, "the plan is the phases the run entered");
    } else {
        assert!(started <= phases);
    }
    phases
}

/// A single-group card is group 1 of 1, and the run's position follows the
/// phase records.
#[test]
fn a_single_group_run_reports_group_one_of_one() {
    let tmp = tempfile::tempdir().unwrap();
    let objects = run(fixed_beam_integrate(tmp.path(), &[]));

    let phases = assert_position_matches_the_phase_records(&objects, true);
    assert_eq!(phases, 9);

    let channels = objects
        .iter()
        .find(|o| o["kind"] == "phase" && o["event"] == "end" && o["phase"] == "channels")
        .and_then(|o| o["sizes"]["channels"].as_u64())
        .unwrap();
    let survey = progress(&objects, "alpha_survey");
    assert!(!survey.is_empty());
    for o in &survey {
        let f = &o["fields"];
        assert_eq!(f["group"], 1, "{o}");
        assert_eq!(f["groups"], 1, "{o}");
        assert_eq!(f["label"], "@1", "{o}");
        assert_eq!(f["channels"], channels, "{o}");
        assert_eq!(f["phase"], 6, "{o}");
    }
    let compile = progress(&objects, "compile");
    assert!(!compile.is_empty());
    for o in &compile {
        let f = &o["fields"];
        assert_eq!(
            (f["group"].as_u64(), f["groups"].as_u64()),
            (Some(1), Some(1))
        );
        assert_eq!(f["label"], "@1");
        assert!(f.get("channels").is_none(), "{o}");
    }
    for stage in ["vegas", "ufo_load"] {
        for o in progress(&objects, stage) {
            for key in ["group", "groups", "label", "channels"] {
                assert!(o["fields"].get(key).is_none(), "{o}");
            }
        }
    }
}

/// `--check` plans the phases up to the channels, and a run that ends there
/// has entered exactly those.
#[test]
fn a_check_run_plans_the_phases_it_executes() {
    let tmp = tempfile::tempdir().unwrap();
    let objects = run(fixed_beam_integrate(tmp.path(), &["--check"]));
    assert_eq!(assert_position_matches_the_phase_records(&objects, true), 5);
}

/// `generate` knows its three phases before it reads anything, so every
/// progress object carries them; its compile stage is one group.
#[test]
fn a_generate_run_carries_its_three_phases() {
    let tmp = tempfile::tempdir().unwrap();
    run(fixed_beam_integrate(tmp.path(), &[]));
    let mut cmd = common::vibegraph(tmp.path(), tmp.path());
    cmd.arg("generate")
        .arg(tmp.path().join("out/grid.bin.zst"))
        .args(["--nevents", "50", "-o"])
        .arg(tmp.path().join("events.lhe"))
        .args(["--seed", "7", "-j", "1", "--output-ldjson"]);
    let objects = run(cmd);
    assert_eq!(assert_position_matches_the_phase_records(&objects, true), 3);
    for o in objects.iter().filter(|o| o["kind"] == "progress") {
        assert!(o["fields"]["phases"] == 3, "{o}");
    }
    let compile = progress(&objects, "compile");
    assert!(!compile.is_empty());
    for o in compile {
        assert_eq!(o["fields"]["group"], 1);
        assert_eq!(o["fields"]["groups"], 1);
    }
}

/// A card of two final-state multiplicities is two groups: each is surveyed and
/// compiled under its own tag and channel count, and the position never goes
/// backwards across the stream.
#[test]
fn a_two_group_proton_run_numbers_its_groups() {
    let tmp = tempfile::tempdir().unwrap();
    let objects = run(proton_two_group_integrate(tmp.path(), &[]));

    let phases = assert_position_matches_the_phase_records(&objects, true);
    assert_eq!(phases, 10);

    let result = objects.iter().find(|o| o["kind"] == "result").unwrap();
    let parts = result["multiplicities"].as_array().unwrap();
    assert_eq!(parts.len(), 2, "{result}");

    let survey = progress(&objects, "alpha_survey");
    let compile = progress(&objects, "compile");
    let mut labels = Vec::new();
    for (stage, records) in [("alpha_survey", &survey), ("compile", &compile)] {
        assert!(!records.is_empty(), "{stage}");
        let mut by_group: Vec<(u64, &Value)> = Vec::new();
        for o in records {
            let f = &o["fields"];
            assert_eq!(f["groups"], 2, "{o}");
            by_group.push((f["group"].as_u64().unwrap(), o));
        }
        let groups: Vec<u64> = by_group.iter().map(|(g, _)| *g).collect();
        assert!(
            groups.windows(2).all(|w| w[0] <= w[1]),
            "{stage}: {groups:?}"
        );
        assert_eq!(
            groups.iter().copied().collect::<BTreeSet<_>>(),
            BTreeSet::from([1, 2]),
            "{stage}"
        );
        // Within a group the count rises.
        let key: Vec<(u64, u64)> = by_group
            .iter()
            .map(|(g, o)| (*g, o["fields"]["done"].as_u64().unwrap()))
            .collect();
        assert!(key.windows(2).all(|w| w[0] <= w[1]), "{stage}: {key:?}");
        for (g, o) in &by_group {
            let part = &parts[*g as usize - 1];
            let ids: Vec<String> = part["process_ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| format!("@{i}"))
                .collect();
            assert_eq!(o["fields"]["label"], ids.join(" "), "{stage}: {o}");
            if stage == "alpha_survey" {
                assert_eq!(o["fields"]["channels"], part["channels"], "{o}");
            } else {
                assert!(o["fields"].get("channels").is_none(), "{o}");
            }
        }
        labels.push(
            by_group
                .iter()
                .map(|(g, o)| (*g, o["fields"]["label"].as_str().unwrap().to_string()))
                .collect::<BTreeSet<_>>(),
        );
    }
    assert_eq!(labels[0], labels[1]);
    let distinct: BTreeSet<&String> = labels[0].iter().map(|(_, l)| l).collect();
    assert_eq!(distinct.len(), 2, "{labels:?}");
    assert_eq!(
        labels[0].iter().cloned().collect::<Vec<_>>(),
        [(1, "@1".to_string()), (2, "@2".to_string())]
    );

    // The pooled stages carry no group.
    for o in progress(&objects, "vegas") {
        assert!(o["fields"].get("group").is_none(), "{o}");
    }

    // The global position, read as (phase, group, done) over the per-group
    // stages and as `phase` over the whole stream, never goes backwards.
    let key = |o: &Value| {
        let f = &o["fields"];
        (
            f["phase"].as_u64().unwrap(),
            f["group"].as_u64().unwrap_or(0),
            f["done"].as_u64().unwrap(),
        )
    };
    let grouped: Vec<(u64, u64, u64)> = objects
        .iter()
        .filter(|o| o["kind"] == "progress" && o["fields"].get("group").is_some())
        .map(key)
        .collect();
    assert!(grouped.windows(2).all(|w| w[0] <= w[1]), "{grouped:?}");
}

/// The extras are additive: with them removed, every progress object of every
/// stage has the fields it had, and the records come in the order they did.
#[test]
fn the_extras_leave_the_existing_fields_and_order_alone() {
    let tmp = tempfile::tempdir().unwrap();
    let objects = run(proton_two_group_integrate(tmp.path(), &[]));

    let extras = ["phase", "phases", "group", "groups", "label", "channels"];
    let mut keys_by_stage: std::collections::BTreeMap<String, BTreeSet<String>> =
        Default::default();
    for o in objects.iter().filter(|o| o["kind"] == "progress") {
        let stage = o["fields"]["stage"].as_str().unwrap().to_string();
        let keys: BTreeSet<String> = o["fields"]
            .as_object()
            .unwrap()
            .keys()
            .filter(|k| !extras.contains(&k.as_str()))
            .cloned()
            .collect();
        keys_by_stage.entry(stage).or_default().extend(keys);
        assert_eq!(o["schema_version"], 1);
        for k in ["timestamp", "level", "target", "fields", "kind"] {
            assert!(o.get(k).is_some(), "{o}");
        }
    }
    let set = |keys: &[&str]| keys.iter().map(|k| k.to_string()).collect::<BTreeSet<_>>();
    assert_eq!(keys_by_stage["ufo_load"], set(&["stage", "done", "total"]));
    assert_eq!(keys_by_stage["compile"], set(&["stage", "done", "total"]));
    assert_eq!(
        keys_by_stage["alpha_survey"],
        set(&["stage", "done", "total"])
    );
    assert_eq!(
        keys_by_stage["enumerate"],
        set(&[
            "stage",
            "done",
            "total",
            "line",
            "lines",
            "subprocesses",
            "diagrams"
        ])
    );
    let vegas = &keys_by_stage["vegas"];
    for k in ["stage", "done", "sigma", "err"] {
        assert!(vegas.contains(k), "{vegas:?}");
    }

    // The order of the stream: each run of records of one kind and stage, in
    // turn, with the phase records between them.
    let mut order: Vec<String> = Vec::new();
    for o in &objects {
        let token = match o["kind"].as_str().unwrap() {
            "phase" => format!(
                "phase {} {}",
                o["phase"].as_str().unwrap(),
                o["event"].as_str().unwrap()
            ),
            "progress" => format!("progress {}", o["fields"]["stage"].as_str().unwrap()),
            "result" => "result".to_string(),
            _ => continue,
        };
        if order.last() != Some(&token) {
            order.push(token);
        }
    }
    let position = |t: &str| {
        order
            .iter()
            .position(|x| x == t)
            .unwrap_or_else(|| panic!("{t}: {order:?}"))
    };
    let expected = [
        "phase input start",
        "progress ufo_load",
        "phase input end",
        "phase enumerate start",
        "progress enumerate",
        "phase enumerate end",
        "phase compile start",
        "progress compile",
        "phase compile end",
        "phase alpha_survey start",
        "progress alpha_survey",
        "phase alpha_survey end",
        "phase vegas start",
        "progress vegas",
        "phase vegas end",
        "phase bank end",
        "result",
    ];
    let at: Vec<usize> = expected.iter().map(|t| position(t)).collect();
    assert!(at.windows(2).all(|w| w[0] < w[1]), "{order:?}");
}
