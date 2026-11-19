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
    /// Python's `then if cond else otherwise`, as `(cond, then, otherwise)`;
    /// only the selected branch is evaluated.
    Cond(Box<Expr>, Box<Expr>, Box<Expr>),
    /// A comparison, `1` when it holds and `0` otherwise.
    Cmp(CmpOp, Box<Expr>, Box<Expr>),
    /// `and`, `or` and `not`.
    Logic(LogicOp, Vec<Expr>),
}

impl Expr {
    /// The sub-expressions of the forms that select between values or test
    /// them (`Cond`, `Cmp`, `Logic`); empty for every other form.
    pub(crate) fn test_operands(&self) -> Vec<&Expr> {
        match self {
            Expr::Cond(c, t, f) => vec![c, t, f],
            Expr::Cmp(_, l, r) => vec![l, r],
            Expr::Logic(_, args) => args.iter().collect(),
            _ => Vec::new(),
        }
    }
}

/// A Python comparison operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

/// A Python boolean operator. `And` and `Or` return one of their operands, as
/// Python's do; `Not` returns `1` or `0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogicOp {
    And,
    Or,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

/// A function a UFO value string may call: a `cmath` function or a name the
/// model's `function_library.py` defines.
///
/// The library functions are evaluated as the SM UFO's `function_library.py`
/// writes them, on the complex argument (`sec(z) = 1/cos(z)`, `asec(z) =
/// acos(1/z)`), until the model's own library says otherwise: SMEFTsim's takes
/// `z.real` in `sec`, `csc`, `asec` and `acsc`, which
/// [`FunctionLibrary::resolve`](super::functions::FunctionLibrary::resolve)
/// selects as the `...Re` variants. Library functions with no variant here
/// (`cot`, `theta_function`, `cond`, `reglog`) are refused when the value
/// string is parsed.
///
/// New variants go at the end: the interned SM model stores these by index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Func {
    Sqrt,
    Log,
    Exp,
    Abs,
    Arg,
    /// `complexconjugate`, `conj`
    Conj,
    Re,
    Im,
    /// `1/cos(z)`
    Sec,
    /// `1/sin(z)`
    Csc,
    /// `acos(1/z)`
    ASec,
    /// `asin(1/z)`
    ACsc,
    Sin,
    Cos,
    Tan,
    ATan,
    ASin,
    ACos,
    /// Python's `max(a, b, ...)`: the argument largest in its real part.
    Max,
    /// Python's `min(a, b, ...)`: the argument smallest in its real part.
    Min,
    Log10,
    Sinh,
    Cosh,
    Tanh,
    ASinh,
    ACosh,
    ATanh,
    /// `sec` on `z.real`
    SecRe,
    /// `csc` on `z.real`
    CscRe,
    /// `asec` on `z.real`
    ASecRe,
    /// `acsc` on `z.real`
    ACscRe,
}

impl Func {
    /// The function a library `name` refers to, when [`FunctionLibrary`] governs it.
    ///
    /// [`FunctionLibrary`]: super::functions::FunctionLibrary
    pub(crate) fn from_library_name(name: &str) -> Option<Func> {
        match name {
            "sec" => Some(Func::Sec),
            "csc" => Some(Func::Csc),
            "asec" => Some(Func::ASec),
            "acsc" => Some(Func::ACsc),
            _ => None,
        }
    }

    /// The variant that reads the real part of its argument.
    pub(crate) fn real_part_variant(self) -> Option<Func> {
        match self {
            Func::Sec => Some(Func::SecRe),
            Func::Csc => Some(Func::CscRe),
            Func::ASec => Some(Func::ASecRe),
            Func::ACsc => Some(Func::ACscRe),
            _ => None,
        }
    }
}

impl Expr {
    /// The expression with every call's function replaced by `f` of it.
    pub(crate) fn map_calls(self, f: &dyn Fn(Func) -> Func) -> Expr {
        let go = |e: Expr| e.map_calls(f);
        let boxed = |e: Box<Expr>| Box::new(go(*e));
        match self {
            Expr::Call(func, args) => Expr::Call(f(func), args.into_iter().map(go).collect()),
            Expr::BinOp(op, l, r) => Expr::BinOp(op, boxed(l), boxed(r)),
            Expr::Neg(e) => Expr::Neg(boxed(e)),
            Expr::Complex(re, im) => Expr::Complex(boxed(re), boxed(im)),
            Expr::Cond(c, t, o) => Expr::Cond(boxed(c), boxed(t), boxed(o)),
            Expr::Cmp(op, l, r) => Expr::Cmp(op, boxed(l), boxed(r)),
            Expr::Logic(op, args) => Expr::Logic(op, args.into_iter().map(go).collect()),
            leaf @ (Expr::Num(_) | Expr::Pi | Expr::Param(_)) => leaf,
        }
    }
}

/// Collect all parameter names referenced in an expression.
pub(crate) fn collect_deps(expr: &Expr, deps: &mut Vec<String>) {
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
        Expr::Cond(..) | Expr::Cmp(..) | Expr::Logic(..) => {
            for operand in expr.test_operands() {
                collect_deps(operand, deps);
            }
        }
    }
}

/// Evaluate an expression given a map of parameter name → complex value.
///
/// A reference to a parameter missing from `params` panics: an internal
/// invariant, not an input check. Every path from a public entry point supplies
/// the model's own expressions and a map holding every parameter the model
/// defines, and the guards are:
///
/// * `parse_parameters` refuses an internal expression naming an undefined
///   parameter, and any `Parameter` defined under an `if`;
/// * model load refuses a coupling naming an undefined parameter, and any
///   particle, Lorentz structure, coupling, vertex, coupling order or
///   propagator defined under an `if`;
/// * `ParameterSet::evaluate` inserts every external and internal parameter
///   (restriction-zeroed ones as `0.0`), so `EvaluatedModel::from_model_card`,
///   `from_model` and the restriction pruning see a complete map;
/// * `EvaluatedModel::recompute` and `set_alpha_s` check the caller's parameter
///   name and return `UnknownParameter` before evaluating anything.
pub(crate) fn eval(expr: &Expr, params: &HashMap<String, Complex64>) -> Complex64 {
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
                Func::ASin => a(0).asin(),
                Func::ACos => a(0).acos(),
                Func::SecRe => Complex64::new(1.0, 0.0) / Complex64::new(a(0).re, 0.0).cos(),
                Func::CscRe => Complex64::new(1.0, 0.0) / Complex64::new(a(0).re, 0.0).sin(),
                Func::ASecRe => (Complex64::new(1.0, 0.0) / Complex64::new(a(0).re, 0.0)).acos(),
                Func::ACscRe => (Complex64::new(1.0, 0.0) / Complex64::new(a(0).re, 0.0)).asin(),
                Func::Log10 => a(0).log(10.0),
                Func::Sinh => a(0).sinh(),
                Func::Cosh => a(0).cosh(),
                Func::Tanh => a(0).tanh(),
                Func::ASinh => a(0).asinh(),
                Func::ACosh => a(0).acosh(),
                Func::ATanh => a(0).atanh(),
                Func::Max | Func::Min => {
                    let pick_larger = matches!(func, Func::Max);
                    args.iter()
                        .map(|e| eval(e, params))
                        .reduce(|best, v| {
                            // A later argument replaces the held one only when it is
                            // strictly better, as Python's `max`/`min` keep the first.
                            if (pick_larger && v.re > best.re) || (!pick_larger && v.re < best.re) {
                                v
                            } else {
                                best
                            }
                        })
                        .expect("max and min parse with at least two arguments")
                }
            }
        }
        Expr::Complex(re, im) => Complex64::new(eval(re, params).re, eval(im, params).re),
        Expr::Cond(cond, then, otherwise) => {
            if truthy(eval(cond, params)) {
                eval(then, params)
            } else {
                eval(otherwise, params)
            }
        }
        Expr::Cmp(op, lhs, rhs) => {
            let (l, r) = (eval(lhs, params), eval(rhs, params));
            let holds = match op {
                CmpOp::Eq => l == r,
                CmpOp::Ne => l != r,
                CmpOp::Lt => l.re < r.re,
                CmpOp::Le => l.re <= r.re,
                CmpOp::Gt => l.re > r.re,
                CmpOp::Ge => l.re >= r.re,
            };
            from_bool(holds)
        }
        Expr::Logic(LogicOp::Not, args) => from_bool(!truthy(eval(&args[0], params))),
        Expr::Logic(LogicOp::And, args) => {
            let left = eval(&args[0], params);
            if truthy(left) {
                eval(&args[1], params)
            } else {
                left
            }
        }
        Expr::Logic(LogicOp::Or, args) => {
            let left = eval(&args[0], params);
            if truthy(left) {
                left
            } else {
                eval(&args[1], params)
            }
        }
    }
}

/// Python's truth of a number: anything but zero.
fn truthy(z: Complex64) -> bool {
    z != Complex64::new(0.0, 0.0)
}

fn from_bool(b: bool) -> Complex64 {
    Complex64::new(if b { 1.0 } else { 0.0 }, 0.0)
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

/// The function a call names, with its arguments, if the name is one `cmath`
/// (`cmath_prefixed`) or a UFO value string may use and the argument count fits.
fn call(name: &str, cmath_prefixed: bool, args: Vec<Expr>) -> Result<Expr, &'static str> {
    // (function, bare spelling allowed, cmath spelling allowed)
    let (func, bare, cmath) = match name {
        "sqrt" => (Func::Sqrt, true, true),
        "log" => (Func::Log, false, true),
        "exp" => (Func::Exp, false, true),
        "sin" => (Func::Sin, true, true),
        "cos" => (Func::Cos, true, true),
        "tan" => (Func::Tan, true, true),
        "atan" => (Func::ATan, true, true),
        "asin" => (Func::ASin, true, true),
        "acos" => (Func::ACos, true, true),
        "log10" => (Func::Log10, false, true),
        "sinh" => (Func::Sinh, false, true),
        "cosh" => (Func::Cosh, false, true),
        "tanh" => (Func::Tanh, false, true),
        "asinh" => (Func::ASinh, false, true),
        "acosh" => (Func::ACosh, false, true),
        "atanh" => (Func::ATanh, false, true),
        "complexconjugate" | "conj" => (Func::Conj, true, false),
        "abs" => (Func::Abs, true, false),
        "arg" => (Func::Arg, true, false),
        "re" => (Func::Re, true, false),
        "im" => (Func::Im, true, false),
        "asec" => (Func::ASec, true, false),
        "acsc" => (Func::ACsc, true, false),
        "sec" => (Func::Sec, true, false),
        "csc" => (Func::Csc, true, false),
        "max" => (Func::Max, true, false),
        "min" => (Func::Min, true, false),
        _ => return Err("a function this loader evaluates"),
    };
    if (cmath_prefixed && !cmath) || (!cmath_prefixed && !bare) {
        return Err("a function this loader evaluates");
    }
    let fits = match func {
        Func::Max | Func::Min => args.len() >= 2,
        _ => args.len() == 1,
    };
    if !fits {
        return Err("the function's argument count");
    }
    Ok(Expr::Call(func, args))
}

peg::parser! {
    /// PEG grammar for UFO expression strings.
    ///
    /// Precedence (low to high): additive, multiplicative, unary, power, primary.
    pub(crate) grammar ufo_expr() for str {

        // Entry point: Python's conditional expression, the loosest form.
        pub(crate) rule expression() -> Expr = conditional()

        // `then if cond else otherwise`; the `else` arm is itself conditional,
        // so the form is right-associative, as in Python.
        rule conditional() -> Expr
            = then:disjunction() branch:(_ k_if() _ c:disjunction() _ k_else() _ f:conditional() {(c, f)})? {
                match branch {
                    Some((c, f)) => Expr::Cond(Box::new(c), Box::new(then), Box::new(f)),
                    None => then,
                }
            }

        rule disjunction() -> Expr
            = l:conjunction() rest:(_ k_or() _ r:conjunction() {r})* {
                rest.into_iter().fold(l, |acc, r| Expr::Logic(LogicOp::Or, vec![acc, r]))
            }

        rule conjunction() -> Expr
            = l:inversion() rest:(_ k_and() _ r:inversion() {r})* {
                rest.into_iter().fold(l, |acc, r| Expr::Logic(LogicOp::And, vec![acc, r]))
            }

        rule inversion() -> Expr
            = k_not() _ e:inversion() { Expr::Logic(LogicOp::Not, vec![e]) }
            / comparison()

        // One comparison: a chain (`a < b < c`) leaves input unread and is refused.
        rule comparison() -> Expr
            = l:additive() rest:(_ op:cmpop() _ r:additive() {(op, r)})? {
                match rest {
                    Some((op, r)) => Expr::Cmp(op, Box::new(l), Box::new(r)),
                    None => l,
                }
            }

        rule cmpop() -> CmpOp
            = "==" { CmpOp::Eq }
            / "!=" { CmpOp::Ne }
            / "<=" { CmpOp::Le }
            / ">=" { CmpOp::Ge }
            / "<"  { CmpOp::Lt }
            / ">"  { CmpOp::Gt }

        rule k_if() = "if" !ident_continue()
        rule k_else() = "else" !ident_continue()
        rule k_and() = "and" !ident_continue()
        rule k_or() = "or" !ident_continue()
        rule k_not() = "not" !ident_continue()
        rule keyword() = (k_if() / k_else() / k_and() / k_or() / k_not())

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
            = "(" _ e:expression() _ ")" { e }
            / float_literal()
            / complex_constructor()
            / "cmath.pi" !ident_continue() { Expr::Pi }
            / cmath_call()
            / func_call()
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
            = "complex(" _ re:expression() _ "," _ im:expression() _ ")" {
                Expr::Complex(Box::new(re), Box::new(im))
            }

        rule args() -> Vec<Expr>
            = "(" _ a:(expression() ** (_ "," _)) _ ")" { a }

        // cmath.func(args) — `cmath.pi` is matched earlier as a constant
        rule cmath_call() -> Expr
            = "cmath." name:ident() a:args() {? call(name, true, a) }

        // function_library and other bare function names
        rule func_call() -> Expr
            = name:ident() a:args() {? call(name, false, a) }

        rule ident_expr() -> Expr
            = !keyword() name:ident() { Expr::Param(name.to_owned()) }

        rule ident() -> &'input str
            = $(['a'..='z' | 'A'..='Z' | '_'] ident_continue()*)

        rule ident_continue() = ['a'..='z' | 'A'..='Z' | '0'..='9' | '_']

        rule _ = [' ' | '\t' | '\n' | '\r']*
    }
}

/// Parse a UFO expression string into an [`Expr`].
pub(crate) fn parse_expr(s: &str) -> Result<Expr, peg::error::ParseError<peg::str::LineCol>> {
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

    /// Every function spelling, bare and `cmath.`, against the value of the
    /// function it names, at arguments where swapping any two of them changes
    /// the value: `x` inside `[-1, 1]` for `asin`/`acos`, `y` outside it for
    /// `asec`/`acsc`, and a complex `z` for the functions that read only part
    /// of it.
    #[test]
    fn every_function_spelling_evaluates_the_function_it_names() {
        let (x, y) = (0.3_f64, 2.5_f64);
        let z = Complex64::new(0.4, -0.7);
        let mut values = params(&[("x", x), ("y", y)]);
        values.insert("z".to_owned(), z);
        let one = Complex64::new(1.0, 0.0);
        let r = |v: f64| Complex64::new(v, 0.0);
        let table: &[(&str, Complex64)] = &[
            ("cmath.sqrt(y)", r(y.sqrt())),
            ("sqrt(y)", r(y.sqrt())),
            ("cmath.log(y)", r(y.ln())),
            ("cmath.exp(x)", r(x.exp())),
            ("cmath.sin(x)", r(x.sin())),
            ("sin(x)", r(x.sin())),
            ("cmath.cos(x)", r(x.cos())),
            ("cos(x)", r(x.cos())),
            ("cmath.tan(x)", r(x.tan())),
            ("tan(x)", r(x.tan())),
            ("cmath.atan(y)", r(y.atan())),
            ("atan(y)", r(y.atan())),
            ("cmath.asin(x)", r(x.asin())),
            ("asin(x)", r(x.asin())),
            ("cmath.acos(x)", r(x.acos())),
            ("acos(x)", r(x.acos())),
            ("sec(x)", r(1.0 / x.cos())),
            ("csc(x)", r(1.0 / x.sin())),
            ("asec(y)", r((1.0 / y).acos())),
            ("acsc(y)", r((1.0 / y).asin())),
            ("sec(z)", one / z.cos()),
            ("abs(z)", r(z.norm())),
            ("arg(z)", r(z.arg())),
            ("re(z)", r(z.re)),
            ("im(z)", r(z.im)),
            ("conj(z)", z.conj()),
            ("complexconjugate(z)", z.conj()),
        ];
        for (text, want) in table {
            let e = parse_expr(text).unwrap_or_else(|err| panic!("{text}: {err}"));
            let got = eval(&e, &values);
            let tol = 8.0 * f64::EPSILON * want.norm().max(1.0);
            assert!((got - want).norm() <= tol, "{text} = {got}, not {want}");
        }
    }

    fn value(text: &str, pairs: &[(&str, f64)]) -> Complex64 {
        eval(
            &parse_expr(text).unwrap_or_else(|e| panic!("{text}: {e}")),
            &params(pairs),
        )
    }

    /// The value strings the CMS model store's `loop_qcd_qed_sm_Gmu` and DM
    /// models write, on both sides of each condition.
    #[test]
    fn conditional_expressions_select_a_branch() {
        let aew = "0.0 if aEW0M1==0. else 1./aEW0M1";
        assert_eq!(value(aew, &[("aEW0M1", 0.0)]), Complex64::new(0.0, 0.0));
        assert_eq!(value(aew, &[("aEW0M1", 125.0)]), Complex64::new(0.008, 0.0));
        // Right-associative chain, and the operators between `if` and `else`.
        let chain = "1 if a < 0 else 2 if a == 0 else 3";
        for (a, want) in [(-1.0, 1.0), (0.0, 2.0), (4.0, 3.0)] {
            assert_eq!(value(chain, &[("a", a)]).re, want, "a = {a}");
        }
        let logic = "10 if a > 1 and not b >= 2 or c != 0 else 20";
        for (a, b, c, want) in [
            (2.0, 1.0, 0.0, 10.0),
            (2.0, 3.0, 0.0, 20.0),
            (0.0, 0.0, 5.0, 10.0),
            (0.0, 0.0, 0.0, 20.0),
        ] {
            let got = value(logic, &[("a", a), ("b", b), ("c", c)]).re;
            assert_eq!(got, want, "a={a} b={b} c={c}");
        }
    }

    /// `and`/`or` return an operand and a sign binds tighter than a comparison,
    /// as in Python.
    #[test]
    fn boolean_operators_follow_python() {
        assert_eq!(value("a and b", &[("a", 3.0), ("b", 4.0)]).re, 4.0);
        assert_eq!(value("a and b", &[("a", 0.0), ("b", 4.0)]).re, 0.0);
        assert_eq!(value("a or b", &[("a", 3.0), ("b", 4.0)]).re, 3.0);
        assert_eq!(value("a or b", &[("a", 0.0), ("b", 4.0)]).re, 4.0);
        assert_eq!(value("-a < b", &[("a", 1.0), ("b", 0.0)]).re, 1.0);
        assert_eq!(value("1 + (a == 1)", &[("a", 1.0)]).re, 2.0);
    }

    /// The value written by DMPseudoScalar_MonoB for `WPhiee`, and `max`/`min`
    /// returning the first of equal arguments.
    #[test]
    fn max_min_and_the_dm_width_form() {
        let w = "max(1e-5, re(cmath.sqrt(1 - 4*m**2/s)*s/(8*cmath.pi)))";
        let (m, s) = (1.0_f64, 100.0_f64);
        let want = ((1.0 - 4.0 * m * m / s).sqrt() * s / (8.0 * PI)).max(1e-5);
        assert!((value(w, &[("m", m), ("s", s)]).re - want).abs() < 1e-14);
        assert_eq!(value(w, &[("m", 100.0), ("s", 100.0)]).re, 1e-5);
        assert_eq!(value("max(a, b, 7)", &[("a", 2.0), ("b", 9.0)]).re, 9.0);
        assert_eq!(value("min(a, b, 7)", &[("a", 2.0), ("b", 9.0)]).re, 2.0);
        assert_eq!(value("min(abs(a), 3)", &[("a", -5.0)]).re, 3.0);
        // Equal real parts: the first argument is kept, imaginary part and all.
        assert_eq!(value("max(complex(1, 2), complex(1, 3))", &[]).im, 2.0);
        assert_eq!(value("min(complex(1, 2), complex(1, 3))", &[]).im, 2.0);
    }

    #[test]
    fn the_hyperbolic_and_log10_cmath_functions_evaluate() {
        let x = 0.7_f64;
        for (text, want) in [
            ("cmath.log10(x)", x.log10()),
            ("cmath.sinh(x)", x.sinh()),
            ("cmath.cosh(x)", x.cosh()),
            ("cmath.tanh(x)", x.tanh()),
            ("cmath.asinh(x)", x.asinh()),
            ("cmath.acosh(1 + x)", (1.0 + x).acosh()),
            ("cmath.atanh(x)", x.atanh()),
        ] {
            let got = value(text, &[("x", x)]);
            assert!(
                (got.re - want).abs() < 1e-14 && got.im.abs() < 1e-14,
                "{text}"
            );
        }
    }

    /// Forms that are not Python, or not a value this loader evaluates, stay
    /// parse errors.
    #[test]
    fn malformed_conditionals_and_calls_are_refused() {
        for text in [
            "1 if a",
            "1 else 2",
            "1 if a else",
            "a < b < c",
            "max(a)",
            "max()",
            "sqrt(a, b)",
            "cmath.max(a, b)",
            "cmath.re(a)",
            "if",
            "a and",
            "cot(x)",
        ] {
            assert!(parse_expr(text).is_err(), "{text} parsed");
        }
    }

    /// A parameter whose name begins with a keyword is a parameter.
    #[test]
    fn names_that_start_with_a_keyword_are_parameters() {
        let p = params(&[
            ("order", 3.0),
            ("android", 5.0),
            ("iffy", 7.0),
            ("notx", 9.0),
        ]);
        for (name, want) in [
            ("order", 3.0),
            ("android", 5.0),
            ("iffy", 7.0),
            ("notx", 9.0),
        ] {
            assert_eq!(eval(&parse_expr(name).unwrap(), &p).re, want);
        }
    }

    /// `function_library.py` names with no evaluation here are refused, not read
    /// as a parameter.
    #[test]
    fn library_functions_without_an_evaluation_are_refused() {
        for text in [
            "cot(x)",
            "theta_function(x, y, z)",
            "cond(x, y, z)",
            "reglog(x)",
        ] {
            assert!(parse_expr(text).is_err(), "{text} parsed");
        }
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
