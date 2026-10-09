---
type: Procedure
title: Generating and regenerating MadGraph reference runs
description: "Adding a run (script + manifest row), regenerating through generate-references and build.sh, the --skip-deps and missing-directory traps, what it costs, and the instrumented MLM replay."
status: draft
tags: [madgraph, references, pixi, regeneration, mlm]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n24-p0-bank, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L392-L445", title: "Note 24 §P0 outcome: what was banked"}
  - {id: n24-p0-files, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L637-L657", title: "Note 24 §P0 outcome: files and commands"}
  - {id: n30-regen, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/30-perf-baseline-timings.md#L253-L281", title: "Note 30 §4.3: the regeneration-cost answer"}
  - {id: n41-m0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L342-L440", title: "Note 41 §M0: MLM references and the instrumented replay"}
  - {id: n41-z, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L2958-L3000", title: "Note 41 §Z.3: banking refdata-9 on the bank host"}
  - {id: n41-b1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L3336-L3403", title: "Note 41 §B1: refdata-9 generated, reproduction check"}
  - {id: n41-writer, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L3234-L3247", title: "Note 41: write_mlm_sigma_reference.py"}
  - {id: fact-missing-dir, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/facts/extract-diagrams-reruns-madgraph-on-missing-dir.md#L11-L30", title: "Fact: MadGraph pixi tasks regenerate any missing run directory"}
  - {id: gen-refs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/generate_references.sh", title: "validation/generate_references.sh"}
  - {id: gen-mlm, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/madgraph/gen_mlm_references.sh", title: "validation/madgraph/gen_mlm_references.sh"}
---
# Generating and regenerating MadGraph reference runs

This is how MadGraph process directories and their runs are produced in the
work area `validation/madgraph/output/` (gitignored, ~1 GB) and reduced to the
committed reference files. Packaging the work area into a fetchable
`refdata-N` bundle is a separate step,
[refdata banking](../validation/refdata-banking-procedure.md). How a reference
run is designed and read is
[MadGraph reference runs](../validation/madgraph-reference-runs.md); the
environment every command below runs in is
[the MadGraph toolchain](madgraph-toolchain.md).

## Adding a run

1. Write `validation/madgraph/scripts/<name>.mg5`: the `import model`,
   `generate`, `output <name>` lines and a `launch` block whose `set` lines
   pin the run card. MadGraph writes the output directory name into
   `proc_card_mg5.dat`, so two otherwise identical scripts differ in that line.[^n24-p0-bank]
2. Add a row to `validation/manifest.toml`: the process string, the script,
   the categories and layer, and the reason the process is in the set. For an
   amplitude-oracle row, give it an `mg_amplitude` table.
   `gen_amplitude.py` reads that table, and `build_amplitude.sh` builds an f2py
   module for every row it resolves (and a probe module for every key
   `gen_amplitude_tables.py --dump-keys` prints), so there is no second registry
   to edit.
3. Generate: `pixi run -e madgraph build-diagrams` builds only scripts whose
   output directory does not yet exist. Then run the extraction the row needs
   (`extract-sigma`, `generate-amplitude`, …) or the `refs` stage below.
4. Commit the regenerated reference files the gate reads; `output/` itself is
   never committed.[^n24-p0-files]

A script whose run another generator makes carries a `# built-by: <generator>`
line, and `build.sh` skips it (the MLM rows: `gen_mlm_references.sh` needs the
samples-grade run to be the first its directory makes).

## The entry point: `generate-references`

`pixi run -e madgraph generate-references [stage …]` runs
`validation/generate_references.sh`, stages in order:

| stage | what | cached? |
|---|---|---|
| `deps` | the pinned submodule and the two LHAPDF sets, via `fetch_common.sh` | — |
| `madgraph` | the MadGraph runs (`build.sh`-equivalent generation plus each script's `launch`) | **the work area is the cache**: an existing process directory is never rebuilt, nor a cross-section run whose answer is written |
| `seeds` | MadEvent references committed as one run per seed (widths, decay-chain and `$` σ, decay-chain and `@N` event summaries, `>`/`$$`/polarized σ), in `validation/madgraph/work/` | an existing directory is never regenerated; a finished seed is read back (`madevent_seeds.sh`) |
| `mlm` | the five MLM rows (below) | as `seeds` |
| `refs` | every committed reference recomputed from the work area | **always reruns**: cheap pure functions of the work area, so a moved reference shows as a diff |
| `bundle` | the banked-reference archive (`assemble_bundle.sh`) | — |

`refs` needs a complete work area: the bundle trims process directories
(`coloramps.inc` and the rest of a buildable `SubProcesses`), so `refs` cannot
run on a work area unpacked from the bundle alone.[^n41-b1] The Fortran77 HELAS
grid is not reached from here; it needs the `helas-validation` environment.

Committed references are regenerated only when the banked phase-space points,
the process list or the pinned MadGraph version change, never to make a
failing gate pass. A reference that moves is a finding.

## Two traps: `--skip-deps` and a missing directory

Most validation pixi tasks chain `depends-on` steps that regenerate reference
data. The `extended-validation` skill states when to use
`pixi run --skip-deps <task>` (generated inputs known fresh) and when not to
(after a submodule bump, a run-card edit, a process-list change, or anything
under `validation/madgraph/`).

What that statement does not say: `extract-diagrams`, `extract-configs`,
`extract-sigma`, `build-amplitude`, `validate-color-cf`,
`validate-unweighting`, `validate-lhef` and others declare
`depends-on = ["build-diagrams"]`, and `build.sh` **regenerates every script
whose output directory is missing**, printing a quiet "Skipping" line for the
rest.[^fact-missing-dir] So:

- a run directory moved aside (to test a fetched-only work area, or to hold a
  run out) comes back as a fresh MadGraph job, possibly long, the next time
  any dependent task runs without `--skip-deps`; with a held-out run, use only
  `pixi run --skip-deps <task>`;
- a fresh worktree without `validation/madgraph/output` copied in triggers
  the same full regeneration (`AGENTS.md`, "Own the worktrees");
- a regenerated multi-group run such as `pp_to_jj` is a different valid event
  sample, not the banked bytes.

## What it costs

The `madgraph` stage for 31 process directories took about 17 minutes on the
M3 Max, warm-cache, and the bank host's full `deps madgraph seeds mlm refs`
for `refdata-9` took 415 minutes, most of it the MLM replays. These are dated
measurements owned by
[validation-layer timings](../performance/validation-layer-timings.md); quote
them from there. The `refs` stage's own cost has not been measured
separately.[^n30-regen]

## MLM references (`gen_mlm_references.sh`)

Task `generate-mlm-references` (the `mlm` stage), five rows: `pp_to_llj_mlm`,
`pp_to_llj_xqcut_only`, `pp_to_llj_mlm_alps2`, `pp_to_ll_0j2j_mlm`,
`pp_to_ttx_0j1j_mlm`. `MLM_STAGE=runs|dumps|census` runs one stage;
`ROWS=…`, `SEEDS=…`, `FRESH_ROWS`, `FRESH_SEEDS`, `NB_CORE` (default 2) and
`VG_FORCE=1` narrow or force.

- **`runs`.** Each row runs ten seeds (20260928–37 by default), each making
  the run card's full event count. The first seed is the samples-grade run in
  `output/<row>/Events/run_01`, always **the first run of a freshly generated
  directory**, because an instrumented replay reproduces a run only if that
  run was the first its directory made. The other seeds run one after another
  in one shared directory `work/mlm/<row>`; each inherits the grids of the
  runs before it, so they are not independent (seed χ²/dof 0.3–0.8). Rows in
  `FRESH_ROWS` (default `pp_to_ll_0j2j_mlm`) also run `FRESH_SEEDS`
  (20261101–20), each the only run of its own fresh directory, banked as
  `output/<row>/Events/run_s<seed>`; they are that row's σ reference. Why
  independent directories matter is the
  [seed policy](../validation/madevent-reference-seed-policy.md).
  Before any run, every `set` line of the script's launch block is checked
  against the committed `<row>_run_card.dat`; after each run, the banner is
  checked for `vector_size = 1`, because on the vector path `SCALUP` records
  the lowered matrix-element PDF scale instead of `q2bck`.
- **`write_mlm_sigma_reference.py`** writes `mlm_sigma_reference.json` from
  the seed records. It knows three kinds of run: `samples`, `fresh` and
  `sigma` (a shared-directory seed). A row with fresh runs is written with
  `independent_directories: true`, its shared seeds in a separate
  `shared_directory_runs` block that no gate reads.[^n41-writer]
- **`dumps`.** `gen_kt_cluster_dumps.sh` replays each samples-grade run
  through instrumented Fortran (`wrappers/ktdump*`), and **the replay must
  reproduce the banked event file byte for byte** or the stage stops. Output:
  `output/ktdump/dumps/<row>.jsonl.gz`, outside the bundle, pinned in
  `mlm_dump_manifest.json`. The record schema is documented field by field in
  the module docstring of `gen_kt_cluster_dumps.py`. A matched 2 → 3 flushes
  ~60 000 record sets per 10 000 kept events, gigabytes raw, so with
  `VG_KTDUMP_GZIP=1` the Fortran writes to a named pipe that a detached gzip
  drains, and `VG_KT_DROP_RAW=1` drops the raw shards after extraction.
  Replays took 8–62 minutes per row in a Linux container. The oracle the dumps
  feed is the [MLM dump oracle](../validation/mlm-dump-oracle.md).[^n41-m0]
- **`census`.** `dump_mlm_census.py` writes `mlm_census.json` (jet-ness mixed
  within an IPROC, the jet memo's re-cluster branches) from the dumps and the
  process directories.

**Reproducing across hosts.** Not every MLM row's MadEvent runs are
bit-reproducible across hosts: on the `refdata-9` re-bank,
`pp_to_llj_xqcut_only` and `pp_to_llj_mlm_alps2` were 10/10 bit-equal and
`pp_to_llj_mlm` 9/10, while `pp_to_ll_0j2j_mlm` and `pp_to_ttx_0j1j_mlm` were
not bit-equal.[^n41-b1] So:

- `mlm_sigma_reference.json` is checked value by value **within seed error**,
  not for equality;
- `mlm_census.json` is reproducible in its **findings** (no mixed jet-ness,
  every channel's memo single-valued, no restricted re-cluster on t t̄, no stale
  final-state `ipdgcl`), not in its bytes, because its event counts come from
  the non-reproducible rows;
- `mlm_dump_manifest.json` differs per host beyond `sha256` (gzip writes a
  timestamp) and the shard names; commit the regenerated file so it pins the
  dumps that host gated.

Every other committed table regenerated byte-identical on that re-bank. The
σ values and their gate are [the MLM σ gate](../validation/mlm-sigma-gate.md).
The general method for claiming nothing changed is
[no-change claims](../validation/no-change-claims.md).

## Linking

Every generator that builds `madevent` for a `pdlabel = lhapdf` run needs the
C++ runtime appended to `LDFLAGS`; the platform rules are in
[the toolchain concept](madgraph-toolchain.md#the-c-runtime-link-fix).

[^n24-p0-bank]: Note 24 §P0, "What was banked" (the `pp_to_llj_fixed` and amplitude banks).
[^n24-p0-files]: Note 24 §P0, "Files and commands".
[^n30-regen]: Note 30 §4.3, "The regeneration-cost answer".
[^n41-m0]: Note 41 §M0, what ran and the extended replay.
[^n41-writer]: Note 41 §Z, "How it was written, and how B1 reproduces it".
[^n41-b1]: Note 41 §B1, generation and the reproduction check.
[^fact-missing-dir]: `research/notes/facts/extract-diagrams-reruns-madgraph-on-missing-dir.md`.
