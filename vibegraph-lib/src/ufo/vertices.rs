use super::ast_util::{
    constructor_calls, extract_attr, extract_int, extract_str, parse_stmts, FieldError, Fields,
    UniqueNames,
};
use super::color::ColorExpr;
use super::couplings::CouplingId;
use super::lorentz::LorentzId;
use super::particles::{Particle, ParticleId};
use indexmap::IndexMap;
use rustpython_parser::ast;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VertexError {
    #[error("Failed to parse vertices.py: {0}")]
    Parse(String),
    #[error(transparent)]
    Field(#[from] FieldError),
}

/// Opaque index into `UFOModel::vertices`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) struct VertexId(pub(crate) usize);

/// A fully resolved interaction vertex.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vertex {
    pub(crate) name: String,
    pub particles: Vec<ParticleId>,
    /// Parsed, `Identity`-resolved color factors, one per color structure
    /// (e.g. `[ColorExpr { atoms: [] }]` for `["1"]`, or a `T`/`f` product).
    pub(crate) color: Vec<ColorExpr>,
    pub lorentz: Vec<LorentzId>,
    /// `(color_idx, lorentz_idx)` → coupling id.
    pub couplings: BTreeMap<(usize, usize), CouplingId>,
}

impl Vertex {
    /// Whether the vertex survives unitary gauge: none of its legs is a
    /// Goldstone mode or a ghost ([`Particle::is_unphysical`]). Diagram
    /// enumeration builds its topology model from exactly these vertices.
    pub fn is_physical(&self, particles: &IndexMap<String, Particle>) -> bool {
        !self
            .particles
            .iter()
            .any(|&pid| particles[pid].is_unphysical())
    }
}

/// Intermediate form from the Python AST parse; names not yet resolved to IDs.
pub(crate) struct RawVertex {
    pub(crate) name: String,
    pub(crate) particles: Vec<String>,
    pub(crate) color: Vec<String>,
    pub(crate) lorentz: Vec<String>,
    /// Each `(color, lorentz)` entry's couplings: one, or the members of a list.
    pub(crate) couplings: BTreeMap<(usize, usize), Vec<String>>,
}

/// A vertex with its names resolved and its couplings not yet separated by
/// order: a `(color, lorentz)` entry may carry several couplings.
pub(crate) struct UnsplitVertex {
    pub(crate) name: String,
    pub(crate) particles: Vec<ParticleId>,
    pub(crate) color: Vec<ColorExpr>,
    pub(crate) lorentz: Vec<LorentzId>,
    /// `((color_idx, lorentz_idx), coupling)` in entry order, a list's members
    /// in list order.
    pub(crate) couplings: Vec<((usize, usize), CouplingId)>,
}

/// Parse `vertices.py` content into raw vertices (names, not IDs).
pub(crate) fn parse_vertices(src: &str) -> Result<Vec<RawVertex>, VertexError> {
    let stmts = parse_stmts(src).map_err(|e| VertexError::Parse(e.to_string()))?;
    let mut result = Vec::new();
    let mut seen = UniqueNames::new("vertex");

    // `Vertex.require_args` in UFO's object library
    // (`mg5amcnlo/models/sm/object_library.py` L170) lists all five fields. The
    // loader keys vertices by `name`, so a repeated one is refused
    // ([`UniqueNames`]).
    for (python_name, keywords) in constructor_calls(&stmts, "Vertex") {
        let fields = Fields::new("vertex", python_name, keywords);
        let name = fields.str("name")?;
        seen.insert(python_name, &name)?;
        let particles = extract_name_list(&fields, "particles", "P", &name)?;
        let color = extract_str_list(&fields, "color")?;
        let lorentz = extract_name_list(&fields, "lorentz", "L", &name)?;
        let couplings = extract_couplings_dict(&fields)?;

        // Validation: check all coupling indices are in range of lorentz/color lists
        for (&(color_idx, lor_idx), coup_names) in &couplings {
            let coup_name = coup_names.join(", ");
            if lor_idx >= lorentz.len() {
                return Err(VertexError::Parse(format!(
                    "Coupling {} references invalid lorentz index {} (only {} provided)",
                    coup_name,
                    lor_idx,
                    lorentz.len()
                )));
            }
            if color_idx >= color.len() {
                return Err(VertexError::Parse(format!(
                    "Coupling {} references invalid color index {} (only {} provided)",
                    coup_name,
                    color_idx,
                    color.len()
                )));
            }
        }

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

/// Extract a list of `prefix.Name` references (`P.e__minus__` under
/// `particles`, `L.FFV1` under `lorentz`), returning just the `Name` part. An
/// entry that is not an attribute of `prefix` is an error naming `vertex`.
fn extract_name_list(
    fields: &Fields<'_>,
    kw: &'static str,
    prefix: &str,
    vertex: &str,
) -> Result<Vec<String>, VertexError> {
    let ast::Expr::List(ast::ExprList { elts, .. }) = fields.required(kw)? else {
        return Err(fields.malformed(kw, "a list").into());
    };
    elts.iter()
        .map(|e| match extract_attr(e) {
            Some((module, attr)) if module == prefix => Ok(attr.to_owned()),
            _ => Err(VertexError::Parse(format!(
                "vertex '{vertex}': a `{kw}` entry is not a `{prefix}.<name>` reference"
            ))),
        })
        .collect()
}

/// Extract a list of string literals.
fn extract_str_list(fields: &Fields<'_>, kw: &'static str) -> Result<Vec<String>, FieldError> {
    const EXPECTED: &str = "a list of strings";
    let ast::Expr::List(ast::ExprList { elts, .. }) = fields.required(kw)? else {
        return Err(fields.malformed(kw, EXPECTED));
    };
    elts.iter()
        .map(|e| {
            extract_str(e)
                .map(str::to_owned)
                .ok_or_else(|| fields.malformed(kw, EXPECTED))
        })
        .collect()
}

/// Extract `couplings = {(0,0): C.GC_3, (0,1): [C.GC_4, C.GC_5]}` into a map
/// from each index pair to its coupling names: a list's, or the one.
fn extract_couplings_dict(
    fields: &Fields<'_>,
) -> Result<BTreeMap<(usize, usize), Vec<String>>, FieldError> {
    const EXPECTED: &str = "a dict of `(colour, lorentz)` index pairs to a `C.<name>` reference \
                            or a non-empty list of them";
    let malformed = || fields.malformed("couplings", EXPECTED);
    let ast::Expr::Dict(ast::ExprDict { keys, values, .. }) = fields.required("couplings")? else {
        return Err(malformed());
    };
    let index = |e: &ast::Expr| extract_int(e).and_then(|n| usize::try_from(n).ok());
    keys.iter()
        .zip(values.iter())
        .map(|(k, v)| {
            let Some(ast::Expr::Tuple(ast::ExprTuple { elts, .. })) = k.as_ref() else {
                return Err(malformed());
            };
            let [i, j] = elts.as_slice() else {
                return Err(malformed());
            };
            let reference = |e: &ast::Expr| match extract_attr(e) {
                Some(("C", coupling)) => Some(coupling.to_owned()),
                _ => None,
            };
            let names = match v {
                ast::Expr::List(ast::ExprList { elts, .. }) if !elts.is_empty() => {
                    elts.iter().map(reference).collect::<Option<Vec<_>>>()
                }
                other => reference(other).map(|c| vec![c]),
            };
            match (index(i), index(j), names) {
                (Some(i), Some(j), Some(names)) => Ok(((i, j), names)),
                _ => Err(malformed()),
            }
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
        assert_eq!(v1.couplings.get(&(0, 0)), Some(&vec!["GC_33".to_owned()]));
    }

    #[test]
    fn test_multiple_lorentz() {
        let vs = parse_vertices(SAMPLE).unwrap();
        let v73 = vs.iter().find(|v| v.name == "V_73").unwrap();
        assert_eq!(v73.lorentz, vec!["FFV1", "FFV2"]);
        assert_eq!(v73.couplings.get(&(0, 0)), Some(&vec!["GC_3".to_owned()]));
        assert_eq!(v73.couplings.get(&(0, 1)), Some(&vec!["GC_50".to_owned()]));
    }

    #[test]
    fn test_color_strings() {
        let vs = parse_vertices(SAMPLE).unwrap();
        let v73 = vs.iter().find(|v| v.name == "V_73").unwrap();
        assert_eq!(v73.color, vec!["1"]);
    }

    #[test]
    fn a_list_of_couplings_keeps_its_members_in_order() {
        let src = "V_1 = Vertex(name = 'V_1', particles = [ P.A, P.A ], color = [ '1' ], \
                   lorentz = [ L.VV1, L.VV2 ], \
                   couplings = {(0,0):[ C.GC_2, C.GC_1 ],(0,1):C.GC_3})\n";
        let v = &parse_vertices(src).unwrap()[0];
        assert_eq!(v.couplings[&(0, 0)], ["GC_2", "GC_1"]);
        assert_eq!(v.couplings[&(0, 1)], ["GC_3"]);
    }

    #[test]
    fn an_empty_or_mixed_coupling_list_is_refused() {
        for entry in [
            "[]",
            "[ C.GC_1, 2 ]",
            "[ C.GC_1, L.GC_2 ]",
            "( C.GC_1, C.GC_2 )",
        ] {
            let src = format!(
                "V_1 = Vertex(name = 'V_1', particles = [ P.A, P.A ], color = [ '1' ], \
                 lorentz = [ L.VV1 ], couplings = {{(0,0):{entry}}})\n"
            );
            assert!(
                matches!(
                    parse_vertices(&src),
                    Err(VertexError::Field(FieldError::Malformed { .. }))
                ),
                "{entry}"
            );
        }
    }

    /// A `particles` or `lorentz` entry that is not a reference into its own
    /// module is an error, not a dropped or misfiled entry.
    #[test]
    fn a_name_list_entry_outside_its_module_is_refused() {
        let vertex = |particles: &str, lorentz: &str| {
            format!(
                "V_1 = Vertex(name = 'V_1', particles = [ {particles} ], color = [ '1' ], \
                 lorentz = [ {lorentz} ], couplings = {{(0,0):C.GC_33}})\n"
            )
        };
        assert!(parse_vertices(&vertex("P.G0, P.G0", "L.SSS1")).is_ok());
        for (particles, lorentz) in [
            ("P.G0, L.G0", "L.SSS1"),
            ("P.G0, G0", "L.SSS1"),
            ("P.G0, P.G0", "P.SSS1"),
            ("P.G0, P.G0", "'SSS1'"),
        ] {
            let result = parse_vertices(&vertex(particles, lorentz));
            assert!(
                matches!(result, Err(VertexError::Parse(_))),
                "particles [{particles}], lorentz [{lorentz}]: {:?}",
                result.map(|v| v.len())
            );
        }
    }
}
