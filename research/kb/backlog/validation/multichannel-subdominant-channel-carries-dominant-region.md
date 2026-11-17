---
type: Backlog Item
title: "A subdominant channel's map alone reaches a region of the dominant diagram, and its grid never adapts there"
description: "TprimeBToTZ M1800 reads ~3% low with a small quoted error on 4 of 5 seeds: only channel 4 (diagram share 0.6%) maps onto a large-|M|² corner of configuration 2, and it samples it as rare heavy points."
area: validation
state: open
priority: high
closes_when: "The region is named from per-event output. Multichannel integration then reaches it on every seed at the campaign budget (neval 120000, target_rel 0.01): ≥ 5 seeds, unweighted mean within MadGraph's error, scatter χ²/dof in band, and the quoted errors calibrated. Banked rows are unchanged within error."
blocked_by: []
opened: 2026-11-17
tags: [sampling, multichannel, vegas, sigma, error-estimate, confidently-wrong, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Reported by the CERN host from its v0.2.1 gridpack check campaign, on gridpack
126: `TprimeBToTZ_LH_twentypctLO_M1800`.

**Card.** The process is `p p > z tt bb j / bp x y QED=1 QCD=1`, with
`VLQ_v4_4FNS_UFO-3rdL_noCKM` and `lhaid 325500`. MadGraph's σ is 2.504e-4 ±
2.3e-6 pb.

**Seed spread.** At `--neval 120000 --target-rel 0.01 -j 4`, seed 1 hits and
seeds 2–4 read low, stopping at 6 iterations. The `inspect --ranking` output
puts the missing ~5% on configuration 2, the dominant diagram (share 0.637).
That diagram's extra σ on the hit seed comes from points drawn in channel 4,
which carries propagators tp, tp~ and W−. On seed 1 channel 4 gives 2.32e-5 pb;
on the misses it gives 3.4–5.2e-6. Its inter-iteration χ²/dof is 190–5500.
Seed 5 misses another way, with an excess in channel 1 (χ²/dof 211).

The region is **not yet named**. The host's guess is the below-peak tail of the
broad T′ (Γ/M = 20%).

**Knob sweep, 5 seeds each:**

| variant | hits | mean/MG | scatter χ²/dof | cost |
|---|---|---|---|---|
| base | 1/5 | 0.969 | 29.6 | 1× |
| `--min-iters 20` | 2.5/5 | 0.981 | | 2.4× |
| `--alpha-floor 1e-3` | 2.5/5 | 0.981 | | 1.1× |
| `--neval 480k` | 4/5 | 0.990 | 108 | 2.5× |
| `--neval 1.92M` | 5/5 | 1.004 | 9.8 | 12× |

The seeds that hit change with the knob, which AGENTS.md treats as the
signature of a bug. Channel 4's α is already 0.014–0.045, so an α floor cannot
help. Even at 16× the evaluations the quoted errors are understated.

**Candidate fixes:**
- weight the α survey by each channel's measured variance contribution;
- give each channel an adaptation-budget floor;
- draw in channel 4 with the dominant diagram's multichannel weight.

The artifacts are on the CERN host in `/home/ncsmith/bench-v0.2.0/diag-tprime/`.
A related one-point σ is
[`ttz2b-sigma-carried-by-one-point`](../performance/ttz2b-sigma-carried-by-one-point.md).
