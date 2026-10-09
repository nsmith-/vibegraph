---
type: Physics Convention
title: "How MadGraph factorizes colour: colorize, basis, JAMPs, CF matrix"
description: "MadGraph splits M into JAMPs times an exact-rational colour matrix from symbolic SU(3) simplification; runtime never sees a colour index."
status: draft
tags: [colour, madgraph, jamp, colour-matrix, su3]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n16-ncolor6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/16-color-flow-design.md#L37-L99", title: "Note 16, the NCOLOR=6 JAMP caveat resolved"}
  - {id: n16-mg, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/16-color-flow-design.md#L100-L228", title: "Note 16 §1 (how MadGraph factorizes colour from Lorentz)"}
  - {id: guide-colour, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/docs/src/guide/05-color.md", title: "User guide, Colour chapter"}
  - {id: mg-colorize, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/core/color_amp.py#L62", title: "MadGraph color_amp.py, ColorBasis.colorize"}
  - {id: mg-matrix, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/core/color_amp.py#L537", title: "MadGraph color_amp.py, ColorMatrix"}
  - {id: mg-algebra, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/core/color_algebra.py#L304-L333", title: "MadGraph color_algebra.py, f → traces"}
  - {id: mg-jamp, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/core/helas_objects.py#L4978", title: "MadGraph helas_objects.py, get_color_amplitudes"}
  - {id: mg-treat, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/models/import_ufo.py#L1946", title: "MadGraph import_ufo.py, treat_color"}
  - {id: mg-cfdata, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/iolibs/export_v4.py#L1250-L1287", title: "MadGraph export_v4.py, get_color_data_lines"}
  - {id: mg-template, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/iolibs/template_files/matrix_madevent_group_v4.inc#L360-L374", title: "MadGraph matrix_madevent_group_v4.inc, the colour-sum loop"}
  - {id: code-oracle, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/tests/amplitude_oracle.rs", title: "The amplitude oracle (per-flow JAMPs)"}
  - {id: code-cforacle, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/tests/color_cf_oracle.rs", title: "The CF-matrix oracle"}
---

# How MadGraph factorizes colour: colorize, basis, JAMPs, CF matrix

MadGraph never evaluates colour numerically per phase-space point. It factors
the amplitude symbolically, once per process[^n16-mg]:

```
M = Σ_f C_f · J_f(p, h)            Σ_colour |M|² = Σ_{f,f'} J_f · CF_{ff'} · J_{f'}*
```

The `J_f` (MadGraph's JAMPs) are kinematic partial amplitudes, linear
combinations of HELAS diagram amplitudes with rational coefficients. `CF` is a
constant matrix of exact rationals from symbolic SU(3) algebra. The
floating-point runtime never sees a colour index. Vibegraph follows the same
scheme exactly; the user-level account, with the rule table and the
`helas::color` module map, is the guide's Colour chapter[^guide-colour]. This
concept records how MadGraph does it, which is what our oracles compare
against.

The factorisation rests on the UFO vertex format: a vertex is
`Σ_{c,l} couplings[(c,l)] · color[c] ⊗ lorentz[l]`, so each diagram splits into
a choice of colour structure per vertex times a Lorentz/coupling chain.

## 1. `colorize`: one colour string per (diagram, colour-index chain)

`ColorBasis.colorize` walks a diagram's vertices in wavefunction-chain order
and builds products of colour atoms[^mg-colorize]:

- The vertex's colour-string indices `1..n` are positions in the UFO
  interaction's particle list. They are replaced by external leg numbers, or by
  fresh negative summed indices for internal lines. The propagator's δ is
  implicit in the repeated index.
- The **output leg of a non-final vertex is conjugated**
  (`get_anti_color()`): the far end of the propagator sees the antiparticle.

  ```python
  if index == len(vertex.get('legs')) - 1 and \
      vertex != diagram.get('vertices')[-1]:
      curr_color = curr_part.get_anti_color()
  ```
- A vertex with `k` colour structures multiplies the accumulated dictionary
  out `k` ways. The dictionary key is the tuple of chosen structure indices, one
  per vertex: the **colour-index chain**. A colourless vertex appends index 0
  and no atom. In the SM only the four-gluon vertex has `k > 1` (`k = 3`).
- A fully colourless diagram gets `ColorOne()`.

Each HELAS amplitude carries its chain as `color_indices`
(`helas_objects.get_color_indices`), so amplitudes and colour strings pair
exactly.

## 2. Simplification to a canonical basis, in exact rationals

`ColorBasis.update_color_basis` runs `ColorFactor.full_simplify()` on each
string. The rewrite system in `color_algebra.py` works on `T(a1..an, i, j)`
(open fundamental chain), `Tr(a1..an)`, `f` and `d`, with coefficients
`Fraction × (i if is_imaginary) × Nc^power`[^n16-mg]:

| rule | identity |
|---|---|
| f → traces | `f(a,b,c) = −2i Tr(a,b,c) + 2i Tr(c,b,a)` |
| chain merge | `T(A,i,j) T(B,j,k) = T(A+B,i,k)` |
| chain closes | `T(A,i,i) = Tr(A)` |
| Fierz within T | `T(a,X,b,X,c,i,j) = ½ T(a,c,i,j) Tr(b) − ½Nc⁻¹ T(a,b,c,i,j)` |
| Fierz T·T | `T(a,X,b,i,j) T(c,X,d,k,l) = ½ T(a,d,i,l) T(c,b,k,j) − ½Nc⁻¹ T(a,b,i,j) T(c,d,k,l)` |
| Fierz Tr·Tr, Tr·T | the same pattern (`Tr.pair_simplify`) |
| trace values | `Tr() = Nc`, `Tr(a) = 0` |
| conjugation | `T(A,i,j)* = T(reverse A, j, i)` |

The `f` rule as MadGraph writes it[^mg-algebra]:

```python
col_str1 = ColorString([Tr(*indices)]); indices.reverse()
col_str2 = ColorString([Tr(*indices)])
col_str1.coeff = fractions.Fraction(-2, 1)
col_str2.coeff = fractions.Fraction(2, 1)
col_str1.is_imaginary = True; col_str2.is_imaginary = True
```

`full_simplify` iterates single-object `simplify` and two-object
`pair_simplify` to a fixed point, cached by canonical form. Each distinct
canonical string that survives is one basis element; the basis records, per
contributing (diagram, chain), the exact coefficient
`(Fraction, is_imaginary, Nc_power)`.

This is the trace/δ basis that falls out of simplification, not the
leading-colour flow decomposition. MadGraph computes the latter separately
(`color_flow_decomposition`) and uses it only for the LHEF colour tags; see
[colour-flow-lines-and-conjugation](../amplitudes/colour-flow-lines-and-conjugation.md).

## 3. JAMPs and the colour matrix

`get_color_amplitudes` emits, per basis element `f`, a list of
`(fermion factor × coefficient × i^im × Nc^power, AMP number)` pairs. The
generated Fortran is literally `JAMP(1,1) = (-1d0)*AMP(1) + (-1d0)*AMP(2) + …`,
with the rationals printed as floating-point numbers[^mg-jamp].

`ColorMatrix` computes `CF_{ff'} = ⟨basis_f | basis_f'⟩` by multiplying string
`f` with the complex conjugate of string `f'` (summed indices of `f'` relabelled
first), simplifying, and setting `Nc = 3`. Everything stays a `Fraction` until
code generation[^mg-matrix].

At the pinned MadGraph (3.7.1, `b7687064`), code generation writes the matrix
as **integers over one common denominator, upper triangle only, off-diagonal
entries doubled**[^mg-cfdata]:

```fortran
      DATA Denom/6/
      DATA (CF(i),i=  1,  6) /19,-4,-4,-4,-4,8/
```

and the squaring loop runs over `J ≥ I` and divides once at the end[^mg-template]:

```fortran
      DO I = 1, NCOLOR
        ZTEMP = (0.D0,0.D0)
        DO J = I, NCOLOR
          CF_INDEX = CF_INDEX + 1
          ZTEMP = ZTEMP + CF(CF_INDEX)*JAMP(J,M)
        ...  MATRIX = MATRIX + ZTEMP*DCONJG(JAMP(I,N))
      MATRIX = MATRIX/DENOM
```

`Denom` is the **maximum** of the per-row denominators, not their LCM; the
exporter asserts that every numerator is then an integer. Older exporters wrote
the full square `DATA (CF(I,J),…)` form with per-row denominators.
`tests/color_cf_oracle.rs` parses both forms (`parse_packed_cf` for the
packed one)[^code-cforacle].

`MATRIX1` is therefore colour-summed (and its caller sums helicities), so every
MadGraph `|M|²` reference already contains the full CF contraction. Vibegraph
computes the same CF itself (`AmplitudeEvaluator::cf_matrix`, exact
`Ratio<i64>`); no external colour factor is multiplied in.

## 4. SM tree-level colour vocabulary

From the SM UFO's `vertices.py`[^n16-mg]:

```
'1'                       ×94   colourless
'Identity(1,2)'           ×50   q q̄ neutral-boson vertices: δ_ij
'T(3,2,1)'                ×6    q q̄ g: (T^{a3})_{i2 j̄1}
'f(1,2,3)'                ×2    g g g
'f(-1,1,2)*f(3,4,-1)' + 2 perms   g g g g: three structures on one vertex
```

Index convention (UFO paper, `color_algebra.T`): `T(a, …, i, j)` puts the
adjoint indices first, then the fundamental (3) index `i`, then the
antifundamental (3̄) index `j`.

`Identity(m, n)` depends on the representations at slots m and n, so MadGraph
resolves it at UFO import (`import_ufo.treat_color`)[^mg-treat]:

- a 3/3̄ pair becomes `T(i, j)` with the fundamental first (δ_{i j̄});
- an 8/8 pair becomes `Tr(m, n)` with an extra **factor 2**
  (`Tr[T^aT^b] = δ^{ab}/2`);
- sextets become `T6`.

The resolution needs particle colour representations, so it belongs in the
UFO load, not in the algebra engine.

## How vibegraph's JAMPs relate to MadGraph's

Vibegraph's per-flow JAMPs equal MadGraph's element-wise under the identity
flow pairing, at every helicity and every point, up to one constant per
process[^n16-ncolor6]:

```
J_f^vg(p, h) = G · JAMP_f^mg(p, h),     G = ±i
```

`G` is one constant per process, not per flow and not per point. It is the
same `i` placement seen per diagram: vibegraph roots each diagram with a
factor of `i` that MadGraph's `AMP()` leaves out. The amplitude oracle
(`tests/amplitude_oracle.rs`) fits a single `G` over every diagram and every
flow, and asserts `|G| = 1` and `Re G = 0`. One constant for both levels is
what ties the diagram convention to the colour convention: vibegraph puts the
annihilation/exchange sign in the diagram root and keeps colour coefficients
of +1, while MadGraph puts it in the colour coefficient `c_i`, and only the
product is observable[^code-oracle]. The phase ledger is
[global-phase-i-counting](../amplitudes/global-phase-i-counting.md).

Two facts about `g g > g g` (NCOLOR = 6) that bear on any matcher:

- **The tree-level `[flow × helicity]` JAMP matrix is rank 1.** The amplitude is
  MHV, so by Parke–Taylor every colour-ordered partial amplitude has the same
  helicity dependence `⟨ij⟩⁴` and differs only in its ordering-dependent
  denominator. Every pair of rows has overlap 1, and a greedy max-overlap
  matcher picks a pairing arbitrarily. `gg_to_ttx` is rank 1 as well. Pair flows
  by identity, never by overlap.
- **Trace-reversal partners carry identical JAMPs** (`J₁ = J₆`, `J₂ = J₄`,
  `J₃ = J₅`, visible in MadGraph's own
  `JAMP(1,1) = JAMP(6,1) = 2·(AMP(3)+AMP(6)−AMP(1)−AMP(4))`). Swapping such a
  pair is invisible to JAMP values, to `JAMP2` and to |M|². The colour-tag
  oracle sees it, because it compares each flow's colour-line endpoints with
  `leshouche.inc`.

`|M|²` cannot see any of this. The `gg_to_gg` CF is `(7/2)I + P − (1/3)J`, with
`P` the trace-reversal involution `(1,6)(2,4)(3,5)` and `J` the all-ones
matrix. Its eigenvalues are 5/2 (×4) and 9/2 (×2), so it is positive definite
and no flow combination is invisible to the contraction. It does have a large
automorphism group, and any permutation in it leaves |M|² exact while
permuting `JAMP2`, the colour-flow selection weight. `JAMP2_i = Σ_h |J_i|²` is
unaffected by `G` because `|G| = 1`. Which oracle closes which of these blind
spots is [validation/colour-oracles](../validation/colour-oracles.md); the
evaluator side is [colour-flow-evaluator](../amplitudes/colour-flow-evaluator.md).

[^n16-mg]: Note 16 §1a–§1d. Its §1c says current MadGraph emits `REAL*8 CF` with the denominator divided through; at the pinned 3.7.1 the generated code uses packed integer numerators over `DENOM`, as above. Its "stand-in `color_factor(name)`" no longer exists.
[^n16-ncolor6]: Note 16, "The NCOLOR=6 JAMP caveat, resolved": the earlier "same space, not 1:1" reading was a greedy-matcher artifact. The oracle it names, `color_jamp_oracle`, has been folded into `tests/amplitude_oracle.rs`.
[^guide-colour]: `docs/src/guide/05-color.md`.
[^mg-colorize]: `ColorBasis.colorize`, `madgraph/core/color_amp.py` line 62; the output-leg conjugation at line 120.
[^mg-matrix]: `ColorMatrix.build_matrix`, `madgraph/core/color_amp.py` line 570.
[^mg-algebra]: `f.simplify`, `madgraph/core/color_algebra.py` lines 318–333.
[^mg-jamp]: `HelasMatrixElement.get_color_amplitudes`, `madgraph/core/helas_objects.py` line 4978.
[^mg-treat]: `import_ufo.treat_color`, `models/import_ufo.py` line 1946 (`factor *= 2` on the 8/8 branch).
[^mg-cfdata]: `get_color_data_lines`, `madgraph/iolibs/export_v4.py` lines 1250–1287: `denominator = max(…get_line_denominators())`, and `(1 if (k==index and pos==0) else 2)*int(i)`.
[^mg-template]: `matrix_madevent_group_v4.inc` lines 360–374; `matrix_standalone_v4.inc` has the same loop with `REAL(ZTEMP*DCONJG(JAMP(I)))`.
[^code-oracle]: `vibegraph-lib/tests/amplitude_oracle.rs` module doc.
[^code-cforacle]: `vibegraph-lib/tests/color_cf_oracle.rs`, `parse_packed_cf`.
