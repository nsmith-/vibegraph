---
type: Sprint Record
title: "MadGraph's HT cuts compiled"
description: "htjmin, htjmax, ihtmin and ihtmax are compiled per cuts.f (htj over the jet class, iht adding the b class, no do_cuts gate); two HT-binned rows gate against MadGraph at pulls −0.35 and −0.05."
closed: 2026-10-26
status: draft
tags: [cuts, ht, runcard, cms, sprint-d, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: f1, resource: "sessions/F1-report.md", title: "F1 report"}
  - {id: v1, resource: "sessions/V1-report.md", title: "V1 report"}
  - {id: hy1, resource: "sessions/HY1-report.md", title: "HY1 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/74", title: "nsmith-/vibegraph#74"}
---
One PR ([nsmith-/vibegraph#74](https://github.com/nsmith-/vibegraph/pull/74),
merged 2026-10-26) closing `ht-cuts-unimplemented`. The four cuts stopped every
HT-binned γ+jets, QCD and V+jets card in the CMS survey (12 of 65 LO cards set
`htjmin`, 8 set `ihtmin`). Sessions: [F1](sessions/F1-report.md) (feature-dev,
Opus), [V1](sessions/V1-report.md) (validation-dev, Sonnet),
[HY1](sessions/HY1-report.md) (hygiene-dev, Opus). The M3 banked both rows.

## What was established

- **MadGraph's definitions** (`cuts.f` at `b7687064`). `htj` is the scalar pT sum
  of the jet-class legs (gluons and quarks with |pdg| ≤ `maxjetflavor`,
  `:521-529`, `:876-892`); `iht` adds the b class (`:898-910`); leptons and
  photons enter neither. No `do_cuts` gate and no pT or η preselection. A
  maximum is read only when ≥ 0, a minimum only when > 0. *Manager-verified:*
  `cuts.f:521-534` and `:888-910` read; the `cuts::` tests (65) pass.
- **Per-event tests.** Each threshold edge is pinned; counting the b class in
  `htj`, or ignoring `maxjetflavor`, fails them (F1's and HY1's mutations).
- **σ against a twin, before the bank.** With two partons an HT window is a pT
  window, so each card has a pT twin over the same phase space (F1, five or
  fifteen seeds): jj −0.054 %, pull −0.49; bb within 0.1 % at both budgets. The
  HT-only `p p > b b~` card samples worse: χ²/dof 3.76 at 4× where its twin reads
  0.62, its quoted error about half its spread. An HT cut gives the channels no
  regulator.
- **The gate** (banked on the M3; quoted from #74, not reproduced here), five
  seeds a side, |pull| < 3, |rel| < 0.005, seeds' χ²/dof < 4:

  | row | vibegraph | MadGraph | pull |
  |---|---|---|---|
  | `pp_to_jj_ht` | 213424 ± 201 pb | 213516 ± 176 pb | −0.35 |
  | `pp_to_bb_iht` | 4844.8 ± 4.7 pb | 4845.0 ± 1.6 pb | −0.05 |

  The first MadGraph reading of `pp_to_bb_iht` showed a +4.41σ difference
  between 1× and 4× on 5 + 2 seeds. A 20 + 20 ladder read +0.40σ ("stays at
  1×"), so the 5 + 2 difference was noise and the 20-seed 1× record is the
  reference. The ladder's SCATTER line flagged `P1_gg_bbx/G1` at χ²/dof 555 on
  19 dof, carrying 0.0 % of σ; the row's note records it, cause unknown.
- **Two red tests at the branch tip, fixed by HY1.** Unplanning the rows
  (`ed5d8ea`) left the HT gates failing on a checkout without the records, and
  the collator's planned-row test asserting a state the manifest no longer had.

## Backlog

- Deleted: `ht-cuts-unimplemented`.
- Drafted for filing at close-out: `ht-cuts-give-channels-no-regulator`
  (performance; F1 Found 1, V1 Found 3), `ht-sigma-unchecked-beyond-two-partons`
  (F1 Found 3), `ht-rows-have-no-vibegraph-budget-ladder` (V1 Found 2),
  `cut-threshold-rule-spelled-inline` (HY1 Found 1),
  `absent-run-escapes-spelled-per-gate` (HY1 Found 3, V1 Found 1),
  `pp-to-bb-iht-gg-channel-scatter` (#74 comments).
- Not filed:
  - F1 Found 2 (the σ gate not yet run): run and gating;
  - F1 Found 4 (`ht2min`…`ht4max` and `ptj1min`…`ptj4max` stay refused): no
    surveyed card sets them active, and they are refused before enumeration
    with a message naming them;
  - HY1 Found 2 (two stale kb sites): both concepts now say the cuts are
    compiled;
  - HY1 Found 4 (`planned_rows()` without a caller): it has callers again since
    the merges (`validate_hadronic.rs`, `validate_samples_proton.rs`).

## Process notes

The 20 + 20 ladder settled what a 5 + 2 sweep could not, as the seed policy
predicts. The manifest state change that unplanned the rows broke tests on two
PRs at once (here and #72); the absent-run consolidation item is the response.
