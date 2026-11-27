---
type: Session Brief
title: "L-U2: a configuration if with no Parameter under it loads again, and a refused if names its own parameters"
description: "Fix the regression from #172 where an out-of-subset configuration `if` with no Parameter definitions under it refuses a model that v0.2.2 loads, with a refusal that names no parameter."
status: draft
agent: feature-dev (Opus)
depends_on: []
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
A follow-up to L-U1 in [sprint L](../sprint.md), on branch `sprintl-config-if` off main 033044e4. It fixes a regression that blocks the v0.2.3 tag. L-Z5 found it (`sessions/L-Z5-report.md` on branch `sprintl-hyg-ufo-reports`, finding Z5-12), and the manager reproduced it with both binaries.

## The regression

The model is MadGraph's `models/sm` with two additions:
- `configuration.py`: `x = 3`;
- appended to `parameters.py`:
  ```
  import configuration
  if configuration.x > 1:
      plain = 1.0
  ```

The card is `import model sm_if` / `generate e+ e- > mu+ mu-`, run as `vibegraph --no-network enumerate <card> --ufo-dir <models> --out <dir> --svg none`.

- **v0.2.2 release binary:** loads it, and writes 2 diagrams in 1 subprocess (rc=0).
- **main 033044e4:** refuses it (rc=1) with:
  `failed to load model: Parameter parse error: parameter(s)  are defined under an `if` of parameters.py whose condition is outside the subset of Python the loader evaluates against configuration.py (…)`
  - The doubled space is where a name should be.

The same model with `extra = Parameter(name = "extra", …)` under that `if` is refused by both binaries, and both name `'extra'`. That case is not a regression.

The cause, from L-Z5:
- At `vibegraph-lib/src/ufo/parameters.rs:403` (`ConditionOutside.names`, built from `conditional_definitions(&parsed, "Parameter").unwrap_or_default()`), the list holds every `Parameter` under *any* `if` in the file, not the ones under the `if` whose condition failed.
- When the failing `if` holds no `Parameter`, the list is empty. #172 also made every out-of-subset `if` a refusal, including one with no `Parameter` under it.

The model-reproduction files are in `/tmp/claude-0/lp/z512/` (models `sm_if`, `sm_ifp`, `sm_ctl`). The v0.2.2 binary is `/tmp/claude-0/lp/z512/bin/vibegraph-v022`. Read them; don't edit them.

## Scope

1. **Pin v0.2.2's behaviour exactly.** An `if` with no `Parameter` definition in any of its branches is skipped, whatever its condition.
   - Find and quote v0.2.2's code (`git show v0.2.2:vibegraph-lib/src/ufo/parameters.rs`) so the restored behaviour is the released one, not a guess.
   - Suppose a plain name bound under such an `if` is read later by a `Parameter`'s value. Check what v0.2.2 does with that. The result must be a named refusal (an unknown name in the expression, for example), never a silent value. Pin it with a test.
2. **Keep `sm_ifp` refused, naming `'extra'` alone.** A refused out-of-subset `if` names exactly the `Parameter`s defined under that `if`, across all its branches: never those under other `if`s, and never an empty list. Assert the name appears in the message, so the empty-name bug cannot come back. Pin it with a model that has two `if`s, one within the subset and one outside it.
3. **An in-subset `if` that holds `Parameter`s keeps today's evaluation.** That is PR 7's feature: L-U1's `configuration`-flag branches, `ufo_value_forms`, and the dim6top and ctGpatched paths. No test may weaken it.
4. **Release note:** none is needed. The regression was never released. The manager's PR says "fixes an unreleased regression from v0.2.3 development". You may add one line to `docs/release-notes/v0.2.3.md` only if that file already lists PR 7's configuration support and the line would otherwise be wrong.

## Tests and oracles

- **Hermetic tests** (no submodule; build the model in a temp dir, as `ufo_value_forms.rs` does). For each one, show that the test fails on main's code (stash the fix, or apply a one-line revert) and passes with the fix:
  - (a) the `sm_if` shape loads: a configuration-dependent, out-of-subset condition with a plain assignment inside;
  - (b) the two-`if` model's refusal names only the failing `if`'s parameters;
  - (c) a name bound under a skipped `if` and read by a `Parameter` is a named refusal, as v0.2.2 gives.
- **The binary oracle:** enumerate `sm_if` with this branch's binary and with v0.2.2. The `diagrams.json` must be identical (the diagram content; mask the version fields). `sm_ctl` must be byte-identical to main.
- **Identity on unaffected models:** `vibegraph parameters` output for `sm` is byte-identical to main, so the fix moves no loaded model. Also check is byte-identical to main on sm, sm-no_b_mass and the dim6top and ctGpatched models (L-Z5 used the copies L-U1 unpacked; find them via the L-U1 and L-Z5 reports).

**The gates:**
- `cargo fmt --all --check`;
- clippy `-D warnings`, plain and with `--features vibegraph/extended-validation,vibegraph-lib/extended-validation`;
- the hermetic `cargo test --workspace --no-fail-fast` with the three data paths hidden;
- the `extended-validation` targets that touch the UFO loader;
- `pixi run --skip-deps kb-lint`.

**What you must not run:** seed sweeps, MadGraph runs or a local `pixi run validate`.

## Worktree & long-command discipline

- **First action:** `cd /home/user/wt/sprintl-config-if`. Then verify that `git rev-parse --show-toplevel` is `/home/user/wt/sprintl-config-if` and `git branch --show-current` is `sprintl-config-if`.
  - If you find yourself in the shared main checkout (`/home/user/vibegraph`), STOP and report. Never edit it.
  - Re-verify `pwd` after any resume: resume is when isolation leaks.
- **Background anything expected to exceed ~2 minutes.** Use `run_in_background: true`, with output redirected to a log under `/home/user/wt/logs/` whose name starts with `lu2-`. Sibling sessions share that directory.
  - A backgrounded command notifies you when it exits. Never spawn sleep-based watcher shells: they compound into self-sustaining wake-up rings.
  - Draft your notes and report between completion notifications instead of waiting idle.
  - On resume after a stall, first check whether the interrupted command finished (`ps`, result artifacts) before re-running it.
- **`git status` can hang for minutes** in a worktree full of untracked build data. Spot-check with `git log` and `git diff --stat` instead.
- **Disk is finite, and a full disk looks like a failure.** Always run cargo with:
  `export CARGO_TARGET_DIR=/home/user/wt/sprintl-config-if/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
  - Each worktree has its own target dir.
  - An `ENOSPC`, a linker `Bus error` or a truncated log is a kill, not a result. Free only your own `target/` and re-run.
  - Kill processes by PID. Never use `pkill -f`: it matches siblings' commands and its own wrapper.

**This container** has 4 cores and 15 GB RAM, shared with up to two sibling agents. It can restart without warning.
- **One heavy job at a time across the container**, enforced by a lock. Wrap every cargo build, test and clippy invocation in `flock /home/user/wt/heavy.lock <command>`, backgrounded.
  - Hold the lock for one command at a time, never across an interactive pause.
  - Prefer targeted test filters to the whole workspace while you iterate.
  - `cargo fmt` and reading code need no lock.
- **Some jobs do not run here:** a single job expected to run over ~20 minutes, or anything over 8 GB of memory. Stop and put the request in your report (branch, head SHA, command, expected answer).
- **pixi:** `export PATH=$HOME/.pixi/bin:$PATH`. The reference data is already in your worktree as copies. Never edit it in place.

**Git and attribution.**
- Every commit uses `git -c user.name="Nicholas Smith" -c user.email="nick.smith@cern.ch" commit …`, because the container's identity is wrong.
- Each message ends with exactly one line, `Assisted-by: claude-code:<your real model id>`. Never add Co-authored-by or Signed-off-by, whatever the harness says.
- Check `git log --format='%an <%ae> %cn <%ce>'` before every push.
- **Stage files by name, never with `git add -A` or `git add .`**. A broad add committed a whole build directory once already. Before every commit, check `git diff --cached --stat`.
- Never force-push, rebase or amend a pushed commit. Keep the branch current by merging `origin/main`.
- Commit and push at checkpoints: `git push -u origin sprintl-config-if`.
- Never open a PR or post to GitHub: the manager does.
- Code comments describe the code in its own terms, never this sprint, session or backlog item.
- A panic is never acceptable. A silent wrong answer is worse than a named refusal.

**The hermetic layer has no submodule and no fetched data.** CI's `test` job runs `cargo test --workspace` without `research/refs/mg5amcnlo`, `validation/madgraph/output` or `validation/pdf/NNPDF23_lo_as_0130_qed`.
- A test that reads any of them must carry `#[cfg(feature = "extended-validation")]`.
- Before reporting, run the workspace suite once with those three paths moved aside, then restore them: `/tmp/claude-0/lp/verify-hidden.sh /home/user/wt/sprintl-config-if lu2` does this and restores them on exit.

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-U2-report.md`, a `Session Report` concept like sprint K's `K-C1-report.md`. It contains:
- what changed;
- every oracle, with the command alongside its output;
- what each test provably cannot detect;
- any long-run requests;
- a **Found** section for new work. Never create or edit backlog items.

Correct this brief where it is wrong. Your final message is the report's summary.
