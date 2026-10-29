//! The systematics weights against MadGraph's own, event by event, on the
//! twelve banked unmatched proton runs that carry them.
//!
//! Every banked event carries MadGraph's `<mgrwt>` record and the 145 weights
//! its systematics module wrote from it under the default request (`--mur=0.5,1,2
//! --muf=0.5,1,2 --pdf=errorset` on `NNPDF23_lo_as_0130_qed`). The gate parses
//! the record and the particle lines, evaluates every variation with this
//! crate's [`lo_weight`] through the production [`Writer`], and compares each
//! printed weight with MadGraph's string. MadGraph prints eight significant
//! digits, so a value is known to half a unit in the eighth; a weight passes
//! when its spelling equals MadGraph's or differs by one unit in the last
//! digit, and the fraction equal is reported.
//!
//! What this cannot see: how this crate *fills* the record (the gate reads
//! MadGraph's), the MLM emission and chain terms (no banked MLM weights), any
//! set but 247000 with member-level couplings, and the dynamical-scale frame
//! and `scalefact` on the seven runs whose final states are massless at
//! `scalefact = 1` — the third negative control below measures exactly that
//! blindness.
//!
//! The MLM terms are reached by [`madgraphs_module_on_other_files`], which
//! reads event files MadGraph's module weighted outside the reference bundle
//! (the banked MLM runs carry the record and no weights): every weight within
//! one unit, the `--alps`-only weights exactly `XWGTUP` on an event whose
//! record gives them nothing to vary, and a control that drops the emission
//! and chain terms.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use vibegraph::cache::lhapdf::{DownloadRequest, Tier, Tiers};
use vibegraph::cache::store::Fetch;
use vibegraph::lhef::parse::LheFile;
use vibegraph::lhef::record::LheEvent;
use vibegraph::pdf::select::{load_for_lhaid, LoadedPdf};
use vibegraph::runcard::RunCard;
use vibegraph::systematics::plan::{resolve, Writer};
use vibegraph::systematics::record::{LoRecord, RecordKinematics};
use vibegraph::systematics::request::{Axis, DynScale, RequestSource, VariationRequest};
use vibegraph::systematics::weight::{lo_weight, Variation};
use vibegraph::ufo::identity::digest_bytes;

/// The banked runs carrying MadGraph's default systematics weights.
const RUNS: [&str; 12] = [
    "pp_to_bb",
    "pp_to_bb_fixed",
    "pp_to_bb_iht",
    "pp_to_bb_qcd2",
    "pp_to_jj",
    "pp_to_jj_ht",
    "pp_to_ll",
    "pp_to_ll_qcd0",
    "pp_to_ll_scalefact2",
    "pp_to_llj",
    "pp_to_llj_dyn",
    "pp_to_llj_fixed",
];

/// The SHA-256 of the `<initrwgt>` block every one of the twelve carries.
const BANKED_INITRWGT_SHA256: &str =
    "eb55efbb97a2cabfb6dd59595d3534770ed4175971d0cc217de18320d893c7d4";

/// The runs the `setscales.f` control detects on: massive final states, or a
/// `scalefact` off one.
const FRAME_DETECTED: [&str; 5] = [
    "pp_to_bb",
    "pp_to_bb_fixed",
    "pp_to_bb_iht",
    "pp_to_bb_qcd2",
    "pp_to_ll_scalefact2",
];

/// The gate's tolerance on a control's deviation: one unit in the eighth
/// significant digit is at most `1e-7` relative.
const DETECTION: f64 = 1e-7;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn banked_text(run: &str) -> String {
    let path = root()
        .join("validation/madgraph/output")
        .join(run)
        .join("Events/run_01/unweighted_events.lhe.gz");
    let file = std::fs::File::open(&path).unwrap_or_else(|e| {
        panic!(
            "{}: {e}; fetch the banked runs with `pixi run fetch-refdata`",
            path.display()
        )
    });
    let mut text = String::new();
    flate2::read::GzDecoder::new(file)
        .read_to_string(&mut text)
        .expect("a gzip LHE file");
    text
}

/// The `<initrwgt>` block, tags and final newline included.
fn initrwgt_block(text: &str) -> String {
    let start = text.find("<initrwgt>").expect("an <initrwgt> block");
    let end = text.find("</initrwgt>").expect("</initrwgt>") + "</initrwgt>\n".len();
    text[start..end].to_string()
}

/// The `<MGRunCard>` text the banner records.
fn banner_card(text: &str) -> String {
    let start = text.find("<MGRunCard>").expect("an <MGRunCard>") + "<MGRunCard>".len();
    let end = text.find("</MGRunCard>").expect("</MGRunCard>");
    text[start..end].replace("<![CDATA[", "").replace("]]>", "")
}

fn refuse(request: &DownloadRequest) -> Result<Box<dyn Fetch>, String> {
    Err(format!(
        "the gate reads only validation/pdf, and {} is not there; run `pixi run fetch-pdf`",
        request.what
    ))
}

fn tiers() -> (Tiers, tempfile::TempDir) {
    let cache = tempfile::tempdir().expect("a scratch cache");
    (
        Tiers {
            lhapdf_data_path: vec![root().join("validation/pdf")],
            data_path_tier: Tier::LhapdfDataPath,
            cvmfs: None,
            cache_root: cache.path().to_path_buf(),
        },
        cache,
    )
}

fn central(tiers: &Tiers) -> LoadedPdf {
    load_for_lhaid(247000, tiers, &refuse).expect("247000 in validation/pdf")
}

/// The writer of `request` over the banked runs' set.
fn writer(request: &VariationRequest, tiers: &Tiers, central: &LoadedPdf) -> Writer {
    let resolved = resolve(request, central, tiers, &refuse).expect("the request resolves");
    Writer::new(request, &resolved).expect("the request expands")
}

/// The header this crate writes for `writer`, as the block MadGraph writes.
fn our_block(writer: &Writer) -> String {
    let mut block = String::from("<initrwgt>\n");
    for line in writer.header_lines() {
        block.push_str(line);
        block.push('\n');
    }
    block.push_str("</initrwgt>\n");
    block
}

/// The request each MadGraph entry point reads off the banked card: the
/// arguments as given, the card with its `systematics_arguments` line
/// (`--from_card`), and the card without it (the `sys_*` fields).
fn entry_points(text: &str) -> Vec<(&'static str, VariationRequest)> {
    let card = banner_card(text);
    let arguments = VariationRequest::from_arguments(
        &[
            "--mur=0.5,1,2".to_string(),
            "--muf=0.5,1,2".to_string(),
            "--pdf=errorset".to_string(),
        ],
        RequestSource::CommandLine,
    )
    .expect("the default arguments");
    let from_card = |text: &str| {
        RunCard::parse(text)
            .expect("the banked card parses")
            .split()
            .expect("the banked card splits")
            .1
            .systematics
            .request
            .expect("use_syst = T with the systematics program")
    };
    let without_line: String = card
        .lines()
        .filter(|l| !l.contains("= systematics_arguments"))
        .map(|l| format!("{l}\n"))
        .collect();
    let with_line = from_card(&card);
    let sys_fields = from_card(&without_line);
    // Each path is the entry point it is named for, so an identical header
    // from all three is three readings and not one read three times.
    assert_eq!(with_line.source, RequestSource::Arguments);
    assert_eq!(sys_fields.source, RequestSource::SysFields);
    assert_eq!(
        sys_fields.together,
        [vec![Axis::Mur, Axis::Muf, Axis::Alps, Axis::Dyn]]
    );
    vec![
        ("arguments", arguments),
        ("--from_card", with_line),
        ("sys_*", sys_fields),
    ]
}

/// A weight's printed spelling, `%+13.7e`.
fn printed(w: f64) -> String {
    vibegraph::systematics::format::wgt_text(w)
}

/// How many units of the eighth significant digit `ours` is from `theirs`,
/// both as printed.
fn units_apart(ours: &str, theirs: &str) -> u64 {
    let (_, exponent) = theirs.split_once('e').expect("an exponent");
    let exponent: i32 = exponent.parse().expect("an integer exponent");
    let unit = 10f64.powi(exponent - 7);
    let a: f64 = ours.parse().expect("ours parses");
    let b: f64 = theirs.parse().expect("theirs parses");
    ((a - b).abs() / unit).round() as u64
}

/// The `(id, value text)` of every `<wgt>` line of an event.
fn banked_weights(event: &LheEvent) -> Vec<(String, String)> {
    event
        .trailer
        .iter()
        .filter_map(|line| {
            let rest = line.strip_prefix("<wgt id='")?;
            let (id, rest) = rest.split_once("'>")?;
            let value = rest.strip_suffix("</wgt>")?.trim();
            Some((id.to_string(), value.to_string()))
        })
        .collect()
}

/// Which kind of variation a weight is, for the per-class maxima.
fn class(v: &Variation<'_>, nominal: &Variation<'_>) -> &'static str {
    if v.dyn_scale != DynScale::Nominal {
        "dyn"
    } else if !std::ptr::addr_eq(v.pdf, nominal.pdf) {
        "pdf"
    } else if v.alps != 1.0 {
        "alps"
    } else if v.mur != 1.0 && v.muf != 1.0 {
        "mur_muf"
    } else if v.mur != 1.0 {
        "mur"
    } else if v.muf != 1.0 {
        "muf"
    } else {
        "nominal"
    }
}

/// The partonic centre-of-mass frame's momenta, every one scaled by
/// `scalefact`: what `setscales.f` reads a dynamical scale off, in place of
/// the lab frame and no factor MadGraph's Python module reads.
fn setscales_frame(k: &RecordKinematics, scalefact: f64) -> RecordKinematics {
    let total = [0, 1, 2, 3].map(|i| k.incoming.iter().map(|p| p[i]).sum::<f64>());
    let beta = total[3] / total[0];
    let gamma = 1.0 / (1.0 - beta * beta).sqrt();
    let boost = |p: &[f64; 4]| {
        let e = gamma * (p[0] - beta * p[3]);
        let pz = gamma * (p[3] - beta * p[0]);
        [e, p[1], p[2], pz].map(|c| c * scalefact)
    };
    RecordKinematics {
        incoming: k.incoming.iter().map(boost).collect(),
        final_state: k.final_state.iter().map(boost).collect(),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Control {
    /// The production weight.
    None,
    /// `μR` and `μF` factors exchanged.
    Swap,
    /// The αs power one higher than the record's.
    WrongPower,
    /// The dynamical scale off `setscales.f`'s frame and factor.
    SetscalesFrame,
    /// The MLM terms dropped: no emission scales (the `μR` power kept at the
    /// `<rscale>` integer) and each chain read at its last entry alone.
    NoMlmTerms,
}

const CONTROLS: [(Control, &str); 3] = [
    (Control::Swap, "muR <-> muF"),
    (Control::WrongPower, "alpha_s power n_qcd + 1"),
    (Control::SetscalesFrame, "setscales.f dynamical scale"),
];

/// `XWGTUP × W(var)/W(nominal)` for every variation of one event, under a
/// control's change.
fn control_weights(
    writer: &Writer,
    record: &LoRecord,
    kinematics: &RecordKinematics,
    xwgtup: f64,
    control: Control,
    scalefact: f64,
) -> Vec<f64> {
    if control == Control::None {
        return writer
            .weights(record, kinematics, xwgtup)
            .expect("the weights evaluate");
    }
    let mut record = record.as_written();
    if control == Control::WrongPower {
        record.n_qcd += 1;
    }
    if control == Control::NoMlmTerms {
        record.n_qcd -= record.emission_scales.len() as u32;
        record.emission_scales.clear();
        for chain in &mut record.beams {
            let last = chain.entries.pop();
            chain.entries = last.into_iter().collect();
        }
    }
    let frame = setscales_frame(kinematics, scalefact);
    let kinematics = if control == Control::SetscalesFrame {
        &frame
    } else {
        kinematics
    };
    let raw: Vec<f64> = writer
        .variations()
        .iter()
        .map(|v| {
            let v = if control == Control::Swap {
                Variation {
                    mur: v.muf,
                    muf: v.mur,
                    ..*v
                }
            } else {
                *v
            };
            lo_weight(&record, kinematics, &v).expect("the control evaluates")
        })
        .collect();
    raw[1..].iter().map(|w| xwgtup * w / raw[0]).collect()
}

/// One run's measurement.
#[derive(Default)]
struct RunResult {
    weights: u64,
    exact: u64,
    one_unit: u64,
    beyond: u64,
    worst_rel: BTreeMap<&'static str, f64>,
    controls: BTreeMap<&'static str, BTreeMap<&'static str, f64>>,
    /// Events whose record gives `--alps` something to vary (an emission
    /// scale, or a chain of more than one entry); of the others, those whose
    /// `--alps`-only weights are all `XWGTUP`'s spelling exactly; and of the
    /// first, those whose `--alps`-only weights all differ from it.
    alps_events: u64,
    alps_still: u64,
    alps_moved: u64,
}

fn measure(run: &str, writer: &Writer) -> RunResult {
    measure_text(run, &banked_text(run), writer, &CONTROLS)
}

/// [`measure`] on any file whose weights MadGraph's module wrote for the
/// writer's request, under `controls`.
fn measure_text(
    run: &str,
    text: &str,
    writer: &Writer,
    controls: &[(Control, &'static str)],
) -> RunResult {
    let file = LheFile::parse(text).expect("the file parses");
    let card = RunCard::parse(&banner_card(text)).expect("the banner's card parses");
    let scalefact = card.float("scalefact");
    let variations = writer.variations();
    let classes: Vec<&'static str> = variations[1..]
        .iter()
        .map(|v| class(v, &variations[0]))
        .collect();
    let per_event: Vec<RunResult> = file
        .events
        .par_iter()
        .map(|event| {
            let record = LoRecord::parse(&event.trailer)
                .expect("the record parses")
                .expect("every banked event carries <mgrwt>");
            let kinematics = RecordKinematics::of(event);
            let banked = banked_weights(event);
            let ids: Vec<&str> = banked.iter().map(|(id, _)| id.as_str()).collect();
            assert_eq!(
                ids,
                writer.ids().iter().map(String::as_str).collect::<Vec<_>>(),
                "{run}: the banked ids are not this crate's, in order"
            );
            let mut r = RunResult::default();
            let varied = !record.emission_scales.is_empty()
                || record.beams.iter().any(|c| c.entries.len() > 1);
            r.alps_events += u64::from(varied);
            let nominal_text = printed(event.weight);
            let alps: Vec<bool> = banked
                .iter()
                .zip(&classes)
                .filter(|(_, class)| **class == "alps")
                .map(|((_, theirs), _)| *theirs == nominal_text)
                .collect();
            if varied {
                r.alps_moved += u64::from(alps.iter().all(|&same| !same));
            } else {
                r.alps_still += u64::from(alps.iter().all(|&same| same));
            }
            let ours = control_weights(
                writer,
                &record,
                &kinematics,
                event.weight,
                Control::None,
                1.0,
            );
            for ((w, (_, theirs)), class) in ours.iter().zip(&banked).zip(&classes) {
                r.weights += 1;
                let spelled = printed(*w);
                match units_apart(&spelled, theirs) {
                    _ if spelled == *theirs => r.exact += 1,
                    0 | 1 => r.one_unit += 1,
                    _ => r.beyond += 1,
                }
                let value: f64 = theirs.parse().expect("a number");
                let rel = (w / value - 1.0).abs();
                let slot = r.worst_rel.entry(class).or_insert(0.0);
                *slot = slot.max(rel);
            }
            for &(control, name) in controls {
                let got = control_weights(
                    writer,
                    &record,
                    &kinematics,
                    event.weight,
                    control,
                    scalefact,
                );
                let by_class = r.controls.entry(name).or_default();
                for ((w, (_, theirs)), class) in got.iter().zip(&banked).zip(&classes) {
                    let value: f64 = theirs.parse().expect("a number");
                    let rel = (w / value - 1.0).abs();
                    let slot = by_class.entry(class).or_insert(0.0);
                    *slot = slot.max(rel);
                }
            }
            r
        })
        .collect();
    let mut total = RunResult::default();
    for r in per_event {
        total.weights += r.weights;
        total.exact += r.exact;
        total.one_unit += r.one_unit;
        total.beyond += r.beyond;
        total.alps_events += r.alps_events;
        total.alps_still += r.alps_still;
        total.alps_moved += r.alps_moved;
        for (class, rel) in r.worst_rel {
            let slot = total.worst_rel.entry(class).or_insert(0.0);
            *slot = slot.max(rel);
        }
        for (name, by_class) in r.controls {
            let into = total.controls.entry(name).or_default();
            for (class, rel) in by_class {
                let slot = into.entry(class).or_insert(0.0);
                *slot = slot.max(rel);
            }
        }
    }
    total
}

/// The header this crate writes for the default request, from each of
/// MadGraph's entry points, is the banked block byte for byte on every run.
#[test]
fn the_header_is_madgraphs_byte_for_byte_from_every_entry_point() {
    let (tiers, _cache) = tiers();
    let central = central(&tiers);
    for run in RUNS {
        let text = banked_text(run);
        let banked = initrwgt_block(&text);
        assert_eq!(
            digest_bytes(banked.as_bytes()),
            BANKED_INITRWGT_SHA256,
            "{run}: the banked block moved"
        );
        for (entry, request) in entry_points(&text) {
            let ours = our_block(&writer(&request, &tiers, &central));
            let identical = ours == banked;
            println!(
                "header {run:<20} {entry:<12} sha256 {} identical={identical}",
                &digest_bytes(ours.as_bytes())[..16]
            );
            assert!(identical, "{run} via {entry}:\n{ours}");
        }
    }
}

/// Every weight of every event of the twelve runs at MadGraph's printed
/// digits, and the three negative controls each detected where the design
/// says it is and blind where it says it is.
#[test]
fn every_banked_weight_is_reproduced_and_the_controls_detect() {
    let (tiers, _cache) = tiers();
    let central = central(&tiers);
    let writer = writer(&VariationRequest::madgraph_default(), &tiers, &central);
    assert_eq!(writer.ids().len(), 145);
    let mut failures = Vec::new();
    println!(
        "{:<22} {:>9} {:>9} {:>8} {:>8} {:>10}  worst relative deviation before rounding",
        "run", "weights", "exact", "1 unit", ">1 unit", "exact %"
    );
    for run in RUNS {
        let r = measure(run, &writer);
        let worst = r
            .worst_rel
            .iter()
            .map(|(c, v)| format!("{c} {v:.3e}"))
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "{run:<22} {:>9} {:>9} {:>8} {:>8} {:>9.5}%  {worst}",
            r.weights,
            r.exact,
            r.one_unit,
            r.beyond,
            100.0 * r.exact as f64 / r.weights as f64
        );
        if r.weights != 1_450_000 || r.beyond != 0 {
            failures.push(format!(
                "{run}: {} weights, {} beyond one unit",
                r.weights, r.beyond
            ));
        }
        for (name, by_class) in &r.controls {
            let worst = by_class.values().copied().fold(0.0f64, f64::max);
            let detected = worst > DETECTION;
            let classes = by_class
                .iter()
                .map(|(c, v)| format!("{c} {v:.3e}"))
                .collect::<Vec<_>>()
                .join(", ");
            println!(
                "    control {name:<28} {} (max {worst:.3e}; {classes})",
                if detected { "DETECTED" } else { "blind   " }
            );
            let expected = match *name {
                "setscales.f dynamical scale" => FRAME_DETECTED.contains(&run),
                _ => true,
            };
            if detected != expected {
                failures.push(format!(
                    "{run}: control '{name}' detected={detected}, the design says {expected}"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// `NNPDF31_nnlo_as_0118_mc_hessian_pdfas` member 102 carries its own
/// `AlphaS_MZ: 0.120`, `AlphaS_Vals` and `ForcePositive: 2` in its header,
/// over a `.info` at `0.118` with no `ForcePositive`: the member reads its own,
/// and member 0 the set's. Not banked, so it runs only against a copy on
/// `$LHAPDF_DATA_PATH` (`lhapdf install NNPDF31_nnlo_as_0118_mc_hessian_pdfas`).
#[test]
#[ignore = "needs NNPDF31_nnlo_as_0118_mc_hessian_pdfas on $LHAPDF_DATA_PATH"]
fn the_pdfas_member_reads_its_own_coupling_and_clamp() {
    const SET: &str = "NNPDF31_nnlo_as_0118_mc_hessian_pdfas";
    let dir = std::env::var_os("LHAPDF_DATA_PATH")
        .map(PathBuf::from)
        .expect("LHAPDF_DATA_PATH")
        .join(SET);
    let set = vibegraph::pdf::PdfSet::load(&dir, SET).expect("the set loads");
    assert_eq!(set.info.force_positive, 0);
    let central = set.member(0).unwrap();
    let varied = set.member(102).unwrap();
    let mz = |m: &vibegraph::pdf::PdfMember| m.alpha_s_info().unwrap().mz();
    println!(
        "{SET}: .info AlphaS_MZ {} ForcePositive {}; member 0 AlphaS_MZ {} ForcePositive {}; \
         member 102 AlphaS_MZ {} ForcePositive {} AlphaS_Vals[0] {} (set's {})",
        set.info.alpha_s.mz(),
        set.info.force_positive,
        mz(&central),
        central.force_positive(),
        mz(&varied),
        varied.force_positive(),
        varied.alpha_s_info().unwrap().vals[0],
        set.info.alpha_s.vals[0]
    );
    assert_eq!(mz(&central), 0.118);
    assert_eq!(central.force_positive(), 0);
    assert_eq!(mz(&varied), 0.120);
    assert_eq!(varied.force_positive(), 2);
    assert_ne!(
        varied.alpha_s_info().unwrap().vals,
        set.info.alpha_s.vals,
        "member 102 tabulates its own alpha_s"
    );
}

/// MadGraph's own systematics module on event files outside the bundle, the
/// MLM-matched ones in particular: `SYSTEMATICS_FILES` lists LHE files (plain
/// or gzipped, `:`-separated) whose weights MadGraph's module wrote under
/// `SYSTEMATICS_ARGS` (one command-line string) on `NNPDF23_lo_as_0130_qed`.
/// Every weight must print MadGraph's string or one unit from it; where the
/// request varies `--alps`, its weights must be `XWGTUP`'s spelling exactly on
/// the events whose record holds no emission scale and no chain longer than
/// one entry, and how many of the other events they move on is reported; and
/// dropping the emission and chain terms must be detected wherever such events
/// exist.
#[test]
#[ignore = "needs files weighted by MadGraph's module: SYSTEMATICS_FILES, SYSTEMATICS_ARGS"]
fn madgraphs_module_on_other_files() {
    let files = std::env::var("SYSTEMATICS_FILES").expect("SYSTEMATICS_FILES");
    let arguments = std::env::var("SYSTEMATICS_ARGS").expect("SYSTEMATICS_ARGS");
    let request = VariationRequest::from_command_line(&arguments).expect("the arguments parse");
    let (tiers, _cache) = tiers();
    let central = central(&tiers);
    let writer = writer(&request, &tiers, &central);
    let controls = [(Control::NoMlmTerms, "MLM terms dropped")];
    let mut failures = Vec::new();
    for path in files.split(':').filter(|p| !p.is_empty()) {
        let mut text = String::new();
        let file = std::fs::File::open(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        if path.ends_with(".gz") {
            flate2::read::GzDecoder::new(file).read_to_string(&mut text)
        } else {
            std::io::BufReader::new(file).read_to_string(&mut text)
        }
        .expect("the file reads");
        let r = measure_text(path, &text, &writer, &controls);
        let worst = r
            .worst_rel
            .iter()
            .map(|(c, v)| format!("{c} {v:.3e}"))
            .collect::<Vec<_>>()
            .join(", ");
        let events = text.matches("<event>").count() as u64;
        println!(
            "{path}\n  weights {} exact {} one-unit {} beyond {} ({:.5}% exact); worst {worst}",
            r.weights,
            r.exact,
            r.one_unit,
            r.beyond,
            100.0 * r.exact as f64 / r.weights as f64
        );
        println!(
            "  --alps: {} of {events} events give it something to vary, and it moves every \
             --alps weight on {} of them; it leaves every one at XWGTUP on {} of the other {}",
            r.alps_events,
            r.alps_moved,
            r.alps_still,
            events - r.alps_events
        );
        if r.weights == 0 || r.beyond != 0 {
            failures.push(format!("{path}: {} beyond one unit", r.beyond));
        }
        if request.alps.iter().any(|&a| a != 1.0) && r.alps_still != events - r.alps_events {
            failures.push(format!(
                "{path}: --alps moved a weight on {} events with nothing to vary",
                events - r.alps_events - r.alps_still
            ));
        }
        for (name, by_class) in &r.controls {
            let worst = by_class.values().copied().fold(0.0f64, f64::max);
            let detected = worst > DETECTION;
            println!(
                "  control {name}: {} (max {worst:.3e})",
                if detected { "DETECTED" } else { "blind" }
            );
            if detected != (r.alps_events > 0) {
                failures.push(format!(
                    "{path}: control '{name}' detected={detected} with {} events to vary",
                    r.alps_events
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
