//! Decay-chain enumeration by stitching: `p p > t t~, (t > w+ b, w+ > e+ ve), t~ > w- b~`.
//!
//! MadGraph generates the core and each decay as separate amplitudes
//! (`DecayChainAmplitude`, `diagram_generation.py:1337`) and joins them into one matrix
//! element per combination of decays (`HelasDecayChainProcess.combine_decay_chain_processes`,
//! `helas_objects.py:5427`). Here the core and each decay are enumerated on their own —
//! every decay a `1 → n` process with its own lowest-order search, recursively for a
//! decay with decays of its own — and each decay's diagrams are glued onto the matching
//! final-state leg of each core diagram. The leg becomes a propagator, flagged
//! [`OnShell::Forced`], from the core vertex it attached to into the decay vertex the
//! decaying particle entered, and the decay's products take the leg's place in the final
//! state, in the decay's order (`Process.get_legs_with_decays`, `base_objects.py:3667`).
//!
//! **Which decay goes to which leg** follows `combine_decay_chain_processes`, per core
//! subprocess, over its final-state legs that some decay applies to:
//!
//! - as many such legs as decays, each decay applying to the leg at its position: decay
//!   `i` to leg `i`;
//! - otherwise, as many legs as decays, each decay one concrete process, and the same
//!   particles on both sides: the decays of each particle to its legs, in order;
//! - otherwise every combination, with repetition, of all the decays of each particle
//!   over its legs, an unordered combination once (`z z, z > l+ l-` gives `ee ee`,
//!   `ee μμ`, `μμ μμ`).
//!
//! **Identical particles from different decays** are where this differs from MadGraph by
//! design. MadGraph keeps each decay's products on the legs it put them on and divides by
//! a factor for identical decay chains (`identical_decay_chain_factor`,
//! `helas_objects.py:4581`), dropping the interference between assignments
//! (`e+ e- > z z, z > e+ e-`: two diagrams, factor 2). Here the stitched diagrams are
//! closed under every permutation of identical final-state particles, as the diagrams of
//! the undecayed final state are (four diagrams, and the usual 2!·2! for the identical
//! final state), so a stitched set is exactly the diagrams of the full final state in
//! which every chain resonance is an s-channel line with its stated products.
//!
//! **What is rebuilt from the graph.** A stitched diagram's propagator momenta
//! ([`Diagram::tree_momentum`]) and its sign (the fermion-pairing parity times the line
//! sign) are recomputed from the stitched graph, not combined from the parts: both are
//! functions of the whole graph (the momentum representative drops the last external leg,
//! the pairing parity reads the global leg order), and both reproduce the enumeration on
//! every diagram it produces.

use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use tracing::{debug, info};

use super::check::{AmplitudeOrder, SupportedProcess};
use super::diagram::{
    CanonicalDiagram, ChainNode, DecayOrigin, Diagram, Leg, LegIdx, OnShell, Prop, PropIdx, Ray,
    VtxIdx,
};
use super::{generate_from_process, DiagramError, DiagramSet};
use crate::ufo::UFOModel;

/// Enumerate a decay-chain process by stitching: one [`DiagramSet`] per core subprocess
/// and combination of decays, its final state with each decayed particle replaced by its
/// products.
pub(super) fn generate_chain(
    process: &SupportedProcess,
    model: &UFOModel,
) -> Result<Vec<DiagramSet>, DiagramError> {
    let sets = chain_sets(process, ChainNode(0), model)?;
    Ok(sets.into_iter().map(|s| s.set).collect())
}

/// One stitched subprocess, with what identifies its decays among the others.
struct ChainSet {
    set: DiagramSet,
    /// The concrete process with its decays, as MadGraph's `Process` equality tells two
    /// decay processes apart: the particles and the definition they came from.
    key: String,
}

/// The number of chain nodes in `process`'s tree, itself included.
fn node_count(process: &SupportedProcess) -> usize {
    1 + process.decays.iter().map(node_count).sum::<usize>()
}

/// The stitched subprocesses of `process`, whose chain node is `node`.
fn chain_sets(
    process: &SupportedProcess,
    node: ChainNode,
    model: &UFOModel,
) -> Result<Vec<ChainSet>, DiagramError> {
    let refuse = |reason: String| DiagramError::DecayChain {
        process: process.to_string(),
        reason,
    };
    let core_process = SupportedProcess {
        decays: Vec::new(),
        ..process.clone()
    };
    let core_sets: Vec<DiagramSet> = generate_from_process(&core_process, model)?
        .into_iter()
        .filter(|s| !s.diagrams.is_empty())
        .collect();
    // A core without diagrams has nothing to decay: the caller reports the empty line.
    if core_sets.is_empty() {
        return Ok(Vec::new());
    }
    if process.decays.is_empty() {
        return Ok(core_sets
            .into_iter()
            .map(|set| ChainSet {
                key: format!(
                    "{} > {} [{core_process}]",
                    set.particles_in.join(" "),
                    set.particles_out.join(" ")
                ),
                set,
            })
            .collect());
    }

    // Each decay, with the chain node it is: numbered in the order written, a decay's own
    // decays straight after it.
    let mut elements: Vec<(ChainNode, Vec<ChainSet>)> = Vec::new();
    let mut next = node.0 + 1;
    for decay in &process.decays {
        if decay.initial.len() != 1 {
            return Err(DiagramError::NotADecay {
                process: decay.to_string(),
                n_in: decay.initial.len(),
            });
        }
        let sets = chain_sets(decay, ChainNode(next), model)?;
        if sets.is_empty() {
            return Err(DiagramError::NoDiagrams {
                process: decay.to_string(),
            });
        }
        elements.push((ChainNode(next), sets));
        next += node_count(decay);
    }
    let initial_of = |c: &ChainSet| c.set.particles_in[0].clone();
    let is_ids: Vec<Vec<String>> = elements
        .iter()
        .map(|(_, sets)| sets.iter().map(initial_of).collect())
        .collect();
    let decaying: HashSet<&String> = is_ids.iter().flatten().collect();

    // MadGraph discards a decay whose particle is in no core subprocess, with a warning
    // (`diagram_generation.py:1405`): that is almost always a missing parenthesis, so here
    // it is an error.
    let core_final: HashSet<&String> = core_sets.iter().flat_map(|s| &s.particles_out).collect();
    let unused: Vec<&String> = decaying
        .iter()
        .copied()
        .filter(|p| !core_final.contains(p))
        .sorted()
        .collect();
    if !unused.is_empty() {
        return Err(refuse(format!(
            "no final state of the core has {} to decay; MadGraph would drop the decay (did a \
             nested decay lose its parentheses?)",
            unused.iter().join(", ")
        )));
    }

    let mut out = Vec::new();
    for core in &core_sets {
        let fs: Vec<(usize, &String)> = core
            .particles_out
            .iter()
            .enumerate()
            .filter(|(_, p)| decaying.contains(p))
            .collect();
        if fs.is_empty() {
            return Err(refuse(format!(
                "no decay applies to the subprocess {} > {}; MadGraph would keep it undecayed, \
                 beside decayed subprocesses of another multiplicity",
                core.particles_in.join(" "),
                core.particles_out.join(" ")
            )));
        }
        for assignment in assignments(&fs, &elements, &is_ids) {
            out.push(stitch_set(core, &assignment, node, model).map_err(refuse)?);
        }
    }

    if !process.chain_orders.is_empty() {
        for s in &mut out {
            s.set
                .diagrams
                .retain(|d| within_orders(d, &process.chain_orders, model));
        }
    }

    let n_out = out[0].set.particles_out.len();
    if let Some(other) = out.iter().find(|s| s.set.particles_out.len() != n_out) {
        return Err(refuse(format!(
            "its decays give final states of {n_out} and of {} particles, which only make \
             sense merged",
            other.set.particles_out.len()
        )));
    }
    let mut seen = HashSet::new();
    for s in &out {
        let mut key = s.set.particles_out.clone();
        key.sort();
        let mut initial = s.set.particles_in.clone();
        initial.sort();
        if !seen.insert((initial, key)) {
            return Err(DiagramError::DuplicateSubprocess {
                subprocess: format!(
                    "{} > {}",
                    s.set.particles_in.join(" "),
                    s.set.particles_out.join(" ")
                ),
                first: process.to_string(),
                second: process.to_string(),
            });
        }
    }
    Ok(out)
}

/// Whether a stitched diagram's coupling orders — the sum over its vertices, and
/// `WEIGHTED` as the model's hierarchy weighs them — stay within a decay chain's
/// overall orders. MadGraph removes a combined diagram exceeding any of them
/// (`HelasMatrixElement.insert_decay_chains`, `helas_objects.py:3986`).
fn within_orders(diagram: &Diagram, bounds: &[AmplitudeOrder], model: &UFOModel) -> bool {
    let mut orders: HashMap<&str, i64> = HashMap::new();
    for vertex in &diagram.vertices {
        let def = model.vertex_def(vertex.interaction);
        // A vertex carries one coupling-order tuple: the model splits a vertex
        // whose couplings differ in their orders into one interaction per tuple.
        if let Some(&coupling) = def.couplings.values().next() {
            for (name, n) in &model.coupling_def(coupling).orders {
                *orders.entry(name.as_str()).or_default() += *n as i64;
            }
        }
    }
    let weighted: i64 = orders
        .iter()
        .map(|(name, n)| n * model.order_hierarchy.get(*name).copied().unwrap_or(0) as i64)
        .sum();
    bounds.iter().all(|b| {
        let have = if b.name == "WEIGHTED" {
            weighted
        } else {
            orders.get(b.name.as_str()).copied().unwrap_or(0)
        };
        have <= b.value
    })
}

/// One assignment of decays to decaying legs: `(final-state position, node, decay)` per leg.
type Assignment<'a> = Vec<(usize, ChainNode, &'a ChainSet)>;

/// Every assignment of decays to the decaying final-state legs `fs` (position in the
/// final state, particle) of one core subprocess, per `combine_decay_chain_processes`
/// (`helas_objects.py:5510`–`5566`): a list of `(position, node, decay)`.
fn assignments<'a>(
    fs: &[(usize, &String)],
    elements: &'a [(ChainNode, Vec<ChainSet>)],
    is_ids: &[Vec<String>],
) -> Vec<Assignment<'a>> {
    let fs_ids: Vec<&String> = fs.iter().map(|(_, p)| *p).collect();
    let in_order =
        fs.len() == elements.len() && fs_ids.iter().zip(is_ids).all(|(id, ids)| ids.contains(*id));
    let reordered = fs.len() == elements.len()
        && is_ids.iter().all(|ids| ids.len() == 1)
        && fs_ids.iter().copied().sorted().collect::<Vec<_>>()
            == is_ids
                .iter()
                .map(|ids| &ids[0])
                .sorted()
                .collect::<Vec<_>>();

    let of = |index: usize, id: &String| -> Vec<(ChainNode, &'a ChainSet)> {
        let (node, sets) = &elements[index];
        sets.iter()
            .filter(|s| &s.set.particles_in[0] == id)
            .map(|s| (*node, s))
            .collect()
    };

    // Per particle: every combination of decays over its legs, as (position, node, decay).
    let mut decay_lists: Vec<Vec<Assignment<'a>>> = Vec::new();
    for fs_id in fs_ids.iter().copied().unique() {
        let positions: Vec<usize> = fs
            .iter()
            .filter(|(_, p)| *p == fs_id)
            .map(|(i, _)| *i)
            .collect();
        let mut chains: Vec<Vec<(ChainNode, &'a ChainSet)>> = Vec::new();
        if in_order {
            for (index, (_, p)) in fs.iter().enumerate() {
                if *p == fs_id {
                    chains.push(of(index, fs_id));
                }
            }
        } else if reordered {
            for index in 0..elements.len() {
                let out = of(index, fs_id);
                if !out.is_empty() {
                    chains.push(out);
                }
            }
        }
        if fs.len() != elements.len() || chains.is_empty() || chains[0].is_empty() {
            let chain: Vec<(ChainNode, &'a ChainSet)> =
                (0..elements.len()).flat_map(|i| of(i, fs_id)).collect();
            chains = vec![chain; positions.len()];
        }
        let mut used: HashSet<Vec<&str>> = HashSet::new();
        let mut list = Vec::new();
        for product in chains.into_iter().multi_cartesian_product() {
            let unordered: Vec<&str> = product
                .iter()
                .map(|(_, s)| s.key.as_str())
                .sorted()
                .collect();
            if !used.insert(unordered) {
                continue;
            }
            list.push(
                positions
                    .iter()
                    .zip(product)
                    .map(|(&pos, (node, set))| (pos, node, set))
                    .collect(),
            );
        }
        decay_lists.push(list);
    }
    decay_lists
        .into_iter()
        .multi_cartesian_product()
        .map(|parts| {
            let mut all: Vec<(usize, ChainNode, &ChainSet)> = parts.into_iter().flatten().collect();
            all.sort_by_key(|(pos, _, _)| *pos);
            all
        })
        .collect()
}

/// One core subprocess with one decay per decaying leg: every core diagram glued to every
/// combination of the decays' diagrams, closed under permutations of identical final-state
/// particles, each diagram once.
fn stitch_set(
    core: &DiagramSet,
    assignment: &[(usize, ChainNode, &ChainSet)],
    node: ChainNode,
    model: &UFOModel,
) -> Result<ChainSet, String> {
    let n_in = core.particles_in.len();

    // The final state with every decayed particle replaced by its products, each with its
    // polarization, and which block each final-state position belongs to: 0 for a core
    // leg, `i + 1` for the products of the `i`-th decay.
    let mut particles_out = Vec::new();
    let mut polarizations = core.polarizations[..n_in].to_vec();
    let mut block = Vec::new();
    for (pos, name) in core.particles_out.iter().enumerate() {
        match assignment.iter().position(|a| a.0 == pos) {
            Some(index) => {
                let decay = &assignment[index].2.set;
                for (product, pol) in decay.particles_out.iter().zip(&decay.polarizations[1..]) {
                    particles_out.push(product.clone());
                    polarizations.push(pol.clone());
                    block.push(index + 1);
                }
            }
            None => {
                particles_out.push(name.clone());
                polarizations.push(core.polarizations[n_in + pos].clone());
                block.push(0);
            }
        }
    }
    // Particles are identical only with the same polarization.
    let species: Vec<String> = particles_out
        .iter()
        .zip(&polarizations[n_in..])
        .map(|(name, pol)| format!("{name}{pol:?}"))
        .collect();
    let permutations = block_permutations(&species, &block);

    let decay_diagrams: Vec<&[Diagram]> = assignment
        .iter()
        .map(|(_, _, s)| s.set.diagrams.as_slice())
        .collect();
    let mut diagrams = Vec::new();
    // Each graph once. Two ways to reach one graph must agree on its sign, which is a
    // property of the graph; they can disagree on which of its lines is forced on shell
    // only where the core itself holds a line that could be the resonance, with the
    // identical products — then no one line is the decay's.
    let mut seen: HashMap<CanonicalDiagram, (i8, CanonicalDiagram)> = HashMap::new();
    let tuples: Vec<Vec<&Diagram>> = decay_diagrams
        .iter()
        .map(|d| d.iter())
        .multi_cartesian_product()
        .collect();
    for core_diagram in &core.diagrams {
        for tuple in &tuples {
            let parts: Vec<(usize, ChainNode, &Diagram)> = assignment
                .iter()
                .zip(tuple)
                .map(|(&(pos, node, _), &d)| (n_in + pos, node, d))
                .collect();
            let glued = glue(core_diagram, &parts);
            for permutation in &permutations {
                let d = relabelled(&glued, permutation, n_in, model);
                let mut unsigned = d.clone();
                unsigned.sign = 1;
                let flagged = unsigned.canonical(model);
                for p in &mut unsigned.props {
                    p.onshell = OnShell::Free;
                }
                let graph = unsigned.canonical(model);
                match seen.get(&graph) {
                    None => {
                        seen.insert(graph, (d.sign, flagged));
                        diagrams.push(d);
                    }
                    Some((sign, first)) => {
                        assert_eq!(
                            *sign, d.sign,
                            "one stitched graph reached twice with opposite signs"
                        );
                        if *first != flagged {
                            return Err(format!(
                                "in {} > {}, one diagram holds two lines either of which is the \
                                 decay's resonance with its products, identical particles \
                                 between the decay and the rest of the process; which of \
                                 them is on shell is ambiguous",
                                core.particles_in.join(" "),
                                particles_out.join(" ")
                            ));
                        }
                    }
                }
            }
        }
    }

    let subprocess = format!(
        "{} > {}",
        core.particles_in.join(" "),
        particles_out.join(" ")
    );
    let decays = assignment.iter().map(|(_, _, s)| s.key.as_str()).join(", ");
    let unpermuted = core.diagrams.len() * tuples.len();
    if node.0 == 0 {
        info!(
            "{} diagrams for {subprocess} ({} > {} with {decays})",
            diagrams.len(),
            core.particles_in.join(" "),
            core.particles_out.join(" ")
        );
    }
    debug!(
        "{subprocess}: {unpermuted} glued diagrams, {} after permuting identical particles \
         between decays ({} permutations)",
        diagrams.len(),
        permutations.len()
    );
    Ok(ChainSet {
        key: format!(
            "{} > {} [{}]",
            core.particles_in.join(" "),
            core.particles_out.join(" "),
            decays
        ),
        set: DiagramSet {
            particles_in: core.particles_in.clone(),
            particles_out,
            polarizations,
            diagrams,
        },
    })
}

/// `core` with each `(leg, node, decay)` glued in: the core's leg `leg` and the decay's
/// incoming leg become one propagator, flagged forced on shell, from the core vertex to the
/// decay vertex; the decay's products take the leg's place in the final state.
///
/// Momenta and sign are left for [`relabelled`] to rebuild.
fn glue(core: &Diagram, parts: &[(usize, ChainNode, &Diagram)]) -> Diagram {
    let n_in = core.n_in;
    // New index of every core leg that stays, and of every decay's products.
    let mut core_leg = vec![None; core.legs.len()];
    let mut product_leg: Vec<Vec<LegIdx>> = vec![Vec::new(); parts.len()];
    let mut legs: Vec<Leg> = Vec::new();
    for (k, leg) in core.legs.iter().enumerate() {
        match parts.iter().position(|(at, _, _)| *at == k) {
            Some(i) => {
                let decay = parts[i].2;
                for product in &decay.legs[decay.n_in..] {
                    let idx = LegIdx(legs.len());
                    product_leg[i].push(idx);
                    legs.push(Leg {
                        leg_idx: idx,
                        incoming: false,
                        ..product.clone()
                    });
                }
            }
            None => {
                let idx = LegIdx(legs.len());
                core_leg[k] = Some(idx);
                legs.push(Leg {
                    leg_idx: idx,
                    incoming: k < n_in,
                    ..leg.clone()
                });
            }
        }
    }

    let n_ext = legs.len();
    let placeholder = vec![0i8; n_ext];
    let n_core_props = core.props.len();
    let n_decay_props: usize = parts.iter().map(|(_, _, d)| d.props.len()).sum();
    let glue_prop = |i: usize| PropIdx(n_core_props + n_decay_props + i);

    let mut props: Vec<Prop> = core
        .props
        .iter()
        .map(|p| Prop {
            momentum: placeholder.clone(),
            ..p.clone()
        })
        .collect();
    let mut vertices: Vec<super::diagram::Vertex> = core
        .vertices
        .iter()
        .map(|v| super::diagram::Vertex {
            rays: v
                .rays
                .iter()
                .map(|&ray| match ray {
                    Ray::Leg(k) => match parts.iter().position(|(at, _, _)| *at == k.0) {
                        Some(i) => Ray::Prop {
                            prop: glue_prop(i),
                            end: 0,
                        },
                        None => Ray::Leg(core_leg[k.0].expect("a kept core leg")),
                    },
                    ray => ray,
                })
                .collect(),
            ..v.clone()
        })
        .collect();

    let mut provenance = core.provenance.clone();
    let mut glue_ends = Vec::with_capacity(parts.len());
    for (i, &(at, node, decay)) in parts.iter().enumerate() {
        let vertex_offset = vertices.len();
        let prop_offset = props.len();
        let shift = |(v, s): (VtxIdx, super::diagram::RaySlot)| (VtxIdx(v.0 + vertex_offset), s);
        props.extend(decay.props.iter().map(|p| Prop {
            endpoints: p.endpoints.map(shift),
            momentum: placeholder.clone(),
            ..p.clone()
        }));
        vertices.extend(decay.vertices.iter().map(|v| {
            super::diagram::Vertex {
                rays: v
                    .rays
                    .iter()
                    .map(|&ray| match ray {
                        Ray::Leg(LegIdx(0)) => Ray::Prop {
                            prop: glue_prop(i),
                            end: 1,
                        },
                        Ray::Leg(LegIdx(l)) => Ray::Leg(product_leg[i][l - decay.n_in]),
                        Ray::Prop { prop, end } => Ray::Prop {
                            prop: PropIdx(prop.0 + prop_offset),
                            end,
                        },
                    })
                    .collect(),
                ..v.clone()
            }
        }));
        provenance.decays.push(DecayOrigin {
            node,
            prop: glue_prop(i),
        });
        provenance
            .decays
            .extend(decay.provenance.decays.iter().map(|o| DecayOrigin {
                node: o.node,
                prop: PropIdx(o.prop.0 + prop_offset),
            }));
        let core_end = core.leg_attachment(LegIdx(at));
        let (decay_vertex, decay_slot) = decay.leg_attachment(LegIdx(0));
        glue_ends.push((
            decay.legs[0].particle,
            [
                core_end,
                (VtxIdx(decay_vertex.0 + vertex_offset), decay_slot),
            ],
        ));
    }
    for (particle, endpoints) in glue_ends {
        props.push(Prop {
            particle,
            endpoints,
            momentum: placeholder.clone(),
            onshell: OnShell::Forced,
        });
    }
    provenance.decays.sort_by_key(|o| o.node);

    Diagram {
        legs,
        props,
        vertices,
        sign: 1,
        symmetry_factor: core.symmetry_factor
            * parts
                .iter()
                .map(|(_, _, d)| d.symmetry_factor)
                .product::<usize>(),
        n_in,
        provenance,
    }
}

/// `diagram` with its final-state legs relabelled, final-state position `i` moving to
/// `permutation[i]`, and its momenta and sign rebuilt from the graph.
fn relabelled(diagram: &Diagram, permutation: &[usize], n_in: usize, model: &UFOModel) -> Diagram {
    let map = |l: LegIdx| {
        if l.0 < n_in {
            l
        } else {
            LegIdx(n_in + permutation[l.0 - n_in])
        }
    };
    let mut legs = diagram.legs.clone();
    for leg in &diagram.legs {
        let to = map(leg.leg_idx);
        legs[to.0] = Leg {
            leg_idx: to,
            ..leg.clone()
        };
    }
    let mut d = Diagram {
        legs,
        vertices: diagram
            .vertices
            .iter()
            .map(|v| super::diagram::Vertex {
                rays: v
                    .rays
                    .iter()
                    .map(|&ray| match ray {
                        Ray::Leg(l) => Ray::Leg(map(l)),
                        ray => ray,
                    })
                    .collect(),
                ..v.clone()
            })
            .collect(),
        ..diagram.clone()
    };
    for p in 0..d.props.len() {
        d.props[p].momentum = d.tree_momentum(PropIdx(p));
    }
    d.sign = d.fermion_pairing_sign(model) * d.fermion_line_sign(model);
    d
}

/// Every relabelling of the final state that moves identical particles between blocks: per
/// particle, every distinct arrangement of its positions over the blocks that keeps each
/// block's count, a block's own positions kept in order. The identity comes first.
///
/// Permutations inside one block are left out: a core's or a decay's enumeration already
/// holds every diagram they reach.
fn block_permutations(particles: &[String], block: &[usize]) -> Vec<Vec<usize>> {
    let species: Vec<&String> = particles.iter().unique().collect();
    // Per particle: its positions, and every arrangement of their blocks.
    let per_species: Vec<(Vec<usize>, Vec<Vec<usize>>)> = species
        .iter()
        .map(|&s| {
            let positions: Vec<usize> = (0..particles.len())
                .filter(|&i| &particles[i] == s)
                .collect();
            let blocks: Vec<usize> = positions.iter().map(|&i| block[i]).collect();
            (positions, arrangements(&blocks))
        })
        .collect();
    per_species
        .iter()
        .map(|(_, arrangements)| arrangements.iter())
        .multi_cartesian_product()
        .map(|choice| {
            let mut permutation: Vec<usize> = (0..particles.len()).collect();
            for ((positions, _), arrangement) in per_species.iter().zip(choice) {
                let original: Vec<usize> = positions.iter().map(|&i| block[i]).collect();
                for b in original.iter().unique() {
                    let from = positions
                        .iter()
                        .zip(&original)
                        .filter(|(_, ob)| *ob == b)
                        .map(|(&p, _)| p);
                    let to = positions
                        .iter()
                        .zip(arrangement)
                        .filter(|(_, nb)| *nb == b)
                        .map(|(&p, _)| p);
                    for (f, t) in from.zip(to) {
                        permutation[f] = t;
                    }
                }
            }
            permutation
        })
        .collect()
}

/// Every distinct ordering of the multiset `items`, `items` itself first.
fn arrangements(items: &[usize]) -> Vec<Vec<usize>> {
    fn extend(
        counts: &mut [(usize, usize)],
        current: &mut Vec<usize>,
        out: &mut Vec<Vec<usize>>,
        n: usize,
    ) {
        if current.len() == n {
            out.push(current.clone());
            return;
        }
        for i in 0..counts.len() {
            if counts[i].1 == 0 {
                continue;
            }
            counts[i].1 -= 1;
            current.push(counts[i].0);
            extend(counts, current, out, n);
            current.pop();
            counts[i].1 += 1;
        }
    }
    let mut out = Vec::new();
    let mut counts: Vec<(usize, usize)> = items
        .iter()
        .unique()
        .map(|&v| (v, items.iter().filter(|&&w| w == v).count()))
        .collect();
    extend(&mut counts, &mut Vec::new(), &mut out, items.len());
    let identity = out
        .iter()
        .position(|a| a == items)
        .expect("the multiset's own order is among its arrangements");
    out.swap(0, identity);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrangements_are_distinct_and_start_with_the_identity() {
        let a = arrangements(&[1, 2, 1, 2]);
        assert_eq!(a.len(), 6);
        assert_eq!(a[0], [1, 2, 1, 2]);
        assert_eq!(a.iter().unique().count(), 6);
    }

    #[test]
    fn block_permutations_move_identical_particles_between_blocks_only() {
        let names = |s: &[&str]| s.iter().map(|n| n.to_string()).collect::<Vec<_>>();
        // e+ e- | e+ e-: 2 ways for the e+, 2 for the e-.
        let p = block_permutations(&names(&["e+", "e-", "e+", "e-"]), &[1, 1, 2, 2]);
        assert_eq!(p.len(), 4);
        assert_eq!(p[0], [0, 1, 2, 3]);
        // Two identical particles inside one block: nothing to move.
        let p = block_permutations(&names(&["u", "u", "g"]), &[1, 1, 0]);
        assert_eq!(p, [vec![0, 1, 2]]);
    }
}
