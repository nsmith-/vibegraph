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

use super::ast::{Ast, AstBuilder};
use super::op::{Const, Op, Sym};
use super::tree::Tree;
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
    /// Same structure as the symbolic AST, with leaves rewritten to pool indices.
    pub ast: Ast<Const>,
    /// `consts_c[i] = resolve(pool_c[i])`.
    pool_c: Vec<ComplexReq>,
    /// `consts_f[j] = resolve(pool_f[j])`.
    pool_f: Vec<RealReq>,
    /// `Const::Ext(k)` resolves to `pool_ext[k]`.
    pool_ext: Vec<ExtLeg>,
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
                    (Const::Complex(intern_c(ComplexReq::Coupling(cid))), vec![])
                }
                (Op::Mass, Sym::Particle(pid)) => {
                    (Const::Real(intern_f(RealReq::Mass(pid))), vec![])
                }
                (Op::Width, Sym::Particle(pid)) => {
                    (Const::Real(intern_f(RealReq::Width(pid))), vec![])
                }
                (Op::Coeff, Sym::Coeff(c)) => {
                    (Const::Real(intern_f(RealReq::Coeff(c.to_bits()))), vec![])
                }
                (Op::CoeffRat, Sym::Rational { num, den, imag }) => {
                    if imag {
                        (Const::Complex(intern_c(ComplexReq::Rat(num, den))), vec![])
                    } else {
                        (Const::Real(intern_f(RealReq::Rat(num, den))), vec![])
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
                    (Const::Ext(k), vec![])
                }
                _ => (
                    Const::None,
                    sym.children_ids(id)
                        .iter()
                        .map(|&c| remap[c as usize])
                        .collect(),
                ),
            };
            remap[id as usize] = builder.add(node.op, leaf, children);
        }

        Folded {
            ast: builder.finish(remap[sym.root() as usize]),
            pool_c,
            pool_f,
            pool_ext,
        }
    }

    /// Resolve the two numeric pools for a parameter card at scalar precision `F`.
    pub fn pools<F: Real + FromPrimitive>(
        &self,
        evaluated: &EvaluatedModel,
    ) -> (Box<[C<F>]>, Box<[F]>) {
        let consts_c: Box<[C<F>]> = self
            .pool_c
            .iter()
            .map(|req| match *req {
                ComplexReq::Coupling(id) => cplx::<F>(evaluated.coupling(id)),
                ComplexReq::Rat(num, den) => C::new(F::ZERO, ratio::<F>(num, den)),
            })
            .collect();
        let consts_f: Box<[F]> = self
            .pool_f
            .iter()
            .map(|req| match *req {
                RealReq::Mass(id) => real::<F>(evaluated.mass(id)),
                RealReq::Width(id) => real::<F>(evaluated.width(id)),
                RealReq::Coeff(bits) => real::<F>(f64::from_bits(bits)),
                RealReq::Rat(num, den) => ratio::<F>(num, den),
            })
            .collect();
        (consts_c, consts_f)
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
    use crate::ufo::sm::{sm_model, SMRestrict};
    use crate::ufo::EvaluatedModel;

    /// `Op::Flows`/`Op::CoeffRat` cannot come from `lower()` yet (nothing emits them),
    /// so the fold mapping is exercised directly against a hand-built AST: a `Flows`
    /// root over a real rational (`imag == false`) and an imaginary one
    /// (`imag == true`), plus a repeated leaf to confirm pool dedup.
    #[test]
    fn coeff_rat_folds_to_real_and_complex_pool_entries() {
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
        // A repeat of `real_leaf`'s rational under `Add`, to confirm the real pool
        // dedups `CoeffRat` requests the same way it dedups `Coeff`.
        let real_leaf_dup = b.add(
            Op::CoeffRat,
            Sym::Rational {
                num: 1,
                den: 3,
                imag: false,
            },
            vec![],
        );
        let dup_sum = b.add(Op::Add, Sym::None, vec![real_leaf_dup, imag_leaf]);
        let root = b.add(Op::Flows, Sym::None, vec![real_leaf, dup_sum]);
        let ast = b.finish(root);

        let folded = Folded::build(&ast);

        // Structure: the folded root is still `Op::Flows` over its two children,
        // and the folded leaves keep `Op::CoeffRat` with a resolved `Const`.
        assert_eq!(folded.ast.value(folded.ast.root()).op, Op::Flows);
        let flows_kids = folded.ast.children_ids(folded.ast.root());
        assert_eq!(flows_kids.len(), 2);
        let real_node = folded.ast.value(flows_kids[0]);
        assert_eq!(real_node.op, Op::CoeffRat);
        let Const::Real(real_idx) = real_node.leaf else {
            panic!(
                "real CoeffRat leaf must fold to Const::Real, got {:?}",
                real_node.leaf
            );
        };

        let sum_kids = folded.ast.children_ids(flows_kids[1]);
        let dup_node = folded.ast.value(sum_kids[0]);
        let imag_node = folded.ast.value(sum_kids[1]);
        let Const::Real(dup_idx) = dup_node.leaf else {
            panic!("duplicated real CoeffRat leaf must fold to Const::Real");
        };
        assert_eq!(
            dup_idx, real_idx,
            "repeated rational must dedup to one pool entry"
        );
        let Const::Complex(imag_idx) = imag_node.leaf else {
            panic!(
                "imaginary CoeffRat leaf must fold to Const::Complex, got {:?}",
                imag_node.leaf
            );
        };

        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model);
        let (consts_c, consts_f): (Box<[C<f64>]>, Box<[f64]>) = folded.pools(&evaluated);

        assert_eq!(consts_f[real_idx as usize], 1.0 / 3.0);
        assert_eq!(consts_c[imag_idx as usize], C::new(0.0, 2.0 / 5.0));
        // Only one real pool entry: the duplicate rational didn't grow the pool.
        assert_eq!(consts_f.len(), 1);
        assert_eq!(consts_c.len(), 1);
    }
}
