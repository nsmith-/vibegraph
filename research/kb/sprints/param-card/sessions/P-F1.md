---
type: Session Brief
title: "P-F1: integrate --param-card and the recorded parameter point"
description: "integrate --param-card binds through from_model_card and fills inputs.params; generate rebinds from it; reweight denominators at the recorded point; refusals for Auto widths, revived zeroed parameters and split merged pairs; <slha> header block."
status: draft
agent: feature-dev (Opus)
depends_on: []
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-28}
---
Worktree `/home/user/wt/P`, branch `sprinte-param-card`, at the commit that
adds this brief (main f5fb3e65 plus the sprint folder). Pixi is not installed:
run the commands it wraps directly (`pixi.toml`). The reference data is in the
worktree as hard links (`validation/madgraph/output`, the sets in
`validation/pdf`, `research/refs/mg5amcnlo`). Do not commit or write into any
of it.

## Read first

- `python3 scripts/kb.py backlog --item cli-has-no-param-card-option`, and the
  item's scope list.
- [Run inputs in the artifact](../../../pipeline/run-inputs-in-the-artifact.md)
  §1 (`MovedParam`, `params`), §3 (what `generate` checks) and §6, the last
  paragraph (this item's tests). Also
  [the split](../../../pipeline/integrate-generate-split.md) §3: `generate`
  takes no param card.
- The sprint record's decisions P1–P4 (`../sprint.md`).
- `EvaluatedModel::from_model_card` (`vibegraph-lib/src/ufo/mod.rs`),
  `ufo/slha.rs`, `RunInputs::refuse_moved_params` (`artifact.rs`), and how
  `generate` binds the model and resolves the reweight card today.

## Work

1. **`integrate --param-card <file>`** binds the model through
   `from_model_card`. `inputs.params` records every external parameter whose
   bound value differs from the restricted model's default, in the model's
   external order (§1). Lift the non-empty-list refusal on write and read. No
   artifact version change: the shape already exists.
2. **`generate`** rebuilds a `ParamCard` from `inputs.params` and binds the
   model with it, after the model digest check. It takes no card option.
3. **`generate --reweight-card`** resolves its hypotheses at the recorded
   point: denominators and the per-event audit use it.
4. **Refusals at `integrate`**, each with a test:
   - a card value that revives a parameter the restriction zeroed (the
     restriction also pruned its vertices);
   - `DECAY <pdg> Auto` (P1): the message says to give the width or use the
     card MadGraph wrote, and that computing Auto widths is not supported yet;
   - a move of one parameter of a pair the restriction merged (P4). Find how
     MadGraph's restriction merges identical parameters and how vibegraph
     represents that, and say so in the report;
   - an unknown block or code, if `from_model_card` does not already refuse it.
5. **Dependent parameters** (P2): a card value for an internal parameter (MW
   in the SM) is ignored with one warning naming the parameter, the card's
   value and the computed value. Not recorded.
6. **`<slha>` header block** (P3): `generate` writes the bound param card in
   MadGraph's `<slha>` layout. Compare the layout against a banked MadGraph
   event file's header and say what differs, if anything.
7. **Docs**: correct `docs/src/cli/overview.md` (~114) and
   `docs/src/guide/02-ufo.md` (~95), the CLI reference (`scripts/gen-cli-docs.sh`),
   and the concepts that describe the param card as not yet read.

## Validation (hermetic in this session; the σ gate is P-V1's)

- `params == []` for no card, an empty card and a card restating the
  defaults, and those artifacts compare byte for byte. A defaults card then
  generates the same events as no card.
- A moved `MZ` records one `MovedParam`. `generate` on that artifact writes
  events whose replayed weights equal integrate's at the same points: the
  rebind is exact.
- `generate --reweight-card` uses the recorded point as its denominator.
- **A known-wrong informational comparison**: the reweight oracle
  (`validation/madgraph/reweight_mg_reference.json`, `gen_reweight_oracle.py`)
  carries MadGraph's per-event weights at moved points. Report the σ that
  vibegraph integrates at one moved `MZ` (or `ymt`) card against MadGraph's
  reweighted σ at that point, and against vibegraph's default-card σ. This is
  informational only, ≥5 seeds per side. The gate against a fresh MadGraph
  run at the card is P-V1's, after banking.
- Propose the two banking rows for P-V1 (a moved `MZ` Drell-Yan row on the
  `dy13_mmll` card, and `ee_tth_ymt`'s process at `ymt = 250`): the exact
  cards and the `bank_seeded_sigma.sh` commands. Put them in the report; the
  manager posts the request.

## Gate

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`, and again with
  `--features vibegraph/extended-validation`
- `cargo test --workspace`
- `python3 scripts/kb.py lint` (0 errors)
- the gates the `extended-validation` skill names for a coupling or model
  binding change, run detached. The reweight gates (`validate-reweight` in
  `pixi.toml`) must stay green.

Each refusal and rule gets a test with mutation evidence. Write and commit
`sessions/P-F1-report.md`: every command with its output, the mutations, the
informational table, the proposed banking request, and a **Found** section.
Correct this brief where it is wrong, and say so in the report.

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
