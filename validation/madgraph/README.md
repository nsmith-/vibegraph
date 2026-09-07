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
cards/<family>/           restrict cards this repository authors for a vendored
                            UFO model, staged into the work-area copy by build.sh
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
  models/                   work-area copies of the vendored UFO models a script
                              may import, with the authored cards added
  f2py/                     compiled matrix-element extension modules
  bundle/                   the assembled banked-reference archive
  ktdump/                   instrumented replays of the banked runs, and the
                              per-event clustering dumps they produce
```

Beside the one directory per `.mg5` script, the work area also holds the
bespoke runs some references are measured from — the two `dy13_*` Drell-Yan
cards and the `ee_to_mumu_tata_qcd0_h*` Higgs-window runs. They are named after
what they measure rather than after a script, and only the generators that ask
for them by name read them; `extract_diagrams.py` skips them for that reason.

**Which model.** A script that names no model is generated against MadGraph's
built-in `sm`. A script that names one imports it as
`import model validation/ufo/<model>-<card>`, repository-relative, and `build.sh`
rewrites that path to the copy it stages under `output/models/` — MadGraph writes
a cached pickle into a model directory it imports and the vendored directories
are committed byte for byte, so neither generator reads `validation/ufo/` in
place. The cards the model ships travel with the copy; the ones this repository
authors are added to it from `cards/`. The manifest row's `model` and `restrict`
fields say the same thing to the Rust side, and the gates compare the two.

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
| `interactions.json` | `extract_interactions.py` | model topology per (UFO model, restrict card) |
| `sigma_reference.json` | `extract_sigma.py` | the fixed-energy σ gate |
| `amplitudes/<process>.json` | `gen_amplitude_tables.py` | the amplitude gate |
| `hadronic_sigma_reference.json` | `gen_hadronic_sigma.sh` | the hadronic σ gate |
| `dy_integrand_oracle.json` | `gen_dy_oracle.sh` | the pointwise Drell-Yan oracle |
| `runcard_defaults.json` | `dump_runcard_defaults.py` | the run-card defaults transcription |
| `dy13_*_card.dat` | copied verbatim into the runs | both sides of the hadronic σ gate |
| `higgs_window_reference.json` | `gen_higgs_window.sh` | the h → ττ pole window measurement |
| `kt_cluster_dump_manifest.json` | `gen_kt_cluster_dumps.sh` | what the kT-clustering dumps are, and their checksums |

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

**A banked event sample is not regenerable from its cards for a run whose
process directory holds more than one subprocess group.** `VG_FORCE=1` will
re-run the card and the cross section comes back identical to every printed
digit, but the unweighting draw is sensitive to how the groups' jobs are
scheduled, so the events are a different — equally valid — sample. `p p > j j`
is the measured case: five groups, and a re-run yields a different event file.
Its banked sample is the reference, and the `samples` gate on it compares
distributions rather than bytes. Single-group runs do regenerate bit-identically,
which is what the clustering dumps below rely on.

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

`VIBEGRAPH_REFDATA_SOURCE=/path/to/vibegraph-refdata-4.tar.zst` takes the archive
from a local file instead; the pinned checksum is enforced either way. It is also
the only route while `[refdata].published` is `false`: a bundle whose release
asset has not been uploaded yet has a pin but no URL that serves it.

## The kT-clustering dumps

`dynamical_scale_choice = -1` gives no closed form for anything past a 2 → 2, and
MadGraph writes no `<clustering>` tag at `ickkw = 0`, so the merge sequence a
banked event's scale came out of is not recoverable from the event file. The
oracle for it is an instrumented replay: `gen_kt_cluster_dumps.sh` regenerates
each banked row's process directory, patches the clustering sources MadGraph
copied into it (`wrappers/ktdump_*.patch`, applied only after the copies are
confirmed byte-identical to the pinned template), installs the banked cards with
the seed the banked banner records, and re-runs it.

Two things make the dump worth reading, and both are checked rather than assumed:
the replay's unweighted event file is byte-identical to the bank over every
event, and every event's dumped μR and μF reproduce that event's own `SCALUP`,
`<rscale>` and `<pdfrwt>`. The dumps themselves are work-area sized (75 MB) and
live under `output/ktdump/dumps/`; the committed manifest pins their checksums.

The reference bundle does **not** carry them, and the consequence is not a
detail: `validate_kt_cluster` returns early when the dumps are absent, so on a
checkout that fetched the bundle that gate is green without having compared
anything. Whether the dumps join a bundle or the gate moves to the oracle layer
is an open decision (`TODO.md`, gate + tooling hygiene).

## Coupling-order semantics

`ORDERS.md` derives what MadGraph's default and explicit coupling-order
constraints select, and which scripts pin which half of that behaviour.
