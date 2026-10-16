---
type: Session Report
title: "T-H report: hygiene review of the triple-gluon derivation, and the concepts it corrects"
description: "Hygiene review of PR #39's triple_gluon test module and triple_gluon_sign doc: findings written before any edit, the local ones fixed under a to_bits identity digest, the stated blind spots confirmed by mutation, and four kb concepts rewritten from T1's derivation."
status: draft
tags: [hygiene, review, sign-convention, triple-gluon, colour, crossing, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: t1, resource: "T1-report.md", title: "T1 report: the triple-gluon −1 is the slot exchange every other triple-vector vertex gets from the crossing"}
  - {id: brief, resource: "T-H.md", title: "T-H brief"}
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
