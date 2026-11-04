---
type: Session Brief
title: "M-V1: the bit-level oracle in the tree, the CMS fetch task, and the item closed on independent measurements"
description: "Commit an ignored test that carries the precomputed-table interpolator as an oracle over the CMS sets, a script that fetches the 880 members, and re-measure RSS and the CMS gate independently."
status: draft
agent: validation-dev (Opus)
depends_on: [M-P1]
closes: [pdf-members-exceed-cms-memory-slot]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
---
Worktree `/home/user/wt/M`, branch `sprintf-pdf-memory` (draft PR #86). Start
at the commit that adds this brief; M-P1's commit, 428c4d8, is below it. Read
the [M-P1 report](M-P1-report.md) and the user's
[D1](../decisions/D1-lossless-knots-first.md). M-P1's scratch harness, its
baseline binaries and its logs are in
`/tmp/claude-0/-home-user-vibegraph/7497b6d2-0784-5f2c-a2c9-2ed0beaa757e/scratchpad/M-P1/`.
The 880 CMS members are in `/home/user/cms-pdfs`. Pixi is not installed. The
crate packages are `vibegraph-lib` and `vibegraph` (the CLI). This brief may
be wrong: if the code says otherwise, say so in the report.

## 1. The bit-level oracle in the tree (M-P1's Found)

Add an `#[ignore]` test that carries the precomputed-table interpolator as
an oracle and compares it bit for bit against the production `LogBicubic`:
- the oracle reproduces the removed `coeffs` construction (from 86b8e5c);
- the points are knots, seams with one ulp either side, random points, and
  extrapolation;
- it runs over every member of a set tree named by an environment variable
  (the CMS tree), and over NNPDF23 when that set is present;
- it reports how many values it compared, and it fails on zero.

Show three things:
- it passes on the 880 members;
- it catches a one-ulp mutation of the production path;
- it cannot pass vacuously (count asserted).

Wire it to the `extended-validation` feature or tier the way the other
`#[ignore]` CMS test is, and update the `extended-validation` skill's map if
it lists such gates.

## 2. Fetching the 880 members (D4, S-V1's FO)

Bring the manager's scratch fetcher into the tree, in the repository's
fetch-script conventions (`validation/fetch_common.sh`: consent, sha256
checks):
- it reads `pdfsets.json` from a `run_01_cms` directory;
- it checks each archive's sha256;
- it keeps the `.info` file and the members the list reads, and fetches
  `pdfsets.index`;
- it is resumable.

The fetcher is at
`/tmp/claude-0/-home-user-vibegraph/7497b6d2-0784-5f2c-a2c9-2ed0beaa757e/scratchpad/tools/fetch_cms.py`.
Add a pixi task for it, a pixi task that runs the two CMS gates, and the
`pixi.toml` entries in the existing style. You cannot run pixi here, so say
so. Test the script by fetching into a fresh directory, at least three sets
(say which), and diffing against `/home/user/cms-pdfs`.

## 3. Independent closing measurements

- Peak RSS on the item's command (`M-P1-gen.sh`), committed build, at `-j 1`
  and `-j 4`, five runs each. Use `/usr/bin/time -v` or a `getrusage`
  wrapper of your own, not M-P1's, and record which.
- The CMS-arguments gate, re-run.
- One control:
  - a build where a single member keeps its old tables shows up in the
    bytes-per-member guard;
  - or the oracle test catches a member read through the wrong band;
  - your choice, with the evidence.
- Set the item's `closes_when` against these numbers in the report,
  clause by clause.

Update `research/kb/events/systematics-weights.md` §4.4. The memory and
per-member figures are now measured, and the layout description changed;
point to this sprint. Gates: fmt, clippy (default and extended features),
the default `cargo test`.

**Report.** `M-V1-report.md` (type Session Report), with commands, outputs
and Found. Commit as described in your agent definition. Do not push. Do not
delete the backlog item; the manager does.
