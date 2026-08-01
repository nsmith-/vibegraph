//! The clustering-topology facts a process implies, read off its diagrams.
//!
//! [`scales`](super::scales) takes the four facts MadGraph's clustering consults
//! as a declaration ([`ClusterTopology`]) rather than inferring them from PDG
//! codes, so that a wrong declaration fails against MadGraph instead of landing
//! one branch out. This module produces that declaration from the enumerated
//! diagrams and the model, so an integrator carries no table keyed by process
//! name.
//!
//! Each fact is read off the one structure that defines it.
//!
//! * **Beam connections.** A spacelike line is one through which exactly one beam
//!   flows, which is a property of the diagram's momentum routing
//!   ([`Prop::is_spacelike`]) and not of any vertex: in `e⁺e⁻ → μ⁺μ⁻τ⁺τ⁻` no
//!   vertex joins an electron to a muon, yet the `ZZ` diagram hangs both bosons
//!   off the electron line and the electron propagator between them carries one
//!   beam. Reading the momentum therefore sees the t-channel that vertex
//!   adjacency alone would miss.
//! * **The two-body merge mask.** For a `2 → 2` the clustering's only possible
//!   move is to merge a pair that shares a vertex in some diagram, and with four
//!   externals a tree puts a pair at a common vertex exactly when a single
//!   propagator separates them. So the mask is beam↔leg vertex adjacency.
//! * **Coloured beams.** The beams' colour representation. The question the
//!   clustering asks is whether a colour line can run from beam to beam, so both
//!   beams must carry colour, not just one.
//! * **The coloured central line.** With no spacelike line the two beams must meet
//!   at a common vertex — any internal line on a path between them would separate
//!   them and so be spacelike — and the propagators at that vertex are what the
//!   final state clusters onto.
//! * **Jet legs.** `isjet`: a gluon or a quark at or below `maxjetflavor`.
//!
//! The derivation is a set of claims about MadGraph's clustering, so it is pinned
//! by tests that reproduce the topologies the banked runs are replayed under
//! rather than left to agree by inspection.

use crate::diagrams::diagram::{Diagram, LegIdx, Ray};
use crate::ufo::particles::ParticleId;
use crate::ufo::UFOModel;

use super::scales::{BeamConnections, ClusterTopology};

/// The gluon's PDG code, which `isjet` accepts at any `maxjetflavor`.
const GLUON_PDG: i64 = 21;

/// `isjet` (`SubProcesses/cuts.f`): a gluon, or a quark of a flavour light enough
/// to be counted as one.
fn is_jet(pdg: i64, maxjetflavor: i64) -> bool {
    pdg == GLUON_PDG || (pdg.abs() >= 1 && pdg.abs() <= maxjetflavor)
}

fn is_coloured(model: &UFOModel, id: ParticleId) -> bool {
    model.particle(id).color.unsigned_abs() != 1
}

/// The topology declaration a process's diagrams imply.
///
/// `externals` are the process's external particles with the `n_in` incoming legs
/// first — the ordering [`Diagram`]'s [`LegIdx`] indexes — and `maxjetflavor` is
/// the run card's.
pub fn cluster_topology(
    diagrams: &[Diagram],
    externals: &[ParticleId],
    n_in: usize,
    model: &UFOModel,
    maxjetflavor: i64,
) -> ClusterTopology {
    let n_out = externals.len() - n_in;

    let spacelike = diagrams
        .iter()
        .any(|d| d.props.iter().any(|p| p.is_spacelike(n_in)));
    let beam_connections = if spacelike {
        BeamConnections::TChannel {
            two_body_pairs: two_body_merges(diagrams, n_in, n_out),
        }
    } else {
        BeamConnections::SChannelOnly
    };

    ClusterTopology {
        beam_connections,
        coloured_beams: externals[..n_in].iter().all(|&id| is_coloured(model, id)),
        coloured_central_line: central_line_is_coloured(diagrams, model, n_in),
        jet_legs: externals[n_in..]
            .iter()
            .all(|&id| is_jet(model.particle(id).pdg_code, maxjetflavor)),
    }
}

/// Which `(beam, outgoing leg)` pairs some diagram lets the clustering merge.
fn two_body_merges(diagrams: &[Diagram], n_in: usize, n_out: usize) -> [[bool; 2]; 2] {
    // Only a 2 -> 2 has a mask of this shape, and a longer final state has no
    // closed-form cluster scale at all: `ScaleChoice::scales` refuses on the
    // multiplicity before reading the mask. The permissive value keeps the refusal
    // about the multiplicity rather than about an under-populated mask.
    if n_in != 2 || n_out != 2 {
        return [[true; 2]; 2];
    }
    let mut pairs = [[false; 2]; 2];
    let mut legs: Vec<usize> = Vec::new();
    for diagram in diagrams {
        for vertex in &diagram.vertices {
            legs.clear();
            legs.extend(vertex.rays.iter().filter_map(|ray| match ray {
                Ray::Leg(LegIdx(i)) => Some(*i),
                Ray::Prop { .. } => None,
            }));
            for &beam in legs.iter().filter(|&&i| i < n_in) {
                for &leg in legs.iter().filter(|&&i| i >= n_in) {
                    pairs[beam][leg - n_in] = true;
                }
            }
        }
    }
    pairs
}

/// Whether the propagator an s-channel-only tree collapses onto carries colour.
///
/// With no spacelike line both beams sit at one vertex, so the internal lines at
/// that vertex are the ones joining the beams to the final state.
fn central_line_is_coloured(diagrams: &[Diagram], model: &UFOModel, n_in: usize) -> bool {
    diagrams.iter().any(|diagram| {
        diagram.vertices.iter().any(|vertex| {
            let beams = vertex
                .rays
                .iter()
                .filter(|ray| matches!(ray, Ray::Leg(LegIdx(i)) if *i < n_in))
                .count();
            beams == n_in
                && vertex.rays.iter().any(|ray| match ray {
                    Ray::Prop { prop, .. } => is_coloured(model, diagram.props[prop.0].particle),
                    Ray::Leg(_) => false,
                })
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::ufo::sm::{sm_model, SMRestrict};

    /// MadGraph's default `maxjetflavor`, the value every banked run card carries.
    const MAXJETFLAVOR: i64 = 4;

    /// Derive the topology of a process the way an integrand does: every diagram of
    /// every non-empty subprocess, against the first subprocess's external state.
    fn topology(proc: &str) -> ClusterTopology {
        let model = sm_model(SMRestrict::Default);
        let card = parse_proc_card(&format!("generate {proc}"), &ParsingOptions::default())
            .expect("proc card parses");
        let sets = generate_from_proc_card(&card, &model).expect("enumerate");
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
        let diagrams: Vec<Diagram> = sets
            .iter()
            .flat_map(|s| s.diagrams.iter().cloned())
            .collect();
        cluster_topology(
            &diagrams,
            &externals,
            set.particles_in.len(),
            &model,
            MAXJETFLAVOR,
        )
    }

    /// The QCD `2 → 2` processes whose cluster scale the cross-section gate rests
    /// on. Each row is a hypothesis about MadGraph's clustering tree; a derivation
    /// that produced a different one would move the scale, so it fails here rather
    /// than shifting a cross section smoothly.
    #[test]
    fn coloured_two_to_two_topologies_are_derived() {
        // Every leg a gluon, so either outgoing leg may follow either beam.
        assert_eq!(
            topology("g g > g g"),
            ClusterTopology {
                beam_connections: BeamConnections::TChannel {
                    two_body_pairs: [[true, true], [true, true]],
                },
                coloured_beams: true,
                coloured_central_line: true,
                jet_legs: true,
            }
        );
        // Same free mask, but the tops are too heavy to count as jets.
        assert_eq!(
            topology("g g > t t~"),
            ClusterTopology {
                beam_connections: BeamConnections::TChannel {
                    two_body_pairs: [[true, true], [true, true]],
                },
                coloured_beams: true,
                coloured_central_line: true,
                jet_legs: false,
            }
        );
        // Flavour locks each outgoing leg to the beam of its own flavour: no vertex
        // joins the incoming u to the outgoing u~. This is the mask that lets both
        // allowed pairs be crossed at once, which is how the clustering's tie-break
        // reaches the scale at all.
        assert_eq!(
            topology("u u~ > u u~"),
            ClusterTopology {
                beam_connections: BeamConnections::TChannel {
                    two_body_pairs: [[true, false], [false, true]],
                },
                coloured_beams: true,
                coloured_central_line: true,
                jet_legs: true,
            }
        );
    }

    /// A colour-singlet final state off coloured beams, and off colourless ones.
    /// The distinction moves which vertex the scale is read off, and it is not the
    /// final state's colour that decides it: `e⁺e⁻ → t t̄` is coloured throughout
    /// its final state and still has a colourless central line.
    #[test]
    fn s_channel_annihilation_topologies_are_derived() {
        assert_eq!(
            topology("u u~ > mu+ mu-"),
            ClusterTopology {
                beam_connections: BeamConnections::SChannelOnly,
                coloured_beams: true,
                coloured_central_line: false,
                jet_legs: false,
            }
        );
        assert_eq!(
            topology("e+ e- > mu+ mu-"),
            ClusterTopology {
                beam_connections: BeamConnections::SChannelOnly,
                coloured_beams: false,
                coloured_central_line: false,
                jet_legs: false,
            }
        );
        assert_eq!(
            topology("e+ e- > t t~"),
            ClusterTopology {
                beam_connections: BeamConnections::SChannelOnly,
                coloured_beams: false,
                coloured_central_line: false,
                jet_legs: false,
            }
        );
    }

    /// Bhabha scattering exchanges a photon between the two electron lines, so each
    /// outgoing lepton is locked to the beam it shares a vertex with.
    #[test]
    fn lepton_t_channel_locks_each_leg_to_one_beam() {
        assert_eq!(
            topology("e+ e- > e+ e-"),
            ClusterTopology {
                beam_connections: BeamConnections::TChannel {
                    two_body_pairs: [[true, false], [false, true]],
                },
                coloured_beams: false,
                coloured_central_line: false,
                jet_legs: false,
            }
        );
        // `W` pair production exchanges a neutrino, and the charged current pairs
        // each beam with the `W` carrying its own charge: an incoming `e⁺` reaches
        // the `ν̄` by emitting the `W⁺`, an incoming `e⁻` reaches the `ν` by
        // emitting the `W⁻`. So the mask is Bhabha's diagonal and not its
        // transpose, in the beam and leg orders the process is written in.
        assert_eq!(
            topology("e+ e- > W+ W-"),
            ClusterTopology {
                beam_connections: BeamConnections::TChannel {
                    two_body_pairs: [[true, false], [false, true]],
                },
                coloured_beams: false,
                coloured_central_line: false,
                jet_legs: false,
            }
        );
    }

    /// A t-channel that no vertex adjacency reveals: no vertex joins an electron to
    /// a muon or a tau, yet the `ZZ` diagram hangs both bosons off the electron
    /// line, so an electron propagator carries exactly one beam. Reading the
    /// momentum sees it; reading the vertices would report `SChannelOnly` and hand
    /// the scale to a branch that does not apply.
    #[test]
    fn a_beam_line_inside_a_diagram_counts_as_a_t_channel() {
        assert!(matches!(
            topology("e+ e- > mu+ mu- ta+ ta-").beam_connections,
            BeamConnections::TChannel { .. }
        ));
        // A photon radiated off the electron line does the same at three legs.
        assert!(matches!(
            topology("e+ e- > mu+ mu- a").beam_connections,
            BeamConnections::TChannel { .. }
        ));
    }

    #[test]
    fn isjet_follows_maxjetflavor() {
        assert!(is_jet(GLUON_PDG, 4));
        assert!(is_jet(-4, 4));
        assert!(!is_jet(5, 4));
        assert!(is_jet(5, 5));
        assert!(!is_jet(11, 4));
    }
}
