---
type: Backlog Item
title: "A banked run that dies on a signal leaves nothing to diagnose"
description: "validate.sh neither raises the core limit nor points at macOS DiagnosticReports, and stderr is the console only, so a SIGABRT in a banked suite cannot be attributed afterwards."
area: hygiene
state: open
priority: low
closes_when: "validate.sh keeps crash evidence: it raises ulimit -c (or tees stderr to a log), and on a cargo 'signal:' failure prints where the core or the DiagnosticReports .ips is."
blocked_by: []
opened: 2026-10-26
tags: [validation, tooling, crash, macos]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: probe, resource: "https://github.com/nsmith-/vibegraph/pull/60#issuecomment-6423488513", title: "nsmith-/vibegraph#60, the SIGABRT probe summary"}
---
The one-off SIGABRT in `validate_hadronic` during the refdata-11 export (see
[validate-hadronic-sigabrt-unexplained](../validation/validate-hadronic-sigabrt-unexplained.md))
could not be attributed from what the run kept. `validation/validate.sh` runs
`cargo test … --no-fail-fast -- --nocapture` with no `ulimit -c`, writes no log
of its own, and does not say where macOS puts a crash report
(`~/Library/Logs/DiagnosticReports/<binary>-*.ips`). Every abort route in Rust
std prints a line first (stack overflow, a panic during a panic, an allocation
failure, rayon's abort), so a stderr log alone would usually name the cause.

Source: the probe session's report (scratchpad, uncommitted), summarised on
[#60](https://github.com/nsmith-/vibegraph/pull/60#issuecomment-6423488513).
