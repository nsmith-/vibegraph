---
type: Session Report
title: "T-H report: hygiene review of the triple-gluon derivation, and the concepts it corrects"
description: "Hygiene review of PR #39's triple_gluon test module and triple_gluon_sign doc: findings written before any edit, the local ones fixed under a to_bits identity digest, the stated blind spots confirmed by mutation, and four kb concepts rewritten from T1's derivation."
status: draft
tags: [hygiene, review, sign-convention, triple-gluon, colour, crossing, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-16}]
sources:
  - {id: t1, resource: "T1-report.md", title: "T1 report: the triple-gluon −1 is the slot exchange every other triple-vector vertex gets from the crossing"}
  - {id: brief, resource: "T-H.md", title: "T-H brief"}
  - {id: findings, resource: "https://github.com/nsmith-/vibegraph/commit/bb412dd", title: "kb: T-H findings, before any edit"}
  - {id: code, resource: "https://github.com/nsmith-/vibegraph/commit/ca3073c", title: "test(amplitudes): triple_gluon reads through the shared single-diagram probe and refuses an empty table"}
  - {id: concepts, resource: "https://github.com/nsmith-/vibegraph/commit/1900957", title: "kb: the triple-gluon -1 is the crossing's slot exchange, not colour"}
---
Reviewed: `9549fcb` (the code commit of PR #39) against its parent, that is
`vibegraph-lib/src/helas/eval/triple_gluon.rs` (new, 531 lines, four tests),
the doc and visibility of `triple_gluon_sign` and the `convention_sign` bullet in
`root_diagram.rs`, and the `#[cfg(test)] mod triple_gluon;` line in
`helas/eval/mod.rs`. Built on `a1e48bd`.

## Findings

Written before any edit. Line numbers are those of `a1e48bd`.

| id | pt | site | evidence | mutation the test should catch | conf. | disposition |
|---|---|---|---|---|---|---|
| H1 | 1 | `triple_gluon.rs:108–290` | `MadGraphRow::read` is 183 lines: table parse, model and diagrams, basis check, colour, per-diagram compile, per-point evaluation and the transcription check in one body | — | checked | fix |
| H2 | 1, 4 | `triple_gluon.rs:185–271` | compiles each diagram as a one-diagram `DiagramSet` and runs `eval_amplitude` / `run_flows` by hand; `renumbering::single_diagram_amplitudes_at` (`renumbering.rs:242`, already `pub(super)`) is that loop. Filed item `one-diagram-diagramset-built-by-hand` lists seven sites; this is an eighth | — | checked | fix |
| H3 | 1 | `triple_gluon.rs:79–91` | `coeff` evaluates `q · 3^nc_power` by hand and asserts `denom == 1`; `ColorCoeff::eval_nc` (`color/coeff.rs:121`) is that evaluation, and `probe_bridge::colour_coeff` (`run/probe_bridge.rs:187`) is the same conversion to a complex | — | checked | fix (local: `eval_nc`); the cross-module copy → Found |
| H4 | 1, 4 | `triple_gluon.rs:94–105` | `key_name` renders a basis key in MadGraph's notation; `tests/color_cf_oracle.rs:372` `render_structure` does the same, differently (`key_name` prints `TensorKind`'s `Debug`, so `F(...)` where MadGraph writes `f(...)`). A lib unit test cannot reach `tests/` | — | checked | Found |
| H5 | 4 | `triple_gluon.rs:111–146, 221–234` | the banked-table reader (param card, helicities, momenta) is also written at `rooting_soundness.rs:121–150, 422–455, 527, 684`. Filed item `test-banked-readers-duplicated` covers only `tests/` | — | checked | Found |
| H6 | 2 | `triple_gluon.rs:215–219` | `detailed` filters the points with a `detail`; nothing asserts it is non-empty. On an empty set the transcription check runs zero times and `the_triple_gluon_diagram_has_madgraphs_colour` compares our colour with unverified constants | MB: filter keeps no point → the colour test must fail | suspected (settled below) | fix |
| H7 | 2 | `triple_gluon.rs:112, 202, 395, 400, 422, 482` | failure messages: `read_to_string(..).unwrap()` drops the path; `position(..).unwrap()` drops the key; `\|G\| = {}` drops the process and `G`; `assert_eq!(row.triple_gluon[s], -1.0)` prints only `left: 1.0` (T1's M2 output); `phi` norm and the all-`+1` check print nothing | — | checked | fix |
| H8 | 1 | `triple_gluon.rs:55` | cites MadGraph's `matrix1.f`; the banked export has `SubProcesses/P1_gg_ttx/matrix1_orig.f` (lines 426–429: `JAMP(1,1) = (0,1)·AMP(1) + (−1)·AMP(2)`, `JAMP(2,1) = (0,−1)·AMP(1) + (−1)·AMP(3)`), which the constant matches | — | checked | fix |
| H9 | 2 | `triple_gluon.rs:252–258` | brief question: is the JAMP transcription checked against the banked `JAMP()`? The loop rebuilds each banked flow from the transcribed row and the banked `AMP()` | MA: `JAMP(1)`'s `i` → `−i` must fail `read` | suspected (settled below) | leave if MA fails |
| H10 | 2 | `triple_gluon.rs:22–26` and the T1 table | stated blind spots | M1 (`triple_gluon_sign` → `+1`): colour and rest tests pass, totals fails; MC (a sign on every diagram's `fermi_sign`): all four pass | suspected (settled below) | confirm |
| H11 | 1 | `root_diagram.rs:1122` | a doc line of 133 characters (`awk 'length > 100'`); every other line of the new doc wraps at 88–90 | — | checked | fix |
| H12 | 2 (physics) | `root_diagram.rs:1116–1118` | "`VVV1`, like every triple-vector structure the SM and SMEFTsim put on a W pair, is odd under that exchange" rests on T1's scratch `vvv_swap.py`; no committed test pins it. `grep -rln vvv_swap research/kb/backlog` is empty | a W-pair structure even under the exchange | checked | routed (validation-dev) / Found |
| H13 | 3 | `root_diagram.rs:1143` | `pub(super) fn triple_gluon_sign`: its only reader outside `root_diagram` is the `cfg(test)` sibling `triple_gluon` | — | checked | leave: `pub(super)` is the narrowest visibility that reaches a sibling module, and making the tests a child of `root_diagram` needs a `#[path]` attribute for no gain |
| H14 | 2 | `triple_gluon.rs:400` | the totals test also asserts the literal `triple_gluon_sign == −1`, so it fails on T1's M2/M3 although every value agrees | — | checked | leave: it is the test's stated claim ("the triple-gluon sign is on"); the colour and rest tests own the split |
| H15 | 2 | `triple_gluon.rs:306–315` | `fit`'s deviation is relative to the largest MadGraph value over all entries, not per entry | — | checked | leave: the defects in scope are whole-diagram signs, O(1) on the largest entries; T1's M1 reads 4.0e-1 |
| H16 | 1 | `triple_gluon.rs:364–384, 473–474` | `sm_model` is called again after `read` already loaded it | — | checked | leave: `sm_model` is a `OnceLock` cache |

`git diff 9549fcb^ 9549fcb` adds no plan, session or history reference
(`grep -nE '\bT[0-9]\b|\bC[0-9]\b|Phase [0-9]|note [0-9]+|used to|no longer|session|sprint|backlog'`
on both files returns nothing). The doc of `triple_gluon_sign` matches the T1
report's derivation, steps 1–5 and "The convention, stated".

Backlog items naming a reviewed path (`grep -rln 'triple_gluon\.rs\|/root_diagram\.rs\|triple_gluon_sign'`
and the eval siblings): `colourless-self-conjugate-vvv-sign-unoracled` (T1
Found 3; its sites are right), `one-diagram-diagramset-built-by-hand` (H2),
`test-banked-readers-duplicated` (H5), `standalone-amps-single-flow-only` and
`per-diagram-dump-tools-uncommitted` (T1 Found 4 and 5; right).

## Final dispositions

| id | disposition |
|---|---|
| H1 | **fixed** (`ca3073c`): `read` is 97 lines; `read_table`, `param_card`, `detailed_points`, `flow_maps` and `check_transcription` carry the rest |
| H2 | **fixed** (`ca3073c`): each diagram is evaluated through `single_diagram_amplitudes_at`, with an assert that it returns one value per own flow and helicity (the old `zip` truncated silently) |
| H3 | **fixed** locally (`ca3073c`): `coeff` is `eval_nc(3)`; the second copy in `probe_bridge` → Found |
| H4 | **Found** |
| H5 | **Found** |
| H6 | **fixed** (`ca3073c`), mutation MB below |
| H7 | **fixed** (`ca3073c`) |
| H8 | **fixed** (`ca3073c`) |
| H9 | **leave**: the check works (MA fails `read` in all three `g g > t t~` tests) |
| H10 | **confirmed**: every stated blind spot reproduces (M1, M2, MC below) |
| H11 | **fixed** (`ca3073c`) |
| H12 | **routed** to validation-dev (needs a committed oracle) |
| H13–H16 | **leave**, reasons in the table |

## Identity

Chosen before the first edit: a temporary test `identity_digest` that hashes
the `to_bits` of every entry of `values`, `amps`, `colour`, `jamp` and
`triple_gluon` of both rows (`gg_to_ttx`, `ee_to_wpwm`), run with the four tests
under `--nocapture`. Before (on `a1e48bd` plus the digest) and after (the refactor
plus the digest), the whole filtered output is byte-identical
(`diff target/th-identity-before.log target/th-identity-after.log` is empty):

```
DIGEST gg 1755 3b882228ef54916b
DIGEST ee 1311 0cc28691d16410e3
```

The digest test was removed before the commit.

## Mutations

Driver: a scratch `mutate.py` that applies each named edit by exact-string
replacement (asserting one occurrence), runs
`cargo test -p vibegraph-lib --lib helas::eval::triple_gluon -- --test-threads=1 --nocapture`
with `CARGO_BUILD_JOBS=3 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0`, logs, and restores the saved file contents.
`git diff --stat -- vibegraph-lib` was empty after each batch.

| id | edit | old tests (`a1e48bd`) | new tests (`ca3073c`) |
|---|---|---|---|
| MA | `GG_TO_TTX_JAMP[0][0]`: `i` → `−i` | 3 `g g > t t~` tests fail at the transcription check (`triple_gluon.rs:254`); the `e+ e- > w+ w-` test passes | same, at `:220` |
| MB | `detailed_points`' filter keeps no point | **`the_triple_gluon_diagram_has_madgraphs_colour` passes**; the other three fail only through NaN (`worst NaN, G = NaN+NaNi`) | all four fail at `:276`: `ee_to_wpwm: no banked point carries MadGraph's per-graph AMP()` (and `gg_to_ttx: …`) |
| M1 | `triple_gluon_sign` returns `+1` | totals fails (`worst 4.039e-1`, `\|G\| = 0.628`), message `\|G\| = 0.6282175585242568`; colour, rest and control pass | same split; message `g g > t t~: the fitted G = …-0.6282175585242568i is not a phase (\|G\| = 0.6282175585242568)` |
| M2 | `f_to_traces` negated and `triple_gluon_sign` at `+1` | totals fails on the literal only (values at 1.720e-15), message `left: 1.0`; colour fails (`diagram 1, flow 0: … -1+-0i, against 1+0i`); rest and control pass | same split; message `triple_gluon_sign on the triple-gluon diagram 0 of g g > t t~` |
| MC | every diagram's `fermi_sign` negated | all four pass (`G`, `K` flip sign) | not re-run (no assert it reaches changed) |

M1, M2 and MC confirm the module doc's and T1's blind spots: the rest test
cannot see a flip of `triple_gluon_sign` alone, the colour test sees nothing
outside colour, and no test sees a sign shared by every diagram. MB shows H6
was live: on an empty set the colour test was green against coefficients
nothing had checked.

**Sites no mutation reaches.** The slot check of
`a_charged_triple_vector_vertex_carries_its_sign_in_its_kernel` (each
final-state W in its antiparticle's slot) has no one-line production mutation:
changing feyngraph's slot reading moves every vertex of every process. Its
`assert_ne!` follows from the `assert_eq!` before it for any non-self-conjugate
leg, so it adds no power. Left as is.

## Concepts rewritten

From T1's "The derivation" and "The convention, stated", each at
`status: draft` (`1900957`):

- `amplitudes/vector-vertex-signs`: description; the table gains a column
  for the kernel's reading against MadGraph (charged VVV `−1` in the kernel,
  triple gluon `+1` in the kernel and `−1` from the constant); the "why no sign"
  section is restated as why `convention_sign` has no factor, with the slot
  exchange; "The triple-gluon `−1`" gives the colour, slot-order and coupling
  evidence and the proxy predicate; T1's M4 replaces "no mutation has been run
  that puts a sign on a colourless vector vertex alone"; the four factor tests
  and their blind spots are tabled; the "has no derivation" open item is
  replaced by the predicate proxy, the uncommitted parity check and the
  unbanked `u u~ > g g` `AMP()`.
- `amplitudes/convention-sign-inventory`: the triple-gluon paragraph, a line
  saying the slot exchange is why the GF(2) constant is `−1`, the arms table
  row, and the open item.
- `validation/convention-channel-coverage`: "calibrated, not derived" replaced
  by T1's M2/M3 finding that both MadGraph gates are blind to which factor
  carries the sign, and the proxy predicate.
- `validation/bit-exact-amplitude-debugging`: the one "calibrated `−1`"
  mention. Its "triple-gluon `+1`" at step 4 is a historical candidate, left.

## Routed

- **H12 → validation-dev.** `triple_gluon_sign`'s doc and
  `vector-vertex-signs` state that every triple-vector structure the SM and
  SMEFTsim put on a W pair is odd under the W⁺/W⁻ slot exchange. The evidence is
  T1's scratch `vvv_swap.py` (numeric parity at random momenta); nothing
  committed pins it, and a structure even under the exchange would need the
  `−1` this crate does not give it. Closing it needs a committed test that
  evaluates each such UFO Lorentz structure under the exchange of two slots and
  their momenta.

## Gate

Every cargo call with `CARGO_BUILD_JOBS=3 CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`, on `ca3073c`.

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0
  (`Finished \`dev\` profile [optimized] target(s) in 20.70s`).
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: exit 0.
- `cargo doc -p vibegraph-lib --no-deps --document-private-items`: 8 warnings,
  all pre-existing and outside the touched files (`coupling/`, `phasespace/channel.rs`,
  `ufo/parameters.rs`, `lhef/mod.rs`).
- `cargo test --workspace --no-fail-fast`: exit 0, 38 suites, 1439 passed, 0
  failed, 16 ignored (summed `test result` lines). No test added or deleted:
  `git diff a1e48bd HEAD -- '*.rs' | grep -c '^[+-].*#\[test\]'` is 0. T1's
  1438 at `9549fcb` plus the one test the `5566169` merge adds is 1439.
- `cargo test -p vibegraph-lib --lib helas::eval::triple_gluon -- --nocapture`:
  4 passed, the printed numbers identical to T1's (`worst 1.720e-15`,
  `K = +0.999999999999999i`, `e+ e- > w+ w-` `worst 1.046e-15`).
- No banked target touched: the change is a `cfg(test)` module and a doc
  comment, and the identity digest shows no value moved.
- `pixi run -e kb kb-lint`: `628 concepts, 212 backlog items; 0 errors, 2 warnings`
  (the two pre-existing `state: blocked` warnings). `python3 scripts/kb.py index
  --check`: exit 0, after `python3 scripts/kb.py index` (`wrote 2 index files`).

## Method

- Diff reading and the checklist greps (comment, line length, `unwrap`
  without context): about 25%. Found H7, H8, H11.
- Workspace grep of distinctive body lines (`diagrams: vec![d.clone()]`,
  `param_card"]`, `eval_nc(`, `ColorOne`) and of the backlog by path tail:
  about 25%. Found H2–H5, and that H2 and H5 extend filed items.
- Asking what each test is blind to, then running it: about 35% (two batches,
  nine mutation runs). Settled H6, H9, H10; MB was the only finding the
  reading suspected that a run turned from "probably" into "live".
- Identity oracle and the concept rewrite: the rest.
- Leads that did not hold: the `fit` tolerance relative to a global maximum
  (H15) and the redundant `assert_ne!` look loose but cannot let a
  whole-diagram sign through.

## Found

1. **Two copies of the colour-coefficient conversion.** `run/probe_bridge.rs:187`
   `colour_coeff` and `triple_gluon.rs` `coeff` both turn a `ColorCoeff` into a
   complex at `Nc = 3`; `lower.rs:108` and `colorize.rs:477` do it inline. To
   close: one `cfg(any(test, doc))` method on `ColorCoeff` (or one helper in
   `helas::color`) used by both test modules.
2. **The lib-side banked-table reader is written five times.**
   `rooting_soundness.rs:121–150, 422–455, 527, 684` and `triple_gluon.rs`
   (`read_table`, `param_card`, `detailed_points`) each parse a
   `validation/madgraph/amplitudes/*.json` table's param card, helicities and
   momenta. `test-banked-readers-duplicated` names only the `tests/` copies. To
   close: one `cfg(test)` reader in `helas::eval` (or the crate's test
   support), used by both modules.
3. **Two renderings of a colour basis key in MadGraph's notation.**
   `triple_gluon.rs` `key_name` (prints `TensorKind`'s `Debug`, so `F(...)`) and
   `tests/color_cf_oracle.rs:372` `render_structure` (`f(...)`, `ColorOne()`).
   The first would mismatch MadGraph on an `f`-structure basis, loudly. To
   close: one renderer in `helas::color`, used by both.
4. **`amplitudes/global-phase-i-counting.md:96–100`** says "a colourless VVV
   vertex takes no sign at all. The only vector-vertex sign is the
   triple-gluon `−1`". It is true of `convention_sign` but reads as if the
   charged vertex's kernel agreed with MadGraph, which it does not. The brief
   did not list it. To close: the same restatement as `vector-vertex-signs`.
5. **The W-pair structures' parity (H12)**, if not routed: a committed test of
   each W-pair triple-vector structure's parity under the slot exchange.

## Brief corrections

- The brief lists `tests/common/mg_graphs.rs` as a helper the module might
  duplicate. It holds graph pairings and rounded-coupling tables, not readers,
  and a library unit test cannot reach `tests/common` at all. The duplicates
  that are reachable live in `helas::eval` itself (`renumbering`,
  `rooting_soundness`, `run::probe_bridge`).
- The brief calls `bit-exact-amplitude-debugging`'s triple-gluon text "the
  'calibrated −1' mentions"; there is one. A fifth concept,
  `global-phase-i-counting`, carries the same stale premise (Found 4).
