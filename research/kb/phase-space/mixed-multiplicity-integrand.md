---
type: Design
title: Mixed-multiplicity composite integrand
description: "MultiplicitySum: one ProtonIntegrand per @N multiplicity with its own channels and grid dimension (channel_grid_ndim), a budget split by part spread, union-shape maps, versioned keys."
status: draft
tags: [phase-space, mlm, multiplicity, integrand, artifact]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n41-33, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L253-L275", title: "Note 41 §3.3 (a composite integrand, not a wider one)"}
  - {id: n41-m3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L1014-L1214", title: "Note 41 M3 (MultiplicitySum as built)"}
  - {id: n41-dec, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L3490-L3512", title: "Note 41 §5 decisions (mixed multiplicity without matching)"}
---

# Mixed-multiplicity composite integrand

A proc card whose process lines differ in outgoing-leg count
(`p p > e+ e- @0`, `add process p p > e+ e- j @1`, …) describes one sample over
several phase spaces of different dimension. MadEvent integrates each `P<n>`
directory on its own and unweights every channel of every directory together.
`MultiplicitySum` (`vibegraph-lib/src/multiplicity.rs`) is the same
arrangement.[^n41-33]

## Shape

- **`split_by_multiplicity(sets, card)`** partitions the enumeration by
  outgoing-leg count, in increasing order, keeping the card's order within each
  part; empty sets are dropped.
- **`MultiplicitySum`** owns one fully configured `ProtonIntegrand` per part and
  exposes their channels as one list: channel `c` is channel `c − offset(k)` of
  part `k`. Each part keeps its own channels, grids, and selection weights `α_j`
  normalised over its own channels, so each part's terms sum to its own σ and
  all of them to the total (`part_results` gives σ per multiplicity).
- **With one part** it *is* that part, bit for bit: every channel, value, key
  and allocation.

The channel-split estimator is a sum of terms either way, so the unweighter
needed no change: it draws a channel `∝ w_max_j` whatever multiplicity it
belongs to ([events/unweighting](../events/unweighting.md)).

## Per-channel grid dimension, not padding

`ChannelIntegrand::channel_grid_ndim(channel)` (`unweight.rs:220`) takes the
channel index and is a **required** trait method with no default: the trait
holds no constant a default could return, so every uniform implementer ignores
the index. `MultiplicitySum` returns the owning part's dimension
(`multiplicity.rs:181`, and the trait impl at `:430`); the concrete integrands
keep an inherent zero-argument `channel_grid_ndim()`. `integrate_channels`
builds each grid at its channel's dimension and `Unweighter` scans and draws
each channel at its own.

The rejected alternative, padding every channel to the widest dimension, needs
no trait change but gives grids axes nothing depends on and changes the uniform
stream every channel draws. `scale_draw_ndim` is the most trailing uniforms any
part consumes; a part that needs fewer reads the leading ones.

## Budget across parts

Each part is α-surveyed on its own mixture, then the per-iteration budget is
split between parts by `n_k ∝ s_k`, the standard deviation of part `k`'s
undivided mixture estimator (`√(Σ_j α_j W_j − σ_k²)` from the survey;
`MultiplicitySum::adapt_alphas`); a channel gets `α_j · share_k`, and the banked
`alpha` is the term's own `α_j`. Under Neyman, every channel is re-split by its
measured spread from the second iteration on, so `s_k` sets only the first
iteration. The floors and the re-split are
[phase-space/channel-budget-allocation](channel-budget-allocation.md); the
matched two-jet part's heavy tail is what this split does not yet price.

## Channels

Each part builds its channels as a single-multiplicity proton run does,
including merging pairs with identical maps
([phase-space/channel-set](channel-set.md)). On `pp_to_ll_0j2j_mlm` that is
1 + 6 + 36 = 43 sampling channels.

## Maps

An artifact banks one set of map choices. For several parts they are settled
once over `union_shape` of the parts' shapes. The only shape-dependent choice
is the split angle (soft-emission wherever any channel has a soft-emission
split), and a part with no such split is unaffected by it, so the union settles
each part where it would settle alone
([phase-space/map-choices](map-choices.md)).

## Where it applies, and what it refuses

- **Proton beams only.** Fixed-energy beams and decays have one phase space and
  refuse a mixed card (`refuse_mixed_multiplicity`,
  `vibegraph-cli/src/integrate.rs:804`).
- **Without matching** (`ickkw = 0`) a mixed card runs, as MadGraph runs it, with
  a warning that the sum double counts the radiation the higher multiplicities
  share with the lower ones.[^n41-dec] MadGraph itself only auto-enables
  matching in the default card it writes for such a process. Matching is
  [scales-pdf/mlm-matching](../scales-pdf/mlm-matching.md).

## Record and normalisation

Each event's `IDPRUP` is its member's `@N`: MadEvent writes the `P<n>`
directory number, which `group_subprocs.py` sets to the group's first process's
`id`, i.e. its `@N`, so the two coincide. `<init>` keeps one line per `@N`, and
the buffered writer normalises each part's weights to that part's integrated σ
([events/multi-process-normalisation](../events/multi-process-normalisation.md)).

## Artifacts

Channels of a several-part run are banked as
`ChannelKey::MultiplicityChannel { final_state, group, channel }` (a merged one
as `MergedChannel`; a single-part run keeps `GroupChannel`). The writer records
the oldest format version that holds its keys, so a single-multiplicity
artifact is still byte-for-byte the older version. `generate` refuses an
artifact below `MULTIPLICITY_VERSION` on a card of several multiplicities,
naming both versions (`refuse_stale_artifact_on_mixed_multiplicity`), and an
older build refuses a newer file by its version. The version table is
`artifact.rs`'s `FORMAT_VERSION` doc comment
(`MULTIPLICITY_VERSION = 10`, `FORMAT_VERSION = 11` at the time of writing);
see [pipeline/artifact-format-versioning](../pipeline/artifact-format-versioning.md).

## Tests

Hermetic tests in `multiplicity.rs` pin the split order, offsets and `locate`,
per-channel `ndim`, keys and samplers, values bit for bit against the parts, a
single part equal to itself bit for bit, and σ per part matching each part
integrated alone (pulls +0.17, +0.75), with an unweighted sample splitting
across multiplicities as σ does. End to end,
`cli_generate_proton::a_mixed_multiplicity_card_is_integrated_and_sampled_as_a_sum`
runs `@0 + @1` on the banked card: the warning, version, keys, grid dimensions,
per-part `α_j` sums, `IDPRUP`, the sample split and the stale-version
refusal.[^n41-m3] The banked σ gate on `pp_to_ll_0j2j_mlm` reads per `@N` and in
total ([validation/mlm-sigma-gate](../validation/mlm-sigma-gate.md)).

Per-point cost is dominated by the two-jet part's mixture density and per-group
reclustering ([backlog](../backlog/performance/matched-mixture-reclustering-per-point-cost.md)).

[^n41-33]: Note 41 §3.3: the composite design and the rejected padding.
[^n41-m3]: Note 41 M3: `MultiplicitySum` as built, its tests and keys.
[^n41-dec]: Note 41 §5 (a): accept an unmatched mixed card with a warning.
