use super::ast_util::{call_func_name, get_kwarg, kwarg_str, parse_stmts};
use rustpython_parser::ast;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LorentzError {
    #[error("Failed to parse lorentz.py: {0}")]
    Parse(String),
    #[error("Failed to parse Lorentz structure '{name}': {cause}")]
    StructureParse { name: String, cause: String },
}

/// Opaque index into `UFOModel::lorentz`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LorentzId(pub usize);

/// A single Lorentz tensor operator.
#[derive(Debug, Clone, PartialEq)]
pub enum LorentzOp {
    /// Dirac gamma matrix: Γ^μ_{ij}
    Gamma { mu: i32, i: i32, j: i32 },
    /// Sigma: σ^{μν}_{ij} = i/2 [γ^μ, γ^ν]
    Sigma { mu: i32, nu: i32, i: i32, j: i32 },
    /// Identity in spinor space: δ_{ij}
    Identity { i: i32, j: i32 },
    /// Left projector: P_L = (1 - γ^5)/2
    ProjM { i: i32, j: i32 },
    /// Right projector: P_R = (1 + γ^5)/2
    ProjP { i: i32, j: i32 },
    /// Metric tensor: g^{μν}
    Metric { mu: i32, nu: i32 },
    /// Momentum insertion: p_leg^μ
    P { mu: i32, leg: i32 },
    /// Levi-Civita tensor: ε^{μνρσ}
    Epsilon {
        mu: i32,
        nu: i32,
        rho: i32,
        sigma: i32,
    },
    /// Charge-conjugation matrix: C_{ij}
    C { i: i32, j: i32 },
}

/// A term in a Lorentz structure: `coeff * op1 * op2 * ...`
#[derive(Debug, Clone, PartialEq)]
pub struct LorentzTerm {
    pub coeff: f64,
    pub ops: Vec<LorentzOp>,
}

/// A Lorentz structure expression: sum of `LorentzTerm`s.
pub type LorentzExpr = Vec<LorentzTerm>;

/// A Lorentz tensor structure from `lorentz.py`.
#[derive(Debug, Clone)]
pub struct LorentzStructure {
    /// Python variable name, e.g. `"FFV1"`.
    pub python_name: String,
    /// UFO `name` field.
    pub name: String,
    /// External leg spins (2s+1 per leg).
    pub spins: Vec<i32>,
    /// Verbatim `structure` string from the UFO file.
    pub structure: String,
    /// Parsed symbolic expression.
    pub expr: LorentzExpr,
}

/// Parse `lorentz.py` content into a list of [`LorentzStructure`]s.
pub fn parse_lorentz(src: &str) -> Result<Vec<LorentzStructure>, LorentzError> {
    let stmts = parse_stmts(src).map_err(|e| LorentzError::Parse(e.to_string()))?;
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
        if call_func_name(func) != Some("Lorentz") {
            continue;
        }

        let name = kwarg_str(keywords, "name").unwrap_or_else(|| python_name.clone());
        let spins = extract_spins(keywords);
        let structure = kwarg_str(keywords, "structure").unwrap_or_default();

        let expr =
            lorentz_structure::structure(&structure).map_err(|e| LorentzError::StructureParse {
                name: name.clone(),
                cause: e.to_string(),
            })?;

        result.push(LorentzStructure {
            python_name,
            name,
            spins,
            structure,
            expr,
        });
    }

    Ok(result)
}

fn extract_spins(keywords: &[ast::Keyword]) -> Vec<i32> {
    use super::ast_util::extract_int;
    let Some(val) = get_kwarg(keywords, "spins") else {
        return vec![];
    };
    let ast::Expr::List(ast::ExprList { elts, .. }) = val else {
        return vec![];
    };
    elts.iter()
        .filter_map(extract_int)
        .map(|n| n as i32)
        .collect()
}

peg::parser! {
    grammar lorentz_structure() for str {
        pub rule structure() -> LorentzExpr
            = _ terms:term_list() _ { terms }

        rule term_list() -> LorentzExpr
            = head:term() tail:( _ sign:sign() _ t:unsigned_term() { (sign, t) } )* {
                let mut result = vec![head];
                for (sign, mut t) in tail {
                    t.coeff *= sign;
                    result.push(t);
                }
                result
            }

        rule term() -> LorentzTerm
            = s:sign()? _ t:unsigned_term() {
                let mut t = t;
                t.coeff *= s.unwrap_or(1.0);
                t
            }

        rule unsigned_term() -> LorentzTerm
            = head:factor() tail:( _ "*" _ f:factor() { f } )* {
                let mut coeff = 1.0;
                let mut ops = Vec::new();
                // collect coefficient from numeric factors; ops from operator factors
                let all_factors = std::iter::once(head).chain(tail);
                for f in all_factors {
                    match f {
                        Factor::Num(n) => coeff *= n,
                        Factor::Op(op) => ops.push(op),
                    }
                }
                LorentzTerm { coeff, ops }
            }

        rule factor() -> Factor
            = n:number() { Factor::Num(n) }
            / op:operator() { Factor::Op(op) }
            / "(" _ t:term() _ ")" { Factor::Op(LorentzOp::Identity { i: 0, j: 0 }) } // shouldn't occur

        rule operator() -> LorentzOp
            = "Gamma(" _ mu:idx() _ "," _ i:idx() _ "," _ j:idx() _ ")" {
                LorentzOp::Gamma { mu, i, j }
            }
            / "Sigma(" _ mu:idx() _ "," _ nu:idx() _ "," _ i:idx() _ "," _ j:idx() _ ")" {
                LorentzOp::Sigma { mu, nu, i, j }
            }
            / "Identity(" _ i:idx() _ "," _ j:idx() _ ")" {
                LorentzOp::Identity { i, j }
            }
            / "ProjM(" _ i:idx() _ "," _ j:idx() _ ")" {
                LorentzOp::ProjM { i, j }
            }
            / "ProjP(" _ i:idx() _ "," _ j:idx() _ ")" {
                LorentzOp::ProjP { i, j }
            }
            / "Metric(" _ mu:idx() _ "," _ nu:idx() _ ")" {
                LorentzOp::Metric { mu, nu }
            }
            / "P(" _ mu:idx() _ "," _ leg:idx() _ ")" {
                LorentzOp::P { mu, leg }
            }
            / "Epsilon(" _ mu:idx() _ "," _ nu:idx() _ "," _ rho:idx() _ "," _ sigma:idx() _ ")" {
                LorentzOp::Epsilon { mu, nu, rho, sigma }
            }
            / "C(" _ i:idx() _ "," _ j:idx() _ ")" {
                LorentzOp::C { i, j }
            }

        // Identity written as bare "1"
        rule number() -> f64
            = n:$(['0'..='9']+ ("." ['0'..='9']*)?) {? n.parse().or(Err("number")) }

        rule idx() -> i32
            = "-" n:$(['0'..='9']+) {? n.parse::<i32>().map(|v| -v).or(Err("idx")) }
            / n:$(['0'..='9']+) {? n.parse().or(Err("idx")) }

        rule sign() -> f64
            = "+" { 1.0 }
            / "-" { -1.0 }

        rule _ = [' ' | '\t' | '\n' | '\r']*
    }
}

enum Factor {
    Num(f64),
    Op(LorentzOp),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_identity() {
        let expr = lorentz_structure::structure("1").unwrap();
        assert_eq!(expr.len(), 1);
        assert_eq!(expr[0].coeff, 1.0);
        assert!(expr[0].ops.is_empty());
    }

    #[test]
    fn test_parse_gamma() {
        let expr = lorentz_structure::structure("Gamma(3,2,1)").unwrap();
        assert_eq!(expr.len(), 1);
        assert_eq!(expr[0].ops[0], LorentzOp::Gamma { mu: 3, i: 2, j: 1 });
    }

    #[test]
    fn test_parse_projm_product() {
        // FFV2: Gamma(3,2,-1)*ProjM(-1,1)
        let expr = lorentz_structure::structure("Gamma(3,2,-1)*ProjM(-1,1)").unwrap();
        assert_eq!(expr.len(), 1);
        assert_eq!(expr[0].ops.len(), 2);
        assert_eq!(expr[0].ops[0], LorentzOp::Gamma { mu: 3, i: 2, j: -1 });
        assert_eq!(expr[0].ops[1], LorentzOp::ProjM { i: -1, j: 1 });
    }

    #[test]
    fn test_parse_sum() {
        // ProjM(2,1) - ProjP(2,1)
        let expr = lorentz_structure::structure("ProjM(2,1) - ProjP(2,1)").unwrap();
        assert_eq!(expr.len(), 2);
        assert_eq!(expr[0].coeff, 1.0);
        assert_eq!(expr[1].coeff, -1.0);
    }

    #[test]
    fn test_parse_momentum() {
        // UUV1: P(3,2) + P(3,3)
        let expr = lorentz_structure::structure("P(3,2) + P(3,3)").unwrap();
        assert_eq!(expr.len(), 2);
        assert_eq!(expr[0].ops[0], LorentzOp::P { mu: 3, leg: 2 });
        assert_eq!(expr[1].ops[0], LorentzOp::P { mu: 3, leg: 3 });
    }

    #[test]
    fn test_parse_coefficient() {
        // e.g. "2*Gamma(3,2,1)"
        let expr = lorentz_structure::structure("2*Gamma(3,2,1)").unwrap();
        assert_eq!(expr[0].coeff, 2.0);
    }

    #[test]
    fn test_parse_epsilon() {
        let expr = lorentz_structure::structure("Epsilon(1,2,3,4)").unwrap();
        assert_eq!(
            expr[0].ops[0],
            LorentzOp::Epsilon {
                mu: 1,
                nu: 2,
                rho: 3,
                sigma: 4
            }
        );
    }

    const LORENTZ_SAMPLE: &str = r#"
from object_library import all_lorentz, Lorentz

FFV1 = Lorentz(name = 'FFV1',
               spins = [ 2, 2, 3 ],
               structure = 'Gamma(3,2,1)')

UUV1 = Lorentz(name = 'UUV1',
               spins = [ -1, -1, 3 ],
               structure = 'P(3,2) + P(3,3)')
"#;

    #[test]
    fn test_parse_lorentz_file() {
        let ls = parse_lorentz(LORENTZ_SAMPLE).unwrap();
        assert_eq!(ls.len(), 2);

        let ffv1 = ls.iter().find(|l| l.name == "FFV1").unwrap();
        assert_eq!(ffv1.spins, vec![2, 2, 3]);
        assert_eq!(ffv1.expr.len(), 1);
        assert_eq!(ffv1.expr[0].ops[0], LorentzOp::Gamma { mu: 3, i: 2, j: 1 });
    }
}
