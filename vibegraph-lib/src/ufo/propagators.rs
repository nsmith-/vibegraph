//! `propagators.py`: UFO propagator forms a model attaches to individual particles.
//!
//! The numerator and denominator are kept as the verbatim strings the model
//! wrote them as. Nothing evaluates them: a particle carrying a custom
//! propagator is rejected when it actually propagates in a selected diagram
//! (see [`crate::diagrams`]), so the strings exist to identify and report the
//! form, not to compute with it.

use super::ast_util::{call_func_name, kwarg_str, parse_stmts};
use rustpython_parser::ast;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PropagatorError {
    #[error("Failed to parse propagators.py: {0}")]
    Parse(String),
    #[error("propagator '{name}': {field} is not a string expression this loader can read")]
    UnreadableForm { name: String, field: &'static str },
}

/// One `Propagator(...)` entry from `propagators.py`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Propagator {
    /// Python variable name, e.g. `"V1"`.
    pub python_name: String,
    /// UFO `name` field.
    pub name: String,
    /// Verbatim `numerator` string, with module-level string variables substituted.
    pub numerator: String,
    /// Verbatim `denominator` string, same treatment.
    pub denominator: String,
}

/// Parse `propagators.py` into its [`Propagator`] entries.
///
/// The shipped files build the forms by concatenating module-level string
/// variables (`denominatorSq = denominator + "**2"`), so string assignments are
/// tracked and `+` over strings is folded as the file goes.
pub fn parse_propagators(src: &str) -> Result<Vec<Propagator>, PropagatorError> {
    let stmts = parse_stmts(src).map_err(|e| PropagatorError::Parse(e.to_string()))?;
    let mut strings: HashMap<String, String> = HashMap::new();
    let mut result = Vec::new();

    for stmt in &stmts {
        let ast::Stmt::Assign(ast::StmtAssign { targets, value, .. }) = stmt else {
            continue;
        };
        let ast::Expr::Name(ast::ExprName { id, .. }) = targets.first().unwrap() else {
            continue;
        };
        let python_name = id.as_str().to_owned();

        match value.as_ref() {
            ast::Expr::Call(ast::ExprCall { func, keywords, .. })
                if call_func_name(func) == Some("Propagator") =>
            {
                let name = kwarg_str(keywords, "name").unwrap_or_else(|| python_name.clone());
                let field = |kw: &'static str| -> Result<String, PropagatorError> {
                    super::ast_util::get_kwarg(keywords, kw)
                        .and_then(|e| fold_string(e, &strings))
                        .ok_or(PropagatorError::UnreadableForm {
                            name: name.clone(),
                            field: kw,
                        })
                };
                let numerator = field("numerator")?;
                let denominator = field("denominator")?;
                result.push(Propagator {
                    python_name,
                    name,
                    numerator,
                    denominator,
                });
            }
            other => {
                if let Some(s) = fold_string(other, &strings) {
                    strings.insert(python_name, s);
                }
            }
        }
    }

    Ok(result)
}

/// Evaluate a string-valued expression: a literal, a module-level string
/// variable, or a `+` chain of those. Anything else yields `None`.
fn fold_string(expr: &ast::Expr, strings: &HashMap<String, String>) -> Option<String> {
    match expr {
        ast::Expr::Constant(ast::ExprConstant {
            value: ast::Constant::Str(s),
            ..
        }) => Some(s.clone()),
        ast::Expr::Name(ast::ExprName { id, .. }) => strings.get(id.as_str()).cloned(),
        ast::Expr::BinOp(ast::ExprBinOp {
            left,
            op: ast::Operator::Add,
            right,
            ..
        }) => {
            let l = fold_string(left, strings)?;
            let r = fold_string(right, strings)?;
            Some(l + &r)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
from object_library import all_propagators, Propagator

denominator = "(P('mu', id) * P('mu', id) - Mass(id) * Mass(id))"
denominatorSq = denominator + "**2"
numV = "(- Metric(1, 2))"

V2 = Propagator(name = "V2",
                numerator = "- Metric(1, 2)",
                denominator =  "P('mu', id) * P('mu', id)"
               )

Z1 =  Propagator(name = "Z1",
                numerator = "-" + numV + "* complex(0,1) * Mass(id) * dWZ",
                denominator = denominatorSq
               )
"#;

    #[test]
    fn parses_literal_and_concatenated_forms() {
        let props = parse_propagators(SAMPLE).unwrap();
        assert_eq!(props.len(), 2);

        let v2 = &props[0];
        assert_eq!(v2.python_name, "V2");
        assert_eq!(v2.name, "V2");
        assert_eq!(v2.numerator, "- Metric(1, 2)");
        assert_eq!(v2.denominator, "P('mu', id) * P('mu', id)");

        let z1 = &props[1];
        assert_eq!(z1.name, "Z1");
        assert_eq!(
            z1.numerator,
            "-(- Metric(1, 2))* complex(0,1) * Mass(id) * dWZ"
        );
        assert_eq!(
            z1.denominator,
            "(P('mu', id) * P('mu', id) - Mass(id) * Mass(id))**2"
        );
    }

    #[test]
    fn empty_file_yields_no_propagators() {
        assert!(parse_propagators("").unwrap().is_empty());
    }
}
