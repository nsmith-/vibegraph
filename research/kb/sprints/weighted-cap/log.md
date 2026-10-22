# weighted-cap log

## 2026-10-20
* **Creation**: Sprint folder created.

## 2026-10-20
* **F1 checked** (d6ca65a, b012977): the manager re-ran `weighted_search` (6), `validate_madgraph_diagrams` (57) and `decay_chain_census` (1) and fmt from the committed tree, and read the two load-bearing MadGraph citations at b7687064: `Interaction.get_WEIGHTED_order` divides by `max(n_particles − 2, 1)` (`base_objects.py:893-899`), and `find_optimal_process_orders` returns the process's own orders for a one-particle initial state that is not a decay chain (`diagram_generation.py:1978-1980`).
* **Brief corrected by F1:** the SM cap does not rise (per-vertex WEIGHTED counts per equivalent three-point vertex, so the SM maximum stays 2). F1 also matched MadGraph's start value and its 1→n skip, which the brief did not name. The 1→n skip changes top-level decays without orders (`z > u u~ d d~`: 4 → 17 diagrams), so V1 checks every banked decay.
* **V1 checked** (2e6c2ca, afb48e5, 93dc970): the manager re-ran `weighted_search` (6 passed, 1 ignored) and kb-lint from the committed tree. No decay the suite computes changes under the 1→n rule; the search's bound agrees with MadGraph's recorded bound on all 109 committed cards that generate. The SMEFTsim `e+ e- > z g g` disagreement (MadGraph's gluon-free trial refuses; we return 1 diagram) predates F1: the old loop stopped at the same bound with the same diagram. It is pinned as an ignored test and filed at close-out, not fixed here.
* **HY1 checked** (2f31cce, 2a262f9): the manager re-ran `ufo::weighted` (5), `weighted_search` (7, 1 ignored), `validate_madgraph_diagrams` (57), `diagrams` (20) and fmt from the committed tree.
* **Filed:** `weighted-search-gluon-free-trial-missing` (the SMEFTsim gap, medium), `coupling-orders-fallback-differs-from-madgraph` (low), `diagram-count-test-helper-duplicated` (low); a dated note on `weighted-order-computed-three-ways` naming the fourth site. The item `auto-weighted-search-cap-uses-hierarchy` stays until the user decides whether it closes before the banking run.
- 2026-10-21: M3 ran the V1 banking request. Diagram counts match (2, 6, 4, 17); the Trying lines differ from V1 on rows 2 and 3; the z row needs no import model. The tree is uncommitted on the M3. V2 dispatched to reconcile.
- 2026-10-21: V2 checked (f30f94c): MG diagram_generation.py:2013-2014, 2104-2107 and 2115 confirmed; weighted_search 8 passed. Main merged (2030cde). Corrected banking text is V1-report §5.
- 2026-10-21: M3 banked the four rows (67ee2c2); the Trying lines match §5. validate_madgraph_diagrams passes 61 locally. Item deleted; PR marked ready.
