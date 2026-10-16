---
type: Session Brief
title: "HY1: hygiene review of the seeded banking driver"
description: "Review F1 and F2's driver, seed-record binary and gate reader for maintainability, test non-vacuity and duplication; write findings first, then fix with mutation evidence."
status: draft
agent: hygiene-dev (Opus)
depends_on: [F1, F2]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
Start from [F1's report](F1-report.md) and [F2's report](F2-report.md). The
change under review is this PR's code diff against `main`
(`git diff origin/main...HEAD -- . ':!research'`):
`validation/madgraph/{bank_seeded_sigma.sh, madevent_seeds.sh, build.sh,
gen_pta_windows.sh, gen_mlm_references.sh, gen_onshell_veto.sh,
assemble_bundle.sh, time_stages.py}`,
`validation-report/src/bin/seed-record.rs`,
`vibegraph-lib/tests/common/seed_record.rs`, the readers in
`validate_sigma.rs` / `validate_hadronic.rs`, and `pixi.toml`.

## Scope

1. **Findings first.** Write `HY1-report.md` with the findings before any
   edit. Cover:
   - duplication between `seed-record.rs` and `tests/common/seed_record.rs`
     (schema, checks, combination), and whether one should own it;
   - test non-vacuity: for each refusal (`SeedRecord::check`, `seed-record`,
     `mes_check_run`), show a mutation that removes the check makes a
     test fail; add the test where none does;
   - `seed-record.rs` size (1.3k lines): whether it splits cleanly;
   - shell: quoting, `set -euo pipefail` coverage, the `build.sh` sourceable
     main guard, dead options.
2. **Fix** what is in proportion, each with mutation evidence. Physics or
   statistics findings (for example the heavy-tail rule, which awaits a user
   decision) go in the report's "Found" section, not into code.
3. **Inert.** No record is banked; a gate with no record must behave exactly
   as on `main`. Keep that true and show it.

## Out of scope

- The `SCATTER` flag's rule (pending user decision); no change to its
  semantics.
- The other `mes_run_seed` callers not refusing empty `results.dat` (to be
  filed); note in Found if you see more.
- No banking, no refdata commits.

## Gate

`cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`
(use `-j 2`); `cargo test -p vibegraph-validation-report`;
`cargo test -p vibegraph-lib --test seed_combination`; the two gates F2 ran
against a copied record (`sigma_gate_matches_madgraph`,
`sigma_bb_fixed_scale_vs_mg`), with records removed after;
`python3 scripts/kb.py index --check` and `lint`. Delete `target/debug`
when done.
