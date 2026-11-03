//! `integrate --param-card`: what the artifact records of a card, what
//! `generate` evaluates at, and the cards `integrate` refuses.
//!
//! Needs only the interned SM. The process is `e+ e- > mu+ mu-` at
//! `√s = 91.2 GeV`, on the Z peak, where moving `MZ` by a GeV changes σ by
//! tens of percent: a run evaluated at the wrong point is far outside any
//! statistical band.
//!
//! # What these tests cannot see
//!
//! They compare vibegraph with itself. Whether the σ at a moved card is
//! MadGraph's σ at that card is a banked-reference question, not answered here.
//! The bit-for-bit rebind of every parameter and coupling is pinned in
//! `vibegraph::ufo::bind`'s unit tests; here the evidence that `generate`
//! evaluates at the recorded point is statistical (the sample's own σ estimate
//! against the artifact's) plus the reweighter's per-event audit and its ratios.

mod common;

use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::OnceLock;

use vibegraph::artifact::{IntegrateArtifact, MovedParam};
use vibegraph::lhef::emit::sample_estimate_in;
use vibegraph::lhef::parse::LheFile;

const PROCESS: &str = "e+ e- > mu+ mu-";
const EBEAM: f64 = 45.6;
const NEVAL: &str = "8000";
const NITER: &str = "3";
const SEED: &str = "20261028";
const EVENTS: &str = "2000";

/// MadGraph's own `param_card.dat` for the default SM restriction, with every
/// dependent mass and width it writes for a shower.
fn madgraph_card() -> PathBuf {
    common::validation_dir().join("dy13_param_card.dat")
}

struct Integrated {
    output: Output,
    artifact: PathBuf,
}

impl Integrated {
    fn read(&self) -> IntegrateArtifact {
        IntegrateArtifact::read_from_path(&self.artifact).expect("reload artifact")
    }

    fn bytes(&self) -> Vec<u8> {
        std::fs::read(&self.artifact).unwrap()
    }
}

fn scratch() -> &'static Path {
    static DIR: OnceLock<tempfile::TempDir> = OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().unwrap()).path()
}

/// `integrate` the fixed process into `<scratch>/<name>/`, with the param card
/// whose text is `card` when there is one.
fn integrate(name: &str, card: Option<&str>) -> Integrated {
    let dir = scratch().join(name);
    std::fs::create_dir_all(&dir).unwrap();
    let proc_card = dir.join("proc_card.dat");
    let run_card = dir.join("run_card.dat");
    std::fs::write(&proc_card, format!("import model sm\ngenerate {PROCESS}\n")).unwrap();
    std::fs::write(&run_card, common::fixed_beams(EBEAM, EBEAM)).unwrap();
    let out = dir.join("out");
    let mut cmd = common::vibegraph_ambient();
    cmd.arg("integrate")
        .arg(&proc_card)
        .arg("--run-card")
        .arg(&run_card)
        .arg("--out")
        .arg(&out)
        .args(["--fixed-budget", "--neval", NEVAL, "--niter", NITER])
        .args(["--seed", SEED]);
    if let Some(text) = card {
        let path = dir.join("param_card.dat");
        std::fs::write(&path, text).unwrap();
        cmd.arg("--param-card").arg(path);
    }
    let output = cmd.output().expect("spawn vibegraph");
    Integrated {
        output,
        artifact: out.join("grid.bin.zst"),
    }
}

fn succeeded(run: &Integrated) {
    assert!(
        run.output.status.success(),
        "integrate failed:\n{}",
        common::stderr_of(&run.output)
    );
}

/// The run with no card, shared by the cases that compare against it.
fn default_run() -> &'static Integrated {
    static RUN: OnceLock<Integrated> = OnceLock::new();
    RUN.get_or_init(|| {
        let run = integrate("none", None);
        succeeded(&run);
        run
    })
}

const MOVED_MZ: f64 = 92.5;

/// The run with `MZ` moved, shared by the cases that read it.
fn moved_run() -> &'static Integrated {
    static RUN: OnceLock<Integrated> = OnceLock::new();
    RUN.get_or_init(|| {
        let run = integrate("mz", Some(&format!("Block MASS\n 23 {MOVED_MZ} # MZ\n")));
        succeeded(&run);
        run
    })
}

/// `generate` from `artifact` into `<scratch>/<name>`, with `extra` flags;
/// returns the file's text.
fn generate(artifact: &Path, name: &str, extra: &[&str]) -> String {
    let out = scratch().join(name);
    let output = common::vibegraph_ambient()
        .arg("generate")
        .arg(artifact)
        .args(["--seed", "7", "--nevents", EVENTS, "--strategy", "buffer"])
        .arg("-o")
        .arg(&out)
        .args(extra)
        .output()
        .expect("spawn vibegraph");
    assert!(
        output.status.success(),
        "generate failed:\n{}",
        common::stderr_of(&output)
    );
    std::fs::read_to_string(&out).unwrap()
}

/// The file from `<init>` on: the events, without the header that names the
/// artifact's path.
fn from_init(text: &str) -> &str {
    &text[text.find("<init>").expect("an <init> block")..]
}

/// No card, an empty card and a card restating the defaults all record no
/// parameter and write the same artifact, byte for byte; and the defaults card
/// generates the same events.
#[test]
fn cards_at_the_defaults_record_nothing_and_write_the_same_artifact() {
    let none = default_run();
    assert_eq!(none.read().inputs.params, Vec::<MovedParam>::new());
    for (name, card) in [
        ("empty", ""),
        (
            "restated",
            "Block MASS\n 23 9.118800e+01 # MZ\nBlock SMINPUTS\n 1 1.325070e+02\n\
             DECAY 23 2.441404e+00\nDECAY 25 6.38233934e-03\n",
        ),
    ] {
        let run = integrate(name, Some(card));
        succeeded(&run);
        assert_eq!(run.read().inputs.params, Vec::<MovedParam>::new(), "{name}");
        assert!(run.bytes() == none.bytes(), "{name}: the artifact differs");
        let notices = common::stderr_of(&run.output);
        assert!(!notices.contains("ignored"), "{name}: {notices}");
    }
    let restated = scratch().join("restated/out/grid.bin.zst");
    let a = generate(&none.artifact, "none.lhe", &[]);
    let b = generate(&restated, "restated.lhe", &[]);
    assert!(from_init(&a) == from_init(&b), "the event files differ");
}

/// MadGraph's own card for the restriction binds with no refusal and no
/// warning, and records the one value it does not restate exactly: MadGraph
/// writes the Higgs width with `%e`, two digits short of the restriction's.
#[test]
fn madgraphs_own_card_records_the_width_it_rounds() {
    let text = std::fs::read_to_string(madgraph_card()).unwrap();
    let run = integrate("madgraph", Some(&text));
    succeeded(&run);
    let notices = common::stderr_of(&run.output);
    assert!(!notices.contains("ignored"), "{notices}");
    let params = run.read().inputs.params;
    assert_eq!(params.len(), 1, "{params:?}");
    assert_eq!(
        (params[0].name.as_str(), params[0].value),
        ("WH", 6.382339e-03)
    );
}

/// A moved `MZ` is recorded as one parameter at the card's value, and moves σ.
#[test]
fn a_moved_parameter_is_recorded() {
    let moved = moved_run().read();
    assert_eq!(
        moved.inputs.params,
        vec![MovedParam {
            name: "MZ".to_string(),
            block: "MASS".to_string(),
            code: vec![23],
            value: MOVED_MZ,
        }]
    );
    let default = default_run().read();
    let shift = moved.sigma_pb / default.sigma_pb - 1.0;
    assert!(shift.abs() > 0.2, "σ moved by {shift}");
}

/// `generate` evaluates at the recorded point: the sample's own σ estimate
/// agrees with the artifact's σ, which the defaults' σ is far from; and the
/// `<slha>` header carries the moved MZ and the W mass computed from it.
#[test]
fn generate_evaluates_at_the_recorded_point() {
    let run = moved_run();
    let artifact = run.read();
    let text = generate(&run.artifact, "mz.lhe", &[]);
    let (estimate, error) = sample_estimate_in(&text).expect("a sample estimate");
    let pull = (estimate - artifact.sigma_pb) / error.hypot(artifact.sigma_err_pb);
    assert!(
        pull.abs() < 5.0,
        "sample σ {estimate} ± {error} against the artifact's {} ± {}",
        artifact.sigma_pb,
        artifact.sigma_err_pb
    );
    let default = default_run().read();
    assert!(
        ((estimate - default.sigma_pb) / error).abs() > 20.0,
        "the sample σ {estimate} is the defaults' {}",
        default.sigma_pb
    );

    let slha = &text[text.find("<slha>").unwrap()..text.find("</slha>").unwrap()];
    assert!(slha.contains("\n   23 9.250000e+01 # MZ \n"), "{slha}");
    let mw_line = slha
        .lines()
        .find(|l| l.contains("# W+ : MW") || l.contains("# w+ : MW"))
        .unwrap_or_else(|| panic!("no W mass line:\n{slha}"));
    let mw: f64 = mw_line.split_whitespace().nth(1).unwrap().parse().unwrap();
    assert!((mw - 80.419).abs() > 0.5, "{mw_line}");
}

/// A reweight card's hypotheses are taken against the recorded point: a
/// hypothesis at the recorded MZ weighs every event as it is, one at the
/// restriction's default does not.
#[test]
fn reweighting_takes_the_recorded_point_as_its_denominator() {
    let run = moved_run();
    let card = scratch().join("reweight_card.dat");
    std::fs::write(
        &card,
        format!(
            "launch --rwgt_name=recorded\n set mass 23 {MOVED_MZ}\n\
             launch --rwgt_name=default\n set mass 23 91.188\n"
        ),
    )
    .unwrap();
    let text = generate(
        &run.artifact,
        "mz_rwgt.lhe",
        &["--reweight-card", card.to_str().unwrap()],
    );
    let file = LheFile::parse(&text).unwrap();
    let mut off_default = 0.0;
    for event in &file.events {
        let weight = |id: &str| -> f64 {
            event
                .trailer
                .iter()
                .find_map(|l| l.trim().strip_prefix(&format!("<wgt id='{id}'>")))
                .unwrap_or_else(|| panic!("no weight {id}"))
                .trim_end_matches("</wgt>")
                .trim()
                .parse()
                .unwrap()
        };
        // Both weights are written to eight significant digits.
        let recorded = weight("recorded") / event.weight;
        assert!((recorded - 1.0).abs() < 1e-7, "recorded ratio {recorded}");
        off_default += (weight("default") / event.weight - 1.0).abs();
    }
    let mean = off_default / file.events.len() as f64;
    assert!(mean > 0.1, "the default hypothesis moved weights by {mean}");
}

/// A card value for the W mass, which the SM computes, is ignored with one
/// warning naming the parameter, the card's value and the computed one; the
/// run is the default run, byte for byte.
#[test]
fn a_dependent_mass_is_ignored_with_a_warning() {
    let run = integrate("mw", Some("Block MASS\n 24 80.0 # MW\n"));
    succeeded(&run);
    let notices = common::stderr_of(&run.output);
    let warnings: Vec<&str> = notices.lines().filter(|l| l.contains("ignored")).collect();
    assert_eq!(warnings.len(), 1, "{notices}");
    assert!(
        warnings[0].contains("MASS 24 sets MW to 80") && warnings[0].contains("computes as 80.4"),
        "{}",
        warnings[0]
    );
    assert!(run.bytes() == default_run().bytes());
}

/// Each card `integrate` refuses, by what its message says; none writes an
/// artifact.
#[test]
fn cards_the_model_cannot_honour_are_refused() {
    for (name, card, says) in [
        (
            "auto",
            "DECAY 25 Auto\n",
            &[
                "width of particle 25 to Auto",
                "not supported yet",
                "give the width",
            ][..],
        ),
        (
            "revive",
            "Block MASS\n 4 1.27 # MC\n",
            &["MC (block mass, code 4)", "fixes it to zero"][..],
        ),
        (
            "unknown",
            "Block NOSUCH\n 1 2.0\n",
            &["block nosuch code 1", "no parameter there"][..],
        ),
    ] {
        let run = integrate(name, Some(card));
        let notices = common::stderr_of(&run.output);
        assert!(!run.output.status.success(), "{name}: accepted");
        for text in says {
            assert!(notices.contains(text), "{name}: no {text:?} in\n{notices}");
        }
        assert!(!run.artifact.exists(), "{name}: an artifact was written");
    }
}
