//! Vibegraph-owned diagram representation.
//!
//! A tree Feynman diagram is an undirected pseudograph. feyngraph generates the topology
//! and exposes it through borrowed `DiagramView`s whose conventions are implicit:
//! outgoing legs are presented in the all-incoming crossing, particle vs. antiparticle
//! lives in an `is_anti` flag, and each vertex's rays must be re-ordered to the UFO
//! interaction's particle-slot order (`propagators_ordered`).
//!
//! [`Diagram::from_view`] is the single boundary where those conventions are read: it
//! produces a UFO-resolved, `feyngraph`-free owned copy so downstream rooting
//! ([`crate::helas::eval`]) never touches a feyngraph view. Every positional index is a
//! distinct newtype so a leg index can't be used where a ray slot is expected.
//!
//! **Directedness.** Three orientations live on the pseudograph, and they are distinct:
//! momentum flow (intrinsic, fixed once the external convention is chosen — feyngraph
//! commits to it), the fermion-number arrow (intrinsic, from particle content), and the
//! evaluation direction the rooting pass imposes toward an arbitrary root. Momentum is the
//! one carried here: each [`Prop`] records the momentum it carries (as a signed
//! external-momentum combination) with the convention that momentum flows
//! `endpoints[0] → endpoints[1]`, giving each half-edge ([`Ray`]) a natural direction
//! (`endpoints[1]` is momentum-in). This does **not** replace rooting — it makes the
//! momentum-routing convention explicit data rather than something reconstructed later.

use std::collections::HashMap;

use feyngraph::diagram::view::{DiagramView, LegView};
use itertools::Either;
use thiserror::Error;

use crate::helas::repr::numbers::Charge;
use crate::ufo::particles::ParticleId;
use crate::ufo::vertices::VertexId;
use crate::ufo::UFOModel;

// ── Newtype indices ─────────────────────────────────────────────────────────────

/// External leg, in `0..n_ext` with incoming legs first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LegIdx(pub usize);
/// A vertex within one diagram.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VtxIdx(pub usize);
/// An internal propagator within one diagram.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PropIdx(pub usize);
/// The position of a ray within one vertex, in UFO interaction particle-slot order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RaySlot(pub usize);

// ── Owned diagram ───────────────────────────────────────────────────────────────

/// An external leg, resolved and crossing-baked.
///
/// `particle`/`charge` are as seen *from the attached vertex* (feyngraph's all-incoming
/// convention: an outgoing leg carries its crossed antiparticle here). `incoming` is the
/// momentum-flow direction — equivalently `leg_idx.0 < n_in`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Leg {
    pub particle: ParticleId,
    pub charge: Charge,
    /// UFO spin code (2S+1).
    pub spin: i32,
    pub leg_idx: LegIdx,
    pub incoming: bool,
}

/// An internal propagator. Momentum flows `endpoints[0] → endpoints[1]`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Prop {
    pub particle: ParticleId,
    /// The two `(vertex, ray-slot)` endpoints this line connects.
    pub endpoints: [(VtxIdx, RaySlot); 2],
    /// Signed combination of external momenta (entry `i` = coefficient of external `i`).
    pub momentum: Vec<i8>,
}

impl Prop {
    /// Whether this propagator is spacelike (t-channel) for an `n_in`-beam process:
    /// exactly one of the two initial-state beams flows through it. Read directly off
    /// the baked momentum — the beams are externals `0..n_in`, so the line separates
    /// them iff exactly one of those coefficients is nonzero. Only meaningful for
    /// 2→n (`n_in == 2`); timelike (s-channel) lines carry both beams or neither.
    pub fn is_spacelike(&self, n_in: usize) -> bool {
        n_in == 2 && self.momentum[..n_in].iter().filter(|&&c| c != 0).count() == 1
    }
}

/// A directed half-edge attached to a vertex, in UFO particle-slot order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ray {
    /// An external leg.
    Leg(LegIdx),
    /// An internal propagator, `end` selecting which of its `endpoints` this vertex is
    /// (`0` = momentum-out, `1` = momentum-in).
    Prop { prop: PropIdx, end: usize },
}

/// An internal vertex: its UFO interaction and its rays in interaction-slot order.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Vertex {
    pub interaction: VertexId,
    /// Which of the interaction's fermion-flow groups
    /// ([`topo::flow_groups`](crate::ufo::topo::flow_groups)) this occurrence uses:
    /// the index of the spinor pairing its Lorentz structures share. A vertex whose
    /// structures agree on one pairing — every vertex with fewer than four fermion
    /// legs, and every Standard-Model vertex — has only group `0`.
    pub flow_group: usize,
    pub rays: Vec<Ray>,
}

/// A single UFO-resolved, convention-baked Feynman diagram.
#[derive(Clone, Debug)]
pub struct Diagram {
    /// External legs, indexed by [`LegIdx`]; incoming first.
    pub legs: Vec<Leg>,
    /// Internal propagators, indexed by [`PropIdx`].
    pub props: Vec<Prop>,
    /// Internal vertices, indexed by [`VtxIdx`].
    pub vertices: Vec<Vertex>,
    /// The diagram's complete relative Fermi sign: the parity of the permutation that
    /// takes the external fermions, paired by line, to their leg order (feyngraph's
    /// `view.sign()`), times the per-line [`fermion_line_sign`](Self::fermion_line_sign).
    /// Both are properties of the graph.
    pub sign: i8,
    /// Combined vertex × propagator symmetry factor (feyngraph `view.symmetry_factor()`).
    pub symmetry_factor: usize,
    /// Number of incoming external legs.
    pub n_in: usize,
}

/// One fermion line of a diagram ([`Diagram::fermion_lines`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FermionLine {
    /// The external legs the line starts and ends at, the lower index first.
    pub legs: [LegIdx; 2],
    /// Every vertex the line passes through, from `legs[0]`'s to `legs[1]`'s. One more
    /// than the number of internal fermion propagators on the line.
    pub vertices: Vec<VtxIdx>,
}

/// A [`Diagram`] under its canonical internal numbering ([`Diagram::canonical`]).
///
/// Equality and hashing compare every field of the renumbered diagram — legs,
/// propagators with their particles, endpoints, slots and momenta, vertices with their
/// interaction, flow group and slot-ordered rays, the sign and the symmetry factor — so
/// two canonical forms are equal exactly when the diagrams are equal up to renumbering
/// their internal vertices and propagators. Slot order within a vertex is part of the
/// identity: it is the UFO interaction's particle order, so two diagrams that bind the
/// same lines to a vertex's slots in different orders are different diagrams here even
/// where the vertex is symmetric under the swap.
#[derive(Clone, Debug)]
pub struct CanonicalDiagram(Diagram);

impl CanonicalDiagram {
    /// The canonically numbered diagram.
    pub fn diagram(&self) -> &Diagram {
        &self.0
    }

    fn key(&self) -> (&[Leg], &[Prop], &[Vertex], i8, usize, usize) {
        let d = &self.0;
        (
            &d.legs,
            &d.props,
            &d.vertices,
            d.sign,
            d.symmetry_factor,
            d.n_in,
        )
    }
}

impl PartialEq for CanonicalDiagram {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Eq for CanonicalDiagram {}

impl std::hash::Hash for CanonicalDiagram {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().hash(state);
    }
}

/// Errors from translating a feyngraph view into an owned [`Diagram`]. This is pure
/// translation (no rooting), so these are distinct from `helas::eval`'s rooting errors.
#[derive(Clone, Debug, Error)]
pub enum ConvertError {
    /// A leg or propagator particle name is absent from the UFO model.
    #[error("particle not found in model: {0}")]
    ParticleNotFound(String),
    /// A vertex's interaction name is absent from the UFO model.
    #[error("vertex not found in model: {0}")]
    VertexNotFound(String),
    /// feyngraph's `is_anti` flag disagrees with the model's pdg-code sign.
    #[error(
        "antiparticle flag mismatch for {name}: feyngraph is_anti={is_anti}, model pdg_code={pdg}"
    )]
    AntiparticleMismatch {
        name: String,
        is_anti: bool,
        pdg: i64,
    },
    /// A particle whose model gives it a `propagators.py` form of its own is on an
    /// internal line. The evaluator builds every propagator from the default UFO
    /// forms, so honouring the custom one is out of reach and quietly using the
    /// default would be wrong physics.
    #[error(
        "particle '{particle}' propagates in this diagram with the custom propagator \
         '{propagator}' from propagators.py, which this evaluator does not implement"
    )]
    CustomPropagator {
        particle: String,
        propagator: String,
    },
}

impl Diagram {
    /// Convert a feyngraph [`DiagramView`] into an owned diagram, resolving every particle
    /// and interaction against `model` and baking the crossing/ordering/momentum
    /// conventions. The single point where a feyngraph view is consumed.
    pub fn from_view(view: &DiagramView, model: &UFOModel) -> Result<Diagram, ConvertError> {
        let n_in = view.incoming().count();
        let n_ext = view.n_ext();

        // Propagators get a stable PropIdx from `view.propagators()`; alongside them build
        // a `(vertex, ordered slot) → (prop, endpoint)` map so vertex rays can reference
        // them by index. The key is unique (one ray per vertex slot), so this is safe for
        // multi-edges and self-loops that the raw pseudograph may contain.
        let mut props = Vec::new();
        let mut slot_to_prop: HashMap<(usize, usize), (PropIdx, usize)> = HashMap::new();
        for (pi, p) in view.propagators().enumerate() {
            let pid = resolve_particle(model, p.particle().name())?;
            let (v0, s0) = (p.vertex(0).id(), p.ray_index_ordered(0));
            let (v1, s1) = (p.vertex(1).id(), p.ray_index_ordered(1));
            slot_to_prop.insert((v0, s0), (PropIdx(pi), 0));
            slot_to_prop.insert((v1, s1), (PropIdx(pi), 1));
            if let Some(propagator) = &model.particle(pid).propagator {
                return Err(ConvertError::CustomPropagator {
                    particle: model.particle(pid).name.clone(),
                    propagator: propagator.clone(),
                });
            }
            props.push(Prop {
                particle: pid,
                endpoints: [(VtxIdx(v0), RaySlot(s0)), (VtxIdx(v1), RaySlot(s1))],
                momentum: p.momentum(),
            });
        }

        // Vertices (rays in interaction-slot order) and legs, populated together: an
        // external leg attaches to exactly one vertex, so it is resolved the first (only)
        // time its ray is seen — using the vertex-perspective particle, matching feyngraph's
        // all-incoming crossing.
        let mut vertices = Vec::with_capacity(view.vertices().count());
        let mut legs: Vec<Option<Leg>> = vec![None; n_ext];
        for vtx in view.vertices() {
            let (base, flow_group) = crate::ufo::topo::split_flow_group(vtx.interaction().name());
            let interaction = model
                .vertex_id(base)
                .ok_or_else(|| ConvertError::VertexNotFound(base.to_string()))?;
            let mut rays = Vec::new();
            for (slot, ray) in vtx.propagators_ordered().enumerate() {
                match ray {
                    Either::Left(leg) => {
                        let li = leg.id();
                        rays.push(Ray::Leg(LegIdx(li)));
                        if legs[li].is_none() {
                            legs[li] = Some(make_leg(model, &leg, li, n_in)?);
                        }
                    }
                    Either::Right(_) => {
                        let &(prop, end) = slot_to_prop
                            .get(&(vtx.id(), slot))
                            .expect("every internal ray slot was mapped from view.propagators()");
                        rays.push(Ray::Prop { prop, end });
                    }
                }
            }
            vertices.push(Vertex {
                interaction,
                flow_group,
                rays,
            });
        }

        let legs = legs
            .into_iter()
            .map(|l| l.expect("every external leg attaches to a vertex"))
            .collect();

        let mut diagram = Diagram {
            legs,
            props,
            vertices,
            sign: view.sign(),
            symmetry_factor: view.symmetry_factor(),
            n_in,
        };
        diagram.sign *= diagram.fermion_line_sign(model);
        Ok(diagram)
    }

    /// Number of external legs.
    pub fn n_ext(&self) -> usize {
        self.legs.len()
    }

    /// External leg by index.
    pub fn leg(&self, idx: LegIdx) -> &Leg {
        &self.legs[idx.0]
    }

    /// Internal vertex by index.
    pub fn vertex(&self, idx: VtxIdx) -> &Vertex {
        &self.vertices[idx.0]
    }

    /// Internal propagator by index.
    pub fn prop(&self, idx: PropIdx) -> &Prop {
        &self.props[idx.0]
    }

    /// Whether the line at `ray` carries momentum flowing *into* this vertex. For an
    /// external leg this is its incoming flag; for a propagator, momentum flows
    /// `endpoints[0] → endpoints[1]`, so the `end == 1` endpoint is momentum-in.
    pub fn ray_momentum_in(&self, ray: Ray) -> bool {
        match ray {
            Ray::Leg(li) => self.leg(li).incoming,
            Ray::Prop { end, .. } => end == 1,
        }
    }

    /// The same diagram with its internal vertices and propagators renumbered.
    ///
    /// New vertex `i` is old vertex `vertex_order[i]`; new propagator `j` is old
    /// propagator `prop_order[j]`, with its orientation reversed when `flip[j]` (its
    /// endpoints swapped and its momentum negated, so the momentum it carries along the
    /// line is unchanged). Legs, rays' slot order, sign and symmetry factor are carried
    /// over, so the result is the same graph under another internal numbering.
    pub fn renumbered(
        &self,
        vertex_order: &[usize],
        prop_order: &[usize],
        flip: &[bool],
    ) -> Diagram {
        assert_eq!(vertex_order.len(), self.vertices.len());
        assert_eq!(prop_order.len(), self.props.len());
        assert_eq!(flip.len(), self.props.len());
        let mut new_vertex = vec![usize::MAX; self.vertices.len()];
        for (new, &old) in vertex_order.iter().enumerate() {
            new_vertex[old] = new;
        }
        let mut new_prop = vec![usize::MAX; self.props.len()];
        for (new, &old) in prop_order.iter().enumerate() {
            new_prop[old] = new;
        }
        assert!(new_vertex.iter().chain(&new_prop).all(|&i| i != usize::MAX));

        let props = prop_order
            .iter()
            .zip(flip)
            .map(|(&old, &flip)| {
                let p = &self.props[old];
                let mut endpoints = p.endpoints.map(|(v, slot)| (VtxIdx(new_vertex[v.0]), slot));
                let mut momentum = p.momentum.clone();
                if flip {
                    endpoints.swap(0, 1);
                    momentum.iter_mut().for_each(|c| *c = -*c);
                }
                Prop {
                    particle: p.particle,
                    endpoints,
                    momentum,
                }
            })
            .collect();
        let vertices = vertex_order
            .iter()
            .map(|&old| {
                let v = &self.vertices[old];
                Vertex {
                    interaction: v.interaction,
                    flow_group: v.flow_group,
                    rays: v
                        .rays
                        .iter()
                        .map(|&ray| match ray {
                            Ray::Leg(li) => Ray::Leg(li),
                            Ray::Prop { prop, end } => {
                                let p = new_prop[prop.0];
                                Ray::Prop {
                                    prop: PropIdx(p),
                                    end: if flip[p] { 1 - end } else { end },
                                }
                            }
                        })
                        .collect(),
                }
            })
            .collect();
        Diagram {
            legs: self.legs.clone(),
            props,
            vertices,
            sign: self.sign,
            symmetry_factor: self.symmetry_factor,
            n_in: self.n_in,
        }
    }

    /// The fermion lines of the diagram: per line, its two external legs and every vertex
    /// it passes through, in order from the first leg.
    ///
    /// A line is followed from each unvisited external fermion leg through each vertex's
    /// fermion pairing (the vertex's [`flow_group`](Vertex::flow_group) of
    /// [`flow_groups`](crate::ufo::topo::flow_groups)) until it leaves by another external
    /// leg; legs are taken in index order, so the list is a property of the graph.
    pub fn fermion_lines(&self, model: &UFOModel) -> Vec<FermionLine> {
        let pairing = |v: VtxIdx| {
            let vertex = self.vertex(v);
            crate::ufo::topo::flow_groups(model.vertex_def(vertex.interaction), &model.lorentz)
                .swap_remove(vertex.flow_group)
        };
        let mut visited = vec![false; self.legs.len()];
        let mut lines = Vec::new();
        for leg in &self.legs {
            if leg.spin.abs() != 2 || visited[leg.leg_idx.0] {
                continue;
            }
            let (mut vtx, RaySlot(mut slot)) = self.leg_attachment(leg.leg_idx);
            let mut vertices = Vec::new();
            // A tree line ends at a leg; the bound only guards against a malformed graph.
            let end = (0..=self.vertices.len())
                .find_map(|_| {
                    vertices.push(vtx);
                    match self.vertex(vtx).rays[pairing(vtx).partner(slot)] {
                        Ray::Leg(end) => Some(end),
                        Ray::Prop { prop, end } => {
                            (vtx, RaySlot(slot)) = self.prop(prop).endpoints[1 - end];
                            None
                        }
                    }
                })
                .expect("a fermion line of a tree diagram ends at an external leg");
            visited[leg.leg_idx.0] = true;
            visited[end.0] = true;
            lines.push(FermionLine {
                legs: [leg.leg_idx, end],
                vertices,
            });
        }
        lines
    }

    /// The relative fermion sign that the external-leg permutation parity leaves out,
    /// one factor per fermion line.
    ///
    /// It comes from which vertex slot each external wavefunction is bound to. Diagram
    /// enumeration binds slots in the *all-incoming* identity — an outgoing leg sits at
    /// its antiparticle's slot — while the HELAS bookkeeping the amplitudes are defined
    /// against binds them in the *all-outgoing* identity, where an *incoming* leg sits at
    /// its antiparticle's slot. A slot fixes a spinor adjoint (the pair-first slot takes
    /// the ket, the pair-second the bra), so the two bindings disagree exactly on the
    /// legs neither convention crosses the same way:
    ///
    /// * A line with at least one **initial-state** end is read against its own arrow at
    ///   every vertex (the initial leg always, and a mixed line's final leg because the
    ///   evaluator types it by its physical wavefunction while the slot stays the
    ///   all-incoming one). Reading a bilinear against its arrow replaces each vertex
    ///   structure by `C Γᵀ C⁻¹`, which for `Γ = γ^μ P_χ` is `−γ^μ P_χ̄`: the evaluator
    ///   applies the chirality flip per vertex and one of the `V` minus signs at the
    ///   line's single vector-rooted sink (its reversed-bilinear parity). The remaining
    ///   `V − 1` — one per internal fermion propagator on the line — are this factor.
    ///   Pinned by the `u u~ > c c~ e+ e- mu+ mu- QCD=0` per-diagram oracle for the
    ///   initial–initial case and by `u d > e+ e- u d QCD=0`, whose 35 diagrams split on
    ///   whether a *mixed* quark line carries the propagator, for the mixed case.
    ///
    ///   **A line whose every bilinear is Dirac-matrix-free takes none of it.**
    ///   `C Γᵀ C⁻¹` is `Γ` itself for `Identity`, `Gamma5` and the bare chiral
    ///   projectors, so a line built only from those — a chain of Yukawa-type vertices,
    ///   with no `Gamma` and no `Sigma` anywhere on it — reverses into itself and carries
    ///   no propagator sign at all. Measured on `qt qt~ > o8 o8` in the toy colour model,
    ///   whose s-channel (no fermion propagator) and t/u-channel (one) diagrams must
    ///   enter the JAMPs with the *same* sign to reproduce MadGraph's `|M|²`. A line that
    ///   carries a Dirac matrix anywhere keeps the propagator count, including the mixed
    ///   gauge/Yukawa case: the `b` line of `b b~ > c c~ e+ e- mu+ mu- QCD=0`, one photon
    ///   vertex and one `b b~ H` vertex across one propagator, is bit-for-bit against
    ///   MadGraph only with the −1. Which vertex on a mixed line owns which factor is not
    ///   resolved by any oracle in the suite — every measured case is decided by the
    ///   all-or-nothing form.
    /// * A **crossed** line — both ends final-state, kept in the all-incoming
    ///   (conjugate-wavefunction) representation — is read *along* its arrow at every
    ///   vertex, so it takes no per-propagator factor; its single −1 is the operator
    ///   reordering of the conjugated pair relative to the reference's physical pair.
    ///   Exposed by Bhabha, where the s-channel has one crossed line and the t-channel
    ///   none; its propagator-independence by `u d > e+ e- u d QCD=0`, where one
    ///   propagator on the crossed lepton line and one on a mixed quark line must give
    ///   opposite signs.
    ///
    /// Both arms read only the lines themselves — their ends, their propagator count and
    /// what their vertices carry — so this is a property of the diagram, not of how it
    /// is rooted or numbered.
    pub fn fermion_line_sign(&self, model: &UFOModel) -> i8 {
        let carries_dirac_matrix = |v: VtxIdx| {
            use crate::ufo::lorentz::LorentzOp;
            let vertex = self.vertex(v);
            let def = model.vertex_def(vertex.interaction);
            crate::ufo::topo::flow_groups(def, &model.lorentz)
                .swap_remove(vertex.flow_group)
                .lorentz
                .iter()
                .any(|&pos| {
                    model.lorentz_struct(def.lorentz[pos]).expr.iter().any(|t| {
                        t.ops.iter().any(|op| {
                            matches!(op, LorentzOp::Gamma { .. } | LorentzOp::Sigma { .. })
                        })
                    })
                })
        };
        let mut sign = 1i8;
        for line in self.fermion_lines(model) {
            let crossed = line.legs.iter().all(|l| l.0 >= self.n_in);
            let propagators = line.vertices.len() - 1;
            let flip = crossed
                || (propagators % 2 == 1 && line.vertices.iter().any(|&v| carries_dirac_matrix(v)));
            if flip {
                sign = -sign;
            }
        }
        sign
    }

    /// The diagram's anchor: of the vertices with the fewest rays, the first in canonical
    /// order ([`canonical`](Self::canonical)).
    ///
    /// On a diagram built only from three-point vertices this is the vertex external leg
    /// 0 attaches to; a lower-arity vertex elsewhere displaces a four-point vertex holding
    /// leg 0. The rule reads only the labelled external legs and slot order, so every
    /// numbering of one diagram has the same anchor. The per-diagram HELAS convention
    /// signs are defined at the rooting that takes the anchor as the amplitude vertex.
    pub fn anchor(&self) -> VtxIdx {
        let (order, _, _) = self.canonical_numbering();
        let fewest = self.vertices.iter().map(|v| v.rays.len()).min();
        let first = order
            .into_iter()
            .find(|&v| Some(self.vertices[v].rays.len()) == fewest)
            .expect("a diagram has a vertex");
        VtxIdx(first)
    }

    /// The diagram under its canonical internal numbering.
    ///
    /// Vertices are numbered in depth-first preorder from the vertex external leg 0
    /// attaches to, descending through each vertex's rays in slot order. Each propagator
    /// takes the number of the vertex it leads to, less one, and is oriented with
    /// `endpoints[0]` at the end the walk reaches first (its momentum negated where that
    /// reverses it). The walk reads only the labelled external legs and slot order, so
    /// every numbering of one diagram has the same canonical form, and two diagrams with
    /// the same canonical form differ only in numbering.
    pub fn canonical(&self) -> CanonicalDiagram {
        let (vertex_order, prop_order, flip) = self.canonical_numbering();
        CanonicalDiagram(self.renumbered(&vertex_order, &prop_order, &flip))
    }

    /// The [`canonical`](Self::canonical) numbering as [`renumbered`](Self::renumbered)
    /// arguments: vertices in walk order, propagators in the order the walk crosses them,
    /// and whether each is crossed against its orientation.
    fn canonical_numbering(&self) -> (Vec<usize>, Vec<usize>, Vec<bool>) {
        struct Walk<'a> {
            diagram: &'a Diagram,
            seen: Vec<bool>,
            vertex_order: Vec<usize>,
            prop_order: Vec<usize>,
            flip: Vec<bool>,
        }
        impl Walk<'_> {
            fn visit(&mut self, v: VtxIdx) {
                self.seen[v.0] = true;
                self.vertex_order.push(v.0);
                for &ray in &self.diagram.vertex(v).rays {
                    let Ray::Prop { prop, end } = ray else {
                        continue;
                    };
                    let (w, _) = self.diagram.prop(prop).endpoints[1 - end];
                    if self.seen[w.0] {
                        continue;
                    }
                    self.prop_order.push(prop.0);
                    self.flip.push(end == 1);
                    self.visit(w);
                }
            }
        }
        let mut walk = Walk {
            diagram: self,
            seen: vec![false; self.vertices.len()],
            vertex_order: Vec::with_capacity(self.vertices.len()),
            prop_order: Vec::with_capacity(self.props.len()),
            flip: Vec::with_capacity(self.props.len()),
        };
        walk.visit(self.leg_attachment(LegIdx(0)).0);
        assert!(
            walk.vertex_order.len() == self.vertices.len()
                && walk.prop_order.len() == self.props.len(),
            "a tree diagram is connected and every propagator lies on the walk"
        );
        (walk.vertex_order, walk.prop_order, walk.flip)
    }

    /// The `(vertex, ray-slot)` where an external leg attaches. Every leg attaches to
    /// exactly one vertex.
    pub fn leg_attachment(&self, target: LegIdx) -> (VtxIdx, RaySlot) {
        for (vi, v) in self.vertices.iter().enumerate() {
            for (slot, ray) in v.rays.iter().enumerate() {
                if let Ray::Leg(li) = ray {
                    if *li == target {
                        return (VtxIdx(vi), RaySlot(slot));
                    }
                }
            }
        }
        unreachable!("every external leg attaches to a vertex")
    }
}

fn resolve_particle(model: &UFOModel, name: &str) -> Result<ParticleId, ConvertError> {
    model
        .particle_id(name)
        .ok_or_else(|| ConvertError::ParticleNotFound(name.to_string()))
}

/// Resolve one external leg, validating feyngraph's `is_anti` against the model pdg sign.
///
/// Uses `pdg_code < 0` (not charge sign) because up-type quarks have positive charge yet
/// are particles (`is_anti = false`), which a charge-based check would misclassify.
fn make_leg(model: &UFOModel, leg: &LegView, li: usize, n_in: usize) -> Result<Leg, ConvertError> {
    let particle = leg.particle();
    let pid = resolve_particle(model, particle.name())?;
    let mp = model.particle(pid);
    if particle.is_anti() != (mp.pdg_code < 0) {
        return Err(ConvertError::AntiparticleMismatch {
            name: particle.name().to_string(),
            is_anti: particle.is_anti(),
            pdg: mp.pdg_code,
        });
    }
    Ok(Leg {
        particle: pid,
        charge: if particle.is_anti() {
            Charge::Antiparticle
        } else {
            Charge::Particle
        },
        spin: mp.spin,
        leg_idx: LegIdx(li),
        incoming: li < n_in,
    })
}

#[cfg(test)]
mod tests {
    use crate::diagrams::{
        generate_from_proc_card, parse_proc_card, ConvertError, DiagramError, ParsingOptions,
    };
    use crate::ufo::propagators::Propagator;
    use crate::ufo::sm::{sm_parsed_model, SMRestrict};

    /// A particle carrying a `propagators.py` form of its own is refused where it
    /// propagates, and only there.
    ///
    /// No model in reach reaches this refusal on its own — SMEFTsim's four
    /// width-corrected auxiliary fields are the only particles that carry a custom
    /// propagator, and no coverage-table process selects a diagram they appear in —
    /// so the refusal is pinned on a Standard Model deliberately given one. The `Z`
    /// is an internal line of `e+ e- > mu+ mu-` and only an external leg of
    /// `e+ e- > Z Z` (which goes through t- and u-channel electrons), which is the
    /// distinction the refusal turns on.
    #[test]
    fn a_custom_propagator_is_refused_where_it_propagates() {
        let mut parsed = sm_parsed_model();
        parsed.propagators.insert(
            "V1".to_owned(),
            Propagator {
                python_name: "V1".to_owned(),
                name: "V1".to_owned(),
                numerator: "- Metric(1, 2)".to_owned(),
                denominator: "P('mu', id) * P('mu', id)".to_owned(),
            },
        );
        parsed
            .particles
            .get_mut("Z")
            .expect("Z in the SM")
            .propagator = Some("V1".to_owned());
        let card = SMRestrict::Default
            .restrict_card_text()
            .parse()
            .expect("parse the interned SM restrict card");
        let model = parsed
            .into_model(Some(&card))
            .expect("build the SM with a custom Z propagator");

        let generate = |process: &str| {
            let pc = parse_proc_card(&format!("generate {process}"), &ParsingOptions::default())
                .unwrap();
            generate_from_proc_card(&pc, &model)
        };

        let err = match generate("e+ e- > mu+ mu-") {
            Err(e) => e,
            Ok(_) => panic!("a propagating Z must be refused"),
        };
        assert!(
            matches!(
                err,
                DiagramError::Convert(ConvertError::CustomPropagator { ref particle, .. })
                    if particle == "Z"
            ),
            "expected CustomPropagator(Z), got {err}"
        );

        // The same particle as an external leg is not propagating, so it is fine.
        let sets = generate("e+ e- > Z Z").expect("an external Z must not be refused");
        let diagrams: usize = sets.iter().map(|s| s.diagrams.len()).sum();
        assert!(diagrams > 0, "no diagrams to judge");
        for set in &sets {
            for diagram in &set.diagrams {
                for prop in &diagram.props {
                    assert_ne!(model.particle(prop.particle).name, "Z");
                }
            }
        }
    }

    /// The anchor is the vertex external leg 0 attaches to wherever every vertex has three
    /// legs, and a three-point vertex displaces a four-point contact holding leg 0.
    #[test]
    fn the_anchor_is_the_first_lowest_arity_vertex() {
        use crate::diagrams::diagram::LegIdx;
        use crate::ufo::sm::sm_model;

        let model = sm_model(SMRestrict::Default);
        let generate = |process: &str| {
            let pc = parse_proc_card(&format!("generate {process}"), &ParsingOptions::default())
                .unwrap();
            generate_from_proc_card(&pc, &model).unwrap()
        };
        for process in ["e+ e- > W+ W-", "g g > t t~", "u d > e+ e- u d QCD=0"] {
            for set in generate(process) {
                for d in &set.diagrams {
                    assert!(d.vertices.iter().all(|v| v.rays.len() == 3));
                    assert_eq!(d.anchor(), d.leg_attachment(LegIdx(0)).0, "{process}");
                }
            }
        }

        let mut displaced = 0;
        for set in generate("g g > g g g") {
            for d in &set.diagrams {
                let holder = d.leg_attachment(LegIdx(0)).0;
                let anchor = d.anchor();
                assert_eq!(
                    d.vertex(anchor).rays.len(),
                    3,
                    "g g > g g g has a VVV everywhere"
                );
                if d.vertex(holder).rays.len() == 4 {
                    assert_ne!(anchor, holder);
                    displaced += 1;
                } else {
                    assert_eq!(anchor, holder);
                }
            }
        }
        assert_eq!(
            displaced, 6,
            "the six contact diagrams with leg 0 on the contact"
        );
    }
}
