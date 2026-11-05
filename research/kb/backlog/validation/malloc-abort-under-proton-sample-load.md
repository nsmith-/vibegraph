---
type: Backlog Item
title: Nondeterministic heap-corruption abort under the proton-sample suite
description: A vibegraph integrate child aborted once in seven runs with libmalloc's "pointer being freed was not allocated" during VEGAS warm-up, unreproduced since.
area: validation
state: open
priority: low
closes_when: The abort is reproduced and its cause fixed, or attributed outside this crate (allocator, OS or a dependency) with the evidence recorded.
blocked_by: []
opened: 2026-09-07
tags: [flaky, memory-safety, rayon, kt-clustering, proton, macos]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L362-L374", title: "TODO.md entry T024"}
---
On 2026-09-07, one `validate_samples_proton` run in seven aborted. The aborting
process was a `vibegraph integrate` child on `pp_to_llj_dyn`'s card, in VEGAS
warm-up, and macOS libmalloc reported "pointer being freed was not allocated".
The crashing rayon worker was dropping a `Vec<f64>` inside `kt::Clustering`
while another worker was in `cluster::kt` / `cluster::graph`.

Ruled out so far:
- `unsafe` in the workspace (there is none).
- A non-`System` allocator.
- Any local difference against `main` in `coupling/cluster/**`, `hadronic.rs`,
  `proton.rs` or `vegas`.

It has not reproduced in any targeted rerun since, including about 12 full
`validate` runs at host loads up to 84, or in the three
`validate_samples_proton` invocations of the seed-headroom census
([note 36a §9](../../history/notes/36a-seed-headroom-census.md)). The crash report is on the
user's macOS host at
`~/Library/Logs/DiagnosticReports/vibegraph-2026-09-07-085712.ips`.

Standing protocol until this closes: when a gate under this suite aborts this
way, rerun it once and report both outcomes. Do not treat either run alone as
the measurement ([note 36 §2](../../history/notes/36-banked-open-ends-plan.md)).

**Deferred until it recurs (user, 2026-11-05).** Sprint G looked for it and
found nothing:
- **Linux** ([G-V3](../../sprints/v02-diagnoses/sessions/G-V3-report.md)):
  no workspace `unsafe` outside `unchecked-study`; mimalloc not linked; zstd
  runs only at start-up and after integration. ASan with `zstd-sys`
  instrumented, valgrind on the asm decode, Miri on the scratch pattern and
  the clustering tests, and TSan were all clean. glibc's `MALLOC_CHECK_` needs
  `libc_malloc_debug.so` preloaded (since 2.34), so earlier Linux runs that
  only set the variable had no heap checking.
- **M3 soak of the exact refdata-11 binary** (`47ac21a`, sha256 `c1918131…`;
  [#89](https://github.com/nsmith-/vibegraph/pull/89#issuecomment-6654446012)):
  0 aborts in 200 `validate_hadronic` runs, 100 with the default nano
  allocator and 100 with `MallocNanoZone=0`. At the prior 1-in-70 rate that
  happens 24 % of the time per arm: neither a reproduction nor a ruling-out.

**If it recurs:** post the `.ips` crash report, `git rev-parse HEAD` and the
binary's sha256, and rerun the soak harness from the #89 comment.
