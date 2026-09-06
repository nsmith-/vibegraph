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

The UFO loader is model-generic, and its Lorentz-structure surface now reaches
past the Standard Model's. SMEFTsim's `SMEFTsim_topU3l_MwScheme_UFO` — a
dimension-six SMEFT model with derivative gauge-boson vertices, Levi-Civita
structures, `γ⁵` and momenta inside γ-chains, four-fermion contacts in both
pairings and a cyclic tensor⊗tensor one, under the {m_W, m_Z, G_F} input scheme
— is enumerated, compiled and compared against MadGraph's own matrix elements
per diagram, per helicity and per colour flow, and its `e+ e- > t t~ NP<=1`
cross section is gated against a banked MadGraph run.
[`validation/manifest.toml`](validation/manifest.toml) is the per-row record of
which of those comparisons are enforced and which are measured and reported.

Representations neither model uses — color sextets, baryonic epsilon tensors,
spin ≥ 3/2, Majorana fermions — are hard errors rather than silent gaps, as are
squared-order constraints (`NP^2==1`: a bound on an interference term, which
this generator selects diagrams too early to express). Beam configurations
other than unpolarized proton–proton or fixed-energy partonic collisions, and
MadGraph's decay-chain process syntax, are likewise out of scope for now; the
audit making every such boundary a hard error is part of the validation
backlog. The remaining open validation items are detailed in the sections below
and tracked in [`TODO.md`](TODO.md).

**Future scope may include**: the rest of the arbitrary-BSM-UFO surface — the
boundary checklist already lives in [`TODO.md`](TODO.md) — plus LO MLM-style
matching + merging, and NLO event generation.

## Quickstart

Two cards and two commands. A **process card** picks the model and the process,
a **run card** sets the beams, cuts and scales; `vibegraph integrate` adapts the
phase-space grids and prints σ, and `vibegraph generate` unweights against those
frozen grids into a Les Houches event file. Both cards are MadGraph's own
formats, so the same pair drives a MadGraph reference run unchanged.

### Get the binary

Precompiled binaries for macOS (Apple Silicon and Intel) and Linux x86\_64 are
attached to every [release](../../releases) as bare executables — nothing to
unpack. The Linux build is statically linked; none of them needs a Rust
toolchain, a Python installation, or LHAPDF. License notices for the binary
and the Standard Model description compiled into it are printed by
`./vibegraph --version` (`-V` for just the version).

```bash
# pick the binary matching your platform, and mark it executable
curl -fsSL -o vibegraph https://github.com/nsmith-/vibegraph/releases/latest/download/vibegraph-aarch64-apple-darwin
chmod +x vibegraph
./vibegraph -V
```

The Linux build comes in two flavours. `vibegraph-x86_64-unknown-linux-musl`
runs on any x86\_64 machine; `…-musl-v3` is the same code compiled for
[x86-64-v3](https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels)
(AVX2/FMA, roughly Haswell and later), which is worth picking up for the
amplitude evaluation if your CPU is recent enough. It will die with an illegal
instruction on anything older, so take the plain one if unsure — this prints
`supported` when the `-v3` asset will run:

```bash
grep -qw avx2 /proc/cpuinfo && grep -qw fma /proc/cpuinfo \
  && echo supported || echo "use the baseline build"
```

To build from a checkout instead:

```bash
git clone --recurse-submodules <repo-url>
cd vibegraph
cargo build --release        # library + `vibegraph` CLI
cargo test                   # the hermetic suite: complete on a bare clone
```

The layers that assume fetched data or a MadGraph installation use
[pixi](https://pixi.sh) environments; see [Validation](#validation) below.

### A lepton collider: `e+ e- > mu+ mu-`

![Terminal recording: the two cards are written, then vibegraph integrate takes
the cross section past its 0.1% target in six VEGAS iterations and banks
grid.bin.zst, vibegraph generate unweights 10000 events out of it, and
check-events reads the file back.](assets/demo_ee.gif)

The process card is one line, and the run card is the shortest one that says
anything at all: fixed-energy partonic beams (`lpp = 0`) at the Z pole. Every
parameter left out of it — cuts, scales, PDF choice — takes MadGraph's own LO
default.

```bash
echo 'generate e+ e- > mu+ mu-' > proc_card.dat

cat > run_card.dat <<EOF
0    = lpp1
0    = lpp2
45.6 = ebeam1
45.6 = ebeam2
EOF
```

```bash
vibegraph integrate proc_card.dat --run-card run_card.dat --out ee_to_mumu/
#   → σ ± err (pb) on stdout, the adapted grids in ee_to_mumu/grid.bin.zst

vibegraph generate ee_to_mumu/grid.bin.zst proc_card.dat --run-card run_card.dat \
  --nevents 10000 --seed 1 -o events.lhe

vibegraph check-events events.lhe
```

A process card with no `import model` line gets the Standard Model compiled into
the binary; `import model <name>` loads a UFO model directory instead, and
`import model <name>-<restrict>` reads `restrict_<restrict>.dat` from inside it
the way MadGraph does.
`events.lhe` is a standard Les Houches event file, and `check-events` re-reads it
and checks momentum balance, mass shells, weight bounds and the `<init>`
cross-references — a self-read, so it catches a damaged or truncated file but not
a format both the writer and the reader agree on wrongly.

The proc-card argument also accepts `-` for a card on stdin, so a one-process run
needs no card file at all:

```bash
echo 'generate e+ e- > mu+ mu-' | vibegraph integrate - --run-card run_card.dat --out ee_to_mumu/
```

### Proton beams: `p p > l+ l- j`

![Terminal recording: vibegraph integrate runs p p > l+ l- j over 24 phase-space
channels with a live status pane pinned below the scrolling log, reaching 0.1%
in 140 iterations and 12 seconds, then vibegraph generate unweights 10000 events
and check-events reads them back.](assets/demo_pp.gif)

The same two commands, with no run card at all — which is to say MadGraph's LO
defaults: 13 TeV protons, its default jet and lepton cuts (`ptj = 20`,
`ptl = 10`, `etaj = 5`, `etal = 2.5`, `drjl = 0.4`), and, since nothing here
fixes the scales, its default *dynamical* prescription — the kT-clustered scale
(`dynamical_scale_choice = -1`), computed per event exactly as a MadGraph
reference run would compute it. The parton densities go through a pure-Rust
LHAPDF6 grid reader.

```bash
echo 'generate p p > l+ l- j' > proc_card.dat

vibegraph integrate proc_card.dat --out llj/
vibegraph generate llj/grid.bin.zst proc_card.dat --nevents 10000 --seed 1 -o events.lhe
vibegraph check-events events.lhe
```

Nothing sets an iteration count here either: the run stops when σ's relative
uncertainty reaches 0.1%, so the `integrating 117/136` the pane counts is
against a *projected* total, re-estimated from the error every iteration — which
is why the bar's denominator moves, and why the bar itself is not monotonic.
[The status pane](#the-status-pane) has the arithmetic, and `--fixed-budget`
trades the target for a set number of iterations.

`p`, `j` and `l+ l-` are MadGraph's multiparticle labels, and coupling-order
constraints belong to the same grammar (`generate p p > l+ l- j QCD=2 QED=2`).
Concrete subprocesses that share a matrix element are grouped — the grouping is
*measured* pointwise, not listed — and each group's parton luminosity is summed
over its members and both beam orderings, so one compiled program serves the
whole group.

A proton run also needs a PDF set — the one input the binary does not carry, and
the only thing here that can involve the network.

### Data the binary does not carry

The Standard Model is compiled in, so the leptonic run above needs nothing else.
Two kinds of data are resolved on demand, each in the order
`--flag` → environment variable → `~/.vibegraph/` → the working directory:

| Data | Flag | Environment | Cached at |
|---|---|---|---|
| PDF sets (proton beams) | `--pdf-dir` | `$VIBEGRAPH_PDF_DIR` | `~/.vibegraph/pdf/<set>/` |
| UFO models (non-SM) | `--ufo-dir` | `$VIBEGRAPH_UFO_DIR` | `~/.vibegraph/ufo/<model>/` |

A **PDF set** that is not there can be downloaded: vibegraph shows the URL, the
size and the SHA-256 it will be checked against, and asks — in the status pane
when the live display is up (`y` accepts, `n` declines), on `stderr` otherwise.
The checksum is compiled into the binary, so the data a set name resolves to
cannot drift from what the build was validated against.

**Nothing is ever downloaded without consent.** Without a terminal to ask on —
a script, a CI job, a container build — the answer is no, and the run fails
with a message naming the URL, the checksum, the directory to unpack it
into by hand, and the `-y` that would consent on a rerun. Pass `-y` (`--yes`)
to consent up front, `--no-network` (or set `$VIBEGRAPH_NO_NETWORK`) to forbid
downloads outright; a refusal always wins over `-y`. Set `$VIBEGRAPH_HOME` to
move the cache off `~/.vibegraph`.

**UFO models are never downloaded.** FeynRules publishes no per-model index
that a model name could be resolved through, so there is no URL to pin;
unpack the model's UFO directory into `~/.vibegraph/ufo/<model>/` yourself,
or point `--ufo-dir` at whatever directory holds it. `import model
<name>-<restrict>` selects `restrict_<restrict>.dat` from that directory, and
the restriction's values become the model's parameter defaults — so a run with
no param card of its own computes at exactly the values MadGraph's generated
`param_card.dat` would carry. The SMEFT model the validation ladder gates is
committed at [`validation/ufo/`](validation/ufo), so

```bash
printf 'import model SMEFTsim_topU3l_MwScheme_UFO-massless\ngenerate e+ e- > t t~ NP<=1\n' \
  | vibegraph integrate - --ufo-dir validation/ufo --run-card run_card.dat --out ee_to_ttx_smeft/
```

runs it out of a checkout. Every run records the model it was built from —
name, restrict card and a digest of both — in its artifact, and `generate`
refuses grids trained on a different one.

`scripts/acceptance.sh` runs the whole proton path on a clean machine: download
the binary, write the cards, watch an unattended run *refuse* to fetch, consent,
fetch and verify the PDF set, emit events off the cache, read them back.

## Using the CLI

The two phases split the work the way MadGraph's survey/refine-then-generate
structure does, and share one on-disk artifact:

1. `vibegraph integrate` — enumerate diagrams, compile the matrix element,
   adapt one VEGAS grid **per phase-space channel**, and bank everything in
   `grid.bin.zst`. It runs until σ's relative uncertainty reaches
   `--target-rel` (0.1% by default); `--fixed-budget` spends a flat
   `--neval × --niter` instead, which is the mode a banked run is reproducible
   under.
2. `vibegraph generate` — reload the artifact, unweight against the frozen
   grids by accept/reject, and write a Les Houches event file.

A third command, `vibegraph check-events`, reads an emitted file back and checks
it against itself. `vibegraph <cmd> --help` has the full option list.

### Unweighting

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

### Cards beyond the defaults

Pinning the scales and the cuts is what a full run card does — this one is the
13 TeV card the `p p > l+ l- j` validation rows run on:

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

The three `fixed_*_scale` lines are what take μR and μF off the dynamical
default; drop them and the kT-clustered per-event scale comes back.

`--pdf-set` selects the set (default `NNPDF23_lo_as_0130_qed`, MG5's LO
default), which is downloaded on first use as described
[above](#data-the-binary-does-not-carry). Inside a checkout,
`pixi run fetch-pdf` puts a set in `validation/pdf/<set>/`, which resolution
falls back to last — so a dev tree that has already fetched one never reaches
for the network.

The banked MadGraph cards in the checkout are ordinary inputs; Drell–Yan takes
no special route:

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

### Watching a run

`stdout` carries the result and nothing else — the `σ = … pb` line, the path
written, the `check-events` report — at every verbosity, so a pipeline reading
it sees the same bytes however loud the run is. Everything the run has to say
about itself goes to `stderr`.

| flag | effect |
|---|---|
| *(none)* | `info`: MadGraph-style running commentary |
| `-q` | `warn`: only what went wrong |
| `-vv` | `debug`: per-stage internals (diagram counts, compile-pass stats, per-channel iterations) |
| `-vvv` | `trace`: per-item detail |
| `--log-level <off\|error\|warn\|info\|debug\|trace>` | the level, spelled out |
| `--log-file <path>` | additionally record **everything**, at `trace`, to a file — whatever the screen is showing |
| `RUST_LOG=…` | the expert override, replacing the flags and scoping per module (`RUST_LOG=vibegraph::helas=debug`) |

At `debug` and above each line carries its module and the time since start;
below that the lines read as commentary and carry neither.

### The status pane

On a terminal — both `stdout` and `stderr` — `integrate` and `generate` draw a
six-row status pane pinned below the log: model and process brief, the stage in
progress and its bar, σ ± err (or, while unweighting, the sample and its
accept/reject efficiency), and the cost of an integrand evaluation.

**The integration bar is a projection, not a plan**, which is why its total
moves and why the fraction can fall as well as rise. A run converging to
`--target-rel` does not know how many iterations it will need, so the total the
bar divides by is `iteration × (δ / target)²`, with δ the relative uncertainty
σ currently carries: where the stop lands if the error goes on contracting as
1/√n. It is re-projected from δ every iteration, so an iteration that measures
worse than the trend pushes the destination out from under the bar. The
projection is floored at the earliest iteration the stopping test may fire at,
so the bar cannot read full while the run is still obliged to continue, capped
where the run would give up (`--max-iters`, or the iterations `--max-points`
buys, whichever binds first), and absent through the warm-up iterations, which
have no δ to project from. At the stop δ ≤ target puts it at the current
iteration, so a converged run does end on a full bar. `--fixed-budget` spends a
flat `--neval × --niter` instead, and that bar counts against a plan.

The pane is an *inline* viewport, not an alternate screen: log lines are pushed
into the terminal's real scrollback above it, so scrolling, searching and
copying work as they do for any other command and the history survives the run.
Nothing is erased on exit but the pane itself, and the run closes on a plain
summary line.

`--tui` draws it where it would not be drawn by default; `--no-tui` never draws
it. A redirected `stdout` means something downstream is reading the result, so
that is a run nobody is watching and the pane stays down.

Keys, with the current setting always shown in the pane:

| key | effect |
|---|---|
| `↑` / `↓` | raise / lower the visible level (`off … trace`) |
| `←` / `→` | cycle the module scope the *detailed* tiers are shown for: all → diagrams → helas → sampling → pdf |
| `q` / `^C` | ask the run to stop; press again to quit immediately |

Narrowing the scope leaves `info` and above alone and restricts only `debug`
and `trace`, so a narrowed filter can never swallow a warning. Every change
writes a marker line into the scrollback (`── log level → DEBUG ──`), and it
applies from the next event on — lines already emitted are part of the
terminal's history and are not rewritten. The progress measurements the pane
folds in are never printed as lines at any level; `--log-file` records them.

A first `q`/`^C` is a *graceful* stop: the integration finishes the iteration it
is in, banks the grids and terms of the iterations it completed, and writes the
artifact, which is therefore a usable — if less converged — input to `generate`.
The run's own report says it was abandoned early. A second press quits at once
with status 130.
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

### The matrix element, per point

Matrix-element evaluation currently runs at **0.65×–1.54×** the cost of
MadGraph's generated, helicity-filtered Fortran (`matrix1_optim.f`) —
**geometric mean 0.87×** over the 19 processes `scripts/mg_perf_compare.sh`
covers, 2→2 through 2→6. It is a cost ratio, so below 1.0× is faster than
MadGraph, and thirteen of the nineteen are. Of the six that are not, four are
the colour-dense rows where colour-flow contraction dominates — `u u~ > u u~`
1.54×, `g g > g g` 1.34×, `g g > t t~` 1.23×, `e+ e- > W+ W-` 1.23× — and the
other two sit at parity (1.02×). Not bad for a runtime evaluator built at
model-load time against code MadGraph generates and compiles per process.

The starting point was 8.6×–110× slower: a general expression DAG walked per
point. Four things closed the distance — compiling that DAG into a flat typed
tape, expanding one straight-line program per contributing helicity
combination, reproducing MadGraph's own two layers of zero-filtering (survivor
sets match bit for bit), and specialising the arithmetic itself — all of it
holding the ≤ 1e-12 matrix-element gate throughout. The full record lives in
[`research/notes/`](research/notes/).

### End to end: the integrand, not just the matrix element

A per-point ratio is not what anyone waits for, and the matrix element is not
the whole integrand: the phase-space map, the cuts, the scale draw and the
multichannel density sum `Σⱼ αⱼgⱼ(p)` are in every point too. On the heaviest
rows — 2→6, at 579 and 615 channels — a point that survives the cuts costs
138 µs and 207 µs, of which the density sum is 36 and 38 µs (**26%** and
**18%**). Most points never get that far — acceptance on these rows is 3.6%
and 4.3% — and the density runs only after the cut and the matrix element, so
the average *drawn* point costs 5.8 µs and 9.0 µs. Both figures were an order
of magnitude worse before the density moved behind the cut and began sharing
subtree momenta across channels — `probe_2to6_eval_cost` and
`probe_2to6_density_decomposition` in
[`validate_sigma.rs`](vibegraph-lib/tests/validate_sigma.rs) are what
re-measure them.

The figures below are therefore the ones that describe running the generator.

**CPU-seconds to a target accuracy on σ.** The figure of merit that folds in
variance as well as per-point cost, and the closest thing here to "how long
until I have the number". Our side is measured — `vibegraph integrate
--target-rel 0.001 -j 1`, three seeds, wall time including model load and
diagram enumeration. MadGraph's is its banked run's `<cumulated_time>` scaled
to the accuracy our run actually reached, by the 1/δ² Monte-Carlo law, so the
whole extrapolation sits on its side and none on ours. Errors are the χ²-scaled
ones, which is the stricter reading.

| process | channels | ours (CPU-s) | at δ | MadGraph, same δ | ratio |
|---|--:|--:|--:|--:|--:|
| `e+ e- > mu+ mu-` | 2 | 0.39 | 0.046% | 6.9 | 17.6× |
| `p p > e+ e-` (dy13) | 4 | 3.09 | 0.097% | 26.0 | 8.4× |
| `g g > g g` | 4 | 3.95 | 0.061% | 8.7 | 2.2× |
| `p p > j j` | 19 | 18.83 | 0.099% | 48.7 | 2.6× |
| `p p > l+ l- j` | 24 | 122.5 | 0.100% | 121.2 | 0.99× |

**Geometric mean 3.8× faster to a given accuracy** over the five rows —
5.4× over the first four, pulled toward parity by `p p > l+ l- j`. Both sit
*below* the throughput ratio further down, and the gap is the point: a faster
point is not a faster answer if it is a worse point.

`p p > l+ l- j` is where that shows. It converges in 140–156 iterations (all
three seeds, 16.8–18.7M evaluations) and lands at parity, its χ²/dof ≈ 1.4
pricing real iteration-to-iteration disagreement into the stop. Behind the
parity is a ~9× point deficit — it needs nine times the points MadGraph does
for the same accuracy — and that is phase-space map quality at the fiducial
cut boundary, the next lever on this row. The default `--max-iters 500`
gives it the headroom; the cap is a safety bound a converging run never
touches, so it costs the other rows nothing.

**Integrand throughput.** The per-point half of the same comparison: our points
per single-threaded second against MadEvent's per Fortran CPU-second — a
denominator that deliberately leaves its 16-way job farm out. **8.76× more
points per second, geometric mean over the 26 gated rows**, from 1.9× on
`g g > g g` (a pure-gluon 2→2 with the densest colour algebra of any process
here and no PDF work to win) to 37× on the cheapest leptonic rows. Bigger is
faster here — the opposite direction to the cost ratio above.

That it exceeds the time-to-accuracy figure means the sampler gives back part
of what the evaluator wins, and it localises the remaining work:
`p p > l+ l- j` draws 8.2× more points per second but needs 9.0× more of
them, which is exactly the parity in the table. Nothing there is wrong with
the evaluator; points per second is structurally blind to map quality.

Caveats on both. Our per-point work is not MadGraph's, so throughput is an
integrand ratio and not a matrix-element one, and whether MadEvent's recorded
point count includes its survey pass was never established — a systematic factor
of order unity on its column. The accuracy figure uses time and error rather
than point counts, but takes on the 1/δ² extrapolation instead — the factor by
which MadGraph's recorded time was rescaled runs from ×0.3 (`p p > e+ e-`) and
×0.8 (`e+ e- > mu+ mu-`) through ×1.2 (`g g > g g`) and ×4.8 (`p p > j j`) to
×11.1 on `p p > l+ l- j`. And MadGraph's `cumulated_time` excludes its
`output` and `compile` stages — about 130 CPU-s a process — while our model
load and diagram enumeration sit inside our wall time: an asymmetry against us,
left uncorrected.

**Parallel scaling.** `vibegraph integrate -j 16` against `-j 1`: **8.68×** on
Drell–Yan and **9.48×** on `p p > l+ l- j`. Thread count moves no bit — all
twenty runs behind those figures wrote one artifact digest per card, so `-j 16`
is the same number computed faster, not an approximation of it. The residual
serial term is the α-adaptation survey.

**Unweighting.** Taking `w_max` from MadGraph's own `unwgt.f` truncation-ladder
rule instead of the weight scan's extremum — which a Pareto tail of index ≈ 2
never lets converge — raised accept/reject efficiency on the five gating rows
from 22.2 / 20.6 / 23.3 / 10.6 / 4.21% to 54.1 / 52.1 / 52.9 / 38.9 / 9.98% at
matched budgets. `p p > l+ l- j` needed 2 269 051 trials for 20 000 events
before the change and 477 125 after: **4.36× cheaper per effective event**.

**The validation layer as a user runs it.** `pixi run validate` — the whole
banked gate, 29 rows — went from 691 s to **341 s wall / 1 306 s CPU** on a
quiet host, over a window in which the suite itself grew by 42 running tests.

Caveats. Every ratio here is a single-host (Apple M3 Max) measurement, not a
constant; `scripts/mg_perf_compare.sh` re-derives the full matrix-element table
on any platform, and records the fingerprint that says both sides were built at
the same optimisation level (the headline is its 2026-08-06 output). Wall times
are the noise-sensitive direction — the figures above were taken on a
deliberately quiet host, because a busy one has been worth double-digit
percentages. Pruned evaluators inherit MadGraph's frame contract (partonic-CM
momenta, beams along ±z). Candidate next steps are listed in
[`TODO.md`](TODO.md).

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
research/notes/       Numbered design and outcome notes (the project's real record)
research/refs/        Reference code as submodules (mg5amcnlo, feyngraph) — see
                      research/refs/README.md
scripts/              Acceptance run, profiling and perf-comparison kits, and
                      the VHS tapes behind the quickstart recordings
```

`TODO.md` holds the prioritized task lists and pipeline status; the notes in
`research/notes/` record the design, the bugs found, and the measured outcome of
each piece of work in full.

## Contributing

Patches welcome, with no process to clear first —
[`CONTRIBUTING.md`](CONTRIBUTING.md) is short and mostly says so. Written with
an AI harness? Also welcome, and most of this repository was:
[`AI_POLICY.md`](AI_POLICY.md) says what we ask (name the model, own the code,
back physics claims with a gate), and [`AGENTS.md`](AGENTS.md) is the context
file to point your tool at.

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
is redistributed and reproduces those terms. That file and both of
vibegraph's own license texts are compiled into every binary and printed by
`vibegraph --version`, so the notices accompany even a bare downloaded
executable; the same three files are also attached to every release. PDF
grids are fetched at run time, never redistributed.
