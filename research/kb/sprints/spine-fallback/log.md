# spine-fallback log

## 2026-10-28
* **Creation**: sprint folder created. The user added the item to sprint E through the planner and decided T1 (fixed default scale, pole floor only), T2 (always build the spine) and T3 (the CMS ladder runs in a session here).
* T-F1 dispatched on worktree `/home/user/wt/T`, branch `sprinte-spine-fallback`, from main 44bf09c9.
* 2026-10-28: T-F1 checked (c5d44b86, 1ed01f9f). Manager re-ran fmt and `diagram_channel` (26 passed). Composite-subsystem bias check passed, so T2's fallback is not needed. Default pole scale 1e4 GeV².
* Rule 3 of artifact-format-versioning (bump on a meaning change) applies: T-F1 changed which channels a grid maps through without a version bump. T-V1 fixes it first.
* The CERN host supplied the three CMS cards (branch `vbf-dy-spine-inputs`, 4cc4011b). They set `etaj = -1`, `ptj = ptl = drjj = 0` and move the param card (aEWM1, MZ, WZ, MW, MT, WT, ymt). So the ladder needs #82, and T-F1's hypothesis needs answering first: is σ regulated at all with massless quarks and a t-channel photon?
* 2026-10-28/29: T-V1 checked (4fdc92b9, a09a7806). Manager verified MadGraph's 2 GeV μF floor at `reweight.f:1205-1218` (weight zeroed at `:1907`) and vibegraph's `MUF_FLOOR` at `setclscales.rs:43/508`. The CMS σ is regulated by it on both sides. The user decided to add the channel-map digest with the version bump (done in 4fdc92b9). If the CMS σ had been ill-defined, the user's fallback was to close on a test that catches the original defect.
* 2026-10-29: the user chose to run T-V2 in a throwaway local merge of #81 and #82 (`/home/user/wt/TV2`, branch `scratch-t-v2`, never pushed), measurement only.
