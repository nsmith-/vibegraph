//! How `generate` names and writes its event file: the encoding the extension
//! asks for, the refusal of a compression extension it cannot write, and the
//! file appearing under its name only once complete.
//!
//! Needs only the interned SM model, so it runs in the default suite. It reads
//! the files back with this crate's own reader and the `flate2`/`zstd` decoders,
//! so it says nothing about whether a shower accepts the bytes; the file's
//! content is under test elsewhere.

#![cfg(unix)]

mod common;

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Output, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

const NEVENTS: usize = 300;

struct Grid {
    _tmp: tempfile::TempDir,
    dir: PathBuf,
    artifact: PathBuf,
}

fn grid() -> &'static Grid {
    static GRID: OnceLock<Grid> = OnceLock::new();
    GRID.get_or_init(|| {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_path_buf();
        std::fs::write(
            dir.join("proc_card.dat"),
            "import model sm\ngenerate e+ e- > mu+ mu-\n",
        )
        .unwrap();
        std::fs::write(dir.join("run_card.dat"), common::fixed_beams(45.6, 45.6)).unwrap();
        let output = common::vibegraph(&dir, &dir)
            .args(["integrate", "proc_card.dat", "--run-card", "run_card.dat"])
            .args(["--out", "grid", "--fixed-budget", "--neval", "4000"])
            .args(["--niter", "4", "-j", "1"])
            .output()
            .expect("spawn vibegraph");
        assert!(output.status.success(), "{}", common::stderr_of(&output));
        let artifact = dir.join("grid/grid.bin.zst");
        Grid {
            _tmp: tmp,
            dir,
            artifact,
        }
    })
}

fn generate(out: &str, extra: &[&str]) -> Output {
    let grid = grid();
    common::vibegraph(&grid.dir, &grid.dir)
        .arg("generate")
        .arg(&grid.artifact)
        .args(["--seed", "7", "--nevents", &NEVENTS.to_string()])
        .args(["-j", "1", "--force", "-o"])
        .arg(grid.dir.join(out))
        .args(extra)
        .output()
        .expect("spawn vibegraph")
}

fn bytes(out: &str) -> Vec<u8> {
    std::fs::read(grid().dir.join(out)).expect("the event file")
}

/// A whole file: it opens the document, closes it, and holds every event.
fn assert_complete_lhe(text: &str) {
    assert!(text.starts_with("<LesHouchesEvents"), "{}", &text[..40]);
    assert!(text.trim_end().ends_with("</LesHouchesEvents>"));
    assert_eq!(text.matches("<event>").count(), NEVENTS);
}

#[test]
fn a_plain_name_writes_plain_text() {
    for name in ["plain.lhe", "cmsgrid_final.lhe", "noextension"] {
        let output = generate(name, &[]);
        assert!(output.status.success(), "{}", common::stderr_of(&output));
        let raw = bytes(name);
        assert!(raw.starts_with(b"<LesHouchesEvents"), "{name}");
        assert_complete_lhe(&String::from_utf8(raw).unwrap());
    }
}

#[test]
fn a_gz_name_writes_gzip() {
    let output = generate("ev.lhe.gz", &[]);
    assert!(output.status.success(), "{}", common::stderr_of(&output));
    let raw = bytes("ev.lhe.gz");
    assert_eq!(raw[..2], [0x1f, 0x8b], "gzip magic");
    let mut text = String::new();
    flate2::read::GzDecoder::new(&raw[..])
        .read_to_string(&mut text)
        .expect("a complete gzip stream");
    assert_complete_lhe(&text);
}

#[test]
fn a_zst_name_writes_zstd() {
    let output = generate("ev.lhe.zst", &[]);
    assert!(output.status.success(), "{}", common::stderr_of(&output));
    let raw = bytes("ev.lhe.zst");
    assert_eq!(raw[..4], [0x28, 0xb5, 0x2f, 0xfd], "zstd magic");
    let text = String::from_utf8(zstd::decode_all(&raw[..]).expect("a complete zstd frame"))
        .expect("text");
    assert_complete_lhe(&text);
}

#[test]
fn a_compression_extension_it_cannot_write_is_refused_by_name() {
    for name in ["ev.lhe.xz", "ev.lhe.bz2"] {
        let output = generate(name, &[]);
        assert!(!output.status.success(), "{name}");
        let stderr = common::stderr_of(&output);
        assert!(
            stderr.contains("does not write") && stderr.contains(".gz"),
            "{stderr}"
        );
        assert!(!grid().dir.join(name).exists(), "{name} was written");
    }
}

#[test]
fn a_finished_run_leaves_only_the_named_file() {
    let output = generate("only.lhe", &[]);
    assert!(output.status.success());
    let leftovers: Vec<String> = std::fs::read_dir(&grid().dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with(".only.lhe.") && n.ends_with(".partial"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

/// A run killed while it is writing leaves no file under the requested name:
/// what it had written is under a hidden temporary one.
#[test]
fn a_killed_run_leaves_nothing_under_the_final_name() {
    let grid = grid();
    let target = "killed.lhe";
    let mut child = common::vibegraph(&grid.dir, &grid.dir)
        .arg("generate")
        .arg(&grid.artifact)
        .args(["--seed", "7", "--nevents", "5000000"])
        .args(["--strategy", "stochastic-rounding", "-j", "1", "-o", target])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn vibegraph");
    let partial = wait_for_partial(&grid.dir, target);
    child.kill().expect("SIGKILL");
    let _ = child.wait();
    assert!(
        std::fs::metadata(&partial).unwrap().len() > 0,
        "killed before anything was written"
    );
    assert!(!grid.dir.join(target).exists(), "{target} exists");
    std::fs::remove_file(partial).unwrap();
}

/// The hidden temporary file for `target`, once it holds some output.
fn wait_for_partial(dir: &Path, target: &str) -> PathBuf {
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        assert!(Instant::now() < deadline, "no partial file appeared");
        let found = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap())
            .find(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                name.starts_with(&format!(".{target}.")) && name.ends_with(".partial")
            });
        if let Some(entry) = found {
            if entry.metadata().map(|m| m.len()).unwrap_or(0) > 50_000 {
                return entry.path();
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
