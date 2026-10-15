# sigma-h1-ickkw0-census log

## 2026-10-15
* **Creation**: Sprint folder created by the sprint manager from the planner's
  hand-off (planner session `session_014wzXyZJfXdGTAVjvtKqbZM`). Draft PR opened
  as the claim.
* **V1 interim** (`0954868`): census of 89 banked banners, the vectorised path,
  and a phased plan (B: read permutations; C0: zero-variance screen; C1:
  statistical runs). Spot-checked by the manager: commit and author, the
  `update_scale_coupling_vec` call site, the 89-run census file. Phase B held
  until a compute slot frees (load 7 on 4 cores).
* **Amendment** (brief error reported by V1): the item's criterion
  "`dynamical_scale_choice = -1` or matching" is wrong in both directions.
  `xqcut > 0` at `ickkw = 0` also clusters, and `-1` with all three scales
  fixed does not. The census uses the corrected criterion. The item text is
  corrected at close-out.
* **V1 final** (`ef002da`): phases B and C0 done. Every banked grouped row
  is checked. Identity-only: the llj family, the four fixed-beam `e+e-`+jet
  rows, and the `pp_to_ll` and `uux` rows. Permuted but immune, with 0
  differing lines in a same-seed screen: the 2 → 2 rows and
  `pp_ttx_lep_dyn`. Permuted and moving, but negligible: `uu_full`/`uu_tt`
  (≤ 3e-5 of σ). The R1 positive control moves +2.1 %. No banking request.
  Machine-confirmed by the manager: the commit, the screen summary, the
  permutation table, and the script's CLI.
* **Relay**: the `uu_tt` bound has gone to `sigma-uux-wbwb-veto`'s V1 brief.
* **Next**: HY1 (hygiene-dev, Sonnet) on `census_permuted_first_call.py`, then
  close-out.
* **HY1 landed** (`b6d546d`, `18ffc52`, `8de7fb4`): the census fails loudly;
  `runs` output is byte-identical. Manager-checked: `runs` re-run and `cmp`
  against V1's output; `perms /nonexistent` exits 1. The agent's request that
  the manager delete `/hy1_runs.tsv` (a stray file its sandbox refused to
  remove) was surfaced to the user, not acted on.
* **Close-out**: item deleted, two Found items filed, concepts updated, record
  written.
