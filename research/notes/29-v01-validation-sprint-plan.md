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

## Chain F design (2026-08-02)

Research-only sidecar. This section is written **before any derivation is
attempted**; everything below is a pre-registration. The research session
executes it, the review session checks the derivations against the dumps
itself. No production file is touched by either.

Note that §2's design-session output shape (change list, gates, cells expected
to move) does not apply to a chain that changes no code; F's design deliverable
is the inventory + bar + hostile-case order + risks below.

### F.1 The pinned-convention inventory, and what the rule must predict

`fermi_sign` is a product of five factors, assembled in
`helas/eval/root_diagram.rs::compile_single_diagram` (~line 1042):

```
fermi_sign = diagram.sign
           · spine_sign_from_flow(canonical)
           · yang_mills_vvv_sign(diagram, model)
           · canonical.build_convention_sign()
           · canonical.reversed_convention_sign() · tree.reversed_convention_sign()
```

with `build_convention_sign` itself a product of four independent sub-arms
inside `root_lorentz.rs::build_at_leg`. Counting the colour-side swap and the
two fitted constants, there are **nine** pinned conventions, not four. Each row
below records where it lives, the test that pins it, the pinned value, and the
charge-flow rule's predicted value — or, where the prediction cannot be stated
in advance, the derivation step that decides it.

| # | Convention | Implemented at | Pinned by | Pinned value |
|---|---|---|---|---|
| 1 | Fermi permutation sign | `diagrams/diagram.rs:115` (`sign`, from feyngraph `view.sign()`) | only transitively, by every per-diagram row of `tests/amplitude_oracle.rs` | ±1 per diagram, relative |
| 2 | Fermion-spine sign | `root_diagram.rs:695` `spine_sign_from_flow` | `root_diagram::tests::spine_sign_from_flow_matches_heuristic`; `…::spine_sign_separates_mixed_line_and_crossed_line_propagators`; `mg_guard_processes_exercise_every_convention_channel` (Bhabha arm) | crossed line → −1 once; uncrossed → −1 iff `n_props` odd |
| 3 | Reversed-bilinear parity | `root_lorentz.rs:676` `term_reversed_parity` (+ runtime `resolve_bra_ket`) | `mg_guard_processes_exercise_every_convention_channel` (`e+ e- > mu+ mu-` arm, channel 3 > 0) | −1 per `GammaVout` whose UFO row index `i` is a **ket** |
| 4a | Crossed-pair −1 | `root_lorentz.rs:486/502/558` (`pair_crossed`) | `e+ e- > ta+ ta- H` per-diagram vs MG `AMP()` (`amplitudes/ee_to_tatah.json`) | −1 when either bilinear leg is on a crossed line |
| 4b | Scalar-sink bilinear −1 | `root_lorentz.rs:485/501/557` (ProjM/ProjP/Identity) | `mg_guard_…` (`e+ e- > ta+ ta- H`, channel 2 > 0); bit-for-bit by `ee_to_tatah` | −1, unconditional, at a scalar/amplitude sink |
| 4c | Pure-metric vertex −1 (VVS/VVSS/VVVV) | `root_lorentz.rs:530-537`, once per term (`metric_vertex_applied`) | `mg_guard_…` (`g g > g g`, channel 2 > 0); `root_lorentz::tests::test_root_vvs_metric` (`build_sign == -1`, line 1202, amplitude root) and `…::test_root_vvs_metric_scalar_out` (line 1235, scalar-out root) | −1 once per term |
| 4d | Standalone-projector-crossed −1 | `root_lorentz.rs:654` | `e+ e- > ta+ ta- H` (H off the crossed τ line) vs MG `AMP()` | −1 |
| 5 | Yang–Mills VVV source sign | `root_diagram.rs:996` `yang_mills_vvv_sign` | `root_diagram::tests::yang_mills_vvv_sign_fires_only_for_source_vvv`; `mg_guard_…` (`e+ e- > W+ W-`, channel 0 > 0) | −1 per VVV vertex at index ≥ 1 |
| 6 | VVVV amplitude phase | commit `eda4412`, which deleted `Op::MetricNegI` end to end | `gg_to_gg` promoted to `EXPECT_MATCH` in `validate_helas_mg.rs` at `REL_TOL` 1e-12 (observed 8.25e-14); `amplitudes/gg_to_gg.json` row | real **−1** — *not* −i |
| 7 | Colour 3/3̄ slot swap | colorize walk (note 16 §2.4): unconditional swap of the single 3 and 3̄ slots at every vertex having one of each | `tests/color_cf.rs::gg_to_ttx_flow_structures_untransposed` | `flow_structures == [T(1,2,3,4), T(2,1,3,4)]`, exactly one imaginary contribution per flow |
| 8 | Fitted global constant `G` | `tests/amplitude_oracle.rs:581` `fit_constant` (least squares `Σ conj(mg)·vg / Σ|mg|²`), called at line 1016 | asserted `|G| = 1` at 1061-1068 and `Re G = 0` at 1069-1075, both at `LINEAR_REL_TOL = 1e-12` | `G ∈ {+i, −i}`; **the sign is not asserted**, only printed (`g.im.signum()`, ~line 1136) |
| 9 | Fitted per-configuration phases `k` | `tests/amplitude_oracle.rs:1101` (`fit_constant` per configuration) | `|k| = 1` asserted at 1117-1124; residual at 1111-1116 | unit modulus only — **the phase of `k` is entirely free**, one per configuration per process |

Two observations that the brief's framing misses and that the bar must account
for. First, the fitted constants are **not** "the one fitted constant `G`":
row 9 fits one further unit phase *per configuration per process* (21 of them on
`ud_to_epemud_qcd0` alone), and nothing asserts those phases are even real. Any
"reduce the fitted constants to derived ones" claim has to count rows 8 **and**
9. Second, `AmplitudesRow` (`tests/common/report.rs`) banks the *deviations*
`per_diagram` / `per_flow` / `per_config` but banks **neither `G` nor `k`**, so
the dataset this whole investigation needs does not currently exist on disk and
must be harvested from stdout or a scratch script (see F.4).

**What the charge-flow rule predicts.** Sorting the nine rows by whether a
fermion arrow is even present:

- **Predicted DERIVED (charge flow is the whole content).** Row 1 — the Wick
  ordering of external fermion operators is the permutation parity of the
  charge-flow path endpoints. Row 3 — `Cγ^{μT}C⁻¹ = −γ^μ`, the archetype.
  Row 4a and row 4d — `ū₁Γv₂ = −ū₂(CΓᵀC⁻¹)v₁`, C-conjugation of a bilinear read
  against its arrow. Row 7 — "index `T(…,i,j)` by the arrow-out leg" *is* a
  charge-flow rule already, written in the colour sector; note 16 §2.4 recorded
  it as an unconditional swap precisely because nobody derived it. Row 2's
  **crossed-line arm** — same C-conjugation as 4a, at the line rather than the
  vertex.
- **Predicted NOT derivable from charge flow.** Rows 4b, 4c, 5, 6 involve no
  fermion arrow at all; `g g > g g` has no fermion line anywhere and still
  carries a nonzero build sign (row 4c). Row 8's *quadrant* is predicted derived
  but by **i-counting, not charge flow** (see below).
- **Undecided before derivation, and this is the crux.** Row 2's
  **per-propagator arm** (`−1` iff `n_props_a + n_props_b` is odd on an
  uncrossed line). The derivation step that decides it: whether "one −1 per
  internal fermion propagator" can be re-expressed as "one −1 per vertex the
  charge arrow traverses against the rooting direction". If it can, the
  charge-flow rule covers row 2 entirely; if it can only be expressed by
  counting propagators, the arm is i-counting in disguise (each propagator
  carries `−i`) and belongs in the second bucket. **This single question is the
  highest-information step in the investigation** and H4 below is built for it.

**The i-counting observation, stated now so it cannot be retrofitted.**
`kernel.rs:639` records the convention explicitly: "the UFO coupling carries the
vertex `i` and the propagator its `−i`". For a tree diagram with `V` vertices
and `P = V − 1` internal propagators the accumulated phase is
`i^V · (−i)^{V−1} = (−1)^{V−1} i^{2V−1} = i`, **exactly, independently of `V`**.
That is the derivation of row 8's `|G| = 1, Re G = 0` — zero free parameters —
and it is also why row 6 had to be a real −1: the VVVV contact diagram is the
one banked diagram whose line carries *no* propagator, so an extra `−i` there
was an uncancelled 90° phase rather than a convention (commit `eda4412`'s own
message says exactly this). Consequently, of row 8 only **the sign of `G`**
is genuinely fitted, and the charge-flow rule's headline test is whether that
sign, across all 20 processes in `validation/madgraph/amplitudes/`, is a
function of a charge-flow invariant.

### F.2 The pre-registered bar (verbatim) and the verdict taxonomy

> **BAR.** Let `S` be the nine pinned conventions of F.1. A *charge-flow rule*
> is a function assigning a phase to each (diagram, rooting), defined **only**
> from: the diagram's fermion-number/charge arrows, the per-leg incoming/
> outgoing (crossed) bits, the vertex and propagator incidence of each fermion
> line, and the rooting's chosen output leg. It may contain **no per-Lorentz-
> structure special case, no per-process table, and no constant fitted to a
> MadGraph dump**. The rule must be written down **in full, in the findings
> section, before its first comparison**; it may not be amended after seeing a
> mismatch — an amended rule is a *new* rule, recorded as such, and the total
> number of amendments is reported alongside the verdict.
>
> Let `k` be the number of independent binary choices the rule contains (sign
> conventions the author picked rather than derived) and `n` the number of
> pinned binary conventions it reproduces. Only `n − k` is evidence.
>
> - **VERY PROMISING** iff one rule reproduces every member of `S`, *plus* the
>   sign of `G` on every one of the 20 banked processes, with `n − k ≥ 5`; **or**
>   reproduces all of `S` and strictly reduces the fitted-quantity count below
>   today's `1 + Σ_process N_config` (rows 8 and 9 together — 21 configurations
>   on `ud_to_epemud_qcd0` alone), again with `n − k ≥ 5`.
> - **INTERESTING BUT NOT ACTIONABLE** iff the rule reproduces a proper subset
>   of `S` containing at least rows 1, 2, 3, 4a, 4d and 7 (the fermionic arms
>   plus the colour swap) but needs at least one non-charge-flow input for the
>   rest — i.e. the phase *factorizes* as (charge flow) × (i-counting/Lorentz)
>   rather than being determined by charge flow alone.
> - **REFUTED** iff no rule of the stated form reproduces even the fermionic
>   arms. One **witness pair** suffices and is the required form of the negative
>   result: two configurations agreeing on every charge-flow input listed above
>   yet carrying different pinned signs, exhibited concretely from the dumps.
>
> Every reproduced value is checked against `validation/madgraph/amplitudes/
> *.json`, **not** against the current Rust code. A row where the rule and the
> code agree but neither was compared to a dump is recorded as **unchecked**,
> never as green (AGENTS.md: a report is only evidence if every green cell is a
> recorded measurement).

> **Pre-registered prediction P0 (cheap, first hour, resolvable before any
> derivation).** The *strong* reading — "charge flow determines the diagram's
> phase" — predicts that a process with no fermion line anywhere carries
> `fermi_sign ≡ +1` on every diagram. `g g > g g` has no fermion external and no
> fermion propagator. The currently-green assertion at `root_diagram.rs:1253`
> (`channel_counts(&model, "g g > g g").2 > 0`) says its build-sign channel
> *does* fire. **If the research session's own measurement of
> `channel_counts(&model, "g g > g g")` confirms a nonzero third component, the
> strong reading is refuted before any derivation**, and the investigation
> continues only on the factorized reading. The four measured counts must be
> recorded as numbers, not inferred from the test passing.

P0 is deliberately a prediction we expect to fail. Pre-registering it is what
stops the session from quietly reinterpreting the hypothesis after the fact and
reporting a factorized result as if the strong claim had been tested.

### F.3 Hostile cases, in order

Ordered by information per hour; each names what it uniquely discriminates and
the dump it reads. Stop-rule: if H1–H4 produce a witness pair, jump straight to
writing the negative result — H5–H7 are only worth running on a live rule.

1. **`g g > g g` — the fermion-free control** (~1 h). The only banked process
   where the charge-flow input set is *empty*, so any nonzero convention sign is
   a bare counterexample to the strong claim. It is also the only process
   containing a diagram with `V = 1, P = 0` (the 4-gluon contact), which is the
   sole place the i-counting cancellation `i^V(−i)^{V−1} = i` can be tested
   diagram-by-diagram rather than process-wide. Reads:
   `validation/madgraph/amplitudes/gg_to_gg.json` (`points[].detail.amps`,
   6 diagrams × 6 flows, `amp2_groups == [[3],[4],[5]]` — three diagrams carry no
   configuration), plus `channel_counts` from `root_diagram.rs:1204`.
2. **`g g > t t~` — the colour slot swap** (~1 h). Uniquely: the only banked
   process mixing f-derived (imaginary) and T-chain (rational) colour
   contributions, which is what makes the 3/3̄ transpose *observable* at all
   (note 16 §2.4, found in C5c) — everywhere else it complex-conjugates a real
   quantity and hides. Discriminates whether a charge-flow rule already exists
   in the colour sector and whether it is the *same* bookkeeping as the spinor
   one. Reads: `amplitudes/gg_to_ttx.json` (`flow_structures`,
   `detail.jamps`, 2 flows) against `color_cf.rs::gg_to_ttx_flow_structures_untransposed`.
3. **`e+ e- > e+ e-` (Bhabha)** (~1 h). The two arms of row 2 appear inside one
   process with a *relative* sign: the s-channel has one crossed line, the
   t-channel none. Interference makes that relative sign visible where a
   uniform crossed-line count would not be. Reads: `amplitudes/ee_to_ee.json`
   (4 diagrams, `jamp_coefficients = [[-1,0],[-1,0],[1,0],[1,0]]` — MG's own
   `c_i`, a second independent reading of the same signs). Note `ee_to_ee` is one
   of the two `KNOWN_CONFIG_MERGE` rows.
4. **`u d > e+ e- u d QCD=0`** (~2 h) — **the crux case.** 35 diagrams splitting
   24/11 on whether a mixed quark line carries the propagator: the per-propagator
   arm of row 2 at the finest granularity that exists. This is the one test that
   decides the undecided row in F.1, i.e. whether the arm is charge flow or
   i-counting wearing charge flow's clothes. Reads:
   `amplitudes/ud_to_epemud_qcd0.json` (`detail.amps` 35 wide, `detail.jamps`
   2 flows, 21 `amp2_groups`), with the banked pairing
   `MG_DIAGRAM_ORDER["ud_to_epemud_qcd0"]` (`amplitude_oracle.rs:218`) — the
   pairing is banked, not searched, so a mis-derivation here shows as a
   permutation and not as a sign.
5. **`e+ e- > ta+ ta- H`** (~1 h). The only process whose build sign comes
   *solely* from the τ-Yukawa scalar bilinear, so rows 4b (predicted
   not-charge-flow) and 4d (predicted charge-flow) fire on the same vertex and
   only a rule that gets both right predicts the total. Reads:
   `amplitudes/ee_to_tatah.json`, with `MG_DIAGRAM_ORDER["ee_to_tatah"] =
   [3,4,1,2,0]`.
6. **An alternative rooting** (~2 h, conditional on the rule surviving 1–5). A
   rule that is genuinely a *flow* statement must be rooting-covariant in a
   stated way: it must **predict** the compensating factor
   `canonical.reversed_convention_sign() · tree.reversed_convention_sign()`,
   not merely tolerate it. Discriminates rule from bookkeeping. Reads:
   `helas/eval/rooting_soundness.rs::all_rootings_preserve_amplitude`
   (`#[ignore]`, ~52 s, the sweep is **133** re-rootings and currently deviates
   on **0/133**), run with the per-rooting sign dumped by a scratch patch that is
   never committed.
7. **`u u~ > u u~` — multi-flow** (~1 h, conditional). Discriminates whether the
   rule's phase is flow-*independent* (a per-diagram scalar, as `fermi_sign` is
   today) or must be assigned per colour flow. If one per-diagram phase cannot
   serve both flows, the rule is under-determined at exactly the level the
   `color-flow` sprint's slot-swap bug lived. Reads:
   `amplitudes/uux_to_uux.json` (`flow_structures = ["T(2,1) T(3,4)", "T(3,1)
   T(2,4)"]`, `detail.jamps` `[row][flow][re,im]`), against
   `color_cf.rs::uux_to_uux_two_flows`.

`p p > j j` is **not** available for this: it has an MG5 export under
`validation/madgraph/output/` but **no** `amplitudes/*.json`, so the conjugate-rep
territory chain A works in has no per-diagram complex dump. H2/H7 are the
substitutes.

### F.4 Method

Derivation on paper against the banked dumps. Scratch evaluation scripts live in
the worktree and are **never merged**; nothing under `vibegraph-lib/src/` is
edited, `fermi_sign` and the evaluators are not touched, `TODO.md` is not
updated by the research session.

Named inputs the research session reads:

- The 20 tables in `validation/madgraph/amplitudes/` — schema:
  `points[].detail.amps` is `[helicity_row][graph][re,im]`,
  `points[].detail.jamps` is `[helicity_row][flow][re,im]`,
  `detail.helicities` indexes the top-level `helicities` table, `detail` is
  present on 6 points per file. `bbx_to_ccx_emmm_qcd0` and
  `uux_to_ccx_emmm_qcd0` bank **flows only** (615/579 diagrams), so no
  per-diagram check is possible there — record those two as out of scope rather
  than as agreeing.
- `tests/amplitude_oracle.rs` — `fit_constant` (581), the `G` call site (1016),
  the `|G|`/`Re G` assertions (1061-1075), the per-configuration `k` loop
  (1097-1125), `MG_DIAGRAM_ORDER` (218), `KNOWN_CONFIG_MERGE` (183),
  `KNOWN_LINEAR_DISAGREEMENT` (210, **currently empty** — every row is gated).
- **Harvesting the fitted constants**: `G`'s sign is printed per process at
  ~line 1136 (`g.im.signum()`), so `cargo test -p vibegraph-lib --test
  amplitude_oracle -- --nocapture` yields the 20-row `G`-sign dataset. The
  per-configuration `k` phases are *not* printed at all; obtaining them needs a
  scratch copy of the harness. Both are prerequisites for grading the bar and
  neither exists on disk today.
- `helas/eval/root_lorentz.rs` (`build_at_leg` 406-579, `term_reversed_parity`
  676, `pair_crossed` 700, `chiral_correction` 734,
  `standalone_projector_crossed` 654) and `helas/eval/root_diagram.rs`
  (`spine_sign_from_flow` 695, `yang_mills_vvv_sign` 996,
  `compile_single_diagram` ~1042, `channel_counts` 1204,
  `mg_guard_processes_exercise_every_convention_channel` 1237).
- `kernel.rs:238-243` and `:639` for the propagator/vertex `i` convention;
  commit `eda4412` for the VVVV history.

Long runs (`amplitude_oracle`, `all_rootings_preserve_amplitude`) go to the
background with a `chainF_` log prefix. No MG regeneration is needed anywhere in
this chain; gates, if run at all, run with `--skip-deps`.

**Expected cost**: writing the rule down 1–2 h; H1 + H2 ~2 h; H3 ~1 h; H4 ~2 h;
H5 ~1 h; H6 + H7 ~3 h if reached. **≈ 8–12 h of research-session time**, one
`--nocapture` test run and one `#[ignore]`d 52 s sweep, no reference data
regenerated.

### F.5 Risks, vacuity modes, and what this provably cannot decide

**Ways a "success" would be vacuous.**

- **Fitting to the same dumps twice.** Several rows in F.1 were *themselves*
  derived by fitting against these very dumps — row 4a and 4d were pinned by
  `e+ e- > ta+ ta- H` vs MG `AMP()`, row 2's per-propagator arm by the uux 2→6
  class and `ud_to_epemud_qcd0`. A rule that reproduces them is reproducing
  choices already fitted to the same data; the agreement carries no independent
  information unless the rule was constrained *less* than the choices were. This
  is what the `n − k ≥ 5` clause in the bar exists to police, and why the rule
  must be written in full before the first comparison.
- **Deriving something no observable depends on.** `|M|²` and `JAMP2` are blind
  to a phase common to every diagram and flow, and `G` absorbs it by
  construction. A rule that "derives" `G`'s sign has derived a quantity nothing
  in this generator's output depends on. That is bookkeeping elegance, and must
  be reported as such rather than as a physics result.
- **Confirming against the code instead of the reference.** Every gate is green
  today, so "the rule agrees with `fermi_sign`" is guaranteed for any rule
  reverse-engineered from `fermi_sign`. Only agreement with
  `amplitudes/*.json` counts.
- **`gg_to_gg`'s trace-reversal degeneracy.** Flows related by trace reversal
  carry identical JAMPs (`J₁=J₆, J₂=J₄, J₃=J₅`), so H1's per-flow check is
  provably blind to a swap within such a pair. That class is covered by
  `tests/color_flow_tags_oracle.rs` (against `leshouche.inc`), not by the
  amplitude dumps — do not claim H1 closes it.

**What the investigation provably cannot decide.**

1. **Whose convention a residual is.** `G`'s sign is a *relative* convention
   between two codes, and MadGraph's half includes its own arbitrary choice of
   the overall sign of `JAMP(1)` per colour structure (`amplitude_oracle.rs`
   "Known blind spots"). Deciding it would require modelling MG's own colorize/
   JAMP sign conventions from its generated Fortran — out of scope here. An
   unexplained `G` sign is therefore **not** evidence against the rule, and must
   not be graded as a miss.
2. **Any phase common to all diagrams and all flows** (as above — absorbed, and
   unobservable in this tree).
3. **Majorana fermions.** A fermion-*flow* rule's most characteristic prediction
   is the one for fermion-number-violating lines, and the SM UFO has none, so no
   banked oracle can test it. The v0.1 scope is SM-only, so the rule's
   generalization claim is untestable here in principle, not merely unmeasured.
4. **The two 8-point processes** bank flows only, so no per-diagram statement
   about `bbx_to_ccx_emmm_qcd0` / `uux_to_ccx_emmm_qcd0` can be made.
5. **Loop level and non-SM Lorentz structures** — no reference exists in this
   tree.

**Scope risk.** `fermi_sign` is load-bearing and pinned bit-for-bit across 14+
processes; a "cleaner" formulation is worth nothing unless it reproduces every
pinned value, and the sprint forbids attempting it here. If the verdict is *very
promising*, the deliverable is a TODO backlog entry (scope, migration path,
which of the nine rows would become derivations rather than assertions) and
nothing else.

**What this chain provably cannot break.** It edits one note section and, at
most, one TODO entry. It compiles no production change, moves no tolerance, and
regenerates no reference data, so no census cell can move as a result of it.

### F.6 Errors found in the dispatch brief

- The brief says "an alternative rooting from the **165**-rooting sweep". The
  sweep is **133** re-rootings (note 19 §V5, lines 140/173/520; currently
  `0/133` deviating). `165264` appears in `rooting-study-results.md` as a node
  count, unrelated. The brief is internally inconsistent — it also says `0/133`.
- The brief (following §3) calls `G` "the one fitted constant". There is a
  second family: the per-configuration unit phases `k`
  (`amplitude_oracle.rs:1101`), one per configuration per process, of which only
  the *modulus* is asserted. Row 9 above; the bar counts them.
- The brief attributes the VVVV −i to "note 16 §6". §6 mentions it only in
  passing (observation 5, "imaginary VVVV coupling"); the actual record is
  commit `eda4412`, and the resolution was to replace the −i with a real −1 and
  delete `Op::MetricNegI` end to end. The research session should read the
  commit, not only §6.
- `p p > j j` is named in the brief's neighbourhood as slot-swap territory but
  has **no** `amplitudes/*.json` dump, only an MG5 export; the per-diagram
  hostile cases must use `gg_to_ttx` / `uux_to_uux` / `ud_to_epemud_qcd0`.

## Chain F findings (2026-08-03)

Research-only sidecar, executed against the pre-registration in
`## Chain F design (2026-08-02)` (commit `391affd`). No production file is
modified by this section; the instrumentation used to obtain the numbers below
was a scratch patch to `helas/eval/root_diagram.rs`'s test module and to
`tests/amplitude_oracle.rs`, run and then reverted — the working tree was
verified clean before this note was committed. **Rule amendments after first
comparison: 0.**

### F.7 P0 — the pre-registered falsifier, resolved first

P0 asked for `channel_counts(&model, "g g > g g")` as four measured numbers.
Measured (scratch `#[ignore]`d probe in `root_diagram`'s test module, canonical
`VtxIdx(0)` rooting, exactly the function at `root_diagram.rs:1204`):

```
CHANNEL_COUNTS g g > g g = (vvv=3, spine=0, build=1, reversed=0)
```

The third component is **1 > 0**. Per the pre-registration, *the strong reading
— "charge flow determines the diagram's phase", predicting `fermi_sign ≡ +1` on
a process with no fermion line — is **REFUTED**, before any derivation.* The
investigation continues only on the factorized reading, the one weaker reading
the design sanctions.

The per-diagram detail, which the pre-registration did not ask for but which
changes how the refutation should be read:

| diagram | vertices | `sign` | spine | vvv | build | revC·revL | `fermi_sign` |
|---|---|---|---|---|---|---|---|
| D0 | `V_37` (VVVV contact) | +1 | +1 | +1 | **−1** | +1 | **−1** |
| D1 | `V_36`,`V_36` | +1 | +1 | **−1** | +1 | +1 | **−1** |
| D2 | `V_36`,`V_36` | +1 | +1 | **−1** | +1 | +1 | **−1** |
| D3 | `V_36`,`V_36` | +1 | +1 | **−1** | +1 | +1 | **−1** |

So `fermi_sign` is **uniformly −1** across `g g > g g`. Two consequences the
design did not anticipate, both of which matter for grading:

1. The counterexample to the strong reading is a *global* sign, and a global
   per-process sign is exactly the class F.5's "what this provably cannot
   decide" item 2 declares unobservable (it is absorbed by `G`). P0 as
   pre-registered is stated on `channel_counts` — a code-internal quantity —
   not on an observable, so it can refute the bookkeeping claim ("every pinned
   convention on a fermion-free process is +1") and nothing stronger. It does
   refute that claim, decisively; it does not touch a physics claim. This is a
   defect in the falsifier's construction, recorded rather than patched around.
2. Individually, rows 4c and 5 *do* vary across the four diagrams (build:
   `−,+,+,+`; vvv: `+,−,−,−`), and their patterns are complementary, so only
   their product is uniform. Flipping **either alone** makes `fermi_sign`
   non-uniform and is therefore observable; flipping **both together** is not
   observable in this process. That residual two-row degeneracy is broken
   elsewhere: `e+ e- > W+ W-` has vvv `−,−,+` with build uniform, so row 5
   alone is pinned there.

### F.8 What the dumps actually pin — the framework the rest of this uses

Let `A_d = φ_d · H_d` be vibegraph's amplitude for diagram `d`, with `φ_d =
fermi_sign` (the nine-row convention lift) and `H_d` the sign-free current
evaluation. The gate establishes `A_d = G · c_d · AMP_d^{MG}` elementwise at
`1e-12`. A candidate rule `R` replaces `φ` by `ρ` and leaves `H` alone;
substituting, the gate still passes **iff `ρ_d / φ_d` is constant over `d`
within each process**.

So the dump-observable content of the whole inventory is exactly *the pattern of
`φ_d` up to one global sign per process*. Measured (same probe, over the
processes the guard test names plus every hostile case):

| process | n diagrams | `fermi_sign` values | pattern |
|---|---|---|---|
| `g g > g g` | 4 | all −1 | **uniform** |
| `g g > t t~` | 3 | all −1 | **uniform** |
| `e+ e- > e+ e-` | 4 | all −1 | **uniform** |
| `e+ e- > mu+ mu-` | 2 | all −1 | **uniform** |
| `e+ e- > W+ W-` | 3 | all −1 | **uniform** |
| `u u~ > u u~` | 2 | all −1 | **uniform** |
| `u u~ > d d~` | 1 | −1 | **uniform** |
| `e+ e- > ta+ ta- h` | 5 | +1,+1,+1,+1,−1 | **varies 4:1** |
| `u d > e+ e- u d QCD=0` | 35 | 18×+1, 17×−1 | **varies 18:17** |

**Seven of the nine processes carry a `fermi_sign` that is a pure global sign.**
The entire observable content of the nine-row inventory, at the `fermi_sign`
level, is concentrated in two processes. That is the most important structural
fact this investigation found, and it is what caps the bar's arithmetic below.

Note what uniformity is *not*: it is not absence of evidence. A rule predicting
a non-uniform pattern where a uniform one is measured is refuted by that
measurement. Uniformity in `g g > t t~` (crossed top line with 0, 1, 1
propagators across D0/D1/D2) is precisely how the crossed line's
propagator-independence is pinned.

### F.9 The rule, written in full before its first dump comparison

Written after reading `root_diagram.rs` / `root_lorentz.rs` and after the P0
measurement (both pre-registered as preceding it), and **before** any comparison
against `validation/madgraph/amplitudes/*.json`. Disclosure required by F.5's
first vacuity mode: the derivation was informed by the existing doc comment on
`spine_sign_from_flow`, which already contains a partial C-conjugation argument.
It was *not* informed by any dump.

For a diagram `d` rooted at vertex `r`, the SM's fermion lines partition the
external fermion legs into pairs. For a line `ℓ`: `V(ℓ)` vertices, `P(ℓ) =
V(ℓ) − 1` internal propagators, and a crossing bit `x(ℓ) = 1` iff both endpoints
are final-state. At each vertex `v ∈ ℓ` define the **binding orientation**
`ω(v,ℓ) = −1` if the line's charge arrow runs against the UFO slot pairing at
`v` (the bilinear is read reversed), `+1` if along it.

> **RULE R.**  `Φ(d,r) = ε(d) · Π_ℓ [ σ(ℓ) · Π_{v∈ℓ} ω(v,ℓ) ] · κ(d,r)`
>
> - `ε(d)`: parity of the permutation carrying the reference ordering of
>   external fermion legs to the ordering induced by the diagram's line pairing
>   (Wick).
> - `σ(ℓ) = (−1)^{x(ℓ)}`: one −1 per crossed line, the reordering of the
>   conjugated pair.
> - `ω(v,ℓ)`: −1 at every vertex of a line with an initial-state endpoint
>   (enumeration binds all-incoming, the reference binds all-outgoing, so such a
>   line is read against its arrow everywhere, and `C γ^{μT} C⁻¹ = −γ^μ`); +1 at
>   every vertex of a crossed line (read along its arrow).
> - `κ(d,r) = Π_ℓ ω(sink(ℓ),ℓ)`: removes the one `ω` factor the runtime
>   `resolve_bra_ket` already supplies at each line's single non-fermion-output
>   sink.
>
> No per-Lorentz-structure case, no per-process table, no dump-fitted constant.

Collapsing: an uncrossed line contributes `(+1)·(−1)^V·(−1) = (−1)^{V−1} =
(−1)^{P(ℓ)}`; a crossed line contributes `(−1)·(+1)^V·(+1) = −1`. That is
`spine_sign_from_flow` exactly, with `ε` supplying row 1 and `κ` row 3.

**The crux question of F.1, resolved — structurally, not by a dump.** The design
asked whether row 2's per-propagator arm is charge flow or "i-counting in
disguise (each propagator carries −i)". Two findings:

- The i-counting reading is *structurally* impossible. The propagator's `−i` is
  applied in `kernel.rs` (the propagator numerator/denominator), not in
  `fermi_sign`; the two factors live in different places and both are present.
  The spine's per-propagator `−1` is therefore not a repackaged `−i`.
- The two candidate charge-flow phrasings — "one −1 per internal fermion
  propagator" and "one −1 per vertex the arrow traverses against the binding,
  less the one absorbed at the sink" — are **extensionally identical on every
  tree diagram**, because `P(ℓ) = V(ℓ) − 1` identically on a tree line. No dump
  can discriminate them, now or ever, at tree level. H4 can confirm the count;
  it cannot choose the reading. This blind spot is intrinsic to the question,
  not a shortfall of the banked set.

So the crux resolves **in favour of charge flow**, on the structural argument,
with the explicit caveat that the dump evidence is degenerate.

### F.10 Hostile cases, in the pre-registered order

Harvested datasets (none existed on disk, per F.4): `G` per process, the
per-configuration phases `k`, and per-diagram fitted ratios `r_d` against
MadGraph's *bare* `AMP()`. All three came from a reverted scratch patch to
`tests/amplitude_oracle.rs`; every residual quoted is a least-squares fit over
all banked (point, helicity) entries.

**A new instrument, not anticipated by the design.** F.1 row 9 records that the
per-configuration phases `k` are asserted only in modulus and that "the phase of
`k` is entirely free". Measured, across every configuration of every banked
process (**113** configurations: 70 in the single-flow set plus 43 across the
multi-flow set): **`k/G ∈ {+1, −1}` exactly**, every value real to the printed
precision, suite-wide worst residual `1.19e-13` (on `ee_to_mumu_tata_qcd0`).
The phases are not free; each is one bit. Where `jamp_coefficients`
are banked, that bit *is* MadGraph's own `c_j` (verified: `r_d/G =
c_{order[d]}` on `ee_to_ee` `(−1,−1,+1,+1)`, `ee_to_tatah` `(−1)×5`,
`ee_to_mumu` `(−1,−1)`, `uux_to_mumu` `(+1,+1)`). Where they are not banked —
the four multi-flow processes, which are H1, H2, H4 and H7 — `k/G` is the only
per-diagram sign oracle that exists.

**H1 — `g g > g g`, the fermion-free control.** Done as P0 above. Two
corrections to the case as designed. (i) vibegraph enumerates **4** diagrams
against MadGraph's **6** graphs: MG splits the 4-gluon contact into three colour
structures (`amp2_groups = [[3],[4],[5]]` are the three propagator diagrams), so
there is no 1:1 per-diagram pairing to read. (ii) `gg_to_gg.json` banks
`jamp_coefficients: null`, so the production oracle performs **no** per-diagram
comparison here at all; the sign structure is pinned through the six JAMPs and
through `k/G = (−1,−1,−1)` on the three configurations. The i-counting
cancellation `i^V(−i)^{V−1} = i` was checked diagram-by-diagram as the design
asked: D0 has `V=1, P=0 → i`; D1–D3 have `V=2, P=1 → i²(−i) = i`. Consistent,
and consistent with the measurement that **all 20 processes have `Re G = 0` and
`|G| = 1`** — row 8's quadrant is derived by i-counting with zero free
parameters, as pre-registered.
*Blind spot, as F.5 required: trace-reversal-degenerate flow pairs (`J₁=J₆,
J₂=J₄, J₃=J₅`) are invisible here; H1 does not close that class.*

**H2 — `g g > t t~`, the colour slot swap (row 7).** MadGraph's own banked
`flow_structures` are `["T(1,2,3,4)", "T(2,1,3,4)"]` — untransposed, matching
`color_cf.rs::gg_to_ttx_flow_structures_untransposed`. Checked against reference
data, so row 7's *value* is green. **But the check cannot see what it is claimed
to see.** Every SM UFO FFV vertex lists its particles antifermion-first (`V_98 =
[e+, e-, a]`, and `V_135`/`V_137`/… alike), so the 3̄ slot is uniformly first.
Under that uniform ordering, "swap the 3 and 3̄ slots unconditionally" (what note
16 §2.4 recorded) and "index `T(…,i,j)` by the arrow-out leg" (the charge-flow
phrasing) are the same function on every vertex in the model. The dump confirms
the value and is provably blind to *which rule produced it*. Row 7 counts toward
`n`, but its charge-flow content is untested and untestable in the SM UFO.

Also measured here: `φ = (−1,−1,−1)` uniform, with crossed top lines carrying 0,
1, 1 propagators respectively. That uniformity is what pins the crossed arm's
propagator-independence — a rule giving the crossed line a per-propagator factor
predicts `(+1,−1,−1)` and is refuted.

**H3 — `e+ e- > e+ e-` (Bhabha).** `φ = (−1,−1,−1,−1)`, uniform. Decomposed:
`diagram.sign = (+1,+1,−1,−1)` and `spine = (−1,−1,+1,+1)` — the two cancel
exactly. MadGraph's independently banked `jamp_coefficients` are `c =
(−1,−1,+1,+1)`, i.e. `c = −diagram.sign` exactly, and the harvested `r_d/G`
reproduces `c` to ≤ `1.9e-14`. **Row 1 is checked against the dump**: MG's own
per-diagram relative sign between the annihilation-type and exchange-type
pairings equals vibegraph's Wick parity up to one global sign, and that parity is
the permutation parity of the charge-flow endpoint pairing — the `ε` of rule R.

The finding the design's framing missed: because `spine` cancels `diagram.sign`
here, the *relative* sign Bhabha pins lives, on vibegraph's side, in `H_d` and
not in `φ_d`. Bhabha does not pin `fermi_sign`'s relative structure — it pins
the product. It is still evidence (flipping the crossed arm alone makes `φ`
non-uniform and breaks the gate), but evidence about a combination, not about
row 2 in isolation.

**H4 — `u d > e+ e- u d QCD=0`, the crux case.** 35 diagrams, `build ≡ +1` and
`revC·revL ≡ +1` throughout, so `φ_d = sign_d · spine_d · vvv_d`. Harvested
`k/G` for all 35 configurations (`config_diagrams` is the identity, `counts` all
1, so configuration `i` is diagram `i`): `−1` at `{4…11, 18}`, `+1` on the other
26, every residual ≤ `6.6e-15`.

The design predicted a **24/11 split "on whether a mixed quark line carries the
propagator"**. A 24/11 split is indeed present, but it is not that split. Form
`σ_d = (k/G)_d / φ_d` — MadGraph's per-diagram convention read against
vibegraph's sign-free evaluation. Measured: `σ = −1` on `{0…15, 25…32}` (**24
diagrams**) and `+1` on `{16…24, 33, 34}` (**11 diagrams**). Those two sets are
exactly the **neutral-current** and **charged-current** diagrams (the 11 all
carry `V_123`/`V_89`, the W–quark vertices). The split is a *Wick-pairing* (row
1) effect — NC diagrams pair the quark lines `u→u, d→d`, CC diagrams pair them
`u→d, d→u`, a transposition — not the per-propagator arm. The propagator
structure is identical across the divide (D0 and D16 have the same line/prop
pattern and differ only in pairing, hence in `diagram.sign`).

The per-propagator arm's real observable content in this process is a different
partition, and a machine-checked one: `spine = −1` on **11** diagrams —
`{4…11, 18, 21, 22}` — and `+1` on the other **24**, matching
`channel_counts(&model, "u d > e+ e- u d QCD=0").1 = 11`. The 11 are exactly the
diagrams in which **no mixed (initial↔final) quark line carries a propagator**,
so only the crossed line's single −1 survives: D4–D11 and D18 put the propagator
on the *crossed* line, and D21/D22 carry no fermion propagator on any line at
all. The 24 carry one propagator on a mixed line, which cancels the crossed −1.
A rule dropping the per-propagator factor flips `φ` on those 24 and not on the
11 — detectable. D4–D11 and D18 do double duty: they are also where the crossed
arm's propagator-independence is pinned, since a crossed line taking a
per-propagator factor would flip exactly those 9.

*(An earlier revision of this section claimed the split was "D21/D22 against
33". That was a by-hand miscount of the same probe output — the AGENTS.md
"machine-check census claims" failure, committed inside a section arguing for
machine-checked censuses. Corrected above against `channel_counts` and the
per-diagram readout; that both arms of row 2 are checked is unchanged, and the
evidence for each is broader than the miscount suggested.)*

**H5 — `e+ e- > ta+ ta- H`.** `φ = (+1,+1,+1,+1,−1)`; the variation comes
entirely from `build = (−1,−1,−1,−1,+1)`. Per-vertex attribution (scratch dump
of each node's `build_sign`): in D0–D3 exactly one off-shell current carries
`−1`, and it sits on `V_106 = [ta+, ta-, H]` (`FFS4`) rooted at a **fermion**
output — that is `standalone_projector_crossed` (`root_lorentz.rs:654`), **row
4d**. In D4 every node is `+1`, including `V_69 = [Z, Z, H]` (`VVS1`), because
D4's rooting makes a `Z` that vertex's output and the pure-metric arm is only
reached at a scalar or amplitude sink.

Therefore **the design's row 4b attribution is wrong**: `e+ e- > ta+ ta- H` does
not exercise the scalar-sink ProjM/ProjP/Identity `−1` at all — its build sign
is row 4d alone. (The same conflation sits in the production comment on
`mg_guard_processes_exercise_every_convention_channel`'s fourth assertion, which
says the build sign there is "ProjM/ProjP scalar-sink + the crossed-τ standalone
projector"; only the second half fires.) Consequence for the bar: **row 4d is
checked** — `r_d/G = c_{order[d]} = −1` for all five diagrams at residual ≤
`5.7e-15`, so flipping D4's `φ` alone would break the per-diagram gate — while
**row 4b is unchecked**, having no varying instance anywhere in the banked set.

**H6 — alternative rooting.** Conditional on the rule surviving H1–H5 as the
strong reading. It did not (P0), and the design's stop-rule directs the session
to the write-up. **Not run.** The 133-rooting sweep's rooting-covariance claim
therefore stands unexamined by this chain; nothing here touches it.

**H7 — `u u~ > u u~`, multi-flow.** `φ = (−1,−1)`, uniform, with the same
`diagram.sign`/`spine` cancellation as Bhabha (`(+1,−1)` against `(−1,+1)`).
Harvested per-diagram per-flow ratios against the bare `AMP()`:

```
d=0 f=0  r/G = +0.166666667    d=0 f=1  r/G = −0.500000000
d=1 f=0  r/G = +0.500000000    d=1 f=1  r/G = −0.166666667
```

all real (imaginary parts `0.000000000`), residuals ≤ `4.6e-16`. Both diagrams
show the same sign pattern across flows `(+, −)`, the magnitudes being the
colour weights `1/6` and `1/2`. **A per-diagram scalar phase suffices**: the
flow-dependence of the sign is entirely the colour weight, so the rule need not
be assigned per colour flow. That answers H7's question in the rule's favour.

### F.11 Verdict against the pre-registered bar

`S` is graded as the nine members `{1, 2, 3, 4a, 4b, 4c, 4d, 5, 7}` — see F.13
for why `S`'s size is ambiguous in the pre-registration and why this is the
reading adopted.

| row | R covers it? | checked against a dump? | evidence |
|---|---|---|---|
| 1 Wick parity | yes (`ε`) | **yes** | `ee_to_ee`: MG `c = −diagram.sign`, `r/G` reproduces `c` to `1.9e-14`; `ud` NC/CC `σ` split |
| 2 spine, both arms | yes | **yes** | crossed arm: `gg_to_ttx` uniform `φ` over 0/1 props, `ud` D4–D11/D18. per-prop arm: `ud` 11 (`spine = −1`) vs 24 (`spine = +1`) |
| 3 reversed-bilinear parity | yes (`κ`) | **no** | `revC·revL ≡ +1` on every canonically-rooted diagram; content lives in `H`, which no dump separates from `φ` |
| 4a crossed-pair −1 | yes | **no** | never produces a varying `build` pattern in any of the 9 probed processes |
| 4b scalar-sink −1 | no (no arrow) | **no** | does not fire in `ee_to_tatah`; no varying instance found |
| 4c pure-metric −1 | no (no arrow) | yes | `gg_to_gg` D0 vs D1–D3 |
| 4d standalone-projector-crossed | yes | **yes** | `ee_to_tatah` 4:1, `r/G` uniform `−1` at `5.7e-15` |
| 5 Yang–Mills VVV | no (no arrow) | yes | `gg_to_gg`; `ee_to_wpwm` (`−,−,+`) |
| 7 colour 3/3̄ swap | yes | yes (value only) | MG `flow_structures = ["T(1,2,3,4)","T(2,1,3,4)"]`; blind to which rule produced it |

**The arithmetic.**

- `n` (pinned binary conventions R reproduces **and** that a recorded dump
  measurement checks) = **4** — rows 1, 2, 4d, 7. Rows 3 and 4a are reproduced
  but **unchecked**, and the bar's closing clause says an unchecked row is
  "never green".
- `k` (independent binary choices R contains): (i) the global sign of `Φ` per
  process; (ii) the crossed-line reordering sign `σ = −1`; (iii) which of the
  `V` per-vertex `ω` factors is the one absorbed at the sink, i.e. the placement
  of `κ`; (iv) the colour-sector arrow-indexing direction in row 7. **`k = 4`.**
- **`n − k = 4 − 4 = 0`.**

Most generous admissible accounting — count the two unchecked reproductions
(`n = 6`) and grant that (i) is unobservable and (iii) is fixed by the runtime
rather than chosen (`k = 2`) — gives **`n − k = 4`**, still short of the
pre-registered `≥ 5`.

The bar's second VERY PROMISING clause (strictly reduce the fitted-quantity
count below `1 + Σ_process N_config`) also fails: a large reduction **is**
available and is reported in F.12, but it comes from *measurement* of `k`, not
from rule R, and the clause requires the rule to do the reducing.

> **VERDICT: INTERESTING BUT NOT ACTIONABLE.**
>
> Rule R reproduces rows 1, 2, 3, 4a, 4d and 7 — the fermionic arms plus the
> colour swap — and requires non-charge-flow input (i-counting and the vertex's
> Lorentz structure) for rows 4b, 4c and 5. The diagram phase **factorizes** as
> (charge flow) × (i-counting / Lorentz), exactly the middle bucket the design
> defined. It is not VERY PROMISING under any admissible reading of the `n − k`
> clause. It is not REFUTED: no witness pair exists against the fermionic arms.

Two qualifications that must travel with that verdict:

- Membership in the middle bucket requires rows 3 and 4a to count as reproduced;
  both are **unchecked**, and row 3 is unchecked *in principle* at the level R
  operates, not merely unmeasured. A stricter reading — the middle bucket
  requires its six rows to be checked — puts the result below the middle bucket
  with no bucket to fall into. The taxonomy has no cell for "reproduces the
  right rows, but a third of them are unobservable"; that is where this landed.
- The witness pairs the design asked for do exist, but against the *strong*
  claim only. They are recorded in F.12.

### F.12 The negative result, stated with the same care

**Why the rule under-determines the phase.** The nine-row inventory is a
partition of a quantity nothing observes. What the dumps pin is `A_d = φ_d·H_d`;
the split between the convention lift `φ` and the honest evaluation `H` is
internal to vibegraph, and **every reassignment of a sign between the two is
invisible to every banked oracle.** Bhabha and `u u~ > u u~` are the clean
demonstrations: in both, `diagram.sign` and `spine` cancel exactly, `φ` is a
global sign, and the relative sign MadGraph pins sits on vibegraph's side inside
`H`. A charge-flow rule for `φ` is therefore graded against a quantity the
reference never isolates — the deep reason `n` cannot be pushed up by banking
more processes.

**Which conventions escape the rule.** Rows 4b, 4c and 5 involve no fermion
arrow anywhere. Row 4c fires on `g g > g g`'s four-gluon contact, a diagram with
no fermion line and no propagator; row 5 fires on a VVV vertex at index ≥ 1;
row 4b's condition is a scalar or amplitude sink. None of the three has any
charge-flow input to read. They are Lorentz-structure and i-counting facts, and
the factorization is not a defect of R but the shape of the answer.

**Witness pair W1 — the strong reading.** `g g > g g` D0 (the `V_37` contact)
and D1 (two `V_36` vertices). Charge-flow input sets: both **empty and
identical** — no fermion external, no fermion propagator, no line, no arrow, no
crossing bit. Pinned values: row 4c is `−1` on D0 and `+1` on D1; row 5 is `+1`
on D0 and `−1` on D1. Two configurations agreeing on every charge-flow input yet
carrying different pinned signs — the design's required form of the negative
result, delivered against the strong reading. (Their *product* agrees, `φ = −1`
both, which is why the strong reading survives at the level of `φ` while failing
at the level of `S`'s members, and why P0's falsifier had to be sited on
`channel_counts`.)

**Witness pair W2 — the `G`-sign clause.** `e+ e- > mu+ mu-` and
`u u~ > mu+ mu-`: same diagram count (2), same topology (γ/Z in the s-channel),
same fermion-line structure (one initial–initial line, one crossed final line,
no fermion propagator), same crossing bits. Measured `G = −i` and `G = +i`
respectively. The two differ only in the SU(3) representation of the initial
pair, and MadGraph's banked coefficients differ in exactly the compensating way
(`c = (−1,−1)` against `c = (+1,+1)`), so `G·c₀ = +i` for both. Across the 16
coefficient-banked processes, `G·c₀ = +i` for 13 and `−i` for 3
(`ee_to_mumu_tata_qcd0`, `ee_to_wpwm`, `ee_to_zh`). **The sign of `G` tracks
MadGraph's own colour-coefficient sign, not any vibegraph-side invariant** —
F.5's "cannot decide" item 1 confirmed quantitatively rather than assumed. The
bar's headline test, "the sign of `G` on every one of the 20 banked processes",
is therefore a test of MadGraph's convention and is not winnable by any rule of
the pre-registered form.

**A positive by-product: rows 8 and 9 are far smaller than pre-registered.**
`|G| = 1` and `Re G = 0` are derived by i-counting (`i^V(−i)^{V−1} = i`,
independent of `V`) and measured to hold on all 20 processes, so row 8 contains
one bit, not a phase. Row 9 contains one bit per configuration, not a free
phase: `k/G ∈ {±1}` exactly on all 113 configurations measured, suite-wide worst
residual `1.19e-13`. Where `jamp_coefficients` exist, those bits *are*
MadGraph's `c_j`.
The fitted-quantity count is therefore not `1 + Σ_process N_config` free complex
phases but `1 + Σ_process N_config` **bits**, most of them already banked in the
reference. This is measurement, not derivation, and it does not earn the bar's
second clause — but it is the most useful thing this chain produced, and it
turns into a concrete gate hardening (F.14).

### F.13 Errors found in the pre-registration and the dispatch brief

Reported, not patched around, as the brief requires.

1. **`S`'s size is ambiguous.** F.1's prose says "there are **nine** pinned
   conventions" and the BAR says "the nine pinned conventions of F.1", but F.1's
   table has **twelve** numbered rows (1, 2, 3, 4a, 4b, 4c, 4d, 5, 6, 7, 8, 9).
   The only reading that yields nine is `{1, 2, 3, 4a, 4b, 4c, 4d, 5, 7}` — the
   five `fermi_sign` factors with `build` expanded into its four sub-arms, plus
   the colour swap, excluding row 6 (a phase, not a sign) and rows 8–9 (the
   fitted constants, which the bar counts separately). That is the reading
   graded above; a different reading changes `n`'s ceiling but not the verdict,
   since `n − k` falls short even at its most generous.
2. **The bar's clause 1 contradicts F.5's "cannot decide" item 1.** Clause 1
   requires the rule to reproduce `G`'s sign on all 20 processes; F.5 item 1
   says an unexplained `G` sign "is therefore **not** evidence against the rule,
   and must not be graded as a miss". A criterion that cannot be failed cannot
   be passed as evidence. W2 above shows the clause is in fact unwinnable.
   Clause 2 (the disjunct) is unaffected, so the bar survives, but clause 1
   should be struck rather than repaired.
3. **Row 4b's pinning attribution is wrong** (F.1's table, and the same error in
   the production doc comment on
   `mg_guard_processes_exercise_every_convention_channel`). `e+ e- > ta+ ta- H`
   exercises row **4d** (`standalone_projector_crossed`), not row 4b: the `−1`
   sits on the `ta ta H` `FFS4` vertex rooted at a *fermion* output, and the
   scalar-sink arm at `root_lorentz.rs:485/501/557` never fires in that process.
   Row 4b has **no** varying instance anywhere in the banked set and must be
   recorded as unchecked.
4. **All four multi-flow hostile cases lack the per-diagram comparison the
   design assumed.** H1 `gg_to_gg`, H2 `gg_to_ttx`, H4 `ud_to_epemud_qcd0` and
   H7 `uux_to_uux` all bank `jamp_coefficients: null`, so `per_diagram_fit =
   banks_amps && table.coefficients.is_some()` is false and the oracle runs
   **no** per-diagram check on any of them. F.3 describes H4 as reading
   "`detail.amps` 35 wide … with the banked pairing `MG_DIAGRAM_ORDER`" as
   though a per-diagram comparison were available; it is not. The working
   substitute — per-configuration `k/G`, which the design listed only as an
   un-harvested quantity — is what actually carries the per-diagram sign for
   these four.
5. **`g g > g g` is 4 diagrams against MadGraph's 6 graphs**, not "6 diagrams ×
   6 flows" as F.3 states: MG splits the four-gluon contact into three colour
   structures. There is no 1:1 diagram pairing for that process at all.
6. **H4's "24/11 split on the per-propagator arm" mis-describes the data.** The
   24/11 split exists and is the neutral-current / charged-current split, i.e. a
   Wick-pairing (row 1) effect. The per-propagator arm's own partition of the
   same 35 diagrams is 11 (`spine = −1`, `{4…11, 18, 21, 22}`) against 24. The
   crux case still decides the crux, but not by the mechanism the design named,
   and the two 24/11 partitions are different partitions that happen to share a
   shape.
7. **P0's falsifier is sited on a code-internal quantity.** `channel_counts` is
   not an observable, so P0 can only refute a bookkeeping claim. Recorded in
   F.7; noted here because a future pre-registration should site its falsifier
   on a dump-visible quantity.
8. The dispatch brief's own errors were already caught by the design's F.6 (133
   not 165 re-rootings; the second family of fitted phases; `eda4412` rather
   than note 16 §6; `p p > j j` has no amplitude dump). All four confirmed
   correct as F.6 states; nothing to add.

### F.14 Backlog drafts — NOT attempted this sprint

The verdict is not *very promising*, so **no refactor entry is proposed**. The
design's scope risk stands and is reinforced: `fermi_sign` should not be
reformulated on the strength of this, because F.12's central finding is that the
`φ`/`H` split is unobservable, so a reformulation could not be validated beyond
preserving a product the current gate already preserves. Two *other* entries
fell out of the work and are drafted here for the manager to land or discard.

**(a) Assert what `k` actually is (cheap, high value).** `amplitude_oracle.rs`
asserts only `|k| = 1` per configuration. Measured: `k/G ∈ {±1}` exactly on all
113 configurations of the banked set, suite-wide worst residual `1.19e-13`
(`ee_to_mumu_tata_qcd0`) — so any tolerance for this must sit above that, not at
the `1e-15` the per-process figures might suggest. Asserting
`|Im(k/G)| < LINEAR_REL_TOL` — that `k` is a *real* multiple of `G` — converts
a free per-configuration phase into a pinned bit, and would catch any future
defect that rotates a configuration amplitude in the complex plane, a class
`AMP2` is blind to by construction. Scope: one assertion plus its message in the
existing per-configuration loop; no reference data, no tolerance move. Two-way,
per house style: it should fail if a `k` stops being real.

**(b) A real per-diagram sign between `run_config_amps` and the per-diagram
amplitude — structurally inert, worth pinning anyway.** The research session
found this on `ee_to_tatah` and left it as a lead, explicitly not excluding a
scratch-harvest artifact. **The chain F review settled it**, reproducing the
disagreement with production's own evaluators rather than the harvest: the
ratio is exactly `±1` with spread `0.00e0` over 48 samples, so the effect is
**real, not an artifact of the harvest**, and the alternative explanation this
entry originally offered is withdrawn.

It is also broader than one process. The per-configuration sign pattern is
**non-uniform in three processes** — `ee_to_tatah` (`+ + + + −`),
`ee_to_mumua` (`− − − − + + + +`) and `ee_to_mumu_tata_qcd0` (17:8) — and
uniform in the other 11, where a uniform sign is absorbed and harmless.

**Severity is settled: not a live defect, structurally.** `run_config_amps` has
no production consumer at all — `amplitude_oracle.rs` is its only caller — and
`eval_amp2` accumulates `norm_sqr()` incoherently, so no configuration sign can
reach `AMP2`, the configuration draw, or `ICOLAMP`. There is no path by which
this changes an event.

**The research session's proposed next step was wrong and is replaced.**
Asserting `run_config_amps` against the per-diagram amplitudes *directly* —
i.e. demanding they agree — **would fail as stated** on those three processes.
Any such assertion has to permit a per-diagram sign, and once it does it is
nearly vacuous. The useful form instead: **pin the measured sign *patterns* per
process** — bank the 14 per-process sign vectors and assert against them, so
the three non-uniform patterns are frozen as data and a future change that
reshuffles them fails, without asserting a uniformity that is false. Scope: one
banked table plus a comparison in the existing per-configuration loop; no
reference data regenerated, no tolerance moved. Sequencing note: this is
independent of (a) — (a) constrains `k`'s *phase*, this constrains the
per-configuration *sign pattern* — so the "investigate before (a) lands"
condition in the original draft is dropped.

### F.15 What this section provably did not decide

Beyond F.5's own list, which stands unchanged (`G`-sign attribution, phases
common to all diagrams and flows, Majorana lines, the two 8-point flows-only
processes, loop level), this execution adds four:

1. **The `φ`/`H` split** — no banked oracle separates the convention lift from
   the honest evaluation, only their product (F.12).
2. **Row 2's two phrasings** — "per internal propagator" and "per vertex less
   the sink absorption" are extensionally identical on every tree diagram
   (`P = V − 1`), so no tree-level dump can ever discriminate them (F.9).
3. **Row 7's rule** — under the SM UFO's uniform antifermion-first FFV slot
   ordering, "unconditional swap" and "index by the arrow-out leg" are the same
   function; the dump confirms the value and cannot see the rule (H2).
4. **Rows 3 and 4a** — no varying instance exists in the banked set, so they are
   unchecked, and row 3 is unchecked in principle at the `fermi_sign` level
   because `revC·revL ≡ +1` there (F.11).
## Chain C1 design (2026-08-02)

Three independent hard-error additions, one per parser boundary. Each is a
guard added at the point that already fully resolves the field/token in
question, so no new resolution logic is needed — only a check and a new error
variant. All three are unconditional refusals (no `ParsingOptions`-style
override), matching `UnsupportedLpp`'s existing precedent for an
out-of-restricted-scope beam configuration.

### (a) Concrete change list

**1. Beam polarization — `vibegraph-lib/src/runcard.rs`**

- Add a variant to `RunCardError` (next to `UnsupportedLpp`, ~line 138):
  ```rust
  #[error(
      "beam polarization is not supported: polbeam1={polbeam1}, polbeam2={polbeam2} \
       (both must be 0)"
  )]
  UnsupportedPolarization { polbeam1: f64, polbeam2: f64 },
  ```
- In `RunCard::from_values` (~line 266), immediately after the existing
  `lpp1`/`lpp2` check and before constructing `RunCard { .. }`:
  ```rust
  let polbeam1 = f("polbeam1");
  let polbeam2 = f("polbeam2");
  if polbeam1 != 0.0 || polbeam2 != 0.0 {
      return Err(RunCardError::UnsupportedPolarization { polbeam1, polbeam2 });
  }
  ```
  `polbeam1`/`polbeam2` are already `Def::F(0.0)` in `PARAM_DEFAULTS`
  (lines 395–396) — resolved fields, nothing new to parse. The check must be
  `||`, not `&&`: MadGraph allows setting either beam's polarization
  independently, so a card polarizing only one beam is exactly the failure
  mode a `&&` bug would miss (this is what the two acceptance tests below are
  built to catch).

**2. Decay-chain commas — `vibegraph-lib/src/diagrams/parse.rs`**

- Add a variant to `ParseError` (next to `BadParticleTok`, ~line 33):
  ```rust
  #[error(
      "decay-chain process syntax is not supported: '{0}' separates a hard \
       process from a decay chain with ','"
  )]
  DecayChainUnsupported(String),
  ```
- In `parse_process_string` (line 260), as a new **Step 0** before the
  existing Step 1 (`strip_proc_tag`):
  ```rust
  let mut line = s.trim().to_owned();

  // Step 0: reject decay-chain syntax outright. ',' has no other meaning
  // anywhere in this grammar (checked against every proc-card fixture and
  // every banked reference card — none carries one), so its presence
  // unambiguously marks a decay chain rather than a hard process.
  if line.contains(',') {
      return Err(ParseError::DecayChainUnsupported(line));
  }
  ```
  Placing this before any stripping means it fires regardless of where the
  comma sits relative to `@N`/`[...]`/`$$`/`$`/`/` syntax, and it never reaches
  `parse_process_body`'s `>`-splitting — which is what today turns the comma
  into a bogus 3-way split (`"p p > t t~, t > w+ b"` splits on `>` into
  `["p p ", " t t~, t ", " w+ b"]`, so `t t~, t` is misread as a
  required-s-channel list, then fails downstream in `diagrams/mod.rs` as
  `DiagramError::UnknownParticle` — the misleading error the brief refers to).
  Grepped confirmation this is the actual failure path: `tokenize_names(" t
  t~, t ")` → `["t~,",  ...]`-shaped tokens reach `expand_name_list`/feyngraph
  with no particle of that name, is genuinely what produces today's
  misleading message — the new Step 0 preempts it entirely.

**3. `propagators.py` presence — `vibegraph-lib/src/ufo/mod.rs`**

- Add a variant to `UfoError` (next to the other UFO-file variants, ~line 95):
  ```rust
  #[error(
      "custom UFO propagators are not supported: '{file}' defines propagator \
       forms this loader does not read"
  )]
  UnsupportedPropagators { file: String },
  ```
- In `ParsedModel::parse` (line 148), as the first statement in the function
  body, before the `read` closure and the `REQUIRED_SOURCE_FILES` reads:
  ```rust
  pub fn parse(path: &Path) -> Result<Self, UfoError> {
      let propagators_path = path.join("propagators.py");
      if propagators_path.exists() {
          return Err(UfoError::UnsupportedPropagators {
              file: propagators_path.display().to_string(),
          });
      }
      let read = |name: &str| -> Result<String, UfoError> { /* unchanged */ };
      ...
  ```
  Checking presence *before* any required-file read means the refusal fires
  unconditionally on directory contents — it does not depend on
  particles.py/lorentz.py/etc. being well-formed, which is also what keeps
  the hermetic test below trivial (no valid UFO fixture content needed, only
  the file's presence).
  This only guards the on-disk load path. `import model sm` never reaches
  `ParsedModel::parse` at all — `GlobalConfig::load_ufo_with_identity`
  (`vibegraph-lib/src/config.rs:59`) special-cases `import.name == "sm"` and
  returns the interned built-in model directly, so the interned SM is
  unaffected by this check by construction, and no gated row (all SM,
  `import model sm` or bare) can reach it.

None of the three touches `RunCard`'s/`ProcessSpec`'s/`ParsedModel`'s public
shape beyond adding an error variant — no field renames, no new struct.

### (b) Acceptance tests (named)

All hermetic: no network, no fetched reference data, no `extended-validation`
feature. Layer = plain `#[cfg(test)] mod tests` in the same file as the
change (matching each file's existing convention), except the CLI-level pair
which is a new `vibegraph-cli/tests/` integration test file. Run under a bare
`cargo test --workspace` / `cargo test -p vibegraph-lib` / `cargo test -p
vibegraph` on a clone with no `pixi run fetch-*` steps run.

1. `runcard::tests::polbeam1_nonzero_is_rejected` — parse a run-card string
   containing `1.0 = polbeam1` (polbeam2 left at default). Asserts
   `Err(RunCardError::UnsupportedPolarization { .. })` and that
   `err.to_string()` contains `"beam polarization is not supported"`. Fails
   today (silently succeeds); would fail post-fix under a `&&`-instead-of-`||`
   implementation bug (the field this test exists to catch).
2. `runcard::tests::polbeam2_nonzero_is_rejected` — mirror of (1) with
   `polbeam1` at default and `-1.0 = polbeam2` set. Together, (1)+(2) are the
   pair that makes an `&&` bug observable — either alone leaves it hidden.
3. `runcard::tests::unpolarized_default_still_parses` — parse the empty
   card (`RunCard::parse("")`, already implicitly covered by
   `RunCard::default()`'s use in every other runcard test, but stated
   explicitly here as the negative control for (1)/(2): the guard must not
   fire on the untouched default).
4. `diagrams::parse::tests::decay_chain_comma_is_rejected` — call
   `parse_process_string("p p > t t~, t > w+ b", &opts())` (the brief's own
   example) and assert `Err(ParseError::DecayChainUnsupported(_))` with the
   message containing `"decay-chain process syntax is not supported"`. Fails
   today by returning `Ok(ProcessSpec { required_s_channels: ["t~,", ...],
   .. })` (or a `BadParticleTok`/other error further down the stack, depending
   on exact split — either way, not today's actual failure a caller sees,
   which is `DiagramError::UnknownParticle` one layer up in `diagrams/mod.rs`;
   this test pins the boundary at the parser, where the fix lives).
5. `diagrams::parse::tests::decay_chain_comma_after_tag_is_rejected` —
   same but with a trailing `@1` tag (`"p p > t t~, t > w+ b @1"`), proving
   the Step-0 placement catches the comma regardless of what else is on the
   line, not just the brief's exact string.
6. `diagrams::parse::tests::comma_free_processes_still_parse` — negative
   control: re-run a handful of the module's existing non-comma fixtures
   (e.g. `"p p > e+ e- $$ Z"`, `"p p > e+ e- [QCD]"`) through
   `parse_process_string` and assert `Ok(..)`, guarding against an
   over-broad comma detector (e.g. one that fires on a byte inside a
   multi-byte token by accident — not a real risk here since `contains(',')`
   is a plain byte scan, but the existing suite already exercises these
   strings, so this is cheap insurance, not new fixture-building).
7. `ufo::mod::tests::propagators_py_present_is_rejected` — build a fixture
   dir at `std::env::temp_dir().join(format!("vibegraph-ufo-propagators-test-{}",
   std::process::id()))` (the exact idiom `artifact.rs`/`cache/*.rs` already
   use for hermetic filesystem tests), write six **empty** files into it —
   the five `REQUIRED_SOURCE_FILES` plus `propagators.py` — call
   `ParsedModel::parse(&dir)`, assert `Err(UfoError::UnsupportedPropagators {
   .. })` with the message containing `"custom UFO propagators are not
   supported"`, then remove the fixture dir. Content-free files are
   sufficient *because* the check runs before any required-file read (design
   decision above) — this test would fail to compile as easy hermetic
   coverage if the check were placed after the reads, which is itself a
   reason to keep it first.
8. `ufo::mod::tests::propagators_py_absent_still_parses` — negative control:
   same fixture minus `propagators.py`, but with the five required files
   holding minimal syntactically-valid empty-ish UFO content (or, more
   simply, reuse whatever minimal fixture content an existing hermetic UFO
   parse test in this module already builds, if one exists — if the module
   has none, the smallest viable stand-in is acceptable, e.g.
   `particles.py`/`lorentz.py`/`couplings.py`/`parameters.py` each containing
   nothing but a trailing newline and `vertices.py` the same; `parse_particles`
   et al. must already tolerate an empty file for this to pass as `Ok`, which
   the implementer should confirm empirically rather than assume — if it
   doesn't, this control test can instead just assert the error variant is
   *not* `UnsupportedPropagators`, which is weaker but does not depend on the
   rest of the parser accepting empty input). This is the test that would
   catch an inverted condition (e.g. `!path.exists()`).
9. **CLI-level pair**, new file `vibegraph-cli/tests/cli_hard_errors.rs`
   (pattern lifted from `cli_first_run.rs`: temp `cwd`, `VIBEGRAPH_HOME` set
   to a temp dir so `cache_root()` never touches the real `$HOME`,
   `CARGO_BIN_EXE_vibegraph`, `--no-network` passed defensively even though
   none of these three refusals reach network code):
   - `cli_polarized_beam_card_is_refused` — `vibegraph integrate` with a
     minimal fixed-energy (`lpp1 = lpp2 = 0`) run card carrying `1.0 =
     polbeam1`, a comma-free `generate e+ e- > mu+ mu-` proc card (built-in
     `sm` model, no `--ufo-dir` needed), asserts non-zero exit and stderr
     containing `"beam polarization is not supported"`.
   - `cli_decay_chain_proc_card_is_refused` — `vibegraph integrate` with the
     brief's `generate p p > t t~, t > w+ b` proc card (built-in `sm` model)
     and a default run card, asserts non-zero exit and stderr containing
     `"decay-chain process syntax is not supported"`. (Fires at proc-card
     parse, before model/run-card load, so this is reachable with nothing
     else on disk.)
   - `cli_propagators_py_model_is_refused` — `vibegraph integrate` with
     `import model fixturemodel` / `generate e+ e- > mu+ mu-`, `--ufo-dir`
     pointed at a temp dir containing `fixturemodel/` with the six
     content-free files from test 7, and a default run card. Asserts
     non-zero exit and stderr containing `"custom UFO propagators are not
     supported"`. This is the one CLI case that cannot use the built-in `sm`
     model (which never touches disk) — it is also the only one of the three
     that exercises `--ufo-dir` resolution at all, so it is worth keeping
     even though it duplicates test 7's assertion, per the brief's explicit
     ask that the review check these fire from the CLI and not only from
     unit tests.
   All three assert on `Command::new(env!("CARGO_BIN_EXE_vibegraph"))`'s
   captured stderr and `ExitStatus::success() == false` — `main.rs` prints
   `"error: {err}"` via each error's `Display`, so the same message text
   asserted in the unit tests is what a real invocation prints.

### (c) Gates and expected report movement

Expected: **no report cell moves.** This chain closes silent acceptances of
card surfaces that are, by construction, outside the restricted v0.1 scope —
every banked reference card in `validation/madgraph/output/*/Cards/` was
checked directly:
- Every banked `run_card.dat` sets `polbeam1 = polbeam2 = 0.0` (grepped
  across all of them; MadGraph's own default, never overridden in this repo's
  fixtures).
- No banked `proc_card_mg5.dat`'s `generate`/`add process` line contains a
  `,` (grepped across all of them).
- No UFO directory this crate's loader ever reads (the interned SM, or
  `research/ufo` were it used as an on-disk fallback) contains
  `propagators.py` — confirmed absent from `research/ufo`; the
  `propagators.py` files that do exist in the tree
  (`research/refs/mg5amcnlo/models/*_UFO/`,
  `validation/madgraph/output/*/bin/internal/ufomodel/`) belong to MadGraph's
  own installation and its per-run copies, never read by
  `vibegraph::ufo::UFOModel::load`.

So every gate that reads a banked card takes the same path through
`RunCard::from_values`/`parse_process_string`/`ParsedModel::parse` it did
before, hits none of the three new early-return branches, and produces
byte-identical `RunCard`/`ProcessSpec`/`ParsedModel` values. Gates to run
(as regression checks, not as gates expected to flip anything):
- `cargo test -p vibegraph-lib --lib` (covers the three unit-test groups
  above plus every other hermetic lib test — this is the primary check that
  nothing regressed).
- `cargo test -p vibegraph` (covers the new `cli_hard_errors` integration
  test alongside the existing CLI test binaries that don't require
  `extended-validation`).
- `pixi run --skip-deps validate` — the full banked gate, run once after the
  change lands, expected to reproduce the same census (87/85/2 going into
  this chain) with **zero** cells moving. This is the recorded measurement
  the design asks the implementer to capture (before/after `validate`
  summary line or report diff), not merely asserted from "tests passed."

No `pixi run --skip-deps validate-diagrams`/`validate-amplitudes`/etc.
sub-gate is expected to differ either, since none of them exercise a
polarized, comma-bearing, or `propagators.py`-carrying card.

### (d) Risks and what this provably cannot break

**Risks:**
- The `line.contains(',')` decay-chain check is a blunt instrument: if some
  future in-scope syntax legitimately wanted a comma (none does today, and
  MadGraph's own grammar doesn't put one anywhere but the decay-chain
  separator), this guard would need to move past Step 0. Low probability
  inside the v0.1 restricted scope, which explicitly excludes decay chains
  entirely.
- The `propagators.py` check is presence-only, at `ParsedModel::parse`'s
  entry, ahead of the required-file reads. If a future model directory
  legitimately ships a *stale, unused* `propagators.py` (e.g. copied from
  another model by accident) this refusal fires even though nothing would
  have read it — a false positive in principle, but exactly the intended
  behavior per the brief ("presence... must be a hard error until it is
  implemented"), and consistent with the project's hard-error-over-silent-gap
  convention.
- `RunCard::from_values`'s new check reads `polbeam1`/`polbeam2` via the
  existing `f()` closure, which panics (`"no such parameter"`) if the name
  is ever removed from `PARAM_DEFAULTS` — no new panic surface, this is the
  same failure mode every other typed field in that function already has.
- The CLI-level `propagators.py` test is the one place this design asks the
  implementer to touch `--ufo-dir`/`VIBEGRAPH_HOME` resolution, which has
  its own edge cases (`cache_root()` needing a writable/settable home). If
  that proves fragile in the implementer's sandbox, dropping CLI test 9's
  third case and relying on unit test 7 alone for `propagators.py` is an
  acceptable fallback — but the design's preference is to keep it, since the
  brief explicitly asks the review to check CLI firing, not just unit tests.

**What this provably cannot break:** no code path reachable by an
unpolarized, comma-free, `propagators.py`-free card changes at all — every
new check is a guard clause that returns early on a condition no existing
banked card satisfies (per the grep evidence in (c)), and every non-error
return value downstream of each guard (`RunCard`, `ProcessSpec`,
`ParsedModel`) is constructed identically to before once the guard is
passed. The three changes touch three different files with no shared state
and no call-graph overlap between them (`runcard.rs` has no dependency on
`diagrams/parse.rs` or `ufo/mod.rs` and vice versa), so a defect in one
guard cannot manifest as a defect in another. What this design does *not*
provably prevent: the general run-card-ignored-field-audit problem
(Chain C2's job — this chain closes exactly the three named surfaces, not
every parsed-but-unread field), and it does not implement `propagators.py`
support, decay chains, or polarization — it only converts three silent
acceptances into three named refusals.
## Chain E design (2026-08-02)

Five items, in the order they should be implemented — each one lands and is
committed before the next starts, because item 4's report diff is only readable
if nothing else is half-done. Every measurement this design asks for is a
command plus its output; no cell, count or margin below may be inferred from
"the suite passed".

**Read before touching anything.** Two facts decided several choices here and
are load-bearing for the implementer:

- The oracle's out-of-grid probes carry **two** values per point (`xf_raw`, the
  continuation with nothing on top; `xf`, `PDF::xfxQ2` with the clamp). Every
  gate that compares against `xf_raw` must keep reading an **unclamped** member
  after item 1, or it starts failing on 205 probes.
- Regenerating `diagrams.json` from the work area is **not** surgical today: the
  work area now holds four `.mg5`-scripted runs the committed file does not, and
  three of them (`pp_to_jj`, `pp_to_llj_dyn`, `pp_to_ll_scalefact2`) would arrive
  as measurements of cells the manifest declares `uncovered` / `covered-by` —
  which the collator rejects as unexpected cells, and `pp_to_jj` would fail the
  count gate outright (15 topologies against MadGraph's 17). Item 4 fixes the
  selector before regenerating.

### E.1 `ForcePositive`

**What LHAPDF does**, from `src/PDF.cc:49` (6.5.3, identical in the installed
6.5.6): physical-range checks, then `id == 0 → 21`, then
`if (!hasFlavor(id2)) return 0.0;`, then `_xfxQ2` (which is where the
in-range/out-of-range split lives), then the switch — `0` nothing, `1`
`if (xfx < 0) xfx = 0`, `2` `if (xfx < 1e-10) xfx = 1e-10`, anything else a
`LogicError`. Two orderings matter and are testable: the clamp sits **outside**
the interpolate/continue split (so it applies to both), and **after** the absent
flavour's exact zero (so an absent flavour reads `0.0`, never `1e-10`).

The resolved level is `info().get_entry_as<unsigned int>("ForcePositive", 0)`
through `PDFInfo → PDFSet → Config`. Measured in this tree:
`NNPDF31_lo_as_0130.info` line 7 carries `ForcePositive: 2`;
`NNPDF23_lo_as_0130_qed.info` carries no such key and the installed
`share/LHAPDF/lhapdf.conf` says `ForcePositive: 0`. Reading the set's own
`.info` with a default of `0` therefore reproduces LHAPDF here — and that
"therefore" is a hypothesis, so it is pinned against the oracle's own resolved
value rather than left as a reading of a config file, exactly as `extrapolator`
already is.

**Changes.**

| file | change |
|---|---|
| `vibegraph-lib/src/pdf/grid.rs` | `SetInfo` gains `pub force_positive: i32`. `parse_info` fills it from the optional `ForcePositive` key: absent → `0`; present → `parse_num::<i32>` and then a range check, where anything outside `0..=2` is `GridError::InvalidValue { key: "ForcePositive", expected: "0, 1 or 2" }`. Absent must stay absent-legal — the module's own `.info` fixtures do not carry the key. |
| `vibegraph-lib/src/pdf/mod.rs` | `PdfMember` gains a private `force_positive: i32`, defaulted to `0` in `from_subgrids` (which keeps its signature, so every in-memory fixture in `proton.rs` and elsewhere compiles unchanged and stays unclamped). Add `pub fn with_force_positive(mut self, level: i32) -> Self` and `pub fn force_positive(&self) -> i32`. `PdfSet::member` returns `PdfMember::from_subgrids(subgrids).with_force_positive(self.info.force_positive)`. |
| `vibegraph-lib/src/pdf/mod.rs` | `try_xfx_q2` applies the clamp on the way out, in LHAPDF's order. Sketch, and the shape is deliberate: <br>`if !self.interp.has_flavor(pdg) { return Ok(F::zero()); }` before the range split (the trait method already exists and both branches already return zero there, so this moves the check rather than adding one), then `let value = if in_grid_range { … } else { … }?;` and `Ok(force_positive_clamp(self.force_positive, value))`. |
| `vibegraph-lib/src/pdf/mod.rs` | New private `fn force_positive_clamp<F: Real>(level: i32, value: F) -> F`, written as an explicit comparison — `1 => if value < F::zero() { F::zero() } else { value }`, `2 => if value < floor { floor } else { value }` with `floor = F::from(1e-10).unwrap()` — **not** `Float::max`, which returns the non-NaN operand and would silently clamp a NaN where LHAPDF's `if` passes it through. `0 => value`; any other level is unreachable because the parser refused it, and is a `panic!` naming the level. |

That is the whole production change. Nothing else constructs a `PdfMember`: the
CLI's two call sites (`integrate.rs:302`, `generate.rs:432`) go through
`PdfSet::member`, and the PDF cache stores set *directories*, so the cached route
loads through the same `PdfSet::load` + `member`.

**The `FORCE_POSITIVE_FLOOR` re-read.** The constant stays at `1e-8` and its
value is not touched — but its doc comment is now false ("vibegraph applies no
such clamp … a clamp the gate deliberately does not test") and must be rewritten
to say what the screen still buys: it absorbs the band around the floor where
this crate's raw log-bicubic and LHAPDF's could land on opposite sides of
`1e-10`. The measured width of that band, from the committed oracle, is the
justification and should be quoted in the comment: on `oracle_multigrid.json`
the smallest in-range value strictly above the floor is `1.1039e-10` (10.4 %
above it) and 84 in-range points sit exactly at it, so nothing in the probe set
is within nine orders of the straddle. The coverage the clamp earns is claimed
by a **new** test rather than by tightening this one (E.1's acceptance tests
below); if that new test fails, it is a straddle finding to report, never a
reason to move a screen.

**Which gate reads which member.** After the change `PdfSet::member` returns a
clamped member for NNPDF31, and three existing gates compare against the
unclamped `xf_raw`. Add two helpers to `validate_pdf_grid.rs` and route every
test through one of them explicitly:

```rust
fn load_member(oracle: &Oracle) -> PdfMember          // the set's own level: what MadGraph reads
fn load_unclamped_member(oracle: &Oracle) -> PdfMember // load_member(..).with_force_positive(0)
```

| test | member |
|---|---|
| `multigrid_off_knot_interpolation_matches_lhapdf` | clamped (compares against `p.xf`) |
| `multigrid_seam_interpolation_matches_lhapdf` | clamped |
| `multigrid_value_is_continuous_across_seams` | clamped |
| `extrapolation_matches_lhapdf_past_every_grid_boundary` | **unclamped** (`xf_raw`) |
| `the_branch_of_the_upper_continuation_is_the_one_the_endpoint_values_select` | **unclamped** — it reconstructs `y_lo`/`y_hi` from this crate's interpolator, and LHAPDF's extrapolator reads its endpoints below the clamp |
| `the_upper_continuation_misses_lhapdf_by_one_ulp_of_its_own_conditioning` | **unclamped**, same reason |
| `the_only_difference_from_madgraphs_own_value_is_the_positivity_clamp` | both |
| `multigrid_on_knot_values_match_oracle_exactly`, `on_knot_*`, every `alpha_s` test | unaffected — they read `SubGrid::xf_at` or the `AlphaS_*` block, neither of which the clamp may touch |

The last row is a free tripwire: 22 of the multigrid oracle's `knot` values are
negative, so a clamp misplaced into the interpolator or into `xf_at` fails
`multigrid_on_knot_values_match_oracle_exactly` immediately.

### E.2 `validate_kt_cluster` becomes an oracle-layer gate

**Changes.**

- `vibegraph-lib/tests/validate_kt_cluster.rs`: delete the
  `if !dumps.is_dir() || !manifest_path().is_file() { println!(…); return; }`
  block at the head of `the_clustering_engine_reproduces_madgraphs_own` and
  replace it with a hard failure naming both paths and the task that builds
  them — a plain `assert!`/`panic!`, **not** `vibegraph::validation::require`,
  whose message tells the reader to run `pixi run validate` and would be wrong
  for a gate that layer no longer runs.
- Same test: add
  `#[ignore = "oracle layer: the 75 MB kT dumps are outside the reference bundle; `pixi run -e madgraph validate-kt-cluster` builds and runs them"]`.
  Registration, not a runtime skip — the same mechanism the full
  rooting-soundness sweep already uses. `required-features =
  ["extended-validation"]` stays as it is in `vibegraph-lib/Cargo.toml`.
- `pixi.toml`, `[feature.madgraph.tasks]`, beside `generate-kt-cluster-dumps`:
  ```toml
  validate-kt-cluster = { cmd = "cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_kt_cluster -- --ignored --nocapture", depends-on = ["generate-kt-cluster-dumps"] }
  ```
- `pixi.toml`, the `validate-deep` stub's printed inventory: add a line for the
  kT clustering replay, so the oracle layer's list of what it owes is complete.
- `validation/manifest.toml`, in the standalone block (alphabetical position
  next to `color-flow-tags` is fine; keep the file's existing loose grouping):
  ```toml
  [[standalone]]
  key = "kt-cluster"
  layer = "oracle"
  task = "validate-kt-cluster"
  environment = "madgraph"
  targets = ["vibegraph-lib/tests/validate_kt_cluster.rs"]
  inputs = ["validation/madgraph/kt_cluster_dump_manifest.json", "output/ktdump/dumps (75 MB, outside the bundle)"]
  rationale = "The finest oracle in the tree: 90000 events of MadGraph's own clustering intermediates -- every candidate pair with the arm of the measure it took, every merge with its leg sets and scale, the beam walk, both scales -- replayed merge by merge and compared in order, so the first divergence is reported by merge index. Its dumps are 75 MB and deliberately outside the reference bundle, which is what puts it in this layer rather than the banked one."
  note = "The dumps are absent on every fetching checkout, so in the banked layer this gate could only be green without having compared anything. Here their absence is a failure naming them and the task that builds them."
  ```
  No `row` key: the row-file branch of the renderer is Pythia-shaped (it reports
  consumed events and a negative control) and would render nonsense here.
- `validation-report/src/render.rs`, `standalone_verdict`, the no-`row` branch:
  make it layer-aware, so a gate this invocation did not run does not read as
  one that did.
  ```rust
  let Some(row) = standalone.row.as_deref() else {
      if standalone.layer == "oracle" {
          return format!(
              "the oracle layer runs it — `pixi run{} {}` ({})",
              standalone.environment.as_deref().map(|e| format!(" -e {e}")).unwrap_or_default(),
              standalone.task.as_deref().unwrap_or("<task>"),
              standalone.targets.join(", "),
          );
      }
      return format!("ran with the {} layer's suite ({})", standalone.layer, standalone.targets.join(", "));
  };
  ```
- `validation-report/src/main.rs`, where the manifest is checked against the
  measurements: assert every `[[standalone]]` row's `layer` is one of
  `hermetic` / `banked` / `oracle`, and that a row declaring `oracle` carries a
  `task`. The field is a free `String` today, so a typo would render silently;
  the layer set is a declaration and gets enforced like the rest of them.
- `validation/manifest.toml` header, the `## Layers` block: it says a check
  declares its layer by where it is registered. Add the oracle layer's Rust
  form to that sentence — `#[ignore]` plus a task that passes `--ignored` — so
  the mechanism is documented where the layers are.

### E.3 The `release-debug` contract tests

The mechanism is not what `TODO.md` records. `eval_m2_pruned_rejects_boosted_frame`
does not fail because of a `debug_assert!`: the frame guard it exercises is a
plain `assert!` inside
`#[cfg(any(debug_assertions, feature = "extended-validation"))]`
(`vibegraph-lib/src/helas/eval/run.rs:762`), and the test fails wherever that cfg
is false — `cargo test --profile release-debug` with **default** features. Under
the banked layer's own invocation (`release-debug` *plus*
`extended-validation`, which is what `validation/validate.sh` runs) the guard is
compiled and the test passes today.

That distinction decides the fix. Gating the test on `cfg(debug_assertions)`
alone — the brief's wording — would drop it from the banked configuration where
it currently runs and passes, trading a build error for a silent coverage loss.

**Change.** Put the guard's own predicate on the test, which is the form its
sibling twenty lines below already carries:

```rust
#[test]
#[cfg(any(debug_assertions, feature = "extended-validation"))]
#[should_panic(expected = "partonic-CM kinematics")]
fn eval_m2_pruned_rejects_boosted_frame() {
```

**Siblings.** Eight `#[should_panic]` tests exist. Six panic unconditionally
(`panic!`, `assert!`, or `.expect` on a `checked_mul`) and are already clean
under any profile: `validation.rs::a_missing_input_names_itself_and_fails`,
`helas/color/tests.rs::coeff_multiply_overflow_panics`,
`helas/eval/prop_harness.rs::driver_catches_disagreement`,
`coupling/alphas.rs::non_positive_scale_panics`,
`validation/samples.rs::an_unknown_strategy_is_refused_rather_than_guessed`,
`tests/validate_sigma.rs::a_row_the_bundle_carries_may_not_be_absent`. The
seventh, `one_shot_validation_catches_corrupted_momentum_route`, already carries
the cfg. That reading is from grep and is **not** the acceptance criterion: the
criterion is the measured run below, and any further failure it turns up is
either the same class (a test asserting a guard that a profile compiles out →
mirror the guard's cfg) or an escalation.

### E.4 The two uncovered `ud_to_epemud_qcd0` cells

#### (c) `diagrams` — the counts, banked

The extractor's committed-file selector is the problem. It writes a key for
every `validation/madgraph/scripts/*.mg5` stem that has a work-area directory,
which today is a superset of what the manifest declares measurable. Measured
against the current tree: `diagrams.json` holds 25 keys, and those 25 are
**exactly** the manifest rows whose `diagrams` tier is `hermetic` — the rule is
already true and merely unenforced. A plain regeneration would add four keys,
three of which are cells no gate may measure.

**Changes, in this order.**

1. `validation/manifest.toml`: `ud_to_epemud_qcd0`'s `diagrams` cell becomes
   `{ tier = "hermetic", mode = "gate", note = "…" }`. The note records the
   measurement (35 topologies against MadGraph's `NGRAPHS = 35` over the single
   `P1_qq_llqq` class) and drops the "not in diagrams.json" wording.
2. `validation/madgraph/extract_diagrams.py`: replace
   `validated = {s.stem for s in (script_dir / "scripts").glob("*.mg5")}` with a
   set read from `validation/manifest.toml` — every `[[process]]` whose
   `categories.diagrams.tier == "hermetic"`. `tomllib` is stdlib on the
   `madgraph` environment's Python 3.11, so no dependency moves. A declared row
   with no work-area directory is an error naming the row (the committed file
   may not silently lose a gated row); a work-area directory with no
   declaration is skipped as it is today. Update the module docstring and
   `COMMITTED_HEADER` to state the new rule: the committed reference covers the
   rows the manifest declares hermetic, so its content is a function of the
   manifest and the work area rather than of which runs a machine happens to
   have.
3. Regenerate, without MadGraph. The extraction is a pure function of the
   existing work area, but the task carries `depends-on = ["build-diagrams"]`,
   which regenerates any missing process directory through `build.sh`:

   ```sh
   pixi run --skip-deps -e madgraph extract-diagrams
   ```

   `--skip-deps` is not optional. Verify immediately with
   `git diff --stat validation/madgraph/diagrams.json` and read the diff: it
   must add exactly one block,
   `"ud_to_epemud_qcd0": {"diagrams_by_subprocess": {"P1_qq_llqq": 35}, "total_diagrams": 35}`,
   and change nothing else. Anything else in that diff is a stop-and-report.
   (The per-directory `output/*.json` files it also rewrites are gitignored
   work-area artifacts.)
4. `vibegraph-lib/tests/validate_madgraph_diagrams.rs`: add a trial
   `diagrams_json_covers_exactly_the_hermetic_rows` asserting set equality
   between `diagrams.json`'s keys and the manifest's hermetic-`diagrams` rows,
   naming both differences. It reads two committed files, so it stays hermetic.
   The manifest reader belongs in `vibegraph-lib/tests/common/manifest.rs`
   beside `unbundled_rows` (the module is `#![allow(dead_code)]`, so adding a
   function costs the other binaries nothing).

#### (a) `samples` — the comparison, written

**Changes.**

- `vibegraph-lib/tests/validate_samples.rs`: one entry appended to `ROWS`,
  `key: "ud_to_epemud_qcd0"`, `process: "u d > e+ e- u d QCD=0"`. Everything
  else is generic — `with_integrand` reads the run's own card (`lpp1 = lpp2 =
  0`, `ebeam1 = ebeam2 = 250`, so the fixed-beam frame assertion it already
  makes holds), compiles the cuts, and builds the multichannel integrand.
- Budget: start at `neval: 60_000, niter: 6`, the file's own budget for the
  other `2 → 4` fixed-beam row (`ee_to_mumu_tata_qcd0`). If a seed reports
  "produced N of 20000 events" (the `MAX_TRIALS_PER_EVENT` ceiling), raise to
  the σ gate's `120_000 × 8` and record the efficiency that forced it. Do not
  lower `EVENTS_PER_SEED`, `GEN_SEEDS` or `MAX_TRIALS_PER_EVENT`.
- `P_FLOOR` does **not** move. Its doc comment counts the rows the floor was
  chosen against ("twelve fixed-beam rows and four proton ones"); that count
  becomes thirteen and the comment is updated, with the arithmetic it states
  re-checked — at `1e-4` and a few hundred draws from the null the expected
  spurious-failure count stays under `0.05`.
- `validation/manifest.toml`: the `samples` cell becomes
  `{ tier = "banked", mode = "…", note = "…" }` with the measurement in the
  note (min KS p, min χ² p, over which seeds and how many events).

**The mode is decided by a rule stated before the measurement**, so the outcome
cannot be argued into either box afterwards: if every column of every seed
clears `P_FLOOR`, the cell is `mode = "gate"` and `Row.mode = "gate"`; if any
column falls below it, the cell is `mode = "info"` with the failing column, its
p-value and its seed in the note, the row is `Row.mode = "info"`, and the
implementer reports the disagreement to the review session rather than tuning
anything. The manifest's mode and the `Row`'s mode must agree — the collator
fails on a measurement that disagrees with its declaration.

### E.5 `validate_scales`: the four grid-`αs` runs join the `AQCDUP` oracle

`banked_events_reproduce_aqcdup_from_the_computed_scale` steps over
`GRID_ALPHA_S_RUNS` (`pp_to_bb_fixed`, `pp_to_jj`, `pp_to_llj_dyn`,
`pp_to_llj_fixed`) because `RunningAlphaS::from_run_card` refuses a
`pdlabel = lhapdf` card. The library already carries the arm that does not:
`AlphaSSource::from_run_card(card, param_card_as, grid)` returns
`AlphaSSource::Grid(GridAlphaS)` for exactly those cards, and `validate_alphas.rs`
already uses it — with the *printed* `SCALUP` as the scale. Feeding it this
gate's *computed* `μR` instead is what closes cluster scale → `μR` → `αs(μR)` in
one per-event comparison.

**Changes.**

- Move `PDF_SET_BY_LHAID` and `set_alpha_s_info` out of
  `vibegraph-lib/tests/validate_alphas.rs` into a new
  `vibegraph-lib/tests/common/pdfset.rs` (`pub mod pdfset;` in `common/mod.rs`),
  used by both binaries. A lhaid → set-name table transcribed twice is exactly
  the kind of thing that rots; and `validate_alphas.rs` must keep behaving
  identically, which its own run proves.
- `validate_scales.rs`: replace `RunningAlphaS::from_run_card(&card, a_s)` and
  its `Err` arm with
  `AlphaSSource::from_run_card(&card, a_s, common::pdfset::set_alpha_s_info(&card).as_ref())`,
  keeping `.eval(q)` at both call sites (the value and the `moved` budget). Drop
  the `grid_alpha_s` accumulator and its `assert_eq!` against
  `present(GRID_ALPHA_S_RUNS, &runs)`; the run count assertion loses its
  `- present(GRID_ALPHA_S_RUNS, &runs).len()` term. `GRID_ALPHA_S_RUNS` itself
  stays — it is still the classification, now of which arm each run takes rather
  than of which runs are skipped — and its doc comment is rewritten to say so.
  Nothing else in the test body changes: the per-event budget
  (`printed_half_ulp(aqcdup, 7) + moved`) and `TIE_BREAK_MISSES` are untouched.
- `the_grid_alpha_s_runs_are_refused_for_a_measurable_reason` measures something
  that is still true and is now the negative control rather than a justification
  for a skip: substituting the parameter card's `αs(M_Z)` for the grid's own
  reading misses the printed field by well over its budget on every event. Keep
  the body, rename to
  `the_grid_runs_need_the_grids_alpha_s_and_not_the_parameter_cards`, and
  rewrite the doc comment around what it now guards — that the two arms are not
  interchangeable, which is the only thing that makes the new arm's agreement
  informative.
- `validation/manifest.toml`, the `scales-replay` standalone row: `inputs` gains
  the fetched PDF set, since the gate now loads one. The `rationale` gains the
  chain it closes.

**Expected outcome, stated in advance.** `validate_alphas.rs` already reproduces
`AQCDUP` from the *printed* scale through the same grid on all four runs, and
this gate already reproduces the printed scale from the momenta on all four. The
new comparison is the composition of two passing statements, so it should pass;
if it fails, the failure localises to the composition — the computed `μR`, not
`αs` — because each factor is independently gated. `pp_to_jj`'s nine tie-break
events are already declared in `TIE_BREAK_MISSES` and the assertion is
`outside <= allowed`, so they cannot force a tolerance question either way.

---

### (b) Acceptance tests

Existing tests whose meaning changes:

| test | what it now asserts, and what would fail it |
|---|---|
| `the_only_difference_from_madgraphs_own_value_is_the_positivity_clamp` | keeps the oracle-internal relationship (`xf == clamp(xf_raw)` per level) and gains this crate's side: for every probe where the clamp fired, `load_member(..).try_xfx_q2(..)` is **bit-equal** to `probe.xf`; for every probe where it did not, the clamped and unclamped readings are bit-equal to each other. Counts asserted, not printed: `205` clamped of `935` on `oracle_multigrid.json`, `0` of `1190` on `oracle.json`, and the multigrid count asserted `> 0` so the test cannot go vacuous if the probe set is regenerated. Fails on: a clamp that is not applied, applied at the wrong level, applied to the wrong branch, or applied unconditionally (the level-0 set carries continued values down to `−1.1e-5` that must survive). |
| `extrapolation_matches_lhapdf_past_every_grid_boundary`, `the_branch_of_the_upper_continuation_…`, `the_upper_continuation_misses_lhapdf_by_one_ulp_…` | unchanged assertions, now explicitly against an unclamped member. Fails if the clamp leaks into a comparison whose reference has none. |
| `multigrid_off_knot_interpolation_matches_lhapdf`, `multigrid_seam_interpolation_matches_lhapdf` | unchanged bar (`1e-9`) and unchanged screen (`1e-8`); the 84 floored in-range points now agree exactly instead of being screened. |
| `the_clustering_engine_reproduces_madgraphs_own` | absent dumps are a failure naming them and the task, instead of a `println!` and a green return. Fails on: a fetching checkout running it at all — which is the point, and why it is `#[ignore]`d out of the banked suite. |
| `eval_m2_pruned_rejects_boosted_frame` | same assertion, compiled exactly where the guard it exercises is. Fails on: the guard being removed while the cfg stays. |
| `banked_events_reproduce_aqcdup_from_the_computed_scale` | four more runs, `20 → 24`, and roughly 40 000 more events. Fails on: a computed `μR` that misses the grid's own `αs` at the printed field's precision on any of the four. |
| `the_grid_alpha_s_runs_are_refused_for_a_measurable_reason` → `the_grid_runs_need_the_grids_alpha_s_and_not_the_parameter_cards` | same measurement, new job. |
| `unweighted_samples_agree_with_madgraphs_banked_ones` | one more row. |
| the `diagrams` gate | one more trial, `ud_to_epemud_qcd0`, at `35 = 35`. |

New tests:

| name | file | fails on | provably cannot detect |
|---|---|---|---|
| `an_in_grid_value_lhapdf_floors_is_floored_here_too` | `validate_pdf_grid.rs` | any in-range oracle point whose `xf` is exactly `1e-10` where this crate does not return exactly `1e-10`; the count of such points asserted `> 0` on the multigrid set and `== 0` on `oracle.json`. Print, per set, the count and the smallest `|our unclamped reading − 1e-10| / 1e-10` over those points (via `with_force_positive(0)`), so the straddle margin is a recorded number and a later session can decide the screen's fate with data. | a point where **both** LHAPDF and this crate floor a value that neither should have floored — the oracle carries no `xf_raw` for in-range points, so the clamp's *input* is not visible in range. Only the out-of-grid probes see both levels. |
| `the_clamp_level_is_the_one_lhapdf_resolved` | `validate_pdf_grid.rs` | `PdfSet::load(..).info.force_positive != oracle.force_positive` on either set. Pins the `.info`-only reading against LHAPDF's `PDFInfo → PDFSet → Config` chain, so a build whose `lhapdf.conf` differs fails here instead of silently redefining the reference. | a level that is wrong in the *same* way on both sides — i.e. a set whose `.info` and whose resolved value agree and are both not what MadGraph linked. Nothing in this tree can see that; it would need MadGraph's own PDF call. |
| `force_positive_clamp_matches_lhapdfs_switch` | `pdf/mod.rs` unit tests | level 0 altering anything; level 1 not flooring a negative or altering a positive; level 2 not flooring `9.4e-11` to exactly `1e-10` or altering `1.34e-10`; a NaN being clamped at any level (LHAPDF's `if (xfx < 1e-10)` is false for NaN and passes it through). | that the clamp is *called* — it is a unit test of the function, and the wiring is what the oracle tests above cover. |
| `an_absent_flavour_is_zero_and_not_the_floor` | `pdf/mod.rs` unit tests | a level-2 member returning `1e-10` for a PDG code its subgrids do not carry, in range and out of it. This is `PDF.cc`'s `if (!hasFlavor(id2)) return 0.0;` ordering, and it is invisible to every oracle gate because `gen_oracle.cpp` probes only `gpdf.flavors()`. | which flavours a *real* set carries — it runs on an in-memory fixture. |
| `an_info_without_forcepositive_reads_as_the_config_default` / `an_unknown_forcepositive_level_is_refused` | `pdf/grid.rs` unit tests | a missing key becoming anything but `0`; `ForcePositive: 3` parsing instead of erroring. | whether `0` is the right default for a *different* LHAPDF installation — that is what `the_clamp_level_is_the_one_lhapdf_resolved` covers against the two real sets. |
| `diagrams_json_covers_exactly_the_hermetic_rows` | `validate_madgraph_diagrams.rs` | a manifest row declared `diagrams = hermetic` with no counts committed, or committed counts for a row the manifest does not declare hermetic — which is precisely the state a wholesale regeneration would leave. | whether a committed *count* is right; it compares key sets, and the per-row trials compare the numbers. |

### (c) Gates to run, and the cells expected to move

Every command below is backgrounded with its output to a `chainE_`-prefixed log
if it can exceed ~2 minutes; the banked suite and the kT replay certainly do.

| # | command | what it establishes |
|---|---|---|
| 1 | `cargo test --workspace` | the hermetic layer is still complete on default features (items 1, 3, 4c touch it). |
| 2 | `cargo test --workspace --profile release-debug` | E.3's acceptance: clean. Record the full pass/fail list before and after the change — "it passed" is not the evidence, the two lists are. |
| 3 | `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --lib -- eval_m2_pruned_rejects_boosted_frame --exact --nocapture` | the test still **runs** in the banked configuration (`1 passed`), i.e. the cfg gate did not delete the coverage. Run the same command without the feature and record `0 passed; … filtered out`. |
| 4 | `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_pdf_grid -- --nocapture` | item 1, with the printed per-category counts and the clamp/straddle numbers. |
| 5 | `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_scales -- --nocapture` and `--test validate_alphas -- --nocapture` | item 5, and that moving the helper left `validate_alphas` unchanged. Record the printed `runs`/`events` counts: `20 → 24` runs. |
| 6 | `pixi run --skip-deps validate` (or `bash validation/validate.sh` with the inputs in place) | the banked layer end to end, and the report the census is read from. `--skip-deps` is mandatory. |
| 7 | `pixi run --skip-deps -e madgraph validate-kt-cluster` | item 2's gate, run once in its new layer. Expect the `90 000 events … merge sequences and scale pairs reproduced` summary and a pass. |
| 8 | `git diff --stat validation/madgraph/diagrams.json` + the full diff | item 4c: exactly one added block. |

**Report cells.** Render the report before the first change and diff it against
the final one (`§Z.5`'s method: normalise the footnote indices away, then
compare cell by cell).

| cell | before | after |
|---|---|---|
| `ud_to_epemud_qcd0` · `diagrams` | `uncovered` | ✅ hermetic (`35 = 35`) |
| `ud_to_epemud_qcd0` · `samples` | `uncovered` | ✅ banked, or ⚠️ informational under the pre-registered rule above |
| census | `87 measured (85 ✅, 2 ⚠️, 4 ⏳, 8 ⛔, 17 — / uncovered)` | `89 measured (87 ✅, 2 ⚠️, …, 15 — / uncovered)`, or `88 ✅ / 3 ⚠️` if the samples cell lands informational |
| standalone table | 24 rows | 25 rows, the new one reading `the oracle layer runs it — pixi run -e madgraph validate-kt-cluster` |

**Cells that must not move**, and this is an assertion about the diff rather
than an expectation: every one of the 87 currently measured cells keeps its mark
*and its value* — in particular `pp_to_jj` · `samples` stays ⚠️ with its
`ICOLUP` χ² unchanged (chain A owns it), `gg_to_gg` · `diagrams` stays ⚠️ at
4/6, the eight `mg-internal-pdf` ⛔ cells stay ⛔ (chain §G owns them), the four
⏳ stay ⏳, and no σ or `samples` number changes anywhere. Nothing in this chain
touches an integrand: the only production change is E.1, and it is the identity
on every set any banked run reads.

### (d) Risks, and what this provably cannot break

**What this chain provably cannot break: any banked cross section or event
sample.** The single production-code change is the `ForcePositive` clamp, and
its level comes from the set's `.info`. Every `pdlabel = lhapdf` banked run
carries `lhaid = 247000` → `NNPDF23_lo_as_0130_qed`, whose `.info` has no
`ForcePositive` key, so the level is `0` and `force_positive_clamp(0, v) == v`
for all `v` including NaN — the change is *literally* the identity function on
every path a gated row takes. The only set in the tree with a nonzero level is
`NNPDF31_lo_as_0130`, which is a shape fixture for the interpolation gates and
is named by no run card. Every other item in the chain edits tests, manifests,
a Python extractor, a pixi task and the report renderer; none of them is
compiled into `vibegraph-cli`. If any σ, `samples` or `amplitudes` number moves,
that is a defect in this chain, not statistics.

Second provable claim, narrower: **no tolerance moves.** The constants this
chain touches are enumerated and each keeps its value — `FORCE_POSITIVE_FLOOR`
(`1e-8`), `EXTRAP_REL_TOL` (`1e-11`), `EXTRAP_CONDITIONED_TOL` (`1e-14`),
`REL_TOL` (`1e-12`), `ALPHA_S_REL_TOL` (`1e-14`), `AGREEMENT` (`1e-12`),
`P_FLOOR` (`1e-4`), `TIE_BREAK_MISSES`, every `rel_tol` in `validate_sigma`. Two
doc comments around them change because they became false; a review that greps
this chain's diff for numeric literals in `const` positions should find only
additions (the `1e-10` floor), never an edit.

Risks, each with what it looks like and what to do:

- **The extrapolation gates start failing on 205 probes.** The symptom of
  routing a `xf_raw` comparison through a clamped member. The member table in
  E.1 is the fix; the failure is loud and immediate.
- **`an_in_grid_value_lhapdf_floors_is_floored_here_too` fails.** Then some
  in-range point straddles the floor between the two libraries. The measured
  margin says it should not (10.4 % on the nearest point), so a failure is a
  finding: record the point, its two readings and its margin, and report it —
  do not screen it and do not touch `FORCE_POSITIVE_FLOOR`.
- **The samples row is slow or short.** A `2 → 4` with 35 channels at three
  seeds × 20 000 accepted events is the most expensive thing in the chain.
  Background it; if seeds come up short, raise the budget per E.4(a) and record
  the efficiency rather than lowering the event count.
- **The samples row fails a column.** The pre-registered rule sends it to
  `info` with the measurement, and the disagreement is reported. It is a new
  cell, so an `info` landing is still a census improvement (`uncovered` → ⚠️
  measured) and E5 holds.
- **`diagrams.json` regeneration is not surgical.** Covered by the selector
  change, and caught by reading the diff. If the diff shows a fourth key, stop:
  the manifest and the extractor disagree, and the collator will reject the
  extra cells anyway.
- **`#[ignore]` loses a measurement that this machine was really making.** True,
  and it is the coverage trade note 28 §Z already framed: the gate was green
  without comparing anything everywhere else. Mitigated by running it once in
  its new layer (command 7) and recording the summary.
- **Moving `set_alpha_s_info` perturbs `validate_alphas`.** Mechanical, and
  command 5 runs both binaries.

**What each new test provably cannot detect** is in the table in (b); the two
worth restating because they are the chain's real blind spots: the in-range
clamp gate cannot see the clamp's *input* (the oracle carries no unclamped
in-range value, so a point both libraries wrongly floor is invisible to it), and
`the_clamp_level_is_the_one_lhapdf_resolved` cannot see a level that is wrong
identically on both sides — it pins this crate's reading against LHAPDF's
resolution, not against MadGraph's own PDF call. Closing either would need a new
oracle block from `gen_oracle.cpp`, which is oracle-layer work and out of this
chain.
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

## Chain A design amendment 2 (2026-08-03)

A scoping clause on amendment 1's `π` refusal, which fired on a case that has no
correspondence question. The scoping the implementation session proposes is
adopted; the reason below is the load-bearing part, because it is what keeps
this from being the tie-break B.5 forbids.

### C.1 The two clauses

**Clause 1 — B.2's uniqueness requirement is scoped.** When the member and the
group representative are the **same compiled subprocess**, `π` is the identity
and the fingerprint is not consulted. The condition is the member's index
equalling the group head's — an identity of objects, not of process strings, leg
reps, or fingerprints, and it applies to the representative-as-member of *every*
group, not only to one-member groups. For every **distinct** pair the refusal of
B.2 stands unconditionally: no tie-break, no heuristic, no numeric fallback.

**Clause 2 — T12's first assertion is scoped the same way.** Fingerprints must
be pairwise distinct only within a basis that has to be paired against a
*different* subprocess's. `g g > g g`'s three reversal-degenerate pairs
`(0,5) (1,3) (2,4)` are the known block and are exempt; the block is already
handled by T12's JAMP2 degeneracy rule, which asserts that `π` maps a degenerate
block onto a block of equal JAMP2 multiset rather than pinning members of it
individually.

### C.2 Why this is not a tie-break

A tie-break would be choosing one of two admissible answers. Here there is only
one, and there is no question: **the table indexed is the table drawn from.** The
fingerprint matcher exists to relate two *different* bases; applied to a basis
against itself it is being asked which element of a set corresponds to itself,
and the identity is the answer by construction, not by preference. The
degenerate flows do carry different connectivity, so an intra-block *choice*
between two distinct subprocesses would be observable in the emitted `ICOLUP` and
must stay forbidden — which is exactly what clause 1 preserves by scoping rather
than weakening.

The degeneracy itself is physical and not a defect of the fingerprint: a trace
and its reverse are related by the gluon amplitudes' reflection identity, so they
carry the same contributions and the same `JAMP2`, and no coefficient data
separates them — measured, with signs and `i^imag` retained. That is why the
remedy B.5 offered does not apply here, and it is the one statement of amendment
1 that this section corrects (see C.3).

The refusal remains live where it matters: a group whose representative's basis
is degenerate *and* which has a distinct member would still be refused, because
that ambiguity is real. `g g > g g` cannot become such a group — it is a single
flavour assignment.

### C.3 What else in amendment 1 changes

Nothing, with one correction. **B.5's principal-risk paragraph** says that if the
fingerprint ties, "the answer is a richer fingerprint, not a fallback". That
remains true for a distinct pair, but it is false for the self-paired reversal
block: retaining the sign and the `i^imag` phase leaves the pairs identical, so
no enrichment separates them and the right answer is the scoping above. The
sentence should be read as applying to distinct pairs only.

Everything else stands as written: B.2's per-member tables and their reordering,
B.2/3's `ICOLAMP` argument (`π = id` makes the mask identity trivially true for a
self-paired member, and T10's anti-vacuity is carried by the 16 non-identity `π`
the multi-member groups supply), B.3's T9–T11 with their anti-vacuity conditions,
B.4's expected cell movement, and B.5's remaining risks. The measurements the
implementation session took under the provisional scoping — T9's 238 tables over
classes 39/14/12, T10's 112 rows with 95 restricting, T11's 52 mirrored members,
`check_legs` 238/238, and the dijet `ICOLUP` χ² at `p` 1.05e-1 / 2.63e-1 /
1.40e-1 against `p 0` at 2494/25 — are consistent with every prediction B.1 and
B.5 made, and A.4's second-defect rule is not triggered.

## Close-out

(To be written at sprint close: per-chain outcomes, census before/after,
protocol observations on the design–implement–review structure.)
