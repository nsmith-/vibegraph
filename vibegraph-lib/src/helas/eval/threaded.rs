//! Threaded dispatch of the typed instruction stream through guaranteed tail calls.
//!
//! The `match` loop in [`super::run`] funnels every instruction through one indirect
//! jump, so a single branch-target-buffer entry has to predict the whole program's
//! opcode sequence. Here every opcode has a handler of its own: it executes its
//! instruction and ends in `become`, a tail call to the next instruction's handler. The
//! jump that dispatches instruction `k + 1` therefore sits at the end of instruction
//! `k`'s handler, where its target history is that opcode's successors alone.
//!
//! `become` is what makes this a loop rather than a recursion: without a guaranteed tail
//! call each handler would push a frame, and a program of tens of thousands of
//! instructions would overflow the stack. It needs the nightly `explicit_tail_calls`
//! feature, which is why this module exists only under the `threaded-dispatch` cargo
//! feature.
//!
//! Handlers are one generic function instantiated per opcode (`handler::<F, OP>`). Each
//! calls the same inlined [`step`] the `match` loop does, behind a check that the
//! instruction's opcode is `OP`; inside that branch the compiler knows the variant, and
//! `step`'s `match` folds to its one arm. The instruction bodies are therefore
//! textually shared between the two dispatchers, and any difference between them is the
//! dispatch.

use super::layout::{Instr, InstrOpcode, Program};
use super::run::{step, Arenas, EvalEnv};
use crate::helas::repr::Real;

/// The interpreter state every handler receives: the arenas and the program's
/// per-instruction streams. Handlers share one signature, as `become` requires.
struct Vm<'v, 's, 'e, F: Real> {
    arenas: &'v mut Arenas<'s, F>,
    env: &'v EvalEnv<'e, F>,
    instrs: &'v [Instr],
    dest: &'v [u32],
    opcodes: &'v [u8],
}

type Handler<F> = for<'x, 'v, 's, 'e> fn(&'x mut Vm<'v, 's, 'e, F>, usize);

/// Run `prog`'s instruction stream over `arenas`.
pub(super) fn run<F: Real>(prog: &Program, env: &EvalEnv<'_, F>, arenas: &mut Arenas<'_, F>) {
    let mut vm = Vm {
        arenas,
        env,
        instrs: &prog.instrs,
        dest: &prog.dest,
        opcodes: &prog.opcodes,
    };
    let first = vm.opcodes[0];
    (Table::<F>::HANDLERS[first as usize])(&mut vm, 0);
}

/// Execute instruction `pc`, whose opcode is `OP`, then tail-call the next one's handler.
fn handler<F: Real, const OP: u8>(vm: &mut Vm<'_, '_, '_, F>, pc: usize) {
    let instr = vm.instrs[pc];
    if InstrOpcode::from(&instr) as u8 == OP {
        step(instr, vm.dest[pc] as usize, vm.arenas, vm.env);
    } else {
        unreachable!("opcode stream disagrees with instruction {pc}");
    }
    let next = pc + 1;
    let op = vm.opcodes[next];
    become (Table::<F>::HANDLERS[op as usize])(vm, next)
}

/// The sentinel after the last instruction: returns out of the handler chain.
fn halt<F: Real>(_: &mut Vm<'_, '_, '_, F>, _: usize) {}

fn bad_opcode<F: Real>(_: &mut Vm<'_, '_, '_, F>, pc: usize) {
    unreachable!("no handler for the opcode at {pc}");
}

struct Table<F>(core::marker::PhantomData<F>);

macro_rules! handler_table {
    ($($op:literal)*) => {
        impl<F: Real> Table<F> {
            /// One entry per `u8`, so an opcode byte indexes it without a bounds check.
            const HANDLERS: &'static [Handler<F>; 256] = &{
                let mut t: [Handler<F>; 256] = [bad_opcode::<F>; 256];
                $(t[$op] = handler::<F, $op>;)*
                t[HALT as usize] = halt::<F>;
                t
            };
        }
        const N_HANDLERS: usize = [$($op),*].len();
    };
}

handler_table!(
    0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26
    27 28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52
);

const _: () = assert!(
    N_HANDLERS == <Instr as strum::EnumCount>::COUNT,
    "one handler per instruction variant"
);

/// The opcode of the sentinel ending every opcode stream.
pub(super) const HALT: u8 = u8::MAX;

/// The opcode stream of `instrs`, terminated by [`HALT`].
pub(super) fn opcodes(instrs: &[Instr]) -> Box<[u8]> {
    instrs
        .iter()
        .map(|i| InstrOpcode::from(i) as u8)
        .chain([HALT])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::compile::{AmplitudeEvaluator, MG_VALIDATED_PROCESSES};
    use super::super::lane_field::LaneField;
    use super::super::run::{BoundAmplitude, MATCH_DISPATCH};
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::helas::repr::Real;
    use crate::helas::LorentzVector;
    use crate::phasespace::rambo_massless;
    use crate::ufo::sm::{sm_model, SMRestrict};
    use crate::ufo::EvaluatedModel;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    /// Per-configuration `AMP2`, per-flow `JAMP2` and per-helicity |M|² — the
    /// finest read-outs of both arenas (base and helicity-expanded) — through one
    /// dispatcher.
    fn read_outs<F: Real>(
        amp: &BoundAmplitude<'_, F>,
        eval: &AmplitudeEvaluator,
        p: &[LorentzVector<F>],
        use_match: bool,
    ) -> Vec<F> {
        MATCH_DISPATCH.set(use_match);
        let mut amp2 = vec![F::zero(); eval.n_configs()];
        amp.eval_amp2(p, &mut amp.scratch_space(), &mut amp2);
        let mut jamp2 = vec![F::zero(); eval.n_flows()];
        amp.eval_jamp2(p, &mut amp.scratch_space(), &mut jamp2);
        let mut hel = vec![F::zero(); eval.helicities().len()];
        amp.eval_hel_m2(p, &mut amp.scratch_space(), &mut hel);
        MATCH_DISPATCH.set(false);
        amp2.into_iter().chain(jamp2).chain(hel).collect()
    }

    /// The two dispatchers run the same inlined instruction bodies, so they must
    /// agree to the bit on every read-out, at scalar precision and over a lane pack.
    #[test]
    fn threaded_dispatch_matches_match_loop_bit_for_bit() {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let opts = ParsingOptions::default();
        let mut rng = StdRng::seed_from_u64(0x7A11_C0);
        let sqrt_s = 500.0;
        for process in MG_VALIDATED_PROCESSES {
            let pc = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
            for set in &generate_from_proc_card(&pc, &model).unwrap() {
                let mut eval = AmplitudeEvaluator::compile(set, &model).unwrap();
                eval.prune_zero_helicities(&evaluated);
                let amp = BoundAmplitude::<f64>::bind(&eval, &evaluated);
                let lanes = amp.broadcast_lanes::<4>();
                let pts: Vec<Vec<LorentzVector<f64>>> = (0..4)
                    .map(|_| {
                        let mut p = vec![
                            LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, sqrt_s / 2.0),
                            LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, -sqrt_s / 2.0),
                        ];
                        p.extend(rambo_massless(sqrt_s, eval.n_ext() - 2, &mut rng));
                        p
                    })
                    .collect();
                for p in &pts {
                    let threaded = read_outs(&amp, &eval, p, false);
                    let looped = read_outs(&amp, &eval, p, true);
                    assert!(
                        threaded.iter().any(|v| *v != 0.0),
                        "{process}: no live read-out"
                    );
                    let t: Vec<u64> = threaded.iter().map(|v| v.to_bits()).collect();
                    let l: Vec<u64> = looped.iter().map(|v| v.to_bits()).collect();
                    assert_eq!(t, l, "{process}: scalar read-outs differ");
                }
                let refs: [&[LorentzVector<f64>]; 4] = std::array::from_fn(|k| pts[k].as_slice());
                let packed = super::super::run::pack_lane_points::<4>(&refs);
                let threaded = read_outs(&lanes, &eval, &packed, false);
                let looped = read_outs(&lanes, &eval, &packed, true);
                let bits = |v: &[LaneField<4>]| -> Vec<u64> {
                    v.iter()
                        .flat_map(|x| x.to_array().map(f64::to_bits))
                        .collect()
                };
                assert_eq!(
                    bits(&threaded),
                    bits(&looped),
                    "{process}: lane read-outs differ"
                );
            }
        }
    }
}
