---
type: Design
title: CLI output modes
description: "Proposed: one output mode per run (plain, rich TUI, or ldjson) formats log events, progress events and the result together; ldjson conflicts with --tui; an inspect subcommand reads a saved artifact's results."
status: draft
tags: [cli, logging, tui, tracing, progress, json]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: logging, resource: "../../../vibegraph-cli/src/logging.rs", title: "vibegraph-cli/src/logging.rs, the tracing layers and LogArgs::wants_tui"}
  - {id: progress, resource: "../../../vibegraph-lib/src/progress.rs", title: "vibegraph-lib/src/progress.rs, the progress target and its field contract"}
  - {id: result-line, resource: "../../../vibegraph-cli/src/tui/mod.rs", title: "vibegraph-cli/src/tui/mod.rs, result_line"}
  - {id: artifact, resource: "../../../vibegraph-lib/src/artifact.rs", title: "vibegraph-lib/src/artifact.rs, the recorded sigma_pb/sigma_err_pb/chi2_per_dof"}
---
A proposal set by the user on 2026-11-04 and not yet built. It extends
[logging and the terminal UI](logging-and-tui.md).

## Today

A run's output takes three paths:

- **log lines:** `tracing` events at the visible level;
- **progress:** `tracing` events on `vibegraph::progress` at TRACE, with a
  field contract (`stage`, `done`, `total` plus stage fields);
- **the result:** `tui::result_line`, which goes straight to stdout.

`logging.rs` decides which layers consume these:

- **piped:** formatted log lines go to stderr, and progress is not shown at all;
- **on a terminal:** the TUI takes the formatted lines for its scrollback,
  and a second layer filtered on the progress target updates its state.

## One output mode per run

The user's design: choose one output mode per run, and have it format all
three paths.

| mode | when | logs | progress | result or refusal |
|---|---|---|---|---|
| plain | not a terminal (background, piped), or `--no-tui` | text lines | text status lines | text lines |
| rich | a terminal on both streams, or `--tui` | the TUI scrollback | the TUI pane | held until the pane comes down |
| ldjson | `--output-format ldjson` | one object per event | one object per event | a final object |

### ldjson

- **The stream.** Every line is one JSON object with a `kind` field: `log`,
  `progress` or `result`/`refusal`. Everything goes on one stream (stdout)
  so a reader sees events in order. The `log` objects respect the visible
  level. The `progress` objects carry the progress target's fields
  whatever the level, as the TUI's layer does today.
- **The final object.** The result carries σ, its error, χ²/dof or null,
  μF, the per-multiplicity and per-channel results, and the artifact path.
  A refusal carries its error kind and message. Refusals write no artifact,
  so this is the only place a tool can read one.
- **Values.** They are in base units (pb), at full precision. SI-prefixed
  rounding belongs only to the text modes.
- **Versioning.** The schema carries a version, separate from the artifact
  format, so a consumer can refuse an unknown one.
- **`--tui`.** The flag is declared `conflicts_with = "tui"`, so combining
  them is a usage error. ldjson also suppresses the pane on a terminal.
- **Serialization.** Use `tracing-subscriber`'s `json` feature and
  `serde_json`, already a dependency of the CLI, rather than hand-writing JSON.

### plain

Progress is now shown here too. It needs a rate limit (per stage, or every
N seconds) so a background log does not fill with VEGAS iterations; that
choice is open.

## `vibegraph inspect <grid.bin.zst>`

Prints the results a saved artifact records, in the plain or ldjson form of
the final result object. The artifact already stores `sigma_pb`,
`sigma_err_pb` and `chi2_per_dof`, for the run and for each channel, along
with the run card, maps and format version, so nothing needs re-running.
