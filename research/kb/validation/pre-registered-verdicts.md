---
type: Validation Methodology
title: Pre-registered verdict tables and may-move sets
description: "Before measuring, fix the verdict table, the cells allowed to move and the falsifier; then read the measured change against that set, and stop on anything outside it."
status: draft
tags: [validation, methodology, pre-registration, may-move, verdict]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n29-f2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L438-L491", title: "Note 29 F.2, the pre-registered bar"}
  - {id: n29-f5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L606-L663", title: "Note 29 F.5, vacuity modes"}
  - {id: n29-d6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L3114-L3138", title: "Note 29 D.6, the pre-registered decision rule"}
  - {id: n29-b2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L4826-L4886", title: "Note 29 B.2, the movement census"}
  - {id: n29-b6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L5102-L5167", title: "Note 29 B.6-B.7, before/after comparison and stages"}
  - {id: n29-b0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L5353-L5522", title: "Note 29 B-0 output, the census measured"}
  - {id: n36-b3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36-banked-open-ends-plan.md#L567-L650", title: "Note 36 B3, a may-move set wrong twice"}
  - {id: n40, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/40-per-group-dynamic-scales.md#L1-L110", title: "Note 40, per-group dynamic scales"}
---
# Pre-registered verdict tables and may-move sets

A measurement whose reading is decided after the numbers are in can be made to
say anything. So before measuring, write down three things: what each possible
outcome will be taken to mean (the verdict table), which cells the change is
allowed to move (the may-move set), and the test that would show the hypothesis
false (the falsifier). Then measure, and read the result against what was
written. A cell that moves outside the set, or an outcome no row of the table
covers, is stop-and-report: never retune, re-seed or widen a tolerance to absorb
it. This is how validation work exposes rather than fixes
([workflow/expose-dont-fix](../workflow/expose-dont-fix.md)), and the instruments
for reading "what moved" are in [no-change-claims](no-change-claims.md), over the
per-cell report the collator renders ([validation-report](validation-report.md)).

## The verdict table

Rows are read top to bottom and the **first** whose pattern holds is the verdict;
no row may be reached by reasoning about which side "ought" to be right. A
catch-all last row says "no verdict: report what was measured, which pattern was
expected and which was seen, and escalate"[^n29-d6].

The worked case is the `ee_to_mumua` drift (σ about +0.8% above the bank). Its
table had ten rows over two closure tests, one per side: does MadGraph's
partition of its own phase space into windows sum to its own unwindowed σ
(`C_MG`), and does ours (`C_VG`)? Then: is the disagreement localised in one
window, is our side stable over seeds and budget, and do the two MadGraph
versions agree. Examples of its rows:

| pattern | verdict |
|---|---|
| `C_MG` fails, `C_VG` holds, our side stable | the reference owns it; tolerances stay, and the record says the reference moved, not that a tolerance was loosened |
| `C_VG` fails, `C_MG` holds | we own it; file a defect naming the windows |
| both closures hold, localised, our `Δ_w` migrates between seeds or does not shrink with 4× budget | we own it, localised: a bug, per `AGENTS.md` |
| both hold, localised, both sides stable, versions agree | localised but unattributed; escalate with the window named |
| anything else | no verdict |

Standing riders were written in too: if a premise measurement contradicts the
design, stop before the main runs; if one MadGraph version cannot run, the rows
that need it become unreachable and route to named fallbacks, recorded as a
degraded measurement. The outcome was the first row: MadGraph's own windowed
partition in `pt(γ)` exceeded its unwindowed σ (+3.07σ on quoted errors, +2.82σ
on seed spreads, both readings recorded). A secondary `m(μμ)` axis, registered
beforehand to carry no verdict, then showed the same signature five times larger,
and it stayed localisation evidence only. The part still unattributed is
[ee-mumua-radiative-return-sigma-high](../backlog/validation/ee-mumua-radiative-return-sigma-high.md).

## A bar for "success" that counts its own freedom

When the claim is that a rule derives conventions currently set by hand (here,
diagram phases from charge flow), the bar has to price the rule's own
choices[^n29-f2]:
- The rule is written in full **before** its first comparison and may not be
  amended after a mismatch; an amended rule is a new rule, and the number of
  amendments is reported with the verdict.
- With `k` independent binary choices the author picked and `n` pinned binary
  conventions reproduced, only `n − k` is evidence.
- A negative result takes a required form: one witness pair, two configurations
  agreeing on every input the rule may read yet carrying different pinned signs.
- Every reproduced value is checked against the MadGraph dumps, never against
  the current code; a row checked against neither is "unchecked", not green.
- A cheap prediction expected to fail is registered first (`g g > g g` has no
  fermion line, so the strong reading predicts no fermion sign anywhere on it),
  so the hypothesis cannot be quietly reinterpreted afterwards.

Ways such a success would be vacuous, written down before measuring[^n29-f5]:
fitting the same dumps twice (several pinned conventions were themselves fitted to
those dumps), deriving a quantity no observable depends on (|M|² and JAMP² are
blind to a common phase), and confirming against the code instead of the
reference (every gate is green, so a rule reverse-engineered from the code
agrees with it).

## The may-move set

Derive the set mechanically, before implementing, from what the change can
reach. For a change to how the clustering configuration is drawn, the criterion
was: the scale is the only path by which the draw reaches σ, so a row whose μR and
both μF are the same number in every configuration at every point cannot move.
A probe measured the worst spread of μR and μF over all configurations at 64
cut-passing points per row, and the result was reported before implementation
was dispatched[^n29-b2][^n29-b0]:
- rows with zero spread, or with no per-event prescription compiled, or with
  fully fixed scales: **must be bit-identical**, byte-identical JSON, not "inside
  tolerance";
- rows with non-zero spread: **may move**, and must still pass;
- the hand-written prediction was corrected by the measurement (nine rows the
  card-level criterion called clustered compiled no prescription at all at the
  time), which is why the census is measured rather than written from the card.

Then stage the work so each stage has a predicted state: plumbing wired with its
output discarded must be bit-identical everywhere (the negative control for the
stream discipline); the live change may move only the may-move set; and a
known-wrong informational probe that should collapse when the feature works runs
at every stage as the end-to-end signal[^n29-b6].

**The measurement stands; the conclusion did not.** The same census found that a
group axis reached no scale the configuration axis inside the sampled group did
not already reach, and the per-group variant was not built. A later per-event
oracle showed events taking another flavour group's scale (79 of 2000 on
`p p > l+ l- j`, 621 of 2000 on decayed `p p > t t~`), with two wrong classes
compensating in the `SCALUP` marginal the σ and samples gates read[^n40]. A range
measured at 64 points says nothing about which scale each event is labelled with;
the fix is [scales-pdf/configuration-draw-sigma-shifts](../scales-pdf/configuration-draw-sigma-shifts.md).

## A may-move set written by hand was wrong twice

Before moving the integrator onto MadGraph's merged channel set, a nine-row
may-move set had been written from a reading of which rows merge. Measured, three
of the nine were 2 → 1 rows with no channels at all, and the set missed six gated
σ rows that merge and three that lose a contact channel; the real set was 16
σ/samples rows plus 3 amplitude-only. Five seeds per row outside the corrected set
were then identical digit for digit[^n36-b3]. Build the set from a machine census
of the code (`config_groups` over every row), as `AGENTS.md` asks for census
claims, and treat a hand-written set as a prediction to check.

[^n29-d6]: Note 29 D.6.
[^n29-f2]: Note 29 F.2.
[^n29-f5]: Note 29 F.5.
[^n29-b2]: Note 29 B.2, the pre-registration.
[^n29-b0]: Note 29 B-0 output, the census table and its correction.
[^n29-b6]: Note 29 B.6–B.7.
[^n40]: Note 40 §1 and §3, the per-event oracles.
[^n36-b3]: Note 36 B3, "Brief correction".
