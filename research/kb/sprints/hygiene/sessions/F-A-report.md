---
type: Session Report
title: "F-A report: evaluator fixes"
description: "All 16 R-A findings and evaluator-doc-comments-stale fixed in helas/eval over 7 commits, with the emitted Program proven identical on 45 subprocesses; the R-A.13 arena bound turned out false at the base."
status: draft
tags: [hygiene, fix, evaluator, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-10}]
sources:
  - {id: 9bb2849, resource: "https://github.com/nsmith-/vibegraph/commit/9bb2849", title: "split long compile-time builders, pin the op-kind numbering"}
  - {id: ac5b95d, resource: "https://github.com/nsmith-/vibegraph/commit/ac5b95d", title: "split compile, move the colour-flow selection contract"}
  - {id: 7db1698, resource: "https://github.com/nsmith-/vibegraph/commit/7db1698", title: "drop dead tree/analysis utilities, correct stale evaluator docs"}
  - {id: 46c28f0, resource: "https://github.com/nsmith-/vibegraph/commit/46c28f0", title: "restore the fold_recursive test"}
  - {id: c01b93a, resource: "https://github.com/nsmith-/vibegraph/commit/c01b93a", title: "relative HELAS current gates, the expansion sharing check"}
  - {id: 6619227, resource: "https://github.com/nsmith-/vibegraph/commit/6619227", title: "clippy useless_vec"}
  - {id: 5c92658, resource: "https://github.com/nsmith-/vibegraph/commit/5c92658", title: "every intra-doc link in helas/eval resolves"}
---
The dev agent's report, condensed by the manager. Everything except the
manager check is the agent's claim. The session was interrupted by two
container restarts and resumed each time from its transcript.

## Fixed

| finding | commit | change | evidence |
|---|---|---|---|
| R-A.1 | c01b93a | jioxxx current and FFV1 amplitude checked relatively, at 1e-12 of the largest reference | Mutation (zero Z current) fails the new test and passes the old one. \|J_Z\| ≈ 5.2e-5 |
| R-A.2 (docs) | c01b93a | Probe docs say test-only `lower::lower` plus the generic slot pass | — |
| R-A.4, item | 7db1698, 5c92658 | Module map rewritten; all 14 item sites fixed | `cargo doc`: no broken link in `helas/eval`; lib warnings 28 → 14 |
| R-A.5 | 7db1698 | Module-wide allows removed; dead `tree`/`analysis` utilities deleted; `analyze_sym` → `cfg(test)` | — |
| R-A.6 | 9bb2849 | Strum `InstrKind` derives `N_KINDS`/`kind_name`; numbering unchanged; bijection test added | Renumbering `AddScaled` fails the test |
| R-A.7 | 9bb2849 | `Op::is_pool_leaf`; one `readout_combinations` walk | Program identical |
| R-A.8 | 9bb2849 | `build_at_leg` 289 → 106 lines | Program identical |
| R-A.9 | 9bb2849 | `Baker` context struct; `too_many_arguments` allow gone | Program identical |
| R-A.10 | ac5b95d | `compile` 229 → 92 lines, seven helpers | Program identical |
| R-A.12 | ac5b95d | Test-only `select_color_flow` deleted; contract on `select_config_and_flow`; links in `hadronic.rs`, `proton.rs`, `flow_tags.rs` and the book retargeted | — |
| R-A.13 | c01b93a | Ignore reasons added; `expansion_shares_nodes` in the default suite | Disabling hash-consing fails it |
| R-A.14 | c01b93a | Z path checked on four components at 1e-6 | `vg_ez[2] *= 1.01` fails the new test and passes the old one |
| R-A.15 | ac5b95d | Binary add/mul invariant checked on `lower_flows` | — |
| R-A.16 | 7db1698 | `GammaJout` → `GammaOout`; dead probe reference removed | — |
| R-A.19 | 7db1698 | αs study test deleted; its result is in the kb | — |
| R-A.20 | 7db1698 | Tolerance is `CLIFFORD_TOL` | A 5e-14 perturbation fails the new bound and passes the old one |

**Program identity:** a temporary probe, never committed. It hashes the
`Debug` text of `Folded::program()`, `folded_hel().program()` and the folded
arena for 45 subprocesses: the 19 MadGraph-validated processes, four QCD
2→3, and all 22 SMEFTsim and toy manifest rows. The tree after the refactors
matched a twice-stable baseline. Later commits touch only tests and docs.

**Mistake caught:** `7db1698` deleted the `fold_recursive` test along with the
dead utilities' tests, and `46c28f0` restored it.

## Stopped (to file)

- **`expansion_bounds_arenas` fails at the base on 3 of 4 processes.** Peak
  live slots are at 72–83% of nodes, against the asserted bound of half the
  nodes, because read-out scalars are pinned live. It stays ignored with a
  reason. A new bound needs re-derivation.

## Gate (agent)

- **fmt** and **clippy, both configurations:** pass at `5c92658`.
- **`cargo test --workspace`:** 32 suites ok at `6619227`. The lib suite has
  973 tests run (989 at base, after deletions and additions).
- **`cargo doc --document-private-items`:** passes.
- **Banked `color_cf_oracle`:** 97 passed.

## Resume and environment

- **Restart 1:** the uncommitted work was intact. The agent committed it in
  four checkpoints, each checked in isolation.
- **Restart 2:** the gate script was killed. The agent fixed a clippy
  `useless_vec` and re-ran the gate with `CARGO_INCREMENTAL=0`.
- **Disk:** it filled to 287 MB free during the test run. The agent deleted
  stale test executables (about 25 GB).

## Found

1. **The `expansion_bounds_arenas` bound is false at the base** (Stopped).
2. **Kb sites:**
   - `events/colour-and-helicity-selection.md:62` and
     `phase-space/multichannel.md:182` name the deleted `select_color_flow`;
   - `per-diagram-amp2.md:224` describes the contradiction this session fixed;
   - `helicity-expansion.md` says 149k live slots, where 162 188 was measured.
3. **`helas/mod.rs:261` still says `GammaJout`.** This belongs to F-B.
4. **Plan references in comments:** "SESSION 6b" (`run.rs` ~:2745) and
   "note 16 §6" (`root_diagram.rs`).
5. **14 lib doc warnings remain outside `helas/eval`.**
6. **`test_eval_jioxxx` checks only the FFV1 amplitude sink.** The chiral sinks
   need a per-diagram `iovxxx` check.

## Brief corrections

- **Neither the op census nor a dump could prove the program unchanged.** A
  `Debug` digest works, provided it leaves out the `HashMap`-bearing analysis.
- **R-A.13's gate was false, not only ignored.**
- **R-A.14's test is built only under `extended-validation`.**

## Manager check (2026-10-10)

- **Commits and trailers:** seven commits, `Assisted-by` only. The diff is
  29 files, +865/−1069.
- **`kernel.rs`:** the diff touches one comment and one test tolerance only.
- **Gate re-run:** the probe at the final commit, both clippy
  configurations and the hermetic suite. The results are in `log.md`.
