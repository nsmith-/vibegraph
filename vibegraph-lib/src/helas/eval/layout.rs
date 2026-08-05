//! Typed instruction stream over a folded [`Ast<Const>`].
//!
//! A [`Program`] lowers the folded arena into one instruction per node, carrying operand
//! references already resolved to a *typed* slot: a `(result class, index)` pair into the
//! per-class result arenas ([`super::run::ScratchSpace`]). Because every node's output
//! class is statically known ([`NodeAnalysis::out_type`]), the runtime reads each operand
//! directly from the arena its class lives in — no per-value enum tag, and the bra/ket
//! resolution and chirality that the generic dispatch decides at runtime are baked into
//! the instruction at build time.
//!
//! The element types are the `wavefn.rs` currents (momentum still embedded), so the
//! runtime arithmetic is byte-identical to the generic [`WaveformSlot`](super::waveform_slot::WaveformSlot)
//! forward pass — the layout changes *where* results live, not *how* they are computed.

use super::analysis::{NodeAnalysis, NodeType, Storage};
use super::ast::Ast;
use super::op::{Const, ConstKind, NodeId, Op};
use super::tree::Tree;
use crate::helas::repr::numbers::Chirality;

/// The result-arena classes, in a fixed index order (`0..N_ARENAS`). Mirrors
/// [`Storage`]; the runtime holds one arena per class.
pub(super) const N_ARENAS: usize = 5;

/// The arena index of a [`Storage`] class.
#[inline]
pub(super) fn arena_index(s: Storage) -> usize {
    match s {
        Storage::Real => 0,
        Storage::Scalar => 1,
        Storage::Vector => 2,
        Storage::FermionIn => 3,
        Storage::FermionOut => 4,
    }
}

/// A typed operand reference: result-arena class (top 3 bits) + arena index (low 29).
/// Used where an instruction's operand class varies across nodes (the mixed `Mul` factor
/// list and the momentum read-offs); fixed-class operands carry a bare arena index.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct OperandRef(u32);

impl OperandRef {
    const CLASS_SHIFT: u32 = 29;
    const INDEX_MASK: u32 = (1 << Self::CLASS_SHIFT) - 1;

    fn new(class: Storage, index: u32) -> OperandRef {
        assert!(index <= Self::INDEX_MASK, "operand index {index} overflows");
        OperandRef(((arena_index(class) as u32) << Self::CLASS_SHIFT) | index)
    }

    /// The operand's result-arena class index (`0..N_ARENAS`).
    #[inline]
    pub(super) fn class(self) -> usize {
        (self.0 >> Self::CLASS_SHIFT) as usize
    }

    /// The operand's index within its arena.
    #[inline]
    pub(super) fn index(self) -> usize {
        (self.0 & Self::INDEX_MASK) as usize
    }
}

impl std::fmt::Debug for OperandRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "op[{}]#{}", self.class(), self.index())
    }
}

/// One lowered node. Operand fields are indices into the arena the operand's class fixes
/// (a `GammaVout` reads its bra from the flow-out arena and its ket from the flow-in
/// arena, etc.); variadic/mixed-class operands are a `(start, len)` slice of the shared
/// [`Program::operands`] table.
#[derive(Clone, Copy, Debug)]
pub(super) enum Instr {
    /// Complex-pool read (coupling or imaginary rational) → a zero-momentum scalar.
    ComplexConst {
        pool: u32,
    },
    /// Real-pool read (mass/width/coeff or real rational) → a bare real.
    RealConst {
        pool: u32,
    },
    ExternalScalar {
        leg: u32,
    },
    ExternalVector {
        leg: u32,
    },
    ExternalFin {
        leg: u32,
    },
    ExternalFout {
        leg: u32,
    },
    PropagateScalar {
        input: u32,
        mass: u32,
        width: u32,
        mom: u32,
    },
    PropagateVector {
        input: u32,
        mass: u32,
        width: u32,
        mom: u32,
    },
    PropagateFin {
        input: u32,
        mass: u32,
        width: u32,
        mom: u32,
    },
    PropagateFout {
        input: u32,
        mass: u32,
        width: u32,
        mom: u32,
    },
    AddScalar {
        start: u32,
        len: u32,
    },
    AddVector {
        start: u32,
        len: u32,
    },
    AddFin {
        start: u32,
        len: u32,
    },
    AddFout {
        start: u32,
        len: u32,
    },
    /// scalar × scalar → scalar: one complex multiply.
    MulScalarC {
        a: u32,
        b: u32,
    },
    /// scalar × real → scalar: real-scale the complex value (two real muls).
    MulScalarR {
        s: u32,
        r: u32,
    },
    /// scalar × vector: complex-scale the current.
    ScaleVecC {
        v: u32,
        scale: u32,
    },
    /// real × vector: real-scale the current.
    ScaleVecR {
        v: u32,
        scale: u32,
    },
    /// scalar × flow-in current: complex-scale.
    ScaleFinC {
        f: u32,
        scale: u32,
    },
    /// real × flow-in current: real-scale.
    ScaleFinR {
        f: u32,
        scale: u32,
    },
    /// scalar × flow-out current: complex-scale.
    ScaleFoutC {
        f: u32,
        scale: u32,
    },
    /// real × flow-out current: real-scale.
    ScaleFoutR {
        f: u32,
        scale: u32,
    },
    GammaVout {
        bra: u32,
        ket: u32,
        reversed: bool,
    },
    FfvVout {
        bra: u32,
        ket: u32,
        gl: u32,
        gr: u32,
        reversed: bool,
    },
    GammaFin {
        v: u32,
        f: u32,
    },
    GammaFout {
        v: u32,
        f: u32,
    },
    FfvFin {
        v: u32,
        f: u32,
        gl: u32,
        gr: u32,
    },
    FfvFout {
        v: u32,
        f: u32,
        gl: u32,
        gr: u32,
    },
    ProjFin {
        f: u32,
        chirality: Chirality,
    },
    ProjFout {
        f: u32,
        chirality: Chirality,
    },
    Bilinear {
        bra: u32,
        ket: u32,
        chirality: Chirality,
    },
    Metric {
        a: u32,
        b: u32,
    },
    MetricVout {
        v: u32,
    },
    /// `P` read-off of an input line: its structure momentum is the momentum-table entry
    /// `mom` (the operand's momentum id), promoted to a vector current.
    PMom {
        mom: u32,
    },
    /// `P` read-off of a vertex's output leg: `−Σ` over the input operands' momentum-table
    /// entries, indexed by `[start, start+len)` into [`Program::mom_operands`].
    PMomOut {
        start: u32,
        len: u32,
    },
    /// Variadic per-flow-JAMP amplitude root: computes nothing (its children's scalars
    /// are read out by the multi-flow evaluator).
    Flows,
    /// Variadic per-helicity-combination root: computes nothing (its children's scalars
    /// are read out by the helicity-summed evaluator).
    Hels,
    /// Variadic root bundle: computes nothing (the amplitude root and the
    /// per-configuration diagram amplitudes under it are read out after the pass).
    Configs,
}

impl Instr {
    /// The variant's index in declaration order — the jump-table entry the forward
    /// pass's single dispatch site selects. Its run lengths over the instruction
    /// stream are what that indirect branch's predictability is made of.
    #[cfg_attr(not(any(test, feature = "eval-schedule-study")), allow(dead_code))]
    pub(super) fn kind(&self) -> u8 {
        match self {
            Instr::ComplexConst { .. } => 0,
            Instr::RealConst { .. } => 1,
            Instr::ExternalScalar { .. } => 2,
            Instr::ExternalVector { .. } => 3,
            Instr::ExternalFin { .. } => 4,
            Instr::ExternalFout { .. } => 5,
            Instr::PropagateScalar { .. } => 6,
            Instr::PropagateVector { .. } => 7,
            Instr::PropagateFin { .. } => 8,
            Instr::PropagateFout { .. } => 9,
            Instr::AddScalar { .. } => 10,
            Instr::AddVector { .. } => 11,
            Instr::AddFin { .. } => 12,
            Instr::AddFout { .. } => 13,
            Instr::MulScalarC { .. } => 14,
            Instr::MulScalarR { .. } => 15,
            Instr::ScaleVecC { .. } => 16,
            Instr::ScaleVecR { .. } => 17,
            Instr::ScaleFinC { .. } => 18,
            Instr::ScaleFinR { .. } => 19,
            Instr::ScaleFoutC { .. } => 20,
            Instr::ScaleFoutR { .. } => 21,
            Instr::GammaVout { .. } => 22,
            Instr::FfvVout { .. } => 23,
            Instr::GammaFin { .. } => 24,
            Instr::GammaFout { .. } => 25,
            Instr::FfvFin { .. } => 26,
            Instr::FfvFout { .. } => 27,
            Instr::ProjFin { .. } => 28,
            Instr::ProjFout { .. } => 29,
            Instr::Bilinear { .. } => 30,
            Instr::Metric { .. } => 31,
            Instr::MetricVout { .. } => 32,
            Instr::PMom { .. } => 33,
            Instr::PMomOut { .. } => 34,
            Instr::Flows => 35,
            Instr::Hels => 36,
            Instr::Configs => 37,
        }
    }

    /// Human-readable variant name, for the study's per-kind tables.
    #[cfg_attr(not(any(test, feature = "eval-schedule-study")), allow(dead_code))]
    pub(super) fn kind_name(kind: u8) -> &'static str {
        const NAMES: [&str; 38] = [
            "ComplexConst",
            "RealConst",
            "ExternalScalar",
            "ExternalVector",
            "ExternalFin",
            "ExternalFout",
            "PropagateScalar",
            "PropagateVector",
            "PropagateFin",
            "PropagateFout",
            "AddScalar",
            "AddVector",
            "AddFin",
            "AddFout",
            "MulScalarC",
            "MulScalarR",
            "ScaleVecC",
            "ScaleVecR",
            "ScaleFinC",
            "ScaleFinR",
            "ScaleFoutC",
            "ScaleFoutR",
            "GammaVout",
            "FfvVout",
            "GammaFin",
            "GammaFout",
            "FfvFin",
            "FfvFout",
            "ProjFin",
            "ProjFout",
            "Bilinear",
            "Metric",
            "MetricVout",
            "PMom",
            "PMomOut",
            "Flows",
            "Hels",
            "Configs",
        ];
        NAMES[kind as usize]
    }
}

/// Where the amplitude value(s) live after a run.
#[derive(Clone, Debug)]
pub(super) enum RootKind {
    /// Single scalar amplitude at this index of the scalar arena.
    Single(u32),
    /// Per-flow JAMP scalars, at these indices of the scalar arena. Only the
    /// per-flow JAMP test probes read these locations (the production helicity sum
    /// reads [`RootKind::Hels`] instead).
    Flows(#[cfg_attr(not(test), allow(dead_code))] Box<[u32]>),
    /// Helicity-expanded amplitude: `locs` holds the scalar-arena indices of every
    /// combination's per-flow JAMPs, combination-major (`locs[c*n_flows + i]` is
    /// combination `c`'s flow-`i` JAMP; `n_flows = 1` stores each combination's
    /// single amplitude scalar).
    Hels { n_flows: u32, locs: Box<[u32]> },
}

/// A folded arena lowered to a typed instruction stream.
#[derive(Clone, Debug)]
pub(super) struct Program {
    /// One instruction per node, in execution order.
    pub(super) instrs: Box<[Instr]>,
    /// Destination slot of each instruction (`dest[pos]`), within the arena its
    /// output class fixes — the write index the forward pass uses.
    pub(super) dest: Box<[u32]>,
    /// Per-node index within its result arena (`loc[id]`), for the debug /
    /// `extended-validation` cross-check that reconstructs every node's slot. The
    /// forward pass reads `dest` instead, so this is absent from a release build.
    #[cfg(any(debug_assertions, feature = "extended-validation"))]
    pub(super) loc: Box<[u32]>,
    /// Slots each arena needs for a run (`arena_sizes[class]`): the peak number of
    /// simultaneously live results of that class, not the node count — slots are
    /// recycled once their last reader has executed.
    pub(super) arena_sizes: [u32; N_ARENAS],
    /// Shared operand table for the variadic/mixed-class instructions.
    pub(super) operands: Box<[OperandRef]>,
    /// Momentum-table ids for the `PMomOut` operand slices — the momenta whose negated sum
    /// is the vertex output leg's structure momentum.
    pub(super) mom_operands: Box<[u32]>,
    pub(super) root: RootKind,
    /// Scalar-arena indices of the per-configuration diagram amplitudes `A_d` (the
    /// children of the [`Op::Configs`] root bundle), in configuration order. Under a
    /// [`RootKind::Hels`] root they are combination-major:
    /// `amp_locs[c * n_amps + d]`. Empty when the arena carries no `Configs` bundle.
    pub(super) amp_locs: Box<[u32]>,
    /// Diagram amplitudes per helicity combination — the row length of `amp_locs`.
    pub(super) n_amps: u32,
}

/// The operand nodes an instruction actually reads from the result arenas: all
/// children, except that `PMom`/`PMomOut` read only their operands' momentum-table
/// ids and the variadic roots' children are read out by the evaluator *after* the
/// pass (kept live to the end instead).
pub(super) fn arena_reads(op: Op, kids: &[NodeId]) -> &[NodeId] {
    match op {
        Op::PMom | Op::PMomOut | Op::Flows | Op::Hels | Op::Configs => &[],
        _ => kids,
    }
}

/// Unwrap an [`Op::Configs`] root bundle into `(amplitude root, per-configuration
/// diagram amplitudes)`. A node that is not a bundle is the amplitude root itself and
/// carries no configuration amplitudes.
fn split_configs(ast: &Ast<Const>, id: NodeId) -> (NodeId, &[NodeId]) {
    if ast.value(id).op == Op::Configs {
        let kids = ast.children_ids(id);
        (kids[0], &kids[1..])
    } else {
        (id, &[])
    }
}

/// Liveness of every node's result over one execution order: where its last arena read
/// happens, and whether the evaluator reads it out after the pass.
pub(super) struct Liveness {
    /// `expiry[expiry_off[p]..expiry_off[p + 1]]` are the nodes whose last arena read
    /// is the instruction at position `p` — a CSR so a forward scan releases slots in
    /// O(1). A node never read is listed at its own position.
    pub(super) expiry_off: Vec<u32>,
    pub(super) expiry: Vec<NodeId>,
    /// Nodes the evaluator reads out of the arenas after the pass; their slots are
    /// never recycled.
    pub(super) live_end: Vec<bool>,
}

/// Liveness over the execution order `order` (`order[pos]` is the node executed at
/// `pos`; it must list every node exactly once, children before parents).
pub(super) fn liveness(ast: &Ast<Const>, order: &[NodeId]) -> Liveness {
    let n = ast.len();
    let mut pos_of = vec![0u32; n];
    for (p, &id) in order.iter().enumerate() {
        pos_of[id as usize] = p as u32;
    }
    // Last arena read of each node, as a position (its own position if never read).
    let mut last_use: Vec<u32> = pos_of.clone();
    for (p, &id) in order.iter().enumerate() {
        for &k in arena_reads(ast.value(id).op, ast.children_ids(id)) {
            last_use[k as usize] = p as u32;
        }
    }
    // Root scalars read out by the evaluator after the pass.
    let mut live_end = vec![false; n];
    {
        let root_id = ast.root();
        let mark = |live_end: &mut Vec<bool>, id: NodeId| {
            let (amplitude, amps) = split_configs(ast, id);
            if ast.value(amplitude).op == Op::Flows {
                for &j in ast.children_ids(amplitude) {
                    live_end[j as usize] = true;
                }
            } else {
                live_end[amplitude as usize] = true;
            }
            for &a in amps {
                live_end[a as usize] = true;
            }
        };
        if ast.value(root_id).op == Op::Hels {
            for &c in ast.children_ids(root_id) {
                mark(&mut live_end, c);
            }
        } else {
            mark(&mut live_end, root_id);
        }
    }
    let mut expiry_off = vec![0u32; n + 1];
    for &lu in &last_use {
        expiry_off[lu as usize + 1] += 1;
    }
    for i in 0..n {
        expiry_off[i + 1] += expiry_off[i];
    }
    let mut expiry = vec![0 as NodeId; n];
    let mut cursor = expiry_off.clone();
    for (k, &lu) in last_use.iter().enumerate() {
        expiry[cursor[lu as usize] as usize] = k as NodeId;
        cursor[lu as usize] += 1;
    }
    Liveness {
        expiry_off,
        expiry,
        live_end,
    }
}

impl Program {
    /// Lower a folded arena + its analysis into a typed instruction stream, in arena
    /// (storage) order — or, when the execution-order study hook selects one, in an
    /// alternative topological order over the same DAG.
    pub(super) fn build(ast: &Ast<Const>, an: &NodeAnalysis) -> Program {
        let arena_order: Vec<NodeId> = (0..ast.len() as NodeId).collect();
        let base = Program::build_ordered(ast, an, &arena_order);
        #[cfg(any(test, feature = "eval-schedule-study"))]
        {
            let sched = super::schedule::active();
            if sched != super::schedule::Schedule::Arena {
                let order = super::schedule::build_order(ast, an, &base, sched);
                return Program::build_ordered(ast, an, &order);
            }
        }
        base
    }

    /// Lower a folded arena + its analysis into a typed instruction stream, executing
    /// nodes in the order `order` (any topological order of the DAG; `order[pos]` is
    /// the node at position `pos`).
    ///
    /// Result slots are allocated by liveness: a node's slot is recycled once its last
    /// arena read has executed, so `arena_sizes` is the peak number of simultaneously
    /// live values per class, not the node count. A node's own slot is never one of its
    /// operands' (operands release only after the instruction's slot is assigned), and
    /// the root scalars the evaluator reads after the pass stay live to the end.
    pub(super) fn build_ordered(ast: &Ast<Const>, an: &NodeAnalysis, order: &[NodeId]) -> Program {
        let n = ast.len();
        assert_eq!(order.len(), n, "execution order must cover every node once");
        let mut instrs: Vec<Instr> = Vec::with_capacity(n);
        let mut dest: Vec<u32> = Vec::with_capacity(n);
        let mut loc = vec![0u32; n];
        let mut counts = [0u32; N_ARENAS];
        let mut operands: Vec<OperandRef> = Vec::new();
        let mut mom_operands: Vec<u32> = Vec::new();

        let Liveness {
            expiry_off,
            expiry,
            live_end,
        } = liveness(ast, order);
        let mut free: [Vec<u32>; N_ARENAS] = Default::default();

        // The (bra = flow-out, ket = flow-in, reversed) resolution of a two-fermion
        // bilinear, mirroring the runtime `resolve_bra_ket`: `reversed` is set when the
        // operands arrive in (ket, bra) order.
        let bra_ket = |a: NodeId, b: NodeId| -> (NodeId, NodeId, bool) {
            match an.out_type(a) {
                NodeType::FermionOut => (a, b, false),
                _ => (b, a, true),
            }
        };
        let opref = |id: NodeId, loc: &[u32]| -> OperandRef {
            let class = an
                .out_type(id)
                .storage()
                .expect("operand node has no result class");
            OperandRef::new(class, loc[id as usize])
        };

        for (pos, &id) in order.iter().enumerate() {
            let node = ast.value(id);
            let kids = ast.children_ids(id);
            let li = |k: NodeId| loc[k as usize];

            let instr = match node.op {
                Op::Coupling => Instr::ComplexConst {
                    pool: node.leaf.index(),
                },
                Op::Mass | Op::Width | Op::Coeff => Instr::RealConst {
                    pool: node.leaf.index(),
                },
                Op::CoeffRat => match node.leaf.kind() {
                    ConstKind::Complex => Instr::ComplexConst {
                        pool: node.leaf.index(),
                    },
                    ConstKind::Real => Instr::RealConst {
                        pool: node.leaf.index(),
                    },
                    other => panic!("CoeffRat leaf has unresolved kind {other:?}"),
                },
                Op::External => {
                    let leg = node.leaf.index();
                    match an.out_type(id) {
                        NodeType::ScalarWf => Instr::ExternalScalar { leg },
                        NodeType::Vector => Instr::ExternalVector { leg },
                        NodeType::FermionIn => Instr::ExternalFin { leg },
                        NodeType::FermionOut => Instr::ExternalFout { leg },
                        other => panic!("External node has unexpected output type {other:?}"),
                    }
                }
                Op::Propagate => {
                    let input = li(kids[0]);
                    let mass = li(kids[1]);
                    let width = li(kids[2]);
                    // A propagator preserves its input's momentum, so this node's own
                    // momentum id is the routed momentum the propagator sees.
                    let mom = an.mom_id(id);
                    match an.out_type(kids[0]).storage().unwrap() {
                        Storage::Scalar => Instr::PropagateScalar {
                            input,
                            mass,
                            width,
                            mom,
                        },
                        Storage::Vector => Instr::PropagateVector {
                            input,
                            mass,
                            width,
                            mom,
                        },
                        Storage::FermionIn => Instr::PropagateFin {
                            input,
                            mass,
                            width,
                            mom,
                        },
                        Storage::FermionOut => Instr::PropagateFout {
                            input,
                            mass,
                            width,
                            mom,
                        },
                        Storage::Real => panic!("Propagate on a real-constant input"),
                    }
                }
                Op::Add => {
                    let start = operands.len() as u32;
                    for &k in kids {
                        operands.push(opref(k, &loc));
                    }
                    let len = kids.len() as u32;
                    match an.out_type(id).storage().unwrap() {
                        Storage::Scalar => Instr::AddScalar { start, len },
                        Storage::Vector => Instr::AddVector { start, len },
                        Storage::FermionIn => Instr::AddFin { start, len },
                        Storage::FermionOut => Instr::AddFout { start, len },
                        Storage::Real => panic!("Add produced a real-constant"),
                    }
                }
                Op::Mul => {
                    // Production Muls are binary with at most one non-scalar operand and
                    // never real × real (every real-class node is a card constant, so an
                    // all-real product is folded away). Each case maps to exactly one typed
                    // variant; a violation is a compile-DAG bug, so it panics rather than
                    // falling back to a generic path.
                    assert_eq!(kids.len(), 2, "production Mul must be binary");
                    let s0 = an
                        .out_type(kids[0])
                        .storage()
                        .expect("Mul operand has no result class");
                    let s1 = an
                        .out_type(kids[1])
                        .storage()
                        .expect("Mul operand has no result class");
                    let (a, b) = (li(kids[0]), li(kids[1]));
                    match (s0, s1) {
                        (Storage::Scalar, Storage::Scalar) => Instr::MulScalarC { a, b },
                        (Storage::Scalar, Storage::Real) => Instr::MulScalarR { s: a, r: b },
                        (Storage::Real, Storage::Scalar) => Instr::MulScalarR { s: b, r: a },
                        (Storage::Scalar, Storage::Vector) => Instr::ScaleVecC { v: b, scale: a },
                        (Storage::Vector, Storage::Scalar) => Instr::ScaleVecC { v: a, scale: b },
                        (Storage::Real, Storage::Vector) => Instr::ScaleVecR { v: b, scale: a },
                        (Storage::Vector, Storage::Real) => Instr::ScaleVecR { v: a, scale: b },
                        (Storage::Scalar, Storage::FermionIn) => Instr::ScaleFinC { f: b, scale: a },
                        (Storage::FermionIn, Storage::Scalar) => Instr::ScaleFinC { f: a, scale: b },
                        (Storage::Real, Storage::FermionIn) => Instr::ScaleFinR { f: b, scale: a },
                        (Storage::FermionIn, Storage::Real) => Instr::ScaleFinR { f: a, scale: b },
                        (Storage::Scalar, Storage::FermionOut) => {
                            Instr::ScaleFoutC { f: b, scale: a }
                        }
                        (Storage::FermionOut, Storage::Scalar) => {
                            Instr::ScaleFoutC { f: a, scale: b }
                        }
                        (Storage::Real, Storage::FermionOut) => Instr::ScaleFoutR { f: b, scale: a },
                        (Storage::FermionOut, Storage::Real) => Instr::ScaleFoutR { f: a, scale: b },
                        (x, y) => panic!(
                            "Mul invariant violated: unsupported operand storage classes {x:?} × {y:?}"
                        ),
                    }
                }
                Op::GammaVout => {
                    let (bra, ket, reversed) = bra_ket(kids[0], kids[1]);
                    Instr::GammaVout {
                        bra: li(bra),
                        ket: li(ket),
                        reversed,
                    }
                }
                Op::FfvVout => {
                    let (bra, ket, reversed) = bra_ket(kids[0], kids[1]);
                    Instr::FfvVout {
                        bra: li(bra),
                        ket: li(ket),
                        gl: li(kids[2]),
                        gr: li(kids[3]),
                        reversed,
                    }
                }
                Op::GammaIout | Op::GammaOout => {
                    let v = li(kids[0]);
                    let f = li(kids[1]);
                    match an.out_type(kids[1]).storage().unwrap() {
                        Storage::FermionIn => Instr::GammaFin { v, f },
                        Storage::FermionOut => Instr::GammaFout { v, f },
                        other => panic!("off-shell fermion current on {other:?} input"),
                    }
                }
                Op::FfvIout | Op::FfvOout => {
                    let v = li(kids[0]);
                    let f = li(kids[1]);
                    let gl = li(kids[2]);
                    let gr = li(kids[3]);
                    match an.out_type(kids[1]).storage().unwrap() {
                        Storage::FermionIn => Instr::FfvFin { v, f, gl, gr },
                        Storage::FermionOut => Instr::FfvFout { v, f, gl, gr },
                        other => panic!("fused fermion current on {other:?} input"),
                    }
                }
                Op::ProjM | Op::ProjP => {
                    let chirality = if node.op == Op::ProjM {
                        Chirality::Left
                    } else {
                        Chirality::Right
                    };
                    let f = li(kids[0]);
                    match an.out_type(kids[0]).storage().unwrap() {
                        Storage::FermionIn => Instr::ProjFin { f, chirality },
                        Storage::FermionOut => Instr::ProjFout { f, chirality },
                        other => panic!("chiral projection on {other:?} input"),
                    }
                }
                Op::ProjMAmp | Op::ProjPAmp | Op::IdentityAmp => {
                    let chirality = match node.op {
                        Op::ProjMAmp => Chirality::Left,
                        Op::ProjPAmp => Chirality::Right,
                        _ => Chirality::Both,
                    };
                    let (bra, ket, _) = bra_ket(kids[0], kids[1]);
                    Instr::Bilinear {
                        bra: li(bra),
                        ket: li(ket),
                        chirality,
                    }
                }
                Op::Metric => Instr::Metric {
                    a: li(kids[0]),
                    b: li(kids[1]),
                },
                Op::MetricVout => Instr::MetricVout { v: li(kids[0]) },
                Op::PMom => Instr::PMom {
                    mom: an.mom_id(kids[0]),
                },
                Op::PMomOut => {
                    let start = mom_operands.len() as u32;
                    for &k in kids {
                        mom_operands.push(an.mom_id(k));
                    }
                    Instr::PMomOut {
                        start,
                        len: kids.len() as u32,
                    }
                }
                Op::Flows => Instr::Flows,
                Op::Hels => Instr::Hels,
                Op::Configs => Instr::Configs,
            };
            instrs.push(instr);

            // This node's result lands at a recycled slot of its arena, else a fresh one.
            if let Some(s) = an.out_type(id).storage() {
                let ai = arena_index(s);
                loc[id as usize] = free[ai].pop().unwrap_or_else(|| {
                    counts[ai] += 1;
                    counts[ai] - 1
                });
            }
            dest.push(loc[id as usize]);
            // Release slots whose last read was this instruction — after this node's own
            // slot is assigned, so an instruction never writes over one of its operands.
            for k in expiry_off[pos]..expiry_off[pos + 1] {
                let dead = expiry[k as usize];
                if live_end[dead as usize] {
                    continue;
                }
                if let Some(s) = an.out_type(dead).storage() {
                    free[arena_index(s)].push(loc[dead as usize]);
                }
            }
        }

        let root_id = ast.root();
        let mut amp_locs: Vec<u32> = Vec::new();
        let mut n_amps: Option<u32> = None;
        // Collect one combination's configuration amplitudes, checking that every
        // combination carries the same number of them (the row length `amp_locs` is
        // read back with).
        let mut take_amps = |amps: &[NodeId], loc: &[u32]| {
            amp_locs.extend(amps.iter().map(|&a| loc[a as usize]));
            let k = amps.len() as u32;
            assert_eq!(
                n_amps.unwrap_or(k),
                k,
                "helicity combinations disagree on configuration-amplitude count"
            );
            n_amps = Some(k);
        };
        let root = match ast.value(root_id).op {
            Op::Hels => {
                let mut n_flows = 0u32;
                let mut locs: Vec<u32> = Vec::new();
                for &c in ast.children_ids(root_id) {
                    let (amplitude, amps) = split_configs(ast, c);
                    take_amps(amps, &loc);
                    let combo_flows = if ast.value(amplitude).op == Op::Flows {
                        let jamps = ast.children_ids(amplitude);
                        locs.extend(jamps.iter().map(|&j| loc[j as usize]));
                        jamps.len() as u32
                    } else {
                        locs.push(loc[amplitude as usize]);
                        1
                    };
                    assert!(
                        n_flows == 0 || n_flows == combo_flows,
                        "helicity combinations disagree on flow count"
                    );
                    n_flows = combo_flows;
                }
                RootKind::Hels {
                    n_flows,
                    locs: locs.into_boxed_slice(),
                }
            }
            _ => {
                let (amplitude, amps) = split_configs(ast, root_id);
                take_amps(amps, &loc);
                if ast.value(amplitude).op == Op::Flows {
                    RootKind::Flows(
                        ast.children_ids(amplitude)
                            .iter()
                            .map(|&c| loc[c as usize])
                            .collect(),
                    )
                } else {
                    RootKind::Single(loc[amplitude as usize])
                }
            }
        };

        Program {
            instrs: instrs.into_boxed_slice(),
            dest: dest.into_boxed_slice(),
            #[cfg(any(debug_assertions, feature = "extended-validation"))]
            loc: loc.into_boxed_slice(),
            arena_sizes: counts,
            operands: operands.into_boxed_slice(),
            mom_operands: mom_operands.into_boxed_slice(),
            root,
            amp_locs: amp_locs.into_boxed_slice(),
            n_amps: n_amps.unwrap_or(0),
        }
    }
}
