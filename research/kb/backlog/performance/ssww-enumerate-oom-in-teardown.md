---
type: Backlog Item
title: "Enumerating the VBS ssWW card takes 65 min and is OOM-killed in teardown after writing its artifact, with no result record"
description: "Card 703 (p p > j j l+ l+ vl vl, QED=6) enumerates 354,528 diagrams in 864 subprocesses in 3,880 s, then exits rc=137 at ≥ 4.2 GB about 20 s later; the 1200 s campaign cap killed it at about 30%."
area: performance
state: open
priority: medium
closes_when: "Enumerate on card 703 emits its summary/result record before teardown, and teardown does not raise peak RSS above the enumeration's own peak. Peak RSS and wall are measured and recorded for one ssWW card. The 14 ssWW cards (703–709, 760–766) have a stated enumeration budget, and either fit the campaign cap or the campaign gives them their own."
blocked_by: []
opened: 2026-11-25
tags: [enumerate, memory, teardown, vbs, cms, timeout]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-25}
---
Sprint L's validator ran this uncapped on bin-pr1 9f196713 with MemoryMax 4500M.
The process is `p p > j j l+ l+ vl vl`, QED=6, QCD=0, EFT `^2==1/2`:
- 354,528 diagrams in 864 subprocesses (144 compiled sets), written after
  3,880 s.
- Then rc=137, about 20 s after the last log line. Peak RSS was ≥ 4.21 GB by 20 s
  polls, so this points to an OOM kill while dropping the enumeration state.
- No summary or result record was emitted, so a caller sees only a killed
  process, even though `diagrams.bin.zst` (1.9 MB) and `diagrams.json` (327 MB)
  were complete.

Two problems:
1. The result record should be written before teardown. A completed artifact
   should never be reported as a crash.
2. Teardown should not need more memory than the enumeration itself.

The 14 ssWW cards were first measured on PR 1's request A. Before that they were
refused at parse for squared orders. The 1200 s cap killed them at about 30% of
their enumeration cost.
