//! Ignored probes of where each channel's weight tail sits against the run
//! card's cut edges, how alike the channels' trained `(τ, y)` grids are, what
//! a rejected point costs against an accepted one, and what a `(τ, y)` grid
//! shared across channels would do to each channel's variance.
//!
//! They live inside the crate rather than beside the banked gates because a
//! rejected point's momenta, a trained grid's bin edges and the split of an
//! accepted point's cost between its matrix elements and its mixture density are
//! not reachable through the public interface.
//!
//! Inputs are the banked run cards, the fetched NNPDF set and the SM UFO, read by
//! path. Every probe prints tab-separated `EDGE`/`TAIL`/`REJECT`/`GRID`/`COND`/
//! `TIME`/`REPLAY` records for a script to aggregate across seeds.
//!
//! Environment: `VG_EDGE_ROWS` (comma-separated, default every row),
//! `VG_EDGE_SEEDS` (seed count, default 1), `VG_EDGE_SEED0` (first seed),
//! `VG_EDGE_DRAWS` (draws a channel, default 200 000) and `VG_EDGE_DUMP` (a
//! directory the near-edge points of the first seed are written to as CSV).
//!
//! Run with `--ignored --nocapture` under `--profile release-debug`.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::Instant;

use rand_chacha::ChaCha8Rng;

use super::*;
use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
use crate::pdf::PdfSet;
use crate::ufo::sm::{sm_model, SMRestrict};
use crate::unweight::{MaxRule, Unweighter};
use std::sync::Arc;

const SQRT_S_HAD: f64 = 13000.0;
const MU_F: f64 = 91.1880;
const PDF_SET: &str = "NNPDF23_lo_as_0130_qed";
/// The unweighting scan's stream base, restated so the probe's sample can be
/// the scan's own draws; the probe asserts its truncated maximum against
/// [`Unweighter::scan_with`]'s, which fails if the two ever drift apart.
const SCAN_STREAM_BASE: u64 = 0x0057_4D41;
const EPSILONS: [f64; 3] = [0.01, 0.05, 0.10];
const TOP_N: usize = 20;

/// One measured row: its process, its card and the budget its gate integrates
/// it at, `(survey, survey iterations, neval, niter)`.
struct Row {
    name: &'static str,
    process: &'static str,
    card: PathBuf,
    budget: (usize, usize, usize, usize),
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn rows() -> Vec<Row> {
    let mg = repo().join("validation/madgraph");
    let all = vec![
        Row {
            name: "pp_to_llj_fixed",
            process: "p p > l+ l- j QCD=2 QED=2",
            card: mg.join("output/pp_to_llj_fixed/Cards/run_card.dat"),
            budget: (8_000, 5, 150_000, 10),
        },
        Row {
            name: "pp_to_jj",
            process: "p p > j j",
            card: mg.join("output/pp_to_jj/Cards/run_card.dat"),
            budget: (8_000, 5, 75_000, 10),
        },
        Row {
            name: "pp_to_llj_mlm",
            process: "p p > e+ e- j",
            card: mg.join("pp_to_llj_mlm_run_card.dat"),
            budget: (8_000, 5, 150_000, 10),
        },
    ];
    match std::env::var("VG_EDGE_ROWS") {
        Ok(list) => {
            let want: Vec<&str> = list.split(',').map(str::trim).collect();
            all.into_iter().filter(|r| want.contains(&r.name)).collect()
        }
        Err(_) => all,
    }
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn seeds() -> Vec<u64> {
    let n = env_usize("VG_EDGE_SEEDS", 1) as u64;
    let s0 = env_usize("VG_EDGE_SEED0", 20_261_020) as u64;
    (0..n).map(|i| s0 + i).collect()
}

/// Everything an integrand borrows, built once per row.
struct Inputs {
    model: Arc<UFOModel>,
    evaluated: EvaluatedModel,
    groups: FlavorGroups,
    set: PdfSet,
    rc: RunCard,
}

fn inputs(row: &Row) -> Inputs {
    let rc = RunCard::parse_file(&row.card).expect("run card");
    let model = sm_model(SMRestrict::Default);
    let evaluated = EvaluatedModel::from_model(model.clone());
    let proc_card = parse_proc_card(
        &format!("generate {}", row.process),
        &ParsingOptions::default(),
    )
    .expect("proc card");
    let sets = generate_from_proc_card(&proc_card, &model).expect("enumeration");
    let groups = derive_flavor_groups(sets, &model, &evaluated, &rc).expect("flavour groups");
    let dir = repo().join("validation/pdf").join(PDF_SET);
    let set = PdfSet::load(&dir, PDF_SET).expect("PDF set");
    Inputs {
        model,
        evaluated,
        groups,
        set,
        rc,
    }
}

/// The row's integrand with the card's scales installed and the channel weights
/// adapted on `seed`, as the row's gate builds it, then its grids trained.
fn trained<'a>(
    row: &Row,
    inp: &'a Inputs,
    amps: &'a [BoundAmplitude<'a, f64>],
    pdf: &'a PdfMember,
    seed: u64,
) -> (ProtonIntegrand<'a>, Vec<ChannelIntegration>, VegasResult) {
    let (survey, iters, neval, niter) = row.budget;
    let mut integ = ProtonIntegrand::new(
        &inp.groups,
        amps,
        &inp.evaluated,
        pdf,
        ProtonBeams::equal(SQRT_S_HAD),
        MU_F,
    )
    .expect("hadronic integrand");
    integ
        .use_run_card_scales(
            &inp.model,
            &inp.evaluated,
            &inp.rc,
            Some(&inp.set.info.alpha_s),
        )
        .expect("run card scales");
    integ.adapt_alphas(seed, survey, iters, 0.5);
    let (channels, total) = integ.adapt_grids(neval, niter, seed);
    (integ, channels, total)
}

#[derive(Clone, Copy, Debug)]
enum EdgeKind {
    PtMin { leg: usize, min: f64 },
    EtaMax { leg: usize, max: f64 },
    DrMin { a: usize, b: usize, r: f64 },
    MassMin { a: usize, b: usize, m: f64 },
}

/// One active cut, with its signed margin in its own variable: positive inside
/// the accepted region, negative outside, relative to the threshold.
#[derive(Clone, Debug)]
struct Edge {
    name: String,
    kind: EdgeKind,
}

fn transverse(p: &V) -> f64 {
    (p.px() * p.px() + p.py() * p.py()).sqrt()
}

fn rapidity(p: &V) -> f64 {
    if p.e() <= p.pz().abs() {
        return -1e99;
    }
    0.5 * ((p.e() + p.pz()) / (p.e() - p.pz())).ln()
}

fn delta_r(a: &V, b: &V) -> f64 {
    let denom = transverse(a) * transverse(b);
    let cos = ((a.px() * b.px() + a.py() * b.py()) / denom).clamp(-0.99999999, 0.99999999);
    let dphi = cos.acos();
    let dy = rapidity(a) - rapidity(b);
    (dphi * dphi + dy * dy).sqrt()
}

fn add(a: &V, b: &V) -> V {
    V::new(
        a.e() + b.e(),
        a.px() + b.px(),
        a.py() + b.py(),
        a.pz() + b.pz(),
    )
}

fn sub(a: &V, b: &V) -> V {
    V::new(
        a.e() - b.e(),
        a.px() - b.px(),
        a.py() - b.py(),
        a.pz() - b.pz(),
    )
}

impl Edge {
    fn margin(&self, lab: &[V]) -> f64 {
        match self.kind {
            EdgeKind::PtMin { leg, min } => transverse(&lab[leg]) / min - 1.0,
            EdgeKind::EtaMax { leg, max } => (max - rapidity(&lab[leg]).abs()) / max,
            EdgeKind::DrMin { a, b, r } => delta_r(&lab[a], &lab[b]) / r - 1.0,
            EdgeKind::MassMin { a, b, m } => add(&lab[a], &lab[b]).m2().max(0.0).sqrt() / m - 1.0,
        }
    }
}

/// The run card's active cuts on this process's final legs, in the order
/// [`Cuts::pass`] tests them: per leg its `pT` then its `η`, then per pair its
/// `ΔR` then its mass. The first negative margin is then the cut that rejected
/// the point.
fn edges_of(inp: &Inputs) -> Vec<Edge> {
    let rc = &inp.rc;
    let (codes, _) = inp.groups.groups()[0].event_legs(0, BeamOrdering::Direct);
    let is_jet = |p: i32| p.abs() <= 5 || p == 21;
    let is_lep = |p: i32| matches!(p.abs(), 11 | 13 | 15);
    let letter = |p: i32| {
        if is_jet(p) {
            'j'
        } else if is_lep(p) {
            'l'
        } else {
            '?'
        }
    };
    let mut out = Vec::new();
    for (leg, &p) in codes.iter().enumerate().skip(2) {
        let c = letter(p);
        if c == '?' {
            continue;
        }
        let pt = rc.float(&format!("pt{c}"));
        if pt > 0.0 {
            out.push(Edge {
                name: format!("pt{c}[{leg}]"),
                kind: EdgeKind::PtMin { leg, min: pt },
            });
        }
        let eta = rc.float(&format!("eta{c}"));
        if eta >= 0.0 {
            out.push(Edge {
                name: format!("eta{c}[{leg}]"),
                kind: EdgeKind::EtaMax { leg, max: eta },
            });
        }
    }
    for a in 2..codes.len() {
        for b in (a + 1)..codes.len() {
            let (ca, cb) = (letter(codes[a]), letter(codes[b]));
            if ca == '?' || cb == '?' {
                continue;
            }
            let tag: String = if ca == cb {
                format!("{ca}{cb}")
            } else {
                // MadGraph names the mixed pair jet first.
                "jl".to_string()
            };
            let dr = rc.float(&format!("dr{tag}"));
            if dr > 0.0 {
                out.push(Edge {
                    name: format!("dr{tag}[{a},{b}]"),
                    kind: EdgeKind::DrMin { a, b, r: dr },
                });
            }
            let same_flavour_pair = ca == cb && (ca == 'j' || codes[a] == -codes[b]);
            if !same_flavour_pair {
                continue;
            }
            let mm = rc.float(&format!("mm{tag}"));
            if mm > 0.0 {
                out.push(Edge {
                    name: format!("mm{tag}[{a},{b}]"),
                    kind: EdgeKind::MassMin { a, b, m: mm },
                });
            }
        }
    }
    out
}

/// One drawn point: its grid coordinates, its weight (zero where it carries
/// none), every cut's margin, and the absolute values of the invariants the
/// channel's coordinates map to.
struct Pt {
    u: Vec<f64>,
    w: f64,
    pass: bool,
    margins: Vec<f64>,
    /// `[ln τ, y, ln(−t_a/s), ln(−t_b/s), ln(m_ll²/s)]`, `t_a` and `t_b` the
    /// transfers from beam 0 and beam 1 to the first final-state jet, and the
    /// last entry only on a lepton-pair process.
    abs: Vec<f64>,
}

/// The momenta a channel draws at `u` in the lab frame, whether or not the cuts
/// accept them: the steps [`ProtonIntegrand::event_in_channel`] takes before its
/// cut test.
fn lab_frame(integ: &ProtonIntegrand<'_>, channel: usize, u: &[f64]) -> (Vec<V>, Vec<V>, f64) {
    let (grid_u, _) = integ.split_point(u);
    let m = integ.map_point(grid_u);
    let point = integ
        .combiner
        .draw_in_channel_at(channel, m.sqrt_shat, &grid_u[OUTER_NDIM..]);
    let sc = integ.scratch();
    integ.build_frames(sc, &m, &point.momenta);
    let lab = sc.lab_buf.borrow().clone();
    let cm = sc.cm_buf.borrow().clone();
    (lab, cm, m.x1 * m.x2)
}

fn absolute_values(
    cm: &[V],
    tau: f64,
    y: f64,
    jet: usize,
    leptons: Option<(usize, usize)>,
) -> Vec<f64> {
    let s_had = SQRT_S_HAD * SQRT_S_HAD;
    let ta = sub(&cm[0], &cm[jet]).m2();
    let tb = sub(&cm[1], &cm[jet]).m2();
    let mut v = vec![tau.ln(), y, (-ta / s_had).ln(), (-tb / s_had).ln()];
    if let Some((a, b)) = leptons {
        v.push((add(&cm[a], &cm[b]).m2() / s_had).ln());
    }
    v
}

fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let i = ((sorted.len() - 1) as f64 * q).round() as usize;
    sorted[i]
}

fn iqr(mut v: Vec<f64>) -> f64 {
    v.retain(|x| x.is_finite());
    v.sort_by(f64::total_cmp);
    quantile(&v, 0.75) - quantile(&v, 0.25)
}

/// The share of a set of weights' sum `Σ f(w)` carried by the marked points.
fn share(points: &[(f64, bool)], f: impl Fn(f64) -> f64) -> f64 {
    let (mut near, mut all) = (0.0, 0.0);
    for &(w, m) in points {
        let x = f(w);
        all += x;
        if m {
            near += x;
        }
    }
    if all > 0.0 {
        near / all
    } else {
        f64::NAN
    }
}

/// The truncated maximum [`MaxRule::default`] reads off sorted weights, restated
/// so the probe can mark which of its own scan points lie above it.
fn truncated_max(weights: &[f64], excess: f64) -> f64 {
    let n = weights.len();
    if n == 0 {
        return 0.0;
    }
    let total: f64 = weights.iter().sum();
    let mut above = 0.0f64;
    let mut i = n;
    while i > 2 && above - weights[i - 1] * ((n - i) as f64) < total * excess {
        above += weights[i - 1];
        i -= 1;
    }
    if i < n {
        i += 1;
    }
    weights[i - 1]
}

/// Which absolute value each grid coordinate maps to, by index into
/// [`Pt::abs`]: the outer `τ` and `y`, the lepton-pair invariant where the
/// process has one, and whichever of the two transfers tracks the channel's
/// polar coordinate more closely (a peripheral channel draws one of them; an
/// all-timelike one draws an angle linear in both).
fn abs_index_of(dim: usize, has_ll: bool, t_pick: usize) -> Option<usize> {
    match (dim, has_ll) {
        (0, _) => Some(0),
        (1, _) => Some(1),
        (2, true) => Some(4),
        (2, false) => Some(t_pick),
        (3, true) => Some(t_pick),
        _ => None,
    }
}

fn pearson(xs: &[f64], ys: &[f64]) -> f64 {
    let pairs: Vec<(f64, f64)> = xs
        .iter()
        .zip(ys)
        .filter(|(x, y)| x.is_finite() && y.is_finite())
        .map(|(&x, &y)| (x, y))
        .collect();
    let n = pairs.len() as f64;
    let (mx, my) = pairs
        .iter()
        .fold((0.0, 0.0), |(a, b), (x, y)| (a + x / n, b + y / n));
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for (x, y) in &pairs {
        sxy += (x - mx) * (y - my);
        sxx += (x - mx) * (x - mx);
        syy += (y - my) * (y - my);
    }
    sxy / (sxx * syy).sqrt()
}

/// How tightly the points near an edge sit in coordinate `k` once the earlier
/// coordinates are held in a cell, against how the channel's points spread there:
/// the near-count-weighted mean, over quantile cells of coordinates `0..k`, of
/// the two interquartile ranges' ratio. Small where the edge is a function of
/// the earlier coordinates in `x_k`.
fn conditional_ratio(points: &[Pt], near: &[bool], k: usize) -> (f64, usize) {
    if k == 0 {
        let all: Vec<f64> = points.iter().map(|p| p.u[0]).collect();
        let nr: Vec<f64> = points
            .iter()
            .zip(near)
            .filter(|(_, &n)| n)
            .map(|(p, _)| p.u[0])
            .collect();
        return (iqr(nr) / iqr(all), 1);
    }
    let bins = ((256f64).powf(1.0 / k as f64).floor() as usize).clamp(2, 16);
    let cuts: Vec<Vec<f64>> = (0..k)
        .map(|d| {
            let mut v: Vec<f64> = points.iter().map(|p| p.u[d]).collect();
            v.sort_by(f64::total_cmp);
            (1..bins)
                .map(|i| quantile(&v, i as f64 / bins as f64))
                .collect()
        })
        .collect();
    let cell = |p: &Pt| -> usize {
        (0..k).fold(0usize, |acc, d| {
            acc * bins + cuts[d].iter().take_while(|&&c| p.u[d] >= c).count()
        })
    };
    let mut cells: HashMap<usize, (Vec<f64>, Vec<f64>)> = HashMap::new();
    for (p, &n) in points.iter().zip(near) {
        let e = cells.entry(cell(p)).or_default();
        e.0.push(p.u[k]);
        if n {
            e.1.push(p.u[k]);
        }
    }
    let (mut num, mut den) = (0.0, 0.0);
    for (all, nr) in cells.into_values() {
        if nr.len() < 10 || all.len() < 20 {
            continue;
        }
        let a = iqr(all);
        if !(a > 0.0) {
            continue;
        }
        let w = nr.len() as f64;
        num += w * iqr(nr) / a;
        den += w;
    }
    (num / den, bins.pow(k as u32))
}

/// Weight-tail shares near every cut edge, rejected-point census, edge geometry
/// in the grid coordinates and the cross-channel `(τ, y)` grid distances, per
/// row and seed.
#[test]
#[ignore]
fn probe_cut_edge_weight_tail() {
    let draws = env_usize("VG_EDGE_DRAWS", 200_000);
    let dump = std::env::var("VG_EDGE_DUMP").ok().map(PathBuf::from);
    for row in rows() {
        let inp = inputs(&row);
        let pdf = inp.set.member(0).expect("PDF member 0");
        let amps: Vec<BoundAmplitude<f64>> = inp
            .groups
            .groups()
            .iter()
            .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), &inp.evaluated))
            .collect();
        let edges = edges_of(&inp);
        let (codes, _) = inp.groups.groups()[0].event_legs(0, BeamOrdering::Direct);
        let jet = (2..codes.len())
            .find(|&i| codes[i].abs() <= 5 || codes[i] == 21)
            .expect("a jet");
        let lep: Vec<usize> = (2..codes.len())
            .filter(|&i| matches!(codes[i].abs(), 11 | 13 | 15))
            .collect();
        let leptons = (lep.len() == 2).then(|| (lep[0], lep[1]));
        eprintln!(
            "== {}: legs {:?}, edges {:?}",
            row.name,
            codes,
            edges.iter().map(|e| e.name.as_str()).collect::<Vec<_>>()
        );
        let mut first_grids: Option<Vec<Vec<Vec<f64>>>> = None;
        for (si, seed) in seeds().into_iter().enumerate() {
            let clock = Instant::now();
            let (integ, channels, total) = trained(&row, &inp, &amps, &pdf, seed);
            let ndim = integ.channel_grid_ndim();
            let scale_ndim = integ.scale_draw_ndim();
            let n_total: usize = channels.iter().map(|c| c.neval).sum();
            eprintln!(
                "-- {} seed {seed}: σ̂ = {:.6e} pb ± {:.3e} (χ²/dof {:.2}), {} channels, \
                 {} evals/iter, ndim {ndim}+{scale_ndim}, trained in {:.0} s",
                row.name,
                total.integral * GEV2_TO_PB,
                total.std_dev * GEV2_TO_PB,
                total.chi2_per_dof,
                channels.len(),
                n_total,
                clock.elapsed().as_secs_f64()
            );

            // M4: the outer grids' bin densities across channels.
            let grids: Vec<Vec<Vec<f64>>> = channels
                .iter()
                .map(|c| c.grid.xi()[..OUTER_NDIM].to_vec())
                .collect();
            report_outer_grids(&row, seed, &channels, &grids, first_grids.as_deref());
            if si == 0 {
                first_grids = Some(grids);
            }

            // The production scan's w_max, for the check below.
            let scan_seed = seed ^ 0x5CA4;
            let scan = Unweighter::scan_with(
                &integ,
                channels.iter().map(|c| (&c.grid, c.neval)),
                scan_seed,
                MaxRule::default(),
            );
            let scan_wmax = scan.w_max();

            let mut dump_file = match (&dump, si) {
                (Some(dir), 0) => {
                    std::fs::create_dir_all(dir).expect("dump dir");
                    let f = std::fs::File::create(dir.join(format!("{}-near-edge.csv", row.name)))
                        .expect("dump file");
                    let mut f = std::io::BufWriter::new(f);
                    let xs: Vec<String> = (0..ndim).map(|k| format!("x{k}")).collect();
                    writeln!(
                        f,
                        "row,seed,channel,edge,margin,w,{},ln_tau,y,ln_mta_over_s,ln_mtb_over_s,ln_mll2_over_s",
                        xs.join(",")
                    )
                    .unwrap();
                    Some(f)
                }
                _ => None,
            };

            for (j, ch) in channels.iter().enumerate() {
                let mut rng = ChaCha8Rng::seed_from_u64(scan_seed);
                rng.set_stream(SCAN_STREAM_BASE + j as u64);
                let mut trailing =
                    SubStream::from_stream(scan_seed, SCALE_DRAW_STREAM_BASE + j as u64);
                let mut u = vec![0.0f64; ndim + scale_ndim];
                let mut pts: Vec<Pt> = Vec::with_capacity(draws);
                let (mut mismatch, mut zero_after_pass) = (0usize, 0usize);
                for _ in 0..draws {
                    let jac = ch.grid.draw(&mut rng, &mut u[..ndim]);
                    trailing.fill_uniforms(&mut u[ndim..]);
                    let (lab, cm, tau) = lab_frame(&integ, j, &u);
                    let pass = integ.cuts.pass(&lab);
                    let margins: Vec<f64> = edges.iter().map(|e| e.margin(&lab)).collect();
                    let mine = margins.iter().all(|&m| m >= 0.0);
                    if mine != pass {
                        mismatch += 1;
                    }
                    let w = if pass {
                        jac * integ.value_in_channel(j, &u)
                    } else {
                        0.0
                    };
                    if pass && !(w > 0.0) {
                        zero_after_pass += 1;
                    }
                    let y = 0.5 * (lab[0].e() / lab[1].e()).ln();
                    pts.push(Pt {
                        u: u[..ndim].to_vec(),
                        w: if w > 0.0 { w } else { 0.0 },
                        pass,
                        margins,
                        abs: absolute_values(&cm, tau, y, jet, leptons),
                    });
                }
                assert_eq!(
                    mismatch, 0,
                    "{} channel {j}: the restated cuts disagree with the compiled filter",
                    row.name
                );

                // The scan's own truncated maximum, off the first `neval` draws.
                let mut scan_w: Vec<f64> = pts[..ch.neval]
                    .iter()
                    .map(|p| p.w)
                    .filter(|&w| w > 0.0)
                    .collect();
                scan_w.sort_by(f64::total_cmp);
                let wmax = truncated_max(&scan_w, crate::unweight::DEFAULT_EXCESS_SHARE);
                assert_eq!(
                    wmax.to_bits(),
                    scan_wmax[j].to_bits(),
                    "{} channel {j}: the probe's sample is not the scan's",
                    row.name
                );

                report_channel(
                    &row,
                    seed,
                    j,
                    ch,
                    n_total,
                    &edges,
                    &pts,
                    wmax,
                    zero_after_pass,
                    leptons.is_some(),
                    dump_file.as_mut(),
                );
            }
        }
    }
}

/// Whether a point lies near one edge, or near any.
type NearTest<'a> = Box<dyn Fn(&Pt) -> bool + 'a>;

#[allow(clippy::too_many_arguments)]
fn report_channel(
    row: &Row,
    seed: u64,
    j: usize,
    ch: &ChannelIntegration,
    n_total: usize,
    edges: &[Edge],
    pts: &[Pt],
    wmax: f64,
    zero_after_pass: usize,
    has_ll: bool,
    mut dump: Option<&mut std::io::BufWriter<std::fs::File>>,
) {
    let n = pts.len();
    let accepted = pts.iter().filter(|p| p.w > 0.0).count();
    let rejected = pts.iter().filter(|p| !p.pass).count();
    let sum: f64 = pts.iter().map(|p| p.w).sum();
    let m2: f64 = pts.iter().map(|p| p.w * p.w).sum();
    // REJECT: acceptance and which cut a rejected point failed first.
    let mut first = vec![0usize; edges.len()];
    for p in pts.iter().filter(|p| !p.pass) {
        if let Some(i) = p.margins.iter().position(|&m| m < 0.0) {
            first[i] += 1;
        }
    }
    println!(
        "REJECT\t{}\t{seed}\t{j}\tN={n}\tQ={:.6}\trejected={:.6}\tzero_after_cuts={:.6}\tneval={}\tNtot_over_Nj={:.3}\tsigma_pb={:.6e}\tM2={:.6e}\t{}",
        row.name,
        accepted as f64 / n as f64,
        rejected as f64 / n as f64,
        zero_after_pass as f64 / n as f64,
        ch.neval,
        n_total as f64 / ch.neval as f64,
        sum / n as f64 * GEV2_TO_PB,
        m2 / n as f64,
        edges
            .iter()
            .zip(&first)
            .map(|(e, c)| format!("{}={:.4}", e.name, *c as f64 / rejected.max(1) as f64))
            .collect::<Vec<_>>()
            .join("\t")
    );

    // TAIL: the shares near each edge (and near any edge) of the weight sum,
    // second moment, top 0.1% second moment, top-20 second moment, and the scan's
    // excess above w_max and its top-20 second moment.
    let mut acc: Vec<&Pt> = pts.iter().filter(|p| p.w > 0.0).collect();
    acc.sort_by(|a, b| b.w.total_cmp(&a.w));
    let top01 = (acc.len() / 1000).max(1);
    let scan_n = ch.neval;
    let mut scan: Vec<&Pt> = pts[..scan_n].iter().filter(|p| p.w > 0.0).collect();
    scan.sort_by(|a, b| b.w.total_cmp(&a.w));
    let any_min = |p: &Pt| p.margins.iter().copied().fold(f64::INFINITY, f64::min);
    for &eps in &EPSILONS {
        let mut cols: Vec<(String, NearTest)> = edges
            .iter()
            .enumerate()
            .map(|(i, e)| {
                (
                    e.name.clone(),
                    Box::new(move |p: &Pt| p.margins[i] < eps) as NearTest,
                )
            })
            .collect();
        cols.push((
            "any".to_string(),
            Box::new(move |p: &Pt| any_min(p) < eps) as NearTest,
        ));
        for (name, near) in &cols {
            let all: Vec<(f64, bool)> = acc.iter().map(|p| (p.w, near(p))).collect();
            let count = share(&all, |_| 1.0);
            let s_w = share(&all, |w| w);
            let s_m2 = share(&all, |w| w * w);
            let s_top01 = share(&all[..top01], |w| w * w);
            let s_top20 = share(&all[..TOP_N.min(all.len())], |w| w * w);
            let sc: Vec<(f64, bool)> = scan.iter().map(|p| (p.w, near(p))).collect();
            let s_excess = share(&sc, |w| (w - wmax).max(0.0));
            let s_scan20 = share(&sc[..TOP_N.min(sc.len())], |w| w * w);
            println!(
                "TAIL\t{}\t{seed}\t{j}\t{eps}\t{name}\tcount={count:.5}\tw={s_w:.5}\tw2={s_m2:.5}\ttop0.1%={s_top01:.5}\ttop20={s_top20:.5}\tscan_excess={s_excess:.5}\tscan_top20={s_scan20:.5}",
                row.name
            );
        }
    }

    // GEOM: for the points within 1% of each edge (both sides), how they sit in
    // each grid coordinate, relative and absolute.
    let ndim = pts[0].u.len();
    let col = |k: usize, sel: &dyn Fn(&Pt) -> bool| -> Vec<f64> {
        pts.iter().filter(|p| sel(p)).map(|p| p.u[k]).collect()
    };
    let t_a: Vec<f64> = pts.iter().map(|p| p.abs[2]).collect();
    let t_b: Vec<f64> = pts.iter().map(|p| p.abs[3]).collect();
    let polar = if has_ll { 3 } else { 2 };
    let xs: Vec<f64> = pts.iter().map(|p| p.u[polar]).collect();
    let (ca, cb) = (pearson(&xs, &t_a), pearson(&xs, &t_b));
    let t_pick = if ca.abs() >= cb.abs() { 2 } else { 3 };
    println!(
        "TPICK\t{}\t{seed}\t{j}\tcorr(x{polar},ln -t_a)={ca:.3}\tcorr(x{polar},ln -t_b)={cb:.3}\tpick={}",
        row.name,
        if t_pick == 2 { "t_a" } else { "t_b" }
    );
    for (i, e) in edges.iter().enumerate() {
        let near: Vec<bool> = pts.iter().map(|p| p.margins[i].abs() < 0.01).collect();
        let n_near = near.iter().filter(|&&b| b).count();
        if n_near < 50 {
            println!(
                "COND\t{}\t{seed}\t{j}\t{}\tnear={n_near}\t(too few)",
                row.name, e.name
            );
            continue;
        }
        for k in 0..ndim {
            let sel_near = |p: &Pt| p.margins[i].abs() < 0.01;
            let all_k = col(k, &|_| true);
            let near_k = col(k, &sel_near);
            let r_rel = iqr(near_k.clone()) / iqr(all_k);
            let at_edge = near_k
                .iter()
                .filter(|&&x| !(0.01..=0.99).contains(&x))
                .count() as f64
                / near_k.len() as f64;
            let all_at_edge = pts
                .iter()
                .filter(|p| !(0.01..=0.99).contains(&p.u[k]))
                .count() as f64
                / pts.len() as f64;
            let r_abs = match abs_index_of(k, has_ll, t_pick) {
                Some(a) => {
                    let all_a: Vec<f64> = pts.iter().map(|p| p.abs[a]).collect();
                    let near_a: Vec<f64> = pts
                        .iter()
                        .filter(|p| sel_near(p))
                        .map(|p| p.abs[a])
                        .collect();
                    iqr(near_a) / iqr(all_a)
                }
                None => f64::NAN,
            };
            let (r_cond, cells) = conditional_ratio(pts, &near, k);
            println!(
                "COND\t{}\t{seed}\t{j}\t{}\tnear={n_near}\tk={k}\tR_rel={r_rel:.3}\tat_window_edge={at_edge:.3}\tbaseline_at_edge={all_at_edge:.3}\tR_abs={r_abs:.3}\tR_cond={r_cond:.3}\tcells={cells}",
                row.name, e.name
            );
        }
        if let Some(f) = dump.as_deref_mut() {
            for p in pts.iter().filter(|p| p.margins[i].abs() < 0.01) {
                let xs: Vec<String> = p.u.iter().map(|x| format!("{x:.6}")).collect();
                let ab: Vec<String> = p.abs.iter().map(|x| format!("{x:.6}")).collect();
                let pad = if p.abs.len() == 4 { ",nan" } else { "" };
                writeln!(
                    f,
                    "{},{seed},{j},\"{}\",{:.6},{:.6e},{},{}{pad}",
                    row.name,
                    e.name,
                    p.margins[i],
                    p.w,
                    xs.join(","),
                    ab.join(",")
                )
                .unwrap();
            }
        }
    }
}

/// Piecewise-constant density of one grid dimension, as `(edges, density)`.
fn l1(a: &[f64], b: &[f64]) -> f64 {
    let dens = |e: &[f64], x: f64| -> f64 {
        let n = e.len() - 1;
        let i = e.partition_point(|&v| v <= x).clamp(1, n) - 1;
        1.0 / (n as f64 * (e[i + 1] - e[i]))
    };
    let mut cuts: Vec<f64> = a.iter().chain(b).copied().collect();
    cuts.sort_by(f64::total_cmp);
    cuts.dedup();
    let mut s = 0.0;
    for w in cuts.windows(2) {
        let (lo, hi) = (w[0], w[1]);
        if hi <= lo {
            continue;
        }
        let mid = 0.5 * (lo + hi);
        s += (dens(a, mid) - dens(b, mid)).abs() * (hi - lo);
    }
    s
}

/// The pooled density `Σⱼ Nⱼ pⱼ / Σⱼ Nⱼ` of one dimension on a fine mesh, as the
/// edges of an equal-probability grid: what a shared outer grid trained on
/// every channel's points would start from.
fn pooled_l1(channels: &[ChannelIntegration], grids: &[Vec<Vec<f64>>], d: usize, j: usize) -> f64 {
    let mesh = 20_000usize;
    let total: f64 = channels.iter().map(|c| c.neval as f64).sum();
    let dens = |e: &[f64], x: f64| -> f64 {
        let n = e.len() - 1;
        let i = e.partition_point(|&v| v <= x).clamp(1, n) - 1;
        1.0 / (n as f64 * (e[i + 1] - e[i]))
    };
    let mut s = 0.0;
    for m in 0..mesh {
        let x = (m as f64 + 0.5) / mesh as f64;
        let pooled: f64 = channels
            .iter()
            .zip(grids)
            .map(|(c, g)| c.neval as f64 / total * dens(&g[d], x))
            .sum();
        s += (dens(&grids[j][d], x) - pooled).abs() / mesh as f64;
    }
    s
}

fn report_outer_grids(
    row: &Row,
    seed: u64,
    channels: &[ChannelIntegration],
    grids: &[Vec<Vec<f64>>],
    first: Option<&[Vec<Vec<f64>>]>,
) {
    let n_total: usize = channels.iter().map(|c| c.neval).sum();
    for d in 0..OUTER_NDIM {
        let mut pair = Vec::new();
        for a in 0..grids.len() {
            for b in (a + 1)..grids.len() {
                pair.push(l1(&grids[a][d], &grids[b][d]));
            }
        }
        pair.sort_by(f64::total_cmp);
        let mean = pair.iter().sum::<f64>() / pair.len().max(1) as f64;
        println!(
            "GRID\t{}\t{seed}\tdim={d}\tpairs={}\tL1_mean={mean:.4}\tL1_median={:.4}\tL1_max={:.4}",
            row.name,
            pair.len(),
            quantile(&pair, 0.5),
            pair.last().copied().unwrap_or(f64::NAN)
        );
        for (j, c) in channels.iter().enumerate() {
            let seed_l1 = first.map_or(f64::NAN, |f| l1(&f[j][d], &grids[j][d]));
            println!(
                "GRIDCH\t{}\t{seed}\tdim={d}\tch={j}\tNtot_over_Nj={:.3}\talpha={:.5}\tL1_to_pooled={:.4}\tL1_to_first_seed={seed_l1:.4}",
                row.name,
                n_total as f64 / c.neval as f64,
                c.alpha,
                pooled_l1(channels, grids, d, j)
            );
        }
    }
}

/// What a rejected point costs against an accepted one, per channel, on the
/// trained grid and one thread: the grid draw, the whole evaluation of rejected
/// and of accepted points, and of an accepted point the mixture weight `αⱼ/g`
/// alone, each amortised over the channel's points.
#[test]
#[ignore]
fn probe_rejected_point_cost() {
    let draws = env_usize("VG_EDGE_DRAWS", 50_000);
    let reps = env_usize("VG_EDGE_REPS", 3);
    for row in rows() {
        let inp = inputs(&row);
        let pdf = inp.set.member(0).expect("PDF member 0");
        let amps: Vec<BoundAmplitude<f64>> = inp
            .groups
            .groups()
            .iter()
            .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), &inp.evaluated))
            .collect();
        let seed = seeds()[0];
        let (integ, channels, _) = trained(&row, &inp, &amps, &pdf, seed);
        let ndim = integ.channel_grid_ndim();
        let scale_ndim = integ.scale_draw_ndim();
        let (mut tot_acc, mut tot_rej, mut tot_n_acc, mut tot_n_rej) = (0.0, 0.0, 0usize, 0usize);
        for (j, ch) in channels.iter().enumerate() {
            let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0x7111);
            rng.set_stream(j as u64);
            let mut trailing =
                SubStream::from_stream(seed ^ 0x7111, SCALE_DRAW_STREAM_BASE + j as u64);
            let mut u = vec![0.0f64; ndim + scale_ndim];
            let mut points: Vec<Vec<f64>> = Vec::with_capacity(draws);
            let t0 = Instant::now();
            for _ in 0..draws {
                ch.grid.draw(&mut rng, &mut u[..ndim]);
                trailing.fill_uniforms(&mut u[ndim..]);
                points.push(u.clone());
            }
            let t_draw = t0.elapsed().as_secs_f64() / draws as f64;
            let (acc, rej): (Vec<Vec<f64>>, Vec<Vec<f64>>) = points
                .into_iter()
                .partition(|u| integ.value_in_channel(j, u) > 0.0);
            let time_of = |set: &[Vec<f64>]| -> f64 {
                let mut best = f64::INFINITY;
                for _ in 0..reps {
                    let t = Instant::now();
                    let mut s = 0.0;
                    for u in set {
                        s += integ.value_in_channel(j, u);
                    }
                    std::hint::black_box(s);
                    best = best.min(t.elapsed().as_secs_f64() / set.len().max(1) as f64);
                }
                best
            };
            let t_acc = time_of(&acc);
            let t_rej = time_of(&rej);
            // The mixture weight of an accepted point on its own: `g` sums every
            // channel's density.
            let frames: Vec<(OuterPoint, Vec<V>)> = acc
                .iter()
                .map(|u| {
                    let (grid_u, _) = integ.split_point(u);
                    let m = integ.map_point(grid_u);
                    let p =
                        integ
                            .combiner
                            .draw_in_channel_at(j, m.sqrt_shat, &grid_u[OUTER_NDIM..]);
                    (m, p.momenta)
                })
                .collect();
            let mut t_g = f64::INFINITY;
            for _ in 0..reps {
                let t = Instant::now();
                let mut s = 0.0;
                for (m, p) in &frames {
                    s += integ.channel_weight(j, m, p);
                }
                std::hint::black_box(s);
                t_g = t_g.min(t.elapsed().as_secs_f64() / frames.len().max(1) as f64);
            }
            let q = acc.len() as f64 / draws as f64;
            let per_point = t_draw + q * t_acc + (1.0 - q) * t_rej;
            println!(
                "TIME\t{}\t{seed}\t{j}\tQ={q:.5}\tn_acc={}\tn_rej={}\tdraw_ns={:.1}\trej_ns={:.1}\tacc_ns={:.1}\tacc_g_ns={:.1}\tacc_rest_ns={:.1}\tper_point_ns={:.1}\tneval={}",
                row.name,
                acc.len(),
                rej.len(),
                1e9 * t_draw,
                1e9 * t_rej,
                1e9 * t_acc,
                1e9 * t_g,
                1e9 * (t_acc - t_rej - t_g),
                1e9 * per_point,
                ch.neval
            );
            tot_acc += t_acc * acc.len() as f64 * ch.neval as f64 / draws as f64;
            tot_rej += t_rej * rej.len() as f64 * ch.neval as f64 / draws as f64;
            tot_n_acc += acc.len() * ch.neval / draws;
            tot_n_rej += rej.len() * ch.neval / draws;
        }
        println!(
            "TIMEROW\t{}\t{seed}\tacc_share_of_eval_time={:.4}\tmean_acc_ns={:.1}\tmean_rej_ns={:.1}\tQ_alloc={:.4}",
            row.name,
            tot_acc / (tot_acc + tot_rej),
            1e9 * tot_acc / tot_n_acc.max(1) as f64,
            1e9 * tot_rej / tot_n_rej.max(1) as f64,
            tot_n_acc as f64 / (tot_n_acc + tot_n_rej) as f64
        );
    }
}

/// Fine bins a replayed outer density is estimated on, per dimension.
const REPLAY_BINS: usize = 128;

/// A one-dimensional density on `[0, 1]`, piecewise constant on
/// [`REPLAY_BINS`] equal bins.
type FineDensity = Vec<f64>;

fn fine_at(q: &[f64], x: f64) -> f64 {
    q[((x * q.len() as f64) as usize).min(q.len() - 1)]
}

/// A trained grid's density at `x` in one dimension, `1/(nbins·Δ)`.
fn grid_density(edges: &[f64], x: f64) -> f64 {
    let n = edges.len() - 1;
    let i = edges.partition_point(|&v| v <= x).clamp(1, n) - 1;
    1.0 / (n as f64 * (edges[i + 1] - edges[i]))
}

/// `√s` normalised to a density, then raised to a floor `c` and renormalised so
/// that no value falls below `1/k`: the fine-bin form of the bin-width bound
/// `K = k` a trained grid carries (an equal-probability bin is at most `k`
/// uniform widths wide exactly when the density is at least `1/k`).
fn bounded_sqrt_density(s: &[f64], k: f64) -> FineDensity {
    let n = s.len() as f64;
    let raw: Vec<f64> = s.iter().map(|v| v.max(0.0).sqrt()).collect();
    let z: f64 = raw.iter().sum::<f64>() / n;
    if !(z > 0.0) {
        return vec![1.0; s.len()];
    }
    let raw: Vec<f64> = raw.iter().map(|v| v / z).collect();
    let floored = |c: f64| -> (f64, Vec<f64>) {
        let v: Vec<f64> = raw.iter().map(|r| r.max(c)).collect();
        let z = v.iter().sum::<f64>() / n;
        (z, v)
    };
    // c/Z(c) rises from 0 at c = 0 towards 1, so bisect for c/Z(c) = 1/k.
    let (mut lo, mut hi) = (0.0_f64, k * raw.iter().copied().fold(1.0, f64::max));
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        let (z, _) = floored(mid);
        if mid / z < 1.0 / k {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let (z, v) = floored(hi);
    v.into_iter().map(|x| x / z).collect()
}

/// One channel's replay sample: per point its weight `w = h/(q_j p_j)` and outer
/// coordinates.
struct ReplayPoint {
    w: f64,
    x: [f64; OUTER_NDIM],
}

/// The A1 decision's offline replay: each channel's trained-grid points
/// reweighted from its own outer density `q_j(τ)q_j(y)` to a shared one, and the
/// per-point variance read at the production allocation `N_j`.
///
/// With `u ~ q_j p_j` and `w = h_j/(q_j p_j)`,
/// `∫ h_j²/(q p_j) = E[w² q_j/q]`, so the variance under a shared `q` needs no
/// new matrix elements. The shared optimum given the inner grids is
/// `q* ∝ √(Σ_j G_j/N_j)` with `G_j(x_o) = ∫ h_j²/p_j`, estimated per outer
/// dimension (the other held at the channel's own grid) from
/// `E[w² q_{j,d}(x_d) 1{x_d ∈ b}]/|b|` on the even-indexed draws and
/// evaluated on the odd-indexed ones. Every candidate density is bounded at
/// `K = 6` like the trained grids. Arms:
///
/// * `vegas`: the channel's own trained grid (the baseline, `r = 1`);
/// * `own`: the same estimator built from channel `j` alone, so the recipe is
///   compared like for like;
/// * `pool`: the `N_j`-weighted mixture of the trained grids (M4's pooled
///   density);
/// * `shared`: `q*` from every channel;
///
/// each applied to `τ` only, `y` only, or both.
#[test]
#[ignore]
fn probe_shared_outer_tier_replay() {
    let draws = env_usize("VG_EDGE_DRAWS", 200_000);
    let k = crate::vegas::DEFAULT_MAX_BIN_WIDTH;
    for row in rows() {
        let inp = inputs(&row);
        let pdf = inp.set.member(0).expect("PDF member 0");
        let amps: Vec<BoundAmplitude<f64>> = inp
            .groups
            .groups()
            .iter()
            .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), &inp.evaluated))
            .collect();
        for seed in seeds() {
            let clock = Instant::now();
            let (integ, channels, total) = trained(&row, &inp, &amps, &pdf, seed);
            let ndim = integ.channel_grid_ndim();
            let scale_ndim = integ.scale_draw_ndim();
            eprintln!(
                "-- {} seed {seed}: σ̂ = {:.6e} pb, trained in {:.0} s",
                row.name,
                total.integral * GEV2_TO_PB,
                clock.elapsed().as_secs_f64()
            );
            let scan_seed = seed ^ 0x5CA4;
            let samples: Vec<Vec<ReplayPoint>> = channels
                .iter()
                .enumerate()
                .map(|(j, ch)| {
                    let mut rng = ChaCha8Rng::seed_from_u64(scan_seed);
                    rng.set_stream(SCAN_STREAM_BASE + j as u64);
                    let mut trailing =
                        SubStream::from_stream(scan_seed, SCALE_DRAW_STREAM_BASE + j as u64);
                    let mut u = vec![0.0f64; ndim + scale_ndim];
                    (0..draws)
                        .map(|_| {
                            let jac = ch.grid.draw(&mut rng, &mut u[..ndim]);
                            trailing.fill_uniforms(&mut u[ndim..]);
                            let w = jac * integ.value_in_channel(j, &u);
                            ReplayPoint {
                                w: if w > 0.0 { w } else { 0.0 },
                                x: [u[0], u[1]],
                            }
                        })
                        .collect()
                })
                .collect();
            let n: Vec<f64> = channels.iter().map(|c| c.neval as f64).collect();
            let n_total: f64 = n.iter().sum();
            let own_q = |j: usize, d: usize, x: f64| grid_density(&channels[j].grid.xi()[d], x);

            // G_j per dimension on the fine bins, from the even draws.
            let g: Vec<[Vec<f64>; OUTER_NDIM]> = samples
                .iter()
                .enumerate()
                .map(|(j, pts)| {
                    let mut out = [vec![0.0; REPLAY_BINS], vec![0.0; REPLAY_BINS]];
                    let m = pts.len().div_ceil(2) as f64;
                    for p in pts.iter().step_by(2) {
                        for (d, gd) in out.iter_mut().enumerate() {
                            let b = ((p.x[d] * REPLAY_BINS as f64) as usize).min(REPLAY_BINS - 1);
                            gd[b] += p.w * p.w * own_q(j, d, p.x[d]) * REPLAY_BINS as f64 / m;
                        }
                    }
                    out
                })
                .collect();
            let shared: Vec<FineDensity> = (0..OUTER_NDIM)
                .map(|d| {
                    let s: Vec<f64> = (0..REPLAY_BINS)
                        .map(|b| (0..channels.len()).map(|j| g[j][d][b] / n[j]).sum())
                        .collect();
                    bounded_sqrt_density(&s, k)
                })
                .collect();
            let own: Vec<Vec<FineDensity>> = (0..channels.len())
                .map(|j| {
                    (0..OUTER_NDIM)
                        .map(|d| bounded_sqrt_density(&g[j][d], k))
                        .collect()
                })
                .collect();
            let pool = |d: usize, x: f64| -> f64 {
                (0..channels.len())
                    .map(|j| n[j] / n_total * own_q(j, d, x))
                    .sum()
            };

            #[derive(Clone, Copy)]
            enum Arm {
                Vegas,
                Own,
                Pool,
                Shared,
            }
            let arms = [
                ("vegas", Arm::Vegas),
                ("own", Arm::Own),
                ("pool", Arm::Pool),
                ("shared", Arm::Shared),
            ];
            let dimsets: [(&str, &[usize]); 3] = [("tau", &[0]), ("y", &[1]), ("both", &[0, 1])];
            let mut row_var = std::collections::BTreeMap::<String, f64>::new();
            for (j, pts) in samples.iter().enumerate() {
                let eval: Vec<&ReplayPoint> = pts.iter().skip(1).step_by(2).collect();
                let m = eval.len() as f64;
                let mean = eval.iter().map(|p| p.w).sum::<f64>() / m;
                for (dname, dims) in dimsets {
                    for (aname, arm) in arms {
                        if matches!(arm, Arm::Vegas) && dname != "both" {
                            continue;
                        }
                        let mut m2 = 0.0;
                        let mut m4 = 0.0;
                        for p in &eval {
                            let mut r = 1.0;
                            for &d in dims {
                                let qd = match arm {
                                    Arm::Vegas => own_q(j, d, p.x[d]),
                                    Arm::Own => fine_at(&own[j][d], p.x[d]),
                                    Arm::Pool => pool(d, p.x[d]),
                                    Arm::Shared => fine_at(&shared[d], p.x[d]),
                                };
                                r *= own_q(j, d, p.x[d]) / qd;
                            }
                            let t = p.w * p.w * r;
                            m2 += t;
                            m4 += t * t;
                        }
                        m2 /= m;
                        let var = m2 - mean * mean;
                        // The standard error of the second-moment estimate itself.
                        let se = ((m4 / m - m2 * m2) / m).max(0.0).sqrt();
                        let key = format!("{dname}/{aname}");
                        *row_var.entry(key.clone()).or_default() += var / n[j];
                        println!(
                            "REPLAY\t{}\t{seed}\tch={j}\tNtot_over_Nj={:.3}\tarm={key}\tvar={var:.6e}\tm2_se={se:.3e}\tI={mean:.6e}",
                            row.name,
                            n_total / n[j],
                        );
                    }
                }
            }
            let base = row_var["both/vegas"];
            for (key, v) in &row_var {
                println!(
                    "REPLAYROW\t{}\t{seed}\tarm={key}\tvar_over_N={v:.6e}\tratio_to_vegas={:.4}",
                    row.name,
                    v / base
                );
            }
        }
    }
}
