---
type: Session Report
title: "S-V2 report: MadEvent's chain-head slot, the xqcut_only labels, H1 on ll_0j2j"
description: "The ttx flavour codes are MadEvent's, not ours: the record's chain head is an ipdgcl slot filled for the first call's graph and read after rewgt's own clustering, so under H1 it holds another point's flavour (219 ttx, 19 ll_0j2j events); production writes the event's own flavour and is unchanged. The xqcut_only labels follow unwgt.f's flip on P1 (IMIRROR). ll_0j2j's 77 events are H1; permuted events are now gated unless H1 moved the first call. Merge of #82."
status: draft
tags: [systematics, mlm, report, sprint-e]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-30}
---
Worktree `/home/user/wt/S`, branch `sprinte-systematics`, from `03d00872`.
Implements the [S-V2 brief](S-V2.md), plus the manager's mid-session merge of
`origin/main` (#82). Commits: `9381617c` (the merge), `e83defe3` (the tests),
and the commit adding this report. Logs are in the session's scratchpad
(`S-V2-*`).

## 1. The `ttx` flavour code: MadEvent's slot, not our record

**The brief's premise does not hold.** There is no production defect: the
codes MadEvent writes on those events are not the flavours its own matrix
element and `rewgt` used.

### MadEvent's rule

`rewgt` (`Template/LO/SubProcesses/reweight.f`, pinned `b7687064`):

1. `:1439-1440` writes the event's flavours into the slot of the graph the
   *last* clustering named: `ipdgcl(1,igraphs(1),iproc)=idup(1,ipsel,iproc)`,
   the same for leg 2. On entry `igraphs(1)` is the first call's graph.
2. `:1465` runs `rewgt`'s own `setclscales`, which sets `igraphs(1)` to the
   graph it names.
3. `:1479` records each chain's head from the slot of **that** graph:
   `i_pdgpdf(1,j,ivec)=ipdgcl(j,igraphs(1),iproc)`.
4. `:1537-1538` only then resets the slot to `idup(i,ipsel,iproc)`, which
   is what `rewgt`'s density ratios read.
5. `:1811` puts the head's density into `<totfact>`
   (`s_rwfact = rewgt·f(i_pdgpdf(1,i))·…·asref^n`).

Where both calls name one graph, the head is the event's flavour. Under H1
(`super_auto_dsig_group_v4.inc:842`, the first call clusters the unpermuted
`PP`, `rewgt`'s call clusters `P1`), the two graphs can differ. The slot then
holds the flavour that the last point of the same `IPROC` writing that slot
left there, either through `:1439` or `:1538`. That point can be any
evaluated point in the job, written or not, so it cannot be reproduced from
anything a run keeps.

`config_subproc_map.inc` plays no part. The permutation (`PERMS(MAPCONFIG)`)
acts on identical final-state legs only, so the incoming legs' flavours are
the same on `PP` and `P1`. The M3's guess ("the flavour from the written leg
rather than from the permuted channel's leg") is therefore ruled out. A
record head of `3` on a `d`-quark event (event 133) is no leg of that event
at all.

### Evidence, all from data held here

- **The census reproduces the M3's counts.** A record head that differs from
  the written flavour of its leg occurs on 106 (beam 1) and 113 (beam 2)
  `ttx` events: the M3's `<pdfrwt>` misses exactly. Events 48, 61 and 64
  read `written −1, record [−2, 21]`, `2` against `[1, 21]`, and `1` against
  `[2, 21]`: the M3's printed pairs (`S-V2-probe/census.py`).
- **Every such head is a flavour of the event's own `IPROC`.** All 219 heads
  on `ttx` and 31 on `ll_0j2j` are what another `IPSEL` of the same
  subprocess carries on that leg, which is the only flavour such a slot can
  hold. On `ll_0j2j` the swaps are `u↔c` and `d↔s` and never `u↔d`, because
  `u` and `d` couple differently to the Z and sit in different `IPROC`s. On
  `ttx` (`P1_gq_ttxq`) `u↔d` swaps occur, because the matrix element is pure
  QCD and both flavours share one `IPROC`. Every head sits in a permuted
  directory: `P1_gq_ttxq` (219), `P2_qq_llqq` (15) and `P2_qq_llgg` (16).
- **None occurs on the rows without permutations.** `pp_to_llj_mlm` and
  `pp_to_llj_xqcut_only` have 0. At `ickkw = 0` nothing clusters between
  `:1439` and `:1455`, so the slot cannot go stale there.
- **MadEvent's weight used the event's flavour.** On the M3 run the `RWPDF`
  chain (the flavour `rewgt` reads densities for) and the `x`, `q` agree with
  this crate's. Only the record's head and `<totfact>` differ.
- **FI is the same mechanism.** S-F2's `c c̄` event recorded as `u ū` is a
  stale slot.

### What would falsify it

These are all gated in the M3 run below:
- a matched event whose record head is not `RWLEG`'s slot value before the
  reset (field 3);
- a stale head on an event whose two calls named one graph (`SCLOUT` field 4
  on each call);
- a stale head on a non-permuted event;
- a `<totfact>` that differs once the density is read for the slot's flavour.

### The fix

**Production is unchanged.** `generate/source.rs:436-439` writes the written
incoming parton of each beam as the head, which is the flavour the matrix
element and `rewgt` used. Generation clusters both calls on `P1` (H1 is not
reproduced), so the slot problem does not arise.

The tests now follow MadEvent's rule (`e83defe3`):
- **`validate_mlm_dumps`.** The record is compared under MadEvent's slot
  (`RWLEG` field 3), and the falsifiers above are gated:
  `mgrwt: chain heads are the event's flavours where both calls named one graph`
  and `mgrwt: chain heads other than the event's flavours only on permuted events`.
  The stale events are listed with their own flavours and MadEvent's.
- **`validate_mlm_records`.** A gated head census against the bundled
  `leshouche.inc`: the beam order is the mirror reading on every event,
  every non-written head belongs to the event's own `IPROC`, and the counts
  are pinned (0 / 0 / 219 / 31). The replay reads such heads off the record.
- **`validate_systematics`.** On the default-argument MLM runs the stale
  events are pinned (0 / 19 / 219 / 0), and the move of MadGraph's weights
  under the written flavour is reported.

Reproducing the slot is impossible, because it depends on unwritten points.
Refusing would refuse the canonical matched cards. So under
[the defect policy](../../../validation/madgraph-defect-policy.md) this is a
consequence of the registered deviation H1. It is not yet written into H1's
registration, so that is a manager/user step (Found FV).

### The weights

**This crate's production weights are not affected.** They are computed for
the event's own flavour.

**MadGraph's own weights on those events are.** Its module reads the head's
density ratio, so each variation is `f'(stale)/f(stale)` where it should be
`f'(own)/f(own)`. The measurement on the banked `run_01_syst_default` files
(`--mur=0.5,1,2 --muf=0.5,1,2 --pdf=errorset --alps=0.5,1,2`) reports the
relative move of MadGraph's weights if the head were the written flavour,
worst per class:

| run | events | pdf | dyn | muf, mur_muf | alps | mur | mean of each event's largest move |
|---|---|---|---|---|---|---|---|
| `pp_to_ttx_0j1j_mlm` | 219 (2.2%) | 1.32e1 | 4.72e-2 | 2.50e-2 | 4.22e-2 | 3.8e-8 | 3.18e-1 |
| `pp_to_ll_0j2j_mlm` | 19 (0.19%) | 2.12 | 2.77e-1 | 1.21e-1 | 1.89e-2 | 0 | 4.32e-1 |
| `pp_to_llj_mlm`, `pp_to_llj_xqcut_only` | 0 | | | | | | |

On 2.2% of matched `t t̄ + jet` events, MadGraph's PDF-replica weights are off
by up to a factor of 14 (for a replica whose `s`/`c` density differs strongly
from its `u`/`d`), and its scale weights by up to 5%. A sample this crate
writes for the same events carries the other value. The central weight is
unaffected, because the ratio is 1 at the nominal point.

The existing gate, `every weight is MadGraph's string on the banked records`,
still holds: it reads MadGraph's record, stale head included, so it gates the
module's arithmetic, not the head.

### `validate_mlm_records`: before and after

| field (this crate's forests) | S-V1 | S-V2 |
|---|---|---|
| `ttx` beam 1 / beam 2 codes | 9842 / 9838 of 9947 | 9947 / 9947 |
| `ttx` `<totfact>` | 7523 / 9947 | 7627 / 9947 |
| `ttx` beam 2 `q` (FG) | 9022 / 9947 | 9022 / 9947 |
| `ll_0j2j` beam 1 / beam 2 codes | 9895 / 9896 of 9905 | 9904 / 9905 |
| `ll_0j2j` `<totfact>` | 9891 / 9905 | 9900 / 9905 |
| `llj_mlm`, `xqcut_only` | unchanged | unchanged |

The flavour accounts for every code miss except one: the `ll_0j2j` tie of two
chain scales (`0.13921602E+03` twice), which S-F2 already found. It accounts
for 104 of `ttx`'s `<totfact>` misses. **FG's 9% (beam 2 `q`) is untouched**:
it is not the flavour. The M3 run showed production reproducing `q` on every
`ttx` event from MadEvent's own inputs, so the 9% is the replay's
reconstruction (FG, Found below).

## 2. `pp_to_llj_xqcut_only`: the beam labels

**The rule.** `unwgt.f:622-630` sets `flip` when `p(3,1) < 0` on the momenta
it receives, and `:788-791` writes the first chain under beam 2 when `flip`
is set. Those momenta are `DSIG`'s `PP` argument (`auto_dsig_v4.inc:181`,
`CALL UNWGT(PP,…)`), which is `DSIGPROC`'s `P1`. `DSIGPROC` rotates `P1` by π
about x under `IMIRROR = 2` (`super_auto_dsig_group_v4.inc:815-819`).
Leg 1 of the sampled `PP` runs along +z, so `flip ⇔ IMIRROR = 2`.

**Why S-V1's code failed.** It read `flip` off the first call's momenta,
which are `PP`, never mirrored. It therefore wrote `[1, 2]` on every event.
The banked file opens with `beam="1"` on exactly 4978 events (`zcat … | grep`
over the first `<pdfrwt>` per event), which is the M3's 4978/10000.

The codes, `x` and `q` (MadEvent's `SYST` buffer, `reweight.f:1450-1457`:
`i_pdgpdf(1,j) = idup(j)`, `xbk(ib(j))`, `√q2fact(j)`) were right. On every
mirrored event the swapped labels made each beam's comparison read the other
chain, so every `<pdfrwt>` field failed on the same 5022 events.

**The fix.** The labels are now computed on `P1` (the dumped `PP`, rotated
when `SCL`'s `IMIRROR` is 2). Each label disagreement prints a diagnostic
line: `IMIRROR`, the `PP` and `P1` leg-1 `pz`, both label pairs, the `SYST`
codes, `x` and `q`, and MadEvent's record.

**Evidence that it passes.** The dumps are not here, so `IMIRROR` itself
cannot be read; the equivalent rule was checked on the banked records:
- `validate_mlm_records`'s head census, against each process directory's
  `leshouche.inc`: the record writes the first chain under beam 2 exactly
  where the written flavours, beam 1 first, are no `IDUP` row and beam 2
  first are one (`IMIRROR = 2`). That holds on `10000/10000` events of all
  four rows, `xqcut_only` included (5022 mirrored).
- The replay of `xqcut_only` itself passes every field on 10000/10000, the
  labels included.
- Mutation MR2 (the order read without the mirror, the S-V1 reading) fails
  the census with `beam order not the mirror reading on 5022 events` on
  `xqcut_only`.

## 3. `ll_0j2j`'s permuted events: H1

**Where H1 is registered.**
- `research/kb/validation/madgraph-permuted-first-call.md` (its own concept);
- the H1 row of `research/kb/validation/madgraph-defects.md`;
- mg5amcnlo#453.

**The case matches its mechanism.** The concept's measurements on the
refdata-9 dumps were: 160 permuted events, 83 agreeing, and 77 with other
first-call scales, all `@2`. The M3 run gives the same split, 77 of 160 with
other first-call scales. The mechanism is H1's own: the gated field
`permuted P1: first call reproduced from PP` (this crate's clustering of the
**unpermuted** `PP` reproduces MadEvent's first call) passed on the row's
gated part in the M3 run. The written event is the clustered point `P1` on
160/160, so the first call clustered a different point from the one the
matrix element read.

**What the gate does now.** It reports, without gating, every field of a
permuted event whose first call handed on other scales (`μR`, `q2fact`) or
another `q2bck` than this crate's (`info, H1 first call moved: …`). It gates
every other permuted event like any event; before this session, every
permuted event was informational. The chain heads on all events follow the
slot rule of §1. On the M3 run this gates all 3744 permuted `ttx` events
(none moved) and 83 of `ll_0j2j`'s 160.

**What the gate cannot see as a result:**
- on the 77 events, any production error in the first-call path, `q2bck` or
  anything downstream of them (the chain scales, `<rscale>`, `<totfact>`)
  that occurs only where H1 acts;
- H1's main effect on σ, the first-call rejections on points never written
  (+0.25% of `@2`, the concept's measurement), which no per-event dump sees.

**A risk the M3 run settles.** The M3 saw the PDF chain disagree on 2/160
permuted `ll_0j2j` events (first at event 1318), and `<totfact>` on 86 where
77 are H1. If either of those 2 events is among the 83 now gated, the row
fails. The pass conditions below say what to post.

## 4. The M3 re-run

To send once the manager has pushed the branch:

```
# A fresh work area, so fetch-refdata unpacks refdata-12, every process
# directory's .inc files (config_subproc_map.inc, leshouche.inc, configs.inc) included.
git clone https://github.com/nsmith-/vibegraph.git vg-s-v2 && cd vg-s-v2
git checkout <sprinte-systematics tip, carrying e83defe3>
pixi run fetch-refdata && pixi run fetch-pdf
test -z "$(ls validation/madgraph/output/ktdump/dumps 2>/dev/null)"   # nothing there yet
mkdir -p validation/madgraph/output/ktdump/dumps
cp <m3 staging>/dumps/{pp_to_llj_mlm,pp_to_llj_mlm_alps2,pp_to_llj_xqcut_only,pp_to_ttx_0j1j_mlm,pp_to_ll_0j2j_mlm}.jsonl.gz \
   validation/madgraph/output/ktdump/dumps/
python3 -c "import json,hashlib;m=json.load(open('validation/madgraph/mlm_dump_manifest.json'))['runs'];\
print(all(hashlib.sha256(open(r['path'],'rb').read()).hexdigest()==r['sha256'] for r in m.values()))"   # True
pixi run --skip-deps validate-mlm-dumps 2>&1 | tee validate-mlm-dumps.S-V2.log
```

**Pass:** `test result: ok. 3 passed`, which requires all of the following:
- `pp_to_llj_xqcut_only`: every `mgrwt:` field 10000/10000, and no
  `unmatched diagnostics` line;
- `pp_to_ttx_0j1j_mlm`: every gated field full, now including the 3744
  permuted events; `events whose record chain heads are another point's
  flavours: 219, on permuted events 219`;
- `pp_to_ll_0j2j_mlm`: every gated field full; `with the first call's scales
  or q2bck moved by H1 (reported, not gated): 77`; the chain-head line reads
  `19, on permuted events 19`;
- `llj_mlm`, `alps2`: unchanged (full);
- both chain-head checks full on every matched row.

**Fail, and what to post:**
- any `unmatched diagnostics` block;
- for any gated field below full, its line and the first `mgrwt differs`
  blocks;
- the full `ll_0j2j` field table if the 2 PDF-chain events fall in the gated
  set;
- the H1 count, if it is not 77 (that would mean the moved-first-call reading
  is wrong).

## 5. The merge of `origin/main` (#82)

`git merge origin/main` (3fbb1773) as `9381617c`. Four conflicts, each in an
import block or one expression, each resolved by keeping both sides:

| file | conflict | resolution |
|---|---|---|
| `vibegraph-cli/src/generate.rs` | imports: systematics `resolve`/`VariationRequest` against `bind_recorded` | both. `EvaluatedModel` dropped from the import, since `bind_recorded` now builds the model and the old `EvaluatedModel::from_model` call is gone |
| `vibegraph-cli/src/integrate.rs` | imports: `VariationRequest` against `bind_card`, `ParamCard` | both |
| `vibegraph-lib/src/generate.rs` | `header_blocks`: `if matched \|\| settings.systematics.enabled` with `[run card, inputs]` against `if matched` with `[run card, slha, inputs]` | the branch's condition with main's blocks: `<MGRunCard>` when matched or under `use_syst`, `<slha>` on every sample |
| `vibegraph-lib/src/lhef/write.rs` | imports: `SystematicsSet` against the param-card writer's model imports | both |

The refdata-12 pin and the `ee_to_tth_ymt250` inventory merged without
conflict. **Artifact version:** main is at 14 and this branch at 15. Of
#81 (the spine) and this PR, whichever merges second must move to 16.

## Gates

`S-V2-banked.sh`, one cargo invocation per target, release-debug where pixi
uses it, `CARGO_BUILD_JOBS=2`, no debug info, on `e83defe3`:

```
GATES-PLACEHOLDER
```

## Mutations

`S-V2-mut/run.py`: an exact replacement, then the target, then the restore
(`git diff --stat` printed after: only the session's uncommitted test edits,
unchanged).

| mutation | fails |
|---|---|
| MR1 the head's `IPROC` taken as the next table's | `validate_mlm_records`: `pp_to_ttx_0j1j_mlm: 219 stale chain heads (pinned Some(219)), 179 outside the event's subprocess`; `pp_to_ll_0j2j_mlm: … 25 outside` |
| MR2 beam order without the mirror (S-V1's unmatched reading) | `validate_mlm_records`: `beam order not the mirror reading on 5062 / 5022 / 1090 / 4952 events` (llj, xqcut_only, ttx, 0j2j) |
| MR3 the old head rule: only the written flavour admitted | `validate_mlm_records`: `219 stale chain heads …, 219 outside`; `31 …, 31 outside` |
| MS1 heads read against the other beam | `validate_systematics::madgraphs_default_arguments_on_the_banked_mlm_runs`: `chain heads other than the written flavours on 2136 events, pinned Some(219)` (and on every run) |

The dump test's new checks (the slot rule, the H1 classification, the
unmatched labels) have no mutation run here, because there are no dumps.
**Not run.**

## Corrections to the brief

- §1's premise ("a production defect", "fix our record to follow it") does
  not hold. MadEvent's head is a stale slot, which depends on unwritten
  points, and our record writes the flavour that MadEvent's own weight used.
  The M3's guess about the permuted channel's leg is ruled out (§1).
- S-V1's decision rule ("equal first-call scales with a different record
  would mean a production defect") was incomplete. H1 also acts through the
  graph the first call names, which leaves the scales equal.
- "about 9% of `ttx` events disagree (FG)": that 9% is beam 2's `q`, which
  the flavour does not touch. The flavour accounts for the ~1% code misses
  and 104 `<totfact>` misses.

## Found

- **FV. H1's registration misses its record consequence.**
  `madgraph-permuted-first-call.md` says "The event record is unaffected …
  every record field agrees on the permuted events too", which is false for
  `<pdfrwt>`'s head and `<totfact>` on 219 `ttx` and 19 `ll_0j2j` banked
  events, and for MadGraph's systematics weights there (§1's table). Closing
  it takes three changes: the mechanism (`reweight.f:1439-1440` against
  `:1479`) added to the concept and the H1 row of `madgraph-defects.md`, the
  affected rows' manifest notes, and an addendum to mg5amcnlo#453. The
  one-line H1 patch also fixes this one; a standalone fix would move
  `:1439-1440` after the `setclscales` call.
- **FW. FI closes.** The record-against-written census over the banked MLM
  runs is the head census (pinned 0/0/219/31). It needs a line in the
  design's quirk table.
- **FX. FG is the replay's alone.** Beam 2's `q` misses on 925 `ttx` events
  in the event-file replay, but the M3 run reproduces `q` on every `ttx`
  event from MadEvent's own inputs. What the replay rebuilds differently is
  not isolated. A replay that dumps its chosen channel and `jlast` beside
  MadEvent's `RWBEG` on the M3 would settle it.
- **FY. MadGraph's weights on stale-head events are wrong by its own
  intent** (up to ×14 on PDF replicas on 2.2% of matched `t t̄ j`). Any
  comparison of this crate's weights against MadGraph's on samples generated
  independently (not on MadGraph's records) will see it. Such a gate would
  need to exclude those events through the registered deviation.
- **FZ. `validate_mlm_records` still gates nothing but the power on the
  matched rows.** Codes now agree on all compared events except the one
  `ll_0j2j` scale tie. Closing it means gating codes with the tie handled
  (the record's tie rule, which is not identified).
- **Artifact version 16** for whichever of #81 and #83 merges second (§5).
