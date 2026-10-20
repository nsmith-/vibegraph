//! The manifest rows that evaluate in the interned Standard Model: how a row's
//! `model` and `restrict` resolve to it, and what MadGraph's own generated
//! parameter card says the restriction left behind.
//!
//! `model = "sm"` with `restrict = "<suffix>"` is `import model sm-<suffix>`:
//! MadGraph resolves it from its own `models/sm`, and this crate from the
//! interned SM's restrict cards, so the gate stays hermetic. The resolution
//! tests run on restrictions MadGraph ships, so they need no banked row; the
//! manifest sweep holds every row that uses the spelling to a restriction the
//! interned SM has.
//!
//! What they cannot see: whether the MadGraph tree that generated a row carried
//! the same card bytes as the interned one. For a card MadGraph ships, the
//! `sm_interned_blob` gate compares the two; for one it does not, the card's
//! SHA-256 is pinned beside it and the generator's copy is the banking run's
//! responsibility.

mod common;

use std::path::Path;
use std::sync::Arc;

use common::manifest::{RowModel, INTERNED_SM};
use vibegraph::ufo::parameters::ParamNature;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::sm::{sm_model, SMRestrict};
use vibegraph::ufo::EvaluatedModel;

fn row(restrict: Option<&str>) -> RowModel {
    RowModel {
        dir: INTERNED_SM.to_owned(),
        restrict: restrict.map(str::to_owned),
    }
}

/// The spelling selects the interned variant itself, not a model built from a
/// file: the same cached instance `sm_model` hands out.
#[test]
fn interned_sm_row_loads_the_named_restriction() {
    let no_b = common::load_row_model(&row(Some("no_b_mass"))).expect("sm-no_b_mass");
    assert!(Arc::ptr_eq(&no_b, &sm_model(SMRestrict::NoBMass)));
    assert!(!Arc::ptr_eq(&no_b, &sm_model(SMRestrict::Default)));
    assert_ne!(
        no_b.vertices.len(),
        sm_model(SMRestrict::Default).vertices.len(),
        "no_b_mass prunes the b Yukawa vertices the default keeps"
    );

    let default = common::load_row_model(&row(None)).expect("sm");
    assert!(Arc::ptr_eq(&default, &sm_model(SMRestrict::Default)));

    let cms = common::load_row_model(&row(Some("ckm_no_b_mass"))).expect("sm-ckm_no_b_mass");
    assert!(Arc::ptr_eq(&cms, &sm_model(SMRestrict::CkmNoBMass)));
}

/// A restriction the interned SM does not have is a refusal naming it, never a
/// silent fall-back to the default restriction or a search for a file.
#[test]
fn unknown_interned_sm_restriction_is_refused() {
    let Err(err) = common::load_row_model(&row(Some("ckm_no_b_mas"))) else {
        panic!("a misspelt restriction must not load");
    };
    assert!(err.contains("`ckm_no_b_mas`"), "{err}");
    assert!(err.contains("ckm_no_b_mass"), "{err}");
}

/// Every manifest row that names the interned SM names a restriction it has.
///
/// A row whose restriction did not resolve would fail only inside whichever gate
/// first loads its model, as that gate's failure; here it fails as itself.
#[test]
fn manifest_rows_naming_the_interned_sm_resolve() {
    let rows: Vec<(String, RowModel)> = common::manifest::row_models()
        .into_iter()
        .filter(|(_, m)| m.is_interned_sm())
        .collect();
    for (key, model) in &rows {
        common::load_row_model(model).unwrap_or_else(|e| panic!("[{key}] {e}"));
    }
    println!("{} manifest rows name the interned SM", rows.len());
}

/// The relative rounding of MadGraph's generated `param_card.dat`, which prints
/// every value at seven significant digits (`%e`): `WH = 6.38233934e-03` in the
/// restrict card is `6.382339e-03` there.
const CARD_PRINT_REL: f64 = 5e-7;

/// Every interned-SM row's restriction defaults to the values MadGraph's own
/// generated `param_card.dat` carries, external by external.
///
/// The card MadGraph writes for a process is the restricted model's baseline: a
/// parameter the restriction keeps free is printed at the restrict card's value,
/// and one it locks at zero is printed as a zero dependent entry. So this is the
/// check on which parameters MadGraph's restriction locks, at the level of the
/// parameter itself: `ckm_no_b_mass` keeping `WTau = 2.27e-12` where
/// `restrict_ckm` locks it to zero is a `DECAY 15` line reading 2.27e-12 rather
/// than 0, and `MB = ymb = 0` a zero `MASS 5` and an absent `YUKAWA 5`. It reads
/// the cards banked beside the amplitude tables, so a row's restriction is
/// checked here from the day its table is.
///
/// Blind to a parameter MadGraph's card omits altogether, to a difference below
/// the card's seven printed digits, and to `aS`, which a hadronic run rewrites.
#[test]
fn interned_sm_defaults_are_madgraphs_generated_param_card() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/amplitudes");
    let models = common::manifest::row_models();
    let mut tables = 0;
    let mut compared = 0;
    let mut by_restriction = std::collections::BTreeMap::<String, usize>::new();
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let key = json["key"].as_str().expect("table key");
        let restriction = match models.get(key) {
            None => SMRestrict::Default.suffix().to_owned(),
            Some(row) if row.is_interned_sm() => {
                row.restrict.clone().unwrap_or_else(|| "default".to_owned())
            }
            Some(_) => continue,
        };
        let model = common::model_for_row(key).unwrap_or_else(|e| panic!("[{key}] {e}"));
        let card: ParamCard = json["param_card"]
            .as_array()
            .unwrap_or_else(|| panic!("[{key}] the table banks no param card"))
            .iter()
            .map(|l| l.as_str().expect("param card line"))
            .collect::<Vec<_>>()
            .join("\n")
            .parse()
            .unwrap_or_else(|e| panic!("[{key}] banked param card: {e:?}"));
        let evaluated = EvaluatedModel::from_model(model.clone());
        let mut here = 0;
        for p in &model.params.externals {
            let ParamNature::External {
                lha_block,
                lha_code,
                ..
            } = &p.nature
            else {
                unreachable!("externals are external")
            };
            // The card a hadronic run leaves carries its PDF set's alpha_s(M_Z)
            // in `aS` (`pp_to_ll_qcd0`'s reads 0.13 where the restriction's is
            // 0.118), so the card's `aS` is the run's. No SM restriction locks it.
            if p.name == "aS" {
                continue;
            }
            let Some(want) = card.get(lha_block, lha_code) else {
                continue;
            };
            let got = evaluated.param_values[&p.name].re;
            assert!(
                (got - want).abs() <= CARD_PRINT_REL * got.abs(),
                "[{key}] {} defaults to {got} under `{restriction}` where MadGraph's \
                 generated card says {want}",
                p.name
            );
            here += 1;
        }
        assert!(here >= 15, "[{key}] only {here} externals compared");
        compared += here;
        tables += 1;
        *by_restriction.entry(restriction).or_default() += 1;
    }
    println!("{tables} interned-SM tables, {compared} externals compared: {by_restriction:?}");
    assert!(tables >= 20, "only {tables} interned-SM amplitude tables");
}
