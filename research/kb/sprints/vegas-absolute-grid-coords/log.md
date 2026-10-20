# vegas-absolute-grid-coords log

## 2026-10-16
* **Creation**: Sprint folder created by the sprint manager from the planner's
  hand-off "statistical gates and seed calibration" (planner session
  `session_014wzXyZJfXdGTAVjvtKqbZM`). Draft PR opened as the claim.

## 2026-10-20
* **P1 dispatched**: #48 is on main, so the binning rework can start. Main
  `e3f543f` merged in at `90b6215`; [P1's brief](sessions/P1.md) is the inert
  plumbing (contract inversion and fixed-transform maps), banked layer identical.
* **P1 paused, the user's decision (2026-10-20, relayed by the planner).** The
  user wants to design multi-tier VEGAS before more plumbing lands. The
  candidate is a shared outer tier (τ/y, absolute coordinates) common to all
  channels, per-channel inner tiers, the K bound possibly per tier, and
  non-aligned cuts (pT, ΔR, HT) left to the pluggable-sampler work. That could
  change the interface P1 was building. P2 waits on the planner's design note and
  a session with the user.
  - P1's work in progress (`3b23d87`, untested beyond `cargo check`) is on the
    branch `vegas-absolute-grid-coords-p1-wip`, not on this PR.
  - [P1's report](sessions/P1-report.md) holds its findings and the P2 design
    note.
  - The pre-edit baseline banked run is at `a8b1427`, with its collator output in
    the manager's scratchpad as `gridcoords-P1-baseline-report`.
