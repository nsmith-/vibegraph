"""Tests for ew_recombine.py on hand-built MadEvent run directories.

Run: python3 -m unittest discover -s validation/madgraph -p test_ew_recombine.py
"""

import io
import os
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout

import ew_recombine as ew

# An ordinary channel lists its value, then one row per iteration: the iteration
# number, mean, error, and two more fields. A row of fewer than five fields and an
# XML tag line, even one with several fields, are not iterations. A refine-combined channel's rows start
# flush left and use a Fortran D exponent.
ORDINARY = """  1.0  0.1  1.0  0.0  1.0
<tag>
<iteration number=1 mean=2.0 err=0.1 flag=0>
  1  1.0  0.1  9  9
  2  7.0  0.7  9
  2  3.0  0.3  9  9
"""
REFINED = """  5.0  0.5
1  4.0D+00  0.4D+00  9  9
2  6.0  0.6  9  9
3  8.0  0.8  9  9
"""
SUMMARY_ONLY = "  2.0  0.2\n"


def build_run(root: str, channels: dict) -> str:
    """A run directory with `channels` ({"P1/G1": results.dat text}) and a total."""
    os.makedirs(f"{root}/SubProcesses")
    with open(f"{root}/SubProcesses/results.dat", "w") as f:
        f.write("8.0 0.5\n")
    for name, text in channels.items():
        os.makedirs(f"{root}/SubProcesses/{name}")
        with open(f"{root}/SubProcesses/{name}/results.dat", "w") as f:
            f.write(text)
    return root


class Recombine(unittest.TestCase):
    def run_in(self, channels: dict) -> str:
        with tempfile.TemporaryDirectory() as tmp:
            return ew.recombine(build_run(tmp, channels))

    def test_equal_weights_replace_the_reported_values(self):
        # Equal-weight means: ordinary (1+3)/2 = 2, refined (4+6+8)/3 = 6, and a
        # channel without iteration rows keeps its own 2. Reported values: 1+5+2.
        out = self.run_in(
            {"P1/G1": ORDINARY, "P1/G2.01": REFINED, "P2/G7": SUMMARY_ONLY}
        )
        self.assertIn("channels 3;", out)
        self.assertIn("sum of channel values 8;", out)
        self.assertIn("equal-weight 10 +- ", out)
        self.assertIn("shift +0.2500", out)

    def test_a_run_directory_named_with_glob_characters_is_read_whole(self):
        with tempfile.TemporaryDirectory() as tmp:
            run = build_run(f"{tmp}/run[1]", {"P1/G1": ORDINARY})
            self.assertIn("channels 1;", ew.recombine(run))

    def test_the_error_adds_the_channels_in_quadrature(self):
        # Ordinary: variance (0.1^2 + 0.3^2)/2/2 = 0.025; a channel without rows
        # adds its own 0.2^2 = 0.04: sqrt(0.065) = 0.255.
        out = self.run_in({"P1/G1": ORDINARY, "P2/G7": SUMMARY_ONLY})
        self.assertIn("equal-weight 4 +- 0.255;", out)

    def refuses(self, channels: dict, message: str):
        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaises(ew.RunError) as raised:
                ew.recombine(build_run(tmp, channels))
        self.assertIn(message, str(raised.exception))

    def test_a_run_without_channels_is_refused(self):
        self.refuses({}, "no channel results.dat")

    def test_an_empty_channel_file_is_refused(self):
        self.refuses({"P1/G1": ""}, "no channel value and error")

    def test_a_channel_line_without_an_error_is_refused(self):
        self.refuses({"P1/G1": "1.0\n"}, "no channel value and error")

    def test_a_field_that_is_not_a_number_is_refused(self):
        self.refuses({"P1/G1": "1.0 0.1\n1 two 0.1 9 9\n"}, "P1/G1/results.dat: ")

    def test_a_total_without_an_error_is_refused(self):
        with tempfile.TemporaryDirectory() as tmp:
            run = build_run(tmp, {"P1/G1": ORDINARY})
            with open(f"{run}/SubProcesses/results.dat", "w") as f:
                f.write("8.0\n")
            with self.assertRaises(ew.RunError) as raised:
                ew.recombine(run)
        self.assertIn("no total and error", str(raised.exception))

    def test_an_unfinished_run_is_refused(self):
        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaises(ew.RunError) as raised:
                ew.recombine(tmp)
        self.assertIn("did not finish", str(raised.exception))

    def test_the_command_line_exits_nonzero_on_a_refusal(self):
        with tempfile.TemporaryDirectory() as tmp:
            run = build_run(tmp, {"P1/G1": ORDINARY})
            err = io.StringIO()
            with redirect_stdout(io.StringIO()), redirect_stderr(err):
                with self.assertRaises(SystemExit) as code:
                    ew.main([run, f"{tmp}/missing"])
        self.assertEqual(code.exception.code, 1)
        self.assertIn("did not finish", err.getvalue())


if __name__ == "__main__":
    unittest.main()
