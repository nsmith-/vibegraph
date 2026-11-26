---
type: Backlog Item
title: "The one-line result record embeds every configuration, so large cards emit a multi-megabyte JSON line"
description: "Card 51's result record lists 62080 configurations, about 7 MB on one line. A 10 MiB stdout cap on batch cut it, and the collector read the job as having no σ."
area: feature
state: open
priority: medium
closes_when: "The result record's size is bounded independently of the configuration count: per-configuration rows move to the artifact or to inspect, or are summarised. A test on a many-configuration card shows the record under a fixed size, with σ, error and the per-multiplicity breakdown intact."
blocked_by: []
opened: 2026-11-26
tags: [output, result-record, ldjson, batch, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-26}
---
Found by `vg-validator` in the pre-tag stage B. The jobs for cards 51
(WBtoLNuB-4Jets) and 14 (WWjj_SS_ewk) exited 0, but their result lines were
truncated by the batch proxy's 10 MiB stdout cap. The collector now falls back to
the last `iteration N` log line. It agrees with the record to 3e-7 on three
complete jobs. Card 51 was a v0.2.2 timeout, so the truncation would have hidden
the σ it now reaches.
