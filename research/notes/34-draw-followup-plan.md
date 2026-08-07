# 34 — Draw follow-up sprint plan (2026-08-06)

Four sessions in two waves, following the draw-performance pair
(`density-draw` merged at `443f6bc`; `timelike-floor` merge pending its
5-seed-gate amendment). Every item here was unblocked or re-scoped by those
two sessions' reports; the TODO backlog entries remain the per-item
authority, this note adds the sequencing and the session boundaries.

State the plan assumes: the cut-first short-circuit is in production (mixture
density priced only on accepted points; 2→6 per-point 4.6/6.8 µs), and the
cut-implied timelike floors are merged (per-channel variances on every
2→3-and-up hadronic row differ from every pre-floor measurement — nothing
below may cite a pre-floor baseline).

## Wave 1 — dispatch after `timelike-floor` merges

### S1 — α-survey: one density pass, then the budget constants, then the stop-factor calibration (performance-dev)

Three parts, one instrument: the survey/stop statistics on wide splits.

**Part A — collapse the survey's two density passes.** Both α surveys
(`proton.rs` hadronic, `phasespace/channel.rs` fixed-energy) evaluate the
full mixture density twice per accepted point: once inside the weight, once
for the `Wⱼ` row. The density session left this alone because recovering `g`
from the returned weight is an `αⱼ/(αⱼ/g)` round trip, not an identity. The
clean fix is plumbing: the split draw/weight API can hand back `g` itself, so
one evaluation feeds both consumers — computing the identical arithmetic once
and consuming it twice is bit-preserving. Gate that claim with a fixed-seed
byte-identical artifact (24- and 579-channel rows); if the plumbing forces
any order change, classify honestly and gate at REL_TOL 1e-12 instead.
Expected win ≈2× on accepted-point survey cost; measure it.

**Part B — the budget-constants experiment**, exactly as the TODO entry
specifies (`n_survey ∈ {10k, 40k, 160k, 640k}`, α trajectory + converged α +
σ with ≥5-seed spread; rows `bbx`/`uux` 2→6 + `pp_to_llj` control + `p p >
l+ l- j j` for α stability only; verdict rule as recorded there). Run it on
Part A's collapsed loop so the measurement prices the loop that will live.

**Part C — wide-split stop scale-factor calibration**, riding on Part B's
seed sweeps at no extra integration cost: at each budget record the stop's
`scaled_rel` per seed alongside the realized seed-to-seed spread on the
579/615-channel rows, and state whether the ~20×-at-8-iterations reading is
calibrated, over-conservative, or budget-dependent. The factor is
conservative by construction (it can only delay a stop), so the deliverable
is a recorded calibration statement replacing "uncalibrated" in the TODO
entry — no code change is in scope.

### S2 — the recarded `pp_to_llj` ladder climb: diagnosis (validation-dev)

The climb (+0.04% → +0.21% over 75k–600k, reproduced post-floor as
+0.23/−0.02/+0.11/+0.27) survived a 4× reduction in cut-edge variance, which
falsified the "one defect" hypothesis: it is now an unexplained σ drift with
no candidate mechanism. Session shape:

1. **Establish it is real before explaining it.** The recorded drift is
   1.2–2.7 sd. Extend the ladder upward (1.2M, 2.4M) at ≥5 seeds per rung: a
   drift that keeps growing in sd terms is bias; one that plateaus inside the
   reference's own error may be the reference's (the chain-D precedent).
2. If real, work the finer-oracle ladder before touching σ: which per-event
   intermediate drifts with budget (the kT-clustered scale distribution, the
   flavour-group shares, the channel/partition axes chain B named, the w_max
   rule's interaction with the unweighted combination) — a per-event field is
   a finer oracle than a cross section.
3. Deliverable: a diagnosis with a falsifier, or a recorded
   reference-side verdict. No tolerance, budget, seed, or gate changes —
   the row's 150k budget note stands regardless.

Wave-1 concurrency: S1 and S2 may share the host. σ, χ², and α statistics
are load-robust; neither session may quote a wall-clock or ns/eval figure
taken while the other runs.

## Wave 2 — dispatch after S1 merges

### S3 — `MIN_CHANNEL_NEVAL` counts post-cut points (performance-dev)

The user's directive, unblocked by the cheap draw: a cut point contributes
nothing, so the 512-point floor's coverage promise should be denominated in
accepted points (count accepted draws toward the floor, or scale the floor by
measured acceptance — design choice for the session, argued from Part B's
variance data). Constraints: the floor's coverage rationale is the invariant
(a channel must keep sampling its own region); wide-split spend must stay
bounded — 579 channels × floor/acceptance needs an explicit cap with the
overshoot warning extended to it; budget accounting
(`points_per_iteration`) must stay honest. Statistical re-gate; sequenced
after S1 because it reads Part B's per-channel variance/acceptance data and
shares the allocation surface.

### S4 — shared-subtree jacobian memoisation (performance-dev)

The density session's mechanism 3, re-sized: it pays only on accepted
points, where the residual density share was *inferred* at ≈35% (≈1.6 µs of
4.6). First deliverable is therefore the instrument: a decomposition probe
that measures the density share directly (the inference is currently two
totals subtracted). Then memoise `node_invariant`/`subtree_momentum` by leg
bitmask (≤64 masks at 2→6) across channels at one point, threading a
per-point cache through `ScaledChannel::density_at`. Pre-committed kill
line: <5% end-to-end on 2→6 accepted-point cost, measured against the
instrument, kills the session cheaply — the census's 411/447 class structure
bounds sharing, not the win. Respect the `e059092` channel↔diagram consumer
audit throughout. Sequenced after S1 because Part A rewires the survey's
density call sites this cache must thread through.

## Deferred — named, not scheduled

**S5, the map's lower edge on the cut edge**: with a small floor the
residual `ΔR`/`pT` cut boundary concentrates at the map edge
(`pp_to_llj` `m_ll [0,5)` var/σ 24.5 → 55.7 post-floor). Fixing it means
bounds in the other cut coordinates or a softened lower edge — a map-shape
change the floor session rightly refused. Blocked on S2: if the ladder climb
implicates this edge, the two become one design; if not, its ≈9% variance
share prices the session on its own.
