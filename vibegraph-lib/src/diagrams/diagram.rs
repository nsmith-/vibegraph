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
#[derive(Clone, Debug)]
pub struct Leg {
    pub particle: ParticleId,
    pub charge: Charge,
    /// UFO spin code (2S+1).
    pub spin: i32,
    pub leg_idx: LegIdx,
    pub incoming: bool,
}

/// An internal propagator. Momentum flows `endpoints[0] → endpoints[1]`.
#[derive(Clone, Debug)]
pub struct Prop {
    pub particle: ParticleId,
    /// The two `(vertex, ray-slot)` endpoints this line connects.
    pub endpoints: [(VtxIdx, RaySlot); 2],
    /// Signed combination of external momenta (entry `i` = coefficient of external `i`).
    pub momentum: Vec<i8>,
}

/// A directed half-edge attached to a vertex, in UFO particle-slot order.
#[derive(Clone, Copy, Debug)]
pub enum Ray {
    /// An external leg.
    Leg(LegIdx),
    /// An internal propagator, `end` selecting which of its `endpoints` this vertex is
    /// (`0` = momentum-out, `1` = momentum-in).
    Prop { prop: PropIdx, end: usize },
}

/// An internal vertex: its UFO interaction and its rays in interaction-slot order.
#[derive(Clone, Debug)]
pub struct Vertex {
    pub interaction: VertexId,
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
    /// Relative Fermi permutation sign (feyngraph `view.sign()`).
    pub sign: i8,
    /// Combined vertex × propagator symmetry factor (feyngraph `view.symmetry_factor()`).
    pub symmetry_factor: usize,
    /// Number of incoming external legs.
    pub n_in: usize,
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
            let interaction = model.vertex_id(vtx.interaction().name()).ok_or_else(|| {
                ConvertError::VertexNotFound(vtx.interaction().name().to_string())
            })?;
            let mut rays = Vec::new();
            for (slot, ray) in vtx.propagators_ordered().enumerate() {
                match ray {
                    Either::Left(leg) => {
                        let li = leg.index();
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
            vertices.push(Vertex { interaction, rays });
        }

        let legs = legs
            .into_iter()
            .map(|l| l.expect("every external leg attaches to a vertex"))
            .collect();

        Ok(Diagram {
            legs,
            props,
            vertices,
            sign: view.sign(),
            symmetry_factor: view.symmetry_factor(),
            n_in,
        })
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

    /// The `(vertex, ray-slot)` where an external leg attaches. Every leg attaches to
    /// exactly one vertex.
    #[cfg(test)]
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
