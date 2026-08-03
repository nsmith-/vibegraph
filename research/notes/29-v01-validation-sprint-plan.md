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

## Close-out

(To be written at sprint close: per-chain outcomes, census before/after,
protocol observations on the design–implement–review structure.)
