# vibegraph

<p align="center">
  <img src="assets/badger.png" width="420"
       alt="A whimsical photograph of a blissful European badger seated cross-legged on a mossy forest floor at sunset. The badger is dressed in a vibrant tie-dye t-shirt, wears large wooden headphones, and listens to music on a vintage cassette tape player. Its eyes are contentedly closed, soaking in the moment, surrounded by ferns, glowing mushrooms, small daisies, a sprig of lavender, and a dreamcatcher hanging from a tree branch above. The soft, filtering golden light of the woods creates a warm and peaceful atmosphere." />
</p>

A toy **leading-order (LO) tree-level Monte Carlo event generator** written in
Rust, following the same pipeline used by
[MadGraph5\_aMC@NLO](https://arxiv.org/abs/1405.0301).

## What it does (eventually)

1. **Load a UFO model** — parse the Python-format
   [Universal FeynRules Output](https://arxiv.org/abs/1108.2040) files
   (`particles.py`, `vertices.py`, `lorentz.py`, …) to build an internal
   representation of particle content, Feynman rules, and coupling constants.

2. **Enumerate Feynman diagrams** — given an initial and final state, find all
   tree-level diagrams by recursive vertex matching (à la MadGraph) or using
   the [FeynGraph](https://github.com/Jens-Braun/FeynGraph) crate.

3. **Evaluate helicity amplitudes** — implement
   [HELAS](https://inspirehep.net/literature/336604)-style wavefunction and
   vertex routines; generate calls automatically from UFO Lorentz structures
   via [ALOHA](https://arxiv.org/abs/1108.2041).

4. **Sample phase space** — map the unit hypercube to physical four-momenta
   and importance-sample with [VEGAS](https://inspirehep.net/literature/119196).

5. **Compute the cross section** — integrate |M|² over phase space; produce
   an unweighted event sample.

### Toy process

**e⁺e⁻ → μ⁺μ⁻** via a single photon / Z propagator — the simplest
non-trivial 2→2 process, with an analytic check: σ = 4πα²/3s (massless limit).

## Status

Early research and infrastructure phase. See [`research/notes/`](research/notes/)
for working notes and paper summaries.

## Repository layout

```
src/                  Rust source (not yet started)
research/
  notes/              Working notes and algorithm derivations
    00-overview.md      Pipeline walkthrough and references
    01-paper-summaries.md  Quick-reference summaries of key papers
    02-reference-implementations.md  Analysis of reference submodules
  refs/               Reference code (git submodules)
    feyngraph/          FeynGraph Rust diagram generator (Jens-Braun/FeynGraph)
    mg5amcnlo/          MadGraph5 v3.7.1 (HELAS, ALOHA, SM UFO)
  ufo/                Sample UFO model files
assets/               Images and other static assets
```

## Getting started

```bash
git clone --recurse-submodules <repo-url>
cd vibegraph
cargo build
```

## Command line

### `vibegraph integrate` — hadronic cross section

Computes the leading-order Drell–Yan cross section σ(pp → e⁺e⁻) and saves the
adapted [VEGAS](https://inspirehep.net/literature/119196) grid so a later
sampling phase can reuse it:

```bash
vibegraph integrate <proc_card> [--run-card <run_card.dat>] [--out <dir>]
```

The proc card supplies the model import (`import model sm`) and must describe
`p p > e+ e-` — the only process wired through this pipeline so far. Beam energy
and the fixed factorization scale μF are read from the MadGraph `run_card.dat`
(omit `--run-card` for the MadGraph LO defaults), so the same card file can
drive both this integration and a MadGraph reference run.

The command prints `σ ± err` (pb) and writes `<out>/grid.bin.zst` — a
bincode + zstd artifact holding the trained grid plus the run metadata (process,
PDF set, μF, √s, seed, evaluation counts, and the resolved run card) that a
downstream sampling phase needs to detect a mismatched input. It refuses to
overwrite an existing artifact without `--force`.

The PDF grid files are fetched separately (they are gitignored):

```bash
pixi run -e madgraph fetch-pdf          # into validation/pdf/<set>/
vibegraph integrate validation/madgraph/dy13_proc_card.dat \
  --run-card validation/madgraph/dy13_default_run_card.dat --out run/
```

`vibegraph integrate` locates the PDF data via `--pdf-dir`, else the
`VIBEGRAPH_PDF_DIR` environment variable, else `validation/pdf` under the
current directory; `--pdf-set` selects the set (default
`NNPDF23_lo_as_0130_qed`). Key options: `--neval` / `--niter` (VEGAS evaluations
per iteration / iteration count) and `--seed` (reproducible RNG). Run
`vibegraph integrate --help` for the full list.

## Validation and profiling

Two MadGraph-referenced nets guard the physics (both need the gitignored
`validation/madgraph/output/` reference tree):

```bash
pixi run -e madgraph validate-helas-mg   # per-point bit-exact |M|^2 net
pixi run -e madgraph validate-sigma      # cross-section (sigma) gate
```

`validate-sigma` drives each fixed-energy process through the integration path
with its real `run_card.dat` and compares the integrated `σ` against the banked
MadGraph value (`validation/madgraph/sigma_reference.json`, regenerated by
`pixi run -e madgraph extract-sigma`). It catches what the bit-exact net cannot:
flux, spin/colour averaging, symmetry factors, cuts, and beam handling.

For performance, profile the sigma gate — its per-process time is weighted by how
hard each process is to *integrate*:

```bash
pixi run -e madgraph profile-sigma       # samply record of the profiling build
```

To regenerate the HELAS OCR output from the scanned PDF:

```bash
pixi run -e nougat ocr
```

See [`research/refs/README.md`](research/refs/README.md) for details on the
reference materials and OCR workflow.
