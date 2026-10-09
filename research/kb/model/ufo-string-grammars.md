---
type: Physics Convention
title: UFO expression, Lorentz and colour string grammars
description: "Python-precedence expression grammar; the Lorentz-structure grammar (Gamma, Sigma, ProjM/P, Epsilon, P, X**n expansion); colour atoms, T index order and rep-dependent Identity resolution."
status: draft
tags: [ufo, grammar, peg, colour, lorentz]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n16-vocab, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/16-color-flow-design.md#L208-L228", title: "Note 16 §1d, SM tree-level colour vocabulary"}
  - {id: n16-parser, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/16-color-flow-design.md#L300-L317", title: "Note 16 §2.3, UFO parser changes for colour"}
  - {id: n35-l1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L604-L675", title: "Note 35 §4 L1, Gamma5, ** powers, colour strings under restriction"}
  - {id: n36-b5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36-banked-open-ends-plan.md#L413-L457", title: "Note 36 B5, the coupling-level oracle and the precedence defect it found"}
  - {id: n36-b7, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36-banked-open-ends-plan.md#L525-L564", title: "Note 36 B7, UFO expression precedence fixed"}
  - {id: mg-treat-color, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/models/import_ufo.py#L1946", title: "MadGraph import_ufo.py treat_color"}
  - {id: python-grammar, resource: "https://docs.python.org/3/reference/expressions.html#the-power-operator", title: "Python reference, the power operator"}
measured:
  - {commit: f3425e2, pr: 6, landed_in: 02e8b25, command: "cargo test -p vibegraph-lib --test coupling_oracle"}
---

A UFO model writes three kinds of string that are not Python statements but are parsed as
expressions of their own: parameter and coupling **values**, Lorentz **structures**, and
vertex **colour** factors. Each has a small PEG grammar in `vibegraph-lib/src/ufo/`; the
`.py` files around them are read with a Python parser
([UFO parsing](ufo-parsing.md)).

**All three feed the interned SM blob.** The blob (`ufo/sm_assets/sm_parsed.bin.zst`)
stores the parsed ASTs, so any edit to one of these grammars or to the types they produce
must regenerate it with the `gen_sm_blob` dev binary in the same change. The check is
`pixi run check-sm-blob-fresh`, which needs `extended-validation`; the hermetic suite
does not see a stale blob. [^n36-b7]

## Value expressions (`ufo/expr.rs`)

Values are Python expressions, and the grammar follows Python's precedence exactly:
[^python-grammar]

```text
expression     = additive
additive       = multiplicative (("+" / "-") multiplicative)*
multiplicative = unary (("*" / "/") unary)*
unary          = "-" unary / "+" unary / power
power          = primary "**" unary / primary
primary        = "(" additive ")" / number / complex(re, im) / cmath.f(x) / f(x) / cmath.pi / name
```

- A sign binds **looser** than `**` on its left: `-a**2` is `-(a²)`, as in Python's
  `factor: ('+'|'-') factor | power`.
- The exponent is itself a signed factor, so `a**-b` parses, and `**` is
  right-associative: `2**3**2` is 512.
- `complex(re, im)` takes the real part of each argument.
- Functions: `cmath.` `sqrt`, `log`, `exp`, `sin`, `cos`, `tan`, `atan`, `asin`, `acos`;
  bare `complexconjugate`/`conj`, `sqrt`, `abs`, `arg`, `re`, `im`, `sin`, `cos`, `tan`,
  `atan`, `asin`, `acos`, `sec`, `csc`, `asec`, `acsc`. These stand in for `function_library.py`, which
  is not read. Any other function name (`cot`, which SMEFTsim's library defines) is a
  parse error.
- Evaluation is in `Complex64`. An unknown parameter reference panics in every build
  (the doc comment on `eval` still says release builds return 0; the code does not).

Pins: unit tests on `-a**2`, `(-a)**2`, `a**-b`, `-a**-b`, `2**3**2` and on three model
expressions against Python's own arithmetic; and `coupling_oracle`
(`vibegraph-lib/tests/coupling_oracle.rs`), which compares every coupling of every banked
row with MadGraph's Python `model_reader` (`PYTHON_REL_TOL` 1e-13) and fails if a listed
crate defect disappears (`KNOWN_CRATE_DEFECTS`, empty). It is the check that
sees a precedence error: binding unary minus tighter than `**` reads `-ee**2/(2.*cw)` as
`(-ee)**2/…`, which flips SM `GC_7`, `GC_54` and SMEFTsim's `dWT`, none of them reachable
by a banked row, so no amplitude gate would. [^n36-b5] See [coupling oracle](../validation/coupling-oracle.md).

**Powers.** `pow` keeps a negative real base off the complex branch: an integral
exponent uses real `powf` (so `(-x)**2` has no spurious imaginary part), and a
non-integral one takes Python's principal branch, rebuilding the base so the `−0.0`
imaginary part that negation leaves cannot select the conjugate branch. A non-negative
real base still goes through `powc`, `exp(e·log b)`: `3**2` evaluates to
`9.000000000000002`. Switching that path to `powf` moves 335 of 3254 model values by
about one ulp; it is measured and not adopted, since it needs a before/after oracle of
its own. [^n36-b7]

**Known latent defect.** `asin` and `acos` (bare or `cmath.`) map to the `ACsc`/`ASec`
functions, which evaluate `asin(1/x)` and `acos(1/x)` (`expr.rs`, the
`cmath_func_name` and `bare_func_name` rules). No value string in any model the
repository loads calls them; SMEFTsim's library defines `asec`/`acsc` in terms of them,
but the grammar implements `asec`/`acsc` directly.

## Lorentz structures (`ufo/lorentz.rs`)

```text
structure = signed_product (("+" / "-") product)*
product   = factor (("*" / "/") factor)*
factor    = atom ("**" integer)?
atom      = number / Operator(arg, …) / "(" structure ")"
```

- **Operators** are dispatched by name and arity into `LorentzOp`: `Gamma(μ,i,j)`,
  `Gamma5(i,j)`, `Sigma(μ,ν,i,j)`, `Identity(i,j)`, `ProjM(i,j)`, `ProjP(i,j)`,
  `Metric(μ,ν)`, `P(μ,leg)`, `Epsilon(μ,ν,ρ,σ)`, `C(i,j)`. Any other name is
  `LorentzError::UnknownOperator`; arguments that are not plain indices are parsed only
  far enough to report the name.
- **Indices.** A positive index is a 1-based leg number, converted to 0-based at import;
  a negative index is summed.
- **Powers.** `X**n` expands into `n` copies of `X` multiplied, so Einstein summation
  over the repeated dummy index does the contraction (`P(-1,2)**2` = `p₂·p₂`). A power
  above 2 on an indexed object has no Einstein reading and is rejected; numeric powers
  are unrestricted. SMEFTsim uses only `**2`.
- **Coefficients** are real `f64`; parenthesised sums distribute (`2*(A + B)` → `2A + 2B`).
  Numbers are `digits[.digits]`, with no exponent notation.

What each operator means numerically (ALOHA's half-size `Sigma`, `Gamma5 = ProjP −
ProjM`, the `Epsilon` sign) is in
[gamma chains, Gamma5 and Epsilon](../amplitudes/gamma-chains-gamma5-and-epsilon.md) and
[UFO spin codes](ufo-aloha-type-matrix.md).

## Colour strings (`ufo/color.rs`)

A colour string is a `*`-product of atoms with integer indices: a **positive** index is
a 1-based slot in the vertex's particle list, a **negative** one is summed within the
string (the SM four-gluon vertex writes `f(-1,1,2)*f(3,4,-1)`). [^n16-parser]

| Atom | Meaning |
|---|---|
| `1` | colourless |
| `T(a…,i,j)` | generator chain: adjoint indices first, then the **fundamental (3) index `i`**, then the **antifundamental (3̄) index `j`** |
| `f(a,b,c)`, `d(a,b,c)` | antisymmetric and symmetric structure constants |
| `Epsilon(i,j,k)`, `EpsilonBar(i,j,k)` | baryonic invariants of three 3 or three 3̄ indices |
| `K6(m,i,j)`, `K6Bar(m,i,j)` | sextet Clebsch–Gordan, sextet index first |
| `T6(a…,i,j)` | sextet generator chain |
| `Identity(m,n)` | resolved at load, below |

The grammar admits no whitespace inside an atom. UFO colour strings carry no numeric
literals, so the colour layer is exact; rationals first appear in the algebra engine.

**`Identity` is representation-dependent** and is resolved at load from the colour
representations of the particles in slots `m` and `n`, as MadGraph's
`import_ufo.treat_color` does [^mg-treat-color] [^n16-vocab]:

| Slots | Becomes |
|---|---|
| 3 / 3̄ | `T(i,j)` with the **fundamental slot first**, i.e. δ_{i j̄} |
| 6 / 6̄ | `T6(i,j)` with the sextet slot first |
| 8 / 8 | **`2·Tr(m,n)`**: the factor 2 because Tr[TᵃTᵇ] = δᵃᵇ/2 |
| anything else | `ColorError::IdentityRepMismatch` |

This resolution needs the particles' representations, which is why it happens in the UFO
load and not in the algebra engine. It is a convention trap: a slot-order or factor-2
error is invisible to any oracle that normalises colour columns. An unknown atom is a
parse error naming the string.

**Colour strings are not pruned under restriction**: MadGraph prunes only Lorentz
structures, and every consumer reaches a colour structure through the coupling keys
([restrict-card semantics](restriction-semantics.md)). [^n35-l1]

**The SM vocabulary** (from `models/sm/vertices.py`, as note 16 counted it): `'1'` ×94,
`Identity(1,2)` ×50 on the quark-antiquark neutral-boson vertices, `T(3,2,1)` ×6 on
`q q̄ g`, `f(1,2,3)` ×2 on `g g g`, and the four-gluon vertex's three `f*f` structures on
one vertex. [^n16-vocab]

How colour strings become flows and the CF matrix is in
[the colour-flow evaluator](../amplitudes/colour-flow-evaluator.md); which epsilon and
sextet configurations stay refused is in
[colour crossing, epsilon and sextets](../amplitudes/colour-crossing-epsilon-and-sextets.md).

A related defect is MadGraph's, not ours: for a fermion built from a vertex's second
spinor slot, ALOHA flips momentum signs with a regex substitution (`P(` → `-P(`), which
turns the `P(-1,id)**2` inside the `$`-veto theta function into `-(p²)`; only
`FFV2P1D_1` changes. That defect and its consequences are in
[MadGraph defects](../validation/madgraph-defects.md).

[^n16-vocab]: Note 16 §1d: the SM colour vocabulary, the `T` index convention and `treat_color`'s `Identity` rules.
[^n16-parser]: Note 16 §2.3: the colour-string grammar, signed indices, unknown atoms as hard errors, exactness.
[^n35-l1]: Note 35 §4, the loader and model-topology surface: `Gamma5`, `**` expansion with `n > 2` rejected on indexed objects, colour strings unpruned.
[^n36-b5]: Note 36 §4, the coupling-level oracle: the coupling oracle and the precedence defect's reach.
[^n36-b7]: Note 36 §4, UFO expression precedence: Python's grammar adopted, the `−0.0` branch fix, the unadopted `powf` change, blob regeneration.
[^mg-treat-color]: `models/import_ufo.py` `treat_color`, L1946.
[^python-grammar]: Python's `power` and `factor` grammar.
