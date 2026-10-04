//! The interpreter against its own programs compiled ahead of time.
//!
//! For every row compiled into the `aot-study` feature, times `eval_m2` through the
//! bytecode forward pass (`vm`) and through the row's rendered straight-line program
//! (`aot`), at scalar `f64` and at `LaneField<4>` on prepacked momenta, and with its
//! kernels called out of line (`aot_out`, scalar only), over the 16
//! points of the `eval_strategies` bench. Before timing, every arm's |M|² is checked
//! against the scalar interpreter bit for bit.
//!
//! Protocol, for a noisy shared host: the arms run round-robin, the starting arm
//! rotating each round; each cell is a fixed iteration count calibrated to about
//! `VIBEGRAPH_AOT_CELL_MS` (default 200) ms; `VIBEGRAPH_AOT_ROUNDS` rounds (default
//! 7). Reported per cell: the minimum ns/event over rounds, the median, and the
//! spread `max/min − 1`.
//!
//! Run: `cargo bench -p vibegraph-lib --features aot-study --bench aot_kernels`
//! (add `aot-study-large` for the 2 → 6, after rendering it).

use std::hint::black_box;
use std::time::Instant;

use rand::rngs::StdRng;
use rand::SeedableRng;

use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
use vibegraph::helas::eval::aot_study::{AotAmplitude, Binding, ROWS};
use vibegraph::helas::eval::{
    eval_m2_lanes_packed, pack_lane_points, AmplitudeEvaluator, BoundAmplitude, LaneField,
};
use vibegraph::helas::LorentzVector;
use vibegraph::phasespace::rambo_massless;
use vibegraph::ufo::sm::{sm_model, SMRestrict};
use vibegraph::ufo::EvaluatedModel;

/// `eval_strategies`' rows and external-leg counts, in the order it draws their
/// points from one generator.
const BENCH_ROWS: [(&str, usize); 8] = [
    ("ee_to_mumu", 4),
    ("ee_to_wpwm", 4),
    ("uux_to_uux", 4),
    ("gg_to_gg", 4),
    ("gg_to_ttx", 4),
    ("ee_to_mumua", 5),
    ("ee_to_mumu_tata_qcd0", 6),
    ("uux_to_ccx_emmm_qcd0", 8),
];

fn bench_points(name: &str) -> Vec<Vec<LorentzVector<f64>>> {
    let mut rng = StdRng::seed_from_u64(0xBE7C4);
    let sqrt_s = 500.0;
    for (row, n_ext) in BENCH_ROWS {
        let pts: Vec<Vec<LorentzVector<f64>>> = (0..16)
            .map(|_| {
                let mut p = vec![
                    LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, sqrt_s / 2.0),
                    LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, -sqrt_s / 2.0),
                ];
                p.extend(rambo_massless(sqrt_s, n_ext - 2, &mut rng));
                p
            })
            .collect();
        if row == name {
            return pts;
        }
    }
    panic!("`{name}` is not a bench row")
}

/// One timed arm: its label and one pass over the row's points, returning the |M|² sum.
type Arm<'a> = (&'static str, Box<dyn FnMut() -> f64 + 'a>);

fn env_or<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Time `iters` passes of `f` over all points, in ns per event.
fn time_cell(iters: u64, events: usize, f: &mut dyn FnMut() -> f64) -> f64 {
    let t = Instant::now();
    let mut acc = 0.0;
    for _ in 0..iters {
        acc += f();
    }
    black_box(acc);
    t.elapsed().as_nanos() as f64 / (iters as f64 * events as f64)
}

fn main() {
    let rounds: usize = env_or("VIBEGRAPH_AOT_ROUNDS", 7);
    let cell_ms: f64 = env_or("VIBEGRAPH_AOT_CELL_MS", 200.0);
    let only = std::env::var("VIBEGRAPH_AOT_BENCH_ROWS").ok();
    let model = sm_model(SMRestrict::Default);
    let evaluated = EvaluatedModel::from_model(model.clone());

    println!("row\tarm\tmin_ns\tmedian_ns\tmax_ns\tspread\titers\trounds");
    for row in ROWS {
        if let Some(only) = &only {
            if !only.split(',').any(|r| r.trim() == row.name) {
                continue;
            }
        }
        let pc = parse_proc_card(
            &format!("generate {}", row.process),
            &ParsingOptions::default(),
        )
        .unwrap();
        let sets = generate_from_proc_card(&pc, &model).unwrap();
        let mut eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
        eval.prune_zero_helicities(&evaluated);
        let amp = BoundAmplitude::<f64>::bind(&eval, &evaluated);
        let lane_amp = amp.broadcast_lanes::<4>();
        let pts = bench_points(row.name);
        let packed: Vec<Vec<LorentzVector<LaneField<4>>>> = pts
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| pack_lane_points(&std::array::from_fn(|k| c[k].as_slice())))
            .collect();

        let mut vm_scratch = amp.scratch_space();
        let mut vm_lane_scratch = lane_amp.scratch_space();
        let mut aot = AotAmplitude::new(row.name, &amp, Binding::Inlined);
        let mut aot_out = AotAmplitude::new(row.name, &amp, Binding::Outlined);
        let mut aot_lane = AotAmplitude::new(row.name, &lane_amp, Binding::Inlined);

        // Bit-for-bit, every arm against the scalar interpreter.
        let want: Vec<u64> = pts
            .iter()
            .map(|p| amp.eval_m2(p, &mut vm_scratch).to_bits())
            .collect();
        let got: Vec<u64> = pts.iter().map(|p| aot.eval_m2(p).to_bits()).collect();
        assert_eq!(
            got, want,
            "{}: aot f64 differs from the interpreter",
            row.name
        );
        let got: Vec<u64> = pts.iter().map(|p| aot_out.eval_m2(p).to_bits()).collect();
        assert_eq!(
            got, want,
            "{}: aot_out f64 differs from the interpreter",
            row.name
        );
        let lanes = |v: Vec<[f64; 4]>| v.concat().iter().map(|x| x.to_bits()).collect::<Vec<_>>();
        let got = lanes(
            packed
                .iter()
                .map(|m| eval_m2_lanes_packed(&lane_amp, m, &mut vm_lane_scratch))
                .collect(),
        );
        assert_eq!(got, want, "{}: vm lanes4 differs", row.name);
        let got = lanes(
            packed
                .iter()
                .map(|m| aot_lane.eval_m2(m).to_array())
                .collect(),
        );
        assert_eq!(got, want, "{}: aot lanes4 differs", row.name);

        let n = pts.len();
        let mut arms: Vec<Arm<'_>> = vec![
            (
                "vm_f64",
                Box::new(|| pts.iter().map(|p| amp.eval_m2(p, &mut vm_scratch)).sum()),
            ),
            (
                "aot_f64",
                Box::new(|| pts.iter().map(|p| aot.eval_m2(p)).sum()),
            ),
            (
                "aot_out_f64",
                Box::new(|| pts.iter().map(|p| aot_out.eval_m2(p)).sum()),
            ),
            (
                "vm_lanes4",
                Box::new(|| {
                    packed
                        .iter()
                        .map(|m| {
                            eval_m2_lanes_packed(&lane_amp, m, &mut vm_lane_scratch)
                                .iter()
                                .sum::<f64>()
                        })
                        .sum()
                }),
            ),
            (
                "aot_lanes4",
                Box::new(|| {
                    packed
                        .iter()
                        .map(|m| aot_lane.eval_m2(m).to_array().iter().sum::<f64>())
                        .sum()
                }),
            ),
        ];

        // Calibrate each arm's iteration count to about `cell_ms`.
        let iters: Vec<u64> = arms
            .iter_mut()
            .map(|(_, f)| {
                let mut k = 1u64;
                loop {
                    let ns = time_cell(k, n, f.as_mut()) * (k as f64 * n as f64);
                    if ns > cell_ms * 1e6 / 4.0 || k > 1 << 30 {
                        break ((k as f64 * cell_ms * 1e6 / ns).ceil() as u64).max(1);
                    }
                    k *= 4;
                }
            })
            .collect();

        let mut samples: Vec<Vec<f64>> = vec![Vec::new(); arms.len()];
        for r in 0..rounds {
            for k in 0..arms.len() {
                let a = (r + k) % arms.len();
                samples[a].push(time_cell(iters[a], n, arms[a].1.as_mut()));
            }
        }
        for (a, (name, _)) in arms.iter().enumerate() {
            let mut s = samples[a].clone();
            s.sort_by(f64::total_cmp);
            let (min, max) = (s[0], s[s.len() - 1]);
            println!(
                "{}\t{name}\t{min:.1}\t{:.1}\t{max:.1}\t{:.3}\t{}\t{rounds}",
                row.name,
                s[s.len() / 2],
                max / min - 1.0,
                iters[a]
            );
        }
    }
}
