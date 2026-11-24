//! `vibegraph generate` on a card with a flavour group whose matrix element
//! vanishes at the card and whose members carry an `s`-channel quark line of
//! different flavours.
//!
//! The card is the Standard-Model image of g2HDM's `g u > h+ d` with
//! `g d~ > h+ u~` where H± has no `u d` coupling: under `restrict_ckm` with the
//! Cabibbo angle at zero, `g u > w+ s` and `g s~ > w+ u~` vanish, form one group
//! on `0 = 0`, and carry an `s`-channel `u` in one member and `s~` in the other.
//! `g u > w+ d` beside them carries the cross section.
//!
//! Hermetic: the Standard Model is interned in the binary, and the PDF set is a
//! synthetic grid this file writes under the default set's name, so nothing
//! fetched is read. The densities are a smooth stand-in, not a fit: the test
//! reads whether events are written and what labels them, not how many of each.
//!
//! What it cannot see: the flavour of a line the record never writes. Every
//! line that differs between the two members here is massless, so neither this
//! crate nor MadEvent writes it as an intermediate; the codes such a line
//! resolves to are pinned in `generate`'s and `lhef::resonance`'s unit tests.

mod common;
use common::PdfDataPath;

use std::fmt::Write as _;
use std::path::Path;
use std::process::Output;

use vibegraph::cache::pinned::DEFAULT_PDF_SET;
use vibegraph::lhef::parse::LheFile;
use vibegraph::lhef::record::{STATUS_INCOMING, STATUS_INTERMEDIATE};

const PROC_CARD: &str = "import model sm-ckm
define q = u s~
define qq = s u~
generate g q > w+ qq, w+ > e+ ve
add process g u > w+ d, w+ > e+ ve
";

const PARAM_CARD: &str = "Block Wolfenstein
    1 0.000000e+00 # lamWS
";

const RUN_CARD: &str = "  1 = lpp1
  1 = lpp2
  6500.0 = ebeam1
  6500.0 = ebeam2
  lhapdf = pdlabel
  247000 = lhaid
  20.0 = ptj
  15.0 = bwcutoff
";

/// A one-member LHAPDF set under [`DEFAULT_PDF_SET`]'s name in `base`: every
/// parton's `x f(x, Q)` a positive power law in `x` and `1 - x`, the same at
/// every `Q`, and `αs` falling through its tabulated points.
fn synthetic_pdf_set(base: &Path) {
    let dir = base.join(DEFAULT_PDF_SET);
    std::fs::create_dir_all(&dir).unwrap();
    let flavors = [-5, -4, -3, -2, -1, 1, 2, 3, 4, 5, 21];
    let flavor_list = flavors.map(|f| f.to_string()).join(", ");
    let info = format!(
        "SetDesc: \"synthetic stand-in grid for a hermetic test\"\n\
         SetIndex: 247000\n\
         Format: lhagrid1\n\
         DataVersion: 1\n\
         NumMembers: 1\n\
         Particle: 2212\n\
         Flavors: [{flavor_list}]\n\
         OrderQCD: 0\n\
         FlavorScheme: variable\n\
         NumFlavors: 5\n\
         ErrorType: replicas\n\
         XMin: 1e-07\n\
         XMax: 1\n\
         QMin: 1\n\
         QMax: 100000\n\
         MZ: 91.1876\n\
         MUp: 0\n\
         MDown: 0\n\
         MStrange: 0\n\
         MCharm: 1.275\n\
         MBottom: 4.18\n\
         MTop: 173.07\n\
         AlphaS_MZ: 0.118\n\
         AlphaS_OrderQCD: 0\n\
         AlphaS_Type: ipol\n\
         AlphaS_Qs: [1.0, 3.0, 10.0, 30.0, 91.1876, 300.0, 1000.0, 3000.0, 10000.0, 100000.0]\n\
         AlphaS_Vals: [0.45, 0.25, 0.18, 0.145, 0.118, 0.101, 0.089, 0.080, 0.073, 0.062]\n"
    );
    std::fs::write(dir.join(format!("{DEFAULT_PDF_SET}.info")), info).unwrap();

    let x: Vec<f64> = (0..=28)
        .map(|k| {
            if k == 28 {
                1.0
            } else {
                10f64.powf(-7.0 + 7.0 * f64::from(k) / 28.0) * 0.999
            }
        })
        .collect();
    let q = [1.0, 10.0, 100.0, 1000.0, 10000.0, 100000.0];
    let xf = |flavor: i32, x: f64| -> f64 {
        let sea = 0.2 * x.powf(-0.1) * (1.0 - x).powi(7);
        match flavor {
            21 => 1.5 * x.powf(-0.1) * (1.0 - x).powi(5),
            2 => sea + 2.0 * x.sqrt() * (1.0 - x).powi(3),
            1 => sea + x.sqrt() * (1.0 - x).powi(4),
            f if f.abs() >= 4 => 0.3 * sea,
            _ => sea,
        }
    };
    let mut dat = String::from("PdfType: central\nFormat: lhagrid1\n---\n");
    let row = |values: &[f64]| {
        values
            .iter()
            .map(|v| format!("{v:.8e}"))
            .collect::<Vec<_>>()
            .join(" ")
    };
    writeln!(dat, "{}", row(&x)).unwrap();
    writeln!(dat, "{}", row(&q)).unwrap();
    writeln!(dat, "{}", flavors.map(|f| f.to_string()).join(" ")).unwrap();
    for &xi in &x {
        for _ in &q {
            let values: Vec<f64> = flavors.iter().map(|&f| xf(f, xi)).collect();
            writeln!(dat, "{}", row(&values)).unwrap();
        }
    }
    dat.push_str("---\n");
    std::fs::write(dir.join(format!("{DEFAULT_PDF_SET}_0000.dat")), dat).unwrap();
}

fn run(dir: &Path, pdf: &Path, args: &[&std::ffi::OsStr]) -> Output {
    common::vibegraph(dir, &dir.join("home"))
        .env("VIBEGRAPH_NO_NETWORK", "1")
        .pdf_data_path(pdf)
        .args(args)
        .output()
        .expect("spawn vibegraph")
}

/// The group that vanishes is set aside by `integrate` and never drawn, so
/// `generate` writes every event from the live group, with its `W` as the one
/// intermediate.
#[test]
fn a_vanishing_group_with_a_mixed_flavour_line_does_not_block_generation() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    let pdf = dir.join("pdf");
    synthetic_pdf_set(&pdf);
    for (name, text) in [
        ("proc_card.dat", PROC_CARD),
        ("param_card.dat", PARAM_CARD),
        ("run_card.dat", RUN_CARD),
    ] {
        std::fs::write(dir.join(name), text).unwrap();
    }

    let integrate = run(
        dir,
        &pdf,
        &[
            "integrate".as_ref(),
            "proc_card.dat".as_ref(),
            "--run-card".as_ref(),
            "run_card.dat".as_ref(),
            "--param-card".as_ref(),
            "param_card.dat".as_ref(),
            "--out".as_ref(),
            "run".as_ref(),
            "--seed".as_ref(),
            "1".as_ref(),
            "--fixed-budget".as_ref(),
            "--neval".as_ref(),
            "4000".as_ref(),
            "--niter".as_ref(),
            "1".as_ref(),
        ],
    );
    let stderr = common::stderr_of(&integrate);
    assert!(integrate.status.success(), "integrate failed:\n{stderr}");
    assert!(
        stderr
            .contains("1 of 2 flavour groups (2 subprocesses) have a matrix element that vanishes"),
        "the card must hold one vanishing group of two subprocesses:\n{stderr}"
    );

    let events = dir.join("events.lhe");
    let generate = run(
        dir,
        &pdf,
        &[
            "generate".as_ref(),
            "run/grid.bin.zst".as_ref(),
            "--seed".as_ref(),
            "2".as_ref(),
            "--nevents".as_ref(),
            "50".as_ref(),
            "--out".as_ref(),
            events.as_os_str(),
        ],
    );
    assert!(
        generate.status.success(),
        "generate refused a card whose only unresolvable line is in a group that \
         vanishes:\n{}",
        common::stderr_of(&generate)
    );

    let file = LheFile::parse(&std::fs::read_to_string(&events).unwrap()).unwrap();
    assert_eq!(file.events.len(), 50);
    for event in &file.events {
        let mut incoming: Vec<i32> = event
            .particles
            .iter()
            .filter(|p| p.status == STATUS_INCOMING)
            .map(|p| p.pdg)
            .collect();
        incoming.sort_unstable();
        assert_eq!(incoming, [2, 21], "only g u > w+ d carries weight");
        let intermediates: Vec<i32> = event
            .particles
            .iter()
            .filter(|p| p.status == STATUS_INTERMEDIATE)
            .map(|p| p.pdg)
            .collect();
        assert_eq!(intermediates, [24]);
    }
}
