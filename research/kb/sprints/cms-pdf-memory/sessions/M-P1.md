---
type: Session Brief
title: "M-P1: CMS's 880 PDF members under 1.5 GB, losslessly"
description: "Attribute the 3.76 GB first; then store each member as its knots and build the x cubic on use, bit-identical to today's interpolation; pin bytes per member in CI; measure RSS and time on the CMS request."
status: draft
agent: performance-dev (Opus)
depends_on: []
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
---
Worktree `/home/user/wt/M`, branch `sprintf-pdf-memory` (draft PR #86). Start
at the commit that adds this brief. Read the [sprint](../sprint.md), the user's
[decision D1](../decisions/D1-lossless-knots-first.md), the item
(`python3 scripts/kb.py backlog --item pdf-members-exceed-cms-memory-slot`),
§4.4 of `research/kb/events/systematics-weights.md`, and §5 of
`research/kb/sprints/systematics/sessions/S-V1-report.md`. This brief may be
wrong: if the code says otherwise, say so in the report.

**Environment.**
- Linux container, 4 cores, 15 GB RAM, about 25 GB free disk. Pixi is not
  installed; run the cargo commands that pixi tasks wrap.
- Build with
  `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.
- A sibling session builds in `/home/user/wt/G` at the same time, so expect
  CPU contention and record it next to every timing.
- Reference data is hard-linked into the worktree; never write into it:
  `validation/madgraph/output` is refdata-13, `research/refs/mg5amcnlo` is the
  SM UFO at b7687064, and `validation/pdf/NNPDF23_lo_as_0130_qed` is present.
- The 43 CMS sets are being fetched into `/home/user/cms-pdfs`, an LHAPDF tree
  with `pdfsets.index` that holds only the members the list reads. A set is
  complete when `<set>/.done` exists. The fetch log ends `ALL_DONE`:
  `/tmp/claude-0/-home-user-vibegraph/7497b6d2-0784-5f2c-a2c9-2ed0beaa757e/scratchpad/logs/fetch_cms.log`.
  Point `SYSTEMATICS_CMS_PDFS` and `LHAPDF_DATA_PATH` at it.

## 1. Measure first (stop-and-report condition)

Before you change any code, build the current code and keep its `generate`
binary aside as the baseline. Then measure the item's command:
- `generate`, `pp_to_ll`, lhaid 325300, 10 000 events, the CMS
  `--systematics` arguments from
  `validation/madgraph/output/pp_to_ll/Events/run_01_cms/command.txt`;
- peak RSS from the child's `getrusage`, wall and user time, at `-j 1` and
  `-j 4`.

Attribute the bytes:
- per member, by field: `coeffs`, `xf`, `qcells`, knots, everything else;
- transient parse buffers (whole `.dat` strings, intermediate `Vec`s);
- other copies of member data;
- glibc malloc arenas and fragmentation at `-j 4`.

Use a counting global allocator in a scratch binary, `heaptrack` or
`/proc/self/smaps_rollup`, whichever answers the question.

If less than about 60% of the peak is coefficient and knot storage that the
knot-only layout would remove, stop. Report the attribution and what would
reach 1.5 GB instead, and do not build the layout. The manager takes it to the
user.

## 2. The layout (the user's D1)

Store each member as its knots. Build the x-direction cubic on use, from the
same expressions in the same order as today's precomputation, so every
interpolated value is **bit-identical**. Drop every derived table that is
not needed. Keep the bilinear fallback for two-knot bands working from the
same knots. Keep what is small and saves time (e.g. the Q² stencil
constants) if it measurably pays. Free the parse transients before the next
member loads.

## 3. Gates

- **Bit-identical interpolation.** Compare the new interpolator against the
  current one, value for value with `to_bits`, on every flavour of every
  member of:
  - NNPDF23;
  - at least five CMS sets of different families, including a
    multi-Q²-subgrid set and one with a two-knot band if any exists (say
    which);
  - dense random points across the grid, knot-exact points, band seams, and
    the extrapolation regions.

  The comparison may be a scratch harness against the baseline commit's
  library, or an `#[ignore]` test that carries the old algorithm as an
  oracle. Say which, and why it cannot pass vacuously. A tolerance is not
  acceptable here: D1 promises bit-identical values.
- **CMS gate.** `cms_arguments_reproduce_madgraphs_weights` passes with
  `SYSTEMATICS_CMS_PDFS` set (it is `#[ignore]`; run it with `--ignored`,
  backgrounded). Also run the default `validate_systematics` tests and the PDF
  grid oracles (the `extended-validation` skill names them).
- **Guard (D4).** A default test pins heap bytes per loaded member against
  its knot count, so a member that again carries derived tables fails it.
  Show it fails on the baseline layout.
- **Memory (D3).** Peak RSS on the item's command at `-j 1` and `-j 4`, five
  repeats each, below 1.5 GB. Give every number with its command.
- **Time (D2).** Wall and user time on the same command against the baseline
  binary, same machine, interleaved runs. Also the per-member evaluation cost.
  The limit is 2× wall. If you exceed it, report the measurement rather than
  trading exactness for speed.
- `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test`.

## Report

`research/kb/sprints/cms-pdf-memory/sessions/M-P1-report.md` (type Session
Report). Every claim comes with its command and output: the attribution, the
gates, the tables, and a Found section. Commit as described in your agent
definition. Do not push; the manager pushes.
