---
type: Session Report
title: "C2 report: the Identity crossed-line and derivative all-scalar signs land, and the seven rows gate"
description: "The Identity arm takes the crossed-line −1 and every all-scalar vertex takes the scalar-sink −1. The seven informational standalone rows now gate per MadGraph graph at ≤1.8e-14. A mixed-sign vertex is refused with an error instead of panicking, and a merged-order toy vertex equals its split sum to 8e-16. All 49 amplitude_oracle lines and every banked cell are unchanged."
status: draft
tags: [amplitude, sign-convention, fermion-flow, scalar, smeftsim, toy-ufo, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-15}]
sources:
  - {id: fix, resource: "https://github.com/nsmith-/vibegraph/commit/4e8b614", title: "fix(amplitudes): crossed-line sign for Identity currents and the all-scalar sign for derivative vertices"}
---
Items: `standalone-identity-crossed-line-sign-unpinned`,
`derivative-scalar-contact-sign-unoracled`. The diagnosis is in the
[C1 report](C1-report.md). Built on `cb49b40`.

## What was done

- **Identity arm** (`build_child`, `root_lorentz.rs`). It now takes the
  `standalone_projector_crossed` −1, like `ProjM`, `ProjP` and `Gamma5`. The
  comment gives the reason (`C·1ᵀ·C⁻¹ = 1`) and names the pinning rows.
- **All-scalar arm** (`vertex_contact_sign`). The `term.ops.is_empty()`
  restriction is gone, so a term carrying `P` takes the scalar-sink −1 too. The
  comment now says the −1 belongs to the vertex, not to its operators. That left
  the `term` parameter unused, so I removed it. The function has one caller.
- **Refusal instead of panic** (`diagram_eval.rs`). The four mixed-sign
  `assert!`s are in `VertexTerm::{build_sign, reversed_sign}` and
  `VertexInfo::{build_sign, reversed_sign}`. Each now returns
  `Result<i8, RootLorentzError>`, through one helper, `uniform_sign`. A mismatch
  is `RootLorentzError::UnsupportedVertex`, and the message names the vertex.
  `DiagramEvalTree::{build_convention_sign, reversed_convention_sign}` propagate
  it, and `compile_single_diagram` returns it as `CompileError::RootVertex`.
  The refusal fires exactly where the asserts were evaluated: on the anchor tree
  for the build sign, and on both trees for the reversed parity. So nothing that
  compiled before is refused now. The `tensor()` uniformity assert is not a sign,
  and I left it alone (see Found).
- **Tests.**
  - `standalone_jamps.rs`: I removed the seven rows from `KNOWN_DISAGREEMENT` and
    rewrote their doc comments to say what they now gate.
  - `toy_models.rs::mixed_structure_vertices_equal_their_split_sum` (hermetic,
    new). It copies `vibegraph_toy_UFO` into `CARGO_TARGET_TMPDIR`. In the copy,
    `GC_LP`/`GC_QP` (`NPCP`) and `GC_S3D` (`NPD`) move to `NP`. That makes
    `V_ltltst`/`V_qtqtst` (`Identity` + `Gamma5`) and `V_ststst` (`SSS1` +
    `SSSD`) single vertices. Three processes are compiled under `restrict_all`:
    `lt~ lt > qt qt~ st`, `st lt > st lt` and `lt~ lt > st st`. The test asserts
    that the merged process has fewer diagrams, and that every helicity
    amplitude equals the split model's at three massive-RAMBO points, to 1e-12
    of max|A|.
  - `diagram_eval.rs::tests::mixed_vertex_signs_are_refused` (unit, new). Mixed
    build signs and mixed reversed parities, across couplings and within one
    coupling, each return `UnsupportedVertex`. Uniform signs return the sign.
- **The brief** is committed with the fix.

## Falsifiers: before and after

Every cargo call used `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`. All runs used
`cargo test -p vibegraph-lib --no-fail-fast --test standalone_jamps --test toy_models -- --nocapture --test-threads=1`.

**Step 0: the seven rows delisted, production unchanged.** The result was
`test result: FAILED. 12 passed; 7 failed`. Each of the seven failed on its own
disagreement, through the `(false, None) => panic!` arm. For example:

```
ee_to_ttxh_identity: 48 flow entries, G = Complex { re: -0.009115721481584026, im: -0.8261515833626926 }, worst flow 3.977e-1, worst |M|² 2.788e-1
ee_to_ttxh_identity: 240 graph entries, worst graph 1.047e0
ee_to_ttxh_identity: the flows agree only up to |G| = 0.8262018730738958
hh_to_hh_chbox: 8 flow entries, G = Complex { re: 0.0, im: -0.7393974079609698 }, worst flow 4.246e-1, worst |M|² 8.083e-1
```

The worst-graph values were `ee_to_ttxh_identity` 1.047e0,
`ll_to_qqxs_toy_yukawa` 5.297e-2, `ll_to_ss_toy_dscalar` 1.785e-1,
`sl_to_sl_toy_dscalar` 9.067e-3, `ttx_to_hh_chbox` 2.236e-1, `ht_to_ht_chbox`
3.114e-1 and `hh_to_hh_chbox` 2.606e-1. These are C1's numbers.

The new toy test panicked at
`diagram_eval.rs:268: a vertex's couplings carry mixed rooting-convention signs`.

**Step 1: the asserts became refusals, arms unchanged.** The same seven rows
failed with the same numbers. The toy test now failed with an error, not a panic:

```
compile 'lt~ lt > qt qt~ st NP<=3': unsupported vertex: the structures of `CouplingId(6)*(1*ScalarProduct(Leg(0),Leg(1))) + CouplingId(8)*(1*ScalarProduct(Gamma5(Leg(0)),Leg(1)))` carry mixed rooting-convention signs, so the vertex has no single sign to lift into the diagram's
```

**Step 2: the Identity arm alone.** The result was `14 passed; 5 failed`.
Exactly the two `Identity` rows flipped, to C1's predicted values:

```
ee_to_ttxh_identity: 48 flow entries, G = Complex { re: -2.2041676406561736e-17, im: 0.9999999999999978 }, worst flow 1.911e-15, worst |M|² 2.985e-15
ee_to_ttxh_identity: 240 graph entries, worst graph 4.119e-15
ll_to_qqxs_toy_yukawa: 48 flow entries, G = Complex { re: -8.502227252314508e-17, im: 0.9999999999999997 }, worst flow 2.950e-16, worst |M|² 1.447e-16
ll_to_qqxs_toy_yukawa: 1920 graph entries, worst graph 4.066e-16
```

The five derivative rows were unchanged to the digit. The toy test's Yukawa
vertex now compiled, and the test was refused at the next mixed vertex, the
merged `SSS1 + SSSD`:

```
compile 'lt~ lt > qt qt~ st NP<=3': unsupported vertex: the structures of `CouplingId(11)*(1*ScalarProduct(Leg(0),Leg(1))) + CouplingId(12)*(1*ScalarProduct(Metric(P0,POut),Leg(0),Leg(1))+…)` carry mixed rooting-convention signs, …
```

**Step 3: both arms.** The result was `test result: ok. 19 passed; 0 failed`
for standalone_jamps, and `5 passed` for toy_models.

| row | G | worst flow | worst graph | C1's dump prediction (graph) |
|---|---|---|---|---|
| `ee_to_ttxh_identity` | +i | 1.911e-15 | 4.119e-15 | 4.12e-15 |
| `ll_to_qqxs_toy_yukawa` | +i | 2.950e-16 | 4.066e-16 | 4.07e-16 |
| `ll_to_ss_toy_dscalar` | +i | 2.678e-15 | 2.318e-15 | 2.32e-15 |
| `sl_to_sl_toy_dscalar` | −i | 3.171e-16 | 4.257e-16 | 4.26e-16 |
| `ttx_to_hh_chbox` | −i | 1.797e-14 | 1.596e-14 | 1.60e-14 |
| `ht_to_ht_chbox` | −i | 6.209e-16 | 3.423e-15 | 3.28e-15 |
| `hh_to_hh_chbox` | −i | 4.526e-15 | 1.810e-15 | 1.81e-15 |

All seven are inside `REL_TOL = 1e-12`. `ht_to_ht_chbox`'s worst graph,
3.423e-15, is not the 3.28e-15 C1 printed, and its worst flow, 6.209e-16, is
not C1's 6.49e-16 either. Both are ulp-level, 300× inside the tolerance. They most likely differ because C1's offline refit and the gate's
own fit are different estimators. I did not chase it.

The toy test after the fix:

```
'lt~ lt > qt qt~ st NP<=3': 21 diagrams against 72 split, worst |A_split - A_merged| / max|A| = 7.886e-16
'st lt > st lt NP<=2': 3 diagrams against 12 split, worst |A_split - A_merged| / max|A| = 3.934e-16
'lt~ lt > st st NP<=2': 3 diagrams against 12 split, worst |A_split - A_merged| / max|A| = 7.456e-16
```

So C1's Found 1 is confirmed. With both rules, all three mixed-vertex processes
compile, and each merged vertex equals the sum of its split diagrams.

## Controls

**amplitude_oracle, 49 rows, digit for digit.** I captured
`cargo test -p vibegraph-lib --test amplitude_oracle -- --nocapture --test-threads=1`
at `cb49b40` and again after the fix. I stripped ANSI codes and the
`finished in` line from each, then sorted the lines (169 each):

```
$ diff <(grep -v "Running tests" C2-oracle-before.txt) <(grep -v "Running tests" C2-oracle-after.txt) && echo IDENTICAL
IDENTICAL
```

Both runs report `49 passed; 0 failed`.

**standalone_jamps, the other twelve rows.** Every one prints the same G, worst
flow and worst graph as at `cb49b40`. For example, `ee_to_ttxh` worst graph
4.048e-15, `ee_to_ee_4f` 2.981e-14, `bbx_to_hh` worst flow 2.327e-15,
`tata_to_ttxhh` worst flow 2.167e-14, and `wpwm_to_epem` still informational at
worst flow 6.287e0.

**renumbering, rooting_soundness.** These are lib-test modules, and they ran in
`cargo test --workspace`. Every non-ignored test passed. The slow
`all_rootings_preserve_amplitude` sweep is `#[ignore]`d by design, and I did not
run it.

**`probe_bridge`: the brief names a control that does not exist.** No test or
task in the tree is called that. C1's "probe bridge" was a throwaway check: the
per-diagram sum against the full evaluator. The committed gate now bounds it
anyway. Each row gates per MadGraph graph (one diagram compiled at a time
through the production evaluator, then summed) and per flow (the full
evaluator), under one fitted G, and JAMP = Σ c·AMP. Both levels agree to
≤1.8e-14, so the per-diagram sum and the full evaluator agree at that scale.

**`pixi run --skip-deps validate`, before and after.** To answer "which σ rows
move", I ran the banked layer twice: once on the fix, and once with the five
touched files checked out at `cb49b40`. I then restored them; the restored diff
`cmp`s identical to the committed one. I diffed every per-cell JSON under
`target/validation-report/{diagrams,amplitudes,integrals,samples}`, with
duration fields removed:

```
diagrams 56 cells; 0 differ []
amplitudes 48 cells; 0 differ []
integrals 52 cells; 0 differ []
samples 53 cells; 0 differ []
```

- After: `VALIDATE EXIT 0`, 63 suites, 1791 passed, 0 failed, 84 ignored.
  `64 rows × 4 categories = 256 cells: 203 measured in the layers this run drove
  (196 ✅, 7 ⚠️, 13 ⏳, 40 — / uncovered)`. `the measured cells are the declared
  cells`.
- Before: `VALIDATE EXIT 0`, with the same cell line.

**No banked σ row moves.** Every recorded σ, error, pull and χ²/dof is
identical. No banked row reaches a crossed-line `Identity` current or a
derivative all-scalar vertex.

## Gates

All gates ran on the committed tree, `4e8b614`.

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: exit 0.
- `cargo test --workspace --no-fail-fast`: `TESTWS EXIT 0`, 38 suites, 1406
  passed, 0 failed, 16 ignored (summed `test result` lines).
- `pixi run --skip-deps validate`: exit 0, as above.

## Blind spots

- **The merged-vertex test is internal.** It shows that the merged vertex equals
  the split diagrams. The split signs are pinned against MadGraph by the
  standalone rows, but the merged vertex itself has no MadGraph row. A sign
  shared by both structures of a merged vertex is invisible to this test, and
  the split rows have to see it.
- **Global phase** per standalone row, as in C1. Both rules move one diagram
  class against the rest, which the rows see.
- **Symmetric derivative structures only.** Every derivative structure reached
  is symmetric in its legs: `SSSD`, `SSS2`, `SSS3`, `SSSS2` and `SSSS3`. A
  leg-slot or `POut` error confined to cross terms stays invisible (C1 Found).
- **`Identity` through a summed spinor index** (`idx < 0`). This is excluded
  from the rule by `standalone_projector_crossed`, as for `Gamma5`, and no row
  reaches it.
- **The refusal has no real-model trigger left.** After both fixes, no model in
  tree produces a mixed-sign vertex, so the refusal is pinned only by the unit
  test and by the step-1 run above.

## Found

- **The `tensor()` uniformity asserts still panic.**
  `VertexTerm::tensor` and `VertexInfo::tensor` (`diagram_eval.rs`) `assert!`
  that a vertex does not mix a cyclic tensor term with a tree-rootable one. A
  user UFO that writes, for example, a tensor four-fermion structure beside a
  scalar one under one coupling order panics at compile. To close it: the same
  `Result` treatment as the sign asserts, plus a hermetic test.
- **No MadGraph row for a mixed-structure vertex.** C1 Found 1 asked for one. The
  hermetic split-sum test covers the arithmetic but not MadGraph's per-vertex
  reading. To close it: bank a toy row from a merged-order copy of
  `vibegraph_toy_UFO` (for example `lt~ lt > qt qt~ st NP<=3`) through
  `gen_standalone_jamps.py`. That needs the copy as an authored model, or a
  restrict-time order override.
- **The brief named a nonexistent `probe_bridge` control.** That check was
  C1's throwaway script. C1's Found already asks for a committed per-diagram
  dump tool.
- **The `ht_to_ht_chbox` worst-graph value differs at the ulp level from C1's
  print** (3.423e-15 against 3.28e-15). This is informational only.
