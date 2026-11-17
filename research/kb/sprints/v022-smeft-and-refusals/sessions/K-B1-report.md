---
type: Session Report
title: "K-B1 report: batch-safe integrate and enumerate totals"
description: "SIGTERM as a stop request, stop_reason and reached error in the integrate result, integrate --check, atomic and compressed generate output, enumerate progress with a total, and default threads from the affinity mask, cgroup quota and OMP_NUM_THREADS."
status: draft
tags: [cli, batch, signals, ldjson, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-11-17}
---
Branch `sprintk-batch`, commits `7efe29b` (enumerate progress) and `651dc3f`
(the other six items; one commit because the help text, `reference.md` and
`integrate.rs` edits are interleaved).

## What changed

- **Signals** (`vibegraph-cli/src/signals.rs`, `signal-hook`): once sampling
  has started, the first SIGTERM or SIGINT raises the `StopSignal`; a second
  takes the default action. Before sampling, and for `generate`, every signal
  takes the default action. The TUI uses the same handler. A stop before any
  kept iteration ends in the existing `support` refusal (no new error kind,
  so `schema_version` stays 1).
- **Result fields**: `stop_reason` (`fixed_budget`, `target_met`, `max_iters`,
  `max_evals`, `stopped`, `constant`), `target_rel`, `rel_err_quoted`,
  `rel_err_scaled`; `null` from `inspect`, since the artifact does not record them.
- **`integrate --check`**: returns before the survey; `check ok` or the same refusal.
- **`generate`**: written to a hidden `.<name>.<pid>.partial` and renamed;
  `.gz`/`.zst` compress; other compression extensions are refused by name.
- **Enumerate progress**: `done` over a `total` known before the walk, plus
  `line`/`lines`, `subprocesses`, running `diagrams`.
- **Threads**: min of `available_parallelism` (affinity and cgroup, via std)
  and `OMP_NUM_THREADS`; `RAYON_NUM_THREADS` wins; logged.

## Oracles

Each is a test in `vibegraph-cli/tests/`: `cli_batch_stop` (a real child gets
SIGTERM and SIGINT in ldjson and SIGTERM in plain mode; second signal dies of
SIGTERM; stop in warm-up refuses; each reason pinned), `cli_check_only`
(refusal parity on four cards, three kinds), `cli_generate_output`,
`cli_enumerate_progress`, `cli_default_threads` (`taskset`). Results:
`cargo test -p vibegraph-lib --lib`: 1356 passed, 0 failed, 15 ignored;
`cargo test -p vibegraph --no-fail-fast`: all `test result` lines ok.
Workspace test and `validate-sigma` results are in the final message.

## What the tests cannot detect

- The cgroup quota is read by std and not exercised; only the affinity mask is.
- `constant` is pinned by its serde name only, not by a CLI run.
- The TUI signal path needs a tty and is untested.
- `--check` cannot see refusals that appear only once points are drawn.
- "Fails before" is argued from the old behaviour, not run against an old binary.

## Found

- `check-events` cannot read `.gz`/`.zst` event files.
- `generate-has-no-graceful-stop` is still open; a SIGTERM to `generate` now
  loses the run cleanly.
- A SIGKILL leaves the hidden `.partial` file behind.
- `kb-lint` errors: `run-card-resolution-not-reported.md: link to
  integrate-has-no-check-only-mode.md does not resolve`.
