---
type: Session Report
title: "C-H report: hygiene review of the Identity and derivative-scalar sign fixes"
description: "Hygiene review of PR #32's C1 and C2 change: findings written before any edit, then the local ones fixed with run mutations and an identity check on standalone_jamps' and amplitude_oracle's printed numbers."
status: draft
tags: [hygiene, review, amplitude, sign-convention, scalar, toy-ufo, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: findings, resource: "https://github.com/nsmith-/vibegraph/commit/FINDINGS", title: "kb: C-H brief and findings"}
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
