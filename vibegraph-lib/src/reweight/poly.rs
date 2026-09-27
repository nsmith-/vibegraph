//! Which powers of one model parameter a matrix element is a polynomial in.
//!
//! A tree amplitude is, diagram by diagram, a product of vertex factors and
//! propagators, and every vertex factor is a sum of UFO couplings times Lorentz and
//! colour structures. When a parameter `P` moves only couplings — no mass or width
//! anywhere in the process depends on it — and every coupling is a polynomial in
//! `P` with `P`-independent coefficients, the amplitude is a polynomial in `P` too:
//! a diagram's powers are the sums of one power per vertex, and the amplitude's are
//! the union over diagrams. For real `P`, `|M|² = Σ conj(A)·C·A` is a real polynomial
//! whose powers are the pairwise sums of the amplitude's.
//!
//! [`Support`] is that set of powers. It is computed symbolically from the UFO
//! expressions, never guessed from numbers, and anything the rules below do not
//! prove polynomial makes the analysis answer `None`: a function of `P`, a division
//! by it, a non-integer or `P`-dependent exponent.

use std::collections::{HashMap, HashSet};

use crate::diagrams::Diagram;
use crate::ufo::expr::{BinOp, Expr, Func};
use crate::ufo::parameters::ParamNature;
use crate::ufo::UFOModel;

/// A set of non-negative integer powers, bit `n` standing for `Pⁿ`.
///
/// Powers above 63 are not representable; an operation that would produce one
/// fails instead, which the analysis reports as "not a polynomial it can use".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Support(u64);

impl Support {
    /// `{0}`: independent of the parameter.
    pub const CONSTANT: Support = Support(1);

    /// `{n}`.
    pub fn power(n: u32) -> Option<Support> {
        (n < 64).then(|| Support(1u64 << n))
    }

    /// The powers, in increasing order.
    pub fn powers(self) -> impl Iterator<Item = u32> {
        (0..64).filter(move |n| self.0 >> n & 1 == 1)
    }

    /// The largest power, or `None` for the empty set.
    pub fn degree(self) -> Option<u32> {
        (self.0 != 0).then(|| 63 - self.0.leading_zeros())
    }

    /// The powers a sum of two such polynomials can carry.
    pub fn union(self, other: Support) -> Support {
        Support(self.0 | other.0)
    }

    /// The powers a product of two such polynomials can carry: every pairwise sum.
    pub fn product(self, other: Support) -> Option<Support> {
        let mut out = 0u64;
        for n in self.powers() {
            if other.0 != 0 && n + other.degree()? > 63 {
                return None;
            }
            out |= other.0 << n;
        }
        Some(Support(out))
    }

    pub fn is_constant(self) -> bool {
        self == Support::CONSTANT
    }
}

/// The symbolic analysis of UFO expressions as polynomials in one external
/// parameter.
pub struct PolyAnalysis<'m> {
    model: &'m UFOModel,
    param: String,
    /// Every internal parameter that transitively depends on `param`.
    driven: HashSet<String>,
    internal: HashMap<&'m str, &'m Expr>,
    memo: HashMap<String, Option<Support>>,
}

impl<'m> PolyAnalysis<'m> {
    pub fn new(model: &'m UFOModel, param: &str) -> Self {
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
            param: param.to_string(),
            driven: model.params.dependents(param),
            internal,
            memo: HashMap::new(),
        }
    }

    /// Whether `name` moves when the parameter does, the parameter itself included.
    pub fn moves(&self, name: &str) -> bool {
        name == self.param || self.driven.contains(name)
    }

    /// The powers of the parameter a model parameter carries.
    pub fn param_support(&mut self, name: &str) -> Option<Support> {
        if name == self.param {
            return Support::power(1);
        }
        if !self.driven.contains(name) {
            return Some(Support::CONSTANT);
        }
        if let Some(&cached) = self.memo.get(name) {
            return cached;
        }
        let result = match self.internal.get(name) {
            Some(expr) => self.expr_support(expr),
            None => None,
        };
        self.memo.insert(name.to_string(), result);
        result
    }

    /// The powers of the parameter an expression carries, or `None` if it is not
    /// provably a polynomial in it.
    pub fn expr_support(&mut self, expr: &Expr) -> Option<Support> {
        match expr {
            Expr::Num(_) | Expr::Pi => Some(Support::CONSTANT),
            Expr::Param(name) => self.param_support(name),
            Expr::Neg(inner) => self.expr_support(inner),
            Expr::BinOp(op, lhs, rhs) => {
                let l = self.expr_support(lhs)?;
                let r = self.expr_support(rhs)?;
                match op {
                    BinOp::Add | BinOp::Sub => Some(l.union(r)),
                    BinOp::Mul => l.product(r),
                    // Division by a constant keeps a polynomial one.
                    BinOp::Div => r.is_constant().then_some(l),
                    BinOp::Pow => {
                        if !r.is_constant() {
                            return None;
                        }
                        if l.is_constant() {
                            return Some(Support::CONSTANT);
                        }
                        match rhs.as_ref() {
                            Expr::Num(k) if k.fract() == 0.0 && *k >= 0.0 && *k <= 63.0 => {
                                (0..*k as u32).try_fold(Support::CONSTANT, |acc, _| acc.product(l))
                            }
                            _ => None,
                        }
                    }
                }
            }
            // The parameter is real, so conjugation and the real/imaginary split of
            // `complex(re, im)` act on the coefficients alone and keep the powers.
            Expr::Call(Func::Conj | Func::Re | Func::Im, args) if args.len() == 1 => {
                self.expr_support(&args[0])
            }
            Expr::Call(_, args) => {
                for a in args {
                    if !self.expr_support(a)?.is_constant() {
                        return None;
                    }
                }
                Some(Support::CONSTANT)
            }
            Expr::Complex(re, im) => Some(self.expr_support(re)?.union(self.expr_support(im)?)),
        }
    }

    /// The powers of the parameter a set of diagrams' amplitude carries, or `None`
    /// when it is not a polynomial in it: a coupling that is not, or a mass or width
    /// of any particle in the diagrams that moves with the parameter.
    pub fn amplitude_support(&mut self, diagrams: &[Diagram]) -> Option<Support> {
        let model = self.model;
        let mut total = Support(0);
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
            let mut support = Support::CONSTANT;
            for vertex in &diagram.vertices {
                let mut vertex_support = Support(0);
                for &coupling in model.vertex_def(vertex.interaction).couplings.values() {
                    let expr = &model.coupling_def(coupling).value;
                    vertex_support = vertex_support.union(self.expr_support(expr)?);
                }
                support = support.product(vertex_support)?;
            }
            total = total.union(support);
        }
        Some(total)
    }
}

/// The powers `|A|²` carries when `A` carries `amplitude`: every pairwise sum.
pub fn squared(amplitude: Support) -> Option<Support> {
    amplitude.product(amplitude)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::ufo::expr::parse_expr;
    use crate::ufo::sm::{sm_model, SMRestrict};

    fn support(model: &UFOModel, param: &str, src: &str) -> Option<Vec<u32>> {
        PolyAnalysis::new(model, param)
            .expr_support(&parse_expr(src).unwrap())
            .map(|s| s.powers().collect())
    }

    #[test]
    fn expressions_are_read_as_polynomials() {
        let model = sm_model(SMRestrict::Default);
        assert_eq!(support(&model, "ymt", "ymt"), Some(vec![1]));
        assert_eq!(support(&model, "ymt", "2*ymt**2 - ymt"), Some(vec![1, 2]));
        assert_eq!(
            support(&model, "ymt", "(1 + ymt)**3"),
            Some(vec![0, 1, 2, 3])
        );
        assert_eq!(support(&model, "ymt", "ymt/4"), Some(vec![1]));
        assert_eq!(
            support(&model, "ymt", "complex(0,1)*complexconjugate(ymt)"),
            Some(vec![1])
        );
        assert_eq!(support(&model, "ymt", "cmath.sqrt(MT)"), Some(vec![0]));
        // The ones the analysis must refuse rather than guess at.
        assert_eq!(support(&model, "ymt", "1/ymt"), None);
        assert_eq!(support(&model, "ymt", "cmath.sqrt(ymt)"), None);
        assert_eq!(support(&model, "ymt", "ymt**0.5"), None);
        assert_eq!(support(&model, "ymt", "2**ymt"), None);
        assert_eq!(support(&model, "ymt", "ymt**ymt"), None);
    }

    /// The Standard Model's Yukawa couplings reach `ymt` through the internal `yt`,
    /// which the analysis has to follow rather than treat as a constant.
    #[test]
    fn internal_parameters_are_followed() {
        let model = sm_model(SMRestrict::Default);
        let mut a = PolyAnalysis::new(&model, "ymt");
        assert_eq!(a.param_support("yt"), Support::power(1));
        assert_eq!(a.param_support("MT"), Some(Support::CONSTANT));
        // `ee` is `2·√(π·aEW)` and `aEW` is `1/aEWM1`.
        let mut b = PolyAnalysis::new(&model, "aEWM1");
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
    fn amplitude_powers_follow_the_diagrams() {
        let model = sm_model(SMRestrict::Default);
        // `h > t t~`-type diagrams carry one Yukawa vertex; the Higgsstrahlung
        // diagrams of `e+ e- > t t~ h` carry none, so the amplitude is `a + b·ymt`.
        let tth = diagrams("e+ e- > t t~ h", &model);
        let s = PolyAnalysis::new(&model, "ymt")
            .amplitude_support(&tth)
            .unwrap();
        assert_eq!(s.powers().collect::<Vec<_>>(), [0, 1]);
        assert_eq!(squared(s).unwrap().powers().collect::<Vec<_>>(), [0, 1, 2]);

        // A process the parameter does not enter.
        let mumu = diagrams("e+ e- > mu+ mu-", &model);
        let s = PolyAnalysis::new(&model, "ymt")
            .amplitude_support(&mumu)
            .unwrap();
        assert!(s.is_constant());

        // `MZ` is the Z propagator's pole: not a coupling-only parameter.
        assert_eq!(
            PolyAnalysis::new(&model, "MZ").amplitude_support(&mumu),
            None
        );
    }

    #[test]
    fn products_refuse_to_overflow() {
        let high = Support::power(40).unwrap();
        assert_eq!(high.product(high), None);
        assert_eq!(
            Support::power(3)
                .unwrap()
                .product(Support::power(4).unwrap()),
            Support::power(7)
        );
    }
}
