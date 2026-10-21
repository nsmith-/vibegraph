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
* **M1 done** (`ffefe94`, [report](sessions/M1-report.md)). The measurements
  feed `sampler-and-phase-space-seams-designed-apart`; no production code
  changed (the probe is a `#[cfg(test)]` module inside `vibegraph-lib`, because
  rejected points' momenta and grid edges are crate-private).
  - On `pp_to_jj`, about 0.7 of the top-0.1% second moment sits within 5% of a
    cut edge, mostly jet pT, which τ alone pins (a D window).
  - On `pp_to_llj_fixed` and `pp_to_llj_mlm`, 53–76% of the tail is away from
    every edge, and lepton η aligns with no coordinate (C).
  - No high-weight edge is fixed in absolute coordinates.
  - The y grids agree across channels; starved channels' τ grids differ from
    the pool at 1.8–2.8× seed noise.
  - A rejected point costs 0.06–1.7% of the evaluation time, and removing
    rejection could buy at most 1.3–1.5× in variance × CPU. That corrects the
    design draft's "tends to 1".
* **Close-out, the user's decision (2026-10-20, relayed by the planner): #49
  merges the probe and the M1 report.** It closes no backlog item, and
  `vegas-grid-coords-not-absolute` stays open for the design session to
  rewrite. M1's Found 1–4 are folded into
  `sampler-and-phase-space-seams-designed-apart` as inputs. Found 5 is filed as
  `resolve-mom-dead-code-under-release-debug-tests`.
