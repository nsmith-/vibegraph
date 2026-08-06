---
name: extended-validation
description: The map from a change to the validation gates it must clear. Invoke after touching amplitudes, colour, couplings, diagram enumeration, phase space, sampling/integration, PDFs, scales, or the LHEF writer — and before claiming any of those validated. Covers the three dependency layers, which pixi task runs which gate, when a committed reference must be regenerated, and what `--skip-deps` does.
---

# Extended validation

`cargo test` is not the gate for physics changes. It is the *hermetic* layer —
complete and skip-free, but it can only see what a bare clone can compute. Every
comparison against MadGraph lives in a heavier layer that this skill maps.

`validation/manifest.toml` is the authority: one row per validated process, a
cell per category (`diagrams`, `amplitudes`, `integrals`, `samples`), and each
cell's tier and mode. When this file and the manifest disagree, the manifest is
right and this file is stale — fix it.

## The three layers

A check declares its layer by **where it is registered**, never by deciding at
runtime whether it has inputs.

| Layer | May assume | Driven by |
| --- | --- | --- |
| `hermetic` | nothing — bare clone, no submodule, no network, no pixi | `cargo test --workspace` |
| `banked` | `mg5amcnlo` submodule, fetched PDF sets, frozen MadGraph runs. May **not** run MadGraph | `pixi run validate` |
| `oracle` | `mg5_aMC`, LHAPDF, f2py — all reference generation, plus runs too heavy for the banked budget | `pixi run -e madgraph generate-references`, `pixi run validate-deep` |

Banked membership is spelled `required-features = ["extended-validation"]` on the
test target in its crate's `Cargo.toml`, so the test is *absent* from a default
build rather than silently inert in it. The `--lib` target has no per-test
registration, so the few library unit tests needing banked inputs use
`#[cfg(feature = "extended-validation")]` instead; integration tests never do.

The banked layer takes no runtime skips. Its inputs arrive through `pixi run
validate`'s dependency tasks, each of which fails when it cannot acquire what it
names; a gate that still finds an input missing calls
`vibegraph::validation::require` and fails naming it.

## The two commands you will use most

```bash
cargo test --workspace          # hermetic; the floor for every change
pixi run validate               # the banked layer end to end + the report collator
```

`pixi run validate` fetches (submodule, both PDF sets, the refdata bundle), runs
every banked gate under `--profile release-debug`, then collates
`target/validation-report/report.md` and fails if the rendered cells are not the
ones `validation/manifest.toml` declares. It is the merge gate CI runs.

Once those inputs are on disk, skip the re-fetch:

```bash
pixi run --skip-deps validate
```

## `--skip-deps`

Most validation tasks chain `depends-on` steps that *regenerate reference data* —
building MadGraph process directories, compiling Fortran through f2py, relinking
`alfas_functions.f`. That regeneration dominates the wall time and is what turns a
minutes-long gate into an hours-long one.

`pixi run --skip-deps <task>` runs the final step alone. Use it when the generated
inputs are known fresh — you changed only Rust, or you are rerunning the same task
against the same reference. Do **not** use it after bumping the submodule,
editing a run card, changing the process list, or touching anything under
`validation/madgraph/`: those are exactly the cases where the dependency step has
work to do, and skipping it compares against a stale reference that will agree
with the wrong thing.

## Change → gate map

Run the hermetic suite for everything. On top of it:

| You touched | Run |
| --- | --- |
| `helas/` representations, kernels, amplitude evaluation | `pixi run -e helas-validation validate-helas`, then `pixi run -e madgraph validate-amplitudes` |
| diagram enumeration, subprocess classification | `pixi run -e madgraph validate-diagrams` |
| `helas/color/`, the flow root, anything reordering the colour basis | `pixi run -e madgraph validate-color-cf` **and** `validate-color-flow-tags` **and** `validate-amplitudes` |
| `coupling/` — αs RGE, μR/μF | `pixi run -e madgraph validate-alphas`, `validate-scales`, `validate-scale-couplings` |
| kT clustering, `coupling/cluster/` | the three above, plus the oracle-layer `pixi run -e madgraph validate-kt-cluster` |
| phase space, channel maps, VEGAS, multichannel budgets | `pixi run -e madgraph validate-sigma`, `validate-hadronic` |
| unweighting, event selection | `pixi run -e madgraph validate-unweighting`, `validate-generate-proton` |
| the LHEF writer/reader, colour selection in output | `pixi run -e madgraph validate-lhef`, plus `pixi run -e pythia validate-pythia` |
| PDF grids, interpolation, the subgrid walk | `pixi run -e madgraph validate-pdf-grid` |
| the UFO loader, the SM model source, the submodule pin | `pixi run check-sm-blob-fresh` |
| run-card parsing or defaults | `pixi run -e madgraph dump-runcard-defaults`, then the hermetic `scales_run_cards` test |

The amplitude gate (`amplitude_oracle`) is itself hermetic — it reads committed
tables — so `cargo test` runs it. `pixi run -e madgraph validate-amplitudes`
exists to run it against *freshly regenerated* tables, which is the stronger
check and the one that matters after an amplitude change.

`validate-pythia` needs Pythia 8 and so sits outside `pixi run validate`; run it
under its own environment. The collator says so when its verdict predates the
current run's cells.

## Regenerating references

One entry point over every generator, staged `deps → madgraph → refs → bundle`:

```bash
pixi run -e madgraph generate-references              # all four stages
pixi run -e madgraph generate-references refs bundle  # re-extract and re-archive
```

The MadGraph work area is the cache — an existing process directory is never
rebuilt — while the extractions are cheap pure functions of it and always rerun,
so a reference that moved shows up as a diff. Committed references
(`validation/madgraph/*_reference.json`, `validation/madgraph/diagrams.json`,
`validation/alphas/reference.csv`) are regenerated only when the banked phase-space
points, the process list, or the pinned MadGraph version change — never to make a
failing gate pass. A reference that moves is a finding; diagnose it before
rebanking it.

## The long tier

Cells too expensive for the banked budget, each with a task rather than only a
note:

```bash
pixi run validate-sigma-2to6      # the 2 -> 6 sigma rows (info, not enforced)
pixi run validation-report        # collate after, so the report carries them
pixi run ladder-2to6              # budget ladders, five seeds a rung
pixi run ladder-bb
pixi run ladder-recarded
pixi run -e madgraph validate-kt-cluster
pixi run validate-deep            # prints what the long tier consists of
```

Run `validate-sigma-2to6` *between* `validate` and `validation-report`:
`validate` clears the per-category cells first, which is what stops a stale
long-tier measurement being served as this run's.

## Running discipline

These are long jobs. A foreground command silent for ~600 s kills an agent
session, and a killed run leaves zombie `cargo` processes needing `kill -9`.
Background anything heavier than the hermetic suite and poll its output; never
wrap one in a foreground `sleep` loop.

Before reporting a gate green: the command and its output are the evidence, not
the exit status you remember. A cell in `report.md` is only evidence if this run
wrote it.
