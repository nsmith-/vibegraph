# cut-refusal log

## 2026-10-20
* **Creation**: Sprint folder created.

## 2026-10-20
* **F1 checked** (cb18fd5): the manager re-ran `cli_unimplemented_cuts` (4 passed) and `cuts::tests` (53 passed) from the committed tree. F1 edited `ht-cuts-unimplemented` only to drop a link to the item this PR deletes; the manager accepts that edit as its own.
* **Close-out** (2026-10-20): HY1 checked; Found filed; sprint closed.

## 2026-10-20 (re-opened)
* **User request on the PR** (22:37): fix `collider-energy-sum-of-beam-energies`, `cli-test-helpers-copied-per-file` and `generate-reads-run-card-after-artifact` in this PR. The sprint re-opens for session F2; the close-out record is amended after it.
* **F2 checked** (30d51b4, 66b5551, 24b93ee): the manager re-ran the five touched CLI test binaries (5, 5, 10, 14, 2) from the committed tree and read `build_frames` (`proton.rs:2004-2019`), which places the partons at x_i·√s/2 and boosts by (x1−x2)/(x1+x2): equal beams only. `generate-reads-run-card-after-artifact` and `cli-test-helpers-copied-per-file` are closed. `collider-energy-sum-of-beam-energies` stopped at the finding, as the user asked: the fixed-energy path is already exact; the proton path needs √s plumbing, a `build_frames` fix and an unequal-beam MadGraph reference; put to the user. Filed `fixed-beam-written-event-frame-unsettled` from F2's Found 1.
* **User decision** (2026-10-21, relayed by the planner): option (b). `collider-energy-sum-of-beam-energies` moves to its own PR (stacked on this branch, since the item was filed here); its text now states the whole proton frame chain is equal-beam only. This PR closes the other two items after a hygiene pass on F2.
* **HY2 checked** (05c0a72, 6b68be5): the manager re-ran the six touched CLI binaries (5, 5, 10, 14, 3+1 ignored, 2) and fmt. HY2 found 13 CLI test files still build the command by hand and 35 sites inline the stderr read, short of `cli-test-helpers-copied-per-file`'s closes_when: a migration session (F3) follows. The CERN host's fixed-beam frame measurement went into its item and the MadGraph defects register.
