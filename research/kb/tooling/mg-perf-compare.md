---
type: Procedure
title: "mg_perf_compare: per-point timing against MadGraph MATRIX1"
description: "Joins MadGraph MATRIX1 ns/eval (host-labelled mg_timings.json) with criterion eval_m2 benches over manifest rows, reporting one-sided rows; what the ratio does and does not license."
status: draft
tags: [performance, madgraph, benchmark, timing]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n32-triage, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L114-L156", title: "Note 32 §1.2: the mg_perf_compare triage findings"}
  - {id: n32-s4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L478-L597", title: "Note 32 §5.1: per-session outcomes (the tool changes)"}
  - {id: n32-remeasure, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L794-L864", title: "Note 32 §6: MATRIX1 re-measurement"}
  - {id: script, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/scripts/mg_perf_compare.sh", title: "scripts/mg_perf_compare.sh"}
  - {id: bench, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/benches/eval_strategies.rs#L110-L165", title: "vibegraph-lib/benches/eval_strategies.rs, BENCH_ROWS"}
---
# mg_perf_compare: per-point timing against MadGraph MATRIX1

`scripts/mg_perf_compare.sh [--skip-bench]` measures the per-point cost of
vibegraph's evaluator against MadGraph's compiled `MATRIX1` and prints a
ratio table that can be compared across hosts. The results, dated and
host-labelled, are
[matrix element vs MadGraph](../performance/matrix-element-vs-madgraph.md);
this concept is the tool.

## The two sides

**MadGraph**: `MATRIX1` ns/eval from the `processes` table of
`mg_timings.json`, written by `pixi run -e madgraph generate-amplitude`, which
times each compiled f2py matrix-element module. The file carries a `host` block
(CPU, OS, toolchain including gfortran and the MadGraph version), and that
block, not the file's name or location, is the host identity. Sources, in
order:

1. the work area, `validation/madgraph/output/mg_timings.json`, generated on
   this machine; regenerate it before trusting a ratio;
2. otherwise the committed, host-labelled
   `validation/madgraph/mg_timings.json`, and the script says the ratios then
   compare two machines.

Which processes have a MATRIX1 module is not a list in the generator: every
`validation/manifest.toml` row with an `mg_amplitude` table gets one.

**vibegraph**: criterion median ns/iter of the `eval_m2/forward/*` rows of
`vibegraph-lib/benches/eval_strategies.rs`, divided by the bench's points per
iteration (parsed from the source). Never the `amplitude_oracle` timing report,
whose `extended-validation` build compiles per-node cross-checks into the
evaluation loop.[^n32-triage] The bench covers `BENCH_ROWS`, a fixed SM subset of the
manifest's `mg_amplitude` rows chosen to span the evaluator's shapes:

```rust
const BENCH_ROWS: &[&str] = &[
    "ee_to_mumu", "ee_to_wpwm", "uux_to_uux", "gg_to_gg", "gg_to_ttx",
    "ee_to_mumua", "ee_to_mumu_tata_qcd0", "uux_to_ccx_emmm_qcd0",
];
```

Each key's process string is read from that row's `mg_amplitude` table, so a
joined row compares the same card on both sides; a key naming a UFO model of
its own is rejected, since the bench builds the interned SM only.
`VIBEGRAPH_BENCH_EXTRA_PROCESSES="name=card;…"` adds rows for a study; the
joined set stays `BENCH_ROWS` unless it is set.

## The join

A process on only one side is **reported, not dropped**: an `mg_amplitude` row
outside the bench subset (or not yet re-benched), or a bench row with no
compiled module or timing entry. `--skip-bench` joins against existing
`target/criterion` results without re-running, and each row prints its
measurement mtime so a stale join is visible. A work-area `mg_timings.json`
older than the `host` block (no `processes` key) is named as such, rather than
read as an empty table.

Output: a host fingerprint and the table on stdout, plus
`target/mg-perf/mg_compare_<os>_<arch>.{md,tsv}`.

## What the ratio licenses

- **Same codegen on both sides.** Recorded tables use default codegen. If one
  side is raised (`RUSTFLAGS=-C target-cpu=native`), raise the other too
  (`-march=native` in `build_amplitude.sh`'s `--f77flags`), or the ratio does
  not compare like with like.
- **Same host, quiet host.** A geomean over the 19 MATRIX1 rows read 0.95×
  with sibling sessions resident on the host and 0.87× on a quiet host
  against a same-day MadGraph table.[^n32-remeasure] Contention moves the
  ratio by more than many optimisations do.
- **The row set is a sample.** A narrower row set once dropped the QCD-dense
  llj-class rows and read better (14-row geomean 1.06× against 0.95× for 19
  rows at the time). Report which rows were joined with any geomean; the
  one-sided list says what was left out.[^n32-s4]
- **Scalar only.** The forward bench rows are scalar evaluation; SIMD
  lane-width questions use the separate kit (`scripts/dump_lane_asm.sh`).
- **Per point, not per cross section.** It says nothing about integration
  efficiency or time to a given accuracy; that comparison is
  [integration vs MadGraph](../performance/integration-vs-madgraph.md).

Other timing instruments are in [profiling](profiling.md).

[^n32-triage]: Note 32 §1.2, the triage findings the current design answers.
[^n32-s4]: Note 32 §5.1, the session that moved the registry into the manifest, added the host block and one-sided reporting.
[^n32-remeasure]: Note 32 §6, MATRIX1 re-measurement on 2026-08-06.
