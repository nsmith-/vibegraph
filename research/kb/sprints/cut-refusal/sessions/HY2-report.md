---
type: Session Report
title: "HY2 report: hygiene review of F2"
description: "Findings table (written before any edit), dispositions, mutation evidence and gates."
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-21}
---
## Findings (written before any edit)

| id | pt | site | evidence | conf. | disposition |
|---|---|---|---|---|---|
| H1 | 3 | `tests/common/mod.rs:4` module-wide `#![allow(dead_code)]` | `fixed_beams` is used by all five users, `vibegraph` only by `cli_hard_errors`, `vibegraph_behind_dead_proxy` only by `cli_first_run`, `stderr_of` by two; `DEAD_PROXY` is used only by `vibegraph_behind_dead_proxy` and is `pub` for no reader. A module-wide allow hides a helper unused by every binary | checked | fix: per-item allow on the helpers some binary leaves unused; `DEAD_PROXY` inlined |
| H2 | 1 | `common/mod.rs` doc of `vibegraph_behind_dead_proxy` | says "every HTTP route through DEAD_PROXY"; it sets `ALL_PROXY` and removes `NO_PROXY`/`no_proxy` only | checked | fix: doc states what it sets |
| H3 | 1 | `generate.rs:782` run-card load | no comment says why the card precedes the artifact (brief asks for one in the code's own terms); no stale comment elsewhere describes the old order (`grep -rniE "(run card\|artifact) (first\|before)"` over `vibegraph-cli docs README.md`: only the new test's) | checked | fix: comment added |
| H4 | 4 | `validation_dir()` copied in `cli_first_run`, `cli_integrate`, `cli_pdf_cache`, `cli_pdf_cache_fetched`, `validate_samples_proton` | five identical bodies `Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph")` | checked | fix: `common::validation_dir` |
| H5 | 4 | `integrate_from_bare_cwd` in `cli_pdf_cache.rs:29` and `cli_pdf_cache_fetched.rs:78` | byte-identical 20-line bodies; both also build the Command with `VIBEGRAPH_HOME` set and `VIBEGRAPH_PDF_DIR` removed by hand, the part `common::vibegraph` does | checked | fix: one `common::integrate_from_bare_cwd` on `common::vibegraph` |
| H6 | 2 | `cli_generate_refuses_the_run_card_before_reading_the_artifact` | must catch (a) the order reversed and (b) the cut refusal not applied; also the control arm catches the artifact not being read | suspected | settled by mutation (below): both caught |
| H7 | 4 | `cli_hard_errors.rs` `cut_beams()` | a one-line wrapper around `fixed_beams(45.6, 45.6)` | checked | leave: three call sites share the name and the card's meaning |
| H8 | 4 | `String::from_utf8_lossy(&output.stderr)` in 12 other test files | `stderr_of` exists in `common` but those files (`cli_decay*`, `cli_generate*`, `cli_pdf_cache*`, ...) inline the conversion, 35 sites | checked | Found (a mechanical sweep across 12 files; behavior-neutral, not this PR's change) |

## Fixed

All in one commit (H1-H5): `vibegraph-cli/src/generate.rs` (comment only),
`vibegraph-cli/tests/common/mod.rs`, and the five test files that lost their
copies (`mod common;` added to the four that had none).

* `common`: `#[allow(dead_code)]` on `vibegraph`, `vibegraph_behind_dead_proxy`,
  `stderr_of`, `fixed_beams`, `validation_dir`, `integrate_from_bare_cwd`. With the
  module-wide allow gone, `cargo check --tests` reported `fixed_beams` unused in the
  PDF-cache binaries: the per-item list is the exact set of items some binary
  leaves unused, and a new item without an allow is reported by the first binary
  that does not call it. `DEAD_PROXY` is no longer `pub`; it is the literal in the
  one function that uses it.
* `common::validation_dir` and `common::integrate_from_bare_cwd` replace seven copies.
  The command built by the shared fn also removes `VIBEGRAPH_UFO_DIR` (the old
  copies did not); no test in those binaries sets a UFO model.

## Mutations

Ordering test, `cargo test -p vibegraph --test cli_hard_errors cli_generate_refuses_the_run_card_before`:

| mutation | result |
|---|---|
| none | `1 passed` |
| M1: `generate.rs` reads the artifact before `load_run_card` | `panicked at cli_hard_errors.rs:624:5`, `FAILED. 0 passed; 1 failed` |
| M2: `detect_unimplemented(&rc)` removed from `load_run_card` (the card is not refused) | `panicked at cli_hard_errors.rs:624:5`, `FAILED. 0 passed; 1 failed` |

Both fail on the `'ptj1min' (= 30` assertion. The test is unchanged by this
session; the mutations settle the brief's question for it.

Shared helpers, `cargo test -p vibegraph --test cli_pdf_cache --test cli_first_run`:

| mutation | result |
|---|---|
| none | `cli_pdf_cache 3 passed 1 ignored; cli_first_run 5 passed` |
| M4: `integrate_from_bare_cwd` stops setting `VIBEGRAPH_NO_NETWORK=1` | `a_missing_pinned_set_reports_what_it_would_download ... FAILED` |
| M5: `validation_dir` points at a missing directory | `cli_first_run: 3 failed` (`an_unattended_run_refuses...`, `the_refusal_names_the_directory...`, `the_kill_switch_outranks_explicit_consent`) |

(`VIBEGRAPH_NO_NETWORK` set to `0` still refuses, as the `cli_pdf_cache`
test of that variable pins, so the variable's value is not a mutation the helper's
users can see.) Not reached: `cli_integrate`, `cli_pdf_cache_fetched`,
`validate_samples_proton` need the extended-validation data and ran as
compile-checks only (below).

## Routed

None.

## Found

1. **`String::from_utf8_lossy(&output.stderr)` is inlined at 35 sites in 12 files in the CLI
   tests** (`grep -c` over `vibegraph-cli/tests/*.rs`; `cli_decay.rs`,
   `cli_generate.rs`, `cli_generate_proton.rs`, `cli_pdf_cache*.rs`, ... ) beside
   `common::stderr_of`. Close: replace with `common::stderr_of`, behavior-neutral.
2. **Raw `Command::new(env!("CARGO_BIN_EXE_vibegraph"))` in 13 CLI test files**
   (`grep -c CARGO_BIN_EXE_vibegraph`), most without `VIBEGRAPH_HOME` or the
   `VIBEGRAPH_PDF_DIR`/`VIBEGRAPH_NO_NETWORK` removal that `common::vibegraph` gives;
   an ambient `VIBEGRAPH_PDF_DIR` in a developer's shell changes what they read.
   Close: move them onto `common::vibegraph` once their environments are checked
   one by one.

## Brief corrections

None; the brief's "any CLI test file F2 missed" was right: `validation_dir` and
`integrate_from_bare_cwd` were the misses.

## Gate

```
cargo fmt --all --check                                   exit 0
cargo clippy --workspace --all-targets -- -D warnings     Finished `dev` profile
cargo clippy --workspace --all-targets --all-features -- -D warnings   Finished `dev` profile
cargo test --workspace                                    40 result lines; passed 1549 failed 0 ignored 16 (F2: 1549 passed, 16 ignored; no test added or deleted)
cargo check -p vibegraph --tests --features extended-validation   Finished, no warnings
cargo doc -p vibegraph --no-deps --document-private-items 1 warning, assets.rs:5 (untouched file, pre-existing)
python3 scripts/kb.py lint                                831 concepts, 290 backlog items; 0 errors, 4 warnings (pre-existing)
```

No banked MG gate: no amplitude, colour, coupling, phase-space or sampling code
changed. `cli_integrate`, `cli_pdf_cache_fetched` and `validate_samples_proton`
(extended-validation, need fetched data) are compile-checked only.

## Method

Reading the diff and grepping the helper names (`validation_dir`,
`CARGO_BIN_EXE_vibegraph`, `from_utf8_lossy`) found H4, H5, H8 and the Found
items; removing the module allow and letting the compiler report which binary
leaves which helper unused settled H1; a two-mutation script settled the ordering
test.

## Commit

`test(cli): per-item dead-code allows; one validation_dir and integrate_from_bare_cwd in tests/common`
