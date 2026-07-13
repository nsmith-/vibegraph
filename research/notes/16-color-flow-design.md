# 16 — Color flow: design + session plan

Design for the `color-flow` task (TODO.md): multi-flow color algebra, the prerequisite
for `mg-validation-coverage` #8 (`u u~ > u u~`), any QCD≠0 validation, and the hadronic
pp→ll cross section. Sources: `refs/mg5amcnlo` (`madgraph/core/color_algebra.py`,
`color_amp.py`, `helas_objects.py`, `models/import_ufo.py`), generated `matrix1_orig.f`
files under `validation/madgraph/output/`, and the current vibegraph eval pipeline.

## Outcome (2026-07-12, sessions C1–C6 complete on branch `color-flow`)

The design below was implemented as planned; §4's session plan matches what shipped.
Final state: `validate_helas_mg` enforces **13 processes** — the original 11 (bit-
identical) plus `uux_to_uux` (5.61e-14) and `gg_to_ttx` (1.89e-15) via the multi-flow
CF-weighted evaluator built here. `gg_to_gg` (NCOLOR=6, the 4-gluon-vertex stress test)
stays **informational**: its color side is proven correct — the CF matrix and per-flow
coefficients match MG, and its 3 pure-exchange diagrams are bit-for-bit — but
enforcement is blocked by a **pre-existing, unrelated** Lorentz bug: the imaginary
4-gluon coupling `GC_12 = i·g²` carries a spurious +90° phase on the contact diagram
relative to the exchange diagrams. That bug is filed to `validation-sprint`
(`TODO.md`); the MG reference data and JAMP probe built here are already in place, so
fixing it should enforce `gg_to_gg` straight to ≤1e-12 with no color-side changes.

One correction to the design as originally written: §2.4 below claimed the 3-vs-3̄
slot pairing across a propagator was "automatic" from the diagram walk. It is not —
see the note inline in §2.4 and the C5c writeup in `TODO.md`'s `color-flow` entry for
the actual fix (an explicit fermion-flow slot swap).

Timing (profiling, `--test-threads=1`, ns/eval vs MG MATRIX1): `uux_to_uux` 5,121/278
(18.5×), `gg_to_ttx` 9,148/659 (13.9×), `gg_to_gg` 24,110/949 (25.4×, informational).
Cross-flow CSE works as designed — NCOLOR=6 costs ~2× NCOLOR=2, far below naive
NCOLOR× scaling.

## 1. How MadGraph factorizes color from Lorentz

MadGraph never evaluates color numerically per phase-space point. The amplitude is
factorized **at the symbolic level** into

```
M = Σ_flows  C_f · J_f(p, hel)          |M|² = Σ_{f,f'}  J_f · CF_{f,f'} · J_{f'}*
```

where the `J_f` ("JAMPs") are kinematic partial amplitudes built from HELAS calls
(Lorentz × couplings only), and `CF` is a constant matrix of **exact rational numbers**
computed once per process by symbolic SU(3) algebra. The floating-point runtime never
sees a color index.

The factorization rests on the UFO vertex format itself: a vertex is a *tensor
product* over three axes

```python
V = Σ_{c,l}  couplings[(c,l)] · color[c] ⊗ lorentz[l]
```

so each diagram amplitude splits into (choice of color structure per vertex) ×
(Lorentz/coupling chain). The pipeline, concretely:

### 1a. `colorize` — one color string per (diagram, color-index chain)

`ColorBasis.colorize` (`color_amp.py`) walks a diagram's vertices in
wavefunction-chain order and builds products of color atoms:

- The vertex's color-string leg indices `1..n` refer to **positions in the UFO
  interaction's particle list**; they are substituted with actual external leg numbers
  or fresh negative "summed" indices for internal lines (one per propagator end-pair —
  the δ gluing of a propagator is implicit in index repetition).
- The *output* leg of a non-final vertex gets its color rep **conjugated**
  (`get_anti_color()`): the propagator's far end sees the antiparticle.
- A vertex with `k` color structures multiplies the accumulated dict out `k` ways;
  the key is the tuple of chosen color indices, one per vertex (the **color-index
  chain**). Colorless vertices append index `0` with no atom. In the SM only the
  4-gluon vertex has `k>1` (`k=3`).
- A fully colorless diagram gets `ColorOne()` (coefficient 1).

Each HELAS amplitude carries its chain as `color_indices`
(`helas_objects.get_color_indices`), so amplitudes and color strings pair up exactly.

### 1b. Simplification to a canonical basis — exact rationals

`ColorBasis.update_color_basis` runs `ColorFactor.full_simplify()` on each string.
The rewrite system (`color_algebra.py`) works over generalized objects
`T(a1..an, i, j)` (fundamental chain), `Tr(a1..an)`, `f`, `d`, with coefficients of
the form

```
coeff ∈ Q (python fractions.Fraction)  ×  (i if is_imaginary)  ×  Nc^{Nc_power}
```

The complete tree-level SU(3) rule set (all coefficients exact rationals):

| rule | identity |
|---|---|
| f → traces | `f(a,b,c) = -2i·Tr(a,b,c) + 2i·Tr(c,b,a)` |
| T chain merge | `T(A,i,j)·T(B,j,k) = T(A+B,i,k)` |
| T closes to trace | `T(A,i,i) = Tr(A)` |
| Fierz (within T) | `T(a,X,b,X,c,i,j) = ½ T(a,c,i,j)Tr(b) − ½Nc⁻¹ T(a,b,c,i,j)` |
| Fierz (T·T) | `T(a,X,b,i,j)T(c,X,d,k,l) = ½ T(a,d,i,l)T(c,b,k,j) − ½Nc⁻¹ T(a,b,i,j)T(c,d,k,l)` |
| Fierz (Tr·Tr, Tr·T) | same pattern (`Tr.pair_simplify`) |
| trace values | `Tr() = Nc`, `Tr(a) = 0` |
| conjugation | `T(A,i,j)* = T(reverse A, j, i)` |

`full_simplify` iterates `simplify` (single-object rules) and `pair_simplify`
(two-object contractions) to a fixed point; results are cached by canonical form.
Each **distinct canonical color string** surviving simplification becomes one basis
element (a "color flow" in the generalized sense); the basis entry records, per
contributing (diagram, chain): the exact coefficient `(Fraction, is_imaginary,
Nc_power)`.

Note the basis is the *trace/δ basis* that falls out of simplification, not the
leading-Nc "color flow decomposition" (that exists separately —
`color_flow_decomposition`, used only for LHEF color tags at event-writing time).

### 1c. JAMPs and the color matrix

`get_color_amplitudes` emits, per basis element `f`, the list of `(fermionfactor ×
coeff × i^{im} × Nc^{power}, AMP number)` — the generated Fortran is exactly

```fortran
JAMP(1,1) = (-1d0)*AMP(1) + (-1d0)*AMP(2) + ...        ! coefficients are the
                                                        ! rationals, printed as floats
```

`ColorMatrix` computes `CF_{ff'} = ⟨basis_f | basis_f'⟩` by multiplying string `f`
with the **complex conjugate** of string `f'` (summed indices of `f'` relabeled to
avoid collisions), `full_simplify`, and evaluating `Nc = 3`. Everything stays a
`Fraction` until code generation; in current MG output the matrix is emitted as
`REAL*8 CF(NCOLOR,NCOLOR)` `DATA` statements with the denominator divided through
(older exporters kept integer numerators + per-line `DENOM` via
`get_line_denominators` — LCM of a row's denominators). The squaring loop:

```fortran
DO I = 1, NCOLOR
  ZTEMP = Σ_J CF(J,I)*JAMP(J)
  MATRIX1 = MATRIX1 + ZTEMP*DCONJG(JAMP(I))
```

MATRIX1 is therefore **color-summed** (helicity-summed by its caller) — which is why
our `validate_helas_mg` references already contain the full CF contraction, and the
current test multiplies the Rust result by a stand-in `color_factor(name)` ∈ {1,3,9}
(all 11 validated processes have NCOLOR=1).

### 1d. SM tree-level color vocabulary

From `models/sm/vertices.py`, the complete set of color strings we must handle:

```
'1'                       ×94   (colorless)
'Identity(1,2)'           ×50   (qq̄ neutral-boson vertices: δ_ij)
'T(3,2,1)'                ×6    (qqg: (T^{a3})_{i2 j̄1})
'f(1,2,3)'                ×2    (ggg)
'f(-1,1,2)*f(3,4,-1)' + 2 perms (gggg: 3 structures on one vertex)
```

Convention (UFO paper / `color_algebra.T`): `T(a,…,i,j)` — adjoint indices first,
then the **fundamental (3) index `i`**, then the **antifundamental (3̄) index `j`**.
`Identity(m,n)` is rep-dependent: MG's importer (`import_ufo.treat_color`) rewrites
it using the particle reps at slots m,n — for a 3/3̄ pair it becomes `T(i,j)` with the
slot order fixed by which particle is the 3 (i.e. δ_{i j̄} with the fundamental first);
for an 8/8 pair it becomes `2·Tr(m,n)` (note the **factor 2**: Tr[T^aT^b] = δ^{ab}/2);
sextets → `K6` (out of scope). This resolution needs the *particle* color reps, so it
belongs in the UFO load, not in the algebra engine.

## 2. Vibegraph design

### 2.1 Shape of the change

Same factorization, mapped onto the existing passes:

```
compile (per subprocess):
  pass 1+2   root diagrams (unchanged)                      root_diagram.rs
  pass C     colorize each diagram → color strings          NEW  (uses diagrams::Diagram)
             full_simplify → color basis + per-chain coeffs NEW  (exact rationals)
             CF matrix (exact rationals, Nc=3)              NEW
  pass 3a    lower: one AST with NCOLOR flow roots          lower.rs (extended)
  pass 3b    fold: rationals → F pools  ← ONLY float point  fold.rs (extended)
bind:        CF matrix → Box<[F]>                           run.rs
eval:        per hel: J_f per flow; Σ CF_{ff'} J_f J_f'*    run.rs
```

Exact rationals (`num_rational::Ratio<i64>`, already a dependency and already the
`GroupScalar` choice in `repr/color.rs`) flow from the UFO color strings through
colorize, simplification, JAMP coefficients, and the CF matrix; they are resolved to
`F` only in `fold.rs` (`Folded::pools`) and `BoundAmplitude::bind` — per the
convention that binding is where numbers happen.

The coefficient scalar mirrors MG exactly:

```rust
/// q · i^{imag} · Nc^{nc_power}, exact.
struct ColorCoeff { q: Ratio<i64>, imag: bool, nc_power: i32 }
```

with checked `i64` arithmetic (MG uses arbitrary precision; tree-level SM factors are
tiny — panic on overflow is an acceptable tripwire, `Ratio<i128>` is the escape
hatch behind the existing `GroupScalar` trait boundary).

### 2.2 `helas/repr/color.rs` — from marker traits to a working algebra

Current state: `GroupScalar` trait + `ColorRepr` marker types (`SU3Fundamental`,
`SU3Adjoint`, `ColorSinglet`) carrying Casimir/Dynkin constants and numeric fiber
types `[C<F>; 3]` / `[C<F>; 8]`, plus two open questions in comments.

Changes:

1. **Resolve the module-level "library choice" TODO**: `Ratio<i64>` is the decision
   (checked ops; `GroupScalar` boundary retained as the escape hatch).
2. **Resolve the `SU3Adjoint` RESEARCH comment** the same way MG answers it: we adopt
   symbolic factorization; there will be **no numeric `f^{abc}` contractions and no
   `[[f64;8];8]` table** — delete that claim. (Mangano–Parke–Xu leading-Nc flow
   decomposition is likewise not needed for the CF-matrix approach; it returns only
   for LHEF color tags, see §5.)
3. **The numeric `Color` fibers stay unused** by this feature. They remain the typed
   vocabulary of `wavefn.rs`-style hand-built objects; the runtime never carries a
   color vector. No change beyond doc honesty.
4. **Add the algebra layer** (suggest growing `repr/color.rs` into a directory or a
   sibling `helas/color/` module — it is compile-time-only and independent of `F`):
   - `ColorCoeff` (above) with `mul`, `add`-compatibility, `conj`, `eval_nc(3) -> Ratio<i64>`.
   - `ColorTensor` — generalized atoms, ported from `color_algebra.py`:
     `T(Vec<Idx>, Idx, Idx)`, `Tr(Vec<Idx>)`, `F(Idx,Idx,Idx)`, `D(...)`, `One`.
     (`Epsilon`/`K6` sextet/baryonic atoms: explicit unsupported error.)
   - `ColorString` = `ColorCoeff × Vec<ColorTensor>`; `ColorFactor` = sum of strings.
   - `simplify` / `pair_simplify` / `full_simplify` fixpoint + canonical immutable
     form (`to_canonical`) for use as basis keys — port MG's rule set (§1b) verbatim,
     coefficients exact.
   - `Idx = i32`, negative = summed (MG convention), external legs positive.
5. **Rep bookkeeping**: a small `ColorRep` enum (`Singlet | Triplet | AntiTriplet |
   Octet`) with `from_ufo(i32)` (1, 3, −3, 8) and `anti()` — used by colorize and by
   the `Identity` resolution. The `ColorRepr` marker types gain a link to it; their
   Casimir/Dynkin constants become **oracles for the engine's unit tests**
   (e.g. `T(a,i,j)T(a,j,k)` must simplify to `C_F δ_ik` with
   `C_F = SU3Fundamental::casimir()`).

### 2.3 UFO parser changes

1. **New `ufo/color.rs`**: parse vertex color strings into a structured expression at
   model load — grammar is tiny: product (`*`) of atoms `1`, `Identity(m,n)`,
   `T(a…,m,n)`, `f(a,b,c)`, `d(a,b,c)` with signed integer indices (positive =
   vertex particle slot, 1-based; negative = summed). Unknown atoms → hard error
   naming the vertex.
2. **`Vertex.color: Vec<String>` → `Vec<ColorExpr>`** (`vertices.rs`), keeping
   `Display` for round-trip/debug. `Identity` is resolved here (rep-dependent, §1d),
   using the already-parsed particle `color: i32` fields — including the ×2 for the
   octet case and the 3/3̄ slot-direction rule. This is a **convention bug-magnet**;
   pin it with fixture tests against MG's `treat_color` behavior.
3. **Interned SM blob**: if the parsed representation is serialized into the SM blob,
   regenerate via `gen_sm_blob` and note the diff-check follow-up already tracked in
   `validation-sprint`.
4. No numeric literals appear in UFO color strings — exactness is free at this layer;
   rationals first arise inside the simplification engine.

### 2.4 Colorize over `diagrams::Diagram`

We can improve on MG's vertex-list walk: the owned `diagrams::Diagram` already
records each `Vertex`'s rays **in interaction-slot order** (`Diagram::from_view`),
which is precisely the order color-string indices refer to. The walk per diagram:

- slot ray → external `Leg` ⇒ index = leg number (free); rep from the leg particle.
- slot ray → `Prop(p)` ⇒ index = fresh summed index, **one per propagator** (its two
  end-vertices use the same index; reps are asserted mutually conjugate as a
  cross-check). The 3-vs-3̄ slot *assignment* is not automatic: `Diagram::from_view`
  presents every ray in feyngraph's all-incoming crossing, which orders a directed
  quark line by the opposite of its fermion-number arrow from MadGraph's convention
  (MG's `T(a…,i,j)` indexes `i` by the arrow-out leg, `j` by arrow-in). Left alone,
  every T tensor comes out transposed from MG's — invisible for purely-rational
  T-chain terms, but it complex-conjugates the whole color string and so flips the
  sign of the imaginary `f → trace` coefficient relative to T-chain terms (found in
  C5c on `g g > t t~`, which mixes both). The walk corrects this with an explicit,
  unconditional swap of the single 3 and 3̄ slots at every vertex that has one of each
  (octet/singlet slots are untouched, so pure-gluon vertices don't need it); pinned by
  the regression test `vibegraph-lib/tests/color_cf.rs::gg_to_ttx_flow_structures_untransposed`.
- vertex with k color structures ⇒ expand the accumulated map over chains
  (`Vec<u8>` color-index chain, only indices that appear in `couplings` keys — the
  same `(color_idx, lorentz_idx)` table `VertexInfo::from_ufo` already iterates).

Then per subprocess: `full_simplify` each chain's string, accumulate the **color
basis** (`IndexMap<CanonicalString, Vec<(diag, chain, ColorCoeff)>>` — insertion
order must be made deterministic; MG sorts basis keys, and matching MG's JAMP
ordering makes JAMP-level debugging against Fortran dumps possible), and build the
exact **CF matrix** `CF_{ff'} = eval_nc(⟨f, f'*⟩)` as `Vec<Ratio<i64>>`.

Degenerate case: all strings empty ⇒ single `ColorOne` flow, CF = [[1]] — the
colorless processes reduce to today's behavior; the quark-line QCD=0 processes get
CF(1,1) = 3 or 9 **computed instead of hard-coded**, deleting the
`color_factor(name)` stand-in in `validate_helas_mg`.

### 2.5 Evaluator integration (`helas/eval`)

**Vertex terms.** `VertexTerm::from_ufo` currently ignores its `_color: &str`
parameter and `VertexInfo::from_ufo` sums *all* `(color_idx, lorentz_idx)` couplings
into one term list — correct only when every used coupling shares one color
structure. Change: build `VertexInfo` **restricted to one `color_idx`** (the terms
with that first key component); pass 1+2 rooting is otherwise unchanged. A diagram
with a multi-structure vertex (gggg) compiles into one rooted tree per color-index
chain — chains share everything except the affected vertex, and CSE already collapses
the shared subtrees.

**Lowering.** `lower()` currently emits `Σ_d coeff_d · amp_d` under a single root.
New shape: per flow `f`, `JAMP_f = Σ_{(d,chain) ∈ basis_f} colorcoeff · sym·fermi ·
amp_{d,chain}`, and a single variadic root over the flows:

```
(Flows jamp_0 jamp_1 … jamp_{NCOLOR-1})
```

A variadic `Op::Flows` root (like `PMomOut`) keeps `Ast` single-rooted — no changes
to the `Tree` trait, `fold` reachability, or the egglog encoding beyond one more
constructor. Cross-flow sharing (the same `amp_{d,chain}` appearing in several JAMPs
with different rational weights, e.g. both flows of `u u~ > u u~`) comes free from
the existing hash-cons CSE.

**s-expression / `Op` additions** (the bijection test in `op.rs` self-extends):

- `Op::Flows` — variadic root, no leaf.
- `Op::CoeffRat` — exact scalar leaf; payload `Sym::Rational { num: i64, den: i64,
  imag: bool }` (Nc already evaluated), rendered `(CoeffRat (Rational num den imag))`.
  Existing `Coeff(f64)` stays for symmetry/fermi/Lorentz coefficients (migrating those
  to rationals is a separate, optional cleanup).
- egglog schema (`egraph.rs`): one constructor each; `CoeffRat`'s payload is three
  leading `i64` fields — fits the existing leaf-payload encoding; `Flows` is variadic
  via `(Vec Node)` like `PMomOut`. Round-trip fixtures extend mechanically.

**Folding (the only float boundary).** `fold.rs` maps `CoeffRat`: `imag == false` ⇒
real pool entry `RealReq::Rat(num, den)` resolved as `F::from(num)/F::from(den)`;
`imag == true` ⇒ complex pool entry. This is where "exact rationals all the way
through, floats at constant folding" lands. (Track 1 session A2's constant-subgraph
folding will happily swallow `CoeffRat · Coupling` products later.)

**Runtime.** `AmplitudeEvaluator` gains `n_flows` and the exact CF matrix
(`Vec<Ratio<i64>>`); `BoundAmplitude::bind` resolves it to `Box<[F]>`. `run()` grows
a flows-aware variant returning the root's children slots; `eval_m2` per helicity:

```
m2 += Σ_i ( Σ_j CF[j,i]·J_j ) · conj(J_i)      // MG's ZTEMP order
```

Bit-for-bit care: for NCOLOR = 1 keep today's op order — multiply CF **after** the
helicity sum (`CF·Σ_hel|J|²`), since `Σ 9·x_h ≠ 9·Σ x_h` bitwise. That keeps the
whole existing 11-process net bit-identical and confines REL_TOL-only gating to the
genuinely new multi-flow processes.

**Interaction with the 3-track program (note 15):** independent of Tracks 2–3, but
Track 1's A1 node-typing pass must classify the two new ops (`CoeffRat` = scalar
const; `Flows` = sink), and the Track-3 typed egglog schema gains the two
constructors. Land C4 (below) either before A1 or rebase it; coordinate in TODO.

### 2.6 What we deliberately do NOT do

- No numeric color wavefunctions / per-point color sums (MG's factorization instead).
- No leading-Nc flow sampling (Sherpa-style `Color_Integrator`) — irrelevant at these
  multiplicities; CF is exact and tiny.
- No `d`-heavy, sextet, or ε-baryon structures (SM tree needs none; explicit errors).
- No color average factors inside `eval_m2` — averaging (1/Nc² for qq̄, 1/(Nc²−1)² for
  gg, plus helicity averages) stays a cross-section-layer concern (`sigma_*`), matching
  the MATRIX1 comparison convention.

## 3. Validation strategy

Per the note-12 lessons: establish the bit-exact oracle *first*, compare
per-diagram/per-flow, don't trust aggregate ratios.

1. **CF-matrix oracle**: parse the `DATA (CF(I,J)…)` block (and NCOLOR/NGRAPHS) out of
   each process's `matrix1_orig.f`; assert our exact matrix matches (float-parse
   tolerance 1e-15 — MG prints rationals as decimals; also hand-pin
   `u u~ > u u~` and `g g > g g` matrices as exact-rational fixtures in unit tests).
   This validates C1–C3 **before any evaluator change exists**.
2. **JAMP-level probe**: extend the `amp_probe.f.in`/`compare_amps.py` rig to dump
   `JAMP(i)` per helicity alongside `AMP(k)`; matching per-flow complex values
   bit-for-bit is the debugging workhorse for C4/C5 (requires matching MG's basis
   sort order, §2.4).
3. **Existing 11-process net**: unchanged results, stand-in `color_factor` deleted
   (CF now computed; NCOLOR=1 path bit-for-bit by the op-order rule in §2.5).
4. **New reference processes** (registry additions to `gen_amplitude.py` /
   `build_amplitude.sh`, wrapper is already generic), in difficulty order:
   - `uux_to_uux` — NCOLOR=2, s⊕t gluon exchange + identical quarks: the #8 blocker.
     Exercises T·T Fierz, interference CF off-diagonals, fermi signs across flows.
   - `gg_to_ttx` — external octets, `f(1,2,3)` ggg vertex feeding a quark line
     (f→trace rules), NCOLOR=2.
   - `gg_to_gg` — the stress test: 4-gluon vertex = first multi-color-structure
     vertex (per-vertex chain expansion ×3), NCOLOR=6, pure-f algebra. Also likely
     to finally exercise `MetricNegI`-adjacent rootings (validation-sprint note).
5. **Algebra unit tests**: Casimir/Dynkin closures vs the `repr/color.rs` constants,
   `f f` contraction identities (`f(a,x,y)f(b,x,y) = Nc δ^{ab}` → `2·Nc·Tr(ab)` form),
   canonicalization idempotence, conjugation involution, overflow tripwire.

## 4. Session plan (`color-flow` sprint)

Gate for every session: `cargo test` + the 11-process `validate_helas_mg` net
(bit-for-bit until C5 adds multi-flow processes at REL_TOL 1e-12).

- **C1 — `ufo-color-parse`**: `ufo/color.rs` atom grammar + `Vertex.color` typed;
  `Identity` rep-resolution (3/3̄ direction, octet `2·Tr`); SM-wide parse coverage
  test (the 5 distinct SM structures as goldens); SM blob regen if serialization
  changes. Small session.
- **C2 — `color-algebra`**: `ColorCoeff`/`ColorTensor`/`ColorString`/`ColorFactor` +
  the full MG rule set with exact rationals; canonical form; unit-test suite (§3.5).
  The meaty session; pure compile-time code, no evaluator contact.
- **C3 — `colorize-basis-cf`**: diagram walk (slot-ordered rays, propagator summed
  indices, conjugate-rep assertion), chain expansion, basis accumulation with
  MG-compatible deterministic ordering, exact CF matrix; **CF oracle test** parsing
  `matrix1_orig.f` across all existing outputs (all NCOLOR=1: CF ∈ {1,3,9}) plus
  early `uux_to_uux` MG output generation for a real NCOLOR=2 fixture.
- **C4 — `eval-flows`**: per-`color_idx` `VertexInfo`; per-chain rooting;
  `Op::Flows` + `Op::CoeffRat` (s-expr, egglog round-trip, op-coverage lists);
  `fold.rs` rational resolution; CF-weighted `eval_m2` with the NCOLOR=1
  bit-for-bit op-order rule; delete the `color_factor` stand-in. Gate: 11-process
  net bit-identical.
- **C5 — `mg-color-validation`**: JAMP probe; `uux_to_uux`, `gg_to_ttx`, `gg_to_gg`
  references + enforcement in `validate_helas_mg` (≤1e-12); timing-table addendum
  (CSE sharing across flows should keep the cost growth ≪ NCOLOR×).
- **C6 — close-out**: TODO/notes/memory updates; record the unblocks (hadronic
  pp→ll needs PDF + n-body LIPS next; `mg-validation-coverage` #8 done); file the
  LHEF follow-up (§5).

## 5. Follow-ups deliberately out of scope

- **LHEF color tags** (`event-output-lhef`): needs MG's *leading-Nc* flow
  decomposition (`color_flow_decomposition` / `get_color_flow_string`) to assign
  (color, anticolor) integer pairs per external leg — a separate small feature on
  top of the basis machinery built here.
- **Rationalizing `Coeff(f64)`** (Lorentz structure coefficients, symmetry factors)
  onto `CoeffRat` — optional cleanup once `CoeffRat` exists.
- **PDF interface + n-body LIPS** for hadronic σ (tracked in TODO).
- **Color-flow sampling** for high multiplicity (Sherpa-style) — not needed.

## 6. Sprint debrief (2026-07-12)

How it ran: nine planned sessions (C3/C4/C5 each split in two for tighter
checkpointing) plus one inserted debugging session (C5c), each executed as an isolated
agent with a fixed scope, a hard validation gate, and a stop-rule ("fix only if
unambiguous, else pin the finding and report"). Every commit landed with the full gate
green; no session committed a regressing state. The splits paid for themselves: C3a/C3b
separated pure-Rust work from MG-environment work, and C4a landed the AST plumbing
provably inert (byte-identical net before/after), so when C4b's evaluator change ran
into trouble the suspect diff was minimal.

Observations worth carrying into future physics-feature sprints:

1. **Every validation layer has a blind spot; enumerate them up front.** The CF matrix
   is a Gram matrix, invariant under a uniform index transpose — so the CF oracle
   (23/23 green) provably could not see the conjugation-convention bug that flipped
   physical signs. |M|² is blind to global phases; per-diagram AMP ratios are polluted
   by benign convention spread. The sprint's real bug was found exactly one layer down
   (per-flow complex JAMPs). When planning oracles, ask of each: *what error class is
   this one provably unable to detect?* — and make sure some other layer covers it.

2. **Design-note convention claims are hypotheses, not facts.** §2.4's original "the
   conjugation is automatic" claim read plausibly and survived five sessions' gates —
   because those gates could not falsify it; the first process mixing f- and
   T-structures did. Where a design asserts a convention comes for free, schedule the
   probe that would falsify it early, and treat "all gates green" as "not yet
   contradicted," never as "confirmed."

3. **Keep a known-wrong informational trial in the net.** `uux_to_uux` sat unenforced
   in `validate_helas_mg` from C3b onward, visibly wrong by ~0.6 while color was
   stripped; the moment the multi-flow evaluator went live it snapped to 5.6e-14. A
   free, always-on end-to-end signal, set up before the feature could possibly work.

4. **Bit-for-bit gates confine debugging.** The NCOLOR=1 op-order rule (CF applied
   after the helicity sum) kept the 11-process net bit-identical through every
   evaluator change, so any digit change anywhere was an unambiguous regression
   signal; REL_TOL judgment stayed reserved for genuinely new processes.

5. **Expect the stress-test process to surface someone else's bug.** The first process
   to exercise a code path (4-gluon multi-structure expansion, imaginary VVVV
   coupling) is as likely to find a pre-existing defect as a sprint defect —
   `gg_to_gg`'s failure decomposed into one of each. An explicit stop-rule for "the
   residual is not ours" kept the sprint from scope-creeping into Lorentz-layer
   surgery, and the precise localization made the deferral cheap to pick back up.

6. **Verify the fix arithmetically before implementing it.** C5c reconstructed |M|² by
   hand from dumped MG amplitudes under candidate colorize hypotheses and confirmed the
   MG-convention combination reproduced the reference value exactly — before touching
   production code. This converts "plausible sign fix" into "derived fix" (note-12's
   oracle-first lesson, restated for symbolic-layer bugs).

7. **Session hand-off notes are load-bearing.** The T-transpose observation traveled
   C3b → C4b → C5a → C5b as a watch item and was the thread that unraveled the real
   bug; the "for downstream sessions" section of each report cost little and paid
   repeatedly. Same for interrupted sessions: resuming from transcript with a
   mandatory `git status`/`git log` reconciliation step worked cleanly all three times
   it was needed.
