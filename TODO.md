# vibegraph — Task Backlog

## Pipeline Status

| Step | Component | Status | Notes |
|------|-----------|--------|-------|
| 1 | UFO model loading (particles, parameters, couplings, vertices) | ✅ Done | Python AST parser; restrict cards baked into params |
| 2 | Feynman diagram enumeration | ✅ Done | feyngraph + process grammar; validated vs MadGraph |
| 3 | HELAS helicity amplitudes (topology-driven, arbitrary process) | ✅ Done | 11 processes bit-match MadGraph (≤6e-13, incl. 2→6, VVV, massive externals); single color flow only |
| 4 | Phase-space sampling (LIPS + VEGAS) | ✅ Done | Lepage VEGAS + 2-body LIPS |
| 5 | Cross-section integration (e⁺e⁻→μ⁺μ⁻) | ✅ Done | Lepage VEGAS on `AmplitudeEvaluator::eval_m2`; `validate_vegas.rs`: `sigma_z_pole` σ≈2025 pb at √s=91.2 (<0.1% vs MG), `sigma_qed_limit` (√s=10 vs 4πα²/3s, 3%) |
| 6 | Unweighted event output (LHEF) | 🔲 Pending | Accept/reject sampling + Les Houches format |

`helas-generalize` is **done** for single-color-flow processes: `AmplitudeEvaluator`
drives the VEGAS integrand, and `validate_helas_mg` enforces bit-for-bit agreement
with MadGraph across **11 processes** (all ≤6.3e-13): `ee_to_mumu`, `pp_to_ll_qcd0`
(×CF=3), `ee_to_mumu_tata_qcd0`, `uux_to_ccx_emmm_qcd0` (×CF=9), plus the 7
`mg-validation-coverage` additions below (`ee_to_ee`, `ee_to_mumua`, `ee_to_ttx`,
`ee_to_wpwm`, `ee_to_zh`, `ee_to_tatah`, `bbx_to_ccx_emmm_qcd0` ×CF=9). The
three-week continuum bug hunt that got there is written up in
`research/notes/12-helas-continuum-bugfix-journey.md`.

---

## 🧹 Cleanup Refactor (branch `cleanup-refactor`)

Multi-session structural cleanup at the post-`mg-validation-coverage` checkpoint.
Ordered by dependency; each is its own session (task 4 spans several).

### 1. `intern-sm-model` — Intern the SM UFOModel, drop the submodule from the build ✅ Done

The SM model is baked into the binary; `cargo build`/`test` (default suite) never
touch the submodule. Verified: the full default suite passes with the `sm/` submodule
dir moved away, and `ufo::sm::tests::interned_default_matches_submodule` asserts the
interned `SMRestrict::Default` model matches a fresh `UFOModel::load` from the
submodule (particle/vertex keys, lorentz/coupling counts, order hierarchy, and
evaluated masses/couplings).

What landed:
- `serde` `Serialize`/`Deserialize` on the parsed UFO types (`Particle`,
  `LorentzStructure`/`LorentzTerm`/`LorentzOp`, `Coupling`, `Vertex`, `ParameterSet`/
  `Parameter`/`ParamNature`, `Expr`/`BinOp`/`Func`, + the `*Id` newtypes). `topo:
  TopoModel` is **not** serialized — rebuilt by `ParsedModel::into_model`.
- `UFOModel::load` split into serializable `ParsedModel::parse(dir)` (read+parse, no
  prune, no topo) + `ParsedModel::into_model(Option<&ParamCard>)` (bake restrict,
  prune zero-coupling vertices, build topo). `load` = parse + default-card discovery
  + `into_model`.
- Assets under `vibegraph-lib/src/ufo/sm_assets/` (committed): one `sm_parsed.bin.zst`
  (zstd(bincode(ParsedModel)), ~5 KB) + the 9 raw `restrict_*.dat` cards baked via
  `include_str!`. Blob decompressed once (`OnceLock<ParsedModel>`).
- `ufo::sm`: `SMRestrict { Default, CMass, Ckm, LeptonMasses, NoBMass, NoMasses,
  NoTauMass, NoWidths, ZeromassCkm }` + `sm_model(SMRestrict) -> Arc<UFOModel>` with a
  per-variant `OnceLock<Arc<UFOModel>>` cache. `SMRestrict::from_suffix` maps
  `sm-<variant>` directive suffixes.
- Dev bin `vibegraph-lib/src/bin/gen_sm_blob.rs` (`cargo run -p vibegraph-lib --bin
  gen_sm_blob`) → `ufo::sm::regenerate`: reparses the submodule, rewrites the blob +
  cards into the source tree. Only this binary reads the submodule sm dir.
- Deps: `serde`, `bincode`, `zstd` moved to normal deps; `indexmap` `serde` feature.
- In-tree SM callers rewired to the interned model (`tests/common`, the inline test
  mods in `ufo/mod.rs`/`helas/eval/run.rs`/`root_diagram.rs`, `benches/amplitude_eval`,
  and the model load in `validate_madgraph_diagrams`).
- **`global-config`** folded in: `config::GlobalConfig::load_ufo(&Option<ModelImport>)`
  returns the interned SM (no filesystem) for `import model sm[-variant]`, else loads a
  UFO dir from `ufo_search_path`.

Optional follow-up: a CI job that runs `gen_sm_blob` and `git diff --exit-code`s to
catch a stale blob vs the pinned submodule.

_Deps: none. Unblocks: `feature-gate-mg-tests`._

### 2. `feature-gate-mg-tests` — Move MadGraph-dependent tests out of the default suite ✅ Core done

Default `cargo test` now passes on a clean checkout with **all** pixi-generated
validation data absent (verified by moving `validation/madgraph/output` +
`validation/helas/reference.csv` away: 169 lib + integration tests green).

What landed:
- The three inline tests in `helas/eval/run.rs` that read MG data from
  `../validation/madgraph/output/...` — `probe_process_diagrams` (already `#[ignore]`),
  `test_z_current_outgoing_mupair_vs_mg`, `test_espine_eline_z_absorption_ratio_vs_mg`
  (both previously panicked on the gitignored `param_card_masslesstau.dat`) — plus the
  `lf` helper they alone use, are gated with `#[cfg(feature = "extended-validation")]`.
  `cargo test --features extended-validation` still compiles them and the two `_vs_mg`
  tests pass bit-for-bit.
- Gated **inline** rather than physically moved to `tests/`: the `eval` submodules are
  private (`mod run`, `mod diagram_eval`, …) and the tests use private fns
  (`run_forward_slot`, `eval_single_diagram_slot`), so an external `tests/` crate would
  force a large public-API expansion. This matches the existing idiom in
  `validate_helas.rs` (`#[cfg(feature = "extended-validation")] mod extended`).
- `root_diagram.rs` needed no change — its inline tests use the interned model +
  hardcoded reference numbers only (no filesystem reads).

Remaining: fold in **`madgraph-diagram-cmp-per-flavor`** (below) — deferred; it is an
independent, verification-heavy refactor (see that section for the design).

_Deps: `intern-sm-model`._

### 3. `diagram-canonical-stream` — Self-owned diagram structure + streaming API ✅ Owned type done

Part A (the owned diagram type) landed; feyngraph is now used for topology generation only.
Part B (streaming `generate_*` API) is deferred to its own follow-up (see below).

What landed:
- `diagrams::diagram::Diagram` — a UFO-resolved, `feyngraph`-free owned diagram with
  newtype indices (`LegIdx`/`VtxIdx`/`PropIdx`/`RaySlot`), interned `ParticleId`/`VertexId`,
  and rays in UFO interaction-slot order. `Diagram::from_view` is the **single** boundary
  where a feyngraph `DiagramView` is consumed: it bakes the all-incoming crossing, resolves
  particles, runs the `is_anti`-vs-pdg check (now `ConvertError::AntiparticleMismatch`, a
  `diagrams`-module error — no longer `helas::eval`'s), and carries each propagator's
  **momentum** (signed external combination, `endpoints[0]→[1]`), giving each `Ray` a
  natural momentum-flow direction.
- `DiagramSet.diagrams` is now `Vec<Diagram>`; `generate_sets_inner` converts the feyngraph
  container at the module boundary and drops it. No feyngraph view escapes `diagrams/`.
- `helas/eval/root_diagram.rs` is fully `feyngraph`-free: walk/bake/`root_tree`/compile and
  both test oracles (`trace_fermion_line`, `initial_state_spine_sign`) operate on the owned
  `Diagram`. `RootDiagramError` shrank to just `ExternalLegAsResult` (resolution moved to the
  boundary). Regression: default suite (169) green, `validate_helas_mg` 11/11 bit-for-bit,
  `validate_madgraph_diagrams` 16/16, Fortran HELAS cross-check green.

Momentum-sign rewire done: `t_channel` now reads the baked propagator momentum
(`Prop::is_spacelike(n_in)` — spacelike iff exactly one of the two beam coefficients is
nonzero), replacing the `n_inc` subtree-incoming-count threading (dropped from `bake_node`);
each leg's `incoming`/`crossed` now reads the baked `Leg.incoming` (momentum-flow direction)
instead of recomputing `leg_idx < n_in`. Bit-for-bit preserved: all 11 `validate_helas_mg`
processes unchanged (incl. Bhabha t-channel Z width and the `bbx` lowered propagator), 169
default tests + `spine_sign_from_flow_matches_heuristic` green.

Deferred — **Part B (`generate-stream`)**: make `generate_*` return
`impl Iterator<Item = Result<DiagramSet, DiagramError>>` (yield subprocesses lazily; keep
per-spec WEIGHTED discovery eager) so the whole card's subprocesses aren't materialized at
once. `feyngraph-perf` below is adjacent but a submodule change.

_Deps: none; done before task 4 so the typed-convention work sits on a clean base._

### 4. `typed-repr-conventions` — Make all wavefunction/Lorentz conventions explicit in types

**Status (2026-07-09, `cleanup-refactor`): terminology rename + typed propagator seam + Stage 0
property-test harness + kernel factor-out (1-1 `Op`↔`kernel.rs`) + Stage B (one contravariant
vector convention, `VectorCo` deleted) DONE; only Stage A (fuse) remains, deferred pending
profiling.** Landed:
- §7 step 1 terminology rename `SpinorFlow`→`DiracAdjoint`/`Ket`/`Bra`, runtime `Flow`→`Adjoint`
  (`84bd2d3`).
- `VectorWf` variance-parameterized (`aed1a80`).
- **The typed propagator seam (`402322c`):** `WaveformSlot::VectorCo(VectorWf<F, Covariant>)`
  carries the index-lowered off-shell vector currents; the propagator dispatches on slot
  variance and raises them back (massive branch). Removed `Op::PropagateLowered`,
  `PropInfo.lowered_storage` + its computation, `has_flipped_vector_out`, and the manual
  component-poking in `metric_vout`/`lower_vout`/`propagate_core`. 169 lib + validate_helas_mg
  11/11 bit-for-bit.

This delivered the type-safety + bool-elimination goal (variance now lives on the slot at the
duality boundary). Findings that revise the remaining plan: (1) don't naively rebalance the pinned
−i/+i V-vs-S chain-phase split; (2) the propagator is not a clean musical iso — the massive
covariant branch raises the lowered index in place, the massless branch defers the raise to the
downstream contraction — but BOTH sides are MG-validated (`ee_to_wpwm` roots its s-channel photon
off the WWγ vertex → `LowerVout` → massless photon propagator, exercising the massless covariant
branch bit-for-bit; `mu+ mu- > w+ w-` hits the same path). See `research/notes/13` §7 and memory
`lorentz-eval-node-refactor`.

**Staged plan — harness + kernel factor-out + Stage B done; Stage A (fuse) deferred
pending profiling.**

_Design decision (2026-07-09): **drop the two-level `LorentzEvalNode` enum** — nested enums carry
two discriminants (larger/cache-worse than the flat dataless `Op`), and the variance/adjoint it was
to carry already live on the `WaveformSlot` (`Vector`/`VectorCo`, `FermionIn`/`FermionOut`) after the
seam work. The "output type" axis stays a naming discipline on the flat `Op` set. See note 13 §1b
REVISION + §7._

- **Stage 0 — shared property-test harness (§7 step 3) ✅ Done.** `helas/eval/prop_harness.rs`
  (`#[cfg(test)]`, `pub(crate)`): typed random-input generators (`rand_ket`/`rand_bra`,
  `rand_vector`/`rand_vector_co` at each `Variance`, `rand_scalar`/`rand_real`, `rand_momentum`,
  plus `rand_c` component-level for bespoke inputs) + a variant-strict `slots_approx_eq` comparator
  (compares every stored component *and* routed momentum) + the `check_agree(n, seed, tol, gen,
  lhs, rhs)` driver = the "evaluate two kernels on the same random inputs and compare
  `WaveformSlot`s" core. Inputs are deliberately off-shell/EOM-violating (the identities are
  algebraic). 5 self-tests (reflexivity, variant-mismatch/perturbation rejection, driver
  pass + `#[should_panic]` disagreement) + a `run.rs` smoke test driving the real private
  `metric_contract` kernel (bilinear symmetry). 175 lib tests green (was 169). Both goals below
  layer on this; they are two *different* equivalence relations sharing this one toolbox.
- **Kernel factor-out + 1-1 `Op` naming ✅ Done** (`cleanup-refactor`). Moved the Lorentz-primitive
  kernels out of `run.rs` into `helas/eval/kernel.rs` (`pub(crate)`); every Lorentz `Op` now maps
  1-to-1 to a kernel fn named for it, so `run::apply` collapses to a match of `kernel::<op>(children)`.
  The N-to-1 cases got thin per-Op wrappers over shared private helpers (`gamma_iout`/`gamma_oout`→
  `off_shell_fermion_current`; `proj_m`/`proj_p`→`chiral_project`; `proj_m_amp`/`proj_p_amp`/
  `identity_amp`→`scalar_bilinear_current`; `metric_neg_i`; `pmom`/`pmom_out`; `propagate`→
  `propagate_core`); `metric_contract`→`metric`. Structural/const/algebraic ops stay inline in `apply`.
  Pure structural → **bit-for-bit** (175 lib + `validate_helas_mg` 11/11, max_rel_diff ≤ 6.25e-13).
  **Perf baseline** (`benches/amplitude_eval`, ee→μμ, release, N=10k): ~14.0 µs/eval (139.8–142.2
  ms/10k, this machine) — feeds the Stage-A profiling decision.
- **Stage B ✅ DONE (2026-07-09, `cleanup-refactor` d537e1e/3c60fe0/5c95d5b): one physical
  contravariant vector convention; `WaveformSlot::VectorCo` deleted.** The corrected-scope
  fear (a global amplitude-load-bearing rewrite) dissolved once the composites were checked
  *with the right producer phases*: `propagate(metric_vout(v))` ≡ `propagate(Vector(+v))` and
  `propagate(lower_vout(v), m>0)` ≡ `propagate(Vector(−v), m>0)` are IEEE-exact sign shuffles
  (certified bit-exact on 10k random samples via the Stage-0 harness before rewiring). The one
  genuinely inequivalent case — the massless `VectorCo` branch's relabel-vs-raise — differs only
  in the propagated current's *time component*, and every validation point multiplies it by an
  exactly-zero conserved sink current J⁰ (CM s-channel, massless fermions; verified on real node
  dumps: both J⁰ and the assembled VVV current's time component are exactly 0.0). New convention:
  producers emit the physical contravariant current with a momentum-grade sign (+1 P-less VVS,
  −1 P-carrying VVV), the UFO coupling carries the vertex i, the propagator its −i; `MetricVout`
  is now an identity, `LowerVout` a negation, 4 vector-prop branches → 2. **Verified per-diagram:**
  ee_to_zh / ee_to_wpwm / ee_to_tatah [diagram × helicity] dumps bit-identical to the pre-Stage-B
  baseline (ee_to_zh got its own MG AMP probe, registered in build_amplitude.sh); 175 lib +
  `validate_helas_mg` 11/11 with rel_diffs unchanged digit-for-digit. Perf re-checkpoint:
  ~14.1 µs/eval (unchanged; ee→μμ exercises no VVS/VVV chain). NOTE for future rooting work:
  the massless-vector chain equality leans on sink-current conservation at CM s-channel points —
  a t-channel massless VVV rooting (not in the current 11-process net) would see the (physical,
  now-correct) time component; the old relabel convention was only ever valid on that corner.
- **Stage A (own session, DEFERRED pending profiling) — `fused == generic`.** Peephole/
  instruction-selection layer (note 13 §7 step 5a) + coordinate read-off (FFV → `[g_L,g_R]`, …),
  oracle = the generic path. This is a *performance* play, so **profile `run.rs` first** — the
  forward-pass `Vec` churn, `C<F>`-vs-`F` multiply, and arena traversal may dominate kernel-fusion
  wins. Pursue only if fusion proves the bottleneck. (No two-level node scaffolding — dropped.)

The big one. Supersedes `lorentz-eval-node-2level` (below) and absorbs the note-11
`Flow`-into-`repr` move. Every convention bug in the note-12 hunt lived at a hand-coded
duality boundary (flow, crossing, variance).

- **Design stage first**: pick the call formalism (HELAS F/I/O/V/S call style vs. the
  `intertwiner` abstraction) so hand-written diagram evals map 1:1 to the s-expression
  language, and each call is typed so it cannot be misapplied. Success: run.rs's
  evaluator just unpacks the `WaveformSlot`, dispatches typed calls, and repacks.
- Catalog every vertex- and wavefunction-altering call (`ProjM`/`ProjP`, `Gamma*out`,
  `Metric*`, `P`, propagators) in that one formalism. Note flow is a spin-independent
  property fixed by the sign of the free-EOM exponential — split responsibilities:
  variance/metric at `helas::repr::lorentz`, flow at `helas::wavefn`.
- **Terminology fix (do first):** what is currently named `Flow`/`SpinorFlow`/
  `FlowIn`/`FlowOut` (`repr/lorentz.rs`) and `Flow`/`LegFlow` (`root_lorentz.rs`) is
  actually the **bra/unbar (Dirac-adjoint) duality** — ket (u/v columns) ↔ bra
  (ū/v̄ rows), iso `bar()` = ψ†γ⁰ — which is **spinor-only**. Three orthogonal axes:
    - **Variance** (index up/down, metric g) — vectors/tensors — symmetric musical iso.
    - **bra/unbar** (Dirac adjoint) — spinors only — Hermitian musical iso. *This* is
      the one that belongs next to `Variance` (note 11's placement is right), but
      **rename** it off "Flow": e.g. trait `SpinorAdjoint` with sides `Ket`/`Bra`
      (parallel to `Contravariant`/`Covariant`). Rename runtime `Flow`/`LegFlow` too.
    - **Flow** (in/out, sign of e^{∓ipx}, HELAS nsf/nss/nsv) — **all** wavefunctions,
      currently has no dedicated type (scattered across `Charge`/`crossed`/momentum
      signs). Reserve the name "Flow" for *this*; it lives at `wavefn`, is NOT next to
      Variance, and is not a musical iso.
    - Invariant to enforce: for spinors bra/ket is *derived* from (Flow) ⊕ (Charge)
      via the rooting-chosen fermion-arrow, with `crossed` recording the mismatch vs
      physical momentum. Flow and Charge are the independent inputs; bra/ket is not a
      free fourth axis.
- Implementation: ~~two-level `LorentzEvalNode`~~ **SUPERSEDED — see the revised staged plan
  above and note 13 §1b REVISION.** The two-level enum is dropped (layout cost + redundant with
  slot-carried variance/adjoint). What actually landed: variance-parameterized `VectorWf` +
  `WaveformSlot::VectorCo` (variance on the register) and a variance-dispatching propagator that
  killed `Op::PropagateLowered`. Remaining structure work is the flat kernel factor-out (NEXT) and
  the Stage B convention simplification, not a node-level restructure.
- Regression net: the 11 MG-validated processes (task 2 keeps them fast).
- **Design of record: `research/notes/13-typed-repr-conventions-design.md`** — general typed IR
  + peephole/instruction-selection rewrite layer, per-kernel `fused==generic` property tests,
  the §2b intertwiner-basis catalog, and the three-axis terminology fix. Supersedes the input
  notes `08`/`09`/`10`/`11` as design input. Scope decision (i): land the typed general SM node
  set + peephole + property-test harness now; leave `ε`/`σ`/high-grade (dim-8 EFT) as a
  documented, un-implemented extension point in the general tier.

_Deps: benefits from `diagram-canonical-stream` (clean rooting) + `feature-gate-mg-tests`
(fast regression). Design stage before any code._

---

## 🔴 High — broaden the MadGraph amplitude validation surface

### `mg-validation-coverage` — New processes for `validate_helas_mg` ✅ Done

All 7 single-flow processes are enforced bit-for-bit in `validate_helas_mg`
(≤6.3e-13); `u u~ > u u~` (#8) remains blocked on color flow. Each added exactly one
convention axis; the fixes each landed with a per-diagram AMP-dump cross-check:

1. **`ee_to_ee` (Bhabha)** — s⊕t interference with identical flavors. Needed the
   crossed-line `−1` (s-channel has one crossed line, t-channel none) and ZERO width
   on the t-channel Z (MadGraph passes ZERO for spacelike propagators). 2.7e-14.
2. **`ee_to_mumua`** — first external vector wavefunction (`vxxxxx`) vs MG. Required
   the massless-vector 3→2 helicity fix (`[-1,0,1]` → `[-1,1]`). 3.9e-13.
3. **`ee_to_ttx`** — massive external fermions. 4.8e-15.
4. **`ee_to_wpwm`** — VVV triple-gauge + massive charged vector externals + t-channel
   ν. Needed `LowerVout` (VVV's P-carrying structure lowers its output index without
   the vertex −i, vs VVS's `MetricVout`). 4.4e-14.
5. **`ee_to_zh`** — external scalar + on-shell VVS + massive s-channel Z propagator.
   9.5e-14.
6. **`ee_to_tatah`** — external FFS Yukawa emission. Needed goldstone/ghost exclusion
   in unitary gauge (`is_goldstone` + `ghost_number` filter in `topo.rs`). 3.9e-13.
7. **`bbx_to_ccx_emmm_qcd0`** — 2-propagator spine with massive internal fermions +
   massive-vector propagators fed by index-flipped (VVS `MetricVout`) currents. Needed
   the `PropagateLowered` op: the massive-vector longitudinal term reads its `g^{μν}`
   term off the raised current and undoes the `MetricVout` storage sign. 6.3e-14.

Infra delivered with this work (the reusable-scripts request):
- `wrappers/generic.f` — one Fortran wrapper (`MG_EVAL_M2` / `..._BATCH`) for all
  momentum-CSV processes; `TS` sized `3**NEXTERNAL` for massive-vector helicities.
  `ee`/`pp_to_ll` migrated off their bespoke wrappers.
- `gen_amplitude.py` — registry-driven (`Process` dataclass + `PROCESSES` list),
  massive RAMBO (Newton ξ-rescale), momentum-based CSV schema shared by all.
- `build_amplitude.sh` — registry-driven (`GENERIC_PROCESSES`, `AMP_PROBE_PROCESSES`);
  `subprocess_dir()` glob helper.
- `wrappers/amp_probe.f.in` + `compare_amps.py` — one process-parameterized
  per-diagram AMP-dump probe + matcher, replacing the two bespoke note-12 probes.
  Driven by the `probe_process_diagrams` Rust test (`VG_PROBE_NAME`/`VG_PROBE_CF`).

### `color-flow` — Multi-flow color algebra

For NCOLOR=1 processes the scalar color factor `CF(1,1)` suffices (implemented in
`validate_helas_mg::color_factor`). True multi-flow color (same-flavor `u u~ > u u~`,
gluon exchange) needs per-flow amplitudes and the color matrix. Prerequisite for
hadronic cross sections and any QCD≠0 validation.

_Unblocks: `mg-validation-coverage` #8, PDF-weighted pp→ll σ_

### Hadronic pp→ll cross section (after color flow)

σ = Σ_q ∫ dx₁ dx₂ f_q(x₁) f_q̄(x₂) σ̂(q q̄ → l⁺ l⁻). Blocked on: color flow, a PDF
interface (e.g. LHAPDF), and n-body LIPS for the partonic √ŝ scan. Flavors group by
charge type since MG treats light quarks as massless.

---

## 🟡 Medium — CLI integration

### `global-config` — Implement `vibegraph_lib::config::GlobalConfig` ✅ Done

_Folded into cleanup task 1 `intern-sm-model` (same model-loading wiring)._ Landed as
`config::GlobalConfig::load_ufo(&Option<ModelImport>) -> Arc<UFOModel>`: interned SM
for `import model sm[-variant]`, else a UFO dir under `ufo_search_path`. CLI wiring of
a full proc card is still pending.

A thin coordinator that wires `ParsedProcCard` → `UFOModel` loading for the CLI.

```rust
pub struct GlobalConfig {
    pub ufo_search_path: PathBuf,
    pub restrict_path_override: Option<PathBuf>,
}
impl GlobalConfig {
    pub fn load_ufo(&self, spec: &Option<ModelImport>) -> Result<UFOModel, UfoError> { ... }
}
```

_Depends on: `helas-generalize` (✅)_
_Unblocks: Full CLI with process cards_

---

## 🟢 Later — polish and extensibility

### `lorentz-eval-node-2level` — Two-level LorentzEvalNode + variance-aware slots

_Superseded by cleanup task 4 `typed-repr-conventions` (kept here for the detailed
motivation)._

Reorganize the Lorentz eval nodes (`helas/eval/root_lorentz.rs` + `run.rs`) into two
levels:
- **outer = output type**, carrying variance/flow: `ScalarOut` / `VectorOut<V>` /
  `SpinorOut<Flow>` (+ tensor later);
- **inner = the UFO primitive** (Gamma, Metric, P, Proj, …), with one node per
  (structure × distinct output-leg type) — e.g. `Metric`→scalar vs `MetricVout`
  →vector are the same structure with different outputs.

Make `VectorWf` (and `WaveformSlot`) variance-parameterized so each vertex returns its
natural variance, the propagator's raise/lower is type-checked, and contractions can't
silently double-apply or drop the metric. This removes the manual component/index
hand-coding in `MetricVout`/`LowerVout` and the `PropagateLowered` branch (`run.rs`),
which bypass the typed repr because `VectorWf.eps` is pinned to `Contravariant` — the
exact variance-bug class the typing was meant to prevent. Note 12's lesson 10 is the
motivation: every convention bug in the hunt lived at a duality boundary (flow,
crossing, variance) that was hand-coded — and the `bbx` lowered-propagator fix added
another `lowered_storage` flag + hand-raised `g^{μν}` term to that pile. The
flow-typed `WaveformSlot` design this builds on is written up in
`research/notes/11-variance-flow-duality.md`.

**Now unblocked**: the continuum bug is fixed, VVS is regression-pinned
(`test_metric_vout_vs_aloha_vvs1p1n1`), and VVV + lowered-vector propagation are now
MG-validated (`ee_to_wpwm`, `bbx_to_ccx_emmm_qcd0`). This refactor is the natural next
structural cleanup: it would collapse `Metric`/`MetricNegI`/`MetricVout`/`LowerVout`
and `Propagate`/`PropagateLowered` into variance-typed nodes.

### `feyngraph-perf` — Fix feyngraph allocation hot spot

**Hot spot identified** (samply profile, pp→qq̃4l run): `workspace.rs:L122` in
`AssignWorkspace::assign()` calls `.counts()` (itertools) on every candidate vertex for every
topology for every subprocess. Each `.counts()` call allocates a fresh `HashMap<particle_index,
count>`. For pp→qq̃4l: ~1,664 subprocesses × 34,300 topologies × O(vertices) = ~340M HashMap
allocations. **Fix**: pre-compute per-vertex particle counts in `AssignWorkspace::new()` and
reuse them in the inner loop. This is a change to the `feyngraph` submodule; deferred to a
dedicated feyngraph session.

Vibegraph-side mitigations already applied:
- Topology caching: `generate_topologies()` called once per `(n_ext, n_loops)`; all subprocesses
  share the same `Vec<Topology>` via `DiagramGenerator::assign_topologies()` (pp→qq̃4l: 4.86s once
  vs ~15h naive).
- Charge conservation pre-filter: eliminates ~86% of alias-expanded candidates before topology
  assignment (11,520 → ~1,664 for pp→qq̃4l).

### `madgraph-diagram-cmp-per-flavor` — Match subprocesses by flavor in diagram validation

_Was to fold into cleanup task 2 `feature-gate-mg-tests`; deferred as its own session —
it is an independent, verification-heavy refactor (Python extractor + Rust matching +
JSON regen), not part of the feature-gating itself._

The `validate_madgraph_diagrams` reference count now uses the representative subprocess's
true Feynman-diagram count (`NGRAPHS` from `matrix1_orig.f`), not `MAPCONFIG(0)` from
`configs.inc` (which counts the phase-space integration-channel *union* across all flavor
variants in a P-class — e.g. 2672 vs the actual 2316 for `u u~ > u u~ l+ l- l+ l-`).

**Remaining gap**: the comparison (`count_mg_style_topologies` in
`vibegraph-lib/tests/validate_madgraph_diagrams.rs`) still collapses vibegraph subprocesses
into coarse particle-type classes (`quark`/`lepton`/…) and compares one representative per
class against the summed `total_diagrams`. Fragile: it assumes vibegraph's first-enumerated
subprocess in each class matches MadGraph's `matrix1` representative.

**Design for the refinement** (per-flavor matching, validates *all* variants incl. the 40
of the qq4l class):
- **Robust flavor source — the matrix-file header, not `IDUP`.** Each
  `SubProcesses/P*/matrix<N>_orig.f` carries `C     Process: u u~ > u u~ e+ e- e+ e- QCD=0 @1`
  comment lines — one per concrete flavor process sharing that variant's `NGRAPHS` (u/c and
  e/mu are grouped). Parse these directly: it avoids reverse-engineering MG's fragile
  `matrix<N> ↔ IDUP(I,J,K)` 3-index mapping in `leshouche.inc`. `extract_diagrams.py` grows
  a per-concrete-process `{in:[pdg…], out:[pdg…], ngraphs}` list (name→PDG via a bounded SM
  dict: the full token set is `a b b~ c c~ d d~ e± g h mu± s s~ t t~ ta± u u~ w± z`).
- **Rust side**: key each MG entry and each vibegraph subprocess by
  `(sorted initial PDGs, sorted final PDGs)`; look up and compare per-subprocess
  (`set.diagrams.len()` vs `ngraphs`).
- **Known risk to resolve first**: this exposes whether vibegraph enumerates the *same set*
  of concrete subprocesses as MG's `C Process:` union — i.e. whether the multiparticle `p`/`l`
  definitions and flavor-symmetry pruning align. Validate on a small process (`pp_to_ll`)
  before the qq4l class; a set mismatch here is a real finding, not a test bug, and needs
  physics judgment (note-12 territory: MG-convention reconciliation is a bug magnet).

### `lips-nbody` — n-body LIPS phase-space generator

Generalize phase-space sampling to 3+ final-state particles using recursive 2-body
decomposition (RAMBO-style). Research Rust options before committing to an approach.
(The MG validation side already generates n-body points via RAMBO in
`gen_amplitude.py`; the MG-computed partonic σ̂ = 6.556e-7 pb for the uux 2→6 at
√s=500 is banked as a future `validate-vegas` reference.)

_Depends on: `xsec-ee-mumu` (✅)_

### `event-output-lhef` — Unweighted events in LHEF format

Accept/reject sampling with `w(p) = |M(p)|²/w_max`; serialize to Les Houches Event File
format for downstream tools (Pythia, Herwig, etc.).

_Depends on: `helas-generalize` (✅)_

### `typed-units` — Typed physical units

Research `uom`/`dimensioned`/`units` crates for typed four-momenta and cross sections.

---

## Dependency graph

```
feyngraph-ufo-replace (✅) ──→ lorentz-runtime-eval (✅) ──→ helas-generalize (✅) ──→ event-output-lhef
lorentz-parse (✅) ──────────────────────────────────────┘              │
diagram-enum (✅) ──────────────────────────────────────────────────────┤
color-flow ──→ mg-validation-coverage #8, hadronic pp→ll               │
lips-nbody ─────────────────────────────────────────────────────────────┴──→ event-output-lhef

Cleanup (branch cleanup-refactor):
intern-sm-model (+global-config) ──→ feature-gate-mg-tests (+mg-diagram-cmp-per-flavor)
diagram-canonical-stream ──┐
feature-gate-mg-tests ─────┴──→ typed-repr-conventions (supersedes lorentz-eval-node-2level)
```
