//! The amplitude gate: vibegraph against MadGraph's own matrix elements at the
//! finest linear level MadGraph exposes, over committed per-process tables.
//!
//! # What is compared
//!
//! Each table (`validation/madgraph/amplitudes/<key>.json`) carries two labelled
//! sets of phase-space points — MadGraph's own banked events, projected exactly
//! on shell, and the fixed RAMBO grid that covers the off-peak corners an event
//! sample under-populates — with MadGraph's values at each:
//!
//! 1. `|M|²` summed over helicities and colour, at every point.
//! 2. `AMP(1:NGRAPHS)` per helicity, at a few points of each set.
//! 3. `JAMP(1:NCOLOR)` per helicity, at the same points.
//!
//! and the param card MadGraph evaluated with, so both sides use the same
//! rounded masses, SM inputs and widths and the comparison is not limited by a
//! seven-significant-figure transcription.
//!
//! # Why each level is here
//!
//! `|M|²` is one real number per point: it cannot see a relative sign or phase
//! between two diagrams whose interference is small at the sampled points, a
//! permutation of the helicity assignment that leaves the sum invariant, a
//! per-diagram error compensated by a colour coefficient, or anything at all
//! about the individual colour flows — which is what `JAMP2` selects events
//! from. The per-diagram and per-flow comparisons close those blind spots, and
//! the points they run at are few because one global constant fitted over every
//! (point, helicity, diagram) entry is already a heavily over-determined fit.
//!
//! What is asserted at the linear level, per process:
//!
//! - The helicity-combination set is MadGraph's own `NHEL` table, and (where the
//!   per-diagram table is banked) the diagram count is `NGRAPHS`.
//! - A **single** complex constant `G`, fitted once per process over every
//!   entry, satisfies `A_i^vg = G · c_j · AMP_j^mg` for the diagrams *and*
//!   `J_f^vg = G · JAMP_f^mg` for the flows. One constant for both is what ties
//!   the diagram convention to the colour convention: vibegraph puts the
//!   annihilation/exchange sign in the diagram root and keeps colour
//!   coefficients of +1, MadGraph puts it in `c_i`, and only the product is
//!   observable.
//! - `|G| = 1` — a uniform rescaling is the one deviation the fit absorbs with
//!   zero residual, and it would rescale every `JAMP2` weight — and `Re G = 0`.
//!   The second half is a convention claim with teeth: vibegraph roots its
//!   diagrams with one factor of `i` MadGraph's `AMP()` leaves out, so the
//!   constant is `±i` and nothing else.
//! - Helicity combinations MadGraph's amplitude vanishes on are not stored; ours
//!   must vanish there too, which is what keeps the omission an assertion.
//! - `eval_jamp2` reproduces `Σ_hel |JAMP_f^mg|²` flow by flow — the colour-flow
//!   selection weight itself, against MadGraph rather than against our own JAMPs.
//! - The helicity-pruned evaluator (the production `eval_m2` configuration) is
//!   bit-for-bit against the unpruned one at every point.
//!
//! # Known blind spots
//!
//! - A phase common to every diagram and flow is absorbed into `G` by
//!   construction. It is genuine convention freedom — MadGraph itself chooses
//!   the overall sign of `JAMP(1)` per colour structure — and `|M|²` cannot see
//!   it. Requiring `Re G = 0` narrows it to a sign but cannot remove it.
//! - The diagram pairing is banked in [`MG_DIAGRAM_ORDER`], not searched for.
//!   Two independent enumeration orders have no reason to agree, so a
//!   disagreement is recorded rather than re-derived per run: a reordering on
//!   either side then fails here instead of being silently re-matched.
//! - For `g g > g g`, flows related by trace reversal carry identical JAMPs
//!   (`J₁ = J₆`, `J₂ = J₄`, `J₃ = J₅`, an exact consequence of the four-gluon
//!   amplitude being real-coefficient in the trace basis), so swapping such a
//!   pair is invisible here, to `JAMP2` and to `|M|²`. It is not invisible to
//!   `color_flow_tags_oracle`, which compares the two orientations' colour-line
//!   endpoints against `leshouche.inc`.
//! - The CF matrix itself is not re-derived here; `color_cf_oracle` pins it
//!   against every generated `DATA CF` block.

mod common;

use libtest_mimic::{Arguments, Failed, Trial};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use common::report::AmplitudesRow;

use vibegraph::diagrams::DiagramSet;
use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
use vibegraph::helas::repr::C;
use vibegraph::helas::LorentzVector;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::EvaluatedModel;

/// Relative tolerance on `|M|²` at the fixed-grid points.
///
/// Both sides evaluate the same HELAS kernels in f64 from the same param card,
/// so the difference is accumulated rounding over the diagram sum; the suite max
/// is 5.4e-13 (`ee_to_mumu_tata_qcd0`) and most processes sit at 1e-15..1e-13.
/// The margin rides above that so genuine regressions fail while benign
/// reassociation (fused kernels, balanced sums) passes.
const GRID_REL_TOL: f64 = 1e-12;

/// Relative tolerance on `|M|²` at the projected event points.
///
/// Measured, not assumed: 17 of the 19 tables agree to 6e-14 or better on their
/// event points, at the same scale as their grid points — which is what the
/// on-shell projection is for, since an unprojected event point disagrees at
/// ~1e-10 where the two sides evaluate different gauge-dependent parts of an
/// off-shell configuration.
const EVENT_REL_TOL: f64 = 1e-12;

/// Processes whose event points need a wider tolerance than [`EVENT_REL_TOL`],
/// with the measurement that set it.
///
/// `e+ e- > mu+ mu- ta+ ta-` is the one. Two of its 24 banked events land within
/// 40 MeV of the Higgs pole in m(τ⁺τ⁻) — the only two of the 74 points that do —
/// and they are exactly the two that exceed 1e-12, at 1.5e-12 and 4.2e-12 while
/// the other 72 stay under 5.2e-13. A propagator whose `s - M² + iMΓ` is a
/// difference of large numbers amplifies the last bits of `s`, and the two sides
/// reach `s` by different summation orders, so this is the point's conditioning
/// rather than a disagreement: at the worse of the two, moving one momentum
/// component by one ulp moves `|M|²` by 8.4e-12, *twice* the deviation being
/// gated. The linear level at the same kinematics is clean — the per-diagram
/// amplitudes agree to 1.2e-14 — which is what rules out a width scheme or a
/// propagator convention, either of which would move the resonant diagram by far
/// more than parts in 1e12.
const EVENT_REL_TOL_OVERRIDE: &[(&str, f64)] = &[("ee_to_mumu_tata_qcd0", 1e-11)];

/// Relative tolerance on the element-wise per-diagram and per-flow comparisons,
/// normalised to the largest reference entry in the process.
const LINEAR_REL_TOL: f64 = 1e-12;

/// MadGraph graph index of each vibegraph diagram, for processes whose two
/// enumeration orders differ. Absent processes pair by the identity.
///
/// Nothing derives one order from the other, so a disagreement is data. The
/// pairing is massively over-determined by what it has to reproduce (every
/// helicity, every point, one constant), so banking it is not fitting it.
const MG_DIAGRAM_ORDER: &[(&str, &[usize])] = &[
    ("ee_to_tatah", &[3, 4, 1, 2, 0]),
    ("ee_to_mumua", &[2, 3, 0, 1, 4, 5, 6, 7]),
    (
        "ee_to_mumu_tata_qcd0",
        &[
            9, 11, 13, 15, 10, 12, 14, 16, 0, 2, 4, 6, 1, 3, 5, 7, 8, 17, 19, 21, 23, 18, 22, 20,
            24,
        ],
    ),
];

fn tables_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/amplitudes")
}

/// One phase-space point: which set it belongs to, the momenta both sides
/// evaluate at, MadGraph's `|M|²`, and — at a few points — its per-helicity
/// tables.
struct Point {
    set: String,
    momenta: Vec<LorentzVector<f64>>,
    m2: f64,
    detail: Option<Detail>,
}

/// MadGraph's per-helicity tables at one point, over the helicity combinations
/// its amplitude does not vanish on (indices into [`Table::helicities`]).
struct Detail {
    helicities: Vec<usize>,
    /// `jamps[row][flow]`
    jamps: Vec<Vec<C<f64>>>,
    /// `amps[row][graph]`, absent for the processes whose diagram count makes
    /// the table impractical to commit.
    amps: Option<Vec<Vec<C<f64>>>>,
}

struct Table {
    key: String,
    process: String,
    n_graphs: usize,
    n_flows: usize,
    flow_structures: Vec<String>,
    /// The colour coefficients of `JAMP(1) = Σ_i c_i AMP(i)`, single-flow only.
    coefficients: Option<Vec<C<f64>>>,
    helicities: Vec<Vec<i32>>,
    param_card: String,
    points: Vec<Point>,
}

fn complex_list(v: &serde_json::Value) -> Vec<C<f64>> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|z| {
            let c = z.as_array().unwrap();
            C::new(c[0].as_f64().unwrap(), c[1].as_f64().unwrap())
        })
        .collect()
}

fn parse_table(json: &serde_json::Value) -> Table {
    let helicities = json["helicities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| {
            h.as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i32)
                .collect()
        })
        .collect();
    let points = json["points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|pt| {
            let momenta = pt["momenta"]
                .as_array()
                .unwrap()
                .iter()
                .map(|m| {
                    let c: Vec<f64> = m
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_f64().unwrap())
                        .collect();
                    LorentzVector::new(c[0], c[1], c[2], c[3])
                })
                .collect();
            let detail = pt.get("detail").map(|d| Detail {
                helicities: d["helicities"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_u64().unwrap() as usize)
                    .collect(),
                jamps: d["jamps"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(complex_list)
                    .collect(),
                amps: d
                    .get("amps")
                    .map(|a| a.as_array().unwrap().iter().map(complex_list).collect()),
            });
            Point {
                set: pt["set"].as_str().unwrap().to_owned(),
                momenta,
                m2: pt["m2"].as_f64().unwrap(),
                detail,
            }
        })
        .collect();
    Table {
        key: json["key"].as_str().unwrap().to_owned(),
        process: json["process"].as_str().unwrap().to_owned(),
        n_graphs: json["n_graphs"].as_u64().unwrap() as usize,
        n_flows: json["n_flows"].as_u64().unwrap() as usize,
        flow_structures: json["flow_structures"]
            .as_array()
            .map(|a| a.iter().map(|s| s.as_str().unwrap().to_owned()).collect())
            .unwrap_or_default(),
        coefficients: json["jamp_coefficients"]
            .as_array()
            .map(|_| complex_list(&json["jamp_coefficients"])),
        helicities,
        param_card: json["param_card"]
            .as_array()
            .unwrap()
            .iter()
            .map(|l| l.as_str().unwrap())
            .collect::<Vec<_>>()
            .join("\n"),
        points,
    }
}

/// How much a point's `|M|²` moves when one momentum component moves by one
/// unit in the last place, relative to `|M|²` itself.
///
/// This separates the two things a `|M|²` disagreement can mean. Two programs
/// summing the same diagrams in different orders differ in the last bits of the
/// momenta they feed each propagator; where the amplitude is well conditioned
/// that is invisible, and where it is not — a point sitting a few widths from a
/// narrow resonance, so that `s - M² + iMΓ` is a difference of large numbers —
/// it is amplified by the propagator. A deviation of the same size as this
/// number is the point's conditioning; one much larger is a real disagreement.
fn ulp_sensitivity(bound: &BoundAmplitude<f64>, momenta: &[LorentzVector<f64>]) -> f64 {
    let mut scratch = bound.scratch_space();
    let base = bound.eval_m2(momenta, &mut scratch);
    let mut worst = 0.0f64;
    for leg in 0..momenta.len() {
        let p = momenta[leg];
        for c in 0..4 {
            let mut components = [p.e(), p.px(), p.py(), p.pz()];
            components[c] = f64::from_bits(components[c].to_bits() + 1);
            let mut nudged: Vec<LorentzVector<f64>> = momenta.to_vec();
            nudged[leg] =
                LorentzVector::new(components[0], components[1], components[2], components[3]);
            let moved = bound.eval_m2(&nudged, &mut scratch);
            worst = worst.max((moved - base).abs() / base.abs().max(1e-300));
        }
    }
    worst
}

/// The largest of the per-set `|M|²` deviations, reported whether or not it
/// fails so the gate's own margin stays measurable.
struct M2Result {
    grid: f64,
    event: f64,
    failures: Vec<String>,
}

fn compare_m2(
    table: &Table,
    bound: &BoundAmplitude<f64>,
    pruned: &BoundAmplitude<f64>,
) -> M2Result {
    let event_tol = EVENT_REL_TOL_OVERRIDE
        .iter()
        .find(|(p, _)| *p == table.key)
        .map_or(EVENT_REL_TOL, |(_, t)| *t);
    let mut scratch = bound.scratch_space();
    let mut scratch_pruned = pruned.scratch_space();
    let mut result = M2Result {
        grid: 0.0,
        event: 0.0,
        failures: Vec::new(),
    };
    for (i, pt) in table.points.iter().enumerate() {
        let ours = bound.eval_m2(&pt.momenta, &mut scratch);
        let rel = (ours - pt.m2).abs() / pt.m2.abs().max(1e-300);
        let (worst, tol) = if pt.set == "grid" {
            (&mut result.grid, GRID_REL_TOL)
        } else {
            (&mut result.event, event_tol)
        };
        if rel > *worst {
            *worst = rel;
        }
        if rel > tol {
            result.failures.push(format!(
                "point {i} ({}): |M|² {ours:e} vs MadGraph {:e}, rel {rel:.3e} > {tol:.0e} \
                 (one ulp on a momentum component moves it by {:.3e})",
                pt.set,
                pt.m2,
                ulp_sensitivity(bound, &pt.momenta)
            ));
        }
        // The production configuration drops helicity combinations that provably
        // contribute below rounding, so it must not move the sum at all.
        let m2_pruned = pruned.eval_m2(&pt.momenta, &mut scratch_pruned);
        if m2_pruned.to_bits() != ours.to_bits() {
            result.failures.push(format!(
                "point {i} ({}): helicity-pruned eval_m2 diverged from unpruned, \
                 {m2_pruned:e} vs {ours:e}",
                pt.set
            ));
        }
    }
    result
}

/// Per-diagram breakdown printed when the element-wise comparison fails: each
/// vibegraph diagram's own best-fit constant with its residual under it, and the
/// normalised overlap against every MadGraph term. A constant that is unit and a
/// residual that is at rounding on every row means the diagrams are individually
/// right and only their relative weighting is wrong; a single off-diagonal
/// overlap of 1 names the diagram this one should have been paired with.
fn report_breakdown(name: &str, vg: &[Vec<C<f64>>], mg: &[Vec<C<f64>>], scale: f64) {
    let n = mg.first().map_or(0, Vec::len);
    eprintln!("[{name}] per-diagram breakdown (vibegraph row vs MadGraph term):");
    let mut best = Vec::with_capacity(n);
    for i in 0..n {
        let (mut num, mut den) = (C::new(0.0, 0.0), 0.0f64);
        for (row, ours) in mg.iter().zip(vg) {
            num += row[i].conj() * ours[i];
            den += row[i].norm_sqr();
        }
        let g = if den > 0.0 {
            num / den
        } else {
            C::new(0.0, 0.0)
        };
        let mut res = 0.0f64;
        let mut overlaps = String::new();
        let (mut top, mut top_at) = (0.0f64, 0usize);
        for j in 0..n {
            let (mut o, mut a, mut b) = (C::new(0.0, 0.0), 0.0f64, 0.0f64);
            for (row, ours) in mg.iter().zip(vg) {
                if j == i {
                    res = res.max((ours[i] - g * row[i]).norm() / scale);
                }
                o += row[j].conj() * ours[i];
                a += ours[i].norm_sqr();
                b += row[j].norm_sqr();
            }
            let overlap = o.norm() / (a * b).sqrt().max(1e-300);
            overlaps.push_str(&format!(" {overlap:.4}"));
            if overlap > top {
                top = overlap;
                top_at = j;
            }
        }
        best.push(top_at);
        eprintln!(
            "  diagram {i}: own constant |g|={:.6} arg={:+7.2}° residual={res:.2e} \
             | overlaps{overlaps}",
            g.norm(),
            g.arg().to_degrees()
        );
    }
    // A candidate pairing, not a fix: it only says which MadGraph term each row
    // looks most like. It becomes a pairing when it is a permutation *and*
    // reproduces every entry under one constant, which is what banking it in
    // MG_DIAGRAM_ORDER and re-running asserts.
    eprintln!("  strongest-overlap pairing: {best:?}");
}

/// The per-flow partial amplitudes at one (point, helicity).
///
/// A single-flow amplitude has no flow node to run — its one flow *is* the
/// coherent amplitude — so the two cases go through different entry points and
/// come back in the same shape.
fn flows(
    bound: &BoundAmplitude<f64>,
    n_flows: usize,
    momenta: &[LorentzVector<f64>],
    hel: &[i32],
    scratch: &mut vibegraph::helas::eval::ScratchSpace<f64>,
) -> Vec<C<f64>> {
    if n_flows == 1 {
        vec![bound.eval_amplitude(momenta, hel, scratch)]
    } else {
        bound.run_flows(momenta, hel, scratch)
    }
}

/// One row of the flattened linear comparison: what MadGraph says, what we say,
/// and where it came from.
struct Entry {
    mg: C<f64>,
    vg: C<f64>,
    what: String,
}

fn fit_constant(entries: &[Entry]) -> (C<f64>, f64) {
    let mut num = C::new(0.0, 0.0);
    let mut den = 0.0f64;
    let mut scale = 0.0f64;
    for e in entries {
        num += e.mg.conj() * e.vg;
        den += e.mg.norm_sqr();
        scale = scale.max(e.mg.norm());
    }
    (num / den, scale)
}

fn worst_deviation(entries: &[Entry], g: C<f64>, scale: f64) -> (f64, String) {
    let mut worst = 0.0f64;
    let mut what = String::new();
    for e in entries {
        let dev = (e.vg - g * e.mg).norm() / scale;
        if dev > worst {
            worst = dev;
            what = format!(
                "{}: vibegraph {:?} vs G·MadGraph {:?}",
                e.what,
                e.vg,
                g * e.mg
            );
        }
    }
    (worst, what)
}

/// The trial, and the report row it writes either way: a failure is a cell the
/// collator has to see as failed, not a cell that silently went missing.
fn run_trial(path: PathBuf) -> Result<(), Failed> {
    let key = path.file_stem().unwrap().to_string_lossy().into_owned();
    match measure(path) {
        Ok(row) => {
            row.write();
            Ok(())
        }
        Err(failed) => {
            let mut row = AmplitudesRow::new(&key, "", "gate");
            row.status = "fail";
            row.note = Some(failed.message().unwrap_or_default().to_string());
            row.write();
            Err(failed)
        }
    }
}

fn measure(path: PathBuf) -> Result<AmplitudesRow, Failed> {
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("{e}"))?;
    let table = parse_table(&json);
    let name = table.key.as_str();

    let model = common::sm_model();
    let card = table
        .param_card
        .parse::<ParamCard>()
        .map_err(|e| format!("[{name}] banked param card: {e:?}"))?;
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &card);

    let sets = common::generate_with(&table.process, model.as_ref());
    let set = sets
        .first()
        .ok_or_else(|| format!("[{name}] no diagram set for '{}'", table.process))?;

    let evaluator = AmplitudeEvaluator::compile(set, model.as_ref())?;
    if evaluator.n_flows() != table.n_flows {
        return Err(format!(
            "[{name}] NCOLOR: vibegraph {} vs MadGraph {}",
            evaluator.n_flows(),
            table.n_flows
        )
        .into());
    }

    // The helicity sum both sides run over must be the same set, or the
    // element-wise comparisons would silently skip combinations and the |M|²
    // comparison would compare different sums.
    let ours: BTreeSet<Vec<i32>> = evaluator.helicities().iter().map(|h| h.to_vec()).collect();
    let theirs: BTreeSet<Vec<i32>> = table.helicities.iter().cloned().collect();
    if ours != theirs {
        return Err(format!(
            "[{name}] helicity sets differ: vibegraph {} combinations, MadGraph {}",
            ours.len(),
            theirs.len()
        )
        .into());
    }

    let mut pruned = AmplitudeEvaluator::compile(set, model.as_ref())?;
    let n_dropped = pruned.prune_zero_helicities(&evaluated);
    let bound = BoundAmplitude::<f64>::bind(&evaluator, &evaluated);
    let bound_pruned = BoundAmplitude::<f64>::bind(&pruned, &evaluated);

    let m2 = compare_m2(&table, &bound, &bound_pruned);

    // ── the linear level ──────────────────────────────────────────────────────

    let banks_amps = table
        .points
        .iter()
        .filter_map(|p| p.detail.as_ref())
        .any(|d| d.amps.is_some());
    let order: Vec<usize> = MG_DIAGRAM_ORDER
        .iter()
        .find(|(p, _)| *p == name)
        .map(|(_, o)| o.to_vec())
        .unwrap_or_else(|| (0..table.n_graphs).collect());

    let mut per_diagram: Vec<AmplitudeEvaluator> = Vec::new();
    if banks_amps {
        if set.diagrams.len() != table.n_graphs {
            return Err(format!(
                "[{name}] diagram count: vibegraph {} vs MadGraph NGRAPHS {}",
                set.diagrams.len(),
                table.n_graphs
            )
            .into());
        }
        if order.len() != table.n_graphs
            || order.iter().collect::<BTreeSet<_>>().len() != order.len()
        {
            return Err(format!(
                "[{name}] MG_DIAGRAM_ORDER is not a permutation of {} indices",
                table.n_graphs
            )
            .into());
        }
        // One evaluator per diagram: a single-diagram `DiagramSet` compiles the
        // same rooted tree the full set gives that diagram — the rooting and its
        // fermion sign are properties of the diagram — so its amplitude root is
        // the diagram's contribution up to the process-wide constant fitted below.
        per_diagram = set
            .diagrams
            .iter()
            .map(|d| {
                AmplitudeEvaluator::compile(
                    &DiagramSet {
                        particles_in: set.particles_in.clone(),
                        particles_out: set.particles_out.clone(),
                        diagrams: vec![d.clone()],
                    },
                    model.as_ref(),
                )
            })
            .collect::<Result<_, _>>()?;
    }

    let mut scratch = bound.scratch_space();
    let mut diagram_entries: Vec<Entry> = Vec::new();
    let mut flow_entries: Vec<Entry> = Vec::new();
    // Kept aligned for the failure breakdown, which needs the tables rather than
    // the flattened entries.
    let (mut vg_rows, mut mg_rows) = (Vec::new(), Vec::new());
    let mut worst_zero = 0.0f64;
    let mut worst_zero_where = String::new();
    let mut worst_jamp2 = 0.0f64;
    let mut worst_jamp2_where = String::new();

    for (pi, pt) in table.points.iter().enumerate() {
        let Some(detail) = pt.detail.as_ref() else {
            continue;
        };
        let listed: BTreeSet<usize> = detail.helicities.iter().copied().collect();

        // Every helicity combination MadGraph's amplitude vanishes on must vanish
        // for us too — the reason the banked tables can omit them.
        let scale_here = detail
            .jamps
            .iter()
            .flatten()
            .fold(0.0f64, |m, z| m.max(z.norm()));
        for (hi, hel) in table.helicities.iter().enumerate() {
            if listed.contains(&hi) {
                continue;
            }
            let ours = flows(&bound, table.n_flows, &pt.momenta, hel, &mut scratch);
            for (fi, value) in ours.iter().enumerate() {
                let dev = value.norm() / scale_here.max(1e-300);
                if dev > worst_zero {
                    worst_zero = dev;
                    worst_zero_where =
                        format!("point {pi}, hel {hel:?}, flow {fi}: vibegraph {value:?}");
                }
            }
        }

        for (row, &hi) in detail.helicities.iter().enumerate() {
            let hel = &table.helicities[hi];
            let ours = flows(&bound, table.n_flows, &pt.momenta, hel, &mut scratch);
            for (fi, mg) in detail.jamps[row].iter().enumerate() {
                let structure = table
                    .flow_structures
                    .get(fi)
                    .map(String::as_str)
                    .unwrap_or("?");
                flow_entries.push(Entry {
                    mg: *mg,
                    vg: ours[fi],
                    what: format!("point {pi}, hel {hel:?}, flow {fi} [{structure}]"),
                });
            }
            let Some(amps) = detail.amps.as_ref() else {
                continue;
            };
            let coefficients = table
                .coefficients
                .as_ref()
                .ok_or_else(|| format!("[{name}] per-diagram table without colour coefficients"))?;
            let mut vg_row = Vec::with_capacity(table.n_graphs);
            let mut mg_row = Vec::with_capacity(table.n_graphs);
            for (di, &j) in order.iter().enumerate() {
                let ours = BoundAmplitude::<f64>::bind(&per_diagram[di], &evaluated);
                let mut own_scratch = ours.scratch_space();
                let value = ours.eval_amplitude(&pt.momenta, hel, &mut own_scratch);
                // MadGraph's per-diagram *contribution* to the amplitude: it puts
                // the relative sign between an annihilation and an exchange
                // diagram into c_i, we put it into the diagram root, and only the
                // product is observable.
                let term = coefficients[j] * amps[row][j];
                diagram_entries.push(Entry {
                    mg: term,
                    vg: value,
                    what: format!(
                        "point {pi}, hel {hel:?}, diagram {di} (MadGraph graph {})",
                        j + 1
                    ),
                });
                vg_row.push(value);
                mg_row.push(term);
            }
            vg_rows.push(vg_row);
            mg_rows.push(mg_row);
        }

        // The colour-flow selection weight, against MadGraph's own JAMPs rather
        // than against ours, so it does not inherit the fit below.
        let mut mg_jamp2 = vec![0.0f64; table.n_flows];
        for row in &detail.jamps {
            for (acc, z) in mg_jamp2.iter_mut().zip(row) {
                *acc += z.norm_sqr();
            }
        }
        let mut ours = vec![0.0f64; table.n_flows];
        bound.eval_jamp2(&pt.momenta, &mut scratch, &mut ours);
        let norm = mg_jamp2.iter().cloned().fold(0.0f64, f64::max);
        for (fi, (a, b)) in ours.iter().zip(&mg_jamp2).enumerate() {
            let dev = (a - b).abs() / norm;
            if dev > worst_jamp2 {
                worst_jamp2 = dev;
                worst_jamp2_where = format!("point {pi}, flow {fi}: {a:e} vs {b:e}");
            }
        }
    }

    // One constant for the whole process, least squares over every entry it has.
    // Fitting it globally rather than per diagram, per flow or per point is what
    // makes the residual sensitive to relative structure: the fit has nowhere to
    // hide it. The diagram entries define it when they exist, because they are
    // the finer object; the flows then have to follow the *same* constant, which
    // is what ties our colour coefficients to MadGraph's.
    let (g, diagram_scale) = fit_constant(if diagram_entries.is_empty() {
        &flow_entries
    } else {
        &diagram_entries
    });
    if !g.norm().is_finite() || g.norm() == 0.0 {
        return Err(format!("[{name}] the reference has no non-zero amplitude").into());
    }

    let (worst_diagram, worst_diagram_where) = worst_deviation(&diagram_entries, g, diagram_scale);
    if worst_diagram > LINEAR_REL_TOL {
        report_breakdown(name, &vg_rows, &mg_rows, diagram_scale);
        return Err(format!(
            "[{name}] per-diagram amplitudes disagree with MadGraph beyond a single \
             global phase (max element-wise deviation {worst_diagram:.3e}) at \
             {worst_diagram_where}"
        )
        .into());
    }

    let flow_scale = flow_entries.iter().fold(0.0f64, |m, e| m.max(e.mg.norm()));
    let (worst_flow, worst_flow_where) = worst_deviation(&flow_entries, g, flow_scale);
    if worst_flow > LINEAR_REL_TOL {
        return Err(format!(
            "[{name}] per-flow JAMPs disagree with MadGraph beyond the per-diagram \
             constant (max element-wise deviation {worst_flow:.3e}) at {worst_flow_where}"
        )
        .into());
    }

    // A uniform rescaling is the one deviation the fit absorbs with zero
    // residual, and it multiplies every JAMP2 weight by |G|².
    let mag_dev = (g.norm() - 1.0).abs();
    if mag_dev > LINEAR_REL_TOL {
        return Err(format!(
            "[{name}] the vibegraph↔MadGraph amplitude constant is not a pure phase: \
             |G| = {:.17} (deviation {mag_dev:.3e})",
            g.norm()
        )
        .into());
    }
    if g.re.abs() > LINEAR_REL_TOL {
        return Err(format!(
            "[{name}] the vibegraph↔MadGraph amplitude constant is not ±i: G = {g:?}. \
             The two sides are expected to differ by exactly the one factor of i \
             vibegraph's diagram roots carry and MadGraph's AMP() does not"
        )
        .into());
    }

    if worst_zero > LINEAR_REL_TOL {
        return Err(format!(
            "[{name}] a helicity combination MadGraph's amplitude vanishes on does not \
             vanish for us (largest {worst_zero:.3e} of the point's scale) at \
             {worst_zero_where}"
        )
        .into());
    }
    if worst_jamp2 > LINEAR_REL_TOL {
        return Err(format!(
            "[{name}] eval_jamp2 disagrees with Σ_hel |MadGraph JAMP|²: max relative \
             deviation {worst_jamp2:.3e} at {worst_jamp2_where}"
        )
        .into());
    }

    println!(
        "  [{name}] '{}' NGRAPHS={} NCOLOR={}: |M|² max_rel grid {:.2e} / event {:.2e}, \
         per-diagram {worst_diagram:.2e}{}, per-flow {worst_flow:.2e}, JAMP2 {worst_jamp2:.2e} \
         (G = {:+.0}i, |G|-1 = {mag_dev:.1e}, {n_dropped} helicity combinations pruned)",
        table.process,
        table.n_graphs,
        table.n_flows,
        m2.grid,
        m2.event,
        if banks_amps { "" } else { " (not banked)" },
        g.im.signum(),
    );

    if !m2.failures.is_empty() {
        return Err(format!(
            "[{name}] {} of {} points disagree with MadGraph's |M|²:\n  {}",
            m2.failures.len(),
            table.points.len(),
            m2.failures.join("\n  ")
        )
        .into());
    }

    let mut row = AmplitudesRow::new(name, &table.process, "gate");
    row.n_graphs = table.n_graphs;
    row.n_flows = table.n_flows;
    row.points_grid = table.points.iter().filter(|p| p.set == "grid").count();
    row.points_event = table.points.len() - row.points_grid;
    row.max_rel_grid = m2.grid;
    row.max_rel_event = m2.event;
    row.per_diagram = banks_amps.then_some(worst_diagram);
    row.per_flow = worst_flow;
    row.jamp2 = worst_jamp2;
    // Every row is compared as the full per-helicity × per-flow outer product;
    // nothing here weakens it to the two projections of it.
    row.factorized = false;
    Ok(row)
}

fn main() {
    let args = Arguments::from_args();

    let dir = tables_dir();
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    paths.sort();
    assert!(
        !paths.is_empty(),
        "no amplitude tables in {} — the committed references are the gate's only input",
        dir.display()
    );

    let trials: Vec<Trial> = paths
        .into_iter()
        .map(|p| {
            let name = p.file_stem().unwrap().to_string_lossy().into_owned();
            Trial::test(name, move || run_trial(p))
        })
        .collect();

    libtest_mimic::run(&args, trials).exit();
}
