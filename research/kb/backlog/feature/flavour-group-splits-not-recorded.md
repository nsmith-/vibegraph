---
type: Backlog Item
title: "A run does not record its flavour groups: their members, their parts, or the splits that made them"
description: "The ldjson names no group's subprocesses or part, its channel-key group index does not line up with result.multiplicities, and a near-degenerate split (#103) leaves no record of the pair."
area: feature
state: open
priority: low
closes_when: "The ldjson result carries one object per flavour group (its index as the channel keys use it, its part and multiplicity line, and its member subprocesses), and for each split decided by |M|² separation the two subprocesses and their best-probe separation (also in the log), with a test on a card that splits by mass and by separation."
blocked_by: []
opened: 2026-11-07
tags: [flavour-groups, logging, ldjson, ckm, cms, diagnosis]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
sources:
  - {id: log, resource: "../../sprints/cms-gridpack-coverage/log.md", title: "Sprint H log: #103, the closing campaign, and VBF_HH's unnamed groups (2026-11-07)"}
  - {id: hn1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-N1-report.md", title: "H-N1 Found 5: the CERN host should record each split"}
---
H-N1 asked the closing campaign to record, for the survey's CKM cards, the pair
and separation each now splits on, because 3.88e-8 was unattributed. The CERN
host reports that the 3.88e-8 card (089) was not in the campaign's sample, and
that the split is not logged anywhere: a run does not say that a near-degenerate
pair was kept apart.

Recording it would have served
[ckm-flavour-groups-near-degenerate-refused](../validation/ckm-flavour-groups-near-degenerate-refused.md)
directly, and is the first thing to read on any CKM card whose σ disagrees with
MadGraph after a split.

The VBF_HH diagnosis hit the wider gap (2026-11-07 log). Per-channel spikes in
groups 20, 11, 22 and 6 could not be named from campaign output. The ldjson
channel key's `group` does not line up with `result.multiplicities`, and no
map from group to subprocesses or part is written. That key counts a
multiplicity's groups across its mass parts (`ChannelKey::MultiplicityChannel`,
`artifact.rs`). The mismatch is the CERN host's reading. Naming the groups took
a scratch instrumented build. One per-group record serves both needs: the
split pair is an attribute of the groups it produced.
