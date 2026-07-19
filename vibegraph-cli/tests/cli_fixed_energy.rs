//! End-to-end test of `vibegraph integrate` on fixed-energy (`lpp = 0`) proc
//! cards driven through the flat-RAMBO phase-space path.
//!
//! Unlike the hadronic `cli_integrate` gate, these need no PDF set and no banked
//! reference data — only the interned SM model — so they run in the default test
//! suite. They demonstrate the two generalizations of the CLI: `lpp = 0` beams
//! (no PDF convolution, √ŝ = ebeam1 + ebeam2) and n-body final states through
//! RAMBO. The σ checks are smoke-level (finite, positive); pinning σ statistically
//! against banked MadGraph values is left to a dedicated σ gate.

use std::process::Command;

use vibegraph::artifact::IntegrateArtifact;

/// Drive `vibegraph integrate` on a fixed-energy proc/run card pair written to a
/// tempdir, with symmetric beams of energy `ebeam` (√ŝ = 2·ebeam). `expected_ndim`
/// pins the `4n` RAMBO VEGAS dimensionality baked into the persisted grid.
fn run_fixed_energy(process: &str, ebeam: f64, expected_ndim: usize) -> IntegrateArtifact {
    let tmp = tempfile::tempdir().unwrap();
    let proc_path = tmp.path().join("proc_card.dat");
    let run_path = tmp.path().join("run_card.dat");
    std::fs::write(&proc_path, format!("import model sm\ngenerate {process}\n")).unwrap();
    // Fixed-energy partonic beams; MG default cuts otherwise.
    std::fs::write(
        &run_path,
        format!("  0 = lpp1\n  0 = lpp2\n  {ebeam} = ebeam1\n  {ebeam} = ebeam2\n"),
    )
    .unwrap();
    let sqrt_s = 2.0 * ebeam;

    let out = tmp.path().join("out");
    let status = Command::new(env!("CARGO_BIN_EXE_vibegraph"))
        .arg("integrate")
        .arg(&proc_path)
        .arg("--run-card")
        .arg(&run_path)
        .arg("--out")
        .arg(&out)
        .arg("--neval")
        .arg("20000")
        .arg("--niter")
        .arg("4")
        .status()
        .expect("spawn vibegraph");
    assert!(status.success(), "vibegraph integrate exited non-zero");

    let artifact =
        IntegrateArtifact::read_from_path(&out.join("grid.bin.zst")).expect("reload artifact");
    assert_eq!(artifact.process, process);
    assert_eq!(artifact.pdf_set, "none");
    assert_eq!(artifact.sqrt_s_had, sqrt_s);
    assert_eq!(artifact.grid.ndim(), expected_ndim);
    assert!(
        artifact.sigma_pb.is_finite() && artifact.sigma_pb > 0.0,
        "[{process}] σ = {} pb is not finite-positive",
        artifact.sigma_pb
    );
    assert!(
        artifact.sigma_err_pb.is_finite() && artifact.sigma_err_pb >= 0.0,
        "[{process}] Δσ = {} pb is not finite",
        artifact.sigma_err_pb
    );
    eprintln!(
        "[{process}] fixed-energy σ = {:.4e} ± {:.2e} pb (RAMBO ndim {expected_ndim})",
        artifact.sigma_pb, artifact.sigma_err_pb
    );
    artifact
}

/// 2→2 fixed-energy final state through flat RAMBO (n = 2 → 8 VEGAS dims). At
/// √ŝ = 500 GeV this is the MG-validated `ee_to_ttx` point (σ_MG ≈ 0.549 pb);
/// flat RAMBO on this non-resonant 2→2 tracks it closely.
#[test]
fn fixed_energy_2to2_finite_sigma() {
    let a = run_fixed_energy("e+ e- > t t~", 250.0, 8);
    // A smooth 2→2: flat RAMBO lands in the ballpark of the banked MG σ.
    assert!(
        (0.3..0.9).contains(&a.sigma_pb),
        "e+e- > t t~ σ = {} pb outside the plausible band",
        a.sigma_pb
    );
}

/// n-body (2→3) fixed-energy final state through flat RAMBO (n = 3 → 12 dims).
/// Run below the Z pole (√ŝ = 200 GeV, so m_ττ < √ŝ − m_H ≈ 75 GeV) to keep the
/// integrand off the sharp Z resonance that flat RAMBO samples poorly.
#[test]
fn fixed_energy_nbody_finite_sigma() {
    run_fixed_energy("e+ e- > ta+ ta- H", 100.0, 12);
}
