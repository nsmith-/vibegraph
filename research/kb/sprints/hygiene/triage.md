---
type: Audit
title: "Hygiene sprint triage"
description: "The manager's disposition of the eight reviews' 176 findings: 125 fix here across nine fix sessions, 50 filed as 35 new items, 4 rejected or close-out only, 3 needing the user's call."
status: draft
tags: [hygiene, triage, audit]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: reports, resource: "sessions/", title: "R-A … R-G2 session reports"}
  - {id: d3, resource: "decisions/D3-fix-small-file-large.md", title: "D3: fix small, file large"}
---
The triage of the eight review reports, by the rules of
[D3](decisions/D3-fix-small-file-large.md). Finding ids are the reports'
(`R-E.1` is `sessions/R-E-report.md`, finding 1).

## Totals

| | A | B | C | D | E | F | G1 | G2 | total |
|---|---|---|---|---|---|---|---|---|---|
| reported | 21 | 26 | 20 | 17 | 15 | 28 | 27 | 22 | 176 |
| fix here | 16 | 24 | 12 | 14 | 9 | 22 | 14 | 14 | 125 |
| filed | 4 | 1 | 9 | 4 | 6 | 6 | 13 | 7 | 50 |
| rejected, or close-out only | 1 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 4 |
| user's call | 1 | 0 | 0 | 0 | 0 | 0 | 1 | 1 | 3 |

Some findings split into a *fix here* part and a *file* part, so a column can
sum to more than its "reported" count. R-B.0 is a rediscovery of V1's work,
counted as rejected. R-E.15 (drifted item lines) is close-out work.

## How the rules were applied

- **Local doc, test and dead-code fixes are *fix here*,** including those that
  touch several files inside one cluster (R-B.12, R-G1.26).
- **A new shared type or helper used across clusters is *filed***
  (`common::banked`, a `Sign` type, `LheEvent::incoming`). The same helper
  within one module is *fix here* (R-C.16, R-D.8, R-F.13).
- **A real bug with an unambiguous fix is *fix here* under the stop-rule,**
  pinned by a test that fails before the fix:
  - R-C.5 (`div_terms` drops a divisor)
  - R-C Found 1 (`recompute` revives a locked parameter)
  - R-C Found 2 (`scan` substring refusal)
  - R-E.1 (the duplicated `SDE_strategy` predicate)
  - R-G2.2 (the Drell–Yan OR gate)
  - R-G2.15 (the stale RAMBO reference)
- **Tightening a gate is *fix here* only where a recorded reading sets the new
  bound** (the census, a manifest note or a banked error), so no seed sweep is
  needed:
  - R-G2.2 and R-G2.5 tighten to the recorded readings;
  - R-D.9, R-F.5, R-F.25 and R-G2.14 tighten to a stated multiple of the
    run's own quoted error.

  A tightening that would need a new multi-seed measurement is filed.
- **The evaluator hot path:** no fix in F-A may change the emitted program or
  a kernel body (R-A brief).
- **Corrections to filed items' site lists** are applied by the manager at
  close-out, because dev agents never edit items:
  - R-E.15 (drifted lines);
  - R-F.20 (artifact literals);
  - R-A.4 (evaluator module map);
  - R-G2.6, R-G2.7 and R-G2.8 (llj and manifest notes).

  The sites themselves are fixed in the fix sessions.

## Fix here, by session

Each fix session gets its cluster's claimed items (from [sprint.md](sprint.md),
"Scope") plus these findings.

| session | findings | claimed items |
|---|---|---|
| F-A | R-A.1, .2 (docs), .4, .5, .6, .7, .8, .9, .10, .12, .13, .14, .15, .16, .19, .20 | evaluator-doc-comments-stale |
| F-B | R-B.1–15, .17–25; R-B Found 1 (document `ColorTensor::conj` of `f` as MadGraph-faithful and unreachable); R-G1 Found 2 (`LeadingColorFlows::of` refuses out-of-range contributions) | — |
| F-C | R-C.2, .3, .4, .5, .6, .7, .8, .9, .10, .16, .18 (test), .20; R-C Found 1, Found 2 | ufo-asin-acos-evaluate-as-acsc-asec, make-anti-negates-singlet-octet-colour, process-model-and-artifact-doc-comments-stale, reweight-forbidden-onshell-guard-is-dead, feyngraph-submodule-pin-differs-from-build |
| F-D | R-D.1, .2 + .3 (delete the test-only parallel scheme), .5, .6, .7, .8, .9, .10, .11, .13, .14 (shared `assert_normalized`), .15, .16 (doc) | sampler-and-phase-space-doc-comments-stale, madgraph-line-citations-predate-pin |
| F-E | R-E.1, .6, .7, .8 (value tests), .9, .11, .12, .13, .14; R-G2 Found 1 (the `event_scales` doc) | configuration-weights-wrong-at-sde1-with-tmin |
| F-F | R-F.9, .10, .11 (lib sites), .12, .13, .14, .20 | artifact-reader-arm-names-format-version, runcard-opaque-defaults-unverified |
| F-CLI | R-F.1, .3, .5, .6, .7, .8, .11 (CLI and report sites), .16, .17, .18, .21, .22, .23, .24, .25, .26 | no-network-variable-read-two-ways |
| F-G1 | R-G1.1, .2, .3, .4, .6, .7, .9, .10, .11, .12, .14 (docs and non-empty guards), .23, .25, .26 | config-amp-phase-and-sign-unpinned, smeftsim-vendored-checksum-not-hermetic |
| F-G2 | R-G2.2, .3, .4, .5, .6, .7, .8, .9, .14, .15, .19, .20, .21, .22 | validation-test-comments-stale, validate-scales-module-doc-stale, validate-hadronic-calibration-comments-superseded, manifest-notes-describe-superseded-state, jj-banked-orderings-eta-uses-wrong-components |

The F cluster splits in two (library I/O, and CLI plus report) because it
holds 22 findings across three crates.

## Filed at close-out

Each line becomes one backlog item. Areas are in brackets.

1. **[hygiene] Sibling integrands duplicate their methods:** R-E.2, R-F.4
   (integrate/generate assembly), R-D.14 (one combiner), and the duplicated
   `multiplicity.rs` impl from R-D's also-seen.
2. **[hygiene] `hadronic.rs` and `proton.rs` want splitting,** with the
   `budget` cycle: R-E.3, R-E.4, R-E.5.
3. **[hygiene] `IterationCombination` is threaded but never selected:** R-D.4,
   and `combine_kept` ignoring `rule` (R-D Found 2).
4. **[hygiene] RNG stream families have no registry or disjointness test:**
   R-D.12.
5. **[hygiene] Three categorical draws:** R-D.16 (unification).
6. **[hygiene] Kinematic helpers are hand-rolled:** about six Lorentz boosts,
   pT/η/Δφ in three modules (R-D cross-cluster pattern).
7. **[feature] `pub` fields beside derived state:** R-C.11, R-G1.16, R-F.19,
   plus the unnameable `pub` return types (R-A.11) and the empty
   `ParsingOptions` (R-C.12). Linked from lib-pub-api-surface-unaudited.
8. **[hygiene] Masslessness is defined two ways:** R-C.13.
9. **[hygiene] The final-state side of a line is computed four ways:** R-C.14.
10. **[hygiene] `WEIGHTED` is computed three ways:** R-C.15.
11. **[hygiene] Long functions to split:**
    - `generate_sets_inner` and `generate_undecayed` (R-C.17);
    - the parse, chain and engine functions over 110 lines;
    - `integrate_channels` if F-D does not get to it (R-D.13).
12. **[hygiene] UFO required fields default silently,** and the SLHA `DECAY Auto`
    policy: R-C.19, R-C.18 (policy).
13. **[validation] Refusal oracles accept any refusal:** R-C.1, R-G1.14 (class
    matching).
14. **[hygiene] Test-side banked readers and census skeletons are duplicated:**
    R-G1.17–20, .22, .27; R-G2.16, .17 (with the `gzip` shell-outs), .18.
15. **[hygiene] The σ-gate scaffold is hand-copied:** R-G2.11, R-G2.12 (probe
    targets), R-G2.13 (`plan_for`).
16. **[validation] The HELAS reference tolerances are unexplained:** R-G1.8.
17. **[validation] The decay-chain identical-particle factor is unasserted:**
    R-G1.13.
18. **[validation] SMEFTsim census literals are not banked:** R-G1.15.
19. **[hygiene] `finite_field_msq.rs` is a study in the test layer:** R-G1.24.
20. **[validation] The validation-report collator has no tests:** R-F.2.
21. **[hygiene] `LheEvent` leg accessors and the record-line rule:** R-F.15,
    R-F.27.
22. **[hygiene] The cache root is resolved two ways:** R-F.28.
23. **[performance] The egraph dependencies are not optional:** R-A.3.
24. **[validation] Per-diagram probes bypass the production runtime:** R-A.2
    (routing).
25. **[hygiene] ±1 signs flip between `f64` and `i8`:** R-A.18.
26. **[hygiene] Spinor traits have single implementors:** R-B.16.
27. **[validation] Pruned `AMP2` is never compared with MadGraph:** R-G1 Found 1.
28. **[validation] Vector `nsv` conjugation is unpinned end to end:** R-B Found 3
    (the unit test is fixed in F-B).
29. **[feature] Fixed-beam closed-form scale choices 1, 2 and 5 have no
    reference:** R-E.8 (policy).
30. **[validation] The `from_diagram` volume probe was never promoted:**
    R-E.10.
31. **[hygiene] Run-card rationale strings name unchecked readers:** R-E Found.
32. **[hygiene] Scratch directories are hand-rolled instead of `tempfile`:**
    R-C, R-F and R-G cross-cluster patterns (AGENTS.md "never hand-write a
    standard primitive").
33. **[hygiene] About 33 MadGraph line citations in run card and LHEF code are
    unverified:** R-F Found. This extends madgraph-line-citations-predate-pin's
    scope after F-D closes it.
34. **[validation] Three-seed χ² gates are bounded by five-seed calibrations:**
    R-G2.10 (a gate-cost decision, `needs-user`).
35. **[validation] The seed-combination rule contradicts itself:** R-G2.1
    (`needs-user`, see below).

## Rejected or no action

- **R-B.0:** V1 already closed `dead-types-with-stale-docs`; close-out deletes
  the file.
- **R-D.17:** integration tests need these `pub`.
- **R-A.21:** a hot-path match; splitting it costs more than it buys.
- **R-E "also seen", Fortran transcriptions:** they stay 1:1 for bit-exact
  review.
- **R-A.17:** no code change. Its decision is the user's call (below).

## The user's call

1. **R-A.17:** record "Lorentz coefficients stay f64", which closes
   lorentz-coefficients-still-f64. The reasons are in the R-A report.
2. **R-G1.5:** widen the claimed `smeftsim-vendored-checksum-not-hermetic` so
   that F-G1 moves the whole `smeftsim` and `toy_models` targets into the
   hermetic layer (manifest and `Cargo.toml` registration). Otherwise F-G1
   moves only the checksum test and R-G1.5 is filed.
3. **R-G2.1:** this side's seeds are combined by 1/σ² in the grammar gate and
   unweighted in the MLM gates. The written seed policy says 1/σ², while
   `combine_seeds` argues 1/σ² is biased. Which rule holds is a validation
   policy decision. It is filed `needs-user` either way.

## What triage learned (for L)

- **Seven of eight reviewers hit the same protocol defects:** a `git branch`
  check that fails on detached worktrees, the absent `pixi`, the empty
  `mg5amcnlo` submodule, and a noisy bare-name backlog grep.
- **The `dead-types` rediscovery was the only re-report of filed or done work.**
  The backlog check worked.
- **Independent rediscovery corroborates a finding:**
  - R-A Found and R-C.5 (`div_terms`);
  - R-C.1 and R-G1.14 (any-refusal oracles);
  - R-G2.2 and R-F.5 (OR-gated σ checks);
  - R-D.1, R-E.7, R-A.12 and R-B.23 (test-only contract anchors).

  Two reviewers on different clusters finding the same defect class is the
  strongest signal for a lint or an agent checklist line.
- **Confidence held up on every spot check:** 26 claims checked by the manager
  across eight reports, 26 reproduced (one line number off by 20).
