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

Z also executes **D4 (§6)**: prune the duplicate `pp_to_llj_qcd2_qed2` —
manifest rows, gate registrations, reference entries, and its bundle
membership — keeping `pp_to_llj` as the one dynamical-scale llj row.

## 6. Decisions (user — resolved 2026-08-01, recommended options adopted)

1. **D1 — capstone card (decided)**: `p p > j j` with MadGraph's default run
   card (dynamical scale, default `ptj`). No fixed-scale stepping-stone row.
2. **D2 — spine reference process (decided; revised 2026-08-01)**: S2's
   measurement deviated from the `p p > e+ e- j j` candidate (112
   subprocesses / 3024 diagrams / 1608 pooled channels — §S2.6) and the user
   approved the narrowed partonic spelling **`u d > e+ e- u d QCD=0`** at a
   fixed scale, which carries the full ladder spectrum (12/14/9 diagrams over
   one/two/three spacelike lines) in one flavour assignment. Card:
   `validation/madgraph/scripts/ud_to_epemud_qcd0.mg5`; banked by Sb.
3. **D3 — massless-t-channel cut (decided)**: resolved inside S2 by
   measurement (flat-fallback vs fiducially-bounded `t_max`); the decision and
   its numbers land in this note.
4. **D4 — duplicate-run pruning (user, 2026-08-01)**: `pp_to_llj_qcd2_qed2`'s
   banked run is event-for-event identical to `pp_to_llj` (K2 found it;
   verified independently — equal sha over the event payloads: the `QCD=2
   QED=2` restriction coincides with the default orders for this process), so
   the census counts one measurement twice. Keep `pp_to_llj`; prune
   `pp_to_llj_qcd2_qed2` at Z — manifest rows, gate registrations, reference
   entries, and the refdata-4 bundle. Coupling-order grammar coverage is
   retained by the other order-restricted rows (`ee_to_mumu_tata_qcd0`,
   `bbx_to_ccx_emmm_qcd0`, …). Until Z, sessions treat the pair as one
   independent row: K4 enforces on `pp_to_llj`, and the duplicate earns no
   separate cell anywhere.

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
## S1 — channel-enumeration decision (identical particles)

**Decision.** Multichannel treats a repeated outgoing species as **one channel set
over the full labelled `dΦ_n`**, not as permutation copies. The enumeration stays
*one channel per diagram*; the maps keep sampling every permutation of the
identical legs as a distinct configuration; and `1/Π_s n_s!` is a **per-subprocess
scalar** multiplying that subprocess's term of the summed matrix element. Nothing
about channel enumeration changes when a final state repeats a species — this is
the shape the performance sprint can freeze on.

**Why not permutation copies.** The per-diagram channel set is *already* closed
under permutations of identical outgoing legs: the image of a diagram under a swap
of two identical legs is another diagram of the same process (`g g → g g`'s `t` and
`u` channels are each other's image). Enumerating the copies would duplicate
channels the set already has — `Π_s n_s!` times the per-point density cost for no
new coverage — and would manufacture exactly-degenerate channels that the
Kleiss–Pittau reallocation cannot separate, which is the failure mode note 27 §B3.2
already registers. The closure claim is *pinned*, not assumed:
`the_channel_set_of_identical_outgoing_legs_is_permutation_closed` (`hadronic.rs`)
asserts the combined density of `g g → g g`'s regulated channel set is invariant
under exchanging the two outgoing momenta, and refuses to report a pass unless
dropping some single channel breaks that invariance.

That control caught a real blind spot on its first run: built *unregulated*
(spacelike floor zero), all four of `g g → g g`'s channels collapse onto a common
all-timelike map whose density is symmetric one channel at a time, so the check
saw nothing. It is therefore stated at the floor a hadronic run gives the
channels — and the collapse itself is another instance of the degenerate-map
finding, now confirmed for `g g → g g` at fixed beams.

**Why not a fundamental domain.** Restricting the map to one ordering of the
identical legs would put the factor in the map, but a [`Channel`] must report the
density it assigns to a *foreign* configuration, and a foreign point need not lie
in the domain — every channel would have to symmetrise its density over the `n_s!`
images. It would also put the cut filter and the event record, both written on
labelled legs, on a different footing from the sampler. Rejected.

**Where the factor lives — a refinement of §4 S1's wording.** "Into the phase-space
map, which knows its own outgoing multiset" is right for a fixed-beam run, where
the map is per-process, and *unsound* for a hadronic one: `ProtonIntegrand`
deliberately pools the channels of every flavour group into a single mixture, so
one map is shared by subprocesses whose outgoing masses agree while their multisets
do not — `p p > j j` has `g g → g g` and `q q̄ → q q̄` both at `[0, 0]`. A map weight
carrying the factor would stop being a density on the one labelled `dΦ_n` those
subprocesses share: the same one-factor-for-many-subprocesses bug, one level lower.
So the factor moved into the phase-space *layer*, but not into the weight:

- `phasespace::identical_particle_factor(outgoing)` is the single definition,
  documented with the `dΦ_n` over-counting it undoes. It takes any species labels
  that compare equal for the same species, which model particle ids and PDG codes
  both do, both separating a particle from its antiparticle.
- Every consumer derives it from the outgoing legs *it* owns, and no consumer holds
  a field another could write to. `BoundSubprocess` carries its own amplitude's
  factor and `FixedBeamIntegrand` applies it inside the sum — the integrand-level
  `symmetry_factor` field is gone, so the `amps[0]` derivation has nowhere to land.
  `proton::Subprocess::symmetry_factor` reads the concrete flavour assignment and
  `FlavorGroup::symmetry_weighted_luminosity` folds each member's own factor into
  the luminosity sum, which is where a sum over subprocesses can still tell members
  apart. `ProtonError::IdenticalFinalState` — the assert-the-factor-is-1 refusal —
  is deleted, so C's `p p > j j` decomposition is no longer refused.

**Consequence for C.** A flavour group's members are not constrained to share an
outgoing multiset by anything in the grouping rule, so the factor is applied per
*member* rather than per group. C needs no further work here beyond exercising it.

## S2 — multi-rung spine design + ordering oracle spec

A design session: no engine. What lands here is the binding shape of the ordered
rung chain, the oracle S3 is judged by, the density contract the chain has to
satisfy, the D3 decision with its measurement, and the D2 card. Two probes were
written because two of the claims are measurements rather than readings —
`spacelike_lines_of_a_diagram_nest_into_an_ordered_rung_chain` and
`probe_fiducial_t_max_against_the_floored_pole_on_llj_cuts`, both in
`vibegraph-lib/tests/diagram_channel.rs` — plus one experimental knob,
`DiagramChannel::with_fiducial_t_max`, which no production caller reaches and which
S3 owns productionising or deleting.

### S2.1 The rung chain, derived from the `Prop` chain

**The reading.** A spacelike line is one through which exactly one beam flows
(`Prop::is_spacelike`, `vibegraph-lib/src/diagrams/diagram.rs:83-85`). Each such
line cuts the externals in two, one side carrying beam `0` and the other beam `1`.
For spacelike line `k` define

```
S_k = the outgoing-leg slots on beam 0's side of that cut
```

read exactly as `spine_partition` (`phasespace/diagram_channel.rs`) reads it today:
the stored `momentum` sign-decorates the externals on one side, so only the nonzero
pattern is used, and the side is complemented when the stored coefficients do not
touch beam `0`. Nothing new is needed to obtain the `S_k`; what is new is the claim
about their *structure*.

**The claim.** For a tree diagram the `S_k` are **totally ordered by strict
inclusion**, `S_1 ⊂ S_2 ⊂ … ⊂ S_r`, and no two are equal. The chain order is then
`|S_k|`-sorted order, the blob emitted at rung `i` is

```
B_i = S_i \ S_{i-1}      (S_0 = ∅),        recoil = full \ S_r
```

and the running momentum transfer is `q_i = p_a − (p_{B_1} + … + p_{B_i})`, i.e.
`q_i = p_a − Σ_{slot ∈ S_i} p_slot`, with `t_i = q_i²`. That is note 21's running
`q_i` made constructive: the `S_k` *are* the prefixes.

**Why it is a claim and not a definition.** Sorting by `|S_k|` agrees with sorting
along the chain only if the sides nest; two incomparable sides would mean the
spacelike lines are not a path, and two equal-sized sides would leave the order
undetermined. Both are pinned by
`spacelike_lines_of_a_diagram_nest_into_an_ordered_rung_chain`, which fails on
either. Measured over `u d > e+ e- u d QCD=0`, `u u~ > e+ e- u u~ QCD=0`,
`u u~ > u u~` and `u u~ > e+ e- g`: 89 diagrams, spacelike-line counts
`{0: 17, 1: 33, 2: 22, 3: 17}`, **zero** violations. The test refuses to pass unless
a three-rung ladder is present, so the nesting property cannot be satisfied
vacuously. The same sweep was run in-session over the full `p p > e+ e- j j QCD=0`
enumeration — 3024 diagrams, counts `{0: 624, 1: 1024, 2: 768, 3: 608}` — and also
found zero violations; that sweep is not committed, since enumerating its 465
subprocess sets on every test run buys nothing the four processes above do not.

A three-rung chain it prints, on `u d > e+ e- u d QCD=0` with slots
`[e+, e-, u, d]`: rung blobs `[[2], [0], [1]]`, recoil `[3]` — the multiperipheral
topology where the `e+` and the `e-` leave the chain at *different* vertices across
a spacelike lepton line. That is the shape a single-rung spine cannot express at
all.

**Cross-check S3 owes.** The `S_k` are read from the stored momentum routing. The
same partition follows from an independent graph cut — remove the propagator, take
connected components — which is the precedent note 21 set for `subsystem_mask`. S3
derives the chain both ways and asserts they agree as sets, so a feyngraph routing
change trips one derivation or the other.

**Degrees of freedom.** With blobs `B_1 … B_r` of sizes `k_1 … k_r` and recoil
`B_{r+1}` of size `k_{r+1}`, the chain consumes

```
2r                                        one t_i and one φ_i per rung
+ #{ i ≤ r : k_i ≥ 2 }                    each composite blob's own invariant s_i
+ #{ i ≤ r : |R_i| ≥ 2 }                  each running remainder's invariant ŝ_i
+ Σ_i (3k_i − 4) over composite blobs     each blob's internal decay tree
```

coordinates, where `R_i = B_{i+1} ∪ … ∪ B_{r+1}`. Checked: `r = 1`, blobs `{0,1}`
and recoil `{2}` gives `2 + 1 + 0 + 2 = 5 = 3·3 − 4`, which is what `sample_spine`
consumes today; `r = 2`, blobs `{2}`, `{0,1}`, recoil `{3}` gives
`4 + 1 + 1 + 2 = 8 = 3·4 − 4`. S3 asserts `ndim() == 3·n_out − 4` for every derived
chain, which is a real check once the count is non-trivial.

**The recursion, and what it reuses.** Write `Q_0 = p_a + p_b` (invariant `ŝ`), and
at rung `i` let the system `Q_{i-1}` (invariant `ŝ_{i-1}`) split into the blob `B_i`
(invariant `s_i`) and the remaining system `Q_i = q_i + p_b` (invariant `ŝ_i`).
Then rung `i` is *exactly* the existing peripheral 2-body step with

```
t_kinematics( ŝ_{i-1}, ma2 = t_{i-1}, mb2 = m_b², s1 = s_i, s2 = ŝ_i )
```

and `t_0 = m_a²`. The single-rung spine is `r = 1`. Note what changes: **for `i > 1`
the incoming line is spacelike, so `ma2 < 0`.** `t_kinematics` is already
algebraically fine there — `kallen(...).max(0).sqrt()` and
`ea = (s + ma2 − mb2)/(2√s)` do not assume a timelike incoming leg — but
`beam_momenta`, which takes *masses*, is not: S3 needs the `m²`-taking variant so a
spacelike incoming line can be built (`e_a` below `|k|`, which is the point).

**A consequence worth having in hand: where the collinear edge actually is.** With a
massless spectator beam (`m_b² = 0`) and a massless emitted blob (`s_i = 0`) the
transfer's upper edge works out to

```
t_max^(i) = t_{i-1} · ŝ_i / ŝ_{i-1}
```

— an exact identity under those two conditions, not an approximation, and **pinned
against the production kinematics** by
`a_spacelike_incoming_line_pushes_the_transfer_edge_off_the_pole`
(`phasespace/diagram_channel.rs`), which also exercises the negative `ma2` an
interior rung supplies. So:

- **rung 1** has `t_0 = 0` and lands on `t_max = 0` — the collinear edge the
  single-rung spine already meets, and D3's subject;
- **interior rungs** are pushed strictly off it by the previous transfer, in
  proportion to `t_{i-1}`;
- the **last rung** is back on the edge whenever the recoil is a single massless leg
  (`ŝ_r = 0`).

The push-off is proportional to `t_{i-1}`, which is itself regulated rather than
large, so this does *not* mean interior rungs need no regulator — it means the
edge-degeneracy is a first-and-last-rung phenomenon and the interior ones are
merely further from it. The same test keeps both routes back to the edge visible, so
the formula is not read as "an interior rung is always safe". What S3 still owes is
the *distribution*: a per-rung `t_min/t_max` dump on the D2 reference process, which
says how small the push-off actually gets on real events.

### S2.2 The types, and what the chain supersedes

```rust
struct SpineRung<F: Real> {
    /// The final-state blob this rung emits: `B_i = S_i \ S_{i-1}`.
    emitted: Node<F>,
    /// The spacelike propagator's mass²; width zero by construction.
    t_mass2: F,
}

struct Spine<F: Real> {
    /// Ordered away from beam 0; `rungs[i]` emits against `q_i`.
    rungs: Vec<SpineRung<F>>,
    recoil: Node<F>,
}
```

`ChannelTopology`, `Node`, `Branch`, `sample_branch` and `branch_jacobian` are
untouched — blobs and the recoil hang off the chain through the existing timelike
machinery exactly as `emitted`/`recoil` do today.

- **Single-rung bit-identity is a hard requirement.** `rungs.len() == 1` must
  reproduce today's `Spine { emitted, recoil, t_mass2 }` bit for bit on sampled
  momenta, walk weights and densities, in the shape of
  `a_zero_spacelike_floor_leaves_every_channel_bit_identical`. Every enforced σ row
  that already runs peripheral channels depends on it.
- **`t_channels()` stops being kinematic metadata.** It is a `props`-order list, so
  its order carries no meaning; the kinematic driver becomes `rungs[i].t_mass2` in
  chain order, and `spine_pole() -> Option<F>` is superseded by
  `spine_poles() -> &[F]`. Keep the accessor for the diagrams that still fall back
  to the all-timelike tree, and say in its doc that it is unordered.
- **Anchor convention.** Beam `0`, matching `spine_partition`. Anchoring at beam `1`
  reads the same ladder from the other end: it is a *different map* (a different
  blob becomes the recoil), not a relabelling, and §S2.3's second control pins that.
- **The multiset `{t_i}` is anchor-independent; the chain order is not.** `t_i` is
  the square of a propagator momentum, the same invariant computed from either side
  by momentum conservation. This is exactly why no invariant-level check —
  `Vₙ`, σ, or a per-`t_i` histogram of the *volume* — can see a wrong ordering, and
  why §S2.3 has to be a coverage test rather than an agreement test.
- **The rotation is the new bug site.** Rung `i > 1` is built in the CM of `Q_{i-1}`
  with `q_{i-1}` along `+z`; the existing single-rung code needs no rotation because
  beam `0` is already along `+z` there. A chain does. A wrong rotation leaves the
  drawn `t_i` and the `t_i` reconstructed from the final momenta disagreeing, which
  is what `assert_valid`'s walk-weight-vs-`1/density` comparison
  (`WALK_DENSITY_TOL = 1e-7`) already measures — it caught the unregulated spine at
  `4e4` on the same measure. S3 additionally dumps the per-rung pair.

### S2.3 The ordering firing test, and its negative control

Note 21 deferred the multi-rung spine because `Vₙ` and σ are blind to a
wrong-but-valid rung ordering. The test below is not blind to it, and the reason is
stated before the mechanism: **a wrong ordering is not a wrong number, it is a wrong
importance sampling.** Both orderings integrate `dΦ` correctly; only the right one
concentrates its draws where the diagram's propagators peak. So the test is a
coverage test on a peaked integrand, not an agreement test on a volume.

Concretely, on the D2 reference process at fixed `√ŝ` with its run-card cuts, for a
chain of `r ≥ 2`:

**Probe integrand.** `f(p) = [Π_{i=1..r} 1/(m_i² − t_i)²] · BW(s_pair)`, with the
`t_i` computed **in the test's own words** from the diagram's chain (the
`S_i`-prefix definition of §S2.1), never asked of the channel. `BW` is the lepton
pair's Z line shape. The cut indicator multiplies it, which is what makes the
massless rungs integrable.

**T-ORD-1 — per-rung coverage.** Draw `N = 400 000` points from the channel alone.
For each rung `i`, bin the *raw* draws by `ln|t_i|` into 12 bins spanning
`[|t|_cut, |t|_max]`. Every bin holds at least 500 draws. A rung that is not being
importance-sampled starves its small-`|t_i|` bins.

**T-ORD-2 — per-bin precision.** Over the same bins, the estimator of `∫f`
restricted to each bin has relative error ≤ 10% at `N`.

**T-ORD-3 — seed stability.** Five independent seeds; χ²/dof of the total `∫f` about
its inverse-variance mean below 4, worst single-seed pull below 5. This is the guard
a scalar cannot be: a map that misses a region reports a small integral *and* a
small variance.

**T-ORD-4 — volume neutrality (stated as ordering-blind).** The channel reproduces
`V_n` against flat RAMBO within MC error. It checks the Jacobian, not the ordering,
and is recorded here so nobody later mistakes it for confirmation.

**NEG-A — the swapped chain (the load-bearing control).** Build the *same* channel
with `rungs` reversed and everything else identical (S3 exposes a test-only
`with_rung_order`), and assert that **at least one of T-ORD-1..3 fails for it**,
printing the measured margin. `assert!(swapped_fails, "the ordering test cannot
fire")`. Without this assertion the whole group is the vacuous-check failure. Note
what a swap of a 2-rung chain actually changes: `t_2 = (p_a − p_{B_1} − p_{B_2})²`
is *unchanged* (it is `p_a` minus every emitted blob), and only `t_1` moves, from
`(p_a − p_{B_1})²` to `(p_a − p_{B_2})²` — which is not a propagator of the diagram
at all. So the firing is expected in rung 1's projection specifically, and S3 should
assert it there rather than anywhere.

**NEG-B — the anchor flip.** Build the chain anchored at beam `1` and assert its
density differs from the beam-`0` chain's by more than rounding on a majority of
points drawn from the latter. Cheap, deterministic, and it pins that the two
conventions are genuinely different maps.

**NEG-C — the maps are distinguishable at all (the precondition).** Generate points
from the correctly-ordered channel and evaluate the swapped channel's `density` on
them; require a relative gap above `1e-6` on more than half. If the two densities
coincided, the ordering question would be moot *and* T-ORD would have no content —
and that is not hypothetical: S1 found all four `g g > g g` channels collapsing onto
a common map at spacelike floor zero (note 27 §B3.2's finding made sharp). NEG-C is
the check that the same collapse has not silently happened here. It must run at the
floor a real run gives the channels, not at zero, for exactly the reason S1's
permutation-closure control had to.

**What this test provably cannot detect.**

- **A symmetric chain.** If two rungs carry the same pole mass *and* kinematically
  interchangeable blobs, the swapped map is the same map and the test has no
  content. NEG-C is what refuses to report a pass in that case; D2's process is
  chosen so it does not arise (blobs are a jet, a lepton and a lepton pair; poles
  mix massless and `m_Z`).
- **Anything invisible to a real positive density** — a global phase, an amplitude
  sign, a colour-flow index. The map is a density; it sees none of that.
- **An error common to both orderings** — a wrong per-rung Jacobian factor, a wrong
  anchor beam applied to every rung, a misread blob content. Those are volume and
  reciprocity errors; T-ORD-4 and the walk-vs-density comparison own them, and
  T-ORD would pass with them present.
- **A wrong ordering whose coverage happens to survive.** The test fires on
  starvation, so at a `√ŝ` where all the `t_i` windows overlap heavily a wrong
  ordering could stay adequate. The test is therefore specified at a *stated* `√ŝ`
  and cut configuration, and NEG-A's measured margin is printed and asserted rather
  than assumed to be large.
- **Whether the chain belongs to *this* diagram.** T-ORD only sees that the sampled
  invariants are the ones the probe peaks in. That the chain comes from this
  diagram's `Prop` chain is §S2.1's structural test and the graph-cut cross-check,
  not this one.

### S2.4 The foreign-config density contract (L4), extended to rung chains

The L4 contract is that a [`Channel`] reports the density it assigns to an
*arbitrary* on-shell, momentum-conserving configuration, not only to points it
generated, because a combiner weights every point by `αⱼ / Σₖ αₖ gₖ` gathered from
every channel at the *same* configuration
(`phasespace/channel.rs`, `Channel::density`). Note 21 discharged it for one rung by
recomputing `t` as the frame-independent `(beams[0] − p_emitted)²`. For a chain:

- **(C1) Reciprocity.** At any point the channel itself generated, `density` is the
  exact reciprocal of the walk-accumulated weight, to rounding, for every rung
  count. The two are separate computations — the walk multiplies the invariants it
  drew, `density` rebuilds them from the momenta — so their agreement is a real
  check. `WALK_DENSITY_TOL = 1e-7` is the existing bound; an unregulated single-rung
  spine reaches `4e4` on the same measure.
- **(C2) Domain totality — the sharpest new statement.** The density must be
  defined, strictly positive and finite at **every** on-shell momentum-conserving
  configuration at the channel's `√ŝ`, *including one that does not look like this
  chain's ordering*. A configuration whose `t_2` is smaller than its `t_1`, or whose
  blobs are nowhere near the chain's own peaks, is a perfectly ordinary point that
  another channel drew. **The rung order is a property of the map, not a constraint
  on the configuration**, and a chain that refused such a point — or returned zero,
  or `NaN` — would bias every other channel's estimate through the shared `Σₖ αₖ gₖ`.
  This is the failure mode a chain makes newly available, and S3 pins it by
  evaluating every chain's density at points drawn from every *other* channel of the
  D2 process and requiring finite positivity throughout.
- **(C3) Frame independence.** Every quantity the density reads is an invariant
  built from the configuration and the stored beams: `t_i = (p_a − Σ_{S_i} p)²`, the
  blob invariants `s_i`, the running remainder invariants `ŝ_i`. Nothing is read
  from a frame the channel assumed, so no rotation or boost the sampler performed
  can leak into the density.
- **(C4) Window totality.** The per-rung `[t_min, t_max]` and the per-invariant
  `[lo, hi]` windows are the *kinematic* limits of the chain's own decomposition, so
  any physical configuration satisfies them and no genuine zero-density region
  arises. The only way to create one is to restrict the map deliberately — which is
  precisely what D3's fiducial bound does, hence (C5).
- **(C5) Support honesty.** A channel that deliberately narrows its support must
  report density **exactly zero** outside it, never a positive number: the multichannel
  estimator is unbiased only when each `gⱼ` is the true pushforward density of
  channel `j` everywhere. And the channel *set* must then still cover, between its
  members, everywhere the integrand is non-zero. Both halves are measured in §S2.5,
  not assumed. In the experimental knob this is implemented as `spine_jacobian`
  returning `+∞` outside the restricted window, so `density = 1/jac` is `0`.
- **(C6) Degenerate configurations return zero, not `NaN`.** A lightlike `q_i`, a
  blob at threshold, a vanishing `k`: the existing `peripheral_factor` guard
  (`k > 0 && √s > 0` else zero) is the precedent and generalises per rung.

### S2.5 D3 — the massless-t-channel cut, decided by measurement

**The question.** With a massless beam and a massless emitted blob, rung 1's upper
edge sits analytically on the pole (`t_max = m² = 0`) and is computed as a cancelling
difference, so the propagator map either switches on over a window reaching
`|t| ~ 1e-11` or falls back flat, on rounding noise
(`a_massless_spacelike_pole_puts_the_transfer_edge_on_rounding_noise`). Production
avoids that by **flooring the pole** at the scale the cuts imply,
`t_mass² ← max(m², Cuts::spacelike_floor())` — the largest single-leg `pT_min`
squared (`cuts.rs`), 400 GeV² on the banked llj card. The alternative note 21 left
open is to keep the bare pole and **bound the window** instead, `t_max ← −pT_min²`.

**What was measured.** `probe_fiducial_t_max_against_the_floored_pole_on_llj_cuts`,
on the six single-spacelike-line cuts of `u u~ > e+ e- g` and `g u > e+ e- u` at
`√ŝ = 500` with the default run card's cuts (`ptj 20 → 400 GeV²`), integrand
`cut_pass · 1/[((s_ll − m_Z²)² + (m_Z Γ_Z)²) · t²]` — the spacelike propagator left
**massless**, which is the singular thing the question is about. Five seeds,
200 000 points each, per-point estimator variance reported. Three maps: the
all-timelike channel the derivation builds when no floor is supplied (which is what
"the map falls back flat" means concretely past two outgoing legs — no spine is
built at all), the floored pole, and the bounded window. The bounded arm keeps a
token pole at `400/1000 = 0.4 GeV²`, three orders below the bound and seven above
the cancellation noise, so a configuration whose window the bound cannot narrow
still gets a well-posed map instead of the unregulated one.

| cut | var(all-timelike) / var(floored) | var(floored) / var(bounded) | cut efficiency floored → bounded |
|---|---|---|---|
| `u u~` 0 | 38.11× | 1.665× | 0.3843 → 0.4197 (1.09×) |
| `u u~` 1 | 39.07× | 1.833× | 0.7610 → 0.8381 (1.10×) |
| `u u~` 2 | 26.89× | 1.689× | 0.3880 → 0.4238 (1.09×) |
| `u u~` 3 | 41.74× | 1.834× | 0.7617 → 0.8378 (1.10×) |
| `g u` 0 | 38.11× | 1.665× | 0.3843 → 0.4197 (1.09×) |
| `g u` 1 | 39.07× | 1.833× | 0.7610 → 0.8381 (1.10×) |

(The `g u` rows reproduce the `u u~` ones exactly: the spine is built from masks,
masses and the pair's pole, which coincide between the two subprocesses for those
cuts. Four distinct configurations, not six.)

Integrals agree across all three arms within combined error on every cut — e.g.
`u u~` cut 1: all-timelike `2.324e-10 ± 3.7e-12`, floored `2.4074e-10 ± 5.9e-13`,
bounded `2.4048e-10 ± 4.4e-13` — so nothing is being bought with a bias.

**Union coverage, and the control that makes it mean something.** A channel set
whose members each renounce part of phase space is unbiased only if between them
they still reach everywhere the integrand lives. Measured on the sharpest available
integrand — the cut indicator itself — over a combiner built from **only** bounded
spines, against flat RAMBO (400 000 points):

| bound | as a fraction of `ŝ` | bounded-spine combiner | flat RAMBO | pull |
|---|---|---|---|---|
| 400 GeV² (the cut scale) | 0.002 | 2.843149e5 | 2.827958e5 | 0.9σ |
| 4 000 | 0.016 | 2.831375e5 | " | 0.2σ |
| 40 000 | 0.160 | 2.756865e5 | " | 4.6σ |
| 100 000 | 0.400 | 2.348088e5 | " | 34.1σ |
| 150 000 | 0.600 | 1.899247e5 | " | 49.9σ |
| 200 000 | 0.800 | 2.011633e5 | " | 33.9σ |

The check fires, monotonically and in the right direction (a bound that renounces
surviving phase space *under*-estimates), first breaking at 100× the cut scale. So
the coverage pass at 400 GeV² is not vacuous, and the bound the design would install
carries an order of magnitude of margin.

**A side finding worth recording.** `Cuts::spacelike_floor() = pT_min²` is a
*provable* bound but a very loose one: on this configuration the surviving region
actually sits above `|t| ≈ 4 000–40 000 GeV²`, ten to a hundred times higher.
Central leptons (`etal 2.5`) force the jet to recoil at a large angle, which the
per-leg `pT` bound knows nothing about. A tighter fiducial bound would buy more than
the 1.7× measured here — but deriving one from general cuts is real work and is
**not** in this sprint; the conservative bound is what §S2.5's decision installs.

**Decision (D3).** **Bound `t_max` at the fiducial scale where one exists; keep a
pole floor as the fallback where none does.** Concretely, for a rung whose window
reaches the collinear edge:

- when `Cuts::spacelike_floor() > 0`, set that rung's `t_max ← −floor` and leave the
  pole at `max(m², ε·floor)` with `ε` small (`1e-3` measured), so the draw is the
  bare `1/|t|` over the fiducial region and a configuration the bound cannot narrow
  still has a well-posed map;
- when it is `0` — a partonic run with no active `pT` cut, and every fixed-beam
  `2 → 2` row — **nothing changes**: there is no fiducial scale to bound with, so
  the flat fallback stands and the existing bit-for-bit behaviour is preserved. D3's
  answer is scoped to cut-regulated processes and does not touch any enforced
  partonic σ row.

Rationale in one line: 1.67×–1.83× variance at equal cost, no measurable bias, and a
10× coverage margin — against the standing hazard that a narrowed support is a new
way to be wrong, which (C5) and the ladder above are what keep visible.

**What S3 owns.** Productionising this means threading the per-rung bound from
`Cuts` into `DiagramChannel` construction (`proton.rs` and `hadronic.rs` are the two
call sites of `from_diagram_regulated`) and generalising it per rung — the bound
applies to whichever rungs reach the edge, which by §S2.1 is rung 1 and a last rung
with a single massless recoil leg. `with_fiducial_t_max` is a whole-channel knob and
is *not* the production shape; it exists to have made this measurement.

### S2.6 D2 — the spine reference process and its card

**Deviation from the approved candidate, with the numbers.** §6 approved
`p p > e+ e- j j` at QCD=0. Enumerated here, that process is **112 non-empty
subprocesses and 3024 diagrams**, which `derive_flavor_groups` turns into **60
flavour groups pooling 1608 sampling channels**. A multichannel evaluates every
channel's density at every point, so that is some 67× the per-point cost of the
`pp_to_llj_fixed` row (24 channels) on a four-body final state needing more points —
and MadGraph would have to integrate and store all 112 subprocesses, where
`pp_to_llj_fixed` alone is 151 MB and would ride into `refdata-4` at Z.

The card written instead is the **one concrete flavour assignment**,
`u d > e+ e- u d QCD=0` at fixed partonic beams —
`validation/madgraph/scripts/ud_to_epemud_qcd0.mg5`. Measured: **1 subprocess, 35
diagrams, 35 channels**, splitting `12 / 14 / 9` over one, two and three spacelike
lines. It carries the whole ladder spectrum the design has to handle — including the
three-rung multiperipheral topology — at a channel count comparable to the llj row,
and its rungs are asymmetric (blobs are a jet, a lepton and a lepton pair; poles mix
massless photon/quark/lepton lines with `m_Z`), which is the precondition §S2.3's
NEG-C insists on. `QCD=0` removes the strong coupling outright and `lpp = 0` removes
both PDFs, so Track S cannot wait on Track K by construction rather than by run-card
setting. The flavour union and the `(τ, y)` convolution are already gated by the
Drell–Yan and llj rows, so fixing the initial state costs the sprint no coverage.

The card follows the `uux_to_epemg` partonic precedent plus `pp_to_llj_fixed`'s
fixed-scale lines: `lpp1 = lpp2 = 0`, `ebeam 250` each (`√ŝ = 500`, the energy the
D3 measurement was made at), all three fixed-scale switches on at `m_Z`, `mmll = 50`
for the reason `pp_to_llj_fixed` gives, `use_syst False` as every `lpp = 0` run, and
the default `ptj 20 / ptl 10 / etaj 5 / etal 2.5 / drll = drjl = 0.4` left alone.
`ptj 20` is load-bearing twice: it regulates the t-channel singularity that would
otherwise dominate `σ̂`, and it is the scale §S2.5's bound uses.

**If the manager wants the proton path exercised**, the middle option is
`p p > e+ e- u u~ QCD=0`: 4 non-empty subprocesses, 131 diagrams, 4 groups, 131
channels, rung counts `{0: 64, 1: 44, 2: 14, 3: 9}`. Five and a half times the llj
row rather than sixty-seven, and still a real `p p` run — at the cost of half its
diagrams having no spacelike line at all. It is not what the card proposes, but it
is the one variant worth a second thought before Sb runs.

### S2.7 For S3 and the sprint manager

1. **The chain hypothesis is pinned, not assumed** —
   `spacelike_lines_of_a_diagram_nest_into_an_ordered_rung_chain`, zero violations
   over 89 surveyed diagrams and over the full 3024-diagram `p p > e+ e- j j QCD=0`
   set, with a non-vacuity guard requiring a three-rung ladder to be present.
2. **Ladders reach three rungs, not two.** `u d > e+ e- u d QCD=0` gives `12/14/9`
   over one/two/three spacelike lines. A design that only handles `r = 2` covers a
   quarter of that process's diagrams and none of its multiperipheral ones.
3. **The interior rungs' incoming line is spacelike.** `t_kinematics` already
   handles `ma2 < 0`; `beam_momenta` does not, and needs an `m²`-taking twin.
4. **The rotation between rungs is the new bug site**, and the existing
   walk-weight-vs-`1/density` comparison is already the instrument that sees it.
5. **D3 is decided and scoped**: bound `t_max` where a fiducial scale exists, change
   nothing where it does not, so no enforced partonic row moves.
6. **`DiagramChannel::with_fiducial_t_max` is experimental and unwired.** No caller
   in the crate reaches it. S3 either productionises the per-rung form or deletes
   it; leaving it as a whole-channel knob is not an option, because a chain needs
   the bound per rung.
7. **Follow-up worth filing, not for this sprint:** a tighter fiducial bound on
   `|t|` derived from the full cut set rather than from `pT_min²` alone. The measured
   gap is one to two orders of magnitude, and the variance win scales with it.

## S4 — spine in production: measurements

The switch was two lines, not one. `from_diagram_regulated` now admits every rung
(the ladder cap moves to `from_diagram_capped`, which nothing in production calls
and which exists so the chain can be measured against the map it replaced), and
`FixedBeamIntegrand::use_multichannel` now passes its own `Cuts::spacelike_floor()`
the way `ProtonIntegrand::new` always has. The second line is what makes the first
reach anything: **no production process today is both a ladder and regulated
without it.** `pp_to_llj_fixed`'s diagrams carry one spacelike line each, every
fixed-beam `2 → 2` carries one propagator, and the fixed-beam path was supplying
floor `0` — under which a final state of more than two legs gets no spine at all.
The ladder switch alone would have moved nothing.

**Deviation from D3's scoping.** §S2.5 recorded that D3 "does not touch any enforced
partonic σ row", on the reading that a fixed-beam row has no fiducial scale. Six of
them do: `uux_to_uux` and `gg_to_gg` carry `ptj 20` (floor 400 GeV²), and `ee_to_ee`,
`ee_to_mumua`, `ee_to_mumu_tata_qcd0` and `ud_to_epemud_qcd0` carry `ptl 10`
(floor 100 GeV²). Their maps moved, every one of them for the better. Two disjoint
controls stay bit-for-bit unchanged and say the change is the floor acting on a
spacelike line and nothing else: rows whose cuts imply no floor even though their
diagrams are peripheral (`gg_to_ttx`, floor 0 with 2 peripheral channels;
`ee_to_wpwm`, floor 0 with 1), and rows with a floor but no spacelike line at all
(`ee_to_mumu`, `uux_to_mumu`, `ee_to_tatah`, all floor 100 with 0).

### Coverage, per process switched on

`every_bounded_channel_set_covers_its_own_fiducial_region` (banked gate, 100 000
flat-RAMBO draws per row, the cut indicator as the integrand). Every accepted point
must be reachable by some channel's density; the bound narrows support, so this is
the check that the narrowing renounced nothing.

| row | floor | peripheral/total channels | accepted | reachable | only a bounded channel reaches | at 100× the bound |
|---|---|---|---|---|---|---|
| `ee_to_ee` | 100 | 2/4 | 98 642 | 98 642 | 0 | 98 642 |
| `ee_to_mumua` | 100 | 4/8 | 92 247 | 92 247 | 0 | 92 247 |
| `ee_to_mumu_tata_qcd0` | 100 | 8/25 | 83 142 | 83 142 | 0 | 83 142 |
| `gg_to_gg` | 400 | 2/4 | 99 652 | 99 652 | 0 | 99 652 |
| `uux_to_uux` | 400 | 1/2 | 99 652 | 99 652 | 0 | 99 652 |
| `ud_to_epemud_qcd0` | 400 | 35/35 | 75 360 | 75 360 | **75 360** | 75 190 |

Five of the six keep an unbounded member — an s-channel or contact diagram whose
tree spans the whole final state — so their coverage is not a constraint and the
table says so rather than claiming a pass. `ud_to_epemud_qcd0` is the one that
constrains: all 35 channels are bounded chains, coverage is exact, and pushing every
bound out by 100× loses 170 accepted points, which is the control that says the
check can see where the bound sits.

### B1 — the `uux_to_uux` five-seed mean

`probe_qcd_seed_stability`, five seeds × two budgets, unchanged budgets.

| row | before: worst \|pull\| / worst \|rel\| / 5-seed mean rel | after |
|---|---|---|
| `uux_to_uux` 1× | 2.69 / 6.4e-3 / **−0.30%** | 0.93 / 1.1e-3 / **+0.019%** |
| `uux_to_uux` 4× | — / 3.5e-3 / −0.25% | 0.54 / 5.1e-4 / +0.015% |
| `gg_to_gg` 1× | 1.63 / 4.9e-3 | 1.24 / 1.4e-3 / +0.077% |
| `gg_to_gg` 4× | — / 2.7e-3 | 1.19 / 1.0e-3 / +0.041% |
| `gg_to_ttx` | 0.68 / 8.6e-4 | bit-for-bit identical (floor 0) |

**Verdict: the bias is gone.** The standing −0.30% was the spacelike collinear
region, and it was under-resolved not because a rung was missing but because the
transfer was drawn *flat* — a massless line at the collinear edge cannot shape its
own draw, so every peripheral fixed-beam channel was an isotropic 2-body split.
Floor and bound together turn it into a `1/|t|` draw over the fiducial window. The
quoted error at the gate budget fell 2.4× on `uux_to_uux` (5.13e1 → 2.17e1 pb) and
2.6× on `gg_to_gg` (3.66e2 → 1.40e2 pb) — 5.6× and 6.8× in variance at equal cost.

### B2 — the degenerate-map finding

`probe_channel_map_degeneracy`, worst *pairwise* relative density difference over
2 000 drawn points beside the converged α.

| row | worst pairwise density difference | bit-identical pairs | converged α |
|---|---|---|---|
| `uux_to_uux` | **1.000** (was 0, bit-identical) | 0 / 2 000 | `[8.5e-6, 0.99999]` |
| `gg_to_gg` | **1.000** (was 0) | 2 000 / 12 000 | `[3.2e-5, 3.2e-5, 0.496, 0.504]` |
| `gg_to_ttx` (control) | 0.8385 | 0 / 6 000 | `[0.267, 0.364, 0.369]` |

**Verdict: the maps differentiate and α moves.** `uux_to_uux`'s t-channel channel
takes essentially the whole selection weight; `gg_to_gg` splits between its two
peripheral channels and starves the s-channel and four-gluon ones. The control
reproduces note 27 §B3.2's recorded numbers (0.84, `[0.267, 0.364, 0.369]`)
digit for digit, so the instrument is the same one that recorded the finding. One
degeneracy survives and is expected: `gg_to_gg`'s two *non*-peripheral channels
remain bit-identical to each other (2 000 of 12 000 pairs = one pair per point),
since neither has a spacelike line for the floor to act on.

This unblocks the per-flow α item the performance backlog parked on it.

### B3 — the spine reference σ, and what it found instead

σ(`u d > e+ e- u d QCD=0`) = **1.0860e-1 pb** at the gate seed (120 000 × 8,
χ²/dof 1.22), 1.0816e-1 at a second seed and 1.0821e-1 as a plain multichannel
average with no VEGAS at all — stable to 0.4% — against MadGraph's banked
**1.4107e-2 ± 3.4241e-5 pb**. A factor 7.7.

It is not the map. Registered temporarily into the f2py amplitude registry and
compared against MadGraph's own `MATRIX1` on the 20-point fixed grid
`gen_amplitude.py` writes for it, this side's colour- and helicity-summed |M|²
disagrees **point by point by factors of 2 to 63**, ours the larger nearly
everywhere. The same comparison, same code path, reproduces `uux_to_uux` to 5.7e-14
over 75 points and `ee_to_mumu_tata_qcd0` to 5.1e-13 over 50, so it is this process
and not the method. Everything countable already agrees with MadGraph: 35 diagrams
to 35, `NCOLOR = 2` with `CF = [[9,3],[3,9]]` against `leshouche.inc`, 8 surviving
helicity combinations to `NCOMB = 8`, the same external ordering, the same compiled
cuts, spin/colour average 1/36. No recontraction of our two JAMPs reproduces
MadGraph's number either, so the colour algebra is not the lever — the coherent
amplitude is wrong.

What is new about this process: it is **the first row here whose diagrams put a `W`
between two quark lines** (`u → d W⁺`, `d → u W⁻`, closed by `W⁺W⁻ → γ*/Z* → e⁺e⁻`),
which is what makes its second colour flow physical at all. That the ratio is
region-dependent and almost always > 1 is what a missed cancellation between
diagrams looks like, not a normalisation.

**So the row lands informational, not gated** — `Plan::Info` with the reason, and
the manifest `integrals` cell `banked` / `info`. Enforcing a σ whose linear level is
known to be wrong is the thing the Physics Validation section exists to forbid. The
phase-space side of the row is exercised regardless: all 35 channels are peripheral
chains, so the number above is drawn entirely through the ordered rung chain, and
the chain's coverage of the fiducial region is gated separately (table above).

The registry entry was reverted rather than committed, because the `amplitude_oracle`
would then fail — and it fails first on a *structural* pre-check worth recording:
MadGraph groups this process's 35 diagrams into **21 AMP2 accumulators**
(`N_MAX_CG = 21`), where this side has 35 singleton configurations. Teaching the
oracle that grouping is the first step of the per-diagram comparison that would
localise the bug.

### B4 — the D3 delta on `pp_to_llj_fixed`

`probe_fiducial_bound_on_llj_fixed`, the same three seeds and the same
300 000 × 10 budget on both arms, the only difference being whether the peripheral
channels keep their transfer bound.

| arm | σ (pb) | χ²/dof | rel vs MG | pull | per-seed errors |
|---|---|---|---|---|---|
| bound on (production) | 423.3142 ± 0.2313 | 0.10 | −0.12% | −0.34 | 0.399 / 0.401 / 0.402 |
| bound off (pole floor only) | 422.5653 ± 0.2642 | 0.14 | −0.30% | −0.83 | 0.462 / 0.449 / 0.462 |

The bound buys **1.25×–1.34× in variance** per seed (1.30× on the mean) at equal
cost — less than the 1.67×–1.83× §S2.5 measured on isolated cuts, which is what one
expects once VEGAS and 23 other channels are between the map and the answer. Both
arms sit inside `LLJ_MAX_REL`, both agree with MadGraph, and the bounded arm is the
closer of the two: no bias is bought with the variance.

### C — every enforced σ row, before and after

Fixed seed, unchanged budgets. Rows not listed are bit-for-bit unchanged.

| row | before (σ, pull, rel, χ²/dof) | after |
|---|---|---|
| `uux_to_uux` | 2.818429e4 ± 5.129e1, −1.49, −3.00e-3, 1.76 | 2.825463e4 ± 2.172e1, −0.44, −5.08e-4, 0.76 |
| `gg_to_gg` | 1.427908e5 ± 3.660e2, +0.05, +1.45e-4, 1.16 | 1.427420e5 ± 1.400e2, −0.16, −1.96e-4, 0.56 |
| `ee_to_ee` | 1.556023e2 ± 9.323e-2, −0.83, −7.56e-4, 1.55 | 1.556415e2 ± 9.439e-2, −0.55, −5.04e-4, 1.64 |
| `ee_to_mumu_tata_qcd0` | 1.367003e-3 ± 2.685e-6, −1.45, −4.01e-3, 0.99 | 1.372287e-3 ± 2.078e-6, −0.06, −1.55e-4, 1.18 |
| `ee_to_mumua` | 1.007660e-1 ± 2.022e-4, +3.12, +9.67e-3, 0.97 | 1.006000e-1 ± 1.665e-4, +2.79, +8.01e-3, 0.72 |

Every moved row moved toward MadGraph, and four of the five shrank their own error.
`ee_to_mumua` stays the widest row of the set (+0.80%, the standing 3.7.1 drift
recorded in TODO.md) but is now 2.79σ rather than 3.12σ from it.

### For the sprint manager

1. **The fixed-beam path was never regulated.** That is the finding behind B1 and
   B2, and it means every measurement of "what the spine is worth" taken through
   `FixedBeamIntegrand` before this session was taken on flat transfer draws.
2. **`ud_to_epemud_qcd0`'s matrix element does not agree with MadGraph.** The spine
   reference row cannot gate its σ until that is reconciled; the phase-space work it
   was banked for is unaffected and is exercised by it regardless. This wants a
   session of its own: register the amplitude table, teach the oracle MadGraph's 21
   AMP2 accumulators, and read the disagreement per diagram.
3. **The work area's inventory debt is now load-bearing.** With the row's σ in
   `sigma_reference.json`, `sigma_gate_matches_madgraph` requires
   `output/ud_to_epemud_qcd0` present, and `validate_scales`/`validate_alphas`
   assert a run inventory that Sb's four new directories break. Registering those
   four is Track K's task and this session did not touch it, so the local banked
   layer is green except for those two inventory assertions.
4. **`with_fiducial_t_max` still exists** and now has a committed caller: the
   coverage gate's 100× control. `without_transfer_bound` and
   `ProtonIntegrand::new_unbounded` are B4's, and `from_diagram_capped` is the
   informational arm's.
5. The artifact schema is at **version 6**: `ChannelSampler::spine_poles_gev2`, one
   entry per rung in chain order, with a version-5 reader that upgrades the old
   scalar to a one-entry chain (which is what it always was — a version-5 writer
   left every ladder all-timelike).
## K3 — engine vs oracle: results and consumed state

The engine is `vibegraph-lib/src/coupling/cluster/`: `graph.rs` (the merge graph
a directory's channel forests imply), `kt.rs` (`cluster.f`), `setclscales.rs`
(`reweight.f`'s walk and the two scale formulas). The comparison against K2's
dumps is `vibegraph-lib/tests/validate_kt_cluster.rs`, gated on
`extended-validation` and skipped when the work area has no dumps.

### K3.1 Result

**All 90 000 dumped events reproduce, merge sequence and scales.** The gate is
a relative agreement of `1e-12` — four orders above the last-ulp spread of
`pow`, `cosh` and `log` between two libms, and four below anything a wrong
branch could produce. The *observed* worst difference over every compared
quantity of every event is `0.0`: both sides evaluate the same expressions on
the same inputs in the same order, so on this platform they agree to the bit.
That is reported, not required — the assertion stays at `1e-12`, since a system
libm may legitimately differ in the last place.

| run | events | sequences | scales | worst dj | worst μ | ambiguous dirs | carried flags |
|---|---|---|---|---|---|---|---|
| `bbx_to_ccx_emmm_qcd0` | 10000 | 10000 | 10000 | 0 | 0 | 0 | 81 |
| `ee_to_mumu_tata_qcd0` | 10000 | 10000 | 10000 | 0 | 0 | 0 | 0 |
| `ee_to_mumua` | 10000 | 10000 | 10000 | 0 | 0 | 0 | 0 |
| `ee_to_ttx` | 10000 | 10000 | 10000 | 0 | 0 | 0 | 0 |
| `pp_to_bb_qcd2` | 10000 | 10000 | 10000 | 0 | 0 | 3 | 0 |
| `pp_to_llj` | 10000 | 10000 | 10000 | 0 | 0 | 8 | 0 |
| `pp_to_llj_qcd2_qed2` | 10000 | 10000 | 10000 | 0 | 0 | 8 | 0 |
| `uux_to_ccx_emmm_qcd0` | 10000 | 10000 | 10000 | 0 | 0 | 0 | 163 |
| `uux_to_uux` | 10000 | 10000 | 10000 | 0 | 0 | 0 | 0 |

2 402 444 candidate pairs and 120 whole merge tables were compared. The
comparison is finer than the scale in seven separate places, each of which
found a real bug during the session: every candidate pair's admissibility, the
arm of the measure it took, its raw and inflated values, the number of channels
`findmt` left alive and `zclus`; every merge's participating leg sets, kind,
scale, `mt2ij`, `zcl` and written leg numbers; the winner of every pass; every
frame change's fired flag, `nleft` and boost vector; the surviving channel list
either side of the point the integration channel claims it; the on-shell
resonance list; `jfirst`/`jlast`/`jcentral` raw and final; every line's PDG,
provenance and jet flag; the jet tags and jet code; which of the two scale
rewrites fired; every vertex scale after them; and the branch index of both
scale formulas. **Every attempt** is compared, not only the accepted one — the
1873 `pp_to_llj` events that re-cluster have both of their clusterings checked.

Non-vacuity is asserted, not assumed: the test reproduces the manifest's own
per-run `coverage` counts branch for branch (`candidate_measure`, `boost`,
`memo`, `mur_branch`, `muf_branch`, `beam_crossing_inflation`,
`cluster_calls_per_event`, `igraphs1_is_iconfig`, `mt2last_override`,
`jcentral_override_beam*`), and fails if the engine ever takes a branch the
reference never took.

### K3.2 Declared consumed state — K4's replay-scope answer

Per event, the engine is *given*:

1. **`iconfig`** (the integration channel) and **`iproc`** (the subprocess).
   Both are live, not inert: §K1.11's finding 2 is confirmed three times over.
2. **The momenta as `cluster()` received them**, from the dump's `MOM` records
   rather than the LHE's ten digits.
3. **`njetstore(iconfig)` at event entry.** The memo *logic* is derived — the
   1873 `pp_to_llj` restricted re-clusters are the engine's own decision, and
   they match one for one — but the stored count itself is a per-directory
   history the dump's write-order cannot reconstruct.
4. **The channel forests** (`iforest`/`sprop`/`tprid`/`prmass`/`prwidth`), from
   the dump's `IFOR` records. Everything downstream of them is derived: the leg
   sets and their complements, the PDG on each line, the resonance map, the
   coupling-order filter, and the Breit-Wigner tagging. The derived tables are
   compared whole against the reference's own on the seven runs whose dump holds
   a single process directory (120 tables, all equal).
5. **The run-card constants**, from the `CONST` record.

Two further consumptions are small, counted, and named because they are not
functions of the event:

6. **Carried-over on-shell flags — 244 events of 90 000 (0.27 %).** See K3.3.
7. **Process-directory identity — 7 flavour assignments across 3 runs.** See
   K3.4.

Nothing else is read from the dump before the engine runs. In particular the
merge graph, the resonance tagging, every measure, the tie-break, the merge
order, the frame changes, the beam walk and both scale formulas are derived.

### K3.3 Diagnosed exception: `isbw` is stale across events

`cluster.f`'s on-shell flag array `isbw` lives in a common block. `checkbw`
(`cluster.f:414`) clears it only for the leg sets of the *integration channel's*
own timelike lines, `i = -1 … -(nexternal-3)`. Every other leg set keeps
whatever a previous event left there. Because one MadEvent process integrates
many channels — `bbx_to_ccx_emmm_qcd0` shows 150 distinct `this_config` values,
and consecutive written events jump between them — a leg set flagged on-shell
under one channel is still flagged when the next event runs under another.

Firing signature: MadGraph measures a final-state pair by `SumDot` (the pair's
invariant mass) on a leg set that is **not** in that event's own `ibwlist`.
Example, `bbx_to_ccx_emmm_qcd0` event 442, `this_config = 487`: its forest's
leg sets are `{48, 12, 60, 192, 61}` and its `ibwlist` is `[(48, −1)]`, yet the
pair `{3,4,7,8}` (mask 204) is measured as a resonance. Mask 204 is `12 + 192`,
the `h → ZZ` line of a *different* channel of the same directory.

Counts: 81 events of `bbx_to_ccx_emmm_qcd0`, 163 of `uux_to_ccx_emmm_qcd0`,
zero elsewhere. Both are `2 → 6` with three `Z`s and an `h`; no `2 → 2`,
`2 → 3` or `2 → 4` run is affected.

Handling: `cluster()` takes a `carried_on_shell: &[u32]` argument, documented as
exactly this. The comparison runs each event first with an empty list; only if
that disagrees does it take the reference's own extra flags (the leg sets it
measured by `SumDot` that are absent from the event's `ibwlist`) and re-run.
All 244 then reproduce **completely** — every candidate, merge, frame change and
scale — which is what makes this a diagnosis rather than a tolerance: the
divergence is entirely explained by the one input, and nothing else moves.

**What K4 must decide.** The flags do not enter the resonance filter on the
merge graph (that is rebuilt per event from `ibwlist`), only the measure of a
final-state pair and the mass its mother carries. For production this is a
non-issue — our generator owns its own state and the pure-function reading is
the correct one. For a *replay* of a banked `2 → 6` run it is not reproducible
from an LHE record. Of the ten names in `validate_scales`'s
`CLUSTERING_REQUIRED_RUNS`, four (`pp_to_llj{,_qcd2_qed2}`, `ee_to_mumua`,
`ee_to_mumu_tata_qcd0`) are unaffected and flip cleanly, while
`bbx_to_ccx_emmm_qcd0` and `uux_to_ccx_emmm_qcd0` carry a 0.8 % / 1.6 % event
population a pure replay cannot reach. Enforcing those two per-event is
therefore not free: either they stay informational, or their gate admits this
class explicitly with its count.

### K3.4 Diagnosed exception: the dump cannot name a process directory

The per-directory tables (`RUN`, `NQCD`, `MAP`, `PDG`, `RES`, `IFOR`) carry
`this_config` but no directory name, and the extraction de-duplicates them by
text. A run whose bank spans several subprocess directories therefore merges
their tables under one key. Two runs do: `pp_to_bb_qcd2` (`gg → bb̄` with
`maxsproc = 1`, `qq̄ → bb̄` with 2) and `pp_to_llj{,_qcd2_qed2}` (`qg → ℓℓq`
with 4, `qq̄ → ℓℓg` with 2). `NQCD` collides outright there: `(this_config 1,
config 1)` is `nqcd = 2` in one directory and `0` in the other.

Three consequences and how the session handles them:

- **The forests are separable** — an `IFOR` row carries one `sprop` per
  subprocess, so its length is `8 + maxsproc` and names the directory.
- **`nqcd` is not.** It is re-derived instead, by counting a channel's vertices
  whose three lines are all coloured (including the vertex that closes an
  s-channel-only tree on the beams, which `configs.inc` does not write). The
  merge graph needs only the *partition* of channels by equal order, and the
  seven unambiguous runs check the derivation: 120 tables equal.
- **The directory of an event** is settled first by a model test — every vertex
  a channel's forest implies must have a colour combination the model has, and
  where the line's flavour is per-subprocess (`sprop`, not `tprid`, which
  `configs.inc` writes once per channel from the group's *first* subprocess) the
  PDG triple must be one the model has too. That leaves 7 flavour assignments
  across the 3 affected runs undecided, and those consult the event's own
  candidate list once — per flavour assignment, not per event, so 7 bits
  altogether decide 30 000 events.

For K4 this is not a production concern (our channels know their own process),
but it is a **dump-format finding for any future re-bank**: the K2 records
should carry the process-directory name, which the writer already has in its
`SHARD` record and the extraction drops.

### K3.5 Unexplained by the reading, reproduced as behaviour

`filgrp` registers each line under its leg set *and its complement*
(`cluster.f:262`), writing the same PDG to both. For a channel that reaches the
beams through a spacelike line, the outermost such line's complement is a single
external leg — beam 2 — so the reading says that leg's `ipdgcl` entry ends up
carrying the line's `tprid` rather than the leg's own flavour. The dump's
initcluster tables agree with the reading: `PDG|4|2|2|4|2` for `pp_to_llj`
says mask `2` carries `2`.

**The live array does not.** Every `LINE` record of every event of every run
gives a single-leg mask the *subprocess flavour*: `pp_to_llj` event 0 has
`LINE|2|−1` where the reading predicts `2`. Reproducing the reading makes 7837
of 10 000 `pp_to_llj` events disagree on that leg's flavour and on the mother
PDGs derived from it; reproducing the behaviour makes all 90 000 events agree
everywhere. The engine therefore keeps a single external leg's own flavour and
skips the complement's PDG write for it (`graph.rs`), pinned by
`a_single_leg_keeps_its_flavour`. The leg set is still registered — only the
code on it differs.

No banked scale moves either way: both readings give the same `isqcd`/`isjet`
answers on every event here, because a t-channel `tprid` and the beam flavour it
sits on are both quarks. **This is a genuine open thread, not a settled one.**
K4 should re-derive it before relying on it, and the falsifier is cheap: a
process where the spacelike line's `tprid` and beam 2's flavour differ in
`isjet` — a `b`-initiated channel at `maxjetflavor = 4`, say — would separate
the two readings in the scale itself.

### K3.6 Branch coverage: what the bank cannot judge

Implemented per §K1 and reached by nothing in the bank, so uncovered:

- `ktscheme = 2` (`PYDJ`, `PYJB`) and `ickkw > 0` — `IS_PYJB`, `FS_PYDJ` never
  fire, and the `MatchingWeight` μF branch is unreachable.
- `dj`'s second massless–massive arm (`FS_DJ_MLESS_MASSIVE_2`) and its
  zero-three-momentum guard (`FS_DJ_DEGENERATE`). Only the first arm fires,
  1611× on `pp_to_llj`.
- μR branches `L1157`, `L1160`, `L1163`, `L1165`, `L1167`; only `L1153`
  (5 runs) and `L1169` (3 runs) are exercised. μF branches `NEXT3`,
  `JC0_BACKFILL1/2`, `JC0_BEAM1/2`, `PDFWGT`; only `GEOM_COLLAPSED` and
  `JC0_BOTH` are exercised. Confirms K2's finding 2.
- The `2 → 1` short-circuit (`nexternal = 3`), the `xqcut`/`xmtc` refusals, the
  μF floor refusal, and `MixedFixedFactorisationScales`.
- `scalefact ≠ 1`. Every banked run has `1.0`, so §K1.9's table stays a reading;
  the engine applies exactly one power everywhere and `beam2_from_beam1` has no
  counterpart in the new code — K4 deletes it from `scales.rs`.
- `jcentral_override` on its *initial-state* feeder: it fires 141× on
  `pp_to_bb_qcd2` and only through `mt2last`, as §K1.6 predicted.

### K3.7 Confirmed against the bank

- **§K1.11 finding 2 is live on all three routes.** The coupling-order filter
  (`pp_to_bb_qcd2`: `this_config = 3` sees only the two `nqcd = 0` channels), the
  resonance tagging, and the memo (1873 restricted re-clusters on `pp_to_llj`).
  `igraphs(1) ≠ iconfig` on 7 to 7877 events per run, and the engine reproduces
  the collapse at `cluster.f:811-817` every time.
- **The tie-break.** `uux_to_uux`'s 32 inflated candidates over 16 events are
  reproduced, and the general path gives `μR = μF = 250.000125` — note 22's
  `250.0001` row, re-derived rather than fitted. Pinned hermetically by
  `a_wholly_crossed_event_carries_the_tie_break_into_the_scale`, with
  `colourless_beams_keep_the_tie_break_out_of_the_scale` as its negative control
  and `an_exact_tie_goes_to_the_pair_visited_first` for the strict comparison.
- **The dead mass-propagation guard** (§K1.11 finding 6): implemented as
  `A .or. B` and confirmed by every `2 → 6` event.
- **`mt2last` is set only after a final-state last merge**, and the un-boost at
  `cluster.f:786-792` sits inside that branch: `pp_to_llj` events whose last real
  merge is initial-state keep the core scale in the boosted frame, and the
  engine reproduces both.
- **`ipartupdate` mutates `ipdgcl`** in-event, and the mutation is visible: the
  `LINE` comparison pins the mutated jet flavours (`−1` for a beam line that
  emitted a gluon) event for event. The cross-event persistence that would
  follow from the common block is *not* observed to matter here — every event's
  mutations are re-derived from the merge graph and agree.

### K3.8 What K4 inherits

`ScaleChoice` is untouched: `-1` still takes the closed-form-or-refuse path, and
no enforced row moved.

One inventory correction for the flip: `CLUSTERING_REQUIRED_RUNS` holds **ten**
names, not six. Six are the dumped no-closed-form runs; the other four —
`ddx_to_epemg`, `gu_to_epemu`, `gux_to_epemux`, `uux_to_epemg` — are the `2 → 3`
partonic σ rows, which have **no clustering dump at all**. K4 gets no
intermediate oracle for them: their only reference is the banked
`SCALUP`/`<rscale>`/`<pdfrwt>`, which §K1.10 already showed is blind to a wrong
tie-break or a wrong line PDG that does not move the final number. They are
`2 → 3` at `lpp = 0`, so the boost cannot fire and the merge graph is small; the
honest framing is that flipping them tests the *scale*, and this session's dumps
are what tests the path that produced it. The wiring K4 has to do is to build a `ChannelSet` from
our own diagram enumeration rather than from a dump's `IFOR` records — the one
derivation step this session did not take, because the dump's directory
collision makes it uncheckable on 2 of the 9 runs and the merge-graph derivation
*from forests* is checkable on all 7 others. §K1.11 finding 3 (drop every
diagram whose maximum vertex arity exceeds the process minimum) belongs to that
step, and `ConfigForest` is the interface it must produce: one entry per vertex
of the diagram re-rooted toward the highest-numbered initial leg, s-channel
entries carrying `sprop` per subprocess and t-channel entries carrying `tprid`,
with the closing vertex written only when the channel reaches the beams through
a spacelike line (`export_v4.py:2229`, `if len(tchannels) > 1`).

## S5 — the W-current amplitude: the defect, localised

S4 left the spine reference row's |M|² disagreeing with MadGraph by factors of 2 to 63
point by point. This session took it to the finest linear level MadGraph exposes. The
answer is **a relative sign between diagrams**, not a coupling, a colour factor, a
propagator or a width — and it is localised to eleven of the 35 diagrams, exactly.

**It is not fixed, and that is a deliberate stop.** Three candidate correction rules were
written down and each was falsified by a gated control before it could be landed; the one
predicate that fits this process exactly cannot be added to `fermi_sign` without breaking
a bit-exact row. Landing a sign rule that reproduces this process without a derivation is
what the Physics Validation section forbids, so what landed instead is the instrument, the
localisation and the falsification record — enough that the next session can check any
candidate rule against a known-exact answer in one run.

### The instrument

`ud_to_epemud_qcd0` is now a registered amplitude process end to end:
`gen_amplitude.PROCESSES` (25 points at each of 200 and 500 GeV, seed 71),
`build_amplitude.sh`'s generic and amp-probe lists, and the committed table
`validation/madgraph/amplitudes/ud_to_epemud_qcd0.json` — 74 points (24 projected
events + 50 grid), six of them carrying MadGraph's `AMP(1:35)` and `JAMP(1:2)` for
every one of the 8 helicity combinations its amplitude does not vanish on. Regenerating
the other 19 tables left them byte-identical, so the registry entry moved nothing else.

MadGraph's own bookkeeping, read out of its generated `matrix1_orig.f`:
`NGRAPHS = 35`, `NCOLOR = 2` with the upper-triangle `CF = /9,6/,/9/` over `DENOM = 1`
— which the `DO J = I, NCOLOR` sum makes the symmetric `[[9,3],[3,9]]` — flows
`T(5,1)T(6,2)` and `T(5,2)T(6,1)`, and `JAMP(1) = -Σ AMP(i)` over graphs 1–16 and
28–35 against `JAMP(2) = +Σ AMP(i)` over 17–27, every coefficient of unit modulus. The
21 `AMP2` accumulators are parsed from the same file; the row is listed in
`KNOWN_CONFIG_MERGE`, which is exactly the case that constant exists for (our 35
singleton configurations are finer than MadGraph's 21).

### The measurement

Per diagram, per helicity, over 48 (point, helicity) rows: **every one of our 35
diagrams pairs with exactly one MadGraph graph at normalised overlap `1.00000`, with a
fitted constant of modulus `1.000000` and a residual of at most `3.5e-15`.** The
pairing is banked in `MG_DIAGRAM_ORDER`. So no diagram is individually wrong: not its
coupling, not its propagator, not its kinematics.

What is wrong is the *phase* of that constant. It should be one constant for the whole
process (`±i`, the factor our diagram roots carry and MadGraph's `AMP()` does not).
Instead it takes both values:

| fitted constant | our diagrams | MadGraph graphs |
|---|---|---|
| `-i` | d00–d03, d12–d15, d25–d32; d18, d21, d22 | 9–16, 28–35; 17, 18, 19 |
| `+i` | d04–d11; d16, d17, d19, d20, d23, d24, d33, d34 | 1–8; 20–27 |

Against MadGraph's uniform colour coefficients this makes **eleven diagrams wrong
relative to the other 24**: MadGraph graphs **1–8, 17, 18, 19**. Flipping exactly those
eleven and recontracting through `CF` takes the worst relative `|M|²` deviation over all
74 banked points from **5.1e+1 to 4.1e-14** — inside the standing `1e-12`. The eleven
are a complete explanation of the disagreement, and nothing else is wrong.

### What the eleven are

- graphs 1–8 (our d04–d11): the three-rung ladders `γ/Z^t — e^t — γ/Z^t`, where the
  **lepton line itself is the spacelike spine** between the two quark lines;
- graph 17 (d18): the same with `W^t — ν^t — W^t`;
- graphs 18, 19 (d21, d22): `W^t — W^t` closed by a timelike `γ*/Z* → e+e-`, the only
  two diagrams with a triple-gauge vertex.

The exact predicate is **the number of boson lines on the beam-to-beam spine is even**
(2 for all eleven; 1 for all 24 others). Every diagram of this process has the same
fermion-line endpoint classes — two mixed quark lines and one crossed lepton line — so
`spine_sign_from_flow` returns `-1` for all 35 and carries no relative information at
all here. `diagram.sign` splits only along the colour flows (`-1` neutral-current,
`+1` charged-current), matching MadGraph's own `JAMP` coefficients. `build_convention_sign`
and `reversed_convention_sign` are `+1` for all 35. `yang_mills_vvv_sign` is `-1` for
d21 and d22 and `+1` for the rest. That accounts for every factor in `fermi_sign`, and
none of them separates d00 from d04.

### Three mechanisms, three falsifications

1. **"a crossed final–final line takes one further −1 per internal fermion
   propagator"** — fits the nine ladder diagrams exactly. Falsified by
   `ee_to_mumu_tata_qcd0`: its diagrams 0–15 carry a final–final μ or τ line with one
   internal propagator alongside diagrams 17–24 that carry none, and the process is
   gated at `6.0e-14`. The rule would flip their relative sign.
2. **"the WWγ/WWZ vertex must not take the Yang–Mills VVV sign"** — fixes d21 and d22.
   Falsified by `ee_to_wpwm`, gated at `6.2e-15`: its two `s`-channel diagrams carry
   exactly one WWV vertex and need `ym = -1` to sit at the same `fermi_sign` as the
   ν-exchange diagram. (`g g > g g` cannot decide this — its `ym` and
   `build_convention_sign` move together across all four diagrams.)
3. **"one further −1 per spacelike boson propagator"** — the predicate that fits all
   eleven. Cannot be *added* to `fermi_sign`: `u u~ > u u~`'s `s`- and `t`-channel
   diagrams differ by exactly that count, their relative sign is already right, and a
   second −1 would break a bit-exact row.

So the defect is not a missing multiplicative factor of any of those forms. It is in the
interaction between the crossing conventions and a topology no banked process had
before: `ud_to_epemud_qcd0` is the first row with **two mixed quark lines**, hence the
first where a *final–final* fermion line can itself be spacelike, and the first with a
triple-gauge vertex all of whose legs are internal propagators.

Two further controls, both clean, that narrow where it is *not*:

- **Rooting soundness passes on the process**: adding it to `MG_VALIDATED_PROCESSES`
  took the sweep from 133 to **270 re-rootings, 0 failures**. Every diagram's amplitude
  is root-invariant, so the `GammaIout`/`GammaOout` asymmetry between d00 and d04 (they
  build their off-shell fermion from opposite ends, where MadGraph uses `FFV1_2` for
  both) is not the lever.
- **The CKM is not involved**: the pinned model's `MDL_CONJG__CKM1X1 = 1.000000D+00`,
  so `GC_100` is `ee·i/(sw·√2)` on both sides.

### What landed

The row is `hermetic` / `info` in the manifest and listed in the oracle's new
`KNOWN_LINEAR_DISAGREEMENT`. A listed row's linear-level checks *record* rather than
raise, so the whole comparison still runs and the `info` cell carries the same numeric
fields a gated one does — |M|² max rel `5.12e1` (grid) and `2.80e1` (event), per-flow
`1.29e0`, JAMP2 `9.71e0`, per-configuration `3.03e1` — rather than the zeros an
early-return would have left. Its note names how many checks the disagreement reached
and the first of them, so a *change* in the disagreement is visible rather than silent.
The entry is two-way: if the row starts agreeing, the oracle fails and asks for the
exemption to be dropped and the cell promoted. `validate_sigma`'s `Plan::Info` reason
now names the relative sign rather than the aggregate ratio.

`MG_VALIDATED_PROCESSES` was **not** extended — the process is not validated, and that
list drives the library-level coverage sweeps.

### For the sprint manager

The remaining work is one question: what is the correct rooting-convention sign for a
diagram whose beam-to-beam spine passes through a crossed fermion line, and for a
triple-gauge vertex whose legs are all internal? It wants a derivation from the
C-conjugation identity the crossing rests on (`ū₁Γv₂ = −ū₂(CΓᵀC⁻¹)v₁`, and what it does
to a chain whose propagator momentum crosses the beam cut), not another fit — the
required sign vector is now known exactly, so any candidate rule can be checked against
it in one run, and against the 19 gated tables in the same run.
## K4 — production wiring and the scale-row flips

`ScaleChoice`'s closed forms are gone. `dynamical_scale_choice = -1` now takes
one path — the kT clustering of §K3's engine — through
`ScaleChoice::cluster_scales`, and `coupling::topology`, `ClusterTopology`,
`BeamConnections` and the whole `clustered()` collapse are deleted rather than
left as a second implementation.

### K4.1 `ConfigForest` from our own diagrams

`coupling::cluster::configs` derives the channel forests from vibegraph's
enumerated diagrams: the tree re-rooted toward beam 2, s-channel lines (subtree
carrying neither beam) ahead of the spacelike chain (subtree carrying beam 1)
ordered from beam 1 inward, the closing vertex written only for a channel that
reaches the beams through a spacelike line, and the diagrams whose largest
vertex exceeds the set's minimum dropped.

**The consistency gate is exact and it is whole.** For every dumped run whose
process directory groups a single subprocess, the derived forests are compared
against the `IFOR` records line for line — every line's leg set, both its
daughters' leg sets, `tprid`, `sprop`, the mass and the width — as a bijection
over channels (`derived_channel_forests_match_the_generated_ones`):

| process | channels / diagrams | lines |
|---|---|---|
| `b b̄ → c c̄ e⁺e⁻ μ⁺μ⁻ QCD=0` | 615 / 615 | 3327 |
| `e⁺e⁻ → μ⁺μ⁻ τ⁺τ⁻ QCD=0` | 25 / 25 | 83 |
| `e⁺e⁻ → μ⁺μ⁻ γ` | 8 / 8 | 20 |
| `e⁺e⁻ → t t̄` | 2 / 2 | 2 |
| `u ū → c c̄ e⁺e⁻ μ⁺μ⁻ QCD=0` | 579 / 579 | 3135 |
| `u ū → u ū` | 2 / 2 | 3 |

**6570 lines, all equal.** Channel *numbering* is not compared and cannot be —
MadGraph numbers configs by its own diagram order — and nothing needs it to be:
the merge table reads a channel's identity only through its QCD order.

Two things the comparison settled that the reading had not.

- **The sign on a timelike line.** `configs.inc` writes the particle that
  *decays into* the line's subtree, not the one leaving it. Above `e⁺ e⁻ μ⁺`
  sits a `μ⁺`, and the first derivation had it backwards; 552 of 615 forests
  disagreed on exactly that entry and nothing else.
- **The four-point filter is unexercised by the bank.** No diagram of any of the
  six is dropped by it, so finding 3 is pinned hermetically instead:
  `g g → g g` has 4 diagrams and 3 channels, and the surviving three are the
  s-channel gluon (whose closing vertex stays implicit, so it carries one line)
  and the two spacelike ones (two lines each).

### K4.2 What replaced the closed forms, and what it cost

`ScaleEvent` no longer carries a topology declaration; the caller supplies a
`ClusterInput` — the process's `ChannelSet`, a `ColorTable`, the integration
channel and the subprocess. `hadronic::compile_scale_source` builds it from the
same diagrams it already had, so no integrand carries a table keyed by process
name.

**`beam2_from_beam1` is deleted** (§K1.9's action). Nothing carries it forward:
`setclscales` applies one power of `scalefact` to `μR` and to each `μF`, and
`scalefact_reaches_every_scale_exactly_once` asserts that on the branch where
3.5.7 applied two — colourless beams, `jcentral` zero on both.

**The general path against the closed forms it replaces**, run once before the
deletion, every event of all 14 closed-form runs under *every* channel: 8 runs
agree to `0.0` or `1.1e-16`; `pp_to_bb*` do not. There the clustering's first
merge is initial-state and the leftover leg's measure is taken in the boosted
frame, which the collapse did not model — the two differ by `1e-9` relative,
four orders inside the printed field, and it grows to `1.5e-6` on some events.
Where they differ **the general path is the one pinned bit-for-bit against
MadGraph's own intermediates** (`pp_to_bb_qcd2`, 10000/10000, §K3.1), so the
closed form was the approximation. The comparison was a scaffold and is not
kept; the standing net is the enforced replay below, plus
`the_general_path_keeps_the_beam_crossing_population`, which requires
`u ū → u ū`'s tie-break population to be **exactly 16 events at 250.000125**
— K2's dump count, and note 22's `250.0001` row re-derived rather than fitted.

### K4.3 The flip, and the integration channel

`validate_scales` now replays **every** banked run through the clustering.
840 000 comparisons over 270 000 events in 27 runs, all inside their printing
budget; `AQCDUP` recomputed from the same scales for 230 000 events across 23
runs. The budget gained a term it should always have had: the *incoming* momenta
are printed inputs too, and the clustering reads them.

The honest difficulty is §K1.11 finding 2. The cluster scale is a function of
the event **and** of the integration channel, and an LHE record does not carry
one. The replay adopts the first channel whose `μF` lands inside `SCALUP`'s
budget and reads every other field — and the independent `AQCDUP` oracle — off
that same channel, so a wrong clustering cannot be repaired field by field. How
often the choice matters is reported per run rather than assumed away:

| run | events needing a channel other than the first |
|---|---|
| `gu_to_epemu` / `gux_to_epemux` | 7204 / 7231 |
| `pp_to_llj_dyn` / `pp_to_llj` | 5768 / 5572 |
| `ee_to_mumua` | 370 |
| `ee_to_mumu_tata_qcd0` | 262 |
| `pp_to_bb_qcd2` | 141 |
| every other replayed run | 0 |

So the channel is live on six runs and inert on twenty-one. That is a
measurement, and it is what the production default — channel 1 where the caller
sampled none — rests on.

### K4.4 The four decisions this session owed

- **D4.** `pp_to_llj_qcd2_qed2` is declared a duplicate of `pp_to_llj` and
  skipped with a printed note; enforcement is on `pp_to_llj`. Its inventory row
  stays until Z prunes it.
- **The two `2 → 6` runs stay informational**, and not for a tolerance. Two
  separate inputs are missing from an LHE record: the carried-over on-shell
  flags of §K3.3 (81 and 163 events), and the integration channel out of 615 and
  579 — searching that many for one that agrees is a gate almost anything
  passes. What enforces them is finer and already green: `validate_kt_cluster`
  reproduces all 20 000 of their events, every candidate and both scales,
  against the instrumented dump.
- **The four llj partonic runs flip** (`uux_to_epemg`, `ddx_to_epemg`,
  `gu_to_epemu`, `gux_to_epemux`), all inside budget. **Their blind spot, stated:
  they have no clustering dump, so their only reference is the banked
  `SCALUP`/`<rscale>`/`<pdfrwt>`, which §K1.10 already showed cannot see a wrong
  tie-break or a wrong line PDG that does not move the final number.** What
  tests the path that produced their scale is this session's dumps on other
  processes, not these rows.
- **`pp_to_jj` flips with a diagnosed exception of 9 events in 10 000
  (0.09 %).** All are `q q' → q q'` subprocesses with a single integration
  channel and two allowed beam–leg pairs. MadGraph inflated the winning
  candidate and the replay did not, and the two scales differ by `√(1 + 10⁻⁶)`
  **and by nothing else** — `<rscale>`'s eight digits put the ratio at `5.03e-7`
  and `5.16e-7` against the inflation's `5.000e-7`. The gate admits an event only
  if it carries that signature and asserts the count for equality, so the class
  cannot quietly become a different one. Which of two numerically degenerate
  candidates `cluster.f` chose is decided below the eleven digits the record
  prints; the engine's own tie-break is pinned bit-for-bit elsewhere.
  **Settling this population needs a K2-style clustering dump for `p p → j j`,
  which the sprint did not bank.**

### K4.5 §K3.5 is explained, not merely reproduced

The falsifier K3 left was run and came back *negative* — and the reason is the
explanation.

A line's complement is a single external leg exactly when the line has
`nexternal − 1` legs below it, and **only one line ever does**: the vertex that
closes a channel on the beams, whose complement is beam 2 alone. An
s-channel-only channel does not write that vertex at all, so it has no
single-leg complement. And `export_v4.py:2262` gives the closing line
`tprid = abs(leg 2's own id)` — the vertex's last leg *is* beam 2. So the code
the complement rule would write is the leg's own code **up to sign**, and
`isqcd`, `isjet` and `is_octet` are all questions about `abs(pdg)`.

The two readings are therefore the same table wherever the clustering reads
them, which is why no banked scale moved either way and why the falsifier could
not fire: `b b̄ → b b̄` at `maxjetflavor = 4`, where the exchanged gluon is a jet
and the beams are not, gives the identical scale to the bit under both. Both
halves are pinned (`only_the_closing_line_can_write_a_single_leg_entry`,
`overwriting_a_single_leg_entry_moves_no_scale`). What remains signed is
`ipartupdate`'s flavour propagation, and that is what the dump's per-event
`LINE` records compare against. **The thread is closed.**

### K4.6 What K5 must know

Landing the general scale moved `p p → ℓ⁺ℓ⁻ j` at a dynamical scale from
"refused" to "runs", and it immediately met the next limit: **NNPDF23's `αs`
table stops at `Q = 10 TeV` and a per-event scale on a 13 TeV collider can
exceed it.** LHAPDF extrapolates past its own table; this crate does not, and it
was reaching that edge as a panic on whichever events happened to pass it.
`EventScaleSource::from_run_card` now refuses at setup when a tabulated coupling
stops below `ebeam1 + ebeam2`, naming both — a run stops before it starts rather
than mid-integration.

**This blocks K5's dynamical-scale σ rows and the capstone as it stands**, and
the decision is not one this session should improvise: either extrapolate as
LHAPDF does (and validate the extrapolation), or bank against a set whose table
covers the collider. It is stated here rather than worked around.

**And a second one, found by registering the new runs in `validate_alphas`.**
`GridAlphaS` reads a set's `αs` knots with a straight line in `log Q²`; LHAPDF
reads the same knots with a cubic. At `Q = M_Z` the two agree to `1e-8` — the
scale sits 2.4e-5 of the way into its knot interval — which is why
`pp_to_bb_fixed` and `pp_to_llj_fixed` reproduce all 20 000 of their `AQCDUP`
digits and why `GRID_ALPHA_S_TOL` could be set at `1e-7` at all. A *dynamical*
scale lands mid-interval, and there the gap is the `~1.7e-4` relative that
tolerance's own reasoning predicted: measured on the two new lhapdf runs at
**1076 and 1777 times the printed budget, on 9993 and 9976 of 10 000 events**.

The two are declared as `GRID_INTERPOLANT_RUNS` and excluded from the `AQCDUP`
oracle, with an assertion that they stay excluded — a linear reading that had
quietly become accurate mid-interval would make the list wrong rather than
harmless. Nothing about the *scale* is in question: the same events' `SCALUP` is
reproduced from their momenta inside its own budget. But a cross section
computed at a dynamical scale off an `lhapdf` set carries a systematic `1.7e-4`
on `αs` until the knots are read as LHAPDF reads them, and **K5 should size that
against the σ agreement it is trying to demonstrate before flipping those
rows.**

## S6 — the crossing sign rule

The fix is one condition in `spine_sign_from_flow`: the per-propagator flip that fired
on *initial–initial* fermion lines now fires on every line with **at least one
initial-state endpoint**, mixed lines included. `inc_a && inc_b` became
`inc_a || inc_b`.

What makes that a derivation rather than a fit is that the rule it generalises — the
initial-state spine sign, pinned since the `u u~ → c c~ e+e- μ+μ-` per-diagram oracle —
is a *special case* of a slot-binding mismatch that mixed lines share and crossed lines
do not. Both of S5's falsified candidates are the same −1 attributed to the wrong line
class.

### The convention that actually differs

Both sides cross half the external legs, and not the same half.

| leg | diagram enumeration (this side) | reference HELAS bookkeeping |
|---|---|---|
| initial | physical identity | **anti** identity (all-outgoing) |
| final, on a mixed line | anti identity, restored to physical by `mixed_line_final_legs` | physical identity |
| final, on a crossed line | **anti** identity, kept | physical identity |

A UFO fermion slot pairs with a definite spinor adjoint — the pair-first slot takes the
ket, the pair-second the bra. Because MadGraph crosses the *initial* legs into the
all-outgoing identity before binding them, its binding never disagrees with the adjoint
of the wavefunction it holds: an incoming `e-` is bound to the `ℓ+` slot and `IXXXXX`
gives it a ket; an outgoing `e-` is bound to the `ℓ-` slot and `OXXXXX` gives it a bra.
Ours disagrees at exactly two kinds of leg — every **initial** leg (we keep the physical
identity where the reference crosses it) and every **uncrossed final** leg (we replace
the wavefunction but not the slot binding). It agrees at every **crossed final** leg,
where the anti identity and the anti wavefunction move together.

So the lines read against their own slot arrow are the ones with at least one
initial-state endpoint: initial–initial **and mixed**. A crossed (final–final) line is
read along its arrow.

### What reading a line backwards costs

Reading a bilinear against its slot arrow replaces each vertex structure by `C Γᵀ C⁻¹`,
which for `Γ = γ^μ P_χ` is `−γ^μ P_χ̄`. The chirality flip is applied per vertex by
`chiral_correction`. The −1 is applied **once per line**, by
`reversed_convention_sign` at the line's single vector-rooted sink — a fermion line
meets exactly one such vertex in a rooted tree, so that channel can contribute at most
one factor however long the line is. A line with `V` vertices needs `(−1)^V`; the
remaining `V − 1` are one per internal fermion propagator, and supplying them is what
`spine_sign_from_flow`'s first arm is for.

The account makes a structural prediction that is not the thing being fixed:
`reversed_convention_sign` should be exactly `(−1)^(#initial–initial + #mixed lines)`
per diagram and should never fire on a crossed line. Measured over
`e+e- → e+e-`, `u u~ → u u~`, `g g → t t~`, `e+e- → μ+μ-τ+τ-`, `e+e- → W+W-`,
`g u → e+e- u`, `u u~ → e+e- g` and `u d → e+e- u d`, it is.

The arm was written for initial–initial lines only, so **mixed lines were getting
`(−1)^1` where they needed `(−1)^V`.** No banked row could see it before:
`e+e- → e+e-` and `u u~ → u u~` have mixed lines with no propagator at all;
`g u → e+e- u` and `g u~ → e+e- u~` have one mixed line carrying one propagator in
*every* diagram, a uniform sign the per-configuration phase fit absorbs; no other banked
process has a mixed line at all. `u d > e+ e- u d QCD=0` is the first row with **two**
mixed quark lines, so its single internal fermion propagator sits on a mixed line in
some diagrams and on the crossed lepton line in others. That split is 24 / 9 (+2 with no
fermion propagator at all) — exactly S5's partition, with the two triple-gauge diagrams
landing on the un-flipped side without being mentioned.

### Why the S5 candidates failed

- **"one further −1 per internal fermion propagator on a crossed final–final line"**
  puts the factor on the one line class that is *not* read backwards. On this process
  alone it is indistinguishable from the right rule — it selects the complement of the
  24, and a global sign is absorbed by the fit — but it moves `e+e- → μ+μ-τ+τ-`
  diagrams 0–15 against 17–24, and `g g → t t~`'s t/u-channel diagrams against its
  s-channel. Two independent bit-exact rows falsify it.
- **"the WWγ/WWZ vertex must not take the Yang–Mills VVV sign"** was needed only
  because graphs 18 and 19 carry no internal fermion propagator: under the crossed-line
  reading they fell outside the nine ladders and wanted a second mechanism. Under the
  mixed-line reading they need nothing, and `yang_mills_vvv_sign` is untouched. Two
  diagrams landing in the right class *without* being named is the check the account had
  to pass and the fitted rules did not.
- **"one further −1 per spacelike boson propagator"** is the predicate that fits this
  process; it coincides with the mixed-line propagator count here and is false on
  `u u~ > u u~`.

The C-conjugation identity S5 pointed at is the right tool; what it acts on is the
vertex, not the whole chain, and the count that matters is vertices on a
slot-reversed line, not boson rungs on the beam-to-beam spine.

### Measured

`ud_to_epemud_qcd0`, against the committed table (74 points, 6 of them per-helicity):

| check | S5 | S6 |
|---|---|---|
| `\|M\|²` max rel, grid / event | 5.12e1 / 2.80e1 | **4.16e-14 / 2.62e-14** |
| per-flow (JAMP) | 1.29e0 | **2.96e-15** |
| `JAMP2` | 9.71e0 | **2.61e-14** |
| per-configuration amplitude vs bare `AMP()` | 3.03e1 | **6.57e-15** |
| fitted global constant `G` | split ±i | **−1i**, `\|G\|−1 = 1.1e-14` |

The other 19 amplitude tables are unmoved and still gated; the rooting-soundness sweep
stays at **270 re-rootings, 0 failures**; `e+e- → μ+μ-τ+τ-` (6.06e-14), `e+e- → W+W-`
(8.44e-15) and `u u~ → u u~` (2.00e-15) — the three controls that falsified the S5
candidates — are unmoved.

One oracle bug had to be fixed to see the last row of that table. The per-configuration
comparison built its MadGraph `AMP()` index by flattening MadGraph's own `AMP2` grouping,
which is only an index source while that grouping lists graphs in graph order. This
process's grouping is `[0,2,4,6],[1,3,5,7],…`, so position `k` was not graph `k` and the
comparison was scrambled — it read 3.03e1 whatever the physics did. Where a row is in
`KNOWN_CONFIG_MERGE` the pairing now comes from the diagram behind each of our
configurations instead. `ee_to_ee`, the other merged row, is unaffected (its flattening
happens to be sorted).

### The row, promoted

With the linear level agreeing, σ was re-measured and moved from **1.0860e-1 pb** (a
factor 7.7 high) to **1.409864e-2 ± 1.841e-5 pb** against MadGraph's banked
**1.410700e-2 ± 3.4241e-5 pb** — pull −0.22, rel −0.06%, χ²/dof 1.20, at the same
120 000 × 8 budget and the same seed S4 used. Nothing about the map changed; the
factor 7.7 was the missing sign all along, which is what S4's "the ratio is
region-dependent and almost always > 1" was reading.

Two axes before enforcing it:

| seed | σ (pb) | pull | rel | χ²/dof |
|---|---|---|---|---|
| 20260719 | 1.409864e-2 | −0.22 | −5.9e-4 | 1.20 |
| 11 | 1.409501e-2 | −0.31 | −8.5e-4 | 1.00 |
| 22 | 1.407280e-2 | −0.88 | −2.4e-3 | 1.15 |
| 33 | 1.406969e-2 | −0.96 | −2.6e-3 | 1.16 |
| 44 | 1.409319e-2 | −0.36 | −9.8e-4 | 1.39 |
| ×2 budget | 1.410882e-2 | +0.05 | +1.3e-4 | 1.26 |
| ×4 budget | 1.408890e-2 | −0.51 | −1.3e-3 | 1.19 |

The budget ladder does not shrink the residual, and should not: at ×4 our own error is
9.5e-6 against the reference's 3.4e-5, so the pull is floored by MadGraph's error, not
ours. What the ladder does rule out is a defect, which would migrate between seeds at
fixed size rather than scatter inside a fixed band. `rel_tol` is set to **0.01**, 3.8×
the worst seed, and `probe_resonant_seed_stability` now carries the row (and a
budget-ladder arm for every row it sweeps).

Cells flipped: `amplitudes` `hermetic`/`info` → `hermetic`/**gate**, `integrals`
`banked`/`info` → `banked`/**gate**. `MG_VALIDATED_PROCESSES` grew to 19 entries — which
is what took the rooting-soundness sweep to 270 re-rootings — and
`KNOWN_LINEAR_DISAGREEMENT` is empty again. The `samples` cell stays `uncovered`, but
its reason is no longer "the cross section is informational": the comparison is simply
unwritten.

### What the new test cannot see

`spine_sign_separates_mixed_line_and_crossed_line_propagators` asserts the 24 / 9 / 2
split of `u d > e+ e- u d QCD=0` and that the mixed-line class comes out at the opposite
spine sign to the other two, with `g g > t t~` as the negative control that a crossed
line's −1 does not count propagators. It is blind to a sign common to every diagram of a
process — the per-configuration phase fit absorbs those, and so does |M|² — and to
anything outside the spine channel. The absolute per-diagram values are pinned only by
the `ud_to_epemud_qcd0` row of `amplitude_oracle`, which is now gated; if that row were
ever demoted, this test alone would still pass with `spine_sign_from_flow` off by a
global sign.
## K5a — αs grid fidelity

Both of §K4.6's blockers are closed, and the second one is what closed the
first. `GridAlphaS` now *is* LHAPDF's `AlphaS_Ipol`, so a scale past the top of
the table has a defined reading and the setup-time refusal that stood in for one
is deleted. `GRID_INTERPOLANT_RUNS` and its stay-excluded assertion are gone;
`pp_to_jj` and `pp_to_llj_dyn` join `banked_events_reproduce_aqcdup`.

### K5a.1 The algorithm, and what pins each line of it

The call MadGraph makes is `ALPHAS(Q)` in `alfas_functions_lhapdf.f`, one line
forward to `alphasPDF(Q)`; `LHAGlue.cc:1411` forwards that to
`PDF::alphasQ(Q)`, which is `alphasQ2(q*q)` (`PDF.h:504`) on the set's `AlphaS`
object, and for `AlphaS_Type: ipol` that object is `AlphaS_Ipol`. Every element
below is read off that class rather than inferred from its output.

| element | what it is | source |
|---|---|---|
| interpolation variable | `ln Q²`, natural log | `AlphaSArray::_syncq2s` |
| interpolant | cubic Hermite, `2t³−3t²+1` basis | `AlphaS_Ipol::_interpolateCubic` |
| endpoint slopes | central inside a subgrid, forward at its first knot, backward at its last | `AlphaS_Ipol::alphasQ2`, `AlphaSArray::ddlogq_*` |
| subgrids | table cut at every repeated `Q²`, pieces keyed by their first `Q²` so a repeat of an earlier key replaces it | `AlphaS_Ipol::_setup_grids` |
| above the last knot | **frozen** at the last tabulated value | `if (q2 > _q2s.back()) return _as.back();` |
| below the first knot | power law in `Q²` whose exponent is the first interval's gradient in the `log₁₀`–`log₁₀` plane | `AlphaS_Ipol::alphasQ2` |
| a reading `≥ 2` in magnitude | replaced by `DBL_MAX` | `_interpolateCubic`'s return |

The reading is of 6.5.3's sources; the oracle below is MadGraph's own 6.5.6, and
414 probes agreeing to the bit is what says the two are the same routine.

The first three rows are where a plausible-looking wrong answer could have sat,
and one hermetic fixture separates them instead of asserting them together. **A
quadratic in `ln Q²` sampled on the knots is reproduced exactly, and only in the
intervals where both slopes are central** — a central difference of a quadratic
*is* its derivative, a one-sided difference is not, and Hermite with exact
endpoint derivatives is exact for anything cubic or below. So a linear reading,
a reading in `Q`, and central differences carried into the edges each fail a
different half of `a_quadratic_in_log_q2_is_exact_inside_and_only_inside`.

### K5a.2 The 20 000 events, at the same standard as every other run

| run | a straight line through the same knots | the cubic |
|---|---|---|
| `pp_to_llj_dyn` | 9976 / 10000 outside, worst **1776.6×** budget, 23 events digit-exact | **0** outside, worst **0.999**, 9798 digit-exact |
| `pp_to_jj` | 9993 / 10000 outside, worst **1076.3×** budget, 6 events digit-exact | **0** outside, worst **0.996**, 9681 digit-exact |

The left column is remeasured here rather than carried over, and it corrects
§K4.6's pairing: the `1777×` belongs to `pp_to_llj_dyn` and the `1076×` to
`pp_to_jj`, not the other way round. The counts were right.

Both runs now sit against the same bound every other run does — the budget is
saturated because events pile up against a rounding boundary, not because the
gate is close to failing. `banked_events_reproduce_aqcdup` reports **280 000
events across 28 runs inside their printing budget, worst 0.999** (in
`pp_to_llj_dyn`), up from 260 000 across 26.

The `M_Z` reading gained six orders with them. This crate's `αs(M_Z)` off the
set is now `0.13000271085472234` against MadGraph's own 17-digit report of
`0.13000271085472234` — every bit — so `GRID_ALPHA_S_TOL` drops from `1e-7` to
`1e-14`, which is two orders above the arithmetic noise of one `ln` call and
leaves room for a system `libm` that rounds it differently.

### K5a.3 The ceiling: K4's premise corrected, and the oracle that replaced it

**No banked event is above the ceiling.** The largest `SCALUP` in
`pp_to_llj_dyn` is `845.5386` GeV and in `pp_to_jj` `167.1938`, against a table
that runs to `10000`; the smallest `AQCDUP` in either is `0.0976`, nowhere near
the top knot's `0.07695485`. So the expectation that llj_dyn's tail carries the
above-ceiling answer is wrong. The panic K4 met came from the integrator
sampling high-`ŝ` points, not from anything that survived unweighting into the
bank — §K5a.5's measurement is the same mechanism seen directly, an integration
reaching `Q = 10647` GeV where no banked event passes `846`. **The banked oracle
cannot speak about either end of the table**, and a policy adopted on its
strength would have been adopted on no evidence.

What speaks instead is LHAPDF itself, through the oracle generator the PDF gate
already had. `validation/pdf/gen_oracle.cpp` now also dumps `gpdf.alphasQ(q)` —
the same call, through MG's own LHAPDF 6.5.6 — on a probe set built for the
branches the events cannot reach:

| category | probes (NNPDF23 / NNPDF31) | what it covers |
|---|---|---|
| `knot` | 51 / 50 | every tabulated scale |
| `interval` | 150 / 144 | `t = ¼, ½, ¾` of every interval in `ln Q²`, edges included |
| `threshold` | — / 3 | either side of and exactly at NNPDF31's repeated `Q = 4.92`, the only real subgrid split in the two sets |
| `above_qmax` | 4 / 4 | `q_max·(1+10⁻¹²)`, `1.3`, `2.6`, `10³` |
| `below_qmin` | 4 / 4 | `q_min·(1−10⁻¹²)`, `0.5`, `0.1`, `10⁻³` |

**All 414 probes across both sets reproduce LHAPDF exactly — `0.00e0` relative
in every category**
(`alpha_s_matches_lhapdf_across_the_table_and_past_both_ends`, bounded at
`1e-14`). The ceiling answer is asserted separately as the *shape* claim it is: LHAPDF's own values at `Q` up to `10⁷` are the last
tabulated value to the bit, over both sets
(`above_the_alpha_s_table_lhapdf_freezes_rather_than_extrapolates`). NNPDF23
freezes at `0.07695485` from `10000` GeV up, which is what makes a 13 TeV
dynamical scale evaluable at all.

Regenerating the two oracle files changed **no existing point** — only the
`alphas` block was added.

### K5a.4 What is still refused, and why each one is undefined rather than awkward

- **A scale that is not a positive finite number.** `AlphaS_Ipol` asserts
  `q2 >= 0` and would return `+inf` at `Q = 0`; NaN would take an interpolating
  branch by way of a false comparison. Both are refused here.
- **A table with fewer than three knots in any subgrid that a query can select.**
  `alphasQ2` reads `alphas()[i+2]` when it takes a central slope at `i+1`, so a
  two-knot subgrid indexes off the end of its own `std::vector`. A one-knot
  piece is *not* refused: it can only arise from a repeated first knot, and the
  keyed-by-front-`Q²` insertion means the piece that follows replaces it, so
  nothing ever interpolates on it
  (`a_leading_repeated_scale_is_shadowed_rather_than_refused`).
- **A non-`ipol` `AlphaS_Type`**, unchanged: the knots are not the source there.
- **A non-positive tabulated value**, new: the below-table power law takes
  `log₁₀` of a ratio of them, so it would surface as a NaN rather than as a bad
  table.

`HadronicError::GridAlphaSBelowCollider` is deleted, and
`proton.rs`'s test of it is replaced by the opposite assertion — the same
dynamical card over the same 10 TeV table on a 13 TeV collider now compiles, and
the coupling it installs returns the frozen value at and above the collider
energy (`a_dynamical_scale_resolves_where_the_table_stops_below_the_collider`).

### K5a.5 For K5b and the capstone

- **The `αs` systematic is gone**, not reduced: there is no `1.7e-4` left to size
  against a σ agreement. A dynamical-scale cross section off an `lhapdf` set now
  reads the same coupling MadGraph read, event for event, to the printed digit.
- **The *density* grid has the same ceiling, it is untouched, and it is
  reached.** NNPDF23's parton densities also stop at `QMax = 10000`, and
  `PdfMember::xfx_q2` refuses an out-of-grid point rather than continuing —
  LHAPDF has an `Extrapolator` hierarchy this crate does not implement. With the
  coupling no longer stopping it, `vibegraph integrate` on the banked
  `pp → ℓ⁺ℓ⁻ j` card with its three `fixed_*_scale` switches turned off now gets
  further and **stops on the densities instead**, at `Q² = 1.1337e8` —
  `Q = 10647` GeV — against a grid reaching `1e8`. So it is measured rather than
  anticipated: a per-event `μF` on a 13 TeV collider does cross a 10 TeV grid.
  That is the next wall for the dynamical σ rows, it needs an extrapolator
  rather than a wider bound, and until one lands the stop arrives part-way
  through an integration instead of at setup. The measurement is kept running as
  `a_dynamical_scale_card_is_stopped_by_the_density_grid_and_not_by_the_coupling`,
  which asserts that the stop is the density grid's, that it is not the
  coupling's, and that the `Q²` it names is above the grid's own maximum — so a
  continuation landing later reads as a test that needs rewriting rather than as
  a quiet pass.
- **An available strengthening, deliberately not taken here.**
  `validate_scales`'s `banked_events_reproduce_aqcdup_from_the_computed_scale`
  steps over the four `lhapdf` runs because its second oracle is the
  beta-function solve. With a faithful grid reading those four could join it,
  which would tie the clustering-computed `μR` and the grid coupling together on
  `pp_to_jj` and `pp_to_llj_dyn` in one comparison instead of two. It is left
  alone because this session's gate was `validate_scales` **unmoved**.

## K5a2 — the density extrapolator

§K5a.5's wall is down. `PdfMember::try_xfx_q2` no longer refuses a point past
the grid; it continues it, the way LHAPDF does, and the dynamical `llj` card
that used to stop at `Q = 10647 GeV` now runs to a cross section.

### K5a2.1 Which continuation, and what pins each line of it

Neither fetched set's `.info` carries an `Extrapolator` key, so
`GridPDF::_loadExtrapolator`'s `info().get_entry("Extrapolator")` falls through
`PDFInfo::get_entry` → `PDFSet::get_entry` → `Config`, and `lhapdf.conf` says
`Extrapolator: continuation` — in 6.5.3's source tree and in the installed
6.5.6's `share/LHAPDF/lhapdf.conf` alike. `mkExtrapolator` (`Factories.cc:113`)
builds a `ContinuationExtrapolator`. The resolved name is *dumped into the
oracle* rather than left as a reading of a config file, so a build configured
differently fails the gate instead of quietly redefining the reference.

| element | what it is | source |
|---|---|---|
| the split | in range → interpolator, out of range → extrapolator, both edges inclusive | `GridPDF::_xfxQ2`, `KnotArray::inRangeX/inRangeQ2` |
| the edges it reads | `xs(0)`, `xs(1)`, `xs(nx−1)`, `q2s(0)`, `q2s(nq−2)`, `q2s(nq−1)` of the **flattened** array | `ContinuationExtrapolator::extrapolateXQ2` |
| above the Q² ceiling | straight line in `ln Q²` through the last two flattened Q² knots | same, branch 2 |
| below the x floor | straight line in `ln x` through the first two x knots | same, branch 1 |
| past both | the Q² line at each of the two lowest x knots, then the x line between them | same, branch 3 |
| in the value or its log | `ln y` when **both** endpoints exceed `1e-3`, `y` otherwise | `_extrapolateLinear` |
| below the Q² floor | `f(q2Min)·(Q²/Q²ₘᵢₙ)^γ`, `γ = anom·Q²/Q²ₘᵢₙ + 1 − Q²/Q²ₘᵢₙ` | same, branch 4 |
| that `anom` | `dlog f/dlog Q²` from a `1.01×` forward difference, floored at `−2.5`; `1` outright if `|f(q2Min)| < 1e-5` | same |
| `x` above the last knot | `RangeError` — the one direction with no continuation | same, final branch |

The flattened-edge row is the one that could have looked right while being
wrong. Our grids are stored per band; LHAPDF's `q2s(nq−2)` is the second-to-last
entry of *all* bands concatenated, which for a two-band set is the upper band's
penultimate knot and not the lower band's anything. `edges_of` builds the
concatenation explicitly for that reason, and `oracle_multigrid.json`'s
`above_q2max` probes are what would catch a per-band reading.

### K5a2.2 The oracle, and what it says

`gen_oracle.cpp` gained an `extrapolated` block: 1190 probes on NNPDF23 and 935
on NNPDF31, one category per out-of-range quadrant, every flavour at every
probe. Each record carries **two** values —
`Extrapolator::extrapolateXQ2` called directly (`xf_raw`, no positivity clamp on
top of it) and `PDF::xfxQ2` (`xf`, the number MadGraph sees). The comparison is
against `xf_raw`, so unlike the interpolated categories it needs no absolute
floor and an exact oracle zero demands an exact zero back.

| category | NNPDF23 | worst rel | NNPDF31 | worst rel |
|---|---|---|---|---|
| `above_q2max` | 560 | **2.37e-13** | 440 | 3.77e-14 |
| `below_xmin` | 168 | 0.00e0 | 132 | 1.83e-15 |
| `below_xmin_above_q2max` | 56 | 9.81e-16 | 44 | 5.15e-14 |
| `below_q2min` | 350 | 3.75e-16 | 275 | 6.80e-15 |
| `below_q2min_below_xmin` | 56 | 0.00e0 | 44 | 3.17e-14 |

Regenerating both files changed **no existing value**: the diff has zero deleted
lines and every pre-existing key compares equal.

### K5a2.3 The residual is one ulp, and the flat bound alone would not say so

Unlike §K5a's `αs` probes, these are not `0.00e0` across the board, and the
reason is conditioning rather than arithmetic. A straight line evaluated far
*outside* the pair of points that defines it is a difference of much larger
numbers: at `x = 0.7` four decades above the ceiling the gluon's continuation
carries a condition number of `1.9e3`, and one ulp on its two endpoints comes out
as `2.4e-13` on the result.

So the gate makes two statements rather than one. The flat bound
(`EXTRAP_REL_TOL = 1e-11`) is the coarse net — an order above the worst case,
eight orders below what a branch or knot-pair confusion produces. The sharp one
divides each point by its own condition number
`(|y_lo(1−t)| + |t·y_hi|)/|result|` and requires what is left to be one ulp:
**8.93e-16 worst on NNPDF23, 6.34e-16 on NNPDF31**, bounded at `1e-14`
(`the_upper_continuation_misses_lhapdf_by_one_ulp_of_its_own_conditioning`).
Reconstructing the endpoints from our own interpolator is sound because those
endpoints are independently gated against LHAPDF's, at `≤4e-16`.

Branch selection is pinned the same way, against LHAPDF's values rather than
against the source it was read from: for every `above_q2max` probe both candidate
continuations are built from our own edge readings, and LHAPDF's number must be
the one its endpoint values select. **184 NNPDF23 probes and 124 NNPDF31 probes
sit where the two candidates visibly differ**, with 205 and 180 on the linear
branch — so neither branch is covered only in name.

### K5a2.4 What is still refused, and why each one is undefined

- **`x` above the grid's last knot.** `ContinuationExtrapolator` raises
  `RangeError` there. For both fetched sets `xMax = 1`, so this coincides with an
  unphysical momentum fraction and is unreachable by a second route — but it is
  the extrapolator's refusal, not the physical-range check's, and a set whose
  grid stopped short of `x = 1` would separate them.
- **A point that is not a point**: non-finite `x` or `Q²`, `x ≤ 0` (the small-x
  continuation is a straight line in `ln x`, so `x = 0` sends LHAPDF's own
  reading to `±inf`), or `Q² < 0`.
- **`Q² = 0` is *not* refused.** The power law's exponent collapses to exactly
  `1` there and the reading is exactly zero, which is defined and is what LHAPDF
  returns.
- **A point in no subgrid while inside the overall extent** — a gap between
  bands, which a well-formed `lhagrid1` member does not have. `OutOfRange`
  survives as exactly that condition and nothing else.

### K5a2.5 The measured finding K5b must know

`ForcePositive` is applied by `PDF::xfxQ2` on top of everything, and this crate
does not apply it — a documented blind spot from the interpolation gates, where
it only ever bit "where the PDF is physically negligible". **Out of grid that is
no longer true.** On NNPDF31 (`ForcePositive: 2`) the clamp fires on **205 of
935** out-of-grid probes, and the largest continued value it replaces with `1e-10`
has magnitude **25.7** — a small-`x` continuation that ran negative, where
MadGraph would read `1e-10` and this crate reads `−25.7`.

It costs this sprint nothing. **No banked run reads NNPDF31**: all six
`pdlabel = lhapdf` runs (`dy13_default`, `dy13_mmll_60_120`, `pp_to_bb_fixed`,
`pp_to_llj_fixed`, `pp_to_llj_dyn`, `pp_to_jj`) carry `lhaid = 247000`, which is
NNPDF23 with no `ForcePositive` key and so the config default `0` — where the
clamp fires on **0 of 1190** probes. Every other banked run takes MadGraph's
built-in `nn23lo1`. NNPDF31 is in the tree only as the multi-subgrid *shape*
fixture for the interpolation gates. But it is a real gap against MadGraph for any
`ForcePositive`-carrying set, it is now measured rather than assumed
(`the_only_difference_from_madgraphs_own_value_is_the_positivity_clamp` asserts
`xf == clamp(xf_raw)` per level, so the relationship is checked data rather than
a claim), and closing it is a five-line change plus a re-reading of the
interpolation gates' `FORCE_POSITIVE_FLOOR` screen. Deliberately not taken here:
this session's gate was every existing `validate_pdf_grid` test green.

### K5a2.6 The guard test, rewritten

`a_dynamical_scale_card_is_stopped_by_the_density_grid_and_not_by_the_coupling`
is now
`a_dynamical_scale_card_runs_past_the_density_grid_and_past_the_coupling_table`.
The same command, the same budget, the same default seed: it used to stop
part-way through naming `Q² = 1.1337e8`, and it now completes and reports a cross
section. That is the end-to-end signal — the continuation is live on the path an
integration actually takes, not merely on a probe set. The scale the old stop
named is re-asserted directly in the same test (above the grid's own ceiling, and
a finite positive gluon density rather than a refusal), so the crossing stays
pinned even if the sampler's route moves.

The σ it reports at that budget (`--neval 2000 --niter 2`) is deliberately not
compared to anything: **the dynamical σ rows are K5b's**, and this says the run
finishes and produces a number, not that the number is right.

## K5b — the σ flips

Four cross sections were waiting on nothing but K4, and the session that
integrates them is the first to run the clustering *under the sampler* rather
than over MadGraph's own events. Two of the four agree and are enforced. The
other two are `5.5 %` low on every seed at every budget, and what separates the
pairs is neither their amplitudes nor their phase space: it is an input the
production integrand does not supply.

### K5b.1 The four partonic rows, both axes

Five seeds at the gate budget and the same five at four times it
(`probe_llj_parton_seed_stability`, `validate_sigma.rs`), against
`sigma_reference.json` — which this session re-extracted with all runs present
and which came back **byte-identical**, so the four σ were already banked and
the flip is a plan change and not a data change.

| row | MG σ ± Δ (pb) | mean rel, 1× | mean rel, 4× | worst \|rel\| | verdict |
|---|---|---|---|---|---|
| `uux_to_epemg` | 0.55507 ± 0.00116 | **+2.83e-3** | +2.34e-3 | 3.93e-3 | GATE, `rel_tol` 0.01 |
| `ddx_to_epemg` | 0.61770 ± 0.00125 | **+4.29e-3** | +4.37e-3 | 5.58e-3 | GATE, `rel_tol` 0.01 |
| `gu_to_epemu` | 0.10870 ± 0.00019 | **−5.55e-2** | −5.48e-2 | 5.71e-2 | ⚠️ Info |
| `gux_to_epemux` | 0.10884 ± 0.00022 | **−5.62e-2** | −5.57e-2 | 5.70e-2 | ⚠️ Info |

The budget is `60 000 × 8` on all four and the ladder rung is four times it. Two
things are worth reading off the table before the diagnosis.

- **Nothing here is sampling.** Quadrupling the budget moves every row by less
  than its own seed spread, and the two low rows do not move at all — they sit
  at `−5.5 %` on all ten runs. VEGAS's `1/σ²` combination makes an
  under-sampled region *confidently* wrong, which is why the ladder and not the
  sweep is what says so; the sweep alone would have shown five mutually
  consistent seeds either way.
- **The two enforced rows are inside their reference's own error.** `0.28 %` and
  `0.43 %` against banked Monte-Carlo errors of `0.21 %` and `0.20 %`. That is
  the term the pull is dominated by and the one no budget on this side can
  shrink, so `rel_tol` is set from the measured seed spread — `0.01`, about
  twice the worst rung — rather than from a pull that would keep growing as
  `err_vg` fell.

### K5b.2 The diagnosis: §K1.11 finding 2, met in production

The disagreement is the **integration channel**, and the sprint's own design note
named the mechanism before the engine existed.

§K1.11 finding 2: *the scale is not a pure function of (momenta, process)*.
Three channel dependencies exist — `filmap`'s `nqcd(this_config)` filter
(`cluster.f:360`), `checkbw`'s use of `this_config` (`cluster.f:419`), and the
`njetstore` memo (`reweight.f:985-1030`). K3 measured them on the bank and K4
reported, per run, how many events the choice moves at all (§K4.3).

`hadronic::Channels` carries a `default_config` of `1`, and
`EventScaleSource::scales` reads it on **every** point, because the sampled
channel is not plumbed from the multichannel map to the scale prescription:
`FixedBeamIntegrand::matrix_element` and `ProtonIntegrand`'s inner map both take
momenta and nothing else. So the production integrand computes MadGraph's
channel-dependent scale in a channel it did not sample.

**The rows split exactly along that line**, and the split is K4's own table read
against this session's:

| row | banked events needing a channel other than the first | σ deviation |
|---|---|---|
| `uux_to_epemg` | 0 of 10 000 | +0.28 % |
| `ddx_to_epemg` | 0 of 10 000 | +0.43 % |
| `gu_to_epemu` | **7204** of 10 000 | **−5.5 %** |
| `gux_to_epemux` | **7231** of 10 000 | **−5.6 %** |

Every other explanation is excluded by a gate that is already green on the same
process. The **amplitudes** cells enforce `4.07e-14` and `3.95e-14` on
`gu_to_epemu` and `gux_to_epemux` — per-diagram, per-helicity, per-flow — so it
is not the matrix element. The **diagrams** cells are `4/4`. `uux_to_epemg` and
`gu_to_epemu` are crossings of one another with the same channel maps, the same
cut compiler and the same multichannel combiner, so it is not the phase space.
And `validate_scales` reproduces all four runs' banked `SCALUP` / `<rscale>` /
`<pdfrwt>` inside their printing budgets — *when it is allowed to search for the
channel*, which is precisely the input production lacks.

Note also that channel *numbering* is not shared with MadGraph and cannot be
(§K4.1): the forests were checked as a bijection, not as a list. So "channel 1"
on this side is an arbitrary member of the set, and the default was never more
than a placeholder — K4 said as much and reported the counts that would price it.

**What the fix is.** Thread the sampled channel index through to
`EventScaleSource::scales`, and map the multichannel map's channel onto the
derived config it came from. Both sides are per-diagram over the same diagram
list, so the mapping exists; it is not the identity in general, because
`derive_channels` drops the diagrams the four-point filter removes (§K1.11
finding 3), and it has to be constructed rather than assumed. That is an
integrand-contract change touching every process, and it is **not** this
session's scope: it is filed rather than improvised, and the two rows land
informational meanwhile — a live, known-wrong comparison that turns the fix into
an instant end-to-end signal.

### K5b.3 The size of it, priced in the coupling rather than in a count

A count of moved events does not say what a cross section pays, because σ reads
the scale only through `αs(μR)`. So the diagnosis is closed by an instrument that
prices it directly: `probe_first_channel_cost_in_alpha_s`
(`validate_scales.rs`) evaluates `αs` at the **channel-1** scale on every banked
event of every clustered run whose coupling comes from the evolution, and divides
by `AQCDUP` — MadGraph's own coupling for that event, seven printed digits, read
off the record. The mean ratio is the multiplicative bias a σ linear in `αs`
inherits.

| run | events on another channel | mean `αs(ch 1)/AQCDUP − 1` | measured σ deviation |
|---|---|---|---|
| `uux_to_epemg` | 0 | −2.1e-9 | +2.8e-3 |
| `ddx_to_epemg` | 0 | −1.6e-9 | +4.3e-3 |
| `gu_to_epemu` | 7204 | **−5.540e-2** | **−5.55e-2** |
| `gux_to_epemux` | 7231 | **−5.557e-2** | **−5.62e-2** |
| `pp_to_llj` | 5572 | −4.667e-2 | (proton, see below) |
| `uux_to_uux`, `gg_to_gg`, `gg_to_ttx`, `pp_to_ll`, `pp_to_bb` | 0 | ≤ 1.6e-7 | enforced, unmoved |

**The two low rows' σ deficits are their `αs` deficits to two digits.** Nothing
else in the chain has to be invoked, and nothing else could produce a number that
close by coincidence. The enforced QCD rows sit at `1.6e-7` — the printed field's
own rounding — which is why none of them moved: for them channel 1 *is* every
channel as far as the scale is concerned.

The rows that agree also gain a statement they did not have: their `+0.28%` and
`+0.43%` are **not** the scale. At `2e-9` the channel default costs them nothing,
so what is left is the ordinary distance between two Monte-Carlo estimates, one
of which carries a `0.2%` error of its own.

### K5b.4 σ(p p → ℓ⁺ℓ⁻ j) at the dynamical scale

Five seeds per rung, the fixed-scale row of the same process left enforced and
untouched, so the scale is the only moving part in the chain
(`probe_llj_dyn_budget_ladder`). MadGraph: **415.42 ± 1.36 pb**.

| neval | σ ± Δ (pb) | χ²/dof | rel | pull |
|---|---|---|---|---|
| 75 000 | 399.505 ± 0.349 | 4.89 | −3.83 % | −11.33 |
| 150 000 | 401.263 ± 0.242 | 0.50 | −3.41 % | −10.24 |
| 300 000 | 402.379 ± 0.169 | 1.57 | −3.14 % | −9.51 |
| 600 000 | 402.769 ± 0.119 | 0.09 | **−3.05 %** | −9.26 |

The estimator rises with the budget and the increments halve — `+1.76`, `+1.12`,
`+0.39` — which is the fixed-scale row's own approach-from-below, and at the last
rung the five seeds agree at `χ²/dof 0.09` with a `0.03%` spread. So this is
converged and seed-stable at about `403 pb`, and the `393.71` the K5a2 guard test
printed at `--neval 2000 --niter 2` was the bottom of that same ladder rather than
a second effect.

The banked layer runs three of those seeds at `300 000` and writes the cell:
**`402.411 ± 0.218 pb`, `χ²/dof 0.26`, rel `−3.13%`, pull `−9.44`** — the ladder's
own `300k` rung, reached independently.

**`−3.05%` is the partonic `−5.5%` diluted by the groups that do not carry it.**
The gluon-initiated share of `p p → ℓ⁺ℓ⁻ j` at 13 TeV is a little over half the
cross section, and half of `5.5%` is `3%`; the banked-event mean for `pp_to_llj`
in §K5b.3 is `−4.7%` for the same reason, weighted differently. The row therefore
lands **⚠️ Info** with the finding recorded, not GATE — `sigma_llj_fixed_scale_vs_mg`
stays enforced and unmoved, which is what makes the attribution to the scale
sound.

### K5b.5 The `samples` cells, and what they provably cannot see

All four partonic rows' `samples` cells unblock with their `integrals` cells and
follow the precedent exactly — the same `validate_samples` row list, the same
three generation seeds, the same 20 000 events a seed against MadGraph's own
10 000, the same weighted-ECDF KS on the kinematics and χ² on `SPINUP`. They are
the only fixed-beam samples rows whose accept/reject draw runs over an integrand
with a per-event coupling; every other one draws on a constant.

| row | min KS p (worst observable) | min `SPINUP` χ² p |
|---|---|---|
| `uux_to_epemg` | 3.7e-2 `phi(e+)` | 0.48 |
| `ddx_to_epemg` | 5.8e-3 `m(e-,g)` | 0.87 |
| `gu_to_epemu` | 2.0e-2 `cs_cos(ll)` | 8.0e-2 |
| `gux_to_epemux` | 9.4e-3 `m(e-,u~)` | 0.31 |

All four gate, against a `1e-4` floor. **Including the two whose cross section is
`5.5%` wrong**, and that is stated on the cells rather than left to be noticed:
these are shape statistics, the defect is a nearly uniform multiplicative factor
over the fiducial region, and a normalisation that moves σ without moving a shape
is exactly what a KS test is blind to. The passing cells are not evidence about
the σ deficit, and the σ cells are where it lives.

`the_llj_parton_rows_take_a_per_event_cluster_scale` replaces the capability check
that used to assert the refusal: it requires, per row, that the prescription
resolved, that it did **not** collapse to a constant, and that it was handed
channel forests. A collapse to `m_Z` — where these rows' lepton pair sits — would
leave both the σ and the sample comparisons close enough to look fine while
measuring nothing about the clustering, and that is the assertion which fails.

### K5b.6 Where this leaves the sprint

**Landed.** Two σ rows GATE (`uux_to_epemg`, `ddx_to_epemg` at `rel_tol` 0.01),
four `samples` rows GATE, two σ rows ⚠️ Info with a diagnosed and priced
disagreement (`gu_to_epemu`, `gux_to_epemux`), and `pp_to_llj_dyn`'s `integrals`
cell ⚠️ Info at `−3.05%`. The census moves `77 → 86` measured cells, `76 → 82` ✅,
`1 → 4` ⚠️, `22 → 12` ⛔. `sigma_gate_matches_madgraph` asserts 15 processes where
it asserted 13. **Every other row of the report is byte-identical to the
pre-session one** — the two rendered tables differ on exactly the five rows above
and nowhere else, which is a stronger statement than "unmoved to the printed
digit".

Both work-area states are green. With the four unbundled runs held out, the
report renders `pp_to_llj_dyn`'s `integrals` cell as **⏳ awaiting the bundle**
and the census reads `84` measured (`81 ✅, 3 ⚠️, 6 ⏳`) against `86` (`82, 4, 4`)
with them present — the two unbundled `integrals` cells moving to `⏳` and nothing
else. The four llj partonic rows stay `✅` in both, which is right: they are in
`refdata-3`, so a fetching checkout has them.

`pp_to_llj_dyn`'s `samples` cell moves `blocked → uncovered`: nothing refuses it
any more, and nothing measures it either — `validate_samples_proton` integrates
and generates the fixed-scale card only, so the dynamical one needs an artifact
and a generation pass of its own. Leaving it `blocked` on `kt-clustering` would
have been a false statement once the cell above it started producing a number.

**Owed, and not by this session.** The integration channel has to reach the scale
prescription. Until it does:

- the two Info rows stay Info, and so does `pp_to_llj_dyn`;
- **the capstone will meet the same wall.** `p p → j j` at the default dynamical
  scale is `pp_to_jj`, and the whole point of it is a 2 → 2 core of massless
  coloured legs where no closed form collapses the prescription — which is to say
  a process with many channels and no reason for channel 1 to be the right one.
  Its 9-event tie-break exception is a separate and much smaller thing. Session C
  should not be attempted before the channel is plumbed through.

## K6 — the sampled channel reaches the scale

K5b's two Info rows and `pp_to_llj_dyn` were waiting on one input: the cluster
scale is a function of the event **and** of the integration channel, and the
integrand named channel 1 on every point. This session threads the channel the
sampler actually drew through to `EventScaleSource::scales`, and the three rows
move from `−5.5 %`, `−5.6 %` and `−3.05 %` to `+1.07 %`, `+0.97 %` and `−0.68 %`.
All three are now enforced. Every other σ row of the report is **bit-for-bit
identical**, and there is a reason stronger than a count for why.

### K6.1 What was threaded

`EventScaleSource::scales` takes a `SampledChannel` — the channel set a draw
belongs to (the flavour group), and that channel's diagram inside it. Both
integrands supply it from the draw they made:

- `MultiChannel::sample_from` reports the drawn index alongside the point, and
  `PhaseSpaceMap::sample` delegates to it, so no draw and no weight moved by a
  bit;
- `FixedBeamIntegrand`'s `matrix_element`, `apply_scale`, `event_scales`,
  `select_event`, `value`, `value_in_channel` and `event_in_channel` all carry
  the channel, as does the setup-time scale probe;
- `ProtonIntegrand::shape` and `event_scales` take it from `channel_ids[j]`, the
  `(group, diagram)` pair the pooled sampler was already keyed on;
- `MultiChannel::adapt_alphas`'s survey integrand became
  `Fn(&[momenta], usize)`, so the α-adaptation surveys the integrand the
  integration will run rather than a channel-1 shadow of it;
- `vibegraph generate` writes each event's `SCALUP`/`AQCDUP` from the same
  channel the point was drawn in.

### K6.2 The map is not the identity, and it is derived

`derive_channels` drops a diagram whose largest vertex exceeds the set's minimum
(§K1.11 finding 3), so the sampler's channels — one per *diagram* — and
`configs.inc`'s channels are two numberings. `DerivedChannels` now carries
`config_of_diagram`, the inverse of `diagram_of` over the whole diagram slice:
`None` for a dropped diagram, whose region MadGraph covers from the surviving
channels and whose draw therefore takes the set's default rather than indexing
one channel too far. `RunningCouplingReport::unmapped_channels` reports how many
such channels a run has, so a run that has any says so.

`g g → g g` is the hermetic pin — 4 diagrams, 3 channels, nothing in the bank
exercising the filter:

- `the_channel_to_config_map_is_not_the_identity` (`coupling::cluster::configs`)
  identifies the unmapped diagram **by the four-gluon vertex that gets it
  dropped**, not by its position, asserts the table
  `[None, Some(1), Some(2), Some(3)]`, asserts both directions of the map, and
  asserts each entry's forest leg-set masks — so a reorder that carried both
  sides along together, leaving the indices alone, still fails.
- `a_sampled_channel_names_the_integration_channel_of_its_own_diagram`
  (`hadronic`) pins the *wiring* on the same process: the compiled prescription
  reports `channels = 3`, `unmapped_channels = 1`, `config_of_channel` reads
  `[1, 1, 2, 3]`, and the installed sampler's channel count equals the set's
  diagram count. `use_multichannel` asserts that last equality at run time too,
  because a caller that sampled one diagram slice and clustered against another
  would name the wrong channel on every point in perfect silence.

### K6.3 Per flavour group, not per process

`compile_scale_source` takes one `(representative amplitude, diagrams)` pair per
subprocess a sampling channel can be drawn from and builds one `Channels` each.
A fixed-beam run passes one. A hadronic run passes one per flavour group — and
that is a correction, not just plumbing: `ProtonIntegrand` derived its forests
from **group 0's** diagrams and used them for every group, while its sampler
pools channels across all of them. `g u → ℓ⁺ℓ⁻ u` and `u ū → ℓ⁺ℓ⁻ g` do not
share a merge graph, and `validate_scales`'s banked replay has always keyed its
forests on the event's own external flavours, which is the same statement made
against MadGraph. `use_run_card_scales` now asserts that every pooled
`(group, diagram)` names a forest in its own group's set.

### K6.4 The four partonic rows

Five seeds at the gate budget and the same five at four times it
(`probe_llj_parton_seed_stability`), against `sigma_reference.json`.

| row | events on another channel | mean rel 1× (was) | mean rel 4× | worst \|rel\| | verdict |
|---|---|---|---|---|---|
| `uux_to_epemg` | 0 of 10 000 | **+2.83e-3** (+2.83e-3) | +2.34e-3 | 3.93e-3 | GATE `rel_tol` 0.01 |
| `ddx_to_epemg` | 0 of 10 000 | **+4.29e-3** (+4.29e-3) | +4.37e-3 | 5.58e-3 | GATE `rel_tol` 0.01 |
| `gu_to_epemu` | 7204 | **+1.07e-2** (−5.55e-2) | +1.13e-2 | 1.20e-2 | GATE `rel_tol` 0.02 |
| `gux_to_epemux` | 7231 | **+9.74e-3** (−5.62e-2) | +1.03e-2 | 1.12e-2 | GATE `rel_tol` 0.02 |

Their four `samples` cells stay GATE. The two annihilation rows' are unchanged
(`3.74e-2` / `5.85e-3` minimum KS `p`, same worst observable, same `χ²`); the two
gluon rows' moved with the integrand the accept/reject draw runs over —
`gu_to_epemu` from `2.01e-2` to `1.32e-2`, both on `cs_cos(ll)`, and
`gux_to_epemux` from `9.40e-3` on `m(e−,u~)` to `9.37e-3` on `m(e⁺,u~)` — against
a `1e-4` floor. They remain blind to the normalisation by construction, which is
why the σ cells are where §K6.5 lives.

**The two annihilation rows are unchanged to the last bit, and the reason is
sharper than their event count.** `the_sampled_channel_reaches_the_cluster_scale`
evaluates `μR` in *every* sampling channel at the same drawn point and reports
the spread:

| row | `μR` spread over its channels |
|---|---|
| `uux_to_epemg`, `ddx_to_epemg` | **0.000e0** |
| `gu_to_epemu`, `gux_to_epemux` | **9.93e-1** |

So on the annihilation rows the cluster scale is not merely *usually* the same in
every channel — it is identically the same, and no threading could have moved
them. On the gluon-beam rows it differs by a factor of two between channels,
which is why they carried the whole of K5b's deficit and why they are the rows
that could pay for the partition below. That test is also this session's guard:
it fails if the channel ever stops reaching the prescription, which is exactly
the change whose *absence* every other number here would silently survive.

### K6.5 The residual is the channel partition, and it has a negative control

`gu_to_epemu` and `gux_to_epemux` do not land at zero; they land at `+1.07 %` and
`+0.97 %`, seed-stable, and `+1.13 %` / `+1.03 %` at four times the budget — a
bias, not sampling. It is not a defect of the clustering, and it is not a
tolerance question either. It is a property the cross section acquired the moment
the scale started reading the integration channel:

> With a channel-dependent scale, the channel-split estimator
> `σ = Σⱼ ∫ dΦ f(p, j)·αⱼgⱼ(p)/g(p)` is **no longer independent of `αⱼ`**. The
> selection weights decide which scale a region of phase space is evaluated at,
> not merely how often it is visited. σ is therefore defined only up to the
> channel partition.

`probe_channel_partition_moves_sigma` measures it: the same rows integrated at the
converged `αⱼ` and at uniform `αⱼ`, one seed, everything else held.

| row | adapted α, rel to MG | uniform α, rel to MG | partition gap | Monte Carlo |
|---|---|---|---|---|
| `uux_to_epemg` | +3.09e-3 | +4.14e-3 | +1.05e-3 | 1.6e-3 |
| `ddx_to_epemg` | +4.73e-3 | +6.59e-3 | +1.86e-3 | 1.5e-3 |
| `gu_to_epemu` | +1.08e-2 | **−4.24e-3** | **−1.48e-2** | 1.6e-3 |
| `gux_to_epemux` | +9.75e-3 | **−5.69e-3** | **−1.53e-2** | 1.6e-3 |

Three things are worth reading off it.

- **The two rows whose scale is channel-independent are the negative control.**
  Their partition gap is inside and at their own Monte-Carlo error, which is what
  a partition-invariant integral looks like. The effect is 9σ on the other two.
- **MadGraph's own σ lies *inside* the interval this crate's two partitions
  span.** On `gu_to_epemu` the adapted partition is `+1.08 %` and the uniform one
  `−0.42 %` relative to the same reference. There is no single number to agree
  with.
- **MadEvent's partition is a third one.** Single-diagram enhancement weights the
  integrand of channel `c` by `AMP2_c/Σ AMP2`, which is not reachable from either
  of ours by any choice of `αⱼ` — it is a function of the point, not a constant.

So `rel_tol` `0.02` on these two rows is the **algorithm's own ambiguity with
headroom** over the worst measured `1.20e-2`, in exactly the sense AGENTS.md's
tolerance rule means: it is not the reference's `0.18 %` error, and it is not a
bound fitted around one number — the `−5.5 %` this row read before the channel
reached the scale is far outside it.

**And their pull is reported rather than asserted** — the same judgment
`pp_to_llj_dyn` gets below, and the only two places in the σ gate where it is
made. A pull bounds a disagreement only while the residual is a fluctuation; a
systematic of fixed size drives it to infinity as `err_vg` falls, so a budget
increase would eventually fail a row that had not moved. `PULL_REPORTED_NOT_ASSERTED`
carries the list and the reason, `rel`, the five-seed sweep and `χ²/dof` are the
criteria, and the gate asserts that no row on the list is ungated — an exemption
on a row that asserts nothing would hide that it asserts nothing.

### K6.6 σ(p p → ℓ⁺ℓ⁻ j) at the dynamical scale

Five seeds per rung, the fixed-scale twin left enforced and untouched
(`probe_llj_dyn_budget_ladder`). MadGraph: **415.42 ± 1.36 pb**.

| neval | σ ± Δ (pb) | χ²/dof | rel | was (channel 1) |
|---|---|---|---|---|
| 75 000 | 409.554 ± 0.357 | 6.13 | −1.41 % | −3.83 % |
| 150 000 | 411.393 ± 0.247 | 0.80 | −0.97 % | −3.41 % |
| 300 000 | 412.529 ± 0.172 | 0.82 | **−0.70 %** | −3.14 % |
| 600 000 | 412.950 ± 0.121 | 0.09 | −0.59 % | −3.05 % |

The estimator still rises with the budget and the increments still halve
(`+1.84`, `+1.14`, `+0.42`), which is the fixed-scale row's own approach from
below, and the last rung's five seeds agree at `χ²/dof 0.09` with a `0.05 %`
spread — converged and seed-stable at about `413 pb`. The
banked layer runs three seeds at `300 000` and writes the cell: **412.585 ±
0.224 pb, χ²/dof 0.22, rel −0.68 %**.

The row is enforced at `LLJ_DYN_MAX_REL = 0.015` — the partonic partition band of
§K6.5 diluted by the flavour groups that do not carry it, with headroom, and
above MadGraph's own `0.33 %` on this run. **The pull is reported and not
asserted**: the residual is a systematic of about `0.6 %`, so its pull grows
without bound as this side's budget rises while the disagreement stays put, and
asserting it would be asserting a precision the number does not have.
`sigma_llj_fixed_scale_vs_mg` stays enforced at `0.005` and unmoved, which is
what keeps the attribution to the scale sound.

### K6.7 The instruments, and what each can and cannot say

- `probe_first_channel_cost_in_alpha_s` (`validate_scales`) is **unchanged by
  construction and was re-run to confirm it**: it replays MadGraph's banked
  events at *channel 1* and divides by `AQCDUP`, so it is a property of the bank
  and of one channel, not of any integrand. It still reads `−5.540e-2` /
  `−5.557e-2` on the two gluon rows and `≤1.6e-7` on every enforced QCD row. What
  it prices is what the old default cost, and that is all it can ever price.
- `probe_sampled_channel_cost_in_alpha_s` (`validate_sigma`) is its
  production-side counterpart and is what actually moved: the integrand draws its
  own points in its own channels and reports the σ-weighted `⟨αs⟩` against
  `⟨AQCDUP⟩` over the run's banked unweighted events, which are distributed as
  MadGraph's own cross section.

  | row | σ-weighted `⟨αs⟩` | MG `⟨AQCDUP⟩` | rel |
  |---|---|---|---|
  | `uux_to_epemg` | 0.1015708 | 0.1014733 | +9.61e-4 |
  | `ddx_to_epemg` | 0.1019197 | 0.1017793 | +1.38e-3 |
  | `gu_to_epemu` | 0.1016983 | 0.0996450 | +2.06e-2 |
  | `gux_to_epemux` | 0.1016959 | 0.0996616 | +2.04e-2 |

  Per-event agreement is neither claimed nor available — MadGraph's sampled
  channel differs event by event and an LHE record carries none — so the mean is
  the statement, and it is the multiplicative factor a σ linear in `αs` inherits.
  The two gluon rows' residual `+2 %` in the coupling is the same partition
  effect §K6.5 measures in σ, seen one step earlier in the chain.

### K6.8 What this leaves

**Landed.** Three σ rows Info → GATE (`gu_to_epemu`, `gux_to_epemux` at
`rel_tol` 0.02; `pp_to_llj_dyn` at 0.015), the `ProtonIntegrand`'s per-group
forest correction, and two guards that fail if the channel stops reaching the
scale or if either numbering reorders. `sigma_gate_matches_madgraph` asserts 17
processes where it asserted 15. The census moves `82 → 85` ✅ and `4 → 1` ⚠️ over
the same 86 measured cells.

**The report's two rendered tables differ on exactly five cells** — the three
`integrals` cells that flipped, and the two `samples` cells whose accept/reject
draw follows the integrand — and are identical elsewhere in every compared field,
which is a stronger statement than "unmoved to the printed digit". Both work-area
states are green: with the four unbundled runs held out the census reads `84`
measured (`83 ✅, 1 ⚠️, 6 ⏳`) against `86` (`85, 1, 4`) with them present, the two
unbundled `integrals` cells moving to ⏳ and nothing else. The four held-out runs
were restored and verified byte-identical over all 2867 files.

**Owed, and filed rather than improvised.** The comparison can be made
partition-free, and the reading says how: take the scale's channel from
`AMP2_c(p)` at each point instead of from the phase-space draw. That is what
MadEvent's `iconfig` distribution *is* — single-diagram enhancement — so it would
reproduce MadGraph's own effective channel distribution and stop the scale
riding on this crate's sampler at all. It is a second per-point draw inside the
integrand with its own reproducibility and artifact consequences, which is why it
is a design decision and not a session's improvisation. Until it lands, `σ` at
`dynamical_scale_choice = -1` agrees with MadGraph to the partition ambiguity and
no better, and the three cells say so.

**For the capstone.** `p p → j j` is no longer blocked on the channel: it is a
many-channel process at the default dynamical scale, which is exactly the
configuration K6 supplies. It should expect the partition residual of §K6.5 to
be at its largest there — every leg coloured, every channel's merge graph
different — so session C should measure the partition gap on `pp_to_jj` before
choosing its tolerance, rather than inheriting `0.02` from the partonic rows.

## C — capstone: `p p > j j` at MadGraph's default dynamical scale

The capstone ran, and what it found is not a tolerance question. `p p > j j` is
the first banked row whose process card **repeats a multiparticle label in the
final state**, and that is the one case this crate's alias expansion and
MadGraph's do not treat alike. σ is high by `36 %`. The cause is counted exactly,
against MadGraph's own `leshouche.inc` and against its own events, and collapsing
the surplus at enumeration — with the library untouched — puts the row inside the
reference's own Monte-Carlo error. **Both cells land informational with the
measurement recorded**, and the fix is named rather than improvised: it is one
line in the enumeration, and it belongs to a session that can re-run the whole
banked layer behind it.

### C.1 The surplus, counted against `leshouche.inc`

`generate_sets_inner` (`diagrams/mod.rs`) deduplicates concrete assignments on
`(sorted initial, final state as written)` — the initial state as an unordered
pair, the final state **in the order the card's slots were filled**. MadGraph
keeps one representative per *unordered* outgoing assignment. So `g u > g u` and
`g u > u g` are two subprocesses here and one there, and both are then summed over
the same labelled `dΦ₂`, whose polar angle runs over the whole sphere: the second
is the first relabelled, and its term is added twice.

`jj_subprocesses_are_madgraphs_own_plus_the_outgoing_permutations`
(`validate_hadronic`, enforced) states it as a set comparison against the banked
run's own `leshouche.inc` — the file the colour-flow dictionary is already checked
against, and the only place a run declares which concrete assignments its cross
section sums over. Read from the run, not from a committed list. No Monte Carlo,
no tolerance:

| | |
|---|---|
| MadGraph's assignments | **65** (52 with unequal outgoing flavours, 13 with equal) |
| this side's | **117** = 65 + 52 surplus |
| surplus that is an outgoing swap of a MadGraph one | **52 of 52** |
| MadGraph assignments this side does not enumerate | **0** |

`13 + 2 × 52 = 117` is the whole of it, and the identity is what makes the reading
a derivation rather than a fit: the surplus is exactly one extra copy per
assignment whose two outgoing flavours differ, and nothing else.

**The premise is measured too, not read off `dΦ₂`.** If the two orderings were
distinct subprocesses for MadGraph, its sample would carry both, and the region
the second covers would be missing from the first. Over the banked 10 000 events:
**77** distinct emitted flavour assignments (65 × the two beam orderings, less the
ones with equal beams), **0** of them with their outgoing swap also emitted, and
inside the unequal-outgoing ones the two outgoing legs are found more-forward-first
**1808** times and more-forward-second **1814** — the even split a single
representative covering the whole sphere predicts.

**And the collapse reproduces MadGraph's set exactly**, which is the part that
sizes the fix: filtering the enumerated `DiagramSet`s on `(sorted incoming,
sorted outgoing)` leaves **65** assignments, **all 65 in MadGraph's own outgoing
order**. So repairing the count does not also need a rule for which representative
to keep — the survivor is already the one MadGraph writes.

**Negative control.** A card whose final-state slots draw on *disjoint* alias sets
cannot produce two ordered assignments with the same multiset, so the same collapse
must change nothing there. `p p > l+ l- j`: **212** enumerated sets, **212**
surviving, 24 of them carrying diagrams. Without it the surplus would read as a
blanket property of alias expansion rather than as the repeated label it is. By the
same criterion `p p > j j` is the **only** process in the manifest whose final-state
slots draw on intersecting alias sets — every other row's card spells its final
state with concrete particles or with pairwise-disjoint aliases (`l+`, `l-`, `j`).

### C.2 What it costs the cross section, and the bracket the bank makes for it

If the surplus is one extra copy of every unequal-outgoing assignment, then

```text
σ(as enumerated) = σ(MG) + σ(MG restricted to unequal outgoing flavours)
```

and the banked run brackets the second term with no fit at all: its five
subprocess directories carry their own cross sections, and `gg_qq` and `gq_gq` are
entirely unequal-outgoing while `gg_gg` and `qq_gg` are entirely equal-outgoing.
`qq_qq` mixes the two and is the only unresolved part, so it sets the width.

| directory | σ (pb) | outgoing |
|---|---|---|
| `P1_gg_gg` | 4.307376e8 | equal |
| `P1_gg_qq` | 1.136845e7 | unequal |
| `P1_gq_gq` | 2.163323e8 | unequal |
| `P1_qq_gg` | 2.937100e5 | equal |
| `P1_qq_qq` | 2.012659e7 | mixed |
| total | 6.788587e8 | (`results.dat`: 6.788500e8) |

**Predicted `σ(as enumerated)/σ(MG) ∈ [1.3354, 1.3651]`. Measured 1.3628**
(`probe_jj_outgoing_permutation_costs_the_cross_section`, 300 000 × 10, one seed).
A defect of any other shape would have had no reason to land in that interval.

The same probe's second arm is the falsifier:

| arm | subprocesses / groups / channels | σ (pb) | σ/σ_MG | pull |
|---|---|---|---|---|
| as enumerated | 117 / 11 / 25 | 9.251119e8 ± 5.824e5 | 1.3628 | +155.51 |
| permutations collapsed | 65 / 8 / 19 | 6.798582e8 ± 4.326e5 | **1.0015** | **+0.66** |

`+0.15 %` against a reference whose own Monte-Carlo error is `0.22 %`.

### C.3 The channel partition is not the residual here — §K6.8's expectation, corrected

§K6.8 handed this session the instruction to measure the channel-partition
ambiguity on `pp_to_jj` before choosing a tolerance, on the reading that "every leg
coloured and every channel's merge graph different" would make it largest here.
**Measured, it is the smallest of any clustered row in the set**, and the reason is
structural rather than accidental.

`probe_jj_channel_partition`: the same row at the converged `αⱼ` and at uniform
`αⱼ` (`n_adapt_iter = 0`), everything else held, one seed, 300 000 × 10.

| arm | adapted α | uniform α | partition gap | Monte Carlo | MG rel adapted | MG rel uniform |
|---|---|---|---|---|---|---|
| `j j`, as enumerated | 9.251119e8 ± 5.82e5 | 9.252758e8 ± 6.53e5 | **+1.77e-4** | 9.5e-4 | +3.628e-1 | +3.630e-1 |
| `j j`, permutations collapsed | 6.798582e8 ± 4.33e5 | 6.805569e8 ± 4.90e5 | **+1.03e-3** | 9.6e-4 | +1.485e-3 | +2.514e-3 |
| `pp_to_llj_fixed` (control) | 4.230616e2 ± 3.99e-1 | 4.230056e2 ± 4.63e-1 | −1.32e-4 | 1.4e-3 | −1.836e-3 | −1.969e-3 |

Both `j j` arms sit at their own Monte-Carlo error — `1.07 σ` on the collapsed one
— against the `1.5e-2` at `9 σ` that `gu_to_epemu` and `gux_to_epemux` carry
(§K6.5). The control is a fixed-scale row on the same hadronic path with the same
instrument, so it says the uniform-`α` arm is not simply noisier in a way that
would hide a gap.

**Why it is small, and why that is not luck.** The cluster scale depends on the
integration channel only through *which merge sequence* the channel's forest
admits. A `2 → 2` final state has no merge to choose: the clustering's terminal
"`2 → 2` core" is the event itself, so `jlast`/`jcentral` — and therefore `μR` and
both `μF` — are functions of the momenta alone. That is the same structural
statement K6 measured as an exactly-zero `μR` spread on the two annihilation llj
rows, and `validate_scales` records it independently for this run: `pp_to_jj` is
one of the twenty-one replayed runs needing **0** events on a channel other than
the first (§K4.3).

**Consequence for whoever repairs C.1**: this row's tolerance is set by the
reference's own `0.22 %` and by the seed spread, not by a partition band. It does
not inherit `0.02`, and it does not need `PULL_REPORTED_NOT_ASSERTED` either — the
residual it is left with is Monte Carlo, not a systematic of fixed size.

### C.4 The σ cell, both axes

`probe_jj_budget_ladder`, five seeds a rung, against MadGraph's
`6.788500e8 ± 1.4726e6 pb` — a reference whose own Monte-Carlo error is `0.22 %`.

| arm | neval | σ ± Δ (pb) | χ²/dof | rel |
|---|---|---|---|---|
| as enumerated | 300 000 | 9.244620e8 ± 2.604e5 | 3.41 | **+36.18 %** |
| permutations collapsed | 75 000 | 6.799571e8 ± 3.900e5 | 1.41 | +0.16 % |
| permutations collapsed | 150 000 | 6.802836e8 ± 2.753e5 | 0.61 | +0.21 % |
| permutations collapsed | 300 000 | 6.803185e8 ± 1.943e5 | 1.26 | +0.22 % |
| permutations collapsed | 600 000 | 6.806965e8 ± 1.375e5 | 1.25 | +0.27 % |

The collapsed arm climbs by `0.11 %` over an eightfold budget — increments
`+0.05 %`, `+0.01 %`, `+0.05 %` — against the `0.82 %` `pp_to_llj_dyn` climbs over
the same range, and every rung sits inside `1.25 σ` of the reference. That is half
the reference's own error and it is not resolved into an asymptote, so the honest
statement is that the row is converged **at the scale the comparison is made at**
rather than demonstrably asymptotic; a session that gates it should keep the ladder
and say so. Its five seeds at `300 000` span `0.18 %` and scatter at `χ²/dof 1.26`,
so what is left there is Monte Carlo.

**The enumerated arm's `χ²/dof 3.41` is worth recording on its own.** Its five
seeds scatter by about `1.8×` their own quoted errors where the collapsed arm's
stay near one. The surplus does not only double a term: it puts six extra sampling
channels into the mixture that are relabelled images of channels already in it, and
a multichannel whose members are near-degenerate is the ill-conditioned case the
`αⱼ` reallocation cannot separate (note 27 §B3.2's failure mode). So the defect
costs variance as well as normalisation. Observed rather than derived — nothing
here separates the degeneracy from the three extra flavour groups.

The banked layer writes the cell from three seeds at `300 000 × 10` on the
production path (`sigma_jj_dynamical_scale_vs_mg`, mode `info`):
**`9.246158e8 ± 3.363e5 pb`, `χ²/dof 6.53`, `rel +36.20 %`, `pull +162.70`**. The
`χ²/dof` is part of the record rather than an embarrassment: it is the same
over-dispersion the five-seed ladder reads as `3.41`, and it goes with the arm
that carries the surplus.

### C.5 The `samples` cell

The event side, through the shipped binary: one `integrate` at `300 000 × 8` off
the banked cards, then 20 000 events at each of three seeds, read back and compared
column by column against MadGraph's banked 10 000
(`generated_dijet_events_agree_with_madgraphs_banked_ones`, `validate_samples_proton`,
mode `info`). The `p`-floor is `1e-4`, the same one every other `samples` row uses.

**Never by bytes**, and that is a property of this run rather than a convenience:
MadGraph regenerates a single-group run's events bit-identically, but `pp_to_jj`'s
five subprocess directories make its unweighting draw scheduling-sensitive, so a
re-run of the same card yields a different and equally valid sample (Sb). Only its
distributions are statements about it.

| seed | n_eff | worst KS | `χ²` SPINUP | `χ²` ICOLUP | `χ²` flavour |
|---|---|---|---|---|---|
| `0x5a4d1001` | 19 956 | `y(j1)` p 9.07e-6 (D 0.0303) | p 6.10e-3 (18.1/6) | p 0 (5089.1/31) | p 0 (3252.2/112) |
| `0x5a4d1002` | 19 996 | `y(j1)` p 1.25e-6 (D 0.0327) | p 6.14e-2 (13.5/7) | p 0 (5117.2/32) | p 0 (3256.2/104) |
| `0x5a4d1003` | 19 986 | `y(j1)` p 1.12e-7 (D 0.0353) | p 7.63e-4 (23.1/6) | p 0 (5095.2/28) | p 0 (3290.9/108) |

Minimum KS `p` `1.12e-7` on `y(j1)` (and `cos(j1)`, its image), minimum `χ²` `p`
`0` on `ICOLUP`, over three seeds. The cell fails every column it can fail, on
every seed, and it fails them the way the surplus predicts: the draw over
subprocesses is what is wrong, so the *frequencies* move — `ICOLUP` because which
colour topologies occur is a property of which subprocess an event came from,
`flavour` directly, and the jet rapidity because `g q → g q` and `g g → g g`
populate `y` differently and one of them is doubled. **`SPINUP` is the column that
survives** on all three seeds, which is consistent: the helicity fractions are the
observable least sensitive to reweighting one subprocess against another.

**What this cell cannot do is attribute.** It says the sample disagrees, and the
surplus is sufficient to explain everything it shows — but a distribution
comparison cannot separate a sufficient cause from the only cause. What would close
that is re-measuring this cell once the enumeration is repaired: the columns coming
back inside the floor is the statement, and it is not one this session can make.

Two counts worth carrying forward: the comparison finds `171`–`174` distinct
flavour keys of which `105`–`113` are compared (the rest below the pooling
threshold, `0.5`–`0.6 %` of the sample pooled), and `35`–`36` `ICOLUP` keys of which
`29`–`33` are compared. `flavour_key` is taken on a `canonical()` event, whose
outgoing legs are sorted by class label and then by `pT` — so within a jet final
state the key is flavour *and* `pT` ordering, and it is blind to the leg ordering
the enumeration surplus is made of. That is why the surplus appears in this cell as
a frequency and never as a category MadGraph does not emit.

### C.6 The instruments, and what each provably cannot see

- **`jj_subprocesses_are_madgraphs_own_plus_the_outgoing_permutations`** is the
  finest level available for this defect: a set comparison at zero tolerance
  against the run's own `leshouche.inc`. It cannot see anything about the
  *diagrams* of a listed subprocess, or about any cross section — a set is blind to
  both. It is enforced, so it fails the moment the enumeration changes on either
  side.
- **The banked-sample ordering counts** turn "a labelled `dΦ₂` covers both
  orderings" from a reading into a measurement against MadGraph's own events. They
  cannot see whether the coverage is *correctly weighted* — only that both
  orderings occur inside one assignment and that neither side emits the swap.
- **The bracket from the per-directory cross sections** is an independent
  prediction of the σ ratio from numbers this session did not fit. It is blind to
  anything that moves the equal- and unequal-outgoing parts by the same factor.
- **The collapsed arm** is a scalar and therefore blind to a compensating pair of
  errors, and to a mis-sampled region of small measure. What guards those is the
  seed sweep and the ladder in §C.4, and beneath them `validate_scales`'s replay of
  this run's 10 000 events field by field.
- **`probe_jj_channel_partition`** brackets the ambiguity between two constant-`αⱼ`
  partitions. It cannot locate MadGraph inside that bracket: MadEvent partitions by
  single-diagram enhancement, `AMP2_c(p)/Σ AMP2`, a function of the point rather
  than a constant, which is not reachable from any choice of `αⱼ`.
- **The `samples` comparison** is by distribution and never by bytes, because this
  run's five subprocess groups make MadGraph's own unweighting draw
  scheduling-sensitive (Sb). It is blind to correlations between columns and to a
  discrepancy confined to a small tail, and — `canonical()` sorting the outgoing
  legs by label — it is blind to the *ordering* within an assignment, which is
  precisely why the surplus shows up in it as a frequency and not as a new category.

### C.7 What this leaves

**Landed.** The `pp_to_jj` `integrals` and `samples` cells move from ⛔ `blocked`
to ⚠️ `banked`/`info` — measured, with the reason attached, rather than a silent
gap — and `jj_subprocesses_are_madgraphs_own_plus_the_outgoing_permutations` is an
*enforced* comparison against the run's own `leshouche.inc` which fails the moment
either side's enumeration moves. The census over the 30-row × 4-category report
moves `86 → 88` measured — `85 ✅, 3 ⚠️, 4 ⏳, 10 ⛔, 18 uncovered` against
`85, 1, 4, 12, 18` — with the two ⛔ that became ⚠️ being exactly this row's.

**The two rendered tables differ on exactly the two cells and nothing else.** A
line-by-line diff of the report against the one taken before any of this session's
changes moves 25 lines: the `pp_to_jj` row itself, its two footnotes, the mechanical
renumbering of the four footnotes after them, the two new measurement lines in the
appendix, and the census line. Every other row is identical in every printed field,
so nothing this session added reached a gated measurement.

**Both work-area states are green.** With the four unbundled runs present the
banked layer exits `0` at the census above; with them held out it exits `0` at
`84` measured (`83 ✅, 1 ⚠️, 8 ⏳, 10 ⛔, 18 uncovered`), the two new `pp_to_jj`
cells reading *awaiting the bundle* and nothing else moving. The four held-out runs
were restored and verified byte-identical over all 2867 files.

**Not landed, deliberately: the fix.** It is one line — sort the final state in
`generate_sets_inner`'s dedup key — and it is filed rather than written here,
because it changes the enumeration every process goes through and wants the whole
banked layer behind it. Everything that session needs is measured above: the
collapse reproduces MadGraph's set entry for entry *in MadGraph's own outgoing
order*, so no representative rule is wanted; the control says the `ℓℓj`
enumeration does not move under it, and by the same criterion — final-state slots
drawing on *intersecting* alias sets — no other manifest row's can; and the
collapsed arm's ladder and seed sweep already say where
the row would gate — `rel_tol` at the reference's own `0.22 %` scale rather than at
a partition band, with the pull **asserted** rather than reported. The reason the
pull is safe here and not on the `ℓℓj` rows is arithmetic: MadGraph's own error on
this run is `1.47e6 pb` against this side's `1.9e5` at the gate budget, so the
combined error is the reference's and the pull cannot be driven up by raising this
side's budget. It reads `+0.99` at `300 000` and `+1.25` at `600 000`.

**For Z.** `pp_to_jj` stays `bundled = false`, so on a fetching checkout its two new
cells are ⏳ until the re-cut. This session touches nothing else about the bundle,
and it does not prune `pp_to_llj_qcd2_qed2`.

## C2 — the enumeration repaired, and what the repair uncovered

C localised the `p p > j j` surplus exactly and named the fix without making it.
This session makes it, measures the blast radius, and gates what the fix earned —
which is the cross section and not the event sample. Re-measuring the `samples`
cell through the repaired enumeration is what C said would close its attribution
blind spot, and it closed it in the direction C could not see: the surplus was
sufficient to explain every column that cell failed, and it was not the only
cause. One column still fails, for a reason that has nothing to do with
enumeration.

### C2.1 The fix

`generate_sets_inner` (`diagrams/mod.rs`) keyed its dedup on `(sorted initial,
final state as written)`. It now sorts the final state too, **in the key only** —
the surviving `DiagramSet` keeps the order the expansion emitted it in, which
§C.1 measured to be MadGraph's own:

```rust
let mut final_sorted = concrete.final_state.clone();
final_sorted.sort();
if !seen_processes.insert((initial_sorted, final_sorted)) {
    continue;
}
```

The comment above it states the rule and why it is sound — `dΦ_n` is integrated
over the whole labelled region and every run-card cut is a per-class one, so a
permutation of the outgoing legs relabels the integral without moving it — and
keeps the guarantee the old comment carried, that distinct final-state *content*
never collapses.

The enforced set test moves with it.
`jj_subprocesses_are_madgraphs_own_plus_the_outgoing_permutations` is now
`jj_subprocesses_are_madgraphs_own`: the surplus assertion becomes set equality,
and two statements are added so the equality is not satisfiable for a trivial
reason —

- MadGraph's own side of the rule: of its 65 assignments, **52** have unequal
  outgoing flavours, and **0** of those 52 have the swap that `leshouche.inc`
  could have listed. Without this the equality could hold because there was never
  a choice to make.
- The equality itself carries the representative claim: each entry lists its
  outgoing legs in the order that side enumerated them, so `ours == mg` says the
  surviving representative is in **MadGraph's own outgoing order**, not merely one
  of the two.

`probe_jj_outgoing_permutation_costs_the_cross_section` and the
`collapse_permutations` flag on `jj_groups` are **deleted**. Their counterfactual
arm was the defect, and reconstructing it would mean re-adding the surplus to
demonstrate a bug that no longer exists; §C.2 is its record. `probe_jj_budget_ladder`
and `probe_jj_channel_partition` lose their duplicated arm and run the production
enumeration.

### C2.2 The set, at zero tolerance

```text
[pp_to_jj] concrete subprocesses: MadGraph 65 (52 with unequal outgoing flavours,
  0 of those with the swap also listed), this side 65 — 0 missing, 0 surplus
[pp_to_llj_fixed] control: 212 enumerated sets, 212 surviving the same key,
  24 of them carrying diagrams
test jj_subprocesses_are_madgraphs_own ... ok
```

`117 = 65 + 52` → **65 = 65**, and the negative control is unmoved: a card whose
final-state slots draw on disjoint alias sets keeps every one of its 212
enumerated sets under the same key, so the rule merges where a label repeats and
nowhere else.

### C2.3 Blast radius, measured

`cargo build` and `cargo test --workspace` (hermetic, no features) both exit `0`;
nothing in the hermetic layer pinned the `117`. The banked layer exits `0` and the
rendered report is diffed against the one taken before the change:

| | baseline | fixed |
|---|---|---|
| table rows differing (footnote indices normalised) | — | **1** (`pp_to_jj`) |
| appendix measurement lines | 82 | 82, **2** differing (both `pp_to_jj`) |
| census | `85 ✅, 3 ⚠️, 4 ⏳, 10 ⛔, 18 uncovered` | `86 ✅, 2 ⚠️, 4 ⏳, 10 ⛔, 18 uncovered` |
| measured cells | 88 | 88 |

Every other row is character-identical in every printed field. That is what §C.1
predicted and why: `gg_to_gg` and `uux_to_uux` spell their final states with
concrete particles, so there is no duplicate to collapse, and `p p > j j` is the
only manifest row whose final-state slots draw on intersecting alias sets. The raw
`diff` moves 44 lines, of which everything but the `pp_to_jj` row, its two
footnotes and the census line is the mechanical renumbering of the footnotes after
them.

**Both work-area states are green.** With the four unbundled runs present the
banked layer exits `0` at the census above. With them held out it exits `0` at
`84` measured (`83 ✅, 1 ⚠️, 8 ⏳, 10 ⛔, 18 uncovered`), both `pp_to_jj` cells
reading *awaiting the bundle*. That report is **character-identical to §C.7's
held-out one across all 78 of its measurement lines**, which is the blast-radius
statement again from the other side: with the only row the fix can reach removed,
the fix is invisible. The four held-out runs were restored and verified
byte-identical over all **2867** files.

### C2.4 The σ cell — GATE

The banked layer's own three seeds at `300 000 × 10`, on the production path:

```text
[jj] GATE vibegraph σ = 6.803009e8 ± 2.511e5 pb (3 seeds, χ²/dof = 2.52)
     | MG σ = 6.788500e8 ± 1.473e6 pb | pull = +0.97 | rel = +0.0021
```

against `9.246158e8 ± 3.363e5 pb, χ²/dof 6.53, rel +0.3620, pull +162.70` before
the fix. The tolerance is `JJ_MAX_REL = 0.005` with `|pull| < 3` **asserted** and
`χ²/dof < 4`.

**The oracle-layer ladder, five seeds a rung, through the real fix rather than
through C's counterfactual filter** (`probe_jj_budget_ladder`):

| neval | σ ± Δ (pb) | χ²/dof | rel | pull |
|---|---|---|---|---|
| 75 000 | 6.799571e8 ± 3.900e5 | 1.41 | +0.16 % | +0.73 |
| 150 000 | 6.802836e8 ± 2.753e5 | 0.61 | +0.21 % | +0.96 |
| 300 000 | 6.803185e8 ± 1.943e5 | 1.26 | +0.22 % | +0.99 |
| 600 000 | 6.806965e8 ± 1.375e5 | 1.25 | +0.27 % | +1.25 |

Every rung reproduces §C.4's collapsed arm **to the printed digit**, which is a
statement about the fix and not about the row: it says the production enumeration
and C's `(sorted incoming, sorted outgoing)` filter over the enumerated
`DiagramSet`s are the same estimator, so C's evidence transfers rather than
merely agreeing.

**Why the tolerance is the reference's error and not a partition band.**
`probe_jj_channel_partition`, one seed, `300 000 × 10`, everything else held:

| arm | adapted α | uniform α | partition gap | Monte Carlo |
|---|---|---|---|---|
| `j j` | 6.798582e8 ± 4.33e5 | 6.805569e8 ± 4.90e5 | **+1.028e-3** | 9.6e-4 |
| `pp_to_llj_fixed` (control) | 4.230616e2 ± 3.99e-1 | 4.230056e2 ± 4.63e-1 | −1.324e-4 | 1.4e-3 |

`1.07 σ` — the gap is at its own Monte-Carlo error, against the `1.5e-2` at `9 σ`
that `gu_to_epemu` and `gux_to_epemux` carry (§K6.5). So this row does not inherit
`0.02`, and the pull is asserted rather than reported: the residual is a
fluctuation, not a systematic of fixed size, and the arithmetic keeps it that way
— MadGraph's error on this run is `1.47e6 pb` against this side's `2.5e5` at the
gate budget, so the combined error is essentially the reference's and raising this
side's budget cannot drive the pull up. It reads `+0.99` at `300 000` and `+1.25`
at `600 000`.

`0.005` is `2.3×` MadGraph's own `0.22 %` on this run and `1.9×` the ladder's
worst rung. What it cannot see is a residual below about `0.2 %`: the estimator
climbs `0.11 %` across an eightfold budget — increments `+0.05 %`, `+0.01 %`,
`+0.05 %` — which is half the reference's own error and is not resolved into an
asymptote. The row is converged **at the scale the comparison is made at**, and
the ladder is kept so a later session can say more.

**Variance recovered, and it was predicted.** §C.4 read the enumerated arm's five
seeds at `χ²/dof 3.41` where the collapsed arm's sat near one, and attributed the
excess to six extra sampling channels that were relabelled images of channels
already in the mixture — the ill-conditioned multichannel of note 27 §B3.2. Fixed,
the five seeds at the gate budget scatter at **1.26**, and the whole ladder sits in
`0.61–1.41`. The gate's own three seeds read `6.53 → 2.52`; `2.52` on two degrees
of freedom is `χ² = 5.0` at `p ≈ 0.08`, unremarkable at that sample size and the
reason the bound is `4.0` rather than something tighter. The mechanism the
prediction named is directly visible in the integrator's own header, on the same
card before and after the fix — `channels: 25 grids` → `channels: 19 grids`, with
σ `9.242427e8 ± 1.17e6` → `6.799516e8 ± 8.79e5` at `120 000 × 6` — so the six
channels that left are the six the prediction said were relabelled images.
(§C.2's counts of the flavour groups behind them, `11 → 8`, are that session's.)

### C2.5 The `samples` cell — one column, and it is not the enumeration

Three seeds of 20 000 events against the banked 10 000, `p`-floor `1e-4`,
everything else as C ran it.

| column | C (`117` subprocesses) | this session (`65`) |
|---|---|---|
| worst KS | `y(j1)` p **1.12e-7** | `phi(j1)/pi` p **7.94e-2** |
| `SPINUP` | p 7.6e-4 … 6.1e-2 | p **0.18 … 0.35** |
| `flavour` | χ² ≈ **3270 / 108**, p 0 | χ² **80.5/64, 83.2/70, 78.2/67**, p **0.079 … 0.164** |
| `ICOLUP` | χ² ≈ **5100 / 30**, p 0 | χ² **2455.0/26, 2494.2/25, 2479.1/24**, p **0** |

The kinematics and the flavour frequencies come back inside the floor, which is
the statement C wanted: the subprocess mixture was the surplus and the surplus is
gone. `ICOLUP` does not, and the cell stays `info` for that column alone rather
than being forced.

**What it is.** Not a frequency question and not MadGraph's to settle: the events
this row emits for a flavour assignment containing an **antiquark** put that leg's
colour line in `ICOLUP(1)`, where the Les Houches convention puts an antiquark's
in `ICOLUP(2)`. Checked on the record alone, with no reference in the comparison —
one integration per row at `120 000 × 6` off the row's banked cards, then
20 000 events at one seed:

| sample | events | legs violating the `ICOLUP` slot convention |
|---|---|---|
| `p p > j j`, this session | 20 000 | **4 758 / 80 000** — every one an antiquark, and always *both* antiquark legs of the same event (2 309 events) |
| `p p > j j`, enumeration as C left it | 20 000 | **7 382 / 80 000**, including **238 quark** legs |
| `p p > l+ l- j` (samples GATE) | 20 000 | **0 / 100 000** |
| `p p > b b~` (samples GATE) | 20 000 | **0 / 80 000** |
| MadGraph's own banked `pp_to_jj` | 10 000 | **0 / 40 000** |

The second row is the load-bearing one: **the defect predates this session and the
fix strictly reduces it**, removing an entire class (the quark-leg violations were
the surplus's swapped orderings). The third and fourth say it reaches no currently
gated row.

**Where it comes from**, read off the code rather than guessed.
`SubprocessRecord::relabelled` (`lhef/build.rs`) carries a flavour group's colour
flows from the group representative to every member, and says so:

> The colour flows and the pole masses travel with the legs and only the codes
> change: the flavours sharing an amplitude are the ones whose legs carry the same
> masses.

Sharing an amplitude and a mass list does not imply sharing a colour *rep*. `u`
and `ū` share both and carry conjugate SU(3) reps, so their `ICOLUP` slots must be
swapped — and `color_flow_tags` derives each flow's slots from the leg's own rep
and *checks* them, so tags legal for an antiquark leg cannot be what was applied.
By elimination the tags applied are the representative's. `p p > j j` is the first
row whose groups mix the two: `g q > g q` and `g q̄ > g q̄` share a pointwise
`|M|²`, mass list, cut filter and colour basis, so the flavour decomposition puts
them together.

**Why the net did not have this already.** `color_flow_tags_oracle` compares the
derived `ICOLUP` table against `leshouche.inc` for every generated subprocess —
but for the *first* subprocess of each `SubProcesses/P*` directory, the one
`matrix1_orig.f`'s header names. That is the same representative whose tags this
crate then reuses, so the oracle validates exactly the member that is right and
never a member whose reps are conjugate. A cross-tabulation of flavour key against
colour key over the two samples makes the gap concrete: **39** flavour assignments
where the two sides' colour-key sets are *disjoint* (MadGraph 1 145 of 10 000
events, ours 2 274 of 20 000), every one of them containing an antiquark, against
**32** where the sets agree (MadGraph 8 838, ours 17 620).

### C2.6 The instruments, and what each cannot see

- **`jj_subprocesses_are_madgraphs_own`** is a set comparison at zero tolerance
  against the run's own `leshouche.inc`, and it is enforced, so it fails the moment
  either side's enumeration moves. It cannot see anything about the *diagrams* of a
  listed subprocess, about any cross section, or about what the record layer later
  does with a subprocess — a set is blind to all three, and §C2.5 is the third one
  biting.
- **The report table-diff** says nothing else moved *in a printed field*. It cannot
  see a change that moves no cell — a re-association inside an equal σ, say — and
  it is only as fine as the layer that wrote the cells.
- **The σ gate** is a scalar over three seeds. It cannot see a compensating pair of
  errors, a mis-sampled region of small measure, or anything the sum over
  subprocesses averages out; the ladder and the seed sweep guard the first two, and
  `validate_scales`'s per-event replay of this run's 10 000 events the third.
- **The colour-slot check of §C2.5** needs no reference and is therefore not
  blind to a shared convention error the way a comparison against MadGraph would
  be — but it only sees *legality*, not correctness: a record whose slots are legal
  and whose flow is the wrong one of two would pass it. What sees that is the
  `ICOLUP` χ², which is why the cell keeps it rather than being retired to the
  legality check.
- **The `samples` comparison** remains blind to correlations between columns, to a
  discrepancy confined to a small tail, and — `canonical()` sorting the outgoing
  legs — to the leg ordering *within* an assignment, which is why the surplus only
  ever showed up in it as a frequency.

### C2.7 What this leaves

**Landed.** `pp_to_jj`'s `integrals` cell is **GATE**, the census moves
`85 ✅, 3 ⚠️` → `86 ✅, 2 ⚠️` at 88 measured, and the enumeration defect is closed
with an enforced zero-tolerance pin behind it.

**Open, and newly attributed.** A flavour group's colour flows are the
representative's, reused for members whose legs carry conjugate colour reps. It is
filed rather than fixed here: it is a change to the record layer with its own
oracle question (the `leshouche.inc` comparison has to reach past each directory's
first subprocess before it can be gated), and this session's scope was the
enumeration. Everything a repairing session needs is above — the mechanism, the
code path, the counts, and a reference-free instrument that fails today.

**For Z.** `pp_to_jj` stays `bundled = false`, so on a fetching checkout both its
cells are ⏳ until the re-cut. Nothing about the bundle changed, and
`pp_to_llj_qcd2_qed2` is untouched.

## Z — close-out

The one-shot session: `refdata-4` cut and verified, the four unbundled rows
flipped, D4's duplicate pruned, the stale blocked-cell wording corrected, one
formatting pass, and the sprint's record written. Nothing else rides on it.

### Z.1 D4 — the duplicate, pruned

The premise was re-measured before anything was deleted. Over the `<event>`
payloads alone, stripped of banners:

```text
pp_to_llj            49544f8c58658aae64cec18952b0a9d0fba88ae4aa47b60d0c0698d15dab6193
pp_to_llj_qcd2_qed2  49544f8c58658aae64cec18952b0a9d0fba88ae4aa47b60d0c0698d15dab6193
```

The whole files differ (`0a84d85d…` against `7049873…`) and the difference is the
banner's process string. The clustering agrees with that reading from a second
direction: the two runs' entries in `kt_cluster_dump_manifest.json` are equal
field for field in every one of their 13 coverage tables — `NONE 125029`,
`IS_DJB 17733`, `GEOM_COLLAPSED 10000`, all of it — and differ only in the dump's
own path and sha. Two dumps of one measurement.

Removed: the manifest row, `scripts/pp_to_llj_qcd2_qed2.mg5`, the `diagrams.json`
counts, the `kt_cluster_dump_manifest.json` entry and its 8.7 MB dump,
the run's place in `gen_kt_cluster_dumps.sh`'s default list, its name in
`validate_alphas`'s `SCALUP_IS_THE_RENORMALISATION_SCALE`, and
`validate_scales`'s entire `DUPLICATE_RUNS` / `Coverage::DuplicateOf` mechanism —
a three-arm classifier with one member, now two arms with none missing.

The run directory itself is retired to
`/Users/ncsmith/src/generators/vibegraph-refdata-retired/` rather than deleted:
the work area is what `assemble_bundle.sh` and every run-inventory assertion
scan, so a pruned run cannot stay in it, and a banked MadGraph run is not
something to throw away.

What the pruning cost in coverage: nothing. `pp_to_llj` keeps the default-order
half of the order-constraint pair and `pp_to_llj_fixed` / `pp_to_llj_dyn` carry
the explicit `QCD=2 QED=2` half, so the two spellings are still required to
select the same diagram content — and now the pair measures two different runs
instead of one run twice.

### Z.2 `refdata-4`

Assembled twice from the same work area:

| | |
|---|---|
| archive | `vibegraph-refdata-4.tar.zst` |
| members | 2505 files, 37 process directories, 18 amplitude tables |
| size | 118 015 652 bytes (`refdata-3`: 104 789 332) |
| sha256 | `c8ef939ec6336fe53015115b7c3194604b1bd2f7cc6b52b5d21be69a82a325e9` |
| two assemblies | **byte-identical** (`cmp` clean) |

The growth is the four new runs less the pruned one. The event text survives the
round trip: the sha256 of the decompressed Les Houches text of all **37** banked
event files is identical between the work area and a fresh checkout that
unpacked the bundle — which is the thing that has to hold, since
`vg_ensure_refdata` re-gzips as it unpacks and a gzip encoder's bytes are not
the ones MadGraph wrote.

**`published = false`.** The pin is live and the fetch enforces it on every
route; until the asset exists the only route is `$VIBEGRAPH_REFDATA_SOURCE`, and
CI's `banked` job is red. The two commands are §Z.7.

### Z.3 The flips, and both work-area states

`bundled = false` is gone from all four rows (`ud_to_epemud_qcd0`,
`pp_to_llj_dyn`, `pp_to_jj`, `pp_to_ll_scalefact2`), which restores the hard
`require()` their cells were exempt from. Both states were run rather than
reasoned about:

- **with the runs present** — `bash validation/validate.sh` exits `0` at
  `87 measured (85 ✅, 2 ⚠️, 4 ⏳, 8 ⛔, 17 — / uncovered)`;
- **from the bundle alone** — a clean `git archive` export of the branch, given
  the pinned submodule and the two fetched PDF sets (neither is in any bundle)
  and nothing else, unpacks `refdata-4` and exits `0` at the same census.

The two rendered reports are **byte-identical**, which is the strongest form the
statement comes in: a fetching checkout does not merely pass, it produces the
same table, so no cell is quietly standing on something only this machine has.
`vg_ensure_refdata` re-gzips the event files as it unpacks and the export's own
LHE gate still reads the same corpus the work area does — `744 759 events /
3 869 480 particle lines across 37 banked runs`, same 16/21 dialect split.

That is what the flips are for: before them a fetching checkout read those eight
cells as ⏳ *awaiting the bundle*; after them a missing run is a failure naming
the run, and `a_row_the_bundle_carries_may_not_be_absent` is what proves the
failure still happens.

**What the export proof also showed, and it is a gap rather than a result.**
`validate_kt_cluster` prints

```text
no kT clustering dumps in …/output/ktdump/dumps: run `pixi run generate-kt-cluster-dumps` to build them
test the_clustering_engine_reproduces_madgraphs_own ... ok
```

— green, having compared nothing. The dumps are 75 MB and deliberately outside
the bundle, so that is every fetching checkout including CI's `banked` job. It is
filed in §Z.8 and `TODO.md` rather than fixed here: bundling them roughly doubles
`refdata-4`, and the alternative is to register the gate at the oracle layer, so
the choice is a coverage decision and not a close-out edit.

**The flip turned the banked layer red, and the reason is worth keeping.**
`validate_sigma`'s `a_row_the_bundle_does_not_carry_may_be_absent` opened with

```rust
assert!(!unbundled.is_empty(),
    "no manifest row is marked bundled = false, so this rule has nothing to check \
     and the gate's tolerance is untested");
```

and with the last four rows flipped there was no such row left. The guard is
right in principle — a rule with no instance is untested, and refusing to pass
vacuously is the `validation-3` lesson — but it drew its instance from the wrong
place. `bundled = false` is *transient*: it exists between a banked run and the
next bundle and is empty the rest of the time, so the coverage it gated vanished
precisely when the manifest was tidiest, which is also when a regression in
`run_presence` would ship unnoticed.

All three classifications are now reached from sets the test builds itself, and
the arm that was weakest got stronger: "present beats declared-absent" used to
run only `if` the first unbundled row's directory happened to exist, and now runs
unconditionally against a bundled run — so it is exercised on a fetching checkout
too, where the old form had nothing to look at. The manifest's own set stays the
input to the `Missing` arm, where it *is* the right oracle: a row that silently
acquired `bundled = false` fails there. No tolerance moved.

The transferable shape: **a vacuity guard is only as good as where it gets its
instance.** Keying one to a state the repository is supposed to leave behind
turns "this rule is covered" into "this rule is covered while we are mid-sprint".

### Z.4 The blocked cells, re-worded

Ten cells named `kt-clustering` as their blocker. It landed in this sprint, so
the name was false on every one of them, and two of the notes under it said the
dynamical scale was "refused" — a claim `validate_scales` now contradicts 10 000
events at a time.

Two of the ten belonged to the pruned row. What blocks the other eight was
measured off the cards:

```text
pp_to_bb             nn23lo1   lhaid 230000   dynamical
pp_to_bb_qcd2        nn23lo1   lhaid 230000   dynamical
pp_to_llj            nn23lo1   lhaid 230000   dynamical
pp_to_ll_scalefact2  nn23lo1   lhaid 230000   dynamical, scalefact 2.0
pp_to_bb_fixed       lhapdf    lhaid 247000   fixed        ← gates
pp_to_llj_dyn        lhapdf    lhaid 247000   dynamical    ← gates
pp_to_jj             lhapdf    lhaid 247000   dynamical    ← gates
```

`nn23lo1` is MadGraph's internal parton-density parameterisation, not an LHAPDF6
grid the `pdf/` layer can load — the reading `pp_to_jj`'s own rationale already
carried, and the reason its card departs from MadGraph's shipped defaults in
`pdlabel` and `lhaid` alone. A cross section built here against one of those runs
would convolve different densities and measure that difference. The blocker is
`mg-internal-pdf` on all eight, each note says what is true now, and no tier
moved: cells that were not measured are still not measured.

The attribution is checkable rather than a story, and the bottom three rows are
what makes it so: each of the three blocked processes has a twin carded onto
`lhaid = 247000`, and every one of those twins gates.

### Z.5 The report, diffed

Against the report rendered before this session's first change, with the runs
present:

| | pre-Z | post-Z |
|---|---|---|
| rows × categories | 30 × 4 = 120 | 29 × 4 = 116 |
| census | `86 ✅, 2 ⚠️, 4 ⏳, 10 ⛔, 18 — / uncovered` | `85 ✅, 2 ⚠️, 4 ⏳, 8 ⛔, 17 — / uncovered` |
| measured | 88 | 87 |
| unbundled rows listed | 4 | none |

Compared cell by cell with the footnote indices normalised away, the two tables
differ in exactly four places and **no measured cell is one of them**:

| what moved | count |
|---|---|
| rows removed (the pruned duplicate) | 1 |
| rows added | 0 |
| ⛔ cells whose blocker string changed `kt-clustering` → `mg-internal-pdf` | 8 |
| `covered-by` pointers dropping the pruned row (`pp_to_llj_dyn` / `diagrams`) | 1 |
| ✅ or ⚠️ cells whose value changed | **0** |

The raw `diff` moves 59 lines; everything beyond the above is the mechanical
renumbering of the footnotes after the ones that moved, the two appendix lines
the pruned row contributed, the coverage-bookkeeping line (four unbundled rows →
`none`) and the census line. That is the whole blast radius, measured rather
than asserted: nothing this session did reached a gated measurement.

### Z.6 The environment

Nothing changed it. `pixi.lock` is byte-identical to the sprint's base
(`b676b6f`), and the only `pixi.toml` change in the whole sprint is K1b's added
`generate-kt-cluster-dumps` task — a task, not a dependency. `pixi install
--locked -e madgraph` succeeds, and the local pixi is `0.63.2`, which is what
CI's `setup-pixi` pins. The refdata-3 close-out's CI failure has no counterpart
here.

### Z.7 For the user: publish, then flip the pin

Two steps, in this order. The bundle sits in the `kt-spine/z-closeout` worktree's
work area, which is gitignored — so the path is absolute rather than relative to
wherever the release is cut from:

```sh
BUNDLE=/Users/ncsmith/src/generators/vibegraph-wt-k4/validation/madgraph/output/bundle/vibegraph-refdata-4.tar.zst

shasum -a 256 "$BUNDLE"
# expect c8ef939ec6336fe53015115b7c3194604b1bd2f7cc6b52b5d21be69a82a325e9

gh release create refdata-4 \
  --repo nsmith-/vibegraph \
  --prerelease \
  --title "Banked reference data, cut 4" \
  --notes "Frozen MadGraph reference runs for the banked validation layer. 37 runs, events as plain .lhe under zstd. Adds the kt-spine sprint's four banked runs (ud_to_epemud_qcd0, pp_to_llj_dyn, pp_to_jj, pp_to_ll_scalefact2) and drops pp_to_llj_qcd2_qed2, whose events were byte-identical to pp_to_llj's. sha256 c8ef939ec6336fe53015115b7c3194604b1bd2f7cc6b52b5d21be69a82a325e9, pinned in validation/manifest.toml [refdata]. Partonic cross sections are comparable to refdata-3 and not to refdata-2." \
  "$BUNDLE"
```

Then, in `validation/manifest.toml`, `[refdata].published = false → true` and
commit. Until that flip CI's `banked` job is red: the pinned `url` resolves to a
release asset that does not exist, and `$VIBEGRAPH_REFDATA_SOURCE` is the only
route to the bytes. `pp_to_jj`'s banked event sample must **never** be regenerated to
reproduce this bundle: its five subprocess groups make MadGraph's own unweighting
draw scheduling-sensitive, so a re-run of the same card yields a different and
equally valid sample (Sb's finding, now also in
`validation/madgraph/README.md`). Any "regenerable from the cards" claim about
the bundle has to exempt multi-group runs.

### Z.8 The sprint, closed

Eleven sessions plus banking and this one, over two tracks:

| track | section | what it landed |
|---|---|---|
| K | §K1 | the binding spec for MadGraph's kT clustering, each claim with its falsifier |
| K | K2 (banking) | the instrumented 3.7.1 oracle — 9 runs × 10k events of dumped intermediates |
| K | §K3 | the clustering engine, informational: 90 000 events, 2.4M candidate pairs, zero observed deviation |
| K | §K4 | the closed forms deleted; `-1` takes one path; every banked run replayed |
| K | §K5a, §K5a2 | `GridAlphaS` *is* LHAPDF's `AlphaS_Ipol`; the density grid continued past its edges |
| K | §K5b | the four llj partonic σ rows and their `samples` cells leave `blocked` |
| K | §K6 | the scale reads the channel the point was drawn in, per flavour group |
| S | §S1 | the identical-particle factor, one definition, derived per subprocess |
| S | §S2, §S4 | the multi-rung spine, and the finding that the fixed-beam path was never regulated |
| S | §S5, §S6 | the fermion-line spine sign on mixed lines — a factor 7.7 in σ, found at the per-diagram level |
| — | Sb | the four MadGraph runs, banked once |
| — | §C, §C2 | the capstone, and the 36% enumeration surplus it uncovered and then repaired |
| — | §Z | this |

**Census.** `75 measured / 74 ✅ / 1 ⚠️` over 26 rows at the sprint's base, to
`87 / 85 / 2` over 29 rows at its close. The two ⚠️ are the decided `gg_to_gg`
4/6 diagram-counting convention and `pp_to_jj`'s `samples` cell, whose single
failing column (`ICOLUP`) is diagnosed rather than tolerated — §C2.5.

**What each session could not see** is in its own subsection above; that is the
sprint's findings register, and the entries still open are filed in `TODO.md`
rather than left here. Three are worth naming because they are load-bearing for
what comes next:

1. **A flavour group's colour flows are the representative's**, reused for
   members whose legs carry conjugate colour reps (§C2.5). It is the only thing
   between `p p > j j` and a gated event sample, it reaches no gated row today,
   and the oracle that should have caught it validates exactly the member that is
   right.
2. **σ at a channel-dependent scale is defined only up to the channel
   partition** (§K6.5). Three rows carry tolerances set by that ambiguity rather
   than by the reference's error, and the fix — drawing the scale's channel
   `∝ AMP2_c(p)` — is a design decision with artifact consequences.
3. **`validate_kt_cluster` is the sprint's finest oracle and the manifest does
   not know it exists.** It has no `[[standalone]]` row, and it returns early
   with a `println!` when `output/ktdump/dumps/` is absent — which is every
   fetching checkout, since the 75 MB of dumps are deliberately not bundled. On
   CI's `banked` job that gate is green without having compared anything. Found
   at close-out, filed rather than fixed: bundling the dumps would roughly double
   `refdata-4`, and registering the row `oracle` instead is a coverage decision,
   not a bookkeeping edit.

The third is the sprint's own lesson turned back on itself. Every σ that moved
here moved because a per-event field was compared first — the clustering was
pinned merge by merge against MadGraph's own intermediates long before any cross
section flipped, which is why each flip arrived with a diagnosis attached rather
than a tolerance. The instrument that made that possible is the one the coverage
bookkeeping cannot see.
