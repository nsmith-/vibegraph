---
type: Session Brief
title: "S-V1: the CMS-arguments gate, the MLM gates in CI, and FG"
description: "Gate the 926-weight CMS runs per event and on the banked header; make MadGraph's default-argument MLM runs a CI gate; fix the dump test's unmatched branch; explain or fix the permuted-P1 record differences (FG); measure memory and time on the 880 members."
status: draft
agent: validation-dev (Opus)
depends_on: [S-F2]
closes: [systematics-variation-weights-not-written]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-29}
---
Worktree `/home/user/wt/S`, branch `sprinte-systematics` (PR #83), at the
commit that adds this brief. refdata-12 is pinned (cd038fd1) and unpacked in
the worktree (hard links; never write into it). It adds:
- `Events/run_01_cms/` on `pp_to_ll`, `pp_to_jj`, `pp_to_ll_0j2j_mlm` and
  `pp_to_ttx_0j1j_mlm`: 926 weights, ids 1001–1926, the 43 `.info` files
  and `pdfsets.json`;
- `Events/run_01_syst_default/` on the four MLM rows: 147 weights;
- the matched rows' `nexternal.inc`, `configs.inc`, `config_nqcd.inc` and
  `config_subproc_map.inc`.

The MLM per-event dumps are **not** in the bundle (user's choice); the dump
check runs on the M3.

Pixi is not installed. Run the banked layer as separate, resumable, detached
targets (S-F1's `S-F1-banked2.sh` pattern). Read `S-F1-report.md`,
`S-F2-report.md` and the M3's two comments on #83 first.

## 1. The CMS-arguments gate (closes the item)

The item's `closes_when` asks that the weights for CMS's `runcmsgrid.sh`
arguments match MadGraph's `systematics` on a banked sample. Gate the four
`run_01_cms` files.
- Every `<wgt>` value matches MadGraph's string at one unit, ids in order.
- **The header is byte-identical to the banked block** (`ba7e21bc…`), not
  the design's predicted `2cbb8781…`. The difference is in how real LHAPDF
  6.5.6 returns `.info` descriptions (the folded HERAPDF20 `SetDesc`, the
  quoted CT18 ones). Diff the banked block against ours and fix our
  description handling to match LHAPDF's. Pin the fix with a unit test on
  those `.info` files.
- The PDF sets' files must be reachable through the LHAPDF tiers. If the 43
  sets cannot be fetched here (about 4 GB, with consent), say so. Then gate
  what the bundled `.info` files allow (the header), and the weights against
  the members you can fetch, at least the 325300 set's.
- Both αs-variation members (325300 members 101 and 102, with their own
  `AlphaS_MZ`) must appear correctly. This is F1's real-data pin.

## 2. MadGraph's default-argument MLM runs as a CI gate (S-F2's FL)

Turn `madgraphs_module_on_other_files` over the four `run_01_syst_default`
files into a banked gate that runs by default in the banked layer. It must
cover all weights, the `--alps` counts, and the dropped-MLM-terms control.

## 3. The dump test's unmatched branch

`validate_mlm_dumps.rs` (around lines 1411–1435) fixes the beam labels as
`[1, 2]` and takes x from printed energies (about 2e-7). MadGraph's reader
accepts either `<pdfrwt>` order. Read the labels and x the way the matched
branch does, or gate that row at the precision its inputs carry, with the
reason written down. You can't run the dump test here. Prepare the M3
re-run command for the manager.

## 4. FG: the permuted-P1 events

On `pp_to_ttx_0j1j_mlm` (3744 events) and `pp_to_ll_0j2j_mlm` (160), events
whose P1 is a symmetry permutation of PP are not gated, and some disagree.
Both record fields and weights differ, so this may affect production weights.
- The bundle now carries `configs.inc`. Compare this crate's forests for the
  affected subprocesses with MadGraph's, and find whether the difference is
  the replay's channel or permutation handling, or this crate's record
  construction.
- If it is production behaviour, fix it, and gate the permuted events.
- If it is the replay's, fix the replay so they are gated.
- If you can't settle it, report the evidence and the M3 run that would.

## 5. Scale

Measure the resident memory and wall time of `generate` with the full CMS
880-member request on one process (O2: about 3 GB was estimated against CMS's
2 GB slot). Measure, don't extrapolate.

## Also

- **FH:** `generate` refuses the `pp_to_ttx_0j1j_mlm` card (`group_resonances`,
  `generate.rs:1214`). Check whether `main` does too; build main in a scratch
  worktree only if it's cheap. Report, don't fix.
- The `refdata-12` pin changes the banked tree. Run the banked targets the
  `extended-validation` skill names for LHEF, PDF, scale and MLM changes, so
  that nothing moved because of the new bundle.

## Gate

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`, and again with
  `--features vibegraph/extended-validation`
- `cargo test --workspace --no-fail-fast`
- `python3 scripts/kb.py lint` (0 errors)
- the banked targets above

Every new gate needs a known-wrong control that it detects. Write and commit
`sessions/S-V1-report.md`, with **Found**.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>` (already the repo's git config).
Each message ends with exactly one line `Assisted-by: claude-code:<your model id>`.
No `Co-authored-by`, no `Signed-off-by`, even if the harness asks. Do not push,
and never amend a pushed commit. Do not edit backlog items. Commit
checkpoints as you go.

## Worktree & long-command discipline

- **First action**: `cd` to the absolute worktree path in your assignment, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current`. If you
  find yourself in the shared main checkout, STOP and report — never edit it.
  Re-verify `pwd` after any resume; resume is when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log file prefixed with your
  worktree/session name — parallel siblings share one scratchpad and have
  clobbered each other's unprefixed logs). A backgrounded command notifies on
  exit: never spawn sleep-based or `pgrep`-based watcher shells; draft your
  notes and report between completion notifications instead of idle-waiting.
  Never `pkill -f`; kill only PIDs you started.
- **No foreground command over ~8 minutes.** Detach longer runs.
- **`git status` can hang for minutes** in a worktree full of untracked build
  data; spot-check with `git log` / `git diff --stat` instead.
