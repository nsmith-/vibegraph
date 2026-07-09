# Geometric picture for `repr.rs`

> **⚠ Superseded as design input (2026-07-08) by `13-typed-repr-conventions-design.md`.**
> `13` adopts the intertwiner/form *typing discipline* from this note but deliberately does
> **not** lift the blanket `Intertwiner<In,Out>` trait / product-bundle machinery into the eval
> layer (too much churn/altitude for the payoff). Kept for the geometric motivation and history.

**Status:** Complete, partially oudated by updates made following `09-ufo-aloha-type-matrix.md`. Implemented in `5cd7fd5fcf62ac686a3fdfccc42b140fd88ae5de`.

It would be nice to capture some hint of the geometric picture of the field
theory we are simulating in our eventual repr.rs implementation of the various
representations of spin(1,3) x (SM gauge groups, or even BSM gauges if the UFO
model provides them) that we need in our library. The motivation being that the
geometric language is quite amenable to functional programming: if we define the
right generic functions and types, the type checker becomes a theorem prover
that can help ensure certain invariants hold in our program, reducing the unit
testing burden and incresing confidence in correctness. The following is a short
note with some strategies for implementing the main representations and
intertwiners we may need to compute helicity amplitudes in a geometrically
natural way, with the goal of eventually implementing these ideas in Rust.

Key strategies:

1. Split Lorentz and gauge reps — LorentzRepr and GaugeRepr traits separately, combined as a product type for wavefunctions. This mirrors the Spin×G bundle structure (or Spin^c for SM).

2. Intertwiners as a trait — left_current/right_current are already instances. Generalize to Intertwiner<In, Out> covering γ^μ (vector), σ^μν (tensor), scalar (Majorana ε), etc.

3. UFO vertices as Vertex3<R1,R2,R3> — a color structure × Lorentz structure × coupling scalar. This matches MG5's internal decomposition and is naturally a Hom(R1⊗R2⊗R3, ℂ).

4. Propagator<R> trait — all propagators (Dirac, massless/massive spin-1, scalar) as one generic interface T*M → End(R).

The WeylBasis you have is already the geometrically natural choice since S_L/S_R are manifest.

## The bundle picture

The full structure group is **Spin(1,3) × G** (or Spin^c in the SM due to
hypercharge), acting on sections of associated vector bundles over momentum
space (or spacetime).  Each "wavefunction" is a section of one such bundle:

| Object | Bundle | Rep |
|--------|--------|-----|
| `DiracWf` (LH) | S_L ⊗ V_charge | (½,0) ⊗ ρ_q |
| `DiracWf` (RH) | S_R ⊗ V_charge | (0,½) ⊗ ρ_q |
| `VectorWf` | T*M ⊗ ad(P_G) | (½,½) ⊗ adj |
| scalar off-shell | trivial ⊗ V_charge | (0,0) ⊗ ρ |

γ^μ is not a section but a **bundle map** S_L → T*M ⊗ S_R (and vice versa) —
an invariant intertwiner guaranteed by (½,0) ⊗ (½,½) ⊃ (0,½).

---

## Strategy for `SpinorRepr` / representation traits

### 1. Separate the Lorentz rep from the gauge rep

```rust
trait LorentzRepr<F: Real> {
    type Spinor: Copy;            // section of the Lorentz bundle
}

trait GaugeRepr {
    type Color: Copy;             // section of the gauge bundle (e.g. [C;3] for SU(3))
    type Coupling: Copy;          // element of Hom(rep, rep) — vertex factor
}
```

A full wavefunction is then `(B::Spinor, G::Color)` — a section of the product
bundle.  This mirrors the Spin^c amalgamation: you can't always split them, but
the product type encodes the direct-product case cleanly.

### 2. Intertwiners as trait methods, not free functions

The current `left_current` / `right_current` are already intertwiners
`S* ⊗ S → T*M`.  Generalise:

```rust
trait Intertwiner<F: Real, In: LorentzRepr<F>, Out: LorentzRepr<F>> {
    /// Apply the intertwiner: Out::Spinor = self applied to In::Spinor
    fn apply(v: &In::Spinor) -> Out::Spinor;
}
```

Specific instances:
- `GammaL`: `(S*, S) → T*M`  (left current — already implemented)
- `GammaR`: `(S*, S) → T*M`  (right current)
- `GammaV`: `T*M ⊗ S → S`   (vector × spinor → spinor, for off-shell fermion currents)
- `Sigma`: `(S*, S) → T*M ∧ T*M`  (tensor current, σ^μν)
- `Epsilon`: `S ⊗ S → C`     (Lorentz scalar / charge conjugation, for Majorana)

### 3. UFO coupling objects as elements of `Hom(rep1 ⊗ rep2 ⊗ rep3, C)`

UFO vertices are rank-3 tensors in the space of representations meeting at the
vertex.  Model them as:

```rust
struct Vertex3<R1, R2, R3> {
    color: ColorStructure,       // SU(3) Clebsch — e.g. delta_{ij}, f^{abc}
    lorentz: LorentzStructure,   // which intertwiner combination
    coupling: C<f64>,            // overall coefficient from UFO
}
```

`LorentzStructure` becomes an enum or trait object over the intertwiner types
above.  This is exactly the decomposition MG5 uses internally (color × Lorentz
× coupling).

### 4. Basis independence via `SpinorRepr` (already present)

`WeylBasis` is the right default — LH/RH components are manifest, matching the
bundle decomposition S = S_L ⊕ S_R.  A `DiracBasis` impl would be a unitary
rotation of the same sections; the intertwiner trait doesn't change.

---

## Concrete next steps

1. **`ColorRepr` trait** — analogous to `SpinorRepr` but for SU(3) color:
   fundamental `[C;3]`, adjoint `[C;8]`, trivial `C`.
2. **`GaugeVertex` intertwiner** — `Hom(color1 ⊗ color2, color_out)` for each
   color structure (δ, T^a, f^abc, d^abc).
3. **`LorentzStructure` enum** — covers all structures appearing in SM UFO
   vertices: `VectorFF`, `AxialFF`, `TensorFF`, `ScalarFF`, `VVVV`, etc.
4. **`Propagator<R>` trait** — for each rep, the map `T*M → End(R::Fiber)`,
   i.e. the Feynman propagator in that representation.  Massless spin-1 in
   Feynman gauge, massive spin-1 in unitary gauge, Dirac propagator, scalar
   propagator — all as instances of one trait.
