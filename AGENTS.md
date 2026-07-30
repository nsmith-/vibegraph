# vibegraph — Agent Context

## Project Goal

Implement a toy LO (leading-order, tree-level) Monte Carlo event generator in Rust,
studying the standard HEP event simulation toolchain step by step.

## Planning & Progress

**Before starting any new feature or task**, read `TODO.md` — the prioritized task
list and pipeline status. Deeper derivations live in `research/notes/`.

**After completing any planned change**, update `TODO.md` to reflect current status.

## Codebase Exploration

Do not rely on a static layout description. Instead:
- Read `Cargo.toml` (and workspace member `Cargo.toml` files) to understand crate structure and dependencies.
- Use `ls`, `find`, and `grep` to explore the source tree as needed.

## Key Concepts

The pipeline uses standard HEP concepts: **UFO** (Python-based model description format), **Feynman diagram enumeration** (recursive generation from vertices + external legs), **HELAS/ALOHA** (helicity amplitude routines and their automatic generation from UFO Lorentz structures), **VEGAS** (adaptive Monte Carlo integrator), **LIPS** (Lorentz-invariant phase space), and **cross section** (σ = ∫ dΦ_n |M|²/flux, events sampled with weight |M|²/max|M|²).

For paper references, submodule locations and key paths, and instructions for fetching papers and populating submodules, see `research/refs/README.md`. Keep that file up to date when references change.

## Implementation Conventions

### Rust Type System

- **Basis-independence via trait bounds**: Lorentz/spinor/color representations are generic over
  the scalar field `F` to keep physics-layer code independent of representation details.
- **Phantom types for zero-cost abstraction**: Types like `DiracWf` use `PhantomData` to distinguish
  physical meaning (flowing-in vs. flowing-out) at compile time with zero runtime cost.
- **Import style**: Direct submodule imports are preferred over re-exports to avoid unused-import warnings.

### Code Style & Conventions

- **Natural units**: ℏ = c = 1 (GeV is the fundamental energy scale)
- **Metric signature**: (+, −, −, −)
- **Comment guidelines**: Avoid narrative comments; add notes only for non-obvious constraints or physics assumptions. Document what the code *does now*, not what it used to do or what was tried before — git history records that, and "the old X" / "no longer Y" framing is just distraction. Comments must be self-contained: never reference `TODO.md`, planning docs, sprint/task names, or plan "stages"/"sessions" (e.g. "Stage A", "the convention-refactor session"). Those artifacts are temporary and invisible to a future reader of the code, so such comments read as vacuous. Describe the code's behavior and rationale in its own terms; if a follow-up is genuinely worth flagging, describe the work itself, not the plan item that tracks it.
- **Four-momentum layout**: `[E, px, py, pz]` (energy first, spatial components follow)

### Physics Validation

- **Every oracle has a blind spot** — |M|² is blind to global phases; Gram-type
  matrices (e.g. the color CF matrix) are blind to uniform index transposes;
  per-diagram amplitude ratios differ by benign phase conventions. Validate new
  physics features at the finest linear level available (per-diagram, per-flow
  complex values), and for each test know what error class it provably cannot detect.
- **Convention claims are hypotheses**: any assertion that a sign/index/phase
  convention is "automatic" or "comes for free" must be pinned by a test that would
  fail if it were false — a passing gate that cannot see the convention is not
  confirmation.
- **Keep a known-wrong informational comparison running** while a feature is under
  construction (enforce it later): it turns "the feature went live" into an instant
  end-to-end signal against the reference.

## Build & Test

Standard `cargo build` / `cargo test`. The slow, feature-gated MadGraph/HELAS
cross-check gates — which one to run after which kind of change, and the
`--skip-deps` regeneration semantics — live in the `extended-validation` skill;
invoke it after modifying amplitudes, color, coupling, or diagram enumeration.

## Agent Tooling Guidelines

### Rust Code Exploration

**Prefer the LSP tool** for Rust code queries when available. It provides intelligent navigation:
- Find all references to a symbol
- Type information and trait implementations
- Accurate definition lookup and call hierarchies

Fall back to Unix CLI tools (`grep`, `sed`, `find`, etc.) when the LSP is unavailable or the query is simpler (e.g., finding a specific string literal).

### General Search & Extraction

**Prefer Unix CLI tools over Python scripts** for search and extraction tasks.

Use `grep`, `sed`, `awk`, `find`, etc. instead of writing ad-hoc Python scripts. Only write a
script when the task genuinely requires logic these tools cannot express.

Key flags: `grep -n` (line numbers), `grep -r` (recursive), `grep -C N` (context), `grep -l`
(filenames only), `sed -n 'N,Mp'` (line range), `find . -name "*.rs"`.

## Working Notes

See `research/notes/` for step-by-step derivations and implementation notes.
