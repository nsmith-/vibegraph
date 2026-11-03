---
type: Session Brief
title: "S-V2: the permuted-event flavour code, the xqcut_only branch, and FG's close"
description: "Fix the production <pdfrwt> flavour code on permuted-P1 MLM events (ttx); make the dump test's unmatched branch correct, with diagnostics; classify ll_0j2j's permuted events under H1; hand the M3 a final re-run."
status: draft
agent: validation-dev (Opus)
depends_on: [S-V1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-30}
---
Worktree `/home/user/wt/S`, branch `sprinte-systematics` (PR #83), at the
commit that adds this brief. refdata-12 is unpacked in it as hard links; never
write into it. Pixi is not installed.

Read first:
- `S-F2-report.md`
- `S-V1-report.md`
- the M3's re-run on #83, comment 6527838644. The full log is on the M3, not
  here.

You can't run `validate_mlm_dumps` here: the dumps are on the M3 only.

## 1. The `ttx` permuted-event flavour code (a production defect)

On `pp_to_ttx_0j1j_mlm`, the M3's re-run shows the following on the 3744
events whose P1 is a symmetry permutation of PP:
- the rebuilt event is the clustered point on every event;
- the first-call scales and `q2bck` match MadEvent exactly;
- x, q and the rewgt chain's x and q are equal.

Yet `<pdfrwt>` differs on 106 (beam 1) and 113 (beam 2) of those events, and
`<totfact>` on 219. On the four printed events (48, 61, 64 and 133, all
`P1_gq_ttxq`), only the **flavour code** of the first parton-density entry
differs: ours is −1 / 2 / 1 / 1, MadEvent's is −2 / 1 / 2 / 3. Our rewgt chain
uses our flavour.

The M3's guess, untested: we take the flavour from the written leg rather
than from the permuted channel's leg.

- Find the rule MadEvent uses (`reweight.f`'s `ipdgcl`/`ipart` handling under
  a permuted IPROC, `config_subproc_map.inc`), and fix our record to follow
  it.
- Check whether the same wrong flavour also reaches the **weights** our
  production path writes, not just the record. `<totfact>` differs, so the
  PDF ratio probably uses it. If it does, this changes the systematics
  weights of about 2% of matched t t̄ + jet events; say so, with the size of
  the change.
- `validate_mlm_records` runs here. Its event-file replay saw about 9% of
  `ttx` events disagree (FG); report what this fix does to that count.
- Add a test that fails under the old rule, with mutation evidence.

## 2. The `pp_to_llj_xqcut_only` branch of the dump test

S-V1's rewrite left the result unchanged: `<pdfrwt>` count, codes, x, q and
beam labels all agree on 4978/10000 events, the same as before, and the row
prints no diagnostics. So the new code either didn't take effect or rests on
a wrong rule.
- Add the same per-event diagnostics the matched rows print.
- Re-derive the beam-label and flavour rule from `unwgt.f` and `reweight.f`
  for `ickkw = 0` with `xqcut > 0`.
- Show that it would pass. Check it on the banked event file and `<mgrwt>`
  records you have here (`validate_mlm_records` covers this row) before
  handing it to the M3.

## 3. `ll_0j2j`'s permuted events

There the first-call scales themselves differ on 77/160 events. By S-V1's
rule that is MadEvent's registered first-call deviation H1. Confirm the
classification: find where H1 is registered and check that this case matches
its mechanism. Then decide how the gate treats those events. Excluding them
is acceptable only through the registered deviation, never silently. Say
what the gate can't see as a result.

## 4. The M3 re-run

Prepare the exact re-run command, with the pass and fail conditions, for the
manager to send. Ask the M3 to run from a fresh `fetch-refdata` work area this
time, so the refdata-12 `.inc` files (including `config_subproc_map.inc`)
are exercised.

## Gate

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`, and again with
  `--features vibegraph/extended-validation`
- `cargo test --workspace --no-fail-fast`
- `python3 scripts/kb.py lint` (0 errors)
- the banked targets touched by any change, as separate detached runs:
  `validate_mlm_records`, `validate_systematics`, `validate-mlm-sigma`,
  `validate-mlm-samples`, `cli_systematics`, `validate_lhef`

Build with debuginfo off and `-j 2`. Write and commit `sessions/S-V2-report.md`,
with a **Found** section.

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
- **Use absolute paths in every command**, and never background a
  `cd … && …` chain: the shell's working directory then leaks into the next
  command.
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
