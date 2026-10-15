//! MadGraph per-graph facts the amplitude gates share: the diagram pairing of the
//! one row compared both as a MadEvent amplitude table and as a standalone table,
//! and the graphs whose MadGraph value carries a coupling MadGraph's Fortran writes
//! from a seven-significant-digit literal.

/// The MadGraph graph of each diagram of `w+ w- > w+ w- z NP<=1` under `vg_cW`, in
/// enumeration order, 0-based in `AMP()` order: `amplitude_oracle`'s
/// `MG_DIAGRAM_ORDER` entry and `standalone_jamps`'s `MG_GRAPH_DIAGRAMS` entry for
/// the row.
///
/// Derived from MadGraph's per-graph values alone: each diagram goes to the graphs
/// whose `c_k·AMP_k` has overlap above `1 − 1e-10` with it over every banked
/// (point, helicity) entry, and where several qualify — the photon and Z s-channel
/// exchanges of one topology, exactly proportional at every point because their
/// `q²` is `s` — to the one whose ratio has unit modulus, which is unique for every
/// diagram. The standalone table's per-graph gate measures it. The standalone
/// export's amplitude-closing `CALL` lines are textually those of the MadEvent
/// export's `matrix1_orig.f`, so the indices are the MadEvent row's too.
pub const WPWM_TO_WPWMZ_CW_GRAPHS: &[usize] = &[
    0, 1, 52, 51, 50, 49, 48, 56, 55, 57, 54, 53, 108, 107, 106, 105, 104, 112, 111, 113, 110, 109,
    151, 150, 152, 149, 148, 188, 187, 191, 190, 189, 193, 192, 196, 195, 194, 201, 198, 197, 200,
    199, 203, 202, 206, 205, 204, 208, 207, 211, 210, 209, 216, 213, 212, 215, 214, 218, 217, 221,
    220, 219, 37, 34, 33, 36, 35, 32, 31, 44, 41, 40, 43, 42, 39, 38, 47, 46, 45, 20, 17, 16, 19,
    18, 15, 14, 27, 24, 23, 26, 25, 22, 21, 30, 29, 28, 4, 3, 2, 7, 6, 5, 10, 9, 8, 13, 12, 11, 93,
    90, 89, 92, 91, 88, 87, 100, 97, 96, 99, 98, 95, 94, 103, 102, 101, 76, 73, 72, 75, 74, 71, 70,
    83, 80, 79, 82, 81, 78, 77, 86, 85, 84, 60, 59, 58, 63, 62, 61, 66, 65, 64, 69, 68, 67, 141,
    138, 137, 140, 139, 146, 143, 142, 145, 144, 147, 134, 135, 136, 131, 132, 133, 124, 121, 120,
    123, 122, 129, 126, 125, 128, 127, 130, 117, 118, 119, 114, 115, 116, 159, 156, 155, 158, 157,
    154, 153, 166, 163, 162, 165, 164, 161, 160, 169, 168, 167, 180, 177, 176, 179, 178, 185, 182,
    181, 184, 183, 186, 173, 174, 175, 170, 171, 172,
];

/// One coupling MadGraph's Fortran carries rounded, and the graphs it reaches.
pub struct RoundedCoupling {
    /// The coupling, as the UFO and MadGraph name it.
    pub coupling: &'static str,
    /// `|this side| / |MadGraph| − 1` on every graph carrying the coupling: the
    /// ratio of the full-precision coupling to the one the Fortran computes from
    /// seven-digit literals.
    pub deviation: f64,
    /// The graphs carrying it, 0-based in `AMP()` order.
    pub graphs: &'static [usize],
}

const GC_303: &str = "GC_303";
const GC_299: &str = "GC_299";

/// `GC_303 = 2i·gHza/vevhat`. Measured per graph over the 50 graphs that carry it
/// in the rows below: −1.1970234e-8, all within 9e-15 of each other. The
/// `wpwm_to_wpwmz_cw` coupling table's own Python and Fortran values give
/// −1.1970231e-8.
const GC_303_DEVIATION: f64 = -1.197023e-8;

/// `GC_299 = 4i·gHaa/vevhat`. Measured per graph over the 25 graphs that carry it:
/// −2.7695589e-8, all within 4e-15 of each other. No coupling table carries it.
const GC_299_DEVIATION: f64 = -2.769559e-8;

/// Per row, the graphs whose MadGraph value carries SMEFTsim's loop-induced
/// photon–Z–Higgs or photon–photon–Higgs coupling (`VVS4`, coupling order
/// `SMHLOOP`). MadGraph's Fortran writes `gHza` and `gHaa` from UFO literals such
/// as `0.4583333333333333` at seven significant digits, so its `GC_303` and
/// `GC_299` are 1.2e-8 and 2.8e-8 off the value this side and MadGraph's own Python
/// compute, and so is the modulus of every graph carrying them. That is a
/// registered deviation (`madgraph-defect-policy`, approximation kind), and the
/// gates exempt exactly it, by name: a listed graph's modulus is compared at the
/// stated deviation, and its phase and every other graph at the ordinary
/// tolerance.
///
/// Each list is the same set two ways: the graphs whose measured modulus reads the
/// deviation (no other graph reads anything above rounding), and the graphs whose
/// wavefunction chain in MadGraph's standalone `matrix.f` passes through a HELAS
/// call with the coupling.
pub const ROUNDED_COUPLING_GRAPHS: &[(&str, &[RoundedCoupling])] = &[
    (
        "wpwm_to_az_cw",
        &[RoundedCoupling {
            coupling: GC_303,
            deviation: GC_303_DEVIATION,
            graphs: &[3],
        }],
    ),
    (
        "az_to_wpwm_cw",
        &[RoundedCoupling {
            coupling: GC_303,
            deviation: GC_303_DEVIATION,
            graphs: &[3],
        }],
    ),
    (
        "aa_to_wpwm_cw",
        &[RoundedCoupling {
            coupling: GC_299,
            deviation: GC_299_DEVIATION,
            graphs: &[3],
        }],
    ),
    (
        "wpwm_to_wpwmz_cw",
        &[RoundedCoupling {
            coupling: GC_303,
            deviation: GC_303_DEVIATION,
            graphs: &[2, 3, 4, 8, 9, 10, 58, 59, 60, 64, 65, 66],
        }],
    ),
    (
        "wpwm_to_wpwma_cw",
        &[
            RoundedCoupling {
                coupling: GC_303,
                deviation: GC_303_DEVIATION,
                graphs: &[5, 6, 7, 11, 12, 13, 61, 62, 63, 67, 68, 69],
            },
            RoundedCoupling {
                coupling: GC_299,
                deviation: GC_299_DEVIATION,
                graphs: &[2, 3, 4, 8, 9, 10, 58, 59, 60, 64, 65, 66],
            },
        ],
    ),
    (
        "ee_to_wpwmwpwm_cw",
        &[
            RoundedCoupling {
                coupling: GC_303,
                deviation: GC_303_DEVIATION,
                graphs: &[
                    91, 92, 93, 97, 98, 99, 100, 101, 102, 106, 107, 108, 203, 204, 205, 209, 210,
                    211, 212, 213, 214, 218, 219, 220,
                ],
            },
            RoundedCoupling {
                coupling: GC_299,
                deviation: GC_299_DEVIATION,
                graphs: &[88, 89, 90, 94, 95, 96, 200, 201, 202, 206, 207, 208],
            },
        ],
    ),
];

/// The row's registered rounded couplings, empty for a row with none.
pub fn rounded_couplings(key: &str) -> &'static [RoundedCoupling] {
    ROUNDED_COUPLING_GRAPHS
        .iter()
        .find(|(k, _)| *k == key)
        .map_or(&[], |(_, r)| r)
}

/// The factor that takes this side's value of each of `n_graphs` graphs to the
/// modulus MadGraph's rounded couplings give it: `1 / (1 + deviation)` for each
/// registered coupling the graph carries, and 1 on every other graph. A graph
/// listed twice under one coupling, or a graph index out of range, panics.
pub fn modulus_factors(key: &str, n_graphs: usize) -> Vec<f64> {
    let mut factors = vec![1.0; n_graphs];
    for r in rounded_couplings(key) {
        let mut seen = std::collections::BTreeSet::new();
        for &k in r.graphs {
            assert!(
                k < n_graphs,
                "{key}: {} names graph {k} of {n_graphs}",
                r.coupling
            );
            assert!(
                seen.insert(k),
                "{key}: {} names graph {k} twice",
                r.coupling
            );
            factors[k] /= 1.0 + r.deviation;
        }
    }
    factors
}
