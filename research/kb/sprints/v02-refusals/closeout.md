---
type: Sprint Record
title: "v0.2 refusals: panics and silent wrong answers"
description: "Sprint G part one (#88): spin 2, Lorentz and spin-code panics, defaulted UFO fields, MadGraph's restriction zero rule, all three input cards classified, decay tables carried, negative masses refused off Majorana fermions, and σ = 0 or too few effective points refused; six items closed, eleven filed."
closed: 2026-11-04
status: draft
tags: [refusal, panic, ufo, param-card, restrict-card, reweight, sigma, sprint-g, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: sprint, resource: "sprint.md", title: "Sprint overview and decisions G1–G7"}
  - {id: log, resource: "log.md", title: "Sprint log"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/88", title: "nsmith-/vibegraph#88"}
---
Part one of sprint G, on #88. The plan, decisions G1–G7 and the session table
are in [the sprint overview](sprint.md); every check the manager made is a
dated line in [the log](log.md).

## What landed

| Session | Change | Evidence |
|---|---|---|
| G-F1 | Spin 2 (and spin 3/2) refused at `AmplitudeEvaluator::compile` on any line; a Lorentz index naming no leg refused at rooting; `build_child`'s `todo!` and three spin-code `panic!`s gone. | 5 refusal tests panicked on the base; [report](sessions/G-F1-report.md). |
| G-F2 | Every UFO field MadGraph's `object_library` requires refused by name; duplicate names refused; a parameter's UFO `name` is canonical; `from_model_card` fallible (Auto width a typed error); MadGraph's restriction zero rule ported. | 51 field cases and 8 zero-rule cases flipped; vertex census of 56 (model, card) pairs unchanged except MSSM_SLHA2 815 → 812, which equals MadGraph's count. |
| G-F3 | Classification tables for the param (25 rows), restrict (24) and reweight (45) cards, each asserted by running its examples; `propagators.py` fields required. | 18 of 20 refusal tests failed on the base (2 controls). |
| G-F4 | `integrate` refuses σ = 0 (naming cut rejection or a vanishing matrix element) and an effective sample size below 20, writing no artifact; `generate` refuses a σ = 0 artifact. | Lowest legitimate ESS measured 148, banked rows ≥ 1.1e5, card 048 0–2. |
| G-F5 | A param card's decay tables recorded (artifact format 18) and written into `<slha>`; MadEvent's negative-mass width rule ported and pinned against MadGraph on `ee_to_mumu_negmz` (1e-13); a negative mass off a Majorana fermion refused at binding (G7 amended). | All nine gate steps on 91b4f60. |
| G-HY1 | 15 findings: three vacuous tests tightened (the C census split by layer), shared readers, docs. | Hermetic suite in a submodule-free clone: 1883 passed. |

Closed: `spin-2-panics-instead-of-refusing`,
`lorentz-and-spin-code-paths-panic-instead-of-refusing`,
`ufo-required-fields-default-silently`,
`restriction-zero-coupling-threshold-differs`,
`input-cards-lack-field-classification`,
`zero-cross-section-reported-as-ok`.

## Filed

From the sessions' Found sections: `spin-three-halves-refusal-gaps`,
`survey-models-unswept-for-charge-conjugation`, `mssm-interaction-count-unbanked`,
`ufo-optional-files-and-running-py-unchecked`,
`param-card-qnumbers-mismatch-not-caught`,
`restriction-conditional-simplification-unaudited`,
`zero-sigma-refusal-cause-too-coarse`, `generate-accepts-nan-or-negative-sigma`,
`negative-mass-rules-undecided`, `standalone-jamps-generator-fragile`,
`card-refusal-plumbing-duplicated`. Site (e) added to
`madgraph-compat-sites-unconditional`; a note added to
`reweight-helicity-summed-only`. G-F2 suggests
`restrict-card-unit-values-not-fixed` may be closable bar its MadGraph
param-card comparison, which can now run in a cloud container.

## Process lessons

- **Dev agents ran the hermetic suite and the gates their brief named, not the
  banked CLI targets.** CI's `banked` job caught a CLI test whose 200-point
  budget the new refusal stopped. Briefs that change `integrate`'s outcome now
  name the banked CLI targets.
- **A hermetic test read the MadGraph submodule** and passed only because the
  worktree had a copy; another skipped it at runtime. The manager's combined
  gate caught the first, G-HY1 the second. Running the hermetic layer in a
  clone without the submodule, as CI does, is the check.
- **MadGraph's `import model` runs in a cloud container** from the pinned
  submodule, so a model-level oracle (an interaction count, a restriction)
  needs no M3 relay.
- **Two container restarts** killed background runs; uncommitted work
  survived each time. Briefs now require detached long runs and checkpoint
  commits.
