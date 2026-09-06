//! Vertex dispatch: resolve LorentzTerm into rooted contraction trees at compile time.
//!
//! Each LorentzTerm is a tensor network, we root it with an output fixed by `result_leg_idx`.
//! The rooted descriptor (RootedNode) specifies the concrete primitive and its input/output
//! orientation so that eval walks resolved nodes

use crate::{
    helas::eval::tree::Tree,
    ufo::lorentz::{LorentzOp, LorentzTerm},
};

/// Fermion-number (spinor) adjoint direction of a wavefunction, resolved structurally
/// during the bake step's first (topology) pass.
///
/// `Ket` (`u`/`v` column) roots the HELAS `ixxxxx`; `Bra` (`ū`/`v̄` row) roots `oxxxxx`. The
/// adjoint is constant along a fermion line, so an off-shell current and the propagator on
/// it inherit the adjoint of their continuing fermion input. Bosonic and scalar-amplitude
/// nodes carry no adjoint. The pair (bra, ket) meeting at a vertex always have opposite
/// adjoint; the runtime `resolve_bra_ket` reads the same distinction off the evaluated slot
/// variants, so baking it makes the line direction explicit and lets the rooting choose
/// the correct in/out fermion routine ([`LorentzEvalTree::build_at_leg`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Adjoint {
    /// Ket (`u`/`v` column); external via `ixxxxx`.
    Ket,
    /// Bra (`ū`/`v̄` row); external via `oxxxxx`.
    Bra,
}

impl std::fmt::Display for Adjoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Adjoint::Ket => "ket",
            Adjoint::Bra => "bra",
        })
    }
}

/// Per-leg spinor binding as seen by the Lorentz rooting: the baked [`Adjoint`] plus
/// whether the leg sits on a *crossed* fermion line.
///
/// Diagram enumeration presents outgoing legs in the all-incoming convention, so a
/// final-state fermion is bound at the UFO slot of its antiparticle with the
/// conjugate wavefunction type (the outgoing μ⁺ is a *bra* at the `mu-` slot). The
/// crossing inverts slot identity and adjoint together, so adjoint-vs-slot inspection
/// cannot see it — the bit must be carried explicitly (external legs: `!incoming`;
/// off-shell currents inherit it from their continuing fermion input). A crossed
/// pair evaluates `ū₁Γv₂` where the vertex is defined as `ū₂Γv₁`; by
/// `ū₁Γv₂ = −ū₂(CΓᵀC⁻¹)v₁` this is exact for vector structures and requires
/// conjugating `P_χ → P_χ̄` (no sign) for gamma-chained chiral projectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LegAdjoint {
    /// Ket/bra adjoint of the bound wavefunction.
    pub adjoint: Adjoint,
    /// True iff the leg's fermion line is crossed (its externals are final-state).
    pub crossed: bool,
}

/// A single LorentzTerm, already rooted at the output leg and ready to eval.
#[derive(Clone, Debug)]
pub struct RootedTerm {
    /// Coefficient carried from the UFO LorentzTerm.coeff (real, since couplings are stored separately).
    pub coeff: f64,
    /// The ±1 rooting-convention sign this term picked up from [`LorentzEvalTree::build_at_leg`]
    /// (VVS `pure_metric`, FFS scalar-sink, crossed-pair). It is **not** folded into `coeff`,
    /// because it depends on the output-leg (rooting) choice; the honest tensor `tree` is
    /// rooting-invariant. All terms of a vertex share this sign, so it is lifted to a
    /// per-diagram scalar computed from the *canonical* `VtxIdx(0)` rooting
    /// ([`DiagramEvalTree::build_convention_sign`]) and carried in the diagram's `fermi_sign`.
    pub build_sign: i8,
    /// The ±1 runtime `reversed`-bilinear parity this term's fermion→vector sink
    /// contributes (see [`term_reversed_parity`]). Like `build_sign` it depends on the
    /// rooting and is common to a vertex's terms, so it is lifted to a per-diagram scalar
    /// at the canonical rooting ([`DiagramEvalTree::reversed_convention_sign`]).
    pub reversed_sign: i8,
    /// Resolved primitive with output fiber fixed.
    pub tree: LorentzEvalTree,
}

impl std::fmt::Display for RootedTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}*{}", self.coeff, self.tree)
    }
}

/// Errors from rooting a vertex's Lorentz structure into a contraction tree.
#[derive(Clone, Debug, thiserror::Error)]
pub enum RootLorentzError {
    /// The vertex structure contains unsupported ops (Sigma, Epsilon, C) or too many free indices.
    #[error("unsupported vertex: {0}")]
    UnsupportedVertex(String),
    /// The structure is syntactically invalid (e.g., mismatched indices).
    #[error("invalid structure: {0}")]
    InvalidStructure(String),
    /// The spin index does not match the expected spinor adjoint.
    #[error("spinor adjoint mismatch: {0}")]
    MissingAdjoint(String),
}

/// Descriptor for one term in the vertex lorentz structure, with dispatch info and resolved rooted node.
///
/// Index labels follow the convention:
/// - a,b for scalars
/// - i,j for fermions (i=row, j=col)
/// - mu, nu, ... for vectors
#[derive(Clone, Debug, PartialEq)]
pub enum LorentzEvalNode {
    /// Leg index (0-indexed)
    Leg(usize),
    /// 2-fermion in, vector out
    GammaVout { i: usize, j: usize },
    /// vector+in-flowing fermion in, in-flowing fermion out
    GammaIout { mu: usize, j: usize },
    /// vector+out-flowing fermion in, out-flowing fermion out
    GammaOout { mu: usize, i: usize },
    /// fermion left projection
    ProjM { i: usize },
    /// fermion right projection
    ProjP { i: usize },
    /// Left chiral amplitude
    ProjMAmp { i: usize, j: usize },
    /// Right chiral amplitude
    ProjPAmp { i: usize, j: usize },
    /// contract two vector indices → scalar (`g_{μν} V^μ W^ν`)
    Metric { mu: usize, nu: usize },
    /// metric with one free index → vector: the off-shell vector current of a
    /// `Metric(out, v)` structure (e.g. the VVS/HVV vertex rooted at a vector
    /// leg), the physical contravariant `g^{μν}V_ν = V^μ` on the partner vector
    /// `v` (an identity on contravariant storage); cf. ALOHA `VVS1P1N_1`, whose
    /// −i lives in vibegraph's vector propagator instead. Output type: vector.
    MetricVout { v: usize },
    /// Handle the implicit product over the disconnected structures.
    /// At most one child can be non-scalar (which then implies the output type)
    Mul { children: Vec<usize> },
    /// 4-momentum of leg `leg` (0-indexed) as a vector at a free Lorentz index
    P { leg: usize },
    /// 4-momentum of the *output* leg as a vector: −Σ (input momenta). Emitted by
    /// the leg-compaction pass in [`build_at_leg`] when a `P` references the leg
    /// the tree is rooted at (which has no input current to read a momentum from).
    POut,
    /// Full scalar bilinear ψ̄_i δ ψ_j (Identity amplitude contraction)
    IdentityAmp { i: usize, j: usize },
    /// γ⁵ on a continuing fermion current.
    Gamma5 { i: usize },
    /// Pseudoscalar bilinear ψ̄_i γ⁵ ψ_j.
    Gamma5Amp { i: usize, j: usize },
    /// Three vectors → off-shell vector: `ε^{μνρσ} a_μ b_ν c_ρ`, free index last.
    /// The children are the `Epsilon` arguments in source order with the output
    /// slot removed; the sign of moving that slot to the end is absorbed by
    /// [`epsilon_out_order`], which swaps two of them when it is negative.
    EpsilonVout { a: usize, b: usize, c: usize },
    /// Four vectors → scalar: `ε^{μνρσ} a_μ b_ν c_ρ d_σ`, children in argument order.
    EpsilonAmp {
        a: usize,
        b: usize,
        c: usize,
        d: usize,
    },
    /// Two fermions → the cut line's `γ^αγ^β` chain as a Clifford element, in the
    /// index order the other line reads it (see [`Op::FierzOut`](super::op::Op::FierzOut)).
    /// Children are the line's two ends in vertex slot order (row first).
    FierzOut { i: usize, j: usize },
    /// [`FierzOut`](Self::FierzOut) with the two lines traversing the shared indices
    /// in opposite orders — the grade-2 coefficient enters with the other sign.
    FierzOutRev { i: usize, j: usize },
    /// Clifford element + flow-in fermion → flow-in fermion current `M ψ`.
    MultivectorIout { m: usize, j: usize },
    /// Clifford element + flow-out fermion → flow-out fermion current `ψ̄ M`.
    MultivectorOout { m: usize, i: usize },
    /// Clifford element + two fermions → the scalar `ψ̄ M ψ`, the grade-diagonal
    /// Fierz pairing of the element with the pair's own sixteen bilinears.
    FierzPair { m: usize, i: usize, j: usize },
    // TODO: Sigma
}

impl LorentzEvalNode {
    pub fn children(&self) -> Vec<usize> {
        match self {
            LorentzEvalNode::Leg(_) => vec![],
            LorentzEvalNode::Gamma5 { i } => vec![*i],
            LorentzEvalNode::Gamma5Amp { i, j } => vec![*i, *j],
            LorentzEvalNode::EpsilonVout { a, b, c } => vec![*a, *b, *c],
            LorentzEvalNode::EpsilonAmp { a, b, c, d } => vec![*a, *b, *c, *d],
            LorentzEvalNode::GammaVout { i, j } => vec![*i, *j],
            LorentzEvalNode::GammaIout { mu, j } => vec![*mu, *j],
            LorentzEvalNode::GammaOout { mu, i } => vec![*mu, *i],
            LorentzEvalNode::ProjM { i } | LorentzEvalNode::ProjP { i } => vec![*i],
            LorentzEvalNode::ProjMAmp { i, j } | LorentzEvalNode::ProjPAmp { i, j } => vec![*i, *j],
            LorentzEvalNode::Metric { mu, nu } => vec![*mu, *nu],
            LorentzEvalNode::MetricVout { v } => vec![*v],
            LorentzEvalNode::Mul { children } => children.clone(),
            LorentzEvalNode::P { .. } => vec![],
            LorentzEvalNode::POut => vec![],
            LorentzEvalNode::IdentityAmp { i, j } => vec![*i, *j],
            LorentzEvalNode::FierzOut { i, j } | LorentzEvalNode::FierzOutRev { i, j } => {
                vec![*i, *j]
            }
            LorentzEvalNode::MultivectorIout { m, j } => vec![*m, *j],
            LorentzEvalNode::MultivectorOout { m, i } => vec![*m, *i],
            LorentzEvalNode::FierzPair { m, i, j } => vec![*m, *i, *j],
        }
    }

    fn render(&self, body: String) -> String {
        use LorentzEvalNode::*;
        match self {
            Leg(i) => format!("Leg({})", i), // leaf node
            GammaVout { .. } => format!("GammaVout({})", body),
            GammaIout { .. } => format!("GammaIout({})", body),
            GammaOout { .. } => format!("GammaOout({})", body),
            ProjM { .. } => format!("ProjM({})", body),
            ProjP { .. } => format!("ProjP({})", body),
            ProjMAmp { .. } => format!("ProjMAmp({})", body),
            ProjPAmp { .. } => format!("ProjPAmp({})", body),
            Metric { .. } => format!("Metric({})", body),
            MetricVout { .. } => format!("MetricVout({})", body),
            Mul { .. } => format!("ScalarProduct({})", body),
            P { leg } => format!("P{leg}"),
            POut => "POut".to_string(), // leaf node
            IdentityAmp { .. } => format!("IdentityAmp({})", body),
            Gamma5 { .. } => format!("Gamma5({})", body),
            Gamma5Amp { .. } => format!("Gamma5Amp({})", body),
            EpsilonVout { .. } => format!("EpsilonVout({})", body),
            EpsilonAmp { .. } => format!("EpsilonAmp({})", body),
            FierzOut { .. } => format!("FierzOut({})", body),
            FierzOutRev { .. } => format!("FierzOutRev({})", body),
            MultivectorIout { .. } => format!("MultivectorIout({})", body),
            MultivectorOout { .. } => format!("MultivectorOout({})", body),
            FierzPair { .. } => format!("FierzPair({})", body),
        }
    }
}

/// The `Epsilon` argument slots other than the output slot `k`, in source order,
/// already carrying the antisymmetry sign of moving `k` to the last position.
///
/// `ε(x₀,x₁,x₂,x₃)` with the output at slot `k` equals `(−1)^{3−k}` times
/// `ε(remaining…, out)`, which is the form [`LorentzEvalNode::EpsilonVout`]
/// evaluates. A `−1` is absorbed by swapping the first two remaining slots (one
/// transposition), so the node never needs a sign of its own.
fn epsilon_out_order(args: [isize; 4], k: usize) -> [isize; 3] {
    let mut rest = [0isize; 3];
    let mut n = 0;
    for (slot, &idx) in args.iter().enumerate() {
        if slot != k {
            rest[n] = idx;
            n += 1;
        }
    }
    if (3 - k) % 2 == 1 {
        rest.swap(0, 1);
    }
    rest
}

#[derive(Clone, Debug, PartialEq)]
pub struct LorentzEvalTree {
    nodes: Vec<LorentzEvalNode>,
    root: Option<usize>,
}

impl Tree for LorentzEvalTree {
    type Item = LorentzEvalNode;
    type NodeId = usize;

    fn children(&self, node: Self::NodeId) -> impl Iterator<Item = usize> {
        self.value(node).children().into_iter()
    }

    fn value(&self, node: Self::NodeId) -> &Self::Item {
        &self.nodes[node]
    }

    fn root(&self) -> Self::NodeId {
        self.root.expect("LorentzEvalTree has no root node")
    }

    fn iter(&self) -> impl Iterator<Item = Self::NodeId> {
        0..self.nodes.len()
    }
}

/// The vector-output transform for a rooted structure term: the honest
/// contravariant current `+V^μ` ([`LorentzEvalNode::MetricVout`]), for every
/// vector-output structure alike.
///
/// This current is rooting-invariant by construction — a plain tensor contraction
/// with consistent momenta and no added sign. The momentum-odd −1 that the
/// Yang-Mills (VVV) vertex needs relative to it is *not* a property of the rooted
/// current (which would make it depend on the output-leg choice); it is a
/// rooting-invariant per-vertex sign carried at the diagram level by
/// [`super::root_diagram::yang_mills_vvv_sign`], applied once per non-root VVV
/// vertex so `σ_V·(honest current)` matches MadGraph independent of the root.
fn vector_out_node(child: usize) -> LorentzEvalNode {
    LorentzEvalNode::MetricVout { v: child }
}

impl LorentzEvalTree {
    /// The value at the root node. Used by tests to assert the rooted primitive.
    #[cfg(test)]
    pub fn root_value(&self) -> &LorentzEvalNode {
        self.value(self.root())
    }

    fn add_node(&mut self, node: LorentzEvalNode) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        idx
    }

    fn build_child(
        &mut self,
        term: &LorentzTerm,
        idx: isize,
        visited_ops: &mut Vec<usize>,
        flows: &[Option<LegAdjoint>],
        out_adjoint: Option<Adjoint>,
        sign: &mut f64,
    ) -> Result<usize, RootLorentzError> {
        // Find an operator that involves this index and has not been visited
        let Some((iop, op)) = term.ops.iter().enumerate().find(|&(i, op)| {
            (op.involves_spinor(idx) || op.involves_vector(idx))
                && visited_ops.iter().all(|&j| i != j)
        }) else {
            // No operator involves this index
            if idx >= 0 {
                // This is a scalar leaf node
                return Ok(self.add_node(LorentzEvalNode::Leg(idx as usize)));
            } else {
                return Err(RootLorentzError::InvalidStructure(format!(
                    "Free index {} has no operator in term",
                    idx
                )));
            }
        };

        visited_ops.push(iop);

        match op {
            LorentzOp::Gamma { mu, i, j } => {
                if *mu == idx {
                    let child_i =
                        self.build_child(term, *i, visited_ops, flows, out_adjoint, sign)?;
                    let child_j =
                        self.build_child(term, *j, visited_ops, flows, out_adjoint, sign)?;
                    Ok(self.add_node(LorentzEvalNode::GammaVout {
                        i: child_i,
                        j: child_j,
                    }))
                } else if *i == idx || *j == idx {
                    // Fermion output: the continuing line's *physical* adjoint (resolved by
                    // the first pass) chooses the in/out routine — not the UFO `i`/`j`
                    // position, which only fixes the vertex's *defined* adjoint and can run
                    // opposite to the actual line (e.g. an incoming-pair spine). The
                    // input fermion is the gamma's other fermion index.
                    let other = if *i == idx { *j } else { *i };
                    // At the rooted output leg the adjoint is that leg's; at a *summed*
                    // spinor index (a γ-chain) there is no leg to read, so it is that of
                    // the external fermion this gamma's input chain leads to
                    // ([`chain_adjoint`]) — the two agree at the root, because a current
                    // keeps one adjoint along its whole line.
                    let node_adjoint = if idx >= 0 {
                        out_adjoint
                    } else {
                        chain_adjoint(term, other, iop, flows)
                    };
                    let child_mu =
                        self.build_child(term, *mu, visited_ops, flows, out_adjoint, sign)?;
                    let child_f =
                        self.build_child(term, other, visited_ops, flows, out_adjoint, sign)?;
                    let node = match node_adjoint {
                        Some(Adjoint::Ket) => LorentzEvalNode::GammaIout {
                            mu: child_mu,
                            j: child_f,
                        },
                        Some(Adjoint::Bra) => LorentzEvalNode::GammaOout {
                            mu: child_mu,
                            i: child_f,
                        },
                        None => {
                            return Err(RootLorentzError::InvalidStructure(
                                "fermion-output Gamma rooted without a spinor adjoint".to_string(),
                            ))
                        }
                    };
                    Ok(self.add_node(node))
                } else {
                    unreachable!("Gamma op should involve idx {}", idx);
                }
            }
            LorentzOp::ProjM { i, j } => {
                let (wrapped, wrapped_is_row) = if *i == idx {
                    (*j, false)
                } else if *j == idx {
                    (*i, true)
                } else {
                    unreachable!("ProjM op should involve idx {}", idx);
                };
                if standalone_projector_crossed(idx, wrapped, flows) {
                    *sign = -*sign;
                }
                let child =
                    self.build_child(term, wrapped, visited_ops, flows, out_adjoint, sign)?;
                let node = if chiral_correction(idx, wrapped, wrapped_is_row, flows) {
                    LorentzEvalNode::ProjP { i: child }
                } else {
                    LorentzEvalNode::ProjM { i: child }
                };
                Ok(self.add_node(node))
            }
            LorentzOp::ProjP { i, j } => {
                let (wrapped, wrapped_is_row) = if *i == idx {
                    (*j, false)
                } else if *j == idx {
                    (*i, true)
                } else {
                    unreachable!("ProjP op should involve idx {}", idx);
                };
                if standalone_projector_crossed(idx, wrapped, flows) {
                    *sign = -*sign;
                }
                let child =
                    self.build_child(term, wrapped, visited_ops, flows, out_adjoint, sign)?;
                let node = if chiral_correction(idx, wrapped, wrapped_is_row, flows) {
                    LorentzEvalNode::ProjM { i: child }
                } else {
                    LorentzEvalNode::ProjP { i: child }
                };
                Ok(self.add_node(node))
            }
            LorentzOp::Metric { mu, nu } => {
                let other = if *mu == idx {
                    *nu
                } else if *nu == idx {
                    *mu
                } else {
                    unreachable!("Metric op should involve idx {}", idx);
                };
                let child = self.build_child(term, other, visited_ops, flows, out_adjoint, sign)?;
                Ok(self.add_node(vector_out_node(child)))
            }
            LorentzOp::P { mu, leg } => {
                // mu == idx (guaranteed by involves_vector fix); leg is the momentum source particle
                assert_eq!(*mu, idx, "Momentum index mismatch");
                Ok(self.add_node(LorentzEvalNode::P { leg: *leg as usize }))
            }
            LorentzOp::Identity { i, j } => {
                if *i == idx {
                    self.build_child(term, *j, visited_ops, flows, out_adjoint, sign)
                } else if *j == idx {
                    self.build_child(term, *i, visited_ops, flows, out_adjoint, sign)
                } else {
                    unreachable!("Identity op should involve idx {}", idx);
                }
            }
            LorentzOp::Gamma5 { i, j } => {
                let wrapped = if *i == idx {
                    *j
                } else if *j == idx {
                    *i
                } else {
                    unreachable!("Gamma5 op should involve idx {}", idx);
                };
                // `C γ⁵ᵀ C⁻¹ = γ⁵`, so a standalone γ⁵ over a crossed pair keeps the
                // −1 the reversed reading gives it, exactly as a standalone chiral
                // projector does (see [`standalone_projector_crossed`]); and, for the
                // same reason, a γ⁵ reached through a summed index needs no chirality
                // conjugation of its own.
                if standalone_projector_crossed(idx, wrapped, flows) {
                    *sign = -*sign;
                }
                let child =
                    self.build_child(term, wrapped, visited_ops, flows, out_adjoint, sign)?;
                Ok(self.add_node(LorentzEvalNode::Gamma5 { i: child }))
            }
            LorentzOp::Sigma { .. } => Err(RootLorentzError::UnsupportedVertex(
                "Sigma tensors are deferred to future work".to_string(),
            )),
            LorentzOp::Epsilon { mu, nu, rho, sigma } => {
                let args = [*mu, *nu, *rho, *sigma];
                let Some(slot) = args.iter().position(|&a| a == idx) else {
                    unreachable!("Epsilon op should involve idx {}", idx);
                };
                let rest = epsilon_out_order(args, slot);
                let a = self.build_child(term, rest[0], visited_ops, flows, out_adjoint, sign)?;
                let b = self.build_child(term, rest[1], visited_ops, flows, out_adjoint, sign)?;
                let c = self.build_child(term, rest[2], visited_ops, flows, out_adjoint, sign)?;
                Ok(self.add_node(LorentzEvalNode::EpsilonVout { a, b, c }))
            }
            LorentzOp::C { .. } => Err(RootLorentzError::UnsupportedVertex(
                "Charge conjugation is deferred to future work".to_string(),
            )),
        }
    }

    /// Build a LorentzEvalTree rooted at the specified leg index.
    ///
    /// This turns the undirected tensor network of the LorentzTerm into a
    /// directed tree with a known output. If the leg index is None, the
    /// tree is rooted at an amplitude (scalar sink) and an arbitrary leg
    /// is chosen for routing.
    ///
    /// `out_adjoint` is the spinor adjoint of the output leg (`Some` iff the output is a
    /// fermion), used to pick the in/out gamma routine. The disconnected scalar
    /// structures collected for an amplitude/scalar sink contract to scalars, so they
    /// root through vectors and never consult the adjoint.
    ///
    /// Returns the tree together with a ±1 sign for the enclosing term coefficient:
    /// a standalone scalar bilinear (`ψ̄ Γ ψ`, Γ gamma-less) over a *crossed* pair
    /// evaluates `ū₁Γv₂` for a vertex defined as `ū₂Γv₁ = −ū₁(CΓᵀC⁻¹)v₂`, and with
    /// `CΓᵀC⁻¹ = Γ` for `1`/`P_χ` the −1 survives (unlike the gamma-chained case,
    /// where `Cγ^{μT}C⁻¹ = −γ^μ` cancels it — see [`chiral_correction`]).
    ///
    /// Note: idx is 0-indexed
    pub fn build_at_leg(
        term: &LorentzTerm,
        spins: &[i32],
        flow: &[(usize, usize)],
        idx: Option<usize>,
        flows: &[Option<LegAdjoint>],
    ) -> Result<(Self, f64, i8), RootLorentzError> {
        let out_adjoint = idx.and_then(|i| flows.get(i).copied().flatten().map(|lf| lf.adjoint));
        // A cyclic index graph is not rootable as a tree; the tensor path cuts the
        // cycle at one fermion line instead (its gammas never become a `GammaVout`,
        // so the term carries no reversed-bilinear parity of its own).
        if let Some(tensor) = cyclic_tensor_term(term, spins)? {
            let (tree, sign) =
                LorentzEvalTree::build_tensor_term(term, &tensor, idx, out_adjoint, flows)?;
            return Ok((tree.compact_legs(idx), sign, 1));
        }
        let idx = correct_spin_index_for_flow(spins, flow, idx, out_adjoint)?;
        let reversed_parity = term_reversed_parity(term, idx, flows);
        let mut tree = LorentzEvalTree {
            nodes: vec![],
            root: None,
        };
        let mut sign = 1.0;
        let mut visited_ops = Vec::new(); // LorentzOp is so small that Vec is probably better than HashSet
        let mut term_roots = Vec::new();
        // Whether this term's once-per-vertex −1 has been applied; guards against
        // double application for structures with several Metric ops (VVVV).
        let mut metric_vertex_applied = false;

        // An all-vector contact of four or more legs carries the −1 as a property of
        // the *vertex*, not of the operators a particular term happens to contain: the
        // four-gluon vertex and the field-strength operators sharing its legs sit in
        // one interaction whose structures range over pure metrics, momentum products
        // and Levi-Civita tensors, and a term-by-term test would give the same vertex
        // different signs (and leave the Levi-Civita-only terms, which carry no Metric
        // at all, unsigned). Applying it here also covers those terms.
        if spins.len() >= 4 && spins.iter().all(|&s| s == 3) {
            metric_vertex_applied = true;
            sign = -sign;
        }

        // If idx is specified, build the tree rooted at that leg
        if let Some(root_leg) = idx {
            let node_idx = tree.build_child(
                term,
                root_leg as isize,
                &mut visited_ops,
                flows,
                out_adjoint,
                &mut sign,
            )?;
            // If no operator connects to this leg, build_child returns a trivial Leg leaf.
            // Pop it — the disconnected structures collected below are the actual output.
            let is_trivial_leaf =
                matches!(tree.nodes[node_idx], LorentzEvalNode::Leg(i) if i == root_leg);
            if is_trivial_leaf {
                tree.nodes.pop();
            } else if matches!(tree.nodes[node_idx], LorentzEvalNode::P { .. }) {
                // A P carrying the output Lorentz index (e.g. the VVV1 terms
                // P(1,2)·Metric(2,3) rooted at leg 1) emits a bare momentum vector.
                // Wrap it in the term's vector-output transform so the mixed
                // Metric-/P-rooted terms of the structure stay coherent.
                let wrapped = tree.add_node(vector_out_node(node_idx));
                term_roots.push(wrapped);
            } else {
                term_roots.push(node_idx);
            }
        }

        // find all the remaining scalar roots
        while visited_ops.len() < term.ops.len() {
            // find an unvisited op
            let Some((iop, op)) = term
                .ops
                .iter()
                .enumerate()
                .find(|&(i, _)| visited_ops.iter().all(|&j| i != j))
            else {
                break; // no more unvisited ops
            };
            // pick a leg index from this op that we know how to contract and return a scalar
            let term = match op {
                LorentzOp::Gamma { mu, .. } => {
                    // route through a vector leg, which can always be contracted with a metric to return a scalar
                    // two-pass: one for the gamma and one for the leg
                    let v_in =
                        tree.build_child(term, *mu, &mut visited_ops, flows, None, &mut sign)?;
                    let v_out =
                        tree.build_child(term, *mu, &mut visited_ops, flows, None, &mut sign)?;
                    tree.add_node(LorentzEvalNode::Metric {
                        mu: v_in,
                        nu: v_out,
                    })
                }
                LorentzOp::ProjM { i, j } => {
                    visited_ops.push(iop);
                    // Scalar-sink bilinear (amplitude or scalar-out current): −1
                    // against the −i/D scalar propagator (see `propagate_core`),
                    // on top of the crossed-pair −1.
                    sign = -sign;
                    if pair_crossed(*i, *j, flows) {
                        sign = -sign;
                    }
                    let child_i =
                        tree.build_child(term, *i, &mut visited_ops, flows, None, &mut sign)?;
                    let child_j =
                        tree.build_child(term, *j, &mut visited_ops, flows, None, &mut sign)?;
                    tree.add_node(LorentzEvalNode::ProjMAmp {
                        i: child_i,
                        j: child_j,
                    })
                }
                LorentzOp::ProjP { i, j } => {
                    visited_ops.push(iop);
                    // Scalar-sink bilinear: −1, as in the ProjM arm above.
                    sign = -sign;
                    if pair_crossed(*i, *j, flows) {
                        sign = -sign;
                    }
                    let child_i =
                        tree.build_child(term, *i, &mut visited_ops, flows, None, &mut sign)?;
                    let child_j =
                        tree.build_child(term, *j, &mut visited_ops, flows, None, &mut sign)?;
                    tree.add_node(LorentzEvalNode::ProjPAmp {
                        i: child_i,
                        j: child_j,
                    })
                }
                LorentzOp::Metric { mu, nu } => {
                    visited_ops.push(iop);
                    let child_mu =
                        tree.build_child(term, *mu, &mut visited_ops, flows, None, &mut sign)?;
                    let child_nu =
                        tree.build_child(term, *nu, &mut visited_ops, flows, None, &mut sign)?;
                    // A pure-metric boson structure (VVS/VVSS) carries an explicit
                    // −1 vertex factor, once per term (Gamma-/P-carrying structures —
                    // FFV, VVV — contract plainly). The sign holds whether the
                    // contraction sinks into the *amplitude* or into a scalar output
                    // leg (the H-current from two Z chains, −1 against the −i/D scalar
                    // propagator): both are pinned per-diagram against MadGraph
                    // AMP() — gg→gg for the amplitude root, the uux 2→6 and b b̄ 2→6
                    // H classes for the output-leg root. The all-vector contact takes
                    // the same −1 from the vertex-level test above, whatever its
                    // structure contains.
                    let pure_metric = !term
                        .ops
                        .iter()
                        .any(|op| matches!(op, LorentzOp::P { .. } | LorentzOp::Gamma { .. }));
                    if pure_metric && !metric_vertex_applied {
                        metric_vertex_applied = true;
                        sign = -sign;
                    }
                    tree.add_node(LorentzEvalNode::Metric {
                        mu: child_mu,
                        nu: child_nu,
                    })
                }
                LorentzOp::P { mu, .. } => {
                    // p^μ contracted with the vector leg at the same index
                    let p_node =
                        tree.build_child(term, *mu, &mut visited_ops, flows, None, &mut sign)?;
                    let leg_node =
                        tree.build_child(term, *mu, &mut visited_ops, flows, None, &mut sign)?;
                    tree.add_node(LorentzEvalNode::Metric {
                        mu: p_node,
                        nu: leg_node,
                    })
                }
                LorentzOp::Identity { i, j } => {
                    visited_ops.push(iop);
                    // Scalar-sink bilinear: −1, as in the ProjM arm above.
                    sign = -sign;
                    if pair_crossed(*i, *j, flows) {
                        sign = -sign;
                    }
                    let child_i =
                        tree.build_child(term, *i, &mut visited_ops, flows, None, &mut sign)?;
                    let child_j =
                        tree.build_child(term, *j, &mut visited_ops, flows, None, &mut sign)?;
                    tree.add_node(LorentzEvalNode::IdentityAmp {
                        i: child_i,
                        j: child_j,
                    })
                }
                LorentzOp::Gamma5 { i, j } => {
                    visited_ops.push(iop);
                    // Scalar-sink bilinear: −1, as in the ProjM arm above.
                    sign = -sign;
                    if pair_crossed(*i, *j, flows) {
                        sign = -sign;
                    }
                    let child_i =
                        tree.build_child(term, *i, &mut visited_ops, flows, None, &mut sign)?;
                    let child_j =
                        tree.build_child(term, *j, &mut visited_ops, flows, None, &mut sign)?;
                    tree.add_node(LorentzEvalNode::Gamma5Amp {
                        i: child_i,
                        j: child_j,
                    })
                }
                LorentzOp::Epsilon { mu, nu, rho, sigma } => {
                    visited_ops.push(iop);
                    let a =
                        tree.build_child(term, *mu, &mut visited_ops, flows, None, &mut sign)?;
                    let b =
                        tree.build_child(term, *nu, &mut visited_ops, flows, None, &mut sign)?;
                    let c =
                        tree.build_child(term, *rho, &mut visited_ops, flows, None, &mut sign)?;
                    let d =
                        tree.build_child(term, *sigma, &mut visited_ops, flows, None, &mut sign)?;
                    tree.add_node(LorentzEvalNode::EpsilonAmp { a, b, c, d })
                }
                _ => {
                    todo!(
                        "Routing for remaining ops not yet implemented in tree builder: {:?}",
                        op
                    );
                }
            };
            term_roots.push(term);
        }

        // Find any scalar legs not connected to any operator and add them as scalar roots.
        // Skip the output leg (excluded_leg) — it is not an input for off-shell currents.
        for (ileg, spin) in spins.iter().enumerate() {
            if *spin == 1 {
                if Some(ileg) == idx {
                    continue; // output leg — not an input
                }
                if !tree
                    .nodes
                    .iter()
                    .any(|node| matches!(node, LorentzEvalNode::Leg(i) if *i == ileg))
                {
                    term_roots.push(tree.add_node(LorentzEvalNode::Leg(ileg)));
                }
            }
        }

        // take scalar product
        let root = if term_roots.len() == 1 {
            term_roots[0]
        } else {
            tree.add_node(LorentzEvalNode::Mul {
                children: term_roots,
            })
        };

        tree.root = Some(root);

        Ok((tree.compact_legs(idx), sign, reversed_parity))
    }

    /// Drop the output leg's hole from the input-leg numbering.
    ///
    /// The output leg's wavefunction is never referenced by an off-shell current, so
    /// its position is a hole: every input leg above `out` shifts down by one, and
    /// `Leg(i)`/`P{leg}` then index directly into the caller's gap-free input list
    /// (vertex legs in order, output omitted) with no per-eval reindexing. A `P` *can*
    /// reference the output leg's momentum (e.g. VVV1); it becomes the leg-less
    /// `POut`, evaluated from the input currents.
    fn compact_legs(mut self, idx: Option<usize>) -> Self {
        if let Some(out) = idx {
            for node in &mut self.nodes {
                match node {
                    LorentzEvalNode::Leg(i) if *i > out => *i -= 1,
                    LorentzEvalNode::P { leg } if *leg == out => *node = LorentzEvalNode::POut,
                    LorentzEvalNode::P { leg } if *leg > out => *leg -= 1,
                    _ => {}
                }
            }
        }
        self
    }

    fn render_expression(&self) -> String {
        self.fold_recursive(
            &|node, acc| node.render(acc),
            &|acc, r| if acc.is_empty() { r } else { acc + "," + &r },
            String::new(),
            self.root.expect("should have a root node"),
        )
    }
}

impl std::fmt::Display for LorentzEvalTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.render_expression())
    }
}

/// True iff a fermion pair's plain legs sit on a crossed line (see [`LegAdjoint`]).
/// Summed (negative) indices carry no binding and are skipped.
/// A *standalone* chiral projector (`ψ̄ P_χ ψ`, not gamma-chained) rooted as an
/// off-shell fermion current, whose wrapped input leg sits on a crossed line:
/// the reversed bilinear reading takes the same −1 as the amplitude case
/// ([`pair_crossed`]), but with only one external leg in view. `idx ≥ 0` (the
/// output fermion slot) excludes gamma-chained projectors reached through a
/// summed index, whose −1 is supplied by the runtime reversed-bilinear sign.
/// Pinned by e+e-→τ+τ-H (H emitted off the crossed τ line) vs MadGraph AMP().
fn standalone_projector_crossed(idx: isize, wrapped: isize, flows: &[Option<LegAdjoint>]) -> bool {
    idx >= 0
        && wrapped >= 0
        && matches!(
            flows.get(wrapped as usize).copied().flatten(),
            Some(lf) if lf.crossed
        )
}

/// The runtime `reversed`-bilinear parity this rooted term contributes.
///
/// A `Gamma` op becomes a fermion→vector sink (`GammaVout`, later fused to `FfvVout`)
/// unless the term is rooted at one of the gamma's own fermion legs — then it is a
/// fermion-continuing `GammaIout`/`GammaOout`, which takes no reversed sign. At a
/// `GammaVout` the runtime [`super::kernel::resolve_bra_ket`] reads `reversed = true`
/// when the first operand (the gamma's UFO row index `i`) is a *ket*; the C-conjugation
/// `Cγ^{μT}C⁻¹ = −γ^μ` then flips the current's sign. That flag is fixed by the baked
/// leg adjoint in `flows`, so it is knowable at compile time here. `idx` is the corrected
/// output leg (post [`correct_spin_index_for_flow`]), matching the routing `build_child`
/// performs. Like the build-convention sign, this depends on the rooting, so it is lifted
/// to a per-diagram scalar evaluated at the canonical `VtxIdx(0)` rooting
/// ([`DiagramEvalTree::reversed_convention_sign`]).
fn term_reversed_parity(
    term: &LorentzTerm,
    idx: Option<usize>,
    flows: &[Option<LegAdjoint>],
) -> i8 {
    let mut parity = 1i8;
    for op in &term.ops {
        let LorentzOp::Gamma { i, j, .. } = op else {
            continue;
        };
        // Rooted at a fermion leg → GammaIout/GammaOout, no reversed bilinear.
        if idx == Some(*i as usize) || idx == Some(*j as usize) {
            continue;
        }
        // GammaVout{i, j}: reversed iff the first operand (UFO index i) is a ket.
        if let Some(Some(lf)) = flows.get(*i as usize) {
            if lf.adjoint == Adjoint::Ket {
                parity = -parity;
            }
        }
    }
    parity
}

/// The spinor adjoint a node rooted at the *summed* spinor index `idx` must produce:
/// that of the external fermion its input chain leads to.
///
/// Every operation a fermion line passes through — a gamma slash, a chiral
/// projector, γ⁵ — preserves the adjoint, so one adjoint holds from the external
/// leg the chain starts at to any node on it. Walking spinor indices until a plain
/// leg appears is therefore the whole rule, and it is what makes a γ-chain rootable
/// at a leg that is not on the chain (the vector leg of an FFVV or of a
/// momentum-slashed dipole), where the rooted output carries no adjoint at all.
/// `from_op` is the operator the walk starts inside, so the first step cannot turn
/// straight back through it.
fn chain_adjoint(
    term: &LorentzTerm,
    idx: isize,
    from_op: usize,
    flows: &[Option<LegAdjoint>],
) -> Option<Adjoint> {
    let leg_adjoint = |leg: isize| {
        flows
            .get(leg as usize)
            .copied()
            .flatten()
            .map(|lf| lf.adjoint)
    };
    if idx >= 0 {
        return leg_adjoint(idx);
    }
    let mut walked = vec![from_op];
    let mut cursor = idx;
    // Each step consumes one operator, so the term's operator count bounds the walk
    // (and a cyclic index graph — the four-fermion tensor structures — terminates
    // with `None` rather than looping).
    for _ in 0..term.ops.len() {
        let (iop, op) = term
            .ops
            .iter()
            .enumerate()
            .find(|&(i, op)| op.involves_spinor(cursor) && !walked.contains(&i))?;
        walked.push(iop);
        cursor = other_spinor_index(op, cursor)?;
        if cursor >= 0 {
            return leg_adjoint(cursor);
        }
    }
    None
}

/// The spinor index on the other side of a two-spinor-index operator.
fn other_spinor_index(op: &LorentzOp, idx: isize) -> Option<isize> {
    let (i, j) = match op {
        LorentzOp::Gamma { i, j, .. }
        | LorentzOp::Sigma { i, j, .. }
        | LorentzOp::Identity { i, j }
        | LorentzOp::ProjM { i, j }
        | LorentzOp::ProjP { i, j }
        | LorentzOp::Gamma5 { i, j }
        | LorentzOp::C { i, j } => (*i, *j),
        _ => return None,
    };
    if i == idx {
        Some(j)
    } else if j == idx {
        Some(i)
    } else {
        None
    }
}

fn pair_crossed(i: isize, j: isize, flows: &[Option<LegAdjoint>]) -> bool {
    [i, j].into_iter().any(|k| {
        k >= 0
            && matches!(
                flows.get(k as usize).copied().flatten(),
                Some(lf) if lf.crossed
            )
    })
}

/// Decide whether a gamma-chained chiral projector must emit the opposite chirality
/// (`P_χ → P_χ̄`), for a projector reached through `idx` and wrapping the index
/// `wrapped` on its other side.
///
/// Only gamma-chained projectors (`γ^μ·P_χ` via a summed index) conjugate under a
/// fermion-line reversal (`γ^μ P_χ = P_χ̄ γ^μ`); standalone scalar projectors
/// (`ψ̄ P_χ ψ`) are reversal-invariant and never flip. Two distinct defects require
/// the conjugation, neither with an explicit sign (the reversal −1 of
/// `C(γ^μP_χ)ᵀC⁻¹ = −γ^μP_χ̄` is supplied at runtime by the reversed-bilinear sign,
/// and a crossed pair's two −1s cancel):
///
/// - **Uncrossed reversal** (`idx` summed, `wrapped` a plain leg whose adjoint
///   contradicts its UFO slot — column expects a ket, row a bra): the line
///   traverses the vertex against its arrow, e.g. the initial-state annihilation
///   pair. (The rooted-output variant of this case is canonicalized by
///   [`correct_spin_index_for_flow`], which realizes the conjugation by
///   re-rooting.)
/// - **Crossed line** (the adjacent plain leg has [`LegAdjoint::crossed`]): the two
///   pair wavefunctions sit in each other's slots with conjugate types
///   (all-incoming diagram convention), evaluating `ū₁Γv₂` for a vertex defined as
///   `ū₂Γv₁ = −ū₁(CΓᵀC⁻¹)v₂`. Crossing inverts slot identity and adjoint together,
///   so this case is adjoint-aligned and disjoint from the uncrossed reversal; it is
///   also checked at the rooted output leg (`idx` plain, projector wrapping the
///   gamma), where re-rooting cannot see it.
fn chiral_correction(
    idx: isize,
    wrapped: isize,
    wrapped_is_row: bool,
    flows: &[Option<LegAdjoint>],
) -> bool {
    if idx >= 0 {
        // Projector adjacent to the rooted output leg (wraps the gamma chain). The
        // adjoint-vs-slot alignment was already canonicalized by re-rooting; only a
        // crossed line still needs the conjugation.
        return wrapped < 0
            && matches!(
                flows.get(idx as usize).copied().flatten(),
                Some(lf) if lf.crossed
            );
    }
    if wrapped < 0 {
        return false;
    }
    let Some(lf) = flows.get(wrapped as usize).copied().flatten() else {
        return false;
    };
    if lf.crossed {
        return true;
    }
    let expected = if wrapped_is_row {
        Adjoint::Bra
    } else {
        Adjoint::Ket
    };
    lf.adjoint != expected
}

/// Adjust leg index for spinor adjoint if it is a spinor leg.
///
/// UFO spinor pairs are oriented `(column/ket slot, row/bra slot)`: for `ψ̄₂Γψ₁`
/// (e.g. SM FFV `Gamma(3,2,1)` with particles (ℓ⁺, ℓ⁻, V)) the ket slot is the column
/// the ket contracts into, the bra slot the row for the bra. An off-shell output at the
/// ket slot leaves `ψ̄Γ` — a bra (`Adjoint::Bra`); an output at the bra slot leaves
/// `Γψ` — a ket (`Adjoint::Ket`). When the baked adjoint disagrees with the rooted slot
/// (the line traverses the vertex against its UFO arrow), re-root at the adjoint-matching
/// slot so the chiral projector lands on the physical side of the gamma (ket:
/// `ε̸·P_χ·ψ`, bra: `ψ̄·ε̸·P_χ`); the leg compaction in `build_at_leg` keeps the caller's
/// child binding unchanged.
///
/// `flow` is the vertex's own pairing, which is the only thing that says which slots
/// share a line. It is not `(1,2)(3,4)…` in general: a four-fermion structure can pair
/// `(1,4)(2,3)`, and reading consecutive slots as pairs there re-roots the output onto a
/// leg of the *other* line, which produces an amplitude that depends on the root chosen.
fn correct_spin_index_for_flow(
    spins: &[i32],
    flow: &[(usize, usize)],
    idx: Option<usize>,
    adjoint: Option<Adjoint>,
) -> Result<Option<usize>, RootLorentzError> {
    match (idx, adjoint) {
        (Some(idx), Some(f)) => {
            let Some(&(ket, bra)) = flow.iter().find(|&&(k, b)| k == idx || b == idx) else {
                return Err(RootLorentzError::InvalidStructure(format!(
                    "leg {idx} carries a spinor adjoint but no fermion line of this \
                     structure reaches it"
                )));
            };
            Ok(Some(match f {
                Adjoint::Bra => ket,
                Adjoint::Ket => bra,
            }))
        }
        (Some(i), None) if spins[i] == 2 => Err(RootLorentzError::MissingAdjoint(
            "adjoint (bra/ket) must be specified for spinor output".to_string(),
        )),
        (None, Some(_)) => {
            // Amplitude contraction: reversed/crossed pairs are handled per
            // projector in `chiral_correction` via the per-leg flows.
            Ok(idx)
        }
        (_, _) => Ok(idx), // No index to validate
    }
}

// ───────────────── Cyclic four-fermion (tensor⊗tensor) structures ─────────────────
//
// A four-fermion structure whose two fermion lines share *two* summed Lorentz indices
// closes a 4-cycle in the index graph, so no rooted tree can contract it one index at
// a time. What makes it evaluable anyway is that a two-gamma chain is a Clifford
// element of grades 0 and 2 alone (`γ^αγ^β = g^{αβ} − i σ^{αβ}`): each line's chain is
// fixed by two numbers per index pair, the bilinears `ψ̄ψ` and `ψ̄σ^{μν}ψ` of its own
// two ends. Evaluating one line into those coefficients cuts the cycle; contracting
// them against the other line's two gammas gives a Clifford element again
//
//     γ_α γ_β (g^{αβ} s − i t^{αβ}) = 4 s − σ_{αβ} t^{αβ},
//
// which is then applied to the surviving line — as an operator on its continuing
// spinor, or as the pairing that closes it into the amplitude. The two index orders
// (`γ^αγ^β` against `γ_αγ_β` or against `γ_βγ_α`) differ only in the sign of the
// grade-2 term.

/// One fermion line of a recognised cyclic tensor⊗tensor term.
///
/// The chain reads `ψ_row · row_ops · γ^{shared[0]} γ^{shared[1]} · col_ops · ψ_col`
/// in the vertex's own (row = bra slot, column = ket slot) orientation.
#[derive(Clone, Debug)]
struct TensorLine {
    /// Vertex leg at the chain's row (bra) end.
    row_leg: usize,
    /// Vertex leg at the chain's column (ket) end.
    col_leg: usize,
    /// Non-gamma operators between the row leg and the first shared gamma, as
    /// operator indices into the term. Order within the list is immaterial — the
    /// recognised set is mutually commuting (see
    /// [`apply_chain_ops`](LorentzEvalTree::apply_chain_ops)).
    row_ops: Vec<usize>,
    /// Non-gamma operators between the second shared gamma and the column leg.
    col_ops: Vec<usize>,
    /// The two shared Lorentz index labels, in chain order.
    shared: [isize; 2],
}

/// A term recognised as two fermion lines joined by two summed Lorentz indices.
#[derive(Clone, Debug)]
struct TensorTerm {
    lines: [TensorLine; 2],
    /// Whether the two lines traverse the shared indices in opposite orders
    /// (`γ^αγ^β` against `γ_βγ_α`).
    reversed_order: bool,
}

/// The spinor index pair `(row, column)` of an operator the tensor path can place on a
/// fermion line, or `None`.
///
/// `Sigma` and `C` carry a spinor pair too and are deliberately absent: a `Sigma`
/// carries the two Lorentz indices itself (it is the two gammas already contracted, and
/// belongs where they do rather than beside them), and charge conjugation is not
/// supported anywhere in the rooting.
fn spinor_pair(op: &LorentzOp) -> Option<(isize, isize)> {
    match op {
        LorentzOp::Gamma { i, j, .. }
        | LorentzOp::Identity { i, j }
        | LorentzOp::ProjM { i, j }
        | LorentzOp::ProjP { i, j }
        | LorentzOp::Gamma5 { i, j } => Some((*i, *j)),
        _ => None,
    }
}

/// Recognise a cyclic term as a tensor⊗tensor four-fermion structure.
///
/// `Ok(None)` for a term whose index graph is a tree — the rooted contraction handles
/// those. `Err` for a cycle this shape does not cover, which is a refusal naming what
/// it found rather than an "index has no operator" failure deep in the walk.
///
/// The recognised shape is deliberately narrow: four fermion legs and nothing else,
/// every operator a spinor-index one, each line carrying exactly two adjacent `Gamma`
/// factors whose Lorentz indices are the two labels the lines share. That is every
/// cyclic structure SMEFTsim writes (`FFFF5`–`8`, `FFFF19`–`21`); a literal
/// `Sigma(α,β,i,j)` carrying both shared indices on one line is the same object with
/// the two gammas already contracted and is the natural next case.
fn cyclic_tensor_term(
    term: &LorentzTerm,
    spins: &[i32],
) -> Result<Option<TensorTerm>, RootLorentzError> {
    if !cyclic_index_graph(term) {
        return Ok(None);
    }
    let refuse = |what: &str| {
        Err(RootLorentzError::UnsupportedVertex(format!(
            "cyclic Lorentz structure: {what}; only two fermion lines joined by two \
             summed Lorentz indices are evaluable as a rank-2 current"
        )))
    };
    if spins.len() != 4 || spins.iter().any(|&s| s != 2) {
        return refuse("not a four-fermion vertex");
    }
    if term.ops.iter().any(|op| spinor_pair(op).is_none()) {
        return refuse(
            "an operator outside the fermion-line set (Gamma/ProjM/ProjP/Gamma5/Identity)",
        );
    }

    // Each chain starts at the plain leg sitting at an operator's *row* index and
    // walks column → row until it reaches a second plain leg.
    let mut lines: Vec<(Vec<usize>, usize, usize)> = Vec::new();
    let mut used: Vec<usize> = Vec::new();
    for leg in 0..spins.len() {
        let start = term
            .ops
            .iter()
            .position(|op| spinor_pair(op).is_some_and(|(i, _)| i == leg as isize));
        let Some(start) = start.filter(|k| !used.contains(k)) else {
            continue;
        };
        let mut chain = vec![start];
        let mut cursor = spinor_pair(&term.ops[start]).unwrap().1;
        while cursor < 0 {
            let next = term.ops.iter().enumerate().find(|(k, op)| {
                spinor_pair(op).is_some_and(|(i, _)| i == cursor) && !chain.contains(k)
            });
            let Some((next, _)) = next else {
                return refuse("a summed spinor index with no continuing operator");
            };
            chain.push(next);
            cursor = spinor_pair(&term.ops[next]).unwrap().1;
        }
        used.extend(chain.iter().copied());
        lines.push((chain, leg, cursor as usize));
    }
    if lines.len() != 2 {
        return refuse("the fermion lines do not split the vertex into two chains");
    }
    used.sort_unstable();
    used.dedup();
    if used.len() != term.ops.len() {
        return refuse("an operator on no fermion line");
    }

    let mut built: Vec<TensorLine> = Vec::new();
    for (chain, row_leg, col_leg) in lines {
        let gammas: Vec<usize> = chain
            .iter()
            .enumerate()
            .filter(|&(_, &k)| matches!(term.ops[k], LorentzOp::Gamma { .. }))
            .map(|(pos, _)| pos)
            .collect();
        if gammas.len() != 2 || gammas[1] != gammas[0] + 1 {
            return refuse("a fermion line without exactly two adjacent Gamma factors");
        }
        let mu_of = |pos: usize| match term.ops[chain[pos]] {
            LorentzOp::Gamma { mu, .. } => mu,
            _ => unreachable!("gamma positions were filtered on the op"),
        };
        built.push(TensorLine {
            row_leg,
            col_leg,
            row_ops: chain[..gammas[0]].to_vec(),
            col_ops: chain[gammas[1] + 1..].to_vec(),
            shared: [mu_of(gammas[0]), mu_of(gammas[1])],
        });
    }

    let (a, b) = (built[0].shared, built[1].shared);
    if a[0] == a[1] || !a.iter().all(|x| *x < 0) {
        return refuse("the two gammas of a line do not carry distinct summed indices");
    }
    let reversed_order = if a == b {
        false
    } else if a == [b[1], b[0]] {
        true
    } else {
        return refuse("the two lines do not share the same pair of summed indices");
    };
    let [line_a, line_b]: [TensorLine; 2] = built
        .try_into()
        .expect("exactly two lines were built above");
    Ok(Some(TensorTerm {
        lines: [line_a, line_b],
        reversed_order,
    }))
}

/// The adjoint the vertex expects at the *row* slot of a line, as bound: `Bra` unless
/// the line runs against the vertex's own arrow.
///
/// At the rooted output leg there is no bound wavefunction — `flows` holds the adjoint
/// of the current the vertex *produces* there, which is the opposite of what the slot
/// would have held (an output at the bra slot leaves `Γψ`, a ket).
fn row_slot_adjoint(
    line: &TensorLine,
    out: Option<usize>,
    flows: &[Option<LegAdjoint>],
) -> Option<Adjoint> {
    let bound = flows.get(line.row_leg).copied().flatten()?.adjoint;
    Some(if out == Some(line.row_leg) {
        match bound {
            Adjoint::Ket => Adjoint::Bra,
            Adjoint::Bra => Adjoint::Ket,
        }
    } else {
        bound
    })
}

/// True iff either end of the line sits on a crossed fermion line.
fn line_crossed(line: &TensorLine, flows: &[Option<LegAdjoint>]) -> bool {
    [line.row_leg, line.col_leg].into_iter().any(|leg| {
        matches!(
            flows.get(leg).copied().flatten(),
            Some(lf) if lf.crossed
        )
    })
}

impl LorentzEvalTree {
    /// Attach a line's non-gamma operators to the end they sit on.
    ///
    /// The nodes act on whichever adjoint the child turns out to carry, so the same
    /// tree serves a line read along the vertex's arrow and one read against it: the
    /// reversal replaces `X γ^αγ^β Y` by `Y γ^βγ^α X`, which moves each factor to the
    /// other side of the chain *together with* the slot it belongs to. Order within a
    /// side is immaterial — the recognised set (chiral projectors, `γ⁵`, the identity)
    /// is diagonal in chirality and mutually commuting.
    fn apply_chain_ops(
        &mut self,
        term: &LorentzTerm,
        node: usize,
        ops: &[usize],
    ) -> Result<usize, RootLorentzError> {
        let mut node = node;
        for &iop in ops {
            node = match term.ops[iop] {
                LorentzOp::Identity { .. } => node,
                LorentzOp::ProjM { .. } => self.add_node(LorentzEvalNode::ProjM { i: node }),
                LorentzOp::ProjP { .. } => self.add_node(LorentzEvalNode::ProjP { i: node }),
                LorentzOp::Gamma5 { .. } => self.add_node(LorentzEvalNode::Gamma5 { i: node }),
                ref other => {
                    return Err(RootLorentzError::UnsupportedVertex(format!(
                        "cyclic Lorentz structure: {other:?} on a fermion line beside the \
                         two shared gammas"
                    )))
                }
            };
        }
        Ok(node)
    }

    /// A line end: its leg leaf with that end's operators applied.
    fn chain_end(
        &mut self,
        term: &LorentzTerm,
        leg: usize,
        ops: &[usize],
    ) -> Result<usize, RootLorentzError> {
        let leaf = self.add_node(LorentzEvalNode::Leg(leg));
        self.apply_chain_ops(term, leaf, ops)
    }

    /// The cut line evaluated as a Clifford element, with the index order the pairing
    /// of the two chains dictates.
    ///
    /// The order flips once per line that is read against the vertex's own arrow: the
    /// reversal replaces the chain by `C Γᵀ C⁻¹`, and for `X γ^α γ^β Y` that is
    /// `Y γ^β γ^α X` — the same operators with the two gammas transposed, so only the
    /// grade-2 coefficient changes sign. A crossed line reads the same way and adds
    /// the `−1` of the conjugated pair's operator reordering.
    fn add_cut_line(
        &mut self,
        term: &LorentzTerm,
        tensor: &TensorTerm,
        cut: usize,
        out: Option<usize>,
        flows: &[Option<LegAdjoint>],
        sign: &mut f64,
    ) -> Result<usize, RootLorentzError> {
        let line = &tensor.lines[cut];
        let row = self.chain_end(term, line.row_leg, &line.row_ops)?;
        let col = self.chain_end(term, line.col_leg, &line.col_ops)?;
        let mut reversed = tensor.reversed_order;
        for l in &tensor.lines {
            if row_slot_adjoint(l, out, flows) != Some(Adjoint::Bra) {
                reversed = !reversed;
            }
            if line_crossed(l, flows) {
                reversed = !reversed;
                *sign = -*sign;
            }
        }
        Ok(self.add_node(if reversed {
            LorentzEvalNode::FierzOutRev { i: row, j: col }
        } else {
            LorentzEvalNode::FierzOut { i: row, j: col }
        }))
    }

    /// Root a recognised cyclic tensor⊗tensor term.
    ///
    /// The cycle is cut at the fermion line that does not carry the output leg: that
    /// line becomes a Clifford element, and the element is contracted into the other
    /// line — applied to its continuing spinor when the output is one of its own legs,
    /// paired with its two ends when the term sinks into the amplitude.
    fn build_tensor_term(
        term: &LorentzTerm,
        tensor: &TensorTerm,
        idx: Option<usize>,
        out_adjoint: Option<Adjoint>,
        flows: &[Option<LegAdjoint>],
    ) -> Result<(Self, f64), RootLorentzError> {
        let mut tree = LorentzEvalTree {
            nodes: vec![],
            root: None,
        };
        let mut sign = 1.0;
        let keep = match idx {
            Some(out) => tensor
                .lines
                .iter()
                .position(|l| l.row_leg == out || l.col_leg == out)
                .ok_or_else(|| {
                    RootLorentzError::InvalidStructure(format!(
                        "leg {out} carries no fermion line of this cyclic structure"
                    ))
                })?,
            // Both lines sink into the amplitude, so either may be cut; the choice is
            // fixed here and its irrelevance is what `rooting_soundness` measures by
            // re-rooting at each leg (which cuts the other line).
            None => 0,
        };
        let cut = 1 - keep;
        let m = tree.add_cut_line(term, tensor, cut, idx, flows, &mut sign)?;

        let line = &tensor.lines[keep];
        let root = match idx {
            None => {
                let row = tree.chain_end(term, line.row_leg, &line.row_ops)?;
                let col = tree.chain_end(term, line.col_leg, &line.col_ops)?;
                tree.add_node(LorentzEvalNode::FierzPair { m, i: row, j: col })
            }
            Some(out) => {
                // The continuing input is the end that is not the output leg, and the
                // current keeps that line's adjoint; the output end's own operators
                // apply to the current the vertex produces, not to any input.
                let (leg, cont_ops, out_ops) = if line.row_leg == out {
                    (line.col_leg, &line.col_ops, &line.row_ops)
                } else {
                    (line.row_leg, &line.row_ops, &line.col_ops)
                };
                let f = tree.chain_end(term, leg, cont_ops)?;
                let current = match out_adjoint {
                    Some(Adjoint::Ket) => {
                        tree.add_node(LorentzEvalNode::MultivectorIout { m, j: f })
                    }
                    Some(Adjoint::Bra) => {
                        tree.add_node(LorentzEvalNode::MultivectorOout { m, i: f })
                    }
                    None => {
                        return Err(RootLorentzError::InvalidStructure(
                            "cyclic four-fermion structure rooted at a leg with no spinor \
                             adjoint"
                                .to_string(),
                        ))
                    }
                };
                tree.apply_chain_ops(term, current, out_ops)?
            }
        };
        tree.root = Some(root);
        Ok((tree, sign))
    }
}

/// True iff the term's index graph has a cycle: two operators joined by more than one
/// contracted index, directly or through a chain.
///
/// The rooting turns a term into a tree by walking outward from one index, so it can
/// express exactly the terms whose index graph *is* a tree. SMEFTsim's tensor⊗tensor
/// four-fermion structures are the first that are not: two fermion lines joined by two
/// summed Lorentz indices close a 4-cycle, and evaluating them needs a rank-2 tensor
/// intermediate carried between the lines rather than a rooted contraction. Detecting
/// the cycle up front is what turns that into one statement about the structure instead
/// of an "index has no operator" failure deep in the walk, where the walk has already
/// consumed half the cycle.
///
/// Operators are the nodes and each shared index one edge; a cycle is an edge joining
/// two operators already connected.
fn cyclic_index_graph(term: &LorentzTerm) -> bool {
    let mut parent: Vec<usize> = (0..term.ops.len()).collect();
    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }
    // Every index each operator carries, spinor and Lorentz alike.
    let indices = |op: &LorentzOp| -> Vec<isize> {
        match op {
            LorentzOp::Gamma { mu, i, j } => vec![*mu, *i, *j],
            LorentzOp::Sigma { mu, nu, i, j } => vec![*mu, *nu, *i, *j],
            LorentzOp::Identity { i, j }
            | LorentzOp::ProjM { i, j }
            | LorentzOp::ProjP { i, j }
            | LorentzOp::Gamma5 { i, j }
            | LorentzOp::C { i, j } => vec![*i, *j],
            LorentzOp::Metric { mu, nu } => vec![*mu, *nu],
            LorentzOp::P { mu, .. } => vec![*mu],
            LorentzOp::Epsilon { mu, nu, rho, sigma } => vec![*mu, *nu, *rho, *sigma],
        }
    };
    let mut seen: std::collections::HashMap<isize, usize> = std::collections::HashMap::new();
    for (iop, op) in term.ops.iter().enumerate() {
        for idx in indices(op) {
            let Some(&other) = seen.get(&idx) else {
                seen.insert(idx, iop);
                continue;
            };
            let (a, b) = (find(&mut parent, other), find(&mut parent, iop));
            if a == b {
                return true;
            }
            parent[a] = b;
        }
    }
    false
}

/// [`cyclic_tensor_term`] over a whole structure, as the error the evaluator reports.
///
/// Reported here rather than left to the per-term rooting so that a cycle the tensor
/// path does not cover is one statement naming the structure, instead of an
/// "index has no operator" failure deep in a walk that has already consumed half the
/// cycle.
pub fn reject_cyclic_structure(
    name: &str,
    structure: &str,
    spins: &[i32],
    expr: &crate::ufo::lorentz::LorentzExpr,
) -> Result<(), RootLorentzError> {
    for term in expr.iter() {
        if let Err(why) = cyclic_tensor_term(term, spins) {
            return Err(RootLorentzError::UnsupportedVertex(format!(
                "Lorentz structure {name} ({structure}): {why}"
            )));
        }
    }
    Ok(())
}

/// Resolve a single LorentzTerm into a rooted primitive with the output leg fixed.
///
/// # Arguments
/// * `term` — The UFO LorentzTerm to resolve.
/// * `spins` — Spin codes [1, 2, 3] for each leg
/// * `flow` — The vertex's fermion pairing, `(ket slot, bra slot)` per line.
/// * `result_leg_idx` — The output leg (0-indexed), or `None` for amplitude (scalar sink).
/// * `out_adjoint` — Spinor adjoint of the output leg (`Some` iff a fermion output), used to
///   pick the in/out gamma routine.
///
/// # Returns
/// A `RootedTerm` ready for evaluation, or a `RootLorentzError`.
pub fn root_term(
    term: &crate::ufo::lorentz::LorentzTerm,
    spins: &[i32],
    flow: &[(usize, usize)],
    result_leg_idx: Option<usize>,
    flows: &[Option<LegAdjoint>],
) -> Result<RootedTerm, RootLorentzError> {
    let (tree, sign, reversed_sign) =
        LorentzEvalTree::build_at_leg(term, spins, flow, result_leg_idx, flows)?;
    Ok(RootedTerm {
        coeff: term.coeff,
        build_sign: if sign < 0.0 { -1 } else { 1 },
        reversed_sign,
        tree,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ufo::lorentz::LorentzTerm;

    /// The consecutive fermion pairing `(1,2)(3,4)…` UFO writes for every structure
    /// with two fermion legs, as the hand-built terms here all have.
    fn flow_of(spins: &[i32]) -> Vec<(usize, usize)> {
        let fermions: Vec<usize> = spins
            .iter()
            .enumerate()
            .filter(|(_, &s)| s == 2)
            .map(|(i, _)| i)
            .collect();
        fermions.chunks(2).map(|p| (p[0], p[1])).collect()
    }

    /// Uncrossed per-leg binding shorthand for hand-built adjoint vectors.
    fn lf(adjoint: Adjoint) -> Option<LegAdjoint> {
        Some(LegAdjoint {
            adjoint,
            crossed: false,
        })
    }

    #[test]
    fn test_vvs_rooted_at_vector_leg_is_a_vector_current() {
        // VVS1: Metric(1,2), spins [3,3,1] (Z, Z, H). Rooting at a *vector* leg
        // (the off-shell Z current of an HZZ vertex) must raise that index via
        // the metric: the current is the OTHER vector leg × the scalar leg — a
        // vector, NOT a scalar contraction. It must never reference its own output
        // leg as an input; since `build_at_leg` compacts leg references over the
        // removed output, that means every `Leg(i)` must land in the gap-free input
        // range `0..n_legs-1`. Regression for the VVS Leg/placeholder crash.
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::Metric { mu: 0, nu: 1 }],
        };
        let spins = vec![3, 3, 1];
        let n_inputs = spins.len() - 1; // gap-free input legs (output removed)

        let max_leg = |t: &LorentzEvalTree| {
            t.nodes
                .iter()
                .filter_map(|n| match n {
                    LorentzEvalNode::Leg(i) => Some(*i),
                    _ => None,
                })
                .max()
        };

        // Output = vector leg 0: current = (other vector) × (scalar), both compacted
        // into 0..n_inputs.
        let (t0, _, _) =
            LorentzEvalTree::build_at_leg(&term, &spins, &flow_of(&spins), Some(0), &[]).unwrap();
        assert!(
            max_leg(&t0).is_some_and(|m| m < n_inputs),
            "VVS rooted at leg 0 must only index the gap-free inputs: {t0:?}"
        );
        assert!(
            matches!(t0.root_value(), LorentzEvalNode::Mul { .. }),
            "VVS rooted at a vector leg must yield a vector (scalar×vector), got {:?}",
            t0.root_value()
        );

        // Output = vector leg 2 (idx 1): same invariant.
        let (t1, _, _) =
            LorentzEvalTree::build_at_leg(&term, &spins, &flow_of(&spins), Some(1), &[]).unwrap();
        assert!(
            max_leg(&t1).is_some_and(|m| m < n_inputs),
            "VVS rooted at leg 1 must only index the gap-free inputs: {t1:?}"
        );

        // Output = scalar leg 3 (idx 2): unchanged — a Metric contraction → scalar H.
        let (t2, _, _) =
            LorentzEvalTree::build_at_leg(&term, &spins, &flow_of(&spins), Some(2), &[]).unwrap();
        assert!(
            matches!(t2.root_value(), LorentzEvalNode::Metric { .. }),
            "VVS rooted at the scalar leg must contract the two vectors: {:?}",
            t2.root_value()
        );
    }

    #[test]
    fn test_root_ffv1_photon_current_at_leg3() {
        // FFV1: Gamma(3,2,1) rooted at vector leg 3 → SpinorCurrent{Both,row=2,col=1}
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::Gamma { mu: 2, i: 1, j: 0 }],
        };
        let spins = vec![2, 2, 3]; // e+, e-, photon
        let result = root_term(
            &term,
            &spins,
            &flow_of(&spins),
            Some(2),
            &[lf(Adjoint::Ket), lf(Adjoint::Bra), None],
        )
        .unwrap();
        assert_eq!(result.coeff, 1.0);
        assert_eq!(
            result.tree,
            LorentzEvalTree {
                nodes: vec![
                    LorentzEvalNode::Leg(1),
                    LorentzEvalNode::Leg(0),
                    LorentzEvalNode::GammaVout { i: 0, j: 1 },
                ],
                root: Some(2)
            }
        );
    }

    #[test]
    fn test_root_ffv1_fermion_current_uses_flow() {
        // FFV1: Gamma(mu=2, i=1, j=0) rooted at fermion leg 0 must produce
        // GammaIout with that leg, and rooted at fermion leg 1 must produce
        // GammaOout with that leg. Any other configuration should raise a MissingAdjoint error.
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::Gamma { mu: 2, i: 1, j: 0 }],
        };
        let spins = vec![2, 2, 3];

        let ket = root_term(
            &term,
            &spins,
            &flow_of(&spins),
            Some(0),
            &[lf(Adjoint::Ket), lf(Adjoint::Ket), None],
        )
        .unwrap();
        assert_eq!(
            ket.tree,
            LorentzEvalTree {
                nodes: vec![
                    LorentzEvalNode::Leg(1), // mu (vector), compacted over removed out=0
                    LorentzEvalNode::Leg(0), // the other fermion input
                    LorentzEvalNode::GammaIout { mu: 0, j: 1 },
                ],
                root: Some(2)
            }
        );

        let bra = root_term(
            &term,
            &spins,
            &flow_of(&spins),
            Some(1),
            &[lf(Adjoint::Bra), lf(Adjoint::Bra), None],
        )
        .unwrap();
        assert_eq!(
            bra.tree,
            LorentzEvalTree {
                nodes: vec![
                    LorentzEvalNode::Leg(1),
                    LorentzEvalNode::Leg(0),
                    LorentzEvalNode::GammaOout { mu: 0, i: 1 },
                ],
                root: Some(2)
            }
        );

        // A fermion output with no adjoint is an internal inconsistency.
        assert!(matches!(
            root_term(&term, &spins, &flow_of(&spins), Some(0), &[]),
            Err(RootLorentzError::MissingAdjoint(_))
        ));
    }

    #[test]
    fn test_root_ffv1_amplitude_at_sink() {
        // FFV1 rooted at amplitude (scalar sink) → ScalarProduct of the vector output and itself
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::Gamma { mu: 2, i: 1, j: 0 }],
        };
        let spins = vec![2, 2, 3];
        let result = root_term(&term, &spins, &flow_of(&spins), None, &[]).unwrap();
        assert_eq!(result.coeff, 1.0);
        // When rooted at amplitude, builds the Gamma structure and contracts with a vector leg
        assert_eq!(
            result.tree,
            LorentzEvalTree {
                nodes: vec![
                    LorentzEvalNode::Leg(1),
                    LorentzEvalNode::Leg(0),
                    LorentzEvalNode::GammaVout { i: 0, j: 1 },
                    LorentzEvalNode::Leg(2),
                    LorentzEvalNode::Metric { mu: 2, nu: 3 },
                ],
                root: Some(4)
            }
        )
    }

    #[test]
    fn test_root_ffv2_left_chiral() {
        // FFV2: Gamma(3,2,-1)*ProjM(-1,1) → left-chiral fermion rooted at amplitude
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![
                LorentzOp::Gamma { mu: 2, i: 1, j: -1 },
                LorentzOp::ProjM { i: -1, j: 0 },
            ],
        };
        let spins = vec![2, 2, 3];

        // amplitude case
        let result = root_term(&term, &spins, &flow_of(&spins), None, &[]).unwrap();
        assert_eq!(result.coeff, 1.0);
        // The tree routes through ProjM → Gamma
        assert_eq!(
            result.tree,
            LorentzEvalTree {
                nodes: vec![
                    LorentzEvalNode::Leg(1),
                    LorentzEvalNode::Leg(0),
                    LorentzEvalNode::ProjM { i: 1 },
                    LorentzEvalNode::GammaVout { i: 0, j: 2 },
                    LorentzEvalNode::Leg(2),
                    LorentzEvalNode::Metric { mu: 3, nu: 4 },
                ],
                root: Some(5)
            }
        );

        // A ket output (Adjoint::Ket) roots at the row slot regardless of the requested
        // pair leg, so the projector lands on the input: `ε̸·P_χ·ψ` (the reversed
        // traversal's chirality conjugation is realized by the re-rooting; no
        // explicit sign — see `chiral_correction`).
        let flows_in = [lf(Adjoint::Ket), lf(Adjoint::Ket), None];
        for requested in [0usize, 1] {
            let result =
                root_term(&term, &spins, &flow_of(&spins), Some(requested), &flows_in).unwrap();
            assert_eq!(result.coeff, 1.0);
            assert_eq!(
                result.tree,
                LorentzEvalTree {
                    nodes: vec![
                        LorentzEvalNode::Leg(1),
                        LorentzEvalNode::Leg(0),
                        LorentzEvalNode::ProjM { i: 1 },
                        LorentzEvalNode::GammaIout { mu: 0, j: 2 },
                    ],
                    root: Some(3)
                }
            );
        }

        // A bra output (Adjoint::Bra) roots at the column slot, so the projector lands
        // on the output: `ψ̄·ε̸·P_χ`.
        let flows_out = [lf(Adjoint::Bra), lf(Adjoint::Bra), None];
        for requested in [0usize, 1] {
            let result =
                root_term(&term, &spins, &flow_of(&spins), Some(requested), &flows_out).unwrap();
            assert_eq!(result.coeff, 1.0);
            assert_eq!(
                result.tree,
                LorentzEvalTree {
                    nodes: vec![
                        LorentzEvalNode::Leg(1),
                        LorentzEvalNode::Leg(0),
                        LorentzEvalNode::GammaOout { mu: 0, i: 1 },
                        LorentzEvalNode::ProjM { i: 2 },
                    ],
                    root: Some(3)
                }
            );
        }
    }

    #[test]
    fn test_root_ffs_yukawa() {
        // FFS1: ProjM(2,1) rooted at amplitude → ScalarProduct (bilinear)
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::ProjM { i: 1, j: 0 }],
        };
        let spins = vec![2, 2, 1];
        let result = root_term(&term, &spins, &flow_of(&spins), None, &[]).unwrap();
        assert_eq!(
            result.tree,
            LorentzEvalTree {
                nodes: vec![
                    LorentzEvalNode::Leg(1),
                    LorentzEvalNode::Leg(0),
                    LorentzEvalNode::ProjMAmp { i: 0, j: 1 },
                    LorentzEvalNode::Leg(2),
                    LorentzEvalNode::Mul {
                        children: vec![2, 3]
                    },
                ],
                root: Some(4)
            }
        )
    }

    #[test]
    fn test_root_ffs_off_shell_scalar() {
        // FFS1: ProjM(2,1) rooted at scalar leg 3 → just the bilinear, no Leg(2)
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::ProjM { i: 1, j: 0 }],
        };
        let spins = vec![2, 2, 1];
        let result = root_term(
            &term,
            &spins,
            &flow_of(&spins),
            Some(2),
            &[lf(Adjoint::Ket), lf(Adjoint::Bra), None],
        )
        .unwrap();
        assert_eq!(
            result.tree,
            LorentzEvalTree {
                nodes: vec![
                    LorentzEvalNode::Leg(1),
                    LorentzEvalNode::Leg(0),
                    LorentzEvalNode::ProjMAmp { i: 0, j: 1 },
                ],
                root: Some(2)
            }
        )
    }

    #[test]
    fn test_root_sigma_unsupported() {
        // Sigma should trigger UnsupportedVertex when it's on the path to the root.
        // Root at leg 1 (fermion) where Sigma(1,2,2,1) is present
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::Sigma {
                mu: 0,
                nu: 1,
                i: 1,
                j: 0,
            }],
        };
        let spins = vec![2, 2, 3];
        let result = root_term(
            &term,
            &spins,
            &flow_of(&spins),
            Some(0),
            &[lf(Adjoint::Ket), lf(Adjoint::Ket), None],
        ); // root at leg 1 (0-indexed as 0)
        assert!(matches!(
            result,
            Err(RootLorentzError::UnsupportedVertex(_))
        ));
    }

    #[test]
    fn test_root_vvs_metric() {
        // VVS1: Metric(1,2) rooted at amplitude → plain Metric contraction. The
        // pure-metric vertex's −1 is the rooting-convention `build_sign`, carried
        // separately from `coeff` (it is lifted per-vertex into the diagram's
        // `fermi_sign` at the canonical rooting).
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::Metric { mu: 0, nu: 1 }],
        };
        let spins = vec![3, 3, 1];
        let result = root_term(&term, &spins, &flow_of(&spins), None, &[]).unwrap();
        assert_eq!(result.coeff, 1.0);
        assert_eq!(result.build_sign, -1);
        // When rooted at amplitude with 2 vector legs, uses Metric to contract them
        assert_eq!(
            result.tree,
            LorentzEvalTree {
                nodes: vec![
                    LorentzEvalNode::Leg(0),
                    LorentzEvalNode::Leg(1),
                    LorentzEvalNode::Metric { mu: 0, nu: 1 },
                    LorentzEvalNode::Leg(2),
                    LorentzEvalNode::Mul {
                        children: vec![2, 3]
                    },
                ],
                root: Some(4)
            }
        )
    }

    /// The four-vector contact's −1 is a property of the vertex, not of the operators
    /// an individual term happens to carry.
    ///
    /// The four-gluon vertex and the field-strength operators sharing its legs form one
    /// interaction whose Lorentz structures range over pure metrics (`VVVV1`), momentum
    /// products (`VVVV2`) and Levi-Civita tensors carrying no `Metric` at all (`VVVV3`),
    /// and the per-vertex sign factorisation requires one sign across all of them.
    /// Keying the −1 on the term's operator content signs only the first, which is what
    /// put the six higher-derivative four-gluon contact amplitudes of `g g > g g NP<=1`
    /// a relative −1 away from the three Standard-Model ones. The triple-vector vertex
    /// is the control: its own −1 is a source-side diagram sign, not a build sign, so a
    /// rule keyed on "all-vector" alone would double it.
    #[test]
    fn four_vector_contact_sign_is_uniform_over_its_structures() {
        let vvvv = vec![3, 3, 3, 3];
        let structures = [
            // VVVV1: Metric(1,4)*Metric(2,3)
            (
                "pure metric",
                vec![
                    LorentzOp::Metric { mu: 0, nu: 3 },
                    LorentzOp::Metric { mu: 1, nu: 2 },
                ],
            ),
            // VVVV2: P(3,2)*P(4,1)*Metric(1,2)
            (
                "momentum",
                vec![
                    LorentzOp::P { mu: 2, leg: 1 },
                    LorentzOp::P { mu: 3, leg: 0 },
                    LorentzOp::Metric { mu: 0, nu: 1 },
                ],
            ),
            // VVVV3: Epsilon(1,2,3,4)*P(-1,2)*P(-1,3)
            (
                "Levi-Civita",
                vec![
                    LorentzOp::Epsilon {
                        mu: 0,
                        nu: 1,
                        rho: 2,
                        sigma: 3,
                    },
                    LorentzOp::P { mu: -1, leg: 1 },
                    LorentzOp::P { mu: -1, leg: 2 },
                ],
            ),
        ];
        for (what, ops) in structures {
            let term = LorentzTerm { coeff: 1.0, ops };
            let rooted = root_term(&term, &vvvv, &[], None, &[None; 4]).unwrap();
            assert_eq!(
                rooted.build_sign, -1,
                "the {what} four-vector contact structure must carry the contact −1"
            );
        }

        // VVV5: P(3,1)*Metric(1,2) at the amplitude sink — three vector legs, so no
        // contact factor.
        let triple = LorentzTerm {
            coeff: 1.0,
            ops: vec![
                LorentzOp::P { mu: 2, leg: 0 },
                LorentzOp::Metric { mu: 0, nu: 1 },
            ],
        };
        let rooted = root_term(&triple, &[3, 3, 3], &[], None, &[None; 3]).unwrap();
        assert_eq!(
            rooted.build_sign, 1,
            "a triple-vector vertex takes no contact −1"
        );
    }

    #[test]
    fn test_root_vvs_metric_scalar_out() {
        // Same VVS1 Metric(0,1), now rooted at the *scalar* leg (idx 2): the two vectors
        // become inputs contracted by the Metric, and the scalar is the output current
        // (H produced from two vector chains). The pure-metric −1 fires just as it does
        // at the amplitude sink — this is the branch reached only by the 2→6 H classes in
        // the process suite, pinned here at the primitive level.
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![LorentzOp::Metric { mu: 0, nu: 1 }],
        };
        let spins = vec![3, 3, 1];
        let result = root_term(
            &term,
            &spins,
            &flow_of(&spins),
            Some(2),
            &[None, None, None],
        )
        .unwrap();
        assert_eq!(result.coeff, 1.0);
        assert_eq!(result.build_sign, -1);
        assert_eq!(result.reversed_sign, 1);
        assert_eq!(
            result.tree,
            LorentzEvalTree {
                nodes: vec![
                    LorentzEvalNode::Leg(0),
                    LorentzEvalNode::Leg(1),
                    LorentzEvalNode::Metric { mu: 0, nu: 1 },
                ],
                root: Some(2)
            }
        )
    }

    #[test]
    fn test_root_sss_scalar() {
        // SSS1: Empty ops (all scalars) rooted at amplitude → ScalarProduct
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![],
        };
        let spins = vec![1, 1, 1];
        let result = root_term(&term, &spins, &flow_of(&spins), None, &[]).unwrap();
        assert_eq!(
            result.tree,
            LorentzEvalTree {
                nodes: vec![
                    LorentzEvalNode::Leg(0),
                    LorentzEvalNode::Leg(1),
                    LorentzEvalNode::Leg(2),
                    LorentzEvalNode::Mul {
                        children: vec![0, 1, 2]
                    },
                ],
                root: Some(3)
            }
        )
    }

    /// SMEFTsim's cyclic Lorentz structures are exactly the seven tensor⊗tensor
    /// four-fermion ones, and each is recognised as two fermion lines joined by two
    /// summed Lorentz indices.
    ///
    /// Two statements at once. The census — which structures of the model have a cyclic
    /// index graph — is a claim about the model that the by-hand reading of
    /// `lorentz.py` could get wrong, so it is machine-checked here rather than asserted
    /// in prose. And the recognised decomposition is pinned per structure: which legs
    /// each line joins, and whether the two lines traverse the shared indices in the
    /// same order (`γ^αγ^β` against `γ_αγ_β`) or opposite ones. The order is the sign of
    /// the grade-2 half of the contact, so getting it wrong on one structure of a vertex
    /// and right on another is a real failure mode; `FFFF8`/`FFFF21` sit on one side of
    /// that split and the other five on the other, in one vertex.
    #[test]
    fn smeftsims_cyclic_structures_are_the_tensor_four_fermion_ones() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let parsed = crate::ufo::ParsedModel::parse(
            &root.join("../validation/ufo/SMEFTsim_topU3l_MwScheme_UFO"),
        )
        .expect("parse SMEFTsim");

        /// Structure name, whether the two lines run the shared indices in opposite
        /// orders, and the `(row leg, column leg)` of each line.
        type Recognised = (String, bool, Vec<(usize, usize)>);
        let mut seen: Vec<Recognised> = Vec::new();
        for structure in parsed.lorentz.values() {
            let mut cyclic = false;
            for term in structure.expr.iter() {
                let recognised = cyclic_tensor_term(term, &structure.spins)
                    .unwrap_or_else(|e| panic!("{}: {e}", structure.name));
                let Some(tensor) = recognised else { continue };
                cyclic = true;
                let lines: Vec<(usize, usize)> = tensor
                    .lines
                    .iter()
                    .map(|l| (l.row_leg, l.col_leg))
                    .collect();
                seen.push((structure.name.clone(), tensor.reversed_order, lines));
            }
            assert_eq!(
                cyclic,
                structure.expr.iter().any(cyclic_index_graph),
                "{}: recognition and the cycle test disagree",
                structure.name
            );
        }
        seen.sort();

        // Every one is a single term, so one entry per structure. `(row, column)` is the
        // vertex's own (bra slot, ket slot) orientation, 0-based; the lines come out in
        // ascending row-leg order, so every structure of the model pairs `(1,0)` with
        // `(3,2)` and only the traversal order distinguishes them.
        assert_eq!(
            seen,
            vec![
                ("FFFF19".to_string(), true, vec![(1, 0), (3, 2)]),
                ("FFFF20".to_string(), true, vec![(1, 0), (3, 2)]),
                ("FFFF21".to_string(), false, vec![(1, 0), (3, 2)]),
                ("FFFF5".to_string(), true, vec![(1, 0), (3, 2)]),
                ("FFFF6".to_string(), true, vec![(1, 0), (3, 2)]),
                ("FFFF7".to_string(), true, vec![(1, 0), (3, 2)]),
                ("FFFF8".to_string(), false, vec![(1, 0), (3, 2)]),
            ]
        );
    }

    /// A cycle the tensor path does not cover is refused by name, not walked into.
    ///
    /// The recognised shape is narrow on purpose (four fermion legs, two adjacent
    /// `Gamma` factors per line, nothing else); this is the other half of that
    /// statement — a term whose two lines are joined by a `Metric` rather than by their
    /// own gammas closes the same 4-cycle and is not evaluable this way.
    #[test]
    fn an_unrecognised_cycle_is_refused_by_name() {
        // Two γγ lines whose shared indices meet through a pair of metrics rather than
        // directly: `Gamma(-1,2,-5)*Gamma(-2,-5,1)*Gamma(-3,4,-6)*Gamma(-4,-6,3)
        //           *Metric(-1,-3)*Metric(-2,-4)`.
        let term = LorentzTerm {
            coeff: 1.0,
            ops: vec![
                LorentzOp::Gamma {
                    mu: -1,
                    i: 1,
                    j: -5,
                },
                LorentzOp::Gamma {
                    mu: -2,
                    i: -5,
                    j: 0,
                },
                LorentzOp::Gamma {
                    mu: -3,
                    i: 3,
                    j: -6,
                },
                LorentzOp::Gamma {
                    mu: -4,
                    i: -6,
                    j: 2,
                },
                LorentzOp::Metric { mu: -1, nu: -3 },
                LorentzOp::Metric { mu: -2, nu: -4 },
            ],
        };
        assert!(cyclic_index_graph(&term), "the probe term must be cyclic");
        let err = cyclic_tensor_term(&term, &[2, 2, 2, 2]).unwrap_err();
        assert!(
            format!("{err}").contains("outside the fermion-line set"),
            "unexpected refusal: {err}"
        );
    }
}
