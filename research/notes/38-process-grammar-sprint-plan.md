# 38 — `process-grammar` feature sprint plan: MadGraph LO process parity

**Status: PLANNED (scope decision 2026-09-25, user).** No session has run.

The sprint that closes the gap between MadGraph's leading-order process
language and ours, everywhere except MLM matching and NLO. Three
deliverables, as asked:

1. **A full-featured proc-card parser** into a data structure that mirrors
   MadGraph's `ProcessDefinition`, followed by **one scan** that refuses every
   feature not yet supported, before anything downstream sees the card. That
   scan is the single place where the feature backlog is kept against
   MadGraph.
2. **The s-channel restriction syntax** (`>` required, `$$` forbidden, `$`
   forbidden on-shell), or-multiparticles, and `add process` over processes
   with the same final-state multiplicity.
3. **Decays**: 1→n decay processes, and decay-chain syntax
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

### S1: signs to the diagrams stage (validation-dev; before D2)

Per §3.3. Gate: every amplitude row byte-identical; `rooting_soundness`
green; the container canonical form round-trips over every banked process.

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
- **Open (user):** the parity goal also covers two LO features that earlier
  decisions descoped: squared-order constraints (note 35 §7 D4) and polarized
  external bosons (`w+{0}`). G1 parses both and refuses them in the check. Do
  they join this sprint or stay refused?
- **Open (user):** by the working rhythm, the next slot is performance. This
  plan assumes the user is taking a feature slot instead.

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
