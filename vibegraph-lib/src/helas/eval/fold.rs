//! Pass 3b: constant-fold an [`Ast<Sym>`] into a folded [`Ast<Const>`] plus two deduped
//! constant pools.
//!
//! The folded structure (op tags, children, leaf *kinds*+indices) is independent of the
//! parameter card and the scalar field `F`, so it is built once ([`Folded::build`]). The
//! numeric pools are rebuilt cheaply per `(EvaluatedModel, F)` ([`Folded::pools`]):
//! `consts_c` (complex couplings) and `consts_f` (real masses/widths/coeffs) are kept
//! separate so real chains multiply in `F`.

use std::collections::HashMap;

use num_complex::Complex64;
use num_traits::FromPrimitive;

use super::analysis::{self, NodeAnalysis, NodeType};
use super::ast::{Ast, AstBuilder};
use super::layout::Program;
use super::op::{Const, NodeId, Op, Sym};
use super::run::{apply, EvalEnv};
use super::tree::Tree;
use super::waveform_slot::WaveformSlot;
use crate::helas::repr::numbers::Charge;
use crate::helas::repr::{Real, C};
use crate::ufo::couplings::CouplingId;
use crate::ufo::particles::ParticleId;
use crate::ufo::EvaluatedModel;

/// A request for one entry of the real pool. `Coeff` stores the `f64` bit pattern so the
/// request is hashable/dedupable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum RealReq {
    Mass(ParticleId),
    Width(ParticleId),
    Coeff(u64),
    /// `Op::CoeffRat` with `imag == false`: resolves to `num/den`.
    Rat(i64, i64),
}

/// A request for one entry of the complex pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ComplexReq {
    Coupling(CouplingId),
    /// `Op::CoeffRat` with `imag == true`: resolves to `i·num/den`.
    Rat(i64, i64),
}

/// One entry of the folded external-leg table: everything an `Op::External` node
/// needs to build its wavefunction, resolved from the symbolic leaf and its `Mass`
/// child so the folded node is a bare `Const::Ext(u32)` with no children.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExtLeg {
    /// Index into the process's external momenta/helicities.
    pub leg_idx: u32,
    /// UFO spin code (2s+1).
    pub spin: i32,
    pub charge: Charge,
    /// Whether this leg is an incoming external (see [`Sym::Ext`]).
    pub incoming: bool,
    /// The leg's mass: index into `consts_f`.
    pub mass: u32,
}

/// The folded, card-independent skeleton plus the pool specifications that resolve it.
#[derive(Debug, Clone)]
pub struct Folded {
    /// Same structure as the symbolic AST, with leaves rewritten to pool indices. The
    /// canonical folded arena the analysis and typed instruction stream are derived from;
    /// the runtime executes the derived [`Program`], so this is otherwise read only by the
    /// structural tests and op-coverage checks.
    #[cfg_attr(not(test), allow(dead_code))]
    pub ast: Ast<Const>,
    /// `consts_c[i] = resolve(pool_c[i])`.
    pool_c: Vec<ComplexReq>,
    /// `consts_f[j] = resolve(pool_f[j])`.
    pool_f: Vec<RealReq>,
    /// `Const::Ext(k)` resolves to `pool_ext[k]`.
    pool_ext: Vec<ExtLeg>,
    /// Constant sub-graph arena: every card-time-constant node reachable from a folded
    /// composite. Its leaves read the base `consts_c`/`consts_f`; a single forward pass
    /// in [`pools`](Self::pools) resolves the folded-composite values once per card.
    /// Empty when the amplitude has no foldable constant composite.
    const_ast: Ast<Const>,
    /// [`const_ast`](Self::const_ast) node ids whose evaluated complex value appends to
    /// `consts_c` after the base entries, in the order the folded leaves reference them
    /// (`consts_c[pool_c.len() + k]` is entry `fold_complex[k]`).
    fold_complex: Box<[NodeId]>,
    /// [`const_ast`](Self::const_ast) node ids whose evaluated real value appends to
    /// `consts_f` after the base entries.
    fold_real: Box<[NodeId]>,
    /// Static per-node annotations of `ast` (output type, constness, momentum id,
    /// helicity support), computed once from the card-independent skeleton.
    analysis: NodeAnalysis,
    /// The typed instruction stream the runtime executes against per-class result
    /// arenas, lowered once from `ast` + `analysis`.
    program: Program,
}

impl Folded {
    /// Build the folded skeleton from the symbolic AST, deduping constants into the
    /// pool specs.
    ///
    /// An `External`'s symbolic `Mass` child is absorbed into its [`ExtLeg`] table
    /// entry (as a `consts_f` index), so the folded node is a childless leaf; the
    /// rebuild keeps only nodes still reachable from the root, dropping the orphaned
    /// `Mass` nodes from the arena.
    pub fn build(sym: &Ast<Sym>) -> Folded {
        let mut pool_c: Vec<ComplexReq> = Vec::new();
        let mut c_index: HashMap<ComplexReq, u32> = HashMap::new();
        let mut pool_f: Vec<RealReq> = Vec::new();
        let mut f_index: HashMap<RealReq, u32> = HashMap::new();
        let mut pool_ext: Vec<ExtLeg> = Vec::new();
        let mut ext_index: HashMap<ExtLeg, u32> = HashMap::new();

        let mut intern_c = |req: ComplexReq| -> u32 {
            *c_index.entry(req).or_insert_with(|| {
                pool_c.push(req);
                (pool_c.len() - 1) as u32
            })
        };
        let mut intern_f = |req: RealReq| -> u32 {
            *f_index.entry(req).or_insert_with(|| {
                pool_f.push(req);
                (pool_f.len() - 1) as u32
            })
        };

        // Reachability from the root, not descending into `External` (its Mass child
        // moves into the leg table and would otherwise linger as an orphan node).
        let mut reachable = vec![false; sym.len()];
        let mut stack = vec![sym.root()];
        while let Some(n) = stack.pop() {
            if std::mem::replace(&mut reachable[n as usize], true) {
                continue;
            }
            if sym.value(n).op != Op::External {
                stack.extend_from_slice(sym.children_ids(n));
            }
        }

        let mut builder = AstBuilder::new();
        // remap[old id] = new id; only meaningful for reachable nodes.
        let mut remap = vec![u32::MAX; sym.len()];
        for id in sym.iter() {
            if !reachable[id as usize] {
                continue;
            }
            let node = sym.value(id);
            let (leaf, children) = match (node.op, node.leaf) {
                (Op::Coupling, Sym::Coupling(cid)) => {
                    (Const::complex(intern_c(ComplexReq::Coupling(cid))), vec![])
                }
                (Op::Mass, Sym::Particle(pid)) => {
                    (Const::real(intern_f(RealReq::Mass(pid))), vec![])
                }
                (Op::Width, Sym::Particle(pid)) => {
                    (Const::real(intern_f(RealReq::Width(pid))), vec![])
                }
                (Op::Coeff, Sym::Coeff(c)) => {
                    (Const::real(intern_f(RealReq::Coeff(c.to_bits()))), vec![])
                }
                (Op::CoeffRat, Sym::Rational { num, den, imag }) => {
                    if imag {
                        (Const::complex(intern_c(ComplexReq::Rat(num, den))), vec![])
                    } else {
                        (Const::real(intern_f(RealReq::Rat(num, den))), vec![])
                    }
                }
                (
                    Op::External,
                    Sym::Ext {
                        leg_idx,
                        spin,
                        charge,
                        incoming,
                    },
                ) => {
                    let mass_child = sym.children_ids(id)[0];
                    let mass_node = sym.value(mass_child);
                    let (Op::Mass, Sym::Particle(pid)) = (mass_node.op, mass_node.leaf) else {
                        panic!("External's child must be a Mass leaf, got {mass_node:?}");
                    };
                    let leg = ExtLeg {
                        leg_idx: leg_idx as u32,
                        spin,
                        charge,
                        incoming,
                        mass: intern_f(RealReq::Mass(pid)),
                    };
                    let k = *ext_index.entry(leg).or_insert_with(|| {
                        pool_ext.push(leg);
                        (pool_ext.len() - 1) as u32
                    });
                    (Const::ext(k), vec![])
                }
                _ => (
                    Const::NONE,
                    sym.children_ids(id)
                        .iter()
                        .map(|&c| remap[c as usize])
                        .collect(),
                ),
            };
            remap[id as usize] = builder.add(node.op, leaf, children);
        }

        let ast0 = builder.finish(remap[sym.root() as usize]);
        let an0 = analysis::analyze(&ast0, &pool_ext);

        // Collapse every maximal constant composite (a `Mul`/`Add` subgraph of
        // card-time constants) into a single pool-read leaf, so it is resolved once
        // per parameter card rather than re-evaluated at every phase-space point.
        let folded = fold_constant_subgraphs(&ast0, &an0, pool_c.len() as u32, pool_f.len() as u32);
        let FoldRewrite {
            ast,
            const_ast,
            fold_complex,
            fold_real,
        } = folded;

        let analysis = analysis::analyze(&ast, &pool_ext);
        let program = Program::build(&ast, &analysis);
        Folded {
            ast,
            pool_c,
            pool_f,
            pool_ext,
            const_ast,
            fold_complex,
            fold_real,
            analysis,
            program,
        }
    }

    /// The static per-node analysis of the folded arena.
    pub(super) fn analysis(&self) -> &NodeAnalysis {
        &self.analysis
    }

    /// The typed instruction stream lowered from the folded arena.
    pub(super) fn program(&self) -> &Program {
        &self.program
    }

    /// Resolve the two numeric pools for a parameter card at scalar precision `F`.
    pub fn pools<F: Real + FromPrimitive>(
        &self,
        evaluated: &EvaluatedModel,
    ) -> (Box<[C<F>]>, Box<[F]>) {
        let mut consts_c: Vec<C<F>> = self
            .pool_c
            .iter()
            .map(|req| match *req {
                ComplexReq::Coupling(id) => cplx::<F>(evaluated.coupling(id)),
                ComplexReq::Rat(num, den) => C::new(F::ZERO, ratio::<F>(num, den)),
            })
            .collect();
        let mut consts_f: Vec<F> = self
            .pool_f
            .iter()
            .map(|req| match *req {
                RealReq::Mass(id) => real::<F>(evaluated.mass(id)),
                RealReq::Width(id) => real::<F>(evaluated.width(id)),
                RealReq::Coeff(bits) => real::<F>(f64::from_bits(bits)),
                RealReq::Rat(num, den) => ratio::<F>(num, den),
            })
            .collect();

        // Resolve the folded constant composites: one forward pass over `const_ast`
        // (its leaves reading the base pools just filled) reproduces exactly the
        // per-point reduction the collapsed subgraphs used to perform, so the appended
        // pool entries are bit-for-bit what the runtime computed inline.
        if !self.fold_complex.is_empty() || !self.fold_real.is_empty() {
            let slots = eval_const_subgraph(&self.const_ast, &consts_c, &consts_f);
            let cvals: Vec<C<F>> = self
                .fold_complex
                .iter()
                .map(|&id| scalar_value(&slots[id as usize]))
                .collect();
            let rvals: Vec<F> = self
                .fold_real
                .iter()
                .map(|&id| real_value(&slots[id as usize]))
                .collect();
            consts_c.extend(cvals);
            consts_f.extend(rvals);
        }

        (consts_c.into_boxed_slice(), consts_f.into_boxed_slice())
    }

    /// The external-leg table resolving `Const::Ext` indices.
    pub fn ext_legs(&self) -> &[ExtLeg] {
        &self.pool_ext
    }

    /// Coupling ids referenced by the amplitude (from the complex pool spec).
    pub fn coupling_ids(&self) -> impl Iterator<Item = CouplingId> + '_ {
        self.pool_c.iter().filter_map(|req| match req {
            ComplexReq::Coupling(id) => Some(*id),
            ComplexReq::Rat(..) => None,
        })
    }

    /// Particle ids referenced by the amplitude (mass/width entries of the real pool).
    pub fn particle_ids(&self) -> impl Iterator<Item = ParticleId> + '_ {
        self.pool_f.iter().filter_map(|req| match req {
            RealReq::Mass(id) | RealReq::Width(id) => Some(*id),
            RealReq::Coeff(_) | RealReq::Rat(..) => None,
        })
    }
}

/// The main folded arena with its constant composites collapsed, plus the constant
/// sub-graph arena and the per-pool lists of its fold-root ids that resolve the leaves.
struct FoldRewrite {
    ast: Ast<Const>,
    const_ast: Ast<Const>,
    fold_complex: Box<[NodeId]>,
    fold_real: Box<[NodeId]>,
}

/// Whether an op is a bare constant-pool leaf (already a single pool read), as opposed
/// to a constant *composite* (`Mul`/`Add` of constants) worth collapsing.
fn is_const_leaf_op(op: Op) -> bool {
    matches!(
        op,
        Op::Coupling | Op::Mass | Op::Width | Op::Coeff | Op::CoeffRat
    )
}

/// Rewrite `ast0` so every maximal constant composite becomes one pool-read leaf
/// (`Op::CoeffRat` over a `Const::Complex`/`Const::Real` index past the base pool).
///
/// A *fold root* is a constant composite consumed by at least one non-constant node:
/// the runtime reads it, but its value is card-time-fixed. Constants used only by other
/// constants are absorbed into the enclosing fold and dropped from the main arena. The
/// collapsed subgraphs are copied into `const_ast` (reachable from the fold roots), whose
/// one bind-time forward pass resolves the appended pool entries.
fn fold_constant_subgraphs(
    ast0: &Ast<Const>,
    an0: &NodeAnalysis,
    base_c: u32,
    base_f: u32,
) -> FoldRewrite {
    let n = ast0.len();

    // Top-down from the root through non-constant edges: mark every node kept in the
    // main arena, and every constant composite reached from a non-constant parent as a
    // fold root (a leaf in the rewritten arena; not descended into).
    let mut kept = vec![false; n];
    let mut fold_root = vec![false; n];
    let root = ast0.root();
    kept[root as usize] = true;
    let mut stack = vec![root];
    while let Some(nid) = stack.pop() {
        for &c in ast0.children_ids(nid) {
            if kept[c as usize] {
                continue;
            }
            kept[c as usize] = true;
            if an0.is_const(c) {
                // A bare constant leaf stays a pool read; a constant composite folds.
                if !is_const_leaf_op(ast0.value(c).op) {
                    fold_root[c as usize] = true;
                }
            } else {
                stack.push(c);
            }
        }
    }

    // Constant sub-graph reachable from the fold roots (all descendants are constant).
    let mut need = vec![false; n];
    let mut cstack: Vec<NodeId> = Vec::new();
    for id in ast0.iter() {
        if fold_root[id as usize] {
            need[id as usize] = true;
            cstack.push(id);
        }
    }
    while let Some(nid) = cstack.pop() {
        for &c in ast0.children_ids(nid) {
            if !need[c as usize] {
                need[c as usize] = true;
                cstack.push(c);
            }
        }
    }

    // Copy the needed constant nodes into `const_ast`, preserving arena order (children
    // before parents) so a forward pass evaluates them directly.
    let mut cbuilder = AstBuilder::new();
    let mut const_remap = vec![u32::MAX; n];
    let mut last_const = 0u32;
    for old in ast0.iter() {
        if !need[old as usize] {
            continue;
        }
        let node = ast0.value(old);
        let kids: Vec<NodeId> = ast0
            .children_ids(old)
            .iter()
            .map(|&c| const_remap[c as usize])
            .collect();
        let new = cbuilder.add(node.op, node.leaf, kids);
        const_remap[old as usize] = new;
        last_const = new;
    }
    let const_ast = cbuilder.finish(last_const);

    // Rebuild the main arena: fold roots become pool-read leaves (indexed past the base
    // pool, in fold-list order); every other kept node is copied with remapped children.
    let mut mbuilder = AstBuilder::new();
    let mut main_remap = vec![u32::MAX; n];
    let mut fold_complex: Vec<NodeId> = Vec::new();
    let mut fold_real: Vec<NodeId> = Vec::new();
    for old in ast0.iter() {
        if !kept[old as usize] {
            continue;
        }
        let new = if fold_root[old as usize] {
            let cid = const_remap[old as usize];
            let leaf = match an0.out_type(old) {
                NodeType::ScalarConst => {
                    let idx = base_c + fold_complex.len() as u32;
                    fold_complex.push(cid);
                    Const::complex(idx)
                }
                NodeType::RealConst => {
                    let idx = base_f + fold_real.len() as u32;
                    fold_real.push(cid);
                    Const::real(idx)
                }
                other => panic!("fold root {old} has non-constant output type {other:?}"),
            };
            mbuilder.add(Op::CoeffRat, leaf, vec![])
        } else {
            let node = ast0.value(old);
            let kids: Vec<NodeId> = ast0
                .children_ids(old)
                .iter()
                .map(|&c| main_remap[c as usize])
                .collect();
            mbuilder.add(node.op, node.leaf, kids)
        };
        main_remap[old as usize] = new;
    }
    let ast = mbuilder.finish(main_remap[root as usize]);

    FoldRewrite {
        ast,
        const_ast,
        fold_complex: fold_complex.into_boxed_slice(),
        fold_real: fold_real.into_boxed_slice(),
    }
}

/// Evaluate the constant sub-graph in one forward pass through the runtime [`apply`]
/// reduction (with no kinematics — constant ops read only the pools), so the resolved
/// values match the inline per-point evaluation bit-for-bit.
fn eval_const_subgraph<F: Real + FromPrimitive>(
    const_ast: &Ast<Const>,
    consts_c: &[C<F>],
    consts_f: &[F],
) -> Vec<WaveformSlot<F>> {
    let env = EvalEnv {
        consts_c,
        consts_f,
        ext_legs: &[],
        momenta: &[],
        helicities: &[],
        ward_leg: None,
    };
    let mut res: Vec<WaveformSlot<F>> = Vec::with_capacity(const_ast.len());
    for id in const_ast.iter() {
        let ids = const_ast.children_ids(id);
        let value = apply(
            const_ast.value(id),
            ids.len(),
            |i| &res[ids[i] as usize],
            &env,
        );
        res.push(value);
    }
    res
}

/// The complex value of a folded scalar-constant subgraph.
fn scalar_value<F: Real>(slot: &WaveformSlot<F>) -> C<F> {
    match slot {
        WaveformSlot::Scalar(s) => s.value,
        WaveformSlot::Empty => C::new(F::ZERO, F::ZERO),
        other => panic!("folded scalar constant did not reduce to a scalar: {other:?}"),
    }
}

/// The real value of a folded real-constant subgraph.
fn real_value<F: Real>(slot: &WaveformSlot<F>) -> F {
    match slot {
        WaveformSlot::Real(r) => *r,
        other => panic!("folded real constant did not reduce to a real: {other:?}"),
    }
}

fn real<F: Real + FromPrimitive>(x: f64) -> F {
    F::from_f64(x).expect("value convertible to real scalar")
}

/// `num/den` computed in `F`, each converted from `i64` before the division (rather
/// than pre-dividing in `f64`) so the folded value isn't limited to `f64` precision
/// when `F` is a higher-precision scalar.
fn ratio<F: Real + FromPrimitive>(num: i64, den: i64) -> F {
    F::from_i64(num).expect("i64 numerator convertible to scalar")
        / F::from_i64(den).expect("i64 denominator convertible to scalar")
}

fn cplx<F: Real + FromPrimitive>(x: Complex64) -> C<F> {
    C::new(real(x.re), real(x.im))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helas::eval::op::ConstKind;
    use crate::ufo::sm::{sm_model, SMRestrict};
    use crate::ufo::EvaluatedModel;

    /// Bare `Op::CoeffRat` leaves under a non-constant root stay pool reads (not folded):
    /// a real rational (`imag == false`) routes to `consts_f`, an imaginary one
    /// (`imag == true`) to `consts_c`, and a repeated real rational dedups to one entry.
    #[test]
    fn coeff_rat_leaves_route_to_real_and_complex_pools() {
        let mut b = AstBuilder::new();
        let real_leaf = b.add(
            Op::CoeffRat,
            Sym::Rational {
                num: 1,
                den: 3,
                imag: false,
            },
            vec![],
        );
        let imag_leaf = b.add(
            Op::CoeffRat,
            Sym::Rational {
                num: 2,
                den: 5,
                imag: true,
            },
            vec![],
        );
        // A repeat of `real_leaf`'s rational, to confirm the real pool dedups `CoeffRat`
        // requests the same way it dedups `Coeff`.
        let real_leaf_dup = b.add(
            Op::CoeffRat,
            Sym::Rational {
                num: 1,
                den: 3,
                imag: false,
            },
            vec![],
        );
        // A `Flows` root is non-constant, so its constant children are kept as-is rather
        // than folded — each stays a `CoeffRat` pool-read leaf.
        let root = b.add(
            Op::Flows,
            Sym::None,
            vec![real_leaf, imag_leaf, real_leaf_dup],
        );
        let ast = b.finish(root);

        let folded = Folded::build(&ast);

        assert_eq!(folded.ast.value(folded.ast.root()).op, Op::Flows);
        let flows_kids = folded.ast.children_ids(folded.ast.root());
        assert_eq!(flows_kids.len(), 3);
        let real_node = folded.ast.value(flows_kids[0]);
        let imag_node = folded.ast.value(flows_kids[1]);
        let dup_node = folded.ast.value(flows_kids[2]);
        assert_eq!(real_node.op, Op::CoeffRat);
        assert_eq!(
            real_node.leaf.kind(),
            ConstKind::Real,
            "real CoeffRat leaf must fold to a real-pool index, got {:?}",
            real_node.leaf
        );
        let real_idx = real_node.leaf.index();
        assert_eq!(
            imag_node.leaf.kind(),
            ConstKind::Complex,
            "imaginary CoeffRat leaf must fold to a complex-pool index, got {:?}",
            imag_node.leaf
        );
        let imag_idx = imag_node.leaf.index();
        assert_eq!(
            dup_node.leaf.kind(),
            ConstKind::Real,
            "duplicated real CoeffRat leaf must fold to a real-pool index"
        );
        let dup_idx = dup_node.leaf.index();
        assert_eq!(
            dup_idx, real_idx,
            "repeated rational must dedup to one pool entry"
        );

        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model);
        let (consts_c, consts_f): (Box<[C<f64>]>, Box<[f64]>) = folded.pools(&evaluated);

        assert_eq!(consts_f[real_idx as usize], 1.0 / 3.0);
        assert_eq!(consts_c[imag_idx as usize], C::new(0.0, 2.0 / 5.0));
        // Only one real pool entry: the duplicate rational didn't grow the pool.
        assert_eq!(consts_f.len(), 1);
        assert_eq!(consts_c.len(), 1);
    }

    /// A constant composite (`Mul(Coupling, Coeff)`) consumed by a non-constant node
    /// collapses into a single `CoeffRat` pool-read leaf, and its card-time value is
    /// appended to `consts_c` (past the base coupling entry) as `coupling · coeff`.
    #[test]
    fn constant_composite_folds_into_a_pool_entry() {
        let mut b = AstBuilder::new();
        // A vector external → non-constant current, so its parent `Mul` is not folded.
        let mass = b.add(Op::Mass, Sym::Particle(ParticleId::from(23usize)), vec![]);
        let ext = b.add(
            Op::External,
            Sym::Ext {
                leg_idx: 0,
                spin: 3,
                charge: Charge::Particle,
                incoming: false,
            },
            vec![mass],
        );
        let coup = b.add(
            Op::Coupling,
            Sym::Coupling(CouplingId::from(5usize)),
            vec![],
        );
        let coeff = b.add(Op::Coeff, Sym::Coeff(2.0), vec![]);
        // Constant composite: folds to one bind-time complex value.
        let g = b.add(Op::Mul, Sym::None, vec![coup, coeff]);
        // Non-constant product (scales the external current by the constant).
        let root = b.add(Op::Mul, Sym::None, vec![ext, g]);
        let ast = b.finish(root);

        let folded = Folded::build(&ast);

        // The root stays a binary `Mul`; its constant operand is now a childless
        // `CoeffRat` leaf indexing the complex pool past the base coupling (index 1).
        let root_kids = folded.ast.children_ids(folded.ast.root());
        assert_eq!(root_kids.len(), 2);
        let g_node = folded.ast.value(root_kids[1]);
        assert_eq!(
            g_node.op,
            Op::CoeffRat,
            "folded composite must be a pool-read leaf"
        );
        assert!(
            folded.ast.children_ids(root_kids[1]).is_empty(),
            "folded composite must be a leaf (no children)"
        );
        assert_eq!(
            g_node.leaf.kind(),
            ConstKind::Complex,
            "scalar-constant composite must fold to a complex-pool index, got {:?}",
            g_node.leaf
        );
        let g_idx = g_node.leaf.index();

        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model);
        let (consts_c, _consts_f): (Box<[C<f64>]>, Box<[f64]>) = folded.pools(&evaluated);

        // consts_c[0] is the base coupling; consts_c[g_idx] is the folded product.
        let expected = consts_c[0] * 2.0;
        assert_eq!(consts_c[g_idx as usize], expected);
        assert_eq!(consts_c.len(), 2, "base coupling + one folded composite");
    }
}
