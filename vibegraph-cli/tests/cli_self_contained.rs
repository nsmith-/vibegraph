//! An integration started from a proc card or from that card's diagram
//! artifact, and a generation that replays it from the integrate artifact alone.
//!
//! What is pinned here, on the binary:
//!
//! * `integrate` tells a diagram artifact from a proc card by content, and both
//!   give the same integrate artifact, byte for byte, at a pinned seed;
//! * from a diagram artifact nothing is enumerated, with or without a run card,
//!   and `--ufo-dir` is refused;
//!   a proc card on stdin still integrates;
//! * `generate` reads no UFO directory and enumerates nothing: a non-SM model
//!   replays with its directory deleted and no search path set;
//! * `generate` recomputes the banked amplitude probe and refuses an artifact
//!   whose banked |M|² moved by a relative 1e-8, accepting one moved by 1e-13.
//!
//! Hermetic: the built-in SM and a committed toy UFO, fixed-energy beams, no PDF
//! set.

mod common;

use std::path::{Path, PathBuf};
use std::process::Output;

use common::{fixed_beams, stderr_of};
use vibegraph::artifact::IntegrateArtifact;
use vibegraph::diagrams::artifact::DiagramArtifact;
use vibegraph::diagrams::{parse_proc_card, EnumerationPool, ParsingOptions};
use vibegraph::ufo::loaded::LoadedModel;
use vibegraph::ufo::sm::SMRestrict;

/// Two subprocesses over the same particles (a fixed-energy run requires
/// that), told apart by the W⁺ polarization, the second on an `@`-numbered
/// line, so the sets, their order and the per-line process numbers all reach
/// the artifact.
const SM_CARD: &str =
    "import model sm\ngenerate e+ e- > w+{0} w-\nadd process e+ e- > w+{T} w- @7\n";

const TOY_MODEL: &str = "vibegraph_toy_UFO";
const TOY_CARD: &str = "import model vibegraph_toy_UFO\ngenerate lt~ lt > vt vt\n";

/// The line enumeration logs, at the default level, every time it runs.
const ENUMERATED: &str = "enumerated ";

fn toy_ufo_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../validation/ufo")
}

fn write(path: &Path, text: &str) -> PathBuf {
    std::fs::write(path, text).unwrap();
    path.to_path_buf()
}

/// The SM card's diagram artifact, written through the library as `vibegraph
/// enumerate` writes it.
fn sm_diagram_artifact(dir: &Path) -> PathBuf {
    let card = parse_proc_card(SM_CARD, &ParsingOptions::default()).unwrap();
    let model = LoadedModel::interned_sm(SMRestrict::Default);
    let artifact =
        DiagramArtifact::enumerate(SM_CARD, &card, &model, EnumerationPool::Serial).unwrap();
    assert!(artifact.sets.len() > 1);
    let path = dir.join("diagrams.bin.zst");
    artifact.write_to_path(&path, false).unwrap();
    path
}

fn integrate(cwd: &Path, input: &Path, out: &Path, extra: &[&str]) -> Output {
    common::vibegraph(cwd, cwd)
        .arg("--no-network")
        .arg("integrate")
        .arg(input)
        .arg("--out")
        .arg(out)
        .args([
            "--fixed-budget",
            "--neval",
            "2000",
            "--niter",
            "2",
            "--seed",
            "11",
        ])
        .args(extra)
        .output()
        .expect("spawn vibegraph")
}

fn generate(cwd: &Path, grid: &Path, out: &str, extra: &[&str]) -> Output {
    common::vibegraph(cwd, cwd)
        .arg("--no-network")
        .arg("generate")
        .arg(grid)
        .args(["--nevents", "20", "--seed", "5", "--force"])
        .arg("-o")
        .arg(cwd.join(out))
        .args(extra)
        .output()
        .expect("spawn vibegraph")
}

fn assert_ok(output: &Output, what: &str) {
    assert!(
        output.status.success(),
        "{what} failed:\n{}",
        stderr_of(output)
    );
}

fn raw(path: &Path) -> Vec<u8> {
    zstd::decode_all(std::fs::read(path).unwrap().as_slice()).unwrap()
}

/// The same card integrated from its text and from its diagram artifact, at one
/// seed, writes the same integrate artifact byte for byte, with and without a
/// run card. The artifact route enumerates nothing, its default run card built
/// from the facts the artifact banks; the card route enumerates the card once,
/// its default run card read off that same enumeration.
///
/// Byte equality covers every field, the diagram-set digest, the probe and the
/// default run card included, so no field legitimately differs. It cannot see
/// a difference both routes share (both compile from the same sets in the same
/// build), and the card has no decay chain, so the core enumeration a
/// decay-chain line adds is not counted here.
#[test]
fn a_proc_card_and_its_diagram_artifact_integrate_to_the_same_artifact() {
    let tmp = tempfile::tempdir().unwrap();
    let card = write(&tmp.path().join("proc_card.dat"), SM_CARD);
    let diagrams = sm_diagram_artifact(tmp.path());
    let run_card = write(&tmp.path().join("run_card.dat"), &fixed_beams(250.0, 250.0));
    let run_card = run_card.to_str().unwrap();

    for (tag, extra) in [("run", &["--run-card", run_card][..]), ("default", &[][..])] {
        let from_card = tmp.path().join(format!("{tag}-card"));
        let from_diagrams = tmp.path().join(format!("{tag}-diagrams"));
        let a = integrate(tmp.path(), &card, &from_card, extra);
        assert_ok(&a, "integrate from the proc card");
        let b = integrate(tmp.path(), &diagrams, &from_diagrams, extra);
        assert_ok(&b, "integrate from the diagram artifact");
        assert_eq!(
            stderr_of(&a).matches(ENUMERATED).count(),
            1,
            "{tag}: {}",
            stderr_of(&a)
        );
        assert!(
            !stderr_of(&b).contains(ENUMERATED),
            "{tag}: integrating a diagram artifact enumerated:\n{}",
            stderr_of(&b)
        );

        let grid_a = from_card.join("grid.bin.zst");
        let grid_b = from_diagrams.join("grid.bin.zst");
        let (x, y) = (
            IntegrateArtifact::read_from_path(&grid_a).unwrap(),
            IntegrateArtifact::read_from_path(&grid_b).unwrap(),
        );
        let (_, enumerated) = DiagramArtifact::read_from_path(&diagrams).unwrap();
        assert_eq!(x.model, enumerated.identity);
        assert_eq!(x.proc_card, y.proc_card);
        assert_eq!(x.model, y.model);
        assert_eq!(x.diagram_set_digest, y.diagram_set_digest);
        assert_eq!(
            y.diagram_set_digest,
            DiagramArtifact::read_from_path(&diagrams)
                .unwrap()
                .0
                .diagram_set_digest,
            "an integration from an enumerate run records that run's digest"
        );
        assert_eq!(x.amplitude_probe, y.amplitude_probe);
        assert_eq!(x.amplitude_probe.len(), x.sets.len());
        assert_eq!(x.sigma_pb.to_bits(), y.sigma_pb.to_bits());
        assert_eq!(x.channels.len(), y.channels.len());
        assert!(x.channels.len() > 2);
        assert_eq!(raw(&grid_a), raw(&grid_b), "{tag}: the artifacts differ");
    }
}

/// `--ufo-dir` beside a diagram artifact is refused, naming why; a proc card on
/// stdin is still read as a proc card.
#[test]
fn ufo_dir_with_a_diagram_artifact_is_refused_and_stdin_is_a_proc_card() {
    let tmp = tempfile::tempdir().unwrap();
    let diagrams = sm_diagram_artifact(tmp.path());
    let refused = integrate(
        tmp.path(),
        &diagrams,
        &tmp.path().join("out"),
        &["--ufo-dir", toy_ufo_dir().to_str().unwrap()],
    );
    assert!(!refused.status.success());
    let stderr = stderr_of(&refused);
    assert!(
        stderr.contains("--ufo-dir") && stderr.contains("embeds the restricted model"),
        "{stderr}"
    );
    assert!(!tmp.path().join("out").join("grid.bin.zst").exists());

    let run_card = write(&tmp.path().join("run_card.dat"), &fixed_beams(250.0, 250.0));
    let mut child = common::vibegraph(tmp.path(), tmp.path())
        .arg("--no-network")
        .arg("integrate")
        .arg("-")
        .arg("--run-card")
        .arg(&run_card)
        .arg("--out")
        .arg(tmp.path().join("stdin"))
        .args(["--fixed-budget", "--neval", "2000", "--niter", "2"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    {
        use std::io::Write;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(SM_CARD.as_bytes())
            .unwrap();
    }
    let output = child.wait_with_output().unwrap();
    assert_ok(&output, "integrate from stdin");
    let artifact =
        IntegrateArtifact::read_from_path(&tmp.path().join("stdin").join("grid.bin.zst")).unwrap();
    assert_eq!(artifact.proc_card, SM_CARD);
}

/// A toy-UFO integration, with a param card, replays with the UFO directory
/// deleted and no `--ufo-dir` or `$VIBEGRAPH_UFO_DIR`: the model comes from the
/// artifact's embedded bytes, the param card's text is banked, and nothing is
/// enumerated. `generate --ufo-dir` is refused.
///
/// The positive control is the integration itself, which reads the directory
/// and enumerates. Cannot detect a UFO directory read from a location this test
/// does not delete (the cache under `VIBEGRAPH_HOME` is a fresh temp directory,
/// and the working directory holds no model).
#[test]
fn a_toy_ufo_artifact_generates_with_no_ufo_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let models = tmp.path().join("models");
    let model_dir = models.join(TOY_MODEL);
    std::fs::create_dir_all(&model_dir).unwrap();
    for entry in std::fs::read_dir(toy_ufo_dir().join(TOY_MODEL)).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            std::fs::copy(entry.path(), model_dir.join(entry.file_name())).unwrap();
        }
    }
    let card = write(&tmp.path().join("proc_card.dat"), TOY_CARD);
    let run_card = write(&tmp.path().join("run_card.dat"), &fixed_beams(500.0, 500.0));
    let param_text = "Block TOYCOUP\n    1 0.5 # gl\n";
    let param_card = write(&tmp.path().join("param_card.dat"), param_text);
    let out = tmp.path().join("out");
    let integrated = integrate(
        tmp.path(),
        &card,
        &out,
        &[
            "--ufo-dir",
            models.to_str().unwrap(),
            "--run-card",
            run_card.to_str().unwrap(),
            "--param-card",
            param_card.to_str().unwrap(),
        ],
    );
    assert_ok(&integrated, "integrate the toy model");
    assert!(stderr_of(&integrated).contains(ENUMERATED));
    let grid = out.join("grid.bin.zst");
    let artifact = IntegrateArtifact::read_from_path(&grid).unwrap();
    assert_eq!(artifact.param_card.as_deref(), Some(param_text));
    assert_eq!(artifact.embedded.naming.model_name, TOY_MODEL);
    assert_eq!(artifact.inputs.params.len(), 1);

    std::fs::remove_dir_all(&models).unwrap();
    let output = generate(tmp.path(), &grid, "events.lhe", &[]);
    assert_ok(&output, "generate with the UFO directory deleted");
    let stderr = stderr_of(&output);
    assert!(
        !stderr.contains(ENUMERATED),
        "generate enumerated:\n{stderr}"
    );
    assert!(stderr.contains("amplitude probe"), "{stderr}");
    let events = std::fs::read_to_string(tmp.path().join("events.lhe")).unwrap();
    assert_eq!(events.matches("<event>").count(), 20);

    let refused = generate(
        tmp.path(),
        &grid,
        "refused.lhe",
        &["--ufo-dir", toy_ufo_dir().to_str().unwrap()],
    );
    assert!(!refused.status.success());
    assert!(
        stderr_of(&refused).contains("generate reads no UFO directory"),
        "{}",
        stderr_of(&refused)
    );
    assert!(!tmp.path().join("refused.lhe").exists());
}

/// `generate` on an SM artifact recomputes the probe and enumerates nothing. A
/// copy whose one banked |M|² was moved by a relative 1e-8 is refused, naming
/// the set, the point and the remedy; one moved by 1e-13 is accepted.
#[test]
fn generate_refuses_an_artifact_whose_banked_amplitude_moved() {
    let tmp = tempfile::tempdir().unwrap();
    let card = write(&tmp.path().join("proc_card.dat"), SM_CARD);
    let run_card = write(&tmp.path().join("run_card.dat"), &fixed_beams(250.0, 250.0));
    let out = tmp.path().join("out");
    let integrated = integrate(
        tmp.path(),
        &card,
        &out,
        &["--run-card", run_card.to_str().unwrap()],
    );
    assert_ok(&integrated, "integrate");
    let grid = out.join("grid.bin.zst");

    let clean = generate(tmp.path(), &grid, "clean.lhe", &[]);
    assert_ok(&clean, "generate from the untouched artifact");
    assert!(
        !stderr_of(&clean).contains(ENUMERATED),
        "generate enumerated:\n{}",
        stderr_of(&clean)
    );

    let moved = |factor: f64, name: &str| {
        let mut artifact = IntegrateArtifact::read_from_path(&grid).unwrap();
        artifact.amplitude_probe[1].points[3].m2 *= factor;
        let path = tmp.path().join(name);
        artifact.write_to_path(&path, true).unwrap();
        path
    };
    let tiny = generate(
        tmp.path(),
        &moved(1.0 + 1e-13, "tiny.bin.zst"),
        "tiny.lhe",
        &[],
    );
    assert_ok(&tiny, "generate with a 1e-13 move");

    let refused = generate(
        tmp.path(),
        &moved(1.0 + 1e-8, "moved.bin.zst"),
        "moved.lhe",
        &[],
    );
    assert!(!refused.status.success(), "a 1e-8 move was accepted");
    let stderr = stderr_of(&refused);
    for needle in [
        "set 1 (e+ e- > W+{1,-1} W-)",
        "point 3",
        "relative deviation of",
        "integrate again",
    ] {
        assert!(stderr.contains(needle), "missing {needle:?}:\n{stderr}");
    }
    assert!(!tmp.path().join("moved.lhe").exists());
}
