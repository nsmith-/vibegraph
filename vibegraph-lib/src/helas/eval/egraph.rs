//! Skeleton of the egglog rewrite stage: round-trip an [`Ast<Sym>`] through an
//! egglog e-graph and back, unchanged.
//!
//! The lowered, binary-arity [`Ast<Sym>`] (see [`super::lower`]) maps onto an egglog
//! `datatype`: one constructor per [`Op`], its constructor name the same head token
//! the s-expression I/O uses ([`Op::name`]). Leaf payloads become leading base-sort
//! fields (`Coupling`/`Mass`/`Width` → `i64`, `Coeff` → `f64`, `CoeffRat` → three
//! leading `i64` fields `num den imag`, `External` → the
//! `leg spin sign incoming` quadruple); arena children become `Node` arguments. Every
//! op has fixed arity except [`Op::PMomOut`] (a vertex's whole input list) and
//! [`Op::Flows`] (the per-color-flow JAMP list), which take a `(Vec Node)` and so are
//! declared as separate `constructor`s after the vector sort.
//!
//! [`roundtrip`] declares the schema, then encodes the whole AST as a single `let`
//! binding whose value is the root constructor call with every child nested inline,
//! followed by an `extract` of that binding. Evaluating the `let` inserts the entire
//! tree in one traversal and rebuilds the database once — egglog rebuilds (its parallel
//! step) after each command, so a node-at-a-time encoding starves that step of work.
//! The gap between the `let` and the `extract` is where the future rewrite stage will
//! run its rule schedule. The extracted [`TermDag`] is then decoded back into an
//! `Ast<Sym>`.
//! Commands are built directly rather than rendered to text, so the encoding never
//! round-trips through egglog's parser. With no rewrite rules registered, extraction
//! returns exactly the inserted term, so the result is structurally identical to the
//! input. This is the seam the future algebraic-rewrite and congruence-CSE rules slot
//! into (see `research/notes/14-egglog-notes.md`); the rules will turn this identity
//! pass into an optimizing one.
//!
//! No production code consumes this module: measurement showed a greedy extractor
//! over the slot-traffic cost model cannot realize a sharing payoff on these
//! amplitudes, so extraction is not wired into the compile pipeline. The
//! enumeration and cost scaffolding here are kept for a future global (ILP-style)
//! extractor with a compute-aware cost model.

use std::collections::{HashMap, HashSet, VecDeque};

use egglog::ast::{Command, Expr, GenericAction, Literal, RustSpan, Span};
use egglog::{CommandOutput, EGraph, SerializeConfig, Term, TermDag, TermId};
use ordered_float::OrderedFloat;

use super::ast::{Ast, AstBuilder};
use super::op::{charge_from_sign, NodeId, Op, Sym};
use super::tree::Tree;
use crate::ufo::couplings::CouplingId;
use crate::ufo::particles::ParticleId;

/// The egglog schema for [`Ast<Sym>`]: the `Node` datatype (one constructor per
/// fixed-arity [`Op`], names matching [`Op::name`]), the `(Vec Node)` sort backing
/// [`Op::PMomOut`]/[`Op::Flows`]/[`Op::Hels`]/[`Op::Configs`], and those ops as
/// variable-arity constructors over it.
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
  (MetricVout Node)
  (IdentityAmp Node Node)
  (Gamma5 Node)
  (Gamma5Amp Node Node)
  (EpsilonVout Node Node Node)
  (EpsilonAmp Node Node Node Node)
  (FfvVout Node Node Node Node)
  (FfvIout Node Node Node Node)
  (FfvOout Node Node Node Node)
  (PMom Node)
  (Coupling i64)
  (Mass i64)
  (Width i64)
  (Coeff f64)
  (CoeffRat i64 i64 i64))
(sort NodeVec (Vec Node))
(constructor PMomOut (NodeVec) Node)
(constructor Flows (NodeVec) Node)
(constructor Hels (NodeVec) Node)
(constructor Configs (NodeVec) Node)
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

// ────────────────────────────── enumerate ──────────────────────────────────

/// A serialized e-class identifier: egglog's `sort-value` tag string (e.g.
/// `Node-42`, `i64-5`). Stable within one e-graph and consistent with the ids
/// `extract`/`TermDag` operate over, since both are keyed off the same
/// canonicalized backend value.
pub type ClassId = String;

/// The leaf value carried by an e-node whose e-class is a base (primitive) sort.
/// Constructor e-nodes (those whose e-class sort is `Node`/`NodeVec`) carry
/// [`Payload::None`]; their leaf data lives in child primitive e-classes.
#[derive(Debug, Clone, PartialEq)]
pub enum Payload {
    /// An ordinary constructor e-node — no primitive value of its own.
    None,
    /// A primitive `i64` leaf (a `Coupling`/`Mass`/`External` field, `CoeffRat` term…).
    Int(i64),
    /// A primitive `f64` leaf (a `Coeff` value).
    Float(f64),
    /// A primitive egglog rendered as neither `i64` nor `f64`.
    Other(String),
}

/// One e-node: a function symbol applied to child e-classes. Children are given
/// as [`ClassId`]s (resolved from the serialized graph's node-id edges), the form
/// a DAG-cost extractor consumes directly.
#[derive(Debug, Clone)]
pub struct ENode {
    /// The head symbol: an [`Op::name`] for constructor nodes, the rendered value
    /// for primitive nodes, the container tag for `NodeVec` nodes.
    pub op: String,
    /// The e-classes this e-node references, in argument order.
    pub children: Vec<ClassId>,
    /// The primitive value, when this e-node is a base-sort leaf.
    pub payload: Payload,
    /// Extraction cost egglog assigned this node (constructor cost or 1.0 for
    /// primitives); carried through for M2 to override with slot-traffic weights.
    pub cost: f64,
}

/// One e-class: a set of equivalent e-nodes sharing an id and sort.
#[derive(Debug, Clone)]
pub struct EClass {
    /// The canonical e-class id.
    pub id: ClassId,
    /// The egglog sort of this class (`Node`, `NodeVec`, `i64`, `f64`, …).
    pub sort: Option<String>,
    /// The e-nodes in this class. With no rewrite rules registered every class
    /// holds exactly one e-node.
    pub nodes: Vec<ENode>,
}

/// A whole e-graph enumerated into owned Rust structures: every e-class, its
/// e-nodes, and their child e-class ids, plus the root e-class(es). This is the
/// input format for a sharing-aware (DAG-cost) extractor built outside egglog —
/// egglog 2.0's own extraction is tree-cost only.
#[derive(Debug, Clone)]
pub struct DagEGraph {
    /// Every e-class, in serialization order.
    pub classes: Vec<EClass>,
    /// The e-class(es) the root expression evaluates to (one, for a single AST).
    pub roots: Vec<ClassId>,
    index: HashMap<ClassId, usize>,
}

impl DagEGraph {
    /// The e-class with id `id`, if present.
    pub fn class(&self, id: &str) -> Option<&EClass> {
        self.index.get(id).map(|&i| &self.classes[i])
    }

    /// Whether an e-class with id `id` exists.
    pub fn contains(&self, id: &str) -> bool {
        self.index.contains_key(id)
    }

    /// The number of e-classes of a given egglog sort.
    pub fn classes_of_sort(&self, sort: &str) -> usize {
        self.classes
            .iter()
            .filter(|c| c.sort.as_deref() == Some(sort))
            .count()
    }
}

/// Build the e-graph from `ast` exactly as [`roundtrip`] does (schema + a single
/// inlined insertion), then enumerate every e-class and e-node from egglog's
/// serialized view into an owned [`DagEGraph`]. No rewrite rules run, so the
/// result is the hash-consed input DAG: one e-node per e-class, one `Node`-sort
/// e-class per distinct AST subterm.
///
/// The enumeration goes through [`EGraph::serialize`], egglog's supported,
/// backend-canonicalized export (the same path its GraphViz/JSON tooling uses).
/// The alternative — iterating each function table via `function_to_dag` — routes
/// through the tree-cost `Extractor` per row, so it cannot recover raw e-node
/// child edges without re-imposing an extraction; `serialize` exposes them
/// directly.
pub fn enumerate(ast: &Ast<Sym>) -> Result<DagEGraph, EgraphError> {
    let mut egraph = EGraph::default();
    egraph
        .parse_and_run_program(None, NODE_SCHEMA)
        .map_err(|e| EgraphError::Egglog(e.to_string()))?;

    // Evaluating the fully-inlined root expression inserts every subterm (via each
    // constructor's get-or-make-set default) and returns the root's canonical
    // (sort, value) — the handle serialization needs to mark the root e-class.
    let root_expr = encode_expr(ast, ast.root());
    let (root_sort, root_value) = egraph
        .eval_expr(&root_expr)
        .map_err(|e| EgraphError::Egglog(e.to_string()))?;

    let out = egraph.serialize(SerializeConfig {
        root_eclasses: vec![(root_sort, root_value)],
        ..SerializeConfig::default()
    });

    // Translate egglog's serialized e-graph into owned structures: group nodes by
    // e-class, resolve each child node-id edge to the e-class it belongs to, and
    // recover primitive leaf payloads. `out.egraph` is `egraph_serialize::EGraph`
    // (a transitive dep); working through it by field/method access keeps that
    // type out of this crate's public surface.
    let serialized = &out.egraph;
    let mut classes = Vec::new();
    let mut index = HashMap::new();
    for (class_id, class) in serialized.classes() {
        let sort = serialized
            .class_data
            .get(class_id)
            .and_then(|d| d.typ.clone());
        let nodes = class
            .nodes
            .iter()
            .map(|node_id| {
                let node = &serialized[node_id];
                // A serialized child edge points at a specific node in the child
                // e-class; the child's e-class id is what a DAG extractor needs.
                let children = node
                    .children
                    .iter()
                    .map(|child| serialized[child].eclass.to_string())
                    .collect();
                ENode {
                    op: node.op.clone(),
                    children,
                    payload: payload_of(sort.as_deref(), &node.op),
                    cost: node.cost.into_inner(),
                }
            })
            .collect();
        let id = class_id.to_string();
        index.insert(id.clone(), classes.len());
        classes.push(EClass { id, sort, nodes });
    }
    let roots = serialized
        .root_eclasses
        .iter()
        .map(|c| c.to_string())
        .collect();
    Ok(DagEGraph {
        classes,
        roots,
        index,
    })
}

/// Recover the leaf value of a primitive e-node from its rendered op string.
/// Constructor and container e-classes (`Node`/`NodeVec`) carry no primitive.
fn payload_of(sort: Option<&str>, op: &str) -> Payload {
    match sort {
        None | Some("Node") | Some("NodeVec") => Payload::None,
        _ => {
            if let Ok(i) = op.parse::<i64>() {
                Payload::Int(i)
            } else if let Ok(f) = op.parse::<f64>() {
                Payload::Float(f)
            } else {
                Payload::Other(op.to_string())
            }
        }
    }
}

// ─────────────────────────────── encode ────────────────────────────────────

/// The global bound to the whole encoded amplitude, between insertion and extraction.
const ROOT_VAR: &str = "$root";

/// Build the encoding commands: one `let` binding the whole AST as a single fully-inlined
/// expression, then an `extract` of that binding. Evaluating the `let` inserts the entire
/// tree in one traversal and rebuilds the database once, rather than once per node —
/// egglog's parallel rebuild has no work to do on a one-node delta. Binding the root
/// (rather than extracting the expression directly) leaves the seam where the future
/// rewrite stage inserts its `run` schedule between insertion and extraction. Assumes the
/// schema ([`NODE_SCHEMA`]) has already been declared on the e-graph.
///
/// The lowered arena is a tree (each node has one parent; see [`super::lower::lower`]),
/// so inlining expands nothing. Were a shared subtree present, egglog would hash-cons
/// it back together on insert, so the extracted result is unaffected either way.
fn encode_commands(ast: &Ast<Sym>) -> Vec<Command> {
    vec![
        Command::Action(GenericAction::Let(
            span(),
            ROOT_VAR.to_string(),
            encode_expr(ast, ast.root()),
        )),
        // Default variant count (0) — the same expr the s-expr parser fills in for `(extract e)`.
        Command::Extract(span(), var(ROOT_VAR.to_string()), int(0)),
    ]
}

/// Encode one node as an egglog constructor call `(OpName leaf-fields… child…)`, its
/// children inlined as nested expressions.
fn encode_expr(ast: &Ast<Sym>, id: NodeId) -> Expr {
    let node = ast.value(id);
    let kids = ast.children_ids(id);
    let mut args = Vec::with_capacity(kids.len() + 4);
    match (node.op, &node.leaf) {
        (Op::Coupling, Sym::Coupling(c)) => args.push(int(c.index() as i64)),
        (Op::Mass | Op::Width, Sym::Particle(p)) => args.push(int(p.index() as i64)),
        (Op::Coeff, Sym::Coeff(c)) => args.push(float(*c)),
        (Op::CoeffRat, Sym::Rational { num, den, imag }) => {
            args.push(int(*num));
            args.push(int(*den));
            args.push(int(*imag as i64));
        }
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
    if matches!(node.op, Op::PMomOut | Op::Flows | Op::Hels | Op::Configs) {
        // The variable-arity ops: children go inside a Vec argument.
        let elems: Vec<Expr> = kids.iter().map(|&k| encode_expr(ast, k)).collect();
        args.push(if elems.is_empty() {
            call("vec-empty", vec![])
        } else {
            call("vec-of", elems)
        });
    } else {
        args.extend(kids.iter().map(|&k| encode_expr(ast, k)));
    }
    call(node.op.name(), args)
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
        Op::CoeffRat => {
            let leaf = Sym::Rational {
                num: int_arg(dag, kids, 0)?,
                den: int_arg(dag, kids, 1)?,
                imag: int_arg(dag, kids, 2)? != 0,
            };
            (leaf, vec![])
        }
        Op::External => {
            let leaf = Sym::Ext {
                leg_idx: int_arg(dag, kids, 0)? as usize,
                spin: int_arg(dag, kids, 1)? as i32,
                charge: charge_from_sign(int_arg(dag, kids, 2)? as i32),
                incoming: int_arg(dag, kids, 3)? != 0,
            };
            (leaf, kids.get(4..).unwrap_or_default().to_vec())
        }
        Op::PMomOut | Op::Flows | Op::Hels | Op::Configs => {
            let vec_id = *kids.first().ok_or_else(|| {
                decode_err(format_args!("{} without its Vec argument", op.name()))
            })?;
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

// ───────────────────────── DAG-cost extraction ──────────────────────────────

/// A per-e-node cost assignment, keyed on the node's e-class sort and head op. The
/// extractor is generic over this so a slot-traffic model and a uniform (or any
/// future) model plug into the same greedy machinery; the tree- vs DAG-cost choice
/// is orthogonal (see [`CostKind`]).
pub trait CostModel {
    /// Cost charged for selecting `op` in an e-class of the given `sort`.
    fn node_cost(&self, sort: Option<&str>, op: &str) -> f64;
}

/// Cost ≈ the bytes of the runtime output slot each op materializes (the "slot
/// traffic" the evaluator moves per node): off-shell fermion/vector currents are the
/// hot 96 B majority, scalar bilinears/contractions 16 B, constant leaves and
/// primitive base-sort values carry no runtime slot, and the variadic containers
/// (`NodeVec`) pass their cost straight through to their elements.
///
/// The algebraic combinators `Mul`/`Add` (and the `Flows` root) have an
/// operand-dependent output type a per-op map cannot resolve; they predominantly
/// scale or sum currents in these amplitudes, so they are charged at the current
/// slot. A future static output-type analysis would refine the scalar-valued cases;
/// the model is a swappable parameter precisely so that refinement is a drop-in.
pub struct SlotTrafficCost;

impl CostModel for SlotTrafficCost {
    fn node_cost(&self, sort: Option<&str>, op: &str) -> f64 {
        match sort {
            Some("Node") => op_slot_bytes(op),
            // NodeVec containers pass through; primitive leaves are free.
            _ => 0.0,
        }
    }
}

/// The slot-traffic weight of a `Node`-sort constructor, by head op.
fn op_slot_bytes(op: &str) -> f64 {
    match Op::from_name(op) {
        Some(op) => match op {
            Op::External
            | Op::Propagate
            | Op::GammaVout
            | Op::GammaIout
            | Op::GammaOout
            | Op::ProjM
            | Op::ProjP
            | Op::MetricVout
            | Op::EpsilonVout
            | Op::Gamma5
            | Op::FfvVout
            | Op::FfvIout
            | Op::FfvOout
            | Op::PMom
            | Op::PMomOut => 96.0,
            Op::Metric
            | Op::ProjMAmp
            | Op::ProjPAmp
            | Op::IdentityAmp
            | Op::Gamma5Amp
            | Op::EpsilonAmp => 16.0,
            Op::Mul | Op::Add | Op::Flows | Op::Hels | Op::Configs => 96.0,
            Op::Coupling | Op::Mass | Op::Width | Op::Coeff | Op::CoeffRat => 0.0,
        },
        None => 0.0,
    }
}

/// Uniform unit cost (every constructor and container weighs 1, primitives 0) — the
/// simplest model, handy as an M3 baseline for comparing a rewrite's node-count
/// effect independent of slot weights.
pub struct UnitCost;

impl CostModel for UnitCost {
    fn node_cost(&self, sort: Option<&str>, _op: &str) -> f64 {
        match sort {
            Some("Node") | Some("NodeVec") => 1.0,
            _ => 0.0,
        }
    }
}

/// Whether a candidate's cost counts each shared descendant e-class once
/// (sharing-aware, [`CostKind::Dag`]) or once per occurrence (tree-shaped,
/// [`CostKind::Tree`], reproducing egglog's own `TreeAdditiveCostModel`). Both run
/// the same greedy fixpoint over the same [`CostModel`], so a sharing rule's payoff
/// is visible to `Dag` and invisible to `Tree` — the comparison the sharing-rule
/// demos need, with no separate plumbing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CostKind {
    /// Cost of the *set* of chosen descendant e-classes (shared classes once).
    Dag,
    /// Cost summed over the chosen tree (shared classes per occurrence).
    Tree,
}

/// The result of extracting a single e-node from every reachable e-class: the chosen
/// node index per e-class, and the cost of the root's extraction under the model.
#[derive(Debug, Clone)]
pub struct Extraction {
    /// Chosen e-node index within each costed e-class.
    choices: HashMap<ClassId, usize>,
    /// Cost of the first root's extraction under the model and [`CostKind`] used.
    pub root_cost: f64,
    /// Which cost interpretation produced this extraction.
    pub kind: CostKind,
}

impl Extraction {
    /// The e-node chosen for `cid`, if that class was reached and costed.
    pub fn choice(&self, cid: &str) -> Option<usize> {
        self.choices.get(cid).copied()
    }

    /// Number of distinct `Node`-sort e-classes reachable from the roots through the
    /// chosen e-nodes — the size of the extracted DAG in AST nodes (`NodeVec`
    /// containers and primitive leaves are not AST nodes, so they are not counted).
    /// On the rule-free graph this equals the input AST's distinct-subterm count.
    pub fn reachable_node_count(&self, dag: &DagEGraph) -> usize {
        let mut seen: HashSet<&str> = HashSet::new();
        let mut stack: Vec<&str> = dag.roots.iter().map(|s| s.as_str()).collect();
        let mut count = 0;
        while let Some(cid) = stack.pop() {
            if !seen.insert(cid) {
                continue;
            }
            let Some(class) = dag.class(cid) else {
                continue;
            };
            if class.sort.as_deref() == Some("Node") {
                count += 1;
            }
            if let Some(&idx) = self.choices.get(cid) {
                for child in &class.nodes[idx].children {
                    stack.push(child.as_str());
                }
            }
        }
        count
    }
}

/// One class's best extraction so far: the chosen e-node, its total cost, and (for
/// [`CostKind::Dag`]) the set of descendant e-classes it draws in, each mapped to the
/// op cost of the node chosen for it. The set is what makes the cost sharing-aware —
/// a class reachable by two paths appears once, counted once.
struct CostSet {
    total: f64,
    choice: usize,
    /// Descendant e-classes in this extraction → their per-node op cost. Empty for
    /// [`CostKind::Tree`], which needs no dedup.
    set: HashMap<ClassId, f64>,
}

/// Greedy DAG-cost extraction over a [`DagEGraph`], after the extraction-gym
/// `faster-greedy-dag` algorithm: iterate a worklist of e-classes to a fixpoint,
/// each class taking the min-cost node whose children are all already costed, and
/// re-examining a class's parents whenever its cost improves. A candidate node's DAG
/// cost is its own op cost plus the cost of the *union* of its children's chosen
/// descendant sets (shared classes counted once).
///
/// **Cycle guard.** A node is only ever costed once every child e-class already has a
/// finite cost, so a class whose only nodes are (transitively) self-referential never
/// enters `costs` and is reported as having no finite extraction. As a second guard,
/// a candidate whose unioned descendant set already contains its own class is
/// rejected (it would be an extraction that includes itself). Rule-free graphs here
/// are acyclic, but rewrite rules make them cyclic, and this extractor is the point.
pub fn extract(
    dag: &DagEGraph,
    model: &dyn CostModel,
    kind: CostKind,
) -> Result<Extraction, EgraphError> {
    // For each e-class, the classes that reference it as a child — the wake-up list
    // when its cost improves.
    let mut parents: HashMap<&str, Vec<&str>> = HashMap::new();
    for class in &dag.classes {
        for node in &class.nodes {
            for child in &node.children {
                parents.entry(child).or_default().push(&class.id);
            }
        }
    }

    let mut costs: HashMap<ClassId, CostSet> = HashMap::new();
    let mut queue: VecDeque<&str> = VecDeque::new();
    let mut queued: HashSet<&str> = HashSet::new();
    // Seed with classes that have at least one childless node (the base cases:
    // primitive leaves and any nullary constructor).
    for class in &dag.classes {
        if class.nodes.iter().any(|n| n.children.is_empty()) && queued.insert(&class.id) {
            queue.push_back(&class.id);
        }
    }

    while let Some(cid) = queue.pop_front() {
        queued.remove(cid);
        let class = dag.class(cid).expect("queued id resolves");
        let sort = class.sort.as_deref();
        let mut best: Option<CostSet> = None;
        for (idx, node) in class.nodes.iter().enumerate() {
            if !node.children.iter().all(|c| costs.contains_key(c.as_str())) {
                continue;
            }
            let node_cost = model.node_cost(sort, &node.op);
            let candidate = match kind {
                CostKind::Dag => {
                    let mut set: HashMap<ClassId, f64> = HashMap::new();
                    for c in &node.children {
                        for (k, v) in &costs[c.as_str()].set {
                            set.entry(k.clone()).or_insert(*v);
                        }
                    }
                    // Self-inclusion would make this a cyclic extraction.
                    if set.contains_key(cid) {
                        continue;
                    }
                    set.insert(cid.to_string(), node_cost);
                    let total: f64 = set.values().sum();
                    CostSet {
                        total,
                        choice: idx,
                        set,
                    }
                }
                CostKind::Tree => {
                    let total = node_cost
                        + node
                            .children
                            .iter()
                            .map(|c| costs[c.as_str()].total)
                            .sum::<f64>();
                    CostSet {
                        total,
                        choice: idx,
                        set: HashMap::new(),
                    }
                }
            };
            if best.as_ref().is_none_or(|b| candidate.total < b.total) {
                best = Some(candidate);
            }
        }

        if let Some(cand) = best {
            let improved = costs.get(cid).is_none_or(|old| cand.total < old.total);
            if improved {
                costs.insert(cid.to_string(), cand);
                if let Some(ps) = parents.get(cid) {
                    for &p in ps {
                        if queued.insert(p) {
                            queue.push_back(p);
                        }
                    }
                }
            }
        }
    }

    for r in &dag.roots {
        if !costs.contains_key(r) {
            return Err(EgraphError::Decode(format!(
                "root e-class {r} has no finite extraction (cyclic or unreachable)"
            )));
        }
    }
    let root_cost = dag
        .roots
        .first()
        .and_then(|r| costs.get(r))
        .map(|c| c.total)
        .unwrap_or(0.0);
    let choices = costs.into_iter().map(|(k, v)| (k, v.choice)).collect();
    Ok(Extraction {
        choices,
        root_cost,
        kind,
    })
}

/// Decode an [`Extraction`] back into an `Ast<Sym>` by walking the chosen e-node of
/// each e-class from the root, memoized on [`ClassId`] so shared e-classes become
/// shared arena nodes — the same DAG-preserving decode as [`decode`], sourced from
/// the extractor's selection instead of a [`TermDag`]. On the rule-free graph the
/// result is byte-identical to the enumerated AST.
pub fn decode_extraction(dag: &DagEGraph, ex: &Extraction) -> Result<Ast<Sym>, EgraphError> {
    let root = dag.roots.first().ok_or(EgraphError::NoExtract)?;
    let mut b = AstBuilder::new();
    let mut memo: HashMap<ClassId, NodeId> = HashMap::new();
    let nid = decode_class(dag, ex, root, &mut b, &mut memo)?;
    Ok(b.finish(nid))
}

/// The chosen e-node of e-class `cid`.
fn chosen<'a>(dag: &'a DagEGraph, ex: &Extraction, cid: &str) -> Result<&'a ENode, EgraphError> {
    let class = dag
        .class(cid)
        .ok_or_else(|| decode_err(format_args!("e-class {cid} does not resolve")))?;
    let idx = ex
        .choices
        .get(cid)
        .ok_or_else(|| decode_err(format_args!("e-class {cid} was not extracted")))?;
    class
        .nodes
        .get(*idx)
        .ok_or_else(|| decode_err(format_args!("chosen node {idx} out of range in {cid}")))
}

/// The `i64` leaf value of a primitive child e-class.
fn prim_int(dag: &DagEGraph, ex: &Extraction, cid: &str) -> Result<i64, EgraphError> {
    match &chosen(dag, ex, cid)?.payload {
        Payload::Int(v) => Ok(*v),
        other => Err(decode_err(format_args!(
            "expected an i64 leaf in {cid}, found {other:?}"
        ))),
    }
}

/// The `f64` leaf value of a primitive child e-class.
fn prim_float(dag: &DagEGraph, ex: &Extraction, cid: &str) -> Result<f64, EgraphError> {
    match &chosen(dag, ex, cid)?.payload {
        Payload::Float(v) => Ok(*v),
        other => Err(decode_err(format_args!(
            "expected an f64 leaf in {cid}, found {other:?}"
        ))),
    }
}

/// The child e-class at argument position `i`.
fn child_at<'a>(kids: &'a [ClassId], i: usize, op: &str) -> Result<&'a str, EgraphError> {
    kids.get(i)
        .map(|s| s.as_str())
        .ok_or_else(|| decode_err(format_args!("{op} missing child argument {i}")))
}

/// The element e-classes of a `NodeVec` container class (`vec-of` / `vec-empty`).
fn vec_element_classes(
    dag: &DagEGraph,
    ex: &Extraction,
    cid: &str,
) -> Result<Vec<ClassId>, EgraphError> {
    let node = chosen(dag, ex, cid)?;
    match node.op.as_str() {
        "vec-of" => Ok(node.children.clone()),
        "vec-empty" => Ok(vec![]),
        other => Err(decode_err(format_args!(
            "expected a NodeVec container, found `{other}` in {cid}"
        ))),
    }
}

/// Rebuild the `Ast<Sym>` node for e-class `cid` from its chosen e-node, recursing
/// into the chosen child e-classes. Memoized on [`ClassId`] so a shared e-class
/// yields one arena node. Mirrors the argument split of [`decode`], reading leaf
/// primitives from child base-sort classes rather than `TermDag` literals.
fn decode_class(
    dag: &DagEGraph,
    ex: &Extraction,
    cid: &str,
    b: &mut AstBuilder<Sym>,
    memo: &mut HashMap<ClassId, NodeId>,
) -> Result<NodeId, EgraphError> {
    if let Some(&n) = memo.get(cid) {
        return Ok(n);
    }
    let node = chosen(dag, ex, cid)?;
    let name = node.op.as_str();
    let op = Op::from_name(name).ok_or_else(|| decode_err(format_args!("unknown op `{name}`")))?;
    let kids = &node.children;

    let (leaf, child_classes): (Sym, Vec<ClassId>) = match op {
        Op::Coupling => (
            Sym::Coupling(CouplingId::from(
                prim_int(dag, ex, child_at(kids, 0, name)?)? as usize,
            )),
            vec![],
        ),
        Op::Mass | Op::Width => (
            Sym::Particle(ParticleId::from(
                prim_int(dag, ex, child_at(kids, 0, name)?)? as usize,
            )),
            vec![],
        ),
        Op::Coeff => (
            Sym::Coeff(prim_float(dag, ex, child_at(kids, 0, name)?)?),
            vec![],
        ),
        Op::CoeffRat => {
            let leaf = Sym::Rational {
                num: prim_int(dag, ex, child_at(kids, 0, name)?)?,
                den: prim_int(dag, ex, child_at(kids, 1, name)?)?,
                imag: prim_int(dag, ex, child_at(kids, 2, name)?)? != 0,
            };
            (leaf, vec![])
        }
        Op::External => {
            let leaf = Sym::Ext {
                leg_idx: prim_int(dag, ex, child_at(kids, 0, name)?)? as usize,
                spin: prim_int(dag, ex, child_at(kids, 1, name)?)? as i32,
                charge: charge_from_sign(prim_int(dag, ex, child_at(kids, 2, name)?)? as i32),
                incoming: prim_int(dag, ex, child_at(kids, 3, name)?)? != 0,
            };
            (leaf, kids.get(4..).unwrap_or_default().to_vec())
        }
        Op::PMomOut | Op::Flows | Op::Hels | Op::Configs => (
            Sym::None,
            vec_element_classes(dag, ex, child_at(kids, 0, name)?)?,
        ),
        _ => (Sym::None, kids.clone()),
    };

    let children = child_classes
        .iter()
        .map(|c| decode_class(dag, ex, c, b, memo))
        .collect::<Result<Vec<_>, _>>()?;
    let nid = b.add(op, leaf, children);
    memo.insert(cid.to_string(), nid);
    Ok(nid)
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
        assert_roundtrip("(CoeffRat (Rational 1 3 0))");
    }

    #[test]
    fn coeff_rat_real_and_imaginary_and_negative() {
        assert_roundtrip("(CoeffRat (Rational 1 1 0))");
        assert_roundtrip("(CoeffRat (Rational -1 3 0))");
        assert_roundtrip("(CoeffRat (Rational 2 9 1))");
        assert_roundtrip("(CoeffRat (Rational -2 9 1))");
    }

    #[test]
    fn flows_variadic_arg() {
        assert_roundtrip(
            "(Flows (CoeffRat (Rational 1 1 0)) \
                     (CoeffRat (Rational 1 3 0)) \
                     (CoeffRat (Rational -1 3 1)))",
        );
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

    /// Compile each process to its binary `Ast<Sym>` and assert the egglog round-trip
    /// returns it byte-for-byte. Shared by the full-suite and the fast rewrite-dev tests.
    fn assert_processes_roundtrip(processes: &[&str]) {
        use super::super::lower;
        use super::super::root_diagram::compile_diagram_ast;
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use crate::ufo::sm::{sm_model, SMRestrict};

        let model = sm_model(SMRestrict::Default);
        let opts = ParsingOptions::default();
        for &process in processes {
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

    /// One representative 2→2, 2→3, and 2→4 process — enough op variety (multi-diagram
    /// sharing, a radiated photon, fused `Ffv*` kernels, massive externals) to develop
    /// and regression-test egglog rewrite rules against, while staying fast enough for
    /// the default `cargo test`. The full validated suite is
    /// `representative_processes_roundtrip` (gated behind `extended-validation`).
    #[test]
    fn rewrite_dev_processes_roundtrip() {
        assert_processes_roundtrip(&[
            "e+ e- > mu+ mu-",               // 2→2
            "e+ e- > mu+ mu- a",             // 2→3
            "e+ e- > mu+ mu- ta+ ta- QCD=0", // 2→4
        ]);
    }

    /// The real payoff: every MG-validated process, compiled to its binary `Ast<Sym>`,
    /// survives the egglog round-trip byte-for-byte. This exercises genuine HELAS output
    /// — multi-diagram sharing, `PMomOut` from the triple-gauge `W+ W-` vertex, fused
    /// `Ffv*` kernels, massive externals — not just the hand-written fixtures above, up
    /// to the 2→6 EW amplitudes. The full suite round-trips in about a second because the
    /// whole AST is inserted as one expression (see [`encode_commands`]): egglog rebuilds
    /// its database once, not once per node. Runs the same `MG_VALIDATED_PROCESSES`
    /// suite as `amplitude_oracle`, so it never drifts from the validated set.
    ///
    /// Gated behind `extended-validation`: egglog is ~100× slower unoptimized (~1s
    /// release, ~140s debug over the full suite), too heavy for the default `cargo test`.
    ///
    /// ```text
    /// cargo test -p vibegraph-lib --release --features extended-validation \
    ///   representative_processes_roundtrip
    /// ```
    #[cfg(feature = "extended-validation")]
    #[test]
    fn representative_processes_roundtrip() {
        use super::super::compile::MG_VALIDATED_PROCESSES;
        assert_processes_roundtrip(&MG_VALIDATED_PROCESSES);
    }

    // ─────────────────────────── enumeration ───────────────────────────────

    /// Number of structurally-distinct subterms of `ast`: the count egglog's
    /// hash-consing collapses the arena to, computed here independently by
    /// assigning each distinct `(op, leaf, child-keys)` a fresh key bottom-up.
    fn distinct_ast_subterms(ast: &Ast<Sym>) -> usize {
        fn key(
            ast: &Ast<Sym>,
            id: NodeId,
            keys: &mut HashMap<String, usize>,
            memo: &mut HashMap<NodeId, usize>,
        ) -> usize {
            if let Some(&k) = memo.get(&id) {
                return k;
            }
            let child_keys: Vec<usize> = ast
                .children_ids(id)
                .iter()
                .map(|&c| key(ast, c, keys, memo))
                .collect();
            let node = ast.value(id);
            let s = format!("{}|{:?}|{:?}", node.op.name(), node.leaf, child_keys);
            let next = keys.len();
            let k = *keys.entry(s).or_insert(next);
            memo.insert(id, k);
            k
        }
        let mut keys = HashMap::new();
        let mut memo = HashMap::new();
        key(ast, ast.root(), &mut keys, &mut memo);
        keys.len()
    }

    /// The set of `Op` head tokens appearing anywhere in `ast`.
    fn ast_op_names(ast: &Ast<Sym>) -> HashSet<&'static str> {
        ast.iter().map(|id| ast.value(id).op.name()).collect()
    }

    /// Enumerate `ast` and assert the result is a consistent view of the rule-free
    /// (hash-consed identity) e-graph: one `Node` e-class per distinct AST subterm,
    /// exactly one e-node per class, every AST op present, every child id resolvable,
    /// and a root that resolves to a `Node` class.
    fn assert_enumeration_consistent(ast: &Ast<Sym>, ctx: &str) {
        let dag = enumerate(ast).unwrap_or_else(|e| panic!("[{ctx}] enumerate failed: {e}"));

        // No rules ran, so structural identity: #Node e-classes == #distinct subterms.
        assert_eq!(
            dag.classes_of_sort("Node"),
            distinct_ast_subterms(ast),
            "[{ctx}] Node e-class count != distinct AST subterm count",
        );

        // Every child id resolves to a real e-class, and each e-node has a head op.
        for class in &dag.classes {
            assert!(
                !class.nodes.is_empty(),
                "[{ctx}] empty e-class {}",
                class.id
            );
            for node in &class.nodes {
                assert!(!node.op.is_empty(), "[{ctx}] e-node with empty op");
                for child in &node.children {
                    assert!(
                        dag.contains(child),
                        "[{ctx}] child id {child} does not resolve to an e-class",
                    );
                }
            }
        }

        // With no unions, every e-class holds exactly one e-node.
        for class in dag
            .classes
            .iter()
            .filter(|c| c.sort.as_deref() == Some("Node"))
        {
            assert_eq!(
                class.nodes.len(),
                1,
                "[{ctx}] rule-free Node e-class {} has {} e-nodes",
                class.id,
                class.nodes.len(),
            );
        }

        // Every op used by the AST shows up as some Node e-node head.
        let enumerated_ops: HashSet<&str> = dag
            .classes
            .iter()
            .filter(|c| c.sort.as_deref() == Some("Node"))
            .flat_map(|c| c.nodes.iter().map(|n| n.op.as_str()))
            .collect();
        for op in ast_op_names(ast) {
            assert!(
                enumerated_ops.contains(op),
                "[{ctx}] AST op `{op}` absent from the enumerated e-graph",
            );
        }

        // The single root resolves and is a Node e-class.
        assert_eq!(dag.roots.len(), 1, "[{ctx}] expected exactly one root");
        let root = &dag.roots[0];
        let root_class = dag
            .class(root)
            .unwrap_or_else(|| panic!("[{ctx}] root id {root} does not resolve"));
        assert_eq!(
            root_class.sort.as_deref(),
            Some("Node"),
            "[{ctx}] root e-class is not a Node",
        );
    }

    fn assert_enumeration_str(sexpr: &str) {
        let ast: Ast<Sym> = sexpr.parse().expect("parse input s-expr");
        assert_enumeration_consistent(&ast, sexpr);
    }

    /// Leaf-only fixtures: each is a single `Node` e-class over one or more
    /// primitive child classes; primitive leaf values decode into [`Payload`].
    #[test]
    fn enumerate_leaves_all_flavors() {
        assert_enumeration_str("(Coupling (CouplingId 5))");
        assert_enumeration_str("(Mass (ParticleId 11))");
        assert_enumeration_str("(Coeff (Real 1.5))");
        assert_enumeration_str("(CoeffRat (Rational 1 3 0))");
        assert_enumeration_str("(External (ExtLegInfo 0 2 1 1) (Mass (ParticleId 11)))");
    }

    /// A nested vertex and a variadic `PMomOut` (its children live behind a
    /// `NodeVec` container e-class): child edges must still resolve.
    #[test]
    fn enumerate_nested_and_variadic() {
        assert_enumeration_str(
            "(Mul (Coupling (CouplingId 5)) (Add \
               (Mul (Coeff (Real 1.5)) \
                    (GammaVout (External (ExtLegInfo 0 2 1 1) (Mass (ParticleId 11))) \
                               (External (ExtLegInfo 1 2 -1 0) (Mass (ParticleId 11))))) \
               (Mul (Coeff (Real -1.0)) \
                    (ProjM (External (ExtLegInfo 2 2 1 1) (Mass (ParticleId 0)))))))",
        );
        assert_enumeration_str(
            "(PMomOut (External (ExtLegInfo 0 2 1 1) (Mass (ParticleId 24))) \
                      (External (ExtLegInfo 1 2 -1 0) (Mass (ParticleId 24))) \
                      (External (ExtLegInfo 2 3 1 1) (Mass (ParticleId 23))))",
        );
    }

    /// Leaf primitives decode into typed payloads, and a genuine DAG (a shared
    /// subtree feeding two parents) enumerates to a single shared child e-class,
    /// not two — so #Node e-classes stays below the arena-node count.
    #[test]
    fn enumerate_shared_subtree_dag() {
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
        let root = b.add(Op::Add, Sym::None, vec![ext, ext]);
        let ast = b.finish(root);
        assert_enumeration_consistent(&ast, "shared-subtree-dag");

        // Add, External, Mass — three distinct subterms despite External appearing twice.
        let dag = enumerate(&ast).unwrap();
        assert_eq!(dag.classes_of_sort("Node"), 3);
        // The External e-class is referenced by the Add node's two children as one id.
        let add = dag
            .classes
            .iter()
            .find(|c| c.nodes[0].op == "Add")
            .expect("Add class");
        assert_eq!(add.nodes[0].children.len(), 2);
        assert_eq!(add.nodes[0].children[0], add.nodes[0].children[1]);

        // The Mass e-class's i64 child carries an Int payload of 11.
        let has_int_11 = dag
            .classes
            .iter()
            .flat_map(|c| &c.nodes)
            .any(|n| n.payload == Payload::Int(11));
        assert!(has_int_11, "expected a primitive Int(11) payload");
    }

    /// The real ops: each rewrite-dev process, compiled to its binary `Ast<Sym>`,
    /// enumerates into a consistent e-graph (the rule-free structural-identity gate
    /// M2's DAG extractor must reproduce).
    #[test]
    fn enumerate_dev_processes() {
        use super::super::lower;
        use super::super::root_diagram::compile_diagram_ast;
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use crate::ufo::sm::{sm_model, SMRestrict};

        let model = sm_model(SMRestrict::Default);
        let opts = ParsingOptions::default();
        for process in [
            "e+ e- > mu+ mu-",
            "e+ e- > mu+ mu- a",
            "e+ e- > mu+ mu- ta+ ta- QCD=0",
        ] {
            let pc = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
            let sets = generate_from_proc_card(&pc, &model).unwrap();
            assert!(!sets.is_empty(), "no diagrams for '{process}'");
            for set in &sets {
                let diagrams = compile_diagram_ast(set, &model).unwrap();
                let ast = lower::lower(&diagrams);
                assert_enumeration_consistent(&ast, process);
            }
        }
    }

    // ─────────────────────────── extraction ────────────────────────────────

    /// Extract `ast` with the DAG-cost greedy extractor under the slot-traffic model,
    /// then assert the decoded result reproduces the input exactly: byte-identical
    /// s-expression and one extracted `Node` per distinct input subterm. On the
    /// rule-free e-graph every class holds one node, so extraction is forced and this
    /// is the identity — the M2 acceptance criterion.
    fn assert_extraction_identity(ast: &Ast<Sym>, ctx: &str) {
        let dag = enumerate(ast).unwrap_or_else(|e| panic!("[{ctx}] enumerate failed: {e}"));
        let ex = extract(&dag, &SlotTrafficCost, CostKind::Dag)
            .unwrap_or_else(|e| panic!("[{ctx}] extract failed: {e}"));
        let out =
            decode_extraction(&dag, &ex).unwrap_or_else(|e| panic!("[{ctx}] decode failed: {e}"));
        assert_eq!(
            out.to_string(),
            ast.to_string(),
            "[{ctx}] extracted AST is not byte-identical to the input",
        );
        assert_eq!(
            ex.reachable_node_count(&dag),
            distinct_ast_subterms(ast),
            "[{ctx}] extracted Node count != distinct input subterm count",
        );
        // Rule-free extraction has one node per class, so DAG and tree extraction pick
        // the same nodes and decode identically (they differ only on shared costs).
        let ex_tree = extract(&dag, &SlotTrafficCost, CostKind::Tree)
            .unwrap_or_else(|e| panic!("[{ctx}] tree extract failed: {e}"));
        let out_tree = decode_extraction(&dag, &ex_tree)
            .unwrap_or_else(|e| panic!("[{ctx}] tree decode failed: {e}"));
        assert_eq!(
            out_tree.to_string(),
            ast.to_string(),
            "[{ctx}] tree-cost extraction changed the AST",
        );
    }

    fn assert_extraction_identity_str(sexpr: &str) {
        let ast: Ast<Sym> = sexpr.parse().expect("parse input s-expr");
        assert_extraction_identity(&ast, sexpr);
    }

    /// The fixtures the round-trip/enumeration tests use, now through the extractor:
    /// leaves, a nested vertex, a variadic `PMomOut`, a fused `Ffv*`/propagator.
    #[test]
    fn extract_identity_fixtures() {
        assert_extraction_identity_str("(Coupling (CouplingId 5))");
        assert_extraction_identity_str("(Mass (ParticleId 11))");
        assert_extraction_identity_str("(Coeff (Real 1.5))");
        assert_extraction_identity_str("(CoeffRat (Rational -2 9 1))");
        assert_extraction_identity_str("(External (ExtLegInfo 0 2 1 1) (Mass (ParticleId 11)))");
        assert_extraction_identity_str(
            "(Mul (Coupling (CouplingId 5)) (Add \
               (Mul (Coeff (Real 1.5)) \
                    (GammaVout (External (ExtLegInfo 0 2 1 1) (Mass (ParticleId 11))) \
                               (External (ExtLegInfo 1 2 -1 0) (Mass (ParticleId 11))))) \
               (Mul (Coeff (Real -1.0)) \
                    (ProjM (External (ExtLegInfo 2 2 1 1) (Mass (ParticleId 0)))))))",
        );
        assert_extraction_identity_str(
            "(Propagate \
               (FfvVout (External (ExtLegInfo 0 2 1 1) (Mass (ParticleId 11))) \
                        (External (ExtLegInfo 1 2 -1 0) (Mass (ParticleId 11))) \
                        (Coupling (CouplingId 3)) (Coupling (CouplingId 4))) \
               (Mass (ParticleId 23)) (Width (ParticleId 23)))",
        );
        assert_extraction_identity_str(
            "(PMomOut (External (ExtLegInfo 0 2 1 1) (Mass (ParticleId 24))) \
                      (External (ExtLegInfo 1 2 -1 0) (Mass (ParticleId 24))) \
                      (External (ExtLegInfo 2 3 1 1) (Mass (ParticleId 23))))",
        );
    }

    /// A shared subtree exercises the DAG-vs-tree cost distinction directly: the
    /// External current feeds both `Add` operands, so the tree cost double-counts it
    /// while the DAG cost counts it once. Both extractions still decode to the same
    /// (shared) AST — this is the seam M3's sharing-rule demo measures across.
    #[test]
    fn extract_dag_vs_tree_shared_subtree() {
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
        let root = b.add(Op::Add, Sym::None, vec![ext, ext]);
        let ast = b.finish(root);

        assert_extraction_identity(&ast, "shared-subtree");

        let dag = enumerate(&ast).unwrap();
        let dag_ex = extract(&dag, &SlotTrafficCost, CostKind::Dag).unwrap();
        let tree_ex = extract(&dag, &SlotTrafficCost, CostKind::Tree).unwrap();
        // Add(96) + External(96, shared once) + Mass(0) = 192 as a DAG.
        assert_eq!(dag_ex.root_cost, 192.0);
        // Tree double-counts the shared External: Add(96) + 2·External(96) = 288.
        assert_eq!(tree_ex.root_cost, 288.0);
        assert!(
            dag_ex.root_cost < tree_ex.root_cost,
            "DAG cost must reward the shared subterm the tree cost cannot see",
        );
        // The extracted DAG still has the shared child as one arena node.
        let out = decode_extraction(&dag, &dag_ex).unwrap();
        let kids = out.children_ids(out.root());
        assert_eq!(kids[0], kids[1]);
    }

    /// Compile each process to its binary `Ast<Sym>` and assert the DAG extractor
    /// reproduces it byte-for-byte with the CSE node count intact.
    fn assert_processes_extract_identity(processes: &[&str]) {
        use super::super::lower;
        use super::super::root_diagram::compile_diagram_ast;
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use crate::ufo::sm::{sm_model, SMRestrict};

        let model = sm_model(SMRestrict::Default);
        let opts = ParsingOptions::default();
        for &process in processes {
            let pc = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
            let sets = generate_from_proc_card(&pc, &model).unwrap();
            assert!(!sets.is_empty(), "no diagrams for '{process}'");
            for set in &sets {
                let diagrams = compile_diagram_ast(set, &model).unwrap();
                let ast = lower::lower(&diagrams);

                let dag = enumerate(&ast).unwrap();
                let t0 = std::time::Instant::now();
                let ex = extract(&dag, &SlotTrafficCost, CostKind::Dag)
                    .unwrap_or_else(|e| panic!("[{process}] extract failed: {e}"));
                let dt = t0.elapsed();
                let out = decode_extraction(&dag, &ex).unwrap();
                assert_eq!(
                    out.to_string(),
                    ast.to_string(),
                    "[{process}] extraction changed the lowered AST",
                );
                assert_eq!(
                    ex.reachable_node_count(&dag),
                    distinct_ast_subterms(&ast),
                    "[{process}] extracted node count != distinct subterm count",
                );
                eprintln!(
                    "[extract] {process:38} nodes={:5} ({} classes) dag_cost={:9.0} extract={:?}",
                    ex.reachable_node_count(&dag),
                    dag.classes.len(),
                    ex.root_cost,
                    dt,
                );
            }
        }
    }

    /// The rule-free identity gate over the fast rewrite-dev processes (2→2 … 2→4):
    /// the DAG extractor must reproduce each lowered amplitude byte-for-byte.
    #[test]
    fn extract_dev_processes_identity() {
        assert_processes_extract_identity(&[
            "e+ e- > mu+ mu-",
            "e+ e- > mu+ mu- a",
            "e+ e- > mu+ mu- ta+ ta- QCD=0",
        ]);
    }

    /// The full validated suite, up to the 2→6 EW amplitudes: every MG-validated
    /// process's lowered AST survives extraction byte-for-byte, with per-process
    /// extractor timing printed for the scaling signal. Gated behind
    /// `extended-validation` (egglog enumeration of the largest ASTs is heavy);
    /// mirrors `representative_processes_roundtrip`.
    ///
    /// ```text
    /// cargo test -p vibegraph-lib --release --features extended-validation \
    ///   extract_validated_processes_identity -- --nocapture
    /// ```
    #[cfg(feature = "extended-validation")]
    #[test]
    fn extract_validated_processes_identity() {
        use super::super::compile::MG_VALIDATED_PROCESSES;
        assert_processes_extract_identity(&MG_VALIDATED_PROCESSES);
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
