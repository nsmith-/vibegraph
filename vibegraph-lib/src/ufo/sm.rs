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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    /// The MadGraph restrict-suffix for this variant (`None` for the default card).
    ///
    /// Maps `import model sm-<suffix>` to a variant; `sm` (no suffix) → `Default`.
    pub fn from_suffix(suffix: Option<&str>) -> Option<SMRestrict> {
        Some(match suffix {
            None | Some("") | Some("default") => SMRestrict::Default,
            Some("c_mass") => SMRestrict::CMass,
            Some("ckm") => SMRestrict::Ckm,
            Some("lepton_masses") => SMRestrict::LeptonMasses,
            Some("no_b_mass") => SMRestrict::NoBMass,
            Some("no_masses") => SMRestrict::NoMasses,
            Some("no_tau_mass") => SMRestrict::NoTauMass,
            Some("no_widths") => SMRestrict::NoWidths,
            Some("zeromass_ckm") => SMRestrict::ZeromassCkm,
            Some(_) => return None,
        })
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

    const CARDS: [&str; 9] = [
        "restrict_default.dat",
        "restrict_c_mass.dat",
        "restrict_ckm.dat",
        "restrict_lepton_masses.dat",
        "restrict_no_b_mass.dat",
        "restrict_no_masses.dat",
        "restrict_no_tau_mass.dat",
        "restrict_no_widths.dat",
        "restrict_zeromass_ckm.dat",
    ];
    for card in CARDS {
        let src = sm_dir.join(card);
        let dst = out_dir.join(card);
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
    use crate::ufo::slha::ParamCard;

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
        let ev_i = interned.evaluate(&card);
        let ev_f = fresh.evaluate(&card);
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
        assert_eq!(SMRestrict::from_suffix(None), Some(SMRestrict::Default));
        assert_eq!(
            SMRestrict::from_suffix(Some("no_b_mass")),
            Some(SMRestrict::NoBMass)
        );
        assert_eq!(
            SMRestrict::from_suffix(Some("zeromass_ckm")),
            Some(SMRestrict::ZeromassCkm)
        );
        assert_eq!(SMRestrict::from_suffix(Some("bogus")), None);
    }
}
