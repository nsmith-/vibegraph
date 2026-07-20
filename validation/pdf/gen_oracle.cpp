// Generate the PDF grid oracle JSON using the LHAPDF C++ library directly.
//
// MadGraph evaluates PDFs through this same LHAPDF build, so its log-bicubic
// interpolation (LogBicubicInterpolator, the lhagrid1 default) is the correct
// reference for matching an MG cross section -- not a generic scipy-style
// global spline. The on-knot ("knot") category reads raw grid values straight
// out of LHAPDF's KnotArray, which is the ground truth the pure-parser gate
// checks; the interpolated categories (off_knot, seam, x_to_one_tail, corner)
// are evaluated through LHAPDF's xfxQ2 and banked for the interpolation gate.
//
// Handles both single- and multi-Q^2-subgrid sets. LHAPDF flattens all bands
// into one KnotArray with a shared x axis and the bands' Q^2 knots concatenated
// (each seam knot duplicated back to back); this generator recovers the per-
// band structure from that flat axis and emits seam-category points on both
// sides of, and exactly at, every internal boundary. For a single-subgrid set
// the "seam" category falls back to the global QMin/QMax edges.
//
// Usage: gen_oracle <data_dir> <set_name> <member> <output_json>
//   <data_dir>  directory containing <set_name>/<set_name>_<member>.dat + .info
//
// JSON is written with %.17g (round-trip-exact for f64); no JSON library.

#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <cmath>
#include <string>
#include <vector>

#include "LHAPDF/LHAPDF.h"
#include "LHAPDF/GridPDF.h"
#include "LHAPDF/KnotArray.h"
#include "LHAPDF/Paths.h"

namespace {

// A single emitted (category, pdg, x, Q^2, x*f) record.
struct Point {
  const char* category;
  int pdg;
  double x;
  double q2;
  double xf;
};

void write_double(FILE* out, double v) {
  // %.17g round-trips an IEEE-754 double exactly; JSON has no NaN literal, so
  // emit a large sentinel only if one ever appears (it should not here).
  if (std::isnan(v)) {
    std::fprintf(out, "null");
  } else {
    std::fprintf(out, "%.17g", v);
  }
}

void write_point(FILE* out, const Point& p, bool last) {
  std::fprintf(out, "    {\"category\": \"%s\", \"pdg\": %d, \"x\": ",
               p.category, p.pdg);
  write_double(out, p.x);
  std::fprintf(out, ", \"q2\": ");
  write_double(out, p.q2);
  std::fprintf(out, ", \"xf\": ");
  write_double(out, p.xf);
  std::fprintf(out, "}%s\n", last ? "" : ",");
}

}  // namespace

int main(int argc, char** argv) {
  if (argc != 5) {
    std::fprintf(stderr,
                 "usage: %s <data_dir> <set_name> <member> <output_json>\n",
                 argv[0]);
    return 2;
  }
  const std::string data_dir = argv[1];
  const std::string set_name = argv[2];
  const int member = std::atoi(argv[3]);
  const std::string out_path = argv[4];

  // Point LHAPDF at the fetched set only (a trailing "::" would append the
  // install prefix; we deliberately do not, so the fetched copy is the sole
  // source).
  LHAPDF::setPaths(data_dir);

  LHAPDF::GridPDF gpdf(set_name, member);
  const LHAPDF::KnotArray& ka = gpdf.knotarray();

  const std::vector<int>& flavors = gpdf.flavors();
  const std::vector<double>& xs = ka.xs();
  const std::vector<double>& q2s = ka.q2s();
  const size_t nx = ka.xsize();
  const size_t nq = ka.q2size();

  if (nx != xs.size() || nq != q2s.size()) {
    std::fprintf(stderr, "error: knot vector sizes disagree with shape\n");
    return 1;
  }

  // LHAPDF 6.5 flattens every Q^2 subgrid into one KnotArray whose x knots and
  // flavor list are shared across bands, while the Q^2 axis is the bands'
  // Q^2 knots concatenated in order. Each subgrid seam knot appears twice back
  // to back (the upper knot of one band equals the lower knot of the next), so
  // the flattened Q^2 sequence is monotone-nondecreasing but not strictly
  // increasing: a non-increase marks a seam. Split the flat axis at those
  // seams to recover the per-band [start, end) index ranges.
  //
  // The seam is where LHAPDF's bicubic switches subgrids. The x*f value is
  // continuous across it (both bands carry the same knot values there), but the
  // Q^2-derivative is not: at the seam the cross-band central difference would
  // straddle the zero-width duplicated knot, so LHAPDF collapses to a one-sided
  // difference on each band -- a kink, not a jump. Emitting seam points on both
  // sides of, and exactly at, each internal boundary pins that behavior.
  std::vector<std::pair<size_t, size_t>> bands;  // [start, end) into q2s
  {
    size_t start = 0;
    for (size_t i = 1; i < q2s.size(); ++i) {
      if (!(q2s[i] > q2s[i - 1])) {  // seam (duplicated knot) or any non-increase
        bands.push_back({start, i});
        start = i;
      }
    }
    bands.push_back({start, q2s.size()});
  }
  const size_t num_subgrids = bands.size();

  // Internal seam Q^2 values (one per adjacent-band boundary).
  std::vector<double> seam_q2;
  for (size_t b = 0; b + 1 < bands.size(); ++b) {
    seam_q2.push_back(q2s[bands[b].second - 1]);
  }

  std::vector<Point> points;

  // ---- knot: raw grid values, the pure-parser ground truth. -------------
  // Subsample x and Q^2 knot indices (corners + a few interior) rather than
  // the full nx*nq*nflav product, matching the retired oracle's coverage.
  std::vector<size_t> ix_samples = {0, nx / 3, 2 * nx / 3, nx - 1};
  std::vector<size_t> iq_samples = {0, nq / 2, nq - 1};
  // Also sample both flat indices of each internal seam knot: the value there
  // is shared by the two adjacent bands, so the parser must read it identically
  // from either band's block.
  for (size_t b = 0; b + 1 < bands.size(); ++b) {
    iq_samples.push_back(bands[b].second - 1);
    iq_samples.push_back(bands[b].second);
  }
  auto dedup = [](std::vector<size_t>& v) {
    std::sort(v.begin(), v.end());
    v.erase(std::unique(v.begin(), v.end()), v.end());
  };
  dedup(ix_samples);
  dedup(iq_samples);
  for (size_t ix : ix_samples) {
    for (size_t iq : iq_samples) {
      for (int pdg : flavors) {
        const int ipid = ka.get_pid(pdg);
        points.push_back(
            {"knot", pdg, xs[ix], q2s[iq], ka.xf(ix, iq, ipid)});
      }
    }
  }

  // Grid extent for the interpolated categories.
  const double x_min = xs.front();
  const double x_max = xs.back();
  const double q2_min = q2s.front();
  const double q2_max = q2s.back();
  const double q_min = std::sqrt(q2_min);
  const double q_max = std::sqrt(q2_max);

  auto geomspace = [](double a, double b, int n) {
    std::vector<double> v;
    const double la = std::log(a), lb = std::log(b);
    for (int i = 0; i < n; ++i) {
      const double t = (n == 1) ? 0.0 : double(i) / double(n - 1);
      v.push_back(std::exp(la + t * (lb - la)));
    }
    return v;
  };

  auto eval_all = [&](const char* cat, double x, double q2,
                      const std::vector<int>& pdgs) {
    for (int pdg : pdgs) {
      points.push_back({cat, pdg, x, q2, gpdf.xfxQ2(pdg, x, q2)});
    }
  };

  // Probe the gluon under both its PDG spellings (21 and the 0 alias).
  std::vector<int> probe_pdgs = flavors;
  probe_pdgs.push_back(0);

  // ---- off_knot: interior points strictly between knots. -----------------
  // One set of x samples, and Q^2 samples drawn from the interior of every
  // band so each subgrid's local cubic is exercised (a single global sweep
  // could skip a thin band entirely).
  const std::vector<double> off_x = geomspace(x_min * 10.0, 0.5, 4);
  for (const auto& band : bands) {
    const double band_qlo = std::sqrt(q2s[band.first]);
    const double band_qhi = std::sqrt(q2s[band.second - 1]);
    // geomspace endpoints land on the band edges (seam / global knots); drop
    // them so the remaining samples are strictly inside the band.
    std::vector<double> qs = geomspace(band_qlo, band_qhi, 5);
    for (size_t i = 1; i + 1 < qs.size(); ++i) {
      for (double x : off_x) {
        eval_all("off_knot", x, qs[i] * qs[i], flavors);
      }
    }
  }

  // ---- seam: subgrid Q^2 boundaries. -------------------------------------
  // At each internal seam, emit three probes at a couple of interior x values:
  // the lower-band interior (exercises the band's upper-edge backward Q^2
  // difference), the upper-band interior (its lower-edge forward difference),
  // and exactly at the seam knot (where the two bands must agree, since the
  // value is continuous). With a single subgrid there is no internal seam, so
  // these degenerate to the global QMin/QMax edges instead.
  if (!seam_q2.empty()) {
    const std::vector<double> seam_x = {0.01, 0.2};
    for (size_t b = 0; b + 1 < bands.size(); ++b) {
      const double q2_seam = q2s[bands[b].second - 1];
      // Interior of the lower band's last Q^2 interval.
      const double q2_lo = std::sqrt(q2s[bands[b].second - 2] * q2_seam);
      // Interior of the upper band's first Q^2 interval.
      const double q2_hi = std::sqrt(q2_seam * q2s[bands[b].second + 1]);
      for (double x : seam_x) {
        eval_all("seam", x, q2_lo, flavors);
        eval_all("seam", x, q2_seam, flavors);
        eval_all("seam", x, q2_hi, flavors);
      }
    }
  } else {
    for (double q : {q_min, q_max}) {
      eval_all("seam", 0.05, q * q, flavors);
    }
  }

  // ---- x_to_one_tail: x approaching the x=1 boundary. --------------------
  const double q_mid = 0.5 * (q_min + q_max);
  for (double x : {0.9, 0.99, 0.999, 1.0 - 1e-6}) {
    eval_all("x_to_one_tail", x, q_mid * q_mid, flavors);
  }

  // ---- corner: the four grid corners (probe 0<->21 gluon alias here). -----
  eval_all("corner", x_min, q2_min, probe_pdgs);
  eval_all("corner", x_min, q2_max, probe_pdgs);
  eval_all("corner", x_max, q2_min, probe_pdgs);
  eval_all("corner", x_max, q2_max, probe_pdgs);

  // ---- write JSON --------------------------------------------------------
  FILE* out = std::fopen(out_path.c_str(), "w");
  if (!out) {
    std::fprintf(stderr, "error: cannot open %s for writing\n",
                 out_path.c_str());
    return 1;
  }

  std::fprintf(out, "{\n");
  std::fprintf(out, "  \"set\": \"%s\",\n", set_name.c_str());
  std::fprintf(out, "  \"member\": %d,\n", member);
  std::fprintf(out, "  \"oracle_backend\": \"lhapdf-%s\",\n",
               LHAPDF::version().c_str());
  std::fprintf(out, "  \"num_subgrids\": %zu,\n", num_subgrids);
  std::fprintf(out, "  \"single_subgrid\": %s,\n",
               num_subgrids == 1 ? "true" : "false");
  std::fprintf(out, "  \"seams\": [");
  for (size_t i = 0; i < seam_q2.size(); ++i) {
    write_double(out, seam_q2[i]);
    if (i + 1 < seam_q2.size()) std::fprintf(out, ", ");
  }
  std::fprintf(out, "],\n");
  std::fprintf(out, "  \"subgrids\": [\n");
  for (size_t b = 0; b < bands.size(); ++b) {
    const size_t band_nq = bands[b].second - bands[b].first;
    std::fprintf(out, "    {\"nx\": %zu, \"nq\": %zu, \"flavors\": [", nx,
                 band_nq);
    for (size_t i = 0; i < flavors.size(); ++i) {
      std::fprintf(out, "%d%s", flavors[i],
                   i + 1 < flavors.size() ? ", " : "");
    }
    std::fprintf(out, "]}%s\n", b + 1 < bands.size() ? "," : "");
  }
  std::fprintf(out, "  ],\n");
  std::fprintf(out, "  \"points\": [\n");
  for (size_t i = 0; i < points.size(); ++i) {
    write_point(out, points[i], i + 1 == points.size());
  }
  std::fprintf(out, "  ]\n");
  std::fprintf(out, "}\n");
  std::fclose(out);

  size_t n_knot = 0;
  for (const Point& p : points)
    if (std::string(p.category) == "knot") ++n_knot;
  std::fprintf(stderr,
               "Wrote %s: %zu points (%zu on-knot) via LHAPDF %s\n",
               out_path.c_str(), points.size(), n_knot,
               LHAPDF::version().c_str());
  return 0;
}
