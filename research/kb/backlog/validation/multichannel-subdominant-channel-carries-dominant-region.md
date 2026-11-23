---
type: Backlog Item
title: "A transfer bound cut the dominant channel's b-collinear corner out of its map"
description: "TprimeBToTZ M1800: an uncut b̄'s rung was bounded at −ptj², so most seeds missed its collinear corner and read ~5% low. Fixed; open until a same-scheme MadGraph σ confirms."
area: validation
state: open
priority: high
closes_when: "Multichannel integration reaches the region on every seed at the campaign budget (neval 120000, target_rel 0.01): ≥ 5 seeds, the unweighted mean within the error of MadGraph's σ with the complex-mass scheme off (the scheme vibegraph runs), scatter χ²/dof in band, and the quoted errors calibrated. Banked rows are unchanged within error."
blocked_by: []
opened: 2026-11-17
tags: [sampling, multichannel, phase-space, sigma, error-estimate, confidently-wrong, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Reported by the CERN host on gridpack 126, `TprimeBToTZ_LH_twentypctLO_M1800`.
The card runs `p p > z tt bb j / bp x y QED=1 QCD=1` with
`VLQ_v4_4FNS_UFO-3rdL_noCKM` and lhaid 325500
(`NNPDF31_nnlo_as_0118_nf_4_mc_hessian`). The b quark is uncut: `ptb` is 0 and
`maxjetflavor` is 4.

**The miss.** At campaign budget, seeds fall into two populations:
- low: σ about 2.40e-4, small quoted errors;
- high: σ 2.47–2.55e-4.

Budget moves seeds from low to high and never back, so the low seeds were
confidently wrong.

**The region and mechanism** (sprint L PR 5, from a per-point probe):
- The missed weight is configuration 2's own collinear corner: g → b b̄ with
  the b̄ along the gluon beam, at |t_b| < 100 GeV². It is not a T′ tail.
- `spine_chain` bounded every spacelike rung at t ≤ −ptj² (the largest
  single-leg pT cut on any leg), even where the rung's side is the uncut b̄.
  Configuration 2's own channel therefore had zero density where its |M|²
  peaks, and only channels 4 and 1 reached the corner, as rare heavy points.
- The candidate fixes recorded here before (α weighting, budget floors,
  drawing in channel 4 with configuration 2's weight) could not have closed
  it: the channel that owned the corner had no density there.

**The fix.** A peripheral rung whose side is a single final leg is now bounded
by that leg's own pT_min², or not at all if the leg is uncut. The bound is
proven: for an end rung, p_i⁺ ≤ p_a⁺ gives |t| ≥ p_Ti². At campaign budget,
20 seeds now sit in one population (mean 2.5214e-4, scatter χ²/dof 1.54).

**What is left.** The closing comparison needs MadGraph's σ in vibegraph's
scheme. The gridpack's 2.504e-4 is in the complex-mass scheme, which
vibegraph's campaign run had stripped. vg-validator's same-scheme MadGraph
3.7.1 run is queued.
