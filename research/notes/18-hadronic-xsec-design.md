# 18 — Hadronic cross section: design + session plan

Sprint goal: the first hadron-collider observable —

```
σ(pp → l⁺l⁻) = Σ_q ∫ dx₁ dx₂ [ f_q(x₁,μF) f_q̄(x₂,μF) + (x₁ ↔ x₂) ] σ̂_qq̄→l⁺l⁻(ŝ = x₁x₂s)
```

validated against a MadGraph run with the same PDF set, scale, and cuts. Three
infrastructure pieces ride along because they share the seam this formula opens:

1. **PDF module** — LHAPDF6 grid files + bicubic spline interpolation in
   (ln x, ln Q²), pure Rust.
2. **`F: Real` phase-space generality** — massive RAMBO generic over the scalar
   field (first stage of `lips-nbody`), fed by a splittable counter-based RNG
   whose bit-stream is defined independently of `F`. This is the substrate for
   SIMD lane-batching of `eval_m2` via `numeric-array`.
3. **VEGAS as a two-phase serializable object** — an *adapt* phase that saves the
   trained grid, and a *sample* phase that loads a frozen grid; rayon-parallel
   accumulation with deterministic chunked reduction. Endgame (later sprints): CLI
   step 1 computes σ and saves the grid, CLI step 2 fans out across machines to
   generate LHE samples (`event-output-lhef`).

Non-goals (deferred, see §6): multi-channel / propagator-pole channel mappings,
n-body *hadronic* processes, LHE output, α_s running, PDF uncertainty members,
b-quark / 5-flavor scheme.

## 1. Research findings

### 1.1 LHAPDF interpolation is a solved, small problem

[DavidMStraub/parton](https://github.com/DavidMStraub/parton) reimplements LHAPDF
member evaluation in ~200 lines of Python: parse the `.dat` member file
(subgrid blocks of x-knots, Q-knots, flavor list, x·f values), build one
`scipy.interpolate.RectBivariateSpline` per (subgrid, flavor) over
`(ln x, ln Q²) → x·f` with scipy's default cubic degrees (kx=ky=3, s=0
interpolating), and evaluate `xfxQ2` by walking subgrids until a non-NaN value
fills in (subgrids partition the Q² range at flavor thresholds). Flavor 0
aliases gluon 21. `AlphaS_*` metadata in the `.info` file supports α_s(Q)
interpolation — **not needed at LO for Drell–Yan** (pure EW couplings from the
param card), so we parse but do not consume it this sprint.

### 1.2 Rust spline options — trial `scirs2-interpolate`, own the fallback

- [`scirs2-interpolate`](https://docs.rs/scirs2-interpolate) 0.6.1 exports a
  scipy-compatible `RectBivariateSpline` (plus a `BivariateSpline` trait). It is
  the only drop-in candidate. Risks: it drags `scirs2-core`/`scirs2-linalg`/
  `scirs2-spatial`, and the project has a machine-generated flavor — correctness
  must be *demonstrated against the parton oracle, not assumed*.
- `splinify` (Dierckx FFI-style port) is univariate-only; `splines`/`bspline`
  are curve libraries, not rectangular-grid surface fitters. No other credible
  candidate.
- **Fallback (own it)**: for the interpolating case (s=0) on a rectangular grid,
  bicubic B-spline construction is two 1-D problems — not-a-knot cubic spline
  coefficient solves (banded, O(n)) applied per-row then per-column — a bounded
  ~300-line implementation with textbook test cases. No Dierckx `regrid`/`surfit`
  smoothing machinery is needed.
- **No LHAPDF FFI** unless both paths fail: it is a C++ dependency wall, and
  parton proves the format doesn't warrant it.

Decision rule (H2 executes it): put evaluation behind our own minimal trait
(`fn xfx_q2(&self, pdg: i32, x: F, q2: F) -> F`), trial scirs2 first; adopt it
only if it meets the parton oracle at ≤1e-9 relative on interior points *and*
adds acceptable compile weight; otherwise write the fallback. Either way the
public API never exposes the backend.

### 1.3 `numeric-array` fits the existing `Real` seam

`AmplitudeEvaluator<F>` and the whole repr layer are already generic over
`Real` (`num_traits::Float + ConstZero + ConstOne + FloatConst + Copy + Debug`,
`helas/repr/mod.rs:60`). [`numeric-array`](https://docs.rs/numeric-array) 0.6.1
(wrapper over `generic-array`) implements `Float`, `FloatConst`, `FloatCore`,
`Num`, `NumCast` **elementwise**, and is explicitly designed so LLVM
auto-vectorizes fixed-size elementwise ops into SIMD. So `F = NumericArray<f64, N>`
lane-batches N phase-space points through one `eval_m2` call with zero evaluator
changes — *if* two hazards are cleared:

- **Trait bounds**: `ConstZero`/`ConstOne` may be missing on `NumericArray`; if
  so, either relax `Real` to `Zero + One` (touches literal-construction sites) or
  wrap in a local newtype implementing the missing consts. H4 decides.
- **Lane divergence**: any *data-dependent branch* on `F` inside kernels
  (rest-frame / pT=0 special cases in external wavefunctions, mass-vs-massless
  propagator forks) evaluates one branch for the whole lane pack. Elementwise
  `Float` comparisons on arrays reduce to a single bool (lexicographic
  `PartialOrd`), which silently computes the wrong branch for mixed lanes. Audit
  required. Mitigations, in preference order: (a) show the branch predicate is
  *lane-uniform by construction* — beams are exactly along ±z for every lane
  (partonic CM), external masses are compile-time constants, so the pT=0 and
  m=0 forks are uniform; (b) branchless rewrite; (c) exclude the op from the
  batched path and document. Note the existing constraint that pruned `eval_m2`
  already assumes z-beam momenta — the hadronic integrand satisfies it natively.
- **Bit-exactness is achievable and is the gate**: elementwise arrays execute the
  same fp operation sequence per lane as the scalar evaluator (no reassociation),
  so each extracted lane must be bit-identical to the scalar `eval_m2` result.

Hardware note: the dev machine is Apple Silicon (NEON, 128-bit ⇒ 2×f64);
AVX2/AVX-512 machines want 4–8×f64. Width is a benchmark output (H4), not a
design assumption; the batch API takes `N` as a const generic.

### 1.4 RNG — splittable/modern, not RANMAR

RANMAR (CERNLIB's classic, MG-Fortran-compatible) was considered and **rejected**:
its only unique asset is legacy cross-toolchain seed compatibility, which we don't
need (see below), and modern generators dominate it on statistical quality,
splitting discipline, and ecosystem support. Decision:

- **Default: `rand_chacha::ChaCha8Rng`** — counter-based, so substreams are
  *structurally* independent rather than hash-hopeful: 2⁶⁴ selectable streams
  (`set_stream`) plus a settable position (`set_word_pos`). Map the two-phase /
  distributed design onto it directly: stream ← (iteration, chunk/shard index),
  position ← draw counter. Reproducible across platforms; one new small
  dependency (`rand_chacha` 0.9, same `rand_core` ecosystem as our `rand` 0.9).
- **Alternative if the RNG ever shows in a profile: `rand_pcg`** (`Pcg64Mcg`) —
  faster, tiny state, same `SeedableRng`+`RngCore` surface, but stream selection
  is weaker than a counter-based design; keep it behind the same seam. Phase-space
  cost is dominated by `eval_m2` by orders of magnitude, so quality/splitting
  wins over raw RNG speed by default.
- **`F`-portability lives in the conversion, not the generator**: the stream is
  defined on integer bits; a documented bits→uniform rule (u64 → 53-bit f64
  mantissa, then `F` cast) makes lane-batched sampling bit-identical to scalar
  for `f64` lanes automatically. If f32 sampling ever matters, a 24-bit
  conversion rule is a one-line variant — a property of the adapter, not the RNG.

RAMBO validation never needed shared RNG streams anyway: our reference generator
(`validation/madgraph/gen_amplitude.py::rambo`, numpy `Generator`) validates the
*deterministic map* (uniforms in → momenta out) — dump (uniforms, momenta, weight)
tuples from the Python reference and replay the same uniforms through the Rust
implementation.

### 1.5 MadGraph run-card cuts — the inventory to parse, and what must actually cut

To agree with an MG cross section we must apply the same phase-space cuts MG
applies **by default** — silently comparing an uncut σ against MG's cut one is a
guaranteed mismatch (default `ptl = 10`, `etal = 2.5`, `drll = 0.4` are *active*
out of the box). Inventory from `RunCardLO.default_setup`
(`refs/mg5amcnlo/madgraph/various/banner.py:4208`, params tagged `cut=`), file
syntax `<value> = <name> ! comment`:

- **ŝ window**: `dsqrt_shat` / `dsqrt_shatmax` (defaults 0 / −1 = off).
- **Single-leg, per class** (j=light jet incl. gluon per `maxjetflavor`, b, a=γ,
  l=charged lepton, n=ν missing-Et, H=heavy >10 GeV): `pt{j,b,a,l}` +
  `pt{…}max`, `misset`/`missetmax`, `ptheavy`, energy `e{j,b,a,l}`(+max,
  hidden), pseudorapidity `eta{j,b,a,l}` + `eta{…}min`. Active defaults:
  `ptj=20, pta=10, ptl=10, etaj=5, etaa=2.5, etal=2.5`.
- **Pairwise**: ΔR min/max `dr{jj,bb,ll,aa,bj,aj,jl,ab,bl,al}(max)` (active
  defaults: `drjj=drll=draa=draj=drjl=dral=0.4`), pair invariant mass
  `mm{jj,bb,aa,ll}(max)`, incl.-ν pair `mmnl(max)`, dilepton-system pT
  `ptllmin/ptllmax`.
- **Ordered/inclusive families** (all inactive by default): leading-object
  `ptj1..4`/`ptl1..4` min/max, inclusive `xpt{j,b,a,l}`, HT sums
  (`htjmin/max`, `ihtmin/max`, `ht2..4min/max`), photon isolation (`ptgmin`,
  `r0gamma`, `xn`, `epsgamma`, `isoem`), VBF (`xetamin`, `deltaeta`), merging
  (`ktdurham`, `dparameter`, `ptlund`, `xqcut`), per-PDG dict cuts
  (`pt_min_pdg`, `eta_min_pdg`, `eta_max_pdg`, …), `cut_decays`, `bwcutoff`.

The enforced implementation reference is **`SubProcesses/cuts.f`** in generated
output (not the Python layer): it fixes the conventions — which η (pseudorapidity;
≡ rapidity for our massless legs, but pin it), ΔR = √(Δη²+Δφ²), which legs each
letter class matches. Classification hazard: `maxjetflavor` (default 4) decides
b-vs-jet membership. For LO pp→e⁺e⁻ only the `l`-class and `ll`-pair cuts can
fire, but the parser must cover the full inventory to *detect* everything else
(§2.6).

### 1.6 Current code state (what the sprint builds on)

- `phasespace.rs`: massless-only, `f64`-hardcoded `rambo_massless`, 2-body LIPS
  helpers, `GEV2_TO_PB`. Becomes `phasespace/` module tree.
- `vegas.rs`: classic Lepage VEGAS, grid `xi: Vec<Vec<f64>>` private, single
  `integrate()` entry, `FnMut(&[f64]) -> f64` integrand, no serde.
- `AmplitudeEvaluator::<F>::eval_m2` fully generic; helicity-expanded single
  pass; MATRIX1 convention (summed, not averaged).
- CLI is a 3-line stub; `GlobalConfig::load_ufo` seam exists for proc cards.
- `serde` + `bincode` + `zstd` already dependencies (SM blob path).
- Banked oracle: MG partonic σ̂ = 6.556e-7 pb for the uux 2→6 at √s = 500 GeV
  (TODO `lips-nbody` entry) — consumed by H3.

## 2. Design

### 2.1 Module layout

```
vibegraph-lib/src/
  pdf/
    mod.rs        // PdfSet, PdfMember, xfx_q2 API; flavor indexing (0↔21)
    grid.rs       // .info + .dat parsing → SubGrid { x, q2, flavors, xf } (H1)
    interp.rs     // Bicubic2D trait + backend (scirs2 or in-house) (H2)
  phasespace/
    mod.rs        // re-exports; lips2 helpers unchanged
    rng.rs        // ChaCha8-backed streams: (stream, position) addressing + bits→F uniform rule (H3)
    rambo.rs      // rambo::<F>(sqrt_s, &masses, uniforms) -> (Vec<LorentzVector<F>>, F weight) (H3)
  vegas.rs        // VegasGrid (serde) + adapt/freeze/sample phases (H5)
  runcard.rs      // MG run_card.dat parser: typed params + MG defaults table (H6)
  cuts.rs         // Cuts: compiled per-process filter over phase-space samples (H6)
  hadronic.rs     // PDF ⊗ σ̂ convolution, flavor classes, cut application (H7)
```

### 2.2 PDF evaluation

Mirror parton's structure exactly (it is the reference): per (subgrid, flavor)
spline over `(ln x, ln Q²) → x·f`, cubic × cubic, evaluate subgrids in order and
take the first in-range hit; out-of-grid → hard error at first (extrapolation
policy is a recorded non-goal). `PdfSet::load(dir, member)` reads
`<set>_{member:04}.dat` + `<set>.info`. The pinned set is
**NNPDF23_lo_as_0130_qed** (MG5's LO default `nn23lo1`), fetched by a pixi task
from the LHAPDF data server and stored under `validation/pdf/` (gitignored,
like MG outputs). Everything is `F: Real`-generic at the API (`x: F, q2: F`),
with the spline coefficient tables held in `f64` and cast on evaluation.

### 2.3 Massive RAMBO over `F: Real`

Signature decouples RNG from map: `fn rambo<F: Real>(sqrt_s: F, masses: &[F],
u: &[F]) -> RamboPoint<F>` consuming exactly `4n` uniforms, returning momenta
and the **weight** (the massless KSE volume `(π/2)^{n-1} ŝ^{n-2} /
((n-1)!(n-2)!)` times the massive rescale Jacobian
`(Σ|p⃗ᵢ|/√ŝ)^{2n-3} · (Σ|p⃗ᵢ|²/Eᵢ)⁻¹ · √ŝ · Π(|p⃗ᵢ|/Eᵢ)` — Kleiss–Stirling–Ellis
1986). Newton solve for ξ as in `gen_amplitude.py` but with an
`F`-relative tolerance. `rambo_massless` becomes the `masses = 0` fast path of
the same module (kept bit-compatible for the existing benches or migrated with
their goldens regenerated — H3 decides, bit-compat preferred).

### 2.4 VEGAS phases + serialization

Split the current monolith into:

- `VegasGrid { ndim, nbins, alpha, xi }` — `Serialize`/`Deserialize` (derive),
  with a validating constructor on deserialize (monotone `xi`, endpoints 0/1,
  shape consistency). Bin edges are plain `f64` probabilities-space data; no
  format cleverness. Container format chosen at the call site (CLI: bincode+zstd
  like the SM blob; JSON for debug dumps).
- `VegasGrid::adapt(f, neval, niter, rng) -> VegasResult` — today's `integrate`.
- `VegasGrid::sample_frozen(f, neval, rng) -> VegasResult` — one pass, **no grid
  refinement**: the distributed-generation phase primitive.
- **Batched integrand seam**: `adapt_batched` / `sample_frozen_batched` taking
  `FnMut(&[SamplePoint]) -> …` with a caller-chosen batch size, so the H4 lane
  evaluator plugs in without VEGAS knowing about SIMD.
- **Deterministic parallelism**: rayon over fixed-size chunks, each chunk on its
  own counter-based substream — `ChaCha8Rng` with stream ← (iter, chunk_idx),
  position 0 (§1.4) — reduced in chunk order. The result is bit-identical
  regardless of thread count, and the same addressing serves multi-machine
  sharding later (chunk range = shard), with no reliance on hash-mixing for
  substream independence.

Existing `Vegas` tests migrate; the public `Vegas::integrate` wrapper can remain
as a thin compatibility shim for one sprint.

### 2.5 Hadronic assembly (`hadronic.rs`)

- **Variables**: VEGAS on `(u₁, u₂, u₃) ∈ [0,1]³` mapped to `(x₁, x₂, cosθ)`
  with logarithmic x-maps `xᵢ = x_min^(1-uᵢ)` (i.e. `ln xᵢ` uniform,
  Jacobian `xᵢ ln(1/x_min)`), `x_min = m_ll,min²/s`; reject (zero-weight)
  points with `ŝ = x₁x₂s` outside the mass window. VEGAS adapts away the
  rejection band; a τ = x₁x₂ change of variables is a recorded alternative if
  adaption struggles.
- **Kinematics**: build partonic-CM momenta — beams `(√ŝ/2, 0, 0, ±√ŝ/2)`,
  leptons back-to-back at cosθ — and call `eval_m2` there (|M|² is invariant;
  z-beams keep the helicity-pruned path valid). No lab-frame boost until event
  output needs it.
- **Flavor classes**: LO DY has one σ̂ per Z/γ coupling class — up-type (u, c)
  and down-type (d, s) with MG's 4-flavor `p` (no b, no gg at LO). Two
  evaluators, PDF weights summed per class over both proton orderings:
  `Σ_q [f_q(x₁)f_q̄(x₂) + f_q̄(x₁)f_q(x₂)] · σ̂_class(q)`. Subprocess
  classification comes from the existing diagram enumeration for
  `p p > e+ e-` (grouping asserted against it, not hand-coded).
- **Cuts**: the integrand multiplies by the `Cuts` indicator (§2.6); the cuts
  object also exposes a conservative `shat_min()` hint (from `mmll`,
  `dsqrt_shat`, or 2·`ptl` — back-to-back LO leptons imply m_ll ≥ 2·pT_min) so
  the x-mapping's `x_min` stays positive even with `mmll = 0`.
- **Conventions pinned for validation**: μF fixed = m_Z (`fixed_fac_scale`),
  √s = 13 TeV, e⁺e⁻ final state, **MG default run-card cuts** (`ptl=10`,
  `etal=2.5`, `drll=0.4`; the pT cut regulates the photon pole), spin-average
  1/4 and color-average 1/9 for qq̄ initial states (MATRIX1 is summed — the
  1/(4·9) and flux 1/(2ŝ) live in the integrand prefactor). A second reference
  run with m_ll ∈ [60, 120] exercises the `mmll`/`mmllmax` path.

### 2.6 Run card + cuts abstraction

Two layers, deliberately separated:

**`runcard.rs` — parse + typed defaults.** Parses MG's `run_card.dat` syntax
(`<value> = <name> ! comment`) into a `RunCard` struct whose defaults are the
MG LO defaults transcribed from `RunCardLO.default_setup` (§1.5) — so an *empty*
run card reproduces MG's out-of-the-box behavior, and our reference MG run can
share the literal same card file. Non-cut params we consume now: `ebeam1/2`,
`lpp1/2` (only `(1,1)` supported, else hard error), `pdlabel`/`lhaid`, `scale`/
`fixed_ren_scale`/`fixed_fac_scale`/`dsqrt_q2fact1/2`, `iseed`, `nevents`,
`maxjetflavor`. Unknown parameter names are hard errors (typo protection);
recognized-but-unconsumed params parse into a preserved map. `GlobalConfig`
grows `run_card_path: Option<PathBuf>` + `load_run_card()` (defaults when
absent), the same shape as the `load_ufo` seam.

**`cuts.rs` — compiled per-process filter.** `Cuts::compile(&RunCard,
&[ExternalLeg]) -> Result<Cuts, CutError>` classifies each final-state leg into
MG's letter classes (j/b/a/l/n/H, honoring `maxjetflavor`) and bakes the active
thresholds into a flat check list; `cuts.pass(&[LorentzVector<F>]) -> bool` is
the phase-space filter (integrand indicator; later the event accept-gate).
Implemented cut families this sprint: ŝ window, single-leg pT/E/η min/max per
class, pairwise ΔR and invariant-mass min/max, `mmnl`, `ptll` — i.e. everything
whose default is active plus the simple pairwise set. Everything else in the
§1.5 inventory is **parse-and-detect**: `compile` compares each unimplemented
cut's parsed value against the MG default and hard-errors if it deviates
(`CutError::UnimplementedCutActive`) — the SextetUnsupported pattern: never
silently ignore an active cut. Conventions (η vs y, ΔR definition, class
membership) are pinned against `SubProcesses/cuts.f`, not the run-card comments.

### 2.7 CLI (two-phase seed, minimal)

`vibegraph integrate <proc_card> [--run-card <run_card.dat>]` → prints σ ± err,
writes `<out>/grid.bin.zst` (VegasGrid + run metadata: seed, neval, process,
PDF set, the resolved `RunCard` — enough for phase 2 to refuse mismatched
inputs). The `generate` phase
command lands with `event-output-lhef`, not here. Reuses the
`GlobalConfig::load_ufo` seam; full proc-card option coverage stays with the
`cli-proc-card` backlog item.

## 3. Validation regime

Per the AGENTS.md doctrine, each oracle with its blind spot:

| Oracle | Gates | Blind to | Compensation |
|---|---|---|---|
| parton-generated `(pdg, x, Q², x·f)` JSON (pixi env running parton on the pinned set) | H1 parse + H2 interp, rel ≤1e-9 (same algorithm) or documented looser bound if backends differ | subgrid-seam and edge behavior if points are interior-only | sample points *on* knots, at subgrid Q² seams, x→1 tail, and grid corners |
| Uniforms-replay RAMBO oracle (extend `gen_amplitude.py` to dump `(u[4n], momenta, ξ)` tuples) | H3 map, rel ≤1e-13 per component + weight formula vs Python-side computed weight | RNG wiring order (which uniform feeds which draw) | pinned bits→uniform conversion goldens + one end-to-end seeded momenta golden (freezes stream addressing + draw order) |
| Momentum-conservation / on-shell invariants over random `(n, masses, √s)` incl. threshold-adjacent | H3 numerics | anything an exactly-conserving wrong map preserves (it is a consistency check, not a correctness oracle) | the replay oracle above is primary; this is the fuzz layer |
| Banked MG partonic σ̂ = 6.556e-7 pb (uux 2→6, √s=500) via flat RAMBO-weight MC | H3 end-to-end weight normalization | angular mis-sampling of small measure; per-point errors that integrate away | statistical only by nature; the replay oracle covers per-point |
| Scalar-vs-lane bit-identity: each extracted lane == scalar `eval_m2`, all 14 `MG_VALIDATED_PROCESSES`, random + z-beam + threshold points | H4 | nothing within covered ops — bitwise | keep the 14-process scalar gate untouched as the anchor |
| Serde round-trip + frozen-grid reproduction + thread-count invariance (1 vs N threads bit-identical) | H5 | statistical quality of the sampling itself | existing `validate_vegas.rs` σ targets keep running |
| banner.py `RunCardLO` defaults dumped to JSON (pixi script) vs `runcard.rs` defaults table, per-param | H6 parser/defaults | cut *semantics* — it checks transcribed values, not what a cut does to momenta | per-cut boundary unit tests (both sides of each threshold, η sign, ΔR φ-wrap); `UnimplementedCutActive` detection test; the H7 σ gate is the end-to-end semantic check |
| MG5 σ(pp→e⁺e⁻) reference, same PDF/scale/cuts, agreement within combined MC error (target <1%) | H7 | smooth mis-shapes (the classic latent sampler-bug mode, note 07) | pointwise integrand oracle: pin ~10 `(x₁,x₂,cosθ)` integrand values against an independent Python computation (parton × MG standalone |M|²); informational dσ/dm_ll histogram comparison |

**Known-wrong informational comparison from day one of H7**: wire the MG σ
comparison as a printed informational delta *before* conventions are reconciled
(it will disagree while μF/cuts/flux factors are being assembled) — the moment it
snaps to agreement is the end-to-end signal; enforce it at session close.

MG reference generation lives in the session that consumes it (H7): a
`pixi run -e madgraph` task producing the pp→e⁺e⁻ run with `pdlabel=lhapdf`,
the pinned set, and fixed scale, banking σ ± Δσ in the repo the same way
existing references are banked. The reference run consumes **the same run-card
file** vibegraph reads (§2.6), so cut and beam settings cannot drift between
the two sides by construction.

## 4. Session plan (`hadronic-xsec` sprint)

Every session: feature-dev agent, isolated worktree (manage worktrees manually —
see the fragility lesson), `cargo test` green, the 14-process
`validate_helas_mg` net untouched (only H4 touches evaluator-adjacent code, with
its own bit-exact gate on top), TODO.md updated, single commit series on the
sprint branch. Model per session as listed (Opus default; Sonnet where the spec
is tight and judgment is low).

- **H1 — `pdf-grid-io`** (Sonnet). LHAPDF6 `.info`/`.dat` parser → `PdfSet`/
  `SubGrid` structs; pixi task fetching the pinned NNPDF23_lo_as_0130_qed set;
  pixi env + script generating the parton oracle JSON (knot values and off-knot
  samples, incl. seams/edges). *Accept*: parser round-trips the real set (knot
  counts, flavor lists, on-knot x·f values match the oracle exactly); malformed
  input errors are typed; no interpolation yet. *Depends*: —.
- **H2 — `pdf-interpolate`** (Opus). `Bicubic2D` backend behind the `pdf` API;
  trial `scirs2-interpolate::RectBivariateSpline`, execute the §1.2 decision
  rule, fall back to the in-house s=0 bicubic if it misses; subgrid walk +
  flavor aliasing; `F: Real` API surface. *Accept*: parton oracle ≤1e-9 rel
  interior (or the documented decision record if the fallback's construction
  differs — then vs LHAPDF-published values within interpolation accuracy and
  the tolerance justified in the note); seam/edge cases from H1's oracle pass;
  decision (adopt/reject scirs2, with reasons) recorded in this note.
  *Depends*: H1.
- **H3 — `rambo-real-generic`** (Opus). `phasespace/` split; `rng.rs` per §1.4
  (`ChaCha8Rng` stream/position addressing, pinned bits→`F` uniform-conversion
  goldens); `rambo::<F>` massive with weight (§2.3); uniforms-replay oracle
  (Python dump + Rust replay, ≤1e-13); invariant fuzz (conservation, on-shell,
  threshold-adjacent); flat-MC σ̂ check against the banked 6.556e-7 pb within MC
  error; existing `rambo_massless` callers migrated. *Accept*: all of the above
  in `cargo test` (Python-dump fixture checked in); benches still compile.
  *Depends*: —.
- **H4 — `eval-simd-lanes`** (Opus). Bound audit (`ConstZero`/`ConstOne` on
  `NumericArray`; relax `Real` or newtype — record the choice); kernel
  branch-divergence audit with a written inventory of every data-dependent
  branch on `F` and its lane-uniformity argument; SoA transpose helper
  (`&[point; N] → LorentzVector<NumericArray<f64,N>>`); batched
  `eval_m2_lanes::<N>`; criterion bench sweeping N ∈ {2,4,8} across the
  benchmark processes vs scalar, recording per-point speedup and the chosen
  default N. *Accept*: per-lane bit-identity vs scalar `eval_m2` on all 14
  MG-validated processes (random + z-beam + threshold points); bench table in
  the close-out; no change to scalar-path codegen (existing benches within
  noise). *Depends*: — (independent; do not block on it).
- **H5 — `vegas-serde-split`** (Sonnet). `VegasGrid` serde + validating
  deserialize; `adapt`/`sample_frozen` split with compat shim; batched-integrand
  variants; rayon chunked deterministic accumulation (§2.4). *Accept*:
  round-trip test (bincode + JSON); frozen-grid sampling agrees with the
  adapt-phase estimate on the existing test integrands; 1-thread vs N-thread
  bit-identity test; `validate_vegas.rs` targets unchanged. *Depends*: — (the
  batched-integrand signature coordinated with H4's API sketch, not its
  implementation).
- **H6 — `run-card-cuts`** (Opus). `runcard.rs` parser + MG-defaults table and
  `cuts.rs` compiled filter per §2.6; `GlobalConfig::load_run_card` seam; pixi
  script dumping `RunCardLO` defaults from `banner.py` to JSON as the defaults
  oracle. *Accept*: defaults table matches the banner.py JSON dump per-param
  (transcription oracle); parses a real MG-generated `run_card.dat` from the
  validation output; unknown-name and `UnimplementedCutActive` hard-error tests
  (e.g. a set `ptj1min` errors, a default one doesn't); per-cut boundary unit
  tests for every implemented family — pT/E/η min & max both sides of the
  threshold, η sign symmetry, ΔR including the φ-wraparound case, pair-mass and
  ŝ window, `ptll`, `mmnl`; η-vs-y and ΔR conventions documented against the
  `cuts.f` lines that define them; `shat_min()` hint unit-tested against active
  cut combinations. *Depends*: — (wave 1; the ExternalLeg classification input
  is the existing subprocess metadata, not H2/H5 work).
- **H7 — `hadronic-sigma`** (Opus). `hadronic.rs` assembly (§2.5): mappings,
  flavor classes asserted against `p p > e+ e-` enumeration, prefactors,
  `Cuts` indicator wired into the integrand; MG reference generation task
  (pdlabel=lhapdf, pinned set, μF=m_Z, shared run card — default-cuts run + the
  60–120 `mmll` window run) with σ banked; the pointwise integrand oracle (~10
  pinned points vs independent Python, including points just inside/outside a
  cut boundary); informational-then-enforced MG σ comparison; scalar path
  first, lane-batched integrand behind a flag if H4 has landed. *Accept*:
  σ agreement within combined MC error (<1% target) for **both** reference runs
  *enforced* in an extended-validation test; pointwise oracle ≤1e-9 on
  PDF×flux×|M|² factors; the informational dσ/dm_ll comparison committed (plot
  or table, not gated). *Depends*: H2, H5, H6 (H3 for nothing on the 2→2 path;
  H4 optional).
- **H8 — `cli-integrate`** (Sonnet). `vibegraph integrate <proc_card>
  [--run-card …]` per §2.7: σ printout, grid + metadata artifact (bincode+zstd),
  refusing to overwrite without `--force`. *Accept*: end-to-end run on the
  pp→ll card + reference run card reproduces the H7 test σ from a cold start;
  artifact reloads and `sample_frozen` reproduces; documented in README.
  *Depends*: H5, H6, H7.
- **H9 — close-out** (Sonnet). TODO.md (pipeline table row for hadronic σ,
  session ledger, unblocks: `event-output-lhef` now has grid-phase + RAMBO +
  cuts accept-gate; `lips-nbody` remaining scope = channel mappings +
  multi-channel weights); this note's outcome section; memory update;
  defer-list below reconciled. *Depends*: all.

Suggested waves: {H1, H3, H4, H5, H6} → {H2} → {H7} → {H8} → {H9}. H4 is the
only session allowed to report a negative result (bounds or divergence make
lanes not-worth-it at this time) without blocking the sprint — its gate then
becomes the written audit + the decision record, and H7 ships scalar-only.

## 5. Decision records (to be filled by sessions)

- H2: scirs2-interpolate adopted / rejected because …
- H3: `rambo_massless` kept bit-compatible / regenerated goldens because …
- H4: `Real` bounds relaxed / newtype introduced; default lane width N = …;
  divergence inventory location …
- H6 (`run-card-cuts`, DONE): `runcard.rs` parses `<value> = <name> ! comment`
  into a `RunCard` whose 209-entry defaults table transcribes every scalar param
  of `RunCardLO.default_setup`; validated per-param against a banner.py JSON dump
  (`validation/madgraph/dump_runcard_defaults.py`, pixi task
  `dump-runcard-defaults`, snapshot `validation/madgraph/runcard_defaults.json`)
  — 218 params dumped, 209 scalars matched, the 9 unmatched are `system=True`
  internals (`pdg_cut`, `ptmin4pdg`, … never written to a user card). Unknown
  names hard-error; only `lpp1=lpp2=1` accepted; recognized-but-unconsumed params
  retained by name. `GlobalConfig::load_run_card` mirrors the `load_ufo` seam.
  - **Implemented cut families**: ŝ window (`dsqrt_shat`/`dsqrt_shatmax`),
    single-leg pT/E/η min+max for classes j/b/a/l, pairwise ΔR, pairwise
    invariant mass, `ptll`, `mmnl`. Everything else `cut=`-tagged is
    parse-and-detect: `Cuts::compile` returns `CutError::UnimplementedCutActive`
    if the value deviates from its MG default (list in `cuts.rs`
    `UNIMPLEMENTED_CUTS`).
  - **cuts.f convention pins** (LO template `Template/LO/`): single-leg η and the
    ΔR separation both use **rapidity** `rap = ½·ln((E+pz)/(E−pz))`
    (`Source/kin_functions.f:95`; applied as `abs(rap)` in
    `SubProcesses/cuts.f:426`) — *not* pseudorapidity; equal for massless legs
    but rapidity is the enforced definition. ΔR² = Δφ² + Δy²
    (`kin_functions.f:42` `R2`) with Δφ = `acos(clamp(·, ±0.99999999))`
    (`kin_functions.f:180` `DELTA_PHI`), opening angle in [0,π] so φ-wrap is
    intrinsic. **The `dr{...}` threshold is stored un-squared and compared
    against ΔR²** (`setcuts.f:348`, `cuts.f:442`) — i.e. the effective ΔR bound
    is `√dr`; contrast the mass/`ptll` thresholds which are stored as signed
    squares `x·|x|` (`setcuts.f:399,479`). Class membership at `setcuts.f:217`
    (jet = `|pdg|≤min(maxjetflavor,6)` or 21; b = `maxjetflavor<|pdg|≤5`;
    l = {11,13,15}; a = pdg 22; ν = {12,14,16}); single-leg cuts skipped for ν
    and mass>20 GeV (`setcuts.f:212`); E-min is a strict `≤` reject
    (`cuts.f:413`). `mmll`/`ptll` apply only to same-flavour opposite-charge
    lepton pairs (`setcuts.f:396,473`).
  - **`shat_min()` hint**: `max(dsqrt_shat², mmll² if an l⁺l⁻ pair present,
    (2·ptl)² if exactly two final leptons)` — all provably ≤ ŝ of a surviving
    point (§2.5).
  - **Inventory note beyond §1.5**: `etajmin`/`etaamin` carry `cut='a'` in
    banner.py (harmless — value 0, and `etaXmin` is class-keyed by `setcuts.f`);
    `misset`/`ptheavy` are the n/H single-leg cuts but are parse-and-detect this
    sprint (no ν/heavy final state in pp→e⁺e⁻). The `dr` un-squared comparison
    is the one convention H7's σ gate must confirm against a real MG run — it is
    surprising but faithful to the reference; flagged for the informational
    comparison.
- H7: x-mapping kept `ln x` / switched to (τ, y) because …

## 6. Deferred follow-ups

- **Multi-channel phase space** (`lips-nbody` main body): propagator-pole
  channels, `1/Σᵢ(1/Jᵢ)` weights, Sherpa `PHASIC++` survey — next feature
  sprint after this one; the sampler/integrator seams built here (batched
  integrand, frozen grids, RAMBO-as-map) are its substrate.
- **`event-output-lhef`**: unblocked by H5's frozen-grid `sample_frozen` +
  H3's RAMBO; brings the `generate` CLI phase, accept/reject, leading-Nc color
  tags, and the `mg-single-helicity-bench` rider (TODO).
- **Remaining run-card cut families** (HT sums, leading-object orderings,
  photon isolation, VBF, merging, per-PDG dict cuts, `cut_decays` semantics for
  decayed resonances): parse-and-detect only this sprint (§2.6); implement when
  a process with jets/photons in the final state needs them — the hard error
  marks exactly when.
- **α_s from `.info` / running couplings + dynamical scales**
  (`dynamical_scale_choice` is parsed-but-rejected ≠ fixed): first hadronic
  process with QCD final states.
- **PDF error members / multiple sets**: `member > 0` plumbing exists after H1;
  no consumer yet.
- **f32 lanes**: doubles SIMD width; precision impact on |M|² unstudied — needs
  its own validation pass against the f64 gate.
- **Lab-frame kinematics + rapidity distributions**: with event output, not σ.
