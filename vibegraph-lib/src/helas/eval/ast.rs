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

    /// A node's children as the contiguous CSR row, without an iterator adapter —
    /// the forward-pass runtime indexes results directly off this slice.
    pub fn children_ids(&self, node: NodeId) -> &[NodeId] {
        let i = node as usize;
        let lo = self.children_offsets[i] as usize;
        let hi = self.children_offsets[i + 1] as usize;
        &self.children_content[lo..hi]
    }
}

/// The arena exposes its shape through [`Tree`]: `value`/`children`/`root` give a node,
/// its operands, and the whole-amplitude root; `iter` scans every id in storage
/// (topological) order. The default `linearize`/`fold_recursive` then come for free —
/// kept so the forward-scan runtime can be benchmarked against a linearized stack walk.
impl<T> Tree for Ast<T> {
    type Item = Node<T>;
    type NodeId = NodeId;

    fn children(&self, node: NodeId) -> impl Iterator<Item = NodeId> {
        self.children_ids(node).iter().copied()
    }

    fn value(&self, node: NodeId) -> &Node<T> {
        &self.nodes[node as usize]
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

impl<T: fmt::Display> Ast<T> {
    fn render(&self, id: NodeId, out: &mut String) {
        let node = self.value(id);
        out.push('(');
        out.push_str(node.op.name());
        if node.op.has_leaf_token() {
            let leaf = node.leaf.to_string();
            if !leaf.is_empty() {
                out.push(' ');
                out.push_str(&leaf);
            }
        }
        for c in self.children(id) {
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
            let incoming: i32 = toks[3].parse().map_err(|_| bad(&toks[3]))?;
            Sym::Ext {
                leg_idx,
                spin,
                charge: charge_from_sign(sign),
                incoming: incoming != 0,
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

        // Leaf is a nested s-expression (TypeName arg...) when present.
        let leaf = if op.has_leaf_token() {
            match self.next() {
                Some("(") => {}
                Some(other) => return Err(ParseAstError::ExpectedOpen(other.to_string())),
                None => return Err(ParseAstError::UnexpectedEof),
            }
            // consume the type name (CouplingId / ParticleId / Real / ExtLegInfo)
            self.next().ok_or(ParseAstError::UnexpectedEof)?;
            let mut leaf_toks = Vec::new();
            loop {
                match self.peek() {
                    Some(")") => {
                        self.pos += 1;
                        break;
                    }
                    Some(tok) if tok != "(" => {
                        leaf_toks.push(self.next().unwrap().to_string());
                    }
                    _ => return Err(ParseAstError::UnexpectedEof),
                }
            }
            parse_sym_leaf(op, &leaf_toks)?
        } else {
            Sym::None
        };

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helas::eval::op::Const;
    use crate::helas::repr::numbers::Charge;
    use crate::ufo::couplings::CouplingId;
    use crate::ufo::particles::ParticleId;

    fn leaf_ast(op: Op, leaf: Sym) -> Ast<Sym> {
        let mut b = AstBuilder::new();
        let id = b.add(op, leaf, vec![]);
        b.finish(id)
    }

    fn check_roundtrip(ast: &Ast<Sym>) {
        let s = ast.to_string();
        let reparsed: Ast<Sym> = s
            .parse()
            .unwrap_or_else(|e| panic!("parse failed: {e}\ninput: {s}"));
        assert_eq!(s, reparsed.to_string(), "roundtrip changed the tree");
    }

    // ── leaf variants ─────────────────────────────────────────────────────────

    #[test]
    fn coupling_leaf() {
        let ast = leaf_ast(Op::Coupling, Sym::Coupling(CouplingId::from(42usize)));
        assert_eq!(ast.to_string(), "(Coupling (CouplingId 42))");
        check_roundtrip(&ast);
    }

    #[test]
    fn mass_leaf() {
        let ast = leaf_ast(Op::Mass, Sym::Particle(ParticleId::from(11usize)));
        assert_eq!(ast.to_string(), "(Mass (ParticleId 11))");
        check_roundtrip(&ast);
    }

    #[test]
    fn width_leaf() {
        let ast = leaf_ast(Op::Width, Sym::Particle(ParticleId::from(23usize)));
        assert_eq!(ast.to_string(), "(Width (ParticleId 23))");
        check_roundtrip(&ast);
    }

    #[test]
    fn coeff_leaf() {
        let ast = leaf_ast(Op::Coeff, Sym::Coeff(1.5));
        assert_eq!(ast.to_string(), "(Coeff (Real 1.5))");
        check_roundtrip(&ast);
    }

    #[test]
    fn external_particle_incoming() {
        let ast = leaf_ast(
            Op::External,
            Sym::Ext {
                leg_idx: 0,
                spin: 2,
                charge: Charge::Particle,
                incoming: true,
            },
        );
        assert_eq!(ast.to_string(), "(External (ExtLegInfo 0 2 1 1))");
        check_roundtrip(&ast);
    }

    #[test]
    fn external_antiparticle_outgoing() {
        let ast = leaf_ast(
            Op::External,
            Sym::Ext {
                leg_idx: 3,
                spin: 2,
                charge: Charge::Antiparticle,
                incoming: false,
            },
        );
        assert_eq!(ast.to_string(), "(External (ExtLegInfo 3 2 -1 0))");
        check_roundtrip(&ast);
    }

    // ── Const Display ─────────────────────────────────────────────────────────
    // Ast<Const> has no FromStr; verify the Display strings match the egglog schema.

    #[test]
    fn const_display_complex() {
        assert_eq!(Const::Complex(7).to_string(), "(Complex 7)");
    }

    #[test]
    fn const_display_real() {
        assert_eq!(Const::Real(3).to_string(), "(Real 3)");
    }

    #[test]
    fn const_display_ext() {
        let c = Const::Ext {
            leg_idx: 1,
            spin: 3,
            charge: Charge::Antiparticle,
            incoming: true,
        };
        assert_eq!(c.to_string(), "(ExtLegInfo 1 3 -1 1)");
    }

    #[test]
    fn const_display_none() {
        assert_eq!(Const::None.to_string(), "(None)");
    }

    // ── structural trees ──────────────────────────────────────────────────────

    #[test]
    fn projm_wrapping_coupling() {
        let mut b = AstBuilder::new();
        let coup = b.add(
            Op::Coupling,
            Sym::Coupling(CouplingId::from(5usize)),
            vec![],
        );
        let root = b.add(Op::ProjM, Sym::None, vec![coup]);
        let ast = b.finish(root);
        assert_eq!(ast.to_string(), "(ProjM (Coupling (CouplingId 5)))");
        check_roundtrip(&ast);
    }

    #[test]
    fn add_two_coeffs() {
        let mut b = AstBuilder::new();
        let c1 = b.add(Op::Coeff, Sym::Coeff(1.0), vec![]);
        let c2 = b.add(Op::Coeff, Sym::Coeff(2.0), vec![]);
        let root = b.add(Op::Add, Sym::None, vec![c1, c2]);
        let ast = b.finish(root);
        check_roundtrip(&ast);
        let s = ast.to_string();
        assert!(s.starts_with("(Add "), "unexpected: {s}");
        assert!(s.contains("(Coeff (Real"), "unexpected: {s}");
    }

    #[test]
    fn external_with_mass_child() {
        // A realistic External node: leaf + one Mass child (as lowered in practice).
        let mut b = AstBuilder::new();
        let mass = b.add(Op::Mass, Sym::Particle(ParticleId::from(11usize)), vec![]);
        let ext = b.add(
            Op::External,
            Sym::Ext {
                leg_idx: 0,
                spin: 2,
                charge: Charge::Particle,
                incoming: true,
            },
            vec![mass],
        );
        let ast = b.finish(ext);
        assert_eq!(
            ast.to_string(),
            "(External (ExtLegInfo 0 2 1 1) (Mass (ParticleId 11)))"
        );
        check_roundtrip(&ast);
    }

    #[test]
    fn mul_coupling_coeff_external() {
        let mut b = AstBuilder::new();
        let coup = b.add(
            Op::Coupling,
            Sym::Coupling(CouplingId::from(5usize)),
            vec![],
        );
        let coeff = b.add(Op::Coeff, Sym::Coeff(1.5), vec![]);
        let mass = b.add(Op::Mass, Sym::Particle(ParticleId::from(11usize)), vec![]);
        let ext = b.add(
            Op::External,
            Sym::Ext {
                leg_idx: 0,
                spin: 2,
                charge: Charge::Particle,
                incoming: true,
            },
            vec![mass],
        );
        let root = b.add(Op::Mul, Sym::None, vec![coup, coeff, ext]);
        let ast = b.finish(root);
        assert_eq!(
            ast.to_string(),
            "(Mul (Coupling (CouplingId 5)) (Coeff (Real 1.5)) (External (ExtLegInfo 0 2 1 1) (Mass (ParticleId 11))))"
        );
        check_roundtrip(&ast);
    }

    // ── error cases ───────────────────────────────────────────────────────────

    #[test]
    fn error_unexpected_eof() {
        assert!(matches!(
            "(Add".parse::<Ast<Sym>>(),
            Err(ParseAstError::UnexpectedEof)
        ));
    }

    #[test]
    fn error_expected_open() {
        assert!(matches!(
            "Add".parse::<Ast<Sym>>(),
            Err(ParseAstError::ExpectedOpen(_))
        ));
    }

    #[test]
    fn error_unknown_op() {
        assert!(matches!(
            "(Frobnicate)".parse::<Ast<Sym>>(),
            Err(ParseAstError::UnknownOp(_))
        ));
    }

    #[test]
    fn error_trailing_tokens() {
        assert!(matches!(
            "(Add) extra".parse::<Ast<Sym>>(),
            Err(ParseAstError::Trailing)
        ));
    }

    #[test]
    fn error_bad_leaf_not_integer() {
        let r = "(Coupling (CouplingId notanumber))".parse::<Ast<Sym>>();
        assert!(matches!(r, Err(ParseAstError::BadLeaf(_))));
    }
}
