//! Parser for UFO `couplings.py` — extended to include the `value` expression.
//!
//! FeynGraph parses `couplings.py` but only extracts the `order` dict, discarding
//! the symbolic `value` string. We need `value` to evaluate the numerical coupling
//! constant at a given parameter point.
//!
//! Example:
//! ```python
//! GC_10 = Coupling(value = '-G', order = {'QCD':1})
//! GC_33 = Coupling(value = 'complex(0,1)*G**2', order = {'QCD':2})
//! ```

use crate::ufo::expr::{Expr, collect_deps, parse_expr};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CouplingError {
    #[error("Failed to parse couplings.py: {0}")]
    Parse(String),
    #[error("Expression parse error for coupling '{name}': {cause}")]
    ExprParse { name: String, cause: String },
}

#[derive(Debug, Clone)]
pub struct CouplingValue {
    pub name: String,
    /// Symbolic expression for the coupling constant value.
    pub value: Expr,
    /// Coupling order dict, e.g. `{"QCD": 1}`.
    pub orders: HashMap<String, usize>,
    /// Parameter names this coupling directly depends on.
    pub deps: Vec<String>,
}

/// Parse `couplings.py` content into a map of coupling name → [`CouplingValue`].
pub fn parse_couplings(content: &str) -> Result<HashMap<String, CouplingValue>, CouplingError> {
    let raw =
        ufo_couplings::couplings(content).map_err(|e| CouplingError::Parse(e.to_string()))?;

    let mut result = HashMap::new();

    for rc in raw {
        let name = rc.name.clone();
        let expr_str = rc.value_str.as_deref().unwrap_or("0.0");
        let expr = parse_expr(expr_str).map_err(|e| CouplingError::ExprParse {
            name: name.clone(),
            cause: e.to_string(),
        })?;
        let mut deps = Vec::new();
        collect_deps(&expr, &mut deps);

        result.insert(
            name.clone(),
            CouplingValue { name, value: expr, orders: rc.orders, deps },
        );
    }

    Ok(result)
}

/// Raw parsed coupling before expression parsing.
#[derive(Debug, Default)]
struct RawCoupling {
    name: String,
    value_str: Option<String>,
    orders: HashMap<String, usize>,
}

peg::parser! {
    grammar ufo_couplings() for str {

        pub rule couplings() -> Vec<RawCoupling>
            = _ c:(coupling() ** _) _ { c }

        rule coupling() -> RawCoupling
            = _ name:ident() _ "=" _ "Coupling(" _ props:(prop() ** (_ "," _)) _ ","? _ ")" _ {
                let mut rc = RawCoupling { name: name.to_owned(), ..Default::default() };
                for (k, v) in props {
                    match k {
                        "name"  => { /* ignored — use LHS name */ }
                        "value" => rc.value_str = Some(v.unquoted()),
                        "order" => rc.orders = v.order_dict(),
                        _ => {}
                    }
                }
                rc
            }

        rule prop() -> (&'input str, RawVal)
            = _ k:ident() _ "=" _ v:prop_value() _ { (k, v) }

        rule prop_value() -> RawVal
            = s:quoted_string() { RawVal::Str(s) }
            / "{" _ entries:(order_entry() ** (_ "," _)) _ ","? _ "}" {
                RawVal::OrderDict(entries.into_iter().collect())
            }

        rule order_entry() -> (String, usize)
            = _ "'" k:$(['a'..='z' | 'A'..='Z' | '_']['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*)
              "'" _ ":" _ v:uint() _ { (k.to_owned(), v) }

        rule quoted_string() -> String
            = "'" s:$([^'\'']*) "'" { s.to_owned() }
            / "\"" s:$([^'"']*) "\"" { s.to_owned() }

        rule uint() -> usize
            = s:$(['0'..='9']+) {? s.parse().or(Err("uint")) }

        rule ident() -> &'input str
            = $(['a'..='z' | 'A'..='Z' | '_'] ['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*)

        rule _ = (whitespace() / comment() / python_skip_line())*
        rule whitespace() = [' ' | '\t' | '\n' | '\r']+
        rule comment() = "#" [^'\n']* ("\n" / ![_])
        rule python_skip_line()
            = "from" [' ' | '\t'] [^'\n']* ("\n" / ![_])
            / "import" [' ' | '\t'] [^'\n']* ("\n" / ![_])
    }
}

#[derive(Debug)]
enum RawVal {
    Str(String),
    OrderDict(HashMap<String, usize>),
}

impl RawVal {
    fn unquoted(self) -> String {
        match self {
            RawVal::Str(s) => s,
            _ => String::new(),
        }
    }

    fn order_dict(self) -> HashMap<String, usize> {
        match self {
            RawVal::OrderDict(m) => m,
            _ => HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_complex::Complex64;
    use crate::ufo::expr::eval;
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
        assert!(couplings.contains_key("GC_10"));
        assert!(couplings.contains_key("GC_33"));
    }

    #[test]
    fn test_gc10_value() {
        let couplings = parse_couplings(SAMPLE).unwrap();
        let gc10 = &couplings["GC_10"];
        let val = eval(&gc10.value, &g_map(1.2177));
        assert!((val.re + 1.2177).abs() < 1e-8);
        assert!(val.im.abs() < 1e-12);
    }

    #[test]
    fn test_gc33_value() {
        let couplings = parse_couplings(SAMPLE).unwrap();
        let gc33 = &couplings["GC_33"];
        let g = 1.2177f64;
        let val = eval(&gc33.value, &g_map(g));
        // complex(0,1) * G^2  → im = G^2, re = 0
        assert!(val.re.abs() < 1e-8);
        assert!((val.im - g * g).abs() < 1e-6);
    }

    #[test]
    fn test_orders() {
        let couplings = parse_couplings(SAMPLE).unwrap();
        assert_eq!(couplings["GC_10"].orders.get("QCD"), Some(&1));
        assert_eq!(couplings["GC_33"].orders.get("QCD"), Some(&2));
    }

    #[test]
    fn test_deps() {
        let couplings = parse_couplings(SAMPLE).unwrap();
        assert!(couplings["GC_10"].deps.contains(&"G".to_owned()));
        assert!(couplings["GC_33"].deps.contains(&"G".to_owned()));
    }
}
