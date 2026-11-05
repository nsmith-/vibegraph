---
type: Session Report
title: "G-HY4 report: σ and its error decimal-aligned in one SI-prefixed barn unit"
description: "integrate prints a measurement under the one SI prefix its value picks, the error to two significant figures and the value to the same decimal place (8.6812 ± 0.0023 ab; a width 200.1 ± 1.2 eV), through si::fmt_si_measured beside fmt_si; value_and_error is gone. The survey parser reads every barn prefix and no longer reads an unknown part unit as pb. 17 of 17 mutations killed; every gate green."
status: draft
tags: [cli, output, si-prefix, cms-survey, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
measured:
  host: "cloud container, linux-64, 2026-11-04"
  commit: "change 0ed9564 + a61bdc0 + aeb8339 on base db7c3b6"
  command: "see Gate; logs under the session scratchpad, prefixed G-HY4-"
---
# G-HY4 report

Branch `sprintg-cli-output-hy4`, base db7c3b6. Commits: `0ed9564` (Rust and
docs), `a61bdc0` (survey parser), `aeb8339` (the part-line reader's match and
the M6 case), and this report with the closeout claim.

## Findings (written before any edit)

| id | pt | site | evidence | conf. | disposition |
|---|---|---|---|---|---|
| H1 | 1 | `vibegraph-cli/src/integrate.rs:665` `value_and_error` | prints `8.6812e-6 ± 2.3e-9` (its own test, `:1327`); the review wants one SI barn prefix, decimal-aligned | checked | **fixed** |
| H2 | 4 | `integrate.rs:665` beside `si.rs:93` `fmt_si` | a second number formatter in the crate, with its own exponent parsing and carry logic | checked | **fixed** (`value_and_error` replaced by a thin user of `si.rs`) |
| H3 | 2 | `validation/cms_gridpacks/parse.py:535`, `:601` | `UNIT_TO_PB` has no ab/zb/µb/b (and an `ub` key the CLI never prints: it writes U+00B5); `VG_MULT` takes `UNIT_TO_PB.get(unit, 1.0)`, so an `ab` part line would be stored as pb, 1e6 too large | checked (P4 below: the base tests pass it) | **fixed** |
| H4 | 2 | `tests/cli_onshell_veto.rs:138` `parse_result`, `tests/cli_decay.rs:185` `parse_width` | read `fields[2]`, `fields[4]` with no unit; a prefixed line would be misread by a factor of 1000ⁿ (veto) or refused on the `GeV` assert (decay) | checked | **fixed** (one unit-aware reader in `tests/common`) |
| H5 | 1 | `si.rs:16` `PREFIXES` | starts at f: a sub-fb value prints a mantissa below 1 | checked | **fixed** (a, z added) |
| H6 | 4 | `tui/mod.rs:296` `summary`, `tui/footer.rs:108` `cross_section` | the exit summary on stderr prints σ by the footer's rule (one error figure, `2.0247 ± 0.0004 nb`) next to the result line's two-figure rule | checked | **Found** (TUI behaviour, not in the brief) |
| H7 | 1 | `docs/src/cli/overview.md:188`, `docs/src/guide/07-phase-space.md:511` | "`σ = … pb`", "reports the width in GeV (`Γ = … GeV`)" | checked | **fixed** |

Backlog check (path tails `/si.rs`, `cli/src/integrate.rs`,
`cms_gridpacks/parse.py`, `tui/footer.rs`, `tui/mod.rs`, `cli_decay.rs`,
`cli_onshell_veto.rs`, `cli_fixed_energy.rs`): the hits
(`multiplicity-sum-construction-copied`, `cli-tests-define-pdf-dir-per-file`,
`integrate-cannot-select-a-sampler` and three performance items) have no site
this change makes wrong. `flat-weights-print-rounding-noise` loses one
symptom: its `± 1.5e-154` would now print as zeros at the value's 15th digit (see
Found).

## The format and its edge cases

`si::fmt_si_measured(value, error, unit)`, values in the base unit as for
`fmt_si`. The prefix is chosen by `prefix_exponent`, now shared by both
functions; `integrate::measurement` maps a cross section in pb to barns and a
width in GeV to electronvolts. The result line is `{symbol} = {measurement}`;
the part line `… : σ = {measurement} over N channels`.

| case | rule | example |
|---|---|---|
| ordinary | error to 2 figures; value to the same decimal place, so both have the same decimals; prefix from the value | 9e-6 pb → `8.6812 ± 0.0023 ab`; 1.234567 ± 0.000123 pb → `1.23457 ± 0.00012 pb` |
| lower edge | 0.9994 ± 0.0012 pb is below 1 pb | `999.4 ± 1.2 fb`; 1.0004 → `1.0004 ± 0.0012 pb` |
| upper edge / carry | a value that rounds to 1000 at its error's place takes the next prefix | 999.4 ± 1.2 pb → `999.4 ± 1.2 pb`; 999.996 ± 0.12 pb → `1.00000 ± 0.00012 nb` |
| error > σ | where the error's second figure is above the units, both print with no decimals: σ to its units digit (never rounded to 0 under the prefix it chose), the error to its two figures | 3.4e-5 ± 4.37e-4 pb → `34 ± 440 ab`; 3.4e-6 → `3 ± 440 ab` |
| error = 0 | the error bounds no digit: σ to 15 significant digits (`DBL_DIG`), the error at the same decimals | 0.125 pb → `125.000000000000 ± 0.000000000000 fb` |
| error not finite | as error = 0, the error printed as Rust prints it | `125.000000000000 ± NaN fb`, `… ± inf fb` |
| tiny error | the value is never given more than 15 digits | 1.25 ± 1e-30 → `1.25000000000000 ± 0.00000000000000` |
| σ = 0 | refused upstream for the total (#88), but a part line can carry one; the prefix comes from the error | 0 ± 2.3e-9 pb → `0.0 ± 2.3 zb`; −0 prints as 0 |
| σ = 0, error 0; σ not finite | nothing chooses a prefix: both as Rust prints them, in the base unit | `0 ± 0 b`, `NaN ± 1 b` |
| width | the same rule under electronvolts | 2.49523 ± 0.00123 GeV → `2.4952 ± 0.0012 GeV`; 2e-7 GeV → `200.1 ± 1.2 eV` |

The width in eV is a choice the brief did not make: it names only barn units,
but the result line it covers also prints Γ, and a GeV-fixed width at the
error's decimal place would print `0.0000002001 ± 0.0000000012 GeV`. The
manager may prefer to keep Γ in GeV; the guide page (`07-phase-space.md`)
states the current choice.

A part line, from the banked mixed-multiplicity run (`p p > e+ e- @0` +
`… j @1`), observed only under mutation M15 (error ×10):
`2 outgoing legs (@0; masses 0, 0 GeV): σ = 668 ± 24 pb over 1 channels`;
unmutated, the rule gives `667.9 ± 2.4 pb` for the artifact's 667.858 ±
2.407 pb (computed, and passed by the banked check, not printed here).

Sample lines from the built binary (`--fixed-budget --neval 4000 --niter 4`):

```
σ = 10.431 ± 0.032 ab        e+ e- > mu+ mu-, √ŝ = 100 TeV
σ = 2.0335 ± 0.0059 nb       e+ e- > mu+ mu-, √ŝ = 91.2 GeV
Γ = 5.390778958 ± 0.000000039 MeV   h > b b~ (constant integrand, error at its variance's rounding)
Γ = 164.03 ± 0.54 MeV        t > b e+ ve
```

The TUI footer's fixed-width tests hold unchanged (its picobarn constant now
comes from `si::PICOBARN`; `fmt_si`'s own outputs move only below 1 fb, where
they now take a or z).

## Survey parser

`UNIT_TO_PB` reads zb, ab, fb, pb, nb, µb (U+00B5, and U+03BC defensively, and
the old `ub`), mb and b. A unit outside the table leaves σ unread on the
result line (as before) and now skips the part line instead of storing it as
pb. Both old forms still parse: the base `test_parse.py` fixtures (the `{:.6}`
pb form, `VG_OK`, and G-HY3's `e`-notation form, `VG_SIG_FIGS`) pass against
the new parser.

## Tests (suite count +7 hermetic, +2 Python; none deleted)

Hermetic, added in `si.rs`: `a_measurement_is_printed_to_its_errors_second_figure`,
`the_values_magnitude_picks_the_prefix_at_both_edges`,
`an_error_larger_than_the_value_prints_as_integers`,
`an_error_that_bounds_nothing_leaves_the_full_precision`,
`a_zero_value_takes_the_errors_prefix`,
`a_value_keeps_no_more_digits_than_an_f64_holds`,
`a_cross_section_below_a_femtobarn_lands_on_ab`. Replaced in place (count
unchanged): `integrate::tests::a_value_keeps_the_digits_its_error_supports` →
`a_measurement_is_printed_in_its_observables_prefixed_unit`;
`cli_fixed_energy::a_small_cross_section_prints_the_digits_its_error_supports`
now pins the unit `ab`, a mantissa in [1, 1000), two error figures, equal
decimals, the error within 5% and σ within err/20 of the artifact's (the
rounding bound of two error figures). Extended: the banked
`cli_generate_proton::a_mixed_multiplicity_card_is_integrated_and_sampled_as_a_sum`
reads both part lines and checks equal decimals, the error within 5% and σ
within err/20 of the artifact's per-part sums. `cli_decay` and
`cli_onshell_veto` read the line through `common::result_line`. Python:
`VgOutput.test_si_prefixed_units`, `test_an_unknown_unit_is_not_read_as_pb`.

## Mutations

Driver `G-HY4-mutate.py` (apply, run, revert; log `G-HY4-mutations.log`). Each
old string is checked to occur once before the batch starts. "Killed" means
`test result: FAILED` (or a `FAIL:` from unittest), not a build error. The
tree's `git diff --stat` was the same before and after.

| id | mutation | killed by |
|---|---|---|
| M1 | σ printed one decimal past the error (alignment) | `si::a_measurement…`, `integrate::a_measurement…`, `cli_fixed_energy a_small_cross_section` (`:413`, decimals) |
| M2 | error to three figures | `si::a_measurement…`, `an_error_larger…`, `cli_fixed_energy` (`:409`, figures) |
| M3 | prefix exponent `exp10 / 3` (truncates toward zero) | `si::a_measurement…`, `integrate::…`, `fmt_si` tests |
| M4 | carry at `> 1000` instead of `>= 1000` (upper edge) | `the_values_magnitude_picks_the_prefix_at_both_edges`, `an_error_that_bounds_nothing…` |
| M5 | no carry into the next prefix | same two |
| M6 | prefix from the error's magnitude, not σ's | `a_value_keeps_no_more_digits…`; after the added case, also `an_error_larger…` |
| M7 | bracket shifted a decade (`(exp10 + 1).div_euclid(3)`, lower edge) | `si::a_measurement…`, `integrate::…`, `fmt_si` sweep |
| M8 | at least one decimal | `an_error_larger…`, `a_measurement…` |
| M9 | unbounded value at six figures | `an_error_that_bounds_nothing…` |
| M10 | no 15-digit cap | `a_value_keeps_no_more_digits…` |
| M11 | a zero value prefixes from itself | `a_zero_value_takes_the_errors_prefix` |
| M12 | a width under a barn prefix | `integrate::a_measurement…` |
| M13 | table without atto | `si::…`, `integrate::…`, `cli_fixed_energy` (`:397`, unit `ab`) |
| M14 | result line keeps a trailing `pb` | `cli_fixed_energy` (shape panic in `common::result_line`) |
| M15 | part line's error ×10 (banked, run alone) | `cli_generate_proton a_mixed_multiplicity…` (`:1124`: `σ = 668 ± 24 pb … against the artifact's 667.86 ± 2.41 pb`) |
| P3 | parser without `ab` | `test_si_prefixed_units`; the base suite (22 tests) passes it |
| P4 | parser reads an unknown part unit as pb (the base code) | `test_an_unknown_unit_is_not_read_as_pb`; the base suite passes it |

M6 survived every case but the 15-digit cap in the first batch: with the
error below the value, a prefix taken from the error is too small, and the
carry loop climbs it back to the value's, so the output is the same. It
differs only where the error sits a prefix *above* the value; that case
(`3.0e-6 ± 4.4e-3` pb → `3 ± 4400 ab`) was added to
`an_error_larger_than_the_value_prints_as_integers`, and the rerun kills M6
there too (`G-HY4-m6.log`). M6 and M15 ran under `--profile release-debug`
after the batch, on the tree with that case added.

## Gate (final tree, `aeb8339`; the debug steps rerun on it after the banked fix)

- `cargo fmt --all --check` → exit 0
- `cargo clippy --workspace --all-targets --jobs 3 -- -D warnings` → exit 0
- `cargo clippy --workspace --all-targets --features vibegraph/extended-validation --jobs 3 -- -D warnings` → exit 0
- `cargo clippy --workspace --all-targets --all-features --jobs 3 -- -D warnings` → exit 0
- `cargo test --workspace --jobs 3` → exit 0; 54 result lines, 1904 passed,
  0 failed, 25 ignored (G-HY3's 1897 + the 7 `si.rs` tests above)
- `cargo doc -p vibegraph --no-deps --document-private-items` → 0 warnings
  (only `vibegraph-cli` source changed; no lib doc run needed)
- `cargo test -p vibegraph --profile release-debug --features extended-validation --test cli_integrate`
  → `test result: ok. 6 passed; 0 failed` (parses the σ line; run on `0ed9564`,
  whose production code the later test-only edits leave unchanged)
- `cargo test -p vibegraph --profile release-debug --features extended-validation --test cli_generate_proton`
  → `test result: ok. 11 passed; 0 failed; 2 ignored` (reads the σ line and,
  now, both part lines). Its first run failed in my new check:
  `part_line` matched the survey's `2 outgoing legs (masses …)` line from
  `multiplicity.rs:569` before the summary's; it now requires `(@`.
- `python3 -m unittest discover -s validation/cms_gridpacks -p 'test_*.py'` → `Ran 24 tests … OK`
- `python3 scripts/kb.py lint` → `1106 concepts, 348 backlog items; 0 errors, 5 warnings` (after `python3 scripts/kb.py index` added this report to `sessions/index.md`)

Not run: the banked tiers whose sources this change does not touch.

## Method

Reading the G-HY3 function, `si.rs` and every reader of the result line
(grep of `σ = `, `Γ = `, ` pb"`, `strip_prefix`; ~35%), which found H3 and H4;
hand-computed expectations before the unit tests (~15%; two were wrong — I
had 4e-7 pb as 0.4 fb and 0 ± 2.3e-21 b as ab, both zb — fixed before the
first run); the mutation batch (~30%); running the binary for sample lines
(~5%). One lead did not hold: a first sample run printed `σ = 10.431 ±
0.032 b`, which was the binary the mutation batch had left built under M13
(no atto symbol), not the committed code; the rebuilt binary prints `ab`.
A constant-integrand error is not always zero (`flat-weights-print-rounding-noise`,
`h > b b~` above): the first draft printed an unbounded value at six figures,
which would have read the decay rows back outside `cli_decay`'s 1e-6 gate had
an error been exactly zero, so the unbounded case and the cap became the 15
digits an `f64` round-trips.

## Found

1. **The TUI exit summary prints σ by another rule.** `tui/mod.rs:296`
   `summary` and the footer use `footer::cross_section` (`footer.rs:108`):
   significant figures from σ/err, one figure of error (`2.0247 ± 0.0004 nb`),
   printed on stderr beside the result line's `2.02473 ± 0.00036 nb`. The
   footer's live cell needs a fixed width; the one-shot summary line does not.
   Close: the summary uses `fmt_si_measured`, the footer keeps its own.
2. **`check-events` prints the declared σ at `{:.6} pb`.** `check.rs:299`,
   `:305`: `XSECUP` and mean `XWGTUP` print `0.000009 pb` for the small
   cross section, the defect G-HY3 fixed in `integrate`. Close: the same
   formatter (with the `XERRUP` as its error).
3. **`flat-weights-print-rounding-noise` symptom moved.** Its `± 1.5e-154`
   (f64::MIN_POSITIVE through `combine_kept`) would now print as zeros at the
   value's 15th digit (the cap case, `a_value_keeps_no_more_digits_than_an_f64_holds`;
   inferred from the format, not reproduced on that artifact), so the result
   line no longer shows it; the cause and the overweight symptom stand. The
   item's text should say so.
4. **Γ's unit is a choice to confirm.** The width line moved from GeV to an
   SI-prefixed electronvolt unit (above); a reader that parses `Γ = … GeV`
   outside this repository would break. Nothing in the repository does
   (grep), and the survey records no width.

## Brief corrections

- The brief says σ = 0 "cannot reach the line (#88 refuses it)". It cannot
  reach the result line, but a part line can carry a zero part (#88 checks
  the total's support only); the function handles it as above.
- The brief's units list is barn-only; the result line also prints Γ (see
  Found 4).
- The worktree had no `target/` and 4.8 GB free; the release-debug gate
  needed my debug build removed first. `CARGO_PROFILE_RELEASE_DEBUG_DEBUG=0`
  is not a usable disk knob for the `release-debug` profile: cargo reads it
  as `profile.release` and refuses the run.
