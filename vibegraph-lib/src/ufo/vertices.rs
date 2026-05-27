use super::ast_util::{
    call_func_name, extract_attr, extract_int, extract_str, kwarg_str, parse_stmts,
};
use super::couplings::CouplingId;
use super::lorentz::LorentzId;
use super::particles::ParticleId;
use rustpython_parser::ast;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VertexError {
    #[error("Failed to parse vertices.py: {0}")]
    Parse(String),
}

/// Opaque index into `UFOModel::vertices`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexId(pub usize);

/// A fully resolved interaction vertex.
#[derive(Clone)]
pub struct Vertex {
    pub name: String,
    pub particles: Vec<ParticleId>,
    /// Color factor strings, e.g. `["1"]`, `["f(1,2,3)"]`.
    pub color: Vec<String>,
    pub lorentz: Vec<LorentzId>,
    /// `(lorentz_idx, color_idx)` → coupling id.
    pub couplings: HashMap<(usize, usize), CouplingId>,
}

/// Intermediate form from the Python AST parse; names not yet resolved to IDs.
pub(crate) struct RawVertex {
    pub name: String,
    pub particles: Vec<String>,
    pub color: Vec<String>,
    pub lorentz: Vec<String>,
    pub couplings: HashMap<(usize, usize), String>,
}

/// Parse `vertices.py` content into raw vertices (names, not IDs).
pub(crate) fn parse_vertices(src: &str) -> Result<Vec<RawVertex>, VertexError> {
    let stmts = parse_stmts(src).map_err(|e| VertexError::Parse(e.to_string()))?;
    let mut result = Vec::new();

    for stmt in &stmts {
        let ast::Stmt::Assign(ast::StmtAssign { targets, value, .. }) = stmt else {
            continue;
        };
        let ast::Expr::Name(ast::ExprName { .. }) = targets.first().unwrap() else {
            continue;
        };

        let ast::Expr::Call(ast::ExprCall { func, keywords, .. }) = value.as_ref() else {
            continue;
        };
        if call_func_name(func) != Some("Vertex") {
            continue;
        }

        let name = kwarg_str(keywords, "name").unwrap_or_default();
        let particles = extract_name_list(keywords, "particles", "P");
        let color = extract_str_list(keywords, "color");
        let lorentz = extract_name_list(keywords, "lorentz", "L");
        let couplings = extract_couplings_dict(keywords);

        result.push(RawVertex {
            name,
            particles,
            color,
            lorentz,
            couplings,
        });
    }

    Ok(result)
}

/// Extract a list of `Prefix.Name` references, returning just the `Name` part.
fn extract_name_list(keywords: &[ast::Keyword], kw: &str, _expected_prefix: &str) -> Vec<String> {
    use super::ast_util::get_kwarg;
    let Some(val) = get_kwarg(keywords, kw) else {
        return vec![];
    };
    let ast::Expr::List(ast::ExprList { elts, .. }) = val else {
        return vec![];
    };
    elts.iter()
        .filter_map(|e| extract_attr(e).map(|(_, attr)| attr.to_owned()))
        .collect()
}

/// Extract a list of string literals.
fn extract_str_list(keywords: &[ast::Keyword], kw: &str) -> Vec<String> {
    use super::ast_util::get_kwarg;
    let Some(val) = get_kwarg(keywords, kw) else {
        return vec![];
    };
    let ast::Expr::List(ast::ExprList { elts, .. }) = val else {
        return vec![];
    };
    elts.iter()
        .filter_map(|e| extract_str(e).map(|s| s.to_owned()))
        .collect()
}

/// Extract `couplings = {(0,0): C.GC_3, ...}` into a map.
fn extract_couplings_dict(keywords: &[ast::Keyword]) -> HashMap<(usize, usize), String> {
    use super::ast_util::get_kwarg;
    let Some(val) = get_kwarg(keywords, "couplings") else {
        return HashMap::new();
    };
    let ast::Expr::Dict(ast::ExprDict { keys, values, .. }) = val else {
        return HashMap::new();
    };

    keys.iter()
        .zip(values.iter())
        .filter_map(|(k, v)| {
            let key_expr = k.as_ref()?;
            let ast::Expr::Tuple(ast::ExprTuple { elts, .. }) = key_expr else {
                return None;
            };
            if elts.len() != 2 {
                return None;
            }
            let i = extract_int(&elts[0])? as usize;
            let j = extract_int(&elts[1])? as usize;
            let coup_name = extract_attr(v).map(|(_, attr)| attr.to_owned())?;
            Some(((i, j), coup_name))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r"
from object_library import all_vertices, Vertex
import particles as P
import couplings as C
import lorentz as L

V_1 = Vertex(name = 'V_1',
             particles = [ P.G0, P.G0, P.G0, P.G0 ],
             color = [ '1' ],
             lorentz = [ L.SSSS1 ],
             couplings = {(0,0):C.GC_33})

V_73 = Vertex(name = 'V_73',
              particles = [ P.A, P.e__minus__, P.e__plus__ ],
              color = [ '1' ],
              lorentz = [ L.FFV1, L.FFV2 ],
              couplings = {(0,0):C.GC_3,(0,1):C.GC_50})
";

    #[test]
    fn test_single_coupling() {
        let vs = parse_vertices(SAMPLE).unwrap();
        let v1 = vs.iter().find(|v| v.name == "V_1").unwrap();
        assert_eq!(v1.particles, vec!["G0", "G0", "G0", "G0"]);
        assert_eq!(v1.lorentz, vec!["SSSS1"]);
        assert_eq!(v1.couplings.get(&(0, 0)), Some(&"GC_33".to_owned()));
    }

    #[test]
    fn test_multiple_lorentz() {
        let vs = parse_vertices(SAMPLE).unwrap();
        let v73 = vs.iter().find(|v| v.name == "V_73").unwrap();
        assert_eq!(v73.lorentz, vec!["FFV1", "FFV2"]);
        assert_eq!(v73.couplings.get(&(0, 0)), Some(&"GC_3".to_owned()));
        assert_eq!(v73.couplings.get(&(0, 1)), Some(&"GC_50".to_owned()));
    }

    #[test]
    fn test_color_strings() {
        let vs = parse_vertices(SAMPLE).unwrap();
        let v73 = vs.iter().find(|v| v.name == "V_73").unwrap();
        assert_eq!(v73.color, vec!["1"]);
    }
}
