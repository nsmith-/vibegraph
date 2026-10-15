#!/usr/bin/env python3
"""Census of MadEvent runs exposed to the permuted first `setclscales` call.

Grouped MadEvent's `DSIGPROC` hands its first `update_scale_coupling` call the
sampled point `PP`, while the matrix element and `REWGT` read
`P1 = SWITCHMOM(PP, PERMS(MAPCONFIG(ICONFIG)))` (`super_auto_dsig_group_v4.inc`,
the scalar path at `:842`, the vector path at `:312`). The first call reaches
the clustering in `setclscales` (`Template/LO/SubProcesses/reweight.f`) unless
the early return at `ickkw <= 0`, `xqcut <= 0` and both `q2fact` and `scale`
already positive fires. With `dynamical_scale_choice != -1` the scale routines
set those from final-state sums or `sqrt(s)`, which a final-state permutation
leaves unchanged; with fixed scales they are set at start-up. So a run's first
call clusters exactly when

    ickkw > 0, or xqcut > 0, or
    dynamical_scale_choice == -1 and not (fixed_ren_scale and both fixed_fac_scale)

A clustered first call moves a cross section only through the PDF scales
(`lpp != 0`), through mu_R (an alpha_s-dependent coupling in the matrix
elements), or through a rejection (xqcut, xmtc, or the 2 GeV factorisation
floor, which also needs `lpp != 0`). A run with fixed beams and no alpha_s
coupling cannot move, so its `clusters` flag is not exposure. The bias also
needs the output to be grouped and some channel's permutation in `symperms.inc`
to be other than the identity. The `IMIRROR = 2` flip alone is a rotation by pi about the x axis, under which the
clustering's kT measures are invariant, so it is not counted.

Usage:
  census_permuted_first_call.py runs [--output DIR]
      One line per banked run: MadGraph version, grouping, the card switches
      above and whether the first call clusters. The alpha_s dependence of the
      matrix elements is not read; judge it from the `lpp` column and the
      model. Exits non-zero if no banner is found or a banner has no run card.
  census_permuted_first_call.py perms PROCESS_DIR [PROCESS_DIR ...]
      For process directories generated with `output` (launched or not),
      every subprocess directory's configs whose `PERMS(MAPCONFIG(iconfig))`
      in symperms.inc is not the identity, and the configs symfact maps onto
      others. The banked bundle carries neither file, so the directories are
      regenerated from the row's own MG5 script with the pinned MadGraph.
"""

import argparse
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from extract_sigma import parse_run_card_scalars  # noqa: E402


def _section(text, tag):
    m = re.search(rf"<{tag}>(.*?)</{tag}>", text, re.DOTALL)
    return m.group(1) if m else ""


def _bool(v):
    return str(v).strip().lower() in ("t", "true", ".true.")


def _fixed(rc):
    """fixed_ren_scale, fixed_fac_scale1, fixed_fac_scale2; a card that sets
    only `fixed_fac_scale` sets both beams (banner.py's post_set_fixed_fac_scale)."""
    both = rc.get("fixed_fac_scale", "F")
    return tuple(
        _bool(rc.get(k, d))
        for k, d in (("fixed_ren_scale", "F"), ("fixed_fac_scale1", both), ("fixed_fac_scale2", both))
    )


def first_call_clusters(rc):
    ickkw = int(rc.get("ickkw", "0"))
    xqcut = float(rc.get("xqcut", "0"))
    dyn = int(rc.get("dynamical_scale_choice", "-1"))
    return ickkw > 0 or xqcut > 0 or (dyn == -1 and not all(_fixed(rc)))


def runs(output):
    rows = []
    for banner in sorted(output.glob("*/Events/*/*banner.txt")):
        text = banner.read_text(errors="replace")
        rc = parse_run_card_scalars(_section(text, "MGRunCard"))
        if not rc:
            sys.exit(f"{banner}: no <MGRunCard> parameters")
        proc = _section(text, "MG5ProcCard")
        version = _section(text, "MGVersion").strip().lstrip("#") or "?"
        dev = "UNKNOWN DEVELOPMENT VERSION" in proc
        group = re.search(r"^set group_subprocesses (\S+)", proc, re.MULTILINE)
        procdir = banner.parents[2]
        pdirs = sorted(p.name for p in (procdir / "SubProcesses").glob("P*"))
        gens = [
            line.strip()
            for line in proc.splitlines()
            if re.match(r"^\s*(generate|add process)\b", line)
        ]
        ninit = 1 if gens and re.match(r"^\s*(generate|add process)\s+\S+\s*>", gens[0]) else 2
        rows.append(
            dict(
                run=f"{procdir.name}/{banner.parent.name}",
                version=version + ("-dev" if dev else ""),
                grouped=(group is None or group.group(1) in ("Auto", "True"))
                and not (ninit == 1 and len(pdirs) > 1),
                lpp=f"{rc.get('lpp1', '?')},{rc.get('lpp2', '?')}",
                dyn=rc.get("dynamical_scale_choice", "-1"),
                fixed="".join("T" if f else "F" for f in _fixed(rc)),
                ickkw=rc.get("ickkw", "0"),
                xqcut=rc.get("xqcut", "0"),
                vector_size=rc.get("vector_size", "1"),
                clusters=first_call_clusters(rc),
                gen="; ".join(gens),
            )
        )
    if not rows:
        sys.exit(f"no banners under {output}/*/Events/*/*banner.txt")
    cols = ["run", "version", "grouped", "lpp", "dyn", "fixed", "ickkw", "xqcut",
            "vector_size", "clusters", "gen"]
    print("\t".join(cols))
    for r in rows:
        print("\t".join(str(r[c]) for c in cols))


def _symperms(path):
    perms = {}
    for line in path.read_text().splitlines():
        m = re.match(r"\s*DATA \(PERMS\(I,(\d+)\),I=1,\w+\)\s*/(.*)/", line, re.IGNORECASE)
        if m:
            perms[int(m.group(1))] = [int(x) for x in m.group(2).split(",")]
    if not perms:
        sys.exit(f"{path}: no DATA PERMS lines")
    return perms


def _mapconfig(path):
    mc = {}
    for line in path.read_text().splitlines():
        m = re.match(r"\s*data mapconfig\((\d+)\)\s*/\s*(\d+)\s*/", line, re.IGNORECASE)
        if m and int(m.group(1)) > 0:
            mc[int(m.group(1))] = int(m.group(2))
    if not mc:
        sys.exit(f"{path}: no data mapconfig lines")
    return mc


def _symfact(pdir):
    out = {}
    f = pdir / "symfact.dat"
    if not f.exists():
        f = pdir / "symfact_orig.dat"
    if f.exists():
        for line in f.read_text().split("\n"):
            parts = line.split()
            if len(parts) >= 2:
                out[parts[0]] = int(float(parts[1]))
    return out


def perms(dirs):
    print("procdir\tsubproc\tnconfigs\tnon-identity configs (iconfig:mapconfig:perm)\tsymfact<0 (mapped configs)")
    for d in map(pathlib.Path, dirs):
        pdirs = sorted((d / "SubProcesses").glob("P*"))
        if not pdirs:
            sys.exit(f"{d}: no SubProcesses/P* directory")
        for pdir in pdirs:
            for name in ("symperms.inc", "configs.inc"):
                if not (pdir / name).is_file():
                    sys.exit(f"{pdir}: no {name}; generate the directory with `output` first")
            pm = _symperms(pdir / "symperms.inc")
            mc = _mapconfig(pdir / "configs.inc")
            nonid = []
            for iconf, imap in sorted(mc.items()):
                if imap not in pm:
                    sys.exit(f"{pdir}: configs.inc maps config {iconf} to {imap}, absent from symperms.inc")
                p = pm[imap]
                if p != list(range(1, len(p) + 1)):
                    nonid.append(f"{iconf}:{imap}:({','.join(map(str, p))})")
            sf = _symfact(pdir)
            neg = [f"{k}->{-v}" for k, v in sf.items() if v < 0]
            print(f"{d.name}\t{pdir.name}\t{len(mc)}\t{' '.join(nonid) or 'none'}\t{' '.join(neg) or '-'}")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("runs")
    r.add_argument("--output", type=pathlib.Path, default=HERE / "output")
    p = sub.add_parser("perms")
    p.add_argument("dirs", nargs="+")
    a = ap.parse_args()
    if a.cmd == "runs":
        runs(a.output)
    else:
        perms(a.dirs)


if __name__ == "__main__":
    sys.exit(main())
