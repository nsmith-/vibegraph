---
type: Design
title: Multi-flow colour in the evaluator
description: "Exact Ratio<i64> colour coefficients through colorize, basis and CF matrix; an Op::Flows root and Op::CoeffRat leaves; floats only at fold and bind; the 3/3̄ crossing undone per tensor."
status: draft
tags: [colour, evaluator, exact-arithmetic, cf-matrix, jamp]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n16-21, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/16-color-flow-design.md#L231-L263", title: "Note 16 §2.1: shape of the change"}
  - {id: n16-22, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/16-color-flow-design.md#L264-L299", title: "Note 16 §2.2: repr/color.rs as a working algebra"}
  - {id: n16-24, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/16-color-flow-design.md#L318-L352", title: "Note 16 §2.4: colorize over diagrams::Diagram"}
  - {id: n16-25, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/16-color-flow-design.md#L353-L412", title: "Note 16 §2.5: evaluator integration"}
  - {id: n16-26, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/16-color-flow-design.md#L413-L422", title: "Note 16 §2.6: what we deliberately do not do"}
  - {id: code-colorize, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/color/colorize.rs", title: "helas/color/colorize.rs: slot indices, convert_expr, ColorBasis"}
  - {id: code-coeff, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/color/coeff.rs", title: "helas/color/coeff.rs: ColorCoeff"}
  - {id: code-run, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/run.rs#L330-L400", title: "run.rs: eval_m2's CF contraction"}
---

# Multi-flow colour in the evaluator

The evaluator reproduces MadGraph's colour factorization (see
[madgraph-colour-factorization](madgraph-colour-factorization.md)): per subprocess
a basis of colour flows, one JAMP per flow, and an exact colour-factor matrix
`CF`; `|M|² = Σ_hel Σ_ij CF_ij J_i J_j*`. Colour is never a numeric vector at run
time.

## Where colour enters the pipeline

```
compile (per subprocess):
  pass 1+2   root diagrams, one tree per colour-index chain   root_diagram.rs
  pass C     colorize each (diagram, chain) → colour string   helas/color/colorize.rs
             full_simplify → basis keys + exact coefficients
             CF matrix, exact, Nc = 3
  pass 3a    lower: one AST, NCOLOR JAMPs under Op::Flows     lower.rs (lower_flows)
  pass 3b    fold: rationals → F pools   ← first float        fold.rs
bind:        CF matrix → Box<[F]>                             run.rs
eval:        per helicity: J_f per flow; Σ CF J J*            run.rs
```

**Exact all the way to fold.** Coefficients are `ColorCoeff { q: Ratio<i64>,
imag: bool, nc_power: i32 }` — `q · i^imag · Nc^nc_power`, MadGraph's own
factorization. Two coefficients add only when `imag` and `nc_power` agree;
multiplication combines all three. Every operation is checked and panics on
`i64` overflow: tree-level SU(3) factors are tiny, so an overflow is a bug, and
`Ratio<i128>` behind the `GroupScalar` trait is the escape hatch.[^code-coeff]
Rationals become `F` only in `fold.rs` (an `Op::CoeffRat` leaf resolves to a real
or complex pool entry `num/den`, times `i` when `imag`) and in
`BoundAmplitude::bind` (the CF matrix).[^n16-21]

**The algebra** (`helas/color/`, compile-time only, independent of `F`) is a port
of MadGraph's `color_algebra.py`: `ColorTensor` atoms `T(a…, i, j)`, `Tr(a…)`,
`f`, `d`, `Epsilon`/`EpsilonBar`, `K6`/`K6Bar`, `T6`; colour strings and factors;
`simplify`/`pair_simplify`/`full_simplify` to a fixpoint; an immutable sorted form
as basis key. Indices are `i32`, external legs positive, summed indices negative
(MadGraph's convention). There are no numeric `f^{abc}` contractions and no
leading-colour approximation.[^n16-22] The `ColorRepr` marker types' Casimir and
Dynkin constants are oracles for the engine's unit tests
(`T(a,i,j)T(a,j,k) → C_F δ_ik`).

## Colorize

`diagrams::Diagram` already records each vertex's rays in UFO interaction-slot
order, which is the order colour-string indices refer to:[^n16-24]

- a ray to an external leg gets that leg's number;
- a ray to a propagator gets one fresh summed index, shared by its two end
  vertices;
- a vertex with `k` colour structures expands the diagram into one colour-index
  chain per used structure (only the structures that appear in the vertex's
  `couplings` keys). Each chain is rooted as its own tree, with the vertex's
  `VertexInfo` restricted to that structure's couplings; the trees share
  everything except the affected vertex, and hash-consing collapses the shared
  part.

**The crossing.** feyngraph presents every leg in the all-incoming crossing, the
opposite arrow from MadGraph's `T(a…, i, j)` convention (`i` the arrow-out leg).
Read straight through, every `T` comes out transposed: invisible for
purely-rational T-chain terms, but it complex-conjugates the colour string and so
flips the sign of the imaginary `f → trace` coefficients relative to the T-chain
ones (first seen on `g g > t t~`, which mixes both). `convert_expr` transposes
each `T`'s index pair on substitution — per tensor, not by swapping slots — and
`check_slot_reps` rejects a vertex whose atoms do not stand on the reps the
transpose assumes. The baryonic and sextet atoms cross the same way. Details,
and why a positional slot swap was wrong, in
[colour-crossing-epsilon-and-sextets](colour-crossing-epsilon-and-sextets.md).

**The basis.** Per subprocess, each chain's string is fully simplified and
accumulated into a basis keyed by the sorted immutable form. Basis elements are
in sorted-key order, MadGraph's `sorted(ColorBasis.keys())` JAMP order, which is
what makes JAMP-level comparison against MadGraph's Fortran possible. The CF matrix
is `CF_ff' = eval_nc(⟨f, f'*⟩)` at `Nc = 3`, stored as `Vec<Ratio<i64>>`. A
colourless process has one `ColorOne` flow and `CF = [[1]]`; quark-line QCD=0
processes get their `CF(1,1)` of 3 or 9 computed, not hard-coded.

## Lowering and evaluation

Per flow `f`, `JAMP_f = Σ_{(d,chain) ∈ f} colourcoeff · sym · fermi_sign ·
amp_{d,chain}`, all JAMPs under one variadic root `(Flows jamp_0 … jamp_{n−1})`
(at `NCOLOR = 1` the unit coefficient and the wrapper are omitted and the single
JAMP is the root). The AST stays single-rooted, and an amplitude shared by several JAMPs with
different weights (both flows of `u u~ > u u~`) is computed once.[^n16-25] Two ops
carry this: `Op::Flows` (variadic root, no leaf) and `Op::CoeffRat` (exact scalar
leaf `Sym::Rational { num, den, imag }`, `Nc` already evaluated).

`eval_m2` contracts per helicity in MadGraph's ZTEMP order,
`Σ_i (Σ_j CF_ji J_j) · J_i*`. For `NCOLOR = 1` it multiplies `CF(1,1)` *after* the
helicity sum, `CF · Σ_hel |M|²`, because `Σ (CF·x_h) ≠ CF·Σ x_h` bitwise; this keeps
single-flow processes bit-identical to a colour-free evaluation.[^code-run]
`eval_jamp2` reads the same slots for the per-flow `Σ_hel |J_f|²` that colour
selection draws from; `Σ_f JAMP2_f ≠ |M|²` on any non-orthogonal basis, which its
test asserts so a later "simplification" cannot return the contraction.

## Deliberately not done

- No numeric colour wavefunctions or per-point colour sums.
- No leading-`Nc` colour sampling (a Sherpa-style colour integrator): CF is exact
  and small at these multiplicities.
- No colour or helicity averaging inside `eval_m2`: `1/Nc²`, `1/(Nc²−1)²` and spin
  averages belong to the cross-section layer, as in the `MATRIX1` comparison.[^n16-26]
- Leading-colour flow decompositions return only for LHEF colour tags, which are
  read off the basis keys ([colour-flow-lines-and-conjugation](colour-flow-lines-and-conjugation.md)).

The colour grammar the UFO strings are parsed with is in
[model/ufo-string-grammars](../model/ufo-string-grammars.md); the IR it lowers
into is [flat-op-ir](flat-op-ir.md).

[^n16-21]: Note 16 §2.1.
[^n16-22]: Note 16 §2.2.
[^n16-24]: Note 16 §2.4.
[^n16-25]: Note 16 §2.5.
[^n16-26]: Note 16 §2.6. That section also lists `d`, sextet and ε structures as out of scope; they are now supported.
[^code-coeff]: `vibegraph-lib/src/helas/color/coeff.rs`.
[^code-run]: `vibegraph-lib/src/helas/eval/run.rs`, `eval_m2`.
