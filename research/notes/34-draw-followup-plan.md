# 34 — Draw-performance sprint record, and the follow-up plan (2026-08-06)

Two parts. §1 is the record of the **draw-performance sprint** — two parallel
performance-dev sessions off `main@4d03400`, which had no note of its own.
§2 is the follow-up plan: four sessions in two waves, every item unblocked or
re-scoped by §1's reports. The TODO backlog entries remain the per-item
authority; this note adds the record, the sequencing, and the session
boundaries.

## 1. Sprint record — draw performance

Born from the convergence-and-2to3-abort pair's llj diagnosis (cut-edge
variance, note in TODO's closed-sprint history) and three user directives: a
cut point contributes exactly zero, so tighten the original draw from the
cuts, and make the single-channel draw fast enough that low cut efficiency is
affordable.

### 1.1 `density-draw` — cut-first short-circuit (`470eb8f`, merged `443f6bc`)

The samplers now separate the draw from its weight
(`draw_from`/`draw_in_channel`/`channel_weight`/`mixture_weight` on
`MultiChannel`, `ScaledMultiChannel`, and both integrand samplers), and every
consumer prices the `Σⱼ αⱼgⱼ` mixture density only *after* its cut and matrix
element — a rejected point never evaluates the loop. The audit that no path
consumes a rejected point's weight (including unweighting trial accounting)
was done before restructuring, and channel identity/RNG streams are
untouched, so the `e059092` channel↔diagram consumer audit stands.

**Order-preserving, proven**: byte-identical `IntegrateArtifact` SHA-256 at
fixed seed on the 4-, 24-, and 579-channel rows; full `validate` green; the
2→6 long-tier rows reproduce their manifest-recorded σ exactly. Measured:
`probe_2to6_eval_cost` **62.9/68.5 → 4.6/6.8 µs (13.7×/10.1×)** on the
579/615-channel rows, α-survey wall 25.7→8.8 s / 31.6→15.6 s, `pp_to_llj`
−6%, DY flat. Residual density share ≈35% of the accepted point — *inferred*
from two totals, not instrumented (§2's S4 builds the instrument).

Mechanism 2 (zero-support early-out) was **killed on inspection**: the exit
already sits at the earliest knowable rung in `spine_jacobian`, and 339 of
the 579 channels are all-timelike with no zero-support path at all.
Mechanism 3 (jacobian memoisation) was deferred to §2 S4. Brief corrections:
the real fresh-grid multichannel acceptance is ≈3% (the brief's 46% was the
flat-RAMBO sweep — why the result is 13.7× and not ~2×), and note 32's S1 had
moved the cut ahead of the *scale* draw only, not the phase-space density.

### 1.2 `timelike-floor` — cut-implied floors for the invariant draw (merged `7664ff9`)

`Cuts::timelike_floor(slots)`: a provable lower bound on a final-state
subsystem's m² over the accepted region, installed by
`DiagramChannel::with_timelike_floors` as the `lo` of every drawn timelike
invariant through one shared `draw_lo` used identically by sampling and
jacobian — reciprocity structural — wired at all three production sites and
the proton oracle, with `map_key` extended. The bounds and proofs (in the
doc comment): subsystem monotonicity; normal pair-mass windows only (a veto
band implies nothing — pinned); `m² ≥ 2pT₁pT₂(cosh Δy − cos Δφ)` minimised
on the `ΔR` circle to `g_min = 1 − cos R`; and a proof the ŝ refinement
never fires. llj floors: 15.788/31.576 GeV²; the `mmll=50` card 2500 GeV²
exactly, attained within a factor 1.0002 — the bias oracle
`no_accepted_configuration_sits_below_a_subsystem_floor` (40k RAMBO draws ×
every subsystem mask) is the keeper for any future floor.

**Classification statistical**, with a blast-radius correction to its own
brief: a 2→2 final state draws no timelike invariant, so the floors are
provably inert there (pinned by test; `dy13`/`pp_to_jj` σ digit-identical) —
only 2→3-and-up rows move.

**Falsifier, decisive**: `pp_to_llj_dyn` `m_ll [40,70)` var/σ **16.86 →
4.36** (max bin 1.47), top-0.1% second-moment share 80–92% → 51/40/30%,
Hill index 10/12 σ-carrying channels above 2, trained χ²/dof 4.02 → 1.80,
acceptance 22.5% → 33.8%; `pp_to_llj` error² −50%. Two-arm five-seed
ladders show **no bias** (8 rungs within 0.8 sd, variance −4–24%, scatter
shrinking with budget — the statistics criterion, not the bug one).

**Payoffs split**: `--target-rel 0.001` on llj still caps 3/3, but
error² × CPU improves 28.8% and the achieved-δ seed spread tightens 2.4×;
the recarded σ ladder climb **survived** — the "one defect" hypothesis is
falsified and the climb is now an unexplained drift (§2 S2). One residual
worsened: with a small floor the map's lower edge lands on the cut edge and
the leftover `ΔR`/`pT` boundary concentrates there (`m_ll [0,5)` var/σ
24.5 → 55.7) — the deferred map-shape question (§2 S5).

**The gate cascade, and the three decisions.** Because cargo stops at the
first failing test binary, the branch unmasked three at-threshold gate cells
one at a time (24 → 30 → 33 → 35 binaries reached), and all three were the
same defect wearing different clothes: **a gate statistic formed on fewer
seeds than AGENTS.md's own ≥5 standard, sitting at its threshold, re-rolled
by any sampling-stream change.** In each case the session measured causation
two-arm before proposing anything, refused to widen, and escalated.

1. **llj_dyn's 3-seed scatter guard** read χ²/dof 4.24 against
   `LLJ_MAX_CHI2_PER_DOF = 4.0` — a threshold calibrated on five-seed
   ladders — while the five-seed reading at the same budget is 2.49 and σ
   moved *closer* to MG (+0.25% → +0.06%). User decision: form both llj
   gates over the five calibration seeds (`ca4d11f`). Final readings:
   `llj_fixed` 423.747 ± 0.299, χ²/dof 1.86, rel −0.02%; `llj_dyn`
   415.928 ± 0.293, χ²/dof 2.49, rel +0.12% — each reproducing its ladder's
   150k rung digit for digit.
2. **`ee_to_mumua`'s samples KS** crossed `P_FLOOR` on one of three seeds
   (p 8.3e-6 on `pt(a) ≡ pt(ll)`) with the shape agreement unchanged
   (mean D 0.0241 → 0.0242, two of three seeds improved). TODO's watch item
   had pre-registered this flap with its diagnosis context; `P_FLOOR`'s own
   prescription was applied verbatim: cell → info with the measurement and
   the chain-D record in its note, threshold unmoved (`71d47d1`).
3. **`ee_to_mumua`'s σ pull** failed at 3.56 on the gate's *single* seed —
   and the five-seed sweep showed the row's true state is a fixed **+1.04%
   on both arms** (pulls +4.11/+4.17; the floors move σ by +0.006%): the
   base had been passing at +2.83 on a seed 0.089% below its own mean. The
   row joined `PULL_REPORTED_NOT_ASSERTED` by that category's own recorded
   criterion (a systematic of measured size, here reference-adjudicated by
   chain D), `rel_tol 0.03` still enforced (`ec2c5a0`).

Close-out: `validate` exit 0 over all 35 binaries, census 95 ✅ / 3 ⚠️ /
0 ❌ (the one movement being decision 2), **merged at `7664ff9`**. En route
the session also repaired two pre-existing test fragilities (a lab-frame
balance bound measured below its own width on unmodified `main`, and a
luminosity-share test pinned to one lucky event) — neither a tolerance
weakened for the floors. Filed for the validation backlog: the remaining
single-seed `integrate_reported` gates are the same shape of exposure and
have not been swept.

Transferable lesson, continuing the note-32 pattern: **both sessions
corrected their own briefs** — an acceptance sizing wrong by an order of
magnitude, a blast radius wrong in the safe direction, and a pre-registered
payoff that failed while the falsifier passed, which is exactly why falsifier
and payoff were registered separately.

## 2. Follow-up plan

State the plan assumes: the cut-first short-circuit is in production (mixture
density priced only on accepted points; 2→6 per-point 4.6/6.8 µs), and the
cut-implied timelike floors are merged (per-channel variances on every
2→3-and-up hadronic row differ from every pre-floor measurement — nothing
below may cite a pre-floor baseline).

### Wave 1 — dispatch after `timelike-floor` merges

#### S1 — α-survey: one density pass, then the budget constants, then the stop-factor calibration (performance-dev)

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

#### S2 — the recarded `pp_to_llj` ladder climb: diagnosis (validation-dev)

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

**S2 close-out (2026-08-06, `5a3b837`, merged `119e6d3`): the climb was a
misread, not a drift.** The 40-seed-per-rung ensemble puts the estimator's
expectation flat from 150k up — the drift hypothesis dies at 7.3σ, with the
observed 150k→600k step carrying the wrong sign (−0.19 ± 0.23 pb) — while
the recorded ladder is one five-seed draw 2.3σ low at its bottom rung
(1 of 8 disjoint quintets reproduces the pattern; 3 step monotonically the
other way). The instrument was the defect: five-seed scatter understates
this row's measured per-seed spread (sd 1.378 pb at 150k) by 2× at 150k and
5× at 600k, where it produced a χ²/dof of 0.03 — as loud a warning as 4.0,
and read as reassurance. Reference side: MadGraph's `x²/σ²` last-3
combination pulls its own run down 0.146% by down-weighting the iterations
that caught the weight tail; our converged value is +0.09% from its
iterations recombined by point count. Every alternative excluded by a
dedicated measurement (equal-kept controls, budget-independent α survey,
the `mmll = 50` twin stepping identically, `w_max` absent from the path).
Falsifier for the verdict: `probe_llj_seed_ensemble`, ~0.1% between 40-seed
rung means overturns it. Bookkeeping: the row's manifest note re-recorded;
the pre-floor in-code ladder figures in `validate_hadronic.rs` filed in TODO
as a mechanical re-record; AGENTS.md's seed-sweep line extended with the
rung-difference caveat.

### Wave 2 — dispatch after S1 merges

#### S3 — `MIN_CHANNEL_NEVAL` counts post-cut points (performance-dev)

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

#### S4 — shared-subtree jacobian memoisation (performance-dev)

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

### Deferred — named, not scheduled

**S5, the map's lower edge on the cut edge**: with a small floor the
residual `ΔR`/`pT` cut boundary concentrates at the map edge
(`pp_to_llj` `m_ll [0,5)` var/σ 24.5 → 55.7 post-floor). Fixing it means
bounds in the other cut coordinates or a softened lower edge — a map-shape
change the floor session rightly refused. **Unblocked by S2's close-out**:
the ladder does not implicate this edge (the `mmll = 50` twin, which has no
photon-pole edge, stepped identically), so S5 prices independently on its
≈9% variance share — a candidate for wave 2 alongside S3/S4 if that share
is judged worth a session.
