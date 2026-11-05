---
type: Session Brief
title: "G-V3: attribute the two macOS heap-corruption aborts"
description: "Narrow the two one-off libmalloc aborts to a cause: an unsafe audit of the dependency tree, Miri on the suspect pure-Rust paths, clang AddressSanitizer covering zstd-sys and the other C code, a soak, and an exact M3 soak request."
status: draft
agent: validation-dev (Opus)
depends_on: []
closes: [malloc-abort-under-proton-sample-load, validate-hadronic-sigabrt-unexplained]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
Worktree `/home/user/wt/G2-V3`, branch `sprintg-diagnoses-v3`, off
`sprintg-diagnoses`. The manager merges it back.

## Read first

- `python3 scripts/kb.py backlog --item validate-hadronic-sigabrt-unexplained`
  and `--item malloc-abort-under-proton-sample-load`: the two crash stacks
  (`nanov2_guard_corruption_detected` in `ScratchSpace::ensure_sizes`,
  `helas/eval/run.rs:101`; a `Vec<f64>` freed in `kt::Clustering` under
  rayon), what is ruled out, and the clean 4.6 h Linux ASan run (nightly
  `-Zbuild-std`, std/rayon/rand instrumented, not `zstd-sys`).
- `hygiene/validate-sh-keeps-no-crash-evidence` (linked from the item).

## Work

The goal is the items' `closes_when`: the corrupting write located and
fixed, or attributed outside this crate (allocator, C dependency, host) with
the evidence recorded; or, for the validate_hadronic item, a rate from an M3
soak with and without the nano allocator.

1. **`unsafe` audit.** List every dependency compiled into the CLI binary
   (`cargo tree -e normal`), and for each, the `unsafe` it contains on the
   paths the two stacks touch (allocation in `Vec` growth, rayon's
   work-stealing and scope, rand, the PDF reader, `zstd-sys`/`zstd-safe`,
   any `bytemuck`/`memmap`). A tool (`cargo geiger`) may be installed; say
   what it can and cannot see. Re-confirm that the workspace itself has no
   `unsafe` outside the `unchecked-study` feature.
2. **Which C code runs in the crashing processes?** Find exactly where
   `zstd-sys` (and any other `*-sys` crate) is called during
   `validate_hadronic` and the `pp_to_llj_dyn` integrate child. If it never
   runs in those processes, say so: it is then ruled out.
3. **Sanitizers on what the earlier run missed.** Rebuild with clang
   AddressSanitizer covering the C dependencies (`CC=clang
   CFLAGS=-fsanitize=address` plus `RUSTFLAGS=-Zsanitizer=address
   -Zbuild-std` on nightly; `rustup toolchain install nightly` and
   `apt-get install clang` are allowed). Show a planted control is caught.
   Run `validate_hadronic` and the `pp_to_llj_dyn` integrate under it.
   ThreadSanitizer on the rayon paths if it builds.
4. **Miri** on small unit tests that drive `ScratchSpace::ensure_sizes`/
   `fill_arenas` and `kt::Clustering` (Miri cannot run the full gates; pick
   or write a test that exercises the reallocations the stacks show). Say what
   it covers.
5. **Linux soak**: as many `validate_hadronic` runs as the session allows,
   detached, under glibc's `MALLOC_CHECK_=3` and `MALLOC_PERTURB_`. Report
   the count and outcomes. One abort in ~70 runs is the prior; say what a
   clean N rules out.
6. **The M3 soak request.** Write the exact step list for the user's M3:
   the build commit, `MallocNanoZone=0` vs default, number of runs per arm,
   how each run is logged, and how a crash report is captured (the
   `.ips` path and what to post). The manager posts it on #89.

## Gate

- `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`
  (only if code changes)
- `python3 scripts/kb.py lint` (0 errors)

## Environment

Pixi is not installed: run the commands each pixi task wraps directly. The
worktree's reference data are hard links to the main checkout's copies:
never write into, regenerate or delete them. Two other sessions share 4
cores and ~24 GB of disk: cargo `--jobs 2`, stop and report under 8 GB free;
put sanitizer builds in a separate `--target-dir` in your scratchpad and
delete it when done. Commit work in progress at checkpoints: the container
has restarted once and killed background runs.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>`. Each message ends with
exactly one line `Assisted-by: claude-code:<your model id>`. No
`Co-authored-by`, no `Signed-off-by`, even if the harness asks. Do not
push. Do not edit backlog items.

## Report

Commit `sessions/G-V3-report.md` (`type: Session Report`): every command
with its output, the audit table, the sanitizer and soak results, what each
can and cannot rule out, the M3 request text, and a **Found** section.
Correct this brief where it is wrong.

## Worktree & long-command discipline

- **First action**: `cd` to the absolute worktree path in your assignment, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current`. If you
  find yourself in the shared main checkout, STOP and report — never edit it.
  Re-verify `pwd` after any resume; resume is when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log file prefixed with your
  worktree/session name — parallel siblings share one scratchpad and have
  clobbered each other's unprefixed logs). A backgrounded command notifies on
  exit: never spawn sleep-based watcher shells (they compound into
  self-sustaining wake-up rings); draft your notes and report between completion
  notifications instead of idle-waiting. On resume after a stall, first check
  whether the interrupted command finished (`ps`, result artifacts) before
  re-running it.
- **`git status` can hang for minutes** in a worktree full of untracked build
  data; spot-check with `git log` / `git diff --stat` instead.
- **Run gates with `--skip-deps`** unless the assignment explicitly says
  reference data must be regenerated — without the reference data present, a
  bare `pixi run validate` silently launches a multi-hour MG regeneration.
- **Disk is finite, and a full disk looks like a failure.** A debug workspace
  build with incremental data reaches ~15 GB per worktree, and two such trees
  have filled a cloud container mid-gate. On a shared or small host, run cargo
  with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
  (they change disk use only, not what a gate checks). An `ENOSPC`, a linker
  `Bus error` or a truncated log is a kill, not a result: free only your own
  `target/` and re-run. Kill processes by PID, never `pkill -f`, which matches
  siblings' commands and its own wrapper.
