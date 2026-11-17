use super::ast_util::{
    call_func_name, conditional_definitions, extract_float, extract_name, named_assignments,
    parse_stmts, FieldError, Fields, UniqueNames,
};
use super::expr::{collect_deps, eval, parse_expr, Expr};
use super::functions::FunctionLibrary;
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
    #[error(
        "parameter(s) {names} are defined only inside an `if` branch of parameters.py; \
         which branch runs depends on Python evaluated at import time, which the loader does not run"
    )]
    ConditionalDefinition { names: String },
    #[error(
        "internal parameter '{user}' references '{name}', which parameters.py does not define"
    )]
    UnknownParameter { name: String, user: String },
    #[error(
        "parameter '{parameter}': `value` is the name `{name}`, which parameters.py has not \
         defined as a number or a parameter by that point"
    )]
    UnresolvedValueName { parameter: String, name: String },
    #[error(transparent)]
    Field(#[from] FieldError),
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
    pub(crate) complex: bool,
    pub nature: ParamNature,
}

/// The values MadGraph's restriction reads as one and zero without fixing the
/// parameter, each with the value it stands for (`restrict_model`,
/// `import_ufo.py` L2454-L2462; see [`ParameterSet::apply_restrict`]).
pub(crate) const MADGRAPH_FREE_VALUES: [(f64, f64); 2] = [(9.999999e-1, 1.0), (0.000001e-99, 0.0)];

/// Parameters a restriction merged: each one's name and its factor against the
/// first (see [`ParameterSet::merged`]).
pub(crate) type MergedGroup = Vec<(String, f64)>;

/// A complete set of parsed parameters, ready for evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterSet {
    /// External parameters (no deps on other params).
    pub externals: Vec<Parameter>,
    /// Internal parameters in topo-sorted evaluation order.
    pub(crate) internals: Vec<Parameter>,
    /// Reverse dependency map: name → list of parameter names that depend on it.
    pub(crate) rdeps: BTreeMap<String, Vec<String>>,
    /// Const-zero parameters (set by restrictions)
    pub zeros: BTreeSet<String>,
    /// Groups of external parameters a restriction ties together, as MadGraph's
    /// `RestrictModel.detect_identical_parameters` finds them: same block, and
    /// defaults equal up to sign. Each entry is a parameter and its factor
    /// against the group's first (`+1.0` or `-1.0`; the first's is `+1.0`).
    ///
    /// MadGraph keeps the first of each group as the external parameter and
    /// rewrites the others as `factor × first`, so a card must keep the
    /// relation. Derived from the restricted defaults, which the model digest
    /// already covers, so it is left out of the serialized form.
    #[serde(skip)]
    pub(crate) merged: Vec<MergedGroup>,
    /// External parameters a restriction leaves at exactly one. MadGraph's
    /// restriction fixes them (`detect_special_parameters` and
    /// `fix_parameter_values` in `import_ufo.py`, with a `one` rule in the
    /// process's `param_card_rule.dat` that `check_param_card` holds a card to),
    /// so a card must leave them at one. Derived from the restricted defaults,
    /// as [`merged`](Self::merged) is, and left out of the serialized form.
    #[serde(skip)]
    pub(crate) ones: BTreeSet<String>,
}

impl ParameterSet {
    /// Whether `name` is an external or internal parameter of the set.
    pub(crate) fn defines(&self, name: &str) -> bool {
        self.externals
            .iter()
            .chain(&self.internals)
            .any(|p| p.name == name)
    }

    /// What `<variable>.value` is for the parameter bound to the Python
    /// `variable` (`variables` maps each variable to its UFO name): an external
    /// parameter's default value. An internal parameter's `value` is an
    /// expression and a missing one is no parameter, both refused with why.
    pub(crate) fn default_of_variable(
        &self,
        variables: &HashMap<String, String>,
        variable: &str,
    ) -> Result<f64, String> {
        let name = variables
            .get(variable)
            .ok_or_else(|| format!("names `{variable}`, which parameters.py does not define"))?;
        match self.externals.iter().find(|p| &p.name == name) {
            Some(Parameter {
                nature: ParamNature::External { default_value, .. },
                ..
            }) => Ok(*default_value),
            _ => Err(format!(
                "names the internal parameter `{variable}`, whose `value` is an expression"
            )),
        }
    }

    /// The value `card` binds the external parameter `p` to: the card's where it
    /// sets one, the restricted default otherwise. A parameter zeroed by a
    /// restriction is locked to zero: the card cannot revive it (see
    /// [`apply_restrict`](Self::apply_restrict)). `None` for an internal
    /// parameter.
    pub(crate) fn bound_value(&self, p: &Parameter, card: &ParamCard) -> Option<f64> {
        let ParamNature::External {
            default_value,
            lha_block,
            lha_code,
        } = &p.nature
        else {
            return None;
        };
        Some(if self.zeros.contains(&p.name) {
            0.0
        } else {
            card.get(lha_block, lha_code).unwrap_or(*default_value)
        })
    }

    /// Evaluate all parameters using the given param_card for external inputs.
    pub(crate) fn evaluate(&self, slha: &ParamCard) -> HashMap<String, Complex64> {
        let mut values: HashMap<String, Complex64> = HashMap::new();

        for p in &self.externals {
            // A zeroed parameter stays in the map as 0.0: internal params still
            // reference it (e.g. CKM via `lamWS`, `ye` via `yme`), and `eval`
            // panics on a missing name.
            let v = self
                .bound_value(p, slha)
                .expect("externals are external parameters");
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
    ///
    /// The externals the restriction leaves at exactly one are recorded in
    /// [`ones`](Self::ones), and the groups it merges in
    /// [`merged`](Self::merged): a card binding refuses a move of either
    /// (`ufo::bind`), as MadGraph's `param_card_rule.dat` does.
    ///
    /// A value in [`MADGRAPH_FREE_VALUES`] is MadGraph's spelling of one or zero
    /// for a parameter its restriction leaves free. MadGraph tests for 0 and 1
    /// with exact float equality (`detect_special_parameters`, `import_ufo.py`
    /// L2615-L2624), so such a value is neither locked, fixed at one nor merged
    /// (`detect_identical_parameters` skips it, L2724); its couplings are judged
    /// at the value as written; and only then does the value become 1 or 0
    /// (L2454-L2462, and `write_param_card.py` L253-L256 for the generated
    /// card). Here the default becomes 1 or 0 and the parameter stays free;
    /// the couplings are judged against the card itself.
    pub(crate) fn apply_restrict(&mut self, card: &ParamCard) {
        let mut free = BTreeSet::new();
        for p in &mut self.externals {
            if let ParamNature::External {
                default_value,
                lha_block,
                lha_code,
            } = &mut p.nature
            {
                if let Some(v) = card.get(lha_block, lha_code) {
                    if let Some(&(_, stands_for)) = MADGRAPH_FREE_VALUES
                        .iter()
                        .find(|(spelling, _)| *spelling == v)
                    {
                        *default_value = stands_for;
                        free.insert(p.name.clone());
                    } else {
                        *default_value = v;
                        if v.is_zero() {
                            self.zeros.insert(p.name.clone());
                        }
                    }
                }
            }
        }
        self.merged = self.identical_externals();
        self.ones = self
            .externals
            .iter()
            .filter(|p| {
                !free.contains(&p.name)
                    && matches!(p.nature, ParamNature::External { default_value, .. } if default_value == 1.0)
            })
            .map(|p| p.name.clone())
            .collect();
    }

    /// The externals MadGraph's restriction merges
    /// (`import_ufo.py`, `detect_identical_parameters`): over the externals in
    /// model order, those whose restricted default is neither 0 nor 1 and whose
    /// block is not `DECAY`, grouped by `(block, value)`, where a value equal to
    /// the negative of an earlier group's joins that group with factor −1.
    /// Blocks compare as spelled, as MadGraph's dictionary key does.
    fn identical_externals(&self) -> Vec<MergedGroup> {
        let mut groups: Vec<(String, f64, MergedGroup)> = Vec::new();
        for p in &self.externals {
            let ParamNature::External {
                default_value,
                lha_block,
                ..
            } = &p.nature
            else {
                continue;
            };
            let value = *default_value;
            if value == 0.0 || value == 1.0 || lha_block.eq_ignore_ascii_case("decay") {
                continue;
            }
            let existing = groups.iter_mut().find_map(|(block, v, members)| {
                if block != lha_block {
                    None
                } else if *v == value {
                    Some((members, 1.0))
                } else if *v == -value {
                    Some((members, -1.0))
                } else {
                    None
                }
            });
            match existing {
                Some((members, factor)) => members.push((p.name.clone(), factor)),
                None => groups.push((lha_block.clone(), value, vec![(p.name.clone(), 1.0)])),
            }
        }
        groups
            .into_iter()
            .map(|(_, _, members)| members)
            .filter(|members| members.len() > 1)
            .collect()
    }

    /// Every parameter whose value transitively depends on `changed`.
    ///
    /// The set [`recompute`](Self::recompute) re-evaluates, exposed on its own so a
    /// caller can ask *which* parameters a change moves without performing it. A
    /// parameter locked to zero by a restriction moves nothing, so the set is empty.
    pub(crate) fn dependents(&self, changed: &str) -> HashSet<String> {
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

    /// Re-evaluate the transitive dependents of `changed` in place, in
    /// evaluation order, and return them ([`dependents`](Self::dependents)). A
    /// parameter locked to zero by a restriction moves nothing.
    pub(crate) fn recompute(
        &self,
        changed: &str,
        current: &mut HashMap<String, Complex64>,
    ) -> HashSet<String> {
        let affected = self.dependents(changed);
        for p in &self.internals {
            if affected.contains(&p.name) {
                if let ParamNature::Internal { expr, .. } = &p.nature {
                    let v = eval(expr, current);
                    current.insert(p.name.clone(), v);
                }
            }
        }
        affected
    }
}

/// Parse `parameters.py` content into a [`ParameterSet`].
#[cfg(test)]
pub(crate) fn parse_parameters(content: &str) -> Result<ParameterSet, ParameterError> {
    parse_parameters_with_variables(content, &FunctionLibrary::default()).map(|(set, _)| set)
}

/// Parse `parameters.py` content into a [`ParameterSet`], with each
/// parameter's Python variable mapped to its UFO `name`.
///
/// The `name` is the canonical name, as in MadGraph: a value string refers to
/// a parameter by it (`model_reader.py` evaluates each one with `param.name`
/// bound), and a `Param.<variable>` reference in another file is the object,
/// which UFO's `UFOBaseClass.__str__` writes as its `name`. The map is how such
/// a reference is resolved.
pub(crate) fn parse_parameters_with_variables(
    content: &str,
    library: &FunctionLibrary,
) -> Result<(ParameterSet, HashMap<String, String>), ParameterError> {
    let stmts = parse_stmts(content).map_err(|e| ParameterError::Parse(e.to_string()))?;

    if let Some(names) = conditional_definitions(&stmts, "Parameter") {
        return Err(ParameterError::ConditionalDefinition { names });
    }

    let mut externals: Vec<Parameter> = Vec::new();
    let mut raw_internals: Vec<(String, bool, Expr, Vec<String>)> = Vec::new();

    // `Parameter.require_args` in UFO's object library
    // (`mg5amcnlo/models/sm/object_library.py` L148) lists every field read here
    // but `lhablock` and `lhacode`, which its constructor requires of an external
    // parameter (L161). MadGraph refuses a name defined twice (`import_ufo.py`,
    // `UFOMG5Converter.load_model`, "define multiple time").
    let mut seen = UniqueNames::new("parameter");
    let mut variables: HashMap<String, String> = HashMap::new();
    // Numbers assigned at module level so far: a `value = NAME` reads the one
    // bound when the `Parameter` statement runs.
    let mut numbers: HashMap<&str, f64> = HashMap::new();
    for (python_name, assigned) in named_assignments(&stmts) {
        let ast::Expr::Call(call) = assigned else {
            if let Some(x) = extract_float(assigned) {
                numbers.insert(python_name, x);
            }
            continue;
        };
        if call_func_name(&call.func) != Some("Parameter") {
            continue;
        }
        let fields = Fields::new("parameter", python_name, &call.keywords);
        let name = fields.str("name")?;
        seen.insert(python_name, &name)?;
        variables.insert(python_name.to_owned(), name.clone());
        let is_complex = match fields.str("type")?.as_str() {
            "real" => false,
            "complex" => true,
            _ => return Err(fields.malformed("type", "'real' or 'complex'").into()),
        };
        fields.str("texname")?;

        match fields.str("nature")?.as_str() {
            "external" => {
                let default_value = match fields.required("value").map(extract_name) {
                    Ok(Some(n)) => *numbers.get(n).ok_or_else(|| unresolved(&name, n))?,
                    _ => fields.float("value")?,
                };
                let lha_block = fields.str("lhablock")?;
                let lha_code = fields.int32_list("lhacode")?;
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
            "internal" => {
                let expr = match extract_name(fields.required("value")?) {
                    // The value is a Python name: a `Parameter` object, which MadGraph
                    // writes into the expression as its UFO `name`, or a number.
                    Some(n) => match (variables.get(n), numbers.get(n)) {
                        (Some(target), _) => Expr::Param(target.clone()),
                        (None, Some(x)) => Expr::Num(*x),
                        (None, None) => return Err(unresolved(&name, n)),
                    },
                    None => {
                        let expr_str = fields.str("value")?;
                        parse_expr(&expr_str)
                            .map(|e| library.resolve(e))
                            .map_err(|e| ParameterError::ExprParse {
                                name: name.clone(),
                                cause: e.to_string(),
                            })?
                    }
                };
                let mut deps = Vec::new();
                collect_deps(&expr, &mut deps);
                raw_internals.push((name, is_complex, expr, deps));
            }
            _ => {
                return Err(fields
                    .malformed("nature", "'external' or 'internal'")
                    .into())
            }
        }
    }

    let defined: HashSet<&str> = externals
        .iter()
        .map(|p| p.name.as_str())
        .chain(raw_internals.iter().map(|(name, ..)| name.as_str()))
        .collect();
    for (user, _, _, deps) in &raw_internals {
        if let Some(name) = deps.iter().find(|d| !defined.contains(d.as_str())) {
            return Err(ParameterError::UnknownParameter {
                name: name.clone(),
                user: user.clone(),
            });
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

    let set = ParameterSet {
        externals,
        internals,
        rdeps,
        zeros: BTreeSet::new(),
        merged: Vec::new(),
        ones: BTreeSet::new(),
    };
    Ok((set, variables))
}

fn unresolved(parameter: &str, name: &str) -> ParameterError {
    ParameterError::UnresolvedValueName {
        parameter: parameter.to_owned(),
        name: name.to_owned(),
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

    /// MadGraph's spellings of a free one (`9.999999e-1`) and a free zero
    /// (`0.000001e-99`) become defaults of exactly one and zero, and the
    /// parameter stays free: not locked to zero, not fixed at one, not merged
    /// with its equal neighbour. Exactly 1 is fixed and exactly 0 locked, and
    /// −1 is neither. Each half fails on its own: reading the spellings as
    /// written leaves `a` at 0.9999999 and merges `a` with `b`; reading them as
    /// one and zero puts `a`, `b` in `ones` and `c` in `zeros`.
    #[test]
    fn madgraphs_free_spellings_become_free_defaults() {
        let src = r"
a = Parameter(name = 'a', nature = 'external', type = 'real', value = 0.3, texname = 'a', lhablock = 'NP', lhacode = [ 1 ])
b = Parameter(name = 'b', nature = 'external', type = 'real', value = 0.3, texname = 'b', lhablock = 'NP', lhacode = [ 2 ])
c = Parameter(name = 'c', nature = 'external', type = 'real', value = 0.3, texname = 'c', lhablock = 'NP', lhacode = [ 3 ])
d = Parameter(name = 'd', nature = 'external', type = 'real', value = 0.3, texname = 'd', lhablock = 'NP', lhacode = [ 4 ])
e = Parameter(name = 'e', nature = 'external', type = 'real', value = 0.3, texname = 'e', lhablock = 'NP', lhacode = [ 5 ])
f = Parameter(name = 'f', nature = 'external', type = 'real', value = 0.3, texname = 'f', lhablock = 'NP', lhacode = [ 6 ])
";
        let mut ps = parse_parameters(src).unwrap();
        let card =
            "Block NP\n 1 9.999999e-01\n 2 0.9999999\n 3 0.000001e-99\n 4 1.0\n 5 0\n 6 -1\n"
                .parse::<ParamCard>()
                .unwrap();
        ps.apply_restrict(&card);

        let defaults: Vec<(&str, f64)> = ps
            .externals
            .iter()
            .map(|p| match p.nature {
                ParamNature::External { default_value, .. } => (p.name.as_str(), default_value),
                _ => unreachable!(),
            })
            .collect();
        assert_eq!(
            defaults,
            [
                ("a", 1.0),
                ("b", 1.0),
                ("c", 0.0),
                ("d", 1.0),
                ("e", 0.0),
                ("f", -1.0)
            ]
        );
        assert_eq!(ps.ones.iter().collect::<Vec<_>>(), ["d"]);
        assert_eq!(ps.zeros.iter().collect::<Vec<_>>(), ["e"]);
        assert!(ps.merged.is_empty(), "{:?}", ps.merged);

        // Free: a card moves them, and they evaluate to their defaults without one.
        let vals = ps.evaluate(&ParamCard::default());
        assert_eq!((vals["a"].re, vals["c"].re), (1.0, 0.0));
        let user = "Block NP\n 1 0.5\n 3 0.25\n".parse::<ParamCard>().unwrap();
        let vals = ps.evaluate(&user);
        assert_eq!((vals["a"].re, vals["c"].re), (0.5, 0.25));
    }

    #[test]
    fn test_rdeps() {
        let ps = parse_parameters(SM_PARAMS_FRAGMENT).unwrap();
        let g_deps = &ps.rdeps["aS"];
        assert!(g_deps.contains(&"G".to_owned()));
    }

    #[test]
    fn conditional_definitions_are_refused_by_name_at_any_depth() {
        let src = r"
if configuration.a:
    MW = Parameter(name = 'MW', nature = 'internal', type = 'real', value = '80.0')
elif configuration.b:
    if configuration.c:
        MZ = Parameter(name = 'MZ', nature = 'internal', type = 'real', value = '91.0')
";
        let err = parse_parameters(src).unwrap_err();
        assert!(
            matches!(&err, ParameterError::ConditionalDefinition { names } if names == "'MW', 'MZ'"),
            "{err}"
        );
    }

    #[test]
    fn conditional_non_parameter_statements_are_not_refused() {
        let src = r"
if configuration.a:
    helper = Helper(value = 1.0)
    plain = 1.0
ZERO = Parameter(name = 'ZERO', nature = 'internal', type = 'real', value = '0.0', texname = '0')
";
        assert_eq!(parse_parameters(src).unwrap().internals.len(), 1);
    }

    #[test]
    fn internal_expression_naming_an_undefined_parameter_is_refused() {
        let src = r"
a = Parameter(name = 'a', nature = 'internal', type = 'real', value = '2*b', texname = 'a')
";
        assert!(matches!(
            parse_parameters(src).unwrap_err(),
            ParameterError::UnknownParameter { ref name, ref user } if name == "b" && user == "a"
        ));
    }
}
