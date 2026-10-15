---
type: Session Report
title: "Q-H report: hygiene review of the same-flavour four-fermion per-graph gate"
description: "Hygiene review of PR #35's per-graph AMP() route: findings written before any edit, then the local ones fixed with run mutations and an identity check on standalone_jamps' printed numbers."
status: draft
tags: [hygiene, review, four-fermion, madgraph-oracle, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: gate, resource: "https://github.com/nsmith-/vibegraph/commit/98257fb", title: "test(amplitudes): gate same-flavour four-lepton contacts per MadGraph graph (the change reviewed)"}
---
Review of `git diff main...HEAD` on `amp-signs/same-flavour-4f-contact` at
`a7a37c3`: the `bank_amps` option, `read_jamp_coefficients` and the
`ee_to_ee_4f` row in `validation/madgraph/gen_standalone_jamps.py`, and the
per-graph level (`MG_GRAPH_DIAGRAMS` and its comparison) in
`vibegraph-lib/tests/standalone_jamps.rs`. Line numbers are at `a7a37c3`.

## Findings (written before any edit)

| id | pt | site | evidence | mutation the test misses | conf. | disposition |
|---|---|---|---|---|---|---|
| H1 | 1 | `gen_standalone_jamps.py:227` | `read_jamp_coefficients` is a second copy, under the same name, of `gen_amplitude_tables.py:180`'s parser of the same MadGraph statement (`JAMP(1)` in standalone, `JAMP(1,1)` in MadEvent). The older copy refuses a repeated `AMP()`, counts coverage, and reads complex coefficients; the new one does none of these. | — | checked | fix: one shared parser |
| H2 | 1, 2 | `gen_standalone_jamps.py:235` | The term regex is not anchored and nothing checks that it consumed the statement. Probe on synthetic `matrix.f` files: `AMP(1)+IMAG1*AMP(2)` returns `[1.0, 1.0]`, `TMP_JAMP(1)+AMP(2)` returns `[1.0, 1.0]` (the `AMP(1)` inside `TMP_JAMP(1)`), `AMP(1)+AMP(2)+AMP(2)` returns `[1.0, 2.0]`; a complex literal and `(1D0/2D0)` raise `ValueError`. On the real `ee_to_ee_4f/…/matrix.f` it returns the committed 19 coefficients exactly. Census of the 64 single-flow MadEvent `matrix1_orig.f` under `validation/madgraph/output`: every `JAMP(1,1)` is wholly made of `[+]AMP(k)` and `[+](±r.rD±ee)*AMP(k)` terms (1436 × `−1`, 381 × `+1`, 5 × `2`), no `IMAG1`, no `TMP_JAMP` (which appears only in multi-flow files, refused by `bank_amps`). So the parser fits every observed statement, and a mis-parse would still fail the Rust gate (per-graph and per-flow agreement together imply Σ c·AMP = JAMP(1)), but as a vibegraph disagreement, not as a parse error | a statement with an unrecognised term is read silently | checked | fix: refuse an unconsumed remainder |
| H3 | 1 | `gen_standalone_jamps.py:18` | Docstring: "dumped with `--amps` for diagnosis and banked only for a row that sets `bank_amps`". The body writes `amps` into the table for any row when `--amps` is given (`amps = args.amps or row.bank_amps`, written to `standalone/<key>.json` unless `--out`). What `bank_amps` alone adds is the coefficients, and only those make the Rust side gate per graph | — | checked | fix: say what each flag does |
| H4 | 4 | `standalone_jamps.rs:81` | `MG_GRAPH_DIAGRAMS` maps each MadGraph graph to a group of our diagrams. `amplitude_oracle.rs:349`'s `MG_DIAGRAM_ORDER` maps the other way: "MadGraph graph index of each vibegraph diagram". The next declared row is F1's `wpwm_to_wpwmz_cw` (F1 report, "How F2 should bank it": "`MG_GRAPH_DIAGRAMS` one diagram per graph from this order"), whose 222-entry pairing F1 derived as diagram → graph and which F2 must also enter in `MG_DIAGRAM_ORDER`. In today's shape that row is 222 one-element slices of the *inverted* order. `ee_to_ee_4f` itself is 16 of 19 entries singletons | — | checked | fix: declare the graph of each diagram |
| H5 | 2 | `standalone_jamps.rs:299` | `entry["amps"].…iter().zip(groups.iter())`: the per-entry `AMP()` list is never checked against `NGRAPHS`; `zip` stops at the shorter. `coefs.len()` and `groups.len()` are checked, the data the comparison reads is not | a table whose entries drop their last `AMP()` (graph entries 1216 → 1152) passes | checked (run below) | fix |
| H6 | 2 | `standalone_jamps.rs:196` | The declaration is checked one way: a table with coefficients and no declaration panics, a declared row whose table lacks `jamp_coefficients` skips the per-graph level silently and prints no graph line | `jamp_coefficients` removed from `ee_to_ee_4f.json` passes | checked (run below) | fix: two-way |
| H7 | 1 | `standalone_jamps.rs:155` | `measure` was 133 lines at `main` and is 220 at the tip; the per-graph level adds about 75 lines inside it (setup, per-entry sums, worst-graph scan) with no named reason | — | checked | fix: move the setup and the per-entry sums out |
| H8 | 4 | `standalone_jamps.rs:214` | The one-diagram `DiagramSet` + `AmplitudeEvaluator::compile` pattern, field by field: `grep -rn 'diagrams: vec!\['` gives `helas/eval/renumbering.rs:241`, `helas/eval/rooting_soundness.rs:471,616,748,870`, `tests/amplitude_oracle.rs:1053`, `tests/standalone_jamps.rs:223` | — | checked | Found |
| H9 | 4 | `standalone_jamps.rs:81`, `amplitude_oracle.rs:349` | Once F2 banks `wpwm_to_wpwmz_cw`, the same pairing is declared in two test files (F1's plan says so explicitly), with nothing checking that they agree | — | checked | Found |
| H10 | 1 | `gen_standalone_jamps.py:379`, `amplitude_oracle.rs:589` | The key `jamp_coefficients` holds real numbers in standalone tables and `[re, im]` pairs in amplitude tables (`gen_amplitude_tables.py:494`). Unifying needs the committed table rewritten, which this session does not do | — | checked | Found |
| H11 | 1 | `research/kb/backlog/validation/wpwm-to-epem-neutrino-exchange-sign.md:30` | Cites `standalone_jamps.rs:269` for the known-disagreement banking; `KNOWN_DISAGREEMENT` is at line 234 on `main` and 381 at the tip | — | checked | Found |
| H12 | 2 | `standalone_jamps.rs:206` | The partition check (sorted indices equal `0..n`) is two-way: it refuses a missing and a repeated diagram. `graph_scale > 0` refuses an all-zero table | — | checked | leave: sound |
| H13 | 3 | the diff | `git diff main...HEAD \| grep '^+.*pub '` is empty; everything new is test-local or script-local | — | checked | leave |
| H14 | 1 | the diff | Comment grep (`\bT[0-9]\b\|\bC[0-9]\b\|Phase [0-9]\|note [0-9]+\|used to\|no longer`, plus `session\|sprint\|Q1`) over the added lines is empty | — | checked | leave |

The `ee_to_ee_4f` paragraph of the `MG_GRAPH_DIAGRAMS` doc comment (the
counting-convention statement awaiting the user) is out of scope for edits and
is kept byte for byte.

Backlog check: items naming `standalone_jamps` or `gen_standalone_jamps`:
`wpwm-to-epem-neutrino-exchange-sign` (its line citation is H11),
`gg-ttxg-no-amplitude-gate` (names the generator only; unaffected) and
`same-flavour-four-fermion-diagram-count-ungated` (the item this PR closes).
None of their sites is fixed or broken by the review's fixes.
