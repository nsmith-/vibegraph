//! Rendering of a compiled helicity program as straight-line locals.
//!
//! [`render`] writes a compiled, helicity-pruned evaluator's helicity-expanded
//! [`Program`] out as one generic function per process: every instruction, in
//! production execution order, becomes a call to an out-of-line, by-value entry point
//! of [`by_value`](super::kernels::by_value) (or the operator expression) the forward
//! pass in `run.rs` performs, with each result bound to a fresh local. The arena slot a
//! result would occupy is resolved symbolically while rendering: a read of
//! `(class, slot)` names the local the most recent write to that slot produced, so the
//! rendered dataflow is exactly the interpreter's.
//!
//! The rendered function takes the external momenta, both constant pools and the
//! per-point momentum pool as arguments and writes the root amplitudes in the order
//! [`RootKind::Hels`] lists them.
//!
//! A program can be rendered as one function, or split into chunks of consecutive
//! instructions (see [`render`]'s `chunk`); a value read across a chunk boundary is
//! carried in a per-class array of live values.

use std::fmt::Write as _;

use super::super::compile::AmplitudeEvaluator;
use super::super::layout::{Instr, Program, RootKind, N_ARENAS};
use super::super::waveform_slot::WaveformSlot;
use crate::helas::repr::lorentz::{Bispinor, Bra, ComplexVector, Ket, LorentzVector};
use crate::helas::repr::{Real, C};

/// A rendered program: `(external momenta, complex pool, real pool, momentum pool,
/// root amplitudes out)`.
pub(super) type AotKernel<F> =
    fn(&[LorentzVector<F>], &[C<F>], &[F], &[LorentzVector<F>], &mut [C<F>]);

#[allow(
    clippy::all,
    unused_imports,
    unused_variables,
    reason = "rendered code: one local per instruction and literal operands, which \
              the style lints would otherwise ask to fold into each other"
)]
mod rendered {
    use super::super::kernels::by_value as k;
    use super::{ext_fin, ext_fout, ext_scalar, ext_vector, Live};
    use crate::helas::eval::run::build_external_core;
    use crate::helas::repr::lorentz::LorentzVector;
    use crate::helas::repr::numbers::{Charge, Chirality};
    use crate::helas::repr::{Real, C};
    use num_traits::Zero;

    include!("generated/byvalue/ee_to_mumu.rs");
    include!("generated/byvalue/gg_to_gg.rs");
    include!("generated/byvalue/ee_to_mumu_tata_qcd0.rs");
    #[cfg(feature = "aot-mg-study-large-byvalue")]
    include!("generated/byvalue/uux_to_ccx_emmm_qcd0.rs");
}

/// Row `name`'s rendered entry point at scalar type `F`.
pub(super) fn kernel_for<F: Real>(name: &str) -> Option<AotKernel<F>> {
    Some(match name {
        "ee_to_mumu" => rendered::aot_ee_to_mumu::<F>,
        "gg_to_gg" => rendered::aot_gg_to_gg::<F>,
        "ee_to_mumu_tata_qcd0" => rendered::aot_ee_to_mumu_tata_qcd0::<F>,
        #[cfg(feature = "aot-mg-study-large-byvalue")]
        "uux_to_ccx_emmm_qcd0" => rendered::aot_uux_to_ccx_emmm_qcd0::<F>,
        _ => return None,
    })
}

/// Live values carried across a chunk boundary, one array per result class. Each
/// class is sized by the rendered program; a chunk reads the values it needs from it
/// and writes back the ones a later chunk reads. Allocated and zeroed once per call.
#[allow(dead_code, reason = "constructed only by chunked renderings")]
pub(super) struct Live<F: Real> {
    pub reals: Vec<F>,
    pub scalars: Vec<C<F>>,
    pub vectors: Vec<ComplexVector<F>>,
    pub multivectors: Vec<crate::helas::repr::lorentz::Multivector<F>>,
    pub fin: Vec<Bispinor<F, Ket>>,
    pub fout: Vec<Bispinor<F, Bra>>,
}

#[allow(dead_code, reason = "no rendered row has an external scalar")]
#[inline(always)]
fn ext_scalar<F: Real>(slot: WaveformSlot<F>) -> C<F> {
    let WaveformSlot::Scalar(s) = slot else {
        panic!("external scalar leg produced a non-scalar slot");
    };
    s.value
}

#[inline(always)]
fn ext_vector<F: Real>(slot: WaveformSlot<F>) -> ComplexVector<F> {
    let WaveformSlot::Vector(v) = slot else {
        panic!("external vector leg produced a non-vector slot");
    };
    v.eps
}

#[inline(always)]
fn ext_fin<F: Real>(slot: WaveformSlot<F>) -> Bispinor<F, Ket> {
    let WaveformSlot::FermionIn(f) = slot else {
        panic!("external ket leg produced a non-fermion-in slot");
    };
    f.spinor
}

#[inline(always)]
fn ext_fout<F: Real>(slot: WaveformSlot<F>) -> Bispinor<F, Bra> {
    let WaveformSlot::FermionOut(f) = slot else {
        panic!("external bra leg produced a non-fermion-out slot");
    };
    f.spinor
}

/// The result class an instruction writes, as an arena index, or `None` for the
/// variadic roots, which compute nothing.
pub(super) fn out_class(instr: &Instr) -> Option<usize> {
    use Instr::*;
    Some(match instr {
        RealConst { .. } => 0,
        ComplexConst { .. }
        | ExternalScalar { .. }
        | PropagateScalar { .. }
        | AddScalar { .. }
        | MulScalarC { .. }
        | MulScalarR { .. }
        | Bilinear { .. }
        | Pseudoscalar { .. }
        | Metric { .. }
        | EpsilonAmp { .. }
        | FierzPair { .. } => 1,
        ExternalVector { .. }
        | PropagateVector { .. }
        | AddVector { .. }
        | ScaleVecC { .. }
        | ScaleVecR { .. }
        | GammaVout { .. }
        | FfvVout { .. }
        | MetricVout { .. }
        | EpsilonVout { .. }
        | PMom { .. }
        | PMomOut { .. }
        | SigmaVout { .. } => 2,
        FierzOut { .. }
        | SigmaMv { .. }
        | SigmaOut { .. }
        | ScaleMvC { .. }
        | ScaleMvR { .. }
        | AddMultivector { .. } => 3,
        ExternalFin { .. }
        | PropagateFin { .. }
        | AddFin { .. }
        | ScaleFinC { .. }
        | ScaleFinR { .. }
        | GammaFin { .. }
        | FfvFin { .. }
        | ProjFin { .. }
        | Gamma5Fin { .. }
        | MultivectorFin { .. } => 4,
        ExternalFout { .. }
        | PropagateFout { .. }
        | AddFout { .. }
        | ScaleFoutC { .. }
        | ScaleFoutR { .. }
        | GammaFout { .. }
        | FfvFout { .. }
        | ProjFout { .. }
        | Gamma5Fout { .. }
        | MultivectorFout { .. } => 5,
        Flows | Hels | Configs => return None,
    })
}

/// Local-name prefix per result class.
const PREFIX: [char; N_ARENAS] = ['r', 's', 'v', 'm', 'i', 'o'];

/// `Live` field per result class.
const LIVE_FIELD: [&str; N_ARENAS] = ["reals", "scalars", "vectors", "multivectors", "fin", "fout"];

/// A value's identity while rendering: the result class and the position of the
/// instruction that produced it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Val {
    class: usize,
    pos: usize,
}

impl Val {
    fn name(self) -> String {
        format!("{}{}", PREFIX[self.class], self.pos)
    }
}

/// Every instruction's operand values and output, resolved through the symbolic
/// arena: `reads[pos]` in operand order, `writes[pos]` the value produced (if any).
struct Dataflow {
    reads: Vec<Vec<Val>>,
    writes: Vec<Option<Val>>,
    roots: Vec<Val>,
}

fn dataflow(prog: &Program) -> Dataflow {
    let mut cur: [Vec<Option<Val>>; N_ARENAS] =
        std::array::from_fn(|c| vec![None; prog.arena_sizes[c] as usize]);
    let mut reads = Vec::with_capacity(prog.instrs.len());
    let mut writes = Vec::with_capacity(prog.instrs.len());
    for (pos, (instr, &dest)) in prog.instrs.iter().zip(prog.dest.iter()).enumerate() {
        let get = |class: usize, idx: u32| -> Val {
            cur[class][idx as usize].unwrap_or_else(|| {
                panic!("instruction {pos} reads ({class}, {idx}) before a write")
            })
        };
        let ops = |start: u32, len: u32| &prog.operands[start as usize..(start + len) as usize];
        use Instr::*;
        let r: Vec<Val> = match *instr {
            ComplexConst { .. }
            | RealConst { .. }
            | ExternalScalar { .. }
            | ExternalVector { .. }
            | ExternalFin { .. }
            | ExternalFout { .. }
            | PMom { .. }
            | PMomOut { .. }
            | Flows
            | Hels
            | Configs => vec![],
            PropagateScalar {
                input, mass, width, ..
            } => vec![get(1, input), get(0, mass), get(0, width)],
            PropagateVector {
                input, mass, width, ..
            } => vec![get(2, input), get(0, mass), get(0, width)],
            PropagateFin {
                input, mass, width, ..
            } => vec![get(4, input), get(0, mass), get(0, width)],
            PropagateFout {
                input, mass, width, ..
            } => vec![get(5, input), get(0, mass), get(0, width)],
            AddScalar { start, len } => ops(start, len)
                .iter()
                .map(|o| get(1, o.index() as u32))
                .collect(),
            AddVector { start, len } => ops(start, len)
                .iter()
                .map(|o| get(2, o.index() as u32))
                .collect(),
            AddMultivector { start, len } => ops(start, len)
                .iter()
                .map(|o| get(3, o.index() as u32))
                .collect(),
            AddFin { start, len } => ops(start, len)
                .iter()
                .map(|o| get(4, o.index() as u32))
                .collect(),
            AddFout { start, len } => ops(start, len)
                .iter()
                .map(|o| get(5, o.index() as u32))
                .collect(),
            MulScalarC { a, b } => vec![get(1, a), get(1, b)],
            MulScalarR { s, r } => vec![get(1, s), get(0, r)],
            ScaleVecC { v, scale } => vec![get(2, v), get(1, scale)],
            ScaleVecR { v, scale } => vec![get(2, v), get(0, scale)],
            ScaleFinC { f, scale } => vec![get(4, f), get(1, scale)],
            ScaleFinR { f, scale } => vec![get(4, f), get(0, scale)],
            ScaleFoutC { f, scale } => vec![get(5, f), get(1, scale)],
            ScaleFoutR { f, scale } => vec![get(5, f), get(0, scale)],
            ScaleMvC { m, scale } => vec![get(3, m), get(1, scale)],
            ScaleMvR { m, scale } => vec![get(3, m), get(0, scale)],
            GammaVout { bra, ket, .. } => vec![get(5, bra), get(4, ket)],
            FfvVout {
                bra, ket, gl, gr, ..
            } => vec![get(5, bra), get(4, ket), get(1, gl), get(1, gr)],
            GammaFin { v, f } => vec![get(2, v), get(4, f)],
            GammaFout { v, f } => vec![get(2, v), get(5, f)],
            FfvFin { v, f, gl, gr } => vec![get(2, v), get(4, f), get(1, gl), get(1, gr)],
            FfvFout { v, f, gl, gr } => vec![get(2, v), get(5, f), get(1, gl), get(1, gr)],
            ProjFin { f, .. } | Gamma5Fin { f } => vec![get(4, f)],
            ProjFout { f, .. } | Gamma5Fout { f } => vec![get(5, f)],
            Bilinear { bra, ket, .. } | Pseudoscalar { bra, ket } => {
                vec![get(5, bra), get(4, ket)]
            }
            Metric { a, b } => vec![get(2, a), get(2, b)],
            MetricVout { v } => vec![get(2, v)],
            EpsilonVout { a, b, c } => vec![get(2, a), get(2, b), get(2, c)],
            EpsilonAmp { a, b, c, d } => vec![get(2, a), get(2, b), get(2, c), get(2, d)],
            FierzOut { bra, ket, .. } | SigmaOut { bra, ket, .. } => {
                vec![get(5, bra), get(4, ket)]
            }
            MultivectorFin { m, f } => vec![get(3, m), get(4, f)],
            MultivectorFout { m, f } => vec![get(3, m), get(5, f)],
            FierzPair { m, bra, ket } => vec![get(3, m), get(5, bra), get(4, ket)],
            SigmaVout { bra, ket, v, .. } => vec![get(5, bra), get(4, ket), get(2, v)],
            SigmaMv { a, b } => vec![get(2, a), get(2, b)],
        };
        let w = out_class(instr).map(|class| Val { class, pos });
        if let Some(v) = w {
            cur[v.class][dest as usize] = Some(v);
        }
        reads.push(r);
        writes.push(w);
    }
    let RootKind::Hels { locs, .. } = &prog.root else {
        panic!("rendering needs a helicity-expanded root");
    };
    let roots = locs
        .iter()
        .map(|&l| cur[1][l as usize].expect("root amplitude never written"))
        .collect();
    Dataflow {
        reads,
        writes,
        roots,
    }
}

/// The expression instruction `pos` evaluates, over its operands' locals. Each arm is
/// the arithmetic the matching arm of the forward pass performs, operand for operand.
fn expr(eval: &AmplitudeEvaluator, prog: &Program, pos: usize, ops: &[Val]) -> String {
    let n = |k: usize| ops[k].name();
    let legs = eval.folded_hel().ext_legs();
    let ext = |leg: u32, wrap: &str| {
        let l = legs[leg as usize];
        let hel = l.hel.expect("helicity-expanded leg carries its helicity");
        format!(
            "{wrap}(build_external_core(mo[{}], {}, {}, Charge::{:?}, {}, cr[{}]))",
            l.leg_idx, hel, l.spin, l.charge, l.incoming, l.mass
        )
    };
    let fold = |op: &str| {
        let mut s = n(0);
        for k in 1..ops.len() {
            write!(s, " {op} {}", n(k)).unwrap();
        }
        s
    };
    use Instr::*;
    match prog.instrs[pos] {
        ComplexConst { pool } => format!("cc[{pool}]"),
        RealConst { pool } => format!("cr[{pool}]"),
        ExternalScalar { leg } => ext(leg, "ext_scalar"),
        ExternalVector { leg } => ext(leg, "ext_vector"),
        ExternalFin { leg } => ext(leg, "ext_fin"),
        ExternalFout { leg } => ext(leg, "ext_fout"),
        PropagateScalar { mom, .. } => format!(
            "k::propagate_scalar_bare({}, &mm[{mom}], {}, {})",
            n(0),
            n(1),
            n(2)
        ),
        PropagateVector { mom, .. } => format!(
            "k::propagate_vector_bare(&{}, &mm[{mom}], {}, {})",
            n(0),
            n(1),
            n(2)
        ),
        PropagateFin { mom, .. } => format!(
            "k::propagate_fin_bare(&{}, &mm[{mom}], {}, {})",
            n(0),
            n(1),
            n(2)
        ),
        PropagateFout { mom, .. } => format!(
            "k::propagate_fout_bare(&{}, &mm[{mom}], {}, {})",
            n(0),
            n(1),
            n(2)
        ),
        AddScalar { .. } | AddVector { .. } | AddFin { .. } | AddFout { .. } => fold("+"),
        AddMultivector { .. } => fold("+"),
        MulScalarC { .. }
        | MulScalarR { .. }
        | ScaleVecC { .. }
        | ScaleVecR { .. }
        | ScaleFinC { .. }
        | ScaleFinR { .. }
        | ScaleFoutC { .. }
        | ScaleFoutR { .. }
        | ScaleMvC { .. }
        | ScaleMvR { .. } => format!("{} * {}", n(0), n(1)),
        GammaVout { reversed, .. } => {
            format!("k::gamma_vout_bare(&{}, &{}, {reversed})", n(0), n(1))
        }
        FfvVout { reversed, .. } => format!(
            "k::ffv_vout_bare(&{}, &{}, {}, {}, {reversed})",
            n(0),
            n(1),
            n(2),
            n(3)
        ),
        GammaFin { .. } => format!("k::off_shell_fin_bare(&{}, &{})", n(0), n(1)),
        GammaFout { .. } => format!("k::off_shell_fout_bare(&{}, &{})", n(0), n(1)),
        FfvFin { .. } => format!("k::ffv_fin_bare(&{}, &{}, {}, {})", n(0), n(1), n(2), n(3)),
        FfvFout { .. } => format!("k::ffv_fout_bare(&{}, &{}, {}, {})", n(0), n(1), n(2), n(3)),
        ProjFin { chirality, .. } => {
            format!("k::proj_fin_bare(&{}, Chirality::{chirality:?})", n(0))
        }
        ProjFout { chirality, .. } => {
            format!("k::proj_fout_bare(&{}, Chirality::{chirality:?})", n(0))
        }
        Gamma5Fin { .. } => format!("k::gamma5_fin_bare(&{})", n(0)),
        Gamma5Fout { .. } => format!("k::gamma5_fout_bare(&{})", n(0)),
        Bilinear { chirality, .. } => format!(
            "k::scalar_bilinear_bare(&{}, &{}, Chirality::{chirality:?})",
            n(0),
            n(1)
        ),
        Pseudoscalar { .. } => format!("k::pseudoscalar_bilinear_bare(&{}, &{})", n(0), n(1)),
        Metric { .. } => format!("k::metric_bare(&{}, &{})", n(0), n(1)),
        MetricVout { .. } => format!("k::metric_vout_bare(&{})", n(0)),
        EpsilonVout { .. } => format!("k::epsilon_vout_bare(&{}, &{}, &{})", n(0), n(1), n(2)),
        EpsilonAmp { .. } => format!(
            "k::epsilon_amp_bare(&{}, &{}, &{}, &{})",
            n(0),
            n(1),
            n(2),
            n(3)
        ),
        PMom { mom } => format!("k::pmom_bare(&mm[{mom}])"),
        PMomOut { start, len } => {
            let mut s = String::from("k::pmom_bare(&-(LorentzVector::<F>::zero()");
            for &(mid, sign) in &prog.mom_operands[start as usize..(start + len) as usize] {
                let op = if sign < 0 { '-' } else { '+' };
                write!(s, " {op} mm[{mid}]").unwrap();
            }
            s.push_str("))");
            s
        }
        FierzOut { reversed_order, .. } => {
            format!("k::fierz_out_bare(&{}, &{}, {reversed_order})", n(0), n(1))
        }
        MultivectorFin { .. } => format!("k::multivector_fin_bare(&{}, &{})", n(0), n(1)),
        MultivectorFout { .. } => {
            format!("k::multivector_fout_bare(&{}, &{})", n(0), n(1))
        }
        FierzPair { .. } => format!("k::fierz_pair_bare(&{}, &{}, &{})", n(0), n(1), n(2)),
        SigmaVout { negate, .. } => format!(
            "k::sigma_vout_bare(&{}, &{}, &{}, {negate})",
            n(0),
            n(1),
            n(2)
        ),
        SigmaMv { .. } => format!("k::sigma_mv_bare(&{}, &{})", n(0), n(1)),
        SigmaOut { reversed_order, .. } => {
            format!("k::sigma_out_bare(&{}, &{}, {reversed_order})", n(0), n(1))
        }
        Flows | Hels | Configs => unreachable!("variadic roots render no expression"),
    }
}

/// Render `eval`'s helicity-expanded program as the source of `aot_<name>`.
///
/// With `chunk == 0` the program is one function. Otherwise it is split into
/// functions of `chunk` consecutive instructions, called in order by `aot_<name>`; a
/// value produced in one chunk and read in a later one is stored to a [`Live`] array
/// at its production and loaded once at the top of each chunk that reads it.
pub(super) fn render(name: &str, eval: &AmplitudeEvaluator, chunk: usize) -> String {
    let folded = eval.folded_hel();
    let prog = folded.program();
    let df = dataflow(prog);
    let n_instr = prog.instrs.len();
    let n_ext = eval.n_ext();
    let (n_cc, n_cf) = eval.folded().pools_len();
    let n_mm = folded.analysis().mom_table().len();
    let n_roots = df.roots.len();
    let chunk_arg = chunk;
    let chunk = if chunk == 0 { n_instr.max(1) } else { chunk };
    let n_chunks = n_instr.div_ceil(chunk).max(1);
    let chunk_of = |pos: usize| pos / chunk;

    // Cross-chunk values: those read in a chunk after their producer's, or read out as
    // a root from a chunk before the last. Each holds a slot of its class's `Live`
    // array from its producing chunk to the last chunk that reads it; a later value
    // reuses the slot from that chunk on, since a chunk loads all its inputs before it
    // stores anything.
    let mut last_chunk: std::collections::HashMap<Val, usize> = Default::default();
    for (pos, r) in df.reads.iter().enumerate() {
        for &v in r {
            if chunk_of(v.pos) != chunk_of(pos) {
                let e = last_chunk.entry(v).or_insert(0);
                *e = (*e).max(chunk_of(pos));
            }
        }
    }
    for &v in &df.roots {
        if chunk_of(v.pos) != n_chunks - 1 {
            last_chunk.insert(v, n_chunks - 1);
        }
    }
    let mut crossing: Vec<(Val, usize)> = last_chunk.into_iter().collect();
    crossing.sort_by_key(|&(v, _)| v.pos);
    let mut live_slot: std::collections::HashMap<Val, usize> = Default::default();
    let mut live_count = [0usize; N_ARENAS];
    let mut free: [Vec<usize>; N_ARENAS] = Default::default();
    let mut held: [Vec<(usize, usize)>; N_ARENAS] = Default::default();
    for (v, last) in crossing {
        let c = v.class;
        let here = chunk_of(v.pos);
        held[c].retain(|&(until, slot)| {
            let keep = until > here;
            if !keep {
                free[c].push(slot);
            }
            keep
        });
        let slot = free[c].pop().unwrap_or_else(|| {
            live_count[c] += 1;
            live_count[c] - 1
        });
        held[c].push((last, slot));
        live_slot.insert(v, slot);
    }

    let mut out = String::new();
    writeln!(
        out,
        "// chunk: {chunk_arg}. Rendered from the helicity-expanded program of `{name}` \
         ({n_instr} instructions, {n_chunks} function(s), cross-chunk slots per class \
         {live_count:?}); regenerate, do not edit."
    )
    .unwrap();
    let args = "mo: &[LorentzVector<F>; {n_ext}], cc: &[C<F>; {n_cc}], cr: &[F; {n_cf}], \
                mm: &[LorentzVector<F>; {n_mm}]";
    let args = args
        .replace("{n_ext}", &n_ext.to_string())
        .replace("{n_cc}", &n_cc.to_string())
        .replace("{n_cf}", &n_cf.to_string())
        .replace("{n_mm}", &n_mm.to_string());

    // Entry point.
    writeln!(
        out,
        "#[inline(never)]\npub(super) fn aot_{name}<F: Real>(mo: &[LorentzVector<F>], \
         cc: &[C<F>], cr: &[F], mm: &[LorentzVector<F>], roots: &mut [C<F>]) {{"
    )
    .unwrap();
    writeln!(
        out,
        "    let mo: &[LorentzVector<F>; {n_ext}] = mo.try_into().expect(\"external momenta\");\n    \
         let cc: &[C<F>; {n_cc}] = cc.try_into().expect(\"complex pool\");\n    \
         let cr: &[F; {n_cf}] = cr.try_into().expect(\"real pool\");\n    \
         let mm: &[LorentzVector<F>; {n_mm}] = mm.try_into().expect(\"momentum pool\");\n    \
         let roots: &mut [C<F>; {n_roots}] = roots.try_into().expect(\"root count\");"
    )
    .unwrap();
    if n_chunks == 1 {
        render_body(&mut out, eval, prog, &df, 0..n_instr, &live_slot, false);
        for (k, v) in df.roots.iter().enumerate() {
            writeln!(out, "    roots[{k}] = {};", v.name()).unwrap();
        }
        out.push_str("}\n");
        return out;
    }
    writeln!(
        out,
        "    let mut live = Live::<F> {{ reals: vec![F::zero(); {}], scalars: vec![C::new(F::zero(), \
         F::zero()); {}], vectors: vec![Zero::zero(); {}], multivectors: vec![Zero::zero(); {}], \
         fin: vec![Zero::zero(); {}], fout: vec![Zero::zero(); {}] }};",
        live_count[0], live_count[1], live_count[2], live_count[3], live_count[4], live_count[5]
    )
    .unwrap();
    for c in 0..n_chunks {
        if c + 1 == n_chunks {
            writeln!(out, "    aot_{name}_{c}(mo, cc, cr, mm, &mut live, roots);").unwrap();
        } else {
            writeln!(out, "    aot_{name}_{c}(mo, cc, cr, mm, &mut live);").unwrap();
        }
    }
    out.push_str("}\n");
    for c in 0..n_chunks {
        let range = c * chunk..((c + 1) * chunk).min(n_instr);
        let last = c + 1 == n_chunks;
        let roots_arg = if last {
            format!(", roots: &mut [C<F>; {n_roots}]")
        } else {
            String::new()
        };
        writeln!(
            out,
            "#[inline(never)]\nfn aot_{name}_{c}<F: Real>({args}, live: &mut Live<F>{roots_arg}) {{"
        )
        .unwrap();
        // Values produced before this chunk and read in it, loaded once.
        let mut loads: Vec<Val> = Vec::new();
        for pos in range.clone() {
            for &v in &df.reads[pos] {
                if v.pos < range.start && !loads.contains(&v) {
                    loads.push(v);
                }
            }
        }
        if last {
            for &v in &df.roots {
                if v.pos < range.start && !loads.contains(&v) {
                    loads.push(v);
                }
            }
        }
        for v in loads {
            writeln!(
                out,
                "    let {} = live.{}[{}];",
                v.name(),
                LIVE_FIELD[v.class],
                live_slot[&v]
            )
            .unwrap();
        }
        render_body(&mut out, eval, prog, &df, range, &live_slot, true);
        if last {
            for (k, v) in df.roots.iter().enumerate() {
                writeln!(out, "    roots[{k}] = {};", v.name()).unwrap();
            }
        }
        out.push_str("}\n");
    }
    out
}

/// One `let` per instruction in `range`, plus, when `store` is set, the store of each
/// cross-chunk value to its [`Live`] slot right after it is produced.
fn render_body(
    out: &mut String,
    eval: &AmplitudeEvaluator,
    prog: &Program,
    df: &Dataflow,
    range: std::ops::Range<usize>,
    live_slot: &std::collections::HashMap<Val, usize>,
    store: bool,
) {
    for pos in range {
        let Some(w) = df.writes[pos] else {
            continue;
        };
        writeln!(
            out,
            "    let {} = {};",
            w.name(),
            expr(eval, prog, pos, &df.reads[pos])
        )
        .unwrap();
        if store {
            if let Some(&slot) = live_slot.get(&w) {
                writeln!(
                    out,
                    "    live.{}[{slot}] = {};",
                    LIVE_FIELD[w.class],
                    w.name()
                )
                .unwrap();
            }
        }
    }
}

/// Per-row structure of a rendered program, for the study's tables.
#[cfg(test)]
#[derive(Debug)]
pub(super) struct RenderStats {
    pub instrs: usize,
    /// Values produced and never read, by the program or the read-out.
    pub dead: usize,
    pub roots: usize,
}

#[cfg(test)]
pub(super) fn render_stats(eval: &AmplitudeEvaluator) -> RenderStats {
    let prog = eval.folded_hel().program();
    let df = dataflow(prog);
    let mut read = std::collections::HashSet::new();
    for r in &df.reads {
        read.extend(r.iter().copied());
    }
    read.extend(df.roots.iter().copied());
    let dead = df
        .writes
        .iter()
        .flatten()
        .filter(|w| !read.contains(w))
        .count();
    RenderStats {
        instrs: prog.instrs.len(),
        dead,
        roots: df.roots.len(),
    }
}
