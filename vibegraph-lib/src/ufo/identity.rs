//! Which model a run was built from, in a form that can be banked and compared.
//!
//! A model is named by its `import model` directive (`sm`, `sm-no_b_mass`), but a
//! name is not enough to tell two runs apart: the same directive can resolve to
//! different bytes if the interned assets or a restrict card on disk change. So an
//! identity carries both — a human-readable label for error messages, and a digest
//! over the source bytes the model was actually built from.
//!
//! The digest is a 128-bit FNV-1a over length-framed parts. It guards against an
//! accidental mismatch, not a forged one; what it needs is to be stable across
//! builds and platforms, which rules out `std`'s `DefaultHasher` (documented as
//! unstable between releases).

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::sm::{sm_assets, SMRestrict};
use super::{UfoError, OPTIONAL_SOURCE_FILES, REQUIRED_SOURCE_FILES};

const FNV_OFFSET_BASIS: u128 = 0x6c62272e07bb014262b821756295c58d;
const FNV_PRIME: u128 = 0x0000000001000000000000000000013b;

fn absorb(state: &mut u128, bytes: &[u8]) {
    for &byte in bytes {
        *state ^= byte as u128;
        *state = state.wrapping_mul(FNV_PRIME);
    }
}

/// 128-bit FNV-1a over `parts`, each prefixed by its length so that a different
/// split of the same concatenated bytes gives a different digest.
pub fn digest(parts: &[&[u8]]) -> String {
    let mut state = FNV_OFFSET_BASIS;
    for part in parts {
        absorb(&mut state, &(part.len() as u64).to_le_bytes());
        absorb(&mut state, part);
    }
    format!("{state:032x}")
}

/// The model a run was built from: the `import model` directive that selected it,
/// and a digest of the bytes it was built from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelIdentity {
    /// Model name as written in `import model <name>[-<restrict>]`.
    pub name: String,
    /// Restrict-card selector; `"default"` for a bare `import model sm`.
    pub restrict: String,
    /// [`digest`] over the model's source bytes: for the interned SM, the
    /// compressed pre-restriction blob and the restrict card's raw text; for a UFO
    /// directory, every source file the loader reads plus the restrict card.
    pub digest: String,
}

impl ModelIdentity {
    /// The directive form, `<name>-<restrict>` — what an error message names.
    pub fn label(&self) -> String {
        format!("{}-{}", self.name, self.restrict)
    }

    /// Identity of an interned SM variant, digesting the assets [`sm_model`]
    /// builds it from.
    ///
    /// [`sm_model`]: super::sm::sm_model
    pub fn interned_sm(restrict: SMRestrict) -> Self {
        ModelIdentity {
            name: "sm".to_string(),
            restrict: restrict.suffix().to_string(),
            digest: digest(&sm_assets(restrict)),
        }
    }

    /// Identity of a model loaded from a UFO directory, digesting every source
    /// file [`ParsedModel::parse`] reads plus the restrict card
    /// [`UFOModel::load`] resolves — so the digest covers exactly the bytes the
    /// loaded model was derived from.
    ///
    /// Optional files and the restrict card contribute a presence marker as well
    /// as their bytes, so "absent" and "present but empty" are distinct.
    ///
    /// [`ParsedModel::parse`]: super::ParsedModel::parse
    /// [`UFOModel::load`]: super::UFOModel::load
    pub fn from_ufo_dir(
        name: &str,
        restrict: &str,
        dir: &Path,
        restrict_card: Option<&Path>,
    ) -> Result<Self, UfoError> {
        let read = |path: &Path| -> Result<Vec<u8>, UfoError> {
            std::fs::read(path).map_err(|cause| UfoError::Io {
                file: path.display().to_string(),
                cause,
            })
        };

        let mut parts: Vec<Vec<u8>> = Vec::new();
        for file in REQUIRED_SOURCE_FILES {
            parts.push(read(&dir.join(file))?);
        }
        for file in OPTIONAL_SOURCE_FILES {
            let path = dir.join(file);
            // The loader treats an unreadable optional file as absent, so the
            // digest has to as well.
            parts.push(std::fs::read(&path).unwrap_or_default());
            parts.push(vec![path.exists() as u8]);
        }

        // Mirrors the loader's resolution: the explicit card, else
        // `restrict_default.dat` when the directory has one, else no restriction.
        let card_path = match restrict_card {
            Some(path) => Some(path.to_path_buf()),
            None => {
                let default = dir.join("restrict_default.dat");
                default.exists().then_some(default)
            }
        };
        match card_path {
            Some(path) => {
                parts.push(read(&path)?);
                parts.push(vec![1]);
            }
            None => parts.push(vec![0]),
        }

        let refs: Vec<&[u8]> = parts.iter().map(|p| p.as_slice()).collect();
        Ok(ModelIdentity {
            name: name.to_string(),
            restrict: restrict.to_string(),
            digest: digest(&refs),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The digest separates its parts: a different split of the same concatenated
    /// bytes must not collide, or a restrict card could be made to look like a
    /// longer model blob with a shorter card.
    #[test]
    fn digest_is_framed_not_concatenated() {
        assert_ne!(digest(&[b"ab", b"c"]), digest(&[b"a", b"bc"]));
        assert_ne!(digest(&[b"abc"]), digest(&[b"ab", b"c"]));
        assert_eq!(digest(&[b"ab", b"c"]), digest(&[b"ab", b"c"]));
        assert_eq!(digest(&[]).len(), 32);
    }

    /// Every interned SM variant has a distinct identity. The digest is over the
    /// shared blob plus the variant's own card, so this is really a check that the
    /// card bytes reach the digest at all — a digest over the blob alone would
    /// make all nine variants identical.
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

    /// A UFO directory's digest moves when any source file's bytes move, including
    /// the restrict card — the case a name comparison provably cannot see.
    #[test]
    fn a_directory_digest_follows_every_source_file() {
        let tmp = std::env::temp_dir().join(format!(
            "vibegraph-model-identity-{}-{}",
            std::process::id(),
            line!()
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        for file in REQUIRED_SOURCE_FILES {
            std::fs::write(tmp.join(file), b"# empty\n").unwrap();
        }
        std::fs::write(tmp.join("restrict_default.dat"), b"BLOCK MASS\n  5 0.0\n").unwrap();

        let base = ModelIdentity::from_ufo_dir("toy", "default", &tmp, None).unwrap();
        assert_eq!(base.label(), "toy-default");

        for file in REQUIRED_SOURCE_FILES {
            let path = tmp.join(file);
            let original = std::fs::read(&path).unwrap();
            std::fs::write(&path, b"# changed\n").unwrap();
            assert_ne!(
                ModelIdentity::from_ufo_dir("toy", "default", &tmp, None)
                    .unwrap()
                    .digest,
                base.digest,
                "{file} does not reach the digest"
            );
            std::fs::write(&path, original).unwrap();
        }

        // Same name, different card contents.
        std::fs::write(tmp.join("restrict_default.dat"), b"BLOCK MASS\n  5 4.7\n").unwrap();
        let recard = ModelIdentity::from_ufo_dir("toy", "default", &tmp, None).unwrap();
        assert_eq!(recard.label(), base.label());
        assert_ne!(recard.digest, base.digest);

        // An optional file appearing is a real change too.
        std::fs::write(tmp.join("restrict_default.dat"), b"BLOCK MASS\n  5 0.0\n").unwrap();
        std::fs::write(tmp.join(OPTIONAL_SOURCE_FILES[0]), b"# orders\n").unwrap();
        assert_ne!(
            ModelIdentity::from_ufo_dir("toy", "default", &tmp, None)
                .unwrap()
                .digest,
            base.digest
        );

        std::fs::remove_dir_all(&tmp).ok();
    }
}
