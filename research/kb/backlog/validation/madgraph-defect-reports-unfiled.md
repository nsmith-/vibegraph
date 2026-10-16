---
type: Backlog Item
title: MadGraph defect reports are drafted but not filed upstream
description: R1's grouped first-call defect and the MLM study's other MadGraph defects need finalising and filing upstream; the user files them.
area: validation
state: needs-user
priority: low
closes_when: Each MadGraph defect report drafted in note 07 from the MLM study is filed upstream and its link recorded in note 07.
blocked_by: [mlm-madgraph-defects-undrafted]
opened: 2026-09-29
tags: [mlm, madgraph-defect, upstream-report, h1]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L229-L241", title: "TODO.md entry T005"}
---
Ready to finalise: `super_auto_dsig_group_v4.inc`'s first `setclscales` call
on the unpermuted `PP` (H1, note 41 R1). [Note 07](../../history/notes/07-mg5-code-quality.md) ("Direct Bug Found
(grouped MadEvent scales and rejects the unpermuted point)") holds the draft.
The one-line patch is `validation/madgraph/patches/first-call-unpermuted-momenta.patch`
and the `u q > z u q` reproducer is in `validation/madgraph/repro/permuted_first_call/`.

Still to draft: note 41 §1.5's six defects
([mlm-madgraph-defects-undrafted](mlm-madgraph-defects-undrafted.md)).

The older drafts (`FFV2P1D_1`, the `AQCDUP` π truncation) are tracked with the
`process-grammar` findings and in note 07. The CDATA / `setMad` interplay is
tracked separately as
[pythia-setmad-drops-cdata-run-card](pythia-setmad-drops-cdata-run-card.md).

Candidate added 2026-10-15: MadEvent's default survey under-integrates by
0.5–1 % on some processes, and `hard_survey` is silently ignored when `mmjj`
exceeds 1 % of the beam-energy sum
([default survey bias](../../validation/madevent-default-survey-bias.md)).
The same candidate covers the difficult-mode `run_config.inc` substitution,
whose pattern does not match its own output, so a reused process directory
keeps its first survey rate.
