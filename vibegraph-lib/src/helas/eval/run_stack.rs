//! Alternative evaluation strategy: a post-order stack machine over the folded DAG.
//!
//! Where [`super::run`]'s forward scan memoizes *every* node in an arena-wide result
//! buffer, [`StackProgram`] linearizes the DAG into a post-order instruction stream
//! whose live values sit on a small stack (depth ≈ the tree depth, so it stays in
//! L1), with only genuinely shared nodes spilled to a memo pad: the first evaluation
//! of a node with more than one parent is followed by a `Store`, and every later
//! reference becomes a `Load`. Both strategies reduce nodes through the shared
//! [`run::apply`](super::run) dispatch on identical operand values in identical
//! order, so their results agree bit-for-bit.

use num_traits::FromPrimitive;

use super::compile::AmplitudeEvaluator;
use super::fold::Folded;
use super::op::{Const, Node, NodeId, Op};
use super::run::{apply, EvalEnv, ScratchSpace};
use super::tree::Tree;
use super::waveform_slot::WaveformSlot;
use crate::helas::repr::lorentz::LorentzVector;
use crate::helas::repr::{Real, C};
use crate::ufo::EvaluatedModel;

/// One instruction of the linearized post-order program.
#[derive(Clone, Copy, Debug)]
enum Instr {
    /// Reduce the top `n` stack values (in child order) with this node; push the result.
    Apply { op: Op, leaf: Const, n: u32 },
    /// Copy the stack top into the memo pad (emitted once, right after a shared
    /// node's first evaluation).
    Store(u32),
    /// Push a previously stored shared value.
    Load(u32),
}

/// A folded arena compiled to a post-order stack program (card-independent).
#[derive(Debug)]
pub struct StackProgram {
    instrs: Box<[Instr]>,
    /// Peak value-stack depth, exact by simulation — the stack never reallocates.
    max_stack: usize,
    /// Number of shared (refcount > 1) nodes = memo pad length.
    memo_len: usize,
    /// Number of `Apply` instructions (= reachable DAG nodes).
    n_nodes: usize,
}

impl StackProgram {
    /// Linearize the folded arena: iterative post-order DFS from the root, emitting
    /// each node once and `Load`ing it at every later reference.
    pub fn compile(folded: &Folded) -> StackProgram {
        let ast = &folded.ast;
        // Parent-edge counts over the subgraph reachable from the root (the arena may
        // hold unreachable nodes; they must not inflate refcounts or the program).
        let mut refs = vec![0u32; ast.len()];
        refs[ast.root() as usize] = 1;
        let mut seen = vec![false; ast.len()];
        let mut stack = vec![ast.root()];
        while let Some(n) = stack.pop() {
            if std::mem::replace(&mut seen[n as usize], true) {
                continue;
            }
            for &c in ast.children_ids(n) {
                refs[c as usize] += 1;
                stack.push(c);
            }
        }

        enum Task {
            Visit(NodeId),
            Emit(NodeId),
        }
        let mut instrs = Vec::new();
        let mut memo_slot = vec![u32::MAX; ast.len()];
        let mut memo_len = 0u32;
        let mut n_nodes = 0usize;
        let mut tasks = vec![Task::Visit(ast.root())];
        while let Some(task) = tasks.pop() {
            match task {
                Task::Visit(n) => {
                    if memo_slot[n as usize] != u32::MAX {
                        instrs.push(Instr::Load(memo_slot[n as usize]));
                        continue;
                    }
                    tasks.push(Task::Emit(n));
                    // Children pushed in reverse so the leftmost is visited — and its
                    // value lands on the stack — first, preserving operand order.
                    // A shared node's first Visit fully emits it (everything that
                    // Visit pushes runs before any sibling task below), so later
                    // Visits always find its memo slot assigned.
                    for &c in ast.children_ids(n).iter().rev() {
                        tasks.push(Task::Visit(c));
                    }
                }
                Task::Emit(n) => {
                    let node = ast.value(n);
                    instrs.push(Instr::Apply {
                        op: node.op,
                        leaf: node.leaf,
                        n: ast.children_ids(n).len() as u32,
                    });
                    n_nodes += 1;
                    if refs[n as usize] > 1 {
                        memo_slot[n as usize] = memo_len;
                        instrs.push(Instr::Store(memo_len));
                        memo_len += 1;
                    }
                }
            }
        }

        let mut depth = 0usize;
        let mut max_stack = 0usize;
        for instr in &instrs {
            match *instr {
                Instr::Apply { n, .. } => depth = depth - n as usize + 1,
                Instr::Load(_) => depth += 1,
                Instr::Store(_) => {}
            }
            max_stack = max_stack.max(depth);
        }
        debug_assert_eq!(depth, 1, "program must leave exactly the root value");

        StackProgram {
            instrs: instrs.into_boxed_slice(),
            max_stack,
            memo_len: memo_len as usize,
            n_nodes,
        }
    }

    /// (reachable nodes, shared nodes = memo pad length, peak stack depth).
    pub fn stats(&self) -> (usize, usize, usize) {
        (self.n_nodes, self.memo_len, self.max_stack)
    }

    /// Execute the program for one (momenta, helicity) point.
    fn eval_slot<F: Real>(
        &self,
        env: &EvalEnv<'_, F>,
        stack: &mut Vec<WaveformSlot<F>>,
        memo: &mut Vec<WaveformSlot<F>>,
    ) -> WaveformSlot<F> {
        stack.clear();
        stack.reserve(self.max_stack);
        // No clear between evaluations: every `Load` slot was `Store`d earlier in
        // this same program run, so stale values are never observable.
        if memo.len() < self.memo_len {
            memo.resize(self.memo_len, WaveformSlot::Empty);
        }
        for instr in &self.instrs {
            match *instr {
                Instr::Apply { op, leaf, n } => {
                    let n = n as usize;
                    let base = stack.len() - n;
                    let value = {
                        let kids = &stack[base..];
                        apply(&Node::new(op, leaf), n, |i| kids[i], env)
                    };
                    stack.truncate(base);
                    stack.push(value);
                }
                Instr::Store(s) => memo[s as usize] = *stack.last().expect("Store on empty stack"),
                Instr::Load(s) => stack.push(memo[s as usize]),
            }
        }
        debug_assert_eq!(stack.len(), 1, "program must leave exactly the root value");
        stack.pop().unwrap()
    }
}

/// A compiled amplitude bound to a parameter card, evaluated with the stack strategy.
///
/// Mirrors [`super::run::BoundAmplitude`]'s bind/eval shape; results are bit-for-bit
/// identical to the forward scan (same kernels, same operand order).
#[derive(Debug)]
pub struct BoundAmplitudeStack<'a, F: Real> {
    eval: &'a AmplitudeEvaluator,
    program: StackProgram,
    consts_c: Box<[C<F>]>,
    consts_f: Box<[F]>,
}

impl<'a, F: Real + FromPrimitive> BoundAmplitudeStack<'a, F> {
    /// Resolve a parameter card against a compiled evaluator and linearize its folded
    /// arena into the stack program.
    pub fn bind(eval: &'a AmplitudeEvaluator, evaluated: &EvaluatedModel) -> Self {
        let (consts_c, consts_f) = eval.folded().pools::<F>(evaluated);
        let program = StackProgram::compile(eval.folded());
        BoundAmplitudeStack {
            eval,
            program,
            consts_c,
            consts_f,
        }
    }

    /// The compiled stack program (for size/sharing statistics).
    pub fn program(&self) -> &StackProgram {
        &self.program
    }

    /// A workspace sized for this amplitude. Create once and pass to every
    /// `eval_*` call; reuse across points to keep the hot path allocation-free.
    pub fn scratch_space(&self) -> ScratchSpace<F> {
        ScratchSpace {
            res: Vec::new(),
            stack: Vec::with_capacity(self.program.max_stack),
            memo: vec![WaveformSlot::Empty; self.program.memo_len],
        }
    }

    /// Evaluate |M|² summed over all helicities (see
    /// [`BoundAmplitude::eval_m2`](super::run::BoundAmplitude::eval_m2)).
    pub fn eval_m2(&self, momenta: &[LorentzVector<F>], scratch: &mut ScratchSpace<F>) -> F {
        if momenta.len() != self.eval.n_ext() {
            return F::zero();
        }
        self.eval
            .helicities()
            .iter()
            .map(|hel| self.run(momenta, hel, scratch).norm_sqr())
            .fold(F::zero(), |acc, x| acc + x)
    }

    /// Evaluate the complex amplitude M for a single helicity configuration.
    pub fn eval_amplitude(
        &self,
        momenta: &[LorentzVector<F>],
        helicities: &[i32],
        scratch: &mut ScratchSpace<F>,
    ) -> C<F> {
        if momenta.len() != self.eval.n_ext() || helicities.len() != self.eval.n_ext() {
            return C::new(F::zero(), F::zero());
        }
        self.run(momenta, helicities, scratch)
    }

    fn run(
        &self,
        momenta: &[LorentzVector<F>],
        helicities: &[i32],
        scratch: &mut ScratchSpace<F>,
    ) -> C<F> {
        let env = EvalEnv {
            consts_c: &self.consts_c,
            consts_f: &self.consts_f,
            ext_legs: self.eval.folded().ext_legs(),
            momenta,
            helicities,
            ward_leg: None,
        };
        match self
            .program
            .eval_slot(&env, &mut scratch.stack, &mut scratch.memo)
        {
            WaveformSlot::Scalar(s) => s.value,
            WaveformSlot::Empty => C::new(F::zero(), F::zero()),
            other => panic!("amplitude root is not a scalar: {other:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    use super::super::compile::MG_VALIDATED_PROCESSES;
    use super::super::run::BoundAmplitude;
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::phasespace::rambo_massless;
    use crate::ufo::sm::{sm_model, SMRestrict};

    fn assert_bit_eq(a: f64, b: f64, ctx: &dyn Fn() -> String) {
        assert_eq!(
            a.to_bits(),
            b.to_bits(),
            "stack vs forward mismatch ({}): {a:?} vs {b:?}",
            ctx()
        );
    }

    /// The stack program reproduces the forward scan **bit-for-bit** on every process
    /// of the MG-validated suite: same kernels applied to the same operands in the
    /// same order, only the storage strategy differs. RAMBO kinematics are massless
    /// (unphysical for the massive-external processes) — irrelevant here, since both
    /// strategies are pure functions of the same inputs.
    #[test]
    fn stack_matches_forward_bit_for_bit() {
        let model = sm_model(SMRestrict::Default);
        let evaluated = crate::ufo::EvaluatedModel::from_model(model.clone());
        let opts = ParsingOptions::default();
        let mut rng = StdRng::seed_from_u64(0x0F0F_5EED);
        let sqrt_s = 500.0;
        for process in MG_VALIDATED_PROCESSES {
            let pc = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
            let sets = generate_from_proc_card(&pc, &model).unwrap();
            assert!(!sets.is_empty(), "no diagrams for '{process}'");
            for set in &sets {
                let eval = AmplitudeEvaluator::compile(set, &model).unwrap();
                let fwd = BoundAmplitude::<f64>::bind(&eval, &evaluated);
                let stk = BoundAmplitudeStack::<f64>::bind(&eval, &evaluated);
                let (nodes, shared, depth) = stk.program().stats();
                println!(
                    "[{process}] nodes={nodes} shared={shared} ({:.1}%) peak stack={depth}",
                    100.0 * shared as f64 / nodes as f64
                );
                let mut s_fwd = fwd.scratch_space();
                let mut s_stk = stk.scratch_space();
                let point = |rng: &mut StdRng| {
                    let mut p = vec![
                        LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, sqrt_s / 2.0),
                        LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, -sqrt_s / 2.0),
                    ];
                    p.extend(rambo_massless(sqrt_s, eval.n_ext() - 2, rng));
                    p
                };
                // Point 1: every helicity amplitude individually.
                let p = point(&mut rng);
                for hel in eval.helicities() {
                    let a = fwd.eval_amplitude(&p, hel, &mut s_fwd);
                    let b = stk.eval_amplitude(&p, hel, &mut s_stk);
                    assert_bit_eq(a.re, b.re, &|| format!("{process}, hel {hel:?}, re"));
                    assert_bit_eq(a.im, b.im, &|| format!("{process}, hel {hel:?}, im"));
                }
                // Point 2: the full helicity-summed |M|².
                let p = point(&mut rng);
                let a = fwd.eval_m2(&p, &mut s_fwd);
                let b = stk.eval_m2(&p, &mut s_stk);
                assert_bit_eq(a, b, &|| format!("{process}, |M|²"));
            }
        }
    }
}
