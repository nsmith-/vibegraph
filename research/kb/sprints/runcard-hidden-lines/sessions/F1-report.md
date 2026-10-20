---
type: Session Report
title: "F1 report: hidden_parameter lines read, systematics request warns"
description: "split_line cuts comments at the first '#' or '!'; a card requesting systematics weights warns that none are written."
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-20}
sources:
  - {id: commit, resource: "da9beaa", title: "F1 commit on sprintd-runcard-hidden-lines"}
---
## Implemented

1. `split_line` (`vibegraph-lib/src/runcard.rs`) now cuts the comment at the
   first `#` or `!`. `banner.py` is not in this container (submodule absent), so
   the rule is MadGraph's `RunCard.read` as I know it (`split('#')[0]`, then
   `split('!')[0]`, then split at the last `=`), not re-read from the pinned
   source. Consequence: a value cannot contain `#` or `!`. The first-`=` split
   is untouched (`runcard-value-with-equals-dropped` stays open).
2. `classes::warn_unwritten_systematics`, called from `RunCard::from_values`
   beside `refuse_ignored_physics`: with `use_syst` true and any of
   `systematics_program`, `systematics_arguments`, `sys_scalefact`,
   `sys_alpsfact`, `sys_matchscale`, `sys_pdf`, `sys_scalecorrelation` off its
   default, one `tracing::warn!` names the fields and says the `<rwgt>`
   variation weights are not written. Same path as the `alpsfact` and `xqcut`
   warnings in `matching.rs`. Classification stays `IgnoredBenign`; the
   `B_SYST` rationale and `field-classification.md` now say the request warns.
3. Both backlog items deleted; dangling links in the feature item and the CMS
   survey rewritten as plain text.

Brief note: the brief's example card `--mur=...` cannot be used in a test of
`systematics_arguments`, because its value contains `=` and the first-`=` split
drops the line (the other open item); the test uses `['--mur']`.

## Fixtures whose reading changed

None. Every tracked run card with `#` after content has it after a `!` comment
(`! PDF type for beam #1`, the `systematics_arguments` URL comment), which was
already cut. No tracked card carries `# hidden_parameter`. Checked with
`git grep -n "=.*#"` over `validation/madgraph/*run_card.dat`,
`research/upstream/*.dat`, `run_card.dat`, `vibegraph-lib/tests/data`. The CMS
cards themselves are not in the repo; they will now hit refusals for hidden
unimplemented cuts, as intended.

## Tests and mutation evidence

New: `runcard::tests::hidden_parameter_comment_is_stripped` (Float `mmll`, Bool
`pdfwgt`, consumed `ebeam1`; whole-card equality with the uncommented card),
`coupling::scales::tests::hidden_pdfwgt_line_reaches_the_scale_setup`,
`runcard::classes::tests::a_systematics_request_warns_once_and_a_default_card_does_not`,
`...::systematics_fields_do_not_warn_when_use_syst_is_off`.

Mutation (split_line reverted to `!`-only with `#`-at-start skip, warning call
removed), `cargo test -p vibegraph-lib --lib -- hidden systematics`:

    test runcard::tests::hidden_parameter_comment_is_stripped ... FAILED
    test coupling::scales::tests::hidden_pdfwgt_line_reaches_the_scale_setup ... FAILED
    test runcard::classes::tests::a_systematics_request_warns_once_and_a_default_card_does_not ... FAILED
    test result: FAILED. 2 passed; 3 failed

(the two passing are the no-warning-when-off test and an unrelated name match.)
Restored; the same command passes 3/3 hidden-filter and 36/36 `runcard`.
The use_syst-off test cannot fail under the mutation (it asserts absence); its
blind spot is a warning that ignores `use_syst`, which it does catch only if the
fix is wrong that way.

## Gates (log `scratchpad/runcardA-F1-gate.log`)

    cargo fmt --all --check                                  -> fmt=0
    cargo clippy --workspace --all-targets -- -D warnings    -> clippy=0
    cargo test --workspace                                   -> test=0 (40 "test result: ok" lines, 0 FAILED)
    python3 scripts/kb.py lint                               -> 0 errors, 4 warnings (pre-existing blocked-without-blocked_by)

Not run: `validate` (no amplitude code touched; pixi absent).

## Found

- The warning test in `classes.rs` needs `tracing::callsite::rebuild_interest_cache()`
  inside the subscriber scope, as `budget.rs` does; a shared test capture
  helper does not exist (three private copies: `progress.rs`, `budget.rs`,
  `classes.rs`). A `#[cfg(test)]` helper module would close it.
- The `# hidden_parameter` strip is unverified against the pinned `banner.py`
  in this container (submodule absent); someone with the submodule should
  confirm the `#`/`!`/last-`=` order.
