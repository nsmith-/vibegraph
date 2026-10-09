---
type: Design
title: The per-process x category validation report and its collator
description: "Four per-process categories (diagrams, amplitudes, integrals, samples) plus standalone gates; a cell is the worst recorded measurement and nothing is inferred from a passing suite."
status: draft
tags: [validation, report, collator, manifest]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n25-reframe, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/25-validation-layering-plan.md#L15-L39", title: "Note 25 §1 (the reframing)"}
  - {id: n25-categories, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/25-validation-layering-plan.md#L112-L195", title: "Note 25 §3 (categories and the report table)"}
  - {id: n25-driver, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/25-validation-layering-plan.md#L404-L412", title: "Note 25 §5.7 (the report driver)"}
  - {id: n25-report, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/25-validation-layering-plan.md#L622-L677", title: "Note 25 §10 (the report: three rules)"}
  - {id: collator, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation-report/src/main.rs#L1-L64", title: "validation-report/src/main.rs module docs"}
  - {id: render, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation-report/src/render.rs#L310-L420", title: "validation-report/src/render.rs standalone verdicts"}
---

`pixi run validate` ends by running the collator (`validation-report`, a Rust
dev binary; `pixi run validation-report` runs it alone).[^n25-driver] It reads every row
file the gates wrote under `target/validation-report/`, renders
`report.md` and `report.json`, and checks that what was measured is exactly
what [`validation/manifest.toml`](process-manifest.md) declares.[^collator]
CI's `banked` job uploads the report as an artifact. The table makes
per-process coverage, and deliberate non-coverage, auditable in one
place.[^n25-reframe]

This concept describes the collator's rules. The table itself changes with
every row added and is regenerated on each run; read it from a fresh
`pixi run validate`, not from a note.

## The four categories

| category | what is compared | metric | concept |
|---|---|---|---|
| `diagrams` | diagram counts against MadGraph's `NGRAPHS`, one representative per subprocess class (per-concrete-subprocess matching is the design goal, still open; below) | `k/n` diagrams | [structural censuses](structural-censuses.md) |
| `amplitudes` | per point × per helicity × per colour flow complex amplitudes, at MadGraph's own banked events projected on shell | max relative deviation | [amplitude oracle](amplitude-oracle.md) |
| `integrals` | σ against banked MadGraph, always through the generic path (no special-cased integrand) | pull and seed-sweep stability | [σ gate](sigma-gate.md) |
| `samples` | unweighted-event distributions against MadGraph's banked samples | minimum KS / χ² p-value | [samples gate](samples-gate.md) |

Two details from the design carry into reading cells:[^n25-categories]

- `amplitudes` evaluates at MadGraph's banked events rather than a self-chosen
  grid, so the check sits where the cross section lives (peaks, cuts, low
  `m_ll`). A cell may carry `factorized`: whether its flow comparison had to
  weaken from the full per-helicity × per-flow outer product to its two
  projections, the allowance a flow basis differing from MadGraph's would need.
  It is stated wherever the question was asked, so "not needed" is on the
  record.
- `diagrams` on a multi-channel row means the per-flavour union of concrete
  subprocesses, or an explicit `covered-by` pointing at single-channel rows.
  Per-flavour matching of the union is open:
  [per-flavour diagram union unmatched](../backlog/validation/per-flavour-diagram-union-unmatched.md).

## How a cell gets its mark

The manifest's tier decides whether a row file is expected; the row file
decides the mark ([layers and tiers](validation-layers.md)):[^collator]

| tier | expects a row file | mark |
|---|---|---|
| `hermetic`, `banked` | yes | ✅ gate passed, ⚠️ informational, ❌ failed |
| `long` | only when its own driver ran this cycle | ⏳ otherwise; rendered like a banked cell when it ran |
| `blocked` | no | ⛔, naming the blocker |
| `covered-by` | no | `—`, naming the covering rows |
| `uncovered` | no | `uncovered`, with the manifest's account of the gap |

The collator exits nonzero on:[^n25-report][^collator]

- a cell the manifest declares measured and no gate wrote (missing, not empty);
- a row file for a cell the manifest says is not measured (unexpected);
- a gate cell that failed;
- a measurement whose `mode` (`gate`/`info`) or `factorized` disagrees with the
  declaration, or whose status is inconsistent with its mode;
- a declared tier with no mode;
- a standalone row with a layer outside `hermetic`/`banked`/`oracle`, or an
  `oracle` row that names no task.

One exemption: a row marked `bundled = false` (its banked run exists locally
but not yet in the pinned bundle) renders ⏳ "awaiting the bundle" when nothing
measured it, because a fetching checkout legitimately has no run to read.

## Three rules

These are stated in the binary's own docs, and each closes a specific way a
green cell could lie.[^n25-report][^collator]

1. **Nothing is inferred.** A hermetic cell could have been marked green from
   its tier plus "the hermetic suite passed". That prints a cell no
   measurement stands behind, and keeps printing it after the gate stops
   covering the row. So `amplitude_oracle` and `validate_madgraph_diagrams`
   write row files like every other gate, and a hermetic cell without one is a
   missing cell. `validate.sh` clears the per-category row files before the
   gates run, so a stale file cannot stand in for this run's measurement. This
   is the report-level form of the `AGENTS.md` rule that a report is evidence
   only if every green cell is a recorded measurement; see also
   [no-change claims](no-change-claims.md).
2. **The worst measurement is the cell.** A process measured more than once
   (`pp_to_ll` on two run cards, written as `pp_to_ll__default` and
   `pp_to_ll__mmll_60_120`) shows the worst value (largest `|pull|`, smallest
   p, largest deviation) and lists every measurement beside it, so a cell
   cannot be made green by adding an easier variant.
3. **A standalone gate with its own driver is read from its own file.**

## Standalone gates

Process-independent gates are listed under the table with their layer: the
α_s grid and `AQCDUP` replay, the PDF-grid oracle, HELAS kernels, the RAMBO
fixture, run-card defaults, the LHEF byte round trip, per-event scale replay,
the structural censuses, and others (the `[[standalone]]` rows of the
manifest are the list).[^n25-categories] Their verdicts come from three
places:[^render]

- A Rust-test gate in the layer this run drove writes no file. It ran inside
  the same `cargo test` as the cells, so its failure fails the suite before the
  collator runs; the list records what it is, not a fresh verdict.
- An `oracle` gate without a row file renders as "the oracle layer runs it",
  with its `pixi run` command, so a gate this invocation did not run cannot
  read as one that did.
- A gate with its own driver and row file, such as Pythia consumption
  (`pixi run validate-pythia`, its own pixi environment; see
  [Pythia interop](../events/pythia-interop.md)), is rendered from
  `standalone/<row>.json` when present. Absent, it reads "not run in this
  invocation", which is not a failure: nothing marks a standalone gate as
  required here. When the file is older than this invocation's row files, the
  verdict says it was recorded earlier and names the task that refreshes it.

Pythia consumption is a format and consumability property of each emitted
sample, not a physics distribution, which is why it is a standalone gate and
not a `samples` cell.[^n25-categories]

## Related

How a gate's verdict should be fixed before measuring is
[pre-registered verdicts](pre-registered-verdicts.md).

[^n25-reframe]: Note 25 §1.
[^n25-categories]: Note 25 §3.1–3.6.
[^n25-driver]: Note 25 §5.7.
[^n25-report]: Note 25 §10, "The report".
[^collator]: `validation-report/src/main.rs`, module docs and `resolve_cell`.
[^render]: `validation-report/src/render.rs`, `standalone_verdict`.
