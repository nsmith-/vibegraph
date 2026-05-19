//! Re-parses UFO `vertices.py` to extract the `(lorentz_idx, color_idx) → coupling_name`
//! association per vertex.
//!
//! FeynGraph reads `vertices.py` but discards the actual coupling name (e.g. `"GC_10"`),
//! storing only the coupling order. We re-parse it here to recover the name so we can
//! evaluate the numerical coupling value.
//!
//! Example from SM UFO:
//! ```python
//! V_73 = Vertex(name = 'V_73',
//!               particles = [ P.A, P.e__minus__, P.e__plus__ ],
//!               color = [ '1' ],
//!               lorentz = [ L.FFV1, L.FFV2 ],
//!               couplings = {(0,0):C.GC_3, (0,1):C.GC_50})
//! ```

use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VertexExtError {
    #[error("Failed to parse vertices.py for coupling associations: {0}")]
    Parse(String),
}

/// Mapping from original vertex Python name to its coupling dictionary.
///
/// The inner map: `(lorentz_idx, color_idx) → coupling_name`
pub type VertexCouplingMap = HashMap<String, HashMap<(usize, usize), String>>;

/// Parse `vertices.py` and return the coupling association map.
pub fn parse_vertex_couplings(content: &str) -> Result<VertexCouplingMap, VertexExtError> {
    let raw = ufo_vertices_ext::vertices(content)
        .map_err(|e| VertexExtError::Parse(e.to_string()))?;

    Ok(raw
        .into_iter()
        .map(|(name, couplings)| (name, couplings.into_iter().collect()))
        .collect())
}

peg::parser! {
    grammar ufo_vertices_ext() for str {

        /// Returns (vertex_name, coupling_dict) pairs.
        pub rule vertices() -> Vec<(String, Vec<((usize, usize), String)>)>
            = _ v:(vertex() ** _) _ { v }

        rule vertex() -> (String, Vec<((usize, usize), String)>)
            = _ name:ident() _ "=" _ "Vertex(" _ props:(prop() ** (_ "," _)) _ ","? _ ")" _ {
                let mut couplings: Vec<((usize, usize), String)> = Vec::new();
                for (k, v) in props {
                    if k == "couplings" {
                        couplings = v;
                    }
                }
                (name.to_owned(), couplings)
            }

        rule prop() -> (String, Vec<((usize, usize), String)>)
            = _ k:ident() _ "=" _ v:prop_value(k) _ { (k.to_owned(), v) }

        rule prop_value(key: &str) -> Vec<((usize, usize), String)>
            = c:coupling_dict() { c }  // handles couplings = {...}
            / other_value() { vec![] }

        rule coupling_dict() -> Vec<((usize, usize), String)>
            = "{" _ entries:(coupling_entry() ** (_ "," _)) _ ","? _ "}" { entries }

        rule coupling_entry() -> ((usize, usize), String)
            = "(" _ i:uint() _ "," _ j:uint() _ ")" _ ":" _ "C." name:ident() {
                ((i, j), name.to_owned())
            }

        /// Consume any other property value (list, string, identifiers, etc.)
        rule other_value()
            = "[" (!"]" [_])* "]"
            / "'" [^'\'']* "'"
            / "\"" [^'"']* "\""
            / ident_chain()

        rule ident_chain()
            = ident() ("." ident() / "[" [^']']* "]")*

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
        let map = parse_vertex_couplings(SAMPLE).unwrap();
        let v1 = &map["V_1"];
        assert_eq!(v1.get(&(0, 0)), Some(&"GC_33".to_owned()));
    }

    #[test]
    fn test_multiple_lorentz() {
        let map = parse_vertex_couplings(SAMPLE).unwrap();
        let v73 = &map["V_73"];
        assert_eq!(v73.get(&(0, 0)), Some(&"GC_3".to_owned()));
        assert_eq!(v73.get(&(0, 1)), Some(&"GC_50".to_owned()));
    }
}
