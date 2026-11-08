#!/usr/bin/env python3
"""Beam balance of a proton-proton event file: per unordered initial state.

At proton-proton beams an initial state {a, b} with a != b is as likely to put
a on beam 1 as on beam 2, so in an unbiased sample each such pair splits its
events, and its weight, evenly between the two beams. A generator that samples
one beam ordering worse than the other shows here as a pair whose split sits
many binomial standard deviations from one half: confidently, since the
undersampled ordering is missing from the variance as well as from the sum.

Usage: beam_balance.py EVENTS.lhe[.gz] [--min-events N] [--pull P]

Prints one line per unordered initial state with at least N events: the event
count, the share of events and of weight with the pair's first parton (by PDG
code) on beam 1, and the binomial pull of the event share. Exits 1 if any
pair's pull exceeds P (default 5), 0 otherwise.

It applies to a card whose initial states are written in both orderings
(`p p > ...`); a line written in one ordering (`u u~ > ...`) puts every event
of its pair on the written beams, as MadGraph does, and reads as maximally
unbalanced here.

What it cannot see: anything symmetric under exchanging the beams, which
includes every defect that affects both orderings alike, and a pair with too
few events to resolve its split. Identical partons (a == b) and the outgoing
state are not read.
"""

import argparse
import gzip
import math
import sys


def events(path):
    """Yield (weight, [(pdg, pz) of the incoming partons]) per <event>."""
    opener = gzip.open if path.endswith(".gz") else open
    with opener(path, "rt") as fh:
        inside = False
        lines = []
        for line in fh:
            if line.lstrip().startswith("<event"):
                inside, lines = True, []
                continue
            if inside and line.lstrip().startswith("</event"):
                inside = False
                yield parse(lines)
                continue
            if inside:
                lines.append(line)


def parse(lines):
    rows = [l.split() for l in lines if l.strip() and not l.lstrip().startswith("<")]
    head = rows[0]
    n, weight = int(head[0]), float(head[2])
    incoming = []
    for row in rows[1 : 1 + n]:
        if int(row[1]) == -1:
            incoming.append((int(row[0]), float(row[8])))
    return weight, incoming


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("lhe")
    ap.add_argument("--min-events", type=int, default=20)
    ap.add_argument("--pull", type=float, default=5.0)
    args = ap.parse_args()

    tally = {}
    for weight, incoming in events(args.lhe):
        if len(incoming) != 2:
            continue
        (p1, z1), (p2, z2) = incoming
        if z1 < z2:
            (p1, z1), (p2, z2) = (p2, z2), (p1, z1)
        if p1 == p2:
            continue
        pair = tuple(sorted((p1, p2)))
        t = tally.setdefault(pair, [0, 0, 0.0, 0.0])
        first_on_beam_1 = p1 == pair[0]
        t[0] += 1
        t[1] += first_on_beam_1
        t[2] += abs(weight)
        t[3] += abs(weight) if first_on_beam_1 else 0.0

    worst = 0.0
    print("%-14s %8s %10s %10s %8s" % ("pair", "events", "n-share", "w-share", "pull"))
    for pair, (n, k, w, wk) in sorted(tally.items(), key=lambda kv: -kv[1][0]):
        if n < args.min_events:
            continue
        pull = (k - 0.5 * n) / math.sqrt(0.25 * n)
        worst = max(worst, abs(pull))
        print("%-14s %8d %10.4f %10.4f %+8.2f" % ("%d %d" % pair, n, k / n, wk / w, pull))
    print("worst |pull| %.2f (threshold %.1f)" % (worst, args.pull))
    return 1 if worst > args.pull else 0


if __name__ == "__main__":
    sys.exit(main())
