//! MadGraph's per-process edits to its LO run-card template, for a scattering
//! process: `banner.py`'s `RunCardLO.create_default_for_process`
//! (`banner.py:4767-5065`), which its `output` runs before it writes
//! `Cards/run_card.dat`.
//!
//! The card this builds is the one MadEvent reads back at `launch`, not the one
//! `create_default_for_process` holds in memory. The two differ on a hidden
//! parameter: `RunCard.write` writes a hidden parameter only when it is
//! user-set or its block is displayed (`banner.py:3015`), and an assignment in
//! `create_default_for_process` marks nothing user-set, so a hidden parameter
//! whose block stays hidden reads back at its template value. That is why
//! `asrwgtflavor` follows `maxjetflavor` only under matching (the `mlm` block
//! is displayed then), and why `sys_alpsfact` keeps the template's `None`
//! though the matching rule sets it.
//!
//! Every rule of `create_default_for_process` for two incoming particles, and
//! what this does with it:
//!
//! - **Parton beams** (`:4806-4810`): `maxjetflavor` is the heaviest incoming
//!   quark, at least 4, when an incoming parton is a quark up to b, a gluon or
//!   a photon. Reproduced.
//! - **Lepton–proton beams** (`:4812-4826`): `lpp = (0, 1)` or `(1, 0)`, the
//!   lepton beam at 1 TeV and the proton at 6.5 TeV. Reproduced; the card is
//!   then refused as [`RunCardError::UnsupportedLpp`](super::RunCardError).
//! - **Photon–photon UPC** (`:4829-4834`): never taken. It compares the set of
//!   beam ids with the list `[[22], [22]]`, which no set equals, so a
//!   photon–photon process keeps proton beams in MadGraph too.
//! - **Lepton beams** (`:4836-4844`): `lpp = 0`, `ebeam = 500` on both beams
//!   and `use_syst = F`, when an incoming particle is an electron or a muon and
//!   none is a parton. Reproduced.
//! - **EVA beams** (`:4849-4894`): a W or Z on both beams gives `lpp = (-3, 3)`
//!   at 15 TeV, `nhel = 1`, `pdlabel = eva` and a fixed factorisation scale; on
//!   one beam, `pdlabel` of that beam is `eva`, its factorisation scale fixed,
//!   `nhel = 1`, and `lpp = ±3` (electron) or `±4` (muon) when the other beam
//!   holds a lepton. Reproduced; the card is then refused, as
//!   `UnsupportedLpp`, or at proton beams as
//!   [`RunCardError::AsymmetricBeamPdf`](super::RunCardError).
//! - **Neutrino beams** (`:4900-4927`): a neutrino beam gets `lpp = 0`, 1 TeV
//!   and `polbeam = -100` (`+100` for an antineutrino). Reproduced; the card is
//!   then refused on `polbeam` or, beside a proton beam, as `UnsupportedLpp`.
//! - **MLM matching** (`:4929-4967`): see [`matches_by_jets`]. Reproduced:
//!   `ickkw = 1`, `xqcut = 30`, `drjj = drjl = 0`, `--alps=0.5,1,2` appended to
//!   `systematics_arguments` and `dynamical_scale_choice = -1`, with
//!   `asrwgtflavor` and `pdgs_for_merging_cut` shown with the `mlm` block.
//!   `drjj`, a cut on jet pairs, is hidden on a card with fewer than two jets
//!   and reads back at 0.4 there, a value that then cuts nothing; zeroing it
//!   is exact wherever it is read.
//! - **Interference** (`:4969-4991`, the `^2` syntax): unreachable, since the
//!   proc card refuses squared-order constraints.
//! - **`sde_strategy`** (`:4993-5014`, `:5055-5059`): see
//!   [`ProcessFacts::qcd_free_single_color`]. Reproduced.
//! - **Polarized-frame display** (`:5026-5040`): display only; `me_frame` keeps
//!   its value.
//! - **The `MLM` model limitation** (`:5048-5053`): see
//!   [`ProcessFacts::non_qcd_gluon_emission`]. Reproduced.
//! - **The `fix_scale` model limitation** (`:5061-5066`, a massless coloured
//!   scalar in the model): not reproduced. MadGraph fixes both scales and
//!   then refuses at `launch` any card that does not
//!   (`common_run_interface.py:6563`); with no run card such a model is
//!   refused before the card is built (`config::LoadRunCardError::FixedScaleModel`).
//! - **Cut-class display** (`:5068-5099`) and the other display blocks only
//!   choose which lines are written; what they change is the read-back rule
//!   above.

use std::collections::{BTreeMap, BTreeSet};

use super::ParamValue;

/// What `create_default_for_process` reads off a scattering process, as
/// MadGraph's `output` hands it the process: one entry per generated matrix
/// element with a diagram.
#[derive(Clone, Debug, Default)]
pub struct ProcessFacts {
    /// The incoming PDG ids of every generated subprocess, by beam: the first
    /// incoming leg's on beam 1, the second's on beam 2.
    pub beams: [BTreeSet<i64>; 2],
    /// Each generated subprocess's legs as PDG ids, the two incoming first,
    /// in generation order. For a decay chain these are the legs of its core,
    /// before any decay: MadGraph reads `proc[0]['legs']`, which a decay chain
    /// keeps undecayed.
    pub core_legs: Vec<Vec<i64>>,
    /// The absolute PDG ids of the model's coloured particles, ghosts
    /// excluded, ascending (`proc_characteristic['colored_pdgs']`,
    /// `export_v4.py:5027`, over a model MadGraph imports without ghosts).
    pub coloured_pdgs: Vec<i64>,
    /// Some subprocess uses a coupling of a vertex that emits a gluon without
    /// a QCD coupling order among coloured particles only: MadGraph's `MLM`
    /// limitation (`import_ufo.py:1836-1842`, `export_v4.py:4779-4791`). It
    /// turns matching back off and a CKKW scale (`-1`) into `3`.
    pub non_qcd_gluon_emission: bool,
    /// Every process line bounds `QCD` at 0, and the colour basis of every
    /// subprocess has a single power of `Nc` (`proc_characteristic
    /// ['single_color']`, `export_v4.py:1340`). Together with a final state
    /// that is not all leptons and photons from partonic beams, that is
    /// MadGraph's condition for `sde_strategy = 2`. A line with no `QCD`
    /// constraint does not count as 0: MadGraph tests the key in the line's
    /// own orders.
    pub qcd_free_single_color: bool,
    /// Some process line forbids an on-shell s-channel (`$`), which forces
    /// `sde_strategy = 1`.
    pub onshell_veto: bool,
}

const PARTONS: [i64; 12] = [1, -1, 2, -2, 3, -3, 4, -4, 5, -5, 21, 22];
const CHARGED_LEPTONS: [i64; 4] = [11, -11, 13, -13];
const NEUTRINOS: [i64; 6] = [12, -12, 14, -14, 16, -16];
const EVA_BOSONS: [i64; 3] = [23, 24, -24];
const JETS: [i64; 11] = [1, -1, 2, -2, 3, -3, 4, -4, 5, -5, 21];

fn holds_any(ids: &BTreeSet<i64>, of: &[i64]) -> bool {
    of.iter().any(|id| ids.contains(id))
}

/// Whether a card of mixed multiplicities differs by jets only, MadGraph's
/// trigger for MLM matching (`banner.py:4929-4952`).
///
/// It reads the legs of each matrix element's first process. Unless every one
/// has the same number of legs, the first with the fewest is compared with each
/// with the most: one occurrence of each of its ids is struck from the larger
/// one's, and matching is on when what is left of some larger one is all light
/// quarks and gluons. "First" is in generation order, which can differ from
/// MadGraph's order of its matrix elements; the choice only matters when the
/// least multiplicity holds subprocesses that differ in more than jets.
pub(super) fn matches_by_jets(core_legs: &[Vec<i64>]) -> bool {
    let (Some(min), Some(max)) = (
        core_legs.iter().map(Vec::len).min(),
        core_legs.iter().map(Vec::len).max(),
    ) else {
        return false;
    };
    if min == max {
        return false;
    }
    let ids_min = core_legs
        .iter()
        .find(|legs| legs.len() == min)
        .expect("the minimum is some subprocess's");
    core_legs
        .iter()
        .filter(|legs| legs.len() == max)
        .any(|legs| {
            let mut rest = legs.clone();
            for id in ids_min {
                if let Some(at) = rest.iter().position(|r| r == id) {
                    rest.remove(at);
                }
            }
            rest.iter().all(|id| JETS.contains(id))
        })
}

/// Apply the edits to the LO template's `values`.
pub(super) fn apply(values: &mut BTreeMap<String, ParamValue>, facts: &ProcessFacts) {
    let mut set = |name: &str, value: ParamValue| {
        assert!(
            values.contains_key(name),
            "'{name}' is a template parameter"
        );
        values.insert(name.to_string(), value);
    };
    let [beam1, beam2] = &facts.beams;
    let beam_ids: BTreeSet<i64> = beam1.union(beam2).copied().collect();

    let mut maxjetflavor = 4;
    if holds_any(&beam_ids, &PARTONS) {
        maxjetflavor = beam_ids
            .iter()
            .map(|id| id.abs())
            .filter(|&id| id < 7)
            .fold(4, i64::max);
        set("maxjetflavor", ParamValue::Int(maxjetflavor));
        if holds_any(&beam_ids, &CHARGED_LEPTONS) {
            let (lpp, ebeam) = if holds_any(beam1, &CHARGED_LEPTONS) {
                ([0, 1], [1000.0, 6500.0])
            } else {
                ([1, 0], [6500.0, 1000.0])
            };
            set_beams(&mut set, lpp, ebeam);
        }
    } else if holds_any(&beam_ids, &CHARGED_LEPTONS) {
        set_beams(&mut set, [0, 0], [500.0, 500.0]);
        set("use_syst", ParamValue::Bool(false));
    }

    let eva = [holds_any(beam1, &EVA_BOSONS), holds_any(beam2, &EVA_BOSONS)];
    match eva {
        [true, true] => {
            set_beams(&mut set, [-3, 3], [15000.0, 15000.0]);
            set("nhel", ParamValue::Int(1));
            set("pdlabel", ParamValue::Str("eva".to_string()));
            set("fixed_fac_scale", ParamValue::Bool(true));
        }
        [true, false] | [false, true] => {
            let (side, other) = if eva[0] { (1, beam2) } else { (2, beam1) };
            set(
                &format!("pdlabel{side}"),
                ParamValue::Str("eva".to_string()),
            );
            set(&format!("fixed_fac_scale{side}"), ParamValue::Bool(true));
            set("nhel", ParamValue::Int(1));
            // MadGraph walks the other beam's ids as a Python set and keeps the
            // last lepton it meets; either one leaves a beam type refused here.
            for &id in other {
                let kind = match id.abs() {
                    11 => 3,
                    13 => 4,
                    _ => continue,
                };
                let lepton_side = kind * id.signum();
                let lpp = if side == 1 {
                    [-lepton_side, lepton_side]
                } else {
                    [lepton_side, -lepton_side]
                };
                set_beams(&mut set, lpp, [15000.0, 15000.0]);
            }
        }
        [false, false] => {}
    }

    if holds_any(&beam_ids, &NEUTRINOS) {
        for (side, beam) in [(1, beam1), (2, beam2)] {
            let polbeam = if holds_any(beam, &[12, 14, 16]) {
                -100.0
            } else if holds_any(beam, &[-12, -14, -16]) {
                100.0
            } else {
                continue;
            };
            set(&format!("lpp{side}"), ParamValue::Int(0));
            set(&format!("ebeam{side}"), ParamValue::Float(1000.0));
            set(&format!("polbeam{side}"), ParamValue::Float(polbeam));
        }
    }

    let matched = matches_by_jets(&facts.core_legs);
    if matched {
        set("ickkw", ParamValue::Int(1));
        set("xqcut", ParamValue::Float(30.0));
        set("drjj", ParamValue::Float(0.0));
        set("drjl", ParamValue::Float(0.0));
        set("dynamical_scale_choice", ParamValue::Int(-1));
        // Hidden parameters of the `mlm` and `ckkw` blocks, which matching
        // displays, so the card is written with them.
        set("asrwgtflavor", ParamValue::Int(maxjetflavor));
        let pdgs = facts
            .coloured_pdgs
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        set("pdgs_for_merging_cut", ParamValue::Opaque(pdgs));
    }

    if facts.qcd_free_single_color {
        let jet_ids = |id: i64| id == 21 || (1..=maxjetflavor).contains(&id);
        let pure_lepton = facts
            .core_legs
            .iter()
            .all(|legs| legs[2..].iter().all(|id| matches!(id.abs(), 11..=16 | 22)));
        let proton_initial = facts
            .core_legs
            .iter()
            .all(|legs| legs[..2].iter().all(|id| jet_ids(id.abs())));
        if !(pure_lepton && proton_initial) {
            set("SDE_strategy", ParamValue::Int(2));
        }
    }

    if facts.non_qcd_gluon_emission {
        set("ickkw", ParamValue::Int(0));
    }
    if facts.onshell_veto {
        set("SDE_strategy", ParamValue::Int(1));
    }

    if matched {
        append_alps(values);
    }
    if facts.non_qcd_gluon_emission && values["dynamical_scale_choice"].as_i64() == -1 {
        values.insert("dynamical_scale_choice".to_string(), ParamValue::Int(3));
    }
}

fn set_beams(set: &mut impl FnMut(&str, ParamValue), lpp: [i64; 2], ebeam: [f64; 2]) {
    for (side, (lpp, ebeam)) in [1, 2].into_iter().zip(lpp.into_iter().zip(ebeam)) {
        set(&format!("lpp{side}"), ParamValue::Int(lpp));
        set(&format!("ebeam{side}"), ParamValue::Float(ebeam));
    }
}

/// `systematics_arguments.append('--alps=0.5,1,2')`, on the list as the card
/// writes it.
fn append_alps(values: &mut BTreeMap<String, ParamValue>) {
    let list = values["systematics_arguments"].as_str();
    let inner = list
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim();
    let appended = if inner.is_empty() {
        "['--alps=0.5,1,2']".to_string()
    } else {
        format!("[{inner}, '--alps=0.5,1,2']")
    };
    values.insert(
        "systematics_arguments".to_string(),
        ParamValue::Opaque(appended),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// MadGraph's comparison strikes one occurrence per id and asks the rest to
    /// be jets. Each case would read differently under a rule that compared
    /// multiplicities alone, struck every occurrence, or let a W pass as a jet.
    #[test]
    fn matching_needs_the_extra_legs_to_be_jets() {
        let ee = vec![2, -2, -11, 11];
        let eej = vec![21, 2, -11, 11, 2];
        assert!(matches_by_jets(&[ee.clone(), eej.clone()]));
        assert!(!matches_by_jets(&[ee.clone(), ee.clone()]));
        assert!(!matches_by_jets(std::slice::from_ref(&eej)));
        assert!(!matches_by_jets(&[]));
        // A photon is no jet.
        assert!(!matches_by_jets(&[ee.clone(), vec![2, -2, -11, 11, 22]]));
        // A W more than the smaller process, with the jet: not by jets only.
        assert!(!matches_by_jets(&[
            vec![2, -1, 24],
            vec![2, -2, 24, -24, 21]
        ]));
        // The top is no jet either.
        assert!(!matches_by_jets(&[ee.clone(), vec![21, 2, -11, 11, 6]]));
        // One larger subprocess differing by jets is enough.
        let eea = vec![2, -2, -11, 11, 22];
        assert!(matches_by_jets(&[ee.clone(), eea, eej]));
        // `u u~ > u u~` against `u u~ > u u~ u u~`: the two extra u are jets once
        // one of each id is struck.
        assert!(matches_by_jets(&[
            vec![2, -2, 2, -2],
            vec![2, -2, 2, -2, 2, -2]
        ]));
        // The comparison is against the first subprocess of least multiplicity:
        // a W- process first makes the W+ one differ by more than jets.
        assert!(!matches_by_jets(&[
            vec![1, -2, -24],
            vec![2, -1, 24],
            vec![2, -1, 24, 21]
        ]));
    }

    #[test]
    fn alps_is_appended_to_the_argument_list() {
        let mut values = BTreeMap::new();
        values.insert(
            "systematics_arguments".to_string(),
            ParamValue::Opaque("['--mur=0.5,1,2', '--pdf=errorset']".to_string()),
        );
        append_alps(&mut values);
        assert_eq!(
            values["systematics_arguments"],
            ParamValue::Opaque("['--mur=0.5,1,2', '--pdf=errorset', '--alps=0.5,1,2']".into())
        );
        values.insert(
            "systematics_arguments".to_string(),
            ParamValue::Opaque(String::new()),
        );
        append_alps(&mut values);
        assert_eq!(
            values["systematics_arguments"],
            ParamValue::Opaque("['--alps=0.5,1,2']".into())
        );
    }
}
