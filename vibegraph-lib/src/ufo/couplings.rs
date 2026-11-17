use super::ast_util::{
    constructor_calls, extract_int, extract_name, extract_str, named_assignments, parse_stmts,
    FieldError, Fields, UniqueNames,
};
use super::expr::{collect_deps, parse_expr, Expr};
use super::functions::FunctionLibrary;
use indexmap::IndexMap;
use num_complex::Complex64;
use rustpython_parser::ast;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::ops::Index;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CouplingError {
    #[error("Failed to parse couplings.py: {0}")]
    Parse(String),
    #[error("Expression parse error for coupling '{name}': {cause}")]
    ExprParse { name: String, cause: String },
    #[error(
        "coupling '{coupling}' references parameter '{name}', which parameters.py does not define"
    )]
    UnknownParameter { name: String, coupling: String },
    #[error(transparent)]
    Field(#[from] FieldError),
}

/// Strongly typed index for [`Coupling`] lookup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CouplingId(usize);

impl From<usize> for CouplingId {
    fn from(value: usize) -> Self {
        CouplingId(value)
    }
}

impl CouplingId {
    /// The raw index this id wraps.
    pub(crate) const fn index(self) -> usize {
        self.0
    }
}

impl std::fmt::Display for CouplingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Index<CouplingId> for IndexMap<String, Coupling> {
    type Output = Coupling;

    fn index(&self, index: CouplingId) -> &Self::Output {
        self.index(index.0)
    }
}

/// As used in [`super::EvaluatedModel`] for storing evaluated coupling values.
impl Index<CouplingId> for Vec<Complex64> {
    type Output = Complex64;

    fn index(&self, index: CouplingId) -> &Self::Output {
        &self[index.0]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coupling {
    /// Python variable name, e.g. `"GC_10"`.
    pub(crate) python_name: String,
    /// UFO `name` field.
    pub(crate) name: String,
    /// Symbolic expression for the coupling constant value.
    pub(crate) value: Expr,
    /// Coupling order dict, e.g. `{"QCD": 1}`. Signed: SMEFTsim writes its
    /// vev insertions as `{"NP": 1, "QED": -1}`, and MadGraph sums the powers
    /// with their sign.
    pub orders: BTreeMap<String, i64>,
    /// Parameter names this coupling directly depends on.
    pub(crate) deps: Vec<String>,
}

/// The module-level names a model's `configuration.py` binds to an integer or
/// a boolean, as the integer (`True` is 1): the switches a coupling order may
/// test.
pub(crate) type Configuration = HashMap<String, i64>;

/// Read `configuration.py`. An assignment that is not an integer or boolean
/// literal binds nothing here, and a coupling order reading it is refused.
pub(crate) fn parse_configuration(src: &str) -> Result<Configuration, CouplingError> {
    let stmts = parse_stmts(src).map_err(|e| CouplingError::Parse(e.to_string()))?;
    Ok(named_assignments(&stmts)
        .filter_map(|(name, value)| Some((name.to_owned(), literal_int(value)?)))
        .collect())
}

fn literal_int(expr: &ast::Expr) -> Option<i64> {
    match expr {
        ast::Expr::Constant(ast::ExprConstant {
            value: ast::Constant::Bool(b),
            ..
        }) => Some(i64::from(*b)),
        _ => extract_int(expr),
    }
}

/// Parse `couplings.py` content into a list of [`Coupling`]s.
#[cfg(test)]
pub(crate) fn parse_couplings(src: &str) -> Result<Vec<Coupling>, CouplingError> {
    parse_couplings_with(src, &Configuration::new(), &FunctionLibrary::default())
}

/// [`parse_couplings`], reading `configuration.<name>` in a coupling order
/// from `configuration`.
pub(crate) fn parse_couplings_with(
    src: &str,
    configuration: &Configuration,
    library: &FunctionLibrary,
) -> Result<Vec<Coupling>, CouplingError> {
    let stmts = parse_stmts(src).map_err(|e| CouplingError::Parse(e.to_string()))?;
    let module_aliases = configuration_aliases(&stmts);
    let mut result = Vec::new();

    // `Coupling.require_args` in UFO's object library
    // (`mg5amcnlo/models/sm/object_library.py` L187): `name`, `value`, `order`.
    // A repeated name is refused ([`UniqueNames`]).
    let mut seen = UniqueNames::new("coupling");
    for (python_name, keywords) in constructor_calls(&stmts, "Coupling") {
        let fields = Fields::new("coupling", python_name, keywords);
        let name = fields.str("name")?;
        seen.insert(python_name, &name)?;
        let expr_str = fields.str("value")?;
        let orders = extract_orders(&fields, &module_aliases, configuration)?;
        let python_name = python_name.to_owned();
        let expr = parse_expr(&expr_str)
            .map(|e| library.resolve(e))
            .map_err(|e| CouplingError::ExprParse {
                name: name.clone(),
                cause: e.to_string(),
            })?;
        let mut deps = Vec::new();
        collect_deps(&expr, &mut deps);

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

/// The local names `couplings.py` binds to the `configuration` module
/// (`import configuration [as c]`).
fn configuration_aliases(stmts: &[ast::Stmt]) -> Vec<String> {
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

/// The `order = {'QCD': 1, ...}` dict of a coupling.
///
/// An order is an integer literal (a negative one included: SMEFTsim's vev
/// insertion is `QED: -1`), or one of the forms a model writes to switch it on
/// a `configuration.py` flag: `A if configuration.flag else B`,
/// `not configuration.flag`, and a bare `configuration.flag`, nested freely.
/// Python's own evaluation is reproduced for these and nothing else.
fn extract_orders(
    fields: &Fields<'_>,
    module_aliases: &[String],
    configuration: &Configuration,
) -> Result<BTreeMap<String, i64>, FieldError> {
    const EXPECTED: &str = "a dict of order names to integers, or to a conditional expression \
                            over `configuration.<flag>` flags";
    let ast::Expr::Dict(ast::ExprDict { keys, values, .. }) = fields.required("order")? else {
        return Err(fields.malformed("order", EXPECTED));
    };
    keys.iter()
        .zip(values.iter())
        .map(|(k, v)| {
            let key = k.as_ref().and_then(|e| extract_str(e));
            let val = order_value(v, module_aliases, configuration);
            match (key, val) {
                (Some(key), Some(val)) => Ok((key.to_owned(), val)),
                _ => Err(fields.malformed("order", EXPECTED)),
            }
        })
        .collect()
}

/// The integer an order expression evaluates to, or `None` for a form outside
/// the subset [`extract_orders`] reads.
fn order_value(
    expr: &ast::Expr,
    module_aliases: &[String],
    configuration: &Configuration,
) -> Option<i64> {
    let recurse = |e: &ast::Expr| order_value(e, module_aliases, configuration);
    match expr {
        ast::Expr::IfExp(ast::ExprIfExp {
            test, body, orelse, ..
        }) => {
            if recurse(test)? != 0 {
                recurse(body)
            } else {
                recurse(orelse)
            }
        }
        ast::Expr::UnaryOp(ast::ExprUnaryOp {
            op: ast::UnaryOp::Not,
            operand,
            ..
        }) => Some(i64::from(recurse(operand)? == 0)),
        ast::Expr::Attribute(ast::ExprAttribute { value, attr, .. }) => {
            let module = extract_name(value)?;
            module_aliases
                .iter()
                .any(|a| a == module)
                .then(|| configuration.get(attr.as_str()).copied())?
        }
        _ => literal_int(expr),
    }
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
