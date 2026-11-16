---
type: Backlog Item
title: "Logs have no structured form outside the ldjson output mode"
description: "A consumer following vibegraph's log events must match log text; a --log-format json option would let any command emit its log as JSON lines."
area: feature
state: open
priority: low
closes_when: "A --log-format json option (or equivalent) emits every log record as a JSON object with level, target, message and fields, on every subcommand, with a CLI test."
blocked_by: []
opened: 2026-11-16
tags: [cli, logging, json, simshop]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-16}
---
Requested by simshop, which parses `enumerate`'s log text to tell a loaded
model from a refused one. `--output-ldjson` covers `integrate` and `generate`
results, but not log records on every command. A model-check command
([`model-check-command-missing`](model-check-command-missing.md)) removes
simshop's immediate need. This item is for consumers following any command's
progress.
