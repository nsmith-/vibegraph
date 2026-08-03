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

## Chain A design (2026-08-02)

Read-only design session. Nothing outside this section was edited. Everything
below that is stated as a measurement was measured in this session, from the
banked MadGraph reference data in the chain worktree; everything stated as a
derivation is marked as one and carries the test that would falsify it.

### A.0 What the reference data says, before any code is written

Two facts had to be established first, because the obvious fix is wrong.

**Fact 1 — MadGraph keeps a separate `ICOLUP` table per colour-rep assignment,
and it is reachable.** `SubProcesses/P*/leshouche.inc` carries
`ICOLUP(slot, leg, iflow, isproc)` for *every* `isproc`, and `isproc N`
corresponds one-for-one with `matrix<N>_orig.f`, whose `C     Process:` header
names that subprocess. The correspondence is exact across the whole banked
tree: **79 `isproc` entries against 79 `matrix<N>_orig.f` files over 49 `P*`
directories** (49 of the 79 are the `isproc = 1` the oracle reads today). So the
conjugate members this crate gets wrong *do* have a reference table; nothing
needs regenerating to reach them.

`pp_to_jj/SubProcesses/P1_gq_gq/leshouche.inc`, both tables, verbatim:

```text
isproc 1  (matrix1: g u > g u)
  flow 1  ICOLUP(1,·)/501,503,503,501/   ICOLUP(2,·)/502,  0,502,  0/
  flow 2  ICOLUP(1,·)/503,502,503,501/   ICOLUP(2,·)/502,  0,501,  0/
isproc 2  (matrix2: g u~ > g u~)
  flow 1  ICOLUP(1,·)/501,  0,503,  0/   ICOLUP(2,·)/502,501,502,503/
  flow 2  ICOLUP(1,·)/503,  0,503,  0/   ICOLUP(2,·)/502,501,501,502/
```

Reading `isproc 1` flow 1 back as a basis key gives `T([1,3], 4, 2)` and
`derive_flow` reproduces `[501,502], [503,0], [503,502], [501,0]` label for
label — i.e. **the representative side is already right**, which is what the
existing oracle gates and is why §6's reprioritization trigger is *not* fired by
this design (see A.6).

**Fact 2 — the conjugate member's flow index is the reversed one, measured.**
This is the trap. `isproc 2`'s table at flow index `f` is *not* the image of
`isproc 1`'s flow `f`; it is the image of flow `3 − f`. The natural-looking fix
("keep the key, re-assign the `T`-chain ends from the member's own reps, so our
flow `f` gets MG's `isproc 2` flow `f`") is therefore wrong, and it is wrong in
a way that is *legal* — every slot it fills is a slot the member's rep allows —
so no legality check can see it.

The correspondence was measured off MadGraph's own banked `pp_to_jj` sample
(10 000 events), which needs no code: members of one group share `|M|²`
pointwise, so the two flows' *conditional* frequencies must agree between a
representative and its conjugate, and only the index map is in question.

| ordering | quark member | antiquark member | reversed-index hypothesis | same-index hypothesis |
|---|---|---|---|---|
| `g q > g q` | flow 1: 651, flow 2: 395 → `P₁ = 0.622 ± 0.015` | MG flow 1: 208, MG flow 2: 302 | `P(MG flow 2) = 0.592 ± 0.022` → **1.1 σ** | `P(MG flow 1) = 0.408` → **8.1 σ** |
| `q g > g q` | flow 1: 667, flow 2: 473 → `P₁ = 0.585 ± 0.015` | MG flow 1: 204, MG flow 2: 306 | `P(MG flow 2) = 0.600 ± 0.022` → **0.6 σ** | `P(MG flow 1) = 0.400` → **7.1 σ** |

(Aggregated over `q ∈ {d, u, s, c}`; the two rows are independent leg orderings
of the same claim. Command: `gunzip -c
validation/madgraph/output/pp_to_jj/Events/run_01/unweighted_events.lhe.gz`
piped through an `awk` that emits `IDUP | ICOLUP` per event, then `uniq -c` per
flavour class.)

The reversal is what charge conjugation predicts and is therefore not a
coincidence to be pinned by frequency alone: `C` maps QCD's gluon field onto
`−Aᵀ`, so a basis key conjugates as `T(a₁…aₙ, i, j)* = T(aₙ…a₁, j, i)`, and the
member's amplitude satisfies `JAMP'_{σ(f)} = ± JAMP_f` with `σ` the permutation
that key-conjugation induces on the sorted basis. For `g q > g q`, `σ` swaps the
two flows. Squaring kills the sign, so the draw `∝ JAMP2_f` taken off the
*representative* is already the correct draw — it just has to be labelled with
the member's basis element `σ(f)`, not `f`.

**Fact 3 — and that makes the tags transformation trivial.** Conjugating a
basis key flips every endpoint's SU(3) index rep while preserving *which leg*
each endpoint sits on and *which endpoints pair into a line* (checked on both
`T` chains and `Tr` traces). `AmpRep::slot` is `rep XOR incoming` and the
leg's `incoming` flag does not move, so every endpoint's slot flips and nothing
else does. Therefore:

> **The tags of the conjugated key are the tags of the key with both `ICOLUP`
> slots exchanged on every leg.**

Verified against the table above: exchanging both slots on every leg of
`isproc 1` flow 1 gives connectivity `{1a,4a} {1c,3c} {2a,3a}`, which is
`isproc 2` **flow 2** (`{1c,3c} {1a,4a} {2a,3a}`) under a relabelling; the same
holds for flow 2 → flow 1. Both directions checked by hand in this session.

So the fix is a **global slot exchange, applied when and only when every leg's
member rep is the conjugate of the representative's**. Note what it is *not*: it
is not "swap the slots on the legs that changed rep" (that breaks the gluon
legs' connectivity while staying legal), and it is not "re-derive from the same
key with the member's reps" (that is MG's `isproc` table at the *unreversed*
index, refuted at 7–8 σ above).

### A.1 Change list, file by file

**1. `vibegraph-lib/src/helas/repr/color.rs`**
Add `PartialOrd, Ord` to `ColorRep`'s derive list. Only needed so `Subprocess`
keeps its derived ordering after gaining a `ColorRep` field (change 6); the
order itself carries no meaning. If the implementation prefers not to order a
representation, the alternative is a member-indexed `Vec<Vec<ColorRep>>` on
`FlavorGroup` built in the same loop as `members` — acceptable, but it is a
parallel array and must be constructed in that one place.

**2. `vibegraph-lib/src/helas/color/flow_tags.rs`**

- Factor the rep → occupied-slots table out of `FlowBuilder::finish` into a free
  `fn slots_for(rep: ColorRep) -> [bool; 2]`, so the check has exactly one
  definition.
- `pub fn ColorFlowTags::conjugated(&self) -> ColorFlowTags` — exchange
  `[colour, anticolour]` on every leg of every flow. Its doc comment states the
  theorem of A.0 Fact 3 in its own terms (key conjugation flips every
  endpoint's index rep, preserves the pairings and the legs' directions, so the
  slot exchange is the whole of it) and says what it is for: the flows of a
  subprocess whose legs carry the conjugate reps of these.
- `pub fn ColorFlowTags::check_legs(&self, legs: &[LegColor]) -> Result<(),
  ColorAlgebraError>` — for every flow and every leg, the occupied slots must be
  exactly `slots_for(legs[leg].rep)`. This is the derive-and-check the record
  layer runs against the *member's own* reps.

**3. `vibegraph-lib/src/helas/eval/compile.rs`**
`leg_colors` is built at line ~189 and dropped after `color_flow_tags`. Keep it:
store `leg_colors: Vec<LegColor>` on `AmplitudeEvaluator` and add
`pub fn external_colors(&self) -> &[LegColor]`. This is the provenance that
matters — every consumer's leg reps then come from the compiled amplitude, not
from a second PDG → rep table.

**4. `vibegraph-lib/src/lhef/mod.rs`**
One new `LhefError` variant:

```rust
#[error(
    "leg {leg} carries colour rep {member:?} where the compiled subprocess carries \
     {representative:?}; an event record can only reuse a subprocess's colour flows for \
     legs whose reps are all equal to, or all conjugate to, its own"
)]
ColorRepsUnrelated { leg: usize, representative: ColorRep, member: ColorRep },
```

plus a variant (or a `String` payload on the same one) for a `check_legs`
failure, so a wrong table is refused at record construction rather than written.

**5. `vibegraph-lib/src/lhef/build.rs`**

- `SubprocessRecord` gains `legs: Vec<LegColor>` — the reps of the legs *this
  record* describes.
- `SubprocessRecord::new` fills it from `evaluator.external_colors()`.
- `relabelled(&self, order: &[usize], pdg: &[i32], legs: &[LegColor])`:
  1. the existing well-formedness check on `order`/`pdg`, extended to
     `legs.len() == n_ext`;
  2. `flows = self.flows.permuted(order)?` (unchanged);
  3. classify, over all legs `i`, comparing `legs[i].rep` against
     `self.legs[order[i]].rep`: **all equal** → keep; else **all conjugate**
     (`self.legs[order[i]].rep.anti() == legs[i].rep` for every `i`) → `flows =
     flows.conjugated()`; else → `Err(ColorRepsUnrelated)` naming the first leg
     that fits neither. Self-conjugate reps satisfy both, so the non-self-
     conjugate legs decide, and "all equal" is tested first so an all-gluon
     process takes the identity branch.
  4. `flows.check_legs(legs)?` — the derive-and-check, on the member's own reps.
  5. store `legs: legs.to_vec()` in the result.
- The doc comment's current claim ("the flavours sharing an amplitude are the
  ones whose legs carry the same masses") is the defect written down and must be
  replaced: sharing an amplitude and a mass list does not imply sharing a colour
  rep, so the flows travel with the legs only up to conjugation, and the reps
  the caller supplies decide which.

**6. `vibegraph-lib/src/proton.rs`**

- `Subprocess` gains `colors: Vec<ColorRep>` in the group's shared leg order,
  filled at group construction (~line 640) from
  `compiled[i].0.external_colors()` — the member's *own* compiled amplitude,
  which the grouping already builds and currently discards.
- `FlavorGroup::event_leg_colors(&self, member: usize, ordering: BeamOrdering)
  -> Vec<LegColor>` — the member's reps in physical leg order, applying the same
  `order.swap(0, 1)` `event_legs` applies, with `incoming` set positionally.
  Returning it from `event_legs` as a third element is equally fine; one
  function that cannot return the codes without the reps is slightly better.

**7. `vibegraph-cli/src/generate.rs`**
`flavor_records` (~line 655) passes `group.event_leg_colors(i, ordering)` into
`relabelled`. No other call site changes: `validate_lhef.rs:512`,
`validate_samples.rs:331` and `generate.rs:519/1203` use `SubprocessRecord::new`,
which does not relabel.

**8. `vibegraph-lib/tests/color_flow_tags_oracle.rs` — the widening**

- `parse_leshouche` returns every `isproc`, not `isproc == 1`:
  `Vec<(usize, Vec<Vec<[u32; 2]>>)>`, and additionally parses
  `IDUP(I, 1, isproc)`.
- `process_of` takes the `isproc` and reads `matrix<isproc>_orig.f`. A missing
  file is a **hard failure**, never a skip.
- One trial per `(P* directory, isproc)`; name it `pp_to_jj/P1_qq_qq#3`. Trial
  count goes **49 → 79**.
- Each trial additionally asserts that the compiled subprocess's PDG codes equal
  that `isproc`'s `IDUP(·, 1, isproc)` row, so "we compiled the member MadGraph
  names" stops being an assumption.
- The comparison itself is unchanged: connectivity per flow index, labels
  reported as information.

**9. `vibegraph-cli/tests/validate_samples_proton.rs`**

- The `pp_to_jj` row's `mode` flips `"info"` → `"gate"` (line ~699), and the doc
  comment's defect paragraph is replaced by what the cell now measures.
- Two per-event scans over the generated file, both new (note 28's counts came
  from a session-local instrument; nothing in the tree measures this today) —
  see tests **T5** and **T6** in A.2.

**10. `validation/manifest.toml`**
`pp_to_jj.samples`: `mode = "info"` → `"gate"`, and the note rewritten. The
current note is a careful description of the defect and its attribution; it
becomes a description of what the gated cell measures, with the `4758 / 80 000`
figure kept only as the recorded before-state.

**11. `vibegraph-lib/tests/validate_lhef.rs`** — no change expected. Its
`"ICOLUP slots 1 and 2 exchanged"` mutation is a re-serialisation negative
control on a fixed subprocess, not on a relabelled member, and the run it uses
has no conjugate member. Named here only because an implementer who greps for
slot exchanges will find it and wonder.

### A.2 Acceptance tests

**Must keep passing, unchanged** (the regression fence):

- `color_flow_tags_oracle`, every `isproc = 1` trial — the representative side.
- `an_exchanged_ordering_relabels_the_beams_of_every_per_leg_field`
  (`proton.rs`) — the beam-exchange permutation, which the classification of
  A.1/5 step 3 must resolve to the *identity* branch.
- `jj_subprocesses_are_madgraphs_own` — the 65-assignment set.
- `crossing_rule_is_not_free`, `uux_annihilation_flow_matches_madgraph_labels`,
  `uux_exchange_flow_matches_madgraph_labels`, `ggttx_chain_matches_madgraph_labels`,
  `gggg_trace_matches_madgraph_labels` (`flow_tags.rs`).
- `colour_lines_land_in_the_physical_slots` (`build.rs`).
- `cli_generate_proton`'s per-event `(roles, connectivity)` membership test on
  `p p > l+ l- j` — the negative control. `p p > l+ l- j` has no conjugate
  member in any group (MadGraph puts `g u > e+ e- u` and `g u~ > e+ e- u~` in
  separate `isproc`s of `P1_gq_llq`, and this crate separates them too because
  their `|M|²` differ), so **every one of its cells must be character-identical
  after the fix**. Same for `pp_to_bb` and `pp_to_ll`, whose second `isproc` is
  a different generation with identical reps, not a conjugate.
- `validate_lhef`'s byte-for-byte re-serialisation of all 37 banked runs.

**New:**

- **T1 `conjugating_a_flow_is_the_slot_exchange_and_nothing_else`**
  (`flow_tags.rs`, hermetic). Build `T([1,3], 4, 2)` on `g u > g u` legs and
  `T([3,1], 2, 4)` — its key-conjugate — on `g u~ > g u~` legs; assert
  `derive_flow(conjugate_key, conjugate_legs)` has the same connectivity as
  `conjugated(derive_flow(key, legs))`, and assert both against the literal
  `isproc 2` rows quoted in A.0. *Fails on*: a `conjugated()` that swaps
  something other than the two slots; a slot exchange applied per-leg instead of
  globally. *Provably cannot detect*: anything about which flow index a member's
  event should carry — it is a statement about one key at a time.
- **T2 `the_representatives_tags_are_illegal_on_a_conjugate_members_legs`**
  (`flow_tags.rs`, hermetic). `check_legs` on the unconjugated tags with
  antiquark legs must error, naming the leg and its rep. *Fails on*: a
  `check_legs` that is vacuous, or that reads the reps off the tags it is
  checking. *Cannot detect*: a wrong-but-legal flow — precisely the failure mode
  A.0 Fact 2 refutes, which is why T4 exists.
- **T3 `a_conjugate_member_gets_the_conjugated_colour_lines`** (`build.rs`,
  hermetic, hand-built record). Relabel a `g u > g u` record onto
  `g u~ > g u~` and assert the resulting `ICOLUP` matches the `isproc 2`
  connectivity of A.0 at the reversed flow index; assert `relabelled` returns
  `ColorRepsUnrelated` for a leg list that is neither all-equal nor
  all-conjugate. *Fails on*: the classification taking the wrong branch, the
  conjugation being skipped, `check_legs` not being wired in. *Cannot detect*:
  that the flow index the generator draws is the one this record should be
  labelled with — checked-data only, no amplitude in it.
- **T4 `a_conjugate_member_carries_its_own_subprocesss_colour_flows`**
  (`proton.rs`, hermetic — **the linchpin**). Modelled directly on
  `an_exchanged_ordering_relabels_the_beams_of_every_per_leg_field`. For every
  group of `p p > j j` and every member and both beam orderings: compile the
  member from its own process string; evaluate `eval_jamp2` on both the
  representative and the member over the shared probe points; establish the flow
  permutation `π` by matching `JAMP2` values; assert `π` is the *same*
  permutation at every probe point; assert the record layer's tags for that
  member at flow `f` have the same connectivity as the member's own compiled
  tags at `π(f)`. Two anti-vacuity assertions are **required**, or the test
  passes for the wrong reason: (i) at least one member took the conjugation
  branch, and at least one `π` is not the identity — otherwise it never sees the
  defect; (ii) at every probe point the `JAMP2` entries are separated by more
  than a stated relative margin, so `π` is uniquely determined rather than
  matched by rounding. *Fails on*: the whole defect class, including a global
  slot exchange applied at the unreversed index. *Cannot detect*: an error
  shared by the two compilations — if `color_flow_tags` derived both the
  representative's and the member's keys wrongly in the same way, this agrees.
  That is what the widened oracle (T7) is for, and the two together have no
  common blind spot.
- **T5 `no_generated_leg_carries_a_line_in_the_slot_its_rep_forbids`**
  (`validate_samples_proton.rs`). Reference-free scan of the generated
  `pp_to_jj` file: for every event and leg, occupied slots must be
  `slots_for(rep)`. Must read **0 / 80 000** (recorded: 4 758 / 80 000). Run it
  over MadGraph's banked file too, which must also read 0 — an instrument that
  cannot fail on the reference is not an instrument. *Cannot detect*: a legal
  but wrong flow, or a wrong flow *frequency*.
- **T6 `every_generated_dijet_colour_pattern_is_one_madgraph_lists`**
  (`validate_samples_proton.rs` or a shared helper with
  `cli_generate_proton.rs`). Build the allowed `(roles, connectivity)` set from
  `leshouche.inc` — every `isproc`, every flow, both beam orderings — and assert
  every generated event's pattern is in it, at zero tolerance. Derive the set
  from `leshouche.inc` rather than from MadGraph's 10 000-event sample: a
  sample-derived set is incomplete for rare flows and would fail honest events.
  *Cannot detect*: frequencies — every event could carry the same legal pattern
  and pass.
- **T7 the widened `color_flow_tags_oracle`** (extended-validation, 79 trials).
  *Fails on*: any derived table disagreeing with MadGraph's for any subprocess
  of any banked directory. *Cannot detect*: what the record layer does with a
  derived table — it never constructs a record. T4 and T6 cover that.
- **T8 the `ICOLUP` χ² column** of the `pp_to_jj` `samples` cell, flipped to
  gate: below the `1e-4` floor on **every** seed. *Cannot detect*: an error that
  preserves the colour-key frequencies; correlations with other columns; a
  discrepancy in a small tail.

The instrument ladder is deliberate: T5 legality → T6 connectivity legality
against the reference → T4 the right flow for the right `JAMP` → T8 the right
frequencies. Each one is blind to the next one's failure.

### A.3 Gates to run, and the cells expected to move

In order, each backgrounded with a `chainA_`-prefixed log, `--skip-deps` on
every `pixi run`:

1. `cargo build` and `cargo test --workspace` (hermetic, no features) — T1–T4.
2. `pixi run --skip-deps validate-color-flow-tags` — T7, expect **79** trials
   passing where 49 ran before.
3. `pixi run --skip-deps validate-color-cf`, `validate-lhef`,
   `validate-unweighting`, `validate-generate-proton` — the colour/LHEF gates
   named in the acceptance. `validate-generate-proton` is the `p p > l+ l- j`
   negative control and must be character-identical.
4. `pixi run --skip-deps validate` — the full banked layer and the collated
   report. Long; background it.

**Expected to move — exactly one cell:**

| cell | before | after |
|---|---|---|
| `pp_to_jj` / `samples` | ⚠️ banked `info`, `ICOLUP` χ² ≈ 2470 / 25 at `p 0` | ✅ banked `gate`, every column above the `1e-4` floor |
| census | 87 measured / 85 ✅ / 2 ⚠️ | 87 measured / 86 ✅ / **1** ⚠️ |

The surviving ⚠️ is the hermetic `diagrams` `info` cell at
`validation/manifest.toml:235`, which this chain does not touch.

**Must not move:** every other printed field of every other row, including
`pp_to_jj`'s own `integrals` cell (the fix changes no weight — the colour flow
is drawn for the record and never enters the integrand), `pp_to_llj*`,
`pp_to_bb*`, `pp_to_ll*`, and every partonic σ row. The implementation session
must diff the rendered report against a baseline taken **before** its first
edit, and report the diff line count, not the impression. Footnote renumbering
after the changed row is expected and is not a moved cell.

### A.4 Risks

- **The χ² still fails after the fix.** The frequency evidence of A.0 Fact 2
  bounds the residual at ≈1 σ on a 510-event MadGraph sub-sample, which does not
  bound it at the resolution of 3 × 20 000 events against 10 000. If T4, T5, T6
  and T7 are all green and T8 still fails, that is a *second* defect and the
  chain must stop and report it rather than tune anything — the diagnosis is
  already narrowed to the flow *frequencies*, i.e. to `select_color_flow`'s
  `AMP2`-then-`ICOLAMP` composition, not to the tags.
- **`LeadingColorFlows` (`ICOLAMP`) is reused per member too, and this design
  does not change it.** That is deliberate and believed correct: the
  configuration draw, the mask and the flow draw all happen in the
  *representative's* indexing and are mutually consistent there; only the final
  label is translated. If T8 fails while T4 passes, this is the first place to
  look, because a diagram-index permutation between member and representative
  would show up exactly as a frequency error.
- **The group-formation checks cannot see the flow permutation.** `proton.rs`
  requires members to agree on `n_flows` and on `cf_matrix`; for `g q > g q` the
  2 × 2 CF matrix is symmetric with equal diagonals, so it is invariant under
  the reversal `σ` and the check passes on a permuted basis. Nothing existing
  guards this; T4 is what starts guarding it.
- **A member that is neither all-equal nor all-conjugate.** Unreachable today
  (checked: within `p p > j j` the conjugating groups are `g q`/`g q̄`,
  `qq`/`q̄q̄` and `qq'`/`q̄q̄'`, all global; the outgoing-swap cases are removed by
  the enumeration's sorted-final-state key and the beam-exchange cases resolve
  to the identity). The design hard-errors rather than guessing, which is the
  right behaviour at a boundary the restricted scope does not cover.
- **Trial-count creep in the widened oracle.** 79 trials each compile a
  subprocess; `P1_qq_qq` alone adds six. If the gate's wall time becomes a
  problem the answer is not to narrow the oracle back — it is to share the model
  load across trials.
- **T4's cost.** It compiles every member of every `p p > j j` group in the
  hermetic suite. Group formation already compiles all 65 members once, so the
  order of magnitude is known and affordable; if it is not, the acceptable
  reduction is to the groups that actually mix reps (`g q`/`g q̄` and the
  `qq_qq` pairs), never to a single hand-picked pair — the anti-vacuity
  assertions must still hold.

### A.5 What this provably cannot break

The change is confined to the `ICOLUP` columns of emitted event records.

- **No cross section can move.** The colour flow is selected *after* a point is
  accepted and enters no weight: `select_flow`'s own documentation records that
  it "never enters the integrand". `conjugated()` and `check_legs` are called
  once per `(member, ordering)` at record-construction time and touch no
  momentum, weight, coupling or scale. Every σ row is therefore unmoved
  bit-for-bit, and the `integrals` cells are a control on that claim rather than
  a hope.
- **No event's kinematics, flavours, helicities, masses, statuses or mothers
  can move.** `relabelled` gains one argument and one transformation of
  `self.flows`; `pdg`, `mass`, `n_in` and the `order` permutation are untouched,
  and `event()` reads the same fields it reads today.
- **No subprocess whose group has a single rep assignment can move at all** —
  the classification takes the identity branch and `conjugated()` is never
  called. That is every gated row except `pp_to_jj`, verified against the
  reference rather than assumed: `pp_to_bb`'s and `pp_to_ll`'s second `isproc`
  is a different generation with identical reps, and `pp_to_llj`'s `g q̄` is a
  separate `isproc` that this crate also keeps in a separate group.
- **A record that would have been wrong is now refused, not written.**
  `check_legs` runs on the member's own reps before any event is emitted, so the
  4 758-legs failure mode cannot be reintroduced silently by a future change to
  the grouping: it becomes a hard error at record construction.

What it emphatically *can* break, and what the gates are for: any consumer that
depended on the old, wrong `ICOLUP` for an antiquark member. Nothing in this
tree does — `validate_lhef`'s byte-for-byte re-serialisation reads MadGraph's
files, not ours.

### A.6 §6's reprioritization trigger — how the widened oracle distinguishes it

§6's risk is that the widened oracle finds the **representative's** tags wrong
too, not only the conjugate reuse. The widened oracle separates the two by
construction, because it keeps the `isproc` index in the trial name:

- a failure on an `isproc = 1` trial is a **representative-level** finding — the
  oracle already covers those 49 subprocesses today and they pass, so a new
  failure there means the derivation itself moved;
- a failure on an `isproc > 1` trial whose `IDUP` row is the *conjugate* of that
  directory's `isproc 1` row is the defect this chain fixes;
- a failure on an `isproc > 1` trial whose `IDUP` row is **neither equal nor
  conjugate** in rep pattern to `isproc 1` — for instance `P1_qq_qq`'s
  `u u~ > u u~` against its `u u > u u` — is a **new** finding: a subprocess no
  oracle has ever seen, whose derivation is independent of both. The trial name
  and the `IDUP` assertion of A.1/8 make the classification mechanical rather
  than a judgement call.

**This session found no evidence of the trigger.** The representative side was
spot-checked by hand against `P1_gq_gq` `isproc 1`, both flows, and reproduces
MadGraph label for label. The 30 newly-reachable subprocesses have never been
compared, so the trigger remains genuinely open until T7 runs — which is the
point of widening the oracle before, not after, flipping the cell.

## Chain A design amendment (2026-08-03)

Second design session, after implementation falsified the first design's central
classification. It supersedes **A.1 items 4–7** and **A.2's T3 and T4**, and
corrects **A.0 Fact 2** and **A.4's "unreachable today"** claim. A.1 items 1–3
and 8 are landed as `8a56825`; A.2's T1, T2, T5–T8, A.3 and A.5 stand except
where said below. Written against the worktree at `8a56825`, with the withheld
implementation attempt (`chainA_full_attempt.patch`, 1252 lines) as raw material.

### B.0 What was wrong, and the measurement that settles the replacement

**The transformation premise is dead.** A.0 concluded that a member's tags are
the representative's under a global `ICOLUP` slot exchange. That is true for one
of three classes and false for the class that matters most:

| class | example (`p p > j j`) | relation to the representative's table |
|---|---|---|
| identity | `u u > u u` ← `c c > c c` | equal |
| global conjugate | `u u > u u` ← `u~ u~ > u~ u~`; `g u > g u` ← `g u~ > g u~` | global slot exchange |
| **crossing** | `u c > u c` ← `u c~ > u c~` | **no slot operation relates them** |

The crossing class conjugates exactly two of four legs. Read off
`pp_to_jj/P1_qq_qq/leshouche.inc`, as colour lines:

```text
isproc 4  u c > u c      flow 1 {1c,3c} {2c,4c}      flow 2 {1c,4c} {2c,3c}
isproc 6  u c~ > u c~    flow 1 {1c,2a} {3c,4a}      flow 2 {1c,3c} {2a,4a}
```

The *leg pairings* differ: `isproc 4` pairs `{1,4}{2,3}` on its flow 2, and no
flow of `isproc 6` pairs those legs at all. A slot exchange is a per-leg
relabelling of endpoints; it cannot re-route a line from one leg to another. So
the transformation does not merely need a third case — it does not exist. The
global exchange of `isproc 4` produces `isproc 7`'s all-anticolour tables, which
`check_legs` correctly refuses on `isproc 6`'s legs, and the per-leg exchange
gets flow 1 right and flow 2 wrong. 12 of the 65 dijet assignments are in this
class, so the first design's hard-error path aborts `p p > j j` outright. The
implementation session's finding is confirmed here independently and its
recommendation is adopted.

**Why it is a re-routing, in one line.** `u c > u c` has one diagram, colour
factor `T^a_{31} T^a_{42}`, whose Fierz is
`½(δ_{32}δ_{41} − Nc⁻¹ δ_{31}δ_{42})`. Conjugating the `c` line transposes the
second factor: `T^a_{31} T^a_{24}`, Fierz `½(δ_{34}δ_{21} − Nc⁻¹ δ_{31}δ_{24})`.
The subleading term keeps the leg pairing `{1,3}{2,4}`; the **leading** term
moves from `{1,4}{2,3}` to `{1,2}{3,4}`. That is the re-routing, and it is why
the leading flow sits at index **2** for `u c > u c` and index **1** for
`u c~ > u c~`.

**Measured, categorically, on MadGraph's banked `pp_to_jj` sample.** Restricted
to the matched leg ordering (`out = (q, q')`, gluons excluded — an earlier cut of
this measurement mixed orderings and gave a nonsense answer):

| subprocess class | patterns emitted in 10 000 events |
|---|---|
| `q q' > q q'` (`isproc 4` ordering) | **one** pattern, 35 events: `{1c,4c}{2c,3c}` = its flow **2** |
| `q q~' > q q~'` (`isproc 6` ordering) | **one** pattern, 48 events: `{1c,2a}{3c,4a}` = its flow **1** |

MadGraph emits *only* the leading flow for these single-diagram subprocesses —
which is `ICOLAMP` doing exactly what it is for — so the correspondence
`leading ↔ leading` is forced rather than inferred, and it maps flow **2 ↦ 1**.
This is a categorical confirmation, not a statistical one: there is no second
pattern for the correspondence to be wrong about.

### B.1 Amendment to A.0 Fact 2 — the flow permutation is not a rule, it is a computation

A.0 Fact 2 said the conjugate member's flow index is "the reversed one", and
generalised a two-flow measurement into a rule. That is wrong, and the
implementation session is right to call it out. Corrected statement, with every
case verified in this session against `leshouche.inc`:

| representative → member | tags | flow index |
|---|---|---|
| `g u > g u` → `g u~ > g u~` | global exchange | **reversed** (1↔2) |
| `u u > u u` → `u~ u~ > u~ u~` | global exchange | **preserved** |
| `u c > u c` → `u c~ > u c~` | no transformation | **reversed** (2↦1) |

Two members of the *same* class carry different permutations, so the permutation
cannot be read off the class. What survives from A.0 is only the underlying
principle — the member's basis element that corresponds to the representative's
flow `f` is the one carrying the same amplitude, i.e. `JAMP'_{π(f)} = ± JAMP_f`
— and `π` must be **computed per member**, never assumed. B.2 makes that
computation structural.

### B.2 Change list against the current worktree (`8a56825` landed)

The shape of the fix: **each member carries its own subprocess's colour-flow
table, reordered once into the representative's flow indexing.** Nothing is
transformed, so there is no theorem to get wrong and the three classes are one
code path.

**1. `vibegraph-lib/src/helas/color/colorize.rs` + `helas/eval/compile.rs` — the
flow fingerprint.**
`ColorBasis::elements[f].contributions` is already exactly what distinguishes one
flow from another: `Contribution { diagram, chain, coeff }` with `coeff` an exact
`q · i^imag · Nc^power`. Compile keeps a per-flow fingerprint and exposes it:

```rust
/// Per flow, the contributions summing into its JAMP, as a sorted fingerprint:
/// `(diagram, chain, coeff.nc_power, |coeff.q|)`.
flow_fingerprints: Vec<Vec<(usize, Vec<u8>, i32, Ratio<i64>)>>
```

with `pub fn flow_fingerprints(&self) -> &[Vec<…>]`. **The sign and the `i^imag`
phase are deliberately dropped**: charge conjugation can flip a contribution's
sign (`T^a → −T^{aᵀ}` puts a `(−1)ⁿ` on an `n`-gluon-vertex diagram), and the
quantity being matched is which diagram lands on which flow at which power of
`Nc`, which that phase does not move. Retaining the whole `ColorBasis` on the
evaluator instead is acceptable if the implementation prefers it; the
fingerprint is specified because it is the part that is actually used and it
bounds the memory.

**2. `vibegraph-lib/src/proton.rs` — `π` and the member's tags, fixed at group
construction.**
`derive_flavor_groups` already compiles every member (`compiled[i].0`) to test
`|M|²`, `n_flows` and `cf_matrix`, then discards all but the representative.
Stop discarding what the record layer needs. For each member:

- `π` = the unique bijection `rep flow f ↦ member flow π(f)` with equal
  fingerprints. **Uniqueness is required, not assumed**: if either basis has two
  flows with equal fingerprints, or no bijection exists, or more than one does,
  this is a `ProtonError` naming the group, the two subprocesses and the
  ambiguous flows. **No tie-break, no heuristic, no numeric fallback** — a
  refusal at setup is correct, and if it ever fires the chain returns to design
  rather than the implementation inventing a rule.
- the member's `ColorFlowTags` reordered into the representative's indexing:
  `member_flows[f] = member_evaluator.color_flow_tags().flow(π(f))`. Store this
  on `Subprocess` (alongside `colors`, which the withheld patch already adds and
  which is kept verbatim) together with `π` itself, which is wanted for the
  tests and for a failure message even though production only reads the
  reordered table.
- `Subprocess::colors` and `FlavorGroup::event_leg_colors` from the withheld
  patch are adopted unchanged. `ColorRep` gaining `PartialOrd, Ord` (A.1 item 1)
  is still needed for `Subprocess`'s derived ordering.

Because the reordering happens once, **no downstream consumer sees `π`**: the
configuration draw, the `ICOLAMP` mask and the flow draw all stay in the
representative's indexing, and the tag lookup is a plain index.

**3. `LeadingColorFlows` / `ICOLAMP` — no change, and why that is a claim and not
an omission.** The concern is right to raise and is answered by the choice of
`π`: `reached[d][f]` is "diagram `d` contributes to flow `f` at the basis's
maximal `Nc` power", and `π` is defined to preserve `(diagram, chain,
nc_power)`. Therefore `rep.reached_by(d)[f] == member.reached_by(d)[π(f)]`
identically, and masking in the representative's indexing *is* masking in the
member's. The worked case: `u c > u c`'s mask marks flow 2, `π(2) = 1`, and
`u c~ > u c~`'s own leading flow is 1 — which is the only flow MadGraph ever
emits for it (B.0). This identity is not left as reasoning: **T10 asserts it
elementwise**, and if it ever fails, tag-only translation is insufficient and the
mask must be translated too.

**4. `vibegraph-lib/src/lhef/build.rs` — `relabelled` gets simpler, not more
complex.** It no longer classifies anything and no longer transforms anything:

```rust
pub fn relabelled(
    &self,
    order: &[usize],
    pdg: &[i32],
    legs: &[LegColor],
    flows: &ColorFlowTags,   // the member's own, in the representative's indexing
) -> Result<Self, LhefError>
```

1. the existing well-formedness check on `order`/`pdg`, extended to `legs` and to
   `flows` agreeing with `self` on `n_ext` and `n_flows`;
2. `flows.permuted(order)` — the beam exchange, unchanged;
3. `flows.check_legs(legs)` — the derive-and-check against the member's own reps,
   which is what makes the original 4 758-leg defect a refusal rather than an
   emission;
4. store `legs` and the permuted flows.

`LhefError::ColorRepsUnrelated` from A.1 item 4 is **not** added — there is no
longer a classification to fail. The `check_legs` failure variant is. The doc
comment must drop the "flavours sharing an amplitude carry the same masses"
premise; the honest statement is that the flows do **not** travel with the legs,
they are the member's own, and only the beam-exchange permutation is applied here.

**5. `vibegraph-cli/src/generate.rs` — `flavor_records`** passes the member's
`event_leg_colors(i, ordering)` and the member's stored flow table into
`relabelled`.

**6. `ColorFlowTags::conjugated` (landed, and now unused by production).** Keep
it, used by **T9** as an independent oracle for the global-conjugate class: a
second derivation with a different failure mode is worth more than a deleted
function. Its doc comment must gain the boundary it lacks — that it relates a
subprocess to its *full* conjugate only, and that a partially-conjugated member
is not related to it by any slot operation.

**7. Not in scope, designed around.** The slot-order dependence of compiled
`|M|²` at `NCOLOR > 1` (`u g > g u` against `g u > g u`) is on the manager's
backlog. This design never compiles a non-canonical ordering: members are
compiled from their own enumerated `DiagramSet`s, and the beam exchange is a leg
permutation applied to an already-resolved table (`ColorFlowTags::permuted`),
never a second compilation. T11 replaces the withheld T4's Exchanged half on that
basis.

### B.3 Acceptance tests

A.2's **T1, T2, T5, T6, T7, T8 stand unchanged**. T3 and T4 are superseded by
T9–T12. Every one of T9–T12 runs over **all three classes** and carries the
anti-vacuity assertions named.

- **T9 `every_member_carries_its_own_subprocesss_colour_flows`** (`proton.rs`,
  hermetic, `p p > j j` — **the linchpin**). For every group, every member, both
  beam orderings: compare the record layer's table against the member's own
  compiled `ColorFlowTags` under the stored `π`, by connectivity. Then, per
  class: identity members must have `π = id` and tags *equal* to the
  representative's; global-conjugate members must have tags equal to
  `representative.conjugated()` at `π(f)` (the independent oracle of B.2/6);
  crossing members must have tags that are **neither** the representative's nor
  its conjugate at any index — asserted, because that is the statement that
  falsified the first design and it must stay falsified.
  **Anti-vacuity, all required**: at least one member in each of the three
  classes; at least one non-identity `π` *and* at least one identity `π` among
  members whose reps differ from the representative's (`u u > u u` →
  `u~ u~ > u~ u~` is the conjugate-class member with `π = id`, and it is the case
  that kills any "conjugate ⇒ reversed" shortcut). Report the per-class counts.
  *Cannot detect*: an error shared by the member's compilation and the
  representative's — T7's 73 `leshouche` trials are what exclude that, and the
  two have no common blind spot.
- **T10 `the_flow_permutation_carries_the_leading_colour_mask`** (`proton.rs`,
  hermetic). For every group, member, diagram `d` and flow `f`:
  `rep.reached_by(d)[f] == member.reached_by(d)[π(f)]`. *Anti-vacuity*: assert
  that some `reached` row is not all-true and some `π` is not the identity —
  otherwise the identity is trivially satisfied. *Fails on*: a `π` that matches
  tags but not contributions, which is exactly the failure that would leave the
  crossing class with right labels at wrong frequencies. *Cannot detect*: that
  the mask itself is the right mask — that is `validate_unweighting`'s job.
- **T11 `the_exchanged_ordering_is_a_leg_permutation_of_the_direct_one`**
  (`proton.rs`, hermetic). The `Exchanged` record's tags equal the `Direct`
  record's under `permuted([1, 0, 2, …])`, and its leg reps are the member's
  swapped. Replaces the withheld T4's Exchanged half **without compiling a
  swapped process string**, so it is independent of the slot-order finding.
  *Cannot detect*: whether the direct ordering itself is right — T9's job.
- **T12 `the_flow_fingerprint_identifies_a_flow_uniquely`** (`proton.rs` or
  `colorize.rs`, hermetic). Within every basis of every `p p > j j` and
  `p p > l+ l- j` member, the fingerprints are pairwise distinct, and the
  `π`-matched fingerprints agree. Separately, at the group's own probe points,
  `JAMP2_rep[f] == JAMP2_member[π(f)]` to the mirror identity's `1e-11`,
  **subject to the degeneracy rule**: the value comparison is asserted only for
  flows separated from every other flow by more than `1e-3` of the summed `|M|²`
  at some probe point; where a degenerate block exists, assert instead that `π`
  maps the block onto a block of equal JAMP2 multiset. This is the numeric
  cross-check on a structural decision — it must never become the decision, and
  the degeneracy rule is why. (`g g > g g`'s reflection-degenerate flows are the
  known block; it is an identity-class group, so T9 pins it far more tightly than
  any JAMP2 match could.) *Cannot detect*: a fingerprint scheme that is unique but
  matches the wrong pairs — T9 and T10 are what exclude that.

### B.4 Gates and expected movement

Unchanged from A.3, with two additions: `pixi run --skip-deps
validate-color-flow-tags` is already green at **73 trials** on `8a56825` and must
stay so (the counts in A.0/A.1 said 49/79; the correct figures are **47 files /
73 isprocs**, the difference being the since-retired `pp_to_llj_qcd2_qed2`), and
the hermetic suite now carries T9–T12.

E1 is unchanged and is still the target:

| cell | before | after |
|---|---|---|
| `pp_to_jj` / `samples` | ⚠️ banked `info`, `ICOLUP` χ² ≈ 2470 / 25 at `p 0` | ✅ banked `gate`, every column above the `1e-4` floor on every seed |
| T5's reference-free scan | 4 758 / 80 000 legs | **0 / 80 000** |
| census | 87 measured / 85 ✅ / 2 ⚠️ | 87 / **86** ✅ / **1** ⚠️ |

Must not move: every other printed field, `pp_to_jj`'s own `integrals` cell, and
every row of `pp_to_llj*`, `pp_to_bb*`, `pp_to_ll*`. The zero-line report diff
the implementation session already recorded for `8a56825` is the baseline the
next diff is taken against.

### B.5 Risks, and what this provably cannot break

**Principal residual risk — the fingerprint fails to determine `π`.** If two
flows of one basis share a fingerprint, the group is refused at setup. The
design chooses that over a tie-break because a wrong `π` is a silently wrong
event sample, while a refusal is loud and cheap. It is not expected to fire on
`p p > j j`: the crossing pair's two flows are separated by `nc_power` alone
(`0` against `−1`), and the `g q` pair by which diagrams reach which flow. If it
does fire, the answer is a richer fingerprint, not a fallback — and it comes back
to design.

**The helicity correspondence across a conjugate member is assumed, not proved,
and this design does not change that.** The record already labels a member's
event with a helicity drawn off the representative's per-helicity `|M|²`; the
`SPINUP` column of the dijet samples cell clears the floor at `p 0.18–0.35`,
which is evidence and not proof. Out of scope, named so the review does not read
its absence as a claim.

**MadGraph's own sample cannot cross-check every `π`.** For single-diagram
subprocesses it emits only the leading flow, so the crossing class is pinned
categorically but with no second pattern to check the subleading map against.
The `g q` class, with three diagrams and both flows populated, is where the
sample checks a full permutation; T12's `JAMP2` identity is what covers the rest.

**Provably cannot break:**

- **No cross section can move.** The colour flow is selected after a point is
  accepted and enters no weight; `π`, the fingerprint and the per-member tables
  are all resolved at group construction and touch no momentum, weight, coupling
  or scale. The `integrals` cells are the control on that.
- **No event's kinematics, flavours, helicities, masses, statuses or mothers can
  move.** `relabelled` gains arguments and *loses* a transformation; `pdg`,
  `mass`, `n_in` and `order` are untouched.
- **No identity-class group can move at all** — `π = id` and the member's tags
  are asserted equal to the representative's, so every gated row except
  `pp_to_jj` is bit-identical. Verified against the reference rather than
  assumed: `pp_to_bb` and `pp_to_ll` differ between `isproc`s by generation, not
  by rep, and `pp_to_llj`'s `g q~` is a separate `isproc` this crate also keeps
  in a separate group.
- **A table that does not fit its legs is refused, not written.** `check_legs`
  runs against the member's own reps before any event is emitted.

**§6's trigger is now closed, not merely unfired**: all 73 `leshouche` trials
pass, including `u u~ > u u~`, `u u~ > c c~` and `u c~ > u c~`, so the
per-subprocess derivation is right everywhere and the defect is confined to the
record layer's cross-member reuse — which is what this amendment replaces.

## Close-out

(To be written at sprint close: per-chain outcomes, census before/after,
protocol observations on the design–implement–review structure.)
