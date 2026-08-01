# 28 — `kt-spine` feature sprint plan (scales + phase-space, two tracks)

**Status: APPROVED 2026-08-01 — §6 recommended decisions adopted (D1–D3 resolved); sprint launched.**

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

## 6. Decisions (user — resolved 2026-08-01, recommended options adopted)

1. **D1 — capstone card (decided)**: `p p > j j` with MadGraph's default run
   card (dynamical scale, default `ptj`). No fixed-scale stepping-stone row.
2. **D2 — spine reference process (decided)**: the candidate
   `p p > e+ e- j j` (QCD=0, fixed scale) is approved as the reference
   process. S2 still writes the concrete card into this note; Sb banks it
   after the sprint manager's card review — no further user round-trip unless
   S2 finds a reason to deviate from the approved candidate.
3. **D3 — massless-t-channel cut (decided)**: resolved inside S2 by
   measurement (flat-fallback vs fiducially-bounded `t_max`); the decision and
   its numbers land in this note.

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

---

## K1 — binding spec: MadGraph kT clustering for `dynamical_scale_choice = -1`

Read off the **pinned** checkout, `research/refs/mg5amcnlo` at tag `v3.7.1`
(`b7687064b9a013317ca164aa1395bc9c0e39ae1e`). Every path below is relative to
that submodule root. Line numbers are 3.7.1's, not note 22's — note 22 §1 was
written against the *packaged* 3.5.7 template, and §K1.9 below records the one
place where the two versions differ in behaviour, not just in numbering.

The whole of `-1` is: `set_ren_scale` and `set_fac_scale` deliberately return
zero, and `setclscales` then fills `scale` and `q2fact(1:2)` from a kT
clustering of the event down to a core. Nothing else in the LO template
computes the default scale.

Every claim carries a **falsifier**: an observation an instrumented run (K2)
could make that would prove the claim wrong. A claim whose falsifier cannot be
observed is flagged as such rather than asserted.

### K1.0 Entry points and the run-card constants the algorithm reads

| fact | file:line | falsifier |
|---|---|---|
| `-1` makes `set_ren_scale` return `rscale = 0` | `Template/LO/SubProcesses/setscales.f:46-49` | dump `scale` immediately after the `cuts.f:1235` call; a nonzero value falsifies |
| `-1` makes `set_fac_scale` return `q2factorization(1:2) = 0` for each beam not fixed | `setscales.f:133-137` | same, on `q2fact` after `cuts.f:1240` |
| the per-event driver is `set_ren_scale` → `set_fac_scale` → `setclscales`, in that order | `SubProcesses/reweight.f:1890-1913` (`update_scale_coupling_vec`) — the **only** caller of `setclscales` outside `rewgt`. `passcuts` runs the first two independently at `SubProcesses/cuts.f:1233-1240`. The generated driver reaches `_vec` from `madgraph/iolibs/template_files/super_auto_dsig_group_v4.inc:312` (grouped subprocesses) or, ungrouped, through the scalar wrapper `update_scale_coupling` (`auto_dsig_v4.inc:127` → `reweight.f:1850`, which is `_vec` with `VECSIZE_USED = 1`) | instrument all three; an event reaching `setclscales` with `scale > 0` falsifies |
| `setclscales` short-circuits when everything is already set | `reweight.f:643-654` — `ickkw≤0 .and. (xqcut≤0 .or. init_mode) .and. q2fact(1)>0 .and. q2fact(2)>0 .and. scale>0` | for `-1` with free scales all three are zero, so the branch must never be taken; dump a flag |
| the clustering map is built once, at first `passcuts` | `SubProcesses/cuts.f:209-210` → `SubProcesses/initcluster.f:48` (`filmap`) | count `filmap` calls; >1 per process directory falsifies |
| `initcluster` is skipped entirely when `ickkw≤0 .and. xqcut≤0 .and.` all three scales are fixed | `initcluster.f:28` | irrelevant for `-1` (scales are not fixed), but dump the guard |

Run-card constants the algorithm branches on, with their **defaults** from
`madgraph/various/banner.py` (all banked runs leave every one of these at its
default, `scalefact` included):

| symbol | default | banner.py | used at |
|---|---|---|---|
| `dynamical_scale_choice` | `-1` | `:4266` | `setscales.f:46,133` |
| `scalefact` | `1.0` | `:4283` | `setscales.f:93`, `reweight.f:1139,1140,1171,1188,1189,1192,1194,1200,1202` |
| `ickkw` | `0` | `:4284` | `reweight.f:643,1103,1114,1195`; `cluster.f:621,880` |
| `ktscheme` | `1` | `:4286` | `cluster.f:610,621,869,880` |
| `chcluster` | `False` | `:4288` | `cluster.f:468-470`; forced true by `reweight.f:664,1027` |
| `pdfwgt` | `True`, but forced `False` when `ickkw==0` (`Source/setrun.f:82`) | `:4289` | `reweight.f:1195` |
| `maxjetflavor` | `4` | `:4424` | `reweight.f:194` (`isjet`), `Source/kin_functions.f:289-294` |
| `xqcut` | `0.0` | `:4425` | `reweight.f:643,1066` |
| `bwcutoff` | `15.0` | `:4305` | `SubProcesses/myamp.f` on-shell test |
| **`d`** | **`1.0`** | `:4441` (hidden) | `common/to_dj/D`, `Source/kin_functions.f:246,297` |
| `use_syst` | `True` | `:4426` | `reweight.f:1274-1282`, `:1450-1459` |
| `clusinfo` | `True` | `:4291` | `unwgt.f:838` — but gated on `ickkw≠0`, so **no `<clustering>` tag is written in any banked run** |

`d` deserves its own line. It is a *hidden* run-card parameter that lands in
`common/to_dj/D` through `run_card.inc`; the LO `Source/setrun.f:21-22` only
*declares* it and never assigns it (the NLO template assigns `D=1d0` at
`Template/NLO/Source/setrun.f:102`; the LO one has no counterpart). So the value
is whatever `run_card.inc` writes. It appears squared in the denominator of the
hadronic final-state measure (`kin_functions.f:297`), so a run where it somehow
reached zero would make every non-resonant final-state merge `+Infinity` and
break the clustering outright.
**Falsifier: dump `D` from inside `dj` on the first call of each run. Anything
other than `1.0` invalidates every hadronic final-state measure below.** This is
the single cheapest high-value assertion in the K2 dump.

### K1.1 The `djb` / kT measure and its PDF/no-PDF switch

Four measures, all squared scales in GeV².

**`DJB(p)` — one leg against the beams.** `Source/kin_functions.f:401-431`.

```
lpp(1)==0 .and. lpp(2)==0 :  djb = max(p(0), 0)**2                  (:426)
otherwise                 :  djb = (p(0)-p(3))*(p(0)+p(3))          (:428)
```

The switch at `:423` is on **both beams being PDF-less**, not on beam energy and
not per-beam. With a PDF this is the transverse mass squared `m² + p_T²`; with
neither beam carrying one it is `E²`, and the routine's own commented-out
`'Error. No jet measure w.r.t. beam.'` at `:424` says why: there is no beam
direction worth measuring against. This is exactly `scales.rs`'s `djb`
(`vibegraph-lib/src/coupling/scales.rs:385-391`) and the switch it keys off
`hadronic_beams`.

*Falsifier:* an `lpp = 0` run whose per-event scale varies with the outgoing
`p_z` at fixed `E` would falsify the `E²` form; an `lpp ≠ 0` run whose scale is
insensitive to a leg's mass at fixed `p_T` would falsify the `(E−p_z)(E+p_z)`
form. Both are directly readable from the K2 dump's per-candidate table.

**`DJ(p1,p2)` — two final-state legs.** `kin_functions.f:230-311`. Three
branches:

```
lpp(1)==0 .and. lpp(2)==0 :  dj = 2*min(E1²,E2²)*(1-cosθ12)          (:271)   Durham
                             (guarded: zero 3-momentum prints a warning
                              and returns 0, :273-278)
otherwise, "massless-massive" case:
    p1(4)<1 and (p2(4)>=3 and maxjetflavor>4 or p2(4)>=1 and maxjetflavor>3)
                          :  dj = DJB(p1)*(1+1e-6)                    (:291)
    symmetric in 1<->2    :  dj = DJB(p2)*(1+1e-6)                    (:294)
otherwise                 :  dj = max(m1²,m2²)
                                  + min(pT1²,pT2²)*2*(cosh Δη - cos Δφ)/D²  (:296-297)
```

Note the shape of the last line: `cos Δφ` is written as
`(px1·px2 + py1·py2)/√(pT1²·pT2²)`, and the whole `2(cosh Δη − cos Δφ)` factor
is `ΔR²` to leading order. For a **2 → 2** the two legs are exactly back-to-back
in the transverse plane, so `cos Δφ = −1` and the factor is `2(cosh Δη + 1) ≥ 4`.
That is the general reason behind `scales.rs`'s hand-waved "`dj` carries a
`2(cosh Δη − cos Δφ)` factor that is at least 4" — it is now pinned to `:296-297`
and to the exact `Δφ = π` property of a 2 → 2, not to a plausibility argument.

`p1(4)` is **not** the energy: `pcl(0:4, ·)` carries the mass squared in slot 4
(`SubProcesses/cluster.inc`, `pcl(0:4,n_max_cl) ! 4 is mass**2`), filled for
external legs at `cluster.f:586` by `dot(p,p)`. `dot` itself
(`kin_functions.f:593-605`) clamps to exactly zero when `|dot| < 1e-6` **and**
`dot/(ΣpᵢqᵢEuclidean) < 1e-6`, which is what makes the `p1(4) < 1d0`
massless test at `:289` reliable rather than luck.

*Falsifier:* the massless–massive branch is unexercised at `maxjetflavor = 4`
unless one leg has `m² ≥ 1 GeV²` (so: `c`, `b`, `t`, `W`, `Z`, `h`, `τ`) and the
other is light. `bbx_to_ccx_emmm_qcd0` and `uux_to_ccx_emmm_qcd0` should hit it;
K2 must dump which of the three `dj` branches each candidate took, and an
instrumented run in which `:291`/`:294` never fires on those two runs falsifies
the reading.

**`PYDJ`** (`:313-342`) and **`PYJB`** (`:433-483`) are the `ktscheme == 2` /
`ickkw == 2` alternatives. `ktscheme` defaults to 1 and `ickkw` to 0, so
**neither is on any banked path**; the spec covers them only so K3 can assert
they are unreachable. *Falsifier:* dump `ktscheme` and `ickkw` per run.

**Beam–leg pairs do not use `dj` at all.** `cluster.f:626`:
`pt2ij(idij) = djb(pcl(0,idi))` where `idi` is the **final-state** line — the
beam enters only through the tie-break of §K1.3, not through the measure. The
companion `zclus` (`kin_functions.f:485-527`, stored in `zij`/`zcl`) is
MLM-Sudakov bookkeeping and never reaches a scale at `ickkw = 0`.

**Breit–Wigner override on final-state pairs.** `cluster.f:604-608`: if the
merged mask is tagged on-shell (`isbw`), the measure is
`SumDot(pcl_i, pcl_j, 1d0)` — the pair's **invariant mass squared** —
instead of `dj`. Tagging comes from `checkbw` (`cluster.f:386-434`), which walks
`iforest(1:2, i, this_config)` for `i = -1 … -(nexternal-3)`, calls
`cut_bw(p)` (`myamp.f:2`) to refresh `OnBW`, and sets `isbw(icl(i))` from it.
Two consequences worth naming:

- `checkbw` reads `this_config` and only `this_config` (`cluster.f:419-420`),
  so **the BW tagging is a property of the integration channel, not of the
  process.** A replay that does not know `iconfig` cannot in general reproduce
  `isbw`.
- 3.7.1 added the `call cut_bw(p)` at `cluster.f:423`; 3.5.7 relied on whatever
  `OnBW` a previous call had left. This is a behaviour change between the two
  versions, in the resonant direction.

*Falsifier:* dump `nbw`, `ibwlist`, and the `isbw` mask per event, plus
`iconfig`. A banked resonant run (`pp_to_ll`, `ee_to_zh`) in which `isbw` is
never set falsifies the reading of `checkbw`; a run in which the winning
final-state measure equals `dj` on an event where `isbw` is set falsifies
`:604-608`.

### K1.2 Which merges are admissible, and how MG derives the merge graph

The clustering is **graph-guided**: a pair may merge only if some surviving
diagram contains a propagator whose subtree is exactly that pair's set of
external legs. The derivation runs in five steps, three of them at code
generation time.

**(a) Per diagram, split into s- and t-channel vertex chains.**
`madgraph/core/helas_objects.py:1926-2010` (`get_s_and_t_channels`) walks a
diagram from the outermost final-state wavefunctions inward toward the
highest-numbered initial leg, emitting an s-channel `VertexList` and a
t-channel one. Each vertex's legs are `(daughter₁, daughter₂, …, mother)` with
the mother last, and the mother is renumbered to `min` of the daughters'
external numbers (`:1983`).

**(b) Drop every diagram whose vertices are not all of minimal arity.**
`madgraph/iolibs/export_v4.py:2180-2183` computes
`minvert = min over diagrams of (max legs per vertex)`, and `:2193-2197` skips
any config with a vertex of more legs than `minvert`
(`# Only 3-vertices allowed in configs.inc`). For any process that has at least
one all-cubic diagram, this **deletes the four-point contact diagrams from the
merge graph entirely** — `gg → gg` contributes three configs (s, t, u), not
four. *Falsifier:* dump `mapconfig(0)` per process directory; a `gg → gg`
subprocess reporting 4 falsifies it. This is also the cleanest cross-check
against our own diagram enumeration, which keeps the VVVV diagram.

**(c) Write the surviving configs as `iforest` / `sprop` / `tprid`.**
`export_v4.py:2249-2251` writes `iforest(1:2, mother, iconfig)` = the two
daughter leg numbers (positive = external, negative = internal);
`:2259-2261` writes `sprop(iproc, mother, iconfig)` = the s-channel propagator
PDG **per subprocess** of the group; `:2265-2267` writes
`tprid(mother, iconfig)` = `|pdg|` for a t-channel line (with `sprop = 0`), and
`:2262-2263` writes `tprid = 0` for an s-channel line. Masses and widths of the
propagators go to `props.inc` (`export_v4.py:2097`), read at `cluster.f:240`
and `myamp.f:80`. The QCD coupling order per config goes to `config_nqcd.inc`
(`export_v4.py:4457-4458`, writer at `:5264-5275`).

**(d) `filmap` turns the configs into a mask → graph-list table.**
`cluster.f:325-383`.

- `:359` loops configs `1 … mapconfig(0)`.
- `:360-365` **skips every config whose `nqcd` differs from `nqcd(this_config)`.**
  The admissible merge graph is therefore a function of the *integration
  channel's coupling order*, not of the process alone. For a process whose
  diagrams all carry the same QCD order this is a no-op; for a mixed QED/QCD
  process (`p p > e+ e- j` at `QCD<=2 QED<=2`, `ee_to_mumua`) it is not.
- `:367-375` seeds each external leg `j` with the one-bit mask `2^(j-1)`, and
  records `ipdgcl(mask, iconfig, iproc) = idup(j,1,iproc)` from `leshouche.inc`.
- `:379` calls `filgrp` repeatedly until it returns false.

**`filgrp`** (`cluster.f:191-322`) is the actual derivation. At each level it
scans pairs `(i,j)` of the current line list and looks for an internal index
`k ∈ [-nexternal+1, -1]` with
`{iforest(1,k,ignum), iforest(2,k,ignum)} == {ipids(i,2), ipids(j,2)}`
(`:254-258`). On a hit:

- `icmp(1) = combid(mask_i, mask_j)` where `combid(i,j) = i + j`
  (`cluster.f:151-166`, `:163`) — a plain sum, which is the bitwise OR because
  the two masks are disjoint;
- `icmp(2) = 2^nexternal − 1 − icmp(1)` (`:262`) — **the complement.** Both are
  registered. This is what lets the same propagator be found from either side,
  and it is why a t-channel line between beam 1 and leg 3 is looked up as the
  mask `{1,3}` when the clusterer merges them;
- `:264-289` sets `ipdgcl(mask, ignum, iproc)` from `sprop` if nonzero, else
  from `tprid`, else — only at the last level, `ipnum == 3` — from
  `ipdgcl(2, ignum, iproc)`, i.e. **beam 2's own PDG** (`:271-272`), because at
  the core vertex the "propagator" is beam 2's line;
- `:281` calls `filprp` (`:169-189`), which appends `ignum` to
  `id_cl(iproc, mask, ·)` if not already present, in **ascending config order**
  (this ordering is what makes `findmt`'s merge-intersection at `:493-511`
  correct);
- `:283-287` marks `resmap(mask, ignum)` when `prwidth(k, ignum) > 0`;
- `:292-316` collapses the pair into one line and returns, so the caller
  re-enters and the walk proceeds one vertex at a time until two lines remain.

`n_max_cl = 2^nexternal` (`export_v4.py:6087-6088`) and
`n_max_cg = nconfigs` (`:925-926`); `filprp` silently drops any mask above
`n_max_cl` (`:180`).

**(e) `findmt` selects, per candidate pair, the graphs still alive.**
`cluster.f:436-515`.

- First call in a clustering (`icgs(0) == 0`, `:464`): take
  `id_cl(iproc, idij, ·)`, filtered by (i) `chcluster` — keep only `iconfig`
  (`:468-470`), and (ii) the on-shell-BW constraint: a graph survives only if
  `resmap(ibwlist(1,j), graph)` holds for **every** tagged BW `j` (`:471-482`).
  Return true iff at least one graph survives.
- Later calls: **sorted-list intersection** with the running `icgs` (`:493-511`).
- A pair for which `findmt` is false is never given a measure at all
  (`cluster.f:598`, `:857`) and keeps its sentinel `pt2ij = 1e37`
  (`:597`, `:856`).

**Two hard structural rules on top of the graph:**

1. **The two beams are never combined.** `cluster.f:588` and `:841` gate the
   inner loop on `i > 2`, where `i` indexes `imap` and slots 1 and 2 are always
   the two beam lines. Together with the complement registration in (d), this is
   why the DY config's `{1,2}` mask exists in `id_cl` but is unreachable.
2. **Clustering fails if no pair is admissible.** `cluster.f:672-675` returns
   `cluster = .false.` when no winner was found, `reweight.f:667-677` turns that
   into `setclscales = .false.`, and `reweight.f:1907-1908` zeroes the event's
   weight. *Falsifier:* count zeroed events per run; a nonzero count on a banked
   run means the merge graph we derive is missing something MadGraph had.

*Falsifier for the whole of §K1.2:* K2 dumps the entire `id_cl` table
(`mask → sorted config list`) plus `ipdgcl` and `resmap`, once per
(process directory, `iproc`). K3's independently-derived table must be
**equal as a set of (mask, config-list) pairs**. This is the finest linear level
available for the merge graph and it is blind to nothing: a missing mask, an
extra mask, a wrong PDG assignment, and a wrong resonance flag are all visible
before a single momentum is clustered.

### K1.3 Winner selection and the tie-break order

`cluster.f:518-910`. State: `imap(i,1)` = original leg number of the `i`-th
surviving line, `imap(i,2)` = its mask; `pcl(0:4, mask)` its momentum + mass².

**Pass order.** First pass `:579-649`: outer `i = 1 … nexternal` (skipping
`i ≤ 2`), inner `j = 1 … i-1`. So candidates are visited in the order
`(3,1), (3,2), (4,1), (4,2), (4,3), (5,1), …`. Recompute passes `:838-906`
repeat the same nesting over the shrunken `imap`.

**Measure assignment** (identical in both passes, `:602-632` and `:861-890`):

```
j >= 3  (final-state pair)
        isbw(idij)          -> SumDot(pcl_i, pcl_j, +1)          :605 / :864
        ktscheme == 2       -> pydj(pcl_i, pcl_j)                :611 / :870
        else                -> dj(pcl_i, pcl_j)                  :613 / :872
        zij(idij) = 0                                            :616 / :875
j in {1,2}  (beam-leg pair; idi is the FS line, idj the beam)
        ickkw==2 or ktscheme==2 -> pyjb(...)                     :622 / :881
        else                -> djb(pcl_i)                        :626 / :884
                               zij(idij) = zclus(...)            :627 / :885
        THEN, unconditionally:
        if sign(1d0, pcl(3,idi)) /= sign(1d0, pcl(3,idj))
             pt2ij(idij) = pt2ij(idij) * (1d0 + 1d-6)            :630-631 / :888-889
```

**The `1 + 1e-6` inflation.** `cluster.f:630-631` (first pass) and `:888-889`
(recompute). Its comment is `prefer clustering when outgoing in direction of
incoming`. It compares the *sign of `p_z`* of the final-state line against the
sign of `p_z` of the beam line, using Fortran `sign(1d0, x)`, which returns
`+1` for `x ≥ +0.0` and `−1` for `x = −0.0` or `x < 0`. **Signed zero is
observable here** — our `fortran_sign` in `scales.rs` already models it.

Three properties that matter:

- It multiplies the *whole* measure, so on a genuine tie between two beam-leg
  candidates it is decisive; on a non-tie it changes the winner only if the two
  are within `1e-6` relative.
- It is applied to beam-leg candidates **only**. A final-state candidate is
  never inflated (except indirectly, when the massless–massive branch of `dj`
  returns `DJB(p)·(1+1e-6)` at `kin_functions.f:291,294` — a *different*
  `1 + 1e-6`, in a different routine, with a different reason).
- When **every** admissible beam-leg candidate is crossed, the inflation does
  not cancel: the minimum itself carries the factor. This is note 22 §1.3's
  `uux_to_uux` observation (10 of 10 000 events), and §K1.8 below traces the
  factor all the way to `SCALUP`.

**The comparison.** `:641-645` (and `:898-902`): strict `<` against a
`minpt2ij` initialised to `1.0d37` (`:578`, reset to `1.0d37` at `:837` after
every merge). Strictness means **an exact numerical tie is won by the
earlier-visited pair** in the `(i,j)` order above — smallest `i` first, then
smallest `j`. Combined with the sentinel value, a candidate whose measure is
`≥ 1e37` (including `+Infinity`) can never win.

*Falsifiers.* (a) Dump every candidate's `(i, j, idi, idj, idij, branch,
measure-before-inflation, inflated?, final pt2ij)` and the resulting
`(iwin, jwin)`. Any event where the winner is not the first minimiser in visit
order falsifies the tie-break reading. (b) Construct an event with two exactly
equal admissible beam-leg candidates both uncrossed; MadGraph must pick the
lower `(i,j)`. (c) `uux_to_uux`'s 10 crossed events must show *both* admissible
beam-leg candidates inflated; a single uncrossed candidate on any of them
falsifies §K1.8's derivation of `250.000125`.

### K1.4 The merge step, and what the "2 → 2 core" actually is

`cluster.f:677-907`, `nexternal - 2` iterations, but the loop **returns early**
from inside iteration `nexternal - 3`.

Per merge `n`:

```
imocl(n)   = imap(iwin,2) + imap(jwin,2)           mask of the mother      :679
idacl(n,1) = imap(iwin,2)   (the lower-index line) :680
idacl(n,2) = imap(jwin,2)                          :681
pt2ijcl(n) = minpt2ij                              :682   <- the winning measure
zcl(n)     = zij(imocl(n))                         :683
icluster(1..4,n,ivec) = original leg numbers + BW tag                    :689-700
igraphs    <- findmt(imocl(n), igraphs)            :702   (intersection; a
                                                          failure here is a
                                                          hard error, :703)
```

Then **initial-state** (`iwin < 3`, `:709-752`) versus **final-state**
(`:753-768`):

| | initial state | final state |
|---|---|---|
| mother momentum | `pcl(imo) = pcl(ida1) − pcl(ida2)` (`:721`) — spacelike | `pcl(imo) = pcl(ida1) + pcl(ida2)` (`:756`) |
| `mt2ij(n)` | `djb(pcl(idacl(n,2)))` — the **emitted final-state leg's** beam measure (`:712`) | left at 0 |
| mother mass² | 0, or `max` of the daughters' if either is nonzero (`:730-733`) | same rule (`:758-761`); overwritten by `pt2ijcl(n)` if `isbw(imo)` (`:762-767`) |
| frame | boost + rotate, conditionally (below) | none |

The mass²-propagation condition at `:731-733` (and its twin in
`PYJB`, `kin_functions.f:456-457`) reads
`A .or. B .and. .not.(A .and. B)`. Fortran binds `.not.` tighter than `.and.`
tighter than `.or.`, so it evaluates to `A .or. (B .and. .not.(A .and. B))`
≡ `A .or. B`. The parenthesised "only if exactly one is massive" exclusion is
**dead code**; the mother simply inherits `max(m₁², m₂²)`. Reproduce the
behaviour, not the comment. *Falsifier:* an event merging two massive legs
whose mother's `pcl(4)` is 0 rather than `max` would falsify it.

**The boost and rotation** (`:736-752`) fire on an initial-state merge when
both

```
pcmsp·pcmsp > 100 GeV²      with  pcmsp = -(pcl(imo) + pcl(iwinp)),  E-component re-negated  (:727-729)
nleft > 4                   nleft BEFORE the decrement at :770
```

hold. They boost **every surviving line** into the frame where the new spacelike
line and the spectator beam are at rest, then rotate (`constr`/`rotate`,
`cluster.f:18,64`) so the new incoming line lies along `+z`. Because `djb` and
`dj` are *not* invariant, every subsequent measure — and `pt2ijcl(nexternal-2)`
itself — is evaluated in this rotated frame. For `nexternal = 4` the `nleft > 4`
guard is never satisfied, which is exactly why every note 22 §1.3 closed form is
frame-free and why the six no-closed-form runs are not.
*Falsifier:* dump `pcmsp`, the `nleft` used, a fired/not-fired flag, and the
full `pcl(0:4, ·)` of all surviving lines after every merge. A 2 → 3 run in
which the boost never fires falsifies the reading of `:736`.

**Termination and the core.** After `nleft = nleft - 1` (`:770`) and the `imap`
compaction (`:772-776`), `:777` tests `nleft ≤ 3`. When it holds, three lines
remain — `imap(1,2)`, `imap(2,2)` (the two beam lines) and `imap(3,2)` (the
leftover blob) — and the routine writes one **synthetic last vertex** at index
`nc = nexternal - 2`:

```
if the last real merge was final-state (iwin > 2):                       :780-793
    mt2last = sqrt( djb(pcl(idacl(n,1))) * djb(pcl(idacl(n,2))) )        :781
    and, if the boost had fired, imap(3,2) is rotated and boosted back   :786-792
idacl(nc,1) = imap(1,2)      (beam-1 line)                               :795
idacl(nc,2) = imap(2,2)      (beam-2 line)                               :797  [imocl]
imocl(nc)   = imap(3,2)      (the leftover blob)
pt2ijcl(nc) = djb(pcl(imap(3,2)))                                        :799
zcl(nc)     = 1                                                          :824
if this_config is among igraphs, collapse igraphs to it alone            :811-817
```

So **"the 2 → 2 core" is `nc − 1` and `nc` read together**: the last *real*
merge (`n = nexternal - 3`, one blob against a beam or against another blob) plus
the terminal 2 → 1 vertex `nc`. The stored vertex `nc` is a 2 → 1 (two beam
lines into the leftover blob), not a 2 → 2; the K2 dump must record both indices
and K3 must not confuse them. Note also `mt2last` is only ever set when the last
real merge is **final-state**, and that the un-boost at `:786-792` is inside
that same branch — an event whose last real merge is initial-state keeps
`pt2ijcl(nc)` in the boosted frame.

`igraphs(1)` after `:811-817` is the graph whose `ipdgcl` column §K1.5 reads
every internal PDG from. When `this_config` survives the intersection it wins;
otherwise it is the lowest-numbered surviving config. **The clustering's PDG
assignment is therefore channel-dependent.** *Falsifier:* dump `iconfig`,
`igraphs(0)`, `igraphs(1..igraphs(0))` before and after `:811-817`, per event.
If `igraphs(1) == iconfig` on every dumped event of every run, the channel
dependence is inert for the bank and K3 may ignore it — but that must be
*measured*, not assumed.

**Special cases.** `nexternal == 3` with two incoming (`:651-668`) short-circuits
the whole thing: `pt2ijcl(1) = pcl(4, mask₃)`, `igraphs = {this_config}`. No
banked run is 2 → 1.

### K1.5 From the cluster sequence to `jfirst` / `jlast` / `jcentral`

`reweight.f:708-976`. This is the walk that decides *which vertices* the scales
are read off. Everything it asks is answered by `ipdgcl(·, igraphs(1), iproc)`.

**`ipart` — line provenance.** `:720-727` seeds `ipart(1, 2^(i-1)) = i` and
then calls `ipartupdate` (`reweight.f:224-441`) for `n = 1 … nexternal-3`
(**not** the terminal vertex). `ipart(1, mask)` is the beam number for a
t-channel line, and for an s-channel line the hardest constituent; `ipart(2,·)`
is the softer partner for gluon → qq̄ / octet / sextet / singlet splittings.
`ipartupdate` also *rewrites* `ipdgcl(imo)` for jet lines (`:275-292`,
`:300-309`), and `stop 3`s on any colour structure it does not recognise
(`:433-437`) — an honest signal that a process is outside the algorithm.

**Beam-side state.** `:730-748`:

```
ibeam(i)     = 2^(i-1)                    advanced to the mother at each IS merge (:772)
jfirst(i)    = 0     first IS splitting on side i
jlast(i)     = 0     last IS vertex at which side i was still a *parton* line
jcentral(i)  = 0     last IS vertex at which side i was still *QCD*
qcdline(i)   = isqcd(pdg(beam i))         |colour| > 1                    (reweight.f:155-166)
partonline(i)= qcdline(i)
goodjet(beam)= partonline(i)
goodjet(leg) = isjet(pdg(leg))  for legs 3..nexternal                     (:750-754)
```

`isjet(pdg)` (`:181-198`) is `|pdg| ≤ maxjetflavor or |pdg| == 21`. At the
default `maxjetflavor = 4`, **`b` and `t` are not jets** — that single fact
drives the `pp_to_bb` branch in §K1.8.

**The walk** `:766-951`, one iteration per `n = 1 … nexternal-2`:

*Initial-state case* — some `idacl(n,i)` equals `ibeam(j)` (`:770`). Roles:

```
n <  nexternal-2 :  ida(i)=idacl(n,i)  ida(3-i)=idacl(n,3-i)  imo=imocl(n)     :774-777
n == nexternal-2 :  ida(i)=idacl(n,i)  ida(3-i)=imocl(n)      imo=idacl(n,3-i) :779-781
```

i.e. at the terminal vertex the "line continuing past the vertex" is **the other
beam's line**. Then:

```
if partonline(j):  if jfirst(j)==0: jfirst(j)=n                                :786
                   jlast(j) = n                                                :788
                   partonline(j) = goodjet(ida(3-i)) .and. isjet(pdg(imo))     :789-790
elif jfirst(j)==0: jfirst(j)=n ; goodjet(imo)=.false.                          :791-794
else:              goodjet(imo)=.false.                                        :795-797
...
if qcdline(j):     jcentral(j) = n                                             :825
                   qcdline(j)  = isqcd(pdg(imo))                               :826
```

Note the **order**: `jlast`/`jcentral` are assigned *before* the predicate that
turns the flag off, so both point at the last vertex where the line was still
good, inclusive.

*Final-state case* `:831-950` — `isjetvx` (`:443-488`), jet tagging, `goodjet`
propagation, and the `g → gg` `ipart` repair at `:912-946`. None of it moves
`jfirst`/`jlast`/`jcentral`; it only sets `iqjets` (used by `xqcut` and
`ptclus`) and `goodjet` (used by the initial-state branch on later iterations).

**Post-processing.**

- `:958-968` demotes `iqjets` codes `≤ jcode` when a parton line has stopped.
  Scale-irrelevant at `xqcut = 0`.
- `:970-971` `if jfirst(j) ≤ 0 then jfirst(j) = jlast(j)`.
- `:985-1030` the **`njetstore` memo.** On the very first event of a process
  directory, `chcluster` is forced true (`:663-665`) so the first clustering is
  restricted to `iconfig`; the resulting jet count is stored
  (`:987-994`) and the event is **re-clustered unrestricted** (`goto 100`,
  `:998`). On every later event, if the unrestricted clustering yields a
  different `njets`, the event is re-clustered **restricted to `iconfig`**
  (`:1024-1028`). Consequences:
  - the scale is **not** a pure function of (momenta, process, channel): it also
    depends on which event arrived first in that process directory;
  - a mismatch that survives a restricted re-cluster is a hard `stop 4`
    (`:1011-1023`).
  *Falsifier:* dump, per event, `njetstore(iconfig)`, `njets`, the number of
  `cluster()` invocations and the `chcluster` value used for each. If the
  fallback at `:1027` never fires across the six no-closed-form runs, K3 may
  implement the pure function; if it fires even once, K3 must model the memo or
  restrict its claim to the events where it does not.

### K1.6 From `jlast` / `jcentral` to `μR` — the geometric-mean prescription

Two rewrites of `pt2ijcl` come first.

**(i) The `mt2last` override** (`:1034-1045`). Fires only when *all* of:
`mt2last > 4d0`, `nexternal > 3`, `jlast(1) == jlast(2) == nexternal-2`, and
`isqcd` of all three PDGs at the last **real** merge
(`idacl(nexternal-3,1)`, `idacl(nexternal-3,2)`, `imocl(nexternal-3)`).
Effect: `mt2ij(nexternal-2) = mt2ij(nexternal-3) = mt2last`. Recall
`mt2last = sqrt(djb(d₁)·djb(d₂))` from `cluster.f:781` — the geometric mean of
the two daughters' transverse masses squared, and set **only** when the last
real merge was final-state. `MT2LAST_FLOOR = 4.0` in `scales.rs:539` is this
`4d0`.

**(ii) The central-vertex override** (`:1048-1055`):
`if jcentral(j) > 0 and mt2ij(jcentral(j)) > 0 then pt2ijcl(jcentral(j)) = mt2ij(jcentral(j))`.
`mt2ij(n)` is nonzero only for initial-state merges (`cluster.f:712`) or after
(i). So: where a colour line ends on an initial-state vertex, the scale of that
vertex is replaced by **the emitted leg's transverse mass**, not the winning
merge measure.
Note which of the two feeders is exercised by the bank: §K1.8 shows that on
every closed-form row `jcentral` lands on the terminal vertex `nexternal-2`,
where `cluster.f:712` never wrote, so the *initial-state* feeder of (ii) is
**unexercised by any degenerate row** and reaches (ii) only through (i)'s
`mt2last` rewrite (row 4's `qq̄` channel). K3 gains its first coverage of the
initial-state feeder from the 2 → 3 and 2 → 4 dumps.

**(iii) The floor with `jfirst`** (`:1109-1112`), reached only after the early
return at `:1103-1106` declines (it declines for `-1` with free scales, since
`q2fact` and `scale` are all zero):

```
if jlast(1) > 0:  pt2ijcl(jlast(1)) = max(pt2ijcl(jlast(1)), pt2ijcl(jfirst(1)))
if jlast(2) > 0:  pt2ijcl(jlast(2)) = max(pt2ijcl(jlast(2)), pt2ijcl(jfirst(2)))
```

This is the channel through which the §K1.3 tie-break inflation reaches the
final scale (§K1.8).

**`μR` itself.** `reweight.f:1150-1174`, guarded by `scale == 0d0` — so a run
with `fixed_ren_scale = .true.` never enters, and a run with a *fixed* `μR` and
a *dynamic* `μF` still runs everything above. **These are the lines that pin the
form of the geometric mean:**

| condition | line | value |
|---|---|---|
| `jlast(1)>0 .and. jlast(2)>0` | **`:1153-1154`** | `(pt2ijcl(jlast1)·pt2ijcl(jcentral1)·pt2ijcl(jlast2)·pt2ijcl(jcentral2))**0.125` |
| `jlast(1)>0` | **`:1157`** | `(pt2ijcl(jlast1)·pt2ijcl(jcentral1))**0.25` |
| `jlast(2)>0` | **`:1160`** | `(pt2ijcl(jlast2)·pt2ijcl(jcentral2))**0.25` |
| `jcentral(1)>0 .and. jcentral(2)>0` | **`:1163`** | `(pt2ijcl(jcentral1)·pt2ijcl(jcentral2))**0.25d0` |
| `jcentral(1)>0` | **`:1165`** | `sqrt(pt2ijcl(jcentral1))` |
| `jcentral(2)>0` | **`:1167`** | `sqrt(pt2ijcl(jcentral2))` |
| else | **`:1169`** | `sqrt(pt2ijcl(nexternal-2))` |

then `scale = scalefact*scale` (`:1171`) and
`G = sqrt(4π·ALPHAS(scale))` (`:1173`).

Every exponent is the *geometric mean of squared scales* — `0.125` over four
factors, `0.25` over two, `0.5` over one — so `μR` is always the geometric mean
of the participating **linear** scales. Note 22 §1.3 is right that no banked run
distinguishes `:1153` from `:1157` etc. (they coincide when the factors are
equal); these seven lines are the pin, and K3 must implement the *branch
selection*, which the six no-closed-form runs will exercise.

*Falsifier:* dump the branch index taken at `:1151-1168`, the four (or two, or
one) `pt2ijcl` values entering it, `scale` before and after `:1171`, and the
`<rscale>` MadGraph then writes. Recomputing the formula from the dumped inputs
must reproduce the dumped output bit-for-bit modulo the last ulp of `**0.125`.
A run in which two different branch indices produce the same `scale` for the
same inputs would mean the branch is unobservable and must be reported as such.

### K1.7 From `jlast` / `jcentral` to per-beam `μF`

There is one factorisation scale **per beam**, `q2fact(1)` and `q2fact(2)`, with
independent `fixed_fac_scale1` / `fixed_fac_scale2` guards throughout. Order of
the branches, all in `setclscales`:

1. `:1121-1124` — `nexternal == 3 .and. nincoming == 2`: both beams get
   `pt2ijcl(nexternal-2)`. Unreachable for the bank.
2. `:1126-1137` — **the main branch**, entered when either `q2fact` is still 0:
   ```
   if jlast(1)>0 and not fixed_fac_scale1:
       q2fact(1) = sqrt( pt2ijcl(jlast(1)) * pt2ijcl(jcentral(1)) )     :1128
   if jlast(2)>0 and not fixed_fac_scale2:
       q2fact(2) = sqrt( pt2ijcl(jlast(2)) * pt2ijcl(jcentral(2)) )     :1129
   if jcentral(1)>0 and jcentral(1)==jcentral(2)
      and neither beam fixed:
       q2fact(1) = max(q2fact(1), q2fact(2)) ; q2fact(2) = q2fact(1)    :1130-1136
   ```
   Note the shape: `q2fact` is a *squared* scale and the right-hand side is the
   `sqrt` of a product of two squared scales, so `μF = (pt2_jlast · pt2_jcentral)^{1/4}`
   — the same geometric mean as `μR`, over two factors instead of four. The
   `:1130-1136` collapse ("a qcd line going through the whole event, use single
   scale") is what makes note 22's "`μF` on **both** beams is the same number"
   true for `pp_to_ll`.
3. `:1138-1147` — `scalefact²` (see §K1.9) and the `q2bck` back-up.
4. `:1180-1194` — the `jcentral == 0` fill-ins, run **after** `μR`:
   ```
   jcentral(1)==0 and jcentral(2)==0:                                    :1180
       q2fact(1)>0 and not ffs1 -> pt2ijcl(nc)=pt2ijcl(nc-1)=q2fact(1)   :1181-1183
       elif q2fact(2)>0 and not ffs2 -> same from beam 2                 :1184-1186
       else -> q2fact(1) = scalefact**2 * pt2ijcl(nexternal-2)           :1188
               q2fact(2) = scalefact**2 * pt2ijcl(nexternal-2)           :1189
   elif jcentral(1)==0: q2fact(1) = scalefact**2*pt2ijcl(jfirst(1))      :1192
   elif jcentral(2)==0: q2fact(2) = scalefact**2*pt2ijcl(jfirst(2))      :1194
   elif ickkw==2 or (pdfwgt and ickkw>0): ...                            :1195-1203
   ```
   The first two sub-branches only **back-fill `pt2ijcl`** for the downstream
   `ptclus`/matching bookkeeping; they do not change `q2fact`.
5. `:1206-1220` — a hard floor: if a beam carries a PDF and its dynamic
   `q2fact < 4 GeV²`, the event is **rejected** (`setclscales = .false.`,
   warning printed at most 10 times). *Falsifier:* count rejections per run; a
   nonzero count on a banked run means events were dropped for a reason our
   engine does not model.

`SCALUP` in the written event is **not** `μR`: `unwgt.f:750-756` fills it with
`sqrt(max(q2fact(1), q2fact(2)))`, falling back to whichever beam is nonzero.
`<rscale>` (`unwgt.f:775-777`) carries `s_scale(ivec) = scale` from
`reweight.f:1275`, and `<pdfrwt beam="j">` (`unwgt.f:792-828`) carries
`s_qpdf(1,j) = sqrt(q2fact(j))` from `reweight.f:1457`/`:1481` — for `ickkw ≤ 0`
the assignment at `:1457` is the live one (the `goto 100` at `:1461` skips the
rest of `rewgt`). All three confirm note 22 §1.4's corrections against 3.7.1.

### K1.8 Reconciliation with note 22 §1.3's measured collapse table

The general path must reduce, event for event, to each closed form already in
production (`vibegraph-lib/src/coupling/scales.rs:422-529`). Derivations below;
each is a consistency check K4 turns into a test.

Common facts used throughout: for a **2 → 2** the two outgoing legs carry
exactly opposite transverse momenta, so `p_T3 = p_T4`; equal masses then give
`djb₃ = djb₄` **exactly** at `lpp ≠ 0` (`m² + p_T²`) and, in the partonic CM at
`lpp = 0`, `E₃ = E₄ = √ŝ/2` gives `djb₃ = djb₄ = ŝ/4`. And `nexternal = 4` means
the boost of §K1.4 can never fire (`nleft > 4` is false at the only merge).

**Row 1 — `gg_to_gg`, `gg_to_ttx` (`lpp = 0`) → `(djb₃·djb₄)^{1/4} = √ŝ/2 = 250`.**
Merge graph: s, t, u configs (the VVVV contact is dropped by `export_v4.py:2193-2197`),
so all four beam-leg masks are admissible. Candidates:
beam-leg `= djb(leg) = ŝ/4 = 62500`, possibly `×(1+1e-6)`;
final-state `= dj = 2·min(E₃²,E₄²)(1−cos θ₃₄) = 4·(ŝ/4) = 250000` (back-to-back).
Beam-leg wins by a factor 4. Whichever leg is forward pairs with beam 1
uninflated, so the minimum is `62500` exactly. After the merge `nleft = 3`:
`pt2ijcl(1) = 62500`, `mt2ij(1) = djb(leg merged) = 62500`,
`pt2ijcl(2) = djb(leftover leg) = 62500`.
`setclscales`: coloured beams ⇒ `qcdline = partonline = .true.`; at `n = 1` the
merging beam gets `jfirst = jlast = jcentral = 1`; at `n = 2` (terminal) both
beams get `jlast = jcentral = 2` (for `gg_to_ttx`, `isjet(6) = .false.` stops
`partonline` **after** `jlast` was already set to 2 — the assignment precedes the
predicate, `:788` before `:789`). `mt2ij(2) = 0`, and `mt2last = 0` because the
last real merge was initial-state, so neither override fires. `:1109` is a no-op
(both `62500`). Then `:1153` gives `scale = (62500⁴)^{0.125} = 250` and
`:1128-1136` gives `q2fact(1) = q2fact(2) = 62500`, `μF = 250` on both beams. ✔
The note's `(djb₃·djb₄)^{1/4}` form is a coincidence of `djb₃ = djb₄`: the code
actually evaluates `(pt2ijcl(2)⁴)^{0.125}`.

**Row 1b — `uux_to_uux` and the `250.0001` events.** Flavour admits only the
masks `{1,3}` (t-channel gluon) and `{3,4}` (s-channel gluon), so the beam-leg
candidates are exactly `(i=3,j=1)` and `(i=4,j=2)` — one per leg, one per beam.
If leg 3 is forward, `(3,1)` is uninflated and wins at `62500`. If leg 3 is
backward, leg 4 is forward, so `(3,1)` **and** `(4,2)` are both crossed; the
minimum is `62500·(1+1e-6)` and the earlier-visited `(3,1)` wins. Then
`pt2ijcl(1) = 62500·(1+1e-6)` while `pt2ijcl(2) = djb(leg 4) = 62500` — the
leftover leg's own measure is *not* inflated. The inflation reaches the answer
through `:1109`: `jfirst = 1`, `jlast = jcentral = 2`, so
`pt2ijcl(2) ← max(62500, 62500·(1+1e-6)) = 62500·(1+1e-6)`, and
`μR = μF = √(62500·(1+1e-6)) = 250·(1+5·10⁻⁷) = 250.000125`.
That is `scales.rs`'s `q2 *= TIE_BREAK` (`:480`) applied to the squared scale,
and it reproduces note 22's "`SCALUP` reads `250.0001`" (the `e15.7` field prints
`2.5000012E+02`). ✔
**This is the row that pins the tie-break**, and §K1.3's falsifier (b) is its
firing test.

**Row 2 — `ee_to_ee`, `ee_to_wpwm` (`lpp = 0`, colourless beams) → `250`.**
Same clustering, but `isqcd(e⁻) = .false.` ⇒ `qcdline = partonline = .false.`
from the start ⇒ `jfirst = jlast = jcentral = 0` on both sides. `:1128-1129` is
skipped (`jlast = 0`), `:1130` is skipped (`jcentral = 0`), so `q2fact` stays 0;
`μR` falls through six branches to `:1169`, `scale = √pt2ijcl(nexternal-2) = 250`;
and `μF` is filled at `:1188-1189` from the same `pt2ijcl(nexternal-2)`.
Crucially, `:1109` never runs (`jlast = 0`), so the tie-break inflation on
`pt2ijcl(1)` — which *is* present for a crossed `e⁺e⁻ → e⁺e⁻` event — never
reaches the scale. That is precisely `scales.rs`'s
`if crossed && topology.coloured_beams` guard (`:479`), now explained rather
than fitted. ✔

**Row 3 — `ee_to_mumu`, `ee_to_ttx`, `ee_to_zh`, `ee_to_tatah`, `uux_to_mumu`
(`lpp = 0`) → `√(djb(Σp_out)) = √ŝ = 500`.**
Every diagram is s-channel, so the only admissible mask is `{3,4}`; it wins by
being the only candidate. `pt2ijcl(2) = djb(p₃+p₄) = E_tot² = ŝ` at `lpp = 0`. Then:
- colourless beams (`ee_*`): `jlast = jcentral = 0`, so `:1169` gives
  `scale = √ŝ = 500` and `:1188-1189` gives the same `μF`. `ee_to_ttx` takes this
  branch **despite a coloured final state**, because `jcentral` tracks the
  *beam* colour line; note 22's "discriminating row" is confirmed, and the
  reason is `qcdline(j) = isqcd(pdg(beam j))` at `reweight.f:741`, not anything
  about the final state.
- coloured beams (`uux_to_mumu`): `jlast(1) = jlast(2) = jcentral(1) =
  jcentral(2) = 2`, so `:1153` gives `(ŝ⁴)^{0.125} = √ŝ = 500` and `:1128-1136`
  gives `q2fact = ŝ` on both beams. The `mt2last` override does **not** fire
  because `isqcd(μ±) = .false.` at `:1036-1037`. Same number by a different
  route — exactly the degeneracy note 22 flagged. ✔

**Row 4 — `pp_to_bb`, `pp_to_bb_qcd2` (`lpp = 1`) → `√(m_T(b)·m_T(b̄))`.**
Two subprocess channels, and note 22's correction (that the `gg` channel wins by
a beam-leg merge, so `mt2last` is *not* what produces the number) is confirmed —
**and the `qq̄` channel reaches the same number through `mt2last`.**
- `gg → bb̄`: s, t, u configs, all four beam-leg masks admissible. Final-state
  candidate `dj = m_b² + p_T²·2(cosh Δη + 1) ≥ m_b² + 4p_T²`, beam-leg candidate
  `djb = m_b² + p_T²`; beam-leg wins for any `p_T > 0`. `isjet(5) = .false.`
  (`maxjetflavor = 4`), so `partonline` on the merging side dies **after**
  `jlast = 1` is recorded, while `qcdline` survives (the t-channel line is a
  `b`, `isqcd = .true.`). Result: `jlast(1) = 1`, `jcentral(1) = 2`,
  `jlast(2) = jcentral(2) = 2`. `:1128` gives
  `q2fact(1) = √(djb₃·djb₄)`, `:1129` gives `q2fact(2) = djb₄`, and the
  `jcentral(1) == jcentral(2)` collapse at `:1130` takes the max — equal, since
  `djb₃ = djb₄`. `:1153` gives
  `(djb₃·djb₄·djb₄·djb₄)^{0.125} = √djb = m_T(b)`. ✔
- `qq̄ → bb̄`: only the s-channel config, so `{3,4}` is the only mask and the
  merge is final-state. `isjetvx` is false (`isjet(b)` false), so
  `goodjet(blob) = .false.` and both beams get
  `jlast = jcentral = nexternal-2 = 2`. Now `mt2last = √(djb₃·djb₄)` **is** set
  (last real merge was final-state, `cluster.f:781`), it exceeds `4`, and all
  three PDGs at the last real merge (`b`, `b̄`, `g`) are `isqcd` — so the
  override at `:1034-1045` fires and `:1048-1055` sets
  `pt2ijcl(2) = mt2last = √(djb₃·djb₄)`. Then `:1153` gives
  `(mt2last⁴)^{0.125} = √mt2last = (djb₃·djb₄)^{1/4} = √(m_T(b)·m_T(b̄))`, which
  equals the `gg` channel's `m_T(b)` because `djb₃ = djb₄`. ✔
  This is `scales.rs`'s `coloured_central_line` branch (`:510-523`), and the
  reconciliation shows why the two production branches agree on this row: the
  degeneracy `djb₃ = djb₄` is exact for a 2 → 2 with equal-mass legs, so the
  *form* of the geometric mean stays unpinned — note 22 §1.3's conclusion,
  re-derived from the general path.

**Row 5 — `pp_to_ll`, `pp_to_ll_qcd0` (`lpp = 1`) → `m(ℓℓ)`, same on both beams.**
s-channel only ⇒ mask `{3,4}` only. `pt2ijcl(2) = djb(p₃+p₄) = m² + p_T²(pair)`,
and `p_T(pair) = 0` exactly for a 2 → 2, so it is `m²(ℓℓ)`. Coloured beams give
`jlast(1) = jlast(2) = jcentral(1) = jcentral(2) = 2`; `mt2last` is set but the
override is blocked by `isqcd(ℓ) = .false.`. `:1153` gives `m(ℓℓ)`, `:1128-1136`
gives `q2fact(1) = q2fact(2) = m²(ℓℓ)` — the `:1130` collapse is what makes both
beams identical. ✔ (Whether the `{3,4}` measure was `dj` or the `isbw` invariant
mass is irrelevant: it is the only candidate, and `pt2ijcl(1)` never reaches the
answer because `jfirst = jlast = 2`.)

**Row 6 — `bbx_to_ccx_emmm_qcd0`, `uux_to_ccx_emmm_qcd0`, `pp_to_llj{,_qcd2_qed2}`,
`ee_to_mumua`, `ee_to_mumu_tata_qcd0` — no closed form.** These are
`nexternal = 5` and `6`, so (i) more than one real merge, (ii) the boost of
§K1.4 can fire, (iii) `jlast ≠ jcentral` is generic and the four-factor branch
at `:1153` becomes observable, and (iv) note 22's own observation that
`bbx_to_ccx_emmm_qcd0` shows 8720 distinct `SCALUP` over 10k events at fixed
`√ŝ = 500` and `lpp = 0` is explained: `djb = E²` in a frame that the boost at
`cluster.f:736-752` has moved. These six are exactly K2's dump targets.

**Summary of the reduction.** Every closed form in production is the general
path evaluated on a 2 → 2 (no boost, `djb₃ = djb₄`) with one of four
`(jlast, jcentral)` shapes: both zero (`:1169` + `:1188`), both equal and
nonzero (`:1153` + `:1130`), `jlast(1) < jcentral(1)` (`:1153` with a split
first factor), or the `mt2last` rewrite feeding `:1048`. K4 must reproduce all
five rows event-for-event before wiring, as the plan's §K4 already requires.

### K1.9 Where `scalefact` lands — corrected against 3.7.1

Note 22 §1.1's table was read off 3.5.7 and **one entry is now wrong.**
`git diff v3.5.7 v3.7.1 -- Template/LO/SubProcesses/reweight.f` shows exactly one
non-vectorisation change in this region:

```
-            if(.not.fixed_fac_scale2) q2fact(2)=scalefact**2*q2fact(1)      (3.5.7)
+            if(.not.fixed_fac_scale2) q2fact(2)=scalefact**2*pt2ijcl(nexternal-2)   (3.7.1, :1189)
```

In 3.5.7 beam 2 in the "no colour line reaches the beams" branch was built from
an **already-scaled** `q2fact(1)` and picked up `scalefact` twice. In 3.7.1 it is
built from `pt2ijcl(nexternal-2)` like beam 1, so both beams carry exactly one
factor. The corrected table for `dynamical_scale_choice = -1` on 3.7.1:

| path | where `scalefact` enters | net power on `μR` | on `μF(1)` | on `μF(2)` |
|---|---|---|---|---|
| `fixed_ren_scale = .true.` | `set_ren_scale` is never called (`cuts.f:1233`) | 0 | — | — |
| `fixed_fac_scaleN = .true.` | that beam's assignments are all guarded | — | 0 | 0 |
| dynamic 1–5 (for contrast) | `setscales.f:93` once, then squared into `q2fact` at `:184-185` | 1 | 1 | 1 |
| `-1`, `μR` | `reweight.f:1171` | 1 | — | — |
| `-1`, `μF`, main branch `:1128-1137` | `:1139-1140` (`scalefact²` on `q2fact`) | — | 1 | 1 |
| `-1`, `μF`, `jcentral(1)=jcentral(2)=0`, both `q2fact` still 0 | `:1188-1189` | — | 1 | **1** (was **2** in 3.5.7) |
| `-1`, `μF`, `jcentral` both 0 but one `q2fact > 0` | `:1181-1186` back-fills `pt2ijcl` only and leaves `q2fact` alone — but the guards read `q2fact(j) > 0 .and. .not.fixed_fac_scaleN`, and with `-1` and a free beam `q2fact(j)` is still 0 here, so this pair of sub-branches is **unreachable** for the bank | — | n/a | n/a |
| `-1`, `μF`, `jcentral(1)=0` only | `:1192` **replaces** `q2fact(1)` | — | 1 | 1 |
| `-1`, `μF`, `jcentral(2)=0` only | `:1194` | — | 1 | 1 |
| `-1`, `fixed_fac_scale1=.true.`, `fixed_fac_scale2=.false.` | the guard at `:1138` reads `(.not.ffs1 .or. ffs2)` = false, so the whole block is skipped and **beam 2 never gets `scalefact²`** | — | — | **0** |

So on 3.7.1, in every reachable branch, `μR` and both `μF` carry **exactly one**
power of `scalefact` — with the single exception of the last row, which our
`ScaleError::MixedFixedFactorisationScales` already refuses.

**Action for K4:** `scales.rs`'s `beam2_from_beam1` field
(`vibegraph-lib/src/coupling/scales.rs:319-323`, `:484`, `:505`, `:521`, `:527`, `:566`)
implements the 3.5.7 double factor and must be **deleted**, not carried forward.
It is unpinned by reference data — every banked run has `scalefact = 1.0` — so
nothing will catch this but the reading. The last table row is likewise a
reading-only claim.

*Falsifier for the whole table:* a single MadGraph run at
`scalefact = 2.0` (any process, any of the five closed-form rows) settles every
entry at once by comparing `<rscale>` and both `<pdfrwt>` scales against the
`scalefact = 1.0` bank. It costs one MG run and it is the only way to make these
claims non-vacuous. **Recommendation: fold it into Sb** as a cheap extra card
(a `pp_to_ll` re-run at `scalefact = 2.0`, 1k events) rather than leaving the
table pinned to prose. Without it, K4 must land the `beam2_from_beam1` deletion
with a unit test that asserts the *reading*, and the note must say so.

### K1.10 The K2 dump: what an instrumented run must record

**Alignment with the bank.** The dump must be **per written event, in write
order**. `setclscales` runs on every phase-space point, not only accepted ones,
so the instrumentation must write into a per-`ivec` buffer and let
`write_leshouche` flush it at the moment the event is emitted — mirroring
exactly how `use_syst` already carries `s_scale(ivec)`/`s_qpdf(·,·,ivec)` from
`reweight.f:1274-1282` to `unwgt.f:775-828`. The `k`-th dump record then
corresponds to the `k`-th `<event>` in `unweighted_events.lhe`.

**Precondition K2 must establish first** (before any comparison): re-running the
banked cards through the *instrumented* build must reproduce
`unweighted_events.lhe` byte-for-byte against the banked file, modulo run
metadata (timestamps, paths, the new dump). If it does not, the seed handling is
not what we think and the whole "reproduce the banked `SCALUP` for every event"
gate is measuring the wrong thing. *This check comes before the dump is
trusted.*

**Format.** Line-oriented, tagged, one file per run, `%24.17E` for every real
(17 significant digits — the LHE's 10 are not enough to replay a `(E−p_z)(E+p_z)`
cancellation). Records are pipe-separated `TAG|field|field|…`. A Python driver in
`validation/madgraph/wrappers/` normalises to JSONL; the Fortran side stays
dumb. Grammar:

```
# once per process directory, at initcluster (cluster.f:379 / initcluster.f:48)
RUN  |<run tag>|nexternal|nincoming|maxsproc|mapconfig(0)
CONST|lpp1|lpp2|D|maxjetflavor|ktscheme|ickkw|chcluster|pdfwgt|xqcut|xmtc
     |scalefact|fixed_ren_scale|fixed_fac_scale1|fixed_fac_scale2|bwcutoff
     |dynamical_scale_choice|use_syst|stot
NQCD |<iconfig>|nqcd(iconfig)                                    ... one per config
MAP  |<iproc>|<mask>|<n>|<g1>|<g2>|...|<gn>                      ... one per nonempty id_cl
PDG  |<iproc>|<mask>|<graph>|<ipdgcl>                            ... one per assigned entry
RES  |<mask>|<graph>                                             ... one per resmap true
IFOR |<iconfig>|<k>|<iforest(1,k)>|<iforest(2,k)>|<sprop>|<tprid>|<prmass>|<prwidth>

# per written event
EVT  |<index>|<iproc>|<iconfig>|<ivec>|<imirror>
MOM  |<i>|E|px|py|pz|pcl4          ... i = 1..nexternal, as passed to cluster()
BW   |<nbw>|<ibwlist(1,1)>|<ibwlist(2,1)>|...
CLCALL|<attempt>|<chcluster used>|<returned>       ... one per cluster() invocation
                                                       (reweight.f:666, :998, :1028)
# ---- per clustering attempt, per pass p = 0 (first) .. nexternal-4 (recompute) ----
CAND |<attempt>|<p>|<i>|<j>|<leg_i>|<leg_j>|<idi>|<idj>|<idij>|<admissible?>
     |<branch>|<raw measure>|<inflated?>|<pt2ij>|<zij>|<ngraphs after findmt>
       branch in {IS_DJB, IS_PYJB, FS_DJ_DURHAM, FS_DJ_HAD, FS_DJ_MLESS_MASSIVE,
                  FS_SUMDOT_BW, FS_PYDJ, NONE}
WIN  |<attempt>|<p>|<iwin>|<jwin>|<minpt2ij>
# ---- per merge step n = 1 .. nexternal-3 ----
MRG  |<attempt>|<n>|<idacl(n,1)>|<idacl(n,2)>|<imocl(n)>
     |<leg list of idacl(n,1)>|<leg list of idacl(n,2)>          ; original leg numbers
     |<kind: IS|FS>|<pt2ijcl(n)>|<zcl(n)>|<mt2ij(n)>
     |<ipdgcl(idacl(n,1))>|<ipdgcl(idacl(n,2))>|<ipdgcl(imocl(n))>
     |<icluster(1..4,n,ivec)>
BOOST|<attempt>|<n>|<fired?>|<nleft used>|pcmsp0..3|<pcmsp^2>
PCL  |<attempt>|<n>|<mask>|E|px|py|pz|m2                        ; every surviving line, after the step
# ---- the core ----
CORE |<attempt>|<nc = nexternal-2>|<idacl(nc,1)>|<idacl(nc,2)>|<imocl(nc)>
     |<pt2ijcl(nc)>|<mt2last>|<unboost applied?>
GRPH |<attempt>|<before|after>|<igraphs(0)>|<igraphs(1)>|...     ; around cluster.f:811-817
# ---- setclscales ----
IPART|<mask>|<ipart(1,mask)>|<ipart(2,mask)>                     ; after the :724-727 walk
LINE |<mask>|<ipdgcl>|<isqcd>|<isjet>|<goodjet>                  ; every line the walk touched
JETS |<jcode>|<njets>|<njetstore(iconfig)>|<iqjets(3)>|...|<iqjets(nexternal)>
JIDX |<raw|final>|<jfirst1>|<jfirst2>|<jlast1>|<jlast2>|<jcentral1>|<jcentral2>
                                                                  ; raw = before :970, final = after
OVR  |<mt2last override fired?>|<jcentral override 1 fired?>|<jcentral override 2 fired?>
PT2  |<stage>|<n>|<pt2ijcl(n)>       stage in {AFTER_CLUSTER, AFTER_MT2_OVERRIDE,
                                              AFTER_JFIRST_MAX, FINAL}
MUF  |<branch id>|<q2fact1 before :1139>|<q2fact2 before>|<q2fact1 after>|<q2fact2 after>
       branch id in {NEXT3, GEOM, GEOM_COLLAPSED, JC0_BACKFILL1, JC0_BACKFILL2,
                     JC0_BOTH, JC0_BEAM1, JC0_BEAM2, PDFWGT, NONE}
MUR  |<branch id>|<f1>|<f2>|<f3>|<f4>|<scale before :1171>|<scale after>
       branch id in {L1153, L1157, L1160, L1163, L1165, L1167, L1169, NOT_ENTERED}
       f1..f4 = the pt2ijcl values entering the chosen formula (0 for unused slots)
REJ  |<xqcut failed?>|<xmtc failed?>|<mufloor failed?>
OUT  |<scale>|<q2fact1>|<q2fact2>|<SCALUP>|<alphas(scale)>|<G>
SYST |<s_scale>|<n_qcd>|<s_qpdf1>|<s_qpdf2>|<s_xpdf1>|<s_xpdf2>|<i_pdgpdf1>|<i_pdgpdf2>
END  |<index>
```

**Instrumentation points**, all in `Template/LO`:

| record | file:line |
|---|---|
| `RUN`,`CONST`,`NQCD`,`MAP`,`PDG`,`RES`,`IFOR` | `initcluster.f:48` (after `filmap`) |
| `EVT`,`MOM` | `cluster.f:554` (entry) |
| `BW` | `cluster.f:567-569` |
| `CLCALL` | `reweight.f:662-666`, `:998`, `:1027-1028` |
| `CAND` | `cluster.f:598-646` and `:857-903` (emit also on `findmt` false) |
| `WIN` | `cluster.f:641-645`, `:898-902` |
| `MRG`,`PCL` | `cluster.f:679-700`, `:712`, `:721`, `:755-767`, `:770-776` |
| `BOOST` | `cluster.f:736-752` |
| `CORE`,`GRPH` | `cluster.f:777-832` |
| `IPART`,`LINE` | `reweight.f:720-754` and inside the `:766-951` walk |
| `JETS`,`JIDX` | `reweight.f:958-976` (both stages) |
| `OVR`,`PT2` | `reweight.f:1034-1055`, `:1109-1112` |
| `MUF` | `reweight.f:1121-1147`, `:1180-1203` |
| `MUR` | `reweight.f:1150-1176` |
| `REJ` | `reweight.f:1081`, `:1094`, `:1217` |
| `OUT`,`SYST` | `reweight.f:1274-1282`; `SCALUP` from `unwgt.f:750-756` |

**Runs to dump.** The six no-closed-form rows —
`pp_to_llj`, `pp_to_llj_qcd2_qed2`, `ee_to_mumua`, `ee_to_mumu_tata_qcd0`,
`bbx_to_ccx_emmm_qcd0`, `uux_to_ccx_emmm_qcd0` — plus **three known-good
controls** chosen so that each closed form in §K1.8 is covered by one:
`uux_to_uux` (the tie-break row, and the only one whose 10 crossed events
exercise `:1109`), `pp_to_bb_qcd2` (both the `gg` beam-leg route and the `qq̄`
`mt2last` route in one run), and `ee_to_ttx` (the `jcentral = 0` /
coloured-final-state discriminator). Nine runs × 10k events.

**K2's gate**, restated with what it can and cannot see: the dump's `OUT` record
must reproduce the run's own `SCALUP`/`<rscale>`/`<pdfrwt>` for **every** event —
that proves the instrumentation reads the live path — and, given the byte-identity
precondition above, those equal the banked values. What this gate is blind to:
it cannot detect an error in `CAND`/`MRG`/`PT2` that does not change the final
scale (a wrong tie-break on an event where both candidates give the same measure,
a wrong `ipdgcl` on a line the walk never asks about). That blindness is why the
dump records the merge sequence at all, and why K3 is judged on
**sequence identity first, scale identity second** — the plan's §K3 ordering is
the right one, and this is the reason.

### K1.11 Findings for downstream sessions

1. **`beam2_from_beam1` is 3.5.7 behaviour and must be deleted in K4** (§K1.9).
   Unpinned by any reference data at `scalefact = 1.0`.
2. **The scale is not a pure function of (momenta, process).** Three channel /
   history dependencies exist: `filmap`'s `nqcd(this_config)` filter
   (`cluster.f:360`), `checkbw`'s use of `this_config` (`cluster.f:419`), and the
   `njetstore` memo with its restricted re-cluster (`reweight.f:985-1030`).
   K2 must dump `iconfig`, `igraphs(1)`, and the memo state so K3 can *measure*
   whether they are inert on the bank rather than assume it. If any of them is
   live, the six `validate_scales` rows cannot be replayed from an LHE record
   alone and K4's scope changes.
3. **Four-point vertices are absent from MadGraph's merge graph**
   (`export_v4.py:2193-2197`). Our diagram enumeration keeps them; K3's merge-map
   derivation must drop any diagram whose maximum vertex arity exceeds the
   process minimum.
4. **`isjet` uses `maxjetflavor` (default 4), so `b` and `t` are not jets**, and
   the assignment order at `reweight.f:788-790` means `jlast` records the vertex
   *at which* the parton line stopped, inclusive. Both are load-bearing for
   `pp_to_bb`.
5. **`Template/LO` never assigns `D`**; it arrives from the hidden run-card
   parameter `d = 1.0`. Assert it in the dump.
6. **The `A .or. B .and. .not.(A .and. B)` mass-propagation guard is dead code**
   (`cluster.f:731-733`, `kin_functions.f:456-457`) — implement `A .or. B`.
7. **`clusinfo` is gated on `ickkw ≠ 0`** (`unwgt.f:838`), so no banked LHE
   carries a `<clustering>` tag; instrumentation is the only route to the merge
   sequence. Confirms the plan's oracle-before-engine ordering is necessary and
   not merely tidy.
8. **A `scalefact ≠ 1` reference run is the only thing that would make §K1.9
   non-vacuous.** Recommended as a cheap Sb addition; otherwise K4 lands the
   correction against a reading, and the note says so.
