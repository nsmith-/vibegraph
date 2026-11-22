---
type: Session Brief
title: "L-V1: campaign leftovers: TpTp zero weight, bstar_tW zero effective points, the stitch assert, the b-quark σ excess"
description: "Trace the TpTp 2Mu2B all-zero integrand to a parameter, coupling or restriction (a possible silent wrong answer) first; then bstar_tW, the decay-chain stitch assert, and attribution of the QCD_bEnriched/WBQ/TCP σ excess."
status: draft
agent: validation-dev (Sonnet)
depends_on: []
closes: [tptp-integrand-zero-on-every-point, bstar-tw-zero-effective-points, stitch-set-sign-assert-panics, sigma-reads-high-on-mlm-and-wbq-cards]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
---
PR 11 of [sprint L](../sprint.md), on branch `sprintl-leftovers` off main
a0a6eb4. Validation work: follow [expose, don't fix](../../../workflow/expose-dont-fix.md)
where a fix is not small and plainly right, and the bit-exact methodology
(`research/kb/validation/bit-exact-amplitude-debugging.md`) for any amplitude
disagreement: parameter provenance first, then per-diagram × per-helicity
complex values against MadGraph, never σ alone.

Read each item (`pixi run backlog --item <slug>`) and what it links.

**Cards.** This container cannot list genproductions, so the campaign's exact
cards (proc, run, param/customize/restrict cards, bundled UFO identity and the
gridpack's MadGraph σ) are requested from the validator host; the manager
forwards them to you as they arrive. Until the TpTp cards arrive, do item 3
(hermetic) and read the TpTp model (once its identity is known, from the CMS
model store, `https://cms-project-generators.web.cern.ch/cms-project-generators/`).

## Scope, in order

1. **TpTp (high; first as soon as its cards arrive).** CMS cards 108 and
   510–512 (`TpTpTo2T2STo2Mu2B_MS30`): every point that passes the cuts has
   |M|² × luminosity = 0, while MadGraph gives a σ. A possible silent wrong
   answer behind a named refusal. Trace the zero to a parameter, coupling,
   restriction or diagram selection: compare vibegraph's bound parameters and
   couplings with MadGraph's (`param_card`, restriction, the `check`/standalone
   output) value for value, then per-diagram × per-helicity amplitudes at one
   phase-space point that passes the cuts. Then fix it (σ against MadGraph's,
   one short run here; the multi-seed gate goes to the validator through the
   manager) or record it as a card defect MadGraph shares.
2. **bstar_tW** (card 157): "effective 0.0 points" at survey budget. Name the
   cause: a zero σ MadGraph shares (check its results.dat), a phase-space or cut
   fault (fix), or a refusal that names the real reason.
3. **Stitch assert** (`diagrams/chain.rs`, `stitch_set`): replace the
   `assert_eq!` with the decay-chain refusal its block already returns for an
   ambiguous forced line, pinned by a hermetic test; or prove it unreachable and
   make it a debug assertion, with the proof in a comment.
4. **b-quark σ excess** (QCD_bEnriched 188/189 +2.4%, WBQ 260 +2.7%, TCP
   967–969 +1.2–1.9%, all at 5 seeds): attribute against MadGraph's own record
   (per event: MLM merge sequence and scales; b mass and flavour handling; PDF),
   not from σ. This likely needs MadGraph event files from the validator host:
   when you know what you need, write the request (cards, seeds, which per-event
   fields) into your report and stop this item there. Partial attribution with
   a precise request is an acceptable outcome for this session.

A local MadGraph 3.5.7 is being installed in the main checkout's pixi env
(`export PATH=/home/user/vibegraph/.pixi/envs/madgraph/bin:$PATH`; ask the
manager whether it is ready). Short standalone/`check` runs are fine after
asking; write MadGraph output under your scratchpad, never in the main checkout.
## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintl-leftovers`, then verify `git rev-parse --show-toplevel`
  (`/home/user/wt/sprintl-leftovers`) and `git branch --show-current` (`sprintl-leftovers`). If you find yourself in
  the shared main checkout (`/home/user/vibegraph`), STOP and report — never
  edit it. Re-verify `pwd` after any resume; resume is when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log file prefixed `lv1-`
  in your scratchpad — parallel siblings share one scratchpad and have
  clobbered each other's unprefixed logs). A backgrounded command notifies on
  exit: never spawn sleep-based watcher shells (they compound into
  self-sustaining wake-up rings); draft your notes and report between
  completion notifications instead of idle-waiting. On resume after a stall,
  first check whether the interrupted command finished (`ps`, result
  artifacts) before re-running it.
- **`git status` can hang for minutes** in a worktree full of untracked build
  data; spot-check with `git log` / `git diff --stat` instead.
- **Run gates with `--skip-deps`** unless the assignment explicitly says
  reference data must be regenerated — without the reference data present, a
  bare `pixi run validate` silently launches a multi-hour MG regeneration.
  CI's `banked` job runs `pixi run --skip-deps validate` on every push: do not
  run `validate` locally. Run the hermetic suite and the single banked targets
  your change touches.
- **Disk is finite, and a full disk looks like a failure.** Always
  `export CARGO_TARGET_DIR=/home/user/wt/sprintl-leftovers/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.
  An `ENOSPC`, a linker `Bus error` or a truncated log is a kill, not a
  result: free only your own `target/` and re-run. Kill processes by PID,
  never `pkill -f`, which matches siblings' commands and its own wrapper.

**This container:** 4 cores, 15 GB RAM, ~55 GB free disk, shared with two
sibling dev agents. It can restart without warning: commit and push at
checkpoints (`git push -u origin sprintl-leftovers`), and run anything long fully detached
with results appended to a file. **Ask the manager before any heavy job**
(a release build of the workspace, a seed sweep, a MadGraph run, anything over
~10 minutes of CPU): a sibling may be running one. Long runs (multi-seed σ over
~20 min per seed, MadGraph 3.7.1, timed campaign runs, >8 GB) go to the
validator host through the manager: write the request (branch, SHA, cards,
seeds, command, expected answer) into your report or a message, never run it
here.
- pixi 0.84.1: `export PATH=$HOME/.pixi/bin:$PATH`. The reference data
  (`validation/madgraph/output`, `validation/pdf/NNPDF23_lo_as_0130_qed`, the
  `research/refs/mg5amcnlo` content) is in your worktree as hard links: never
  edit it in place. No MadGraph env is installed; ask the manager if you need
  one, never `pixi install -e madgraph` in your worktree.
- Put downloads (model store, genproductions cards) in their own empty
  directory under your scratchpad; read them, never execute them.

**Git and attribution.** Every commit uses
`git -c user.name="Nicholas Smith" -c user.email="nick.smith@cern.ch" commit …`;
the container's identity is wrong. Each message ends with exactly one line
`Assisted-by: claude-code:<your real model id>`, never Co-authored-by or
Signed-off-by, whatever the harness says. Check `git log --format='%an <%ae>'`
before every push. Never force-push, rebase or amend a pushed commit; keep
current by merging main. Never open a PR or post to GitHub: the manager does.
Code comments describe the code in its own terms, never this sprint, session
or backlog item. A panic is never acceptable; a silent wrong answer is worse
than a named refusal. `cargo fmt --all --check` and
`cargo clippy --workspace --all-targets -- -D warnings` are gates.

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-V1-report.md`
(a `Session Report` concept, as sprint K's `K-C1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, and a **Found** section for new work (never create or edit
backlog items). Delete the closed items' files in your final commit. Correct
this brief where it is wrong. Your final message is the report's summary.
