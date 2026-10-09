---
type: Codebase Survey
title: FeynGraph
description: "FeynGraph read at fd5aa83, the rev vibegraph builds against: layout, what its UFO parser keeps and drops, the two-stage generator, the Model builder, DiagramView and DiagramSelector."
resource: "https://github.com/Jens-Braun/FeynGraph/tree/fd5aa8306746b432e098c40dcd96d7cb7ef20125"
status: draft
tags: [feyngraph, diagrams, ufo, rust, external-code]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n02-fg, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/02-reference-implementations.md#L17-L294", title: "Note 02, FeynGraph survey (read at 1dc4ea7)"}
  - {id: n02-cross, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/02-reference-implementations.md#L519-L555", title: "Note 02, cross-cutting notes: UFO parsing, MadGraph vs FeynGraph"}
  - {id: n04-limits, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/04-ufo-parsing-future.md#L88-L134", title: "Note 04, FeynGraph parser limitations and what it drops"}
  - {id: n06-gap, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/06-process-grammar.md#L460-L544", title: "Note 06 §8, feyngraph gap analysis"}
  - {id: n00-refs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/00-overview.md#L57-L69", title: "Note 00, references"}
  - {id: fg-parser, resource: "https://github.com/Jens-Braun/FeynGraph/blob/fd5aa8306746b432e098c40dcd96d7cb7ef20125/src/model/ufo_parser.rs", title: "FeynGraph ufo_parser.rs at fd5aa83"}
  - {id: fg-model, resource: "https://github.com/Jens-Braun/FeynGraph/blob/fd5aa8306746b432e098c40dcd96d7cb7ef20125/src/model/mod.rs", title: "FeynGraph model/mod.rs at fd5aa83"}
  - {id: fg-diagram, resource: "https://github.com/Jens-Braun/FeynGraph/blob/fd5aa8306746b432e098c40dcd96d7cb7ef20125/src/diagram/mod.rs", title: "FeynGraph diagram/mod.rs at fd5aa83"}
  - {id: fg-filter, resource: "https://github.com/Jens-Braun/FeynGraph/blob/fd5aa8306746b432e098c40dcd96d7cb7ef20125/src/diagram/filter.rs", title: "FeynGraph diagram/filter.rs at fd5aa83"}
  - {id: topo-rs, resource: "vibegraph-lib/src/ufo/topo.rs", title: "build_feyngraph_model: vibegraph's UFO data into a feyngraph Model"}
  - {id: cargo, resource: "vibegraph-lib/Cargo.toml#L34", title: "feyngraph git dependency, rev fd5aa8306746"}
---

FeynGraph (Jens Braun, MPL-2.0, Rust edition 2024, version `0.1.0-beta.5`) is a
Feynman diagram generator for tree and loop level. vibegraph uses it for one
job: enumerating diagrams from a model it builds itself. It does not use
FeynGraph's UFO parser, and FeynGraph computes no amplitudes. How vibegraph
drives it is [diagram enumeration](../../process/diagram-enumeration.md); this
concept describes FeynGraph itself.

## Which revision

`vibegraph-lib/Cargo.toml` pins the git dependency at
`fd5aa8306746b432e098c40dcd96d7cb7ef20125` (2026-08-06) with
`default-features = false`[^cargo]. The `research/refs/feyngraph` submodule is
pinned at `1dc4ea7` (2026-05-08), which is what note 02 read[^n02-fg].
`fd5aa83` is eight commits later and this concept is read at `fd5aa83`. Between
the two:

- the drawing system was rewritten (SVG, TikZ and Typst backends);
- `Model::add_particle` was reworked to check existing names and keep
  `anti_map` correct when a particle is replaced;
- `thiserror` was dropped as a dependency;
- `generate_diagrams` and the selector methods take `impl AsRef<str>`;
- the UFO string rule accepts a raw-string prefix `r'...'`;
- leg ids can be queried from the Rust interface.

The parser rules, structs and generator entry points below sit at the same or
nearby lines in both revisions. A reader of the submodule should expect
`model/mod.rs` and `diagram/mod.rs` line numbers to differ by 10–80 lines.

## Source layout

| Path | Contents |
|---|---|
| `src/lib.rs` | re-exports; `generate_diagrams()` (line 28) |
| `src/model/mod.rs` | `Particle`, `InteractionVertex`, `Model`, `LineStyle`, `Statistic` |
| `src/model/ufo_parser.rs` | PEG (`peg` 0.8) UFO 2.0 parser; the SM embedded with `include_str!` for `Model::default()` |
| `src/model/qgraf_parser.rs` | QGRAF model format |
| `src/topology/` | `TopologyGenerator`, `TopologyWorkspace` (adjacency-matrix backtracking), momentum routing |
| `src/diagram/` | `DiagramGenerator`, `AssignWorkspace` (particle assignment), `DiagramView`, `DiagramSelector` |
| `src/drawing/`, `src/bindings/` | drawing backends; PyO3 and Wolfram bindings |

Dependencies: `itertools`, `rayon`, `log`, `indexmap` (insertion-ordered maps,
so particle and vertex order is reproducible), `either`, `peg`, `rustc-hash`.

## The UFO parser: what it keeps and drops

`Model::from_ufo(path)` (`model/mod.rs:612`) calls `parse_ufo_model`
(`ufo_parser.rs:587`), which reads `particles.py`, `coupling_orders.py`,
`couplings.py`, `lorentz.py` and `vertices.py`. It never reads
`parameters.py`[^fg-parser]. The parser keeps only what topology needs:

- **Particles** keep name, antiname, PDG code, spin (stored as `2s`), colour,
  texnames and line style. `mass`, `width` and every other property fall
  through `_ => ()`.
- **Couplings** keep only `order`. The `value` expression is discarded
  (`ufo_parser.rs:258`: `"value" => (),`).
- **Lorentz structures** are reduced to a spin map. `lorentz_atom`
  (`ufo_parser.rs:299`) recognises `Identity`, `Gamma5`, `ProjM`, `ProjP`, `C`,
  `Gamma` and `Sigma`, and returns only the spinor index pair:

  ```rust
  rule lorentz_atom() -> (isize, isize) =
      ("Identity" /  "Gamma5" / "ProjM" / "ProjP"  / "C") _
      "(" _ i:int() _ "," _ j:int() _ ")" {? Ok((i.int()? - 1, j.int()? - 1)) }
      / "Gamma" _ "(" _ int() _ "," _ i:int() _ "," _ j:int() _ ")" {? ... }
      / "Sigma" _ "(" _ int() _ "," _ int() _ "," _ i:int() _ "," _ j:int() _ ")" {? ... }
  ```

  Everything else in the structure string (`P`, `Metric`, `Epsilon`, the
  operator type) is skipped.
- **Vertices** keep particles, Lorentz names (for their spin maps) and the
  coupling dictionary (for orders). `parse_vertex` discards `color`,
  `loop_particles` and `type` (`ufo_parser.rs:445`).

So FeynGraph's model carries no masses, widths, coupling values, Lorentz
tensors or colour structures, which is everything an amplitude needs[^n04-limits].
Note 04 also found that `particles.py` files with attribute assignments
(`x.attr = ...`) after the particle definitions do not parse, so `loop_sm`
fails to load. The `particles` rule at `fd5aa83` still matches only `Particle(...)`
and `.anti()` statements. Note 04's raw-string (`r'...'`) limitation is fixed at
`fd5aa83`.

**Vertex splitting.** A UFO vertex is split only when it is ambiguous, and the
criterion is distinct spin maps and distinct coupling-order sets, not the
number of Lorentz structures. Two structures with the same spinor pairing stay
one `InteractionVertex`. When a split happens, the parts are named
`<name>_0 … <name>_{k-1}`, a warning is logged, and `Model.splittings` records
which `(colour, lorentz)` coupling keys each part took[^fg-parser].

## Model builder

`Model` (`model/mod.rs:307`) holds `particles: IndexMap<String, Particle>`,
`vertices: IndexMap<String, InteractionVertex>`, the coupling names,
`splittings`, and `anti_map` (particle index to antiparticle index). Its
fields are private and those of `Particle` and `InteractionVertex` are
`pub(crate)`, so reading goes through accessors. `Model::default()` is the embedded SM in
Feynman gauge. A model can be built without any file[^fg-model]:

```rust
Model::empty()
model.add_particle(name, anti_name, spin, color, pdg_code,
                   texname, antitexname, linestyle, statistic)   // adds the antiparticle too
model.add_vertex(name, particles, spin_map, coupling_orders)?    // spin_map[i] = leg spin-connected to leg i
```

`add_particle` adds the antiparticle itself when `name != anti_name`, so a
caller passes each particle–antiparticle pair once.

vibegraph builds its model this way in `ufo/topo.rs::build_feyngraph_model`[^topo-rs],
from the model it parsed itself ([UFO parsing](../../model/ufo-parsing.md)). It
drops Goldstones and ghosts (unitary gauge), adds each pair once, and adds one
feyngraph vertex per fermion-flow group of each UFO vertex (`<name>@<g>` when
there are several). The UFO vertices it starts from are already split so that
every coupling of a vertex carries the same order tuple
([coupling orders](../../model/coupling-orders.md)). FeynGraph's own splitting
rule is therefore never exercised.

## Diagram generation

Two stages[^n02-fg]:

1. **Topologies.** `TopologyGenerator::generate()` (`topology/mod.rs:690`)
   enumerates node-degree partitions satisfying `Σ(k−2)·N_k = 2L − 2 + E`,
   fills adjacency matrices by backtracking (`TopologyWorkspace`), and routes
   momenta with a Tarjan bridge-finding DFS (bridges are the 1PI separators;
   loop momenta go on back edges). Momenta are `Vec<i8>` coefficients over the
   external and loop basis.
2. **Particle assignment.** `DiagramGenerator::generate()`
   (`diagram/mod.rs:524`) assigns particles to each topology by backtracking
   (`AssignWorkspace::select_vertex`, `select_leg`), in parallel over
   topologies with rayon.

Entry points[^fg-diagram]:

```rust
pub fn generate_diagrams(
    particles_in:  impl IntoIterator<Item = impl AsRef<str>>,
    particles_out: impl IntoIterator<Item = impl AsRef<str>>,
    n_loops: usize, model: Model, selector: DiagramSelector,
) -> Result<DiagramContainer, ModelError>

DiagramGenerator::new(particles_in, particles_out, n_loops, model, Some(selector))?.generate()
```

Particles are named by the model's `name` field, not by PDG code. There is no
process-string parser; turning a proc card into name lists is vibegraph's job
([proc-card grammar](../../process/proc-card-grammar.md)).

A `Diagram` stores incoming and outgoing `Leg`s (vertex, particle index,
momentum), `Propagator`s (two vertex ids, particle, momentum), `Vertex`es
(propagator ids with −1 for external legs, interaction index), vertex and
propagator symmetry factors, bridges, and `sign: i8`, the ±1 from fermion
ordering. These fields are `pub(crate)` too. The public read interface is
`DiagramView` (`diagram/view.rs`): `incoming()`, `outgoing()`, `legs()`,
`propagators()`, `vertices()`, `bridges()`, `symmetry_factor()`, `sign()`,
`order(coupling)`, `orders()`, `count_particles`, `count_vertices`, and per-leg,
per-propagator and per-vertex views (`particle()`, `momentum()`,
`interaction()`).

## DiagramSelector

Filters applied during generation (`diagram/filter.rs`)[^fg-filter]:

| Method | Keeps diagrams with | MadGraph analogue |
|---|---|---|
| `select_coupling_power(c, n)` | exactly `n` powers of coupling `c` | `QCD=2` |
| `select_coupling_power_list(c, ns)` | a power of `c` in `ns` | `QCD<=2` |
| `select_propagator_count(p, n)` | exactly `n` propagators of species `p` | `/ p` with `n = 0` |
| `select_vertex_count(ps, n)` | `n` vertices involving the named particles | none |
| `select_vertex_degree(d, n)` | `n` vertices of degree `d` | none |
| `select_opi_components`, `select_self_loops`, `select_tadpoles`, `select_on_shell` | loop-level structure filters | implicit at LO |
| `add_custom_function(Arc<dyn Fn(&DiagramView) -> bool + Sync + Send>)` | an arbitrary predicate | `$`, `>` s-channel rules |
| `add_topology_function(...)` | a predicate on the topology | none |

vibegraph uses `select_propagator_count(name, 0)` for forbidden particles and
the two coupling-power selectors for order constraints
(`diagrams/selector.rs`).

## Against MadGraph's generator

| | MadGraph `Amplitude` | FeynGraph |
|---|---|---|
| Approach | leg combination (`reduce_leglist`) | topology first, then particle assignment |
| Duplicates | `DiagramTag` canonical hashing | symmetry factors at the topology stage |
| Loops | no (separate aMC@NLO machinery) | yes, `n_loops` |
| Parallel | no | rayon |

The two must agree on the LO diagram set; the
[structural censuses](../../validation/structural-censuses.md) check that against
MadGraph's own counts. MadGraph's side is in
[the MadGraph survey](madgraph5-amcnlo.md).

[^cargo]: `vibegraph-lib/Cargo.toml:34`.
[^n02-fg]: Note 02, FeynGraph survey, read at the submodule pin `1dc4ea7`.
[^n04-limits]: Note 04, "Known FeynGraph limitations" and "What FeynGraph drops that ALOHA needs".
[^fg-parser]: `src/model/ufo_parser.rs` at `fd5aa83`: rules `particle` (157), `coupling` (253), `lorentz_atom` (299), `particles` (355); `parse_vertex` (424–585).
[^fg-model]: `src/model/mod.rs` at `fd5aa83`: `Model` (307), `empty` (357), `add_particle` (370), `add_vertex` (485), `from_ufo` (612).
[^topo-rs]: `vibegraph-lib/src/ufo/topo.rs`, `build_feyngraph_model`.
[^fg-diagram]: `src/lib.rs:28` and `src/diagram/mod.rs` at `fd5aa83` (`Diagram` at 89, `generate` at 524).
[^fg-filter]: `src/diagram/filter.rs` at `fd5aa83`, lines 72–157.
