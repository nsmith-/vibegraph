---
type: Backlog Item
title: "ptl1min–ptl4min and ptl1max–ptl4max with real values are refused as unimplemented"
description: "Seven CMS ZH_Hto2B_Zto2L_SMEFTsim_topU3l cards set ptl1min = 10; vibegraph refuses it, which keeps SMEFTsim_topU3l out of simshop's registry."
area: feature
state: open
priority: medium
closes_when: "ptl1min–ptl4min and ptl1max–ptl4max are implemented as MadGraph's cuts.f lepton rank cuts (charged leptons ranked by pT; applied in the τ floor only where setcuts.f does), with σ on ZH card 832 or 837 at its real ptl1min = 10 against MadGraph by the multi-seed criterion, and the seven cards integrate unpatched."
blocked_by: []
opened: 2026-11-18
tags: [cuts, run-card, refusal, cms-gridpacks, simshop, smeft]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-18}
---
vibegraph lists `ptl1min`–`ptl4min` and `ptl1max`–`ptl4max` in
`UNIMPLEMENTED_CUTS` (`vibegraph-lib/src/cuts.rs`) and refuses them whenever
they would cut. The jet family is
[`rank-jet-pt-cuts-unimplemented`](rank-jet-pt-cuts-unimplemented.md). Both
use the same mechanism and are meant to land together.

**Evidence.** The CMS gridpacks `ZH_Hto2B_Zto2L_SMEFTsim_topU3l_<point>`
(CERN host IDs 832–838) all set `ptl1min = 10.0`. With `ptl1min` patched to 0
they integrate, and agree with MadGraph at 5 seeds (|mean pull| 1.1–1.6). This
refusal is the only thing keeping SMEFTsim_topU3l out of simshop's registry.

**The cards** (simshop's export of the CERN host, 2026-11-17, sha256 c05d4ed4…):
- Process: `p p > z h SMHLOOP=0 NPprop=0 NP<=1, z > l+ l- , h > b b~`.
- Restriction: `massless_VH`.
- Beam: `ebeam` 6800.
- PDF: lhapdf 325300, `maxjetflavor` 5.
- Lepton cuts: `ptl1min` = 10; `ptl` = `dral` = `drll` = `drjl` = 0; `etal` 3.0; `mmll` 10.0.
- Jet and photon cuts: `ptj` 10, `drjj` 0.001; `pta` 0, `etaa` −1, `draa` 0.
- Also: `auto_ptj_mjj` False, `sde_strategy` 2.

The points are 832 SM, 833 HB0p45, 834 HB0p18, 835 HW0p16, 836 HB0p9, 837 HW0p4
and 838 HB1p35. 833 differs: lhaid 325500 and `maxjetflavor` 4. Use 832 or 837
for the gate.

**MadGraph references at `ptl1min` = 10.** These were regenerated on the CERN
host with MadGraph 3.7.1 on the identical cards, 50k events each, in
`/home/ncsmith/bench-v0.2.0/diag-smeft/run-*-ptl10`:

| Card | Regenerated σ (pb) | Gridpack's own σ (pb) |
|---|---|---|
| 832 SM | 0.07481 ± 0.00005 | 0.07513 ± 0.00036 |
| 837 HW0p4 | 0.09970 ± 0.00006 | 0.10010 ± 0.00053 |

The `ptl1min` = 0 values are in that directory's RESULTS.md. Use them to check
that the cut actually changes σ.

**Not in scope.** The cards constrain `NPprop`, which `massless_VH` removes
entirely. vibegraph accepts that as vacuous, as MadGraph 2.9 does. MadGraph
3.7.1 refuses it. That behaviour stays as it is here.
