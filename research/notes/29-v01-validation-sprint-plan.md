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

### Addenda and manager ruling (2026-08-03, post-review)

Appended after chain D's review returned FIX (documentation only; no re-runs, no
physics rework, no tolerance or gate moved). Everything above this heading is
frozen as recorded — these entries supersede by pointing, never by editing a
measurement in place.

**A1 — §D.M9 item 5 is superseded.** It reads "the `m(μμ)` secondary axis of §D.2
was not measured", which was true when written and was falsified by the later
`### m(mumu) secondary axis` section below §D.M10. The axis *was* measured, and it
carried the sharpest result in the chain. Item 5's remaining clauses — the shared
matrix element, `W1`'s lower edge being the `pta` cut, `η(γ)` integrated over
inside each window — stand unchanged. The original line is left as written.

**A2 — error-propagation conventions, named.** Two are in use, both legitimate,
and the same quantity therefore appears with slightly different digits:

* the *ratio form*, `rel = a/b − 1` with `err² = (σ_a/b)² + (a·σ_b/b²)²` — what
  the committed probe's `rel_with_err` computes, and the source of every figure
  in §D.M4, §D.M5 and §D.M7. `C_MG = +3.07σ`; on the `m(μμ)` axis `+14.98σ` and
  `+16.66σ`.
* *plain quadrature* on the absolute difference, `Δ = a − b` with
  `σ = √(σ_a² + σ_b²)` — used in the `### m(mumu) secondary axis` summary table,
  which compares absolute cross sections rather than ratios. `C_MG = +3.08σ`,
  `+15.05σ`, `+16.70σ`.

So `C_MG` on the `pt(γ)` axis is `+3.07` in §D.M4 (ratio form) and `+3.08` in the
`m(μμ)` table (quadrature); they are the same measurement to rounding. No
conclusion anywhere depends on the choice — the pairs differ by under 0.5% of
their own value, and no threshold sits between them.

**A3 — the basis of the `~20σ` sample-versus-windowed claim in §D.M7.** The quoted
`+19.9σ` for `[10, 20)` divides the `+0.663` pp gap by the **5-seed standard error
of the mean of the sample fractions alone** (`0.033` pp). Other defensible bases:

| error used | σ |
|---|---|
| 5-seed SEM of the sample fractions, `0.0333` pp (as quoted) | 19.9 |
| binomial on the pooled 500 000 events, `0.0399` pp | 16.6 |
| 5-seed SEM ⊕ `MG-part`'s own share error (`0.0134` pp) | 18.5 |
| binomial ⊕ `MG-part`'s own share error | 15.8 |

The conclusion is unaffected at any reading — the smallest is 15.8σ — but "~20σ"
is the most favourable of them, so the figure should be read as "≥15σ, 19.9σ on
the seed-spread basis" wherever it appears above and in `validation/manifest.toml`.

**A4 — provenance of the committed reference.**
`validation/madgraph/pta_window_reference.json` was verified row by row against the
raw MadGraph run directories that produced it: all **58/58** rows reproduce their
`SubProcesses/results.dat` first line (σ and quoted error) to machine precision, 0
missing, 0 mismatched. Those directories live under `validation/madgraph/output/`,
which is **gitignored and may be pruned at any time** — so the JSON is the durable
record and the only one that survives a clean of the work area. Anything a later
reader needs must be read from the JSON, not from `output/`.

**A5 — MANAGER RULING (sprint manager, 2026-08-03).** The chain lands on **D1**,
read at the side level as the clauses were frozen: `C_MG` fails at `+3.07σ`,
`C_VG` holds at `−1.53σ`, and this side is seed-consistent (χ²/dof `0.69`) and
budget-stable (`+0.177%` against a `0.232%` bound). The strict per-window reading —
`VG-part(W1)` budget shift `+2.422%` against a `1.530%` bound, and `VG-cut(W3)`,
`VG-cut(W5)` seed clouds at χ²/dof `2.06` and `2.41` against a ≤ 2 criterion —
would land **D10**; it is recorded in §D.M5, §D.M3 and §D.M8 and is not smoothed.
The frozen clause text is side-level, and D1 and D10 differ here only in label,
not in action: no tolerance and no gate moves under either reading.

**A6 — a stale figure found while applying these fixes, and corrected.** The
`P_FLOOR` doc comment in `vibegraph-lib/tests/validate_samples.rs` quoted the
`ee_to_mumua` `samples` minimum as `2.74e-4` ("`2.7x` above the floor") and its
`integrals` pull as `+3.1`. The banked gate run of 2026-08-03 measures
`min KS p 1.292e-4` over the three generation seeds (per-seed `3.383e-4`,
`1.172e-2`, `1.292e-4`) and `pull +2.79`. The row therefore sits **1.3× above the
`1e-4` floor, not 2.7×** — materially closer than the documented figure, and worth
the attention of whoever next touches the `samples` gate. The doc comment and the
`samples` note in `validation/manifest.toml` now carry the measured values.
## Chain C2 design (2026-08-02)

### C2.0 The §6 trigger: measured, and it does **not** fire

The trigger asks whether a physics-relevant parsed-but-unread field is already
set away from its default by a gated run — a latent wrong result rather than a
future hard error. It was checked mechanically, not by reading: every one of the
209 names in `PARAM_DEFAULTS` was parsed out of the Rust table with its default,
and every run card in the tree was re-read under *this crate's own* parse rules
(`parse_fortran_bool`, the `d`/`D` exponent, `strip_quotes`, and the
`{}`/`[]` → empty normalisation of `Kind::Opaque`) and compared field by field.
Corpus: 36 `validation/madgraph/output/*/Cards/run_card.dat`, 37
`.../run_card_default.dat`, `validation/madgraph/dy13_{default,mmll}_run_card.dat`,
`vibegraph-lib/tests/data/run_card_parser_fixture.dat`.

Across that whole corpus exactly 23 names are ever set away from their table
default. Twenty are **consumed** (`ebeam1/2`, `lpp1/2`, `lhaid`, `pdlabel`,
`nevents`, `maxjetflavor`, `scalefact`, `mmll`, `mmllmax`, `ptb`, `etab`,
`fixed_ren_scale`, `fixed_fac_scale`, `fixed_fac_scale1/2`). The remaining
five are unread, and each is dispositioned below:

| field | value seen | cards | why it is not the trigger |
|---|---|---|---|
| `SDE_strategy` | `2` | 9 | MadEvent's own multi-channel weight rule (`banner.py:4458`, "full single diagram enhanced" vs "product of the denominator"). It selects how *MadGraph* distributes its own sampling, and this crate integrates with its own multichannel; σ is invariant under it up to Monte-Carlo error. The one place it is not — the 3.5.7 narrow-pole bias — is already owned in note 27 §B1, which is why `var_sde1` exists as a banked variant. Nearest call in the audit; recorded rather than waved past. |
| `use_syst` | `False` | 28 | Enables MadGraph's post-hoc systematics reweighting, which writes only the `<mgrwt>`/`<rwgt>` block. Note 22 §5 already puts it out of scope. `validate_scales.rs` reads that block where it exists, so its presence is an oracle *input*, never an output. |
| `mxx_only_part_antipart` | `{'default': False}` | 35 | This is MadGraph's *own* default value; the disagreement is on our side — see C2.5, which is a finding but not a physics one, because the field only qualifies `mxx_min_pdg`, which `UNIMPLEMENTED_CUTS` already hard-errors on. |
| `pdlabel1`, `pdlabel2` | `none` | 14 | All 14 are `lpp1 = lpp2 = 0` fixed-energy runs. `setrun.f` overwrites `pdlabel` with `none` when no beam carries a PDF, and `RunningAlphaS::from_run_card` short-circuits on `lpp1 == 0 && lpp2 == 0` (`coupling/alphas.rs:206`) *before* `pdf_label_alpha_s` is reached, so no PDF label of any kind is read on those runs. Physics-relevant in general (C2.3 makes them a hard error), inert on every card that sets them. |

**Conclusion: no gated run rests on an unread physics-relevant field.** The
audit's output is prophylactic, not corrective. That is a negative result and it
is stated as a measurement, not as "nothing was found".

### C2.1 Audit method (what was actually run, so a reviewer can redo it)

Literal-name search alone **under-reports by 71 fields** and a design that
trusted it would have mis-classified most of the cut block. `Cuts::compile`
builds parameter names at runtime — `rc.float(&format!("pt{c}"))`,
`format!("dr{tag}")`, `format!("mm{tag}")` — so `ptj`, `etal`, `drjl`, `mmbb`
and 67 others have a real consumer and **zero** literal occurrences. The method
that survives is three passes, and the implementation session must redo all
three rather than trusting this table:

1. **Literal sweep.** `grep -rn '"<name>"' --include='*.rs'` over
   `vibegraph-lib/src`, `vibegraph-cli/src`, `validation-report/src`, excluding
   `runcard.rs` itself. 72 hits, 137 misses.
2. **Typed-field sweep.** The 14 `RunCard` struct fields are read as
   `rc.<field>`, not by name; `grep -rn '\.<field>\b'` (LSP references where
   available) resolves them. This found that `RunCard::iseed` has **no consumer
   at all** — MadEvent's RNG seed, where this crate's seed comes from the CLI.
3. **Constructed-name sweep.** Every `format!` argument passed to
   `RunCard::float`/`int`/`get` is expanded over its own generator set:
   `letter_char` gives `{j,b,a,l}` and `pair_tag` gives
   `{jj,bb,ll,aa,bj,jl,aj,bl,ab,al}`. This is the pass a pure LSP query cannot
   do, and it is the reason the audit is judgment-heavy rather than mechanical.

Where a field has no consumer, the classification's evidence is not "no consumer
found" but a *positive* argument for inertness — either an existing vibegraph
guard that makes it unreachable, or a reading of the MadGraph source showing it
cannot enter σ or the event record. Bare absence is never accepted as evidence.

### C2.2 The classification, asserted rather than documentary

**New file `vibegraph-lib/src/runcard/classes.rs`** — or, if keeping `runcard.rs`
a single module is preferred, a `mod classes` block inside it; the implementer
picks, the contract is the contents:

```rust
/// Where a recognized run-card parameter goes.
pub enum FieldClass {
    /// Read by this crate. The string names the consumer.
    Consumed(&'static str),
    /// Not read, and unable to reach σ, the event record or the cuts. The
    /// string is the argument for that, never "no consumer found".
    IgnoredBenign(&'static str),
    /// Not read, and able to change what this generator produces. Rejected
    /// when a card moves it off the MadGraph default.
    IgnoredPhysics { why: &'static str, when: Applicability },
}

/// When an `IgnoredPhysics` field is capable of biting at all.
pub enum Applicability {
    Always,
    /// Only when both beams carry a PDF (`lpp1 == lpp2 == 1`).
    ProtonBeams,
}

pub static FIELD_CLASSES: &[(&str, FieldClass)] = &[ /* 209 rows */ ];
```

`Applicability` is not decoration: `pdlabel1`/`pdlabel2` are set away from
default by 14 banked cards, so a flat "must equal default" rule would reject
them. It has exactly two variants and only those two fields use the second one.

**Enforcement** — in `RunCard::from_values`, *after* the existing
`UnsupportedLpp` check (which is what makes `Applicability::ProtonBeams`
decidable), a single loop over `FIELD_CLASSES` returning a new error:

```rust
#[error("run card sets '{name}' to {value} (MadGraph default {default}): {why}")]
UnsupportedField { name: String, value: String, default: String, why: &'static str },
```

Reuse `cuts.rs::describe` for the value rendering (move it to a shared helper, or
duplicate the four-line match — the implementer picks; do not make `cuts.rs`
depend on the new module for it). The message must describe the boundary in its
own terms and name no plan item, per AGENTS.md's comment rules applied to error
text.

**Deliberate non-changes**, both load-bearing:

- **No new field on `RunCard`.** `RunCard` is `Serialize`/`Deserialize` and
  travels inside `IntegrateArtifact`; adding a `user_set` set would be an
  artifact-format change for a parse-time concern. The cost is a residual blind
  spot recorded in C2.6.
- **Nothing is derived or rewritten.** The enforcement only *rejects*. MadGraph
  resolves `pdlabel` from `pdlabel1`/`pdlabel2` (`banner.py:4055-4086`); mirroring
  that would silently change `RunCard::pdlabel` on the 14 fixed-energy cards
  (`nn23lo1` → `none`) and put a semantic change inside a chain whose acceptance
  is "every gated σ row unmoved". Refuse now; derive in a later chain if a card
  ever needs it.

### C2.3 The audit table — all 209 fields

Counts: **130 Consumed, 58 IgnoredBenign, 21 IgnoredPhysics.** Rows are grouped
only where the evidence string is literally identical; every one of the 209
names appears exactly once below.

**Consumed (130).**

| fields | consumer |
|---|---|
| `nevents` | `RunCard::nevents` → CLI event budget |
| `lpp1` `lpp2` | `RunCard::beam_mode`, `RunCardError::UnsupportedLpp`, `scales.rs` `beam_has_pdf` |
| `ebeam1` `ebeam2` | `RunCard::ebeam1/2` → `proton.rs` / `hadronic.rs` beam energies |
| `pdlabel` | `coupling/alphas.rs::pdf_label_alpha_s` (via `from_run_card`) |
| `lhaid` | `coupling/alphas.rs::from_run_card`; PDF set id |
| `fixed_ren_scale` `fixed_fac_scale` `fixed_fac_scale1` `fixed_fac_scale2` `scale` `dsqrt_q2fact1` `dsqrt_q2fact2` `dynamical_scale_choice` `scalefact` `ickkw` `pdfwgt` `bwcutoff` `xmtcentral` `d` | `coupling/scales.rs::ScaleChoice::from_run_card` (`d` → `ClusterSettings::d_parameter`) |
| `maxjetflavor` | `cuts.rs::Cuts::compile` → `classify()` |
| `dsqrt_shat` `dsqrt_shatmax` `mmll` `mmnl` `mmnlmax` `ptllmin` `ptllmax` `pta` | `cuts.rs::Cuts::compile` / `shat_min_hint` / `pair_ptll`, literal names |
| `ptj` `ptb` `ptl` `ptjmax` `ptbmax` `ptamax` `ptlmax` `ej` `eb` `ea` `el` `ejmax` `ebmax` `eamax` `elmax` `etaj` `etab` `etaa` `etal` `etajmin` `etabmin` `etaamin` `etalmin` | `cuts.rs::Cuts::compile` single-leg block; name built by `format!("pt{c}")`, `format!("e{c}max")`, `format!("eta{c}min")`, … over `letter_char` ∈ {j,b,a,l} |
| `drjj` `drbb` `drll` `draa` `drbj` `drjl` `draj` `drbl` `drab` `dral` `drjjmax` `drbbmax` `drllmax` `draamax` `drbjmax` `drjlmax` `drajmax` `drblmax` `drabmax` `dralmax` | `cuts.rs::pair_dr`, name built by `format!("dr{tag}")` over `pair_tag` |
| `mmjj` `mmbb` `mmaa` `mmjjmax` `mmbbmax` `mmaamax` `mmllmax` | `cuts.rs::pair_mass`, name built by `format!("mm{tag}")` |
| `misset` `missetmax` `ptheavy` `ptonium` `etaonium` `xptj` `xptb` `xpta` `xptl` `ptj1min` `ptj1max` `ptj2min` `ptj2max` `ptj3min` `ptj3max` `ptj4min` `ptj4max` `cutuse` `ptl1min` `ptl1max` `ptl2min` `ptl2max` `ptl3min` `ptl3max` `ptl4min` `ptl4max` `htjmin` `htjmax` `ihtmin` `ihtmax` `ht2min` `ht3min` `ht4min` `ht2max` `ht3max` `ht4max` `ptgmin` `xetamin` `deltaeta` `ktdurham` `dparameter` `ptlund` `xqcut` `pt_min_pdg` `pt_max_pdg` `E_min_pdg` `E_max_pdg` `eta_min_pdg` `eta_max_pdg` `mxx_min_pdg` | `cuts.rs::detect_unimplemented` over `UNIMPLEMENTED_CUTS` — parse-and-detect: an active value is already a hard error (`CutError::UnimplementedCutActive`). `ptgmin` and `xqcut` additionally reach `Cuts::compile` and `ScaleChoice::from_run_card`. |

**IgnoredBenign (58).**

| fields | why it cannot bite |
|---|---|
| `run_tag` `keep_log` `gridpack` `python_seed` `iseed` `gseed` `bypass_check` `issgridfile` `global_flag` `aloha_flag` `matrix_flag` | MadEvent job and codegen bookkeeping: names output files, seeds MadGraph-side RNGs, or passes compiler flags. None reaches a momentum, a weight or a written record. `iseed` is a typed `RunCard` field with no consumer — this crate's seed comes from the CLI. |
| `gridrun` `mc_grouped_subproc` `job_strategy` `hard_survey` `second_refine_treshold` `survey_splitting` `survey_nchannel_per_job` `refine_evt_by_job` `SDE_strategy` `vector_size` `nb_warp` `vecsize_memmax` `hel_recycling` `hel_filtering` `hel_splitamp` `hel_zeroamp` | Directives for MadEvent's own integrator and helicity codegen. This crate integrates with its own sampler and sums helicities explicitly, so the reference value is invariant under them up to Monte-Carlo error. `SDE_strategy`'s one exception (the 3.5.7 narrow-pole bias) is owned in note 27 §B1 with `var_sde1` as its banked control. |
| `use_syst` `systematics_program` `systematics_arguments` `sys_scalefact` `sys_alpsfact` `sys_matchscale` `sys_pdf` `sys_scalecorrelation` | Post-hoc systematics reweighting: touches only the `<mgrwt>`/`<rwgt>` block, never σ (note 22 §5). |
| `ievo_eva` `evaorder` `eva_xcut` | EVA lepton-PDF parameters, reachable only at `lpp = ±3, ±4`; `RunCardError::UnsupportedLpp` admits only (0,0) and (1,1). |
| `fixed_extra_scale` `mue_ref_fixed` `mue_over_ref` | The Ellis–Sexton / "extra scale" family. `grep -rn` over `Template/LO/Source/*.f` and `Template/LO/SubProcesses/*.f` finds **zero** occurrences — they are NLO-only and inert at LO. |
| `highestmult` `ktscheme` `alpsfact` `chcluster` `asrwgtflavor` `clusinfo` `auto_ptj_mjj` `pdgs_for_merging_cut` | MLM matching. `ScaleChoice::from_run_card` refuses `ickkw != 0 \|\| xqcut > 0` (`ScaleError::UnsupportedMatching`), and `ktdurham`/`ptlund`/`dparameter` are in `UNIMPLEMENTED_CUTS`, so no matching path is reachable. |
| `r0gamma` `xn` `epsgamma` `isoem` | Frixione photon isolation, read by `cuts.f` only inside its `ptgmin` block; `ptgmin` is in `UNIMPLEMENTED_CUTS`. |
| `mxx_only_part_antipart` | Qualifies the `mxx_min_pdg` cut only, and `mxx_min_pdg` is in `UNIMPLEMENTED_CUTS`. (See C2.5 — this field's stored default is also wrong, which is *why* it must not be enforced.) |
| `bias_parameters` | The bias module's payload. `bias_module` is `IgnoredPhysics`, so no bias is ever active to read it. |
| `cut_decays` | Sets `do_cuts` for legs produced by a decay chain; decay-chain process syntax is a hard error (chain C1), so no such leg exists. |
| `me_frame` `frame_id` | Select the frame for a matrix element that is not Lorentz invariant, or for a polarised sum. Every amplitude here is Lorentz invariant and `polbeam1/2` are refused, so the frame choice cannot change a value. Also the safe classification: `me_frame`'s MadGraph default is `[1, 2]` while the table stores an empty `Opaque` (C2.5), so enforcing it would misfire. |

**IgnoredPhysics (21) — each becomes a hard error.**

| field | applicability | why it is physics |
|---|---|---|
| `polbeam1` `polbeam2` | Always | Beam polarisation: polarised matrix-element sums and their `SPINUP` consequences are not implemented. **Closed by chain C1 — see C2.7 for the merge rule.** |
| `pdlabel1` `pdlabel2` | ProtonBeams | Per-beam PDF set. Selects the parton densities and, through `pdfwrap.f`, `αs(M_Z)`; only the single `pdlabel` is read. MadGraph itself raises `InvalidRunCard` when `lpp1 = lpp2 = 1` and the two disagree (`banner.py:4087-4089`), so the asymmetric case is a boundary on both sides. |
| `nb_proton1` `nb_proton2` `nb_neutron1` `nb_neutron2` | Always | Ion beam composition: `setrun.f:165-178` builds `IDBMUP` from it, so it changes the beam particle in the event record. |
| `mass_ion1` `mass_ion2` | Always | Ion beam mass: `genps.f:668-669` uses it as the beam mass, so it changes the phase-space kinematics. |
| `small_width_treatment` | Always | Floors every width at `VALUE × mass`. `coupling/cluster/kt.rs:475` (`line.width.max(line.mass * settings.small_width_treatment)`) is fed a **hardcoded** `1e-6` from `scales.rs:243`. The hard error is what makes that hardcode true by construction. Reading the card instead was considered and rejected: MadGraph applies the floor at generation time to the propagators as well, so reading it would track the clustering and not the matrix element — a half-implementation with no gate to catch the missing half. |
| `tmin_for_channel` | Always | "Limit the non-singular reach of --some-- channel of integration related to T-channel diagram" (`banner.py:4447`). No argument shows σ invariant under truncating one channel's reach, so it is not benign by the rule of C2.1. |
| `nhel` | Always | Monte-Carlo over helicities instead of the explicit sum: changes the estimator and the per-event weight. |
| `limhel` | Always | Threshold below which MadGraph drops a helicity configuration. Raising it drops contributions this crate keeps. |
| `event_norm` | Always | Normalisation of `XWGTUP` (`average`/`sum`/`unity`) — a factor of the event count in the record. `lhef/record.rs`'s `WeightStrategy` doc already records that nothing else in an LHE file distinguishes them. |
| `time_of_flight` | Always | Writes a nonzero `VTIMUP` for long-lived particles; this crate always writes `0`. |
| `boost_event` | Always | Boosts the whole event before it is written. |
| `lhe_version` | Always | Les Houches format version; `lhef/write.rs` emits `3.0` unconditionally. |
| `bias_module` | Always | A bias module multiplies every event weight. |
| `custom_fcts` | Always | User hook files that overwrite dummy functions, cuts included (`banner.py:4292`). Its MadGraph default `[]` and the table's empty `Opaque` are the same value under `parse_value`'s `{}`/`[]` normalisation, so it is safe to enforce. |
| `fixed_couplings` | Always | MadGraph itself aborts on `False` — `reweight.f`: `'form factor with fixed_couplings not supported anymore'`, `stop 5`. Mirroring its refusal is strictly the honest behaviour. |

### C2.4 The μF ≥ 2 GeV veto

**Reference semantics, read at the pin.** The brief and note 22 §4 both cite
`reweight.f:1185`; in the pinned `mg5amcnlo` checkout the veto is at
**`reweight.f:1205-1220`** (identical in each generated
`validation/madgraph/output/*/SubProcesses/reweight.f`). Line numbers have
drifted; the code has not. Verbatim condition, with Fortran's `.and.`-binds-tighter
precedence made explicit:

```
(lpp(1)/=0 .and. (q2fact(1) < 4d0 .and. .not.fixed_fac_scale1)) .or.
(lpp(2)/=0 .and. (q2fact(2) < 4d0 .and. .not.fixed_fac_scale2))
   →  warn (first 10 only); setclscales = .false.; clustered = .false.; return
```

Three properties that the design must reproduce and that no summary of it
carries: it is **per beam**; it applies only to a beam that both carries a PDF
*and* has a dynamical factorisation scale; and the comparison is **strict** on
the *square* (`q2fact < 4`, so exactly 2 GeV survives).

**Veto, not error — confirmed at the call site.** `reweight.f:1907-1913`:
`if(.not.setclscales(...)) then all_wgt(i) = 0d0`. The point keeps its place in
the sample and contributes nothing. So: return zero weight, do not abort the run.

**Where it lives.**

- `coupling/scales.rs`, new method on `ScaleChoice`:
  ```rust
  pub fn factorisation_scale_vetoed(&self, scales: &EventScales) -> bool {
      (0..2).any(|b| {
          self.beam_has_pdf[b]
              && self.fixed_fac[b].is_none()
              && scales.mu_f[b] * scales.mu_f[b] < 4.0
      })
  }
  ```
  `beam_has_pdf` and `fixed_fac` are already fields of `ScaleChoice`, so this
  needs no new state. Compare the **square** against `4.0` rather than `mu_f`
  against `2.0`: MadGraph stores `q2fact` and we store its root, and squaring
  back is the closer transcription.
- `hadronic.rs`, forwarding method on `EventScaleSource`: `PerEvent` delegates;
  `ScaleSourceKind::Constant` returns `false`. That is provable, not a
  convenience: `Constant` is built only under `choice.is_fully_fixed()`
  (`hadronic.rs:275`), which requires `fixed_fac[0]` and `fixed_fac[1]` both
  `Some`, exactly the case MadGraph's `.not.fixed_fac_scale` guard excludes. The
  other `Constant` producer, `EventScaleSource::constant(mu)`
  (`hadronic.rs:248`, used at `proton.rs:941`), belongs to a caller that supplied
  a μF directly and is modelling no MadGraph run at all.
- `proton.rs`, one early return in `ProtonIntegrand::shape`, immediately after
  `let scales = self.event_scales(channel);` and **before** `apply_scale` and the
  luminosity loop:
  ```rust
  if self.scales.factorisation_scale_vetoed(&scales) {
      return 0.0;
  }
  ```
  The ordering is load-bearing twice over: it keeps `apply_scale` from moving the
  coupling for a point that carries no weight, and it keeps the PDF from being
  queried below roughly its own grid `Q_min` — which is most of why MadGraph's
  floor is at 2 GeV in the first place.

**Why `ProtonIntegrand` is the only site.** MadGraph's guard is `lpp(i) /= 0`.
The only PDF convolution in this crate is `proton.rs` (`FlavorGroup::luminosity`
and `symmetry_weighted_luminosity` are its only callers; `hadronic.rs`'s
`FixedBeamIntegrand` builds its beams as `[beam_e, 0, 0, ±beam_e]` with no
densities). So the veto is unreachable on every fixed-energy run *by construction*,
not by a flag. `event_in_channel` needs no separate wiring: it calls `shape`
first and returns `None` on a zero, so generation inherits the veto — and
inherits it in a way that keeps a sample's scales consistent with its integral.

**Why no banked reference exercises it — measured, not asserted.** The veto is
reachable on exactly five banked runs (`lpp = (1,1)`, at least one dynamical
factorisation scale): `pp_to_jj`, `pp_to_llj`, `pp_to_bb`, `pp_to_bb_qcd2`,
`pp_to_ll_qcd0`. Both `dy13` cards set `fixed_ren_scale = fixed_fac_scale = True`,
so `setclscales` early-returns before the check ever runs; the `gu_to_epemu`,
`gux_to_epemux`, `ddx_to_epemg`, `uux_to_epemg` and all `ee_*` runs are `lpp = 0`.
Minimum `SCALUP` over each reachable run's 10 000 banked events
(`SCALUP = sqrt(max(q2fact(1), q2fact(2)))`, `unwgt.f:686`), cross-checked
against the per-beam `<pdfrwt beam="i">` field where `use_syst` wrote it:

| run | min μF (GeV) | headroom over 2 GeV |
|---|---|---|
| `pp_to_bb` | **4.7003** | ×2.35 |
| `pp_to_bb_qcd2` | **4.7003** | ×2.35 |
| `pp_to_jj` | 20.0003 | ×10.0 |
| `pp_to_ll_qcd0` | 20.0393 | ×10.0 |
| `pp_to_llj` | 21.5932 | ×10.8 |

The floors are structural, not lucky: `pp_to_jj` / `pp_to_ll_qcd0` / `pp_to_llj`
sit on their cards' `ptj = 20`, and the `pp_to_bb` pair sits on `m_b = 4.7` —
the clustered 2 → 2 core's transverse mass cannot fall below the heaviest leg it
contains. **The global minimum over every banked hadronic event is 4.70 GeV.** A
banked sample also cannot contain a counter-example by construction: MadGraph
vetoed such points before writing them. So the veto's own test must be a
constructed card, and the banked runs' role is the complementary one — proving
the veto is a no-op on all of them (test T6).

### C2.5 A second finding: the `Opaque` defaults are unverified

`defaults_match_banner_py_dump` (`runcard.rs:778`) is the transcription oracle
against `banner.py`, and its `Def::O` arm is `{}` — Opaque payloads are *not*
compared. Consequence, measured: for four fields the table's empty default is
not MadGraph's.

| field | table default | `banner.py` default |
|---|---|---|
| `mxx_only_part_antipart` | `""` | `{'default': False}` |
| `me_frame` | `""` | `[1, 2]` |
| `pdgs_for_merging_cut` | `""` | `[21, 1, 2, 3, 4, 5, 6]` |
| `systematics_arguments` | `""` | `['--mur=0.5,1,2', '--muf=0.5,1,2', '--pdf=errorset']` |

`mxx_only_part_antipart` is the live one: **35 banked cards write MadGraph's own
default and it reads as an override.** That is the trap this chain is one step
away from — all four are classified `IgnoredBenign` above precisely so the
enforcement never compares them, and the reasons given are independent of the
default. Rather than fix the payloads (a Python-repr normalisation problem worth
its own change), the design **pins the discrepancy**: test T4 asserts exactly
this set of four names, so a MadGraph bump that changes it fails loudly instead
of silently re-arming the trap. The eventual fix belongs in a follow-up, and
the review session should say so in its report rather than in `TODO.md` (this
session does not edit `TODO.md`).

### C2.6 Change list, file by file

1. **`vibegraph-lib/src/runcard.rs`** (or `runcard/classes.rs`, implementer's
   choice): `FieldClass`, `Applicability`, `FIELD_CLASSES` (209 rows, C2.3);
   `RunCardError::UnsupportedField`; the enforcement loop in `from_values` after
   the `UnsupportedLpp` check. No change to `PARAM_DEFAULTS`, to `RunCard`'s
   fields, or to its serialised form.
2. **`vibegraph-lib/src/coupling/scales.rs`**:
   `ScaleChoice::factorisation_scale_vetoed`. No change to `ScaleChoice::scales`
   or `cluster_scales`, so `validate_scales`'s replay path is untouched.
3. **`vibegraph-lib/src/hadronic.rs`**:
   `EventScaleSource::factorisation_scale_vetoed`, delegating.
4. **`vibegraph-lib/src/proton.rs`**: the three-line early return in `shape`.
5. **Tests** (T1–T6, C2.7).
6. **Nothing else.** No file under `helas/`, `diagrams/`, `ufo/`, `phasespace/`,
   `lhef/`, `vegas.rs`, `unweight.rs`, `cuts.rs` or `pdf/` is touched.

### C2.7 Acceptance tests

**T1 `every_run_card_field_is_classified`** — hermetic, in `runcard.rs`'s test
module. Every `PARAM_DEFAULTS` name has exactly one `FIELD_CLASSES` row and vice
versa; no duplicates in either direction; every reason string non-empty.
*Fails on*: a field added to the defaults table without a classification, or a
stale classification row. This is the durable form the brief asks for.
*Provably cannot detect*: a **wrong** classification. A physics-relevant field
parked in `IgnoredBenign` passes T1 and every other test here — the audit's
judgment is the oracle, and nothing mechanical replaces it. This is the chain's
single largest residual risk and it is stated, not managed.

**T2 `ignored_physics_fields_are_refused`** — hermetic. For each
`IgnoredPhysics` row, build a one-line card perturbing that field off its
default (numeric → `default + 1` where the default is not a sentinel, bool →
negation, string → a marker) and assert `RunCard::parse` returns
`UnsupportedField` naming that field. For the two `ProtonBeams` rows the
perturbed card is built at `lpp = (1,1)` and the test *also* asserts the same
perturbation at `lpp = (0,0)` is **accepted**.
*Fails on*: an `IgnoredPhysics` row with no enforcement, or an applicability
guard inverted.
*Provably cannot detect*: a misclassification into `IgnoredBenign` (T1's blind
spot); and, because it perturbs one field at a time, any interaction between two
fields that is only unsafe jointly.

**T3 `banked_run_cards_are_accepted`** — refdata-gated, in `validate_scales.rs`
or a small new `validate_run_cards.rs`. Parse every
`validation/madgraph/output/*/Cards/run_card.dat`, every
`.../run_card_default.dat`, and both `dy13` cards; assert `Ok` for each, naming
the file on failure. Its hermetic sibling covers the committed cards, in
`scales_run_cards.rs`.
*Fails on*: any enforcement that rejects a card a banked reference actually ran
with — the single highest-probability defect in this chain, and the test that
would have caught the `pdlabel1 = none` trap.
*Provably cannot detect*: an enforcement that is too **weak**; it proves only
that nothing legitimate is rejected.

**T4 `opaque_defaults_known_to_differ_from_banner_py`** — hermetic. Assert the
mismatch set of C2.5 is *exactly* `{mxx_only_part_antipart, me_frame,
pdgs_for_merging_cut, systematics_arguments}`, comparing against
`validation/madgraph/runcard_defaults.json`.
*Fails on*: a MadGraph bump that adds or removes a mismatch.
*Provably cannot detect*: whether the four are individually harmless — that rests
on their `IgnoredBenign` reasons, not on this test.

**T5a `factorisation_scale_veto_matches_reweight_f`** — hermetic unit test in
`scales.rs`. On a constructed card (`lpp1 = lpp2 = 1`, `fixed_ren_scale = True`,
dynamical factorisation, `dynamical_scale_choice = 4` so μF = √ŝ is exactly
computable by hand), assert: √ŝ = 1.5 GeV vetoes; √ŝ = 3 GeV does not; the same
card with `fixed_fac_scale = True` never vetoes at any √ŝ; and a per-beam
asymmetric case vetoes on the low beam alone. Plus `lpp = (0,0)` never vetoes.
*Fails on*: a dropped `beam_has_pdf` or `fixed_fac` guard, or an inverted
comparison.
*Provably cannot detect*: the **wiring** — it never runs an integrand; and the
exact boundary, since `mu_f * mu_f` and MadGraph's `q2fact` differ by one
round-trip rounding at `q2fact == 4` exactly. Per AGENTS.md that last-ulp
difference on a measure-zero set is not chased.

**T5b `a_sub_threshold_factorisation_scale_gives_zero`** — the wiring test.
Take an existing small hadronic integrand fixture, set `scalefact = 1e-3` on its
card so that μF < 2 GeV at every reachable point, and assert the integrand's
value is **exactly** `0.0` over a fixed set of points; then the same fixture at
`scalefact = 1.0` returns nonzero.
*Fails on*: a veto that never reaches `ProtonIntegrand::shape`.
*Provably cannot detect*: on its own, "vetoed" from "cut away" or "PDF returned
zero" — which is exactly why the `scalefact = 1.0` control is part of the same
test; and a veto that fires **too often** on a normal card, which is what T6 and
the unmoved σ rows cover.

**T6 `banked_hadronic_runs_clear_the_factorisation_floor`** — refdata-gated, in
`validate_scales.rs`, which already replays both per-beam μF per banked event.
Over the five reachable runs, assert `min over events, min over beams` of the
*replayed* μF exceeds 2 GeV, and print the measured minimum.
*Fails on*: a re-bank that introduces a run reaching the floor — precisely when
the gate must start caring — or a scale bug large enough to push a replayed μF
below a threshold MadGraph demonstrably did not hit (a banked event exists, so
MadGraph did not veto it).
*Provably cannot detect*: anything about runs that are not banked; and, since it
reads our replay rather than MadGraph's `q2fact`, a common-mode scale error that
moves both sides together.

**Interaction with chain C1.** `polbeam1`/`polbeam2` are classified
`IgnoredPhysics` in C2.3 and T2 therefore exercises them, so **in this worktree
C2 refuses them by itself** (C2's worktree is off `main` and does not carry C1).
That is a deliberate duplicate, not an oversight: C2's tests must pass standalone.
At merge (order C1 → E → A → D → **C2** → B) the manager keeps **one**
mechanism. Recommendation: keep C2's generic loop and delete C1's dedicated
polbeam check, retaining C1's refusal test unchanged — the `why` string for
`polbeam1/2` is written to name polarised beams explicitly so a message
assertion in C1's test survives the substitution. This is the chain's one
expected merge conflict and it is mechanical; per AGENTS.md the manager resolves
it, never a subagent.

### C2.8 Gates, and the diff assertion about the report

Run, all with `--skip-deps`:

- `cargo test --workspace` — T1, T2, T4, T5a, and the existing `runcard`/`cuts`/
  `scales` unit tests. The hermetic tier must still be complete on a bare clone.
- `pixi run validate` — the banked census, which covers T3 and T6 and every σ row.
- `pixi run validate-scales`, `validate-hadronic`, `validate-sigma`,
  `validate-generate-proton` individually if the full run is inconvenient to
  bisect; these are the four that touch the modified path.
- The MadGraph/HELAS bit-exact amplitude, colour and diagram gates are **not**
  required: no amplitude, colour, coupling or enumeration code is touched
  (C2.6 item 6). Say so in the report rather than running them silently.

**Report cells expected to move: none.** As an assertion about the diff:
`target/validation-report/report.md`, regenerated after the change, must be
identical to the banked report in every σ value, every uncertainty, every pull,
every tolerance verdict, every samples/KS cell and every census cell — the only
admissible differences are run metadata that already varies between runs
(timestamps, wall times). A single moved cell is a defect in this chain, not
statistics, and the review session should treat it as escalation-worthy.

The pre-registered reasons it cannot move:

1. **No card newly refuses.** Measured in C2.0: across 36 + 37 + 2 + 1 cards,
   no `Applicability::Always` `IgnoredPhysics` field is ever set away from its
   default, and the only `IgnoredPhysics` fields set at all — `pdlabel1`,
   `pdlabel2` — are set only on `lpp = (0,0)` cards, which `ProtonBeams` exempts.
2. **No point is newly vetoed.** The minimum μF over every banked hadronic event
   is 4.70 GeV, 2.35× the floor, and the veto is unreachable on fixed-energy runs
   and under a `Constant` scale source (C2.4).
3. **Nothing on the value path changes.** The veto adds one comparison per point
   and no arithmetic; the enforcement runs once at parse time.

### C2.9 Risks, and what this provably cannot break

**Risks, in descending probability.**

- *An enforcement rejects a card a gate needs.* Highest-probability defect;
  T3 is aimed exactly at it, and C2.0's measurement is the pre-check. The
  `Opaque`-default trap of C2.5 is the specific form it would have taken.
- *A classification is wrong.* Unfalsifiable by construction (T1's blind spot).
  Mitigation is that every `IgnoredBenign` reason is a positive argument citing
  an existing guard or a line of MadGraph source, so a reviewer can check them
  one at a time rather than re-deriving the audit.
- *The veto fires where MadGraph does not.* Would need our replayed μF to
  straddle 2 GeV against MadGraph's own. `validate_scales` already gates both
  per-beam μF per event against MadGraph's printed value at that value's own
  precision, so a disagreement of that size fails an existing gate first.
- *`RunCardError` gains a variant.* A public-API change for any downstream
  `match`; in-tree there are few and the compiler finds them all.

**What this provably cannot break.**

- **No amplitude, colour, coupling, phase-space map, sampler or event writer.**
  C2.6 item 6 lists the untouched directories; the claim is checkable as a
  property of the diff, not of the tests.
- **No fixed-energy (`lpp = 0`) run, in any respect.** The veto lives in
  `ProtonIntegrand`, which is the only PDF-convolving code in the crate; and
  every `IgnoredPhysics` field is either at its default on every fixed-energy
  card or guarded to `ProtonBeams`.
- **No parsed value.** The enforcement only rejects; it never rewrites a field.
  This is why MadGraph's `pdlabel` ← `pdlabel1/2` derivation was deliberately
  *not* mirrored (C2.2), and it is what makes "no σ row moves" a structural
  statement rather than a hope.
- **No artifact format.** `RunCard` gains no field, so `IntegrateArtifact`'s
  serialised form is unchanged and every banked artifact still deserialises.
- **No hermetic-tier completeness.** T1, T2, T4 and T5a need no reference data;
  T3's hermetic sibling reads only committed cards.

**Residual blind spot, stated because nothing here closes it.** Without
MadGraph's `user_set` tracking (declined in C2.2 for artifact-format reasons),
"the card wrote this field" and "the field differs from its default" are the
same predicate. So a card that writes a field *at* its MadGraph default is
indistinguishable from one that omits it. For every `IgnoredPhysics` field that
is the correct behaviour — the default is by definition safe. It bites only in
one constructed case: a card writing **both** `pdlabel` and `pdlabel1/2` where
`pdlabel1 = pdlabel2 = nn23lo1` (the default) and `pdlabel` is something else.
MadGraph's own writer emits one template or the other and never both, so no
MadGraph-written card can reach it; a hand-written one could, and would be
resolved as `pdlabel` where MadGraph would resolve it as `nn23lo1`.

## Chain C2 design amendment (2026-08-03)

Supersedes C2.4's change list items 2–4 and tests T5a/T5b. C2.0–C2.3 and
C2.5–C2.7's T1–T4/T6 stand as landed in `30e88c1`. C2.4's *semantics* reading
of `reweight.f` stands and is what convicts the current code; only its "where
it lives" was wrong.

### A.0 What the control falsified, re-verified here

The implementer's three findings were re-checked against the worktree at
`30e88c1` rather than taken on report:

1. **The veto already exists.** `coupling/cluster/setclscales.rs:468-473`:
   `if settings.beam_has_pdf[beam] && q2fact[beam] < MUF_FLOOR && !settings.fixed_fac[beam]`
   → `Err(ScaleRefusal::FactorisationFloor)`, with `MUF_FLOOR = 4.0` at
   `setclscales.rs:37`. Per beam, strict, on the square, guarded on both PDF
   presence and the fixed flag — the faithful transcription C2.4 specified,
   written before this chain existed. C2.4's "where it lives" section was
   designing a duplicate of code already in the tree; that is the error, and it
   came from reading `scales.rs` and `proton.rs` without reading the clustering
   module underneath them.
2. **Its response is an abort, not a zero.** `ScaleRefusal::FactorisationFloor`
   → `ScaleError::Clustering` (`scales.rs:391`, `.map_err(ScaleError::Clustering)`)
   → `ProtonIntegrand::event_scales` (`proton.rs:1184-1185`)
   `.unwrap_or_else(|e| panic!("per-event scale on a sampled point: {e}"))`.
   C2.4 established zero-weight as the reference semantics
   (`reweight.f:1907-1908`, `all_wgt(i) = 0d0`); the crate panics. The chain's
   real defect is therefore a **response** bug, not a missing feature, and it is
   worse than the silent disagreement C2.4 set out to close: an abort mid-VEGAS
   on a card whose support merely dips below the floor.
3. **C2.4's proposed check was unreachable.** `compile_scale_source`
   (`hadronic.rs:670`) always passes `Some(sets)`, and `EventScaleSource::scales`
   dispatches on `channels.is_some()` — never on `dynamical_scale_choice`. Both
   `EventScaleSource::from_run_card` callers in the crate go through
   `compile_scale_source`, so every non-fully-fixed run reaches `cluster_scales`.
   Confirmed one level deeper than the report: `ScaleChoice::cluster_scales`
   (`scales.rs:332-395`) never inspects `self.choice` at all — it short-circuits
   on `is_fully_fixed()` and otherwise goes straight to `setclscales`.

**Two further sites the brief does not name, found while checking (1)–(3):**

4. **The setup probe propagates the refusal too.** `ProtonIntegrand::probe_scale`
   (`proton.rs:1030-1031`) and `FixedBeamIntegrand::probe_scale`
   (`hadronic.rs:997`) resolve the scale on the first cut-passing draw and `?`
   the result. A `FactorisationFloor` there is not a setup failure — it is one
   ordinary vetoed point — so under the corrected semantics the probe must keep
   drawing rather than abort the run before it starts. Any fix confined to
   `shape` would leave a card whose *first* cut-passing probe point is
   sub-threshold still dying at setup.
5. **The fixed-beam path carries the identical panic.**
   `hadronic.rs:1067-1068` repeats `unwrap_or_else(|e| panic!("per-event scale
   on a sampled point: {e}"))`. It is unreachable for this refusal (A.1), but
   the duplication is why a call-site-only fix would not hold.

### A.1 (a) Routing the `FactorisationFloor` refusal

**Decision: give the distinction a type, one level above the clustering, and let
the compiler force every call site to say what it does with a veto.** Matching
`ScaleError::Clustering(ScaleRefusal::FactorisationFloor)` at each call site was
considered and rejected: there are four such sites today (`shape`'s
`event_scales`, two `probe_scale`s, `FixedBeamIntegrand::apply_scale`), finding
(4) shows that a fix aimed at one of them misses the others, and a fifth site
added later would silently inherit the panic — which is precisely how the
present bug survived.

**New in `hadronic.rs`, beside `EventScaleSource`:**

```rust
/// What resolving one point's scales produced.
pub enum PointScales {
    /// The scales to evaluate this point at.
    Scales(EventScales),
    /// The point carries no weight: a beam carrying a parton density ended
    /// below the factorisation floor, where MadGraph zero-weights the point
    /// and moves on.
    Vetoed,
}

impl EventScaleSource {
    /// [`scales`](Self::scales), with the factorisation-floor refusal separated
    /// from the errors that mean the prescription itself does not apply.
    pub fn point_scales(&self, /* same args as scales */) -> Result<PointScales, ScaleError> {
        match self.scales(..) {
            Ok(s) => Ok(PointScales::Scales(s)),
            Err(ScaleError::Clustering(ScaleRefusal::FactorisationFloor)) => Ok(PointScales::Vetoed),
            Err(other) => Err(other),
        }
    }
}
```

The mapping lives in exactly one function. `ScaleError` gains **no** variant and
`ScaleRefusal` gains none either: `FactorisationFloor` already exists and is
already the right name. Nothing about `ScaleChoice::cluster_scales`'s signature
or behaviour changes — deliberately, because `validate_scales.rs` drives it
directly and wants the refusal *as* a refusal when it replays a banked event
against MadGraph's own record. Keeping that API fixed is what makes "the scales
gate is untouched" a structural statement rather than a hope.

**Call sites, all four:**

- `ProtonIntegrand::event_scales` (`proton.rs:1179-1186`) returns
  `Option<EventScales>`: `PointScales::Vetoed` → `None`, `Scales(s)` → `Some(s)`,
  and a genuine `ScaleError` keeps today's panic with today's message. The panic
  is *right* for the remaining errors — they mean the prescription does not
  apply to this process, which no amount of sampling fixes.
- `ProtonIntegrand::shape` → `let Some(scales) = self.event_scales(channel) else
  { return 0.0; };`, placed exactly where C2.4 put it: after the cuts, before
  `apply_scale` and the luminosity loop. That ordering survives the amendment
  unchanged and for the same two reasons — the coupling is not moved for a point
  with no weight, and the PDF is not queried below roughly its own grid `Q_min`.
  This reproduces `reweight.f:1907-1908`.
- `ProtonIntegrand::event_in_channel` inherits it with no edit: it calls `shape`
  first and returns `None` on a zero, so a generated sample and the integral it
  came from veto the same points.
- Both `probe_scale`s: `Ok(PointScales::Vetoed)` means *keep drawing* — the draw
  passed the cuts but carries no weight, which is exactly the case the probe
  should skip rather than report. See A.2 for what happens when every draw is
  vetoed.
- `FixedBeamIntegrand`'s scale sites take `point_scales` too, and handle
  `Vetoed` with an explicit `unreachable!` carrying the reason, converting
  today's silent assumption into a checked one.

**Why the fixed-beam path stays unreachable, and why that survives this change.**
The floor's applicability is decided by `ScaleSettings::beam_has_pdf`, which
`ScaleChoice::from_run_card` sets from the card as `[card.lpp1 != 0, card.lpp2 != 0]`.
A fixed-energy card gives `[false, false]`, so the guard at `setclscales.rs:469`
short-circuits before the comparison and the refusal is never constructed. That
argument is **upstream of everything this amendment touches** — it depends on the
card, not on which integrand runs or how a refusal is routed — so no routing
change can reach it. It is also not the same as "`FixedBeamIntegrand` is only
built for `lpp = 0`": the guard tracks the card, so it would hold even if that
pairing were ever broken. The `unreachable!` above pins it.

### A.2 (b) Tests replacing T5a and T5b, and the diagnostic decision

T5a and T5b are withdrawn: T5a specified a predicate that already exists and is
already unit-tested inside the clustering module, and T5b's construction panics
rather than returning zero, which is the bug — it was a correct experiment
attached to a wrong premise.

**T5a′ `a_sub_threshold_factorisation_scale_gives_zero_weight`** — the
replacement wiring test, aimed at the existing veto. Build a proton integrand on
a card whose factorisation scale is driven below 2 GeV at every reachable point
(`scalefact` small enough that `scalefact² · pt²` clears the floor nowhere, on
an otherwise ordinary hadronic fixture), and assert `value_in_channel` returns
**exactly `0.0`** on a fixed set of points — not a panic. Control: the identical
fixture at `scalefact = 1.0` returns nonzero on the same points.
*Fails on*: today's code, immediately, with the panic — which is the point; and
on any future routing that turns the veto back into an error.
*Provably cannot detect*: on its own, "vetoed" from "cut away" or "PDF returned
zero" — the `scalefact = 1.0` control is what separates them, and the two halves
must stay in one test for that reason. It also cannot see a veto firing **too
often** on a normal card; T6 and the unmoved σ rows cover that direction.

**T5b′ `a_vetoed_point_is_dropped_from_generation_too`** — `event_in_channel` on
the sub-threshold fixture returns `None` at points where the control fixture
returns `Some`.
*Fails on*: a veto wired into `shape` but bypassed on the generation path, which
would produce a sample whose scales disagree with its own integral.
*Provably cannot detect*: whether the *kept* events carry the right scales; that
is `validate_scales`'s job and it is untouched.

**T5c′ `the_factorisation_floor_is_unreachable_on_fixed_beams`** — a fixed-energy
card at the same small `scalefact` integrates normally and returns nonzero.
*Fails on*: a `beam_has_pdf` guard dropped or inverted in the clustering.
*Provably cannot detect*: nothing about proton runs.

**The diagnostic question: decided — refuse at setup, do not count per event.**

MadGraph warns on the first ten vetoed points and then goes quiet, which is a
console affordance for an interactive run. Two reasons not to transcribe it:

- A per-event counter is per-thread mutable state inside the VEGAS loop. Note 22
  §4 already flags that class of state as the thing that makes `adapt_parallel`
  race silently; paying that for a diagnostic is the wrong trade.
- A partially-vetoed run is **legitimate physics** — MadGraph runs them and their
  σ is correct — so a per-event count has no threshold at which it could act.
  Only the degenerate case is actionable.

The degenerate case already has a home: `probe_scale`, which exists to surface at
setup what would otherwise surface mid-integration, draws 64 points
(`SCALE_PROBE_DRAWS`, `hadronic.rs:92`) and stops at the first that passes the
cuts. Amend it to keep drawing past vetoed points and to distinguish three
outcomes: a scale resolved (`Ok`), no draw passed the cuts (`Ok`, as today —
the cuts, not the scale, are what that says something about), and **at least one
draw passed the cuts and every such draw was vetoed** → refuse, with a message
describing the boundary in its own terms:

> every sampled point's factorisation scale fell below the 2 GeV floor a
> parton density is fitted down to, so the cross section this card asks for is
> zero by construction

That is strictly stronger than MadGraph's warning — it fires before the
integration spends anything — and it costs one boolean in a setup-time loop that
already exists. A run with *partial* sub-threshold support integrates normally
and silently, which is the correct behaviour and matches the reference.
*Provably cannot detect*: a run where 64 draws happen to find support the veto
spares while the bulk of the measure is vetoed. That is a sampling statement
about a 64-point probe, not a claim about σ, and the σ it produces is still
right — merely inefficient.

### A.3 (c) `dynamical_scale_choice` 1–5

**Decision: hard-error, and only where the value is actually consulted.**

Wiring the closed forms is rejected now, on the project's own rule rather than
on effort. `ScaleChoice::closed_form` (`scales.rs:398`) already computes all five
and is exercised **only by unit tests**; all 35 banked cards that mention the
field set `-1` (measured across the same corpus as C2.0), so there is no
reference run against which an honoured choice 1–5 could be pinned. Turning on
an unvalidated scale prescription would produce a plausible, smooth, wrong σ with
nothing to notice it by — the exact failure note 22 §4 names for the
unimplemented `-1` cases and answers the same way. A hard error is what this
project does with a boundary that has no oracle.

**Where.** `ScaleChoice::from_run_card` (`scales.rs:197`), beside the existing
`UnsupportedChoice` and `UnsupportedMatching` refusals, gated on reachability:

```rust
if !fully_fixed && choice != DynamicalChoice::Clustered {
    return Err(ScaleError::UnhonouredScaleChoice { choice: choice_int });
}
```

The `!fully_fixed` gate is not leniency, it is accuracy: `cluster_scales` and
`scales` both short-circuit on `is_fully_fixed()` before the choice is read, so
on a fully-fixed card the value provably cannot change a number. Refusing it
there would be a refusal the code cannot justify. (The gate must be computed from
the same `fixed_ren`/`fixed_fac` values `is_fully_fixed` uses, after they are
resolved — the implementer should reorder within `from_run_card` rather than
recompute.)

**Message**, describing what the code does now and naming no plan item:

> run card selects dynamical_scale_choice = {choice}: the integration path
> evaluates the clustered scale only, and the closed forms for 1–5 are computed
> nowhere a cross section reads them

**Refusal test `an_unhonoured_scale_choice_is_refused`**, in `scales.rs`'s test
module next to the existing `UnsupportedChoice` test: for each of 1, 2, 3, 4, 5
a dynamical card is refused with `UnhonouredScaleChoice`; the same choice on a
fully-fixed card is **accepted** and yields the card's constants; `-1` is
accepted.
*Fails on*: the gate dropped, inverted, or applied to `-1`.
*Provably cannot detect*: whether the closed forms are *correct* — nothing here
claims they are, which is the reason for the refusal.

**`FIELD_CLASSES` consequence — the field does not move.**
`dynamical_scale_choice` stays **`Consumed`** (`ScaleChoice::from_run_card`),
and the row's evidence string needs no edit: it is read, which is not in
dispute. The refusal is on a **value**, not on the field, and that distinction is
the whole reason it belongs in `from_run_card` rather than in the classification
loop. Concretely:

- T2 iterates `IgnoredPhysics` rows and expects `RunCardError::UnsupportedField`.
  `dynamical_scale_choice` is not in that class, so T2 never constructs a card
  that touches it, and the new refusal cannot perturb T2.
- The two refusals also live at different layers and produce different types:
  `RunCardError::UnsupportedField` at parse (`RunCard::from_values`), and
  `ScaleError::UnhonouredScaleChoice` at prescription compile
  (`ScaleChoice::from_run_card`). A card asking for choice 4 still *parses*; it
  fails when a scale prescription is compiled from it. That is correct — the
  parse layer has no business knowing which prescriptions are implemented — and
  it is why this is a separate check with a separate test, not an extension of
  T2's framework.
- The C2.3 table is unchanged by this amendment. No row is reclassified.

### A.4 (d) Gates and expected movement

Everything the implementer landed in `30e88c1` stays; nothing in the audit half
is reopened. Run, all with `--skip-deps`:

- `cargo test --workspace` — T1–T4 as landed, plus T5a′, T5b′, T5c′,
  `an_unhonoured_scale_choice_is_refused`, and the existing `runcard`, `cuts`,
  `scales` and `setclscales` unit tests.
- `pixi run validate` — the banked census, covering T3 and T6.
- `pixi run validate-scales` is the one to read first and to quote in the report:
  it drives `ScaleChoice::cluster_scales` directly, that API is deliberately
  unchanged (A.1), and its 400k per-event comparisons are the tightest oracle
  the modified path has.
- `validate-hadronic`, `validate-sigma`, `validate-generate-proton` — the three
  σ/sample gates on the proton path.
- The MadGraph/HELAS amplitude, colour and diagram gates remain **not required**:
  no amplitude, colour, coupling or enumeration code is touched. State that in
  the report rather than running them silently.

**Report cells expected to move: none**, and the assertion is the same shape as
C2.8's — `target/validation-report/report.md` identical to the banked report in
every σ, uncertainty, pull, tolerance verdict, samples/KS and census cell, with
run metadata the only admissible difference. The pre-registered reasons, updated
for what the amendment actually changes:

1. **No banked point is vetoed.** The global minimum μF over every banked
   hadronic event is 4.70 GeV, 2.35× the floor (C2.4's table, reproduced by the
   implementer over the corrected eight-run set). The veto's behaviour changes
   only for points that reach it, and none does.
2. **No banked card is refused for its scale choice.** All 35 cards that set
   `dynamical_scale_choice` set `-1`; the two `dy13` cards omit it and are fully
   fixed, where the new gate does not apply.
3. **The replay API is untouched.** `ScaleChoice::cluster_scales` keeps its
   signature and its error, so `validate_scales` compares exactly what it
   compared before.

A moved cell is a defect in this amendment, not statistics.

### A.5 Corrections to the C2 design section, folded here

Recorded here rather than edited in place, so the record shows what was designed
and what was measured against it:

- **C2.1**, "under-reports by 71 fields" → **58**. 130 consumed less 72 literal
  hits. The claim it supports — that a literal sweep alone would have
  misclassified most of the cut block — is unaffected.
- **C2.0**, "Twenty are consumed … The remaining five are unread" →
  **17 consumed, 6 unread**. The sixth is `iseed = 33`, set by both `dy13`
  cards; it is `IgnoredBenign` and its inertness is already argued in C2.3's
  first `IgnoredBenign` row (MadEvent's RNG seed; this crate's seed comes from
  the CLI). The §6 trigger conclusion is unchanged.
- **C2.4**, the veto-reachable banked run set is **8, not 5**: add `pp_to_ll`,
  `pp_to_ll_scalefact2` and `pp_to_llj_dyn`, all `lpp = (1,1)` with a dynamical
  μF. Every minimum quoted in C2.4's table reproduces and the global minimum
  stays **4.70 GeV**; T6 as landed asserts over the eight.

None of the three changes a decision. All three are cases of a stated number
being narrower than the measurement behind it, which is the failure mode the
"every green cell is a recorded measurement" rule exists to catch — recorded
accordingly.

### A.6 What this amendment provably cannot break, and its blind spots

- **No amplitude, colour, coupling, phase-space map, sampler, cut or event
  writer.** The diff touches `hadronic.rs`, `proton.rs`, `scales.rs` and tests.
  `setclscales.rs` is touched **not at all** — the veto itself, the number
  `MUF_FLOOR`, and every branch of the scale synthesis are left exactly as
  they are gated today.
- **No banked artifact.** No type that `IntegrateArtifact` serialises gains or
  loses a field; `PointScales` is a return type, never stored.
- **No fixed-beam run.** A.1's `beam_has_pdf` argument is upstream of the
  routing, and the amendment converts that assumption into an `unreachable!`.
- **No replay gate.** `ScaleChoice::cluster_scales` is unchanged by design.
- **Strictly fewer aborts.** Every path this amendment changes turns a panic
  into either a zero weight or a typed refusal; none turns anything into a panic
  that was not one before.

**Blind spots, stated because nothing here closes them:**

- The amendment cannot show the veto's *threshold* is right. `MUF_FLOOR = 4.0`
  is a transcription, pinned by reading `reweight.f`, and no banked run
  approaches it — the nearest is 2.35× away. If the constant were wrong, every
  test above would still pass.
- T5a′/T5b′ construct sub-threshold support with `scalefact`, so they exercise
  the floor through one particular route into it. A floor reached by a different
  branch of the μF synthesis (`Backfill1/2`, `Beam1FromFirst`, …) is covered
  only by the shared comparison at `setclscales.rs:468-473`, not by an
  independent point.
- Refusing choices 1–5 leaves the closed forms unvalidated, which is the
  intended state, but it also means the refusal itself is the only thing
  standing between a user and an unpinned scale. If a later chain wires them,
  the oracle has to come first.

## Chain B design (2026-08-03)

Design session, read-only but for this section. Branch `val4-b`, cut from `val4`
at `8ed8467` — after chain A's per-member colour-flow merge. Every claim about
this crate below was read out of *that* tree and carries a `file:line`; every
claim about MadGraph was read out of the pinned checkout under `research/refs/`
or out of a banked run directory, and carries a path.

### B.0 What the code does today, and the three facts the design rests on

**Fact 1 — the scale reads the sampler's channel, as a `(group, diagram)` pair.**
`SampledChannel` (`vibegraph-lib/src/hadronic.rs:152`) is exactly that pair.
`EventScaleSource::scales` (`hadronic.rs:313`) indexes its channel sets by
`channel.group` and turns `channel.diagram` into an integration configuration
through `Channels::config_of_channel` (`hadronic.rs:203`), which falls back to
the set's `default_config` for a diagram the vertex filter dropped. That
configuration becomes `ClusterInput::this_config` (`coupling/scales.rs:110-126`)
and selects both the merge table and the resonance tagging inside
`ScaleChoice::cluster_scales` (`coupling/scales.rs:331`). The two production
call sites are `ProtonIntegrand::shape` (`proton.rs:1297`, via `event_scales`
at `proton.rs:1335` and `sampled_channel` at `proton.rs:1346`) and
`FixedBeamIntegrand::event_scales_of` (`hadronic.rs:1019`, through
`SampledChannel::sole`).

**Fact 2 — the machinery for a per-point `AMP2` draw already exists, and is
already gated.** `BoundAmplitude::eval_amp2` (`helas/eval/run.rs:356`) fills
`AMP2(d) = Σ_hel Σ_chains |A_d(p)|²` per *integration configuration*, in the
order `AmplitudeEvaluator::config_diagrams` (`helas/eval/compile.rs:432`) names,
with `n_configs` at `compile.rs:438`. `select_index` (`select.rs:26`) is the
categorical draw. `AmplitudeEvaluator::select_color_flow`
(`compile.rs:355`) already draws a configuration `∝ AMP2(d)` per accepted event
and uses it to mask the flow draw, and that draw is *measured against MadGraph*:
`generated_b_quark_events_agree_with_madgraphs_banked_ones`
(`vibegraph-cli/tests/validate_samples_proton.rs:501`) states that
`pp_to_bb_fixed`'s two sub-percent `ICOLUP` flows land at MadGraph's
`0.07%`/`0.08%` "only if the configuration draw carries the right shares". So
the *distribution* this chain wants to move the scale onto is not a new
hypothesis — it is an existing, χ²-gated one. What is new is that it stops being
a selection and starts entering the integrand.

**Fact 3 — an accepted point is reconstructed from `(channel, u)` and nothing
else.** `Unweighter::trial` (`unweight.rs:330`) returns
`AcceptedPoint { channel, u, weight }` (`unweight.rs:161`) after calling
`value_in_channel`, and `vibegraph-cli/src/generate.rs:318`/`:776` rebuild the
event with `event_in_channel(point.channel, &point.u)` — a second, independent
evaluation. `ProtonIntegrand::event_in_channel` (`proton.rs:1418`) says so in
its own words: "the same map at the same `u`, hence the same weight".

> **Fact 3 is the binding constraint of this whole design.** Any randomness the
> scale draw consumes must be a *pure function of the arguments
> `value_in_channel` and `event_in_channel` both receive*. A counter advanced
> per integrand call satisfies neither: the trial loop makes rejected calls
> between the accepted one and its reconstruction, and the reconstruction would
> then draw a different configuration, giving an event whose recorded `SCALUP`
> is not the scale its own accept/reject weight was computed at. That is the
> "a sample whose scales disagree with its integral is a new defect" the chain
> brief names, and it is reachable by the obvious implementation.

### B.1 What MadEvent's rule actually is — the note's `∝ AMP2_c` is conditional

The chain brief and TODO's standing-discrepancy entry both state the target rule
as "single-diagram enhancement weights channel `c` by `AMP2_c/Σ AMP2`". Read
against the generated Fortran, that is true **only under a run-card condition
neither document names**. The enhancement block is
`validation/madgraph/output/gu_to_epemu/SubProcesses/P1_gq_llq/matrix1_orig.f:291-317`:

```fortran
      IF (MULTI_CHANNEL) THEN
        XTOT=0D0
        DO I=1,LMAXCONFIGS
          J = CONFSUB(1, I)
          IF (J.NE.0) THEN
            IF(SDE_STRAT.EQ.1) THEN
              AMP2(J) = AMP2(J) * GET_CHANNEL_CUT(P, I)
              XTOT=XTOT+AMP2(J)
            ELSE
              AMP2(J) = GET_CHANNEL_CUT(P, I)
              XTOT=XTOT+AMP2(J)
            ENDIF
          ENDIF
        ENDDO
        IF (XTOT.NE.0D0) THEN
          ANS=ANS*AMP2(CHANNEL)/XTOT
```

So the per-point configuration weight is `AMP2_c · CC_c` at `sde_strategy = 1`
and `CC_c` **alone** at `sde_strategy = 2` — the squared amplitude is discarded
outright in the second case. `CC_c` is `get_channel_cut`
(`research/refs/mg5amcnlo/Template/LO/SubProcesses/genps.f:1817`), a product over
the configuration's propagator denominators. Its first statement
(`genps.f:1879-1881`) is what rescues the simple rule:

```fortran
      if(sde_strat.eq.1.and.tmin_for_channel.eq.-1)then
         get_channel_cut = 1d0
         return
      endif
```

Measured on the cards themselves (`grep sde_strategy` over
`validation/madgraph/output/*/Cards/run_card.dat`): every row this chain touches
runs `sde_strategy = 1` — `gu_to_epemu`, `gux_to_epemux`, `uux_to_epemg`,
`pp_to_llj_dyn`, `pp_to_jj`, `ee_to_mumua` — and `tmin_for_channel` is set in no
card, so it takes its default `-1.0`
(`validation/madgraph/runcard_defaults.json:243`, transcribed at
`vibegraph-lib/src/runcard.rs:596`). **`ee_to_mumu_tata_qcd0` is the exception:
its card carries `sde_strategy = 2`**, where MadEvent's configuration
distribution is not a function of the amplitude at all.

**Design consequence, and it is a hard edge rather than a tolerance.** The draw
this chain installs is `∝ AMP2_c(p)` and is therefore MadEvent's rule *only*
when `sde_strategy == 1 && tmin_for_channel == -1.0`. The implementation must
read both fields and apply the draw only under that conjunction; on any other
card the scale keeps the sampler's channel exactly as today, and the row keeps
its existing partition justification. Both fields are already parsed
(`runcard.rs:596`, `runcard.rs:697`) and — as far as this session could find —
read by nothing, so chain B is what turns them into read fields. **The sprint
manager should tell chain C2 this**, since an audit of parsed-but-unread fields
would otherwise propose a refusal for exactly these two.

*Falsifier for the conditional:* a test that compiles a scale source from a card
with `sde_strategy = 2` and asserts the prescription reports that it is **not**
drawing (and that a `sde_strategy = 1` card reports that it is). Without it the
condition is an unexercised branch, and the `var_sde1` run directory exists
precisely because this field has bitten before.

### B.2 The pre-registered movement census — P0, before any production line changes

The chain brief names "chain B moves a σ row it should not" as a user-escalation
trigger. This design turns that from a judgement into a mechanical comparison, by
measuring *first* which rows are even capable of moving.

**The criterion.** The scale enters σ and nothing else about the draw does. So a
row whose `μR` and both `μF` are the *same number* in every integration
configuration at every phase-space point cannot move, whatever configuration is
drawn — not "usually", but identically. That is already how K6 explained the two
annihilation rows staying bit-identical (§K6.4: `μR` spread `0.000e0` on
`uux_to_epemg`/`ddx_to_epemg` against `9.93e-1` on the two gluon rows), and the
instrument exists: `the_sampled_channel_reaches_the_cluster_scale`
(`vibegraph-lib/tests/validate_sigma.rs:1047`) already evaluates `μR` in every
channel at a drawn point and reports the spread.

**P0 deliverable.** Widen that instrument — or add a sibling probe beside it —
to report, for **every** gated row whose prescription is the clustering branch,
the worst relative spread of `μR` and of each `μF` over *all* integration
configurations (`Channels::len()`), over a few dozen cut-passing points. The
clustered rows are, from `dynamical_scale_choice = -1` with the `fixed_*_scale`
booleans off:

`ddx_to_epemg`, `ee_to_ee`, `ee_to_mumu`, `ee_to_mumua`, `ee_to_mumu_tata_qcd0`,
`ee_to_tatah`, `ee_to_ttx`, `ee_to_wpwm`, `ee_to_zh`, `gg_to_gg`, `gg_to_ttx`,
`gu_to_epemu`, `gux_to_epemux`, `pp_to_jj`, `pp_to_llj_dyn`, `uux_to_epemg`,
`uux_to_mumu`, `uux_to_uux` — 18 of the 23 `integrals` rows.

The remaining five are fully fixed (`fixed_ren_scale` and both
`fixed_fac_scale*` true) and so resolve to `ScaleSourceKind::Constant` before any
event is seen: `pp_to_bb_fixed`, `pp_to_llj_fixed`, `ud_to_epemud_qcd0` and both
`pp_to_ll` variants (the committed `dy13_*_run_card.dat`). **They execute no
changed line at all.**

**The pre-registration, written before the measurement.** Structurally, a
`2 → 2` final state gives the clustering no merge to choose — §C.3's argument,
and the reason `pp_to_jj`'s partition gap is its own Monte-Carlo error. So the
prediction is *zero spread* on the ten `2 → 2` rows (`ee_to_ee`, `ee_to_mumu`,
`ee_to_ttx`, `ee_to_zh`, `ee_to_wpwm`, `gg_to_gg`, `gg_to_ttx`, `uux_to_uux`,
`uux_to_mumu`, `pp_to_jj`), zero on `uux_to_epemg`/`ddx_to_epemg` (already
measured at `0.000e0`, §K6.4), and *nonzero* on `gu_to_epemu`/`gux_to_epemux`
(already measured at `9.93e-1`). The three genuinely unknown rows are
**`ee_to_mumua`** (2 → 3, 8 configs), **`ee_to_tatah`** (2 → 3, 5 configs) and
**`ee_to_mumu_tata_qcd0`** (2 → 4, 25 configs, and the `sde_strategy = 2` card);
config counts from each run's `SubProcesses/*/config_nqcd.inc`.

**`ee_to_mumua` is chain D's row.** If its spread is nonzero, chain B moves the
very number chain D is measuring the `+0.80%` drift on, and the two chains
collide. That is a manager decision, not an implementation one: the P0 census
must be reported *before* implementation is dispatched, and if `ee_to_mumua`
spreads, the manager decides whether B is resequenced behind D, whether D's
measurement is retaken after B, or whether the draw is scoped to hadronic and
QCD rows for v0.1.

**How the census is used afterwards.** Every row with exactly-zero spread is
pre-registered as **must be bit-identical** — not "within tolerance", *equal*,
because §B.3's stream discipline leaves its point sequence untouched and the
scale it evaluates at unchanged. Every row with nonzero spread is pre-registered
as **may move**. Any deviation from that census in the after-comparison of §B.6
is stop-and-report, never fix-inline.

### B.3 Where the draw lives, and where its randomness comes from

**Decision 1 — the draw lives in the integrand, not in the scale prescription.**
`ScaleChoice::cluster_scales` takes a `ClusterInput` and has no amplitude, no
evaluator and no way to form `AMP2`. Pushing the draw down there would mean
handing the coupling layer an `AmplitudeEvaluator`, inverting the dependency the
crate is built on. The integrand already owns both the evaluator and the
momenta, so it forms the weights and hands the *resulting* configuration down
through the existing `SampledChannel`/`ClusterInput` seam. Nothing in
`coupling/` changes.

**Decision 2 — the draw's uniform travels with the point, and is drawn from a
dedicated substream that no existing stream shares.** Concretely:

* `ChannelIntegrand` (`unweight.rs:55`) gains
  `fn scale_draw_ndim(&self) -> usize { 0 }`, returning `1` exactly when the
  installed prescription is the clustering branch under §B.1's card condition.
* The slice handed to `value_in_channel` / `event_in_channel` is
  `channel_grid_ndim() + scale_draw_ndim()` long; both **assert** that length.
  The trailing coordinate is the scale-draw uniform `v`. A driver that forgets
  the tail fails loudly instead of silently clustering in the sampler's channel
  again — the record-layer self-check this defect class needs.
* `Unweighter` opens `SubStream::from_stream(seed, SCALE_DRAW_STREAM_BASE + j)`
  (`phasespace/rng.rs`, the same counter-based construction
  `CHANNEL_STREAM_BASE`/`SCAN_STREAM_BASE` use — `hadronic.rs:82`,
  `unweight.rs:45`) and fills the tail from it in both `scan` (`unweight.rs:206`)
  and `trial` (`unweight.rs:330`). **It must not draw `v` from the caller's
  `rng`**: that would shift the generate sequence of every clustered row,
  including the ones whose scale cannot move. `AcceptedPoint.u` then carries `v`,
  so the reconstruction at `generate.rs:318`/`:776` replays it exactly — Fact 3
  discharged by construction rather than by care.
* On the integration side, `adapt_grids_with` (`hadronic.rs:~1480`,
  `proton.rs:~1650`) wraps its per-channel closure over a scratch buffer and its
  own `SubStream::from_stream(seed, SCALE_DRAW_STREAM_BASE + j)`, leaving the
  grid's own `rng` (`hadronic.rs:1498`, `proton.rs:1664`) untouched. The
  integration never reconstructs a point, so a per-call advance is sound there.
* `MultiChannel::adapt_alphas`'s survey integrand
  (`phasespace/channel.rs:313`, `Fn(&[momenta], usize)`) gets the same treatment
  on its own stream, so the α-adaptation surveys the integrand the integration
  will run. `probe_scale` (`hadronic.rs:~985`, `proton.rs:~1177`) likewise.

> **The invariant this buys, and it is the load-bearing one:** *the scale draw
> consumes zero bits from any pre-existing stream.* Point sequences, channel
> selections, acceptance draws and VEGAS grids are bit-for-bit what they are
> today, on every row. Combined with §B.2's census, that is what makes
> "zero μ-spread ⇒ bit-identical σ" a prediction rather than a hope.

**Decision 3 — rejected: an extra VEGAS grid dimension.** The obvious
implementation appends the coordinate to `channel_grid_ndim()` so the grid draws
it. It is rejected for two independent reasons. (a) It changes `ndim` for all 18
clustered rows, so every one of them resamples and moves by Monte Carlo — the
ten `2 → 2` rows, `ee_to_mumua` and `ee_to_mumu_tata_qcd0` included — which is
precisely the outcome the escalation trigger exists to catch, manufactured on
purpose. (b) It puts a coordinate whose integrand dependence is a step function
with point-dependent breakpoints under grid refinement, which is the
ill-conditioned direction AGENTS.md's sampler rule warns about.

**Decision 4 — rejected: Rao–Blackwellising the draw away.** `σ` could be made
partition-free with no randomness at all by evaluating
`Σ_c w_c(p)·f(p, μ_c)` and grouping configurations by their resulting scale
triple — cheap where the scale collapses (one distinct scale on a `2 → 2`).
Rejected because an event still carries exactly one `SCALUP`, so the event pass
must draw regardless; a σ built from the mixture and events built from a draw
would give every accepted event a weight that is not `f(p, μ_recorded)`, which
is a new version of the same defect. It is also outside the chain's stated
scope. Worth one line in TODO as a variance-reduction idea, not as this chain's
design.

**Decision 5 — `AMP2` is evaluated at a pinned coupling, not at whatever is
bound.** `eval_amp2` reads the currently-set `αs`, and the scale is not known
until after the draw, so evaluating "at the event's coupling" is circular. Two
resolutions were considered:

* *Rely on αs-cancellation.* If every configuration of a subprocess carries the
  same power of `g`, the normalised weights are αs-independent and any bound
  value serves. Measured across every banked `SubProcesses/*/config_nqcd.inc`,
  this holds for **every** clustered row — all configurations share one `NQCD`
  — with a single exception, `pp_to_bb*`'s `P1_qq_bbx`, where `NQCD ∈ {0, 2}`.
  (`pp_to_bb` is a `2 → 2`, so its scale cannot depend on the configuration
  anyway, and it is one of §G's `nn23lo1`-blocked rows besides.) This also
  corrects a loose claim in the existing code: the doc of
  `the_sampled_channel_reaches_the_cluster_scale` (`validate_sigma.rs:1047`)
  attributes the gluon rows' `μR` spread to "the merge graph's coupling-order
  filter admits different channel sets for different `nqcd`", but
  `validation/madgraph/output/gu_to_epemu/SubProcesses/P1_gq_llq/config_nqcd.inc`
  reads `NQCD(1..4) = 1` — every configuration the same order, so that filter
  (`coupling/cluster/graph.rs:181-186`) does nothing on that row. The spread
  comes from the differing *forests*, not from the coupling order. Worth fixing
  in that comment while the file is open.
* *Pin the coupling.* Evaluate `AMP2` at the coupling the amplitudes were bound
  at (`RunningCouplingReport::alpha_s_ref`), then set `αs(μR)` for the matrix
  element. Costs one extra `set_alpha_s` per point.

**Take the second.** It makes the drawn configuration a pure function of the
momenta with no cancellation hypothesis behind it, it survives the mixed-`NQCD`
case without a special path, and it removes any dependence of the draw on
evaluation history — which matters because `event_in_channel` runs after
intervening trials have moved the bound coupling. The cancellation argument then
becomes a *test* rather than a load-bearing assumption: assert that on
`gu_to_epemu` the drawn configuration is unchanged when `AMP2` is formed at two
different couplings. Where `set_alpha_s` falls back to a full model
re-evaluation (`RunningCouplingReport::fallbacks`), this doubles that cost per
point; the implementation must report the measured wall-time change per gated
row rather than absorb it.

**Decision 6 — the index composition, through the diagram and not by assumption.**
`AMP2` index `c` names a configuration in `AmplitudeEvaluator`'s order;
`ClusterInput::this_config` names one in `derive_channels`' order. These are two
independent derivations from the same diagram slice, and §K6.2 already records
that the diagram→config map is *not* the identity (`g g → g g`: 4 diagrams, 3
configs). Compose through the diagram index, which is the common ground:

```
this_config = channels.config_of_channel( eval.config_diagrams()[c] )
```

and, separately, add a test asserting the two orders agree
(`config_of_diagram(config_diagrams()[c]) == Some(c + 1)`) and that
`eval.n_configs() == channels.len()`, over the banked process set. Production
must not depend on that identity; the test exists so a reorder is a named
finding instead of a silent reshuffle of every event's scale.

**Decision 7 — the fallback, counted.** `select_index` returns `None` when the
weights carry no probability (`select.rs:26-40`). Take the set's
`default_config` there, matching `select_color_flow`'s own fallback
(`compile.rs:~372`), and count it into the `RunningCouplingReport` beside
`unmapped_channels` so a run that hits it says so rather than absorbing it.
MadEvent's counterpart is the `NB_FAIL` branch at `matrix1_orig.f:306-315`,
which stops the run after ten such points — a useful sanity bound on how often
this is expected to fire (essentially never).

### B.4 The group is the other half, and the note's scope does not reach it

**What the code does.** `ProtonIntegrand::shape` (`proton.rs:1297`) computes
**one** scale from the sampled channel and applies it to the whole sum over
flavour groups:

```rust
let scales = self.event_scales(channel);
self.apply_scale(scales.mu_r);
for (g, sub) in self.groups.groups().iter().zip(&self.subs) { ... }
```

**Why that is a partition dependence the configuration draw does not touch.**
`SampledChannel` is a `(group, diagram)` pair, and §K6.3 records that the groups
of `p p → ℓ⁺ℓ⁻ j` "do not share a merge graph" — `g u → ℓ⁺ℓ⁻ u` and
`u ū → ℓ⁺ℓ⁻ g` cluster differently at the same momenta. §K6.4 measures those two
subprocess types at `μR` spreads of `9.93e-1` and `0.000e0`, so the two groups
genuinely disagree about the scale at a shared point. Drawing the *configuration*
`∝ AMP2` fixes which config inside the sampled group is used; it leaves the
**group** taken from the sampler, so `σ` still depends on `αⱼ` through which
group's forests a region of phase space is evaluated with. MadEvent has no
counterpart to this at all: it integrates each subprocess directory separately,
each at its own scale, and sums.

**This is where the note's chain B entry falls short of its own acceptance
criterion.** E2 asks for `pp_to_llj` below `0.015` "at tolerances justified by
the reference's own error". `pp_to_llj_dyn` is a 6-group, 24-pooled-channel
hadronic row (`validate_hadronic.rs:~103`), and there is no measurement anywhere
in the bank that says the residual is dominated by the within-group
configuration choice rather than by the cross-group one. The literal scope —
configuration draw only — is **complete and exact for the fixed-beam rows**
(`gu_to_epemu`, `gux_to_epemux` have one subprocess, so the group question does
not arise, and E2's first half is fully reachable), and **only partial for the
hadronic row**.

**Design response — a measurement decides, not an assumption.** Split the work:

* **B-1 (in scope, the note's own):** the configuration draw, all rows. Retires
  the partition dependence on the fixed-beam rows outright.
* **B-2 (conditional):** each flavour group's term evaluated at *its own*
  group's drawn configuration — one clustering, one `AMP2` and one coupling per
  group with nonzero luminosity per point, replacing the single shared scale.
  This removes the sampler from the hadronic scale entirely and is what MadEvent
  does. It is a restructuring of `shape` and of `apply_scale` (`proton.rs:1369`,
  today one global coupling for all subprocesses) and it multiplies the
  per-point clustering and coupling cost by the group count.

**The gate between them, run at the end of B-1:** report `pp_to_llj_dyn`'s
`μR` spread decomposed into *within-group* and *across-group* parts at the same
points, and re-run `probe_llj_dyn_budget_ladder` (`validate_hadronic.rs:1090`)
under B-1. If the row's residual has collapsed into the reference's own `0.33%`,
B-2 is unnecessary and should not be built. If it has not, B-2 is what E2 needs,
and **the manager decides whether it is in this sprint** — it is a scope
extension beyond the chain brief and the design says so rather than smuggling it
in. Either outcome is a recorded measurement, which is the point.

### B.5 The rider — `pp_to_llj_dyn`'s `samples` cell

Smaller than it reads. `validate_samples_proton.rs` drives every proton
`samples` row through one `Row` struct (`:118-137`): a banked run directory, a
manifest key, a process string, an optional committed card, a budget and a mode.
`generated_proton_events_agree_with_madgraphs_banked_ones` (`:472`) is the
`pp_to_llj_fixed` row, and `pp_to_llj_dyn`'s banked sample is already in the
bundle (`validation/manifest.toml:495` lists
`output/pp_to_llj_dyn/Events/run_01/unweighted_events.lhe.gz`, and no row in the
manifest carries `bundled = false` any more).

* Add one `Row` with `run`/`key` `"pp_to_llj_dyn"`, `events: "run_01"`,
  `process: "p p > l+ l- j QCD=2 QED=2"`, `run_card: None` (the run's own card is
  the dynamical one — that *is* the row), the same `NEVAL`/`NITER`/seeds as the
  fixed-scale row, and `scans: &[]`.
* Flip `validation/manifest.toml`'s `pp_to_llj_dyn` `samples` entry from
  `tier = "uncovered"` to `tier = "banked", mode = ...`, with a note recording
  what the comparison measured. Mode is a *measurement outcome*: `gate` if every
  column clears the `1e-4` floor on all three seeds, `info` with the recorded
  reason if not. Do not pre-declare `gate`.
* The header comment of `validate_samples_proton.rs:14-16` says "Three rows" and
  is already stale (the dijet row at `:770` is a fourth); make it right while
  adding the fifth.

Taken here because B-1/B-2 rebuild exactly the path this cell exercises: a
`samples` cell measured before the draw lands would have to be measured again
after it.

### B.6 The before/after σ comparison — the escalation trigger, mechanised

`pixi run --skip-deps validate` writes one JSON per cell under
`target/validation-report/{integrals,samples}/` (schema at
`target/validation-report/integrals/pp_to_llj_dyn.json`: `sigma_vg_pb`,
`sigma_vg_err_pb`, `rel`, `pull`, `chi2_dof`, `per_seed`), and
`validation/validate.sh:23` deletes them before each run so a cell is always
*this* invocation's measurement. That directory is the comparison artifact.

**The procedure, mandatory and in this order:**

1. **Before touching a production line**, on a clean `val4-b`:
   `pixi run --skip-deps validate`, then
   `cp -Rc target/validation-report /tmp/valreport-baseline-B` (`cp -Rc` is
   instant on APFS). Record the census line from `report.md`.
2. Run the §B.2 P0 census and report it to the manager. **Stop here for the
   dispatch decision if `ee_to_mumua` spreads.**
3. Implement, in the stages of §B.7.
4. After each stage: `pixi run --skip-deps validate`, then
   `diff -r /tmp/valreport-baseline-B/integrals target/validation-report/integrals`
   and the same for `samples`.
5. **Read the diff against the §B.2 census, cell by cell.** A row with zero
   μ-spread whose `sigma_vg_pb` differs *in any digit* is a defect in the stream
   discipline of §B.3, not statistics — the drawn configuration cannot have
   changed its scale, so nothing in the estimator may have moved. A row that is
   fully fixed (the five of §B.2) differing at all means changed code ran on a
   path that should not execute it.
6. **Any unexpected row that moved is stop-and-report to the sprint manager.**
   Do not retune, do not re-seed, do not widen a tolerance to absorb it. Write
   the row, the before and after `sigma_vg_pb`/`rel`, and what the census
   predicted.

The five fully-fixed rows and the zero-spread clustered rows should diff to
**nothing at all** — byte-identical JSON. That is a stronger statement than
"inside tolerance" and it is available here, so it is the one to make.

### B.7 Stages, gates, and expected pass/fail at each

Every command is `--skip-deps`; a bare `pixi run validate` (or any task with a
MadGraph dependency taken live) triggers a multi-hour regeneration. Long runs go
in the background per the sprint's long-command discipline and are polled.

| # | Stage | Model | Gates to run | Expected state |
|---|---|---|---|---|
| B-0 | Baseline capture + P0 μ-spread census (§B.2), no production change | **Opus** (judgement on the census) | `pixi run --skip-deps validate`; the new census probe under `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_sigma -- --nocapture <probe> --ignored` | validate **passes**, unchanged from `main`'s report. Census reported to the manager before B-1 is written. |
| B-1a | The draw plumbing: `scale_draw_ndim`, the `u` tail, the substreams, the length asserts — wired but with the drawn config **discarded** and the sampler's still used | **Opus** | `pixi run --skip-deps validate`; `cargo test -p vibegraph-lib --features extended-validation --test validate_unweighting` | validate **passes bit-identically** on all 23 `integrals` and all `samples` rows. This is the negative control: the plumbing must be provably inert. If any cell moves here, the stream discipline is wrong and nothing else should be built on it. |
| B-1b | The draw goes live: `AMP2` at the pinned coupling, index composition, fallback counting, the `sde_strategy`/`tmin_for_channel` condition | **Opus** | `pixi run --skip-deps validate`; `pixi run -e madgraph --skip-deps validate-sigma`; `pixi run -e madgraph --skip-deps validate-hadronic`; `pixi run -e madgraph --skip-deps validate-scales`; `pixi run -e madgraph --skip-deps validate-generate-proton` | `validate-scales` **must pass unchanged** — it replays MadGraph's *own* banked events at *its* channel and is a property of the bank, not of any integrand (§K6.7), so a change there means the prescription itself moved. σ rows: only the §B.2 census's may-move set differs. `gu_to_epemu`/`gux_to_epemux` are expected to move; their `rel_tol` is still `0.02`, so they should still **pass** while moving. |
| B-1c | The partition instruments re-read, tolerances re-justified (§B.8) | **Opus** | `probe_channel_partition_moves_sigma` (`validate_sigma.rs:1247`), `probe_llj_dyn_budget_ladder` (`validate_hadronic.rs:1090`), the five-seed sweeps, both `--ignored` under the extended-validation feature | The named numbers below. |
| B-2 | Per-group scales, **only if B-1c's measurement says E2 needs it and the manager approves** | **Opus** | as B-1b, plus a wall-time comparison per hadronic row | `pp_to_llj_dyn` moves; the fixed-beam rows must be **bit-identical to B-1b** (they have one group, so B-2 executes no changed branch for them). |
| B-3 | The rider: the `samples` row + manifest + the stale header comment (§B.5) | **Sonnet** | `pixi run --skip-deps validate` | The new cell is written and the census gains one measured cell. No other cell moves. |
| B-4 | Artifact-version guard (§B.9) | **Sonnet** | `cargo test -p vibegraph --features extended-validation --test cli_generate`, `--test cli_integrate` | A pre-change artifact plus a clustering run card is refused with a message naming the scale rule; a fixed-scale card still loads. |
| B-5 | The note section: measurements, tolerances, what each instrument cannot see | **Opus** | — | — |

**The known-wrong informational comparison to keep running throughout.**
`probe_channel_partition_moves_sigma` is exactly that instrument and it already
exists: it integrates the same row at the converged `αⱼ` and at uniform `αⱼ` and
reports the gap. Its banked readings are the pre-registered before-values —
`gu_to_epemu` `−1.48e-2`, `gux_to_epemux` `−1.53e-2`, against `uux_to_epemg`
`+1.05e-3` and `ddx_to_epemg` `+1.86e-3` as the negative control, all at a
Monte-Carlo error of about `1.6e-3` (§K6.5). Run it at **B-0** (must reproduce
those four numbers — if it does not, the environment is not the one the bank was
taken in, and nothing downstream means anything), at **B-1a** (must be
unchanged), and at **B-1b**, where the whole claim of this chain is that all four
rows collapse to their own Monte-Carlo error. That single table is the chain's
end-to-end signal, and it is available from the first line of code.

### B.8 Tolerances — the decision rule, pre-registered, not the number

E2 asks for the partition tolerances retired. The honest form of that is a rule
fixed before the measurement:

* A row leaves `PULL_REPORTED_NOT_ASSERTED` (`validate_sigma.rs:145`) **only if**
  its residual behaves like Monte Carlo: it shrinks with budget across the
  ladder, and its five-seed scatter sits at `χ²/dof ≈ 1`. A residual of fixed
  size that merely got smaller is still a systematic, and asserting its pull
  would assert a precision the comparison does not have — the exact reasoning at
  `validate_sigma.rs:130-145`.
* `rel_tol` is then set at the larger of (a) the reference's own Monte-Carlo
  error with headroom — `0.18%` on `gu_to_epemu`/`gux_to_epemux`, `0.33%` on
  `pp_to_llj_dyn` — and (b) the measured five-seed spread. Not fitted around the
  achieved central value.
* `LLJ_DYN_MAX_REL` (`validate_hadronic.rs:132`) and the two `0.02` entries
  (`validate_sigma.rs:369`) move only on that basis, and their doc comments must
  be rewritten to say what the new number is set by. A tolerance whose comment
  still cites the partition band while the partition is gone is a false comment.
* **If the residual does not become Monte Carlo, the tolerances do not move.**
  E2 is then partially met, with a diagnosis, and that is reported — not
  absorbed. A row left where it is with a measurement behind it is a better
  outcome than a tightened row resting on one lucky seed.

`pp_to_jj` is explicitly **not** re-justified here: TODO records it as GATE at
`rel_tol` 0.005 with the pull asserted, set at the reference's own `0.22%`,
because its partition gap is `1.03e-3` against its own `9.6e-4` Monte Carlo
(§C.3). It is a zero-spread row under §B.2's prediction and should come out of
this chain byte-identical.

### B.9 Artifacts

`FORMAT_VERSION` is `6` and `OLDEST_READABLE_VERSION` is `3`
(`vibegraph-lib/src/artifact.rs:45,48`). **The grids do not change meaning**:
their coordinate count is untouched (§B.3 Decision 3 was rejected precisely to
keep it so), the channel decomposition is unchanged, and a VEGAS grid is an
importance weight that remains valid under a changed integrand — it can only be
suboptimal, never wrong.

**One thing does change meaning: the recorded `sigma_pb`.** `vibegraph generate`
takes the file's `XSECUP` from `artifact.sigma_pb`
(`vibegraph-cli/src/generate.rs:553` and `:897`) while the sample's own cross
section comes from `sigma_from_events` (`:377`, `:824`). So an artifact written
before this chain, replayed after it, writes an old-rule `XSECUP` onto a
new-rule sample. That is a boundary a card can reach silently, which is the class
this whole sprint is closing.

**Recommendation:** bump `FORMAT_VERSION` to `7` with no schema change, leave
`OLDEST_READABLE_VERSION` at `3`, and refuse in `generate` when the artifact's
version is `< 7` **and** the run card selects the clustering branch — naming the
scale rule in the message, not a plan item. A fixed-scale artifact keeps
working, which is most of them. The implementation should first confirm no
artifact is committed to the repository (`git ls-files | grep -i 'grid.bin'`);
this session found none, and every test writes its artifact into a `tempfile`
directory (`validate_samples_proton.rs:~205`).

### B.10 Acceptance tests, named

1. `the_scale_channel_is_drawn_from_amp2_and_not_from_the_sampler` — at a fixed
   point on `gu_to_epemu`, sweep the scale-draw uniform across `[0,1)` and assert
   the drawn configuration's empirical frequencies match
   `AMP2_c / Σ AMP2` (the same convergence property `select.rs`'s
   `frequencies_converge_to_the_weight_fractions` asserts), and that the drawn
   configuration is **independent of the sampling channel the point came from**.
   The second half is the one that would fail if the sampler leaked back in.
2. `a_scale_draw_replays_identically_on_reconstruction` — run
   `Unweighter::trial` to an accepted point, then `event_in_channel` on
   `AcceptedPoint.u`, and assert the event's `EventScales` equals the scale the
   trial's own `value_in_channel` evaluated at, **after** interleaving further
   rejected trials between the two. Without the interleaving the test cannot see
   the Fact-3 defect at all, which is the point of specifying it.
3. `the_amp2_config_order_matches_the_forest_config_order` — over the banked
   process set, `eval.n_configs() == channels.len()` and
   `channels.config_of_diagram(eval.config_diagrams()[c]) == Some(c + 1)`.
   Production does not rely on it (§B.3 Decision 6); the test turns a reorder
   into a finding.
4. `the_scale_draw_is_independent_of_the_bound_coupling` — form `AMP2` at two
   different `αs` on `gu_to_epemu` and assert the drawn configuration is the
   same, pinning Decision 5's pinned-coupling behaviour and, incidentally, the
   uniform-`NQCD` cancellation.
5. `a_non_default_sde_strategy_keeps_the_sampler_channel` — a card with
   `sde_strategy = 2`, and one with `tmin_for_channel` set, both report a
   prescription that does **not** draw; a default card reports one that does.
   §B.1's falsifier.
6. `the_clustered_rows_that_cannot_move_are_bit_identical` — the §B.2 census
   promoted into a standing test: every gated clustered row's worst μ-spread over
   all configurations is recorded, and the rows reading exactly `0.0` are
   asserted to. It is the guard that fails if a future change makes a scale
   configuration-dependent on a row this chain declared inert.
7. The rider's own cell (§B.5) and the artifact refusal (§B.9).

Two of these — 1 and 4 — are `--ignored` oracle-layer probes if they need a full
integrand build; the rest belong in the banked suite.

### B.11 Risks, and what this provably cannot break

**Provably cannot break** (each with the reason, not the assurance):

* The five fully-fixed rows (`pp_to_bb_fixed`, `pp_to_llj_fixed`,
  `ud_to_epemud_qcd0`, both `pp_to_ll`). `ScaleChoice::is_fully_fixed`
  (`coupling/scales.rs:266`) resolves them to `ScaleSourceKind::Constant` at
  compile time (`hadronic.rs:~272`), `needs_channels()` is false
  (`scales.rs:274`), `scale_draw_ndim()` is `0`, and no changed line executes.
* Every row whose prescription is a closed form (`dynamical_scale_choice` 1–5) —
  same argument, different branch.
* `validate_scales`, `validate_kt_cluster`, `validate_alphas`,
  `probe_first_channel_cost_in_alpha_s`. All replay MadGraph's *own* banked
  events at *MadGraph's* channel; they are properties of the bank and of
  `coupling/`, and this chain changes no line under `coupling/` (§B.3
  Decision 1). §K6.7 makes exactly this argument for the same reason.
* The amplitude, colour and diagram gates. `eval_amp2` is called, not modified.
* The phase-space maps, the channel `αⱼ` values, the VEGAS grids and the
  unweighting acceptance sequence, by the zero-bits invariant of §B.3.

**Risks:**

* **`ee_to_mumua` spreads and collides with chain D** (§B.2). Mitigated by
  measuring first and reporting before dispatch; resolution is the manager's.
* **`ee_to_mumu_tata_qcd0` spreads *and* runs `sde_strategy = 2`.** Then this
  crate has no implementation of MadEvent's configuration distribution for that
  card, the §B.1 condition leaves it on the sampler's channel, and the row keeps
  a partition ambiguity that this chain cannot retire. That is a recorded
  limitation, not a defect to paper over — and it is a candidate TODO entry
  (implement `get_channel_cut`), not sprint work.
* **The draw is unbiased but noisier.** It adds a categorical step inside the
  integrand, so per-point variance rises on the rows that actually move. Watch
  `err_vg` and `χ²/dof` on the moving rows across the seed sweep; AGENTS.md's
  rule applies — if extra budget makes a failure migrate between seeds rather
  than shrink, it is a bug.
* **Cost.** One `eval_amp2` per point per subprocess is roughly one more matrix
  element's worth of arithmetic, and Decision 5 adds a `set_alpha_s`. If the
  measured slowdown on `pp_to_llj_dyn` or `pp_to_jj` is material, the natural
  remedy is a combined entry point that fills `AMP2` from the same arena
  `eval_m2` already fills (`helas/eval/run.rs`, both call `fill_arenas`) rather
  than running the program twice — an evaluator change to size before writing,
  and to hand to performance work if it grows.
* **The mirror ordering.** `shape` evaluates a group's direct and reflected terms
  at one shared scale (`proton.rs:1297-1320`), so there is one `AMP2` to draw
  from but two arguments it could be formed at. Take the direct ordering, and
  name the falsifier: `σ` must not move outside Monte Carlo when the draw is
  formed from the luminosity-weighted combination of both instead. If it does,
  the ordering is a third partition axis and that is a finding for the manager,
  not an inline fix. Pre-existing, not introduced here.
* **Merge conflicts.** B is merged last (§5) and touches
  `validate_sigma.rs`'s plan table, `validate_hadronic.rs`'s tolerance
  constants, `validate_samples_proton.rs` and `validation/manifest.toml` — the
  declared conflict hotspot. Rows are disjoint from the other chains'; the
  manager resolves.

**What this provably cannot decide.** Whether the residual that survives on
`pp_to_llj_dyn` after B-1 is the flavour-group axis (§B.4) or something else:
only B-2, or a group-resolved measurement, separates them. And nothing here says
anything about `p p → j j`'s `+0.21%`, which is Monte Carlo against a reference
whose own error is `0.22%` and which no scale rule can move.

### B.12 Errors found in the chain brief and in the note

* **The brief's premise is sound and its checks all passed**: toplevel, branch
  `val4-b`, `HEAD = 8ed84676`, clean tree. Chain A's merge (`8ed8467`, "per-member
  colour-flow tables via structurally-determined permutation") is in this
  history, as are C1 (`6018549`) and E (`6f95f83`).
* **The note's chain B entry states MadEvent's rule unconditionally** as
  "weights the integrand of channel `c` by `AMP2_c/Σ AMP2`" (§K6.5, §K6.8, and
  TODO's standing-discrepancy entry). It is conditional on
  `sde_strategy = 1 && tmin_for_channel = -1` (§B.1), and one gated row in the
  set — `ee_to_mumu_tata_qcd0` — does not meet it.
* **The note's chain B scope does not reach E2's second half.** The
  configuration draw leaves the flavour-*group* half of `SampledChannel` on the
  sampler, so `pp_to_llj_dyn`'s σ remains partition-dependent through the group
  (§B.4). The spec predates nothing here — this is a gap in the plan, not a
  consequence of chain A — and it is flagged rather than silently expanded into.
* **Chain A's merge does not conflict with anything in this design.** A's change
  is in `lhef/build.rs`'s per-member colour-flow tables; the scale path is
  disjoint. The one place the two meet is §5's stated reason for ordering A
  before B — B's rider regenerates llj samples that must carry A's fixed tags —
  and that holds.
* **A stale doc comment found en route**, worth correcting while the file is
  open: `the_sampled_channel_reaches_the_cluster_scale`
  (`validate_sigma.rs:1047`) attributes the gluon rows' `μR` spread to the
  coupling-order filter admitting different channel sets for different `nqcd`,
  but that run's `config_nqcd.inc` has `NQCD = 1` on all four configurations, so
  the filter is inert there (§B.3 Decision 5).
* **`validate_samples_proton.rs:14-16` says "Three rows"** and there are four
  (§B.5).

### B-0 output — the baseline, and the pre-registered movement census (2026-08-03)

Implementation session, branch `val4-b` at `1cfd9aa`. No production line changed:
the two probes below are `--ignored` test code, and the baseline was captured from
this worktree before either was written.

**The baseline.** `pixi run --skip-deps validate` in the chain-B worktree, then
`cp -Rc target/validation-report /tmp/valreport-baseline-B`. The run is green and
its census line is

```
29 rows × 4 categories = 116 cells: 89 measured in this layer (87 ✅, 2 ⚠️, 4 ⏳, 8 ⛔, 15 — / uncovered).
```

with `[report] 29 rows x 4 categories: the measured cells are the declared cells`.
The comparison artifact is 23 `integrals` and 22 `samples` cells; over
`find integrals samples -type f | sort | xargs shasum -a 256 | shasum -a 256` the
tree digests to `4bc825da7976dec27bd7ec1a8f9f90b0711a04e2da68606068339ad35a4cd84f`
(`integrals` alone `762e649d…`, `samples` alone `3d6b511f…`).

#### B-0.1 The μ-spread census

Two probes, both `--ignored`:
`probe_cluster_scale_spread_over_configurations` in `validate_sigma.rs` for the
fixed-energy rows and in `validate_hadronic.rs` for the two proton rows. Each
reports the worst relative spread of `μR` and of both `μF` over *all* integration
configurations at one cut-passing point, worst over 64 points. The fixed-beam
sweep runs over sampling channels and is a sweep over configurations because each
surviving diagram yields exactly one configuration; the probe asserts
`channel_count() == diagram_count()` so that stays true. The hadronic probe
rebuilds the clustering from the run card and the groups' own diagrams — the
proton integrand exposes no accessor for its prescription — and checks that
reconstruction on every point against the scales the integrand itself recorded,
which passed on 64/64 points for both rows.

| row | predicted (§B.2) | measured μR spread | measured μF spread | verdict |
|---|---|---|---|---|
| `ee_to_ee` | zero (2→2) | **no prescription** | — | cannot move |
| `ee_to_mumu` | zero (2→2) | **no prescription** | — | cannot move |
| `ee_to_ttx` | zero (2→2) | **no prescription** | — | cannot move |
| `ee_to_zh` | zero (2→2) | **no prescription** | — | cannot move |
| `ee_to_wpwm` | zero (2→2) | **no prescription** | — | cannot move |
| `uux_to_mumu` | zero (2→2) | **no prescription** | — | cannot move |
| `ee_to_mumua` | **unknown** | **no prescription** | — | cannot move |
| `ee_to_tatah` | **unknown** | **no prescription** | — | cannot move |
| `ee_to_mumu_tata_qcd0` | **unknown** | **no prescription** | — | cannot move |
| `gg_to_gg` | zero (2→2) | `0.000e0` (3 configs, 1 unmapped) | `0.000e0` | confirmed |
| `gg_to_ttx` | zero (2→2) | `0.000e0` (3 configs) | `0.000e0` | confirmed |
| `uux_to_uux` | zero (2→2) | `0.000e0` (2 configs) | `0.000e0` | confirmed |
| `uux_to_epemg` | zero (§K6.4) | `0.000e0` (4 configs) | `0.000e0` | confirmed |
| `ddx_to_epemg` | zero (§K6.4) | `0.000e0` (4 configs) | `0.000e0` | confirmed |
| `gu_to_epemu` | **nonzero** | `9.961e-1` (4 configs) | `9.961e-1` | confirmed mover |
| `gux_to_epemux` | **nonzero** | `9.961e-1` (4 configs) | `9.961e-1` | confirmed mover |
| `pp_to_jj` | zero (2→2) | within-group `0.000000e0` | `0.000000e0` | confirmed |
| `pp_to_llj_dyn` | nonzero | within-group `8.274400e0` | `8.274400e0` | confirmed mover |

**Correction to §B.2's row list.** Nine of the eighteen rows it names as
"clustered" compile **no per-event prescription at all**. A fixed-energy run whose
matrix element does not move with `αs` has no consumer for either scale — there is
no parton density on the beams — so `use_running_coupling` sets `scales = None`
(`hadronic.rs:992-995`) and returns before `compile_scale_source`. `event_scales`
then answers `None`, the clustering is never built, and no configuration rule can
reach the row's σ. That is a stronger statement than zero spread and it covers all
three rows §B.2 marked genuinely unknown. The criterion §B.2 derived the list from
— `dynamical_scale_choice = -1` with the `fixed_*_scale` booleans off — is a
property of the card; whether the prescription is *compiled* is a property of the
card **and** the matrix element, and only the second is what decides whether a row
can move.

So the pre-registered **may-move** set is exactly three rows — `gu_to_epemu`,
`gux_to_epemux`, `pp_to_llj_dyn` — and every other `integrals` row is
pre-registered **must be bit-identical**, on two distinct grounds: nine because no
prescription exists, six because every configuration returns one scale, and the
five fully-fixed rows of §B.2 because they resolve to `Constant`.

**`ee_to_mumua` does not collide with chain D.** Its cluster scale is not
configuration-dependent; it has no cluster scale. Chain B cannot move the row
chain D is measuring the drift on, and the two chains are independent.

**`pp_to_jj`'s across-group `5.0e-7`.** Its within-group spread is exactly zero on
all eight groups, but the spread over *every* `(group, configuration)` pair is
`4.999999e-7` rather than zero. The groups agree on the scale to seven digits and
not beyond. Nothing in this chain moves the group, so the row stays pre-registered
bit-identical; the number is recorded because it is the size of a group-axis
effect that a §B.4-style change would expose, and it is far too large to be
rounding of a `2 → 2` scale that ought to be identical.

#### B-0.2 The `pp_to_llj_dyn` group-vs-configuration split

The measurement §B.4 makes B-2 conditional on, taken at the same 64 points:

```
pp_to_llj_dyn: 6 groups, configs [4, 4, 4, 4, 4, 4] over 64 points |
  within-group worst spread mu_R 8.274400e0 mu_F1 8.274400e0 mu_F2 8.274400e0 |
  across-group worst spread mu_R 8.274400e0 mu_F1 8.274400e0 mu_F2 8.274400e0
pp_to_llj_dyn: per-group worst mu_R spread
  g0 8.274400e0 | g1 8.274400e0 | g2 8.274400e0 | g3 8.274400e0 | g4 0.000000e0 | g5 0.000000e0
```

The two numbers are equal to every printed digit, so **the group axis reaches no
scale the configuration axis inside the sampled group does not already reach**.
The six groups split four-and-two exactly as the partonic rows do: the four with a
gluon on a beam carry the whole spread and agree with each other to seven digits
(their forests are the same topology at different flavours), the two annihilation
groups carry none — the hadronic image of `gu_to_epemu` at `9.961e-1` against
`uux_to_epemg` at `0.000e0`.

This is a statement about *range* at 64 points, not about σ. It says a
configuration draw is not structurally short of reach on this row; it does not say
the residual will collapse. The rest of §B.4's gate — `probe_llj_dyn_budget_ladder`
re-run under the draw — is B-1c's and is not answered here.

#### B-0.3 The conjunction gate, per targeted row

`sde_strategy` and `tmin_for_channel` read out of each run's own
`Cards/run_card.dat`. No card in the set sets `tmin_for_channel`, so every one
takes the default `-1.0` (`validation/madgraph/runcard_defaults.json:243`,
transcribed at `vibegraph-lib/src/runcard.rs:596`).

| row | `sde_strategy` | line | `tmin_for_channel` | `∝ AMP2_c` applies |
|---|---|---|---|---|
| `ddx_to_epemg` | 1 | `:80` | absent → `-1.0` | yes |
| `ee_to_ee` | 1 | `:86` | absent → `-1.0` | yes |
| `ee_to_mumu` | 1 | `:86` | absent → `-1.0` | yes |
| `ee_to_mumua` | 1 | `:86` | absent → `-1.0` | yes |
| `ee_to_mumu_tata_qcd0` | **2** | `:86` | absent → `-1.0` | **no** |
| `ee_to_tatah` | 1 | `:86` | absent → `-1.0` | yes |
| `ee_to_ttx` | 1 | `:86` | absent → `-1.0` | yes |
| `ee_to_wpwm` | 1 | `:86` | absent → `-1.0` | yes |
| `ee_to_zh` | 1 | `:86` | absent → `-1.0` | yes |
| `gg_to_gg` | 1 | `:80` | absent → `-1.0` | yes |
| `gg_to_ttx` | 1 | `:80` | absent → `-1.0` | yes |
| `gu_to_epemu` | 1 | `:82` | absent → `-1.0` | yes |
| `gux_to_epemux` | 1 | `:82` | absent → `-1.0` | yes |
| `pp_to_jj` | 1 | `:80` | absent → `-1.0` | yes |
| `pp_to_llj_dyn` | 1 | `:81` | absent → `-1.0` | yes |
| `uux_to_epemg` | 1 | `:80` | absent → `-1.0` | yes |
| `uux_to_mumu` | 1 | `:80` | absent → `-1.0` | yes |
| `uux_to_uux` | 1 | `:80` | absent → `-1.0` | yes |

§B.1's exception is confirmed and is now inert: `ee_to_mumu_tata_qcd0` is the one
card at `sde_strategy = 2`, and it is also a row that compiles no prescription. The
`sde_strategy = 2` branch therefore has **no gated row that exercises it**, which
makes §B.1's falsifier (a card-level test asserting the prescription reports it is
not drawing) the only thing that will ever cover it. `ud_to_epemud_qcd0` also
carries `sde_strategy = 2` (`:82`) and is one of the five fully-fixed rows.

#### B-0.4 The known-wrong informational comparison, re-measured

`probe_channel_partition_moves_sigma`, on this machine, in this worktree:

```
uux_to_epemg:  adapted alpha 5.567836e-1 ± 6.37e-4 | uniform alpha 5.573665e-1 ± 6.55e-4 | partition gap +1.047e-3 (Monte-Carlo 1.6e-3)
ddx_to_epemg:  adapted alpha 6.206199e-1 ± 6.28e-4 | uniform alpha 6.217727e-1 ± 7.19e-4 | partition gap +1.857e-3 (Monte-Carlo 1.5e-3)
gu_to_epemu:   adapted alpha 1.098695e-1 ± 1.15e-4 | uniform alpha 1.082390e-1 ± 1.35e-4 | partition gap -1.484e-2 (Monte-Carlo 1.6e-3)
gux_to_epemux: adapted alpha 1.099007e-1 ± 1.15e-4 | uniform alpha 1.082211e-1 ± 1.35e-4 | partition gap -1.528e-2 (Monte-Carlo 1.6e-3)
```

against §B.7's pre-registered `+1.05e-3`, `+1.86e-3`, `−1.48e-2`, `−1.53e-2`. All
four reproduce, so the environment is the one the bank was taken in and the probe
is live as this chain's end-to-end signal.

**What the census cannot see.** The points are cut-passing draws from one channel's
map, so a configuration dependence confined to a region those points miss reads
zero here; and a spread of zero over configurations does not bound the effect of
the *mirror* ordering, which `shape` evaluates at the same scale as the direct one
(§B.11's last risk). Neither is what the pre-registration is used for — it
predicts which cells may differ, and a cell that moves against it is
stop-and-report either way.

### Chain B results (2026-08-03)

Implementation, in four commits on `val4-b`: `9f7a1ea` (B-0 census), `33f561a`
(B-1a plumbing), `a6384cc` (B-1b the draw), `825ef31` (B-1c tolerances), plus
`a51c258` (B-3 rider) and `0440769` (B-4 artifact guard) from a separate session.

#### The census, and what it corrected

The B-0 output above records the measurement. Its one structural correction to
§B.2 is worth repeating here because every later stage rests on it: **nine of the
eighteen rows §B.2 lists as "clustered" compile no per-event prescription at
all.** A fixed-energy row whose matrix element does not move with `αs` has no
consumer for either scale — no parton density on the beams — so
`use_running_coupling` returns before `compile_scale_source` and `event_scales`
answers `None`. §B.2 derived its list from a property of the *card*; whether the
prescription is compiled is a property of the card **and** the matrix element,
and only the second decides whether a row can move. That covers all three rows
§B.2 marked genuinely unknown, `ee_to_mumua` among them, which is why chain B
never collided with chain D.

The pre-registered may-move set was therefore exactly three rows:
`gu_to_epemu`, `gux_to_epemux`, `pp_to_llj_dyn`.

#### B-1a — the negative control held

`pixi run --skip-deps validate` with the trailing uniform drawn, carried in
`AcceptedPoint.u`, and read by nothing: **byte-identical** to
`/tmp/valreport-baseline-B` over all 23 `integrals` and 22 `samples` cells, same
tree digest `4bc825da…`, 36 suites ok. The zero-bits invariant of §B.3 is not an
argument in this note; it is a measurement.

#### B-1b — the escalation diff landed exactly on the pre-registered set

The `diff -r` against the baseline differed in **three `integrals` cells and two
`samples` cells, and no others**: `gu_to_epemu`, `gux_to_epemux`,
`pp_to_llj_dyn` integrals, and the two fixed-beam movers' samples cells (which
regenerate events at the new scale). Every zero-spread clustered row and every
fully-fixed row came out byte-identical, `pp_to_jj` included — a row with a
*live* draw whose within-group spread is exactly zero, which is the sharpest
confirmation the census got.

**The informational comparison collapsed, which is the chain's own claim.**
`probe_channel_partition_moves_sigma`, converged `αⱼ` against uniform `αⱼ`:

| row | gap before | gap after | Monte Carlo |
|---|---|---|---|
| `uux_to_epemg` | `+1.047e-3` | `+1.047e-3` (bit-identical) | `1.6e-3` |
| `ddx_to_epemg` | `+1.857e-3` | `+1.857e-3` (bit-identical) | `1.5e-3` |
| `gu_to_epemu` | `−1.484e-2` | `+1.867e-3` | `1.6e-3` |
| `gux_to_epemux` | `−1.528e-2` | `+1.493e-3` | `1.6e-3` |

All four rows are now indistinguishable from their own noise, and the two whose
scale no configuration moves reproduced their old numbers *bit for bit* — the
census predicting which rows could not move, and being right at the last bit.

Against MadGraph:

| row | rel before | rel after | reference's own error |
|---|---|---|---|
| `gu_to_epemu` | `+1.076e-2` (pull `+5.21`) | `+3.98e-5` (pull `+0.02`) | `0.18%` |
| `gux_to_epemux` | `+9.75e-3` (pull `+4.32`) | `−1.10e-3` (pull `−0.49`) | `0.20%` |
| `pp_to_llj_dyn` | `−6.82e-3` (pull `−2.05`) | `−7.08e-5` (pull `−0.02`) | `0.33%` |

The hadronic row moved from `−0.68%` to `−0.01%` **without the flavour-group
axis being touched at all**, which is what §B.4's gate asked about and what
retires B-2 below.

#### B-1c — the tolerances, and the rule they were set by

§B.8's rule was fixed before the measurement and is what the numbers were read
against, not the other way round.

`probe_llj_parton_seed_stability`, five seeds at the gate budget and at four
times it:

| row | 1× mean / worst `\|rel\|` | 4× mean / worst | pulls | `χ²/dof` |
|---|---|---|---|---|
| `gu_to_epemu` | `+1.57e-4` / `1.35e-3` | `+7.53e-4` / `1.53e-3` | `≤ 0.65` | `0.58`–`1.25` |
| `gux_to_epemux` | `−8.69e-4` / `1.51e-3` | `−2.19e-4` / `1.20e-3` | `≤ 0.67` | `0.71`–`1.74` |

`probe_llj_dyn_budget_ladder`, five seeds a rung, against MadGraph's
`415.42 ± 1.36`:

| `neval` | σ (pb) | `rel` | pull | `χ²/dof` |
|---|---|---|---|---|
| `75 000` | `412.5969 ± 0.3617` | `−0.68%` | `−2.00` | `6.38` |
| `150 000` | `414.2659 ± 0.2494` | `−0.28%` | `−0.83` | `0.82` |
| `300 000` | `415.2694 ± 0.1733` | `−0.04%` | `−0.11` | `0.65` |
| `600 000` | `415.7450 ± 0.1223` | `+0.08%` | `+0.24` | `0.30` |

Increments `+1.67`, `+1.00`, `+0.48` — halving — and the row crosses the
reference between the last two rungs. Before the draw the same ladder read
`409.55`, `411.39`, `412.53`, `412.95`, asymptoting `0.6%` low.

So all three rows' residuals are Monte Carlo, and the decisions follow:

* `gu_to_epemu` and `gux_to_epemux` **leave `PULL_REPORTED_NOT_ASSERTED`**,
  which is now empty, and go `rel_tol` `0.02` → `0.005`.
* `LLJ_DYN_MAX_REL` `0.015` → `0.005`, and that row's pull is asserted too.

Each bound is the larger of the reference's own error with headroom (`0.18%`,
`0.20%`, `0.33%`) and the measured five-seed spread (`0.15%`, `0.15%`, `0.18%`).
None is fitted to the achieved central value, which on `gu_to_epemu` is `4e-5` —
a bound fitted there would be absurd, and saying so is the point of the rule.

#### The rulings

* **B-2 is out of the sprint, by measurement.** The group-vs-configuration split
  on `pp_to_llj_dyn` is zero to every printed digit (`8.274400e0` within-group
  against `8.274400e0` across-group), so the group axis reaches no scale the
  configuration axis inside the sampled group does not. The σ measurement agrees:
  the row closed to `−0.01%` with the group left on the sampler.
* **`SDE_strategy` becomes a consumed field.** It decides whether the draw runs,
  so the old `IgnoredBenign` claim that a reference cross section is invariant
  under it was false. Its reason string records what the crate does at `2`:
  keeps the sampling channel, which is a partition choice of ours and not
  MadEvent's.
* **`tmin_for_channel` stays `IgnoredPhysics`, against the ruling, and the
  reasoning is the point.** A card that sets it does not parse — the audit
  refuses it. Reclassifying it as consumed would remove that refusal and leave
  such a card integrating silently under a rule that does not describe it, since
  this crate implements no `get_channel_cut` product. A hard error is strictly
  stronger than a silent fallback, and converting one into the other is the
  failure class this sprint exists to close. The field is still *read* by
  `EventScaleSource::draws_configuration`; the parser refusal is simply what
  stands between such a card and a cross section, and
  `the_configuration_draw_needs_both_run_card_fields` asserts both guards.

#### B-3 and B-4

`pp_to_llj_dyn`'s `samples` cell is banked and gated — min KS `p 1.496e-2`, min
χ² `p 3.691e-1` over three seeds — taking the census to 90 measured, 88 ✅, 2 ⚠️.
The artifact guard is `FORMAT_VERSION 7` with a refusal targeted at a pre-v7
artifact replayed on a clustering-scale card. One defect surfaced en route and is
worth recording: the v3/v4/v5 upgrade impls normalised `format_version` to
`FORMAT_VERSION` on read, which would have made every upgraded artifact look
current and blinded the guard to exactly the artifacts it exists to catch. They
now preserve the recorded origin version, and the three round-trip tests assert
it.

#### §B.10's acceptance tests, mapped to what exists

None of §B.10's names exist in the tree. What delivers each, by real name:

| § | design's name | delivered by | status |
|---|---|---|---|
| 1 | `the_scale_channel_is_drawn_from_amp2_and_not_from_the_sampler` | `probe_the_scale_draw_reads_the_point_and_not_the_sampler` (`vibegraph-lib/tests/validate_sigma.rs`, `--ignored`) | **half**: the independence-from-the-sampler half is asserted on `gu_to_epemu` over 64 draws × 3 channels, with a non-vacuity guard that the sweep reaches more than one scale. The `∝ AMP2_c/Σ AMP2` frequency half is **not** delivered |
| 2 | `a_scale_draw_replays_identically_on_reconstruction` | same probe, third property | **yes, in observable form**: the scales at fixed `(momenta, channel, u)` are unchanged after 256 intervening evaluations that move the bound coupling. The Fact-3 defect the design names — a counter advanced per integrand call — would fail exactly this assertion. The design's literal form (an `Unweighter` trial loop with interleaved rejections) is not built; the remaining half of it, that an accepted point's recorded scale is the one its weight was taken at, is true by construction now that `event_scales_at` is a pure function of `(momenta, channel, u)` and `AcceptedPoint.u` carries the trailing coordinate |
| 3 | `the_amp2_config_order_matches_the_forest_config_order` | `the_amp2_configuration_order_matches_the_forest_order` (`vibegraph-lib/src/hadronic.rs`, banked unit test) | **yes**, on `g g → g g` where four diagrams give three configurations, so an off-by-one cannot hide |
| 4 | `the_scale_draw_is_independent_of_the_bound_coupling` | same probe, third property | **yes, in observable form**. The design's literal form — build `AMP2` at two different `αs` — is not reachable from outside the crate, the pin being internal; its observable consequence is that evaluation history does not move the drawn configuration, which is what is asserted |
| 5 | `a_non_default_sde_strategy_keeps_the_sampler_channel` | `the_configuration_draw_needs_both_run_card_fields` (`vibegraph-lib/src/hadronic.rs`, banked unit test) | **yes**, and it additionally pins that a card setting `tmin_for_channel` is refused upstream |
| 6 | `the_clustered_rows_that_cannot_move_are_bit_identical` | — | **no**. The measurement exists as `probe_cluster_scale_spread_over_configurations` (both `validate_sigma.rs` and `validate_hadronic.rs`, `--ignored`) but it *reports* the spreads and asserts nothing about the rows that read zero. Promoting it means asserting `spread == 0.0` on the named rows; what keeps it out of the banked layer as written is that it builds sixteen fixed-beam integrands |
| 7 | the rider's cell and the artifact refusal | `a51c258`, `0440769` | **yes** |

The two gaps are stated rather than papered over. Item 1's frequency half is
blocked on observability: no public API reports *which* configuration was drawn,
only the scale it implies, and two configurations may share a scale — so the
frequency law cannot be measured from outside. Exposing the drawn configuration
would unblock item 1's second half and item 6 at once, and is the natural next
increment.

#### Why the draw reproduces MadEvent even though MadEvent does not draw

Recorded because the design never derived it, and a future reader who opens
`cluster.f` first will otherwise conclude this chain inverted MadGraph's rule.

**MadEvent clusters in the sampler's channel.** `genps.f:221` and `genps.f:245`
both set `this_config = iconfig` — the configuration currently being sampled —
and `cluster.f:663-664` roots the clustering on it (`igraphs(1) = this_config`).
So per *event*, MadEvent's cluster scale is read in the channel that drew the
point, which is exactly the rule this chain replaced.

**The resolution is that MadEvent's channel is not distributed the way ours is.**
Under single-diagram enhancement the integrand of configuration `c` carries the
factor `AMP2_c / XTOT` (`matrix1_orig.f:291-317`), and a point sampled from
configuration `c` carries the map density `g_c`. The density of *events*
generated in configuration `c` at momentum `p` is therefore
`∝ |M(p)|² · (AMP2_c(p)/XTOT(p)) · g_c(p) / g_c(p)` — the sampling density
cancels against the multichannel weight — so the conditional distribution of the
configuration given the point is

```text
P(c | p) = AMP2_c(p) / Σ_i AMP2_i(p)
```

and it does not depend on `g_c` at all. That conditional is a property of the
squared amplitudes, not of the partition, which is why drawing it directly
reproduces MadEvent's per-event scale distribution *despite* this crate's
Kleiss–Pittau `αⱼ` partition being a different one from MadEvent's. It is also
why the two rows whose configurations agree on the scale came out bit-identical:
where `AMP2` moves no scale, the conditional is irrelevant.

This is the chain's justification in one line: **we do not imitate MadEvent's
channel; we sample the conditional its channel induces**, which is the same
distribution and is independent of the partition neither side shares.

#### A drifted probe reading, and why it is not a finding

`probe_cluster_scale_spread_over_configurations` now reports `9.962e-1` on
`gu_to_epemu` and `gux_to_epemux` where B-0's table above recorded `9.961e-1`.
The probe draws its own points from `channel_grid_ndim()`-many uniforms; adding
the trailing scale-draw coordinate changed `point_ndim()` and so shifted the
probe's own point sequence, which moves the worst-over-64-points maximum by one
part in `10⁴`. It is a different set of points, not a different scale. The
inertness of B-1a was established by the report diff being byte-identical, not
by this probe, and nothing downstream reads its value.

#### Open items

* **`pp_to_jj`'s across-group spread is `4.999999e-7`**, against a within-group
  spread of exactly zero on all eight groups. Nothing in this chain moves the
  group, so the row is bit-identical and stays so; but two groups of a `2 → 2`
  agreeing on the scale only to seven digits is larger than rounding and is the
  size of the effect a §B.4-style change would expose.
* **Rows with no prescription record a run-card `SCALUP` and `AQCDUP = 0`**
  (`vibegraph-cli/src/generate.rs`, the `None` arm), while MadGraph's own
  `ee_to_mumua` events carry a clustered, channel-dependent `SCALUP` — 370 of
  10 000 in a channel other than the first, which `validate_scales` replays
  correctly. The σ is right, because nothing reads the scale; the *records* are
  not MadGraph's. No `samples` cell compares `SCALUP`, so no gate sees it.
* **The draw is noisier at low budget**, as §B.11 predicted. `pp_to_llj_dyn`'s
  `75k` rung scatters at `χ²/dof 6.38` over five seeds where the pre-draw ladder
  never exceeded `1.90`. It falls to `0.82`, `0.65`, `0.30` on the next three
  rungs, so it is that rung being under-budget rather than a property of the
  estimator — but a row gated at a budget near `75k` would feel it.
* **`scale_draw_fallbacks()` is observable but unobserved.** Both integrands
  count the points whose `AMP2` carried no probability; nothing reads the
  counter. It is expected to be zero on any run that produces anything.
* **The mirror ordering is still a third axis.** `shape` draws one configuration
  per point from the direct ordering's momenta and applies the resulting scale to
  both orderings. §B.11 names the falsifier — σ must not move outside Monte Carlo
  when the draw is formed from the luminosity-weighted combination — and it is
  not measured here.

## §G re-bank — phase 1: extraction and the candidate bundle (2026-08-03)

Phase 1 turns the four completed re-bank runs into banked references and a
**candidate** `refdata-5`. Nothing here flips the pin, retires a run, or
publishes: `[refdata]` still names `refdata-4` at
`c8ef939ec6336fe53015115b7c3194604b1bd2f7cc6b52b5d21be69a82a325e9`, and that
archive is still on disk at that digest.

### G.1 What the four runs are

Read from each run's own banner, which is the seed record — the run card's
`iseed` line is not, MadGraph resets it.

| run | `pdlabel` | `lhaid` | `iseed` | `nevents` |
|---|---|---|---|---|
| `pp_to_bb` | `lhapdf` | 247000 | 21 | 10000 |
| `pp_to_bb_qcd2` | `lhapdf` | 247000 | 21 | 10000 |
| `pp_to_llj` | `lhapdf` | 247000 | 21 | 10000 |
| `pp_to_ll_scalefact2` | `lhapdf` | 247000 | 21 | 10000 |

Each event file's `<init>` carries `PDFSUP1 = PDFSUP2 = 247000`, so the number
is what the run convolved and not only what the card asked for. **`lhaid =
247000` is `NNPDF23_lo_as_0130_qed`** (`SetIndex: 247000` in the fetched set's
`.info`), the set the Drell-Yan references already use — not NNPDF3.1, whose
index is 315200.

### G.2 The cross sections move, and by how much

`refdata-4`'s banners against the re-banks', both as MadGraph's own
`Integrated weight (pb)`:

| run | `nn23lo1` | `lhaid 247000` | shift |
|---|---|---|---|
| `pp_to_bb` | 417 202 400.0 | 376 243 100.0 | −9.8% |
| `pp_to_bb_qcd2` | 417 208 048.74 | 376 246 848.68 | −9.8% |
| `pp_to_llj` | 503.5553 | 504.6288 | +0.21% |
| `pp_to_ll_scalefact2` | 1958.82 | 1958.95 | +0.0066% |

MadGraph's internal `nn23lo1` and LHAPDF's `NNPDF23_lo_as_0130_qed` are not the
same parton densities, and the two `b b~` rows say so at 10%. Any comparison
that quotes a σ across the re-bank boundary is comparing two different beams,
which is the same rule `refdata-2` → `refdata-3` earned for α_s.

### G.3 The extraction, and what it did not touch

The only reference generators that read these four runs are
`extract_diagrams.py` and `gen_kt_cluster_dumps.sh`; `gen_amplitude*.py` never
listed them, and `extract_sigma.py` skips `lpp = 1` runs by construction.
Re-running the extraction over the new runs leaves both committed references
**bit-identical** — `diagrams.json` at
`af8829cc2ee0f5e8a2637e48cc6fa2cbef2fe9778b4cf46c4df298db93eaac30` and
`sigma_reference.json` at
`fe56b3979d2b98d3f50fc621c10530412e5edcb4ec00630cf9b4f8b70f83664b` before and
after. Diagram content and fixed-energy partonic cross sections do not depend
on the proton's parton densities, so this is the expected reading rather than a
null result: it is what says the re-bank changed the beams and nothing else.

### G.4 The kT dumps

`pp_to_llj` and `pp_to_bb_qcd2` are the two re-banked runs with clustering
dumps; `pp_to_bb` has never had one. Both were replayed against the new runs
through the instrumented MadGraph at the banked seed, and both cleared the
replay's hard precondition — **event text byte-identical over 10000 events** —
and its scale gate: every event's dumped μ_R / μ_F reproduce that event's own
`SCALUP`, `<rscale>` and `<pdfrwt>`. The controls the two runs exist for
survive the re-bank: `pp_to_bb_qcd2` still drives `mt2last_override` and both
`jcentral` overrides on 141 of its 10000 events, and `pp_to_llj` still drives
`RESTRICTED_RECLUSTER` on 1857.

The re-cut dumps sit at `output/ktdump/dumps-rebank/` and their entries in
`validation/madgraph/kt_cluster_dump_manifest.rebank-candidate.json`, beside
rather than over the live ones, so the committed manifest keeps pinning the
dumps that belong to the pinned bundle. The other six runs' entries carry over
with identical digests.

### G.5 The candidate bundle

Assembled twice from this work area through `assemble_bundle.sh` itself. The
archive's bytes do not encode its own file name, so an archive assembled under
the pinned name and renamed is exactly what a manifest flipped to `refdata-5`
would produce.

| | |
|---|---|
| path | `validation/madgraph/output/bundle-candidate/vibegraph-refdata-5-candidate.tar.zst` |
| members | 2505 files — **the same member list as `refdata-4`, name for name** |
| size | 117 974 066 bytes (`refdata-4`: 118 015 652) |
| sha256 | `9f639d4ed651e66a7d7f7c9b863bed9e66059c43cefb213e803d8c8c072d356e` |
| two assemblies | **byte-identical** (`cmp` clean) |

An identical member list is the check that §4's "replacement, not addition" was
carried out at the file level: the four runs' contents changed and nothing
entered or left the bundle. `refdata-4` re-hashes to its pinned digest after
the assembly, so the pin is untouched.

### G.6 The gate does not pass, and that is the guard working

`pixi run --skip-deps -e madgraph validate` exits 101 in this worktree.
`--no-fail-fast` over the whole workspace narrows the blast radius to **one
target and two tests**: `validate_alphas`'s

    pp_to_bb: alpha_s source arm disagrees with GRID_ALPHA_S_RUNS
      left: true   right: false

`resolve()` asserts *both ways* that a run's α_s source arm matches
`GRID_ALPHA_S_RUNS`, precisely so a run that changed source shows up as a
failure rather than as a quiet reclassification. `pdlabel = lhapdf` makes
MadGraph link the grid's α_s, so all four re-banked runs move into that class
and the list — four fixed-scale names, unchanged since `main`'s 4deb883 — no
longer describes them.

The classification list is the same at `2322357`, so **the re-bank commit
itself left the gate red**; the failure is not the merge's and not phase 1's
(removing phase 1's only artifact reproduces it verbatim). Updating
`GRID_ALPHA_S_RUNS` in `validate_alphas.rs` and `validate_scales.rs` is coverage
bookkeeping that belongs with the pin flip.

`validate_scales` meanwhile **passes** on all four, which is the substantive
result underneath the red cell: it resolves the α_s arm from the card at
runtime rather than from the list, and the four re-banks replay field for field
with zero declared misses —

- `pp_to_bb`: 10000 events, 50000 scale comparisons, worst 0.999 of budget
- `pp_to_bb_qcd2`: 10000 events, 50000 comparisons, worst 0.999, 141 events
  needing a channel other than the first
- `pp_to_ll_scalefact2`: 10000 events, 50000 comparisons, worst 1.000
- `pp_to_llj`: 10000 events, 50000 comparisons, worst 1.000, 5540 events
  needing a channel other than the first

— and the factorisation floor is unmoved: the minimum replayed μ_F over every
banked hadronic event is 4.7002 GeV in `pp_to_bb_qcd2`, 2.35× the 2 GeV floor,
the same margin the C2 amendment records.

### G.7 What phase 2 still owes

The pin flip and publication, the four rows' manifest edits, retiring the
`nn23lo1` runs to the local retired area, `GRID_ALPHA_S_RUNS` in both test
files, moving `dumps-rebank/` over `dumps/` with its manifest, and the E6
measurements that turn the 8 ⛔ cells into recorded ones.

## Close-out

(To be written at sprint close: per-chain outcomes, census before/after,
protocol observations on the design–implement–review structure.)
