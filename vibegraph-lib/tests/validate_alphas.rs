//! Validation of the running strong coupling ([`vibegraph::coupling::alphas`])
//! against MadGraph.
//!
//! Two oracles, at two different levels.
//!
//! # 1. The Fortran itself (default test suite)
//!
//! [`fortran_reference_is_bit_exact`] replays
//! `validation/alphas/reference.csv`, produced by linking MadGraph's unmodified
//! `Source/alfas_functions.f` against a driver
//! (`pixi run -e madgraph generate-alphas-reference`). Both sides run the same
//! Newton iteration to the same `TOL = 5e-4`, so this is a *bit-for-bit*
//! comparison, not a tolerance: the Newton solve returns a specific iterate, and
//! agreeing on the iterate is a far stronger statement than agreeing on the root
//! it approximates.
//!
//! **What this cannot see.** Nothing about where `asmz` and `nloop` come from —
//! the grid supplies them directly. That is the job of the second oracle and of
//! the unit tests in the module itself.
//!
//! # 2. The banked MadGraph events (extended validation)
//!
//! Every banked run's `unweighted_events.lhe.gz` carries `SCALUP` and `AQCDUP`
//! per `<event>` line: the scale MadGraph chose and the coupling it evaluated
//! there. Feeding the printed `SCALUP` back through this module must reproduce
//! `AQCDUP` to the precision it is printed at, for every event of every run —
//! a per-event oracle over 180k events rather than a single scalar. Choosing the
//! scale is a separate concern and is not exercised here; `SCALUP` is taken as
//! given, which is also why the two `2 → 6` runs sit outside the gate (see
//! [`banked::SCALUP_IS_THE_RENORMALISATION_SCALE`]).
//!
//! Two properties of the `<event>` line have to be reproduced before the
//! comparison means anything, and both were found by chasing residuals that a
//! loose tolerance would have absorbed: the fields carry seven significant
//! digits in a nine-digit field, and `AQCDUP` is `αs` scaled by `π/3.1415926`
//! because `unwgt.f` divides by a truncated π. Neither is visible at 1e-6, and
//! together they are the difference between 95% and 100% of events reproducing
//! the printed digits.
//!
//! **What this cannot see.** *Not* the PDF override, despite it being the
//! failure mode the field was expected to catch. MadGraph's own tooling writes
//! the PDF's `aS` into the parameter card before the run, so in every banked run
//! the parameter card and the PDF label already agree — dropping the override
//! entirely would leave every `AQCDUP` unchanged. What does see it is the
//! MadGraph run log ([`banked_run_logs_pin_the_alpha_s_source_rule`]), which
//! prints the parameter-card value and the PDF value separately, at 17 digits.
//! The two are enough to pin the rule end to end: the log fixes `asmz`, the
//! events fix the evolution from it.

use std::path::{Path, PathBuf};

use vibegraph::coupling::alphas::{NLoop, RunningAlphaS, BMASS, CMASS, ZMASS};

fn validation_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation")
}

/// One row of the Fortran-generated grid.
struct Row {
    asmz: f64,
    nloop: NLoop,
    q: f64,
    alphas: f64,
}

fn load_reference() -> Vec<Row> {
    let path = validation_dir().join("alphas/reference.csv");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "cannot read {}: {e}\nregenerate with: pixi run -e madgraph generate-alphas-reference",
            path.display()
        )
    });
    let mut rows = Vec::new();
    for line in text.lines().skip(1) {
        let fields: Vec<&str> = line.split(',').map(str::trim).collect();
        assert_eq!(fields.len(), 4, "malformed reference row: {line}");
        rows.push(Row {
            asmz: fields[0].parse().expect("asmz"),
            nloop: NLoop::from_i64(fields[1].parse().expect("nloop")).expect("nloop in 1..=3"),
            q: fields[2].parse().expect("q"),
            alphas: fields[3].parse().expect("alphas"),
        });
    }
    assert!(!rows.is_empty(), "reference grid is empty");
    rows
}

/// The grid is only a net if it actually straddles what it claims to. A grid
/// confined to `Q > BMASS` would pass the comparison below while leaving both
/// flavour-threshold branches untested.
#[test]
fn reference_grid_straddles_every_branch() {
    let rows = load_reference();
    let count = |f: &dyn Fn(&Row) -> bool| rows.iter().filter(|r| f(r)).count();

    assert!(count(&|r| r.q < CMASS) >= 4, "no nf = 3 coverage");
    assert!(
        count(&|r| r.q >= CMASS && r.q < BMASS) >= 4,
        "no nf = 4 coverage"
    );
    assert!(
        count(&|r| r.q >= BMASS && r.q < ZMASS) >= 4,
        "no nf = 5 coverage below M_Z"
    );
    assert!(count(&|r| r.q > ZMASS) >= 4, "no coverage above M_Z");
    // The two flavour thresholds are the branch points, so they are sampled from
    // immediately either side: a branch keyed on the wrong comparison would
    // otherwise have a whole decade to hide in.
    for threshold in [CMASS, BMASS] {
        assert!(
            count(&|r| (r.q - threshold).abs() < 1e-6 * threshold && r.q < threshold) >= 1
                && count(&|r| (r.q - threshold).abs() < 1e-6 * threshold && r.q > threshold) >= 1,
            "threshold {threshold} is not sampled from immediately either side"
        );
    }
    assert!(
        count(&|r| r.q == ZMASS) >= 1,
        "the reference scale itself is not sampled"
    );
    for nloop in [NLoop::One, NLoop::Two, NLoop::Three] {
        assert!(
            count(&|r| r.nloop == nloop) >= 4,
            "no coverage at nloop = {}",
            nloop.as_i64()
        );
    }
}

/// Bit-for-bit against MadGraph's own `ALPHAS`.
#[test]
fn fortran_reference_is_bit_exact() {
    let rows = load_reference();
    let mut mismatches = Vec::new();
    let mut worst_ulps = 0i64;

    for row in &rows {
        let running = RunningAlphaS::new(row.asmz, row.nloop).expect("positive asmz");
        let got = running.eval(row.q);
        if got.to_bits() != row.alphas.to_bits() {
            let ulps = (got.to_bits() as i64 - row.alphas.to_bits() as i64).abs();
            worst_ulps = worst_ulps.max(ulps);
            if mismatches.len() < 10 {
                mismatches.push(format!(
                    "asmz={} nloop={} q={}: fortran {:.17e}, rust {:.17e} ({ulps} ulp)",
                    row.asmz,
                    row.nloop.as_i64(),
                    row.q,
                    row.alphas,
                    got
                ));
            }
        }
    }

    assert!(
        mismatches.is_empty(),
        "{} of {} grid points differ (worst {worst_ulps} ulp):\n{}",
        mismatches.len(),
        rows.len(),
        mismatches.join("\n")
    );
    println!("alpha_s grid: {} points bit-exact", rows.len());
}

#[cfg(feature = "extended-validation")]
mod banked {
    use super::*;

    use std::process::Command;

    use vibegraph::coupling::alphas::asmz_from_param_card;
    use vibegraph::runcard::RunCard;
    use vibegraph::ufo::slha::ParamCard;

    fn output_dir() -> PathBuf {
        validation_dir().join("madgraph/output")
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
    /// `alphasPDF(Q)` and not the beta-function solve this file validates. Their
    /// `AQCDUP` is therefore outside this oracle; what stays inside is the
    /// parameter-card half of the source rule, which
    /// [`banked_run_logs_pin_the_alpha_s_source_rule`] still checks.
    const GRID_ALPHA_S_RUNS: &[&str] = &["pp_to_llj_fixed"];

    /// The evolution MadGraph used for `run`, resolved the way `setrun.f` does
    /// from the run card and the parameter card. `None` for a run whose `αs` the
    /// PDF grid supplies; the refusal is asserted against [`GRID_ALPHA_S_RUNS`]
    /// rather than swallowed.
    fn resolve(name: &str, run: &Path) -> Option<RunningAlphaS> {
        let card = RunCard::parse_file(&run.join("Cards/run_card.dat")).expect("run card");
        let params = ParamCard::from_file(&run.join("Cards/param_card.dat")).expect("param card");
        let a_s = params.get("sminputs", &[3]).expect("aS in SMINPUTS");
        match RunningAlphaS::from_run_card(&card, a_s) {
            Ok(running) => Some(running),
            Err(err) => {
                assert!(
                    GRID_ALPHA_S_RUNS.contains(&name),
                    "{name}: unexpected alpha_s source refusal: {err}"
                );
                None
            }
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
        "pp_to_bb",
        "pp_to_bb_qcd2",
        "pp_to_ll",
        "pp_to_ll_qcd0",
        "pp_to_llj",
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
        const PI: f64 = 3.141592653589793;
        let g = (4.0 * PI * alpha_s).sqrt();
        g * g / 4.0 / 3.1415926
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
            let Some(running) = resolve(&name, &run) else {
                println!("{name}: alpha_s supplied by the PDF grid, outside this oracle");
                continue;
            };
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
            println!(
                "{name}: {} events, {outside} outside the printing budget, \
                 {redigitised} reproducing the printed digits exactly, \
                 asmz = {}, nloop = {}, worst {run_worst:.3} of budget",
                events.len(),
                running.asmz(),
                running.nloop().as_i64(),
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
            let params =
                ParamCard::from_file(&run.join("Cards/param_card.dat")).expect("param card");
            let a_s = params.get("sminputs", &[3]).expect("aS in SMINPUTS");
            let running = resolve(&name, &run);

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
                asmz_from_param_card(a_s),
                from_card,
                "{name}: param-card alpha_s recovered from G disagrees with MadGraph's own report"
            );

            let final_value = printed_after(&text, final_tag)
                .unwrap_or_else(|| panic!("{name}: no '{final_tag}' line in {log:?}"));
            let Some(running) = running else {
                // The grid supplies the value the label would otherwise impose.
                // The override is real and not a rounding of the card's own value,
                // so the log line is the reference a grid-running implementation
                // has to land on.
                assert_ne!(
                    final_value, from_card,
                    "{name}: the PDF grid's alpha_s(M_Z) now equals the parameter card's, so \
                     this run no longer distinguishes the two"
                );
                println!(
                    "{name}: alpha_s(M_Z) overridden by the PDF grid, {from_card} -> {final_value}"
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
}
