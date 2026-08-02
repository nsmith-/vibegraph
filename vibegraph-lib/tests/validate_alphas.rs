//! The running strong coupling ([`vibegraph::coupling::alphas`]) against
//! MadGraph's banked events and run logs.
//!
//! The committed reference grid — the Fortran routine itself, bit-for-bit — is
//! `alphas_reference_grid.rs`; it says nothing about where `asmz` and `nloop`
//! come from, which is what the two oracles here are for.
//!
//! # 1. The banked MadGraph events
//!
//! Every banked run's `unweighted_events.lhe.gz` carries `SCALUP` and `AQCDUP`
//! per `<event>` line: the scale MadGraph chose and the coupling it evaluated
//! there. Feeding the printed `SCALUP` back through this module must reproduce
//! `AQCDUP` to the precision it is printed at, for every event of every run —
//! a per-event oracle over 280k events rather than a single scalar. Choosing the
//! scale is a separate concern and is not exercised here; `SCALUP` is taken as
//! given, which is also why the two `2 → 6` runs sit outside the gate (see
//! [`SCALUP_IS_THE_RENORMALISATION_SCALE`]).
//!
//! Two properties of the `<event>` line have to be reproduced before the
//! comparison means anything, and both were found by chasing residuals that a
//! loose tolerance would have absorbed: the fields carry seven significant
//! digits in a nine-digit field, and `AQCDUP` is `αs` scaled by `π/3.1415926`
//! because `unwgt.f` divides by a truncated π. Neither is visible at 1e-6, and
//! together they are the difference between 95% and 100% of events reproducing
//! the printed digits.
//!
//! **What this cannot see.** *Not* the PDF **label** override, despite it being
//! the failure mode the field was expected to catch. MadGraph's own tooling writes
//! the PDF's `aS` into the parameter card before the run, so in every such banked
//! run the parameter card and the PDF label already agree — dropping the override
//! entirely would leave every `AQCDUP` unchanged. What does see it is the
//! MadGraph run log ([`banked_run_logs_pin_the_alpha_s_source_rule`]), which
//! prints the parameter-card value and the PDF value separately, at 17 digits.
//! The two are enough to pin the rule end to end: the log fixes `asmz`, the
//! events fix the evolution from it.
//!
//! # 2. The grid as the source
//!
//! A `pdlabel = lhapdf` run takes `αs` from the PDF set's own tabulation instead,
//! and there the two candidate sources do *not* agree: `0.1300027` from the set
//! against `0.1300028` from the parameter card, twice the printing budget apart.
//! So for such a run the events are back in play — all 10 000 of
//! `pp_to_llj_fixed`'s reproduce their printed `AQCDUP` from the grid, and none
//! would from the card. The run log adds the same reading at 17 digits, and only
//! at `M_Z`: MadGraph resolves and prints `αs(M_Z)` once and prints no value at
//! any other scale, so the log pins the reading at the reference scale and the
//! events pin it everywhere else.
//!
//! Two of the four grid-sourced runs fix `μR = 91.188`, which *is* `M_Z`, so on
//! those the two statements coincide. The other two carry a per-event scale, and
//! there the events are the only oracle for the shape of the interpolation
//! between knots: `pp_to_llj_dyn` takes 9966 distinct `SCALUP` values over 10 000
//! events. What they cannot reach is either end of the table — no banked event
//! sits below the first knot or above the last — so the two continuations outside
//! it are pinned by LHAPDF probe values instead, in `validate_pdf_grid.rs`.

use std::path::{Path, PathBuf};
use std::process::Command;

use vibegraph::coupling::alphas::{
    asmz_from_param_card, AlphaSSource, NLoop, RunningAlphaS, ZMASS,
};
use vibegraph::pdf::grid::AlphaSInfo;
use vibegraph::pdf::PdfSet;
use vibegraph::runcard::RunCard;
use vibegraph::ufo::slha::ParamCard;

mod common;

fn output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/output")
}

/// Every banked run directory that carries an unweighted event file.
fn banked_runs() -> Vec<(String, PathBuf)> {
    let mut runs: Vec<(String, PathBuf)> = std::fs::read_dir(output_dir())
        .expect("MadGraph output directory (pixi run -e madgraph build-diagrams)")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if !path
                .join("Events/run_01/unweighted_events.lhe.gz")
                .is_file()
            {
                return None;
            }
            let name = path.file_name()?.to_string_lossy().into_owned();
            Some((name, path))
        })
        .collect();
    runs.sort();
    assert!(
        runs.len() >= 5,
        "expected at least the five QCD runs, found {}",
        runs.len()
    );
    runs
}

/// Runs whose `αs` MadGraph reads out of the PDF grid: `pdlabel = lhapdf`
/// links `alfas_functions_lhapdf.f`, whose `ALPHAS(Q)` is LHAPDF's
/// `alphasPDF(Q)` and not the beta-function solve this file's first oracle
/// validates. [`resolve`] asserts the classification both ways, so a run that
/// changed source shows up as a failure rather than as a quiet reclassification.
const GRID_ALPHA_S_RUNS: &[&str] = &[
    "pp_to_bb_fixed",
    "pp_to_jj",
    "pp_to_llj_dyn",
    "pp_to_llj_fixed",
];

/// The declared runs of `list` whose directory is on this machine, sorted.
///
/// A row the manifest marks `bundled = false` has banked artifacts that live in
/// a local work area and are deliberately not in the pinned bundle yet, so a
/// checkout that fetched the bundle and does not have it is complete with
/// respect to what the bundle promises. A declared row that is absent and *is*
/// bundled is an incomplete environment and says so.
fn present(list: &[&str], runs: &[(String, PathBuf)]) -> Vec<String> {
    let unbundled = common::manifest::unbundled_rows();
    let mut names = Vec::new();
    for name in list {
        if runs.iter().any(|(have, _)| have == *name) {
            names.push((*name).to_string());
        } else if !unbundled.contains(*name) {
            vibegraph::validation::require("alphas_gate_matches_madgraph", "a banked run", name);
        }
    }
    names.sort();
    names
}

/// The LHAPDF set each `lhaid` names. A run card carries only the id, and the
/// grid `αs` has to come from the set the *densities* come from, so the mapping
/// is stated here rather than inferred from whatever set happens to be unpacked.
const PDF_SET_BY_LHAID: &[(i64, &str)] = &[(247000, "NNPDF23_lo_as_0130_qed")];

/// How far this crate's reading of the set's `αs` knots is allowed to sit from
/// the value MadGraph's LHAPDF call returned at `M_Z`.
///
/// Both sides run the same algorithm — LHAPDF's `AlphaS_Ipol`, a cubic in
/// `ln Q²` — over the same knots, so what is left is arithmetic noise: one `ln`
/// call and a handful of rounded operations, whose worst case is a few ulp of a
/// number near `0.13`, i.e. a few times `1e-16` relative. The observed residual
/// against MadGraph's 17-digit report is `0`; the bound is set two orders above
/// the noise so a system `libm` whose `ln` rounds differently in the last bit
/// stays inside it.
const GRID_ALPHA_S_TOL: f64 = 1e-14;

/// The `AlphaS_*` metadata of the set a run's beams read, or `None` for a run
/// that names no LHAPDF set.
fn set_alpha_s_info(card: &RunCard) -> Option<AlphaSInfo> {
    if card.pdlabel != "lhapdf" {
        return None;
    }
    let name = PDF_SET_BY_LHAID
        .iter()
        .find(|(id, _)| *id == card.lhaid)
        .map(|(_, name)| *name)
        .unwrap_or_else(|| panic!("no PDF set registered for lhaid {}", card.lhaid));
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/pdf")
        .join(name);
    let set = PdfSet::load(&dir, name).unwrap_or_else(|e| {
        panic!(
            "cannot load PDF set {name} from {}: {e}\n\
             run `pixi run -e madgraph fetch-pdf`",
            dir.display()
        )
    });
    Some(set.info.alpha_s)
}

/// Everything a banked run's cards say about its strong coupling.
struct Resolved {
    source: AlphaSSource,
    /// The parameter card's own `αs(M_Z)`, as `setrun.f` recovers it from `G`.
    param_card_asmz: f64,
}

/// The `αs` source MadGraph linked for `run`, resolved from the same run card
/// and parameter card it read. Which arm comes out is asserted against
/// [`GRID_ALPHA_S_RUNS`] rather than taken on trust: a source silently switching
/// to the beta-function solve would otherwise still produce a number.
fn resolve(name: &str, run: &Path) -> Resolved {
    let card = RunCard::parse_file(&run.join("Cards/run_card.dat")).expect("run card");
    let params = ParamCard::from_file(&run.join("Cards/param_card.dat")).expect("param card");
    let a_s = params.get("sminputs", &[3]).expect("aS in SMINPUTS");
    let info = set_alpha_s_info(&card);
    let source = AlphaSSource::from_run_card(&card, a_s, info.as_ref())
        .unwrap_or_else(|e| panic!("{name}: alpha_s source: {e}"));
    assert_eq!(
        source.grid().is_some(),
        GRID_ALPHA_S_RUNS.contains(&name),
        "{name}: alpha_s source arm disagrees with GRID_ALPHA_S_RUNS"
    );
    Resolved {
        source,
        param_card_asmz: asmz_from_param_card(a_s),
    }
}

/// `(scalup, aqcdup)` for every `<event>` of a run.
fn event_scales(run: &Path) -> Vec<(f64, f64)> {
    let lhe = run.join("Events/run_01/unweighted_events.lhe.gz");
    let out = Command::new("gzip")
        .args(["-dc", lhe.to_str().unwrap()])
        .output()
        .expect("gzip -dc");
    assert!(out.status.success(), "gzip failed on {}", lhe.display());
    let text = String::from_utf8_lossy(&out.stdout);

    let mut scales = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.trim() != "<event>" {
            continue;
        }
        let info = lines.next().expect("event info line");
        // NUP IDPRUP XWGTUP SCALUP AQEDUP AQCDUP
        let fields: Vec<&str> = info.split_whitespace().collect();
        assert_eq!(fields.len(), 6, "unexpected event info line: {info}");
        scales.push((
            fields[3].parse().expect("SCALUP"),
            fields[5].parse().expect("AQCDUP"),
        ));
    }
    assert!(!scales.is_empty(), "no events in {}", lhe.display());
    scales
}

/// Runs whose per-event `SCALUP` *is* the renormalisation scale, so that
/// `AQCDUP = αs(SCALUP)` event by event.
///
/// `SCALUP` is not the renormalisation scale by construction: `unwgt.f:686`
/// fills it with `sqrt(max(q2fact(1), q2fact(2)))`, the larger
/// **factorisation** scale. It doubles as `μR` only where MadGraph's
/// clustering reads both off the same vertex, which is what the runs listed
/// here have in common.
///
/// The two `2 → 6` runs are where the two part company, and they miss by up
/// to 9% — four orders of magnitude outside the printing budget, so the
/// partition is a measurement rather than a judgement call. Inverting this
/// evolution against their `AQCDUP` puts `μR` between `0.50` and `1.00` of the
/// `SCALUP` they print, and exactly the events at the top of that range —
/// 1237 of `bbx_to_ccx_emmm_qcd0`'s 10000, the ones whose `SCALUP` is the full
/// `√ŝ = 500` — are the ones that do reproduce `AQCDUP`. That evidence is kept
/// running in `validate_scales.rs`
/// (`scalup_is_not_the_renormalisation_scale`), which also derives the scale
/// for most of the runs listed here from the momenta rather than from a
/// printed field, and reaches the same `AQCDUP`.
///
/// Recovering a `2 → 6` scale needs the general kT clustering of `cluster.f`,
/// which `coupling::scales` refuses rather than approximates, so the two stay
/// outside. The partition is asserted rather than assumed, so the day it
/// changes is a test failure and not a silent reclassification.
///
/// The four `lhapdf` runs are here on the strength of a *different* source:
/// their `αs` comes from the PDF set's own table, not from this crate's
/// evolution. The parameter card's value would reproduce none of their events —
/// see [`banked_run_logs_pin_the_alpha_s_source_rule`], which asserts the two
/// sources stay far enough apart for these events to tell them apart.
///
/// Two of the four fix `μR` at `M_Z`, which sits `2.4e-5` of the way into its
/// knot interval; the other two carry a per-event scale and so land mid-interval,
/// where the shape of the interpolation between knots is what decides the printed
/// digits. A straight line through the same knots reproduces the first pair and
/// misses the second by up to `1.7e-4` relative — a thousand times the printing
/// budget — so those two are what pin the interpolant to LHAPDF's cubic.
const SCALUP_IS_THE_RENORMALISATION_SCALE: &[&str] = &[
    "ddx_to_epemg",
    "ee_to_ee",
    "ee_to_mumu",
    "ee_to_mumu_tata_qcd0",
    "ee_to_mumua",
    "ee_to_tatah",
    "ee_to_ttx",
    "ee_to_wpwm",
    "ee_to_zh",
    "gg_to_gg",
    "gg_to_ttx",
    "gu_to_epemu",
    "gux_to_epemux",
    "pp_to_bb",
    "pp_to_bb_fixed",
    "pp_to_bb_qcd2",
    "pp_to_jj",
    "pp_to_ll",
    "pp_to_ll_qcd0",
    "pp_to_llj",
    "pp_to_llj_dyn",
    "pp_to_llj_fixed",
    "pp_to_ll_scalefact2",
    "ud_to_epemud_qcd0",
    "uux_to_epemg",
    "uux_to_mumu",
    "uux_to_uux",
];

/// Half a unit in the last of `v`'s seven printed significant digits.
///
/// The `<event>` line is written by `rw_events.f` as `(i2,i5,e16.7e3,3e15.7)`,
/// a wider field than the value behind it — in all 10k events of every banked
/// run the ninth and tenth digits of `SCALUP` and `AQCDUP` are `0`, so
/// `1.01377900e-01` carries seven digits of information and two of padding.
fn printed_half_ulp(v: f64) -> f64 {
    0.5 * 10f64.powf(v.abs().log10().floor() - 6.0)
}

/// The `AQCDUP` field is not quite `αs`.
///
/// `unwgt.f:694` fills it as `g*g/4d0/3.1415926d0`, with π truncated at eight
/// digits, while `g = √(4π·αs)` was built from the full one. The field is
/// therefore `αs · π/3.1415926`, larger by a systematic `1.7e-8` relative —
/// a sixth of the last printed digit, and enough to move the rounding of one
/// event in twenty. Reproducing the field means reproducing the truncation.
fn aqcdup_from_alpha_s(alpha_s: f64) -> f64 {
    // Both literals are spelled as the Fortran spells them: the full one is what
    // built `g`, the truncated one is the divisor `unwgt.f` actually uses, and
    // the gap between them is the effect being reproduced. Writing either as
    // `std::f64::consts::PI` would erase it.
    #[allow(clippy::approx_constant)]
    const PI: f64 = 3.141592653589793;
    #[allow(clippy::approx_constant)]
    const TRUNCATED_PI: f64 = 3.1415926;
    let g = (4.0 * PI * alpha_s).sqrt();
    g * g / 4.0 / TRUNCATED_PI
}

/// The per-event oracle: `AQCDUP` reproduced from `SCALUP` for every event of
/// every banked run whose `SCALUP` is the renormalisation scale.
///
/// Neither field is exact, so the comparison carries a budget derived per
/// event rather than a chosen tolerance: half a unit in `AQCDUP`'s last
/// printed digit, plus however much the field actually moves across
/// `SCALUP`'s own rounding interval — the latter measured by re-evaluating
/// the coupling at the interval's ends, not estimated from a slope. An event
/// outside that budget is a real disagreement.
///
/// The budget is a bound, not a margin: events pile up against it wherever a
/// true value sits near a rounding boundary, so the reported maximum
/// saturating at `0.999` is the bound being tight rather than the gate being
/// close to failing.
#[test]
fn banked_events_reproduce_aqcdup() {
    let runs = banked_runs();
    let mut agreeing_runs: Vec<String> = Vec::new();
    let mut total_events = 0usize;
    let mut worst_fraction = 0.0f64;
    let mut worst_run = String::new();

    for (name, run) in &runs {
        let running = resolve(name, run).source;
        let events = event_scales(run);
        let mut run_worst = 0.0f64;
        let mut outside = 0usize;
        let mut redigitised = 0usize;
        for (scalup, aqcdup) in &events {
            let got = aqcdup_from_alpha_s(running.eval(*scalup));
            let dq = printed_half_ulp(*scalup);
            let spread = (aqcdup_from_alpha_s(running.eval(scalup + dq)) - got)
                .abs()
                .max((aqcdup_from_alpha_s(running.eval(scalup - dq)) - got).abs());
            let budget = printed_half_ulp(*aqcdup) + spread;
            let fraction = (got - aqcdup).abs() / budget;
            if fraction > 1.0 {
                outside += 1;
            }
            if format!("{got:.6e}") == format!("{aqcdup:.6e}") {
                redigitised += 1;
            }
            run_worst = run_worst.max(fraction);
        }
        let origin = match (running.running(), running.grid()) {
            (Some(r), _) => format!("asmz = {}, nloop = {}", r.asmz(), r.nloop().as_i64()),
            (_, Some(g)) => format!("PDF grid, {} knots", g.knots()),
            _ => unreachable!(),
        };
        println!(
            "{name}: {} events, {outside} outside the printing budget, \
             {redigitised} reproducing the printed digits exactly, \
             {origin}, worst {run_worst:.3} of budget",
            events.len(),
        );
        if outside == 0 {
            agreeing_runs.push(name.clone());
            total_events += events.len();
            if run_worst > worst_fraction {
                worst_fraction = run_worst;
                worst_run = name.clone();
            }
        }
    }

    agreeing_runs.sort();
    assert_eq!(
        agreeing_runs,
        present(SCALUP_IS_THE_RENORMALISATION_SCALE, &runs),
        "the set of runs whose AQCDUP is reproduced from SCALUP changed"
    );
    println!(
        "AQCDUP: {total_events} events across {} runs within their printing budget, \
         worst {worst_fraction:.3} of budget (in {worst_run})",
        agreeing_runs.len()
    );
}

/// The `asmz`/`nloop` source rule against MadGraph's own report of it.
///
/// `setrun.f` prints the parameter-card value it recovered from `G` and, when
/// a beam carries a PDF, the value the PDF label replaced it with — both at
/// 17 digits, which round-trip exactly through a double. This is what makes
/// the override observable in the banked data at all: for a `pdlabel` other than
/// `lhapdf` the `AQCDUP` field cannot distinguish the two, because MadGraph's own
/// tooling has already written the PDF's `aS` into the parameter card and the two
/// are numerically equal.
///
/// The `lhapdf` runs are where they differ, and there this reads the set's own
/// knots at `M_Z` against the 17-digit value MadGraph resolved, then asserts the
/// gap to the card's value is wider than half a printed `AQCDUP` digit — without
/// which [`banked_events_reproduce_aqcdup`]'s 20 000 grid-sourced events would
/// agree with either source and pin neither.
///
/// **What it cannot.** The scale dependence of the grid reading: MadGraph prints
/// its resolved `αs` at `M_Z` and at no other scale, so a wrong interpolation
/// away from the reference scale is left entirely to the events' `AQCDUP`
/// budget — six digits rather than seventeen. `91.188` sits `2.4e-5` of the way
/// into its knot interval, close enough to a knot that a straight line through
/// the same knots lands within `1e-8` of the cubic. So what seventeen digits at
/// this one scale pin is the *source*; what pins the interpolation's shape is the
/// two dynamical-scale runs' events, mid-interval and printed to seven.
#[test]
fn banked_run_logs_pin_the_alpha_s_source_rule() {
    let runs = banked_runs();
    let mut checked = 0usize;
    let mut grid_runs: Vec<String> = Vec::new();
    for (name, run) in &runs {
        let Some(log) = find_run_log(run) else {
            continue;
        };
        let text = std::fs::read_to_string(&log).expect("run log");
        let card = RunCard::parse_file(&run.join("Cards/run_card.dat")).expect("run card");
        let resolved = resolve(name, run);

        let has_pdf = card.lpp1 != 0 || card.lpp2 != 0;
        let (from_card_tag, final_tag) = if has_pdf {
            (
                "Old value of alpha_s from param_card:",
                "New value of alpha_s from PDF",
            )
        } else {
            (
                "Value of alpha_s from param_card:",
                "Value of alpha_s from param_card:",
            )
        };

        let from_card = printed_after(&text, from_card_tag)
            .unwrap_or_else(|| panic!("{name}: no '{from_card_tag}' line in {log:?}"));
        assert_eq!(
            resolved.param_card_asmz, from_card,
            "{name}: param-card alpha_s recovered from G disagrees with MadGraph's own report"
        );

        let final_value = printed_after(&text, final_tag)
            .unwrap_or_else(|| panic!("{name}: no '{final_tag}' line in {log:?}"));
        let Some(running) = resolved.source.running() else {
            let grid = resolved.source.grid().expect("one arm or the other");
            // The override is real and not a rounding of the card's own value,
            // so the log line is a reference the grid reading has to land on.
            assert_ne!(
                final_value, from_card,
                "{name}: the PDF grid's alpha_s(M_Z) now equals the parameter card's, so \
                 this run no longer distinguishes the two"
            );
            let rel = (grid.eval(ZMASS) - final_value).abs() / final_value;
            assert!(
                rel <= GRID_ALPHA_S_TOL,
                "{name}: the set's tabulated alpha_s(M_Z) reads {} against MadGraph's \
                 {final_value} (rel {rel:.2e})",
                grid.eval(ZMASS)
            );
            // The swap has to be visible in the printed field too, not only in
            // the 17-digit log line, or the banked events pin no source at all.
            let card_value = RunningAlphaS::new(resolved.param_card_asmz, NLoop::Two)
                .expect("positive asmz")
                .eval(ZMASS);
            let gap = (aqcdup_from_alpha_s(card_value) - aqcdup_from_alpha_s(final_value)).abs();
            let half_ulp = printed_half_ulp(aqcdup_from_alpha_s(final_value));
            assert!(
                gap > half_ulp,
                "{name}: at M_Z the parameter card's alpha_s is no longer distinguishable \
                 from the grid's in a printed AQCDUP ({gap:.2e} against {half_ulp:.2e}), so \
                 nothing pins which source the events used"
            );
            println!(
                "{name}: alpha_s(M_Z) overridden by the PDF grid, {from_card} -> \
                 {final_value}, reproduced from the set's {} knots to {rel:.2e}; \
                 AQCDUP separates the two sources by {:.2} printed half-digits",
                grid.knots(),
                gap / half_ulp
            );
            grid_runs.push(name.clone());
            checked += 1;
            continue;
        };
        assert_eq!(
            running.asmz(),
            final_value,
            "{name}: resolved alpha_s(M_Z) disagrees with MadGraph's own report"
        );

        if !has_pdf {
            assert!(
                text.contains("The default order of alpha_s running is fixed to            2"),
                "{name}: expected the no-PDF path to force two-loop running"
            );
            assert_eq!(running.nloop(), NLoop::Two, "{name}");
        }
        checked += 1;
    }
    assert!(checked >= 5, "only {checked} runs carried a readable log");
    grid_runs.sort();
    assert_eq!(
        grid_runs,
        present(GRID_ALPHA_S_RUNS, &runs),
        "the set of runs whose log reports a PDF-grid alpha_s changed"
    );
    println!("alpha_s source rule pinned against {checked} MadGraph run logs");
}

/// The first `run_01_log.txt` under a run's `SubProcesses/*/G*/`.
fn find_run_log(run: &Path) -> Option<PathBuf> {
    let mut found: Option<PathBuf> = None;
    let subprocesses = std::fs::read_dir(run.join("SubProcesses")).ok()?;
    for p in subprocesses.flatten() {
        let Ok(channels) = std::fs::read_dir(p.path()) else {
            continue;
        };
        for g in channels.flatten() {
            let candidate = g.path().join("run_01_log.txt");
            if candidate.is_file() && found.as_ref().is_none_or(|f| candidate < *f) {
                found = Some(candidate);
            }
        }
    }
    found
}

/// The Fortran-printed double following `tag` in `text`.
fn printed_after(text: &str, tag: &str) -> Option<f64> {
    let line = text.lines().find(|l| l.contains(tag))?;
    let tail = &line[line.find(tag)? + tag.len()..];
    tail.split_whitespace()
        .find_map(|token| token.trim_end_matches(':').parse::<f64>().ok())
}
