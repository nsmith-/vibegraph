# Research knowledge bundle update log

## 2026-10-09
* **Migration**: The bundle moved to `research/kb/` (note 42 Phase 4). The 54 research notes are archived verbatim under [history/notes/](history/notes/index.md) as deprecated Working Notes, each listing the concepts that replaced it (`replaced_by`). The backlog, the reviewed decisions and the Sprint Records moved in beside the topic folders; the Phase B facts and the pipeline status table retired into the concepts that absorbed them.
* **Verification**: All 229 topic concepts carry a machine-tier `verified` stamp after an adversarial pass against the notes, the code and MadGraph at the pin (note 42 Phase 3); they stay `status: draft` until a person reviews them.
* **Creation**: 229 topic concepts drafted from the research notes under the approved taxonomy (note 42 Phase 2).
* **Decision**: Release scope is arbitrary-multiplicity LO with MLM merging; after the migration, one PR per backlog item with each session type in turn, starting with a hygiene sprint; MLM on decays is refused for good ([decisions/](decisions/index.md)).

## 2026-10-06
* **Initialization**: Frontmatter added in place to every research note (note 42 Phase 0). `generated.by` on the pre-existing notes is `claude-code` without a model version, which their history no longer records, and `generated.at` is the note's own date or, where it gives none, an upper bound from git.
* **Creation**: `TODO.md` split into one-file backlog items (note 42 Phase B), with Sprint Record stubs, scope decisions, standing facts and the pipeline status table.
