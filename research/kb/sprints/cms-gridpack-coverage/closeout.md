---
type: Sprint Record
title: "Sprint H: CMS gridpack coverage"
description: "CMS survey ok 34→66 of 199 (refused 139→93), then a diagnosis round: two silent wrong-σ bugs fixed (contact-vertex sign, mirror sampling), four guards, IR-divergent photon cards refused."
closed: 2026-11-08
status: draft
tags: [cms, gridpacks, coverage, flavour-groups, cuts, decay-chains, proton, mirror, sign-convention, sprint, sprint-record, sprint-h]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
sources:
  - {id: sprint, resource: "sprint.md", title: "Sprint overview"}
  - {id: log, resource: "log.md", title: "Sprint log"}
  - {id: d1, resource: "decisions/D1-near-degenerate-merge-rounding.md", title: "D1–D2"}
  - {id: d3, resource: "decisions/D3-decay-duplicates-match-madgraph.md", title: "D3–D4"}
  - {id: d5, resource: "decisions/D5-cut-decays-exemption-and-b-cut-cards.md", title: "D5–D6"}
  - {id: census, resource: "../../validation/cms-gridpack-refusals-202a540.md", title: "The refusal census at 202a540"}
  - {id: baseline, resource: "https://github.com/nsmith-/vibegraph/pull/71#issuecomment-6605096410", title: "Campaign main-23d3a76-200-pc on #71"}
---
Sprint H lifted refusals from the 200-gridpack CMS survey (#71). It ran in two
rounds:
- **Coverage:** eleven PRs, #92 to #106.
- **Diagnosis and fixes:** once the closing campaign showed that some newly
  lifted cards read wrong, the user held the close-out until the new issues
  were diagnosed. That round produced three more PRs, #107 to #109.

## The closing campaign

`main-202a540-200-pc` ran on the CERN host on main 202a540f, with the #106
parser fix, over the same 199 gridpacks and settings as the baseline
`main-23d3a76-200-pc`. The outcomes below are from the host's `gridpacks.sqlite`,
as relayed; the manager did not re-read the database.

| Outcome | Baseline | Closing |
|---|---|---|
| ok | 34 | 66 |
| refused | 139 | 93 |
| timeout | 24 | 37 |
| panic | 2 | 3 |

- **|pull| > 5:** 14 → 11.
- **NMSSM / XtoYY:** all 8 near-zero σ cards now agree with MadGraph.
- **The one ok → not-ok card:** TTZ2b_5f_LO, refused for too few effective
  points by sprint G's #88. The baseline had given 3011 ± 2715 pb with no
  MadGraph reference.
- **The sprint's groups (ok / timeout / refused):**
  - colour split: 10 / 8 / 1;
  - decay-chain duplicates: 6 / 4 / 1;
  - massive incoming: 11 ok, 1 timeout, 1 refused and 1 panic.

[The refusal census](../../validation/cms-gridpack-refusals-202a540.md) maps
all 93 refusals to backlog items.

## The diagnosis round

The campaign also surfaced problems on the paths sprint H opened. Each was
diagnosed before any fix was written.

| Issue | Cards | Cause | Sprint H's part |
|---|---|---|---|
| A | TCP ×2 at 4.2× | In `g g > alp g`, the ALP–three-gluon contact diagram had the wrong sign: the triple-gluon convention sign applied to three-leg vertices only | #95 made the card reachable |
| A′ | VBF_HH at 0.84× | The mirrored beam ordering peaked where no channel of the group sampled it | #95's mass split exposed it |
| B | GJ-2Jets-EWK ×2, VBFtoG ×2 at 0.44–0.77× | The cards set `draj = 0` with no isolation, so σ is log-divergent | #103 made them reachable; vibegraph was not at fault |
| C | DarkPhoton ×2, idm (panics) | No one-body phase space; an unclamped Breit–Wigner draw | Older bugs, reached through lifted refusals |
| D | QCD-4Jets_HT-100to200 (refused) | The scale-floor refusal counted xqcut vetoes | Older; #100 made the card reachable |

How each was settled:
- **A:** a per-diagram, per-helicity comparison against MadGraph standalone.
  The contact diagram was −1 at all 24 entries; flipping it closed the flow
  deviation from 0.38 to 4e-9.
- **A′:** setting the quark masses to zero merged the groups and closed it
  (1.023 ± 0.009). A per-part mirror/direct measurement then showed
  0.005–0.28 where pp symmetry gives exactly 1.
- **B:** at every finite `draj` down to 3e-4, MadGraph and vibegraph agree
  within 0.6%; below that, neither samples the pole
  ([H-B1](sessions/H-B1-report.md)). Forcing the CKM pair to merge changed σ by
  −0.13% ± 0.47%.
- **C and D:** backtraces, and the probe's instrumented counts.

The A and A′ fixes are the important ones. Both bugs gave a silent wrong σ
that predates sprint H; only cards sprint H made reachable exposed them.

## What was banked

| PR | Change | Oracle (measured) |
|---|---|---|
| #92 | Sprint plan; files the xpt* and pt_min_pdg items | – |
| #93 | A subprocess reached by two decay assignments is two subprocesses, summed incoherently (D3) | MadGraph census on all 11 survey cards (890 subprocesses, 417 reached twice); σ ratio −0.02% (H-D2, H-D3) |
| #94 | xpt* ≤ 0 reads as off; `pt_min_pdg` applied as in `setcuts.f` | Per-event pT; `p p > z j` at `{23: 100}` +0.10% (H-C1) |
| #95 | Massive incoming partons (`mom2cx`, 1/(2√λ)); `ptonium`/`etaonium` inert | Per-helicity amplitudes against standalone; b and c mass σ within 1.1e-3 (H-M1, H-M2) |
| #96 | Explicit single-ordering cards counted once | `u u~ > e+ e-` 778.48 against 778.11 (was 1556.77) (H-R1) |
| #100 | Conjugate colour bases pair under π | Per-subprocess colour labels on a stand-in HT card (H-G1) |
| #101 | `maxjetflavor` default from the beams | `p p > j j` −2.7e-4 (H-J1) |
| #103 | Groups within 1e-9 merge as rounding; physical splits kept apart (D1) | γ+jets 9.08e-7 split traced to the W width; `p p > a j j` +0.18% (H-N1) |
| #104 | One cut indicator per flavour group (D4) | Bit for bit on three cards; `p p > j j j` +2.95e-3 (H-K1) |
| #105 | Refuse an IR-unsafe `cut_decays` exemption; the stricter cut registered (D5) | Guard test; bit for bit (H-C3) |
| #106 | gpdb sums a tag's flavour groups | `test_parse` |
| #107 | The mirrored ordering as MadEvent's IMIRROR = 2: one \|M\|², the partons exchanged between beams, μF following the parton, the event written rotated | VBF_HH 1.0111 ± 0.0080 at 1e5 over 10 seeds (was 0.99 ± 0.08, χ²/dof 21); per-event check against MadGraph 0/2504 failures (was 2504/2504); a second bug fixed, −1.6% with unequal fixed μF (H-V2) |
| #108 | `boson_momentum_sign`: −1 for an all-boson vertex of self-conjugate legs iff its momentum parity is odd, derived from MadGraph's all-outgoing against vibegraph's all-incoming momenta | Four new per-flow JAMP rows, among them hggg −1 and hgggg +1; TCP 1.007 and 1.025 over 5 seeds (H-A2, on the CERN host) |
| #109 | Guards: 2→1 refusal; Breit–Wigner clamp, zeroed degenerate points, a counted NaN veto; floor refusal only on floor evidence; `UncutPhotonSplitting` | A test per guard that fails before the change; bit for bit on three cards (H-G2) |

H-Q1 traced the light-quark QCD +1–2% to MadEvent: its default survey, its
iteration combination, and under-sampling at large √ŝ.

### The closing rerun of the 11 diagnosis cards

Run by the CERN host on main 11ecef79, with the campaign settings, each
gridpack's own PDF set and 5 seeds. σ/MG is the unweighted mean (seed sd, sem,
χ²/dof).

| Card | Now | Closing campaign |
|---|---|---|
| TCP_m50 | 0.993 (0.015, 0.007, 0.53) | 4.19 |
| TCP_m55 | 1.022 (0.039, 0.018, 2.17) | 4.22 |
| VBF_HH | 1.116 (0.158, 0.071, 1.11) | 0.840 |
| QCD-4Jets_HT-100to200 | 0.977 (0.020, 0.009, 0.44) | refused |
| DarkPhoton MZd-40 | 0.985 (0.013, 0.006, 0.16) | panic |
| DarkPhoton MZd-86 | 0.971 (0.040, 0.018, 2.44) | panic |
| idm_dilepton_mH80 | 0.962 ± 0.019, seed 1 only; over the 600 s timeout | panic |
| GJ-2Jets-EWK ×2, VBFtoG ×2 | refused, `UncutPhotonSplitting` | 0.44–0.77 |

Caveats:
- **VBF_HH** is unbiased but still scatters at neval 1e4 (seed 5 alone is
  1.38): #95's mass split thins the per-group budget. At 1e5 it reads
  1.011 ± 0.008.
- **DarkPhoton** runs only with a hand-trimmed param card. Through gpdb, its
  card is refused for a `chi` mass the model lacks; gpdb's fallback then drops
  the param card and the σ becomes meaningless.

## Decisions

All the user's:
- [D1–D2](decisions/D1-near-degenerate-merge-rounding.md): merge pairs at
  rounding level, split physical ones; decide on decay-chain duplicates after
  measuring MadGraph.
- [D3–D4](decisions/D3-decay-duplicates-match-madgraph.md): match MadGraph on
  decay-chain duplicates; per-group cut indicators come into scope.
- [D5–D6](decisions/D5-cut-decays-exemption-and-b-cut-cards.md): register the
  `cut_decays` deviation, with a kinematic window rule as the later fix; keep
  integrating the b-cut cards MadGraph refuses.
- **2026-11-07 (in the log):**
  - hold the close-out until the campaign's new issues are diagnosed;
  - fix A, the mirror (following MadGraph) and the guards in sprint H, with
    the larger features going to the backlog;
  - one-body phase space is high priority;
  - `mlm-ptj-below-xqcut-refused` is raised to medium;
  - `majorana-fermions-unsupported` is raised to high, after v0.2 (recorded
    by the CERN host's census).

## Items

**Closed:**
- `xpt-nonpositive-refused` and `pt-min-pdg-refused` (#94; their files left main with this record);
- `decay-chain-duplicate-subprocess-vs-madgraph` (#93);
- `massive-incoming-partons-refused` (#95);
- `explicit-initial-state-ordering-counted-twice` (#95 with #96);
- `absent-run-card-maxjetflavor-ignores-beam-flavours` (#101);
- `five-flavour-ppjjj-refused-cut-indicator-differs` (#104);
- `light-quark-qcd-sigma-high-vs-madgraph` (MadEvent's side);
- `odd-gluon-vertex-sign-unmeasured` (#108).

**Left open:**
- `crossing-pair-colour-basis-differs-refused`: narrowed to the colour-label
  check on a real MLM survey card.
- `ckm-flavour-groups-near-degenerate-refused`: narrowed to the unattributed
  3.88e-8 separation (card 089).

## Lessons

- **Lifting a refusal can expose a wrong answer that was always there.** The
  first campaign after the lifts showed 7 cards off by a steady factor. None of
  the causes was in the lifting code: two were silent bugs older than the
  sprint, and four of the cards had a divergent σ. When a lift's oracle is not
  the survey card itself, run the lifted cards against MadGraph before merging,
  and read a steady factor as a bug until it is shown otherwise.
- **Separate a bug from sampling with the budget ladder.** Three
  constant-looking factors split three ways:
  - TCP held at 4.2× with the spread shrinking: a bug.
  - VBF_HH rose and blew up its χ²/dof: under-sampling.
  - The photon cards kept rising with no plateau: a divergence.
- **Diagnose on the finest oracle available.** A, A′ and B each closed on a
  finer quantity than σ: a per-diagram sign, a per-part mirror/direct ratio,
  and a draj ladder in both programs.
- **Derive a convention rule rather than fit it.** For A, three candidate rules
  fitted every reachable vertex. The derivation (all-outgoing against
  all-incoming momenta), and a CP-even control that the candidates disagreed
  on, decided it.
- **Diagnosing on two hosts works when both stay disciplined.** The CERN host
  took the gridpack-bound work and A; this container took B, the mirror and the
  guards. Every claim crossing over was spot-checked in the code.
- **Branch from `origin/main` after a fetch.** Use one target directory per
  worktree. Check that a check can fail: an events hash was once the SHA-256 of
  empty input. Don't pin the last ulp. Re-gate after every merge: a semantic
  conflict passed a clean text merge.
- **Briefs can be wrong; let sessions say so.** D5's premise and B's working
  hypothesis were both wrong. H-C2 and H-B1 stopped and reported instead of
  building on them.
- **One heavy job at a time on a small container.** It restarted five times,
  each under parallel heavy load. After the fourth, committing and pushing at
  checkpoints meant the fifth lost almost nothing.

## Filed from the sessions and the campaign

Filed in four batches. All are on `sprinth-kb` (#102).

- **The sessions' Found items, first triage (23; 2c6cb303).** Highlights:
  - `ptj-n-min-nonpositive-refused`
  - `negative-ufo-coupling-orders-refused`
  - `zero-shat-floor-panics-with-nan-x`
  - `mxx-only-part-antipart-silently-ignored`
  - `quoted-sigma-error-uncalibrated-on-multijet-rows`
  - `massive-incoming-partons-ungated-beyond-sigma`
  - `madgraph-defects-rows-missing-from-sprint-h`
- **The diagnosis round (15):**
  - high:
    - `spacelike-floor-cuts-phase-space-below-the-largest-pt-threshold`, a silent bias of about 6% at `ptj 0.1`;
    - `one-body-final-state-phase-space-missing`, high by the user's decision;
    - `uncut-collinear-limits-beyond-photon-quark-unrefused`;
  - medium:
    - `mass-split-parts-thin-the-per-group-budget`
    - `rayon-worker-panic-leaves-process-alive`
    - `param-card-entries-for-absent-particles-refused` (MadGraph ignores such entries)
    - `gpdb-compares-sigma-after-physics-changing-patches`
    - `idm-two-to-four-reads-one-percent-high`
  - low:
    - `niter-3-campaigns-have-no-chi2-check`
    - `fermion-vertex-momentum-parity-compensation-unexplained`
    - `sm-alp-higgs-gluon-contacts-unreached`
    - `madevent-exchanged-massive-parton-anchor-unverified`
    - `small-width-treatment-non-default-refused`
    - `build-sh-links-no-cxx-runtime-on-linux`
    - `validate-hadronic-mirror-control-doc-describes-history`
  - `flavour-group-splits-not-recorded` was extended to cover the missing map
    from group to subprocess in the run output.
- **The CERN host's refusal census at 202a540 (12; 7a3959ed), folded in:**
  - `coloured-scalar-seagull-colour-unsupported`
  - `proc-card-import-model-options-refused`
  - `coupling-orders-not-inferred-without-coupling-orders-py`
  - `multijet-mlm-cards-time-out-in-the-gridpack-campaign`
  - and eight more, ranked in [the census](../../validation/cms-gridpack-refusals-202a540.md).
- **The defects catalogue:** `madgraph-defects.md` gained rows for:
  - the `pt_min_pdg` light-code guard;
  - the `setcuts.f:681` index;
  - MadEvent's under-sampling at large √ŝ;
  - the unreadable wrapped proc card;
  - the `cut_decays` stricter-cut approximation;
  - MadEvent's σ on an uncut collinear pole.

Not filed:
- **TCP_m55's +2.5%:** it reads 1.022 ± 0.018 in the closing rerun.
- **The fixed-beam MLM `unreachable!`:** `ScaleChoice::compile` already
  refuses that case.
- **The amplitude residuals after #108:** H-A2 traced the cHG one to the Higgs
  width on spacelike lines.
