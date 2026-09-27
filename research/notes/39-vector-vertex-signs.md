# 39 — Vector-vertex convention signs: the gluon source sign, the contact sign, and the gluon–scalar current (2026-09-25)

**Status: LANDED** on `pg-c4v` (validation session beside the `process-grammar`
sprint). Supersedes the "open" bullets of note 38 §4 S1.

## 1. What was wrong

The diagram-level convention signs `compile_single_diagram` folds into
`fermi_sign` for vector vertices were calibrated on processes where they could
not be told apart, and three of them were wrong:

1. **The Yang–Mills source sign on the triple-gluon vertex.** Production gave every
   Yang–Mills VVV vertex off the anchor a −1. That is right for the colourless
   vertices (γWW, ZWW, their SMEFT structures), and wrong for `ggg`. Every process
   that puts a triple-gluon *source* beside a quark-line anchor had the wrong
   relative sign between its gluon-exchange and quark-exchange diagrams: `u u~ > g g`,
   `u g > u g` (but not `g u > g u`), `u u~ > g g g`, `u u~ > t t~ g`, `t t~ > g g`.
   The error is visible without MadGraph: the same-helicity `u u~ > g g` amplitudes,
   which the Ward identity sets to zero, came out as `|M|² ≈ 105` at `√s = 500`.
2. **The four-vector contact sign.** `root_lorentz` gives every all-vector vertex of
   four or more legs a −1 (uniformly over its structures, which stays right). At the
   diagram level that −1 is wrong for a contact that is a *source*, of any colour
   (`g g > g g g`, `e+ e- > w+ w- z`), and wrong for a gluon contact in *either* role
   once item 1 is fixed. A colourless contact keeps it as the anchor
   (`w+ w- > w+ w-`, `a a > w+ w-` pin that).
3. **The gluon-pair–scalar vertex as a scalar source.** With item 1 fixed, the
   effective `ggH` vertex (SMHLOOP, `O_HG`) needs a −1 whenever the anchor-rooted
   tree has it produce the off-shell scalar current; as the sink, or producing a
   gluon current, it takes none. Production had no such sign; the error was hidden
   because in `g g > g g NP<=1` it was exactly compensated by item 1.

Items 1 and 3 compensated each other in every pure-gluon process with Higgs
exchange (`gg_to_gg_cg`), and items 1 and 2 in `g g > g g`, which is why the
enforced `amplitude_oracle` rows never saw them: in a process with only gluon
vertices, flipping "every gluon source, every gluon contact, every scalar source"
is a global sign.

**Defect 2 of note 38 (`u u~ > t t~ g NP<=1`, `vg_c4q`) is item 1.** The
four-quark contacts are innocent: the mismatch is the Standard-Model
`u u~ > g* > t t~` diagrams with the emitted gluon on the s-channel gluon — a
triple-gluon source beside the `u`-line anchor. With the rule below the process
agrees with MadGraph standalone per flow to 3.6e-16.

## 2. The rule that landed

In `helas/eval/root_diagram.rs`, all read at the anchor rooting:

* `yang_mills_vvv_sign`: −1 per **colourless** Yang–Mills VVV vertex off the anchor
  (unchanged for EW; a vertex with a coloured leg is excluded).
* `vector_contact_sign`: cancels the kernel's contact −1 for every all-vector contact
  that is coloured, or that is not the anchor. Net: gluon contacts +1 everywhere;
  colourless contacts −1 as the anchor, +1 as a source.
* `gluon_scalar_current_sign`: −1 per two-vector–one-scalar vertex with a coloured
  leg whose anchor-rooted output is the scalar.

**Why it is a graph invariant.** Each factor is a function of the vertex's particle
content, of whether the vertex is `Diagram::anchor` (a graph invariant: note 38 S1),
and of the particle on the vertex's path to the anchor (`anchor_rooted_outputs`, a
walk over the undirected graph from the anchor). None reads a vertex index or the
live rooting, so every numbering and every re-rooting of one diagram gets the same
factor; `helas::eval::renumbering` and `rooting_soundness` check it (§6).

## 3. The oracle

`validation/madgraph/gen_standalone_jamps.py` (new) generates a process with MadGraph
`output standalone` (the pinned 3.7.1 via `mg5_pinned.sh`), patches `MATRIX` to
copy `AMP()` and `JAMP()` into a COMMON block, links a Fortran driver, and dumps,
per RAMBO point and per NHEL-table helicity, the colour-summed |M|², every JAMP and
(for diagnosis) every AMP. Both sides read the same param card and momenta. The
comparison fits one complex constant `G` over every (point, helicity, flow) entry
and reports the worst `|ours − G·MG|` relative to the largest MadGraph flow.

Standalone output keeps the width of a spacelike propagator; MadEvent zeroes it
(`zerowidth_tchannel`) and this crate follows MadEvent. Through a massive
t-channel line (`g g > t t~ g`, `a a > w+ w-`, `w+ w- > w+ w-`) that is a
0.1–0.5 % per-helicity difference, not a sign; such rows are compared with every
width zero on both sides (`zero_widths`).

**Per-flow agreement, worst deviation (fraction of the largest MadGraph flow).**
"Before" is `65edb40` (S1 merged), "after" is this change.

| process | model | before | after |
| --- | --- | --- | --- |
| `g g > g g g` | sm | 2.36e-1 | 4.2e-16 |
| `g g > g g g g` (15360 entries) | sm | wrong (S1: 30/30 MHV) | 2.0e-15 |
| `u u~ > g g` | sm | 1.84e-1 | 5.2e-14 |
| `u g > u g` | sm | 2.00e-1 | 3.2e-16 |
| `g u > g u` | sm | 3.8e-16 | 3.8e-16 |
| `g g > u u~` | sm | 1.3e-15 | 1.3e-15 |
| `u u~ > g g g` | sm | 1.29e0 | 6.0e-16 |
| `g g > t t~ g` (zero widths) | sm | 8.85e-1 | 2.3e-16 |
| `t t~ > g g NP<=1` | `vg_cHG` | 2.64e-1 | 5.7e-16 |
| `g g > t t~ NP<=1` | `vg_cHG` | 8.8e-16 | 8.8e-16 |
| `g g > g h NP<=1` | `vg_cHG` | 4.44e-1 | 3.1e-16 |
| `u u~ > t t~ g NP<=1` | `vg_c4q` | 8.87e-1 | 3.6e-16 |
| `e+ e- > w+ w-` | sm | 1.2e-15 | 1.2e-15 |
| `e+ e- > w+ w- z` (zero widths) | sm | 1.38e1 | 6.5e-14 |
| `a a > w+ w-` (zero widths) | sm | 7.2e-16 | 7.2e-16 |
| `w+ w- > w+ w-` (zero widths) | sm | 1.4e-13 | 1.4e-13 |
| `w+ w- > w+ w- z` (zero widths) | sm | 9.56e1 | 3.7e-12 |
| `w+ w- > e+ e-` | sm | 6.29e0 | 6.29e0 (open, §5) |

`amplitude_oracle`: all 42 rows unchanged to every printed digit, except that the
fitted `G` of `gg_to_gg` and `gg_to_gg_cg` turns from −i to +i (the global flip
item 1 + 2 + 3 amounts to in a gluon-only process) and `wpwm_to_wpwmz_cw` moves
from |M|² 2.20e3 to 2.79e1 (§5).

The banked `p p > j j` capstone moves towards MadGraph: σ = 6.800862e8 ± 4.432e5 pb
(five seeds, χ²/dof 1.58) against 6.788500e8 ± 1.473e6, pull +0.80, rel +0.18 %,
where the manifest's recorded measurement is 6.811101e8, pull +1.47, rel +0.33 %.
The quark-first quark–gluon subprocesses (`u g > u g` and the other quark-first orderings) were among the
wrong ones; their `g q` mirrors were right, which is why the shift is small.

## 4. How the rule was found, and the candidates it falsified

1. **Per-diagram sign recovery.** For each probe, a least-squares fit
   `Σ_d c_d · (our diagram d's per-flow values) = MG JAMPs` over all entries. Every
   residual came out at 1e-15 with every `c_d = ±c`: the discrepancies are pure
   per-diagram signs. The fit's first answer on `g g > g g g` (flip the ten contact
   diagrams) is S1's candidate.
2. **S1's candidate is falsified.** "Contact −1 only at the anchor" fixes the
   pure-gluon rows but leaves `u u~ > g g`, `u g > u g`, `u u~ > g g g` and
   `t t~ > g g` wrong: the gluon source sign is a separate defect, and the S1 probe
   set had no quark-anchored gluon source to show it.
3. **Solving for the rule.** Each probe's needed flip pattern, and "no relative
   change" for every diagram of every committed `amplitude_oracle` row, give a
   linear system over GF(2) in per-diagram features (vertex class counts, class
   counts off the anchor, class at the anchor, source output particle, fermion-line
   classes, propagator species). No role-free solution exists (no per-vertex or
   per-propagator constant). The solution space fixes the three items of §2 up to
   choices the data leave open, which the controls below then decided.
4. **"σ_V constant per vertex" (note 19's proposal) is falsified.** Triple-gluon
   +1 and γWW/ZWW −1 in both roles, with every contact +1, fixes every probe
   including `w+ w- > e+ e-` but breaks the enforced `gg_to_gg_cg` (the Higgs
   exchange's sign relative to gluon exchange). Adding item 3 and the compensating
   `VVS` signs the GF(2) solution then asks for keeps every committed row, but
   breaks the Standard-Model `w+ w- > w+ w- z` (1.5 against 3.7e-12 without the
   electroweak part), which was not in the system — the control that rejects the
   electroweak extension.
5. **A prediction, not a fit.** Item 3 was forced only by `gg_to_gg_cg`, where it is
   entangled with item 1. `t t~ > g g NP<=1` in `vg_cHG` has the `O_HG` vertex as a
   scalar source beside the top-line anchor, interfering with QCD; nothing had been
   fitted to it. With items 1+2 alone it misses by 7.6e-2; with item 3 it agrees to
   5.7e-16.

## 5. Open

* **`w+ w- > e+ e-`** (also `w- w+ > e- e+`, `w+ w- > u u~`) disagrees with
  MadGraph: the neutrino (down-quark) exchange carries the wrong sign relative to
  the photon and Z s-channel. The crossings `e+ e- > w+ w-` and `u u~ > w+ w-`
  agree, as do `w+ w- > w+ w-` and `w+ w- > w+ w- z`, so it takes a W pair at the
  anchor *and* a final-state fermion line. The QCD analogue (`g g > u u~`) agrees.
  Banked as a known disagreement in `tests/standalone_jamps.rs` so it keeps running.
  No candidate: a constant sign on the γWW/ZWW vertex fixes it and breaks
  `w+ w- > w+ w- z` (§4.4). The fermion-line side (the final–final line with chiral
  `FFV2` vertices) is the next place to look, per diagram against `AMP()`.
* **`wpwm_to_wpwmz_cw`** moves 2.20e3 → 2.79e1 (JAMP2 2.55e1, `|G|−1` 1.7e-2). The
  Standard-Model part of the same process agrees to 3.7e-12, so what remains is
  `O_W`'s — the five-vector and momentum-bearing contact structures. The row stays
  informational.

## 6. Gates, and what each cannot see

* `tests/gluon_parke_taylor.rs` (hermetic, no MadGraph): 5 and 6 gluons, every MHV
  and anti-MHV configuration, per colour flow read off the flow's own colour-line
  tags; `J_σ · ⟨σ1σ2⟩…⟨σnσ1⟩` constant over flows, `|J|·|cyclic|/|⟨ij⟩|⁴` constant
  over configurations and points; configurations with fewer than two gluons of a
  helicity vanish. Deviations are in units of the point's largest flow: a
  configuration suppressed by a soft gluon of the wrong helicity carries the
  point's rounding (1.2e-10 of its own scale at one point, 2.5e-14 of the point's).
  Measured: 5 gluons worst flow 2.5e-14, normalisation 7.2e-14; 6 gluons 8.2e-15,
  6.2e-13; tolerance 1e-10. *Blind to*: the global phase and normalisation constant,
  a per-configuration phase, NMHV configurations (six gluons' 20 are not compared),
  anything outside pure-gluon processes (no quark, so item 1 beside a quark and
  item 3 are invisible to it).
* `tests/standalone_jamps.rs` (hermetic, committed tables under
  `validation/madgraph/standalone/`): per-helicity per-flow JAMPs and per-helicity
  |M|² against MadGraph standalone, one fitted `G` with `|G| = 1`, tolerance 1e-12.
  Rows: `gg_to_ggg` (4.2e-16), `uux_to_ggg` (6.0e-16), `ug_to_ug` (3.2e-16),
  `ee_to_wpwmz` (6.5e-14), `wpwm_to_wpwm` (1.4e-13), `ttx_to_gg_chg` (5.7e-16), and
  `wpwm_to_epem` as a two-way known disagreement. *Blind to*: the phase of `G` (a
  sign common to every diagram of a process); a pair of compensating errors in
  diagrams that reach the same flows with the same weight; kinematics and
  helicities outside three points.
* Mutation, each against the three suites (`standalone_jamps`,
  `gluon_parke_taylor`, `amplitude_oracle`):

  | mutation | fails |
  | --- | --- |
  | drop `vector_contact_sign` | `gg_to_ggg`, `uux_to_ggg`, `ee_to_wpwmz`; both PT tests; oracle `gg_to_gg`, `gg_to_gg_cg` |
  | triple-gluon vertex keeps the source sign | `uux_to_ggg`, `ug_to_ug`, `ttx_to_gg_chg`; six-gluon PT; oracle `gg_to_gg`, `gg_to_gg_cg` |
  | drop `gluon_scalar_current_sign` | `ttx_to_gg_chg`; oracle `gg_to_gg_cg` |
  | colourless contact treated like a gluon one | `wpwm_to_wpwm` |
  | S1's candidate (contact −1 at the anchor only, nothing else) | `uux_to_ggg`, `ug_to_ug`, `ttx_to_gg_chg` |

  The colourless contact at the anchor is pinned only by the new `wpwm_to_wpwm`
  table; no enforced `amplitude_oracle` row reaches it.
