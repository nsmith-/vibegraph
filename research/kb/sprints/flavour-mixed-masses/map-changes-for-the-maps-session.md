---
type: Sprint Note
title: "Map changes for the maps session: none needed"
description: "F1 splits a mixed-mass card into one integrand per set of outgoing masses, so no map, VEGAS or budget code changes; this records why, and what the in-integrand alternative would have needed."
status: draft
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-22}
---
**No change to the map code is needed.** F1 left `vegas.rs`, `budget.rs`
and `phasespace/` untouched, and the mixed-mass card integrates.

## Why none is needed

A card whose subprocesses differ in outgoing masses is now split before
flavour grouping. There is one part per final-state multiplicity and multiset
of outgoing masses (`split_by_outgoing_masses`, `vibegraph-lib/src/multiplicity.rs`).
Each part gets its own `derive_flavor_groups` call, its own `ProtonIntegrand`,
and so its own channels. Each part builds its channels with today's
construction (`maps.channel(d, model, sqrt_s_had, cuts)`,
`vibegraph-lib/src/proton.rs:1538`) from its own diagrams, and so on its own
mass shell.

The parts are combined by `MultiplicitySum`. That composite already sums
separate integrands for cards of several multiplicities. Each part keeps its
own channel weights, normalised over its own channels, and its own grids.

MadGraph does the same thing. In `madgraph/iolibs/group_subprocs.py` at
b7687064, the process class in `find_process_classes` (lines 482-490) carries
every leg's `p.get('mass')`. `group_amplitudes` (lines 421-449) makes one
`SubProcessGroup`, and so one `P` directory, per class. Each directory is
integrated over its own channels.

The callers changed are `multiplicity_groups`, `MultiplicitySum::new`'s
ordering invariant and `MultiplicitySum::channel_keys`. The keys' `group`
index now counts across the parts of one multiplicity, so two parts never
bank a grid under one key. The artifact format is unchanged, and a card with
one set of masses banks exactly the keys it did before.

## The alternative, and the map change it would need

The brief described a different route: put the mass-split groups inside one
`FlavorGroups` and one `ProtonIntegrand`. That route would be wrong without a
map change. `ProtonIntegrand` evaluates **every** group's `|M|²` at every point
any channel draws: `per_group_sum` and `shared_scale_sum` loop over all groups
(`vibegraph-lib/src/proton.rs:2065`, `:2130`). It also weights the point by
`αⱼ / Σₖ αₖ gₖ(p)` summed over **every** channel (`ScaledMultiChannel::channel_weight_at`,
`vibegraph-lib/src/phasespace/channel.rs:889`). Groups with different masses
live on different mass shells, so this goes wrong in two ways:

- one group's `|M|²` would be read off-shell at the other group's momenta;
- the mixture density would add densities of channels whose map cannot reach
  the point.

For a single mixture over several mass shells, the map code would need:

1. `ScaledMultiChannel::channel_weight_at` / `density_at`
   (`phasespace/channel.rs:889`) restricted to the channels of the drawn
   channel's mass set: `αⱼ / Σ_{k ∈ class(j)} αₖ gₖ(p)`. With that, the
   estimator `Σ_c ∫ f_c` is unbiased, because channel selection still runs
   over all channels.
2. `ProtonIntegrand::shape` evaluating only the groups of the drawn channel's
   mass set. This is integrand code, not map code, and is listed here so the
   pair is complete.

A test that would show the need: build one `ProtonIntegrand` over the two
mass sets of `p p > l+ l-` with `l ∋ ta` (with the refusal bypassed) and
integrate it. Then compare against the two parts integrated separately
(`mixed_mass_parts_integrate_to_one_flavour_each` in `multiplicity.rs`).
Without change 1, the density sums channels that cannot have drawn the point,
and the result is biased. Without change 2, it reads `|M|²` off its mass shell.

The maps session would take this route only to share one channel-selection
mixture across mass sets. Nothing in this sprint needs that.
