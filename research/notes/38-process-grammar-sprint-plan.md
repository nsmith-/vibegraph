# 38 — `process-grammar` feature sprint plan: MadGraph LO process parity

**Status: PLANNED (scope decision 2026-09-25, user).** No session has run.

The sprint that closes the gap between MadGraph's leading-order process
language and ours, everywhere except MLM matching and NLO. Four
deliverables, as asked:

1. **A full-featured proc-card parser** into a data structure that mirrors
   MadGraph's `ProcessDefinition`, followed by **one scan** that refuses every
   feature not yet supported, before anything downstream sees the card. That
   scan is the single place where the feature backlog is kept against
   MadGraph.
2. **The s-channel restriction syntax** (`>` required, `$$` forbidden, `$`
   forbidden on-shell), or-multiparticles, and `add process` over processes
   with the same final-state multiplicity.
3. **Polarized external particles** (`w+{0}`, `z{T}`, fermion `{L}`/`{R}`).
4. **Decays**: 1→n decay processes, and decay-chain syntax
   (`p p > t t~ h h, (t > w+ b, w+ > j j), (t~ > w- b~, w- > l- vl~)`), with
   decay chains enumerated by stitching separate feyngraph enumerations
   together.

MLM matching and NLO come next, in that order, in later sprints. This sprint
does not implement either of them. It does leave room for both in the data
structures that pass along the pipeline (§3.2).

The standing rules are unchanged: new physics lands informational and is
enforced only when agreement is shown; every convention claim is pinned by a
test that would fail if it were false; amplitude disagreements go straight to
the per-diagram × per-helicity dump (note 12); samplers are gated over seed
sweeps, not a fixed-seed pull.

## 1. MadGraph semantics (read from the pinned source)

Every citation is to `research/refs/mg5amcnlo` at `b7687064`.

### 1.1 The process line

`extract_process` (`madgraph/interface/madgraph_interface.py:4822`) takes
modifiers off the line from the back, in this order:

- `@N`: the pattern is `^(.+)@\s*(\d+)\s*(.*)$`, so text *may follow*
  the process number (`p p > j j @1 QED=0` is valid).
- `[...]`: the loop / perturbation spec.
- Coupling orders.
- `/`: forbidden particles.
- `$$`: forbidden s-channels.
- `$`: forbidden on-shell s-channels.
- `> … >`: required s-channels.

When `@N` is absent, the process number is the count of
`generate`/`add process` lines in the history (`:3309`).

Legs (`:5043` onward) are resolved in this order: a multiparticle label, then
an integer PDG code (`11`, `-11`), then a model particle name. Only when
none of these matches is a leading digit read as a repeat count (`2e+`).
Legs also carry a polarization (`w+{0}`, `{T}`, `{L}`, `{R}`, `{A}`, …) and a
photon-tag flag (`!a!`), which exists for NLO.

`extract_particle_ids` (`:5591`) builds the restriction lists. For required
s-channels, the lists are **or-lists of and-lists**. Separate names are
and-ed. A plain multiparticle expands into the and-list. A `|`
or-multiparticle (`define v = z | a`) gives alternatives. Or-multiparticles
are refused everywhere except required s-channels. `> A A >`, a repeated
required particle, is an error.

### 1.2 The s-channel restrictions

`get_s_channel_id` (`madgraph/core/base_objects.py:2435`) returns the
propagator's PDG id **oriented as outgoing toward the final state**, and 0
for a t-channel line. So `p p > w- > e+ ve` has no diagrams.

- **`>` required** (`madgraph/core/diagram_generation.py:715`): a pure
  diagram filter. A diagram is kept if every id in any one of the and-lists
  occurs among its s-channel propagators. There is no phase-space component.
- **`$$` forbidden** (`:742`): a pure diagram filter. A diagram is dropped if
  any s-channel propagator is forbidden. For one initial particle there is a
  separate path that allows the decaying particle's own first s-channel
  (`:754`).
- **`$` forbidden on-shell** (`:781`): keeps every diagram and marks the
  propagator `onshell = False`. MadGraph acts on the mark twice (corrected by
  S3; the first reading here had only the second):
  - *in the amplitude*: `helas_call_writers.py:1184` gives a marked
    propagator ALOHA's `P1D` form, which multiplies it by
    `THETA_FUNCTIONR((p² − (M − c·Γ)²)(p² − (M + c·Γ)²) ≥ 0)`, `c = bwcutoff`,
    `Γ = fk_W = max(|W|, |M·small_width_treatment|)` (`export_v4.py:4820`).
    The line is zero inside |m − M| < `bwcutoff`·Γ whatever Γ/M;
  - *in the phase space*: `export_v4.py:5879` writes `gForceBW = 2`, and
    MadEvent's `cut_bw` (`Template/LO/SubProcesses/myamp.f:136`) rejects the
    point in integration channels whose configuration carries the marked line
    on the same window, when `sde_strat == 1` and Γ/M < 0.1. A `$` in the
    process line forces `sde_strategy = 1` in the generated card
    (`madgraph/various/banner.py:5055`).

  The rejection moves nothing: the rejected configuration's diagram carries
  the zeroed line, so its enhancement share `AMP2_c/Σ AMP2` is already zero
  there. What MadEvent integrates is

  F(x) = |M'(x)|², M' = M with every marked line zeroed on its window,

  which keeps the interference among the surviving diagrams. MadGraph 3.7.1
  mis-generates one form: a fermion `P1D` built from its second spinor slot
  (`FFV2P1D_1`) has the theta argument `(p² + (M − cΓ)²)(p² + (M + cΓ)²)`, the
  P flip applied inside the square, so that line is never zeroed (see S3
  Landed). The first reading,
  |M|²·(1 − Σ_{c ∋ forbidden} w_c 1_W), reweights the whole |M|² by the
  surviving configurations' `AMP2` share instead; on `e+ e- > mu+ mu- $ z` at
  100 GeV it integrates 0.93% below MadEvent (35 σ), and |M'|² agrees.

- **`$` and a decay chain are complements.** Writing W for the on-shell
  window:
  - `p p > z, z > l+ l-` gives ∫_W |M_Z|².
  - `p p > l+ l- $ z` gives ∫_outside W |M|² + ∫_W |M_γ|².

  Together they reproduce `p p > l+ l-` except for the γ–Z interference
  inside the window. `p p > z > l+ l-` is **not** a complement of `$ z`: it has
  no window, so it double-counts the off-shell Z tail.

### 1.3 Decays and decay chains

`extract_decay_chain_process` (`madgraph_interface.py:5661`) recurses:

- `,` starts a decay at the current level.
- `(` goes down one level; a parenthesised group that contains no `,` has its
  parentheses dropped.
- An `@N` followed by `ORDER=n` sets overall orders for the whole chain.
- `[...]` and squared orders are refused with decay chains (`:3277`).

`DecayChainAmplitude` (`madgraph/core/diagram_generation.py:1337`) generates
the core and each decay as separate amplitudes. A decay must have exactly one
initial particle. The decaying legs of the core are flagged
`onshell = True`, which becomes `gForceBW = 1`: the point is cut outside the
`bwcutoff` window (`myamp.f:179`) and sampled with a Breit–Wigner (BW) there.

A decay whose particle does not appear in the core is **discarded with a
warning** (`:1405`). We refuse it instead.

How a single decay spec is assigned to several identical legs of the core
(`p p > z z, z > e+ e-` against `…, z > e+ e-, z > mu+ mu-`) is decided in
`helas_objects`'s decay-chain combination. G1 pins it with a MadGraph dump
rather than from memory.

1→n decay processes (`generate t > w+ b`) are ordinary processes with
`ninitial = 1`. MadEvent integrates them to a partial width. A card cannot
mix processes with different numbers of initial particles (`:3317`).

### 1.4 Proc-card commands

- `generate` calls `clean_process()` (`:4795`), which **discards every
  earlier process**, and then appends.
- `add process` appends.
- A duplicate amplitude raises `Duplicate process … found` (`:3375`) unless
  the line carries `--no_warning=duplicate`. The check compares whole
  `Amplitude` objects, which carry the process number. Whether
  `generate p p > e+ e-` plus `add process u u~ > e+ e-` trips it is measured
  in G1, not assumed.
- `set` commands can change the physics (`complex_mass_scheme`,
  `group_subprocesses`, `ignore_six_quark_processes`, …).

## 2. Today's surface: what the audit found

Audited 2026-09-25 against `diagrams/parse.rs`, `alias.rs`, `selector.rs` and
`mod.rs`. Each row is one of three kinds: **silent** (a wrong answer), a hard
error (correct under the scope rule), or wrong-but-loud.

| Surface | Today | Kind |
|---|---|---|
| `> X >` required s-channel | parsed, expanded, dropped in `build_selector` | **silent** |
| `$ X` | accepted by default (`ParsingOptions`), dropped | **silent** |
| `$$ X` | refused by default; dropped if a caller opts in | silent behind a flag |
| `[QCD]` loop spec | accepted by default, dropped | **silent** |
| `set …` lines | skipped | **silent** (for the physics-bearing ones) |
| second `generate` line | appends instead of resetting | **silent** |
| same subprocess from two process lines | duplicates removed only within a line (`mod.rs:356`) | **silent** double count |
| PDG-code legs | `11` reads as count 1 of `1`; `21` as two legs named `1` | **silent** misparse |
| `@N` then orders | `@1` left in the final state | loud |
| malformed `add process` (< 3 tokens) | skipped | **silent** |
| `\|` or-multiparticle | unsupported; the flat name list can only mean "and" | loud (unknown name) |
| polarization `{…}`, tag `!a!` | read as part of the particle name | loud, but a confusing message |
| decay chain `,` | `DecayChainUnsupported` | hard error ✅ |
| squared orders `^2` | `DiagramError::SquaredOrder` | hard error ✅ |
| `@N` downstream | LHEF `LPRUP` is a constant 1; metadata keeps only the first process | **silent** |
| `add process`, hadronic | same slot-ordered final masses required (`proton.rs:701`) | hard error (conservative) |
| `add process`, fixed energy | identical externals required (`hadronic.rs:683`) | hard error (conservative) |
| different multiplicity | refused as "different outgoing masses" | hard error, wrong reason |

## 3. Design

### 3.1 Parse everything, check once, narrow the type

The run card already has this pattern: every recognised name is classified in
`runcard/classes.rs` as `Consumed`, `IgnoredBenign` or `IgnoredPhysics`, and
refused when it could bite. The proc card gets the same treatment:

- **`ProcCardAst`**: MadGraph's `ProcessDefinition` with nothing dropped.
  - Legs are id-sets with polarization and a tag flag.
  - Required s-channels are `Vec<Vec<Vec<Name>>>` (or of and).
  - Plus forbidden particles, forbidden s-channels and forbidden on-shell
    s-channels.
  - Amplitude orders with their operators, squared orders, and the loop spec
    (option and perturbation orders).
  - `@N` and overall orders.
  - Decay chains as child `ProcessDefinition`s.
  - The command sequence (`import model`, `define` including `|`,
    `generate`/`add process` with reset semantics, `set` with its arguments).
- **`check_supported(&ProcCardAst) -> Result<SupportedCard, Vec<Unsupported>>`**:
  - The one scan.
  - It reports every unsupported feature in the card at once, not the first.
  - `SupportedCard` is a **narrower type**: it has no field for a feature
    that is refused, so the compiler keeps anything downstream from reading
    one. Supporting a feature means moving its field from the refused set into
    `SupportedCard`, together with its gate.
- **The `Unsupported` enum is the feature backlog against MadGraph.** Each
  variant carries its reason and the backlog entry it waits on. Its doc table
  replaces the scattered "descoped" lists.

### 3.2 Room for MLM and NLO

Nothing below is implemented for MLM or NLO in this sprint. The point is that
their fields and variants exist, so the later sprints extend types rather
than re-thread them.

- **Parse, don't drop.** The loop spec, perturbation orders, squared/split
  orders, photon tags and polarizations are all in the AST. MLM's
  multi-multiplicity `add process` and NLO's `[QCD]` are `Unsupported`
  variants, not parse errors.
- **No single multiplicity at the type level.** `SupportedCard` holds a list
  of processes, each with its own external legs and `@N`. The
  equal-multiplicity rule is a `check_supported` refusal (reason: MLM),
  **not** an assumption built into the integrand. Today `ProtonIntegrand`
  reads `n_out` from `groups[0]`; the refusal moves to the check, and the
  integrand keeps its per-group `n_out`.
- **Provenance on every diagram.** The diagram container that is passed to
  `helas` records:
  - which process (`@N`) and which decay-chain node the diagram came from;
  - for each propagator, MadGraph's `onshell` flag (`None` / forced / forbidden).

  The same slot later carries MLM's clustering hints and NLO's Born/real
  bookkeeping.
- **Run card.** `ickkw`/`xqcut` stay `IgnoredPhysics` refusals.
  `cut_decays` moves from `IgnoredBenign` to `Consumed` in D3, because once
  decay chains exist it can bite.

### 3.3 The diagram container is the oracle boundary

**Decision (user):** the stitching oracle compares the **diagram container**
passed to `helas`, not complex amplitudes. Any sign-convention ambiguity that
`helas/eval` resolves today moves back to the diagrams stage, so that
container equality is sufficient.

The audit says that is not yet true. `compile_single_diagram`
(`helas/eval/root_diagram.rs:1219`) builds `fermi_sign` from:

- `Diagram.sign` (feyngraph's `view.sign()`);
- `spine_sign_from_flow`: the relative fermion sign `view.sign()` omits (the
  Bhabha, `g g > t t~` and `u d > e+ e- u d` cases, `:642`);
- `yang_mills_vvv_sign`: one −1 per VVV vertex at index `1..`, that is, it
  reads which vertex is `VtxIdx(0)` (`:1163`);
- `build_convention_sign` and `reversed_convention_sign`, both taken at the
  canonical `VtxIdx(0)` rooting.

`VtxIdx(0)` appears in 28 non-test sites. Two of these factors are properties
of the diagram, not of HELAS: the spine sign, and the dependence on vertex
numbering. A stitched diagram has a vertex 0 that no feyngraph enumeration
chose, so the sign that currently leans on feyngraph's numbering has no
defined value for it.

S1 therefore:

- defines the complete relative Fermi sign on `Diagram`, plus whatever
  canonical anchor currently hides behind `VtxIdx(0)` (hypothesis: the vertex
  that external leg 0 attaches to), at the diagrams stage;
- leaves `helas` only the signs that compensate its own kernel conventions,
  and that are functions of (diagram, rooting) and cancel under the
  rooting-soundness tests;
- gives `Diagram` a canonical form and equality up to renumbering of
  internal vertices and propagators.

The gate is that every amplitude-oracle row stays byte-identical: S1 moves
signs, it does not change any.

### 3.4 The s-channel predicate

`Prop.momentum` (the signed combination of external momenta) and
`Prop::is_spacelike` already carry everything the filters need. That
supersedes the `TODO(s-channel)` in `selector.rs`, which waits on feyngraph.

- With two initial particles, a propagator is s-channel iff it is not
  spacelike.
- Its oriented id is the particle as it flows toward the final-state side of
  the line.
- With one initial particle, every propagator is s-channel.

The filters run on `Diagram` after conversion, **inside** the automatic
lowest-WEIGHTED search, so that an order the filter leaves empty moves the
search on, as `find_optimal_process_orders` does (`:2095`).

## 4. Sessions

### G1: grammar, AST and the one check (feature-dev; first, no physics change)

- `ProcCardAst` per §3.1, covering the full §1.1/§1.3/§1.4 grammar.
- `check_supported`, the `Unsupported` table, and `SupportedCard`.
- Every **silent** row in §2 becomes either a correct parse or a hard error:
  - `>`, `$`, `$$` and `[…]` are refused until their sessions land;
  - `generate` resets;
  - cross-line duplicates are refused;
  - PDG codes resolve;
  - `@N` followed by orders parses;
  - physics-bearing `set` lines are refused, and benign ones are
    classified with an argument, as in `classes.rs`.
- **Oracle:** MadGraph's own parser. A corpus of process lines is run through
  `extract_process` / `extract_decay_chain_process` in the pixi MadGraph
  environment, and the resulting `ProcessDefinition` is dumped to JSON and
  compared field by field. The corpus includes the §1.4 duplicate question
  and the §1.3 multi-leg decay assignment.
- `docs/src/guide/03-diagrams.md` is rewritten to describe the new grammar.

**Landed** (2026-09-25; `34d6d45` grammar, check and resolution, `1f5f924`
oracle, `f67f787` guide). `diagrams::parse` (AST), `diagrams::check`
(`check_supported`, `Unsupported`, `SupportedCard`) and `diagrams::resolve`
(names against the model, and MadGraph's `ProcessDefinition` field for field).
The oracle banks 132 cards through `MasterCmd` with generation stubbed
(`validation/madgraph/dump_proc_grammar.py` → `proc_grammar.json`); the
hermetic `proc_grammar_oracle` test agrees on all 92 MadGraph reads and
refuses all 40 it refuses. Every supported card enumerates the same diagram
sets as before (51 banked scripts plus extra cards, per-subprocess diagram
hashes), with one deliberate exception: `/ X` for a charged `X` now forbids
both orientations, as MadGraph's |PDG| test does (the selector forbade one).
Measured with MadGraph's own generation:

- *Duplicates* (§1.4): `Duplicate process` fires only for identical
  amplitudes, process number included. `generate p p > e+ e-` + `add process
  u u~ > e+ e-` generates `u u~ > e+ e-` twice (also under one `@1`);
  identical lines with different (or implicit) `@N` generate twice;
  `--no_warning=duplicate` drops the second copy. G1 refuses any subprocess
  reached from two lines, flag or not.
- *Decay assignment* (§1.3, `combine_decay_chain_processes`): when the number
  of decays equals the number of decaying core legs, decays go to legs in
  order (`z z, z > e+ e-, z > mu+ mu-` → one ME, ee+μμ); otherwise every
  combination with repetition of the decays of that particle
  (`z z, z > e+ e-` → both ee; three decays over two Z → six MEs; `z > l+ l-`
  → ee/ee, ee/μμ, μμ/μμ).

Corrections to this note: MadGraph's spacing fix-up regex only ever matches
`]`, so `p p>e+ e-` is a MadGraph error (and one here); only `=`, `<=`, `==`,
`>` are valid order operators (the old parser accepted `<`, `>=`, `!=`, `===`);
`EW=n` on the SM is accepted by MadGraph and constrains nothing (refused
here); `/` forbids by |PDG|; `set` lines after `launch` are run-card edits
(refused as `LaunchDialogue`).

### S1: signs to the diagrams stage (validation-dev; before D2)

Per §3.3. Gate: every amplitude row byte-identical; `rooting_soundness`
green; the container canonical form round-trips over every banked process.

**Landed** (`93fff0f`, `2d99872`, 2026-09-25). Measured first, on a census of every
manifest row's process in its own model, 18 extra SM processes and 5 four-fermion
contact-emission processes: every diagram, every colour chain, the tree rooted at
every vertex.

- **The factors.** feyngraph's `view.sign()` is the parity of the external fermions
  paired by line (class a, a graph property). The spine sign is class a as well: it
  counts, per fermion line, its ends, its internal propagators and whether it carries
  a Dirac matrix; on the 2553 (diagram, chain) pairs of the first census it never
  varied with the rooting. It is now `Diagram::fermion_line_sign`, folded into
  `Diagram::sign` at conversion. The rooted-tree derivation is a debug-build
  cross-check on every compiled diagram, and it counts every closed line: it used to
  skip a line closed at a four-fermion current rooted at a fermion leg, which is where
  the cross-check first fired. The Yang-Mills source sign, the build sign and the
  reversed-bilinear sign of the reference rooting are class b (kernel compensations),
  and each varies with which vertex is the reference root (214, 92 and 12 of the 2553
  pairs, their product 234): that choice was feyngraph's `VtxIdx(0)`, the only class-c
  dependence. `helas/eval` now reads all three at `Diagram::anchor`. The live tree's
  reversed parity is class b and invariant.
- **feyngraph's vertex 0 is not "the vertex leg 0 attaches to".** It is in 2475 of the
  2517 census diagrams. The other 42 (`g g > g g g` 6, `g g > t t~ g` 1,
  `W+ W- > W+ W- Z NP<=1` 35) put leg 0 on a four-point contact, and feyngraph numbers
  a three-point vertex first. The sign product differs between the two choices on 53
  (diagram, chain) pairs, so the choice carries physics. `Diagram::anchor` is "the first
  lowest-arity vertex in canonical order", which reproduces it.
- **Canonical form.** `Diagram::canonical`: depth-first from leg 0's vertex, rays in slot
  order, propagators numbered and oriented by the walk. `CanonicalDiagram` is `Eq + Hash`
  on every field (sign and symmetry factor included). It is idempotent, and it keeps the
  diagrams of every census subprocess distinct.
- **Gates.** `renumbering_preserves_signs_and_amplitudes` renumbers every census diagram
  twice, with the anchor forced off index 0. It checks exact `fermi_sign` for every chain,
  the same anchor and canonical form, and per-helicity, per-flow amplitudes to 1e-10.
  Against `c7037dd`'s `helas` it fails 658 times; on `93fff0f` it passes on 2641 diagrams
  (6838 signs, 1960 amplitude comparisons). No physics change in `93fff0f`: a
  per-diagram dump of `fermi_sign` and the bits of every per-helicity, per-flow
  single-diagram amplitude is byte-identical to `c7037dd` on 2898 diagrams. Mutation:
  dropping the line sign from `Diagram::sign` fails 9 `amplitude_oracle` rows.
- **`2d99872` is a physics fix**, found by the cross-check and adjudicated against
  MadGraph standalone (5 points each, `MG/ours` against the 1/4 helicity average).
  `93fff0f` keeps the old rule, where a line closed at a fermion-output four-fermion
  current takes no line sign. That is right for the tensor contact:
  `ta+ ta- > t t~ a` (`O_leQt3`) comes out 0.25000 ± 3e-6, and the toy
  `lt~ lt > qt qt~ vt` 0.25 exactly; counting the line misses by up to 20%. It is wrong
  for the vector contact: `e+ e- > mu+ mu- a NP<=1` (`vg_c4l`) comes out 0.21–0.49
  under it and 0.25 exactly without it. `2d99872` confines the rule to tensor-path
  vertices. Every census diagram outside four-fermion contact emission is unchanged.
- **Open, not fixed (outside S1):**
  - The contact sign is wrong for all-vector contacts next to a Yang-Mills vertex.
    With the anchor at a three-point vertex, `g g > g g g` misses the exact five-gluon
    Parke–Taylor `|M|²`: the ratio varies 2.15e3–2.90e3 over six points. It also
    misses the per-flow six-gluon MHV amplitudes on 30/30 helicity configurations.
    Taking the all-vector contact's −1 only at the anchor (and none elsewhere), with
    the anchor at leg 0's vertex, gives a ratio constant to 12 digits at five gluons.
    It matches every six-gluon MHV flow to 6e-10, and leaves every `amplitude_oracle`
    row unchanged except `wpwm_to_wpwmz_cw`, which moves from 2.20e3 to 2.79e1.
  - `u u~ > t t~ g NP<=1` under `vg_c4q` misses MadGraph standalone by a varying
    factor under either line rule (`MG/ours` 0.0147–0.0277 against 1/36), so a second
    defect sits in four-quark contacts with a gluon emission.
- **Blind spots of container equality.** Slot order is part of the identity, so two
  containers that bind identical particles to a symmetric vertex's slots in a
  different order compare unequal (a false inequality, never a false equality). A
  sign convention that is a wrong *function of the graph* passes renumbering and
  equality alike; only the amplitude oracles see it. The production root's tie-break
  still reads vertex indices, so equal containers agree to rounding (≤1e-10), not
  bit-for-bit.

### S2: `>` and `$$` as diagram filters (feature-dev; after G1)

- Per §3.4, with MadGraph's warning about gauge invariance.
- Gates: diagram census against `diagrams.json` for new cards:
  - `e+ e- > z > mu+ mu-`;
  - `p p > w+ > e+ ve`, and the orientation pin `p p > w- > e+ ve` giving
    zero diagrams;
  - `p p > t t~ > w+ b w- b~`;
  - `p p > w+ b w- b~ $$ t t~`;
  - an or-multiparticle `define v = z | a`.
- Plus one σ row each for `>` and `$$`.

**Landed** (2026-09-25; `c52e4f7` filters, five-flavour labels and
census, the docs commit after it guide and notes). `diagrams::schannel` filters converted
diagrams on `Prop.momentum`: a non-spacelike line is s-channel, and its
oriented id is the particle when the line's energy, evaluated at E_in = n_out,
E_out = n_in (so the Σp_in − Σp_out ambiguity of the representation drops
out), is positive along `endpoints[0] → endpoints[1]`, the antiparticle
otherwise. The filter runs inside the automatic WEIGHTED search, so
`u u~ > a > d d~` lands at WEIGHTED = 4 as in MadGraph. `>` and `$$` are
lifted from `Unsupported`; `WEIGHTED<=n`/`=n` is lifted too, as the same
per-diagram bound the search uses (`==`/`>` stay refused: MadGraph adds a
squared-order constraint for them). A process line with no diagram is now
`DiagramError::NoDiagrams`, MadGraph's `NoDiagramException`. On import, `p`
and `j` follow `add_default_multiparticles` (`madgraph_interface.py:6042`):
b b~ added when the model's b is massless, removed when massive, the photon
removed; replayed from the card's history so a label defined through `p`
after the import sees the rewrite.

- *Census* (`validation/madgraph/dump_schannel_census.py` →
  `schannel_census.json`, hermetic `schannel_census` test): 57 cards through
  MadGraph's own generation, 43 generated and matched subprocess for
  subprocess, count and per-diagram oriented s-channel multiset; 6 refused
  by both (`u d~ > w- > e+ ve`, `p p > w- > e+ ve`, `e+ e- > z a > mu+ mu- a`,
  `u d~ > e+ ve $$ w+`, `e+ e- > mu+ mu- WEIGHTED<=3`, one `/ j` card); 8
  one-initial cards banked for D1 and refused here as decays. Flipping the
  orientation sign fails 10 cards; disabling the five-flavour rewrite fails
  21 cards.
- *σ rows* (e+e- at 500 GeV, MadGraph's default run card, pinned MadEvent
  via `mg5_pinned.sh`, 10k events; `vibegraph integrate`, seeds 1–5):
  `e+ e- > z > mu+ mu-` MadGraph 0.05221 ± 0.000019 pb, here mean 0.052206
  (−0.007 %), pulls −0.84 … 0.53; `e+ e- > e+ e- $$ z` MadGraph
  157.5 ± 0.11 pb, here mean 157.56 (+0.04 %), pulls −0.78 … 1.66. Known-wrong
  comparisons: the unrestricted `e+ e- > mu+ mu-` there is 0.4196 pb.
- *Unchanged enumeration*: the 51 banked scripts and 23 extra cards dump
  identical per-subprocess diagram lists (Debug-string digest) at `77f9868`
  and after, except the massless-b cards, which gain exactly their b
  subprocesses. No banked validation card uses a massless-b model with `p`
  or `j`.

Corrections to this note and to G1: MadGraph emits **no** gauge-invariance
warning for `>` or `$$`; the only statement is the 1.4.3 release note on
`$$` (`UpdateNotes.txt:2149`). The warning logged here quotes it. The
SMEFTsim restrictions in reach (`massless`, `SMlimit_massless`, every
`vg_*`) keep MB = 4.18, so SMEFTsim cards stay four-flavour; only the SM's
`no_b_mass`, `no_masses` and `zeromass_ckm` switch.

For D1 (one initial particle, MadGraph banked in the census): `>` is the
same membership filter over every propagator except the decaying particle's
own line (`t > w+ > b e+ ve` keeps its one diagram, `t > w- > …` has none);
`$$` likewise ignores that line (`t > b e+ ve $$ t` keeps its diagram,
`$$ w+` empties it, `h > e+ e- mu+ mu- $$ h` keeps it, `$$ z` empties it).
Every propagator is oriented away from the decaying particle.

### D1: 1→n decay processes (feature-dev; after G1)

- `n_in = 1` through enumeration and phase space: rest-frame generation,
  flux 1/(2M).
- A width run mode, beam-free.
- The event file for decays in MadGraph's convention.
- Gates: partial widths against MadEvent for `t > w+ b`, `t > b e+ ve`,
  `h > e+ e- mu+ mu-` and `z > e+ e-`, plus one sample.
- D1's enumeration is the decay-side building block for D2.

**Landed** (2026-09-25; `596c32b` check and `enumerate_decay`, `49f3ab0`
decay run card, `11dcd0a` integrand, CLI and event file, `fa07c04` gates,
`d86c226` census dump fix, `046282e` renumbering over decays, the docs
commit after them). A 1→n line passes `check_supported`; mixing initial-state
counts stays refused. `diagrams::enumerate_decay` runs one decay's
enumeration (lowest-WEIGHTED search included) as a unit and refuses a
process without exactly one initial particle; it returns a
`Vec<DiagramSet>` since a decay with labels (`w+ > j j`) has several
assignments. `hadronic::InitialState` is `Beams(FixedBeams)` or
`Decay(DecayAtRest)`; the decay supplies `√s = M`, incoming `(M,0,0,0)`, flux
`1/(2M)`, no boost, and `Observable::PartialWidth` (GeV), and the same
`FixedBeamIntegrand` and per-diagram channels serve it (every line timelike,
so every channel is the all-timelike tree). No `helas/eval` change was
needed: helicity pruning already skips `n_in != 2`, and the evaluator took
an incoming massive fermion, vector or scalar unchanged (per-point `|M|²`
below). `hadronic.rs`, `cuts.rs`, `lhef/build.rs`, `diagram_channel.rs`
(`beam_masses` slot 1 on a decay), `runcard.rs`, `config.rs` and the two CLI
commands are the touched files outside `diagrams/`.

- *MadEvent's decay run* (pinned checkout, 3.7.1 per its `VERSION`):
  `banner.py:4784` writes a decay's default card with `remove_all_cut()`,
  `:5045` forces `sde_strategy = 1`; `setcuts.f:137` (`nincoming.eq.1`) sets
  `lpp = 0`, `ebeam = M/2`, `scale = M` and `fixed_ren_scale` unless the card
  fixes it, both `fixed_fac_scale` true. So cuts *are* applied, by `cuts.f`
  in the rest frame, except the ŝ window (`cuts.f:310`, `nincoming.eq.2`).
  Measured on `t > w+ b` with `dsqrt_q2fact = 50/60`, `scale = 70`: SCALUP
  60 = max of the fixed factorisation scales; AQCDUP αs(M_t) = 0.1076279
  unless `fixed_ren_scale` (then αs(70) = 0.1229055); `dynamical_scale_choice
  = 3` changes nothing. `RunCard::decay_default` and `RunCard::for_decay`
  transcribe this (`runcard_decay_defaults.json` pins the 107 cut resets),
  and `use_running_coupling` applies `for_decay` itself on a decay.
- *Event file* (MadEvent, `t > b e+ ve`): `<init>` `6 0 1.730000e+02
  0.000000e+00 0 0 247000 247000 -4 1`, XSECUP = width in GeV; per event the
  top status −1 at rest (pz printed 1e-14), products' mothers `1 0`
  (`unwgt.f:741`), XWGTUP = width, SCALUP 91.188, AQCDUP 0.1076279. Matched
  field by field except PDFSUP (0 here, as for every fixed-energy run) and
  MadEvent's status-2 `W` inside its BW window (no intermediate records here
  for any process; E1). `check-events` reads an empty beam 2 as a decay.
- *Two-body* (`sm_decay_widths.json`: `decays.py` via `model_reader` on
  `restrict_default`): 19 open channels of t, W+, Z, H to ≤ 5e-13 in Γ and
  1e-12 in `Σ|M|²` at four orientations. *Many-body `|M|²`*
  (`decay_amplitudes.json`, MadGraph standalone `SMATRIX`, 24 points each):
  `t > b e+ ve` 5.7e-14, `h > e+ e- mu+ mu-` 1.7e-14, `z > e+ e- mu+ mu-`
  (8 diagrams) 1.3e-12, `t > b e+ ve a` 3.3e-14.
- *Widths* (`cli_decay`, ten seeds at `--target-rel 2e-3`; MadEvent ten
  seeds of 10k events; each mean ± max(quoted, spread/√n)):

  | row | exact | MadEvent | here | pull |
  |---|---|---|---|---|
  | `t > w+ b` | 1.4914721 | 1.491506 ± 2.3e-5 | 1.491582 ± 1.6e-4 | +0.71 vs exact |
  | `z > e+ e-` | 0.08396539 | 0.08396686 ± 1.3e-6 | 0.08397159 ± 8.7e-6 | +0.71 vs exact |
  | `h > e+ e- mu+ mu-` | 2.4193128e-7 | 2.416009e-7 ± 1.5e-10 | 2.421078e-7 ± 1.4e-10 | +1.25 vs exact |
  | `t > b e+ ve` | — | 0.1632493 ± 4.4e-5 | 0.1632357 ± 4.0e-5 | −0.23 vs MG |
  | `h > 4l`, ptl 10 etal 2.5 drll 0.4 mmll 12 | — | 8.121266e-8 ± 1.0e-10 | 8.125823e-8 ± 6.1e-11 | +0.38 vs MG |
  | `t > b e+ ve`, ptb 20 etab 2.5 ptl 15 etal 2.5 drbl 0.4 | — | 0.1418069 ± 5.8e-5 | 0.1417557 ± 4.4e-5 | −0.71 vs MG |

  Our χ²/dof over seeds 0.54–1.85. The `h > e+ e- mu+ mu-` exact width is a
  quadrature over the two off-shell Z lines (`decay_semianalytic.py`, 3e-9).
  **MadEvent is the one that misses there**: 10k-event seeds −0.14% (−2.2σ of
  their mean; 24 runs over three setups all ≈ 2.416), seed scatter 1.33× the
  quoted error, three 100k-event seeds −0.05% — it converges up with budget.
  Here: thirty seeds at 120k×10 +1.25σ, five at 480k×20 −0.3σ, twelve with
  the lepton pairs swapped +0.7σ. Flat RAMBO is useless as a control (seeds
  scatter 10% against a 0.6% quote). A five-seed first cut of this row read
  +3.5σ against the 10-seed MadEvent mean: the seed set was the high tail of
  our own (thirty-seed) distribution and the reference was the low-biased one.
- *Sample* (`t > b e+ ve`, 10k unit-weight events vs MadEvent's 10k):
  m(e+ ve) χ²/dof 1.15 over 16 bins, m(b e+) 0.84 over 17, every bin within
  4σ; AQCDUP equal to MadEvent's 0.1076279 within 5e-7.
- *Unchanged 2 → n*: the binary at the S1 head (`65edb40`) and this branch
  write byte-identical artifacts and 500-event LHE files, same seeds, fixed
  budget, for `e+ e- > mu+ mu-` (45.6 GeV beams), `e+ e- > mu+ mu- ta+ ta-
  QCD=0` (250 GeV), `u u~ > g g g` (250 GeV, ptj 20, clustering scale) and
  `p p > e+ e-` (NNPDF23). The shared paths are unchanged for two incoming
  legs by construction (`point_scales_of`'s constant short-cut returns what
  the constant source returned; the ŝ window and channel forests branch on
  one incoming leg). One relaxation: `detect_unimplemented` accepts
  `remove_all_cut`'s reset values of `ktdurham`/`ptlund`/`dparameter`/
  `deltaeta`, which `cuts.f`'s `> 0` guards leave off, so a 2 → n card
  carrying them is no longer refused.

Findings for later: VEGAS refinement makes a constant integrand noisy (a
two-body width after grid refinement reads ±0.03% instead of exact — the
bins follow the first iteration's sampling noise), so a single-channel
two-body run should skip refinement; `SDE_strategy = 2` on a decay card is
refused (channel forests are 2 → n only); a user's decay card with a
proton-beam-only physics field off default is refused at parse although a
decay ignores it. The S2 census dump sliced decay diagrams as if they still
carried MadGraph's filtering-time artificial vertex, dropping the `W` of
`t > b e+ ve` and one `Z` of `h > e+ e- mu+ mu-`; fixed and regenerated
(only those five cards change).

### D2: decay-chain enumeration by stitching (feature-dev; after S1, S2, D1)

- Core and decay enumerations are glued at the resonance, which becomes an
  s-channel propagator carrying `onshell = forced`.
- Fermi sign (on `Diagram`, per S1), symmetry factor, permutations of
  identical particles across decays, and colour through coloured decays.
- **Oracle** (§3.3): container equality against the full final state,
  enumerated and then filtered to diagrams in which each chain resonance is an
  s-channel with exactly its stated daughters. The generalised S2 filter
  does this. Cases: `e+ e- > z z, z > e+ e-`; `e+ e- > t t~, t > w+ b,
  t~ > w- b~`; a nested `(t > w+ b, w+ > …)`.
- Plus the diagram census against MadGraph for decay-chain cards.

**Landed** (2026-09-25; `c933976` stitching, `337b5c3` MadGraph census, `ff290ae` merge of
`f67c8d0` (C4V, P1), the docs commit after them). `diagrams::chain` enumerates the core and each decay (recursively, each with its own
lowest-WEIGHTED search, via the `1 → n` path) and glues each decay onto the matching
final-state leg of each core diagram: the leg and the decay's incoming leg become one
propagator from the core vertex into the decay vertex, `Prop::onshell = OnShell::Forced`,
and the products replace the leg in place (`get_legs_with_decays`, `base_objects.py:3667`).
Decays go to legs as `combine_decay_chain_processes` assigns them
(`helas_objects.py:5510`–`5566`: in order when the counts match, the out-of-order branch,
else every combination with repetition, an unordered combination once). The stitched graph's
momenta (`Diagram::tree_momentum`) and sign (`Diagram::fermion_pairing_sign` × the line
sign) are rebuilt from the graph; symmetry factor is the parts' product (1 on every tree).
Then every permutation of identical final-state particles *between* blocks (core legs, each
decay's products) is applied and each graph kept once.

- *Representation* (§3.2). `Prop::onshell: OnShell {Free, Forced, Forbidden}` (in the
  canonical key); `Diagram::provenance: Provenance { process: u32 (@N), decays:
  Vec<DecayOrigin { node: ChainNode, prop: PropIdx }> }` (not in the key). `ChainNode` is the
  preorder index of the decay on the card (core 0; `(t > w+ b, w+ > e+ ve), t~ > w- b~` →
  1, 2, 3). D3 reads the `Forced` lines (mass and width from `Prop::particle`); S3 sets
  `Forbidden`; E1's mother pointers are `Diagram::final_state_side` of each forced line.
- *Check*. One check, one variant: `check_supported` still refuses `DecayChain` (reason now:
  the BW window); `check_enumerable` is the same scan without that refusal and returns an
  `EnumerableCard` whose card cannot be taken out, consumed only by
  `generate_decay_chains`. D3 lifts it by deleting the variant, the `Scope` parameter and
  `EnumerableCard`. New refusals: `Unsupported::ChainOrders` (`@N QED=2` overall orders,
  which in MadGraph also cap each part and switch off its search), and
  `DiagramError::DecayChain` for a decay whose particle is in no core final state (MadGraph
  drops it with a warning, `diagram_generation.py:1405`), a core subprocess no decay applies
  to (MadGraph keeps it undecayed), mixed post-decay multiplicities, and the ambiguous case
  below.
- *Oracle 1, container equality* (`helas::eval::stitching`): stitched against the
  undecayed final state enumerated at the stitched `WEIGHTED` order and filtered by
  `schannel::match_resonances` (s-channel, oriented id, final-state side = stated products
  recursively), matched lines flagged forced. Ten cases, 147 diagrams: `e+ e- > z z, z >
  e+ e-` 4, `…, z > e+ e-, z > mu+ mu-` 2, `e+ e- > t t~, t > w+ b, t~ > w- b~` 2, nested
  `(t > w+ b, w+ > e+ ve)` 2, `u u~ > t t~ g, t > w+ b` 5, `g b > w- t, t > w+ b` 2,
  `g g > t t~ g, t > w+ b` 16 (one 4-gluon), `u u~ > z g g g, z > e+ e-` 50 (two 4-gluon),
  `e+ e- > w+ w- z, z > mu+ mu-` 16, `p p > z j, z > l+ l-` 24 subprocesses / 48. All
  equal as containers except two of `w+ w- z`, which bind the forced `Z` and an s-channel
  `Z` to the two `Z` slots of `W W Z Z` / `H Z Z` in the other order — S1's documented false
  inequality; the test pairs them by a slot-order-free description. *Oracle 2*: all 147
  pairs' per-helicity, per-flow single-diagram amplitudes agree, worst 1.2e-16 relative
  (bit-equal except the two slot-swapped pairs), on the merged tree (C4V's anchor-read
  vector-contact signs included).
- *Mutations*: dropping the between-block permutations fails oracle 1 (`z z, z > e+ e-`: 2
  stitched, 4 filtered). A naive sign (core × decays, not rebuilt) **passes the six brief
  cases** — block insertion never changes the pairing parity, and crossed lines keep their
  flip — and fails only on the added `g b > w- t, t > w+ b` (the initial `b` line gains the
  `t` propagator; a global sign, so only container equality and per-diagram amplitudes see
  it, never |M|²).
- *Finding: ambiguous forced line.* `e+ e- > z e+ e-, z > e+ e-`: the core holds its own
  s-channel `Z → e+ e-`, so after permutation one graph is stitched twice with a different
  line forced (64 stitched against 60 filtered graphs). Refused (`DecayChain`, "ambiguous").
- *Oracle 3, MadGraph census* (`validation/madgraph/dump_decay_chain_census.py` →
  `decay_chain_census.json`, hermetic `decay_chain_census` test; `HelasMultiProcess` on the
  pinned checkout): 19 cards, 17 stitched and matched on 54 (process, decays), final state
  order included, MadGraph's per-ME diagram count equal to the stitched diagrams whose
  forced lines lead to MadGraph's blocks; 2 refused (a two-particle "decay", refused by both;
  the dropped `w+` decay, refused here by design). MadGraph does **not** permute identical
  particles between decays: it divides by `identical_decay_chain_factor`
  (`helas_objects.py:4581`), 2 for `z z, z > e+ e-`, where here the permuted diagrams (+2)
  and the final state's 2!·2! = 4 apply. So the σ of such a card differs from MadGraph's by
  the interference between pairings (D3: gate σ first on cards without identical particles
  across decays). `trim_diagrams(decay_ids)` (`:1270`) removes nothing: it only flags the
  decaying external legs `onshell = True`.
- *S1 correction*: `Diagram::renumbered` (hence `canonical`) swapped a flipped
  propagator's endpoints without conjugating its particle, so two equal graphs whose
  enumerations oriented a charged line oppositely could compare unequal. `Prop::particle`
  is the particle of the slot at `endpoints[1]` (pinned on all 2656 census diagrams, 3397
  charged lines, with momentum and sign rebuilt from the graph); `renumbered` and
  `canonical` now take the model and conjugate. `renumbering_preserves_signs_and_amplitudes`
  stays at 0 failures with seven chain cards added (three with VVV/VVVV vertices, whose
  signs are read at a stitched diagram's anchor).
- *Unchanged enumeration*: the 51 banked scripts and 14 extra cards dump identical
  per-subprocess diagrams (legs, props' particle/endpoints/momentum, vertices, sign, symmetry
  factor) at `f67c8d0` and after the merge: 2550 diagrams (and 2537 at `a3c4c06` before it).

### D3: decay-chain phase space, σ and the sampler ladder (performance-dev or feature-dev; after D2)

- Forced BW windows (`gForceBW = 1`, `bwcutoff`) in the channel maps.
- `cut_decays` becomes consumed.
- σ gates against MadGraph decay-chain runs. The comparison is with
  MadGraph's decay-chain σ, not σ × BR: the window keeps
  ≈ (2/π)·atan(2·bwcutoff) of the BW, about 0.979 at 15.
- **The sampler ladder.** Wall time per effective event and unweighting
  efficiency against final-state leg count, on:
  - `e+ e- > z z, z > …`;
  - `p p > t t~, (t > w+ b, w+ > l+ vl), t~ > b~ j j`;
  - the fully decayed `t t~ h h`.

  This measurement is what decides whether a MadSpin-style
  decay-after-generation step is ever needed. The hypothesis is no: each
  forced resonance is one BW-mapped invariant, the decay subtrees are shared
  by every channel, and what remains are smooth decay angles.

### S3: `$` as the pointwise integrand (feature-dev; after G1, needs the per-channel |M_c|²)

- F(x) per §1.2, with MadGraph's Γ/M < 0.1 rule and the `bwcutoff` window.
- σ gate against a MadGraph `$` run (`p p > w+ b w- b~ $ t t~`,
  `p p > l+ l- $ z`).
- Informational: the complement identity of §1.2 against `p p > l+ l-`.

**Landed** (2026-09-26; `fdd0f34` first form, `c46d268` the amplitude-level
fix, `7a1eb52` gates, the docs commit after them). `$` is lifted from
`Unsupported`; `SupportedProcess` carries the list as written and
`diagrams::forbidden_onshell_ids` resolves it as `$$`'s is (oriented ids),
refusing a card whose process lines name different lists. `onshell.rs`
marks, per subprocess, the s-channel lines whose oriented id is listed,
keyed by (final-side legs, M, Γ), and compiles for every nonempty subset of
them (≤ 6 lines) an amplitude from the diagrams carrying none; a point
evaluates the subset on its window (the `P1D` theta, §1.2). `hadronic.rs` and
`proton.rs` evaluate `|M'|²` through these (scale-aware, moved to the
subprocess's own coupling), in the survey, the integral, `event_in_channel`
and `select_event`; the scale's and colour flow's `AMP2` draws drop the
configurations whose representative carries a zeroed line (at
`SDE_strategy = 1`; at 2 the channel-cut weights are MadEvent's own and are
left alone); a zeroed-amplitude colour flow is mapped into the whole
amplitude's basis by its tag row. Flavour-group members must share the
representative's marking (checked, refused otherwise). The Prop-level marking
D2 adds can replace the leg-key bookkeeping at merge.

- *What MadEvent does* (§1.2, corrected): the first form here (|M|² times the
  surviving configurations' `AMP2` share, with cut_bw's Γ/M < 0.1 rule and an
  `SDE_strategy = 2` refusal) read `e+ e- > mu+ mu- $ z` at 100 GeV 0.93% low,
  35 σ; the generated `matrix1_optim.f` showed `FFV2_4P1D_3(…, BWCUTOFF, …)`
  for the Z. The refusal of `SDE_strategy = 2` was dropped with it: the
  propagator acts at any SDE strategy (read from the source, not run).
- *Pointwise*: on the pole and at 100 GeV the zeroed amplitude equals the
  separately compiled `e+ e- > mu+ mu- / z` to 1e-12; for
  `u u~ > w+ b w- b~ $ t t~` MadGraph's standalone `SMATRIX` (with the
  `FFV2P1D_1` fix below) equals the zeroed amplitudes here to 1e-12 at six
  points (both, one, the other and no top in the window).
- *σ rows* (MadEvent 3.7.1 via `mg5_pinned.sh`, five seeds, 10k events,
  `validation/madgraph/onshell_veto_reference.json`; here five seeds,
  `cli_onshell_veto`; each mean ± max(quoted, spread/√n)):

  | row | MadEvent | here | pull | χ²/dof here |
  |---|---|---|---|---|
  | `e+ e- > mu+ mu- $ z`, √s = M_Z | 10.7673 ± 0.0021 | 10.7678 ± 0.0020 | +0.16 | 1.02 |
  | same, √s = 100 | 9.00988 ± 0.0017 | 9.01035 ± 0.0016 | +0.19 | 0.75 |
  | same, √s = 100, bwcutoff 3 (outside) | 51.282 ± 0.012 | 51.289 ± 0.010 | +0.43 | 0.22 |
  | same, √s = 200 (outside) | 2.78715 ± 0.00064 | 2.78740 ± 0.00050 | +0.31 | 0.60 |
  | `t > b e+ ve $ w+` (GeV) | 1.43508e-3 ± 7.4e-7 | 1.43423e-3 ± 4.8e-7 | −0.97 | 0.59 |
  | `p p > e+ e- $ z`, dy13 card | 303.83 ± 0.27 | 303.84 ± 0.14 | +0.05 | 1.47 |
  | `u u~ > w+ b w- b~ $ t t~`, √s = 500 | 0.06811 ± 0.00008 | 0.04982 ± 0.00004 | (MadGraph defect) | — |

  Known-wrong comparisons (MadEvent's own unrestricted rows): 2023.95 pb at
  the pole, 51.28 pb at 100 GeV, 933.15 pb for Drell–Yan, 13.664 pb for the
  top pair; the unrestricted 100 GeV row equals the bwcutoff-3 row seed for
  seed in MadEvent.
- *The top-pair row is a MadGraph defect*: ALOHA writes the `1D` theta of a
  fermion propagator built from its second spinor slot (`FFV2P1D_1`, the `t`
  here) with the P flip applied inside the square,
  `(p² + (M − cΓ)²)(p² + (M + cΓ)²) ≥ 0`, which is always true, while
  `FFV2P1D_2` (the `t~`) and the vector forms are right. So MadEvent zeroes
  the `t~` window and not the `t` one. Patching that one line in the
  generated `Source/DHELAS` brings MadEvent to 0.04982, 0.04942, 0.04957,
  0.04970 (10k events, fixed μ = 173) and 0.04934 ± 0.00007 (50k) against
  0.05059–0.05080 here (fixed μ; 0.050695 ± 0.000024 at a 5e-4 target):
  about 2% apart, and MadEvent moves down with budget while this side does
  not, nor under another split-angle map (`isotropic`, 0.050724 ± 0.000049);
  the unrestricted fixed-μ rows agree, 15.060 against 15.052. Left open;
  the row is reported, not gated.
- *Complement identity* (informational, Drell–Yan card, MadEvent's
  `p p > z, z > e+ e-` with `cut_decays = T`, 631.08 ± 0.35): MadEvent
  chain + `$` = 934.91 against its full 933.15 (+1.75 ± 0.77); with this
  side's `$` and full (933.57 ± 0.64) +1.35 ± 0.75. The difference is minus
  the γ–Z interference inside the window, −0.15% of the total.
- *Unchanged cards*: the merged base `f67c8d0` and this branch write
  byte-identical artifacts and 500-event LHE files (header's artifact path
  aside), seed 7, `--fixed-budget` 4000 × 4, for `e+ e- > mu+ mu-` (45.6 GeV
  beams), `e+ e- > mu+ mu- ta+ ta- QCD=0` (125 GeV beams), `u u~ > g g g`
  (125 GeV beams, ptj 20, clustering scale), `p p > e+ e-` (dy13) and
  `t > b e+ ve`.

### P1: polarized external particles (feature-dev; after G1)

- `{0}`, `{T}`, `{L}`, `{R}`, `{±1}` on external legs restrict that leg's
  helicity loop. `{T}` means ±1. `{0}` on a massless boson is kept and
  bypassed at generation time, as MadGraph does
  (`madgraph_interface.py:5175`). Fermion helicities go through the same
  path.
- **The frame is part of the answer.** Unlike the helicity sum, a polarized
  |M|² is not Lorentz invariant. MadGraph evaluates it after
  `boost_to_frame` into the rest frame of the particles listed in the run
  card's `me_frame` (`Template/LO/SubProcesses/genps.f:1759`). The default,
  `[1, 2]`, is the partonic centre-of-mass frame, which our evaluators already
  require. So:
  - the default frame needs no new code, only a test that would fail if the
    evaluation frame were different;
  - a non-default `me_frame` on a polarized card is refused until the boost
    exists;
  - `me_frame` moves out of `IgnoredBenign` in `runcard/classes.rs`, since
    `B_FRAME` argues from "every amplitude here is Lorentz invariant", which
    stops being true. Its stored-default mismatch (empty against MadGraph's
    `[1, 2]`) is fixed at the same time.
- **Initial-state polarization:** the spin-averaging denominator for a
  polarized incoming leg is pinned against MadGraph's `IDEN`, not assumed.
- **Event records:** `SPINUP` of a polarized leg is its fixed helicity.
- **Not in P1: polarized intermediate resonances**
  (`p p > w+{0} w-, w+ > e+ ve`) and the propagator-only codes
  `{A}`/`{G}`/`{H}`/`{Q}`/`{W}`/`{S}`. They replace the resonance's
  propagator numerator with a helicity projection, which is a change to the
  `helas/eval` propagator (the area the performance PRs are editing), not to
  the helicity loop. `check_supported` refuses them, with a backlog entry.
  `{A}` on an external leg is a MadGraph error and stays one
  (`helas_objects.py:686`).
- Gates:
  - amplitude rows against MadGraph for `e+ e- > w+{0} w-{T}`,
    `p p > z{0} j` and a polarized fermion leg;
  - one σ row;
  - the `samples` category on `SPINUP`;
  - a mutation pin that evaluates the default-frame card in a boosted frame
    and must disagree.

**Landed** (2026-09-25; `3b3f71e` feature, `3c023b2` rows, the docs commit after
them guide and notes). `Unsupported::Polarization` is gone; `{A}` and the other propagator
codes stay `PropagatorPolarization`, and a polarization on a particle a decay
chain decays (or on a decay's own initial leg) is the new
`DecayedPolarization`. `SupportedLeg` carries the text between the braces,
enumeration reads it into MadGraph's codes per concrete particle, and
`DiagramSet.polarizations` carries each leg's `NHEL` list to
`AmplitudeEvaluator::compile`, which sums over the product of the lists. The
diagrams are the unpolarized ones. MadGraph facts pinned, all at `b7687064`
unless a run is named:

- *Helicities* are the listed codes in the listed order
  (`get_helicity_matrix`, `helas_objects.py:4834`), MadGraph's own `NHEL`.
  `{T}` is `[1, -1]`, `{L}`/`{R}` are −1/+1 for every spin (`L` on a vector
  only logs a warning, `madgraph_interface.py:5122`), and a comma in the
  braces is a decay separator: `a{0,T}` is a MadGraph parse error, `a{0T}` is
  the concatenation.
- *Averaging* (`get_denominator_factor`, `helas_objects.py:4910`): a
  polarized incoming leg contributes `len(polarization)` in place of its
  spin states. Generated code: `DATA IDEN/ 2/` for `e+ e-{L} > mu+ mu-`
  (`matrix1_optim.f:89` of this sprint's run). The census has `e+{R} e-{L}`
  at 1, `u{L} u~ > z{0} g` at 18, `g{R} g > t t~` at 128, `g{T} g` at 256.
  `initial_spin_color_average` follows it.
- *Identical particles* key on `(id, polarization)`
  (`identical_particle_factor`, `base_objects.py:3742`): `z{0} z{0}` and
  `z{T} z{T}` are 1/2, `z{L} z{R}` and `z{0} z{T}` are 1.
  `hadronic::outgoing_symmetry_factor` and `Subprocess::symmetry_factor`
  follow it; within a line the deduplication key is `(name, polarization)`
  as in MadGraph's `tag = zip(prod, polids)` (`diagram_generation.py:1765`).
- *Helicity 0 on a massless boson* is kept by the parser
  (`madgraph_interface.py:5180`) and removed at generation, all zeros for an
  initial leg and one for a final leg (`diagram_generation.py:1751`,
  `:1791`); a leg left empty drops the assignment. `u u~ > v{0} g` with
  `v = z a` generates `u u~ > z{0} g` alone; `e+ e- > a{0} z` and
  `g{0} g > t t~` are `NoDiagramException`.
- *Ambiguity* (`check_polarization`, `base_objects.py:3869`, called at
  `madgraph_interface.py:3324`): an outgoing particle both polarized and not,
  or with overlapping unequal lists, makes MadGraph ask, and a batch run
  answers no and raises `InvalidCmd` (`p p > z{T} z`, `z{L} z{T}`,
  `z{RL} z{LR}`, `a{0} a`). Ported as `resolve::polarizations_unambiguous`
  and refused in both `resolve_card` and enumeration.
- *Frame*: `me_frame` defaults to `[1, 2]` (`banner.py:4296`), `frame_id =
  Σ 2^n` over it (`banner.py:4705`), and `auto_dsig_v4.inc:134` boosts only
  when `frame_id != 6`. The polarized run's `run_card.dat` shows
  `1, 2 = me_frame` and `Source/run_card.inc` sets `FRAME_ID = 6`; the frame
  block is displayed only for a polarized *massive* leg (`banner.py:5027`,
  and the `e-{L}` run's card has no such line). `create_default_for_process`
  changes no frame default.

Refused here though MadGraph generates: a code that is no helicity state of
the particle (`z{2}`, `h{R}`), a code listed twice (`z{00}`), and two lines
whose same-particle subprocesses would both count a helicity state
(`z{0} h` + `z h`); lines that split a process by disjoint polarizations
(`z{0} h` + `z{T} h`) are accepted. A fixed-energy card whose subprocesses
average differently is refused (`InconsistentSpinAverage`), and flavour
groups only join subprocesses polarized alike. `me_frame` moved to
`Consumed` (`RunCard::frame_id`, read by `hadronic::refuse_polarized_frame`
for a polarized massive leg); its stored default is now MadGraph's `1, 2`;
`frame_id` is `IgnoredBenign` as a system parameter MadGraph recomputes.

- *Census* (`dump_polarization_census.py` → `polarization_census.json`,
  hermetic `polarization_census`): 35 cards through MadGraph's generation
  and `HelasMatrixElement`; 24 generated and matched subprocess for
  subprocess on legs, `NHEL` set and `IDEN` (`pp_z0_j`'s 12 included), 7
  refused by both, 4 refused here only (above).
- *Amplitude rows* (hermetic `amplitude_oracle`, gated, `bundled = false`):
  `ee_to_wp0wmt`, `ee_to_wp0wm`, `ee_to_z0h`, `uux_to_ztg`,
  `ee_to_mumu_eml`, `ee_to_tlt`, each against MadGraph's own `NHEL` table
  per diagram and per flow; worst |M|² 2.2e-13, per-diagram ≤ 6.0e-15.
  Landed informational, promoted once every level agreed.
- *Frame* (`polarization_frame`, and `proton` unit tests): at the banked
  centre-of-mass points this side reproduces MadGraph; boosted by
  β = (0.3, −0.2, 0.5) the polarized massive rows move 0.32–0.95 away while
  the helicity sums stay within 1.6e-14 and the massless `e-{L}` row within
  2.5e-14. The proton integrand of `u u~ > z{0} g` reproduces a
  centre-of-mass assembly to 1.5e-12 and sits 6.7 (relative) from the
  laboratory one. A finding on the way: the banked W points sit on the card's
  printed `MW = 80.419`, 6e-8 off the derived mass both programs evaluate
  with, and an off-shell W's helicity sum moves by ~2e-8 under that boost
  (∝ β²); the invariance control therefore projects on shell first. The
  amplitude gate is unaffected, since both sides share the points.
- *σ* (`e+ e- > w+{0} w-`, 500 GeV, MadGraph's default card, pinned
  MadEvent, 10k events: 0.2578 ± 0.00047 pb; `vibegraph integrate`, seeds
  1–5): mean 0.257932 (+5.1e-4), pulls −0.02 … +0.48, seed χ²/dof 0.97,
  VEGAS χ²/dof 0.31–2.61. Decomposition: `w+{T} w-` here 6.93829 against
  MadGraph's 6.954 ± 0.011 and `w+ w-` 7.19601 against 7.2123 ± 0.011 (both
  −2.3e-3, pulls −1.2 … −1.9; the unpolarized row is the same offset, so it
  is not polarization); `w+{0}` + `w+{T}` = `w+ w-` to 3.0e-5 across
  independent seeds, and pointwise to 3.8e-16 in any frame (an external
  leg's helicities add incoherently, so no interference is lost).
- *Samples* (`vibegraph generate`, 20000 events × 3 seeds against the 10000
  of the MadEvent run): the W+ `SPINUP` is 0 on every event on both sides;
  the W− distribution χ² 1.45/2, 0.35/2, 2.70/2; the beams 0.73/1, 0.37/1,
  0.80/1.
- *Decays* (after D1 merged): `t > w+{0} b` and `t > w+{T} b` match
  MadGraph's NHEL and IDEN (6) in the census; the decaying particle itself,
  `t{L} > w+ b` (MadGraph: IDEN 3), is refused as `DecayedPolarization`,
  since at rest its helicity is a spin projection on an axis nothing pins.
  `initial_spin_color_average`, `refuse_polarized_frame` and the
  `InconsistentSpinAverage` check read the incoming legs generically, so a
  decay's single leg goes through them unchanged.
- *Unchanged unpolarized enumeration*: the 31 Standard-Model banked scripts and 12
  extra cards (`p p > j j`, `p p > l+ l- j`, `p p > w+ w- j`, `u u~ > z z`,
  two-line cards, a five-flavour card, …) dump byte-identical
  per-subprocess diagram lists (`Debug` of every diagram, 1892 subprocess
  entries, 383 populated, 2890 diagrams) at `65edb40` and at `3c023b2`; the
  unpolarized helicity order is unchanged, and `amplitude_oracle`'s 42
  earlier rows pass unchanged.


### E1: event records and `add process` completion (feature-dev; after D2)

- Status-2 resonance records with mother pointers, and colour through decays.
- MadGraph's scale choice for decay chains (the core process) through
  `coupling::scales`.
- `@N` → `LPRUP` with one `<init>` line per process, and every process in
  the run metadata.
- Grouping `add process` final states by content rather than by slot order.
- Gates: the `samples` category against MadGraph for one decay-chain run and
  one two-`@N` run.

## 5. Decisions

- **2026-09-25 (user):** the release scope becomes MadGraph LO feature parity
  without MLM and without NLO. The scope now includes decay chains, 1→n
  decays, the s-channel restrictions and same-multiplicity `add process`.
  MLM, then NLO, follow in later sprints; §3.2 is the room left for them.
- **2026-09-25 (user):** the parser is full-featured, and one check refuses
  what is unsupported (§3.1).
- **2026-09-25 (user):** the stitching oracle is diagram-container equality;
  sign resolution moves to the diagrams stage (§3.3).
- **2026-09-25 (user): polarized external particles join the sprint** (session
  P1). At the matrix-element level this only restricts the helicity loop.
- **2026-09-25 (user): squared-order constraints stay refused, shelved.**
  Supporting them means extracting the complex amplitudes grouped by
  coupling order. That is also what reweighting in a coupling needs, so it is
  worth doing, but it is a sizable refactor of `helas/eval`, where several
  open performance PRs are working. G1 parses `^2` constraints in full and
  `check_supported` refuses them.
- **2026-09-25 (user): the working rhythm.** Performance work is already
  running in parallel on PRs that mostly touch the evaluator, so this feature
  sprint takes the slot. Sessions should stay out of `helas/eval` where they
  can. The exception is S1, which removes sign factors from there: it lands
  after, or rebases onto, whichever evaluator PRs are open at the time.

- **2026-09-26 (user): identical particles across decays keep the full
  permutation.** MadGraph does not permute identical final-state particles
  between decays: it keeps one pairing and divides by
  `identical_decay_chain_factor` (`helas_objects.py:4581`), dropping the
  interference between pairings (D2 Landed). We keep every pairing, with the
  interference and the final state's own identical-particle factor. This is a
  deliberate, documented deviation, so a decay-chain σ with identical particles
  across decays is not expected to equal MadGraph's. D3 gates σ on cards without
  that overlap, and reports the overlapping cards against MadGraph only as
  informational, with the expected difference being the interference between
  pairings.

## 6. Risks

- **The container-equality oracle is only as strong as S1.** If any sign that
  depends on numbering stays in `helas`, two equal containers can still
  disagree. S1's byte-identical gate cannot see this; a stitched-vs-filtered
  amplitude comparison can. So keep one informational per-diagram amplitude
  comparison on the D2 cases until S1's migration is shown complete.
- **Cost of the full-final-state oracle.** Enumerating the filtered full final
  state is feasible only on small cases, which is why the D2 cases are
  lepton-collider sized.
- **Leg count against the sampler (D3).** This is measured, not argued. If
  the ladder degrades faster than ME cost alone explains, that is a sampler
  finding for the performance backlog, not a reason to reopen MadSpin
  without the data.
