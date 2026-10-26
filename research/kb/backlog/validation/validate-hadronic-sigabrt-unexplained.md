---
type: Backlog Item
title: "One unexplained SIGABRT in validate_hadronic during the refdata-11 export"
description: "validate_hadronic died once on SIGABRT with no panic message on the M3; 22 Linux runs under the banked flags did not reproduce it, and the macOS crash report has not been read."
area: validation
state: open
priority: medium
closes_when: "The abort is attributed from the M3's DiagnosticReports .ips or its stderr, or it recurs with crash evidence captured and is fixed or attributed outside this crate."
blocked_by: []
opened: 2026-10-26
tags: [flaky, banked, macos, abort]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: pr60-m3, resource: "https://github.com/nsmith-/vibegraph/pull/60#issuecomment-6373556349", title: "nsmith-/vibegraph#60, the M3's refdata-11 comment"}
  - {id: pr60-ask, resource: "https://github.com/nsmith-/vibegraph/pull/60#issuecomment-6423488513", title: "nsmith-/vibegraph#60, the request for the crash report"}
---
During the refdata-11 export on the M3, the first full `validate` run on a clean
`git archive` tree died with SIGABRT in `validate_hadronic`, with no panic
message. The test passed 20 of 20 alone and a full rerun was clean
([#60 comment](https://github.com/nsmith-/vibegraph/pull/60#issuecomment-6373556349)).

A probe on Linux (worktree `sprintd-sigabrt-probe` at `47ac21a`, refdata-11,
the banked job's `release-debug` flags) ran the binary 22 times and the full
banked step once, all exit 0. Test threads 1–16, 16 rayon threads and
`RUST_MIN_STACK` down to 64 KiB changed nothing. The printed σ, pull and χ²
lines were identical across runs, so the abort is not value-driven. There is no
`panic = "abort"`, no `process::abort` in the library and no `unsafe`. Every
abort route in Rust std prints a line first, so "no panic message" probably
means a filtered read.

The abort host is macOS. The M3 was asked for
`~/Library/Logs/DiagnosticReports/validate_hadronic-*.ips` and the stderr near
the abort ([#60 comment](https://github.com/nsmith-/vibegraph/pull/60#issuecomment-6423488513)).
Related: [malloc-abort-under-proton-sample-load](malloc-abort-under-proton-sample-load.md),
another one-off macOS abort.
