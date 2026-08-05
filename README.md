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
Standard Model processes**, driven end to end by the standard toolchain
formats: a UFO model and a MadGraph-style process card go in, an unweighted
`.lhe` event sample comes out. That includes hadronic processes at MadGraph's
own default dynamical scale — the kT-clustering prescription
(`dynamical_scale_choice = -1`) is reproduced against MadGraph's clustering
itself, not approximated.

```
UFO model ──▶ diagram enumeration ──▶ helicity amplitudes (HELAS/ALOHA)
                                              │
        LHEF events ◀── unweighting ◀── σ integration ◀── phase-space sampling
                                                          (multichannel VEGAS)
```

The UFO loader is model-generic, but the supported feature surface is
deliberately scoped to the Standard Model's: representations the SM does not
use — color sextets, baryonic epsilon tensors, spin ≥ 3/2, Majorana fermions —
are hard errors rather than silent gaps. Beam configurations other than
unpolarized proton–proton or fixed-energy partonic collisions, and MadGraph's
decay-chain process syntax, are likewise out of scope for now; the audit making
every such boundary a hard error is part of the validation backlog. The
remaining open validation items are detailed in the sections below and tracked
in [`TODO.md`](TODO.md).

**Future scope may include**: full support for arbitrary (BSM) UFO models —
the boundary checklist already lives in [`TODO.md`](TODO.md) — plus LO
MLM-style matching + merging, and NLO event generation.

## Getting started

### Quick start — no toolchain required

Precompiled binaries for macOS (Apple Silicon and Intel) and Linux x86\_64 are
attached to every [release](../../releases). The Linux build is statically
linked; none of them needs a Rust toolchain, a Python installation, or LHAPDF.

```bash
# 1. Download and unpack (pick the archive matching your platform)
curl -fsSLO https://github.com/nsmith-/vibegraph/releases/latest/download/vibegraph-aarch64-apple-darwin.tar.gz
tar xzf vibegraph-aarch64-apple-darwin.tar.gz
cd vibegraph-aarch64-apple-darwin
./vibegraph --version
```

```bash
# 2. Write two MadGraph-format cards
cat > proc_card.dat <<'EOF'
import model sm
generate e+ e- > mu+ mu-
EOF

cat > run_card.dat <<'EOF'
  0    = lpp1
  0    = lpp2
  45.6 = ebeam1
  45.6 = ebeam2
EOF
```

```bash
# 3. Integrate, generate, and read the events back
./vibegraph integrate proc_card.dat --run-card run_card.dat --out run/
./vibegraph generate run/grid.bin.zst proc_card.dat --run-card run_card.dat \
  --nevents 10000 --seed 1 -o events.lhe
./vibegraph check-events events.lhe
```

`events.lhe` is a standard Les Houches event file. `check-events` re-reads it
and checks momentum balance, mass shells, weight bounds and the `<init>`
cross-references — a self-read, so it catches a damaged or truncated file but
not a format both the writer and the reader agree on wrongly.

#### Data the binary does not carry

The Standard Model is compiled in, so the run above needs nothing else. Two
kinds of data are resolved on demand, each in the order
`--flag` → environment variable → `~/.vibegraph/` → the working directory:

| Data | Flag | Environment | Cached at |
|---|---|---|---|
| PDF sets (proton beams) | `--pdf-dir` | `$VIBEGRAPH_PDF_DIR` | `~/.vibegraph/pdf/<set>/` |
| UFO models (non-SM) | `--ufo-dir` | `$VIBEGRAPH_UFO_DIR` | `~/.vibegraph/ufo/<model>/` |

A **PDF set** that is not there can be downloaded: vibegraph shows the URL, the
size and the SHA-256 it will be checked against, and asks. The checksum is
compiled into the binary, so the data a set name resolves to cannot drift from
what the build was validated against.

**Nothing is ever downloaded without consent.** Without a terminal to ask on —
a script, a CI job, a container build — the answer is no, and the run fails
with a message naming the URL, the checksum, and the directory to unpack it
into by hand. Pass `--yes` to consent up front, `--no-network` (or set
`$VIBEGRAPH_NO_NETWORK`) to forbid downloads outright; a refusal always wins
over `--yes`. Set `$VIBEGRAPH_HOME` to move the cache off `~/.vibegraph`.

**UFO models are never downloaded.** FeynRules publishes no per-model index
that a model name could be resolved through, so there is no URL to pin;
unpack the model's UFO directory into `~/.vibegraph/ufo/<model>/` yourself,
or point `--ufo-dir` at whatever directory holds it.

`scripts/acceptance.sh` runs this whole path on a clean machine, on the harder
process `p p > l+ l- j`: download the binary, write the cards, watch an
unattended run *refuse* to fetch, consent, fetch and verify the PDF set, emit
events off the cache, read them back.

### For developers

```bash
git clone --recurse-submodules <repo-url>
cd vibegraph
cargo build --release        # library + `vibegraph` CLI
cargo test                   # the hermetic suite: complete on a bare clone
```

The layers that assume fetched data or a MadGraph installation use
[pixi](https://pixi.sh) environments; see [Validation](#validation) below.

## Using the CLI

The CLI splits event generation into two phases that share one on-disk
artifact, mirroring MadGraph's survey/refine-then-generate structure:

1. `vibegraph integrate` — enumerate diagrams, compile the matrix element,
   adapt one VEGAS grid **per phase-space channel**, and bank everything in
   `grid.bin.zst`.
2. `vibegraph generate` — reload the artifact, unweight against the frozen
   grids by accept/reject, and write a Les Houches event file.

A third command, `vibegraph check-events`, reads an emitted file back and
checks it against itself.

Both phases consume MadGraph card formats, so the same cards can drive a
MadGraph reference run unchanged.

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

How many of them there are is set by the frozen scan that estimates each
channel's maximum weight, and `--scan-points <N|share>` sizes it: `N` points on
every channel, or (the default) each channel's share of the integration budget.
A longer scan finds higher maxima — fewer and smaller overweights, and a lower
acceptance for the same number of events. It is a trade rather than a
convergence: on the `p p > l+ l- j` grids the summed maximum still grows as
`n^0.51` at 2.6·10⁵ points per channel.

### Example: proton beams

Both phases run at `lpp = 1`, PDF-convolved through a pure-Rust LHAPDF6 grid
reader, over an arbitrary process. Cards to events for `p p > l+ l- j`:

```bash
# proc_card.dat
import model sm
generate p p > l+ l- j QCD=2 QED=2
```

```bash
# run_card.dat — 13 TeV proton beams, fixed scales, jet and lepton cuts
  1       = lpp1
  1       = lpp2
  6500.0  = ebeam1
  6500.0  = ebeam2
  lhapdf  = pdlabel
  247000  = lhaid
  True    = fixed_ren_scale
  True    = fixed_fac_scale1
  True    = fixed_fac_scale2
  91.188  = scale
  91.188  = dsqrt_q2fact1
  91.188  = dsqrt_q2fact2
  50.0    = mmll
  20.0    = ptj
  10.0    = ptl
  5.0     = etaj
  2.5     = etal
  0.4     = drll
  0.4     = drjl
```

The scale lines pin μR and μF to fixed values; leave them out and MadGraph's
default prescription applies instead — the kT-clustered dynamical scale
(`dynamical_scale_choice = -1`), computed per event exactly as a MadGraph
reference run would compute it.

```bash
vibegraph integrate proc_card.dat --run-card run_card.dat --out run/
vibegraph generate run/grid.bin.zst proc_card.dat --run-card run_card.dat \
  --nevents 2000 --seed 1 -o events.lhe
```

Concrete subprocesses that share a matrix element are grouped — the grouping is
*measured* pointwise, not listed — and each group's parton luminosity is summed
over its members and both beam orderings, so one compiled program serves the
whole group.

`--pdf-set` selects the set (default `NNPDF23_lo_as_0130_qed`, MG5's LO
default), which is downloaded on first use as described in the
[quick start](#data-the-binary-does-not-carry). Inside a checkout,
`pixi run fetch-pdf` puts a set in `validation/pdf/<set>/`, which
resolution falls back to last — so a dev tree that has already fetched one
never reaches for the network.

Drell–Yan is one such process and takes no special route:

```bash
vibegraph integrate validation/madgraph/dy13_proc_card.dat \
  --run-card validation/madgraph/dy13_default_run_card.dat --out run/
```

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

**What a worker machine needs.** The artifact is not the whole run: the
compiled amplitude is deliberately not banked, so every `generate` job
re-reads the same cards and recompiles, refusing on any mismatch. A clean
worker therefore needs the binary, `grid.bin.zst`, and both cards — and, for
proton beams, the PDF set itself, because unweighting reads parton densities
and the grid's αs at every trial point. Copy the set's directory (e.g.
`NNPDF23_lo_as_0130_qed/`) into the worker's working directory, or point
`--pdf-dir` / `$VIBEGRAPH_PDF_DIR` at a shared location; otherwise each worker
asks to download the set — and an unattended job refuses instead of asking. A
non-SM run ships its UFO model directory the same way. Folding all of this
into the artifact, so a worker needs one file, is a tracked post-v0.1 feature
(see [`TODO.md`](TODO.md)).

Run `vibegraph <cmd> --help` for the full option list.

## Feature breakdown

| Pipeline step | Status |
|---|---|
| UFO model loading | ✅ Python-AST parser for UFO models; restrict cards baked into parameters; model identity (label + SHA-256 over the parsed model) banked into artifacts |
| Feynman diagram enumeration | ✅ [feyngraph](https://github.com/Jens-Braun/FeynGraph) topology generation + a MadGraph-style process grammar (`p p > e+ e-`, coupling-order constraints, multiparticle labels); validated against MadGraph's diagram counts |
| Helicity amplitudes | ✅ HELAS-style evaluation compiled directly from UFO Lorentz structures (the ALOHA role), topology-driven for arbitrary processes; exact color-factor \|M\|² via per-flow JAMPs; per-helicity program expansion with MadGraph-matched helicity filtering |
| Phase-space sampling | ✅ Lepage VEGAS (deterministic parallel chunking, serde-frozen grids) + n-body LIPS/RAMBO generic over the scalar type, and MadGraph-style **multichannel**: per-diagram propagator-pole channel trees, Breit–Wigner / multi-rung t-channel-spine / massless-log maps with per-subprocess identical-particle factors, variance-minimising weights with α-adaptation, one grid per channel |
| Cross section + running couplings | ✅ Leptonic and hadronic (PDF-convolved) σ with compiled MadGraph run-card cuts; MadGraph's αs RGE and per-event μR / per-beam μF prescriptions, including the default kT-clustered dynamical scale reproduced against MadGraph's own clustering; at proton beams, an arbitrary process through measured flavour groups summed over both beam orderings |
| Unweighted event output | ✅ Accept/reject over the frozen grids at fixed-energy **and** proton beams; per-event helicity and colour-flow selection following MadEvent's own rules, with the flow→`ICOLUP` dictionary checked against MadGraph's `leshouche.inc`; `SCALUP`/`AQCDUP`; a four-layer LHEF writer/reader that round-trips MadGraph's own event files byte-for-byte |

Notable current boundaries (hard errors or tracked rows, not silent
wrongness): color sextets / baryonic epsilon tensors, spin-3/2 and spin-2
wavefunctions, Majorana fermions, loop-level UFOs (out of the LO charter),
beam configurations beyond unpolarized proton–proton or fixed-energy partonic
beams, and decay-chain process syntax. See the backlogs in
[`TODO.md`](TODO.md) and the design notes in
[`research/notes/`](research/notes/).

## Validation

The validation strategy is built on one principle: **every oracle has a blind
spot**, so agreement with MadGraph is enforced at the finest level each
quantity permits, and for each gate the suite records what error class it
provably cannot see. Three levels of strictness are in play.

**Bit-level (exact equality)** wherever both sides compute the same number
from the same inputs. The inputs are shared with MadGraph exactly — vibegraph
consumes MadGraph's own `param_card.dat` and `run_card.dat` files verbatim,
and the parsed UFO model is pinned by a SHA-256 digest — so quantities with no
legitimate room to differ are compared bitwise: the αs RGE against MadGraph's
own `alfas_functions.f`, helicity-filter survivor sets against the generated
`NHEL` tables, the colour-flow `ICOLUP` dictionary against `leshouche.inc`,
the kT-clustering merge sequence against an instrumented MadGraph run, and the
LHEF writer against MadGraph's own event files byte-for-byte.

**Floating-point-reassociation level (≤ 1e-12 relative)** on per-point \|M\|²
against MadGraph's generated Fortran — a budget sized for summation-order
differences only, since both sides read the identical `param_card.dat`.
Beneath the colour-summed \|M\|² sit two finer oracles, each reaching a level
it is blind to: per-flow complex `JAMP` values (which see per-flow phases and
basis permutations) and per-diagram amplitudes per helicity (which see what a
single-flow \|M\|² cannot).

**Statistical level** for integrated cross sections and unweighted event
samples, which carry Monte Carlo error. σ gates compare against banked
MadGraph values at tolerances derived from the reference's own uncertainty —
and because VEGAS's inverse-variance iteration combining can make a missed
phase-space region *confidently* wrong, sampler gates run multi-seed sweeps
with the budget scanned as a second axis, never a single fixed-seed pull.
Event samples are compared distribution by distribution against MadGraph's
banked samples, and every emitted file is also read back end to end by
Pythia 8.

**Self-consistency, independent of MadGraph.** Convention claims ("this sign
is automatic") are treated as hypotheses to be pinned: sweeps re-derive every
amplitude from alternative diagram rootings, and guard tests ensure every
convention channel is exercised by at least one gated process, so a future
edit cannot silently step outside the validated envelope.

What each check may assume is a declared contract:
[`validation/manifest.toml`](validation/manifest.toml) is the single source of
truth, and every test registers into exactly one of three dependency layers —
a test that quietly skips when its data is absent is a structure the suite
does not permit.

| Layer | May assume | Command |
|---|---|---|
| `hermetic` | A bare clone — no submodule, no fetched data, no network. Complete there: it never skips. | `cargo test` |
| `banked` | The `mg5amcnlo` submodule, fetched PDF sets, and a pinned, checksummed bundle of frozen MadGraph reference runs. May *not* run MadGraph — which is what makes it reproducible on a machine that has never built a process directory. | `pixi run validate` |
| `oracle` | A full MadGraph/LHAPDF toolchain; regenerates every reference. | `pixi run generate-references` |

`pixi run validate` ends by rendering a report table: one row per validated
process, one column per category (diagrams, amplitudes, integrals, samples),
each cell carrying its metric and whether it is gated, informational, blocked
on a named feature, covered by another row, or an admitted gap. The same
driver asserts that the cells measured are exactly the cells the manifest
declares, so coverage cannot quietly shrink. **CI runs the hermetic and banked
layers as a merge gate and publishes the rendered report as each run's job
summary** — the current validation status of `main` is always one click away,
and a failing cell is read off the table rather than dug out of a log. One
banked gate lives outside that command because it needs its own environment:

```bash
pixi run -e pythia validate-pythia   # every emitted event read back by Pythia 8
```

The report's own account of what is *not* covered is the current list of open
validation items; [`TODO.md`](TODO.md) carries them with the evidence — kept
as tracked rows, never as loosened tolerances.

## Performance

Per-point matrix-element evaluation currently runs at **0.72×–1.69×** the
cost of MadGraph's generated, helicity-filtered Fortran (`matrix1_optim.f`) —
**geometric mean 1.24×** over the 14 processes the comparison kit covers, 2→2
through 2→6. Several processes sit at parity, and `e+ e- > e+ e-` runs faster
than MadGraph. Not bad for a runtime evaluator built at model-load time
against code MadGraph generates and compiles per process.

The starting point was 8.6×–110× slower: a general expression DAG walked per
point. The distance was closed by compiling that DAG into a flat typed tape
(struct-of-arrays layout, constant folding and pooling, bounds-check
elimination), expanding one straight-line program per contributing helicity
combination, reproducing MadGraph's own two layers of zero-filtering (whole
helicity combinations, then per-diagram amplitudes within them — survivor sets
match MadGraph's bitwise), and specialising the arithmetic itself (typed
multiply variants, diagram rooting chosen to maximise current sharing) — all
of it holding the ≤ 1e-12 matrix-element gate throughout. The full
optimization record lives in [`research/notes/`](research/notes/).

Caveats: ratios are single-host (Apple M3 Max) measurements, not constants —
`scripts/mg_perf_compare.sh` is the rerun kit that re-derives the full ratio
table directly on any platform (the headline is its 2026-07-28 output). Pruned
evaluators inherit MadGraph's frame contract (partonic-CM momenta, beams along
±z). The remaining gap is largest on colored 2→2s, where color-flow
contraction dominates; candidate next steps are triaged in the performance
backlog in [`TODO.md`](TODO.md).

```bash
pixi run profile-sigma   # samply profile of the σ gate
```

## Repository layout

```
vibegraph-lib/        Library: ufo/, diagrams/, helas/, phasespace/, vegas,
                      coupling/, pdf/, hadronic, proton, cuts, unweight,
                      lhef/, artifact
vibegraph-cli/        The `vibegraph` binary (integrate, generate, check-events)
validation/           MadGraph/HELAS/PDF reference generation + banked references
research/notes/       Numbered design + close-out notes (the project's real record)
research/refs/        Reference code as submodules (mg5amcnlo, feyngraph) — see
                      research/refs/README.md
scripts/              Acceptance run + profiling and perf-comparison kits
```

`TODO.md` holds the prioritized backlogs and pipeline status; the notes in
`research/notes/` record each sprint's design, bugs, and close-out in full.

## License

vibegraph's own code is dual-licensed under either

* the MIT license ([`LICENSE-MIT`](LICENSE-MIT)), or
* the Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE)),

at your option — the Rust ecosystem's convention. Unless you state otherwise,
any contribution you intentionally submit for inclusion in this work is
dual-licensed on the same terms, with no additional conditions.

Not everything shipped is vibegraph's own code. The MadGraph5\_aMC@NLO
Standard Model assets interned into the binary keep their own NCSA-derived
license; [`THIRD-PARTY-NOTICES`](THIRD-PARTY-NOTICES) identifies exactly what
is redistributed and reproduces those terms, and it travels inside every
release tarball alongside both license texts. PDF grids are fetched at run
time, never redistributed.
