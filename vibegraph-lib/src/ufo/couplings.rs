use super::ast_util::{
    constructor_calls, extract_str, parse_stmts, FieldError, Fields, UniqueNames,
};
use super::config_eval::{
    configuration_modules, eval_int, read_configuration, Configuration, Scope,
};
use super::expr::{collect_deps, parse_expr, Expr};
use super::functions::FunctionLibrary;
use indexmap::IndexMap;
use num_complex::Complex64;
use rustpython_parser::ast;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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

/// Read `configuration.py`; see [`read_configuration`].
pub(crate) fn parse_configuration(src: &str) -> Result<Configuration, CouplingError> {
    let stmts = parse_stmts(src).map_err(|e| CouplingError::Parse(e.to_string()))?;
    Ok(read_configuration(&stmts))
}

/// [`parse_couplings_with`] with no `configuration.py` and the SM's function
/// library.
#[cfg(test)]
pub(crate) fn parse_couplings(src: &str) -> Result<Vec<Coupling>, CouplingError> {
    parse_couplings_with(src, &Configuration::new(), &FunctionLibrary::default())
}

/// Parse `couplings.py` content into a list of [`Coupling`]s, reading
/// `configuration.<name>` in a coupling order from `configuration` and each
/// value's library functions as `library` defines them.
pub(crate) fn parse_couplings_with(
    src: &str,
    configuration: &Configuration,
    library: &FunctionLibrary,
) -> Result<Vec<Coupling>, CouplingError> {
    let stmts = parse_stmts(src).map_err(|e| CouplingError::Parse(e.to_string()))?;
    let modules = configuration_modules(&stmts);
    let scope = Scope {
        names: configuration,
        modules: &modules,
        bare: false,
    };
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
        let orders = extract_orders(&fields, &scope)?;
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

/// The `order = {'QCD': 1, ...}` dict of a coupling.
///
/// An order is an integer literal (a negative one included: SMEFTsim's vev
/// insertion is `QED: -1`), or an expression in the subset of Python
/// [`config_eval`](super::config_eval) evaluates over `configuration.py`'s
/// values.
fn extract_orders(
    fields: &Fields<'_>,
    scope: &Scope<'_>,
) -> Result<BTreeMap<String, i64>, FieldError> {
    const EXPECTED: &str = "a dict of order names to integers, or to an expression over \
                            `configuration.<name>` values built from integers, `not`, \
                            `and`, `or`, `+`, `-`, `*` and `A if T else B`";
    let ast::Expr::Dict(ast::ExprDict { keys, values, .. }) = fields.required("order")? else {
        return Err(fields.malformed("order", EXPECTED));
    };
    keys.iter()
        .zip(values.iter())
        .map(|(k, v)| {
            let key = k.as_ref().and_then(|e| extract_str(e));
            let val = eval_int(v, scope);
            match (key, val) {
                (Some(key), Some(val)) => Ok((key.to_owned(), val)),
                _ => Err(fields.malformed("order", EXPECTED)),
            }
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
