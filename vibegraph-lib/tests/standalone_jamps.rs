//! Per-helicity, per-flow amplitudes against MadGraph standalone output.
//!
//! Each table under `validation/madgraph/standalone/` (written by
//! `validation/madgraph/gen_standalone_jamps.py`) carries MadGraph's JAMP(1..NCOLOR)
//! and colour-summed |M|² for every helicity combination of its NHEL table, at a
//! few RAMBO points, together with the param card it computed them with. This side
//! binds the same card, evaluates the same helicities at the same momenta, and
//! compares:
//!
//! * the per-flow amplitudes, flow by flow in MadGraph's colour-basis order, under
//!   one complex constant fitted over every (point, helicity, flow) entry — the
//!   finest linear object MadGraph exposes that is one-to-one with ours;
//! * the per-helicity colour-summed |M|², which the fitted constant must also
//!   explain (it is `|G|²` there).
//!
//! The fitted constant must have modulus one — the two sides normalise amplitudes
//! alike — but its phase is free: external-wavefunction phase conventions differ
//! between the two programs by a process-dependent `±1`/`±i`.
//!
//! A single-flow table may also bank MadGraph's per-graph `AMP()` with JAMP(1)'s
//! coefficient on each. Such a row declares in [`MG_GRAPH_DIAGRAMS`] which of its
//! diagrams make up each MadGraph graph, and each group, compiled one diagram at a
//! time through the production evaluator and summed, must equal `G · c_k · AMP_k`
//! under the same `G`. A group is not always one diagram: see
//! [`MG_GRAPH_DIAGRAMS`] on the four-fermion contacts.
//!
//! A row in `common::mg_graphs::ROUNDED_COUPLING_GRAPHS` carries graphs whose
//! MadGraph value is built on a coupling MadGraph's Fortran writes from a
//! seven-digit literal. Those graphs, named there, are compared at the registered
//! modulus deviation instead of at one: this side's value of each is divided by
//! `1 + deviation` before it is compared, and the flows are compared with the same
//! graphs so divided. Everything else, the phase of those graphs included, is held
//! to [`REL_TOL`]. Such a row must bank `AMP()`, and each of its graphs must also
//! match MadGraph on its own scale ([`GRAPH_RATIO_TOL`]), which is what makes the
//! named list exact in both directions.
//!
//! What it cannot see: that global phase (a sign shared by every diagram of the
//! process); the split of a flow into diagrams, so two compensating errors in
//! diagrams that reach the same flows with the same weights at every point (closed
//! per MadGraph graph where `AMP()` is banked, but not inside a group); and any
//! helicity or kinematic region the few banked points do not visit.
//!
//! Needs only the committed tables, the interned SM model, the vendored SMEFTsim
//! UFO and the authored toy UFO.

mod common;

use std::path::PathBuf;
use std::sync::Arc;

use common::mg_graphs;
use common::{generate_with, sm_model};
use vibegraph::diagrams::DiagramSet;
use vibegraph::helas::eval::{AmplitudeEvaluator, BoundAmplitude};
use vibegraph::helas::repr::C;
use vibegraph::helas::LorentzVector;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::{EvaluatedModel, UFOModel};

/// Relative deviation allowed per entry, against the largest MadGraph amplitude of
/// the table. Both sides round over the same few dozen diagrams, which sits near
/// 1e-15; a sign on one diagram class is O(1).
const REL_TOL: f64 = 1e-12;

/// On a row with registered rounded couplings, the largest `|r_k − 1|` allowed for
/// any graph `k`, where `r_k` is this side's graph over `G · c_k · AMP_k`, fitted
/// over the graph's own entries after the registered moduli are applied.
///
/// [`REL_TOL`] is normalised to the largest graph of the table, so a carrier the
/// list left out passes it unless it is within a factor of about 1e4 of the largest:
/// on `wpwm_to_wpwmz_cw`, leaving out any one of eight of the twelve `GC_303` graphs
/// moves the worst graph to between 1.0e-14 and 1.2e-13 and the worst flow to at
/// most 3.3e-13. This check is normalised to each graph's own size. It separates
/// the rounding a graph carries, measured at most 1.6e-13 over the six rows' 1174
/// graphs, from the smallest registered deviation, 1.2e-8, with two decades to
/// either side, so a carrier left out or a graph named wrongly fails it whatever its
/// size.
const GRAPH_RATIO_TOL: f64 = 1e-10;

fn table_path(key: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/madgraph/standalone")
        .join(format!("{key}.json"))
}

fn complex(v: &serde_json::Value) -> C<f64> {
    C::new(v[0].as_f64().unwrap(), v[1].as_f64().unwrap())
}

/// For a row that banks MadGraph's `AMP()`, the MadGraph graph each of this side's
/// diagrams belongs to, in enumeration order: entry `d` is diagram `d`'s graph,
/// 0-based in `AMP()` order, the convention of `amplitude_oracle`'s
/// `MG_DIAGRAM_ORDER`. The diagrams that share a graph make it up together, and
/// every graph must have at least one.
///
/// `ee_to_ee_4f`: the three four-lepton contacts are three MadGraph graphs and six
/// diagrams here. MadGraph builds each contact on one canonical pairing of the
/// four legs and writes both spinor pairings of its Lorentz structures into that
/// one `AMP()` — its merged ALOHA routine carries the second pairing's terms with
/// the `-1` of `get_sign_flow` — while this side builds one diagram per pairing on
/// that pairing's own fermion lines, and the relative Fermi sign is the diagrams'.
/// Each contact's two diagrams sum to its graph. For the `cll1` and `cee`
/// contacts the two pairings are Fierz-equal, so each diagram is half the graph
/// and the wrong relative sign would cancel the contact; for `cle` they differ.
/// The photon and Z exchanges pair one to one. MadGraph's `AMP(5)`/`AMP(6)` and
/// `AMP(13)`/`AMP(14)` carry the Standard-Model and SMEFT photon couplings on the
/// two vertices in either order and are equal, so the values cannot tell which of
/// each pair is which: those two pairings are banked in enumeration order, and
/// swapping either is invisible here.
///
/// Every other row pairs one diagram to one graph. In `ttx_to_hh_chbox`,
/// `ht_to_ht_chbox` and `hh_to_hh_chbox` several graphs are equal at every point:
/// a Yukawa pair whose two couplings are swapped between its vertices, and the
/// O_Hbox shift of the cubic Higgs coupling against the same shift of the coupling
/// on the other vertex. Within each such set any assignment fits equally, so the
/// one banked is arbitrary and a swap inside the set is invisible.
///
/// The six O_W rows (`wpwm_to_az_cw` through `ee_to_wpwmwpwm_cw`) are paired from
/// MadGraph's values alone: each diagram to the graphs whose `c_k·AMP_k` has overlap
/// above `1 − 1e-10` with it over every banked entry, and where several qualify — the
/// photon and Z s-channel exchanges of one topology are exactly proportional, their
/// `q²` being `s` at every point — to the one whose ratio has unit modulus, unique
/// for every diagram. A registered rounded coupling moves a ratio by 1e-8, far inside
/// the gap between the tied graphs' moduli. The pairing so derived is a bijection on
/// every row, and on `wpwm_to_wpwmz_cw` it is the same from one point as from four.
const MG_GRAPH_DIAGRAMS: &[(&str, &[usize])] = &[
    ("ee_to_ttxh", &[3, 4, 1, 2, 0]),
    ("ee_to_ttxh_identity", &[3, 4, 1, 2, 0]),
    ("ll_to_qqxs_toy_yukawa", TOY_YUKAWA_GRAPHS),
    ("ll_to_ss_toy_dscalar", &[0, 1, 2, 3]),
    ("sl_to_sl_toy_dscalar", &[0, 1, 2, 3]),
    ("ttx_to_hh_chbox", TTX_TO_HH_GRAPHS),
    ("ht_to_ht_chbox", HT_TO_HT_GRAPHS),
    ("hh_to_hh_chbox", HH_TO_HH_GRAPHS),
    ("tata_to_ttxa_tensor4f", TATA_TO_TTXA_TENSOR4F_GRAPHS),
    (
        "ee_to_ee_4f",
        &[
            0, 0, 1, 1, 2, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
        ],
    ),
    ("wpwm_to_az_cw", VVS_PAIR_GRAPHS),
    ("az_to_wpwm_cw", VVS_PAIR_GRAPHS),
    ("aa_to_wpwm_cw", AA_TO_WPWM_CW_GRAPHS),
    ("wpwm_to_wpwmz_cw", mg_graphs::WPWM_TO_WPWMZ_CW_GRAPHS),
    ("wpwm_to_wpwma_cw", WPWM_TO_WPWMA_CW_GRAPHS),
    ("ee_to_wpwmwpwm_cw", EE_TO_WPWMWPWM_CW_GRAPHS),
];

const VVS_PAIR_GRAPHS: &[usize] = &[2, 1, 0, 3, 8, 7, 6, 5, 4, 13, 12, 11, 10, 9];

const AA_TO_WPWM_CW_GRAPHS: &[usize] = &[1, 0, 2, 3, 8, 7, 6, 5, 4, 13, 12, 11, 10, 9];

const WPWM_TO_WPWMA_CW_GRAPHS: &[usize] = &[
    0, 1, 51, 50, 52, 49, 48, 57, 56, 55, 54, 53, 107, 106, 108, 105, 104, 113, 112, 111, 110, 109,
    151, 150, 152, 149, 148, 188, 187, 191, 190, 189, 193, 192, 196, 195, 194, 198, 197, 201, 200,
    199, 206, 203, 202, 205, 204, 208, 207, 211, 210, 209, 213, 212, 216, 215, 214, 221, 218, 217,
    220, 219, 37, 34, 33, 36, 35, 32, 31, 44, 41, 40, 43, 42, 39, 38, 47, 46, 45, 20, 17, 16, 19,
    18, 15, 14, 27, 24, 23, 26, 25, 22, 21, 30, 29, 28, 4, 3, 2, 7, 6, 5, 10, 9, 8, 13, 12, 11, 93,
    90, 89, 92, 91, 88, 87, 100, 97, 96, 99, 98, 95, 94, 103, 102, 101, 76, 73, 72, 75, 74, 71, 70,
    83, 80, 79, 82, 81, 78, 77, 86, 85, 84, 60, 59, 58, 63, 62, 61, 66, 65, 64, 69, 68, 67, 141,
    138, 137, 140, 139, 146, 143, 142, 145, 144, 147, 134, 135, 136, 131, 132, 133, 124, 121, 120,
    123, 122, 129, 126, 125, 128, 127, 130, 117, 118, 119, 114, 115, 116, 159, 156, 155, 158, 157,
    154, 153, 166, 163, 162, 165, 164, 161, 160, 169, 168, 167, 180, 177, 176, 179, 178, 185, 182,
    181, 184, 183, 186, 173, 174, 175, 170, 171, 172,
];

const EE_TO_WPWMWPWM_CW_GRAPHS: &[usize] = &[
    440, 441, 442, 443, 431, 430, 434, 433, 432, 436, 435, 439, 438, 437, 421, 420, 424, 423, 422,
    426, 425, 429, 428, 427, 411, 410, 414, 413, 412, 416, 415, 419, 418, 417, 401, 400, 404, 403,
    402, 406, 405, 409, 408, 407, 319, 318, 320, 315, 312, 339, 338, 337, 334, 331, 364, 363, 362,
    359, 356, 382, 381, 383, 378, 375, 231, 230, 232, 227, 224, 251, 250, 249, 246, 243, 276, 275,
    274, 271, 268, 294, 293, 295, 290, 287, 119, 118, 120, 115, 112, 139, 138, 137, 134, 131, 164,
    163, 162, 159, 156, 182, 181, 183, 178, 175, 7, 6, 8, 3, 0, 27, 26, 25, 22, 19, 52, 51, 50, 47,
    44, 70, 69, 71, 66, 63, 487, 486, 488, 532, 531, 533, 569, 568, 570, 606, 605, 607, 325, 316,
    313, 324, 323, 344, 335, 332, 343, 342, 352, 322, 341, 351, 321, 340, 350, 369, 360, 357, 368,
    367, 388, 379, 376, 387, 386, 396, 366, 385, 395, 365, 384, 394, 237, 228, 225, 236, 235, 256,
    247, 244, 255, 254, 264, 234, 253, 263, 233, 252, 262, 281, 272, 269, 280, 279, 300, 291, 288,
    299, 298, 308, 278, 297, 307, 277, 296, 306, 330, 317, 314, 329, 328, 349, 336, 333, 348, 347,
    355, 327, 346, 354, 326, 345, 353, 374, 361, 358, 373, 372, 393, 380, 377, 392, 391, 399, 371,
    390, 398, 370, 389, 397, 125, 116, 113, 124, 123, 144, 135, 132, 143, 142, 152, 122, 141, 151,
    121, 140, 150, 169, 160, 157, 168, 167, 188, 179, 176, 187, 186, 196, 166, 185, 195, 165, 184,
    194, 130, 117, 114, 129, 128, 149, 136, 133, 148, 147, 155, 127, 146, 154, 126, 145, 153, 174,
    161, 158, 173, 172, 193, 180, 177, 192, 191, 199, 171, 190, 198, 170, 189, 197, 13, 4, 1, 12,
    11, 32, 23, 20, 31, 30, 40, 10, 29, 39, 9, 28, 38, 57, 48, 45, 56, 55, 76, 67, 64, 75, 74, 84,
    54, 73, 83, 53, 72, 82, 242, 229, 226, 241, 240, 261, 248, 245, 260, 259, 267, 239, 258, 266,
    238, 257, 265, 286, 273, 270, 285, 284, 305, 292, 289, 304, 303, 311, 283, 302, 310, 282, 301,
    309, 18, 5, 2, 17, 16, 37, 24, 21, 36, 35, 43, 15, 34, 42, 14, 33, 41, 62, 49, 46, 61, 60, 81,
    68, 65, 80, 79, 87, 59, 78, 86, 58, 77, 85, 90, 89, 88, 93, 92, 91, 96, 95, 94, 99, 98, 97,
    102, 101, 100, 105, 104, 103, 108, 107, 106, 111, 110, 109, 202, 201, 200, 205, 204, 203, 208,
    207, 206, 211, 210, 209, 214, 213, 212, 217, 216, 215, 220, 219, 218, 223, 222, 221, 475, 471,
    469, 474, 473, 483, 479, 477, 482, 481, 485, 458, 454, 452, 457, 456, 466, 462, 460, 465, 464,
    468, 476, 472, 470, 484, 480, 478, 448, 459, 455, 453, 467, 463, 461, 444, 447, 446, 445, 451,
    450, 449, 520, 516, 514, 519, 518, 528, 524, 522, 527, 526, 530, 503, 499, 497, 502, 501, 511,
    507, 505, 510, 509, 513, 521, 517, 515, 529, 525, 523, 493, 504, 500, 498, 512, 508, 506, 489,
    492, 491, 490, 496, 495, 494, 557, 553, 551, 556, 555, 565, 561, 559, 564, 563, 567, 540, 536,
    534, 539, 538, 548, 544, 542, 547, 546, 550, 594, 590, 588, 593, 592, 602, 598, 596, 601, 600,
    604, 577, 573, 571, 576, 575, 585, 581, 579, 584, 583, 587, 620, 618, 616, 626, 624, 622, 612,
    614, 608, 610, 640, 638, 636, 646, 644, 642, 632, 634, 628, 630, 660, 658, 656, 666, 664, 662,
    652, 654, 648, 650, 680, 678, 676, 686, 684, 682, 672, 674, 668, 670, 661, 653, 649, 681, 673,
    669, 659, 679, 657, 677, 667, 655, 651, 687, 675, 671, 665, 685, 663, 683, 621, 613, 609, 641,
    633, 629, 619, 639, 617, 637, 627, 615, 611, 647, 635, 631, 625, 645, 623, 643, 578, 574, 572,
    586, 582, 580, 541, 537, 535, 549, 545, 543, 595, 591, 589, 603, 599, 597, 558, 554, 552, 566,
    562, 560,
];

/// Each of the 16 diagrams equals one graph at unit modulus, uniquely.
const TATA_TO_TTXA_TENSOR4F_GRAPHS: &[usize] =
    &[9, 13, 14, 15, 3, 4, 5, 0, 1, 2, 6, 7, 8, 10, 11, 12];

const TOY_YUKAWA_GRAPHS: &[usize] = &[
    10, 11, 12, 14, 13, 15, 16, 18, 17, 19, 0, 1, 2, 4, 3, 5, 6, 8, 7, 9, 20, 21, 23, 22, 24, 25,
    26, 28, 27, 29, 30, 31, 33, 32, 34, 35, 36, 38, 37, 39,
];

const TTX_TO_HH_GRAPHS: &[usize] = &[0, 2, 1, 5, 6, 3, 4, 7, 10, 11, 8, 9, 12, 15, 16, 13, 14];

const HT_TO_HT_GRAPHS: &[usize] = &[0, 3, 4, 1, 2, 5, 10, 11, 9, 8, 6, 7, 12, 15, 16, 13, 14];

const HH_TO_HH_GRAPHS: &[usize] = &[
    0, 2, 1, 3, 4, 5, 11, 10, 12, 13, 7, 6, 8, 9, 14, 20, 19, 21, 22, 16, 15, 17, 18, 23, 29, 28,
    30, 31, 25, 24, 26, 27,
];

/// The per-graph level of a row that banks `AMP()`.
struct GraphLevel {
    /// JAMP(1)'s coefficient on each `AMP()`.
    coefs: Vec<f64>,
    /// The diagrams that make up each graph, in `AMP()` order.
    groups: Vec<Vec<usize>>,
    /// One evaluator per diagram, each compiled from a one-diagram set.
    per_diagram: Vec<AmplitudeEvaluator>,
    /// What each graph's value on this side is multiplied by before it is compared:
    /// `1 / (1 + deviation)` on a graph a registered rounded coupling reaches, 1
    /// elsewhere ([`mg_graphs::modulus_factors`]).
    factors: Vec<f64>,
    /// Whether any factor differs from 1, so the flows are compared with the same
    /// graphs divided.
    rounded: bool,
}

impl GraphLevel {
    /// Reads the row's [`MG_GRAPH_DIAGRAMS`] entry against its table, two-way: a
    /// table that banks `AMP()` must be declared there, and a declared row must bank
    /// them. The entry must give every diagram a graph and every one of the NGRAPHS
    /// graphs a diagram, which is the diagram-count convention stated as a check.
    ///
    /// `pairing` and `factors`, where given, replace the declared pairing and the
    /// registered modulus factors, for the checks that the gate sees a wrong one.
    fn read(
        key: &str,
        table: &serde_json::Value,
        set: &DiagramSet,
        model: &UFOModel,
        pairing: Option<&[usize]>,
        factors: Option<Vec<f64>>,
    ) -> Option<Self> {
        let rounded_declared = !mg_graphs::rounded_couplings(key).is_empty();
        let declared = MG_GRAPH_DIAGRAMS
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, g)| pairing.unwrap_or(g));
        let (coefs, graph_of) = match (table["jamp_coefficients"].as_array(), declared) {
            (None, None) => {
                assert!(
                    !rounded_declared,
                    "{key}: registers rounded-coupling graphs but its table banks no AMP()"
                );
                return None;
            }
            (Some(coefs), Some(graph_of)) => (coefs, graph_of),
            (Some(_), None) => panic!("{key}: banks AMP() but declares no MG_GRAPH_DIAGRAMS"),
            (None, Some(_)) => {
                panic!("{key}: declares MG_GRAPH_DIAGRAMS but its table banks no AMP()")
            }
        };
        let coefs: Vec<f64> = coefs.iter().map(|c| c.as_f64().unwrap()).collect();
        let n_graphs = table["n_graphs"].as_u64().unwrap() as usize;
        assert_eq!(coefs.len(), n_graphs, "{key}: one coefficient per AMP()");
        assert_eq!(
            graph_of.len(),
            set.diagrams.len(),
            "{key}: MG_GRAPH_DIAGRAMS must name one graph for each diagram"
        );
        let mut groups = vec![Vec::new(); n_graphs];
        for (d, &k) in graph_of.iter().enumerate() {
            assert!(
                k < n_graphs,
                "{key}: diagram {d} names graph {k} of {n_graphs}"
            );
            groups[k].push(d);
        }
        let empty: Vec<usize> = (0..n_graphs).filter(|&k| groups[k].is_empty()).collect();
        assert!(empty.is_empty(), "{key}: graphs {empty:?} have no diagram");
        let factors = factors.unwrap_or_else(|| mg_graphs::modulus_factors(key, n_graphs));
        assert_eq!(
            factors.len(),
            n_graphs,
            "{key}: one modulus factor per graph"
        );
        let rounded = factors.iter().any(|&f| f != 1.0);
        if rounded {
            // The flow correction below subtracts the rounded graphs' share from the
            // one flow; a multi-flow row would need it per flow.
            assert_eq!(
                table["n_flows"].as_u64(),
                Some(1),
                "{key}: rounded-coupling graphs are compared on single-flow rows only"
            );
        }
        let per_diagram = set
            .diagrams
            .iter()
            .map(|d| {
                AmplitudeEvaluator::compile(
                    &DiagramSet {
                        particles_in: set.particles_in.clone(),
                        particles_out: set.particles_out.clone(),
                        polarizations: set.polarizations.clone(),
                        diagrams: vec![d.clone()],
                    },
                    model,
                )
                .unwrap()
            })
            .collect();
        Some(GraphLevel {
            coefs,
            groups,
            per_diagram,
            factors,
            rounded,
        })
    }

    /// The amount this side's one flow carries above what MadGraph's rounded
    /// couplings give it: `Σ_k (1 − factor_k) · (graph k)` over this side's
    /// per-diagram values.
    fn rounding_excess(&self, diagram_values: &[C<f64>]) -> C<f64> {
        self.groups
            .iter()
            .zip(&self.factors)
            .filter(|(_, &f)| f != 1.0)
            .map(|(group, &f)| group.iter().map(|&d| diagram_values[d]).sum::<C<f64>>() * (1.0 - f))
            .sum()
    }

    /// `(c_k · AMP_k, graph k's diagrams summed and multiplied by its modulus factor,
    /// k, where)` for each graph of one helicity entry, from MadGraph's `amps` and
    /// this side's per-diagram values. Empty where the entry banks no `AMP()`.
    fn entries(
        &self,
        key: &str,
        amps: &serde_json::Value,
        diagram_values: &[C<f64>],
        what: &str,
    ) -> Vec<(C<f64>, C<f64>, usize, String)> {
        let Some(amps) = amps.as_array() else {
            return Vec::new();
        };
        assert_eq!(
            amps.len(),
            self.coefs.len(),
            "{key}: {what}: one AMP() per graph"
        );
        amps.iter()
            .zip(&self.groups)
            .enumerate()
            .map(|(k, (amp, group))| {
                let mg = complex(amp) * self.coefs[k];
                let vg: C<f64> =
                    group.iter().map(|&d| diagram_values[d]).sum::<C<f64>>() * self.factors[k];
                (
                    mg,
                    vg,
                    k,
                    format!("{what}, AMP({}) = diagrams {group:?}", k + 1),
                )
            })
            .collect()
    }
}

/// The model a table was computed in: the interned SM, the model and restrict card
/// of the manifest row it names, or (`"ufo"`) the committed model directory and
/// restrict card it records itself, for a model card no manifest row carries.
fn table_model(key: &str, table: &serde_json::Value) -> Arc<UFOModel> {
    match table["model"].as_str().unwrap() {
        "sm" => sm_model(),
        "ufo" => {
            let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
            let ufo = &table["ufo"];
            let dir = repo.join(ufo["dir"].as_str().unwrap());
            let card = repo.join(ufo["restrict_card"].as_str().unwrap());
            UFOModel::load(&dir, Some(&card))
                .unwrap_or_else(|e| panic!("{key}: load {}: {e}", dir.display()))
        }
        row => common::model_for_row(row).unwrap_or_else(|e| panic!("{key}: {e}")),
    }
}

struct Measured {
    g: C<f64>,
    worst_flow: f64,
    worst_flow_where: String,
    worst_m2: f64,
    worst_m2_where: String,
    worst_graph: f64,
    worst_graph_where: String,
    /// The largest per-graph `|r_k − 1|` ([`GRAPH_RATIO_TOL`]), on a row with
    /// registered rounded couplings; zero elsewhere.
    worst_graph_ratio: f64,
    worst_graph_ratio_where: String,
    /// Per registered rounded coupling, its graphs' measured `|this side| /
    /// |MadGraph| − 1`, before the registered deviation is applied.
    rounded: Vec<String>,
    n_entries: usize,
    n_graph_entries: usize,
}

impl Measured {
    /// Every way the comparison can disagree with MadGraph, empty when it agrees.
    fn disagreements(&self, key: &str) -> Vec<String> {
        let tol = REL_TOL;
        let mut out = Vec::new();
        if self.worst_flow >= tol {
            out.push(format!(
                "{key}: {} ({:.3e} of the largest amplitude)",
                self.worst_flow_where, self.worst_flow
            ));
        }
        if self.worst_m2 >= tol {
            out.push(format!(
                "{key}: {} ({:.3e})",
                self.worst_m2_where, self.worst_m2
            ));
        }
        if self.worst_graph >= tol {
            out.push(format!(
                "{key}: {} ({:.3e} of the largest graph)",
                self.worst_graph_where, self.worst_graph
            ));
        }
        if self.worst_graph_ratio >= GRAPH_RATIO_TOL {
            out.push(format!(
                "{key}: {} (|r - 1| = {:.3e} of the graph itself)",
                self.worst_graph_ratio_where, self.worst_graph_ratio
            ));
        }
        if (self.g.norm() - 1.0).abs() >= tol {
            out.push(format!(
                "{key}: the flows agree only up to |G| = {}",
                self.g.norm()
            ));
        }
        out
    }
}

/// Reads the table, binds its card and evaluates every listed entry. A table that
/// cannot be read, or a process whose colour or helicity structure is not the
/// table's, panics: those are not disagreements with MadGraph, and a row listed in
/// [`KNOWN_DISAGREEMENT`] must not absorb them. The deviations themselves are
/// returned for [`check`] to judge.
fn measure(key: &str) -> Measured {
    measure_with(key, |_| {}, None, None)
}

/// [`measure`] on the table's model after `alter` has edited a private copy of it,
/// and under `pairing` and `factors` in place of the row's declared graph pairing
/// and registered modulus factors where they are given.
fn measure_with(
    key: &str,
    alter: impl FnOnce(&mut UFOModel),
    pairing: Option<&[usize]>,
    factors: Option<Vec<f64>>,
) -> Measured {
    let path = table_path(key);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let table: serde_json::Value = serde_json::from_str(&text).unwrap();
    let model = table_model(key, &table);
    let mut altered = (*model).clone();
    alter(&mut altered);
    let model = Arc::new(altered);
    let process = table["process"].as_str().unwrap();
    let card: ParamCard = table["param_card"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l.as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        .parse()
        .unwrap();
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &card);
    let set = generate_with(process, model.as_ref()).remove(0);
    let evaluator = AmplitudeEvaluator::compile(&set, model.as_ref()).unwrap();
    let n_flows = table["n_flows"].as_u64().unwrap() as usize;
    assert_eq!(evaluator.n_flows(), n_flows, "{key}: NCOLOR");
    let bound = BoundAmplitude::<f64>::bind(&evaluator, &evaluated);
    let mut scratch = bound.scratch_space();

    let helicities: Vec<Vec<i32>> = table["helicities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| {
            h.as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_i64().unwrap() as i32)
                .collect()
        })
        .collect();
    let graphs = GraphLevel::read(key, &table, &set, model.as_ref(), pairing, factors);
    let per_diagram_bound: Vec<BoundAmplitude<f64>> = graphs
        .iter()
        .flat_map(|level| level.per_diagram.iter())
        .map(|ev| BoundAmplitude::<f64>::bind(ev, &evaluated))
        .collect();

    let ours: std::collections::BTreeSet<&Vec<i32>> = evaluator.helicities().iter().collect();
    let theirs: std::collections::BTreeSet<&Vec<i32>> = helicities.iter().collect();
    assert_eq!(
        ours, theirs,
        "{key}: the helicity sums run over different sets"
    );

    // (MadGraph, ours) per flow entry, and per-helicity |M|² pairs.
    let mut flows: Vec<(C<f64>, C<f64>, String)> = Vec::new();
    let mut m2: Vec<(f64, f64, String)> = Vec::new();
    // (c_k · AMP_k, the group's summed diagrams, k) per graph entry.
    let mut graph_entries: Vec<(C<f64>, C<f64>, usize, String)> = Vec::new();
    // Per graph, `(Σ conj(c·AMP)·ours, Σ |c·AMP|²)` over its entries before the
    // modulus factor, for the measured deviation of the rounded graphs.
    let mut raw_ratio: Vec<(C<f64>, f64)> = Vec::new();
    for (pi, point) in table["points"].as_array().unwrap().iter().enumerate() {
        let momenta: Vec<LorentzVector<f64>> = point["momenta"]
            .as_array()
            .unwrap()
            .iter()
            .map(|k| {
                let c: Vec<f64> = k
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_f64().unwrap())
                    .collect();
                LorentzVector::new(c[0], c[1], c[2], c[3])
            })
            .collect();
        for (hel, entry) in helicities.iter().zip(point["helicity"].as_array().unwrap()) {
            let mg: Vec<C<f64>> = entry["jamps"]
                .as_array()
                .unwrap()
                .iter()
                .map(complex)
                .collect();
            let mut vg = if n_flows == 1 {
                vec![bound.eval_amplitude(&momenta, hel, &mut scratch)]
            } else {
                bound.run_flows(&momenta, hel, &mut scratch)
            };
            let diagram_values: Vec<C<f64>> = per_diagram_bound
                .iter()
                .map(|b| b.eval_amplitude(&momenta, hel, &mut b.scratch_space()))
                .collect();
            if let Some(level) = graphs.as_ref().filter(|l| l.rounded) {
                vg[0] -= level.rounding_excess(&diagram_values);
            }
            for (f, (a, b)) in mg.iter().zip(&vg).enumerate() {
                flows.push((*a, *b, format!("point {pi}, helicities {hel:?}, flow {f}")));
            }
            let cf = evaluator.cf_matrix();
            let mut ours_m2 = 0.0;
            for i in 0..n_flows {
                for j in 0..n_flows {
                    let c =
                        *cf[i * n_flows + j].numer() as f64 / *cf[i * n_flows + j].denom() as f64;
                    ours_m2 += c * (vg[i].conj() * vg[j]).re;
                }
            }
            m2.push((
                entry["m2"].as_f64().unwrap(),
                ours_m2,
                format!("point {pi}, helicities {hel:?}"),
            ));
            if let Some(level) = &graphs {
                let entries = level.entries(
                    key,
                    &entry["amps"],
                    &diagram_values,
                    &format!("point {pi}, helicities {hel:?}"),
                );
                raw_ratio.resize(level.factors.len(), (C::new(0.0, 0.0), 0.0));
                for (mg, vg, k, _) in &entries {
                    let raw = vg / level.factors[*k];
                    raw_ratio[*k].0 += mg.conj() * raw;
                    raw_ratio[*k].1 += mg.norm_sqr();
                }
                graph_entries.extend(entries);
            }
        }
    }

    let (mut num, mut den, mut scale) = (C::new(0.0, 0.0), 0.0f64, 0.0f64);
    for (mg, vg, _) in &flows {
        num += mg.conj() * vg;
        den += mg.norm_sqr();
        scale = scale.max(mg.norm());
    }
    let g = num / den;
    let (mut worst_flow, mut worst_flow_where) = (0.0f64, String::new());
    for (mg, vg, what) in &flows {
        let dev = (vg - g * mg).norm() / scale;
        if dev > worst_flow {
            worst_flow = dev;
            worst_flow_where = format!(
                "{what}: vibegraph {vg:?} against G·MadGraph {:?} (G = {g:?})",
                g * mg
            );
        }
    }
    let m2_scale = m2.iter().map(|e| e.0.abs()).fold(0.0, f64::max);
    let (mut worst_m2, mut worst_m2_where) = (0.0f64, String::new());
    for (mg, vg, what) in &m2 {
        let dev = (vg - g.norm_sqr() * mg).abs() / m2_scale;
        if dev > worst_m2 {
            worst_m2 = dev;
            worst_m2_where = format!(
                "{what}: |M|² {vg:e} against |G|²·MadGraph {:e}",
                g.norm_sqr() * mg
            );
        }
    }
    let graph_scale = graph_entries.iter().map(|e| e.0.norm()).fold(0.0, f64::max);
    let (mut worst_graph, mut worst_graph_where) = (0.0f64, String::new());
    for (mg, vg, _, what) in &graph_entries {
        let dev = (vg - g * mg).norm() / graph_scale;
        if dev > worst_graph {
            worst_graph = dev;
            worst_graph_where = format!("{what}: vibegraph {vg:?} against G·c·AMP {:?}", g * mg);
        }
    }
    if graphs.is_some() {
        assert!(graph_scale > 0.0, "{key}: every banked AMP() vanishes");
    }
    // Each graph on its own scale, on a row whose graph list the rounded couplings
    // name: `r_k = Σ conj(G·c·AMP)·ours / Σ |G·c·AMP|²` over the graph's entries.
    let (mut worst_graph_ratio, mut worst_graph_ratio_where) = (0.0f64, String::new());
    let mut rounded = Vec::new();
    if let Some(level) = graphs.as_ref().filter(|l| l.rounded) {
        let mut sums = vec![(C::new(0.0, 0.0), 0.0f64); level.factors.len()];
        for (mg, vg, k, _) in &graph_entries {
            let theirs = g * mg;
            sums[*k].0 += theirs.conj() * vg;
            sums[*k].1 += theirs.norm_sqr();
        }
        for (k, (num, den)) in sums.iter().enumerate() {
            assert!(
                *den > 0.0,
                "{key}: AMP({}) vanishes at every banked entry, so its modulus is unchecked",
                k + 1
            );
            let r = num / den;
            let dev = (r - 1.0).norm();
            if dev > worst_graph_ratio {
                worst_graph_ratio = dev;
                worst_graph_ratio_where = format!(
                    "AMP({}) = diagrams {:?}: this side over G·c·AMP is {r:?} (modulus factor {})",
                    k + 1,
                    level.groups[k],
                    level.factors[k]
                );
            }
        }
        for r in mg_graphs::rounded_couplings(key) {
            let measured: Vec<f64> = r
                .graphs
                .iter()
                .map(|&k| (raw_ratio[k].0 / raw_ratio[k].1).norm() / g.norm() - 1.0)
                .collect();
            let (lo, hi) = measured
                .iter()
                .fold((f64::MAX, f64::MIN), |(lo, hi), &m| (lo.min(m), hi.max(m)));
            rounded.push(format!(
                "{} on {} graphs {:?}: |this side| / |MadGraph| - 1 measured \
                 {lo:.9e} .. {hi:.9e}, registered {:.6e}",
                r.coupling,
                r.graphs.len(),
                r.graphs.iter().map(|k| k + 1).collect::<Vec<_>>(),
                r.deviation
            ));
        }
    }
    Measured {
        g,
        worst_flow,
        worst_flow_where,
        worst_m2,
        worst_m2_where,
        worst_graph,
        worst_graph_where,
        worst_graph_ratio,
        worst_graph_ratio_where,
        rounded,
        n_entries: flows.len(),
        n_graph_entries: graph_entries.len(),
    }
}

/// Rows whose comparison is known to disagree with MadGraph, with the finding. A listed
/// row is measured in full and reported; the test fails if it starts agreeing, so the
/// exemption cannot outlive the defect. The exemption covers the measured deviations
/// only: a listed row whose table is missing or whose process stops matching the
/// table's colour and helicity structure fails like any other.
const KNOWN_DISAGREEMENT: &[(&str, &str)] = &[];

fn check(key: &str) {
    let known = KNOWN_DISAGREEMENT.iter().find(|(k, _)| *k == key);
    let m = measure(key);
    println!(
        "{key}: {} flow entries, G = {:?}, worst flow {:.3e}, worst |M|² {:.3e}",
        m.n_entries, m.g, m.worst_flow, m.worst_m2
    );
    if m.n_graph_entries > 0 {
        println!(
            "{key}: {} graph entries, worst graph {:.3e}",
            m.n_graph_entries, m.worst_graph
        );
    }
    if !m.rounded.is_empty() {
        println!(
            "{key}: worst graph on its own scale {:.3e}",
            m.worst_graph_ratio
        );
        for line in &m.rounded {
            println!("{key}: {line}");
        }
    }
    let disagreements = m.disagreements(key);
    match (disagreements.is_empty(), known) {
        (true, None) => {}
        (false, None) => panic!("{}", disagreements.join("\n")),
        (true, Some((_, why))) => panic!(
            "{key} is listed as disagreeing ({why}) but now agrees: G = {:?}, worst flow \
             {:.3e}; remove it from KNOWN_DISAGREEMENT",
            m.g, m.worst_flow
        ),
        (false, Some((_, why))) => println!(
            "{key}: informational, disagrees as listed: {why}\n  {}",
            disagreements.join("\n  ")
        ),
    }
}

/// Five gluons: ten of the 25 diagrams join a four-gluon contact to a triple-gluon
/// vertex, the smallest multiplicity where a contact is not the whole diagram.
#[test]
fn gg_to_ggg() {
    check("gg_to_ggg");
}

/// A quark line at leg 0 beside triple-gluon vertices and a four-gluon contact.
#[test]
fn uux_to_ggg() {
    check("uux_to_ggg");
}

/// Quark-gluon scattering with the quark first: a triple-gluon vertex beside the
/// quark line holding leg 0.
#[test]
fn ug_to_ug() {
    check("ug_to_ug");
}

/// Colourless four-vector contacts beside a lepton line holding leg 0.
#[test]
fn ee_to_wpwmz() {
    check("ee_to_wpwmz");
}

/// A colourless four-vector contact holding leg 0, beside photon, Z and Higgs exchange.
#[test]
fn wpwm_to_wpwm() {
    check("wpwm_to_wpwm");
}

/// O_HG's gluon-pair–scalar vertex producing the s-channel Higgs current, interfering
/// with QCD.
#[test]
fn ttx_to_gg_chg() {
    check("ttx_to_gg_chg");
}

/// A W pair at leg 0 beside a final-state lepton line: the neutrino exchange against
/// the photon and Z s-channel, whose two kinds of diagram hold leg 0 at a two-fermion
/// vertex and at the WWV vertex.
#[test]
fn wpwm_to_epem() {
    check("wpwm_to_epem");
}

/// The same graphs with both pairs of legs listed in the other order.
#[test]
fn wmwp_to_emep() {
    check("wmwp_to_emep");
}

/// A quark line in place of the lepton line: the down-quark exchange against the
/// s-channel.
#[test]
fn wpwm_to_uux() {
    check("wpwm_to_uux");
}

/// A colourless vector at leg 0 on a mixed lepton line: the W exchange holds leg 0 at
/// the WWZ vertex, the other two diagrams at a lepton vertex.
#[test]
fn wpem_to_zve() {
    check("wpem_to_zve");
}

/// The photon at leg 0: the s-channel W through the AWW vertex beside the electron
/// exchange.
#[test]
fn aem_to_wmve() {
    check("aem_to_wmve");
}

/// The W at leg 0 of `W+ H > t b~`: the quark exchanges hold it at the Wtb vertex,
/// on a final-state quark pair whose other vertex is a Yukawa.
#[test]
fn wph_to_tbx() {
    check("wph_to_tbx");
}

/// The same graphs with the Higgs at leg 0, held at the Yukawa vertex.
#[test]
fn hwp_to_tbx() {
    check("hwp_to_tbx");
}

/// A Z pair at leg 0: the s-channel Higgs, a ZZH vertex producing it and the top
/// Yukawa closing it, beside the top exchange.
#[test]
fn zz_to_ttx() {
    check("zz_to_ttx");
}

/// A Higgs pair at leg 0: the triple-Higgs vertex beside the top exchange.
#[test]
fn hh_to_ttx() {
    check("hh_to_ttx");
}

/// A gluon pair at leg 0: the triple-gluon vertex beside the quark exchange, the
/// coloured analogue of [`wpwm_to_uux`].
#[test]
fn gg_to_uux() {
    check("gg_to_uux");
}

/// 2 → 3 with a W pair at leg 0: a Z off the final-state lepton line, off the
/// s-channel boson, or from a WWZ or WWZZ vertex.
#[test]
fn wpwm_to_epemz() {
    check("wpwm_to_epemz");
}

/// 2 → 3 with a W pair at leg 0 and a gluon off the final-state quark line.
#[test]
fn wpwm_to_uuxg() {
    check("wpwm_to_uuxg");
}

/// O_W's four-vector contacts at leg 0, beside photon, Z and Higgs exchange through
/// O_W's triple-gauge structures.
#[test]
fn wpwm_to_wpwm_cw() {
    check("wpwm_to_wpwm_cw");
}

/// O_W's WWZZ contact beside the W exchange and the s-channel Higgs.
#[test]
fn wpwm_to_zz_cw() {
    check("wpwm_to_zz_cw");
}

/// The photon–Z–Higgs vertex producing the off-shell Higgs from the external photon
/// and Z, beside O_W's WWAZ contact.
#[test]
fn wpwm_to_az_cw() {
    check("wpwm_to_az_cw");
}

/// The photon–Z–Higgs vertex as the amplitude, with the photon at leg 0.
#[test]
fn az_to_wpwm_cw() {
    check("az_to_wpwm_cw");
}

/// The photon–photon–Higgs vertex as the amplitude.
#[test]
fn aa_to_wpwm_cw() {
    check("aa_to_wpwm_cw");
}

/// O_W's two five-vector contacts, each the whole of one diagram, and the
/// photon–Z–Higgs vertex producing a Higgs or a photon current.
#[test]
fn wpwm_to_wpwmz_cw() {
    check("wpwm_to_wpwmz_cw");
}

/// The photon twin: the WWWW-photon five-vector contacts and the photon–photon–Higgs
/// vertex beside the photon–Z–Higgs one.
#[test]
fn wpwm_to_wpwma_cw() {
    check("wpwm_to_wpwma_cw");
}

/// The five-vector contacts as sources: the s-channel photon or Z off the electron
/// line ends on a WWWW-photon or WWWW-Z contact.
#[test]
fn ee_to_wpwmwpwm_cw() {
    check("ee_to_wpwmwpwm_cw");
}

/// A gluon off the initial quark line of a four-quark vector contact, which produces
/// the off-shell quark current the emission closes, in both colour contractions.
#[test]
fn uux_to_ttxg_4f() {
    check("uux_to_ttxg_4f");
}

/// A photon off either line of the tensor four-fermion contact (`O_leQt3`), read
/// through the tensor path as the current the emission closes; compared per
/// MadGraph graph.
#[test]
fn tata_to_ttxa_tensor4f() {
    check("tata_to_ttxa_tensor4f");
}

/// The tau Yukawa's s-channel Higgs beside Higgsstrahlung and the top Yukawa: the
/// diagrams carrying both Yukawas, one of them through the Higgs self-coupling.
#[test]
fn tata_to_ttxh() {
    check("tata_to_ttxh");
}

/// Two Higgs bosons: final-state top lines built of Yukawa vertices alone, across
/// zero, one and two internal propagators, interfering in one sum.
#[test]
fn tata_to_ttxhh() {
    check("tata_to_ttxhh");
}

/// The all-Yukawa `b` line: t/u-channel exchange across one propagator beside the
/// triple-Higgs annihilation, which must interfere at MadGraph's relative sign.
#[test]
fn bbx_to_hh() {
    check("bbx_to_hh");
}

/// Bhabha scattering through the four-lepton contacts: each contact's two spinor
/// pairings interfere with each other and with photon and Z exchange in both
/// channels, compared per MadGraph graph as well as per helicity.
#[test]
fn ee_to_ee_4f() {
    check("ee_to_ee_4f");
}

/// The top Yukawa as MadGraph's `sm` writes it, `FFS4 = ProjM + ProjP`, with the
/// Higgs emitted off the final-state top pair beside Higgsstrahlung: the control
/// for the bare-`Identity` spelling of the same Yukawa.
#[test]
fn ee_to_ttxh() {
    check("ee_to_ttxh");
}

/// SMEFTsim's bare `Identity(2,1)` top Yukawa on the crossed top line: the four
/// Higgs-off-top graphs against Higgsstrahlung, the same relative sign the
/// `ProjM + ProjP` spelling of [`ee_to_ttxh`] has.
#[test]
fn ee_to_ttxh_identity() {
    check("ee_to_ttxh_identity");
}

/// The toy model's bare `Identity` and `Gamma5` Yukawas emitted off a final-state
/// quark pair and off the initial lepton line, beside the vector exchange.
#[test]
fn ll_to_qqxs_toy_yukawa() {
    check("ll_to_qqxs_toy_yukawa");
}

/// The derivative cubic scalar vertex producing the s-channel scalar, beside its
/// plain partner.
#[test]
fn ll_to_ss_toy_dscalar() {
    check("ll_to_ss_toy_dscalar");
}

/// The derivative cubic scalar vertex at the t-channel scalar exchange, with a
/// scalar on leg 0, beside its plain partner.
#[test]
fn sl_to_sl_toy_dscalar() {
    check("sl_to_sl_toy_dscalar");
}

/// SMEFTsim's derivative Higgs self-couplings (`SSS2`, `SSS3`) as the s-channel
/// Higgs current off an initial top pair.
#[test]
fn ttx_to_hh_chbox() {
    check("ttx_to_hh_chbox");
}

/// SMEFTsim's derivative Higgs self-couplings (`SSS2`, `SSS3`) as the amplitude of
/// the t-channel Higgs exchange, with a Higgs on leg 0.
#[test]
fn ht_to_ht_chbox() {
    check("ht_to_ht_chbox");
}

/// SMEFTsim's derivative quartic Higgs contacts (`SSSS2`, `SSSS3`) and derivative
/// cubic vertices beside their plain partners, in four-Higgs scattering.
#[test]
fn hh_to_hh_chbox() {
    check("hh_to_hh_chbox");
}

/// The gluon-initiated `t t~ + jet` subprocess: a four-gluon contact beside the top
/// line, whose three colour structures each multiply one of its Lorentz structures.
#[test]
fn gg_to_ttxg() {
    check("gg_to_ttxg");
}

/// Quark-gluon `t t~ + jet` with the quark first: the light-quark line and the top
/// pair joined by a gluon, the radiated gluon on either line or at a triple-gluon
/// vertex.
#[test]
fn ug_to_ttxu() {
    check("ug_to_ttxu");
}

/// The same channel in MadEvent's own leg order, the gluon first.
#[test]
fn gu_to_ttxu() {
    check("gu_to_ttxu");
}

/// The antiquark-gluon channel in MadEvent's leg order.
#[test]
fn gux_to_ttxux() {
    check("gux_to_ttxux");
}

/// Quark annihilation into `t t~ + g`: the gluon off either quark line or off the
/// s-channel gluon.
#[test]
fn uux_to_ttxg() {
    check("uux_to_ttxg");
}

/// The gate is sensitive to which colour structure of the four-gluon contact
/// multiplies which Lorentz structure: each of the five non-identity assignments
/// disagrees with `gg_to_ttxg` far above [`REL_TOL`]. Measured over the five, the
/// weakest worst-flow deviation is 4.4e-2 and the weakest |M|² deviation 2.9e-2, so
/// the 1e-2 floor is a factor of three below the weakest. The identity assignment
/// is the banked agreement at 1e-15.
#[test]
fn gg_to_ttxg_sees_the_contact_colour_assignment() {
    let permutations = [[1, 2, 0], [2, 0, 1], [1, 0, 2], [0, 2, 1], [2, 1, 0]];
    for perm in permutations {
        let m = measure_with(
            "gg_to_ttxg",
            |model| {
                let gluon = model.particle_id("g").unwrap();
                let contacts: Vec<_> = model
                    .vertices
                    .values_mut()
                    .filter(|v| v.particles.len() == 4 && v.particles.iter().all(|&p| p == gluon))
                    .collect();
                assert_eq!(contacts.len(), 1, "the SM has one four-gluon contact");
                let contact = &mut *contacts.into_iter().next().unwrap();
                contact.couplings = std::mem::take(&mut contact.couplings)
                    .into_iter()
                    .map(|((c, l), id)| ((perm[c], l), id))
                    .collect();
            },
            None,
            None,
        );
        assert!(
            m.worst_flow > 1e-2 && m.worst_m2 > 1e-2,
            "colour permutation {perm:?}: worst flow {:.3e}, worst |M|² {:.3e}",
            m.worst_flow,
            m.worst_m2
        );
    }
}

/// Every row that registers rounded-coupling graphs has a standalone table, so no
/// registration outlives the table it was measured on.
#[test]
fn rounded_coupling_rows_have_tables() {
    for (key, _) in mg_graphs::ROUNDED_COUPLING_GRAPHS {
        assert!(
            table_path(key).is_file(),
            "{key} registers rounded-coupling graphs and has no standalone table"
        );
    }
}

/// The per-graph gate of `wpwm_to_wpwmz_cw` sees a wrong pairing where the values
/// alone could not tell two graphs apart by shape: AMP(15) and AMP(22) are the
/// photon and Z exchanges of one topology, exactly proportional at every point, and
/// swapping the two diagrams paired with them fails on their modulus ratio of 3.6.
#[test]
fn wpwm_to_wpwmz_cw_sees_a_swapped_pairing() {
    let mut pairing = mg_graphs::WPWM_TO_WPWMZ_CW_GRAPHS.to_vec();
    let a = pairing.iter().position(|&k| k == 14).unwrap();
    let b = pairing.iter().position(|&k| k == 21).unwrap();
    pairing.swap(a, b);
    let m = measure_with("wpwm_to_wpwmz_cw", |_| {}, Some(&pairing), None);
    println!(
        "swapped AMP(15)/AMP(22): worst graph {:.3e}, worst flow {:.3e}",
        m.worst_graph, m.worst_flow
    );
    assert!(
        m.worst_graph > 1e-3,
        "swapped AMP(15)/AMP(22): worst graph {:.3e}",
        m.worst_graph
    );
}

/// The rounded-coupling list of `wpwm_to_wpwmz_cw` is exact: leaving out any one of
/// its twelve `GC_303` graphs, or naming a graph that does not carry the coupling
/// (the five-vector contact AMP(1)), fails, whatever the size of the graph against
/// the largest of the table.
#[test]
fn wpwm_to_wpwmz_cw_sees_a_wrong_rounded_list() {
    let n_graphs = mg_graphs::WPWM_TO_WPWMZ_CW_GRAPHS.len();
    let registered = mg_graphs::modulus_factors("wpwm_to_wpwmz_cw", n_graphs);
    let carriers: Vec<usize> = (0..n_graphs).filter(|&k| registered[k] != 1.0).collect();
    assert_eq!(carriers.len(), 12, "the twelve GC_303 graphs");
    let mut mutations: Vec<(String, Vec<f64>)> = carriers
        .iter()
        .map(|&k| {
            let mut factors = registered.clone();
            factors[k] = 1.0;
            (format!("AMP({}) left out", k + 1), factors)
        })
        .collect();
    let mut extra = registered.clone();
    extra[0] = registered[carriers[0]];
    mutations.push(("AMP(1) named".to_owned(), extra));
    for (what, factors) in mutations {
        let m = measure_with("wpwm_to_wpwmz_cw", |_| {}, None, Some(factors));
        let failures = m.disagreements("wpwm_to_wpwmz_cw");
        println!(
            "{what}: worst graph {:.3e}, on its own scale {:.3e}, worst flow {:.3e}",
            m.worst_graph, m.worst_graph_ratio, m.worst_flow
        );
        assert!(
            m.worst_graph_ratio > GRAPH_RATIO_TOL && !failures.is_empty(),
            "{what}: the gate does not see it (on its own scale {:.3e})",
            m.worst_graph_ratio
        );
    }
}
