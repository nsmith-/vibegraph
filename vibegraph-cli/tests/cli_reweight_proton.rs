//! End-to-end gate on `generate --reweight-card` at proton beams: the subprocess
//! each event is reweighted in is the one the integrand drew it in.
//!
//! Gated behind `extended-validation`; needs the fetched PDF set and the banked
//! MadGraph run cards:
//!
//!     pixi run validate-reweight
//!
//! # The oracle
//!
//! A reweighting ratio `|M|²_new / |M|²_old` taken in the wrong subprocess is
//! still a ratio near one: numerator and denominator come from the same wrong
//! amplitude, so no check on the weights can see the indexing. What can is the
//! denominator itself. `generate` checks every reweighted event's card-point
//! `|M|²` — the reweighter's own compilation of the member it indexes — against
//! the `|M|²` the integrand evaluated for the drawn (part, group, beam ordering)
//! at the event's coupling, and the reweighter's subprocess flavours against the
//! event record's, and refuses the file on any mismatch. These tests run that
//! audit on samples that populate every term it distinguishes and read its
//! report back, so a green run is a recorded count of checked events per part and
//! ordering, not the absence of an error.
//!
//! What each half sees: the `|M|²` comparison sees another part, another flavour
//! group, an unmirrored exchanged ordering, and a coupling taken at another
//! scale; it cannot see another member of the same flavour group, which shares the
//! card-point `|M|²` by construction. The flavour comparison sees exactly that. On
//! these processes, every one of those mutations fails the run (the report of the
//! session that added this gate lists the measurements).
//!
//! What neither can see: anything the reweighter and the integrand share — the
//! event's momenta, the beam mirror both apply (validated through
//! `validate_hadronic`'s cross sections, which carry the mirrored term), the
//! model's card-point parameters — and anything about the hypotheses themselves:
//! the numerators are compared against an independent generator in the
//! MadGraph-reweight oracle, not here.

use std::path::{Path, PathBuf};
use std::process::Command;

use vibegraph::lhef::parse::LheFile;

const PDF_SET: &str = "NNPDF23_lo_as_0130_qed";
const SEED: &str = "20261005";

/// One hypothesis on the exact path, a no-op launch (whose weight must be the
/// event's own), and one more exact one.
const CARD: &str = "\
launch --rwgt_name=wz
 set DECAY 23 2.6
launch --rwgt_name=nominal
launch --rwgt_name=aew
 set aEWM1 130
";

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn pdf_dir() -> PathBuf {
    repo().join("validation/pdf")
}

fn require(test: &str, run_card: &Path) {
    if !(run_card.is_file() && pdf_dir().join(PDF_SET).is_dir()) {
        vibegraph::validation::require(test, "the banked run card and the fetched PDF set", "");
    }
}

/// The audit's report: events checked per `(part, ordering)`, the largest
/// relative deviation, the two mismatch counts and the non-finite weights.
#[derive(Debug)]
struct Audit {
    events: usize,
    terms: Vec<(usize, String, usize)>,
    largest: f64,
    m2_mismatches: usize,
    flavour_mismatches: usize,
    non_finite: usize,
}

fn audit_of(log: &str) -> Audit {
    let line = log
        .lines()
        .find(|l| l.contains("card-point |M|^2 checked against the integrand"))
        .unwrap_or_else(|| panic!("generate did not report the reweighting audit:\n{log}"));
    // `… on N events (part P ORDERING n, …); largest relative deviation D;
    // beyond T: M; flavour mismatches: F; non-finite weights: W`
    let fields: Vec<&str> = line.split("; ").collect();
    assert_eq!(fields.len(), 5, "{line}");
    let last = |f: &str| f.rsplit(' ').next().unwrap().to_string();
    let head = fields[0];
    let events = head[head.find(" on ").unwrap() + 4..]
        .split(' ')
        .next()
        .unwrap()
        .parse()
        .unwrap();
    let inner = &head[head.find('(').unwrap() + 1..head.rfind(')').unwrap()];
    let terms = inner
        .split(", ")
        .map(|t| {
            let w: Vec<&str> = t.split(' ').collect();
            assert_eq!(w.len(), 4, "{t}");
            (
                w[1].parse().unwrap(),
                w[2].to_string(),
                w[3].parse().unwrap(),
            )
        })
        .collect();
    let largest = last(fields[1]).parse().unwrap();
    let m2_mismatches = last(fields[2]).parse().unwrap();
    let flavour_mismatches = last(fields[3]).parse().unwrap();
    let non_finite = last(fields[4]).parse().unwrap();
    Audit {
        events,
        terms,
        largest,
        m2_mismatches,
        flavour_mismatches,
        non_finite,
    }
}

/// `<wgt id='…'>` values of one event, in file order.
fn weights(trailer: &[String]) -> Vec<(String, f64)> {
    trailer
        .iter()
        .filter_map(|l| l.trim().strip_prefix("<wgt id='"))
        .map(|rest| {
            let (id, rest) = rest.split_once("'>").unwrap();
            let value = rest.trim().trim_end_matches("</wgt>").trim();
            (id.to_string(), value.parse().unwrap())
        })
        .collect()
}

/// Integrate `process` under `run_card`, generate `nevents` with the reweight
/// card, and return the audit and the file.
fn reweighted_sample(
    process: &str,
    run_card: &Path,
    budget: [&str; 2],
    nevents: usize,
) -> (Audit, LheFile) {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    let proc_card = dir.join("proc_card.dat");
    std::fs::write(&proc_card, format!("import model sm\n{process}")).unwrap();
    let rw_card = dir.join("reweight_card.dat");
    std::fs::write(&rw_card, CARD).unwrap();
    let out = dir.join("out");
    let integrate = Command::new(env!("CARGO_BIN_EXE_vibegraph"))
        .arg("integrate")
        .arg(&proc_card)
        .arg("--run-card")
        .arg(run_card)
        .arg("--out")
        .arg(&out)
        .arg("--pdf-dir")
        .arg(pdf_dir())
        .args(["--fixed-budget", "--neval", budget[0], "--niter", budget[1]])
        .args(["--seed", SEED])
        .output()
        .expect("spawn vibegraph");
    assert!(
        integrate.status.success(),
        "integrate failed:\n{}",
        String::from_utf8_lossy(&integrate.stderr)
    );
    let lhe = dir.join("events.lhe");
    let generate = Command::new(env!("CARGO_BIN_EXE_vibegraph"))
        .arg("generate")
        .arg(out.join("grid.bin.zst"))
        .arg(&proc_card)
        .arg("--run-card")
        .arg(run_card)
        .arg("--pdf-dir")
        .arg(pdf_dir())
        .args(["--seed", SEED, "--nevents", &nevents.to_string()])
        .arg("--reweight-card")
        .arg(&rw_card)
        .arg("-o")
        .arg(&lhe)
        .arg("--force")
        .output()
        .expect("spawn vibegraph");
    let log = String::from_utf8_lossy(&generate.stderr);
    assert!(generate.status.success(), "generate failed:\n{log}");
    let audit = audit_of(&log);
    eprintln!("{audit:?}");
    let file = LheFile::parse(&std::fs::read_to_string(&lhe).unwrap()).expect("our file parses");
    (audit, file)
}

/// The audit ran on every event, found nothing, and saw both beam orderings of
/// every part in `parts`; every event carries the card's weights, the no-op
/// launch's equal to `XWGTUP` as printed.
fn check(audit: &Audit, file: &LheFile, nevents: usize, parts: usize) {
    assert_eq!(audit.events, nevents, "{audit:?}");
    assert_eq!(audit.m2_mismatches, 0, "{audit:?}");
    assert_eq!(audit.flavour_mismatches, 0, "{audit:?}");
    assert_eq!(audit.non_finite, 0, "{audit:?}");
    // Independent compilations of one subprocess: rounding, at most.
    assert!(audit.largest <= 1e-12, "{audit:?}");
    for part in 0..parts {
        for ordering in ["direct", "exchanged"] {
            let n = audit
                .terms
                .iter()
                .find(|(p, o, _)| *p == part && o == ordering)
                .map_or(0, |t| t.2);
            assert!(
                n >= 50,
                "part {part} {ordering}: {n} events checked, too few to see an indexing error \
                 confined to it: {audit:?}"
            );
        }
    }
    assert_eq!(file.events.len(), nevents);
    // Events a hypothesis leaves unmoved at the file's eight digits: the Z width
    // is invisible far below the pole, so a few are expected, but not most.
    let mut unmoved = [0usize; 2];
    for event in &file.events {
        let w = weights(&event.trailer);
        let ids: Vec<&str> = w.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(ids, ["wz", "nominal", "aew"]);
        let nominal = w[1].1;
        assert!(
            (nominal / event.weight - 1.0).abs() < 1e-7,
            "the no-op launch's weight {nominal} is not the event's {}",
            event.weight
        );
        for (k, i) in [0, 2].into_iter().enumerate() {
            assert!(w[i].1 > 0.0, "{w:?}");
            unmoved[k] += usize::from(w[i].1 == nominal);
        }
    }
    eprintln!(
        "events a hypothesis leaves at the nominal weight: wz {}, aew {}",
        unmoved[0], unmoved[1]
    );
    assert!(
        unmoved.iter().all(|&n| n * 20 < nevents),
        "a hypothesis leaves most events where they were: {unmoved:?} of {nevents}"
    );
}

/// `p p > l+ l- j` under the banked fixed-scale card: one part, 24 subprocesses
/// in flavour groups of several members, both beam orderings.
#[test]
fn hadronic_reweighting_is_taken_in_the_drawn_subprocess() {
    let run_card = repo().join("validation/madgraph/output/pp_to_llj_fixed/Cards/run_card.dat");
    require(
        "hadronic_reweighting_is_taken_in_the_drawn_subprocess",
        &run_card,
    );
    let nevents = 5_000;
    let (audit, file) = reweighted_sample(
        "generate p p > l+ l- j QCD=2 QED=2\n",
        &run_card,
        ["40000", "4"],
        nevents,
    );
    check(&audit, &file, nevents, 1);
}

/// The MLM-matched `p p > e+ e- + 0,1,2 j` sample: three parts of different
/// multiplicity, whose subprocesses the reweighter numbers across parts, each
/// with both orderings populated, and the 1- and 2-jet parts' couplings taken at
/// each event's own clustered scale.
#[test]
fn matched_mixed_multiplicity_reweighting_is_taken_in_the_drawn_subprocess() {
    let run_card = repo().join("validation/madgraph/pp_to_ll_0j2j_mlm_run_card.dat");
    require(
        "matched_mixed_multiplicity_reweighting_is_taken_in_the_drawn_subprocess",
        &run_card,
    );
    let nevents = 3_000;
    let (audit, file) = reweighted_sample(
        "define p = g u c d s u~ c~ d~ s~\ndefine j = g u c d s u~ c~ d~ s~\n\
         generate p p > e+ e- @0\nadd process p p > e+ e- j @1\n\
         add process p p > e+ e- j j @2\n",
        &run_card,
        ["20000", "4"],
        nevents,
    );
    check(&audit, &file, nevents, 3);
}
