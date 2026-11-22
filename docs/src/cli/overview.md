# Command line

The `vibegraph` binary has six commands. Two of them are the generator, and
the split between them mirrors MadEvent's survey-and-refine phase followed
by its event-generation phase:

1. **`vibegraph integrate`** enumerates the diagrams, compiles the matrix
   element, adapts one VEGAS grid per phase-space channel, and banks the
   result in `grid.bin.zst`. It runs until the cross section's relative
   uncertainty reaches `--target-rel` (0.1% by default), or for a flat
   `--neval × --niter` under `--fixed-budget`.
2. **`vibegraph generate`** reloads the artifact, unweights against the
   frozen grids by accept/reject, and writes a Les Houches event file. It
   takes the artifact and no card: the cards `integrate` read, the model
   itself and the diagrams are recorded in it, so a worker needs the binary,
   the artifact and, at proton beams, the PDF set, and nothing else.

The third, **`vibegraph check-events`**, reads an event file back and checks
momentum balance, mass shells, weight bounds and the `<init>` cross
references. The fourth, **`vibegraph inspect`**, prints the results a saved
`grid.bin.zst` records (σ, its error, χ²/dof, the factorization scale, each
multiplicity's and each channel's share) without re-running anything, and
with `--ranking` writes its configurations ranked by their share of σ as
JSON ([below](#ranking-the-diagrams)).

The fifth, **`vibegraph enumerate`**, takes a process card and reports every
diagram that contributes, so that a user can check that a card means the
process they intended and nothing more, the way MadGraph's `display diagrams`
is used. It is described [below](#enumerating-diagrams). `integrate` takes
the `diagrams.bin.zst` it writes in place of a proc card, told apart by
content, and then enumerates nothing and reads no UFO directory: the card and
the restricted model are the artifact's.

The sixth, **`vibegraph run-card`**, reads a run card and reports what
`integrate` would make of it, without enumerating or sampling anything: every
parameter resolved, classified and, where `integrate` would refuse it, the
refusal ([below](#inspecting-a-run-card)).

The [command reference](reference.md) lists every option of each command;
this page is about how they fit together. The physics behind each
step is the subject of the [guided tour](../guide/01-pipeline.md).

## Enumerating diagrams

```sh
vibegraph enumerate proc_card.dat --out diagrams/
```

writes three things into `--out` (the current directory by default):

- `diagrams.bin.zst`, the diagram artifact: the card, the restricted model it
  was enumerated against, every subprocess's diagrams in enumeration order,
  and a digest of them;
- `diagrams.json`, a summary: per subprocess (`s0`, `s1`, … in enumeration
  order) its label, diagram count, coupling-order histogram, integration
  configurations (`c0`, `c1`, …, MadGraph's `AMP2` grouping) and the contact
  diagrams that have none; per diagram (`s3.d17`) its propagators, vertices,
  coupling orders, configuration and drawing;
- `svg/<set>/<diagram>.svg`, one drawing per diagram, its external legs
  labelled with their particle and their index in the subprocess. `--svg`
  selects them: `all`, `none`, or `first=N` per subprocess (the default is
  `first=500`, which draws every diagram of most cards; drawing costs about a
  millisecond and 5 KB per diagram).

It refuses to overwrite any of the three without `--force`, before reading the
card. Given `--run-card` or `--param-card`, the summary also previews the
flavour groups `integrate` would form under those cards (the one not given
takes `integrate`'s default), and records which cards it used; without
either, it reports no grouping, since the grouping depends on both cards.

### Ranking the diagrams

Once a run has integrated (from the card or from `diagrams.bin.zst`),

```sh
vibegraph inspect grid.bin.zst --ranking ranking.json --diagrams diagrams/
```

writes a JSON document that ranks the run's integration configurations
(`s0.c3`: `diagrams.json`'s configuration `c3` of subprocess `s0`) by σ_c,
their share of the cross section under MadEvent's `AMP2` partition, with each
one's diagrams, share and cumulative share, the channels that sample it and,
at proton beams, the flavour group it stands for. Contact diagrams and the
weight no configuration carried are listed apart, and the sampler's own
per-channel σⱼ is reported as a diagnostic, not as a ranking: it measures
which channel drew the points, not which diagrams matter. Everything is read
from the artifact; nothing is integrated again.

`--diagrams` pairs the ranking with an `enumerate` output directory (or its
`diagrams.json`): it is refused unless the two record the same
`diagram_set_digest`, and each diagram's `svg` is then the drawing
`diagrams.json` records, relative to that directory. Without it, each carries
the conventional `svg/<set>/<diagram>.svg`. `--ranking -` writes to stdout,
and an existing file is kept unless `--force` is given. The schema is in the
knowledge bundle (`research/kb/pipeline/ranking-json.md`).

## Inspecting a run card

```sh
vibegraph run-card run_card.dat --proc-card proc_card.dat --output-ldjson
```

resolves the card the way `integrate` reads it and reports, in one JSON
object, every parameter MadGraph's LO template knows. Without `--output-ldjson`
the same object is printed as indented JSON on stdout. Nothing is enumerated
unless `--proc-card` is given, and nothing is ever sampled. Per parameter
(`keys`, in the template's order):

- `mg_default`, MadGraph's LO default, and `card_value` and `line`, what the
  card writes (`null` for a parameter it leaves out);
- `resolved`, the value the run reads after MadGraph's own edits of the card
  (for example `xqcut` rewriting `ptj`), with `moved_by_card` (the card writes a
  value other than the default) and `moved_by_resolution` (the edits moved it);
- `class` (`consumed`, `ignored_benign` or `ignored_physics`), `status`
  (`honoured`, `ignored` or `refused`) and a one-line `note`;
- `refusal`, the message `integrate` emits for this value of this parameter, or
  `null`.

The top-level `refusal` is the first refusal `integrate` emits for the whole
card from the run-card stages (`parse`, `cuts` or `split`): it is a field of the
report, and the command itself succeeds. A refusal that needs the process (the
cut compiler's, the phase-space guards) is outside this view, and `integrate
--check` is the command that reaches it. A line that does not read is listed in
`unknown_keys` with its line, and the remaining parameters are still resolved.
`warnings` carries the notices the resolution logs. With `--proc-card`,
`process_defaults` lists what the process moves from MadGraph's LO template in
the card `integrate` writes for it when no run card is given (the incoming
quark's `maxjetflavor`, a decay's removed cuts, matching under jet
multiplicities), with `overridden_by_card` for those the given card writes
itself; with no run card the report is of that default card.

```sh
vibegraph run-card --table > run-card-table.json
```

prints the table of every known parameter and no card: its `status`, its
`refused_values` where only some values are refused, and a one-line `note`. It
is generated from the same lists the parser and the cut compiler hold, so it
cannot drift from the binary, and it is printed by the binary rather than
shipped as a separate file so that it is always the table of the binary that
prints it (`version` names the release).

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
dynamical scale. As in the card MadGraph's `output` writes for the process,
`maxjetflavor` is 5 when some subprocess has an incoming b (a five-flavour
model such as `sm-no_b_mass`), so its b quarks take the jet cuts; it stays 4
otherwise. One default departs from MadGraph's card: the parton
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
`integrate` also refuses a result its points do not carry, and writes no
artifact for it. If no point of the integration has weight, the error says how
many points passed the cuts: none means the cuts reject everything the sampler
reaches (the run card's, or the `bwcutoff` window a decay chain holds its
resonances in); some means the matrix element is zero wherever they passed, so
a coupling or width is zero at the parameters in force, or the restriction
removes the process. If some points have weight but fewer than an effective 20
carry it, measured as `(Σw)²/Σw²` over the points the cross section averages,
the estimate and its error are set by those few draws, and the error gives the
counts. A process whose phase space the sampler reaches only by chance, such as
a resonance far narrower than the window a decay chain holds it in, ends this
way.

## The artifact

`grid.bin.zst` is a bincode + zstd snapshot of the adapted per-channel VEGAS
grids together with the run that trained them: the proc card's text, the
process, the model identity (its import label and a SHA-256 digest of the
parsed model), the PDF set, the seed and evaluation counts, the fully
resolved run card, and the run card's generation settings (`nevents`,
`event_norm`, `use_syst` and the systematics request, typed: the scale, PDF
and αs variations MadGraph's systematics module would compute, which
`generate --systematics` can replace). Like a gridpack, it is all
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
`generate` jobs. An artifact that records `σ = 0`, which a build without
`integrate`'s refusal could write, is refused: there is nothing to sample. Each job must pass its own `--seed`, which has no default,
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

runs it out of a checkout. Every run embeds the restricted model it was built
from in its artifact, with its name, restrict card and digest, and `generate`
rebuilds the model from those bytes: it takes no `--ufo-dir`. Each artifact
also banks |M|² at a few fixed phase-space points per subprocess, and
`generate` refuses a build that no longer reproduces them to 1e-10, since the
banked cross section would then describe another matrix element. Both
artifacts, `diagrams.bin.zst` and `grid.bin.zst`, also record the version of
the diagram enumeration that produced their diagrams, and a build that
enumerates differently (a fixed sign, a diagram added or dropped) refuses
them by name: `enumerate` or `integrate` the card again. Squared-order constraints (`NP^2==1`) are a
hard error: this generator selects diagrams by their coupling orders and squares
the whole amplitude, so it cannot bound an interference term.

## Watching a run

Standard output carries the result and nothing else, at every verbosity:
the `σ` line, the path written, the `check-events` report. The cross section
and its error are printed under the one SI-prefixed barn unit σ's magnitude
picks, the error to two significant figures and σ to the same decimal place:
`σ = 8.6812 ± 0.0023 ab`, `σ = 1.23457 ± 0.00012 pb`. The per-multiplicity
lines of a mixed-multiplicity card follow the same rule.
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

`integrate` takes the same request from a signal, with or without the pane:
the first SIGTERM or SIGINT, once the iterations have started, finishes the
iteration in flight, writes the artifact and ends the run with a result whose
`stop_reason` is `stopped`; a second signal takes the default action. That is
the signal a batch system sends before it kills a job. A signal before the
first iteration (during enumeration, compilation or the channel survey) takes
the default action at once, and so does any signal to `generate`; a stop
before an iteration is kept ends in a `support` refusal, since nothing was
measured. `generate` writes its event file under a hidden temporary name
beside it and renames it once complete, so a killed run leaves nothing under
the requested name.

`integrate --check` runs everything `integrate` does before its first
sampling point (every card, the model and the parton densities, enumeration,
flavour groups, cuts and the phase-space guards) and then reports `check ok`,
or fails with the refusal the full run would have ended in. It draws no
point and writes no artifact. Under `--output-ldjson` its `result` is
`{"check": "ok", "process", "model", "seed"}`.

## Output modes

A run picks one output mode, which formats its log lines, its progress and
its result or refusal together:

| mode | when | log | progress | result or refusal |
|---|---|---|---|---|
| plain | not a terminal, or `--no-tui` | text lines on standard error | not shown | text lines |
| rich | a terminal on both streams, or `--tui` | the pane's scrollback | the status pane | text lines, after the pane comes down |
| ldjson | `--output-ldjson` | one object per event | one object per event | one final object |

`--output-ldjson` is for a program reading the run. Standard output then
carries one JSON object per line and nothing else, in the order the run
produced them; the status pane is never drawn, and `--tui` with it is a
usage error. Every object has a `kind` and a `schema_version` (currently
`1`); a reader refuses a version it does not know. A new field can appear in
any object without a version bump, so a reader ignores fields it does not
know; removing, renaming or re-meaning a field, or changing the list of error
kinds, bumps the version.

- `log`: an event at the visible level, as `tracing-subscriber`'s JSON
  formatter writes it: `timestamp`, `level`, `target`, the event's `fields`
  (its text is `fields.message`) and, inside a stage, the `span` it ran in.
- `progress`: a progress measurement, at any level, in the same layout. Its
  `fields` are the progress stream's own: `stage`, `done`, `total` when the
  stage knows it, and the stage's extras. The `enumerate` stage counts the
  candidate particle assignments of one process line's alias expansion, those
  pruned as duplicates or charge-violating included, out of the expansion's
  size; its extras are `line` of `lines` (the process line, from 1),
  `subprocesses` enumerated in this line's walk and the running `diagrams`
  count over the card. The count restarts for each line, for each further
  coupling-order bound the automatic search tries, and for each core of a
  decay chain. The VEGAS stage's running `sigma`
  and `err` are in the integrand's natural units (GeV⁻² for a cross section,
  GeV for a width).
- `result`: the command's final object. `integrate`'s carries `process`,
  `model` (`label`, `digest`), `observable` and `unit` (`pb`, or `GeV` for a
  decay's width), `sigma`, `sigma_err`, `chi2_per_dof` (`null` over fewer than
  two iterations), `mu_f` (tagged by `type`: `no_densities`, `constant` with
  `gev`, `per_beam` with two, `per_event` with the `dynamical_scale_choice`),
  `seed`, `stop_reason` (`fixed_budget`, `target_met`, `max_iters`,
  `max_evals`, `stopped`, or `constant` for an integrand that is constant by
  construction), `target_rel` (the `--target-rel` asked for, `null` under
  `--fixed-budget`), `rel_err_quoted` (σ's relative error as `sigma_err`
  quotes it) and `rel_err_scaled` (the same with each channel's variance
  widened to the scatter its iterations show, the number `--target-rel` is
  compared with), `multiplicities` (each final-state phase space of a proton run:
  `final_state`, `process_ids`, `final_masses`, `channels` and its σ),
  `channels` (each channel's `key`, `alpha`, `neval` and σ), the `artifact`
  path and its `format_version`, the `diagram_set_digest` of the diagrams it
  integrated, and `param_card` (whether a param card's text is banked).
  `inspect --output-ldjson` prints the same
  object for a saved artifact, with `stop_reason`, `target_rel`,
  `rel_err_quoted` and `rel_err_scaled` `null`, since the artifact does not
  record how its run ended; with `--ranking <file>` its result carries
  `ranking_path` and the document under `ranking` instead. `generate`'s carries `events_written`,
  `events_drawn`, the `output` file, the `artifact` it read, the sample's own
  `sample_sigma` ± `sample_sigma_err`, the integration's `integration_sigma` ±
  `integration_sigma_err`, and the `pull` between them. `enumerate`'s carries
  the `artifact` and `summary_path` it wrote, the `svg_dir` (`null` when
  nothing was drawn) and the number of `drawings`, and the whole of
  `diagrams.json` under `summary`. `generate -o` writes `.gz` and `.zst`
  names compressed and any other name as plain text, and refuses another
  compression extension (`.xz`, `.bz2`, ...) by name.
- `refusal`: in place of a result, `error_kind` and `message`. A refused
  run writes no artifact, so this is its only record.

Values are at full precision in base units: picobarns and GeV, never an SI
prefix. The `error_kind` is one of a fixed list, which only a schema version
bump changes: `usage` (the flags), `card` (a proc, run or param card),
`model`, `process` (enumeration, compilation, or a process shape the
integrator refuses), `pdf` (the parton densities), `network` (a download
refused or failed), `support` (an integration whose points do not support a
result), `artifact` (a saved artifact unreadable or inconsistent), `io`
(another file or directory), and `run` (anything else that failed once the
inputs were accepted). `check-events` prints a report for a person and
refuses `--output-ldjson`.

### Phase timing

Under `--output-ldjson`, `integrate` and `generate` also write a `phase`
object when each phase of the run starts and another when it ends, at any
verbosity. Each is written and flushed as it happens, so a run killed from
outside (a batch system's `SIGKILL` at its time limit) still names the phase
it was in: the last `phase` record without its `end`. A `phase` is a new
`kind` under `schema_version` 1; a reader skips kinds it does not know.

| field | on | meaning |
|---|---|---|
| `event` | both | `start` or `end` |
| `phase` | both | the phase's name, below |
| `elapsed_s` | both | seconds since the process started, on a monotonic clock |
| `wall_s` | end | the phase's own wall time, `elapsed_s` at the end less at the start |
| `outcome` | end | `ok`, or `failed` when the phase ended in the run's refusal |
| `sizes` | end | the counts that explain the time, by phase (below); `{}` where none |

`integrate` enters, in order and each at most once: `input` (the card or
diagram artifact read, the model loaded), `enumerate` (from a proc card;
`subprocesses`, `diagrams`), `amplitude_probe` (each diagram set compiled
and evaluated at the probe points the artifact banks; `diagram_sets`), `pdf`
(proton beams), `compile` (flavour groups and one evaluator per subprocess
or group, with the cuts; `subprocesses`, `compiled_diagrams`, `helicities`,
`colour_flows`, and on proton beams `parts` and `flavour_groups`),
`channels` (the maps and the integrand over its channels; `channels`),
`alpha_survey` (the channel-weight survey; `points_per_iteration`,
`iterations`; on fixed-energy beams and decays it also builds the
per-diagram channel maps), `vegas` (every VEGAS iteration, the warm-up
included; `iterations`, `points`), `configurations` (the per-configuration
shares and channel records) and `bank` (the artifact written). The VEGAS
iterations inside `vegas` are the `progress` objects of the `vegas` stage.
`--check` ends after `channels`. `generate` enters `input`, `pdf` and
`generate` (`channels`, `events_written`), the last resolved further by its
`compile`, `weight_scan` and `unweight` progress stages.

Without `--output-ldjson` the same records are log events: the start at
`debug`, the end at `info` as `phase vegas: 12.345 s (iterations 3, points
30000)`. `--log-file` records both.

## Parallelism

`-j` sets the thread count for the integration. Thread count moves no bit:
the sampling is chunked deterministically over counter-based random
substreams, so `-j 16` writes the same artifact as `-j 1`. Job-level
parallelism comes from the artifact split described above.
