# runcard-hidden-lines log

## 2026-10-20
* **Creation**: Sprint folder created.

## 2026-10-20
* **F1 checked** (de96d8c, 16f19d6): the manager re-ran the four new tests from the committed tree (all pass) and confirmed the comment rule against the pinned MadGraph source: `RunCard.read` strips at `#`, then at `!`, then splits at the last `=` (`madgraph/various/banner.py:2900-2902` at b7687064). F1 edited `systematics-variation-weights-not-written` only to unlink an item this PR deletes; the manager accepts that edit as its own.
* **HY1 checked** (ba65b18, 6def5c4). The manager reverted HY1's `budget.rs` test edit: the sprint keeps out of `budget.rs` while the sampler seam design is open (the HY1 brief wrongly invited it). Clippy (`-p vibegraph-lib --all-targets`), fmt and the progress/budget/systematics/hidden tests (11) pass after the revert; the migration is filed as `budget-test-keeps-private-capture-subscriber`. HY1's Found 1 (the `banner.py` rule) was already confirmed at F1 check.
* **Close-out** (2026-10-20).
