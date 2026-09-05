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
    // TODO: Sigma, Epsilon
}

impl LorentzEvalNode {
    pub fn children(&self) -> Vec<usize> {
        match self {
            LorentzEvalNode::Leg(_) => vec![],
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
        }
    }
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
                    let child_mu =
                        self.build_child(term, *mu, visited_ops, flows, out_adjoint, sign)?;
                    let child_f =
                        self.build_child(term, other, visited_ops, flows, out_adjoint, sign)?;
                    let node = match out_adjoint {
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
            LorentzOp::Gamma5 { .. } => Err(RootLorentzError::UnsupportedVertex(
                "the chirality matrix Gamma5 is deferred to future work".to_string(),
            )),
            LorentzOp::Sigma { .. } => Err(RootLorentzError::UnsupportedVertex(
                "Sigma tensors are deferred to future work".to_string(),
            )),
            LorentzOp::Epsilon { .. } => Err(RootLorentzError::UnsupportedVertex(
                "Epsilon tensors are deferred to future work".to_string(),
            )),
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
        idx: Option<usize>,
        flows: &[Option<LegAdjoint>],
    ) -> Result<(Self, f64, i8), RootLorentzError> {
        let out_adjoint = idx.and_then(|i| flows.get(i).copied().flatten().map(|lf| lf.adjoint));
        let idx = correct_spin_index_for_flow(spins, idx, out_adjoint)?;
        let reversed_parity = term_reversed_parity(term, idx, flows);
        let mut tree = LorentzEvalTree {
            nodes: vec![],
            root: None,
        };
        let mut sign = 1.0;
        let mut visited_ops = Vec::new(); // LorentzOp is so small that Vec is probably better than HashSet
        let mut term_roots = Vec::new();
        // Whether this term's once-per-vertex pure-metric −1 has been applied;
        // guards against double application for structures with several Metric
        // ops (VVVV).
        let mut metric_vertex_applied = false;

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
                    // A pure-metric structure (VVS/VVSS, or the propagator-free
                    // 4-vector contact VVVV) carries an explicit −1 vertex factor,
                    // once per term (Gamma-/P-carrying structures — FFV, VVV —
                    // contract plainly). The sign holds whether the contraction
                    // sinks into the *amplitude* (the VVVV contact term, which has
                    // no propagator on its line) or into a scalar output leg (the
                    // H-current from two Z chains, −1 against the −i/D scalar
                    // propagator): both are pinned per-diagram against MadGraph
                    // AMP() — gg→gg for the amplitude root, the uux 2→6 and b b̄ 2→6
                    // H classes for the output-leg root.
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

        // The output leg's wavefunction is never referenced by an off-shell current,
        // so its position is a hole in the input-leg numbering. Compact the leg
        // references by dropping that hole: every input leg above `out` shifts down
        // by one, so `Leg(i)`/`P{leg}` index directly into the caller's gap-free
        // input list (vertex legs in order, output omitted) with no per-eval
        // reindexing. A `P` *can* reference the output leg's momentum (e.g. VVV1);
        // it becomes the leg-less `POut`, evaluated from the input currents.
        if let Some(out) = idx {
            for node in &mut tree.nodes {
                match node {
                    LorentzEvalNode::Leg(i) if *i > out => *i -= 1,
                    LorentzEvalNode::P { leg } if *leg == out => *node = LorentzEvalNode::POut,
                    LorentzEvalNode::P { leg } if *leg > out => *leg -= 1,
                    _ => {}
                }
            }
        }

        Ok((tree, sign, reversed_parity))
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
/// UFO spinor pairs are ordered (column/ket slot, row/bra slot): for `ψ̄₂Γψ₁`
/// (e.g. SM FFV `Gamma(3,2,1)` with particles (ℓ⁺, ℓ⁻, V)) the pair-first leg is
/// the column the ket contracts into, the pair-second leg the row for the bra.
/// An off-shell output at the column leg leaves `ψ̄Γ` — a bra (`Adjoint::Bra`); an
/// output at the row leg leaves `Γψ` — a ket (`Adjoint::Ket`). When the baked adjoint
/// disagrees with the rooted slot (the line traverses the vertex against its UFO
/// arrow), re-root at the adjoint-matching slot so the chiral projector lands on the
/// physical side of the gamma (ket: `ε̸·P_χ·ψ`, bra: `ψ̄·ε̸·P_χ`); the leg
/// compaction in `build_at_leg` keeps the caller's child binding unchanged.
///
/// spins: 2s+1 convention
fn correct_spin_index_for_flow(
    spins: &[i32],
    idx: Option<usize>,
    adjoint: Option<Adjoint>,
) -> Result<Option<usize>, RootLorentzError> {
    match (idx, adjoint) {
        (Some(idx), Some(f)) => {
            // The index should be the first of a spin pair (column slot) if the adjoint
            // is outgoing and the second (row slot) if the adjoint is incoming.

            let mut current_pair = (None, None);
            for (i, s) in spins.iter().enumerate() {
                if *s != 2 {
                    continue;
                }
                if current_pair.0.is_none() {
                    current_pair.0 = Some(i);
                } else if current_pair.1.is_none() {
                    current_pair.1 = Some(i);
                    if current_pair.0 == Some(idx) || current_pair.1 == Some(idx) {
                        // We found this pair
                        break;
                    }
                } else {
                    current_pair = (Some(i), None);
                }
            }
            match (current_pair, f) {
                // Correct pairing of spin index and adjoint
                ((Some(i), Some(_)), Adjoint::Bra) if i == idx => Ok(Some(i)),
                ((Some(_), Some(j)), Adjoint::Ket) if j == idx => Ok(Some(j)),
                // Incorrect pairing of spin index and adjoint, fix by swapping the indices
                ((Some(i), Some(j)), Adjoint::Bra) if j == idx => Ok(Some(i)),
                ((Some(i), Some(j)), Adjoint::Ket) if i == idx => Ok(Some(j)),
                _ => unreachable!("One of the spin indices must match the given index"),
            }
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

/// Resolve a single LorentzTerm into a rooted primitive with the output leg fixed.
///
/// # Arguments
/// * `term` — The UFO LorentzTerm to resolve.
/// * `spins` — Spin codes [1, 2, 3] for each leg
/// * `result_leg_idx` — The output leg (0-indexed), or `None` for amplitude (scalar sink).
/// * `out_adjoint` — Spinor adjoint of the output leg (`Some` iff a fermion output), used to
///   pick the in/out gamma routine.
///
/// # Returns
/// A `RootedTerm` ready for evaluation, or a `RootLorentzError`.
pub fn root_term(
    term: &crate::ufo::lorentz::LorentzTerm,
    spins: &[i32],
    result_leg_idx: Option<usize>,
    flows: &[Option<LegAdjoint>],
) -> Result<RootedTerm, RootLorentzError> {
    let (tree, sign, reversed_sign) =
        LorentzEvalTree::build_at_leg(term, spins, result_leg_idx, flows)?;
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
        let (t0, _, _) = LorentzEvalTree::build_at_leg(&term, &spins, Some(0), &[]).unwrap();
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
        let (t1, _, _) = LorentzEvalTree::build_at_leg(&term, &spins, Some(1), &[]).unwrap();
        assert!(
            max_leg(&t1).is_some_and(|m| m < n_inputs),
            "VVS rooted at leg 1 must only index the gap-free inputs: {t1:?}"
        );

        // Output = scalar leg 3 (idx 2): unchanged — a Metric contraction → scalar H.
        let (t2, _, _) = LorentzEvalTree::build_at_leg(&term, &spins, Some(2), &[]).unwrap();
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
            root_term(&term, &spins, Some(0), &[]),
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
        let result = root_term(&term, &spins, None, &[]).unwrap();
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
        let result = root_term(&term, &spins, None, &[]).unwrap();
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
            let result = root_term(&term, &spins, Some(requested), &flows_in).unwrap();
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
            let result = root_term(&term, &spins, Some(requested), &flows_out).unwrap();
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
        let result = root_term(&term, &spins, None, &[]).unwrap();
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
        let result = root_term(&term, &spins, None, &[]).unwrap();
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
        let result = root_term(&term, &spins, Some(2), &[None, None, None]).unwrap();
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
        let result = root_term(&term, &spins, None, &[]).unwrap();
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
}
