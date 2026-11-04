---
type: Backlog Item
title: integrate's results are readable only by scraping its human-formatted output
description: "Tools that consume integrate parse its human-formatted summary lines, which breaks whenever the wording changes; add a line-delimited JSON output and an inspect subcommand."
area: feature
state: open
priority: medium
closes_when: "The CLI has three output modes chosen once per run (plain lines, the TUI, and ldjson under the `--output-ldjson` flag), each formatting log events and the result or refusal, with progress events shown by the TUI and ldjson but not by plain; ldjson with `--tui` is a usage error; an `inspect` subcommand prints a saved artifact's recorded results as text, or as ldjson under `--output-ldjson`; the ldjson schema carries a version and is pinned by CLI tests; and the gridpack survey reads ldjson instead of the summary lines."
blocked_by: []
opened: 2026-11-04
tags: [cli, output, artifact, cms-survey]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: integrate, resource: "../../../../vibegraph-cli/src/integrate.rs", title: "vibegraph-cli/src/integrate.rs, the summary lines"}
  - {id: artifact, resource: "../../../../vibegraph-lib/src/artifact.rs", title: "vibegraph-lib/src/artifact.rs, the artifact's sigma_pb/sigma_err_pb/chi2_per_dof and per-channel fields"}
  - {id: parser, resource: "../../../../validation/cms_gridpacks/parse.py", title: "validation/cms_gridpacks/parse.py, the survey's line parser"}
  - {id: pr90, resource: "https://github.com/nsmith-/vibegraph/pull/90", title: "PR #90: CLI summary lines, which had to update the survey parser"}
  - {id: design, resource: "../../tooling/cli-output-modes.md", title: "CLI output modes"}
---
The CMS gridpack survey (`validation/cms_gridpacks/parse.py`) learns what
`integrate` did by matching its summary lines with regular expressions. When
the per-multiplicity line gained `; masses …`, the parser silently stopped
matching it, and #90's σ formatting needed another parser change. Any tool
that consumes vibegraph has the same problem: the output that a person reads
is the only interface.

The fix is one output mode per run, plain, rich (TUI) or ldjson, which formats
log events, progress events and the result or refusal alike, plus `vibegraph
inspect` to print a saved artifact's results. The design is in
[CLI output modes](../../tooling/cli-output-modes.md).
