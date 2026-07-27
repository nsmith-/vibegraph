//! Interned Standard Model UFO model.
//!
//! The SM model data is baked into the binary so that normal `cargo build`/`test`
//! never touch the `research/refs/mg5amcnlo` submodule. A single compressed blob
//! holds the pre-restriction parsed model ([`ParsedModel`]); the nine SM restrict
//! cards are baked in as raw SLHA text. [`sm_model`] deserializes the blob, applies
//! a variant's restrict card, and caches the resulting [`UFOModel`] per variant.
//!
//! To regenerate the blob after the submodule model changes, run the committed dev
//! binary `gen_sm_blob` (see [`regenerate`]).

use std::sync::{Arc, OnceLock};

use super::slha::ParamCard;
use super::{ParsedModel, UFOModel, UfoError};

/// The nine SM restrict-card variants shipped with the MadGraph SM UFO model.
///
/// Each selects one `restrict_*.dat` card (`import model sm-<variant>`), which
/// zeroes a set of parameters and prunes the corresponding zero-coupling vertices.
/// [`SMRestrict::Default`] is the plain `restrict_default.dat` (`import model sm`).
///
/// The `snake_case` variant name is the MadGraph restrict-suffix and card stem
/// (`restrict_<suffix>.dat`); the suffix ↔ variant round-trip
/// ([`suffix`](SMRestrict::suffix)/[`from_suffix`](SMRestrict::from_suffix)) is
/// derived by `strum` from the enum itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumString, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum SMRestrict {
    Default,
    CMass,
    Ckm,
    LeptonMasses,
    NoBMass,
    NoMasses,
    NoTauMass,
    NoWidths,
    ZeromassCkm,
}

impl SMRestrict {
    /// All variants, in declaration order (parallel to the per-variant cache).
    pub const ALL: [SMRestrict; 9] = [
        SMRestrict::Default,
        SMRestrict::CMass,
        SMRestrict::Ckm,
        SMRestrict::LeptonMasses,
        SMRestrict::NoBMass,
        SMRestrict::NoMasses,
        SMRestrict::NoTauMass,
        SMRestrict::NoWidths,
        SMRestrict::ZeromassCkm,
    ];

    fn index(self) -> usize {
        SMRestrict::ALL.iter().position(|&v| v == self).unwrap()
    }

    /// This variant's MadGraph restrict-suffix (the `snake_case` variant name);
    /// `Default` → `"default"`. Inverse of [`from_suffix`](SMRestrict::from_suffix).
    pub fn suffix(self) -> &'static str {
        self.into()
    }

    /// Maps `import model sm-<suffix>` to a variant.
    ///
    /// A missing or empty suffix (`import model sm`) selects `Default`, as does the
    /// explicit `"default"`; an unrecognized suffix returns `None`.
    pub fn from_suffix(suffix: Option<&str>) -> Option<SMRestrict> {
        match suffix {
            None | Some("") => Some(SMRestrict::Default),
            Some(s) => s.parse().ok(),
        }
    }

    /// The raw SLHA text of this variant's restrict card (baked into the binary).
    pub fn restrict_card_text(self) -> &'static str {
        match self {
            SMRestrict::Default => include_str!("sm_assets/restrict_default.dat"),
            SMRestrict::CMass => include_str!("sm_assets/restrict_c_mass.dat"),
            SMRestrict::Ckm => include_str!("sm_assets/restrict_ckm.dat"),
            SMRestrict::LeptonMasses => include_str!("sm_assets/restrict_lepton_masses.dat"),
            SMRestrict::NoBMass => include_str!("sm_assets/restrict_no_b_mass.dat"),
            SMRestrict::NoMasses => include_str!("sm_assets/restrict_no_masses.dat"),
            SMRestrict::NoTauMass => include_str!("sm_assets/restrict_no_tau_mass.dat"),
            SMRestrict::NoWidths => include_str!("sm_assets/restrict_no_widths.dat"),
            SMRestrict::ZeromassCkm => include_str!("sm_assets/restrict_zeromass_ckm.dat"),
        }
    }
}

/// The compressed, serialized pre-restriction SM [`ParsedModel`] blob.
static SM_PARSED_BLOB: &[u8] = include_bytes!("sm_assets/sm_parsed.bin.zst");

/// zstd + bincode (de)serialization of the pre-restriction parsed model.
///
/// Shared by the runtime loader and the `gen_sm_blob` dev binary so both agree
/// on the on-disk format.
pub(crate) fn serialize_parsed(model: &ParsedModel) -> Vec<u8> {
    let raw = bincode::serialize(model).expect("serialize ParsedModel");
    zstd::encode_all(raw.as_slice(), 19).expect("zstd-compress ParsedModel blob")
}

fn deserialize_parsed(blob: &[u8]) -> ParsedModel {
    let raw = zstd::decode_all(blob).expect("zstd-decompress interned SM blob");
    bincode::deserialize(&raw).expect("deserialize interned SM ParsedModel")
}

/// The shared pre-restriction parsed model, deserialized once.
fn sm_parsed() -> &'static ParsedModel {
    static PARSED: OnceLock<ParsedModel> = OnceLock::new();
    PARSED.get_or_init(|| deserialize_parsed(SM_PARSED_BLOB))
}

/// A copy of the interned pre-restriction SM, for callers that need a model the baked
/// restrict variants do not cover — deriving a deliberately altered model to test how
/// the rest of the pipeline reacts to it, for instance. Restricting it with
/// [`ParsedModel::into_model`] reproduces exactly what [`sm_model`] caches.
pub fn sm_parsed_model() -> ParsedModel {
    sm_parsed().clone()
}

/// The interned Standard Model, restricted per `restrict` and cached per variant.
///
/// Deserializes the baked blob and applies the variant's restrict card (parameter
/// zeroing + zero-coupling vertex pruning + topology rebuild), matching
/// `UFOModel::load(sm_dir, Some(restrict_card))` bit-for-bit. Each variant is a
/// distinct pruned model, cached behind its own `OnceLock` and shared cheaply via
/// `Arc`.
pub fn sm_model(restrict: SMRestrict) -> Arc<UFOModel> {
    static CACHE: [OnceLock<Arc<UFOModel>>; SMRestrict::ALL.len()] =
        [const { OnceLock::new() }; SMRestrict::ALL.len()];

    CACHE[restrict.index()]
        .get_or_init(|| {
            let card: ParamCard = restrict
                .restrict_card_text()
                .parse()
                .expect("parse interned SM restrict card");
            let model = sm_parsed()
                .clone()
                .into_model(Some(&card))
                .expect("build interned SM model");
            Arc::new(model)
        })
        .clone()
}

/// Regenerate the interned SM assets from a UFO `sm` model directory.
///
/// Used by the `gen_sm_blob` dev binary: parses `sm_dir` with the crate's own
/// [`ParsedModel::parse`], writes the compressed blob and the nine restrict cards
/// into `out_dir` (the committed `src/ufo/sm_assets/` tree). Normal builds only
/// read these via `include_bytes!`/`include_str!`; the submodule is needed only here.
pub fn regenerate(sm_dir: &std::path::Path, out_dir: &std::path::Path) -> Result<(), UfoError> {
    std::fs::create_dir_all(out_dir).map_err(|e| UfoError::Io {
        file: out_dir.display().to_string(),
        cause: e,
    })?;

    let parsed = ParsedModel::parse(sm_dir)?;
    let blob = serialize_parsed(&parsed);
    let blob_path = out_dir.join("sm_parsed.bin.zst");
    std::fs::write(&blob_path, &blob).map_err(|e| UfoError::Io {
        file: blob_path.display().to_string(),
        cause: e,
    })?;

    for variant in SMRestrict::ALL {
        let card = format!("restrict_{}.dat", variant.suffix());
        let src = sm_dir.join(&card);
        let dst = out_dir.join(&card);
        std::fs::copy(&src, &dst).map_err(|e| UfoError::Io {
            file: src.display().to_string(),
            cause: e,
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ufo::{slha::ParamCard, EvaluatedModel};

    fn sm_submodule_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../research/refs/mg5amcnlo/models/sm")
    }

    #[test]
    fn interned_default_matches_submodule() {
        let dir = sm_submodule_dir();
        if !dir.exists() {
            eprintln!("SM submodule not present — skipping interned-vs-submodule check");
            return;
        }
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
    /// a bare `cargo test` skips it when the submodule isn't checked out, but the
    /// `check-sm-blob-fresh` pixi task inits the submodule first so the check always
    /// runs there.
    #[test]
    fn interned_blob_matches_submodule_exactly() {
        let dir = sm_submodule_dir();
        if !dir.exists() {
            eprintln!("SM submodule not present — skipping interned-blob staleness check");
            return;
        }
        let fresh = ParsedModel::parse(&dir).expect("parse SM UFO source from submodule");
        let interned = sm_parsed();

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

    #[test]
    fn all_variants_load_and_cache() {
        for v in SMRestrict::ALL {
            let a = sm_model(v);
            let b = sm_model(v);
            assert!(Arc::ptr_eq(&a, &b), "{v:?} should be cached (same Arc)");
            assert!(!a.particles.is_empty());
            assert!(!a.vertices.is_empty());
        }
    }

    #[test]
    fn suffix_mapping_round_trips() {
        // Every variant round-trips through its `snake_case` suffix, and suffixes are
        // distinct (so the mapping is a bijection).
        let mut seen = std::collections::HashSet::new();
        for v in SMRestrict::ALL {
            let s = v.suffix();
            assert!(seen.insert(s), "duplicate suffix {s:?}");
            assert_eq!(SMRestrict::from_suffix(Some(s)), Some(v));
        }

        // The default card has no suffix in `import model sm`, and `default` is explicit.
        assert_eq!(SMRestrict::from_suffix(None), Some(SMRestrict::Default));
        assert_eq!(SMRestrict::from_suffix(Some("")), Some(SMRestrict::Default));
        assert_eq!(SMRestrict::Default.suffix(), "default");

        assert_eq!(SMRestrict::from_suffix(Some("bogus")), None);
    }
}
