"""The tolerance of the matched Pythia comparison (mlm_match.py) on synthetic
inputs: the five pull tests, the jet-rate chi2 and per-bin legs with their
look-elsewhere factors, and the null that decides whether the jet rates gate.

    pixi run -e pythia test-mlm-pythia
"""

from __future__ import annotations

import math
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import mlm_match as m  # noqa: E402

#: Bins per synthetic histogram: three of them make 72, the comparison's size.
N_PER_HIST = 24


def hist(pulls: list[float]) -> dict:
    """A jet-rate histogram whose bins read the given pulls B - A, with unit
    combined error, and the chi2 and p-value the driver would compute."""
    e = 1.0 / math.sqrt(2.0)
    chi2 = float(sum(z * z for z in pulls))
    dof = len(pulls) - 1
    return {
        "bins": [[0.1 * i, 0.1 * (i + 1)] for i in range(len(pulls))],
        "shape_a": [[10.0, e] for _ in pulls],
        "shape_b": [[10.0 + z, e] for z in pulls],
        "chi2": chi2, "dof": dof, "p_value": m.chi2_sf(chi2, dof),
    }


def background(level: float = 0.5) -> list[float]:
    """Alternating +-level pulls: chi2 = level^2 per bin, nothing localised."""
    return [level if i % 2 else -level for i in range(N_PER_HIST)]


def jet_rates(*pulls: list[float]) -> dict:
    return {name: hist(p) for name, p in zip(m.DJR_NAMES, pulls)}


def with_excess(base: list[float], bins: list[int], z: float) -> list[float]:
    out = list(base)
    for j in bins:
        out[j] = z
    return out


QUIET = jet_rates(background(), background(), background())
GOOD_PULLS = {"0": -0.8, "1": -0.95, "2": -0.98, "all": -0.66, "merged_sigma": 0.19}


class ZStar(unittest.TestCase):
    def test_z_star_at_72_bins(self):
        z = m.z_star(72)
        self.assertAlmostEqual(math.erfc(z / math.sqrt(2.0)) * 72, m.JET_RATE_ALPHA, places=12)
        self.assertGreater(z, 3.96)
        self.assertLess(z, 3.99)

    def test_z_star_grows_with_the_bin_count(self):
        self.assertLess(m.z_star(10), m.z_star(72))
        self.assertLess(m.z_star(72), m.z_star(500))


class Pulls(unittest.TestCase):
    def test_all_pulls_below_three_pass(self):
        legs = m.jet_rate_legs(QUIET)
        self.assertEqual(m.verdict(GOOD_PULLS, legs, legs)["status"], "pass")

    def test_each_pull_fails_alone_at_three(self):
        legs = m.jet_rate_legs(QUIET)
        for key in m.PULL_KEYS:
            for value in (3.0, -3.0, 3.4):
                pulls = {**GOOD_PULLS, key: value}
                d = m.verdict(pulls, legs, legs)
                self.assertEqual(d["status"], "fail", (key, value))
                self.assertEqual(d["pulls_failed"], [key])
            self.assertEqual(m.verdict({**GOOD_PULLS, key: 2.99}, legs, legs)["status"], "pass")

    def test_a_missing_pull_fails(self):
        legs = m.jet_rate_legs(QUIET)
        d = m.verdict({**GOOD_PULLS, "2": None}, legs, legs)
        self.assertEqual((d["status"], d["pulls_failed"]), ("fail", ["2"]))
        pulls = dict(GOOD_PULLS)
        del pulls["merged_sigma"]
        self.assertEqual(m.verdict(pulls, legs, legs)["pulls_failed"], ["merged_sigma"])


class JetRateLegs(unittest.TestCase):
    def test_quiet_histograms_pass_both_legs(self):
        legs = m.jet_rate_legs(QUIET)
        self.assertEqual(legs["per_bin"]["n_bins"], 72)
        self.assertEqual((legs["chi2"]["verdict"], legs["per_bin"]["verdict"]), ("pass", "pass"))

    def test_a_two_bin_three_sigma_excess_passes_both_legs(self):
        # Below z*: the blind spot the per-bin leg leaves, and the chi2 leg too.
        h = with_excess(background(), [7, 8], 3.0)
        legs = m.jet_rate_legs(jet_rates(background(), h, background()))
        self.assertEqual((legs["chi2"]["verdict"], legs["per_bin"]["verdict"]), ("pass", "pass"))
        self.assertAlmostEqual(abs(legs["per_bin"]["max_abs_pull"]["pull"]), 3.0)

    def test_a_two_bin_excess_beyond_z_star_is_caught_by_the_per_bin_leg_only(self):
        h = with_excess(background(), [7, 8], 4.1)
        legs = m.jet_rate_legs(jet_rates(background(), h, background()))
        self.assertGreater(4.1, legs["per_bin"]["z_star"])
        self.assertEqual(legs["chi2"]["verdict"], "pass", legs["chi2"])
        self.assertEqual(legs["per_bin"]["verdict"], "fail")
        self.assertEqual(legs["verdict"], "fail")
        worst = legs["per_bin"]["max_abs_pull"]
        self.assertEqual((worst["histogram"], worst["bin"]), (m.DJR_NAMES[1], 7))
        d = m.verdict(GOOD_PULLS, legs, m.jet_rate_legs(QUIET))
        self.assertEqual((d["status"], d["jet_rates"]), ("fail", {"mode": "gate", "verdict": "fail"}))

    def test_a_deficit_counts_as_an_excess(self):
        h = with_excess(background(), [3, 4], -4.1)
        legs = m.jet_rate_legs(jet_rates(h, background(), background()))
        self.assertEqual(legs["per_bin"]["verdict"], "fail")
        self.assertLess(legs["per_bin"]["max_abs_pull"]["pull"], 0)

    def test_a_spread_distortion_is_caught_by_the_chi2_leg_only(self):
        legs = m.jet_rate_legs(jet_rates(background(), background(1.6), background()))
        self.assertEqual(legs["per_bin"]["verdict"], "pass")
        self.assertEqual(legs["chi2"]["verdict"], "fail", legs["chi2"])

    def test_the_chi2_leg_takes_the_three_histogram_trial_factor(self):
        legs = m.jet_rate_legs(QUIET)
        p = legs["chi2"]["p_values"]
        self.assertEqual(legs["chi2"]["trials"], 3)
        self.assertAlmostEqual(legs["chi2"]["p_min_x_trials"], 3 * min(p.values()))


class Null(unittest.TestCase):
    def test_a_failing_null_makes_the_jet_rates_information(self):
        failing = m.jet_rate_legs(jet_rates(with_excess(background(), [5], 5.0), background(), background()))
        self.assertEqual(failing["verdict"], "fail")
        bad = m.jet_rate_legs(jet_rates(with_excess(background(), [7, 8], 4.5), background(), background()))
        d = m.verdict(GOOD_PULLS, bad, failing)
        self.assertEqual(d["jet_rates"]["mode"], "info")
        self.assertEqual(d["jet_rates"]["verdict"], "null failed")
        self.assertEqual(d["status"], "pass")

    def test_a_failing_null_does_not_excuse_a_pull(self):
        failing = m.jet_rate_legs(jet_rates(with_excess(background(), [5], 5.0), background(), background()))
        d = m.verdict({**GOOD_PULLS, "1": 3.5}, m.jet_rate_legs(QUIET), failing)
        self.assertEqual((d["status"], d["pulls_failed"]), ("fail", ["1"]))

    def test_a_null_failing_the_chi2_leg_alone_is_a_failed_null(self):
        failing = m.jet_rate_legs(jet_rates(background(1.6), background(), background()))
        self.assertEqual((failing["chi2"]["verdict"], failing["per_bin"]["verdict"]), ("fail", "pass"))
        self.assertEqual(m.verdict(GOOD_PULLS, m.jet_rate_legs(QUIET), failing)["jet_rates"]["verdict"],
                         "null failed")

    def test_no_null_makes_the_jet_rates_information(self):
        d = m.verdict(GOOD_PULLS, m.jet_rate_legs(QUIET), None)
        self.assertEqual((d["status"], d["jet_rates"]["mode"], d["jet_rates"]["verdict"]),
                         ("pass", "info", "no null"))


class Record(unittest.TestCase):
    def test_the_report_row_carries_every_number_the_verdict_rests_on(self):
        legs = m.jet_rate_legs(jet_rates(background(), with_excess(background(), [7], 3.2), background()))
        null_a = {"pulls": GOOD_PULLS, "legs": m.jet_rate_legs(QUIET)}
        decision = m.verdict(GOOD_PULLS, legs, null_a["legs"])
        row = m.gate_record(GOOD_PULLS, legs, {"a": null_a, "b": None}, decision)
        self.assertEqual(row["status"], "pass")
        self.assertEqual(row["pulls"], GOOD_PULLS)
        jet = row["jet_rates"]
        self.assertEqual((jet["mode"], jet["verdict"], jet["legs"]), ("gate", "pass", "pass"))
        self.assertEqual(set(jet["p_values"]), set(m.DJR_NAMES))
        self.assertEqual(jet["n_bins"], 72)
        self.assertAlmostEqual(jet["z_star"], m.z_star(72))
        self.assertEqual((jet["max_abs_pull"]["histogram"], jet["max_abs_pull"]["bin"]), (m.DJR_NAMES[1], 7))
        self.assertAlmostEqual(jet["max_abs_pull"]["pull"], 3.2)
        self.assertEqual(row["null_a"]["n_bins"], 72)
        self.assertIn("max_abs_pull", row["null_a"])
        self.assertIsNone(row["null_b"])


if __name__ == "__main__":
    unittest.main()
