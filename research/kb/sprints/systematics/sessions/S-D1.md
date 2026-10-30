---
type: Session Brief
title: "S-D1: design the systematics weights"
description: "Design only: the typed request and artifact version 15, the per-event weight calculation mirroring MadGraph's systematics.py, the exact header and <rwgt>/<mgrwt> format, PDF loading at scale, refusals, migration, gates and the M3 banking request."
status: draft
agent: feature-dev (Opus)
depends_on: []
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-28}
---
Worktree `/home/user/wt/S`, branch `sprinte-systematics`, at the commit that
adds this brief (main f5fb3e65 plus the sprint folder). **Design only: no
production code.** Pixi is not installed; `python3 scripts/kb.py lint` is the
lint. MadGraph's source is in `research/refs/mg5amcnlo` and the banked runs
are in `validation/madgraph/output` (hard links: do not write into them).

## Read first

- `python3 scripts/kb.py backlog --item systematics-variation-weights-not-written`.
- `../sprint.md`: what the banked data holds, and the user's decisions Q1–Q9.
  They are settled; do not reopen them. Where one turns out to be unworkable,
  say so with evidence and propose the smallest change.
- [The split](../../../pipeline/integrate-generate-split.md) §1–§2
  (`GenerateSettings`, the stage column) and
  [run inputs](../../../pipeline/run-inputs-in-the-artifact.md) §3a (the
  `<vibegraphInputs>` systematics entries; user decision C1).
- `research/kb/pipeline/release-scope.md`: refuse what is not honoured.
- MadGraph: `madgraph/various/systematics.py` (all of it),
  `lhe_parser.py` (`parse_lo_weight`, `get_et_scale`, `get_ht_scale`,
  `get_sqrts_scale`), the `systematics` parts of `banner.py` and
  `common_run_interface.py` (`do_systematics`, `--from_card`), and where
  madevent writes `<mgrwt>` and decides to run the systematics after
  generation (and in what order relative to `reweight`).
- vibegraph: `GenerateSettings`/`Systematics` (`generate.rs`), the run-card
  stage classes (`runcard/classes.rs`, `warn_unwritten_systematics`), the
  LHEF writer (`lhef/write.rs`, `lhef/emit.rs`, `initrwgt_block`), the scale
  code, the αs sources, the PDF loader, and the MLM reweighting record.
  `research/kb/events/` holds the LHEF concepts.

## Deliverable

A new concept, `research/kb/events/systematics-weights.md` (status draft),
plus `sessions/S-D1-report.md`. The concept must settle:

1. **The typed request.** Types for the parsed request. Parsing both entry
   points: `systematics_arguments` when the card sets it, else `sys_*`, as
   `call_systematics` does. Establish how vibegraph knows the card *set*
   `systematics_arguments`; MadGraph uses `user_set`. Cover the `--systematics`
   override at generate, the refusal table (Q1) with each message, and where
   each refusal fires (integrate, and generate for the override). Then
   artifact version 15: what changes in `GenerateSettings`, the floor, and
   what is migrated or refused.
2. **The weight calculation.** A library function from a per-event record to
   the weights, mirroring `get_lo_wgt` term by term:
   - the μR part, with αs from each PDF member's own αs; fixed-energy beams
     use the run card's αs running;
   - the μF/PDF part;
   - the dynamical-scale recomputation from the written event's status-1
     particles;
   - for MLM, the `asrwt` emission-scale αs and the `pdfrwt` chain with
     `--alps`;
   - the ratio to the nominal.

   Specify the per-event record so it is filled identically from vibegraph's
   own generation and parsed from MadGraph's `<mgrwt>`. One function then
   serves both production and the gate on MadGraph's events. Record every
   MadGraph quirk found (for example the `asrwt` loop that assigns rather than
   multiplies on fixed-energy beams) and decide mirror or refuse for each.
3. **Format, byte for byte.** `<initrwgt>` groups, names, `combine`, tags,
   info text (Python `str(float)` spellings), the `# …` trailing comments, id
   numbering (`get_id`: after existing numeric ids, or `--start_id`),
   ordering, the per-event `<wgt>` number format, and `<mgrwt>`'s layout.
   Show a banked MadGraph header next to what the design writes for the same
   request. Specify how this composes with `--reweight-card` weights (MadGraph's
   order of the two).
4. **PDFs at scale.** Resolving every requested set and member through the
   LHAPDF tiers, with consent. Recording each file read in
   `<vibegraphInputs>` (§3a). Memory and wall time for about 880 members:
   estimate them, and say whether members load lazily. Support for each
   member's `AlphaS_Type`, and the refusal for a type that cannot be
   evaluated. MadGraph's central-member fallback in `get_pdfQ2`, if it is
   reachable.
5. **Q3's consequence.** A default proton card now writes 145 weights.
   List the tests, scripts, docs and pinned outputs this moves, and the cost
   per event.
6. **Gates.**
   - The per-event gate on the twelve banked runs: which fields, what
     tolerance and why (MadGraph prints 8 significant digits), and what it
     cannot detect.
   - Negative controls: μR and μF swapped, the wrong αs power, the
     `setscales.f` dynamical scale in place of the Python one.
   - The `<mgrwt>` round trip on vibegraph's own events.
   - What runs in CI's banked layer.
7. **The M3 banking request** (Q9). Determine the most common CMS PDF list
   from the survey material (`research/kb/validation/cms-gridpack-survey-2026-10.md`,
   PR #71's database branch `cms-gridpack-db`), and the exact `--pdf` string.
   Write the exact commands that run MadGraph's standalone `systematics` with
   the CMS arguments
   `--mur=1,2,0.5 --muf=1,2,0.5 --together=muf,mur,dyn --dyn=-1,1,2,3,4 --alps=0.5,1,2 --pdf=<list>`
   on `pp_to_ll`, `pp_to_jj`, `pp_to_ll_0j2j_mlm` and `pp_to_ttx_0j1j_mlm`
   (run_01). Include what to bank, the expected weight counts, and what makes
   the run pass or fail. It must finish in about 5 hours.
8. **The implementation split** into S-F1 (unmatched) and S-F2 (MLM), with
   each session's gate.

## Gate

`python3 scripts/kb.py index` then `python3 scripts/kb.py lint`, 0 errors.
Machine-check every count or format claim against the banked files and say how
(no by-hand census). In the report, list the open choices the design could not
settle, each with a recommendation, and include a **Found** section. Correct
this brief where it is wrong.

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
