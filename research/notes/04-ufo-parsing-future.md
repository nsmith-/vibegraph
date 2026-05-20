# Future: Full UFO Parsing (Take Over from FeynGraph)

**Status:** Stub / future work

## Current situation

`UfoModel::load()` delegates UFO topology parsing (particle spin/color/PDG,
vertex particle lists, Lorentz structure names) to FeynGraph's
`feyngraph::parse_ufo_model()`. Our own PEG parsers then layer on top to
recover what FeynGraph drops (mass/width param refs, coupling value expressions,
parameter evaluation).

This split works for LO tree-level models (SM, MSSM, taudecay). It breaks on
loop-level models:

- **`loop_sm`** adds `.counterterm` and `.loop_particles` attribute assignments
  to `particles.py` after each `Particle(...)` block. FeynGraph's parser does
  not recognize the `x.attr = ...` syntax and fails with a parse error at column
  2 of such lines.

## What full ownership would buy us

- Support for loop-level UFOs (`loop_sm`, any NLO model)
- No dependency on FeynGraph's Python-specific quirks or update lag
- Ability to extend to `CT_vertices.py`, `CT_parameters.py`, `CT_couplings.py`
  (the counterterm files used in loop UFOs)
- Cleaner architecture: one parsing layer instead of two

## Approach options

### Option A: Extend our PEG parsers (current approach, incremental)

Add `x.attr = ...` skip rules to `particles_ext.rs`'s `skip_nonparticle()` and
to FeynGraph's upstream `particles` grammar (would require forking or patching).

**Pro:** Minimal change, consistent with existing code.  
**Con:** Still blocked by FeynGraph's topology parser; requires either forking
FeynGraph or submitting upstream PRs. The topology structs (`Model`, `Particle`,
`Vertex`) are FeynGraph types — we'd be locked into their data model.

### Option B: Python AST walker via `rustpython-parser` or similar

Instead of hand-rolled PEG grammars, parse each `.py` file to a proper Python
AST and walk the assignment statements.

**Pro:** Handles all valid Python syntax (raw strings, escaped quotes, attribute
assignments, list comprehensions in parameter expressions) without grammar
maintenance. More robust to UFO file variations.  
**Con:** Adds a heavier dependency; `rustpython-parser` supports Python 3 syntax
(UFO files are Python 2/3 compatible, so this is fine). Need to replace
FeynGraph's topology structs too.

Candidates:
- [`rustpython-parser`](https://crates.io/crates/rustpython-parser) — full
  Python 3 AST, actively maintained as part of RustPython
- [`ruff_python_parser`](https://crates.io/crates/ruff_python_parser) — used
  internally by Ruff; fast, well-maintained, Python 3.13+

### Option C: Keep PEG for values, replace FeynGraph topology parser

Write our own PEG grammar for the structural parts (`particles.py`,
`vertices.py`) that currently go through FeynGraph, but keep the expression
grammar in `expr.rs` as-is.

**Pro:** No new heavy dep; we control the full parse.  
**Con:** More PEG grammar to maintain; attribute assignment skipping still needs
care.

## Recommendation (when the time comes)

If expanding to NLO / loop-level UFOs, **Option B** (Python AST walker) is
probably worth the dependency weight — it removes an entire class of parser
fragility. The expression evaluator (`expr.rs`) can stay as-is since we control
exactly which expression forms appear in `value = '...'` strings.

If staying LO-only and just fixing `loop_sm`, **Option A + Option C** (extend
PEG, drop FeynGraph topology dep) is lower risk.

## Known FeynGraph limitations discovered during implementation

| File | Issue | Impact |
|------|-------|--------|
| `particles.py` | Does not parse `x.attr = ...` attribute assignments | `loop_sm` fails to load |
| `particles.py` | Drops `mass` and `width` fields | Workaround: `particles_ext.rs` |
| `couplings.py` | Drops `value` expression string | Workaround: `couplings.rs` |
| `parameters.py` | Not parsed at all | Workaround: `parameters.rs` |
| `particles.py` | Drops `r'...'` raw string texnames | Workaround: fixed in `parameters.rs` |

## What FeynGraph drops that ALOHA needs

ALOHA (Automatic Lorentz and Helicity Amplitude generator) derives numerical
routines from the symbolic Lorentz structures and coupling values in a UFO
model. FeynGraph discards exactly the information ALOHA requires:

### 1. Lorentz structure expressions (`lorentz.py`)

FeynGraph's `lorentz_atom` rule (ufo_parser.rs lines 299-303) recognises
`Identity`, `Gamma`, `Sigma`, `Gamma5`, `ProjM`, `ProjP`, `C` but only
extracts the pair of *spinor leg indices* `(i, j)` — discarding the operator
type entirely. The full `structure` string (e.g.
`"Gamma(1,2,3)*ProjM(4,5)"`) is reduced to just the connectivity `[(2,5)]`
(which spinor legs are linked). Crucially:

- `P(mu, i)` (momentum insertions) — parsed as `None`, silently dropped
- `Metric(mu,nu)` (vector boson Lorentz contractions) — same
- `Epsilon(mu,nu,rho,sigma)` (Levi-Civita) — same
- The operator type (`Gamma` vs `Identity` vs `ProjM`) — stripped away

ALOHA needs the full symbolic expression to emit code like
`Gamma(mu,i,j)*P(nu,k)` → a Dirac-matrix times momentum tensor contraction.

### 2. Coupling values (`couplings.py`)

In the `coupling` rule (ufo_parser.rs line 258), `"value" => ()` explicitly
discards the complex-valued expression string (e.g. `"-(ee*complex(0,1))"`).
FeynGraph only retains `order = {'QED':1}` for power-counting purposes.
ALOHA needs the value to evaluate the coupling constant numerically.

### 3. Color structures (`vertices.py`)

In `parse_vertex`, `"color" => ()` discards color factor strings like
`'f(1,2,3)'` (SU(3) structure constants) and `'T(1,2,3)'` (fundamental
generators). These are needed for color-summed squared amplitudes.
`InteractionVertex` has no field to store them.

### Implication for ALOHA implementation

To implement ALOHA-style automatic Lorentz routine generation we will need to
either take full ownership of UFO parsing (Options B or C above) or add
parallel parsers for `lorentz.py` and the coupling `value` field, similar to
what was done for `parameters.py` and `couplings.rs`. The color structures
will also need a new parser and storage layer.

The minimum viable ALOHA path (assuming LO SM only) is:
1. Parse `lorentz.py` fully — preserve the `structure` string and parse it
   into a symbolic tensor-product AST (operators × index labels)
2. Extend `couplings.rs` to recover the `value` expression (already dropped
   by FeynGraph; our `CouplingValue` struct already has a `value: String`
   field ready for this)
3. Add color factor parsing to `vertices_ext.rs` (currently `"color" => ()`)
4. Feed the symbolic Lorentz AST through a code-generator that emits Rust
   functions matching the trait interface already defined in `src/helas/repr.rs`
