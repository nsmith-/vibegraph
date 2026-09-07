//! The committed interned SM model against a fresh parse of the pinned
//! `mg5amcnlo` submodule.
//!
//! `src/ufo/sm_assets/` holds a zstd+bincode blob of the parsed SM UFO, written
//! by the `gen_sm_blob` dev binary, plus every restrict card verbatim. That blob
//! is what a bare clone gets and what every hermetic test reads, so the blob —
//! not the submodule — is the hermetic truth. What the blob cannot tell anyone
//! is whether it still agrees with the source it was built from, which is what
//! these two comparisons are: one on the evaluated physics, one field by field
//! including the restrict cards byte-for-byte.
//!
//! Runs after a submodule bump or an edit to the SM UFO source:
//!
//!     pixi run check-sm-blob-fresh

use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::sm::{sm_model, sm_parsed_model, SMRestrict};
use vibegraph::ufo::{EvaluatedModel, ParsedModel, UFOModel};

/// Panics naming the command that would check the submodule out, so an
/// uninitialised submodule reads as the setup error it is.
fn sm_submodule_dir() -> std::path::PathBuf {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../research/refs/mg5amcnlo/models/sm");
    assert!(
        dir.is_dir(),
        "SM UFO source not found at {} (run `pixi run init-sm-submodule`)",
        dir.display()
    );
    dir
}

#[test]
fn interned_default_matches_submodule() {
    let dir = sm_submodule_dir();
    let fresh = UFOModel::load(&dir, None).expect("load SM from submodule");
    let interned = sm_model(SMRestrict::Default);

    // Same collection shapes and keys (order-preserving IndexMaps).
    assert_eq!(
        interned.particles.keys().collect::<Vec<_>>(),
        fresh.particles.keys().collect::<Vec<_>>()
    );
    assert_eq!(
        interned.vertices.keys().collect::<Vec<_>>(),
        fresh.vertices.keys().collect::<Vec<_>>(),
        "pruned vertex set must match"
    );
    assert_eq!(interned.lorentz.len(), fresh.lorentz.len());
    assert_eq!(interned.couplings.len(), fresh.couplings.len());
    assert_eq!(interned.order_hierarchy, fresh.order_hierarchy);
    assert_eq!(interned.expansion_order, fresh.expansion_order);

    // Same evaluated physics on the empty param card.
    let card = "".parse::<ParamCard>().unwrap();
    let ev_i = EvaluatedModel::from_model_card(interned.clone(), &card);
    let ev_f = EvaluatedModel::from_model_card(fresh.clone(), &card);
    for name in ["MZ", "MW", "MT", "aS", "G", "ee"] {
        let vi = ev_i.param_values.get(name).map(|c| c.re);
        let vf = ev_f.param_values.get(name).map(|c| c.re);
        assert_eq!(vi, vf, "param {name} differs (interned vs submodule)");
    }
    let gc = interned.coupling_id("GC_10").expect("GC_10");
    assert_eq!(
        ev_i.coupling(gc),
        ev_f.coupling(fresh.coupling_id("GC_10").unwrap())
    );
}

/// Full-field parity between `gen_sm_blob`'s committed output and a fresh parse
/// of the pinned submodule: every particle/Lorentz-structure/coupling/vertex
/// entry (both key order and value contents), the parameter set, and the
/// coupling-order hierarchy, plus every restrict card byte-for-byte. Catches a
/// stale interned blob after the submodule is bumped or its SM model edited;
/// the submodule is a declared dependency of this layer, so its absence is a
/// failure rather than a skip.
#[test]
fn interned_blob_matches_submodule_exactly() {
    let dir = sm_submodule_dir();
    let fresh = ParsedModel::parse(&dir).expect("parse SM UFO source from submodule");
    let interned = sm_parsed_model();

    assert_eq!(
        interned.particles.keys().collect::<Vec<_>>(),
        fresh.particles.keys().collect::<Vec<_>>(),
        "particle key order drifted from the submodule"
    );
    assert_eq!(
        interned.particles.values().collect::<Vec<_>>(),
        fresh.particles.values().collect::<Vec<_>>(),
        "particle data drifted from the submodule"
    );

    assert_eq!(
        interned.lorentz.keys().collect::<Vec<_>>(),
        fresh.lorentz.keys().collect::<Vec<_>>(),
        "Lorentz-structure key order drifted from the submodule"
    );
    assert_eq!(
        interned.lorentz.values().collect::<Vec<_>>(),
        fresh.lorentz.values().collect::<Vec<_>>(),
        "Lorentz-structure data drifted from the submodule"
    );

    assert_eq!(
        interned.couplings.keys().collect::<Vec<_>>(),
        fresh.couplings.keys().collect::<Vec<_>>(),
        "coupling key order drifted from the submodule"
    );
    assert_eq!(
        interned.couplings.values().collect::<Vec<_>>(),
        fresh.couplings.values().collect::<Vec<_>>(),
        "coupling data drifted from the submodule"
    );

    assert_eq!(
        interned.vertices.keys().collect::<Vec<_>>(),
        fresh.vertices.keys().collect::<Vec<_>>(),
        "vertex key order drifted from the submodule"
    );
    assert_eq!(
        interned.vertices.values().collect::<Vec<_>>(),
        fresh.vertices.values().collect::<Vec<_>>(),
        "vertex data drifted from the submodule"
    );

    assert_eq!(
        interned.params, fresh.params,
        "parameter set drifted from the submodule"
    );
    assert_eq!(
        interned.order_hierarchy, fresh.order_hierarchy,
        "coupling-order hierarchy drifted from the submodule"
    );
    assert_eq!(
        interned.expansion_order, fresh.expansion_order,
        "coupling-order expansion caps drifted from the submodule"
    );
    assert_eq!(
        interned.propagators.keys().collect::<Vec<_>>(),
        fresh.propagators.keys().collect::<Vec<_>>(),
        "custom propagator set drifted from the submodule"
    );
    assert!(
        fresh.propagators.is_empty(),
        "the SM UFO gained a propagators.py"
    );

    for variant in SMRestrict::ALL {
        let card = format!("restrict_{}.dat", variant.suffix());
        let on_disk = std::fs::read_to_string(dir.join(&card))
            .unwrap_or_else(|e| panic!("read {card} from submodule: {e}"));
        assert_eq!(
            variant.restrict_card_text(),
            on_disk,
            "interned {card} is stale vs the submodule"
        );
    }
}
