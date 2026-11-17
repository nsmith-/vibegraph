---
type: Sprint
title: "Sprint K: v0.2.2, SMEFT models and the largest refusal classes"
description: "Before v0.2.2: SMEFTsim and the CMS store's SMEFT family load (negative orders, restriction cards), the cheapest UFO and run-card refusal classes close, the MLM matching panic, batch-safe integrate, one-body phase space, the coloured-scalar seagull and mixed-mass subprocess splitting, integrate memory growth on multijet MLM, and the 4× repeated-decay σ."
status: draft
active: true
tags: [release, v0.2.2, smeft, smeftsim, ufo, refusals, cms-gridpacks, simshop, sprint-k]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
sources:
  - {id: scope, resource: "../../pipeline/release-scope.md", title: "Release scope and the hard-error rule"}
  - {id: j, resource: "../v021-fixes/closeout.md", title: "Sprint J record"}
---
## Goal and exit criteria

v0.2.1 is tagged on e4a90128. Two surveys ran on it:
- **simshop's model-store scan.** simshop, a CMS private-MC web app, loaded all
  452 UFO roots in the CMS model store, bare and under each restriction: 337
  refused. Of 268 SMEFT model and restriction probes, 221 were refused.
- **The CERN host's stage-1 gridpack campaign** `v021-1009-s1`: 1005 gridpacks,
  one seed. Results: 294 ok, 478 refused (170 of them NLO), 179 timeouts and
  7 panics.

This sprint makes SMEFTsim v3 and v2 and the store's SMEFT family load and
generate the full |M|² (up to `NP<=1`). It closes the largest in-scope refusal
classes from both surveys, fixes the panic, and makes integrate batch-safe for
simshop. It is done when:

- every claimed item's `closes_when` holds, with its oracle measured (not
  inferred) on the PR's head;
- every banked σ line is unchanged: bit for bit where the change cannot reach
  it, within error where it can;
- simshop's SMEFT re-scan at the release candidate leaves no SMEFTsim v3 or v2
  probe refused except for a named, out-of-scope reason;
- the CERN host's campaign at the release candidate shows no panics and no
  unexplained ok → not-ok (`full-cms-campaign-after-v02-cut`).

Every PR is based on `main`; none is stacked on another. The toolchain pin
(#135) merges before the sprint branches are cut. The user tags the release.

**Decided (the user, 2026-11-17):**
- Squared-order constraints (`squared-order-constraints-refused`; 21 campaign
  refusals, simshop's second SMEFT blocker) are deferred to v0.2.3. v0.2.2's
  SMEFT samples are full |M|² only.
- One-body phase space, the coloured-scalar seagull and mixed-mass splitting
  are in.
- Majorana fermions (82 campaign refusals) are deferred to their own sprint.

Out of scope:
- NLO (170 refusals);
- spin-2 and spin-3/2 propagators (28);
- polarized intermediate resonances (14);
- the multijet MLM timeouts (179);
- form-factor Lorentz functions;
- `model-check-command-missing` and `structured-log-output-missing` (simshop:
  enumerate covers it for now);
- SMEFTatNLO;
- #125, which belongs to the performance re-bench;
- the CERN host's small-width (Γ/M < 1e-6) and T′ channel-miss reports until
  they are refined.

## PRs, items and oracles

### PR 1: negative coupling orders (`sprintk-negative-orders`), feature-dev, Opus

| Item | Pri | Oracle |
|---|---|---|
| `negative-ufo-coupling-orders-refused` | high | Read MadGraph's order counting and WEIGHTED treatment of a negative order (SMEFTsim's `QED:-1` vev insertion) first, and cite it. Per-diagram × per-helicity amplitudes on a SMEFT_alpha (or SMEFTsim v2 A-model) process against MadGraph standalone. The GluGluH_HToUpsilon card's σ at ≥ 5 seeds. Diagram counts unchanged on every banked row. |
| `weighted-search-gluon-free-trial-missing` | medium | The ignored `a_final_state_gluon_pair_without_a_qcd_vertex_is_refused` test runs and passes. |

Evidence: about 160 simshop probes across 8 models:

| Model | Restrictions |
|---|---|
| SMEFTsim v2 A-models | 83 |
| smeft4vbs | 29 |
| SMEFTsim_U35_MwScheme | 19 |
| SMEFT_alpha and SMEFT_mW | 12 |
| SMEFTsim_topU3l ctGpatched | 11 |

### PR 2: restriction cards (`sprintk-restriction`), feature-dev, Opus

| Item | Pri | Oracle |
|---|---|---|
| `restriction-card-near-one-values-refused` | medium | MadGraph's rule for 1 against 9.999999e-1, cited. The restricted model's parameter and coupling set equals MadGraph's on a SMEFTsim per-operator restriction. |
| `restriction-card-missing-ckm-entry-refused` | medium | MadGraph's handling of an absent entry, cited. The 22 restrictions load. |
| `ufo-full-restriction-unsupported` | medium | `-full` loads with no restriction, tested on a model that ships `restrict_default.dat`. |
| `restrict-card-unit-values-not-fixed` | low | Taken if it falls out of the same reading. |

### PR 3: UFO value forms and `import model` options (`sprintk-ufo-values`), feature-dev, Sonnet

| Item | Pri |
|---|---|
| `coupling-orders-not-inferred-without-coupling-orders-py` | high |
| `proc-card-import-model-options-refused` | high |
| `ufo-parameter-value-expressions-refused` | medium |
| `ufo-vertex-coupling-lists-refused` | medium |
| `ufo-coupling-order-expressions-refused` | medium |
| `ufo-particle-attribute-parameter-references-refused` | medium |
| `ufo-parameter-value-bare-name-refused` | low |
| `smeftsim-function-library-real-part` | low |

The oracle for each is its own `closes_when`. For the real-part item, report
whether any banked SMEFTsim value moves. PR 2 and PR 3 touch different loader
files; they run in parallel and merge in order.

### PR 4: the MLM matching panic (`sprintk-mlm-panic`), validation-dev, Opus

| Item | Pri | Oracle |
|---|---|---|
| `mlm-reweight-merge-colour-structure-panics` | high | The 7 cards never panic. The cause is named against reweight.f: does MadEvent reach the merge or cluster differently? Then a fix, with σ and the per-event merge sequence against MadGraph's record, or a named refusal before sampling. Failing-first test. |

### PR 5: run-card sentinels and the classification table (`sprintk-runcard`), feature-dev, Sonnet

| Item | Pri |
|---|---|
| `run-card-off-sentinel-values-refused` | medium |
| `ptj-n-min-nonpositive-refused` | medium |
| `run-card-classification-table-missing` | medium |
| `run-card-resolution-not-reported` | medium |
| `rank-jet-pt-cuts-unimplemented` | medium |
| `rank-lepton-pt-cuts-unimplemented` (filed in this PR's branch) | medium |

The per-card report and the per-release table come from one implementation. Both read `FIELD_CLASSES` and the parser's resolved card, and the report can ship first. Added at the user's request, 2026-11-17.

The two rank-cut items were added by the user's decision on 2026-11-17. They share one mechanism: `ptj1min`–`ptj4max` and `ptl1min`–`ptl4max` with real values are refused in `UNIMPLEMENTED_CUTS`. The lepton family keeps SMEFTsim_topU3l out of simshop's registry. Seven CMS `ZH_Hto2B_Zto2L_SMEFTsim_topU3l_*` cards integrate only with `ptl1min` patched to 0, and then agree with MadGraph at 5 seeds (|mean pull| 1.1–1.6). PR 5 files `rank-lepton-pt-cuts-unimplemented`. Its oracle is σ on one ZH card at its real `ptl1min` against MadGraph by the multi-seed criterion, and the seven cards must integrate unpatched.

PR 5 starts after PR 4 and PR 7 merge, since all three touch the run-card and
cut code.

### PR 6: batch safety and enumerate totals (`sprintk-batch`), feature-dev, Sonnet

| Item | Pri |
|---|---|
| `integrate-ignores-sigterm-outside-the-tui` | high |
| `integrate-result-has-no-stop-reason` | high |
| `enumerate-progress-has-no-total` | medium |
| `integrate-has-no-check-only-mode` | medium |
| `generate-output-not-written-atomically` | medium |
| `lhe-output-extension-ignored` | medium |
| `default-threads-ignore-batch-cpu-limits` | low |

### PR 7: one-body phase space (`sprintk-one-body`), feature-dev, Opus

| Item | Pri | Oracle |
|---|---|---|
| `one-body-final-state-phase-space-missing` | high | σ on p p > z and e+ e- > z against local MadGraph by the multi-seed criterion. |

### PR 8: the coloured-scalar seagull (`sprintk-seagull`), feature-dev, Opus

| Item | Pri | Oracle |
|---|---|---|
| `coloured-scalar-seagull-colour-unsupported` | high | Per-flow complex colour dumps against MadGraph standalone on stop pair production, then σ on an SMS-StopStop card. |

### PR 9: mixed outgoing masses (`sprintk-mass-split`), feature-dev, Opus

| Item | Pri | Oracle |
|---|---|---|
| `mixed-outgoing-mass-subprocesses-not-split` | medium | 2 campaign refusals (the 202a540 census). The grouping splits by outgoing masses itself. σ on ZPrimeToTT against MadGraph at ≥ 5 seeds. Cards whose groups have uniform masses stay bit for bit. |

### PR 10: integrate memory growth on multijet MLM (`sprintk-mlm-memory`), performance-dev, Opus

| Item | Pri | Oracle |
|---|---|---|
| `mlm-photon-four-jet-integrate-memory-grows-unbounded` | high | Profile the heap on the GJ-4Jets card and name what grows. Then bound it: peak RSS is flat in the iteration count, measured, and the card completes under 12 GB. σ is bit for bit where the change cannot reach it. |

Added at the user's request, 2026-11-17.

### PR 11: repeated decays in a decay chain (`sprintk-repeated-decay`), validation-dev, Opus

| Item | Pri | Oracle |
|---|---|---|
| `decay-chain-repeated-decay-sigma-four-times-high` | high | Per-diagram amplitudes and the symmetry factor against MadGraph standalone on `p p > w+ h01, (h01 > a01 a01, a01 > b b~, a01 > b b~)`. The cause is named and arithmetic on the dumps confirms the factor before any code changes. σ on the 4 HToAATo4B cards against MadGraph at ≥ 5 seeds. A failing-first test. Cards without repeated decays stay bit for bit. |

Added at the user's request, 2026-11-17. It is a silent wrong answer, so the hard-error rule makes it release-blocking.

## Dispatch

1. Merge #135.
2. Wave 1, in parallel: PRs 1, 2, 3, 4, 6, 7, 8, 9, 10 and 11. PR 7 (phase space) and
   PR 9 (flavour groups) both touch the sampler's subprocess setup; whichever
   merges second merges main in.
3. Wave 2: PR 5.
4. A hygiene-dev pass per merged feature PR.
5. Release candidate: simshop's SMEFT re-scan, the CERN host's campaign, then
   the tag.
