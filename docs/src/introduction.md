# vibegraph

vibegraph is a leading-order, tree-level Monte Carlo event generator written
in Rust. A [UFO model](guide/02-ufo.md) and a MadGraph-style process card go
in; an unweighted Les Houches event file comes out. Every stage follows the
pipeline of [MadGraph5\_aMC@NLO](https://arxiv.org/abs/1405.0301) closely
enough that it is validated against it, at the strictest level each stage
permits.

```text
UFO model ──▶ diagram enumeration ──▶ helicity amplitudes (HELAS/ALOHA)
                                              │
        LHEF events ◀── unweighting ◀── σ integration ◀── phase-space sampling
                                                          (multichannel VEGAS)
```

This site has three parts.

**[A guided tour](guide/01-pipeline.md)** is a pedagogical introduction to
what an event generator computes and how this one computes it: the physics
of each stage, the algorithm chosen for it, the paper the algorithm comes
from, and where in the crate it lives. One chapter,
[the amplitude compiler](guide/06-compiler.md), is about computer science
rather than physics: vibegraph does not generate and compile source code per
process the way MadGraph does, it compiles the model's Lorentz structures
into an executable program at load time, and that chapter sketches the
compiler concepts involved.

**[Command line](cli/overview.md)** describes the `vibegraph` binary: the
two-phase `integrate` / `generate` workflow, the artifact that connects them,
and how data the binary does not carry is resolved. The
[command reference](cli/reference.md) is generated from the binary's own
`--help` output.

**[Library API](api.md)** introduces the `vibegraph-lib` crate module by
module and links into the rustdoc reference built from its doc comments.

The [bibliography](bibliography.md) collects every paper cited in the tour.

## Reading order

Readers who want to run the generator can start with the
[command line](cli/overview.md). Readers who want to understand it should
read the tour in order: each chapter assumes the ones before it, and the
chapters are interlinked where a concept is used before it is introduced.
Readers who want to change it will find the tour's "where it lives" pointers
and the [API reference](api.md) the fastest route into the source.

## Conventions

Natural units, \\(\hbar = c = 1\\), with GeV as the energy scale. The metric
signature is \\((+,-,-,-)\\). Four-momenta are stored as `[E, px, py, pz]`.
Cross sections are computed in GeV\\(^{-2}\\) and reported in picobarns.
