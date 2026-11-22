---
type: Sprint
title: "Sprint L: v0.2.3, squared orders, Majorana, multijet timeouts and simshop's reports"
description: "Before v0.2.3: squared-order constraints, Majorana and fermion-number-violating processes, the multijet MLM timeouts, the open validation highs (VBS, TprimeB, DY1j ptj1), silent run-card placeholders, and simshop's parameter, width, provenance and reweighting requests."
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
- **The CERN host's batch campaign `v022-1009`, seed 1, against v0.2.1's seed 1.**
  - Pass 230 → 295 and panics 7 → 0.
  - Refusals and timeouts remain the largest non-ok classes: refused 511 → 434, timeouts 182 → 209. The g2HDM cards that panicked now time out, and the retries clear some of the timeouts.
  - Two possible silent wrong answers are open: the DY1j ptj1 window, which reads +7–9% with pull 14 in two bins, and the TpTp zero-weight cards.
- **simshop's SMEFT re-scan.** Of 268 probes, 37 are still refused, down from 221, with 0 regressions.

Scope was set by the user between 2026-11-20 and 2026-11-22: squared orders first; the next-release refusals; the DY1j outcome; three large blocks (the MLM multijet timeouts, Majorana and fermion-number violation, the validation highs); and all four of simshop's requests.

v0.2.3 is tagged when:
- every claimed item's `closes_when` holds, with its oracle measured (not inferred) on the PR head;
- every banked σ line is unchanged: bit for bit where a change cannot reach it, within the multi-seed criterion where it can;
- CI is green on main, `banked` included;
- the CERN host's targeted check of v022-1009's failures, built from the release-candidate code, shows no panics, and every case still failing is deliberate, deferred or filed.

After the tag, two checks run on the released binary: the full batch campaign and simshop's SMEFT re-scan. Regressions they find go to a patch release.

Every PR is based on `main`; none is stacked.

## Hosts and sessions

Sprint K ran in one 4-core, 15 GB container, with one heavy job at a time. That container bounded the sprint. Sprint L is about twice sprint K's size, so it runs as two sprint sessions and sends long runs to the CERN host.

- **Session L-A, amplitudes and physics:** PRs 1, 2, 4, 5, 6, 9 and 10. PRs 1 and 10 both change how amplitudes are split, so one session owns both and runs them in order.
- **Session L-B, refusals, reports and performance:** PRs 3, 3b, 7, 8 and 11. PR 8's parameter report and PR 9's width report share one record shape. L-B lands PR 8 first and L-A builds PR 9 on it.
- **CERN host, the only place long runs go.** It builds the PR branch locally; batch takes pinned release binaries only. Long runs means:
  - multi-seed σ gates on cards over about 20 minutes per seed;
  - MadGraph regenerations at the pinned 3.7.1;
  - timed runs on campaign cards (PR 3);
  - anything over 8 GB of memory.

  A sprint session sends the request through the manager. The request names the branch, head SHA, cards, seeds and the expected answer. The sprint session reports the result as host-measured.
- **In each sprint container:** fmt, clippy, the hermetic suite, the touched banked targets, and short MadGraph 3.5.7 checks. Each container still runs one heavy job at a time.
- **Cross-session edits.** Both sessions file kb items in their own sprint PRs. Neither edits the other's claimed items. The manager merges one PR at a time, checking that a merge doesn't conflict with a PR whose `banked` run is in flight.

## PRs, items and oracles

### PR 1: squared-order constraints (`sprintl-squared-orders`), L-A, feature-dev, Opus

| Item | Pri | Evidence |
|---|---|---|
| `squared-order-constraints-refused` | medium (first, by user decision) | 21 campaign gridpacks, plus 3 with `M3^2==1` / `T6^2==1`; simshop's #2 SMEFT blocker |

- **The split.** Split the amplitude by coupling-order class and form only the cross terms the constraint selects. Read MadGraph's squared-order semantics (`==`, `<=`, `>`, interference-only) before writing code. `reweight::poly` already proves each diagram's monomials.
- **`WEIGHTED==` / `>`.** These need the same split. Take them if they fall out of it; otherwise file them.
- **Oracles:** |M|² and σ against MadGraph for `e+ e- > t t~ NP^2==1` and `NP^2==2` on banked SMEFTsim rows. The 24 campaign cards go to the CERN host.

### PR 2: Majorana and fermion-number violation (`sprintl-majorana`), L-A, feature-dev, Opus, with validation-dev for the oracle

| Item | Pri | Evidence |
|---|---|---|
| `majorana-fermions-unsupported` | high | 82 campaign refusals: 46 fermion-number-violating and 36 Majorana |

- **Validate amplitudes first.** Check per diagram and per helicity against MadGraph's `AMP()` before trusting any |M|². Cover one neutralino process and one RPV/diquark process. The existing toy model `vibegraph_toy_majorana_UFO` is the first row.
- **Keep MadGraph's own refusals.** Majorana fermions in four-fermion vertices stay refused, as MadGraph refuses them.
- **Fallback.** This is sprint-sized. If it runs long, it can move to v0.2.4 without holding the tag.

### PR 3: MLM multijet timeouts, triage (`sprintl-timeout-triage`), L-B, performance-dev, Opus

| Item | Pri |
|---|---|
| `multijet-mlm-cards-time-out-in-the-gridpack-campaign` | high |

- **Attribute every timeout.** Each timeout that remains after v022-1009's retries is attributed to a phase from a timed run: enumeration, compilation, channel construction or sampling.
- **Where it runs.** The timed runs are on the CERN host. The triage reads them here, and the dominant causes are filed as their own items.

### PR 3b: the timeout fixes PR 3 names (`sprintl-timeout-fixes`), L-B, performance-dev, Opus

| Item | Pri |
|---|---|
| `mlm-vanishing-flavour-groups-still-sampled` | medium |
| `zero-weight-integrand-runs-to-point-cap` | medium |
| What PR 3 files | — |

- **Measure the gain.** Wall time on the named cards, measured on the CERN host.
- **Prove nothing else moves.** σ bit for bit on the cards the fixes don't reach.

### PR 4: VBS stable-boson σ high (`sprintl-vbs`), L-A, validation-dev, Opus

| Item | Pri | Evidence |
|---|---|---|
| `vbs-stable-boson-sigma-reads-high` | high | 5 CMS VBS cards read +3–13% at 5 seeds; still pull > 3 in v022-1009 |

The excess is attributed per event or per diagram against MadGraph, not from σ alone, on one polarised and one unpolarised card. It is then fixed.

### PR 5: multichannel misses a region (`sprintl-multichannel`), L-A, validation-dev, then performance-dev, Opus

| Item | Pri | Evidence |
|---|---|---|
| `multichannel-subdominant-channel-carries-dominant-region` | high | TprimeBToTZ M1800 reads about 3% low on 4 of 5 seeds |

Name the region from per-event output. Then reach it on every seed at the campaign budget: at least 5 seeds there, which run on the CERN host.

### PR 6: the DY1j ptj1 window (`sprintl-dy1j`), L-A, validation-dev, Opus

| Item | Pri | Evidence |
|---|---|---|
| `dy1j-ptj1-100to200-sigma-reads-high`, retitled to the whole ptj1 window | high | 200to400 and 400to600 read +9.0% and +7.2% against MadGraph 2.9.18, pull 14 |

It starts when the CERN host's MadGraph 3.7.1 result on cards 408 and 410 arrives.
- **If MadGraph 3.7.1 agrees with vibegraph:** register the MadGraph 2.9.18 deviation and document it.
- **If MadGraph 3.7.1 agrees with the gridpacks:** fix the rank cut. A v0.2.2.1 patch refusing ptj1 windows goes first, as its own small PR off the v0.2.2 tag.

### PR 7: run-card and UFO refusals (`sprintl-refusals`), L-B, feature-dev, Sonnet

| Item | Pri |
|---|---|
| `run-card-dollar-lines-skipped-silently` | high |
| `ufo-non-string-parameter-value-refused` | medium |
| `ufo-conditional-parameter-definitions-refused` (dim6top; it unblocks `ufo-coupling-order-expressions-refused`, which this PR then claims) | medium |
| `time-of-flight-unimplemented` (if it is small) | low |

Each item gets a CLI refusal test. ctGpatched either enumerates or is refused with MadGraph's reason. dim6top loads.

### PR 8: simshop's check-time reports (`sprintl-reports`), L-B, feature-dev, Sonnet

| Item | Pri |
|---|---|
| `model-parameters-not-reported` (including the per-process list of the parameters that enter) | medium |
| `artifact-provenance-not-machine-readable` | medium |

The record shapes are documented in the ldjson schema as added fields, under schema_version 1.

### PR 9: Auto widths (`sprintl-widths`), L-A, feature-dev, Opus, after PR 8

| Item | Pri |
|---|---|
| `auto-widths-not-computed` | medium |

Two-body tree-level widths for t, W, Z, H and one BSM particle, against MadGraph's MadWidth. The widths are reported through PR 8's record and banked with the run.

### PR 10: helicity-aware reweighting (`sprintl-helicity-reweight`), L-A, feature-dev, Opus, after PR 1

| Item | Pri |
|---|---|
| `reweight-helicity-summed-only` | medium |

The `gen_reweight_oracle.py` rows with `change helicity True` must agree event by event. Use PR 1's graded evaluator if it serves.

### PR 11: campaign leftovers (`sprintl-leftovers`), L-B, validation-dev, Sonnet

| Item | Pri |
|---|---|
| `bstar-tw-zero-effective-points` | medium |
| The TpTp zero-weight item and the QCD-4Jets item, which sprint K files after the v022-1009 verdicts | — |
| `stitch-set-sign-assert-panics` | low |
| `g2hdm-mlm-sigma-unmeasured` (closes on the batch campaign's workday round) | medium |

The TpTp item comes first: a zero |M|² on points that pass the cuts may be a silent wrong answer behind a named refusal.

## Dispatch

1. **Inputs:**
   - v022-1009's final verdict table, plus the refusal and timeout breakdown, for PRs 3 and 11;
   - the CERN host's MadGraph 3.7.1 run, for PR 6.
2. **Wave 1.** L-A runs PRs 1, 2 and 4, then PR 6 when its input lands. L-B runs PRs 3, 7 and 8. Each session runs at most three agents at once and one heavy job.
3. **Wave 2.** L-A runs PR 10 after PR 1 merges, PR 9 after PR 8 merges, and PR 5. L-B runs PR 3b after PR 3's triage, and PR 11.
4. **Hygiene.** One hygiene-dev pass per merged feature cluster.
5. **Release candidate.** CI on main and the CERN host's targeted check on locally built release-candidate code, then the tag. After the tag, on the released binary: the full campaign and simshop's re-scan.

**If it runs long.** The release can cut after wave 1 plus PR 10. PR 2 (Majorana) or PR 3b (timeout fixes) would then move to v0.2.4.

## Deferred

- NLO (170 refusals).
- Spin-2 and spin-3/2.
- Polarized intermediates.
- SMEFTatNLO.
- Lane-batched evaluation in production, the SIMD line. It changes the sampling order, so σ lines are no longer bit for bit, and it needs the artifact format to carry a sampler kind. That belongs to a v0.3 line, planned separately.
