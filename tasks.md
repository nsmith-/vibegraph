# vibegraph — Next-Step Tasks

This file tracks near-term research and implementation tasks.
Status is maintained in the session SQL database; update here at milestones.

---

## T1 · MadGraph Code Quality Review

**Goal:** Read `research/refs/mg5amcnlo/UpdateNotes.txt` and survey the MadGraph5 Python
codebase to identify categories of bugs that have historically appeared, with the aim of
informing what unit tests vibegraph needs to catch the same classes of errors.

**Output:** `research/notes/07-mg5-code-quality.md` *(renamed from 04- to avoid collision with main's `04-ufo-parsing-future.md`)*

**Approach:**
1. Fully read `UpdateNotes.txt` (2633 lines) — each entry describes a fix or new feature;
   bugs are labelled with their category (physics, crash, output format, etc.).
2. Survey high-churn Python modules: `madgraph/core/`, `aloha/`, `models/sm/`.
3. Classify findings into:
   - **Physics-correctness bugs** (wrong matrix element, sign errors, missing diagrams)
   - **Numerical precision / stability issues**
   - **Process-specification / model-loading bugs**
   - **Phase-space / integration bugs**
   - **Output / I/O bugs**
   - **Crashes / regressions**
4. For each category note: (a) how often it recurred, (b) what triggered it, (c) what a
   unit test to catch it would look like in vibegraph's Rust test suite.

**Dependencies:** None (read-only).

**Status: ✅ Done**

**Results:** `research/notes/04-mg5-code-quality.md` written (408 lines, 110+ bug entries
across 7 categories). Key findings:
- **Strengths:** Exact rational arithmetic for color algebra (no FP rounding in color-factor
  reduction); `DiagramTag` canonical deduplication (O(1) identity check); clean RAMBO
  implementation with correct massless weight formula.
- **Weaknesses:** Color-matrix bugs recur across 6+ major versions — no systematic algebraic
  test suite; helicity-recycling optimization introduced 7 distinct correctness bugs after
  introduction; RAMBO overflow-check has a sign error (`iwarn[4] > 5` should be `< 5`,
  still present in source); 30+ Python 2→3 migration bugs showing lack of version-agnostic
  coding discipline.
- Document includes 7 bug-category tables with version, description, trigger, and
  vibegraph test implications for each entry.

---

## T2 · Runnable MadGraph via pixi/conda

**Goal:** Set up a pixi environment that installs MadGraph5_aMC@NLO as a conda package and
can run a simple process (e.g. `e+ e- > mu+ mu-`) to produce a parton-level cross section
and/or LHE event file.  This will be the reference we validate vibegraph against.

**Context:**
- `pixi.toml` already has one custom environment (`nougat`); add a new feature/environment.
- MadGraph is available on conda-forge as `mg5amc` or similar — confirm exact package name.
- Running from `research/refs/mg5amcnlo` directly is blocked because MadGraph checks for
  a release tag; a conda-installed release is needed.
- Target: `pixi run -e madgraph generate-ee` produces a cross-section number we can record.

**Output:**
- New `[feature.madgraph]` section and `[environments]` entry in `pixi.toml`.
- A small shell script or pixi task `generate-ee` that runs the generation.
- Document the cross-section result (and any caveats) in `research/notes/05-madgraph-setup.md`.

**Note:** Both T2 and T4 modify `pixi.toml`. Coordinate with T4 so changes are merged cleanly;
prefer adding non-overlapping feature blocks so a textual merge is trivial.

**Dependencies:** None (independent of other tasks).

**Status: ✅ Done**

**Results:**
- `pixi.toml` updated with `[feature.madgraph]` block: `mg5amcnlo = "==3.5.7"` from
  conda-forge (native `osx-arm64`, Python 3.11); `generate-ee` pixi task; environment
  entry `madgraph = { features = ["madgraph"], solve-group = "madgraph" }`.
- `research/mg5_scripts/ee_to_mumu.mg5` — MadGraph batch script for `e+ e- > mu+ mu-`
  at √s = 91.2 GeV.
- `research/notes/05-madgraph-setup.md` — documents package name, setup steps, expected
  cross sections (~1.8 nb at Z-pole, ~6 pb at 200 GeV), and caveats.
- To install: `pixi install -e madgraph`; to run: `pixi run -e madgraph generate-ee`.

---

## T3 · arXiv Paper Scan of mg5amcnlo Source

**Goal:** Find every arXiv paper ID cited in the mg5amcnlo source tree, filter to those not
already in `research/refs/fetch-papers.sh`, add them to the script, fetch them, and add
summaries to `research/notes/01-paper-summaries.md`.

**Approach:**
1. `grep -ri arxiv research/refs/mg5amcnlo` — collect all matches.
2. Extract canonical arXiv IDs (patterns: `hep-ph/YYMMNNN`, `hep-th/YYMMNNN`,
   `YYMM.NNNNN`, `arXiv:YYMM.NNNNN`).
3. Deduplicate; cross-reference against existing entries in `fetch-papers.sh`.
4. For each new ID:
   a. Append an entry to the `PAPERS` array in `fetch-papers.sh`.
   b. Fetch HTML via `https://ar5iv.org/html/<id>`.
   c. Write a concise summary (what it does, relevance to vibegraph) in
      `research/notes/01-paper-summaries.md` following the existing format.

**Output:** Updated `fetch-papers.sh` and `research/notes/01-paper-summaries.md`.

**Dependencies:** None (read-only scan + file edits in non-overlapping files from T2/T4).

**Status: ✅ Done**

**Results:** 4 new papers added (50+ references scanned; majority skipped as NLO/phenomenology):

| Key | arXiv | Topic |
|---|---|---|
| `polarized_me` | 1912.01725 | Automated polarized/helicity ME in MadGraph5 — fermion/vector helicity conventions |
| `polarized_propagator` | 2512.10015 | Truncated propagator paradigm for polarized amplitudes — covariant/axial gauge |
| `loop_induced_ps` | 1507.00020 | Phase-space optimisation appendix (multi-channel decomposition, channel Jacobians) |
| `madwidth` | 1402.1178 | MadWidth — automatic decay widths; extends UFO with `decay.py` / `Decay` objects |

Skipped: NLO loop integral libraries, PDF sets, shower tunes, BSM phenomenology, etc.

---

## T4 · HELAS Validation Harness

**Goal:** Cross-validate vibegraph's Rust HELAS implementation against the original
Fortran77 HELAS routines in `research/refs/mg5amcnlo/HELAS/`.

**Approach:**
1. Create `validation/helas/` directory.
2. Copy (or symlink) the relevant HELAS `.F` files needed for `e+ e- → μ+ μ-`:
   - `ixxxxx.F` / `oxxxxx.F` (incoming/outgoing fermion wavefunctions)
   - `vxxxxx.F` (vector wavefunction, for the photon/Z)
   - `FFV1_0.F` (or equivalent vertex amplitude, possibly from ALOHA-generated code)
3. Add a new `[feature.helas-validation]` pixi environment with:
   `python >=3.10`, `numpy`, `gfortran` (for f2py).
4. Write a `validation/helas/build.sh` that uses f2py to compile HELAS into a Python extension.
5. Write `validation/helas/gen_reference.py` that:
   - Parameterises the `e+ e- → μ+ μ-` kinematics by (√s, cos θ) on a 20×20 grid.
   - Calls the Fortran HELAS routines via f2py to compute |M|² for each point.
   - Saves results as `reference.npz` (or `.npy`).
6. Write a Rust integration test `tests/helas_validation.rs` that:
   - Reads `reference.npz` (or a CSV companion).
   - Evaluates vibegraph's helicity amplitude over the same grid.
   - Asserts max relative deviation < 1e-6.

**Note:** T2 also modifies `pixi.toml`. Add a separate `[feature.helas-validation]` block
and a corresponding `[environments]` entry; the main session will merge both.

**Output:** `validation/helas/` directory with build script, Python reference generator,
and updated `pixi.toml` (feature block only; environment registration too).

**Dependencies:** None hard, but complements T1 (insights about which routines have bugs).

**Status: ✅ Done** *(updated 2026-05-22: extended-validation test gating added)*

**Results:**
- `validation/helas/` created with copies of `ixxxxx.F`, `oxxxxx.F`, `jioxxx.F`,
  `iovxxx.F`, `vxxxxx.F` (with `cf2py intent(in/out)` directives added for f2py).
- `validation/helas/build.sh` — compiles HELAS into `helas_f.so` via `python -m numpy.f2py`;
  includes `-fallow-argument-mismatch` for gfortran ≥ 10.
- `validation/helas/gen_reference.py` — iterates 20×20 grid over (√s, cos θ), calls HELAS
  routines for all 16 helicity combinations of `e+ e- → μ+ μ-`, saves `reference.npz` and
  `reference.csv`; cross-checks against analytic result `Σ|M|² = 4e⁴(1 + cos²θ)`.
- `validation/helas/README.md` — explains build/run steps and expected output.
- `src/lib.rs` — exposes `pub mod helas` with stub `compute_m2_ee_mumu` returning 0.0.
- `tests/helas_validation.rs` — integration test gated behind `#[cfg(feature = "extended-validation")]`;
  invisible to plain `cargo test`, opt-in via `cargo test --features extended-validation`.
- `Cargo.toml` — `[features] extended-validation = []` declared; also has explicit `[lib]`/`[[bin]]`.
- `pixi.toml` — `[feature.helas-validation]` (gfortran + numpy) with three tasks:
  - `build-helas` — compile f2py extension
  - `gen-reference` — generate `reference.npz`/`reference.csv` (depends-on: build-helas)
  - `validate-helas` — full pipeline ending in `cargo test --features extended-validation` (depends-on: gen-reference)
- One-shot command: `pixi run -e helas-validation validate-helas`

---

## T5 · MadGraph Process-Specification PEG Grammar

**Goal:** Locate where MadGraph parses its process specification language
(`generate p p > e+ e- j`, `add process ...`, particle/multiparticle definitions, etc.)
and distil it into a formal PEG grammar that vibegraph can use to steer feyngraph.

**Approach:**
1. Search `research/refs/mg5amcnlo/madgraph/` for the lexer/parser:
   - Look for `cmd.py`, `MadGraphCmd`, `generate`, `import model` handlers.
   - Identify tokenisation and grammar rules (likely `pyparsing`, hand-rolled recursive
     descent, or a PLY/ANTLR grammar).
2. Trace parsing of `generate <process>` → diagram generation call.
3. Collect all grammar productions (tokens, operators `>`, `@`, `/`, `$`, `^2` etc.).
4. Write a clean PEG grammar (`.peg` or inline in a research note) covering:
   - Process line: `<is> > <fs> [<restrictions>] [<options>]`
   - Multiparticle aliases: `define p = u d u~ d~ g`
   - Amplitude ordering: `[QCD]`, `[virt=QCD]`, decay chains `(p > ...)`
5. Note which parts of the grammar feyngraph currently supports and what gaps exist.

**Output:** `research/notes/06-process-grammar.md` with the PEG grammar and feyngraph gap analysis.

**Dependencies:** None (read-only analysis of mg5amcnlo Python source).

**Status: ✅ Done**

**Results:** `research/notes/06-process-grammar.md` written (529 lines). Key contents:
- **Parser location** — exact file + line numbers: `do_generate` at L4811,
  `do_add` at L3232, `extract_process` at L4822, `extract_decay_chain_process` at L5661,
  `check_process_format` at L1150 (in `madgraph/interface/madgraph_interface.py`).
- **Complete PEG grammar** in pest.rs-style notation covering:
  - Particle tokens: names, PDG codes, duplication prefix (`2e+`), polarization (`z{T}`)
  - Process body: `A > B` and `A > X > B` (required s-channel insertion)
  - All restrictions: `/` (forbidden), `$` (no on-shell s-channel), `$$` (no s-channel)
  - Coupling order constraints: `QCD=2`, `QED<=4`, `QCD^2==4`
  - Loop spec: `[QCD]`, `[virt=QCD]`
  - Process tag: `@N`
  - Decay chains: `t t~, (t > b w+)`
  - `define` multiparticle alias command
  - Each rule marked 🟢 Core (LO tree-level) vs 🔵 Extended (NLO/decay chains)
- **Data flow trace** — string → `ProcessDefinition` → `MultiProcess` → `AmplitudeList`
- **Worked example** — `e+ e- > mu+ mu-` traced token-by-token with PDG codes and objects
- **Feyngraph gap analysis** — notes submodule uninitialized; recommends a translation layer
  between the MadGraph syntax parser and feyngraph's diagram generator API

---

## Rebase onto main — Follow-up Notes

*Rebased onto `origin/main` (ae92da6) on 2026-05-22. The following commits were new on main
and have implications for the 5 tasks above.*

### What came in on main

| Commit | Summary |
|---|---|
| `9a22e58` | `feat(ufo)`: full UFO model loader — `src/ufo/` with parameter/coupling eval, 31 tests |
| `0491ea7` | `research`: stub note on future full UFO parsing ownership (`04-ufo-parsing-future.md`) |
| `01784e9` | `feat(helas)`: full HELAS implementation for `e+ e- → μ+ μ-` in `src/helas/` |
| `8659cd3` | `style`: cargo fmt across helas + ufo modules |
| `c6fde0e` | `style`: cargo fmt |
| `ae92da6` | `docs`: ALOHA gap analysis added to `04-ufo-parsing-future.md` and `src/ufo/mod.rs` |

### Follow-up per task

**T1 — Code quality review** (`research/notes/07-mg5-code-quality.md`)
- ⚠️ **Note numbering collision fixed**: renamed from `04-` to `07-mg5-code-quality.md`
  since main added `04-ufo-parsing-future.md`.
- ✅ **Follow-up done (2026-05-22):** Cross-referenced T1's physics-correctness and
  numerical stability categories against the actual `repr.rs`/`vertex.rs` implementation.
  Added 5 new `#[test]` cases to `src/helas/mod.rs` (commit `4b961e3`):
  - `test_ee_to_mumu_multi_angle` — validates `Σ|M|² = 4e⁴(1+cos²θ)` at 7 angles
    with physical coupling `e = √(4πα)`; directly covers T1's "range of kinematics" gap
  - `test_ward_identity` — replaces ε^μ with q^μ, asserts amplitude vanishes; catches
    sign/normalisation errors (T1 v1.4.3 ALOHA bug class)
  - `test_backward_direction_massless` — exercises the `sqp0p3 = 0` branch in
    `weyl_ixxxxx`/`weyl_oxxxxx`; guards collinear-limit divergences
  - `test_massive_wavefunction_moving` — checks `fi†fi = 2E` for massive moving fermion
  - `test_massive_wavefunction_at_rest` — checks `fi†fi = 2m` for the pp=0 at-rest branch
- **Remaining:** Helicity-recycling tests (not yet implemented; relevant if/when recycling
  is added); color-matrix tests (for diagram generation phase).

**T2 — Runnable MadGraph** (`research/notes/05-madgraph-setup.md`)
- Main now has a fully working UFO loader and HELAS implementation. The MadGraph
  validation is therefore more important — we now have a Rust side to compare against.
- ✅ **Follow-up done (2026-05-22):** `pixi install -e madgraph` and
  `pixi run -e madgraph generate-ee` both succeeded on `osx-arm64`.
  Actual cross-section measured: **σ = 2025 ± 1 pb** (= 2.025 ± 0.001 nb) at √s = 91.2 GeV.
  10 000 unweighted LHE events written to `ee_to_mumu/Events/run_01/unweighted_events.lhe.gz`.
  `05-madgraph-setup.md` updated with the actual measured value and LHE file location.

**T3 — arXiv paper scan** (`research/notes/01-paper-summaries.md`)
- `04-ufo-parsing-future.md` and `src/ufo/mod.rs` explicitly document three FeynGraph
  gaps needed for ALOHA: Lorentz structure expressions, coupling values, color structures.
  These gaps reference ALOHA (already fetched) and implicitly the original MadGraph
  ALOHA paper. No new arXiv IDs were introduced by the main commits.
- **Follow-up:**
  1. ✅ **Done (2026-05-22):** Ran `./research/refs/fetch-papers.sh polarized_me
     polarized_propagator loop_induced_ps madwidth`. All four fetched successfully with
     valid HTML content (sizes: polarized_me 2.5 MB, polarized_propagator 1.5 MB,
     loop_induced_ps 1.1 MB, madwidth 1.0 MB — well above the ~8KB error-page threshold).
  2. When ALOHA implementation begins, the `lorentz.py` parsing problem may surface
     papers on tensor-product/Lorentz algebra code generation worth adding.

**T4 — HELAS validation harness** (`validation/helas/`)
- 🔴 **Major change:** Main landed a full, working HELAS Rust implementation in
  `src/helas/` (`repr.rs`, `wavefn.rs`, `vertex.rs`, `mod.rs`). The stub
  `compute_m2_ee_mumu` in `src/lib.rs` (created by T4) is now redundant.
- The real entry point is `src/helas/mod.rs` which exposes `ixxxxx`/`oxxxxx`/`j3xxxx`/
  `iovxxx` directly. The integration test `tests/helas_validation.rs` needs to be
  updated to call the actual HELAS Rust functions rather than the stub.
- ✅ **Extended-validation test gating implemented:** `tests/helas_validation.rs` is now
  gated behind `#[cfg(feature = "extended-validation")]` — invisible to normal `cargo test`,
  opt-in via `cargo test --features extended-validation`. The `Cargo.toml` `[features]`
  section declares `extended-validation = []`. The pixi `helas-validation` environment
  has a `validate-helas` task that chains the full pipeline:
  ```
  pixi run -e helas-validation validate-helas
  # → build-helas → gen-reference → cargo test --features extended-validation --test helas_validation
  ```
- **Follow-up (high priority):**
  1. ✅ **Done (2026-05-22):** `src/lib.rs` updated to `pub mod helas; pub mod ufo;`
     (no longer a stub). `compute_m2_ee_mumu` implemented in `src/helas/mod.rs` using
     real `DiracWf`/`j3xxxx`/`iovxxx` routines with physical QED coupling. Tests pass.
  2. ✅ **Done (2026-05-22):** Full Fortran f2py harness run and validated.
     `pixi run -e helas-validation validate-helas` passes all 400 grid points with
     max rel diff < 1e-6. Two bugs found and fixed:
     - `np.trapz` → `np.trapezoid` in `gen_reference.py` (NumPy 2.0 API change)
     - `ELEM_CHARGE` corrected from `sqrt(4π/137.0)` to `sqrt(4π/137.035999084)`;
       the old value caused a systematic ~5.32e-4 bias in |M|² across all 400 points.
  3. ✅ **Done (2026-05-22):** 5 new tests added covering multi-angle kinematics and
     physical coupling (see T1 follow-up above).
  4. ✅ **Done:** Cross-checked MadGraph HELAS call sequence vs harness and Rust.
     Found 3 discrepancies; all 3 fixed (see SM alignment note below).

### T4 SM alignment: discrepancies and fixes

After inspecting `ee_to_mumu/SubProcesses/P1_ll_ll/matrix1_optim.f` (MadGraph γ+Z matrix element).

**Discrepancy 1 — Z boson missing from harness (`gen_reference.py`)**
- Original: photon only (`jioxxx` with `vmass=0`). At Z pole: MadGraph=2025 pb, pure QED≈2 pb (×1000 off).
- Fixed: `gen_reference.py` now calls `jioxxx` twice (γ and Z), sums amplitudes coherently before squaring.

**Discrepancy 2 — Z decoupled in Rust (`compute_m2_ee_mumu`)**
- Original: `j3xxxx` with `zmass=1e12` (Z→0) and Thompson alpha (α=1/137).
- Fixed: uses `jioxxx` for separate γ and Z, coherent sum, MadGraph SM params:
  `aEWM1=132.507`, `Gf=1.16639e-5`, `MZ=91.188 GeV`, `WZ=2.441404 GeV`.

**Discrepancy 3 — `j3xxxx` not suitable for physical SM γ+Z couplings**
- `j3xxxx` computes the SU(2)×U(1) W³ gauge-eigenstate current; introduces spurious
  sin θW factors. MadGraph uses separate FFV1P0_3 (γ) and FFV2_4_3 (Z) routines.
- Fixed: added `jioxxx` to `src/helas/vertex.rs` — single-boson off-shell current with
  independent `[g_L, g_R]` couplings. `j3xxxx` retained for W³-basis unit tests.

**SM coupling values (param_card.dat):**
- `e = sqrt(4π/132.507) ≈ 0.30803`,  `sw² ≈ 0.2221`
- `gL_Z = e(−½+sw²)/(sw·cw) ≈ −0.20590`  (= Im GC_59 in MadGraph)
- `gR_Z = e·sw/cw ≈ +0.16469`             (= Im GC_50 in MadGraph)

**`test_ee_to_mumu_multi_angle` updated:** replaced pure-QED check at 91.2 GeV with:
1. Off-Z-pole QED agreement at √s=10 GeV (10% tolerance)
2. Z-pole resonance enhancement: SM > 50× QED at √s=MZ

**T5 — Process grammar** (`research/notes/06-process-grammar.md`)
- Main initialized and uses `feyngraph` as a real Rust dependency (it was uninitialized
  when T5 ran). The feyngraph crate is now checked out at `research/refs/feyngraph/`.
- ✅ **Follow-up done (2026-05-22):** Section 8 of `06-process-grammar.md` rewritten
  as a concrete gap analysis based on the actual feyngraph source. Key findings:
  - feyngraph exposes a **fully programmatic Rust API** (`DiagramGenerator::new` takes
    `&[&str]` particle names, no MadGraph text format required).
  - `DiagramSelector` covers coupling orders (`select_coupling_power`), forbidden
    propagators (`select_propagator_count("Z", 0)`), and arbitrary diagram predicates
    (`add_custom_function`). Required/forbidden s-channels need custom functions (no
    built-in); decay chains are handled by vibegraph iteratively.
  - feyngraph's own `from_ufo` parser is incomplete (drops mass/width/Lorentz/color
    — same defects documented in `04-ufo-parsing-future.md`). The recommended approach
    is to build feyngraph's `Model` programmatically from vibegraph's own `UfoModel`,
    bypassing feyngraph's parser entirely.
  - **Conclusion:** A thin translation layer (~300 lines) is sufficient for LO
    tree-level use. A full MadGraph-compatible PEG parser is optional / future work.
- ✅ **Follow-up done (2026-05-22):** Grammar notation clarified — the `pest`
  syntax in section 5 is reference notation only; implementation will use the
  `peg` crate (`peg::parser!`) to stay consistent with the UFO parsers in
  `src/ufo/`. The conclusion section in section 8 updated accordingly.
