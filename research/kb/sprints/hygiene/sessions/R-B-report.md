---
type: Session Report
title: "R-B report: Lorentz, colour and wavefunction layer"
description: "26 findings on helas/repr, helas/color and the helas root files: oracles blind to what they claim (Z-mixing, nsv, dualize, conj sign), squared-norm tolerances, dead test-only abstractions, and a hand-written rational arithmetic."
status: draft
tags: [hygiene, review, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: commit, resource: "https://github.com/nsmith-/vibegraph/commit/f7efda6", title: "Reviewed at f7efda6, read-only"}
---
The reviewer's report, condensed by the manager. Paths are under
`vibegraph-lib/src/helas/`. Mutations were reasoned from the code, not run.

**Backlog check:** 10 filed items touch the cluster; none was re-reported.
R-B.0 notes that `dead-types-with-stale-docs` already meets `closes_when`.
That was expected, since V1 closed it and close-out deletes the file, so it is
a rediscovery, not a finding.

## Findings

| id | point | site | finding | proposed | confidence |
|---|---|---|---|---|---|
| R-B.1 | 2 | `vertex.rs:34-86`; `mod.rs:88-328` | Every `j3xxxx` caller passes `gzf = [0, √2]`, so the Z-mixing terms are zero. A sign flip at `vertex.rs:80` passes all five tests. `j3xxxx` exists only for those tests. | fix here (tests use `jioxxx`; delete `j3xxxx`) | checked |
| R-B.2 | 2 | `mod.rs:166-209` | `test_ward_identity` claims to catch normalisation sign errors, but q·J is homogeneous, so a ×2 in `weyl_ixxxxx` passes. Its doc also cites "T1". | fix here | checked |
| R-B.3 | 2 | `wavefn.rs:621-634` | `test_in_out_conversion` compares `dualize` with itself. Dropping `.conj()` in `dualize` passes. | fix here (explicit components, or delete) | checked |
| R-B.4 | 2 | `wavefn.rs:523-531,152-157` | `test_vxxxxx_incoming_vs_outgoing` uses `nhel = 0`, where `nsv` is inert. The conjugation claim is unpinned. | fix here (`nhel = ±1`, assert conj) | checked; end-to-end MG coverage suspected |
| R-B.5 | 2, 1 | `vertex.rs:659-803,553,601` | Five `*_basic` tests assert only "non-zero" plus momenta. `jvvxxx` and `jsixxx` have no caller outside their own tests. | fix here (delete the two; strengthen or delete the rest) | checked |
| R-B.6 | 1 | `vertex.rs:461` | The `fvixxx` doc says `q = fi.p + V.p`; the code subtracts. | fix here | checked |
| R-B.7 | 1 | `wavefn.rs:215-217` | The `DiracWf` doc describes `B::Fiber` and a `LorentzRepr` associated type that do not exist. | fix here | checked |
| R-B.8 | 1 | `repr/intertwiner.rs` | Its `(j_L,j_R)` table is wrong under either reading, and its orientations restate `repr/mod.rs`. | fix here (fold a corrected table into `repr/mod.rs`, delete the module; `clifford_product` becomes `cfg(test)`) | checked |
| R-B.9 | 3 | `repr/color.rs:26-29,136-262`; `color/coeff.rs:8-9` | `ColorRepr`/`SU3*`/`GroupScalar` are used only by tests that also assert the literal value. Docs claim a production `GroupScalar` boundary that does not exist. | fix here (delete; reword docs) | checked |
| R-B.10 | 4 | `color/coeff.rs:13-54` | Hand-written `gcd_i64` and checked rational multiply and add. num-rational 0.4.2's `CheckedMul`/`CheckedAdd` do the same (AGENTS.md "never hand-write a standard primitive"). | fix here | checked |
| R-B.11 | 1 | `vertex.rs:591-595,628-633` | Garbled `# Arguments` blocks. | fix here | checked |
| R-B.12 | 1 | `vertex.rs:455,643`; `mod.rs:170,223,264,348`; `color/tests.rs:175`; `repr/{numbers,vectorspace,color,lorentz}.rs` | Plan references ("T1", "C3", "Phase 2"), history narration and future-tense comments. | fix here | checked |
| R-B.13 | 1, 2 | `mod.rs:301,347,379` | Test docs name a nonexistent `weyl_oxxxxx` and claim checks (projector span, sign errors) the asserts cannot make. | fix here | checked |
| R-B.14 | 1, 3 | `repr/lorentz.rs:197,364,437,485,532`; `wavefn.rs:334,336`; `repr/numbers.rs:59` | Empty `impl` blocks left by V1's deletions; `DiracAdjoint::KET` is never read. | fix here | checked |
| R-B.15 | 4 | `wavefn.rs` | `VectorWf<F, V>`'s variance parameter is never non-default. (The `ComplexVector` marker is relied on.) | fix here, with evaluator-doc-comments-stale's site; or file | checked |
| R-B.16 | 4 | `repr/lorentz.rs:62-91,573` | `IsBra` and `SpinorRepr` have single implementors; `LorentzRepr` is never a bound. | file (multi-file API change) | checked |
| R-B.17 | 3, 1 | `color/factor.rs:34-40,110-139` | `CanonicalString`/`to_canonical` are test-only code tested only by themselves. | fix here (delete with its two tests) | checked |
| R-B.18 | 1 | `color/tensor.rs:30-31` | The `ColorAlgebraError::Unsupported` doc lists sextets as unsupported; they are supported. | fix here | checked |
| R-B.19 | 2 | 13 sites, e.g. `repr/lorentz.rs:1847`; `mod.rs:248,289` | `bare_norm_sq() < 1e-12` is an effective 1e-6 linear tolerance, so a 1e-7 component error passes. | fix here (`EPS²` or sqrt) | checked by arithmetic |
| R-B.20 | 4 | `color/flow_tags.rs:142-151,171-179` | The permutation-validation loop is duplicated. | fix here | checked |
| R-B.21 | 2 | `color/tests.rs:209-237`; `coeff.rs:150` | `ColorCoeff::conj`'s sign is pinned only by involution, which the identity also satisfies. | fix here | checked |
| R-B.22 | 1, 3 | `repr/vectorspace.rs:19-32,49,86` | The `VectorSpace` trait is unused, kept by an `allow`; the macro doc example is wrong. | fix here | checked |
| R-B.23 | 3 | `repr/lorentz.rs:1112,1155-1171` | `AsymRank2Tensor::get/slot` can be plain `cfg(test)` after rewording one sentence. `fvixxx`'s doc gate is justified by the book. | fix here | checked |
| R-B.24 | 1 | `wavefn.rs:319` | The `VectorWf` doc names test-only `j3xxxx` as a user. | fix here | checked |
| R-B.25 | 1 | `mod.rs:4-11` | The submodule list omits `color`; there are trailing empty doc lines. | fix here | checked |

**Also seen:**
- typos;
- `colorize_diagram`'s `c as u8` truncates past 255;
- `ColorCoeff`'s mixed field visibility (the surface question);
- weak single-component checks;
- the `jsixxx`/`jioxxx` momentum conventions are opposite (moot after R-B.5).

**Cross-cluster patterns:**
- doc-gated test-only items held by one production sentence;
- reference ports with `_basic` sanity tests;
- empty `impl {}` blocks after deletions;
- squared-norm-against-linear-tolerance asserts (suspected in `eval/run.rs` and
  `eval/prop_harness.rs`);
- hand-rolled permutation checks (also `diagrams/resolve.rs:515`, suspected);
- plan-stage references (grep `\bT[0-9]\b|\bC[0-9]\b|Phase [0-9]`).

## Method

| step | share | produced |
|---|---|---|
| backlog check | 10% | R-B.0 |
| small files whole, usage grep per trait, marker and const | 15% | R-B.14–16, R-B.22 |
| every test, asking which one-line mutation its asserts are blind to | 35% | R-B.1–5, R-B.13, R-B.19, R-B.21 |
| docs against bodies, representation theory, num-rational source | 25% | R-B.6–8, R-B.10, R-B.18, R-B.24 |
| colour engine and length scan | 15% | — |

**Most productive:** grepping a definition and getting no other line (points 1
and 3), and the per-test mutation question (point 2).

**Low yield:** the `lorentz.rs` tests are strong, and gave only the tolerance
finding.

**Dead ends:** four suspected convention bugs that checked out
(`build_bispinor` vs `oxxxxx`, a completeness formula, an `iovxxx` citation,
the `jsixxx` sign).

## Found

1. **`ColorTensor::conj` maps `f` to −f.** It is a faithful MadGraph port, and
   unreachable today. Document it as such, or assert that it is unreachable.
2. **The kb concept `amplitudes/repr-layer-geometry-and-axes.md` may carry the
   same wrong `(j_L,j_R)` chains as `intertwiner.rs`.** Close-out.
3. **No unit test pins vector `nsv` conjugation (R-B.4).** If no banked row has
   an incoming vector at a helicity where it matters, that is a validation
   gap.

## Brief corrections

- **`git branch --show-current` is empty on a detached checkout.**
- **`color/flow_tags.rs` is under `helas/color`,** not `helas/repr`.
- **Judging `vertex.rs` needed a caller grep outside the cluster.** The brief
  should allow that explicitly.
- **`fvixxx`'s doc gate is justified by the book,** not a production doc.

## Manager check (2026-10-09)

Worktree clean.
- **R-B.5:** reproduced. `jvvxxx(` and `jsixxx(` appear only in `vertex.rs`.
- **R-B.14:** reproduced. `KET` appears on exactly three lines.
- **R-B.10:** reproduced. The three hand-written functions are at
  `coeff.rs:14,27,42`.
- **R-B.1:** the `vertex.rs:80` expression is as quoted.
