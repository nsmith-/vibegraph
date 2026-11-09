---
type: Sprint Record
title: "Sprint I: v0.2 last refusals and fixes"
description: "Before v0.2.0: absent-card defaults follow MadGraph, five panics or silent answers become refusals, every uncut soft or collinear limit is refused, a divergent banked row is retired; the window rule proved IR-divergent (D2, D3)."
closed: 2026-11-09
status: draft
tags: [release, v0.2, run-card, defaults, cuts, ir-safety, decay-chains, sprint, sprint-record, sprint-i]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-09}
sources:
  - {id: sprint, resource: "sprint.md", title: "Sprint overview"}
  - {id: log, resource: "log.md", title: "Sprint log"}
  - {id: d1, resource: "decisions/D1-ewk-lljj-refused.md", title: "D1"}
  - {id: d2, resource: "decisions/D2-cut-decays-keep-the-stricter-cut.md", title: "D2"}
  - {id: d3, resource: "decisions/D3-identical-decays-behind-madgraph-compat.md", title: "D3"}
  - {id: iz2, resource: "sessions/I-Z2-report.md", title: "validate-deep on 2b26c109"}
---
Sprint I was the last sprint before v0.2.0. It closed the remaining items where
a card reached something vibegraph does not honour and got a panic or a silent
wrong answer. It ran as three PRs, as the user asked.

## What was banked

| PR | Change | Oracle (measured) |
|---|---|---|
| #111 (merged, 2803c0e9) | With no run card, the default card is the one MadGraph's `output` writes: MLM on mixed jet multiplicities, lepton beams, `sde_strategy`, and e p / neutrino / EVA beams then refused by name. Untabulated partons, a zero τ floor and light `mxx_only_part_antipart` codes refused. A worker panic exits 101 at once. | Every field of three MadGraph-written default cards compares equal. σ with no run card on `p p > e+ e-` + `add process … j`: vibegraph 1020.10 pb against MadGraph 1020.04 pb, 5 seeds a side, pull +0.06 (was +7.4%). A refusal test for each change, each failing before it. |
| #113 (approved; merge pending) | Every massless soft, final-collinear or beam-collinear limit that no compiled cut (or MLM's xqcut veto, for coloured jet pairs) regulates is refused. `pp_to_epemjj_qcd0` retired (D1). | A sweep of all 62 seeded rows: 61 accepted, the retired row refused. 10 tests, each failing with its rule disabled. The pole probe for D1. The CERN host's refusal census on the CMS gridpacks found no new false refusal. |
| #110 | The plan and the record; release tooling (`v*`-only describe, hyphenated tags as prereleases, committed notes the publish job requires); version 0.2.0; release notes; D1–D3 and the kb corrections they imply. | A `release.yml` dispatch with publish skipped (run 42455545280); `release-check.sh` accepts v0.2.0-rc1 and v0.2.0. |

## Decisions (all the user's)

- **The spacelike floor left the sprint.** The CERN host's MadGraph reference
  on `p p > a j j` showed no σ bias at convergence (stock 1.000 ± 0.006 at
  `ptj 0.1`), only a larger seed spread. The item became a performance item on
  the vibegraph manager's branch.
- **[D1](decisions/D1-ewk-lljj-refused.md):** refuse the equivalent-photon
  limit and retire `pp_to_epemjj_qcd0`. Its |M|²·|t| is constant over 8
  decades of t; the banked agreement with MadGraph was two pole floors agreeing.
- **[D2](decisions/D2-cut-decays-keep-the-stricter-cut.md):** keep the stricter
  `cut_decays` rule for v0.2. The window rule is IR-divergent on the
  every-pairing |M|² (I-C1's probe).
- **[D3](decisions/D3-identical-decays-behind-madgraph-compat.md):** identical
  particles across decays move behind `--madgraph-compat`, which defaults to off
  for every site. The planned default is a pairing-weighted cut. This is
  documentation only in v0.2.
- **The release gates:**
  - skip `ladder-2to6` (6–8 h here) for v0.2.0;
  - skip the full CMS rerun, because #113's census is the real-card check;
  - run the full campaign at the v0.2.0 revision with the performance
    benchmarks (`full-cms-campaign-after-v02-cut`);
  - drop the ignored probes the user judged largely defunct.

## validate-deep on the release candidate

Run on 2b26c109, #113's approved head containing main
([I-Z2](sessions/I-Z2-report.md)). The workspace tests (2011 passed) and
`validate.sh` (exit 0) passed on the same commit.

| Task | Result |
|---|---|
| 2→6 σ rows (info) | bbx +0.06%, pull +0.31; uux +0.48%, pull +1.85; one seed each |
| ladder-bb, ladder-recarded | every rung within pull 1.98, no drift with budget |
| CMS 880-member PDFs | every member exact; 9,260,000 weights per run agree with MadGraph to ≤ 5e-8 |
| MLM σ gates (catch-all) | all five against MadEvent pass |
| kT clustering (both tests) | not run: no LHAPDF here, and the dump generator demands a byte-identical replay |
| ladder-2to6 | not run (the user's decision) |
| catch-all | 53 ok; 15 failures: 9 expected (their ignore reasons or missing env), 2 MLM dump tests with no dumps, and 4 probes that fail identically before sprint I |

## Items

**Closed:**
- by #111: `absent-run-card-mixed-multiplicity-unmatched`,
  `absent-run-card-lepton-beams-panic`, `zero-shat-floor-panics-with-nan-x`,
  `mxx-only-part-antipart-silently-ignored` and
  `rayon-worker-panic-leaves-process-alive`;
- by #113, when it merges: `uncut-collinear-limits-beyond-photon-quark-unrefused`
  and `ir-unsafe-exemption-guard-sees-coloured-legs-only`.

**Released:**
- `spacelike-floor-cuts-phase-space-below-the-largest-pt-threshold`, re-scoped
  as a performance item;
- `cut-decays-identical-quarks-cut-a-stricter-observable`, rewritten as the
  pairing-weighted cut (D3).

**Not claimed:** `long-tier-gates-never-run-in-ci`. This sprint ran the long
tier by hand.

## Corrections to the brief and the items

- **The matching rule.** `create_default_for_process` also sets `drjj`,
  `sys_alpsfact` and `dynamical_scale_choice`, and has e p / neutrino / EVA
  beam rules.
- **The zero τ floor.** MadGraph does not refuse the card: it integrates it
  behind an implicit √ŝ ≥ 2 GeV floor. The `madgraph-defects` row is added.
- **`mxx_only_part_antipart`.** The `{11: True}` oracle is refused, like
  `pt_min_pdg` light codes. No remaining code moves σ.
- **The window rule** was not IR-safe (I-C1). H-C3's Found 2 is unsound for
  full-|M|² exemptions but the right guard under the weighted cut (D3).
- **`lhaid 247000` and `nn23lo1`** are the same set, so the local MadGraph
  comparisons are like for like.
- **The kT clustering cut.** I-C2 at first claimed vibegraph does not
  implement it. It does (`setclscales.rs`). The manager's review caught the
  false refusal that followed.

## Filed from the sessions

`full-cms-campaign-after-v02-cut` (high), and at medium:
- `ignored-probes-unmaintained`
- `kt-dump-generator-demands-bit-exact-replay`
- `explicit-card-without-sde-strategy-line`
- `degenerate-point-guard-masks-structural-errors`
- `xqcut-veto-coverage-unproven`
- `banked-cards-guard-not-on-the-production-path`
- `ewk-lljj-row-needs-a-regulated-rebank` (filed in #113)

At low:
- `validate-deep-recipe-lacks-its-inputs`
- `two-to-six-info-rows-iteration-chi2-large`
- `recarded-ladder-comments-stale`
- `madgraph-launch-refusals-not-mirrored`
- `lhef-banner-records-no-seed`

`ldjson-stream-gaps` gained the panic hook's flush caveat.

## Lessons

- **A guard needs real cards, not just unit tests.** The `xqcut` false refusal
  passed every unit test and the banked sweep. Review against MadEvent's code
  found it, and the census on real CMS cards was the check.
- **A clean text merge is not a clean merge.** Twice, two PRs touching
  `cuts.rs` and `proton.rs` merged without conflict and broke a test. Re-run
  the full workspace tests on every merged head before pushing.
- **An IR-safety argument needs a probe.** "A soft quark has m → 0" covered the
  soft limit and missed the beam-collinear one. I-C1's matrix-element probe
  settled it before any code was written, and D1's probe settled the EWK row
  the same way. A flat ladder showed only the sampler's pole floor.
- **Provision every declared input.** A missing multigrid PDF set turned a green
  `validate.sh` red. Use the pixi task's dependency list, not a copied
  shortlist.
- **Long runs on this container need resumable records.** Two restarts killed
  runs; per-test timestamped logs let the catch-all resume without repeating
  finished work.
- **A probe nobody runs rots.** Four ignored probes failed identically before
  the sprint. Running the long tier by hand found them, which argues for the
  CI job that `long-tier-gates-never-run-in-ci` asks for.
