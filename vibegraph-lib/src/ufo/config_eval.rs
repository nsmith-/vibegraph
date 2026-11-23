//! The part of Python a UFO model uses to switch on its `configuration.py`
//! flags, evaluated against that module's values.
//!
//! MadGraph imports `parameters.py` and `couplings.py` as Python modules, so a
//! model may write `if configuration.flag:` around a definition, or
//! `'A' if configuration.flag else 'B'` as a value. The loader does not run
//! Python: it evaluates this subset and refuses anything else by name.
//!
//! The subset is integer-valued (`True` is 1, `False` is 0) and is the forms
//! [`config_subset!`] lists, which every refusal of a form outside it quotes:
//! `and`/`or` follow Python's operand-returning semantics, `configuration.<name>`
//! resolves through any alias the file binds the module to, and, in
//! `configuration.py` itself, a bare name bound earlier is read too.

use std::collections::HashMap;

use rustpython_parser::ast;

use super::ast_util::{extract_int, extract_name, named_assignments};

/// The forms [`eval_int`] evaluates, as a refusal names them. A macro so that a
/// `&'static str` message can `concat!` it.
macro_rules! config_subset {
    () => {
        "integer and boolean literals, `configuration.<name>` for a name \
         configuration.py binds to one, unary `-`, `+` and `not`, binary `+`, `-` \
         and `*`, `and`, `or`, and `A if T else B`"
    };
}
pub(crate) use config_subset;

/// The module-level names a model's `configuration.py` binds, as integers
/// (`True` is 1): the switches a definition or value may test.
pub(crate) type Configuration = HashMap<String, i64>;

/// Where a name or `module.name` reference resolves.
pub(crate) struct Scope<'a> {
    /// The values `configuration.py` binds.
    pub(crate) names: &'a Configuration,
    /// The local names the file binds to the `configuration` module.
    pub(crate) modules: &'a [String],
    /// Whether a bare name reads `names` (true inside `configuration.py`).
    pub(crate) bare: bool,
}

/// The local names a file binds to the `configuration` module
/// (`import configuration [as c]`).
pub(crate) fn configuration_modules(stmts: &[ast::Stmt]) -> Vec<String> {
    stmts
        .iter()
        .filter_map(|stmt| match stmt {
            ast::Stmt::Import(ast::StmtImport { names, .. }) => Some(names),
            _ => None,
        })
        .flatten()
        .filter(|alias| alias.name.as_str() == "configuration")
        .map(|alias| alias.asname.as_ref().unwrap_or(&alias.name).to_string())
        .collect()
}

/// The integer `expr` evaluates to within the subset, or `None` outside it
/// (or on a name the scope does not bind).
pub(crate) fn eval_int(expr: &ast::Expr, scope: &Scope<'_>) -> Option<i64> {
    let recurse = |e: &ast::Expr| eval_int(e, scope);
    match expr {
        ast::Expr::Constant(ast::ExprConstant {
            value: ast::Constant::Bool(b),
            ..
        }) => Some(i64::from(*b)),
        ast::Expr::Constant(_) => extract_int(expr),
        ast::Expr::UnaryOp(ast::ExprUnaryOp { op, operand, .. }) => match op {
            ast::UnaryOp::Not => Some(i64::from(recurse(operand)? == 0)),
            ast::UnaryOp::USub => recurse(operand)?.checked_neg(),
            ast::UnaryOp::UAdd => recurse(operand),
            _ => None,
        },
        ast::Expr::BinOp(ast::ExprBinOp {
            left, op, right, ..
        }) => {
            let (l, r) = (recurse(left)?, recurse(right)?);
            match op {
                ast::Operator::Add => l.checked_add(r),
                ast::Operator::Sub => l.checked_sub(r),
                ast::Operator::Mult => l.checked_mul(r),
                _ => None,
            }
        }
        ast::Expr::BoolOp(ast::ExprBoolOp { op, values, .. }) => {
            let mut result = 0;
            for (i, value) in values.iter().enumerate() {
                result = recurse(value)?;
                let stop = match op {
                    ast::BoolOp::And => result == 0,
                    ast::BoolOp::Or => result != 0,
                };
                if stop && i + 1 < values.len() {
                    return Some(result);
                }
            }
            Some(result)
        }
        ast::Expr::IfExp(ast::ExprIfExp {
            test, body, orelse, ..
        }) => {
            if recurse(test)? != 0 {
                recurse(body)
            } else {
                recurse(orelse)
            }
        }
        ast::Expr::Attribute(ast::ExprAttribute { value, attr, .. }) => {
            let module = extract_name(value)?;
            scope
                .modules
                .iter()
                .any(|a| a == module)
                .then(|| scope.names.get(attr.as_str()).copied())?
        }
        ast::Expr::Name(ast::ExprName { id, .. }) => {
            scope.bare.then(|| scope.names.get(id.as_str()).copied())?
        }
        _ => None,
    }
}

/// Read `configuration.py`: each module-level assignment the subset evaluates
/// binds its name, in order, so a later one may read an earlier. An assignment
/// outside the subset binds nothing, and a definition reading it is refused.
pub(crate) fn read_configuration(stmts: &[ast::Stmt]) -> Configuration {
    let mut names = Configuration::new();
    for (name, value) in named_assignments(stmts) {
        let scope = Scope {
            names: &names,
            modules: &[],
            bare: true,
        };
        if let Some(v) = eval_int(value, &scope) {
            names.insert(name.to_owned(), v);
        }
    }
    names
}

/// The branch of a conditional expression chain that runs: `expr` itself when
/// it is not one.
pub(crate) fn select_branch<'a>(
    mut expr: &'a ast::Expr,
    scope: &Scope<'_>,
) -> Result<&'a ast::Expr, ConditionOutside> {
    while let ast::Expr::IfExp(ast::ExprIfExp {
        test, body, orelse, ..
    }) = expr
    {
        expr = if eval_int(test, scope).ok_or(ConditionOutside)? != 0 {
            body
        } else {
            orelse
        };
    }
    Ok(expr)
}

/// A condition the subset does not evaluate.
#[derive(Debug)]
pub(crate) struct ConditionOutside;

/// The module's statements with each `if`/`elif`/`else` replaced by the branch
/// its test selects, at any depth: the statements Python would run.
pub(crate) fn flatten_branches(
    stmts: &[ast::Stmt],
    scope: &Scope<'_>,
) -> Result<Vec<ast::Stmt>, ConditionOutside> {
    let mut out = Vec::with_capacity(stmts.len());
    for stmt in stmts {
        match stmt {
            ast::Stmt::If(ast::StmtIf {
                test, body, orelse, ..
            }) => {
                let chosen = if eval_int(test, scope).ok_or(ConditionOutside)? != 0 {
                    body
                } else {
                    orelse
                };
                out.extend(flatten_branches(chosen, scope)?);
            }
            other => out.push(other.clone()),
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ufo::ast_util::parse_stmts;

    fn config(src: &str) -> Configuration {
        read_configuration(&parse_stmts(src).unwrap())
    }

    fn eval(src: &str, names: &Configuration) -> Option<i64> {
        let stmts = parse_stmts(&format!("x = {src}")).unwrap();
        let (_, expr) = named_assignments(&stmts).next().unwrap();
        let modules = vec!["configuration".to_owned()];
        eval_int(
            expr,
            &Scope {
                names,
                modules: &modules,
                bare: false,
            },
        )
    }

    #[test]
    fn configuration_assignments_read_earlier_ones() {
        let c =
            config("a = True\nb = 1 if a else 0\nc = not a\nd = 2 * 3 - 1\ne = 'text'\nf = g\n");
        assert_eq!(c["a"], 1);
        assert_eq!(c["b"], 1);
        assert_eq!(c["c"], 0);
        assert_eq!(c["d"], 5);
        assert!(!c.contains_key("e") && !c.contains_key("f"));
    }

    #[test]
    fn the_subset_matches_python() {
        let c = config("t = True\nf = False\n");
        let cases = [
            ("configuration.t and configuration.f", 0),
            ("configuration.f or 7", 7),
            ("configuration.t and 4", 4),
            // Python stops at the first operand that decides: not the last one.
            ("configuration.f and 4", 0),
            ("configuration.t or 9", 1),
            ("3 if configuration.f else 4 if configuration.t else 5", 4),
            ("0 + configuration.t", 1),
            ("not configuration.f", 1),
            ("-configuration.t", -1),
        ];
        for (src, expected) in cases {
            assert_eq!(eval(src, &c), Some(expected), "{src}");
        }
        for outside in [
            "configuration.missing",
            "configuration.t / 2",
            "other.t",
            "x",
        ] {
            assert_eq!(eval(outside, &c), None, "{outside}");
        }
    }

    #[test]
    fn branches_flatten_to_the_ones_that_run() {
        let c = config("a = False\nb = True\n");
        let modules = vec!["configuration".to_owned()];
        let scope = Scope {
            names: &c,
            modules: &modules,
            bare: false,
        };
        let stmts = parse_stmts(
            "if configuration.a:\n    z = 9\nif configuration.a:\n    p = 1\nelif configuration.b:\n    if configuration.b:\n        q = 2\n    else:\n        r = 3\nelse:\n    s = 4\n",
        )
        .unwrap();
        let flat = flatten_branches(&stmts, &scope).unwrap();
        let names: Vec<&str> = named_assignments(&flat).map(|(n, _)| n).collect();
        assert_eq!(names, ["q"]);
        let unbound = parse_stmts("if configuration.nope:\n    p = 1\n").unwrap();
        assert!(flatten_branches(&unbound, &scope).is_err());
    }
}
