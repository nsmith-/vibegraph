#!/usr/bin/env python3
"""Check that MadGraph's fused couplings stay fused at a moved parameter point.

At `import model`, MadGraph's restriction (`models/import_ufo.py`,
`RestrictModel.restrict_model`) fuses the couplings that are equal, or opposite,
at the restriction's parameter values (`detect_identical_couplings`,
`merge_iden_couplings`): every vertex then reads the group's first coupling,
whatever the param card says later. It also removes the couplings that vanish
there, and fixes the parameters that are exactly zero or one
(`detect_special_parameters`). A param card that moves a parameter so that a
fused pair splits, a removed coupling revives, or a fixed parameter moves is
evaluated by MadGraph at a point vibegraph does not evaluate (vibegraph
evaluates every coupling of the card on its own), and a sigma comparison at that
card would show MadGraph's merge as a disagreement.

This runs MadGraph's own restriction on a temporary copy of the UFO (MadGraph
writes a pickle into the model directory it imports), records what it fuses,
removes and fixes, then evaluates the unrestricted model with MadGraph's own
`set_parameters_and_couplings` at each point: the restrict card with the
point's entries changed. Nothing is generated and MadGraph does not run.

Points checked by default: the two moved-parameter sigma rows
(`dy13_mmll_60_120_mz95`, MZ = 95; `ee_to_tth_ymt250`, ymt = 250), a generic
point that moves every non-zero external of the SM at once, and a control that
revives a zero-locked Wolfenstein parameter, at which the CKM group must split
(otherwise the check could not see a split at all). `--point NAME:BLOCK CODE
VALUE[,BLOCK CODE VALUE...]` adds one.

Exit status 0 when no fused pair splits, no removed coupling revives and no
fixed parameter moves at any point except the control, and the control does
show a split; 1 otherwise.

usage: python3 -I validation/madgraph/check_coupling_fusion.py
           [--mg-root DIR] [--model DIR] [--restrict FILE] [--point SPEC]...
"""
import argparse
import logging
import os
import re
import shutil
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))

# A relative difference above this is a split: MadGraph fuses at ten
# significant digits (`limit_to_6_digit`), and a coupling that moves with the
# card moves by far more.
SPLIT = 1e-10
ZERO = 1e-13

DEFAULT_POINTS = [
    ("dy13_mmll_60_120_mz95: MZ = 95", {("mass", 23): 95.0}, False),
    ("ee_to_tth_ymt250: ymt = 250", {("yukawa", 6): 250.0}, False),
    (
        "generic: every non-zero external moved",
        {
            ("sminputs", 1): 128.1, ("sminputs", 2): 1.21e-05, ("sminputs", 3): 0.125,
            ("mass", 5): 4.9, ("mass", 6): 171.3, ("mass", 15): 1.81, ("mass", 23): 90.4,
            ("mass", 25): 127.7, ("decay", 6): 1.37, ("decay", 23): 2.52,
            ("decay", 24): 2.11, ("decay", 25): 4.4e-03, ("yukawa", 5): 4.31,
            ("yukawa", 6): 166.9, ("yukawa", 15): 1.69,
        },
        False,
    ),
    ("control: lamWS = 0.2265 (vibegraph refuses this card)", {("wolfenstein", 1): 0.2265}, True),
]


def parse_point(spec):
    name, _, entries = spec.partition(":")
    moves = {}
    for entry in entries.split(","):
        words = entry.split()
        if len(words) != 3:
            sys.exit("--point %r: each entry is BLOCK CODE VALUE" % spec)
        moves[(words[0].lower(), int(words[1]))] = float(words[2])
    return (name.strip(), moves, False)


def card_with(restrict, moves, directory):
    """The restrict card with the (block, code) entries of `moves` replaced."""
    out, block, found = [], None, set()
    for line in open(restrict).read().splitlines():
        m = re.match(r"^\s*block\s+(\S+)", line, re.I)
        d = re.match(r"^\s*decay\s+(\d+)\s+\S+", line, re.I)
        e = re.match(r"^(\s*)(\d+)(\s+)\S+(.*)$", line)
        if m:
            block = m.group(1).lower()
        elif d:
            block = None
            key = ("decay", int(d.group(1)))
            if key in moves:
                line, _ = "DECAY %s %e" % (d.group(1), moves[key]), found.add(key)
        elif e and block is not None:
            key = (block, int(e.group(2)))
            if key in moves:
                line = "%s%s%s%e%s" % (e.group(1), e.group(2), e.group(3), moves[key], e.group(4))
                found.add(key)
        out.append(line)
    missing = set(moves) - found
    if missing:
        sys.exit("the restrict card has no entry %s" % sorted(missing))
    path = os.path.join(directory, "point_param_card.dat")
    with open(path, "w") as f:
        f.write("\n".join(out) + "\n")
    return path


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--mg-root", default=os.path.join(ROOT, "research/refs/mg5amcnlo"))
    parser.add_argument("--model", default=None, help="UFO directory (default: MG_ROOT/models/sm)")
    parser.add_argument("--restrict", default="restrict_default.dat")
    parser.add_argument("--point", action="append", default=[])
    args = parser.parse_args()

    # MadGraph's tree is a pinned checkout: no bytecode is written into it.
    sys.dont_write_bytecode = True
    sys.path.insert(0, os.path.abspath(args.mg_root))
    logging.basicConfig(level=logging.WARNING)
    import models.import_ufo as import_ufo  # noqa: E402

    restrict_model = import_ufo.RestrictModel
    seen = {"fused": [], "zero": [], "fixed0": [], "fixed1": []}
    merge, detect, special = (
        restrict_model.merge_iden_couplings,
        restrict_model.detect_identical_couplings,
        restrict_model.detect_special_parameters,
    )

    def record_merge(self, couplings):
        seen["fused"].append(list(couplings))
        return merge(self, couplings)

    def record_detect(self, strict_zero=False):
        zero, iden = detect(self, strict_zero)
        seen["zero"] = list(zero)
        return zero, iden

    def record_special(self):
        zeros, ones = special(self)
        seen["fixed0"], seen["fixed1"] = list(zeros), list(ones)
        return zeros, ones

    restrict_model.merge_iden_couplings = record_merge
    restrict_model.detect_identical_couplings = record_detect
    restrict_model.detect_special_parameters = record_special

    source = os.path.abspath(args.model or os.path.join(args.mg_root, "models/sm"))
    work = tempfile.mkdtemp(prefix="vg_fusion_")
    try:
        ufo = os.path.join(work, os.path.basename(source))
        shutil.copytree(source, ufo, ignore=shutil.ignore_patterns("*.pkl", "__pycache__"))
        restrict = os.path.join(ufo, args.restrict)
        import_ufo.import_model(ufo, restrict=restrict, prefix="mdl_")
        full = restrict_model(import_ufo.import_model(ufo, restrict=False, prefix="mdl_"))
        exprs = {c.name: c.expr for cl in full["couplings"].values() for c in cl}

        print("MadGraph", open(os.path.join(args.mg_root, "VERSION")).readline().strip())
        print("model %s, restriction %s" % (os.path.basename(source), args.restrict))
        print("fused groups (member = factor * first):")
        for group in seen["fused"]:
            print("  " + ", ".join("%s(%+d)" % (n, f) for n, f in group))
            for n, _ in group:
                print("      %s = %s" % (n, exprs[n]))
        print("couplings removed as zero: %d" % len(seen["zero"]))
        print("parameters fixed to zero: %s" % " ".join(sorted(seen["fixed0"])))
        print("parameters fixed to one: %s" % " ".join(sorted(seen["fixed1"])))

        failed = False
        points = DEFAULT_POINTS + [parse_point(p) for p in args.point]
        for name, moves, control in [("restriction point", {}, False)] + points:
            full.set_parameters_and_couplings(card_with(restrict, moves, work))
            cd, pd = full["coupling_dict"], full["parameter_dict"]
            splits = []
            for group in seen["fused"]:
                first = group[0][0]
                for member, factor in group[1:]:
                    a, b = cd[first], cd[member]
                    scale = max(abs(a), abs(b))
                    rel = abs(b - factor * a) / scale if scale else 0.0
                    if rel > SPLIT:
                        splits.append("%s = %+d * %s (rel %.3e)" % (member, factor, first, rel))
            revived = [n for n in seen["zero"] if abs(cd[n]) > ZERO]
            moved = [n for n in seen["fixed0"] if pd[n] != 0] + [
                n for n in seen["fixed1"] if pd[n] != 1
            ]
            clean = not (splits or revived or moved)
            if control:
                verdict = "control splits, as it must" if splits else "CONTROL DID NOT SPLIT"
                failed |= not splits
            else:
                verdict = "holds" if clean else "BROKEN"
                failed |= not clean
            print("\n== %s: %s" % (name, verdict))
            for (block, code), value in sorted(moves.items()):
                print("   card: %s %d = %g" % (block, code, value))
            print("   MZ %.6g, MW %.10g, ymt %.6g" % (pd["mdl_MZ"], pd["mdl_MW"].real, pd["mdl_ymt"]))
            print("   fused pairs split: %s" % ("; ".join(splits) or "none"))
            print("   removed couplings non-zero: %s" % (" ".join(revived) or "none"))
            print("   fixed parameters moved: %s" % (" ".join(moved) or "none"))
        print("\n%s" % ("FAIL" if failed else "PASS"))
        return 1 if failed else 0
    finally:
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
