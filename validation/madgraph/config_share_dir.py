"""Read one MadEvent run's channel bookkeeping, or assemble the oracle file.

`python3 config_share_dir.py PROCDIR "VALUE ERR WALL"` prints, for the finished
run in PROCDIR, the lines `gen_config_share.sh` banks before the per-channel
results:

    TOTAL <value> <err> <wall_s>
    SDE <sde_strategy>
    PROC <P dir> <ordering> <subprocess>      one per processes.dat entry
    CONFIG <P dir> <iconfig> <signature JSON> one per configs.inc channel
    SYMFACT <P dir> <iconfig> <factor>        one per symfact.dat line

A channel's signature is its propagators, each as
`[legs, class, kind]`: the external legs (1-based) on the side of the line that
does not hold leg 1, the propagating particle's class (`a`, `z`, `w`, `h`,
`g`, `q` for any quark, `l` for any lepton) and `s` or `t`. The class rather
than the PDG code, because a grouped directory lists one t-channel code for
every subprocess it holds. The forest's closing t-channel entry, which joins
the chain to leg 2 and is not a propagator, is left out: its far side is one
external leg.

`python3 config_share_dir.py --write RECORDS OUT_JSON VERSION_FILE` reads the
banked files RECORDS lists (`name|proc|seed|path` per line) and writes the
oracle file, keeping the committed rows not listed.
"""

import json
import os
import re
import sys


def particle_class(pdg):
    a = abs(pdg)
    if a == 22:
        return "a"
    if a == 23:
        return "z"
    if a == 24:
        return "w"
    if a == 25:
        return "h"
    if a == 21:
        return "g"
    if 1 <= a <= 6:
        return "q"
    if 11 <= a <= 16:
        return "l"
    raise ValueError("no class for PDG %d" % pdg)


def read_configs(path, nexternal):
    """{iconfig: signature} from one configs.inc."""
    text = open(path).read()
    forest = {}
    sprop = {}
    tprid = {}
    for m in re.finditer(r"IFOREST\(I,-(\d+),(\d+)\),I=1,2\)/(-?\d+),(-?\d+)/", text):
        line, cfg, a, b = (int(x) for x in m.groups())
        forest.setdefault(cfg, {})[line] = (a, b)
    for m in re.finditer(r"SPROP\(I,-(\d+),(\d+)\),I=1,\d+\)/([-\d,]+)/", text):
        line, cfg = int(m.group(1)), int(m.group(2))
        sprop[(cfg, line)] = int(m.group(3).split(",")[0])
    for m in re.finditer(r"TPRID\(-(\d+),(\d+)\)/(-?\d+)/", text):
        line, cfg, pdg = (int(x) for x in m.groups())
        tprid[(cfg, line)] = pdg
    every = set(range(1, nexternal + 1))
    out = {}
    for cfg, lines in forest.items():
        legs = {}
        sig = []
        for line in sorted(lines):
            a, b = lines[line]
            side = set()
            for d in (a, b):
                side |= {d} if d > 0 else legs[-d]
            legs[line] = side
            far = every - side if 1 in side else side
            if len(far) < 2 or len(far) > nexternal - 2:
                continue
            s = sprop.get((cfg, line), 0)
            t = tprid.get((cfg, line), 0)
            kind, pdg = ("s", s) if s != 0 else ("t", t)
            sig.append([sorted(far), particle_class(pdg), kind])
        out[cfg] = sorted(sig)
    return out


def describe(procdir, result):
    value, err, wall = result.split()
    print("TOTAL", value, err, wall)
    card = open(os.path.join(procdir, "Cards", "run_card.dat")).read()
    m = re.search(r"^\s*(\S+)\s*=\s*sde_strategy\b", card, re.M)
    print("SDE", m.group(1) if m else "?")
    sub = os.path.join(procdir, "SubProcesses")
    for p in sorted(os.listdir(sub)):
        pdir = os.path.join(sub, p)
        if not (p.startswith("P") and os.path.isdir(pdir)):
            continue
        for raw in open(os.path.join(pdir, "processes.dat")):
            words = raw.split(None, 1)
            if len(words) < 2:
                continue
            ordering = "mirror" if words[0] == "mirror" else "direct"
            for proc in words[1].strip().split(","):
                if proc.strip() != "none":
                    print("PROC", p, ordering, proc.strip())
        n = int(re.search(r"NEXTERNAL=(\d+)", open(os.path.join(pdir, "nexternal.inc")).read()).group(1))
        for cfg, sig in sorted(read_configs(os.path.join(pdir, "configs.inc"), n).items()):
            print("CONFIG", p, cfg, json.dumps(sig, separators=(",", ":")))
        symfact = os.path.join(pdir, "symfact.dat")
        if not os.path.exists(symfact):
            symfact = os.path.join(pdir, "symfact_orig.dat")
        for raw in open(symfact):
            words = raw.split()
            if len(words) >= 2:
                print("SYMFACT", p, words[0], words[1])


def fortran_float(s):
    return float(s.replace("D", "E").replace("d", "e"))


def read_banked(path):
    run = {"channels": {}}
    dirs = {}
    for raw in open(path):
        w = raw.split()
        if not w:
            continue
        tag = w[0]
        if tag == "TOTAL":
            run["sigma_pb"], run["err_pb"], run["wall_s"] = (
                fortran_float(w[1]), fortran_float(w[2]), int(w[3]))
        elif tag == "SDE":
            run["sde_strategy"] = int(w[1])
        elif tag == "PROC":
            dirs.setdefault(w[1], {"subprocesses": [], "channels": {}, "symfact": {}})
            dirs[w[1]]["subprocesses"].append({"ordering": w[2], "process": " ".join(w[3:])})
        elif tag == "CONFIG":
            dirs[w[1]]["channels"][w[2]] = json.loads(" ".join(w[3:]))
        elif tag == "SYMFACT":
            dirs[w[1]]["symfact"][w[2]] = int(w[3])
        else:
            p, g, value, err = w
            run["channels"].setdefault(p, {})[g] = [fortran_float(value), fortran_float(err)]
    return run, dirs


def write(records_path, out_path, version_path):
    old = json.load(open(out_path))["rows"] if os.path.exists(out_path) else {}
    rows = {}
    for raw in open(records_path):
        name, proc, seed, path = raw.rstrip("\n").split("|")
        run, dirs = read_banked(path)
        row = rows.setdefault(name, {
            "process": proc,
            "run_card": "config_share_%s_run_card.dat" % name,
            "sde_strategy": run["sde_strategy"],
            "directories": dirs,
            "runs": [],
        })
        if row["directories"] != dirs:
            sys.exit("%s seed %s: a fresh directory's channels differ from the first's" % (name, seed))
        if row["sde_strategy"] != run["sde_strategy"]:
            sys.exit("%s seed %s: sde_strategy differs between seeds" % (name, seed))
        row["runs"].append({
            "iseed": int(seed),
            "sigma_pb": run["sigma_pb"],
            "err_pb": run["err_pb"],
            "wall_s": run["wall_s"],
            "channels": run["channels"],
        })
    old.update(rows)
    version = open(version_path).read().split("version")[1].split("=")[1].split()[0]
    doc = {
        "description": (
            "MadEvent per-channel cross sections (SubProcesses/P*/G*/results.dat) per seed, "
            "one freshly generated process directory per seed, with each directory's "
            "subprocesses and channel propagators; written by gen_config_share.sh."
        ),
        "madgraph_version": version,
        "rows": dict(sorted(old.items())),
    }
    with open(out_path, "w") as f:
        json.dump(doc, f, indent=1, sort_keys=False)
        f.write("\n")


if __name__ == "__main__":
    if sys.argv[1] == "--write":
        write(*sys.argv[2:5])
    else:
        describe(sys.argv[1], sys.argv[2])
