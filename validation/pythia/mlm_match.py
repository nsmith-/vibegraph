"""Shower two sets of MLM-matched Les Houches files through one Pythia 8
configuration and compare what the kT-MLM matching makes of them.

Side A and side B are each one or more event files of the same matched card
(typically MadEvent's and vibegraph's `pp_to_ll_0j2j_mlm`). Every file is
showered once per Pythia seed, with the same seeds on every file, by
`mlm_match.cc` (built here against the environment's Pythia), which records per
Les Houches event its `@N` (IDPRUP), the matching's veto and the showered
event's kT clustering scales d_01, d_12, d_23 as JetMatchingMadgraph computes
them. Out come, per side:

* the matching acceptance per `@N`, accepted / read;
* the merged cross section, main164's normalisation at IDWTUP = -4: the sum
  of the accepted events' XWGTUP over the number of events in the file, raw
  and as a fraction of the file's own sigma_LHE (the mean XWGTUP);
* the jet rates log10(d_01), log10(d_12), log10(d_23) of the accepted events,
  as normalised shapes;

Every event counts with its XWGTUP, as Pythia weights it at IDWTUP = -4. A
MadEvent file's weights are all equal; a vibegraph file keeps the events its
unweighting found above w_max at their own, larger weight.

and their comparison: a pull per `@N` acceptance and per merged sigma, and a
chi2 per jet-rate histogram with bin errors from both sides.

Statistics. The unit of independence is the Les Houches event: the seeds of one
file shower the same events. Each event's outcome is therefore first averaged
over the seeds (its accepted fraction, its fraction in each bin), and the
errors are the spread of those per-event averages over the side's events, with
the delta method for the normalised shapes. That is exact for independent
events whatever the number of seeds, and it includes the Les Houches
sample's own fluctuation, which the per-seed scatter alone would miss. The
per-file and per-seed values are written beside it so the spreads can be read.
The jet-rate bin errors are re-derived by resampling each side's events
(`--bootstrap`), which assumes nothing about the weight distribution; the
chi2 is reported with both. Independent events is itself an assumption: the
per-file values' chi2/dof about the side's mean tests it (a MadEvent
directory, or a vibegraph sample with a heavy overweight, can scatter beyond
its own error).

A finished Pythia run is read back from `--work` when its command file (which
names the input's and the driver's sha256) is unchanged.

Configurations (`--config`):

* `mg` — MadGraph's own driving, as `madevent_interface.py`'s
  `setup_Pythia8RunAndCard` writes the command file its `pythia8` command
  hands Pythia's main164 (pinned tree `b7687064`, MadGraph 3.7.1; every line
  cites its source in `MG_SETTINGS`). The matching parameters come from the
  command line and the files' own run card, never from `<MGRunCard>`:
  `setMad = off`.
* `setmad` — the same with `JetMatching:setMad = on` and no qCut / nQmatch /
  merge lines: Pythia takes xqcut, maxjetflavor, alpsfact and ickkw from each
  file's `<MGRunCard>`. It shows whether a file's header drives Pythia as
  MadGraph's command file does; on MadEvent's own file it needs
  `--transform strip-cdata`, since Pythia 8.312 drops the CDATA section
  MadGraph wraps the card in.

Transforms (`--transform-a` / `--transform-b`), applied to a copy:

* `strip-cdata` — delete the CDATA markers around `<MGRunCard>`'s content;
* `drop-scales` — delete every event's `<scales .../>` line, so each parton's
  production scale is SCALUP (the negative control: `<scales>` decides which
  partons the matching counts and where each one's shower starts).

Usage:
    pixi run -e pythia validate-mlm-pythia
    pixi run -e pythia python validation/pythia/mlm_match.py \\
        --a MADEVENT.lhe.gz ... --b VIBEGRAPH.lhe ... [--config mg|setmad] \\
        [--seeds S ...] [--qcut Q] [--transform-b drop-scales] --out result.json
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import math
import os
import re
import shutil
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass, field
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parents[2]
SOURCE = Path(__file__).resolve().with_name("mlm_match.cc")
BUILD = ROOT / "target" / "mlm-pythia"
DEFAULT_OUT = ROOT / "target" / "validation-report" / "standalone" / "mlm_pythia.json"

#: MadEvent's samples-grade run of the matched row and the vibegraph samples
#: `generate_mlm_samples.sh` writes: the task's default inputs.
DEFAULT_A = [ROOT / "validation/madgraph/output/pp_to_ll_0j2j_mlm/Events/run_01/unweighted_events.lhe.gz"]
DEFAULT_B_DIR = ROOT / "target" / "mlm-pythia-samples"

DEFAULT_SEEDS = [20261201, 20261202, 20261203]

#: The command-file lines MadGraph's `pythia8` command writes for an MLM run
#: (ickkw = 1) at the default pythia8_card, with where each comes from. Keys
#: whose value depends on the run are filled in by `mg_settings`.
MG_SETTINGS = [
    ("Beams:frameType", "4", "banner.py:1925 (PY8Card, hidden, always written)"),
    ("Check:epTolErr", "1e-2", "banner.py:1931"),
    ("JetMatching:etaJetMax", "1000.0", "banner.py:1936 (always written)"),
    ("JetMatching:setMad", "off", "madevent_interface.py:4398"),
    ("JetMatching:qCut", None, "madevent_interface.py:4408-4409: 1.5*xqcut when the card leaves -1 (pythia8_card_default.dat)"),
    ("Beams:setProductionScalesFromLHEF", "on", "madevent_interface.py:4419"),
    ("JetMatching:merge", "on", "madevent_interface.py:4456"),
    ("JetMatching:scheme", "1", "madevent_interface.py:4457"),
    ("JetMatching:nQmatch", None, "madevent_interface.py:4460: maxjetflavor"),
    ("JetMatching:coneRadius", "1.0", "madevent_interface.py:4462"),
    ("JetMatching:nJetMax", None, "madevent_interface.py:4467-4471: max_n_matched_jets (export_v4.py:5010-5024)"),
    ("JetMatching:doShowerKt", "off", "pythia8_card_default.dat (visible, default off)"),
]

#: Lines of this driver's own, none of which MadGraph sets: the seed, and quiet
#: progress output.
DRIVER_SETTINGS = [
    ("Random:setSeed", "on"),
    ("Next:numberCount", "0"),
    ("Next:numberShowEvent", "0"),
    ("Next:numberShowProcess", "0"),
    ("Next:numberShowInfo", "0"),
]

#: log10(d / GeV) histogram range and the finest bin width; bins are merged
#: from the left until each holds `--min-count` accepted events on both sides.
HIST_LO, HIST_HI, HIST_STEP = 0.0, 3.0, 0.1
DJR_NAMES = ["log10_d01", "log10_d12", "log10_d23"]

RUN_CARD_LINE = re.compile(r"^\s*(\S+)\s*=\s*(\w+)\s*(?:[!#].*)?$")
SCALES_LINE = re.compile(r"^\s*<scales\b[^>]*(?:/>|>\s*</scales>)\s*$")


# ───────────────────────────── the input files ──────────────────────────────


@dataclass
class LheFile:
    path: Path
    sha256: str
    n_events: int
    xsecup: dict[int, float]
    run_card: dict[str, str]
    max_light_partons: int
    counts: dict[int, int] = field(default_factory=dict)
    weights: np.ndarray = field(default_factory=lambda: np.zeros(0))

    @property
    def sigma_lhe(self) -> float:
        """The sample's own cross section, the mean XWGTUP (pb): what main164
        normalises to at IDWTUP = -4, and the sum of XSECUP on an equal-weight file."""
        return float(self.weights.mean())


def open_text(path: Path):
    return gzip.open(path, "rt") if path.suffix == ".gz" else open(path)


def inspect(path: Path) -> LheFile:
    """Header run card, <init> cross sections, event count per IDPRUP, and the
    largest number of light final-state partons (gluons and quarks up to the
    card's maxjetflavor) in any event."""
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    run_card: dict[str, str] = {}
    xsecup: dict[int, float] = {}
    counts: dict[int, int] = {}
    in_card = in_init = in_event = False
    init_line = 0
    event_line = 0
    idprup = 0
    n_light = 0
    max_light = 0
    events: list[tuple[int, int]] = []
    weights: list[float] = []
    xwgtup = 0.0
    with open_text(path) as f:
        for line in f:
            s = line.strip()
            if s.startswith("<MGRunCard"):
                in_card = True
                continue
            if s.startswith("</MGRunCard"):
                in_card = False
                continue
            if in_card:
                m = RUN_CARD_LINE.match(line)
                if m:
                    run_card[m.group(2).lower()] = m.group(1)
                continue
            if s.startswith("<init"):
                in_init, init_line = True, 0
                continue
            if in_init:
                if s.startswith("</init") or s.startswith("<"):
                    in_init = False
                else:
                    init_line += 1
                    if init_line > 1:
                        v = s.split()
                        xsecup[int(v[3])] = float(v[0])
                continue
            if s.startswith("<event"):
                in_event, event_line, n_light = True, 0, 0
                continue
            if in_event:
                if s.startswith("</event"):
                    in_event = False
                    events.append((idprup, n_light))
                    weights.append(xwgtup)
                    continue
                if s.startswith("<") or s.startswith("#"):
                    continue
                event_line += 1
                v = s.split()
                if event_line == 1:
                    idprup, xwgtup = int(v[1]), float(v[2])
                elif len(v) >= 13 and int(v[1]) == 1:
                    pid = abs(int(v[0]))
                    if pid == 21 or pid <= int(float(run_card.get("maxjetflavor", 5))):
                        n_light += 1
    for idprup, n in events:
        counts[idprup] = counts.get(idprup, 0) + 1
        max_light = max(max_light, n)
    return LheFile(path, digest, len(events), xsecup, run_card, max_light, counts, np.array(weights))


def transform(src: Path, kind: str, work: Path) -> Path:
    if kind == "none":
        return src
    work.mkdir(parents=True, exist_ok=True)
    digest = hashlib.sha256(str(src.resolve()).encode()).hexdigest()[:12]
    dst = work / f"{src.name.split('.')[0]}-{digest}-{kind}.lhe"
    in_header = True
    in_card = False
    changed = 0
    with open_text(src) as f, open(dst, "w") as out:
        for line in f:
            if in_header and "</header>" in line:
                in_header = False
            if kind == "strip-cdata" and in_header:
                if line.strip().startswith("<MGRunCard"):
                    in_card = True
                if in_card and ("<![CDATA[" in line or "]]>" in line):
                    line = line.replace("<![CDATA[", "").replace("]]>", "")
                    changed += 1
                if line.strip().startswith("</MGRunCard"):
                    in_card = False
            if kind == "drop-scales" and not in_header and SCALES_LINE.match(line):
                changed += 1
                continue
            out.write(line)
    if changed == 0:
        sys.exit(f"--transform {kind} changes nothing in {src}")
    return dst


# ──────────────────────────────── the driver ────────────────────────────────


def build_driver() -> Path:
    BUILD.mkdir(parents=True, exist_ok=True)
    exe = BUILD / "mlm_match"
    if exe.is_file() and exe.stat().st_mtime >= SOURCE.stat().st_mtime:
        return exe
    flags = subprocess.run(["pythia8-config", "--cxxflags", "--libs"], capture_output=True,
                           text=True, check=True).stdout.split()
    cxx = os.environ.get("CXX") or shutil.which("c++") or "g++"
    cxxflags = [f for f in flags if not f.startswith(("-L", "-l", "-Wl"))]
    libs = [f for f in flags if f.startswith(("-L", "-l", "-Wl"))]
    cmd = [cxx, "-O2", *cxxflags, "-std=c++17", str(SOURCE), "-o", str(exe), *libs]
    print("building:", " ".join(cmd))
    subprocess.run(cmd, check=True)
    return exe


def mg_settings(xqcut: float, maxjetflavor: int, njetmax: int, qcut: float | None):
    values = {
        "JetMatching:qCut": repr(float(qcut if qcut is not None else 1.5 * xqcut)),
        "JetMatching:nQmatch": str(maxjetflavor),
        "JetMatching:nJetMax": str(njetmax),
    }
    rows = []
    for key, value, source in MG_SETTINGS:
        if value is None:
            value = values[key]
            if key == "JetMatching:qCut" and qcut is not None:
                source = "--qcut override (not MadGraph's value)"
        rows.append((key, value, source))
    return rows


def setmad_settings(njetmax: int):
    dropped = {"JetMatching:qCut", "JetMatching:nQmatch", "JetMatching:merge", "JetMatching:setMad"}
    rows = [(k, v, s) for k, v, s in mg_settings(0.0, 0, njetmax, None) if k not in dropped]
    rows.append(("JetMatching:setMad", "on", "Pythia reads xqcut, maxjetflavor, alpsfact, ickkw from <MGRunCard>"))
    return rows


def run_one(exe: Path, lhe: Path, settings, seed: int, work: Path, tag: str, key: str) -> dict:
    """One Pythia run. `key` (the input's and the driver's digests) goes into the
    command file as a comment, and a finished run whose command file matches is
    read back instead of re-run."""
    cmnd = work / f"{tag}.cmnd"
    tsv = work / f"{tag}.tsv"
    log = work / f"{tag}.log"
    done = work / f"{tag}.done"
    lines = [f"! {key}", f"Beams:LHEF = {lhe}"]
    lines += [f"{k} = {v}" for k, v, _ in settings]
    lines += [f"{k} = {v}" for k, v in DRIVER_SETTINGS]
    lines.append(f"Random:seed = {seed}")
    text = "\n".join(lines) + "\n"
    if done.is_file() and done.read_text() == text and tsv.is_file() and log.is_file():
        stdout = log.read_text()
    else:
        done.unlink(missing_ok=True)
        cmnd.write_text(text)
        proc = subprocess.run([str(exe), str(cmnd), str(tsv)], capture_output=True, text=True, check=False)
        log.write_text(proc.stdout + proc.stderr)
        if proc.returncode != 0 or not tsv.is_file():
            raise RuntimeError(f"{tag}: mlm_match exited {proc.returncode}; see {log}")
        stdout = proc.stdout + proc.stderr
        done.write_text(text)
    summary = {}
    for line in stdout.splitlines():
        if line.startswith("MLM "):
            key, _, rest = line[4:].partition(" ")
            summary.setdefault(key, []).append(rest)
    data = np.loadtxt(tsv, ndmin=2)
    return {"tag": tag, "seed": seed, "data": data, "summary": summary,
            "messages": pythia_messages(stdout)}


def pythia_messages(stdout: str) -> list[str]:
    out = []
    for line in stdout.splitlines():
        m = re.match(r"\s*\|\s+(\d+)\s+((?:Abort|Error|Warning) .*?)\s*\|\s*$", line)
        if m:
            out.append(f"{m.group(1)} x {m.group(2)}")
    return out


# ─────────────────────────────── statistics ─────────────────────────────────


def ratio(num: np.ndarray, den: np.ndarray) -> tuple[float, float]:
    """sum(num)/sum(den) over independent units, with its delta-method error."""
    d = den.sum()
    if d <= 0:
        return float("nan"), float("nan")
    r = num.sum() / d
    return float(r), float(math.sqrt(((num - r * den) ** 2).sum()) / d)


def per_event(runs: list[dict], weights: np.ndarray) -> dict:
    """Seed-averaged outcome of each Les Houches event of one file."""
    first = runs[0]["data"]
    for r in runs[1:]:
        if r["data"].shape[0] != first.shape[0] or not np.array_equal(r["data"][:, 1], first[:, 1]):
            raise RuntimeError(f"{r['tag']}: event list differs from {runs[0]['tag']}'s")
    acc = np.mean([r["data"][:, 4] for r in runs], axis=0)
    djr = np.stack([r["data"][:, 6:9] for r in runs])  # seeds x events x 3
    accepted = np.stack([r["data"][:, 4] for r in runs])
    return {"idprup": first[:, 1].astype(int), "accepted": acc, "weight": weights, "djr": djr,
            "accepted_by_seed": accepted}


def djr_log(djr: np.ndarray) -> np.ndarray:
    with np.errstate(divide="ignore", invalid="ignore"):
        return np.where(djr > 0, np.log10(np.where(djr > 0, djr, 1.0)), -np.inf)


def bin_fractions(ev: dict, k: int, edges: np.ndarray) -> np.ndarray:
    """events x bins: each event's seed-averaged weight in each bin of d_k
    (accepted only). Bin 0 is the underflow (and a missing scale), the last the
    overflow."""
    vals = djr_log(ev["djr"][:, :, k])  # seeds x events
    idx = np.digitize(vals, edges)  # 0 = underflow, len(edges) = overflow
    nb = len(edges) + 1
    out = np.zeros((vals.shape[1], nb))
    for s in range(vals.shape[0]):
        out[np.arange(vals.shape[1]), idx[s]] += ev["accepted_by_seed"][s]
    return out * ev["weight"][:, None] / vals.shape[0]


def merge_bins(counts_a: np.ndarray, counts_b: np.ndarray, min_count: float) -> list[list[int]]:
    groups, cur, ca, cb = [], [], 0.0, 0.0
    for i in range(len(counts_a)):
        cur.append(i)
        ca += counts_a[i]
        cb += counts_b[i]
        if ca >= min_count and cb >= min_count:
            groups.append(cur)
            cur, ca, cb = [], 0.0, 0.0
    if cur:
        if groups:
            groups[-1].extend(cur)
        else:
            groups.append(cur)
    return groups


def group_edges(edges: np.ndarray, groups: list[list[int]]) -> list[list[float | None]]:
    full = [None, *edges.tolist(), None]
    return [[full[g[0]], full[g[-1] + 1]] for g in groups]


# ─────────────────────────────── the analysis ───────────────────────────────


def side_summary(files: list[LheFile], events: list[dict], runs: list[list[dict]]) -> dict:
    codes = sorted({c for f in files for c in f.counts})
    idp = np.concatenate([e["idprup"] for e in events])
    acc = np.concatenate([e["accepted"] * e["weight"] for e in events])
    wgt = np.concatenate([e["weight"] for e in events])
    out: dict = {"n_files": len(files), "n_events": int(len(acc)), "acceptance": {}, "per_file": []}
    for c in codes:
        sel = idp == c
        a, da = ratio(acc[sel], wgt[sel])
        out["acceptance"][str(c)] = {"n": int(sel.sum()), "value": a, "error": da}
    a, da = ratio(acc, wgt)
    out["acceptance"]["all"] = {"n": int(len(acc)), "value": a, "error": da}

    # merged sigma, main164's normalisation, averaged over files of equal weight
    merged, merged_err, norm, norm_err = [], [], [], []
    for f, e, rr in zip(files, events, runs):
        frac, dfrac = ratio(e["accepted"] * e["weight"], e["weight"])
        row = {
            "path": str(f.path), "sha256": f.sha256, "n_events": f.n_events,
            "sigma_lhe_pb": f.sigma_lhe, "sigma_xsecup_pb": sum(f.xsecup.values()),
            "xsecup_pb": {str(k): v for k, v in sorted(f.xsecup.items())},
            "weights": {"distinct": int(len(np.unique(f.weights))), "max_over_mode": float(
                f.weights.max() / np.median(f.weights)), "share_above_mode": float(
                (f.weights - np.median(f.weights)).clip(min=0).sum() / f.weights.sum())},
            "events_per_idprup": {str(k): v for k, v in sorted(f.counts.items())},
            "accepted_fraction": frac, "accepted_fraction_error": dfrac,
            "merged_sigma_pb": f.sigma_lhe * frac, "merged_sigma_error_pb": f.sigma_lhe * dfrac,
            "acceptance": {}, "per_seed": [],
        }
        for c in codes:
            sel = e["idprup"] == c
            v, dv = ratio((e["accepted"] * e["weight"])[sel], e["weight"][sel])
            row["acceptance"][str(c)] = {"value": v, "error": dv}
        for r in rr:
            d = r["data"]
            per = {"seed": r["seed"], "records": int(d.shape[0]),
                   "matching_vetoed": int((d[:, 3] == 1).sum()),
                   "process_vetoed": int((d[:, 2] == 1).sum()),
                   "accepted": int(d[:, 4].sum()),
                   "failed_next": int(r["summary"].get("records", ["0 failed_next 0"])[0].split()[2]),
                   "settings_read_back": r["summary"].get("settings", [""])[0],
                   "mgruncard_bytes": r["summary"].get("header", [""])[0],
                   "acceptance": {}, "messages": r["messages"]}
            for c in codes:
                sel = d[:, 1] == c
                per["acceptance"][str(c)] = (float((d[sel, 4] * f.weights[sel]).sum() / f.weights[sel].sum())
                                             if sel.any() else None)
            per["merged_sigma_pb"] = float((d[:, 4] * f.weights).mean())
            row["per_seed"].append(per)
        out["per_file"].append(row)
        merged.append(row["merged_sigma_pb"])
        merged_err.append(row["merged_sigma_error_pb"])
        norm.append(frac)
        norm_err.append(dfrac)

    def mean_of(vals, errs):
        vals, errs = np.array(vals), np.array(errs)
        m = float(vals.mean())
        e = float(math.sqrt((errs ** 2).sum()) / len(vals))
        chi2 = float((((vals - m) / errs) ** 2).sum() / (len(vals) - 1)) if len(vals) > 1 else None
        return {"value": m, "error": e, "spread": float(vals.std(ddof=1)) if len(vals) > 1 else None,
                "chi2_per_dof_of_files": chi2}

    out["merged_sigma_pb"] = mean_of(merged, merged_err)
    out["merged_over_sigma_lhe"] = mean_of(norm, norm_err)
    out["sigma_lhe_pb"] = {"value": float(np.mean([f.sigma_lhe for f in files])),
                           "per_file": [f.sigma_lhe for f in files]}
    # file-to-file spread of the per-@N acceptance
    for c in [*map(str, codes)]:
        vals = np.array([r["acceptance"][c]["value"] for r in out["per_file"]])
        errs = np.array([r["acceptance"][c]["error"] for r in out["per_file"]])
        m = out["acceptance"][c]["value"]
        out["acceptance"][c]["chi2_per_dof_of_files"] = (
            float((((vals - m) / errs) ** 2).sum() / (len(vals) - 1)) if len(vals) > 1 else None)
    return out


def compare(a: dict, b: dict, key: str) -> dict:
    d = b[key]["value"] - a[key]["value"]
    e = math.hypot(a[key]["error"], b[key]["error"])
    return {"a": a[key]["value"], "b": b[key]["value"], "b_over_a": b[key]["value"] / a[key]["value"],
            "difference": d, "error": e, "pull": d / e if e > 0 else None}


def bootstrap_shape_errors(g: np.ndarray, acc: np.ndarray, n: int, rng) -> np.ndarray:
    """Standard deviation of each normalised bin over `n` resamplings of the
    side's events: a check of the delta-method errors that assumes nothing about
    the weight distribution."""
    out = np.empty((n, g.shape[1]))
    for i in range(n):
        idx = rng.integers(0, len(acc), len(acc))
        out[i] = g[idx].sum(0) / acc[idx].sum()
    return out.std(0, ddof=1)


def histograms(ev_a: list[dict], ev_b: list[dict], min_count: float, n_boot: int = 0) -> dict:
    edges = np.round(np.arange(HIST_LO, HIST_HI + HIST_STEP / 2, HIST_STEP), 10)
    out = {}
    for k, name in enumerate(DJR_NAMES):
        fa = np.concatenate([bin_fractions(e, k, edges) for e in ev_a])
        fb = np.concatenate([bin_fractions(e, k, edges) for e in ev_b])
        acc_a = np.concatenate([e["accepted"] * e["weight"] for e in ev_a])
        acc_b = np.concatenate([e["accepted"] * e["weight"] for e in ev_b])
        # events-equivalent entries (weighted sums over the side's mean weight) decide the merging
        wa = np.concatenate([e["weight"] for e in ev_a]).mean()
        wb = np.concatenate([e["weight"] for e in ev_b]).mean()
        groups = merge_bins(fa.sum(0) / wa, fb.sum(0) / wb, min_count)
        ga = np.stack([fa[:, g].sum(1) for g in groups], axis=1)
        gb = np.stack([fb[:, g].sum(1) for g in groups], axis=1)
        ha, hb, chi2 = [], [], 0.0
        for j in range(len(groups)):
            va, ea = ratio(ga[:, j], acc_a)
            vb, eb = ratio(gb[:, j], acc_b)
            ha.append([va, ea])
            hb.append([vb, eb])
            chi2 += (va - vb) ** 2 / (ea ** 2 + eb ** 2)
        dof = len(groups) - 1
        out[name] = {
            "bins": group_edges(edges, groups),
            "entries_a": (ga.sum(0) / wa).tolist(), "entries_b": (gb.sum(0) / wb).tolist(),
            "shape_a": ha, "shape_b": hb,
            "chi2": chi2, "dof": dof, "chi2_per_dof": chi2 / dof if dof > 0 else None,
            "p_value": chi2_sf(chi2, dof),
            "bootstrap": None,
        }
        if n_boot > 0:
            rng = np.random.default_rng(20261229 + k)
            ba = bootstrap_shape_errors(ga, acc_a, n_boot, rng)
            bb = bootstrap_shape_errors(gb, acc_b, n_boot, rng)
            va, vb = np.array(ha)[:, 0], np.array(hb)[:, 0]
            chi2b = float((((va - vb) ** 2) / (ba ** 2 + bb ** 2)).sum())
            out[name]["bootstrap"] = {
                "resamplings": n_boot, "error_a": ba.tolist(), "error_b": bb.tolist(),
                "chi2": chi2b, "p_value": chi2_sf(chi2b, dof),
                "error_ratio_to_delta_a": (ba / np.array(ha)[:, 1]).tolist(),
                "error_ratio_to_delta_b": (bb / np.array(hb)[:, 1]).tolist(),
            }
    return out


def chi2_sf(x: float, k: int) -> float | None:
    """Upper tail of the chi2 distribution (regularised incomplete gamma), so the
    driver needs nothing beyond numpy."""
    if k <= 0:
        return None
    if x <= 0:
        return 1.0
    a, xx = k / 2.0, x / 2.0
    if xx < a + 1:  # series for P, then Q = 1 - P
        term = s = 1.0 / a
        n = a
        for _ in range(1000):
            n += 1
            term *= xx / n
            s += term
            if abs(term) < abs(s) * 1e-15:
                break
        return float(max(0.0, 1.0 - s * math.exp(-xx + a * math.log(xx) - math.lgamma(a))))
    b = xx + 1 - a  # continued fraction for Q
    c, d = 1.0 / 1e-300, 1.0 / b
    h = d
    for i in range(1, 1000):
        an = -i * (i - a)
        b += 2
        d = an * d + b
        d = 1e-300 if abs(d) < 1e-300 else d
        c = b + an / c
        c = 1e-300 if abs(c) < 1e-300 else c
        d = 1.0 / d
        h *= d * c
        if abs(d * c - 1) < 1e-15:
            break
    return float(math.exp(-xx + a * math.log(xx) - math.lgamma(a)) * h)


# ────────────────────────────────── main ────────────────────────────────────


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--a", nargs="+", type=Path, default=DEFAULT_A)
    ap.add_argument("--b", nargs="+", type=Path, default=None)
    ap.add_argument("--label-a", default="madevent")
    ap.add_argument("--label-b", default="vibegraph")
    ap.add_argument("--transform-a", default="none", choices=["none", "strip-cdata", "drop-scales"])
    ap.add_argument("--transform-b", default="none", choices=["none", "strip-cdata", "drop-scales"])
    ap.add_argument("--config", default="mg", choices=["mg", "setmad"])
    ap.add_argument("--seeds", nargs="+", type=int, default=DEFAULT_SEEDS)
    ap.add_argument("--qcut", type=float, default=None, help="override MadGraph's 1.5*xqcut")
    ap.add_argument("--njetmax", type=int, default=None,
                    help="max_n_matched_jets; default: the largest number of light partons in any event")
    ap.add_argument("--min-count", type=float, default=100.0,
                    help="accepted events per jet-rate bin required on each side")
    ap.add_argument("--unit-weights", action="store_true",
                    help="diagnostic: count every event once, ignoring XWGTUP (not what Pythia does)")
    ap.add_argument("--bootstrap", type=int, default=200,
                    help="resamplings of each side's events that re-derive the jet-rate bin errors")
    ap.add_argument("--jobs", type=int, default=os.cpu_count() or 1)
    ap.add_argument("--work", type=Path, default=BUILD / "work")
    ap.add_argument("--out", type=Path, default=DEFAULT_OUT)
    args = ap.parse_args()

    if args.b is None:
        args.b = sorted(DEFAULT_B_DIR.glob("*.lhe"))
        if not args.b:
            sys.exit(f"no vibegraph samples in {DEFAULT_B_DIR}: run validation/pythia/generate_mlm_samples.sh")

    exe = build_driver()
    driver_sha = hashlib.sha256(exe.read_bytes()).hexdigest()
    args.work.mkdir(parents=True, exist_ok=True)

    sides = {"a": (args.label_a, args.a, args.transform_a), "b": (args.label_b, args.b, args.transform_b)}
    info: dict[str, list[LheFile]] = {}
    shower_inputs: dict[str, list[Path]] = {}
    for s, (label, paths, kind) in sides.items():
        info[s] = [inspect(p) for p in paths]
        if args.unit_weights:
            for f in info[s]:
                f.weights = np.full(len(f.weights), f.weights.mean())
        shower_inputs[s] = [transform(p, kind, args.work) for p in paths]

    cards = [f.run_card for fs in info.values() for f in fs]
    for key in ("xqcut", "maxjetflavor", "ickkw"):
        vals = {c.get(key) for c in cards}
        if len({float(v) for v in vals if v is not None}) != 1 or None in vals:
            sys.exit(f"run card '{key}' differs or is missing across the inputs: {vals}")
    xqcut = float(cards[0]["xqcut"])
    maxjetflavor = int(float(cards[0]["maxjetflavor"]))
    if int(float(cards[0]["ickkw"])) != 1:
        sys.exit("the inputs are not matched (ickkw != 1)")
    observed = max(f.max_light_partons for fs in info.values() for f in fs)
    njetmax = args.njetmax if args.njetmax is not None else observed
    if observed > njetmax:
        sys.exit(f"an event carries {observed} light partons, more than nJetMax = {njetmax}")

    settings = mg_settings(xqcut, maxjetflavor, njetmax, args.qcut) if args.config == "mg" \
        else setmad_settings(njetmax)

    jobs = []
    for s in ("a", "b"):
        for i, lhe in enumerate(shower_inputs[s]):
            for seed in args.seeds:
                jobs.append((s, i, seed, f"{s}{i}-{lhe.stem}-{args.config}-{seed}"))
    print(f"{len(jobs)} Pythia runs ({args.jobs} at a time), config {args.config}, "
          f"qCut {dict((k, v) for k, v, _ in settings).get('JetMatching:qCut', 'from <MGRunCard>')}")
    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        futures = {j: pool.submit(run_one, exe, shower_inputs[j[0]][j[1]], settings, j[2], args.work, j[3],
                                  f"input {info[j[0]][j[1]].sha256} driver {driver_sha}")
                   for j in jobs}
        results = {j: f.result() for j, f in futures.items()}

    events, runs = {}, {}
    for s in ("a", "b"):
        runs[s] = [[results[j] for j in jobs if j[0] == s and j[1] == i] for i in range(len(info[s]))]
        for f, rr in zip(info[s], runs[s]):
            for r in rr:
                n = r["data"].shape[0] if r["data"].shape[1:] == (9,) else 0
                if n != f.n_events:
                    sys.exit(f"{f.path}: the matching recorded {n} of its {f.n_events} events "
                             f"(Pythia read back: {r['summary'].get('settings', ['?'])[0]}); see {args.work}")
        events[s] = [per_event(rr, f.weights) for rr, f in zip(runs[s], info[s])]
    summary = {s: side_summary(info[s], events[s], runs[s]) for s in ("a", "b")}

    comparison = {"acceptance": {}}
    for c in summary["a"]["acceptance"]:
        if c in summary["b"]["acceptance"]:
            comparison["acceptance"][c] = compare(summary["a"]["acceptance"], summary["b"]["acceptance"], c)
    comparison["merged_sigma_pb"] = compare(summary["a"], summary["b"], "merged_sigma_pb")
    comparison["merged_over_sigma_lhe"] = compare(summary["a"], summary["b"], "merged_over_sigma_lhe")
    comparison["jet_rates"] = histograms(events["a"], events["b"], args.min_count, args.bootstrap)

    version = results[jobs[0]]["summary"].get("version", ["?"])[0]
    row = {
        "row": "mlm-pythia",
        "task": "validate-mlm-pythia",
        "pythia_version": version,
        "config": args.config,
        "settings": [{"key": k, "value": v, "source": src} for k, v, src in settings],
        "driver_settings": [{"key": k, "value": v} for k, v in DRIVER_SETTINGS],
        "seeds": args.seeds,
        "njetmax": {"value": njetmax, "max_light_partons_seen": observed},
        "min_count_per_bin": args.min_count,
        "unit_weights": args.unit_weights,
        "sides": {
            s: {"label": sides[s][0], "transform": sides[s][2], **summary[s]} for s in ("a", "b")
        },
        "comparison": comparison,
        "mode": "info",
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(row, indent=1) + "\n")

    la, lb = args.label_a, args.label_b
    print(f"Pythia {version}; {len(args.seeds)} seeds; {la}: {summary['a']['n_events']} events in "
          f"{len(info['a'])} file(s) [{args.transform_a}], {lb}: {summary['b']['n_events']} in "
          f"{len(info['b'])} [{args.transform_b}]")
    for c, v in comparison["acceptance"].items():
        print(f"  acceptance @{c}: {la} {v['a']:.4f} +- {summary['a']['acceptance'][c]['error']:.4f}  "
              f"{lb} {v['b']:.4f} +- {summary['b']['acceptance'][c]['error']:.4f}  pull {v['pull']:+.2f}")
    for key, unit in (("merged_sigma_pb", "pb"), ("merged_over_sigma_lhe", "")):
        v = comparison[key]
        print(f"  {key}: {la} {v['a']:.4f} +- {summary['a'][key]['error']:.4f} {unit}  "
              f"{lb} {v['b']:.4f} +- {summary['b'][key]['error']:.4f}  ratio {v['b_over_a']:.4f}  "
              f"pull {v['pull']:+.2f}")
    for name, h in comparison["jet_rates"].items():
        boot = h["bootstrap"]
        extra = f"; bootstrap errors: chi2 {boot['chi2']:.1f} (p = {boot['p_value']:.3g})" if boot else ""
        print(f"  {name}: chi2 {h['chi2']:.1f} / {h['dof']} (p = {h['p_value']:.3g}), {len(h['bins'])} bins{extra}")
    print(f"wrote {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
