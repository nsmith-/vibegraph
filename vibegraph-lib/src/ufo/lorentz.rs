use indexmap::IndexMap;
use std::{collections::HashSet, ops::Index};

use super::ast_util::{call_func_name, get_kwarg, kwarg_str, parse_stmts};
use rustpython_parser::ast;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LorentzError {
    #[error("Failed to parse lorentz.py: {0}")]
    Parse(String),
    #[error("Failed to parse Lorentz structure '{name}': {cause}")]
    StructureParse { name: String, cause: String },
    #[error("Unknown Lorentz operator '{0}'")]
    UnknownOperator(String),
    #[error("Lorentz operator '{name}': {cause}")]
    OperatorArguments { name: String, cause: String },
    #[error("Spin map error in structure '{structure}': {cause}")]
    SpinMap { structure: String, cause: String },
    #[error("Invalid Lorentz index '{0}'")]
    InvalidIndex(i32),
}

/// A single Lorentz tensor operator.
///
/// The indices i,j are leg numbers from the UFO definition, with negative values for internal contractions.
/// We convert from 1-indexed convention to 0-indexed isize at import
/// i and j are spinor indices for fermion legs, or dummy indices for internal contractions. mu, nu, etc. are Lorentz indices.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LorentzOp {
    /// Dirac gamma matrix: Γ^μ_{ij}
    Gamma { mu: isize, i: isize, j: isize },
    /// Sigma: ALOHA's `σ^{μν}`, which is **half** the textbook
    /// `(i/2)[γ^μ, γ^ν]` — `aloha/aloha_object.py`'s `L_Sigma.sigma` table carries
    /// ±½ and ±½i where the textbook matrix carries ±1 and ±1i. The evaluator's
    /// kernels carry that half (`helas::eval::kernel`), and the
    /// `ll_to_qqx_toy_tensor` row measures it.
    Sigma {
        mu: isize,
        nu: isize,
        i: isize,
        j: isize,
    },
    /// Identity in spinor space: δ_{ij}
    Identity { i: isize, j: isize },
    /// Left projector: P_L = (1 - γ^5)/2
    ProjM { i: isize, j: isize },
    /// Right projector: P_R = (1 + γ^5)/2
    ProjP { i: isize, j: isize },
    /// Metric tensor: g^{μν}
    Metric { mu: isize, nu: isize },
    /// Momentum insertion: p_leg^μ
    P { mu: isize, leg: isize },
    /// Levi-Civita tensor: ε^{μνρσ}
    Epsilon {
        mu: isize,
        nu: isize,
        rho: isize,
        sigma: isize,
    },
    /// Charge-conjugation matrix: C_{ij}
    C { i: isize, j: isize },
    /// Chirality matrix: (γ^5)_{ij} = ProjP_{ij} − ProjM_{ij}
    Gamma5 { i: isize, j: isize },
}

impl LorentzOp {
    // TODO: involves_scalar (is this possible? momentum insertion?)

    /// Returns true if this operator involves a spinor index contraction with the given leg index.
    pub fn involves_spinor(&self, idx: isize) -> bool {
        match self {
            LorentzOp::Gamma { i, j, .. }
            | LorentzOp::Sigma { i, j, .. }
            | LorentzOp::Identity { i, j }
            | LorentzOp::ProjM { i, j }
            | LorentzOp::ProjP { i, j }
            | LorentzOp::Gamma5 { i, j }
            | LorentzOp::C { i, j } => *i == idx || *j == idx,
            _ => false,
        }
    }

    /// Returns true if this operator involves a Lorentz index contraction with the given leg index.
    pub fn involves_vector(&self, idx: isize) -> bool {
        match self {
            LorentzOp::Gamma { mu, .. } => *mu == idx,
            LorentzOp::P { mu, .. } => *mu == idx,
            LorentzOp::Sigma { mu, nu, .. } => *mu == idx || *nu == idx,
            LorentzOp::Metric { mu, nu } => *mu == idx || *nu == idx,
            LorentzOp::Epsilon { mu, nu, rho, sigma } => {
                *mu == idx || *nu == idx || *rho == idx || *sigma == idx
            }
            _ => false,
        }
    }
}

/// A term in a Lorentz structure: `coeff * op1 * op2 * ...`
///
/// The term itself is a fully connected graph of Lorentz operators
/// The LorentzOp indices indicate connections between operators when negative,
/// and (0-indexed) external leg indices when positive. An implicit product
/// over all connected operators is assumed, with the given coefficient.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LorentzTerm {
    pub coeff: f64,
    pub ops: Vec<LorentzOp>,
}

/// A Lorentz structure expression: sum of `LorentzTerm`s.
pub type LorentzExpr = Vec<LorentzTerm>;

/// Strongly-typed index for [`LorentzStructure`] lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LorentzId(usize);

impl From<usize> for LorentzId {
    fn from(value: usize) -> Self {
        LorentzId(value)
    }
}

impl Index<LorentzId> for IndexMap<String, LorentzStructure> {
    type Output = LorentzStructure;

    fn index(&self, index: LorentzId) -> &Self::Output {
        self.index(index.0)
    }
}

/// A Lorentz tensor structure from `lorentz.py`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LorentzStructure {
    /// Python variable name, e.g. `"FFV1"`.
    pub python_name: String,
    /// UFO `name` field.
    pub name: String,
    /// External leg spins (2s+1 per leg).
    pub spins: Vec<i32>,
    /// Verbatim `structure` string from the UFO file.
    pub structure: String,
    /// Parsed symbolic expression (0-indexed legs, negative for internal contractions).
    pub expr: LorentzExpr,
    /// Spinor index mapping for feyngraph: `spin_map[i]` is the external leg
    /// The i-th entry of the spin_map must be the leg j to which leg i is spin-connected
    /// (so i=j for any leg with no spinor contractions)
    /// Built from the Lorentz expression by tracing spinor index chains.
    pub spin_map: Vec<isize>,
}

fn find_connections(expr: &LorentzExpr, idx: isize) -> HashSet<isize> {
    let mut out = HashSet::new();
    for term in expr {
        for op in &term.ops {
            match op {
                // Spinor operators with index pairs (i, j)
                LorentzOp::Gamma { i, j, .. }
                | LorentzOp::Sigma { i, j, .. }
                | LorentzOp::Identity { i, j }
                | LorentzOp::ProjM { i, j }
                | LorentzOp::ProjP { i, j }
                | LorentzOp::Gamma5 { i, j }
                | LorentzOp::C { i, j } => {
                    if *i == idx {
                        out.insert(*j);
                    } else if *j == idx {
                        out.insert(*i);
                    }
                }
                _ => {}
            };
        }
    }
    out
}

/// Compute the spinor index mapping from a Lorentz expression.
///
/// Traces spinor index chains through the Lorentz operators to find which external legs
/// are spinor-contracted together. Nonnegative indices (0..n_legs) are external leg indices;
/// negative indices are internal contraction dummies. For each external leg, we find its
/// contracted partner by following the chain of contractions.
///
/// Returns a vector of length `n_legs` where `spin_map[i]` (0-indexed) is the 0-indexed
/// external leg that leg `i` contracts with.
pub fn compute_spin_map(expr: &LorentzExpr, n_legs: usize) -> Result<Vec<isize>, String> {
    // Trace from each external leg to find its partner.
    // External legs are indices 0..n_legs
    // Follow the chain of dummies (negative indices) to the other external endpoint.
    let mut spin_map = vec![None; n_legs];

    for leg in 0..n_legs as isize {
        if spin_map[leg as usize].is_some() {
            continue; // already paired
        }

        // Trace from this leg through dummy indices to find the other endpoint
        let mut current = leg;
        let mut visited = vec![];

        loop {
            if visited.contains(&current) {
                return Err(format!("cycle detected at index {current}"));
            }
            visited.push(current);

            // Advance current by following connections
            let Some(next) = find_connections(expr, current)
                .into_iter()
                .find(|i| !visited.contains(i))
            else {
                // Leg is not connected to any other leg through dummy indices
                spin_map[leg as usize] = Some(leg);
                break;
            };
            current = next;

            // If we reach another external leg, record the pairing
            if current >= 0 {
                spin_map[leg as usize] = Some(current);
                spin_map[current as usize] = Some(leg);
                break;
            }
        }
    }

    spin_map
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .ok_or("Not all spins were connected!".to_string())
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
        let spins = extract_spins(keywords)?;
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

        let n_legs = spins.len();
        let spin_map = compute_spin_map(&expr, n_legs).map_err(|cause| LorentzError::SpinMap {
            structure: structure.clone(),
            cause,
        })?;

        result.push(LorentzStructure {
            python_name,
            name,
            spins,
            structure,
            expr,
            spin_map,
        });
    }

    Ok(result)
}

/// Parse the `spins` keyword argument from a Lorentz structure definition
fn extract_spins(keywords: &[ast::Keyword]) -> Result<Vec<i32>, LorentzError> {
    use super::ast_util::extract_int;
    let mut spins = Vec::new();
    let Some(val) = get_kwarg(keywords, "spins") else {
        return Ok(spins);
    };
    let ast::Expr::List(ast::ExprList { elts, .. }) = val else {
        return Ok(spins);
    };
    for elt in elts {
        let s = extract_int(elt).ok_or_else(|| LorentzError::StructureParse {
            name: "spins".into(),
            cause: format!("expected integer spin, got {elt:?}"),
        })? as i32;
        spins.push(s);
    }
    Ok(spins)
}

// ── Intermediate (raw) types used by the PEG grammar ─────────────────────────

/// One argument of a syntactically parsed operator call.
///
/// A UFO Lorentz operator takes plain indices, but the grammar accepts an
/// arbitrary sub-expression so that a model-specific operator built on top of
/// them (`FFCT2((P(-3,3)+P(-3,4))*…)`) still parses far enough to be reported by
/// name rather than as a syntax error.
#[derive(Debug, Clone)]
enum RawArg {
    Index(i32),
    Nested,
}

/// A syntactically parsed operator call before name dispatch.
#[derive(Debug, Clone)]
struct RawOp(String, Vec<RawArg>);

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

fn to_isize(i: &i32) -> Result<isize, LorentzError> {
    let out: isize = (*i)
        .try_into()
        .map_err(|_| LorentzError::InvalidIndex(*i))?;
    if out == 0 {
        Err(LorentzError::InvalidIndex(*i))
    } else if out > 0 {
        // Convert to 0-indexed
        Ok(out - 1)
    } else {
        Ok(out)
    }
}

/// The operator names this loader understands. A call to anything else is an
/// [`LorentzError::UnknownOperator`], whatever shape its arguments have.
const KNOWN_OPERATORS: [&str; 10] = [
    "Gamma", "Gamma5", "Sigma", "Identity", "ProjM", "ProjP", "Metric", "P", "Epsilon", "C",
];

fn build_lorentz_op(raw: &RawOp) -> Result<LorentzOp, LorentzError> {
    let RawOp(name, args) = raw;
    if !KNOWN_OPERATORS.contains(&name.as_str()) {
        return Err(LorentzError::UnknownOperator(name.clone()));
    }
    let indices = args
        .iter()
        .map(|a| match a {
            RawArg::Index(i) => Ok(*i),
            RawArg::Nested => Err(LorentzError::OperatorArguments {
                name: name.clone(),
                cause: "takes plain indices, not sub-expressions".to_owned(),
            }),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let iargs = indices
        .iter()
        .map(to_isize)
        .collect::<Result<Vec<_>, _>>()?;
    match (name.as_str(), iargs.as_slice()) {
        ("Gamma", &[mu, i, j]) => Ok(LorentzOp::Gamma { mu, i, j }),
        ("Gamma5", &[i, j]) => Ok(LorentzOp::Gamma5 { i, j }),
        ("Sigma", &[mu, nu, i, j]) => Ok(LorentzOp::Sigma { mu, nu, i, j }),
        ("Identity", &[i, j]) => Ok(LorentzOp::Identity { i, j }),
        ("ProjM", &[i, j]) => Ok(LorentzOp::ProjM { i, j }),
        ("ProjP", &[i, j]) => Ok(LorentzOp::ProjP { i, j }),
        ("Metric", &[mu, nu]) => Ok(LorentzOp::Metric { mu, nu }),
        ("P", &[mu, leg]) => Ok(LorentzOp::P { mu, leg }),
        ("Epsilon", &[mu, nu, rho, sigma]) => Ok(LorentzOp::Epsilon { mu, nu, rho, sigma }),
        ("C", &[i, j]) => Ok(LorentzOp::C { i, j }),
        _ => Err(LorentzError::OperatorArguments {
            name: name.clone(),
            cause: format!("does not take {} index argument(s)", iargs.len()),
        }),
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

        /// A product of factors joined by `*` or `/`.
        /// Returns `Vec<RawTerm>` because a parenthesized factor may expand into
        /// multiple terms (e.g. `2*(A + B)` → `[2A, 2B]`).
        rule product() -> RawExpr
            = head:factor() tail:( _ op:['*' | '/'] _ a:factor() { (op, a) } )* {
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

        /// An atom, optionally raised to a non-negative integer power.
        rule factor() -> Atom
            = a:atom() e:( _ "**" _ n:exponent() { n } )? {?
                match e {
                    None => Ok(a),
                    Some(k) => pow_atom(a, k),
                }
            }

        /// A single atom: number, operator call, or parenthesised sub-expression.
        rule atom() -> Atom
            = n:number()    { Atom::Num(n) }
            / op:operator() { Atom::Op(op) }
            / "(" _ e:structure() _ ")" { Atom::Group(e) }

        /// Capture any `Identifier(arg, ...)` call by name; dispatch happens in Rust.
        /// An argument that is not a plain index is kept only as `Nested`, which is
        /// enough for an unknown operator to be reported by name.
        rule operator() -> RawOp
            = name:$(['A'..='Z' | 'a'..='z']['A'..='Z' | 'a'..='z' | '0'..='9' | '_']*)
              "(" _ args:(oparg() ** (_ "," _)) _ ")" {
                RawOp(name.to_owned(), args)
            }

        rule oparg() -> RawArg
            = e:structure() { classify_arg(&e) }

        rule number() -> f64
            = n:$(['0'..='9']+ ("." ['0'..='9']*)?) {? n.parse().or(Err("number")) }

        rule exponent() -> u32
            = n:$(['0'..='9']+) {? n.parse().or(Err("exponent")) }

        rule sign() -> f64
            = "+" { 1.0 }
            / "-" { -1.0 }

        rule _ = [' ' | '\t' | '\n' | '\r']*
    }
}

#[derive(Clone)]
enum Atom {
    Num(f64),
    Op(RawOp),
    Group(RawExpr),
}

/// Classify one operator argument: a lone signed integer literal is an index,
/// anything else is an opaque sub-expression.
fn classify_arg(e: &RawExpr) -> RawArg {
    match e.as_slice() {
        [term] if term.ops.is_empty() && term.coeff.fract() == 0.0 => {
            RawArg::Index(term.coeff as i32)
        }
        _ => RawArg::Nested,
    }
}

/// `X**n`: `n` copies of `X` multiplied together, so Einstein summation over a
/// repeated dummy index does the contraction (`P(-1,2)**2` = `p₂·p₂`).
///
/// A tensor index may appear at most twice in a term, so a power above 2 on an
/// operator has no Einstein reading and is rejected rather than guessed at.
fn pow_atom(a: Atom, k: u32) -> Result<Atom, &'static str> {
    let carries_ops = match &a {
        Atom::Num(_) => false,
        Atom::Op(_) => true,
        Atom::Group(terms) => terms.iter().any(|t| !t.ops.is_empty()),
    };
    if carries_ops && k > 2 {
        return Err("power above 2 on an indexed Lorentz object");
    }
    match k {
        0 => Ok(Atom::Num(1.0)),
        1 => Ok(a),
        _ => {
            let mut terms = atom_to_terms(a.clone());
            for _ in 1..k {
                terms = mul_terms(terms, a.clone());
            }
            Ok(Atom::Group(terms))
        }
    }
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
        assert_eq!(expr[0].ops[0], LorentzOp::Gamma { mu: 2, i: 1, j: 0 });
    }

    #[test]
    fn test_parse_projm_product() {
        // FFV2: Gamma(3,2,-1)*ProjM(-1,1)
        let expr = parse_structure("Gamma(3,2,-1)*ProjM(-1,1)").unwrap();
        assert_eq!(expr.len(), 1);
        assert_eq!(expr[0].ops.len(), 2);
        assert_eq!(expr[0].ops[0], LorentzOp::Gamma { mu: 2, i: 1, j: -1 });
        assert_eq!(expr[0].ops[1], LorentzOp::ProjM { i: -1, j: 0 });
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
        assert_eq!(expr[0].ops[0], LorentzOp::P { mu: 2, leg: 1 });
        assert_eq!(expr[1].ops[0], LorentzOp::P { mu: 2, leg: 2 });
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
        assert_eq!(expr[0].ops[0], LorentzOp::Metric { mu: 0, nu: 3 });
        assert_eq!(expr[0].ops[1], LorentzOp::Metric { mu: 1, nu: 2 });
        assert_eq!(expr[1].ops[0], LorentzOp::Metric { mu: 0, nu: 2 });
        assert_eq!(expr[1].ops[1], LorentzOp::Metric { mu: 1, nu: 3 });
        assert_eq!(expr[2].ops[0], LorentzOp::Metric { mu: 0, nu: 1 });
        assert_eq!(expr[2].ops[1], LorentzOp::Metric { mu: 2, nu: 3 });
    }

    #[test]
    fn test_parse_epsilon() {
        let expr = parse_structure("Epsilon(1,2,3,4)").unwrap();
        assert_eq!(
            expr[0].ops[0],
            LorentzOp::Epsilon {
                mu: 0,
                nu: 1,
                rho: 2,
                sigma: 3
            }
        );
    }

    #[test]
    fn test_ungrouped_division() {
        // A*B/2. without wrapping parens — the fragile old grammar couldn't handle this.
        let expr = parse_structure("Metric(1,2)*Metric(3,4)/2.").unwrap();
        assert_eq!(expr.len(), 1);
        assert!((expr[0].coeff - 0.5).abs() < 1e-10);
        assert_eq!(expr[0].ops[0], LorentzOp::Metric { mu: 0, nu: 1 });
        assert_eq!(expr[0].ops[1], LorentzOp::Metric { mu: 2, nu: 3 });
    }

    #[test]
    fn test_unknown_operator_error() {
        let result = parse_structure("FFCT2(1,2,3)");
        assert!(
            matches!(result, Err(LorentzError::UnknownOperator(ref s)) if s == "FFCT2"),
            "expected UnknownOperator(FFCT2), got {result:?}"
        );
    }

    #[test]
    fn test_unknown_operator_taudecay_ufo() {
        // The UFO for tau decays contains a non-standard operator FFCT2 that we don't support.
        let result = parse_structure("FFCT2((P(-3,3)+P(-3,4))*(P(-3,3)+P(-3,4))) *(P(-1,3)*Gamma(-1,2,-2)*ProjM(-2,1) - P(-1,4)*Gamma(-1,2,-2)*ProjM(-2,1))");
        assert!(
            matches!(result, Err(LorentzError::UnknownOperator(ref s)) if s == "FFCT2"),
            "expected UnknownOperator(FFCT2), got {result:?}"
        );
    }

    #[test]
    fn test_parse_gamma5() {
        let expr = parse_structure("Gamma5(2,1)").unwrap();
        assert_eq!(expr[0].ops[0], LorentzOp::Gamma5 { i: 1, j: 0 });

        // A γ5 inside a chain keeps its dummy indices, so the chain still traces.
        let expr = parse_structure("Gamma5(-2,1)*Gamma(3,2,-2)").unwrap();
        assert_eq!(expr[0].ops[0], LorentzOp::Gamma5 { i: -2, j: 0 });
        let spin_map = compute_spin_map(&expr, 3).unwrap();
        assert_eq!(spin_map, vec![1, 0, 2]);
    }

    #[test]
    fn test_parse_squared_momentum() {
        // `P(-1,1)**2` is the same term as `P(-1,1)*P(-1,1)`: the repeated dummy
        // index contracts the two copies into a scalar product.
        let squared = parse_structure("P(-1,1)**2").unwrap();
        let written_out = parse_structure("P(-1,1)*P(-1,1)").unwrap();
        assert_eq!(squared, written_out);
        assert_eq!(squared[0].ops.len(), 2);

        // Inside a product, with a coefficient, and with two independent dummies.
        let expr = parse_structure("P(-2,2)**2*P(-1,1)**2*Metric(1,2)/2.").unwrap();
        assert_eq!(expr.len(), 1);
        assert!((expr[0].coeff - 0.5).abs() < 1e-12);
        assert_eq!(expr[0].ops.len(), 5);
        assert_eq!(
            expr[0].ops,
            vec![
                LorentzOp::P { mu: -2, leg: 1 },
                LorentzOp::P { mu: -2, leg: 1 },
                LorentzOp::P { mu: -1, leg: 0 },
                LorentzOp::P { mu: -1, leg: 0 },
                LorentzOp::Metric { mu: 0, nu: 1 },
            ]
        );

        // Powers distribute over a sum.
        let expr = parse_structure("(P(-1,1) + P(-1,2))**2").unwrap();
        assert_eq!(expr.len(), 4);
    }

    #[test]
    fn test_numeric_power_folds_into_the_coefficient() {
        let expr = parse_structure("2**3*Metric(1,2)").unwrap();
        assert_eq!(expr.len(), 1);
        assert!((expr[0].coeff - 8.0).abs() < 1e-12);
        assert_eq!(expr[0].ops, vec![LorentzOp::Metric { mu: 0, nu: 1 }]);
    }

    #[test]
    fn test_cubed_operator_is_rejected() {
        // A tensor index may appear at most twice in a term, so there is no
        // Einstein reading of a cube — it must not silently become three copies.
        let result = parse_structure("P(-1,1)**3");
        assert!(
            matches!(result, Err(LorentzError::StructureParse { .. })),
            "expected a structure parse error, got {result:?}"
        );
    }

    #[test]
    fn test_known_operator_rejects_a_nested_argument() {
        let result = parse_structure("Gamma((P(-1,1)+P(-1,2)),2,1)");
        assert!(
            matches!(
                result,
                Err(LorentzError::OperatorArguments { ref name, .. }) if name == "Gamma"
            ),
            "expected OperatorArguments(Gamma), got {result:?}"
        );
    }

    #[test]
    fn test_known_operator_rejects_wrong_arity() {
        let result = parse_structure("Metric(1,2,3)");
        assert!(
            matches!(
                result,
                Err(LorentzError::OperatorArguments { ref name, .. }) if name == "Metric"
            ),
            "expected OperatorArguments(Metric), got {result:?}"
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
        assert_eq!(ffv1.expr[0].ops[0], LorentzOp::Gamma { mu: 2, i: 1, j: 0 });
    }

    #[test]
    fn test_compute_spin_map_ffv1() {
        // FFV1: Gamma(3,2,1) connects legs 1 and 2 (indices 2 and 1)
        let expr = vec![LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::Gamma { mu: 2, i: 1, j: 0 }],
        }];
        let spin_map = compute_spin_map(&expr, 3).expect("failed to compute spin map");
        // Leg 1 should connect to leg 2, and leg 2 should connect to leg 1
        assert_eq!(spin_map[0], 1); // leg 1 (index 0) connects to leg 2
        assert_eq!(spin_map[1], 0); // leg 2 (index 1) connects to leg 1
        assert_eq!(spin_map[2], 2); // leg 3 (index 2) has no spinor contraction
    }

    #[test]
    fn test_compute_spin_map_no_spinors() {
        // UUV1: P(3,2) + P(3,3) — momentum operators, no spinor contractions
        let expr = vec![
            LorentzTerm {
                coeff: 1.0,
                ops: vec![LorentzOp::P { mu: 3, leg: 2 }],
            },
            LorentzTerm {
                coeff: 1.0,
                ops: vec![LorentzOp::P { mu: 3, leg: 3 }],
            },
        ];
        let spin_map = compute_spin_map(&expr, 3).expect("failed to compute spin map");
        assert_eq!(spin_map, vec![0, 1, 2]);
    }

    #[test]
    fn test_compute_spin_map_projector_chain() {
        // Gamma(3,2,-1)*ProjM(-1,1): connects leg 2 to leg 1 via dummy -1
        let expr = vec![LorentzTerm {
            coeff: 1.0,
            ops: vec![
                LorentzOp::Gamma { mu: 2, i: 1, j: -1 },
                LorentzOp::ProjM { i: -1, j: 0 },
            ],
        }];
        let spin_map = compute_spin_map(&expr, 2).expect("failed to compute spin map");
        assert_eq!(spin_map[0], 1); // leg 1 connects to leg 2
        assert_eq!(spin_map[1], 0); // leg 2 connects to leg 1
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
