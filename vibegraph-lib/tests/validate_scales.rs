//! Validation of the per-event scale choice ([`vibegraph::coupling::scales`])
//! against MadGraph's banked events.
//!
//! # The oracle
//!
//! Every banked run writes 10k events with the scales MadGraph chose for them.
//! Three fields carry them, at two different precisions:
//!
//! * `SCALUP` on the `<event>` line. `unwgt.f:686` fills it as
//!   `sqrt(max(q2fact(1), q2fact(2)))` — the **factorisation** scale, not the
//!   renormalisation scale. The two coincide wherever the clustering reads both
//!   off the same vertex, which is every run below and is why the field passes for
//!   `μR` at all; [`scalup_is_not_the_renormalisation_scale`] shows where it stops.
//! * `<rscale>` inside `<mgrwt>`: `s_scale`, which `reweight.f:1250` sets to
//!   `scale` — `μR` itself, at one more printed digit than `SCALUP`.
//! * `<pdfrwt beam="i">`: `sqrt(q2fact(i))`, so `μF` **per beam**.
//!
//! The `<mgrwt>` block only appears with `use_syst`, which is 6 of the 20 banked
//! runs. The other 14 are pinned by `SCALUP` alone — plus, independently of any
//! scale field, by `AQCDUP`: `αs` at `μR`, which
//! [`banked_events_reproduce_aqcdup_from_the_computed_scale`] recomputes from the
//! scale this crate derives from the momenta rather than from a printed field.
//!
//! # The budget
//!
//! Nothing here is compared to a chosen tolerance. Each event carries a bound
//! built from the precision of the numbers that went into it: the momenta are
//! printed to eleven significant digits and the scale fields to seven or eight, so
//! the bound is the scale's own last-digit rounding plus however far the scale
//! actually moves when each printed momentum component is walked to the ends of
//! its own rounding interval. That last part is not decoration — the transverse
//! mass of a forward leg is `(E − p_z)(E + p_z)`, a difference that cancels four
//! or five digits away, so a `pp → b b̄` event can lose two orders of magnitude of
//! precision before any scale is formed. The budget is a bound and not a margin:
//! events pile up against it wherever the true value sits near a rounding
//! boundary, so a reported worst case near `1.0` means the bound is tight.
//!
//! # What this cannot see
//!
//! * **The geometric-mean structure of the coloured two-body scale.** For a
//!   2 → 2 the two outgoing transverse momenta are equal and opposite, so
//!   equal-mass legs carry equal transverse masses and `(djb₃·djb₄)^¼` cannot be
//!   told apart from either leg's own `√djb`. Every banked run with a coloured
//!   final state has equal-mass legs, so the *form* of the mean is unpinned; what
//!   is pinned is that the scale is that common transverse mass.
//! * **`scalefact`.** Every banked run has `scalefact = 1`, so where MadGraph
//!   applies it — and the one place it applies it twice — is pinned only by the
//!   unit tests in the module, against a reading of the Fortran.
//! * **Whether a fixed scale is right for the *reason* it is right.** One banked
//!   run (`pp_to_llj_fixed`) pins all three scales at `m_Z`, so its replay
//!   confirms that the fixed branch reaches every printed field and that the
//!   run-card constant is the one that lands there — but a constant cannot
//!   distinguish `μR` from `μF`, and no perturbation of the momenta can move it,
//!   so the run says nothing about the kinematic dependence the other runs pin.
//!   Its value is the complementary one: it is the same `p p → l+ l- j` process
//!   its dynamical siblings run, differing in the three `fixed_*_scale` booleans
//!   alone, so it separates the prescription from the process. The
//!   `dy13_*_run_card.dat` cards the hadronic cross-section
//!   reference was generated with are asserted to still compile to the constants
//!   that reference assumed. That assertion, and the rest of what the committed
//!   cards compile to, is `scales_run_cards.rs` — no events, so it runs on a bare
//!   clone.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use vibegraph::coupling::alphas::{asmz_from_param_card, AlphaSSource, RunningAlphaS};
use vibegraph::coupling::cluster::configs::{derive_channels_permuted, DerivedChannels};
use vibegraph::coupling::cluster::graph::ColorTable;
use vibegraph::coupling::scales::{
    ClusterInput, DynamicalChoice, ScaleChoice, ScaleError, ScaleEvent,
};
use vibegraph::runcard::RunCard;
use vibegraph::ufo::particles::ParticleId;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::EvaluatedModel;

mod common;

fn output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/output")
}

/// What a banked run's card asks the scale prescription for.
enum Coverage {
    /// The card leaves at least one scale dynamic at `dynamical_scale_choice =
    /// -1`, so every event is clustered.
    Clustered,
    /// The card fixes every scale, so no clustering enters and the printed
    /// fields are run-card constants.
    Fixed,
}

fn coverage(run: &str) -> Coverage {
    if FIXED_SCALE_RUNS.contains(&run) {
        return Coverage::Fixed;
    }
    if CLUSTERED_RUNS.contains(&run) || UNREPLAYABLE_RUNS.contains(&run) {
        return Coverage::Clustered;
    }
    panic!(
        "banked run {run} is in none of this gate's inventories: add it to CLUSTERED_RUNS, \
         UNREPLAYABLE_RUNS or FIXED_SCALE_RUNS"
    )
}

/// Every run whose scales come out of the clustering, replayed event by event.
const CLUSTERED_RUNS: &[&str] = &[
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
    "pp_to_bb_qcd2",
    "pp_to_jj",
    "pp_to_ll",
    "pp_to_ll_qcd0",
    "pp_to_ll_scalefact2",
    "pp_to_llj",
    "pp_to_llj_dyn",
    "uux_to_epemg",
    "uux_to_mumu",
    "uux_to_uux",
];

/// The two `2 → 6` runs, which are clustered but **not** replayed here.
///
/// Both blockers are properties of the record rather than of the engine, and
/// neither is a tolerance:
///
/// * MadGraph's on-shell flags survive across events: `checkbw` clears them only
///   for the integration channel's own timelike lines, so a leg set another
///   channel flagged keeps its flag into the next event. 81 of
///   `bbx_to_ccx_emmm_qcd0`'s events and 163 of `uux_to_ccx_emmm_qcd0`'s take a
///   measure that a flag left by a *previous* event under a different channel
///   set, and no function of one event can produce them.
/// * The scale is not a function of the event: these directories carry 615 and
///   579 integration channels, and the resonance tagging reads the one being
///   integrated. An LHE record does not say which, and searching 615 of them for
///   one that agrees would be a gate that almost anything passes.
///
/// What enforces them instead is finer: `validate_kt_cluster.rs` reproduces all
/// 20 000 of their events — every candidate pair, every merge, both scales —
/// against MadGraph's own instrumented intermediates, given the channel and the
/// carried flags. The scale field is the coarser oracle of the two.
const UNREPLAYABLE_RUNS: &[&str] = &["bbx_to_ccx_emmm_qcd0", "uux_to_ccx_emmm_qcd0"];

/// The runs whose `αs` MadGraph reads out of the PDF grid rather than solving
/// for: with `pdlabel = lhapdf` it links `alfas_functions_lhapdf.f`, whose
/// `ALPHAS(Q)` forwards to LHAPDF's `alphasPDF(Q)`. `RunningAlphaS` refuses
/// those cards, so [`AlphaSSource::from_run_card`] takes its grid arm for
/// them instead of its evolution one — the classification is now of which
/// arm a run takes rather than of which runs a plain `RunningAlphaS` oracle
/// has to skip. [`the_grid_runs_need_the_grids_alpha_s_and_not_the_parameter_cards`]
/// is what keeps the classification from being a convenience: the parameter
/// card's own `αs(M_Z)` is not an interchangeable substitute for the grid's.
const GRID_ALPHA_S_RUNS: &[&str] = &[
    "pp_to_bb_fixed",
    "pp_to_jj",
    "pp_to_llj_dyn",
    "pp_to_llj_fixed",
];

/// The runs whose scales are run-card constants. `pp_to_llj_fixed` is the same
/// process as one of the clustered runs, so the pair is a controlled comparison:
/// what separates the two regimes is the `fixed_*_scale` flags alone, not the
/// final state.
const FIXED_SCALE_RUNS: &[&str] = &["pp_to_bb_fixed", "pp_to_llj_fixed", "ud_to_epemud_qcd0"];

/// `cluster.f`'s inflation of a beam–leg candidate whose legs point in opposite
/// directions, as it reaches the scale: the factor lands on `pt2ijcl`, so a
/// scale that carries it is larger by its square root.
const CROSSING_INFLATION: f64 = 1e-6;

/// Events a run's replay is allowed to miss, and how many.
///
/// This is not a tolerance and not a placeholder count. Each event admitted here
/// has to carry the *signature* below — the printed field is reproduced by the
/// computed scale times exactly the crossing inflation — and the count is
/// asserted for equality, so a population that grows, shrinks, or changes
/// character fails.
///
/// **`pp_to_jj`, 9 events of 10 000.** All are `q q' → q q'` subprocesses whose
/// merge graph has a single integration channel and two allowed beam–leg pairs.
/// MadGraph inflated the winning candidate and this replay did not, and the two
/// scales differ by `√(1 + 1e-6)` and by nothing else — `<rscale>`'s eight digits
/// resolve the ratio to `5.03e-7` and `5.16e-7` against the inflation's
/// `5.000e-7`. Which of two numerically degenerate candidates `cluster.f` chose
/// is decided below the eleven digits the record prints, so the record cannot
/// settle it; the engine's own tie-break is pinned bit-for-bit elsewhere
/// (`uux_to_uux`'s 32 inflated candidates, `validate_kt_cluster.rs`). Settling
/// *this* population needs a clustering dump for `p p → j j`, which the sprint
/// did not bank.
const TIE_BREAK_MISSES: &[(&str, usize)] = &[("pp_to_jj", 9)];

/// The runs whose `scalefact` is not `1`, with the value their card carries.
///
/// One exists, and it is the only thing that pins where MadGraph applies the
/// factor. `reweight.f` on 3.7.1 applies exactly one power to
/// `μR` and to each `μF`; `pp_to_ll_scalefact2` is that reading's oracle.
const SCALEFACT_RUNS: &[(&str, f64)] = &[("pp_to_ll_scalefact2", 2.0)];

/// Every run this gate names, whether or not its directory is on this machine.
fn declared_runs() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = CLUSTERED_RUNS
        .iter()
        .chain(UNREPLAYABLE_RUNS)
        .chain(FIXED_SCALE_RUNS)
        .copied()
        .collect();
    names.sort_unstable();
    let mut unique = names.clone();
    unique.dedup();
    assert_eq!(names, unique, "a run is declared in two inventories");
    names
}

/// The banked runs on this machine, checked against what the gate declares.
///
/// Two failures are distinguished, and the manifest is what distinguishes them.
/// A declared run whose directory is absent is normally an incomplete
/// environment and says so; a run the manifest marks `bundled = false` has
/// artifacts that live in a local work area and are deliberately not in the
/// pinned bundle yet, so a checkout that fetched the bundle and lacks it is
/// complete with respect to what the bundle promises. A directory the gate does
/// not name at all fails either way — silently ignoring a new run is how a gate
/// stops covering what it claims to.
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
            Some((path.file_name()?.to_string_lossy().into_owned(), path))
        })
        .collect();
    runs.sort();

    let declared = declared_runs();
    for (name, _) in &runs {
        assert!(
            declared.contains(&name.as_str()),
            "banked run {name} is in none of this gate's inventories"
        );
    }
    let unbundled = common::manifest::unbundled_rows();
    for name in declared {
        if runs.iter().any(|(present, _)| present == name) || unbundled.contains(name) {
            continue;
        }
        vibegraph::validation::require("scales_gate_replays_madgraph", "a banked run", name);
    }
    runs
}

/// The declared runs of `list` that are on this machine, in the order
/// [`banked_runs`] walks them — what an inventory assertion compares against
/// when some rows are awaiting the bundle.
fn present(list: &[&str], runs: &[(String, PathBuf)]) -> Vec<String> {
    let mut names: Vec<String> = list
        .iter()
        .filter(|name| runs.iter().any(|(present, _)| present == *name))
        .map(|name| (*name).to_string())
        .collect();
    names.sort();
    names
}

/// One `<event>`, reduced to what a scale depends on and what MadGraph
/// recorded for it.
#[derive(Clone)]
struct Event {
    incoming: [[f64; 4]; 2],
    outgoing: Vec<[f64; 4]>,
    /// The matrix element's own external flavours, incoming first — which
    /// subprocess of the process definition produced the event.
    flavours: Vec<i64>,
    scalup: f64,
    aqcdup: f64,
    /// `<rscale>`, present only with `use_syst`.
    rscale: Option<f64>,
    /// `<pdfrwt beam="1">` and `beam="2"` scales, present only with `use_syst`.
    pdf_scale: [Option<f64>; 2],
}

fn parse_events(run: &Path) -> Vec<Event> {
    let lhe = run.join("Events/run_01/unweighted_events.lhe.gz");
    let out = Command::new("gzip")
        .args(["-dc", lhe.to_str().unwrap()])
        .output()
        .expect("gzip -dc");
    assert!(out.status.success(), "gzip failed on {}", lhe.display());
    let text = String::from_utf8_lossy(&out.stdout);

    let mut events = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.trim() != "<event>" {
            continue;
        }
        let info: Vec<&str> = lines
            .next()
            .expect("event info line")
            .split_whitespace()
            .collect();
        let nup: usize = info[0].parse().expect("NUP");
        let scalup: f64 = info[3].parse().expect("SCALUP");
        let aqcdup: f64 = info[5].parse().expect("AQCDUP");

        let mut incoming = Vec::new();
        let mut outgoing = Vec::new();
        let mut flavours_in = Vec::new();
        let mut flavours_out = Vec::new();
        for _ in 0..nup {
            let f: Vec<&str> = lines
                .next()
                .expect("particle line")
                .split_whitespace()
                .collect();
            let status: i32 = f[1].parse().expect("ISTUP");
            let pdg: i64 = f[0].parse().expect("IDUP");
            let p = [
                f[9].parse().expect("E"),
                f[6].parse().expect("px"),
                f[7].parse().expect("py"),
                f[8].parse().expect("pz"),
            ];
            // Status 2 is an intermediate resonance the writer added back in;
            // the matrix element only ever saw the incoming and outgoing legs.
            match status {
                -1 => {
                    incoming.push(p);
                    flavours_in.push(pdg);
                }
                1 => {
                    outgoing.push(p);
                    flavours_out.push(pdg);
                }
                _ => {}
            }
        }
        assert_eq!(incoming.len(), 2, "expected two incoming legs");
        flavours_in.extend(flavours_out);

        let mut rscale = None;
        let mut pdf_scale = [None, None];
        for line in lines.by_ref() {
            let line = line.trim();
            if line == "</event>" {
                break;
            }
            if let Some(body) = tag_body(line, "<rscale>", "</rscale>") {
                // `<rscale> n_qcd  value`
                rscale = Some(fortran_double(
                    body.split_whitespace().nth(1).expect("rscale value"),
                ));
            } else if line.starts_with("<pdfrwt beam=") {
                let beam: usize = line[14..15].parse().expect("beam index");
                let body = tag_body(line, ">", "</pdfrwt>").expect("pdfrwt body");
                let fields: Vec<&str> = body.split_whitespace().collect();
                // `n` flavours, then n x values, then n scales.
                let n: usize = fields[0].parse().expect("n_pdfrw");
                if n > 0 {
                    pdf_scale[beam - 1] = Some(fortran_double(fields[1 + 3 * n - 1]));
                }
            }
        }

        events.push(Event {
            incoming: [incoming[0], incoming[1]],
            outgoing,
            flavours: flavours_in,
            scalup,
            aqcdup,
            rscale,
            pdf_scale,
        });
    }
    assert!(!events.is_empty(), "no events in {}", lhe.display());
    events
}

fn tag_body<'a>(line: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let start = line.find(open)? + open.len();
    let end = line.find(close)?;
    (start <= end).then(|| &line[start..end])
}

/// Fortran `E` exponents, which Rust's parser does not accept.
fn fortran_double(token: &str) -> f64 {
    token.replace(['E', 'D'], "e").parse().expect("double")
}

/// Half a unit in the last of `v`'s `digits` printed significant digits.
///
/// The `<event>` line is written by `rw_events.f` as `(i2,i5,e16.7e3,3e15.7)`,
/// seven significant digits in a nine-digit field; the `<mgrwt>` values come
/// from `unwgt.f`'s `E15.8` and carry eight.
fn printed_half_ulp(v: f64, digits: i32) -> f64 {
    0.5 * 10f64.powf(v.abs().log10().floor() - f64::from(digits - 1))
}

/// Momentum components are printed to eleven significant digits.
const MOMENTUM_DIGITS: i32 = 11;

/// How far the computed scale moves when each printed momentum component is
/// walked to the ends of its own rounding interval, summed over components.
///
/// Measured rather than estimated from a derivative, exactly because the
/// interesting cases are the ones where the derivative is enormous: a forward
/// leg's `(E − p_z)(E + p_z)` cancels most of the digits it was given.
fn momentum_spread(
    scales: &mut dyn FnMut(&[[f64; 4]; 2], &[[f64; 4]]) -> Result<MuTriple, ScaleError>,
    event: &Event,
    base: MuTriple,
) -> MuTriple {
    let mut spread = MuTriple::default();
    let mut outgoing = event.outgoing.clone();
    let mut incoming = event.incoming;
    // The beams are printed inputs too, and the clustering reads them: an
    // initial-state merge measures a leg against the beam it followed and boosts
    // into the frame that leaves. At a hadron collider their components are the
    // largest numbers in the record, so eleven digits leave the coarsest steps.
    for beam in 0..2 {
        for comp in 0..4 {
            let saved = incoming[beam][comp];
            if saved == 0.0 {
                continue;
            }
            let step = printed_half_ulp(saved, MOMENTUM_DIGITS);
            let mut worst = MuTriple::default();
            for shifted in [saved + step, saved - step] {
                incoming[beam][comp] = shifted;
                if let Ok(moved) = scales(&incoming, &outgoing) {
                    worst = worst.max(&base.difference(&moved));
                }
            }
            incoming[beam][comp] = saved;
            spread = spread.sum(&worst);
        }
    }
    for leg in 0..outgoing.len() {
        for comp in 0..4 {
            let saved = outgoing[leg][comp];
            if saved == 0.0 {
                continue;
            }
            let step = printed_half_ulp(saved, MOMENTUM_DIGITS);
            let mut worst = MuTriple::default();
            for shifted in [saved + step, saved - step] {
                outgoing[leg][comp] = shifted;
                if let Ok(moved) = scales(&incoming, &outgoing) {
                    worst = worst.max(&base.difference(&moved));
                }
            }
            outgoing[leg][comp] = saved;
            spread = spread.sum(&worst);
        }
    }
    spread
}

/// `(μR, μF₁, μF₂)`.
#[derive(Clone, Copy, Debug, Default)]
struct MuTriple([f64; 3]);

impl MuTriple {
    fn difference(&self, other: &MuTriple) -> MuTriple {
        MuTriple(std::array::from_fn(|i| (self.0[i] - other.0[i]).abs()))
    }
    fn max(&self, other: &MuTriple) -> MuTriple {
        MuTriple(std::array::from_fn(|i| self.0[i].max(other.0[i])))
    }
    fn sum(&self, other: &MuTriple) -> MuTriple {
        MuTriple(std::array::from_fn(|i| self.0[i] + other.0[i]))
    }
}

/// The fixed-scale branch, which reads no kinematics at all.
fn fixed(choice: &ScaleChoice) -> Result<MuTriple, ScaleError> {
    let scales = choice.scales(&ScaleEvent {
        incoming: [[0.0; 4]; 2],
        outgoing: &[],
    })?;
    Ok(MuTriple([scales.mu_r, scales.mu_f[0], scales.mu_f[1]]))
}

fn run_card(run: &Path) -> RunCard {
    RunCard::parse_file(&run.join("Cards/run_card.dat")).expect("run card")
}

/// Every banked run leaves `dynamical_scale_choice` at its default and carries
/// the `scalefact` its row declares, so the replay below cannot be quietly
/// reading a different prescription than the one it claims to validate. The
/// `fixed_*_scale` flags are what separate the two regimes, and each run must
/// land in the regime its [`coverage`] row claims — a run listed in
/// [`FIXED_SCALE_RUNS`] whose card stopped fixing a scale would otherwise
/// silently start taking the clustering branch.
#[test]
fn every_banked_run_uses_the_clustering_default() {
    for (name, run) in banked_runs() {
        let card = run_card(&run);
        let choice = ScaleChoice::from_run_card(&card).expect("compiled");
        assert_eq!(
            choice.choice(),
            DynamicalChoice::Clustered,
            "{name}: dynamical_scale_choice"
        );
        let declared = SCALEFACT_RUNS
            .iter()
            .find(|(run, _)| *run == name)
            .map_or(1.0, |(_, value)| *value);
        assert_eq!(choice.scalefact(), declared, "{name}: scalefact");
        let fixed = FIXED_SCALE_RUNS.contains(&name.as_str());
        assert_eq!(
            choice.is_fully_fixed(),
            fixed,
            "{name}: fixed-scale classification disagrees with the card"
        );
        assert_eq!(
            choice.needs_channels(),
            !fixed,
            "{name}: channel requirement disagrees with the card"
        );
    }
}

/// One event's replay: the three scales, how far each moves across the momenta's
/// own printed rounding, and which integration channel produced them.
struct Replay {
    mu: MuTriple,
    spread: MuTriple,
    /// `1` for a fixed-scale run, which reads no channels at all.
    config: usize,
}

/// Replay one event, choosing the integration channel the LHE record does not
/// carry.
///
/// MadGraph's clustering scale is a function of the event *and* of the channel
/// being integrated — the coupling-order filter on the merge table, the
/// resonance tagging and the jet-count memo all read it, and an LHE record says
/// nothing about it. The channel adopted here
/// is the first one whose factorisation scale lands inside `SCALUP`'s own
/// printing budget; every other field of the event, and the `AQCDUP` oracle in
/// the next test, is then read off that same channel rather than off a per-field
/// best. So a wrong clustering cannot be repaired field by field: one channel has
/// to reproduce all of them.
fn replay(choice: &ScaleChoice, channels: Option<&Channels>, event: &Event) -> Replay {
    let Some(channels) = channels else {
        let mu = fixed(choice).expect("a fixed-scale card resolves without an event");
        return Replay {
            mu,
            spread: MuTriple::default(),
            config: 1,
        };
    };
    let n_configs = channels.of(event).set.configs.len();
    let mut first: Option<Replay> = None;
    for config in 1..=n_configs {
        let Ok(mu) = general(choice, channels, event, config) else {
            continue;
        };
        let mut scales =
            |incoming: &[[f64; 4]; 2], outgoing: &[[f64; 4]]| -> Result<MuTriple, ScaleError> {
                general(
                    choice,
                    channels,
                    &Event {
                        incoming: *incoming,
                        outgoing: outgoing.to_vec(),
                        ..event.clone()
                    },
                    config,
                )
            };
        let spread = momentum_spread(&mut scales, event, mu);
        let candidate = Replay { mu, spread, config };
        let budget = printed_half_ulp(event.scalup, 7) + spread.0[1].max(spread.0[2]);
        if (mu.0[1].max(mu.0[2]) - event.scalup).abs() <= budget {
            return candidate;
        }
        first.get_or_insert(candidate);
    }
    first.expect("no integration channel produced a scale at all")
}

/// Per-event replay: the scales this crate derives against every scale
/// MadGraph printed, for every event of every banked run.
///
/// The count of events that needed a channel other than the first is reported,
/// because it is the measure of what the missing input is worth: a run where it
/// is zero has a cluster scale that is a function of the event alone.
///
/// The two `2 → 6` runs are outside for a reason that is not a tolerance; see
/// [`UNREPLAYABLE_RUNS`]. `pp_to_jj` carries a small declared exception of its
/// own; see [`TIE_BREAK_MISSES`].
#[test]
fn banked_events_reproduce_every_printed_scale() {
    let runs = banked_runs();
    let mut clustered: Vec<String> = Vec::new();
    let mut fixed_runs: Vec<String> = Vec::new();
    let mut total_events = 0usize;
    let mut total_comparisons = 0usize;
    let mut worst = (0.0f64, String::new(), String::new());

    for (name, run) in &runs {
        let card = run_card(run);
        let choice = ScaleChoice::from_run_card(&card).expect("compiled");
        let events = parse_events(run);
        let channels = match coverage(name) {
            // A fixed scale reads no kinematics, so the replay hands it no
            // channels at all: passing them would let a clustering bug hide
            // behind the constant.
            Coverage::Fixed => {
                fixed_runs.push(name.clone());
                None
            }
            Coverage::Clustered => {
                if UNREPLAYABLE_RUNS.contains(&name.as_str()) {
                    println!(
                        "{name}: {} events, not replayable from an LHE record — enforced \
                         against the instrumented dump instead",
                        events.len()
                    );
                    continue;
                }
                clustered.push(name.clone());
                Some(channels_for(run))
            }
        };

        let mut run_worst = (0.0f64, "all fields".to_string());
        let mut missed: BTreeSet<usize> = BTreeSet::new();
        let mut inflated: BTreeSet<usize> = BTreeSet::new();
        let mut comparisons = 0usize;
        let mut other_channel = 0usize;
        let mut reported = 0usize;
        for (index, event) in events.iter().enumerate() {
            let got = replay(&choice, channels.as_ref(), event);
            if got.config != 1 {
                other_channel += 1;
            }
            for (field, printed, digits, pick, moved) in checks(event, got.mu, got.spread) {
                let budget = printed_half_ulp(printed, digits) + moved;
                let fraction = (pick(got.mu) - printed).abs() / budget;
                comparisons += 1;
                if fraction > 1.0 {
                    missed.insert(index);
                    // The one class of miss this gate admits: the same scale
                    // with `cluster.f`'s crossing inflation on it.
                    let with_inflation = pick(got.mu) * (1.0 + CROSSING_INFLATION).sqrt();
                    if (with_inflation - printed).abs() <= budget {
                        inflated.insert(index);
                    }
                    if reported < 12 {
                        reported += 1;
                        println!(
                            "  {name} event {index} ({:?}) {field}: printed {printed:.9e}, \
                             computed {:.9e}, {fraction:.3} of budget {budget:.3e} under channel \
                             {} of {}",
                            event.flavours,
                            pick(got.mu),
                            got.config,
                            channels
                                .as_ref()
                                .map_or(1, |c| c.of(event).set.configs.len()),
                        );
                    }
                }
                if fraction > run_worst.0 {
                    run_worst = (fraction, field.to_string());
                }
            }
        }

        let allowed = TIE_BREAK_MISSES
            .iter()
            .find(|(run, _)| run == name)
            .map_or(0, |(_, count)| *count);
        assert_eq!(
            missed.len(),
            allowed,
            "{name}: {} of {} events outside the printing budget against a declared {allowed}",
            missed.len(),
            events.len()
        );
        assert_eq!(
            missed.len() - inflated.len(),
            0,
            "{name}: {} of the {} missed events do not carry the crossing inflation, so they \
             are a different failure than the one declared",
            missed.len() - inflated.len(),
            missed.len()
        );
        println!(
            "{name}: {} events, {comparisons} scale comparisons, worst {:.3} of budget (in {}), \
             {other_channel} needing a channel other than the first, {} declared misses",
            events.len(),
            run_worst.0,
            run_worst.1,
            missed.len()
        );
        if run_worst.0 > worst.0 && allowed == 0 {
            worst = (run_worst.0, name.clone(), run_worst.1);
        }
        total_events += events.len();
        total_comparisons += comparisons;
    }

    assert_eq!(
        clustered,
        present(CLUSTERED_RUNS, &runs),
        "the set of runs replayed through the clustering changed"
    );
    assert_eq!(
        fixed_runs,
        present(FIXED_SCALE_RUNS, &runs),
        "the set of runs whose scales are run-card constants changed"
    );
    println!(
        "scales: {total_comparisons} comparisons over {total_events} events in {} runs \
         within their printing budget, worst {:.3} of budget ({} in {}); \
         {} fixed-scale, {} not replayable from an LHE record",
        clustered.len() + fixed_runs.len(),
        worst.0,
        worst.2,
        worst.1,
        fixed_runs.len(),
        UNREPLAYABLE_RUNS.len()
    );
}

/// Each printed scale field of one event, with the digits it carries, which of
/// the three computed scales it is compared against, and how far that scale
/// moves across the momenta's own rounding.
#[allow(clippy::type_complexity)]
fn checks(
    event: &Event,
    base: MuTriple,
    spread: MuTriple,
) -> Vec<(&'static str, f64, i32, fn(MuTriple) -> f64, f64)> {
    // SCALUP is sqrt(max(q2fact)), so it is compared against whichever
    // factorisation scale is larger; mu_R rides along wherever the clustering
    // assigns both from the same vertex.
    let mu_f_max: fn(MuTriple) -> f64 = |mu| mu.0[1].max(mu.0[2]);
    let mu_r: fn(MuTriple) -> f64 = |mu| mu.0[0];
    let mu_f1: fn(MuTriple) -> f64 = |mu| mu.0[1];
    let mu_f2: fn(MuTriple) -> f64 = |mu| mu.0[2];
    let _ = base;
    let mut checks = vec![
        (
            "SCALUP vs mu_F",
            event.scalup,
            7,
            mu_f_max,
            spread.0[1].max(spread.0[2]),
        ),
        ("SCALUP vs mu_R", event.scalup, 7, mu_r, spread.0[0]),
    ];
    if let Some(rscale) = event.rscale {
        checks.push(("rscale", rscale, 8, mu_r, spread.0[0]));
    }
    if let Some(q) = event.pdf_scale[0] {
        checks.push(("pdfrwt beam 1", q, 8, mu_f1, spread.0[1]));
    }
    if let Some(q) = event.pdf_scale[1] {
        checks.push(("pdfrwt beam 2", q, 8, mu_f2, spread.0[2]));
    }
    checks
}

/// A second oracle for `μR` that does not read a scale field at all.
///
/// `AQCDUP` is `αs(μR)`, and `dαs/αs ≈ −0.1 · dQ/Q` at these scales, so its
/// seven printed digits locate `μR` to about `1e-6` relative — tighter than
/// `SCALUP`'s own rounding. Feeding the scale computed from the momenta through
/// [`RunningAlphaS`] and landing inside the field's printing budget therefore
/// confirms the scale independently of the field that names it.
///
/// Runs listed in [`GRID_ALPHA_S_RUNS`] take part through the grid arm of
/// [`AlphaSSource`] rather than the evolution: the reading their beams' PDF
/// set was fitted with, which closes cluster scale → `μR` → `αs(μR)` in one
/// per-event comparison for them too instead of skipping them.
#[test]
fn banked_events_reproduce_aqcdup_from_the_computed_scale() {
    let mut runs_checked = 0usize;
    let mut events_checked = 0usize;
    let mut worst = (0.0f64, String::new());

    let runs = banked_runs();
    for (name, run) in &runs {
        let channels = match coverage(name) {
            Coverage::Fixed => None,
            Coverage::Clustered if UNREPLAYABLE_RUNS.contains(&name.as_str()) => continue,
            Coverage::Clustered => Some(channels_for(run)),
        };
        let card = run_card(run);
        let choice = ScaleChoice::from_run_card(&card).expect("compiled");
        let params = ParamCard::from_file(&run.join("Cards/param_card.dat")).expect("param card");
        let a_s = params.get("sminputs", &[3]).expect("aS in SMINPUTS");
        let source = AlphaSSource::from_run_card(
            &card,
            a_s,
            common::pdfset::set_alpha_s_info(&card).as_ref(),
        )
        .unwrap_or_else(|e| panic!("{name}: alpha_s source: {e}"));

        let mut run_worst = 0.0f64;
        let mut outside = 0usize;
        for event in parse_events(run).iter() {
            // The same channel the scale replay adopted, so this is a second
            // oracle on that choice rather than a second search.
            let got = replay(&choice, channels.as_ref(), event);
            let mu_r = got.mu.0[0];
            let spread = got.spread.0[0];
            let got = aqcdup_from_alpha_s(source.eval(mu_r));
            let moved = [mu_r + spread, mu_r - spread]
                .into_iter()
                .map(|q| (aqcdup_from_alpha_s(source.eval(q)) - got).abs())
                .fold(0.0f64, f64::max);
            let budget = printed_half_ulp(event.aqcdup, 7) + moved;
            let fraction = (got - event.aqcdup).abs() / budget;
            if fraction > 1.0 {
                outside += 1;
            }
            run_worst = run_worst.max(fraction);
            events_checked += 1;
        }
        let allowed = TIE_BREAK_MISSES
            .iter()
            .find(|(run, _)| run == name)
            .map_or(0, |(_, count)| *count);
        assert!(
            outside <= allowed,
            "{name}: {outside} events miss AQCDUP against a declared {allowed}"
        );
        if run_worst > worst.0 {
            worst = (run_worst, name.clone());
        }
        runs_checked += 1;
    }

    // Every declared run except the two the LHE cannot replay and the
    // byte-identical duplicate.
    assert_eq!(
        runs_checked,
        present(CLUSTERED_RUNS, &runs).len() + present(FIXED_SCALE_RUNS, &runs).len()
    );
    println!(
        "AQCDUP from the computed scale: {events_checked} events across {runs_checked} runs, \
         worst {:.3} of budget (in {})",
        worst.0, worst.1
    );
}

/// The two arms of [`AlphaSSource`] are not interchangeable on
/// [`GRID_ALPHA_S_RUNS`], which is the negative control that makes
/// [`banked_events_reproduce_aqcdup_from_the_computed_scale`]'s agreement on
/// them informative rather than a foregone conclusion.
///
/// Those runs fix `μR` at the run card's `scale`, which sits within `1e-5`
/// relative of `M_Z`, so any evolution from `M_Z` to `μR` is negligible at the
/// field's seven digits and `AQCDUP` is essentially `αs(M_Z)` as the run used
/// it. Substituting the parameter card's `αs(M_Z)` — the only value available
/// without the grid's own running — therefore isolates the grid's override,
/// and it misses the printed field by well over its printing budget on every
/// event. Adopting the parameter-card value as an approximation would be a
/// visible error, which is why the two arms stay separate rather than one
/// standing in for the other.
#[test]
fn the_grid_runs_need_the_grids_alpha_s_and_not_the_parameter_cards() {
    let runs = banked_runs();
    // The argument below reads `AQCDUP` as `αs(M_Z)`, which is only true where
    // the card fixes `μR` there; a grid run at a dynamical scale is separated
    // from the parameter card by the events of the fixed ones.
    let pinning: Vec<String> = present(GRID_ALPHA_S_RUNS, &runs)
        .into_iter()
        .filter(|name| FIXED_SCALE_RUNS.contains(&name.as_str()))
        .collect();
    let mut checked = 0usize;
    for name in &pinning {
        let run = output_dir().join(name);
        let card = run_card(&run);
        let params = ParamCard::from_file(&run.join("Cards/param_card.dat")).expect("param card");
        let a_s = params.get("sminputs", &[3]).expect("aS in SMINPUTS");
        assert!(
            matches!(
                RunningAlphaS::from_run_card(&card, a_s),
                Err(vibegraph::coupling::alphas::AlphaSError::LhapdfRunning { .. })
            ),
            "{name}: expected the grid-running refusal"
        );
        assert!(
            (card.scale / 91.1876 - 1.0).abs() < 1e-4,
            "{name}: mu_R is no longer close enough to M_Z for this argument"
        );

        let from_param_card = aqcdup_from_alpha_s(asmz_from_param_card(a_s));
        let mut worst = 0.0f64;
        let mut redigitised = 0usize;
        let events = parse_events(&run);
        for event in &events {
            let budget = printed_half_ulp(event.aqcdup, 7);
            worst = worst.max((from_param_card - event.aqcdup).abs() / budget);
            if format!("{from_param_card:.6e}") == format!("{:.6e}", event.aqcdup) {
                redigitised += 1;
            }
        }
        assert!(
            worst > 1.0 && redigitised == 0,
            "{name}: the parameter card's alpha_s reproduces AQCDUP to {worst:.2} of budget \
             ({redigitised} events digit-exact), so the grid override may no longer be \
             observable here"
        );
        println!(
            "{name}: AQCDUP over {} events misses the parameter card's alpha_s by up to \
             {worst:.1}x its printing budget, on none of which the printed digits agree \
             (param card {from_param_card:.9e} vs printed {:.9e})",
            events.len(),
            events[0].aqcdup
        );
        checked += 1;
    }
    assert_eq!(checked, pinning.len());
    assert!(
        checked > 0,
        "no fixed-scale grid run is on this machine to pin the source"
    );
}

/// What an integrand pays for naming the first integration channel on every
/// event, in the coupling rather than in a count.
///
/// The cluster scale is a function of the event **and** of the integration
/// channel, and an integrand that samples a channel but does not tell the scale
/// prescription which one gets channel 1 on every point. How often that choice
/// changes the scale at all is already reported per run by
/// [`banked_events_reproduce_every_printed_scale`]; what it *costs* is not a
/// count, because a cross section reads the scale only through `αs(μR)`.
///
/// So this compares `αs` at the channel-1 scale against `AQCDUP` — MadGraph's own
/// coupling for that event, at seven printed digits — over every event of every
/// clustered run whose `αs` comes from the evolution rather than from a PDF grid.
/// The mean ratio is the multiplicative bias a cross section linear in `αs`
/// inherits, and it is directly comparable to the σ deviations `validate_sigma`
/// reports.
///
/// Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn probe_first_channel_cost_in_alpha_s() {
    for (name, run) in &banked_runs() {
        if !matches!(coverage(name), Coverage::Clustered)
            || UNREPLAYABLE_RUNS.contains(&name.as_str())
            || GRID_ALPHA_S_RUNS.contains(&name.as_str())
        {
            continue;
        }
        let card = run_card(run);
        let choice = ScaleChoice::from_run_card(&card).expect("compiled");
        let params = ParamCard::from_file(&run.join("Cards/param_card.dat")).expect("param card");
        let a_s = params.get("sminputs", &[3]).expect("aS in SMINPUTS");
        let Ok(running) = RunningAlphaS::from_run_card(&card, a_s) else {
            continue;
        };
        let channels = channels_for(run);
        let events = parse_events(run);
        let mut moved = 0usize;
        let mut sum = 0.0f64;
        let mut worst: f64 = 0.0;
        for event in &events {
            let adopted = replay(&choice, Some(&channels), event).config;
            if adopted != 1 {
                moved += 1;
            }
            let mu_r = general(&choice, &channels, event, 1)
                .expect("channel 1 resolves on a banked event")
                .0[0];
            let ratio = aqcdup_from_alpha_s(running.eval(mu_r)) / event.aqcdup;
            sum += ratio;
            worst = worst.max((ratio - 1.0).abs());
        }
        let n = events.len() as f64;
        println!(
            "{name}: {moved} of {} events need a channel other than the first; \
             alpha_s at the channel-1 scale is {:+.3e} relative to AQCDUP on average, \
             worst {:.3e}",
            events.len(),
            sum / n - 1.0,
            worst
        );
    }
}

/// `unwgt.f:694` fills `AQCDUP` as `g*g/4d0/3.1415926d0`, with π truncated at
/// eight digits while `g` was built from the full one — a systematic `1.7e-8`
/// relative that is a sixth of the field's last printed digit.
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

/// The measured evidence for the refusal, and for `SCALUP` being the
/// factorisation scale rather than the renormalisation one.
///
/// In the two `2 → 6` runs the clustering assigns `μR` off a different vertex
/// than `μF`, and `SCALUP` follows `μF`. The `AQCDUP` field is the witness:
/// evaluating `αs` at the printed `SCALUP` misses it by up to `9%`, five orders
/// of magnitude outside its printing budget, and inverting `αs` instead
/// recovers a `μR` well below `SCALUP` on most events. This is what makes the
/// partition in `validate_alphas.rs` a measurement rather than a convention.
#[test]
fn scalup_is_not_the_renormalisation_scale() {
    let mut checked = 0usize;
    for run_name in ["bbx_to_ccx_emmm_qcd0", "uux_to_ccx_emmm_qcd0"] {
        let run = output_dir().join(run_name);
        let card = run_card(&run);
        let params = ParamCard::from_file(&run.join("Cards/param_card.dat")).expect("param card");
        let a_s = params.get("sminputs", &[3]).expect("aS in SMINPUTS");
        let running = RunningAlphaS::from_run_card(&card, a_s).expect("supported alpha_s");

        let events = parse_events(&run);
        let mut agreeing = 0usize;
        let mut worst_relative = 0.0f64;
        let mut ratio_min = f64::INFINITY;
        let mut ratio_max = 0.0f64;
        for event in &events {
            let at_scalup = aqcdup_from_alpha_s(running.eval(event.scalup));
            let relative = (at_scalup - event.aqcdup).abs() / event.aqcdup;
            if relative <= printed_half_ulp(event.aqcdup, 7) / event.aqcdup {
                agreeing += 1;
            } else {
                worst_relative = worst_relative.max(relative);
            }
            let ratio = invert_alpha_s(&running, event.aqcdup) / event.scalup;
            ratio_min = ratio_min.min(ratio);
            ratio_max = ratio_max.max(ratio);
        }
        assert!(
            agreeing < events.len() / 2,
            "{run_name}: SCALUP now reproduces AQCDUP for {agreeing} of {} events, so the \
             renormalisation scale may have become recoverable from it",
            events.len()
        );
        assert!(
            worst_relative > 1e-2,
            "{run_name}: worst AQCDUP mismatch from SCALUP is only {worst_relative:.2e}"
        );
        println!(
            "{run_name}: {agreeing} of {} events have AQCDUP = alpha_s(SCALUP) (worst miss \
             {worst_relative:.2e} relative); mu_R inverted from AQCDUP spans \
             {ratio_min:.3}-{ratio_max:.3} of SCALUP",
            events.len()
        );
        checked += 1;
    }
    assert_eq!(checked, 2);
}

/// `μR` recovered from `αs(μR)` by bisection, which is monotone over the range
/// the banked events cover.
fn invert_alpha_s(running: &RunningAlphaS, aqcdup: f64) -> f64 {
    // The same truncated pi `unwgt.f` divided by, undone here.
    #[allow(clippy::approx_constant)]
    const TRUNCATED_PI: f64 = 3.1415926;
    let target = aqcdup * TRUNCATED_PI / std::f64::consts::PI;
    let (mut lo, mut hi) = (1.0f64, 1.0e4f64);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if running.eval(mid) > target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

// ── the general clustering path, per run ─────────────────────────────────────

/// One run's process, enumerated the way an integrand enumerates it: every
/// subprocess of its `proc_card_mg5.dat`, with the channel forests each one's
/// diagrams imply.
///
/// A banked event names its own subprocess by its external flavours, so the
/// replay looks the event's flavour tuple up here rather than assuming one. A
/// tuple the enumeration does not carry is a real disagreement about the process
/// definition and fails; it is not filled in with a neighbour.
struct Channels {
    colors: ColorTable,
    by_flavour: BTreeMap<Vec<i64>, DerivedChannels>,
}

impl Channels {
    fn of(&self, event: &Event) -> &DerivedChannels {
        self.by_flavour.get(&event.flavours).unwrap_or_else(|| {
            panic!(
                "no enumerated subprocess with external flavours {:?}",
                event.flavours
            )
        })
    }
}

fn channels_for(run: &Path) -> Channels {
    let model = common::sm_model();
    let params = ParamCard::from_file(&run.join("Cards/param_card.dat")).expect("param card");
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &params);
    let card = run_card(run);
    let proc = vibegraph::diagrams::parse_proc_card_file(
        &run.join("Cards/proc_card_mg5.dat"),
        &vibegraph::diagrams::ParsingOptions::default(),
    )
    .expect("proc card");
    let sets = vibegraph::diagrams::generate_from_proc_card(&proc, model.as_ref())
        .expect("enumerate the process");

    let mut by_flavour = BTreeMap::new();
    for set in &sets {
        if set.diagrams.is_empty() {
            continue;
        }
        let externals: Vec<ParticleId> = set
            .particles_in
            .iter()
            .chain(set.particles_out.iter())
            .map(|name| model.particle_id(name).expect("external in model"))
            .collect();
        let flavours: Vec<i64> = externals
            .iter()
            .map(|&id| model.particle(id).pdg_code)
            .collect();
        let n_in = set.particles_in.len();
        // MadGraph generates each ordering of the initial state as its own
        // subprocess directory and our enumeration produces one of them, so the
        // crossed ordering is registered as the same diagrams read with legs 1
        // and 2 exchanged. Exchanging two entries is its own inverse, so the
        // same vector reads as the leg-to-position map and as the external
        // state in position order.
        let mut positions: Vec<usize> = (0..externals.len()).collect();
        for _ in 0..2 {
            let ordered: Vec<ParticleId> = positions.iter().map(|&p| externals[p]).collect();
            let key: Vec<i64> = ordered
                .iter()
                .map(|&id| model.particle(id).pdg_code)
                .collect();
            let derived = derive_channels_permuted(
                &set.diagrams,
                &ordered,
                n_in,
                &positions,
                model.as_ref(),
                &evaluated,
            )
            .expect("channel forests");
            by_flavour.insert(key, derived);
            if flavours[0] == flavours[1] {
                break;
            }
            positions.swap(0, 1);
        }
    }
    Channels {
        colors: ColorTable::new(
            model
                .particles
                .values()
                .map(|p| (p.pdg_code, p.color))
                .collect::<Vec<(i64, i32)>>(),
            card.maxjetflavor,
        ),
        by_flavour,
    }
}

/// The scales the general path derives for one event, under one integration
/// channel.
fn general(
    choice: &ScaleChoice,
    channels: &Channels,
    event: &Event,
    config: usize,
) -> Result<MuTriple, ScaleError> {
    let derived = channels.of(event);
    let scales = choice.cluster_scales(
        &ScaleEvent {
            incoming: event.incoming,
            outgoing: &event.outgoing,
        },
        &ClusterInput {
            set: &derived.set,
            colors: &channels.colors,
            this_config: config,
            iproc: 1,
        },
    )?;
    Ok(MuTriple([scales.mu_r, scales.mu_f[0], scales.mu_f[1]]))
}

/// The beam-crossing tie-break, as a population rather than as a formula.
///
/// `u ū → u ū` is the one banked run where the inflation `cluster.f` puts on a
/// crossed beam–leg candidate reaches the scale: flavour locks each outgoing leg
/// to the beam of its own flavour, so both allowed candidates can be crossed at
/// once, and a colour line runs from beam to beam to carry the result out. The
/// events it moves sit at `250.000125` against the run's `250` — the seventh
/// digit, which is exactly why a replay that lost the branch would still look
/// right on nine thousand nine hundred and eighty-four events.
///
/// K2's instrumented dump counts 16 such events, and the general path has to
/// keep all 16 and invent none. That count is the standing check on the branch;
/// `coupling::scales`'s own tests pin the value it produces.
#[test]
fn the_general_path_keeps_the_beam_crossing_population() {
    let run = output_dir().join("uux_to_uux");
    if !run.join("Cards/run_card.dat").exists() {
        vibegraph::validation::require(
            "scales_gate_replays_madgraph",
            "a banked run",
            "uux_to_uux",
        );
    }
    let choice = ScaleChoice::from_run_card(&run_card(&run)).expect("compiled");
    let channels = channels_for(&run);
    // The partonic beam energy, which is what the uninflated clustering returns.
    const CORE: f64 = 250.0;
    let mut inflated = 0usize;
    let mut worst = 0.0f64;
    for event in parse_events(&run).iter() {
        let mu = replay(&choice, Some(&channels), event).mu.0[0];
        if mu > CORE * (1.0 + 1e-9) {
            inflated += 1;
            worst = worst.max(mu);
            // Nothing else moves this scale: the inflation is the whole of the
            // difference from the core.
            assert!(
                (mu / (CORE * (1.0 + CROSSING_INFLATION).sqrt()) - 1.0).abs() < 1e-12,
                "an event above the core scale that the crossing inflation does not explain: \
                 {mu:.12}"
            );
        } else {
            assert!((mu / CORE - 1.0).abs() < 1e-12, "{mu:.12}");
        }
    }
    assert_eq!(
        inflated, 16,
        "the beam-crossing tie-break moved a different number of uux_to_uux events than the \
         instrumented dump counted"
    );
    println!(
        "uux_to_uux: {inflated} events carry the beam-crossing inflation, at {worst:.9} against \
         the core's {CORE}"
    );
}
