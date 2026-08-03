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

## Close-out

(To be written at sprint close: per-chain outcomes, census before/after,
protocol observations on the design–implement–review structure.)
