# v021-fixes log

## 2026-11-11
* **Creation**: Sprint folder created by the sprint-J manager session, started by the vibegraph manager session on the user's instruction.
* `send_message` to the vibegraph manager fails (`elevated_session_untrusted_device`), as it did in sprint I. Messages go through the user.
* `--madgraph-compat` does not exist on `main` (9a1349c). The plan's option A (the weighted default only, no flag) was put to the vibegraph manager and the user. J-W1 waits on the answer.
* The user decided: skip `validate-deep`; the CERN host runs the full CMS campaign on the release-candidate head.
* Plan posted (sprint.md; sessions J-W1, J-A1, J-E1, J-E2). New item `diagram-artifacts-replay-stale-enumeration` filed here, to be closed by PR 2.
* J-A1 (feature-dev, Opus), J-E1 (validation-dev, Opus) and J-E2 (validation-dev, Sonnet) dispatched on `sprintj-stale-artifacts`, `sprintj-panics` and `sprintj-sde`, each worktree off main 9a1349c, with a shared `CARGO_TARGET_DIR`.
* The vibegraph manager relayed the user's answer: **option A**. The weighted cut is the only behaviour; MadGraph's single pairing stays as site (d) of the feature item, whose text says so; the MadGraph σ is an attributed difference, and `release-scope.md` and the deviation text say the same. The manager approved PR 2's design and asked that its blind spot be named in the concept. J-W1 dispatched.
