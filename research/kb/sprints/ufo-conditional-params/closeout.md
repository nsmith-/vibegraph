---
type: Sprint Record
title: "UFO definitions under 'if configuration' are refused by name"
description: "A UFO object defined only under 'if' is refused at load naming it (MadGraph supplies no configuration to pick a branch); no public entry point panics on an unknown parameter."
closed: 2026-10-20
status: draft
tags: [ufo, parser, panic, sprint-d, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: f1, resource: "sessions/F1-report.md", title: "F1 report"}
  - {id: f2, resource: "sessions/F2-report.md", title: "F2 report"}
  - {id: hy1, resource: "sessions/HY1-report.md", title: "HY1 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/63", title: "nsmith-/vibegraph#63"}
  - {id: mg-models, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/models/__init__.py#L97-L102", title: "MadGraph imports the UFO package as-is"}
---
One PR ([nsmith-/vibegraph#63](https://github.com/nsmith-/vibegraph/pull/63))
closing `ufo-parameters-conditional-branches-panic`. Sessions:
[F1](sessions/F1-report.md) and [F2](sessions/F2-report.md) (feature-dev,
Sonnet; F2 the same agent resumed) and [HY1](sessions/HY1-report.md)
(hygiene-dev, Sonnet).

## What was established

- **Refuse, not choose a branch.** MadGraph supplies no `configuration`: it
  puts the model directory on `sys.path` and imports the UFO package
  (`models/__init__.py:97-102` at b7687064), and `import_ufo.py` has no
  handling. The flag's value is whatever the UFO itself imports, so there is
  no default to read without running Python. A `Parameter`, `Particle`,
  `Coupling`, `Vertex`, `Lorentz`, `CouplingOrder` or `Propagator` defined
  only under an `if`/`elif`/`else` (at any depth) is refused at load, naming
  the file and object.
- **No public panic on an unknown parameter.** Model load refuses an
  internal expression or coupling naming an undefined parameter;
  `EvaluatedModel::recompute` and `set_alpha_s` return `UnknownParameter`;
  `ReweightPlan::new` refuses a launch naming an unknown parameter.
  `expr::eval` keeps its panic as an internal invariant whose doc lists the
  guard on each public path. F1 alone stopped short of the item (the public
  `recompute` still panicked); F2 closed that.
- **The no-`aS` rescale path is unreachable, and tested as such.** A
  scale-dependent plan needs a pool power of `G`, which only a model with
  `aS` produces; a model without one never rescales, and a test pins that the
  per-event α_s leaves its pools and |M|² bit-identical.
- **Pinned by** toy-model tests through `try_edited_toy_model` (every refused
  file/constructor pair, exact file labels) and unit tests, each with mutation
  evidence; dim6top_LO_UFO itself (survey cards 034, 035) was not run.

The manager re-ran the toy-model and reweight tests from the committed tree
after each session.

## Backlog

- Deleted: `ufo-parameters-conditional-branches-panic`.
- Filed: `reweight-launch-accepts-internal-parameters` (validation, low,
  suspected), `evaluated-model-param-values-public` (hygiene, low),
  `ufo-conditional-statements-skipped` (feature, low).
