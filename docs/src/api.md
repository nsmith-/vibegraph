# Library API

The `vibegraph-lib` crate (library name `vibegraph`) holds everything but the
command-line front end. Its API reference is built by `cargo doc` from the
source's doc comments and published alongside this book:

<p style="font-size:1.2em"><a href="api/vibegraph/index.html">Open the rustdoc reference →</a></p>

Doc comments use LaTeX in places (the HELAS conventions, for instance), so
the reference is built with a KaTeX header; `pixi run docs` and
`scripts/build-docs.sh` both do this, a plain `cargo doc` renders the
formulae as source.

## The modules, in pipeline order

| Module | What it holds | Tour chapter |
|---|---|---|
| [`ufo`](api/vibegraph/ufo/index.html) | The UFO model loader: a Python-AST parser for the model files, expression evaluation for parameters and couplings, Lorentz and colour structure parsers, the interned Standard Model, and the model identity digest | [UFO models](guide/02-ufo.md) |
| [`diagrams`](api/vibegraph/diagrams/index.html) | The MadGraph process grammar, multiparticle aliases and coupling-order selection, and the bridge to feyngraph's diagram generation | [Feynman diagrams](guide/03-diagrams.md) |
| [`helas`](api/vibegraph/helas/index.html) | Helicity amplitudes: the representation layer (`repr`), external wavefunctions (`wavefn`), hand-written vertex routines (`vertex`), the symbolic colour algebra (`color`), and the amplitude compiler and evaluator (`eval`) | [Helicity amplitudes](guide/04-helicity-amplitudes.md), [Colour](guide/05-color.md), [The amplitude compiler](guide/06-compiler.md) |
| [`phasespace`](api/vibegraph/phasespace/index.html) | Phase-space maps: RAMBO, the two-body LIPS map, per-diagram multichannel maps with resonance and t-channel sampling, and the counter-based random substreams | [Phase space](guide/07-phase-space.md) |
| [`vegas`](api/vibegraph/vegas/index.html) | The VEGAS adaptive integrator, its iteration-combination rules, and frozen-grid sampling | [VEGAS](guide/08-vegas.md) |
| [`budget`](api/vibegraph/budget/index.html) | How a multichannel run splits points across channels and iterations, and when it may stop | [Multichannel integration](guide/09-multichannel.md) |
| [`hadronic`](api/vibegraph/hadronic/index.html) | The cross-section integrands for fixed-energy beams, with flux and averaging factors | [Phase space](guide/07-phase-space.md) |
| [`proton`](api/vibegraph/proton/index.html) | Flavour groups, both beam orderings, and the PDF-convolved integrand for proton beams | [Proton beams](guide/10-hadronic.md) |
| [`pdf`](api/vibegraph/pdf/index.html) | A pure-Rust LHAPDF6 grid reader with LHAPDF's log-bicubic interpolation | [Proton beams](guide/10-hadronic.md) |
| [`coupling`](api/vibegraph/coupling/index.html) | The strong coupling's running, the per-event scale prescriptions, and MadGraph's kT clustering | [Proton beams](guide/10-hadronic.md) |
| [`cuts`](api/vibegraph/cuts/index.html) | The run card's phase-space cuts, compiled per process with MadGraph's conventions | [Proton beams](guide/10-hadronic.md) |
| [`runcard`](api/vibegraph/runcard/index.html) | The `run_card.dat` parser with MadGraph's LO defaults | [Command line](cli/overview.md) |
| [`unweight`](api/vibegraph/unweight/index.html) | Accept/reject unweighting over frozen grids and the maximum-weight rule | [Unweighting and event files](guide/11-unweighting-events.md) |
| [`select`](api/vibegraph/select/index.html) | The per-event helicity and colour-flow draws | [Unweighting and event files](guide/11-unweighting-events.md) |
| [`lhef`](api/vibegraph/lhef/index.html) | Les Houches event files: records, MadGraph's byte-exact layout, parsing, event assembly, and weighting strategies | [Unweighting and event files](guide/11-unweighting-events.md) |
| [`artifact`](api/vibegraph/artifact/index.html) | The `grid.bin.zst` artifact that carries an integration to the generation phase | [Command line](cli/overview.md) |
| [`cache`](api/vibegraph/cache/index.html) | Resolving and pinning PDF sets and UFO models on a user's machine | [Command line](cli/overview.md) |
| [`config`](api/vibegraph/config/index.html) | Resolving a card's `import model` directive to a loaded model | [UFO models](guide/02-ufo.md) |
| [`progress`](api/vibegraph/progress/index.html) | The machine-readable progress stream a live display subscribes to | [Command line](cli/overview.md) |
| [`stats`](api/vibegraph/stats/index.html) | Weighted two-sample tests for comparing event samples | [Validation](guide/12-validation.md) |

## Using the crate

`vibegraph-lib` is not published on crates.io; depend on it by git or path.
The entry points a program uses, in order, are
[`UFOModel`](api/vibegraph/ufo/struct.UFOModel.html) or the interned
[`sm_model`](api/vibegraph/ufo/sm/fn.sm_model.html), the process card parser
[`parse_proc_card`](api/vibegraph/diagrams/fn.parse_proc_card.html) and
[`generate_from_proc_card`](api/vibegraph/diagrams/fn.generate_from_proc_card.html),
[`AmplitudeEvaluator::compile`](api/vibegraph/helas/eval/struct.AmplitudeEvaluator.html#method.compile)
and [`BoundAmplitude::bind`](api/vibegraph/helas/eval/struct.BoundAmplitude.html#method.bind),
and then the integrands in `hadronic` and `proton` that drive the
[`Vegas`](api/vibegraph/vegas/struct.Vegas.html) integrator. The `vibegraph`
binary's `integrate` and `generate` commands are worked examples of that
sequence, and the crate's integration tests under `vibegraph-lib/tests/` are
others.

## Building the reference locally

```bash
pixi run docs                 # rustdoc with the KaTeX header, into target/doc/
scripts/build-docs.sh         # the whole site, into target/site/ (needs mdbook)
```
