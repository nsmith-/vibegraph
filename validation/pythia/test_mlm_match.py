"""The tolerance of the matched Pythia comparison (mlm_match.py) on synthetic
inputs: the five pull tests, the jet-rate chi2 and per-bin legs with their
look-elsewhere factors, and the null that decides whether the jet rates gate.

    pixi run -e pythia test-mlm-pythia
"""

from __future__ import annotations

import json
import math
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

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


def level_hist(chi2: float) -> list[float]:
    """24 bins of equal magnitude whose pulls square-sum to `chi2`."""
    return background(math.sqrt(chi2 / N_PER_HIST))


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


class Chi2Sf(unittest.TestCase):
    """chi2_sf against values that do not come from it."""

    def test_scipy_values_on_both_branches(self):
        # scipy.stats.chi2.sf; (12, 23) is the series, (40, 23) and (61.44, 23)
        # the continued fraction.
        for x, k, want in [
            (3.0, 1, 0.08326451666355042), (0.5, 10, 0.999993388289439), (7.5, 5, 0.186029833602867),
            (25.0, 5, 0.0001393337911856263), (12.0, 23, 0.9704706782005461),
            (23.0, 23, 0.46077089055229187), (40.0, 23, 0.015369082400709319),
            (61.44, 23, 2.359411787345888e-05),
        ]:
            self.assertAlmostEqual(m.chi2_sf(x, k) / want, 1.0, delta=1e-9, msg=(x, k))

    def test_even_dof_closed_form(self):
        for k in (2, 6, 24):
            for x in (0.7, 5.0, 14.0, 31.0, 70.0):
                want = math.exp(-x / 2) * sum((x / 2) ** j / math.factorial(j) for j in range(k // 2))
                self.assertAlmostEqual(m.chi2_sf(x, k) / want, 1.0, delta=1e-9, msg=(x, k))

    def test_degenerate_arguments(self):
        self.assertIsNone(m.chi2_sf(5.0, 0))
        self.assertEqual(m.chi2_sf(0.0, 4), 1.0)


class Thresholds(unittest.TestCase):
    """Each leg's threshold, probed on both sides of it, so that a change of
    alpha, of a trial factor or of the sidedness moves a verdict."""

    # chi2 at 23 dof whose p-value is 0.0015 (3p = 0.0045 < 0.005) and 0.0019
    # (3p = 0.0057): scipy.stats.chi2.isf. Factor 2 or "no factor" fails the
    # second; factor 4 passes the first.
    CHI2_P0015 = 48.36989538822336
    CHI2_P0019 = 47.566858589722116

    def test_chi2_leg_threshold_is_p_min_times_three_against_alpha(self):
        fail = m.jet_rate_legs(jet_rates(background(), level_hist(self.CHI2_P0015), background()))
        self.assertAlmostEqual(fail["chi2"]["p_values"][m.DJR_NAMES[1]], 0.0015, delta=1e-7)
        self.assertEqual((fail["chi2"]["verdict"], fail["per_bin"]["verdict"]), ("fail", "pass"))
        passing = m.jet_rate_legs(jet_rates(background(), level_hist(self.CHI2_P0019), background()))
        self.assertAlmostEqual(passing["chi2"]["p_values"][m.DJR_NAMES[1]], 0.0019, delta=1e-7)
        self.assertEqual((passing["chi2"]["verdict"], passing["per_bin"]["verdict"]), ("pass", "pass"))

    def test_per_bin_leg_threshold_is_z_star_at_the_total_bin_count(self):
        # z*(72) = 3.978: erfc(z / sqrt 2) * 72 is 0.0061 at 3.93 and 0.0040 at 4.03.
        self.assertAlmostEqual(m.z_star(72), 3.978181192219203, places=9)
        for z, want in [(3.93, "pass"), (4.03, "fail"), (-3.93, "pass"), (-4.03, "fail")]:
            legs = m.jet_rate_legs(jet_rates(background(), with_excess(background(), [7], z), background()))
            self.assertEqual(legs["per_bin"]["verdict"], want, z)
            self.assertEqual(legs["chi2"]["verdict"], "pass", z)

    def test_per_bin_trial_factor_counts_every_histogram_bin(self):
        # Twelve bins a histogram: 36 in all, z* = 3.810, so 3.93 fails here and passes at 72.
        short = background()[:12]
        legs = m.jet_rate_legs(jet_rates(short, with_excess(short, [7], 3.93), short))
        self.assertEqual(legs["per_bin"]["n_bins"], 36)
        self.assertAlmostEqual(legs["per_bin"]["z_star"], 3.810138630616644, places=9)
        self.assertEqual(legs["per_bin"]["verdict"], "fail")


class Judge(unittest.TestCase):
    def comparison(self):
        return {
            "acceptance": {"0": {"pull": -0.1}, "1": {"pull": -0.2}, "2": {"pull": -0.3}, "all": {"pull": -0.4}},
            "merged_sigma_pb": {"pull": 0.5}, "merged_over_sigma_lhe": {"pull": 2.9},
            "jet_rates": QUIET,
        }

    def test_gated_pulls_reads_the_five_documented_quantities(self):
        self.assertEqual(m.gated_pulls(self.comparison()),
                         {"0": -0.1, "1": -0.2, "2": -0.3, "all": -0.4, "merged_sigma": 0.5})

    def test_a_missing_acceptance_is_a_missing_pull(self):
        c = self.comparison()
        del c["acceptance"]["all"]
        pulls = m.gated_pulls(c)
        self.assertIsNone(pulls["all"])
        self.assertEqual(m.judge(c, {"a": None, "b": None})[2]["pulls_failed"], ["all"])

    def test_side_a_null_decides_and_side_b_null_does_not(self):
        failing = {"legs": m.jet_rate_legs(jet_rates(background(1.6), background(), background()))}
        quiet = {"legs": m.jet_rate_legs(QUIET)}
        c = self.comparison()
        self.assertEqual(m.judge(c, {"a": failing, "b": quiet})[2]["jet_rates"]["verdict"], "null failed")
        self.assertEqual(m.judge(c, {"a": quiet, "b": failing})[2]["jet_rates"],
                         {"mode": "gate", "verdict": "pass"})
        self.assertEqual(m.judge(c, {"a": None, "b": quiet})[2]["jet_rates"]["verdict"], "no null")


class NullSplit(unittest.TestCase):
    """The split is the first ceil(n/2) files against the rest, files, events
    and runs sliced alike."""

    def run_split(self, n):
        class F:
            def __init__(self, i):
                self.path = Path(f"f{i}")
        files = [F(i) for i in range(n)]
        events = [f"e{i}" for i in range(n)]
        runs = [f"r{i}" for i in range(n)]
        seen = {"summaries": [], "compared": None}

        def side_summary(fs, es, rs):
            seen["summaries"].append(([f.path.name for f in fs], es, rs))
            return {"which": len(seen["summaries"])}

        def compare_sides(sa, sb, ev_a, ev_b, min_count, n_boot):
            seen["compared"] = (sa, sb, ev_a, ev_b, min_count, n_boot)
            return {"acceptance": {}, "merged_sigma_pb": {"pull": 0.0},
                    "merged_over_sigma_lhe": {"pull": 0.0}, "jet_rates": QUIET}

        with mock.patch.object(m, "side_summary", side_summary), \
                mock.patch.object(m, "compare_sides", compare_sides):
            out = m.null_split(files, events, runs, 100.0)
        return out, seen

    def test_five_files_split_three_and_two(self):
        out, seen = self.run_split(5)
        self.assertEqual(seen["summaries"], [(["f0", "f1", "f2"], ["e0", "e1", "e2"], ["r0", "r1", "r2"]),
                                             (["f3", "f4"], ["e3", "e4"], ["r3", "r4"])])
        _, _, ev_a, ev_b, min_count, n_boot = seen["compared"]
        self.assertEqual((ev_a, ev_b, min_count, n_boot), (["e0", "e1", "e2"], ["e3", "e4"], 100.0, 0))
        self.assertEqual(out["files"], [["f0", "f1", "f2"], ["f3", "f4"]])
        self.assertEqual(out["legs"]["verdict"], "pass")

    def test_two_files_split_one_and_one_and_one_file_has_no_null(self):
        _, seen = self.run_split(2)
        self.assertEqual([len(s[0]) for s in seen["summaries"]], [1, 1])
        out, _ = self.run_split(1)
        self.assertIsNone(out)


class Build(unittest.TestCase):
    def test_the_version_prints_as_the_version_string(self):
        self.assertEqual(m.pythia_version("8.3120e+00"), "8.312")
        self.assertEqual(m.pythia_version("8.31"), "8.310")
        self.assertEqual(m.pythia_version("?"), "?")

    def test_the_build_names_the_platform_and_the_conda_package(self):
        with tempfile.TemporaryDirectory() as prefix:
            meta = Path(prefix) / "conda-meta"
            meta.mkdir()
            (meta / "pythia8-8.312-py312hd50de4e_105.json").write_text(json.dumps({
                "name": "pythia8", "version": "8.312", "build": "py312hd50de4e_105",
                "channel": "https://conda.anaconda.org/conda-forge/", "subdir": "linux-64"}))
            (meta / "pythia8-data-1.json").write_text(json.dumps({"name": "other", "version": "0", "build": "x"}))
            build = m.pythia_build(Path(prefix))
        self.assertEqual(build["package"], "pythia8-8.312-py312hd50de4e_105")
        self.assertEqual((build["channel"], build["subdir"]),
                         ("https://conda.anaconda.org/conda-forge/", "linux-64"))
        self.assertRegex(build["platform"], r"^\w+ \w+$")

    def test_without_a_conda_package_only_the_platform_and_config_remain(self):
        with tempfile.TemporaryDirectory() as prefix:
            build = m.pythia_build(Path(prefix))
        self.assertNotIn("package", build)
        self.assertIn("platform", build)


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
