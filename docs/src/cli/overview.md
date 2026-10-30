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
   frozen grids by accept/reject, and writes a Les Houches event file. It
   takes the artifact and no card: the cards `integrate` read are recorded in
   it.

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

## Cards

`integrate` takes MadGraph's own card formats, so the same files drive a
MadGraph reference run unchanged.

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
dynamical scale. One default departs from MadGraph's card: the parton
densities. MadGraph's template says `pdlabel = nn23lo1`, its own built-in
parameterisation; vibegraph reads densities only from LHAPDF6 grids, so its
default is `pdlabel = lhapdf` at `lhaid = 247000`, `NNPDF23_lo_as_0130_qed`,
the set `nn23lo1` is fitted to. A card that omits the lines gets the same. A
proton card naming `nn23lo1` or any other built-in label is refused, with a
message that says to set `pdlabel = lhapdf` and an `lhaid`.

The run card alone chooses the PDF: `lhaid` names the set and the member
(`lhaid` minus the set's first id), found through LHAPDF's `pdfsets.index`.
`integrate` takes no option for it.

```bash
echo 'generate e+ e- > mu+ mu-' > proc_card.dat
vibegraph integrate proc_card.dat --run-card run_card.dat --out ee_to_mumu/
vibegraph generate ee_to_mumu/grid.bin.zst --nevents 10000 --seed 1 -o events.lhe
vibegraph check-events events.lhe
```

The process-card argument accepts `-` for a card on standard input.

A third card is optional: **`--param-card`** takes an SLHA `param_card.dat`
and evaluates the model at its values instead of the restriction's defaults.
A gridpack's `Cards/param_card.dat` is the card to give, since it carries
every edit MadGraph applied before the run. The artifact records each
parameter the card moved off the defaults, and `generate` takes no param card:
it evaluates at the recorded values, and writes the card it evaluated at into
the event file's `<slha>` header block, where a shower reads masses and widths.
`integrate` refuses a card it cannot honour: a width given as `Auto` (MadGraph
computes those at launch and writes the number into the card it ran with), a
non-zero value for a parameter the restriction zeroes, a value other than one
for a parameter the restriction fixes to one, a value that splits two
parameters the restriction merged, and an entry the model has no parameter
for. A value for a mass or width the model computes, such as the W mass in
the Standard Model, is ignored, as MadGraph ignores it, with a warning when it
disagrees with the computed one.
## The artifact

`grid.bin.zst` is a bincode + zstd snapshot of the adapted per-channel VEGAS
grids together with the run that trained them: the proc card's text, the
process, the model identity (its import label and a SHA-256 digest of the
parsed model), the PDF set, the seed and evaluation counts, the fully
resolved run card, and the run card's generation settings (`nevents`,
`event_norm`, the systematics request). Like a gridpack, it is all
`generate` needs besides the assets: `generate` takes no proc card, run card
or PDF choice, and a command line that passes one is an argument error.

The compiled amplitude is deliberately not banked. Every `generate` job
re-parses the recorded proc card and recompiles, so a worker machine needs
the binary, the artifact, the model if it is not the built-in SM, and, for
proton beams, the PDF set the artifact names. What it reads is checked
against the record: a recorded proc card this build parses to another
process, or a model whose digest is not the recorded one, is refused, with
the advice to re-integrate. In return the expensive adaptive integration
runs once and the artifact is a read-only input to any number of
`generate` jobs. Each job must pass its own `--seed`, which has no default,
so that jobs on one grid cannot write the same events by accident;
`--nevents` and `--event-norm` override the recorded values. The random number generator is a counter-based ChaCha8, so
distinct seeds give independent streams and the resulting files concatenate
into one sample. See [unweighting and event files](../guide/11-unweighting-events.md)
for what a seed reproduces.

## Data the binary does not carry

The Standard Model is compiled in. Two kinds of data are resolved on demand:

| Data | Flag | Environment | Cached at | Last |
|---|---|---|---|---|
| PDF sets (proton beams) | none: the run card's `lhaid` | `$LHAPDF_DATA_PATH`, else `$LHAPATH` | `~/.vibegraph/pdf/<set>/` | a download, with consent |
| UFO models (non-SM) | `--ufo-dir` | `$VIBEGRAPH_UFO_DIR` | `~/.vibegraph/ufo/<model>/` | the working directory |

The PDF variables are read as LHAPDF reads them: a `:`-separated list of
directories, searched in order for `<set>/`, with `$LHAPATH` read only when
`$LHAPDF_DATA_PATH` is unset. A user whose LHAPDF installation works
therefore needs no setting of vibegraph's own. `$VIBEGRAPH_PDF_DIR` is no
longer read, and a run that finds it set says so.

A missing PDF set can be downloaded. The binary shows the URL, the size and
the SHA-256 it will check against, and asks; the checksum is compiled in, so
the data a set name resolves to cannot drift from what the build was
validated against. Nothing is downloaded without consent: without a terminal
to ask on, the answer is no, and the run fails naming the URL, the checksum
and the `-y` that would consent on a rerun. `--no-network` (or
`$VIBEGRAPH_NO_NETWORK`, set to any value) forbids downloads outright and
outranks `-y`. `$VIBEGRAPH_HOME` moves the cache.

UFO models are never downloaded, because FeynRules publishes no per-model
index a name could be pinned against. Unpack the model directory under the
cache or point `--ufo-dir` at it.

`import model <name>-<restrict>` selects `restrict_<restrict>.dat` from inside
the model directory, the way MadGraph does, and the restriction's values become
the model's parameter defaults: a run with no param card of its own computes at
exactly the values MadGraph's generated `param_card.dat` would carry. A card
given to `integrate --param-card` that sets a parameter the restriction zeroes
to anything but zero is refused, because the restriction is also what pruned
the vertices that parameter feeds. The SMEFT model the validation
ladder gates is committed in the repository, so

```bash
printf 'import model SMEFTsim_topU3l_MwScheme_UFO-massless\ngenerate e+ e- > t t~ NP<=1\n' \
  | vibegraph integrate - --ufo-dir validation/ufo --run-card run_card.dat --out ee_to_ttx_smeft/
```

runs it out of a checkout. Every run records the model it was built from — name,
restrict card and a digest of both — in its artifact, and `generate` refuses
grids trained on a different one. Squared-order constraints (`NP^2==1`) are a
hard error: this generator selects diagrams by their coupling orders and squares
the whole amplitude, so it cannot bound an interference term.

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
