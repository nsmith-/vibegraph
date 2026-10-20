# event-norm-sum log

## 2026-10-20
* **Creation**: Sprint folder created.

## 2026-10-20
* **F1 checked** (b6175f5): the manager re-ran the new library tests (5) and `cli_generate` (13) and fmt from the committed tree, and read MadGraph at b7687064: the LO run card allows `sum`, `average`, `unity` (`banner.py:4298`), while `MultiEventFile.unweight` tests for `'unit'` (`lhe_parser.py:1232`), so MadGraph's own `unity` path leaves its strategy unset. F1 implemented the unit-weight branch MadGraph evidently intends; whether to accept `unity` at all is put to the user.
* `sum` is pinned by five banked MadEvent `<init>` blocks (`decay_chain_events_reference.json`: `IDWTUP -3`, `XMAXUP = ΣXSECUP/10000`); there is no banked per-event weight, so a banking request stands.
* **HY1 checked** (e8dff31, 8010bf7): the manager re-ran `lhef::emit` (17), `cli_generate` (13), `cli_reference_docs` (1) and fmt from the committed tree. HY1's refactor was checked byte-identical on the emitted files; its new multi-part test catches a mutation the old tests missed. F1's Found 2 (`include=False` on `<MGRunCard>`) is struck: `include` governs the Fortran `run_card.inc`, and MadGraph's own committed cards write `event_norm`.
* **Open for the user:** whether to accept `event_norm = unity` (MadGraph's own path tests `'unit'`), and whether the item closes before the per-event banking run. The item file stays until then.
