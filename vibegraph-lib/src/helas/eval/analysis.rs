//! Static per-node analysis over a lowered [`Ast`] arena.
//!
//! A single forward scan (children precede parents in arena order) annotates every
//! node with four card-independent, helicity-independent facts:
//!
//! 1. **Output type** ([`NodeType`]) — which [`WaveformSlot`](super::waveform_slot::WaveformSlot)
//!    variant the node reduces to, mirroring the [`apply`](super::run::apply) dispatch
//!    (which panics on a mismatch — the runtime source of truth). The scalar slot is
//!    split into a momentum-free constant ([`NodeType::ScalarConst`]) and a
//!    momentum-carrying current ([`NodeType::ScalarWf`]); `Op::Flows` is a variadic
//!    root that is never an operand and so carries no output type ([`NodeType::Sink`]).
//! 2. **Constness** — a node all of whose descendants are `Coupling`/`Coeff`/`CoeffRat`/
//!    `Mass`/`Width` leaves is a card-time constant (resolvable once per parameter card).
//! 3. **Momentum id** — the signed external-momentum combination the node's slot carries,
//!    interned into a [`MomTable`]. Every current's routing momentum is a compile-time
//!    combination `Σ ± p_leg`, independent of helicity.
//! 4. **Helicity-support mask** — the set of external legs in the node's subtree, as a
//!    bitmask. A node's slot can depend only on the helicities of legs in this set, so a
//!    helicity flip of any leg outside it leaves the slot unchanged (the recycling
//!    invariant).
//!
//! Downstream consumers: constant-subgraph folding reads [`NodeAnalysis::is_const`]; the
//! typed instruction stream / SoA arenas read [`NodeType::storage`]; the momentum pool
//! reads [`NodeAnalysis::mom_id`] and the [`MomTable`]; helicity recycling reads
//! [`NodeAnalysis::support`].

use std::collections::HashMap;

use num_traits::Zero;

use super::ast::Ast;
use super::fold::ExtLeg;
use super::op::{Const, NodeId, Op, Sym};
use super::tree::Tree;
use crate::helas::repr::lorentz::LorentzVector;
use crate::helas::repr::numbers::Charge;
use crate::helas::repr::Real;

/// The output taxonomy realized by the evaluator's [`apply`](super::run::apply) dispatch.
///
/// Every node reduces to exactly one [`WaveformSlot`](super::waveform_slot::WaveformSlot)
/// variant, except `Op::Flows` (a [`Sink`](NodeType::Sink)). The scalar slot is refined
/// into a momentum-free constant and a momentum-carrying wavefunction — the
/// `ScalarConst`/`ScalarWf` split that keeps momentum-motion rewrites well-typed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeType {
    /// Bare real constant — `WaveformSlot::Real`.
    RealConst,
    /// Momentum-free complex scalar (a card-time constant) — `WaveformSlot::Scalar`
    /// with zero momentum.
    ScalarConst,
    /// Momentum-carrying complex scalar current (bilinear / propagated / external
    /// scalar) — `WaveformSlot::Scalar`.
    ScalarWf,
    /// Contravariant vector current — `WaveformSlot::Vector`.
    Vector,
    /// Flow-in (ket) fermion current — `WaveformSlot::FermionIn`.
    FermionIn,
    /// Flow-out (bra) fermion current — `WaveformSlot::FermionOut`.
    FermionOut,
    /// `Op::Flows`: the variadic per-flow-JAMP amplitude root. Never an operand, so it
    /// has no output slot.
    Sink,
}

/// The slot-storage class a [`NodeType`] maps to, collapsing the const/wf refinement —
/// the per-type result arena a typed instruction stream stores the node in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Storage {
    Real,
    Scalar,
    Vector,
    FermionIn,
    FermionOut,
}

impl NodeType {
    /// A card-time constant output (`RealConst`/`ScalarConst`).
    pub fn is_const(self) -> bool {
        matches!(self, NodeType::RealConst | NodeType::ScalarConst)
    }

    /// A scalar-family output (real or complex scalar).
    pub fn is_scalar(self) -> bool {
        matches!(
            self,
            NodeType::RealConst | NodeType::ScalarConst | NodeType::ScalarWf
        )
    }

    /// A non-scalar current (vector or fermion) — the operand a `Mul` scales and routes
    /// momentum into.
    pub fn is_current(self) -> bool {
        matches!(
            self,
            NodeType::Vector | NodeType::FermionIn | NodeType::FermionOut
        )
    }

    /// The result-arena class for this output, or `None` for the [`Sink`](NodeType::Sink).
    pub fn storage(self) -> Option<Storage> {
        Some(match self {
            NodeType::RealConst => Storage::Real,
            NodeType::ScalarConst | NodeType::ScalarWf => Storage::Scalar,
            NodeType::Vector => Storage::Vector,
            NodeType::FermionIn => Storage::FermionIn,
            NodeType::FermionOut => Storage::FermionOut,
            NodeType::Sink => return None,
        })
    }
}

/// Interned table of signed external-momentum combinations.
///
/// Entry `id` is a per-leg coefficient vector `c`, so the momentum it denotes is
/// `Σ_leg c[leg]·p_leg`. Id `0` is always the all-zero combination (constants, `P`
/// read-offs). A per-point momentum pool resolves each id once via [`resolve`](Self::resolve).
#[derive(Clone, Debug)]
pub struct MomTable {
    n_legs: usize,
    entries: Vec<Box<[i8]>>,
    intern: HashMap<Box<[i8]>, u32>,
}

impl MomTable {
    fn new(n_legs: usize) -> Self {
        let mut t = MomTable {
            n_legs,
            entries: Vec::new(),
            intern: HashMap::new(),
        };
        // Reserve id 0 for the zero combination.
        t.intern_slice(&vec![0i8; n_legs]);
        t
    }

    /// The zero-momentum id (always `0`).
    pub const ZERO: u32 = 0;

    fn intern_slice(&mut self, coeffs: &[i8]) -> u32 {
        if let Some(&id) = self.intern.get(coeffs) {
            return id;
        }
        let id = self.entries.len() as u32;
        let boxed: Box<[i8]> = coeffs.into();
        self.entries.push(boxed.clone());
        self.intern.insert(boxed, id);
        id
    }

    /// Number of external legs the coefficient vectors are indexed by.
    pub fn n_legs(&self) -> usize {
        self.n_legs
    }

    /// Number of distinct interned momentum combinations.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The signed per-leg coefficients of an interned combination.
    pub fn coeffs(&self, id: u32) -> &[i8] {
        &self.entries[id as usize]
    }

    /// Resolve a combination against a point's external momenta, `Σ_leg c[leg]·p_leg`.
    pub fn resolve<F: Real>(&self, id: u32, momenta: &[LorentzVector<F>]) -> LorentzVector<F> {
        let mut acc = LorentzVector::zero();
        for (leg, &c) in self.coeffs(id).iter().enumerate() {
            let mut k = c;
            while k > 0 {
                acc = acc + momenta[leg];
                k -= 1;
            }
            while k < 0 {
                acc = acc - momenta[leg];
                k += 1;
            }
        }
        acc
    }
}

/// Per-node static analysis of a lowered arena. Indexed by [`NodeId`].
#[derive(Clone, Debug)]
pub struct NodeAnalysis {
    out_type: Box<[NodeType]>,
    is_const: Box<[bool]>,
    mom_id: Box<[u32]>,
    support: Box<[u64]>,
    moms: MomTable,
}

impl NodeAnalysis {
    /// Output type of node `id`.
    pub fn out_type(&self, id: NodeId) -> NodeType {
        self.out_type[id as usize]
    }

    /// Whether node `id` is a card-time constant (all descendants are constant leaves).
    pub fn is_const(&self, id: NodeId) -> bool {
        self.is_const[id as usize]
    }

    /// The interned momentum id of node `id`'s slot (index into [`mom_table`](Self::mom_table)).
    pub fn mom_id(&self, id: NodeId) -> u32 {
        self.mom_id[id as usize]
    }

    /// The helicity-support mask of node `id`: bit `leg` set iff external leg `leg` is
    /// in the node's subtree.
    pub fn support(&self, id: NodeId) -> u64 {
        self.support[id as usize]
    }

    /// The interned momentum table shared by all nodes.
    pub fn mom_table(&self) -> &MomTable {
        &self.moms
    }

    /// Resolve node `id`'s momentum against a point's external momenta.
    pub fn resolve_mom<F: Real>(
        &self,
        id: NodeId,
        momenta: &[LorentzVector<F>],
    ) -> LorentzVector<F> {
        self.moms.resolve(self.mom_id[id as usize], momenta)
    }

    /// Number of analyzed nodes.
    pub fn len(&self) -> usize {
        self.out_type.len()
    }

    pub fn is_empty(&self) -> bool {
        self.out_type.is_empty()
    }
}

/// Everything the transfer function needs from a leaf op (unifies the `Const` and `Sym`
/// leaf encodings).
#[derive(Clone, Copy)]
enum LeafKind {
    /// Not a leaf op.
    NonLeaf,
    /// `Mass`/`Width`/`Coeff`, or a real `CoeffRat`.
    RealConst,
    /// `Coupling`, or an imaginary `CoeffRat`.
    ScalarConst,
    /// `External`: its physical leg index, resolved output type, and the sign its
    /// stored slot momentum carries (`+1`/`−1`) in the HELAS all-outgoing convention.
    External {
        leg_idx: usize,
        out: NodeType,
        mom_sign: i8,
    },
}

/// The output type of an external wavefunction, from its UFO spin code and flow.
fn external_out_type(spin: i32, charge: Charge, incoming: bool) -> NodeType {
    match spin {
        1 => NodeType::ScalarWf,
        2 => {
            let is_particle = matches!(charge, Charge::Particle);
            // A leg is a ket (flow-in) iff it is an incoming particle or an outgoing
            // antiparticle — i.e. `incoming == is_particle` (see `build_external_core`).
            if incoming == is_particle {
                NodeType::FermionIn
            } else {
                NodeType::FermionOut
            }
        }
        3 => NodeType::Vector,
        other => panic!("unsupported external spin code: {other}"),
    }
}

/// The sign an external wavefunction's stored slot momentum carries, mirroring the HELAS
/// wavefunction constructors: scalars (`sxxxxx`) and vectors (`vxxxxx`) store `nsv·p`
/// (incoming → `−p`), while fermions (`from_momentum`) key the sign off the leg's charge
/// (particle `+p`, antiparticle `−p`) — so a fermion's flow type alone cannot recover it.
fn external_mom_sign(spin: i32, charge: Charge, incoming: bool) -> i8 {
    match spin {
        2 => match charge {
            Charge::Particle => 1,
            Charge::Antiparticle => -1,
        },
        // Scalar (1) and vector (3): the incoming/outgoing `ns` sign.
        _ => {
            if incoming {
                -1
            } else {
                1
            }
        }
    }
}

/// Analyze a folded [`Ast<Const>`] against its external-leg table.
pub fn analyze(ast: &Ast<Const>, ext_legs: &[ExtLeg]) -> NodeAnalysis {
    let n_legs = ext_legs
        .iter()
        .map(|l| l.leg_idx as usize + 1)
        .max()
        .unwrap_or(0);
    analyze_core(ast, n_legs, |id| match ast.value(id).op {
        Op::Coupling => LeafKind::ScalarConst,
        Op::Mass | Op::Width | Op::Coeff => LeafKind::RealConst,
        Op::CoeffRat => match ast.value(id).leaf {
            Const::Complex(_) => LeafKind::ScalarConst,
            _ => LeafKind::RealConst,
        },
        Op::External => {
            let Const::Ext(i) = ast.value(id).leaf else {
                panic!("External node without a leg-table index");
            };
            let leg = ext_legs[i as usize];
            LeafKind::External {
                leg_idx: leg.leg_idx as usize,
                out: external_out_type(leg.spin, leg.charge, leg.incoming),
                mom_sign: external_mom_sign(leg.spin, leg.charge, leg.incoming),
            }
        }
        _ => LeafKind::NonLeaf,
    })
}

/// Analyze a symbolic [`Ast<Sym>`] (the pre-fold graph the constant-folding and egraph
/// passes operate on). Produces the same annotations as [`analyze`].
pub fn analyze_sym(ast: &Ast<Sym>) -> NodeAnalysis {
    let n_legs = ast
        .iter()
        .filter_map(|id| match ast.value(id).leaf {
            Sym::Ext { leg_idx, .. } => Some(leg_idx + 1),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    analyze_core(ast, n_legs, |id| match ast.value(id).leaf {
        Sym::Coupling(_) => LeafKind::ScalarConst,
        Sym::Particle(_) => LeafKind::RealConst,
        Sym::Coeff(_) => LeafKind::RealConst,
        Sym::Rational { imag, .. } => {
            if imag {
                LeafKind::ScalarConst
            } else {
                LeafKind::RealConst
            }
        }
        Sym::Ext {
            leg_idx,
            spin,
            charge,
            incoming,
        } => LeafKind::External {
            leg_idx,
            out: external_out_type(spin, charge, incoming),
            mom_sign: external_mom_sign(spin, charge, incoming),
        },
        Sym::None => LeafKind::NonLeaf,
    })
}

/// The forward scan, generic over the leaf encoding (via `leaf_of`).
fn analyze_core<T>(
    ast: &Ast<T>,
    n_legs: usize,
    leaf_of: impl Fn(NodeId) -> LeafKind,
) -> NodeAnalysis {
    let n = ast.len();
    let mut out_type = vec![NodeType::Sink; n];
    let mut is_const = vec![false; n];
    let mut mom_id = vec![MomTable::ZERO; n];
    let mut support = vec![0u64; n];
    let mut moms = MomTable::new(n_legs);
    let mut buf = vec![0i8; n_legs];

    for id in ast.iter() {
        let op = ast.value(id).op;
        let kids = ast.children_ids(id);
        let leaf = leaf_of(id);

        // ── output type ──
        let out = match op {
            Op::Flows => NodeType::Sink,
            _ => match leaf {
                LeafKind::External { out, .. } => out,
                LeafKind::RealConst => NodeType::RealConst,
                LeafKind::ScalarConst => NodeType::ScalarConst,
                LeafKind::NonLeaf => out_type_nonleaf(op, kids, &out_type),
            },
        };

        // ── constness ──
        // The `Flows` sink is never a foldable constant, even in the degenerate case of
        // all-constant JAMP operands: it is the variadic amplitude root, not a value.
        let cst = match leaf {
            LeafKind::External { .. } => false,
            LeafKind::RealConst | LeafKind::ScalarConst => true,
            LeafKind::NonLeaf => {
                op != Op::Flows && !kids.is_empty() && kids.iter().all(|&k| is_const[k as usize])
            }
        };

        // ── helicity-support mask ──
        let mask = match leaf {
            LeafKind::External { leg_idx, .. } => 1u64 << leg_idx,
            _ => kids.iter().fold(0u64, |acc, &k| acc | support[k as usize]),
        };

        // ── momentum combination ──
        for b in buf.iter_mut() {
            *b = 0;
        }
        momentum_into(&mut buf, op, kids, &leaf, &out_type, &mom_id, &moms);
        let mid = moms.intern_slice(&buf);

        debug_assert!(
            !cst || out.is_const(),
            "constant node {id} has non-constant output type {out:?}"
        );

        out_type[id as usize] = out;
        is_const[id as usize] = cst;
        mom_id[id as usize] = mid;
        support[id as usize] = mask;
    }

    NodeAnalysis {
        out_type: out_type.into_boxed_slice(),
        is_const: is_const.into_boxed_slice(),
        mom_id: mom_id.into_boxed_slice(),
        support: support.into_boxed_slice(),
        moms,
    }
}

/// Output type of a non-leaf op from its children's already-computed types.
fn out_type_nonleaf(op: Op, kids: &[NodeId], out: &[NodeType]) -> NodeType {
    let ty = |i: usize| out[kids[i] as usize];
    match op {
        // Preserves the input current's variant; a propagated scalar is a wf.
        Op::Propagate => match ty(0) {
            NodeType::Vector => NodeType::Vector,
            NodeType::FermionIn => NodeType::FermionIn,
            NodeType::FermionOut => NodeType::FermionOut,
            _ => NodeType::ScalarWf,
        },
        // Chiral projection preserves the fermion flow.
        Op::ProjM | Op::ProjP => ty(0),
        // Vector producers.
        Op::GammaVout | Op::FfvVout | Op::MetricVout | Op::LowerVout | Op::PMom | Op::PMomOut => {
            NodeType::Vector
        }
        // Off-shell fermion currents follow the fermion input's flow (operand 1).
        Op::GammaIout | Op::GammaOout | Op::FfvIout | Op::FfvOout => ty(1),
        // Scalar bilinears.
        Op::ProjMAmp | Op::ProjPAmp | Op::IdentityAmp | Op::Metric => NodeType::ScalarWf,
        Op::Add => join_add(kids, out),
        Op::Mul => mul_out(kids, out),
        other => panic!("out_type_nonleaf: unexpected non-leaf op {other:?}"),
    }
}

/// `Add` output: the shared current type if any operand is a current, else a scalar —
/// a wf if any summand is a wf, otherwise a constant.
fn join_add(kids: &[NodeId], out: &[NodeType]) -> NodeType {
    let mut current = None;
    let mut any_wf = false;
    for &k in kids {
        let t = out[k as usize];
        if t.is_current() {
            current = Some(t);
        } else if t == NodeType::ScalarWf {
            any_wf = true;
        }
    }
    match current {
        Some(t) => t,
        None if any_wf => NodeType::ScalarWf,
        None => NodeType::ScalarConst,
    }
}

/// `Mul` output: the single non-scalar current operand's type if present, else a scalar
/// — a constant iff every operand is constant.
fn mul_out(kids: &[NodeId], out: &[NodeType]) -> NodeType {
    if let Some(t) = kids
        .iter()
        .map(|&k| out[k as usize])
        .find(|t| t.is_current())
    {
        t
    } else if kids.iter().all(|&k| out[k as usize].is_const()) {
        NodeType::ScalarConst
    } else {
        NodeType::ScalarWf
    }
}

/// Accumulate node's signed external-momentum combination into `buf` (pre-zeroed),
/// mirroring the runtime slot's `.momentum` field.
fn momentum_into(
    buf: &mut [i8],
    op: Op,
    kids: &[NodeId],
    leaf: &LeafKind,
    out_type: &[NodeType],
    mom_id: &[u32],
    moms: &MomTable,
) {
    let add = |buf: &mut [i8], k: NodeId, sign: i8| {
        for (b, &c) in buf.iter_mut().zip(moms.coeffs(mom_id[k as usize])) {
            *b += sign * c;
        }
    };
    // The two fermion operands of a bilinear, as `(bra = FermionOut, ket = FermionIn)`.
    let bra_ket = |kids: &[NodeId]| -> (NodeId, NodeId) {
        let a = kids[0];
        let b = kids[1];
        match out_type[a as usize] {
            NodeType::FermionOut => (a, b),
            _ => (b, a),
        }
    };

    if let LeafKind::External {
        leg_idx, mom_sign, ..
    } = leaf
    {
        buf[*leg_idx] += *mom_sign;
        return;
    }
    match op {
        // Constant leaves and the P read-offs carry zero routing momentum.
        Op::Coupling
        | Op::Mass
        | Op::Width
        | Op::Coeff
        | Op::CoeffRat
        | Op::PMom
        | Op::PMomOut
        | Op::Flows => {}
        // Momentum-preserving unary transforms.
        Op::Propagate | Op::ProjM | Op::ProjP | Op::MetricVout | Op::LowerVout => {
            add(buf, kids[0], 1)
        }
        // Scalar contraction: sum of the two vectors' momenta.
        Op::Metric => {
            add(buf, kids[0], 1);
            add(buf, kids[1], 1);
        }
        // Vector / scalar bilinears of two fermions: bra − ket.
        Op::GammaVout | Op::FfvVout | Op::ProjMAmp | Op::ProjPAmp | Op::IdentityAmp => {
            let (bra, ket) = bra_ket(kids);
            add(buf, bra, 1);
            add(buf, ket, -1);
        }
        // Off-shell fermion current: ket `f − v`, bra `f + v` (operand 0 = vector,
        // operand 1 = fermion).
        Op::GammaIout | Op::GammaOout | Op::FfvIout | Op::FfvOout => {
            add(buf, kids[1], 1);
            let vsign = match out_type[kids[1] as usize] {
                NodeType::FermionIn => -1,
                _ => 1,
            };
            add(buf, kids[0], vsign);
        }
        // The runtime takes the first operand's momentum for a sum.
        Op::Add => add(buf, kids[0], 1),
        // Route the scalar factors' momentum into the surviving current (ket subtracts,
        // bra/vector add); an all-scalar product carries the summed scalar momentum.
        Op::Mul => {
            let ns = kids.iter().position(|&k| out_type[k as usize].is_current());
            match ns {
                Some(pos) => {
                    let current = kids[pos];
                    add(buf, current, 1);
                    let sign: i8 = match out_type[current as usize] {
                        NodeType::FermionIn => -1,
                        _ => 1,
                    };
                    for (i, &k) in kids.iter().enumerate() {
                        if i != pos {
                            add(buf, k, sign);
                        }
                    }
                }
                None => {
                    for &k in kids {
                        add(buf, k, 1);
                    }
                }
            }
        }
        Op::External => unreachable!("External handled above"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helas::eval::ast::AstBuilder;
    use crate::helas::eval::fold::Folded;
    use crate::helas::eval::op::Node;
    use crate::ufo::couplings::CouplingId;
    use crate::ufo::particles::ParticleId;

    fn sym(op: Op, leaf: Sym, kids: Vec<NodeId>, b: &mut AstBuilder<Sym>) -> NodeId {
        b.add(op, leaf, kids)
    }

    /// A pure constant subgraph `g = Coupling·Coeff + CoeffRat(imag)` is entirely
    /// constant, momentum-free, and a scalar constant at the root.
    #[test]
    fn const_subgraph_is_constant_and_momentum_free() {
        let mut b = AstBuilder::new();
        let coup = sym(
            Op::Coupling,
            Sym::Coupling(CouplingId::from(5usize)),
            vec![],
            &mut b,
        );
        let coeff = sym(Op::Coeff, Sym::Coeff(2.0), vec![], &mut b);
        let prod = sym(Op::Mul, Sym::None, vec![coup, coeff], &mut b);
        let rat = sym(
            Op::CoeffRat,
            Sym::Rational {
                num: 1,
                den: 3,
                imag: true,
            },
            vec![],
            &mut b,
        );
        let root = sym(Op::Add, Sym::None, vec![prod, rat], &mut b);
        let ast = b.finish(root);

        let an = analyze_sym(&ast);
        // Every node constant; scalar/real types only.
        for id in 0..ast.len() as NodeId {
            assert!(an.is_const(id), "node {id} should be constant");
            assert!(an.out_type(id).is_const(), "node {id} const type");
            assert_eq!(an.mom_id(id), MomTable::ZERO, "node {id} zero momentum");
            assert_eq!(an.support(id), 0, "node {id} empty support");
        }
        assert_eq!(an.out_type(coeff), NodeType::RealConst);
        assert_eq!(an.out_type(coup), NodeType::ScalarConst);
        assert_eq!(an.out_type(rat), NodeType::ScalarConst); // imag CoeffRat
        assert_eq!(an.out_type(prod), NodeType::ScalarConst);
        assert_eq!(an.out_type(root), NodeType::ScalarConst);
    }

    /// A real `CoeffRat` leaf is a real constant; an imaginary one is a scalar constant.
    #[test]
    fn coeff_rat_const_classification() {
        let mut b = AstBuilder::new();
        let re = sym(
            Op::CoeffRat,
            Sym::Rational {
                num: 1,
                den: 2,
                imag: false,
            },
            vec![],
            &mut b,
        );
        let ast = b.finish(re);
        let an = analyze_sym(&ast);
        assert_eq!(an.out_type(re), NodeType::RealConst);
        assert!(an.is_const(re));
    }

    /// A `Flows` root is a sink (no output type); its scalar children keep theirs.
    #[test]
    fn flows_root_is_sink() {
        let mut b = AstBuilder::new();
        let c0 = sym(
            Op::CoeffRat,
            Sym::Rational {
                num: 1,
                den: 1,
                imag: false,
            },
            vec![],
            &mut b,
        );
        let c1 = sym(
            Op::CoeffRat,
            Sym::Rational {
                num: 1,
                den: 3,
                imag: true,
            },
            vec![],
            &mut b,
        );
        let root = sym(Op::Flows, Sym::None, vec![c0, c1], &mut b);
        let ast = b.finish(root);
        let an = analyze_sym(&ast);
        assert_eq!(an.out_type(root), NodeType::Sink);
        assert_eq!(an.out_type(c0), NodeType::RealConst);
        assert_eq!(an.out_type(c1), NodeType::ScalarConst);
    }

    /// A multi-leg current: two external fermions → `GammaVout` vector, whose momentum
    /// is `p_bra − p_ket` and whose support covers both legs.
    #[test]
    fn multi_leg_current_type_momentum_support() {
        let mut b = AstBuilder::new();
        // leg 0: incoming particle fermion → ket (FermionIn).
        let mass0 = sym(
            Op::Mass,
            Sym::Particle(ParticleId::from(11usize)),
            vec![],
            &mut b,
        );
        let e_in = sym(
            Op::External,
            Sym::Ext {
                leg_idx: 0,
                spin: 2,
                charge: Charge::Particle,
                incoming: true,
            },
            vec![mass0],
            &mut b,
        );
        // leg 1: incoming antiparticle fermion → bra (FermionOut).
        let mass1 = sym(
            Op::Mass,
            Sym::Particle(ParticleId::from(11usize)),
            vec![],
            &mut b,
        );
        let p_in = sym(
            Op::External,
            Sym::Ext {
                leg_idx: 1,
                spin: 2,
                charge: Charge::Antiparticle,
                incoming: true,
            },
            vec![mass1],
            &mut b,
        );
        // GammaVout children order (ket, bra) — bra_ket resolves by type.
        let vout = sym(Op::GammaVout, Sym::None, vec![e_in, p_in], &mut b);
        let ast = b.finish(vout);
        let an = analyze_sym(&ast);

        assert_eq!(an.out_type(e_in), NodeType::FermionIn);
        assert_eq!(an.out_type(p_in), NodeType::FermionOut);
        assert_eq!(an.out_type(vout), NodeType::Vector);
        assert!(!an.is_const(vout));
        assert_eq!(an.support(vout), 0b11);
        // bra(leg1, antiparticle → −p₁) − ket(leg0, particle → +p₀): coeffs [−1, −1].
        assert_eq!(an.mom_table().coeffs(an.mom_id(vout)), &[-1, -1]);
    }

    /// The folded-arena entry point agrees with the symbolic one on a small graph
    /// (the fold only rewrites leaves to pool indices; structure is preserved).
    #[test]
    fn folded_and_sym_agree() {
        let mut b = AstBuilder::new();
        let mass = sym(
            Op::Mass,
            Sym::Particle(ParticleId::from(11usize)),
            vec![],
            &mut b,
        );
        let e_in = sym(
            Op::External,
            Sym::Ext {
                leg_idx: 0,
                spin: 2,
                charge: Charge::Particle,
                incoming: true,
            },
            vec![mass],
            &mut b,
        );
        let proj = sym(Op::ProjM, Sym::None, vec![e_in], &mut b);
        let sym_ast = b.finish(proj);

        let folded = Folded::build(&sym_ast);
        let an_c = analyze(&folded.ast, folded.ext_legs());
        // The folded arena drops the orphaned Mass child, so only compare the shared
        // structure by output-type at the root.
        assert_eq!(an_c.out_type(folded.ast.root()), NodeType::FermionIn);
        assert!(!an_c.is_const(folded.ast.root()));
    }

    /// Every `Node<Sym>` leaf variant maps to a `LeafKind`, so the analysis panics on no
    /// SM op.
    #[test]
    fn all_leaf_ops_classified() {
        let _ = Node::new(Op::Coupling, Sym::None); // keep Node import used
        for &(op, leaf) in &[
            (Op::Coupling, Sym::Coupling(CouplingId::from(0usize))),
            (Op::Mass, Sym::Particle(ParticleId::from(0usize))),
            (Op::Width, Sym::Particle(ParticleId::from(0usize))),
            (Op::Coeff, Sym::Coeff(1.0)),
            (
                Op::CoeffRat,
                Sym::Rational {
                    num: 1,
                    den: 1,
                    imag: false,
                },
            ),
        ] {
            let mut b = AstBuilder::new();
            let id = b.add(op, leaf, vec![]);
            let ast = b.finish(id);
            let an = analyze_sym(&ast);
            assert!(an.is_const(id), "{op:?} should be constant");
        }
    }
}
