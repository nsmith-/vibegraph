use super::ast_util::{call_func_name, get_kwarg, kwarg_str, parse_stmts};
use rustpython_parser::ast;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LorentzError {
    #[error("Failed to parse lorentz.py: {0}")]
    Parse(String),
    #[error("Failed to parse Lorentz structure '{name}': {cause}")]
    StructureParse { name: String, cause: String },
    #[error("Unknown Lorentz operator '{0}'")]
    UnknownOperator(String),
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

        let raw =
            lorentz_structure::structure(&structure).map_err(|e| LorentzError::StructureParse {
                name: name.clone(),
                cause: e.to_string(),
            })?;

        let expr = convert_expr(raw).map_err(|e| match e {
            LorentzError::UnknownOperator(_) => e,
            _ => LorentzError::StructureParse {
                name: name.clone(),
                cause: e.to_string(),
            },
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

// ── Intermediate (raw) types used by the PEG grammar ─────────────────────────

/// A syntactically parsed operator call before name dispatch.
#[derive(Debug, Clone)]
struct RawOp(String, Vec<i32>);

/// Raw version of LorentzTerm: coeff may have been folded from numeric atoms,
/// ops are still RawOps to be resolved in the conversion pass.
#[derive(Debug, Clone)]
struct RawTerm {
    coeff: f64,
    ops: Vec<RawOp>,
}

type RawExpr = Vec<RawTerm>;

// ── Conversion pass: RawExpr → LorentzExpr ───────────────────────────────────

fn convert_expr(raw: RawExpr) -> Result<LorentzExpr, LorentzError> {
    raw.into_iter()
        .map(|t| {
            let ops: Result<Vec<LorentzOp>, _> =
                t.ops.into_iter().map(|ro| build_lorentz_op(&ro)).collect();
            Ok(LorentzTerm {
                coeff: t.coeff,
                ops: ops?,
            })
        })
        .collect()
}

fn build_lorentz_op(raw: &RawOp) -> Result<LorentzOp, LorentzError> {
    let RawOp(name, args) = raw;
    match (name.as_str(), args.as_slice()) {
        ("Gamma", &[mu, i, j]) => Ok(LorentzOp::Gamma { mu, i, j }),
        ("Sigma", &[mu, nu, i, j]) => Ok(LorentzOp::Sigma { mu, nu, i, j }),
        ("Identity", &[i, j]) => Ok(LorentzOp::Identity { i, j }),
        ("ProjM", &[i, j]) => Ok(LorentzOp::ProjM { i, j }),
        ("ProjP", &[i, j]) => Ok(LorentzOp::ProjP { i, j }),
        ("Metric", &[mu, nu]) => Ok(LorentzOp::Metric { mu, nu }),
        ("P", &[mu, leg]) => Ok(LorentzOp::P { mu, leg }),
        ("Epsilon", &[mu, nu, rho, sigma]) => Ok(LorentzOp::Epsilon { mu, nu, rho, sigma }),
        ("C", &[i, j]) => Ok(LorentzOp::C { i, j }),
        _ => Err(LorentzError::UnknownOperator(name.clone())),
    }
}

// ── Helpers for the grammar's product rule ───────────────────────────────────

fn atom_to_terms(a: Atom) -> RawExpr {
    match a {
        Atom::Num(n) => vec![RawTerm {
            coeff: n,
            ops: vec![],
        }],
        Atom::Op(ro) => vec![RawTerm {
            coeff: 1.0,
            ops: vec![ro],
        }],
        Atom::Group(terms) => terms,
    }
}

/// Multiply all terms in `lhs` by one `rhs` atom, returning the expanded list.
fn mul_terms(lhs: RawExpr, rhs: Atom) -> RawExpr {
    match rhs {
        Atom::Num(n) => lhs
            .into_iter()
            .map(|mut t| {
                t.coeff *= n;
                t
            })
            .collect(),
        Atom::Op(ro) => lhs
            .into_iter()
            .map(move |mut t| {
                t.ops.push(ro.clone());
                t
            })
            .collect(),
        Atom::Group(rhs_terms) => lhs
            .into_iter()
            .flat_map(|lt| {
                rhs_terms.iter().map(move |rt| RawTerm {
                    coeff: lt.coeff * rt.coeff,
                    ops: lt.ops.iter().chain(rt.ops.iter()).cloned().collect(),
                })
            })
            .collect(),
    }
}

/// Divide all terms in `lhs` by one `rhs` atom (only numbers make physical sense).
fn div_terms(lhs: RawExpr, rhs: Atom) -> RawExpr {
    match rhs {
        Atom::Num(n) => lhs
            .into_iter()
            .map(|mut t| {
                t.coeff /= n;
                t
            })
            .collect(),
        _ => lhs, // division by operator/group is not a valid UFO construct
    }
}

// ── PEG grammar ───────────────────────────────────────────────────────────────

peg::parser! {
    grammar lorentz_structure() for str {
        /// Top-level: a sum of signed products.
        pub rule structure() -> RawExpr
            = _ first:signed_product() rest:( _ t:addend() { t } )* _ {
                let mut out = first;
                for t in rest { out.extend(t); }
                out
            }

        /// An addend: `+` or `-` followed by a product (sign is mandatory).
        rule addend() -> RawExpr
            = s:sign() _ p:product() {
                p.into_iter().map(|mut t| { t.coeff *= s; t }).collect()
            }

        /// The first term in a structure (sign optional).
        rule signed_product() -> RawExpr
            = s:sign() _ p:product() {
                p.into_iter().map(|mut t| { t.coeff *= s; t }).collect()
            }
            / p:product() { p }

        /// A product of atoms joined by `*` or `/`.
        /// Returns `Vec<RawTerm>` because a parenthesized atom may expand into
        /// multiple terms (e.g. `2*(A + B)` → `[2A, 2B]`).
        rule product() -> RawExpr
            = head:atom() tail:( _ op:['*' | '/'] _ a:atom() { (op, a) } )* {
                let mut terms = atom_to_terms(head);
                for (op, a) in tail {
                    terms = match op {
                        '*' => mul_terms(terms, a),
                        '/' => div_terms(terms, a),
                        _   => terms,
                    };
                }
                terms
            }

        /// A single atom: number, operator call, or parenthesised sub-expression.
        rule atom() -> Atom
            = n:number()    { Atom::Num(n) }
            / op:operator() { Atom::Op(op) }
            / "(" _ e:structure() _ ")" { Atom::Group(e) }

        /// Capture any `Identifier(int, ...)` call by name; dispatch happens in Rust.
        rule operator() -> RawOp
            = name:$(['A'..='Z' | 'a'..='z']['A'..='Z' | 'a'..='z' | '0'..='9' | '_']*)
              "(" _ args:(idx() ** (_ "," _)) _ ")" {
                RawOp(name.to_owned(), args)
            }

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

enum Atom {
    Num(f64),
    Op(RawOp),
    Group(RawExpr),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_identity() {
        let expr = parse_structure("1").unwrap();
        assert_eq!(expr.len(), 1);
        assert_eq!(expr[0].coeff, 1.0);
        assert!(expr[0].ops.is_empty());
    }

    #[test]
    fn test_parse_gamma() {
        let expr = parse_structure("Gamma(3,2,1)").unwrap();
        assert_eq!(expr.len(), 1);
        assert_eq!(expr[0].ops[0], LorentzOp::Gamma { mu: 3, i: 2, j: 1 });
    }

    #[test]
    fn test_parse_projm_product() {
        // FFV2: Gamma(3,2,-1)*ProjM(-1,1)
        let expr = parse_structure("Gamma(3,2,-1)*ProjM(-1,1)").unwrap();
        assert_eq!(expr.len(), 1);
        assert_eq!(expr[0].ops.len(), 2);
        assert_eq!(expr[0].ops[0], LorentzOp::Gamma { mu: 3, i: 2, j: -1 });
        assert_eq!(expr[0].ops[1], LorentzOp::ProjM { i: -1, j: 1 });
    }

    #[test]
    fn test_parse_sum() {
        // ProjM(2,1) - ProjP(2,1)
        let expr = parse_structure("ProjM(2,1) - ProjP(2,1)").unwrap();
        assert_eq!(expr.len(), 2);
        assert_eq!(expr[0].coeff, 1.0);
        assert_eq!(expr[1].coeff, -1.0);
    }

    #[test]
    fn test_parse_momentum() {
        // UUV1: P(3,2) + P(3,3)
        let expr = parse_structure("P(3,2) + P(3,3)").unwrap();
        assert_eq!(expr.len(), 2);
        assert_eq!(expr[0].ops[0], LorentzOp::P { mu: 3, leg: 2 });
        assert_eq!(expr[1].ops[0], LorentzOp::P { mu: 3, leg: 3 });
    }

    #[test]
    fn test_parse_coefficient() {
        // e.g. "2*Gamma(3,2,1)"
        let expr = parse_structure("2*Gamma(3,2,1)").unwrap();
        assert_eq!(expr[0].coeff, 2.0);
    }

    #[test]
    fn test_parse_grouped_div() {
        // VVVV5: Metric(1,4)*Metric(2,3) - (Metric(1,3)*Metric(2,4))/2. - (Metric(1,2)*Metric(3,4))/2.
        let s =
            "Metric(1,4)*Metric(2,3) - (Metric(1,3)*Metric(2,4))/2. - (Metric(1,2)*Metric(3,4))/2.";
        let expr = parse_structure(s).unwrap();
        assert_eq!(expr.len(), 3);
        assert!((expr[0].coeff - 1.0).abs() < 1e-10);
        assert!((expr[1].coeff + 0.5).abs() < 1e-10);
        assert!((expr[2].coeff + 0.5).abs() < 1e-10);
        assert_eq!(expr[0].ops[0], LorentzOp::Metric { mu: 1, nu: 4 });
        assert_eq!(expr[0].ops[1], LorentzOp::Metric { mu: 2, nu: 3 });
        assert_eq!(expr[1].ops[0], LorentzOp::Metric { mu: 1, nu: 3 });
        assert_eq!(expr[1].ops[1], LorentzOp::Metric { mu: 2, nu: 4 });
        assert_eq!(expr[2].ops[0], LorentzOp::Metric { mu: 1, nu: 2 });
        assert_eq!(expr[2].ops[1], LorentzOp::Metric { mu: 3, nu: 4 });
    }

    #[test]
    fn test_parse_epsilon() {
        let expr = parse_structure("Epsilon(1,2,3,4)").unwrap();
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

    #[test]
    fn test_ungrouped_division() {
        // A*B/2. without wrapping parens — the fragile old grammar couldn't handle this.
        let expr = parse_structure("Metric(1,2)*Metric(3,4)/2.").unwrap();
        assert_eq!(expr.len(), 1);
        assert!((expr[0].coeff - 0.5).abs() < 1e-10);
        assert_eq!(expr[0].ops[0], LorentzOp::Metric { mu: 1, nu: 2 });
        assert_eq!(expr[0].ops[1], LorentzOp::Metric { mu: 3, nu: 4 });
    }

    #[test]
    fn test_unknown_operator_error() {
        let result = parse_structure("FFCT2(1,2,3)");
        assert!(
            matches!(result, Err(LorentzError::UnknownOperator(ref s)) if s == "FFCT2"),
            "expected UnknownOperator(FFCT2), got {result:?}"
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

    /// Helper: parse a structure string directly to LorentzExpr (skips the .py wrapper).
    fn parse_structure(s: &str) -> Result<LorentzExpr, LorentzError> {
        let raw = lorentz_structure::structure(s).map_err(|e| LorentzError::StructureParse {
            name: "test".into(),
            cause: e.to_string(),
        })?;
        convert_expr(raw)
    }
}
