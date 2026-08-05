//! Alternative execution orders for a compiled [`Program`], and the structural metrics
//! that judge them.
//!
//! [`Program::build`] emits one instruction per node, grouped by
//! [`Instr`](super::layout::Instr) variant inside each ASAP dependency level. Any other
//! topological order of the same DAG
//! computes exactly the same values with exactly the same arithmetic; only slot
//! recycling, operand reuse distance and the discriminant sequence the dispatch sees
//! change. This module builds such orders and measures the four properties an execution
//! order can plausibly trade between:
//!
//! - **producer→consumer distance** — how far an operand's definition sits from its
//!   use, in instructions (locality of the arena reads);
//! - **live-set width** — simultaneously live slots and bytes over the stream, whose
//!   peak is `Program::arena_sizes` (the working set the arenas must hold);
//! - **discriminant run length** — how long the stream stays on one [`Instr`] variant,
//!   which is what the forward pass's single indirect dispatch predicts on;
//! - **critical-path depth vs stream length** — the instruction-level parallelism the
//!   order leaves available.
//!
//! Selecting anything but [`PRODUCTION`] is a study hook: the alternatives exist only
//! under `cfg(test)` or the `eval-schedule-study` feature, and a release build carries
//! the production order alone (which lives with the lowering, in
//! [`super::layout::op_blocked_order`]).

use std::cell::Cell;

use super::analysis::NodeAnalysis;
use super::ast::Ast;
use super::layout::{
    arena_elem_bytes, arena_index, arena_reads, asap_levels, instr_kinds, liveness, Liveness,
    Program, N_ARENAS,
};
use super::op::{Const, NodeId};
use super::tree::Tree;

/// Which topological order [`Program::build`] emits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Schedule {
    /// Arena (storage) order: node id order, the order the folded arena interns nodes in.
    Arena,
    /// Post-order depth-first from the root, last operand emitted immediately before
    /// its consumer — minimizes producer→consumer distance on the chain being followed.
    DepthFirst,
    /// Greedy list scheduling that at each step takes the ready node freeing the most
    /// arena bytes net of what it defines (Sethi–Ullman-flavored), so the live set
    /// stays as narrow as the DAG allows.
    MinLive,
    /// Instructions grouped by [`super::layout::Instr`] variant inside each dependency
    /// level, so the dispatch site sees long runs of one discriminant. The compiled
    /// default ([`PRODUCTION`]), where it additionally carries the arena-bytes fallback.
    OpBlocked,
    /// Op-blocking confined to a sliding window of `w` instructions inside each
    /// dependency level: long enough runs to amortize the dispatch, short enough that
    /// operands stay near their consumers and slots keep being recycled.
    OpWindow(u32),
}

impl Schedule {
    pub(super) fn from_name(name: &str) -> Option<Schedule> {
        match name {
            "arena" => Some(Schedule::Arena),
            "dfs" | "depth-first" => Some(Schedule::DepthFirst),
            "minlive" | "min-live" => Some(Schedule::MinLive),
            "opblocked" | "op-blocked" => Some(Schedule::OpBlocked),
            _ => name
                .strip_prefix("opwin")
                .and_then(|w| w.parse().ok())
                .filter(|&w| w > 0)
                .map(Schedule::OpWindow),
        }
    }

    pub(super) fn name(self) -> String {
        match self {
            Schedule::Arena => "arena".to_string(),
            Schedule::DepthFirst => "dfs".to_string(),
            Schedule::MinLive => "minlive".to_string(),
            Schedule::OpBlocked => "opblocked".to_string(),
            Schedule::OpWindow(w) => format!("opwin{w}"),
        }
    }

    /// Every order the study measures, default first.
    pub(super) const ALL: [Schedule; 7] = [
        Schedule::Arena,
        Schedule::DepthFirst,
        Schedule::MinLive,
        Schedule::OpBlocked,
        Schedule::OpWindow(32),
        Schedule::OpWindow(128),
        Schedule::OpWindow(512),
    ];
}

/// The order a build with no study hook active emits.
pub(super) const PRODUCTION: Schedule = Schedule::OpBlocked;

thread_local! {
    /// The order the next [`Program::build`] on this thread emits. Initialised from
    /// `VIBEGRAPH_EVAL_SCHEDULE` so a prebuilt bench binary can be swept over the
    /// orders without a rebuild — the machine code is identical across the sweep, only
    /// the compiled program order differs.
    static ACTIVE: Cell<Schedule> = Cell::new(
        std::env::var("VIBEGRAPH_EVAL_SCHEDULE")
            .ok()
            .and_then(|v| Schedule::from_name(v.trim()))
            .unwrap_or(PRODUCTION),
    );
}

pub(super) fn active() -> Schedule {
    ACTIVE.with(|c| c.get())
}

/// The order to build with when the study hook selects away from [`PRODUCTION`], or
/// `None` to let [`Program::build`] take its own path.
pub(super) fn override_order(ast: &Ast<Const>, an: &NodeAnalysis) -> Option<Vec<NodeId>> {
    let sched = active();
    (sched != PRODUCTION).then(|| build_order(ast, an, sched))
}

/// Select the order for subsequent program builds on this thread; returns the previous
/// one, so a caller can restore it.
#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn set_active(s: Schedule) -> Schedule {
    ACTIVE.with(|c| c.replace(s))
}

/// A topological order of every node of `ast` (children before parents), for `sched`.
pub(super) fn build_order(ast: &Ast<Const>, an: &NodeAnalysis, sched: Schedule) -> Vec<NodeId> {
    let order = match sched {
        Schedule::Arena => (0..ast.len() as NodeId).collect(),
        Schedule::DepthFirst => depth_first(ast),
        Schedule::MinLive => min_live(ast, an),
        Schedule::OpBlocked => super::layout::op_blocked_order(ast, an),
        Schedule::OpWindow(w) => op_windowed(ast, an, w),
    };
    debug_assert!(is_topological(ast, &order));
    order
}

/// Whether `order` lists every node exactly once with each node's children before it.
pub(super) fn is_topological(ast: &Ast<Const>, order: &[NodeId]) -> bool {
    let n = ast.len();
    if order.len() != n {
        return false;
    }
    let mut pos = vec![u32::MAX; n];
    for (p, &id) in order.iter().enumerate() {
        if pos[id as usize] != u32::MAX {
            return false;
        }
        pos[id as usize] = p as u32;
    }
    order.iter().enumerate().all(|(p, &id)| {
        ast.children_ids(id)
            .iter()
            .all(|&c| (pos[c as usize] as usize) < p)
    })
}

/// Post-order DFS from the root, then from any node the root does not reach (an arena
/// may carry nodes no root reads). Children are walked left to right, so the last
/// operand of a node is emitted immediately before it whenever that operand's subtree
/// is still unemitted — the chain-following property.
fn depth_first(ast: &Ast<Const>) -> Vec<NodeId> {
    let n = ast.len();
    let mut seen = vec![false; n];
    let mut order: Vec<NodeId> = Vec::with_capacity(n);
    let mut stack: Vec<(NodeId, u32)> = Vec::new();
    let mut start_with = |root: NodeId, order: &mut Vec<NodeId>, seen: &mut Vec<bool>| {
        if seen[root as usize] {
            return;
        }
        seen[root as usize] = true;
        stack.push((root, 0));
        while let Some(&mut (id, ref mut next)) = stack.last_mut() {
            let kids = ast.children_ids(id);
            if (*next as usize) < kids.len() {
                let c = kids[*next as usize];
                *next += 1;
                if !seen[c as usize] {
                    seen[c as usize] = true;
                    stack.push((c, 0));
                }
            } else {
                order.push(id);
                stack.pop();
            }
        }
    };
    start_with(ast.root(), &mut order, &mut seen);
    for id in 0..n as NodeId {
        start_with(id, &mut order, &mut seen);
    }
    order
}

/// Greedy list scheduling on live width: among the nodes whose children are all
/// scheduled, take the one whose execution frees the most arena bytes net of the bytes
/// its own result occupies. Ties break toward the most recently defined operand, which
/// keeps the greedy walk on a chain instead of scattering across the DAG.
fn min_live(ast: &Ast<Const>, an: &NodeAnalysis) -> Vec<NodeId> {
    let n = ast.len();
    let bytes = arena_elem_bytes();
    let node_bytes = |id: NodeId| -> i64 {
        an.out_type(id)
            .storage()
            .map_or(0, |s| bytes[arena_index(s)] as i64)
    };

    // Parents by child, so scheduling a node can wake its consumers in O(deg).
    let mut parent_off = vec![0u32; n + 1];
    for id in 0..n as NodeId {
        for &c in ast.children_ids(id) {
            parent_off[c as usize + 1] += 1;
        }
    }
    for i in 0..n {
        parent_off[i + 1] += parent_off[i];
    }
    let mut parents = vec![0 as NodeId; parent_off[n] as usize];
    let mut cursor = parent_off.clone();
    for id in 0..n as NodeId {
        for &c in ast.children_ids(id) {
            parents[cursor[c as usize] as usize] = id;
            cursor[c as usize] += 1;
        }
    }

    let mut pending: Vec<u32> = (0..n)
        .map(|i| ast.children_ids(i as NodeId).len() as u32)
        .collect();
    // Unscheduled consumers that will actually read the value out of its arena.
    let mut readers = vec![0u32; n];
    for id in 0..n as NodeId {
        for &c in arena_reads(ast.value(id).op, ast.children_ids(id)) {
            readers[c as usize] += 1;
        }
    }
    // Values the evaluator reads after the pass never free, so they are worth nothing
    // to the greedy score.
    let live_end = liveness(ast, &(0..n as NodeId).collect::<Vec<_>>()).live_end;

    // Bytes this node's execution frees: every arena-read operand whose last reader it
    // is, counted once per distinct operand.
    let mut seen_stamp = vec![u32::MAX; n];
    let freed_bytes = |id: NodeId, readers: &[u32], stamp: u32, seen: &mut [u32]| -> i64 {
        let mut freed = 0i64;
        for &c in arena_reads(ast.value(id).op, ast.children_ids(id)) {
            if seen[c as usize] == stamp {
                continue;
            }
            seen[c as usize] = stamp;
            if !live_end[c as usize] && readers[c as usize] == 1 {
                freed += node_bytes(c);
            }
        }
        freed
    };

    // A lazy max-heap over (score, recency, id): entries go stale as neighbours are
    // scheduled, so the popped entry is re-scored and re-pushed when it no longer
    // matches.
    let mut heap: std::collections::BinaryHeap<(i64, u32, NodeId)> =
        std::collections::BinaryHeap::new();
    let mut recency = vec![0u32; n];
    let mut tick = 0u32;
    let mut scheduled = vec![false; n];
    for id in 0..n as NodeId {
        if pending[id as usize] == 0 {
            tick += 1;
            let s = freed_bytes(id, &readers, tick, &mut seen_stamp) - node_bytes(id);
            heap.push((s, 0, id));
        }
    }

    let mut order: Vec<NodeId> = Vec::with_capacity(n);
    while let Some((s, r, id)) = heap.pop() {
        if scheduled[id as usize] {
            continue;
        }
        tick += 1;
        let now = freed_bytes(id, &readers, tick, &mut seen_stamp) - node_bytes(id);
        if now != s || r != recency[id as usize] {
            heap.push((now, recency[id as usize], id));
            continue;
        }
        scheduled[id as usize] = true;
        order.push(id);
        let step = order.len() as u32;
        tick += 1;
        for &c in arena_reads(ast.value(id).op, ast.children_ids(id)) {
            if seen_stamp[c as usize] == tick {
                continue;
            }
            seen_stamp[c as usize] = tick;
            readers[c as usize] -= 1;
        }
        for k in parent_off[id as usize]..parent_off[id as usize + 1] {
            let p = parents[k as usize];
            pending[p as usize] -= 1;
            recency[p as usize] = step;
            if pending[p as usize] == 0 {
                tick += 1;
                let sc = freed_bytes(p, &readers, tick, &mut seen_stamp) - node_bytes(p);
                heap.push((sc, step, p));
            }
        }
        // A consumer that just lost an operand's last reader is now worth more; the
        // stale entry is caught by the re-score check above, so nothing else is needed.
    }
    assert_eq!(order.len(), n, "min-live schedule dropped nodes");
    order
}

/// [`super::layout::op_blocked_order`] with the grouping confined to a sliding window:
/// each ASAP dependency level is cut into runs of `window` consecutive nodes *in arena
/// order* and only those are sorted by variant, so arena order's operand locality
/// survives at window granularity while the dispatch still sees runs.
fn op_windowed(ast: &Ast<Const>, an: &NodeAnalysis, window: u32) -> Vec<NodeId> {
    let n = ast.len();
    let level = asap_levels(ast);
    let kind = instr_kinds(ast, an);
    let mut order: Vec<NodeId> = (0..n as NodeId).collect();
    order.sort_by_key(|&id| (level[id as usize], id));
    let w = window.min(n.max(1) as u32) as usize;
    let mut start = 0usize;
    while start < n {
        // One window: the next `w` nodes, never crossing a level boundary.
        let lvl = level[order[start] as usize];
        let mut end = (start + w).min(n);
        while end > start + 1 && level[order[end - 1] as usize] != lvl {
            end -= 1;
        }
        order[start..end].sort_by_key(|&id| (kind[id as usize], id));
        start = end;
    }
    order
}

/// Structural metrics of one compiled program under one execution order.
#[derive(Clone, Debug)]
pub(super) struct ProgramMetrics {
    pub(super) n_instrs: usize,
    /// Arena reads whose distance was measured (one per operand occurrence).
    pub(super) n_edges: usize,
    /// Producer→consumer distance in instructions: mean, median, 90th percentile, max.
    pub(super) dist_mean: f64,
    pub(super) dist_p50: u32,
    pub(super) dist_p90: u32,
    pub(super) dist_max: u32,
    /// Share of arena reads whose producer is within 1 / 8 / 64 instructions.
    pub(super) dist_le1: f64,
    pub(super) dist_le8: f64,
    pub(super) dist_le64: f64,
    /// Simultaneously live slots, summed over arenas: peak and mean over the stream.
    pub(super) live_slots_peak: u32,
    pub(super) live_slots_mean: f64,
    /// The same in bytes at `F = f64` (the arenas' working set).
    pub(super) live_bytes_peak: usize,
    pub(super) live_bytes_mean: f64,
    /// Peak slots per arena — exactly `Program::arena_sizes`.
    pub(super) arena_sizes: [u32; N_ARENAS],
    /// Runs of one [`super::layout::Instr`] variant over the stream.
    pub(super) n_runs: usize,
    pub(super) mean_run: f64,
    /// The six most frequent variants: `(name, instruction count, run count)` — the
    /// discriminant-level view of what the dispatch site is predicting.
    pub(super) top_kinds: Vec<(&'static str, usize, usize)>,
    /// Longest chain of arena reads, in instructions (order-independent), and the
    /// stream length over it — the instruction-level parallelism on offer.
    pub(super) depth: u32,
    pub(super) ilp: f64,
}

/// Measure one program's execution order. `order` must be the order `prog` was built
/// with (`Program::build_ordered`'s argument).
pub(super) fn measure(
    ast: &Ast<Const>,
    an: &NodeAnalysis,
    prog: &Program,
    order: &[NodeId],
) -> ProgramMetrics {
    let n = ast.len();
    let bytes = arena_elem_bytes();
    let mut pos = vec![0u32; n];
    for (p, &id) in order.iter().enumerate() {
        pos[id as usize] = p as u32;
    }

    // Producer→consumer distances over the arena-read edges.
    let mut dists: Vec<u32> = Vec::new();
    for (p, &id) in order.iter().enumerate() {
        for &c in arena_reads(ast.value(id).op, ast.children_ids(id)) {
            dists.push(p as u32 - pos[c as usize]);
        }
    }
    dists.sort_unstable();
    let n_edges = dists.len();
    let pick = |q: f64| -> u32 {
        if n_edges == 0 {
            0
        } else {
            dists[((n_edges as f64 - 1.0) * q).round() as usize]
        }
    };
    let share = |lim: u32| -> f64 {
        if n_edges == 0 {
            0.0
        } else {
            dists.partition_point(|&d| d <= lim) as f64 / n_edges as f64
        }
    };
    let dist_mean = if n_edges == 0 {
        0.0
    } else {
        dists.iter().map(|&d| d as f64).sum::<f64>() / n_edges as f64
    };

    // Live-width profile: replay the same slot allocation the lowering does, counting
    // occupancy after each instruction.
    let Liveness {
        expiry_off,
        expiry,
        live_end,
    } = liveness(ast, order);
    let mut live = [0i64; N_ARENAS];
    let mut slots_sum = 0f64;
    let mut bytes_sum = 0f64;
    let mut slots_peak = 0i64;
    let mut bytes_peak = 0i64;
    for (p, &id) in order.iter().enumerate() {
        if let Some(s) = an.out_type(id).storage() {
            live[arena_index(s)] += 1;
        }
        let slots: i64 = live.iter().sum();
        let b: i64 = live.iter().zip(bytes).map(|(&l, e)| l * e as i64).sum();
        slots_peak = slots_peak.max(slots);
        bytes_peak = bytes_peak.max(b);
        slots_sum += slots as f64;
        bytes_sum += b as f64;
        for k in expiry_off[p]..expiry_off[p + 1] {
            let dead = expiry[k as usize];
            if live_end[dead as usize] {
                continue;
            }
            if let Some(s) = an.out_type(dead).storage() {
                live[arena_index(s)] -= 1;
            }
        }
    }

    // Discriminant runs over the emitted stream.
    let mut n_runs = 0usize;
    let mut prev = u8::MAX;
    let mut per_kind = [(0usize, 0usize); 38];
    for instr in prog.instrs.iter() {
        let k = instr.kind();
        per_kind[k as usize].0 += 1;
        if k != prev {
            n_runs += 1;
            per_kind[k as usize].1 += 1;
            prev = k;
        }
    }
    let mut top: Vec<(u8, usize, usize)> = (0u8..38)
        .map(|k| (k, per_kind[k as usize].0, per_kind[k as usize].1))
        .filter(|&(_, c, _)| c > 0)
        .collect();
    top.sort_by_key(|&(_, c, _)| std::cmp::Reverse(c));
    top.truncate(6);
    let top_kinds: Vec<(&'static str, usize, usize)> = top
        .into_iter()
        .map(|(k, c, r)| (super::layout::Instr::kind_name(k), c, r))
        .collect();

    // Critical path over the arena-read edges — a property of the DAG, not the order.
    let mut depth = vec![0u32; n];
    let mut max_depth = 0u32;
    for id in 0..n as NodeId {
        let d = arena_reads(ast.value(id).op, ast.children_ids(id))
            .iter()
            .map(|&c| depth[c as usize] + 1)
            .max()
            .unwrap_or(0);
        depth[id as usize] = d;
        max_depth = max_depth.max(d);
    }

    ProgramMetrics {
        n_instrs: n,
        n_edges,
        dist_mean,
        dist_p50: pick(0.5),
        dist_p90: pick(0.9),
        dist_max: dists.last().copied().unwrap_or(0),
        dist_le1: share(1),
        dist_le8: share(8),
        dist_le64: share(64),
        live_slots_peak: slots_peak as u32,
        live_slots_mean: slots_sum / n.max(1) as f64,
        live_bytes_peak: bytes_peak as usize,
        live_bytes_mean: bytes_sum / n.max(1) as f64,
        arena_sizes: prog.arena_sizes,
        n_runs,
        mean_run: n as f64 / n_runs.max(1) as f64,
        top_kinds,
        depth: max_depth + 1,
        ilp: n as f64 / (max_depth + 1) as f64,
    }
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

    /// Study processes: the llj subprocesses the hadronic gates spend their time in,
    /// plus a 2→2 and the 2→6 stress case.
    const STUDY: [(&str, &str); 7] = [
        ("gu_to_epemu", "g u > e+ e- u QCD=2 QED=2"),
        ("gux_to_epemux", "g u~ > e+ e- u~ QCD=2 QED=2"),
        ("uux_to_epemg", "u u~ > e+ e- g QCD=2 QED=2"),
        ("ddx_to_epemg", "d d~ > e+ e- g QCD=2 QED=2"),
        ("gg_to_gg", "g g > g g"),
        ("ee_to_mumu_tata_qcd0", "e+ e- > mu+ mu- ta+ ta- QCD=0"),
        ("uux_to_ccx_emmm_qcd0", "u u~ > c c~ e+ e- mu+ mu- QCD=0"),
    ];

    fn compiled(process: &str) -> (AmplitudeEvaluator, EvaluatedModel) {
        compiled_maybe_pruned(process, true)
    }

    /// The production evaluator is the helicity-pruned one (`prune = true`, what
    /// `eval_m2` and the MG-comparable bench run); `prune = false` keeps every
    /// helicity combination, which is the far larger arena.
    fn compiled_maybe_pruned(process: &str, prune: bool) -> (AmplitudeEvaluator, EvaluatedModel) {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let opts = ParsingOptions::default();
        let pc = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
        let sets = generate_from_proc_card(&pc, &model).unwrap();
        let mut eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
        if prune {
            eval.prune_zero_helicities(&evaluated);
        }
        (eval, evaluated)
    }

    fn points(n_ext: usize, n: usize) -> Vec<Vec<LorentzVector<f64>>> {
        let mut rng = StdRng::seed_from_u64(0xBE7C4);
        let sqrt_s = 500.0;
        (0..n)
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

    /// Every alternative order computes the same `eval_m2`, to the last bit: a
    /// topological reorder changes which slot a value lands in and when, never the
    /// arithmetic that produces it or the readout that consumes it.
    #[test]
    fn alternative_orders_are_bit_identical() {
        check_bit_identical(&[
            "e+ e- > mu+ mu-",
            "u u~ > u u~",
            "g g > g g",
            "g u > e+ e- u QCD=2 QED=2",
            "e+ e- > mu+ mu- a",
            "e+ e- > t t~",
            "e+ e- > w+ w-",
            "e+ e- > mu+ mu- ta+ ta- QCD=0",
        ]);
    }

    /// The same equality on the 2→6 stress case, whose arena is two orders of
    /// magnitude larger. Ignored by default — it compiles for minutes.
    #[test]
    #[ignore = "compiles the 2→6; run explicitly"]
    fn alternative_orders_are_bit_identical_2to6() {
        check_bit_identical(&["u u~ > c c~ e+ e- mu+ mu- QCD=0"]);
    }

    fn check_bit_identical(processes: &[&str]) {
        for &process in processes {
            let prev = set_active(Schedule::Arena);
            let (eval, evaluated) = compiled(process);
            let amp = BoundAmplitude::<f64>::bind(&eval, &evaluated);
            let pts = points(eval.n_ext(), 8);
            let mut scratch = amp.scratch_space();
            let want: Vec<f64> = pts.iter().map(|p| amp.eval_m2(p, &mut scratch)).collect();
            drop(amp);
            // Bit equality against a stream of zeros or NaNs would be vacuous.
            assert!(
                want.iter().all(|v| v.is_finite() && *v != 0.0),
                "{process}: reference |M|² stream is not finite and non-zero: {want:?}"
            );

            for sched in Schedule::ALL {
                set_active(sched);
                let (eval2, _) = compiled(process);
                // The comparison is only evidence if the alternative order is in fact
                // a different stream: assert the reorder happened before trusting the
                // values it produced.
                if sched != Schedule::Arena {
                    let folded = eval2.folded_hel();
                    let ast = &folded.ast;
                    let arena: Vec<NodeId> = (0..ast.len() as NodeId).collect();
                    let order = build_order(ast, folded.analysis(), sched);
                    assert_ne!(
                        order,
                        arena,
                        "{process}: schedule {} reproduced arena order — the \
                         bit-identity check would be vacuous",
                        sched.name()
                    );
                }
                let amp2 = BoundAmplitude::<f64>::bind(&eval2, &evaluated);
                let mut scratch2 = amp2.scratch_space();
                let got: Vec<f64> = pts.iter().map(|p| amp2.eval_m2(p, &mut scratch2)).collect();
                for (i, (w, g)) in want.iter().zip(&got).enumerate() {
                    assert_eq!(
                        w.to_bits(),
                        g.to_bits(),
                        "{process}: schedule {} changed eval_m2 at point {i}: {w:e} vs {g:e}",
                        sched.name()
                    );
                }
            }
            set_active(prev);
        }
    }

    /// A build with no hook active emits the op-blocked order — the arena-bytes
    /// fallback stays out of the way on the programs production actually compiles, and
    /// that order is a different stream from the interning order it replaced.
    #[test]
    fn production_builds_are_op_blocked() {
        for process in ["u u~ > u u~", "g g > g g", "g u > e+ e- u QCD=2 QED=2"] {
            let prev = set_active(PRODUCTION);
            let (eval, _) = compiled(process);
            let folded = eval.folded_hel();
            let (ast, an) = (&folded.ast, folded.analysis());
            let order = super::super::layout::op_blocked_order(ast, an);
            let arena: Vec<NodeId> = (0..ast.len() as NodeId).collect();
            assert_ne!(order, arena, "{process}: op-blocked reproduced arena order");
            let want = Program::build_ordered(ast, an, &order);
            let got = Program::build(ast, an);
            assert_eq!(
                got.dest, want.dest,
                "{process}: the default build is not the op-blocked order"
            );
            assert_eq!(got.arena_sizes, want.arena_sizes, "{process}");
            set_active(prev);
        }
    }

    /// What share of a production program's nodes produce a value a change of `αs`
    /// cannot move — the ceiling on splitting the stream into a coupling-invariant
    /// prefix that a second evaluation at a moved coupling could inherit.
    ///
    /// A node is invariant when every constant it reaches carries `G` to the power
    /// zero (the same tagging [`ScaleAwareAmplitude`](super::super::rescale) rescales
    /// the pools by) and every operand it reads is invariant; momentum read-offs are
    /// functions of the point alone.
    #[test]
    #[ignore = "study instrumentation; run with --nocapture"]
    fn alpha_s_invariant_share() {
        use super::super::layout::arena_reads;
        use super::super::op::{ConstKind, Op};
        println!("process\tnodes\tinvariant\tshare\tconsts\tconsts_invariant");
        for (name, process) in STUDY {
            let (eval, evaluated) = compiled(process);
            let folded = eval.folded_hel();
            let model = evaluated.model();
            let mut driven = model.params.dependents("aS");
            driven.insert("aS".to_owned());
            let powers = folded.g_powers(model, &driven);
            let ast = &folded.ast;

            let mut inv = vec![false; ast.len()];
            let mut n_const = 0usize;
            let mut n_const_inv = 0usize;
            for id in 0..ast.len() as NodeId {
                let n = ast.value(id);
                let kids = ast.children_ids(id);
                let v = match n.op {
                    Op::Coupling | Op::Mass | Op::Width | Op::Coeff | Op::CoeffRat => {
                        n_const += 1;
                        let idx = n.leaf.index() as usize;
                        let p = match n.leaf.kind() {
                            ConstKind::Complex => powers.complex.get(idx).copied().flatten(),
                            ConstKind::Real => powers.real.get(idx).copied().flatten(),
                            _ => None,
                        };
                        let v = p == Some(0);
                        n_const_inv += usize::from(v);
                        v
                    }
                    Op::PMom | Op::PMomOut => true,
                    Op::Flows | Op::Hels | Op::Configs => false,
                    _ => arena_reads(n.op, kids).iter().all(|&k| inv[k as usize]),
                };
                inv[id as usize] = v;
            }
            let total = ast.len();
            let n_inv = inv.iter().filter(|b| **b).count();
            println!(
                "{name}\t{total}\t{n_inv}\t{:.1}%\t{n_const}\t{n_const_inv}",
                n_inv as f64 / total as f64 * 100.0
            );
        }
    }

    /// The study table: per process × order, the metrics an execution order can move.
    /// Ignored by default — it compiles a 2→6.
    #[test]
    #[ignore = "study instrumentation; run with --nocapture"]
    fn execution_order_metrics() {
        println!(
            "process\torder\tinstrs\tedges\tdist_mean\tdist_p50\tdist_p90\tdist_max\t\
             le1\tle8\tle64\tslots_peak\tslots_mean\tbytes_peak\tbytes_mean\t\
             runs\tmean_run\tdepth\tilp\tarenas\ttop_kinds"
        );
        for (name, process, prune) in STUDY
            .iter()
            .flat_map(|&(n, p)| [(n, p, true), (n, p, false)])
        {
            let prev = set_active(Schedule::Arena);
            let (eval, _) = compiled_maybe_pruned(process, prune);
            let folded = eval.folded_hel();
            let (ast, an) = (&folded.ast, folded.analysis());
            let name = if prune {
                name.to_string()
            } else {
                format!("{name}@unpruned")
            };
            for sched in Schedule::ALL {
                let order = build_order(ast, an, sched);
                assert!(is_topological(ast, &order), "{name}/{}", sched.name());
                let prog = Program::build_ordered(ast, an, &order);
                let m = measure(ast, an, &prog, &order);
                println!(
                    "{name}\t{}\t{}\t{}\t{:.1}\t{}\t{}\t{}\t{:.3}\t{:.3}\t{:.3}\t\
                     {}\t{:.0}\t{}\t{:.0}\t{}\t{:.2}\t{}\t{:.1}\t{:?}\t{}",
                    sched.name(),
                    m.n_instrs,
                    m.n_edges,
                    m.dist_mean,
                    m.dist_p50,
                    m.dist_p90,
                    m.dist_max,
                    m.dist_le1,
                    m.dist_le8,
                    m.dist_le64,
                    m.live_slots_peak,
                    m.live_slots_mean,
                    m.live_bytes_peak,
                    m.live_bytes_mean,
                    m.n_runs,
                    m.mean_run,
                    m.depth,
                    m.ilp,
                    m.arena_sizes,
                    m.top_kinds
                        .iter()
                        .map(|(k, c, r)| format!("{k}:{c}/{r}"))
                        .collect::<Vec<_>>()
                        .join(","),
                );
            }
            set_active(prev);
        }
    }
}
