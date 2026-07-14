# 17 — Hot-loop bounds/capacity-check elimination: feasibility memo

**Status:** Feasibility investigation + go/no-go (2026-07-14). Companion to
`research/notes/15-eval-optimization-plan.md` §2 (the `eval-layout` sprint). Scopes
whether the typed SoA forward pass (`helas/eval/run.rs::fill_arenas`) can have its
remaining hot-loop branches — arena index bounds checks and `Vec::push` capacity
checks — removed *without* `unsafe`, and whether the payoff justifies a production
pass.

All measurements are on Apple Silicon (arm64), `rustc 1.94.1`, `--profile profiling`
(release codegen + debuginfo), single-threaded, over the MadGraph reference momenta.

---

## 1. The question

The typed forward pass reduces each node from its children's already-computed results,
reading operands out of five per-class result arenas (`reals`/`scalars`/`vectors`/
`fin`/`fout`) and pushing its own result to the arena its static output class selects.
Two branch classes remain in that O(nodes) loop:

1. **Arena index bounds checks** on every operand read (`scratch.vectors[i]`,
   `scratch.reals[mass]`, the `ops[start..start+len]` operand sub-slice, `slice[0]`,
   the mixed-class `Mul`/momentum read-offs).
2. **`Vec::push` capacity checks** on every result write. `reset` reserves each arena to
   its exact final length (`arena_sizes`), so the grow path is *never taken*, yet each
   push still branches `len == cap` and carries a cold `grow_one` call.

Both are safe-Rust artifacts of indexing/growing heap collections, not intrinsic to the
arithmetic. The investigation: are they costing anything, and can they be removed safely?

## 2. The branches are really there (claim verification)

Disassembly of the monomorphised (`f64`) `fill_arenas` in a **release** build with the
per-node analysis cross-check compiled out (no `extended-validation`, the shipping
configuration) confirms exactly these branch classes in the loop body:

| branch target | count | class |
|---|--:|---|
| `core::panicking::panic_bounds_check` | 57 | arena read bounds checks (target 1) |
| `slice::index::slice_index_fail` | 6 | `ops[start..start+len]` sub-slicing (target 1) |
| `RawVec::grow_one` | ~32 | `push` capacity-check cold path (target 2) |
| `RawVec…reserve…do_reserve_and_handle` | 5 | the five `reset` reserves (once per run) |
| `panic_fmt` / `assert_failed` | 7 | the `Add*` momentum `assert_eq!` (target 4 — out of scope) |

The kernels themselves (`gamma_vout_c`, `propagate_*`, …) are almost entirely inlined;
the branch targets above dominate the non-arithmetic control flow. The claim in note 15
§2 (A3b bullet) is confirmed: the remaining hot-loop branches are the arena bounds
checks and the push capacity checks. **Note:** the A0 timing rig
(`tests/instruction_size_bench.rs`) requires `extended-validation`, which compiles the
per-node cross-check *into* the loop; a bounds-check microbench must be built without it
(as the rig here is) or the cross-check's own bounds checks and `resolve_mom` swamp the
signal.

## 3. Method

A measurement-only probe (feature `a3b-probe`, `run.rs`, never built into production)
provides a twin of the forward pass, `fill_arenas_probe<RU, WU>`, const-generic on
whether the arena bounds checks (`RU`) and push capacity checks (`WU`) are removed —
reads via `get_unchecked`, writes into reserved spare capacity via `set_len`. The const
generics fold at monomorphisation, so each variant compiles to exactly one path.
`RU = WU = false` is behaviourally identical to `fill_arenas`; the four `BoundAmplitude`
entry points (`eval_m2`, `eval_m2_unchecked`, `eval_m2_uncheckedreads`,
`eval_m2_uncheckedwrites`) share it. `unsafe` is confined to this probe and does not
appear in any shipping code path.

Correctness oracle: the microbench (`tests/a3b_bounds_bench.rs`) asserts every probe
twin reproduces the checked `eval_m2` **bit-for-bit** (`to_bits()` equality) for every
reference point before timing — so the numbers below are for a provably order- and
value-preserving transformation, and the twins cannot silently drift from the checked
path.

Timing: each process's AST is compiled **once** and reused (note 15 §4.2 — lowering is
nondeterministic across process invocations); ns/eval is amortised over the reference
points in ~200 ms blocks, min over 15 blocks after a discarded warmup, with the
median−min spread as the noise floor.

## 4. Results

Min ns/eval, `--profile profiling`, `--test-threads=1`. Percentages are speedup vs the
checked `eval_m2` baseline. `both` = remove reads+writes; `reads` = remove only bounds
checks; `writes` = remove only capacity checks.

| process | nflows | base ns/eval | spread | both% | reads% | writes% |
|---|--:|--:|--:|--:|--:|--:|
| ee_to_mumu_tata_qcd0 | 1 | 103 926 | 0.6% | **+7.2** | −2.0 | +0.2 |
| gg_to_ttx | 2 | 6 099 | 0.4% | **+9.8** | −0.4 | +4.2 |
| uux_to_uux | 2 | 3 636 | 0.6% | **+7.9** | −2.0 | +0.6 |
| gg_to_gg | 6 | 13 698 | 0.6% | **+10.6** | −3.5 | +7.0 |
| uux_to_ccx_emmm_qcd0 (2→6) | 1 | 7 798 294 | 0.2% | **+7.1** | −1.3 | +0.8 |

A separate earlier run reproduced the `both` column within run-to-run noise (7.1–10.0%),
so the ceiling is stable, not a fluctuation.

## 5. Interpretation — the win is coupled, not additive

The three columns are the load-bearing finding:

- **The ceiling is real: +7 to +11%**, comfortably above the 0.2–0.6% noise floor. The
  hot loop is *not* purely arithmetic-bound as note 15 §1.5 hypothesised — these
  never-failing branches cost measurable time.
- **Removing the read bounds checks *alone* is neutral-to-negative** (−0.4 to −3.5%). By
  itself, dropping the 57+6 read checks does not help and often hurts (register
  pressure / code-layout shifts with no compensating branch removed).
- **Removing the push capacity checks alone captures part** of it — 7% for gg_to_gg,
  4.2% for gg_to_ttx — but almost nothing for the 1-flow and 2→6 giants (0.2–0.8%).
- **Only removing *both together* reaches the full ceiling**, and the two are
  super-additive on the giants (e.g. uux 2→6: reads −1.3, writes +0.8, both +7.1).

This is a coupled codegen effect, not two separable savings. The `push` growth branch
carries a cold `grow_one` call; a never-taken call still forces the optimiser to preserve
a cold path and spill caller-saved registers around every write, and the bounds-check
panic paths interact with that spill pattern. The win materialises only when the loop
body is freed of *both* cold-path branch families at once so the whole reduction
schedules tightly. Any partial removal leaves the coupling intact.

## 6. Candidate mechanisms — none capture the ceiling safely

The plan named three safe mechanisms. Each fails for a structural reason, and §5 says why
a partial fix is worthless even where it "works":

- **Bind-time pre-resolution into `&'a mut T` / `&'a [Cell<T>]` / split borrows.** The
  operand *values* are recomputed every phase-space point (the arena is refilled per
  point), so any per-point pre-pass that turns instruction indices into references must
  itself index the arena — bounds-checked — merely relocating the check. `Cell` resolves
  the aliasing (a result is read by many later instructions) but a `cells[idx]` access
  still bounds-checks. No win.
- **Provably-elidable checks (hoisted asserts, exact sub-slicing, pre-sizing).** LLVM can
  drop a per-access check only when it can bound the index. Operand indices are opaque
  `u32` *data* from the instruction stream, not loop induction variables; a hoisted
  `assert!(idx < len)` immediately before `arena[idx]` removes the *second* (redundant)
  check but keeps the first — same branch count. Bounding requires either `idx % len`
  (extra work, changes semantics) or const-size arrays (arena sizes vary per process) —
  neither available. On the write side, pre-sizing each arena and writing by a monotone
  cursor into `&mut [T]` trades the capacity-check for a bounds-check on the cursor —
  *the same cold-path branch family* — and adds an O(nodes) default-fill per point;
  §5 shows swapping one read/write branch for another does not help (reads-only is
  negative), so this is a wash-or-worse. No safe write elision exists that avoids both a
  branch and `set_len` (`unsafe`).
- **`OnceCell<T>`-element arenas** (user-suggested). Two independent disqualifiers, both
  confirmed head-on:
  1. *The get-path check just replaces the bounds-check branch.* `OnceCell::get` returns
     `Option<&T>`; the is-initialised test is a branch. §5's `reads` column already shows
     that removing the read branch *entirely* is neutral-to-negative — so replacing it
     with a *different* branch (the init check) is strictly no better. This is the caveat
     note 15 flagged, now measured rather than assumed.
  2. *Write-once semantics are incompatible with A5.* `OnceCell` cannot overwrite an
     initialised slot (no `set` after init; no interior reset in std). A5's cross-helicity
     recycling persists results in the arenas across helicity combinations and
     **overwrites** the changed ones — exactly the reset/overwrite semantics `OnceCell`
     lacks. Adopting it would wall off the largest downstream win for a benefit that
     measures as zero.

The only mechanism that reaches the measured ceiling is `get_unchecked` reads +
spare-capacity (`set_len`) writes — i.e. **`unsafe`, explicitly out of scope** for this
track. There is no safe substitute because (a) the read indices are unbounded data and
(b) the write win specifically comes from deleting the cold-call branch, which safe code
can only relocate.

## 7. Go/no-go for A3c

**NO-GO under the current (no-`unsafe`) scope.** The recoverable ceiling is real and not
negligible (+7 to +11%, ~13.7→12.2 µs/eval on gg_to_gg), but it is a coupled reads+writes
codegen effect that only materialises when *both* check families are removed together,
and no safe mechanism can remove the data-indexed read checks or delete (rather than
relocate) the write branch. A partial safe fix does not bank a proportional fraction —
reads-only is negative, writes-only helps only the multi-flow gluon processes.

Two paths for the manager to choose between (a scope decision, not an implementation one):

- **(a) Cancel A3c.** The +7–11% is worth less than the slot-traffic / momentum-pool
  wins A4 targets (payload shrink 96→64 B), and is unreachable within the no-`unsafe`
  constraint. Spend the effort on A4/A5.
- **(b) Re-scope A3c to a narrowly-audited `unsafe` core.** The full ceiling needs only
  ~40 lines: `get_unchecked` arena reads (justified by A1's static output-type +
  location analysis guaranteeing every operand index is in range by construction) and
  spare-capacity writes (justified by `reset` reserving exact `arena_sizes`). It would
  ship behind the full `validate_helas_mg` bit-for-bit gate plus the equality-twin test
  built here (`tests/a3b_bounds_bench.rs`, which asserts `to_bits()` equality against the
  checked path). This is the only way to capture the measured win.

Because A3b's charter is *safe* removal and `unsafe` is out of scope, the default
recommendation is **(a) cancel**, escalating **(b)** as an option if the manager judges
+7–11% on the gluon/2→6 processes worth a small, gated `unsafe` block. Either way A3c as
originally framed ("safe production implementation of the chosen mechanism") has **no
chosen mechanism** — there isn't a safe one.

## 8. Artifacts (branch `eval-layout/a3b`, throwaway probe)

- `helas/eval/run.rs` — `#[cfg(feature = "a3b-probe")]` block: `fill_arenas_probe<RU,WU>`
  and the `eval_m2_unchecked{,reads,writes}` entry points. Compiled out of every build
  without the feature; production `fill_arenas` unchanged, 256 lib tests green.
- `vibegraph-lib/Cargo.toml` — `a3b-probe` feature (measurement only).
- `tests/a3b_bounds_bench.rs` — the decomposition microbench + bit-for-bit twin oracle.

Reproduce:
```
cargo test -p vibegraph-lib --profile profiling --features a3b-probe \
  --test a3b_bounds_bench -- --test-threads=1 --nocapture
```
Branch-claim disassembly (release, cross-check compiled out):
```
cargo bench -p vibegraph-lib --bench eval_strategies --no-run
objdump -d target/release/deps/eval_strategies-*  # inspect `fill_arenas`
```
