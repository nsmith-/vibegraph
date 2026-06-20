//! The unified evaluation AST: an arena of [`Node<T>`] with children in a ragged
//! (CSR) table, generic over the leaf payload `T`.
//!
//! Built bottom-up so a node's children always have smaller ids than the node itself
//! — the runtime ([`super::run`]) relies on that to evaluate the arena in one forward
//! pass, computing each (possibly shared) node exactly once.
//!
//! `Display` renders the tree as an s-expression `(Op leaf? child...)`; `FromStr`
//! parses it back into an `Ast<Sym>`. This is the boundary for the future egglog
//! optimization passes.

use std::fmt;
use std::str::FromStr;

use super::op::{charge_from_sign, Node, NodeId, Op, Sym};
use super::tree::Tree;
use crate::ufo::couplings::CouplingId;
use crate::ufo::particles::ParticleId;

/// A unified evaluation AST over leaf payload `T`.
#[derive(Clone, Debug)]
pub struct Ast<T> {
    nodes: Box<[Node<T>]>,
    /// CSR offsets, length `nodes.len() + 1`: node `i`'s children are
    /// `children_content[children_offsets[i]..children_offsets[i + 1]]`.
    children_offsets: Box<[u32]>,
    children_content: Box<[NodeId]>,
    root: NodeId,
}

impl<T> Ast<T> {
    /// Number of nodes in the arena.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// The root node id (evaluates to the whole-amplitude result).
    pub fn root_id(&self) -> NodeId {
        self.root
    }

    /// All nodes, in arena (topological) order.
    pub fn nodes(&self) -> &[Node<T>] {
        &self.nodes
    }

    /// The node at `id`.
    pub fn node(&self, id: NodeId) -> &Node<T> {
        &self.nodes[id as usize]
    }

    /// The children of `id`, in operand order.
    pub fn child_ids(&self, id: NodeId) -> &[NodeId] {
        let i = id as usize;
        let lo = self.children_offsets[i] as usize;
        let hi = self.children_offsets[i + 1] as usize;
        &self.children_content[lo..hi]
    }
}

impl<T> Tree for Ast<T> {
    type Item = Node<T>;
    type NodeId = NodeId;

    fn children(&self, node: NodeId) -> impl Iterator<Item = NodeId> {
        self.child_ids(node).iter().copied()
    }

    fn value(&self, node: NodeId) -> &Node<T> {
        self.node(node)
    }

    fn root(&self) -> NodeId {
        self.root
    }

    fn iter(&self) -> impl Iterator<Item = NodeId> {
        0..self.nodes.len() as NodeId
    }
}

/// Incremental builder. Add children before their parent so the finished arena keeps
/// the children-before-parents invariant.
#[derive(Debug)]
pub struct AstBuilder<T> {
    nodes: Vec<Node<T>>,
    children: Vec<Vec<NodeId>>,
}

impl<T> Default for AstBuilder<T> {
    fn default() -> Self {
        AstBuilder {
            nodes: Vec::new(),
            children: Vec::new(),
        }
    }
}

impl<T> AstBuilder<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a node and return its id. `children` must already be in the builder.
    pub fn add(&mut self, op: Op, leaf: T, children: Vec<NodeId>) -> NodeId {
        let id = self.nodes.len() as NodeId;
        self.nodes.push(Node::new(op, leaf));
        self.children.push(children);
        id
    }

    /// Finalize into a CSR-backed [`Ast`] rooted at `root`.
    pub fn finish(self, root: NodeId) -> Ast<T> {
        let mut offsets = Vec::with_capacity(self.nodes.len() + 1);
        let mut content = Vec::new();
        offsets.push(0u32);
        for kids in &self.children {
            content.extend_from_slice(kids);
            offsets.push(content.len() as u32);
        }
        Ast {
            nodes: self.nodes.into_boxed_slice(),
            children_offsets: offsets.into_boxed_slice(),
            children_content: content.into_boxed_slice(),
            root,
        }
    }
}

// ───────────────────────────── s-expression I/O ─────────────────────────────

/// Number of leaf payload tokens an op carries in the s-expression.
fn leaf_token_count(op: Op) -> usize {
    match op {
        Op::External => 3,                                    // leg_idx spin charge_sign
        Op::Coupling | Op::Mass | Op::Width | Op::Coeff => 1, // id / coeff
        _ => 0,
    }
}

impl<T: fmt::Display> Ast<T> {
    fn render(&self, id: NodeId, out: &mut String) {
        let node = self.node(id);
        out.push('(');
        out.push_str(node.op.name());
        if node.op.has_leaf_token() {
            let leaf = node.leaf.to_string();
            if !leaf.is_empty() {
                out.push(' ');
                out.push_str(&leaf);
            }
        }
        for &c in self.child_ids(id) {
            out.push(' ');
            self.render(c, out);
        }
        out.push(')');
    }
}

impl<T: fmt::Display> fmt::Display for Ast<T> {
    /// Render as an s-expression. A shared (DAG) subtree is expanded once per parent.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        self.render(self.root, &mut s);
        f.write_str(&s)
    }
}

/// Error parsing an s-expression into an [`Ast<Sym>`].
#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseAstError {
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("expected `(`, found `{0}`")]
    ExpectedOpen(String),
    #[error("unknown op `{0}`")]
    UnknownOp(String),
    #[error("bad leaf token `{0}`")]
    BadLeaf(String),
    #[error("trailing tokens after root")]
    Trailing,
}

fn tokenize(s: &str) -> Vec<String> {
    s.replace('(', " ( ")
        .replace(')', " ) ")
        .split_whitespace()
        .map(|t| t.to_string())
        .collect()
}

fn parse_sym_leaf(op: Op, toks: &[String]) -> Result<Sym, ParseAstError> {
    let bad = |t: &str| ParseAstError::BadLeaf(t.to_string());
    Ok(match op {
        Op::Coupling => {
            let v: usize = toks[0].parse().map_err(|_| bad(&toks[0]))?;
            Sym::Coupling(CouplingId::from(v))
        }
        Op::Mass | Op::Width => {
            let v: usize = toks[0].parse().map_err(|_| bad(&toks[0]))?;
            Sym::Particle(ParticleId::from(v))
        }
        Op::Coeff => {
            let v: f64 = toks[0].parse().map_err(|_| bad(&toks[0]))?;
            Sym::Coeff(v)
        }
        Op::External => {
            let leg_idx: usize = toks[0].parse().map_err(|_| bad(&toks[0]))?;
            let spin: i32 = toks[1].parse().map_err(|_| bad(&toks[1]))?;
            let sign: i32 = toks[2].parse().map_err(|_| bad(&toks[2]))?;
            Sym::Ext {
                leg_idx,
                spin,
                charge: charge_from_sign(sign),
            }
        }
        _ => Sym::None,
    })
}

struct Parser<'a> {
    toks: &'a [String],
    pos: usize,
    builder: AstBuilder<Sym>,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&str> {
        self.toks.get(self.pos).map(|s| s.as_str())
    }

    fn next(&mut self) -> Option<&str> {
        let t = self.toks.get(self.pos).map(|s| s.as_str());
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn parse_node(&mut self) -> Result<NodeId, ParseAstError> {
        match self.next() {
            Some("(") => {}
            Some(other) => return Err(ParseAstError::ExpectedOpen(other.to_string())),
            None => return Err(ParseAstError::UnexpectedEof),
        }
        let head = self.next().ok_or(ParseAstError::UnexpectedEof)?.to_string();
        let op = Op::from_name(&head).ok_or_else(|| ParseAstError::UnknownOp(head.clone()))?;

        let n_leaf = leaf_token_count(op);
        let mut leaf_toks = Vec::with_capacity(n_leaf);
        for _ in 0..n_leaf {
            leaf_toks.push(self.next().ok_or(ParseAstError::UnexpectedEof)?.to_string());
        }
        let leaf = parse_sym_leaf(op, &leaf_toks)?;

        let mut children = Vec::new();
        loop {
            match self.peek() {
                Some(")") => {
                    self.pos += 1;
                    break;
                }
                Some("(") => children.push(self.parse_node()?),
                Some(other) => return Err(ParseAstError::BadLeaf(other.to_string())),
                None => return Err(ParseAstError::UnexpectedEof),
            }
        }
        Ok(self.builder.add(op, leaf, children))
    }
}

impl FromStr for Ast<Sym> {
    type Err = ParseAstError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let toks = tokenize(s);
        let mut p = Parser {
            toks: &toks,
            pos: 0,
            builder: AstBuilder::new(),
        };
        let root = p.parse_node()?;
        if p.pos != toks.len() {
            return Err(ParseAstError::Trailing);
        }
        Ok(p.builder.finish(root))
    }
}
