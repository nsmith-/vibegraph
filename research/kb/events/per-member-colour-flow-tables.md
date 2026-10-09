---
type: Design
title: Each flavour-group member writes its own colour-flow table
description: "Flavour-group members can carry conjugate or crossed colour reps, so each writes its own ColorFlowTags under a fingerprint-matched flow permutation; ambiguity is refused."
status: draft
tags: [events, colour, icolup, flavour-groups, proton]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n28-c25, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L3985-L4049", title: "Note 28 C2.5, the pp_to_jj ICOLUP slot defect"}
  - {id: n29-a2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L2205-L2301", title: "Note 29 chain A, A.2 acceptance tests"}
  - {id: n29-a4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L2335-L2372", title: "Note 29 chain A, A.4 risks"}
  - {id: n29-a5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L2373-L2403", title: "Note 29 chain A, A.5 what it cannot break"}
  - {id: n29-b0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L2437-L2491", title: "Note 29 chain A amendment, B.0 the crossing class"}
  - {id: n29-b2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L2512-L2624", title: "Note 29 chain A amendment, B.2 the design"}
  - {id: n29-b3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L2625-L2677", title: "Note 29 chain A amendment, B.3 tests T9–T12"}
  - {id: n29-b5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L2699-L2745", title: "Note 29 chain A amendment, B.5 risks"}
  - {id: n29-c1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L2753-L2770", title: "Note 29 chain A amendment 2, C.1 the self-pairing clause"}
  - {id: n29-c2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L2771-L2794", title: "Note 29 chain A amendment 2, C.2 why it is not a tie-break"}
  - {id: n29-c3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L2795-L2813", title: "Note 29 chain A amendment 2, C.3 measurements"}
---

# Each flavour-group member writes its own colour-flow table

At proton beams one compiled amplitude serves a whole flavour group: every
member shares the representative's matrix element, so the integrand evaluates
the representative and draws the member per event ([events/generate](generate.md)).
Sharing an amplitude, a mass list, a cut filter and a colour basis does **not**
imply sharing colour representations. `g u > g u` and `g u~ > g u~`, `u u > u u`
and `u~ u~ > u~ u~`, `u c > u c` and `u c~ > u c~` all group together in
`p p > j j`, and their `ICOLUP` tables differ. So each member carries its own
table.

## The three classes

| class | example | relation to the representative's table |
|---|---|---|
| identity | `u u > u u` ← `c c > c c` | equal |
| global conjugate | `g u > g u` ← `g u~ > g u~`; `u u > u u` ← `u~ u~ > u~ u~` | a global `ICOLUP` slot exchange |
| crossing | `u c > u c` ← `u c~ > u c~` | **no slot operation relates them** |

The crossing class conjugates two of four legs, and that re-routes the
leading colour line rather than relabelling its endpoints. `u c > u c` has one
diagram with colour factor `T^a_{31} T^a_{42}`, Fierz
`½(δ_{32}δ_{41} − Nc⁻¹ δ_{31}δ_{42})`; conjugating the `c` line gives
`T^a_{31} T^a_{24}`, Fierz `½(δ_{34}δ_{21} − Nc⁻¹ δ_{31}δ_{24})`. The leading
term moves from pairing `{1,4}{2,3}` to `{1,2}{3,4}`, so the leading flow sits at
index 2 for `u c > u c` and index 1 for `u c~ > u c~`. MadGraph's banked
`pp_to_jj` sample confirms it categorically: each of these single-diagram
subprocesses emits only its leading flow (35 and 48 events, one pattern
each)[^n29-b0]. 12 of the 65 dijet assignments are in this class.

`ColorFlowTags::conjugated` (`helas/color/flow_tags.rs:218`) relates a
subprocess to its **full** conjugate only. It survives as an independent test
oracle for the global-conjugate class and is not used to build records.

## The design

1. **Flow fingerprint.** Each compiled subprocess exposes, per flow, the sorted
   contributions summing into its JAMP: `(diagram, chain, Nc power, |q|)`
   (`AmplitudeEvaluator::flow_fingerprints`). The sign and the `i^imag` phase
   are dropped, because charge conjugation can flip a contribution's sign
   (`T^a → −T^{aᵀ}`) without moving which diagram lands on which flow at which
   power of `Nc`[^n29-b2].
2. **Flow permutation `π`.** At group construction, `flow_permutation`
   (`vibegraph-lib/src/proton.rs:1050`) finds the unique bijection
   `rep flow f ↦ member flow π(f)` with equal fingerprints. **Ambiguity is
   refused, never broken**: two equal fingerprints within either basis, no
   bijection, or several is a `ProtonError::ColorFlowPairing` naming the group,
   the subprocesses and the flows. No tie-break, no heuristic, no numeric
   fallback; a wrong `π` would be a silently wrong label on every event of that
   member.
3. **Self-pairing is exempt.** When the member is the same compiled subprocess
   as the group's head (an identity of objects, not of process strings or
   fingerprints), `π` is the identity and the fingerprint is not consulted.
   This matters for `g g > g g`, whose trace-reversal pairs `(0,5) (1,3) (2,4)`
   carry identical contributions and identical JAMP2 — with signs and phases
   retained — so no fingerprint separates them. It is not a tie-break: applied
   to a basis against itself the identity is the only answer. A group whose
   representative basis is degenerate **and** which has a distinct member is
   still refused; `g g > g g` cannot be one, being a single flavour
   assignment[^n29-c1][^n29-c2][^n29-c3].
4. **The member's table, reordered once.** Each member stores
   `member_flows[f] = member.color_flow_tags().flow(π(f))`, in the
   representative's indexing. The configuration draw, the `ICOLAMP` mask and the
   flow draw all stay in the representative's indexing
   ([events/colour-and-helicity-selection](colour-and-helicity-selection.md));
   no downstream consumer sees `π`.
5. **`SubprocessRecord::relabelled(order, pdg, legs, flows)`**
   (`lhef/build.rs:220`) applies only the beam-exchange permutation
   (`ColorFlowTags::permuted`) and then `check_legs` against the member's own
   leg reps, so a table whose occupied slots its legs' reps forbid is refused at
   record construction rather than written[^n29-b2].

### Why the leading-colour mask need not be translated

`reached[d][f]` means "diagram `d` contributes to flow `f` at the basis's
maximal `Nc` power", and `π` preserves `(diagram, chain, Nc power)` by
construction, so `rep.reached_by(d)[f] == member.reached_by(d)[π(f)]`
identically. Masking in the representative's indexing is masking in the
member's. Worked case: `u c > u c`'s mask marks flow 2, `π(2) = 1`, and
`u c~ > u c~`'s own leading flow is 1. This is asserted elementwise (T10 below),
not left as reasoning[^n29-b2].

## The defect this replaced

The record layer once carried the representative's flows to every member.
On `p p > j j` every antiquark leg of a conjugate member then had its colour line
in `ICOLUP(1)`: 4 758 of 80 000 legs, always both antiquark legs of an event;
`p p > l+ l- j` and `p p > b b~` read 0, because their conjugate subprocesses are
separate groups (their `|M|²` differ), and MadGraph's own `pp_to_jj` reads 0. The
first repair, a global slot exchange, was falsified by the crossing class[^n28-c25][^n29-b0].

**Why the net missed it.** `color_flow_tags_oracle` compared the derived table
against `leshouche.inc` only for the first subprocess of each `P*` directory —
the representative, the one member that was right. It now reads every `isproc`
row of every banked directory ([validation/colour-oracles](../validation/colour-oracles.md)).
An oracle on each directory's first subprocess cannot see this defect class at
all[^n28-c25].

## Tests, and what each cannot see

The ladder is deliberate: legality → connectivity against the reference → the
right flow for the right JAMP → the right frequencies. Each is blind to the
next one's failure[^n29-a2][^n29-b3].

| test | asserts | cannot see |
|---|---|---|
| `check_legs` at record construction | no leg's line in a slot its rep forbids (a refusal, so the 4 758-leg mode cannot be written) | a legal but wrong flow |
| `scan_colour_patterns` (`vibegraph-cli/tests/validate_samples_proton.rs`) | every generated `(roles, connectivity)` pattern is one `leshouche.inc` lists — every `isproc`, every flow, both orderings, derived from the Fortran tables rather than MadGraph's sample so rare honest flows are admissible | frequencies |
| widened `color_flow_tags_oracle` (extended-validation) | every derived table equals MadGraph's for every subprocess of every banked directory | what the record layer does with a table |
| T9 `every_member_carries_its_own_subprocesss_colour_flows` (`proton.rs`) | per group, member, ordering: record tables equal the member's own compiled tags under `π`; identity members equal, conjugate members equal `conjugated()`, crossing members neither; anti-vacuity: each class present, both identity and non-identity `π` among rep-changing members | an error shared by both compilations (the oracle above excludes it) |
| T10 `the_flow_permutation_carries_the_leading_colour_mask` | `rep.reached_by(d)[f] == member.reached_by(d)[π(f)]`, with some non-trivial mask row and non-identity `π` | whether the mask is MadGraph's |
| T11 `the_exchanged_ordering_is_a_leg_permutation_of_the_direct_one` | the exchanged record's tags are the direct one's under the beam swap, without compiling a swapped process string | whether the direct ordering is right |
| T12 `the_flow_fingerprint_identifies_a_flow_uniquely` | fingerprints pairwise distinct where a basis is paired with another; `JAMP2_rep[f] = JAMP2_member[π(f)]` to 1e-11 outside degenerate blocks | a unique fingerprint matching the wrong pairs (T9/T10) |
| `ICOLUP` χ² of the `pp_to_jj` samples cell | frequencies against MadGraph, 3 seeds, p-floor 1e-4 | errors that preserve colour-key frequencies |

Measured when the design landed: 238 tables over the three classes 39/14/12,
`check_legs` 238/238, T10 on 112 rows with 95 restricting, and the dijet
`ICOLUP` χ² p 0.105/0.263/0.140 against p ≈ 0 before[^n29-c3]. The cell is
gated ([validation/samples-gate](../validation/samples-gate.md)).

## What it cannot move, and what stays assumed

No cross section, kinematics, flavour, helicity, mass, status or mother can
move: the flow is selected after acceptance, and `π` and the tables are built
once at group construction. Identity-class groups are bit-identical, which is
every gated row except `pp_to_jj`[^n29-a5][^n29-b5].

The **helicity** correspondence across a conjugate member is assumed, not
proved: a member's event takes a helicity drawn off the representative's
per-helicity `|M|²`. The dijet `SPINUP` column clearing its floor (p 0.18–0.35)
is evidence, not proof[^n29-b5]. The group-formation checks (`n_flows`, the CF
matrix) cannot see a flow permutation either: for `g q > g q` the 2×2 CF matrix
is invariant under the reversal, which is why T9/T10 exist[^n29-a4]. Colour-flow
conventions under charge conjugation in general are
[amplitudes/colour-flow-lines-and-conjugation](../amplitudes/colour-flow-lines-and-conjugation.md).

[^n28-c25]: Note 28 C2.5: the 4 758-leg defect, its origin in `relabelled`, and why the oracle missed it.
[^n29-a2]: Note 29 A.2, the instrument ladder and tests carried into B.3.
[^n29-a4]: Note 29 A.4, the CF-matrix check cannot see `π`.
[^n29-a5]: Note 29 A.5.
[^n29-b0]: Note 29 B.0, the crossing class and MadGraph's categorical confirmation.
[^n29-b2]: Note 29 B.2, the fingerprint, `π`, the per-member tables and the mask argument.
[^n29-b3]: Note 29 B.3, T9–T12.
[^n29-b5]: Note 29 B.5, residual risks.
[^n29-c1]: Note 29 C.1, the self-pairing clause.
[^n29-c2]: Note 29 C.2, why self-pairing is not a tie-break.
[^n29-c3]: Note 29 C.3, the correction and the measurements.
