# runcard-hidden-lines log

## 2026-10-20
* **Creation**: Sprint folder created.

## 2026-10-20
* **F1 checked** (de96d8c, 16f19d6): the manager re-ran the four new tests from the committed tree (all pass) and confirmed the comment rule against the pinned MadGraph source: `RunCard.read` strips at `#`, then at `!`, then splits at the last `=` (`madgraph/various/banner.py:2900-2902` at b7687064). F1 edited `systematics-variation-weights-not-written` only to unlink an item this PR deletes; the manager accepts that edit as its own.
