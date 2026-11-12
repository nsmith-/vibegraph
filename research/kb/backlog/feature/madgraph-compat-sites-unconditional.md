---
type: Backlog Item
title: MadGraph-compatibility choices are unconditional
description: Several sites reproduce a MadGraph choice a clean design would not make; with no switch, the cleaner behaviour is unreachable and its cost unmeasured.
area: feature
state: open
priority: medium
closes_when: "A --madgraph-compat flag (default off, recorded in the integrate artifact and LHEF header) exists; the banked MadGraph gates run with it on and every banked byte and σ gate is unchanged, and a documented per-site delta table gives what turning it off changes."
blocked_by: []
opened: 2026-09-06
tags: [cli, madgraph-parity, lhef, alphas, kt-clustering, cuts, decay-chains, user-request]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L643-L667", title: "TODO.md entry T058"}
---
A user request on PR #4. One flag, recorded in the artifact and LHEF header.
It **defaults to off**, for every site: vibegraph's own behaviour is the default,
and the banked MadGraph gates run with `--madgraph-compat` on (the user, sprint
I [D3](../../sprints/v02-last-refusals/decisions/D3-identical-decays-behind-madgraph-compat.md),
2026-11-08; earlier the plan was default on). Each site
gets a "MadGraph compatibility" admonition in the docs. Sites:

- (a) `vibegraph-lib/src/coupling/cluster/kt.rs`: the `1 + 1e-6` crossed
  beam–leg inflation (`TIE_BREAK`), which does not cancel when every admissible
  candidate is crossed and so leaks a part in 1e6 into `SCALUP`, and the
  first-pair-in-visit-order tie-break. Off: an inflation-free measure and a
  tie rule that cannot enter the value.
- (b) `vibegraph-lib/src/coupling/alphas.rs`: `CMASS = 1.42`, `BMASS = 4.7`,
  `ZMASS`, the `TOL = 5e-4` Newton stop (a specific iterate, not the root) and
  the transcribed β-function constants. Off: the model's quark masses, computed
  constants, and an ODE solver run to convergence.
- (c) `lhef/mod.rs` + `lhef/write.rs`: the Python post-processor column layout
  and the two-dialect re-emission keeping a Fortran-dialect file's seven
  significant digits on the scale and coupling columns. Off: one layout at
  full precision.

Not sites, leave alone: `AQCDUP` is already untruncated, the jet-count memo
is not carried across events, `SCALUP = max(μF)` is the accord's definition,
and the truncated `w_max` rule is an improvement. No validation gate runs in
the off mode.

The flag also carries the MadGraph defects that are easy to reproduce
([defect policy](../../validation/madgraph-defect-policy.md)): on, the defect
is reproduced bug-for-bug; off, the more robust calculation runs. A defect
upstream fixes leaves the flag's scope when the MadGraph pin moves past the
fix.

- (d) identical particles across decays, with the `cut_decays = F` exemption
  folded in (formerly site (g); [concept](../../process/identical-particles-across-decays.md)).
  - **On:** MadGraph's single pairing per block, its
    `identical_decay_chain_factor`, its per-pairing window and its
    configuration-1 exemption (`check_decay`, `setcuts.f:961`) together. It also
    evaluates fewer diagrams. MadGraph's pairing must be carried out of
    `stitch_set` (the forced-line patterns cannot name it).
  - **Off (the default):** every pairing coherently, the window union, and
    `cut_decays = F` through the pairing-weighted cut Σ_p w_p Θ_p |M|²
    (implemented in v0.2.1, sprint J).
  - **Size:** +1.72% (F) / +1.39% (T) on `g g > z u u~, z > u u~`, +0.44% on
    `g g > z j j, z > j j` at F (sprint J, J-W1): per event +0.78% interference
    and +0.86% window union, zero from the exemption.
  - **History:** registered deviations (user, 2026-09-26 and 2026-11-06);
    D3 (2026-11-08) puts MadGraph's behaviour behind the flag; option A (user,
    2026-11-11): no flag is built in sprint J, and MadGraph's single pairing
    stays this site.
  - **No exemption on the full every-pairing |M|² is IR-safe.** MadGraph's
    configuration-1 legs (H-C2) and the kinematic window rule (I-C1) both leave
    an exempt quark uncut where it is a core quark of another pairing.

- (e) the restriction's zero-coupling rule (`ufo/mod.rs` `is_zero_coupling`):
  on, MadGraph's `detect_identical_couplings` rule, |g| < 1e-13 is zero with a
  strict-zero rerun when any coupling lies in [1e-13, 1e-10); off, a rule that
  does not depend on another coupling's value (exact zeros only, or a fixed
  relative threshold). Added at the user's request (sprint G, 2026-11-03).

- (f) MadEvent's small-width convention: for Γ below M·1e-6 MadEvent evaluates
  the propagator at fk_W = M·1e-6 and corrects σ through the mapped channel's
  Jacobian (`transpole.f:44`), broadening the lineshape; vibegraph uses the
  card's width. About 1.5 % on σ for a 1 MeV, 3.5 TeV resonance (sprint G,
  G-V1). On, emulate it; off, the card's width. Registered as a deviation by
  the user (2026-11-04).
