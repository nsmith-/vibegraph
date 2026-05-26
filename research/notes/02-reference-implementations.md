# Reference Implementation Analysis

**Status:** Reference material — no action items. Analysis of reference submodules in `research/refs/`.
Each section notes the exact git revision examined and highlights relevance to vibegraph's pipeline goals.

---

## FeynGraph (`research/refs/feyngraph`)

**Revision:** `1dc4ea7` (2025-05-18, `main`)
**License:** MPL-2.0
**Version:** 0.1.0-beta.5 (Rust edition 2024)

### Source Layout

| File | Description |
|---|---|
| `src/lib.rs` | Public API: re-exports `DiagramGenerator`, `DiagramSelector`, `Model`; top-level `generate_diagrams()` |
| `src/model/mod.rs` | `Particle`, `InteractionVertex`, `Model`, `LineStyle`, `Statistic`, `TopologyModel` structs |
| `src/model/ufo_parser.rs` | PEG-based UFO 2.0 parser; SM embedded at compile time via `include_str!()` |
| `src/model/qgraf_parser.rs` | QGRAF format parser (alternative input) |
| `src/diagram/mod.rs` | `Diagram`, `DiagramContainer`, `DiagramGenerator`, `Leg`, `Propagator`, `Vertex` |
| `src/diagram/workspace.rs` | `AssignWorkspace` — recursive particle-assignment backtracking algorithm |
| `src/diagram/components.rs` | `AssignVertex`, `AssignPropagator`, `VertexClassification` helpers |
| `src/diagram/filter.rs` | `DiagramSelector` — post-generation filtering criteria |
| `src/diagram/view.rs` | `DiagramView` — read-only public interface for diagram inspection |
| `src/topology/mod.rs` | `Topology`, `TopologyGenerator`, `Node`, `Edge`, `TopologyContainer`; Tarjan momentum DFS |
| `src/topology/workspace.rs` | `TopologyWorkspace` — adjacency-matrix backtracking topology enumerator |
| `src/topology/components.rs` | `TopologyNode`, `NodeClassification` |
| `src/topology/matrix.rs` | `SymmetricMatrix` — adjacency matrix with self-loop support |
| `src/topology/filter.rs` | `TopologySelector` |
| `src/drawing/` | Diagram visualisation (not relevant to amplitude calculation) |
| `src/bindings/` | PyO3 Python bindings and Wolfram bindings |
| `src/util.rs` | Partition finding, permutation generation, index contraction utilities |

### Key Dependencies (Cargo.toml)

- `peg = "0.8"` — PEG parser for UFO format
- `rayon = "1"` — parallel topology iteration
- `indexmap = "2"` — insertion-order-preserving maps (important for reproducible particle ordering)
- `rustc-hash = "2"` — fast FxHashMap
- `itertools = "0.14"`, `either = "1"`, `thiserror = "2"`

---

### Goal 1: UFO Model Loading

**This is the most important area for vibegraph to study.** FeynGraph has a complete UFO 2.0 parser in Rust that we can use as a direct template.

#### Entry points

| Item | File | Line |
|---|---|---|
| `Model::from_ufo(path: &Path)` | `src/model/mod.rs` | 530 |
| `Model::from_qgraf(path: &Path)` | `src/model/mod.rs` | 544 |
| `parse_ufo_model(path: &Path) -> Result<Model, ModelError>` | `src/model/ufo_parser.rs` | 587 |

#### Embedded Standard Model

The crate ships the SM UFO inline (for tests and `Model::default()`):

```rust
// src/model/ufo_parser.rs lines 12-16
const SM_PARTICLES:        &str = include_str!("../../tests/resources/Standard_Model_UFO/particles.py");
const SM_COUPLING_ORDERS:  &str = include_str!("../../tests/resources/Standard_Model_UFO/coupling_orders.py");
const SM_COUPLINGS:        &str = include_str!("../../tests/resources/Standard_Model_UFO/couplings.py");
const SM_LORENTZ:          &str = include_str!("../../tests/resources/Standard_Model_UFO/lorentz.py");
const SM_VERTICES:         &str = include_str!("../../tests/resources/Standard_Model_UFO/vertices.py");
```

#### Intermediate `Value<'a>` enum (ufo_parser.rs lines 18–29)

The PEG grammar produces `Value<'a>` tokens before converting to domain structs:

```rust
enum Value<'a> {
    Int(isize),
    Rational(isize, isize),
    String(&'a str),
    Bool(bool),
    List(Vec<Value<'a>>),
    SIDict(HashMap<String, usize>),      // coupling-order maps
    CODict(HashMap<(usize, usize), String>),  // vertex coupling maps
    Particle(Particle),
    None,
}
```

#### PEG grammar rules (ufo_parser.rs)

| Rule | Line | Produces |
|---|---|---|
| `particle()` | 157 | `Particle` struct (name, antiname, pdg_code, spin, color, texname, linestyle) |
| `anti_particle()` | 233 | antiparticle from `.anti()` call |
| `coupling_order()` | 236 | coupling name string |
| `coupling()` | 253 | `(String, HashMap<String, usize>)` |
| `lorentz()` | 310 | `(&str, Vec<isize>)` — structure string + spin connections |
| `vertex()` | 279 | `Vec<InteractionVertex>` (may split one UFO vertex into several) |
| `parse_vertex()` | 424 | handles multi-Lorentz / multi-coupling-order vertex splitting (lines 506–582) |

The UFO data flow from file to `Model`:
1. Read `particles.py` → parse particles + build identifier mapping
2. Read `coupling_orders.py` → coupling name list
3. Read `couplings.py` → coupling powers per order
4. Read `lorentz.py` → spin flow structures
5. Read `vertices.py` → interaction rules; split if multiple Lorentz / coupling combinations
6. `Model::new()` (line 307) normalises: builds `anti_map`, `particle_counts` per vertex

#### Produced domain structs (model/mod.rs)

**`Particle`** (lines 49–64):
```rust
pub struct Particle {
    pub name: String, pub anti_name: String,
    pub spin: isize,        // 2s
    pub color: isize,
    pub pdg_code: isize,
    pub texname: String, pub antitexname: String,
    pub linestyle: LineStyle,
    pub self_anti: bool,
    pub statistic: Statistic,   // Fermi | Bose
}
```

**`InteractionVertex`** (lines 151–161):
```rust
pub struct InteractionVertex {
    pub name: String,
    pub particles: Vec<String>,                    // particle names
    pub spin_map: Vec<isize>,                      // spin-flow connections between legs
    pub coupling_orders: HashMap<String, usize>,   // QCD/QED power
    pub particle_counts: HashMap<usize, usize>,    // filled at Model::new()
}
```

**`Model`** (lines 287–343):
```rust
pub struct Model {
    particles: IndexMap<String, Particle>,
    vertices:  IndexMap<String, InteractionVertex>,
    couplings: Vec<String>,
    splittings: HashMap<String, HashMap<String, Vec<(usize, usize)>>>,
    anti_map: Vec<usize>,
}
```

**Relevance for vibegraph:** The UFO parser is essentially what we need to write. We can use FeynGraph's PEG grammar as the specification and copy/adapt the `Value` enum and rule structure directly. Note the `spin_map` field on `InteractionVertex` — this encodes which legs are spin-connected (Lorentz structure), which is needed for ALOHA-style wavefunction contraction.

---

### Goal 2: Feynman Diagram Enumeration

FeynGraph uses a clean two-stage algorithm: **topology generation** then **particle assignment**.

#### Stage 1 — Topology generation

| Item | File | Line |
|---|---|---|
| `TopologyGenerator::generate()` | `src/topology/mod.rs` | 690 |
| `TopologyWorkspace` (recursive adjacency-matrix builder) | `src/topology/workspace.rs` | 13 |
| Validity formula: Σ(k−2)·N_k = 2L − 2 + E | `src/topology/mod.rs` | 693–699 |

The formula for valid partitions of node degrees ensures the correct edge count. Then `TopologyWorkspace` fills an adjacency matrix by DFS backtracking.

Momentum assignment uses a modified Tarjan bridge-finding DFS (lines 131–188 in `src/topology/mod.rs`):
- Identifies bridge edges (1PI separators)
- Assigns loop momenta to back-edges
- External momenta assigned; last eliminated by conservation

**`Topology`** (src/topology/mod.rs lines 74–86):
```rust
pub struct Topology {
    pub n_external: usize, pub n_loops: usize,
    pub nodes: Vec<Node>,  pub edges: Vec<Edge>,
    pub node_symmetry: usize, pub edge_symmetry: usize,
    pub momentum_labels: Vec<String>,
    pub bridges: Vec<(usize, usize)>,
    pub node_classification: NodeClassification,
}
```

Edges store momenta as `Vec<i8>` — a linear combination of external and loop momentum basis vectors.

#### Stage 2 — Particle assignment (backtracking)

| Item | File | Line |
|---|---|---|
| `DiagramGenerator::new()` | `src/diagram/mod.rs` | 446 |
| `DiagramGenerator::generate() -> DiagramContainer` | `src/diagram/mod.rs` | 497 |
| `AssignWorkspace::assign()` | `src/diagram/workspace.rs` | ~50 |
| `select_vertex()` (pick next vertex to fill) | `src/diagram/workspace.rs` | 148 |
| `select_leg()` (iterate particle choices for one leg) | `src/diagram/workspace.rs` | 199 |

The parallel loop (rayon, line 511) processes topologies concurrently.

**`Diagram`** (src/diagram/mod.rs lines 84–106):
```rust
pub struct Diagram {
    pub incoming_legs: Vec<Leg>,
    pub outgoing_legs: Vec<Leg>,
    pub propagators: Vec<Propagator>,
    pub vertices: Vec<Vertex>,
    pub vertex_symmetry: usize,
    pub propagator_symmetry: usize,
    pub bridges: Vec<usize>,
    pub sign: i8,            // ±1 from fermion loop ordering
}
```

**`Leg`** (lines 24–43):
```rust
pub struct Leg {
    pub vertex: usize,              // index of the internal vertex this leg connects to
    pub particle: usize,            // particle type index in Model
    pub momentum: Vec<i8>,          // linear combination of basis momenta
}
```

**`Propagator`** (lines 45–64):
```rust
pub struct Propagator {
    pub vertices: [usize; 2],
    pub particle: usize,
    pub momentum: Vec<i8>,
}
```

**`Vertex`** (lines 66–82):
```rust
pub struct Vertex {
    pub propagators: Vec<isize>,    // −1 for external legs
    pub interaction: usize,         // index into Model::vertices
}
```

#### Public API

```rust
// Top-level convenience (lib.rs line 28)
pub fn generate_diagrams(
    particles_in:  &[&str],
    particles_out: &[&str],
    n_loops:       usize,
    model:         Model,
    selector:      DiagramSelector,
) -> Result<DiagramContainer, ModelError>

// DiagramContainer (diagram/mod.rs)
container.len()
container.get(i) -> DiagramView
container.views()  // iterator
container.query(selector)

// DiagramView (diagram/view.rs)
view.incoming()           // external incoming legs
view.outgoing()           // external outgoing legs  
view.propagators()        // internal lines
view.vertices()           // internal vertices
view.sign()               // ±1 fermion sign
view.symmetry_factor()
```

**Relevance for vibegraph:** We can either use FeynGraph as a library dependency for diagram enumeration (keeping our own Rust code for amplitudes), or study its algorithm and port the relevant subset. The `sign` field on `Diagram` is critical — it tracks the ±1 from fermion loop permutations that must multiply the amplitude.

#### DiagramSelector filters (filter.rs lines 24–47)

Post-generation filters reduce the diagram set:
- `select_opi_components(n)` — e.g. keep only connected diagrams (n=1)
- `select_self_loops(0)` — discard tadpoles
- `select_coupling_power(coupling, power)` — enforce LO in QCD/QED
- Custom closure-based filters

---

### Goal 3: HELAS Amplitude Construction

FeynGraph does **not** implement helicity amplitudes — it stops at diagram enumeration. The `Diagram` output (particle labels + momentum routing) is the input to a separate HELAS evaluation step.

For vibegraph, the bridge between FeynGraph output and our HELAS module will be:
1. Iterate `container.views()`
2. For each diagram: read leg particles + vertex types → call appropriate HELAS routines in topological order
3. Sum over diagrams; square; sum over helicities

---

### Goal 4: Phase Space Sampling and Goal 5: Cross Section

Not addressed by FeynGraph. Irrelevant to this crate.

---

---

## MadGraph5_aMC@NLO (`research/refs/mg5amcnlo`)

**Revision:** `b768706` (tag `v3.7.1`, branch `3.x`)
**Language:** Python 3 + Fortran77

---

### Goal 1: UFO Model Loading

#### ALOHA UFO expression parser (aloha/aloha_parsers.py)

| Item | Line |
|---|---|
| `class UFOExpressionParser` | 45 |
| `parse(buf)` method | 60 |
| Token definitions: `POWER`, `SQRT`, `CONJ`, `RE`, `IM`, `PI`, `COMPLEX`, `FUNCTION`, `VARIABLE`, `NUMBER` | 66–70 |

The parser uses PLY Lex/Yacc to convert UFO Lorentz structure strings (e.g. `"Gamma(3,2,1)"`) into evaluable expression trees. This is the Python analogue of FeynGraph's PEG `lorentz()` rule.

#### SM UFO model files (models/sm/)

| File | Purpose |
|---|---|
| `particles.py` | 11 particle constructors generated by FeynRules 1.7.69 |
| `vertices.py` | All SM interaction vertices |
| `lorentz.py` | Lorentz tensor structures (strings like `'Gamma(3,2,1)'`, `'ProjM(2,1)'`) |
| `parameters.py` | External (SLHA input) and internal (computed) parameters |
| `couplings.py` | Symbolic coupling expressions |
| `coupling_orders.py` | QCD/QED coupling order metadata |
| `object_library.py` | UFO base classes (`Particle`, `Vertex`, `Lorentz`, `Parameter`, `Coupling`) |
| `function_library.py` | Custom functions (e.g. `complexconjugate`, `re`, `im`) |
| `decays.py` | Partial width declarations |
| `build_restrict.py` | Restriction file builder |

**First particles in particles.py** (approximate lines):

| Line | Particle | PDG | spin | color |
|---|---|---|---|---|
| ~10 | `a` (photon) | 22 | 3 | 1 |
| ~24 | `Z` | 23 | 3 | 1 |
| ~38 | `W__plus__` | 24 | 3 | 1 |
| ~52 | `W__minus__` | −24 | 3 | 1 (via `.anti()`) |
| ~54 | `g` (gluon) | 21 | 3 | 8 |

**Vertex pattern** (vertices.py): each vertex is `V_n = Vertex(name, particles, color, lorentz, couplings)` where `couplings` maps `(lorentz_index, color_index)` → coupling object.

**Lorentz structures** (lorentz.py): `Lorentz(name, spins, structure)` where `structure` is a string parsed by ALOHA. The `spins` list encodes spin (1=scalar, 2=fermion, 3=vector, −1=ghost).

**Parameter classes** (parameters.py):
- `nature='external'`: read from param card (has `lhablock` + `lhacode`)
- `nature='internal'`: evaluated from algebraic `value` expression

---

### Goal 2: Feynman Diagram Enumeration (madgraph/core/)

#### Core files

| File | Description |
|---|---|
| `base_objects.py` | `Particle`, `Interaction`, `Model`, `Process`, `Diagram`, `Leg`, `Vertex` |
| `diagram_generation.py` | `Amplitude`, `DiagramTag`; the generation engine |
| `helas_objects.py` | HELAS-specific matrix element objects |
| `color_algebra.py` | Color flow algebra |
| `drawing.py` | Diagram visualisation |

#### Diagram generation algorithm

**Class `Amplitude`** (diagram_generation.py line 433):

| Method | Line | Purpose |
|---|---|---|
| `generate_diagrams(returndiag, diagram_filter)` | 520 | Core recursive algorithm |
| `get('diagrams')` | 478 | Lazy accessor (triggers generation on first call) |
| `get_number_of_diagrams()` | 492 | Count |
| `get_ninitial()` | 511 | Number of initial-state particles |

The algorithm (docstring lines 522–545):
1. Build interaction dictionaries: n→0 (amplitude vertices) and n→1 (propagator vertices)
2. Mark external particles; flip incoming particles to their antiparticles
3. Iteratively combine particle groups using interaction dictionary (`reduce_leglist`)
4. After each combination step, replace group with new off-shell leg (`merge_comb_legs`)
5. Repeat until ≤2 particles remain (final vertex)
6. All vertices stored with outgoing convention; incoming legs flipped back at use

**Class `DiagramTag`** (diagram_generation.py line 46) — compact hashable representation for duplicate elimination:

| Method | Line | Purpose |
|---|---|---|
| `__init__(diagram, model, ninitial)` | 72 | Build tag from diagram |
| `diagram_from_tag(model)` | 132 | Reconstruct full `Diagram` from tag |
| `vertices_from_link(link, model, first_vertex)` | 148 | Recursive vertex reconstruction |
| `leg_from_legs(legs, vertex_id, model)` | 198 | Compute output PDG code from input legs by elimination |
| `vertex_from_link(legs, vertex_id, model)` | 222 | Create `Vertex` object |

Key leg-combination logic (lines 198–219): given legs entering a vertex and the interaction id, remove their PDG codes from the vertex's particle list to find the off-shell output leg.

**`Particle`** (base_objects.py):

| Method | Line | Purpose |
|---|---|---|
| class `Particle` | 202 | spin, color, mass, width, pdg_code, name |
| `is_fermion()` | 506 | `spin % 2 == 0` |
| `get_helicity_states()` | 469 | returns list of allowed NHEL values based on spin |

---

### Goal 3: HELAS Amplitude Construction

#### HELAS Fortran subroutines (HELAS/)

The directory contains 113 Fortran files. Subroutine definitions start at line 1 in each file.

**Wavefunction subroutines:**

| Subroutine | File | Computes |
|---|---|---|
| `IXXXXX` | `ixxxxx.F` | Flowing-in fermion spinor u(p) / v(p) |
| `OXXXXX` | `oxxxxx.F` | Flowing-out fermion u-bar(p) / v-bar(p) |
| `VXXXXX` | `vxxxxx.F` | Vector boson polarization ε(p) / ε*(p) |
| `SXXXXX` | `sxxxxx.F` | Scalar boson wavefunction |

**Vertex subroutines (amplitude variants):**

| Subroutine | File | Vertex type |
|---|---|---|
| `IOVXXX` | `iovxxx.F` | FFV → amplitude |
| `VVVXXX` | `vvvxxx.F` | VVV → amplitude |
| `IOSXXX` | `iosxxx.F` | FFS → amplitude |
| `VVSXXX` | `vvsxxx.F` | VVS → amplitude |
| `VSSXXX` | `vssxxx.F` | VSS → amplitude |
| `SSSXXX` | `sssxxx.F` | SSS → amplitude |
| `WWWWXX` | `wwwwxx.F` | VVVV → amplitude |
| `W3W3XX` | `w3w3xx.F` | VVVV (W/Z variant) → amplitude |

**Current subroutines (off-shell leg variants):**

| Subroutine | File | Output |
|---|---|---|
| `FVIXXX` | `fvixxx.F` | FV → off-shell flowing-in fermion |
| `FVOXXX` | `fvoxxx.F` | FV → off-shell flowing-out fermion |
| `JIOXXX` | `jioxxx.F` | FF → off-shell vector |
| `J3XXXX` | `j3xxxx.F` | FF → off-shell Z/γ (combined) |
| `JVVXXX` | `jvvxxx.F` | VV → off-shell vector |
| `HIOXXX` | `hioxxx.F` | FF → off-shell scalar |
| `JVSXXX` | `jvsxxx.F` | VS → off-shell vector |
| `HVVXXX` | `hvvxxx.F` | VV → off-shell scalar |
| `JSSXXX` | `jssxxx.F` | SS → off-shell vector |
| `HVSXXX` | `hvsxxx.F` | VS → off-shell scalar |
| `HSSXXX` | `hssxxx.F` | SS → off-shell scalar |
| `JWWWWX` | `jwwwwx.F` | VVV → off-shell vector |
| `JW3WXX` | `jw3wxx.F` | VVV → off-shell W/Z |

**Singular (collinear) vertex:**

| Subroutine | File | Purpose |
|---|---|---|
| `EAIXX` | `eaixxx.F` | Collinear e + γ → off-shell e (incoming) |
| `EAOXX` | `eaoxxx.F` | Collinear e + γ → off-shell e (outgoing) |
| `JEEXX` | `jeexx.F` | ee → off-shell γ (collinear) |

**Kinematics utilities:**

| Subroutine | File | Purpose |
|---|---|---|
| `MOMNTX` | `momntx.F` | 4-momentum from E, m, cos θ, φ |
| `MOM2CX` | `mom2cx.F` | Two-body CM momenta |
| `BOOSTX` | `boostx.F` | Lorentz boost |
| `ROTXXX` | `rotxxx.F` | 3-rotation |

**SM coupling subroutines:**

| Subroutine | File | Covers |
|---|---|---|
| `COUP1X` | `coup1x.F` | VVV, VVVV vertices |
| `COUP2X` | `coup2x.F` | FFV vertices |
| `COUP3X` | `coup3x.F` | VVS, SSS, VVSS, SSSS |
| `COUP4X` | `coup4x.F` | FFS (Yukawa) |

#### ALOHA code generator (aloha/)

ALOHA is the bridge between UFO Lorentz structures and HELAS-style Fortran routines. It auto-generates functions equivalent to the hand-coded HELAS subroutines above, but for any UFO model.

| Item | File | Line |
|---|---|---|
| `class AbstractRoutine` | `create_aloha.py` | 59 |
| `class AbstractRoutineBuilder` | `create_aloha.py` | 120 |
| `compute_routine(mode, tag, factorize)` | `create_aloha.py` | 159 |
| `write(output_dir, language, mode, combine, options)` | `create_aloha.py` | 91 |
| `class UFOExpressionParser` | `aloha_parsers.py` | 45 |
| `class WriteALOHA` (base code writer) | `aloha_writers.py` | 28 |
| Lorentz object `L_P` (momentum four-vector) | `aloha_object.py` | 40 |
| `class Computation` (expression tree) | `aloha_lib.py` | 63 |

**Code generation flow:**
1. `UFOExpressionParser.parse(structure_string)` — tokenises the Lorentz structure expression
2. `AbstractRoutineBuilder.compute_routine(mode)` — contracts indices, inserts propagator denominators
3. `AbstractRoutine` stores the resulting expression tree
4. `WriteALOHA.write(language='Fortran')` serialises to `.F` code

Supported output languages: Fortran, Python, C (set via `language` parameter to `write()`).

**Relevance for vibegraph:**

For the toy problem (e⁺e⁻ → μ⁺μ⁻) we need exactly three wavefunction routines (`IXXXXX`, `OXXXXX`, `VXXXXX`) and one vertex routine (`IOVXXX`). The Fortran sources in `HELAS/` are the ground truth for the computation these routines perform. The ALOHA code shows how to derive equivalent routines automatically from `FFV1 = Gamma(3,2,1)`.

---

### Goals 4 & 5: Phase Space and Cross Section

Not in MadGraph's core directory — the phase space generator (RAMBO or MadGraph's own parametrisation) lives in `Template/` and `madgraph/various/`. Not examined here.

---

## Cross-Cutting Notes

### UFO Parsing: FeynGraph vs vibegraph

FeynGraph's `ufo_parser.rs` is our clearest template. Key design decisions to carry over:

1. **Split multi-Lorentz vertices** — one UFO `Vertex` with multiple Lorentz structures becomes multiple `InteractionVertex` entries. The splitting map in `Model` (`splittings` field) tracks this. Essential for correct amplitude summation.
2. **`spin_map` on `InteractionVertex`** — encodes which legs are spin-contracted (the Lorentz structure connection). This drives ALOHA-style wavefunction index contraction.
3. **`anti_map`** — maps each particle index to its antiparticle index. Essential for fermion-number-flow tracking in diagrams.
4. **`IndexMap` for ordering** — particle and vertex ordering must be deterministic for reproducible tests.

### Diagram Enumeration: Using FeynGraph as a Dependency

We could add FeynGraph as a `[dependencies]` entry in vibegraph's `Cargo.toml`. The key interface would be:

```rust
let model = Model::from_ufo(Path::new("research/refs/mg5amcnlo/models/sm"))?;
let selector = DiagramSelector::new()
    .select_opi_components(1)   // connected only
    .select_self_loops(0);      // no tadpoles
let diagrams = generate_diagrams(
    &["e-", "e+"], &["mu-", "mu+"], 0, model, selector
)?;
```

Then iterate `diagrams.views()` to read particle labels and momentum routing into our HELAS evaluator.

### MadGraph Diagram Algorithm vs FeynGraph

| Aspect | MadGraph `Amplitude` | FeynGraph |
|---|---|---|
| Approach | Leg-combination (Dyson-Schwinger style) | Topology-first, then particle assignment |
| Language | Python | Rust |
| Duplicate removal | `DiagramTag` hashing | Symmetry factors at topology stage |
| Loop support | No (separate aMC@NLO) | Yes (n_loops parameter) |
| Parallel | No | Yes (rayon) |

For LO vibegraph the simpler MadGraph-style leg-combination is easier to implement from scratch if we don't use FeynGraph directly. FeynGraph's topology-first approach is better for multi-loop or high-multiplicity processes.
