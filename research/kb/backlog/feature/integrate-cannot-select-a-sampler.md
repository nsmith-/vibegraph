---
type: Backlog Item
title: vibegraph integrate has no way to choose a sampler
description: "There is no --sampler option, and the --grid-* flags PR #48 adds would be global VEGAS knobs. Every candidate sampler's measurement needs a per-run choice with kind-scoped flags."
area: feature
state: open
priority: medium
closes_when: "vibegraph integrate --sampler {vegas,uniform} selects the kind (default vegas), refuses another kind's flags, reports the kind in its summary and banks it in the artifact; the default invocation's artifact is byte-identical to before; the CLI reference is regenerated."
blocked_by: [sampler-hardwired-to-vegas-grid, artifact-has-no-sampler-kind]
opened: 2026-10-17
tags: [cli, sampler, integration, refactor]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
---
Once the seam exists, a run has to be able to pick its sampler, and each later
candidate adds a value. `vibegraph integrate` (`vibegraph-cli/src/integrate.rs`)
gains `--sampler <kind>` with `vegas` the default. This waits on PR #48
merging: its `GridRuleArgs` (`--grid-bin-weight`, `--grid-acceptance-rescale`,
`--grid-max-bin-width`), or whatever flags that work leaves, become the `vegas`
kind's flag group and are refused under any other kind rather than ignored. Each later kind brings its own group. The selection is a CLI choice,
not a run-card key: MadGraph's run card has no such entry, and `gridrun` and
`gridpack` are classified as ignored (`runcard/classes.rs`).

`vibegraph generate` takes no sampler flag: the kind comes from the artifact
([artifact-has-no-sampler-kind](artifact-has-no-sampler-kind.md)). The `uniform`
kind is the second value from the start, so the option is never a one-value
switch, and it is the "no adaptation" arm every comparison wants. The default
invocation must be byte-identical to today's, by the same procedure as the seam
([sampler-hardwired-to-vegas-grid](sampler-hardwired-to-vegas-grid.md)).
