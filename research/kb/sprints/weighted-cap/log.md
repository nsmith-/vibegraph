# weighted-cap log

## 2026-10-20
* **Creation**: Sprint folder created.

## 2026-10-20
* **F1 checked** (d6ca65a, b012977): the manager re-ran `weighted_search` (6), `validate_madgraph_diagrams` (57) and `decay_chain_census` (1) and fmt from the committed tree, and read the two load-bearing MadGraph citations at b7687064: `Interaction.get_WEIGHTED_order` divides by `max(n_particles − 2, 1)` (`base_objects.py:893-899`), and `find_optimal_process_orders` returns the process's own orders for a one-particle initial state that is not a decay chain (`diagram_generation.py:1978-1980`).
* **Brief corrected by F1:** the SM cap does not rise (per-vertex WEIGHTED counts per equivalent three-point vertex, so the SM maximum stays 2). F1 also matched MadGraph's start value and its 1→n skip, which the brief did not name. The 1→n skip changes top-level decays without orders (`z > u u~ d d~`: 4 → 17 diagrams), so V1 checks every banked decay.
