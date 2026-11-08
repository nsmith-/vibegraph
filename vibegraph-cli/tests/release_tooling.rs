//! The version string must come from a `v*` tag even when a newer tag of
//! another kind (the reference-data bundles) is reachable.

#[path = "../build_support/describe.rs"]
mod describe;

use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(dir)
        .args(["-c", "user.name=t", "-c", "user.email=t@example.org"])
        .args(args)
        .status()
        .expect("git runs");
    assert!(status.success(), "git {args:?} failed");
}

fn commit(dir: &Path, n: u32) {
    std::fs::write(dir.join("f"), n.to_string()).unwrap();
    git(dir, &["add", "f"]);
    git(dir, &["commit", "-q", "-m", &format!("c{n}")]);
}

#[test]
fn a_newer_reference_data_tag_does_not_name_the_version() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    git(dir, &["init", "-q"]);
    commit(dir, 1);
    git(dir, &["tag", "v0.1.0"]);
    commit(dir, 2);
    commit(dir, 3);
    git(dir, &["tag", "-a", "-m", "data", "refdata-9"]);

    let described = describe::git_describe(Some(dir)).expect("describes");
    assert!(
        described.starts_with("v0.1.0-2-g"),
        "expected the v* tag, got {described}"
    );
}

#[test]
fn without_any_release_tag_the_hash_is_reported() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    git(dir, &["init", "-q"]);
    commit(dir, 1);
    git(dir, &["tag", "refdata-9"]);

    let described = describe::git_describe(Some(dir)).expect("describes");
    assert!(
        !described.contains("refdata") && !described.starts_with('v'),
        "expected a bare hash, got {described}"
    );
}

#[test]
fn a_release_candidate_tag_is_a_version_tag() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    git(dir, &["init", "-q"]);
    commit(dir, 1);
    git(dir, &["tag", "v0.2.0-rc1"]);

    assert_eq!(
        describe::git_describe(Some(dir)).as_deref(),
        Some("v0.2.0-rc1")
    );
}

#[test]
fn release_check_accepts_and_refuses_the_expected_inputs() {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../scripts/test-release-check.sh");
    let output = Command::new("bash")
        .arg(&script)
        .output()
        .expect("bash runs");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
