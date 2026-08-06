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
- **Never hand-write a standard primitive** (hash, RNG, compression): add the
  crate instead — "no suitable dependency in the set yet" is a reason to add
  one, not to write one. State the actual requirement explicitly and pin it
  (e.g. cross-build digest stability → `sha2` with known-answer tests); reserve
  in-tree implementations for things specific to this project's physics.

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
- **Amplitude disagreements: bit-exact oracle first** (note 12 is the full
  methodology). Match parameter provenance on both sides, then go straight to
  per-diagram × per-helicity (and per-flow) complex dumps. Ward identities,
  hand-built test diagrams, two-helicity ratios and total |M|² are
  underdetermined oracles; machine-check census claims (by-hand diagram counts
  have been wrong); verify a candidate fix arithmetically on the dumped
  amplitudes before writing production code.
- **Samplers gate statistically, and a fixed-seed pull is not evidence.**
  Bit-for-bit exists only with a pinned seed and unchanged sampling order.
  VEGAS's inverse-variance iteration combination turns a missed region into a
  *confidently wrong* σ — small error bar included — so sweep ≥5 seeds and read
  the spread and χ²/dof, not the headline pull. A clean sweep is necessary, not
  sufficient: five mutually consistent seeds have been collectively 1% low, so
  budget convergence is a second axis. If extra budget makes a failure migrate
  between seeds instead of shrinking, it is a bug, not statistics.
- **A per-event field is a finer oracle than a cross section, and it exists more
  often than it looks** — pin intermediates (scales, merge sequences, per-event
  replays) against the reference's own record before flipping a σ gate, so every
  flip carries a diagnosis.
- **A report is only evidence if every green cell is a recorded measurement** —
  inferring a cell from "the suite passed" is the same failure as a vacuous
  check.
- **ULP exactness is never the target.** This is numerical physics simulation:
  last-ulp effects are always subdominant to some algorithmic limit — sampling
  error, re-association when simplifying algebra, and eventually reduced
  precision (f32 and below on GPUs). Don't pin float results to exact bits
  across platforms (system libms legitimately differ at the last ulp), and
  don't add dependencies to force bit-reproducibility. Set each tolerance at
  the scale of the algorithm's own error, with measured headroom for ulp
  noise. When a variation *is* intolerable at that scale, treat it as a
  numerical-stability problem and reformulate the algorithm (ordering,
  cancellation, conditioning) — not the comparison.

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

### Sprint & Subagent Operations (manager side)

Sprints are run by a manager session dispatching one dev agent per session
(`feature-dev` / `validation-dev` / `performance-dev`; for miscellaneous tasks
use the `claude` agent type with an explicit model override — never
`general-purpose`, which ignores model overrides).

- **Own the worktrees.** Harness worktree isolation has repeatedly failed here:
  agents (especially resumed ones) editing the shared main checkout, sessions
  branched from a stale base, and fresh worktrees missing the gitignored MG
  reference data (whose absence silently triggers a multi-hour MG regeneration).
  Pre-create each worktree off `main` (`git worktree add -b <branch> <path>
  main`), verify its HEAD equals current `main` right after dispatch, COW-copy
  the reference data in (`cp -Rc`, instant on APFS) — that means
  `validation/madgraph/output`, the fetched `validation/pdf` sets, **and** the
  `research/refs/mg5amcnlo` submodule content (at minimum `models/`; a fresh
  worktree gets none of the submodule, and `cargo test` fail-fasts on the
  missing SM UFO source so the entire `validate_*` layer silently never runs)
  — and require the agent's first action to be `cd` + toplevel/branch
  verification. Worker branches often
  carry zero commits — find work via `git worktree list` plus per-worktree
  status, not `git log`.
- **Subagent reports are evidence, not truth.** Demand the command alongside its
  output ("it passed" is unfalsifiable), spot-check cheap high-consequence
  claims (`git log` after "I committed"; a build after "clean tree"), and
  sanity-check numbers against physical plausibility (a wall time that cannot
  contain the work it claims). When relaying, mark what was verified versus
  asserted. Briefs carry errors in the other direction too — invite sessions to
  correct their brief.
- **Long foreground commands kill agents** at the ~600 s stream watchdog, and
  killed runs leave zombie `cargo` processes needing `kill -9`. Dispatch briefs
  must carry the dev-agent prompts' worktree/long-command discipline verbatim.
  Watch for sleep-based watcher-shell rings (`ps -ax -o pid,ppid,etime,stat,command
  | grep sleep`) — kill the sleepers, never the real job. For a stall-proof long
  run, detach it entirely (Python `os.setsid()` double-fork; `setsid` the CLI is
  absent on macOS). An interrupted worktree `git submodule update` leaves partial
  state under `.git/worktrees/<wt>/modules/` — remove it before retrying.

## Working Notes

See `research/notes/` for step-by-step derivations and implementation notes.
