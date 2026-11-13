# vibegraph

<p align="center">
  <img src="assets/badger.png" width="420"
       alt="A European badger in a tie-dye t-shirt and wooden headphones, listening to a cassette player on a mossy forest floor at sunset." />
</p>

A **leading-order (LO) tree-level Monte Carlo event generator** written in Rust,
following the same pipeline as
[MadGraph5\_aMC@NLO](https://arxiv.org/abs/1405.0301) — and validated against it
at the strictest level each stage permits: bit-identical derived parameters and
couplings from the same UFO model and cards, machine precision on the matrix
elements, and statistical agreement on the emitted Les Houches event samples.

Per-point |M|² agrees with MadGraph's to ≤ 1e-12 relative, and cross sections and
event samples agree statistically with the banked MadGraph references. Reaching
0.1% on σ takes about half of MadEvent's CPU time on x86 and under a third on an
Apple M3 Max, and EFT reweighting runs about 200–1000× faster than MadGraph's
reweight module ([Performance](#performance)).

**Documentation:** <https://nsmith-.github.io/vibegraph/> — a guided tour of
how the generator works, stage by stage and paper by paper, the command-line
reference, and the `vibegraph-lib` API docs. Its source is [`docs/`](docs/)
and `scripts/build-docs.sh` builds it locally.

```
UFO model ──▶ diagram enumeration ──▶ helicity amplitudes (HELAS/ALOHA)
                                              │
        LHEF events ◀── unweighting ◀── σ integration ◀── phase-space sampling
                                                          (multichannel VEGAS)
```

## Quickstart

Two cards and two commands. A **process card** picks the model and the process,
a **run card** sets the beams, cuts and scales; `vibegraph integrate` adapts the
phase-space grids and prints σ, and `vibegraph generate` unweights against those
frozen grids into a Les Houches event file. Both cards are MadGraph's own
formats, so the same pair drives a MadGraph reference run unchanged.

### Get the binary

Every [release](https://github.com/nsmith-/vibegraph/releases) attaches
precompiled bare executables — nothing to unpack, and none needs a Rust
toolchain, a Python installation, or LHAPDF. The Linux builds are statically
linked. `./vibegraph --version` prints the license notices for the binary and
the Standard Model description compiled into it (`-V` for just the version).

```bash
# asset: vibegraph-aarch64-apple-darwin      (macOS, Apple Silicon)
#        vibegraph-x86_64-apple-darwin       (macOS, Intel)
#        vibegraph-x86_64-unknown-linux-musl     (Linux, any x86_64)
#        vibegraph-x86_64-unknown-linux-musl-v3  (Linux, x86-64-v3)
asset=vibegraph-aarch64-apple-darwin
curl -fsSL -o vibegraph https://github.com/nsmith-/vibegraph/releases/latest/download/$asset
chmod +x vibegraph
./vibegraph -V
```

`…-musl-v3` is the same code compiled for
[x86-64-v3](https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels)
(AVX2/FMA, roughly Haswell and later), which speeds up the amplitude
evaluation. It dies with an illegal instruction on older CPUs, so take the
plain build if unsure — this prints `supported` when the `-v3` asset will run:

```bash
grep -qw avx2 /proc/cpuinfo && grep -qw fma /proc/cpuinfo \
  && echo supported || echo "use the baseline build"
```

To build from a checkout instead:

```bash
git clone --recurse-submodules https://github.com/nsmith-/vibegraph
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

Nothing sets an iteration count: the run stops when σ's relative uncertainty
reaches 0.1%, so the pane's progress bar counts against a projected total that
is re-estimated every iteration
([the status pane](https://nsmith-.github.io/vibegraph/cli/overview.html#watching-a-run));
`--fixed-budget` trades the target for a set number of iterations.

`p`, `j` and `l+ l-` are MadGraph's multiparticle labels, and coupling-order
constraints belong to the same grammar (`generate p p > l+ l- j QCD=2 QED=2`).
Subprocesses that share a matrix element are grouped and served by one compiled
program ([flavour groups](https://nsmith-.github.io/vibegraph/guide/10-hadronic.html#flavour-groups)).

### Data the binary does not carry

The Standard Model is compiled in, so the leptonic run above needs nothing
else. PDF sets (for proton beams) and non-SM UFO models are resolved on
demand, from a flag, an environment variable, the `~/.vibegraph/` cache or
the working directory, in that order. A PDF set is the only input that can
involve the network: a missing one is downloaded only with consent, against a
checksum compiled into the binary; UFO models are never downloaded. The
resolution order, the consent rules (`-y`, `--no-network`) and the cache layout
are documented under
[Data the binary does not carry](https://nsmith-.github.io/vibegraph/cli/overview.html#data-the-binary-does-not-carry).
`scripts/acceptance.sh` runs the whole proton path on a clean machine.

## What is supported, and what is refused

Leading-order event generation for tree-level processes at any leg
multiplicity, with MLM matching: a UFO model and MadGraph's own process and run
cards go in, an unweighted `.lhe` event sample comes out.

**Anything the cards ask for that is not implemented is refused by name, not
ignored.** Process-card features are refused before diagram enumeration;
run-card settings and cuts when `integrate` starts.

| Surface | Supported | Refused by name | More |
|---|---|---|---|
| Process grammar | MadGraph's: multiparticle labels, coupling orders, `add process` with `@N`, decay chains (`p p > t t~, t > b e+ ve`) stitched onto their core legs, 1→n decays to a partial width, polarized external legs (`w+{0}`, `e-{L}`) and polarized cross sections. The s-channel restrictions `/`, `>`, `$` and `$$`: `>` and `$$` filter diagrams, `$` is MadEvent's zeroed on-shell propagator. Diagrams by [feyngraph](https://github.com/Jens-Braun/FeynGraph), checked against MadGraph's diagram counts and its own generation on censuses of proc cards | squared-order constraints; polarized intermediate resonances; one-body final states (`p p > z'` with no decay); `add process` lines of different multiplicity at fixed-energy beams (at proton beams they are summed, the way MadEvent sums its `P<n>` directories) | [grammar](https://nsmith-.github.io/vibegraph/guide/03-diagrams.html#the-process-grammar) |
| Matching | MLM of mixed jet multiplicities (`ickkw = 1`, `xqcut`): MadEvent's cut, clustering scales and α_s/PDF reweighting, and the `<scales>` record Pythia's kT-MLM matching reads | MLM at fixed-energy beams or on decays | [hadronic](https://nsmith-.github.io/vibegraph/guide/10-hadronic.html) |
| Scales, couplings, cuts | MadGraph's α_s RGE, per-event μR and per-beam μF prescriptions, including the default kT-clustered dynamical scale (`dynamical_scale_choice = -1`) reproduced against MadGraph's own clustering; MadGraph's run-card cuts, compiled | photon isolation; a card that leaves a soft or collinear divergence uncut (naming the legs and the cut that would regulate it) | [scales](https://nsmith-.github.io/vibegraph/guide/10-hadronic.html#scales) |
| UFO models | Any UFO model, read by a Python-AST parser; restrict cards baked into parameters; model identity (label + SHA-256 over the parsed model) banked into artifacts. SMEFTsim's `topU3l_MwScheme` and two small models written here are gated against MadGraph's own matrix elements | spin-3/2 and spin-2 particles; Majorana fermions and charge conjugation; custom UFO propagators and Fortran operators; coupling orders in a model without `coupling_orders.py`; loop-level UFOs (out of the LO charter) | [UFO chapter](https://nsmith-.github.io/vibegraph/guide/02-ufo.html#beyond-the-standard-model), [`manifest.toml`](validation/manifest.toml) |
| Amplitudes | HELAS-style evaluation compiled directly from UFO Lorentz structures (the ALOHA role), one program per contributing helicity combination, with MadGraph-matched helicity filtering. Matrix elements are evaluated in the partonic CM frame, with beams along ±z | the coloured-scalar seagull vertex (so squark-pair production) | [amplitudes](https://nsmith-.github.io/vibegraph/guide/04-helicity-amplitudes.html) |
| Colour | Exact colour-factor \|M\|² via per-flow JAMPs; colour sextets and baryonic epsilon tensors (`Epsilon`/`EpsilonBar`, `K6`/`K6Bar`/`T6`, `ColorRep::Sextet`), each gated against MadGraph on a row of its own | a `T6` carrying adjoint indices; any colour basis in which such a tensor survives to an external leg, which no Les Houches record can write | [colour](https://nsmith-.github.io/vibegraph/guide/05-color.html) |
| Beams, phase space | Unpolarized proton–proton, including unequal beam energies and massive incoming partons, or fixed-energy partonic beams. At proton beams, flavour groups summed over both beam orderings. Multichannel VEGAS, one grid per diagram channel; decay chains inside MadEvent's Breit–Wigner windows | any other beam configuration | [multichannel](https://nsmith-.github.io/vibegraph/guide/09-multichannel.html) |
| Weights | MadGraph reweight cards; scale, PDF and α_s variations in MadGraph's systematics format | — | [CLI](https://nsmith-.github.io/vibegraph/cli/overview.html) |
| Output | Unweighted LHEF at fixed-energy and proton beams; per-event helicity and colour-flow selection following MadEvent's rules, the flow→`ICOLUP` dictionary checked against MadGraph's `leshouche.inc`; `SCALUP`/`AQCDUP`; decay-chain resonance records and one `<init>` entry per `@N`, as MadEvent writes them; an LHEF writer and reader that round-trip MadGraph's own event files byte for byte | MadSpin decays | [events](https://nsmith-.github.io/vibegraph/guide/11-unweighting-events.html) |

Open validation items are tracked in the
[backlog](https://nsmith-.github.io/vibegraph/backlog.html), and the design
record in [`research/kb/`](research/kb/index.md).

**Future scope may include**: the rest of the arbitrary-BSM-UFO surface — the
boundary checklist already lives in the backlog's items tagged `non-sm-ufo` — MadSpin-style
decays, CKKW-L merging, and NLO event generation.

### Differences from MadGraph

Where vibegraph knowingly gives a different answer from MadGraph (the
[v0.2.1 release notes](docs/release-notes/v0.2.1.md) carry the current list):
- **Identical particles across decays.** vibegraph keeps every pairing, their
  interference and the union of their decay windows; MadGraph keeps one pairing.
  σ reads +0.6% ± 0.2% on `g g > z u u~, z > u u~` (`cut_decays = F`) and
  +0.25% ± 0.08% on `e+ e- > z z, z > e+ e-`, of which +0.14% is not yet
  attributed.
- **Widths below 1e-6 of the mass** keep the model's width, where MadEvent
  floors it: 1.4% on one decay-chain card.
- **Open measurements:** the `p p > l+ l- j` family reads +0.15% to +0.23% above
  MadEvent's seed records, and `p p > b b~` drifts from MadEvent as the budget
  grows. Neither is attributed yet.
- **`-o events.lhe.gz`** (or `.zst`) writes an uncompressed file under that
  name; the extension is not acted on.

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

Ratios are costs, ours over MadGraph's, so **below 1.0× is faster than
MadGraph** unless a figure says "faster". Both sides run on the same host: a
4-vCPU Intel Ice Lake Xeon (KVM guest) with the v0.2.1 binaries, and an Apple
M3 Max with the v0.2.0 macOS binary (nothing performance-related changed for
macOS in v0.2.1), against the pinned MadGraph 3.7.1 run through its own
`launch` with its default compiler flags set explicitly.

| measure | x86, plain / x86-64-v3 | M3 Max |
|---|--:|--:|
| CPU time to 0.1% on σ, geometric mean over 7 processes | **0.52×** / **0.47×** | **0.30×** |
| matrix element per point, geometric mean over 19 processes | 0.85× / 0.75× | 0.80× |
| EFT reweighting, 45 hypotheses on 1000 `e+ e- > t t~` events | 214×–708× faster | 351×–1015× faster |
| systematics, 926 weights on 20 000 `p p > l+ l- j` events | 11.8× faster | 17.5× faster |

Time to 0.1% on σ, per process:

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

Every σ agrees with MadEvent's, the largest pull 1.47. **The MLM row is the one
open gap**: almost all of it is the two-jet part's cost per point, 485 µs against
MadEvent's 104 µs, because each of our clusterings costs about 4× MadEvent's and
we evaluate every flavour group's matrix element where MadEvent evaluates the one
term it selects ([diagnosis and levers](research/kb/performance/mlm-integration-cost-vs-madevent.md)).
The matrix element is slowest relative to MadGraph on `u u~ > u u~`, `g g > g g`
and `e+ e- > W+ W-` (1.1×–1.4×), where colour-flow contraction dominates.

- **Our time is measured, MadGraph's derived:** ours is `integrate --target-rel
  0.001 -j 1` over three seeds, model load and diagram enumeration included;
  MadEvent's `<cumulated_time>`, which excludes `output` and `compile`, is scaled
  by 1/δ² to the accuracy our run reached. `e+ e- > mu+ mu-` and `g g > g g`
  overshoot 0.1% at the six-iteration minimum and compare at the δ reached.
- **The ratios are measurements, not constants:** they differ by about 2× between
  the two hosts, mostly on MadEvent's side.
- **Use v0.2.1 or later on Linux:** v0.2.0's musl builds slowed down with more
  threads ([record](research/kb/performance/musl-allocator-thread-scaling.md)).

Every command, the remaining caveats, integrand throughput, per-point cost
breakdowns and thread scaling are in
[`research/kb/performance/`](research/kb/performance/index.md)
([matrix element](research/kb/performance/matrix-element-vs-madgraph.md),
[integration](research/kb/performance/integration-vs-madgraph.md),
[reweighting](research/kb/performance/reweighting-cost-vs-madgraph.md),
[systematics](research/kb/performance/systematics-cost-vs-madgraph.md));
unweighting efficiency is in [`events/unweighting.md`](research/kb/events/unweighting.md).
`pixi run profile-sigma` records a samply profile of the σ gate.

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

Three more commands: `vibegraph check-events` reads an emitted file back and
checks it against itself; `vibegraph enumerate` writes a process's diagrams (an
artifact `integrate` can start from, a summary and one SVG drawing per diagram);
and `vibegraph inspect` prints what a saved integration records, or its
configuration ranking as JSON. `vibegraph <cmd> --help` has the full option list, and the
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

## Repository layout

```
vibegraph-lib/        Library: ufo/, diagrams/, helas/, phasespace/, vegas,
                      coupling/, pdf/, hadronic, proton, cuts, unweight,
                      lhef/, artifact
vibegraph-cli/        The `vibegraph` binary (integrate, generate, check-events,
                      enumerate, inspect)
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
