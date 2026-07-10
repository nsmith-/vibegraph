# 13 — Typed Repr Conventions: the intertwiner-basis + peephole design

**Status:** Design anchor (2026-07-08). Backs `typed-repr-conventions` (TODO cleanup
task 4).
**Supersedes as design input:** `08-repr-geometry.md`, `09-ufo-aloha-type-matrix.md`,
`10-lorentz-runtime-eval-plan.md`, `11-variance-flow-duality.md`. Those notes are kept for
history/implementation reference; this note is the current design of record. `12-helas-
continuum-bugfix-journey.md` (the bug journey) is *not* superseded — it is the motivating
evidence.

---

## 0. One-paragraph summary

Reorganize the Lorentz amplitude evaluator around two tiers: a **general typed node tree**
(the semantics — handles *any* UFO structure, dim-6/8 EFT included, with variance/adjoint
carried in the types) and a **peephole / instruction-selection rewrite layer** that, at
graph-compile time, recognizes common vertex patterns and dispatches them to **fused,
hand-optimized intertwiner kernels** parameterized by their coordinate in a small irreducible
basis (e.g. FFV → `[g_L, g_R]`). The peephole layer never has to be exhaustive: coverage is
guaranteed by the general tier, so exotic EFT structures simply fall through and evaluate
generically. Each fused kernel ships with a property test asserting `fused == generic` on
random inputs — replacing the current "add a node when a validation process breaks" growth
model with a local, machine-checkable oracle. This decouples the two goals that were
entangled in earlier notes: **type-safety comes from typing the general nodes** (needs no
closed primitive set); **fusion/performance comes from opportunistic peepholes**.

---

## 1. The formalism decision

TODO task 4 framed a fork: "HELAS F/I/O/V/S call style vs. the `intertwiner` abstraction."
Two decisions settle it.

### 1a. Primitives are irreducible intertwiners, not HELAS routines

In `S* ⊗ S → V` the vector appears with exactly **two** invariant intertwiners — the left
current `γ^μ P_L` and the right current `γ^μ P_R`. Every SM FFV structure is a point in that
2-D span, in `[g_L, g_R]` coordinates:

| UFO | structure | `[g_L, g_R]` |
|---|---|---|
| FFV1 | `γ^μ` | `[1, 1]` |
| FFV2 | `γ^μ P_L` | `[1, 0]` |
| FFV3 | `γ^μ P_L − 2 γ^μ P_R` | `[1, −2]` |
| FFV4 | `γ^μ P_L + 2 γ^μ P_R` | `[1, +2]` |
| FFV5 | `γ^μ P_L + 4 γ^μ P_R` | `[1, +4]` |

The `1, −2, +2, +4` numbers are **not** Clebsch–Gordan coefficients — they are the SM's
phenomenological chiral charges (roughly `T³ − Q sin²θ_W` combinations). The
representation-theoretic content is one level up: the *dimension* of the intertwiner space is
2, fixed by rep theory, and the FFVn are named lattice points in it. HELAS's own modern form
already agrees — `iovxxx(fo, fi, v, [g_L, g_R])` *is* that 2-D space in coordinates.

The current compiler already half-exploits this: it carries each UFO `LorentzTerm.coeff`
separately and lowers `Gamma·ProjM` / `Gamma·ProjP` into distinct eval nodes, so FFV3 is
already stored as two coefficient-weighted terms in the `{left, right}` basis. So we do **not**
chase a 1-1 map to ALOHA *routine names* (`FFV2_4_3` etc.); we chase a 1-1 map to
**irreducible intertwiners**, and coefficient vectors reconstruct the named structures. ALOHA
stays the *oracle* (validate that `{left, right}·coeff` reproduces each `FFVn`), not the
*interface*.

Optimization payoff: FFV3 today builds two independent subtrees and sums them, duplicating the
shared γ-contraction. A single `[g_L, g_R]`-parameterized current computes the two cheap
chiral projections of one spinor pass and takes the linear combination — one contraction, not
two, fewer temporaries.

### 1b. Typed two-level enum, not a blanket `Intertwiner` trait

Adopt the *typing discipline* of the intertwiner/form picture without the trait-level
generality:

- **outer node level = output type carrying variance/adjoint**: `ScalarOut` /
  `VectorOut<V>` / `SpinorOut<Ket|Bra>` (+ tensor later). This *is* the codomain `Out` of an
  `Intertwiner<In…, Out>`, made concrete as an enum tag.
- **inner level = the primitive** (granular general op, or a fused peephole kernel).

We deliberately do **not** lift the full `08`/`09` machinery — `Intertwiner{2,3,4}Leg` generic
traits, the Lorentz×gauge product bundle, the symplectic Weyl ε — into the eval layer. Those
buy compile-time proof at a large churn/altitude cost that is not justified here. Gauge/color
belongs to `color-flow`; the Weyl ε belongs to a future Weyl refactor; design the sign in (per
`11`) but do not use it yet.

> **REVISION (2026-07-09, `cleanup-refactor`): drop the two-*level* enum; keep a flat, clearly-
> labeled op set.** Two reasons, learned after the typed-propagator-seam work landed:
> 1. **Layout cost.** A nested `enum Outer { VectorOut(V), SpinorOut(Adj), … }` carries *two*
>    discriminants (Rust niche-packs only opportunistically), so it is strictly larger and
>    cache-worse than the flat, dataless `Op` (`op.rs`) that already fits in a byte. The eval AST
>    is an arena walked in the VEGAS inner loop — node size matters.
> 2. **The tag is redundant.** The variance/adjoint the outer level was meant to carry *already
>    live on the register* after the seam work: variance via `WaveformSlot::Vector`/`VectorCo`,
>    adjoint via `FermionIn`/`FermionOut`. A `VectorOut<V>` node tag would only re-encode what the
>    produced slot already says — paying memory to duplicate a fact.
>
> So the "output type" axis stays a **naming/documentation discipline on the flat `Op` set**, not
> a second enum level. Nothing to tear down: the two-level node was only ever planned. The typing
> that prevents duality-boundary bugs is the slot's variance/adjoint (already done); the flat `Op`
> enum is the node representation. Concretely this rewrites §7 step 2 (below): no two-level node —
> instead **factor the kernels out of `run.rs` and name them 1-1 with `Op`.**

---

## 2. The intertwiner basis is (mostly) enumerable from the leg reps

> The **spinor/index structure** of the basis is completely fixed by the leg reps via classical
> invariant theory. What the reps alone do **not** fix is the number of **momentum (derivative)
> insertions** — a graded tower bounded by renormalizability (operator mass-dimension ≤ 4).

**Complete basis = {invariant index-structures saturating the leg indices} × {momentum
insertions up to the dimension bound}.**

The index-structure part is a finite enumeration from the only Lorentz-invariant tensors that
exist: `δ` (identity), `g^{μν}` (metric), the Clifford generators `{1, γ⁵, γ^μ, γ^μγ⁵, σ^μν}`,
`ε^{μνρσ}` (Levi-Civita, parity-odd), and `P^μ` (momenta, the graded piece). List the free
indices the legs demand, saturate them every independent way with that toolkit, reduce by the
symmetries below, truncate the momentum grade by renormalizability.

### 2a. Glossary (terms used above)

- **Grade / graded momenta.** Count structures by number of momentum factors `P^μ`; the space
  is a stack of levels (grade 0 = no momenta, grade 1 = one, …) that don't mix. Each extra `P^μ`
  is a spacetime derivative `∂^μ`, raising the operator's mass-dimension by 1. Reps fix the index
  skeleton; renormalizability (dim ≤ 4) truncates the grade tower. This truncation is a *physics
  input*, not a rep-theory fact — hence it is the extension point for EFT (§4).
- **Bose/Fermi reduction.** Identical legs force (anti)symmetry under exchange; index structures
  related by such a swap are not independent. E.g. VVVV's three naive metric-pairings collapse
  under full Bose symmetry + color to the single physical quartic. Momentum conservation
  (`Σ p = 0`) prunes derivative terms the same way.
- **Schouten / Fierz identities.** Linear relations that make a naive basis over-complete, special
  to 4D. *Schouten*: you cannot antisymmetrize 5 spacetime indices in 4D, giving identities among
  `g·ε` products. *Fierz*: the 16 gamma covariants are complete, so a product of two spinor
  bilinears re-expands over the exchanged pairing. Don't bite for the 2-/3-point SM vertices below,
  but must be quotiented out at 4-point and wherever `ε`/tensors appear.
- **YM structure.** The Yang–Mills triple-gauge vertex
  `g^{μν}(p₁−p₂)^ρ + g^{νρ}(p₂−p₃)^μ + g^{ρμ}(p₃−p₁)^ν` — three vector legs, one derivative
  (grade 1), fully Bose-symmetric. It is *the* grade-1 VVV intertwiner (what `LowerVout` computes).

### 2b. Renormalizable-SM enumeration (per output leg)

| Family | spins | output | basis | dim | ∂ | current node(s) |
|---|---|---|---|---|---|---|
| **FFV** | ½,½,1 | V | `{γ^μP_L, γ^μP_R}` | 2 | 0 | `GammaVout`, coeff `[g_L,g_R]` |
| | | F | `{γ̸P_L, γ̸P_R}` on ket | 2 | 0 | `GammaIout`/`GammaOout` |
| **FFS** | ½,½,0 | S | `{P_L, P_R}` | 2 | 0 | `ProjM`/`ProjP` |
| | | F | `{P_L, P_R}` on ket | 2 | 0 | `ProjMAmp`/`ProjPAmp` |
| **VVS** | 1,1,0 | S | `{g^{μν}}` | 1 | 0 | `Metric` |
| | | V | `{g^{μν}→V^μ}` | 1 | 0 | `MetricVout` |
| **VVV** | 1,1,1 | V | `{YM}` | 1 | **1** | `LowerVout` |
| **VVVV** | 1,1,1,1 | V | `{g^{μν}g^{ρσ}, g^{μρ}g^{νσ}, g^{μσ}g^{νρ}}` | 3 | 0 | (metric-pairings) |
| **VSS** | 1,0,0 | V | `{(p₁−p₂)^μ}` | 1 | **1** | (P-node) |
| **SSS/SSSS** | 0,… | S | `{1}` | 1 | 0 | identity·coupling |

Two facts this makes concrete: **(1)** VVV *must* carry a derivative (three vector indices can't
form a scalar without a fourth index; `g` supplies two, `ε` needs four) — this is *why* VVV
needed `LowerVout` while VVS did not, the exact asymmetry that bit us in note 12. **(2)** FFV is
*one* 2-D coefficient space regardless of whether the output is the vector or a fermion — the
two-level node's "one node per (structure × output-leg type)": the **basis** is keyed by the
leg-rep multiset; the **realization** is keyed by which leg is output (its variance/adjoint).

---

## 3. Why the enumeration is a *rewrite catalog*, not the whole design

A closed enumerated primitive set is only closed under the renormalizability truncation. We do
want **dim-6 (and potentially dim-8) SMEFT UFOs**, where the basis blows up: higher grade,
`ε`-structures, `σ^μν p_ν` dipoles, tensor currents. Making the typed intertwiners the *sole*
representation would therefore be a trap. Hence the two-tier architecture (§0), a general typed
IR + instruction selection:

1. **General typed node tree = the semantics.** Every UFO structure lowers to granular nodes
   (`Gamma`, `Proj`, `Metric`, `P`; and — for EFT — `Epsilon`, `Sigma`, arbitrary-grade momentum
   insertions). Complete for any UFO. **Variance/adjoint typing lives here**, so correctness at
   the duality boundaries (the note-12 bug class) is enforced on the general path, whether or not
   any optimization fires.

2. **Peephole / tree-pattern rewrite = instruction selection.** A hand-written pattern table
   keyed on the UFO structure signature (maximal-munch, BURG-style, minus the machinery). A match
   emits a fused typed kernel and reads off the basis coordinates (`[g_L, g_R]`, …). No match →
   fall through to tier 1.

The key property: **the peephole layer never has to be exhaustive.** It is pure optimization;
coverage is guaranteed by tier 1. The renormalizable SM vertices dominate every diagram's
runtime and get kernels; a rare dim-8 dipole eats the generic cost and nobody cares.

**This decouples type-safety from fusion.** Earlier notes entangled "fewer primitives *and*
typed." They pull apart: type-safety comes from typing the general nodes (no closed set needed —
a dim-8 operator gets the same discipline as `γ^μ P_L`); fusion comes from the peephole layer,
opportunistically, on whatever patterns are worth a kernel.

### 3a. Mechanics

- **Matching is at graph-compile time**, once per (vertex, output-leg) when building the
  `WaveformSlot` program — *not* per phase-space point. Runtime executes the selected fused ops;
  zero matching cost in the VEGAS inner loop.
- **Coordinate read-off**: express the vertex's term list in the kernel's known basis and take the
  coordinate vector. Clean failure condition: if the terms don't lie in that basis (extra grade,
  an `ε` the kernel doesn't model), the read-off fails and the structure falls through to generic.
- **Per-kernel property test `fused(random) == generic(random)`.** This is the linchpin. It
  replaces "add a node when a validation process breaks" (exactly how the
  `MetricVout`/`LowerVout`/`PropagateLowered` stopgaps accreted) with a local, machine-checkable
  oracle per kernel. Note 12's lesson — every convention bug lived at a hand-coded duality
  boundary — becomes tractable: each hand-coded kernel now has a mechanical definition of correct
  (equal the typed general path on random momenta). The dangerous hand-coding stops being
  trust-me and becomes test-me.

The §2b enumeration is thus the **catalog of rewrite targets** for the common cases, explicitly
**non-exhaustive**. The grade/parity truncation flagged in §2 is a *feature*: it is precisely the
boundary between "has a peephole kernel" and "generic fallback." SMEFT work adds kernels above the
line (or doesn't, and eats the generic cost).

---

## 4. Propagator stays separate from vertex

HELAS/ALOHA fuse the propagator into the off-shell current (`jioxxx`/`ffv2_3` bake
`1/(q²−m²+imΓ)` in). We keep it separate. The clinching argument is a concrete asymmetry, not
abstract combinatorics:

> The **amplitude-closing vertex does not propagate its output** — it contracts onto an external
> wavefunction. So the *un-fused* vertex is required no matter what.

Given the unfused form is needed anyway, fusing forces carrying *both* fused (internal) and
unfused (closing) versions — precisely ALOHA's N×M routine explosion. Separation gives N + M.

Caveat: separation is clean only if the **seam is typed.** The propagator numerator's variance
(fermion `q̸+m`, massive-vector `−g^{μν}+q^μq^ν/m²`) meets the vertex output's variance at what is
currently an untyped boundary — the whole `MetricVout`/`LowerVout`/`PropagateLowered` mess. So:
keep the propagator separate, and make it a **typed musical-iso node** (the propagator numerator's
raise/lower is `♯/♭` under the metric form). That is the variance-parameterization the refactor
delivers.

---

## 5. Form/adjoint discipline and the three-axis terminology fix (from note 11)

The type-safety half of the refactor adopts note 11's form-duality unification, but *only the
invariants*, not a blanket `Paired`/`Intertwiner` trait. `Variance` and the spinor bra/ket duality
are two realizations of one gadget: a space with a nondegenerate form, giving a musical isomorphism
`V ≅ V*`; the "two sides" are `V`/`V*`, and index-raise / Dirac-adjoint *is* the iso. The payoff: a
node's output variance is *derived* (the adjoint of the map w.r.t. the forms), not hand-set;
contraction only type-checks between dual sides; the propagator can't double-apply or drop `g`.

**Do the terminology fix first** — three orthogonal axes, currently conflated under "Flow":

- **Variance** (index up/down, metric `g`) — vectors/tensors — *symmetric* bilinear form,
  ℂ-linear musical iso. Lives at `helas::repr::lorentz` (`Contravariant`/`Covariant`).
- **bra/unbar (Dirac adjoint)** — spinors only — *Hermitian* sesquilinear form (`bar()` = ψ†γ⁰),
  conjugate-linear iso. This is what today's `SpinorFlow`/`FlowIn`/`FlowOut` (`repr/lorentz.rs`)
  and runtime `Flow`/`LegFlow` (`root_lorentz.rs`) *actually* are. **Rename** off "Flow": trait
  `SpinorAdjoint` with sides `Ket`/`Bra` (parallel to `Contravariant`/`Covariant`). Belongs next
  to `Variance` (note 11's placement is right).
- **Flow** (in/out, sign of `e^{∓ipx}`, HELAS nsf/nss/nsv) — *all* wavefunctions — **not** a
  musical iso. Reserve the name "Flow" for this; it lives at `wavefn`.

Invariant to enforce: for spinors, bra/ket is **derived** from `Flow ⊕ Charge` via the
rooting-chosen fermion arrow (`crossed` records the mismatch vs physical momentum) — not a free
fourth axis. Flow and Charge are the independent inputs. A future Weyl refactor adds the
*alternating* (symplectic ε) form, `♭∘♭ = −id`; design the sign in now, don't use it.

---

## 6. Scope for this refactor — decision (i)

**Land now:** the typed general SM node set + the peephole layer + the property-test harness, on
the existing 11-process MG regression net. The general tier's variance/adjoint typing and the
terminology rename are in scope.

**Documented extension point, not built now:** genuine dim-8 completeness in the general tier
needs `Epsilon` (`ε^{μνρσ}`), `Sigma` (`σ^μν`), and arbitrary-grade momentum handling — primitives
the SM tree never exercised, and for which we have **no reference coverage**. SMEFT is therefore
*architecturally supported* (structures fall through to a general tier designed to host these
nodes) but **not yet exercised**. This matches the project's incremental-validation ethos (never
reorganize conventions without reference coverage) and the peephole/property-test architecture is
exactly what makes adding EFT kernels later safe. Revisit if a dim-6 model needing `σ^μν` dipoles at
LO becomes imminent.

---

## 7. Implementation sketch (aligns with TODO task 4)

*Steps 1–3 revised per the §1b REVISION (flat op set, not a two-level enum) and the 2026-07-09
re-plan (harness first; simplify before optimize; Stage A may defer). Status flags reflect
`cleanup-refactor`.*

1. **Terminology rename ✅ Done** (`84bd2d3`): `SpinorFlow`/`FlowIn`/`FlowOut` →
   `DiracAdjoint`/`Ket`/`Bra` in `repr/lorentz.rs`; runtime `Flow`/`LegFlow` → `Adjoint`/`LegAdjoint`
   in `root_lorentz.rs`. "Flow" reserved for the in/out axis at `wavefn`.
2. **Typed propagator seam ✅ Done** (`aed1a80`, `402322c`): `VectorWf` variance-parameterized;
   `WaveformSlot::VectorCo` carries the index-lowered currents; the propagator dispatches on slot
   variance. Variance/adjoint now live **on the register**, not a node tag. Killed
   `Op::PropagateLowered` + `PropInfo.lowered_storage` + `has_flipped_vector_out`.
3. **Property-test harness (Stage 0) ✅ Done** (uncommitted): `helas/eval/prop_harness.rs` —
   typed random-input generators + variant-strict `slots_approx_eq` + the `check_agree` driver.
   The shared toolbox for steps 5a/5b.
4. **Kernel factor-out + 1-1 `Op` naming ✅ Done** (`cleanup-refactor`, `e2ba8e7`; Stage 0 harness
   committed first as `820614e`). New `helas/eval/kernel.rs` (`pub(crate)`) holds the Lorentz-
   primitive kernels; each Lorentz `Op` maps 1-to-1 to a kernel fn named for it (snake_cased), so
   `run::apply` collapses to a match of `kernel::<op>(children)`. The N-to-1 cases got thin per-Op
   wrappers over shared private helpers (`gamma_iout`/`gamma_oout`→`off_shell_fermion_current`;
   `proj_m`/`proj_p`→`chiral_project`; `proj_m_amp`/`proj_p_amp`/`identity_amp`→
   `scalar_bilinear_current`; `metric_neg_i`; `pmom`/`pmom_out`; `propagate`→`propagate_core`);
   `metric_contract`→`metric`. Structural/const/algebraic ops (`External`/`Mul`/`Add`/`Coupling`/…)
   stayed inline in `apply`. Harness smoke test repointed off `metric_contract`. **Pure structural →
   bit-for-bit** (175 lib + `validate_helas_mg` 11/11, max_rel_diff ≤ 6.25e-13). Perf baseline
   (`benches/amplitude_eval`, ee→μμ, release, N=10k): ~14.0 µs/eval — the Stage-A profiling anchor.
5. **Two equivalence stages on the harness — Stage B done, Stage A deferred:**
   - **5b. Stage B ✅ DONE (2026-07-09, `cleanup-refactor` d537e1e/3c60fe0/5c95d5b): one physical
     contravariant vector convention; `WaveformSlot::VectorCo` deleted.** The corrected-scope
     analysis (global amplitude-load-bearing rewrite) had shown a *producer-only* change with the
     producers held at their covariant values fails (5/11 MG broken); what resolved it was choosing
     the producer *phases* per composite:
     - `propagate(metric_vout(v), m>0)` ≡ `propagate(Vector(+v), m>0)` and
       `propagate(lower_vout(v), m>0)` ≡ `propagate(Vector(−v), m>0)` are IEEE-exact sign
       shuffles — certified **bit-exact** on 10k random inputs with the Stage-0 harness before
       any rewiring. The old −1(`MetricVout`)/+i(covariant-massive) pair equals the new
       +1/−i(plain) pair exactly.
     - The single genuinely inequivalent case, the massless `VectorCo` branch (relabel, no raise),
       differs from the physical convention only in the propagated current's **time component**;
       node dumps at the MG validation points show both the conserved sink current's J⁰ (CM
       s-channel, massless fermions) and the assembled VVV current's time component are exactly
       0.0, so the amplitudes are bit-identical there. (The old relabel convention was only ever
       valid on that corner; a future t-channel massless VVV rooting gets the physical — correct —
       time component.)
     - **New convention:** every producer emits the physical contravariant current with a
       momentum-grade sign — +1 for P-less structures (VVS: `MetricVout` = identity, `g^{μν}V_ν
       = V^μ`), −1 for P-carrying ones (VVV: `LowerVout` = negation) — the UFO coupling carries
       the vertex i, the propagator its −i. 4 vector-propagator branches → 2; `VectorCo` slot,
       covariant `propagate_core` arms, `rand_vector_co` all deleted.
     - **Verification:** per-diagram × per-helicity dumps for ee_to_zh / ee_to_wpwm / ee_to_tatah
       bit-identical to the pre-Stage-B baseline (ee_to_zh got its own MG AMP probe in
       build_amplitude.sh); 175 lib + `validate_helas_mg` 11/11, rel_diffs unchanged
       digit-for-digit; ALOHA pin re-derived (vibegraph VVS current = +i × `VVS1P1N_1`).
       Perf re-checkpoint ~14.1 µs/eval (unchanged).
   - **5a. Stage A (fuse, deferred pending profiling):** peephole/instruction-selection rewrite +
     coordinate read-off for the §2b catalog; per-kernel oracle `fused(random) == generic(random)`.
     This is a *performance* play. **Profile `run.rs` first** — the forward-pass `Vec` churn,
     `C<F>`-vs-`F` multiply, and arena traversal may dominate kernel-fusion wins. Pursue only if
     fusion proves the bottleneck.
6. **Regression net:** the 11 MG-validated processes stay bit-for-bit throughout (task 2 keeps them
   fast).

Design stage before any code (per TODO). This note is the design of record; the §2b table is the
primitive/rewrite-target checklist.
