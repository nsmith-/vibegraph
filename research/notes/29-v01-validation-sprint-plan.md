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

## Close-out

(To be written at sprint close: per-chain outcomes, census before/after,
protocol observations on the design–implement–review structure.)
