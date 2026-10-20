---
type: Session Report
title: "F1 report: MadGraph's default particle names, and --modelname"
description: "import model renames particles to MadGraph's PDG-keyed default names in a lookup table on the model (resolution follows MadGraph, outputs keep the UFO names); --modelname switches it off; the table is vendored and hash-pinned; 9 new tests, 10 of 10 mutations caught."
status: draft
tags: [process-grammar, model, particle-names, madgraph-parity, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: 63f50a6, resource: "https://github.com/nsmith-/vibegraph/commit/63f50a6", title: "feat(model): MadGraph's default particle names; accept --modelname"}
  - {id: mg-base-objects, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/core/base_objects.py#L1577-L1623", title: "Model.pass_particles_name_in_mg_default at the pinned commit"}
  - {id: mg-interface, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/interface/madgraph_interface.py#L1256-L1322", title: "check_import at the pinned commit"}
---
Session F1 of [the sprint](../sprint.md), branch `sprintd-default-particle-names`,
commit `63f50a6` (author Nicholas Smith). It closes
`madgraph-default-particle-names-missing`.

## What MadGraph does (mg5amcnlo `b7687064`)

- **When:** `do_import` calls `self._curr_model.pass_particles_name_in_mg_default()`
  after the model loads, unless `'-modelname' in args`
  (`madgraph/interface/madgraph_interface.py:5841-5842`).
  `check_import` (`:1259-1266`) takes out `-modelname` or else `--modelname`
  (one occurrence, from anywhere in the args, before the model name is read) and
  appends `-modelname` back (`:1320-1321`). So both spellings are accepted.
  Help text: `:355`, "'--modelname' keeps the original particle names for the model".
- **The table:** `Model.load_default_name` (`base_objects.py:1759-1777`) reads
  `input/particles_name_default.txt`. It left-strips each line and skips lines that
  start with `#`. Other lines that do not split into exactly two fields are skipped
  with a warning. It stores `int(code) -> name.lower()` in a dict (file order).
  PDG 25 has no entry.
- **The renaming** (`base_objects.py:1577-1623`): it walks the table in order. For
  each code the model has (`get_particle(pdg)`, where antiparticles are copies with
  the negated code), a name that differs from the default must pass
  `check_name_free`. That function raises `MadGraph5Error` ("... is in conflict
  with MG convention name ... Use -modelname ...") if any particle's *current* name
  or antiname already is the default. The particle then takes the default. For
  `is_part` this sets `name`, and for a self-conjugate particle the antiname too.
  For the antiparticle code it sets the particle's `antiname`. **The antiparticle is
  renamed through its own table entry** (13 and -13 are both listed). The check runs
  against names as they stand mid-renaming. After the loop, a model whose
  `name == 'mssm'` or starts with `mssm-` sets PDG 25 to `h1`, with no check
  (`:1619-1623`). The model name is the directory's base name
  (`models/import_ufo.py:412`).
- **Old names afterwards:** the renaming is in place, and process lines resolve
  through `ParticleList.find_name` / `get_copy` (`base_objects.py:526-577`, called
  from `extract_particle_ids`, `madgraph_interface.py:5604`), which match only the
  current `name`/`antiname`. **So `m+` on `loop_sm` is refused by MadGraph** ("No
  particle m+ in model") without `--modelname`.
- **Other `import model` options:** `--noprefix` (`:1268-1270`, the `mdl_`
  parameter prefix) and `--last` (`:1272`, the most recent model directory). Both
  stay refused as `Unsupported::ModelOption`. MadGraph silently ignores any other
  extra token (only `args[1]` is read), so `--modelname=x` is *not* `--modelname`
  to MadGraph: renaming stays on. vibegraph refuses it.

## Rename vs alias: neither; a renamed lookup table

The decision, and why no output changes (so the STOP condition did not trigger):

- A pure **alias** (both names resolve) accepts `m+` on `loop_sm`, which MadGraph
  refuses, and `resolve.rs` reads cards "as MadGraph reads it".
- A real **rename** of the `Particle`s would change the name every downstream
  consumer prints.
- Implemented instead: the renaming builds `ufo::names::ParticleNames`, a table on
  `UFOModel` from accepted name to the UFO name. The particles, and every
  `IndexMap` keyed by UFO name, are untouched. `particle_by_name` (and through it
  every leg, label member, `/`, `$`, `$$` and `>` list) resolves through the table.
  A UFO name that was renamed away gets a new `ResolveError::RenamedParticle`, which
  names the default and `--modelname`.

Where a name leaves the library, the checked list:

- `DiagramSet::label` reaches logs, TUI lines (`generate.rs:1949`), `proton.rs:682`,
  and `onshell.rs` / `diagrams/mod.rs` error messages.
- LHEF records PDG codes only (`lhef/`).
- Artifacts identify the model by `ModelIdentity` (an asset digest), not by names.
- `onshell.rs:420` and `diagrams/mod.rs` charge and helicity lookups take UFO names
  from `leg_names` output.
- MG oracles: `proc_grammar_oracle` compares PDG codes. The diagram and amplitude
  oracles run only on the SM, SMEFTsim and the toys, whose names equal the
  defaults up to case, which resolution already ignored.

All of these print the UFO's own name, unchanged. MadGraph would print `mu+`
where vibegraph prints `m+` in a subprocess label for a `loop_sm`-like model. No
gate compares those strings. This already happens for SMEFTsim (`Z` against
MadGraph's `z`). Pinned: `particle_names.rs` asserts the label `up up~ > m+ m-` for
`generate u u~ > mu+ mu-`.

`--modelname`: `ModelImport` gains `particle_naming: ParticleNaming`, set by the
parser when either spelling appears anywhere after `model`. Only the remaining
options go to `Command::ImportModel::options`, so `check.rs` refuses
`--noprefix` / `--last` / `--modelname=x` / `add model` as before. `GlobalConfig`
loads a non-SM model with `UFOModel::load_with_digest_and_naming`. The interned
SM is returned as is under either naming: its names are the defaults up to case
(`names::tests::the_sm_is_unchanged`, all nine restrictions). The model digest
does not depend on naming.

One divergence: MadGraph lowercases a model's names before renaming when they
stay distinct lowercased. This is not replicated, because it cannot change the
outcome (see the next section).

## Gate

All from `/home/user/wt/F` with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`, on
the final tree (log `namesF-F1-gate3.log`):

```
== cargo fmt --all --check
exit 0
== cargo clippy --workspace --all-targets -- -D warnings
exit 0
== cargo test --workspace
exit 0          # 41 result lines: 1553 passed, 0 failed, 18 ignored
     Running tests/particle_names.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
     Running tests/proc_grammar_oracle.rs
test result: ok. 1 passed; 0 failed; ...
     Running tests/validate_helas.rs
test result: ok. 1 passed; 0 failed; ...
     Running tests/validate_madgraph_diagrams.rs
test result: ok. 1 passed; 0 failed; ...
== python3 scripts/kb.py lint
829 concepts, 290 backlog items; 0 errors, 4 warnings
exit 0
```

The 4 kb-lint warnings are on unrelated backlog files (`state: blocked` without
`blocked_by`), identical before this change.

The first full run failed on one existing test,
`resolve::tests::a_repeat_token_that_names_a_particle_is_ambiguous`. It inserts a
particle into `model.particles` directly, and the name table is derived from the
particles, so the table went stale. The test now rebuilds the table after the
insertion (`with_particle_naming`).

## Mutation evidence

Script `namesF-F1-mutate.py`: apply, run the named tests, restore and `touch`.

| # | Mutation | Caught by |
|---|---|---|
| M1 | `particle_by_name` bypasses the table (the old exact/case-insensitive lookup) | all 3 `particle_names.rs` tests |
| M2 | parser sets `MadGraphDefaults` on `--modelname` | `modelname_option`, `modelname_is_accepted`, `modelname_keeps_the_models_names` |
| M3 | `GlobalConfig` ignores `import.particle_naming` | `modelname_keeps_the_models_names` |
| M4 | no `check_name_free` | `a_taken_default_name_is_refused` |
| M5 | conflict checked against the original names, not mid-renaming | `renaming_is_sequential` |
| M6 | vendored table `22 a` → `22 A` | `vendored_table_is_the_pinned_file` |
| M8 | no MSSM `h1` rule | `mssm_calls_the_light_higgs_h1` |
| M9 | renamed names reported as plain `UnknownParticle` | `renamed_model_names_are_refused` |
| M10 | default names not lowercased | `parse_rules` (after the fix below) |
| M11 | `--modelname` left in the refused options | `modelname_option`, `modelname_is_accepted`, `modelname_keeps_the_models_names` |

Every caught row was exit 101, and after restore the lib and integration suites
pass (12 + 3). The first sweep found two real gaps:

- **M10 survived:** `parse_rules` overwrote its only uppercase entry with a later
  duplicate. The test was rewritten so that `T` and `T~` must lowercase.
- **M7 (drop the case folding) survived:** the step was unobservable. A model
  whose names are distinct when lowercased has no particle whose exact name is
  another's lowercased one, so a case-only difference (`Z` for `z`) takes its
  default unopposed, and resolution ignores case. The folding code was removed
  and the doc comment says why. M7 no longer exists.

## Files

- New: `vibegraph-lib/src/ufo/names.rs`,
  `vibegraph-lib/src/ufo/mg_input/particles_name_default.txt` (byte for byte,
  sha256 `8b4f80eb…1719e64`), `vibegraph-lib/tests/particle_names.rs`, and
  `validation/ufo/vibegraph_toy_names_UFO/` (authored: muon `m-`/`m+`, up quark
  `up`/`up~`, photon `a`).
- Changed:
  - `vibegraph-lib/src/ufo/{mod.rs,sm.rs}`
  - `vibegraph-lib/src/diagrams/{resolve.rs,parse.rs,check.rs}`
  - `vibegraph-lib/src/config.rs`
  - `vibegraph-cli/src/assets.rs` (a struct literal)
  - `THIRD-PARTY-NOTICES`, a new section for the table
  - `research/kb/tooling/licensing.md`
  - `validation/ufo/README.md`
- Deleted: `research/kb/backlog/feature/madgraph-default-particle-names-missing.md`.
- Link changed: `research/kb/validation/cms-gridpack-survey-2026-10.md` "Backlog
  filed from this survey" now reads "default particle names and `--modelname`
  (closed by [the sprint])", following the precedent in
  `seed-headroom-census-2026-09.md`.

## Found

- **`UFOModel.particles` is `pub` and the name table is derived from it.**
  Evidence: the stale-table failure above. Any code that edits
  `model.particles` after load must call `with_particle_naming` again, and nothing
  enforces it. To close it: make `particles` private behind accessors, or have
  resolution check that the table and the particle set agree in debug builds.
- **`UFOModel` docs say the maps are "keyed by the Python variable name";
  particles are keyed by the UFO `name`** (`ufo/mod.rs`, the
  `ParsedModel::parse` `.map(|p| (p.name.clone(), p))`). To close it: correct the
  doc comment.
- **A UFO particle without an `.anti()` entry has an unresolvable antiname.**
  `particle_by_name` (before and after this change) matches entries only. MadGraph
  matches `antiname` on the particle itself. Evidence: `find_name`
  (`base_objects.py:563-577`) against `ParticleNames::of_model`, built from
  entries. To close it: synthesise antiparticle entries at parse when a UFO omits
  them, or index antinames too. Then check any CMS-survey card that fails this
  way.
- **Subprocess labels print UFO names where MadGraph prints default names** (`up
  up~ > m+ m-` against `u u~ > mu+ mu-`; already `Z` against `z` on SMEFTsim).
  This is human-facing only and no gate reads it. If MadGraph-identical labels are
  ever wanted, `DiagramSet::label` could print through the inverse of
  `ParticleNames`.
- **`--modelname=x`, `--noprefix` and `--last` are refused**, though MadGraph
  ignores the first and the other two do not change what is generated
  (`--noprefix` renames parameters, `--last` picks a directory). To close it:
  accept `--noprefix` as benign if a survey card needs it. This is out of this
  item's scope.
