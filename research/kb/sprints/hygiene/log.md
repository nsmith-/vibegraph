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
