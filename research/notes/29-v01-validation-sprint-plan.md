# 29 — v0.1 validation sprint plan (design–implement–review chains)

Planned 2026-08-02, from TODO's "Validation-sprint slate (restricted scope,
decided 2026-08-02)". Precedes the `v0.1` tag's follow-up work; the scope it
validates is the restricted one: arbitrary fixed-order SM processes, unpolarized
pp / fixed-energy partonic beams, no decay chains.

## 1. Goal and exit criteria

Harden the restricted scope so every boundary a card can reach is a hard error
and every standing discrepancy is resolved or owned. Exit criteria, each a
recorded measurement (never an inference from "the suite passed"):

- **E1** `pp_to_jj` `samples` cell **GATE**: the conjugate-rep `ICOLUP` defect
  fixed, its χ² clearing the `1e-4` floor on every seed, and a regenerated
  20k-event sample carrying **0** mis-slotted antiquark legs (was 4758/80 000).
- **E2** The channel-partition tolerances retired: `gu_to_epemu` /
  `gux_to_epemux` gate below `rel_tol` 0.02 and `pp_to_llj` below 0.015, at
  tolerances justified by the reference's own error, after the per-point
  `AMP2_c` scale-channel draw. The partition probe
  (`probe_channel_partition_moves_sigma`) reads within its own Monte-Carlo
  error under the new draw.
- **E3** Hard errors, each with a refusal test: nonzero `polbeam1/2`;
  decay-chain commas in a process line; `propagators.py` present in a UFO
  directory; every physics-relevant run-card field the audit (§C2) finds
  parsed-but-unread. README's boundary claims become true statements.
- **E4** `ee_to_mumua`'s +0.80% drift **owned**: the windowed `pt(a)`
  comparison decides whether the remaining ~1% is ours or MadGraph 3.7.1's
  channel-weight change, with the evidence in this note's close-out.
- **E5** Census strictly improves: the three `uncovered` cells the `kt-spine`
  runs earned are measured; no green cell regresses; `validate_kt_cluster`
  stops being silently vacuous on fetching checkouts.
- **E6** the 8 `nn23lo1`-blocked ⛔ cells move to measured via the §G re-bank
  at `lhaid = 247000`, with `refdata-5` **replacing** the four superseded runs
  (bundle size roughly flat, retired runs to the local retired area).
- **E7** *(sidecar, non-blocking)* the U(1) charge-flow phase hypothesis
  (chain F) has a recorded verdict against its pre-registered bar — and, if
  very promising, a written refactor backlog entry in TODO (the refactor
  itself is out of this sprint).

## 2. Session protocol: design → implement → review chains

New this sprint: each work item is a **chain of three separate subagent
sessions**, so the sprint manager's job reduces to supervising and
reprioritizing on unexpected outcomes — not carrying design context through
implementation, and not being the only reviewer of work it also specified.

1. **Design session** — agent type `claude`, model **Opus**, read-only intent
   (it writes exactly one thing: its design section, appended to this note in
   its worktree). Input: the chain's brief below plus named code/note
   pointers. Output: a design section with (a) the concrete change list,
   file by file; (b) the acceptance tests, named; (c) the gates to run and the
   cells expected to move; (d) risks and an explicit "what this provably
   cannot break" claim. The manager reads the design before dispatching
   implementation — this is the reprioritization point.
2. **Implementation session** — agent type `validation-dev` (Opus by default;
   Sonnet where marked light). Executes exactly the design, no scope
   invention; runs the gates the design names; commits to the chain branch
   with command+output evidence in its report.
3. **Review session** — agent type `claude`, model **Opus**, fresh context,
   pointed at the chain worktree. Verifies rather than trusts: reads the full
   diff, re-runs the cheap gates itself, checks each design acceptance
   criterion against recorded output, and hunts specifically for the error
   class the implementation's own tests cannot see (AGENTS.md: every oracle
   has a blind spot). Verdict: **merge / fix (named defects) / escalate**.
   A "fix" verdict loops the implementation session (same worktree, same
   agent continued via SendMessage where possible); a second fix loop
   escalates to the manager.

Manager-side mechanics (AGENTS.md "Sprint & Subagent Operations" is binding):
pre-create each chain worktree off `main` (`git worktree add -b
val4/<chain> <path> main`), verify HEAD after dispatch, COW-copy the MG
reference data in (`cp -Rc`), require `cd` + toplevel/branch verification as
the first action, carry the long-command discipline verbatim in every brief.
The manager merges chains into the sprint branch in the §5 order and runs the
full banked gate after each merge, not only at the end.

## 3. The chains

### Chain A — conjugate-rep colour-flow tags (the `p p > j j` samples blocker)

The defect (note 28 §C2.5, TODO standing discrepancy): `SubprocessRecord::
relabelled` (`lhef/build.rs`) reuses the representative's `ColorFlowTags` for
every group member, so members whose legs carry conjugate SU(3) reps get
`ICOLUP` slots that are provably wrong for their own reps — measured 4758/80 000
legs on `p p > j j`, all antiquarks, reaching no other gated row.

- **Design questions**: conjugate the tags on relabel vs derive per member
  (note 28 recorded both as viable — pick one with reasons); how to widen
  `color_flow_tags_oracle` past each `P*` directory's *first* subprocess so
  the repair is gated rather than asserted; which record-layer self-check
  makes the defect class impossible to reintroduce (derive-and-check from the
  leg's own rep per member, not per representative).
- **Acceptance**: E1; oracle widened to every subprocess in every banked `P*`
  directory; `pixi run validate` census shows only the intended flips; the
  extended-validation colour/LHEF gates pass.
- **Models**: design Opus, implement Opus, review Opus. The subtle sign/index
  territory of the `color-flow` sprint's slot-swap bug lives here — nobody
  light.

### Chain B — per-point `AMP2_c` scale-channel draw (+ the dynamical-llj samples cell)

The residual `kt-spine` K6 left (note 28 §K6.5/§K6.8/§C.3): the scale reads
the sampler's channel, so σ at a channel-dependent scale is only defined up to
the channel partition. Fix by drawing the scale's channel `∝ AMP2_c(p)` per
point — MadEvent's own `iconfig` rule — decoupling the scale from this crate's
α-adaptation.

- **Design questions**: where the extra per-point draw's randomness comes from
  (a dedicated ChaCha8 substream so integrate/generate reproducibility is
  preserved seed-for-seed); whether the draw lives in the integrand or the
  scale prescription; artifact consequences (does anything banked change
  meaning — if yes, format-version bump and refusal semantics); which
  tolerances tighten and to what, justified by the reference's own error;
  interaction with `generate` (the same scale rule must run in the event
  pass — a sample whose scales disagree with its integral is a new defect).
- **Rider**: the `pp_to_llj_dyn` `samples` cell (uncovered cell (b), note 28
  §K5b.6) — the dynamical-scale llj card gets its own artifact + generation
  pass + samples comparison, taken here because this chain already rebuilds
  that path.
- **Acceptance**: E2, the rider cell measured, seed sweeps ≥5 with the budget
  axis scanned (AGENTS.md sampler-gating rules), `p p > j j` unmoved (its
  tolerance is already the reference's own error — a 2→2 gives the clustering
  no merge to choose).
- **Models**: all Opus. Largest blast radius in the sprint; merged last.

### Chain C1 — three small hard errors *(light)*

Per the scope decision: nonzero `polbeam1`/`polbeam2` (today parsed as known
fields, read by nothing); decay-chain commas (today misparsed as
required-s-channel syntax, dying with a misleading unknown-particle error —
`diagrams/parse.rs::parse_process_body`); `propagators.py` in a UFO directory
(today unread — a model defining custom propagators silently gets default
propagators, `ufo/mod.rs` file lists).

- **Acceptance**: three refusal tests with messages naming the unsupported
  feature and the tracked backlog entry's subject (not the plan item —
  AGENTS.md comment rules apply to error text too: describe the boundary in
  its own terms); hermetic suite still complete on a bare clone.
- **Models**: design **Sonnet**, implement **Sonnet**, review Opus (cheap
  chain, but the review checks the error paths actually fire from the CLI,
  not only from unit tests).

### Chain C2 — run-card ignored-field audit + the μF ≥ 2 GeV veto

The general form of C1's disease: the parser accepts every known field (typo
protection rejects unknown keys) but nothing proves the known-and-unread ones
are physics-inert. Plus the one *behavioral* silent disagreement in scope:
`reweight.f:1185` vetoes points with μF < 2 GeV; `coupling::scales` only
reports the scale (note 22 §4).

- **Design questions**: the audit method (walk `runcard.rs`'s default table
  against actual consumers — LSP reference queries per field); the
  classification (consumed / ignored-benign / ignored-physics→hard-error) and
  where its table lives so it is *asserted*, not documentary — a test that
  fails when a new field is added without classification is the durable form;
  veto semantics matched to `reweight.f` (veto the point vs error the run —
  MG vetoes, so we veto) and which gated row proves it (none today reaches
  μF < 2 GeV, so the test is a constructed card, and the design must say why
  no banked reference exercises it).
- **Acceptance**: E3's audit half; the classification test in the hermetic
  suite; veto implemented with a constructed-card test; DY and every gated σ
  row unmoved.
- **Models**: design Opus, implement Opus (the audit is judgment-heavy),
  review Opus.

### Chain D — `ee_to_mumua`: own the 3.7.1 drift *(measurement, not code)*

The widest σ row (+2.79 pull, rel +0.80%) and the tightest samples cell
(min KS p 2.74e-4), both moved when references went 3.5.7 → 3.7.1. The photon
is soft/collinear-regulated by cuts — exactly the region MG's channel-weight
change reallocates. The chain produces a **measurement**, in the style of
note 27 §B1's Higgs-pole windows: a windowed comparison over `pt(a)` that
localizes the disagreement or refutes localization.

- **Design questions**: window boundaries (from the banked sample's own
  `pt(a)` distribution); which side's per-window σ carries which error; the
  decision rule stated *before* the measurement (what pattern would convict
  our sampler vs the reference's reweighting) — pre-registered, so the
  outcome cannot be argued into either box after the fact.
- **Acceptance**: E4 — the measurement recorded in this note with the verdict;
  if ours, a filed defect with the diagnosis attached (fix may be its own
  chain, reprioritized then); if the reference's, the row's tolerance stays
  and the note documents why that is not a loosened tolerance (the reference
  moved, not us).
- **Models**: design Opus, implement Opus (it is an investigation — the
  "implementation" is instrumented runs), review Opus.

### Chain E — ForcePositive + cheap hygiene bundle *(light implement)*

- **`ForcePositive`** (note 28 §K5a2): ~5 lines in the PDF extrapolation path
  plus re-reading the interpolation gates' `FORCE_POSITIVE_FLOOR` screen; the
  checked-data relationship test
  (`the_only_difference_from_madgraphs_own_value_is_the_positivity_clamp`)
  becomes an exact-agreement assertion (205/205 clamped probes matching MG).
- **`validate_kt_cluster` declared tier** (note 28 §Z): register the gate as
  an `oracle`-layer `[[standalone]]` row and delete the silent
  `println!`-and-return — in the oracle layer absent dumps are a hard fail;
  the banked layer stops pretending to run it. (The bundle-the-75-MB
  alternative is rejected here for size; revisit only if the oracle layer
  proves too rarely run.)
- **`release-debug` `#[should_panic]` tests**: gate
  `eval_m2_pruned_rejects_boosted_frame` (and any sibling) on
  `cfg(debug_assertions)` so `cargo test --profile release-debug` is clean.
- **Uncovered cells (a) and (c)** (note 28 §S6): `ud_to_epemud_qcd0`
  `samples` comparison written against its banked event file;
  `ud_to_epemud_qcd0` counts banked into `diagrams.json` (35 = 35, becomes a
  passing hermetic gate).
- **`validate_scales` widening** (note 28 §K5a): the four `pdlabel = lhapdf`
  runs join `banked_events_reproduce_aqcdup_from_the_computed_scale`, closing
  cluster-scale → μR → αs(μR) in one per-event comparison.
- **Acceptance**: E5; every touched cell's flip visible in the report diff;
  no tolerance moved anywhere.
- **Models**: design Opus (one design session covers the bundle), implement
  **Sonnet**, review Opus.

### Chain F — sidecar: diagram phase from U(1) charge flow *(research-only, linked to nothing)*

A standing pattern rather than a standing defect (user, 2026-08-02): the
amplitude's flow-related phase conventions have been the sprint-over-sprint
bug generator — the `color-flow` fermion-flow slot-swap, the VVVV −i (note
16), every rooting sign now lifted into `fermi_sign`, and the one *fitted*
constant `G = ±i` serving the per-diagram and per-flow oracles. Each was fixed
locally and pinned by a test, but nothing derives them from a single
principle. The hypothesis to investigate: **following the diagram's U(1)
charge flow — the fermion-number/charge arrows as a flow structure, the way
SU(3) is handled as color flow — determines the diagram's phase, holistically
replacing the accumulated per-case conventions** (much simpler than SU(3),
since U(1) has no basis to permute — but possibly the same *kind* of
bookkeeping).

- **Strictly research-only.** No production code, no refactor, no touching
  `fermi_sign` or the evaluators. The deliverable is evidence and a verdict.
- **Design questions** (pre-registered before any derivation): what, exactly,
  the claim predicts — for each convention currently pinned by a test
  (`fermi_sign`'s cases, the VVVV phase, the fitted `G = ±i`, per-flow JAMP
  phases), what value the charge-flow rule derives and which existing oracle
  dump can check it; what result counts as "very promising" (proposed bar:
  the rule reproduces *every* pinned convention with zero free parameters, or
  reduces the fitted constants to derived ones) versus "interesting but not
  actionable"; which known-hostile cases to test first (the slot-swap
  configuration, an alternative rooting from the 165-rooting sweep, a
  multi-flow process).
- **Method**: derive on paper against the banked per-diagram × per-helicity ×
  per-flow complex dumps — the finest oracles already exist, so the
  investigation needs no new instrumentation, only scratch evaluation scripts
  in the worktree (never merged).
- **Acceptance**: a findings section in this note with the verdict against
  the pre-registered bar. **If very promising**: a written refactor backlog
  entry in TODO's feature backlog — scope, migration path, which tests would
  become derivations — explicitly *not* attempted this sprint. If not: the
  negative result recorded with the same care (why the rule under-determines
  the phase, which convention escapes it).
- **Sidecar semantics**: dispatched whenever bandwidth allows, blocks
  nothing, nothing blocks it, and no other chain may take a dependency on its
  outcome. Merges at most a note section and a TODO entry.
- **Models**: design Opus, research Opus, review Opus (the review checks the
  derivations against the dumps itself, not the researcher's summary of
  them).

## 4. §G — the `nn23lo1` re-bank (decided: option (i), user 2026-08-02)

Four banked runs (`pp_to_bb`, `pp_to_bb_qcd2`, `pp_to_llj`,
`pp_to_ll_scalefact2`) carry MG's internal `nn23lo1` parameterisation; 8 ⛔
cells. **Decision: option (i)** — re-bank the four cards at `lhaid = 247000`.
(The alternatives from note 28 §Z — implementing the internal parameterisation
with no oracle in this tree, or carrying the ⛔ cells into v0.1 — are
declined.)

**The re-bank is a replacement, not an addition.** The bundle is already
~100 MB and does not get to grow by superseded runs: `refdata-5` is
`refdata-4` with the four `nn23lo1` runs **removed** and their `247000`
re-banks in their place, so the size stays roughly flat. The retired runs go
to the local retired area, following the `pp_to_llj_qcd2_qed2` precedent
(note 27 D4: `~/src/generators/vibegraph-refdata-retired/`) — kept off the
bundle, recoverable by hand. The manifest rows move with them: the four
processes' cards change (`pdlabel`/`lhaid`), so their references change
identity, and any test still naming the `nn23lo1` variants must fail loudly at
the pin flip rather than read stale data.

It runs as an **oracle-layer background task** (MG runs detached per the
long-command discipline) started at sprint open, since nothing else depends on
it; the `refdata-5` publish + pin flip is its own gate, and the 8 cells flip
only on recorded measurements against the new runs (E6).

## 5. Sequencing and merge order

Dispatch waves (chains are independent worktrees; the wave structure is about
review/merge bandwidth and conflict surface, not data dependencies):

- **Wave 1 (dispatch together)**: C1, E designs; A design; the §G re-bank
  starts in the background (decided — no gate on the user remains).
- **Wave 2**: C1, E implement+review while A implements; C2 and D designs.
- **Wave 3**: B design (reads A's merged state — both touch event records);
  C2, D implement+review.
- **Wave 4**: B implement+review; §G re-bank lands as `refdata-5` + pin flip
  (replacement semantics per §4).
- **F (sidecar)**: dispatched whenever review/merge bandwidth allows —
  earliest in wave 2 — on no wave's critical path; it merges at most a note
  section and a TODO entry, so it never contends for the manifest/report
  conflict hotspot.

Merge order into the sprint branch: **C1 → E → A → D → C2 → B**, full banked
gate after each merge. Rationale: smallest blast radius first; A before B
because B's rider regenerates llj samples that must carry A's fixed tags; B
last because it is the only chain that can move σ cells. The manifest and
report tables are the expected conflict hotspot — chains touch disjoint rows,
so conflicts are mechanical, but the manager resolves them (never a subagent).

## 6. Risks and reprioritization triggers

- **Chain A finds the representative's tags wrong too** (not only the
  conjugate reuse): the widened oracle looks at members no oracle has seen —
  a real-finding risk. If it fires, A's review escalates and the fix becomes
  the sprint's centerpiece; nothing else depends on A except B's rider.
- **Chain B moves a σ row it should not**: the draw changes the integrand's
  variance structure. The pre-registered claim is that *only* the
  channel-dependent-scale rows move; any other movement is a defect in the
  draw, not statistics (AGENTS.md: a failure migrating between seeds under
  budget is a bug).
- **Chain D convicts our sampler**: E4 then spawns a fix chain, and the
  sprint's budget question goes to the user rather than being absorbed
  silently.
- **The audit (C2) finds a physics-relevant field a gated run already sets**:
  that is a latent wrong result, not a future hard error — it escalates
  immediately, because a census cell may be resting on it.
- **Session protocol overhead**: three sessions per chain is new. If the
  light chains (C1, E) show the design step adding latency without content,
  the manager may collapse design+implement for *light* chains only — a
  protocol observation to record in the close-out either way.

## Chain D design (2026-08-03)

Design session output. Nothing here is a measurement: every number below is
either read off a banked artifact, read off MadGraph source, or derived from the
run card. The measurement is the implementation session's job, and §D.6 is
pre-registered so its outcome cannot be argued into a box after the fact.

### D.0 — the brief's stated mechanism is refuted before the measurement starts

The chain brief (§3, and TODO's standing-discrepancy entry) motivates the
measurement with "the photon is soft/collinear-regulated by cuts — exactly the
region MadGraph's channel-weight change reallocates". **That mechanism provably
cannot reach this process**, and the design has to say so before it sets a
window, because otherwise the windows get placed around a mechanism that is not
operating.

The channel-weight change is B1's `get_channel_cut` fix (note 27 §B1). In the
banked `ee_to_mumua` directory:

- `Cards/run_card.dat:86` and `Cards/run_card_default.dat:85` both carry
  `1 = sde_strategy`. The auto-selection rule (`banner.py`, in the branch
  guarded by `single_color`) lands on 1 for this process because the final state
  is pure-lepton-plus-photon while the initial state is not partonic, so it
  falls through `pure_lepton and proton_initial` to `elif not no_qcd:
  self['sde_strategy'] = 1`. `SubProcesses/proc_characteristics` confirms
  `single_color = True` and `gauge = unitary`, so the extra
  `proc_characteristic['gauge'] != 'FD'` guard 3.7.1 added to that branch does
  not change the outcome. Diffing `banner.py` between `v3.5.7` and `v3.7.1`
  shows that guard is the *only* change to the rule — **`sde_strategy = 1` in
  both versions.**
- `Source/run_card.inc:333` sets `TMIN_FOR_CHANNEL = -1`.
- `genps.f`'s `get_channel_cut` opens with

  ```fortran
        if(sde_strat.eq.1.and.tmin_for_channel.eq.-1)then
           get_channel_cut = 1d0
           return
        endif
  ```

  so for this run the function is **identically 1** and returns before reaching
  either of the two expressions the 3.5.7 → 3.7.1 fix touched (both of which are
  in any case guarded by `if(sde_strat.eq.2)`).

So whatever moved `ee_to_mumua` between reference banks, it is not
`get_channel_cut`. The implementation session **must re-verify this claim first**
(§D.5 Run 0) and treat a contradiction as a finding, not a nuisance.

What is left as a candidate mechanism is much less exotic, and the numbers point
straight at it:

| | σ (pb) | rel err |
|---|---|---|
| ours (gate budget, seed `20260719`) | `1.006000e-1 ± 1.665e-4` | 0.166% |
| MadGraph **3.5.7** (previous bank) | `1.00630e-1 ± 3.865e-4` | 0.384% |
| MadGraph **3.7.1** (current bank) | `9.980100e-2 ± 2.3352e-4` | 0.234% |

- ours vs 3.5.7: **−0.03%, 0.07σ** — agreement to well inside either error.
- ours vs 3.7.1: **+0.80%, +2.79σ** — the gated row.
- 3.5.7 vs 3.7.1: **−0.82%, 1.84σ** — *MadGraph's own two numbers do not
  disagree at 2σ on their own quoted errors.*

The whole "drift" is therefore a claim resting on the 3.7.1 run's quoted 0.234%
being honest. B1 established, on this same reference generator, that a MadEvent
run's quoted error does **not** cover a coverage miss — three fresh seeds agreed
with each other and with the bank, all confidently wrong by 2.3%. AGENTS.md
generalises that to a rule this design applies *to the reference*: a single run's
error bar is not evidence, a seed sweep is. Nobody has ever swept seeds on
MadGraph's `ee_to_mumua`. That is the cheapest decisive experiment available and
§D.5 makes it Run 1, ahead of any window.

### D.1 — what the process actually is, and where a coverage miss would live

`e+ e- > mu+ mu- a` at `250 + 250` GeV, cuts `pta 10`, `ptl 10`, `etaa 2.5`,
`etal 2.5`, `drll 0.4`, `dral 0.4`; 8 diagrams, 6 surviving MadEvent channels.
Their banked per-channel cross sections (`SubProcesses/P1_ll_lla/G*/results.dat`,
summing to `9.98009e-2` and quadrature-summing to exactly the quoted `2.335e-4`):

| channel | propagator structure (`configs.inc` + `props.inc`) | σ (pb) | share |
|---|---|---|---|
| `G1` | FSR off `mu+`, fixed s-channel γ\* | `1.2042e-2 ± 1.09e-4` | 12.1% |
| `G2` | FSR off `mu+`, fixed s-channel Z | `1.7409e-3 ± 2.97e-5` | 1.7% |
| `G5` | ISR, γ\* at `m(μμ)`, 2 t-channel `e` rungs | `1.6334e-2 ± 1.15e-4` | 16.4% |
| **`G6`** | **ISR, Z Breit–Wigner at `m(μμ)`** | `2.6886e-2 ± 9.09e-5` | **26.9%** |
| `G7` | mirror ISR, γ\* at `m(μμ)` | `1.6159e-2 ± 1.15e-4` | 16.2% |
| **`G8`** | **mirror ISR, Z Breit–Wigner at `m(μμ)`** | `2.6639e-2 ± 8.41e-5` | **26.7%** |

**54% of this cross section is the Z radiative return**, carried by the two
channels whose maps are Breit–Wigners in `m(μμ)` (`PRMASS(-1,6) = MDL_MZ`,
`PRWIDTH(-1,6) = MDL_WZ`). The banked event sample agrees: 39.4% of events have
`m(μμ) ∈ [88, 94]`, and **36.9% sit in the single 1 GeV bin
`E(a) ∈ [241, 242]`**, which is the radiative-return energy

```
p_RR = (s − M_Z²) / (2√s) = (250000 − 8315.3) / 1000 = 241.685 GeV
```

The soft/collinear region the brief names carries, by contrast, **2.1% of the
sample below `pt(a) = 12` and 8.3% below 20**. A +0.80% total offset localised
there would be a +10% error in that window; localised in the radiative return it
would be +1.5%. Both are findable, but they are very different claims and the
windows have to be able to tell them apart. This is the correction the design
makes to the brief: the suspect region is not only the cut-regulated edge, it is
also — and on σ-share grounds primarily — a narrow-resonance coverage question of
exactly B1's kind, with a Z whose width is 2.7% of `m(μμ)` but whose image in
`E(a)` is `M_Z Γ_Z / √s = 0.46` GeV out of a 240 GeV range, i.e. 0.2%.

### D.2 — the windows, and the rule that fixes them

Windows are in `pt(a)`, per the brief (it is the worst `samples` observable, min
KS p `2.74e-4`). The rule is **physics-derived outer and boundary edges, equal
population in between**, and it is stated here so the edges are frozen before any
measurement:

1. **Outer edges** come from the run card and kinematics, not from the sample:
   `pt_lo = pta = 10` GeV, `pt_hi = √s/2 = 250` GeV.
2. **Cut-boundary edge** at `2 × pta = 20` GeV, isolating the region where the
   `pta` cut, not the dynamics, sets the density. Both sides must reproduce a cut
   boundary here, and this is the only window whose lower edge coincides with a
   cut (a blind spot, §D.8).
3. **Radiative-return threshold edge** at

   ```
   pt_RR^min = p_RR / cosh(etaa) = 241.685 / cosh(2.5) = 39.41 → 39.4 GeV
   ```

   the `pt(a)` below which **no on-shell-Z event can survive the `etaa = 2.5`
   cut**. This is a genuine kinematic boundary, not a fitted one: it separates
   the phase space the two Breit–Wigner channels can reach from the phase space
   they cannot. The banked sample confirms it — 0.0% of events below `pt(a) = 20`
   and 2.1% between 20 and 40 have `m(μμ) ∈ [86, 96]`, against 46% just above.
4. **Interior edges** are the equal-population tertiles of the banked sample
   *restricted to* `pt(a) ≥ pt_RR^min`, rounded to 1 GeV: `77.02 → 77` and
   `143.51 → 144`. Equal population above the threshold, rather than over the
   whole range, because below the threshold the physics question is different and
   population-balancing would smear the two regimes together.

**Frozen edges: `10, 20, 39.4, 77, 144, 250`.** Five windows, with the banked
sample's population and the composition measured on the coarse design binning:

| window | `pt(a)` | share of banked sample | binomial rel err at 10k | character |
|---|---|---|---|---|
| **W1** | `[10, 20)` | 8.33% | 3.3% | cut boundary; Z-free by kinematics (0% in the Z mass window, 100% at `m(μμ) > 200`) |
| **W2** | `[20, 39.4)` | 9.62% | 3.1% | soft continuum, still Z-free (2.1% in the Z window) |
| **W3** | `[39.4, 77)` | 27.34% | 1.6% | radiative-return turn-on (≈46% in the Z window) |
| **W4** | `[77, 144)` | 27.47% | 1.6% | radiative-return bulk (≈51–57%) |
| **W5** | `[144, 250]` | 27.24% | 1.6% | radiative-return core, mean `abs(eta_a)` falling to 0.16 (67–74% in the Z window) |

No window carries under 8% of σ, so no window's Monte-Carlo error blows up
relative to the total.

**A secondary axis is reported but not gated**: the same five-window table in
`m(μμ)` with edges `0, 60, 86, 96, 200, 500` (below-Z continuum, low shoulder,
Z peak, high shoulder, the `m(μμ) → √s` non-radiative region). Reason: `pt(a)`
is a *smeared image* of the structure that carries the cross section — an on-Z
event can land anywhere in `pt(a) ∈ [39.4, 241.7]` depending on `η_a`, whereas
`m(μμ)` resolves the Breit–Wigner directly. If the `pt(a)` table localises
nothing, the `m(μμ)` table is what says whether that is because there is nothing
to localise or because `pt(a)` cannot see it. It carries no verdict of its own
(§D.6 keys only on `pt(a)`), which keeps the pre-registration honest.

### D.3 — the four estimators, and which error each carries

The asymmetry that matters: **a partition of one integration cannot audit that
integration**, because the windows sum to the total by construction. Only an
*independently re-surveyed* windowed integral can. So each side gets both, and
the closure test is the oracle.

**MadGraph, `MG-part(w)` — the B1 estimator.** The window imposed through
`dummy_cuts` (`SubProcesses/dummy_fct.f`), which `passcuts` applies after every
other cut and which leaves MadEvent's phase-space generation untouched, so a
windowed run integrates *the same integrand* restricted to `w`. Verified usable
here: the banked 3.7.1 `dummy_fct.f` has the same `logical FUNCTION dummy_cuts(P)`
signature and the same single `      dummy_cuts=.true.` marker line
`gen_higgs_window.sh` patches, and `leshouche.inc` gives
`DATA (IDUP(I,1,1),I=1,5)/-11,11,-13,13,22/`, so **the photon is external leg 5**
— to be asserted by the script, not assumed. Error: MadEvent's own quoted error
per run. **Closure:** `Σ_w MG-part(w)` must equal the unwindowed control. It need
not, and B1 is the precedent where it did not by 7.2σ.

**MadGraph, `MG-cut(w)` — the refocused estimator.** The window imposed as
run-card cuts `pta = lo`, `ptamax = hi`. This is *not* a substitute for
`MG-part`: `setcuts.f:301` feeds `ptamax` into `etmax(i)`, which the phase-space
generator reads, so the generator re-optimises for the window. That is precisely
what makes it a better estimate of the *true* windowed σ and precisely what
disqualifies it from the closure test. Run only on `W1` and `W5` (the two
extremes), as an independent check on whichever window the partition implicates.

**Ours, `VG-part(w)`.** Indicator accumulators on the production sampler at the
σ gate's own configuration (`neval 80_000`, `niter 8`, `MULTICHANNEL_SURVEY
30_000 × 6`), summing `w·1[pt(a) ∈ window]` and `w²` per window over the same
draws — the pattern
`validate_samples::the_higgs_pole_window_is_measured_against_madgraph` already
uses. Per-window MC error from the same-draw variance. Closure is **trivially
exact** and therefore carries no information; this estimator exists to give our
*shape* at exactly the configuration the gated σ comes from.

**Ours, `VG-cut(w)`.** An independent integration per window with a run card
carrying `pta = lo`, `ptamax = hi`. Both fields are already supported
(`runcard.rs:455/462`; `cuts.rs` reads `pt{c}max` for the photon class), and
because `pta` sets the process's fiducial scale, the channel maps and the VEGAS
grids genuinely re-adapt inside the window. **Closure:** `Σ_w VG-cut(w)` against
the unwindowed `VG` total, errors in quadrature — the mirror of MadGraph's, and
the only coverage audit our side gets.

**Third witness — MadGraph 3.5.7.** `pixi.toml:106` pins the packaged
`mg5amcnlo = "==3.5.7"` while the reference bank is generated from the pinned
`research/refs/mg5amcnlo` submodule at 3.7.1, so **both versions are runnable on
this machine**. The third witness is a 5-seed unwindowed control at 3.5.7 plus,
if a window is implicated, `MG-part(w)` at 3.5.7 in that window. This is what
turns "the reference moved" from an assertion into a measurement: two seed clouds
that overlap mean the reference did not move, it fluctuated.

**Statistics, all defined now.** With `Δ_w ≡ σ_VG(w)/σ_MG(w) − 1` and `ε_w` its
combined error:

- `χ²_flat ≡ Σ_w (Δ_w − Δ̄)² / ε_w²` on **4 dof**, `Δ̄` the inverse-variance mean.
  **Localised ⟺ `χ²_flat > 13.28`** (p < 0.01). Anything below is "not localised".
- `C_MG ≡ Σ_w MG-part(w) / MG-control − 1`; **MG closure fails ⟺ `|C_MG| > 3`
  combined σ.** Same form for `C_VG`.
- **Seed-consistent** ⟺ the 5-seed χ²/dof about the sweep mean is ≤ 2 on 4 dof.
- **Budget-stable** ⟺ under a 4× budget the quoted error shrinks by ≥ 1.7× *and*
  `|Δ_w(4×) − Δ_w(1×)| ≤ 2 ε_w(1×)`. Per AGENTS.md, a residual that migrates
  between seeds instead of shrinking under budget is a bug, not statistics.
- **Version-separated** ⟺ the 3.5.7 and 3.7.1 5-seed clouds are separated by more
  than 3× the combined seed spread (not the quoted per-run errors).

### D.4 — seed and budget protocol (binding, AGENTS.md)

- **Ours**: seeds `{20260719, 11, 22, 33, 44}` — the existing
  `probe_resonant_seed_stability` set, so the sweep is comparable to the recorded
  one. Two budgets: the gate's (`neval 80_000, niter 8`) and 4× (`neval
  320_000, niter 8`). `VG-cut(W1)` additionally gets 4× at the base budget
  because `ptamax = 20` rejects ≈92% of draws and its error would otherwise not
  be comparable.
- **MadGraph**: seeds `{20260803, 20260804, 20260805, 20260806, 20260807}`,
  `nevents = 100000` (the bank used 10000 for `2.335e-4`; MadEvent's refine
  targets scale with `nevents`, so expect ≈0.08%). The banked run's own
  `Cards/run_card.dat` and `Cards/param_card.dat` verbatim, with only `nevents`
  and `iseed` changed — the `gen_higgs_window.sh` discipline.
- **Report the spread and χ²/dof, never a headline pull.** A fixed-seed pull is
  not evidence on either side, and this chain's whole thesis is that it was not
  evidence on MadGraph's side either.

### D.5 — the runs, in order (each may stop the chain early)

**Run 0 — source verification (minutes, no compute).** Re-derive §D.0:
`sde_strategy` in the banked card and in a freshly generated 3.5.7 directory;
`TMIN_FOR_CHANNEL` in `Source/run_card.inc` for both; the early return in
`genps.f`'s `get_channel_cut`. Record the four facts with the grep output. If any
differs from §D.0, stop and report — the design's premise moved.

**Run 1 — the reference's own error honesty (≈15 min).** MadGraph 3.7.1
unwindowed, 5 seeds, `nevents = 100000`; then the same 5 seeds at
`nevents = 10000` (the bank's budget) so the sweep is directly comparable to the
banked number. Report spread vs quoted error, and where the banked `9.980100e-2`
sits in the cloud.

```bash
# in the chain worktree, backgrounded, log prefixed chainD_
pixi run -e madgraph bash validation/madgraph/gen_pta_windows.sh --stage control-371
```

**Run 2 — the third witness (≈15 min).** The same 5-seed unwindowed control from
the packaged MadGraph **3.5.7**. Together with Run 1 this decides
"version-separated".

```bash
pixi run -e madgraph bash validation/madgraph/gen_pta_windows.sh --stage control-357
```

**Run 3 — our own seed and budget sweep (≈30 min).** `VG-part` over the 5 seeds
at both budgets, producing `Δ_w` and the total.

```bash
cargo test -p vibegraph --test validate_samples \
  probe_pta_windows_against_madgraph -- --ignored --nocapture
```

**Run 4 — MadGraph's partition (≈60–90 min).** `MG-part(w)` for all five windows
via `dummy_cuts`, 3 seeds each, `nevents = 100000`; the closure test `C_MG`.

```bash
pixi run -e madgraph bash validation/madgraph/gen_pta_windows.sh --stage partition-371
```

**Run 5 — our partition audit (≈30 min).** `VG-cut(w)` for all five windows, 5
seeds, plus 3 seeds at 4× on whichever windows Run 3 or Run 4 implicates; the
closure test `C_VG`.

**Run 6 — conditional refocus (≈15 min).** `MG-cut` on `W1` and `W5`, and
`MG-part` at 3.5.7 on any window Run 4 implicates. Run only if Runs 1–5 leave the
verdict in D3–D6.

**Total expected cost ≈ 3 hours wall**, dominated by MadGraph and fully
parallelisable; the banked run's own `<cumulated_time>` is 10.3 s at
`nevents = 10000`, so this is a cheap measurement by this project's standards.
Everything over ~2 minutes is backgrounded with a `chainD_`-prefixed log, per the
worktree discipline.

### D.6 — THE PRE-REGISTERED DECISION RULE

Read top to bottom; the **first** row whose pattern holds is the verdict. Every
symbol is defined in §D.3. No other mapping is permitted, and in particular no
row may be reached by reasoning about which side "ought" to be right.

| # | observed pattern | verdict |
|---|---|---|
| **D1** | `C_MG` fails (>3σ) · `C_VG` holds · our side seed-consistent and budget-stable | **The reference owns it.** MadGraph's own partition of its own phase space does not close on its own errors — B1's shape, second occurrence. The `integrals` and `samples` tolerances **stay**, and the note records explicitly that this is not a loosened tolerance: the reference moved, we did not. E4 closed. |
| **D2** | `C_VG` fails · `C_MG` holds · MadGraph seed-consistent within and across versions | **We own it.** File a defect naming the window(s) carrying the closure failure and the size of the miss. The fix is **not** this chain — it spawns its own, and per §6 the sprint's budget question goes to the user. |
| **D3** | both `C_MG` and `C_VG` fail | **Two defects; chain D does not choose.** Report both closure failures per window; escalate to the manager. Neither tolerance moves in the meantime. |
| **D4** | both closures hold · localised (`χ²_flat > 13.28`) · in the implicated window our `Δ_w` is seed-inconsistent **or** not budget-stable | **We own it, localised.** A residual that migrates between seeds or fails to shrink under 4× budget is a bug (AGENTS.md). File the defect with the window and the migration recorded. |
| **D5** | both closures hold · localised · our `Δ_w` there is seed-consistent and budget-stable · MadGraph's value in that window is **version-separated** | **The reference owns it, localised.** The note names the window and the version dependence, and Run 6's `MG-part` at 3.5.7 in that window is the recorded evidence. Tolerances stay; same "not a loosened tolerance" statement as D1. |
| **D6** | both closures hold · localised · both sides stable in that window · **not** version-separated | **Localised but unattributed.** A per-window disagreement that is stable on both sides is not a sampling question; it points at the integrand — the cut boundary, the window definition, or something `amplitudes` (gated to 1e-11) cannot see. Escalate with the window named. Chain D does **not** demote the row; only the manager may. |
| **D7** | closures hold · **not** localised · `Δ̄ ≈ Δ_tot` · our side budget-stable · the two MadGraph versions' seed clouds **overlap** | **Localisation refuted, and the drift is the reference's error estimate rather than a shift.** The measurement's positive content: MadGraph's 3.5.7 and 3.7.1 numbers agree within their own seed spread (they already agree at 1.84σ on quoted errors) while the 3.7.1 run's quoted 0.234% understates its own spread. Verdict goes on the reference's side; tolerances stay; the note records the seed cloud, not a widened bound. |
| **D8** | closures hold · not localised · `Δ̄` shrinks with our budget by ≈√N and the 4× value is within `2ε` of zero | **Ours, and statistical rather than a defect.** No defect filed. Re-report the pull at the higher budget and record the budget dependence, so the row's number stops being read as a standing discrepancy. |
| **D9** | closures hold · not localised · `Δ̄` stable on both sides at every budget · versions agree | **A flat normalisation difference.** Not a coverage question at all. Escalate to the manager with the `amplitudes` cell (1e-11 at fixed points) as the constraint that rules out the matrix element, pointing instead at the cut boundary, the flux factor, or units. |
| **D10** | anything else, including any run that fails to produce its estimator | **No verdict.** Report the measurement as taken, state which pattern was expected and which was seen, and escalate. Inventing a verdict post hoc is the failure this table exists to prevent. |

Two standing riders on every row: (i) if Run 0 contradicts §D.0, the chain stops
before Run 1 and reports; (ii) if MadGraph 3.5.7 cannot be run on this machine,
the version axis is unavailable, **D5 and the version clause of D7 become
unreachable**, and their outcomes route to D6 and D10 respectively — recorded as
a degraded measurement, not silently absorbed.

### D.7 — gates afterwards, and where the record lands

**This chain changes no production code, so the expectation is that no report
cell moves at all.** That is itself the check:

```bash
cargo test --workspace                 # hermetic suite, unchanged
pixi run validate --skip-deps          # NEVER bare: a bare run can launch a
                                       # multi-hour MadGraph regeneration
git diff --stat validation-report/     # expected: empty
```

Any moved cell is a defect in this chain, not a finding. The
`extended-validation` gates are **not** run and the reason is recorded rather
than assumed: they cover amplitude, colour, coupling and diagram-enumeration
changes, and this chain touches none of those.

**Committed artifacts — authorised explicitly, and nothing beyond this list**
(the brief permits a small committed probe only if the design names it and says
where it lives):

1. `validation/madgraph/gen_pta_windows.sh` — the MadGraph driver, staged
   (`--stage control-371 | control-357 | partition-371 | refocus`). Precedent and
   template: `gen_higgs_window.sh`, whose leg-index assertion, run-card-verbatim
   discipline and "σ comes from `results.dat`, not from the event count" rule it
   inherits.
2. `validation/madgraph/pta_window_reference.json` — a few dozen scalars
   (per-window and per-seed σ ± err, both versions, both estimators). Committed
   like `higgs_window_reference.json` and `sigma_reference.json`: expensive to
   produce, stable, far too small for the fetched bundle.
3. `probe_pta_windows_against_madgraph` in `vibegraph-lib/tests/validate_samples.rs`,
   **`#[ignore]`d**, following `probe_resonant_seed_stability`. It stays ignored
   in this chain: the acceptance is the verdict, not a new gate, and promoting it
   to a live measurement (as B1's window test was promoted) costs suite runtime
   and is a manager decision recorded in the close-out.

**The record**: a `## Chain D measurement (date)` section appended to this note,
carrying the full per-window table with every cell a recorded measurement — no
cell inferred from "the suite passed" — the closure statistics, the seed clouds,
the decision-rule row that fired, and the verdict. On a D1/D5/D7 verdict the
`ee_to_mumua` `integrals` and `samples` notes in `validation/manifest.toml` are
rewritten to say which side is wrong and how it was measured, exactly as B1 did.
TODO's standing-discrepancy entry is the manager's to rewrite, not this chain's.

### D.8 — risks, and what this measurement provably cannot decide

**Risks.**

- The `dummy_cuts` patch must land on the 3.7.1 body (verified above: same
  signature, one marker line) and the photon must be leg 5 (verified in
  `leshouche.inc`). Both are assertions in the script, not assumptions — a silent
  mismatch would window the wrong leg and produce a confidently wrong table.
- MadEvent may fail to fill `nevents` in a narrow window. σ is read from
  `SubProcesses/results.dat`, which is survey+refine and independent of the event
  count; a short event file is not a failure.
- `VG-cut(W1)` rejects ≈92% of draws through `ptamax`; without the 4× budget its
  error would silently dominate `C_VG` and make our closure test vacuous.
- The 3.5.7 environment may not build or generate here; §D.6's rider covers the
  degradation rather than leaving it to judgement.
- `pt(a)` must be computed identically on both sides. Ours comes through
  `lhef::observables`; MadGraph's window is Fortran in the rest frame. For fixed
  beams these frames coincide, but the script asserts it rather than relying on
  it.

**Blind spots — the error classes this measurement provably cannot detect.**

- **Anything both sides get wrong the same way inside a window.** The two sides
  share the matrix element (gated to 1e-11 by the `amplitudes` cell at fixed
  points) and share the window definition. This is a statement about phase-space
  coverage and cut boundaries, not about the matrix element — B1's blind spot,
  inherited unchanged.
- **Our closure test is structurally weaker than MadGraph's.** `VG-cut(w)`
  re-adapts grids and fiducial scale but reuses the same channel construction and
  the same map code as the unwindowed run, so a defect in that shared code can
  survive in both and let `C_VG` pass. MadGraph's windowed runs re-survey with a
  genuinely different channel allocation, so `C_MG` is the stronger oracle. A D2
  verdict is therefore *harder* to reach than a D1 — the asymmetry is in the
  reference's favour, which is the safe direction but must be stated.
- **`W1` cannot separate "the cut boundary is implemented differently" from "the
  region is mis-covered"**, because its lower edge *is* the `pta = 10` cut. A
  disagreement localised in `W1` alone routes to D6, not to a coverage verdict.
- **The σ verdict does not automatically own the `samples` KS cell.** KS is
  shape-only and normalisation-free; a flat `Δ_w` (D7–D9) would leave the
  `pt(a)` KS failure unexplained. The per-window shape table is the evidence that
  speaks to the KS cell; the σ verdict is not.
- **A compensating error that cancels in the `pt(a)` projection is invisible** —
  `η(a)` is integrated over inside each window. The `m(μμ)` secondary axis
  covers part of this, but a defect orthogonal to both projections is not
  reachable by this design.
- **Nothing here tests** the scale prescription (fixed EW couplings), PDFs (fixed
  beams), or polarisation — all inert for this process, which is why the
  measurement can be read as a pure coverage statement.

## Chain D measurement (2026-08-03)

Implementation session output. Every number below is a recorded measurement: the
command that produced it is named, and no cell is inferred from another cell or
from a suite passing. §D.6's decision rule was applied top to bottom without
reference to which side "ought" to be right; §D.6.1 records exactly which clauses
were marginal.

Driver: `validation/madgraph/gen_pta_windows.sh` (stages `control-371`,
`control-357`, `partition-371`, `refocus`). This side:
`probe_pta_windows_against_madgraph` in `vibegraph-lib/tests/validate_samples.rs`,
run as

```
cargo test -p vibegraph-lib --features extended-validation --test validate_samples \
  probe_pta_windows_against_madgraph -- --ignored --nocapture
```

(the design's §D.5 Run 3 command omits `-lib` and the required feature; the test
target is feature-gated and does not build without it).

### D.M0 — Run 0: §D.0's premise re-verified, all four facts

Nothing moved. The channel-weight mechanism the chain brief named is unreachable
for this process in *both* MadGraph lines.

1. `sde_strategy = 1` in the banked cards:

   ```
   Cards/run_card.dat:86:  1	= sde_strategy ! default integration strategy (hep-ph/2021.00773)
   Cards/run_card_default.dat:85:   1  = sde_strategy  ! default integration strategy (hep-ph/2021.00773)
   ```

2. `TMIN_FOR_CHANNEL = -1` in the banked generated include, alongside the
   strategy the run actually used:

   ```
   Source/run_card.inc:333:      TMIN_FOR_CHANNEL = -1.000000000000000D+00
   Source/run_card.inc:337:      SDE_STRAT = 1
   ```

   Re-read off `Source/run_card.inc` after *every* run this chain made
   (`run_one` prints it): `TMIN_FOR_CHANNEL=-1.000000000000000D+00 SDE_STRAT=1`
   without exception.

3. The early return is present and identical in both lines —
   `.pixi/envs/madgraph/MG5_aMC/Template/LO/SubProcesses/genps.f:1858`,
   `research/refs/mg5amcnlo/Template/LO/SubProcesses/genps.f:1878`, and the
   banked `SubProcesses/genps.f:1878`:

   ```fortran
         if(sde_strat.eq.1.and.tmin_for_channel.eq.-1)then
            get_channel_cut = 1d0
            return
         endif
   ```

   so `get_channel_cut` is identically 1 here and never reaches either
   expression the 3.5.7 → 3.7.1 fix touched.

4. Freshly generated directories auto-select the same strategy in both lines
   (printed by `generate_template`):

   ```
   >>> [3.7.1] auto-selected sde_strategy:    1  = sde_strategy ...
   >>> [3.5.7] auto-selected sde_strategy:    1  = sde_strategy ...
   ```

   The only change to the selection rule between versions is the guard
   `proc_characteristic['gauge'] != 'FD' and` prepended at `banner.py:4995`
   (3.7.1) relative to `banner.py:4684` (3.5.7); `proc_characteristics` gives
   `single_color = True`, `gauge = unitary`, so the branch is entered either way
   and falls through `pure_lepton and proton_initial` (the initial state is
   `e+ e-`) to `elif not no_qcd`.

Two further equivalence facts, asserted by the driver rather than assumed:

* `leshouche.inc` gives `DATA (IDUP(I,1,1),I=1,5)/-11,11,-13,13,22/` in the bank
  and in both freshly generated templates — **the photon is external leg 5**, and
  the `dummy_cuts` window cuts on `p(1,5), p(2,5)`.
* `configs.inc`, `props.inc` and `leshouche.inc` in the 3.7.1 template are
  byte-identical to the bank's. The 3.5.7 template differs from the bank in
  `configs.inc` by two lines only — `C     used fake id` / `DATA FAKE_ID/7/`,
  which 3.5.7 does not emit — and is identical in `props.inc`, `leshouche.inc`
  and `maxamps.inc`. **Both lines build the same six-channel decomposition**, so
  the version comparison is a comparison of integrators, not of processes.

The banked per-channel cross sections in §D.1 were re-read off
`SubProcesses/P1_ll_lla/G*/results.dat` and match that table exactly.

The chain proceeded.

### D.M1 — Run 1: the reference's own error honesty

`pixi run -e madgraph bash validation/madgraph/gen_pta_windows.sh --stage control-371`,
five seeds, and the same again with `NEVENTS=10000 TAG_SUFFIX=_n10k`.

| budget | σ per seed (pb) | mean | quoted/run | seed sd | cloud χ²/dof |
|---|---|---|---|---|---|
| `nevents = 100000` | 9.9966e-2, 9.9956e-2, 1.00010e-1, 1.00110e-1, 9.9901e-2 | `9.998860e-2` | 8.398e-5 | 7.817e-5 | **0.90** |
| `nevents = 10000` (the bank's) | 9.9845e-2, 9.9917e-2, 1.00090e-1, 1.00010e-1, 9.9712e-2 | `9.991480e-2` | 2.569e-4 | 1.464e-4 | **0.32** |

**§D.0's central hypothesis is refuted.** MadGraph's quoted error on this process
is honest at the 100k budget (χ²/dof 0.90; spread 7.8e-5 against a quoted 8.4e-5)
and *conservative* at the bank's own 10k budget (χ²/dof 0.32). The banked
`9.980100e-2` sits 0.78 seed-sd below its own 10k cloud mean — an ordinary draw,
not an outlier. Whatever the +0.80% is, it is not a reference whose error bar
fails to cover its own spread.

### D.M2 — Run 2: the third witness, MadGraph 3.5.7

`--stage control-357`, five seeds, `nevents = 100000`. The 3.5.7 environment ran
here without difficulty, so §D.6's degradation rider was **not** invoked and the
version axis is fully available.

| | σ per seed (pb) | mean | quoted/run | seed sd | cloud χ²/dof |
|---|---|---|---|---|---|
| 3.5.7 | 1.00200e-1, 9.9949e-2, 9.9943e-2, 1.00120e-1, 1.00100e-1 | `1.000624e-1` | 9.556e-5 | 1.127e-4 | 1.40 |

3.5.7 − 3.7.1 = `+7.380e-5` pb = **+0.074%**, `+1.20` combined-seed-spread σ,
against a separation criterion of 3× the combined spread (`1.840e-4`).

> **Version-separated: NO. The two clouds overlap.**

The two MadGraph lines agree on this process to 0.074%. The apparent 0.82%
"drift" between reference banks is two single draws from two clouds that overlap;
it is not a version effect, and §D.5's `--stage partition-357` was therefore not
needed.

### D.M3 — Run 3: `VG-part`, our shape at the gate's own configuration

Five seeds at `neval 80 000 × niter 8` (the σ gate's budget for this row), plus
three seeds at 4×. The measurement is a frozen pass over the grids `adapt_grids`
trains, on a ChaCha stream disjoint from the integration's. Self-check: at seed
`20260719` the run's own `adapt_grids` total is `1.006000e-1`, bit-for-bit the
banked gate value, so the integrand measured is the gated one.

Seed clouds (mean, quoted error on the mean, seed-spread error, χ²/dof on 4 dof):

| | σ (pb) | quoted | spread | χ²/dof |
|---|---|---|---|---|
| total | `1.006713e-1` | 9.961e-5 | 7.969e-5 | **0.69** |
| `[10, 20)` | `9.315376e-3` | 6.869e-5 | 8.195e-5 | 1.23 |
| `[20, 39.4)` | `9.865348e-3` | 4.085e-5 | 3.005e-5 | 0.62 |
| `[39.4, 77)` | `2.785841e-2` | 5.343e-5 | 5.683e-5 | 1.10 |
| `[77, 144)` | `2.728223e-2` | 4.145e-5 | 4.137e-5 | 0.99 |
| `[144, 250)` | `2.634998e-2` | 3.758e-5 | 3.364e-5 | 0.80 |

> **Seed-consistent: YES** — every cloud is at or under χ²/dof 1.23, well inside
> the ≤ 2 criterion.

Budget stability, base against 4× on the three shared seeds:

| | base | 4× | err shrinks | shift | `2 ε(1×)` | stable? |
|---|---|---|---|---|---|---|
| total | `1.005657e-1` ± 1.166e-4 | `1.007437e-1` ± 5.628e-5 | 2.07× | +0.177% | 0.232% | **yes** |
| `[10, 20)` | `9.248053e-3` ± 7.075e-5 | `9.471996e-3` ± 3.731e-5 | 1.90× | **+2.422%** | 1.530% | **no** |
| `[20, 39.4)` | `9.870744e-3` ± 4.909e-5 | `9.826276e-3` ± 2.144e-5 | 2.29× | −0.451% | 0.995% | yes |
| `[39.4, 77)` | `2.786214e-2` ± 7.068e-5 | `2.790989e-2` ± 3.243e-5 | 2.18× | +0.171% | 0.507% | yes |
| `[77, 144)` | `2.728244e-2` ± 5.391e-5 | `2.723621e-2` ± 2.660e-5 | 2.03× | −0.169% | 0.395% | yes |
| `[144, 250)` | `2.630229e-2` ± 4.867e-5 | `2.629937e-2` ± 2.406e-5 | 2.02× | −0.011% | 0.357% | yes |

> **Budget-stable on the total: YES.** Budget-stable per window: yes in W2–W5,
> **no in W1**, where the base-budget frozen pass sits 2.4% low. W1 is the window
> that keeps ~9% of the multichannel draws, and its 4× value moves *towards* both
> MadGraph's own windowed cross section and this side's independent `VG-cut(W1)`
> (below) rather than away from either — the error also shrinks by 1.90×, i.e.
> ≈ √4. The residual converges under budget; it does not migrate between seeds.

### D.M4 — Run 4: `MG-part`, and the closure test `C_MG`

`--stage partition-371`, five windows × **five** seeds (the design's §D.5 asked
for three; §D.4's seed protocol names five, and `C_MG` is the statistic the whole
verdict turns on, so the full five were run), `nevents = 100000`.

| window | σ per seed (pb) | mean | quoted (mean) | seed sd | cloud χ²/dof |
|---|---|---|---|---|---|
| `[10, 20)` | 9.4130e-3, 9.4426e-3, 9.3615e-3, 9.4182e-3, 9.3969e-3 | `9.406440e-3` | 6.242e-6 | 3.000e-5 | **4.27** |
| `[20, 39.4)` | 9.8537e-3, 9.8383e-3, 9.8285e-3, 9.8265e-3, 9.8052e-3 | `9.830440e-3` | 6.948e-6 | 1.774e-5 | 1.33 |
| `[39.4, 77)` | 2.7516e-2, 2.7520e-2, 2.7607e-2, 2.7489e-2, 2.7480e-2 | `2.752240e-2` | 1.267e-5 | 5.029e-5 | **3.51** |
| `[77, 144)` | 2.7074e-2, 2.7014e-2, 2.7003e-2, 2.7079e-2, 2.7066e-2 | `2.704720e-2` | 1.274e-5 | 3.584e-5 | 1.53 |
| `[144, 250)` | 2.6320e-2, 2.6316e-2, 2.6317e-2, 2.6308e-2, 2.6317e-2 | `2.631560e-2` | 7.717e-6 | 2.015e-6 | 0.07 |

A finding in its own right: **MadEvent's quoted error on a `dummy_cuts`-windowed
run understates its own seed spread**, by 2.1× in `[10, 20)` and 1.9× in
`[39.4, 77)` (χ²/dof 4.27 and 3.51 on 4 dof). The unwindowed runs of D.M1 do not
show this. Whatever the windowed error estimator is doing, it is not covering the
seed spread when a large fraction of generated points is rejected after the fact.

```
C_MG: sum of windows 1.001221e-1 +- 2.167e-5 against unwindowed 9.998860e-2 +- 3.758e-5
      -> +0.133% +- 0.043%, +3.07 sigma
```

> **`C_MG` fails: +3.07 σ** on the quoted errors combined — the form §D.3
> pre-registers and the form B1 was measured in (7.2σ there). On the *seed-spread*
> errors instead it is +2.82σ, i.e. just inside. **The clause is marginal and
> both readings are recorded**; see §D.M8.

### D.M5 — Run 5: `VG-cut`, and the closure test `C_VG`

Five seeds per window at `neval 80 000 × niter 8`, with `[10, 20)` at 4× because
`ptamax = 20` rejects ~92% of draws.

| window | σ (pb) | quoted | spread | χ²/dof |
|---|---|---|---|---|
| `[10, 20)` (4×) | `9.425007e-3` | 1.804e-5 | 1.646e-5 | 0.83 |
| `[20, 39.4)` | `9.776201e-3` | 2.393e-5 | 2.840e-5 | 1.42 |
| `[39.4, 77)` | `2.784266e-2` | 2.951e-5 | 4.193e-5 | **2.06** |
| `[77, 144)` | `2.717872e-2` | 2.087e-5 | 2.334e-5 | 1.23 |
| `[144, 250)` | `2.627963e-2` | 1.294e-5 | 1.993e-5 | **2.41** |

```
C_VG: sum of windows 1.005022e-1 +- 4.870e-5 against unwindowed 1.006713e-1 +- 9.961e-5
      -> -0.168% +- 0.110%, -1.53 sigma
```

> **`C_VG` holds: −1.53 σ.** Five independent re-surveys, each with its own
> fiducial scale, channel maps and VEGAS grids, reproduce the unwindowed integral.
> Two of the five per-window seed clouds sit marginally over the ≤ 2 χ²/dof
> criterion (2.06 and 2.41); on 4 dof the χ²/dof estimator's own sd is 0.71, so
> both are ≈ 2σ high and neither is a failure at any conventional level. Recorded,
> not smoothed.

### D.M6 — Run 6: not run, and why

§D.5 makes Run 6 conditional — "*Run only if Runs 1–5 leave the verdict in
D3–D6*". They did not (§D.M8). Additionally, Run 6 as designed refocuses `W1` and
`W5`, and the window the partition actually implicates is `W3` (§D.M7), which the
stage as specified would not have measured. `--stage refocus` is implemented,
tested to the point of stage dispatch, and left unrun; `MG-cut(W3)` is filed as
the follow-up in §D.M9.

### D.M7 — `Δ_w`, `χ²_flat`, and a shape contradiction internal to MadGraph

`Δ_w ≡ VG-part(w)/MG-part(w) − 1`:

| window | `Δ_w` | `ε_w` | pull |
|---|---|---|---|
| `[10, 20)` | **−0.968%** | 0.733% | −1.32 |
| `[20, 39.4)` | +0.355% | 0.422% | +0.84 |
| `[39.4, 77)` | **+1.221%** | 0.200% | **+6.11** |
| `[77, 144)` | **+0.869%** | 0.160% | **+5.42** |
| `[144, 250)` | +0.131% | 0.146% | +0.90 |

```
inverse-variance mean Delta_bar +0.597%, chi2_flat 27.75 on 4 dof (localised iff > 13.28)
Delta_tot +0.683% +- 0.107%
```

> **Localised: YES** (`χ²_flat = 27.75`, p ≈ 1.4e-5). Recomputing with MadGraph's
> *spread*-based per-window errors instead of its quoted ones gives `χ²_flat =
> 24.9`, so the conclusion does not rest on the error estimator D.M4 just showed
> to be optimistic. The disagreement is not flat: it is concentrated in
> `[39.4, 77)` and `[77, 144)`, the radiative-return turn-on and bulk, and is
> consistent with zero in the two lowest and the highest window.

**The finest oracle in this chain is not a cross section at all.** MadGraph banks
an unweighted event sample with every run, all events carrying an identical
`XWGTUP` (verified: 1 distinct weight in both the bank and a fresh 100k run, mean
= σ), so the sample's `pt(γ)` fractions estimate the *same* σ shares the windowed
runs measure. They disagree, internally to MadGraph:

| window | `MG-part` share | `VG-cut` share | `VG-part` share (4×) | MG **unwindowed sample** share (5 × 100k) | MG-part − sample |
|---|---|---|---|---|---|
| `[10, 20)` | 9.395% | 9.378% | 9.402% | **8.732% ± 0.033%** | **+0.663 pp, +19.9 σ** |
| `[20, 39.4)` | 9.818% | 9.727% | 9.754% | 9.549% ± 0.060% | +0.269 pp, +4.5 σ |
| `[39.4, 77)` | 27.489% | 27.704% | 27.704% | 27.859% ± 0.049% | −0.370 pp, −7.6 σ |
| `[77, 144)` | 27.014% | 27.043% | 27.035% | 27.319% ± 0.050% | −0.305 pp, −6.1 σ |
| `[144, 250)` | 26.284% | 26.148% | 26.105% | 26.541% ± 0.081% | −0.258 pp, −3.2 σ |

(the banked 10k sample gives 8.330% ± 0.276% in `[10, 20)`, consistent with the
100k sample cloud and 3.9σ from `MG-part`; the 3.5.7 samples give 8.805% and
8.655%, i.e. the same deficit in the other MadGraph line.)

MadGraph's unweighted event sample moves ≈0.9 pp of the cross section out of the
two lowest `pt(γ)` windows and into the three radiative-return windows, relative
to MadGraph's own windowed cross sections for those same regions. **This side's
two independent estimators land on MadGraph's windowed numbers, not on
MadGraph's sample** — `VG-cut(W1)/Σ VG-cut = 9.378%` and
`VG-part(W1)/total = 9.402%` against `MG-part` 9.395% and the MG sample 8.732%.

This is not a threshold call: it is a ~20σ contradiction between two objects
MadGraph produces from one run, it reproduces across five seeds and both MadGraph
versions, and the two sides' *integrals* of that window agree to
`+0.20% ± 0.24%` (`VG-cut(W1) = 9.425007e-3` against `MG-part(W1) =
9.406440e-3`). The `samples` gate compares our events against exactly the object
that is the outlier, which is what the `pt(a)` KS cell at p = 2.74e-4 has been
seeing.

### D.M8 — the decision rule, applied

Read top to bottom.

| row | clause | measured | holds? |
|---|---|---|---|
| **D1** | `C_MG` fails (> 3σ) | +3.07 σ (quoted errors, §D.3's form); +2.82 σ (seed-spread errors) | **yes** (marginal) |
| | `C_VG` holds | −1.53 σ | **yes** |
| | our side seed-consistent | total χ²/dof 0.69; `VG-part` ≤ 1.23 in every window; `VG-cut` ≤ 1.42 except 2.06 and 2.41 | **yes** (two marginal) |
| | our side budget-stable | total +0.177% against a 0.232% bound, error ×2.07; W2–W5 stable; **W1 +2.422% against a 1.530% bound** | **yes at the side level, no for W1** |

**D1 fires. Verdict: the reference owns it.**

Every row below D1 requires either `C_VG` to fail (D2, D3) or *both* closures to
hold (D4–D9), so with `C_MG` failing and `C_VG` holding the table admits only D1
or D10. The clauses above are stated at the level §D.6 states them: §D.3 defines
seed-consistency without a window index, and D4 — not D1 — is the row that
carries the per-window form ("*in the implicated window our `Δ_w`*"). Under a
strict per-window reading of D1's "budget-stable", W1's failure would push the
chain to D10; that reading is recorded here so the manager can overrule, and
§D.M9 flags it.

What makes D1 more than a threshold call is that its verdict is independently
confirmed by a measurement no threshold enters: §D.M7's ~20σ contradiction
between MadGraph's own event sample and MadGraph's own windowed cross sections
for the same region, with this side on the windowed side of it. The σ closure
failure (+0.133%) and the shape contradiction have the same sign and the same
location — MadGraph's unwindowed run under-represents low `pt(γ)`.

**This is not a loosened tolerance.** No tolerance moved in this chain and none is
proposed. `ee_to_mumua`'s `integrals` `rel_tol` stays at 0.03 and the `samples`
p-floor stays at 1e-4. What changed is the record of *which side* the residual
sits on and how that was measured.

**Second occurrence of B1's shape.** As with `ee_to_mumu_tata_qcd0`, MadGraph's
partition of its own phase space exceeds its own unwindowed integral by more than
its own quoted errors allow. The mechanism is different — B1's was a
`get_channel_cut` defect specific to 3.5.7 and `sde_strategy = 2`, refuted for
this process in §D.M0 — and the size is 17× smaller (+0.13% against +2.3%).

### D.M9 — what this chain did **not** settle

1. **The localised `W3`/`W4` excess is unexplained and survives the verdict.**
   `Δ_3 = +1.221% ± 0.200%` and `Δ_4 = +0.869% ± 0.160%` — this side above
   MadGraph in the radiative-return turn-on and bulk, at 6.1σ and 5.4σ, confirmed
   by `VG-cut` (`+1.164%` and `+0.486%` in the same windows) and not removed by
   inflating MadGraph's errors to its seed spread. D1's verdict does not account
   for it. A candidate that this chain could not test: `W3`'s lower edge, 39.4
   GeV, is the radiative-return kinematic turn-on, and `MG-part` reaches it
   through a generator that does not know the window exists while `VG-cut` sets
   it as a real `pta` cut the maps adapt to. **Recommended follow-up:
   `MG-cut(W3)` via `--stage refocus WINDOWS=3`, which the committed driver
   already supports** — if it lands on `VG-cut(W3)`, the residual is
   `MG-part`'s window-blind sampling; if it lands on `MG-part(W3)`, it is ours.
2. **`C_MG` is marginal**: +3.07σ on quoted errors, +2.82σ on seed-spread errors,
   against a 3σ threshold. The verdict does not rest on it alone (§D.M7), but the
   statistic on its own would not carry a verdict.
3. **`VG-part(W1)` is not budget-stable** at the base budget. It converges under
   4× toward both independent estimates rather than migrating, which per AGENTS.md
   reads as sampling rather than a defect, but the pre-registered inequality is
   violated and this is the one clause where a stricter reading changes the row.
4. **The mechanism of MadGraph's sample/integral shape contradiction is not
   diagnosed.** A lead, not a conclusion: in a fresh 100k control the per-channel
   σ shares (`G1` 12.10%, `G7` 16.15%) and the per-channel written-event shares
   (`G1` 9.69%, `G7` 11.95%) differ substantially, and `G1`/`G5`/`G7` are the
   γ\*-mapped channels that carry low `pt(γ)`. Whether MadEvent's combination step
   is what moves the shape was not established and is out of this chain's scope.
5. **Blind spots, unchanged from §D.8**: anything both sides get wrong the same
   way inside a window (they share the matrix element, gated to 1e-11); `W1`'s
   lower edge is the `pta` cut, so a disagreement confined there cannot separate
   a cut-boundary convention from a coverage miss; `η(γ)` is integrated over
   inside each window. The `m(μμ)` secondary axis of §D.2 was not measured —
   `pt(γ)` localised the disagreement on its own, so the axis that exists to say
   "there was nothing to localise" was not needed.

### D.M10 — gates after the measurement

This chain changed no production code, and no report cell moved.

```
$ cargo test --workspace
   ... 19 test binaries, all ok
   test result: ok. 606 passed; 0 failed; 8 ignored ...   (vibegraph-lib unit)
   === WORKSPACE EXIT 0 ===
```

`pixi run validate --skip-deps` — never bare, which would launch a multi-hour
MadGraph regeneration — and `git diff --stat validation-report/` are recorded in
the session report. The `extended-validation` gates were **not** run: they cover
amplitude, colour, coupling and diagram-enumeration changes, and this chain
touched none of those.

### W3 refocus supplement (2026-08-03, authorised after the verdict)

Run after D1 was upheld, to settle §D.M9 item 1 — the localised `[39.4, 77)`
excess that survives the verdict. `MG-cut(W3)` is MadGraph's *re-surveyed*
integral of the same window: the run card carries `pta = 39.4`, `ptamax = 77.0`
and `dummy_fct.f` is left stock, so `setcuts.f` feeds the upper edge into
`etmax(i)` and the phase-space generator adapts to the window instead of having
it applied after the fact. It is therefore the estimator that discriminates
between "`MG-part(W3)` is low because its generator is window-blind at a
kinematic turn-on" and "the two integrands genuinely disagree there".

```
WINDOWS=3 SEEDS="20260803 20260804 20260805" \
  pixi run -e madgraph bash validation/madgraph/gen_pta_windows.sh --stage refocus
```

**The three-way interpretation, fixed before the measurement:** if `MG-cut(W3)`
lands on `MG-part(W3)`, the excess is a real disagreement between the two
integrands in the radiative-return turn-on, surviving D1 and changing what the
row's note should say (but not the verdict); if it lands on `VG-cut(W3)` /
`VG-part(W3)`, `MG-part(W3)` is the artifact and the excess dissolves into D1's
existing story; if it lands between them with errors too wide to discriminate,
that is recorded and no third measurement is taken.

| | σ (pb) | quoted (mean) | seed spread (mean) | cloud χ²/dof |
|---|---|---|---|---|
| **`MG-cut(W3)`, 3 seeds** | **`2.759167e-2`** | 1.704e-5 | 3.367e-5 | 4.17 |
| `MG-part(W3)`, 5 seeds | `2.752240e-2` | 1.267e-5 | 2.249e-5 | 3.51 |
| `VG-cut(W3)`, 5 seeds | `2.784266e-2` | 2.951e-5 | 4.193e-5 | — |
| `VG-part(W3)`, 5 seeds | `2.785841e-2` | 5.343e-5 | 5.683e-5 | — |

per-seed: `2.7659e-2, 2.7558e-2, 2.7558e-2` — three independent runs, confirmed
by their banners (`20260803/4/5 = iseed`), their point counts (5 848 448 /
4 540 128 / 4 039 108) and their distinct quoted errors. The two equal entries
agree only to the five significant digits `results.dat` prints, a resolution
≈60× finer than the seed spread.

| `MG-cut(W3)` − | Δ | quoted σ | seed-spread σ |
|---|---|---|---|
| `MG-part(W3)` | **+0.252%** | +3.26 | **+1.71** |
| `VG-cut(W3)` | **−0.901%** | −7.37 | **−4.67** |
| `VG-part(W3)` | −0.957% | −4.76 | −4.04 |

> **Branch 1 fired: `MG-cut(W3)` lands with `MG-part(W3)`, not with this side.**
> On the seed-spread errors — the honest ones here, since both MadGraph windowed
> clouds have χ²/dof ≈ 3.5–4.2 — it is consistent with `MG-part(W3)` at 1.71σ and
> inconsistent with `VG-cut(W3)` at 4.67σ. On MadGraph's optimistic quoted errors
> it is 3.26σ from `MG-part` and 7.37σ from `VG-cut`, i.e. decisively nearer
> `MG-part` on either basis. Re-surveying moves MadGraph's W3 value up by
> +0.252%, closing only **22% of the 1.164% gap** to `VG-cut(W3)`: window-blind
> sampling at the turn-on is a real but small part of it, and it is not the
> explanation.

**Consequence.** The `[39.4, 77)` and `[77, 144)` excess is a genuine
disagreement between the two integrands in the radiative-return region, ~0.9–1.2%
with this side high, and it **survives D1**. It is not a sampling artifact of
`MG-part`'s window-blindness, and it is not covered by D1's verdict, whose
evidence is the low-`pt(γ)` coverage miss in MadGraph's unwindowed run and its
event sample. Two separate effects live in this row's +0.80%, with opposite
locations:

* low `pt(γ)` — the reference under-covers, measured at 19.9σ against its own
  windowed cross sections (§D.M7). D1.
* radiative-return turn-on and bulk — this side sits ~1% above MadGraph in a
  comparison where both sides re-survey the window, 4.7σ. **Open, unattributed,
  and not explained by D1.** Per §D.6 an unattributed localised residual on which
  both sides are stable is D6 territory; chain D does not demote the row and only
  the manager may act on it. What would falsify "ours": an `m(μμ)`-axis
  measurement (§D.2's unmeasured secondary axis) showing the excess sits off the
  Z peak, or a per-channel comparison against `G6`/`G8`'s banked terms, which
  carry 53.6% of σ and dominate exactly this region.

### m(mumu) secondary axis

§D.2's secondary axis, frozen there with edges `0, 60, 86, 96, 200, 500` and
explicitly carrying no verdict. Dropped for time during the main measurement; run
afterwards because the D6-class `W3`/`W4` finding needs a discriminator, and
`pt(γ)` cannot supply one — `η(γ)` smears an on-shell-Z event across most of the
`pt(γ)` range, whereas `m(μμ)` resolves the Breit–Wigner directly.

**Pre-registered before running, and binding on the reading below: whatever this
table shows it maps to NO decision-rule row. It is localisation evidence for the
D6 finding and nothing else. D1 stands, the `W3`/`W4` item stays a recorded
D6-class subsidiary finding, and no verdict in this chain moves.**

`MG-part` via `dummy_cuts` on externals 3 and 4 (`IDUP = -13, 13`, asserted from
`leshouche.inc`), 3 seeds × `nevents = 100000` — this axis carries no verdict, so
three seeds suffice. `VG-part` is accumulated on the *same draws* as the `pt(γ)`
split, five seeds at the gate budget, so any difference between the two tables is
the projection and not the sample.

```
WINDOWS="1 2 3 4 5" SEEDS="20260803 20260804 20260805" \
  pixi run -e madgraph bash validation/madgraph/gen_pta_windows.sh --stage mll-371
```

| `m(μμ)` | `MG-part` (3 seeds) | `VG-part` (5 seeds) | `Δ` | quoted σ | spread σ | MG share |
|---|---|---|---|---|---|---|
| `[0, 60)` | `5.431867e-3` ± 3.341e-6 | `5.428301e-3` ± 2.308e-5 | −0.066% | −0.15 | −0.19 | 5.40% |
| `[60, 86)` | `4.510933e-3` ± 1.569e-6 | `4.498985e-3` ± 2.554e-5 | −0.265% | −0.47 | −0.41 | 4.48% |
| **`[86, 96)` (Z peak)** | `4.408767e-2` ± 1.289e-5 | `4.414335e-2` ± 4.478e-5 | **+0.126%** | **+1.19** | **+1.14** | **43.80%** |
| `[96, 200)` | `1.171800e-2` ± 4.570e-6 | `1.172257e-2` ± 3.565e-5 | +0.039% | +0.13 | +0.12 | 11.64% |
| `[200, 500)` | `3.490667e-2` ± 1.867e-5 | `3.487814e-2` ± 8.092e-5 | −0.082% | −0.34 | −0.36 | 34.68% |

Seed clouds are healthy on both sides on this axis (MadGraph χ²/dof 0.32–1.84,
ours 0.63–1.46) — none of the 3.5–4.3 inflation the `pt(γ)` windowed clouds
showed.

> **The two integrands agree in every `m(μμ)` window**, worst pull 1.19σ, and the
> largest single window — the Z peak carrying 43.8% of σ — agrees to
> `+0.126% ± 0.106%`. Summed: `VG 1.006713e-1` against `Σ MG-part(m) 1.006551e-1`,
> **`+0.016% ± 0.107%`**.

**The `W3`/`W4` question is answered: the excess sits OFF the resonance's
normalisation.** `[86, 96)` is where a resonance-mapping or width disagreement
would have to appear, and it agrees to a tenth of a percent. So the ~0.9–1.2%
`pt(γ)` excess in `[39.4, 77)` and `[77, 144)` is not the Z propagator, not the
width, and not the radiative-return normalisation.

**And the axis exposes something larger, which needs recording even though it
keys nothing.** Both partitions cover the phase space completely (`pt(γ) ∈
[10, 250)` from the cut to `√s/2`; `m(μμ) ∈ [0, 500)` from threshold to `√s`), and
both are imposed the same way, through `dummy_cuts` after every other cut. They
do not agree with each other:

| | Δ | σ |
|---|---|---|
| `Σ MG-part(pt_a)` − MG control | +0.134% | **+3.08** |
| `Σ MG-part(m_mumu)` − MG control | **+0.667%** | **+15.05** |
| `Σ MG-part(m_mumu)` − `Σ MG-part(pt_a)` | +0.532% | **+16.70** |
| VG total − `Σ MG-part(m_mumu)` | +0.016% | +0.16 |
| VG total − MG control | +0.683% | +6.41 |

Two complete `dummy_cuts` partitions of one MadGraph run's own phase space differ
from each other by **16.7σ**, and both exceed the unwindowed control. This is a
statement about MadGraph that needs no reference to this side at all. It is D1's
own signature — the reference's partition exceeding its own unwindowed integral —
at **five times the size** the `pt(γ)` axis measured, and it recovers essentially
exactly this side's number: **when MadGraph re-integrates its own phase space in
`m(μμ)` slices, the entire +0.68% disagreement disappears (+0.016%, 0.16σ).**

For whoever picks up the D6 item, that reframes it. Windowing in `pt(γ)` — by
`dummy_cuts` *or* by run-card `pta`/`ptamax`, which the W3 refocus showed give the
same answer — leaves MadGraph low in the radiative-return region; windowing the
same phase space in `m(μμ)` does not. The discriminating variable is which
observable the window is cut in, not whether the generator re-surveys. That points
at MadGraph's coverage in the `pt(γ)`/`η(γ)` plane rather than at either side's
matrix element (gated to 1e-11) or at the Z propagator. The cheapest next probe is
the two-dimensional one this chain never took: `MG-part` on `[39.4, 77) × [86, 96)`
against the same cell on this side, which separates "our `η(γ)` distribution at
fixed `m(μμ)` differs" from "MadGraph's `pt(γ)`-restricted runs under-recover".

Blind spot, stated because it is the reason this axis carries no verdict: `VG-part`
closes on both axes by construction (same draws), so nothing here audits *this*
side's coverage. `C_VG` on the `pt(γ)` axis (§D.M5) remains the only coverage audit
this side has.

## Close-out

(To be written at sprint close: per-chain outcomes, census before/after,
protocol observations on the design–implement–review structure.)
