//! A per-point record of what a fixed-beam multichannel integration's channels
//! draw: every accepted point's weight and its factors, and the largest-weight
//! points of each iteration decomposed into momenta, densities and matrix element.
//!
//! The record replays one channel of a [`FixedBeamIntegrand::adapt_grids_budget`]
//! run at a time. Under [`BlockAllocation::ByAlpha`] a channel's schedule depends on
//! nothing but its own history: its point count is its α share floored at
//! [`floor_for_acceptance`] of its own measured acceptance, its generator is its own
//! stream positioned at its own cumulative draw, and its grid refines from its own
//! histogram. Replaying it serially therefore reproduces the production run's draws
//! for that channel, given the run's α vector, which the record shows by printing
//! each iteration's estimate, variance, point count and accepted count for
//! comparison with the production run's own.
//!
//! A point's weight in channel `j` is
//!
//! ```text
//! w = pref · |M|² · αⱼ / g(p) · w_grid,   g = Σₖ αₖ gₖ(p),
//! ```
//!
//! with `w_grid` the VEGAS Jacobian of the channel's grid. Each factor is printed,
//! with every `gₖ(p)`, so a large weight can be read as a matrix element the
//! mixture does not follow (`|M|²/g` large) or as a grid that put few points there
//! (`w_grid` large).

use std::io::Write as _;
use std::path::{Path, PathBuf};

use rand_chacha::ChaCha8Rng;

use super::*;
use crate::budget::{floor_for_acceptance, MIN_CHANNEL_NEVAL};
use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
use crate::ufo::slha::ParamCard;
use crate::ufo::sm::{sm_model, SMRestrict};

const SURVEY: usize = 30_000;
const SURVEY_ITERS: usize = 6;

fn output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/output")
}

fn env_list(name: &str, default: &str) -> Vec<String> {
    std::env::var(name)
        .unwrap_or_else(|_| default.to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn env_usizes(name: &str, default: &str) -> Vec<usize> {
    env_list(name, default)
        .iter()
        .map(|s| {
            s.parse()
                .unwrap_or_else(|_| panic!("{name}: `{s}` is not an integer"))
        })
        .collect()
}

fn join(values: impl IntoIterator<Item = f64>) -> String {
    values
        .into_iter()
        .map(|v| format!("{v:e}"))
        .collect::<Vec<_>>()
        .join(",")
}

/// The α vector a traced run integrated `seed` at: the alpha column of its
/// first iteration's lines, one per channel in channel order.
fn traced_alphas(path: &str, seed: u64, channels: usize) -> Vec<f64> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let mut alphas = vec![f64::NAN; channels];
    for line in text.lines() {
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.len() != 14 || p[0] != seed.to_string() || p[3] != "0" {
            continue;
        }
        let j: usize = p[2].parse().expect("channel index");
        alphas[j] = p[9].parse().expect("alpha");
    }
    assert!(
        alphas.iter().all(|a| a.is_finite()),
        "{path} does not carry seed {seed}'s α for every channel"
    );
    alphas
}

/// Whether `D` lines carry, after the α-floor ratios, the ratio of channel `j`'s
/// share of the point under two other partitions of unity to its share `αⱼgⱼ/g`:
/// `αⱼgⱼ²/Σₖαₖgₖ²`, and MadEvent's `AMP2ⱼ/Σₖ AMP2ₖ` (`VG_TAIL_PARTITIONS=1`).
fn compare_partitions() -> bool {
    std::env::var("VG_TAIL_PARTITIONS").is_ok_and(|v| v == "1")
}

/// Whether `D` lines carry each point's outgoing momenta (`VG_TAIL_MOMENTA=1`).
fn keep_momenta() -> bool {
    std::env::var("VG_TAIL_MOMENTA").is_ok_and(|v| v == "1")
}

/// What one accepted point of a replayed iteration evaluated to, factor by factor.
struct PointFactors {
    index: u64,
    value: f64,
    m2: f64,
    channel_weight: f64,
    own_weight: f64,
    /// `g/g'` for each α floor: how much the mixture density at this point would
    /// grow with every αₖ raised to the floor and the set renormalised.
    floor_ratios: Vec<f64>,
    /// The outgoing momenta, kept where the record asks for them.
    momenta: Vec<V>,
}

/// The integrand's factors at one point, as [`FixedBeamIntegrand::value_in_channel`]
/// forms them, or `None` where the cuts or the matrix element zero it.
fn factors_at(
    integ: &FixedBeamIntegrand,
    j: usize,
    point: &[f64],
    index: u64,
    floored: &[Vec<f64>],
) -> Option<PointFactors> {
    let (grid_u, scale_u) = integ.split_point(point);
    let pt = integ.sampler.draw_in_channel(j, grid_u);
    let ext = integ.externals(&pt.momenta);
    if !integ.passes_cuts(&ext) {
        return None;
    }
    let sc = integ.scratch();
    let scale_channel = integ.evaluation_channel(sc, &ext, j, scale_u);
    let m2 = integ.matrix_element_at(sc, &ext, &pt.momenta, scale_channel);
    if m2 == 0.0 {
        return None;
    }
    let weight = integ.sampler.channel_weight(j, &pt.momenta, pt.weight);
    let value = integ.prefactor() * weight * m2;
    let partitions = compare_partitions();
    let floor_ratios = match &integ.sampler {
        Sampler::Multi(mixture) if !floored.is_empty() || partitions => {
            let mut dens = Vec::new();
            mixture.mixture_weight_recording(&pt.momenta, &mut dens);
            let alphas = mixture.alphas();
            let g: f64 = dens.iter().zip(alphas).map(|(d, a)| a * d).sum();
            let mut ratios: Vec<f64> = floored
                .iter()
                .map(|al| g / dens.iter().zip(al).map(|(d, a)| a * d).sum::<f64>())
                .collect();
            if partitions {
                let share = alphas[j] * dens[j] / g;
                let squared: f64 = dens.iter().zip(alphas).map(|(d, a)| a * d * d).sum();
                ratios.push(alphas[j] * dens[j] * dens[j] / squared / share);
                let mut amp2 = vec![0.0; dens.len()];
                sc.subs[0].eval_amp2(&ext, &mut amp2);
                let total: f64 = amp2.iter().sum();
                ratios.push(amp2[j] / total / share);
            }
            ratios
        }
        _ => Vec::new(),
    };
    Some(PointFactors {
        index,
        value,
        m2,
        channel_weight: weight,
        own_weight: pt.weight,
        floor_ratios,
        momenta: if keep_momenta() {
            pt.momenta
        } else {
            Vec::new()
        },
    })
}

/// One point decomposed in full: the line a `P` record carries after its key.
fn decompose(
    integ: &FixedBeamIntegrand,
    j: usize,
    seed: u64,
    index: u64,
    x: &[f64],
    wgt: f64,
    edges: Option<&[Vec<f64>]>,
) -> String {
    let scale_ndim = integ.scale_draw_ndim();
    let mut scale = vec![0.0; scale_ndim];
    SubStream::new(
        seed,
        SCALE_DRAW_STREAM_BASE + j as u64,
        index * scale_ndim as u64,
    )
    .fill_uniforms(&mut scale);
    let mut full = x.to_vec();
    full.extend_from_slice(&scale);
    let Sampler::Multi(mixture) = &integ.sampler else {
        panic!("the multichannel map did not install");
    };
    let alphas = mixture.alphas();
    let f = factors_at(integ, j, &full, index, &[]).expect("a recorded point carries weight");
    assert_eq!(
        integ.value_in_channel(j, &full).to_bits(),
        f.value.to_bits(),
        "the factors do not reproduce the integrand's value"
    );
    let pt = integ.sampler.draw_in_channel(j, x);
    let mut densities = Vec::new();
    let g = 1.0 / mixture.mixture_weight_recording(&pt.momenta, &mut densities);
    let mut shares: Vec<(usize, f64)> = densities
        .iter()
        .zip(alphas)
        .map(|(gk, ak)| ak * gk / g)
        .enumerate()
        .collect();
    shares.sort_by(|a, b| b.1.total_cmp(&a.1));
    let dimw: Vec<f64> = match edges {
        Some(edges) => x
            .iter()
            .zip(edges)
            .map(|(&xd, e)| {
                let nbins = e.len() - 1;
                let k = e.partition_point(|&v| v <= xd).clamp(1, nbins) - 1;
                nbins as f64 * (e[k + 1] - e[k])
            })
            .collect(),
        None => Vec::new(),
    };
    format!(
        "{index} fval {:e} wgt {wgt:e} value {:e} m2 {:e} pref {:e} alpha {:e} g {g:e} \
         gown {:e} own_share {:e} x {} s {} dimw {} p {} top {} dens {}",
        f.value * wgt,
        f.value,
        f.m2,
        integ.prefactor(),
        alphas[j],
        1.0 / f.own_weight,
        alphas[j] * densities[j] / g,
        join(x.iter().copied()),
        join(scale.iter().copied()),
        join(dimw),
        pt.momenta
            .iter()
            .map(|p| join(components(p)))
            .collect::<Vec<_>>()
            .join(" "),
        shares
            .iter()
            .take(8)
            .map(|(k, s)| format!("{k}:{s:.4e}"))
            .collect::<Vec<_>>()
            .join(","),
        join(densities.iter().copied()),
    )
}

/// The record for a banked fixed-energy row, built as the σ gate builds it:
/// the interned Standard Model at its default restriction under the row's own
/// param card, the run card's cuts and per-event scale, and the multichannel at
/// the α vector a traced run used, or α-surveyed on `SURVEY × SURVEY_ITERS` points
/// from each seed where no trace is named.
///
/// Parameters, all environment variables:
///
/// * `VG_TAIL_DIR`, `VG_TAIL_PROCESS`: the row (default `bbx_to_ccx_emmm_qcd0`).
/// * `VG_TAIL_SEEDS`, `VG_TAIL_NEVAL`, `VG_TAIL_CHANNELS`: comma lists; every
///   channel is replayed at every budget for every seed.
/// * `VG_TAIL_NITER`: iterations (default 8).
/// * `VG_TAIL_TOP`: points per iteration decomposed in full (default 4).
/// * `VG_TAIL_TRACE`: a `VG_TRACE` file whose α column is installed instead of
///   a survey.
/// * `VG_TAIL_RECORDED`: a file of `P seed neval channel iteration rank fval
///   index wgt first_point x …` lines recorded during a run; each one of a
///   selected seed, budget and channel is decomposed instead of replaying.
/// * `VG_TAIL_ALPHA_FLOORS`: comma list of α floors; each accepted point's
///   `g/g'` under each is appended to its `D` line.
/// * `VG_TAIL_PARTITIONS`: `1` appends the two partition ratios of
///   [`compare_partitions`] after the α-floor ratios.
/// * `VG_TAIL_MOMENTA`: `1` appends each accepted point's outgoing momenta to
///   its `D` line.
/// * `VG_TAIL_OUT`: file appended to (default `tail-points.txt`); the accepted
///   points' factors go to the same path with `.points` appended.
///
/// Line kinds: `C` one channel's diagram (propagator particle, mass, width and
/// signed external-momentum combination), `A` one seed's α vector, `I` one
/// replayed iteration, `P` one decomposed point, `D` one accepted point's factors
/// (`index fval wgt m2 αⱼ/g 1/gⱼ g/g'…`), `R` one recorded point's weight as
/// recorded and as re-evaluated.
#[test]
#[ignore]
fn probe_2to6_tail_points() {
    let dir = std::env::var("VG_TAIL_DIR").unwrap_or_else(|_| "bbx_to_ccx_emmm_qcd0".into());
    let process = std::env::var("VG_TAIL_PROCESS")
        .unwrap_or_else(|_| "b b~ > c c~ e+ e- mu+ mu- QCD=0".into());
    let seeds: Vec<u64> = env_list("VG_TAIL_SEEDS", "55")
        .iter()
        .map(|s| s.parse().expect("VG_TAIL_SEEDS: an integer"))
        .collect();
    let budgets = env_usizes("VG_TAIL_NEVAL", "1200000");
    let watched = env_usizes("VG_TAIL_CHANNELS", "613");
    let niter = env_usizes("VG_TAIL_NITER", "8")[0];
    let top = env_usizes("VG_TAIL_TOP", "4")[0];
    let trace = std::env::var("VG_TAIL_TRACE").ok();
    let recorded = std::env::var("VG_TAIL_RECORDED").ok();
    let alpha_floors: Vec<f64> = env_list("VG_TAIL_ALPHA_FLOORS", "")
        .iter()
        .map(|s| s.parse().expect("VG_TAIL_ALPHA_FLOORS: a number"))
        .collect();
    let out_path = std::env::var("VG_TAIL_OUT").unwrap_or_else(|_| "tail-points.txt".into());
    let open = |path: &str| {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap_or_else(|e| panic!("{path}: {e}"))
    };
    let mut out = open(&out_path);
    let mut points_out = open(&format!("{out_path}.points"));

    let row = output_dir().join(&dir);
    let card = RunCard::parse_file(&row.join("Cards/run_card.dat")).expect("run card parses");
    let model = sm_model(SMRestrict::Default);
    let param: ParamCard = std::fs::read_to_string(row.join("Cards/param_card.dat"))
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| "".parse().unwrap());
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &param);
    let proc_card = parse_proc_card(&format!("generate {process}"), &ParsingOptions::default())
        .expect("process parses");
    let sets = generate_from_proc_card(&proc_card, &model).expect("diagrams generate");
    let evals = compile_subprocesses(&sets, &model, &evaluated).expect("subprocesses compile");
    let bounds: Vec<_> = evals
        .iter()
        .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
        .collect();
    let rep = &evals[0];
    let legs = process_external_legs(rep, &model, &evaluated);
    let beams = FixedBeams::from_run_card(&card, &legs);
    let cuts = Cuts::compile(&card, &legs).expect("cuts compile");
    let final_masses: Vec<f64> = rep.external_particles()[rep.n_in()..]
        .iter()
        .map(|&id| evaluated.mass(id))
        .collect();
    let spin_color_avg = initial_spin_color_average(rep, &model, &evaluated);
    let diagrams: Vec<Diagram> = sets
        .iter()
        .flat_map(|s| s.diagrams.iter().cloned())
        .collect();
    let amps: Vec<&BoundAmplitude<f64>> = bounds.iter().collect();
    let mut integ = FixedBeamIntegrand::new(amps, &cuts, beams, final_masses, spin_color_avg);
    integ
        .use_running_coupling(&diagrams, &model, &evaluated, &card)
        .expect("the run card's scale installs");

    let channel_diags = channel_diagrams(&diagrams, &evaluated);
    let mut header = format!(
        "# {dir} `{process}` incoming {}\n",
        integ
            .incoming()
            .iter()
            .map(|p| join(components(p)))
            .collect::<Vec<_>>()
            .join(" ")
    );
    for (j, d) in channel_diags.iter().enumerate() {
        let props: Vec<String> = d
            .props
            .iter()
            .map(|p| {
                format!(
                    "{}:{:e}:{:e}:{}",
                    model.particle(p.particle).name,
                    evaluated.mass(p.particle),
                    evaluated.width(p.particle),
                    p.momentum
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join("/")
                )
            })
            .collect();
        header.push_str(&format!("C {j} {}\n", props.join(" ")));
    }
    out.write_all(header.as_bytes()).expect("record writable");

    for &seed in &seeds {
        match &trace {
            Some(path) => {
                let alphas = traced_alphas(path, seed, channel_diags.len());
                integ
                    .use_multichannel_with_alphas(&diagrams, &evaluated, &alphas)
                    .expect("the multichannel map installs")
                    .expect("one α per channel");
            }
            None => {
                integ.use_multichannel(&diagrams, &evaluated, SURVEY, SURVEY_ITERS, seed);
            }
        }
        let alphas = integ.channel_alphas();
        let floored: Vec<Vec<f64>> = alpha_floors
            .iter()
            .map(|&floor| {
                let raised: Vec<f64> = alphas.iter().map(|&a| a.max(floor)).collect();
                let total: f64 = raised.iter().sum();
                raised.iter().map(|a| a / total).collect()
            })
            .collect();
        let mut head = format!("A {seed} {}\n", join(alphas.iter().copied()));
        for &j in &watched {
            head.push_str(&format!("S {seed} {j} {:?}\n", integ.channel_samplers()[j]));
        }
        out.write_all(head.as_bytes()).expect("record writable");

        if let Some(path) = &recorded {
            let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
            for line in text.lines() {
                let p: Vec<&str> = line.split_whitespace().collect();
                if p.first() != Some(&"P") || p[1] != seed.to_string() {
                    continue;
                }
                let neval: usize = p[2].parse().expect("budget");
                let j: usize = p[3].parse().expect("channel");
                if !budgets.contains(&neval) || !watched.contains(&j) {
                    continue;
                }
                let fval: f64 = p[6].parse().expect("weight");
                let index: u64 = p[7].parse().expect("point index");
                let wgt: f64 = p[8].parse().expect("grid Jacobian");
                let xs = p.iter().position(|&t| t == "x").expect("x field") + 1;
                let xe = p.iter().position(|&t| t == "d").unwrap_or(p.len());
                let x: Vec<f64> = p[xs..xe]
                    .iter()
                    .map(|t| t.parse().expect("coordinate"))
                    .collect();
                let body = decompose(&integ, j, seed, index, &x, wgt, None);
                let value = integ.value_in_channel(
                    j,
                    &[x.clone(), {
                        let mut s = vec![0.0; integ.scale_draw_ndim()];
                        SubStream::new(
                            seed,
                            SCALE_DRAW_STREAM_BASE + j as u64,
                            index * integ.scale_draw_ndim() as u64,
                        )
                        .fill_uniforms(&mut s);
                        s
                    }]
                    .concat(),
                );
                out.write_all(
                    format!(
                        "R {seed} {neval} {j} {} {} recorded {fval:e} now {:e}\nP {seed} {neval} \
                         {j} {} {} {body}\n",
                        p[4],
                        p[5],
                        value * wgt,
                        p[4],
                        p[5],
                    )
                    .as_bytes(),
                )
                .expect("record writable");
            }
            continue;
        }

        let ndim = integ.channel_grid_ndim();
        let scale_ndim = integ.scale_draw_ndim();
        for &neval in &budgets {
            for &j in &watched {
                let share = channel_share(alphas[j], neval);
                let mut grid = VegasGrid::new(ndim, VEGAS_NBINS, integ.vegas_alpha)
                    .with_combination(IterationCombination::default())
                    .with_rule(integ.grid_rule);
                let warmup = grid.warmup().min(niter.saturating_sub(1));
                let mut rng = ChaCha8Rng::seed_from_u64(seed);
                rng.set_stream(CHANNEL_STREAM_BASE + j as u64);
                let mut scale_draw = SubStream::new(seed, SCALE_DRAW_STREAM_BASE + j as u64, 0);
                let mut drawn = 0_u64;
                let mut accepted_total = 0_u64;
                let mut n_j = share.max(MIN_CHANNEL_NEVAL);
                for iteration in 0..niter {
                    let floor = floor_for_acceptance(
                        (drawn > 0).then(|| accepted_total as f64 / drawn as f64),
                    );
                    if iteration > 0 {
                        n_j = share.max(floor);
                    }
                    let replay = rng.clone();
                    let mut factors: Vec<PointFactors> = Vec::new();
                    let mut index = drawn;
                    let mut point = vec![0.0; ndim + scale_ndim];
                    let (integral, variance, hist) = grid.adapt_iteration(
                        |x| {
                            point[..ndim].copy_from_slice(x);
                            scale_draw.fill_uniforms(&mut point[ndim..]);
                            let here = index;
                            index += 1;
                            match factors_at(&integ, j, &point, here, &floored) {
                                Some(f) => {
                                    let v = f.value;
                                    factors.push(f);
                                    v
                                }
                                None => 0.0,
                            }
                        },
                        n_j,
                        &mut rng,
                    );

                    // The same generator, redrawn through the public draw, gives each
                    // point's coordinates and grid Jacobian in point order.
                    let mut redraw = replay;
                    let mut coords: Vec<(Vec<f64>, f64)> = Vec::with_capacity(n_j);
                    let mut x = vec![0.0; ndim];
                    for _ in 0..n_j {
                        let wgt = grid.draw(&mut redraw, &mut x);
                        coords.push((x.clone(), wgt));
                    }
                    let wgt_of = |f: &PointFactors| coords[(f.index - drawn) as usize].1;
                    let mut sum = 0.0_f64;
                    let mut next = factors.iter().peekable();
                    for (i, (_, wgt)) in coords.iter().enumerate() {
                        if let Some(f) = next.next_if(|f| f.index == drawn + i as u64) {
                            sum += f.value * wgt;
                        } else {
                            sum += 0.0;
                        }
                    }
                    let mean = sum / n_j as f64;
                    assert!(
                        (mean - integral).abs() <= 1e-12 * integral.abs(),
                        "seed {seed} channel {j} iteration {iteration}: the redrawn weights \
                         average to {mean:e}, the iteration to {integral:e}"
                    );

                    let accepted = factors.len();
                    let mut lines = format!(
                        "I {seed} {neval} {j} {iteration} {} {integral:e} {variance:e} {n_j} \
                         {accepted} {:e} {floor} {share}\n",
                        u8::from(iteration >= warmup),
                        alphas[j],
                    );
                    let mut order: Vec<&PointFactors> = factors.iter().collect();
                    order.sort_by(|a, b| {
                        (b.value * wgt_of(b))
                            .total_cmp(&(a.value * wgt_of(a)))
                            .then(a.index.cmp(&b.index))
                    });
                    for (rank, f) in order.iter().take(top).enumerate() {
                        let (x, wgt) = &coords[(f.index - drawn) as usize];
                        let body = decompose(&integ, j, seed, f.index, x, *wgt, Some(grid.xi()));
                        lines
                            .push_str(&format!("P {seed} {neval} {j} {iteration} {rank} {body}\n"));
                    }
                    out.write_all(lines.as_bytes()).expect("record writable");
                    out.flush().expect("record writable");

                    let mut lines = String::new();
                    for f in &factors {
                        lines.push_str(&format!(
                            "D {seed} {neval} {j} {iteration} {} {:e} {:e} {:e} {:e} {:e} {} {}\n",
                            f.index,
                            f.value * wgt_of(f),
                            wgt_of(f),
                            f.m2,
                            f.channel_weight,
                            f.own_weight,
                            join(f.floor_ratios.iter().copied()),
                            join(f.momenta.iter().flat_map(components)),
                        ));
                    }
                    points_out
                        .write_all(lines.as_bytes())
                        .expect("record writable");

                    drawn += n_j as u64;
                    accepted_total += accepted as u64;
                    if iteration + 1 < niter {
                        grid.refine_grid(&hist, n_j);
                    }
                }
                eprintln!("seed {seed} neval {neval} channel {j}: replayed {niter} iterations");
            }
        }
    }
}
