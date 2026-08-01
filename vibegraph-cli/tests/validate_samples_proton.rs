//! The `samples` category at proton beams: the shipped binary's own unweighted
//! events against MadGraph's banked ones.
//!
//! The fixed-beam rows are compared in `validate_samples` inside the library. A
//! hadron-collider row cannot be: the event a flavour group emits is assembled by
//! the `generate` command — the flavour drawn from the parton luminosities, the
//! record relabelled onto that flavour, both beam orderings — and that assembly
//! lives in the binary. So this row runs the binary: one `integrate`, then one
//! `generate` per seed off the frozen grids, and the emitted `.lhe` is read back
//! and compared through the same
//! [`validation::samples`](vibegraph::validation::samples) machinery the library
//! rows use.
//!
//! # What this adds over the fixed-beam rows
//!
//! The **flavour-group frequencies**. Which subprocess an event is labelled with
//! is drawn `∝ luminosity × σ̂`, a rule `cli_generate_proton` checks only for
//! *admissibility* — every emitted flavour assignment is one MadGraph's
//! `leshouche.inc` lists — and never for *frequency*. The χ² homogeneity on the
//! flavour column is the first comparison of the realised populations against
//! MadGraph's own sample.
//!
//! # What it provably cannot detect
//!
//! Everything the library rows cannot (normalisation, correlations between
//! columns, a discrepancy confined to a small tail), plus one of its own: the
//! comparison is against a *single* banked MadGraph run of 10 000 events, so a
//! flavour carrying well under a per-mille of the cross section is pooled into the
//! χ²'s residual category rather than compared on its own.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use flate2::read::MultiGzDecoder;
use vibegraph::lhef::parse::LheFile;
use vibegraph::validation::samples::{compare, labelling_for, EventSample};

#[path = "../../vibegraph-lib/tests/common/report.rs"]
mod report;

use report::{CategoryCount, Chi2Cell, KsCell, SamplesRow, SeedSample};

/// The banked run this replays: MadGraph's own cards, its own PDF set and its own
/// event sample.
const RUN: &str = "pp_to_llj_fixed";
const PROCESS: &str = "p p > l+ l- j QCD=2 QED=2";
const PDF_SET: &str = "NNPDF23_lo_as_0130_qed";

/// Integration budget, the one `cli_generate_proton` measured the sample's own
/// cross section to be converged at.
const NEVAL: &str = "300000";
const NITER: &str = "8";
const INTEGRATION_SEED: &str = "20260731";
/// Events per generation seed, against MadGraph's banked 10 000.
const NEVENTS: usize = 20_000;
/// Independent generation seeds, replayed off the one set of frozen grids.
const GEN_SEEDS: [u64; 3] = [0x5A_4D_1001, 0x5A_4D_1002, 0x5A_4D_1003];

/// The p-value a column must clear, the same floor the library rows use and
/// chosen there from the same kind of trial count.
const P_FLOOR: f64 = 1e-4;

fn output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/output")
}

fn pdf_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/pdf")
}

fn run_dir() -> PathBuf {
    output_dir().join(RUN)
}

/// MadGraph's banked events for this run.
fn banked_sample() -> EventSample {
    let path = run_dir().join("Events/run_01/unweighted_events.lhe.gz");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let mut text = String::new();
    // MadGraph concatenates the per-channel event files, so a banked `.lhe.gz` is
    // a *multi-member* gzip stream and a single-member reader stops silently at
    // the first member's end — with a perfectly parseable prefix of the run.
    MultiGzDecoder::new(&bytes[..])
        .read_to_string(&mut text)
        .unwrap_or_else(|e| panic!("decompress {}: {e}", path.display()));
    assert!(
        text.trim_end().ends_with("</LesHouchesEvents>"),
        "{} decompressed to a truncated document",
        path.display()
    );
    EventSample::from_lhe(LheFile::parse(&text).expect("MadGraph's own file parses"))
}

#[test]
fn generated_proton_events_agree_with_madgraphs_banked_ones() {
    let mg = banked_sample();
    let tmp = tempfile::tempdir().expect("temporary directory");
    let dir = tmp.path();
    let proc_card = dir.join("proc_card.dat");
    std::fs::write(&proc_card, format!("import model sm\ngenerate {PROCESS}\n"))
        .expect("write the proc card");
    // MadGraph's own card, byte for byte, so both generators read one file.
    let run_card = run_dir().join("Cards/run_card.dat");

    let out = dir.join("out");
    let integrate = Command::new(env!("CARGO_BIN_EXE_vibegraph"))
        .arg("integrate")
        .arg(&proc_card)
        .arg("--run-card")
        .arg(&run_card)
        .arg("--out")
        .arg(&out)
        .arg("--pdf-dir")
        .arg(pdf_dir())
        .args([
            "--neval",
            NEVAL,
            "--niter",
            NITER,
            "--seed",
            INTEGRATION_SEED,
        ])
        .output()
        .expect("spawn vibegraph");
    assert!(
        integrate.status.success(),
        "vibegraph integrate failed:\n{}",
        String::from_utf8_lossy(&integrate.stderr)
    );
    let artifact = out.join("grid.bin.zst");

    eprintln!(
        "-- {RUN} ({} banked events, sigma {:.4} pb, PDF set {PDF_SET}) --",
        mg.len(),
        mg.sigma_pb
    );

    let mut row = SamplesRow::new(RUN, PROCESS, "gate");
    row.p_floor = P_FLOOR;
    row.mg_events = mg.len();
    row.sigma_mg_pb = mg.sigma_pb;
    row.labelling = "coarse";
    let mut failures: Vec<String> = Vec::new();

    for &seed in &GEN_SEEDS {
        let file = dir.join(format!("events_{seed:#x}.lhe"));
        let generate = Command::new(env!("CARGO_BIN_EXE_vibegraph"))
            .arg("generate")
            .arg(&artifact)
            .arg(&proc_card)
            .arg("--run-card")
            .arg(&run_card)
            .arg("--pdf-dir")
            .arg(pdf_dir())
            .arg("--seed")
            .arg(seed.to_string())
            .arg("--nevents")
            .arg(NEVENTS.to_string())
            .arg("-o")
            .arg(&file)
            .arg("--force")
            .output()
            .expect("spawn vibegraph");
        assert!(
            generate.status.success(),
            "vibegraph generate failed:\n{}",
            String::from_utf8_lossy(&generate.stderr)
        );
        let text = std::fs::read_to_string(&file).expect("read the emitted event file");
        let ours = EventSample::from_lhe(LheFile::parse(&text).expect("our own file parses"));
        assert_eq!(
            ours.len(),
            NEVENTS,
            "the generator emitted {} of {NEVENTS} events",
            ours.len()
        );

        let labelling = labelling_for(&ours, &mg);
        let found = compare(&ours, &mg, labelling);
        let worst = found.worst_ks().expect("a comparable observable");
        eprintln!(
            "  seed {seed:#010x} | {} events (n_eff {:.0}) | worst KS: {} p {:.3e} (D {:.4})",
            ours.len(),
            ours.effective_size(),
            worst.observable,
            worst.p,
            worst.d,
        );
        for cell in &found.chi2 {
            eprintln!(
                "             chi2 {:<8} p {:.3e} (chi2 {:.1} / {} dof over {} of {} categories, \
                 {:.1}% pooled)",
                cell.column,
                cell.p,
                cell.chi2,
                cell.dof,
                cell.categories,
                cell.distinct_keys,
                100.0 * cell.pooled_share
            );
        }
        for cell in &found.ks {
            if cell.p < P_FLOOR {
                failures.push(format!(
                    "seed {seed:#010x} KS {} p {:.3e} (D {:.4}) below the {P_FLOOR:.0e} floor",
                    cell.observable, cell.p, cell.d
                ));
            }
        }
        for cell in &found.chi2 {
            if cell.p < P_FLOOR {
                failures.push(format!(
                    "seed {seed:#010x} chi2 {} p {:.3e} ({:.1}/{} dof) below the \
                     {P_FLOOR:.0e} floor",
                    cell.column, cell.p, cell.chi2, cell.dof
                ));
            }
        }

        row.constant_observables = found.constant.clone();
        row.single_category = found
            .single_category
            .iter()
            .map(|c| (*c).to_string())
            .collect();
        row.per_seed.push(SeedSample {
            seed,
            events: ours.len(),
            sigma_pb: ours.sigma_pb,
            ks: found
                .ks
                .iter()
                .map(|c| KsCell {
                    observable: c.observable.clone(),
                    d: c.d,
                    p: c.p,
                })
                .collect(),
            chi2: found
                .chi2
                .iter()
                .map(|c| Chi2Cell {
                    column: c.column.to_string(),
                    chi2: c.chi2,
                    dof: c.dof,
                    p: c.p,
                    categories: c.categories,
                    distinct_keys: c.distinct_keys,
                    pooled_share: c.pooled_share,
                    detail: c
                        .detail
                        .iter()
                        .map(|(key, ours, theirs)| CategoryCount {
                            key: key.clone(),
                            ours: *ours,
                            theirs: *theirs,
                        })
                        .collect(),
                })
                .collect(),
        });
    }

    row.finish();
    eprintln!(
        "  min KS p {:.3e} ({}), min chi2 p {:.3e} ({}) over {} seeds",
        row.min_ks_p,
        row.worst_ks_observable,
        row.min_chi2_p,
        row.worst_chi2_column,
        GEN_SEEDS.len()
    );
    row.status = if failures.is_empty() { "pass" } else { "fail" };
    row.write();

    assert!(failures.is_empty(), "[{RUN}] samples gate:\n{failures:#?}");
}
