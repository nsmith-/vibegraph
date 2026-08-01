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
//! a per-event oracle over 180k events rather than a single scalar. Choosing the
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
//! would from the card. [`the_grid_alpha_s_reproduces_the_scale_its_run_log_prints`]
//! adds the 17-digit log line at the run's own scale, and states what the pair can
//! and cannot distinguish.

use std::path::{Path, PathBuf};
use std::process::Command;

use vibegraph::coupling::alphas::{
    asmz_from_param_card, AlphaSSource, NLoop, RunningAlphaS, ZMASS,
};
use vibegraph::pdf::grid::AlphaSInfo;
use vibegraph::pdf::PdfSet;
use vibegraph::runcard::RunCard;
use vibegraph::ufo::slha::ParamCard;

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
const GRID_ALPHA_S_RUNS: &[&str] = &["pp_to_bb_fixed", "pp_to_llj_fixed"];

/// The LHAPDF set each `lhaid` names. A run card carries only the id, and the
/// grid `αs` has to come from the set the *densities* come from, so the mapping
/// is stated here rather than inferred from whatever set happens to be unpacked.
const PDF_SET_BY_LHAID: &[(i64, &str)] = &[(247000, "NNPDF23_lo_as_0130_qed")];

/// How far the log-linear reading of the set's `αs` knots is allowed to sit from
/// the value MadGraph's LHAPDF call returned.
///
/// This is not a numerical-noise budget: LHAPDF interpolates the same knots with
/// a cubic (`AlphaS_Type: ipol`) and this reads them with a straight line, so the
/// residual is a property of where the scale sits in its knot interval. At
/// `Q = 91.188`, `2.4e-5` of the way into `[91.1876, 109.8541]`, it is `1.0e-8`;
/// mid-interval it would be `~1.7e-4` and this bound would (rightly) fail.
/// Tightening it would therefore pin the knot spacing rather than the source.
const GRID_ALPHA_S_TOL: f64 = 1e-7;

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
/// `pp_to_bb_fixed` and `pp_to_llj_fixed` are here on the strength of a
/// *different* source: their `αs` comes from the PDF set's own table, not from
/// this crate's evolution. All 10 000 events of each reproduce the printed
/// `AQCDUP` digits, and the parameter card's value would reproduce none of them —
/// see [`the_grid_alpha_s_reproduces_the_scale_its_run_log_prints`].
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
    "pp_to_ll",
    "pp_to_ll_qcd0",
    "pp_to_llj",
    "pp_to_llj_fixed",
    "pp_to_llj_qcd2_qed2",
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
/// close to failing. The runs it excludes miss by `1.7e5` times the budget.
#[test]
fn banked_events_reproduce_aqcdup() {
    let mut agreeing_runs: Vec<String> = Vec::new();
    let mut total_events = 0usize;
    let mut worst_fraction = 0.0f64;
    let mut worst_run = String::new();

    for (name, run) in banked_runs() {
        let running = resolve(&name, &run).source;
        let events = event_scales(&run);
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

    assert_eq!(
        agreeing_runs, SCALUP_IS_THE_RENORMALISATION_SCALE,
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
/// the override observable in the banked data at all: the `AQCDUP` field
/// cannot distinguish the two, because they are numerically equal in every
/// banked run.
#[test]
fn banked_run_logs_pin_the_alpha_s_source_rule() {
    let mut checked = 0usize;
    for (name, run) in banked_runs() {
        let Some(log) = find_run_log(&run) else {
            continue;
        };
        let text = std::fs::read_to_string(&log).expect("run log");
        let card = RunCard::parse_file(&run.join("Cards/run_card.dat")).expect("run card");
        let resolved = resolve(&name, &run);

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
            println!(
                "{name}: alpha_s(M_Z) overridden by the PDF grid, {from_card} -> \
                 {final_value}, reproduced from the set's knots to {rel:.2e}"
            );
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
    println!("alpha_s source rule pinned against {checked} MadGraph run logs");
}

/// `αs` at the scale a grid-sourced run actually evaluated it at, against the
/// value that run's log prints.
///
/// `alfas_functions_lhapdf.f` prints one `alpha_s for scale Q is V` line per
/// distinct scale, at 17 digits — the same round-tripping precision as the
/// `M_Z` line, but taken at the run's own `μR` rather than at the reference
/// scale. Reading the set's tabulated knots has to land on it.
///
/// **What this pins.** The *source*: the PDF set's table versus the parameter
/// card's `aS`. The two differ by `1.1e-5` relative here, which is `2×` the
/// `AQCDUP` printing budget, so the choice is observable in the banked events —
/// and [`banked_events_reproduce_aqcdup`] observes it, 10 000 times.
///
/// **What it cannot.** The scale dependence: this run fixes `μR = 91.188`, which
/// is `M_Z`, so evaluating at `μR` and evaluating at the reference scale give the
/// same number and nothing here separates them. Nor the interpolation *shape* —
/// `91.188` sits `2.4e-5` of the way into its knot interval, where a linear and a
/// cubic reading of the same knots agree to `1e-8`. A dynamical scale would need
/// both, and would need LHAPDF's cubic `ipol` to have an oracle at all.
#[test]
fn the_grid_alpha_s_reproduces_the_scale_its_run_log_prints() {
    let mut checked = 0usize;
    for (name, run) in banked_runs() {
        let resolved = resolve(&name, &run);
        let Some(grid) = resolved.source.grid() else {
            continue;
        };
        let log = find_run_log(&run).unwrap_or_else(|| panic!("{name}: no readable run log"));
        let text = std::fs::read_to_string(&log).expect("run log");
        let evaluations = alpha_s_evaluations(&text);
        assert!(
            !evaluations.is_empty(),
            "{name}: the log prints no 'alpha_s for scale' line to compare against"
        );

        let mut worst = 0.0f64;
        for &(q, printed) in &evaluations {
            let got = grid.eval(q);
            let rel = (got - printed).abs() / printed;
            assert!(
                rel <= GRID_ALPHA_S_TOL,
                "{name}: alpha_s({q}) reads {got} against MadGraph's {printed} \
                 (rel {rel:.2e})"
            );
            worst = worst.max(rel);

            // The parameter card is the source this replaces, and the swap is
            // visible in the printed field rather than only in the 17-digit log.
            let card_value = RunningAlphaS::new(resolved.param_card_asmz, NLoop::Two)
                .expect("positive asmz")
                .eval(q);
            let gap = (aqcdup_from_alpha_s(card_value) - aqcdup_from_alpha_s(printed)).abs();
            assert!(
                gap > printed_half_ulp(aqcdup_from_alpha_s(printed)),
                "{name}: at Q = {q} the parameter card's alpha_s is no longer \
                 distinguishable from the grid's in a printed AQCDUP, so nothing here \
                 pins which source was used"
            );
        }
        println!(
            "{name}: {} tabulated alpha_s scales from the run log, worst {worst:.2e} relative \
             (grid of {} knots)",
            evaluations.len(),
            grid.knots()
        );
        checked += 1;
    }
    assert_eq!(
        checked,
        GRID_ALPHA_S_RUNS.len(),
        "not every grid-sourced run was compared against its log"
    );
}

/// Every `alpha_s for scale Q is V` line of a `pdlabel = lhapdf` run log.
fn alpha_s_evaluations(text: &str) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let Some(tail) = line.split_once("alpha_s for scale").map(|(_, t)| t) else {
            continue;
        };
        let numbers: Vec<f64> = tail
            .split_whitespace()
            .filter_map(|token| token.parse::<f64>().ok())
            .collect();
        assert_eq!(
            numbers.len(),
            2,
            "unexpected 'alpha_s for scale' line: {line}"
        );
        let entry = (numbers[0], numbers[1]);
        if !out.contains(&entry) {
            out.push(entry);
        }
    }
    out
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
