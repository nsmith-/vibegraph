//! Artifact provenance on the `--output-ldjson` stream: the integrate and
//! inspect results name the enumeration version of the artifact they report,
//! and every `artifact` refusal carries a stable `reason`.
//!
//! The refusing artifacts are built by editing the bytes of a real artifact
//! (decompressed, patched at a known field, recompressed), so no old binary is
//! needed. Needs only the interned SM model.

mod common;

use std::path::{Path, PathBuf};
use std::process::Output;

use serde_json::{json, Value};
use vibegraph::artifact::{IntegrateArtifact, FORMAT_VERSION};
use vibegraph::diagrams::artifact::ENUMERATION_VERSION;

const PROCESS: &str = "e+ e- > mu+ mu-";
const EBEAM: f64 = 45.6;

struct Fixture {
    _tmp: tempfile::TempDir,
    dir: PathBuf,
    artifact_path: PathBuf,
    artifact: IntegrateArtifact,
    integrate: Value,
}

fn last_object(output: &Output) -> Value {
    let stdout = common::stdout_of(output);
    let line = stdout.lines().last().expect("a final line");
    serde_json::from_str(line).unwrap_or_else(|e| panic!("{e}: {line}"))
}

fn fixture() -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().to_path_buf();
    let proc_card = dir.join("proc_card.dat");
    let run_card = dir.join("run_card.dat");
    std::fs::write(&proc_card, format!("import model sm\ngenerate {PROCESS}\n")).unwrap();
    std::fs::write(&run_card, common::fixed_beams(EBEAM, EBEAM)).unwrap();
    let out = dir.join("out");
    let output = common::vibegraph(&dir, &dir)
        .arg("integrate")
        .arg(&proc_card)
        .arg("--run-card")
        .arg(&run_card)
        .arg("--out")
        .arg(&out)
        .args(["--neval", "2000", "--niter", "2", "--output-ldjson"])
        .output()
        .expect("spawn vibegraph");
    assert!(output.status.success(), "{}", common::stdout_of(&output));
    let artifact_path = out.join("grid.bin.zst");
    let artifact = IntegrateArtifact::read_from_path(&artifact_path).unwrap();
    Fixture {
        integrate: last_object(&output),
        _tmp: tmp,
        dir,
        artifact_path,
        artifact,
    }
}

fn inspect(f: &Fixture, path: &Path) -> Value {
    let output = common::vibegraph(&f.dir, &f.dir)
        .arg("inspect")
        .arg(path)
        .arg("--output-ldjson")
        .output()
        .expect("spawn vibegraph");
    last_object(&output)
}

fn generate(f: &Fixture, path: &Path) -> Value {
    let output = common::vibegraph(&f.dir, &f.dir)
        .arg("generate")
        .arg(path)
        .args(["--seed", "1", "--nevents", "10", "--output-ldjson", "-o"])
        .arg(f.dir.join("events.lhe"))
        .arg("--force")
        .output()
        .expect("spawn vibegraph");
    last_object(&output)
}

/// A copy of the fixture's artifact whose decompressed bytes `edit` has changed.
fn edited(f: &Fixture, name: &str, edit: impl FnOnce(&mut Vec<u8>)) -> PathBuf {
    let compressed = std::fs::read(&f.artifact_path).unwrap();
    let mut raw = zstd::decode_all(compressed.as_slice()).unwrap();
    edit(&mut raw);
    let path = f.dir.join(name);
    std::fs::write(&path, zstd::encode_all(raw.as_slice(), 3).unwrap()).unwrap();
    path
}

/// The first place `needle` occurs in `haystack`, which must be there.
fn find(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .position(|w| w == needle)
        .expect("the field is in the artifact")
}

fn reason_of(refusal: &Value) -> &Value {
    assert_eq!(refusal["kind"], "refusal", "{refusal}");
    assert_eq!(refusal["error_kind"], "artifact", "{refusal}");
    &refusal["reason"]
}

/// The result objects of `integrate` and `inspect` carry the artifact's
/// enumeration version; `integrate` names its own release as the writer, and
/// `inspect`, which cannot know the writer, says so.
#[test]
fn results_carry_the_enumeration_version_and_the_writer() {
    let f = fixture();
    assert_eq!(f.artifact.enumeration_version, ENUMERATION_VERSION);
    assert_eq!(
        f.integrate["enumeration_version"],
        json!(ENUMERATION_VERSION)
    );
    assert_eq!(
        f.integrate["vibegraph_version"],
        json!(env!("VIBEGRAPH_VERSION"))
    );
    let version_line = common::vibegraph(&f.dir, &f.dir)
        .arg("--version")
        .output()
        .unwrap();
    assert_eq!(
        common::stdout_of(&version_line).lines().next(),
        Some(format!("vibegraph {}", env!("VIBEGRAPH_VERSION")).as_str())
    );

    let inspected = inspect(&f, &f.artifact_path);
    assert_eq!(inspected["kind"], "result");
    assert_eq!(inspected["enumeration_version"], json!(ENUMERATION_VERSION));
    assert_eq!(inspected["format_version"], json!(FORMAT_VERSION));
    assert!(inspected["vibegraph_version"].is_null(), "{inspected}");
    assert_eq!(
        inspected["writer_version"],
        json!(env!("CARGO_PKG_VERSION"))
    );
}

/// An artifact whose diagrams a version-2 enumeration wrote is refused with
/// both versions, by `inspect` and by `generate`.
#[test]
fn an_enumeration_version_2_artifact_is_refused_by_reason() {
    let f = fixture();
    let old = edited(&f, "v2.bin.zst", |raw| {
        raw[4..8].copy_from_slice(&2u32.to_le_bytes());
    });
    let expected = json!({"kind": "enumeration_version", "found": 2,
                          "expected": ENUMERATION_VERSION});
    assert_eq!(reason_of(&inspect(&f, &old)), &expected);
    assert_eq!(reason_of(&generate(&f, &old)), &expected);
}

#[test]
fn an_unreadable_format_version_is_refused_by_reason() {
    let f = fixture();
    let newer = edited(&f, "f26.bin.zst", |raw| {
        raw[0..4].copy_from_slice(&(FORMAT_VERSION + 1).to_le_bytes());
    });
    let expected = json!({"kind": "format_version", "found": FORMAT_VERSION + 1,
                          "oldest": 24, "expected": FORMAT_VERSION});
    assert_eq!(reason_of(&inspect(&f, &newer)), &expected);
    assert_eq!(reason_of(&generate(&f, &newer)), &expected);
    let older = edited(&f, "f23.bin.zst", |raw| {
        raw[0..4].copy_from_slice(&23u32.to_le_bytes());
    });
    assert_eq!(
        reason_of(&inspect(&f, &older)),
        &json!({"kind": "format_version", "found": 23, "oldest": 24,
                "expected": FORMAT_VERSION})
    );
}

/// A new artifact is format 25 and records the release that wrote it, which
/// `integrate` and `inspect` both report.
#[test]
fn a_new_artifact_records_its_writer() {
    let f = fixture();
    assert_eq!(FORMAT_VERSION, 25);
    assert_eq!(f.artifact.format_version, 25);
    assert_eq!(
        f.artifact.writer_version.as_deref(),
        Some(env!("CARGO_PKG_VERSION"))
    );
    for result in [f.integrate.clone(), inspect(&f, &f.artifact_path)] {
        assert_eq!(result["format_version"], json!(25));
        assert_eq!(result["writer_version"], json!(env!("CARGO_PKG_VERSION")));
        assert_eq!(result["enumeration_version"], json!(ENUMERATION_VERSION));
    }
}

/// A format-24 artifact, made by removing the writer field from a fresh one and
/// stamping the old version, loads as before: the same cross section, no writer,
/// and `generate` reads it.
#[test]
fn a_format_24_artifact_loads_unchanged_with_no_writer() {
    let f = fixture();
    let writer = f.artifact.writer_version.clone().unwrap();
    let old = edited(&f, "f24.bin.zst", |raw| {
        raw.truncate(raw.len() - (1 + 8 + writer.len()));
        raw[0..4].copy_from_slice(&24u32.to_le_bytes());
    });
    let loaded = IntegrateArtifact::read_from_path(&old).unwrap();
    assert_eq!(loaded.format_version, 24);
    assert_eq!(loaded.writer_version, None);
    assert_eq!(loaded.sigma_pb.to_bits(), f.artifact.sigma_pb.to_bits());
    assert_eq!(loaded.process, f.artifact.process);

    let inspected = inspect(&f, &old);
    assert_eq!(inspected["kind"], "result", "{inspected}");
    assert_eq!(inspected["format_version"], json!(24));
    assert!(inspected["writer_version"].is_null(), "{inspected}");
    assert_eq!(inspected["enumeration_version"], json!(ENUMERATION_VERSION));
    assert_eq!(inspected["sigma"], f.integrate["sigma"]);
    assert_eq!(generate(&f, &old)["kind"], "result");
}

/// An edited digest is a corrupt or edited file, and which digest it was is in
/// the reason.
#[test]
fn an_edited_digest_is_refused_by_reason() {
    let f = fixture();
    let flip = |raw: &mut Vec<u8>, digest: &str| {
        let at = find(raw, digest.as_bytes());
        raw[at] = if raw[at] == b'0' { b'1' } else { b'0' };
    };
    let model = f.artifact.model.digest.clone();
    let bad_model = edited(&f, "model.bin.zst", |raw| flip(raw, &model));
    let reason = inspect(&f, &bad_model);
    let reason = reason_of(&reason);
    assert_eq!(reason["kind"], "model_digest", "{reason}");
    assert_eq!(reason["embedded"], json!(model));
    assert_ne!(reason["recorded"], json!(model));

    let sets = f.artifact.diagram_set_digest.clone();
    let bad_sets = edited(&f, "sets.bin.zst", |raw| flip(raw, &sets));
    let reason = generate(&f, &bad_sets);
    let reason = reason_of(&reason);
    assert_eq!(reason["kind"], "diagram_set_digest", "{reason}");
    assert_eq!(reason["computed"], json!(sets));
    assert_ne!(reason["recorded"], json!(sets));
}

/// A file that is not an artifact is `unreadable`; a refusal of another kind
/// has no `reason`.
#[test]
fn other_refusals_name_unreadable_or_carry_no_reason() {
    let f = fixture();
    let junk = f.dir.join("junk.bin.zst");
    std::fs::write(&junk, b"not zstd").unwrap();
    assert_eq!(
        reason_of(&inspect(&f, &junk)),
        &json!({"kind": "unreadable"})
    );

    let usage = common::vibegraph(&f.dir, &f.dir)
        .arg("generate")
        .arg(&f.artifact_path)
        .arg("--output-ldjson")
        .output()
        .unwrap();
    let usage = last_object(&usage);
    assert_eq!(usage["error_kind"], "usage");
    assert!(usage.get("reason").is_none(), "{usage}");
}
