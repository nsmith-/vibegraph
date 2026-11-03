---
type: Session Brief
title: "T-V3: a banked no-pT-cut EWK ℓℓjj row, and the MLM re-gates"
description: "Prepare the M3 banking of p p > e+ e- j j QCD=0 with ptj = ptl = 0 (cards, manifest row, driver support, the σ gate awaiting its bank, a smoke-sized cost estimate); re-gate the two MLM rows T-F2's maps changed."
status: draft
agent: validation-dev (Opus)
depends_on: [T-F2]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-30}
---
Worktree `/home/user/wt/T`, branch `sprinte-spine-fallback` (PR #81), at the
commit that adds this brief. That branch is on refdata-11; the spine needs
nothing from refdata-12. Pixi is not installed. Read `T-F2-report.md` first,
then the P-V1 report on #82
(`git show origin/sprinte-param-card:research/kb/sprints/param-card/sessions/P-V1-report.md`),
which is how the last banking request was prepared.

## 1. The row (user decision, 2026-10-30)

No banked row has `ptj = ptl = 0`. The one CMS reference is a single
MadGraph 2.9.18 survey, run without refine. Bank a generic row that covers
the no-pT-cut spine regime, under the same seed policy as every other row:

- **Process:** `p p > e+ e- j j QCD=0`. One massless lepton flavour.
  - Take the structure from the CMS DY card
    (`git show origin/vbf-dy-spine-inputs:validation/cms_gridpacks/inputs/DYto2L-2Jets-EWK_Bin-MLL-50/proc_card.dat`),
    including its diagram exclusion (the CMS card uses `$$ h` or `/ t t~ h`).
  - Drop what is CMS-specific: use `sm` or `sm-no_b_mass` for a 5-flavour
    `j`, not `loop_sm`/`ckm_no_b_mass`, and no param-card moves.
  - Say what you chose and why.
- **Run card:**
  - `ptj = ptl = 0`, no `etaj`/`etal`/`drjj`/`drll`/`drjl` cuts, `mmjj = 4`,
    `mmll = 50`;
  - `dynamical_scale_choice = -1` with `fixed_fac_scale = False` (so MadEvent's
    2 GeV μF floor applies);
  - `pdlabel = lhapdf`, `lhaid = 247000`, 13 TeV.

  Base it on an existing banked proton card, and list every field that
  differs from that card.
- **Banking inputs** (follow P-V1's pattern):
  - the `.mg5` script under `validation/madgraph/scripts/`, plus the run
    card;
  - a planned manifest row with the rationale;
  - `bank_seeded_sigma.sh` support if it needs any, checked with `bash -n`
    and a dry run.

  MadGraph doesn't run here.
- **The gate:** a vibegraph σ test against the row's seed record, written now.
  - It reports "awaiting the bank" until the record exists.
  - It runs ≥5 seeds per side, takes the unweighted mean, and reads the spread
    and χ²/dof.
  - Give it a budget that converges on this host (use T-F2's measurements).
  - Put it where the `extended-validation` skill says such a gate goes, and
    give its runtime.
- **vibegraph's own side now:** run the row's vibegraph σ here at the gate's
  budget, ≥5 seeds, and report it. Use it as the expected value in the
  request; it is not a tuning target.
- **The M3 request text, in the report:**
  - the exact commands: a one-seed smoke run first, to size MadEvent's cost
    (CMS's survey on the sibling card took 3–4 CPU hours);
  - the banking itself, with refine;
  - the gate run on the M3;
  - the expected outputs, pass/fail, and what to send back.

  The manager posts it on #81 and sends it to the M3.

## 2. The MLM re-gates (T-F2 not-run)

T-F2's remainder mixture changed the maps of `pp_to_ll_0j2j_mlm` and
`pp_to_ttx_0j1j_mlm`. Run `validate-mlm-sigma` and `validate-mlm-samples`
(see `pixi.toml` for the cargo commands), detached, and report the pulls
against the last recorded ones (ttx total +0.75, 0j2j total +1.54, llj_mlm
+1.00). The run takes about an hour.

## Gate

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`, and again with
  `--features vibegraph/extended-validation` and with
  `-p vibegraph-lib --features extended-validation`
- `cargo test --workspace --no-fail-fast`
- `python3 scripts/kb.py lint`, with 0 errors

Build with debuginfo off (`--config profile.release-debug.debug=0`) and
`-j 2`; free disk is about 15 GB. Write and commit `sessions/T-V3-report.md`,
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
  command. This is how T-F2 wrote into the main checkout.
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
