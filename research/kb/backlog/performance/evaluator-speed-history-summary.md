---
type: Backlog Item
title: No summary of evaluator speed over the project's history
description: "For a presentation on how the project was built, the evaluator's speed across its history should be summarised from recorded numbers, approximately, without rerunning old versions."
area: performance
state: open
priority: low
closes_when: "A short dated table or chart of evaluator speed per milestone exists, each figure citing the note or commit it came from, with host differences marked."
blocked_by: []
opened: 2026-10-09
tags: [presentation, history, evaluator, measurement]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: user, resource: "https://claude.ai/artifact/N5Rm9hTa3rpkQ3duboK5UB", title: "User comment on the taxonomy review, 2026-10-09"}
---
A one-off investigation into the history, not a standing concept. The
migration leaves superseded timings in the archived notes, so the numbers
are there to collect: note 15's evaluator program (8.6×–110× → 1.2×–3.5×
against MadGraph), note 20's eval-perf-2 speedups, note 31 §6 and note 32 §5,
the lane and FMA studies, the top-down and roofline studies, and the weighted
JAMP sums.

Approximate is fine; do not rerun old versions. Mark each figure's host and
metric (ns per event, ratio to MadGraph's MATRIX1, per-process or median),
since they are not directly comparable across hosts. The output belongs with
the presentation material, not in the knowledge bundle.
