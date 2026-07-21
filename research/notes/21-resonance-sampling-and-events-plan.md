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
