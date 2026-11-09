//! On-disk diagram artifact: a proc card's enumerated diagram sets together with
//! the restricted model they were enumerated against, so a later step reads the
//! diagrams back instead of enumerating again and needs no UFO directory.
//!
//! Encoding is bincode, zstd-compressed, with the format version as the
//! payload's first field, decoded on its own before the body — the same rules
//! as the integrate artifact ([`crate::artifact`]), with a version sequence of
//! its own.
//!
//! The artifact checks itself on read. The embedded model must digest to the
//! [`ModelIdentity`] it records, and the sets to the recorded
//! [`diagram_set_digest`](DiagramArtifact::diagram_set_digest), so an edited or
//! corrupted file is refused rather than silently describing other physics.

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{generate_from_proc_card_in, DiagramError, DiagramSet, EnumerationPool, SupportedCard};
use crate::ufo::identity::{digest_bytes, model_digest, ModelIdentity};
use crate::ufo::loaded::{LoadedModel, ModelNaming};
use crate::ufo::{ParsedModel, UfoError};

/// The schema version of a [`DiagramArtifact`].
///
/// `1`: the proc card, the canonical process string, the model's identity, the
/// restricted parsed model and its particle naming, the diagram sets in
/// enumeration order, and the SHA-256 of the sets' bincode.
pub const DIAGRAM_FORMAT_VERSION: u32 = 1;

/// The oldest schema version [`DiagramArtifact::read_from_path`] decodes.
pub const OLDEST_READABLE_DIAGRAM_VERSION: u32 = 1;

const ZSTD_LEVEL: i32 = 19;

#[derive(Debug, Error)]
pub enum DiagramArtifactError {
    #[error("{path}: {source}")]
    Io { path: String, source: io::Error },
    #[error("{path} already exists (pass --force to overwrite)")]
    AlreadyExists { path: String },
    #[error("failed to encode diagram artifact: {0}")]
    Encode(bincode::Error),
    #[error("failed to decode diagram artifact: {0}")]
    Decode(bincode::Error),
    #[error("zstd (de)compression failed: {0}")]
    Zstd(io::Error),
    #[error(
        "diagram artifact format version {found}, but this build reads versions \
         {oldest}..={expected} (regenerate it with `vibegraph enumerate`)"
    )]
    UnsupportedVersion {
        found: u32,
        oldest: u32,
        expected: u32,
    },
    #[error(
        "refusing to write diagram artifact format version {found}: this build reads \
         versions {oldest}..={expected}"
    )]
    UnwritableVersion {
        found: u32,
        oldest: u32,
        expected: u32,
    },
    #[error(
        "the model embedded in the diagram artifact digests to {embedded}, but the artifact \
         records {label} with digest {recorded}; the file is corrupt or was edited \
         (regenerate it with `vibegraph enumerate`)"
    )]
    ModelDigestMismatch {
        label: String,
        recorded: String,
        embedded: String,
    },
    #[error(
        "the diagram sets in the artifact digest to {computed}, but the artifact records \
         {recorded}; the file is corrupt or was edited (regenerate it with `vibegraph enumerate`)"
    )]
    DiagramDigestMismatch { recorded: String, computed: String },
    #[error("cannot rebuild the model embedded in the diagram artifact: {0}")]
    Model(#[from] UfoError),
    #[error("proc card has no process")]
    NoProcess,
    #[error(transparent)]
    Enumerate(#[from] DiagramError),
}

/// SHA-256 (lowercase hex) over the bincode of `sets`, in the order given.
///
/// Every type reachable from [`DiagramSet`] uses ordered collections only, so
/// the encoding, and with it the digest, is a function of the sets' contents
/// and order.
pub fn diagram_set_digest(sets: &[DiagramSet]) -> String {
    digest_bytes(&bincode::serialize(sets).expect("serialize diagram sets"))
}

/// A proc card's enumeration, with the model it was enumerated against.
#[derive(Clone, Serialize, Deserialize)]
pub struct DiagramArtifact {
    /// Schema version; [`DIAGRAM_FORMAT_VERSION`] when written by this build.
    /// The first field, so it can be decoded on its own.
    pub format_version: u32,
    /// The proc card's text, verbatim.
    pub proc_card: String,
    /// The card's canonical process string ([`SupportedCard::process_string`]).
    pub process: String,
    /// Which model the diagrams were enumerated against; its digest is
    /// [`model_digest`] of [`parsed_model`](Self::parsed_model).
    pub model: ModelIdentity,
    /// The model with its restriction applied, from which the run's model is
    /// rebuilt on read.
    pub parsed_model: ParsedModel,
    /// The particle naming the model was built with, so the rebuilt model
    /// accepts exactly the names the card was resolved against.
    pub naming: ModelNaming,
    /// Every diagram set in enumeration order.
    pub sets: Vec<DiagramSet>,
    /// [`diagram_set_digest`] of [`sets`](Self::sets).
    pub diagram_set_digest: String,
}

/// Prefix of the encoded artifact, decoded on its own so a file written by a
/// different schema version is refused before its body is interpreted.
#[derive(Debug, Deserialize)]
struct VersionHeader {
    format_version: u32,
}

impl DiagramArtifact {
    /// Enumerate `card` against `model` on `pool` and record the result.
    ///
    /// `proc_card` is the text `card` was parsed from, and `model` the model its
    /// `import model` directive resolves to; both are recorded as given.
    pub fn enumerate(
        proc_card: &str,
        card: &SupportedCard,
        model: &LoadedModel,
        pool: EnumerationPool,
    ) -> Result<Self, DiagramArtifactError> {
        let process = card
            .process_string()
            .ok_or(DiagramArtifactError::NoProcess)?;
        let sets = generate_from_proc_card_in(card, &model.model, pool)?;
        let diagram_set_digest = diagram_set_digest(&sets);
        Ok(DiagramArtifact {
            format_version: DIAGRAM_FORMAT_VERSION,
            proc_card: proc_card.to_string(),
            process,
            model: model.identity.clone(),
            parsed_model: (*model.parsed).clone(),
            naming: model.naming.clone(),
            sets,
            diagram_set_digest,
        })
    }

    /// Serialize and write to `path`, refusing to overwrite an existing file
    /// unless `force` is set, and refusing a
    /// [`format_version`](Self::format_version) that
    /// [`read_from_path`](Self::read_from_path) would refuse.
    pub fn write_to_path(&self, path: &Path, force: bool) -> Result<(), DiagramArtifactError> {
        if !(OLDEST_READABLE_DIAGRAM_VERSION..=DIAGRAM_FORMAT_VERSION)
            .contains(&self.format_version)
        {
            return Err(DiagramArtifactError::UnwritableVersion {
                found: self.format_version,
                oldest: OLDEST_READABLE_DIAGRAM_VERSION,
                expected: DIAGRAM_FORMAT_VERSION,
            });
        }
        if !force && path.exists() {
            return Err(DiagramArtifactError::AlreadyExists {
                path: path.display().to_string(),
            });
        }
        let raw = bincode::serialize(self).map_err(DiagramArtifactError::Encode)?;
        let compressed =
            zstd::encode_all(raw.as_slice(), ZSTD_LEVEL).map_err(DiagramArtifactError::Zstd)?;
        std::fs::write(path, compressed).map_err(|source| DiagramArtifactError::Io {
            path: path.display().to_string(),
            source,
        })
    }

    /// Read and decode a previously written artifact, and rebuild its model.
    ///
    /// The format version is read from the payload's prefix before the body is
    /// decoded, so a file written by another schema version is refused by name.
    /// The embedded model is refused unless it digests to the recorded identity,
    /// and the sets unless they digest to the recorded
    /// [`diagram_set_digest`](Self::diagram_set_digest).
    pub fn read_from_path(path: &Path) -> Result<(Self, LoadedModel), DiagramArtifactError> {
        let compressed = std::fs::read(path).map_err(|source| DiagramArtifactError::Io {
            path: path.display().to_string(),
            source,
        })?;
        let raw = zstd::decode_all(compressed.as_slice()).map_err(DiagramArtifactError::Zstd)?;
        let header: VersionHeader =
            bincode::deserialize(&raw).map_err(DiagramArtifactError::Decode)?;
        let artifact: Self = match header.format_version {
            DIAGRAM_FORMAT_VERSION => {
                bincode::deserialize(&raw).map_err(DiagramArtifactError::Decode)?
            }
            found => {
                return Err(DiagramArtifactError::UnsupportedVersion {
                    found,
                    oldest: OLDEST_READABLE_DIAGRAM_VERSION,
                    expected: DIAGRAM_FORMAT_VERSION,
                })
            }
        };

        let embedded = model_digest(&artifact.parsed_model);
        if embedded != artifact.model.digest {
            return Err(DiagramArtifactError::ModelDigestMismatch {
                label: artifact.model.label(),
                recorded: artifact.model.digest.clone(),
                embedded,
            });
        }
        let computed = diagram_set_digest(&artifact.sets);
        if computed != artifact.diagram_set_digest {
            return Err(DiagramArtifactError::DiagramDigestMismatch {
                recorded: artifact.diagram_set_digest.clone(),
                computed,
            });
        }
        let model = LoadedModel::from_restricted(
            artifact.parsed_model.clone(),
            artifact.naming.clone(),
            &artifact.model.name,
            &artifact.model.restrict,
        )?;
        Ok((artifact, model))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::diagrams::{parse_proc_card, ParsingOptions};
    use crate::ufo::parameters::ParamNature;
    use crate::ufo::sm::SMRestrict;

    /// Several subprocesses from one line, with the multiparticle labels
    /// redefined.
    const MULTI_SUBPROCESS: &str = "define p = g u d u~ d~\ndefine j = p\ngenerate p p > e+ e- j\n";
    /// A decay chain, whose diagrams are stitched rather than enumerated whole.
    const DECAY_CHAIN: &str = "generate p p > z j, z > e+ e-\n";

    fn enumerate(card_text: &str, pool: EnumerationPool) -> DiagramArtifact {
        let card = parse_proc_card(card_text, &ParsingOptions::default()).unwrap();
        let model = LoadedModel::interned_sm(SMRestrict::Default);
        DiagramArtifact::enumerate(card_text, &card, &model, pool).unwrap()
    }

    fn scratch() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn encode(artifact: &DiagramArtifact) -> Vec<u8> {
        zstd::encode_all(bincode::serialize(artifact).unwrap().as_slice(), ZSTD_LEVEL).unwrap()
    }

    /// A written artifact reads back to the same sets, digests and identity, and
    /// its rebuilt model gives every particle of every set the same id under the
    /// same name as the model the sets were enumerated against.
    ///
    /// Cannot detect a field serde skips on both sides symmetrically (none is
    /// marked to be skipped), nor a difference in the rebuilt model outside its
    /// particle table and names.
    #[test]
    fn round_trip_preserves_sets_digest_and_model() {
        let dir = scratch();
        let path = dir.path().join("diagrams.bin.zst");
        let original = LoadedModel::interned_sm(SMRestrict::Default);
        let artifact = enumerate(DECAY_CHAIN, EnumerationPool::Serial);
        assert!(!artifact.sets.is_empty());
        artifact.write_to_path(&path, false).unwrap();

        let (read, model) = DiagramArtifact::read_from_path(&path).unwrap();
        assert_eq!(read.format_version, DIAGRAM_FORMAT_VERSION);
        assert_eq!(read.proc_card, DECAY_CHAIN);
        assert_eq!(read.process, artifact.process);
        assert_eq!(read.model, artifact.model);
        assert_eq!(model.identity, original.identity);
        assert_eq!(read.naming, ModelNaming::interned_sm());
        assert_eq!(read.diagram_set_digest, artifact.diagram_set_digest);
        assert_eq!(diagram_set_digest(&read.sets), artifact.diagram_set_digest);
        // Structural equality independent of serde: the Debug rendering of every
        // field of every diagram.
        assert_eq!(format!("{:?}", read.sets), format!("{:?}", artifact.sets));

        let mut checked = 0;
        for set in &read.sets {
            for name in set.particles_in.iter().chain(&set.particles_out) {
                assert_eq!(
                    model.model.particle_id(name),
                    original.model.particle_id(name),
                    "{name}"
                );
                assert!(model.model.particle_id(name).is_some(), "{name}");
                checked += 1;
            }
            for diagram in &set.diagrams {
                for id in diagram
                    .legs
                    .iter()
                    .map(|l| l.particle)
                    .chain(diagram.props.iter().map(|p| p.particle))
                {
                    assert_eq!(
                        model.model.particle(id).name,
                        original.model.particle(id).name
                    );
                }
            }
        }
        assert!(checked > 0);
        for name in original.model.particles.keys() {
            assert_eq!(
                model.model.particle_names().resolve(name),
                original.model.particle_names().resolve(name)
            );
        }
        for alias in ["mu+", "ta-", "a", "h"] {
            assert_eq!(
                model.model.particle_names().resolve(alias),
                original.model.particle_names().resolve(alias),
                "{alias}"
            );
        }
    }

    /// Enumeration order does not depend on the worker pool: the diagram-set
    /// digest of a multi-subprocess card and of a decay-chain card is the same
    /// on one thread as on four, over repeated four-thread runs.
    ///
    /// Cannot detect an order that varies only between builds or platforms, or
    /// a nondeterminism no four-thread schedule in three tries exposes.
    #[test]
    fn digest_is_independent_of_the_enumeration_pool() {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .unwrap();
        for card in [MULTI_SUBPROCESS, DECAY_CHAIN] {
            let serial = enumerate(card, EnumerationPool::Serial);
            assert!(serial.sets.iter().map(|s| s.diagrams.len()).sum::<usize>() > 1);
            for _ in 0..3 {
                let parallel = pool.install(|| enumerate(card, EnumerationPool::Ambient));
                assert_eq!(
                    parallel.diagram_set_digest, serial.diagram_set_digest,
                    "{card}: the enumeration order depends on the pool"
                );
            }
        }
    }

    /// The digest reads every diagram's contents: changing one field of one
    /// diagram changes it.
    ///
    /// Cannot detect a field the encoding omits on purpose (there is none).
    #[test]
    fn digest_sees_a_changed_diagram() {
        let artifact = enumerate(MULTI_SUBPROCESS, EnumerationPool::Serial);
        let base = diagram_set_digest(&artifact.sets);

        let populated = artifact
            .sets
            .iter()
            .rposition(|s| !s.diagrams.is_empty())
            .unwrap();

        let mut sets = artifact.sets.clone();
        let last = sets[populated].diagrams.last_mut().unwrap();
        last.symmetry_factor += 1;
        assert_ne!(diagram_set_digest(&sets), base, "symmetry factor");

        let mut sets = artifact.sets.clone();
        let diagram = &mut sets[populated].diagrams[0];
        diagram.sign = -diagram.sign;
        assert_ne!(diagram_set_digest(&sets), base, "sign");

        let mut sets = artifact.sets.clone();
        sets.swap(0, 1);
        assert_ne!(diagram_set_digest(&sets), base, "set order");
    }

    /// A payload whose header names a version outside the readable range is
    /// refused by name, with the range and the remedy, before its body is read.
    ///
    /// Version 1 is the first, so there is no older real shape to write: both
    /// payloads carry the current body under a foreign version number, which is
    /// exactly what a positional misread would accept if the header were not
    /// checked first.
    #[test]
    fn foreign_versions_are_refused_by_name() {
        let dir = scratch();
        let artifact = enumerate(DECAY_CHAIN, EnumerationPool::Serial);
        for version in [
            OLDEST_READABLE_DIAGRAM_VERSION - 1,
            DIAGRAM_FORMAT_VERSION + 1,
        ] {
            let mut foreign = artifact.clone();
            foreign.format_version = version;
            let path = dir.path().join(format!("v{version}.bin.zst"));
            std::fs::write(&path, encode(&foreign)).unwrap();
            let err = DiagramArtifact::read_from_path(&path)
                .err()
                .expect("a foreign version must be refused");
            assert!(
                matches!(
                    err,
                    DiagramArtifactError::UnsupportedVersion { found, oldest, expected }
                        if found == version
                            && oldest == OLDEST_READABLE_DIAGRAM_VERSION
                            && expected == DIAGRAM_FORMAT_VERSION
                ),
                "{err}"
            );
            let message = err.to_string();
            assert!(message.contains("1..=1"), "{message}");
            assert!(message.contains("vibegraph enumerate"), "{message}");

            assert!(matches!(
                foreign.write_to_path(&dir.path().join("w.bin.zst"), true),
                Err(DiagramArtifactError::UnwritableVersion { .. })
            ));
        }
    }

    /// An artifact whose embedded model was altered (one external parameter's
    /// default) is refused on read, because it no longer digests to the identity
    /// recorded beside it.
    ///
    /// Cannot detect an alteration made consistently to both the model and its
    /// recorded digest; that is a different model, correctly identified.
    #[test]
    fn an_altered_embedded_model_is_refused() {
        let dir = scratch();
        let path = dir.path().join("altered.bin.zst");
        let mut artifact = enumerate(DECAY_CHAIN, EnumerationPool::Serial);
        let ParamNature::External { default_value, .. } = &mut artifact
            .parsed_model
            .params
            .externals
            .iter_mut()
            .find(|p| p.name == "MZ")
            .expect("the SM has MZ")
            .nature
        else {
            panic!("MZ is external");
        };
        *default_value *= 1.0 + 1e-12;
        std::fs::write(&path, encode(&artifact)).unwrap();

        let err = DiagramArtifact::read_from_path(&path)
            .err()
            .expect("an altered model must be refused");
        assert!(
            matches!(err, DiagramArtifactError::ModelDigestMismatch { .. }),
            "{err}"
        );
        assert!(err.to_string().contains("sm-default"), "{err}");
    }

    /// An artifact whose sets were altered after the digest was taken is refused.
    #[test]
    fn altered_sets_are_refused() {
        let dir = scratch();
        let path = dir.path().join("sets.bin.zst");
        let mut artifact = enumerate(DECAY_CHAIN, EnumerationPool::Serial);
        artifact.sets[0].mirrored = !artifact.sets[0].mirrored;
        std::fs::write(&path, encode(&artifact)).unwrap();
        assert!(matches!(
            DiagramArtifact::read_from_path(&path),
            Err(DiagramArtifactError::DiagramDigestMismatch { .. })
        ));
    }

    /// Writing over an existing file needs `force`.
    #[test]
    fn refuses_to_overwrite_without_force() {
        let dir = scratch();
        let path: PathBuf = dir.path().join("diagrams.bin.zst");
        let artifact = enumerate(DECAY_CHAIN, EnumerationPool::Serial);
        artifact.write_to_path(&path, false).unwrap();
        let before = std::fs::read(&path).unwrap();
        let err = artifact
            .write_to_path(&path, false)
            .expect_err("a second write without force must fail");
        assert!(matches!(err, DiagramArtifactError::AlreadyExists { .. }));
        assert_eq!(std::fs::read(&path).unwrap(), before);
        artifact.write_to_path(&path, true).unwrap();
        DiagramArtifact::read_from_path(&path).unwrap();
    }
}
