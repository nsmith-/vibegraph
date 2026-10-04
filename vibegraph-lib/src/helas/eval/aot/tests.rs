use super::super::compile::AmplitudeEvaluator;
use super::super::lanes::{transpose_points, unpack};
use super::super::run::BoundAmplitude;
use super::byvalue;
use super::*;
use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
use crate::phasespace::rambo_massless;
use crate::ufo::sm::{sm_model, SMRestrict};
use crate::ufo::EvaluatedModel;
use rand::rngs::StdRng;
use rand::SeedableRng;

/// The `eval_strategies` bench's rows with their external-leg counts. The bench draws
/// every row's points from one generator in this order, so reproducing one row's
/// points means drawing (and discarding) the rows before it.
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

/// The bench's 16 points for row `name`: seed `0xBE7C4`, √s = 500, beams along ±z,
/// then massless RAMBO, drawn in bench-row order.
pub(super) fn bench_points(name: &str) -> Vec<Vec<LorentzVector<f64>>> {
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

fn compiled(process: &str) -> (AmplitudeEvaluator, EvaluatedModel) {
    let model = sm_model(SMRestrict::Default);
    let evaluated = EvaluatedModel::from_model(model.clone());
    let pc = parse_proc_card(&format!("generate {process}"), &ParsingOptions::default()).unwrap();
    let sets = generate_from_proc_card(&pc, &model).unwrap();
    let mut eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
    eval.prune_zero_helicities(&evaluated);
    (eval, evaluated)
}

/// The rows [`aot_render`] knows, with their process strings: the compiled-in ones
/// plus the 2 → 6, which is rendered before it can be compiled in.
const RENDERABLE: [(&str, &str); 4] = [
    ("ee_to_mumu", "e+ e- > mu+ mu-"),
    ("gg_to_gg", "g g > g g"),
    ("ee_to_mumu_tata_qcd0", "e+ e- > mu+ mu- ta+ ta- QCD=0"),
    ("uux_to_ccx_emmm_qcd0", "u u~ > c c~ e+ e- mu+ mu- QCD=0"),
];

/// Write the rendered sources into `generated/<form>/`. Rows from
/// `VIBEGRAPH_AOT_ROWS` (comma-separated; default the three small rows), forms from
/// `VIBEGRAPH_AOT_FORMS` (`mg`, `byvalue`; default both), split into functions of
/// `VIBEGRAPH_AOT_CHUNK` instructions (default `0`, one function).
#[test]
#[ignore = "writes the rendered sources; run before building with aot-mg-study(-large)"]
fn aot_render() {
    let rows = std::env::var("VIBEGRAPH_AOT_ROWS")
        .unwrap_or_else(|_| "ee_to_mumu,gg_to_gg,ee_to_mumu_tata_qcd0".to_string());
    let forms = std::env::var("VIBEGRAPH_AOT_FORMS").unwrap_or_else(|_| "mg,byvalue".to_string());
    let chunk: usize = std::env::var("VIBEGRAPH_AOT_CHUNK")
        .map(|c| {
            c.parse()
                .expect("VIBEGRAPH_AOT_CHUNK is an instruction count")
        })
        .unwrap_or(0);
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/helas/eval/aot/generated");
    for name in rows.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let (_, process) = RENDERABLE
            .iter()
            .find(|(n, _)| *n == name)
            .unwrap_or_else(|| panic!("no process string for `{name}`"));
        let (eval, _) = compiled(process);
        let stats = byvalue::render_stats(&eval);
        for form in forms.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            let (f, sub) = match form {
                "mg" => (Form::Mg, "mg"),
                "byvalue" => (Form::ByValue, "byvalue"),
                other => panic!("unknown form `{other}`"),
            };
            let src = render(f, name, &eval, chunk);
            let path = dir.join(sub).join(format!("{name}.rs"));
            std::fs::write(&path, &src).unwrap();
            println!(
                "{name} {form}: {} instructions, {} roots, {} dead values, {} bytes -> {}",
                stats.instrs,
                stats.roots,
                stats.dead,
                src.len(),
                path.display()
            );
        }
    }
}

/// Every compiled-in rendering gives the interpreter's |M|² bit for bit on the bench
/// points: both forms at `f64`, and the MadGraph form at `LaneField<4>` on the same
/// points packed four at a time.
#[test]
fn aot_matches_interpreter_bit_for_bit() {
    for row in ROWS {
        let (eval, evaluated) = compiled(row.process);
        let amp = BoundAmplitude::<f64>::bind(&eval, &evaluated);
        let mut scratch = amp.scratch_space();
        let pts = bench_points(row.name);
        let want: Vec<f64> = pts.iter().map(|p| amp.eval_m2(p, &mut scratch)).collect();
        for form in [Form::Mg, Form::ByValue] {
            if !row.has(form) {
                continue;
            }
            let mut aot = AotAmplitude::new(row.name, &amp, form);
            for (p, &w) in pts.iter().zip(&want) {
                assert!(w > 0.0, "{}: vacuous |M|² {w:e}", row.name);
                let got = aot.eval_m2(p);
                assert_eq!(
                    got.to_bits(),
                    w.to_bits(),
                    "{} {form:?}: f64 {got:e} vs {w:e}",
                    row.name
                );
            }
        }

        let lane_amp = amp.broadcast_lanes::<4>();
        let mut lane_aot = AotAmplitude::new(row.name, &lane_amp, Form::Mg);
        for (chunk, want4) in pts.as_chunks::<4>().0.iter().zip(want.as_chunks::<4>().0) {
            let refs: [&[LorentzVector<f64>]; 4] = std::array::from_fn(|k| chunk[k].as_slice());
            let got4: [f64; 4] = unpack(lane_aot.eval_m2(&transpose_points(&refs)));
            for (g, w) in got4.iter().zip(want4) {
                assert_eq!(
                    g.to_bits(),
                    w.to_bits(),
                    "{}: Mg lanes4 {g:e} vs {w:e}",
                    row.name
                );
            }
        }
    }
}

/// Each rendering reads the bound pools: moving one coupling by a part in 10⁷ moves
/// its |M|², so the bit-for-bit oracle compares two live computations.
#[test]
fn aot_reads_the_bound_pools() {
    let (eval, evaluated) = compiled("e+ e- > mu+ mu-");
    let amp = BoundAmplitude::<f64>::bind(&eval, &evaluated);
    let mut scratch = amp.scratch_space();
    let pts = bench_points("ee_to_mumu");
    let mut pools = amp.pools().0.to_vec();
    pools[0] *= 1.000_000_1;
    let mut moved = amp.clone();
    moved.pools_mut().0.copy_from_slice(&pools);
    let p = &pts[0];
    for form in [Form::Mg, Form::ByValue] {
        let mut aot = AotAmplitude::new("ee_to_mumu", &amp, form);
        let mut moved_aot = AotAmplitude::new("ee_to_mumu", &moved, form);
        assert_eq!(
            aot.eval_m2(p).to_bits(),
            amp.eval_m2(p, &mut scratch).to_bits()
        );
        assert_ne!(moved_aot.eval_m2(p).to_bits(), aot.eval_m2(p).to_bits());
    }
}

/// The MadGraph form leaves nothing in locals: every line of a rendered body is one
/// entry-point call, optionally preceded by the split of its destination's array.
#[test]
fn mg_form_is_calls_only() {
    for row in ROWS {
        let src = row.source(Form::Mg).unwrap();
        let body: Vec<&str> = src
            .lines()
            .filter(|l| {
                l.starts_with("    ")
                    && !l.trim_start().starts_with("let mo")
                    && !l.trim_start().starts_with("let cc")
                    && !l.trim_start().starts_with("let cr")
                    && !l.trim_start().starts_with("let mm")
                    && !l.trim_start().starts_with("mg_")
                    && !l.trim_start().starts_with("bound::")
            })
            .collect();
        assert!(!body.is_empty(), "{}: empty body", row.name);
        for l in body {
            let l = l.trim_start();
            assert!(
                l.starts_with("k::")
                    || l.starts_with("{ let (")
                        && l.contains("= sp(&mut a.")
                        && l.contains("; k::"),
                "{}: not a call: {l}",
                row.name
            );
        }
    }
}
