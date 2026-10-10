# Hygiene sprint log

## 2026-10-09
* **Creation**: Sprint folder created with `pixi run new-sprint hygiene`.
* **Approval** (human:nsmith-, in the planning session): the four shape
  decisions. Scope is the localised hygiene items ([D1](decisions/D1-scope-localised-items.md)).
  Visibility is mechanical demotion only ([D2](decisions/D2-visibility-mechanical-demotion.md)).
  Findings are fixed when small and filed when large ([D3](decisions/D3-fix-small-file-large.md)).
  The sprint is one PR, with sessions by module cluster ([D4](decisions/D4-module-clusters-one-pr.md)).
  The decision concepts' wording awaits the user's review before they are stamped
  `verified` and moved to `stable`.
* **Claims check**: no open PRs on nsmith-/vibegraph, so no claimed item conflicts
  and no other PR is touching `helas/eval`.
* **Amendment** (D1, at the user's request): the first scope read only
  `backlog/hygiene/`. A filename grep over `backlog/validation/` found one item,
  and the other 45 went unread. D1 now states a rule that applies to every area:
  the fix is local, it needs no MadGraph or Pythia run and no seed
  re-measurement, and it settles no open physics question. Re-reading
  `validation/` against that rule added three items, all fixed in F-G:
  jj-banked-orderings-eta-uses-wrong-components, config-amp-phase-and-sign-unpinned
  and smeftsim-vendored-checksum-not-hermetic. It kept
  jioxxx-reference-port-comparison-has-no-teeth (rule 3) and
  pythia-gate-momenta-unchecked (rule 2) out.
* **Amendment** (review protocol): each review starts by listing the filed items,
  in any area, that name its cluster's paths, so it does not report filed work
  as new findings. Rediscovered findings would inflate the hit rates the lessons
  measure.
* **Approval** (human:nsmith-): D1–D4 reviewed and signed off, D1 including the
  amendment above. All four are stamped `verified` and moved to `stable`.
* **T1 landed** (fast-forward of `hygiene-t1`, commits f578bde..a4db76f): four
  items meet `closes_when`. `acceptance-yml-fails-on-refdata-releases` waits for
  the next `refdata-*` release. The report is recorded as `sessions/T1-report.md`,
  machine-confirmed by the manager's re-run of the lints and demonstrations.
  Three Found entries, to be filed at close-out.
* **V1 reported** (`hygiene-v1`, 827e098..34f92ea). The manager re-ran fmt,
  both clippy configurations and the hermetic suite (35 suites, 1348 passed,
  0 failed, 17 ignored) and reproduced the `pub` counts. The report is recorded
  as `sessions/V1-report.md`, machine-confirmed. It is not yet merged.
* **Decision** (human:nsmith-, on V1's results), amending
  [D2](decisions/D2-visibility-mechanical-demotion.md):
  - V1's 144 `#[allow(dead_code)]` are replaced by `cfg(test)` or
    feature gating, or by deletion.
  - The ~120 intra-doc links V1 turned into code spans are restored, and
    every rustdoc build documents private items.
  - This work runs as **V1b**, in V1's worktree, by a fresh agent seeded with
    V1's report rather than a resumed V1. Its transcript was about 333k
    tokens, and the user asked for it compacted.
  - The reviews now depend on V1b.
* **V1b reported** (`hygiene-v1`, 22f1955..371f854). The report is recorded as
  `sessions/V1b-report.md`. The manager's gate re-run is in progress, and the
  merge waits on it.
* **V1 and V1b merged** into the sprint branch. Manager gate re-run at `371f854`:
  - `cargo fmt --all --check` passes;
  - `cargo clippy` with `-D warnings` exits 0 in both configurations;
  - `cargo test --workspace` exits 0 (35 suites, 1348 passed, 0 failed,
    17 ignored), unchanged from before V1;
  - `cargo doc --workspace --no-deps --document-private-items` exits 0, with
    28 lib warnings and 1 bin warning. All 16 unresolved links predate V1.
* **Reviews dispatched**: R-A to R-G2, each a `claude` (Opus) agent in its own
  detached, read-only worktree at `f7efda6` (`/home/user/wt/hygiene-r-<x>`).
  The leads from V1b's Found item 4 (production contracts documented on
  test-only APIs) went to the clusters that hold them.
* **R-E reported**: 15 findings, recorded as `sessions/R-E-report.md`. The
  manager spot-checked R-E.1, R-E.8 and R-E.11. Triage waits for all eight.
* **R-D reported**: 17 findings, recorded as `sessions/R-D-report.md`. The
  manager spot-checked R-D.2, R-D.4 and R-D.11. Protocol note for later reviews:
  grep path tails (`/vegas.rs`), not bare names, in the backlog check.
* **R-B reported**: 25 findings plus one rediscovery (R-B.0), recorded as
  `sessions/R-B-report.md`. The manager spot-checked R-B.1, R-B.5, R-B.10 and
  R-B.14.
* **R-C reported**: 20 findings, recorded as `sessions/R-C-report.md`. The
  manager spot-checked R-C.1, R-C.5 and Found 1–2. Three reviewers have now
  hit the empty `mg5amcnlo` submodule. Fix sessions get it copied in, and the
  dispatch procedure should say review worktrees need it too.
* **R-G1, R-G2 and R-F reported**: 27, 22 and 28 findings, recorded as
  `sessions/R-G1-report.md`, `R-G2-report.md` and `R-F-report.md`, with the
  manager's spot checks in each. Seven of eight reviews are in; R-A is
  outstanding.
* **R-A reported**: 21 findings, recorded as `sessions/R-A-report.md`. All
  eight reviews are in, with 175 findings in total. Triage follows.
* **Reference data**: zstd installed, and the pinned refdata-9 bundle fetched
  with `fetch_refdata.sh` (consent by env), so fix sessions can run the banked
  layer.
* **Triage** written as `triage.md`: 176 findings, 125 fixed here across nine
  fix sessions (F-F split into F-F and F-CLI), 50 filed as 35 new items at
  close-out, 4 rejected or close-out only, 3 for the user. PDF set fetched
  (`validation/pdf/fetch.sh`). Fix sessions wait on the user's three calls.
* **Decisions** (human:nsmith-, on the triage):
  - R-A.17: leave lorentz-coefficients-still-f64 open, with no decision
    recorded.
  - R-G1.5: F-G1 moves the whole `smeftsim` and `toy_models` targets into the
    hermetic layer.
  - R-G2.1: the seed-combination contradiction is filed `needs-user`, with no
    gate change now.
* **Fix briefs** written: F-A, F-B, F-C, F-D, F-E, F-F, F-CLI, F-G1 and F-G2,
  with a shared `sessions/fix-protocol.md`. They run serially in that order,
  each from the previous one's merged head.
* **F-A dispatched** (`performance-dev`, Opus) on branch `hygiene-fa` from
  `ea3fd20`, in the warm worktree `/home/user/wt/hygiene-v1`. The refdata
  bundle, PDF set and `mg5amcnlo` content were copied in. The review and T1
  worktrees were removed and the debug incremental cache cleared, which freed
  about 9 GB.
* **Container restart during F-A.** F-A's process was lost with no commits,
  but 24 files of uncommitted work (+760/−1029) and its program-dump
  baselines survived on disk. The manager saved a backup patch and resumed
  the agent from its transcript with a reconciliation step: verify, commit in
  checkpoints, re-run what was in flight (session scoping rule 7).
* **Second container restart during F-A**, after 5 commits (9bb2849..c01b93a),
  with its gate run in flight. Clippy had found one `useless_vec` in the lib
  tests. The disk was down to 2 GB free because of the debug incremental cache,
  which the manager cleared again (9.8 GB free). The agent was resumed to fix
  the lint and re-run the gate with `CARGO_INCREMENTAL=0`.
* **F-A reported** (`hygiene-fa`, 9bb2849..5c92658, 7 commits). The report is
  recorded as `sessions/F-A-report.md`. The manager's gate re-run (program
  probe, clippy, hermetic suite) is in progress, and the merge waits on it.
* **F-A merged.** Manager gate re-run at `5c92658`:
  - the program probe gives `PROGRAMS IDENTICAL TO BASELINE (45 sets)`;
  - clippy exits 0 in both configurations;
  - `cargo test --workspace` gives 35 suites, 1343 passed, 0 failed,
    16 ignored. That is 5 fewer passing than V1b's 1348, matching the
    deleted tests net of those added.
* **F-B dispatched** (`validation-dev`, Opus) on `hygiene-fb` from `17d6e84`,
  with F-A's `GammaJout` Found item added to its scope.
* **F-B reported** (`hygiene-fb`, 7d33250..2127cfe, 3 commits). The report is
  recorded as `sessions/F-B-report.md`. The manager's gate re-run is in
  progress, and the merge waits on it.
* **F-B merged.** Manager gate re-run at `2127cfe`: fmt passes, clippy exits 0
  in both configurations, `cargo test --workspace` gives 35 suites with 1340
  passed, 0 failed and 16 ignored (matching the agent's account), and the
  banked `color_cf_oracle` passes 97.
* **F-C dispatched** (`feature-dev`, Opus) on `hygiene-fc` from `abb1a16`.
* **F-C reported and merged** (`hygiene-fc`, 4c6d9cb..52f6f99, 6 commits).
  The report is recorded as `sessions/F-C-report.md`. Manager gate re-run:
  fmt passes, clippy exits 0 in both configurations, and the hermetic suite
  gives 1352 passed. The banked `sm_interned_blob` (2), `color_cf_oracle` (97)
  and `validate_madgraph_diagrams` (57) pass. The feyngraph gitlink and the
  regenerated SM blob were checked.
* **Decision** (human:nsmith-): close reweight-forbidden-onshell-guard-is-dead
  by option (b), a required `forbidden_onshell` argument to `ReweightPlan::new`
  with `OnShell::Forbidden` deleted. It runs in this sprint as **F-C2**, before
  F-D.
* **F-C2 reported and merged** (`ed5b166`). The report is recorded as
  `sessions/F-C2-report.md`. Manager check: all-features clippy exits 0, the
  reweight lib tests (33) and CLI unit tests (95) pass, and no
  `OnShell::Forbidden` remains. The agent's full suite gave 1353 passed, and
  the banked `reweight_mg_oracle` and `cli_reweight_proton` pass.
* **F-D dispatched** (`performance-dev`, Opus) on `hygiene-fd` from this merge.
