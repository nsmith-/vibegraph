---
type: Session Report
title: "C-H report: hygiene review of the Identity and derivative-scalar sign fixes"
description: "Hygiene review of PR #32's C1 and C2 change: findings written before any edit, then the local ones fixed with run mutations and an identity check on standalone_jamps' and amplitude_oracle's printed numbers."
status: draft
tags: [hygiene, review, amplitude, sign-convention, scalar, toy-ufo, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: 8938adf, resource: "https://github.com/nsmith-/vibegraph/commit/8938adf", title: "kb: C-H brief and findings"}
  - {id: d53bd49, resource: "https://github.com/nsmith-/vibegraph/commit/d53bd49", title: "docs(amplitudes): name every rooting-sign source and the mixed-vertex refusal"}
  - {id: a8c53d6, resource: "https://github.com/nsmith-/vibegraph/commit/a8c53d6", title: "validation: SSSD's comment names the operator SMEFTsim's SSS2 carries"}
  - {id: 9bb7f88, resource: "https://github.com/nsmith-/vibegraph/commit/9bb7f88", title: "test(amplitudes): pin the mixed-sign refusal through compile; toy copies in tempdirs"}
---
Review of the C1 and C2 change on `amp-signs/identity-derivative-scalar-signs`
at `5c0957f`: `fea4aef` (the standalone rows, the toy-UFO cubic scalar, the
`vg_cHbox` card, the `"ufo"` table model), `4e8b614` (the two sign arms in
`root_lorentz.rs`, `uniform_sign` and the mixed-sign refusal in
`diagram_eval.rs`, `root_diagram.rs`, the `toy_models.rs` split-sum test) and
`06acd5a` (kb). Line numbers are at `5c0957f`.

## Findings (written before any edit)

| id | pt | site | evidence | mutation the test misses | conf. | disposition |
|---|---|---|---|---|---|---|
| H1 | 1 | `helas/eval/root_lorentz.rs:881` | `standalone_projector_crossed`'s doc opens with two lines that describe `pair_crossed` ("True iff a fermion pair's plain legs sit on a crossed line … Summed (negative) indices carry no binding and are skipped"), and `pair_crossed` (`:1043`) has no doc. The rest of the doc describes "a *standalone* chiral projector", while the new `Identity` arm (`:484`) and the `Gamma5` arm call it and cite it as their rule | — | checked | fix: move the two lines to `pair_crossed`; the doc names every standalone bilinear that uses the predicate |
| H2 | 1 | `root_lorentz.rs:64`, `root_diagram.rs:555`, `root_lorentz.rs:609` | `RootedTerm::build_sign` and `DiagramEvalTree::build_convention_sign` list the sign's sources as "(VVS `pure_metric`, FFS scalar-sink, crossed-pair)", and `build_at_leg` names only the crossed bilinear. The vertex-shape −1 of `vertex_contact_sign` (all-scalar vertices, which this change extends to derivative structures, and four-vector contacts) starts `sign` at `build_at_leg`'s `:638` and lands in `build_sign`; the new unit test asserts a plain all-scalar term's `build_sign == -1` | — | checked | fix: name it in the three lists |
| H3 | 1 | `helas/eval/diagram_eval.rs:285` | `VertexInfo::reversed_sign`'s doc does not say a mixed vertex is refused; `VertexInfo::build_sign`'s does, and both return the same `UnsupportedVertex` | — | checked | fix |
| H4 | 1 | `validation/ufo/vibegraph_toy_UFO/lorentz.py:39` | "the shape of SMEFTsim's `SSS2`, the (dH)^2 piece of O_Hbox and O_HDD". In the vendored SMEFTsim the `H H H` vertex is `{(0,0): GC_336, (0,2): GC_345, (0,1): GC_346}` over `[SSS1, SSS2, SSS3]`; `GC_346 = -(cHDD·i·vevhat)/Λ²` multiplies `SSS2` and `GC_345 = (-3·cHbox·i·vevhat)/Λ²` multiplies `SSS3`. `SSS2` carries O_HDD alone (C1's own row comment in `gen_standalone_jamps.py` says so) | — | checked | fix the comment |
| H5 | 1 | `vibegraph_toy_UFO/vertices.py:5` | The docstring line the change extended is 112 characters, against the file's ~80 | — | checked | fix: reflow |
| H6 | 1 | `tests/standalone_jamps.rs:474` | `KNOWN_DISAGREEMENT`'s continuation lines kept the extra indent of C1's eight-entry list after C2 shrank it back to one. A `\` continuation strips leading whitespace, so the string is unchanged | — | checked | fix: indent as on `main` |
| H7 | 1 | `tests/toy_models.rs:284` | `toy_point` draws RAMBO's uniforms from a hand-written golden-ratio Weyl sequence, `((k + 1 + 7·seed)·0.618…).fract()`, which `grep -rn '\.fract())'` finds nowhere else. AGENTS.md: no hand-written RNG. The library's `rambo_massive(sqrt_s, masses, &mut rng)` with a seeded `StdRng` is what `decay_chain_ladder.rs:166` uses, and `rand` is a dependency | — | checked | fix |
| H8 | 1, 4 | `tests/toy_models.rs:330–386` | Each case hand-writes its legs' helicity states and the test builds their Cartesian product itself; `AmplitudeEvaluator::helicities()` (`compile.rs:254`) is that product, built from the model's spins by `cartesian_helicity_product`. A hand list that dropped a state would shrink the comparison silently. The test is 108 lines | a `states` entry `&[-1]` in place of `&[-1, 1]` passes | checked | fix: read `helicities()` and assert the split and merged lists agree |
| H9 | 2 | `helas/eval/root_diagram.rs:571`, `:594`, `:1262` | The refusal is pinned only by the unit test on `VertexInfo` directly. Nothing compiles a process whose vertex is refused, so the path `build_convention_sign` → `compile_single_diagram` → `AmplitudeEvaluator::compile` is unpinned (C2's "Blind spots" says the refusal has no real-model trigger left). A probe on a copy of `vibegraph_toy_UFO` with `FFS5 = P(-1,3)*Gamma(-1,2,1)` merged into `NP` beside `FFS1 = Identity(2,1)` is refused through `compile` for `lt~ lt > st`, `lt~ lt > st st`, `st lt > st lt`, `lt~ lt > st vt`, `qt qt~ > st st` and `lt~ lt > qt qt~ st` | `sign *= info.build_sign().unwrap_or(1)` at `:571`: `--lib helas::eval` 170 passed, `toy_models` 5 passed, `standalone_jamps` 25 passed | checked | fix: a hermetic compile-level test |
| H10 | 1 | `diagram_eval.rs:300` (`uniform_sign`) | The brief's question. The message names the vertex only by its rooted rendering and internal coupling ids: `unsupported vertex: the structures of \`CouplingId(5)*(1*ScalarProduct(IdentityAmp(Leg(1),Leg(0)),Leg(2))) + CouplingId(7)*(1*ScalarProduct(Metric(P2,GammaVout(Leg(1),Leg(0))),Leg(2)))\` carry mixed rooting-convention signs, …` (the H9 probe). It names neither the UFO vertex (`V_ltltst`), its Lorentz structures (`FFS1`, `FFS5`), its couplings (`GC_LS`, `GC_LP`) nor their orders, and the library adds no process: the `compile '…'` prefix C2 quotes is the test's own. `reject_cyclic_structure` (`root_lorentz.rs:1675`) does name its structure (`Lorentz structure {name} ({structure})`). `VertexInfo` carries no UFO identity; giving it one touches its 12 constructions (`diagram_eval.rs`, `run.rs`) | — | checked | Found |
| H11 | 2 | the H9 probe | The refusal depends on the rooting. The same mixed vertex compiles for `st > lt lt~` (one diagram, the scalar on leg 0, so the vertex is the amplitude root with no crossed pair), and is refused for `lt~ lt > st`. A mixed `Identity` + `P·Gamma` vertex can therefore compile with a sign no oracle has checked. Whether that sign is right is physics | — | checked | routed: validation-dev |
| H12 | 4 | `gen_standalone_jamps.py:95`, `tests/common/manifest.rs:160`, `build.sh:99` | Three copies of "where is `restrict_<name>.dat`": `Row.ufo_paths` (new, authored card under `cards/` first), `RowModel::restrict_card` (the model's own first), and `build.sh`'s `model_card_dir` plus copy order (authored cards copied over the model's, so authored wins in MadGraph). Rust disagrees with MadGraph on precedence; no name collides today (checked over every card under `validation/madgraph/cards/`) | — | checked | Found |
| H13 | 4 | `root_lorentz.rs:426–520` | The four standalone-bilinear arms of `build_child` (`ProjM`, `ProjP`, `Identity`, `Gamma5`) each repeat the wrapped-index pick and the `standalone_projector_crossed` flip; the change adds the fourth. `build_child` is 250 lines (239 on `main`), one arm per op | — | checked | leave: the brief defers restructuring these role-sign functions to the follow-up session; named in Found for it |
| H14 | 1 | `research/kb/backlog/feature/mixed-dirac-and-matrix-free-vertex-terms.md` | Says the uniformity assertions at `diagram_eval.rs:131-169` and `:265` would "most likely panic" on an `Identity` + `Gamma` vertex. The sign ones now return `UnsupportedVertex` (H9's probe shows the refusal, not a panic), the item's second resolution ("refuse it with a typed error") now exists for some rootings only (H11), and the line numbers have moved | — | checked | Found |
| H15 | 1 | `tests/toy_models.rs:11` | The module doc says the file asks "the same three questions `smeftsim.rs` asks"; the new split-sum test is a fourth kind of check the doc does not mention | — | checked | fix: one sentence |
| H16 | 4 | `tests/toy_models.rs:274`, `reweight/engine.rs:954` | Two hand copies of the CM beam momenta at `√s` for two masses, beside `phasespace::beams::beam_momenta` (`pub(crate)`, so an integration test cannot call it; `reweight/engine.rs` is in the crate and could) | — | checked | Found |
| H17 | 2 | `diagram_eval.rs:346` (`mixed_vertex_signs_are_refused`) | Matches the variant and the `what` word of the path under test, across couplings and within one coupling. Mutations M1, M2 below | — | checked | leave: sound |
| H18 | 2 | `toy_models.rs:303` (split-sum test) | Bound `1e-12` of max|A| against a measured ≤7.9e-16; blind to a sign both structures share, which its doc says. Mutations M3, M4 below | — | checked | leave: sound |
| H19 | 3 | the diff | `git diff origin/main...HEAD -- '*.rs' \| grep '^+.*pub '` returns only `pub(super)`/`pub(crate)` signatures that were already that visibility; `uniform_sign`, `table_model`, `toy_point` and `merged_order_toy_model` are private | — | checked | leave |
| H20 | 1 | the diff | Comment grep over added lines (`\bT[0-9]\b\|\bC[0-9]\b\|Phase [0-9]\|note [0-9]+\|used to\|no longer\|session`) returns one line, the `assert_ne!` message "the toy couplings no longer carry NPCP/NPD", which describes a future edit of the source, not this code's history | — | checked | leave |

**Backlog check.** Items naming a reviewed path (path tails `/root_lorentz.rs`,
`/diagram_eval.rs`, `/root_diagram.rs`, `/toy_models.rs`,
`/standalone_jamps.rs`, `gen_standalone_jamps.py`, `vibegraph_toy_UFO`):
`majorana-fermions-unsupported` (its `root_lorentz.rs:580` citation of the `C`
refusal is now `:591`, a drift only), `mixed-dirac-and-matrix-free-vertex-terms`
(H14), `tensor-uniformity-asserts-panic` (the brief's point; still accurate:
both `tensor()` asserts remain at `diagram_eval.rs:149` and `:280`),
`one-diagram-diagramset-built-by-hand`,
`standalone-generator-duplication-and-coefficient-format`,
`standalone-table-without-test-is-never-run`,
`standalone-spacelike-width-convention-unbanked`, and the two filed from C1/C2
(`mixed-structure-vertex-unoracled`, `derivative-scalar-asymmetric-unoracled`).
None of their sites is wrong or fixed by this change except H14.

**Scope.** `git diff origin/main...HEAD` also carries the Q1 branch (the
same-flavour four-fermion rows and Q-H's fixes), merged in by C1 and already
reviewed by Q-H. This review covers C1 and C2 only.

**Found while fixing** (after the findings commit, so outside the hit rate):

| id | pt | site | evidence | conf. | disposition |
|---|---|---|---|---|---|
| H21 | 1 | `tests/toy_models.rs:252` | `merged_order_toy_model` writes its copy into a fixed-name directory under `CARGO_TARGET_TMPDIR` and only overwrites, so a file removed from `vibegraph_toy_UFO` would survive in the copy from an earlier run and still be loaded. The filed item `scratch-dirs-hand-rolled` (AGENTS.md: no hand-written temp directories) does not list this site; `tempfile` is already in the lock file through the CLI's tests | checked | fixed |

## Dispositions

| id | disposition | where |
|---|---|---|
| H1, H2, H3 | fixed | `d53bd49` |
| H4, H5, H6 | fixed | `a8c53d6` |
| H7, H8, H9, H15, H21 | fixed | `9bb7f88` |
| H10, H12, H13, H14, H16 | filed (Found below) | — |
| H11 | routed to the manager for validation-dev (physics) | Found 2 |
| H17, H18, H19, H20 | rejected: sound on re-reading, with the mutations below | — |

What changed:
- **H1–H3** (doc comments only). `standalone_projector_crossed`'s doc names the
  four standalone bilinears whose arms apply it and the `FFS2` row that pins the
  `Identity` case; its stray first two lines are now `pair_crossed`'s doc. The
  sign-source lists on `RootedTerm::build_sign`,
  `DiagramEvalTree::build_convention_sign` and `build_at_leg` name
  `vertex_contact_sign`'s all-scalar and four-vector contact sign, and
  `build_at_leg` names `γ⁵` beside `1` and `P_χ`. `build_convention_sign` and
  `VertexInfo::reversed_sign` say that a mixed vertex is refused.
- **H4–H6** (comments and whitespace). `SSSD`'s comment says SMEFTsim's `SSS2`
  carries O_HDD (O_Hbox goes on `SSS3`); the `vertices.py` docstring is
  reflowed; `KNOWN_DISAGREEMENT`'s continuation lines are indented as on `main`.
- **H7, H8, H15, H21** (`toy_models.rs`). `edited_toy_model(edits)` replaces
  `merged_order_toy_model`: it copies the toy UFO into a `tempfile` directory and
  requires every replacement's pattern to be present, where the old
  `assert_ne!` passed if either of the two did. `toy_point` draws RAMBO through
  `rambo_massive` with a seeded `ChaCha8Rng`. The split-sum test sums over
  `AmplitudeEvaluator::helicities()` and asserts the split and merged lists are
  equal, in place of a hand-written state list per case; it is 92 lines (108
  before). The module doc names the two copy-based checks. `vibegraph-lib`
  gains the dev-dependency `tempfile = "3"`, the version the CLI's tests use
  (`Cargo.lock` gains the one dependency line).
- **H9** (`toy_models.rs::mixed_sign_vertex_is_refused_at_compile`, new). It
  compiles `lt~ lt > st NP<=1` in a toy copy whose `FFS5` is
  `P(-1,3)*Gamma(-1,2,1)` under `NP`, beside `FFS1 = Identity(2,1)`, and asserts
  that `AmplitudeEvaluator::compile` fails with `unsupported vertex: …` and
  `carry mixed rooting-convention signs`. `EvalError` lives in the private
  `helas::eval::error` module and is not re-exported, so an integration test
  cannot match the variant; the message is the most specific handle it has (see
  Found 6). Its doc says that the same vertex compiles at the other rooting
  (H11), so it pins the refusal's path, not where a sign exists.

## Mutations

Each was applied by a script that edits, runs, logs and restores the file
(`CH-mutate.py` in the session scratchpad), on the tree named in the row.

| id | mutation | tree, test | result |
|---|---|---|---|
| MA | `root_diagram.rs:571` `sign *= info.build_sign()?` → `.unwrap_or(1)` | before the fixes: `--lib helas::eval`, `toy_models`, `standalone_jamps` | **passes**: 170, 5 and 25 passed (H9's evidence) |
| MA | same | after: `toy_models` | **fails**: `mixed_sign_vertex_is_refused_at_compile`, "was refused for another reason: … carry mixed reversed-bilinear parities" (this vertex mixes both; the test asserts *which* refusal fired) |
| MA2 | MA plus the same on `reversed_sign()?` (`:594`) | after: `toy_models` | **fails**: "'lt~ lt > st NP<=1' compiled a vertex with mixed rooting-convention signs" |
| M1 | `uniform_sign`: `if it.all(…) \|\| true` (accepts a mismatch) | before: `--lib helas::eval::diagram_eval` | **fails**: `mixed_vertex_signs_are_refused` at `diagram_eval.rs:390` |
| M1 | same | after: `toy_models` | **fails**: `mixed_sign_vertex_is_refused_at_compile`, "compiled a vertex with mixed rooting-convention signs" |
| M2 | `VertexTerm::build_sign` reads `terms.iter().take(1)` | before: `diagram_eval` unit test | **fails** at `:390`, the within-one-coupling case (the across-couplings case still refuses at the `VertexInfo` level) |
| M3 | the `Identity` arm's flip under `if false && …` | before: `standalone_jamps`, `toy_models` | **fails**: `ee_to_ttxh_identity`, `ll_to_qqxs_toy_yukawa` (23 passed, 2 failed); split-sum test refused at compile |
| M3 | same | after: `toy_models` | **fails**: the split-sum test, at its `compile` (`toy_models.rs:371`) |
| M4 | the all-scalar sign restricted to `term.ops.is_empty()` (C2's base) | before: `standalone_jamps`, `toy_models` | **fails**: the five derivative rows (20 passed, 5 failed); split-sum test refused at compile |
| M4 | same | after: `toy_models` | **fails**: the split-sum test, at its `compile` |
| MH8 | the old split-sum test with `lt~`'s states `&[-1, 1]` → `&[-1]` | before: `toy_models mixed_structure` | **passes** (1 passed): a dropped state shrinks the comparison silently. After the fix there is no hand list to mutate |

Sites no mutation reaches:
- The split-sum test's value comparison. Every sign mutation tried that leaves
  both structures' signs equal (M3 and M4 move one structure, so they refuse
  instead) passes it, as its doc says. A sign shared by both structures is the
  standalone rows' to see.
- H7's RNG swap and H21's scratch directory change no assertion's power; the
  printed split-sum ratios moved from 7.9e-16/3.9e-16/7.5e-16 to
  6.5e-16/6.2e-16/8.5e-16 with the new points.
- The refusal at rootings where the mixed vertex compiles (H11): no test can
  pin a refusal that does not fire.

## Identity check

Production changes are doc comments only, so no printed number may move. The
oracle, chosen before the first edit: the `--nocapture --test-threads=1` output
of `standalone_jamps` and `amplitude_oracle` at `5c0957f`
(`CH-baseline.log`) against the same run on the fixed tree, with ANSI codes,
`Running` lines and `finished in` timings removed.

```
$ sect() { awk '/Running tests\/(standalone_jamps|amplitude_oracle)/{on=1} /Running tests\/toy_models/{on=0} on' "$1" \
    | sed 's/\x1b\[[0-9;]*m//g' | grep -v -E "Running |finished in" | sort; }
$ diff <(sect CH-baseline.log) <(sect CH-gate-identity.log) && echo IDENTICAL
IDENTICAL
```

235 lines on each side, 33 of them `worst …` measurements: every row's `G`,
worst flow, worst |M|² and worst graph, and all 49 `amplitude_oracle` lines.
For example `ee_to_ttxh_identity: 240 graph entries, worst graph 4.119e-15` and
`hh_to_hh_chbox: 256 graph entries, worst graph 1.810e-15`, C2's numbers.
`toy_models` is outside the oracle: H7 changes its points on purpose.

## Gates

All on the committed tree `9bb7f88`, with `CARGO_BUILD_JOBS=2
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.

- `cargo fmt --all --check`: exit 0, no output.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0,
  `Finished \`dev\` profile [optimized] target(s) in 21.06s`.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: exit
  0, `Finished \`dev\` profile [optimized] target(s) in 27.52s`.
- `cargo test --workspace --no-fail-fast`: exit 0, 38 suites, 1413 passed, 0
  failed, 16 ignored (summed `test result` lines). The one test added is
  `toy_models::mixed_sign_vertex_is_refused_at_compile` (`toy_models` 5 → 6
  passed); no test was deleted, and no other test file changed beyond
  `standalone_jamps.rs`'s whitespace (25 passed before and after).
- `cargo doc -p vibegraph-lib --no-deps --document-private-items`: exit 0, `generated 8
  warnings`, none in a file this session touched (`coupling/cluster/configs.rs`,
  `phasespace/channel.rs`, `coupling/alphas.rs`, `coupling/scales.rs` ×2,
  `ufo/parameters.rs`, `lhef/mod.rs` ×2: pre-existing, the class
  `rustdoc-warnings-ungated` tracks). Only `vibegraph-lib` was touched.
- `pixi run --skip-deps validate`: `VALIDATE EXIT 0`, 63 suites, 1798 passed, 0
  failed, 84 ignored; `64 rows × 4 categories = 256 cells: 203 measured in the
  layers this run drove (196 ✅, 7 ⚠️, 13 ⏳, 40 — / uncovered)`, and `the measured
  cells are the declared cells`. The cell line is C2's.

The production changes are doc comments, so no banked target's inputs move and
none was run on its own; `validate` drives the whole banked layer.

## Method

| technique | share | yield |
|---|---|---|
| Reading each changed item's doc against its body, and the docs of the items the change newly calls (`standalone_projector_crossed`, `vertex_contact_sign`'s readers) | 20% | H1, H2, H3, H15 |
| Checking comments' physics claims against the vendored source (the `H H H` vertex's coupling map) | 5% | H4 |
| Grep of the new test helpers' bodies workspace-wide (`.fract())`, `beam_momenta`, `cartesian_helicity_product`, `card_dir`) | 15% | H7, H8, H12, H16 |
| A throwaway probe test on an edited toy copy, to find a real trigger for the refusal | 15% | H9, H10, H11 |
| Run mutations (MA before writing H9; M1–M4 on the existing tests; the same on the new ones) through an apply-run-revert script | 30% | H9 settled, H17 and H18 confirmed sound |
| Backlog check by path tail | 10% | H14; `scratch-dirs-hand-rolled` led to H21 while fixing |
| Comment and `pub` greps over the added lines | 5% | nothing (H19, H20) |

Leads that did not hold up: the `assert_ne!` "no longer" message (a
diagnostic, not history); merging the four standalone-bilinear arms (real, but
the brief reserves that restructuring for the follow-up); the split-sum test's
tolerance (linear, relative, 1000× above its measured residual).

## Routed

- **H11, to validation-dev (physics).** A vertex whose structures disagree on
  the rooting-convention sign is refused only at rootings where the disagreement
  shows on the anchor tree. The H9 probe vertex (`Identity(2,1)` beside
  `P(-1,3)*Gamma(-1,2,1)` under one order) is refused for `lt~ lt > st`,
  `lt~ lt > st st`, `st lt > st lt`, `lt~ lt > st vt`, `qt qt~ > st st` and
  `lt~ lt > qt qt~ st`, and compiles for `st > lt lt~` (one diagram). Whether
  the sign it compiles with is right needs a MadGraph row, which is what
  `mixed-dirac-and-matrix-free-vertex-terms` asks for.

## Found

1. **The mixed-sign refusal does not name the vertex a user would recognise
   (H10, the brief's question).** It prints the rooted rendering with internal
   ids, `the structures of \`CouplingId(5)*(1*ScalarProduct(IdentityAmp(Leg(1),Leg(0)),Leg(2))) + CouplingId(7)*(…)\``,
   and no process. A user with an arbitrary UFO cannot map `CouplingId(5)` to
   `GC_LS`, nor the rendering to `V_ltltst`'s `FFS1` and `FFS5`.
   `reject_cyclic_structure` names its structure (`Lorentz structure {name}
   ({structure})`). To close it: carry the UFO vertex's id on `VertexInfo` (12
   constructions in `diagram_eval.rs` and `run.rs`) so the message names the
   vertex, its Lorentz structures and their couplings with orders, and have
   `AmplitudeEvaluator::compile` add the subprocess it already formats for its
   tracing span. The follow-up session that restructures these sign functions
   is the natural owner.
2. **A mixed-sign vertex compiles at some rootings (H11).** Routed above;
   evidence there.
3. **`restrict_<name>.dat` is located three ways, and Rust disagrees with
   MadGraph on precedence (H12).** `gen_standalone_jamps.py:95`
   `Row.ufo_paths` and `build.sh:99` `model_card_dir` (with its copy order)
   prefer the authored card under `validation/madgraph/cards/<family>/`;
   `tests/common/manifest.rs:160` `RowModel::restrict_card` prefers the model's
   own. No name collides today. To close it: one rule, stated once (for example
   the manifest records the card path, as the `"ufo"` standalone tables do), and
   a check that no card name exists in both places.
4. **`mixed-dirac-and-matrix-free-vertex-terms` is stale (H14).** Its "would
   most likely panic on the first assertion" is now a typed refusal at the
   rootings H11 lists, and a compile at `st > lt lt~`; its
   `diagram_eval.rs:131-169`, `:265` sites have moved (`uniform_sign` is at
   `:300`, the remaining `tensor()` asserts at `:149` and `:280`). The item
   should say which half of its "refuse it with a typed error" option exists,
   and cite `toy_models.rs::mixed_sign_vertex_is_refused_at_compile`.
5. **The CM beam momenta are hand-copied (H16).** `tests/toy_models.rs`
   `toy_point` and `reweight/engine.rs:954` `points` compute
   `e1 = (s + m1² − m2²)/(2√s)` and its momentum beside
   `phasespace::beams::beam_momenta`, which is `pub(crate)`. To close it: the
   in-crate copy calls `beam_momenta`; exposing it (or a test helper in
   `tests/common`) serves the integration test. A site for
   `kinematic-helpers-hand-rolled`.
6. **`AmplitudeEvaluator::compile`'s error type cannot be named outside the
   crate.** It returns `EvalError`, a `pub enum` in the private
   `helas::eval::error` module with no re-export, as are `CompileError` and
   `RootLorentzError`. An integration test can only match the message, which is
   what the new refusal test does. To close it: re-export the error tree from
   `helas::eval`, then match the variant.
7. **For the follow-up restructuring (H13).** `build_child`'s `ProjM`, `ProjP`,
   `Identity` and `Gamma5` arms (`root_lorentz.rs:426–520`) each repeat the
   wrapped-index pick and the `standalone_projector_crossed` flip.
8. **Backlog drift, one line.** `majorana-fermions-unsupported` cites
   `root_lorentz.rs:580` for the `C` refusal, now `:591`.

## Brief corrections

- The brief's diff, `git diff origin/main...HEAD`, also carries the Q1 branch
  merged in by C1 (Q-H reviewed it); the C1 and C2 commits are `fea4aef`,
  `4e8b614` and `06acd5a`. Local `main` in the shared checkout is stale
  (`cc8b12c`); `origin/main` (`68e7f95`) is the merge base.
- The brief asks whether the refusal names "vertex, structures, coupling
  order": it names none of them in UFO terms (Found 1).
- The brief lists `root_diagram.rs` as reviewed code; its change is the
  `Result` propagation only, and its one doc fix is H2.
