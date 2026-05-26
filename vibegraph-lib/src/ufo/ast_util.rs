use rustpython_parser::ast;
use rustpython_parser::{parse, Mode, ParseError};

/// Parse Python source into a list of top-level statements.
pub fn parse_stmts(src: &str) -> Result<Vec<ast::Stmt>, ParseError> {
    match parse(src, Mode::Module, "<ufo>")? {
        ast::Mod::Module(ast::ModModule { body, .. }) => Ok(body),
        _ => Ok(vec![]),
    }
}

/// Extract a string constant from an expression.
pub fn extract_str(expr: &ast::Expr) -> Option<&str> {
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
pub fn extract_int(expr: &ast::Expr) -> Option<i64> {
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

/// Extract a float/int constant from an expression, handling unary negation.
pub fn extract_float(expr: &ast::Expr) -> Option<f64> {
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
        _ => None,
    }
}

/// Extract a bare identifier name from an expression.
pub fn extract_name(expr: &ast::Expr) -> Option<&str> {
    if let ast::Expr::Name(ast::ExprName { id, .. }) = expr {
        Some(id.as_str())
    } else {
        None
    }
}

/// Extract `(object, attribute)` from an `Obj.attr` expression.
pub fn extract_attr(expr: &ast::Expr) -> Option<(&str, &str)> {
    if let ast::Expr::Attribute(ast::ExprAttribute { value, attr, .. }) = expr {
        let obj = extract_name(value)?;
        Some((obj, attr.as_str()))
    } else {
        None
    }
}

/// Find a keyword argument by name in a keyword list.
pub fn get_kwarg<'a>(kws: &'a [ast::Keyword], name: &str) -> Option<&'a ast::Expr> {
    kws.iter()
        .find(|kw| kw.arg.as_deref().map(|a| a == name).unwrap_or(false))
        .map(|kw| &kw.value)
}

/// Get a keyword argument as a string.
pub fn kwarg_str(kws: &[ast::Keyword], name: &str) -> Option<String> {
    extract_str(get_kwarg(kws, name)?).map(|s| s.to_owned())
}

/// Get a keyword argument as an integer.
pub fn kwarg_int(kws: &[ast::Keyword], name: &str) -> Option<i64> {
    extract_int(get_kwarg(kws, name)?)
}

/// Get a keyword argument as a float (accepts int literals too).
pub fn kwarg_float(kws: &[ast::Keyword], name: &str) -> Option<f64> {
    extract_float(get_kwarg(kws, name)?)
}

/// Get the function name from a Call expression (handles bare names and `mod.name` attributes).
pub fn call_func_name(expr: &ast::Expr) -> Option<&str> {
    match expr {
        ast::Expr::Name(ast::ExprName { id, .. }) => Some(id.as_str()),
        ast::Expr::Attribute(ast::ExprAttribute { attr, .. }) => Some(attr.as_str()),
        _ => None,
    }
}
