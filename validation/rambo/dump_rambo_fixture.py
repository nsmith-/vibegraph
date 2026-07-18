#!/usr/bin/env python3
"""Dump a RAMBO uniforms-replay fixture for the Rust phase-space map.

Mirrors ``validation/madgraph/gen_amplitude.py::rambo`` but consumes an explicit
list of uniforms (rather than drawing them) so the deterministic map
``uniforms -> (momenta, xi, weight)`` can be replayed bit-for-bit in Rust. The
operation order matches ``vibegraph-lib/src/phasespace/rambo.rs`` exactly.

Pure standard library (no numpy): the uniforms come from ``random.Random`` with
fixed seeds, so the fixture is reproducible anywhere. Regenerate with:

    python3 validation/rambo/dump_rambo_fixture.py

and commit the resulting ``rambo_fixture.json``. The JSON is the checked-in
oracle; this script only regenerates it.
"""

import json
import math
import os
import random

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.join(HERE, "rambo_fixture.json")


def massless_momenta(sqrt_s, u):
    """n = len(u)//4 massless momenta summing to (sqrt_s, 0, 0, 0)."""
    n = len(u) // 4
    q = []
    for i in range(n):
        c = 2.0 * u[4 * i] - 1.0
        phi = 2.0 * math.pi * u[4 * i + 1]
        e = -math.log(u[4 * i + 2] * u[4 * i + 3])
        st = math.sqrt(1.0 - c * c)
        q.append([e, e * st * math.cos(phi), e * st * math.sin(phi), e * c])

    tot = [0.0, 0.0, 0.0, 0.0]
    for qi in q:
        for k in range(4):
            tot[k] += qi[k]
    m = math.sqrt(tot[0] * tot[0] - tot[1] * tot[1] - tot[2] * tot[2] - tot[3] * tot[3])
    b = [-tot[1] / m, -tot[2] / m, -tot[3] / m]
    gamma = tot[0] / m
    a = 1.0 / (1.0 + gamma)
    x = sqrt_s / m

    p = []
    for qi in q:
        bq = b[0] * qi[1] + b[1] * qi[2] + b[2] * qi[3]
        e = x * (gamma * qi[0] + bq)
        f = x * (qi[0] + a * bq)
        p.append([e, x * qi[1] + f * b[0], x * qi[2] + f * b[1], x * qi[3] + f * b[2]])
    return p


def factorial(k):
    out = 1.0
    for j in range(1, k + 1):
        out *= float(j)
    return out


def massless_volume(sqrt_s, n):
    s = sqrt_s * sqrt_s
    half_pi = math.pi / 2.0
    numer = half_pi ** (n - 1) * s ** (n - 2)
    return numer / (factorial(n - 1) * factorial(n - 2))


def massive_jacobian(sqrt_s, k):
    n = len(k)
    sum_p = 0.0
    sum_p2_over_e = 0.0
    prod_p_over_e = 1.0
    for ki in k:
        p = math.sqrt(ki[1] * ki[1] + ki[2] * ki[2] + ki[3] * ki[3])
        e = ki[0]
        sum_p += p
        sum_p2_over_e += p * p / e
        prod_p_over_e *= p / e
    return (sum_p / sqrt_s) ** (2 * n - 3) / sum_p2_over_e * sqrt_s * prod_p_over_e


def rambo(sqrt_s, masses, u):
    n = len(masses)
    massless = massless_momenta(sqrt_s, u)
    volume = massless_volume(sqrt_s, n)

    if all(m == 0.0 for m in masses):
        return massless, 1.0, volume

    p2 = [px * px + py * py + pz * pz for (_, px, py, pz) in massless]
    m2 = [m * m for m in masses]
    ratio = sum(masses) / sqrt_s
    xi = math.sqrt(max(1e-12, 1.0 - ratio * ratio))
    tol = 1e-13 * sqrt_s
    for _ in range(100):
        f = -sqrt_s
        df = 0.0
        for p2i, m2i in zip(p2, m2):
            e = math.sqrt(m2i + xi * xi * p2i)
            f += e
            df += p2i / e
        if abs(f) < tol:
            break
        xi -= f / (xi * df)

    k = []
    for (_, px, py, pz), m2i, p2i in zip(massless, m2, p2):
        e = math.sqrt(m2i + xi * xi * p2i)
        k.append([e, xi * px, xi * py, xi * pz])

    weight = volume * massive_jacobian(sqrt_s, k)
    return k, xi, weight


# (name, sqrt_s, masses, rng seed). Covers massless fast path, 2-body and
# multi-body massive, mixed masses, and threshold-adjacent mass sums.
CASES = [
    ("massless_2_100", 100.0, [0.0, 0.0], 1),
    ("massless_3_250", 250.0, [0.0, 0.0, 0.0], 2),
    ("massless_6_500", 500.0, [0.0] * 6, 3),
    ("massive_2_ww_400", 400.0, [80.4, 80.4], 4),
    ("massive_3_mixed_300", 300.0, [10.0, 20.0, 5.0], 5),
    ("massive_4_mixed_500", 500.0, [0.0, 5.0, 0.0, 30.0], 6),
    ("threshold_3_91", 91.0, [30.0, 30.0, 30.0], 7),
    ("threshold_2_90p5", 90.5, [45.0, 45.0], 8),
]


def main():
    entries = []
    for name, sqrt_s, masses, seed in CASES:
        rng = random.Random(seed)
        n = len(masses)
        u = [rng.random() for _ in range(4 * n)]
        momenta, xi, weight = rambo(sqrt_s, masses, u)
        entries.append(
            {
                "name": name,
                "sqrt_s": sqrt_s,
                "masses": masses,
                "uniforms": u,
                "momenta": momenta,
                "xi": xi,
                "weight": weight,
            }
        )
    with open(FIXTURE, "w") as fh:
        json.dump({"cases": entries}, fh, indent=2)
        fh.write("\n")
    print(f"wrote {len(entries)} cases to {FIXTURE}")


if __name__ == "__main__":
    main()
