# Resonance-aware sampling → event generation (program plan)

The last two remaining pipeline features, planned as **two sequenced feature
sprints** under the standing feature → validation → performance rhythm:

- **Sprint A — `lips-nbody` remainder: resonance-aware multichannel phase space**
  (5 sessions, L1–L5). Detailed session-by-session below.
- **Sprint B — `event-output-lhef`: unweighted event output** (4 sessions,
  E1–E4). Outlined below; expanded into its own note when it opens. B depends on
  A (genuine n-body final states + a sampler that resolves propagator peaks, so
  accept/reject efficiency is not catastrophic on resonant processes).

A validation pass may slot between A and B in the usual rhythm — the natural
subject is whatever A's sampler exposes at the gate (see §"Validation regime").

Program decisions (2026-07-21, with the user):
- **Two sequenced sprints**, not one combined (LHEF rides on the sampler).
- **Full MadGraph-style multichannel** is the target for Sprint A: per-diagram
  propagator-pole channels + Breit-Wigner mappings + the variance-minimising
  weight `1/Σᵢ(1/Jᵢ)` + α-adaptation across channels.
- **`mg-single-helicity-bench` rides with Sprint B** (session E2), where
  single-helicity evaluation through the *unexpanded* program becomes the actual
  accept/reject hot path and the MG-harness single-config timing change is on the
  critical path anyway (the A6 go/no-go deferral condition, TODO Later section).
- `dynamical-scales` is **not** in scope (separate feature; not required for
  LHEF). It stays the blocker on a real σ gate for the QCD processes.

Design inputs already gathered (TODO `lips-nbody` section; notes 01 §"Loop-Induced
Processes and Phase-Space Optimisation", 03 §1.5 Sherpa integrators, 07
phase-space/numerical hazard catalog, 11 variance↔flow duality, 18 §H3 RAMBO
seam). Reference implementations (submodules, paths in `research/refs/README.md`):
Sherpa `PHASIC++/Main/` (multi-channel adaptive integrator + separate
`Color_Integrator`/`Helicity_Integrator`), MG `madgraph/various/rambo.py` (note-07
line-218 sign bug), POWHEG `integrator.f` (MINT).

---

## Substrate already on `main` (what A/B build on)

- **Flat RAMBO** generic over `F: Real` with the KSE weight
  (`phasespace/rambo.rs`), and **splittable counter-based RNG substreams**
  (`phasespace/rng.rs`, `SubStream(seed, stream, position)`) — the sampler input
  seam (H3, note 18).
- **2-body LIPS map** + prefactor helpers (`phasespace/mod.rs`:
  `lips2_jacobian_u`, `u_to_costheta`, `prefactor2`, `GEV2_TO_PB`).
- **VEGAS as a two-phase serde object** (`vegas.rs`): `adapt` / `sample_frozen`
  (+ `_batched`/`_parallel` variants, deterministic ChaCha8 substream per rayon
  chunk). `sample_frozen` is the accept/reject primitive for Sprint B.
- **Compiled cuts** (`cuts.rs`, `Cuts::pass(&momenta) -> bool`) — already an
  accept-gate shape. **Run/proc-card assembly** + `vibegraph integrate` CLI +
  `IntegrateArtifact` (bincode+zstd: trained grid + run metadata) — the handoff
  format into Sprint B's `generate` phase.
- **Per-diagram propagator topology** is directly readable off the owned
  `Diagram` (`diagrams/diagram.rs`): `Prop { particle, endpoints, momentum:
  Vec<i8> }` gives, per internal line, the particle (→ mass + width via the UFO
  model, `UFOModel::decay_width`/particle mass) and the signed combination of
  external momenta (→ which invariant the pole sits on). `Prop::is_spacelike(n_in)`
  already separates t-channel (spacelike) from s-channel (timelike) lines. This
  is the raw material for channel construction — no new diagram plumbing needed.

Gap to close (Sprint A): the multichannel sampler, its BW/t-channel maps, the
channel-weight adaptation, and the phase-space **abstraction seam** that lets
sampler / channel-map / integrator be swapped independently (the explicit
"abstraction is the point" design constraint from the TODO).

---

## Sprint A — resonance-aware multichannel phase space

Figure of merit: **variance × CPU-time at fixed target precision** (not ns/point;
note the explicit warning in the TODO `lips-nbody` section). Every session stays
behind the 14-process `validate_helas_mg` bit-exact net (unchanged — the sampler
does not touch |M|²) and gains a phase-space-specific gate as it comes online.

| Session | Scope |
|---|---|
| **L1 `phasespace-abstraction`** | Trait seam so sampler, channel map, and integrator are separately swappable/composable. Introduce `Channel` (unit-hypercube → `n` momenta + Jacobian, on a fixed `√ŝ` and external-mass set), `PhaseSpaceMap`/`Sampler`, and a combiner interface; refactor **flat RAMBO** and the **2-body LIPS map** to sit behind it with no numeric change. **Gate: pure refactor** — DY σ(pp→e⁺e⁻) and the banked partonic σ̂ (uux 2→6 flat-MC check `rambo_oracle::flat_mc_partonic_sigma`) reproduce **bit-for-bit** (pinned seed + unchanged sampling order). |
| **L2 `diagram-channels`** | Turn a `Diagram` into a **recursive 2-body-decomposition channel tree** from its `Prop` chain: each internal line is a node parametrised by its invariant (s-channel timelike vs t-channel spacelike via `is_spacelike`), carrying the propagator particle's mass/width. Build the tree + a **flat** parametrisation of every invariant first (no BW yet). **Gate:** every channel emits on-shell, momentum-conserving points, and the flat channel Jacobian **reproduces the RAMBO phase-space volume** ∫dΦₙ = Vₙ(√ŝ; masses) for 2→2…2→6 (analytic massless volume + the banked massive σ̂). Keep a known-wrong flat-RAMBO comparison running (AGENTS.md rule). |
| **L3 `bw-invariant-map`** | The resonance mapping proper: **Breit-Wigner tan-substitution** for timelike invariants (pole at `m²`, width `mΓ`) with its exact Jacobian, plus the massless/massive **t-channel** map for spacelike lines. **Gate:** each 1-D map's Jacobian verified against the analytic BW / t-channel integral; a single-channel sampler on a resonant process (Z pole) reproduces σ at **lower variance than flat RAMBO at fixed N**, and the sampled invariant-mass histogram matches the analytic Breit-Wigner. |
| **L4 `multichannel-weight`** | Combine channels: draw channel `i` with probability `αᵢ`, weight each point by the **variance-minimising** `1/Σⱼ(1/Jⱼ)` (note 01 §"RAMBO/multichannel"; note 11 variance↔flow duality). Wire the multichannel sampler as VEGAS's integrand map (VEGAS refines the per-channel unit-hypercube on top). **Gate:** σ of a resonant/multi-peak process matches MG within MC uncertainty with **variance strictly below** the single-channel and flat samplers at fixed N. |
| **L5 `alpha-adaptation`** | MadGraph-style **α refinement** (survey → refine of channel weights, driving αᵢ toward each channel's variance share; job-strategy analogue, note 01 §A) **plus the distribution-level validation regime** (below). **Gate:** invariant-mass / angular **histograms vs MG** (not σ alone) on ≥1 resonant and ≥1 multi-peak process; α converges; documented variance×CPU improvement vs L4 at fixed precision; the note-07 sampler-bug hazard checks (BW mapping, T-channel ordering, threshold kinematics, conflicting-BW configs) each have a firing test. |

Order: **L1 → L2 → L3 → L4 → L5** (strictly linear; each builds on the prior).

### Validation regime (the load-bearing part — plan it *with* the feature)

σ-agreement is a **weak oracle** here: MG's own sampler bugs stayed latent 5–10
years precisely because a mis-sampled region of small measure shifts σ smoothly
rather than tripping a gate (note 07; AGENTS.md "every oracle has a blind spot").
So the sampler is gated at three levels, finest first:

1. **Bit-for-bit** where a pinned RNG seed + unchanged sampling order allow it
   (L1's refactor gate; any later change that must preserve order).
2. **Distribution-level** — sampled invariant-mass and angular histograms vs the
   analytic BW (L3) and vs MG's `.lhe`/plots (L5). This is what catches
   mis-sampled small-measure regions that σ hides.
3. **σ within quoted MC uncertainty** — the `validate_vegas.rs` targets + the
   banked σ̂ flat-MC check, as a coarse backstop only.

Each of the note-07 hazards (BW denominator, T-channel invariant ordering,
threshold kinematics `s → (m₁+m₂)²`, overlapping resonances) gets a test that
would fire if the map were wrong — a passing σ is never accepted as confirmation
of a convention (AGENTS.md "convention claims are hypotheses").

### Addendum — non-prefix s-channel recovery (momentum-routing convention)

While wiring the per-diagram channels (`DiagramChannel::from_diagram`), the
subsystem classifier that reads each internal line's stored `Prop.momentum`
(`Vec<i8>`, a signed external-momentum combination) was found to miss a class of
genuine final-state s-channel poles. Root cause is a **feyngraph momentum-routing
convention**:

- feyngraph assigns each external the unit momentum indicator, then **eliminates
  the highest-indexed external** via global conservation (`assign_momenta`
  last-external elimination). The stored vector for an internal line is therefore
  the signed combination for the cut side **away from** that highest external —
  the raw beam coefficients are **gauge-dependent** and cannot be read as "is this
  the beam side".
- The convention-robust classifier is the **beam content of the cut**: a genuine
  final-state s-channel subsystem is the side carrying **no beam**. That zero-beam
  side is the stored side when the stored coefficients touch no beam (`beams == 0`,
  the "prefix" case that already worked), and the **complementary** final-state set
  when they touch every beam (`beams == n_in`, the case that was missed). A cut
  whose two sides each carry a beam (`beams == 1` for a 2→n process) is a spacelike
  transfer and bounds no subsystem — that is genuine t-channel, and it is **left
  untouched here** (its importance map is a separate, still-deferred concern).

Concrete instance, `e+ e- > mu+ mu- ta+ ta-` (externals `0=e+,1=e-,2=mu-,3=mu+,
4=ta-,5=ta+`; feyngraph eliminates `5=τ⁺`): the τ⁺τ⁻ Z line is genuinely timelike
(a real s-channel pole on the `{ta⁻,ta⁺}` pair) but is stored as
`[1,1,-1,-1,0,0]` — both beams present, τ slots zero. The previous "any beam
coefficient nonzero ⇒ not a subsystem" test dropped it, so `from_diagram`
resonated only on the µ⁺µ⁻ pair (stored `[0,0,1,1,0,0]`, zero beams) and never on
τ⁺τ⁻. The µµ line is stored as a zero-beam indicator only because µ⁻,µ⁺ are not
the eliminated external; the τ line's both-beam form is a pure artifact of the
elimination, not a physical difference between the two pairs.

Note the empirical stored vector is `[1,1,-1,-1,0,0]`, **not** a bare indicator
`[1,1,1,1,0,0]`: feyngraph stores the *signed* momentum-flow combination, so the
individual coefficients carry flow signs. Only the **nonzero pattern** is
load-bearing for classification (the beam count and the outgoing-slot set), and
that pattern confirms the model exactly — both beams nonzero, τ slots zero.

Fix (relabel only): when the stored side carries every beam, return the
**complement** of its outgoing-slot set as the subsystem, under the same
`2 ≤ count < n_out` guard (which still excludes the s-channel core, whose
zero-beam complement is the whole final state). No new node type, no Jacobian,
kinematics, or sampler change — the recovered poles flow through the existing
L3 Breit-Wigner `draw_invariant`/`invariant_measure` machinery unchanged. The
classification is cross-checked against an **independent graph-cut** derivation
of the same partition (connected components after removing the line), so a future
feyngraph routing-convention change trips a test in either derivation.

### Addendum — t-channel spine (single spacelike line, `2 → 2`)

Genuine spacelike lines are no longer metadata-only for the simplest case. A
diagram with **exactly one spacelike line building a `2 → 2` final state** is
decomposed as a peripheral **spine** rather than an all-timelike tree.

- **New peripheral node type.** `DiagramChannel` now holds a
  `ChannelTopology` — `Timelike(Branch)` (the existing decay tree, unchanged) or
  `Spine(Spine)`. A `Spine` carries an `emitted` and a `recoil` `Node` plus the
  spacelike propagator's `t_mass2` (width forced to zero — note-07 2.8.0/2.9.3:
  a spacelike line has no Breit-Wigner). The emitted/recoil subsystems recurse
  into the **existing** `sample_branch`/`branch_jacobian` machinery unchanged, so
  timelike subtrees hang off the spine with no duplication.
- **Beam-frame state.** `DiagramChannel` now stores `beams: [LorentzVector; 2]`
  (beam 0 along `+z` in the CM), computed from `√ŝ` and the incoming masses. This
  is the reference for the transfer `t`; the timelike tree never needed it. It is
  the *only* new channel state, and `channel.rs` is untouched — the spine still
  satisfies `Channel::density`.
- **The `t` map (`draw_t`/`t_measure`).** Importance-samples the propagator
  `1/(t − m²)` with density `∝ 1/(m² − t)` via the logarithmic substitution
  `t = m² − (m²−t_min)·exp(−x·N)`, `N = ln[(m²−t_min)/(m²−t_max)]`, exact Jacobian
  `dt/dx = N·(m² − t)`. Both endpoints are `≤ 0`; a massless beam pins `t_max = 0`
  (collinear edge) and a massive initial state pushes `t_max < 0` (2.9.3). At the
  collinear edge or a threshold-degenerate window the pole cannot shape the draw,
  so it **falls back to flat in `t`** (the spine then reduces to the isotropic
  2-body split) — the exact analogue of the BW map's zero-width flat fallback.
- **Peripheral kinematics.** The emitted subsystem's polar angle is fixed by `t`
  (`t = m_a² + s₁ − 2E_aE₁ + 2k·p*·cosθ`), only `φ` free. The 2-body LIPS `R₂` is
  reparametrised from `(cosθ, φ)` to `(t, φ)` via `dcosθ = dt/(2k·p*)`, giving the
  rung factor `π·(dt/dx)/(4√ŝ·k)` (the `p*` cancels) — a different Jacobian from
  the timelike `r2_factor = π|p*|/√ŝ`.
- **`density` off the channel's own points.** Each rung's `t` is recomputed as the
  frame-independent invariant `(beams[0] − p_emitted)²` from the final momenta plus
  the stored beam, and `s₁,s₂` from the subsystem masses — so `Channel::density`
  stays well-defined on foreign configs (the L4 contract).
- **Spine ordering strategy (and why).** The emitted subsystem is anchored to
  **beam 0** and is the final-state legs on beam 0's side of the spacelike cut,
  read from the stored `Prop.momentum` nonzero pattern (the same convention-robust
  reading as `subsystem_mask`, cross-checked against the independent graph cut).
  Pairing the emitted blob with the wrong beam would read the crossed `u`-channel
  invariant; this is pinned by a firing test (emitted/recoil transfer consistency
  by momentum conservation, and a forward-bias test that a silent emitted/recoil
  swap flips). For the single-rung case the spine's `t_mass2` **supersedes** the
  old `t_channels` mass/width metadata as the kinematic driver; the `t_channels`
  accessor is retained only for higher-multiplicity/ladder diagrams that still
  fall back to the all-timelike tree.

**Deferred — multi-rung spine (Part 2).** A genuine multi-spacelike-line ladder
(VBF/DIS, `≥ 2` t-channel lines) needs an **explicit ordered chain of rungs** —
which final-state blobs attach to which rung, in what order along
`q_i = p_a − (p₁+…+p_i)` — derived from the `Prop` chain, superseding the
unordered `t_channels` metadata. This is note-07 2.9.0 ("four ordering strategies;
wrong default for many processes"), the session's stated bug magnet. It was
**deferred rather than committed** because its ordering Jacobian cannot be pinned
against an analytic/independent oracle in-session (volume `Vₙ` and a passing σ are
both blind to a wrong-but-valid ordering — AGENTS.md "a passing gate that cannot
see the convention is not confirmation"), and a single spacelike line inside a
`2 → n>2` final state is folded into the same deferral. Hand-off: extend `Spine`
to `rungs: Vec<SpineRung>` + a terminal `recoil`, each rung emitting one blob with
its own `t` against the running `q_i`; the load-bearing new oracle is an ordering
firing test (1-D `t_i` projections smooth and covering the full range; swapping
rung order changes the result as the physics dictates).

### Sprint A close-out

Sprint A (`resonance-sampling`, sessions L1 → L2 → L3 → L4 → L5, plus the R
non-prefix-recovery and T t-channel-spine addenda) is **complete on branch
`resonance-sampling`**. What it delivered:

- **The phase-space abstraction seam** (L1): `PhaseSpaceMap`/`Channel`/`Combiner`
  with flat RAMBO and the 2-body LIPS map behind it, no numeric change (bit-for-bit
  L1 gate).
- **Per-diagram channels** (L2, R): a `Diagram`'s `Prop` chain becomes a recursive
  2-body-decomposition tree; the flat channel Jacobian reproduces `Vₙ` for 2→2…2→6.
  R fixed the non-prefix s-channel recovery (feyngraph highest-external elimination).
- **Resonance maps** (L3, T): the Breit–Wigner tan-substitution for timelike
  invariants and the logarithmic t-map for a single-rung spacelike spine, each with
  an exact Jacobian pinned by a zero-variance-on-the-pole test.
- **The multichannel combiner** (L4): `MultiChannel` with the variance-minimising
  weight `1/Σⱼαⱼgⱼ`, wired as VEGAS's integrand map.
- **α-adaptation + the distribution-level validation regime** (L5): the
  Kleiss–Pittau survey→refine loop `αⱼ ← αⱼ√Wⱼ` (`MultiChannel::adapt_alphas`),
  composed with VEGAS as an outer survey (fix the mixture) → inner grid (refine the
  per-channel hypercube) with α frozen.

**Cumulative variance wins** (all against flat RAMBO at fixed N, each with its own
firing test so the win is never mistaken for convention confirmation): the BW map
alone (L3, Z pole), the multichannel combiner strictly below every single channel
and below flat on a multi-peak integrand (L4), the t-channel spine below flat on a
forward-peaked integrand (T), and α-adaptation below fixed-uniform α (L5). On the
L5 asymmetric multi-peak (a 4:1 amplitude ratio across two channels) α converged
[0.5,0.5] → [0.80,0.20] in ~2 iterations, tracking the amplitude ratio; the
per-channel variance shares `Wⱼ` equalised (2.66e-2 vs 2.67e-2); and per-point
variance fell ~1900× (uniform 9.4e-3 → adapted 5.3e-6) at essentially equal
per-point cost (248 → 242 ns/pt). That magnitude is the **best case** — the
synthetic integrand is exactly a linear combination of the two channels' BW shapes,
so variance-matched α approaches the zero-variance importance-sampling optimum; on a
real `|M|²` with continuum and interference the practical win is smaller, and the
test pins only the direction and strict inequality, not the number. Distribution
gates: the resonant BW line shape (χ²/dof ≈ 0.6) and the overlapping double-peak
line shape (χ²/dof ≈ 0.6) both match the analytic oracle.

**note-07 sampler-bug hazard firing-test inventory** (each fires if the map were
wrong; a passing σ is never accepted as confirmation):

| Hazard | Firing test |
|---|---|
| BW denominator / `ds/dθ` | `bw_map_is_measure_preserving`, `bw_map_zero_variance_on_bw_integrand` |
| T-channel invariant ordering (2.9.0) | `spine_transfer_pairs_emitted_with_beam0`, `spine_emitted_is_forward_biased` (silent swap flips the bias), `spine_built_for_real_t_channel_process` |
| Threshold kinematics `s→(m₁+m₂)²` (2.9.3) | `t_channel_threshold_window_collapses`, `t_bounds_include_initial_state_mass` |
| Overlapping / conflicting resonances | `overlapping_resonances_double_peak_resolved` (**added in L5**: two nearby timelike poles on the same invariant; the combiner resolves both, dropping the second channel collapses its coverage ~1000×) |

**Deferred** (carried forward, not regressions):

- **Multi-rung spine (Part 2)** — ladder topologies (VBF/DIS, ≥2 spacelike lines);
  the ordering Jacobian needs an in-session firing oracle before it can land (see
  the preceding deferral note).
- **MG-plot distribution comparison** — L5 validated sampled histograms against the
  *analytic* BW and t-channel oracles (exact) and used MG **σ** as the coarse
  backstop; comparing the sampled invariant-mass/angular histograms against MG's own
  `.lhe`/plots needs the MG toolchain and is a follow-up.
- **Massless-t-channel fiducial-cut question** — a massless beam pins `t_max = 0`
  (collinear edge), where the t-map falls back to flat; whether a fiducial cut is
  wanted there (rather than the flat fallback) for a physical massless-initial-state
  t-channel is unresolved.
- **Wiring the multichannel + α sampler into the CLI `integrate` path** — the
  combiner is validated as a unit but flat RAMBO still drives `integrate`; promoting
  it to the production integrand is what would let the resonant `validate_sigma`
  SKIP rows (`ee_to_mumu_tata_qcd0`, `ee_to_tatah`, `ee_to_mumua`) flip to GATE.

**Next: Sprint B — `event-output-lhef`** (E1 → E4, `mg-single-helicity-bench`
folded into E2). Unweighted event output via accept/reject over the frozen VEGAS
grid + this sprint's peak-resolving sampler, serialised to LHEF; expanded into its
own note when it opens.

---

## Helicity & color handling (both sprints)

The multichannel structure of Sprint A is over **momentum configurations only** —
one channel per diagram, parametrised by its propagator poles (L2). **Helicity
and color are not sampling channels.** Spelled out because the phrase "channel"
otherwise invites building per-helicity channels, which this plan does not do:

- **During integration (Sprint A): helicities are summed, colors are contracted**
  — exactly as on `main` today. `|M|²(p) = Σ_hel |M_hel(p)|²` via the shipped
  helicity-expanded arena + `prune_zero_helicities` (MG's `GOODHEL` filter, note
  15 §2.3), with the CF color contraction inside. The momentum channels are
  driven by the *full* helicity-summed `|M|²`; no channel is weighted by any
  single helicity's contribution. There is **no separate channel per helicity**.
- **At event-writing (Sprint B, E2): helicity and color are *selected* per
  accepted event, not sampled into the integral.** Once a point `p` is accepted,
  draw one helicity combination with probability `|M_hel(p)|² / Σ_hel |M_hel(p)|²`
  (MG's `SELECT_HEL`) and one color flow `∝ JAMP2(i)` (MG's `SELECT_COLOR`, E1).
  Both are cheap per-event categorical draws off **diagonal accumulators on the
  existing `eval_m2` loop** (the JAMP2 diagonal, note 15 §2.2; a parallel
  per-helicity `|M|²` diagonal), with **zero effect on σ or the integrand** —
  they only fill in the LHE record's helicity and `(color, anticolor)` tags.
- **Out of scope — Sherpa-style MC *sampling* of helicity/color** (its
  `Helicity_Integrator`/`Color_Integrator`, note 03 §1.5): treating helicity
  and/or color as extra sampled dimensions with their own adaptive weights
  *instead of* summing/contracting. That is a high-multiplicity optimisation
  (smaller per-point cost, extra variance) and a different sampler design; it is
  the TODO `lips-nbody` "possibly Sherpa-style sampling over color/helicity"
  future direction, **not** this program. For 2→2…2→6 with `GOODHEL` already
  pruning dead combinations, MG-style summation is the correct default and reuses
  the on-`main` machinery unchanged.

---

## Sprint B — `event-output-lhef` (outline; own note at open)

Unweighted events via accept/reject `w(p) = |M(p)|²/w_max`, serialised to Les
Houches Event File format. Depends on Sprint A (n-body + peak-resolving sampler,
so unweighting efficiency is usable). Handoff format is A-sprint's
`IntegrateArtifact`; the `generate` phase deserialises it and refuses a mismatched
run rather than re-taking raw CLI flags.

| Session | Scope |
|---|---|
| **E1 `jamp2-flow-select`** | Diagonal `JAMP2(i) = Σ_hel |JAMPᵢ|²` accumulator on the existing `eval_m2` combination loop (cheap, note 15 §2.2), then the **flow → `(color, anticolor)` LHEF tag dictionary** per external leg, sampled ∝ JAMP2. **Pin the dictionary against MG's `SELECT_COLOR` / `color_flow_decomposition` / `get_color_flow_string`** — a transposed dictionary is invisible to any |M|²-level gate (validation backlog; gg_to_gg NCOLOR=6 flow-basis ordering caveat applies). |
| **E2 `accept-reject` + `mg-single-helicity-bench`** | Unweighting over the frozen VEGAS grid + Sprint-A sampler: `w_max` estimation, overweight bookkeeping, unweighting efficiency, and **per-event helicity + color-flow *selection*** (not sampling — see §"Helicity & color handling"): draw helicity `∝ |M_hel(p)|²` (`SELECT_HEL`) and flow `∝ JAMP2(i)` (`SELECT_COLOR`, E1), both off diagonal `eval_m2` accumulators, with zero effect on σ. This makes **single-helicity evaluation through the unexpanded program** the hot path, so land `mg-single-helicity-bench` here (vibegraph `eval_amplitude` at one fixed helicity + the MG single-config Fortran-harness timing — half an oracle until now). **Gate:** unweighted sample reproduces σ and the L5 distributions within MC error. |
| **E3 `lhef-writer`** | LHE serialiser: `<init>` block (beams, PDF, process ids, xsec/xerr/xmax) + `<event>` blocks (NUP, PDG, status, mother indices, **color tags from E1**, momenta, mass, helicity, weight). **Pin the byte-level format against an MG-generated `.lhe`.** |
| **E4 `generate-cli`** | `vibegraph generate <artifact> [--nevents …]`: deserialise `IntegrateArtifact`, refuse a run whose proc/run card mismatches, drive E2 accept/reject → E3 `.lhe`. **Gate:** end-to-end `.lhe` parses in a downstream tool; σ from event weights matches the `integrate` σ. |

Order: **E1 → E2 → E3 → E4** (E1 unblocks color tags E3 needs; E2 the events;
E3 the format; E4 the CLI). `mg-single-helicity-bench` folds into E2.

---

## Execution notes (agent dispatch)

- Use the **`feature-dev`** agent (Opus; never general-purpose — that ignores
  model overrides and always runs Fable). Sonnet override only for a genuinely
  light session (none of L1–L5 obviously qualifies; L1 is a careful refactor).
- **Pre-create worktrees off `main` manually and COW-clone the validation data
  dirs** — worktree isolation has leaked into the shared checkout twice
  (`eval-perf-2`), especially on resume. Hard `cd`-verify before each agent acts.
- One session per agent; measure vs the stated baseline; run the session's gate;
  commit on the sprint branch. ff-merge to `main` at sprint close (user decides
  the merge).
