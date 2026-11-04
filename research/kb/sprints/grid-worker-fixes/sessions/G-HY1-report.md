---
type: Session Report
title: "G-HY1 report: hygiene review of #87"
description: "The carried-flavour test now kills first-wins and the antiparticle-leg shortcut; the cache-root variable is named in the library; pdf_tiers cannot fail and says so; one integrate-and-generate harness replaces three; the no-home CLI tests do not run on CI."
status: draft
tags: [hygiene, report, sprint-f]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-11-04}
---
The session returned its report as a message, and the manager transcribed it
here, abridged. Commit eb03896. Logs are in the manager's scratchpad, under
`G-HY1/`.

## Findings (written before any edit)

| id | site | finding | disposition |
|---|---|---|---|
| F1 | `carried_flavour` test | survives first-leg-wins and dropping `conjugate_pdg` (7 passed); the antiparticle-leg branch is never reached | fixed: antiparticle-leg case, conflict case in both orders |
| F2 | `NoCacheRoot` | hardcodes `$VIBEGRAPH_HOME`; the constant lives in the CLI | fixed: `CACHE_ROOT_VAR` in `vibegraph-lib/src/cache/mod.rs`, advice text pinned |
| F3 | `assets::pdf_tiers` | returns `Result`, but cannot fail | fixed: returns `Tiers`; three `map_err` removed |
| F4 | `cli_generate_proton.rs` | integrate-then-generate written three times | fixed: `integrate_and_generate` |
| F5 | ttx share test | about 220 lines with inline reader and estimator | fixed: `PopulationEvent`, `population_events`, `weighted_share` |
| F6 | `member_line_pdg` doc | did not say that conflicting legs fall to the charge rule | fixed |
| F7 | G-F1 CLI tests | skip on non-root CI, invisibly | left; Found |
| F8 | llj fraction test | pooled binomial, `gzip -dc` | left (a filed item) |

## Mutations

| mutation | old test | new test |
|---|---|---|
| first leg wins | passes | fails |
| `conjugate_pdg(member[k])` → `member[k]` | passes | fails |
| last leg wins | fails | – |
| bypass `carried_flavour` | fails (two tests) | – |
| `CACHE_ROOT_VAR` renamed | not pinned | fails |

The ttx share test applies the same closures to both files, so a mutation
inside the test's own share logic cancels.

**Identity oracles for the refactors:**
- the ttx test's 19 printed lines are byte-identical before and after;
- the matched-sample events' debug-hash is unchanged (321f069e998a3333);
- F3 is checked by the compiler.

## Gates

- fmt and clippy (default and `--all-features`): clean.
- `cargo test --workspace`: passed.
- `cargo doc`: 8 warnings, all of them existing before this change.
- The banked `cli_generate_proton`: 11 passed, 2 ignored.
- `cli_pdf_cache`: 12 passed, 1 ignored.

## Found

1. CI never runs the no-home CLI tests (F7). Filed as
   `no-home-cli-tests-skip-on-ci`.
2. The llj fraction test should reuse the new helpers when it is promoted.
   That belongs to `llj-fraction-test-ignored-though-banked`.
3. The skip message misnames `PermissionDenied`. Folded into item 1.
