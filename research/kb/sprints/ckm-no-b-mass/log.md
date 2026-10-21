# ckm-no-b-mass log

## 2026-10-20
* **Creation**: Sprint folder created.

## 2026-10-20
* **F1 checked** (9dc76b4, 9eb1acf): the manager confirmed the committed card's sha256 (ad6e3eaf…1b05) and re-ran `sm_ckm_no_b_mass` (6 passed) and `coupling_oracle` (48 passed) from the committed tree. The `sm_interned_blob` change (the vendored card must be absent from the submodule) needs the submodule and is evidenced by CI's banked layer.
* **Corrections from F1:** the tau width parameter is `WTau`; no coupling references exist at `ckm` or `no_b_mass` (all 25 SM tables are at `restrict_default`), so composition is against this crate's own restrictions and reaches MadGraph only outside the b and CKM sectors.
* **Open for the user:** the vendored card's licence (MadGraph param-card output edited by CMS genproductions) is unchecked.
* **V1 checked** (768a207, ce05f79): the manager re-ran `sm_ckm_no_b_mass` (9), `interned_sm_rows` (4) and `coupling_oracle` (48) from the committed tree; `validation/manifest.toml` is unchanged. Banking request posted on the PR.
* **HY1 checked** (481c979): the manager re-ran `sm_ckm_no_b_mass` (9), `interned_sm_rows` (4) and fmt from the committed tree, filed HY1's report from its final message, fixed the two "nine restrict cards" concept lines, and filed `banked-param-card-join-copied-in-tests` and `gen-reweight-oracle-empty-mg5-root`. The item stays open until the user decides whether it closes before the M3 banking run.
* **User decision** (2026-10-20, relayed by the planner): the vendored CMS card needs no licence of its own; `licensing.md` records it as settled. Close-before-banking is still open.
- 2026-10-21: M3 banking stopped at the count check: MadGraph (unitary gauge) has 67 interactions; this crate has 133 vertices = 67 + 24 ghost + 42 Goldstone. V2 dispatched.
- 2026-10-21: V2 checked (4111f68, 3d89456). import_ufo.py:1241-1255 confirmed; sm_ckm_no_b_mass and smeftsim suites pass. Feynman-gauge count is 109, not 133.
