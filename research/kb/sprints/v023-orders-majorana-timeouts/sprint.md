---
type: Sprint
title: "Sprint L: v0.2.3, squared orders, Majorana, multijet timeouts and simshop's reports"
description: "Before v0.2.3: squared-order constraints, Majorana and fermion-number-violating processes, the multijet MLM timeouts, the open validation highs (VBS, TprimeB, and the g2HDM charged-Higgs stretch goal), silent run-card placeholders, and simshop's parameter, width, provenance and reweighting requests."
status: draft
active: true
tags: [release, v0.2.3, squared-orders, majorana, mlm, performance, simshop, cms-gridpacks, sprint-l]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
sources:
  - {id: scope, resource: "../../pipeline/release-scope.md", title: "Release scope and the hard-error rule"}
  - {id: k, resource: "../v022-smeft-and-refusals/sprint.md", title: "Sprint K plan and record"}
---
## Goal and exit criteria

v0.2.2 is tagged on c26980f. Two checks ran on it after release:
- **`vg-validator`'s batch campaign `v022-1009`, seed 1, against v0.2.1's seed 1.**
  - Pass 230 → 295 and panics 7 → 0.
  - Refusals and timeouts remain the largest non-ok classes: refused 511 → 434, timeouts 182 → 209. The g2HDM cards that panicked now time out, and the retries clear some of the timeouts.
  - The DY1j ptj1 window is resolved, and v0.2.2 is right. On the gridpacks' exact cards, MadGraph 3.7.1 agrees with vibegraph to 0.16% or better in all four bins. The gridpacks' MadGraph 2.9.18 reads 3–8% low in 100–600 GeV and 1.4% high above 600 GeV. That is a registered deviation, with no patch.
  - One possible silent wrong answer is open: the TpTp 2Mu2B cards return zero |M|² on 28–46 M points that pass the cuts.
  - Offsets that were already present in v0.2.1 remain:
    - VBS +3.4% to +12.7%;
    - QCD_bEnriched and WBQ about +2.5%;
    - TCP +1.2% to +1.9%.
  - The seed-1 refusals by class: NLO 170, fermion-number 53, Majorana 37, spin-2/3/2 28, uncut massless limits 24, squared orders 23, polarized 23, and smaller classes.
- **simshop's SMEFT re-scan.** Of 268 probes, 37 are still refused, down from 221, with 0 regressions.

Scope was set by the user between 2026-11-20 and 2026-11-22: squared orders first; the next-release refusals; three large blocks (the MLM multijet timeouts, Majorana and fermion-number violation, the validation highs); and all four of simshop's requests.

v0.2.3 is tagged when:
- every claimed item's `closes_when` holds, with its oracle measured (not inferred) on the PR head;
- every banked σ line is unchanged: bit for bit where a change cannot reach it, within the multi-seed criterion where it can;
- CI is green on main, `banked` included;
- `release-branch-ci-and-ruleset-missing` is closed, so `release/v0.2` can be cut from the tag;
- `vg-validator`'s targeted check of v022-1009's failures, built from the release-candidate code, shows no panics, and every case still failing is deliberate, deferred or filed.

After the tag, two checks run on the released binary: the full batch campaign and simshop's SMEFT re-scan. Regressions they find go to a patch release.

Every PR is based on `main`; none is stacked.

## Hosts and sessions

Sprint K ran in one 4-core, 15 GB container, with one heavy job at a time. That container bounded the sprint. Sprint L is about twice sprint K's size, so it runs as two sprint sessions and sends long runs to `vg-validator`. Names and the message flow are in [session roles](../../workflow/session-roles.md).

- **`vg-sprint-L-physics`, amplitudes and physics:** PRs 1, 2, 4, 5, 9, 10 and the stretch PR 12. PRs 1 and 10 both change how amplitudes are split, so one session owns both and runs them in order.
- **`vg-sprint-L-pipeline`, refusals, reports and performance:** PRs 3, 3b, 7, 8 and 11. PR 8's parameter report and PR 9's width report share one record shape. vg-sprint-L-pipeline lands PR 8 first, and vg-sprint-L-physics builds PR 9 on it.
- **`vg-validator`, the only place long runs go.** It builds the PR branch locally; batch takes pinned release binaries only. Long runs means:
  - multi-seed σ gates on cards over about 20 minutes per seed;
  - MadGraph regenerations at the pinned 3.7.1;
  - timed runs on campaign cards (PR 3);
  - anything over 8 GB of memory.

  A sprint session sends the request through `vg-router` to `vg-planner`. The request names the branch, head SHA, cards, seeds and the expected answer. The sprint session reports the result as host-measured.
- **In each sprint container:** fmt, clippy, the hermetic suite, the touched banked targets, and short MadGraph 3.5.7 checks. Each container still runs one heavy job at a time.
- **Cross-session edits.** Both sessions file kb items in their own sprint PRs. Neither edits the other's claimed items. `vg-planner` merges one PR at a time, checking that a merge doesn't conflict with a PR whose `banked` run is in flight.

## PRs, items and oracles

### PR 1: squared-order constraints (`sprintl-squared-orders`), vg-sprint-L-physics, feature-dev, Opus

| Item | Pri | Evidence |
|---|---|---|
| `squared-order-constraints-refused` | medium (first, by user decision) | 23 refusals in v022-1009 seed 1, `M3^2==1` / `T6^2==1` among them; simshop's #2 SMEFT blocker |

- **The split.** Split the amplitude by coupling-order class and form only the cross terms the constraint selects. Read MadGraph's squared-order semantics (`==`, `<=`, `>`, interference-only) before writing code. `reweight::poly` already proves each diagram's monomials.
- **`WEIGHTED==` / `>`.** These need the same split. Take them if they fall out of it; otherwise file them.
- **Oracles:** |M|² and σ against MadGraph for `e+ e- > t t~ NP^2==1` and `NP^2==2` on banked SMEFTsim rows. The 24 campaign cards go to `vg-validator`.

### PR 2: Majorana and fermion-number violation (`sprintl-majorana`), vg-sprint-L-physics, feature-dev, Opus, with validation-dev for the oracle

| Item | Pri | Evidence |
|---|---|---|
| `majorana-fermions-unsupported` | high | 90 refusals in v022-1009 seed 1: 53 fermion-number-violating and 37 Majorana |

- **Validate amplitudes first.** Check per diagram and per helicity against MadGraph's `AMP()` before trusting any |M|². Cover one neutralino process and one RPV/diquark process. The existing toy model `vibegraph_toy_majorana_UFO` is the first row.
- **Keep MadGraph's own refusals.** Majorana fermions in four-fermion vertices stay refused, as MadGraph refuses them.
- **Fallback.** This is sprint-sized. If it runs long, it can move to v0.2.4 without holding the tag.

### PR 3: MLM multijet timeouts, triage (`sprintl-timeout-triage`), vg-sprint-L-pipeline, performance-dev, Opus

| Item | Pri |
|---|---|
| `multijet-mlm-cards-time-out-in-the-gridpack-campaign` | high |

- **Attribute every timeout.** Each timeout that remains after v022-1009's retries is attributed to a phase from a timed run: enumeration, compilation, channel construction or sampling.
- **Where it runs.** The timed runs are on `vg-validator`. The triage reads them here, and the dominant causes are filed as their own items.

### PR 3b: the timeout fixes PR 3 names (`sprintl-timeout-fixes`), vg-sprint-L-pipeline, performance-dev, Opus

| Item | Pri |
|---|---|
| `enumeration-hands-flavour-violating-subprocesses-to-feyngraph` | high |
| `set-compilation-loops-are-serial` | high |
| `amplitude-probe-compiles-each-set-twice` | high |
| `generate-parallel-diagrams-is-a-no-op` (if it fits) | low |

Scope from PR 3's triage ([L-T2 report](sessions/L-T2-report.md) on PR 3's branch), ruled 2026-11-23. The enumerate scaling the user approved is folded in: enumeration uses the cores by default.

- **Order-preserving only.** Every change keeps the sampling order. σ, grids and diagram sets are bit for bit against main.
- **The flavour prune reads conservation from the model.** Subprocesses in models that violate the number (CKM, FCNC, RPV, LQ) are left alone. A test on a flavour-violating model pins pruned against unpruned diagram sets.
- **Host-side enumerate diff.** v0.2.2 against this PR's head, on every released v022-1009 card that enumerates within the cap. The sets are identical card for card.
- **Measure the gain.** Cards 58, 15, 1009 and 65 are re-run on `vg-validator` with the phase instrument. DY+0–3j at `-j 2` with no flag enumerates in ≈14 s or less, and small processes are no slower.
- The sampling-schedule, memory and profile items PR 3 files go to the next sprint. So do the two MLM items, which are not the cause on any sampled card.

### PR 4: VBS stable-boson σ high (`sprintl-vbs`), vg-sprint-L-physics, validation-dev, Opus

| Item | Pri | Evidence |
|---|---|---|
| `vbs-stable-boson-sigma-reads-high` | high | Cards 134, 135, 698, 700 and 702 read +3.4% to +12.7% at 5 seeds, with mean pulls up to 22.5, in both v0.2.1 and v0.2.2. These are the largest unexplained σ offsets left. |

The excess is attributed per event or per diagram against MadGraph, not from σ alone, on one polarised and one unpolarised card. It is then fixed.

### PR 5: multichannel misses a region (`sprintl-multichannel`), vg-sprint-L-physics, validation-dev, then performance-dev, Opus

| Item | Pri | Evidence |
|---|---|---|
| `multichannel-subdominant-channel-carries-dominant-region` | high | TprimeBToTZ M1800 reads about 3% low on 4 of 5 seeds |

Name the region from per-event output. Then reach it on every seed at the campaign budget: at least 5 seeds there, which run on `vg-validator`.

### PR 6: dropped

The DY1j ptj1 window closed without a code change. Sprint K's close-out kb PR records it, with the MadGraph 3.7.1 evidence, and registers the MadGraph 2.9.18 deviation.

### PR 7: run-card and UFO refusals (`sprintl-refusals`), vg-sprint-L-pipeline, feature-dev, Sonnet

| Item | Pri |
|---|---|
| `run-card-dollar-lines-skipped-silently` | high |
| `ufo-non-string-parameter-value-refused` | medium |
| `ufo-conditional-parameter-definitions-refused` (dim6top; it unblocks `ufo-coupling-order-expressions-refused`, which this PR then claims) | medium |
| `time-of-flight-unimplemented` (if it is small) | low |

Each item gets a CLI refusal test. ctGpatched either enumerates or is refused with MadGraph's reason. dim6top loads.

### PR 8: simshop's check-time reports (`sprintl-reports`), vg-sprint-L-pipeline, feature-dev, Sonnet

| Item | Pri |
|---|---|
| `model-parameters-not-reported` (including the per-process list of the parameters that enter) | medium |
| `artifact-provenance-not-machine-readable` | medium |

The record shapes are documented in the ldjson schema as added fields, under schema_version 1.

### PR 9: Auto widths (`sprintl-widths`), vg-sprint-L-physics, feature-dev, Opus, after PR 8

| Item | Pri |
|---|---|
| `auto-widths-not-computed` | medium |

Two-body tree-level widths for t, W, Z, H and one BSM particle, against MadGraph's MadWidth. The widths are reported through PR 8's record and banked with the run.

### PR 10: helicity-aware reweighting (`sprintl-helicity-reweight`), vg-sprint-L-physics, feature-dev, Opus, after PR 1

| Item | Pri |
|---|---|
| `reweight-helicity-summed-only` | medium |

The `gen_reweight_oracle.py` rows with `change helicity True` must agree event by event. Use PR 1's graded evaluator if it serves.

### PR 11: campaign leftovers (`sprintl-leftovers`), vg-sprint-L-pipeline, validation-dev, Sonnet

| Item | Pri |
|---|---|
| `bstar-tw-zero-effective-points` | medium |
| `tptp-integrand-zero-on-every-point`, raised from low by sprint K's close-out (cards 108 and 510–512) | high |
| `sigma-reads-high-on-mlm-and-wbq-cards` (QCD_bEnriched 188/189, WBQ 260, TCP 967–969) | medium |
| The QCD-4Jets item, if sprint K files one after the verdicts | — |
| `stitch-set-sign-assert-panics` | low |

The TpTp item comes first: a zero |M|² on points that pass the cuts may be a silent wrong answer behind a named refusal. `g2hdm-mlm-sigma-unmeasured` was closed by sprint K's close-out (#167): M300 and M450 meet its `closes_when` at 5 seeds.

### PR 12: g2HDM charged-Higgs high-mass σ, stretch (`sprintl-g2hdm`), vg-sprint-L-physics, validation-dev, Opus, after PR 4

| Item | Pri | Evidence |
|---|---|---|
| `g2hdm-charged-higgs-high-mass-sigma-reads-high` | high (stretch: does not hold the tag) | M900–M1750 read +2.4% to +4.4% on every seed in v022-1009 (mean pulls 6–8, χ²/dof 0.6–1.9); M300 and M450 pass |

- Added by the user's decision of 2026-11-22. It runs after PR 4, since the two may share a cause.
- **Which MadGraph first.** The VBS and g2HDM gridpacks were made with MadGraph 2.9.18, the DY1j outlier. vg-validator regenerates σ with MadGraph 3.7.1 on VBS cards 134 and 700 and g2HDM cards 894 (M1000) and 889 (M300). If 3.7.1 agrees with vibegraph, the item becomes a registered 2.9.18 deviation.
- **Oracle.** Per event (Breit–Wigner window and threshold, width, scale, merge sequence) or per diagram against MadGraph on card 894, before σ. Locally: reduced-budget single seeds only. Every multi-seed σ runs on vg-validator (2.5–8 h per seed on 4 cores).

## Dispatch

1. **Inputs:** v022-1009's final verdict table, plus the refusal and timeout breakdown, for PRs 3 and 11.
2. **Wave 1.** vg-sprint-L-physics runs PRs 1, 2 and 4. vg-sprint-L-pipeline runs PRs 3, 7 and 8. Each session runs at most three agents at once and one heavy job.
3. **Wave 2.** vg-sprint-L-physics runs PR 10 after PR 1 merges, PR 9 after PR 8 merges, and PR 5. vg-sprint-L-pipeline runs PR 3b after PR 3's triage, and PR 11.
4. **Hygiene.** One hygiene-dev pass per merged feature cluster.
5. **Release candidate.** `release-branch-ci-and-ruleset-missing` is closed by a small hygiene PR and the user's ruleset change. Then CI on main and `vg-validator`'s targeted check on locally built release-candidate code, then the tag. After the tag, on the released binary: the full campaign and simshop's re-scan.

**If it runs long.** The release can cut after wave 1 plus PR 10. PR 2 (Majorana) or PR 3b (timeout fixes) would then move to v0.2.4.

## Deferred

- NLO (170 refusals at seed 1).
- Spin-2 and spin-3/2 (28).
- Polarized intermediates (23).
- SMEFTatNLO.
- Lane-batched evaluation in production, the SIMD line. It changes the sampling order, so σ lines are no longer bit for bit, and it needs the artifact format to carry a sampler kind. It goes to the v0.3 line, which splits from main at the v0.2.3 tag ([decision](../../decisions/v03-branch-after-v023.md)).
