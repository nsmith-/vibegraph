---
type: Session Report
title: "B-H report: hygiene review of the root-free convention sign, and the concepts it supersedes"
description: "Hygiene review of B′'s convention_sign change on PR #30: findings written before any edit, then the local ones fixed under a per-diagram digest identity check, and the five kb concepts that described the anchor convention rewritten around convention_sign."
status: draft
tags: [hygiene, review, sign-convention, rooting, relabelling, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-16}]
sources:
  - {id: fa01bfb, resource: "https://github.com/nsmith-/vibegraph/commit/fa01bfb", title: "kb: B-H brief and findings"}
  - {id: 8d21b42, resource: "https://github.com/nsmith-/vibegraph/commit/8d21b42", title: "refactor(amplitudes): one leg binding and one uncrossed-leg set for the live tree and the convention sign"}
  - {id: a9bb102, resource: "https://github.com/nsmith-/vibegraph/commit/a9bb102", title: "refactor(amplitudes): one partner-index pick and one projector arm in build_child"}
  - {id: 0d67c26, resource: "https://github.com/nsmith-/vibegraph/commit/0d67c26", title: "docs(amplitudes): rooting_soundness says what it tests, without history"}
  - {id: f983616, resource: "https://github.com/nsmith-/vibegraph/commit/f983616", title: "test(amplitudes): relabelling reuses the renumbering helpers and refuses an empty comparison"}
  - {id: 5bb34ab, resource: "https://github.com/nsmith-/vibegraph/commit/5bb34ab", title: "validation: rewrap a 104-column row comment"}
  - {id: 65facf1, resource: "https://github.com/nsmith-/vibegraph/commit/65facf1", title: "kb: rewrite the five concepts that described the anchor convention"}
---
Review of B′'s code commits on `amp-signs/wpwm-epem-neutrino-sign` at
`5d4a346`: `6f2df1e` (the relabelling test) and `f1bb5bb` (the convention
sign), that is `git diff d6f2d41 5d4a346 -- vibegraph-lib validation/madgraph`.
Line numbers are at `5d4a346`.

## Findings (written before any edit)

| id | pt | site | evidence | mutation the test misses | conf. | disposition |
|---|---|---|---|---|---|---|
| H1 | 1, 4 | `root_diagram.rs:1018` (`external_bindings`) against `:405–440` (`bake_node`'s leg arm) and `:690–810` (`collect_fermion_pairs`, `mixed_line_final_legs`) | The convention sign binds each external spinor with its own copy of the live tree's leg rule (`charge.anti()` when uncrossed, `crossed: !incoming && !uncrossed`, `ExtLegInfo::adjoint`), and finds the uncrossed legs by a second algorithm: `external_bindings` walks `Diagram::fermion_lines`, the live tree walks the raw rooted tree (`collect_fermion_pairs`). Both compute "the final leg of each line with an initial-state end", a graph property. `vertex_root_signs` reads one, the kernels' bindings the other; a change to either rule alone makes the convention sign disagree with the live tree's binding with nothing to say so | — | checked | fix: one `mixed_line_final_legs(diagram, model)` from `fermion_lines`, one leg-binding function, both used by the live tree and by the convention sign; the raw-tree walk goes |
| H2 | 1 | `root_diagram.rs:1054` (`line_end`) against `diagram.rs:481` (`Diagram::fermion_lines`) | Both follow a fermion line through each vertex's flow-group partner. `line_end` starts at a `(vertex, slot)` and returns the end leg; `fermion_lines` starts at each leg and returns ends and vertices, without slots | — | checked | leave: merging needs a slot-recording walk that no other reader wants; the two share only the three-line step |
| H3 | 1 | `relabelling.rs:229` (`diagram_values`) | A second copy of `renumbering::single_diagram_amplitudes` (`renumbering.rs:233`): build a one-diagram `DiagramSet`, compile, bind, evaluate per helicity. It differs only in taking the helicity list (the permuted one) instead of the evaluator's own. It is also an eighth site of the filed `one-diagram-diagramset-built-by-hand`, which names seven | — | checked | fix: one helper, which takes the helicities |
| H4 | 1 | `relabelling.rs:176` (`ray_leg_sets`) | Re-walks the tree to find the external legs behind each ray. `Diagram::legs_on_start_side` (`diagram.rs:582`, private) computes exactly that per propagator | — | checked | fix: use it |
| H5 | 1 | `relabelling.rs:150` (`load`) | Re-implements `renumbering::load` (`renumbering.rs:101`): a per-(model, card) cache and the restrict-card path. Its `Model` enum is the `(dir, card)` pair the other takes | — | checked | fix: use `renumbering::load` |
| H6 | 2 | `relabelling.rs:258–345` | `compared` is printed, never asserted. A pair with at most one compared diagram cannot show a relative flip and passes whatever the signs: no green-on-empty guard | `VANISHING = 1.0` (only the largest diagram compared) passes | checked (run below) | fix: require at least two compared diagrams per pair |
| H7 | 2 | `relabelling.rs` | Does the test still catch a leg-keyed sign on the fixed tree? B′'s "before" run is evidence on the old convention only | a `−1` keyed to the vertex holding leg 0 | suspected: settle by running it | — (run below) |
| H8 | 2 | `root_diagram.rs:1338` (`named_processes_exercise_every_convention_factor`) | It calls each factor function directly, so it is blind to `convention_sign` dropping a factor from its product (that is the MadGraph rows' to see, per B′'s mutation table). Its own job is that each factor fires: to be shown by mutation of each factor's parts | each part of each factor set to `+1` | suspected: settle by running them | — (run below) |
| H9 | 1 | `root_diagram.rs:944` | `DiagramEval::fermi_sign` is documented "±1 from the diagram's Fermi permutation sign"; it is `Diagram::sign × convention_sign × live reversed parity` (`:1213`) | — | checked | fix: doc |
| H10 | 1, 3 | `root_diagram.rs:563` (`DiagramEvalTree::reversed_convention_sign`) | The name says convention sign; the value is the live tree's runtime reversed-bilinear parity, which `compile_single_diagram` multiplies in to *cancel*, while the convention's own parity comes from `vertex_root_signs`. `pub(super)`, and its only caller is `compile_single_diagram` in the same module (`git grep -n reversed_convention_sign` outside `root_diagram.rs` finds only doc links) | — | checked | fix: rename to `live_reversed_parity`, narrow to private, update the two doc links |
| H11 | 1 | `rooting_soundness.rs:5`, `:358` | "was originally validated only for feyngraph's `VtxIdx(0)` orientation" is history framing; "without the slow, currently-failing full sweep" is false: B′'s run of the sweep reads `270 re-rootings, 0 failures` | — | checked | fix: both comments |
| H12 | 2 | `rooting_soundness` (the brief's question) | What does it still pin? Claim (B′, and the module doc): the kernels' rooting invariance *and* the live-parity cancellation in `fermi_sign`. Settle with `fermi_sign` missing `* tree.reversed_convention_sign()` against the default-suite tests and the ignored sweep | dropping the live-parity cancellation | suspected: settle by running it | — (run below) |
| H13 | 4 | `root_lorentz.rs:415–486` (C-H's H13) | `ProjM` and `ProjP` arms of `build_child` are the same 15 lines with the chiralities swapped; those two, `Identity`, `Gamma5` and `Metric` each spell the same "the other index of a two-index op, or `unreachable!`" pick | — | checked | fix: one `partner_index` helper and one projector arm; identity by digest |
| H14 | 1 | `root_lorentz.rs:340` (`build_child`) | About 215 lines | — | checked | leave: one arm per `LorentzOp` variant, the reason the dispatch is one match; H13 shortens it |
| H15 | 3 | `root_diagram.rs:650` (`spine_sign_from_flow`), `:909` (`root_tree_at`) | `pub(super)`, no caller outside `root_diagram.rs` (`git grep` at `d6f2d41` and `5d4a346`). Not touched by the change | — | checked | Found (outside the change) |
| H16 | 1 | `gen_standalone_jamps.py:226` | A 104-character comment line from the change's rewrap; every other line of the file is ≤ 100 | — | checked | fix |
| H17 | 1 | `standalone_jamps.rs` row docs, `gen_standalone_jamps.py` row comments | They now describe rows by "holding leg 0", a property no convention reads any more | — | checked | leave: true as written, and it says why each row discriminates a labelling-keyed rule, the class the relabelling test guards |
| H18 | 2 | `amplitude_oracle.rs:435` (`CONFIG_AMP_SIGNS`) | 11 patterns re-banked as whole-pattern flips. The banked patterns now carry the free per-process global sign | — | checked | leave: a two-way tripwire is its stated purpose (B′'s "per-process global sign is free") |
| H19 | — | `SEVEN_DIGIT_COUPLING_ROWS` (`standalone_jamps.rs:268`) | The brief: replaced by a per-graph exemption on PR #33 | — | — | leave (per the brief) |

Comment grep over the added lines
(`git diff d6f2d41 5d4a346 -- vibegraph-lib validation/madgraph/gen_standalone_jamps.py | grep -nE '^\+.*(\bT[0-9]\b|\bC[0-9]\b|Phase [0-9]|note [0-9]+|used to|no longer|originally|anchor|W1|F1|session|sprint|previously|formerly|old )'`)
returns only the four "leg 0" row comments of H17. `git diff … | grep -nE '^\+.*\bpub(\(|\s)'`
is empty: the change adds no `pub` item.

Backlog check (path tails `/root_diagram.rs`, `/root_lorentz.rs`,
`/rooting_soundness.rs`, `/relabelling.rs`, `/renumbering.rs`,
`/diagram_eval.rs`, `/standalone_jamps.rs`, `/diagram.rs`, and the module
names):

- `one-diagram-diagramset-built-by-hand`: H3's site is an eighth it does not
  name; H3's fix removes it again.
- `comment-leftovers-after-hygiene-sprint`: its `root_diagram.rs` site, "note 16
  §6" (F-A Found 4), was in `mg_guard_processes_exercise_every_convention_channel`,
  which the change deleted. The code shows that site fixed.
- `unit-signs-flip-between-f64-and-i8`: `build_at_leg`'s tree path now returns a
  constant `1.0`; the `f64` sign survives on the tensor path
  (`build_tensor_term`) and its `if sign < 0.0` conversion in `root_term`. Still
  open, one site smaller.
- `mixed-dirac-and-matrix-free-vertex-terms`: its body says "An `Identity` term
  takes the scalar-sink −1 and a `Gamma` term does not, so a mixed vertex would
  most likely panic". The scalar-sink build sign is gone; a mixed vertex now
  fails `uniform_sign` on its reversed-bilinear parity only where a bilinear is
  reversed (B′'s H11 account), and the cited lines `diagram_eval.rs:131-169`,
  `:265` have moved. Stale, to Found.
- `tensor-uniformity-asserts-panic`: unchanged; `vertex_root_signs` reaches the
  same `VertexInfo::tensor` assert at every vertex, as the live tree did.
- `weighted-order-computed-three-ways`, `majorana-fermions-unsupported`,
  `diagram-enumeration-cost-unmeasured`: name `stitching.rs` or `diagram.rs`
  for other reasons; unaffected.

## Dispositions

| id | final | commit |
|---|---|---|
| H1 | fixed: `mixed_line_final_legs(diagram, model)` reads `Diagram::fermion_lines`; `external_leg` is the one leg-binding rule; `bake_node` and `external_bindings` both call them; `collect_fermion_pairs` and `input_slot` are deleted (−68 lines net in `root_diagram.rs`) | `8d21b42` |
| H2 | rejected: the shared part is a three-line step; the two walks return different things | — |
| H3 | fixed: `renumbering::single_diagram_amplitudes_at` takes the helicity list (`single_diagram_amplitudes` calls it with `None`, so the four other callers are unchanged); `diagram_values` deleted | `f983616` |
| H4 | fixed: `ray_leg_sets` reads `Diagram::legs_on_start_side` (now `pub(crate)`) | `f983616` |
| H5 | fixed: `relabelling` uses `renumbering::load`; the `Model` enum is the `(dir, card)` pair | `f983616` |
| H6 | fixed: a pair with fewer than two compared diagrams fails, naming the pair | `f983616` |
| H7 | rejected: sound. A leg-0-keyed `−1` fails three pairs (below) | — |
| H8 | rejected: sound. Each of the five parts set to `+1` fails it (below); the product blind spot is the MadGraph rows', as its doc says | — |
| H9 | fixed | `8d21b42` |
| H10 | fixed: `live_reversed_parity`, private; the links in `diagram_eval.rs` and `root_lorentz.rs` follow | `8d21b42` |
| H11 | fixed | `0d67c26` |
| H12 | rejected: the module tests what its doc says. Dropping the cancellation fails `literal_sigma_currents_are_rooting_invariant` (default suite) and 3 of 270 sweep re-rootings. Its name still fits: it is rooting soundness, and the convention sign is not its subject | — |
| H13 | fixed: `partner_index` and one `ProjM \| ProjP` arm (−26 lines) | `a9bb102` |
| H14 | rejected: one arm per op is the reason; now about 190 lines | — |
| H15 | Found | — |
| H16 | fixed | `5bb34ab` |
| H17–H19 | rejected (reasons in the table) | — |

## Mutations

Driven by `scratchpad/BH/mutate.py` (apply, run, restore; one log per
mutation). The commands are `cargo test -p vibegraph-lib --lib -- <filter>`.

| id | mutation | test | old test | new test |
|---|---|---|---|---|
| M1 | `relabelling.rs`: `VANISHING = 1.0` | `relabelling_preserves_per_diagram_amplitudes` | **passes** (`ok. 1 passed`) | fails: `w+ e- > z ve: 1 diagrams compared, too few to show a relative sign` |
| M2 | `convention_sign` × `−1` when leg 0's vertex has a fermion leg | same | fails: `w+ e- > z ve` flips `[1]`, `w+ a > e+ ve a` `[6]`, `st lt > st lt` `[1, 2]` | (same test logic; not rerun) |
| M3a | `propagator_phase_sign`: scalar count `× 0` | `named_processes_exercise_every_convention_factor` | fails (`root_diagram.rs:1353`, `b b~ > h h`) | unchanged test |
| M3b | crossed-line count `× 0` | same | fails | — |
| M3c | `vertex_root_signs` drops `reversed_sign` | same | fails | — |
| M3d | `vertex_root_signs` drops the tensor `build_sign` | same | fails | — |
| M3e | `triple_gluon_sign` always `+1` | same | fails | — |
| M4 | `fermi_sign` without `* tree.reversed_convention_sign()?` | `rooting_soundness` (default), `relabelling`, `renumbering`, the ignored sweep, the identity digest | `literal_sigma_currents_are_rooting_invariant` FAILED, the other four default `rooting_soundness` tests pass; relabelling and renumbering pass (`0 failures`); sweep `270 re-rootings, 3 failures across 1 processes` (`b b~ > c c~ e+ e- mu+ mu- QCD=0`); compiled digest `39280d4b…` against `bdca3693…` | — |

M4 also shows the identity digest has power: it changes on a sign-only
mutation, and its rooted half (which holds no `fermi_sign`) does not.

Sites no mutation reaches: H1's two copies agreed before the fix (the digest
is unchanged), so no test distinguished them; after the fix there is one copy
and nothing left to diverge. M2–M4 are on unchanged tests and measure their
power, not a fix.

## Identity check

Chosen before the first edit:

1. A temporary lib test (not committed) hashing with `sha2` the `Debug` of
   `compile_single_diagram(d, model, chain)` for every census subprocess,
   diagram and colour chain (the renumbering census: 3730 diagrams, 3774
   distinct chains), and of `root_tree_at(d, model, chain, v)` at every vertex
   (15 497 rootings).

   ```
   before (5d4a346): compiled bdca36932a38ea7bb48ec8498b13d192859b34ee220bc9254fd309aabdad0a40 rooted 50aef3ad0a2ee6c9307ccd365fa4e14a5625ea2787aea94f3f14bb9d4f450db8
   after:            compiled bdca36932a38ea7bb48ec8498b13d192859b34ee220bc9254fd309aabdad0a40 rooted 50aef3ad0a2ee6c9307ccd365fa4e14a5625ea2787aea94f3f14bb9d4f450db8
   ```

   `DiagramEval` and `DiagramEvalTree` are `Vec`-based, so the digest is
   deterministic; the program built from them is a function of these values.
2. `standalone_jamps` and `amplitude_oracle`, `--nocapture --test-threads=1`,
   sorted lines with ANSI codes and `finished in` stripped: `diff` empty for
   both (103 and 167 lines). `46 passed` and `49 passed`.
3. `relabelling` and `renumbering` printed lines: `diff` empty.

## Concepts rewritten

All `status: draft`, no `verified:`.

- `amplitudes/rooting-invariance-and-canonical-form` (renamed from
  `rooting-invariance-and-anchor`, whose title named the anchor): every sign
  reads the graph, the live parity is cancelled, so every rooting and labelling
  agrees; canonical form, container equality and the four invariance gates.
- `amplitudes/convention-sign-inventory`: `fermi_sign = Diagram::sign ×
  convention_sign × live parity`, each arm with what it reads and the rows that
  fail without it, the GF(2) derivation, the free per-process sign, and the
  unmeasured classes.
- `amplitudes/vector-vertex-signs`: no colourless vector vertex carries a sign,
  a VVS's scalar line takes its propagator phase, and the triple-gluon `−1` is
  calibrated and invisible to pure-gluon processes.
- `amplitudes/fermion-line-sign`: `(−1)^P` is the fermion propagator's phase;
  the bilinear reversal is `vertex_root_signs`' per-vertex factor; a crossed
  line's propagator phases sit in `propagator_phase_sign`. The falsified
  "WWγ takes no Yang–Mills sign" alternative is dropped (it is now the
  convention).
- `validation/convention-channel-coverage`: the firing test and its mutations,
  B′'s per-factor mutation table, the three invariance gates with what each
  catches (M2, M4), and what none sees.

Inbound links follow the rename in `B.md`, `W1.md`, `evaluator-architecture`,
`diagram-enumeration`, `decay-chains`, `diagram-rooting`,
`bit-exact-amplitude-debugging`, and the `replaced_by` of three archived
notes. The concepts cite code at `5d4a346` (pushed) but name
`live_reversed_parity`, this branch's name for `reversed_convention_sign`.

## Gates

Every cargo call with `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`, on the fixed tree.

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: `Finished`, exit 0.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
  `Finished`, exit 0.
- `cargo test --workspace --no-fail-fast`: exit 0; 38 suites, 1431 passed, 0
  failed, 16 ignored (summed `test result` lines), B′'s count: no test added or
  deleted.
- `cargo doc -p vibegraph-lib --no-deps --document-private-items`: 8 warnings,
  all in files this session does not touch (`coupling/cluster/configs.rs`,
  `phasespace/channel.rs`, `coupling/alphas.rs`, `coupling/scales.rs` ×2,
  `ufo/parameters.rs`, `lhef/mod.rs` ×2).
- Banked targets: `standalone_jamps` (46 passed) and `amplitude_oracle`
  (49 passed), run for the identity check above.
- `pixi run -e kb kb-lint`: `0 errors, 2 warnings` (two blocked backlog items,
  not this session's).
- `pixi run --skip-deps validate` (after the sibling F2's run ended): `VALIDATE
  EXIT 0`, 63 suites, 1816 passed, 0 failed, 84 ignored; `64 rows × 4
  categories = 256 cells: 203 measured in the layers this run drove (196 ✅, 7
  ⚠️, 13 ⏳, 40 — / uncovered)` and `the measured cells are the declared cells`.
  These are B′'s counts.

## Method

- Reading B′'s diff against the helpers the new code sits beside, and
  grepping each new function's body lines workspace-wide: 35%. Found H1, H3,
  H4, H5 (all second copies the change added).
- Running mutations on the tests the change added or touched: 25%. Settled
  H6, H7, H8, H12 (one fix, three sound).
- Comment and `pub` greps over the added lines, visibility of touched items:
  10%. H9, H10, H11, H15.
- Identity oracle and gates: 20%. Concept rewrite: the rest of the session.
- Leads that did not hold: the `ProjM`/`ProjP` refactor (H13) needed no
  physics; `line_end` against `fermion_lines` (H2) shared too little to merge;
  a missing two-way check between the committed standalone tables and the
  `check()` calls (45 tables, 45 checks, `diff` empty, so nothing is lost, but
  nothing enforces it either); the row comments' "leg 0" framing (H17).
- One process slip: a `git mv` staged before the first code commit went into
  it. The five unpushed commits were undone with `git reset --soft` and redone
  with path-limited `git commit -- <paths>`; no work was touched.

## Found

1. **`spine_sign_from_flow` and `root_tree_at` are `pub(super)` with no caller
   outside `root_diagram.rs`** (H15; `git grep` at `5d4a346`). To close: make
   them private.
2. **Seven kb concepts outside this brief still describe the anchor or the
   removed sign functions**, one entry each:
   - `amplitudes/evaluator-architecture.md:74`: the second tree at
     `Diagram::anchor`.
   - `amplitudes/four-fermion-vertices.md:124-152`: `fermion_current_line_sign`
     and "the anchor rooting"; the tensor-path sign is now
     `vertex_root_signs`' factor.
   - `amplitudes/fermion-flow-and-crossing.md:82`: `pair_crossed`,
     `standalone_projector_crossed`.
   - `amplitudes/global-phase-i-counting.md:98`: `yang_mills_vvv_sign`.
   - `amplitudes/charge-flow-phase-study.md:100-143`: `yang_mills_vvv_sign`,
     `vector_contact_sign`, "contact at the anchor", `channel_counts`.
   - `validation/bit-exact-amplitude-debugging.md:92-160`: the three
     vector-vertex sign functions "keyed to `Diagram::anchor`".
   - `performance/diagram-rooting.md:4,56,128`: "soundness rests on anchor-read
     convention signs".

   To close: describe `convention_sign` there, or link the rewritten concepts.
3. **`mixed-dirac-and-matrix-free-vertex-terms` is stale**: its body says an
   `Identity` term "takes the scalar-sink −1"; that build sign is gone, and the
   cited lines `diagram_eval.rs:131-169`, `:265` have moved. To close: restate
   the open question as B′'s H11 account (a mixed vertex has a sign only where
   no bilinear is read reversed) and cite `uniform_sign` by name.
4. **`comment-leftovers-after-hygiene-sprint`: its `root_diagram.rs` site is
   fixed** (the "note 16 §6" comment went with the deleted guard test). To
   close: drop that site from the item.
5. **`one-diagram-diagramset-built-by-hand`**: the change added an eighth site
   (`relabelling.rs:241`) and this session removed it again; the item's seven
   sites stand, and `single_diagram_amplitudes_at` is the natural home for the
   helper the item asks for.
6. **`unit-signs-flip-between-f64-and-i8` is one site smaller**: the tree path
   of `build_at_leg` returns a constant `1.0`; only the tensor path's `f64` and
   `root_term`'s `if sign < 0.0` remain.
7. **The live-parity cancellation is thinly pinned**: dropping it fails one
   default-suite test (`literal_sigma_currents_are_rooting_invariant`) and 3 of
   270 sweep re-rootings, all on one process; the amplitude oracles (not run on
   M4) are the other line of defence. To close, if wanted: a default-suite
   test that compiles a few diagrams at two rootings where the live parity
   differs and compares `fermi_sign × kernel`.
8. **No check ties the committed standalone tables to the `check()` calls**
   (45 each today, by hand). To close: a test that lists
   `validation/madgraph/standalone/*.json` and requires a named test per key,
   or `gen_standalone_jamps.py`'s `ROWS` both ways.

## Brief corrections

- **C-H's report is not on this branch.** Its H13 is in `29da327` on
  `amp-signs/identity-derivative-scalar-signs`
  (`sessions/C-H-report.md:37`), which `5d4a346` does not contain.
- **`main` in the worktree is stale** (`cc8b12c`); `git diff main...HEAD`
  carries the merged W1, F1 and C2 work too. B′'s own change is
  `d6f2d41..5d4a346` on the code paths.
- **"`rooting_soundness` now checks only kernels"** undersells it: it also
  pins the cancellation of the live reversed parity (M4), as B′'s report says.
- **The rename touches three archived notes** (`replaced_by` only), which the
  brief's "update the inbound links" covers but the archive's "unchanged" rule
  does not mention; kb-lint does not check `replaced_by`.
