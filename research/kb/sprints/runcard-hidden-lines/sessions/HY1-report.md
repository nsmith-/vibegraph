---
type: Session Report
title: "HY1 report: hygiene review of the hidden-line parser fix and the systematics warning"
description: "One shared tracing test capture, the systematics field list derived from the classification table, a per-field warning test, split_line's doc in the code's own terms."
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-20}
sources:
  - {id: commit, resource: "ba65b18", title: "HY1 fix commit on sprintd-runcard-hidden-lines"}
---
## Findings (written before any edit)

| id | pt | site | evidence | disposition |
|---|---|---|---|---|
| H1 | 4 | `progress.rs`, `budget.rs`, `runcard/classes.rs` tests | three private `impl Subscriber` copies; `grep -rn rebuild_interest_cache vibegraph-lib/src` finds it in two of them only, `progress.rs` has none | fixed (`ba65b18`) |
| H2 | 1 | `classes.rs` `SYSTEMATICS_REQUEST_FIELDS` | seven names repeated by hand beside the `B_SYST` rows of `FIELD_CLASSES` (`:373-379`); a new `B_SYST` field would not warn | fixed |
| H3 | 2 | `classes.rs` tests | only 4 of 7 fields warn-tested (`sys_alpsfact`, `sys_matchscale`, `sys_scalecorrelation` untested); mutation: drop `sys_alpsfact` from the list passes | fixed |
| H4 | 2 | `systematics_fields_do_not_warn_when_use_syst_is_off` | F1 says it cannot fail under the gate mutation; mutation `if false` in place of the `use_syst` gate | not reproduced as a gap: the test fails (M1); strengthened anyway with a positive control (same card, `use_syst` on, warns once) so it cannot pass because the field never warns |
| H5 | 1 | `runcard.rs` `split_line` doc | says `banner.py` "cuts at both" and "the single `=`"; the submodule is absent here so the claim is unverified, and the code splits at the first `=` | fixed: doc states the code's rule only |
| H6 | 3 | `warn_unwritten_systematics` | `pub(super)`, one caller (`RunCard::from_values`); `SYSTEMATICS_REQUEST_FIELDS` private | leave (already minimal) |
| H7 | 1 | warning text | names the fields, says `<rwgt>` scale/PDF/alpha_s weights are not written and that the cross section and events are unchanged | leave (names what is lost) |
| H8 | 1 | comment grep `\bT[0-9]\b\|used to\|no longer` over the diff | no hits in code | leave |
| H9 | 2 | `hidden_pdfwgt_line_reaches_the_scale_setup` (`scales.rs`) | same mutation class as the runcard test, but it pins the downstream consumer | leave |
| H10 | 1 | `IgnoredBenign(#[cfg_attr(not(test), allow(dead_code))] ..)` | the string is now read in production by `systematics_request_fields` | fixed: stale allow removed |

## Fixed (`ba65b18`)

- H1: `vibegraph-lib/src/test_capture.rs` (`#[cfg(test)] mod`, crate-private): `Captured`, `capture`, and `capture_primed(prime, run)`. The cache trap is handled in one place. Net -145 test lines across progress/budget/classes.
  - First attempt dropped budget's primer (hit the callsite, then rebuild). The full `cargo test --workspace` then failed `the_progress_total_projects_the_convergence_target` ("one event per iteration" left 0, right 6) although the filtered run passed. So a rebuild before the first hit is not enough under load; the helper keeps the primer as an argument. The classes test, which never had a primer, is unchanged in that respect (same exposure as before; it passed in both full runs).
- H2/H3/H10: `systematics_request_fields()` filters `FIELD_CLASSES` for `IgnoredBenign(reason)` with `reason == B_SYST`. New test `every_systematics_field_warns_on_its_own` asserts the derived list equals the seven expected names (both ways) and that each name, moved off its default alone, produces exactly one warning naming it.
- H4: positive control added to the use_syst-off test.
- H5: `split_line` doc.

## Mutations (log `scratchpad/runcardA-HY1-mut.log`, `-mut2.log`; `cargo test -p vibegraph-lib --lib -- systematics`)

| mutation | new tests |
|---|---|
| M1 `use_syst` gate -> `if false` | `systematics_fields_do_not_warn_when_use_syst_is_off` FAILED (other two pass) |
| M2 derivation excludes `sys_alpsfact` | `every_systematics_field_warns_on_its_own` FAILED (2 pass); the old four-field test passes this mutation |
| M3 `*current != default` -> `true` | `a_systematics_request_warns_once_and_a_default_card_does_not` FAILED (the first draft of this mutation matched two sites and never applied; the log's first M3 run is void) |
| M4 call to `warn_unwritten_systematics` removed | all 3 FAILED |
| M5 `split_line` cut at `!` only | `hidden_parameter_comment_is_stripped` FAILED |

Blind spots: the capture helper has no test of the interest-cache heal itself (a flake, not a deterministic mutation); the `classes.rs` capture is unprimed, so under extreme load it can miss the warning and fail rather than pass vacuously (fails loud, never silent).

## Gate

    cargo fmt --all --check                                              -> 0
    cargo clippy --workspace --all-targets -- -D warnings                -> clippy1=0
    cargo clippy --workspace --all-targets --all-features -- -D warnings -> clippy2=0
    cargo test --workspace                                               -> test=0 (lib: 1014 passed, 0 failed, 10 ignored)
    cargo doc -p vibegraph-lib --no-deps --document-private-items        -> doc=0, 8 warnings, none in touched files
    python3 scripts/kb.py lint                                           -> 824 concepts, 288 backlog items; 0 errors, 4 warnings

Suite count: +1 test (`every_systematics_field_warns_on_its_own`); none deleted. No banked target run: no amplitude, colour, coupling-value or sampling code changed. Log: `scratchpad/runcardA-HY1-gate2.log`.

## Method

Diff read against the checklist (about 40%), mutation batch driver (30%), workspace grep for duplicates (15%), backlog path grep (5%: only `runcard-value-with-equals-dropped` names `split_line`, still open and still accurate). The lead "use_syst-off test is vacuous" did not hold: the original also fails under the gate mutation. The lead "a rebuild alone replaces the primer" did not hold (above).

## Found

- The `# hidden_parameter` strip and the `#`/`!`/last-`=` order are unverified against the pinned `banner.py` (submodule absent in this container); needs someone with `research/refs/mg5amcnlo`. Closes with a read of `banner.py:2900-2902`.
- The `classes.rs` warning test has no primer for its `warn!` callsite; if a flake appears, `capture_primed` with a parse of a systematics card as the primer closes it.
