# MadGraph reference data

The MadGraph5_aMC@NLO side of the validation suite: the batch scripts that
produce the reference runs, the generators that reduce those runs to committed
reference files, and the work area both live in.

**The per-process list lives in [`../manifest.toml`](../manifest.toml), not here.**
One row per process: the process string, the script that builds it, which
validation categories cover it, in which dependency layer, and why the process is
in the set at all. Adding a process means adding a row there and a script in
`scripts/` — this file deliberately carries no second copy of that list.

## Layout

```
scripts/*.mg5             one batch script per reference run
wrappers/*.f              Fortran shims f2py compiles against the generated code
build.sh                  runs the scripts that have no output directory yet
build_amplitude.sh        compiles matrix elements into f2py modules
gen_*.py, extract_*.py    reduce the work area to the committed reference files
assemble_bundle.sh        packs the banked outputs into the fetchable bundle
output/                   the work area (gitignored, ~1 GB)
  <process>/                a MadGraph process directory
  <process>.json            diagram counts + configs.inc topologies
  <process>_amplitude.csv   |M|² on the fixed kinematic grid
  f2py/                     compiled matrix-element extension modules
  bundle/                   the assembled banked-reference archive
```

Committed reference files, each the output of one generator:

| file | generator | consumed by |
|---|---|---|
| `diagrams.json` | `extract_diagrams.py` | the diagram-count gate |
| `sigma_reference.json` | `extract_sigma.py` | the fixed-energy σ gate |
| `amp_reference.json` | `gen_amp_reference.py` | the per-diagram AMP gate |
| `jamp_reference.json` | `gen_jamp_reference.py` | the per-flow JAMP gate |
| `hadronic_sigma_reference.json` | `gen_hadronic_sigma.sh` | the hadronic σ gate |
| `dy_integrand_oracle.json` | `gen_dy_oracle.sh` | the pointwise Drell-Yan oracle |
| `runcard_defaults.json` | `dump_runcard_defaults.py` | the run-card defaults transcription |
| `dy13_*_card.dat` | copied verbatim into the runs | both sides of the hadronic σ gate |

## Regenerating

One entry point, staged, over every generator here and in the sibling
directories:

```sh
pixi run generate-references              # deps → madgraph → refs → bundle
pixi run generate-references refs bundle  # a subset
```

**The work area is the cache.** A process directory that exists is never
rebuilt, a compiled f2py module that exists is never recompiled, and the banked
cross-section runs are not repeated once their answer is written; `VG_FORCE=1`
overrides. The extraction steps are cheap and pure functions of the work area, so
they always rerun — which is what makes a reference that moved show up as a diff.

Regenerating from a populated work area reproduces every committed file
byte-for-byte, the bundle archive included.

## Using the reference data without MadGraph

Everything the banked validation layer reads is either committed here or in the
bundle `assemble_bundle.sh` builds — the event files with their banners and logs,
the cards, each subprocess's `leshouche.inc` and `matrix1_orig.f`, the combined
`results.dat`, and the amplitude tables. It unpacks *into* `output/`, so a
checkout that fetched it and a machine that generated the runs present the gates
with identical paths:

```sh
pixi run fetch-refdata      # pinned URL + SHA-256 from ../manifest.toml
pixi run --skip-deps validate
```

`VIBEGRAPH_REFDATA_SOURCE=/path/to/vibegraph-refdata-1.tar.zst` takes the archive
from a local file instead; the pinned checksum is enforced either way.

## Coupling-order semantics

`ORDERS.md` derives what MadGraph's default and explicit coupling-order
constraints select, and which scripts pin which half of that behaviour.
