# 35 — `ufo-lorentz` feature sprint plan: general UFO Lorentz structures

**Status: PLANNED 2026-09-05 — awaiting the §7 decisions; nothing dispatched.**

The feature sprint that takes the UFO surface past the Standard Model's
feature set. Three deliverables, as asked:

1. **Complete the Lorentz tensor representation** in `helas/repr/lorentz.rs`
   and finish the spinor completeness-relation unit tests — the `AsymRank2Tensor`
   placeholder, the `tensor bilinear f̄ σ^μν Γ f` TODO, and the
   `test_completeness_relations` TODO that has waited on it.
2. **SMEFTsim as a submodule, `SMEFTsim_topU3l_MwScheme_UFO` as the test case**
   for every Lorentz-structure vertex primitive the SM never exercised —
   gated the project's way, per-diagram × per-helicity against MadGraph's own
   `MATRIX1`, informational first and enforced when agreement is demonstrated.
3. **A toy UFO** for whatever Lorentz or colour structure SMEFTsim leaves
   unexercised.

§1 is the census this plan is sized from — a static census of the model plus
a **measured probe** of today's loader against it, so every wall below is one
the code actually hit, not one inferred from reading. §2–§4 are the sessions,
§5 the coverage table each primitive is gated by, §6 sequencing, §7 the
decisions the user owns.

The standing rules are unchanged: never a loosened tolerance; new physics
lands informational and is enforced only when agreement is demonstrated; a
known-wrong informational comparison runs while a feature is under
construction; every convention claim is pinned by a test that would fail if
it were false (the ALOHA `Epsilon` sign, MadGraph's four-fermion permutation
sign and the interaction-splitting rule are the three this sprint adds to the
list); amplitude disagreements go straight to the per-diagram × per-helicity
dump (note 12).

## 1. Inputs and the census

### 1.1 Where the code is

- `helas/repr/lorentz.rs`: `LorentzVector`/`ComplexVector` (variance-typed),
  `Bispinor<F, Ket|Bra>` in the Weyl basis with scalar, pseudoscalar, vector
  and axial-vector bilinears; `AsymRank2Tensor<F>(pub [C<F>; 6])` is a
  **placeholder with no operations**; `SpinorRepr` ends at
  `// TODO: tensor bilinear`. `test_completeness_relations` checks the
  helicity-summed scalar/vector/axial/pseudoscalar projections of
  `Σ_h u ū = p̸ + m` and stops at `// TODO: when tensor bilinears are
  implemented`. `intertwiner.rs` documents `SigmaTensor`/`Epsilon` stubs that
  do not exist.
- `ufo/lorentz.rs`: PEG grammar over `Gamma, Sigma, Identity, ProjM, ProjP,
  Metric, P, Epsilon, C`; anything else is `UnknownOperator`. No `Gamma5`, no
  `**` power, and an `#[ignore]`d test recording that nested call arguments
  (`FFCT2((P(-3,3)+P(-3,4))*…)`, `taudecay_UFO`) fail to parse rather than
  reporting the unknown operator.
- `ufo/mod.rs`: `ParsedModel::parse` **refuses any directory containing
  `propagators.py`** (the note-29 descoping hard error); `apply_restriction`
  drops vertices whose *every* coupling is zero but keeps zero couplings and
  their Lorentz structures inside surviving vertices.
- `ufo/topo.rs`: one feyngraph vertex per UFO vertex, `spin_map` taken from the
  **first** Lorentz structure, and `coupling_orders` the **union over all of
  the vertex's couplings** (last value wins per order name).
- `helas/eval/root_lorentz.rs`: the greedy tree rooting handles `Gamma`
  (three orientations), `ProjM/ProjP`, `Identity`, `Metric`, `P`; `Sigma`,
  `Epsilon`, `C` return `UnsupportedVertex` when on the rooted path and hit a
  `todo!()` when they are a disconnected scalar factor. A fermion-output
  `Gamma` reached through a *summed* spinor index (a γ-chain) errors with
  "fermion-output Gamma rooted without a spinor adjoint" because the adjoint
  is read off the output leg only. `root_diagram.rs:763` asserts
  "0 or 2 fermion legs per sink".
- `helas/eval/compile.rs::mg_validated_suite_exercises_every_op`: the two-way
  op census; `KNOWN_UNCOVERED = [Hels, IdentityAmp]`.
- `helas/color/`: a general port of `color_algebra.py` over `T/Tr/f/d/One`;
  `d` is representable but unexercised; `K6/K6Bar/T6/Epsilon/EpsilonBar` are
  explicit `Unsupported` errors. `helicity_states_for_spin` accepts spin code
  5 but nothing downstream builds a tensor wavefunction.
- Validation harness: `validation/manifest.toml` rows + one `.mg5` script per
  row; every script imports the default `sm`; `gen_amplitude.py` reads each
  process's own `Cards/param_card.dat` for masses (so both sides evaluate the
  same rounded card — the same mechanism carries Wilson coefficients for
  free). `mg5_pinned.sh` runs the submodule's MadGraph (3.7.x line).

### 1.2 SMEFTsim `topU3l_MwScheme` — static census (v3.0.2, `db7d4a80`)

21 particles (spins 1/2/3 only, no ghosts; colours 1/3/8), 260 Lorentz
structures, 904 vertices, 1278 couplings, 315 parameters; `propagators.py`
present, bound to the four auxiliary fields `Z1`, `W1±`, `t1`, `H1`
(pdg 9000005–8) whose 125 vertices all carry the `NPprop` order
(`expansion_order = 0`). MIT licence.

| Operator | uses | Status today |
|---|---|---|
| `P` | 3189 | parsed; `P(-1,a)*P(-1,b)` scalar products root today; **`P(-1,a)**2` (123 uses) does not parse** |
| `Metric` | 2885 | parsed and rooted |
| `Epsilon` | 846 | parsed; **`todo!()`/`UnsupportedVertex` at rooting** |
| `Gamma` | 137 | parsed; γ-chains (two `Gamma` sharing a summed spinor index) **fail to root** |
| `ProjM`/`ProjP` | 40 / 40 | parsed and rooted |
| `Gamma5` | 13 | **`UnknownOperator` — the whole model fails to load** |
| `Identity` | 3 | parsed and rooted (`IdentityAmp`, today's one `KNOWN_UNCOVERED` op — SMEFTsim's `FFS2` covers it) |
| `Sigma`, `C` | **0** | FeynRules expands σ^μν into γγ chains; SMEFTsim never emits either |

Leg-spin patterns: 62 six-vector and 57 five-vector structures (the `cG`/`cW`
field-strength cubes), 21 four-fermion, 20 `VVVVS`, 15 `VVVV`, 10 `FFV`,
7 `FFVS`, 5 `FFVVS`, 4 `FFVV`, and the rest three-point. Four-fermion
structures come in three shapes: scalar⊗scalar (`ProjM(2,3)*ProjM(4,1)`),
vector⊗vector (`Gamma(-1,2,-2)*Gamma(-1,4,-3)*Proj*Proj` — tree-shaped), and
**tensor⊗tensor** (`Gamma(-2,-4,-3)*Gamma(-2,2,-6)*Gamma(-1,-6,-5)*Gamma(-1,4,-4)*…`)
whose index graph is a **4-cycle** — no rooting of a tree can evaluate it; it
needs a rank-2 tensor intermediate. Pairings are `(1,2)(3,4)` in 14 of the 21
structures and `(1,4)(2,3)` in 7, and **70 of the 200 four-fermion vertices
mix both pairings** in one vertex. Colour strings: `1`, `Identity`, `T`,
`T(-1,·,·)*T(-1,·,·)`, `f`, `f*f`, `f*f*f` chains; no `d`, no sextets.
Dipoles appear as `P(-1,3)*Gamma(-1,2,-3)*Gamma(3,-3,-2)*ProjM(-2,1)`-type
momentum-slashed γ-chains, with `Gamma5` inside the chain for the CP-odd ones.

### 1.3 The measured probe (2026-09-05, this container)

An untracked example loaded a scratch copy of the UFO with `propagators.py`
removed, `Gamma5(a,b)` rewritten as `(ProjP(a,b) - ProjM(a,b))` and
`P(x,y)**2` as `P(x,y)*P(x,y)`. Everything else is today's code.

| Step | Result |
|---|---|
| `ParsedModel::parse` | **OK in ~100 ms**; 315 parameters, **0 NaN**; MW-scheme derived values correct (`ee` 0.30825, `sth` 0.47208, `vevhat` 246.22, `yt` 0.99228) |
| `restrict_SMlimit_massless` | 60 vertices survive; SM couplings correct (`GC_4 = i·ee`, `GC_265`, `GC_204` the Z couplings) |
| `restrict_massless` (all real Wilson coefficients non-zero) | 540 vertices; arity histogram `{3: 191, 4: 272, 5: 66, 6: 11}` — **feyngraph accepts five- and six-leg vertices**, enumeration in milliseconds |
| `e+ e- > mu+ mu-`, SM limit, default or `QED<=2` | **0 diagrams** (MadGraph: 2) |
| `e+ e- > mu+ mu- QED<=2 NP<=1`, all WCs | 1 diagram (the four-fermion contact only); `QED<=2` alone: 19 |
| `g g > t t~ QCD<=2`, SM limit | 4 diagrams (MadGraph SM: 3 — the fourth is L1's to identify), compile fails: "fermion-output Gamma rooted without a spinor adjoint" (the zero-coupling `FFV2` dipole structure is still rooted) |
| `g g > h QCD<=2 NP<=1` | 2 diagrams (`cHG`, `cHGtil`), compile panics at the `Epsilon` `todo!()` |
| `g g > g g QCD<=2 NP<=1` | 1 diagram, `Epsilon` panic (`cGtil`) |
| four-fermion contact | compile panics at `root_diagram.rs:763` (2-fermion sink assumption) |

**Diagnosis of the zero SM diagrams** (verified from the printed couplings):
every SMEFTsim `FFV` vertex bundles the SM coupling with dipole and
current-shift couplings whose orders include `NP`; `topo.rs` unions them into
one order map per vertex, so an SM photon vertex reads as `NP = 1` (and its
QED power from whichever coupling was last), two of them exceed any `NP<=1`
or WEIGHTED bound, and only the single-vertex contact term survives.
MadGraph's `import_ufo.add_interaction` does the opposite: **one MG interaction
per distinct coupling-order tuple** (`order_to_int`), each carrying only that
tuple's `(color, lorentz)` couplings — and a restriction removes zero
*couplings*, not just empty vertices, so the SM-limit `FFV` vertex reaches the
evaluator with `FFV1` alone. Both are L1's spec; both are prerequisites for
every gate below, including the SM-limit one that needs no new primitive.

### 1.4 Reference conventions read from the pinned MadGraph source

- **`Epsilon`** (`aloha/aloha_object.py::L_Epsilon.give_parity`): the
  component `(l1,l2,l3,l4)` is `−sign(perm)`, i.e. **`ε^{0123} = −1`**
  (equivalently `ε_{0123} = +1`) on ALOHA's upper-index representation, with
  the metric applied at contraction. A convention hypothesis until the MG gate
  pins it (§5, `gg_to_h_cpodd`).
- **Four-fermion permutation sign** (`models/import_ufo.py::get_sign_flow`,
  `aloha/aloha_fct.py::get_fermion_flow`): per Lorentz structure MadGraph
  reads the spinor pairing `{I1: O1, I2: O2}`; if it differs from the
  expected `(1,2)(3,4)` it prefixes the coupling with the parity of the
  induced permutation of particle positions. Reproduce the algorithm, not a
  paraphrase — and note that Majorana fermions in 4-fermion vertices are an
  MG `InvalidModel`, which keeps them out of this sprint by construction.
- **Interaction splitting** (`import_ufo.py::add_interaction`,
  `order_to_int`): one interaction per coupling-order tuple, couplings
  keyed `(color_idx, lorentz_idx)` restricted to that tuple. Diagram counts
  in `diagrams.json` for SMEFTsim rows will be per split interaction; ours
  must split identically to compare.
- **`expansion_order`** (`coupling_orders.py`): `NPprop` is 0 — MadGraph
  caps an unconstrained order at its `expansion_order`, which is what keeps
  the four propagator-corrected auxiliary fields out of every default
  process. L1 verifies the exact rule in `madgraph_interface.py` before
  implementing it; the hypothesis is recorded here so its falsification is
  visible.
- **Sign of `restrict_massless`**: every real Wilson coefficient is set to a
  distinct fixed value (`cG 0.2, cW 0.3, cH 0.4, …`) with `LambdaSMEFT =
  1000`; imaginary parts stay zero. One card turns every structure class on,
  which is why the amplitude ladder below imports it and switches classes
  *off* per row through the param card (§4, V1).

## 2. Session-scoping ground rules

Inherited from notes 28/29/34, with the additions this sprint needs:

- **One deliverable per session**; oracle before engine; the session that
  builds an engine does not bank the reference it is judged by.
- **Land informational first.** Every SMEFTsim `amplitudes` cell is added by
  V1 as `info`, expected red, before the engine session that flips it; a
  session's gate is "the cells its brief names go green **and** every
  previously-enforced cell is unmoved" (`pixi run --skip-deps validate`).
- **Per-model op census is the coverage instrument.** L1 generalises
  `mg_validated_suite_exercises_every_op` to run per `(model, process list)`
  with a `KNOWN_UNCOVERED` per model; a new kernel that no SMEFTsim or toy
  row exercises is a red census, not a passing suite.
- **Hermetic where possible.** R1 is pure `repr` unit tests; the loader
  sessions test on the submodule (banked layer, `required-features`
  registration, never a runtime skip); the SMEFTsim `amplitudes` cells are
  hermetic once their tables are banked, like today's.
- **Worktrees pre-created off `main` by the manager, reference data
  COW-cloned in, submodules (both) present** — a fresh worktree gets neither
  `mg5amcnlo` nor `smeftsim` content and `cargo test` would fail-fast on the
  missing sources. Dev-agent briefs carry the worktree/long-command discipline
  verbatim (`.agents/agents/feature-dev.md`).
- **Sonnet relief valves**: named per session; deterministic bulk only.

## 3. Track R — representation layer and evaluator primitives

### R1 — rank-2 tensor representation and the completeness relations (feature-dev, hermetic)

`helas/repr/lorentz.rs` only. Deliverables:

- `Rank2Tensor<F, V1: Variance, V2: Variance>`: 16 complex components,
  row-major `(μ, ν)`, `ArrayBacked` + `impl_vectorspace!`, `LorentzRepr`,
  per-index `dualize` (raise/lower with the metric), `contract` with a
  same-shape tensor of dual variances (`T^{μν} U_{μν}`), left/right
  contraction with a `ComplexVector` (`T^{μν} v_ν`), outer product, transpose,
  trace, `antisymmetric_part`.
- `AsymRank2Tensor<F>` promoted from placeholder to the (1,0)⊕(0,1)
  representation: six components in a documented order, `From` into the
  general tensor and `antisymmetric_part` back, `contract`, and the Hodge
  dual `½ ε^{μνρσ} T_{ρσ}` (which is the chirality decomposition: on Weyl
  spinors `σ^μν P_L` is anti-self-dual / self-dual — a convention-pinning test
  in itself).
- `SpinorRepr::tensor_bilinear(fi, chirality) -> AsymRank2Tensor`
  (`f̄ σ^μν Γ f`, `σ^μν = i/2 [γ^μ, γ^ν]`), `gamma_pair_bilinear(fi, chirality)
  -> Rank2Tensor` (`f̄ γ^μ γ^ν Γ f`, the form SMEFTsim actually emits), and the
  identity `γ^μγ^ν = g^{μν} − i σ^{μν}` pinned between them. Spinor-side
  `slash_pair(&Rank2Tensor<Cov,Cov>)` applying `T_{μν} γ^μ γ^ν` on both
  adjoints (bra order reversed), pinned by `ψ̄ (T·γγ) ψ = T_{μν} (ψ̄ γ^μγ^ν ψ)`.
- Levi-Civita primitives `epsilon4(a,b,c,d) -> C<F>` and
  `epsilon_vector(a,b,c) -> ComplexVector` at the **ALOHA convention
  `ε^{0123} = −1`** (§1.4), antisymmetry and basis values pinned, plus the
  γ5–ε identity `f̄ σ^μν γ5 f ∝ ε^{μνρσ} f̄ σ_ρσ f` derived in the Weyl basis
  under that convention and pinned — this is the test that ties the ε sign
  to the γ5 sign inside one representation, before MadGraph is consulted.
- **Completeness relations finished**: helicity-summed
  `Σ_h ū σ^μν u = 0` (all six components) and `Σ_h ū γ^μγ^ν u = ±4m g^{μν}`
  (u/v), closing the TODO; and the stronger **per-helicity Fierz
  reconstruction** `u ū = ¼ Σ_A (ū Γ_A u) Γ^A` over the 16-element basis
  `{1, γ5, γ^μ, γ^μγ5, σ^μν}` built from explicit Weyl-basis gamma matrices
  in the test module — passes only if every bilinear's convention is
  mutually consistent, which is exactly what the summed relations cannot see
  (AGENTS.md: know each oracle's blind spot).
- `intertwiner.rs`'s phantom stubs either become thin wrappers over the new
  methods or are deleted with the doc table corrected — no "stub pending"
  text survives.

Gate: `cargo test -p vibegraph-lib --lib helas::repr` plus the whole hermetic
suite; `cargo clippy -D warnings`, `cargo fmt --check`. No MG involvement,
no `KNOWN_UNCOVERED` change (nothing is wired to the evaluator yet). **No
delegation** — small and pure judgment.

### E1 — tree-shaped structures: `Epsilon`, γ-chains, `Gamma5`, momentum algebra (feature-dev)

Everything SMEFTsim needs that is still a tree in the index graph:

- `LorentzOp::Gamma5` end to end: a `Gamma5` node (`γ5` on a continuing
  fermion current; `Gamma5Amp` on a sink), lowering to `Op`, kernels in
  `kernel.rs` reusing `pseudoscalar_bilinear`.
- **Per-node adjoint inference along a fermion line**: the adjoint of a
  fermion-output `Gamma` reached through a summed spinor index is the adjoint
  of the *external* fermion the chain leads to, not the output leg's. This is
  what makes `Gamma(3,2,-2)*Gamma(4,-2,-1)*ProjM(-1,1)` (FFVV), the dipole
  chains `P(-1,3)*Gamma(-1,2,-3)*Gamma(3,-3,-2)*ProjM(-2,1)` and every
  `Gamma5`-inside-a-chain structure rootable at any leg. A momentum-slash is a
  `GammaIout/Oout` whose vector child is a `P` node — check that the kernels
  accept it and that `POut` composes.
- `Epsilon` nodes: `EpsilonVout{a,b,c}` (three vectors → vector) and
  `EpsilonAmp{a,b,c,d}` (four → scalar), with argument order carried
  explicitly (antisymmetry makes slot order a sign), rooted at a vector leg
  or as a disconnected scalar factor. Kernels use R1's primitives.
- Momentum algebra: `P(-1,a)**2` semantics (an object with a summed index
  raised to a power multiplies copies, so `P(-1,2)**2 = p₂·p₂` — parsed by L1,
  rooted here as a scalar `Metric(P,P)`), `P(1,2)*P(2,1)` outer forms in
  `VVS` (`cHG`'s `P(1,2)*P(2,1) − P(-1,1)*P(-1,2)*Metric(1,2)`), and the
  `VVV`/`VVVV` structures with three momenta and one metric.
- Five- and six-leg vertices: verify `build_at_leg`, `Mul` scalar roots and
  the diagram walk are arity-agnostic on `g g > g g` (`cG` contact) and on
  one `VVVVV`-bearing process; fix what is not.
- The lowering fusion path (`FfvVout` etc.) must not fuse a chiral pair when
  the structure carries a momentum slash or a `Gamma5` — pin with a test that
  the fused and generic forms agree on SMEFTsim's `FFV` vertex set.

Kernel-level pins in `kernel::tests` via `prop_harness` (each new kernel
against an algebraic identity of MG-covered ones), then the MG cells:
`gg_to_h_cpeven`, `gg_to_h_cpodd` (the ε sign against MadGraph, visible only
because the CP-even diagram is present in the same helicity amplitude),
`ee_to_wpwm_cw`, `gg_to_gg_cg`, `ee_to_ttx_dipole`, `ee_to_zh_smeft`
(§5) flip `info → gate`. **Sonnet relief**: the sweep that re-runs the
amplitude tables across the ladder and reports per-cell status; the physics
and the sign diagnoses are Opus.

### F1 — four-fermion vertices (feature-dev)

- `root_diagram.rs`: lift the "0 or 2 fermion legs per sink" assumption —
  a vertex may close two fermion lines at once; the rooted tree gets one
  spinor sink per pair, each with its own bra/ket resolution and `crossed`
  bookkeeping.
- Pairing per **structure**, not per vertex: `LorentzStructure::spin_map` is
  already per structure; `topo.rs` must stop taking the first structure's map
  for the vertex (70 of 200 SMEFTsim FFFF vertices mix pairings). Design the
  split so feyngraph's one-`spin_map`-per-vertex API is satisfied — the
  MG-faithful answer is that L1's interaction splitting already yields one
  interaction per coupling tuple, and a mixed-pairing vertex splits further
  by pairing exactly as MadGraph's `get_fermion_flow` reads it. Pin against
  MadGraph's diagram count for `e+ e- > mu+ mu-` with the `cll1`/`cle`/`cee`
  classes on.
- MadGraph's permutation sign (`get_sign_flow`, §1.4) reproduced
  algorithmically and lifted into `fermi_sign` the way the other rooting
  signs are (note 19 §V5: all rooting signs live in one per-diagram scalar),
  with the `rooting_soundness` sweep extended over the four-fermion rows.
- Scalar⊗scalar and vector⊗vector pairings only in this session (both
  tree-shaped); the tensor⊗tensor structures stay `UnsupportedVertex` with a
  message naming R4 — and the `ee_to_mumu_4f` and `uux_to_ttx_4f` cells (§5)
  flip.

### R4 — the tensor slot and the cyclic four-fermion structures (feature-dev; after R1, E1, F1)

- `WaveformSlot::Tensor(TensorWf<F>)` (rank-2 + momentum), `Add`/scalar
  `Mul` arms, `prop_harness::rand_tensor`.
- Rooting: when `build_child` meets an already-visited operator through a
  second Lorentz index (the 4-cycle), cut the cycle at the fermion line that
  does not contain the output leg: evaluate that line as a rank-2 current
  (`GammaPairTout{i,j}` — `ψ̄ γ^μ γ^ν Γ ψ`), and contract it into the output
  line (`TensorSlashIout/Oout` applying `T_{μν} γ^ν γ^μ` in the order the
  chain dictates) or into the amplitude (`TensorContract`). Ops, lowering,
  kernels (R1's `gamma_pair_bilinear`, `slash_pair`, `contract`).
- Pin the γγ⊗γγ evaluation against the toy model's literal `Sigma⊗Sigma`
  vertex once T2 lands (the two must agree by the R1 identity); until then
  against MadGraph on `ee_to_ttx_tensor4f` (§5).

## 4. Track L/V — loader, order bookkeeping and the MadGraph oracle

### L1 — loader and model-topology surface (feature-dev; first session, no MG)

Parser: `Gamma5`; `**` integer powers (expand `X**n` into `n` copies with
Einstein contraction, reject `n > 2` on an indexed object — SMEFTsim uses only
`**2`); nested call arguments parse far enough to report `UnknownOperator`
(un-ignore `test_unknown_operator_taudecay_ufo`). Model surface:

- **Interaction splitting by coupling-order tuple**, MG-faithful (§1.3/§1.4):
  `resolve_vertices` (or a pass after restriction) emits one `Vertex` per
  distinct order tuple with that tuple's `(color, lorentz)` couplings only,
  named deterministically from the UFO vertex; `topo.rs` builds one feyngraph
  vertex per split; the evaluator and `colorize` see split vertices as
  ordinary vertices (their colour and Lorentz lists are subsets). The
  interned SM blob must be **byte-identical** after this change (the SM has
  one order tuple per vertex — assert it, do not assume it; `pixi run
  check-sm-blob-fresh`), and all 19 `MG_VALIDATED_PROCESSES` stay
  bit-for-bit.
- **Restriction drops zero couplings**, then structures and colour strings
  nothing references, then empty vertices — MadGraph's order of operations.
- **`expansion_order`** read from `coupling_orders.py` and applied as the
  default cap for orders the process leaves unconstrained (verify the exact
  MG rule first; record it in this note). With it, `NPprop` vertices are gone
  from every default process, which is what makes the next item safe.
- **`propagators.py`** parsed (name, numerator, denominator strings kept
  verbatim), `Particle.propagator` attached; the hard error moves from
  "file exists" to "a particle with a custom propagator propagates in a
  selected diagram" — still a hard error, now placed where it is true. The
  note-29 refusal test is rewritten to assert the new placement.
- **Per-model op census**: `mg_validated_suite_exercises_every_op` becomes a
  function of `(model loader, process list, KNOWN_UNCOVERED)`; the SM
  instance is unchanged, the SMEFTsim instance starts with every new op listed
  as uncovered and shrinks session by session (two-way).
- Banked-layer tests (`tests/ufo.rs` or a new `tests/smeftsim.rs`,
  `required-features`): load `topU3l_MwScheme` under both shipped cards;
  assert the vertex counts and arity histogram of §1.3, the split interaction
  count against MadGraph's (V1 banks it), the SM-limit derived parameters
  against MadGraph's `param_card.dat` values, and diagram counts for the
  §5 processes (`info` until V1's `diagrams.json` entries exist, then `gate`).
- `validation/fetch_common.sh::vg_ensure_submodule` learns the second
  submodule (and the sparse-checkout option of §7 D1 if adopted).

**Sonnet relief**: none — every item is a convention with a falsifier.

### V1 — SMEFTsim into the MadGraph oracle pipeline (validation-dev; runs on the MG host)

Oracle before engine, banked before any flip:

- `.mg5` scripts gain an `import model <path>` line; `build.sh` rewrites a
  repo-relative model path to the absolute work-area path as it already does
  for `output`. Manifest rows gain `model` and `restrict` fields; the report
  collator and `gen_amplitude.py` read them (its `--dump-processes`
  migration oracle diffed before/after).
- **Per-row Wilson-coefficient selection without editing the submodule**:
  `build.sh` copies the UFO directory into `output/models/…` and adds the
  committed cards from `validation/madgraph/cards/smeft/restrict_vg_<class>.dat`
  (each a copy of `restrict_massless.dat` with only one class of coefficients
  non-zero, at the shipped values). Both MadGraph and vibegraph import the
  copy with the same card, so vertex pruning is identical on both sides and
  a row compiles only the structures its class needs — the ladder in §5 is
  built from these cards.
- Bank, for every §5 row: diagram counts (`extract_diagrams.py`), the
  amplitude tables (`gen_amplitude_tables.py` — per point |M|², per helicity
  `AMP()`/`JAMP()`, MadGraph's own param card with the `SMEFT*` blocks), and
  the interaction split count. Fixed-energy σ for the capstone row.
- Register every SMEFTsim `amplitudes`/`diagrams` cell as **`info`** in the
  manifest; they turn red immediately (nothing compiles) — the known-wrong
  informational comparison the standing rule asks for.
- Extend `assemble_bundle.sh`/`refdata` (next bundle tag) so the banked layer
  fetches these like the others.

**Sonnet relief**: the MG batch driver over the ~12 scripts (backgrounded,
logs), and the per-row table sweep; card authoring and every "which
coefficient class does this row isolate" judgment is Opus.

### L2 — the SM-limit gate (validation-dev; after L1 + V1)

Flip `ee_to_mumu_smlimit`, `gg_to_ttx_smlimit`, `ee_to_ttx_smlimit` (and the
SM-limit `diagrams` cells) to `gate`: SMEFTsim loaded under
`restrict_SMlimit_massless` must reproduce MadGraph's own SMEFTsim-SM-limit
`MATRIX1` bit-for-bit at the amplitude gate's standard (`|G| = 1`, `Re G =
0`, per-diagram phases). No new primitive is involved: this is the loader,
the MW input scheme, the interaction splitting and zero-coupling pruning
under test — the first non-SM model ever gated end to end, closing the
standing "model-generic on SM evidence alone" gap. Also the first
`IdentityAmp` process cell: `FFS2` (`Identity(2,1)`) with the `cbHRe`-class
card, `b b~ > h` — flips `KNOWN_UNCOVERED` on the SM census only if the
op appears in an SM-gated row, which it does not; it leaves the SMEFTsim
census instead.

### C — capstone: a SMEFT cross section and the CLI path (validation-dev; after E1, F1, R4)

`e+ e- > t t~ NP<=1` at √s = 500 GeV under `restrict_massless` (every class
on: Z-coupling shifts, `ctZ`/`ctA` dipoles, scalar/vector/tensor
four-fermion, `cHDD`/`cHWB` input shifts) — σ vs MadGraph's banked fixed-energy
run at `rel_tol` set by the reference's own error, ≥ 5 seeds and the ladder
discipline (AGENTS.md); `amplitudes` cell for the same row already `gate`.
Then the user path: `vibegraph integrate --ufo-dir research/refs/smeftsim/UFO_models …`
with the restrict-name suffix syntax (`-SMlimit_massless`), a proc card
carrying `NP<=1`, and the artifact's model identity (label + digest) recording
the SMEFTsim model — README scope paragraph updated from "SM only" to what is
now gated.

## 5. Coverage table — every new primitive has a MadGraph-gated row

Rows import the work-area copy of `SMEFTsim_topU3l_MwScheme_UFO` with the
named card; process strings carry explicit orders because the WEIGHTED
default would drop NP diagrams (`NP` hierarchy 99). All start `info` (V1)
and flip in the named session. The dev choosing a cheaper process for the
same primitive must keep the primitive column covered.

| key | process | card (classes on) | primitives gated | flips in |
|---|---|---|---|---|
| `ee_to_mumu_smlimit` | `e+ e- > mu+ mu-` | `SMlimit_massless` | loader, MW scheme, splitting, zero-coupling pruning | L2 |
| `gg_to_ttx_smlimit` | `g g > t t~` | `SMlimit_massless` | as above with colour | L2 |
| `ee_to_ttx_smlimit` | `e+ e- > t t~` | `SMlimit_massless` | massive fermions under the new loader | L2 |
| `bbx_to_h_identity` | `b b~ > h NP<=1` | `cbH` | `Identity` FFS (`IdentityAmp`, the last SM-uncovered op) | L2 |
| `gg_to_h_cpeven` | `g g > h NP<=1` | `cHG` | `P(1,2)*P(2,1) − P·P Metric` VVS | E1 |
| `gg_to_h_cpodd` | `g g > h NP<=1` | `cHG` + `cHGtil` | **`Epsilon` VVS, ε sign via interference** | E1 |
| `ee_to_wpwm_cw` | `e+ e- > W+ W- NP<=1` | `cW` + `cWtil` | VVV with three momenta; `Epsilon` VVV | E1 |
| `gg_to_gg_cg` | `g g > g g NP<=1` | `cG` + `cGtil` | higher-derivative VVVV; `Epsilon` VVVV; `f*f*f` colour | E1 |
| `ee_to_ttx_dipole` | `e+ e- > t t~ NP<=1` | `ctZ` + `ctA` | momentum-slashed γ-chains, `Gamma5` in a chain | E1 |
| `ee_to_zh_smeft` | `e+ e- > Z h NP<=1` | `cHW` + `cHB` + `cHWB` + `cHDD` | derivative VVS, input-scheme shifts | E1 |
| `ee_to_mumu_4f` | `e+ e- > mu+ mu- NP<=1` | `cll1` + `cle` + `cee` | scalar/vector four-fermion, **both pairings in one vertex**, MG's permutation sign | F1 |
| `uux_to_ttx_4f` | `u u~ > t t~ NP<=1` | `cQq11` + `cQq18` + `ctu1` + `ctu8` | four-quark `T·T` / `Identity*Identity` colour | F1 |
| `ee_to_ttx_tensor4f` | `e+ e- > t t~ NP<=1` | `cleQt3` | **cyclic tensor⊗tensor** | R4 |
| `ee_to_ttx_smeft` (capstone) | `e+ e- > t t~ NP<=1` | `massless` (all) | everything above at once; σ | C |
| `wpwm_to_wpwmz_cw` (stretch) | `W+ W- > W+ W- Z NP<=1` | `cW` | a five-vector vertex in a diagram | E1 if cheap, else backlog |

Exact coefficient names are read off `parameters.py` by V1 when the cards are
authored; the class labels above are the physics. **Colour**: no SMEFTsim row
needs a colour atom the engine lacks (§1.2); the toy model carries the rest.

## 6. Track T — the toy UFO for what SMEFTsim leaves unexercised

### T1 — author `vibegraph_toy_UFO` and bank its oracle (validation-dev)

Recommendation (§7 D3): **generate** a minimal, hand-written UFO under
`validation/ufo/vibegraph_toy_UFO/` (MIT, ours) rather than adopt a public
BSM model — the public candidates each bring one structure buried in
hundreds of vertices (`RS` for spin-2, `sextet_diquarks` for `K6`, RPV models
for baryonic ε), none is in the pinned submodule (`models/` there holds only
`sm`, `loop_sm`, `MSSM_SLHA2`, `hgg_plugin`, `taudecay_UFO`), and a
hand-written model isolates each primitive in one vertex with couplings we
choose. MadGraph imports any UFO directory, so the oracle machinery is V1's,
unchanged. Contents:

- particles: a Dirac lepton `l`, a Dirac quark `q` (colour 3), a singlet
  scalar `S`, a singlet vector `V`, an octet scalar `O8`, an antitriplet
  diquark scalar `D3`, a sextet diquark scalar `D6`; all masses free
  parameters.
- Lorentz: `FFV` dipole with a **literal `Sigma`**
  (`Sigma(3,-1,2,-2)*P(-1,3)*ProjM(-2,1)`), an `FFFF` **`Sigma⊗Sigma`**
  (`Sigma(-1,-2,2,1)*Sigma(-1,-2,4,3)`), `FFS Identity`, `FFS Gamma5`
  (cross-checks against SMEFTsim's γγ expansions of the same operators —
  the R1 identity made process-level).
- colour: `d(1,2,3)` (`O8 O8 g`), `Epsilon(1,2,3)` (`q q D3~`), `K6(1,2,3)`
  (`q q D6~`) — the three atoms the engine either has-but-never-exercises
  (`d`) or refuses (`Epsilon`, `K6`).
- Processes banked (diagrams + amplitude tables): `l+ l- > q q~` (dipole and
  `Sigma⊗Sigma`), `q q~ > O8 O8`, `q q > D3 S` (2→2 to keep the colour ε in
  a 2-body final state), `q q > D6 S`.

Cells registered `info`. **Sonnet relief**: the MG runs.

### T2 — literal `Sigma` primitive (feature-dev; after R1, R4)

`Sigma` nodes (`SigmaTout` two fermions → `AsymRank2Tensor`; contraction
with a momentum for the dipole; `Sigma⊗Sigma` through R4's tensor slot),
lowering and kernels over R1's `tensor_bilinear`. Flips the two `l+ l- > q q~`
cells and pins **`Sigma⊗Sigma` ≡ the γγ expansion** at the process level
against MadGraph on both.

### T3 — colour beyond the SM vocabulary (feature-dev; stretch, independent of R/E)

`d(a,b,c)` exercised (engine has it; `q q~ > O8 O8` flips). Then the
representable-but-refused atoms, in the `color-flow` sprint's proven order
(note 16 §3: CF oracle vs MadGraph's `DATA CF` first, JAMP-weighted |M|²
second): `Epsilon/EpsilonBar` (baryonic, `ColorRep` unchanged),
then `Sextet/AntiSextet` reps with `K6/K6Bar/T6` and their
`color_algebra.py` reduction rules and `Identity(6,6̄)` resolution. Flow tags
for sextets (two colour lines on one leg) are `info` until the LHEF layer
decides how to write them. If time runs out, `Epsilon` alone is the
deliverable and sextets return to the `non-sm-ufo` checklist with the CF
oracle already banked.

## 7. Decisions (user)

1. **D1 — SMEFTsim submodule footprint.** Added in this planning commit at
   tag `v3.0.2` (`db7d4a80`), `--depth=1`, 101 MB checked out — the UFO the
   gates read is 0.8 MB of it; the rest is FeynRules sources and notebooks.
   CI's `banked` job checks submodules out (`submodules: true`), so this adds
   ~100 MB to that job's checkout. **Recommended**: keep the full submodule
   (provenance by pin, as asked) and have `vg_ensure_submodule` apply a
   `sparse-checkout` to `UFO_models/SMEFTsim_topU3l_MwScheme_UFO` after
   init, measuring whether `--filter=blob:none` on the submodule update is
   honoured by the pinned git. Alternative: vendor the one directory
   (0.8 MB, MIT) and drop the submodule.
2. **D2 — scope of "the tensor rep".** **Recommended**: general rank-2 +
   antisymmetric (R1) hosting σ^μν, γγ currents and ε contractions;
   **spin-2 external wavefunctions and propagators (UFO spin code 5) stay
   deferred** — the symmetric-traceless polarisation tensors and the
   massive spin-2 propagator are a feature of their own with no test model in
   reach, and `Rank2Tensor` is designed so they slot in later. Spin-3/2 and
   Majorana/`C` are out (Majorana is fermion-flow machinery, its own sprint;
   MadGraph itself refuses Majorana in 4-fermion vertices).
3. **D3 — toy UFO: generate, not adopt** (§6 rationale). Which of the three
   colour atoms are in scope decides T3's size: **recommended** `d` and
   baryonic `Epsilon` in scope, sextets as the stretch.
4. **D4 — squared-order constraints (`NP^2==1`, interference-only |M|²).**
   The grammar parses `^2` and the selector treats it like an amplitude
   order (`selector.rs:43`); implementing MadGraph's per-order |M|²
   splitting is a separate feature. **Recommended**: out of this sprint,
   tracked in the feature backlog; every row above compares the full |M|² at
   `NP<=1` (SM + interference + NP²), which MadGraph computes identically.
5. **D5 — capstone.** **Recommended** `e+ e- > t t~ NP<=1` at 500 GeV under
   `restrict_massless` (fixed energy, no PDFs, every structure class in one
   process). Alternative `p p > t t~` with `ctG` at 13 TeV exercises the
   hadronic path too but costs a full hadronic reference run.

## 8. Sequencing

```
Wave 0 (manager, this commit): S0 submodule + this note + TODO
Wave 1:  R1 (hermetic)  ∥  L1 (loader/splitting)  ∥  V1 (bank the ladder, MG host)
Wave 2:  L2 (SM-limit gate; needs L1+V1)  ∥  E1 (tree-shaped primitives; needs R1, L1, V1)
Wave 3:  F1 (four-fermion; needs L1, V1)  →  R4 (tensor slot; needs R1, E1, F1)
Wave 4:  T1 (toy oracle; needs V1's pipeline)  ∥  C (capstone; needs E1, F1, R4)
Wave 5:  T2 (Sigma; needs R4, T1)  ∥  T3 (colour; needs T1)  →  Z (close-out)
```

- Three sessions open the sprint in parallel and touch disjoint files
  (`repr/`, `ufo/`+`topo.rs`+`diagrams`, `validation/`).
- L2 is the sprint's first flip and needs no new primitive — if it is not
  green, nothing downstream is trusted.
- Z does nothing but bookkeeping: `TODO.md` current position and the
  `non-sm-ufo` checklist rewritten from measurement, README scope, the
  refs README, `KNOWN_UNCOVERED` per model reconciled, this note's close-out.
- Eleven sessions plus close-out; every engine session has its oracle banked
  by V1 or T1 before it starts, and four sessions (R1, L1, V1, T1) are pure
  representation, loader or banking work with no MG-convention risk.

## 9. Risk register

- **Interaction splitting ripples further than `topo.rs`**: diagram
  identity for `diagrams.json`, `derive_flavor_groups`' sampled |M|²
  (unchanged physics, more diagrams), channel maps (more channels of the
  same propagator structure — dedupe), colour-flow tables per member.
  L1 measures the 19 SM rows bit-for-bit as its regression net before any
  SMEFTsim row is looked at.
- **Restriction semantics diverge from MadGraph in a corner**: a coupling
  that is zero under the restriction but non-zero under a later param card
  — MadGraph locks it (note 27 §B5's `apply_restrict` precedent); the split
  must respect the lock.
- **The ε sign and the four-fermion permutation sign are the two convention
  hypotheses most likely to be wrong on first contact**; both have a
  dedicated row whose interference makes the sign visible, and note 12's
  dump methodology is the diagnosis path.
- **Five- and six-leg vertices may expose an arity assumption in
  `colorize` or the channel builder** rather than the rooting; E1 keeps
  `gg_to_gg_cg` (four-leg) as its gate and the five-leg row as stretch.
- **Fusion rewrites (`FfvVout`) silently mishandling a chiral pair with a
  momentum slash** — E1's explicit fused-vs-generic pin on SMEFTsim's FFV
  set exists for this.
- **MadGraph's own handling of SMEFTsim could be the wrong side**: the
  pinned 3.7.x imports SMEFTsim 3.0 as validated by its authors against
  CERN-LPCC-2019-02, but a disagreement is diagnosed before it is
  attributed either way.
