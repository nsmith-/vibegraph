//! The `ckm_no_b_mass` SM restriction, which CMS genproductions bundles in its
//! copy of `models/sm` and MadGraph does not ship.
//!
//! The card is `restrict_ckm.dat` with three values changed: `MB = ymb = 0`, and
//! the tau width `WTau` (`DECAY 15`) at 2.27e-12, the value `parameters.py`
//! itself declares, where MadGraph's card has 0. No MadGraph reference has been
//! evaluated at this restriction, so the model is pinned here by composition:
//! against the interned `ckm` and `no_b_mass` restrictions, and through them
//! against the MadGraph coupling tables banked at the default restriction.
//!
//! The b sector is found by probing, not by comparing restrictions: a coupling
//! belongs to it when moving `MB` and `ymb` moves it at a point where every
//! other parameter is non-zero. Comparing `default` with `no_b_mass` would miss
//! the couplings that are products of `yb` and an off-diagonal CKM element
//! (`I1x31 = yb·conj(CKM3x1)` and its kin): both factors are zero at those two
//! restrictions' common CKM, so neither restriction moves them.
//!
//! # What composition cannot see
//!
//! - **MadGraph's own restriction of this card.** Every comparison here runs this
//!   crate's restriction on both sides. Two of the ways MadGraph's
//!   `RestrictModel` acts differently are closed from the card alone: its
//!   `1e-13` coupling-zero threshold (no coupling at this card lies near either
//!   side's threshold) and its merging of couplings that coincide at the card
//!   (every pair it could merge is an identity). What MadGraph's interaction
//!   split and pruning make of the vertex set is not: that needs its
//!   `display interactions` count at this card. MadGraph checks parameters for
//!   *exact* zero (`detect_special_parameters`), which is why `WTau = 2.27e-12`
//!   is expected to survive; that is a reading of the source, not a run.
//! - **The CKM sector against MadGraph.** No coupling table is banked at `ckm`;
//!   the couplings the Wolfenstein parameters enter are compared only with this
//!   crate's own `ckm` restriction.
//! - **Anything downstream of the couplings.** The tau width enters no coupling,
//!   only the tau propagator; an amplitude or cross-section reference at this
//!   restriction is what would see it.

mod common;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::sync::Arc;

use num_complex::Complex64;
use vibegraph::ufo::identity::digest_bytes;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::sm::{sm_model, sm_parsed_model, SMRestrict};
use vibegraph::ufo::{EvaluatedModel, UFOModel};

/// SHA-256 of `restrict_ckm_no_b_mass.dat` as CMS genproductions bundles it
/// (`mgbasedir/models/sm/`, identical in all 88 gridpacks of the 2026-10 CMS
/// survey, MadGraph 2.4.2 to 2.9.18; read from gridpack `041_WWjj_SS_ewk`).
const CMS_CARD_SHA256: &str = "ad6e3eaf03443bc14148d392fe98deb712de29166061a7408eff521a47591b05";

/// The externals the card sets differently from `restrict_ckm.dat`.
const CHANGED_FROM_CKM: [(&str, f64); 3] = [("MB", 0.0), ("ymb", 0.0), ("WTau", 2.27e-12)];

/// Same bound as the coupling gate's `PYTHON_REL_TOL` (`coupling_oracle.rs`):
/// this crate's couplings against MadGraph's Python on the same card. Also the
/// bound on two evaluations of one coupling expression at a moved card.
const PYTHON_REL_TOL: f64 = 1e-13;

/// MadGraph's `RestrictModel` rounds couplings to ten significant digits before
/// it merges them, so two couplings it could merge agree to better than this;
/// the pair search uses it as a looser bound, which finds a superset of its pairs.
const MERGE_PAIR_REL: f64 = 1e-9;

/// Top of the band where MadGraph's coupling-zero rule (`|g| < 1e-13`, or exact
/// zero once a coupling lies in `[1e-13, 1e-10)`) and this crate's prune
/// (`|g| < 1e-20`) could disagree, widened to the whole `(0, 1e-10)`.
const ZERO_RULE_BAND_TOP: f64 = 1e-10;

fn card(variant: SMRestrict) -> ParamCard {
    variant
        .restrict_card_text()
        .parse()
        .unwrap_or_else(|e| panic!("{variant:?} card does not parse: {e:?}"))
}

/// Every coupling of the coupling table (which restriction does not prune), by
/// name, evaluated on `model` at `card`.
fn couplings(model: &Arc<UFOModel>, card: &str) -> BTreeMap<String, Complex64> {
    let card: ParamCard = card.parse().unwrap();
    let evaluated = EvaluatedModel::from_model_card(Arc::clone(model), &card);
    model
        .couplings
        .keys()
        .map(|name| {
            let id = model.coupling_id(name).unwrap();
            (name.clone(), evaluated.coupling(id))
        })
        .collect()
}

/// The couplings that differ between two evaluations.
fn moved(a: &BTreeMap<String, Complex64>, b: &BTreeMap<String, Complex64>) -> BTreeSet<String> {
    a.iter()
        .filter(|(name, v)| b[*name] != **v)
        .map(|(name, _)| name.clone())
        .collect()
}

/// The couplings `MB` or `ymb` enter: those that move with them under `ckm`,
/// where the CKM matrix is generic and neither is locked.
fn b_sector() -> BTreeSet<String> {
    let ckm = sm_model(SMRestrict::Ckm);
    let b_moved = "Block MASS\n 5 5.3\nBlock YUKAWA\n 5 5.3\n";
    moved(&couplings(&ckm, ""), &couplings(&ckm, b_moved))
}

/// The couplings the Wolfenstein parameters enter, probed the same way.
fn ckm_sector() -> BTreeSet<String> {
    let ckm = sm_model(SMRestrict::Ckm);
    let ckm_moved = "Block Wolfenstein\n 1 0.21\n 2 0.79\n 3 0.15\n 4 0.36\n";
    moved(&couplings(&ckm, ""), &couplings(&ckm, ckm_moved))
}

/// Per surviving vertex, the names of the couplings it carries. The
/// `(colour, Lorentz)` keys are reindexed per restriction, so they are dropped.
fn vertex_couplings(model: &UFOModel) -> BTreeMap<String, BTreeSet<String>> {
    let names: HashMap<_, &String> = model
        .couplings
        .keys()
        .map(|name| (model.coupling_id(name).unwrap(), name))
        .collect();
    model
        .vertices
        .iter()
        .map(|(name, v)| {
            let couplings = v.couplings.values().map(|id| names[id].clone());
            (name.clone(), couplings.collect())
        })
        .collect()
}

/// The vertex set MadGraph's unitary-gauge `sm` import holds is 67 of this
/// crate's 133: the 66 with a ghost or a Goldstone are the ones diagram
/// enumeration also leaves out. MadGraph's `display interactions` count at this
/// card is compared with the 67.
///
/// Blind spot: a count is the same if one vertex is swapped for another, so this
/// pins the predicate's reach, not which vertices survive.
#[test]
fn unitary_gauge_leaves_67_of_133_vertices() {
    let model = sm_model(SMRestrict::CkmNoBMass);
    assert_eq!(model.vertices.len(), 133);
    let with_ghost = model
        .vertices
        .values()
        .filter(|v| v.particles.iter().any(|&p| model.particles[p].is_ghost()))
        .count();
    let goldstone_only = model
        .vertices
        .values()
        .filter(|v| {
            let legs = || v.particles.iter().map(|&p| &model.particles[p]);
            legs().any(|p| p.is_goldstone()) && !legs().any(|p| p.is_ghost())
        })
        .count();
    assert_eq!((with_ghost, goldstone_only), (24, 42));
    assert_eq!(model.unitary_gauge_vertices().count(), 67);
    // A tree-level Feynman-gauge import keeps Goldstones and drops ghosts.
    assert_eq!(model.vertices.len() - with_ghost, 109);
}

#[test]
fn vendored_card_is_the_cms_genproductions_bytes() {
    let text = SMRestrict::CkmNoBMass.restrict_card_text();
    assert_eq!(text.len(), 1350);
    assert_eq!(digest_bytes(text.as_bytes()), CMS_CARD_SHA256);
    assert!(!SMRestrict::CkmNoBMass.ships_with_madgraph());
    assert_eq!(SMRestrict::CkmNoBMass.suffix(), "ckm_no_b_mass");
    assert_eq!(
        SMRestrict::from_suffix(Some("ckm_no_b_mass")),
        Some(SMRestrict::CkmNoBMass)
    );
}

/// Every external the SM defines reads the same from this card as from
/// `restrict_ckm.dat`, except the three it changes.
#[test]
fn card_is_restrict_ckm_with_b_massless_and_a_tau_width() {
    let ours = card(SMRestrict::CkmNoBMass);
    let ckm = card(SMRestrict::Ckm);
    let changed: BTreeMap<&str, f64> = CHANGED_FROM_CKM.into_iter().collect();

    let parsed = sm_parsed_model();
    let mut seen = BTreeSet::new();
    for p in &parsed.params.externals {
        let (lha_block, lha_code) = common::lha_slot(p);
        let value = ours.get(lha_block, lha_code);
        match changed.get(p.name.as_str()) {
            Some(&expected) => {
                assert_eq!(value, Some(expected), "{}", p.name);
                assert_ne!(ckm.get(lha_block, lha_code), Some(expected), "{}", p.name);
            }
            None => assert_eq!(value, ckm.get(lha_block, lha_code), "{}", p.name),
        }
        seen.insert(p.name.as_str());
    }
    assert!(
        changed.keys().all(|name| seen.contains(name)),
        "a changed parameter is not an SM external: {seen:?}"
    );
}

/// A restriction locks exactly the parameters it sets to zero: `MB` and `ymb`
/// join `restrict_ckm`'s locked set, and `WTau`, which `restrict_ckm` locks at
/// zero, stays a non-zero default a param card can move.
#[test]
fn tau_width_survives_and_b_mass_locks() {
    let ours = sm_model(SMRestrict::CkmNoBMass);
    let ckm = sm_model(SMRestrict::Ckm);

    assert!(ckm.params.zeros.contains("WTau"));
    let mut expected = ckm.params.zeros.clone();
    expected.remove("WTau");
    expected.insert("MB".to_owned());
    expected.insert("ymb".to_owned());
    assert_eq!(ours.params.zeros, expected);

    let tau = ours.particle_id("ta-").expect("ta-");
    let b = ours.particle_id("b").expect("b");
    let evaluated = EvaluatedModel::from_model(ours.clone());
    assert_eq!(evaluated.width(tau), 2.27e-12);
    assert_eq!(evaluated.mass(b), 0.0);

    let card: ParamCard = "Block MASS\n 5 4.7\nDECAY 15 1.0e-3\n".parse().unwrap();
    let carded = EvaluatedModel::from_model_card(ours, &card);
    assert_eq!(carded.width(tau), 1.0e-3, "WTau is a default, not a lock");
    assert_eq!(carded.mass(b), 0.0, "MB is locked at zero");
}

/// Every coupling outside the b sector takes `ckm`'s value bit for bit (the
/// same expression on the same inputs in the same process); every coupling in
/// it vanishes, as under `no_b_mass`. No coupling moves with the tau width.
#[test]
fn couplings_compose_ckm_and_no_b_mass() {
    let b_sector = b_sector();
    let ckm_sector = ckm_sector();
    // The probe reaches the couplings that are products of the b Yukawa and an
    // off-diagonal CKM element, which a comparison of restrictions cannot see.
    let both: Vec<&String> = b_sector.intersection(&ckm_sector).collect();
    assert!(both.len() >= 4, "b ∩ CKM: {both:?}");
    println!("b sector {b_sector:?}; of which CKM-dependent {both:?}");

    let model = sm_model(SMRestrict::CkmNoBMass);
    let ours = couplings(&model, "");
    assert!(
        moved(&ours, &couplings(&model, "DECAY 15 1.0e-3\n")).is_empty(),
        "a coupling depends on the tau width"
    );

    let ckm = couplings(&sm_model(SMRestrict::Ckm), "");
    let no_b = couplings(&sm_model(SMRestrict::NoBMass), "");
    for (name, value) in &ours {
        if b_sector.contains(name) {
            assert_eq!(
                *value,
                Complex64::new(0.0, 0.0),
                "{name} survives MB = ymb = 0"
            );
            assert_eq!(*value, no_b[name], "{name} differs from no_b_mass's");
        } else {
            assert_eq!(*value, ckm[name], "{name} differs from ckm's");
        }
    }
}

/// The vertex set is `ckm`'s with the b-sector couplings removed and the
/// vertices left with none dropped.
#[test]
fn vertices_compose_ckm_and_no_b_mass() {
    let b_sector = b_sector();
    let ours = vertex_couplings(&sm_model(SMRestrict::CkmNoBMass));
    let ckm = vertex_couplings(&sm_model(SMRestrict::Ckm));
    let default = vertex_couplings(&sm_model(SMRestrict::Default));
    let no_b = vertex_couplings(&sm_model(SMRestrict::NoBMass));

    let expected: BTreeMap<String, BTreeSet<String>> = ckm
        .iter()
        .map(|(name, ids)| (name.clone(), ids - &b_sector))
        .filter(|(_, ids)| !ids.is_empty())
        .collect();
    assert_eq!(ours, expected);

    // Both halves of the composition act: a vertex `ckm` adds to `default`
    // survives, and every vertex `no_b_mass` drops from `default` is gone.
    assert!(ours.keys().any(|v| !default.contains_key(v)));
    let b_dropped: Vec<&String> = default.keys().filter(|v| !no_b.contains_key(*v)).collect();
    assert!(!b_dropped.is_empty());
    assert!(b_dropped.iter().all(|v| !ours.contains_key(*v)));
}

/// MadGraph's own couplings, banked at the default restriction, hold for this
/// restriction wherever neither the b mass nor the CKM matrix enters.
#[test]
fn madgraph_default_couplings_hold_outside_the_b_and_ckm_sectors() {
    let excluded: BTreeSet<String> = b_sector().union(&ckm_sector()).cloned().collect();
    let model = sm_model(SMRestrict::CkmNoBMass);

    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/couplings");
    let mut tables = 0;
    let mut skipped = BTreeSet::new();
    let mut compared = BTreeSet::new();
    let mut worst = 0.0_f64;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        // Rows that name no model ran against MadGraph's own `sm` at its default
        // restriction.
        if !json["model"].is_null() {
            continue;
        }
        tables += 1;
        let card = common::banked_param_card(&json);
        let evaluated = EvaluatedModel::from_model_card(Arc::clone(&model), &card);
        for (name, z) in json["python"].as_object().unwrap() {
            let id = model.coupling_id(name).expect("MadGraph coupling in model");
            if excluded.contains(name) {
                skipped.insert(name.clone());
                continue;
            }
            let theirs = (z[0].as_f64().unwrap(), z[1].as_f64().unwrap());
            let ours = evaluated.coupling(id);
            let gap = ((ours.re - theirs.0).powi(2) + (ours.im - theirs.1).powi(2)).sqrt();
            let scale = (theirs.0.powi(2) + theirs.1.powi(2)).sqrt();
            let dev = if scale > 0.0 { gap / scale } else { gap };
            assert!(
                dev <= PYTHON_REL_TOL,
                "{}: {name} ours {ours} MadGraph {theirs:?} relative {dev:.3e}",
                path.display()
            );
            worst = worst.max(dev);
            compared.insert(name.clone());
        }
    }
    println!(
        "{tables} tables, {} couplings compared (worst {worst:.2e}), {} skipped as \
         b- or CKM-dependent: {skipped:?}",
        compared.len(),
        skipped.len()
    );
    // The recorded reading is 49 compared and 4 skipped over 25 tables; each
    // table adds couplings, none removes them.
    assert!(
        tables >= 25,
        "only {tables} default-restriction SM coupling tables"
    );
    assert!(
        compared.len() >= 49,
        "only {} couplings compared",
        compared.len()
    );
    assert!(!skipped.is_empty(), "the sector exclusion excluded nothing");
}

/// `variant`'s card with every non-zero external moved off its value by a
/// different factor; a parameter the card sets to zero stays locked there.
fn moved_card(variant: SMRestrict) -> String {
    let card = card(variant);
    let mut moved_card = String::new();
    for (i, p) in sm_parsed_model().params.externals.iter().enumerate() {
        let (lha_block, lha_code) = common::lha_slot(p);
        let Some(value) = card.get(lha_block, lha_code) else {
            continue;
        };
        if value == 0.0 {
            continue;
        }
        let moved = value * (1.0 + 0.0137 * (i as f64 + 1.0));
        let code: Vec<String> = lha_code.iter().map(i32::to_string).collect();
        if lha_block.eq_ignore_ascii_case("decay") {
            moved_card.push_str(&format!("DECAY {} {moved:e}\n", code.join(" ")));
        } else {
            moved_card.push_str(&format!(
                "Block {lha_block}\n {} {moved:e}\n",
                code.join(" ")
            ));
        }
    }
    moved_card
}

/// The couplings a set of externals reaches at all, found by poisoning them:
/// each is set to NaN on a card evaluated afresh, and NaN survives every
/// arithmetic step, including a product with a factor that is zero at the
/// card. This is the structural answer the value probes above approximate.
fn poisoned(model: &Arc<UFOModel>, entries: &str) -> BTreeSet<String> {
    couplings(model, entries)
        .into_iter()
        .filter(|(_, v)| v.re.is_nan() || v.im.is_nan())
        .map(|(name, _)| name)
        .collect()
}

/// The value probes behind [`b_sector`] and [`ckm_sector`] find every coupling
/// `MB`, `ymb` or a Wolfenstein parameter enters that this restriction can make
/// non-zero.
///
/// A probe that moves one parameter misses a coupling whose other factor is
/// zero at the probe point; that is how a comparison of restrictions missed the
/// four couplings that are products of `yb` and an off-diagonal CKM element.
/// Poisoning has no such point. What it finds and the probes do not must carry
/// a factor the restriction locks to zero (`yc·CKM2x1` and its kin: `ymc = 0`
/// in every card the composition reads), so it must stay zero with every free
/// external moved, and no vertex may carry it.
#[test]
fn value_probes_find_every_coupling_their_parameters_enter() {
    let ckm = sm_model(SMRestrict::Ckm);
    let ours = sm_model(SMRestrict::CkmNoBMass);
    let off_card = couplings(&ours, &moved_card(SMRestrict::CkmNoBMass));
    let carried: BTreeSet<String> = [&ckm, &ours]
        .into_iter()
        .flat_map(|m| vertex_couplings(m).into_values().flatten())
        .collect();

    let sectors = [
        (
            "b",
            b_sector(),
            "Block MASS\n 5 nan\nBlock YUKAWA\n 5 nan\n",
        ),
        (
            "CKM",
            ckm_sector(),
            "Block Wolfenstein\n 1 nan\n 2 nan\n 3 nan\n 4 nan\n",
        ),
    ];
    for (sector, probed, poison) in sectors {
        let reached = poisoned(&ckm, poison);
        assert!(
            probed.is_subset(&reached),
            "{sector}: the probe moves {:?}, which poisoning does not reach",
            &probed - &reached
        );
        let missed = &reached - &probed;
        for name in &missed {
            assert_eq!(
                off_card[name],
                Complex64::new(0.0, 0.0),
                "{sector}: {name} is missed by the probe and can be non-zero"
            );
            assert!(
                !carried.contains(name),
                "{sector}: {name} is missed by the probe and a vertex carries it"
            );
        }
        println!(
            "{sector} sector: probe {} couplings, poisoning {}; missed, all locked at zero: {missed:?}",
            probed.len(),
            reached.len()
        );
    }
}

/// MadGraph's zero rule and this crate's select the same couplings at this
/// card, so they prune the same vertices.
///
/// MadGraph's `RestrictModel.detect_identical_couplings` evaluates every
/// coupling at the restrict card and treats `|g| < 1e-13` as zero, unless some
/// coupling lies in `[1e-13, 1e-10)`, in which case it starts again treating
/// only exact zeros as zero. This crate prunes `|g| < 1e-20`. The two rules
/// agree whenever no coupling lies in `[1e-20, 1e-10)`; this asserts the wider
/// `(0, 1e-10)` empty, so the margin is the card's, not the rule's.
#[test]
fn no_coupling_sits_between_the_zero_thresholds() {
    let model = sm_model(SMRestrict::CkmNoBMass);
    let values = couplings(&model, SMRestrict::CkmNoBMass.restrict_card_text());
    let small: Vec<(&String, f64)> = values
        .iter()
        .map(|(name, v)| (name, v.norm()))
        .filter(|&(_, n)| n > 0.0 && n < ZERO_RULE_BAND_TOP)
        .collect();
    assert!(
        small.is_empty(),
        "couplings between the zero thresholds: {small:?}"
    );
    let smallest = values
        .values()
        .map(|v| v.norm())
        .filter(|&n| n > 0.0)
        .fold(f64::INFINITY, f64::min);
    let zeros = values.values().filter(|v| v.norm() == 0.0).count();
    println!(
        "{} couplings at the card: {zeros} exactly zero, smallest non-zero {smallest:.3e}",
        values.len()
    );
}

/// Merging cannot change an amplitude: every pair of couplings MadGraph could
/// fuse at this card is one coupling up to sign off the card too.
///
/// `RestrictModel` replaces each coupling by another of the same coupling
/// orders whose value at the restrict card agrees with it, up to sign, to at
/// least ten significant digits (`limit_to_6_digit`); this crate fuses nothing.
/// Where two such couplings were equal only at the card, MadGraph's model would
/// move them together under a param card that moves either, and this crate's
/// would not. Pairs are found here with a looser bound than MadGraph's
/// rounding, a superset of the pairs in its groups, and each must hold at a
/// point where every free external is moved off the card.
#[test]
fn couplings_madgraph_could_merge_are_identities() {
    let model = sm_model(SMRestrict::CkmNoBMass);
    let at_card = couplings(&model, SMRestrict::CkmNoBMass.restrict_card_text());
    let moved_card = moved_card(SMRestrict::CkmNoBMass);
    let moved_any = moved_card
        .lines()
        .filter(|l| l.starts_with(' ') || l.starts_with("DECAY"))
        .count();
    assert!(moved_any > 10, "only {moved_any} externals moved");
    let off_card = couplings(&model, &moved_card);

    let close =
        |a: Complex64, b: Complex64, tol: f64| (a - b).norm() <= tol * a.norm().max(b.norm());
    let names: Vec<&String> = at_card
        .keys()
        .filter(|n| at_card[*n].norm() > 0.0)
        .collect();
    let mut pairs = 0;
    for (i, a) in names.iter().enumerate() {
        for b in &names[i + 1..] {
            if model.couplings[*a].orders != model.couplings[*b].orders {
                continue;
            }
            let (ga, gb) = (at_card[*a], at_card[*b]);
            let sign = if close(ga, gb, MERGE_PAIR_REL) {
                1.0
            } else if close(ga, -gb, MERGE_PAIR_REL) {
                -1.0
            } else {
                continue;
            };
            pairs += 1;
            let (ma, mb) = (off_card[*a], off_card[*b] * sign);
            assert!(
                ma != at_card[*a] || mb != at_card[*b] * sign,
                "{a}, {b}: the moved card moves neither"
            );
            assert!(
                close(ma, mb, PYTHON_REL_TOL),
                "{a} = {sign:+} {b} at the card ({ga}, {gb}) but not off it ({ma}, {mb})"
            );
        }
    }
    println!("{pairs} coupling pairs MadGraph could merge, every one an identity");
}
