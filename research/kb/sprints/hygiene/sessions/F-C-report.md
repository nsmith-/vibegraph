---
type: Session Report
title: "F-C report: model, diagram and reweight fixes"
description: "Four claimed items closed and all 14 findings fixed over 6 commits, with 18 run mutations; asin/acos, grouped divisors, locked-parameter recompute and make_anti fixed; the reweight $ guard stopped for a decision."
status: draft
tags: [hygiene, fix, ufo, diagrams, reweight, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-10}]
sources:
  - {id: 4c6d9cb, resource: "https://github.com/nsmith-/vibegraph/commit/4c6d9cb", title: "pin the feyngraph submodule to the build's rev"}
  - {id: fabb660, resource: "https://github.com/nsmith-/vibegraph/commit/fabb660", title: "process, model and artifact docs match the code"}
  - {id: cbca25f, resource: "https://github.com/nsmith-/vibegraph/commit/cbca25f", title: "asin/acos, grouped Lorentz divisors, locked parameters, vertex name lists"}
  - {id: 055ca79, resource: "https://github.com/nsmith-/vibegraph/commit/055ca79", title: "antiparticles keep singlet and octet colour"}
  - {id: 8f9ac99, resource: "https://github.com/nsmith-/vibegraph/commit/8f9ac99", title: "every resolve and check refusal named; refuse only scan values"}
  - {id: 52f6f99, resource: "https://github.com/nsmith-/vibegraph/commit/52f6f99", title: "clippy type_complexity"}
---
The dev agent's report, condensed by the manager. Everything except the
manager check is the agent's claim. The mutation logs are in the scratchpad
(`fc-mut.log`): 18 mutations, each failing its test and then reverted.

## Fixed

| finding or item | commit | change | evidence |
|---|---|---|---|
| ufo-asin-acos-evaluate-as-acsc-asec, R-C.4 | cbca25f | `Func::ASin`/`ACos` (appended, so the blob indices hold); a 27-spelling table test; `cot`/`theta_function`/`cond`/`reglog` refusal test | Three mutations, including the old mapping, fail |
| R-C.5 | cbca25f | `div_terms` returns `Result`: a numeric group divides, a Lorentz divisor is refused | The old `_ => lhs` fails |
| R-C Found 1 (stop-rule bug) | cbca25f | `recompute` returns early on a restriction-locked parameter | The test failed with the old order and passes after the fix |
| R-C.8 | cbca25f | `recompute` reuses `dependents` (HashSet); one graph walk | Existing tests |
| R-C.9, R-C.16 | cbca25f | Duplicate extractors deleted; `ast_util::named_assignments`/`constructor_calls` replace seven loops | — |
| R-C.10 | cbca25f | Name-list prefix checked and refused | Removing the check fails |
| R-C.18 (test) | cbca25f | Malformed SLHA lines are parse errors at their line | Two mutations fail |
| make-anti-negates-singlet-octet-colour | 055ca79 | 1 and 8 kept, 3 and 6 negated; `from_ufo` rejects −1/−8; resonance `abs()` workaround dropped; SM blob regenerated | The SM test failed before regeneration; the re-negating mutation fails |
| R-C.2 | 8f9ac99, 52f6f99 | Every `ResolveError`/check refusal asserted by variant | Five disabling mutations fail |
| R-C.3 | 8f9ac99 | `!a!`, `--modelname` and `add model` refusals tested | Two mutations fail |
| R-C Found 2 | 8f9ac99 | `is_scan_value` matches only the `scan:`/`scanN:` prefix | The old substring check fails |
| R-C.6, .7, .20, process-model-and-artifact-doc-comments-stale | fabb660 | Docs match code (8 item sites) | — |
| feyngraph-submodule-pin-differs-from-build | 4c6d9cb | Gitlink set to `fd5aa83`; README names the pin | Verified in cargo's git DB: `fd5aa83` is 8 commits after `1dc4ea7` |

**make_anti's readers of `Particle::color`** were listed before the change.
Most go through `abs()` or `from_ufo`. `compile.rs` `config_tag` reads the
signed value (Found 2).

## Stopped

- **reweight-forbidden-onshell-guard-is-dead.** `ReweightPlan` sees only
  `DiagramSet`s, which carry no `$` record. The options were:
  - (a) assign `OnShell::Forbidden` in enumeration, as MadGraph's
    `diagram_generation.py:781` does, which changes enumeration output;
  - (b) delete the never-assigned variant and give `ReweightPlan::new` a
    required `forbidden_onshell` argument.

  The agent recommended (b). **The user chose (b), done in this sprint as
  F-C2** (log, 2026-10-10).

## Gate (agent, at 52f6f99)

- **fmt** and **clippy, both configurations:** pass.
- **`cargo test --workspace`:** 1352 passed, 16 ignored, with 12 added tests.
- **`cargo doc`:** 9 lib warnings, unchanged. One run hit a lib/bin
  `target/doc/vibegraph` name collision (Found 5).
- **Banked:** `sm_interned_blob` 2, `ufo` 4, `smeftsim` 13, `toy_models` 4,
  `color_cf_oracle` 97, `color_flow_tags_oracle` 163,
  `validate_madgraph_diagrams` 57, `validate_lhef` 3; and in the CLI,
  `cli_decay_chain_events` 7 and `cli_reweight_proton` 2.

## Found

1. **Kb concepts made stale:**
   - `model/ufo-string-grammars.md` (the latent-defect paragraph, the function
     list, the divisor rule);
   - `events/resonance-records.md:76-78` (−1/−8);
   - `references/codebases/feyngraph.md:33-36` (the old pin).
2. **`config_tag` puts the signed `particle.color` into the configuration
   tag;** MadGraph's `IdentifyConfigTag` uses the unsigned value. `u` and `u~`
   lines at one split tag differently. No banked gate moved. File it.
3. **SMEFTsim's `function_library` uses `z.real` for sec/csc/asec/acsc;** the
   grammar uses the SM forms. They agree on real arguments.
4. **A history comment** in `ufo/lorentz.rs` `test_ungrouped_division`.
5. **`cargo doc --workspace` races on the lib/bin `vibegraph` doc-name
   collision.**
6. **The reweight `$` decision** (resolved: F-C2).

## Brief corrections

- **The reweight item could not close locally.**
- **`make_anti` needs the SM blob regenerated** (`gen_sm_blob`), and
  `sm_interned_blob` is its gate.
- **`cargo check --tests` (about 15 s) is the better inner loop.**

## Method notes

- **`pkill -f`** twice killed the agent's own wrapper shell, one of them a
  gate run. Kill by PID.
- **One sleep-loop wait** was written, against the discipline.

## Manager check (2026-10-10)

- **Commits and trailers:** six commits, `Assisted-by` only. The diff is
  25 files, +512/−247.
- **Gitlink:** `research/refs/feyngraph` is at `fd5aa83`, equal to
  `vibegraph-lib/Cargo.toml:34`.
- **SM blob:** `sm_parsed.bin.zst` was regenerated (5115 → 5108 bytes).
- **Gate re-run:** fmt passes; clippy exits 0 in both configurations;
  `cargo test --workspace` gives 35 suites, 1352 passed, 0 failed,
  16 ignored; the banked `sm_interned_blob` (2), `color_cf_oracle` (97) and
  `validate_madgraph_diagrams` (57) pass.
