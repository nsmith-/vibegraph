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
"""

import glob
import math
import sys


def fortran_float(text: str) -> float:
    return float(text.replace("D", "E").replace("d", "e"))


def recombine(run: str) -> str:
    total_path = f"{run}/SubProcesses/results.dat"
    try:
        with open(total_path) as f:
            total = f.readline().split()
    except FileNotFoundError:
        return f"{run}: no {total_path}; the run did not finish"
    reported = equal = equal_var = 0.0
    channels = 0
    for path in glob.glob(f"{run}/SubProcesses/P*/G*/results.dat"):
        with open(path) as f:
            rows = [
                line.split()
                for line in f
                if line.strip() and not line.strip().startswith("<")
            ]
        value, error = fortran_float(rows[0][0]), fortran_float(rows[0][1])
        reported += value
        iterations = [
            (fortran_float(row[1]), fortran_float(row[2]))
            for row in rows[1:]
            if len(row) >= 5
        ]
        if iterations:
            n = len(iterations)
            equal += sum(mean for mean, _ in iterations) / n
            equal_var += sum(err * err for _, err in iterations) / n / n
        else:
            equal += value
            equal_var += error * error
        channels += 1
    return (
        f"{run}: channels {channels}; "
        f"reported total {fortran_float(total[0]):.6g} +- {fortran_float(total[1]):.3g}; "
        f"sum of channel values {reported:.6g}; "
        f"equal-weight {equal:.6g} +- {math.sqrt(equal_var):.3g}; "
        f"shift {equal / reported - 1:+.4f}"
    )


if __name__ == "__main__":
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    for run_dir in sys.argv[1:]:
        print(recombine(run_dir))
