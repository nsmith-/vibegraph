---
type: Design
title: Scale, PDF and αs variation weights (MadGraph's systematics module, in-process)
description: "The typed systematics request recorded at integrate (artifact version 15), get_lo_wgt as a library function over a per-event <mgrwt> record, MadGraph's <initrwgt>/<rwgt>/<mgrwt> bytes, PDF members at scale, the default card's 145 weights, the per-event gate on the twelve banked runs, the M3 banking request and the S-F1/S-F2 split."
status: draft
tags: [systematics, lhef, pdf, scales, alpha-s, mlm, cms, artifact, design]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-28}
sources:
  - {id: mg-syst, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/3.7.1/madgraph/various/systematics.py", title: "MadGraph 3.7.1 systematics.py (Systematics, get_lo_wgt, write_banner, get_all_fct, call_systematics)"}
  - {id: mg-lhe, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/3.7.1/madgraph/various/lhe_parser.py#L1578-L1640", title: "MadGraph 3.7.1 lhe_parser.py parse_lo_weight; get_et/ht/sqrts_scale at L2846-L2888; the <wgt> format at L2620"}
  - {id: mg-unwgt, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/3.7.1/Template/LO/SubProcesses/unwgt.f#L767-L834", title: "MadGraph 3.7.1 unwgt.f, the <mgrwt> writer"}
  - {id: mg-rewgt, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/3.7.1/Template/LO/SubProcesses/reweight.f#L1440-L1820", title: "MadGraph 3.7.1 reweight.f, what fills the <mgrwt> fields"}
  - {id: mg-launch, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/3.7.1/madgraph/interface/madevent_interface.py#L2624-L2656", title: "madevent_interface.py: systematics after combine_events, before reweight"}
  - {id: mg-dosyst, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/3.7.1/madgraph/interface/common_run_interface.py#L1776-L1935", title: "common_run_interface.py do_systematics: the accepted options"}
  - {id: cms-runcmsgrid, resource: "https://github.com/cms-sw/genproductions/blob/master/bin/MadGraph5_aMCatNLO/runcmsgrid_LO.sh#L203-L210", title: "genproductions runcmsgrid_LO.sh: the CMS systematics call (fetched 2026-10-28)"}
  - {id: cms-pdflist, resource: "https://github.com/cms-sw/genproductions/blob/master/MetaData/pdflist_5f_run3.dat", title: "genproductions pdflist_5f_run3.dat, the 880-member 5F list (fetched 2026-10-28)"}
  - {id: lhapdf-pyx, resource: "https://gitlab.com/hepcedar/lhapdf/-/blob/lhapdf-6.5.4/wrappers/python/lhapdf.pyx#L868-L883", title: "LHAPDF 6.5.4 Python mkPDF dispatch"}
---
# Scale, PDF and αs variation weights

`generate` writes the weights MadGraph's systematics module
(`madgraph/various/systematics.py`) writes, in-process, in its format, from a
typed request that `integrate` records. The user's decisions Q1–Q9 are in
[the sprint record](../sprints/systematics/sprint.md); this concept is how they
are carried out. Everything here is MadGraph 3.7.1 as checked out in
`research/refs/mg5amcnlo`; line numbers are that tree's.

A MadGraph weight is `XWGTUP × W(variation) / W(nominal)`
(`systematics.py:436`), where `W` is `get_lo_wgt` (`:931-1034`) evaluated on
the event's `<mgrwt>` record and the written event's momenta. The design ports
`W` as one library function over one per-event record, so the same function
writes vibegraph's weights and gates against MadGraph's.

**What has been measured already (S-D1, scratch prototypes, not code).** A
Rust prototype of `get_lo_wgt` over this crate's PDF reader rebuilt every one
of the 12 × 10 000 × 145 = 17 400 000 weights in the twelve banked unmatched
runs; each one, rounded to the eight significant digits MadGraph prints, is
MadGraph's string exactly (worst relative deviation before rounding 4.98e-8).
MadGraph's own `call_systematics` + `write_banner`, run against a
metadata-only stand-in for LHAPDF, writes the banked `<initrwgt>` block byte for
byte (sha256 `eb55efbb…c7d4`, the same block in all twelve files) from both
entry points. §6 has the controls; the S-D1 report has the commands.

## 1. The typed request

### 1.1 What the card says, and how vibegraph knows the card said it

MadGraph has two entry points, and `call_systematics` chooses between them by
`'systematics_arguments' in card.user_set` (`systematics.py:1371`). `user_set`
is every field that **appears on a line of the card as read**: `RunCard.read`
calls `set(name, value, user=True)` for every line (`banner.py:2911`), whatever
the value. The one exception is a list-valued field whose value is `auto`, which
`ConfigFile.__setitem__` moves to `auto_set` and removes from `user_set`
(`banner.py:1106-1115`). MadGraph's default `run_card.dat` writes the
`systematics_arguments` line, so every card MadGraph writes takes the
arguments path; a hand-written or older card without the line takes `sys_*`.

vibegraph records the same bit the way it already records `pdlabel_written`:
`RunCard::parse` keeps the set of canonical names that appeared on a card line
(`written: BTreeSet<&'static str>`, replacing the single `pdlabel_written`
flag), with `auto` on `systematics_arguments` not counted. The banked card's
line parses as MadGraph's list splitter does (`banner.py:1140-1163`, quoted
items kept whole, then outer quotes stripped): `['--mur=0.5,1,2',
'--muf=0.5,1,2', '--pdf=errorset']` → three arguments. An unquoted
`--mur=1,2,0.5 --pdf=a,b` splits at every comma into `--mur=1`, `2`, `0.5`, …,
which `call_systematics` rejects (`unknow argument 2`); vibegraph refuses it at
integrate with that reason.

MadGraph's other flows read the card differently, and Q1 follows
`call_systematics`, so they are recorded rather than mirrored:
`generate_events` (`madevent_interface.py:2638-2645`) passes
`systematics_arguments` whenever it is not `['']`, *including its default when
the line is absent*, and falls back to `--from_card` only for `['']`, which then
fails (`'' = systematics_arguments` is in `user_set`, so `call_systematics`
recurses with the argument `''` and raises). The gridpack flow (`:7117`) always
passes `--from_card`. The two differ only on a card without the line whose
`sys_*` fields are off their defaults; for the defaults both give the same 145
weights (measured, §3.4).

### 1.2 Types

`GenerateSettings` stays MadGraph-neutral in shape; the request is the value
MadGraph's options parse to, with every default filled in, so nothing
downstream knows which entry point produced it.

```rust
pub struct Systematics {                 // GenerateSettings::systematics
    pub enabled: bool,                   // use_syst: writes <mgrwt>
    pub request: Option<VariationRequest>, // None: systematics_program = none
    pub card_fields: Vec<(String, String)>, // verbatim, for <MGRunCard> only
}

pub struct VariationRequest {
    pub source: RequestSource,           // Arguments | SysFields | CommandLine (messages only)
    pub mur: Vec<f64>,                   // --mur, finite and > 0
    pub muf: Vec<f64>,                   // --muf
    pub alps: Vec<f64>,                  // --alps (MLM emission scale, Q7)
    pub dynamical: Vec<DynScale>,        // --dyn
    pub together: Vec<Vec<Axis>>,        // --together blocks, in order; default filled in
    pub pdfs: Vec<PdfEntry>,             // --pdf entries, unresolved
    pub start_id: Option<i64>,           // --start_id
}

pub enum DynScale { Nominal /* -1 */, SumEt /* 1 */, Ht /* 2 */, HalfHt /* 3 */, Sqrts /* 4 */ }
pub enum Axis { Mur, Muf, Alps, Dyn, Pdf }
pub enum PdfEntry {
    ErrorSet,                 // "errorset": the run's own set, every member
    Central,                  // "central": the run's own lhaid
    Set(PdfRef),              // "325300", "NNPDF31_nnlo_as_0118_mc": every member of the set holding it
    Member(MemberRef),        // one member
}
pub enum PdfRef { Id(i64), Name(String) }
pub enum MemberRef {
    Id(i64),                          // "325402@0": the member whose LHAPDF id this is
    Named { set: String, member: u32 }, // "NNPDF31_nnlo_as_0118_mc@7"; "name@0" is member 0
}
```

Values are stored as `f64` and printed with Python's `str(float)` (§3.2), so a
card's `0.50` and `0.5` write the same tag, as in MadGraph.

**Defaults the parser fills.** From arguments (`Systematics.__init__`,
`systematics.py:53-70`): `mur = muf = [0.5, 1, 2]`, `alps = [1]`,
`dynamical = [-1, 1, 2, 3, 4]`, `together = [[Mur, Muf, Dyn]]`,
`pdfs = [ErrorSet]`. A `--together` given replaces the default; repeated ones
accumulate as separate blocks; repeated `--mur` etc. accumulate values
(`call_systematics`, `:1324-1347`). From `sys_*` (`:1374-1410`):
`mur = muf = sys_scalefact.split()`, `alps = sys_alpsfact` (`None` → `[1]`),
`together = [[Mur, Muf, Alps, Dyn]]`, `dynamical = [-1, 1, 2, 3, 4]`, and
`sys_pdf` tokenised as `call_systematics` does: a non-digit token is a name, a
number above 500 an id, a number up to 500 the member count of the token
before it (`name N` → `name@0 … name@N-1`); `&&` separates `name N` groups;
an empty list is `central`.

### 1.3 The refusal table (Q1)

Every refusal names the field or option and says why. "Integrate" means
`RunCard::split`, before anything is integrated; "generate" means before the
first event is drawn.

| input | where | message (substance) |
|---|---|---|
| `systematics_program = syscalc` | integrate | SysCalc is MadGraph's retired C++ program; set `systematics` |
| `systematics_program` other than `none`, `systematics`, `auto` (`auto` reads as `systematics`: vibegraph has no `syscalc_path`) | integrate | unknown program; MadGraph logs and writes no weights |
| `systematics_program` spelled with capitals (`Systematics`) | integrate | MadGraph's membership test is case-insensitive but its dispatch is not (`madevent_interface.py:2632-2638`), so it silently runs nothing |
| `--weight_format`, `--weight_info` (Q1 writes `--weight_info_format`; MadGraph's option is `--weight_info`, `common_run_interface.py:1813-1815`), `--remove_wgts`, `--keep_wgts` | integrate / generate | the custom naming and weight removal are not implemented |
| `--only_beam`, `--ion_scaling`, `--start_event`, `--stop_event`, `--lhapdf_config`, `--result`, `--write_banner`, `--from_card`, any other option, a positional token | integrate / generate | not one of the honoured options (`do_systematics` itself refuses most of them) |
| an argument without `=`, with two `=`, or with an empty value | integrate / generate | MadGraph's parser splits on `=` and indexes the value (`:1324-1331`) |
| `--mur`, `--muf`, `--alps` value not finite or ≤ 0 | integrate / generate | α_s and the densities are undefined there |
| `--dyn` value outside −1, 1–4 | integrate / generate | `write_banner`'s label table has no entry (`:679`, `:761`) |
| `--together` name outside `mur, muf, alps, dyn, pdf` | integrate / generate | |
| `--start_id` not an integer | integrate / generate | |
| `--pdf` entry `id@N`, N ≠ 0, and `sys_pdf` `id N` (which expands to it) | integrate / generate | Q1: name the set (`name@N`); MadGraph adds N to the id |
| `--pdf` entry with whitespace; `sys_pdf` `&&` piece of one token | integrate | MadGraph passes the unstripped name to LHAPDF, which finds no set |
| `sys_pdf` starting with a number ≤ 500 | integrate | `call_systematics` indexes an empty list |
| `use_syst = T` written on fixed-energy beams | integrate | see §2.6; MadGraph's lepton default card sets `F` and its module refuses `pdlabel = none` |
| a `--pdf` list of two or more PDFs whose first is not the run's own lhaid | generate | MadGraph finds the nominal only in first place and otherwise fails (§2.5, quirk A) |
| a set or member not found in the tiers, or consent refused | generate | §4 |
| a member whose `AlphaS_Type` is not `ipol` | generate | §4.3 |
| `ickkw = 1` with a request, until S-F2 lands | integrate | §8 |
| `generate --systematics` when the artifact records `use_syst = F` | generate | MadGraph: "The events have not been generated with use_syst=True" |
| `generate --systematics` on a decay | generate | MadGraph refuses an explicit request on a decay (`common_run_interface.py:1867-1872`) |

`sys_matchscale` and `sys_scalecorrelation` stay `IgnoredBenign` (Q1): only
SysCalc reads them, and SysCalc is refused. A decay with `use_syst = T` from the
card keeps today's behaviour (`for_decay` sets it off): that is what
MadGraph's `--from_card` path does (warn and bypass, `:1868-1870`).

`warn_unwritten_systematics` goes. `use_syst`, `systematics_program`,
`systematics_arguments`, `sys_scalefact`, `sys_alpsfact` and `sys_pdf` become
`Consumed` in `FIELD_CLASSES`; their `B_SYST` reason goes.

### 1.4 The override at generate (Q2)

`vibegraph generate ART --systematics '<MadGraph arguments>'` replaces the
recorded `request` wholesale (the program becomes `systematics`; `use_syst` and
the verbatim card fields are untouched, so `<MGRunCard>` still says what the
card said, as MadGraph's banner does after `runcmsgrid.sh`). The string is
split as a shell splits it (the `shlex` crate; MadGraph's `split_arg`), then
parsed by the same parser as `systematics_arguments`, so every refusal above
applies with "generate" as its stage. CMS's call parses unchanged:
`--start_id=1001 --pdf=<list> --mur=1,2,0.5 --muf=1,2,0.5
--together=muf,mur,dyn --dyn=-1,1,2,3,4 --alps=0.5,1,2`.

### 1.5 Artifact version 15

`GenerateSettings::systematics` changes shape from `{enabled, fields}` to the
struct above: version 15, and `OLDEST_READABLE_VERSION` rises to 15 (Q8). A
version-14 file is refused with the existing `UnsupportedVersion` ("regenerate
it with `vibegraph integrate`"). It cannot be upgraded: it recorded the fields'
values but not whether `systematics_arguments` was on a card line, and the
entry point turns on exactly that. The `FORMAT_VERSION` comment records why.
`RunCard::default().split()` (the no-card artifact) fills the request from the
defaults as the arguments path.

## 2. The weight calculation

### 2.1 The per-event record

One type is filled from vibegraph's generation and parsed from MadGraph's
`<mgrwt>`; it carries exactly what `parse_lo_weight` (`lhe_parser.py:1578-1640`)
returns, plus the event's momenta:

```rust
pub struct LoRecord {
    pub n_qcd: u32,               // <rscale> field 1 + <asrwt> count: the QCD power
    pub mu_r: f64,                // <rscale> field 2, GeV
    pub emission_scales: Vec<f64>,// <asrwt>, GeV, in clustering order
    pub beams: [PdfChain; 2],     // <pdfrwt beam="b">, indexed by the label b
    pub beam_order: [u8; 2],      // the labels in the order written (MadEvent's flip)
    pub tot_fact: f64,            // <totfact>: written, never read by a weight
}
pub struct PdfChain { pub entries: Vec<(i64 /*pdg*/, f64 /*x*/, f64 /*q, GeV*/)> }
pub struct RecordKinematics { pub incoming: Vec<[f64; 4]> /*status -1*/, pub final_state: Vec<[f64; 4]> /*status 1*/ }
```

`<rscale>` writes `n_qcd − n_alpsem` (`unwgt.f:779`); the record holds the total
and writes the difference. A chain's last entry is the one `get_lo_wgt`
reads as the density's flavour, `x` and μF (`pdf_*[-1]`).

**Filled from vibegraph's unmatched generation** (`reweight.f:1450-1458`):
`n_qcd` is the QCD-coupling power of the diagram of the channel that drew the
event (MadEvent's `nqcd(igraphs(1))`, quirk D); `mu_r` the event's μR after
`scalefact`; one chain entry per beam, `(the incoming parton's code, its x, its
μF)`; no emission scales; `tot_fact = f₁(x₁, μF₁) f₂(x₂, μF₂)` on proton beams,
without αs because `asref = 0` when `ickkw = 0` (measured: banked `totfact`
equals `f₁f₂` to 1.6e-7 on 40 000 events of four runs, and differs from
`f₁f₂αs^n` by up to 98% where `n > 0`). The momenta are the event as written:
lab frame, status 1 and −1 only.

**Filled from vibegraph's MLM generation** (S-F2): from the event's
`Rewgt` (`vibegraph-lib/src/coupling/cluster/rewgt.rs`). Emission scales are
`√q2` of each vertex that carries an `alpha_s` ratio, in vertex order
(`reweight.f:1603-1606`); `tot_fact = rewgt × f₁ f₂ × αs(μR)^n_qcd`, the
densities at each chain's *first* entry (`:1803-1818`). The chains are where
the Fortran is least obvious: it seeds entry 1 before the clustering loop
(`:1475-1482`) and appends on every density ratio it takes, recording
`xnow/zcl` and `√q2now` (`:1715-1724`). The banked files show a chain that
starts at the parton the matrix element sees and walks toward the clustered
`2 → 2` core, e.g. on a `g g → e⁺e⁻ c c̄` event of `pp_to_ll_0j2j_mlm`
`<pdfrwt beam="1">  2       21        4 0.11720939E-02 0.79266617E-03 0.24186430E+02 0.12976006E+03</pdfrwt>`:
the incoming gluon at the event's own x (7.6186 GeV / 6500 GeV = 0.0011721)
and the clustering's factorisation scale 24.19 GeV, then the charm line the
`g → c c̄` clustering leaves, at a smaller x and the core's 129.76 GeV. That
last entry is what `get_lo_wgt` reads as "the" density and μF. (1 672 of the
run's events have two entries labelled beam 1, 234 have three.) S-F2 derives the mapping from `Rewgt`'s
`pdf_chain` and pins it by rebuilding every banked MLM `<mgrwt>` (§6.3)
before any weight is computed.

### 2.2 `W`, term by term

`lo_weight(record, kinematics, variation, members, beams) -> Result<f64>`,
with `variation = (Dmur, Dmuf, Dalps, dyn, pdf slot)`, mirrors `get_lo_wgt`:

1. **Scales.** `dyn = −1`: `μR = mu_r`, `μF_b = beams[b].last.q`. Otherwise
   `μR = μF₁ = μF₂ = S(dyn)` from the momenta, and each chain's last `q` is
   replaced by it (`:937-967`).
   * `S(1) = Σ E·pT/√(pT² + pz²)` over status-1 particles with `pT > 0`
     (`get_et_scale`; labelled `sum pt`);
   * `S(2) = Σ √(m² + pT²)` with `m² = E² − |p|²` from the momenta, not the
     mass column (`get_ht_scale`); `S(3) = S(2)/2`;
   * `S(4) = √((p₁ + p₂)²)` of the status −1 particles (`get_sqrts_scale`).
   No `scalefact`, lab frame (quirk F).
2. **μR part.** `αs_pdf(Dmur·μR)^(n_qcd − n_emissions)`, the exponent being
   `<rscale>`'s integer, where `αs_pdf` is the variation member's own αs
   (`pdf.alphasQ`), so a PDF member with a different `AlphaS_MZ` varies the
   coupling with it (Q7). With step 4 the total power is `n_qcd`.
3. **μF/PDF part.** For each proton beam with a non-empty chain:
   `f(b·pdg, x, Dmuf·μF_b)` of the last entry, `f = xfxQ/x`, `pdg ∓21/∓22 →
   21/22`, `pdg 0 → 1` (`get_pdfQ`, `:839-878`; the ion branch is unreachable
   with `nb_proton = 1`, `nb_neutron = 0`).
4. **Emission αs** (MLM). `× αs_pdf(Dalps·q)` for each emission scale (`:1016-1020`).
5. **The PDF chain** (MLM). For `i` in `0 … n−2` on each beam, with
   `scale = min(Dalps·q_i, Dmuf·μF_b)`:
   `× f(pdg_i, x_i, scale) / f(pdg_i, x_{i+1}, scale)` (`:1022-1031`). This is
   the Python's own pairing — one flavour at two x, one scale — not the Fortran
   ratio (one x at two scales); it is mirrored (quirk E).

The ratio is `W(variation) / W(nominal)` with nominal `(1, 1, 1, −1, the run's
own member)`, and the written weight `XWGTUP × ratio` (§3.3). `W(nominal) = 0`
is an error naming the event (MadGraph raises, `:431-434`). A dynamical scale
that is not finite and positive is an error naming the event and choice: on a
2 → 1 process whose written final state is one particle with `pT = 0`,
`S(1) = 0` and MadGraph writes `NaN`; integrate refuses `dyn = 1` there up
front (open choice O3 in the S-D1 report).

### 2.3 One function, two callers

`generate` fills `LoRecord` from its own state, evaluates every variation on
it, writes `<mgrwt>` from it and `<rwgt>` from the ratios. The gate parses
`LoRecord` from MadGraph's `<mgrwt>` and the momenta from MadGraph's particle
lines, evaluates the same function with the same members, and compares with
MadGraph's `<wgt>`. A third caller, the round trip (§6.4), parses vibegraph's
own file.

### 2.4 Where the weights are computed

Per accepted event the source keeps the `LoRecord` (about 100 bytes), not the
ratios: under the buffer strategy a sample holds every event, and 926 ratios
per event would be 7.4 kB each. The ratios are computed when the event is
written, in parallel over the buffered events with the order kept, and the
stochastic-rounding strategy computes them per written event. `W(nominal)`
uses the central member the generation itself read.

### 2.5 MadGraph quirks, and the decision for each

| | quirk | evidence | decision |
|---|---|---|---|
| A | The nominal is found only if it is the **first** `--pdf` PDF. The loop at `:268-275` compares `p.lhapdfID == self.orig_pdf`; on a miss it rebinds `orig_pdf = mkPDF(orig_pdf)`, so from the second iteration it compares an int with a PDF object, and `mkPDF(<PDF>)` returns `None` (`lhapdf.pyx:868-883` falls through). With two or more PDFs and the nominal not first, `orig_pdf.set()` raises. | Reproduced: MadGraph's code on the CMS list over the banked `pp_to_ll` (lhaid 247000) raises `AttributeError: 'NoneType' object has no attribute 'set'`. | refuse at generate, naming the first entry and the run's lhaid; a single-PDF list not holding the nominal is honoured (MadGraph's fresh nominal object) |
| B | On fixed-energy beams the emission loop **assigns** `wgt = alpsrunner(Dalps·q)` (`:1018`), discarding the μR part. | source | moot: fixed-energy systematics are refused (§2.6) |
| C | `get_pdfQ2`'s central-member fallback (`:920-929`) follows a `return` and is dead; `get_pdfQ`'s is commented out (`:874-877`). | source | nothing to mirror: unreachable at LO |
| D | `n_qcd` is the QCD order of the *channel's* diagram, so a mixed-order process records `0` on events from an electroweak channel. | `pp_to_bb_fixed`: 13 of 10 000 events have `<rscale> 0`, the rest 2 | mirror: fill from the drawing channel's diagram (open choice O4) |
| E | The chain pairs flavour `i` with `x_i` and `x_{i+1}` at one scale. | source | mirror |
| F | Dynamical variations use lab-frame status-1 momenta and no `scalefact`; `setscales.f` uses partonic-CM momenta and applies `scalefact`. | §6.2: the two differ on `pp_to_bb*` (massive b, choice 1) and `pp_to_ll_scalefact2` | mirror |
| G | Choice 1 is labelled `sum pt`; it is `Σ E_T`. | source | mirror the text |
| H | Single members after a set are written outside any `<weightgroup>`; closing tags carry `# scale`, `# ALPS`, `# PDFSET to PDFSET`, `# PDFSET to PDF`, `PDF`. | CMS header, §3.4: 31 weights outside a group | mirror byte for byte |
| I | The nominal is re-inserted before the run's `lhaid + 1` (`get_all_fct`, `:822-829`), so a PDF group opens with an unlabelled nominal weight. | banked id 45 equals `XWGTUP` on 120 000 of 120 000 events | mirror |
| J | `pdf == orig_pdf` is object identity: `--pdf=errorset,central` writes a second, *non*-nominal weight for the same member. | source | mirror by slot index, never by lhaid |
| K | A plain id naming member k ≠ 0 expands to its whole set (`mkPDF(id).set()`). | source | mirror |
| L | `get_ht_scale` takes `√(m² + pT²)` with `m²` from the momenta; a negative radicand raises in Python. | source | mirror; a negative radicand is an error naming the event |

### 2.6 Fixed-energy beams

MadGraph's module reads αs there from `models/model_reader.py`'s
`Alphas_Runner(asmz, nloop = 2, MZ, MC → 1.4 if 0, MB → 4.7 if 0)` built from
the **param card** (`systematics.py:286-315`), not the generation's
`alfas_functions.f` running (which fixes `CMASS = 1.42`); `pdf` is forced to
`central`; and it refuses `pdlabel ∈ {none, chff, edff}` outright (`:164-165`).
MadGraph's own default lepton card sets `use_syst = F` and `pdlabel = none`
(`banner.py:4833-4840`; the banked `ee_to_*` cards). No banked run exercises the
path. So: on fixed-energy beams `use_syst` resolves to `F` unless the card
writes it, as MadGraph's process-default card does, and a card that writes
`use_syst = T` there is refused (open choice O5 offers banking one instead).

## 3. Format, byte for byte

### 3.1 `<initrwgt>`

The header is `write_banner` (`:603-738`) over `get_all_fct`'s argument list
(`:798-833`), ported as a state machine over the expanded variations:

* **Expansion.** `args = [nominal] + [a for a in all_args if a != nominal]`,
  `all_args` being, per `together` block, `itertools.product` over the block's
  axes in the order the block names them (first axis outermost), then for
  each axis in `mur, muf, alps, dyn, pdf` order not in any block, its values
  alone; then the nominal re-inserted before `lhaid + 1` (quirk I, `:822-829`). `a !=
  nominal` compares the PDF by slot (quirk J). Weights: `len(args) − 1`.
* **Ids.** `--start_id`, else `max(numeric ids already in <initrwgt>) + 1`,
  else 1 (`get_id`, `:784-796`); vibegraph writes the systematics block before
  any reweighting block (§3.5), so this is 1. Consecutive per weight.
* **Groups.** `Central scale variation` (`combine="envelope"`) opens on the
  first nominal-PDF, `alps = 1` weight with a scale or dyn off its default and
  closes with `</weightgroup> # scale`; `Emission scale variation`
  (`envelope`) holds the nominal-PDF `alps ≠ 1` weights and closes with
  `</weightgroup> # ALPS`; a PDF set group opens on a weight whose PDF is a
  loaded set's member 0, as `<weightgroup name="{set name}"
  combine="{errorType}"> # {SetIndex}: {description}` with the description's
  `=>` → `;`, `>` → `.gt.`, `<` → `.lt.`, and closes with `# PDFSET to PDFSET`,
  `# PDFSET to PDF`, or `</weightgroup> PDF ` (no `#`); a final
  `</weightgroup>` closes whatever is open. `errorType` is LHAPDF's: the
  `.info` `ErrorType` lowercased, `unknown` if absent (`PDFSet.h:143-145`).
* **A weight line.** `<weight id="{id}" {tags}> {info} </weight>`, where tags
  are `MUR="{mur}" `, `MUF="{muf}" `, then `ALPSFACT="{alps}" ` if
  `alps ≠ 1`, `DYN_SCALE="{dyn}" ` if `dyn ≠ −1`, `PDF="{lhapdfID}" `, joined by
  one space (so two spaces between attributes and one before `>`); info is
  `MUR={mur} ` if ≠ 1, `MUF={muf} ` if ≠ 1, `alpsfact={alps} ` if ≠ 1,
  `dyn_scale_choice={sum pt|HT|HT/2|sqrts} ` if ≠ −1, and
  `PDF={SetIndex} MemberID={m}` unless the PDF is the nominal slot.

### 3.2 Number spellings

`mur`, `muf`, `alps` are Python `str(float)`: the shortest round-trip digits,
positional with at least `.0` when `1e-4 ≤ |x| < 1e16`, otherwise
`d.ddde±XX` with at least two exponent digits and no trailing `.0`. Pinned by a
table generated with Python: `0.5`, `1.0`, `2.0`, `0.25`,
`0.3333333333333333`, `1e-05`, `0.0001`, `1000000000000000.0`, `1e+16`,
`1.5e+16`, `1.2345678901234568e+17`, `0.30000000000000004`. `dyn` and ids are
integers. A per-event weight is `<wgt id='{id}'> {w:%+13.7e} </wgt>`
(`lhe_parser.py:2620`; the existing `rwgt_block` already writes it). The
`<mgrwt>` numbers are Fortran `E15.8` (` 0.88513229E+02`), a new formatter.

### 3.3 Per event

```
<event>
 {event line}
 {particle lines}
[<scales …></scales>]            matched events, as today
<mgrwt>                          use_syst = T
<rscale>{n:I3}{μR:E15.8}</rscale>
<asrwt>0</asrwt> | <asrwt>{n:I3}{q:E15.8}…</asrwt>
<pdfrwt beam="{b}">{n:I3}{pdg:I9}…{x:E15.8}…{q:E15.8}…</pdfrwt>   two lines, ME-leg order
<totfact>{E15.8}</totfact>
</mgrwt>
<rwgt>                           systematics ids, then reweighting ids
<wgt id='1'> +1.4889092e+03 </wgt>
…
</rwgt>
</event>
```

Measured on all 120 000 banked events: `</mgrwt>` is followed directly by
`<rwgt>`, ids run 1–145 in order, every `<wgt>` line matches
`<wgt id='\d+'> [+-]\d\.\d{7}e[+-]\d{2} </wgt>`. The `<pdfrwt>` lines are in
the matrix element's leg order and labelled by beam (the banked MLM file shows
`beam="2"` first on a flipped event); the parser maps by label. The empty
form is `<pdfrwt beam="b">0</pdfrwt>` (unreachable here: it is `lpp = ±2`).
Weight value: `XWGTUP × W(var)/W(nom)` with `XWGTUP` as written (MadGraph
multiplies the 8-digit `event.wgt`; the difference is below the printed digit,
§6.4).

### 3.4 The banked header next to what the design writes

MadGraph's `call_systematics` and `write_banner`, run with a metadata-only
LHAPDF stand-in (`mkPDF` dispatching as `lhapdf.pyx` does; `.info` read with a
YAML parser) on the banked `pp_to_ll` banner with its `<initrwgt>` removed,
write:

```
<initrwgt>
<weightgroup name="Central scale variation" combine="envelope">
<weight id="1" MUR="0.5"  MUF="0.5"  PDF="247000" > MUR=0.5 MUF=0.5  </weight>
<weight id="2" MUR="0.5"  MUF="0.5"  DYN_SCALE="1"  PDF="247000" > MUR=0.5 MUF=0.5 dyn_scale_choice=sum pt  </weight>
…
<weight id="44" MUR="2.0"  MUF="2.0"  DYN_SCALE="4"  PDF="247000" > MUR=2.0 MUF=2.0 dyn_scale_choice=sqrts  </weight>
</weightgroup> # scale
<weightgroup name="NNPDF23_lo_as_0130_qed" combine="replicas"> # 247000: LO QCD + LO QED PDF set, alphas(MZ)=0.130. mem=0 ; average on replicas; mem=1-100 ; PDF replicas
<weight id="45" MUR="1.0"  MUF="1.0"  PDF="247000" >  </weight>
<weight id="46" MUR="1.0"  MUF="1.0"  PDF="247001" > PDF=247000 MemberID=1 </weight>
…
<weight id="145" MUR="1.0"  MUF="1.0"  PDF="247100" > PDF=247000 MemberID=100 </weight>
</weightgroup>
</initrwgt>
```

byte-identical to the banked block (sha256 `eb55efbb97a2cabf…`, all twelve
runs), from the arguments `--mur=0.5,1,2 --muf=0.5,1,2 --pdf=errorset`, from
`--from_card` with the line present, and from `--from_card` with the line
deleted (the `sys_*` path). That is the design's output for the default
request: the port is checked against this block, not against prose.

For the CMS arguments on a run whose lhaid is 325300, the same harness writes
926 weights, ids 1001–1926 (sha256 `2cbb8781aabdb04c…`): 44 scale weights
(`--together=muf,mur,dyn`, so `MUF` is the outer loop and the first weight is
`MUR="1.0" MUF="1.0" DYN_SCALE="1"`), 2 in `Emission scale variation`
(`ALPSFACT="0.5"`, `"2.0"`), the re-inserted nominal, and 879 PDF weights, 31
of them single members outside any group. Its `HERAPDF20_NNLO_EIG` line is a
four-line YAML string folded to one line and its CT18 descriptions are YAML
single-quoted: the `.info` must be parsed as YAML (§4.3).

### 3.5 With `--reweight-card`

MadGraph's launch runs `systematics` after `combine_events` and `reweight`
after `store_events` (`madevent_interface.py:2638-2656`): the systematics
groups come first, and the reweight module appends its
`<weightgroup name='mg_reweighting' weight_name_strategy='includeIdInWeightName'>`
after them and its `<wgt>` lines after theirs (`reweight_interface.py:869-983`).
vibegraph follows the launch order. CMS's `runcmsgrid.sh` runs `reweight`
first and `systematics --start_id=1001` after; readers address weights by id,
so the order is cosmetic (open choice O6). Ids cannot collide in MadGraph,
because a numeric `--rwgt_name` is written `rwgt_N` (`:976-978`); vibegraph
writes `--rwgt_name=5` as `5` today (Found F3), and until that is fixed a
collision is refused.

## 4. PDFs at scale

### 4.1 Resolution

At generate, after the central member is verified (run inputs §3), each
`PdfEntry` resolves through the same tiers and consent as the central set
([run inputs](../pipeline/run-inputs-in-the-artifact.md) §5): `ErrorSet` and
`Central` to the run's own set; `Set(Id)` through `pdfsets.index` (largest
`SetIndex ≤ id`, checked against the `.info`); `Set(Name)` and `Member` by
name. Missing sets are collected first and asked for in **one** consent
prompt listing them (the CMS list is 43 sets; open choice O7); a refusal stops
the run before any event (§3a there). Then every requested member is loaded
eagerly, in parallel: each is needed by every event, so loading on first use
saves nothing after the first event and moves a missing-file failure into the
middle of a run. `PdfSet::member` stays lazy per call; the plan calls it for
every slot up front.

### 4.2 What the header records (§3a)

One `<pdf use="systematics" set=… lhaid="{SetIndex}" tier=… [archive_sha256=…]>`
element per set, listing the `.info` and every member `.dat` read with its
sha256, each file once (the read-once loader already hashes as it parses).
For the CMS list that is 43 elements and 923 files, about 100 kB of header.
§3a's example shows one element per member; grouping by set is the change
proposed (open choice O8).

### 4.3 Each member's αs, and the metadata cascade

LHAPDF builds each member's `AlphaS` from the member's own metadata: the `.dat`
header (the YAML block before the first `---`) overrides the set's `.info`. The
CMS list depends on it. Measured on its 43 `.info` files and the 2 038
member headers that preceded each `.info` in its archive (plus member 0 of the
two MSHT20 `as_*range` sets, fetched for the purpose):

* `NNPDF31_nnlo_as_0118_mc_hessian_pdfas` member 102 overrides `AlphaS_MZ:
  0.120`, `AlphaS_Vals` and `ForcePositive: 2`; `AlphaS_MZ`/`AlphaS_Vals`
  overrides also sit in members 101–102 of `NNPDF31_nnlo_hessian_pdfas` and
  `NNPDF31_nlo_hessian_pdfas`, member 102 of `NNPDF40_nnlo_pdfas` and member
  52 of `NNPDF40_nnlo_hessian_pdfas`;
* `ABMP16als118_5_nnlo` (all 30 members), `MSHT20nnlo_as_largerange` and
  `MSHT20nnlo_as_smallrange` give `AlphaS_Type` and the knots *only* in the
  member headers; their `.info` has none.

So `PdfSet::member` must parse the member header and overlay it on the
`.info` for `AlphaS_*` and `ForcePositive` (today it reads neither: Found F1),
and both files must be parsed as YAML (today `parse_raw_map` splits lines at
`:`, which truncates `HERAPDF20_NNLO_EIG`'s four-line `SetDesc`: Found F2). The
40 set-level `AlphaS_Type`s are all `ipol`, and so are the 35 member-level ones
read, the type `GridAlphaS` implements; any member whose resolved type is
another (`analytic`, `ode`) is refused at generate with
`GridAlphaSError::UnsupportedType` naming the set and member, which is how the
member headers this survey did not read get checked. MadGraph's central-member
fallback for a zero density is unreachable at LO (quirk C).

### 4.4 Memory and wall time

Measured with this crate's loader (release build, one core, local disk):
`NNPDF23_lo_as_0130_qed` 101 members in 0.82 s, 3.46 MB resident each;
`NNPDF31_lo_as_0130` 101 in 1.04 s, 3.99 MB each. Evaluating two densities
and one αs costs 0.56–0.59 µs per member per event. For the 880-member CMS
list that extrapolates to **about 3.1–3.5 GB resident** and 7–9 s of loading
(about 2–3 s on four cores), and about **0.55 ms per event** for the PDF
weights. LHAPDF keeps roughly the raw knots (0.5–1 MB per member), so this
loader's precomputed bicubic coefficients cost 3–6 times MadGraph's memory. A
CMS production slot is 2 GB per core; see open choice O2.

## 5. Q3's consequence

`use_syst = T` is the default, so a proton run with no run card, or a card
without the line, now writes `<mgrwt>` and 145 weights, and reads all 101
members of 247000. Measured on the banked `pp_to_ll`: an event grows from
767 bytes to 6 282 (`<rwgt>` 5 272, `<mgrwt>` 243), ×8.2; the 145 weights cost
about 0.09 ms per event and the 101 members 0.8 s and 350 MB once. The CMS
arguments make an event about 35 kB.

What moves (found by grep; S-F1 confirms with the full suite):

* **CLI tests that generate on proton beams without `use_syst = F`**:
  `cli_hard_errors.rs` (four inline cards without the line),
  `cli_generate_proton.rs` and `cli_reweight_proton.rs` where they write their
  own `run_card.dat`, and `onshell_uu_run_card.dat`'s users (it writes
  `True = use_syst`, `systematics`). The validation cards
  (`dy13_*`, `decay_*`, `dy_beams_*`, `onshell_ee`, `onshell_t_bev`) write
  `False` and do not move; the MLM cards write `systematics_program = none`
  and gain `<mgrwt>` only (S-F2).
* **Byte or line comparisons of whole event files**: `cli_generate.rs`,
  `validate_lhef.rs`, `validate_samples_proton.rs`; they compare two vibegraph
  files, so they move only where one side changes its card.
* **`cli_reference_docs.rs`**: the new `--systematics` flag changes the CLI
  help, so `docs/src/cli/reference.md` is regenerated (`scripts/gen-cli-docs.sh`).
* **Scripts**: `scripts/acceptance.sh` (its proton card omits the line) and
  `scripts/demo_pp.tape` (no card) now write 145 weights.
* **Docs**: `docs/src/cli/overview.md:87` (the generate settings) and
  `docs/src/guide/11-unweighting-events.md` (the `<rwgt>` section) gain the
  systematics weights; the run-card defaults list in
  [run-card parser and defaults](../run-card/run-card-parser-and-defaults.md)
  notes `use_syst` resolving to `F` on fixed-energy beams.
* **Pinned outputs**: none; no event file is committed (`git ls-files` holds
  no `.lhe`).
* **Not mirrored**: MadGraph's `create_default_for_process` also appends
  `--alps=0.5,1,2` for a multi-jet merged process (`banner.py:4955-4963`; the
  banked `pp_to_ll_0j2j_mlm` card writes 147 weights from it). vibegraph
  builds no process-default card, so a no-card run never gets it.

## 6. Gates

### 6.1 The per-event gate on the twelve banked runs (S-F1)

`pp_to_bb`, `_bb_fixed`, `_bb_iht`, `_bb_qcd2`, `_jj`, `_jj_ht`, `_ll`,
`_ll_qcd0`, `_ll_scalefact2`, `_llj`, `_llj_dyn`, `_llj_fixed`, all of
`refdata-11`. For each event: parse `<mgrwt>` and the particle lines, evaluate
the 145 variations with `NNPDF23_lo_as_0130_qed`, and compare each with the
written `<wgt>`.

* **Fields.** Every `<wgt>` of every event (1 450 000 per run), the id order,
  and the `<initrwgt>` block byte for byte against the design's header.
* **Tolerance.** MadGraph prints `%+13.7e`, eight significant digits, so a
  value is known to half a unit in the eighth digit (relative 5e-9 to 5e-8).
  vibegraph's value, printed the same way, must equal MadGraph's string or
  differ by one unit in the last digit, and the gate reports the fraction
  equal. Measured by the prototype: equal on 17 400 000 of 17 400 000; worst
  relative deviation before rounding 4.98e-8, inside the half unit. The one
  unit is headroom for density interpolation differing from LHAPDF in the
  last bits.
* **What it cannot see.** Anything about how vibegraph *fills* the record
  (the gate reads MadGraph's); the MLM terms (no banked MLM weights until M3);
  any set other than 247000, member-level αs and the YAML descriptions (one
  set, ipol, no overrides); the dynamical-scale frame and `scalefact` on the
  seven runs with massless final states and `scalefact = 1` (§6.2); `n_qcd`
  from mixed-order channels beyond 13 events.

### 6.2 Negative controls

Each is the prototype with one deliberate change, run on all twelve runs
(max relative deviation per weight class; the gate's tolerance is one unit in
the eighth digit, at most 1e-7 relative):

| control | detected on | blind on |
|---|---|---|
| μR and μF swapped | all 12 (μR-only weights off by 0.32–3.2, combined up to 4e4) | — |
| αs power `n_qcd + 1` | all 12 (μR weights 0.11–0.29) | — |
| `setscales.f`'s dynamical scale (partonic-CM momenta, `scalefact` applied) | `pp_to_bb`, `_bb_qcd2` (1.7e3), `_bb_fixed` (9.8e-3), `_bb_iht` (5.4e-4), `_ll_scalefact2` (0.88) | `_ll`, `_llj`, `_jj`, `_llj_dyn`, `_jj_ht`, `_ll_qcd0`, `_llj_fixed`: massless final states make `E_T` boost-invariant and `√(m² + pT²)` equals `√((E+pz)(E−pz))` |

The third control is therefore a claim about five runs, not twelve; S-F1
asserts each control as detected on the runs listed and records the blind
ones.

### 6.3 MLM (S-F2)

* **The record.** For each event of the dumped MLM runs (`pp_to_llj_mlm`,
  `pp_to_ll_0j2j_mlm`, `pp_to_ttx_0j1j_mlm`, and `pp_to_llj_xqcut_only` for the
  unmatched-with-xqcut path), the `<mgrwt>` rebuilt from the production-path
  `Rewgt` (the replay `validate_mlm_dumps` already runs) equals the banked
  lines field by field at their printed digits, and `<totfact>` at 1e-7.
  This is the finer oracle: it pins §2.1's filling rule before any weight.
* **The weights.** The M3 files (§7): every `<wgt>` of `pp_to_ll_0j2j_mlm` and
  `pp_to_ttx_0j1j_mlm` under the CMS arguments, at §6.1's tolerance. The
  `--alps` weights must differ from the nominal on events with emission
  scales or a chain longer than one, and equal it elsewhere.

### 6.4 The round trip on vibegraph's own events

Generate with the default request, parse the written file with the gate's
parser, re-evaluate, and require every printed weight within one unit of the
last digit. It cannot see a wrong record, only an inconsistent one, so it is
paired with the record's own cross-checks against the same event, each at
the record's printed digits: the first chain entry's x equals the incoming
parton's `E/ebeam`, the last entry's `q` equals the event's factorisation
scale (`SCALUP` is the larger of the two), `αs(μR)` from the record equals
`AQCDUP`, and the nominal-PDF group's first weight equals `XWGTUP`.

### 6.5 CI

The banked layer (which has `validation/pdf` and `refdata-11`) runs §6.1 on
all twelve runs (the prototype took 5.4 s on `pp_to_llj` in a release build,
loading included), the header
byte check, the controls, §6.3's record check and §6.4. The CMS-argument
gate (§6.3's weights and S-V1) needs the 43 CMS sets, about a gigabyte, and
runs in the long tier with them fetched; its header check needs only the 43
`.info` files, which the next bundle carries so that check runs in CI.

## 7. The M3 banking request (Q9)

**The list.** The worktree has the survey's conclusions but not its database
(`validation/cms_gridpacks/gridpacks.sqlite` is not committed on PR #71's
branch and is not on this host), so the frequency of each list could not be
counted. The choice is inferred: 41 of the 65 LO cards have central lhaid
325300 (the survey), and the list genproductions now builds for it, `MetaData/
pdflist_5f_run3.dat` (fetched 2026-10-28), expands to exactly the survey's
"about 880": 43 sets, 880 members, every `NumMembers` matching the list's
count. The manager confirms with the survey database (the S-D1 report has the
query). The argument, as `getMG5_aMC_PDFInputs.py -f systematics -c run3`
builds it (a set with more than one member by id, otherwise `id@0`):

```
--pdf=325300,316200,306000@0,322500@0,322700@0,322900@0,323100@0,323300@0,323500@0,323700@0,323900@0,305800,303200@0,292200@0,331300,331600,332100,332300@0,332500@0,332700@0,332900@0,333100@0,333300@0,333500@0,333700@0,14000,14066@0,14067@0,14069@0,14070@0,14100,14200@0,14300@0,27400,27500@0,27550@0,93300,61200,42780,315000@0,315200@0,262000@0,263000@0
```

CMS also passes `--start_id=1001` (`runcmsgrid_LO.sh:207`), which the brief's
argument list omits; it is included.

**Why the banner is edited.** The banked runs were generated at lhaid 247000,
and MadGraph's module cannot run a list whose first PDF is not the run's own
(quirk A; reproduced). The copy's `<MGRunCard>` therefore says `325300 =
lhaid`. Only `get_lhapdf_id` reads it (`systematics.py:106`), so every weight is
`XWGTUP × W₃₂₅₃₀₀(var) / W₃₂₅₃₀₀(nominal)`, exactly what a CMS sample at 325300
carries, and the gate calls the function with 325300 member 0 as the nominal.
Regenerating the four runs at 325300 would be more faithful and costs MadEvent
time and new σ references (open choice O1).

**Prerequisites on the oracle host.** MadGraph 3.7.1 (the tree banked from);
LHAPDF ≥ 6.5 with its Python module; the 43 sets installed in
`$(lhapdf-config --datadir)` (`lhapdf install <name>` for each name in the
list file). Record `lhapdf-config --version` and, per set, the sha256 of its
`.info` and of the archive.

**Commands**, for `R` in `pp_to_ll pp_to_jj pp_to_ll_0j2j_mlm
pp_to_ttx_0j1j_mlm` (work in a scratch directory; the banked tree is hard
links and is never written):

```bash
MG=/path/to/MG5_aMC_v3_7_1        # the install the refdata runs came from
OUT=/path/to/m3; mkdir -p "$OUT"
PDF=$(cat cms_pdf_arg.txt)        # the list above, one line, no spaces
for R in pp_to_ll pp_to_jj pp_to_ll_0j2j_mlm pp_to_ttx_0j1j_mlm; do
  zcat validation/madgraph/output/$R/Events/run_01/unweighted_events.lhe.gz \
    | python3 -c 'import re,sys; t=sys.stdin.read(); t=re.sub(r"<initrwgt>.*?</initrwgt>\n","",t,flags=re.S); t=re.sub(r"<rwgt>.*?</rwgt>\n","",t,flags=re.S); t=re.sub(r"^(\s*)247000(\s*=\s*lhaid\b)",r"\g<1>325300\2",t,count=1,flags=re.M); sys.stdout.write(t)' \
    | gzip > "$OUT/${R}_in.lhe.gz"
  # smoke: ten events, must print "Will compute 926 weights per event."
  PYTHONPATH="$MG" python3 "$MG/madgraph/various/systematics.py" \
    "$OUT/${R}_in.lhe.gz" "$OUT/${R}_smoke.lhe.gz" --stop_event=10 --write_banner=T \
    --start_id=1001 --pdf="$PDF" --mur=1,2,0.5 --muf=1,2,0.5 \
    --together=muf,mur,dyn --dyn=-1,1,2,3,4 --alps=0.5,1,2 \
    --lhapdf_config=$(which lhapdf-config) > "$OUT/${R}_smoke.log" 2>&1
  # full run, detached
  PYTHONPATH="$MG" python3 "$MG/madgraph/various/systematics.py" \
    "$OUT/${R}_in.lhe.gz" "$OUT/${R}_cms.lhe.gz" \
    --start_id=1001 --pdf="$PDF" --mur=1,2,0.5 --muf=1,2,0.5 \
    --together=muf,mur,dyn --dyn=-1,1,2,3,4 --alps=0.5,1,2 \
    --result="$OUT/${R}_cms_xsec.txt" \
    --lhapdf_config=$(which lhapdf-config) > "$OUT/${R}_cms.log" 2>&1
done
```

`runcmsgrid.sh` reaches the same `call_systematics` through `madevent`'s
`systematics` command; the standalone script skips only the run-directory
bookkeeping. The `pp_to_ll` and `pp_to_jj` inputs are the banked files minus
their 145 default weights, so the output holds only the CMS weights, as a CMS
sample does.

**What to bank**, under `validation/madgraph/output/<R>/Events/run_01_cms/`:
`unweighted_events.lhe.gz` (the output), `xsec.txt`, the log, the exact
command line, the edited input's sha256, `lhapdf-config --version`, and a
`pdfsets.json` of `{set: {info_sha256, archive_sha256, NumMembers}}` for the 43
sets; the 43 `.info` files go in the bundle for the CI header check.

**Pass.** Each log says `Will compute 926 weights per event.`; the header has
926 `<weight>` lines, ids 1001–1926, and equals the predicted block
(`2cbb8781aabdb04c…`, produced by MadGraph's own `write_banner` over the
`.info` metadata; a difference there is a finding about YAML or LHAPDF, not a
failed banking); all 10 000 events carry 926 `<wgt>` lines in id order and no
`nan`/`inf`; weight 1047 (the nominal) equals `XWGTUP`'s string on every event;
on `pp_to_ll` and `pp_to_jj` weights 1045 and 1046 (`alps`) equal it too, and
on the two MLM runs they differ from it on events whose `<asrwt>` is non-zero.
**Fail.** Any exception (an `AttributeError` on `NoneType … set` means the
lhaid edit was not applied), a missing set, a weight count other than 926, a
`nan`.

**Budget** (estimated, not measured: no LHAPDF here). MadGraph evaluates
926 × 10 000 = 9.26 million weights per run in Python, at an assumed 30 µs
each unmatched and up to twice that matched, plus loading 880 members (one to
five minutes, 1–2 GB): roughly 10–20 minutes per run and about an hour for
the four, inside the five hours, with the 43 downloads (about a gigabyte) on
top. The ten-event smoke run measures the per-weight cost before the full
runs are started.

## 8. The implementation split

**S-F1 (unmatched).** The typed request, both parsers and the override, every
refusal of §1.3, version 15, `FIELD_CLASSES`, the `written` set, `use_syst`'s
fixed-energy resolution; the YAML `.info`/member-header cascade with
`ForcePositive` and per-member `GridAlphaS`; systematics PDF resolution,
batched consent, eager loading and the `<vibegraphInputs>` entries; the
Python-float and Fortran-E formatters; `LoRecord` with its writer and parser;
`get_all_fct` and `write_banner` ported; `lo_weight` complete, chain and
emission terms included but gated only through S-F2; the generation path
(unmatched record, ratios at write, `<mgrwt>` then `<rwgt>`, systematics before
reweighting); matched requests refused; Q3's moves. **Gate:** §6.1 on all
twelve runs (fraction of exact strings reported, one-unit tolerance), header
byte-identity with the banked block from the arguments, `--from_card` and
`sys_*` paths, §6.2's controls with their detection table, §6.4, the
Python-float table, the refusal table one test per row, `cargo test`, and
the banked layer (`pixi run --skip-deps validate`, or `bash
validation/validate.sh` with the reference data present where pixi is not
installed).

**S-F2 (MLM).** `LoRecord` from `Rewgt`, `<mgrwt>` on matched events (also
under `systematics_program = none`), the matched refusal lifted. **Gate:**
§6.3's record check on the dumped MLM runs, then §6.3's weights on the M3
`pp_to_ll_0j2j_mlm` and `pp_to_ttx_0j1j_mlm` files, plus the `--alps`
expectations. Depends on M3.

**S-V1** gates the CMS arguments on all four M3 files (including the 880
members' αs overrides and header bytes), measures resident memory and wall
time on the 880 list against §4.4, and closes the item.
