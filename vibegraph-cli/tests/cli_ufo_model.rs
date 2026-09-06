//! The `--ufo-dir` path end to end, on a model that is not the Standard Model.
//!
//! `vibegraph integrate <card> --ufo-dir <dir>` is the only route by which a user
//! reaches a UFO model this binary does not carry, and the SMEFTsim model vendored
//! at `validation/ufo/` is the one the validation ladder gates. What is pinned here
//! is the part of that route no library test sees: that `import model <name>-<card>`
//! resolves the restrict-card suffix against the model directory, that the artifact
//! records *which* model it was built from, and that a second command holding the
//! same grids refuses a different variant of the same model.
//!
//! The cross section is a smoke check only. The number is gated statistically
//! against MadGraph's banked run by `validate_sigma.rs`'s `ee_to_ttx_smeft` row, at
//! a budget this test does not spend.
//!
//! Hermetic: the model is committed to this repository and no PDF set is needed
//! (`lpp = 0`).

use std::path::PathBuf;
use std::process::Command;

use vibegraph::artifact::IntegrateArtifact;
use vibegraph::ufo::UFOModel;

const MODEL: &str = "SMEFTsim_topU3l_MwScheme_UFO";
const PROCESS: &str = "e+ e- > t t~ NP<=1";

fn ufo_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../validation/ufo")
}

/// The digest `import model <MODEL>-<restrict>` must produce, read through the
/// library so the test cannot agree with the binary by sharing its mistake.
fn digest_of(restrict: &str) -> String {
    let dir = ufo_dir().join(MODEL);
    let card = dir.join(format!("restrict_{restrict}.dat"));
    assert!(card.is_file(), "no restrict card at {}", card.display());
    UFOModel::load_with_digest(&dir, Some(&card))
        .unwrap_or_else(|e| panic!("load {MODEL}-{restrict}: {e}"))
        .1
}

fn write_cards(tmp: &std::path::Path, restrict: &str) -> (PathBuf, PathBuf) {
    let proc_path = tmp.join(format!("proc_card_{restrict}.dat"));
    std::fs::write(
        &proc_path,
        format!("import model {MODEL}-{restrict}\ngenerate {PROCESS}\n"),
    )
    .unwrap();
    let run_path = tmp.join("run_card.dat");
    std::fs::write(
        &run_path,
        "  0 = lpp1\n  0 = lpp2\n  250.0 = ebeam1\n  250.0 = ebeam2\n",
    )
    .unwrap();
    (proc_path, run_path)
}

/// A SMEFT proc card runs end to end through `--ufo-dir`, and the artifact it
/// leaves names the model it was built from.
///
/// The suffix is what the assertions are about. This model ships two restrict
/// cards and no `restrict_default.dat`, so `-massless` and `-SMlimit_massless`
/// write the same `generate` line over the same directory and give different
/// physics: 36 diagrams at 2.22 pb against 2 diagrams at 0.55 pb here. The digest
/// is what separates them — it is taken over the restricted model, so the
/// `SMlimit_massless` comparison is what says it moves with the card and not
/// merely with the directory — and the σ band excludes the Standard-Model limit by
/// a factor of three, so a suffix that resolved to nothing could not pass either.
#[test]
fn a_smeft_model_integrates_through_ufo_dir_and_records_its_identity() {
    let tmp = tempfile::tempdir().unwrap();
    let (proc_path, run_path) = write_cards(tmp.path(), "massless");
    let out = tmp.path().join("out");

    let status = Command::new(env!("CARGO_BIN_EXE_vibegraph"))
        .arg("integrate")
        .arg(&proc_path)
        .args(["--ufo-dir".as_ref(), ufo_dir().as_os_str()])
        .arg("--run-card")
        .arg(&run_path)
        .arg("--out")
        .arg(&out)
        .arg("--fixed-budget")
        .args(["--neval", "10000", "--niter", "3"])
        .status()
        .expect("spawn vibegraph");
    assert!(status.success(), "vibegraph integrate exited {status}");

    let artifact =
        IntegrateArtifact::read_from_path(&out.join("grid.bin.zst")).expect("reload artifact");
    assert_eq!(artifact.process, PROCESS);
    assert_eq!(artifact.model.name, MODEL);
    assert_eq!(artifact.model.restrict, "massless");
    assert_eq!(artifact.model.label(), format!("{MODEL}-massless"));
    assert_eq!(
        artifact.model.digest,
        digest_of("massless"),
        "the artifact's digest is not the one the `-massless` card produces"
    );
    assert_ne!(
        digest_of("massless"),
        digest_of("SMlimit_massless"),
        "the two restrict cards of this model produce the same digest, so the \
         digest cannot see which card a run used"
    );
    // 36 diagrams, one channel each. A count that moved would mean the suffix
    // selected a different vertex set than the row the sigma gate measures.
    assert_eq!(artifact.channels.len(), 36);
    // Smoke only: `validate_sigma.rs` gates this against MadGraph's 2.2223 pb.
    assert!(
        (1.5..3.0).contains(&artifact.sigma_pb),
        "σ = {} pb is nowhere near the banked SMEFT cross section",
        artifact.sigma_pb
    );
}

/// `generate` refuses grids trained on a different restrict card of the same
/// model — the case the process string and the run card cannot see.
///
/// The refusal is what is tested, and the matching run above is its control: a
/// check that refused everything would also refuse this.
#[test]
fn generate_refuses_another_restrict_variant_of_the_same_model() {
    let tmp = tempfile::tempdir().unwrap();
    let (proc_path, run_path) = write_cards(tmp.path(), "massless");
    let out = tmp.path().join("out");
    let status = Command::new(env!("CARGO_BIN_EXE_vibegraph"))
        .arg("integrate")
        .arg(&proc_path)
        .args(["--ufo-dir".as_ref(), ufo_dir().as_os_str()])
        .arg("--run-card")
        .arg(&run_path)
        .arg("--out")
        .arg(&out)
        .arg("--fixed-budget")
        .args(["--neval", "10000", "--niter", "3"])
        .status()
        .expect("spawn vibegraph");
    assert!(status.success(), "vibegraph integrate exited {status}");

    let (sm_limit, _) = write_cards(tmp.path(), "SMlimit_massless");
    let refused = Command::new(env!("CARGO_BIN_EXE_vibegraph"))
        .arg("generate")
        .arg(out.join("grid.bin.zst"))
        .arg(&sm_limit)
        .args(["--ufo-dir".as_ref(), ufo_dir().as_os_str()])
        .arg("--run-card")
        .arg(&run_path)
        .args(["--nevents", "10"])
        .arg("-o")
        .arg(tmp.path().join("events.lhe"))
        .output()
        .expect("spawn vibegraph");
    assert!(
        !refused.status.success(),
        "generate accepted grids trained on a different restrict card"
    );
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(
        stderr.contains(&format!("{MODEL}-massless"))
            && stderr.contains(&format!("{MODEL}-SMlimit_massless")),
        "the refusal must name both variants: {stderr}"
    );
    assert!(
        !tmp.path().join("events.lhe").exists(),
        "a refused run wrote an event file anyway"
    );
}
