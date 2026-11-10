---
type: Backlog Item
title: The musl Linux release binaries get slower with more threads
description: "Both v0.2.0 Linux assets (static musl, plain and -v3) run slower at -j 4 than at -j 1; a glibc build of the same commit scales 3.7×. Allocator lock contention is suspected, not profiled."
area: performance
state: open
priority: high
closes_when: "On a ≥4-core x86 host, both musl release assets run pp_to_jj at a fixed budget at -j 4 in at most 1/3 of their -j 1 wall time, and within 10% of a glibc build of the same commit at -j 1 and at -j 4. The measurement (host, commands, wall, sys time) is recorded in a performance concept, and σ at fixed seed is unchanged within the integrator's own error."
blocked_by: []
opened: 2026-11-10
tags: [release, musl, allocator, threads, rayon, linux, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: cern, resource: "https://github.com/nsmith-/vibegraph/releases/tag/v0.2.0", title: "v0.2.0 release assets, benchmarked on the CERN host 2026-11-09/10 (results/phase2a/j4_diag on that host)"}
---
Measured on the CERN host (4-vCPU KVM Ice Lake, AlmaLinux 10.2) at v0.2.0
(`3cc7f9c2`), `pp_to_jj` at a fixed budget (480k evaluations, same σ in
every run, steal 0, host idle):

| binary | -j | wall s | sys s | context switches/s |
|---|--:|--:|--:|--:|
| `vibegraph-x86_64-unknown-linux-musl-v3` | 1 | 26.6 | 0.0 | 800 |
| same | 2 | 43.4 | 18.1 | 51k |
| same | 4 | 63.9 | 75.5 | 204k |
| glibc `cargo build --release`, same commit | 1 | 21.7 | 0.0 | 530 |
| same | 4 | 5.8 | 0.0 | 600 |

Every row run to 0.1% with both musl assets was slower at `-j 4` than at
`-j 1` (`pp_to_jj` 159 s against 52 s), with sys time 40–60% of user time.
Single-threaded, musl was about 20% slower than glibc (27.1 s against 21.6 s,
pinned, `pp_to_jj` only). The default thread count is every core, so every
Linux user of the release binaries gets the slowdown. The macOS assets do not
link musl.

The signature (sys time and context switches rising with threads, none under
glibc) fits contention on musl's allocator (mallocng's global lock), but it
was not profiled. The likely fix is a `#[global_allocator]` from a maintained
crate (mimalloc or jemalloc) in `vibegraph-cli`, at least on
`target_env = "musl"`. Confirm the cause with a profile first, and check that
the musl legs of `release.yml` still build statically with the chosen crate.

Two open items abort in macOS libmalloc
([malloc-abort-under-proton-sample-load](../validation/malloc-abort-under-proton-sample-load.md),
[validate-hadronic-sigabrt-unexplained](../validation/validate-hadronic-sigabrt-unexplained.md)).
A musl-only allocator change does not touch them; one applied on every
target would, so say which in the PR.
