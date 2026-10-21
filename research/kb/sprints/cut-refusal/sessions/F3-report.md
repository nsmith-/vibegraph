---
type: Session Report
title: "F3 report: every CLI test file on tests/common"
description: "Per-file environment before/after, test lists, mutation evidence, gates and Found."
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-21}
---
## What changed

Tests under `vibegraph-cli/tests` only. `common` gains `vibegraph_ambient()` (the bare binary: inherited cwd and environment) and `stdout_of`. After the change
`grep -rn 'CARGO_BIN_EXE\|from_utf8_lossy' vibegraph-cli/tests` matches only `common/mod.rs`.
Every stderr/stdout read is `common::stderr_of` / `common::stdout_of`; `mod common;` was added to seven files that had none. `Command` imports left unused were removed.

## Environment per file (before / after)

"ambient" = inherited cwd and environment, nothing set or removed. Isolated = `common::vibegraph(cwd, home)`: `VIBEGRAPH_HOME=home`, `VIBEGRAPH_NO_NETWORK`/`_PDF_DIR`/`_UFO_DIR` removed, cwd set.
Every PDF/UFO path a test reads is passed by explicit flag (`--pdf-dir`, `--ufo-dir`), never by the ambient variable.

| file (binary) | before | after |
|---|---|---|
| `cli_decay` | 4 sites: ambient env; two set cwd to the tempdir, two inherit cwd | isolated, cwd = home = the test's tempdir |
| `cli_fixed_energy` | 4 sites, ambient, inherited cwd | isolated, cwd = home = tempdir |
| `cli_ufo_model` | 3 sites, ambient, inherited cwd (`--ufo-dir` explicit) | isolated, cwd = home = tempdir |
| `cli_onshell_veto` | ambient env, cwd = dir (shared `run` helper also used by the extended-validation rows, which pass `--pdf-dir`) | isolated, cwd = home = dir |
| `cli_pdf_cache` | hand-set `VIBEGRAPH_HOME`, `NO_NETWORK=value`, `PDF_DIR` removed, cwd | `vibegraph(cwd, home)` + `NO_NETWORK=value`; adds `UFO_DIR` removal (SM tests only) |
| `cli_generate` (6 sites) | ambient, inherited cwd | ambient (minimal edit: another open PR changes this file) |
| `cli_reference_docs` | ambient (`--help`) | ambient |
| `cli_integrate`, `cli_decay_chain`, `cli_decay_chain_events`, `cli_generate_proton`, `cli_reweight_proton`, `validate_samples_proton` (extended-validation) | ambient; `--pdf-dir` explicit | ambient: their data is absent here, so isolation could not be run; see Found |
| `cli_first_run`, `cli_hard_errors`, `cli_pdf_cache_fetched` | already on `common` | unchanged (stderr reads moved to `stderr_of` in `cli_pdf_cache_fetched`) |

## Test lists

`cargo test -p vibegraph --tests -- --list | sort`, before and after: `diff` empty for default features (172 lines). With `--features extended-validation` (248 / 247 lines) the only difference is the cargo `Compiling`/`Finished` timing lines, no test line.

## Mutations (each reverted, file touched)

| mutation of the helper | command | result |
|---|---|---|
| `vibegraph()` stops removing `VIBEGRAPH_NO_NETWORK` | `VIBEGRAPH_NO_NETWORK=1 cargo test -p vibegraph --test cli_first_run` | unmutated: 5 passed; mutated: `an_unattended_run_refuses_the_download_and_names_the_consent_flag FAILED` |
| `vibegraph()` ignores its `home` | `cargo test -p vibegraph --test cli_pdf_cache --test cli_first_run` | `the_refusal_names_the_directory_to_install_into FAILED` |
| `vibegraph_ambient()` adds a bogus flag | `cargo test -p vibegraph --test cli_generate --test cli_reference_docs` | `a_card_that_did_not_train_the_grid_is_refused`, `a_pre_draw_artifact_*`, ... FAILED |

Not reached by any mutation: the `VIBEGRAPH_UFO_DIR` removal (the variable is read for non-SM models only; a bogus ambient value left `cli_fixed_energy` 5 passed with the removal deleted) and `VIBEGRAPH_HOME` in the migrated fixed-energy/UFO files (`cli_fixed_energy`, `cli_ufo_model` stayed green with `home` bogus: their tests never touch the cache root). For those files the isolation is protective, not observed.

## Gate

```
cargo fmt --all --check                                   exit 0
cargo clippy --workspace --all-targets -- -D warnings     Finished `dev` profile
cargo clippy --workspace --all-targets --all-features -- -D warnings   Finished `dev` profile
cargo test --workspace                                    40 result lines; passed 1549 failed 0 ignored 16 (unchanged; no test added or deleted)
cargo test -p vibegraph --tests                           cli_decay 3, cli_first_run 5, cli_fixed_energy 5, cli_generate 10, cli_hard_errors 14, cli_onshell_veto 2, cli_pdf_cache 3+1 ignored, cli_reference_docs 1, cli_ufo_model 2 passed
cargo check -p vibegraph --tests --features extended-validation   Finished, no warnings
python3 scripts/kb.py lint                                833 concepts, 290 backlog items; 0 errors, 4 warnings (pre-existing)
```
No banked MG gate: tests only. `cargo doc` not run: no non-test source changed.

## Found

1. **The six extended-validation CLI test binaries stay on the ambient environment** (`cli_integrate`, `cli_decay_chain`, `cli_decay_chain_events`, `cli_generate_proton`, `cli_reweight_proton`, `validate_samples_proton`): their data is not in this worktree, so a switch to `common::vibegraph` could only be compile-checked. Close: with the data present, move them to `common::vibegraph(dir, home)` and run them.
2. **`cli_generate` still ambient** (6 sites); same switch once the other PR touching it has landed.
