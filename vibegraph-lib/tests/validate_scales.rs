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
//!   whose dynamical siblings are refused, so it separates the refusal from the
//!   process. The `dy13_*_run_card.dat` cards the hadronic cross-section
//!   reference was generated with are asserted to still compile to the constants
//!   that reference assumed. That assertion, and the rest of what the committed
//!   cards compile to, is `scales_run_cards.rs` — no events, so it runs on a bare
//!   clone.

use std::path::{Path, PathBuf};
use std::process::Command;

use vibegraph::coupling::alphas::{asmz_from_param_card, RunningAlphaS};
use vibegraph::coupling::scales::{
    BeamConnections, ClusterTopology, DynamicalChoice, ScaleChoice, ScaleError, ScaleEvent,
};
use vibegraph::runcard::RunCard;
use vibegraph::ufo::slha::ParamCard;

fn output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/output")
}

/// `g g → g g`: every leg a gluon, so either outgoing leg may follow either
/// beam and the clustering's tie-break never has to fire.
const FREE_JET_PAIR: ClusterTopology = ClusterTopology {
    beam_connections: BeamConnections::TChannel {
        two_body_pairs: [[true, true], [true, true]],
    },
    coloured_beams: true,
    coloured_central_line: true,
    jet_legs: true,
};

/// `u ū → u ū`: flavour locks each outgoing leg to the beam of its own
/// flavour, so both allowed pairs can be crossed at once and the tie-break
/// reaches the scale.
const FLAVOUR_LOCKED_JET_PAIR: ClusterTopology = ClusterTopology {
    beam_connections: BeamConnections::TChannel {
        two_body_pairs: [[true, false], [false, true]],
    },
    ..FREE_JET_PAIR
};

/// A colour-singlet system off coloured beams: `q q̄ → ℓ⁺ℓ⁻`.
const COLOUR_SINGLET_OFF_QUARKS: ClusterTopology = ClusterTopology {
    beam_connections: BeamConnections::SChannelOnly,
    coloured_beams: true,
    coloured_central_line: false,
    jet_legs: false,
};

/// Lepton beams annihilating through a colourless propagator, at any final
/// multiplicity: `e⁺e⁻ → μ⁺μ⁻`, `e⁺e⁻ → t t̄`, `e⁺e⁻ → τ⁺τ⁻ h`.
const COLOURLESS_ANNIHILATION: ClusterTopology = ClusterTopology {
    beam_connections: BeamConnections::SChannelOnly,
    coloured_beams: false,
    coloured_central_line: false,
    jet_legs: false,
};

/// How each banked run's topology is declared, or why its cluster scale has no
/// closed form.
enum Coverage {
    Closed(ClusterTopology),
    /// Refused, with the topology the caller would honestly declare — the
    /// refusal has to come out of a truthful declaration, not out of leaving
    /// the run off a list.
    Refused(ClusterTopology),
    /// The run card fixes both scales, so no clustering topology enters and
    /// the printed fields are run-card constants.
    Fixed,
}

/// Every banked run, with the topology facts its process definition implies.
///
/// Read as a table of hypotheses: each row claims a shape for MadGraph's
/// clustering tree, and a row that claims the wrong shape fails against 10k
/// events rather than passing quietly.
fn coverage(run: &str) -> Coverage {
    use Coverage::{Closed, Fixed, Refused};
    match run {
        // The same `p p -> l+ l- j` process as the two refused runs below,
        // banked with all three scales pinned at m_Z. Its clustering tree is
        // just as far out of reach; nothing has to reach it, because a fixed
        // scale never consults one.
        // `p p -> b b~` at fixed scales, the same process as the two closed-form
        // runs below. A fixed scale never consults a clustering tree, so the row
        // says nothing about the shape of one — which is what makes the pair with
        // pp_to_bb a controlled comparison of the two branches on one process.
        "pp_to_llj_fixed" | "pp_to_bb_fixed" => Fixed,
        // Coloured 2 -> 2 with t-channel exchange. `g g -> g g` and
        // `g g -> t t~` put a colour line between either beam and either leg;
        // `u u~ -> u u~` has only the diagonal, since no diagram joins the
        // incoming u to the outgoing u~.
        "gg_to_gg" => Closed(FREE_JET_PAIR),
        "gg_to_ttx" => Closed(ClusterTopology {
            jet_legs: false,
            ..FREE_JET_PAIR
        }),
        "uux_to_uux" => Closed(FLAVOUR_LOCKED_JET_PAIR),
        // `p p -> b b~` mixes g g (t-channel b, either way round) with q q~
        // (s-channel gluon only). The two agree here: with equal-mass legs the
        // leftover leg's transverse mass and the geometric mean of both are the
        // same number, and neither route can be inflated by the tie-break.
        "pp_to_bb" | "pp_to_bb_qcd2" => Closed(ClusterTopology {
            jet_legs: false,
            ..FREE_JET_PAIR
        }),
        // Colour-singlet final states. Quark beams keep a colour line running
        // through the event; lepton beams do not, and the distinction moves
        // which vertex the scale is read off.
        "pp_to_ll" | "pp_to_ll_qcd0" => Closed(COLOUR_SINGLET_OFF_QUARKS),
        "uux_to_mumu" => Closed(ClusterTopology {
            ..COLOUR_SINGLET_OFF_QUARKS
        }),
        "ee_to_mumu" | "ee_to_ttx" | "ee_to_zh" | "ee_to_tatah" => Closed(COLOURLESS_ANNIHILATION),
        // Lepton beams with a t-channel: Bhabha exchanges a photon between the
        // two electron lines, and `W` pair production a neutrino, each locking
        // an outgoing leg to the beam it can share a vertex with.
        "ee_to_ee" => Closed(ClusterTopology {
            beam_connections: BeamConnections::TChannel {
                two_body_pairs: [[true, false], [false, true]],
            },
            ..COLOURLESS_ANNIHILATION
        }),
        "ee_to_wpwm" => Closed(ClusterTopology {
            beam_connections: BeamConnections::TChannel {
                two_body_pairs: [[false, true], [true, false]],
            },
            ..COLOURLESS_ANNIHILATION
        }),
        // A photon can be radiated off the electron line, so the clustering
        // reaches an initial-state merge and the final state never collapses to
        // one propagator.
        "ee_to_mumua" => Refused(ClusterTopology {
            beam_connections: BeamConnections::TChannel {
                two_body_pairs: [[true, true], [true, true]],
            },
            ..COLOURLESS_ANNIHILATION
        }),
        // No vertex joins an electron to a muon or a tau, but the `Z Z`
        // diagram hangs both bosons off the electron line, so a beam still
        // merges with part of the final state two steps in.
        "ee_to_mumu_tata_qcd0" => Refused(ClusterTopology {
            beam_connections: BeamConnections::TChannel {
                two_body_pairs: [[true, true], [true, true]],
            },
            ..COLOURLESS_ANNIHILATION
        }),
        // A jet off a quark line, and six-leg QCD: the general clustering.
        // The four `*_epemg` / `gu*_to_*` runs are single concrete flavour
        // assignments out of the llj process, banked at partonic beams for
        // the amplitude oracles; their clustering is the llj one.
        "pp_to_llj"
        | "pp_to_llj_qcd2_qed2"
        | "uux_to_epemg"
        | "ddx_to_epemg"
        | "gu_to_epemu"
        | "gux_to_epemux"
        | "bbx_to_ccx_emmm_qcd0"
        | "uux_to_ccx_emmm_qcd0" => Refused(ClusterTopology {
            beam_connections: BeamConnections::TChannel {
                two_body_pairs: [[true, true], [true, true]],
            },
            coloured_beams: true,
            coloured_central_line: true,
            jet_legs: true,
        }),
        other => panic!(
            "banked run {other} has no topology declaration: add one, or declare why its \
             cluster scale has no closed form"
        ),
    }
}

/// The runs whose cluster scale this crate computes. Asserted against what the
/// replay actually managed, so widening or narrowing the set is a test failure
/// and not a silent reclassification.
const CLOSED_FORM_RUNS: &[&str] = &[
    "ee_to_ee",
    "ee_to_mumu",
    "ee_to_tatah",
    "ee_to_ttx",
    "ee_to_wpwm",
    "ee_to_zh",
    "gg_to_gg",
    "gg_to_ttx",
    "pp_to_bb",
    "pp_to_bb_qcd2",
    "pp_to_ll",
    "pp_to_ll_qcd0",
    "uux_to_mumu",
    "uux_to_uux",
];

/// The runs whose cluster scale needs the kT clustering of `cluster.f`.
const CLUSTERING_REQUIRED_RUNS: &[&str] = &[
    "bbx_to_ccx_emmm_qcd0",
    "ddx_to_epemg",
    "ee_to_mumu_tata_qcd0",
    "ee_to_mumua",
    "gu_to_epemu",
    "gux_to_epemux",
    "pp_to_llj",
    "pp_to_llj_qcd2_qed2",
    "uux_to_ccx_emmm_qcd0",
    "uux_to_epemg",
];

/// The runs whose `αs` MadGraph reads out of the PDF grid rather than solving
/// for: with `pdlabel = lhapdf` it links `alfas_functions_lhapdf.f`, whose
/// `ALPHAS(Q)` forwards to LHAPDF's `alphasPDF(Q)`. `RunningAlphaS` refuses
/// those cards, so every `AQCDUP` oracle here has to step over them —
/// [`the_grid_alpha_s_runs_are_refused_for_a_measurable_reason`] is what keeps
/// the step from being a convenience.
const GRID_ALPHA_S_RUNS: &[&str] = &["pp_to_bb_fixed", "pp_to_llj_fixed"];

/// The runs whose scales are run-card constants. `pp_to_llj_fixed` is the same
/// process as two of the refused runs above, so the pair is a controlled
/// comparison: what separates a replayable run from an unreachable one is the
/// `fixed_*_scale` flags alone, not the final state.
const FIXED_SCALE_RUNS: &[&str] = &["pp_to_bb_fixed", "pp_to_llj_fixed"];

/// Every banked run directory carrying an unweighted event file.
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
    assert_eq!(
        runs.len(),
        CLOSED_FORM_RUNS.len() + CLUSTERING_REQUIRED_RUNS.len() + FIXED_SCALE_RUNS.len(),
        "the banked run inventory changed"
    );
    runs
}

/// One `<event>`, reduced to what a scale depends on and what MadGraph
/// recorded for it.
struct Event {
    incoming: [[f64; 4]; 2],
    outgoing: Vec<[f64; 4]>,
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
        for _ in 0..nup {
            let f: Vec<&str> = lines
                .next()
                .expect("particle line")
                .split_whitespace()
                .collect();
            let status: i32 = f[1].parse().expect("ISTUP");
            let p = [
                f[9].parse().expect("E"),
                f[6].parse().expect("px"),
                f[7].parse().expect("py"),
                f[8].parse().expect("pz"),
            ];
            // Status 2 is an intermediate resonance the writer added back in;
            // the matrix element only ever saw the incoming and outgoing legs.
            match status {
                -1 => incoming.push(p),
                1 => outgoing.push(p),
                _ => {}
            }
        }
        assert_eq!(incoming.len(), 2, "expected two incoming legs");

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
    choice: &ScaleChoice,
    topology: Option<ClusterTopology>,
    event: &Event,
    base: MuTriple,
) -> MuTriple {
    let mut spread = MuTriple::default();
    let mut outgoing = event.outgoing.clone();
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
                let moved = evaluate(choice, topology, &event.incoming, &outgoing)
                    .expect("perturbed event stays in the closed-form domain");
                worst = worst.max(&base.difference(&moved));
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

fn evaluate(
    choice: &ScaleChoice,
    topology: Option<ClusterTopology>,
    incoming: &[[f64; 4]; 2],
    outgoing: &[[f64; 4]],
) -> Result<MuTriple, ScaleError> {
    let scales = choice.scales(&ScaleEvent {
        incoming: *incoming,
        outgoing,
        topology,
    })?;
    Ok(MuTriple([scales.mu_r, scales.mu_f[0], scales.mu_f[1]]))
}

fn run_card(run: &Path) -> RunCard {
    RunCard::parse_file(&run.join("Cards/run_card.dat")).expect("run card")
}

/// Every banked run leaves `dynamical_scale_choice` and `scalefact` at their
/// defaults, so the replay below cannot be quietly reading a different
/// prescription than the one it claims to validate. The `fixed_*_scale` flags
/// are what separate the two regimes, and each run must land in the regime its
/// [`coverage`] row claims — a run listed in [`FIXED_SCALE_RUNS`] whose card
/// stopped fixing a scale would otherwise silently start taking the clustering
/// branch.
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
        assert_eq!(choice.scalefact(), 1.0, "{name}: scalefact");
        let fixed = FIXED_SCALE_RUNS.contains(&name.as_str());
        assert_eq!(
            choice.is_fully_fixed(),
            fixed,
            "{name}: fixed-scale classification disagrees with the card"
        );
        assert_eq!(
            choice.needs_topology(),
            !fixed,
            "{name}: topology requirement disagrees with the card"
        );
    }
}

/// Per-event replay: the scales this crate derives against every scale
/// MadGraph printed, for every event of every run whose clustering collapses
/// to a closed form or whose card fixes the scales outright.
#[test]
fn banked_events_reproduce_every_printed_scale() {
    let mut closed: Vec<String> = Vec::new();
    let mut refused: Vec<String> = Vec::new();
    let mut fixed: Vec<String> = Vec::new();
    let mut total_events = 0usize;
    let mut total_comparisons = 0usize;
    let mut worst = (0.0f64, String::new(), String::new());

    for (name, run) in banked_runs() {
        let card = run_card(&run);
        let choice = ScaleChoice::from_run_card(&card).expect("compiled");
        let events = parse_events(&run);
        let topology = match coverage(&name) {
            Coverage::Refused(topology) => {
                let err = evaluate(
                    &choice,
                    Some(topology),
                    &events[0].incoming,
                    &events[0].outgoing,
                )
                .expect_err("must be refused, not approximated");
                assert!(
                    matches!(err, ScaleError::ClusteringNotDegenerate { .. }),
                    "{name}: refused for the wrong reason: {err}"
                );
                println!("{name}: {} events, unsupported — {err}", events.len());
                refused.push(name);
                continue;
            }
            Coverage::Closed(topology) => Some(topology),
            // A fixed scale reads no kinematics, so the replay hands it no
            // topology at all: passing one would let a clustering bug hide
            // behind the constant.
            Coverage::Fixed => {
                fixed.push(name.clone());
                None
            }
        };

        let mut run_worst = (0.0f64, "all fields".to_string());
        let mut outside = 0usize;
        let mut comparisons = 0usize;
        for (index, event) in events.iter().enumerate() {
            let base = evaluate(&choice, topology, &event.incoming, &event.outgoing)
                .unwrap_or_else(|e| panic!("{name} event {index}: {e}"));
            let spread = momentum_spread(&choice, topology, event, base);

            // SCALUP is sqrt(max(q2fact)), so it is compared against whichever
            // factorisation scale is larger; mu_R rides along because the
            // clustering assigns both from the same vertex in every run here.
            let mu_f_max = base.0[1].max(base.0[2]);
            let mut checks = vec![
                (
                    "SCALUP vs mu_F",
                    event.scalup,
                    7,
                    mu_f_max,
                    spread.0[1].max(spread.0[2]),
                ),
                ("SCALUP vs mu_R", event.scalup, 7, base.0[0], spread.0[0]),
            ];
            if let Some(rscale) = event.rscale {
                checks.push(("rscale", rscale, 8, base.0[0], spread.0[0]));
            }
            for beam in 0..2 {
                if let Some(q) = event.pdf_scale[beam] {
                    checks.push((
                        if beam == 0 {
                            "pdfrwt beam 1"
                        } else {
                            "pdfrwt beam 2"
                        },
                        q,
                        8,
                        base.0[1 + beam],
                        spread.0[1 + beam],
                    ));
                }
            }

            for (field, printed, digits, computed, moved) in checks {
                let budget = printed_half_ulp(printed, digits) + moved;
                let fraction = (computed - printed).abs() / budget;
                comparisons += 1;
                if fraction > 1.0 {
                    outside += 1;
                    if outside <= 3 {
                        println!(
                            "  {name} event {index} {field}: printed {printed:.9e}, \
                             computed {computed:.9e}, {fraction:.3} of budget {budget:.3e}"
                        );
                    }
                }
                if fraction > run_worst.0 {
                    run_worst = (fraction, field.to_string());
                }
            }
        }

        assert_eq!(
            outside, 0,
            "{name}: {outside} of {comparisons} comparisons outside the printing budget"
        );
        println!(
            "{name}: {} events, {comparisons} scale comparisons, worst {:.3} of budget (in {})",
            events.len(),
            run_worst.0,
            run_worst.1
        );
        if run_worst.0 > worst.0 {
            worst = (run_worst.0, name.clone(), run_worst.1);
        }
        total_events += events.len();
        total_comparisons += comparisons;
        if topology.is_some() {
            closed.push(name);
        }
    }

    assert_eq!(
        closed, CLOSED_FORM_RUNS,
        "the set of runs whose cluster scale is computed here changed"
    );
    assert_eq!(
        refused, CLUSTERING_REQUIRED_RUNS,
        "the set of runs needing the general clustering changed"
    );
    assert_eq!(
        fixed, FIXED_SCALE_RUNS,
        "the set of runs whose scales are run-card constants changed"
    );
    println!(
        "scales: {total_comparisons} comparisons over {total_events} events in {} runs \
         within their printing budget, worst {:.3} of budget ({} in {}); \
         {} runs refused as needing the general clustering, {} fixed-scale",
        closed.len() + fixed.len(),
        worst.0,
        worst.2,
        worst.1,
        refused.len(),
        fixed.len()
    );
}

/// A second oracle for `μR` that does not read a scale field at all.
///
/// `AQCDUP` is `αs(μR)`, and `dαs/αs ≈ −0.1 · dQ/Q` at these scales, so its
/// seven printed digits locate `μR` to about `1e-6` relative — tighter than
/// `SCALUP`'s own rounding. Feeding the scale computed from the momenta through
/// [`RunningAlphaS`] and landing inside the field's printing budget therefore
/// confirms the scale independently of the field that names it.
///
/// Runs listed in [`GRID_ALPHA_S_RUNS`] cannot take part: their `αs` comes
/// from the PDF grid, not from this evolution.
#[test]
fn banked_events_reproduce_aqcdup_from_the_computed_scale() {
    let mut runs_checked = 0usize;
    let mut events_checked = 0usize;
    let mut worst = (0.0f64, String::new());
    let mut grid_alpha_s: Vec<String> = Vec::new();

    for (name, run) in banked_runs() {
        let topology = match coverage(&name) {
            Coverage::Closed(topology) => Some(topology),
            Coverage::Fixed => None,
            Coverage::Refused(_) => continue,
        };
        let card = run_card(&run);
        let choice = ScaleChoice::from_run_card(&card).expect("compiled");
        let params = ParamCard::from_file(&run.join("Cards/param_card.dat")).expect("param card");
        let a_s = params.get("sminputs", &[3]).expect("aS in SMINPUTS");
        let running = match RunningAlphaS::from_run_card(&card, a_s) {
            Ok(running) => running,
            Err(err) => {
                assert!(
                    GRID_ALPHA_S_RUNS.contains(&name.as_str()),
                    "{name}: unexpected alpha_s source refusal: {err}"
                );
                grid_alpha_s.push(name);
                continue;
            }
        };

        let mut run_worst = 0.0f64;
        let mut outside = 0usize;
        for (index, event) in parse_events(&run).iter().enumerate() {
            let base = evaluate(&choice, topology, &event.incoming, &event.outgoing)
                .unwrap_or_else(|e| panic!("{name} event {index}: {e}"));
            let mu_r = base.0[0];
            let spread = momentum_spread(&choice, topology, event, base).0[0];
            let got = aqcdup_from_alpha_s(running.eval(mu_r));
            let moved = [mu_r + spread, mu_r - spread]
                .into_iter()
                .map(|q| (aqcdup_from_alpha_s(running.eval(q)) - got).abs())
                .fold(0.0f64, f64::max);
            let budget = printed_half_ulp(event.aqcdup, 7) + moved;
            let fraction = (got - event.aqcdup).abs() / budget;
            if fraction > 1.0 {
                outside += 1;
            }
            run_worst = run_worst.max(fraction);
            events_checked += 1;
        }
        assert_eq!(outside, 0, "{name}: {outside} events miss AQCDUP");
        if run_worst > worst.0 {
            worst = (run_worst, name.clone());
        }
        runs_checked += 1;
    }

    assert_eq!(grid_alpha_s, GRID_ALPHA_S_RUNS);
    assert_eq!(
        runs_checked,
        CLOSED_FORM_RUNS.len() + FIXED_SCALE_RUNS.len() - GRID_ALPHA_S_RUNS.len()
    );
    println!(
        "AQCDUP from the computed scale: {events_checked} events across {runs_checked} runs, \
         worst {:.3} of budget (in {})",
        worst.0, worst.1
    );
}

/// The refusal of [`GRID_ALPHA_S_RUNS`] is a measurement, not a convention.
///
/// Those runs fix `μR` at the run card's `scale`, which sits within `1e-5`
/// relative of `M_Z`, so any evolution from `M_Z` to `μR` is negligible at the
/// field's seven digits and `AQCDUP` is essentially `αs(M_Z)` as the run used
/// it. Substituting the parameter card's `αs(M_Z)` — the only value available
/// without the grid's own running — therefore isolates the grid's override,
/// and it misses the printed field by well over its printing budget on every
/// event. Adopting the parameter-card value as an approximation would be a
/// visible error, which is why the refusal stands rather than a fallback.
#[test]
fn the_grid_alpha_s_runs_are_refused_for_a_measurable_reason() {
    let mut checked = 0usize;
    for name in GRID_ALPHA_S_RUNS {
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
    assert_eq!(checked, GRID_ALPHA_S_RUNS.len());
}

/// `unwgt.f:694` fills `AQCDUP` as `g*g/4d0/3.1415926d0`, with π truncated at
/// eight digits while `g` was built from the full one — a systematic `1.7e-8`
/// relative that is a sixth of the field's last printed digit.
fn aqcdup_from_alpha_s(alpha_s: f64) -> f64 {
    const PI: f64 = 3.141592653589793;
    let g = (4.0 * PI * alpha_s).sqrt();
    g * g / 4.0 / 3.1415926
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
    let target = aqcdup * 3.1415926 / std::f64::consts::PI;
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
