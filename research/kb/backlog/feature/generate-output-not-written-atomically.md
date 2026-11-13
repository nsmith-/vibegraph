---
type: Backlog Item
title: "A killed generate leaves a truncated LHE under its final name"
description: "generate writes the LHE in place; a SIGTERM 6 s in left 14,336 events ending mid-<event> with no closing tag, under the requested name, so a killed batch shard looks finished."
area: feature
state: open
priority: medium
closes_when: "generate writes to a temporary file in the output directory and renames it to the requested name only after the closing tag is written; a CLI test kills a run and finds no file under the final name."
blocked_by: []
opened: 2026-11-13
tags: [cli, batch, generate, lhef]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-13}
---
Reported by the batch-proxy design session: `generate --nevents 20000 -o
evkill.lhe`, killed with SIGTERM 6 s in, left an 89.6 MB `evkill.lhe` holding
14,336 events, ending mid-`<event>` with no `</LesHouchesEvents>`. A batch
system that checks for the output file treats that shard as done.

Write to a temporary name in the same directory and rename on success, which
is atomic on one filesystem. Keeping a partial file under the final name is not
needed: the batch side reruns killed shards. A deliberate truncated sample is
a separate decision, recorded in
[`generate-has-no-graceful-stop`](generate-has-no-graceful-stop.md), and must
mark itself truncated in the header.
