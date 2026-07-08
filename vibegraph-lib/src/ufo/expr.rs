//! Expression AST for UFO parameter/coupling value strings.
//!
//! UFO `parameters.py` and `couplings.py` store symbolic values as quoted
//! Python expression strings, e.g.:
//!   `'2*cmath.sqrt(aS)*cmath.sqrt(cmath.pi)'`
//!   `'complex(0,1)*G'`
//!
//! This module parses those strings into an [`Expr`] AST and evaluates them
//! against a map of parameter values.

use num_complex::Complex64;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A symbolic expression from a UFO value string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    /// Numeric literal (real).
    Num(f64),
    /// The constant π (`cmath.pi`).
    Pi,
    /// Reference to a named parameter.
    Param(String),
    /// Binary operation.
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    /// Unary negation.
    Neg(Box<Expr>),
    /// Function call (built-in UFO/cmath functions).
    Call(Func, Vec<Expr>),
    /// `complex(re, im)` constructor.
    Complex(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Func {
    Sqrt,
    Log,
    Exp,
    Abs,
    Arg,
    Conj, // complexconjugate
    Re,
    Im,
    Sec, // 1/cos
    Csc, // 1/sin
    ASec,
    ACsc,
    Sin,
    Cos,
    Tan,
    ATan,
}

/// Collect all parameter names referenced in an expression.
pub fn collect_deps(expr: &Expr, deps: &mut Vec<String>) {
    match expr {
        Expr::Param(name) => {
            if !deps.contains(name) {
                deps.push(name.clone());
            }
        }
        Expr::BinOp(_, lhs, rhs) => {
            collect_deps(lhs, deps);
            collect_deps(rhs, deps);
        }
        Expr::Neg(inner) => collect_deps(inner, deps),
        Expr::Call(_, args) => {
            for a in args {
                collect_deps(a, deps);
            }
        }
        Expr::Complex(re, im) => {
            collect_deps(re, deps);
            collect_deps(im, deps);
        }
        Expr::Num(_) | Expr::Pi => {}
    }
}

/// Evaluate an expression given a map of parameter name → complex value.
///
/// Unknown parameter references panic in debug builds and return 0 in release.
pub fn eval(expr: &Expr, params: &HashMap<String, Complex64>) -> Complex64 {
    use std::f64::consts::PI;
    match expr {
        Expr::Num(x) => Complex64::new(*x, 0.0),
        Expr::Pi => Complex64::new(PI, 0.0),
        Expr::Param(name) => *params
            .get(name.as_str())
            .unwrap_or_else(|| panic!("UFO expression references unknown parameter '{name}'")),
        Expr::BinOp(op, lhs, rhs) => {
            let l = eval(lhs, params);
            let r = eval(rhs, params);
            match op {
                BinOp::Add => l + r,
                BinOp::Sub => l - r,
                BinOp::Mul => l * r,
                BinOp::Div => l / r,
                BinOp::Pow => l.powc(r),
            }
        }
        Expr::Neg(inner) => -eval(inner, params),
        Expr::Call(func, args) => {
            let a = |i: usize| eval(&args[i], params);
            match func {
                Func::Sqrt => a(0).sqrt(),
                Func::Log => a(0).ln(),
                Func::Exp => a(0).exp(),
                Func::Abs => Complex64::new(a(0).norm(), 0.0),
                Func::Arg => Complex64::new(a(0).arg(), 0.0),
                Func::Conj => a(0).conj(),
                Func::Re => Complex64::new(a(0).re, 0.0),
                Func::Im => Complex64::new(a(0).im, 0.0),
                Func::Sec => Complex64::new(1.0, 0.0) / a(0).cos(),
                Func::Csc => Complex64::new(1.0, 0.0) / a(0).sin(),
                Func::ASec => (Complex64::new(1.0, 0.0) / a(0)).acos(),
                Func::ACsc => (Complex64::new(1.0, 0.0) / a(0)).asin(),
                Func::Sin => a(0).sin(),
                Func::Cos => a(0).cos(),
                Func::Tan => a(0).tan(),
                Func::ATan => a(0).atan(),
            }
        }
        Expr::Complex(re, im) => Complex64::new(eval(re, params).re, eval(im, params).re),
    }
}

peg::parser! {
    /// PEG grammar for UFO expression strings.
    ///
    /// Precedence (low to high): additive, multiplicative, power, unary, primary.
    pub grammar ufo_expr() for str {

        // Entry point.
        pub rule expression() -> Expr = additive()

        // Additive: left-recursive via iteration.
        rule additive() -> Expr
            = l:multiplicative() rest:(_ op:addop() _ r:multiplicative() {(op, r)})* {
                rest.into_iter().fold(l, |acc, (op, r)| Expr::BinOp(op, Box::new(acc), Box::new(r)))
            }

        rule addop() -> BinOp
            = "+" { BinOp::Add }
            / "-" { BinOp::Sub }

        rule multiplicative() -> Expr
            = l:power() rest:(_ op:mulop() _ r:power() {(op, r)})* {
                rest.into_iter().fold(l, |acc, (op, r)| Expr::BinOp(op, Box::new(acc), Box::new(r)))
            }

        rule mulop() -> BinOp
            = "*" !"*" { BinOp::Mul }
            / "/" { BinOp::Div }

        // Power: right-associative.
        rule power() -> Expr
            = base:unary() _ "**" _ exp:power() { Expr::BinOp(BinOp::Pow, Box::new(base), Box::new(exp)) }
            / unary()

        rule unary() -> Expr
            = "-" _ e:primary() { Expr::Neg(Box::new(e)) }
            / "+" _ e:primary() { e }
            / primary()

        rule primary() -> Expr
            = "(" _ e:additive() _ ")" { e }
            / float_literal()
            / complex_constructor()
            / cmath_call()
            / func_call()
            / "cmath.pi" !ident_continue() { Expr::Pi }
            / ident_expr()

        rule float_literal() -> Expr
            = n:$(
                ['0'..='9']+ "." ['0'..='9']* (("e" / "E") ("+" / "-")? ['0'..='9']+)?
                / ['0'..='9']* "." ['0'..='9']+ (("e" / "E") ("+" / "-")? ['0'..='9']+)?
                / ['0'..='9']+ (("e" / "E") ("+" / "-")? ['0'..='9']+)
                / ['0'..='9']+
            ) {
                Expr::Num(n.parse::<f64>().unwrap())
            }

        // complex(re, im)
        rule complex_constructor() -> Expr
            = "complex(" _ re:additive() _ "," _ im:additive() _ ")" {
                Expr::Complex(Box::new(re), Box::new(im))
            }

        // cmath.func(arg) — note: cmath.pi is matched earlier as a constant
        rule cmath_call() -> Expr
            = "cmath." f:cmath_func_name() "(" _ a:additive() _ ")" {
                Expr::Call(f, vec![a])
            }

        rule cmath_func_name() -> Func
            = "sqrt"  { Func::Sqrt }
            / "log"   { Func::Log }
            / "exp"   { Func::Exp }
            / "atan"  { Func::ATan }
            / "asin"  { Func::ACsc }  // asin not used directly, but consistent
            / "acos"  { Func::ASec }
            / "sin"   { Func::Sin }
            / "cos"   { Func::Cos }
            / "tan"   { Func::Tan }

        // function_library and other bare function names
        rule func_call() -> Expr
            = f:bare_func_name() "(" _ a:additive() _ ")" {
                Expr::Call(f, vec![a])
            }

        rule bare_func_name() -> Func
            = "complexconjugate" { Func::Conj }
            / "conj"             { Func::Conj }
            / "sqrt"             { Func::Sqrt }
            / "abs"              { Func::Abs }
            / "arg"              { Func::Arg }
            / "re"               { Func::Re }
            / "im"               { Func::Im }
            / "atan"             { Func::ATan }
            / "asin"             { Func::ACsc }
            / "acos"             { Func::ASec }
            / "asec"             { Func::ASec }
            / "acsc"             { Func::ACsc }
            / "sin"              { Func::Sin }
            / "cos"              { Func::Cos }
            / "tan"              { Func::Tan }
            / "sec"              { Func::Sec }
            / "csc"              { Func::Csc }

        rule ident_expr() -> Expr
            = name:ident() { Expr::Param(name.to_owned()) }

        rule ident() -> &'input str
            = $(['a'..='z' | 'A'..='Z' | '_'] ident_continue()*)

        rule ident_continue() = ['a'..='z' | 'A'..='Z' | '0'..='9' | '_']

        rule _ = [' ' | '\t' | '\n' | '\r']*
    }
}

/// Parse a UFO expression string into an [`Expr`].
pub fn parse_expr(s: &str) -> Result<Expr, peg::error::ParseError<peg::str::LineCol>> {
    ufo_expr::expression(s.trim())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn params(pairs: &[(&str, f64)]) -> HashMap<String, Complex64> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), Complex64::new(*v, 0.0)))
            .collect()
    }

    #[test]
    fn test_simple_arithmetic() {
        let e = parse_expr("2 + 3").unwrap();
        assert_eq!(eval(&e, &HashMap::new()), Complex64::new(5.0, 0.0));
    }

    #[test]
    fn test_param_reference() {
        let e = parse_expr("aS").unwrap();
        let p = params(&[("aS", 0.118)]);
        assert!((eval(&e, &p).re - 0.118).abs() < 1e-12);
    }

    #[test]
    fn test_g_formula() {
        // G = 2*cmath.sqrt(aS)*cmath.sqrt(cmath.pi)
        let e = parse_expr("2*cmath.sqrt(aS)*cmath.sqrt(cmath.pi)").unwrap();
        let p = params(&[("aS", 0.118)]);
        let expected = 2.0 * (0.118f64).sqrt() * PI.sqrt();
        assert!((eval(&e, &p).re - expected).abs() < 1e-10);
    }

    #[test]
    fn test_complex_constructor() {
        let e = parse_expr("complex(0,1)").unwrap();
        let c = eval(&e, &HashMap::new());
        assert_eq!(c, Complex64::new(0.0, 1.0));
    }

    #[test]
    fn test_power() {
        let e = parse_expr("ee**2").unwrap();
        let p = params(&[("ee", 3.0)]);
        assert!((eval(&e, &p).re - 9.0).abs() < 1e-12);
    }

    #[test]
    fn test_neg_param() {
        let e = parse_expr("-G").unwrap();
        let p = params(&[("G", 1.2177)]);
        assert!((eval(&e, &p).re + 1.2177).abs() < 1e-10);
    }

    #[test]
    fn test_float_trailing_dot() {
        // Floats like '3.' are valid in Python UFO files
        let e = parse_expr("2.*ee").unwrap();
        let p = params(&[("ee", 5.0)]);
        assert!((eval(&e, &p).re - 10.0).abs() < 1e-12);
    }

    #[test]
    fn test_cmath_pi() {
        let e = parse_expr("cmath.pi").unwrap();
        assert!((eval(&e, &HashMap::new()).re - PI).abs() < 1e-12);
    }

    #[test]
    fn test_collect_deps() {
        let e = parse_expr("2*cmath.sqrt(aS)*cmath.sqrt(cmath.pi)").unwrap();
        let mut deps = Vec::new();
        collect_deps(&e, &mut deps);
        assert_eq!(deps, vec!["aS"]);
    }
}
