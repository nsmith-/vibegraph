use super::ast_util::{
    call_func_name, extract_float, extract_int, extract_str, kwarg_str, parse_stmts,
};
use super::expr::{collect_deps, eval, parse_expr, Expr};
use super::slha::ParamCard;
use num_complex::Complex64;
use num_traits::Zero;
use rustpython_parser::ast;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    /// `true` if `type = 'complex'`
    pub complex: bool,
    pub nature: ParamNature,
}

/// A complete set of parsed parameters, ready for evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterSet {
    /// External parameters (no deps on other params).
    pub externals: Vec<Parameter>,
    /// Internal parameters in topo-sorted evaluation order.
    pub internals: Vec<Parameter>,
    /// Reverse dependency map: name → list of parameter names that depend on it.
    pub rdeps: BTreeMap<String, Vec<String>>,
    /// Const-zero parameters (set by restrictions)
    pub zeros: BTreeSet<String>,
}

impl ParameterSet {
    /// Evaluate all parameters using the given param_card for external inputs.
    pub fn evaluate(&self, slha: &ParamCard) -> HashMap<String, Complex64> {
        let mut values: HashMap<String, Complex64> = HashMap::new();

        for p in &self.externals {
            let v = match &p.nature {
                ParamNature::External {
                    default_value,
                    lha_block,
                    lha_code,
                } => {
                    // A parameter zeroed by a restriction is locked to zero: the
                    // user's param card cannot revive it (see `apply_restrict`).
                    // Keep it in the map as 0.0 — internal params still reference
                    // it (e.g. CKM via `lamWS`, `ye` via `yme`), and `eval` panics
                    // on a missing name.
                    if self.zeros.contains(&p.name) {
                        0.0
                    } else {
                        slha.get(lha_block, lha_code).unwrap_or(*default_value)
                    }
                }
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

    /// Bake a model restriction card's external values into the parameter defaults.
    ///
    /// The restriction is the model's new baseline, not merely a list of things to
    /// switch off: MadGraph reads the card into every external parameter it names
    /// (`model_reader.set_parameters_and_couplings` assigns `parameter.value`) and
    /// writes those values out as the `param_card.dat` of the generated process, so
    /// a run of a restricted model with no card of its own is a run *at the
    /// restriction's values*. Baking only the zeros would leave every other
    /// coefficient at the `parameters.py` default — for SMEFTsim, zero — and a
    /// card-less evaluation of a restricted model would silently be its
    /// Standard-Model limit.
    ///
    /// The zeroed parameters (light masses/Yukawas, CKM mixing) are additionally
    /// *locked*: a later param card must not revive one, because the restriction is
    /// also what pruned the vertices and diagrams, so reviving a parameter would
    /// evaluate a vertex set that no longer matches the couplings. Non-zero values
    /// are defaults and stay overridable, which is what makes the generated
    /// `param_card.dat` an editable card rather than a transcript.
    pub fn apply_restrict(&mut self, card: &ParamCard) {
        for p in &mut self.externals {
            if let ParamNature::External {
                default_value,
                lha_block,
                lha_code,
            } = &mut p.nature
            {
                if let Some(v) = card.get(lha_block, lha_code) {
                    *default_value = v;
                    if v.is_zero() {
                        self.zeros.insert(p.name.clone());
                    }
                }
            }
        }
    }

    /// Every parameter whose value transitively depends on `changed`.
    ///
    /// The set [`recompute`](Self::recompute) re-evaluates, exposed on its own so a
    /// caller can ask *which* parameters a change moves without performing it. A
    /// parameter locked to zero by a restriction moves nothing, so the set is empty.
    pub fn dependents(&self, changed: &str) -> HashSet<String> {
        let mut affected: HashSet<String> = HashSet::new();
        if self.zeros.contains(changed) {
            return affected;
        }
        let mut queue: VecDeque<&str> = VecDeque::new();
        queue.push_back(changed);
        while let Some(name) = queue.pop_front() {
            let Some(children) = self.rdeps.get(name) else {
                continue;
            };
            for child in children {
                if affected.insert(child.clone()) {
                    queue.push_back(child.as_str());
                }
            }
        }
        affected
    }

    /// Re-evaluate only the transitive dependents of `changed` in place.
    pub fn recompute(&self, changed: &str, current: &mut HashMap<String, Complex64>) {
        if self.zeros.contains(changed) {
            // This parameter is fixed to zero by a restriction, so ignore any changes.
            return;
        }
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
    let stmts = parse_stmts(content).map_err(|e| ParameterError::Parse(e.to_string()))?;

    let mut externals: Vec<Parameter> = Vec::new();
    let mut raw_internals: Vec<(String, bool, Expr, Vec<String>)> = Vec::new();

    for stmt in &stmts {
        let ast::Stmt::Assign(ast::StmtAssign { targets, value, .. }) = stmt else {
            continue;
        };
        let ast::Expr::Name(ast::ExprName { id, .. }) = targets.first().unwrap() else {
            continue;
        };
        let lhs_name = id.as_str().to_owned();

        let ast::Expr::Call(ast::ExprCall { func, keywords, .. }) = value.as_ref() else {
            continue;
        };
        if call_func_name(func) != Some("Parameter") {
            continue;
        }

        // Use the LHS variable name as the canonical name.
        let name = lhs_name;
        let is_complex = kwarg_str(keywords, "type").as_deref() == Some("complex");
        let nature_str = kwarg_str(keywords, "nature");

        match nature_str.as_deref() {
            Some("external") => {
                let default_value = extract_value_float(keywords).unwrap_or(0.0);
                let lha_block = kwarg_str(keywords, "lhablock").unwrap_or_default();
                let lha_code = extract_lhacode(keywords);
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
                let expr_str = extract_value_str(keywords).unwrap_or_else(|| "0.0".to_owned());
                let expr = parse_expr(&expr_str).map_err(|e| ParameterError::ExprParse {
                    name: name.clone(),
                    cause: e.to_string(),
                })?;
                let mut deps = Vec::new();
                collect_deps(&expr, &mut deps);
                raw_internals.push((name, is_complex, expr, deps));
            }
        }
    }

    let internals = toposort_internals(raw_internals)?;

    let mut rdeps: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for p in &internals {
        if let ParamNature::Internal { deps, .. } = &p.nature {
            for dep in deps {
                rdeps.entry(dep.clone()).or_default().push(p.name.clone());
            }
        }
    }

    Ok(ParameterSet {
        externals,
        internals,
        rdeps,
        zeros: BTreeSet::new(),
    })
}

/// Extract the `value` keyword as a string (for internal parameters).
fn extract_value_str(keywords: &[ast::Keyword]) -> Option<String> {
    use super::ast_util::get_kwarg;
    let val = get_kwarg(keywords, "value")?;
    extract_str(val).map(|s| s.to_owned())
}

/// Extract the `value` keyword as a float (for external parameters).
fn extract_value_float(keywords: &[ast::Keyword]) -> Option<f64> {
    use super::ast_util::get_kwarg;
    let val = get_kwarg(keywords, "value")?;
    extract_float(val).or_else(|| extract_int(val).map(|n| n as f64))
}

/// Extract `lhacode = [ 3 ]` as a Vec<i32>.
fn extract_lhacode(keywords: &[ast::Keyword]) -> Vec<i32> {
    use super::ast_util::get_kwarg;
    let Some(val) = get_kwarg(keywords, "lhacode") else {
        return vec![];
    };
    match val {
        ast::Expr::List(ast::ExprList { elts, .. }) => elts
            .iter()
            .filter_map(extract_int)
            .map(|n| n as i32)
            .collect(),
        _ => extract_int(val).map(|n| vec![n as i32]).unwrap_or_default(),
    }
}

fn toposort_internals(
    raw: Vec<(String, bool, Expr, Vec<String>)>,
) -> Result<Vec<Parameter>, ParameterError> {
    let n = raw.len();
    let names: Vec<String> = raw.iter().map(|(name, _, _, _)| name.clone()).collect();
    let mut in_degree: Vec<usize> = vec![0; n];
    let mut forward: Vec<Vec<usize>> = vec![vec![]; n];

    let name_index: HashMap<&str, usize> = names
        .iter()
        .enumerate()
        .map(|(i, name)| (name.as_str(), i))
        .collect();

    for (i, (_, _, _, deps)) in raw.iter().enumerate() {
        for dep in deps {
            if let Some(&j) = name_index.get(dep.as_str()) {
                in_degree[i] += 1;
                forward[j].push(i);
            }
        }
    }

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
        "".parse::<ParamCard>().unwrap()
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

    const PARAMS_WITH_MASS_DEP: &str = r"
from object_library import all_parameters, Parameter

MZ = Parameter(name = 'MZ',
               nature = 'external',
               type = 'real',
               value = 91.1876,
               texname = 'm_Z',
               lhablock = 'MASS',
               lhacode = [ 23 ])

MZ2 = Parameter(name = 'MZ2',
                nature = 'internal',
                type = 'real',
                value = 'MZ**2',
                texname = 'm_Z^2')
";

    #[test]
    fn test_restrict_zero_is_locked() {
        let mut ps = parse_parameters(PARAMS_WITH_MASS_DEP).unwrap();

        // Restriction zeros MZ, which locks it; a non-zero entry becomes a
        // default instead (see `test_restrict_nonzero_becomes_the_default`).
        let restrict = "Block MASS\n 23 0.0\n".parse::<ParamCard>().unwrap();
        ps.apply_restrict(&restrict);
        assert!(ps.zeros.contains("MZ"));

        // A user card cannot revive a restriction-zeroed parameter, and the
        // internal that references it stays zero (no panic on a missing name).
        let user = "Block MASS\n 23 91.1876\n".parse::<ParamCard>().unwrap();
        let vals = ps.evaluate(&user);
        assert_eq!(vals["MZ"].re, 0.0);
        assert_eq!(vals["MZ2"].re, 0.0);

        // recompute also refuses to change a locked parameter.
        let mut vals = ps.evaluate(&user);
        vals.insert("MZ".to_owned(), Complex64::new(91.1876, 0.0));
        ps.recompute("MZ", &mut vals);
        assert_eq!(vals["MZ2"].re, 0.0);
    }

    /// A non-zero restriction value becomes the parameter's default and stays
    /// overridable — the two halves of what MadGraph's `restrict_model` does with
    /// one, and the pair that decides what a card-less run of a restricted model
    /// computes.
    ///
    /// The first half is the one with teeth: reading the restriction only for the
    /// parameters it zeroes leaves every other one at the `parameters.py` value,
    /// which for a SMEFT model is zero for every Wilson coefficient — so a
    /// restricted model evaluated without a card would silently be its
    /// Standard-Model limit while MadGraph's own generated `param_card.dat`
    /// carries the restriction's values. The `evaluate(&default())` assertion is
    /// what fails if that regresses.
    #[test]
    fn test_restrict_nonzero_becomes_the_default() {
        let mut ps = parse_parameters(PARAMS_WITH_MASS_DEP).unwrap();

        let restrict = "Block MASS\n 23 80.0\n".parse::<ParamCard>().unwrap();
        ps.apply_restrict(&restrict);
        assert!(!ps.zeros.contains("MZ"));

        // No card: the restriction's value, not the `parameters.py` default, and
        // the internals that depend on it follow.
        let vals = ps.evaluate(&ParamCard::default());
        assert!((vals["MZ"].re - 80.0).abs() < 1e-10, "{}", vals["MZ"].re);
        assert!((vals["MZ2"].re - 6400.0).abs() < 1e-7, "{}", vals["MZ2"].re);

        // A card overrides it, because a non-zero restriction value is a default
        // and not a lock.
        let user = "Block MASS\n 23 91.1876\n".parse::<ParamCard>().unwrap();
        let vals = ps.evaluate(&user);
        assert!((vals["MZ"].re - 91.1876).abs() < 1e-10);
    }

    #[test]
    fn test_rdeps() {
        let ps = parse_parameters(SM_PARAMS_FRAGMENT).unwrap();
        let g_deps = &ps.rdeps["aS"];
        assert!(g_deps.contains(&"G".to_owned()));
    }
}
