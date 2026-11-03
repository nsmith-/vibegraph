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
//! MadGraph's), any set but 247000 with member-level couplings unless CMS's
//! 880 members are on disk (below), and the dynamical-scale frame and
//! `scalefact` on the seven runs whose final states are massless at
//! `scalefact = 1` — the third negative control below measures exactly that
//! blindness.
//!
//! The MLM terms are gated on the four banked MLM runs that MadGraph's module
//! weighted under its default arguments plus `--alps=0.5,1,2`
//! (`Events/run_01_syst_default`): every weight within one unit, the
//! `--alps`-only weights exactly `XWGTUP` on an event whose record gives them
//! nothing to vary and moved on every other, and a control that drops the
//! emission and chain terms. [`madgraphs_module_on_other_files`] runs the
//! same check on any other file.
//!
//! CMS's `runcmsgrid.sh` arguments (43 sets, 880 members, `--together` and
//! `--start_id`) are gated on the four `Events/run_01_cms` runs: the
//! `<initrwgt>` block byte for byte from the bundled `.info` files alone, and,
//! where the 880 members are on disk, every weight.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use vibegraph::cache::lhapdf::{DownloadRequest, Tier, Tiers};
use vibegraph::cache::store::Fetch;
use vibegraph::lhef::parse::LheFile;
use vibegraph::lhef::record::{LheEvent, STATUS_INCOMING};
use vibegraph::pdf::alphas::GridAlphaS;
use vibegraph::pdf::select::{load_for_lhaid, open_set_for_lhaid, LoadedPdf};
use vibegraph::pdf::{PdfMember, PdfSet};
use vibegraph::runcard::RunCard;
use vibegraph::systematics::plan::{open_request, resolve, RunSet, Writer};
use vibegraph::systematics::record::{LoRecord, RecordKinematics};
use vibegraph::systematics::request::{Axis, DynScale, RequestSource, VariationRequest};
use vibegraph::systematics::weight::{lo_weight, Variation, VariationPdf};
use vibegraph::ufo::identity::digest_bytes;

mod common;

use common::banked_lhe::{banked_events_dir, banked_text, banner_card, read_lhe};

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

/// The banked runs MadGraph's module weighted with CMS's `runcmsgrid.sh`
/// arguments, on copies whose banner names lhaid 325300.
const CMS_RUNS: [&str; 4] = [
    "pp_to_ll",
    "pp_to_jj",
    "pp_to_ll_0j2j_mlm",
    "pp_to_ttx_0j1j_mlm",
];

/// The SHA-256 of the `<initrwgt>` block all four CMS outputs carry.
const CMS_INITRWGT_SHA256: &str =
    "ba7e21bc8c3cdaff027a91bd6c6f269b9dd4f49585db4c9266aaeaa2d1f730cf";

/// The run's own density in the CMS outputs' banners.
const CMS_LHAID: i64 = 325300;

/// The banked MLM runs MadGraph's module weighted under its default
/// arguments and `--alps=0.5,1,2`, with the number of events whose record
/// gives `--alps` something to vary (an emission scale, or a chain longer
/// than one entry), as counted on the oracle host from the written weights.
const SYST_DEFAULT_RUNS: [(&str, u64); 4] = [
    ("pp_to_llj_mlm", 8877),
    ("pp_to_ll_0j2j_mlm", 3527),
    ("pp_to_ttx_0j1j_mlm", 5253),
    ("pp_to_llj_xqcut_only", 0),
];

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

/// The module arguments of the banked `command.txt`: every `--` option on
/// the line that runs `systematics.py`, less the two that name output and
/// tools.
fn module_arguments(dir: &Path) -> String {
    let command = std::fs::read_to_string(dir.join("command.txt")).expect("command.txt");
    let line = command
        .lines()
        .find(|l| l.contains("systematics.py"))
        .expect("the systematics.py command line");
    line.split_whitespace()
        .filter(|a| a.starts_with("--"))
        .filter(|a| !a.starts_with("--result=") && !a.starts_with("--lhapdf_config="))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The `<initrwgt>` block, tags and final newline included.
fn initrwgt_block(text: &str) -> String {
    let start = text.find("<initrwgt>").expect("an <initrwgt> block");
    let end = text.find("</initrwgt>").expect("</initrwgt>") + "</initrwgt>\n".len();
    text[start..end].to_string()
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
    /// The weights classed `alps`, which the two counts above read.
    alps_weights: u64,
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
    let mut total = RunResult {
        alps_weights: classes.iter().filter(|c| **c == "alps").count() as u64,
        ..RunResult::default()
    };
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
            let ours = block_of(writer(&request, &tiers, &central).header_lines());
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
        for (_, name) in CONTROLS {
            let Some(by_class) = r.controls.get(name) else {
                failures.push(format!("{run}: control '{name}' was not evaluated"));
                continue;
            };
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
            let expected = match name {
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

/// One file MadGraph's module weighted under `request`, against `writer`:
/// every weight within one unit; where the request varies `--alps`, its
/// weights `XWGTUP`'s spelling exactly on every event whose record holds no
/// emission scale and no chain longer than one entry, and moved on every
/// other; and dropping the emission and chain terms detected exactly where
/// such events exist. `alps_events`, where given, is the number of such
/// events counted independently. Returns the failures.
fn module_gate(
    label: &str,
    text: &str,
    request: &VariationRequest,
    writer: &Writer,
    alps_events: Option<u64>,
) -> Vec<String> {
    let controls = [(Control::NoMlmTerms, "MLM terms dropped")];
    let mut failures = Vec::new();
    let r = measure_text(label, text, writer, &controls);
    let worst = r
        .worst_rel
        .iter()
        .map(|(c, v)| format!("{c} {v:.3e}"))
        .collect::<Vec<_>>()
        .join(", ");
    let events = text.matches("<event>").count() as u64;
    println!(
        "{label}\n  weights {} exact {} one-unit {} beyond {} ({:.5}% exact); worst {worst}",
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
    if r.weights != events * writer.ids().len() as u64 || r.beyond != 0 {
        failures.push(format!(
            "{label}: {} weights, {} beyond one unit",
            r.weights, r.beyond
        ));
    }
    if request.alps.iter().any(|&a| a != 1.0) {
        // Each factor off one, varied alone, is one weight: the two counts
        // below read those weights, and over none they would hold vacuously.
        let alone = !request.together.iter().any(|b| b.contains(&Axis::Alps));
        let factors = request.alps.iter().filter(|&&a| a != 1.0).count() as u64;
        if !alone || r.alps_weights != factors {
            failures.push(format!(
                "{label}: {} weights vary --alps alone, the request's {factors} factors off one \
                 (varied alone: {alone})",
                r.alps_weights
            ));
        }
        if r.alps_still != events - r.alps_events {
            failures.push(format!(
                "{label}: --alps moved a weight on {} events with nothing to vary",
                events - r.alps_events - r.alps_still
            ));
        }
        if r.alps_moved != r.alps_events {
            failures.push(format!(
                "{label}: --alps left a weight at XWGTUP on {} events with something to vary",
                r.alps_events - r.alps_moved
            ));
        }
    }
    if let Some(expected) = alps_events {
        if r.alps_events != expected {
            failures.push(format!(
                "{label}: {} events give --alps something to vary, the oracle host counted \
                 {expected}",
                r.alps_events
            ));
        }
    }
    for (_, name) in controls {
        let Some(by_class) = r.controls.get(name) else {
            failures.push(format!("{label}: control '{name}' was not evaluated"));
            continue;
        };
        let worst = by_class.values().copied().fold(0.0f64, f64::max);
        let detected = worst > DETECTION;
        println!(
            "  control {name}: {} (max {worst:.3e})",
            if detected { "DETECTED" } else { "blind" }
        );
        if detected != (r.alps_events > 0) {
            failures.push(format!(
                "{label}: control '{name}' detected={detected} with {} events to vary",
                r.alps_events
            ));
        }
    }
    failures
}

/// MadGraph's own systematics module under its default arguments plus
/// `--alps=0.5,1,2` on the four banked MLM runs: the emission and chain terms
/// of every weight, the `--alps` weights moving on exactly the events whose
/// record gives them something to vary (counted on the oracle host), and the
/// dropped-terms control detected on the three runs that have such events and
/// blind on `pp_to_llj_xqcut_only`, which has none.
#[test]
fn madgraphs_default_arguments_on_the_banked_mlm_runs() {
    let (tiers, _cache) = tiers();
    let central = central(&tiers);
    let mut failures = Vec::new();
    for (run, alps_events) in SYST_DEFAULT_RUNS {
        let dir = banked_events_dir(run, "run_01_syst_default");
        let arguments = module_arguments(&dir);
        assert_eq!(
            arguments, "--mur=0.5,1,2 --muf=0.5,1,2 --pdf=errorset --alps=0.5,1,2",
            "{run}: the banked command"
        );
        let request = VariationRequest::from_command_line(&arguments).expect("the arguments parse");
        let writer = writer(&request, &tiers, &central);
        assert_eq!(writer.ids().len(), 147);
        let text = read_lhe(&dir.join("unweighted_events.lhe.gz"));
        failures.extend(module_gate(
            run,
            &text,
            &request,
            &writer,
            Some(alps_events),
        ));
        let (events, worst, mean) = written_head_shift(&text, &writer);
        if events > 0 {
            println!(
                "  info, chain heads other than the written flavours on {events} events: with \
                 the written flavours MadGraph's module would move their weights by at most {} \
                 (mean of each event's largest move {mean:.3e})",
                worst
                    .iter()
                    .map(|(c, v)| format!("{c} {v:.3e}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        } else {
            println!("  info, every chain head is the written flavour");
        }
        let pinned = STALE_HEAD_EVENTS
            .iter()
            .find(|(r, _)| *r == run)
            .map(|r| r.1);
        if Some(events) != pinned {
            failures.push(format!(
                "{run}: chain heads other than the written flavours on {events} events, pinned {pinned:?}"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Per default-argument MLM run, the events whose record names another
/// flavour than the written one at the head of a chain: MadEvent's
/// `ipdgcl(j, igraphs(1))` slot, filled for the graph its first call named and
/// read after `rewgt`'s own clustering named another (`reweight.f:1439-1440`,
/// `:1479`), on symmetry-permuted channels.
const STALE_HEAD_EVENTS: [(&str, usize); 4] = [
    ("pp_to_llj_mlm", 0),
    ("pp_to_ll_0j2j_mlm", 19),
    ("pp_to_ttx_0j1j_mlm", 219),
    ("pp_to_llj_xqcut_only", 0),
];

/// What MadGraph's module computes for the chain heads its record names,
/// against the written flavours: over the events whose head differs, the
/// relative change of each weight when each chain's head is the written
/// parton of its beam, worst per class and the mean of each event's worst.
fn written_head_shift(text: &str, writer: &Writer) -> (usize, BTreeMap<&'static str, f64>, f64) {
    let file = LheFile::parse(text).expect("the file parses");
    let variations = writer.variations();
    let classes: Vec<&'static str> = variations[1..]
        .iter()
        .map(|v| class(v, &variations[0]))
        .collect();
    let shifts: Vec<(BTreeMap<&'static str, f64>, f64)> = file
        .events
        .par_iter()
        .filter_map(|event| {
            let record = LoRecord::parse(&event.trailer).ok()??.as_written();
            let mut incoming: Vec<_> = event
                .particles
                .iter()
                .filter(|p| p.status == STATUS_INCOMING)
                .collect();
            incoming.sort_by(|a, b| b.momentum[3].total_cmp(&a.momentum[3]));
            let mut written = record.clone();
            let mut moved = false;
            for (chain, parton) in written.beams.iter_mut().zip(&incoming) {
                if let Some(head) = chain.entries.first_mut() {
                    moved |= head.0 != i64::from(parton.pdg);
                    head.0 = i64::from(parton.pdg);
                }
            }
            if !moved {
                return None;
            }
            let kinematics = RecordKinematics::of(event);
            let ours = writer
                .weights(&written, &kinematics, event.weight)
                .expect("the weights evaluate");
            let mut by_class: BTreeMap<&'static str, f64> = BTreeMap::new();
            let mut worst = 0.0f64;
            for ((w, (_, theirs)), class) in ours.iter().zip(banked_weights(event)).zip(&classes) {
                let value: f64 = theirs.parse().expect("a number");
                let rel = (w / value - 1.0).abs();
                let slot = by_class.entry(class).or_insert(0.0);
                *slot = slot.max(rel);
                worst = worst.max(rel);
            }
            Some((by_class, worst))
        })
        .collect();
    let mut total: BTreeMap<&'static str, f64> = BTreeMap::new();
    for (by_class, _) in &shifts {
        for (class, rel) in by_class {
            let slot = total.entry(class).or_insert(0.0);
            *slot = slot.max(*rel);
        }
    }
    let mean = shifts.iter().map(|s| s.1).sum::<f64>() / shifts.len().max(1) as f64;
    (shifts.len(), total, mean)
}

/// MadGraph's own systematics module on event files outside the bundle:
/// `SYSTEMATICS_FILES` lists LHE files (plain or gzipped, `:`-separated)
/// whose weights MadGraph's module wrote under `SYSTEMATICS_ARGS` (one
/// command-line string) on `NNPDF23_lo_as_0130_qed`, each checked as
/// [`module_gate`] checks.
#[test]
#[ignore = "needs files weighted by MadGraph's module: SYSTEMATICS_FILES, SYSTEMATICS_ARGS"]
fn madgraphs_module_on_other_files() {
    let files = std::env::var("SYSTEMATICS_FILES").expect("SYSTEMATICS_FILES");
    let arguments = std::env::var("SYSTEMATICS_ARGS").expect("SYSTEMATICS_ARGS");
    let request = VariationRequest::from_command_line(&arguments).expect("the arguments parse");
    let (tiers, _cache) = tiers();
    let central = central(&tiers);
    let writer = writer(&request, &tiers, &central);
    let mut failures = Vec::new();
    for path in files.split(':').filter(|p| !p.is_empty()) {
        let text = read_lhe(Path::new(path));
        failures.extend(module_gate(path, &text, &request, &writer, None));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// `write_banner`'s description edit, for the control's lines.
fn madgraph_edit(text: &str) -> String {
    text.replace("=>", ";")
        .replace('>', ".gt.")
        .replace('<', ".lt.")
}

/// The bundled `.info` files of a CMS run as an LHAPDF tree: each under
/// `<name>/<name>.info`, its SHA-256 checked against the oracle host's
/// `pdfsets.json`, and a `pdfsets.index` listing each by its `SetIndex`.
fn cms_info_tree(dir: &Path) -> (tempfile::TempDir, Vec<PdfSet>) {
    let recorded: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.join("pdfsets.json")).expect("pdfsets.json"),
    )
    .expect("pdfsets.json parses");
    let tree = tempfile::tempdir().expect("a scratch tree");
    let mut index = String::new();
    let mut sets = Vec::new();
    let mut names: Vec<_> = std::fs::read_dir(dir.join("pdfinfo"))
        .expect("the bundled .info files")
        .map(|e| e.expect("an entry").path())
        .collect();
    names.sort();
    for path in names {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("a set name")
            .to_string();
        let bytes = std::fs::read(&path).expect("the .info reads");
        assert_eq!(
            digest_bytes(&bytes),
            recorded[&name]["info_sha256"]
                .as_str()
                .expect("info_sha256"),
            "{name}: the bundled .info is not the one the oracle host read"
        );
        let set_dir = tree.path().join(&name);
        std::fs::create_dir_all(&set_dir).expect("a set directory");
        std::fs::write(set_dir.join(format!("{name}.info")), &bytes).expect("the .info writes");
        let set = PdfSet::load(&set_dir, &name).expect("the .info parses");
        let set_index = set.info.set_index().expect("a SetIndex");
        assert_eq!(
            Some(set_index),
            recorded[&name]["lhapdf_id"].as_i64(),
            "{name}: SetIndex"
        );
        index.push_str(&format!("{set_index} {name} 1\n"));
        sets.push(set);
    }
    assert_eq!(sets.len(), 43, "the CMS list's 43 sets");
    std::fs::write(tree.path().join("pdfsets.index"), index).expect("the index writes");
    (tree, sets)
}

fn tiers_at(path: &Path) -> (Tiers, tempfile::TempDir) {
    let cache = tempfile::tempdir().expect("a scratch cache");
    (
        Tiers {
            lhapdf_data_path: vec![path.to_path_buf()],
            data_path_tier: Tier::LhapdfDataPath,
            cvmfs: None,
            cache_root: cache.path().to_path_buf(),
        },
        cache,
    )
}

/// `<initrwgt>` lines between the tags, as the block MadGraph writes.
fn block_of(lines: &[String]) -> String {
    let mut block = String::from("<initrwgt>\n");
    for line in lines {
        block.push_str(line);
        block.push('\n');
    }
    block.push_str("</initrwgt>\n");
    block
}

/// The lines of `ours` that differ from `banked`, by position.
fn differing_lines(ours: &str, banked: &str) -> Vec<(usize, String, String)> {
    let (a, b): (Vec<&str>, Vec<&str>) = (ours.lines().collect(), banked.lines().collect());
    (0..a.len().max(b.len()))
        .filter(|&i| a.get(i) != b.get(i))
        .map(|i| {
            (
                i + 1,
                a.get(i).unwrap_or(&"").to_string(),
                b.get(i).unwrap_or(&"").to_string(),
            )
        })
        .collect()
}

/// The `<initrwgt>` block CMS's arguments write, from the banked command
/// line and the 43 bundled `.info` files alone (no member is read), is
/// MadGraph's on all four CMS runs byte for byte. The control writes each
/// set's description as the YAML text of its `.info` reads, without LHAPDF's
/// re-emission: it must differ from the banked block on exactly the group
/// lines of the sets whose description yaml-cpp writes quoted.
#[test]
fn the_cms_header_is_madgraphs_byte_for_byte() {
    for run in CMS_RUNS {
        let dir = banked_events_dir(run, "run_01_cms");
        let text = read_lhe(&dir.join("unweighted_events.lhe.gz"));
        let banked = initrwgt_block(&text);
        assert_eq!(
            digest_bytes(banked.as_bytes()),
            CMS_INITRWGT_SHA256,
            "{run}: the banked block moved"
        );
        let arguments = module_arguments(&dir);
        let request =
            VariationRequest::from_command_line(&arguments).expect("the CMS arguments parse");
        assert_eq!(request.start_id, Some(1001));
        let (tree, sets) = cms_info_tree(&dir);
        let (tiers, _cache) = tiers_at(tree.path());
        let opened_run =
            open_set_for_lhaid(CMS_LHAID, &tiers, &refuse).expect("325300 in the tree");
        let run_set = RunSet {
            set: &opened_run.set,
            tier: opened_run.found.tier,
            lhaid: CMS_LHAID,
            member: 0,
        };
        let opened = open_request(&request, &run_set, &tiers, &refuse).expect("the list opens");
        let lines = opened.header_lines(&request).expect("the request expands");
        let weights = lines
            .iter()
            .filter(|l| l.starts_with("<weight id="))
            .count();
        let ours = block_of(&lines);
        let differ = differing_lines(&ours, &banked);
        println!(
            "cms header {run:<20} weights {weights} sha256 {} identical={}",
            &digest_bytes(ours.as_bytes())[..16],
            differ.is_empty()
        );
        for (n, a, b) in differ.iter().take(5) {
            println!("  line {n}\n    ours   {a}\n    banked {b}");
        }
        assert!(differ.is_empty(), "{run}: the CMS header is not MadGraph's");
        assert_eq!(weights, 926);

        let quoted: Vec<&PdfSet> = sets
            .iter()
            .filter(|s| s.info.lhapdf_description() != s.info.description())
            .collect();
        let mut control = ours.clone();
        for set in &quoted {
            control = control.replace(
                &madgraph_edit(&set.info.lhapdf_description()),
                &madgraph_edit(set.info.description()),
            );
        }
        let caught = differing_lines(&control, &banked);
        let caught_sets: Vec<&str> = caught
            .iter()
            .filter_map(|(_, line, _)| {
                line.strip_prefix("<weightgroup name=\"")
                    .and_then(|l| l.split('"').next())
            })
            .collect();
        println!(
            "  control: the YAML text as the description: sha256 {}, {} line(s) differ ({})",
            &digest_bytes(control.as_bytes())[..16],
            caught.len(),
            caught_sets.join(", ")
        );
        // Only sets read whole open a group, so a quoted description of a
        // set read at `@0` never reaches the header.
        let grouped: Vec<&str> = quoted
            .iter()
            .map(|s| s.name())
            .filter(|name| banked.contains(&format!("<weightgroup name=\"{name}\"")))
            .collect();
        assert!(!caught.is_empty(), "{run}: the control is not detected");
        assert_eq!(caught_sets, grouped, "{run}: the control's lines");
        assert_eq!(caught.len(), grouped.len(), "{run}: the control's lines");
    }
}

/// A member density read with its set's `.info` coupling instead of the
/// member's own: the control for the member-level `AlphaS_*` overrides.
struct SetCoupling {
    member: PdfMember,
    alphas: GridAlphaS,
}

impl VariationPdf for SetCoupling {
    fn alphas_q(&self, q: f64) -> Result<f64, String> {
        Ok(self.alphas.eval(q))
    }

    fn xfx_q(&self, pdg: i64, x: f64, q: f64) -> Result<f64, String> {
        self.member
            .try_xfx_q2(i32::try_from(pdg).map_err(|e| e.to_string())?, x, q * q)
            .map_err(|e| e.to_string())
    }
}

/// The `PDF="…"` lhaid of every `<weight>` line, in id order.
fn header_pdf_ids(lines: &[String]) -> Vec<i64> {
    lines
        .iter()
        .filter(|l| l.starts_with("<weight id="))
        .map(|l| {
            let (_, rest) = l.split_once(" PDF=\"").expect("a PDF attribute");
            rest.split('"').next().unwrap().parse().expect("an lhaid")
        })
        .collect()
}

/// CMS's `runcmsgrid.sh` arguments on the four `run_01_cms` runs: every one
/// of the 926 weights of every event within one unit of MadGraph's string,
/// ids 1001–1926 in order, the header byte for byte from the loaded members,
/// and the `--alps` and dropped-MLM-terms checks of [`module_gate`]. The 43
/// sets are not banked: `SYSTEMATICS_CMS_PDFS` names an LHAPDF tree holding
/// them (at least the 880 members the list reads, and a `pdfsets.index`).
///
/// The member-level couplings are pinned on real data: 325300 members 101 and
/// 102 must read `AlphaS_MZ` 0.116 and 0.120 from their own headers over the
/// set's 0.118, and evaluating every member whose header overrides the set's
/// `AlphaS_MZ` with the set's coupling instead must be detected on every run
/// whose weights depend on αs (all but `pp_to_ll`, whose events carry no
/// power of αs and no emission).
#[test]
#[ignore = "needs the 43 CMS sets on SYSTEMATICS_CMS_PDFS"]
fn cms_arguments_reproduce_madgraphs_weights() {
    let pdfs = PathBuf::from(std::env::var("SYSTEMATICS_CMS_PDFS").expect("SYSTEMATICS_CMS_PDFS"));
    let (tiers, _cache) = tiers_at(&pdfs);
    let central = load_for_lhaid(CMS_LHAID, &tiers, &refuse).expect("325300 member 0");
    let arguments = module_arguments(&banked_events_dir(CMS_RUNS[0], "run_01_cms"));
    let request = VariationRequest::from_command_line(&arguments).expect("the CMS arguments parse");
    let started = std::time::Instant::now();
    let resolved = resolve(&request, &central, &tiers, &refuse).expect("the list resolves");
    println!(
        "resolved {} sets, {} members in {:.1} s",
        resolved.inputs().len(),
        resolved.members_loaded(),
        started.elapsed().as_secs_f64()
    );
    let writer = Writer::new(&request, &resolved).expect("the request expands");
    assert_eq!(writer.ids().len(), 926);
    assert_eq!(writer.ids()[0], "1001");
    let mut failures = Vec::new();

    // The member-level couplings, and the control that reads them from the
    // set instead.
    let pdf_ids = header_pdf_ids(writer.header_lines());
    let mut overrides: BTreeMap<i64, SetCoupling> = BTreeMap::new();
    for id in pdf_ids.iter().collect::<std::collections::BTreeSet<_>>() {
        let set = open_set_for_lhaid(*id, &tiers, &refuse).expect("the set opens");
        let member = u32::try_from(id - set.set_index).unwrap();
        let pdf = set.set.member(member).expect("the member loads");
        let own = pdf.alpha_s_info().expect("member alpha_s").mz();
        let of_set = set.set.info.alpha_s.mz();
        if *id == CMS_LHAID + 101 || *id == CMS_LHAID + 102 {
            let want = if *id == CMS_LHAID + 101 { 0.116 } else { 0.120 };
            println!(
                "{} member {member}: AlphaS_MZ {own} (set's {of_set})",
                set.set.name()
            );
            if own != want {
                failures.push(format!("{id}: AlphaS_MZ {own}, its header says {want}"));
            }
        }
        if of_set.is_finite() && own != of_set {
            println!(
                "  overrides the set's AlphaS_MZ: {} member {member} ({id}): {own} over {of_set}",
                set.set.name()
            );
            let alphas = GridAlphaS::from_info(&set.set.info.alpha_s).expect("ipol");
            overrides.insert(
                *id,
                SetCoupling {
                    member: pdf,
                    alphas,
                },
            );
        }
    }

    for run in CMS_RUNS {
        let dir = banked_events_dir(run, "run_01_cms");
        assert_eq!(
            module_arguments(&dir),
            arguments,
            "{run}: the same arguments"
        );
        let text = read_lhe(&dir.join("unweighted_events.lhe.gz"));
        let banked = initrwgt_block(&text);
        let ours = block_of(writer.header_lines());
        println!(
            "cms {run}: header from the loaded members identical={}",
            ours == banked
        );
        if ours != banked {
            failures.push(format!("{run}: the header from the loaded members differs"));
        }
        failures.extend(module_gate(run, &text, &request, &writer, None));

        let file = LheFile::parse(&text).expect("the file parses");
        let worst = file
            .events
            .par_iter()
            .map(|event| {
                let variations = writer.variations();
                let record = LoRecord::parse(&event.trailer)
                    .expect("the record parses")
                    .expect("<mgrwt>")
                    .as_written();
                let kinematics = RecordKinematics::of(event);
                let banked = banked_weights(event);
                let nominal = lo_weight(&record, &kinematics, &variations[0]).unwrap();
                let xwgtup: f64 = printed(event.weight).parse().unwrap();
                let mut worst = 0.0f64;
                for (k, id) in pdf_ids.iter().enumerate() {
                    let Some(pdf) = overrides.get(id) else {
                        continue;
                    };
                    let v = Variation {
                        pdf,
                        ..variations[k + 1]
                    };
                    let w = xwgtup * lo_weight(&record, &kinematics, &v).unwrap() / nominal;
                    let theirs: f64 = banked[k].1.parse().unwrap();
                    worst = worst.max((w / theirs - 1.0).abs());
                }
                worst
            })
            .reduce(|| 0.0, f64::max);
        let detected = worst > DETECTION;
        println!(
            "  control the set's coupling for those members: {} (max {worst:.3e})",
            if detected { "DETECTED" } else { "blind" }
        );
        if detected != (run != "pp_to_ll") {
            failures.push(format!(
                "{run}: the set-coupling control detected={detected}"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
