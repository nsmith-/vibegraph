//! The ranking JSON `vibegraph inspect --ranking` writes: an integration's
//! configurations ordered by their share of the observable, beside the
//! sampler's per-channel diagnostics, under the ids `vibegraph enumerate` gives
//! its sets, diagrams and drawings.
//!
//! The schema is documented with the knowledge bundle
//! (`research/kb/pipeline/ranking-json.md`); [`SCHEMA_VERSION`] is its version.
//! Everything is read from the integrate artifact: nothing is integrated or
//! enumerated again.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;
use vibegraph::artifact::{ConfigShare, IntegrateArtifact};
use vibegraph::hadronic::Observable;
use vibegraph::helas::eval::config_groups;
use vibegraph::runcard::BeamMode;

use crate::enumerate::{
    configuration_id, diagram_id, set_id, svg_path, SCHEMA_VERSION as DIAGRAMS_SCHEMA_VERSION,
    SUMMARY_FILENAME,
};
use crate::error::{err, CliError, ErrorKind};
use crate::integrate::channel_key_object;
use crate::output::{ChannelKeyObject, ModelRef, ObservableName};

/// The version of the ranking schema.
///
/// Adding a key keeps it; removing, renaming or re-meaning one bumps it.
pub(crate) const SCHEMA_VERSION: u32 = 1;

/// The ranking document.
#[derive(Debug, Serialize)]
pub(crate) struct Ranking {
    schema_version: u32,
    process: String,
    model: ModelRef,
    observable: ObservableName,
    /// The unit of every `sigma` and `sigma_err` in the document.
    unit: &'static str,
    sigma: f64,
    sigma_err: f64,
    chi2_per_dof: Option<f64>,
    seed: u64,
    diagram_set_digest: String,
    artifact_format_version: u32,
    /// `proton`, `fixed_energy` or `decay`.
    beams: &'static str,
    /// The enhancement weight the configuration shares were split by:
    /// `amp2` (`sde_strategy = 1`) or `channel_cut` (`sde_strategy = 2`).
    weight: Option<&'static str>,
    drawings: Drawings,
    flavour_groups: Vec<FlavourGroup>,
    sets: Vec<SetEntry>,
    configurations: Vec<RankedConfiguration>,
    unranked: Vec<Unranked>,
    channels: Vec<ChannelEntry>,
    diagrams: Vec<DiagramEntry>,
}

/// Where the `svg` paths come from.
#[derive(Debug, Serialize)]
struct Drawings {
    /// Whether a `diagrams.json` with this run's `diagram_set_digest` was read,
    /// so the paths are the drawings it records (`null` where it drew none).
    /// Otherwise every diagram carries the conventional path, which exists
    /// only where `enumerate` drew it.
    paired: bool,
    /// The `diagrams.json` read, as the command was given it.
    summary: Option<String>,
}

#[derive(Debug, Serialize)]
struct FlavourGroup {
    id: String,
    representative: String,
    members: Vec<GroupMember>,
}

#[derive(Debug, Serialize)]
struct GroupMember {
    set: String,
    /// Outgoing leg `i` of the integrated term is outgoing leg
    /// `final_order[i]` of the set.
    final_order: Vec<usize>,
}

#[derive(Debug, Serialize)]
struct SetEntry {
    id: String,
    label: String,
    mirrored: bool,
    diagrams: usize,
    /// The set's matrix element was integrated (a fixed-energy subprocess, a
    /// flavour group's representative), so the ranked configurations name it.
    integrated: bool,
    flavour_group: Option<String>,
}

#[derive(Debug, Serialize)]
struct RankedConfiguration {
    rank: usize,
    id: String,
    set: String,
    /// The configuration's diagrams that carry an amplitude the colour basis
    /// references.
    diagrams: Vec<String>,
    sigma: f64,
    sigma_err: f64,
    share: f64,
    cumulative_share: f64,
    /// The same configuration in each set the share includes, as the channels'
    /// membership records it.
    members: Vec<MemberConfiguration>,
    /// The channels whose maps serve the configuration, by index.
    channels: Vec<usize>,
}

#[derive(Debug, Serialize)]
struct MemberConfiguration {
    set: String,
    configuration: Option<String>,
    diagrams: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Unranked {
    set: String,
    /// `no_configuration`, `contact` or `colour_null`.
    reason: &'static str,
    diagrams: Vec<String>,
    sigma: Option<f64>,
    sigma_err: Option<f64>,
    share: Option<f64>,
}

#[derive(Debug, Serialize)]
struct ChannelEntry {
    index: usize,
    key: ChannelKeyObject,
    alpha: f64,
    neval: usize,
    sigma: f64,
    sigma_err: f64,
    chi2_per_dof: Option<f64>,
    /// The ranked configurations the channel's members are counted in,
    /// each once, in member order.
    configurations: Vec<String>,
    members: Vec<ChannelMemberEntry>,
}

#[derive(Debug, Serialize)]
struct ChannelMemberEntry {
    set: String,
    final_order: Vec<usize>,
    /// The set's own configuration holding the member's diagrams.
    configuration: Option<String>,
    diagrams: Vec<String>,
    /// The ranked configuration the member is counted in.
    counted_in: Option<String>,
}

#[derive(Debug, Serialize)]
struct DiagramEntry {
    id: String,
    set: String,
    configuration: Option<String>,
    coupling_orders: BTreeMap<String, usize>,
    svg: Option<String>,
}

/// A configuration's id across the whole card: `s<k>.c<j>`.
fn qualified_configuration(k: usize, c: usize) -> String {
    format!("{}.{}", set_id(k), configuration_id(c))
}

/// What a `diagrams.json` contributes: its drawing paths, and its
/// configurations to check this build's grouping against.
pub(crate) struct DiagramsSummary {
    path: PathBuf,
    svg: BTreeMap<String, Option<String>>,
    configurations: BTreeMap<String, Vec<String>>,
}

impl DiagramsSummary {
    /// Read the summary at `path`, or `path/diagrams.json` for a directory,
    /// and refuse it unless it describes the diagram sets `digest` names.
    pub(crate) fn read(path: &Path, digest: &str) -> Result<Self, CliError> {
        let path = if path.is_dir() {
            path.join(SUMMARY_FILENAME)
        } else {
            path.to_path_buf()
        };
        let unreadable = |why: String| {
            err(
                ErrorKind::Io,
                format!("cannot read the diagram summary {}: {why}", path.display()),
            )
        };
        let text = std::fs::read_to_string(&path).map_err(|e| unreadable(e.to_string()))?;
        let summary: Value = serde_json::from_str(&text).map_err(|e| unreadable(e.to_string()))?;
        let version = summary["schema_version"].as_u64();
        if version != Some(u64::from(DIAGRAMS_SCHEMA_VERSION)) {
            return Err(unreadable(format!(
                "schema version {version:?}, where this build reads {DIAGRAMS_SCHEMA_VERSION}"
            )));
        }
        let theirs = summary["diagram_set_digest"].as_str().unwrap_or_default();
        if theirs != digest {
            return Err(err(
                ErrorKind::Artifact,
                format!(
                    "{} describes other diagrams than the integration: its diagram_set_digest \
                     is {theirs}, the artifact's is {digest}; pass the output directory of the \
                     `vibegraph enumerate` run the integration was started from, or enumerate \
                     the artifact's proc card again",
                    path.display()
                ),
            ));
        }
        let mut svg = BTreeMap::new();
        let mut configurations = BTreeMap::new();
        let strings = |v: &Value| -> Vec<String> {
            v.as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|s| s.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default()
        };
        for set in summary["sets"].as_array().into_iter().flatten() {
            let k = set["id"].as_str().unwrap_or_default();
            for c in set["configurations"].as_array().into_iter().flatten() {
                let id = format!("{k}.{}", c["id"].as_str().unwrap_or_default());
                configurations.insert(id, strings(&c["diagrams"]));
            }
            for d in set["diagram_list"].as_array().into_iter().flatten() {
                svg.insert(
                    d["id"].as_str().unwrap_or_default().to_string(),
                    d["svg"].as_str().map(str::to_string),
                );
            }
        }
        Ok(DiagramsSummary {
            path,
            svg,
            configurations,
        })
    }
}

fn malformed(why: String) -> CliError {
    err(
        ErrorKind::Artifact,
        format!("the artifact's configuration record is inconsistent: {why}"),
    )
}

/// Build the ranking of `artifact`'s configurations, measuring `observable`,
/// with the drawings of `diagrams` where it is given.
pub(crate) fn build(
    artifact: &IntegrateArtifact,
    observable: Observable,
    diagrams: Option<&DiagramsSummary>,
) -> Result<Ranking, CliError> {
    let loaded = artifact.loaded_model().map_err(|e| {
        err(
            ErrorKind::Artifact,
            format!("cannot rebuild the embedded model: {e}"),
        )
    })?;
    let model = &loaded.model;
    let sets = &artifact.sets;
    let n_sets = sets.len();

    // Each set's configurations as `enumerate` numbers them, per diagram.
    let configuration_of: Vec<Vec<Option<usize>>> = sets
        .iter()
        .map(|set| {
            let mut of = vec![None; set.diagrams.len()];
            for (c, group) in config_groups(&set.diagrams, model).iter().enumerate() {
                for &d in group {
                    of[d] = Some(c);
                }
            }
            of
        })
        .collect();
    let check_set = |k: usize| {
        if k < n_sets {
            Ok(())
        } else {
            Err(malformed(format!("set {k} of {n_sets}")))
        }
    };
    let check_diagram = |k: usize, d: usize| {
        if d < sets[k].diagrams.len() {
            Ok(())
        } else {
            Err(malformed(format!(
                "diagram {d} of {} in set {}",
                sets[k].diagrams.len(),
                set_id(k)
            )))
        }
    };
    // The configuration of set `k` holding all of `diagrams`, `None` for no
    // diagrams; an error where they straddle configurations or hold none.
    let configuration_holding = |k: usize, diagrams: &[usize]| -> Result<Option<usize>, CliError> {
        check_set(k)?;
        let Some(&first) = diagrams.first() else {
            return Ok(None);
        };
        let mut found = None;
        for &d in diagrams {
            check_diagram(k, d)?;
            let c = configuration_of[k][d];
            if c.is_none() || (found.is_some() && c != found) {
                return Err(malformed(format!(
                    "diagrams {diagrams:?} of {} are not one of its configurations",
                    set_id(k)
                )));
            }
            found = c;
        }
        debug_assert!(configuration_of[k][first].is_some());
        Ok(found)
    };
    let ids = |k: usize, diagrams: &[usize]| -> Vec<String> {
        diagrams.iter().map(|&d| diagram_id(k, d)).collect()
    };

    let beams = match observable {
        Observable::PartialWidth => "decay",
        Observable::CrossSection => match artifact.run_card.beam_mode() {
            BeamMode::Proton => "proton",
            BeamMode::FixedEnergy => "fixed_energy",
        },
    };
    let weight = artifact
        .run_card
        .get("SDE_strategy")
        .map(|_| match artifact.run_card.int("SDE_strategy") {
            1 => Some("amp2"),
            2 => Some("channel_cut"),
            _ => None,
        })
        .unwrap_or(None);
    let share_of = |sigma: f64| {
        if artifact.sigma_pb != 0.0 {
            sigma / artifact.sigma_pb
        } else {
            0.0
        }
    };

    // Evaluators, in the artifact's order, and their members.
    let mut evaluators: Vec<(usize, &ConfigShare)> = Vec::new();
    for c in &artifact.configurations {
        check_set(c.set)?;
        for m in &c.members {
            check_set(m.set)?;
        }
        if !evaluators.iter().any(|(k, _)| *k == c.set) {
            evaluators.push((c.set, c));
        }
    }
    let mut flavour_groups = Vec::new();
    let mut group_of: Vec<Option<String>> = vec![None; n_sets];
    if beams == "proton" {
        for (k, first) in &evaluators {
            let id = format!("g{}", flavour_groups.len());
            for m in &first.members {
                group_of[m.set] = Some(id.clone());
            }
            flavour_groups.push(FlavourGroup {
                id,
                representative: set_id(*k),
                members: first
                    .members
                    .iter()
                    .map(|m| GroupMember {
                        set: set_id(m.set),
                        final_order: m.final_order.clone(),
                    })
                    .collect(),
            });
        }
    }

    // The ranked configurations' ids, per artifact entry.
    let mut share_id: Vec<Option<String>> = vec![None; artifact.configurations.len()];
    for (i, c) in artifact.configurations.iter().enumerate() {
        if c.configuration.is_some() {
            let j = configuration_holding(c.set, &c.diagrams)?.ok_or_else(|| {
                malformed(format!(
                    "configuration {:?} of {} has no diagrams",
                    c.configuration,
                    set_id(c.set)
                ))
            })?;
            let id = qualified_configuration(c.set, j);
            if let Some(summary) = diagrams {
                let theirs = summary.configurations.get(&id);
                let ours = ids(c.set, &c.diagrams);
                if !theirs.is_some_and(|t| ours.iter().all(|d| t.contains(d))) {
                    return Err(err(
                        ErrorKind::Artifact,
                        format!(
                            "{} groups {id} as {theirs:?}, where this build holds {ours:?} in \
                             it: the summary was written by a build that forms configurations \
                             differently; run `vibegraph enumerate` again",
                            summary.path.display()
                        ),
                    ));
                }
            }
            share_id[i] = Some(id);
        }
    }

    let mut channels = Vec::with_capacity(artifact.channels.len());
    let mut channels_of: Vec<Vec<usize>> = vec![Vec::new(); artifact.configurations.len()];
    let mut member_configurations: Vec<Vec<MemberConfiguration>> =
        (0..artifact.configurations.len())
            .map(|_| Vec::new())
            .collect();
    for (index, channel) in artifact.channels.iter().enumerate() {
        let mut members = Vec::with_capacity(channel.members.len());
        let mut counted = Vec::new();
        for m in &channel.members {
            let k = m.set.set;
            let own = configuration_holding(k, &m.diagrams)?.map(|c| qualified_configuration(k, c));
            let counted_in = match m.share {
                Some(i) => {
                    let id = share_id
                        .get(i)
                        .ok_or_else(|| {
                            malformed(format!(
                                "channel {index} counts a member in share {i} of {}",
                                share_id.len()
                            ))
                        })?
                        .clone();
                    if !channels_of[i].contains(&index) {
                        channels_of[i].push(index);
                    }
                    let entry = MemberConfiguration {
                        set: set_id(k),
                        configuration: own.clone(),
                        diagrams: ids(k, &m.diagrams),
                    };
                    if !member_configurations[i]
                        .iter()
                        .any(|e| e.set == entry.set && e.diagrams == entry.diagrams)
                    {
                        member_configurations[i].push(entry);
                    }
                    id
                }
                None => None,
            };
            if let Some(id) = &counted_in {
                if !counted.contains(id) {
                    counted.push(id.clone());
                }
            }
            members.push(ChannelMemberEntry {
                set: set_id(k),
                final_order: m.set.final_order.clone(),
                configuration: own,
                diagrams: ids(k, &m.diagrams),
                counted_in,
            });
        }
        channels.push(ChannelEntry {
            index,
            key: channel_key_object(channel.key),
            alpha: channel.alpha,
            neval: channel.neval,
            sigma: channel.sigma_pb,
            sigma_err: channel.sigma_err_pb,
            chi2_per_dof: channel.chi2_per_dof,
            configurations: counted,
            members,
        });
    }

    // Descending σ_c; a stable sort keeps the artifact's order (evaluator by
    // evaluator, each in `AMP2` order) among equal values.
    let mut ranked: Vec<usize> = (0..artifact.configurations.len())
        .filter(|&i| share_id[i].is_some())
        .collect();
    ranked.sort_by(|&a, &b| {
        artifact.configurations[b]
            .sigma
            .total_cmp(&artifact.configurations[a].sigma)
    });
    let mut cumulative = 0.0;
    let mut configurations = Vec::with_capacity(ranked.len());
    for (rank, i) in ranked.into_iter().enumerate() {
        let c = &artifact.configurations[i];
        let share = share_of(c.sigma);
        cumulative += share;
        configurations.push(RankedConfiguration {
            rank: rank + 1,
            id: share_id[i].clone().expect("ranked entries have an id"),
            set: set_id(c.set),
            diagrams: ids(c.set, &c.diagrams),
            sigma: c.sigma,
            sigma_err: c.sigma_err,
            share,
            cumulative_share: cumulative,
            members: std::mem::take(&mut member_configurations[i]),
            channels: std::mem::take(&mut channels_of[i]),
        });
    }

    let mut unranked = Vec::new();
    for (k, first) in &evaluators {
        for c in artifact
            .configurations
            .iter()
            .filter(|c| c.set == *k && c.configuration.is_none())
        {
            unranked.push(Unranked {
                set: set_id(*k),
                reason: "no_configuration",
                diagrams: Vec::new(),
                sigma: Some(c.sigma),
                sigma_err: Some(c.sigma_err),
                share: Some(share_of(c.sigma)),
            });
        }
        for &d in &first.contact_diagrams {
            check_diagram(*k, d)?;
        }
        let (colour_null, contact): (Vec<usize>, Vec<usize>) = first
            .contact_diagrams
            .iter()
            .partition(|&&d| configuration_of[*k][d].is_some());
        for (reason, list) in [("contact", contact), ("colour_null", colour_null)] {
            if !list.is_empty() {
                unranked.push(Unranked {
                    set: set_id(*k),
                    reason,
                    diagrams: ids(*k, &list),
                    sigma: None,
                    sigma_err: None,
                    share: None,
                });
            }
        }
    }

    let set_entries = sets
        .iter()
        .enumerate()
        .map(|(k, set)| SetEntry {
            id: set_id(k),
            label: set.label(),
            mirrored: set.mirrored,
            diagrams: set.diagrams.len(),
            integrated: evaluators.iter().any(|(e, _)| *e == k),
            flavour_group: group_of[k].clone(),
        })
        .collect();
    let diagram_entries = sets
        .iter()
        .enumerate()
        .flat_map(|(k, set)| {
            let configuration_of = &configuration_of[k];
            set.diagrams.iter().enumerate().map(move |(j, d)| {
                let id = diagram_id(k, j);
                let svg = match diagrams {
                    Some(summary) => summary.svg.get(&id).cloned().flatten(),
                    None => Some(svg_path(k, j)),
                };
                DiagramEntry {
                    id,
                    set: set_id(k),
                    configuration: configuration_of[j].map(|c| qualified_configuration(k, c)),
                    coupling_orders: d.coupling_orders(model),
                    svg,
                }
            })
        })
        .collect();

    Ok(Ranking {
        schema_version: SCHEMA_VERSION,
        process: artifact.process.clone(),
        model: ModelRef {
            label: artifact.model.label(),
            digest: artifact.model.digest.clone(),
        },
        observable: observable.into(),
        unit: observable.unit(),
        sigma: artifact.sigma_pb,
        sigma_err: artifact.sigma_err_pb,
        chi2_per_dof: artifact.chi2_per_dof,
        seed: artifact.seed,
        diagram_set_digest: artifact.diagram_set_digest.clone(),
        artifact_format_version: artifact.format_version,
        beams,
        weight,
        drawings: Drawings {
            paired: diagrams.is_some(),
            summary: diagrams.map(|s| s.path.display().to_string()),
        },
        flavour_groups,
        sets: set_entries,
        configurations,
        unranked,
        channels,
        diagrams: diagram_entries,
    })
}

impl Ranking {
    /// How many configurations are ranked, and the first of them.
    pub(crate) fn headline(&self) -> (usize, Option<(&str, f64)>) {
        (
            self.configurations.len(),
            self.configurations
                .first()
                .map(|c| (c.id.as_str(), c.share)),
        )
    }
}
