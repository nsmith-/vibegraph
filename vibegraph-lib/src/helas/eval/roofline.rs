//! Per-event operation and arena-traffic census of the forward pass: the inputs to a
//! roofline reading of the evaluator.
//!
//! Two independent counts, each exact for what it measures:
//!
//! - **Arithmetic**, by running `eval_m2` at [`Counted`], an `f64` newtype that tallies
//!   every floating-point operation it performs into a thread-local counter. The run is
//!   the production code path at a different `F`, so the count covers every kernel,
//!   the momentum pool, the external wavefunctions and the helicity/colour read-out —
//!   whatever the scalar build executes, op for op. Each counted value is the `f64`
//!   result bit for bit, which the census asserts against a plain `f64` run.
//! - **Arena traffic**, from the compiled program: each instruction's reads are its
//!   [`arena_reads`] operands (the same edge set slot recycling is computed from, so a
//!   read missing here would be a value the allocator could overwrite before use) plus
//!   the constant-pool and momentum-pool entries it indexes, and its write is one element
//!   of its own result class. Sizes are at `F = f64`; a lane pack of width `N` moves `N`
//!   times the bytes per instruction for `N` events, so bytes per event do not change.
//!
//! Traffic counts the values an instruction consumes and produces, not machine loads:
//! the interpreter's own instruction-stream reads, bounds metadata, spills and the
//! by-value copies a kernel call may make are not in it.

use std::cell::Cell;
use std::num::FpCategory;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use num_traits::{Float, FloatConst, FromPrimitive, Num, NumCast, One, ToPrimitive, Zero};

/// Floating-point operations performed at [`Counted`], by kind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct OpCounts {
    /// Additions and subtractions.
    pub(crate) add: u64,
    pub(crate) mul: u64,
    pub(crate) div: u64,
    /// Fused multiply-adds (`Float::mul_add`). Where the target has no hardware FMA,
    /// [`Real::mul_add_fast`](crate::helas::repr::Real::mul_add_fast) issues a product
    /// and a sum instead, which count under `mul` and `add`.
    pub(crate) fma: u64,
    pub(crate) sqrt: u64,
    /// Sign flips: an XOR or a sign-folded FMA variant in machine code, not a flop.
    pub(crate) neg: u64,
    /// Every other `Float` method that computes a value (`recip`, `powi`, `min`/`max`,
    /// transcendental functions, …), and `%`.
    pub(crate) other: u64,
}

impl OpCounts {
    /// Floating-point operations in the usual convention: an FMA is two, a division
    /// or square root one; sign flips and `other` are left out.
    pub(super) fn flops(&self) -> u64 {
        self.add + self.mul + self.div + 2 * self.fma + self.sqrt
    }

    /// Arithmetic instructions issued for [`flops`](Self::flops) (an FMA is one).
    pub(super) fn fp_instrs(&self) -> u64 {
        self.add + self.mul + self.div + self.fma + self.sqrt
    }
}

thread_local! {
    static COUNTS: Cell<OpCounts> = const {
        Cell::new(OpCounts { add: 0, mul: 0, div: 0, fma: 0, sqrt: 0, neg: 0, other: 0 })
    };
}

#[inline]
fn tally(f: impl FnOnce(&mut OpCounts)) {
    COUNTS.with(|c| {
        let mut v = c.get();
        f(&mut v);
        c.set(v);
    });
}

/// Zero this thread's counters.
pub(super) fn reset() {
    COUNTS.with(|c| c.set(OpCounts::default()));
}

/// This thread's counters since the last [`reset`].
pub(super) fn snapshot() -> OpCounts {
    COUNTS.with(|c| c.get())
}

/// An `f64` that counts the floating-point operations performed on it.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub(super) struct Counted(pub(crate) f64);

macro_rules! counted_binop {
    ($($op:ident::$f:ident => $field:ident),*) => {$(
        impl $op for Counted {
            type Output = Self;
            #[inline]
            fn $f(self, rhs: Self) -> Self {
                tally(|c| c.$field += 1);
                Counted($op::$f(self.0, rhs.0))
            }
        }
    )*};
}

counted_binop!(Add::add => add, Sub::sub => add, Mul::mul => mul, Div::div => div, Rem::rem => other);

impl Neg for Counted {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        tally(|c| c.neg += 1);
        Counted(-self.0)
    }
}

impl Zero for Counted {
    fn zero() -> Self {
        Counted(0.0)
    }
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl One for Counted {
    fn one() -> Self {
        Counted(1.0)
    }
}

impl Num for Counted {
    type FromStrRadixErr = <f64 as Num>::FromStrRadixErr;
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        f64::from_str_radix(s, radix).map(Counted)
    }
}

macro_rules! counted_to_primitive {
    ($($to:ident => $prim:ty),*) => {
        impl ToPrimitive for Counted {
            $(
                fn $to(&self) -> Option<$prim> {
                    self.0.$to()
                }
            )*
        }
    };
}

counted_to_primitive!(
    to_i8 => i8, to_i16 => i16, to_i32 => i32, to_i64 => i64, to_i128 => i128,
    to_isize => isize, to_u8 => u8, to_u16 => u16, to_u32 => u32, to_u64 => u64,
    to_u128 => u128, to_usize => usize, to_f32 => f32, to_f64 => f64
);

impl NumCast for Counted {
    fn from<P: ToPrimitive>(n: P) -> Option<Self> {
        <f64 as NumCast>::from(n).map(Counted)
    }
}

impl FromPrimitive for Counted {
    fn from_i64(n: i64) -> Option<Self> {
        f64::from_i64(n).map(Counted)
    }
    fn from_u64(n: u64) -> Option<Self> {
        f64::from_u64(n).map(Counted)
    }
    fn from_f64(n: f64) -> Option<Self> {
        Some(Counted(n))
    }
}

macro_rules! counted_unary {
    ($($f:ident => $field:ident),*) => {$(
        fn $f(self) -> Self {
            tally(|c| c.$field += 1);
            Counted(f64::$f(self.0))
        }
    )*};
}

macro_rules! counted_binary {
    ($($f:ident),*) => {$(
        fn $f(self, other: Self) -> Self {
            tally(|c| c.other += 1);
            Counted(f64::$f(self.0, other.0))
        }
    )*};
}

macro_rules! uncounted {
    ($($f:ident),*) => {$(
        fn $f(self) -> Self {
            Counted(f64::$f(self.0))
        }
    )*};
}

macro_rules! counted_const {
    ($($f:ident => $v:expr),*) => {$(
        fn $f() -> Self {
            Counted($v)
        }
    )*};
}

macro_rules! counted_predicate {
    ($($f:ident),*) => {$(
        fn $f(self) -> bool {
            self.0.$f()
        }
    )*};
}

impl Float for Counted {
    counted_const!(
        nan => f64::NAN,
        infinity => f64::INFINITY,
        neg_infinity => f64::NEG_INFINITY,
        neg_zero => -0.0,
        min_value => f64::MIN,
        min_positive_value => f64::MIN_POSITIVE,
        max_value => f64::MAX,
        epsilon => f64::EPSILON
    );
    counted_predicate!(
        is_nan,
        is_infinite,
        is_finite,
        is_normal,
        is_sign_positive,
        is_sign_negative
    );

    fn classify(self) -> FpCategory {
        self.0.classify()
    }
    fn integer_decode(self) -> (u64, i16, i8) {
        Float::integer_decode(self.0)
    }

    #[allow(clippy::disallowed_methods)] // counts the fused operation it forwards to
    fn mul_add(self, a: Self, b: Self) -> Self {
        tally(|c| c.fma += 1);
        Counted(self.0.mul_add(a.0, b.0))
    }

    counted_unary!(sqrt => sqrt);
    counted_unary!(
        recip => other, exp => other, exp2 => other, ln => other,
        log2 => other, log10 => other, cbrt => other, sin => other, cos => other,
        tan => other, asin => other, acos => other, atan => other, exp_m1 => other,
        ln_1p => other, sinh => other, cosh => other, tanh => other, asinh => other,
        acosh => other, atanh => other, to_degrees => other, to_radians => other
    );
    // Sign-bit and rounding operations: masks and integer-unit work, not arithmetic.
    uncounted!(abs, floor, ceil, round, trunc, fract, signum);
    counted_binary!(powf, log, max, min, hypot, atan2);

    fn abs_sub(self, other: Self) -> Self {
        tally(|c| c.other += 1);
        Counted(Float::abs_sub(self.0, other.0))
    }

    fn powi(self, n: i32) -> Self {
        tally(|c| c.other += 1);
        Counted(self.0.powi(n))
    }
    fn sin_cos(self) -> (Self, Self) {
        tally(|c| c.other += 2);
        (Counted(self.0.sin()), Counted(self.0.cos()))
    }
}

impl FloatConst for Counted {
    counted_const!(
        E => std::f64::consts::E,
        FRAC_1_PI => std::f64::consts::FRAC_1_PI,
        FRAC_1_SQRT_2 => std::f64::consts::FRAC_1_SQRT_2,
        FRAC_2_PI => std::f64::consts::FRAC_2_PI,
        FRAC_2_SQRT_PI => std::f64::consts::FRAC_2_SQRT_PI,
        FRAC_PI_2 => std::f64::consts::FRAC_PI_2,
        FRAC_PI_3 => std::f64::consts::FRAC_PI_3,
        FRAC_PI_4 => std::f64::consts::FRAC_PI_4,
        FRAC_PI_6 => std::f64::consts::FRAC_PI_6,
        FRAC_PI_8 => std::f64::consts::FRAC_PI_8,
        LN_10 => std::f64::consts::LN_10,
        LN_2 => std::f64::consts::LN_2,
        LOG10_E => std::f64::consts::LOG10_E,
        LOG2_E => std::f64::consts::LOG2_E,
        PI => std::f64::consts::PI,
        SQRT_2 => std::f64::consts::SQRT_2,
        TAU => std::f64::consts::TAU,
        LOG10_2 => std::f64::consts::LOG10_2,
        LOG2_10 => std::f64::consts::LOG2_10
    );
}

/// Bytes one evaluation moves between the arenas/pools and the kernels, at `F = f64`.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Traffic {
    /// Instructions that compute a value (the variadic read-out roots excluded).
    pub(crate) instrs: u64,
    /// Result-arena operand reads.
    pub(crate) arena_read: u64,
    /// Constant-pool and momentum-pool reads.
    pub(crate) pool_read: u64,
    /// Result-arena writes, one element per instruction.
    pub(crate) write: u64,
}

impl Traffic {
    pub(super) fn bytes(&self) -> u64 {
        self.arena_read + self.pool_read + self.write
    }
}

/// Per-pass traffic of `folded`'s compiled program. Order-independent: every node
/// executes once whatever the order, reading the same operands.
pub(super) fn traffic(folded: &super::fold::Folded) -> Traffic {
    use super::layout::{arena_elem_bytes, arena_index, arena_reads, Instr, Program};
    use super::op::NodeId;
    use super::tree::Tree;
    use crate::helas::repr::lorentz::LorentzVector;
    use crate::helas::repr::C;

    let (ast, an) = (&folded.ast, folded.analysis());
    // Interning order makes `instrs[id]` node `id`'s instruction.
    let order: Vec<NodeId> = (0..ast.len() as NodeId).collect();
    let prog = Program::build_ordered(ast, an, &order);
    let elem = arena_elem_bytes();
    let class_bytes = |id: NodeId| {
        an.out_type(id)
            .storage()
            .map_or(0, |s| elem[arena_index(s)] as u64)
    };
    let mom = std::mem::size_of::<LorentzVector<f64>>() as u64;
    let mut t = Traffic::default();
    for id in 0..ast.len() as NodeId {
        let instr = &prog.instrs[id as usize];
        if matches!(instr, Instr::Flows | Instr::Hels | Instr::Configs) {
            continue;
        }
        t.instrs += 1;
        t.write += class_bytes(id);
        t.arena_read += arena_reads(ast.value(id).op, ast.children_ids(id))
            .iter()
            .map(|&k| class_bytes(k))
            .sum::<u64>();
        t.pool_read += match *instr {
            Instr::ComplexConst { .. } => std::mem::size_of::<C<f64>>() as u64,
            Instr::RealConst { .. } => std::mem::size_of::<f64>() as u64,
            Instr::ExternalScalar { .. }
            | Instr::ExternalVector { .. }
            | Instr::ExternalFin { .. }
            | Instr::ExternalFout { .. }
            | Instr::PropagateScalar { .. }
            | Instr::PropagateVector { .. }
            | Instr::PropagateFin { .. }
            | Instr::PropagateFout { .. }
            | Instr::PMom { .. } => mom,
            Instr::PMomOut { len, .. } => mom * len as u64,
            _ => 0,
        };
    }
    t
}

#[cfg(test)]
mod tests {
    use super::super::compile::AmplitudeEvaluator;
    use super::super::run::BoundAmplitude;
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::helas::LorentzVector;
    use crate::phasespace::rambo_massless;
    use crate::ufo::sm::{sm_model, SMRestrict};
    use crate::ufo::EvaluatedModel;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    /// The `eval_strategies` bench's rows (`BENCH_ROWS`, with the process strings
    /// their `mg_amplitude` tables carry), so a census row joins a bench row by name.
    const BENCH_ROWS: [(&str, &str); 8] = [
        ("ee_to_mumu", "e+ e- > mu+ mu-"),
        ("ee_to_wpwm", "e+ e- > w+ w-"),
        ("uux_to_uux", "u u~ > u u~"),
        ("gg_to_gg", "g g > g g"),
        ("gg_to_ttx", "g g > t t~"),
        ("ee_to_mumua", "e+ e- > mu+ mu- a"),
        ("ee_to_mumu_tata_qcd0", "e+ e- > mu+ mu- ta+ ta- QCD=0"),
        ("uux_to_ccx_emmm_qcd0", "u u~ > c c~ e+ e- mu+ mu- QCD=0"),
    ];

    /// The bench's 16 points per row: same seed, same √s, drawn in the same order.
    fn bench_points(n_ext: usize, rng: &mut StdRng) -> Vec<Vec<LorentzVector<f64>>> {
        let sqrt_s = 500.0;
        (0..16)
            .map(|_| {
                let mut p = vec![
                    LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, sqrt_s / 2.0),
                    LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, -sqrt_s / 2.0),
                ];
                p.extend(rambo_massless(sqrt_s, n_ext - 2, rng));
                p
            })
            .collect()
    }

    fn counted(p: &[LorentzVector<f64>]) -> Vec<LorentzVector<Counted>> {
        p.iter()
            .map(|v| {
                LorentzVector::new(
                    Counted(v.e()),
                    Counted(v.px()),
                    Counted(v.py()),
                    Counted(v.pz()),
                )
            })
            .collect()
    }

    fn compiled(process: &str) -> (AmplitudeEvaluator, EvaluatedModel) {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let pc =
            parse_proc_card(&format!("generate {process}"), &ParsingOptions::default()).unwrap();
        let sets = generate_from_proc_card(&pc, &model).unwrap();
        let mut eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
        eval.prune_zero_helicities(&evaluated);
        (eval, evaluated)
    }

    /// A `Counted` evaluation is the `f64` one: same value, bit for bit, and the
    /// counters see every operation a kernel performs (a 2→2 helicity sum is
    /// thousands of them, never zero).
    #[test]
    fn counted_eval_matches_f64_and_counts() {
        let (eval, evaluated) = compiled("e+ e- > mu+ mu-");
        let amp = BoundAmplitude::<f64>::bind(&eval, &evaluated);
        let camp = BoundAmplitude::<Counted>::bind(&eval, &evaluated);
        let mut rng = StdRng::seed_from_u64(0xBE7C4);
        let pts = bench_points(eval.n_ext(), &mut rng);
        let (mut s, mut cs) = (amp.scratch_space(), camp.scratch_space());
        reset();
        for p in &pts {
            let want = amp.eval_m2(p, &mut s);
            let got = camp.eval_m2(&counted(p), &mut cs).0;
            assert_eq!(want.to_bits(), got.to_bits(), "{want:e} vs {got:e}");
        }
        let c = snapshot();
        assert!(c.flops() > 1000 * pts.len() as u64, "{c:?}");
    }

    /// The traffic census sees the stream: a real program reads operands and writes
    /// at least one real per computing instruction.
    #[test]
    fn traffic_counts_arena_reads() {
        let (eval, _) = compiled("e+ e- > mu+ mu-");
        let t = traffic(eval.folded_hel());
        assert!(t.instrs > 0 && t.arena_read > 0 && t.write > 0, "{t:?}");
        // Every computing instruction writes at least a real.
        assert!(t.write >= 8 * t.instrs, "{t:?}");
    }

    /// The roofline inputs per bench row: operations and arena traffic per event, and
    /// the arena footprint. Pair with `eval_strategies` timings on the host in question
    /// to read off achieved FLOP/s and B/s. In a build with debug assertions `eval_m2`
    /// also runs the partonic-CM input check, a dozen operations per event.
    #[test]
    #[ignore = "study instrumentation; run with --nocapture"]
    fn roofline_census() {
        println!(
            "process\tinstrs\tflops\tfp_instrs\tadd\tmul\tfma\tdiv\tsqrt\tneg\tother\t\
             arena_read_B\tpool_read_B\twrite_B\tflops/instr\tB/instr\tflops/B\tarena_KiB"
        );
        let mut rng = StdRng::seed_from_u64(0xBE7C4);
        for (name, process) in BENCH_ROWS {
            let (eval, evaluated) = compiled(process);
            let pts = bench_points(eval.n_ext(), &mut rng);
            let amp = BoundAmplitude::<f64>::bind(&eval, &evaluated);
            let camp = BoundAmplitude::<Counted>::bind(&eval, &evaluated);
            let (mut s, mut cs) = (amp.scratch_space(), camp.scratch_space());
            // The workspace's one-shot cross-check runs on its first pass; keep it out
            // of the count. Warm on a point other than the first counted one, or the
            // arena-reuse cache would skip that point's forward pass.
            camp.eval_m2(&counted(&pts[1]), &mut cs);
            reset();
            for p in &pts {
                let want = amp.eval_m2(p, &mut s);
                let got = camp.eval_m2(&counted(p), &mut cs).0;
                assert_eq!(want.to_bits(), got.to_bits(), "{name}");
            }
            let n = pts.len() as f64;
            let c = snapshot();
            let per = |x: u64| x as f64 / n;
            let t = traffic(eval.folded_hel());
            let sizes = eval.folded_hel().program().arena_sizes;
            let arena_bytes: usize = sizes
                .iter()
                .zip(super::super::layout::arena_elem_bytes())
                .map(|(&k, e)| k as usize * e)
                .sum();
            println!(
                "{name}\t{}\t{:.0}\t{:.0}\t{:.0}\t{:.0}\t{:.0}\t{:.0}\t{:.0}\t{:.0}\t{:.0}\t\
                 {}\t{}\t{}\t{:.2}\t{:.1}\t{:.3}\t{:.1}",
                t.instrs,
                per(c.flops()),
                per(c.fp_instrs()),
                per(c.add),
                per(c.mul),
                per(c.fma),
                per(c.div),
                per(c.sqrt),
                per(c.neg),
                per(c.other),
                t.arena_read,
                t.pool_read,
                t.write,
                per(c.flops()) / t.instrs as f64,
                t.bytes() as f64 / t.instrs as f64,
                per(c.flops()) / t.bytes() as f64,
                arena_bytes as f64 / 1024.0,
            );
        }
    }
}
