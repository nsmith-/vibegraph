//! Rendering of a compiled helicity program in MadGraph's form.
//!
//! Every value lives in memory: one fixed-size slot array per result class
//! ([`MgArenas`]), sized and indexed exactly as the interpreter's arenas
//! (`Program::arena_sizes` and `Program::dest`). Every instruction becomes one call to
//! an out-of-line entry point of [`out_param`](super::kernels::out_param), which takes
//! its operands by reference into those arrays and writes its result through `&mut`
//! into its own slot. The rendered function is that sequence of calls and nothing else:
//! no value is held in a local, and every index is a literal into an array of literal
//! length, so no access carries a bounds check.
//!
//! Constant-pool loads (`ComplexConst`, `RealConst`) emit no call: a read of the slot
//! they filled names the pool entry instead, as MadGraph passes a coupling straight
//! from its coupling array. The value read is the same.
//!
//! A destination slot is never one of its instruction's operand slots (the slot
//! allocator assigns the result before it releases the operands), so an operand of
//! the destination's own class is borrowed from the two halves of that class's array
//! on either side of the destination ([`sp`]).

use std::fmt::Write as _;

use super::super::compile::AmplitudeEvaluator;
use super::super::layout::{Instr, Program, RootKind, N_ARENAS};
use crate::helas::repr::lorentz::{Bispinor, Bra, ComplexVector, Ket, LorentzVector, Multivector};
use crate::helas::repr::{Real, C};
use num_traits::Zero;

/// The value slots of one rendered program, one array per result class: `s` complex
/// scalars, `v` vectors, `m` multivectors, `i` kets, `o` bras.
pub(super) struct MgArenas<
    F: Real,
    const S: usize,
    const V: usize,
    const M: usize,
    const I: usize,
    const O: usize,
> {
    pub(super) s: Box<[C<F>; S]>,
    pub(super) v: Box<[ComplexVector<F>; V]>,
    #[allow(dead_code, reason = "no rendered row has a multivector instruction")]
    pub(super) m: Box<[Multivector<F>; M]>,
    pub(super) i: Box<[Bispinor<F, Ket>; I]>,
    pub(super) o: Box<[Bispinor<F, Bra>; O]>,
}

fn boxed<T: Clone, const N: usize>(zero: T) -> Box<[T; N]> {
    match vec![zero; N].into_boxed_slice().try_into() {
        Ok(b) => b,
        Err(_) => unreachable!("a vector of N elements converts to [T; N]"),
    }
}

impl<F: Real, const S: usize, const V: usize, const M: usize, const I: usize, const O: usize>
    MgArenas<F, S, V, M, I, O>
{
    fn new() -> Self {
        MgArenas {
            s: boxed(C::new(F::zero(), F::zero())),
            v: boxed(Zero::zero()),
            m: boxed(Zero::zero()),
            i: boxed(Zero::zero()),
            o: boxed(Zero::zero()),
        }
    }
}

/// The slot `d` of `a` mutably, with the slots below and above it shared.
#[inline(always)]
pub(super) fn sp<T, const N: usize>(a: &mut [T; N], d: usize) -> (&[T], &mut T, &[T]) {
    let (lo, hi) = a.split_at_mut(d);
    let (dst, hi) = hi
        .split_first_mut()
        .expect("destination slot inside its array");
    (lo, dst, hi)
}

/// A rendered program bound to its slot arrays: one pass over a point, then the
/// complex-scalar slots the read-out reads the root amplitudes from.
pub(super) trait MgRun<F: Real> {
    /// Run the program: external momenta, complex pool, real pool, momentum pool.
    fn run(&mut self, mo: &[LorentzVector<F>], cc: &[C<F>], cr: &[F], mm: &[LorentzVector<F>]);
    fn scalars(&self) -> &[C<F>];
}

/// A rendered entry point over its own slot arrays.
type MgEntry<F, const S: usize, const V: usize, const M: usize, const I: usize, const O: usize> =
    fn(&mut MgArenas<F, S, V, M, I, O>, &[LorentzVector<F>], &[C<F>], &[F], &[LorentzVector<F>]);

struct MgBound<
    F: Real,
    const S: usize,
    const V: usize,
    const M: usize,
    const I: usize,
    const O: usize,
> {
    arenas: MgArenas<F, S, V, M, I, O>,
    entry: MgEntry<F, S, V, M, I, O>,
}

impl<F: Real, const S: usize, const V: usize, const M: usize, const I: usize, const O: usize>
    MgRun<F> for MgBound<F, S, V, M, I, O>
{
    fn run(&mut self, mo: &[LorentzVector<F>], cc: &[C<F>], cr: &[F], mm: &[LorentzVector<F>]) {
        (self.entry)(&mut self.arenas, mo, cc, cr, mm);
    }

    fn scalars(&self) -> &[C<F>] {
        &self.arenas.s[..]
    }
}

fn bound<
    F: Real,
    const S: usize,
    const V: usize,
    const M: usize,
    const I: usize,
    const O: usize,
>(
    entry: MgEntry<F, S, V, M, I, O>,
) -> Box<dyn MgRun<F>> {
    Box::new(MgBound {
        arenas: MgArenas::new(),
        entry,
    })
}

#[allow(
    clippy::all,
    unused_imports,
    reason = "rendered code: one call per instruction with literal operands"
)]
mod rendered {
    use super::super::kernels::out_param as k;
    use super::{bound, sp, MgArenas, MgRun};
    use crate::helas::repr::lorentz::LorentzVector;
    use crate::helas::repr::numbers::{Charge, Chirality};
    use crate::helas::repr::{Real, C};

    include!("generated/mg/ee_to_mumu.rs");
    include!("generated/mg/gg_to_gg.rs");
    include!("generated/mg/ee_to_mumu_tata_qcd0.rs");
    #[cfg(feature = "aot-mg-study-large")]
    include!("generated/mg/uux_to_ccx_emmm_qcd0.rs");
}

/// Row `name`'s rendered program over freshly zeroed slot arrays, at scalar type `F`.
pub(super) fn bind<F: Real>(name: &str) -> Option<Box<dyn MgRun<F>>> {
    Some(match name {
        "ee_to_mumu" => rendered::bind_ee_to_mumu::<F>(),
        "gg_to_gg" => rendered::bind_gg_to_gg::<F>(),
        "ee_to_mumu_tata_qcd0" => rendered::bind_ee_to_mumu_tata_qcd0::<F>(),
        #[cfg(feature = "aot-mg-study-large")]
        "uux_to_ccx_emmm_qcd0" => rendered::bind_uux_to_ccx_emmm_qcd0::<F>(),
        _ => return None,
    })
}

/// Slot-array field per result class; the real class is only ever filled from the
/// real pool, so it has none.
const FIELD: [&str; N_ARENAS] = ["", "s", "v", "m", "i", "o"];

/// One operand of a rendered call.
enum Arg {
    /// The value in slot `(class, slot)`.
    Val(usize, u32),
    /// Several values of one class, as a slice of references (the `Add*` terms).
    Vals(usize, Vec<u32>),
    /// An entry of the per-point momentum pool.
    Mom(u32),
    /// Literal text.
    Lit(String),
}

/// The entry point instruction `instr` calls and its operands after `out`, in the
/// order the forward pass passes them to the kernel. `None` for the constant loads
/// and the variadic roots, which emit no call.
fn call(
    eval: &AmplitudeEvaluator,
    prog: &Program,
    instr: &Instr,
) -> Option<(&'static str, Vec<Arg>)> {
    use Arg::*;
    use Instr::*;
    let ext = |leg: u32, f: &'static str| {
        let l = eval.folded_hel().ext_legs()[leg as usize];
        let hel = l.hel.expect("helicity-expanded leg carries its helicity");
        (
            f,
            vec![Lit(format!(
                "&mo[{}], {}, {}, Charge::{:?}, {}, &cr[{}]",
                l.leg_idx, hel, l.spin, l.charge, l.incoming, l.mass
            ))],
        )
    };
    let terms = |class: usize, start: u32, len: u32| {
        Vals(
            class,
            prog.operands[start as usize..(start + len) as usize]
                .iter()
                .map(|o| o.index() as u32)
                .collect(),
        )
    };
    let prop = |f: &'static str, class: usize, input: u32, mom: u32, mass: u32, width: u32| {
        (
            f,
            vec![Val(class, input), Mom(mom), Val(0, mass), Val(0, width)],
        )
    };
    Some(match *instr {
        ComplexConst { .. } | RealConst { .. } | Flows | Hels | Configs => return None,
        ExternalScalar { leg } => ext(leg, "ext_scalar"),
        ExternalVector { leg } => ext(leg, "ext_vector"),
        ExternalFin { leg } => ext(leg, "ext_fin"),
        ExternalFout { leg } => ext(leg, "ext_fout"),
        PropagateScalar {
            input,
            mass,
            width,
            mom,
        } => prop("propagate_scalar", 1, input, mom, mass, width),
        PropagateVector {
            input,
            mass,
            width,
            mom,
        } => prop("propagate_vector", 2, input, mom, mass, width),
        PropagateFin {
            input,
            mass,
            width,
            mom,
        } => prop("propagate_fin", 4, input, mom, mass, width),
        PropagateFout {
            input,
            mass,
            width,
            mom,
        } => prop("propagate_fout", 5, input, mom, mass, width),
        AddScalar { start, len } => ("add", vec![terms(1, start, len)]),
        AddVector { start, len } => ("add", vec![terms(2, start, len)]),
        AddMultivector { start, len } => ("add", vec![terms(3, start, len)]),
        AddFin { start, len } => ("add", vec![terms(4, start, len)]),
        AddFout { start, len } => ("add", vec![terms(5, start, len)]),
        MulScalarC { a, b } => ("mul", vec![Val(1, a), Val(1, b)]),
        MulScalarR { s, r } => ("mul", vec![Val(1, s), Val(0, r)]),
        ScaleVecC { v, scale } => ("mul", vec![Val(2, v), Val(1, scale)]),
        ScaleVecR { v, scale } => ("mul", vec![Val(2, v), Val(0, scale)]),
        ScaleFinC { f, scale } => ("mul", vec![Val(4, f), Val(1, scale)]),
        ScaleFinR { f, scale } => ("mul", vec![Val(4, f), Val(0, scale)]),
        ScaleFoutC { f, scale } => ("mul", vec![Val(5, f), Val(1, scale)]),
        ScaleFoutR { f, scale } => ("mul", vec![Val(5, f), Val(0, scale)]),
        ScaleMvC { m, scale } => ("mul", vec![Val(3, m), Val(1, scale)]),
        ScaleMvR { m, scale } => ("mul", vec![Val(3, m), Val(0, scale)]),
        GammaVout { bra, ket, reversed } => (
            "gamma_vout",
            vec![Val(5, bra), Val(4, ket), Lit(reversed.to_string())],
        ),
        FfvVout {
            bra,
            ket,
            gl,
            gr,
            reversed,
        } => (
            "ffv_vout",
            vec![
                Val(5, bra),
                Val(4, ket),
                Val(1, gl),
                Val(1, gr),
                Lit(reversed.to_string()),
            ],
        ),
        GammaFin { v, f } => ("off_shell_fin", vec![Val(2, v), Val(4, f)]),
        GammaFout { v, f } => ("off_shell_fout", vec![Val(2, v), Val(5, f)]),
        FfvFin { v, f, gl, gr } => (
            "ffv_fin",
            vec![Val(2, v), Val(4, f), Val(1, gl), Val(1, gr)],
        ),
        FfvFout { v, f, gl, gr } => (
            "ffv_fout",
            vec![Val(2, v), Val(5, f), Val(1, gl), Val(1, gr)],
        ),
        ProjFin { f, chirality } => (
            "proj_fin",
            vec![Val(4, f), Lit(format!("Chirality::{chirality:?}"))],
        ),
        ProjFout { f, chirality } => (
            "proj_fout",
            vec![Val(5, f), Lit(format!("Chirality::{chirality:?}"))],
        ),
        Gamma5Fin { f } => ("gamma5_fin", vec![Val(4, f)]),
        Gamma5Fout { f } => ("gamma5_fout", vec![Val(5, f)]),
        Bilinear {
            bra,
            ket,
            chirality,
        } => (
            "scalar_bilinear",
            vec![
                Val(5, bra),
                Val(4, ket),
                Lit(format!("Chirality::{chirality:?}")),
            ],
        ),
        Pseudoscalar { bra, ket } => ("pseudoscalar_bilinear", vec![Val(5, bra), Val(4, ket)]),
        Metric { a, b } => ("metric", vec![Val(2, a), Val(2, b)]),
        MetricVout { v } => ("metric_vout", vec![Val(2, v)]),
        EpsilonVout { a, b, c } => ("epsilon_vout", vec![Val(2, a), Val(2, b), Val(2, c)]),
        EpsilonAmp { a, b, c, d } => (
            "epsilon_amp",
            vec![Val(2, a), Val(2, b), Val(2, c), Val(2, d)],
        ),
        PMom { mom } => ("pmom", vec![Mom(mom)]),
        PMomOut { start, len } => {
            let mut s = String::from("mm, &[");
            for (k, &(mid, sign)) in prog.mom_operands[start as usize..(start + len) as usize]
                .iter()
                .enumerate()
            {
                if k > 0 {
                    s.push_str(", ");
                }
                write!(s, "({mid}, {sign})").unwrap();
            }
            s.push(']');
            ("pmom_out", vec![Lit(s)])
        }
        FierzOut {
            bra,
            ket,
            reversed_order,
        } => (
            "fierz_out",
            vec![Val(5, bra), Val(4, ket), Lit(reversed_order.to_string())],
        ),
        MultivectorFin { m, f } => ("multivector_fin", vec![Val(3, m), Val(4, f)]),
        MultivectorFout { m, f } => ("multivector_fout", vec![Val(3, m), Val(5, f)]),
        FierzPair { m, bra, ket } => ("fierz_pair", vec![Val(3, m), Val(5, bra), Val(4, ket)]),
        SigmaVout {
            bra,
            ket,
            v,
            negate,
        } => (
            "sigma_vout",
            vec![Val(5, bra), Val(4, ket), Val(2, v), Lit(negate.to_string())],
        ),
        SigmaMv { a, b } => ("sigma_mv", vec![Val(2, a), Val(2, b)]),
        SigmaOut {
            bra,
            ket,
            reversed_order,
        } => (
            "sigma_out",
            vec![Val(5, bra), Val(4, ket), Lit(reversed_order.to_string())],
        ),
    })
}

/// Result class of every instruction that emits a call.
fn out_class(instr: &Instr) -> usize {
    super::byvalue::out_class(instr).expect("a calling instruction writes a slot")
}

/// Render `eval`'s helicity-expanded program in MadGraph's form, as the source of
/// `mg_<name>` and its binder `bind_<name>`.
///
/// With `chunk == 0` the calls form one function; otherwise the program is split into
/// functions of `chunk` consecutive instructions, called in order. No value crosses a
/// function boundary except through the slot arrays, so splitting adds only the calls.
pub(super) fn render(name: &str, eval: &AmplitudeEvaluator, chunk: usize) -> String {
    let folded = eval.folded_hel();
    let prog = folded.program();
    let sizes = prog.arena_sizes;
    let n_instr = prog.instrs.len();
    let n_ext = eval.n_ext();
    let (n_cc, n_cf) = eval.folded().pools_len();
    let n_mm = folded.analysis().mom_table().len();
    let chunk_arg = chunk;
    let chunk = if chunk == 0 { n_instr.max(1) } else { chunk };
    let n_chunks = n_instr.div_ceil(chunk).max(1);

    // `konst[class][slot]`: the pool entry a slot currently holds, when the latest
    // write to it was a constant load.
    let mut konst: [Vec<Option<u32>>; N_ARENAS] =
        std::array::from_fn(|c| vec![None; sizes[c] as usize]);
    let mut bodies = vec![String::new(); n_chunks];
    let mut n_calls = 0usize;
    for (pos, (instr, &dest)) in prog.instrs.iter().zip(prog.dest.iter()).enumerate() {
        match *instr {
            Instr::ComplexConst { pool } => {
                konst[1][dest as usize] = Some(pool);
                continue;
            }
            Instr::RealConst { pool } => {
                konst[0][dest as usize] = Some(pool);
                continue;
            }
            _ => {}
        }
        let Some((f, args)) = call(eval, prog, instr) else {
            continue;
        };
        let class = out_class(instr);
        let d = dest as usize;
        let (mut lo, mut hi) = (false, false);
        let mut operand = |c: usize, slot: u32| -> String {
            let slot = slot as usize;
            if let Some(p) = konst[c][slot] {
                return if c == 0 {
                    format!("&cr[{p}]")
                } else {
                    format!("&cc[{p}]")
                };
            }
            assert!(
                c != 0,
                "{name}: real slot {slot} not filled from the real pool"
            );
            if c != class {
                return format!("&a.{}[{slot}]", FIELD[c]);
            }
            assert!(
                slot != d,
                "{name}: instruction {pos} reads its own destination slot"
            );
            if slot < d {
                lo = true;
                format!("&l[{slot}]")
            } else {
                hi = true;
                format!("&h[{}]", slot - d - 1)
            }
        };
        let mut rendered: Vec<String> = Vec::with_capacity(args.len());
        for arg in &args {
            rendered.push(match arg {
                Arg::Val(c, slot) => operand(*c, *slot),
                Arg::Vals(c, slots) => {
                    let refs: Vec<String> = slots.iter().map(|&s| operand(*c, s)).collect();
                    format!("&[{}]", refs.join(", "))
                }
                Arg::Mom(m) => format!("&mm[{m}]"),
                Arg::Lit(s) => s.clone(),
            });
        }
        let body = &mut bodies[pos / chunk];
        let field = FIELD[class];
        if lo || hi {
            let l = if lo { "l" } else { "_" };
            let h = if hi { "h" } else { "_" };
            writeln!(
                body,
                "    {{ let ({l}, d, {h}) = sp(&mut a.{field}, {d}); k::{f}(d, {}); }}",
                rendered.join(", ")
            )
            .unwrap();
        } else {
            writeln!(
                body,
                "    k::{f}(&mut a.{field}[{d}], {});",
                rendered.join(", ")
            )
            .unwrap();
        }
        konst[class][d] = None;
        n_calls += 1;
    }
    let RootKind::Hels { locs, .. } = &prog.root else {
        panic!("rendering needs a helicity-expanded root");
    };
    for &l in locs.iter() {
        assert!(
            konst[1][l as usize].is_none(),
            "{name}: root slot {l} holds a pool constant, which the read-out would not find"
        );
    }

    let consts = format!(
        "{}, {}, {}, {}, {}",
        sizes[1], sizes[2], sizes[3], sizes[4], sizes[5]
    );
    let arrays = format!(
        "a: &mut MgArenas<F, {consts}>, mo: &[LorentzVector<F>; {n_ext}], cc: &[C<F>; {n_cc}], \
         cr: &[F; {n_cf}], mm: &[LorentzVector<F>; {n_mm}]"
    );
    let mut out = String::new();
    writeln!(
        out,
        "// chunk: {chunk_arg}. Rendered from the helicity-expanded program of `{name}` \
         ({n_instr} instructions, {n_calls} calls, {n_chunks} function(s), slots per class \
         {sizes:?}); regenerate, do not edit."
    )
    .unwrap();
    writeln!(
        out,
        "pub(super) fn bind_{name}<F: Real>() -> Box<dyn MgRun<F>> {{\n    \
         bound::<F, {consts}>(mg_{name}::<F>)\n}}"
    )
    .unwrap();
    writeln!(
        out,
        "#[inline(never)]\nfn mg_{name}<F: Real>(a: &mut MgArenas<F, {consts}>, \
         mo: &[LorentzVector<F>], cc: &[C<F>], cr: &[F], mm: &[LorentzVector<F>]) {{\n    \
         let mo: &[LorentzVector<F>; {n_ext}] = mo.try_into().expect(\"external momenta\");\n    \
         let cc: &[C<F>; {n_cc}] = cc.try_into().expect(\"complex pool\");\n    \
         let cr: &[F; {n_cf}] = cr.try_into().expect(\"real pool\");\n    \
         let mm: &[LorentzVector<F>; {n_mm}] = mm.try_into().expect(\"momentum pool\");"
    )
    .unwrap();
    if n_chunks == 1 {
        out.push_str(&bodies[0]);
        out.push_str("}\n");
        return out;
    }
    for c in 0..n_chunks {
        writeln!(out, "    mg_{name}_{c}(a, mo, cc, cr, mm);").unwrap();
    }
    out.push_str("}\n");
    for (c, body) in bodies.iter().enumerate() {
        writeln!(
            out,
            "#[inline(never)]\nfn mg_{name}_{c}<F: Real>({arrays}) {{"
        )
        .unwrap();
        out.push_str(body);
        out.push_str("}\n");
    }
    out
}
