//! An integration's configuration shares and channel membership, as the
//! integrate artifact banks them ([`ConfigShare`], [`ChannelMember`]).
//!
//! The integrands split every point's value over the configurations of each
//! evaluator they sum, in [`share_offsets`](crate::hadronic::share_offsets)'s
//! layout: per evaluator, its configurations in `AMP2` order, then one slot for
//! the weight no configuration carried. The integration accumulates each part
//! per channel ([`ChannelIntegration::shares`]). The functions here sum the
//! parts over channels and say which sets and diagrams each part and each
//! channel stands for. [`ConfigShare`] documents the partition.

use crate::artifact::{ChannelMember, ConfigShare, SetRef};
use crate::diagrams::diagram::Diagram;
use crate::diagrams::DiagramSet;
use crate::hadronic::ChannelIntegration;
use crate::helas::eval::{config_groups, config_tag, AmplitudeEvaluator};
use crate::multiplicity::MultiplicitySum;
use crate::proton::FlavorGroups;
use crate::ufo::UFOModel;

/// Share `k` summed over every channel's term: integrals added, errors in
/// quadrature, both scaled by `unit`, as the cross section's are.
fn summed(per_channel: &[ChannelIntegration], k: usize, unit: f64) -> (f64, f64) {
    let (mut sigma, mut variance) = (0.0, 0.0);
    for c in per_channel {
        if let Some(r) = c.shares.get(k) {
            sigma += r.integral;
            variance += r.std_dev * r.std_dev;
        }
    }
    (sigma * unit, variance.sqrt() * unit)
}

/// One evaluator's entries: a [`ConfigShare`] per configuration and the
/// unattributed one, its parts from `offset` on.
fn evaluator_shares(
    eval: &AmplitudeEvaluator,
    set: usize,
    members: Vec<SetRef>,
    offset: usize,
    per_channel: &[ChannelIntegration],
    unit: f64,
) -> Vec<ConfigShare> {
    let configs = eval.config_members();
    let mut carried = vec![false; eval.n_diagrams()];
    for d in configs.iter().flatten() {
        carried[*d] = true;
    }
    let contact: Vec<usize> = (0..eval.n_diagrams()).filter(|&d| !carried[d]).collect();
    let entry = |c: Option<usize>, diagrams: Vec<usize>| {
        let k = offset + c.unwrap_or(configs.len());
        let (sigma, sigma_err) = summed(per_channel, k, unit);
        ConfigShare {
            set,
            configuration: c,
            diagrams,
            contact_diagrams: contact.clone(),
            members: members.clone(),
            sigma,
            sigma_err,
        }
    };
    let mut out: Vec<ConfigShare> = configs
        .iter()
        .enumerate()
        .map(|(c, diagrams)| entry(Some(c), diagrams.clone()))
        .collect();
    out.push(entry(None, Vec::new()));
    out
}

/// The evaluator configuration a channel built from `diagrams` (indices into
/// the evaluator's own set) is counted in: the one holding its first diagram
/// that any configuration holds.
fn config_of(configs: &[Vec<usize>], diagrams: &[usize]) -> Option<usize> {
    diagrams
        .iter()
        .find_map(|d| configs.iter().position(|members| members.contains(d)))
}

/// A fixed-energy run's shares and channel members.
///
/// `evals` are the run's subprocesses as compiled, one per non-empty set of
/// `sets` in order. The run's channels, where it has more than one grid's
/// worth, are the [`config_groups`] of every subprocess's diagrams taken
/// together ([`crate::hadronic::channel_diagrams`]); a channel whose group
/// spans several subprocesses serves each of them.
pub fn fixed_energy(
    sets: &[DiagramSet],
    evals: &[AmplitudeEvaluator],
    model: &UFOModel,
    per_channel: &[ChannelIntegration],
    unit: f64,
) -> (Vec<ConfigShare>, Vec<Vec<ChannelMember>>) {
    let set_of: Vec<usize> = sets
        .iter()
        .enumerate()
        .filter(|(_, s)| !s.diagrams.is_empty())
        .map(|(k, _)| k)
        .collect();
    assert_eq!(set_of.len(), evals.len(), "one evaluator per non-empty set");
    let identity = |k: usize| SetRef {
        set: k,
        final_order: (0..sets[k].particles_out.len()).collect(),
    };
    let mut shares = Vec::new();
    let mut offsets = Vec::with_capacity(evals.len());
    for (eval, &k) in evals.iter().zip(&set_of) {
        offsets.push(shares.len());
        shares.extend(evaluator_shares(
            eval,
            k,
            vec![identity(k)],
            shares.len(),
            per_channel,
            unit,
        ));
    }

    // Flattened diagram index → (subprocess, its own index).
    let mut flat: Vec<(usize, usize)> = Vec::new();
    let mut diagrams: Vec<Diagram> = Vec::new();
    for (i, &k) in set_of.iter().enumerate() {
        flat.extend((0..sets[k].diagrams.len()).map(|d| (i, d)));
        diagrams.extend(sets[k].diagrams.iter().cloned());
    }
    let groups = config_groups(&diagrams, model);
    let members = if groups.len() == per_channel.len() && !groups.is_empty() {
        let configs: Vec<Vec<Vec<usize>>> = evals.iter().map(|e| e.config_members()).collect();
        groups
            .iter()
            .map(|group| {
                let mut by_sub: Vec<(usize, Vec<usize>)> = Vec::new();
                for &f in group {
                    let (i, d) = flat[f];
                    match by_sub.iter_mut().find(|(j, _)| *j == i) {
                        Some((_, ds)) => ds.push(d),
                        None => by_sub.push((i, vec![d])),
                    }
                }
                by_sub
                    .into_iter()
                    .map(|(i, ds)| ChannelMember {
                        set: identity(set_of[i]),
                        share: config_of(&configs[i], &ds).map(|c| offsets[i] + c),
                        diagrams: ds,
                    })
                    .collect()
            })
            .collect()
    } else {
        vec![Vec::new(); per_channel.len()]
    };
    (shares, members)
}

/// A member's own diagrams for each of the representative's [`config_groups`]
/// groups, by [`config_tag`]: the member group whose diagrams carry the tag of
/// the representative group's first. The member's legs stand where the
/// representative's do, so a tag is comparable across the two; a group with no
/// counterpart maps to nothing.
fn member_groups(rep: &[Diagram], member: &[Diagram], model: &UFOModel) -> Vec<Vec<usize>> {
    let rep_groups = config_groups(rep, model);
    let own = config_groups(member, model);
    if own == rep_groups {
        return own;
    }
    let own_tags: Vec<_> = own
        .iter()
        .map(|g| config_tag(&member[g[0]], model))
        .collect();
    rep_groups
        .iter()
        .map(|g| {
            let tag = config_tag(&rep[g[0]], model);
            own_tags
                .iter()
                .position(|t| *t == tag)
                .map(|i| own[i].clone())
                .unwrap_or_default()
        })
        .collect()
}

/// What [`hadronic`] keeps per flavour group: its first entry in the shares,
/// its members as enumerated sets, and per member its own diagrams of each of
/// the representative's channel groups ([`member_groups`]).
struct GroupLayout {
    offset: usize,
    origins: Vec<SetRef>,
    own: Vec<Vec<Vec<usize>>>,
}

/// A proton-beam run's shares and channel members: per part of `integ`, per
/// flavour group of `groups[part]`, its representative's evaluator's shares
/// with every member set listed, and per channel every `(group, channel)` pair
/// it serves, once per member of the pair's group.
///
/// # Errors
///
/// Where a group member is not a reordering of any set of `sets`.
pub fn hadronic(
    sets: &[DiagramSet],
    groups: &[FlavorGroups],
    integ: &MultiplicitySum<'_>,
    model: &UFOModel,
    per_channel: &[ChannelIntegration],
    unit: f64,
) -> Result<(Vec<ConfigShare>, Vec<Vec<ChannelMember>>), String> {
    assert_eq!(
        groups.len(),
        integ.parts().len(),
        "one decomposition per part"
    );
    let mut shares = Vec::new();
    let mut layout: Vec<Vec<GroupLayout>> = Vec::new();
    for part in groups {
        let mut part_layout = Vec::new();
        for g in part.groups() {
            let origins = g.member_origins(sets, model)?;
            let rep = g.diagrams();
            let own: Vec<Vec<Vec<usize>>> = (0..g.members().len())
                .map(|m| member_groups(rep, &g.member_diagram_set(m).diagrams, model))
                .collect();
            let offset = shares.len();
            shares.extend(evaluator_shares(
                g.evaluator(),
                origins[0].set,
                origins.clone(),
                offset,
                per_channel,
                unit,
            ));
            part_layout.push(GroupLayout {
                offset,
                origins,
                own,
            });
        }
        layout.push(part_layout);
    }
    assert_eq!(shares.len(), integ.share_count(), "one entry per share");

    let mut members = Vec::with_capacity(per_channel.len());
    for (k, part) in integ.parts().iter().enumerate() {
        let decomposition = &groups[k];
        for pairs in part.channel_members() {
            let mut entries = Vec::new();
            for id in pairs {
                let g = &decomposition.groups()[id.group];
                let GroupLayout {
                    offset,
                    origins,
                    own,
                } = &layout[k][id.group];
                let rep_groups = config_groups(g.diagrams(), model);
                let share = config_of(&g.evaluator().config_members(), &rep_groups[id.channel])
                    .map(|c| offset + c);
                for (origin, own) in origins.iter().zip(own) {
                    entries.push(ChannelMember {
                        set: origin.clone(),
                        diagrams: own.get(id.channel).cloned().unwrap_or_default(),
                        share,
                    });
                }
            }
            members.push(entries);
        }
    }
    Ok((shares, members))
}
