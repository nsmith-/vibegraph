---
type: Validation Gate
title: "Colour oracles: CF matrix, colour basis and leshouche tags"
description: "The CF matrix and colour-basis order against matrix1_orig.f with structure controls, and color_flow_tags_oracle against every leshouche.inc subprocess plus a slot-legality scan."
status: draft
tags: [colour-flow, madgraph, oracle, lhef, icolup]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n16-caveat, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/16-color-flow-design.md#L37-L99", title: "Note 16 — the NCOLOR=6 JAMP question, resolved"}
  - {id: n16-strategy, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/16-color-flow-design.md#L423-L456", title: "Note 16 §3 — validation strategy (CF oracle)"}
  - {id: n23-e1b, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/23-event-output-lhef-plan.md#L66-L129", title: "Note 23 E1b — leshouche.inc as the oracle"}
  - {id: n23-e1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/23-event-output-lhef-plan.md#L130-L195", title: "Note 23 E1 outcome — chain reading, crossing rule, strong form at NCOLOR=6"}
  - {id: n23-e1c, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/23-event-output-lhef-plan.md#L196-L257", title: "Note 23 E1c — JAMPs equal MadGraph's under the identity pairing"}
  - {id: n24-mut, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L756-L809", title: "Note 24 P1 — mutation experiments against the colour gates"}
  - {id: n29-a0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L1994-L2079", title: "Note 29 §A.0 — conjugate members and the reversed flow index"}
  - {id: n29-a1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L2080-L2204", title: "Note 29 §A.1 — the widened oracle"}
  - {id: n29-a2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L2205-L2301", title: "Note 29 §A.2 — the instrument ladder"}
  - {id: n29-b4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L2678-L2698", title: "Note 29 §B.4 — trial counts corrected"}
  - {id: n41-z, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L2916-L3489", title: "Note 41 Z — the four-gluon contact's structure order and the structure controls"}
  - {id: code-cf, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/color_cf_oracle.rs", title: "vibegraph-lib/tests/color_cf_oracle.rs"}
  - {id: code-tags, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/color_flow_tags_oracle.rs", title: "vibegraph-lib/tests/color_flow_tags_oracle.rs"}
  - {id: code-leshouche, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/common/leshouche.rs#L260-L290", title: "tests/common/leshouche.rs — the reference-free slot scan"}
---

# Colour oracles: CF matrix, colour basis and leshouche tags

`|M|²` contracts the colour flows away, so it is blind to everything the event
record carries about colour: which flow an event is drawn into, and which legs a
colour line joins. Three banked-layer gates cover that, each against MadGraph's
generated sources rather than its events. They need the banked work area
(`validation/madgraph/output/`) and run under `extended-validation`; each
compiles every subprocess under the model the manifest records for its row, and
follows that row's `amplitudes` mode (an `info` row is compared and printed but
not asserted).

## `color_cf_oracle`: the CF matrix and the decomposition

For every `SubProcesses/P*/matrix*_orig.f` it parses the process header,
`NCOLOR`, `NGRAPHS` and the `DATA (CF(I,J)…)` block, runs this crate's
enumeration and `colorize_process`, and asserts `NCOLOR` and the full CF matrix
(MadGraph prints exact rationals as 16-digit decimals; `CF_REL_TOL = 1e-14`).
It also compares this crate's basis order with MadGraph's structure comments and
prints an `ORDER-DIFF` note where they differ; `g g > g g`'s six sorted keys
`Tr(1,2,3,4) … Tr(1,4,3,2)` match MadGraph's order exactly, which is what makes
per-flow-index comparison meaningful elsewhere[^n23-e1].

CF is a Gram matrix: it is invariant under a uniform transpose of the basis keys
and says nothing about how a multi-structure vertex distributes over the flows.
So the gate also reads MadGraph's `JAMP(i) = … AMP(j)` lines and compares the
per-amplitude coefficient columns graph by graph (`check_jamp`). A graph's
columns are compared as a set under one unit for the graph, independent of the
order its colour structures came in: MadGraph puts the off-shell gluon first in
a four-gluon contact's slots (`VVVV1P0_1(W(1,1), W(1,2), W(1,5), …)`), so on
`g g > t t~ g` its three contact structures arrive in the reverse of this
crate's order with every coefficient equal. The trial
`jamp-normalisation/structure-controls` pins the normalisation on that graph's
own columns: reversed and rotated by `−i` must compare equal; one structure
negated, or one coefficient moved to another flow, must not[^n41-z].

*Blind to*: whether each colour structure multiplies its own Lorentz structure —
that is for the per-flow amplitude gates (`amplitude_oracle`'s `gg_to_gg`,
`standalone_jamps`' `gg_to_ggg` and `uux_to_ggg`). `g g > t t~ g` itself has no
amplitude gate
([gg-ttxg-no-amplitude-gate](../backlog/validation/gg-ttxg-no-amplitude-gate.md)).

## `color_flow_tags_oracle`: ICOLUP connectivity against leshouche.inc

Each `SubProcesses/P*/leshouche.inc` holds MadGraph's evaluated `ICOLUP(slot, leg, iflow, isproc)` table[^n23-e1b]. The
oracle parses **every** `isproc` of every file — `isproc N` is the process
`matrix<N>_orig.f`'s header names — compiles that subprocess, asserts its PDG
codes equal `IDUP(·, 1, isproc)`, and compares this crate's `ColorFlowTags` flow
by flow, by flow index. A missing `matrix<N>_orig.f` is a failure, never a skip.
One trial per `(P* directory, isproc)`, named like `pp_to_jj/P1_qq_qq#3`; the
trial count grows with the banked tree (73 trials over 47 files on the
`refdata-4`-era tree, before the MLM rows added more)[^n29-b4].

Two deliberate choices:

- **Derive, then check.** The tags come from this crate's own basis keys (the
  `T`/`Tr` chains are literally the colour lines); `leshouche.inc` is only the
  check. A transcribed table would agree with MadGraph by construction and could
  not detect a mislabelled basis.
- **Connectivity, not integers.** Colour-line labels are arbitrary, so the
  comparison is of the set of `(leg, slot)` endpoint pairs sharing a label. Label
  equality is reported as information only; gluon-initiated subprocesses relabel
  the same connectivity. A byte-level `.lhe` comparison must therefore normalise
  colour labels.

The conventions it pins — the chain reading `T([a₁…aₙ], i, j)`, the crossing rule
(`ICOLUP` slots are the *physical* colour/anticolour, so an incoming leg's index
rep is the conjugate of its particle's), and the fermion-flow slot swap — are
described in
[colour lines and conjugation](../amplitudes/colour-flow-lines-and-conjugation.md)
and [MadGraph's colour factorization](../amplitudes/madgraph-colour-factorization.md).

**Conjugate members.** Reading past `isproc 1` is what reaches them: a directory
groups `g u > g u` with `g u~ > g u~`, and each has its own table. A conjugate
member's table at flow `f` is the image of the representative's flow `σ(f)`,
where `σ` is the permutation key conjugation `T(a₁…aₙ, i, j)* = T(aₙ…a₁, j, i)`
induces on the sorted basis — for `g q > g q` it swaps the two flows. The obvious
alternative (same index, re-derived from the member's reps) is legal in every
slot and was refuted at 7–8σ from MadGraph's own banked `pp_to_jj`
frequencies[^n29-a0]. This crate builds each member's table from that member's
own compilation; `ColorFlowTags::conjugated` (`helas/color/flow_tags.rs`)
survives as a test oracle for the slot-exchange identity, and
`SubprocessRecord::relabelled` (`lhef/build.rs`) takes the member's flows as an
argument and runs `check_legs` on them before an event can be written. See
[per-member colour-flow tables](../events/per-member-colour-flow-tables.md).

## Reference-free slot legality, and pattern membership

`tests/common/leshouche.rs::illegal_slots` judges an emitted event against the
Les Houches convention alone: a triplet fills only the colour slot, an
antitriplet only the anticolour slot, an octet both, a singlet neither. It uses
its own PDG → rep table on purpose, so it shares nothing with the generator.
`validate_samples_proton.rs` runs it over generated samples (and it must also
read zero on MadGraph's own file — an instrument that cannot fail on the
reference is not one), and checks every generated event's `(roles,
connectivity)` pattern is one `leshouche.inc` admits, over every `isproc`, every
flow and both beam orderings. The pattern set is built from `leshouche.inc`
rather than from MadGraph's sample, which would be incomplete for rare flows.

The ladder is deliberate, each instrument blind to the next one's failure:
slot legality → pattern membership against the reference → the right flow for
the right `JAMP2` (hermetic, per member) → the right flow *frequencies* (the
`ICOLUP` χ² column of the samples gate)[^n29-a2]. None of the first three sees
frequencies.

## What these gates cannot see

- **Uniform transposes** are invisible to `color_cf_oracle` by construction;
  `color_flow_tags_oracle` and the amplitude gates see them. Mutation experiments
  confirmed this: transposing the `T` chain's fundamental ends, or removing the
  fermion slot swap, failed 22/30 flow-tag trials and the amplitude gates, and
  `color_cf_oracle` passed both[^n24-mut].
- **Single-flow processes with one adjoint index** (the `p p > l+ l- j`
  subprocesses, `NCOLOR = 1`) have exactly two colour lines, fully determined by
  the leg reps, so the connectivity comparison is forced and carries almost no
  information there: a slot rule wrong only for `g u~` surfaced as a compile
  error, not a disagreement. Do not lean on this oracle for colour correctness on
  those rows; their colour arrangement is pinned by construction.
- **Flow frequencies and correlations** with other event fields.
- **The `g g > g g` trace-reversal degeneracy** (`J₁ = J₆, J₂ = J₄, J₃ = J₅`) is
  invisible to JAMP values, `JAMP2` and `|M|²`, but *not* to this oracle, which
  sees the two orientations as different connectivities[^n23-e1c].

## The JAMP values themselves

Per-flow JAMP values are compared by [the amplitude oracle](amplitude-oracle.md),
which absorbed the earlier standalone JAMP gate. At `NCOLOR = 6` (`g g > g g`)
the bases are identical in order and this crate's JAMPs equal MadGraph's
element-wise up to one global phase, at every point and helicity (worst
3.7e-16)[^n16-caveat]. The CF matrix there is `(7/2)I + P − (1/3)J` with `P` the
trace-reversal involution; its eigenvalues 5/2 (×4) and 9/2 (×2) make it
positive definite, so no flow combination is invisible to the contraction. The
danger is its large automorphism group, which permutes `JAMP2` while leaving
`|M|²` exact — which is what the per-flow comparison closes.

[^n16-caveat]: Note 16, "The NCOLOR=6 JAMP caveat, resolved". The earlier "not a 1:1 labelling" reading was an artefact of a greedy matcher on a rank-1 JAMP matrix; see [bit-exact amplitude debugging](bit-exact-amplitude-debugging.md).
[^n23-e1b]: Note 23 E1b.
[^n23-e1]: Note 23 E1 outcome.
[^n23-e1c]: Note 23 E1c.
[^n24-mut]: Note 24 P1, "what would have been caught?".
[^n29-a0]: Note 29 §A.0, Facts 1–3.
[^n29-a2]: Note 29 §A.2. The `ColorRepsUnrelated` error that design proposed was not built; the member supplies its own table instead.
[^n29-b4]: Note 29 §B.4; §A.0's 79/49 was an overcount.
[^n41-z]: Note 41 Z1 close-out record.
