use rustpython_parser::ast;
use rustpython_parser::{parse, Mode, ParseError};

/// Parse Python source into a list of top-level statements.
pub(crate) fn parse_stmts(src: &str) -> Result<Vec<ast::Stmt>, ParseError> {
    match parse(src, Mode::Module, "<ufo>")? {
        ast::Mod::Module(ast::ModModule { body, .. }) => Ok(body),
        _ => Ok(vec![]),
    }
}

/// The module-level `NAME = value` assignments of a UFO file, in file order, as
/// `(NAME, value)`. An assignment to anything but a bare name (loop_sm's
/// `b.counterterm = ...`) is skipped.
pub(crate) fn named_assignments(stmts: &[ast::Stmt]) -> impl Iterator<Item = (&str, &ast::Expr)> {
    stmts.iter().filter_map(|stmt| {
        let ast::Stmt::Assign(ast::StmtAssign { targets, value, .. }) = stmt else {
            return None;
        };
        let ast::Expr::Name(ast::ExprName { id, .. }) = targets.first()? else {
            return None;
        };
        Some((id.as_str(), value.as_ref()))
    })
}

/// The module-level `NAME = Ctor(...)` assignments of a UFO file for one
/// constructor name `ctor` (bare or `module.Ctor`), in file order, as
/// `(NAME, keyword arguments)`.
pub(crate) fn constructor_calls<'a>(
    stmts: &'a [ast::Stmt],
    ctor: &'a str,
) -> impl Iterator<Item = (&'a str, &'a [ast::Keyword])> + 'a {
    named_assignments(stmts).filter_map(move |(name, value)| {
        let ast::Expr::Call(ast::ExprCall { func, keywords, .. }) = value else {
            return None;
        };
        (call_func_name(func) == Some(ctor)).then_some((name, keywords.as_slice()))
    })
}

/// Extract a string constant from an expression.
pub(crate) fn extract_str(expr: &ast::Expr) -> Option<&str> {
    if let ast::Expr::Constant(ast::ExprConstant {
        value: ast::Constant::Str(s),
        ..
    }) = expr
    {
        Some(s.as_str())
    } else {
        None
    }
}

/// Extract an integer constant from an expression, handling unary negation.
pub(crate) fn extract_int(expr: &ast::Expr) -> Option<i64> {
    use num_traits::ToPrimitive;
    match expr {
        ast::Expr::Constant(ast::ExprConstant {
            value: ast::Constant::Int(i),
            ..
        }) => i.to_i64(),
        ast::Expr::UnaryOp(ast::ExprUnaryOp {
            op: ast::UnaryOp::USub,
            operand,
            ..
        }) => extract_int(operand).map(|n| -n),
        _ => None,
    }
}

/// Extract a float/int constant from an expression, handling unary negation
/// and simple binary arithmetic (including fractional charges like `2/3`).
pub(crate) fn extract_float(expr: &ast::Expr) -> Option<f64> {
    match expr {
        ast::Expr::Constant(ast::ExprConstant {
            value: ast::Constant::Float(f),
            ..
        }) => Some(*f),
        ast::Expr::Constant(ast::ExprConstant {
            value: ast::Constant::Int(_),
            ..
        }) => extract_int(expr).map(|n| n as f64),
        ast::Expr::UnaryOp(ast::ExprUnaryOp {
            op: ast::UnaryOp::USub,
            operand,
            ..
        }) => extract_float(operand).map(|f| -f),
        ast::Expr::BinOp(ast::ExprBinOp {
            left, op, right, ..
        }) => {
            let l = extract_float(left)?;
            let r = extract_float(right)?;
            match op {
                ast::Operator::Div => Some(l / r),
                ast::Operator::Mult => Some(l * r),
                ast::Operator::Add => Some(l + r),
                ast::Operator::Sub => Some(l - r),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Extract a bare identifier name from an expression.
pub(crate) fn extract_name(expr: &ast::Expr) -> Option<&str> {
    if let ast::Expr::Name(ast::ExprName { id, .. }) = expr {
        Some(id.as_str())
    } else {
        None
    }
}

/// Extract `(object, attribute)` from an `Obj.attr` expression.
pub(crate) fn extract_attr(expr: &ast::Expr) -> Option<(&str, &str)> {
    if let ast::Expr::Attribute(ast::ExprAttribute { value, attr, .. }) = expr {
        let obj = extract_name(value)?;
        Some((obj, attr.as_str()))
    } else {
        None
    }
}

/// Find a keyword argument by name in a keyword list.
pub(crate) fn get_kwarg<'a>(kws: &'a [ast::Keyword], name: &str) -> Option<&'a ast::Expr> {
    kws.iter()
        .find(|kw| kw.arg.as_deref().map(|a| a == name).unwrap_or(false))
        .map(|kw| &kw.value)
}

/// Get a keyword argument as a string.
pub(crate) fn kwarg_str(kws: &[ast::Keyword], name: &str) -> Option<String> {
    extract_str(get_kwarg(kws, name)?).map(|s| s.to_owned())
}

/// Get a keyword argument as an integer.
pub(crate) fn kwarg_int(kws: &[ast::Keyword], name: &str) -> Option<i64> {
    extract_int(get_kwarg(kws, name)?)
}

/// Get a keyword argument as a float (accepts int literals too).
pub(crate) fn kwarg_float(kws: &[ast::Keyword], name: &str) -> Option<f64> {
    extract_float(get_kwarg(kws, name)?)
}

/// Get a keyword argument as a boolean (`True`/`False` literals).
pub(crate) fn kwarg_bool(kws: &[ast::Keyword], name: &str) -> Option<bool> {
    match get_kwarg(kws, name)? {
        ast::Expr::Constant(ast::ExprConstant {
            value: ast::Constant::Bool(b),
            ..
        }) => Some(*b),
        _ => None,
    }
}

/// Get the function name from a Call expression (handles bare names and `mod.name` attributes).
pub(crate) fn call_func_name(expr: &ast::Expr) -> Option<&str> {
    match expr {
        ast::Expr::Name(ast::ExprName { id, .. }) => Some(id.as_str()),
        ast::Expr::Attribute(ast::ExprAttribute { attr, .. }) => Some(attr.as_str()),
        _ => None,
    }
}
