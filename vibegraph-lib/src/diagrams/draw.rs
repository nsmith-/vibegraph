//! Drawing a [`Diagram`] as SVG, through feyngraph's layout and renderer.
//!
//! feyngraph draws only its own diagrams, so each diagram is rebuilt as one
//! ([`feyngraph::diagram::Diagram::from_tree_parts`]): the inverse of
//! [`Diagram::from_view`]. The conventions are the ones `from_view` reads:
//!
//! - a leg's particle is the one seen from its vertex, so an outgoing leg carries
//!   its crossed antiparticle, which is what feyngraph stores for it;
//! - a propagator's particle flows `endpoints[0] → endpoints[1]` with its momentum,
//!   which is feyngraph's `vertices[0] → vertices[1]`, and its momentum combination
//!   (outgoing coefficients negated) is feyngraph's own;
//! - a vertex lists its lines as a propagator index or `leg - n_ext`, and names the
//!   feyngraph interaction of its UFO interaction and fermion-flow group.
//!
//! The drawing labels each leg with its physical particle and its index in the
//! diagram's own leg order, so a diagram whose legs were reordered or whose decays
//! were stitched on draws with the labels of the set it belongs to. No feyngraph
//! type leaves this module.

use std::collections::HashMap;

use feyngraph::diagram::{
    Diagram as FgDiagram, DiagramPartsError, Leg as FgLeg, Propagator as FgPropagator,
    Vertex as FgVertex,
};
use thiserror::Error;

use super::diagram::{antiparticle, Diagram, Ray};
use crate::ufo::particles::ParticleId;
use crate::ufo::UFOModel;

/// Why a diagram cannot be drawn.
#[derive(Debug, Error)]
pub enum DrawError {
    /// A particle of the diagram is not in the model's topology model (an
    /// unphysical particle, which no enumerated diagram carries).
    #[error("particle '{0}' has no line in the model's topology model")]
    Particle(String),
    /// A vertex's interaction and fermion-flow group name no interaction of the
    /// model's topology model.
    #[error(
        "interaction '{name}' (fermion-flow group {group}) is not in the model's topology model"
    )]
    Interaction { name: String, group: usize },
    /// A vertex's lines are not its interaction's particles, slot for slot, read
    /// all-incoming. feyngraph's drawing matches each line to an interaction
    /// particle by name and does not survive a vertex where none matches.
    #[error(
        "vertex {vertex} binds interaction '{interaction}' to the particles [{found}], not to \
         its own [{expected}]"
    )]
    VertexParticles {
        vertex: usize,
        interaction: String,
        found: String,
        expected: String,
    },
    /// feyngraph refuses the parts as a tree diagram.
    #[error("feyngraph refuses the diagram: {0}")]
    Parts(#[from] DiagramPartsError),
}

/// Draws the diagrams of one model.
///
/// Built once per model: it holds the particle and interaction index tables the
/// conversion reads.
pub struct DiagramDrawer<'m> {
    model: &'m UFOModel,
    /// feyngraph particle index of each [`ParticleId`], `None` for a particle the
    /// topology model leaves out.
    particles: Vec<Option<usize>>,
    /// feyngraph interaction index by feyngraph interaction name.
    interactions: HashMap<String, usize>,
}

impl<'m> DiagramDrawer<'m> {
    pub fn new(model: &'m UFOModel) -> Self {
        let particles = model
            .particles
            .keys()
            .map(|name| model.topo.get_particle_index(name).ok())
            .collect();
        let interactions = model
            .topo
            .vertices_iter()
            .enumerate()
            .map(|(i, v)| (v.name().to_string(), i))
            .collect();
        DiagramDrawer {
            model,
            particles,
            interactions,
        }
    }

    /// `diagram` as an SVG document.
    pub fn svg(&self, diagram: &Diagram) -> Result<String, DrawError> {
        let rebuilt = self.rebuild(diagram)?;
        let labels = Vec::new();
        Ok(rebuilt.view(&self.model.topo, &labels).draw_svg_string())
    }

    fn particle(&self, id: ParticleId) -> Result<usize, DrawError> {
        self.particles[id.index()]
            .ok_or_else(|| DrawError::Particle(self.model.particle(id).name.clone()))
    }

    /// The feyngraph interaction a vertex of `diagram` binds: the UFO interaction,
    /// suffixed `@<group>` when the model split it by fermion flow.
    fn interaction(&self, diagram: &Diagram, v: usize) -> Result<usize, DrawError> {
        let vertex = &diagram.vertices[v];
        let name = vertex.interaction_name(self.model);
        let group = vertex.flow_group;
        self.interactions
            .get(&format!("{name}@{group}"))
            .or_else(|| (group == 0).then(|| self.interactions.get(name)).flatten())
            .copied()
            .ok_or_else(|| DrawError::Interaction {
                name: name.to_string(),
                group,
            })
    }

    /// Refuse a vertex whose rays do not carry its interaction's particles in
    /// slot order: a leg its vertex-side particle, a propagator the particle
    /// flowing into the vertex (its own at `endpoints[1]`, the antiparticle at
    /// `endpoints[0]`).
    fn check_vertex_particles(&self, diagram: &Diagram, v: usize) -> Result<(), DrawError> {
        let vertex = &diagram.vertices[v];
        let found: Vec<Option<ParticleId>> = vertex
            .rays
            .iter()
            .map(|ray| match *ray {
                Ray::Leg(leg) => diagram.legs.get(leg.0).map(|l| l.particle),
                Ray::Prop { prop, end } => diagram.props.get(prop.0).map(|p| match end {
                    1 => p.particle,
                    _ => antiparticle(self.model, p.particle),
                }),
            })
            .collect();
        let expected = &self.model.vertex_def(vertex.interaction).particles;
        if found.len() == expected.len() && found.iter().zip(expected).all(|(f, e)| *f == Some(*e))
        {
            return Ok(());
        }
        let names = |ids: &mut dyn Iterator<Item = Option<ParticleId>>| {
            ids.map(|id| id.map_or("?", |id| self.model.particle(id).name.as_str()))
                .collect::<Vec<_>>()
                .join(", ")
        };
        Err(DrawError::VertexParticles {
            vertex: v,
            interaction: vertex.interaction_name(self.model).to_string(),
            found: names(&mut found.iter().copied()),
            expected: names(&mut expected.iter().map(|&e| Some(e))),
        })
    }

    /// `diagram` as feyngraph's parts, built into its diagram.
    fn rebuild(&self, diagram: &Diagram) -> Result<FgDiagram, DrawError> {
        for v in 0..diagram.vertices.len() {
            self.check_vertex_particles(diagram, v)?;
        }
        let n_ext = diagram.n_ext();
        let mut attached = vec![usize::MAX; n_ext];
        let mut vertices = Vec::with_capacity(diagram.vertices.len());
        for (v, vertex) in diagram.vertices.iter().enumerate() {
            let lines = vertex
                .rays
                .iter()
                .map(|ray| match *ray {
                    Ray::Leg(leg) => {
                        attached[leg.0] = v;
                        leg.0 as isize - n_ext as isize
                    }
                    Ray::Prop { prop, .. } => prop.0 as isize,
                })
                .collect();
            vertices.push(FgVertex {
                propagators: lines,
                interaction: self.interaction(diagram, v)?,
            });
        }
        let mut legs = diagram
            .legs
            .iter()
            .enumerate()
            .map(|(i, leg)| {
                let mut momentum = vec![0; n_ext];
                momentum[i] = 1;
                Ok(FgLeg {
                    // A leg no vertex lists is refused by feyngraph's incidence check;
                    // an out-of-range vertex index is what reaches it.
                    vertex: attached[i],
                    particle: self.particle(leg.particle)?,
                    momentum,
                })
            })
            .collect::<Result<Vec<_>, DrawError>>()?;
        let outgoing = legs.split_off(diagram.n_in.min(legs.len()));
        let propagators = diagram
            .props
            .iter()
            .map(|p| {
                Ok(FgPropagator {
                    vertices: [p.endpoints[0].0 .0, p.endpoints[1].0 .0],
                    particle: self.particle(p.particle)?,
                    momentum: p.momentum.clone(),
                })
            })
            .collect::<Result<Vec<_>, DrawError>>()?;
        Ok(FgDiagram::from_tree_parts(
            &self.model.topo,
            legs,
            outgoing,
            propagators,
            vertices,
        )?)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use feyngraph::diagram::view::DiagramView;
    use feyngraph::DiagramGenerator;
    use itertools::Either;

    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, DiagramSet, ParsingOptions};
    use crate::ufo::sm::{sm_model, SMRestrict};

    /// Everything a feyngraph view exposes about its diagram's graph, in the
    /// view's own numbering: per leg its particle and momentum, per propagator its
    /// particle, vertices and momentum, per vertex its interaction and its lines in
    /// interaction-slot order, and the sign and symmetry factor.
    fn anatomy(view: &DiagramView) -> String {
        let legs: Vec<String> = view
            .legs()
            .map(|l| {
                format!(
                    "{}{:?}@{}",
                    l.particle().name(),
                    l.momentum(),
                    l.vertex().id()
                )
            })
            .collect();
        let props: Vec<String> = view
            .propagators()
            .map(|p| {
                format!(
                    "{}{:?}:{}->{}",
                    p.particle().name(),
                    p.momentum(),
                    p.vertex(0).id(),
                    p.vertex(1).id()
                )
            })
            .collect();
        let vertices: Vec<String> = view
            .vertices()
            .map(|v| {
                let lines: Vec<String> = v
                    .propagators_ordered()
                    .map(|line| match line {
                        Either::Left(l) => format!("L{}", l.id()),
                        Either::Right(p) => format!("P{}{:?}", p.particle().name(), p.momentum()),
                    })
                    .collect();
                format!("{}[{}]", v.interaction().name(), lines.join(","))
            })
            .collect();
        format!(
            "legs {legs:?} props {props:?} vertices {vertices:?} sign {} symmetry {}",
            view.sign(),
            view.symmetry_factor()
        )
    }

    /// Every diagram feyngraph generates for `incoming > outgoing` under `model`,
    /// converted to a vibegraph [`Diagram`] and drawn back, must give feyngraph's
    /// own SVG byte for byte, and its rebuilt feyngraph diagram must have the same
    /// anatomy (sign included). Its [`Diagram::coupling_orders`] must also be
    /// feyngraph's own count of the view's orders. Returns the number of diagrams
    /// compared.
    fn assert_round_trip(model: &UFOModel, incoming: &[&str], outgoing: &[&str]) -> usize {
        let drawer = DiagramDrawer::new(model);
        let container = DiagramGenerator::new(incoming, outgoing, 0, model.topo.clone(), None)
            .unwrap()
            .generate();
        assert!(
            !container.is_empty(),
            "{incoming:?} > {outgoing:?}: no diagram"
        );
        let labels = Vec::new();
        for (k, view) in container.views().enumerate() {
            let diagram = Diagram::from_view(&view, model).unwrap();
            let orders: std::collections::BTreeMap<String, usize> =
                view.orders().into_iter().filter(|(_, n)| *n > 0).collect();
            assert_eq!(
                diagram.coupling_orders(model),
                orders,
                "{incoming:?} > {outgoing:?} diagram {k}: coupling orders"
            );
            let rebuilt = drawer.rebuild(&diagram).unwrap();
            let rebuilt_view = rebuilt.view(&model.topo, &labels);
            assert_eq!(
                anatomy(&rebuilt_view),
                anatomy(&view),
                "{incoming:?} > {outgoing:?} diagram {k}"
            );
            assert_eq!(rebuilt.sign(), view.sign());
            assert_eq!(
                drawer.svg(&diagram).unwrap(),
                view.draw_svg_string(),
                "{incoming:?} > {outgoing:?} diagram {k}: the SVG differs"
            );
        }
        container.len()
    }

    /// How many vertices of the diagrams feyngraph generates for `incoming >
    /// outgoing` bind an interaction the model split by fermion flow.
    fn split_vertex_uses(model: &UFOModel, incoming: &[&str], outgoing: &[&str]) -> usize {
        let container = DiagramGenerator::new(incoming, outgoing, 0, model.topo.clone(), None)
            .unwrap()
            .generate();
        container
            .views()
            .map(|view| {
                view.vertices()
                    .filter(|v| v.interaction().name().contains('@'))
                    .count()
            })
            .sum()
    }

    /// The conversion inverts [`Diagram::from_view`] exactly where drawing can
    /// see it: each enumerated diagram, converted to vibegraph's form and drawn
    /// through the rebuild, is feyngraph's own drawing of it to the byte, and the
    /// rebuilt diagram has feyngraph's legs, propagators, slot-ordered vertices,
    /// sign and symmetry factor.
    ///
    /// The processes cover crossed fermion lines (`u u~ > e+ e- g`), three- and
    /// four-gluon vertices with many propagators (`g g > g g g`), electroweak
    /// four-point vertices (`e+ e- > w+ w- z`), and SMEFTsim's four-fermion
    /// contacts, among them Bhabha's, whose interactions are split by fermion flow
    /// and named `<vertex>@<group>` in the topology model, and whose sign depends
    /// on which pairing a contact uses.
    ///
    /// The oracle is feyngraph's own view, so a conversion error cannot hide
    /// behind a matching error on the reference side. What it cannot see: a
    /// mistake in `from_view` that the conversion undoes exactly (for example a
    /// flow group or propagator orientation recorded wrongly in vibegraph's form
    /// and read back the same wrong way), since only the round trip is compared;
    /// the raw order in which feyngraph lists a vertex's lines, which vibegraph's
    /// form does not keep and no drawing reads; and diagrams that never had a
    /// feyngraph view (stitched, reordered), which only the tests below cover.
    #[test]
    fn drawing_a_converted_diagram_is_feyngraphs_drawing_of_it() {
        let sm = sm_model(SMRestrict::Default);
        let mut checked = 0;
        checked += assert_round_trip(&sm, &["u", "u~"], &["e+", "e-", "g"]);
        checked += assert_round_trip(&sm, &["g", "g"], &["g", "g", "g"]);
        checked += assert_round_trip(&sm, &["e+", "e-"], &["W+", "W-", "Z"]);

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../validation/ufo/SMEFTsim_topU3l_MwScheme_UFO");
        let smeft = UFOModel::load(&dir, Some(&dir.join("restrict_massless.dat"))).unwrap();
        let split = smeft
            .topo
            .vertices_iter()
            .filter(|v| v.name().contains('@'))
            .count();
        assert!(split > 0, "SMEFTsim has no fermion-flow-split interaction");
        checked += assert_round_trip(&smeft, &["e+", "e-"], &["mu+", "mu-"]);
        checked += assert_round_trip(&smeft, &["e+", "e-"], &["e+", "e-"]);
        assert!(split_vertex_uses(&smeft, &["e+", "e-"], &["e+", "e-"]) > 0);
        checked += assert_round_trip(&smeft, &["u", "u~"], &["t", "t~"]);
        assert!(checked > 100, "only {checked} diagrams compared");
    }

    /// `diagram`'s drawing labels leg `i` with the physical particle `names[i]`,
    /// once each.
    fn assert_leg_labels(svg: &str, names: &[String]) {
        for (i, name) in names.iter().enumerate() {
            let label = format!(">{name}({i})</text>");
            assert_eq!(svg.matches(&label).count(), 1, "{label} in\n{svg}");
        }
    }

    fn names(set: &DiagramSet) -> Vec<String> {
        set.particles_in
            .iter()
            .chain(&set.particles_out)
            .cloned()
            .collect()
    }

    fn sets(card: &str) -> Vec<DiagramSet> {
        let card = parse_proc_card(card, &ParsingOptions::default()).unwrap();
        generate_from_proc_card(&card, &sm_model(SMRestrict::Default)).unwrap()
    }

    /// Stitched decay-chain diagrams have no feyngraph view of their own; they
    /// draw without error, each leg labelled with its set's particle at its index.
    ///
    /// Cannot detect a wrong internal line or vertex in the drawing, only the
    /// external labels: the layout is not compared against anything.
    #[test]
    fn stitched_decay_chain_diagrams_draw_with_their_set_labels() {
        let model = sm_model(SMRestrict::Default);
        let drawer = DiagramDrawer::new(&model);
        let sets = sets("generate e+ e- > z h, z > mu+ mu-, h > b b~");
        let mut drawn = 0;
        for set in &sets {
            assert!(set.diagrams.iter().all(|d| !d.provenance.decays.is_empty()));
            for d in &set.diagrams {
                assert_leg_labels(&drawer.svg(d).unwrap(), &names(set));
                drawn += 1;
            }
        }
        assert!(drawn > 0);
    }

    /// A set whose outgoing legs were reordered draws each diagram with the
    /// reordered labels: leg `i` carries the reordered set's particle `i`, and the
    /// drawing differs from the original's wherever two reordered legs carry
    /// different particles.
    ///
    /// Cannot detect a layout error that keeps the labels right.
    #[test]
    fn reordered_sets_draw_with_their_own_leg_order() {
        let model = sm_model(SMRestrict::Default);
        let drawer = DiagramDrawer::new(&model);
        let original = &sets("generate u u~ > e+ e- g")[0];
        let reordered = original.with_final_order(&[2, 0, 1], &model).unwrap();
        assert_eq!(reordered.particles_out, ["g", "e+", "e-"]);
        for (a, b) in original.diagrams.iter().zip(&reordered.diagrams) {
            let svg = drawer.svg(b).unwrap();
            assert_leg_labels(&svg, &names(&reordered));
            assert_ne!(svg, drawer.svg(a).unwrap());
        }

        let chain = sets("generate e+ e- > z h, z > mu+ mu-, h > b b~");
        let set = &chain[0];
        let reordered = set.with_final_order(&[3, 2, 1, 0], &model).unwrap();
        for d in &reordered.diagrams {
            assert_leg_labels(&drawer.svg(d).unwrap(), &names(&reordered));
        }
    }

    /// A diagram feyngraph would refuse, or whose drawing it would not survive,
    /// is an error, not a panic: a vertex that lost its propagators, and two
    /// outgoing legs whose particles are exchanged under their vertices.
    #[test]
    fn a_broken_diagram_is_refused() {
        let model = sm_model(SMRestrict::Default);
        let drawer = DiagramDrawer::new(&model);
        let original = sets("generate u u~ > e+ e- g")[0].diagrams[0].clone();

        let mut detached = original.clone();
        detached.vertices[0]
            .rays
            .retain(|r| !matches!(r, Ray::Prop { .. }));
        assert!(drawer.svg(&detached).is_err());

        let mut swapped = original.clone();
        let (a, b) = (swapped.legs[2].particle, swapped.legs[4].particle);
        swapped.legs[2].particle = b;
        swapped.legs[4].particle = a;
        assert!(
            matches!(drawer.svg(&swapped), Err(DrawError::VertexParticles { .. })),
            "{:?}",
            drawer.svg(&swapped)
        );
        drawer.svg(&original).unwrap();
    }
}
