---
type: Session Report
title: "L-T3 report: the conserved-number prune, parallel walks, parallel set compilation and one compile per set"
description: "Enumeration skips assignments that break a number the model's vertices conserve and assigns a walk's subprocesses across the pool; the run's sets are compiled once, across the pool, for the probe and the integrand; generate refuses --parallel-diagrams. Order-preserving, bit for bit."
status: draft
tags: [performance, enumerate, compile, amplitude-probe, parallelism, cli, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Branch `sprintl-timeout-fixes` off main a5bf8fca, with origin/main (PR 3a
#183 and PR 3 #173) merged in at 6ee36331. Closes
`enumeration-hands-flavour-violating-subprocesses-to-feyngraph`,
`set-compilation-loops-are-serial`, `amplitude-probe-compiles-each-set-twice`
and `generate-parallel-diagrams-is-a-no-op`; their files are deleted in the
final commit, and the one link to them from another page (L-T2's report) now
points here.

**Classification: order-preserving throughout.** No arithmetic is reordered.
Enumeration drops only assignments with no diagram and keeps the walk order;
every compile is the same `compile_class` of the same set, reused instead of
repeated; parallel loops collect in input order and return the first refusal
in input order. Everything below gates bit for bit.

Local measurements are dev profile (opt-level 2, no LTO), on the shared
4-core container, both sides of every comparison built the same way. The
baseline binary was built in this worktree before any code change, at
821053f5, whose `vibegraph-lib`/`vibegraph-cli`/`Cargo.*` are identical to
a5bf8fca (`git diff a5bf8fca --stat -- vibegraph-lib vibegraph-cli
Cargo.toml Cargo.lock` is empty), instead of in a separate main worktree; it
lives in the session scratchpad as `lt3-bin/vibegraph-dev-a5bf8fca`.

## What changed

1. **Conserved-number prune** (`vibegraph-lib/src/ufo/conserved.rs`, new;
   `UFOModel::conserved_numbers`). The additive numbers are the integer null
   space of the vertex × particle-pair matrix over the vertices enumeration
   builds its topology model from (`Vertex::is_physical`, after the restrict
   card), solved exactly with `num-rational`'s `BigRational` (`num-bigint`
   moves from dev- to regular dependency, `num-integer` is added; both were
   already in the lock). Self-conjugate, Goldstone and ghost particles carry
   zero. Nothing about the SM is assumed: sm-no_b_mass derives 7 numbers
   (Q, Le, Lμ, Lτ, three quark generations), sm-ckm derives 5 (Q, three
   lepton flavours, B). `walk_assignments` skips an assignment whose
   incoming and outgoing totals differ in any of them, after the charge
   filter. By the tree-diagram argument in the module doc such an assignment
   has no diagram, so only empty sets disappear, and `enumerate` already
   dropped those.
2. **Parallel walks** (`diagrams/mod.rs`). The walk (polarizations, dedup,
   mirror marking, the two prefilters) stays serial; surviving assignments
   queue in batches of 16 × threads and are assigned across the pool
   (`Walk::assign`), collected in walk order. Mirror marking reaches a
   queued assignment as well as an assigned one. The log lines a subprocess
   produces are written by the walk, in order, after its batch
   (`report_assigned`), so `-vv` logs are in walk order whatever the
   schedule. The `assign_topologies` `.expect` is now the named refusal
   `DiagramError::TopologyAssignment`.
3. **`EnumerationPool::Auto`** is the new default: a walk whose candidates ×
   tree topologies reach `AUTO_PARALLEL_COST` (2¹⁴, measured below) runs on
   the caller's pool, a smaller one inside a process-wide one-thread pool, so
   feyngraph's internal fan-out stays off the pool for the small walks it
   slows. `--parallel-diagrams` (`Ambient`) forces every walk onto the pool;
   `Serial` is unchanged. No force-off flag was added: no measured walk was
   slower on the pool (`-j 1` serves as one).
4. **Parallel set compilation** (`hadronic::compile_classes`,
   `complete_compiles`). `derive_flavor_groups` (now a wrapper of
   `derive_flavor_groups_with`) compiles the sets it is not handed across the
   pool and evaluates its grouping traces across the pool; the per-set checks
   (`n_in`, cuts, one-body) stay a serial loop in set order over the compile
   results, so its refusal is the serial one. `compile_subprocesses` is a
   wrapper of `compile_subprocesses_with`, the same way.
5. **One compile per set.** `integrate` compiles every set once, across the
   pool (`compile_classes`), probes through those evaluators
   (`amplitude_probe::measure_compiled`, also across the pool) and hands them
   to `compile_subprocesses_with` (fixed energy, decays) or
   `multiplicity_groups_of_with` → `derive_flavor_groups_with` (proton).
   `generate` does the same for the probe check
   (`amplitude_probe::check_compiled`) and `generate_sample` /
   `generate_proton_sample`. A set whose outgoing legs `content_ordered`
   reorders is compiled again in its new order (its evaluator would differ),
   and its probe evaluator is dropped first; a set the probe could not
   compile is compiled again where the integrand needs it, so its refusal is
   the one it had. The splits (`split_by_multiplicity`,
   `split_by_external_masses`) are generic internally so the evaluators travel
   with their sets.
6. **Phase records** (merged from #183): the `amplitude_probe` phase now
   holds the run's compiles and the `compile` phase only the grouping, cuts
   and the reordered sets' compiles. The two `PhaseName` docs say so. No
   phase moved or was added, so the order the phase-timing test checks is
   unchanged.
7. **`enumerate`'s tail.** The flavour preview (`--run-card`/`--param-card`)
   groups the sets just enumerated instead of enumerating the card a second
   time (`multiplicity_groups` → `multiplicity_groups_of`). The drawings are
   made across the pool while the artifact is zstd-encoded
   (`DiagramArtifact::encode` / `write_encoded`, split out of
   `write_to_path`, same bytes); nothing is written until both finish, and a
   drawing refusal still comes first.
8. **`generate` refuses `--parallel-diagrams`**: it flattens `ThreadArgs`
   (`-j` alone), so clap rejects the flag as an unknown argument. The CLI
   reference is regenerated with `scripts/gen-cli-docs.sh`.

## Oracles, with what each cannot detect

### Unit tests (new)

```
$ cargo test -p vibegraph-lib --lib -- diagrams:: ufo::conserved amplitude_probe
test result: ok. 98 passed; 0 failed; 0 ignored; 0 measured; 1379 filtered out; finished in 2.03s
$ cargo test -p vibegraph --bin vibegraph
test tests::only_the_enumerating_commands_take_parallel_diagrams ... ok
test result: ok. 125 passed; 0 failed
```

- `ufo::conserved::tests::the_diagonal_standard_model_conserves_charge_lepton_flavour_and_quark_generation`:
  sm-no_b_mass derives exactly 7 numbers spanning exactly {3Q, Le, Lμ, Lτ,
  B₁, B₂, B₃} (rank equality of derived, expected and their union). Cannot
  see a non-additive conserved quantity (a parity), which the prune does not
  use.
- `quark_mixing_leaves_baryon_number_and_no_generation_number`: sm-ckm
  derives 5, spanning {3Q, B, Le, Lμ, Lτ}, and no generation number lies in
  the span. This is the "flavour mixing derives fewer numbers" test.
- `every_derived_number_is_conserved_at_every_vertex` (no_b_mass, ckm,
  default): each derived row sums to zero on every unitary-gauge vertex.
- `the_null_space_is_exact_and_integer`, `balance_compares_incoming_with_outgoing_totals`.
- `diagrams::tests::the_flavour_prune_drops_no_subprocess_with_a_diagram`:
  `p p > l+ vl j` and `p p > e+ e- j j` (4-flavour `p`, `l+ = e+ mu+`,
  `vl = ve vm`) give byte-identical sets (bincode bytes and `Debug`) with
  the model's numbers and with none, on sm-no_b_mass and on sm-ckm, and
  `u c~ > e+ ve g`, `u d~ > mu+ ve g` are pruned on the diagonal model while
  `u s~ > e+ ve g` is kept on ckm. Cannot see an assignment with diagrams
  that only a card this one does not reach would produce; the census below
  extends it to the timed cards.
- `a_parallel_walk_gives_the_serial_walk_byte_for_byte`: a card of 83 sets
  over more than one batch (two DY lines, a W decay chain, a polarized line,
  mirrored and unmirrored sets) under `Ambient` and `Auto` on a 4-thread
  pool against `Serial`, bincode bytes and `Debug` equal. Cannot see an
  effect present only at another thread count.
- `a_refusal_inside_the_pool_is_the_serial_refusal`: custom propagators on
  `u` and `d`; `u u~ > w+ w-` (exchanges a `d`) and `d d~ > w+ w-` (a `u`)
  both refuse in one batch; eight `Ambient` runs on 4 threads return the
  serial string, naming `d`. No panic.
- `main.rs::only_the_enumerating_commands_take_parallel_diagrams`: `generate
  … --parallel-diagrams` is `UnknownArgument`, `generate … -j 2` parses,
  `enumerate`/`integrate`/`parameters` take both.

### Enumerate, base against head, byte for byte

`lt3-scripts/ladder.sh` runs `enumerate --svg none` with each binary at
`-j1`, `-j4` (head: `Auto`) and head `-j4 --parallel-diagrams`, and hashes
`diagrams.json` and `diagrams.bin.zst`. Every configuration of every card
gave one hash pair (all 3 or 2 repeats shown in the logs
`lt3--ladder1.log`, `lt3--ladder2.log`):

| card | sets / diagrams | json | bin |
|---|---|---|---|
| `e+ e- > mu+ mu-` (sm) | 1 / 2 | 9ae189bfcc5c | fa143aa62622 |
| `p p > e+ e-` | 5 / 10 | f12a701f9fde | 288ab8c5dbf6 |
| `p p > j j` | 96 / 154 | 61ec5ab71fda | 08b3d53a76d0 |
| `p p > e+ e- j` | 15 / 60 | f35be3651e8c | a2e402cf1422 |
| `p p > j j j` | 146 / 1095 | 37001bb16045 | 737ceb0cad0b |
| `p p > e+ e- j j` | 95 / 1040 | c93330d8645c | adb7b1d123bc |
| DY+0–2j | 115 / 1110 | 2362e973ed0b | e21033df8d01 |

(5-flavour `p`, sm-no_b_mass unless stated.) With the default drawings
(`--svg first=500`) on DY+0–2j, the whole output tree of base and head at
`-j1` and `-j4` differs in one line only, `parameters.json`'s build
`"version"` string (`diff -rq` then `diff`). Cannot see a card whose
enumeration differs only on lines this ladder does not walk (decay chains
are in the unit test, not here); the DY+0–3j, W+0–4j and Z→νν+4j cards are
in the requests below.

### Integrate and generate, base against head, bit for bit

`lt3-scripts/gate.sh`: `integrate --fixed-budget --neval 20000 --niter 4
--seed 7`, base and head, `-j1` and `-j4`, on the banked `ee_to_mumu` card
(fixed energy) and the banked `pp_to_ll_0j2j_mlm` card (MLM, three
multiplicities, lhaid 247000), then `generate --nevents 300 --seed 3` from
the base `-j1` artifact with each binary at `-j1`/`-j4`:

| card | σ (pb) | error (pb) | grid.bin.zst sha256 | events sha256 |
|---|---|---|---|---|
| ee→μμ, all 4 runs | 2029.5243067486113 | 2.5946227539670867 | af585cd04a180de8… | 617bbee387df7a1d… |
| DY 0–2j MLM, all 4 runs | 1063.0574890622572 | 9.205000902959013 | afd7adaf4b388939… | 3ef3718a208fab1b… |

The grid file holds the probe values and the sets, so its hash is also the
bit-for-bit check on the probe. Events hashed from the first `<event>` on
(the header carries the build version). The head MLM runs record `compile`
0.01 s against `amplitude_probe` 0.29 s (`-j1`), 0.12 s (`-j4`): no set was
compiled twice. Cannot see a card whose sets `content_ordered` reorders
(their recompile path is covered by the existing `derive_flavor_groups`
tests, which run through the same function with no precompiled evaluator),
nor a decay card.

### Workspace gates

Run on the tree pushed with this report (dev profile, shared container):

```
$ cargo test --workspace --no-fail-fast
(74 test binaries) passed 2319 failed 0 ignored 28; exit 0
  incl. cli_reference_docs: committed_cli_reference_matches_the_binary ... ok
  incl. cli_phase_timing: 4 tests ok
$ cargo fmt --all --check                                   # clean
$ cargo clippy --workspace --all-targets -- -D warnings     # Finished, no warnings
$ cargo clippy --workspace --all-targets \
    --features vibegraph/extended-validation,vibegraph-lib/extended-validation -- -D warnings
                                                            # Finished, no warnings
$ pixi run --skip-deps kb-lint
1363 concepts, 467 backlog items; 0 errors, 5 warnings      # the 5 predate this branch
```

`pixi run validate` was not run here (CI's `banked` job runs it on every
push).

## Measurements

### The Auto threshold

`-j1` (one-thread walk) against `-j4 --parallel-diagrams` (pool walk), the
enumerate line's own time, dev, 2–3 repeats:

| card | candidates × topologies | `-j1` (s) | `-j4` pool (s) |
|---|---|---|---|
| `e+ e- > mu+ mu-` | 1 × 4 | 0.001 | 0.001 |
| `p p > e+ e-` | 121 × 4 = 484 | 0.002–0.003 | 0.002 |
| `p p > e+ e- j` | 1331 × 25 = 33 275 | 0.009–0.016 | 0.007–0.012 |
| `p p > j j` | 14 641 × 4 = 58 564 | 0.042–0.052 | 0.027–0.033 |
| `p p > j j j` | 161 051 × 25 | 0.27–0.37 | 0.25–0.27 |
| `p p > e+ e- j j` | 14 641 × 220 | 0.24–0.25 | 0.09–0.11 |
| DY+0–2j (3 lines) | | 0.19–0.27 | 0.09–0.11 |

No walk was slower on the pool; the two below 2¹⁴ take milliseconds either
way. `AUTO_PARALLEL_COST = 2¹⁴` puts `p p > e+ e-` and smaller on one
thread and everything from `p p > e+ e- j` up on the pool. Wall time of the
whole `enumerate` (startup included), Auto head against base: ee→μμ 0.090 s
both; `p p > e+ e-` 0.096–0.101 s against 0.099–0.113 s; `p p > j j` at
`-j4` 0.21–0.26 against 0.27–0.28; so the small cards are no slower at
`-j1` or `-j4`.

### Where the enumerate gain comes from

Base against head, same binary profile, enumerate line (s):

| card | base `-j1` | head `-j1` | head `-j4` Auto |
|---|---|---|---|
| `p p > j j j` | 0.64–0.69 | 0.27–0.37 | 0.24–0.29 |
| `p p > e+ e- j j` | 0.48–0.51 | 0.24–0.25 | 0.10–0.13 |
| DY+0–2j | 0.50–0.52 | 0.19–0.27 | 0.10–0.13 |

The `-j1` column is the prune alone: with `-vv`, `p p > e+ e- j j` walks
14 641 candidates, 10 285 duplicate, 3 780 charge-violating, 468 violating
another conserved number and 108 handed to feyngraph (95 with diagrams);
`p p > j j j` 161 051, 1 970 pruned, 166 handed on. Before, the 468 and
1 970 went to feyngraph too. The `-j4` column adds the pool.

### The serial tail (brief, scope item 3)

Timed with temporary `Instant` lines (not committed) in `enumerate::run` and
`DiagramArtifact::enumerate`, DY+0–2j, head, `-j4`:

```
--svg none:   facts 0.000  digest 0.002  summary 0.002  draw 0.002  artifact_write 0.433  json+params 0.438
default svg:  facts 0.000  digest 0.002  summary 0.003  draw 0.613  artifact_write 1.059  json+params 1.065
```

(cumulative seconds after enumeration). The tail is the artifact's zstd
level-19 encode (0.43 s here; the artifact embeds the parsed model, so most
of it is a per-run constant) and, with drawings, the serial SVG rendering.
The drawings now run across the pool alongside the encode: DY+0–2j with
default drawings, whole run, base 1.67–1.73 s against head 1.35 s (`-j1`)
and 0.84 s (`-j4`), identical trees. The encode itself is single-threaded
zstd; making it parallel changes the compressed bytes, so it is left as
Found 1.

The brief's "compile stage over flavour groups and cuts" is not in an
`enumerate` run without `--run-card`/`--param-card`: no set is compiled
there. With either card, the preview enumerated the whole card a second time
and compiled every set serially; it now reuses the sets and compiles them
across the pool.

## Release measurements (manager-approved jobs 1–3)

Release profile on both sides, `cargo build --release -p vibegraph -j 4`,
each in its own target directory: old = main f08aa3ce (a detached worktree,
removed afterwards), new = c74ade3e. One run per configuration, run one at a
time with nothing else heavy on the container (4 cores, which are not the
reference VM's cores). Driver `lt3-scripts/jobs.sh`, logs `lt3--job{1,2,3}.log`
in the session scratchpad; wall and CPU from bash `time`, the enumerate time
from the run's own `enumerated … (t s)` line.

### Job 1: DY+0–3j, `enumerate --svg none`

| run | wall (s) | user (s) | enumerate (s) |
|---|---|---|---|
| v0.2.2, 2-CPU VM, `-j2` (brief) | 27.2 | — | 20.5 |
| v0.2.2, 2-CPU VM, `-j2 --parallel-diagrams` (brief) | 20.8 | 28.9 | 13.9 |
| old, `-j1` | 18.64 | 18.39 | 14.90 |
| old, `-j2` | 18.41 | 18.15 | 14.86 |
| old, `-j2 --parallel-diagrams` | 14.33 | 20.79 | 10.88 |
| old, `-j4` | 17.53 | 17.26 | 13.90 |
| new, `-j1` | 6.05 | 5.72 | 2.64 |
| **new, `-j2`** | **4.90** | 5.64 | **1.38** |
| new, `-j2 --parallel-diagrams` | 4.94 | 5.70 | 1.44 |
| new, `-j4` | 4.30 | 5.54 | 0.86 |

All eight runs: 260 sets, 10 310 diagrams, `diagrams.json` c2b582fbbe80,
`diagrams.bin.zst` 7eafafbe201c, `diagram_set_digest` dbbc37153bdeceb8…, so
old and new and every thread count are byte-identical. The acceptance (≈14 s
or less at `-j2`, no flag) is met: 4.9 s wall here against the old build's
18.4 s on the same box. Most of the gain is the prune (`-j1`: 14.9 → 2.6 s
enumerate). What is left of the wall at `-j2` is ~3.4 s after enumeration,
the zstd-19 encode of the artifact (Found 1).

### Job 2: W(→ℓν)+0–4j and Z(→νν)+4j, new only, `-j4`, `timeout -s KILL 600`

`enumerate` writes no phase records (they are `integrate`'s and
`generate`'s), so the per-line counts are from its `progress` records (the
last record of each line's walk) and the time from the walk's first and last
record:

| card | line | candidates | handed to feyngraph | line wall (s) |
|---|---|---|---|---|
| W+0–4j | W± +0j | 1 089 each | 6 each | 0.0 |
| | W± +1j | 11 979 each | 18 each | 0.0 |
| | W± +2j | 131 769 each | 180 each | 0.3 |
| | W± +3j | 1 449 459 each | 288 each | 3.5, 3.7 |
| | W± +4j | 15 944 049 each | 1 284 each | 100.4, 107.0 |
| Z→νν+4j | 4j | 15 944 049 | 1 878 | 130.4 |

```
w4j.new.j4   rc=0 real 260.679 user 561.571 sys 1.927 | enumerated 169884 diagrams in 3456 subprocesses (220.164 s)
znn4j.new.j4 rc=0 real 180.780 user 407.086 sys 1.848 | enumerated 206280 diagrams in 1455 subprocesses (132.542 s)
```

Both finish inside the cap. The handed-on counts are exactly L-T2's
flavour-conserving counts (180, 288, 1 284, 1 878) and the 3j lines' 288
are exactly its non-empty count, so the prune removed every empty subprocess
and nothing else on these lines. Before, the same lines handed feyngraph
13 734 (3j), 45 720 (W 4j) and 60 318 (Z 4j) subprocesses; L-T2 projected
~2900 s for 1009's 4j line at release on 4 threads. The old binary was not
run on these cards (it would not finish in the cap). user/real is 2.2 on 4
threads, so the 4j lines do not use the pool fully: see Found 5.

### Job 3: `p p > e+ e- j j j` (sm), `integrate --check -j4`

Phase records (`wall_s`):

| phase | old | new |
|---|---|---|
| enumerate | 5.233 | 0.435 |
| amplitude_probe | 2.881 | 0.849 |
| compile | 3.191 | 0.042 |
| probe + compile | 6.072 | 0.891 |
| whole run (bash `real`) | 11.38 | 1.41 |

Both: 96 sets, 6400 diagrams, 44 flavour groups, 3008 compiled diagrams,
912 helicities, 210 channels. The saving the probe item asks for: the
`compile` phase's 3.19 s of recompiling is gone (0.04 s left), and the
probe's own compile runs across four cores (2.88 → 0.85 s), so probe and
compile together drop 6.07 → 0.89 s. The old `enumerate` here is the old
default single-thread walk.

## Merge with #186

origin/main b4e284a3 (PR 14, run-global progress) merged at e47435fa, with
conflicts in `vibegraph-cli/src/integrate.rs`, `vibegraph-lib/src/generate.rs`
and `vibegraph-lib/src/multiplicity.rs`, resolved by keeping both sides:

- `multiplicity_groups_of_with` takes #186's shape (all parts first, then one
  `progress::enter_group` scope per part with its `group_labels` label) over
  this branch's (set, evaluator) items, and calls `derive_flavor_groups_with`
  inside the scope. `set_process_ids` takes any iterator of sets, so the
  labels are read without cloning.
- The fixed-energy compile in `integrate` and `generate` is
  `compile_subprocesses_with(…, precompiled, …)` inside #186's
  `sole_group` scope.
- **Progress under the pool.** #186's group is thread-local, so records from
  pool workers would carry no group. `compile_classes` therefore reports no
  progress; `derive_flavor_groups_with` and `compile_subprocesses_with`
  report one `compile` step per set on the calling thread, inside the group
  scope, as each takes its compile results in set order (before each set and
  once at the end, and after each set, as on main). Each group's records come
  from one thread in one loop, so `(phase, group, done)` rises by
  construction; nothing interleaves. The probe phase's compiles report no
  progress, as main's probe did not.
- No phase was added, removed or merged, so `integrate_phases` and
  `generate_phases` are unchanged.
- The merge brought back `generate`'s `--parallel-diagrams` entry from main's
  generated CLI chapter; it was regenerated (`scripts/gen-cli-docs.sh`).

Gates on the merged head: `cargo test --workspace --no-fail-fast` exit 0
(`cli_progress_position` 5 ok, `cli_phase_timing` 4 ok, `cli_reference_docs`
1 ok, after the regeneration), fmt clean, both clippy runs clean, kb-lint 0
errors. The bit-for-bit gate re-run on the merged head (dev, `gate2.sh`,
the commands above, head only) gives at `-j1` and `-j4` ee→μμ
2029.5243067486113 ± 2.5946227539670867, grid af585cd04a180de8, and DY 0–2j
MLM 1063.0574890622572 ± 9.205000902959013, grid afd7adaf4b388939: the
values recorded before the merge.

## Validator host request (draft)

- **Enumerate diff.** Branch `sprintl-timeout-fixes`, head SHA as pushed. For
  every released v022-1009 card that enumerates within the cap, `vibegraph
  enumerate <proc_card> --svg none -j 4 --out <dir>` with v0.2.2 and with the
  head (release); compare `diagrams.json` (ignoring nothing) and
  `diagram_set_digest`. Expected: identical card for card. Report the
  enumerate wall of each.
- **Re-timing.** Cards 58, 15, 1009 and 65 with the campaign command, head
  release, `-j 4` and without `--parallel-diagrams`; the phase records.
  Expected: 58/15/1009 past `enumerate` (≈ 90 s-scale 4j lines, from L-T2's
  estimate); 65's `amplitude_probe` holding its compiles across four cores
  and its `compile` phase near zero.
- **Compile wall** (the serial-loops item): the same phase records on a
  Z(→jj)+0–4j card (65) and a VBS ssWW card (14).

## Brief corrections

- The "compile stage over flavour groups and cuts" in the DY+0–3j numbers
  cannot come from `enumerate --svg none` alone (nothing compiles there); the
  run must have had a run or param card, which also enumerated the card twice.
  The 3 s gap is the zstd-19 encode of the artifact (timed on DY+0–2j, and
  on DY+0–3j release it is the ~3.4 s left after enumeration, job 1).
- "A one-thread walk of a single process line dominates" (the 1.5×): the
  pre-change `--parallel-diagrams` parallelised only inside one subprocess's
  topology assignment, a short contended loop; the walk's subprocesses were
  serial. That, not one dominating line, is what capped the gain.
- The brief's phase guidance assumed #183 would not land; it landed and was
  merged (6ee36331, conflicts in `integrate.rs`/`generate.rs` resolved by
  keeping both the phase records and the shared compile). The merge commit
  carries git's default message, without the `Assisted-by` line.
- `amplitude_probe::measure` / `derive_flavor_groups` are at
  `amplitude_probe.rs:255`/`proton.rs:1079` on main, not 262/1107 (the item
  bodies cite the loop lines).

## Found

1. **The artifact encode is a serial zstd-19 pass over the embedded model.**
   0.43 s of a 0.6 s DY+0–2j `enumerate` (dev), most of it constant per run;
   `integrate`'s artifact uses the same level. Multithreaded zstd changes the
   compressed bytes (not the digests). Close by measuring the encode across
   levels and the MT encoder on the DY+0–3j and a 4j card, and deciding
   whether file bytes are part of any contract.
2. **feyngraph clones the model per subprocess.** `DiagramGenerator::new`
   takes `model.topo.clone()`, and `assign_topologies` clones it again into
   the container (`Arc::new(self.model.as_ref().clone())`), once per
   surviving subprocess; L-T2 measured 11 ms per *empty* subprocess, which
   is that floor. Close with an upstream `Arc` model or a fork rev, and the
   per-subprocess time on `p p > e+ e- j j` before and after.
3. **Pairing splits and on-shell vetoes compile subsets serially.**
   `PairingSplit::new` (`pairing.rs:120`) and `group_vetoes`
   (`onshell.rs:285`) call `compile_class` on sub-sets per group, one at a
   time, inside the compile phase. Close by measuring their share of the
   compile phase on a card with pairing-weighted cuts and a `$` veto.
4. **kb pages still describe `--parallel-diagrams` as the only way to
   parallel enumeration** (`research/kb/pipeline/enumerate-and-diagram-ranking.md:112`,
   `research/kb/phase-space/rng-substreams-and-parallel-determinism.md:118`,
   and the `feyngraph-assign-counts-allocation` item's last paragraph). Not
   edited here (no kb edits outside this report).
5. **The 4j walks use half the pool.** W+4j and Z→νν+4j ran at user/real 2.2
   on 4 threads (job 2). Each 4j line walks 15.9M candidates serially
   (cloning a `String` per leg and sorting a key per candidate) between the
   batches it hands to feyngraph. Close by profiling the 4j line and, if the
   walk dominates, walking the alias product without per-candidate
   allocation or splitting it across the pool in order.
