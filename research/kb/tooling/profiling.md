---
type: Procedure
title: Profiling and stage timing
description: "Profile a gate under samply with scripts/profile.sh (release-debug, extended-validation); per-row duration_s with host.json; time_stages.py for MadGraph's own stages."
status: draft
tags: [profiling, samply, timing, performance, madgraph]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n19-survey, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/19-validation-pass-plan.md#L32-L57", title: "Note 19 §2: survey findings (banked runs, profiling substrate)"}
  - {id: n19-v3b, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/19-validation-pass-plan.md#L116-L138", title: "Note 19 §V3b: the profiling recipe on the σ gate"}
  - {id: n30-instrumented, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/30-perf-baseline-timings.md#L70-L105", title: "Note 30 §2: what was instrumented"}
  - {id: n30-profiles, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/30-perf-baseline-timings.md#L460-L478", title: "Note 30 §7: profiles"}
  - {id: profile-sh, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/scripts/profile.sh", title: "scripts/profile.sh"}
  - {id: time-stages, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/validation/madgraph/time_stages.py", title: "validation/madgraph/time_stages.py"}
---
# Profiling and stage timing

Three instruments, each answering a different question. How their numbers may
be read is owned elsewhere:
[end-to-end timing protocol](../performance/end-to-end-timing-protocol.md) for
row durations and integrate runs, and
[microbenchmark protocol](../performance/microbenchmark-protocol.md) for
criterion benches. What the profiles have shown is
[integrate profiles](../performance/integrate-profiles.md).

## 1. Sampling profiles: `scripts/profile.sh`

```
bash scripts/profile.sh <test-name> [test-filter] [-- samply-args...]
pixi run profile-sigma        # = profile.sh validate_sigma
pixi run profile-diagrams     # = profile.sh validate_madgraph_diagrams
```

The script builds the named integration test with
`cargo test --profile release-debug --test <name> --features extended-validation --no-run`,
parses the executable path from cargo's output, and `exec`s
`samply record [samply-args] <executable> [filter]`. The pixi tasks depend on
`install-samply` (`cargo install --locked samply` if absent).

- **Profile**: `release-debug` (thin LTO, `debug = 1`), the same profile the
  banked gates run under, so a profile shows the code a gate measures. There
  is no separate `profiling` profile ([cargo configuration](cargo-configuration.md)).
- **`extended-validation` is on**, so debug-only cross-checks compiled under
  that feature are in the profiled code; check their share before reading
  small percentages (the arena validator took 0.09% in one study).
- **Why profile a gate rather than a fixed loop**: the σ gate loops over
  every banked partonic process with its real run card, so time is weighted
  by how hard each process is to *integrate*, and hotspots rank in a
  tackle-worthy order. A fixed-N loop per process ranks them by evaluation
  count instead.[^n19-v3b] The profile is the gate's process mix; a single-process
  profile shifts the balance.
- **Limits of the script**: it forwards one filter argument only, so it cannot
  pass `--test-threads=1` or other test-harness flags. For those, run its
  `cargo test … --no-run` line yourself and call `samply record` on the
  executable directly.

Useful samply flags: `--save-only -o <path>.json.gz` to record without opening
a browser, and `--unstable-presymbolicate` so the saved profile carries a
`.syms.json` sidecar with its symbols; browse later with `samply load <path>`.
At samply's default 1000 Hz, a single-threaded integrand under a rayon pool
shows the busiest thread doing the work and the other threads parked in
`__psynch_cvwait` (about 94% of all samples on a 16-core host), so read
percentages as self time within the busiest thread.[^n30-profiles] Going
below function level, to instructions and inlined frames, is
[instruction-level profiling](instruction-level-profiling.md).

## 2. Per-row wall time in the validation report

Every gate that writes a report row writes `duration_s` beside it: the wall
time of that row's own measurement, from where its work starts to where the
row is written (`vibegraph-lib/tests/common/report.rs`, `Stopwatch`). The
first row any gate process writes also writes
`target/validation-report/host.json` atomically, so one machine block
accompanies each run's rows; `validation/validate.sh` deletes it with the row
directories, so no run reads its durations against another machine's
identity. The collator renders a `## Timing` section and carries
`durations_s` per cell in `report.json`.[^n30-instrumented]

**`duration_s` is not a benchmark.** `cargo test` runs a binary's tests on
parallel threads and the integrators fan out over rayon underneath, so rows
measured concurrently each charge themselves the contended wall time and
their durations overlap; a category's summed time is not the invocation's
elapsed time. What the numbers license is in
[end-to-end timing protocol](../performance/end-to-end-timing-protocol.md);
the host block's fields and the hosts themselves are
[benchmark hosts](../performance/benchmark-hosts.md).

## 3. MadGraph's own stages: `time_stages.py`

```
pixi run -e madgraph python validation/madgraph/time_stages.py --out <scratch-dir> ee_to_mumu gg_to_gg ...
```

Each named process is regenerated from scratch into `--out` (required; the
script refuses `validation/madgraph/output`, since a regenerated directory
must not be mistaken for a banked one) through `mg5_pinned.sh`, and every line
MadGraph prints is stamped with the seconds since that process started. Stage
boundaries are read off the transcript:

| stage | opens at | closes at |
|---|---|---|
| `startup` | process start | `Checking for minimal orders` / `Trying process` |
| `generate` | that | `N processes with M diagrams generated in X s` |
| `output` | `initialize a new directory` | `Output to directory ... done.` |
| `compile` | `compile directory` | `Running Survey` |
| `integrate` | `Running Survey` | `finish refine` |
| `events` | `Combining Events` | `End Parton` |

`generate` is cross-checked against MadGraph's own printed self-timing; the
two agreed to the millisecond on every process when introduced. The result is
`<out>/timings.json`, rewritten after every process so an interrupted pass
still leaves a record, carrying the same machine identity as `host.json` plus
the Fortran compiler and flags MadGraph built with, and one timestamped
transcript per process under `logs/`. It links the C++ runtime per platform
(`-lc++` on macOS, `-lstdc++` elsewhere).

A host-labelled `validation/madgraph/timings.json` is committed: host-specific
data stays out of the host-independent refdata bundle but lives in git with
its provenance. Its numbers are
[validation-layer timings](../performance/validation-layer-timings.md).
Two things a pass usually has warm that a truly cold one would not: the OS
page cache (a cold first MadGraph invocation pays a few seconds of Python
import) and the conda environment.

## Per-point timing against MadGraph

Comparing evaluator cost per phase-space point against MadGraph's `MATRIX1` is
a fourth, separate tool: [mg_perf_compare](mg-perf-compare.md).

[^n19-v3b]: Note 19 §V3b, the profiling recipe that replaced the `validate_helas_mg` timing print.
[^n30-instrumented]: Note 30 §2, "What was instrumented".
[^n30-profiles]: Note 30 §7, "Profiles".
