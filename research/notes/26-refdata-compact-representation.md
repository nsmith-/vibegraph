# 26 — A compact banked-event representation: measurement and verdict

Measured 2026-07-31 on the 25 banked MadGraph runs in
`validation/madgraph/output/`.

## The question

The banked layer's largest dependency is the reference bundle
(`vibegraph-refdata-1.tar.zst`, 90 597 923 bytes), which is fetched rather than
committed because it is too large for git. Almost all of it is 25 gzipped Les
Houches event files, 93 896 879 bytes of `.lhe.gz` in total.

A Les Houches file is a text record of everything MadGraph knew about an event,
and the gates read a small part of it. If the fields they actually read fit in a
few megabytes, the event data could be committed and the bundle would shrink to
something trivial — collapsing the fetch and making the banked layer honest on a
bare clone.

**Verdict: no.** The projection is exact and it does shrink the data 3.4×, but
the floor is 27.5 MB, not the hoped 5–10 MB, and the residue is irreducible
information rather than format overhead. The bundle stands; no reader changed.

## What was built

`validation/madgraph/compact_events.py` (pixi environment `lhe-compact`:
`pylhe`, `pyarrow`, `awkward`) reads each run's `unweighted_events.lhe.gz`
through `pylhe` and writes one Parquet file per run holding only what the gates
consume:

* per external leg — `IDUP`, `ISTUP`, `MOTHUP`, `ICOLUP`, `(px, py, pz, E)`,
  mass, lifetime, `SPINUP`;
* per event — `IDPRUP`, `XWGTUP`, `SCALUP`, `AQEDUP`, `AQCDUP`, and the
  `<mgrwt>` scale-replay payload (`<rscale>`, and `<pdfrwt>`'s flavours, momentum
  fractions and scales per beam);
* per file — the `<init>` block, as JSON in the Parquet key-value metadata.

Columns are struct-of-arrays (one list column per particle field, not a list of
structs), floats carry `BYTE_STREAM_SPLIT`, integers carry dictionaries, and the
file is zstd level 22 in one row group.

Dropped, because nothing reads it: `<rwgt>`, `<asrwt>`, `<totfact>`.

## Criterion 1 — size: FAIL

| run | events | `.lhe.gz` | Parquet | ratio |
|---|---:|---:|---:|---:|
| `bbx_to_ccx_emmm_qcd0` | 10000 | 2 733 191 | 2 688 757 | 1.0× |
| `ddx_to_epemg` | 10000 | 1 331 956 | 1 250 650 | 1.1× |
| `ee_to_ee` | 10000 | 377 583 | 273 777 | 1.4× |
| `ee_to_mumu` | 10000 | 398 930 | 287 534 | 1.4× |
| `ee_to_mumu_tata_qcd0` | 10000 | 1 753 486 | 1 732 209 | 1.0× |
| `ee_to_mumua` | 10000 | 1 241 267 | 1 213 068 | 1.0× |
| `ee_to_tatah` | 10000 | 1 264 696 | 1 111 922 | 1.1× |
| `ee_to_ttx` | 10000 | 384 223 | 281 677 | 1.4× |
| `ee_to_wpwm` | 10000 | 375 513 | 273 860 | 1.4× |
| `ee_to_zh` | 10000 | 378 783 | 280 998 | 1.3× |
| `gg_to_gg` | 10000 | 384 118 | 290 860 | 1.3× |
| `gg_to_ttx` | 10000 | 388 032 | 289 534 | 1.3× |
| `gu_to_epemu` | 10000 | 1 306 965 | 1 232 267 | 1.1× |
| `gux_to_epemux` | 10000 | 1 307 343 | 1 237 107 | 1.1× |
| `pp_to_bb` | 10000 | 10 912 137 | 1 241 560 | 8.8× |
| `pp_to_bb_qcd2` | 10000 | 10 912 681 | 1 247 593 | 8.7× |
| `pp_to_ll` | 10000 | 9 839 275 | 1 452 723 | 6.8× |
| `pp_to_ll_qcd0` | 10000 | 9 839 289 | 1 452 723 | 6.8× |
| `pp_to_llj` | 10000 | 11 567 486 | 1 939 761 | 6.0× |
| `pp_to_llj_fixed` | 10000 | 11 208 650 | 1 689 151 | 6.6× |
| `pp_to_llj_qcd2_qed2` | 10000 | 11 567 509 | 1 939 761 | 6.0× |
| `uux_to_ccx_emmm_qcd0` | 8747 | 2 357 124 | 2 319 556 | 1.0× |
| `uux_to_epemg` | 10000 | 1 313 387 | 1 259 600 | 1.0× |
| `uux_to_mumu` | 10000 | 380 656 | 276 502 | 1.4× |
| `uux_to_uux` | 10000 | 372 599 | 274 492 | 1.4× |
| **total** | | **93 896 879** | **27 537 642** | **3.4×** |

27.5 MB is roughly twice the outer bound at which committing was worth doing.

The split of the two regimes is the `<rwgt>` block: across the whole corpus it is
369 040 000 of 596 793 244 bytes of plain event text — **61.8% of the Les Houches
data is systematic weight variations no gate reads**. That is where the 6–9× on
the hadronic runs comes from, and its absence is why the lepton-beam runs, which
carry no `<rwgt>`, barely move.

### Where the remaining bytes are

Compressed bytes per column, summed over all 25 files:

| column | bytes |
|---|---:|
| `pz` | 6 443 645 |
| `E` | 5 814 657 |
| `px` | 4 925 721 |
| `py` | 4 914 215 |
| `pdfrwt_x` | 1 059 209 |
| mass | 958 495 |
| `SCALUP` | 854 285 |
| `AQCDUP` | 821 575 |
| everything else | 1 745 840 |

The four momentum columns are 22.1 MB, 81% of the total. There are 1 314 204
external-leg lines in the corpus; at 8 bytes a double the momenta are 42.1 MB
raw, so `BYTE_STREAM_SPLIT` + zstd is already winning 1.9× on them (the incoming
legs' zero transverse components and fixed beam energies are what compresses;
the final-state components do not).

### The floor is information, not format

A momentum component is printed to eleven significant digits — about 36.5 bits —
and the f64 it converts to has a fully random mantissa. Five million independent
such values would be ~24 MB, which is *more* than the 22.1 MB the momentum
columns actually occupy: the encoder is already living off the structural
redundancy (zero transverse components and fixed energies on the incoming legs)
and paying close to full price for everything else. To check that nothing cheaper
is nonetheless being left on the table, the printed decimals were
re-encoded as `(int64 mantissa, int32 exponent)` column pairs with
`DELTA_BINARY_PACKED` — the representation whose entropy matches the printed
digits exactly rather than the f64's 53 bits:

| encoding of `px,py,pz,E,m` over all runs | bytes |
|---|---:|
| f64 + `BYTE_STREAM_SPLIT` + zstd-22 | 25 161 279 |
| printed decimal + `DELTA_BINARY_PACKED` + zstd-22 | 22 979 918 |

A 9% saving — and *worse* than f64 on every fixed-beam lepton run, where the structured
zeros compress better as doubles than as mantissa/exponent pairs. There is no
encoding-side win left. The only lever that reaches 5–10 MB is retaining fewer
events, which costs gate strength: `validate_alphas` is a per-event `AQCDUP`
oracle over 180k events precisely because that is stronger than a scalar, and the
planned `samples` category needs MadGraph's full 10k per run as the reference
sample for its KS and χ² statistics. That is a coverage decision, not a storage
one, and it is not taken here.

## Criterion 2 — fidelity: PASS

Two independent checks, both exact.

**Write/read round trip.** `compact_events.py` reads each file back after writing
it and compares every retained column value-for-value against what went in;
all 25 runs pass. Parquet stores IEEE-754 doubles, so this is exactness, not a
tolerance.

**Cross-language.** The interesting claim is not that Parquet preserves a double,
but that the double in the column is the one a Rust gate would have got by
parsing the printed token itself. Measured directly: an order-sensitive FNV-1a
over the big-endian `f64::to_bits` of every `px, py, pz, E, m` token, computed
once by Rust's `str::parse::<f64>` walking the Les Houches text and once by
Python over the Parquet columns. **All 25 runs hash identically over 6 571 020
values.** Both languages' decimal-to-double conversions are correctly rounded, and
this is that stated hypothesis pinned rather than assumed.

**Determinism.** Re-running the projection over an unchanged input reproduces the
file byte-for-byte (`uux_to_uux.parquet`, sha256 `5a67cd15…dffbd19f`, twice),
within a pinned `lhe-compact` environment. Parquet stamps the writer version into
the footer, so the guarantee is per-environment, which is what a committed
reference needs. The requirement that would have gated committing is met; the
verdict turns on size alone.

*What this pair cannot detect*: a field dropped from the projection entirely. The
round trip compares the columns that exist against the columns that exist; a gate
that later needs `<totfact>` or a `<rwgt>` variation would find it simply absent.
The projection's completeness rests on the reading of the four consumers below,
not on either measurement.

## The non-event components

Everything the bundle carries besides the event files — banners, run and
parameter cards, per-subprocess `leshouche.inc` and `matrix1_orig.f`, combined
`results.dat`, per-channel `run_*_log.txt`, the fixed-grid amplitude CSVs — is
1 711 files, 12 909 861 bytes raw, **493 880 bytes as tar + zstd-19**. It is
committable today and always was; it is not what made the bundle large. Had the
event route won, these would have gone under `validation/madgraph/` as committed
files and the bundle would have disappeared entirely.

## The byte-round-trip gate's home

`validate_lhef::banked_files_round_trip_byte_for_byte` parses every banked run's
`<init>` and every `<event>` into this crate's record types, re-serialises, and
requires identical bytes. It reads raw text by construction and no projection can
serve it.

**Decision: unchanged.** It stays in the banked layer, reading the fetched
bundle's raw `.lhe.gz`, because the bundle stays.

Had the projection won, the recommendation would have been the subset option
rather than moving the gate to the oracle layer: the gate's evidence is per-line
column layout, and three runs reach every layout this crate writes —
`ee_to_mumu` (colourless, no `<mgrwt>`), `gg_to_ttx` (colour lines in both slots
on every leg) and `pp_to_llj_fixed` (hadron-collider `<init>` with a PDF set id,
colour on an incoming leg, a status-2 resonance line, `<mgrwt>` and `<rwgt>`).
Truncated to 200 events each they are 258 774 bytes gzipped and still cover every
layout, at the cost of the "10 000 events long" claim in the gate's own
docstring. Moving the gate to the oracle layer instead would have been the wrong
trade: it is fast, and it is the only thing standing between the writer and a
silent format regression.

## Incidental finding: the bundle double-compresses

`assemble_bundle.sh` tars the already-gzipped `.lhe.gz` files and runs zstd-19
over the result, which cannot compress them further. Decompressing them into the
archive and letting zstd do the work instead:

| bundle contents | bytes |
|---|---:|
| 25 `.lhe.gz` as they sit, tar + zstd-19 | ~90 100 000 |
| the same 25 files as plain `.lhe` text, tar + zstd-19 | 58 629 865 |

A 35% smaller fetch for no fidelity loss and no change to any gate's inputs —
the unpack step would gzip them back, or the gates would read `.lhe`. Not taken
here (re-pinning the archive is the bundle's own concern); filed in the backlog.

## What is committed from this investigation

`validation/madgraph/compact_events.py` and the `lhe-compact` pixi environment,
so the numbers above are reproducible and so the projection exists if the bundle
is ever re-cut around it. Its output directory `validation/madgraph/events/` is
gitignored; nothing in the test suite reads it, and no reference generator runs
it.
