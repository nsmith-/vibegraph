---
type: Session Report
title: "Q-H report: hygiene review of the same-flavour four-fermion per-graph gate"
description: "Hygiene review of PR #35's per-graph AMP() route: findings written before any edit, then the local ones fixed with run mutations and an identity check on standalone_jamps' printed numbers."
status: draft
tags: [hygiene, review, four-fermion, madgraph-oracle, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-15}]
sources:
  - {id: 2fce423, resource: "https://github.com/nsmith-/vibegraph/commit/2fce423", title: "kb: Q-H brief and findings"}
  - {id: 8569c3f, resource: "https://github.com/nsmith-/vibegraph/commit/8569c3f", title: "validation: one JAMP(1) coefficient parser, refusing terms it cannot read"}
  - {id: 0440656, resource: "https://github.com/nsmith-/vibegraph/commit/0440656", title: "test(amplitudes): declare each diagram's MadGraph graph, checked both ways"}
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

## Dispositions

| id | final | commit |
|---|---|---|
| H1 | fixed: `parse_jamp_coefficients` in `gen_amplitude_tables.py` is the one term parser; both readers call it | `8569c3f` |
| H2 | fixed: the parser must consume the whole right-hand side, signs required after the first term, each `AMP()` once and all NGRAPHS present; the standalone reader refuses a complex coefficient because its tables bank reals | `8569c3f` |
| H3 | fixed: the docstring says `--amps` adds `AMP()` for diagnosis and `bank_amps` adds them with the coefficients the Rust gate reads | `8569c3f` |
| H4 | fixed: `MG_GRAPH_DIAGRAMS` names each diagram's graph (see the interface section) | `0440656` |
| H5 | fixed: each entry's `AMP()` list must have NGRAPHS entries | `0440656` |
| H6 | fixed: the declaration and the table are checked against each other both ways | `0440656` |
| H7 | fixed in part: the setup and per-entry sums moved into `GraphLevel::read` and `GraphLevel::entries`; `measure` is 170 lines against 220 at the tip and 133 at `main`. Its remaining length is the three deviation scans and the table reading it already had | `0440656` |
| H8–H11 | Found | — |
| H12–H14 | rejected: sound, or nothing to find | — |

## Python parser: identity and mutations

No MadGraph run. The checks use the generated process directories already
under `validation/madgraph/output/`, with the pre-fix scripts copied aside and
loaded beside the fixed ones (`pixi run -e madgraph python -I identity_py.py`):

```
MadEvent single-flow JAMP(1,1): 64/64 identical old vs new
old standalone on ee_to_ee_4f matrix.f == committed: True
new standalone on ee_to_ee_4f matrix.f == committed: True
IMAG1 factor: old: [1.0, 1.0] | new: AssertionError: JAMP(1): unrecognised term at '+IMAG1*AMP(2)'
TMP_JAMP temporary: old: [1.0, 1.0] | new: AssertionError: JAMP(1): unrecognised term at 'TMP_JAMP(1)+AMP(2)'
repeated AMP: old: [1.0, 2.0] | new: AssertionError: JAMP(1): AMP(2) appears twice
complex literal: old: ValueError: '0.000000000000000E+00,1.000000000000000E+00' | new: AssertionError: a complex JAMP(1) coefficient: [(1+0j), 1j]
missing AMP: old: AssertionError: [1.0, 0.0] | new: AssertionError: JAMP(1): covers 1/2 graphs
unsigned term: old: [1.0, 1.0] | new: AssertionError: JAMP(1): unrecognised term at 'AMP(2)'
control (-1, 2): old: [-1.0, 2.0] | new: [-1.0, 2.0]
```

The "old" column of the first six cases is the standalone reader at `a7a37c3`.
Four of them returned a wrong list with no error. The MadEvent reader at
`a7a37c3` already refused the repeated and the missing `AMP()`. It read past
`IMAG1` and `TMP_JAMP` like the standalone one did. So for the brief's
question: the parser fit every statement MadGraph 3.7 wrote here (64 MadEvent
single-flow processes and the one standalone one), but only by luck. It now
refuses any form outside that set instead of misreading it.

## Rust per-graph level: mutations

Driven by `json_mut.sh` (table edits, restored by copy and `cmp`) and
`src_mut.py` (source edits, applied, run and restored by copy). Each runs
`cargo test -p vibegraph-lib --test standalone_jamps`.

| mutation | at `a7a37c3` | after the fix |
|---|---|---|
| J1: every entry drops its last `AMP()` | **passes**: `1152 graph entries, worst graph 2.981e-14`, `ok` | fails: `ee_to_ee_4f: point 0, helicities [-1, 1, 1, -1]: one AMP() per graph` |
| J2: `jamp_coefficients` removed from the table | **passes**: no graph line, `ok` | fails: `ee_to_ee_4f: declares MG_GRAPH_DIAGRAMS but its table banks no AMP()` |
| D1: diagrams 6 and 9 swap graphs (`3, 4, 5, 6` → `6, 4, 5, 3`) | not run (the comparison is unchanged) | fails: `worst graph 1.885e-1`, `AMP(4) = diagrams [9]` |
| D2: graph 18 named by no diagram (`17, 18` → `17, 17`) | not run | fails: `ee_to_ee_4f: graphs [18] have no diagram` |
| D3: one diagram short (last entry dropped) | not run | fails: `MG_GRAPH_DIAGRAMS must name one graph for each diagram` |
| S1: Q1's falsifier, `fermi_sign` × −1 on a diagram with a flow-group-1 vertex (`root_diagram.rs`) | Q1 report: fails, `worst graph 7.261e-1` | fails as Q1 recorded it, to the digit: `worst graph 7.261e-1` (`AMP(12) = diagrams [14]`), worst flow 2.010e0, `the flows agree only up to \|G\| = 0.12694975993715618`; `root_diagram.rs` restored (`git diff --stat vibegraph-lib/src` empty) |

In every case the other ten rows pass, unchanged. D1–D3 exercise the new
declaration checks and the comparison the refactor rewrote. J1 and J2 are the
two vacuities H5 and H6 named. No mutation can reach the blind spots Q1
records: a swap of the two pairing diagrams inside one contact, and a swap of
the equal `AMP(5)`/`AMP(6)` or `AMP(13)`/`AMP(14)` pairs. Neither the
refactor nor the new checks change that.

## Identity check

The oracle was chosen before the edit. It is `standalone_jamps`' printed
lines, plus the `to_bits` of every per-graph entry.

- **Printed lines.** `cargo test -p vibegraph-lib --test standalone_jamps -- --nocapture`
  at `8569c3f` (test file as at `a7a37c3`) and after the refactor. The 13
  `<key>: … entries …` and `informational` lines, sorted, are identical:
  `diff` is empty and both files have md5 `aa948736ad683eaee63d34f1a2071cbb`.
  Among them are `ee_to_ee_4f: 64 flow entries, G = Complex { re: -5.846016896213112e-17, im: -1.0000000000000109 }, worst flow 4.866e-14, worst |M|² 8.657e-14`
  and `ee_to_ee_4f: 1216 graph entries, worst graph 2.981e-14`.
- **Bits.** I added a temporary `eprintln!` of the `to_bits` of all four
  components of each of the 1216 `(c·AMP, group sum)` entries, and of
  `worst_graph`, to the old and the new file in turn. The md5 of that output
  is `9c917311e098a6ceefe6a3e54ae00a09` both times. The line was removed
  after, and `cmp` confirmed the restored file. Each group is still summed in
  increasing diagram order from zero, which is why the bits agree.

## Interface change to the per-graph machinery

Two things in `vibegraph-lib/tests/standalone_jamps.rs` change. Everything else
the siblings use (`bank_amps`, the table keys, `REL_TOL`, `check`, `measure`'s
output) is unchanged.

1. **`MG_GRAPH_DIAGRAMS: &[(&str, &[usize])]`**, which was `&[(&str, &[&[usize]])]`.
   Entry `d` is the 0-based MadGraph graph (`AMP()` index − 1) of diagram `d`.
   This is the convention of `amplitude_oracle`'s `MG_DIAGRAM_ORDER` and of
   F1's derived `wpwm_to_wpwmz_cw` order, which can now be pasted as it is.
   `ee_to_ee_4f` becomes
   `[0, 0, 1, 1, 2, 2, 3, 4, …, 18]`. The checks are equivalent to the old
   partition check: the length must equal the diagram count, every value must be
   below NGRAPHS, and every graph must be named at least once.
2. **Two-way declaration.** A row listed in `MG_GRAPH_DIAGRAMS` whose table has
   no `jamp_coefficients` now panics. Before, it was silently compared per flow
   only.

The per-graph code moved into `GraphLevel::{read, entries}`. Its messages are
unchanged except the new refusals. The `ee_to_ee_4f` paragraph of the doc
comment, the convention statement awaiting the user, is byte-identical. Only
the paragraph above it, which describes the shape, changed. Neither sibling
branch touches `standalone_jamps.rs` (`git diff
amp-signs/same-flavour-4f-contact...amp-signs/wpwmz-cw-five-vector --stat` and
the same for `identity-derivative-scalar-signs` list no `.rs` file), so the
change merges without conflict.

The Python side's interface is unchanged: `read_jamp_coefficients(pdir,
ngraphs)` keeps its signature, and the table schema is unchanged.
`gen_standalone_jamps.py` now imports `join_continuations` and
`parse_jamp_coefficients` from `gen_amplitude_tables.py`, which imports
`pylhe`. Both scripts run under `pixi run -e madgraph`, which provides it.

## Gates

Every cargo invocation ran with `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`, on the tree of `0440656`.

- `cargo fmt --all --check`: `exit 0`.
- `cargo clippy --workspace --all-targets -- -D warnings`:
  `Finished dev profile [optimized] target(s) in 1m 56s`, exit 0.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
  `Finished dev profile [optimized] target(s) in 34.40s`, exit 0.
- `cargo test -p vibegraph-lib --test standalone_jamps -- --nocapture`:
  `test result: ok. 11 passed; 0 failed`. The printed lines hash to
  `aa948736…`, the same as before the change.
- `cargo test --workspace`: exit 0, 38 suites, 1396 passed, 0 failed,
  16 ignored (summed `test result` lines). This equals Q1's count, and no
  test was added or deleted: the new checks are asserts inside the existing
  `ee_to_ee_4f` test.
- `cargo doc -p vibegraph-lib --no-deps --document-private-items`: exit 0,
  `generated 8 warnings`. All 8 are in library sources this session did not
  touch (`write` module/macro, redundant link targets, `ScaledChannel::ndim`,
  `graph::ChannelSet`, `i32` tag). The only Rust file changed is an
  integration test, which `cargo doc` does not build.
- No banked target was run. `standalone_jamps` has no `required-features`
  (it is hermetic and runs in the workspace suite). The Python change moves
  no banked number: the 64/64 MadEvent identity above covers the amplitude
  tables' reader, and no table was regenerated.
- A false failure along the way, to warn later sessions: the first gate run's
  `standalone_jamps` reported S1's mutated numbers. The mutation driver had
  restored `root_diagram.rs` from a backup with an older mtime, so cargo
  reused the mutated library. After a `touch`, the rerun above passed. A
  mutation driver should restore by rewriting the file's text, or `touch`
  it afterwards. The workspace run had already passed, because its feature
  set rebuilt the library.

## Method

- **Reading the diff against its own module and against its older sibling.**
  About 35% of the session. A grep for the new function's name found
  `gen_amplitude_tables.read_jamp_coefficients` (H1). A grep for the
  declaration's concept found `MG_DIAGRAM_ORDER` (H4). Reading the F1 report on
  the five-vector branch showed what the second row will look like, and that
  settled H4.
- **Probing the parser on real and synthetic input.** About 20%. A census of
  the 64 single-flow MadEvent statements under `output/` showed which forms
  occur, and a few synthetic one-line `matrix.f` files showed what is misread
  (H2).
- **"Which one-line defect does this assert not see?"** About 25%, with run
  mutations: J1 and J2 passed the old test, which settled H5 and H6.
- **Identity oracle, gates and report.** About 20%.
- **Leads that did not hold up.**
  - The partition check looked one-way at first, but sorting and comparing to
    `0..n` is two-way (H12).
  - A bank-time rebuild check (Σ c·AMP = JAMP(1)) in the standalone generator
    looked missing. The Rust gate already implies it: per-graph agreement and
    per-flow agreement under the same `G` together bound
    |G(Σ c_k AMP_k − JAMP(1))|. So a mis-parse fails loudly, only under a
    vibegraph label. Not added.
  - `per_diagram`'s scratch space is allocated once per diagram per helicity.
    Not worth changing in a test.

## Found

1. **The one-diagram `DiagramSet` is built by hand in seven places** (H8):
   `vibegraph-lib/src/helas/eval/renumbering.rs:241`,
   `vibegraph-lib/src/helas/eval/rooting_soundness.rs:471,616,748,870`,
   `vibegraph-lib/tests/amplitude_oracle.rs:1053` and
   `vibegraph-lib/tests/standalone_jamps.rs` (`GraphLevel::read`). Each copies
   `particles_in`, `particles_out` and `polarizations` field by field. To close
   it: a `DiagramSet` method that returns the set restricted to one diagram,
   or one compiled evaluator per diagram, used at all seven sites.
2. **One pairing, declared in two files** (H9). F1's plan for
   `wpwm_to_wpwmz_cw` enters the same 222-entry diagram → graph order in
   `amplitude_oracle`'s `MG_DIAGRAM_ORDER` and in `standalone_jamps`'
   `MG_GRAPH_DIAGRAMS`, and nothing checks that the two agree. They now share
   a convention, so to close it: one table in `tests/common` that both read,
   or a test asserting equality on the rows both declare.
3. **`jamp_coefficients` has two schemas** (H10). It holds reals in
   `validation/madgraph/standalone/*.json` (read by `as_f64` in
   `standalone_jamps.rs`) and `[re, im]` pairs in
   `validation/madgraph/amplitudes/*.json` (`gen_amplitude_tables.py:494`,
   `amplitude_oracle.rs:589`). To close it: bank `[re, im]` in the standalone
   generator and read complex values in `standalone_jamps`, at the next
   regeneration of `ee_to_ee_4f.json`.
4. **A stale line citation** (H11):
   `research/kb/backlog/validation/wpwm-to-epem-neutrino-exchange-sign.md:30`
   cites `standalone_jamps.rs:269`. It was already 234 on `main` and is 381
   at `a7a37c3`. To close it: cite `KNOWN_DISAGREEMENT` by name.
5. **The Q1 report describes the old declaration shape.**
   `research/kb/sprints/amplitude-signs/sessions/Q1-report.md:88` says
   "`MG_GRAPH_DIAGRAMS` declares, for each MadGraph graph, the group of our
   diagrams that makes it up". The meaning holds (the declaration still
   partitions the diagrams into the graphs), but the data now runs diagram →
   graph. The proposed decision wording under "For user approval" is still
   accurate. To close it: a one-line note in the report or in the manager's
   record. This session edits nothing under `research/kb/` but its own report.
6. **`read_masses` exists twice**, in `validation/madgraph/gen_standalone_jamps.py`
   and `validation/madgraph/gen_amplitude.py:215`, and the standalone script
   already imports `gen_amplitude`. This predates the PR. To close it: delete
   the standalone copy and import the shared one, after checking that the two
   bodies agree.

## Brief corrections

- The brief's gate list leaves out CI's `--all-features` clippy and the
  per-package `cargo doc`, which the hygiene-dev definition requires. Both
  were run (see Gates).
- The brief places the parsing in `bank_amps`. It is `read_jamp_coefficients`,
  which `bank()` calls when the row sets `bank_amps`.
- "The diff against `origin/main`": `main` and `origin/main` are the same
  commit here (`cc8b12c`), so the review used `main...HEAD`.
