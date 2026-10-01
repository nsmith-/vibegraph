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

use crate::artifact::{ChannelKey, ChannelSampler};
use crate::budget::{integrate_channels, BlockAllocation, Budget, ConvergenceReport, StopSignal};
use crate::diagrams::DiagramSet;
use crate::hadronic::{combine_channels, ChannelIntegration};
use crate::phasespace::channel::AlphaAdaptation;
use crate::phasespace::maps::{MapChoices, ProcessShape};
use crate::proton::{ProtonEvent, ProtonIntegrand};
use crate::runcard::RunCard;
use crate::unweight::ChannelIntegrand;
use crate::vegas::{IterationCombination, VegasResult};

/// Split a proc card's enumeration by final-state multiplicity, in increasing
/// order of outgoing legs, each part keeping the enumeration's own order.
///
/// Empty sets are dropped, as [`derive_flavor_groups`](crate::proton::derive_flavor_groups)
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

/// The shape the map rule settles its choices on for a card of several
/// multiplicities: every part's shape, summed as if one channel set.
///
/// The choices are one per run, and an artifact banks one. The only choice the
/// rule reads off the shape is the split angle, taken as the soft-emission map
/// wherever any channel has a soft-emission split; on a part with none that map
/// shapes nothing, so the union settles each part where it would settle alone.
pub fn union_shape(shapes: impl IntoIterator<Item = ProcessShape>) -> ProcessShape {
    shapes
        .into_iter()
        .fold(ProcessShape::default(), |acc, s| ProcessShape {
            soft_emission_splits: acc.soft_emission_splits + s.soft_emission_splits,
            moving_splits: acc.moving_splits + s.moving_splits,
            max_rungs: acc.max_rungs.max(s.max_rungs),
            whole_state_resonance: acc.whole_state_resonance || s.whole_state_resonance,
        })
}

/// A hadronic cross section summed over final-state multiplicities: one
/// [`ProtonIntegrand`] per multiplicity, their channels exposed as one list.
///
/// Channel `c` is channel `c − offset(k)` of part `k`, where the parts are in
/// increasing multiplicity and `offset(k)` counts the channels of the parts
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
}

impl<'a> MultiplicitySum<'a> {
    /// Compose the parts, which must be in increasing final-state multiplicity.
    ///
    /// # Panics
    ///
    /// If there is no part, if two parts share a multiplicity or are out of
    /// order, or if the parts were built under different maps — the artifact banks
    /// one set of choices, and a replay rebuilds every part under it.
    pub fn new(parts: Vec<ProtonIntegrand<'a>>) -> Self {
        assert!(!parts.is_empty(), "a multiplicity sum needs a part");
        for pair in parts.windows(2) {
            assert!(
                final_state_count(&pair[0]) < final_state_count(&pair[1]),
                "the parts are one per multiplicity, in increasing order"
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
        }
    }

    /// The parts, in increasing multiplicity.
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
    pub fn locate(&self, channel: usize) -> (usize, usize) {
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
    pub fn scale_draw_ndim(&self) -> usize {
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
    pub fn value_in_channel(&self, channel: usize, u: &[f64]) -> f64 {
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

    /// The key each channel's grid is banked under, in channel order. A single
    /// part keeps its own [`ChannelKey::GroupChannel`] keys, so its artifact is
    /// the one a single-multiplicity card has always written. A channel whose map
    /// serves several `(group, diagram)` pairs is a [`ChannelKey::MergedChannel`]
    /// whatever the part count.
    pub fn channel_keys(&self) -> Vec<ChannelKey> {
        let single = self.parts.len() == 1;
        self.parts
            .iter()
            .flat_map(|part| {
                let final_state = final_state_count(part);
                part.channel_ids()
                    .iter()
                    .zip(part.channel_members())
                    .map(move |(id, members)| {
                        if members.len() > 1 {
                            ChannelKey::MergedChannel {
                                final_state,
                                group: id.group,
                                channel: id.channel,
                                pairs: members.len(),
                            }
                        } else if single {
                            ChannelKey::GroupChannel {
                                group: id.group,
                                channel: id.channel,
                            }
                        } else {
                            ChannelKey::MultiplicityChannel {
                                final_state,
                                group: id.group,
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

    /// Every channel's selection weight in its own part's mixture, in channel
    /// order — the `αⱼ` each term carries. They sum to one per part.
    pub fn channel_alphas(&self) -> Vec<f64> {
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
    pub fn budget_shares(&self) -> &[f64] {
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
                    "{} outgoing legs: {} channels, survey mean {:.3e}, estimator spread {:.3e}, \
                     {:.1}% of the budget",
                    self.final_state_count(k),
                    self.parts[k].channel_count(),
                    self.parts[k].survey_mean(),
                    sd[k],
                    100.0 * share
                );
            }
        }
        adaptations
    }

    /// Install budget shares taken from elsewhere instead of the survey's.
    ///
    /// # Panics
    ///
    /// If `shares` is not one positive share per part summing to one.
    pub fn set_budget_shares(&mut self, shares: Vec<f64>) {
        assert_eq!(shares.len(), self.parts.len(), "one share per part");
        let sum: f64 = shares.iter().sum();
        assert!(
            shares.iter().all(|&s| s > 0.0) && (sum - 1.0).abs() < 1e-9,
            "the budget shares are positive and sum to one"
        );
        self.budget_shares = shares;
    }

    /// The per-channel shares the budget is split by: each channel's `αⱼ` times
    /// its part's budget share. With one part, the part's own weights.
    pub fn allocation_alphas(&self) -> Vec<f64> {
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
}

fn final_state_count(part: &ProtonIntegrand<'_>) -> usize {
    part.groups().groups()[0].final_masses().len()
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
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::helas::eval::BoundAmplitude;
    use crate::pdf::grid::SubGrid;
    use crate::pdf::PdfMember;
    use crate::phasespace::maps::MapOptions;
    use crate::phasespace::rng::SubStream;
    use crate::proton::{derive_flavor_groups, FlavorGroups};
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
            SQRT_S_HAD,
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
