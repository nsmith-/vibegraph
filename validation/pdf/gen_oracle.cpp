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
// Emits the same JSON schema and point categories as the retired Python
// oracle, so the Rust test consumes it unchanged.
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

  // LHAPDF 6.5 flattens all subgrids into one KnotArray. This generator (and
  // the schema it emits) assumes a single rectangular subgrid: verify that
  // nx*nq*nflav fills the grid exactly and that the Q^2 knots are strictly
  // increasing (a repeated Q^2 knot marks a subgrid seam). A future
  // multi-subgrid set must extend this rather than silently mis-describe its
  // structure.
  bool strictly_increasing_q2 = true;
  for (size_t i = 1; i < q2s.size(); ++i) {
    if (!(q2s[i] > q2s[i - 1])) strictly_increasing_q2 = false;
  }
  if (!strictly_increasing_q2) {
    std::fprintf(stderr,
                 "error: %s has repeated Q^2 knots (multiple subgrids); this "
                 "oracle generator only supports a single rectangular "
                 "subgrid.\n",
                 set_name.c_str());
    return 1;
  }
  if (nx != xs.size() || nq != q2s.size()) {
    std::fprintf(stderr, "error: knot vector sizes disagree with shape\n");
    return 1;
  }

  std::vector<Point> points;

  // ---- knot: raw grid values, the pure-parser ground truth. -------------
  // Subsample x and Q^2 knot indices (corners + a few interior) rather than
  // the full nx*nq*nflav product, matching the retired oracle's coverage.
  std::vector<size_t> ix_samples = {0, nx / 3, 2 * nx / 3, nx - 1};
  std::vector<size_t> iq_samples = {0, nq / 2, nq - 1};
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
  const std::vector<double> off_x = geomspace(x_min * 10.0, 0.5, 4);
  const std::vector<double> off_q = geomspace(q_min * 1.3, q_max * 0.7, 4);
  for (double x : off_x) {
    for (double q : off_q) {
      eval_all("off_knot", x, q * q, flavors);
    }
  }

  // ---- seam: subgrid Q^2 boundaries. -------------------------------------
  // With a single subgrid these degenerate to the global QMin/QMax edges
  // (evaluated at a representative interior x); real internal seams need a
  // genuinely multi-subgrid set.
  for (double q : {q_min, q_max}) {
    eval_all("seam", 0.05, q * q, flavors);
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
  std::fprintf(out, "  \"num_subgrids\": 1,\n");
  std::fprintf(out, "  \"single_subgrid\": true,\n");
  std::fprintf(out, "  \"subgrids\": [\n");
  std::fprintf(out, "    {\"nx\": %zu, \"nq\": %zu, \"flavors\": [", nx, nq);
  for (size_t i = 0; i < flavors.size(); ++i) {
    std::fprintf(out, "%d%s", flavors[i],
                 i + 1 < flavors.size() ? ", " : "");
  }
  std::fprintf(out, "]}\n");
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
