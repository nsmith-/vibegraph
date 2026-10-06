# vibegraph — backlog

The backlog no longer lives in this file. Each open work item is one file under
[`research/notes/backlog/`](research/notes/backlog/), grouped by area
(`validation`, `feature`, `performance`, `hygiene`); the format and the rules
are in [note 42 §7](research/notes/42-okf-knowledge-bundle-plan.md).

- `pixi run backlog` renders the whole backlog as one page: the current position,
  standing decisions, the user's open calls, the census, standing measurement
  facts, the items by area and priority, and the closed sprints. The same page
  is on the [documentation site](https://nsmith-.github.io/vibegraph/backlog.html).
- `pixi run backlog --item <slug>` prints one item, what blocks it and the notes
  it links. Work starts from one item or one session brief, not from the whole
  list.
- The PR that closes an item deletes its file. An open pull request claims an
  item with a `Backlog: <slug>` line in its description.

The pipeline status table is
[`research/notes/pipeline/status.md`](research/notes/pipeline/status.md); the
closed-sprint history is `research/notes/sprints/<name>/closeout.md`.

Do not add entries here.
