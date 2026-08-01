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
mg5_pinned.sh             runs the *pinned* mg5_aMC, which every generator uses
build.sh                  runs the scripts that have no output directory yet
build_amplitude.sh        compiles matrix elements into f2py modules
gen_*.py, gen_*.sh,       reduce the work area to the committed reference files,
extract_*.py                or drive the bespoke runs some of them read
assemble_bundle.sh        packs the banked outputs into the fetchable bundle
output/                   the work area (gitignored, ~1 GB)
  <process>/                a MadGraph process directory
  <process>.json            diagram counts + configs.inc topologies
  <process>_amplitude.csv   |M|² on the fixed kinematic grid
  f2py/                     compiled matrix-element extension modules
  bundle/                   the assembled banked-reference archive
```

Beside the one directory per `.mg5` script, the work area also holds the
bespoke runs some references are measured from — the two `dy13_*` Drell-Yan
cards and the `ee_to_mumu_tata_qcd0_h*` Higgs-window runs. They are named after
what they measure rather than after a script, and only the generators that ask
for them by name read them; `extract_diagrams.py` skips them for that reason.

**Which MadGraph.** Every run in this directory comes from the pinned
`research/refs/mg5amcnlo` submodule by way of `mg5_pinned.sh`, not from whatever
`mg5_aMC` a `pixi run -e madgraph` shell puts on `PATH` (packaged 3.5.7). That
script says why the version is load-bearing; the short form is that the 3.5.x
`sde_strategy = 2` multichannel weight never peaks on a resonance, so a run of a
narrow-resonance process misses its own pole. The exception is deliberate: the
`ee_to_mumu_tata_qcd0_h*` runs are the *evidence* for that defect and were taken
with 3.5.7 on purpose.

Committed reference files, each the output of one generator:

| file | generator | consumed by |
|---|---|---|
| `diagrams.json` | `extract_diagrams.py` | the diagram-count gate |
| `sigma_reference.json` | `extract_sigma.py` | the fixed-energy σ gate |
| `amplitudes/<process>.json` | `gen_amplitude_tables.py` | the amplitude gate |
| `hadronic_sigma_reference.json` | `gen_hadronic_sigma.sh` | the hadronic σ gate |
| `dy_integrand_oracle.json` | `gen_dy_oracle.sh` | the pointwise Drell-Yan oracle |
| `runcard_defaults.json` | `dump_runcard_defaults.py` | the run-card defaults transcription |
| `dy13_*_card.dat` | copied verbatim into the runs | both sides of the hadronic σ gate |
| `higgs_window_reference.json` | `gen_higgs_window.sh` | the h → ττ pole window measurement |

## Regenerating

One entry point, staged, over every generator here and in the sibling
directories:

```sh
pixi run generate-references              # deps → madgraph → refs → bundle
pixi run generate-references refs bundle  # a subset
```

**The work area is the cache.** A process directory that exists is never
rebuilt, a compiled f2py module that exists is never recompiled, and a
cross-section run that already banked its events and its `results.dat` is read
rather than repeated; `VG_FORCE=1` overrides. The extraction steps are cheap and pure functions of the work area, so
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

`VIBEGRAPH_REFDATA_SOURCE=/path/to/vibegraph-refdata-3.tar.zst` takes the archive
from a local file instead; the pinned checksum is enforced either way. It is also
the only route while `[refdata].published` is `false`: a bundle whose release
asset has not been uploaded yet has a pin but no URL that serves it.

## Coupling-order semantics

`ORDERS.md` derives what MadGraph's default and explicit coupling-order
constraints select, and which scripts pin which half of that behaviour.
