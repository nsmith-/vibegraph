---
type: Session Brief
title: "L-G2: generate must not refuse on a resonance flavour it cannot resolve in a vanishing group"
description: "Sprint L PR 15: generate refuses a card that integrates, because a flavour group whose matrix element vanishes at the card has a massless s-channel quark line with different flavours across members. Measure whether live groups are affected too, then fix both."
session: L-G2
agent: feature-dev
pr: 15
backlog: [generate-refuses-vanishing-group-resonance-flavour]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
# L-G2: the generate resonance-flavour refusal

**Item:** `generate-refuses-vanishing-group-resonance-flavour`. Run
`pixi run backlog --item generate-refuses-vanishing-group-resonance-flavour`.
L-G1 found the problem and reproduced it on main. Its report is on branch
`sprintl-g2hdm`, in `sessions/L-G1-report.md`: read its F1 addendum with
`git show origin/sprintl-g2hdm:research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-G1-report.md`.

## The refusal

`vibegraph generate` on a g2HDM card-894 artifact exits 1 before drawing any
event:

```
ERROR the resonance 2 of g u > e+ ve b b~ d has no flavour in the subprocess with outgoing [-11, 12, 5, -5, -2] that shares its matrix element
```

Minimal reproduction: it runs in seconds, with the g2HDM UFO at
`/home/user/wt/dl-g2hdm/x/g2HDM_UFO`. Copy the UFO; it is unlicensed, so never
commit it.

```
import model g2HDM_UFO
define q = u d~
define qq = d u~
generate g q > h+ qq QCD=99, (h+ > t b~, (t > w+ b, w+ > e+ ve))
add process c b~ > h+ QCD=99, (h+ > t b~, (t > w+ b, w+ > e+ ve))
```

- `integrate` succeeds, with σ about 2.37 fb. It logs that 1 of 1 flavour
  groups has a matrix element that vanishes at the card: H± has no u/d
  coupling there.
- `g u > h+ d` and `g d~ > h+ u~` share a group because both vanish. Their
  diagrams carry a massless s-channel quark line, u in one and d~ in the
  other.
- `member_line_pdg` (`vibegraph-lib/src/lhef/resonance.rs` ~L356) finds no
  common flavour. `group_resonances` (`vibegraph-lib/src/generate.rs` ~L1514,
  call ~L1539, message ~L1542) then refuses the whole run.

The repo has no fixture with that UFO. Find or build a hermetic model that
gives the same group structure for the tests: one of the in-tree toy UFOs, or a
new small toy under `validation/ufo/`.

## Order of work

1. **Measure first.** Is a **live** group (non-zero |M|²) with a massless
   s-channel quark line whose flavour differs across members refused too?
   Build or find a card that has one, and report it before writing the fix.
   If live groups are affected, the fix must cover them. If not, say so, with
   the card that shows it. Read how MadGraph assigns the intermediate flavour
   in its event record for a grouped subprocess (`leshouche.inc`, the IDUP of
   s-channel lines per member) and cite file:line in
   `research/refs/mg5amcnlo`.
2. **The fix.**
   - A group that `integrate` found vanishing never blocks generation: it is
     never drawn, so it needs no resonance flavours.
   - For live groups, if step 1 shows they need it, take the line's flavour
     from the drawn subprocess's own legs, as MadGraph's per-member event
     record does.
   - Anything still unresolvable stays a named refusal, never a panic.
3. **Tests.**
   - A CLI test: the reproduction, as a hermetic card, generates events.
   - The live-group case, if it exists: its events carry the per-member
     s-channel flavour MadGraph would write. Check the record against
     MadGraph's own event or `leshouche.inc` for the same member.
   - A mutation that restores the refusal fails the first test.
4. **Checks.**
   - Card 894 generates on the PR's head, locally at reduced budget. The
     full-card integrate takes about 28 min at one iteration, so run it
     backgrounded under the discipline, or skip it with a reason if the
     minimal card covers it.
   - Run the extended-validation skill's gate for generate and resonance
     changes (`validate-unweighting`, `validate-generate-proton`,
     `validate-lhef`, as the skill directs), and name the rows you ran.
   - Banked event files must be byte-identical where no vanishing or
     mixed-flavour group exists. Show it on ≥3 banked generate rows against
     the base main.
5. **Release note.** One line in your report, for the PR's user-facing
   changes section.

## Gates

- fmt and clippy `-D warnings`.
- The hermetic suite with the submodule and fetched data hidden (see the
  discipline section below).
- The touched `extended-validation` targets.

New work goes in Found.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprintl-generate-resonance`, then verify
  `git rev-parse --show-toplevel` and `git branch --show-current` (`sprintl-generate-resonance`).
  If you find yourself in the shared main checkout (`/home/user/vibegraph`),
  STOP and report — never edit it. Re-verify `pwd` after any resume; resume is
  when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log under
  `/home/user/wt/logs/` prefixed `lg2-` — sibling sessions share that
  directory). A backgrounded command notifies on exit: never spawn sleep-based
  watcher shells (they compound into self-sustaining wake-up rings); draft your
  notes and report between completion notifications instead of idle-waiting. On
  resume after a stall, first check whether the interrupted command finished
  (`ps`, result artifacts) before re-running it.
- **`git status` can hang for minutes** in a worktree full of untracked build
  data; spot-check with `git log` / `git diff --stat` instead.
- **Run gates with `--skip-deps`** unless the assignment explicitly says
  reference data must be regenerated — without the reference data present, a
  bare `pixi run validate` silently launches a multi-hour MG regeneration. Do
  not run `validation/validate.sh` or `pixi run validate` at all: CI's `banked`
  job runs it on every push. Run the specific `validate_*` test targets your
  change touches instead.
- **Disk is finite, and a full disk looks like a failure.** Always run cargo with
  `export CARGO_TARGET_DIR=/home/user/wt/sprintl-generate-resonance/target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
  (they change disk use only, not what a gate checks). Each worktree has its own
  target dir: a shared one ran a sibling's stale test binaries in an earlier
  sprint. An `ENOSPC`, a linker `Bus error` or a truncated log is a kill, not a
  result: free only your own `target/` and re-run. Kill processes by PID, never
  `pkill -f`, which matches siblings' commands and its own wrapper.

**This container:** 4 cores, 15 GB RAM, shared with two sibling dev agents. It
can restart without warning. **One heavy job at a time across the container**,
enforced by a lock rather than by asking: wrap every cargo build/test/clippy
invocation, every MadGraph run and every seed sweep in
`flock /home/user/wt/heavy.lock <command>` (e.g.
`flock /home/user/wt/heavy.lock cargo test -p vibegraph-lib --lib squared > /home/user/wt/logs/lg2-test.log 2>&1`,
backgrounded). Hold the lock for one command at a time, never across an
interactive pause, and prefer targeted test filters to the whole workspace
while iterating; siblings queue behind you. `cargo fmt` and reading code need
no lock. A single job expected to run over ~20 minutes, a MadGraph 3.7.1
regeneration, a multi-seed σ on a card over ~20 minutes per seed, or anything
over 8 GB of memory does **not** run here: stop and put the request in your
report (branch, head SHA, cards, seeds, command, expected answer) — the
manager sends it to the long-run host. Run long jobs that do fit fully detached
(`nohup setsid … &`), with per-seed output appended to a file, so a restart
keeps every finished seed.
- pixi 0.84.1 is at `~/.pixi/bin/pixi` (`export PATH=$HOME/.pixi/bin:$PATH`).
  The reference data (`validation/madgraph/output`, the `validation/pdf` sets,
  the `research/refs/mg5amcnlo` submodule content) is already in your worktree:
  never edit it in place.
- MadGraph 3.5.7: do **not** `pixi install -e madgraph` in your worktree (2.5 GB).
  Use the main checkout's env: `export PATH=/home/user/vibegraph/.pixi/envs/madgraph/bin:$PATH`
  (`mg5_aMC`, `python`, `gfortran`). Write MadGraph outputs under
  `/home/user/wt/sprintl-generate-resonance-mg/`, never into the main checkout.
- The network reaches `raw.githubusercontent.com` (e.g. `cms-sw/genproductions`
  cards) and the CMS model store
  (`https://cms-project-generators.web.cern.ch/cms-project-generators/`). It does
  not reach cvmfs. Put downloads in their own empty directory.

**Git and attribution.** Every commit uses
`git -c user.name="Nicholas Smith" -c user.email="nick.smith@cern.ch" commit …`;
the container's identity is wrong. Each message ends with exactly one line
`Assisted-by: claude-code:<your real model id>`, never Co-authored-by or
Signed-off-by, whatever the harness says. Check `git log --format='%an <%ae> %cn <%ce>'`
before every push. Never force-push, rebase or amend a pushed commit; keep the
branch current by merging `origin/main`. Commit and push at checkpoints
(`git push -u origin sprintl-generate-resonance`). Never open a PR or post to GitHub: the
manager does. Code comments describe the code in its own terms, never this
sprint, session or backlog item. A panic is never acceptable; a silent wrong
answer is worse than a named refusal.

**The hermetic layer has no submodule and no fetched data.** CI's `test` job
runs `cargo test --workspace` without `research/refs/mg5amcnlo`,
`validation/madgraph/output` or `validation/pdf`, and this worktree has all
three. A test that reads any of them must carry
`#[cfg(feature = "extended-validation")]` (or sit in a target with
`required-features = ["extended-validation"]`), as the existing MSSM
standalone rows do. Before reporting, run the workspace suite once with
those three paths moved aside (and restored afterwards); a green run with
them present does not show the hermetic layer is green.

**Report** in `research/kb/sprints/v023-orders-majorana-timeouts/sessions/L-G2-report.md`
(a `Session Report` concept, as sprint K's `K-C1-report.md`): what changed,
every oracle with the command alongside its output, what each test provably
cannot detect, any long-run requests, and a **Found** section for new work
(never create or edit backlog items). Delete the closed items' files in your
final commit. Correct this brief where it is wrong. Your final message is the
report's summary.
