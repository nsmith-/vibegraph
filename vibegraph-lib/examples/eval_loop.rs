//! Steady-state `eval_m2` loop over one `eval_strategies` bench row, for hardware-counter
//! measurement (`scripts/topdown_kit.sh`).
//!
//! Builds the row's helicity-pruned evaluator, draws the bench's 16 points, warms up,
//! then evaluates the points round and round for a fixed time at one lane width. Only
//! that loop is counted:
//!
//! - With `TOPDOWN_CTL_FIFO` and `TOPDOWN_ACK_FIFO` set (perf's `--control
//!   fifo:<ctl>,<ack>` started with `-D -1`), the loop is bracketed by `enable` and
//!   `disable` commands, so setup and teardown are outside the counts.
//! - Otherwise `--start-at-ms` sleeps until that time after start before looping, for a
//!   perf whose `-D <ms>` delay is the only way to skip setup.
//!
//! Prints one JSON line: the row, width, execution order, events evaluated and wall time,
//! which turn whole-run counts into per-event figures.
//!
//! Usage: `eval_loop --row <name> [--width 1|4|8] [--seconds S] [--start-at-ms MS]`

use std::io::{BufRead, BufReader, Write};
use std::time::{Duration, Instant};

use rand::rngs::StdRng;
use rand::SeedableRng;

use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
use vibegraph::helas::eval::{
    eval_m2_lanes, AmplitudeEvaluator, BoundAmplitude, LaneField, Lanes, SupportedLanes,
};
use vibegraph::helas::repr::Real;
use vibegraph::helas::LorentzVector;
use vibegraph::phasespace::rambo_massless;
use vibegraph::ufo::sm::{sm_model, SMRestrict};
use vibegraph::ufo::EvaluatedModel;

/// The `eval_strategies` bench rows, with the process strings their `mg_amplitude`
/// tables carry.
const ROWS: [(&str, &str); 8] = [
    ("ee_to_mumu", "e+ e- > mu+ mu-"),
    ("ee_to_wpwm", "e+ e- > w+ w-"),
    ("uux_to_uux", "u u~ > u u~"),
    ("gg_to_gg", "g g > g g"),
    ("gg_to_ttx", "g g > t t~"),
    ("ee_to_mumua", "e+ e- > mu+ mu- a"),
    ("ee_to_mumu_tata_qcd0", "e+ e- > mu+ mu- ta+ ta- QCD=0"),
    ("uux_to_ccx_emmm_qcd0", "u u~ > c c~ e+ e- mu+ mu- QCD=0"),
];

const N_POINTS: usize = 16;

struct Args {
    row: String,
    width: usize,
    seconds: f64,
    start_at_ms: Option<u64>,
}

fn parse_args() -> Args {
    let mut args = Args {
        row: String::new(),
        width: 1,
        seconds: 5.0,
        start_at_ms: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let mut value = || it.next().unwrap_or_else(|| panic!("{flag} needs a value"));
        match flag.as_str() {
            "--row" => args.row = value(),
            "--width" => args.width = value().parse().expect("--width is 1, 4 or 8"),
            "--seconds" => args.seconds = value().parse().expect("--seconds is a number"),
            "--start-at-ms" => {
                args.start_at_ms = Some(value().parse().expect("--start-at-ms is an integer"))
            }
            other => panic!("unknown argument {other}"),
        }
    }
    assert!(
        ROWS.iter().any(|(name, _)| *name == args.row),
        "--row must be one of {:?}",
        ROWS.map(|(name, _)| name)
    );
    assert!(matches!(args.width, 1 | 4 | 8), "--width must be 1, 4 or 8");
    args
}

/// The bench's kinematics: beams along ±z at √s = 500, massless RAMBO final states.
fn points(n_ext: usize) -> Vec<Vec<LorentzVector<f64>>> {
    let mut rng = StdRng::seed_from_u64(0xBE7C4);
    let sqrt_s = 500.0;
    (0..N_POINTS)
        .map(|_| {
            let mut p = vec![
                LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, sqrt_s / 2.0),
                LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, -sqrt_s / 2.0),
            ];
            p.extend(rambo_massless(sqrt_s, n_ext - 2, &mut rng));
            p
        })
        .collect()
}

/// perf's control FIFOs, when the kit started perf with counting disabled.
struct PerfControl {
    ctl: std::fs::File,
    ack: BufReader<std::fs::File>,
}

impl PerfControl {
    fn from_env() -> Option<PerfControl> {
        let ctl = std::env::var("TOPDOWN_CTL_FIFO").ok()?;
        let ack = std::env::var("TOPDOWN_ACK_FIFO").ok()?;
        let ctl = std::fs::OpenOptions::new()
            .write(true)
            .open(ctl)
            .expect("open perf control FIFO");
        let ack = BufReader::new(std::fs::File::open(ack).expect("open perf ack FIFO"));
        Some(PerfControl { ctl, ack })
    }

    fn command(&mut self, cmd: &str) {
        writeln!(self.ctl, "{cmd}").expect("write perf control FIFO");
        self.ctl.flush().expect("flush perf control FIFO");
        let mut reply = String::new();
        self.ack.read_line(&mut reply).expect("read perf ack FIFO");
        assert_eq!(reply.trim(), "ack", "perf answered {reply:?} to {cmd}");
    }
}

/// Evaluate every point once at width `N`; returns the |M|² sum so the work is used.
fn pass_lanes<const N: usize>(
    amp: &BoundAmplitude<'_, LaneField<N>>,
    pts: &[Vec<LorentzVector<f64>>],
    scratch: &mut vibegraph::helas::eval::ScratchSpace<LaneField<N>>,
) -> f64
where
    Lanes<N>: SupportedLanes<N>,
    LaneField<N>: Real,
{
    let mut acc = 0.0;
    for chunk in pts.chunks_exact(N) {
        let refs: [&[LorentzVector<f64>]; N] = std::array::from_fn(|k| chunk[k].as_slice());
        acc += eval_m2_lanes(amp, &refs, scratch).iter().sum::<f64>();
    }
    acc
}

/// Run `pass` until `seconds` have elapsed; returns (passes, elapsed, checksum).
fn timed(seconds: f64, mut pass: impl FnMut() -> f64) -> (u64, Duration, f64) {
    let budget = Duration::from_secs_f64(seconds);
    let start = Instant::now();
    let (mut passes, mut acc) = (0u64, 0.0);
    while start.elapsed() < budget {
        acc += pass();
        passes += 1;
    }
    (passes, start.elapsed(), acc)
}

fn main() {
    let launched = Instant::now();
    let args = parse_args();
    let process = ROWS.iter().find(|(name, _)| *name == args.row).unwrap().1;

    let model = sm_model(SMRestrict::Default);
    let evaluated = EvaluatedModel::from_model(model.clone());
    let pc = parse_proc_card(&format!("generate {process}"), &ParsingOptions::default())
        .expect("process card parses");
    let sets = generate_from_proc_card(&pc, &model).expect("process generates");
    let mut eval = AmplitudeEvaluator::compile(&sets[0], &model).expect("process compiles");
    eval.prune_zero_helicities(&evaluated);
    let amp = BoundAmplitude::<f64>::bind(&eval, &evaluated);
    let pts = points(eval.n_ext());

    let mut control = PerfControl::from_env();
    let measure = |pass: &mut dyn FnMut() -> f64, control: &mut Option<PerfControl>| {
        timed(0.5, &mut *pass);
        if control.is_none() {
            if let Some(ms) = args.start_at_ms {
                let at = Duration::from_millis(ms);
                assert!(
                    launched.elapsed() < at,
                    "setup took {:?}, past --start-at-ms {ms}",
                    launched.elapsed()
                );
                std::thread::sleep(at - launched.elapsed());
            }
        }
        if let Some(c) = control.as_mut() {
            c.command("enable");
        }
        let run = timed(args.seconds, &mut *pass);
        if let Some(c) = control.as_mut() {
            c.command("disable");
        }
        run
    };

    let (passes, elapsed, checksum) = match args.width {
        1 => {
            let mut scratch = amp.scratch_space();
            measure(
                &mut || pts.iter().map(|p| amp.eval_m2(p, &mut scratch)).sum(),
                &mut control,
            )
        }
        4 => {
            let lane = amp.broadcast_lanes::<4>();
            let mut scratch = lane.scratch_space();
            measure(&mut || pass_lanes(&lane, &pts, &mut scratch), &mut control)
        }
        _ => {
            let lane = amp.broadcast_lanes::<8>();
            let mut scratch = lane.scratch_space();
            measure(&mut || pass_lanes(&lane, &pts, &mut scratch), &mut control)
        }
    };

    let events = passes * N_POINTS as u64;
    let schedule = std::env::var("VIBEGRAPH_EVAL_SCHEDULE").unwrap_or_else(|_| "production".into());
    println!(
        "{{\"row\":\"{}\",\"process\":\"{}\",\"width\":{},\"schedule\":\"{}\",\
         \"events\":{},\"seconds\":{:.6},\"ns_per_event\":{:.3},\"perf_control\":{},\
         \"checksum\":{:e}}}",
        args.row,
        process,
        args.width,
        schedule,
        events,
        elapsed.as_secs_f64(),
        elapsed.as_secs_f64() * 1e9 / events as f64,
        control.is_some(),
        checksum,
    );
}
