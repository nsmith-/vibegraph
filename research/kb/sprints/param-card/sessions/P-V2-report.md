---
type: Session Report
title: "P-V2 report: the moved-parameter sigma rows gated on refdata-12"
description: "Both moved-parameter sigma rows pass their gate on the banked MadGraph runs (pulls +0.06 and +0.14, as the M3 measured); ee_to_tth_ymt250 is declared in the alpha_s and scales inventories; the ten banked targets pass; the collator wrote both rows' cells and flags only cells outside the targets run."
status: draft
tags: [validation, param-card, sigma, report, sprint-e]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-30}
---
Worktree `/home/user/wt/P`, branch `sprinte-param-card`, from `92d3e052`. Cargo ran
with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`,
`-j 2`, and `--config 'profile.release-debug.debug=0'` for release-debug targets.
Pixi is not installed, so each task's cargo command ran directly from a
resumable driver.

## Commits

- `7d9b2715`: the inventory declaration, alone (`validate_alphas.rs`,
  `validate_scales.rs`: one line each, `"ee_to_tth_ymt250"` before `"ee_to_ttx"`).
- the manifest commit: both rows unplanned, measured integrals cells in their
  notes, the tth diagrams note corrected (the process directory is banked, its
  census is not), and the two request-text fixes in `P-V1-report.md`
  (`pixi run -e madgraph python -I`; `grep -iE` for `# MT`).
- this report.

## The sigma gate

```
[pp_to_ll_mz95] GATE vibegraph σ = 7.4688953e2 ± 4.61e-1 pb (5 seeds, χ²/dof = 2.00) | MG σ = 7.4686000e2 ± 1.58e-1 pb | pull = +0.06 | rel = +0.00004
[ee_to_tth_ymt250] GATE vibegraph σ = 4.0961865e-3 ± 1.01e-6 pb (5 seeds, χ²/dof = 1.02) | MG σ = 4.0958200e-3 ± 2.35e-6 pb | pull = +0.14 | rel = +0.00009
```

Both equal the M3's (+0.06, +0.14). The reported-only ratio read
σ(250)/σ(default) = 2.03073 ± 0.00079 against the reweight 2.03726 ± 0.00550
(pull -1.18); the known-wrong control read rel +1.0307.

## Banked targets

    validate_param_card_sigma PASS exit=0 364s | test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.63s
    validate_alphas PASS exit=0 335s | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 26.41s
    validate_scales PASS exit=0 121s | test result: ok. 10 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 115.30s
    reweight_mg_oracle PASS exit=0 64s | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
    cli_reweight_proton PASS exit=0 108s | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 97.63s
    cli_generate_proton PASS exit=0 43s | test result: ok. 5 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 26.01s
    cli_integrate PASS exit=0 53s | test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.68s
    validate_lhef PASS exit=0 299s | test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 128.98s
    coupling_oracle PASS exit=0 7s | test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.30s
    amplitude_oracle PASS exit=0 13s | test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.75s
    ALL TARGETS DONE

`validate_alphas` passes 2/2 and `validate_scales` 10 passed (1 ignored) with the row declared. Without the declaration the inventory check stops them (not re-run without it).

## Collator

`cargo run --profile release-debug -p vibegraph-validation-report` builds and writes
`target/validation-report/report.{md,json}`, then exits 1: "the report does not
match the manifest", 332 lines, every one "declared banked/hermetic and no gate
wrote it". Only the targets above ran, so cells of other rows are absent. For
the two new rows the cells are recorded measurements (✅ pull +0.06, chi2/dof 2.00;
✅ pull +0.14, chi2/dof 1.02) and neither appears in the mismatch list. The
whole-layer report was not run.

## Gates

```
cargo fmt --all --check                                         exit 0
cargo clippy --workspace --all-targets -- -D warnings           exit 0
cargo clippy ... --features vibegraph/extended-validation       exit 0
cargo test --workspace --no-fail-fast                           49 binaries, 1735 passed, 0 failed, 19 ignored
python3 scripts/kb.py lint                                      0 errors, 4 warnings (blocked backlog items)
```

## Found

- **Whole-layer collation needs the whole layer.** The collator exits 1 after any
  partial run; closing it: a `--only <rows>` mode that checks the named rows'
  cells, or run it after the full `validate`.
- **`ee_to_tth_ymt250` has no diagram census or amplitude table.** Closing it:
  `extract-diagrams` and `generate-amplitude-tables` for the banked directory.
- **`pixi` absent on the cloud host** (as P-V1 found): each target ran as its cargo command.
