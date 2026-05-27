# Plan: Fix diagram-count discrepancies vs MadGraph

Two independent problems. Tackle in order — problem 1 eliminates the dominant
discrepancy (extra Goldstone/Higgs diagrams); problem 2 eliminates the
subprocess-expansion overcounting.

---

## Problem 1: Restrict-card / zero-coupling vertex pruning

### Background

`restrict_default.dat` is a SLHA param card where zero means "prune any vertex
whose coupling vanishes under these parameters". For the SM:
- `yme=0`, `ymm=0`, `ymc=0` → no H or G0 diagrams for e/μ/c
- All Wolfenstein parameters = 0 → diagonal CKM, no off-diagonal weak vertices

`restrict_default.dat` in the UFO directory (and `restrict_<name>.dat` for
variants like `sm-no_b_mass`) plays the role of MadGraph's "state 1" —
parameters used for diagram enumeration only. `param_card.dat` is "state 2"
(physical amplitude computation) and user-supplied cards are "state 3".

### 1.1 Change `UFOModel::load` signature

```rust
pub fn load(path: &Path, restrict_card: Option<&Path>) -> Result<Self, UfoError>
```

Auto-discovery logic (only when `restrict_card.is_none()`):
1. Look for `restrict_default.dat` in `path/`.
2. If found, parse it with `ParamCard::from_file()` and use it.
3. If not found, leave all vertices as-is (no restrict).

All existing call sites pass `None` (or are updated to pass `None`) initially.
The `EvaluatedModel`'s param_card (state 2) is unchanged — it comes from a
separate `param_card.dat` loaded by the caller.

### 1.2 Filter vibegraph vertices during `load()`

After building `params`, `couplings`, and `raw_vertices` (before the
`TopoModel::from_ufo()` call):

1. If a restrict card was loaded, call `evaluate_under_restrict_card()` — a
   helper that runs the same `params.evaluate(card)` path as `UFOModel::evaluate`
   but using the restrict card — to produce a `HashMap<CouplingId, Complex64>`.
2. For each raw vertex, resolve its coupling ids, then check: if **all**
   couplings for that vertex evaluate to zero, mark it for removal.
   - Use a small threshold (e.g. `< 1e-20` absolute value) rather than `== 0.0`
     to handle floating-point restrict cards.
3. Remove marked vertices from vibegraph's own `vertices: IndexMap`.

### 1.3 Build feyngraph `TopoModel` without zero-coupling vertices

**Constraint**: `InteractionVertex::spin_map` is `pub(crate)` in feyngraph, so
we cannot read it out of an already-built model. Rebuilding the model by copying
existing vertices is therefore blocked.

**Chosen approach (minimal, no feyngraph fork)**: replace `TopoModel::from_ufo(path)` 
entirely. Build the feyngraph `Model` from vibegraph's already-parsed data using
feyngraph's mutation API:

```
Model::empty()
  .add_particle(name, anti_name, spin, color, pdg, texname, antitexname, linestyle, statistic)
  ... for every particle in `particles`
  .add_vertex(name, [particle_names], spin_map, coupling_orders)
  ... for every non-zero vertex
```

The missing piece is `spin_map`. Feyngraph derives this by parsing each
`lorentz.py` structure string (e.g. `"Gamma(1,2,3)*ProjM(4,5)"`) and extracting
the `(leg_i, leg_j)` spin-connection pairs; the result is a `Vec<isize>` of
length `n_legs` where entry `i` holds the index of the other leg that leg `i`
is spin-contracted with (or `-1` for un-contracted scalars).

Since vibegraph already parses `lorentz.py` into a symbolic AST
(`LorentzStructure`), we can add a `compute_spin_map(structure: &LorentzExpr, n_legs: usize) -> Vec<isize>` 
function that walks the AST and extracts spinor-index contractions. The Lorentz
operators that produce spin-connections are `Gamma`, `ProjP`, `ProjM`, `C`
(charge conjugation), and `Metric` on spinor indices. This is a targeted addition
to `vibegraph-lib/src/ufo/lorentz/` — not a full ALOHA codegen.

The `linestyle` and `statistic` arguments to `add_particle` can be derived from
the vibegraph `Particle` fields: `spin` (2s+1), `color`, `ghost_number`
(determine Fermi/Bose statistic), and whether the particle is a vector/fermion/scalar.

**Fallback if spin_map computation is deferred**: keep `TopoModel::from_ufo(path)`,
then post-filter diagrams via the `DiagramSelector::add_custom_function` hook.
The custom function receives a `&DiagramView`; reject any diagram where a vertex's
`particles_iter()` matches the particle list of a zero-coupling vibegraph vertex.
This works because vertex particle sets are accessible (public API), independent
of internal vertex name changes from feyngraph's splitting. This is lower risk
and should be implemented first; the spin_map / full replacement follows as a
separate task.

**The benefit of full replacement** (long-term): eliminates the remaining use of
feyngraph's UFO parser, which is known to fail on `loop_sm` and other non-standard
UFOs. After removal, vibegraph owns the entire parsing pipeline.

### 1.4 Side-task: parse `import model` in proc_card ✅ Done

`ParsedProcCard` and `ModelImport` are implemented. Parsing rule: `import model sm-no_b_mass`
splits on the first `-` → name `"sm"`, restrict variant `"no_b_mass"`.

`validate_madgraph_diagrams.rs` already uses `parse_proc_card` directly on `.mg5` script content.

A `vibegraph_lib::config::GlobalConfig` module (to wire `ParsedProcCard` → `UFOModel` loading
for the CLI) is designed but not yet implemented — tracked in TODO.md as `global-config`.

---

## Problem 2: Subprocess deduplication

### Background

Vibegraph's alias expansion generates a Cartesian product of concrete processes
(e.g. 81 for `p p > l+ l-`). Many share identical diagram topologies. MadGraph
counts representative subprocesses only — `P1_qq_ll` covers u/c/d/s initial
states with the same topology.

Two sub-problems:
1. **Mirror processes**: `u u~ > l+ l-` and `u~ u > l+ l-` are distinct expansions
   but identical diagrams (just initial-state ordering swapped). These come from
   the Cartesian product of `p × p` initial state with both orderings present.
2. **Flavor-equivalent subprocesses**: `u u~ > l+ l-` and `c c~ > l+ l-` (both
   up-type quarks, same interactions) generate the same diagram topologies.

### 2.1 Mirror process check

**Action**: add a test or assertion to verify whether feyngraph itself deduplicates
mirrored processes (e.g. inputs `["u", "u~"]` vs `["u~", "u"]`). If it does not,
then `expand_process` produces both orderings and each call to `generate_diagrams`
returns the same set — a factor-of-2 overcount for all asymmetric pairs.

**Fix if needed**: in `generate_from_process_spec`, before calling feyngraph,
deduplicate `ConcreteProcess`es where the initial-state particles are a permutation
of another. This is safe: for cross-section purposes, the PDF convolution over
`(x1, x2)` is symmetric (integrate both orderings with the same integrand), so
keeping one representative with a multiplicity factor is correct.

### 2.2 Topology-fingerprint deduplication

After `generate_from_process_spec` returns `Vec<DiagramSet>`, group sets by their
**topology fingerprint**: for each diagram in the set, sort the propagator PDG
codes; then the fingerprint is the sorted list of per-diagram propagator-PDG tuples.

Two `DiagramSet`s with the same fingerprint represent equivalent subprocesses.
Keep one representative per group and record its multiplicity.

**`DiagramSet` change**:
```rust
pub struct DiagramSet {
    pub particles_in:  Vec<String>,
    pub particles_out: Vec<String>,
    pub diagrams:      DiagramContainer,
    pub multiplicity:  u32,   // NEW, default 1
}
```

The validation test in `validate_madgraph_diagrams.rs` should then compare
*unique topology count* (sum of `diagrams.len()` over one representative per
fingerprint group) rather than total diagrams.

### 2.3 Future work note (not implementing now)

For cross-section calculation with incoming protons, proper subprocess grouping
must account for separate PDF weights per subprocess — `u u~ → l+ l-` and
`c c~ → l+ l-` use different PDFs even though their |M|² is identical. The
`multiplicity` field is insufficient for this case; a full list of equivalent
initial states with their own PDF queries is needed. This is tracked in TODO.md
as a future performance item.

---

## Implementation order

1. **Problem 1, step 1.2–1.3 (fallback path)**: Load restrict card; filter
   vibegraph vertex list; use DiagramSelector custom function to exclude
   zero-coupling diagrams from feyngraph output. Run validation tests.
   ✅ Done (restrict card + zero-coupling filter via custom_function).

2. **Problem 1, step 1.4**: Add `import model` parsing; remove
   `extract_process_from_mg5` from the validation test.
   ✅ Done.

3. **WEIGHTED coupling order filter** ✅ Done:
   - Parse `coupling_orders.py` into `UFOModel::order_hierarchy: HashMap<String, u32>`.
   - When `ProcessSpec::coupling_constraints` is empty, `generate_from_process_spec` iterates
     WEIGHTED starting at `(n_ext-2)*min_hierarchy` and stops at the first level that produces
     any diagrams (mirrors MadGraph's `find_optimal_process_orders`).
   - Filter applied via `DiagramSelector::add_custom_function` using `DiagramView::order()`.
   - Result: `pp > b b~` now gives 4 unique topologies (was 6) matching MadGraph reference.

4. **Problem 2, step 2.1**: Mirror process check; fix if double-counting.
   ✅ Done — `seen_initials` HashSet in `generate_from_process_spec`.

5. **Problem 2, step 2.2**: Topology-fingerprint dedup; update validation test comparisons.
   ⏳ Partially done — fingerprinting by propagator PDG codes; flavor-blind grouping not yet
   implemented.  Tests pass by coincidence for `pp > l+l-j`.

6. **Problem 1, full model replacement (deferred)**: Implement spin_map derivation
   from `LorentzExpr`, build `TopoModel` from vibegraph data directly, remove
   `TopoModel::from_ufo()`. Track as `feyngraph-ufo-replace` in TODO.md.
