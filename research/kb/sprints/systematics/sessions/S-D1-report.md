---
type: Session Report
title: "S-D1 report: the systematics weights designed"
description: "The design concept events/systematics-weights.md; MadGraph's own write_banner reproduces the banked header byte for byte and a get_lo_wgt prototype every one of 17.4M banked weights at MadGraph's printed digits; the controls' blind spots; the nominal-first crash that shapes M3; open choices O1-O12, brief corrections and Found."
status: draft
tags: [systematics, design, report, sprint-e]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-28}
---
Worktree `/home/user/wt/S`, branch `sprinte-systematics`, from `4d74ef8f`.
Design only; no production code. The deliverable is
[scale, PDF and αs variation weights](../../../events/systematics-weights.md).

## What was done

The concept settles the brief's eight points: the typed request and both
MadGraph entry points (§1, including how vibegraph knows a card *wrote*
`systematics_arguments`: MadGraph's `user_set` is "the line is on the card",
recorded the way `pdlabel_written` already is), artifact version 15 with the
floor raised and why version 14 cannot be upgraded (§1.5), `get_lo_wgt` term by
term over one per-event record (§2), the bytes of `<initrwgt>`, `<mgrwt>` and
`<rwgt>` (§3), PDFs at scale (§4), Q3's consequence (§5), the gates (§6), the
M3 banking request (§7) and the S-F1/S-F2 split (§8). Every count and format
claim in it comes from a script run in this session; the checks are below.

The scratch work lives in
`/tmp/claude-0/-home-user-vibegraph/80d5fa95-18c8-5bff-ac38-2a2fde4863d4/scratchpad/S-D1/`
and is not committed; the two stub files that predict MadGraph's header are
reproduced at the end of this report, because S-F1 and the M3 check use them.

## Machine checks (commands and outputs)

1. **Which banked runs carry what** (`census.py` over
   `validation/madgraph/output/*/Events/run_01/unweighted_events.lhe.gz`,
   printing runs with `<mgrwt>` or weights): 12 runs with `mgrwt=10000
   initrwgt_weights=145 rwgt_in_ev1=145 groups=['Central scale variation',
   'NNPDF23_lo_as_0130_qed']` (`pp_to_bb`, `_bb_fixed`, `_bb_iht`, `_bb_qcd2`,
   `_jj`, `_jj_ht`, `_ll`, `_ll_qcd0`, `_ll_scalefact2`, `_llj`, `_llj_dyn`,
   `_llj_fixed`); `pp_to_ll_0j2j_mlm`, `pp_to_llj_mlm`, `pp_to_ttx_0j1j_mlm`
   **and `pp_to_llj_xqcut_only`** with `mgrwt=10000 initrwgt_weights=0`
   (`systematics_program = none`); `pp_to_llj_mlm_alps2` has none (`use_syst
   = F`).
2. **Header identity and per-event layout** (`census2.py`), per run:
   `initrwgt_sha=eb55efbb97a2cabf events=10000 wgts_per_event={145: 10000}
   ids_1_145_in_order=True mgrwt_then_rwgt=True wgt45==XWGTUP_text=10000`;
   `distinct initrwgt blocks: {'eb55efbb97a2cabf': 12}`; `n_qcd` is `{2:
   10000}` on the bb/jj runs except `pp_to_bb_fixed {2: 9987, 0: 13}`, `{0:
   10000}` on ll, `{1: 10000}` on llj; `n_pdfrw={(1, 1): 10000}`,
   `asrwt={'0': 10000}` everywhere.
3. **Every `<wgt>` line's format** (all 17 400 000 lines against
   `<wgt id='\d+'> [+-]\d\.\d{7}e[+-]\d{2} </wgt>`): `total 17400000
   nonmatching 0`.
4. **MadGraph's own header code reproduces the banked block.** With
   `MG_ROOT=research/refs/mg5amcnlo`, the stub below and the banked `pp_to_ll`
   banner minus `<initrwgt>`:
   `python3 mgstub/run_banner.py pp_to_ll_noinit.lhe hdr_default.txt
   --mur=0.5,1,2 --muf=0.5,1,2 --pdf=errorset` → `weights: 145 … ['# Will
   compute 145 weights per event.']`; `cmp hdr_default.txt hdr_banked.txt` →
   identical; both `eb55efbb97a2cabfb6dd59595d3534770ed4175971d0cc217de18320d893c7d4`.
   `--from_card=internal` (line present) → identical; the same with the
   `systematics_arguments` line deleted (the `sys_*` path) → identical.
5. **The nominal-first crash** (quirk A): the CMS arguments on the same
   banner (lhaid 247000) → `AttributeError: 'NoneType' object has no
   attribute 'set'`. With the banner's lhaid edited to 325300 → `weights: 926
   ids: 1001 … ['# Will compute 926 weights per event.']`, 926 `<weight` lines,
   31 outside any group, sha256 `2cbb8781aabdb04cfa353e2613dca15e9386e9341f4e67af621a44c4f32c715a`.
6. **The banked MLM cards' defaults**: `--from_card=internal` on
   `pp_to_ll_0j2j_mlm` and `pp_to_ttx_0j1j_mlm` → `147 weights` (their
   `systematics_arguments` carry `--alps=0.5,1,2`); `pp_to_llj_mlm` → 145.
7. **The CMS list** (`genproductions/MetaData/pdflist_5f_run3.dat`, fetched
   2026-10-28, sha256 `c79364b0…b844`; `runcmsgrid_LO.sh` sha256
   `9f24f1ba…2f3c`), expanded as `getListOfLHAPDFIdsForSystematics` does:
   `entries 43 members 880`. The other lists: `pdflist_5f_2017` 39 sets,
   1 275 members; `pdflist_4f_run3` 16 sets, 557; `pdflist_4f_2017` 28 sets,
   973; `pdflist_5f_2016` 3 sets, 211.
8. **The CMS sets' metadata** (`getinfos.py` streamed each LHAPDF archive to
   its `.info`, keeping the member headers before it; `infocensus.py`):
   every `SetIndex` equals the list's id, every multi-member `NumMembers`
   equals the list's count, `total members used by --pdf: 880`, set-level
   `AlphaS_Type` `{'ipol': 40, None: 3}`; of 2 038 member headers read, 35 set
   `AlphaS_Type`, all `ipol`; member 0 of `MSHT20nnlo_as_smallrange` and
   `_largerange` fetched separately: `AlphaS_Type: ipol`.
9. **`get_lo_wgt` prototype** (Rust, scratch crate over `vibegraph-lib`'s
   `PdfSet`/`GridAlphaS`, 12 runs): `printed-digit differences (units of the
   8th significant digit): {0: 1450000}` on every run; worst relative deviation
   per class 4.98e-8 (`pp_to_jj mur_muf`); 5.4 s for `pp_to_llj` including
   loading.
10. **Controls** (same prototype, `swap`, `wrongpow`, `fortran` modes; worst
    relative deviation per class): swap detects on 12/12 (μR class
    0.316–3.21); wrong power on 12/12 (0.114–0.291); the `setscales.f` scale
    (partonic-CM `E_T`, `scalefact`) detects on `pp_to_bb` and `_bb_qcd2`
    (`dyn_only` 1.668e3), `_bb_fixed` (9.777e-3), `_bb_iht` (5.438e-4),
    `_ll_scalefact2` (8.758e-1), and is at the gate's noise (≤ 5.8e-8) on the
    other seven.
11. **`totfact`** on unmatched events: equals `f₁f₂` to 1.317e-7 (`ll`),
    1.431e-7 (`llj`), 1.485e-7 (`bb`), 1.640e-7 (`jj`); against `f₁f₂αs^n` it
    is off by 0.91–0.99 where `n > 0`.
12. **Loader memory and speed** (`memprobe`, release): `members=101
    load_s=0.823 per_member_kb=3460 … per_event_per_member_us=0.594`
    (NNPDF23); `load_s=1.042 per_member_kb=3993 … 0.557` (NNPDF31_lo_as_0130).
13. **Event size** (banked `pp_to_ll`): `mean_event_bytes=6282
    rwgt_bytes=5272 mgrwt_bytes=243 rest=767`.
14. **MadGraph's standalone entry imports** with `PYTHONPATH=$MG` (it stops
    at `Can not run systematics since can not link python to lhapdf`, past
    every import), so §7's commands run as written on a host with LHAPDF.
15. `python3 scripts/kb.py index` then `python3 scripts/kb.py lint`:
    `1001 concepts, 333 backlog items; 0 errors, 4 warnings` (the four are the
    pre-existing `state: blocked` warnings).

## Open choices, each with a recommendation

- **O1. How M3 gets a nominal MadGraph can run.** (a) Edit `lhaid` to 325300
  in a copy of each banked file's `<MGRunCard>`; (b) regenerate the four runs
  at 325300. *Recommend (a)*: only `get_lhapdf_id` reads the field, the
  weights are then exactly a 325300 sample's ratios, and it costs no
  MadEvent time or new σ references.
- **O2. Memory for the 880 members** (3.1–3.5 GB here vs about 1 GB in
  LHAPDF; CMS slots are 2 GB per core). *Recommend*: S-F1 loads eagerly as
  designed, S-V1 measures peak RSS on the 880 list, and a compact member
  representation is filed if it exceeds 2 GB; evaluating member by member
  over a buffered sample is the fallback that does not need it.
- **O3. A 2 → 1 process with `dyn = 1`** (`E_T = 0`, MadGraph writes `NaN`).
  *Recommend* refusing at integrate when the written final state is a single
  particle and the request holds `dyn = 1`, naming `--dyn=-1,2,3,4`; a
  runtime error for any other non-positive scale.
- **O4. `n_qcd`.** MadEvent records the drawing channel's diagram order (13
  events of `pp_to_bb_fixed` record 0). *Recommend* mirroring when
  vibegraph's channels map to diagrams one to one; S-F1 checks that first,
  and if channels are merged it uses the process's QCD order and documents
  the deviation.
- **O5. Fixed-energy beams.** *Recommend* refusing a written `use_syst = T` on
  `lpp = 0` and resolving the default to `F`, as MadGraph's lepton default
  card does; the alternative (bank an `e⁺e⁻` run with a `pdlabel` MadGraph's
  module accepts, port `Alphas_Runner`) has no user asking for it.
- **O6. Order with `--reweight-card`.** *Recommend* MadGraph's launch order
  (systematics, then reweighting); CMS's gridpack order is the reverse with
  `--start_id=1001`, and readers address weights by id.
- **O7. Consent for many sets.** *Recommend* one prompt listing every missing
  set (43 for the CMS list), `--yes` covering it.
- **O8. `<vibegraphInputs>` for systematics sets.** *Recommend* one element
  per set listing each file, instead of §3a's one element per member (880
  elements, the `.info` repeated each time).
- **O9. Which CMS list.** Inferred as the run-3 5F list (880). *Recommend* the
  manager confirms against the survey database before M3:
  ```python
  import sqlite3, zlib, re, collections
  db = sqlite3.connect('validation/cms_gridpacks/gridpacks.sqlite')
  c = collections.Counter()
  for enc, content, order in db.execute("""SELECT b.encoding, b.content, g.order_
          FROM gridpack_file f JOIN blob b USING (sha256) JOIN gridpack g USING (gridpack_id)
          WHERE f.role = 'runcmsgrid'"""):
      t = (zlib.decompress(content) if enc == 'zlib' else content).decode(errors='replace')
      m = re.search(r'pdfsets="([^"]*)"', t)
      c[(order, m.group(1) if m else None)] += 1
  for (order, lst), n in c.most_common():
      print(n, order, len(lst.split(',')) if lst else None, (lst or '')[:60])
  ```
- **O10. `sys_pdf` `id N`.** It expands to `id@k`, which Q1 refuses in
  `--pdf`. *Recommend* refusing both, so one spelling is not honoured through
  the back door.
- **O11. `systematics_program = Systematics`** (MadGraph silently runs
  nothing). *Recommend* refusing; mirroring would drop a request without a
  word.
- **O12. Where PDF names are checked.** *Recommend* generate only (the sets
  are generate inputs, §3a); checking names against `pdfsets.index` at
  integrate as well would catch a typo earlier at the cost of an index fetch.

## The M3 banking request

The concept's §7 is the request, verbatim: the `--pdf` string, the CMS
arguments with `--start_id=1001`, the input preparation (strip the default
weights, edit `lhaid`), the ten-event smoke and full commands for
`pp_to_ll`, `pp_to_jj`, `pp_to_ll_0j2j_mlm`, `pp_to_ttx_0j1j_mlm`, what to
bank under `run_01_cms/`, the pass conditions (926 weights, ids 1001–1926,
header sha256 `2cbb8781…`, nominal and `alps` weights equal to `XWGTUP` where
they must be) and the estimated hour of work.

## Corrections to the brief

1. **CMS passes `--start_id=1001`** (`runcmsgrid_LO.sh:207`); the brief's
   argument list omits it.
2. **The M3 commands cannot run as the brief frames them**: MadGraph's module
   fails on any `--pdf` list of two or more PDFs whose first is not the run's
   own lhaid (reproduced), and the banked runs are at 247000. §7 edits the
   copy's `lhaid`.
3. **"Fixed-energy beams use the run card's αs running" is not what MadGraph
   does.** Its module builds `Alphas_Runner` from the *param card*
   (`asmz`, `nloop = 2`, `MZ`, `MC` → 1.4, `MB` → 4.7), not the generation's
   `alfas_functions.f` (`CMASS = 1.42`), and it refuses `pdlabel = none`,
   which MadGraph's own lepton card sets (with `use_syst = F`).
4. **Q1's `--weight_info_format` is spelled `--weight_info`** in MadGraph
   (`do_systematics`' prefix list and the `Systematics` keyword). The refusal
   covers the real option.
5. **The "`setscales.f` dynamical scale" control is blind on seven of the
   twelve runs**: for massless final states and `scalefact = 1` the two
   definitions agree. It detects only on the four `pp_to_bb*` runs and
   `pp_to_ll_scalefact2`.
6. **PR #71's branch holds the survey's scripts, not its database**, so the
   most common list could not be counted (O9).
7. The sprint record's list of MLM runs with `<mgrwt>` omits
   `pp_to_llj_xqcut_only`, which also carries it (check 1); it is a fourth
   record oracle for S-F2.

## Found

- **F1. `PdfSet::member` ignores the member's own metadata.** LHAPDF overlays
  a member `.dat` header on the `.info`; `NNPDF31_nnlo_as_0118_mc_hessian_pdfas`
  member 102 overrides `ForcePositive: 2`, `AlphaS_MZ: 0.120` and
  `AlphaS_Vals`, and `ABMP16als118_5_nnlo` defines its αs only per member.
  vibegraph applies the set's `ForcePositive` (`pdf/mod.rs`, `member`) and the
  set's αs, so even a central run at such an lhaid (vibegraph accepts non-zero
  members) reads the wrong clamp and coupling. Closes when the member header is
  parsed and overlaid for `AlphaS_*` and `ForcePositive`, pinned on a synthetic
  set whose member overrides both. S-F1 needs it.
- **F2. `.info` files are not parsed as YAML.** `parse_raw_map`
  (`pdf/grid.rs:142-155`) splits lines at `:`; `HERAPDF20_NNLO_EIG`'s `SetDesc`
  is a four-line double-quoted scalar and the CT18 descriptions are
  single-quoted, so the description (which MadGraph writes into the header) is
  truncated or keeps its quotes. Closes with a YAML parser crate and a test on
  both shapes.
- **F3. A numeric `--rwgt_name` is written as given.** MadGraph writes
  `rwgt_N` for an all-digit name (`reweight_interface.py:976-978`); vibegraph
  writes `N` (`reweight/mod.rs:195` prefixes only unnamed launches). The ids
  differ from MadGraph's and can collide with systematics ids. Closes by
  prefixing as MadGraph does.
- **F4. A loaded PDF member is 3.5–4 MB resident**, against roughly 0.56 MB of
  knots (NNPDF23, 14 flavours) and LHAPDF's footprint near that: the
  precomputed bicubic coefficients. Harmless for one member; 3 GB for the CMS
  list. Closes with a measured compact representation if O2's measurement
  says so.
- **F5. The survey database is not reproducible from the repository.**
  `gridpacks.sqlite` is neither committed nor banked, so any count over the 88
  gridpacks (here, the PDF lists) cannot be redone. Closes by banking it or a
  derived table of each gridpack's `runcmsgrid.sh` PDF list and arguments.

## Stub sources (for S-F1's header port and the M3 check)

`fake_lhapdf.py`, a metadata-only LHAPDF module (needs PyYAML;
`FAKE_LHAPDF_INFO_DIRS` lists directories holding `.info` files):

```python
import os, glob, yaml
INFO_DIRS = os.environ['FAKE_LHAPDF_INFO_DIRS'].split(':')
def _kv(path):
    return {k: (str(v) if not isinstance(v, str) else v)
            for k, v in yaml.safe_load(open(path, errors='replace')).items()}
_SETS = {}
for d in INFO_DIRS:
    for p in glob.glob(os.path.join(d, '**', '*.info'), recursive=True):
        _SETS.setdefault(os.path.basename(p)[:-5], _kv(p))
def setVerbosity(v): pass
class PDFSet:
    def __init__(self, name):
        self.name = name; i = _SETS[name]
        self.lhapdfID = int(i['SetIndex']); self.size = int(i['NumMembers'])
        self.description = i.get('SetDesc', '')
        self.errorType = i.get('ErrorType', 'UNKNOWN').lower()
    def mkPDFs(self): return [PDF(self, m) for m in range(self.size)]
class PDF:
    def __init__(self, s, m):
        if not 0 <= m < s.size: raise RuntimeError(f'member {m} out of range for {s.name}')
        self._set = s; self.memberID = m; self.lhapdfID = s.lhapdfID + m
    def set(self): return self._set
def _by_id(lhaid):
    for n, i in _SETS.items():
        si = int(i['SetIndex'])
        if si <= lhaid < si + int(i['NumMembers']):
            s = PDFSet(n); return PDF(s, lhaid - s.lhapdfID)
    raise RuntimeError(f'no set for lhaid {lhaid}')
def mkPDF(*args):  # dispatches as lhapdf.pyx 6.5.4: one non-int, non-str argument returns None
    if len(args) == 1:
        if type(args[0]) == int: return _by_id(args[0])
        if type(args[0]) == str:
            name, _, mem = args[0].partition('/')
            return PDF(PDFSet(name), int(mem) if mem else 0)
    elif len(args) == 2 and type(args[0]) == str and type(args[1]) == int:
        return PDF(PDFSet(args[0]), args[1])
    else:
        raise Exception("Unknown call signature")
```

`run_banner.py` (`MG_ROOT` is the MadGraph tree; arguments are the input LHE,
the output file, then MadGraph's systematics options):

```python
import sys, os, io, re
sys.path.insert(0, os.environ['MG_ROOT'])
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import fake_lhapdf
from madgraph.various import misc
misc.import_python_lhapdf = lambda *a, **k: fake_lhapdf
from madgraph.various import systematics
lhe, out, args, logs = sys.argv[1], sys.argv[2], sys.argv[3:], []
obj = systematics.call_systematics([lhe, None] + args, running=False, log=logs.append)
buf = io.StringIO(); obj.output = buf; obj.write_banner(buf)
open(out, 'w').write(re.search(r'<initrwgt>.*?</initrwgt>\n', buf.getvalue(), re.S).group(0))
print('weights:', len(obj.args) - 1, [l for l in logs if 'Will compute' in str(l)])
```
