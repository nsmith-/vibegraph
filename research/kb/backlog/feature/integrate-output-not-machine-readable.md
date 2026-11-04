---
type: Backlog Item
title: integrate's results are readable only by scraping its human-formatted output
description: "Tools that consume integrate parse its human-formatted summary lines, which breaks whenever the wording changes; add a line-delimited JSON output and an inspect subcommand."
area: feature
state: open
priority: medium
closes_when: "integrate takes an option that writes line-delimited JSON (one object per progress event, a final result or a typed refusal with its error kind), an `inspect` subcommand prints a saved artifact's recorded results in text or JSON, both schemas carry a version field and are pinned by CLI tests, `--output-format ldjson` together with `--tui` is a usage error, and the gridpack survey reads them instead of the summary lines."
blocked_by: []
opened: 2026-11-04
tags: [cli, output, artifact, cms-survey]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: integrate, resource: "../../../../vibegraph-cli/src/integrate.rs", title: "vibegraph-cli/src/integrate.rs, the summary lines"}
  - {id: artifact, resource: "../../../../vibegraph-lib/src/artifact.rs", title: "vibegraph-lib/src/artifact.rs, the artifact's sigma_pb/sigma_err_pb/chi2_per_dof and per-channel fields"}
  - {id: parser, resource: "../../../../validation/cms_gridpacks/parse.py", title: "validation/cms_gridpacks/parse.py, the survey's line parser"}
  - {id: pr90, resource: "https://github.com/nsmith-/vibegraph/pull/90", title: "PR #90: CLI summary lines, which had to update the survey parser"}
---
The CMS gridpack survey (`validation/cms_gridpacks/parse.py`) learns what
`integrate` did by matching its summary lines with regular expressions. When
the per-multiplicity line gained `; masses …`, the parser silently stopped
matching it, and #90's σ formatting needed another parser change. Any tool
that consumes vibegraph has the same problem: the output that a person reads
is the only interface.

The fix has two parts, and the user asked for both:

- **A machine-readable stream from `integrate`.** An option (say
  `--output-format ldjson`) writes one JSON object per line:
  - each progress event the TUI already consumes;
  - then a final object, either the result (σ, its error, χ²/dof or null,
    μF, per-multiplicity and per-channel results, the artifact path) or a
    refusal carrying its error kind and message.
  
  Refusals write no artifact, so the stream is the only place a tool can read
  one.

  The stream is for a program reading from a pipe, so it goes to stdout and
  the logs stay on stderr. It is declared `conflicts_with = "tui"` in clap,
  so `--tui` with it is a usage error. It also implies `--no-tui`, so a
  terminal on both streams does not draw the pane over the stream either
  (`LogArgs::wants_tui`, `vibegraph-cli/src/logging.rs`).
- **`vibegraph inspect <grid.bin.zst>`.** Prints the results the artifact
  records, as text or JSON. The artifact already stores `sigma_pb`,
  `sigma_err_pb` and `chi2_per_dof`, for the run and for each channel, along
  with the run card, maps and format version. `inspect` would show them after
  the fact without re-running anything.

Both JSON schemas carry a version, separate from the artifact format, so that
a consumer can refuse an unknown one. Values are in base units (pb), at full
precision; the SI-prefixed rounding belongs only to the human-readable output.
