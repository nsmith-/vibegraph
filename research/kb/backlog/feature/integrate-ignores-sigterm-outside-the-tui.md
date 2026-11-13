---
type: Backlog Item
title: "integrate ignores SIGTERM and SIGINT outside the TUI, so a batch kill loses the run"
description: "In plain and ldjson modes a SIGTERM or SIGINT kills integrate at once: no artifact, and the ldjson stream ends mid-progress. HTCondor sends one SIGTERM about 10 minutes before SIGKILL."
area: feature
state: open
priority: high
closes_when: "Outside the TUI, the first SIGTERM or SIGINT raises the StopSignal (finish the iteration, bank the artifact, emit the result) and a second takes the default action; a CLI test sends each signal to a plain and an ldjson run and checks the artifact and final object."
blocked_by: []
opened: 2026-11-13
tags: [cli, batch, signals, ldjson, integrate]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-13}
---
Reported by the batch-proxy design session running the v0.2.1 musl-v3 release
binary as CERN HTCondor jobs. `condor_rm` and runtime-limit kills send one
SIGTERM, then SIGKILL about 10 minutes later. Batch jobs run with
`--output-ldjson`, so the TUI (and its `q`/`^C` handler, `tui/mod.rs`) never
starts.

Measured: `integrate --output-ldjson -j 1 --target-rel 0.0003`, killed 12 s in
after 26 VEGAS progress events, exited 143 (SIGTERM) or 130 (SIGINT), wrote no
artifact, and the stream ended on a `progress` line with no `result` or
`refusal`.

The fix: install a handler in non-TUI modes that raises the existing
`StopSignal` on the first signal, exactly like `q`. Iterations run from seconds
to a few minutes even on heavy cards, so finishing one fits inside the grace
period. Kills during enumeration or compilation may stay hard kills. A stop
before any iteration is kept should end the stream with a refusal
([`integrate-result-has-no-stop-reason`](integrate-result-has-no-stop-reason.md)).
`generate`'s side is [`generate-has-no-graceful-stop`](generate-has-no-graceful-stop.md).
