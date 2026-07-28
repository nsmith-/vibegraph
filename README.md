# vibegraph

<p align="center">
  <img src="assets/badger.png" width="420"
       alt="A whimsical photograph of a blissful European badger seated cross-legged on a mossy forest floor at sunset. The badger is dressed in a vibrant tie-dye t-shirt, wears large wooden headphones, and listens to music on a vintage cassette tape player. Its eyes are contentedly closed, soaking in the moment, surrounded by ferns, glowing mushrooms, small daisies, a sprig of lavender, and a dreamcatcher hanging from a tree branch above. The soft, filtering golden light of the woods creates a warm and peaceful atmosphere." />
</p>

A **leading-order (LO) tree-level Monte Carlo event generator** written in Rust,
following the same pipeline as
[MadGraph5\_aMC@NLO](https://arxiv.org/abs/1405.0301) — and validated against it
at the strictest level each stage permits: bit-level on the UFO-model and card
inputs, machine precision on the matrix elements, and statistical agreement on
the emitted Les Houches event samples.

## Scope

**Current goal — near-MVP**: an LO event generator for **arbitrary fixed-order
processes** with **full support for arbitrary UFO models**. The pipeline runs
end to end today: a UFO model and a MadGraph-style process card go in, an
unweighted `.lhe` event sample comes out.

```
UFO model ──▶ diagram enumeration ──▶ helicity amplitudes (HELAS/ALOHA)
                                              │
        LHEF events ◀── unweighting ◀── σ integration ◀── phase-space sampling
                                                          (multichannel VEGAS)
```

"Arbitrary" currently ends at the SM's feature set: a few tensor-coupling
representations (color sextets, baryonic epsilons, spin ≥ 3/2, Majorana
fermions) are deliberate hard errors rather than silent gaps, and a handful of
validation items remain open — both detailed in the sections below and tracked
in [`TODO.md`](TODO.md).

**Future scope may include**: LO MLM-style matching + merging, and NLO event
generation.

## Getting started

### For users

A packaged distribution is planned (tracked in `TODO.md`): precompiled
binaries hosted on the GitHub releases page with a default PDF set interned,
plus utilities that download UFO models and PDF sets from their canonical URLs
into a local cache (`~/.vibegraph`, prompting on first use). Until that lands,
build from source as below — the only extra step is fetching a PDF set for
hadronic runs.

### For developers

```bash
git clone --recurse-submodules <repo-url>
cd vibegraph
cargo build --release        # library + `vibegraph` CLI
cargo test                   # fast test suite (extended validation is opt-in)
```

The MadGraph-referenced validation and the PDF grids use
[pixi](https://pixi.sh) environments; see [Validation](#validation) below.

## Using the CLI

The CLI splits event generation into two phases that share one on-disk
artifact, mirroring MadGraph's survey/refine-then-generate structure:

1. `vibegraph integrate` — enumerate diagrams, compile the matrix element,
   adapt one VEGAS grid **per phase-space channel**, and bank everything in
   `grid.bin.zst`.
2. `vibegraph generate` — reload the artifact, unweight against the frozen
   grids by accept/reject, and write a Les Houches event file.

Both consume MadGraph card formats, so the same cards can drive a MadGraph
reference run unchanged.

### Example: fixed-energy lepton collider

```bash
# proc_card.dat
import model sm
generate e+ e- > mu+ mu-
```

```bash
# run_card.dat — fixed-energy partonic beams at the Z pole
  0    = lpp1
  0    = lpp2
  45.6 = ebeam1
  45.6 = ebeam2
```

```bash
vibegraph integrate proc_card.dat --run-card run_card.dat --out run/
# → prints σ ± err (pb), writes run/grid.bin.zst

vibegraph generate run/grid.bin.zst proc_card.dat --run-card run_card.dat \
  --nevents 20000 --seed 1 -o events.lhe
```

`generate` writes weighted-buffer output by default (`IDWTUP = -4`); pass
`--strategy stochastic-rounding` for unit-weight events (`IDWTUP = +3`).
Overweighted points are kept at weight > 1 and counted, never silently clipped.

### Example: hadronic Drell–Yan

`integrate` also covers PDF-convolved proton–proton Drell–Yan
(`p p > e+ e-`), using a pure-Rust LHAPDF6 grid reader. Fetch the PDF set
first (gitignored):

```bash
pixi run -e madgraph fetch-pdf     # into validation/pdf/<set>/
vibegraph integrate validation/madgraph/dy13_proc_card.dat \
  --run-card validation/madgraph/dy13_default_run_card.dat --out run/
```

The PDF directory resolves via `--pdf-dir`, else `$VIBEGRAPH_PDF_DIR`, else
`validation/pdf`; `--pdf-set` selects the set (default
`NNPDF23_lo_as_0130_qed`, MG5's LO default). Event generation for proton beams
(`lpp = 1`) is not wired yet — `generate` refuses it by name; the fixed-energy
path above is the full pipeline.

### The grid artifact, and easy parallelization

`grid.bin.zst` is a bincode + zstd snapshot of the adapted per-channel VEGAS
grids **plus everything needed to refuse a mismatched replay**: the process,
the model identity (import label + SHA-256 digest of the parsed model), the PDF
set, the seed and evaluation counts, and the fully resolved run card.
`generate` compares its own cards against the banked ones exactly — any
difference, including one no physics reads, is refused rather than sampled.

This split makes job-level parallelism trivial: the expensive adaptive
integration runs **once**, and the artifact is a read-only input to any number
of `generate` jobs. Each job picks its own `--seed` (same seed → identical
sample; the RNG is a splittable ChaCha8, so distinct seeds give independent
streams) and its own `--nevents`, and the resulting `.lhe` files concatenate
into one sample. Different processes are simply different `integrate` runs —
process-level parallelism falls out of the same structure.

```bash
vibegraph integrate proc_card.dat --run-card run_card.dat --out run/   # once
for seed in 1 2 3 4; do
  vibegraph generate run/grid.bin.zst proc_card.dat --run-card run_card.dat \
    --seed $seed -o events.$seed.lhe &                                 # fan out
done
```

Run `vibegraph <cmd> --help` for the full option list.

## Feature breakdown

| Pipeline step | Status |
|---|---|
| UFO model loading | ✅ Python-AST parser for arbitrary UFO models; restrict cards baked into parameters; model identity (label + SHA-256 over the parsed model) banked into artifacts |
| Feynman diagram enumeration | ✅ [feyngraph](https://github.com/Jens-Braun/FeynGraph) topology generation + a MadGraph-style process grammar (`p p > e+ e-`, coupling-order constraints, multiparticle labels); validated against MadGraph's diagram counts |
| Helicity amplitudes | ✅ HELAS-style evaluation compiled directly from UFO Lorentz structures (the ALOHA role), topology-driven for arbitrary processes; exact color-factor \|M\|² via per-flow JAMPs; per-helicity program expansion with MadGraph-matched helicity filtering |
| Phase-space sampling | ✅ Lepage VEGAS (deterministic parallel chunking, serde-frozen grids) + n-body LIPS/RAMBO generic over the scalar type, and MadGraph-style **multichannel**: per-diagram propagator-pole channel trees, Breit–Wigner / t-channel / massless-log maps, variance-minimising weights with α-adaptation, one grid per channel |
| Cross section + running couplings | ✅ Leptonic and hadronic (PDF-convolved) σ with compiled MadGraph run-card cuts; MadGraph's αs RGE and per-event μR / per-beam μF prescriptions |
| Unweighted event output | ✅ Accept/reject over the frozen grids; per-event helicity (∝ \|M_hel\|²) and colour-flow (∝ JAMP2) selection with a flow→`ICOLUP` dictionary checked against MadGraph's `leshouche.inc`; `SCALUP`/`AQCDUP`; a four-layer LHEF writer/reader |

Notable current boundaries (all hard errors or tracked rows, not silent
wrongness): color sextets / baryonic epsilon tensors, spin-3/2 and spin-2
wavefunctions, Majorana fermions, loop-level UFOs (out of the LO charter),
proton-beam event generation, and kT-clustered dynamical scales
(`dynamical_scale_choice = -1`) — the prerequisite for gating QCD beyond 2→2.
See the backlogs in [`TODO.md`](TODO.md) and the design notes in
[`research/notes/`](research/notes/).

## Validation

The validation strategy is built on one principle: **every oracle has a blind
spot**, so agreement is enforced at the finest level each quantity permits, and
for each gate we record what error class it provably cannot see. Three levels
of strictness are in play.

**Bit-level (exact equality).** The inputs are shared with MadGraph exactly:
vibegraph consumes MadGraph's own `param_card.dat` and `run_card.dat` files
verbatim (an artifact replay compares every run-card field for exact
equality), and the parsed UFO model is pinned by a SHA-256 digest. Where both
sides compute the same number in the same order, the comparison is bitwise:
the αs RGE is **bit-exact** against a reference built from MadGraph's own
unmodified `alfas_functions.f`; helicity-filter survivor sets match MadGraph's
generated `NHEL` tables bitwise (and the pruned \|M\|² sum stays bit-for-bit
equal to the unpruned one); the colour-flow `ICOLUP` dictionary matches
`leshouche.inc` for 24/24 subprocesses. The LHEF layer also re-serialises all
20 banked MadGraph `.lhe.gz` files byte-for-byte (198,747 events) — a format
pin on the writer, not a physics statement.

**Floating-point-reassociation level (≤ 1e-12 relative).** Per-point \|M\|² is
compared against MadGraph's generated Fortran at `REL_TOL = 1e-12` — a budget
sized for summation-order differences only, since both sides use the identical
`param_card.dat`. All 14 gated processes pass: the 11 color-free ones (up to
2→6, massive externals, VVV couplings) sit at ≤ 6.3e-13 with many points
bit-identical, and the three colored ones — where the CF-weighted flow
contraction genuinely reassociates — at 5.6e-14 (`u u~ > u u~`), 1.9e-15
(`g g > t t~`), and 8.3e-14 (`g g > g g`, NCOLOR=6). Beneath that, the
per-flow **JAMP oracle** compares complex per-flow amplitudes element-wise
against banked MadGraph `JAMP()` values — the level a color-summed \|M\|² is
blind to (per-flow phases, basis permutations).

**Statistical level.** Integrated cross sections carry Monte Carlo error, so
the σ gate compares against banked MadGraph values statistically: 11 processes
gated (including resonant Z-pole rows and the three QCD 2→2s), with hadronic
Drell–Yan agreeing to 0.14% / 0.07% on two cut configurations. Because VEGAS's
1/σ² iteration-combining can make a missed phase-space region *confidently*
wrong, sampler gates run **seed sweeps**, never a single fixed-seed pull.
Unweighted samples are checked to reproduce the integrated σ.

**Self-consistency, independent of MadGraph.** Convention claims ("this sign
is automatic") are treated as hypotheses to be pinned: a rooting-soundness
sweep re-derives every amplitude from all 133 alternative diagram rootings
(0 failures, all sign conventions lifted into one `fermi_sign` function), and a
guard test ensures every convention channel is exercised by at least one gated
process, so a future edit cannot silently step outside the validated envelope.

The main gates (each regenerates its own MadGraph reference; add `--skip-deps`
when the reference is fresh):

```bash
pixi run -e madgraph validate-diagrams     # diagram counts vs MG
pixi run -e madgraph validate-helas-mg     # per-point |M|², bit / 1e-12
pixi run -e madgraph validate-color-jamp   # per-flow complex JAMPs
pixi run -e madgraph validate-alphas       # αs RGE + per-event AQCDUP
pixi run -e madgraph validate-scales       # per-event μR, per-beam μF
pixi run -e madgraph validate-sigma        # σ gate (statistical)
pixi run -e madgraph validate-unweighting  # accept/reject machinery
pixi run -e madgraph validate-lhef         # LHEF byte round-trip
pixi run -e helas-validation validate-helas  # Fortran77 HELAS kernel cross-check
```

Open validation items (tracked in `TODO.md`): downstream-shower (Pythia)
consumption of emitted `.lhe` files, a distribution-level comparison of the
unweighted sample against MadGraph's (including empirical helicity/colour-flow
frequencies), per-flavor diagram matching, and two pinned sub-percent σ
discrepancies under active investigation — kept as tracked rows, never as
loosened tolerances.

## Performance

Per-point matrix-element evaluation currently sits **within ~1.2×–3.5×** of
MadGraph's generated, helicity-filtered Fortran (`matrix1_optim.f`) across
2→2 through 2→6 processes, with a further **1.18×–2.19×** per-process gain
landed since that measurement. The hadronic Drell–Yan σ run completes in ~2 s
single-threaded. Not bad for a runtime evaluator built at model-load time
against code MadGraph generates and compiles per process.

The path there (full record in notes 15, 17, 20):

1. **Starting point ~8.6×–110× slower.** The first working evaluator walked a
   general expression DAG per point.
2. **Layout program**: struct-of-arrays arena layout, constant folding and
   pooling, binary-arity lowering with add-flattening, bounds-check
   elimination — the compiled program becomes a flat typed tape.
3. **Helicity expansion**: compile one straight-line program per contributing
   helicity combination instead of looping a generic one, sharing
   helicity-independent subexpressions.
4. **Helicity filtering**: probe and drop vanishing helicity combinations
   exactly as MadGraph's `NHEL` survey does (survivor sets match MG bitwise;
   the pruned sum is bit-for-bit the unpruned one). This was the big one —
   it put the comparison on equal combination counts (e.g. 16/256 for a 2→6)
   and collapsed the 2→6 gap from 25× to 2.5×. Gap after: **1.2×–3.5×**.
5. **Second pass** — four independent sessions, cumulatively **1.18×–2.19×**
   on every benchmarked process, still ≤ 1e-12 vs MadGraph:
   - *Multiply splitting*: one generic complex `Mul` op became eight typed
     variants specialised by operand kind (real×scalar, scalar×vector, …),
     since ~86% of hot multiplies had a cheaper shape than the general
     kernel. The broad base win: −13% to −44% ns/eval on every process,
     biggest on the Mul-heavy `g g > g g`. Bit-exact.
   - *One-shot DAG validation*: per-node cross-checks compiled into the eval
     loop moved to a single up-front arena validation. No release-build
     change, but it made the extended-validation timings honest (3.1×–5.4×
     faster there).
   - *`ZEROAMP` skipping*: MadGraph's second filter layer — probe and drop
     per-(helicity, diagram) zero amplitudes inside surviving combinations.
     The colored-2→2 win (`e+ e- > e+ e-`, `u u~ > u u~`, `g g > g g`),
     beating its "likely small" prior. Bit-exact.
   - *Fewest-external-leg rooting*: re-root each diagram at the vertex
     touching the fewest external legs, maximising current sharing under CSE.
     The multi-leg win: −18% to −26% on 2→3/2→4/2→6, neutral on 2→2. This one
     reassociates the arithmetic, yet the MG gate stayed at 1e-12 — shorter,
     more-shared current chains actually reduced FP drift.

Caveats: ratios are single-host (Apple M3 Max) measurements, not constants —
`scripts/mg_perf_compare.sh` is a rerun kit for other platforms. Pruned
evaluators inherit MadGraph's frame contract (partonic-CM momenta, beams along
±z). The remaining gap is largest on colored 2→2s, where NCOLOR-flow
contraction dominates; candidate next steps (e-graph sharing extraction,
per-event scale hot-path cost, feyngraph enumeration allocations) are triaged
in the performance backlog in `TODO.md`.

```bash
pixi run -e madgraph profile-sigma   # samply profile of the σ gate
```

## Repository layout

```
vibegraph-lib/        Library: ufo/, diagrams/, helas/, phasespace/, vegas,
                      coupling/, pdf/, hadronic, cuts, unweight, lhef/, artifact
vibegraph-cli/        The `vibegraph` binary (integrate, generate)
validation/           MadGraph/HELAS/PDF reference generation + banked references
research/notes/       Numbered design + close-out notes (the project's real record)
research/refs/        Reference code as submodules (mg5amcnlo, feyngraph) — see
                      research/refs/README.md
scripts/              Profiling and cross-platform perf-comparison kits
```

`TODO.md` holds the prioritized backlogs and pipeline status; the notes in
`research/notes/` record each sprint's design, bugs, and close-out in full.
