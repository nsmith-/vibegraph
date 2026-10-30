//! A proton-beam cross section summed over final-state multiplicities.
//!
//! A proc card whose process lines differ in their number of outgoing legs
//! (`p p > e+ e- @0`, `add process p p > e+ e- j @1`, …) describes one sample
//! over several phase spaces of different dimension. MadEvent integrates each
//! `P<n>` directory on its own and unweights every channel of every directory
//! together. [`MultiplicitySum`] is the same arrangement: one [`ProtonIntegrand`]
//! per multiplicity, each keeping its own channels, grids of its own dimension
//! and channel selection weights, exposed as one list of channels offset by
//! multiplicity. The channel-split estimator is a sum of terms either way, so
//! the cross section is the sum of every part's terms and the unweighting pass
//! draws a channel `∝ w_maxⱼ` whatever multiplicity it belongs to.
//!
//! Padding every channel to the widest dimension would need no composite, but it
//! would give most grids axes nothing depends on and change the uniform stream
//! every channel draws.
//!
//! # Outgoing masses
//!
//! A part is one phase space, and the outgoing pole masses fix a phase space as
//! much as the leg count does: `p p > l+ l-` with `l` holding `ta` puts `ta+ ta-`
//! on the `m = 1.777` GeV mass shell and `e+ e-` on the light cone. A
//! [`ProtonIntegrand`] evaluates every group's `|M|²` at every point its channels
//! draw, and weighs the point by the density of every channel, so its groups must
//! share their outgoing masses. A multiplicity whose subprocesses differ in them
//! is therefore split further, one part per set of outgoing masses
//! ([`split_by_outgoing_masses`]). MadGraph separates them the same way: its
//! subprocess-group class (`group_subprocs.py`, `find_process_classes`) carries
//! every leg's mass, so such subprocesses never share a `P` directory, and each
//! directory is integrated over its own channels.

use crate::artifact::{ChannelKey, ChannelSampler};
use crate::budget::{integrate_channels, BlockAllocation, Budget, ConvergenceReport, StopSignal};
use crate::diagrams::{
    forbidden_onshell_ids, generate_from_proc_card_in, DiagramSet, EnumerationPool, SupportedCard,
};
use crate::hadronic::{combine_channels, ChannelIntegration};
use crate::onshell::{group_vetoes, subprocess_markings, OnShellVeto};
use crate::phasespace::channel::AlphaAdaptation;
use crate::phasespace::maps::{MapChoices, MapOptions, ProcessShape};
use crate::proton::{
    derive_flavor_groups, process_shape, FlavorGroups, PointAnatomy, ProtonEvent, ProtonIntegrand,
};
use crate::runcard::RunCard;
use crate::ufo::{EvaluatedModel, UFOModel};
use crate::unweight::ChannelIntegrand;
use crate::vegas::{GridRule, IterationCombination, VegasResult};

/// Split a proc card's enumeration by final-state multiplicity, in increasing
/// order of outgoing legs, each part keeping the enumeration's own order.
///
/// Empty sets are dropped, as [`derive_flavor_groups`]
/// drops them; an enumeration with no diagram at all comes back whole, as one
/// part, for that function to refuse by name.
///
/// More than one part without matching (`ickkw = 0`) is accepted, as MadGraph
/// accepts it, and warned about: the higher multiplicities then overlap the
/// lower ones' radiation and the sum double counts it.
pub fn split_by_multiplicity(sets: Vec<DiagramSet>, card: &RunCard) -> Vec<Vec<DiagramSet>> {
    let mut parts: Vec<(usize, Vec<DiagramSet>)> = Vec::new();
    let mut empty = Vec::new();
    for set in sets {
        if set.diagrams.is_empty() {
            empty.push(set);
            continue;
        }
        let n_out = set.particles_out.len();
        match parts.iter_mut().find(|(n, _)| *n == n_out) {
            Some((_, part)) => part.push(set),
            None => parts.push((n_out, vec![set])),
        }
    }
    if parts.is_empty() {
        return vec![empty];
    }
    parts.sort_by_key(|(n, _)| *n);
    if parts.len() > 1 && card.int("ickkw") == 0 {
        let counts: Vec<String> = parts.iter().map(|(n, _)| n.to_string()).collect();
        tracing::warn!(
            "the processes have {} different final-state multiplicities ({} outgoing legs) and \
             ickkw = 0: each multiplicity is integrated and the samples are summed with no \
             matching, which double counts the radiation the higher multiplicities share with \
             the lower ones. MadGraph runs such a card the same way; set ickkw = 1 and xqcut > \
             0 for an MLM-matched sample",
            parts.len(),
            counts.join(", ")
        );
    }
    parts.into_iter().map(|(_, part)| part).collect()
}

/// Split one multiplicity's subprocesses by their outgoing pole masses: one part
/// per multiset of masses, in the order the enumeration first reaches each, every
/// part keeping the enumeration's own order.
///
/// The key is the multiset rather than the masses in leg order, because two lines
/// listing the same final state in different orders (`p p > w+ j` and
/// `add process p p > j w-`) are one phase space once their legs are paired by
/// content, which [`derive_flavor_groups`] does.
///
/// Masses are compared exactly, with no tolerance, as [`derive_flavor_groups`]
/// compares them: two subprocesses share a part only when every outgoing mass is
/// the same value, which it is when the legs read equal parameters. So no part
/// this returns is one the decomposition refuses for its masses. A name the model
/// does not know keys as NaN, apart from every mass the model has, for the
/// decomposition to refuse by name, and an empty part comes back whole, as one
/// part, for the same reason.
pub(crate) fn split_by_outgoing_masses(
    part: Vec<DiagramSet>,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
) -> Vec<Vec<DiagramSet>> {
    let key = |set: &DiagramSet| {
        mass_multiset(set.particles_out.iter().map(|name| {
            model
                .particle_id(name)
                .map_or(f64::NAN, |id| evaluated.mass(id))
        }))
    };
    if part.is_empty() {
        return vec![part];
    }
    let mut parts: Vec<(Vec<u64>, Vec<DiagramSet>)> = Vec::new();
    for set in part {
        let k = key(&set);
        match parts.iter_mut().find(|(other, _)| *other == k) {
            Some((_, p)) => p.push(set),
            None => parts.push((k, vec![set])),
        }
    }
    parts.into_iter().map(|(_, p)| p).collect()
}

/// A proton-beam card's flavour decomposition, one per phase space: per
/// final-state multiplicity in increasing order ([`split_by_multiplicity`]), and
/// within one, per set of outgoing masses ([`split_by_outgoing_masses`]), each
/// with its groups' on-shell vetoes.
pub struct MultiplicityGroups {
    pub groups: Vec<FlavorGroups>,
    pub vetoes: Vec<Vec<Option<OnShellVeto>>>,
}

/// Enumerate the card and decompose each of its phase spaces — a final-state
/// multiplicity and a set of outgoing masses — into flavour groups.
pub fn multiplicity_groups(
    parsed: &SupportedCard,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    rc: &RunCard,
    enumeration: EnumerationPool,
) -> Result<MultiplicityGroups, String> {
    let sets = generate_from_proc_card_in(parsed, model, enumeration)
        .map_err(|e| format!("failed to enumerate process: {e}"))?;
    let forbidden = forbidden_onshell_ids(parsed, model).map_err(|e| e.to_string())?;
    let markings = subprocess_markings(&sets, &forbidden, model);
    let mut out = MultiplicityGroups {
        groups: Vec::new(),
        vetoes: Vec::new(),
    };
    for multiplicity in split_by_multiplicity(sets, rc) {
        let parts = split_by_outgoing_masses(multiplicity, model, evaluated);
        if parts.len() > 1 {
            tracing::info!(
                "the {}-leg final states have {} different sets of outgoing masses; each is \
                 integrated over channels of its own and the cross sections are summed",
                parts[0][0].particles_out.len(),
                parts.len()
            );
        }
        for part in parts {
            let groups = derive_flavor_groups(part, model, evaluated, rc)
                .map_err(|e| format!("failed to decompose into flavour groups: {e}"))?;
            let vetoes = group_vetoes(&groups, &markings, &forbidden, evaluated, rc)
                .map_err(|e| e.to_string())?;
            out.groups.push(groups);
            out.vetoes.push(vetoes);
        }
    }
    Ok(out)
}

/// The maps every part is built under: `options` as given for a card of one
/// part (one multiplicity and one set of outgoing masses), and for several,
/// `options` settled once over the union of their shapes ([`union_shape`]), since an artifact banks one set of choices.
pub fn multiplicity_maps(
    groups: &[FlavorGroups],
    evaluated: &EvaluatedModel,
    sqrt_s_had: f64,
    options: MapOptions,
) -> MapOptions {
    if groups.len() == 1 {
        return options;
    }
    let shape = union_shape(
        groups
            .iter()
            .map(|g| process_shape(g, evaluated, sqrt_s_had)),
    );
    MapOptions::fixed(options.resolve(&shape))
}

/// Refuse a card of several final-state multiplicities on a path that has one
/// phase space: fixed-energy beams and decays. Only proton beams sum
/// multiplicities ([`MultiplicitySum`]).
pub fn refuse_mixed_multiplicity(sets: &[DiagramSet]) -> Result<(), String> {
    let mut counts: Vec<usize> = sets
        .iter()
        .filter(|s| !s.diagrams.is_empty())
        .map(|s| s.particles_out.len())
        .collect();
    counts.sort_unstable();
    counts.dedup();
    if counts.len() > 1 {
        return Err(format!(
            "the processes have different final-state multiplicities ({} outgoing legs); a sum \
             over multiplicities is integrated at proton beams (lpp = 1) only, and this run has \
             fixed-energy beams or is a decay",
            counts
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    Ok(())
}

/// The shape the map rule settles its choices on for a card of several
/// multiplicities: every part's shape, summed as if one channel set.
///
/// The choices are one per run, and an artifact banks one. The only choice the
/// rule reads off the shape is the `τ` map, which keeps the log map wherever any
/// part carries a resonance spanning its whole final state: a Drell–Yan part
/// keeps the map its peak needs, and the parts above it, which would take
/// `1/τ²` alone, take the log map with it.
pub fn union_shape(shapes: impl IntoIterator<Item = ProcessShape>) -> ProcessShape {
    shapes
        .into_iter()
        .fold(ProcessShape::default(), |acc, s| ProcessShape {
            soft_emission_splits: acc.soft_emission_splits + s.soft_emission_splits,
            max_rungs: acc.max_rungs.max(s.max_rungs),
            whole_state_resonance: acc.whole_state_resonance || s.whole_state_resonance,
        })
}

/// A hadronic cross section summed over phase spaces: one [`ProtonIntegrand`]
/// per final-state multiplicity and set of outgoing masses, their channels
/// exposed as one list.
///
/// Channel `c` is channel `c − offset(k)` of part `k`, where the parts are in
/// non-decreasing multiplicity and `offset(k)` counts the channels of the parts
/// before it. Each part keeps its own channel selection weights, normalised over
/// its own channels — a term's `αⱼ` is its own mixture's, so the terms of each
/// part sum to that part's cross section and all of them to the total.
///
/// With one part this is that part: every channel, value and allocation is the
/// part's own, bit for bit.
pub struct MultiplicitySum<'a> {
    parts: Vec<ProtonIntegrand<'a>>,
    /// `offsets[k]` is part `k`'s first channel; the last entry is the total.
    offsets: Vec<usize>,
    /// Each part's share of the per-iteration budget, summing to one.
    budget_shares: Vec<f64>,
    /// How a per-channel adaptation refines the channels' grids.
    grid_rule: GridRule,
}

impl<'a> MultiplicitySum<'a> {
    /// Compose the parts, which must be in non-decreasing final-state
    /// multiplicity, one per phase space.
    ///
    /// # Panics
    ///
    /// If there is no part, if the parts are out of multiplicity order, if two
    /// parts share both a multiplicity and a set of outgoing masses, or if the
    /// parts were built under different maps — the artifact banks one set of
    /// choices, and a replay rebuilds every part under it.
    pub fn new(parts: Vec<ProtonIntegrand<'a>>) -> Self {
        assert!(!parts.is_empty(), "a multiplicity sum needs a part");
        for (i, a) in parts.iter().enumerate() {
            for b in &parts[i + 1..] {
                assert!(
                    sorted_final_masses(a) != sorted_final_masses(b),
                    "the parts are one per phase space"
                );
            }
        }
        for pair in parts.windows(2) {
            assert!(
                final_state_count(&pair[0]) <= final_state_count(&pair[1]),
                "the parts are in non-decreasing multiplicity"
            );
            assert_eq!(
                pair[0].maps(),
                pair[1].maps(),
                "every part samples under the same maps"
            );
        }
        let mut offsets = Vec::with_capacity(parts.len() + 1);
        let mut total = 0;
        for part in &parts {
            offsets.push(total);
            total += part.channel_count();
        }
        offsets.push(total);
        let n = parts.len();
        MultiplicitySum {
            parts,
            offsets,
            budget_shares: vec![1.0 / n as f64; n],
            grid_rule: GridRule::default(),
        }
    }

    /// Refine the channels' grids under `rule` in
    /// [`adapt_grids_budget`](Self::adapt_grids_budget).
    pub fn set_grid_rule(&mut self, rule: GridRule) {
        self.grid_rule = rule;
    }

    /// The parts, in non-decreasing multiplicity.
    pub fn parts(&self) -> &[ProtonIntegrand<'a>] {
        &self.parts
    }

    /// The number of outgoing legs of part `k`.
    pub fn final_state_count(&self, k: usize) -> usize {
        final_state_count(&self.parts[k])
    }

    /// The first channel of every part, and the total channel count last.
    pub fn offsets(&self) -> &[usize] {
        &self.offsets
    }

    /// The part a channel belongs to and its index among that part's channels.
    ///
    /// # Panics
    ///
    /// If `channel` is not a channel index.
    pub(crate) fn locate(&self, channel: usize) -> (usize, usize) {
        assert!(
            channel < self.channel_count(),
            "channel {channel} of {}",
            self.channel_count()
        );
        let k = self.offsets.partition_point(|&o| o <= channel) - 1;
        (k, channel - self.offsets[k])
    }

    /// Every part's channels, in order.
    pub fn channel_count(&self) -> usize {
        self.offsets[self.parts.len()]
    }

    /// The coordinates channel `channel`'s grid is built over: its part's.
    pub fn channel_grid_ndim(&self, channel: usize) -> usize {
        let (k, _) = self.locate(channel);
        self.parts[k].channel_grid_ndim()
    }

    /// The trailing uniforms a point carries past its grid coordinates: the
    /// most any part consumes. A part that consumes fewer reads the leading ones.
    pub(crate) fn scale_draw_ndim(&self) -> usize {
        self.parts
            .iter()
            .map(ProtonIntegrand::scale_draw_ndim)
            .max()
            .unwrap_or(0)
    }

    /// The coordinates part `k` reads of a point drawn in one of its channels.
    fn part_point<'u>(&self, k: usize, u: &'u [f64]) -> &'u [f64] {
        &u[..self.parts[k].point_ndim()]
    }

    /// The `channel`-th term at `u ∈ [0,1]^(channel_grid_ndim(channel) +
    /// scale_draw_ndim)`: that of the part it belongs to.
    pub(crate) fn value_in_channel(&self, channel: usize, u: &[f64]) -> f64 {
        let (k, j) = self.locate(channel);
        self.parts[k].value_in_channel(j, self.part_point(k, u))
    }

    /// [`value_in_channel`](Self::value_in_channel) with the point kept, and the
    /// part it belongs to: [`ProtonIntegrand::event_in_channel`] of that part.
    pub fn event_in_channel(&self, channel: usize, u: &[f64]) -> Option<(usize, ProtonEvent)> {
        let (k, j) = self.locate(channel);
        self.parts[k]
            .event_in_channel(j, self.part_point(k, u))
            .map(|event| (k, event))
    }

    /// [`ProtonIntegrand::point_anatomy`] at a channel of the sum, with the part
    /// it belongs to; the anatomy's channel indices are that part's own.
    pub fn point_anatomy(&self, channel: usize, u: &[f64]) -> Option<(usize, PointAnatomy)> {
        let (k, j) = self.locate(channel);
        self.parts[k]
            .point_anatomy(j, self.part_point(k, u))
            .map(|anatomy| (k, anatomy))
    }

    /// The key each channel's grid is banked under, in channel order. A single
    /// part keeps its own [`ChannelKey::GroupChannel`] keys, so its artifact is
    /// the one a single-multiplicity card has always written. A channel whose map
    /// serves several `(group, diagram)` pairs is a [`ChannelKey::MergedChannel`]
    /// whatever the part count.
    ///
    /// A key's group counts every group of its multiplicity, across the parts
    /// that multiplicity is split into by outgoing masses, in part order, so two
    /// parts of one multiplicity never bank a channel under the same key.
    pub fn channel_keys(&self) -> Vec<ChannelKey> {
        let single = self.parts.len() == 1;
        let group_offsets: Vec<usize> = self
            .parts
            .iter()
            .enumerate()
            .map(|(k, part)| {
                let final_state = final_state_count(part);
                self.parts[..k]
                    .iter()
                    .filter(|p| final_state_count(p) == final_state)
                    .map(|p| p.groups().groups().len())
                    .sum()
            })
            .collect();
        self.parts
            .iter()
            .zip(group_offsets)
            .flat_map(|(part, group_offset)| {
                let final_state = final_state_count(part);
                part.channel_ids()
                    .iter()
                    .zip(part.channel_members())
                    .map(move |(id, members)| {
                        let group = group_offset + id.group;
                        if members.len() > 1 {
                            ChannelKey::MergedChannel {
                                final_state,
                                group,
                                channel: id.channel,
                                pairs: members.len(),
                            }
                        } else if single {
                            ChannelKey::GroupChannel {
                                group,
                                channel: id.channel,
                            }
                        } else {
                            ChannelKey::MultiplicityChannel {
                                final_state,
                                group,
                                channel: id.channel,
                            }
                        }
                    })
            })
            .collect()
    }

    /// What the composition chose for each channel, in channel order.
    pub fn channel_samplers(&self) -> Vec<ChannelSampler> {
        self.parts
            .iter()
            .flat_map(|p| p.channel_samplers().iter().cloned())
            .collect()
    }

    /// Every channel's [`map_key`](crate::phasespace::DiagramChannel::map_key), in
    /// channel order.
    pub fn channel_map_keys(&self) -> Vec<String> {
        self.parts
            .iter()
            .flat_map(|p| p.channel_map_keys().iter().cloned())
            .collect()
    }

    /// Every channel's selection weight in its own part's mixture, in channel
    /// order — the `αⱼ` each term carries. They sum to one per part.
    pub(crate) fn channel_alphas(&self) -> Vec<f64> {
        self.parts.iter().flat_map(|p| p.channel_alphas()).collect()
    }

    /// Install every part's selection weights from one list in channel order, as
    /// [`channel_alphas`](Self::channel_alphas) returns it.
    ///
    /// # Panics
    ///
    /// If `alphas` is not one weight per channel, or a part's slice is not a
    /// normalised set of positive weights.
    pub fn set_channel_alphas(&mut self, alphas: Vec<f64>) {
        assert_eq!(alphas.len(), self.channel_count(), "one weight per channel");
        for (k, part) in self.parts.iter_mut().enumerate() {
            part.set_channel_alphas(alphas[self.offsets[k]..self.offsets[k + 1]].to_vec());
        }
    }

    /// The maps every part samples under.
    pub fn maps(&self) -> MapChoices {
        self.parts[0].maps()
    }

    /// Each part's share of the per-iteration budget, summing to one.
    #[cfg(test)]
    pub(crate) fn budget_shares(&self) -> &[f64] {
        &self.budget_shares
    }

    /// Adapt every part's selection weights on its own hadronic mixture
    /// ([`ProtonIntegrand::adapt_alphas`]), and split the budget across the parts
    /// by what the final surveys measured.
    ///
    /// For independent terms estimated from `nₖ` points each, the summed variance
    /// `Σ sₖ²/nₖ` is least at `nₖ ∝ sₖ`, so a part's share is `∝ sₖ`, the standard
    /// deviation of its mixture's per-point estimator `f/g`: `√(E[(f/g)²] − σₖ²)`,
    /// with the second moment from the survey's variance shares (`Σⱼ αⱼ Wⱼ` under
    /// the weights the survey drew with) and `σₖ` the survey's own mean. It is
    /// the spread of the undivided mixture, which the per-channel grids then
    /// reduce, so it ranks the parts rather than predicting their errors. A
    /// Neyman allocation re-splits by every channel's measured spread from the
    /// second iteration on, so there it sets the first iteration only.
    pub fn adapt_alphas(
        &mut self,
        seed: u64,
        n_survey: usize,
        n_iter: usize,
        damping: f64,
    ) -> Vec<AlphaAdaptation<f64>> {
        let adaptations: Vec<AlphaAdaptation<f64>> = self
            .parts
            .iter_mut()
            .map(|p| p.adapt_alphas(seed, n_survey, n_iter, damping))
            .collect();
        if self.parts.len() > 1 {
            let sd: Vec<f64> = adaptations
                .iter()
                .zip(&self.parts)
                .map(|(a, p)| survey_sd(a, p.survey_mean()))
                .collect();
            self.budget_shares = normalised_or_uniform(&sd);
            for (k, share) in self.budget_shares.iter().enumerate() {
                tracing::info!(
                    "{} outgoing legs (masses {:?} GeV): {} channels, survey mean {:.3e}, \
                     estimator spread {:.3e}, {:.1}% of the budget",
                    self.final_state_count(k),
                    self.parts[k].groups().groups()[0].final_masses(),
                    self.parts[k].channel_count(),
                    self.parts[k].survey_mean(),
                    sd[k],
                    100.0 * share
                );
            }
        }
        adaptations
    }

    /// The per-channel shares the budget is split by: each channel's `αⱼ` times
    /// its part's budget share. With one part, the part's own weights.
    pub(crate) fn allocation_alphas(&self) -> Vec<f64> {
        if self.parts.len() == 1 {
            return self.parts[0].channel_alphas();
        }
        self.parts
            .iter()
            .zip(&self.budget_shares)
            .flat_map(|(p, &w)| p.channel_alphas().into_iter().map(move |a| a * w))
            .collect()
    }

    /// Integrate every channel of every part under `budget`, one grid per
    /// channel at its part's dimension, allocated by
    /// [`allocation_alphas`](Self::allocation_alphas) (or re-split by the channels'
    /// measured spread under a Neyman allocation).
    ///
    /// Each returned [`ChannelIntegration::alpha`] is the channel's weight in its
    /// own part's mixture — the one its term carries, which a replay installs —
    /// not the allocation share it was integrated under.
    pub fn adapt_grids_budget(
        &self,
        budget: Budget,
        allocation: BlockAllocation,
        seed: u64,
        stop: &StopSignal,
    ) -> (Vec<ChannelIntegration>, VegasResult, ConvergenceReport) {
        let vegas_alpha = self.parts[0].vegas_alpha();
        assert!(
            self.parts.iter().all(|p| p.vegas_alpha() == vegas_alpha),
            "every part's grids are damped alike"
        );
        let (mut per_channel, total, report) = integrate_channels(
            self,
            &self.allocation_alphas(),
            vegas_alpha,
            IterationCombination::default(),
            budget,
            allocation,
            seed,
            stop,
        );
        if self.parts.len() > 1 {
            for (c, alpha) in per_channel.iter_mut().zip(self.channel_alphas()) {
                c.alpha = alpha;
            }
        }
        (per_channel, total, report)
    }

    /// Each part's cross section from a completed integration's channel terms:
    /// its terms summed, their errors in quadrature.
    pub fn part_results(
        &self,
        per_channel: &[ChannelIntegration],
        niter: usize,
    ) -> Vec<VegasResult> {
        assert_eq!(
            per_channel.len(),
            self.channel_count(),
            "one term per channel"
        );
        (0..self.parts.len())
            .map(|k| combine_channels(&per_channel[self.offsets[k]..self.offsets[k + 1]], niter))
            .collect()
    }
}

impl ChannelIntegrand for MultiplicitySum<'_> {
    fn channel_count(&self) -> usize {
        MultiplicitySum::channel_count(self)
    }

    fn channel_grid_ndim(&self, channel: usize) -> usize {
        MultiplicitySum::channel_grid_ndim(self, channel)
    }

    fn scale_draw_ndim(&self) -> usize {
        MultiplicitySum::scale_draw_ndim(self)
    }

    fn value_in_channel(&self, channel: usize, u: &[f64]) -> f64 {
        MultiplicitySum::value_in_channel(self, channel, u)
    }

    fn grid_rule(&self) -> GridRule {
        self.grid_rule
    }
}

fn final_state_count(part: &ProtonIntegrand<'_>) -> usize {
    part.groups().groups()[0].final_masses().len()
}

/// A part's outgoing pole masses as a multiset, the key it was split off by.
fn sorted_final_masses(part: &ProtonIntegrand<'_>) -> Vec<u64> {
    mass_multiset(part.groups().groups()[0].final_masses())
}

/// Outgoing masses as a multiset, compared exactly: each mass's bit pattern,
/// sorted ([`split_by_outgoing_masses`]).
fn mass_multiset(masses: impl IntoIterator<Item = f64>) -> Vec<u64> {
    let mut bits: Vec<u64> = masses.into_iter().map(f64::to_bits).collect();
    bits.sort_unstable();
    bits
}

/// The standard deviation of the mixture estimator `f/g` over an adaptation's
/// final survey, `√(Σⱼ αⱼ Wⱼ − mean²)`, where `Σⱼ αⱼ Wⱼ = E[(f/g)²]` under the
/// weights that survey drew with.
///
/// A survey that carried variance was followed by a step, so it was drawn under
/// the next-to-last entry of the trajectory. One that carried none has every
/// `Wⱼ = 0` and a zero mean, which gives zero whichever weights it was drawn
/// under.
fn survey_sd(adaptation: &AlphaAdaptation<f64>, mean: f64) -> f64 {
    let t = &adaptation.trajectory;
    let surveyed = &t[t.len().saturating_sub(2)];
    let second: f64 = surveyed
        .iter()
        .zip(&adaptation.variance_shares)
        .map(|(a, w)| a * w)
        .sum();
    (second - mean * mean).max(0.0).sqrt()
}

/// `x` normalised to sum to one, or equal shares where it carries nothing
/// usable.
fn normalised_or_uniform(x: &[f64]) -> Vec<f64> {
    let sum: f64 = x.iter().sum();
    if sum > 0.0 && sum.is_finite() && x.iter().all(|&v| v > 0.0) {
        x.iter().map(|&v| v / sum).collect()
    } else {
        vec![1.0 / x.len() as f64; x.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budget::StopSignal;
    use crate::diagrams::{
        generate_from_proc_card, parse_proc_card, EnumerationPool, ParsingOptions,
    };
    use crate::helas::eval::BoundAmplitude;
    use crate::pdf::grid::SubGrid;
    use crate::pdf::PdfMember;
    use crate::phasespace::maps::MapOptions;
    use crate::phasespace::rng::SubStream;
    use crate::proton::{derive_flavor_groups, FlavorGroups, ProtonBeams, ProtonError};
    use crate::ufo::sm::{sm_model, SMRestrict};
    use crate::ufo::{EvaluatedModel, UFOModel};
    use crate::unweight::{MaxRule, Unweighter};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    const SQRT_S_HAD: f64 = 13000.0;
    const MU_F: f64 = 91.188;

    /// Drell–Yan with no jet and with one: two multiplicities whose channels are
    /// over `2 + 2` and `2 + 5` coordinates.
    const TWO_MULTIPLICITIES: &str = "generate p p > e+ e- j @1\nadd process p p > e+ e- @0\n";

    fn card() -> RunCard {
        RunCard::parse(
            "  1 = lpp1\n  1 = lpp2\n  6500.0 = ebeam1\n  6500.0 = ebeam2\n\
             \x20 True = fixed_ren_scale\n  True = fixed_fac_scale1\n  True = fixed_fac_scale2\n\
             \x20 91.188 = scale\n  91.188 = dsqrt_q2fact1\n  91.188 = dsqrt_q2fact2\n\
             \x20 20.0 = ptj\n  10.0 = ptl\n  5.0 = etaj\n  2.5 = etal\n\
             \x20 0.4 = drll\n  0.4 = drjl\n  50.0 = mmll\n  4 = maxjetflavor\n",
        )
        .expect("run card")
    }

    /// A synthetic parton distribution with a different `x·f` per flavour, flat
    /// enough that every channel carries weight.
    fn pdf() -> PdfMember {
        let flavors = vec![-4, -3, -2, -1, 1, 2, 3, 4, 21];
        let x = vec![1e-7, 1.0];
        let q2 = vec![1.0, 1e8];
        let mut xf = Vec::new();
        for ix in 0..x.len() {
            for iq in 0..q2.len() {
                for ifl in 0..flavors.len() {
                    let shape = 1.0 + ix as f64 * 0.1 * (ifl + 1) as f64;
                    xf.push(0.01 * (ifl + 1) as f64 * shape * (1.0 + 0.5 * iq as f64));
                }
            }
        }
        PdfMember::from_subgrids(vec![SubGrid { x, q2, flavors, xf }])
    }

    fn enumerate(card: &str, model: &UFOModel) -> Vec<DiagramSet> {
        let parsed = parse_proc_card(card, &ParsingOptions::default()).expect("proc card");
        generate_from_proc_card(&parsed, model).expect("enumeration")
    }

    fn labels(sets: &[DiagramSet]) -> Vec<String> {
        sets.iter().map(DiagramSet::label).collect()
    }

    fn part_groups(
        card_text: &str,
        model: &UFOModel,
        evaluated: &EvaluatedModel,
    ) -> Vec<FlavorGroups> {
        let card = card();
        split_by_multiplicity(enumerate(card_text, model), &card)
            .into_iter()
            .map(|part| derive_flavor_groups(part, model, evaluated, &card).expect("groups"))
            .collect()
    }

    fn bind_all<'a>(
        groups: &'a FlavorGroups,
        evaluated: &'a EvaluatedModel,
    ) -> Vec<BoundAmplitude<'a, f64>> {
        groups
            .groups()
            .iter()
            .map(|g| BoundAmplitude::<f64>::bind(g.evaluator(), evaluated))
            .collect()
    }

    fn part<'a>(
        groups: &'a FlavorGroups,
        amps: &'a [BoundAmplitude<'a, f64>],
        evaluated: &EvaluatedModel,
        pdf: &'a PdfMember,
    ) -> ProtonIntegrand<'a> {
        ProtonIntegrand::new_with_maps(
            groups,
            amps,
            evaluated,
            pdf,
            ProtonBeams::equal(SQRT_S_HAD),
            MU_F,
            MapOptions::fixed(MapChoices::LEGACY),
        )
        .expect("integrand")
    }

    /// The parts come in increasing multiplicity whatever order the card lists
    /// them in, each keeping the enumeration's own order, and a card of one
    /// multiplicity comes back as it went in.
    #[test]
    fn the_enumeration_splits_by_multiplicity_in_increasing_order() {
        let model = sm_model(SMRestrict::Default);
        let sets = enumerate(TWO_MULTIPLICITIES, &model);
        let all: Vec<String> = sets
            .iter()
            .filter(|s| !s.diagrams.is_empty())
            .map(DiagramSet::label)
            .collect();
        let parts = split_by_multiplicity(sets, &card());
        assert_eq!(parts.len(), 2);
        assert!(parts[0].iter().all(|s| s.particles_out.len() == 2));
        assert!(parts[1].iter().all(|s| s.particles_out.len() == 3));
        for part in &parts {
            let order: Vec<usize> = labels(part)
                .iter()
                .map(|l| all.iter().position(|a| a == l).expect("a set of the card"))
                .collect();
            assert!(order.windows(2).all(|w| w[0] < w[1]), "{order:?}");
        }
        assert_eq!(parts.iter().map(Vec::len).sum::<usize>(), all.len());

        let single = enumerate("generate p p > e+ e- j\n", &model);
        let before: Vec<String> = single
            .iter()
            .filter(|s| !s.diagrams.is_empty())
            .map(DiagramSet::label)
            .collect();
        let parts = split_by_multiplicity(single, &card());
        assert_eq!(parts.len(), 1);
        assert_eq!(labels(&parts[0]), before);
    }

    /// Channel `c` of the sum is channel `c − offset(k)` of part `k`: its grid
    /// dimension, its value at every point bit for bit, its key and its weight.
    #[test]
    fn the_sum_s_channels_are_its_parts_channels_offset() {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let pdf = pdf();
        let groups = part_groups(TWO_MULTIPLICITIES, &model, &evaluated);
        let amps: Vec<_> = groups.iter().map(|g| bind_all(g, &evaluated)).collect();
        let sum = MultiplicitySum::new(
            groups
                .iter()
                .zip(&amps)
                .map(|(g, a)| part(g, a, &evaluated, &pdf))
                .collect(),
        );
        let n0 = sum.parts()[0].channel_count();
        let n1 = sum.parts()[1].channel_count();
        assert_eq!(sum.offsets(), [0, n0, n0 + n1]);
        assert_eq!(sum.channel_count(), n0 + n1);
        assert_eq!((sum.final_state_count(0), sum.final_state_count(1)), (2, 3));
        assert_eq!(sum.locate(0), (0, 0));
        assert_eq!(sum.locate(n0 - 1), (0, n0 - 1));
        assert_eq!(sum.locate(n0), (1, 0));
        assert_eq!(sum.locate(n0 + n1 - 1), (1, n1 - 1));

        let keys = sum.channel_keys();
        let samplers = sum.channel_samplers();
        assert_eq!((keys.len(), samplers.len()), (n0 + n1, n0 + n1));
        let mut stream = SubStream::from_stream(0x3_7A1, 1);
        for c in 0..sum.channel_count() {
            let (k, j) = sum.locate(c);
            let ndim = sum.channel_grid_ndim(c);
            assert_eq!(ndim, if k == 0 { 2 + 2 } else { 2 + 5 });
            assert_eq!(ChannelIntegrand::channel_grid_ndim(&sum, c), ndim);
            let id = sum.parts()[k].channel_ids()[j];
            let pairs = sum.parts()[k].channel_members()[j].len();
            let final_state = sum.final_state_count(k);
            assert_eq!(
                keys[c],
                if pairs > 1 {
                    ChannelKey::MergedChannel {
                        final_state,
                        group: id.group,
                        channel: id.channel,
                        pairs,
                    }
                } else {
                    ChannelKey::MultiplicityChannel {
                        final_state,
                        group: id.group,
                        channel: id.channel,
                    }
                }
            );
            assert_eq!(samplers[c], sum.parts()[k].channel_samplers()[j]);
            for _ in 0..16 {
                let u = stream.uniforms::<f64>(ndim + sum.scale_draw_ndim());
                assert_eq!(
                    sum.value_in_channel(c, &u).to_bits(),
                    sum.parts()[k].value_in_channel(j, &u).to_bits()
                );
            }
        }

        // Every part's weights are its own mixture's, normalised over its own
        // channels; the budget split scales them into one allocation.
        let alphas = sum.channel_alphas();
        let part_sum =
            |k: usize| -> f64 { alphas[sum.offsets()[k]..sum.offsets()[k + 1]].iter().sum() };
        assert!((part_sum(0) - 1.0).abs() < 1e-12 && (part_sum(1) - 1.0).abs() < 1e-12);
        let allocation: f64 = sum.allocation_alphas().iter().sum();
        assert!((allocation - 1.0).abs() < 1e-12, "{allocation}");
    }

    /// A sum of one part is that part: the same keys, the same allocation, and an
    /// integration that is the part's own bit for bit — grids and terms.
    #[test]
    fn a_sum_of_one_multiplicity_is_that_multiplicity() {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let pdf = pdf();
        let groups = part_groups("generate p p > e+ e- j\n", &model, &evaluated);
        assert_eq!(groups.len(), 1);
        let amps = bind_all(&groups[0], &evaluated);
        let alone = part(&groups[0], &amps, &evaluated, &pdf);
        let sum = MultiplicitySum::new(vec![part(&groups[0], &amps, &evaluated, &pdf)]);

        assert!(sum
            .channel_keys()
            .iter()
            .zip(alone.channel_ids().iter().zip(alone.channel_members()))
            .all(|(key, (id, members))| *key
                == if members.len() > 1 {
                    ChannelKey::MergedChannel {
                        final_state: 3,
                        group: id.group,
                        channel: id.channel,
                        pairs: members.len(),
                    }
                } else {
                    ChannelKey::GroupChannel {
                        group: id.group,
                        channel: id.channel,
                    }
                }));
        assert_eq!(sum.allocation_alphas(), alone.channel_alphas());

        let budget = Budget::Fixed {
            neval: 4_000,
            niter: 3,
        };
        let stop = StopSignal::default();
        let (ours, total, _) = sum.adapt_grids_budget(budget, BlockAllocation::ByAlpha, 7, &stop);
        let (theirs, alone_total, _) =
            alone.adapt_grids_budget(budget, BlockAllocation::ByAlpha, 7, &stop);
        assert_eq!(total.integral.to_bits(), alone_total.integral.to_bits());
        assert_eq!(total.std_dev.to_bits(), alone_total.std_dev.to_bits());
        for (a, b) in ours.iter().zip(&theirs) {
            assert_eq!(a.alpha.to_bits(), b.alpha.to_bits());
            assert_eq!(a.neval, b.neval);
            assert_eq!(a.result.integral.to_bits(), b.result.integral.to_bits());
            assert_eq!(
                bincode_bytes(&a.grid),
                bincode_bytes(&b.grid),
                "a trained grid differs"
            );
        }
    }

    /// Drell–Yan over three lepton flavours: `ta` is massive in the default
    /// Standard Model (1.777 GeV), `e` and `mu` are not, so one multiplicity
    /// holds two phase spaces.
    const MIXED_MASSES: &str =
        "define l+ = e+ mu+ ta+\ndefine l- = e- mu- ta-\ngenerate p p > l+ l-\n";

    fn mixed_mass_groups(model: &UFOModel, evaluated: &EvaluatedModel) -> MultiplicityGroups {
        let parsed = parse_proc_card(MIXED_MASSES, &ParsingOptions::default()).expect("proc card");
        multiplicity_groups(&parsed, model, evaluated, &card(), EnumerationPool::Serial)
            .expect("a card of mixed outgoing masses decomposes")
    }

    /// A multiplicity whose subprocesses differ in outgoing masses comes back as
    /// one decomposition per set of masses: every group of a part shares its
    /// masses, the parts hold every subprocess once between them, and each part
    /// holds exactly the flavours its masses admit.
    ///
    /// Handed the same subprocesses unsplit, the decomposition still refuses them:
    /// one decomposition is one phase space, since every group of it is evaluated
    /// at every point its channels draw.
    #[test]
    fn mixed_outgoing_masses_split_into_one_decomposition_per_mass_set() {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let mta = evaluated.mass(model.particle_id("ta-").expect("ta-"));
        assert!(
            mta > 1.0,
            "the default Standard Model's tau is massive: {mta}"
        );

        let MultiplicityGroups { groups, vetoes } = mixed_mass_groups(&model, &evaluated);
        assert_eq!(groups.len(), 2, "one part per set of outgoing masses");
        assert_eq!(vetoes.len(), 2);
        let counts: Vec<usize> = groups.iter().map(|g| g.groups().len()).collect();
        eprintln!("groups per part: {counts:?}");
        assert_eq!(
            counts,
            [2, 2],
            "up-type and down-type initial states per part"
        );
        for (part, want) in groups.iter().zip([[0.0, 0.0], [mta, mta]]) {
            for g in part.groups() {
                assert_eq!(g.final_masses(), want);
                for m in g.members() {
                    let tau = m.outgoing.iter().all(|p| p.abs() == 15);
                    let light = m.outgoing.iter().all(|p| matches!(p.abs(), 11 | 13));
                    assert!(
                        if want[0] == 0.0 { light } else { tau },
                        "{:?} in the part of masses {want:?}",
                        m.outgoing
                    );
                }
            }
        }
        let members: usize = groups
            .iter()
            .flat_map(FlavorGroups::groups)
            .map(|g| g.members().len())
            .sum();
        let enumerated = enumerate(MIXED_MASSES, &model)
            .iter()
            .filter(|s| !s.diagrams.is_empty())
            .count();
        assert_eq!(members, enumerated);

        let unsplit = split_by_multiplicity(enumerate(MIXED_MASSES, &model), &card());
        assert_eq!(unsplit.len(), 1);
        let refused = derive_flavor_groups(
            unsplit.into_iter().next().expect("one multiplicity"),
            &model,
            &evaluated,
            &card(),
        );
        assert!(
            matches!(refused, Err(ProtonError::UnequalFinalMasses { .. })),
            "an unsplit mixed-mass multiplicity must be refused"
        );
    }

    /// Two parts of one phase space (one multiplicity, one set of outgoing masses)
    /// are refused: the sum would integrate that phase space twice.
    #[test]
    #[should_panic(expected = "the parts are one per phase space")]
    fn a_sum_refuses_two_parts_of_one_phase_space() {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let pdf = pdf();
        let groups = part_groups("generate p p > e+ e-\n", &model, &evaluated);
        let amps = bind_all(&groups[0], &evaluated);
        MultiplicitySum::new(vec![
            part(&groups[0], &amps, &evaluated, &pdf),
            part(&groups[0], &amps, &evaluated, &pdf),
        ]);
    }

    /// The two phase spaces integrate as two parts of one sum, banking every
    /// channel under a key of its own, and the massive part carries the cross
    /// section one lepton flavour of the massless part does.
    ///
    /// `ta+ ta-` against `e+ e-` plus `mu+ mu-` at `m_ll > 50 GeV` is a ratio of
    /// one half up to the tau mass, under the lepton cuts all three share. A part
    /// dropped or counted twice moves the ratio to zero, a quarter or one, far
    /// outside its bound; two parts banking a channel under one key fail the key
    /// check.
    ///
    /// Blind to the tau mass itself: the ratio's error at this budget is about
    /// 0.004 and its bound `5σ + 0.002` about ±0.024, while the tau mass moves σ by
    /// about 1% (−1.15% on the dy13 default card), so an error of that size, such
    /// as the tau part read on the massless channels, passes here. The banked
    /// mixed-mass gate's known-wrong control is what sees it.
    #[test]
    fn mixed_mass_parts_integrate_to_one_flavour_each() {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let pdf = pdf();
        let MultiplicityGroups { groups, .. } = mixed_mass_groups(&model, &evaluated);
        let amps: Vec<_> = groups.iter().map(|g| bind_all(g, &evaluated)).collect();
        let sum = MultiplicitySum::new(
            groups
                .iter()
                .zip(&amps)
                .map(|(g, a)| part(g, a, &evaluated, &pdf))
                .collect(),
        );
        assert_eq!((sum.final_state_count(0), sum.final_state_count(1)), (2, 2));

        let keys = sum.channel_keys();
        let distinct: std::collections::HashSet<_> =
            keys.iter().map(|k| format!("{k:?}")).collect();
        assert_eq!(
            distinct.len(),
            keys.len(),
            "two channels share a key: {keys:?}"
        );

        let budget = Budget::Fixed {
            neval: 30_000,
            niter: 5,
        };
        let stop = StopSignal::default();
        let (per_channel, total, report) =
            sum.adapt_grids_budget(budget, BlockAllocation::ByAlpha, 5, &stop);
        let parts = sum.part_results(&per_channel, report.iterations);
        let summed: f64 = parts.iter().map(|p| p.integral).sum();
        assert!((summed / total.integral - 1.0).abs() < 1e-12);
        let (light, tau) = (&parts[0], &parts[1]);
        let ratio = tau.integral / light.integral;
        let err = ratio
            * ((tau.std_dev / tau.integral).powi(2) + (light.std_dev / light.integral).powi(2))
                .sqrt();
        eprintln!(
            "e/mu part {:.6e} ± {:.1e}, tau part {:.6e} ± {:.1e}, ratio {ratio:.5} ± {err:.5}",
            light.integral, light.std_dev, tau.integral, tau.std_dev
        );
        assert!(
            (ratio - 0.5).abs() < 5.0 * err + 0.002,
            "ratio {ratio} ± {err}"
        );
    }

    fn bincode_bytes(grid: &crate::vegas::VegasGrid) -> Vec<u8> {
        bincode::serialize(grid).expect("a grid serialises")
    }

    /// The sum's cross section is the sum of its parts' — each integrated alone,
    /// on streams of its own — and an unweighted sample drawn across every channel
    /// splits between the multiplicities as their cross sections do.
    ///
    /// Blind to an error both sides share (a part's own integrand), which the
    /// single-multiplicity gates cover; what it sees is the composition: a channel
    /// offset, a dimension or a weight crossed between parts moves one of the two
    /// comparisons by many standard deviations.
    #[test]
    fn the_sum_s_cross_section_and_sample_split_are_its_parts() {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let pdf = pdf();
        let groups = part_groups(TWO_MULTIPLICITIES, &model, &evaluated);
        let amps: Vec<_> = groups.iter().map(|g| bind_all(g, &evaluated)).collect();
        let mut sum = MultiplicitySum::new(
            groups
                .iter()
                .zip(&amps)
                .map(|(g, a)| part(g, a, &evaluated, &pdf))
                .collect(),
        );
        let adaptations = sum.adapt_alphas(11, 4_000, 2, 0.5);
        assert_eq!(adaptations.len(), 2);
        let shares = sum.budget_shares().to_vec();
        assert!(
            shares.iter().all(|&s| s > 0.0) && (shares.iter().sum::<f64>() - 1.0).abs() < 1e-12
        );

        let budget = Budget::Fixed {
            neval: 30_000,
            niter: 5,
        };
        let stop = StopSignal::default();
        let (per_channel, total, report) =
            sum.adapt_grids_budget(budget, BlockAllocation::ByAlpha, 5, &stop);
        for (c, term) in per_channel.iter().enumerate() {
            assert_eq!(term.grid.ndim(), sum.channel_grid_ndim(c));
        }
        // The banked weights are the terms' own, not the allocation's.
        assert_eq!(
            per_channel.iter().map(|c| c.alpha).collect::<Vec<_>>(),
            sum.channel_alphas()
        );
        let parts = sum.part_results(&per_channel, report.iterations);
        let summed: f64 = parts.iter().map(|p| p.integral).sum();
        assert!((summed / total.integral - 1.0).abs() < 1e-12);

        for (k, (p, from_sum)) in sum.parts().iter().zip(&parts).enumerate() {
            let (_, alone, _) =
                p.adapt_grids_budget(budget, BlockAllocation::ByAlpha, 99 + k as u64, &stop);
            let pull = (from_sum.integral - alone.integral)
                / (from_sum.std_dev.powi(2) + alone.std_dev.powi(2)).sqrt();
            eprintln!(
                "part {k}: {:.6e} ± {:.1e} in the sum, {:.6e} ± {:.1e} alone (pull {pull:+.2})",
                from_sum.integral, from_sum.std_dev, alone.integral, alone.std_dev
            );
            assert!(pull.abs() < 5.0, "part {k}: pull {pull}");
        }

        let grids: Vec<_> = per_channel.iter().map(|c| c.grid.clone()).collect();
        let mut unweighter =
            Unweighter::scan_with(&sum, grids.iter().map(|g| (g, 1_500)), 3, MaxRule::Extremum);
        let mut rng = ChaCha8Rng::seed_from_u64(17);
        let n = 4_000;
        let mut weight = [0.0f64; 2];
        for _ in 0..n {
            let point = unweighter
                .next_event(&sum, &mut rng, 1_000_000)
                .expect("an event");
            let (k, _) = sum.locate(point.channel);
            assert_eq!(
                point.u.len(),
                sum.channel_grid_ndim(point.channel) + sum.scale_draw_ndim()
            );
            weight[k] += point.weight;
        }
        let f = weight[0] / (weight[0] + weight[1]);
        let expected = parts[0].integral / total.integral;
        let sd = (expected * (1.0 - expected) / n as f64).sqrt();
        eprintln!("multiplicity 2 carries {f:.4} of the sample, {expected:.4} of σ (sd {sd:.4})");
        assert!(
            (f - expected).abs() < 5.0 * sd,
            "sample share {f} against {expected} ± {sd}"
        );
    }
}
