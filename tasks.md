# vibegraph — Next-Step Tasks

This file tracks near-term research and implementation tasks.
Status is maintained in the session SQL database; update here at milestones.

---

## T1 · MadGraph Code Quality Review

**Goal:** Read `research/refs/mg5amcnlo/UpdateNotes.txt` and survey the MadGraph5 Python
codebase to identify categories of bugs that have historically appeared, with the aim of
informing what unit tests vibegraph needs to catch the same classes of errors.

**Output:** `research/notes/04-mg5-code-quality.md`

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
