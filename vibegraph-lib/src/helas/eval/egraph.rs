//! Skeleton of the egglog rewrite stage: round-trip an [`Ast<Sym>`] through an
//! egglog e-graph and back, unchanged.
//!
//! The lowered, binary-arity [`Ast<Sym>`] (see [`super::lower`]) maps onto an egglog
//! `datatype`: one constructor per [`Op`], its constructor name the same head token
//! the s-expression I/O uses ([`Op::name`]). Leaf payloads become leading base-sort
//! fields (`Coupling`/`Mass`/`Width` → `i64`, `Coeff` → `f64`, `External` → the
//! `leg spin sign incoming` quadruple); arena children become `Node` arguments. Every
//! op has fixed arity except [`Op::PMomOut`] (a vertex's whole input list), which takes
//! a `(Vec Node)` and so is declared as a separate `constructor` after the vector sort.
//!
//! [`roundtrip`] declares the schema, then encodes the AST as a sequence of
//! [`Command`]s (a `let $nID …` action per arena node, children before parents so each
//! binding can reference earlier ones by id — this also preserves DAG sharing instead
//! of expanding it) followed by an `extract` of the root, and decodes the extracted
//! [`TermDag`] back into an `Ast<Sym>`. Commands are built directly rather than
//! rendered to text, so the encoding never round-trips through egglog's parser. With
//! no rewrite rules registered, extraction returns exactly the inserted term, so the result is
//! structurally identical to the input. This is the seam the future algebraic-rewrite
//! and congruence-CSE rules slot into (see `research/notes/14-egglog-notes.md`); the
//! rules will turn this identity pass into an optimizing one.

use std::collections::HashMap;

use egglog::ast::{Command, Expr, GenericAction, Literal, RustSpan, Span};
use egglog::{CommandOutput, EGraph, Term, TermDag, TermId};
use ordered_float::OrderedFloat;

use super::ast::{Ast, AstBuilder};
use super::op::{charge_from_sign, NodeId, Op, Sym};
use super::tree::Tree;
use crate::ufo::couplings::CouplingId;
use crate::ufo::particles::ParticleId;

/// The egglog schema for [`Ast<Sym>`]: the `Node` datatype (one constructor per
/// fixed-arity [`Op`], names matching [`Op::name`]), the `(Vec Node)` sort backing
/// [`Op::PMomOut`], and `PMomOut` itself as a variable-arity constructor over it.
const NODE_SCHEMA: &str = "\
(datatype Node
  (External i64 i64 i64 i64 Node)
  (Propagate Node Node Node)
  (Mul Node Node)
  (Add Node Node)
  (GammaVout Node Node)
  (GammaIout Node Node)
  (GammaOout Node Node)
  (ProjM Node)
  (ProjP Node)
  (ProjMAmp Node Node)
  (ProjPAmp Node Node)
  (Metric Node Node)
  (MetricNegI Node Node)
  (MetricVout Node)
  (LowerVout Node)
  (IdentityAmp Node Node)
  (FfvVout Node Node Node Node)
  (FfvIout Node Node Node Node)
  (FfvOout Node Node Node Node)
  (PMom Node)
  (Coupling i64)
  (Mass i64)
  (Width i64)
  (Coeff f64))
(sort NodeVec (Vec Node))
(constructor PMomOut (NodeVec) Node)
";

/// A failure encoding, running, or decoding the egglog round-trip.
#[derive(Debug, thiserror::Error)]
pub enum EgraphError {
    #[error("egglog: {0}")]
    Egglog(String),
    #[error("egglog produced no extracted term")]
    NoExtract,
    #[error("decoding extracted term: {0}")]
    Decode(String),
}

/// Round-trip `ast` through an egglog e-graph: encode it, extract the root, and
/// decode the result back into an `Ast<Sym>`. With no rewrite rules registered this
/// is structurally the identity; it is the skeleton the optimization rules attach to.
pub fn roundtrip(ast: &Ast<Sym>) -> Result<Ast<Sym>, EgraphError> {
    let mut egraph = EGraph::default();
    egraph
        .parse_and_run_program(None, NODE_SCHEMA)
        .map_err(|e| EgraphError::Egglog(e.to_string()))?;
    let outputs = egraph
        .run_program(encode_commands(ast))
        .map_err(|e| EgraphError::Egglog(e.to_string()))?;
    let (dag, root_term) = outputs
        .into_iter()
        .find_map(|o| match o {
            CommandOutput::ExtractBest(dag, _cost, term) => Some((dag, term)),
            _ => None,
        })
        .ok_or(EgraphError::NoExtract)?;

    let mut b = AstBuilder::new();
    let mut memo: HashMap<TermId, NodeId> = HashMap::new();
    let root = decode(&dag, root_term, &mut b, &mut memo)?;
    Ok(b.finish(root))
}

// ─────────────────────────────── encode ────────────────────────────────────

/// Build the encoding commands: one `let $nID …` action per arena node in topological
/// (children-first) order, then an `extract` of the root. Assumes the schema
/// ([`NODE_SCHEMA`]) has already been declared on the e-graph.
fn encode_commands(ast: &Ast<Sym>) -> Vec<Command> {
    let mut cmds = Vec::with_capacity(ast.len() + 1);
    for id in 0..ast.len() as NodeId {
        cmds.push(Command::Action(GenericAction::Let(
            span(),
            node_var(id),
            encode_expr(ast, id),
        )));
    }
    // Default variant count (0) — the same expr the s-expr parser fills in for `(extract e)`.
    cmds.push(Command::Extract(span(), var(node_var(ast.root())), int(0)));
    cmds
}

/// Encode one node as an egglog constructor call `(OpName leaf-fields… $child…)`,
/// referencing children by their already-bound `$nID` globals.
fn encode_expr(ast: &Ast<Sym>, id: NodeId) -> Expr {
    let node = ast.value(id);
    let kids = ast.children_ids(id);
    let mut args = Vec::with_capacity(kids.len() + 4);
    match (node.op, &node.leaf) {
        (Op::Coupling, Sym::Coupling(c)) => args.push(int(c.index() as i64)),
        (Op::Mass | Op::Width, Sym::Particle(p)) => args.push(int(p.index() as i64)),
        (Op::Coeff, Sym::Coeff(c)) => args.push(float(*c)),
        (
            Op::External,
            Sym::Ext {
                leg_idx,
                spin,
                charge,
                incoming,
            },
        ) => {
            args.push(int(*leg_idx as i64));
            args.push(int(*spin as i64));
            args.push(int(charge.sign() as i64));
            args.push(int(*incoming as i64));
        }
        _ => {}
    }
    if node.op == Op::PMomOut {
        // The one variable-arity op: children go inside a Vec argument.
        let elems: Vec<Expr> = kids.iter().map(|&k| var(node_var(k))).collect();
        args.push(if elems.is_empty() {
            call("vec-empty", vec![])
        } else {
            call("vec-of", elems)
        });
    } else {
        args.extend(kids.iter().map(|&k| var(node_var(k))));
    }
    call(node.op.name(), args)
}

/// The global-variable name bound to arena node `id` (`$nID`).
fn node_var(id: NodeId) -> String {
    format!("$n{id}")
}

// egglog `Expr`/`Command` constructors carry a source [`Span`] for error reporting;
// generated terms point back here rather than into any egglog source text.
fn span() -> Span {
    Span::Rust(std::sync::Arc::new(RustSpan {
        file: file!(),
        line: line!(),
        column: column!(),
    }))
}

fn call(head: &str, args: Vec<Expr>) -> Expr {
    Expr::Call(span(), head.to_string(), args)
}

fn var(name: String) -> Expr {
    Expr::Var(span(), name)
}

fn int(v: i64) -> Expr {
    Expr::Lit(span(), Literal::Int(v))
}

fn float(v: f64) -> Expr {
    Expr::Lit(span(), Literal::Float(OrderedFloat(v)))
}

// ─────────────────────────────── decode ────────────────────────────────────

/// Rebuild an `Ast<Sym>` node from extracted term `id`, recursing into children.
/// Memoized on [`TermId`] so the extracted DAG's sharing is preserved in the arena.
fn decode(
    dag: &TermDag,
    id: TermId,
    b: &mut AstBuilder<Sym>,
    memo: &mut HashMap<TermId, NodeId>,
) -> Result<NodeId, EgraphError> {
    if let Some(&n) = memo.get(&id) {
        return Ok(n);
    }
    let (name, kids) = match dag.get(id) {
        Term::App(name, kids) => (name.as_str(), kids.as_slice()),
        other => {
            return Err(decode_err(format_args!(
                "expected an App term, found {other:?}"
            )))
        }
    };
    let op = Op::from_name(name).ok_or_else(|| decode_err(format_args!("unknown op `{name}`")))?;

    // Split each op's term arguments into its leaf payload and its `Node` children.
    let (leaf, child_terms): (Sym, Vec<TermId>) = match op {
        Op::Coupling => (
            Sym::Coupling(CouplingId::from(int_arg(dag, kids, 0)? as usize)),
            vec![],
        ),
        Op::Mass | Op::Width => (
            Sym::Particle(ParticleId::from(int_arg(dag, kids, 0)? as usize)),
            vec![],
        ),
        Op::Coeff => (Sym::Coeff(float_arg(dag, kids, 0)?), vec![]),
        Op::External => {
            let leaf = Sym::Ext {
                leg_idx: int_arg(dag, kids, 0)? as usize,
                spin: int_arg(dag, kids, 1)? as i32,
                charge: charge_from_sign(int_arg(dag, kids, 2)? as i32),
                incoming: int_arg(dag, kids, 3)? != 0,
            };
            (leaf, kids.get(4..).unwrap_or_default().to_vec())
        }
        Op::PMomOut => {
            let vec_id = *kids
                .first()
                .ok_or_else(|| decode_err(format_args!("PMomOut without its Vec argument")))?;
            (Sym::None, vec_elements(dag, vec_id)?)
        }
        _ => (Sym::None, kids.to_vec()),
    };

    let children = child_terms
        .iter()
        .map(|&c| decode(dag, c, b, memo))
        .collect::<Result<Vec<_>, _>>()?;
    let nid = b.add(op, leaf, children);
    memo.insert(id, nid);
    Ok(nid)
}

/// The element terms of an extracted `(vec-of …)` / `(vec-empty)` vector term.
fn vec_elements(dag: &TermDag, id: TermId) -> Result<Vec<TermId>, EgraphError> {
    match dag.get(id) {
        Term::App(name, elems) if name == "vec-of" => Ok(elems.clone()),
        Term::App(name, _) if name == "vec-empty" => Ok(vec![]),
        other => Err(decode_err(format_args!(
            "expected a Vec term, found {other:?}"
        ))),
    }
}

fn int_arg(dag: &TermDag, kids: &[TermId], i: usize) -> Result<i64, EgraphError> {
    match kids.get(i).map(|&k| dag.get(k)) {
        Some(Term::Lit(Literal::Int(v))) => Ok(*v),
        other => Err(decode_err(format_args!(
            "expected i64 argument {i}, found {other:?}"
        ))),
    }
}

fn float_arg(dag: &TermDag, kids: &[TermId], i: usize) -> Result<f64, EgraphError> {
    match kids.get(i).map(|&k| dag.get(k)) {
        Some(Term::Lit(Literal::Float(v))) => Ok(v.0),
        other => Err(decode_err(format_args!(
            "expected f64 argument {i}, found {other:?}"
        ))),
    }
}

fn decode_err(args: std::fmt::Arguments) -> EgraphError {
    EgraphError::Decode(args.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Round-trip an s-expression through egglog and assert it comes back byte-for-byte.
    /// The existing s-expr `FromStr`/`Display` are the oracle: parse the string, run the
    /// egglog round-trip, and require the rendered result to match the input string.
    fn assert_roundtrip(sexpr: &str) {
        let ast: Ast<Sym> = sexpr.parse().expect("parse input s-expr");
        let out = roundtrip(&ast).expect("egglog round-trip");
        assert_eq!(
            out.to_string(),
            ast.to_string(),
            "round-trip changed the tree"
        );
    }

    #[test]
    fn leaves_all_flavors() {
        assert_roundtrip("(Coupling (CouplingId 5))");
        assert_roundtrip("(Mass (ParticleId 11))");
        assert_roundtrip("(Width (ParticleId 23))");
        assert_roundtrip("(Coeff (Real 1.5))");
    }

    #[test]
    fn coeff_whole_and_negative_and_zero() {
        // Whole-number and zero coeffs must survive the i64-vs-f64 literal distinction;
        // negatives (symmetry factor × Fermi sign) must keep their sign.
        assert_roundtrip("(Coeff (Real 1.0))");
        assert_roundtrip("(Coeff (Real 0.0))");
        assert_roundtrip("(Coeff (Real -1.0))");
        assert_roundtrip("(Coeff (Real -2.5))");
    }

    #[test]
    fn external_with_mass_child() {
        assert_roundtrip("(External (ExtLegInfo 0 2 1 1) (Mass (ParticleId 11)))");
        assert_roundtrip("(External (ExtLegInfo 3 2 -1 0) (Mass (ParticleId 23)))");
    }

    #[test]
    fn nested_vertex_shape() {
        assert_roundtrip(
            "(Mul (Coupling (CouplingId 5)) (Add \
               (Mul (Coeff (Real 1.5)) \
                    (GammaVout (External (ExtLegInfo 0 2 1 1) (Mass (ParticleId 11))) \
                               (External (ExtLegInfo 1 2 -1 0) (Mass (ParticleId 11))))) \
               (Mul (Coeff (Real -1.0)) \
                    (ProjM (External (ExtLegInfo 2 2 1 1) (Mass (ParticleId 0)))))))",
        );
    }

    #[test]
    fn fused_ffv_and_propagator() {
        assert_roundtrip(
            "(Propagate \
               (FfvVout (External (ExtLegInfo 0 2 1 1) (Mass (ParticleId 11))) \
                        (External (ExtLegInfo 1 2 -1 0) (Mass (ParticleId 11))) \
                        (Coupling (CouplingId 3)) (Coupling (CouplingId 4))) \
               (Mass (ParticleId 23)) (Width (ParticleId 23)))",
        );
    }

    #[test]
    fn pmom_and_pmomout_vector_arg() {
        assert_roundtrip("(PMom (External (ExtLegInfo 0 2 1 1) (Mass (ParticleId 0))))");
        // PMomOut is the one variable-arity op (Vec Node); exercise several inputs.
        assert_roundtrip(
            "(PMomOut (External (ExtLegInfo 0 2 1 1) (Mass (ParticleId 24))) \
                      (External (ExtLegInfo 1 2 -1 0) (Mass (ParticleId 24))) \
                      (External (ExtLegInfo 2 3 1 1) (Mass (ParticleId 23))))",
        );
    }

    /// A shared subtree (a DAG, not a tree) survives the round-trip: the s-expr renders
    /// shared nodes once per parent, so equal renderings confirm the shape is preserved.
    #[test]
    fn shared_subtree_dag() {
        let mut b = AstBuilder::new();
        let mass = b.add(Op::Mass, Sym::Particle(ParticleId::from(11usize)), vec![]);
        let ext = b.add(
            Op::External,
            Sym::Ext {
                leg_idx: 0,
                spin: 2,
                charge: crate::helas::repr::numbers::Charge::Particle,
                incoming: true,
            },
            vec![mass],
        );
        // `ext` feeds both Add operands: one arena node, two parents.
        let root = b.add(Op::Add, Sym::None, vec![ext, ext]);
        let ast = b.finish(root);
        let out = roundtrip(&ast).expect("egglog round-trip");
        assert_eq!(out.to_string(), ast.to_string());
        let kids = out.children_ids(out.root());
        assert_eq!(kids[0], kids[1], "shared child stays a single arena node");
    }

    /// The real payoff: representative MG-validated processes, compiled to their binary
    /// `Ast<Sym>`, survive the egglog round-trip byte-for-byte. This exercises genuine
    /// HELAS output — multi-diagram sharing, `PMomOut` from the triple-gauge `W+ W-`
    /// vertex, fused `Ffv*` kernels, massive externals — not just the hand-written
    /// fixtures above. Kept to 2→2/2→3 processes: the 2→6 QCD amplitudes lower to tens
    /// of thousands of nodes, and building + extracting an e-graph that large runs on
    /// the order of a minute per subprocess — too slow for a default test. That cost is
    /// egglog's own e-graph construction and extraction, not the encoding: measured on a
    /// 37k-node `g g > g g g g` subprocess it is the same whether the program is built as
    /// [`Command`]s or parsed from text. The eventual optimizer will confront it directly
    /// when it runs egglog on those ASTs.
    #[test]
    fn representative_processes_roundtrip() {
        use super::super::lower;
        use super::super::root_diagram::compile_diagram_ast;
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use crate::ufo::sm::{sm_model, SMRestrict};

        const PROCESSES: [&str; 4] = [
            "e+ e- > mu+ mu-",   // basic s-channel
            "e+ e- > mu+ mu- a", // photon radiation: several diagrams, chiral FFV fusion
            "e+ e- > W+ W-",     // triple-gauge VVV vertex: PMomOut
            "e+ e- > t t~",      // massive externals
        ];

        let model = sm_model(SMRestrict::Default);
        let opts = ParsingOptions::default();
        for process in PROCESSES {
            let pc = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
            let sets = generate_from_proc_card(&pc, &model).unwrap();
            assert!(!sets.is_empty(), "no diagrams for '{process}'");
            for set in &sets {
                let diagrams = compile_diagram_ast(set, &model).unwrap();
                let ast = lower::lower(&diagrams);
                let out = roundtrip(&ast)
                    .unwrap_or_else(|e| panic!("[{process}] egglog round-trip failed: {e}"));
                assert_eq!(
                    out.to_string(),
                    ast.to_string(),
                    "[{process}] round-trip changed the lowered AST"
                );
            }
        }
    }

    /// Every `Op` variant appears as a constructor in the schema, so adding an op forces
    /// a schema update (or this test and the round-trip tests fail).
    #[test]
    fn schema_covers_every_op() {
        use strum::VariantNames;
        for name in <Op as VariantNames>::VARIANTS {
            let datatype_variant = NODE_SCHEMA.contains(&format!("({name} "))
                || NODE_SCHEMA.contains(&format!("({name})"));
            let standalone_constructor = NODE_SCHEMA.contains(&format!("(constructor {name} "));
            assert!(
                datatype_variant || standalone_constructor,
                "schema is missing a constructor for Op::{name}"
            );
        }
    }
}
