# default-particle-names log

## 2026-10-20
* **Creation**: Sprint folder created.

## 2026-10-20
* **F1 checked** (63f50a6, 6029166): the manager re-ran `particle_names` (3), `validate_madgraph_diagrams` (57; F1's report misattributed a "1 passed" line to it) and `proc_grammar_oracle` (1) and fmt from the committed tree, and read `madgraph_interface.py:5841-5842` at b7687064 (`pass_particles_name_in_mg_default()` unless `-modelname`). Design: neither rename nor alias but a renamed lookup table, so MadGraph's refusal of a renamed-away UFO name (`m+` on `loop_sm`) holds and every printed name stays the UFO's.
