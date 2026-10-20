# ckm-no-b-mass log

## 2026-10-20
* **Creation**: Sprint folder created.

## 2026-10-20
* **F1 checked** (9dc76b4, 9eb1acf): the manager confirmed the committed card's sha256 (ad6e3eaf…1b05) and re-ran `sm_ckm_no_b_mass` (6 passed) and `coupling_oracle` (48 passed) from the committed tree. The `sm_interned_blob` change (the vendored card must be absent from the submodule) needs the submodule and is evidenced by CI's banked layer.
* **Corrections from F1:** the tau width parameter is `WTau`; no coupling references exist at `ckm` or `no_b_mass` (all 25 SM tables are at `restrict_default`), so composition is against this crate's own restrictions and reaches MadGraph only outside the b and CKM sectors.
* **Open for the user:** the vendored card's licence (MadGraph param-card output edited by CMS genproductions) is unchecked.
