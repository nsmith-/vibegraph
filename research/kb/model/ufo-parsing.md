---
type: Design
title: "UFO parsing: Python AST for files, PEG for expression strings"
description: "UFO files are parsed with rustpython-parser into our own UFOModel; PEG grammars read only expression strings; feyngraph gets topology only; propagators.py is parsed and attached per particle."
status: draft
tags: [ufo, parser, feyngraph, model, propagators]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-10}]
sources:
  - {id: n01-ufo, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/01-paper-summaries.md#L57-L77", title: "Note 01, UFO module structure and data model"}
  - {id: n04-options, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/04-ufo-parsing-future.md#L30-L97", title: "Note 04, full UFO parsing: options, recommendation, FeynGraph's parser gaps"}
  - {id: n35-l1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L604-L675", title: "Note 35 §4 L1, loader and model-topology surface (propagators.py)"}
  - {id: code-ufo, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/ufo/mod.rs", title: "vibegraph-lib/src/ufo/mod.rs"}
  - {id: code-topo, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/ufo/topo.rs#L64-L183", title: "vibegraph-lib/src/ufo/topo.rs build_feyngraph_model"}
---

vibegraph reads UFO models itself. The `.py` files are parsed as Python, the quoted
expression strings inside them are parsed by small PEG grammars, and feyngraph receives
only the topology it needs to enumerate diagrams. The paper that defines the format is
[UFO](../references/papers/ufo.md).

## What a UFO directory holds, and what is read

| File | Contents | Read here |
|---|---|---|
| `particles.py` | PDG code, spin, colour, mass, width, charge, antiparticle | required |
| `lorentz.py` | Lorentz structures: per-leg spins and a `structure` string | required |
| `couplings.py` | coupling `value` strings and their `order` dictionaries | required |
| `parameters.py` | `external` parameters (`lhablock`, `lhacode`) and `internal` ones (`value` expression) | required |
| `vertices.py` | particles, `color` strings, `lorentz` names and `couplings={(colour_i, lorentz_j): C.GC_n}` | required |
| `coupling_orders.py` | each order's `hierarchy` and `expansion_order` | optional; absent → `QCD = 1, QED = 2` |
| `propagators.py` | custom propagator forms | optional |
| `restrict_*.dat` | restrict cards (SLHA) | by `import model <name>-<variant>` |
| `object_library.py`, `function_library.py`, `decays.py`, `write_param_card.py`, `CT_*.py` | base classes, helper functions, widths, card writer, loop counterterms | not read |

`REQUIRED_SOURCE_FILES` and `OPTIONAL_SOURCE_FILES` (`ufo/mod.rs`) list them in read
order. [^code-ufo] The helper functions of `function_library.py` (`complexconjugate`,
`re`, `im`, `sec`, `csc`, `asec`, `acsc`, …) are built into the expression grammar
instead of being read from the file; see [UFO string grammars](ufo-string-grammars.md).
[^n01-ufo]

## Two parsing layers

**Files: a Python AST.** Each `.py` file is parsed with `rustpython-parser` (0.3) and
its assignment statements are walked (`ufo/ast_util.rs`: `parse_stmts`,
`call_func_name`, `kwarg_str`, `kwarg_int`). A real Python parser handles raw strings,
escaped quotes, multi-line calls and attribute assignments without grammar maintenance.
`particles.py` handles both `Particle(...)` constructors and the `u__tilde__ = u.anti()`
shorthand; attribute assignments such as `loop_sm`'s `.counterterm = …` are skipped.

**Strings: PEG grammars.** Only the quoted expression strings are parsed with `peg`:

- `ufo/expr.rs`: parameter and coupling `value` strings;
- `ufo/lorentz.rs`: Lorentz `structure` strings;
- `ufo/color.rs`: vertex colour strings, with `Identity` resolved against the particles'
  colour representations at load.

This is the split note 04 recommended for a loader that should survive real UFO files:
a Python AST walker for the structure, PEG kept for the value strings whose forms the
loader controls. [^n04-options] Note 04's other options (extending hand-written PEG
grammars over the `.py` files, or keeping FeynGraph's topology parser) were not taken,
and its file names (`particles_ext.rs`, PEG parsers for couplings and parameters)
predate the rewrite. The current modules are `particles.rs`, `couplings.rs`,
`parameters.rs`, `lorentz.rs`, `vertices.rs` and `propagators.rs` over `ast_util.rs`.

## The load path

`UFOModel::load_with_digest(dir, restrict)` is the one entry point for a directory:

1. `ParsedModel::parse` reads the files, resolves each vertex's names to indices
   (`resolve_vertices`), and splits interactions by coupling-order tuple
   ([coupling orders](coupling-orders.md)). A particle naming a propagator that
   `propagators.py` does not define is `UfoError::UnknownPropagator`.
2. The restrict card is applied ([restrict-card semantics](restriction-semantics.md)):
   explicit, else `restrict_default.dat` if the directory has one.
3. The digest of the restricted model is taken ([model identity](model-identity-digest.md)).
4. `into_model` builds the feyngraph topology model.

The Standard Model is not read from disk. `import model sm[-<variant>]` deserializes an
interned blob (`ufo/sm_assets/sm_parsed.bin.zst`, the pre-restriction `ParsedModel`)
and applies one of nine baked-in restrict cards. The blob is regenerated with the
`gen_sm_blob` dev binary; `pixi run check-sm-blob-fresh` (`--features
extended-validation`) compares it with a fresh parse of the pinned MadGraph submodule.
Because the blob holds parsed ASTs, an edit to any of the string grammars or to a
serialized type requires regenerating it in the same change. The hermetic suite does
not see a blob that decodes but is stale (a grammar that now parses differently); a
field added to a serialized type usually fails decoding outright, since bincode is not
schema-evolving. A non-SM model is found on the UFO search path (`--ufo-dir`,
an environment variable, or the asset cache; see
[asset resolution](../tooling/asset-resolution.md)), and a `-<variant>` suffix selects
`restrict_<variant>.dat` (`GlobalConfig::load_ufo_with_identity`, `config.rs`).

**Data model.** `UFOModel` holds `particles`, `lorentz`, `couplings` and `vertices` as
name-keyed `IndexMap`s (their order is what `ParticleId`, `CouplingId` and friends
index), the `ParameterSet`, the feyngraph `topo` model, `order_hierarchy` and
`expansion_order`. `ParsedModel`, the serialized pre-restriction form, holds the same
collections without `topo`, plus the parsed `propagators` forms, which `UFOModel` does
not carry. `EvaluatedModel` binds numeric values:
`from_model` at the restriction's defaults, `from_model_card` at a separate param card.
Nothing reachable from `ParsedModel` may be a `HashMap`, so that its serialization (and
the model digest) is deterministic.

## feyngraph gets topology only

feyngraph's own UFO parser is never called. `build_feyngraph_model` (`ufo/topo.rs`)
starts from `Model::empty()` and adds particles and vertices through feyngraph's
mutation API: [^code-topo]

- one `add_particle` per particle/antiparticle pair (feyngraph adds the antiparticle
  itself; adding both breaks generation), with spin as `2s`;
- Goldstone bosons, ghosts and every vertex touching one are skipped (unitary gauge);
- one `add_vertex` per split interaction and fermion-flow group, carrying the group's
  spin map and the interaction's coupling orders.

Everything else (masses, widths, couplings, Lorentz and colour data) stays in
`UFOModel`. Note 04 recorded why: at FeynGraph `1dc4ea7`, its parser dropped particle
`mass`/`width`, coupling `value` strings and raw-string texnames, did not parse
`parameters.py`, and failed on `loop_sm`'s attribute assignments. [^n04-options] Whether
those gaps remain at the pinned `fd5aa83` has not been rechecked, since nothing calls
that parser. How diagrams are then enumerated is in
[diagram enumeration](../process/diagram-enumeration.md) and
[FeynGraph](../references/codebases/feyngraph.md).

## Custom propagators

`propagators.py` is parsed (`ufo/propagators.rs`) into `ParsedModel::propagators`:
each `Propagator(...)` keeps its
`name`, `numerator` and `denominator` as verbatim strings, with the module-level string
variables the shipped files concatenate (`denominatorSq = denominator + "**2"`) folded
in. `Particle::propagator` records which form a particle uses. Nothing evaluates the
forms. Loading such a model succeeds; the hard error is raised only when a particle with
a custom propagator **propagates in a selected diagram** (`ConvertError::CustomPropagator`,
`diagrams/diagram.rs`). [^n35-l1] SMEFTsim's four auxiliary fields are the case in reach
([SMEFTsim](smeftsim-topu3l.md)). Evaluating the forms is a
[backlog item](../backlog/feature/custom-ufo-propagators-unevaluated.md).

## What the loader does not cover

- **Loop UFOs.** Counterterm files and loop attributes are ignored, and `[QCD]`
  processes are refused ([backlog](../backlog/feature/loop-level-ufo-models-unsupported.md)).
- **Spin-3/2, spin-2, Majorana** fields load but cannot be evaluated; see
  [UFO spin codes](ufo-aloha-type-matrix.md).

[^n01-ufo]: Note 01, the UFO paper summary: module structure and the vertex data model.
[^n04-options]: Note 04: what owning the parse buys, options A–C, the recommendation, and FeynGraph's parser gaps (read at FeynGraph `1dc4ea7`).
[^n35-l1]: Note 35 §4, the loader and model-topology surface: `propagators.py` parsed and the hard error moved to where a custom propagator is used.
[^code-ufo]: `vibegraph-lib/src/ufo/mod.rs`: `REQUIRED_SOURCE_FILES`, `ParsedModel::parse`, `UFOModel::load_with_digest`.
[^code-topo]: `vibegraph-lib/src/ufo/topo.rs` `build_feyngraph_model`.
