---
type: Codebase Survey
title: MadGraph5_aMC@NLO (pinned v3.7.1)
description: "MG5aMC at b7687064 (v3.7.1): layout, UFO import and ALOHA, the SM UFO, process parser and diagram_generation objects, the HELAS library, and where LO kT clustering lives."
resource: "https://github.com/mg5amcnlo/mg5amcnlo/tree/b7687064b9a013317ca164aa1395bc9c0e39ae1e"
status: draft
tags: [madgraph, aloha, helas, diagrams, external-code]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n02-mg, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/02-reference-implementations.md#L303-L510", title: "Note 02, MadGraph5_aMC@NLO survey"}
  - {id: n06-parser, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/06-process-grammar.md#L20-L45", title: "Note 06 §1, parser location in mg5amcnlo"}
  - {id: n06-flow, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/06-process-grammar.md#L334-L392", title: "Note 06 §6, parsed string to diagram generation"}
  - {id: n28-k1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L329-L345", title: "Note 28 K1, MadGraph kT clustering read at the pin"}
  - {id: mg-version, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/VERSION", title: "MadGraph VERSION (3.7.1, 2026-04-29)"}
  - {id: mg-dg, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/core/diagram_generation.py#L433-L545", title: "diagram_generation.py, Amplitude.generate_diagrams"}
  - {id: mg-tag, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/core/diagram_generation.py#L46-L245", title: "diagram_generation.py, DiagramTag"}
  - {id: mg-iface, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/interface/madgraph_interface.py#L4811-L4822", title: "madgraph_interface.py, do_generate and extract_process"}
  - {id: mg-import, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/models/import_ufo.py#L243", title: "models/import_ufo.py, import_model and UFOMG5Converter"}
  - {id: mg-aloha, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/aloha/create_aloha.py#L59-L159", title: "aloha/create_aloha.py, AbstractRoutine and AbstractRoutineBuilder"}
  - {id: mg-sm, resource: "https://github.com/mg5amcnlo/mg5amcnlo/tree/b7687064/models/sm", title: "models/sm, the SM UFO"}
  - {id: mg-helas, resource: "https://github.com/mg5amcnlo/mg5amcnlo/tree/b7687064/HELAS", title: "HELAS/, the hand-written Fortran library"}
  - {id: mg-multi, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/input/multiparticles_default.txt", title: "input/multiparticles_default.txt"}
---

MadGraph5_aMC@NLO is the reference generator vibegraph validates against: every
oracle compares with its output at a pinned version. The submodule
`research/refs/mg5amcnlo` is pinned at `b7687064b9a013317ca164aa1395bc9c0e39ae1e`,
tag `v3.7.1` (`VERSION`: 3.7.1, 2026-04-29)[^mg-version]. Every line number here is
3.7.1's. The submodule is checked out only to generate references; how is in
[the MadGraph toolchain](../../tooling/madgraph-toolchain.md). The paper is
[MadGraph5_aMC@NLO](../papers/madgraph5-amcatnlo.md).

Sibling surveys cover the MadEvent side: [phase-space maps](madevent-phase-space-maps.md),
[LHE output](madgraph-lhe-output.md), and the
[code-quality review and bug catalogue](madgraph5-code-quality-review.md).

## Layout

| Path | Contents |
|---|---|
| `madgraph/core/` | `base_objects.py` (`Particle`, `Interaction`, `Model`, `Process`, `Leg`, `Vertex`, `Diagram`), `diagram_generation.py`, `helas_objects.py`, `color_algebra.py` |
| `madgraph/interface/` | the command interpreter (`madgraph_interface.py`, `extended_cmd.py`) |
| `madgraph/iolibs/` | exporters (`export_v4.py` writes the MadEvent Fortran), `template_files/` |
| `models/` | `import_ufo.py`, `model_reader.py`, the bundled UFOs (`sm`, …) |
| `aloha/` | ALOHA: UFO Lorentz structures to HELAS-style routines |
| `HELAS/` | the hand-written Fortran HELAS library |
| `Template/LO/` | the MadEvent template: `Source/` and `SubProcesses/` |

## UFO import and ALOHA

MadGraph imports a UFO's Python modules directly and converts them into its own
`base_objects.Model` (`models/import_ufo.py`: `import_model` at 243,
`UFOMG5Converter` at 461)[^mg-import]. vibegraph instead parses the files without
running Python ([UFO parsing](../../model/ufo-parsing.md)).

ALOHA turns each UFO Lorentz structure into routines[^n02-mg]:

| Item | File:line |
|---|---|
| `UFOExpressionParser` (PLY lex/yacc; `parse` at 60) | `aloha/aloha_parsers.py:45` |
| `AbstractRoutine`, its `write(output_dir, language, …)` | `aloha/create_aloha.py:59`, `:91` |
| `AbstractRoutineBuilder`, `compute_routine(mode, tag, factorize)` | `aloha/create_aloha.py:120`, `:159` |
| `WriteALOHA` (code writer base) | `aloha/aloha_writers.py:28` |
| `L_P` (momentum object) | `aloha/aloha_object.py:40` |
| `Computation` (expression kernel) | `aloha/aloha_lib.py:63` |

The flow is parse the structure string, contract indices and insert the
propagator for the off-shell leg, store the expression tree, and write it out in
Fortran, C++ or Python. vibegraph does not generate per-vertex code; ALOHA's
routines are the oracle its runtime Lorentz evaluator is checked against
([ALOHA paper](../papers/aloha.md),
[intertwiner basis](../../amplitudes/intertwiner-basis-and-peephole.md)).

## The SM UFO (`models/sm/`)

Files: `particles.py`, `vertices.py`, `lorentz.py`, `couplings.py`,
`parameters.py`, `coupling_orders.py`, `object_library.py`,
`function_library.py`, `decays.py`, `write_param_card.py`, `build_restrict.py`,
and nine `restrict_*.dat` cards[^mg-sm]. `particles.py` was generated by FeynRules
1.7.69 and declares 24 `Particle(...)` objects plus 19 `.anti()` partners,
ghosts and Goldstones included. The first entries are `a` (22, line 10), `Z`
(23, line 24), `W__plus__` (24, line 38), `W__minus__ = W__plus__.anti()` (line
52) and `g` (21, line 54).

A vertex is `Vertex(name, particles, color, lorentz, couplings)` with
`couplings` keyed by `(colour index, Lorentz index)`:

```python
V_3 = Vertex(name = 'V_3',
             particles = [ P.G__minus__, P.G__minus__, P.G__plus__, P.G__plus__ ],
             color = [ '1' ],
             lorentz = [ L.SSSS1 ],
             couplings = {(0,0):C.GC_32})
```

`Lorentz(name, spins, structure)` carries the structure as a string ALOHA
parses. The spin codes and the Lorentz-structure families per spin are in
[the UFO/ALOHA type matrix](../../model/ufo-aloha-type-matrix.md). Parameters
are `nature='external'` (read from the param card by `lhablock`/`lhacode`) or
`nature='internal'` (an expression).

## Process parser

Paths in `madgraph/interface/madgraph_interface.py` unless noted[^mg-iface]:

| Method | Lines | Role |
|---|---|---|
| `do_generate` | 4811 | entry; delegates to `do_add` |
| `do_add` | 3232 | decay-chain check; calls `extract_process` or `extract_decay_chain_process` |
| `check_process_format` | 1150 | parentheses, `>` count, `/` and `$` placement |
| `extract_process` | 4822 | the core parser: strips modifiers by regex, then tokenises |
| `extract_decay_chain_process` | 5661 | recursion over comma-separated decays |
| `do_define` | 3527 | multiparticle aliases |
| `split_arg` | `extended_cmd.py:687` | whitespace split, quote-aware |

These lines match note 06's table at the pin. The default aliases
(`input/multiparticles_default.txt`)[^mg-multi]:

```text
p = g u c d s u~ c~ d~ s~
j = g u c d s u~ c~ d~ s~
l+ = e+ mu+
l- = e- mu-
vl = ve vm vt
vl~ = ve~ vm~ vt~
```

`extract_process` produces a `ProcessDefinition` whose fields include `legs`
(a `MultiLegList`; each `MultiLeg` has `ids`, `state` and `polarization`),
`orders`, `squared_orders` with `sqorders_types`, `constrained_orders`,
`forbidden_particles`, `forbidden_onsh_s_channels`, `forbidden_s_channels`,
`required_s_channels` (a list of PDG lists, OR within each), `id` (from `@N`),
`NLO_mode`, `split_orders` and `decay_chains`.
`diagram_generation.MultiProcess(procdef)` expands the aliases into one
`Amplitude` per concrete assignment[^n06-flow]. vibegraph's grammar and command
semantics are [the proc-card grammar](../../process/proc-card-grammar.md).

## Diagram generation (`madgraph/core/diagram_generation.py`)

`Amplitude` (line 433) generates lazily: `get('diagrams')` triggers
`generate_diagrams(returndiag, diagram_filter)` (line 520). Its docstring
(522–545) gives the algorithm[^mg-dg]:

1. build n→0 (amplitude) and n→1 (propagator) interaction dictionaries;
2. flip incoming particles to their antiparticles, so every leg is outgoing;
3. combine groups of legs that some interaction joins (`reduce_leglist`),
   replacing each group by the off-shell leg it produces (`merge_comb_legs`);
4. repeat until at most two legs remain, then close with a final vertex.

`DiagramTag` (line 46) is a canonical, hashable nested form of a diagram used
to remove duplicates: `__init__` (72), `diagram_from_tag` (132),
`vertices_from_link` (148), `leg_from_legs` (198; finds the output PDG by
removing the input legs' codes from the vertex's particle list) and
`vertex_from_link` (222)[^mg-tag]. In `base_objects.py`, `Particle` is at 202,
`get_helicity_states` at 469 and `is_fermion` (`spin % 2 == 0`) at 506.

vibegraph enumerates with FeynGraph instead ([FeynGraph](feyngraph.md),
[diagram enumeration](../../process/diagram-enumeration.md)); the algorithms
are compared there.

## HELAS (`HELAS/`)

`HELAS/` holds 153 Fortran files (`.F`, a few `.f`), one subroutine per file,
from the hand-written library the [HELAS paper](../papers/helas.md) documents,
plus later additions (multi-gluon, tensor and spin-3/2 routines such as
`jgggxx`, `jvtxxx`, `jiogld`). Generated MadGraph code calls ALOHA routines
instead; `HELAS/` is the reference for what the classic routines
compute[^mg-helas]. The core set:

| Kind | Routines (file = lowercase name + `.F`) |
|---|---|
| External wavefunctions | `IXXXXX`, `OXXXXX`, `VXXXXX`, `SXXXXX` |
| Amplitudes | `IOVXXX` (FFV), `IOSXXX` (FFS), `VVVXXX`, `VVSXXX`, `VSSXXX`, `SSSXXX`, `WWWWXX`, `W3W3XX` |
| Off-shell fermion | `FVIXXX`, `FVOXXX`, `FSIXXX`, `FSOXXX` |
| Off-shell vector | `JIOXXX`, `J3XXXX` (Z/γ combined), `JVVXXX`, `JVSXXX`, `JSSXXX`, `JWWWXX`, `JW3WXX` |
| Off-shell scalar | `HIOXXX`, `HVVXXX`, `HVSXXX`, `HSSXXX` |
| Collinear e–γ | `EAIXXX`, `EAOXXX`, `JEEXXX` |
| Kinematics | `MOMNTX`, `MOM2CX`, `BOOSTX`, `ROTXXX` |

The SM coupling routines `COUP1X`–`COUP4X` that the paper describes are not in
this directory at the pin.

## LO kT clustering

The default dynamical scale (`dynamical_scale_choice = -1`) comes from a kT
clustering of the event in the MadEvent template. `set_ren_scale` and
`set_fac_scale` return zero for that choice, and `setclscales` fills the scale
and `q2fact(1:2)` from the clustering[^n28-k1]. The files:

| File (under `Template/LO/`) | What |
|---|---|
| `SubProcesses/setscales.f` | `set_ren_scale` (1), `set_fac_scale` (98) |
| `SubProcesses/cluster.f` | `findmt` (436), `cluster` (518), the merge bookkeeping |
| `SubProcesses/reweight.f` | `ipartupdate` (224), `setclscales` (555), `rewgt` (1333) |
| `Source/kin_functions.f` | the kT measures: `DJ` (230), `PYDJ` (313), `DJ1` (344), `DJB` (401), `DJ2` (529) |
| `madgraph/iolibs/export_v4.py` | writes the per-process clustering tables |

The behaviour is specified in [the kT clustering algorithm](../../scales-pdf/kt-clustering-algorithm.md)
and [setclscales](../../scales-pdf/setclscales.md).

[^mg-version]: `VERSION` at `b7687064`.
[^n02-mg]: Note 02, MadGraph survey; every line number re-checked at `b7687064`.
[^mg-import]: `models/import_ufo.py` at `b7687064`.
[^mg-iface]: `madgraph/interface/madgraph_interface.py` at `b7687064`.
[^mg-multi]: `input/multiparticles_default.txt` at `b7687064`.
[^n06-flow]: Note 06 §6, the Python objects created.
[^mg-dg]: `madgraph/core/diagram_generation.py` at `b7687064`.
[^mg-tag]: `DiagramTag` in `diagram_generation.py` at `b7687064`.
[^mg-helas]: `HELAS/` at `b7687064`.
[^mg-sm]: `models/sm/` at `b7687064`; `particles.py` counted there.
[^n28-k1]: Note 28, "MadGraph kT clustering for dynamical_scale_choice = -1", read at the pin; subroutine lines re-checked at `b7687064`.
