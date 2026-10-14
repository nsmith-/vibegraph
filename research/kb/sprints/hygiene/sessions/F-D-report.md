---
type: Session Report
title: "F-D report: phase-space and sampling fixes"
description: "All 13 R-D findings (R-D.13 included) and both claimed items fixed in 6 commits: one seeked VEGAS kernel with contracts on production code, the test-only parallel scheme deleted, σ-recovery tests gated on the run's own error with 20-seed headroom."
status: draft
tags: [hygiene, fix, vegas, phase-space, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-10}]
sources:
  - {id: 9c5336f, resource: "https://github.com/nsmith-/vibegraph/commit/9c5336f", title: "one seeked parallel kernel, contracts on production code"}
  - {id: 9bc479e, resource: "https://github.com/nsmith-/vibegraph/commit/9bc479e", title: "production docs describe production maps"}
  - {id: 4178366, resource: "https://github.com/nsmith-/vibegraph/commit/4178366", title: "MadGraph citations at the 3.7.1 pin; split Cuts::compile_with"}
  - {id: c7b780a, resource: "https://github.com/nsmith-/vibegraph/commit/c7b780a", title: "gate sigma recovery on the run's own error; split integrate_channels"}
  - {id: 02e6d05, resource: "https://github.com/nsmith-/vibegraph/commit/02e6d05", title: "the overweight test runs 2M trials"}
  - {id: 2187c59, resource: "https://github.com/nsmith-/vibegraph/commit/2187c59", title: "two doc warnings removed"}
---
The dev agent's report, condensed by the manager. Logs are prefixed `fd-` in
the scratchpad; the main ones are `fd-mutations.log` and `fd-sweep.log`.

## Fixed

| finding or item | commit | change | evidence |
|---|---|---|---|
| R-D.2, R-D.3 | 9c5336f | The test-only `(iteration, chunk)` scheme deleted with its three tests, including the vacuous agreement test | — |
| R-D.8 | 9c5336f | `adapt_parallel_seeded` is a test-only loop of `adapt_blocks_iteration`; `run_iter_seeded` deleted; one `moments()` helper | Golden, the seeded-vs-sequential test and `adapt_grids` reproduction all bit-identical (`to_bits`) |
| R-D.1 | 9c5336f, 9bc479e | Contracts moved to `adapt_blocks_iteration`/`warmup()`; setters and wrappers deleted; six items `cfg(any(test, doc))` → `cfg(test)` | — |
| R-D.6, .7, .10, .16 | 9c5336f, 9bc479e | Docs describe the current code; one redundant golden removed; history comments rewritten | — |
| R-D.11 | 9bc479e | `ProcessShape::moving_splits` removed | — |
| R-D.14 | 9bc479e | Shared `assert_normalized` | — |
| R-D.15 | 4178366 | `Cuts::compile_with` 161 → 77 lines (code motion) | — |
| R-D.13 | c7b780a | `integrate_channels` 424 → 242 lines | `adapt_grids` reproduction bit-identical |
| R-D.5 | c7b780a | New test: the stop reads the widened error, not the quoted one | Mutating the stop to read the quoted error fails it |
| R-D.9 | c7b780a, 02e6d05 | Unweighting and VEGAS smoke tests gated at 5σ of the run's own error; quoted relative error capped at 1% | Five mutations fail. A 20-seed sweep gives max \|pull\| 1.45–2.98 against the gate of 5 |
| sampler-and-phase-space-doc-comments-stale | 9c5336f, 9bc479e, 4178366 | Every listed site corrected (RAMBO now cites Kleiss, Stirling and Ellis) | — |
| madgraph-line-citations-predate-pin | 4178366 | Citations checked against the pinned tree, including the test-file SCALUP/AQCDUP sites | — |

**A lesson inside R-D.9:** at 400k trials the overweight test missed a 1.5%
mutation (about 3σ). It now runs 2M trials, which makes it about 7σ.
Reasoning from the error before running would have predicted the miss.

## Gate (agent, at 2187c59)

- **fmt** and **clippy, both configurations:** pass.
- **`cargo test --workspace`:** 1350 passed, 16 ignored (−4 vegas tests, +1
  budget test).
- **`cargo doc`, per package:** 8 lib warnings (none new) and 1 bin warning.
- **Banked:** `validate_vegas` 3, `validate_unweighting` 1, `validate_sigma`
  7 (plus 31 ignored). No σ row bound changed.

## Found

1. **Kb sites made stale by R-D.3 and R-D.8:**
   `phase-space/rng-substreams-and-parallel-determinism.md:16,88-92,128-129,159`
   and `phase-space/vegas-integrator.md:112-119`. Close-out.
2. **`cuts.rs`'s `myamp.f` and `setcuts.f:431` citations drift at the pin.**
   They are not in the claimed item. File them as an extension.
3. **`validate_scales.rs:9` still cites `unwgt.f:686`.** It belongs to
   validate-scales-module-doc-stale (F-G2).
4. **A pre-existing unresolved link at `phasespace/channel.rs:774`.**
5. **The sampler-tuning docs now say they predate the pooled stop.** That is
   already filed as sampler-tunings-predate-pooled-stop.

## Brief corrections

- **The `rng.rs` module doc was also made stale by R-D.3.** It was fixed.
- **The brief listed no kb sites** for R-D.3.

## Manager check (2026-10-10)

- **Commits and trailers:** six commits, `Assisted-by` only. The diff is
  18 files, +842/−893.
- **Deletions:** no `adapt_parallel`, `run_iter_parallel` or `moving_splits`
  remains in `src/`.
- **Gate re-run:** fmt, both clippy configurations, the hermetic suite
  (including the bit-identical VEGAS tests) and `validate_vegas`. The results
  are in `log.md`.
