---
type: Physics Convention
title: Colour lines from the basis, and how flows transform under conjugation
description: "ICOLUP lines are read from each basis key's T/Tr chains, incoming legs take the conjugate rep, only connectivity is physical; full vs partial (crossing) conjugation of a flow."
status: draft
tags: [colour, lhef, icolup, conjugation, colour-flow]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: code-tags, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/color/flow_tags.rs#L1-L60", title: "helas/color/flow_tags.rs module doc: chain reading, crossing rule, labels"}
  - {id: code-conj, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/color/flow_tags.rs#L195-L230", title: "ColorFlowTags::conjugated (full conjugates only)"}
  - {id: code-perm, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/proton.rs#L1050-L1100", title: "proton.rs: flow_permutation by flow fingerprint"}
  - {id: code-record, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/lhef/build.rs#L195-L230", title: "lhef/build.rs: SubprocessRecord::relabelled takes the member's own flows"}
  - {id: n23-e1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/23-event-output-lhef-plan.md#L66-L195", title: "Note 23 E1b and E1 outcome: deriving tags from the basis"}
  - {id: n29-a0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L1994-L2079", title: "Note 29 A.0: per-isproc leshouche tables, full conjugation"}
  - {id: n29-b0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L2437-L2511", title: "Note 29 B.0–B.1: the crossing class; the permutation is a computation"}
  - {id: n29-c2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L2771-L2794", title: "Note 29 C.2: a basis against itself is the identity"}
  - {id: n29-f10, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L830-L976", title: "Note 29 F.10: H2 and H7 (colour transpose; per-diagram phase across flows)"}
---

# Colour lines from the basis, and how flows transform under conjugation

## Reading a basis key as colour lines

Each colour-basis element is keyed by a simplified product of `T(a…, i, j)`
chains and `Tr(a…)` traces over the external colour indices. In the double-line
picture that key *is* the flow's set of colour lines, so the Les Houches
`ICOLUP(1..2, leg)` pair per leg is derived from it, not transcribed from a
reference (`helas/color/flow_tags.rs`, `ColorFlowTags`).[^code-tags]

Every line endpoint is a `(leg, index rep)` pair: a quark or antiquark leg carries
one index, a gluon both a `3` and a `3̄`.

- `T([a₁…aₙ], i, j)` links `(i, 3) — (a₁, 3̄)`, `(a_k, 3) — (a_{k+1}, 3̄)`, and
  `(aₙ, 3) — (j, 3̄)`. With no adjoint indices it is the single line
  `(i, 3) — (j, 3̄)` of a δ.
- `Tr([a₁…aₙ])` closes the same links cyclically, `(aₙ, 3) — (a₁, 3̄)`.

**The crossing rule.** MadGraph's colour structure treats every leg as outgoing,
so a leg's index rep is its particle's rep when outgoing and the conjugate rep
when incoming. `ICOLUP` slot 1 is the *physical* colour, slot 2 the physical
anticolour. A `3` index therefore lands in the colour slot of an outgoing leg and
the anticolour slot of an incoming one, and the reverse for a `3̄`.
`color_flow_tags` checks per flow that the occupied slots are exactly those the
leg's particle rep allows (triplet: colour only; antitriplet: anticolour only;
octet: both; singlet: neither), each once. The rule is pinned by
`crossing_rule_is_not_free`: flipping it puts an incoming quark's line in its
anticolour slot and the check fires.

**Only connectivity is physical.** Line labels start at 501 (MadGraph's pool) in
derivation order; any consistent relabelling is the same event. Comparisons use
the induced connectivity (which `(leg, slot)` endpoints share a label), never the
integers. Of the 24 MadGraph subprocesses first compared, 20 matched MadGraph's
numbering literally and four gluon-initiated ones (`gg_to_gg`, `gg_to_ttx`, both
`gg_bbx`) were relabellings, so a byte-level `.lhe` diff must normalise colour
labels first.[^n23-e1]

**Flow indices are MadGraph's.** The basis is in MadGraph's sorted-key JAMP order,
including `g g > g g` at `NCOLOR = 6`: the six keys `Tr(1,2,3,4)`, `Tr(1,2,4,3)`,
`Tr(1,3,2,4)`, `Tr(1,3,4,2)`, `Tr(1,4,2,3)`, `Tr(1,4,3,2)` are MadGraph's six
structures in order, and the JAMPs agree flow by flow under the process's one
global constant (see [global-phase-i-counting](global-phase-i-counting.md)). So the
tags can be compared to `leshouche.inc` element-wise per flow index.

Why derive rather than transcribe: a table transcribed from `leshouche.inc` agrees
with MadGraph by construction and cannot detect a mislabelling, and a permuted or
transposed dictionary is invisible to every `|M|²` gate because `|M|²` contracts
the flows away. Permuting flows 0↔1 in the derived table fails 7 subprocesses,
`g g > g g` among them, so the oracle sees what `|M|²` cannot.

The per-flow sign does not need a per-flow phase convention: on `u u~ > u u~` both
diagrams show the same `(+, −)` sign pattern across the two flows, with magnitudes
`1/6` and `1/2` — the colour weights. One per-diagram scalar sign serves every
flow.[^n29-f10]

## Conjugation of a flow

A hadron-collider run compiles one representative per flavour group and reuses
its `|M|²` for every member. Members can still route their colour lines
differently, so each member writes its own table (design in
[events/per-member-colour-flow-tables](../events/per-member-colour-flow-tables.md)).
The derivation of how two members' flows relate:

**Full conjugation** (every leg's rep conjugated: `g u > g u` ↔ `g u~ > g u~`,
`u u > u u` ↔ `u~ u~ > u~ u~`). Charge conjugation maps a basis key as
`T(a₁…aₙ, i, j)* = T(aₙ…a₁, j, i)`. That flips every endpoint's index rep while
keeping which leg each endpoint sits on and which endpoints pair into a line, so
**the conjugate's tags are the representative's with both `ICOLUP` slots
exchanged on every leg**.[^n29-a0] `ColorFlowTags::conjugated` implements exactly
that and is used only as a test oracle. It is not "swap slots on the legs whose rep
changed": that leaves a gluon's endpoints in place while its partners move, which
breaks lines while staying legal on every leg.[^code-conj]

**Partial conjugation — the crossing class** (`u c > u c` ↔ `u c~ > u c~`, two of
four legs conjugated). **No slot operation relates the two tables.** Conjugating
one end of a line re-routes it onto a different pair of legs; a slot exchange can
only move an endpoint between the two slots of its own leg.[^n29-b0]

```text
isproc 4  u c  > u c     flow 1 {1c,3c} {2c,4c}   flow 2 {1c,4c} {2c,3c}
isproc 6  u c~ > u c~    flow 1 {1c,2a} {3c,4a}   flow 2 {1c,3c} {2a,4a}
```

In Fierz terms, `T^a_{31}T^a_{42} = ½(δ₃₂δ₄₁ − Nc⁻¹δ₃₁δ₄₂)` becomes
`T^a_{31}T^a_{24} = ½(δ₃₄δ₂₁ − Nc⁻¹δ₃₁δ₂₄)`: the subleading pairing `{1,3}{2,4}`
survives, the leading one moves from `{1,4}{2,3}` to `{1,2}{3,4}`. In MadGraph's own
banked `pp_to_jj` sample these single-diagram subprocesses emit only their leading
flow (35 events in `{1c,4c}{2c,3c}`, 48 in `{1c,2a}{3c,4a}`), a categorical
confirmation that the correspondence maps flow 2 ↦ 1. Twelve of the 65 dijet
flavour assignments are in this class.

**The flow permutation is computed, never assumed.** The member's basis element
corresponding to the representative's flow `f` is the one carrying the same
amplitude, `JAMP'_{π(f)} = ± JAMP_f`; squaring removes the sign, so the draw taken
off the representative is correct once labelled with `π(f)`. The permutation does
not follow from the class:[^n29-b0]

| representative → member | tags | flow index |
|---|---|---|
| `g u > g u` → `g u~ > g u~` | global slot exchange | reversed (1↔2) |
| `u u > u u` → `u~ u~ > u~ u~` | global slot exchange | preserved |
| `u c > u c` → `u c~ > u c~` | no slot relation | reversed (2 ↦ 1) |

The code therefore compiles each member and takes **its own** tags; the
permutation `π` is found by matching flow fingerprints (per contribution: diagram,
colour chain, power of `Nc`, |coefficient|, sign and `i` excluded because charge
conjugation flips them) in `proton.rs::flow_permutation`.[^code-perm] The table is
reindexed into the representative's flow order once, and
`SubprocessRecord::relabelled` takes the member's own legs and flows and checks
the table against the member's reps before writing.[^code-record]

**Degenerate flows.** A trace and its reverse carry the same contributions and the
same `JAMP2` (the gluon amplitudes' reflection identity), so no fingerprint
separates them. Against itself a representative needs no matching — the table
indexed is the table drawn from, the identity by construction. Between two
*distinct* subprocesses with a degenerate basis the pairing stays refused, because
that ambiguity is real and would show in the emitted `ICOLUP`. `g g > g g` is a
single flavour assignment and cannot become such a group.[^n29-c2]

The reference for all of this is MadGraph's per-subprocess `leshouche.inc`, which
carries `ICOLUP(slot, leg, iflow, isproc)` for every `isproc`, one-for-one with
`matrix<N>_orig.f`. The oracle covers all **73** concrete subprocesses over **47**
`leshouche.inc` files of the banked tree. Selection of the flow written per event:
[events/colour-and-helicity-selection](../events/colour-and-helicity-selection.md);
colour oracles: [validation/colour-oracles](../validation/colour-oracles.md). The
basis these lines come from: [madgraph-colour-factorization](madgraph-colour-factorization.md).

[^code-tags]: `vibegraph-lib/src/helas/color/flow_tags.rs`, module documentation and `color_flow_tags`.
[^code-conj]: `flow_tags.rs`, `ColorFlowTags::conjugated` doc.
[^code-perm]: `vibegraph-lib/src/proton.rs`, `flow_permutation`; `compile.rs`, `FlowFingerprint`.
[^code-record]: `vibegraph-lib/src/lhef/build.rs`, `SubprocessRecord::relabelled`.
[^n23-e1]: Note 23 §E1b and E1 outcome. The 20/24 count is from that comparison; the oracle has since widened to 73 subprocesses.
[^n29-a0]: Note 29 §A.0, Fact 3 — valid for full conjugates only.
[^n29-b0]: Note 29 §B.0–B.1.
[^n29-c2]: Note 29 §C.2.
[^n29-f10]: Note 29 §F.10, H7.
