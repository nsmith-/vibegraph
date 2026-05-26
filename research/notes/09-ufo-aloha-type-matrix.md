# UFO and ALOHA type matrix for medium-term implementation

This note proposes a medium-term Rust type system for Lorentz objects based on
how UFO and ALOHA actually enumerate spins and Lorentz structures.

## Why this note

Current code often uses raw arrays like [C<F>; 4], which loses semantic
information at value level (spinor vs vector, incoming vs outgoing, covariant vs
contravariant, basis tags). UFO and ALOHA are also mostly shape plus context,
so we need stronger local typing than the source formats provide.

## Ground truth from UFO and ALOHA

### UFO representation model

UFO particles encode spin as 2s+1, with -1 reserved for ghosts. UFO Lorentz
objects carry a list of per-leg spin codes and a symbolic structure string.

Observed conventions:
- spin = 1: scalar
- spin = 2: fermion (Dirac spinor leg)
- spin = 3: vector
- spin = 4: spin-3/2
- spin = 5: spin-2
- spin = -1: ghost (anticommuting scalar)

### ALOHA object families

ALOHA constructs wavefunction objects and propagators by branching on those
spin codes. The value-level families are:
- Scalar
- Spinor (4 components)
- Vector (4 components)
- Spin3Half (vector-spinor, 4x4 components)
- Spin2 (rank-2 Lorentz tensor, 4x4 components)

ALOHA Lorentz algebra operators in structure expressions include at least:
Gamma, Sigma, Gamma5, C, Metric, Epsilon, Identity, ProjM, ProjP, and momentum
insertions P(mu, i).

## Type matrix

The matrix below distinguishes value carriers (wavefunction-like data) from
operator/intertwiner objects and records the current implementation status
based on the recent HELAS refactor (see `src/helas/*`).

| UFO spin code | Physical family | Typical shape in ALOHA | Proposed Rust value type | Implementation status |
|---|---|---|---|---|
| 1 | scalar | 1 complex component | `ScalarWf<F>` | not implemented (TODO) |
| -1 | ghost scalar | 1 complex component | `GhostScalarWf<F>` | not implemented (TODO) |
| 2 | Dirac fermion | 4 complex components | `DiracWf<F, Flow>` with aliases `InDiracWf<F>`, `OutDiracWf<F>` | implemented — see [src/helas/wavefn.rs](src/helas/wavefn.rs#L1) |
| 3 | vector boson | 4 complex components | `VectorWf<F>` | implemented — see [src/helas/wavefn.rs](src/helas/wavefn.rs#L1) |
| 4 | spin-3/2 (Rarita–Schwinger) | 4×4 complex components | `VectorSpinorWf<F, B>` | planned (Phase D) |
| 5 | spin-2 | 4×4 complex components | `Rank2TensorWf<F>` | planned (Phase D) |

### Basis and chirality layering

Do not introduce separate Weyl two-component external wavefunction types in the
first medium-term pass unless required by explicit UFO Lorentz structures. ALOHA
external fermion objects are effectively 4-component in generated routines.

Documentation note: `InDiracWf`/`OutDiracWf` are Dirac bispinor wavefunctions
(4 complex components), matching HELAS/ALOHA external-fermion conventions.

Projector methods should live on these types (or thin extension traits), for
example `proj_m()` and `proj_p()`.

Why UFO/ALOHA names are `ProjM`/`ProjP` rather than `ProjL`/`ProjR`:
- They are named by the sign in `P_\pm = (1 \pm \gamma^5)/2`.
- Left/right chirality mapping depends on convention choices (especially
  `\gamma^5` and metric/sign conventions), while plus/minus eigenprojectors are
  convention-agnostic labels at the symbolic-operator level.
- In common HEP conventions, `ProjM` corresponds to the left-chiral projector
  and `ProjP` to the right-chiral projector.

### Variance policy (vector vs covector)

Medium-term implementation should use a single `VectorWf<F>` value type for
all spin-1 wavefunctions and keep index variance as a documented convention at
contraction sites.

Future plan:
- Introduce explicit variance newtypes (for example `VectorWf` vs `CovectorWf`)
  once the Lorentz AST/operator typing work is in place.
- Add explicit raise/lower helpers tied to typed metric operators.

## Operator and intertwiner matrix

These are not wavefunction value families — they are typed maps (intertwiners)
or unary operators. The codebase now exposes an `Intertwiner` trait and a set
of concrete intertwiners; the current implementation status is listed below.

| UFO/ALOHA operator token | Mathematical role | Rust representation | Implementation status |
|---|---|---|---|
| `Gamma` | full vector current `ψ̄ γ^μ ψ` | family of `Intertwiner` impls (`GammaL`, `GammaR`, `GammaV`) | `GammaL`/`GammaR` implemented; `GammaV` stub — see [src/helas/repr/intertwiner.rs](src/helas/repr/intertwiner.rs#L1) |
| `Sigma` | antisymmetric tensor bilinear `σ^μν` | `SigmaTensor` / tensor-intertwiner | stub/TODO — see [src/helas/repr/intertwiner.rs](src/helas/repr/intertwiner.rs#L1) |
| `Metric` | index contraction / raise-lower | metric helper functions / operator type | helper functions in repr; tighten typing planned (Phase C) |
| `Epsilon` | Levi–Civita / spinor metric | `Epsilon` intertwiner / helper | stub/TODO — see [src/helas/repr/intertwiner.rs](src/helas/repr/intertwiner.rs#L1) |
| `ProjM`, `ProjP` | chirality projectors | methods on `DiracWf`/spinor wrappers or small traits | implemented as spinor-repr methods / used via `GammaL`/`GammaR` |
| `P(mu, i)` | momentum insertion | typed momentum accessor by leg / momentum arg to intertwiners | supported via `LorentzVector` momenta in `Intertwiner::apply` |
| `Identity`, `C`, `Gamma5` | algebraic spinor operators | operator/intertwiner types | partially available in `SpinorRepr`; some are TODO |

## Intertwiner trait refactor plan

Use leg-count semantics based on input arity:
- 2-leg intertwiner = 2 inputs -> 1 output
- 3-leg intertwiner = 3 inputs -> 1 output
- 4-leg intertwiner = 4 inputs -> 1 output

Introduce leg-specific traits for Lorentz intertwiners generated from UFO
Lorentz structures.

```rust

pub trait Intertwiner2Leg<F: Real> {
  type In1: LorentzRepr<F>;
  type In2: LorentzRepr<F>;
  type Momentum: LorentzVector<F>;
  type Out: LorentzRepr<F>;

  fn apply(input: (Self::In1, Self::In2), Momentum) -> Self::Out;
}

pub trait Intertwiner3Leg<F: Real> {
  type In1: LorentzRepr<F>;
  type In2: LorentzRepr<F>;
  type In3: LorentzRepr<F>;
  type Momentum: LorentzVector<F>;
  type Out: LorentzRepr<F>;

  fn apply(input: (Self::In1, Self::In2, Self::In3), Momentum) -> Self::Out;
}

pub trait Intertwiner4Leg<F: Real> {
  type In1: LorentzRepr<F>;
  type In2: LorentzRepr<F>;
  type In3: LorentzRepr<F>;
  type In4: LorentzRepr<F>;
  type Momentum: LorentzVector<F>;
  type Out: LorentzRepr<F>;

  fn apply(input: (Self::In1, Self::In2, Self::In3, Self::In4), Momentum) -> Self::Out;
}
```

Notes:
- For local HELAS/UFO vertex maps, output momentum is derived from input
  wavefunction momenta by routing convention (signed sum of inputs). A separate
  explicit momentum argument is therefore not required in the primary trait
  API.
- Intertwiners consume inputs by value to match tree-style reduction (little or
  no input reuse). This enables move-based chaining once wavefunction types are
  non-`Copy` newtypes.
- Inputs/outputs remain explicitly tied to Lorentz representations via
  `LorentzWf::Repr: LorentzRepr<F>`.
- If a future non-local form factor needs extra kinematic context, add an
  auxiliary `apply_with_ctx` extension trait rather than broadening the base
  signatures.


HELAS review outcome:
- A direct quartic-vector contact amplitude routine exists (`ggggxx`), a direct
  4-vector -> scalar map (4-leg intertwiner here), not a chain of lower-arity
  routine calls.
- A dedicated quartic-current reduction also exists (`jgggxx`), a
  3-vector-input -> 1-vector-output map (3-leg intertwiner here).

So both forms should be modeled:
- Intertwiner3Leg for off-shell current reductions (3 -> 1)
- Intertwiner4Leg for direct quartic contact maps (4 -> 1)

## Recommended medium-term Rust type set

### Value wrappers

- FourMomentum<F>
- ScalarWf<F>
- GhostScalarWf<F>
- SpinorIn<F, B>
- SpinorOut<F, B>
- VectorWf<F>
- VectorSpinorWf<F, B> (spin-3/2, planned)
- Rank2TensorWf<F> (spin-2, planned)

### Representation tags

- LorentzRepr should remain as representation identity/tag.
- SpinorRepr should define spinor-space intertwiners/projections.
- Spinor construction can be inherent methods on SpinorIn/SpinorOut in the
  Weyl-first path, or remain in a small constructor trait if basis-polymorphism
  becomes necessary.

## Phased implementation plan

1. Phase A: replace raw [C<F>; 4] in public APIs
- Migrate intertwiners and vertices to typed wrappers.
- Keep internal kernels operating on arrays for now.

2. Phase B: lock spin-1/2 and spin-1 semantics
- Enforce In/Out for fermions.
- Keep one `VectorWf` type and enforce consistent HELAS contraction conventions.

3. Phase C: add operator typing for UFO Lorentz AST
- Parse structure strings into typed AST nodes for Gamma, Metric, P, etc.
- Type-check operator input/output spaces before code generation.

4. Phase D: add higher-spin families
- Introduce VectorSpinorWf and Rank2TensorWf once needed by a target UFO model.

## Design constraints

- Preserve zero-cost wrappers where possible.
- Prefer Copy newtypes for small fixed-size carriers.
- Keep numeric kernels straightforward and benchmarkable.
- Avoid introducing basis polymorphism beyond Weyl until a concrete use case
  appears.

## Open questions

- Should GhostScalarWf be a distinct runtime type or a scalar with a statistics
  tag used only in combinatorics/sign bookkeeping?
- When should explicit variance newtypes (`VectorWf`/`CovectorWf`) be
  introduced without adding unnecessary complexity?
- Do we encode off-shell status in the type system or in constructor APIs?
