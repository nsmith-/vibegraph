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
- **`$` forbidden on-shell** (`:781`): keeps every diagram, marks the
  propagator `onshell = False`, and `export_v4.py:5879` writes it out as
  `gForceBW = 2`. MadEvent's `cut_bw` (`Template/LO/SubProcesses/myamp.f:136`)
  then rejects the point **only in integration channels whose configuration
  contains that propagator**, only when `sde_strat == 1`, and only when
  Γ/M < 0.1. A `$` in the process line forces `sde_strategy = 1`
  (`madgraph/various/banner.py:4774`). The window is
  |m − M| < `bwcutoff`·Γ.

  In our single-sample multichannel form, this is a pointwise integrand:

  F(x) = |M|² · (1 − Σ_{c ∋ forbidden} w_c(x) · 1_W(x)), where
  w_c = |M_c|² / Σ_d |M_d|².

- **`$` and a decay chain are complements.** Writing W for the on-shell
  window:
  - `p p > z, z > l+ l-` gives ∫_W |M_Z|².
  - `p p > l+ l- $ z` gives ∫_outside W |M|² + ∫_W |M|² w_γ.

  Together they reproduce `p p > l+ l-` except for the γ–Z interference
  inside the window, weighted by w_Z. `p p > z > l+ l-` is **not** a
  complement of `$ z`: it has no window, so it double-counts the off-shell Z
  tail.

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
