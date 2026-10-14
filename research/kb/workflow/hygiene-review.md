---
type: Procedure
title: "Running a hygiene review"
description: "How to review code for maintainability, test non-vacuity, visibility and reusable abstractions: measured yield per point, the techniques that found real defects, reviewer calibration, mutation evidence, and a proposed checklist and report shape for a per-PR hygiene agent."
status: draft
tags: [hygiene, review, non-vacuity, visibility, mutation-testing, agents, process]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: sprint, resource: "../sprints/hygiene/", title: "Hygiene sprint folder: plan, decisions, log, triage, protocols and the 21 session reports"}
  - {id: triage, resource: "../sprints/hygiene/triage.md", title: "Hygiene sprint triage: dispositions of 176 findings"}
  - {id: log, resource: "../sprints/hygiene/log.md", title: "Hygiene sprint log: chronology, restarts, the user's decisions, manager gate re-runs"}
  - {id: review-protocol, resource: "../sprints/hygiene/sessions/review-protocol.md", title: "Hygiene review protocol"}
  - {id: fix-protocol, resource: "../sprints/hygiene/sessions/fix-protocol.md", title: "Hygiene fix protocol"}
---
A hygiene review looks at code from four points:
1. **Maintainability.** Duplication, long functions, and comments that contradict the code or narrate its history.
2. **Test non-vacuity.** Asserts that cannot fail on the defect they claim to guard.
3. **Visibility.** `pub` only where something outside the module needs it.
4. **Reusable abstractions.** Patterns that want one home.

This page records what one full pass over the codebase measured: eight read-only reviews, one per module cluster, then a triage, then ten serial fix sessions. It turns those measurements into a design for the hygiene agent in the [per-item PR flow][pr-item] ([hygiene-agent-type][agent-item]).

Every number below comes from the [triage][triage], the [log][log] or a session report. Where a number is derived, the text says how. The finding ids (`R-D.9`) and session names (`F-D`) appear only as citations into those reports.

## The record in brief

| | |
|---|---|
| Reviews | 8, read-only, run in parallel, each on about 9k–30k lines |
| Findings | 175, plus 1 rediscovery of finished work ([triage][triage]) |
| Triage | 125 *fix here*, 50 *filed* (as 35 new items), 4 rejected or close-out only, 3 for the user. Six findings split into a *fix here* part and a *file* part |
| Fix sessions | 10. Each ran serially from the previous session's merged head |
| Mutations run by fix sessions | at least 112 (see [Vacuity proposals](#vacuity-proposals-and-mutations)) |
| Hermetic suite | 1348 passed before the fixes, 1391 after: net +43 tests, with two targets moved into the hermetic layer ([log][log]) |
| Manager spot checks of review claims | 26 checked, 26 reproduced. One line number was off by 20 ([triage][triage]) |

## Yield per point

Each finding is counted under the **first point** its review lists, so a finding marked "2, 1" counts as non-vacuity. Dispositions come from the [triage][triage]'s per-session lists. The per-cluster totals of this count reproduce the triage table exactly.

| point | reported | *checked* | *suspected* | fix here | filed | rejected | user's call |
|---|---|---|---|---|---|---|---|
| 1 maintainability | 73 | 72 | 1 | 60 | 12 | 1 | 0 |
| 2 non-vacuity | 56 | 49 | 7 | 47 | 12 | 0 | 1 |
| 3 visibility | 15 | 14 | 1 | 9 | 5 | 1 | 0 |
| 4 abstractions | 29 | 28 | 1 | 9 | 21 | 0 | 1 |
| none (a disposition, an item's line drift) | 2 | 2 | 0 | 0 | 0 | 1 | 1 |
| **total** | **175** | **165** | **10** | **125** | **50** | **3** | **3** |

- **Fixed.** Every *fix here* finding was reported fixed. Each fix session's gate was re-run by the manager before merge ([log][log]).
- **Not reproduced.** No finding failed entirely on re-verification. Two failed in part, both on point 2 ([F-G1 report][fg1]): one guard never matched, so its test was never silently green, and one "drops two controls" claim held for only one of the two oracles.
- **Rejected at triage.**
  - A hot-path match judged not worth splitting (point 1).
  - Items that integration tests need `pub` (point 3).
  - The rediscovery of finished work.
  - Item line drift, which was left to close-out.
- **Abstractions are mostly filed.** 21 of the 29 abstraction findings needed a new shared type or a cross-module change, which is *file* under the triage rules ([D3][d3]). Maintainability and non-vacuity findings were mostly local (60 of 73 and 47 of 56 were *fix here*).
- **Visibility yield is low after a mechanical pass.** Before the reviews, a compile-driven demotion cut `pub` lines in the library from 2601 to 1359, and to 0 in both binary crates ([V1 report][v1]). The reviews then found 15 visibility issues the compiler cannot see:
  - `pub` fields beside derived state;
  - `pub fn`s that return unnameable types;
  - production docs anchored on test-only items.
- **Real bugs.** The reviews and fix sessions turned up these latent defects:
  - a dropped Lorentz divisor;
  - a locked parameter revived by `recompute`;
  - a substring `scan` refusal;
  - a duplicated `SDE_strategy` predicate whose test pinned the copy production never read;
  - an OR-gated σ check with an effective 1% bound;
  - an 8.2%-stale reference;
  - a signed colour in the configuration tag;
  - a run-card `split_once` that skips a banked line.

  Sources: [triage][triage], [F-C][fc], [F-F][ff].

**Cost.** In the eight reviews' method tables, non-vacuity work took 20–55% of each session and produced most of the fix-here test findings. Visibility, where it was a separate step, took 5–15%, and the backlog check took 10–15%. On the fix side, a mutation costs about 3 minutes of rebuild and the full hermetic suite about 15 minutes ([F-B report][fb]).

## What worked

**Techniques that produced checked findings:**

| technique | point | evidence |
|---|---|---|
| Grep a definition and find no other line | 1, 3 | the most productive technique on its cluster ([R-B][rb]) |
| Ask, for each test, which one-line mutation its asserts are blind to | 2 | 35% of a review, 8 findings ([R-B][rb]) |
| `grep -E 'exists\(\)\|return;\|read_dir\|catch_unwind'`, then ask what each hides | 2 | the fastest technique: zero-trial greens, any-panic arms ([R-G1][rg1]) |
| Grep an accessor's readers across the whole workspace | 1, 2 | the duplicated predicate with the unread copy pinned ([R-E][re]) |
| For each `cfg(any(test, doc))` item, find the production doc that links it | 3, 1 | contracts documented on test-only APIs ([R-D][rd]) |
| Diff the same statistic across gate files and against the written policy | 2, 4 | 40% of a review, 8 findings ([R-G2][rg2]) |
| Count each error enum's variant constructions against the tests that name them | 2 | 10 of 15 variants untested ([R-C][rc]) |
| Compare a grammar table with the upstream source | 2 | `asin`/`acos`, the dropped divisor ([R-C][rc]) |
| Do the arithmetic on a tolerance against the quantity's real magnitude | 2 | a squared-norm gate that is blind to a zero Z current ([R-A][ra]), and 13 squared-norm sites ([R-B][rb]) |
| Grep each user-facing complaint string against the tests | 2 | 12 checks, 1 tested ([R-F][rf]) |

**Low yield or dead ends:**
- Function-length censuses, tolerance regexes and an assertion-less-test awk census were too loose to settle anything ([R-E][re]).
- Outlines and length ranking produced nothing in one review ([R-G2][rg2]), as did the colour engine plus a length scan, at 15% of another ([R-B][rb]).
- Suspected convention bugs mostly checked out: four in one cluster ([R-B][rb]).
- Loose-looking bounds often carried a measured justification ([R-E][re]).

**Corroboration.** The same defect class found independently in two clusters is the strongest signal for a checklist line ([triage][triage]). There were four such classes:
- the dropped divisor;
- any-refusal oracles;
- OR-gated σ checks;
- test-only contract anchors, found by four reviews.

**Oracles for "this refactor changed nothing."** Every fix session that restructured code chose an identity oracle before editing:
- a `Debug` digest of the emitted program for 45 subprocesses ([F-A][fa]); an op census or a dump could not prove identity;
- `to_bits` equality of the seeded VEGAS goldens ([F-D][fd]);
- byte-identical collator output on 157 real rows ([F-CLI][fcli]);
- a byte-identical per-row summary on 48 tables ([F-G1][fg1]).

## Calibration

Reviewers marked each finding *checked* (the evidence settles it) or *suspected* (the report says what would settle it).

- **Checked: 165 of 175.** 121 went to *fix here*. None failed outright when the fix session re-verified it, and two failed in part (above). The common error was a count or a site, not the claim:

  | finding | stated | corrected | source |
  |---|---|---|---|
  | squared-norm sites | 13 | 11 live | [F-B][fb] |
  | refusal-helper callers | 9 | 8 | [F-CLI][fcli] |
  | `rel_tol` ratios | 10–28× | 9.9–29.9× | [F-G2][fg2] |
  | run cards checked | 3 of 20 | the tracked set is 19 | [F-G2][fg2] |
  | which `mg_dot` copy has the right citation | the `scales.rs` one, per the brief | the `kt.rs` one | [F-E][fe] |

  One *checked* finding understated its case. An asserting gate hidden under `#[ignore]` was not merely ignored: its bound was false at the base, at 72–83% where it asserted 50% ([F-A][fa]).
- **Suspected: 10 of 175 (6%).** Four went to *fix here*, and all four held up when tested:
  - the Z-path tolerance;
  - the pooled-error stop;
  - the loose σ-recovery gates;
  - the frozen-grid replay, where the defect turned out to be invisible to σ altogether.

  The other six were filed untested. The manager reproduced the code shape of one of them, the any-refusal oracle.
- **Mixed labels.** Twelve findings were marked "checked" with a *suspected* secondary part, such as "checked; coverage gap suspected". The split was useful: it told triage which half was settled and which still needed a test or a measurement.
- **Reasoned, not run.** Every review reasoned its mutations from the code, because the reviews were read-only and built nothing. The fix sessions' runs (below) showed that this reasoning was rarely wrong about *whether* a test was vacuous, and sometimes wrong about *how much power* the replacement needed.

## Vacuity proposals and mutations

The [fix protocol][fp] required every tightened or new assert to pass on the fixed code and fail on a temporary, reverted mutation.

**Mutations run:** at least 112.
- Five reports state a total: 13 ([F-B][fb]), 18 ([F-C][fc]), 1 ([F-C2][fc2]), 6 ([F-F][ff]) and 32 ([F-CLI][fcli]).
- For the other five, the count is the mutations listed in their evidence tables: 5 ([F-A][fa]), 6 ([F-D][fd]), 6 ([F-E][fe]), 10 ([F-G1][fg1]) and 15 ([F-G2][fg2]).

**Point-2 *fix here* findings confirmed by a run mutation: 41 of 47.** The six without one:
- a doc-only half;
- an invariant moved onto the production path;
- two vacuous tests deleted, leaving nothing to mutate;
- one claim not reproduced;
- one stale reference replaced by a read of the bank.

**Both halves shown for 20 of the 41.** For those, the report gives the pre-fix test passing the mutation and the new one failing it:
- R-A.1, .14, .20 and R-B.21;
- R-F.5, .21, .25;
- R-G1.1, .2, .3, .4, .6, .14;
- R-G2.2, .3, .4, .5, .14, .19, .21.

Counted from the evidence columns of the fix reports. The other 21 report only the new test failing.

**Where the first tightening did not catch its mutation.** These are the cases a hygiene agent must expect:

| case | what happened | lesson |
|---|---|---|
| Overweight test at 400k trials ([F-D][fd]) | Missed a 1.5% mutation (about 3σ). It now runs 2M trials (about 7σ) | Compute the test's power from its own error *before* running. The miss was predictable |
| Overflow tripwire ([F-B][fb]) | It matched `"overflow"`, which the debug-build panic also prints, so it passed with the checked arithmetic removed | Match the message of the path under test, not a word every failure shares |
| CLI refusal cases ([F-CLI][fcli]) | clap rejected `--max-truncation -0.01` and `--scan-points -5` as unknown arguments, so the value parsers never ran | A refusal test must assert *which* refusal fired; `--flag=value` reaches the parser |
| Squared-norm sites ([F-B][fb]) | No mutation was found that fails three of the converted sites | Report the sites a mutation cannot reach rather than claim them |
| e⁺e⁻→μ⁺μ⁻ current tests ([F-B][fb]) | The fix pinned the normalisation; a flip of the L/R relative sign still passes | A fix can move a blind spot rather than remove it. Name the remaining blind spot in Found |
| Frozen-grid replay ([F-CLI][fcli]) | A wrong-map replay passes any σ check on single-channel Drell–Yan. An efficiency bound reads 23–29× on it, against 1.00–1.07× on the fixed code | When the proposed tightening is the wrong oracle, change the oracle (see [oracle blind spots][blind]) |

New bounds came from a recorded reading (the census, a manifest note, a banked error) or a stated multiple of the run's own error, never from a fresh fixed-seed number ([D3][d3]). Two fix sessions found that "recorded" can be stale:
- the census no longer described the tip, so bounds used the larger of the census and a tip re-run of the same probe ([F-G2][fg2]);
- one bound was checked with a 20-seed sweep, max |pull| 1.45–2.98 against a gate of 5 ([F-D][fd]).

## Scope for the agent

**Seen in a single PR's diff.** A per-PR session can run cargo, so its mutations are run rather than reasoned. It can check what the diff adds or touches:
- comments and docs on changed items, and any plan, history or session reference the diff introduces;
- every new or changed test, against the checklist below;
- every new `pub` item, and every new `cfg(test)` or `cfg(any(test, doc))` item;
- whether the diff adds a second copy of something that already exists. Grep the new function's name and a distinctive line of its body across the workspace;
- refactor identity, by an oracle chosen before editing;
- kb and backlog sites the diff makes stale, reported as Found.

**Seen only by a whole-codebase pass.**
- **Cross-module duplication.** Examples: about six Lorentz boosts, three categorical draws, four "final-state side" computations, three `WEIGHTED` computations, and five row-to-model resolutions. None of these is visible from one diff, and two-reviewer corroboration needs several clusters ([triage][triage]).
- **Module size and boundaries.** Examples: 4.7k- and 6.5k-line modules, and an import cycle ([R-E][re]).
- **The visibility census and the dead code it exposes.** One demotion pass needed about 50 compile iterations and left 144 `dead_code` sites to classify ([V1][v1], [V1b][v1b]).
- **Test-only scaffolding kept alive by its own tests.** Example: about 150 lines of a parallel scheme with no production caller ([R-D][rd]).
- **Drift against pins and measurements.** Examples: about 33 unverified MadGraph line citations ([R-F][rf]), census drift ([F-G2][fg2]), and two kb concepts that contradict each other on seed policy ([R-G2][rg2]).
- **Layer registration.** Example: hermetic inputs registered as banked. Audit every `required-features` target's inputs against `git ls-files` ([R-G1][rg1]).

A per-PR agent therefore keeps new debt out, and the whole-codebase pass stays a periodic sweep.

### Proposed checklist

For the agent definition, in the shape of `.agents/agents/validation-dev.md`'s focus section.

**1. Maintainability**
- Comments describe the code as it is: no plan, session, note or history references. Grep `\bT[0-9]\b|\bC[0-9]\b|Phase [0-9]|note [0-9]+|used to|no longer`.
- Docs match bodies. Check signs, formulas, and the names of items that still exist.
- No function the diff adds or grows is over about 100 lines without a named reason.
- No second copy of logic that exists elsewhere. Grep the name and a body line workspace-wide.
- No hand-written standard primitive: gcd or rational arithmetic, temp directories, permutation checks, `gzip` shell-outs (AGENTS.md).

**2. Non-vacuity**, for each new or changed test:
- Name the mutation it must catch. Run it, and report the pre-fix and post-fix results.
- Compute the test's power from its own error before choosing N or a bound.
- Tolerances are linear and relative at the algorithm's own error scale: no squared norm against a linear ε, and no absolute bound on a quantity spanning orders of magnitude.
- No OR between a pull and a relative bound. No exit-code-only or any-refusal assert; match the variant or the message.
- No green on an empty set: zero trials, an empty census, a missing `output/`. Banked gates go through `validation::require`.
- No `catch_unwind` that discards the payload, and no swallowed `read_dir` error.
- Lists checked both ways (allowlists, coverage tables), and counts derived independently of the input they check.

**3. Visibility**
- Every new `pub` has a user outside its module, or it is narrowed.
- No `pub` field beside derived state, and no `pub fn` returning a type its callers cannot name.
- A production contract is documented on the production function, not on a test-only API.
- Dead code is gated or deleted, never `#[allow(dead_code)]` ([D2][d2]).

**4. Abstractions**
- A pattern repeated inside the diff's module is merged there.
- A pattern repeated across modules goes to Found, with every instance named.
- A new knob that every caller passes as `default()` is questioned.

**Refactors:** choose the identity oracle before the first edit, and run it at the end.

### Proposed report shape

These sections keep the calibration measurable, as the sprint's did.

- **Findings**: id, point, `path:line`, the evidence (a command with its output, or quoted lines), the mutation, the disposition (*fixed*, *Found* or *leave*) and the confidence (*checked* or *suspected*).
- **Mutations**: each one's command, its pre-fix result and its post-fix result, with the sites no mutation could reach.
- **Gate**: each command with the tail of its output.
- **Method**: the techniques used and their shares, and which leads failed.
- **Found**: cross-module patterns, kb or backlog sites made stale, and real bugs outside the stop-rule ([session scoping][scoping]).
- **Brief corrections.**

**Open questions for the definition:**
- Should the agent fix what it finds in the same session, or report first? The sprint kept review and fix apart so that hit rates stayed measurable ([D3][d3]). On a PR's diff the scope is small enough to do both, provided the findings table is written before any fix.
- Which model should it run on? Every review and every fix session the log records ran on Opus; the tooling session ran on Sonnet.

## Protocol corrections

From the reports' "Brief corrections" sections.

**Review protocol**
- **`git branch --show-current` is empty on a detached checkout.** All eight reviews hit this. Verify with `git rev-parse --short HEAD`.
- **`pixi` was absent.** Use `python3 scripts/kb.py backlog --item` ([R-E][re]).
- **The `mg5amcnlo` submodule was empty in review worktrees,** so Fortran citations could not be checked ([R-C][rc], [R-E][re], [R-F][rf]). Review worktrees need it as fix worktrees do.
- **Bare-name backlog greps are noisy.** One name matched 34 items, and about half the hits on another cluster were substring matches. Grep path tails such as `/vegas.rs` ([R-C][rc], [R-D][rd]).
- **Judging a cluster can need a caller grep outside it.** Allow that explicitly ([R-B][rb]).
- **The protocol said *leave* where [D3][d3] defines *rejected*** ([R-G2][rg2]).
- **Cluster line counts in briefs were off by up to about 15%** ([R-G1][rg1], [R-G2][rg2]).

**Fix protocol and briefs**
- **Program identity.** An op census or a dump cannot prove the program unchanged. A `Debug` digest that leaves out `HashMap`-bearing state can ([F-A][fa]).
- **Feature-gated tests.** Some tests the brief names compile only under `extended-validation` ([F-A][fa]).
- **Package names.** The library package is `vibegraph-lib` and its target is `vibegraph` ([F-B][fb], [F-E][fe]).
- **`cargo doc`.** `cargo doc --workspace` collides on the lib/bin `vibegraph` doc name, so run it per package ([F-C][fc], [F-CLI][fcli]).
- **Inner loop.** `cargo check --tests` (about 15 s) is the better inner loop ([F-C][fc]).
- **Briefs need every caller and every kb site.** One missed a second CLI caller ([F-C2][fc2]) and one listed no kb sites ([F-D][fd]).
- **Item premises can be wrong at the upstream source.** One item's prescribed weight reads an unassigned variable in MadGraph's own `genps.f`, and the fix became a refusal ([F-E][fe]). Another item could not close locally and needed the user's decision ([F-C][fc]).
- **`closes_when` can fail to fit the code** ([F-F][ff]).
- **Visibility work.**
  - `cfg(test)` alone breaks links from production docs; use `cfg(any(test, doc))`.
  - A field production writes cannot take `cfg(test)`.
  - The gate must include CI's default-feature clippy, and a doc build, the only check that sees breaks in `cfg(any(test, doc))` items ([V1][v1], [V1b][v1b]).

## Operational lessons

- **Manager re-run of every gate.** Each fix session's report was a claim until the manager re-ran fmt, both clippy configurations, the hermetic suite and the touched banked targets. The suite count was then reconciled against the tests the session said it added and deleted ([log][log]). The trust tiers are in [sprint lifecycle](sprint-lifecycle.md).
- **σ-drift attribution.** A fix session found σ calibrations drifted against the census. The manager ran the same probe (`probe_gate_row_seed_headroom`) at the pre-sprint commit and diffed it: all 68 `HEADROOM` lines were identical, digit for digit. So the sprint moved no σ, and the drift predates it ([F-G2][fg2], [log][log]). Any "did this change move a number?" question should get the same treatment.
- **Worktree reuse and disk.**
  - Reusing a warm worktree saves a rebuild, but the debug incremental cache and stale test binaries grow without bound.
  - The manager cleared about 9 GB before one session, and again when free space fell to 2 GB.
  - One run filled the disk to 287 MB free, and deleting stale test executables recovered about 25 GB ([log][log], [F-A][fa]).
  - Clear `target/debug` before dispatching into a reused worktree.
- **`CARGO_INCREMENTAL=0` for gate runs.** After the second disk incident, gates ran with incremental compilation off ([F-A][fa], [F-B][fb]).
- **Container restarts.** One fix session survived two restarts ([log][log]):
  - The first lost the process with no commits, but 24 files of uncommitted work survived on disk. The manager saved a backup patch and resumed the agent from its transcript with a reconciliation step: verify, commit in checkpoints, re-run what was in flight.
  - The second came after five commits, with the gate in flight.

  Commit at checkpoints so a restart costs less.
- **Fresh agent instead of a resumed one.** A follow-up to a session whose transcript reached about 333k tokens went to a fresh agent seeded with the report ([log][log]).
- **Mutation batching.**
  - Apply each group of edits with an assert-checked patch script and build once ([F-B][fb]).
  - Drive mutations with a script that applies, runs and reverts each one and logs the result ([F-B][fb]).
  - Six mutations fit in one batched build ([F-F][ff]).
- **Inputs missing from worktrees.**
  - The `mg5amcnlo` submodule content: three reviews could not check Fortran citations without it.
  - The multigrid PDF set `NNPDF31_lo_as_0130`: without it a banked target passed 7 of 20, and once fetched, 20 of 20 ([F-E][fe]).

  Copy both in with the reference bundle ([agent dispatch][dispatch]).
- **`pkill -f` kills its own wrapper.** One session killed its own shell twice this way, once during a gate run. Kill by PID ([F-C][fc]).
- **Destructive git in scripts.** A stray `git checkout -q -- .` was blocked by the permission classifier before it discarded a pass of work ([V1][v1]).
- **Hunk splitting.** `git apply --cached --unidiff-zero` misplaced insertions in an intermediate commit ([V1b][v1b]).

## Limits of this record

- **One sprint, one codebase, one reviewer model.** The rates are this record, not priors.
- **Untested filed findings.** Six of the ten *suspected* findings were filed without a test, so the 4-of-4 hold rate for *suspected* findings rests on four cases.
- **The mutation total is a floor.** Five sessions report no total, and their counts are read from evidence tables.
- **The reviews could not run cargo.** A per-PR agent will run its mutations, so its *checked* share should be higher and its *suspected* share near zero.

[triage]: ../sprints/hygiene/triage.md
[log]: ../sprints/hygiene/log.md
[rp]: ../sprints/hygiene/sessions/review-protocol.md
[fp]: ../sprints/hygiene/sessions/fix-protocol.md
[d2]: ../sprints/hygiene/decisions/D2-visibility-mechanical-demotion.md
[d3]: ../sprints/hygiene/decisions/D3-fix-small-file-large.md
[v1]: ../sprints/hygiene/sessions/V1-report.md
[v1b]: ../sprints/hygiene/sessions/V1b-report.md
[ra]: ../sprints/hygiene/sessions/R-A-report.md
[rb]: ../sprints/hygiene/sessions/R-B-report.md
[rc]: ../sprints/hygiene/sessions/R-C-report.md
[rd]: ../sprints/hygiene/sessions/R-D-report.md
[re]: ../sprints/hygiene/sessions/R-E-report.md
[rf]: ../sprints/hygiene/sessions/R-F-report.md
[rg1]: ../sprints/hygiene/sessions/R-G1-report.md
[rg2]: ../sprints/hygiene/sessions/R-G2-report.md
[fa]: ../sprints/hygiene/sessions/F-A-report.md
[fb]: ../sprints/hygiene/sessions/F-B-report.md
[fc]: ../sprints/hygiene/sessions/F-C-report.md
[fc2]: ../sprints/hygiene/sessions/F-C2-report.md
[fd]: ../sprints/hygiene/sessions/F-D-report.md
[fe]: ../sprints/hygiene/sessions/F-E-report.md
[ff]: ../sprints/hygiene/sessions/F-F-report.md
[fcli]: ../sprints/hygiene/sessions/F-CLI-report.md
[fg1]: ../sprints/hygiene/sessions/F-G1-report.md
[fg2]: ../sprints/hygiene/sessions/F-G2-report.md
[dispatch]: agent-dispatch-and-worktrees.md
[scoping]: session-scoping-rules.md
[blind]: ../validation/oracle-blind-spots-and-non-vacuity.md
[pr-item]: ../decisions/pr-per-backlog-item.md
[agent-item]: ../backlog/hygiene/hygiene-agent-type.md
