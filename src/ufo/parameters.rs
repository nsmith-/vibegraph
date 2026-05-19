//! Parser for UFO `parameters.py`.
//!
//! Builds a [`ParameterSet`] that:
//! - Holds external parameters (with SLHA block/code for lookup)
//! - Holds internal parameters topo-sorted for evaluation order
//! - Provides a reverse-dependency map for efficient incremental re-evaluation
//!   (e.g. when α_s is updated, only params depending on `aS` need re-eval)

use crate::ufo::expr::{Expr, collect_deps, eval, parse_expr};
use crate::ufo::slha::ParamCard;
use num_complex::Complex64;
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParameterError {
    #[error("Failed to parse parameters.py: {0}")]
    Parse(String),
    #[error("Cyclic dependency among internal parameters")]
    CyclicDep,
    #[error("Expression parse error for parameter '{name}': {cause}")]
    ExprParse { name: String, cause: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParamNature {
    External {
        default_value: f64,
        lha_block: String,
        lha_code: Vec<i32>,
    },
    Internal {
        expr: Expr,
        /// Names of parameters this one directly depends on.
        deps: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    /// `true` if `type = 'complex'`
    pub complex: bool,
    pub nature: ParamNature,
}

/// A complete set of parsed parameters, ready for evaluation.
#[derive(Debug, Clone)]
pub struct ParameterSet {
    /// External parameters (no deps on other params).
    pub externals: Vec<Parameter>,
    /// Internal parameters in topo-sorted evaluation order.
    pub internals: Vec<Parameter>,
    /// Reverse dependency map: name → list of parameter names that depend on it.
    ///
    /// Used for incremental re-evaluation: when `aS` changes, walk `rdeps["aS"]`
    /// and re-evaluate in the order given by `internals`.
    pub rdeps: HashMap<String, Vec<String>>,
}

impl ParameterSet {
    /// Evaluate all parameters using the given param_card for external inputs.
    ///
    /// Missing SLHA entries fall back to the UFO default value.
    pub fn evaluate(&self, slha: &ParamCard) -> HashMap<String, Complex64> {
        let mut values: HashMap<String, Complex64> = HashMap::new();

        for p in &self.externals {
            let v = match &p.nature {
                ParamNature::External {
                    default_value,
                    lha_block,
                    lha_code,
                } => slha.get(lha_block, lha_code).unwrap_or(*default_value),
                _ => unreachable!(),
            };
            values.insert(p.name.clone(), Complex64::new(v, 0.0));
        }

        for p in &self.internals {
            let v = match &p.nature {
                ParamNature::Internal { expr, .. } => eval(expr, &values),
                _ => unreachable!(),
            };
            values.insert(p.name.clone(), v);
        }

        values
    }

    /// Re-evaluate only the transitive dependents of `changed` in place.
    ///
    /// This is the efficient path for α_s running: after updating `aS` in
    /// `current`, call `recompute("aS", current)` to propagate the change.
    pub fn recompute(&self, changed: &str, current: &mut HashMap<String, Complex64>) {
        // Collect all transitively affected params using BFS over rdeps.
        let mut affected: Vec<String> = Vec::new();
        let mut queue: VecDeque<&str> = VecDeque::new();
        queue.push_back(changed);
        while let Some(name) = queue.pop_front() {
            if let Some(children) = self.rdeps.get(name) {
                for child in children {
                    if !affected.contains(child) {
                        affected.push(child.clone());
                        queue.push_back(child.as_str());
                    }
                }
            }
        }

        // Re-evaluate affected internals in topo order.
        for p in &self.internals {
            if affected.contains(&p.name) {
                if let ParamNature::Internal { expr, .. } = &p.nature {
                    let v = eval(expr, current);
                    current.insert(p.name.clone(), v);
                }
            }
        }
    }
}

/// Parse `parameters.py` content into a [`ParameterSet`].
pub fn parse_parameters(content: &str) -> Result<ParameterSet, ParameterError> {
    let raw = ufo_params::parameters(content).map_err(|e| ParameterError::Parse(e.to_string()))?;

    let mut externals: Vec<Parameter> = Vec::new();
    let mut raw_internals: Vec<(String, bool, Expr, Vec<String>)> = Vec::new(); // (name, complex, expr, deps)

    for rp in raw {
        let name = rp.name.clone();
        let is_complex = rp.type_str.as_deref() == Some("complex");

        match rp.nature.as_deref() {
            Some("external") => {
                let default_value = rp.ext_value.unwrap_or(0.0);
                let lha_block = rp.lha_block.unwrap_or_default();
                let lha_code = rp.lha_code.unwrap_or_default();
                externals.push(Parameter {
                    name,
                    complex: is_complex,
                    nature: ParamNature::External {
                        default_value,
                        lha_block,
                        lha_code,
                    },
                });
            }
            _ => {
                // Treat unknown nature as internal.
                let expr_str = rp.int_value.as_deref().unwrap_or("0.0");
                let expr = parse_expr(expr_str).map_err(|e| ParameterError::ExprParse {
                    name: name.clone(),
                    cause: e.to_string(),
                })?;
                let mut deps = Vec::new();
                collect_deps(&expr, &mut deps);
                raw_internals.push((name, is_complex, expr, deps));
            }
        }
    }

    // Build the set of all known names (externals first).
    let mut all_known: Vec<String> = externals.iter().map(|p| p.name.clone()).collect();
    let internals = toposort_internals(raw_internals, &all_known)?;

    // Build reverse dep map.
    let mut rdeps: HashMap<String, Vec<String>> = HashMap::new();
    for p in &internals {
        if let ParamNature::Internal { deps, .. } = &p.nature {
            for dep in deps {
                rdeps.entry(dep.clone()).or_default().push(p.name.clone());
            }
        }
    }

    // Extend all_known with internals (for completeness, not used further here).
    all_known.extend(internals.iter().map(|p| p.name.clone()));

    Ok(ParameterSet {
        externals,
        internals,
        rdeps,
    })
}

/// Kahn's topological sort for internal parameters.
fn toposort_internals(
    raw: Vec<(String, bool, Expr, Vec<String>)>,
    externals: &[String],
) -> Result<Vec<Parameter>, ParameterError> {
    // Build maps.
    let n = raw.len();
    let names: Vec<String> = raw.iter().map(|(name, _, _, _)| name.clone()).collect();
    let mut in_degree: Vec<usize> = vec![0; n];
    let mut forward: Vec<Vec<usize>> = vec![vec![]; n]; // forward[i] = list of indices that depend on i

    let name_index: HashMap<&str, usize> = names
        .iter()
        .enumerate()
        .map(|(i, name)| (name.as_str(), i))
        .collect();

    for (i, (_, _, _, deps)) in raw.iter().enumerate() {
        for dep in deps {
            // dep might be an external (not in names) — skip those.
            if let Some(&j) = name_index.get(dep.as_str()) {
                in_degree[i] += 1;
                forward[j].push(i);
            }
            // Also skip externals — they have no in-degree contribution here.
        }
    }

    // Kahn's algorithm.
    let mut queue: VecDeque<usize> = VecDeque::new();
    for (i, &deg) in in_degree.iter().enumerate() {
        if deg == 0 {
            queue.push_back(i);
        }
    }

    let mut sorted: Vec<Parameter> = Vec::with_capacity(n);
    while let Some(i) = queue.pop_front() {
        let (name, complex, expr, deps) = &raw[i];
        sorted.push(Parameter {
            name: name.clone(),
            complex: *complex,
            nature: ParamNature::Internal {
                expr: expr.clone(),
                deps: deps.clone(),
            },
        });
        for &j in &forward[i] {
            in_degree[j] -= 1;
            if in_degree[j] == 0 {
                queue.push_back(j);
            }
        }
    }

    if sorted.len() != n {
        return Err(ParameterError::CyclicDep);
    }

    Ok(sorted)
}

/// Raw parsed parameter before classification.
#[derive(Debug, Default)]
struct RawParam {
    name: String,
    nature: Option<String>,
    type_str: Option<String>,
    ext_value: Option<f64>,    // external: bare float
    int_value: Option<String>, // internal: quoted string
    lha_block: Option<String>,
    lha_code: Option<Vec<i32>>,
}

peg::parser! {
    grammar ufo_params() for str {

        pub rule parameters() -> Vec<RawParam>
            = _ p:(parameter() ** _) _ { p }

        rule parameter() -> RawParam
            = _ name:ident() _ "=" _ "Parameter(" _ props:(prop() ** (_ "," _)) _ ","? _ ")" _ {
                let mut p = RawParam { name: name.to_owned(), ..Default::default() };
                for (k, v) in props {
                    match k {
                        "name"     => { /* already have it from LHS */ }
                        "nature"   => p.nature   = Some(v.unquoted().to_owned()),
                        "type"     => p.type_str = Some(v.unquoted().to_owned()),
                        "lhablock" => p.lha_block = Some(v.unquoted().to_owned()),
                        "lhacode"  => p.lha_code  = Some(v.int_list()),
                        "value"    => {
                            match v {
                                PropVal::Str(s) => p.int_value = Some(s.to_owned()),
                                PropVal::Float(f) => p.ext_value = Some(f),
                                PropVal::Int(i) => p.ext_value = Some(i as f64),
                                _ => {}
                            }
                        }
                        "texname" | "lhatex" => { /* ignored */ }
                        _ => {}
                    }
                }
                p
            }

        rule prop() -> (&'input str, PropVal<'input>)
            = _ k:ident() _ "=" _ v:prop_value() _ { (k, v) }

        rule prop_value() -> PropVal<'input>
            = s:quoted_string() { PropVal::Str(s) }
            / f:float() { PropVal::Float(f) }
            / i:int() { PropVal::Int(i) }
            / "[" _ items:(int() ** (_ "," _)) _ ","? _ "]" { PropVal::IntList(items) }

        rule quoted_string() -> &'input str
            = "r'" s:$([^'\'']*) "'" { s }                        // raw string r'...'
            / "r\"" s:$([^'"']*) "\"" { s }                       // raw string r"..."
            / "'" s:$(([^'\'' | '\\'] / "\\" [_])*) "'" { s }     // string with \' escapes
            / "\"" s:$(([^'"' | '\\'] / "\\" [_])*) "\"" { s }

        rule float() -> f64
            = s:$(
                "-"? ['0'..='9']+ "." ['0'..='9']* (exponent())?
                / "-"? ['0'..='9']* "." ['0'..='9']+ (exponent())?
                / "-"? ['0'..='9']+ exponent()
            ) {?
                s.parse().or(Err("float"))
            }

        rule exponent() = ("e" / "E") ("+" / "-")? ['0'..='9']+

        rule int() -> i32
            = s:$("-"? ['0'..='9']+) {? s.parse().or(Err("int")) }

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

#[derive(Debug, Clone)]
enum PropVal<'a> {
    Str(&'a str),
    Float(f64),
    Int(i32),
    IntList(Vec<i32>),
}

impl<'a> PropVal<'a> {
    fn unquoted(&self) -> &str {
        match self {
            PropVal::Str(s) => s,
            _ => "",
        }
    }

    fn int_list(&self) -> Vec<i32> {
        match self {
            PropVal::IntList(v) => v.clone(),
            PropVal::Int(i) => vec![*i],
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    const SM_PARAMS_FRAGMENT: &str = r"
from object_library import all_parameters, Parameter
from function_library import complexconjugate, re, im, csc, sec, acsc, asec

ZERO = Parameter(name = 'ZERO',
                 nature = 'internal',
                 type = 'real',
                 value = '0.0',
                 texname = '0')

aS = Parameter(name = 'aS',
               nature = 'external',
               type = 'real',
               value = 0.118,
               texname = '\alpha _s',
               lhablock = 'SMINPUTS',
               lhacode = [ 3 ])

MZ = Parameter(name = 'MZ',
               nature = 'external',
               type = 'real',
               value = 91.1876,
               texname = 'm_Z',
               lhablock = 'MASS',
               lhacode = [ 23 ])

G = Parameter(name = 'G',
              nature = 'internal',
              type = 'real',
              value = '2*cmath.sqrt(aS)*cmath.sqrt(cmath.pi)',
              texname = 'G')
";

    fn empty_card() -> ParamCard {
        ParamCard::from_str("").unwrap()
    }

    #[test]
    fn test_parse_fragment() {
        let ps = parse_parameters(SM_PARAMS_FRAGMENT).unwrap();
        let ext_names: Vec<&str> = ps.externals.iter().map(|p| p.name.as_str()).collect();
        assert!(ext_names.contains(&"aS"));
        assert!(ext_names.contains(&"MZ"));

        let int_names: Vec<&str> = ps.internals.iter().map(|p| p.name.as_str()).collect();
        assert!(int_names.contains(&"ZERO"));
        assert!(int_names.contains(&"G"));
    }

    #[test]
    fn test_evaluate_defaults() {
        let ps = parse_parameters(SM_PARAMS_FRAGMENT).unwrap();
        let vals = ps.evaluate(&empty_card());

        let as_val = vals["aS"].re;
        assert!((as_val - 0.118).abs() < 1e-10);

        let expected_g = 2.0 * (0.118f64).sqrt() * PI.sqrt();
        assert!((vals["G"].re - expected_g).abs() < 1e-8);
    }

    #[test]
    fn test_recompute() {
        let ps = parse_parameters(SM_PARAMS_FRAGMENT).unwrap();
        let mut vals = ps.evaluate(&empty_card());

        let new_as = 0.130f64;
        vals.insert("aS".to_owned(), Complex64::new(new_as, 0.0));
        ps.recompute("aS", &mut vals);

        let expected_g = 2.0 * new_as.sqrt() * PI.sqrt();
        assert!((vals["G"].re - expected_g).abs() < 1e-8);
    }

    #[test]
    fn test_rdeps() {
        let ps = parse_parameters(SM_PARAMS_FRAGMENT).unwrap();
        let g_deps = &ps.rdeps["aS"];
        assert!(g_deps.contains(&"G".to_owned()));
    }
}
