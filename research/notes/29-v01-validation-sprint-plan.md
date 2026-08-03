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

## Close-out

(To be written at sprint close: per-chain outcomes, census before/after,
protocol observations on the design–implement–review structure.)
