//! The channel forests a process's own diagrams imply — `configs.inc`, derived
//! rather than read.
//!
//! [`graph::ChannelSet`] is the input the merge graph is built from, and
//! MadGraph fills it from a generated `configs.inc`. This module produces the
//! same structure from vibegraph's enumerated [`Diagram`]s, so nothing
//! downstream of it needs a MadGraph process directory.
//!
//! Four properties of `configs.inc` are reproduced here, and each of them is a
//! decision MadGraph's generator makes rather than a fact about the diagram.
//!
//! * **The tree is re-rooted toward the highest-numbered initial leg.** Every
//!   internal line is named by the set of external legs *below* it in that
//!   rooting, so a line whose subtree carries beam 1 is spacelike and one whose
//!   subtree carries neither beam is timelike.
//! * **The closing vertex is written only for a channel that reaches the beams
//!   through a spacelike line** (`export_v4.py:2229`, `if len(tchannels) > 1`).
//!   An s-channel-only channel therefore has one line fewer, and the vertex that
//!   joins its outermost propagator to the two beams is implicit.
//! * **Four-point vertices are absent.** The generator keeps only the diagrams
//!   whose largest vertex is no larger than the smallest such vertex over the
//!   whole set, which for a process with any three-point-only diagram drops
//!   every four-point one — `g g → g g` has three channels, not four.
//! * **The line order is s-channels first, then the spacelike chain from beam 1
//!   inward.** The clustering's Breit-Wigner scan walks the lines in that order
//!   and stops tagging at the first spacelike one, so the partition is
//!   load-bearing; the order *within* the s-channel block is not, since every
//!   line is scanned after its own daughters either way.
//!
//! The per-line codes follow the same file: a timelike line carries the signed
//! PDG of the particle that decays into its subtree, a spacelike one the
//! magnitude of its propagator's code, and the closing line the magnitude of
//! beam 2's own. Only the non-closing lines carry a mass and width
//! (`export_v4.py:2107`, `configs[0] + configs[1][:-1]`), which is what keeps
//! the closing line out of the resonance map.

use thiserror::Error;

use crate::diagrams::diagram::{Diagram, LegIdx, PropIdx, Ray, VtxIdx};
use crate::ufo::particles::ParticleId;
use crate::ufo::{EvaluatedModel, UFOModel};

use super::graph::{ChannelSet, ConfigForest, ForestLine};

/// Why a process's diagrams do not yield channel forests.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum ConfigError {
    #[error("channel forests are defined for a 2 -> n process, got {n_in} incoming legs")]
    NotTwoBeams { n_in: usize },
    #[error(
        "every diagram of this process carries a vertex of {arity} lines, so MadGraph's \
         generator would split them with fake propagators rather than drop the diagrams"
    )]
    IrreducibleHigherVertex { arity: usize },
    #[error("diagram {index} is not a tree rooted on its external legs")]
    NotATree { index: usize },
    #[error("the external state has {got} legs and the diagrams have {want}")]
    ExternalMismatch { got: usize, want: usize },
}

/// The channel forests of one subprocess, with the diagram each came from.
#[derive(Clone, Debug)]
pub struct DerivedChannels {
    pub set: ChannelSet,
    /// `diagram_of[c - 1]` indexes the diagram slice this channel was derived
    /// from. The two numberings differ: a diagram the vertex filter drops has no
    /// channel at all.
    pub diagram_of: Vec<usize>,
    /// `config_of_diagram[d]` is the channel (from `1`) diagram `d` yielded, or
    /// `None` where the vertex filter dropped it — the inverse of `diagram_of`,
    /// over the whole diagram slice rather than over the surviving channels.
    ///
    /// This is the map a sampler needs. Its channels are one per *diagram*, so
    /// the channel it drew a point in names an integration channel only through
    /// here, and the two numberings coincide exactly when nothing was dropped.
    pub config_of_diagram: Vec<Option<usize>>,
}

/// One vertex of the re-rooted tree, in the shape `configs.inc` writes.
struct Line {
    /// The external legs below this line, as a bit per leg.
    mask: u32,
    /// Each daughter as either an external leg number (positive) or the mask of
    /// the line below it.
    daughters: [Daughter; 2],
    spacelike: bool,
    pdg: i64,
    mass: f64,
    width: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Daughter {
    Leg(usize),
    Line(u32),
}

impl Daughter {
    /// The leg number MadGraph sorts a vertex's daughters by: an external leg is
    /// its own number, an internal line the smallest leg below it.
    fn number(self) -> usize {
        match self {
            Daughter::Leg(leg) => leg,
            Daughter::Line(mask) => mask.trailing_zeros() as usize + 1,
        }
    }
}

/// The channel forests of one subprocess, derived from its diagrams.
///
/// `externals` are the subprocess's external particles with the `n_in` incoming
/// ones first, in the physical convention `leshouche.inc` uses — an outgoing leg
/// is the particle it is, not the crossed antiparticle the diagram carries.
pub fn derive_channels(
    diagrams: &[Diagram],
    externals: &[ParticleId],
    n_in: usize,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
) -> Result<DerivedChannels, ConfigError> {
    let identity: Vec<usize> = (0..externals.len()).collect();
    derive_channels_permuted(diagrams, externals, n_in, &identity, model, evaluated)
}

/// The channel forests of a *relabelling* of the process the diagrams were
/// enumerated for: `positions[l]` is the external position diagram leg `l`
/// occupies, and `externals` is in position order.
///
/// The one relabelling this is for is the beam crossing. MadGraph generates
/// `ū u → b b̄` as its own subprocess directory, with its own `configs.inc`;
/// vibegraph's enumeration produces the initial state once, so the crossed
/// ordering is the same diagrams read with legs 1 and 2 exchanged. The
/// distinction is not cosmetic — the forests are rooted on beam 2 and the
/// clustering measures each beam separately — so the crossed subprocess is
/// derived rather than served by swapping an event's beams.
pub fn derive_channels_permuted(
    diagrams: &[Diagram],
    externals: &[ParticleId],
    n_in: usize,
    positions: &[usize],
    model: &UFOModel,
    evaluated: &EvaluatedModel,
) -> Result<DerivedChannels, ConfigError> {
    if n_in != 2 {
        return Err(ConfigError::NotTwoBeams { n_in });
    }
    let n_external = externals.len();

    // MadGraph keeps the diagrams whose largest vertex is no larger than the
    // smallest such vertex over the set, and splits the survivors' higher
    // vertices with fake propagators. Nothing here does the splitting, so a
    // process whose every diagram needs it is refused rather than approximated.
    let arity = |d: &Diagram| d.vertices.iter().map(|v| v.rays.len()).max().unwrap_or(0);
    let minvert = diagrams
        .iter()
        .map(arity)
        .filter(|&a| a > 0)
        .min()
        .unwrap_or(0);
    if minvert > 3 {
        return Err(ConfigError::IrreducibleHigherVertex { arity: minvert });
    }

    let mut configs: Vec<ConfigForest> = Vec::new();
    let mut diagram_of: Vec<usize> = Vec::new();
    for (index, diagram) in diagrams.iter().enumerate() {
        if diagram.legs.len() != n_external {
            return Err(ConfigError::ExternalMismatch {
                got: diagram.legs.len(),
                want: n_external,
            });
        }
        if arity(diagram) > minvert {
            continue;
        }
        configs.push(forest(
            diagram, index, n_external, positions, model, evaluated, externals,
        )?);
        diagram_of.push(index);
    }

    let external_pdg: Vec<i64> = externals
        .iter()
        .map(|&id| model.particle(id).pdg_code)
        .collect();
    let n_configs = configs.len();
    let mut config_of_diagram = vec![None; diagrams.len()];
    for (config, &diagram) in diagram_of.iter().enumerate() {
        config_of_diagram[diagram] = Some(config + 1);
    }
    Ok(DerivedChannels {
        set: ChannelSet {
            n_external,
            n_incoming: n_in,
            configs,
            external_pdg: vec![external_pdg],
            contributes: vec![vec![true; n_configs]],
        },
        diagram_of,
        config_of_diagram,
    })
}

#[allow(clippy::too_many_arguments)]
fn forest(
    diagram: &Diagram,
    index: usize,
    n_external: usize,
    positions: &[usize],
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    externals: &[ParticleId],
) -> Result<ConfigForest, ConfigError> {
    // Beam 2 is the root: every line's leg set is what sits below it once the
    // tree hangs from that leg.
    let root_leg = LegIdx(
        positions
            .iter()
            .position(|&p| p == 1)
            .ok_or(ConfigError::NotATree { index })?,
    );
    let root_vertex = diagram
        .vertices
        .iter()
        .position(|v| {
            v.rays
                .iter()
                .any(|r| matches!(r, Ray::Leg(leg) if *leg == root_leg))
        })
        .ok_or(ConfigError::NotATree { index })?;

    let mut walk = Walk {
        diagram,
        index,
        model,
        evaluated,
        positions,
        root_leg,
        seen: vec![false; diagram.vertices.len()],
        timelike: Vec::new(),
        spacelike: Vec::new(),
        root: None,
    };
    let closing = walk.descend(VtxIdx(root_vertex), None)?;
    if walk.seen.iter().any(|&v| !v) {
        return Err(ConfigError::NotATree { index });
    }

    let mut lines = walk.timelike;
    let has_spacelike = !walk.spacelike.is_empty();
    lines.append(&mut walk.spacelike);
    if has_spacelike {
        // The vertex that closes the tree on beam 2 is written only for a
        // channel that reaches the beams through a spacelike line. Its code is
        // beam 2's own, and it carries no mass or width.
        lines.push(Line {
            mask: closing,
            daughters: walk.root.ok_or(ConfigError::NotATree { index })?,
            spacelike: true,
            pdg: model.particle(externals[1]).pdg_code.abs(),
            mass: 0.0,
            width: 0.0,
        });
    }

    let full = (1u32 << n_external) - 1;
    debug_assert_eq!(closing, full & !(1 << 1));
    let _ = full;

    // The negative index of a line, assigned in the order the file writes them.
    let position = |mask: u32| -> Option<i32> {
        lines
            .iter()
            .position(|l| l.mask == mask)
            .map(|k| -(k as i32 + 1))
    };
    let mut written = Vec::with_capacity(lines.len());
    for (k, line) in lines.iter().enumerate() {
        let mut daughters = line.daughters;
        // A vertex's daughters are sorted by their own leg number, ascending on
        // a spacelike line and descending on a timelike one, which is what puts
        // beam 1 first on the outermost spacelike vertex.
        if (daughters[0].number() > daughters[1].number()) == line.spacelike {
            daughters.swap(0, 1);
        }
        let resolve = |d: Daughter| -> Option<i32> {
            match d {
                Daughter::Leg(leg) => Some(leg as i32),
                Daughter::Line(mask) => position(mask),
            }
        };
        written.push(ForestLine {
            index: -(k as i32 + 1),
            daughters: [
                resolve(daughters[0]).ok_or(ConfigError::NotATree { index })?,
                resolve(daughters[1]).ok_or(ConfigError::NotATree { index })?,
            ],
            tprid: if line.spacelike { line.pdg } else { 0 },
            sprop: vec![if line.spacelike { 0 } else { line.pdg }],
            mass: line.mass,
            width: line.width,
        });
    }

    Ok(ConfigForest {
        nqcd: qcd_order(diagram, model),
        lines: written,
    })
}

struct Walk<'a> {
    diagram: &'a Diagram,
    index: usize,
    positions: &'a [usize],
    root_leg: LegIdx,
    model: &'a UFOModel,
    evaluated: &'a EvaluatedModel,
    seen: Vec<bool>,
    timelike: Vec<Line>,
    spacelike: Vec<Line>,
    /// The root vertex's two rays other than beam 2, which the closing line
    /// takes as its daughters.
    root: Option<[Daughter; 2]>,
}

impl Walk<'_> {
    /// Visit `vertex`, having arrived through `from` — the propagator toward
    /// beam 2, with the endpoint index this vertex sits on — and return the set
    /// of external legs below it.
    ///
    /// Each visited vertex other than the root contributes one line, appended
    /// after its own daughters so that the timelike block is bottom-up and the
    /// spacelike block runs from beam 1 inward.
    fn descend(
        &mut self,
        vertex: VtxIdx,
        from: Option<(PropIdx, usize)>,
    ) -> Result<u32, ConfigError> {
        if std::mem::replace(&mut self.seen[vertex.0], true) {
            return Err(ConfigError::NotATree { index: self.index });
        }
        let mut mask = 0u32;
        let mut daughters: Vec<Daughter> = Vec::new();
        for ray in &self.diagram.vertices[vertex.0].rays {
            match *ray {
                Ray::Leg(leg) => {
                    // Beam 2 sits at the root and is the one leg that is not
                    // below any line.
                    if from.is_none() && leg == self.root_leg {
                        continue;
                    }
                    let position = self.positions[leg.0];
                    mask |= 1 << position;
                    daughters.push(Daughter::Leg(position + 1));
                }
                Ray::Prop { prop, end } => {
                    if from.map(|(p, _)| p) == Some(prop) {
                        continue;
                    }
                    let (child, _) = self.diagram.props[prop.0].endpoints[1 - end];
                    let below = self.descend(child, Some((prop, 1 - end)))?;
                    mask |= below;
                    daughters.push(Daughter::Line(below));
                }
            }
        }
        let Some((prop, child_end)) = from else {
            if daughters.len() == 2 {
                self.root = Some([daughters[0], daughters[1]]);
            }
            return Ok(mask);
        };
        if daughters.len() != 2 {
            return Err(ConfigError::IrreducibleHigherVertex {
                arity: daughters.len() + 1,
            });
        }
        let particle = self.diagram.props[prop.0].particle;
        let pdg = self.model.particle(particle).pdg_code;
        // Beam 1 below the line makes it spacelike; every other line is a
        // propagator the final state hangs from.
        let spacelike = mask & 1 != 0;
        let line = Line {
            mask,
            daughters: [daughters[0], daughters[1]],
            spacelike,
            pdg: if spacelike {
                pdg.abs()
            } else {
                // The code MadGraph writes is the particle that decays into the
                // subtree: the propagator as it flows away from the beam-2 side
                // of the line.
                signed_toward_subtree(self.model, particle, pdg, child_end)
            },
            mass: self.evaluated.mass(particle),
            width: self.evaluated.width(particle),
        };
        if spacelike {
            self.spacelike.push(line);
        } else {
            self.timelike.push(line);
        }
        Ok(mask)
    }
}

/// The propagator's code as seen flowing into its own subtree.
///
/// A [`Ray::Prop`] records which endpoint a vertex is, and momentum flows from
/// endpoint `0` to endpoint `1`, so a subtree whose vertex is endpoint `1`
/// receives the stored species and one whose vertex is endpoint `0` receives its
/// antiparticle. Which one `configs.inc` writes is fixed by the file itself: the
/// timelike line above `e⁺ e⁻ μ⁺` is a `μ⁺`, not a `μ⁻`.
fn signed_toward_subtree(
    model: &UFOModel,
    particle: ParticleId,
    pdg: i64,
    child_end: usize,
) -> i64 {
    let p = model.particle(particle);
    if child_end == 1 || p.name == p.antiname {
        pdg
    } else {
        -pdg
    }
}

/// The diagram's QCD coupling order, summed over its vertices the way
/// `calculate_orders` sums it.
fn qcd_order(diagram: &Diagram, model: &UFOModel) -> i64 {
    diagram
        .vertices
        .iter()
        .map(|vertex| {
            model
                .vertex_def(vertex.interaction)
                .couplings
                .values()
                .filter_map(|&id| model.coupling_def(id).orders.get("QCD").copied())
                .max()
                .unwrap_or(0) as i64
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::ufo::sm::{sm_model, SMRestrict};

    /// Derive the channels of a fully concrete process the way an integrand
    /// would: every diagram of the one subprocess, against its own externals.
    fn channels(process: &str) -> (usize, DerivedChannels) {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let card = parse_proc_card(&format!("generate {process}"), &ParsingOptions::default())
            .expect("proc card parses");
        let sets = generate_from_proc_card(&card, model.as_ref()).expect("enumerate");
        let set = sets
            .iter()
            .find(|s| !s.diagrams.is_empty())
            .expect("a non-empty subprocess");
        let externals: Vec<ParticleId> = set
            .particles_in
            .iter()
            .chain(set.particles_out.iter())
            .map(|name| model.particle_id(name).expect("external in model"))
            .collect();
        let derived = derive_channels(
            &set.diagrams,
            &externals,
            set.particles_in.len(),
            model.as_ref(),
            &evaluated,
        )
        .expect("channel forests");
        (set.diagrams.len(), derived)
    }

    /// `g g → g g` has four diagrams and three channels: the four-gluon vertex
    /// exceeds the smallest largest-vertex over the set, so its diagram is
    /// dropped. Nothing in the banked clustering dumps exercises this, and a
    /// channel too many would change the merge graph of every `g g` event.
    #[test]
    fn a_four_point_vertex_leaves_its_diagram_without_a_channel() {
        let (diagrams, derived) = channels("g g > g g");
        assert_eq!(diagrams, 4);
        assert_eq!(derived.set.configs.len(), 3);
        assert_eq!(derived.diagram_of.len(), 3);
        for config in &derived.set.configs {
            assert_eq!(config.nqcd, 2);
        }
        // What survives is the s-channel gluon, whose closing vertex stays
        // implicit and which therefore carries one line, and the two spacelike
        // channels, which write theirs.
        let mut masks: Vec<Vec<u32>> = derived
            .set
            .configs
            .iter()
            .map(|c| c.lines.iter().filter_map(|l| c.mask(l.index)).collect())
            .collect();
        masks.sort();
        assert_eq!(
            masks,
            vec![vec![0b0101, 0b1101], vec![0b1001, 0b1101], vec![0b1100]]
        );
    }

    /// The map from a sampler's channels to integration channels, on the one
    /// process where the two numberings provably differ.
    ///
    /// A per-diagram sampler has one channel per *diagram*; `configs.inc` has one
    /// per surviving diagram. `g g → g g` is where that gap is visible without a
    /// MadGraph run: the four-gluon diagram has no channel, so the sampler's
    /// fourth channel maps to nothing and the other three do not map to
    /// themselves. Anything that reorders either side — the diagram enumeration
    /// or the forest derivation — moves this table.
    #[test]
    fn the_channel_to_config_map_is_not_the_identity() {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let card = parse_proc_card("generate g g > g g", &ParsingOptions::default())
            .expect("proc card parses");
        let sets = generate_from_proc_card(&card, model.as_ref()).expect("enumerate");
        let set = sets
            .iter()
            .find(|s| !s.diagrams.is_empty())
            .expect("a non-empty subprocess");
        let externals: Vec<ParticleId> = set
            .particles_in
            .iter()
            .chain(set.particles_out.iter())
            .map(|name| model.particle_id(name).expect("external in model"))
            .collect();
        let derived = derive_channels(
            &set.diagrams,
            &externals,
            set.particles_in.len(),
            model.as_ref(),
            &evaluated,
        )
        .expect("channel forests");

        // The unmapped channel is the four-gluon one, identified by the vertex
        // that gets it dropped rather than by its position.
        let four_point: Vec<usize> = set
            .diagrams
            .iter()
            .enumerate()
            .filter(|(_, d)| d.vertices.iter().any(|v| v.rays.len() == 4))
            .map(|(i, _)| i)
            .collect();
        assert_eq!(four_point.len(), 1);
        assert_eq!(derived.config_of_diagram[four_point[0]], None);
        assert_eq!(
            derived
                .config_of_diagram
                .iter()
                .filter(|c| c.is_none())
                .count(),
            1
        );

        // Both directions of the same map.
        for (diagram, config) in derived.config_of_diagram.iter().enumerate() {
            if let Some(config) = config {
                assert_eq!(derived.diagram_of[config - 1], diagram);
            }
        }

        // The table itself, and the topology each entry lands on: a reorder that
        // carried both sides along together would leave the indices alone and
        // move the masks.
        let masks: Vec<Option<Vec<u32>>> = derived
            .config_of_diagram
            .iter()
            .map(|c| {
                c.map(|c| {
                    let forest = &derived.set.configs[c - 1];
                    forest
                        .lines
                        .iter()
                        .filter_map(|l| forest.mask(l.index))
                        .collect()
                })
            })
            .collect();
        assert_eq!(
            derived.config_of_diagram,
            vec![None, Some(1), Some(2), Some(3)]
        );
        // The s-channel gluon writes one line and the two spacelike channels
        // write two each, as `a_four_point_vertex_leaves_its_diagram_without_a_channel`
        // reads them off the channel side.
        assert_eq!(
            masks,
            vec![
                None,
                Some(vec![0b1100]),
                Some(vec![0b0101, 0b1101]),
                Some(vec![0b1001, 0b1101]),
            ]
        );
    }

    /// An s-channel-only tree stops one vertex short: the propagator joining the
    /// two beams is implicit, so a `2 → 2` has one line and not two.
    #[test]
    fn an_s_channel_tree_leaves_its_closing_vertex_implicit() {
        let (_, derived) = channels("e+ e- > mu+ mu-");
        for config in &derived.set.configs {
            assert_eq!(config.lines.len(), 1);
            let line = &config.lines[0];
            assert_eq!(config.mask(line.index), Some(0b1100));
            assert_eq!(line.tprid, 0);
            assert!(line.sprop[0] == 22 || line.sprop[0] == 23);
        }
    }

    /// A channel that reaches the beams through a spacelike line writes the
    /// vertex that closes it, and that line carries beam 2's own code with no
    /// mass and no width — which is what keeps it out of the resonance map.
    #[test]
    fn a_spacelike_channel_closes_on_beam_two() {
        let (_, derived) = channels("u u~ > u u~");
        let spacelike = derived
            .set
            .configs
            .iter()
            .find(|c| c.lines.len() == 2)
            .expect("the t-channel gluon config");
        let closing = spacelike.lines.last().expect("a closing line");
        // Every leg but beam 2.
        assert_eq!(spacelike.mask(closing.index), Some(0b1101));
        assert_eq!(closing.tprid, 2);
        assert_eq!(closing.sprop[0], 0);
        assert_eq!(closing.mass, 0.0);
        assert_eq!(closing.width, 0.0);
        // Its daughters are the spacelike gluon and the leg the beam-2 vertex
        // emits, ascending as a spacelike vertex sorts them.
        assert_eq!(closing.daughters, [-1, 4]);
        let gluon = &spacelike.lines[0];
        assert_eq!(spacelike.mask(gluon.index), Some(0b0101));
        assert_eq!(gluon.tprid, 21);
    }

    /// The timelike code is the particle that decays into the line's own
    /// subtree, not the one leaving it. The two differ by a sign for anything
    /// that is not its own antiparticle, and MadGraph's file settles which:
    /// above `e⁺ e⁻ μ⁺` sits a `μ⁺`.
    #[test]
    fn a_timelike_line_carries_the_particle_that_decays_into_it() {
        let (_, derived) = channels("e+ e- > mu+ mu- a");
        let mut seen = 0usize;
        for config in &derived.set.configs {
            for line in &config.lines {
                let Some(mask) = config.mask(line.index) else {
                    continue;
                };
                // The muon line above {mu+, a}: legs 3 and 5 of `mu+ mu- a`.
                if mask == 0b10100 && line.tprid == 0 {
                    assert_eq!(line.sprop[0], -13);
                    seen += 1;
                }
            }
        }
        assert!(seen > 0, "no muon line above the radiating mu+");
    }

    /// A process with no spacelike line at all still refuses nothing: the
    /// derivation is a statement about the diagrams, and a decay-like beam
    /// configuration is simply not one it is defined for.
    #[test]
    fn a_one_beam_process_is_refused() {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        assert_eq!(
            derive_channels(&[], &[], 1, model.as_ref(), &evaluated).unwrap_err(),
            ConfigError::NotTwoBeams { n_in: 1 }
        );
    }
}
