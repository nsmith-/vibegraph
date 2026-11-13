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

**Documentation:** <https://nsmith-.github.io/vibegraph/> — a guided tour of
how the generator works, stage by stage and paper by paper, the command-line
reference, and the `vibegraph-lib` API docs. Its source is [`docs/`](docs/)
and `scripts/build-docs.sh` builds it locally.

## Scope

**Current goal — MadGraph leading-order process parity** (without NLO): an LO
event generator for **arbitrary fixed-order Standard Model
processes**, driven end to end by the standard toolchain formats: a UFO model
and a MadGraph process card go in, an unweighted `.lhe` event sample comes out.
The process card is read in MadGraph's full grammar — decay chains
(`p p > t t~, t > b e+ ve`), 1→n decays to a partial width, the s-channel
restrictions `>`, `$` and `$$`, polarized external legs (`w+{0}`, `e-{L}`),
`add process` with `@N`, including mixed jet multiplicities under MLM matching
(`ickkw = 1`, `xqcut`: MadEvent's cut, clustering scales and α_s/PDF
reweighting, and the `<scales>` record Pythia's kT-MLM matching reads) — and
whatever the grammar can say that is not supported yet is refused as a hard
error before anything reads the card. That includes hadronic processes at MadGraph's
own default dynamical scale — the kT-clustering prescription
(`dynamical_scale_choice = -1`) is reproduced against MadGraph's clustering
itself, not approximated.

```
UFO model ──▶ diagram enumeration ──▶ helicity amplitudes (HELAS/ALOHA)
                                              │
        LHEF events ◀── unweighting ◀── σ integration ◀── phase-space sampling
                                                          (multichannel VEGAS)
```

The UFO loader is model-generic, and its Lorentz and colour surface reaches
past the Standard Model's: SMEFTsim's `topU3l_MwScheme` model and two small
models written here are enumerated, compiled and gated against MadGraph's own
matrix elements. What is covered, what is refused as a hard error (spin ≥ 3/2,
spin-2, Majorana fermions, squared-order constraints) and where each claim is
measured is in the guide's [UFO chapter](https://nsmith-.github.io/vibegraph/guide/02-ufo.html#beyond-the-standard-model)
and in [`validation/manifest.toml`](validation/manifest.toml). Beam
configurations other than unpolarized proton–proton or fixed-energy partonic
collisions are out of scope for now; open validation items are tracked in the
[backlog](https://nsmith-.github.io/vibegraph/backlog.html).

**Future scope may include**: the rest of the arbitrary-BSM-UFO surface — the
boundary checklist already lives in the backlog's items tagged `non-sm-ufo` — CKKW-L merging, and
NLO event generation.

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

vibegraph generate ee_to_mumu/grid.bin.zst --nevents 10000 --seed 1 -o events.lhe

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

![Terminal recording: vibegraph integrate runs p p > l+ l- j over 6 phase-space
channels with a live status pane pinned below the scrolling log, reaching 0.1%
in 50 iterations and 11 seconds, then vibegraph generate unweights 10000 events
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
vibegraph generate llj/grid.bin.zst --nevents 10000 --seed 1 -o events.lhe
vibegraph check-events events.lhe
```

Nothing sets an iteration count here either: the run stops when σ's relative
uncertainty reaches 0.1%, so the `integrating 20/43` the pane counts is
against a *projected* total, re-estimated from the error every iteration — which
is why the bar's denominator moves, and why the bar itself is not monotonic.
[The status pane](https://nsmith-.github.io/vibegraph/cli/overview.html#watching-a-run) has the arithmetic, and `--fixed-budget`
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

The Standard Model is compiled in, so the leptonic run above needs nothing
else. PDF sets (for proton beams) and non-SM UFO models are resolved on
demand, from a flag, an environment variable, the `~/.vibegraph/` cache or
the working directory, in that order. A missing PDF set is downloaded only
with consent, against a checksum compiled into the binary; UFO models are
never downloaded. The resolution order, the consent rules (`-y`,
`--no-network`) and the cache layout are documented under
[Data the binary does not carry](https://nsmith-.github.io/vibegraph/cli/overview.html#data-the-binary-does-not-carry).
`scripts/acceptance.sh` runs the whole proton path on a clean machine.

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
it against itself. `vibegraph <cmd> --help` has the full option list, and the
documentation site's [command-line pages](https://nsmith-.github.io/vibegraph/cli/overview.html)
cover the rest: the unweighting strategies and the maximum-weight scan, run
cards beyond the defaults, what the grid artifact contains and how it makes
`generate` jobs trivially parallel, what a worker machine needs, logging and
the live status pane, and the graceful-stop keys. The banked MadGraph cards
in the checkout are ordinary inputs:

```bash
vibegraph integrate validation/madgraph/dy13_proc_card.dat \
  --run-card validation/madgraph/dy13_default_run_card.dat --out run/
```

## Feature breakdown

| Pipeline step | Status |
|---|---|
| UFO model loading | ✅ Python-AST parser for UFO models; restrict cards baked into parameters; model identity (label + SHA-256 over the parsed model) banked into artifacts |
| Feynman diagram enumeration | ✅ [feyngraph](https://github.com/Jens-Braun/FeynGraph) topology generation under MadGraph's process-card grammar (multiparticle labels, coupling orders, `add process`/`@N`, `/`, the `>` and `$$` s-channel filters, 1→n decays, decay chains stitched onto their core legs, polarized legs); validated against MadGraph's diagram counts and its own generation on censuses of proc cards |
| Helicity amplitudes | ✅ HELAS-style evaluation compiled directly from UFO Lorentz structures (the ALOHA role), topology-driven for arbitrary processes; exact color-factor \|M\|² via per-flow JAMPs; per-helicity program expansion with MadGraph-matched helicity filtering |
| Phase-space sampling | ✅ Lepage VEGAS (deterministic parallel chunking, serde-frozen grids) + n-body LIPS/RAMBO generic over the scalar type, and MadGraph-style **multichannel**: per-diagram propagator-pole channel trees, Breit–Wigner / multi-rung t-channel-spine / massless-log maps with per-subprocess identical-particle factors, variance-minimising weights with α-adaptation, one grid per channel |
| Cross section + running couplings | ✅ Leptonic and hadronic (PDF-convolved) σ with compiled MadGraph run-card cuts; MadGraph's αs RGE and per-event μR / per-beam μF prescriptions, including the default kT-clustered dynamical scale reproduced against MadGraph's own clustering; at proton beams, an arbitrary process through measured flavour groups summed over both beam orderings; partial widths of 1→n decays; decay chains inside MadEvent's Breit–Wigner windows; `$` as MadEvent's zeroed on-shell propagator; polarized cross sections |
| Unweighted event output | ✅ Accept/reject over the frozen grids at fixed-energy **and** proton beams; per-event helicity and colour-flow selection following MadEvent's own rules, with the flow→`ICOLUP` dictionary checked against MadGraph's `leshouche.inc`; `SCALUP`/`AQCDUP`; decay-chain resonance records and one `<init>` entry per `@N`, as MadEvent writes them; a four-layer LHEF writer/reader that round-trips MadGraph's own event files byte-for-byte |

Notable current boundaries (hard errors or tracked rows, not silent
wrongness): spin-3/2 and spin-2 wavefunctions, Majorana fermions and charge
conjugation, loop-level UFOs (out of the LO charter), beam configurations
beyond unpolarized proton–proton or fixed-energy partonic beams, squared-order
constraints, polarized intermediate resonances, and `add process` lines of
different multiplicity at fixed-energy beams (at proton beams they are summed,
the way MadEvent sums its `P<n>` directories). Colour sextets and baryonic epsilon tensors are
*supported* — `Epsilon`/`EpsilonBar`, `K6`/`K6Bar`/`T6` and
`ColorRep::Sextet`, each gated against MadGraph on a row of its own — with two
corners still refused rather than guessed: a `T6` carrying adjoint indices, and
any colour basis in which such a tensor survives to an external leg, which no
Les Houches record can write. See the [backlog](https://nsmith-.github.io/vibegraph/backlog.html) and the
design record in [`research/kb/`](research/kb/index.md).

## Validation

Every stage is checked against MadGraph5\_aMC@NLO at the strictest level it
permits: bit-exact where both sides compute the same number from the same
inputs (the αs iteration, helicity survivor sets, colour tags, the kT merge
sequence, the LHEF writer), at ≤ 1e-12 relative on per-point |M|² and
beneath it on per-flow and per-diagram complex amplitudes, and statistically
on cross sections and event samples, with multi-seed sweeps because a
fixed-seed pull cannot see a confidently wrong VEGAS result. The organising
principle, that every oracle has a blind spot some other oracle must cover,
and the three dependency layers every test registers into
(`validation/manifest.toml` is the source of truth) are described in the
[validation chapter](https://nsmith-.github.io/vibegraph/guide/12-validation.html).

| Layer | May assume | Command |
|---|---|---|
| `hermetic` | A bare clone — no submodule, no fetched data, no network. Complete there: it never skips. | `cargo test` |
| `banked` | The `mg5amcnlo` submodule, fetched PDF sets, and a pinned, checksummed bundle of frozen MadGraph reference runs. May *not* run MadGraph. | `pixi run validate` |
| `oracle` | A full MadGraph/LHAPDF toolchain; regenerates every reference. | `pixi run generate-references` |

`pixi run validate` ends by rendering a report table, one row per validated
process and one column per category, and asserts that the cells measured are
exactly the cells the manifest declares. **CI runs the hermetic and banked
layers as a merge gate and publishes the rendered report as each run's job
summary.** One banked gate needs its own environment:

```bash
pixi run -e pythia validate-pythia   # every emitted event read back by Pythia 8
```

The report's own account of what is *not* covered is the current list of open
validation items; the [backlog](https://nsmith-.github.io/vibegraph/backlog.html) carries them with the evidence — kept
as tracked rows, never as loosened tolerances.

## Performance

Every ratio below compares both sides **on one host, in one sitting**, on two
hosts:
- **x86:** a 4-vCPU Intel Ice Lake Xeon (KVM guest), with the v0.2.1 release
  binaries, plain x86-64 and x86-64-v3.
- **Apple M3 Max:** the v0.2.0 macOS release binary. Nothing performance-related
  changed for macOS between v0.2.0 and v0.2.1.
- **MadGraph:** the pinned 3.7.1, run through its own `launch` on the same host,
  with its default compiler flags set explicitly.

The full record, with every command, is in
[`research/kb/performance/`](research/kb/performance/index.md):
[matrix element](research/kb/performance/matrix-element-vs-madgraph.md),
[integration](research/kb/performance/integration-vs-madgraph.md),
[reweighting](research/kb/performance/reweighting-cost-vs-madgraph.md) and
[systematics](research/kb/performance/systematics-cost-vs-madgraph.md).

All ratios are cost ratios, ours over MadGraph's, so **below 1.0× is faster
than MadGraph**, unless a figure says "faster".

### The matrix element, per point

Our matrix element is evaluated by an interpreter: a helicity-amplitude program
built from the UFO model when the process is loaded, with no code generation or
compiler. Against MadGraph's generated, helicity-filtered Fortran
(`matrix1_optim.f`), over the 19 processes `scripts/mg_perf_compare.sh` covers,
2→2 through 2→6:

| host and build, both sides | geometric mean | range | slower than MadGraph |
|---|--:|--:|--:|
| x86, default x86-64 | **0.85×** | 0.64×–1.45× | 5 of 19 |
| x86, x86-64-v3 | **0.75×** | 0.57×–1.24× | 3 of 19 |
| M3 Max, default | **0.80×** | 0.57×–1.39× | 5 of 19 |

- **Fastest relative to MadGraph:** the multi-leg rows. The 2→4 rows run at
  0.57×–0.59× under x86-64-v3 and on the M3; the 2→6 rows at 0.61×–0.90×.
- **Slower:** `u u~ > u u~` (about 1.4×; 1.2× under x86-64-v3), `g g > g g`
  (1.2×) and `e+ e- > W+ W-` (1.1×–1.3×), where colour-flow contraction
  dominates.
- **x86-64-v3 helps us more than MadGraph**: about 1.24× faster for us against
  1.08× for MadGraph. This is still one point at a time, with no SIMD batching.

The starting point was 8.6×–110× slower: a general expression DAG walked per
point. Four things closed the distance, all of it holding the ≤ 1e-12
matrix-element gate throughout:
- compiling that DAG into a flat typed tape;
- expanding one straight-line program per contributing helicity combination;
- reproducing MadGraph's own two layers of zero-filtering, whose survivor sets
  match bit for bit;
- specialising the arithmetic itself.

### End to end: the integrand, not just the matrix element

A per-point ratio is not what anyone waits for, and the matrix element is not
the whole integrand. The phase-space map, the cuts, the scale draw and the
multichannel density sum `Σⱼ αⱼgⱼ(p)` are in every point too. Take the heaviest
rows, the 2→6 processes at 579 and 615 channels (Apple M3 Max figures):
- **A point that survives the cuts** costs 138 µs and 207 µs, of which the
  density sum is 36 and 38 µs (**26%** and **18%**).
- **Most points never get that far.** Acceptance on these rows is 3.6% and
  4.3%, and the density runs only after the cut and the matrix element. So the
  average *drawn* point costs 5.8 µs and 9.0 µs.

Both figures were an order of magnitude worse before the density moved behind
the cut and began sharing subtree momenta across channels.
`probe_2to6_eval_cost` and `probe_2to6_density_decomposition` in
[`validate_sigma.rs`](vibegraph-lib/tests/validate_sigma.rs) re-measure them.

The figures below are therefore the ones that describe running the generator.

**CPU-seconds to a target accuracy on σ.** This is the figure of merit that
folds in variance as well as per-point cost: the closest thing here to "how
long until I have the number".
- **Our side is measured:** `vibegraph integrate --target-rel 0.001 -j 1`,
  three seeds, CPU time including model load and diagram enumeration.
- **MadGraph's side is derived.** It takes the same host's MadEvent run of the
  same process, and scales its `<cumulated_time>` to the accuracy our run
  actually reached, by the 1/δ² Monte-Carlo law. So the whole extrapolation
  sits on its side and none on ours.
- **Errors** are the ones each run stops on, which three seeds' spread and the
  iterations' own scatter both confirm.

| process | x86, plain | x86, x86-64-v3 | M3 Max |
|---|--:|--:|--:|
| `e+ e- > mu+ mu-` | 0.12× | 0.11× | 0.06× |
| `p p > e+ e-` (dy13) | 0.16× | 0.15× | 0.08× |
| `g g > g g` | 0.76× | 0.66× | 0.48× |
| `p p > j j` | 0.67× | 0.60× | 0.42× |
| `p p > l+ l- j` | 0.88× | 0.80× | 0.62× |
| `u d > e+ e- u d` (2→4) | 0.56× | 0.47× | 0.26× |
| `p p > e+ e-` + 0–2 jets, MLM | 2.09× | 2.12× | 1.56× |
| **geometric mean** | **0.52×** | **0.47×** | **0.30×** |

**The same accuracy for about half of MadEvent's CPU on x86, and under a third
on the M3.** Without the MLM row the geometric means are 0.41×, 0.37× and 0.23×.
Every σ agrees with MadEvent's, the largest pull 1.47. The hosts differ by about
2× on every row, mostly on MadEvent's side: much of its `<cumulated_time>` is
per-job work that barely speeds up on a faster host.

Three rows need a closer reading:
- **`e+ e- > mu+ mu-` and `g g > g g` overshoot.** The default budget's minimum
  of six iterations already takes them past 0.1%, so the table compares equal
  accuracy at the δ actually reached. Measured to 0.1% with a smaller budget
  (`--min-iters 0 --neval 30000`), they read 0.19× and 0.81× on x86, and 0.10×
  and 0.49× on the M3.
- **`p p > l+ l- j` is phase-space map quality.** It draws 3.4× more points per
  second than MadEvent on x86 but needs several times as many. That is the map
  at the fiducial cut boundary, the next lever on this row.
- **The MLM row is the one where we are slower:** 2.1× on x86, 1.6× on the M3.
  Almost all of it is the two-jet part's cost per point, 485 µs against
  MadEvent's 104 µs. Both cluster about as often per point, but each of our
  clusterings costs about 4× MadEvent's, and we evaluate every flavour group's
  matrix element twice where MadEvent evaluates the one term it selects. The
  levers are estimated in
  [the diagnosis](research/kb/performance/mlm-integration-cost-vs-madevent.md);
  together they could take the row from 2.4× to between 1.2× and 1.7× (glibc
  build on x86). Merged multi-jet integration is the clearest open performance
  gap.

**Integrand throughput.** The per-point half of the same comparison: our
evaluations per single-core CPU-second against MadEvent's points per Fortran
CPU-second. **We draw 3.9× more points per second on x86** (4.4× at
x86-64-v3) **and 6.7× on the M3**, geometric mean over the seven rows. On x86 it
ranges from 0.88× on the MLM row and 1.5× on `p p > j j` to 28× on
`e+ e- > mu+ mu-`. Bigger is faster here, the opposite direction to the cost
ratios above.

**Many weights: reweighting and systematics.** These are where the per-event
cost of MadGraph's Python modules shows most. Both comparisons use the same
events on both sides:
- **Reweighting to an EFT operator basis** (8 SMEFTsim operators at NP=1, 45
  hypotheses, 1000 `e+ e- > t t~` events): MadGraph's reweight module takes
  72 s on x86 and 47 s on the M3. vibegraph adds 0.28–0.34 s on x86 and 0.13 s
  on the M3, or about a third of that reading all 45 off nine amplitude
  evaluations per event. That is **214×–708× faster on x86 and 351×–1015× on
  the M3**.
- **Systematics weights** (20 000 `p p > l+ l- j` events): CMS's 926-weight PDF,
  scale and αs request adds 38.6 s to `generate` on x86 and 14.6 s on the M3,
  against 455 s and 256 s for MadGraph's `systematics.py` (**11.8× and 17.5×**,
  at 806 MiB against 3.1 GiB). A 9-point scale request is 18× (x86) and about
  22× (M3).

Caveats on all of these:
- **Throughput is an integrand ratio, not a matrix-element one.** Our
  per-point work is not MadGraph's.
- **MadEvent's point count may include its survey pass.** Whether its
  recorded count does was never established, which puts a systematic factor of
  order unity on its throughput column.
- **One asymmetry is left uncorrected, and it is against us.** MadGraph's
  `cumulated_time` excludes its `output` and `compile` stages, while our model
  load and diagram enumeration sit inside our time.
- **The x86 integration ratios reuse the v0.2.0 MadEvent runs** from the same
  host, four days older than the v0.2.1 timings; the matrix-element,
  reweighting and systematics comparisons reran both sides together.
- **The M3 is a working desktop** (load average about 3, no core pinning on
  macOS), so its figures carry more timing noise than the x86 ones.
- **Use v0.2.1 or later on Linux.** The v0.2.0 Linux binaries ran slower with
  more threads (musl's allocator lock); v0.2.1 fixed it, and its binaries
  speed up 3.7× from `-j 1` to `-j 4` on this host
  ([record](research/kb/performance/musl-allocator-thread-scaling.md)).

**Parallel scaling** (Apple M3 Max). `vibegraph integrate -j 16` against
`-j 1` gives **8.68×** on Drell–Yan and **9.48×** on `p p > l+ l- j`.
- Thread count moves no bit. All twenty runs behind those figures wrote one
  artifact digest per card, so `-j 16` is the same number computed faster, not
  an approximation of it.
- The residual serial term is the α-adaptation survey.

**Unweighting.** `w_max` now comes from MadGraph's own `unwgt.f`
truncation-ladder rule. It used to be the weight scan's extremum, which a
Pareto tail of index ≈ 2 never lets converge. At matched budgets this raised
accept/reject efficiency on the five gating rows from 22.2 / 20.6 / 23.3 / 10.6
/ 4.21% to 54.1 / 52.1 / 52.9 / 38.9 / 9.98%. `p p > l+ l- j` needed 2 269 051
trials for 20 000 events before the change and 477 125 after: **4.36× cheaper
per effective event**. That is a ratio of trial counts, so it holds on any
host.

**The validation layer as a user runs it** (Apple M3 Max). `pixi run validate`,
the whole banked gate at 29 rows, went from 691 s to **341 s wall / 1 306 s
CPU** on a quiet host. Over the same window the suite itself grew by 42 running
tests.

More caveats:
- **These ratios are not constants.** They are measurements on one host, and
  the MadGraph denominator of the end-to-end ones moved by half between two
  machines. `scripts/mg_perf_compare.sh` re-derives the matrix-element table on
  any platform, and records the fingerprint that says both sides were built
  comparably.
- **Wall times are the noise-sensitive direction.** The figures above were
  taken with nothing else running, because a busy host has been worth
  double-digit percentages.
- **Pruned evaluators inherit MadGraph's frame contract:** partonic-CM
  momenta, with beams along ±z.

Candidate next steps are listed in the [backlog](https://nsmith-.github.io/vibegraph/backlog.html).

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
research/kb/          The knowledge bundle: topic concepts, backlog items, decisions,
                      sprint records, and the original notes under history/notes/
research/refs/        Reference code as submodules (mg5amcnlo, feyngraph) — see
                      research/refs/README.md
scripts/              Acceptance run, profiling and perf-comparison kits, and
                      the VHS tapes behind the quickstart recordings
docs/                 The documentation site (mdBook); scripts/build-docs.sh
                      assembles it with the API and CLI references
```

`research/kb/` is the project's record, an
[OKF](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)
knowledge bundle: one concept per convention, design, decision or measurement,
under topic folders, starting at [`research/kb/index.md`](research/kb/index.md).
The backlog is one file per open work item under `research/kb/backlog/`,
rendered as a single page by `pixi run backlog` and on the documentation site.
The original working notes are archived under `research/kb/history/notes/`.

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
