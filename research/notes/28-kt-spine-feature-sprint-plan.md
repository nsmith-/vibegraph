# 28 — `kt-spine` feature sprint plan (scales + phase-space, two tracks)

**Status: DRAFT — awaiting user review; decisions D1–D3 open (§6).**

The feature sprint that unblocks the validation cells `v3-backlog` could not
reach and freezes the channel/map structure ahead of the integration-focused
performance sprint (VEGAS first-iteration bias + `w_max` scan decoupling +
stratified-parallel axes — all measurements *against a channel structure*, so
that structure must stop moving first). Two nearly disjoint tracks:

- **Track K (scales)** — general kT clustering for `dynamical_scale_choice = -1`
  (`coupling/scales.rs` + a new clustering engine). Unlocks: the 6
  asserted-refused per-event scale replays, the 4 blocked llj partonic σ cells
  (already banked, integrable in seconds), and the dynamical-scale re-gate of
  σ(pp→ℓ⁺ℓ⁻j). Hard prerequisite for gating any QCD process at MadGraph's
  default scale choice.
- **Track S (phase-space)** — identical-particle symmetry factor into the
  phase-space map, then the multi-rung t-channel spine deferred by note 21,
  with the massless-t-channel-cut question folded into the spine design.
  Addresses the `uux_to_uux` −0.30% residual bias and the degenerate-map
  finding (note 27 §B3.2), and removes `ProtonIntegrand`'s refusal of any
  flavour group whose symmetry factor ≠ 1.

The tracks converge on one capstone: **`p p > j j` gated at MadGraph's default
dynamical scale** — the canonical QCD process, needing K's scales and S1's
per-subprocess factors at once (the spine improves its variance but is not a
correctness prerequisite, so the capstone is not hostage to the sprint's
hardest item).

The standing rules are unchanged: never a loosened tolerance; new physics lands
informational and is enforced only when agreement is demonstrated; a known-wrong
informational comparison runs while a feature is under construction; every
convention claim is pinned by a test that would fail if it were false.

## 1. Inputs

- Note 22 §1.3 — what `-1` collapses to on the banked runs (the degenerate
  closed forms now in production, which the general path must reproduce
  exactly), the `djb` measure, the `1 + 1e-6` beam-crossing tie-break pinned by
  `uux_to_uux`'s 10/10000 crossed events, `scalefact` placement quirks.
- Note 21 "Deferred — multi-rung spine (Part 2)" — the hand-off design
  (`Spine` → `rungs: Vec<SpineRung>` + terminal `recoil`, running
  `q_i = p_a − (p₁+…+p_i)`), and the deferral reason: `Vₙ` and σ are both
  blind to a wrong-but-valid rung ordering, so the ordering firing test is the
  load-bearing oracle (note 07 §2.9.0).
- `TODO.md` §Feature backlog — the four items with their evidence, and the
  `kt-clustering` session sketch this plan supersedes.
- Banked data already in `refdata-3`: the 6 no-closed-form runs
  (`pp_to_llj{,_qcd2_qed2}`, `ee_to_mumua`, `ee_to_mumu_tata_qcd0`,
  `bbx_to_ccx_emmm_qcd0`, `uux_to_ccx_emmm_qcd0`) carry per-event `SCALUP` /
  `<rscale>` / `<pdfrwt>`, and the 4 llj partonic runs carry cross sections —
  **Track K's gates through K5's flip need no new MadGraph runs.**
- Branch base: `main` (`v3-backlog` merged; `refdata-3` published and pinned).

## 2. Session-scoping ground rules

Sized against what has bitten previous sprints: B5 died of context bloat by
combining a 28-run re-bank with hygiene and close-out; several sessions grew
past their design when an engine and its oracle were built in the same breath.
Rules for this sprint:

- **One deliverable per session**; a session that builds an engine does not
  also build the oracle it is judged by, and no session both banks references
  and does anything else.
- **Oracle before engine** in both tracks: the session that produces the
  reference dump/spec lands and is reviewed before the session that must match
  it starts.
- **Sonnet sub-agent relief valves are named per session below.** The standing
  rule (`.claude/agents/*.md`): deterministic bulk only — run sweeps, fixture
  regeneration, mechanical extraction — script-first, one nesting level,
  `model: "sonnet"`, never judgment, never a gate's meaning; every brief
  carries the worktree-isolation and background-long-command rules verbatim,
  and sub-agent reports are spot-checked claims, not results.
- MG runs and full `pixi run validate` are backgrounded with log files, always.
- Worktrees pre-created off `main` by the sprint manager, data COW-cloned.

## 3. Track K — general kT clustering (`dynamical_scale_choice = -1`)

### K1 — design note (reading session, no code)

Read MG's `cluster.f` / `setscales.f` (`setclscales`) / `reweight.f` path end
to end and write the binding spec into this note: the `djb`/kT measure and its
PDF/no-PDF switch (note 22 §1.3 has the measured collapse table as the
known-good anchor); which merges are admissible — **graph-guided, only vertices
the process's diagrams contain**; the tie-break order including the `1 + 1e-6`
beam-crossing inflation; how the winning cluster sequence maps to μR (the
geometric-mean prescription, whose *form* note 22 §1.3 shows is unpinned by any
banked run — name the Fortran lines that pin it) and to per-beam μF; where
`scalefact` lands in each branch (note 22 §1.1's correction table). Deliverable
also includes the K2 dump format: which per-event quantities an instrumented
run must record for the finest-linear-level comparison (merge sequence with
participating legs and vertex, each merge's `djb`, the final 2→2 core, μR, per-beam
μF). Falsifiers listed per claim. **Small session, pure judgment — no
delegation.**

### K2 — the clustering oracle (instrumented MG dump)

Build the oracle *before* the engine, in the style of the amplitude probes
(`validation/madgraph/wrappers/`): instrument the pinned 3.7.1's clustering to
dump, per banked event, the record K1 specified — for the 6 no-closed-form
runs and (as known-good controls) 2–3 degenerate rows from note 22 §1.3's
table. Committed artifact: per-run dump files in the work area + the extraction
driver, sha-pinned like the other oracles. Gate for the session: the dump's
final μR/μF reproduce the banked `SCALUP`/`<rscale>`/`<pdfrwt>` for every event
— proving the instrumentation reads the code path that actually produced the
bank, not a lookalike. **Sonnet relief: the per-run sweep (driver script
running 8–9 processes × 10k events, background + logs) and the format
normalisation; the instrumentation points and dump semantics are Opus.**

### K3 — clustering engine, informational

Diagram-guided clustering of an event's external momenta down to a 2→2 core,
building on the `ClusterTopology` derivation from the `dynamical-scales`
sprint. Judged only against K2's dumps, as an *informational* per-event
comparison first: merge-sequence identity, then scale identity. A mismatch
class is diagnosed, not tolerated — the K2 dump makes "which merge diverged
first" a direct read. No production wiring in this session; `ScaleChoice`'s
`-1` still takes the closed-form-or-refuse path. Gate: engine reproduces K2's
merge sequences and scales on all dumped runs (target: event-exact sequence
identity; any principled exception is written into this note with its
justification). **Sonnet relief: none in the engine; optionally a sweep script
re-running the comparison across all dumps.**

### K4 — scale synthesis + production wiring

Replace the closed-form-only `-1` branches in `ScaleChoice` with the general
path. The note 22 §1.3 degenerate cases become consistency checks: the general
code must reproduce, event for event, the already-enforced replays on
`gg_to_gg`/`gg_to_ttx`/`uux_to_uux`/`pp_to_bb*`/`pp_to_ll*` (including the
`250.0001` tie-break events) — a regression net that exists before the first
line of wiring. Then flip the 6 asserted-refused rows in `validate_scales` to
enforced per-event replays (`SCALUP`/`<rscale>`/`<pdfrwt>`). Gate:
`pixi run --skip-deps validate` green with the 6 rows enforced and every
previously-enforced row unmoved.

### K5 — the σ flips

Flip the four llj partonic σ rows (`uux_to_epemg`, `ddx_to_epemg`,
`gu_to_epemu`, `gux_to_epemux`) from `blocked` to GATE — banked, seconds to
integrate, waiting on nothing but K4 — with seed sweeps per the standing rule
(a sweep is a floor, budget convergence is the second axis). Their `samples`
cells flip from blocked with them (generation at the now-supported scale).
Then re-gate σ(pp→ℓ⁺ℓ⁻j) against a *dynamical*-scale MG run (banked by Sb):
the fixed-scale row stays enforced, so only the scale moves in that chain.
**Sonnet relief: the integration/seed-sweep runs are script-driven bulk;
tolerance decisions are Opus.**

## 4. Track S — phase-space (permutation factor, then the spine)

### S1 — identical-particle symmetry factor into the map

Move the `Π_s n_s!` factor from per-integrand scalars into the phase-space map,
which knows its own outgoing multiset — every consumer right by construction.
Kill both latent shapes: `FixedBeamIntegrand::new` deriving from `amps[0]` and
applying it to every subprocess, and `ProtonIntegrand`'s assert-factor-is-1
refusal. Also *decide and record here*: whether multichannel treats
permutations of identical particles as distinct channels or one channel with
the factor folded in (the channel-enumeration decision the performance sprint
needs frozen). Gate: `gg_to_gg`'s 1/2 reproduced with the already-enforced σ
row unmoved; unit tests over mixed multisets (`[g g]` vs `[q q̄]` with equal
mass lists); `ProtonIntegrand` accepts a mixed-factor group and applies each
subprocess's own factor. Independent of everything — can run first and in
parallel with K1.

### S2 — spine design + ordering oracle spec (reading/design session, no engine)

The deferral reason *is* the session: note 21 deferred the multi-rung spine
because `Vₙ` and σ are blind to a wrong-but-valid rung ordering. Deliverables,
written into this note: (a) the rung-chain derivation from the `Prop` chain —
which final-state blobs attach to which rung, in what order along the running
`q_i = p_a − (p₁+…+p_i)` — superseding the unordered `t_channels` metadata;
(b) the **ordering firing test** made concrete: 1-D `t_i` projections smooth
and covering their ranges, plus the negative control — a deliberately swapped
rung order must make the test fire (a firing test that cannot fire is the
vacuous-check failure); (c) the foreign-config density contract (L4) extended
to rung chains; (d) **the massless-t-channel cut decision (D3)**: at the
collinear edge a massless beam pins `t_max = 0` and the map falls back flat —
decide with a measurement whether the fiducial cuts should bound `t_max`
instead, comparing variance with/without on a cut-regulated process; (e) the
**spine reference process** (D2): a genuinely two-rung banked reference at a
*fixed* scale (so Track S never waits on Track K) — candidate
`p p > e+ e- j j` (VBF-ish, QCD=0 to keep the diagram set small), card
proposed here for the user to confirm before Sb banks it.

### S3 — spine engine, opt-in + informational

`Spine` → `rungs: Vec<SpineRung>` + terminal `recoil`, each rung emitting one
blob with its own `t` against the running `q_i`; single-rung spines and the
all-timelike fallback untouched. Landed opt-in: ladder diagrams keep falling
back to the timelike tree in production while the spine runs informational
beside it. Gate: the S2 firing test green *and its negative control firing*;
`Vₙ` volume checks; foreign-config density contract; all enforced rows
unmoved (the spine is not yet in production).

### S4 — spine to production + the measurements it owns

Switch ladder diagrams to the spine and measure what the backlog says it
should move: the `uux_to_uux` five-seed mean (currently −0.30%, does it
shrink?), the degenerate-map finding (do the per-diagram densities
differentiate, does α move off uniform?), the spine reference process σ
against Sb's banked run. Every enforced σ row re-measured. An honest outcome
where a number does *not* move is recorded as a finding, not massaged.
**Sonnet relief: the seed-sweep/budget-ladder runs; the verdicts are Opus.**

## 5. Shared sessions

### Sb — banking (MG wall time, code-independent)

One session, nothing else: card + run through `mg5_pinned.sh` (3.7.1) into the
work area — the dynamical-scale llj run (K5), the spine reference process
(after S2/D2 fix its card), and the capstone `p p > j j` run (after D1 fixes
its card). Rows enter the manifest `bundled = false` (the B4 precedent: red on
fetching checkouts until the re-cut). Runs stay local; **the bundle is not
recut here**. Can start as soon as the first card is confirmed; the llj and jj
cards depend on no session. **This is the sprint's most Sonnet-friendly
session: card-templating and run-driving are deterministic bulk under an Opus
review of the cards themselves.**

### C — capstone: `p p > j j` at MadGraph's default dynamical scale

Needs K4 (general scales), S1 (per-subprocess symmetry factors), Sb (the
banked run). The full chain on the canonical QCD process: flavour groups with
nontrivial and *unequal* symmetry factors, general clustering per event,
multichannel over the mixed subprocess set, σ + samples cells measured (gate
or curated ⚠️ with the reason, per the standing rule). The spine, if S4 has
landed, rides along; if not, C runs on the existing maps — degraded variance,
same correctness — and says so.

### Z — close-out

The one-shot **`refdata-4` re-cut** (Sb's runs join the bundle; double
assembly byte-identical, event-text sha-stable, clean `git archive` export
green from the bundle alone; publish + flip the pin — user step, same protocol
as refdata-3), manifest `bundled = false` rows flipped, report re-rendered and
asserted with the new cells, TODO.md and this note's close-out written.
Nothing else rides on Z — that is the B5 lesson. **Check `pixi.lock` against
`pixi.toml` with a locked install if any environment changed (the refdata-3
close-out tripped CI on exactly this).**

## 6. Decisions (user)

1. **D1 — capstone card**: `p p > j j` with MadGraph's default run card
   (dynamical scale, default `ptj`) — recommended. Alternative: also a
   fixed-scale jj row first, giving a scale-independent stepping stone at the
   cost of one more banked run.
2. **D2 — spine reference process**: S2 proposes the card (candidate
   `p p > e+ e- j j` QCD=0, fixed scale); user confirms before Sb banks it.
3. **D3 — massless-t-channel cut**: resolved inside S2 by measurement
   (flat-fallback vs fiducially-bounded `t_max`); the decision and its numbers
   land in this note.

## 7. Sequencing

```
Track K:  K1 ──→ K2 ──→ K3 ──→ K4 ──→ K5 ─┐
Track S:  S1 (independent, start first) ───┼──→ C (capstone) ──→ Z (close-out)
          S2 ──→ S3 ──→ S4 ────────────────┘        ↑
Sb (banking): llj/jj cards any time; spine card after S2 ──┘
```

- S1 ∥ K1 open the sprint (both small); K2 ∥ S2 follow.
- C needs K4 + S1 + Sb's jj run; S4/K5 are not on C's critical path.
- Z needs everything and does nothing but the re-cut and bookkeeping.
- Nine sessions plus banking and close-out — the largest sprint yet, which is
  why every session above is single-deliverable and four of them are
  reading/design or bulk-driving sessions rather than engine work.

## 8. Census impact (what flips where)

- `validate_scales`: 6 asserted-refused rows → enforced per-event replays (K4).
- `integrals`: 4 llj partonic cells `blocked` → GATE (K5); σ(pp→ℓ⁺ℓ⁻j)
  re-gated at dynamical scale (K5); spine reference row new (S4); `pp_to_jj`
  row new (C).
- `samples`: the 4 llj cells unblock (K5); `pp_to_jj` (C).
- Standing-discrepancy register: `uux_to_uux` bias and degenerate-maps entries
  re-measured with verdicts (S4).
- New manifest rows (spine ref, pp_to_jj) enter `bundled = false` until Z.

## 9. Non-goals

- MLM matching (kt-clustering is its prerequisite, not its delivery).
- The integration-focused performance sprint's items: VEGAS first-iteration
  bias, `w_max` scan decoupling, stratified-parallel axes, per-flow α Stage-1
  (which becomes meaningful only after S4 differentiates the maps).
- V7 per-flavour union, `ee_to_mumua` windowed comparison, Pythia blind spots
  (validation backlog, later pass).
- Re-baselining the per-event scale hot-path cost (~100 ns) — measured *after*
  K4 lands, in the performance sprint, not chased here.

## 10. Agents

`feature-dev` (Opus) for K1–K5, S1–S4, C; Sb may run as `feature-dev` with the
Sonnet override (bulk under confirmed cards); Z is Opus — the re-cut is the
step where a cheap mistake costs a published pin. Sessions get the
worktree-fragility and background-long-command rules verbatim, and the
oracle-before-engine ordering as a hard constraint (K3 does not start before
K2 is merged; S3 not before S2 is reviewed).
