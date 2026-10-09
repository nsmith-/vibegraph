---
type: Design
title: Hermetic, banked and oracle validation layers
description: "Each test declares by its registration which external inputs it may assume; banked gates fail hard on a missing input, and oracle gates are #[ignore] plus a pixi task."
status: draft
tags: [validation, layers, ci, testing, manifest]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n25-reframe, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/25-validation-layering-plan.md#L15-L111", title: "Note 25 §1–2 (the reframing and the three layers)"}
  - {id: n25-registration, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/25-validation-layering-plan.md#L236-L242", title: "Note 25 §4.2 (registration in N places)"}
  - {id: n25-decisions, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/25-validation-layering-plan.md#L561-L621", title: "Note 25 §9–10 (decisions and close-out)"}
  - {id: n29-e, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L1522-L1806", title: "Note 29 chain E (kT replay to the oracle layer; diagrams.json selector)"}
  - {id: n32-closeout, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L638-L748", title: "Note 32 §5.3 (validate census and wall-time reading)"}
  - {id: manifest, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/manifest.toml#L1-L60", title: "validation/manifest.toml, the Layers block"}
  - {id: validation-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/validation.rs#L1-L40", title: "vibegraph::validation::require"}
  - {id: ci, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/.github/workflows/ci.yml#L1-L20", title: "ci.yml header: hermetic and banked jobs"}
---

The validation suite needs very different external inputs: nothing at all,
the `mg5amcnlo` submodule and fetched data, or a working MadGraph toolchain.
The layering makes each test state which of these it may assume, and makes
that statement structural rather than a runtime check.[^n25-reframe] The
binding instruction for which gate to run after which change is the
`extended-validation` skill (`.agents/skills/extended-validation/`); this
concept explains the contract the skill relies on.

## The three layers

| layer | may assume | registered as | driven by |
|---|---|---|---|
| `hermetic` | nothing: no submodule, no network, no pixi env | default `cargo test` | `cargo test`; CI job `test` |
| `banked` | the `mg5amcnlo` submodule, the fetched PDF sets, the pinned [refdata bundle](refdata-bundle.md); never *runs* MadGraph | `required-features = ["extended-validation"]` in the crate's `Cargo.toml` | `pixi run validate`; CI job `banked` |
| `oracle` | the full toolchain: `mg5_aMC`, LHAPDF C++, gfortran/f2py | `#[ignore = "..."]` plus a pixi task passing `--ignored` | `pixi run generate-references`, `pixi run validate-deep` (an inventory of the oracle tasks), and per-gate tasks |

The names describe the dependency contract, which is the thing the layering
exists to make visible; cadence-based names (`commit`/`merge`/`release`) were
rejected for that reason.[^n25-reframe] `cargo test` keeps its name because
its name is its contract.[^n25-reframe]

The design budgets were about 3 minutes wall for the hermetic hook (`fmt`,
build delta and tests) and about 15 minutes for the banked layer; a gate that
would break its layer's budget moves to the oracle layer rather than
stretching it.[^n25-reframe]

**Hermetic.** Complete on a bare clone: every test the default-feature build
registers runs and asserts, and none skips.[^ci] Small, stable references
(roughly 100 KB or less) are generated once by the oracle toolchain, projected
down and committed, e.g. `sigma_reference.json`, `diagrams.json`,
`alphas/reference.csv`. A gate whose reference generation is expensive but
whose vibegraph side is quick belongs here.[^n25-reframe]

**Banked.** Everything MadGraph produced that this layer reads is fetched or
committed, never regenerated, which is what makes it runnable on a fresh CI
runner and a merge gate.[^ci] It takes **no runtime skips**: `pixi run
validate`'s dependency tasks acquire the submodule, the PDF sets and the bundle
and fail if they cannot, so a banked gate that still finds an input missing
calls `vibegraph::validation::require`, which panics naming the input and the
task that fetches it. There is no tolerated-skip list.[^validation-rs]
Heavy gates compile under `[profile.release-debug]` (release plus `debug = 1`,
thin LTO), which serves both the profiler and the gates.[^n25-reframe]

**Oracle.** Reference generation (with `--skip-deps` semantics: never
regenerate what the local work area already holds), plus vibegraph-side runs
too heavy for the banked budget: the 2→6 σ rows, the budget ladders behind
enforced σ budgets, and the kT clustering replay.[^n25-reframe] An oracle gate
whose input is absent fails with a plain `assert!`/`panic!` naming the input
and the task that builds it, not with `require`, whose message points at
`pixi run validate`, a layer that does not run it.[^n29-e]

### The `--lib` exception

A library unit test lives in the `--lib` target, which has no per-test
registration, so the few `--lib` tests that need banked inputs carry
`#[cfg(feature = "extended-validation")]`. Integration tests never
do.[^manifest] This is the only place a feature gate inside the code decides
layer membership.

## Why registration, not runtime checks

Before the layering, three mechanisms coexisted: `required-features`, an
internal `#[cfg(feature = ...)]`, and a runtime soft-skip that printed and
returned. The third made green mean "data absent": `validate-pdf-grid`
covered nothing for four sessions before anyone noticed.[^n25-reframe] A test
that decides at runtime whether it has anything to do reports "ok" having
asserted nothing, and an absent input becomes indistinguishable from agreement
with it.[^validation-rs] For what a green CI run therefore does and does not
cover, see [CI coverage](../tooling/ci-coverage.md).

## Cell tiers in the manifest

The [process manifest](process-manifest.md) records, per process and per
category, the **tier** a cell's gate runs in. The rule is: *the tier is where
the gate is registered*, not the weakest layer it could run in.[^manifest]

- `hermetic` / `banked` / `long` — the layer the cell's gate runs in. `long` is
  the oracle layer entered for cost rather than dependencies (the 2→6 rows,
  the MLM rows).
- `blocked`, `covered-by`, `uncovered` — not measured here, with a blocker, the
  covering rows, or an account of the gap.

Two worked cases:

- `validate_madgraph_diagrams` reads committed files only (about 1.3 s), so it
  carries no `required-features` (`vibegraph-lib/Cargo.toml`, `[[test]] name =
  "validate_madgraph_diagrams"`) and every measured `diagrams` cell is
  `hermetic`. The cells it does not measure are `covered-by` or `uncovered`, not
  a weaker tier.[^n25-decisions] Its committed reference
  `validation/madgraph/diagrams.json` holds exactly the rows the manifest
  declares `diagrams`-hermetic: `extract_diagrams.py` selects rows from the
  manifest rather than from whichever work-area directories exist, and the
  hermetic test `diagrams_json_covers_exactly_the_hermetic_rows` asserts the set
  equality.[^n29-e] Regenerating it needs `pixi run --skip-deps -e madgraph
  extract-diagrams`; without `--skip-deps` the task's `build-diagrams`
  dependency regenerates any missing process directory through MadGraph.
- The kT clustering replay (`validate_kt_cluster`) keeps `required-features`
  but is `#[ignore]`d and run by `pixi run -e madgraph validate-kt-cluster`,
  because its 75 MB of dumps are deliberately outside the reference bundle. In
  the banked layer it could only have been green without comparing anything.
  Its manifest `[[standalone]]` row declares `layer = "oracle"` and a `task`;
  the collator rejects a standalone layer outside `hermetic`/`banked`/`oracle`
  and an `oracle` row with no task.[^n29-e] See
  [the kT dump oracle](kt-cluster-dump-oracle.md).

## Per-process coverage, and MadGraph's grouping

The second half of the design is that coverage, including deliberate
non-coverage, is stated per process in one place: the manifest, rendered by the
[validation report](validation-report.md). Before it, adding a process meant
touching `build_amplitude.sh`, `gen_amplitude.py`, reference JSONs and two or
three Rust plan tables.[^n25-registration]

A principle behind the per-row choices: **MadGraph's process grouping is
optimisation-inspired, not physics-motivated, and we owe agreement with it only
where it affects our ability to validate.** Per-helicity amplitude checks run
on single concrete subprocesses precisely so that MadGraph's grouping need not
be reproduced.[^n25-reframe]

## Reading the census line

`pixi run validate` ends with the collator's census (rows × categories, how
many measured, by mark). That census counts only the layers the invocation
drove, so `long` cells read ⏳ unless their own driver ran in the same cycle:
`validate.sh` clears the per-category row files first, so a `long` cell is
rendered as measured only when, for example, `pixi run validate-sigma-2to6`
ran before `pixi run validation-report` in the same tree. Two censuses that
differ only in their `long` cells are not in tension.[^n32-closeout]

`validate` runs its rows concurrently, so a CPU saving shows in wall time only
to the extent the run is CPU-bound; on a loaded host wall time can hide a real
CPU saving entirely. Compare CPU and wall separately, on a quiet host. Current
timings are in
[validation-layer timings](../performance/validation-layer-timings.md).[^n32-closeout]

[^n25-reframe]: Note 25 §1–2.
[^n25-registration]: Note 25 §4.2.
[^n25-decisions]: Note 25 §9–10; the `diagrams` registration move is recorded in note 27 B5.
[^n29-e]: Note 29 chain E, items E.2 and E.4.
[^n32-closeout]: Note 32 §5.3.
[^manifest]: `validation/manifest.toml`, `## Layers` and `## Categories and cell tiers`.
[^validation-rs]: `vibegraph-lib/src/validation.rs`.
[^ci]: `.github/workflows/ci.yml`, header comment.
