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

The organising idea is that a vertex is a sum of tensor products of a
colour structure and a Lorentz structure, with one numerical coupling per
pair:

$$
V = \sum_{c,\,l} g_{cl}\; \mathcal{C}_c \otimes \mathcal{L}_l .
$$

The two independent axes are the colour structures $\mathcal{C}_c$ and the
Lorentz structures $\mathcal{L}_l$; the coupling $g_{cl}$ is a scalar
indexed by the pair, and most pairs a vertex could form are absent. This
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
$\delta_{ij}$ between a triplet and an antitriplet, $2\,\mathrm{Tr}$
between two octets), so it is resolved at load time from the particles'
colour representations, which is where MadGraph resolves it too.

## Restriction cards

A UFO model is usually more general than a run needs. The Standard Model
ships with restriction cards (`restrict_no_b_mass.dat` and others) that set
chosen parameters to zero; MadGraph applies one when a card says
`import model sm-no_b_mass`. What a restriction card means is only which
parameters are zero. vibegraph bakes that into the parsed model: with the
restricted parameters set to zero, every coupling that evaluates to zero is
dropped, and every vertex left with no coupling is removed before any
diagram is enumerated. The card's non-zero values carry no such weight; the
`param_card.dat` read at run time supplies the values the run actually uses
and may override them. The pruning is therefore part of the model's
identity, not a runtime switch, and the parameter values are not.

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
the [cache](../cli/overview.md#data-the-binary-does-not-carry), and
`import model <name>-<restrict>` applies the model's own
`restrict_<restrict>.dat`, whose values become the parameter defaults the way
MadGraph's generated `param_card.dat` records them. The loader splits a UFO
vertex into one interaction per coupling-order tuple and prunes the
couplings a restriction zeroes, as MadGraph's `import_ufo` does, so diagram
counts agree with MadGraph's per split interaction. What the loader and the
evaluator then reach beyond the Standard Model is the next section.

## Beyond the Standard Model

The Lorentz and colour surface reaches past the Standard Model's, and every
claim below is a row of `validation/manifest.toml` enforced against
MadGraph's own matrix elements per diagram, per helicity and per colour flow
(the [validation chapter](12-validation.md) describes the gate).

**SMEFTsim.** `SMEFTsim_topU3l_MwScheme_UFO` — a dimension-six SMEFT model
with derivative gauge-boson vertices, Levi-Civita structures, $\gamma^5$ and
momenta inside $\gamma$-chains, four-fermion contacts in both pairings and a
cyclic tensor⊗tensor one, under the $\{m_W, m_Z, G_F\}$ input scheme — is
vendored byte for byte under `validation/ufo/` (MIT, tag `v3.0.2`), and
thirteen of its rows are enforced, from the SM limit up to the capstone
$e^+e^- \to t\bar t$ at `NP<=1`, whose cross section is gated against a
banked MadGraph run as well.

**Two toy models.** Structures no published model in reach isolates get a
UFO of their own. Two small models written for this repository, also under
`validation/ufo/` and licensed like the rest of it, put one such structure in
one vertex each, so a failure names the structure rather than a corner of a
900-vertex model: a literal $\sigma^{\mu\nu}$ (which FeynRules expands away
before it writes a model file), the bare `Identity` and $\gamma^5$ bilinears,
the symmetric colour structure constant $d^{abc}$, the baryonic
$\epsilon$ tensors and the sextet Clebsch coefficients. All six of their rows
are enforced, and two convention bugs surfaced there that no Standard-Model
process could isolate: the reversal sign of a fermion line is the
$C\Gamma^{\mathsf T}C^{-1}$ parity of its bilinears, not a count of its
propagators, and ALOHA's `Sigma` is half the textbook $\sigma^{\mu\nu}$.

**Hard errors.** Representations neither the models nor the toys reach —
spin $\ge 3/2$, spin-2, Majorana fermions and the charge conjugation they
need, a colour sextet on an external leg, loop-level models — are refused
with an error rather than silently approximated, as are squared-order
constraints (`NP^2==1`: a bound on an interference term, which this
generator selects diagrams too early to express).

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
