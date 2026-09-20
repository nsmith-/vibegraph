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
                BinOp::Pow => pow(l, r),
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

/// `base ** exp`, keeping a negative real base off the complex branch.
///
/// `powc` evaluates `exp(exp·log base)`, and `log(-x) = ln x + iπ`, so a
/// negative real base raised to an integral power comes back with `sin(nπ)`
/// left on the imaginary part — a spurious ~2.4e-16 relative imaginary
/// component on a quantity Python computes in real arithmetic. Python's float
/// power keeps `(-x)**n` real for integral `n` and takes the principal branch
/// (`arg = +π`) otherwise; both are reproduced here. A non-negative real base
/// is left on the complex path, where the polar form already agrees with the
/// real power to within the last bits of `exp` and `log`.
fn pow(base: Complex64, exp: Complex64) -> Complex64 {
    if base.im == 0.0 && base.re < 0.0 && exp.im == 0.0 {
        if exp.re.fract() == 0.0 {
            return Complex64::new(base.re.powf(exp.re), 0.0);
        }
        // Rebuilding the base discards the negative zero `-x` leaves on its
        // imaginary part, which would otherwise select the conjugate branch.
        return Complex64::new(base.re, 0.0).powc(exp);
    }
    base.powc(exp)
}

peg::parser! {
    /// PEG grammar for UFO expression strings.
    ///
    /// Precedence (low to high): additive, multiplicative, unary, power, primary.
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
            = l:unary() rest:(_ op:mulop() _ r:unary() {(op, r)})* {
                rest.into_iter().fold(l, |acc, (op, r)| Expr::BinOp(op, Box::new(acc), Box::new(r)))
            }

        rule mulop() -> BinOp
            = "*" !"*" { BinOp::Mul }
            / "/" { BinOp::Div }

        // A sign binds looser than `**` on the left, as Python's
        // `factor: ('+'|'-') factor | power` does: `-a**2` is `-(a**2)`.
        rule unary() -> Expr
            = "-" _ e:unary() { Expr::Neg(Box::new(e)) }
            / "+" _ e:unary() { e }
            / power()

        // Right-associative, and the exponent is itself a signed factor
        // (Python's `power: primary ['**' factor]`), so `a**-b` parses.
        rule power() -> Expr
            = base:primary() _ "**" _ exp:unary() { Expr::BinOp(BinOp::Pow, Box::new(base), Box::new(exp)) }
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

    /// Equality up to the last few bits: raising a non-negative real base goes
    /// through the complex polar form, which costs a couple of ulp per `**`.
    #[track_caller]
    fn assert_close(got: Complex64, want: Complex64) {
        let tol = 8.0 * f64::EPSILON * want.norm().max(1.0);
        assert!(
            (got - want).norm() <= tol,
            "{got} is not {want} within {tol:e}"
        );
    }

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
    fn a_sign_binds_looser_than_exponentiation() {
        let p = params(&[("a", 3.0)]);
        assert_eq!(parse_expr("-a**2").unwrap(), parse_expr("-(a**2)").unwrap());
        assert_ne!(parse_expr("-a**2").unwrap(), parse_expr("(-a)**2").unwrap());
        assert_close(
            eval(&parse_expr("-a**2").unwrap(), &p),
            Complex64::new(-9.0, 0.0),
        );
        assert_close(
            eval(&parse_expr("(-a)**2").unwrap(), &p),
            Complex64::new(9.0, 0.0),
        );
        assert_close(
            eval(&parse_expr("+a**2").unwrap(), &p),
            Complex64::new(9.0, 0.0),
        );
        // The sign is looser than `**` but tighter than `*`, so a leading minus
        // negates the whole product's first factor only.
        assert_close(
            eval(&parse_expr("-a**2*a").unwrap(), &p),
            Complex64::new(-27.0, 0.0),
        );
    }

    #[test]
    fn an_exponent_carries_its_own_sign() {
        let p = params(&[("a", 2.0), ("b", 3.0)]);
        assert_close(
            eval(&parse_expr("a**-b").unwrap(), &p),
            Complex64::new(0.125, 0.0),
        );
        assert_close(
            eval(&parse_expr("-a**-b").unwrap(), &p),
            Complex64::new(-0.125, 0.0),
        );
        // `10**-40` is how SMEFTsim's parameters guard a division by a
        // possibly-zero Yukawa.
        assert_close(
            eval(&parse_expr("10**-40").unwrap(), &HashMap::new()),
            Complex64::new(1e-40, 0.0),
        );
    }

    #[test]
    fn exponentiation_is_right_associative() {
        // 512, not the 64 a left-associative reading would give.
        assert_close(
            eval(&parse_expr("2**3**2").unwrap(), &HashMap::new()),
            Complex64::new(512.0, 0.0),
        );
        assert_close(
            eval(&parse_expr("2**-3**2").unwrap(), &HashMap::new()),
            Complex64::new(2f64.powi(-9), 0.0),
        );
    }

    #[test]
    fn a_negative_real_base_raised_to_an_integer_power_stays_real() {
        let p = params(&[("a", 3.0)]);
        assert_eq!(
            eval(&parse_expr("(-a)**2").unwrap(), &p),
            Complex64::new(9.0, 0.0)
        );
        assert_eq!(
            eval(&parse_expr("(-a)**3").unwrap(), &p),
            Complex64::new(-27.0, 0.0)
        );
        // A negative base with a fractional exponent does leave the reals, and
        // there the complex branch is the answer Python gives too.
        let root = eval(&parse_expr("(-a)**0.5").unwrap(), &p);
        assert!(root.im > 0.0 && (root.im - 3f64.sqrt()).abs() < 1e-15);
    }

    /// The three expressions in the models this repository loads whose value
    /// depends on a sign binding looser than `**`: the Standard Model's `GC_7`
    /// and `GC_54`, and the `cbWRe` term of SMEFTsim's `dWT`.
    #[test]
    fn model_expressions_read_as_python_reads_them() {
        let p = params(&[
            ("ee", 0.3079537672443688),
            ("cw", 0.875391102200322),
            ("sw", 0.4834153681757587),
            ("cbWRe", 0.008),
            ("MB", 4.7),
            ("MT", 173.0),
            ("MWsm", 79.82436),
        ]);
        for (expr, parenthesized) in [
            ("-ee**2/(2.*cw)", "-(ee**2)/(2.*cw)"),
            ("-ee**2/(2.*sw)", "-(ee**2)/(2.*sw)"),
            (
                "cbWRe*MB*(-MB**2 + MT**2 + MWsm**2)",
                "cbWRe*MB*(-(MB**2) + MT**2 + MWsm**2)",
            ),
        ] {
            let ours = eval(&parse_expr(expr).unwrap(), &p);
            assert_eq!(
                ours,
                eval(&parse_expr(parenthesized).unwrap(), &p),
                "{expr}"
            );
            assert_eq!(ours.im, 0.0, "{expr} is real");
        }
        // The Standard Model's own values on its default card: negative, where
        // reading the sign as part of the base would make them positive.
        let gc_7 = eval(&parse_expr("-ee**2/(2.*cw)").unwrap(), &p);
        let gc_54 = eval(&parse_expr("-ee**2/(2.*sw)").unwrap(), &p);
        assert_close(gc_7, Complex64::new(-0.05416751582328568, 0.0));
        assert_close(gc_54, Complex64::new(-0.0980890648117737, 0.0));
        let d_wt = eval(
            &parse_expr("cbWRe*MB*(-MB**2 + MT**2 + MWsm**2)").unwrap(),
            &p,
        );
        assert_close(d_wt, Complex64::new(1364.0843256978012, 0.0));
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
