# Research knowledge bundle update log

## 2026-10-09
* **Migration**: The bundle moved to `research/kb/` (note 42 Phase 4). The 54 research notes are archived verbatim under [history/notes/](history/notes/index.md) as deprecated Working Notes, each listing the concepts that replaced it (`replaced_by`). The backlog, the reviewed decisions and the Sprint Records moved in beside the topic folders; the Phase B facts and the pipeline status table retired into the concepts that absorbed them.
* **Verification**: All 229 topic concepts carry a machine-tier `verified` stamp after an adversarial pass against the notes, the code and MadGraph at the pin (note 42 Phase 3); they stay `status: draft` until a person reviews them.
* **Creation**: 229 topic concepts drafted from the research notes under the approved taxonomy (note 42 Phase 2).
* **Decision**: Release scope is arbitrary-multiplicity LO with MLM merging; after the migration, one PR per backlog item with each session type in turn, starting with a hygiene sprint; MLM on decays is refused for good ([decisions/](decisions/index.md)).

## 2026-10-06
* **Initialization**: Frontmatter added in place to every research note (note 42 Phase 0). `generated.by` on the pre-existing notes is `claude-code` without a model version, which their history no longer records, and `generated.at` is the note's own date or, where it gives none, an upper bound from git.
* **Creation**: `TODO.md` split into one-file backlog items (note 42 Phase B), with Sprint Record stubs, scope decisions, standing facts and the pipeline status table.

## 2026-10-10
* **Hygiene sprint closed** ([record](sprints/hygiene/closeout.md), [nsmith-/vibegraph#19](https://github.com/nsmith-/vibegraph/pull/19)).
  - Eight cluster reviews produced 176 findings, and ten fix sessions fixed 125 of them.
  - 25 backlog items closed, one released, and 54 filed.
  - The draft [hygiene review](workflow/hygiene-review.md) procedure records the lessons for the hygiene agent.
  - The kb concepts the code changes made stale were updated.
  - The full banked validation is green on the final code.

## 2026-10-14
* **Decision**: The user reviewed the hygiene sprint's lessons, protocols and record. The hygiene agent, [`hygiene-dev`](../../.agents/agents/hygiene-dev.md), synthesises the review and fix protocols: it runs last in the PR sequence, fixes what it finds, routes physics findings to `validation-dev` and performance findings to `performance-dev`, and is dispatched on Opus or Sonnet by the change's scope ([hygiene review](workflow/hygiene-review.md#decided), [pr-per-backlog-item](decisions/pr-per-backlog-item.md)).
* **Decision**: Fixed beams refuse closed-form scale choices 1, 2 and 5, which have no reference run; supporting them is [fixed-beam-closed-form-scales-1-2-5](backlog/feature/fixed-beam-closed-form-scales-1-2-5.md) ([scale choices](scales-pdf/madgraph-scale-choice.md)).
* **Decision**: Seeds of equal budget, on this side and MadEvent's, are read as their unweighted mean with error `max(quoted, spread/√n)` and a separate χ²/dof scatter check, through one helper; the three-seed hadronic σ gates move to five seeds ([seed policy](validation/madevent-reference-seed-policy.md)). The `$$ z` grammar row's budget climb was an artifact of the old `1/σ²` mean ([seed sweeps](validation/seed-sweeps-and-budget-ladders.md)).

## 2026-10-15
* **H1 census closed** ([record](sprints/sigma-h1-ickkw0-census/closeout.md), [nsmith-/vibegraph#24](https://github.com/nsmith-/vibegraph/pull/24)): every banked grouped row checked for the permuted first call; only `pp_to_ll_0j2j_mlm` is measurably affected, and no reference needed re-banking.
* **t t~ MLM @1 excess attributed** ([record](sprints/sigma-ttx-mlm-at1/closeout.md), [nsmith-/vibegraph#23](https://github.com/nsmith-/vibegraph/pull/23)): the hadronic integrand clustered MLM scales in the lab frame (fixed; −4.47 pb on `@1`), and MadEvent's default survey under-integrates `g q > t t~ q` ([measurement](validation/madevent-default-survey-bias.md)); the cell gates once re-banked with `hard_survey = 2`.
