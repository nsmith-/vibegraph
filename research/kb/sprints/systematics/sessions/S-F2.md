---
type: Session Brief
title: "S-F2: systematics weights for MLM events, and S-F1's FA and FE"
description: "LoRecord from the MLM Rewgt record, <mgrwt> on matched events, the matched refusal lifted; n_qcd from the clustering graph under a dynamical scale (FA); <MGRunCard> on every use_syst sample (FE)."
status: draft
agent: feature-dev (Opus)
depends_on: [S-F1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-29}
---
Worktree `/home/user/wt/S`, branch `sprinte-systematics` (PR #83), at the
commit that adds this brief. Pixi is not installed. The reference data is
hard-linked in; do not write into it. Read `S-F1-report.md` in full, then the
approved design §2 (the MLM terms), §6.3 and §8's S-F2 paragraph.

## Scope

1. **S-F2 as designed (§8).**
   - Build `LoRecord` from `Rewgt`.
   - Write `<mgrwt>` on matched events, also under
     `systematics_program = none`.
   - Lift the matched refusal.
   - Gate: §6.3's record check on the dumped MLM runs that refdata-11
     carries (`pp_to_ll_0j2j_mlm`, `pp_to_llj_mlm`, `pp_to_ttx_0j1j_mlm`,
     `pp_to_llj_xqcut_only`). Machine-check the fields of `<rscale>`,
     `<asrwt>` and both `<pdfrwt>` lines against MadGraph's record on every
     event.
2. **FA (S-F1 Found).** Under a dynamical scale, MadEvent takes `n_qcd` from
   the clustering's graph (`reweight.f:1437`); this build takes the labelled
   configuration. Fill it from the clustering's first graph. Show the
   `<rscale>` census `{2: 10000}` on a regenerated `pp_to_bb`-style record,
   and add a test that fails under the old rule.
3. **FE (S-F1 Found).** Q6's purpose is that MadGraph's own `systematics`
   can run on a vibegraph file, and it needs `<MGRunCard>`. Write the run
   card on every sample generated under `use_syst`, unmatched or matched.
   Prove it by running MadGraph's `systematics.py` from
   `research/refs/mg5amcnlo` over one vibegraph unmatched file and one MLM
   file. A metadata-only LHAPDF stand-in is enough for the header. If real
   weights need LHAPDF and it isn't available here, say so and leave that
   part to S-V1 on the M3.

**Out of scope:** the CMS-arguments gate on the M3 files. Those files are
staged on the M3 and reach the cloud only with the next refdata cut, so that
gate is S-V1's.

## Gate

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`, and again with
  `--features vibegraph/extended-validation`
- `cargo test --workspace --no-fail-fast`
- `python3 scripts/kb.py lint`, with 0 errors
- the banked targets the `extended-validation` skill names for an MLM or
  LHEF change, each as its own detached invocation, resumable. Reuse
  S-F1's `S-F1-banked2.sh` pattern. The MLM gates (`validate_mlm_dumps`,
  `validate-mlm-sigma`) must stay green.

Every rule gets a test with mutation evidence. Write and commit
`sessions/S-F2-report.md`, with a **Found** section. Correct this brief where
it is wrong.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>` (already the repo's git config).
Each message ends with exactly one line `Assisted-by: claude-code:<your model id>`.
No `Co-authored-by`, no `Signed-off-by`, even if the harness asks. Do not push.
Do not edit backlog items. Commit a checkpoint whenever a coherent piece
lands: this container has restarted twice this sprint.

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
