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
        debug_assert!(index <= Self::INDEX_MASK, "operand index {index} overflows");
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
    Mul {
        start: u32,
        len: u32,
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
    LowerVout {
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
}

/// Where the amplitude value(s) live after a run.
#[derive(Clone, Debug)]
pub(super) enum RootKind {
    /// Single scalar amplitude at this index of the scalar arena.
    Single(u32),
    /// Per-flow JAMP scalars, at these indices of the scalar arena.
    Flows(Box<[u32]>),
}

/// A folded arena lowered to a typed instruction stream.
#[derive(Clone, Debug)]
pub(super) struct Program {
    /// One instruction per node, in arena (storage) order.
    pub(super) instrs: Box<[Instr]>,
    /// Per-node index within its result arena (`loc[id]`); used to reconstruct a slot for
    /// the debug/extended-validation cross-check, and to locate the root.
    pub(super) loc: Box<[u32]>,
    /// Number of results each arena holds after a full run (`arena_sizes[class]`).
    pub(super) arena_sizes: [u32; N_ARENAS],
    /// Shared operand table for the variadic/mixed-class instructions.
    pub(super) operands: Box<[OperandRef]>,
    /// Momentum-table ids for the `PMomOut` operand slices — the momenta whose negated sum
    /// is the vertex output leg's structure momentum.
    pub(super) mom_operands: Box<[u32]>,
    pub(super) root: RootKind,
}

impl Program {
    /// Lower a folded arena + its analysis into a typed instruction stream.
    pub(super) fn build(ast: &Ast<Const>, an: &NodeAnalysis) -> Program {
        let n = ast.len();
        let mut instrs: Vec<Instr> = Vec::with_capacity(n);
        let mut loc = vec![0u32; n];
        let mut counts = [0u32; N_ARENAS];
        let mut operands: Vec<OperandRef> = Vec::new();
        let mut mom_operands: Vec<u32> = Vec::new();

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

        for id in 0..n as NodeId {
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
                    let start = operands.len() as u32;
                    for &k in kids {
                        operands.push(opref(k, &loc));
                    }
                    Instr::Mul {
                        start,
                        len: kids.len() as u32,
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
                Op::LowerVout => Instr::LowerVout { v: li(kids[0]) },
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
            };
            instrs.push(instr);

            // This node's result lands at the next free slot of its arena.
            if let Some(s) = an.out_type(id).storage() {
                let ai = arena_index(s);
                loc[id as usize] = counts[ai];
                counts[ai] += 1;
            }
        }

        let root_id = ast.root();
        let root = if ast.value(root_id).op == Op::Flows {
            let flows: Box<[u32]> = ast
                .children_ids(root_id)
                .iter()
                .map(|&c| loc[c as usize])
                .collect();
            RootKind::Flows(flows)
        } else {
            RootKind::Single(loc[root_id as usize])
        };

        Program {
            instrs: instrs.into_boxed_slice(),
            loc: loc.into_boxed_slice(),
            arena_sizes: counts,
            operands: operands.into_boxed_slice(),
            mom_operands: mom_operands.into_boxed_slice(),
            root,
        }
    }
}
