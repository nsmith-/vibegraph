#!/usr/bin/env python3
"""Reduce the instrumented MadGraph clustering dump to the committed oracle.

Three subcommands, driven by gen_kt_cluster_dumps.sh:

  reproduce  the precondition: the instrumented replay's unweighted event file
             is the banked one, event text byte-for-byte, banner modulo the
             metadata a second run cannot repeat (dates, paths, run tag).
  extract    normalise the raw per-process Fortran dump to one JSON object per
             written event, in the event file's own order, and run the gate:
             every event's dumped mu_R / mu_F reproduce that event's SCALUP,
             <rscale> and <pdfrwt>.
  manifest   sha256 the per-run dumps into the committed manifest.

The Fortran side stays dumb: it writes pipe-separated tagged records at 18
significant digits, keyed to an event by the momenta the event file carries.
All grammar knowledge lives here.

Record grammar of the matched-generation (ickkw > 0) extension
--------------------------------------------------------------

Each written event's JSON object is {"index": i, "records": [[TAG, ...], ...]}
with the records in the order MadEvent produced them. Under ickkw > 0 one
point runs setclscales twice, and both calls land in the one record set: the
first call's records start at the SCL record whose keepq2bck field is F, the
second call's (from rewgt) at the SCL record whose keepq2bck field is T, and
rewgt's own records follow the second call. Fields, after the tag:

  SCL     iconfig, iproc, ivec, imirror, keepq2bck, init_mode, scale on entry,
          q2fact(1) on entry, q2fact(2) on entry, njetstore(iconfig) on entry
  Q2OVR   BEFORE|AFTER, jcentral(1), jcentral(2), pt2ijcl(jcentral(1)),
          pt2ijcl(jcentral(2)) (0 where jcentral is 0), q2fact(1), q2fact(2):
          the ickkw > 0 overwrite of the central vertices by the q2fact the
          call was entered with (reweight.f:1114-1119); only a call entered
          with both q2fact > 0 has it, which is the second one
  PT2     ... AFTER_Q2FACT_OVERWRITE is the stage following Q2OVR
  Q2BCK   CENTRAL|OUT, keepq2bck, q2bck(1), q2bck(2), q2fact(1), q2fact(2):
          CENTRAL right after the scalefact block (q2fact is then the central
          scale, before the pdfwgt lowering), OUT on the way out
  PTCL    SETCL|OUT, nexternal, ptclus(1..nexternal): SETCL as setclscales
          leaves it (its own leg order), OUT as write_leshouche holds it after
          the symmetry reordering; OUT(3..nexternal) is what <scales
          pt_clust_N> prints, in order
  RWLEG   i, idup(i,ipsel,iproc), ipdgcl before rewgt's assignment, ipdgcl
          after it, isjet(idup), iqjets(i), goodjet: rewgt overwrites the line's
          PDG only for jet flavours, so a final-state leg whose "after" differs
          from idup reads a value left by an earlier event (the stale-ipdgcl
          defect)
  RWBEG   ipsel, iproc, igraphs(1), asref, alpsfact, x1, x2 (xbk(ib(j))), ib(1),
          ib(2), q2bck(1), q2bck(2), q2fact(1), q2fact(2) (the matrix-element
          PDF scales), all_scale(ivec), scale, jlast(1), jlast(2)
  RWVX    n, class, imocl, idacl1, idacl2, pdg(mother), pdg(d1), pdg(d2),
          ipart(1,mother), ipart(2,mother), isr arm, fsr arm, goodjet(mother),
          goodjet(d1), goodjet(d2), q2now, alpsfact*sqrt(q2now),
          alpha_s(alpsfact*sqrt(q2now)) (0 unless reweighted), asref, ratio,
          rewgt after the vertex. class: CORE (n = nexternal-2, never
          reweighted), ISR, FSR (reweighted), NONE (condition false), FAKE_ID
          (qualified, mother is fake_id), KILL_Q2 (qualified at q2now <= 4)
  RWPDF   n, beam side j, daughter slot i, ib(j), pdg of the line entering the
          vertex, zcl(n), x after z, q2now, q2prev (pt2pdf of the daughter),
          action, f(x, q_now), f(x, q_prev), ratio, rewgt after the step.
          action: FIRST (no ratio), RATIO, NOT_RISING, NONE (rising scale with
          n > jlast(j): no ratio and no pt2pdf update), PS_START (ickkw = 2),
          KILL_PDF (denominator below 1e-10)
  RWKILL  reason (SETCLSCALES | ALPHAS_Q2 | PDF_DENOMINATOR), n
  RWEND   rewgt, q2fact(1), q2fact(2) (reset to q2bck), vec_igraph(ivec)
  CFG     iconfig, igraphs(1), vec_igraph(ivec), ickkw at write time
  CNT     running per-process counts at write time: first calls, second
          calls, STORE_AND_RECLUSTER, RESTRICTED_RECLUSTER
  OUT     scale, q2fact(1), q2fact(2), SCALUP, AQCDUP, AQEDUP, npart (the
          header SCALUP / AQCDUP as written)

Outside the event record sets:

  CONST2  (per directory) ickkw, xqcut, alpsfact, asrwgtflavor, maxjetflavor,
          pdfwgt, hmult, ktscheme, chcluster, auto_ptj_mjj, ptj, mmjj, drjj,
          drjl, xptj, use_syst, scalefact -- after setcuts' xqcut rewrite
  MEMOX   kind, iconfig, iproc, njets, njetstore, igraphs(1), keepq2bck,
          first calls so far, second calls so far: every jet-memo branch taken
          at any point, kept or not (the census reads these)
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import math
import re
import sys
from collections import defaultdict
from pathlib import Path


def open_maybe_gz(path: Path, mode: str = "rt"):
    if str(path).endswith(".gz"):
        return gzip.open(path, mode)
    return open(path, mode)


# ─────────────────────────── the precondition check ───────────────────────────

# Banner lines a second run cannot repeat even when it is the same run: the
# dates MadGraph stamps, the absolute path of the directory it ran in, and the
# per-run timing/host lines. Everything else — the cards, the cross section, the
# init block — is physics or configuration and must match.
_METADATA = re.compile(
    r"""^\s*(
          \#\*?\s*(Generated\s+by|on\s+the|Run\s+by)
        | <MGGenerationInfo>|</MGGenerationInfo>
        | \#\s*Integrated\s+weight
        | \#\s*Truncated\s+weight
        | \#\s*Unit\s+wgt
        | <slha>|</slha>
      )""",
    re.VERBOSE | re.IGNORECASE,
)
_DATE = re.compile(r"\d{4}-\d{2}-\d{2}[ T]\d{2}:\d{2}:\d{2}")
_PATH = re.compile(r"/[\w./+-]*(?:vibegraph|madgraph)[\w./+-]*")


def split_lhe(path: Path) -> tuple[list[str], str]:
    """(banner lines, event text) — the event text starting at the first <event>."""
    with open_maybe_gz(path) as f:
        text = f.read()
    idx = text.find("<event>")
    if idx < 0:
        raise SystemExit(f"!!! {path}: no events")
    return text[:idx].splitlines(), text[idx:]


def normalise_banner(lines: list[str]) -> list[str]:
    out = []
    for line in lines:
        if _METADATA.match(line):
            continue
        line = _DATE.sub("<date>", line)
        line = _PATH.sub("<path>", line)
        out.append(line)
    return out


def cmd_reproduce(args: argparse.Namespace) -> int:
    banked_banner, banked_events = split_lhe(Path(args.banked))
    replay_banner, replay_events = split_lhe(Path(args.replay))

    ok = True
    if banked_events != replay_events:
        ok = False
        nb = banked_events.count("<event>")
        nr = replay_events.count("<event>")
        print(f"!!! event text differs: {nb} banked events, {nr} replayed", file=sys.stderr)
        b = banked_events.splitlines()
        r = replay_events.splitlines()
        for i, (x, y) in enumerate(zip(b, r)):
            if x != y:
                print(f"    first differing line {i}:", file=sys.stderr)
                print(f"      banked: {x!r}", file=sys.stderr)
                print(f"      replay: {y!r}", file=sys.stderr)
                break
        else:
            print(f"    lines: {len(b)} banked vs {len(r)} replayed", file=sys.stderr)
    else:
        n = banked_events.count("<event>")
        print(f"    event text byte-identical over {n} events")

    nb, nr = normalise_banner(banked_banner), normalise_banner(replay_banner)
    if nb != nr:
        # A banner difference does not by itself mean a different run, so it is
        # reported rather than fatal; the event text is the claim.
        print("    banner differs after metadata normalisation:", file=sys.stderr)
        shown = 0
        for i, (x, y) in enumerate(zip(nb, nr)):
            if x != y and shown < 10:
                print(f"      {i}: banked {x!r}", file=sys.stderr)
                print(f"      {i}: replay {y!r}", file=sys.stderr)
                shown += 1
        if len(nb) != len(nr):
            print(f"      lengths {len(nb)} vs {len(nr)}", file=sys.stderr)
    else:
        print("    banner identical modulo run metadata")

    return 0 if ok else 1


# ───────────────────────────── the dump grammar ──────────────────────────────

# Records that belong to a process directory rather than to an event. They are
# re-emitted by every job that touches that directory, so they are deduplicated
# on their full text.
PER_DIRECTORY = {"RUN", "CONST", "CONST2", "NQCD", "MAP", "PDG", "RES", "IFOR"}


def parse_number(tok: str):
    if tok in ("T", "F"):
        return tok == "T"
    try:
        return int(tok)
    except ValueError:
        pass
    try:
        return float(tok)
    except ValueError:
        return tok


def read_raw_records(raw_dir: Path):
    """Yield (tag, fields) over every raw dump shard, in file then line order."""
    shards = sorted(p for p in raw_dir.glob("raw.*") if p.is_file())
    if not shards:
        raise SystemExit(f"!!! no raw dump shards under {raw_dir}")
    for shard in shards:
        # A shard written through VG_KTDUMP_GZIP arrives as raw.<pid>.gz.
        with open_maybe_gz(shard) as f:
            for line in f:
                line = line.rstrip("\n")
                if not line:
                    continue
                parts = line.split("|")
                yield shard.name, parts[0].strip(), [p.strip() for p in parts[1:]]


def particle_key(rows) -> tuple:
    """The key that ties a dump record set to an event: its written particles.

    Each row is (pdg, E, px, py, pz) in the order the file lists them, because
    the dump reads the same array write_event does and nothing between them
    reorders it. Order and flavour both matter: a mirrored beam assignment
    gives the same set of momenta with the two beams exchanged, so a key that
    sorted them would hand one event's record set to another.

    Eleven significant digits — what the event file carries — on both sides.
    Rounding the dump's eighteen to anything coarser invites a tie: the event
    file's value is already rounded, so a value whose next digit is a 5
    followed by zeros can round one way there and the other way here. At eleven
    digits there is no tie to break, because a double is a dyadic rational and
    is never exactly a twelve-digit decimal ending in 5.
    """
    return tuple(
        (int(row[0]),) + tuple(f"{v:+.10e}" for v in row[1:]) for row in rows
    )


def lhe_events(path: Path):
    """Yield one dict per <event>: momenta, SCALUP, rscale, pdfrwt scales."""
    with open_maybe_gz(path) as f:
        text = f.read()
    for block in re.findall(r"<event>\n(.*?)</event>", text, re.S):
        lines = [ln for ln in block.splitlines() if ln.strip()]
        header = lines[0].split()
        npart = int(header[0])
        scalup = float(header[3])
        aqed = float(header[4])
        aqcd = float(header[5])
        mom = []
        for ln in lines[1 : 1 + npart]:
            f_ = ln.split()
            # pdg ... px py pz E m  -> (pdg, E, px, py, pz)
            mom.append(
                (int(f_[0]), float(f_[9]), float(f_[6]), float(f_[7]), float(f_[8]))
            )
        rscale = None
        pdfrwt = {}
        mgrwt = {"asrwt": None, "pdfrwt": {}}
        ptclust = None
        for ln in lines[1 + npart :]:
            m = re.match(r"<scales\s+(.*?)>\s*</scales>", ln.strip())
            if m:
                ptclust = [float(v) for v in re.findall(r'pt_clust_\d+="([^"]+)"', m.group(1))]
            m = re.match(r"<asrwt>\s*(\d+)(.*)</asrwt>", ln.strip())
            if m:
                mgrwt["asrwt"] = [float(v) for v in m.group(2).split()][: int(m.group(1))]
            m = re.match(r'<pdfrwt beam="(\d)">\s*(\d+)(.*)</pdfrwt>', ln.strip())
            if m:
                n = int(m.group(2))
                nums = m.group(3).split()
                mgrwt["pdfrwt"][int(m.group(1))] = [
                    (int(nums[k]), float(nums[n + k]), float(nums[2 * n + k])) for k in range(n)
                ]
            m = re.match(r"<rscale>\s*\S+\s+(\S+)</rscale>", ln)
            if m:
                rscale = float(m.group(1))
            m = re.match(r'<pdfrwt beam="(\d)">\s*(\d+)(.*)</pdfrwt>', ln)
            if m and int(m.group(2)) > 0:
                nums = m.group(3).split()
                # n pdg entries, then n x values, then n scales
                n = int(m.group(2))
                pdfrwt[int(m.group(1))] = float(nums[n + 2 * n - 1])
        yield {
            "npart": npart,
            "scalup": scalup,
            "aqed": aqed,
            "aqcd": aqcd,
            "momenta": mom,
            "rscale": rscale,
            "pdfrwt": pdfrwt,
            "mgrwt": mgrwt,
            "ptclust": ptclust,
            "key": particle_key(mom),
        }


def cmd_extract(args: argparse.Namespace) -> int:
    """Normalise one run's raw dump into the event file's own order.

    Three passes so the peak cost is one event's records rather than a run's:
    the event file names the events wanted, the shards are streamed and the
    wanted record sets written out as they appear, and the temporary file is
    re-read in event order through the offsets the second pass noted.
    """
    raw_dir = Path(args.raw_dir)
    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)

    # Pass 1 — the event file: what to keep, and what each event's scale is.
    wanted: dict[tuple, int] = {}
    expect: list[dict] = []
    for idx, ev in enumerate(lhe_events(Path(args.lhe))):
        if ev["key"] in wanted:
            print(f"!!! events {wanted[ev['key']]} and {idx} share a particle key",
                  file=sys.stderr)
            return 1
        wanted[ev["key"]] = idx
        expect.append(ev)
    n_events = len(expect)

    # Pass 2 — the shards.
    directory_records: dict[str, list] = defaultdict(list)
    seen_directory: set[str] = set()
    offsets: dict[int, int] = {}
    digests: dict[int, str] = {}
    n_duplicate = 0
    coverage: dict[str, dict] = {}
    alphas_pairs: list[tuple[float, float]] = []
    current: list | None = None
    n_raw_events = 0
    n_truncated = 0
    mismatched: list[tuple[int, list[str]]] = []
    mlm_checks: dict[str, int] = defaultdict(int)

    memox: dict[str, int] = defaultdict(int)
    shard_dir: dict[str, str] = {}
    memo_state: dict[str, set] = defaultdict(set)
    memox_rows: list[list] = []
    counts_by_shard: dict[str, list[int]] = {}
    tmp_path = out_path.with_suffix(".tmp")
    with open(tmp_path, "w") as tmp:
        for shard, tag, fields in read_raw_records(raw_dir):
            if tag == "SHARD":
                shard_dir.setdefault(shard, Path(fields[0]).name if fields else "?")
                continue
            if tag == "MEMOX":
                row = [parse_number(x) for x in fields]
                memox[f"{row[0]}|keepq2bck={row[6]}"] += 1
                if len(memox_rows) < 200:
                    memox_rows.append([shard] + row)
                continue
            if tag in PER_DIRECTORY:
                text = tag + "|" + "|".join(fields)
                if text not in seen_directory:
                    seen_directory.add(text)
                    directory_records[tag].append([parse_number(x) for x in fields])
                continue
            if tag == "BEG":
                current = []
                continue
            if current is None:
                continue
            current.append([tag] + [parse_number(x) for x in fields])
            if tag != "END":
                continue
            n_raw_events += 1
            records, current = current, None
            for r in records:
                if r[0] == "CNT":
                    old = counts_by_shard.get(shard, [0, 0, 0, 0])
                    counts_by_shard[shard] = [max(a, b) for a, b in zip(old, r[1:5])]
            # The END record carries the writer's running count of fields it
            # could not fit, which is per-process and only ever grows.
            if fields:
                n_truncated = max(n_truncated, records[-1][1])
            # OUTMOM is |i|E|px|py|pz|m|pdg, so the key's (pdg, E, px, py, pz).
            mom = [[r[7]] + r[2:6] for r in records if r[0] == "OUTMOM"]
            if not mom:
                continue
            idx = wanted.get(particle_key(mom))
            if idx is None:
                continue
            # MadGraph evaluates some phase-space points in more than one job, so
            # the same event can be dumped twice. Identical record sets are the
            # same reading of the same event and one is kept; record sets that
            # disagree would mean the event's clustering is not a function of the
            # event, which is a finding and not something to pick a winner from.
            body = json.dumps(records, sort_keys=True)
            digest = hashlib.blake2b(body.encode(), digest_size=16).hexdigest()
            if idx in offsets:
                if digest == digests[idx]:
                    n_duplicate += 1
                    continue
                print(f"!!! event {idx} matched two *differing* dump record sets",
                      file=sys.stderr)
                return 1
            digests[idx] = digest
            # The jet memo is per operating-system process and per channel,
            # fixed by that process's first point in the channel. The value a
            # written event's first call found tells which state it was judged
            # against; more than one value for one directory and channel means
            # the reclustering decision depends on which job made the event.
            for r in records:
                if r[0] == "SCL" and r[5] is False:
                    memo_state[f"{shard_dir.get(shard, '?')}:{r[1]}"].add(r[10])
                    break
            if any(r[0] == "RWBEG" for r in records):
                problems = check_matched(records, expect[idx], mlm_checks)
            else:
                problems = check_scales(records, expect[idx])
            if problems:
                mismatched.append((idx, problems))
            tally(records, coverage)
            out = [r for r in records if r[0] == "OUT"][0]
            alphas_pairs.append((out[1], out[5]))
            offsets[idx] = tmp.tell()
            tmp.write(json.dumps({"index": idx, "records": records}, sort_keys=True) + "\n")

    # The event header's alpha_s is the only handle the runs taken with
    # use_syst = False give on the renormalisation scale, and on its own it says
    # nothing about the *dumped* scale. Sorting the run's events by the dumped
    # scale and requiring the event file's alpha_s to fall along with it ties the
    # two together: a dumped scale read from the wrong place would not order the
    # coupling. The check is only as sharp as the run's spread of scales, so the
    # number of distinct scales is recorded beside it.
    alphas_pairs.sort()
    worst = 0.0
    for (s1, a1), (s2, a2) in zip(alphas_pairs, alphas_pairs[1:]):
        if a2 > a1 and s2 > s1:
            worst = max(worst, (a2 - a1) / a1)
    n_distinct = len({round(s, 10) for s, _ in alphas_pairs})
    alphas_ordering = {"max_rise": worst, "n_distinct_scales": n_distinct}

    # Pass 3 — back out in event order.
    missing = [i for i in range(n_events) if i not in offsets]
    with open(tmp_path) as tmp, gzip.open(out_path, "wt") as out:
        head = {
            "process": args.name,
            "n_events": n_events,
            "n_matched": len(offsets),
            "n_raw_dump_events": n_raw_events,
            "n_duplicate_dumps": n_duplicate,
            "coverage": {k: dict(sorted(v.items())) for k, v in sorted(coverage.items())},
            "alphas_ordering": alphas_ordering,
            "directory": {k: v for k, v in sorted(directory_records.items())},
            "census": {
                "memo_branches_all_points": dict(sorted(memox.items())),
                "memo_branch_examples": memox_rows[:20],
                "setclscales_calls_through_last_written_event": {
                    "first": sum(c[0] for c in counts_by_shard.values()),
                    "second": sum(c[1] for c in counts_by_shard.values()),
                    "store_and_recluster": sum(c[2] for c in counts_by_shard.values()),
                    "restricted_recluster": sum(c[3] for c in counts_by_shard.values()),
                },
                "matched_record_checks": dict(sorted(mlm_checks.items())),
                "njetstore_on_entry_by_channel": {
                    k: sorted(v) for k, v in sorted(memo_state.items())
                },
            },
        }
        out.write(json.dumps(head, sort_keys=True) + "\n")
        for idx in range(n_events):
            if idx not in offsets:
                continue
            tmp.seek(offsets[idx])
            out.write(tmp.readline())
    tmp_path.unlink()

    print(f"    {n_events} events in the file, {len(offsets)} matched to a dump record")
    print(
        f"    {n_raw_events} dump records written; the event file keeps"
        f" {n_events}, the rest were dropped by the unweighting"
    )
    if n_duplicate:
        print(f"    {n_duplicate} events were dumped more than once, identically")
    print(f"    -> {out_path}")
    for key in sorted(coverage):
        print(f"    {key}: {dict(sorted(coverage[key].items()))}")
    print(f"    alphas_ordering: {alphas_ordering}")
    if memox or mlm_checks:
        print(f"    memo branches at any point: {dict(sorted(memox.items()))}")
        print(f"    matched-record checks: {dict(sorted(mlm_checks.items()))}")
    if n_truncated:
        print(f"!!! {n_truncated} fields did not fit the dump's line or buffer limit",
              file=sys.stderr)
    if missing or mismatched or n_truncated:
        for idx in missing[:5]:
            print(f"!!! event {idx}: no dump record, momenta {expect[idx]['momenta']}",
                  file=sys.stderr)
        for idx, problems in mismatched[:10]:
            print(f"!!! event {idx}: {problems}", file=sys.stderr)
        print(
            f"!!! {len(missing)} events without a dump record, "
            f"{len(mismatched)} with a scale mismatch",
            file=sys.stderr,
        )
        return 1
    print("    gate: every event's dumped mu_R / mu_F reproduce the event file's own")
    return 0


def tally(records: list, cov: dict) -> None:
    """Accumulate the read-outs the clustering spec asks a dump to settle.

    Each counter answers one question the spec could only pose: whether the
    integration channel's identity reaches the scale, whether the jet-count memo
    ever re-clusters, whether the boost fires, which arms of the measure and of
    the mu_R / mu_F branch selection a run actually exercises. A branch with a
    zero here is a branch the dump cannot judge an engine on.
    """
    def bump(key, value):
        cov.setdefault(key, {})
        k = str(value)
        cov[key][k] = cov[key].get(k, 0) + 1

    iconfig = None
    for r in records:
        tag = r[0]
        if tag == "SCL":
            iconfig = r[1]
        elif tag == "CLCALL":
            bump("cluster_calls_per_event", r[1])
        elif tag == "MEMO":
            bump("memo", r[1])
        elif tag == "CAND":
            bump("candidate_measure", r[11])
            if r[13]:
                bump("beam_crossing_inflation", "applied")
        elif tag == "BOOST":
            bump("boost", "fired" if r[3] else "not_fired")
        elif tag == "GRPH" and r[2] == "after":
            bump("igraphs1_is_iconfig", r[4] == iconfig)
        elif tag == "OVR":
            bump("mt2last_override", r[1])
            bump("jcentral_override_beam1", r[2])
            bump("jcentral_override_beam2", r[3])
        elif tag == "MUR":
            bump("mur_branch", r[1])
        elif tag == "REJ":
            bump("rejected", r[1])
    muf = [r for r in records if r[0] == "MUF"]
    if muf:
        bump("muf_branch", muf[-1][1])
    if not any(r[0] == "RWBEG" for r in records):
        return
    # The matched-generation read-outs: how many setclscales calls the point
    # made, which vertex classes and PDF-ratio actions rewgt took, whether the
    # central overwrite moved a scale, and the two record-level defects the
    # spec could only name (a final-state PDG left by an earlier event, and
    # addmothers' fallback reached with vec_igraph = 0).
    bump("setclscales_calls_per_event", sum(1 for r in records if r[0] == "SCL"))
    for r in records:
        tag = r[0]
        if tag == "RWVX":
            bump("rwgt_vertex_class", r[2])
        elif tag == "RWPDF":
            bump("rwgt_pdf_action", r[10])
        elif tag == "RWKILL":
            bump("rwgt_kill", r[1])
        elif tag == "Q2OVR" and r[1] == "AFTER":
            bump("q2fact_overwrite_jcentral", f"{r[2]},{r[3]}")
        elif tag == "RWLEG" and r[1] > 2:
            bump("final_leg_ipdgcl_differs_from_idup", r[4] != r[2])
        elif tag == "CFG":
            bump("vec_igraph_zero", r[3] == 0)
            bump("vec_igraph_is_igraphs1", r[3] == r[2])
            bump("vec_igraph_is_iconfig", r[3] == r[1])


# The event file prints SCALUP with 7 significant digits and <rscale> and the
# <pdfrwt> scales with 8, so the dump's eighteen can only be asked to agree to
# within the rounding that writing them cost: half a unit in the last written
# place. The bound below is three quarters of one such unit — enough headroom
# that no correct value is ever called a mismatch, and still inside the next
# value the file could have printed, so a scale that genuinely differs cannot
# hide in it.
def close(a: float, b: float, digits: int) -> bool:
    if a == b:
        return True
    scale = max(abs(a), abs(b))
    if scale == 0:
        return False
    ulp = 10.0 ** (math.floor(math.log10(scale)) - digits + 1)
    return abs(a - b) <= 0.75 * ulp


def check_scales(records: list, ev: dict) -> list[str]:
    out = [r for r in records if r[0] == "OUT"]
    if not out:
        return ["no OUT record"]
    _, scale, q2fact1, q2fact2, scalup, aqcd, aqed = out[0][:7]
    problems = []
    if not close(scalup, ev["scalup"], 7):
        problems.append(f"SCALUP dump {scalup!r} vs event {ev['scalup']!r}")
    # alpha_s(mu_R) is in every event header, and it is strictly monotonic in
    # mu_R. It is what pins the renormalisation scale on the runs that carry no
    # <rscale> because they were taken with use_syst = False.
    if not close(aqcd, ev["aqcd"], 7):
        problems.append(f"AQCD dump {aqcd!r} vs event {ev['aqcd']!r}")
    if not close(aqed, ev["aqed"], 7):
        problems.append(f"AQED dump {aqed!r} vs event {ev['aqed']!r}")
    if ev["rscale"] is not None and not close(scale, ev["rscale"], 8):
        problems.append(f"rscale dump {scale!r} vs event {ev['rscale']!r}")
    for beam, q2 in ((1, q2fact1), (2, q2fact2)):
        want = ev["pdfrwt"].get(beam)
        if want is None:
            continue
        got = q2 ** 0.5
        if not close(got, want, 8):
            problems.append(f"pdfrwt{beam} dump {got!r} vs event {want!r}")
    return problems


def check_matched(records: list, ev: dict, stats: dict) -> list[str]:
    """The ickkw > 0 gate: the dump against the event's own record.

    The header's SCALUP / AQCDUP / AQEDUP and <rscale> as in check_scales. SCALUP
    must further be sqrt(max q2bck) of the second call's exit, and rewgt must
    leave q2fact = q2bck. Where use_syst wrote an <mgrwt> block, it is
    MadEvent's own serialisation of the same reweighting, so it is replayed from
    the dump entry by entry: <asrwt> is the ISR/FSR vertices' sqrt(q2now) in
    order, and each <pdfrwt> is the matrix-element PDF entry followed by one
    entry per RATIO step, with the momentum fraction *before* that vertex's z
    (reweight.f:1719-1723). <scales pt_clust_N> is PTCL OUT's legs 3..n.
    """
    problems = [p for p in check_scales_header(records, ev)]
    out = [r for r in records if r[0] == "OUT"][0]
    scalup = out[4]
    q2b = [r for r in records if r[0] == "Q2BCK" and r[1] == "OUT"]
    if not q2b:
        return problems + ["no Q2BCK OUT record"]
    q2b = q2b[-1]
    want = max(q2b[3], q2b[4]) ** 0.5
    if not close(scalup, want, 12):
        problems.append(f"SCALUP {scalup!r} is not sqrt(max q2bck) {want!r}")
    rwend = [r for r in records if r[0] == "RWEND"]
    if not rwend:
        return problems + ["no RWEND record"]
    rwend = rwend[-1]
    if (rwend[2], rwend[3]) != (q2b[3], q2b[4]):
        problems.append(f"rewgt left q2fact {rwend[2:4]} != q2bck {q2b[3:5]}")
    scl = [r for r in records if r[0] == "SCL"]
    stats[f"setclscales_calls={len(scl)}"] += 1

    if ev["ptclust"] is not None:
        ptcl = [r for r in records if r[0] == "PTCL" and r[1] == "OUT"]
        got = ptcl[-1][3 + 2:] if ptcl else []
        if len(got) != len(ev["ptclust"]) or any(
            abs(a - b) > 6e-6 * max(1.0, abs(b)) for a, b in zip(got, ev["ptclust"])
        ):
            problems.append(f"pt_clust {ev['ptclust']} vs dump {got}")
        else:
            stats["pt_clust_agree"] += 1

    mg = ev["mgrwt"]
    if mg["asrwt"] is not None:
        want_as = [r[16] ** 0.5 for r in records if r[0] == "RWVX" and r[2] in ("ISR", "FSR")]
        if len(want_as) != len(mg["asrwt"]) or not all(
            close(a, b, 8) for a, b in zip(want_as, mg["asrwt"])
        ):
            problems.append(f"asrwt {mg['asrwt']} vs dump {want_as}")
        else:
            stats["asrwt_agree"] += 1
    if mg["pdfrwt"]:
        rwbeg = [r for r in records if r[0] == "RWBEG"][-1]
        legs = {r[1]: r for r in records if r[0] == "RWLEG"}
        chains = {}
        for j in (1, 2):
            chain = [(legs[j][3], rwbeg[5 + j], rwbeg[11 + j] ** 0.5)]
            for r in records:
                if r[0] == "RWPDF" and r[2] == j and r[10] == "RATIO":
                    z, x = r[6], r[7]
                    xb = x / z if 0.0 < z < 1.0 else x
                    chain.append((r[5], xb, r[8] ** 0.5))
            chains[j] = chain

        def agree(file_chain, dump_chain):
            return len(file_chain) == len(dump_chain) and all(
                f[0] == d[0] and close(f[1], d[1], 8) and close(f[2], d[2], 8)
                for f, d in zip(file_chain, dump_chain)
            )

        direct = all(agree(mg["pdfrwt"].get(b, []), chains[b]) for b in (1, 2))
        flipped = all(agree(mg["pdfrwt"].get(b, []), chains[3 - b]) for b in (1, 2))
        if direct:
            stats["pdfrwt_agree_direct"] += 1
        elif flipped:
            stats["pdfrwt_agree_beams_flipped"] += 1
        else:
            problems.append(f"pdfrwt {mg['pdfrwt']} vs dump {chains}")
    return problems


def check_scales_header(records: list, ev: dict) -> list[str]:
    """check_scales without the <pdfrwt> comparison, which under ickkw > 0
    carries the PDF-ratio chain rather than the factorisation scale."""
    saved = ev["pdfrwt"]
    ev["pdfrwt"] = {}
    try:
        return check_scales(records, ev)
    finally:
        ev["pdfrwt"] = saved


# ───────────────────────────────── manifest ──────────────────────────────────


def cmd_manifest(args: argparse.Namespace) -> int:
    dumps = Path(args.dumps)
    entries = {}
    for name in args.processes:
        path = dumps / f"{name}.jsonl.gz"
        if not path.is_file():
            print(f"⊘ {name}: no dump, not in the manifest", file=sys.stderr)
            continue
        h = hashlib.sha256(path.read_bytes()).hexdigest()
        with gzip.open(path, "rt") as f:
            head = json.loads(f.readline())
        entries[name] = {
            "path": f"validation/madgraph/output/ktdump/dumps/{name}.jsonl.gz",
            "sha256": h,
            "n_events": head["n_events"],
            "n_matched": head["n_matched"],
            "coverage": head.get("coverage", {}),
            "alphas_ordering": head.get("alphas_ordering", {}),
        }
        # The matched-generation census is carried for runs with xqcut or
        # ickkw set (CONST: ickkw at field 5, xqcut at field 8).
        const = head.get("directory", {}).get("CONST", [])
        if any(c[5] > 0 or c[8] > 0 for c in const) and "census" in head:
            entries[name]["census"] = head["census"]
    # A manifest that already pins runs this invocation did not name keeps them.
    out_path = Path(args.out)
    if out_path.is_file():
        for name, entry in json.loads(out_path.read_text()).get("runs", {}).items():
            if name not in args.processes:
                entries.setdefault(name, entry)
    doc = {
        "_comment": (
            "Per-event kT-clustering dumps from an instrumented replay of the "
            "banked MadGraph 3.7.1 runs. The dumps themselves live in the "
            "gitignored work area; this file pins what they are. Regenerate "
            "with validation/madgraph/gen_kt_cluster_dumps.sh."
        ),
        "madgraph": "v3.7.1 (b7687064b9a013317ca164aa1395bc9c0e39ae1e)",
        "runs": entries,
    }
    Path(args.out).write_text(json.dumps(doc, indent=2, sort_keys=True) + "\n")
    print(f"    -> {args.out}")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    sub = ap.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("reproduce")
    p.add_argument("--banked", required=True)
    p.add_argument("--replay", required=True)
    p.set_defaults(fn=cmd_reproduce)

    p = sub.add_parser("extract")
    p.add_argument("--name", required=True)
    p.add_argument("--raw-dir", required=True)
    p.add_argument("--lhe", required=True)
    p.add_argument("--out", required=True)
    p.set_defaults(fn=cmd_extract)

    p = sub.add_parser("manifest")
    p.add_argument("--dumps", required=True)
    p.add_argument("--out", required=True)
    p.add_argument("processes", nargs="*")
    p.set_defaults(fn=cmd_manifest)

    args = ap.parse_args()
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
