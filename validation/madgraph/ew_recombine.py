"""Equal-weight recombination of a MadEvent run's per-channel iterations.

MadEvent combines each channel's iterations with weights from their own variance
estimates (mean^2/variance). When an iteration's error is underestimated, that
weighting pulls the channel's value toward the low iterations, and the run's sigma
reads low (`research/kb/validation/madevent-default-survey-bias.md`). This script
recombines the same iterations with equal weights, so the two can be compared.

Rule, per run directory:
1. Channels: every `SubProcesses/P*/G*/results.dat`, refine splits (`G*.NN`)
   included. The first field of each file's first line is MadEvent's value for
   that channel; their sum closes on the run's `SubProcesses/results.dat`.
2. Iteration rows: every later line that is not an XML tag and has at least five
   whitespace-separated fields. Field 2 is the iteration's mean and field 3 its
   error. Fortran `D`/`d` and lowercase `e` exponents are accepted, and leading
   whitespace is not required (refine-combined channels write their rows flush
   left).
3. A channel's equal-weight value is the plain mean of exactly the rows its
   results.dat lists (an ordinary channel lists its last three iterations, a
   refine-combined one all of them). Its error is sqrt(mean(err^2)/n).
4. A channel with no iteration rows keeps MadEvent's own value and error.
5. One run per directory: a relaunched directory (`run_02`, ...) keeps only the
   last run's results.dat, so give every seed its own fresh directory.

Usage: python3 ew_recombine.py RUN_DIR [RUN_DIR ...]

A directory that cannot be recombined (no total results.dat, a results.dat
without its value and error, a field that is not a number, or no channel with a
nonzero value) is named on stderr and the exit status is 1; the other directories
are still printed.
"""

import glob
import math
import sys


def fortran_float(text: str) -> float:
    return float(text.replace("D", "E").replace("d", "e"))


class RunError(Exception):
    """A run directory whose results cannot be recombined."""


def channel_rows(path: str) -> list[list[str]]:
    """The non-blank, non-tag lines of one channel's results.dat, split into fields."""
    with open(path) as f:
        return [
            line.split()
            for line in f
            if line.strip() and not line.strip().startswith("<")
        ]


def recombine(run: str) -> str:
    total_path = f"{run}/SubProcesses/results.dat"
    try:
        with open(total_path) as f:
            total = f.readline().split()
    except FileNotFoundError:
        raise RunError(f"no {total_path}; the run did not finish") from None
    if len(total) < 2:
        raise RunError(f"{total_path} has no total and error on its first line")
    reported = equal = equal_var = 0.0
    channels = 0
    for path in sorted(glob.glob(f"{glob.escape(run)}/SubProcesses/P*/G*/results.dat")):
        rows = channel_rows(path)
        if not rows or len(rows[0]) < 2:
            raise RunError(f"{path} has no channel value and error on its first line")
        try:
            value, error = fortran_float(rows[0][0]), fortran_float(rows[0][1])
            iterations = [
                (fortran_float(row[1]), fortran_float(row[2]))
                for row in rows[1:]
                if len(row) >= 5
            ]
        except ValueError as why:
            raise RunError(f"{path}: {why}") from None
        reported += value
        if iterations:
            n = len(iterations)
            equal += sum(mean for mean, _ in iterations) / n
            equal_var += sum(err * err for _, err in iterations) / n / n
        else:
            equal += value
            equal_var += error * error
        channels += 1
    if channels == 0 or reported == 0.0:
        raise RunError(
            f"{run}/SubProcesses has no channel results.dat with a nonzero value"
        )
    return (
        f"{run}: channels {channels}; "
        f"reported total {fortran_float(total[0]):.6g} +- {fortran_float(total[1]):.3g}; "
        f"sum of channel values {reported:.6g}; "
        f"equal-weight {equal:.6g} +- {math.sqrt(equal_var):.3g}; "
        f"shift {equal / reported - 1:+.4f}"
    )


def main(argv: list[str]) -> None:
    if not argv:
        sys.exit(__doc__)
    failed = False
    for run_dir in argv:
        try:
            print(recombine(run_dir))
        except RunError as why:
            print(f"{run_dir}: {why}", file=sys.stderr)
            failed = True
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main(sys.argv[1:])
