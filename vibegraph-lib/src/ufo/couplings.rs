use super::ast_util::{call_func_name, extract_int, extract_str, kwarg_str, parse_stmts};
use super::expr::{collect_deps, parse_expr, Expr};
use rustpython_parser::ast;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CouplingError {
    #[error("Failed to parse couplings.py: {0}")]
    Parse(String),
    #[error("Expression parse error for coupling '{name}': {cause}")]
    ExprParse { name: String, cause: String },
}

/// Opaque index into `UFOModel::couplings`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CouplingId(pub usize);

#[derive(Debug, Clone)]
pub struct Coupling {
    /// Python variable name, e.g. `"GC_10"`.
    pub python_name: String,
    /// UFO `name` field.
    pub name: String,
    /// Symbolic expression for the coupling constant value.
    pub value: Expr,
    /// Coupling order dict, e.g. `{"QCD": 1}`.
    pub orders: HashMap<String, usize>,
    /// Parameter names this coupling directly depends on.
    pub deps: Vec<String>,
}

/// Parse `couplings.py` content into a list of [`Coupling`]s.
pub fn parse_couplings(src: &str) -> Result<Vec<Coupling>, CouplingError> {
    let stmts = parse_stmts(src).map_err(|e| CouplingError::Parse(e.to_string()))?;
    let mut result = Vec::new();

    for stmt in &stmts {
        let ast::Stmt::Assign(ast::StmtAssign { targets, value, .. }) = stmt else {
            continue;
        };
        let ast::Expr::Name(ast::ExprName { id, .. }) = targets.first().unwrap() else {
            continue;
        };
        let python_name = id.as_str().to_owned();

        let ast::Expr::Call(ast::ExprCall { func, keywords, .. }) = value.as_ref() else {
            continue;
        };
        if call_func_name(func) != Some("Coupling") {
            continue;
        }

        let name = kwarg_str(keywords, "name").unwrap_or_else(|| python_name.clone());
        let expr_str = kwarg_str(keywords, "value").unwrap_or_else(|| "0.0".to_owned());
        let expr = parse_expr(&expr_str).map_err(|e| CouplingError::ExprParse {
            name: name.clone(),
            cause: e.to_string(),
        })?;
        let mut deps = Vec::new();
        collect_deps(&expr, &mut deps);
        let orders = extract_orders(keywords);

        result.push(Coupling {
            python_name,
            name,
            value: expr,
            orders,
            deps,
        });
    }

    Ok(result)
}

/// Extract the `order = {'QCD': 1, ...}` dict from keyword arguments.
fn extract_orders(keywords: &[ast::Keyword]) -> HashMap<String, usize> {
    use super::ast_util::get_kwarg;
    let Some(val) = get_kwarg(keywords, "order") else {
        return HashMap::new();
    };
    let ast::Expr::Dict(ast::ExprDict { keys, values, .. }) = val else {
        return HashMap::new();
    };

    keys.iter()
        .zip(values.iter())
        .filter_map(|(k, v)| {
            let key = k.as_ref().and_then(|e| extract_str(e))?.to_owned();
            let val = extract_int(v).map(|n| n as usize)?;
            Some((key, val))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ufo::expr::eval;
    use num_complex::Complex64;
    use std::collections::HashMap as Map;

    const SAMPLE: &str = r"
from object_library import all_couplings, Coupling

GC_10 = Coupling(name = 'GC_10',
                 value = '-G',
                 order = {'QCD':1})

GC_33 = Coupling(name = 'GC_33',
                 value = 'complex(0,1)*G**2',
                 order = {'QCD':2})
";

    fn g_map(g: f64) -> Map<String, Complex64> {
        [("G".to_owned(), Complex64::new(g, 0.0))].into()
    }

    #[test]
    fn test_parse_couplings() {
        let couplings = parse_couplings(SAMPLE).unwrap();
        assert!(couplings.iter().any(|c| c.python_name == "GC_10"));
        assert!(couplings.iter().any(|c| c.python_name == "GC_33"));
    }

    #[test]
    fn test_gc10_value() {
        let couplings = parse_couplings(SAMPLE).unwrap();
        let gc10 = couplings.iter().find(|c| c.python_name == "GC_10").unwrap();
        let val = eval(&gc10.value, &g_map(1.2177));
        assert!((val.re + 1.2177).abs() < 1e-8);
        assert!(val.im.abs() < 1e-12);
    }

    #[test]
    fn test_gc33_value() {
        let couplings = parse_couplings(SAMPLE).unwrap();
        let gc33 = couplings.iter().find(|c| c.python_name == "GC_33").unwrap();
        let g = 1.2177f64;
        let val = eval(&gc33.value, &g_map(g));
        assert!(val.re.abs() < 1e-8);
        assert!((val.im - g * g).abs() < 1e-6);
    }

    #[test]
    fn test_orders() {
        let couplings = parse_couplings(SAMPLE).unwrap();
        let gc10 = couplings.iter().find(|c| c.python_name == "GC_10").unwrap();
        let gc33 = couplings.iter().find(|c| c.python_name == "GC_33").unwrap();
        assert_eq!(gc10.orders.get("QCD"), Some(&1));
        assert_eq!(gc33.orders.get("QCD"), Some(&2));
    }

    #[test]
    fn test_deps() {
        let couplings = parse_couplings(SAMPLE).unwrap();
        let gc10 = couplings.iter().find(|c| c.python_name == "GC_10").unwrap();
        assert!(gc10.deps.contains(&"G".to_owned()));
    }
}
