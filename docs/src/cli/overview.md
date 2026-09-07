# Command line

The `vibegraph` binary has three commands. Two of them are the generator, and
the split between them mirrors MadEvent's survey-and-refine phase followed
by its event-generation phase:

1. **`vibegraph integrate`** enumerates the diagrams, compiles the matrix
   element, adapts one VEGAS grid per phase-space channel, and banks the
   result in `grid.bin.zst`. It runs until the cross section's relative
   uncertainty reaches `--target-rel` (0.1% by default), or for a flat
   `--neval × --niter` under `--fixed-budget`.
2. **`vibegraph generate`** reloads the artifact, unweights against the
   frozen grids by accept/reject, and writes a Les Houches event file.

The third, **`vibegraph check-events`**, reads an event file back and checks
momentum balance, mass shells, weight bounds and the `<init>` cross
references.

A fourth, `vibegraph enumerate`, is planned: it will take a process card
and report every diagram that contributes, as drawings, a summary page and
a binary artifact `integrate` can take instead of enumerating again, so
that a user can check that a card means the process they intended and
nothing more, the way MadGraph's `display diagrams` is used.

The [command reference](reference.md) lists every option of each command;
this page is about how they fit together. The physics behind each
step is the subject of the [guided tour](../guide/01-pipeline.md).

## Two cards

Both commands take MadGraph's own card formats, so the same pair of files
drives a MadGraph reference run unchanged.

The **process card** picks the model and the process. A card with no
`import model` line uses the Standard Model compiled into the binary;
`import model <name>` loads a UFO model directory instead. The `generate`
line is MadGraph's process grammar, multiparticle labels and coupling-order
constraints included:

```text
generate p p > l+ l- j QCD=2 QED=2
```

The **run card** sets the beams, cuts and scales. Every parameter it leaves
out takes MadGraph's LO default, so the shortest useful run card is the one
that turns on fixed-energy partonic beams:

```text
0    = lpp1
0    = lpp2
45.6 = ebeam1
45.6 = ebeam2
```

With no run card at all, a process runs at MadGraph's defaults: 13 TeV
protons, the default jet and lepton cuts, and the default kT-clustered
dynamical scale.

```bash
echo 'generate e+ e- > mu+ mu-' > proc_card.dat
vibegraph integrate proc_card.dat --run-card run_card.dat --out ee_to_mumu/
vibegraph generate ee_to_mumu/grid.bin.zst proc_card.dat --run-card run_card.dat \
  --nevents 10000 --seed 1 -o events.lhe
vibegraph check-events events.lhe
```

The process-card argument accepts `-` for a card on standard input.

## The artifact

`grid.bin.zst` is a bincode + zstd snapshot of the adapted per-channel VEGAS
grids together with everything needed to refuse a mismatched replay: the
process, the model identity (its import label and a SHA-256 digest of the
parsed model), the PDF set, the seed and evaluation counts, and the fully
resolved run card. `generate` compares its own cards against the banked ones
exactly and refuses on any difference.

The compiled amplitude is deliberately not banked. Every `generate` job
re-reads the cards and recompiles, so a worker machine needs the binary,
the artifact, both cards, and, for proton beams, the PDF set. In return the
expensive adaptive integration runs once and the artifact is a read-only
input to any number of `generate` jobs, each with its own `--seed` and
`--nevents`. The random number generator is a counter-based ChaCha8, so
distinct seeds give independent streams and the resulting files concatenate
into one sample. See [unweighting and event files](../guide/11-unweighting-events.md)
for what a seed reproduces.

## Data the binary does not carry

The Standard Model is compiled in. Two kinds of data are resolved on demand,
each in the order flag, environment variable, `~/.vibegraph/` cache, working
directory:

| Data | Flag | Environment | Cached at |
|---|---|---|---|
| PDF sets (proton beams) | `--pdf-dir` | `$VIBEGRAPH_PDF_DIR` | `~/.vibegraph/pdf/<set>/` |
| UFO models (non-SM) | `--ufo-dir` | `$VIBEGRAPH_UFO_DIR` | `~/.vibegraph/ufo/<model>/` |

A missing PDF set can be downloaded. The binary shows the URL, the size and
the SHA-256 it will check against, and asks; the checksum is compiled in, so
the data a set name resolves to cannot drift from what the build was
validated against. Nothing is downloaded without consent: without a terminal
to ask on, the answer is no, and the run fails naming the URL, the checksum
and the `-y` that would consent on a rerun. `--no-network` (or
`$VIBEGRAPH_NO_NETWORK`) forbids downloads outright and outranks `-y`.
`$VIBEGRAPH_HOME` moves the cache.

UFO models are never downloaded, because FeynRules publishes no per-model
index a name could be pinned against. Unpack the model directory under the
cache or point `--ufo-dir` at it.

## Watching a run

Standard output carries the result and nothing else, at every verbosity:
the `σ = … pb` line, the path written, the `check-events` report.
Everything the run has to say about itself goes to standard error, at a
level chosen by `-v`/`-q`/`--log-level`, with `--log-file` recording
everything at `trace` regardless, and `RUST_LOG` as the per-module override.

On a terminal, `integrate` and `generate` draw a status pane pinned below
the log: the process brief, the stage in progress and its bar, the running
σ ± err or the unweighting efficiency, and the cost of an integrand
evaluation. The integration bar is a projection, not a plan: a run
converging to a target error does not know how many iterations it needs, so
the bar's total is re-estimated from the current error every iteration, and
can move in either direction. Arrow keys change the visible level and
module scope; `q` or `^C` asks for a graceful stop, which finishes the
iteration, banks what completed, and writes a usable artifact. A second
press quits at once.

## Parallelism

`-j` sets the thread count for the integration. Thread count moves no bit:
the sampling is chunked deterministically over counter-based random
substreams, so `-j 16` writes the same artifact as `-j 1`. Job-level
parallelism comes from the artifact split described above.
