# spine-fallback log

## 2026-10-28
* **Creation**: sprint folder created. The user added the item to sprint E through the planner and decided T1 (fixed default scale, pole floor only), T2 (always build the spine) and T3 (the CMS ladder runs in a session here).
* T-F1 dispatched on worktree `/home/user/wt/T`, branch `sprinte-spine-fallback`, from main 44bf09c9.
* 2026-10-28: T-F1 checked (c5d44b86, 1ed01f9f). Manager re-ran fmt and `diagram_channel` (26 passed). Composite-subsystem bias check passed, so T2's fallback is not needed. Default pole scale 1e4 GeV².
* Rule 3 of artifact-format-versioning (bump on a meaning change) applies: T-F1 changed which channels a grid maps through without a version bump. T-V1 fixes it first.
* The CERN host supplied the three CMS cards (branch `vbf-dy-spine-inputs`, 4cc4011b). They set `etaj = -1`, `ptj = ptl = drjj = 0` and move the param card (aEWM1, MZ, WZ, MW, MT, WT, ymt). So the ladder needs #82, and T-F1's hypothesis needs answering first: is σ regulated at all with massless quarks and a t-channel photon?
