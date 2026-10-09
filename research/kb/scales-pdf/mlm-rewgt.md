---
type: Algorithm
title: "rewgt: α_s and PDF reweighting under ickkw = 1"
description: "MadEvent's rewgt factor per flavour combination: vertex classes, the PDF-ratio chain and its kills, goodjet, and vibegraph's coupling/cluster/rewgt.rs with per-member wiring."
status: draft
tags: [mlm, rewgt, alpha-s, pdf, reweighting]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n41-13, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L105-L145", title: "Note 41 §1.3 (rewgt under ickkw = 1)"}
  - {id: n41-32, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L235-L252", title: "Note 41 §3.2 (the reweighting is a per-term factor)"}
  - {id: n41-m0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L303-L530", title: "Note 41 M0 (references, dump records, censuses)"}
  - {id: n41-m2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L758-L1013", title: "Note 41 M2 (rewgt: implementation and dump gates)"}
  - {id: rewgt-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/coupling/cluster/rewgt.rs#L1-L40", title: "coupling/cluster/rewgt.rs module documentation"}
  - {id: mg-reweight, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/reweight.f#L1333-L1824", title: "MadGraph reweight.f rewgt"}
  - {id: mg-auto-dsig, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/template_files/auto_dsig_v4.inc#L141-L151", title: "MadGraph auto_dsig_v4.inc (IPSEL draw)"}
---

# `rewgt`: α_s and PDF reweighting under `ickkw = 1`

A matched sample evaluates its matrix element at one renormalisation scale and its
densities at one lowered factorisation scale ([scales-pdf/mlm-scales](mlm-scales.md)),
then corrects both toward what a shower would have used, vertex by vertex along the
clustering. That correction is MadEvent's `rewgt` (`reweight.f:1333-1824`), and
vibegraph's port is `rewgt` in `vibegraph-lib/src/coupling/cluster/rewgt.rs`.[^mg-reweight][^rewgt-rs]
Where it sits in MLM as a whole: [scales-pdf/mlm-matching](mlm-matching.md).

`rewgt = 1` whenever `ickkw ≤ 0`: it returns at once unless `use_syst` (`:1421`), and with
`use_syst` it only records the systematics inputs and jumps past both factors
(`:1450-1461`). Under `ickkw = 1` it is the product of an `α_s` factor and a PDF-ratio
factor. vibegraph's port returns a unit `Rewgt` at any `ickkw ≤ 0`.

## The `α_s` factor (`:1557-1616`)

Over clusterings `n < nexternal − 2`; the `2 → 2` core vertex is never reweighted.[^n41-13]

- A vertex **qualifies** if either
  - **ISR**: the mother is on a beam line and `goodjet(mother)` holds, where
    `goodjet(mother) = isparton(mother) ∧ goodjet(d1) ∧ goodjet(d2)`; or
  - **FSR**: the mother is final-state, `ispartonvx` holds, and at least one daughter is
    `goodjet`;

  and in both cases the mother's PDG is not `fake_id`.
- On a qualifying vertex, with `q2now = pt2ijcl(n)`: `q2now ≤ 4 GeV²` **kills** the event;
  otherwise `rewgt *= αs(alpsfact·√q2now) / αs(μR)` (`:1597-1601`), where `μR` is the
  first `setclscales` call's.
- `isparton` is `|id| ≤ max(asrwgtflavor, maxjetflavor)` or a gluon. The reference cards
  carry the hidden default `asrwgtflavor = 5`, so `isparton` reads 5 there.
- `use_syst = T` forces `alpsfact = 1` (`setrun.f:151-159`), so a card testing `alpsfact`
  must switch systematics off ([run-card/matching-parameters](../run-card/matching-parameters.md)).

## The PDF-ratio factor (`:1617-1748`)

Applied when `ickkw > 0` and `pdfwgt` (on by default). Each beam's line is walked up
through its initial-state clusterings:[^n41-13][^n41-m2]

- `x ← x·z_n` **before** the evaluation, and only when `0 < z < 1` (the core's `z` is 1).
- `q2now = min(pt2ijcl(n), q2bck(j))`, and exactly `q2bck(j)` at `jlast`.
- The **first** initial-state vertex takes no ratio: the matrix element's density was
  already read there.
- Each later vertex with `n ≤ jlast` and a **rising** scale multiplies by
  `f_fl(x, q_now) / f_fl(x, q_prev)`, where `fl` is the flavour of the line **entering**
  the vertex. A step that does not rise copies the scale
  (`pt2pdf(mother) = pt2pdf(daughter)`).
- **The chain includes the core.** The `α_s` restriction to `n < nexternal − 2` does not
  apply here, and the ratio at `jlast` is usually taken at the core.
- A denominator below `1e-10` **kills** the event. The floor is on `f`, not `x·f`:
  `pdg2pdf` returns `x·f/x`.
- A rising step past `jlast` sets nothing. It is unreachable from a consistent walk,
  because the chain's scale is `q2bck` from `jlast` on (pinned by `past_jlast_no_ratio_is_taken`; never
  reached on any reference row).

`s_xpdf` stores `x` *before* `z` (`:1719-1723`), and `systematics.py:1022-1030` telescopes
differently; neither is a specification of this weight.

## Which codes `rewgt` reads

- **The weight is per flavour combination, not per group.** `DSIG` draws one
  `IPSEL ∝ PD(IPSEL)` (`auto_dsig_v4.inc:141-151`), `rewgt` reads that combination's codes,
  and the product multiplies the whole `PD(0)·|M|²`.[^mg-auto-dsig]
- The incoming legs and the final-state **jets** take the drawn combination's codes
  (`:1531-1537`). Every other line keeps `ipdgcl` as the scale walk left it, and `rewgt`
  re-runs `ipartupdate` over every merge, the core included (`:1577`), which re-transmits
  jet flavours onto the spacelike lines. A non-jet quark transmits nothing: a `b` line at
  `maxjetflavor = 4` keeps the forest's `|tprid|`, and its density is read for `+5`
  whichever way the line runs.
- **`goodjet` on external legs** (`:1538-1549`): a beam is a parton line if `isparton`; a
  final-state leg if `iqjets > 0`, or if it is a parton that is not a jet. A jet-flavour
  leg the walk did not tag is not a parton line.
- `fake_id` never occurs: the forests never split a higher vertex
  (`no_forest_line_carries_a_code_outside_the_model`). An `ipartupdate` failure
  (`stop 3`) refuses the event with an error.

**The stale-table deviation.** MadEvent's `ipdgcl` is a common block carried across
events. A final-state leg that is not a jet keeps whatever code the table last held, and
`setclscales` reads codes an earlier `rewgt` left. `RewgtHistory::pdg` is this event's
walk over the group representative's codes, which is MadEvent's first event of a run.
The two can differ only where a group's members differ in jet-ness on a leg, or in which
vertices transmit a flavour. The census found no `IPROC` mixing jet and non-jet flavours
on any reference row, and no final-state leg whose `rewgt` code differs from its `idup`,
so the class is empty there.[^n41-m0]

## vibegraph's port

`rewgt(history, colors, settings, flavours, x, αs, x·f) → Rewgt` is a pure function of
the clustering history (`RewgtHistory`: the second call's merges with `zcl`, `pt2ijcl`,
`jlast`, `iqjets`, line codes, `q2bck`, the first call's `μR`, the momenta), the run-card
constants, the `αs` evaluator and the PDF.[^n41-32] `Rewgt` lists every factor beside the
product:

- per vertex: its codes after `ipartupdate`, `ipart(1, mother)`, and a class (`Core`,
  `Isr`, `Fsr`, or why not: `IsrNotParton`, `FsrNotPartonVertex`, `FsrNoPartonDaughter`),
  with its `αs` ratio (`q2`, numerator, `αs(μR)`);
- per beam, every chain step: `n`, the entering line and its flavour, `x` after `z`,
  `q_prev`, `q_now`, and `First` / `Ratio` / `NoRise` / `PastLast`;
- the product, and a kill (`AlphaSScale`, `PdfDenominator`).

The product equals the listed factors bit for bit (`Rewgt::product_of_factors`). The list exists
so the oracle compares at the finest linear level: a product alone cannot tell a missing
vertex from a compensating PDF ratio ([AGENTS.md](../../../AGENTS.md), "Physics
Validation").

**Wiring.** The factor applies per term: per flavour group, per beam ordering **and per
member**, on the per-term scale path, which matching always takes.
`EventScaleSource::point_history` returns the scales and the `RewgtHistory` of the same
two calls; `per_group_sum` computes each member's factor per ordering and weights each
member's luminosity by it (`term_luminosity`), and the event selection draws
`(member, ordering)` with the same factors. Summing every combination with its own factor
has the expectation of MadEvent's `IPSEL` draw. The clustering's beam 1 takes the `x` of
the physical beam its leg 1 arrives on, which is the order the mirrored term's
`[x₂, x₁]` assumes; the dump gate checks it on every event. The integrand side is
[hadronic/proton-integrand](../hadronic/proton-integrand.md).[^n41-m2]

## Validation, and what each check can see

- **Factor by factor against MadEvent's dump** (`validate_mlm_dumps`, tolerance 1e-12):
  per vertex the class, lines, codes, `kt²`, `αs(alpsfact·kt)` and ratio; `asref` and
  `jlast`; per beam each chain step's vertex, flavour, action, `x`, both `q²`, both
  densities and the ratio; the kill; the product. Every gated event of every MLM row
  agrees, worst 7.9e-15 on a scale and 2.9e-15 on a product. `pp_to_ll_0j2j_mlm` carries
  the suite's only FSR vertices. No row has a kill or a `NONE` chain step, so those
  branches are pinned by unit tests only ([validation/mlm-dump-oracle](../validation/mlm-dump-oracle.md)).
- **`alpsfact`.** `pp_to_llj_mlm_alps2` (`alpsfact = 2`, `use_syst = F`) pins where it
  enters: the dump shows the numerator read at `2·kt` on every reweighted vertex. A test
  that set `alpsfact` but compared only `alpsfact = 1` events could not see it.
- **Negative control.** Dropping the `α_s` ratios scales σ by `⟨1/A⟩` over MadEvent's
  unweighted events, with `A` an event's product of `α_s` ratios; vibegraph's ratios and
  MadEvent's give the same `⟨1/A⟩` to six digits. The effect is −14.5% on `pp_to_llj_mlm`,
  −4.8% on `_alps2`, −6.6% on `pp_to_ll_0j2j_mlm` and −9.5% on `pp_to_ttx_0j1j_mlm`, each
  far outside the σ gate's tolerance (asserted > 1%).
- **Cross sections** are [validation/mlm-sigma-gate](../validation/mlm-sigma-gate.md); no
  σ is quoted here. The `pp_to_llj_mlm` reference is the `refdata-9` value that gate reads.

The factor has a heavy tail (mean 1.357, range 1.000–2.092 on `pp_to_llj_mlm`), which
inflates per-seed error estimates on matched rows: read matched σ over a seed sweep and
its χ²/dof, not one seed ([validation/seed-sweeps-and-budget-ladders](../validation/seed-sweeps-and-budget-ladders.md)).[^n41-m0][^n41-m2]

[^n41-13]: Note 41 §1.3, `rewgt` read from the source.
[^n41-32]: Note 41 §3.2, the reweighting as a pure per-term factor with its factor list.
[^n41-m0]: Note 41 M0: `asrwgtflavor`, the dump's `RW*` records, the census, vertex-class counts.
[^n41-m2]: Note 41 M2: implementation, the corrections to §1.3, wiring, dump gates and the negative control.
[^rewgt-rs]: `vibegraph-lib/src/coupling/cluster/rewgt.rs` module documentation.
[^mg-reweight]: MadGraph `reweight.f:1333-1824`.
[^mg-auto-dsig]: MadGraph `auto_dsig_v4.inc:141-151`, the flavour-combination draw.
