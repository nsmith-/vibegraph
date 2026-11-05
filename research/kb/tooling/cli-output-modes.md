---
type: Design
title: CLI output modes
description: "Built: one output mode per run (plain, rich TUI, or ldjson via --output-ldjson) formats logs, progress and the result or refusal together; plain shows no progress; ldjson conflicts with --tui and carries a versioned schema with a stable error kind; vibegraph inspect reads a saved artifact's results."
status: draft
tags: [cli, logging, tui, tracing, progress, json]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
sources:
  - {id: output, resource: "../../../vibegraph-cli/src/output.rs", title: "vibegraph-cli/src/output.rs, the mode, the ldjson schema and its final objects"}
  - {id: logging, resource: "../../../vibegraph-cli/src/logging.rs", title: "vibegraph-cli/src/logging.rs, the tracing layers per mode (Sink)"}
  - {id: error, resource: "../../../vibegraph-cli/src/error.rs", title: "vibegraph-cli/src/error.rs, CliError and ErrorKind"}
  - {id: inspect, resource: "../../../vibegraph-cli/src/inspect.rs", title: "vibegraph-cli/src/inspect.rs, vibegraph inspect"}
  - {id: progress, resource: "../../../vibegraph-lib/src/progress.rs", title: "vibegraph-lib/src/progress.rs, the progress target and its field contract"}
  - {id: artifact, resource: "../../../vibegraph-lib/src/artifact.rs", title: "vibegraph-lib/src/artifact.rs, format 20 and MultiplicityTotal"}
  - {id: tests, resource: "../../../vibegraph-cli/tests/cli_output_ldjson.rs", title: "vibegraph-cli/tests/cli_output_ldjson.rs, the schema pinned on the binary"}
---
The design the user set on 2026-11-04, as built. It extends
[logging and the terminal UI](logging-and-tui.md). The user-facing
description, with the full field list, is the "Output modes" section of
the CLI overview (`docs/src/cli/overview.md`); the schema's reference is the
module documentation of `vibegraph-cli/src/output.rs`.

## Three paths, one mode

A run's output takes three paths:

- **log lines:** `tracing` events at the visible level;
- **progress:** `tracing` events on `vibegraph::progress` at TRACE, with a
  field contract (`stage`, `done`, `total` plus stage fields);
- **the result or refusal:** the command's final lines or object.

`main` chooses one `output::Mode` per run from the flags and the terminal,
and `logging::init` builds the layers for it (`logging::Sink`):

| mode | when | logs | progress | result or refusal |
|---|---|---|---|---|
| plain | not a terminal (background, piped), or `--no-tui` | text lines on stderr | not shown | text lines |
| rich | a terminal on both streams, or `--tui` | the TUI scrollback | the TUI pane | held until the pane comes down |
| ldjson | `--output-ldjson` | one `log` object per event | one `progress` object per event | one `result` or `refusal` object |

`check-events` and `inspect` are not watchable, so they are never rich.

## ldjson

- **The flag.** `--output-ldjson` is a global switch on `LogArgs`, declared
  `conflicts_with = "tui"`; `wants_tui` is false under it, so the pane stays
  down on a terminal too. ldjson is the only machine-readable format, so it is
  a switch rather than an `--output-format` value; if a second format ever
  appears, the flag can become an alias of the new option.
- **The stream.** Every line of stdout is one JSON object with `kind` (`log`,
  `progress`, `result`, `refusal`) and `schema_version`. Everything is on
  stdout so a reader sees events in order; each object is written under the
  stdout lock in one call.
- **Version on every line, not a header.** A consumer that filters the
  stream (keeps only `result`), reads a truncated one, or starts mid-stream
  still sees the version on every object it holds; a header line would make
  each of those a special case, for a few bytes per line saved.
- **`log` and `progress`** are `tracing-subscriber`'s JSON formatter's events
  (`timestamp`, `level`, `target`, `fields`, and `span` inside a stage's span),
  restamped with the two keys by the writer (`logging::LdjsonEvent`). The
  event's fields stay nested under `fields` so no event field can collide with
  `kind`. `log` uses the line layer's filter; `progress` is filtered on the
  target alone and so arrives at any level, as the TUI's layer does.
- **Units.** The final objects are in pb (GeV for a decay's width, and for
  masses and scales) at full precision. The `progress` objects carry the field
  contract verbatim, so the VEGAS stage's `sigma`/`err` are in the integrand's
  natural units (GeV⁻² for a cross section, GeV for a width). Converting them
  in the stream would need the observable, which the progress layer does not
  know.
- **The final objects.** `integrate` emits `output::IntegrateResult` after the
  artifact is written; it is built from the artifact (`integrate::result_object`),
  which is what makes `inspect --output-ldjson` of that artifact the same
  object, a property `cli_output_ldjson` pins on a run. μF is reconstructed
  from the recorded inputs and run card by the function the run used
  (`FactorizationScale::recorded_in`). `generate` emits `output::GenerateResult`:
  events written and drawn, the file, the sample's σ and the pull against the
  integration (`vibegraph::generate::sample_pull`, the function its log line uses).
- **The refusal** carries `error_kind` and `message`. `main` emits it in place
  of the `error!` line the text modes print.
- **Schema rule.** Adding a field keeps `SCHEMA_VERSION`; removing, renaming or
  re-meaning a field, or adding or removing an error kind, bumps it.

## Error kinds (decision G10)

`CliError` carries an `ErrorKind`, set where each error is built:

| kind | the failure |
|---|---|
| `usage` | the flags, or a combination of them |
| `card` | a proc, run or param card cannot be read or parsed, or asks for something refused |
| `model` | the model cannot be located, loaded or bound |
| `process` | enumeration or compilation fails, or the process has a shape the integrator refuses |
| `pdf` | the parton densities cannot be located, read or matched to the card |
| `network` | a download the network policy refused, or one that failed |
| `support` | the integration's result is not supported by its points (none kept, too few carrying weight) |
| `artifact` | a saved artifact is unreadable, of another version, or disagrees with the build reading it |
| `io` | another file or directory cannot be used |
| `run` | the run failed after its inputs were accepted, for a reason no other kind names |

A PDF failure is classified from the library's `SelectFailure` and
`LookupError` (`error::pdf_error_kind`).

## Per-multiplicity totals (decision G11)

Artifact format 20 records `MultiplicityTotal` per final-state phase space of
a proton run (`IntegrateArtifact::multiplicities`), the totals `integrate`
prints on its per-multiplicity lines; format 19 is refused by version, tested
on 19's real field layout.

## `vibegraph inspect <grid.bin.zst>`

Prints a saved artifact's results as text (process, model, σ, χ²/dof, the
scale, the per-multiplicity lines when there are several, each channel, the
path and format version), or under `--output-ldjson` the `result` object.

## The survey

`validation/cms_gridpacks/vgrun.py` runs `integrate --output-ldjson`, and
`parse.parse_vg_output` reads the stream's objects when it holds any, and an
older campaign's text output otherwise; both are tested.
