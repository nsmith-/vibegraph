//! Which model a run was built from, in a form that can be banked and compared.
//!
//! A model is named by its `import model` directive (`sm`, `sm-no_b_mass`), but a
//! name is not enough to tell two runs apart: the same directive can resolve to
//! different bytes if the interned assets or a restrict card on disk change. So an
//! identity carries both — a human-readable label for error messages, and a digest
//! over the source bytes the model was actually built from.
//!
//! The digest is SHA-256 over the model's **own serialized form** — the same
//! bincode encoding of a restricted [`ParsedModel`] that the interned SM blob
//! holds — not over the UFO source files it was parsed from. Two models that
//! parse to the same particles, couplings, vertices and parameters are the same
//! model, whatever the Python around them said: a reworded comment, reordered
//! imports or reformatted whitespace must not refuse an artifact. Equally, the
//! restriction is baked in before hashing, so the restrict card contributes
//! through its *effect* rather than its text.
//!
//! The digest has to be stable across builds and platforms, which rules out
//! `std`'s `DefaultHasher` (documented as unstable between releases). Serializing
//! is deterministic because every collection in `ParsedModel` is an `IndexMap`,
//! which preserves the parser's insertion order.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::sm::{sm_digest, SMRestrict};
use super::ParsedModel;

/// SHA-256 of `bytes`, lowercase hex.
pub fn digest_bytes(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut hex, b| {
            let _ = write!(hex, "{b:02x}");
            hex
        })
}

/// SHA-256 over a parsed model's serialized form.
///
/// Callers pass the model with its restriction already applied
/// ([`ParsedModel::apply_restriction`]), so the digest identifies the model that
/// was actually built rather than the pre-restriction one every variant shares.
pub fn model_digest(model: &ParsedModel) -> String {
    digest_bytes(&bincode::serialize(model).expect("serialize ParsedModel"))
}

/// The model a run was built from: the `import model` directive that selected it,
/// and a digest of the bytes it was built from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelIdentity {
    /// Model name as written in `import model <name>[-<restrict>]`.
    pub name: String,
    /// Restrict-card selector; `"default"` for a bare `import model sm`.
    pub restrict: String,
    /// [`model_digest`] over the restricted model this run was built from.
    pub digest: String,
}

impl ModelIdentity {
    /// The directive form, `<name>-<restrict>` — what an error message names.
    pub fn label(&self) -> String {
        format!("{}-{}", self.name, self.restrict)
    }

    /// Identity of an interned SM variant.
    pub fn interned_sm(restrict: SMRestrict) -> Self {
        ModelIdentity {
            name: "sm".to_string(),
            restrict: restrict.suffix().to_string(),
            digest: sm_digest(restrict).to_string(),
        }
    }

    /// Identity of a model loaded from a UFO directory, given the digest the
    /// loader computed from the restricted model it built.
    pub fn from_loaded(name: &str, restrict: &str, digest: String) -> Self {
        ModelIdentity {
            name: name.to_string(),
            restrict: restrict.to_string(),
            digest,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Known answers against `shasum -a 256`, so a banked value is pinned to
    /// SHA-256 and not merely to whatever this build computes. A digest that
    /// changed silently would refuse every artifact written before it.
    #[test]
    fn digest_bytes_matches_sha256() {
        assert_eq!(
            digest_bytes(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            digest_bytes(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    /// A model's serialized form survives a bincode round trip unchanged, which is
    /// what lets a digest banked during integration be compared against a model
    /// deserialized later. `ParsedModel`'s collections are `IndexMap`s, so the
    /// parser's order is preserved; a `HashMap` anywhere in it would make the
    /// digest depend on iteration order and refuse artifacts at random.
    #[test]
    fn the_digest_survives_a_serialization_round_trip() {
        let parsed = super::super::sm::sm_parsed_model();
        let encoded = bincode::serialize(&parsed).unwrap();
        let decoded: ParsedModel = bincode::deserialize(&encoded).unwrap();
        assert_eq!(model_digest(&parsed), model_digest(&decoded));
        assert_eq!(model_digest(&parsed), model_digest(&parsed.clone()));
    }

    /// Dropping a single vertex is a different model and must digest differently —
    /// the check that `model_digest` reads the model's contents rather than some
    /// stable-but-empty summary of it.
    #[test]
    fn a_changed_model_digests_differently() {
        let parsed = super::super::sm::sm_parsed_model();
        let base = model_digest(&parsed);

        let mut fewer = parsed.clone();
        let victim = fewer.vertices.keys().next().unwrap().clone();
        fewer.vertices.shift_remove(&victim);
        assert_ne!(model_digest(&fewer), base, "vertex removal is invisible");
    }

    /// Every interned SM variant has a distinct identity. The digest is over the
    /// variant's *restricted* model, so this checks that the restriction reaches it
    /// at all — a digest over the shared pre-restriction blob would make all nine
    /// variants identical.
    #[test]
    fn every_interned_variant_has_its_own_digest() {
        let mut seen = std::collections::HashSet::new();
        for variant in SMRestrict::ALL {
            let id = ModelIdentity::interned_sm(variant);
            assert_eq!(id.name, "sm");
            assert_eq!(id.restrict, variant.suffix());
            assert!(seen.insert(id.digest.clone()), "{variant:?} collided");
        }
        assert_eq!(
            ModelIdentity::interned_sm(SMRestrict::Default).label(),
            "sm-default"
        );
    }

    /// The identity of a variant does not depend on when it is asked for: banking
    /// it during integration and recomputing it during generation must agree.
    #[test]
    fn interned_identity_is_reproducible() {
        for variant in SMRestrict::ALL {
            assert_eq!(
                ModelIdentity::interned_sm(variant),
                ModelIdentity::interned_sm(variant)
            );
        }
    }
}
