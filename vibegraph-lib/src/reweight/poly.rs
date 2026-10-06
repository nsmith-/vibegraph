//! Which monomials in a set of model parameters a tree amplitude is a polynomial in.
//!
//! A tree amplitude is, diagram by diagram, a product of vertex factors and
//! propagators, and every vertex factor is a sum of UFO couplings times Lorentz and
//! colour structures. When parameters `P₁…Pₙ` move only couplings — no mass or
//! width anywhere in the process depends on them — and every coupling is a
//! polynomial in them with parameter-independent coefficients, the amplitude is a
//! polynomial too: a diagram's monomials are the products of one monomial per
//! vertex, and the amplitude's are the union over diagrams. An SMEFT process
//! restricted to one insertion per diagram has the amplitude monomials
//! `{1, c₁, …, cₙ}`.
//!
//! [`Support`] is that set of monomials, as exponent vectors. It is computed
//! symbolically from the UFO expressions, never guessed from numbers, and anything
//! the rules below do not prove polynomial makes the analysis answer `None`: a
//! function of a parameter, a division by one, a non-integer or parameter-dependent
//! exponent.

use std::collections::{BTreeSet, HashMap, HashSet};

use crate::diagrams::Diagram;
use crate::ufo::expr::{BinOp, Expr, Func};
use crate::ufo::parameters::ParamNature;
use crate::ufo::UFOModel;

/// Most monomials a support may hold before the analysis gives up on it.
const MAX_TERMS: usize = 4096;

/// A set of monomials `Π Pᵢ^eᵢ` over a fixed list of parameters, each an exponent
/// vector `e` in parameter order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Support {
    arity: usize,
    terms: BTreeSet<Box<[u8]>>,
}

impl Support {
    /// The empty set: the zero polynomial.
    pub fn empty(arity: usize) -> Support {
        Support {
            arity,
            terms: BTreeSet::new(),
        }
    }

    /// `{1}`: independent of every parameter.
    pub fn constant(arity: usize) -> Support {
        let mut s = Support::empty(arity);
        s.terms.insert(vec![0; arity].into_boxed_slice());
        s
    }

    /// `{Pᵢ}`.
    pub fn variable(arity: usize, i: usize) -> Support {
        let mut e = vec![0; arity];
        e[i] = 1;
        let mut s = Support::empty(arity);
        s.terms.insert(e.into_boxed_slice());
        s
    }

    /// The number of parameters the exponent vectors run over.
    pub fn arity(&self) -> usize {
        self.arity
    }

    /// The monomials' exponent vectors, in lexicographic order.
    pub fn terms(&self) -> impl Iterator<Item = &[u8]> {
        self.terms.iter().map(|t| &t[..])
    }

    pub fn len(&self) -> usize {
        self.terms.len()
    }

    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    /// Whether the set is `{1}`.
    pub fn is_constant(&self) -> bool {
        self.terms.len() == 1 && self.terms.iter().all(|t| t.iter().all(|&e| e == 0))
    }

    /// The largest total degree, or `None` for the empty set.
    pub fn degree(&self) -> Option<u32> {
        self.terms
            .iter()
            .map(|t| t.iter().map(|&e| u32::from(e)).sum())
            .max()
    }

    /// The largest power of parameter `i` in any monomial.
    pub fn max_exponent(&self, i: usize) -> u8 {
        self.terms.iter().map(|t| t[i]).max().unwrap_or(0)
    }

    /// The monomials a sum of two such polynomials can carry.
    pub fn union(&self, other: &Support) -> Support {
        Support {
            arity: self.arity,
            terms: self.terms.union(&other.terms).cloned().collect(),
        }
    }

    /// The monomials a product of two such polynomials can carry: every pairwise
    /// product. `None` past [`MAX_TERMS`] monomials or an exponent above 255.
    pub fn product(&self, other: &Support) -> Option<Support> {
        let mut terms = BTreeSet::new();
        for a in &self.terms {
            for b in &other.terms {
                let t: Option<Box<[u8]>> = a
                    .iter()
                    .zip(b.iter())
                    .map(|(x, y)| x.checked_add(*y))
                    .collect();
                terms.insert(t?);
                if terms.len() > MAX_TERMS {
                    return None;
                }
            }
        }
        Some(Support {
            arity: self.arity,
            terms,
        })
    }
}

/// The symbolic analysis of UFO expressions as polynomials in a list of external
/// parameters.
pub struct PolyAnalysis<'m> {
    model: &'m UFOModel,
    params: Vec<String>,
    /// Every internal parameter that transitively depends on one of `params`.
    driven: HashSet<String>,
    internal: HashMap<&'m str, &'m Expr>,
    memo: HashMap<String, Option<Support>>,
}

impl<'m> PolyAnalysis<'m> {
    pub fn new(model: &'m UFOModel, params: &[&str]) -> Self {
        let internal = model
            .params
            .internals
            .iter()
            .filter_map(|p| match &p.nature {
                ParamNature::Internal { expr, .. } => Some((p.name.as_str(), expr)),
                ParamNature::External { .. } => None,
            })
            .collect();
        PolyAnalysis {
            model,
            params: params.iter().map(|p| p.to_string()).collect(),
            driven: params
                .iter()
                .flat_map(|p| model.params.dependents(p))
                .collect(),
            internal,
            memo: HashMap::new(),
        }
    }

    fn arity(&self) -> usize {
        self.params.len()
    }

    /// Whether `name` moves when any of the parameters does, the parameters
    /// themselves included.
    pub fn moves(&self, name: &str) -> bool {
        self.params.iter().any(|p| p == name) || self.driven.contains(name)
    }

    /// The monomials a model parameter carries.
    pub fn param_support(&mut self, name: &str) -> Option<Support> {
        if let Some(i) = self.params.iter().position(|p| p == name) {
            return Some(Support::variable(self.arity(), i));
        }
        if !self.driven.contains(name) {
            return Some(Support::constant(self.arity()));
        }
        if let Some(cached) = self.memo.get(name) {
            return cached.clone();
        }
        let result = match self.internal.get(name) {
            Some(expr) => self.expr_support(expr),
            None => None,
        };
        self.memo.insert(name.to_string(), result.clone());
        result
    }

    /// The monomials an expression carries, or `None` if it is not provably a
    /// polynomial in the parameters.
    pub fn expr_support(&mut self, expr: &Expr) -> Option<Support> {
        let n = self.arity();
        match expr {
            Expr::Num(_) | Expr::Pi => Some(Support::constant(n)),
            Expr::Param(name) => self.param_support(name),
            Expr::Neg(inner) => self.expr_support(inner),
            Expr::BinOp(op, lhs, rhs) => {
                let l = self.expr_support(lhs)?;
                let r = self.expr_support(rhs)?;
                match op {
                    BinOp::Add | BinOp::Sub => Some(l.union(&r)),
                    BinOp::Mul => l.product(&r),
                    // Division by a constant keeps a polynomial one.
                    BinOp::Div => r.is_constant().then_some(l),
                    BinOp::Pow => {
                        if !r.is_constant() {
                            return None;
                        }
                        if l.is_constant() {
                            return Some(Support::constant(n));
                        }
                        match rhs.as_ref() {
                            Expr::Num(k) if k.fract() == 0.0 && *k >= 0.0 && *k <= 255.0 => (0..*k
                                as u32)
                                .try_fold(Support::constant(n), |acc, _| acc.product(&l)),
                            _ => None,
                        }
                    }
                }
            }
            // The parameters are real, so conjugation and the real/imaginary split
            // of `complex(re, im)` act on the coefficients alone and keep the
            // monomials.
            Expr::Call(Func::Conj | Func::Re | Func::Im, args) if args.len() == 1 => {
                self.expr_support(&args[0])
            }
            Expr::Call(_, args) => {
                for a in args {
                    if !self.expr_support(a)?.is_constant() {
                        return None;
                    }
                }
                Some(Support::constant(n))
            }
            Expr::Complex(re, im) => Some(self.expr_support(re)?.union(&self.expr_support(im)?)),
        }
    }

    /// The monomials a set of diagrams' amplitude carries, or `None` when it is not
    /// a polynomial in the parameters: a coupling that is not, or a mass or width
    /// of any particle in the diagrams that moves with them.
    pub fn amplitude_support(&mut self, diagrams: &[Diagram]) -> Option<Support> {
        let model = self.model;
        let n = self.arity();
        let mut total = Support::empty(n);
        for diagram in diagrams {
            let particles = diagram
                .legs
                .iter()
                .map(|l| l.particle)
                .chain(diagram.props.iter().map(|p| p.particle));
            for pid in particles {
                let particle = model.particle(pid);
                if self.moves(&particle.mass_param) || self.moves(&particle.width_param) {
                    return None;
                }
            }
            let mut support = Support::constant(n);
            for vertex in &diagram.vertices {
                let mut vertex_support = Support::empty(n);
                for &coupling in model.vertex_def(vertex.interaction).couplings.values() {
                    let expr = &model.coupling_def(coupling).value;
                    vertex_support = vertex_support.union(&self.expr_support(expr)?);
                }
                support = support.product(&vertex_support)?;
            }
            total = total.union(&support);
        }
        Some(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::ufo::expr::parse_expr;
    use crate::ufo::sm::{sm_model, SMRestrict};

    fn terms(s: &Support) -> Vec<Vec<u8>> {
        s.terms().map(<[u8]>::to_vec).collect()
    }

    fn support(model: &UFOModel, params: &[&str], src: &str) -> Option<Vec<Vec<u8>>> {
        PolyAnalysis::new(model, params)
            .expr_support(&parse_expr(src).unwrap())
            .map(|s| terms(&s))
    }

    #[test]
    fn expressions_are_read_as_polynomials() {
        let model = sm_model(SMRestrict::Default);
        let y = &["ymt"];
        assert_eq!(support(&model, y, "ymt"), Some(vec![vec![1]]));
        assert_eq!(
            support(&model, y, "2*ymt**2 - ymt"),
            Some(vec![vec![1], vec![2]])
        );
        assert_eq!(
            support(&model, y, "(1 + ymt)**3"),
            Some(vec![vec![0], vec![1], vec![2], vec![3]])
        );
        assert_eq!(support(&model, y, "ymt/4"), Some(vec![vec![1]]));
        assert_eq!(
            support(&model, y, "complex(0,1)*complexconjugate(ymt)"),
            Some(vec![vec![1]])
        );
        assert_eq!(support(&model, y, "cmath.sqrt(MT)"), Some(vec![vec![0]]));
        // The ones the analysis must refuse rather than guess at.
        assert_eq!(support(&model, y, "1/ymt"), None);
        assert_eq!(support(&model, y, "cmath.sqrt(ymt)"), None);
        assert_eq!(support(&model, y, "ymt**0.5"), None);
        assert_eq!(support(&model, y, "2**ymt"), None);
        assert_eq!(support(&model, y, "ymt**ymt"), None);
    }

    #[test]
    fn several_parameters_give_exponent_vectors() {
        let model = sm_model(SMRestrict::Default);
        let p = &["ymt", "ymtau"];
        assert_eq!(
            support(&model, p, "ymt*ymtau + ymtau**2 + 3"),
            Some(vec![vec![0, 0], vec![0, 2], vec![1, 1]])
        );
        assert_eq!(support(&model, p, "ymt/ymtau"), None);
    }

    /// The Standard Model's Yukawa couplings reach `ymt` through the internal `yt`,
    /// which the analysis has to follow rather than treat as a constant.
    #[test]
    fn internal_parameters_are_followed() {
        let model = sm_model(SMRestrict::Default);
        let mut a = PolyAnalysis::new(&model, &["ymt"]);
        assert_eq!(a.param_support("yt"), Some(Support::variable(1, 0)));
        assert_eq!(a.param_support("MT"), Some(Support::constant(1)));
        // `ee` is `2·√(π·aEW)` and `aEW` is `1/aEWM1`.
        let mut b = PolyAnalysis::new(&model, &["aEWM1"]);
        assert_eq!(b.param_support("ee"), None);
    }

    fn diagrams(process: &str, model: &UFOModel) -> Vec<Diagram> {
        let card =
            parse_proc_card(&format!("generate {process}"), &ParsingOptions::default()).unwrap();
        generate_from_proc_card(&card, model)
            .unwrap()
            .into_iter()
            .flat_map(|s| s.diagrams)
            .collect()
    }

    #[test]
    fn amplitude_monomials_follow_the_diagrams() {
        let model = sm_model(SMRestrict::Default);
        // Higgsstrahlung carries no Yukawa vertex and radiation off the top one, so
        // the amplitude is `a + b·ymt`.
        let tth = diagrams("e+ e- > t t~ h", &model);
        let s = PolyAnalysis::new(&model, &["ymt"])
            .amplitude_support(&tth)
            .unwrap();
        assert_eq!(terms(&s), [vec![0], vec![1]]);

        // `ta+ ta- > t t~` through an s-channel Higgs carries both Yukawas at once;
        // the Z and photon carry neither. `ymb` is in the list and absent from the
        // process.
        let tt = diagrams("ta+ ta- > t t~", &model);
        let s = PolyAnalysis::new(&model, &["ymt", "ymtau", "ymb"])
            .amplitude_support(&tt)
            .unwrap();
        assert_eq!(terms(&s), [vec![0, 0, 0], vec![1, 1, 0]]);

        // A process the parameter does not enter.
        let mumu = diagrams("e+ e- > mu+ mu-", &model);
        assert!(PolyAnalysis::new(&model, &["ymt"])
            .amplitude_support(&mumu)
            .unwrap()
            .is_constant());

        // `MZ` is the Z propagator's pole: not a coupling-only parameter.
        assert_eq!(
            PolyAnalysis::new(&model, &["MZ"]).amplitude_support(&mumu),
            None
        );
    }

    #[test]
    fn products_refuse_to_overflow() {
        let x = Support::variable(1, 0);
        let high = (0..200)
            .try_fold(Support::constant(1), |a, _| a.product(&x))
            .unwrap();
        assert_eq!(high.product(&high), None);
        assert_eq!(high.degree(), Some(200));
    }
}
