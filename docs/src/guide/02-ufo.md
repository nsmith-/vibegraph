# UFO models

A generator needs to know the theory: which particles exist, how they
interact, and with what strength. The **Universal FeynRules Output** (UFO,
[Degrande et al. 2012](../bibliography.md#models)) is the format the
MadGraph ecosystem uses for that, and it is the only model input vibegraph
accepts. A UFO model is not a data file but a Python package: a directory
of modules that, when imported, build lists of `Particle`, `Parameter`,
`Vertex`, `Lorentz` and `Coupling` objects. FeynRules writes these
packages from a Lagrangian; generators read them.

| File | Contents |
|---|---|
| `particles.py` | PDG code, spin, colour representation, mass and width parameters, antiparticle |
| `parameters.py` | External parameters (inputs, with their `param_card` block and index) and internal ones (derived, as Python expression strings) |
| `lorentz.py` | Lorentz tensor structures as expression strings over `Gamma`, `ProjM`, `Metric`, `P`, … |
| `couplings.py` | Coupling constants as expression strings over the parameters |
| `vertices.py` | Interactions: a particle list, a list of colour structures, a list of Lorentz structures, and a coupling for each (colour, Lorentz) pair |
| `coupling_orders.py` | The order-counting hierarchy (`QCD`, `QED`) |

The organising idea is that a vertex is a tensor product over three
independent axes:

\\[
V = \sum_{c,\,l} g_{cl}\; \mathcal{C}_c \otimes \mathcal{L}_l ,
\\]

a colour structure \\(\mathcal{C}_c\\), a Lorentz structure
\\(\mathcal{L}_l\\), and a numerical coupling \\(g_{cl}\\) for each pair. This
factorisation is what lets the [colour algebra](05-color.md) be done
symbolically once per process while the [Lorentz structures](04-helicity-amplitudes.md)
are evaluated numerically per phase-space point, and it is what the
[amplitude compiler](06-compiler.md) consumes.

## Reading Python without running it

vibegraph does not embed a Python interpreter. The model files are parsed
into a Python abstract syntax tree with the `rustpython-parser` crate, and
the object constructions are read straight off the tree: an assignment
whose right-hand side is a call to `Particle(...)`, `Vertex(...)` and so on,
with keyword arguments. That is enough because UFO files are written by a
program and use a fixed, declarative subset of the language.

Two things in a model are genuinely expressions and need an evaluator. The
values of internal parameters and of couplings are strings such as

```python
'2*cmath.sqrt(aS)*cmath.sqrt(cmath.pi)'
'complex(0,1)*G'
```

The `ufo::expr` module parses these into an expression tree over the
arithmetic operators, `cmath` functions, `complex(...)`, and parameter
names, and evaluates them against a table of complex values. External
parameters take their values from a `param_card.dat` in the SLHA block
format ([Skands et al. 2004](../bibliography.md#models)); internal ones are
evaluated in dependency order from those. The result is an
`EvaluatedModel`: every parameter, mass, width and coupling as a number. A
model is loaded once, symbolically, and can be evaluated against any number
of parameter cards; the compiler keeps the same separation.

Lorentz structures are strings too, over a small vocabulary of tensor
symbols with integer index labels:

```python
FFV2 = Lorentz(name='FFV2', spins=[2, 2, 3],
               structure='Gamma(3,2,-1)*ProjM(-1,1)')
```

Positive indices name the vertex's legs, negative ones are summed. The
parser turns each structure into a list of operators (`Gamma`, `Sigma`,
`Identity`, `ProjM`, `ProjP`, `Metric`, `P`, `Epsilon`, `C`) with their
index slots, and works out which legs each spinor and Lorentz index belongs
to. The [helicity amplitudes](04-helicity-amplitudes.md) chapter is about
what is done with them.

Colour structures are strings over `T`, `f`, `d` and `Identity`, parsed by
`ufo::color`. `Identity(1,2)` is representation-dependent (a
\\(\delta_{ij}\\) between a triplet and an antitriplet, \\(2\,\mathrm{Tr}\\)
between two octets), so it is resolved at load time from the particles'
colour representations, which is where MadGraph resolves it too.

## Restriction cards

A UFO model is usually more general than a run needs. The Standard Model
ships with restriction cards (`restrict_no_b_mass.dat` and others) that set
chosen parameters to zero; MadGraph applies one when a card says
`import model sm-no_b_mass`. vibegraph bakes the restriction into the parsed
model: the restricted parameters are fixed, every coupling that evaluates to
zero under them is dropped, and every vertex left with no coupling is
removed before any diagram is enumerated. The restriction is therefore part
of the model's identity, not a runtime switch.

## The Standard Model, compiled in

Parsing a UFO directory needs the directory, and a user of a bare binary has
none. The SM UFO from the MadGraph distribution is parsed once by a
developer tool, serialised with `bincode`, compressed with `zstd`, and
compiled into the library as a byte blob together with its nine restriction
cards. `import model sm` and its variants deserialise the blob and apply the
card, touching no file. The `sm_interned_blob` validation test regenerates
the blob from the pinned MadGraph submodule and fails when the committed one
has drifted.

Any other model is a directory: `import model <name>` resolves it through
the [cache](../cli/overview.md#data-the-binary-does-not-carry). The loader
is generic, but the representation surface is the Standard Model's. Colour
sextets, baryonic epsilon tensors, spin \\(\ge 3/2\\), Majorana fermions and
loop-level models are rejected with an error rather than silently
approximated.

## Model identity

An integration artifact must be able to refuse a mismatched replay, and a
model name is not enough to tell two models apart: the same name can
resolve to different bytes if a restriction card or the interned assets
change. The model identity (`ufo::identity`) is therefore the import label
together with a SHA-256 digest over the model's own serialised form, after
the restriction is applied. Two directories that differ only in comments or
formatting parse to the same model and get the same digest; two that differ
in a coupling do not.

## Where it lives

`vibegraph::ufo`, with one submodule per file kind
([`particles`](../api/vibegraph/ufo/particles/index.html),
[`parameters`](../api/vibegraph/ufo/parameters/index.html),
[`lorentz`](../api/vibegraph/ufo/lorentz/index.html),
[`couplings`](../api/vibegraph/ufo/couplings/index.html),
[`vertices`](../api/vibegraph/ufo/vertices/index.html),
[`color`](../api/vibegraph/ufo/color/index.html)), the expression evaluator in
[`expr`](../api/vibegraph/ufo/expr/index.html), the SLHA reader in
[`slha`](../api/vibegraph/ufo/slha/index.html), the interned model in
[`sm`](../api/vibegraph/ufo/sm/index.html) and the digest in
[`identity`](../api/vibegraph/ufo/identity/index.html).
[`UFOModel`](../api/vibegraph/ufo/struct.UFOModel.html) is the loaded model;
[`config`](../api/vibegraph/config/index.html) resolves an `import model`
directive to one.
