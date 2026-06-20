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
}

/// The folded, card-independent skeleton plus the pool specifications that resolve it.
#[derive(Debug, Clone)]
pub struct Folded {
    /// Same structure as the symbolic AST, with leaves rewritten to pool indices.
    pub ast: Ast<Const>,
    /// `consts_c[i] = coupling(pool_c[i])`.
    pool_c: Vec<CouplingId>,
    /// `consts_f[j] = resolve(pool_f[j])`.
    pool_f: Vec<RealReq>,
}

impl Folded {
    /// Build the folded skeleton from the symbolic AST, deduping constants into the
    /// two pool specs.
    pub fn build(sym: &Ast<Sym>) -> Folded {
        let mut pool_c: Vec<CouplingId> = Vec::new();
        let mut c_index: HashMap<CouplingId, u32> = HashMap::new();
        let mut pool_f: Vec<RealReq> = Vec::new();
        let mut f_index: HashMap<RealReq, u32> = HashMap::new();

        let mut intern_c = |id: CouplingId| -> u32 {
            *c_index.entry(id).or_insert_with(|| {
                pool_c.push(id);
                (pool_c.len() - 1) as u32
            })
        };
        let mut intern_f = |req: RealReq| -> u32 {
            *f_index.entry(req).or_insert_with(|| {
                pool_f.push(req);
                (pool_f.len() - 1) as u32
            })
        };

        let mut builder = AstBuilder::new();
        for (i, node) in sym.nodes().iter().enumerate() {
            let children = sym.child_ids(i as u32).to_vec();
            let leaf = match (node.op, node.leaf) {
                (Op::Coupling, Sym::Coupling(id)) => Const::Cplx(intern_c(id)),
                (Op::Mass, Sym::Particle(id)) => Const::Real(intern_f(RealReq::Mass(id))),
                (Op::Width, Sym::Particle(id)) => Const::Real(intern_f(RealReq::Width(id))),
                (Op::Coeff, Sym::Coeff(c)) => Const::Real(intern_f(RealReq::Coeff(c.to_bits()))),
                (
                    Op::External,
                    Sym::Ext {
                        leg_idx,
                        spin,
                        charge,
                    },
                ) => Const::Ext {
                    leg_idx,
                    spin,
                    charge,
                },
                _ => Const::None,
            };
            builder.add(node.op, leaf, children);
        }

        Folded {
            ast: builder.finish(sym.root_id()),
            pool_c,
            pool_f,
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
            .map(|&id| cplx::<F>(evaluated.coupling(id)))
            .collect();
        let consts_f: Box<[F]> = self
            .pool_f
            .iter()
            .map(|req| match *req {
                RealReq::Mass(id) => real::<F>(evaluated.mass(id)),
                RealReq::Width(id) => real::<F>(evaluated.width(id)),
                RealReq::Coeff(bits) => real::<F>(f64::from_bits(bits)),
            })
            .collect();
        (consts_c, consts_f)
    }

    /// Coupling ids referenced by the amplitude (from the complex pool spec).
    pub fn coupling_ids(&self) -> impl Iterator<Item = CouplingId> + '_ {
        self.pool_c.iter().copied()
    }

    /// Particle ids referenced by the amplitude (mass/width entries of the real pool).
    pub fn particle_ids(&self) -> impl Iterator<Item = ParticleId> + '_ {
        self.pool_f.iter().filter_map(|req| match req {
            RealReq::Mass(id) | RealReq::Width(id) => Some(*id),
            RealReq::Coeff(_) => None,
        })
    }
}

fn real<F: Real + FromPrimitive>(x: f64) -> F {
    F::from_f64(x).expect("value convertible to real scalar")
}

fn cplx<F: Real + FromPrimitive>(x: Complex64) -> C<F> {
    C::new(real(x.re), real(x.im))
}
