# Unifying `Variance` and `Flow` as form-induced dualities

> **⚠ Superseded as design input (2026-07-08) by `13-typed-repr-conventions-design.md`.**
> `13` §5 folds in this note's form/adjoint unification and the three-axis terminology split
> (Variance / bra-unbar adjoint / Flow), but adopts *only the invariants* — not a blanket
> `Paired`/`Side` trait. Kept for the form-kind taxonomy (symmetric/Hermitian/alternating) and
> the Weyl-ε roadmap.

> **⚠ Terminology correction (2026-07-07).** Throughout this note "`Flow`" names the
> **bra/unbar (Dirac-adjoint) duality** — ket (u/v columns) ↔ bra (ū/v̄ rows), iso
> `bar()` = ψ†γ⁰, spinor-only. That is the axis this note pairs with `Variance`, and the
> pairing thesis below is correct. **But the *name* is wrong.** "Flow" should denote a
> *different*, universal axis: momentum in/out, the sign of e^{∓ipx} (HELAS nsf/nss/nsv),
> which every wavefunction has and which is **not** a musical iso. So:
> - Read every "`Flow`"/`SpinorFlow`/`FlowIn`/`FlowOut` below as the **bra/unbar / Dirac
>   adjoint** axis, to be renamed `SpinorAdjoint` with sides `Ket`/`Bra` (parallel to
>   `Contravariant`/`Covariant`). *This* is what belongs next to `Variance`.
> - The name "Flow" is reserved for the universal in/out axis (lives at `wavefn`, not a
>   form iso). For spinors, bra/ket is *derived* from Flow ⊕ Charge via the rooting-chosen
>   fermion arrow (`crossed` records the mismatch vs physical momentum) — not a free axis.
>
> The rename and three-axis split are the first step of `typed-repr-conventions` in
> `TODO.md` (cleanup task 4); this note is its design backing for the adjoint↔variance
> unification.

**Status:** Stub / design note. The bra/unbar (Dirac-adjoint) duality currently lives in
`helas/repr/lorentz.rs` as `SpinorFlow`/`FlowIn`/`FlowOut` (already moved out of
`helas/wavefn.rs`), alongside `Variance`, since both are the same abstraction — see the
terminology correction above for the pending rename.

## The idea

`Variance` (`Contravariant`/`Covariant`, in `repr/lorentz.rs`) and `SpinorFlow`
(`FlowIn`/`FlowOut`, in `wavefn.rs`) are two realizations of one gadget:

> a space carrying a **nondegenerate form**, which gives a musical isomorphism
> V ≅ V*. The "two sides" are V and V*; the form *is* the pairing, and the
> index-raise / Dirac-adjoint *is* the iso.

| Abstract piece | `Variance` | `SpinorFlow` |
|---|---|---|
| `Side` marker, involutive | `Contravariant`/`Covariant`, `type Dual` | `FlowIn`/`FlowOut`, `type Opposite` |
| orientation bit | `const COVARIANT: bool` | `const INCOMING: bool` |
| musical iso ♭/♯ : V<S> → V<S::Dual> | `dual()` (raise/lower via g) | `dirac_adjoint()` / `flip_flow()` (ψ ↦ ψ†γ⁰) |
| pairing V<S> × V<S::Dual> → Scalar | `dot()` | `scalar_bilinear` / `vector_bilinear` |
| law | `dual∘dual = id`; `dot` symmetric | `flip∘flip = id`; bilinear dispatched on `INCOMING` |

`DiracWf::scalar_bilinear` already branches on `Flow::INCOMING` purely to order
arguments into `pair(fo, fi)` — i.e. "use the form, pick which side is which."

## The one real difference (the caveat)

The two forms are different *kinds* of form, and a unified trait must carry that:

- **`Variance` → symmetric *bilinear* form** (metric g). `dual()` is ℂ-**linear**
  — note `ComplexVector::dual()` only negates spatial components, no conjugation;
  `dot` is ℂ-bilinear / symmetric.
- **`Flow` → Hermitian *sesquilinear* form** (Dirac form ⟨ψ,φ⟩ = ψ†γ⁰φ).
  `dirac_adjoint` is conjugate-**linear** (it calls `.conj()`); `f̄Γf` is
  sesquilinear.

So the right generalization is not "inner product space" but **"space with a
nondegenerate form,"** where the form is one of the three classical types:

- **symmetric** (orthogonal / metric) — Lorentz vectors. `♭∘♭ = +id`.
- **Hermitian** (sesquilinear) — Dirac flow. `♭∘♭ = +id`, but ♭ conjugates.
- **alternating** (symplectic) — *not present yet*: the two-component Weyl metric
  εαβ, antisymmetric, so `♭∘♭ = −id`.

The alternating case matters for the roadmap: when duality is pushed down to
genuine Weyl (½,0)/(0,½) spinors instead of the 4-component `Bispinor`, the
spinor index-raise is εαβ (symplectic), and the `Dual::Dual = Self` law must
relax to `dualize ∘ dualize = ±id`. Design the trait to allow a sign even though
both current cases use `+`.

## Trait sketch

```rust
enum FormKind { Symmetric, Alternating, Hermitian }

/// One side of a form-induced duality. Involutive: Dual::Dual = Self.
trait Side: Copy + Eq + 'static {
    type Dual: Side<Dual = Self>;
    const PRIMAL: bool;          // generalizes COVARIANT / INCOMING
}

/// A space carrying a nondegenerate form, exposing both sides + the musical iso.
trait Paired<F: Real, S: Side>: Sized {
    type Scalar;                 // F (self-dual) or C<F>
    type Dual: Paired<F, S::Dual, Scalar = Self::Scalar>;
    const FORM: FormKind;
    const DUALIZE_SQ_SIGN: i8;   // +1 for g & Dirac, -1 for Weyl ε

    fn dualize(self) -> Self::Dual;
    fn pair(primal: &Self, dual: &Self::Dual) -> Self::Scalar;
}
```

`VectorRepr` becomes `Paired<F, V, FORM = Symmetric>`; `DiracWf`'s bilinears
become `Paired<F, Flow, FORM = Hermitian>`.

## Cautions

1. **Altitude mismatch.** `Variance` is a pure `repr`-layer marker; `Flow` lives
   at `wavefn` where `DiracWf` also bundles `momentum`/`charge` and `flip_flow`
   leaves momentum untouched. Unify the **marker/form trait**, not the whole
   types — `DiracWf` is a `Paired` *carrier*, not the `Side` itself.
2. **`Flow` is specifically the Dirac/Hermitian (bra↔ket) duality.** It is *not*
   the holomorphic dual (½,0)↔(0,½), and *not* charge conjugation
   (particle↔antiparticle, the `Charge` enum). All three are Z/2 dualities on
   spinors; keep them distinct.

## Payoff

Also tidies `intertwiner` (see `08-repr-geometry.md`): vertex orientations are
exactly "the adjoint of γ^μ w.r.t. these forms," i.e. raising/lowering applied to
maps. A unified form trait gives a principled place to derive those orientations.
