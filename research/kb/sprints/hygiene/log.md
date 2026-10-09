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
