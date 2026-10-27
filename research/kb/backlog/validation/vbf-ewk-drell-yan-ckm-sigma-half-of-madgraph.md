---
type: Backlog Item
title: VBF and EWK Drell-Yan on loop_sm-ckm_no_b_mass read about half of MadGraph's σ
description: "Three CMS VBF/EWK l+l- jj gridpacks, newly unblocked by the mass split, give 0.46-0.56 of MadGraph's cross section on one seed; a missed region or a real bug, unsettled."
area: validation
state: open
priority: high
closes_when: "The three gridpacks are run at ≥5 seeds and a budget large enough that the seed spread and χ²/dof are read, with the two CKM-restriction controls below. Either σ agrees with MadGraph within the measured spread, recorded with the runs and why the single seed was low, or the cause is found, fixed, and a banked validation row of this process class (VBF/EWK l+l- jj with an mmjj cut) gates σ against MadGraph."
blocked_by: []
opened: 2026-10-27
tags: [sigma, ckm, vbf, drell-yan, cms-survey, madgraph-comparison]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-27}
sources:
  - {id: pr71, resource: "https://github.com/nsmith-/vibegraph/pull/71#issuecomment-6432085527", title: "PR #71: main-6cb9756-200 re-run of the CMS gridpack sample, section 4"}
---
The CMS-gridpack re-run `main-6cb9756-200` (#71, build `refdata-9-61-g6cb9756d`)
has three SM-class gridpacks well below MadGraph's σ. All three:
- became runnable through the outgoing-mass split (#72);
- run as shipped on `loop_sm-ckm_no_b_mass` with PDF 325300 and an `mmjj` cut
  (4 or 100 GeV).

| gridpack | σ / MadGraph | pull | rel. err |
|---|---|---|---|
| `DYto2L-2Jets-EWK_Bin-MLL-50` | 0.563 | −15.4 | 5% |
| `VBFto2L_Bin-MLL-50` | 0.457 | −8.3 | 14% |
| `VBFto2L_Bin-MLL-4to50` | 0.534 | −1.0 | 85% (unconverged) |

**What it is not yet.** The campaign ran one fixed seed at neval 10 000 × 3
iterations. A VEGAS run that misses a region reports a confidently wrong low σ
(AGENTS.md, "Samplers gate statistically"). So this may be phase-space coverage
of a 2 → 4 VBF topology at a small budget, not a matrix-element fault. The SM
rows in the same sample agree: `ewkzlljj_5f_LO` (also `ckm_no_b_mass`, also
split) reads 1.005.

**Where to look, in order:**
1. **Seeds and budget.** Run ≥5 seeds at rising budget. If σ climbs toward
   MadGraph with budget, it is coverage; record the budget at which it settles.
   If it migrates between seeds instead of shrinking, it is a bug.
2. **Per-part σ.** Compare per mass part (the #72 split) and per MadEvent P
   directory, which `pp_to_ll_etau` already does, to localise a missing piece.
3. **Controls:**
   - the same process on plain `sm`, which rules the restriction in or out;
   - the same process without the `mmjj` cut.
   `ubx_to_epve_ckm_no_b_mass`'s amplitude cell is still `info` on a 42° V_ub
   phase (#62); a phase is invisible to |M|² for a single CKM element, so it is
   not an explanation by itself, but it is the open item on this restriction.
