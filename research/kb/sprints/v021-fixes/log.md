# v021-fixes log

## 2026-11-11
* **Creation**: Sprint folder created by the sprint-J manager session, started by the vibegraph manager session on the user's instruction.
* `send_message` to the vibegraph manager fails (`elevated_session_untrusted_device`), as it did in sprint I. Messages go through the user.
* `--madgraph-compat` does not exist on `main` (9a1349c). The plan's option A (the weighted default only, no flag) was put to the vibegraph manager and the user. J-W1 waits on the answer.
* The user decided: skip `validate-deep`; the CERN host runs the full CMS campaign on the release-candidate head.
* Plan posted (sprint.md; sessions J-W1, J-A1, J-E1, J-E2). New item `diagram-artifacts-replay-stale-enumeration` filed here, to be closed by PR 2.
* J-A1 (feature-dev, Opus), J-E1 (validation-dev, Opus) and J-E2 (validation-dev, Sonnet) dispatched on `sprintj-stale-artifacts`, `sprintj-panics` and `sprintj-sde`, each worktree off main 9a1349c, with a shared `CARGO_TARGET_DIR`.
* The vibegraph manager relayed the user's answer: **option A**. The weighted cut is the only behaviour; MadGraph's single pairing stays as site (d) of the feature item, whose text says so; the MadGraph σ is an attributed difference, and `release-scope.md` and the deviation text say the same. The manager approved PR 2's design and asked that its blind spot be named in the concept. J-W1 dispatched.
* J-A1 finished; PR 2 opened as #130 (fb3219a). J-E2 finished on `sprintj-sde` (7d5b472): the path is reachable (`e+ e- > z z, z > b b~, z > b b~` + `e+ e- > h z, …`), the forests were built over the union of every subprocess's diagrams; fixed per subprocess.
* The shared `CARGO_TARGET_DIR` ran sibling worktrees' stale test binaries (J-A1 caught it): workspace members hash identically across worktrees. Every session moved to its own target dir and re-ran its results.
* Three pre-existing amplitude defects surfaced: SMEFTsim dipoles at the amplitude root (J-E1); `u u~ > t t~ b b~ QED<=2`'s W-exchange class sign, also in plain `sm` (J-E1); the antiparticle-first ordering (J-W1), which the manager confirmed on `e+ e- > e- e+` (168.92 against 155.96 pb). J-F1 dispatched to diagnose the ordering.
* The user decided: **fix the bugs.** Read as all three defects. Two items filed here; J-F1 becomes the fix session for both sign defects (PR 5), J-D1 dispatched for the dipoles (PR 6).

## 2026-11-12
* The user approved #130–#134. #130 squash-merged as b76aa859; its item `diagram-artifacts-replay-stale-enumeration`, filed on this branch only, is removed here. The manager brings `main` into each following PR with a merge commit and re-runs the gates before the vibegraph manager merges it.
* #131, #132, #133 and #134 merged (a2a37b1a, 88e58cbc, 95119d91, 5c704bb7), each after a merge commit of `main` and the manager's targeted gates; CI gated. #133's merge enforced `uux_to_ttxbbx_4f_nodipole`; #134's rewrites moved into `build_rerooted`.
* `zz_ee` re-measured after #133: +0.25% ± 0.08% (was +0.26%). The per-event decomposition puts the interference at 4e-5 and the window at +0.11%; +0.14% is unattributed and filed.
* J-W1's rows re-measured on 5c704bb7: +0.60% (F), +0.10% (T), +0.05% (jj), with the interference now −0.46%. Docs, the release notes and the record updated; the CMS campaign requested on #129.
