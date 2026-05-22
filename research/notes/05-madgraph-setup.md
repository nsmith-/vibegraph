# MadGraph5_aMC@NLO Setup via pixi/conda

## Package

| Field | Value |
|-------|-------|
| conda-forge package name | `mg5amcnlo` |
| Version (verified) | `3.5.7` |
| Platform | `osx-arm64` ✓ (native Apple Silicon) |
| Python ABI | `py311` (Python 3.11) |
| pixi search command | `pixi search mg5amcnlo` |

The package is available natively for `osx-arm64` — no Rosetta or Docker needed.
Verified by running `pixi search 'mg5*'` against the conda-forge channel.

### Key conda dependencies (pulled in automatically)
`six`, `numpy`, `lhapdf`, `ply`, `fastjet-contrib`, `hepmc2`, `hepmc3`,
`fortran-compiler`, `cxx-compiler`, `make`, `curl`, `wget`, `perl`,
`mg5amcnlo-pythia8-interface`, and various static loop libraries (`collier`, `iregi`,
`oneloop`, `qcdloop`).

No need to list these explicitly in `pixi.toml` — `mg5amcnlo` declares them as its
own conda dependencies and they will be resolved automatically.

---

## pixi Environment Setup

The `madgraph` feature is defined in `pixi.toml` as:

```toml
[feature.madgraph.dependencies]
python = "3.11.*"    # mg5amcnlo 3.5.7 builds against CPython 3.11
mg5amcnlo = "3.5.7"

[feature.madgraph.tasks]
generate-ee = "mg5_aMC research/mg5_scripts/ee_to_mumu.mg5"

[environments]
madgraph = { features = ["madgraph"], solve-group = "madgraph" }
```

### Install

```bash
pixi install -e madgraph
```

### Run

```bash
pixi run -e madgraph generate-ee
```

This executes the batch script `research/mg5_scripts/ee_to_mumu.mg5`, which:
1. Generates diagrams for `e+ e- > mu+ mu-` (SM, tree-level)
2. Writes the process to the `ee_to_mumu/` output directory
3. Launches the cross-section integration at √s = 91.2 GeV (ebeam1 = ebeam2 = 45.6 GeV)

Output appears in `ee_to_mumu/Events/run_01/` and `ee_to_mumu/crossx.html`.

To run at a different √s, edit the `ebeam1`/`ebeam2` values in the script or
pass a custom script:

```bash
# e.g. at √s = 200 GeV
pixi run -e madgraph mg5_aMC research/mg5_scripts/ee_to_mumu_200gev.mg5
```

---

## Measured Cross Sections (actual MadGraph run)

### e+ e- → μ+ μ- at √s = 91.2 GeV (Z pole) — **confirmed**

Run executed with `pixi run -e madgraph generate-ee` on 2026-05-22:

```
Cross-section :   2025 ± 1.116 pb   (= 2.025 ± 0.001 nb)
Nb of events  :  10000
```

Output files:
- `ee_to_mumu/Events/run_01/unweighted_events.lhe.gz` — 10 000 unweighted parton-level events in LHE format 3.0
- `ee_to_mumu/Events/run_01/run_01_tag_1_banner.txt` — full run parameters

The measured value is consistent with theory: the LO SM calculation at √s = M_Z includes
γ + Z + γ-Z interference at tree level with no ISR. The slightly higher value vs. the
peak formula (1.99 nb purely from Z) is expected because MG5 includes all contributing
diagrams and uses the full SM param_card.

> **Note:** The LEP measured value (with ISR, QED corrections, and experimental
> acceptance) was ≈ 1.45 nb for the visible Z lineshape peak.

### e+ e- → μ+ μ- at √s = 200 GeV (off-pole, LEP2 energy) — theoretical estimate

Off the Z resonance the cross section falls steeply:
- Pure QED (γ only): σ_QED = 4πα²/(3s) ≈ **2.2 pb**
- Full SM tree-level (γ + Z): Z-γ interference is constructive/destructive depending
  on angle; total ≈ **4–8 pb**

MadGraph at LO should give **≈ 5–7 pb** at √s = 200 GeV (not yet measured).

---

## Comparison with vibegraph

The goal is to validate vibegraph's LO amplitude against MadGraph:

| Observable | MadGraph (LO) | vibegraph (target) |
|------------|--------------|-------------------|
| σ(e+e-→μ+μ-, 91.2 GeV) | **2025 ± 1 pb** (measured 2026-05-22) | should match to <1% |
| σ(e+e-→μ+μ-, 200 GeV) | ~6 pb (estimate) | should match to <1% |
| dσ/d(cosθ) shape | 1 + cos²θ + AFB·cosθ | should match |

---

## Caveats

### Git checkout vs. release
The `research/refs/mg5amcnlo/` git submodule is a raw checkout and **cannot be run**
directly — MadGraph checks for a proper release tarball and exits with an error.
The conda package (`mg5amcnlo`) is a proper release build and works correctly.

### Output directory clobbering
MadGraph will refuse to overwrite an existing `ee_to_mumu/` directory.
Delete or rename it before re-running:

```bash
rm -rf ee_to_mumu/
pixi run -e madgraph generate-ee
```

### Fortran compiler requirement
`mg5amcnlo` depends on `fortran-compiler` from conda-forge (gfortran).
pixi resolves this automatically on `osx-arm64` via the conda environment.

### LHAPDF path
MadGraph bundles LHAPDF as a conda dependency. If you see PDF-related errors,
verify the LHAPDF data path:

```bash
pixi run -e madgraph python -c "import lhapdf; print(lhapdf.paths())"
```

### NLO features
The conda package includes NLO infrastructure (OneLoop, IREGI, Collier) but
vibegraph only targets LO. Use `generate e+ e- > mu+ mu-` (no `[QCD]` insertion)
to stay at tree level.

---

## Alternative Approaches (if conda package were unavailable)

1. **pip install**: `pip install 'mg5amcnlo'` — MG5 is installable via PyPI on some
   platforms, but binary extensions may fail on arm64.
2. **Rosetta / x86_64 environment**: Run under `arch -x86_64` with a separate
   conda environment using `CONDA_SUBDIR=osx-64`.
3. **Docker**: `docker run --platform linux/amd64 madgraph5amcnlo/mg5amcnlo:latest`
4. **Build from source**: Clone https://github.com/mg5amcnlo/MG5_aMC and run
   `./bin/mg5_aMC` from a proper release tag (not just any git commit).
