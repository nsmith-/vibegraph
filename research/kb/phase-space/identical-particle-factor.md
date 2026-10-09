---
type: Physics Convention
title: Identical final-state particle factor
description: "1/∏ n_s! over outgoing species keyed on (id, polarization), from phasespace::identical_particle_factor, applied per subprocess, never in a map weight."
status: draft
tags: [phase-space, symmetry-factor, identical-particles, polarization, cross-section]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n22-found, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/22-dynamical-scales-plan.md#L339-L369", title: "Note 22 close-out (gg → gg exactly twice MadGraph)"}
  - {id: n28-s1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L1299-L1363", title: "Note 28 S1 (channel-enumeration decision for identical particles)"}
  - {id: n38-p1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L882-L1031", title: "Note 38 P1 (polarized external particles)"}
  - {id: mg-ipf, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/core/base_objects.py#L3742", title: "MadGraph base_objects.py identical_particle_factor"}
---

# Identical final-state particle factor

`dΦ_n` is written on *labelled* outgoing legs, so it counts every ordering of
identical outgoing particles as a distinct configuration. Each subprocess
undoes that over-counting with

```
S = 1 / ∏_s n_s!
```

where `n_s` is the number of outgoing legs of species `s`. `g g → g g` is the
case that makes it visible: without its `1/2!` the cross section comes out
exactly twice MadGraph's, with `|M|²`, the flux and the initial-state average
already agreeing.[^n22-found]

## The single definition

`phasespace::identical_particle_factor(outgoing)`
(`vibegraph-lib/src/phasespace/mod.rs:107`) is the only implementation. It takes
any labels that compare equal for the same species; model particle ids and PDG
codes both work and both separate a particle from its antiparticle
(`[2, -2]` gives 1, `[2, 2, -2]` gives 1/2). Its unit tests sit beside it.

**Species means `(id, polarization)`.** Two legs of one particle count as
identical only when they carry the same polarization, as in MadGraph's
`identical_particle_factor` (`base_objects.py:3742`), which keys on
`(id, polarization)`.[^mg-ipf][^n38-p1]

| Final state | Factor |
|---|---|
| `z{0} z{0}`, `z{T} z{T}` | 1/2 |
| `z{L} z{R}`, `z{0} z{T}` | 1 |

Both callers zip the outgoing particles with their polarizations before calling
the function: `hadronic::outgoing_symmetry_factor` (`hadronic.rs:1045`, from a
compiled evaluator) and `proton::Subprocess::symmetry_factor` (`proton.rs:207`,
from a concrete flavour assignment). Polarization itself is
[process/polarization](../process/polarization.md).

## Where the factor is applied

The factor belongs to one subprocess's final state, **not to a map weight**.
A hadronic run pools the channels of every subprocess whose outgoing *masses*
agree, while their species need not: `g g → g g` and `u ū → d d̄` both sample at
`[0, 0]`. A map weight carrying the factor would stop being a density on the
one labelled `dΦ_n` those subprocesses share.[^n28-s1]

- **Summed matrix element.** `SubprocessProto` / `BoundSubprocess` carry their
  own amplitude's factor, and the sum multiplies each term by it
  (`hadronic.rs:2343`, `m2 += sub.symmetry_factor() * …`). There is no
  integrand-level factor field, so nothing can derive it from one
  representative amplitude.
- **Flavour groups.** Members of a group share `|M|²` but nothing in the
  grouping rule fixes their outgoing multiset, so the factor is per *member*:
  `FlavorGroup::symmetry_weighted_luminosity` (`proton.rs:447`) folds each
  member's own `S_i` into the luminosity sum,
  `Σ_members S_i · xf_a(x₁) · xf_b(x₂)`. Two members of one group with
  different outgoing multisets therefore carry different factors.

The conventions for the full weight chain (flux, averaging, this factor) are in
[pipeline/overview](../pipeline/overview.md).

## Why permutations are not extra channels

Multichannel treats a repeated outgoing species as one channel set over the full
labelled `dΦ_n`. The per-diagram channel set is already closed under swaps of
identical outgoing legs: the image of a diagram under such a swap is another
diagram of the same process (`g g → g g`'s `t` and `u` channels are each
other's image). Enumerating permutation copies would multiply the per-point
density cost by `∏ n_s!` for no new coverage and would manufacture exactly
degenerate channels that α-adaptation cannot separate.[^n28-s1]

A fundamental-domain map (one ordering only) was rejected: a channel must
report its density at a *foreign* point (see
[phase-space/channel-contract](channel-contract.md)), which need not lie in the
domain, so every channel would have to symmetrise over the `n_s!` images; and
the cut filter and event record are written on labelled legs.

The closure is pinned, not assumed:
`the_channel_set_of_identical_outgoing_legs_is_permutation_closed`
(`hadronic.rs:3272`) checks that the combined density of `g g → g g`'s channel
set is invariant under exchanging the two outgoing momenta, and refuses to pass
unless dropping a single channel breaks the invariance. The control matters:
built *unregulated* (spacelike floor zero), all four channels collapse to a
common all-timelike map whose density is symmetric one channel at a time, and
the invariance check alone would see nothing. The test therefore builds the
channels at the floor a hadronic run gives them
([phase-space/spacelike-floor](spacelike-floor.md)).

## Decays

Identical particles produced in different decays of a decay chain are a
separate question (which pairings to keep):
[process/identical-particles-across-decays](../process/identical-particles-across-decays.md).

[^n22-found]: Note 22, sprint close-out: the factor's discovery on `gg_to_gg`.
[^n28-s1]: Note 28 §S1: the channel-enumeration decision and where the factor lives.
[^n38-p1]: Note 38 P1: polarization keys the factor, with MadGraph citations.
[^mg-ipf]: MadGraph `identical_particle_factor` at `b7687064`.
