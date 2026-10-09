---
type: Algorithm
title: Flavour groups derived from measured |M|² equality
description: "Subprocesses are grouped by pointwise |M|² agreement at shared probe points, also requiring equal masses, cuts and colour basis, with a minimum cross-group separation."
status: draft
tags: [hadronic, flavour-groups, proton, probe, colour]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n24-p1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L834-L854", title: "Note 24 P1 outcome (what P2 must take from it)"}
  - {id: n24-p2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L943-L1010", title: "Note 24 P2 (group by measured |M|²)"}
  - {id: n24-rule, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1202-L1251", title: "Note 24 P2c (the grouping rule, as implemented)"}
  - {id: n24-qqbar, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1296-L1310", title: "Note 24 P2c (q ↔ q̄ grouping: checked, not asserted)"}
  - {id: n24-sym, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1311-L1323", title: "Note 24 P2c (symmetry factor)"}
  - {id: n24-api, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1349-L1386", title: "Note 24 P2c (what the ProtonIntegrand session must know)"}
  - {id: proton-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/proton.rs#L1-L70", title: "proton.rs module documentation"}
  - {id: derive, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/proton.rs#L758-L979", title: "derive_flavor_groups"}
---

# Flavour groups derived from measured `|M|²` equality

A proc card with beam multiparticles (`p p > l+ l- j`) enumerates many concrete
subprocesses, most of which are the same partonic calculation with different flavour
labels on the incoming legs. `derive_flavor_groups(sets, model, evaluated, card)`
(`vibegraph-lib/src/proton.rs`) partitions them into `FlavorGroup`s: one compiled
amplitude, one phase-space map and one cut filter per group, with each group's
luminosity summed over its members. The integrand that consumes them is
[hadronic/proton-integrand](proton-integrand.md).[^proton-rs]

**The partition is measured, not listed.** Nothing names the coupling classes
(up-type against down-type), the generation copies (`u`/`c`, `d`/`s`), the lepton
flavours (`e`/`mu`) or the split between `q g` and `q̄ g`; all of them fall out of the
measurement.[^n24-p2]

## The rule

1. Drop empty `DiagramSet`s and put every subprocess's outgoing legs in the first one's
   order of mass and cut class (`content_ordered`), so `p p > w+ j @1` and
   `add process p p > j w- @2` group instead of being refused for slot-ordered masses.
   Each member keeps its own line's process number.
2. Compile every subprocess. Refuse one with other than two incoming legs
   (`NotTwoIncoming`) or a massive incoming leg (`MassiveInitialState`), and refuse
   unequal outgoing pole masses across subprocesses (`UnequalFinalMasses`): one
   phase-space map has to serve the whole sum.
3. Evaluate every compiled `|M|²` at a shared set of **probe points**: massless beams
   on ±z and a flat RAMBO draw over the outgoing legs, four points per rung
   (`PROBE_POINTS_PER_ENERGY`), at the rungs `probe_energies` returns:

   | rung | why |
   |---|---|
   | `0.2 × base` | the integrator visits `ŝ` below the electroweak scale; a pair agreeing only above it would merge silently |
   | the model's `Z` pole, if it has a massive `Z` | a propagator on its pole is where two subprocesses' weak content separates most sharply |
   | `3×`, `5×`, `13× base` | spread over a decade |

   with `base = max(Σ m_out, 100 GeV)`, every rung raised to at least `1.2 × Σ m_out`, and
   rungs that collide after the clamp collapsed to one.[^derive]
4. Two subprocesses join a group when they are polarized alike and their whole probe
   trace agrees to `GROUP_REL_TOL = 1e-10`.
5. Refuse the partition if two distinct groups separate by no more than
   `GROUP_SEPARATION_MIN = 1e-6` at their best-separated probe point (`DegenerateGroups`).
   A partition made by two traces landing either side of the tolerance by rounding is a
   failed measurement, not a decomposition.
6. Within each group, refuse a member whose compiled `Cuts` differ from the
   representative's (`CutIndicatorDiffers`; `Cuts: PartialEq` is equality of the
   *filter*, since PDG codes are consumed by `compile`), or whose colour basis differs
   in `n_flows` or `cf_matrix` (`ColorStructureDiffers`).
7. Pair each member's colour flows with the representative's (`flow_permutation`),
   refusing when no bijection exists (`ColorFlowPairing`). The member's own `ICOLUP`
   table is reindexed into the representative's flow indexing once, so a flow drawn on
   the representative labels any member's event.

## Why `|M|²` equality is not enough

`|M|²` is a sum. It is blind to a global phase, and it would not move if two members'
colour bases differed by a relabelling. So equal `|M|²` does not license reusing one
member's colour flow, cuts or phase-space map for another, and steps 2, 6 and 7 require
each of those separately.[^n24-rule] The extra requirements are pinned against cases
where they differ, so they are not vacuous guards:

- `a_group_sharing_one_cut_filter_is_a_real_requirement`: a `pdg = 5` leg compiles to a
  different filter from a light jet at `maxjetflavor = 4` and to the same one at `5`.
- The colour check is exercised on `p p → t t̄ QED=0`, where both groups have
  `n_flows = 2` and only the CF matrix separates them.

Members of a group share a matrix element but need not share colour reps: a quark and
an antiquark can share `|M|²`, mass list, cut filter and CF matrix while routing colour
lines between different legs. That is why each member keeps its own reps and its own
reindexed flow table ([events/per-member-colour-flow-tables](../events/per-member-colour-flow-tables.md)).

## What the probe settles that σ̂ cannot

On `p p > l+ l- j QCD=2 QED=2` the partition is **6 groups of 4**, with bit-for-bit
agreement inside each group:[^n24-rule]

| group | members | `spin_color_average` |
|---|---|---|
| `g u > e+ e- u` | `g u`, `g c` × `e`, `mu` | 1/96 |
| `g d > e+ e- d` | `g d`, `g s` × `e`, `mu` | 1/96 |
| `g u~ > e+ e- u~` | `g u~`, `g c~` × `e`, `mu` | 1/96 |
| `g d~ > e+ e- d~` | `g d~`, `g s~` × `e`, `mu` | 1/96 |
| `u u~ > e+ e- g` | `u u~`, `c c~` × `e`, `mu` | 1/36 |
| `d d~ > e+ e- g` | `d d~`, `s s~` × `e`, `mu` | 1/36 |

The `e`/`mu` multiplicity needs no special case: the two lepton flavours are distinct
members with the same initial state, and the luminosity sum supplies the factor of two.
The closest two groups separate by `0.74` at points the partition was not fitted on;
`group_members_agree_where_the_partition_was_not_measured` asserts that margin stays
above `0.1`.

**`g q` and `g q̄` are separate groups.** Their banked partonic σ̂ at `√ŝ = 500` agree
within MC error (`0.11812 ± 0.00022` and `0.11816 ± 0.00026` pb), but their pointwise
`|M|²` differ by up to `0.93`
(`a_quark_and_its_antiquark_against_a_gluon_do_not_share_a_matrix_element`). A grouping
criterion built on σ̂ would have merged them, summing the antiquark's luminosity against
the quark's matrix element and colour structure. This is a worked case of the rule that
every oracle has a blind spot ([validation/oracle-blind-spots-and-non-vacuity](../validation/oracle-blind-spots-and-non-vacuity.md)).[^n24-qqbar]

The probe can trust each side because every `ℓℓj` subprocess class has an enforced
per-diagram amplitude row, so the grouping is checked against matrix elements that are
pinned below `|M|²`.[^n24-p1] Two limits of those rows: they do not pin the spine sign,
and for these single-flow, one-adjoint processes `color_flow_tags_oracle` is forced by
the leg reps (`NCOLOR = 1`), so the `ICOLUP` an `ℓℓj` event carries is determined, not
validated by comparison ([validation/colour-oracles](../validation/colour-oracles.md)).

## Identical outgoing particles

Grouping does not constrain the outgoing multiset, so a group can hold members with
different identical-particle factors. Each member carries its own
`Subprocess::symmetry_factor()` into `symmetry_weighted_luminosity`; the factor is a
per-member scalar and never a map weight ([phase-space/identical-particle-factor](../phase-space/identical-particle-factor.md)).
Members of one group compile alike only at the card's own parameters, so each keeps its
own diagram set (`member_diagram_set`).[^n24-sym]

## What the decomposition hands the integrand

Per group: `evaluator()`, `diagrams()` (the channel derivation's input),
`external_legs()`, `cuts()`, `final_masses()`, `spin_color_average()`, `members()`,
`has_mirror()`, `mirror_into`, `luminosity` / `member_luminosity` (and their `_rows`
forms over the two per-beam flavour rows, read once per point by `beam_rows`),
`event_legs` and `event_leg_colors` for the record.[^n24-api] The mirror identity and
`BeamOrdering` are [hadronic/beam-mirror-identity](beam-mirror-identity.md).

Deriving the decomposition compiles **every** subprocess and keeps only the
representatives. On `ℓℓj` that is cheap (enumeration and compilation take tens of
milliseconds), but its cost scales with the subprocess count.

## Known weakness

The criterion is complete but not sound: two programs that differ only where the probe
does not look would be merged silently. It was accepted on the MadGraph precedent (the
helicity filter drops vanishing configurations on the same sampled-probe basis), with the
probe ladder hardened. The sound replacement, grouping by identical canonical compiled
programs, is [backlog: flavour-grouping-unsound-sampled-criterion](../backlog/feature/flavour-grouping-unsound-sampled-criterion.md).
Its subprocess enumeration side is [process/subprocess-enumeration](../process/subprocess-enumeration.md).

[^n24-p1]: Note 24 P1 outcome: llj amplitude rows, `g q` against `g q̄`, and the forced colour-flow oracle.
[^n24-p2]: Note 24 P2, the decision to group by measured `|M|²` rather than a flavour table.
[^n24-rule]: Note 24 P2c, the grouping rule as implemented, the extra requirements and the `ℓℓj` table.
[^n24-qqbar]: Note 24 P2c, `q ↔ q̄` grouping measured.
[^n24-sym]: Note 24 P2c, the symmetry factor; the per-member form is in `proton.rs` (`Subprocess::symmetry_factor`).
[^n24-api]: Note 24 P2c, the API the integrand consumes.
[^proton-rs]: `proton.rs` module documentation.
[^derive]: `derive_flavor_groups`, `probe_energies` and `probe_momenta`, `vibegraph-lib/src/proton.rs`.
