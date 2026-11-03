---
type: Session Brief
title: "S-F1: systematics weights for unmatched events"
description: "Implement the approved design's S-F1 scope: the typed request at artifact version 15, the PDF member metadata cascade, the weight function and record, MadGraph's exact header and weight format, and the per-event gate on the twelve banked runs."
status: draft
agent: feature-dev (Opus)
depends_on: [S-D1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-28}
---
Worktree `/home/user/wt/S`, branch `sprinte-systematics` (PR #83), at the
commit that adds this brief. Pixi is not installed: run the commands it wraps
directly (`pixi.toml`). The reference data is in the worktree as hard links:
`validation/madgraph/output`, the sets in `validation/pdf`, and
`research/refs/mg5amcnlo`. Do not commit or write into any of it.

## The design is approved

[The systematics weights design](../../../events/systematics-weights.md) is
approved as written (user, 2026-10-28), with its open choices O1–O12 taken as
recommended in `S-D1-report.md`. Implement **§8's S-F1 scope and gate,
exactly**. Where the code disagrees with the design, stop, record the evidence
in your report, and propose the smallest change. Don't silently diverge.

In scope, beyond §8's list:
- **Found F1, PDF members' own headers.** `PdfSet::member` overlays each
  member's metadata header on the set's `.info` (`AlphaS_*`,
  `ForcePositive`). Pin it on a synthetic set, and also on
  `NNPDF31_nnlo_as_0118_mc_hessian_pdfas` member 102 if that set is
  reachable.
- **Found F2, `.info` parsed as YAML,** with a YAML crate rather than a
  hand-written parser (AGENTS.md: never hand-write a standard primitive). The
  multi-line and quoted descriptions found in S-D1 are test cases.

Out of scope:
- MLM records (S-F2): a matched request is refused until then.
- The CMS-arguments gate (S-V1): the M3 files are not banked yet.
- Found F3, the `rwgt_` prefix: it stays a separate item. If an id collision
  between the reweight and systematics weights is reachable in this PR's
  tests, refuse the colliding case and say so.

## Gate (§8, S-F1)

Run everything in §8's S-F1 gate:
- per-event weights on all twelve banked runs (§6.1), reporting the fraction
  of exact strings;
- header byte-identity on the arguments, `--from_card` and `sys_*` paths;
- §6.2's negative controls, with the detection table;
- the `<mgrwt>` round trip (§6.4);
- the Python-float table;
- one test per refusal-table row;
- `cargo fmt --all --check`;
- `cargo clippy --workspace --all-targets -- -D warnings`, and again with
  `--features vibegraph/extended-validation`;
- `cargo test --workspace`;
- `python3 scripts/kb.py lint` (0 errors);
- the banked layer, `bash validation/validate.sh`, run detached.

Invoke the `extended-validation` skill to confirm that list is complete for an
LHEF writer, PDF and scale change.

Q3 moves every default proton generation: list each test or pin you changed,
with its old and new value. Each rule gets a test with mutation evidence.
Write and commit `sessions/S-F1-report.md` containing:
- every command with its output;
- the gate tables;
- the mutations;
- a **Found** section.

Correct this brief where it is wrong.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>` (already the repo's git config).
Each message ends with exactly one line `Assisted-by: claude-code:<your model id>`.
No `Co-authored-by`, no `Signed-off-by`, even if the harness asks. Do not push.
Do not edit backlog items.

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
