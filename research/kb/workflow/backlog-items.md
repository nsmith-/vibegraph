---
type: Design
title: Backlog items
description: "One file per open item with a required closes_when; an uncommitted generated view; draft-PR claims; done means deleted; dev agents report new work under Found and never edit the backlog."
status: draft
tags: [backlog, workflow, knowledge-bundle, process]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n42-items, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/42-okf-knowledge-bundle-plan.md#L282-L323", title: "Note 42 §7.1: one file per item"}
  - {id: n42-view, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/42-okf-knowledge-bundle-plan.md#L325-L352", title: "Note 42 §7.2: the generated view"}
  - {id: n42-readers, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/42-okf-knowledge-bundle-plan.md#L354-L362", title: "Note 42 §7.3: who reads what"}
  - {id: n42-decisions, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/42-okf-knowledge-bundle-plan.md#L514-L535", title: "Note 42 §9: decisions"}
  - {id: n42-risks, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/42-okf-knowledge-bundle-plan.md#L537-L555", title: "Note 42 §10: risks"}
  - {id: kb-py, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/scripts/kb.py", title: "scripts/kb.py: lint and backlog view"}
  - {id: build-docs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/scripts/build-docs.sh", title: "scripts/build-docs.sh: the backlog docs page"}
---
The work backlog is one file per open item under
`research/kb/backlog/<area>/<slug>.md`, where the area is `validation`,
`feature`, `performance` or `hygiene`. Until the migration's Phase 4 moves
them into the bundle, the item files live under `research/notes/backlog/<area>/`
and `scripts/kb.py` reads them from there. There is no backlog document to edit.
`TODO.md` at the repository root is a stub that points to the items and is not
edited.
This design replaced a single `TODO.md` that every work stream edited, which
was the repository's hottest conflict point, and whose entries accreted
accounts of how problems were solved rather than what was still open.

## An item

```yaml
---
type: Backlog Item
title: acceptance.yml has never passed
description: The release acceptance workflow 404s on release assets; the first run since going public should pass.
area: hygiene            # validation | feature | performance | hygiene
state: needs-user        # open | blocked | needs-user
priority: medium         # low | medium | high
closes_when: An acceptance.yml run against a published release is green.
blocked_by: []           # slugs of other items
opened: 2026-08-10
---
Problem, what would resolve it, links to the concepts that hold the detail.
```

The body says what is wrong and what would resolve it, and links to the topic
concepts that hold the detail. It stays short: past about 40 lines the lint
warns, since detail belongs in a linked concept. `tags`, `generated` and
`sources` are as for any concept.

What the structure enforces, rather than discipline:[^n42-items]

- **Parallel streams stop conflicting.** They touch different files. A
  conflict happens only when two streams edit the same item, and that is a
  real conflict.
- **IDs never collide.** An item's ID is its filename slug, which is never
  renamed, so two branches cannot both create "item 43".
- **Every item says what done means.** `closes_when` is required.
- **Done means deleted.** The PR that finishes an item deletes its file. The
  account of how it was solved goes in the sprint record or session report,
  where the trust tiers mark it as an agent's claim until checked. The backlog
  has no place for a success story, so none accumulate there.

## Who writes items

Dev agents never create, edit or close items. New work they find goes in the
**Found** section of their session report, one entry each. The manager or the
close-out session files it as an item. The rule is binding in `AGENTS.md`
("Planning & Progress") and in the dev-agent definitions.

## Claims are draft PRs

A sprint, or under [one PR per backlog item](../decisions/pr-per-backlog-item.md)
a single item's PR, claims items by opening a **draft PR** whose description
lists them, one `Backlog: <slug>` line each.

- A claim written into the item file would be visible only once merged. A
  draft PR is visible to every stream the moment it opens, and claiming edits
  no item file.
- The claim ends when the PR merges (the closed items' files are gone with it)
  or closes. At close-out, the `Backlog:` lines of items left open are dropped,
  which releases them.
- A PR description that drops or mistypes a `Backlog:` line claims nothing. The
  docs build warns about a slug that names no item.

## The generated view

`pixi run backlog` renders the backlog from the item files and the bundle. The
view is **never committed**: a committed generated file changes with every item
edit, so two squash-merged PRs would conflict on it again, and GitHub cannot
resolve that server-side.[^n42-view] The page has these sections:

| Section | Generated from |
|---|---|
| Current position | `Sprint` concepts with `active: true` |
| Standing decisions | non-deprecated `Design Decision` concepts with a `human:` `verified` stamp; the unreviewed rest are only counted, in one line |
| Open, and the user's call | items with `state: needs-user` |
| Census | `validation/manifest.toml` |
| Standing measurement facts and caveats | `Measurement` and `Caveat` concepts |
| Validation / feature / performance / hygiene backlog | items grouped by `area`, then `priority` |
| Closed sprints | each `Sprint Record`'s `description`, newest `closed` first |

The docs workflow renders the same view as a book page.
`scripts/build-docs.sh` writes `docs/src/backlog.md` (gitignored) before
`mdbook build`, and `SUMMARY.md` lists it. With a `GITHUB_TOKEN`, `--claims`
reads the open PRs' `Backlog:` lines and marks claimed items. Offline, the view
shows items without claims. The docs page is the browsable copy, as fresh as
the last docs deploy. If that proves too inconvenient, the fallback is to commit
the view with a CI freshness check and the rule "on conflict, regenerate".[^n42-risks]

## Who reads what

The full view is for planning: the user, and the manager choosing or scoping
work. **An agent doing the work is pointed at one thing**, a session brief or a
single item, and reads outward from its links. No brief tells a dev agent to
read the whole backlog first.[^n42-readers]

- `pixi run backlog --item <slug>` prints the item, its `blocked_by` chain, and
  the titles and descriptions of the concepts it links.
- `--area`, `--state` and `--priority` narrow the planning view.
- `-o FILE` writes it to a file; `--link-base URL` rewrites links for the docs
  site.

## What the lint checks on items

`pixi run kb-lint` refuses a backlog item if any of these hold:

- `area`, `state` or `priority` is outside its allowed values;
- the `area` does not match the folder it is filed under;
- `title`, `description`, `closes_when` or `opened` is missing;
- the description is longer than 200 characters or more than one line;
- its slug duplicates another item's;
- `blocked_by` is not a list, or one of its entries names no item. This is stricter than OKF requires, by
  choice.

It warns on `state: blocked` without a `blocked_by` entry, and on a body
longer than 40 lines.[^kb-py]

## Related

How a sprint opens with the claim and closes by deleting items is in the
[sprint lifecycle](sprint-lifecycle.md). The folder layout and tooling are in
[the knowledge bundle](knowledge-bundle.md).

[^n42-items]: Note 42 §7.1.
[^n42-view]: Note 42 §7.2.
[^n42-readers]: Note 42 §7.3.
[^n42-risks]: Note 42 §10, "No browsable backlog" and "Unreadable claims".
[^kb-py]: `scripts/kb.py`, `lint()` (`ITEM_BODY_WARN = 40`, `DESCRIPTION_MAX = 200`) and `backlog()`.
