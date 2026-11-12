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
//!
//! Beside the sets it banks what MadGraph's default run card reads off the
//! card's processes ([`ProcessFacts`]), so a run with no run card of its own
//! builds that card from the artifact without enumerating anything.
//!
//! Neither digest can tell an artifact written by a build that enumerated
//! differently: the sets digest to what they hold, whatever computed them. So
//! the artifact records the [`ENUMERATION_VERSION`] of the build that wrote it,
//! and a reader refuses any other. The integrate artifact records it too,
//! since it banks the same sets.

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{enumerate_lines_in, DiagramError, DiagramSet, EnumerationPool, SupportedCard};
use crate::config::{process_facts, ProcessFactsError};
use crate::runcard::process_defaults::ProcessFacts;
use crate::ufo::identity::{digest_bytes, model_digest, ModelIdentity};
use crate::ufo::loaded::{LoadedModel, ModelNaming};
use crate::ufo::{ParsedModel, UfoError};

/// The version of what enumeration computes, recorded in every artifact that
/// banks its output and compared on read.
///
/// It covers every field of the [`DiagramSet`]s a card enumerates to, as
/// enumeration, decay-chain stitching and the conversion of feyngraph's
/// diagrams produce them: which subprocesses exist and in what order, their
/// legs, polarizations and `mirrored` flag; per set, which diagrams exist and
/// in what order; per diagram its external legs (particle, charge, spin, index,
/// direction), its propagators (particle, endpoints, momentum routing, on-shell
/// flag), its vertices (interaction index, fermion-flow group, ray order), the
/// Fermi sign, the symmetry factor, and the provenance (process number, decay
/// origins). It also covers the [`ProcessFacts`] a diagram artifact banks,
/// which are read off the same enumeration.
///
/// A change to any of these, including a fix, must bump it: an artifact
/// written under another version is refused, because the amplitude probe of
/// an integrate artifact recompiles the banked diagrams and so agrees with
/// itself whatever enumeration produced them. The known-answer test
/// `enumeration_output_matches_the_pinned_version` pins the output of a fixed
/// card set to this version, so an unbumped change to that output fails it. A
/// change that moves only cards outside that set is not seen there.
pub const ENUMERATION_VERSION: u32 = 1;

/// The schema version of a [`DiagramArtifact`].
///
/// `1`: the proc card, the canonical process string, the model's identity, the
/// restricted parsed model and its particle naming, the diagram sets in
/// enumeration order, and the SHA-256 of the sets' bincode.
///
/// `2`: adds the [`ProcessFacts`] of a scattering card, after the digest.
///
/// `3`: adds the [`ENUMERATION_VERSION`] of the writing build, after the
/// format version.
pub const DIAGRAM_FORMAT_VERSION: u32 = 3;

/// The oldest schema version [`DiagramArtifact::read_from_path`] decodes.
///
/// A version-1 file has no process facts, and they cannot be read off its sets
/// alone (a decay-chain line's facts come from its core, enumerated without
/// the decays), so it is refused with the remedy rather than read. A version-2
/// file does not say which enumeration produced its sets, and only enumerating
/// the card again could tell, so it is refused the same way.
pub const OLDEST_READABLE_DIAGRAM_VERSION: u32 = 3;

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
    #[error(
        "the diagram artifact was written by enumeration version {found}, but this build \
         enumerates as version {expected}: its diagrams, Fermi signs or symmetry factors may \
         differ from what this build computes (enumerate the proc card again with \
         `vibegraph enumerate`)"
    )]
    EnumerationVersion { found: u32, expected: u32 },
    #[error("cannot rebuild the model embedded in the diagram artifact: {0}")]
    Model(#[from] UfoError),
    #[error("proc card has no process")]
    NoProcess,
    #[error(transparent)]
    Enumerate(#[from] DiagramError),
    #[error(transparent)]
    Facts(#[from] ProcessFactsError),
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
    /// The [`ENUMERATION_VERSION`] of the build that enumerated the sets.
    /// Second, so it is checked before the body is decoded.
    pub enumeration_version: u32,
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
    /// What MadGraph's default run card reads off the card's processes
    /// ([`process_facts`]), for a scattering card; `None` for a decay, whose
    /// default card reads none. Not covered by
    /// [`diagram_set_digest`](Self::diagram_set_digest).
    pub process_facts: Option<ProcessFacts>,
}

/// Prefix of the encoded artifact, decoded on its own so a file written by a
/// different schema version is refused before its body is interpreted.
#[derive(Debug, Deserialize)]
struct VersionHeader {
    format_version: u32,
}

impl DiagramArtifact {
    /// Enumerate `card` against `model` on `pool` and record the result, with
    /// the card's [`process_facts`] for a scattering card (which enumerates
    /// each decay-chain line's core again).
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
        let lines = enumerate_lines_in(card, &model.model, pool)?;
        let process_facts = if card.is_decay() {
            None
        } else {
            Some(process_facts(card, &model.model, &lines, pool)?)
        };
        let sets = lines.sets;
        let diagram_set_digest = diagram_set_digest(&sets);
        Ok(DiagramArtifact {
            format_version: DIAGRAM_FORMAT_VERSION,
            enumeration_version: ENUMERATION_VERSION,
            proc_card: proc_card.to_string(),
            process,
            model: model.identity.clone(),
            parsed_model: (*model.parsed).clone(),
            naming: model.naming.clone(),
            sets,
            diagram_set_digest,
            process_facts,
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
    /// decoded, so a file written by another schema version is refused by name,
    /// and then the enumeration version, so a file another enumeration wrote is
    /// refused by name before its sets are read.
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
                let (_, enumeration_version): (u32, u32) =
                    bincode::deserialize(&raw).map_err(DiagramArtifactError::Decode)?;
                if enumeration_version != ENUMERATION_VERSION {
                    return Err(DiagramArtifactError::EnumerationVersion {
                        found: enumeration_version,
                        expected: ENUMERATION_VERSION,
                    });
                }
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
        assert!(artifact.process_facts.is_some());
        assert_eq!(read.process_facts, artifact.process_facts);
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
    /// Both payloads carry the current body under a foreign version number,
    /// which is exactly what a positional misread would accept if the header
    /// were not checked first; [`a_version_1_artifact_is_refused_by_name`]
    /// writes the real older shape.
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
            assert!(message.contains("3..=3"), "{message}");
            assert!(message.contains("vibegraph enumerate"), "{message}");

            assert!(matches!(
                foreign.write_to_path(&dir.path().join("w.bin.zst"), true),
                Err(DiagramArtifactError::UnwritableVersion { .. })
            ));
        }
    }

    /// The version-1 shape: every field of version 2 but the process facts.
    #[derive(Serialize)]
    struct DiagramArtifactV1<'a> {
        format_version: u32,
        proc_card: &'a str,
        process: &'a str,
        model: &'a ModelIdentity,
        parsed_model: &'a ParsedModel,
        naming: &'a ModelNaming,
        sets: &'a [DiagramSet],
        diagram_set_digest: &'a str,
    }

    /// A file in the real version-1 shape, with a model and sets that would pass
    /// every check, is refused by its version, naming the version, the readable
    /// range and the remedy.
    ///
    /// Cannot detect a version-1 file mislabelled with the current version
    /// number; that one fails to decode, as any truncated body does.
    #[test]
    fn a_version_1_artifact_is_refused_by_name() {
        let dir = scratch();
        let path = dir.path().join("v1.bin.zst");
        let artifact = enumerate(DECAY_CHAIN, EnumerationPool::Serial);
        let v1 = DiagramArtifactV1 {
            format_version: 1,
            proc_card: &artifact.proc_card,
            process: &artifact.process,
            model: &artifact.model,
            parsed_model: &artifact.parsed_model,
            naming: &artifact.naming,
            sets: &artifact.sets,
            diagram_set_digest: &artifact.diagram_set_digest,
        };
        let raw = bincode::serialize(&v1).unwrap();
        std::fs::write(&path, zstd::encode_all(raw.as_slice(), ZSTD_LEVEL).unwrap()).unwrap();
        let err = DiagramArtifact::read_from_path(&path)
            .err()
            .expect("a version-1 artifact must be refused");
        assert!(
            matches!(
                err,
                DiagramArtifactError::UnsupportedVersion {
                    found: 1,
                    oldest: 3,
                    expected: 3
                }
            ),
            "{err}"
        );
        let message = err.to_string();
        assert!(message.contains("version 1"), "{message}");
        assert!(message.contains("vibegraph enumerate"), "{message}");
    }

    /// A file in the real version-2 shape (every field of version 3 but the
    /// enumeration version), with a model and sets that would pass every check,
    /// is refused by its version with the remedy: it does not say which
    /// enumeration produced its sets.
    ///
    /// The version-2 bytes are the current encoding with the enumeration
    /// version's four bytes removed, which is exactly version 2's field order.
    #[test]
    fn a_version_2_artifact_is_refused_by_name() {
        let dir = scratch();
        let path = dir.path().join("v2.bin.zst");
        let artifact = enumerate(DECAY_CHAIN, EnumerationPool::Serial);
        let current = bincode::serialize(&artifact).unwrap();
        assert_eq!(
            current[4..8],
            ENUMERATION_VERSION.to_le_bytes(),
            "the enumeration version is the second field"
        );
        let mut v2 = 2u32.to_le_bytes().to_vec();
        v2.extend_from_slice(&current[8..]);
        std::fs::write(&path, zstd::encode_all(v2.as_slice(), ZSTD_LEVEL).unwrap()).unwrap();
        let err = DiagramArtifact::read_from_path(&path)
            .err()
            .expect("a version-2 artifact must be refused");
        assert!(
            matches!(
                err,
                DiagramArtifactError::UnsupportedVersion {
                    found: 2,
                    oldest: 3,
                    expected: 3
                }
            ),
            "{err}"
        );
        assert!(err.to_string().contains("vibegraph enumerate"), "{err}");
    }

    /// An artifact recording another enumeration version is refused by name,
    /// naming both versions and the remedy, although its model and sets pass
    /// every digest check: the sets digest to what they hold, whatever
    /// enumeration computed them.
    ///
    /// Cannot detect an enumeration change made without bumping
    /// [`ENUMERATION_VERSION`]; the known-answer test is what sees that.
    #[test]
    fn another_enumeration_version_is_refused_by_name() {
        let dir = scratch();
        let artifact = enumerate(DECAY_CHAIN, EnumerationPool::Serial);
        for version in [ENUMERATION_VERSION - 1, ENUMERATION_VERSION + 1] {
            let mut other = artifact.clone();
            other.enumeration_version = version;
            let path = dir.path().join(format!("e{version}.bin.zst"));
            other.write_to_path(&path, false).unwrap();
            let err = DiagramArtifact::read_from_path(&path)
                .err()
                .expect("another enumeration version must be refused");
            assert!(
                matches!(
                    err,
                    DiagramArtifactError::EnumerationVersion { found, expected }
                        if found == version && expected == ENUMERATION_VERSION
                ),
                "{err}"
            );
            let message = err.to_string();
            assert!(
                message.contains(&format!("enumeration version {version}")),
                "{message}"
            );
            assert!(
                message.contains(&format!("version {ENUMERATION_VERSION}:")),
                "{message}"
            );
            assert!(message.contains("vibegraph enumerate"), "{message}");
        }
        let path = dir.path().join("same.bin.zst");
        artifact.write_to_path(&path, false).unwrap();
        DiagramArtifact::read_from_path(&path).unwrap();
    }

    /// The known-answer cards: each with the model it is enumerated against,
    /// and the [`diagram_set_digest`] of its sets and the digest of its
    /// [`ProcessFacts`] at [`PINNED_ENUMERATION_VERSION`].
    ///
    /// Together they reach fermion-flow signs of both parities (Bhabha
    /// scattering), a decay chain with identical fermions across its decays
    /// (`z z`, both to `mu+ mu-`), a multi-subprocess decay chain with
    /// mirrored sets (`p p > z j`), a polarized leg, and a four-fermion
    /// contact vertex with two fermion-flow groups in one interaction
    /// (SMEFTsim's four-lepton operators, `e+ e- > e+ e-`), a nested decay
    /// (`t > w+ b, w+ > e+ ve`), and an on-shell s-channel veto (`$ z`). Every
    /// tree-level
    /// symmetry factor is 1: each internal vertex of a tree is fixed by the
    /// external legs beneath it, so the only automorphism fixing the legs is
    /// the identity. The test asserts it, so a card reaching another factor
    /// would show here first.
    const KNOWN_ANSWERS: &[(KnownModel, &str, &str, &str)] = &[
        (
            KnownModel::Sm,
            "generate e+ e- > e+ e-\n",
            "df0708649b8048b2e8cd3b5d245fbc4d9331374d99d679fff58883cc74a5f28f",
            "d6b775e94607f5b40530653e6221314db4dc741a3956b50c15ad116baad1d217",
        ),
        (
            KnownModel::Sm,
            "generate e+ e- > z z, z > mu+ mu-\n",
            "99a0d99581dec0e63c7d08e0d26ef94ce8a89fe10d0bfc05dfaa6ab95abdee4a",
            "8e7ac86a8a558c179465567b409cd30beee390d6686dfb2f37fc307bbca04bd5",
        ),
        (
            KnownModel::Sm,
            "generate p p > z j, z > e+ e-\n",
            "d904cbfa5ecadb4dc4cae2e0fcb4a0739841d77769a1d073c2b39827c15645da",
            "1c5a4fdfe718de09dfe70060a536fddf9d96654ee19dcb68eb108d9b2f35624c",
        ),
        (
            KnownModel::Sm,
            "generate u u~ > z{0} g\n",
            "52839bbd96e5aa09c4b11d80109c30863c72146cce76798403a42f5a19a1e9ff",
            "54295cbb4d4d5c5b5d12203cd3af923b56625f55bc6d5eaafb84233ab767007e",
        ),
        (
            KnownModel::SmeftFourLepton,
            "generate e+ e- > e+ e- NP<=1\n",
            "96a7f460f1649d30bf512e859f8c7eb78a211a713e3e924d8cd20f7137d66eb3",
            "cc16e6d7f5b40fa515d58e1d8b1447cc1de2bc5a3b289b644ef2662550a8b909",
        ),
        (
            KnownModel::Sm,
            "generate e+ e- > t t~, (t > w+ b, w+ > e+ ve), t~ > w- b~\n",
            "9088a370bbee74e2047309980bd1dec62063594415213018a1633076c1c0e86f",
            "e32695af77e2374ac3e4ec3425ff4173dc589c3710efc035fefd46ab781fa366",
        ),
        (
            KnownModel::Sm,
            "generate e+ e- > mu+ mu- a $ z\n",
            "44f701aef0e6a4edfa57095ada1bb4826a002d1aaa0b0b0552a41cf1bab87d4c",
            "81ea234ccbc5b4c33146bd6c833adee75a2293f52c900a3fb8fa9842004492b5",
        ),
    ];

    /// The [`ENUMERATION_VERSION`] the digests of [`KNOWN_ANSWERS`] were
    /// recorded at.
    const PINNED_ENUMERATION_VERSION: u32 = 1;

    #[derive(Clone, Copy, Debug)]
    enum KnownModel {
        /// The interned SM, default restriction.
        Sm,
        /// The vendored SMEFTsim under the repository's four-lepton restrict
        /// card (`cll1`, `cee`, `cle`).
        SmeftFourLepton,
    }

    impl KnownModel {
        fn load(self) -> LoadedModel {
            match self {
                KnownModel::Sm => LoadedModel::interned_sm(SMRestrict::Default),
                KnownModel::SmeftFourLepton => {
                    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
                    LoadedModel::from_dir(
                        &root.join("../validation/ufo/SMEFTsim_topU3l_MwScheme_UFO"),
                        Some(&root.join("../validation/madgraph/cards/smeft/restrict_vg_c4l.dat")),
                        crate::ufo::names::ParticleNaming::default(),
                        "SMEFTsim_topU3l_MwScheme_UFO",
                        "vg_c4l",
                    )
                    .expect("load SMEFTsim under the four-lepton restrict card")
                }
            }
        }
    }

    /// The enumeration output of a fixed card set is the output pinned at
    /// [`ENUMERATION_VERSION`]: an artifact's enumeration version says what
    /// computed its sets only while every change to that output bumps it.
    ///
    /// Each card's sets and process facts are digested and compared with the
    /// pinned digests, and each card is checked to still reach the feature it
    /// is in the set for. Cannot detect a change that moves only cards outside
    /// the set, nor a change to how a model is parsed or built that moves a
    /// card's output (it moves the digest here too, and the artifacts' model
    /// digest already refuses those).
    #[test]
    fn enumeration_output_matches_the_pinned_version() {
        let mut mismatches = Vec::new();
        let mut signs = std::collections::BTreeSet::new();
        let mut stitched_identical = false;
        let mut contact_flow_groups = std::collections::BTreeSet::new();
        let mut polarized = false;
        let mut mirrored = false;
        let mut nested = false;
        let mut vetoed = false;
        for &(known, card_text, sets_digest, facts_digest) in KNOWN_ANSWERS {
            let model = known.load();
            let card = parse_proc_card(card_text, &ParsingOptions::default()).unwrap();
            let artifact =
                DiagramArtifact::enumerate(card_text, &card, &model, EnumerationPool::Serial)
                    .unwrap();
            let facts = digest_bytes(&bincode::serialize(&artifact.process_facts).unwrap());
            if artifact.diagram_set_digest != sets_digest || facts != facts_digest {
                mismatches.push(format!(
                    "{card_text:?} ({known:?}): sets {} (pinned {sets_digest}), \
                     facts {facts} (pinned {facts_digest})",
                    artifact.diagram_set_digest
                ));
            }
            vetoed |= artifact
                .process_facts
                .as_ref()
                .is_some_and(|f| f.onshell_veto);
            for set in &artifact.sets {
                mirrored |= set.mirrored;
                polarized |= set.polarizations.iter().any(Option::is_some);
                let repeated = set
                    .particles_out
                    .iter()
                    .any(|p| set.particles_out.iter().filter(|q| *q == p).count() > 1);
                for diagram in &set.diagrams {
                    assert_eq!(diagram.symmetry_factor, 1, "{card_text:?}");
                    signs.insert(diagram.sign);
                    stitched_identical |= repeated && diagram.provenance.decays.len() > 1;
                    nested |= diagram.provenance.decays.len() > 2;
                    for vertex in &diagram.vertices {
                        let fermions = vertex
                            .rays
                            .iter()
                            .filter(|r| match r {
                                crate::diagrams::diagram::Ray::Leg(l) => {
                                    diagram.legs[l.0].spin == 2
                                }
                                crate::diagrams::diagram::Ray::Prop { prop, .. } => {
                                    model.model.particle(diagram.props[prop.0].particle).spin == 2
                                }
                            })
                            .count();
                        if fermions == 4 {
                            contact_flow_groups.insert(vertex.flow_group);
                        }
                    }
                }
            }
        }
        assert!(
            mismatches.is_empty(),
            "the enumeration output changed at ENUMERATION_VERSION {ENUMERATION_VERSION}:\n{}\n\
             If the change is intended, bump ENUMERATION_VERSION and record every digest \
             printed here in KNOWN_ANSWERS, with PINNED_ENUMERATION_VERSION set to the new \
             version, in the same change: artifacts written before it no longer describe what \
             this build enumerates, and the version is what refuses them.",
            mismatches.join("\n")
        );
        assert_eq!(
            PINNED_ENUMERATION_VERSION, ENUMERATION_VERSION,
            "ENUMERATION_VERSION was bumped with the known answers unchanged: a bump records \
             a change to the enumeration output, so re-pin KNOWN_ANSWERS with \
             PINNED_ENUMERATION_VERSION in the same change"
        );
        assert_eq!(
            signs,
            [-1, 1].into(),
            "the card set reaches both Fermi signs"
        );
        assert!(
            stitched_identical,
            "the card set reaches identical particles across decays"
        );
        assert_eq!(
            contact_flow_groups,
            [0, 1].into(),
            "the card set reaches a four-fermion contact in both flow groups"
        );
        assert!(polarized, "the card set reaches a polarized leg");
        assert!(mirrored, "the card set reaches a mirrored set");
        assert!(nested, "the card set reaches a nested decay");
        assert!(vetoed, "the card set reaches an on-shell s-channel veto");
    }

    /// A decay card banks no process facts, and a scattering card does.
    #[test]
    fn only_a_scattering_card_banks_process_facts() {
        let decay = enumerate("generate z > e+ e-\n", EnumerationPool::Serial);
        assert_eq!(decay.process_facts, None);
        let scattering = enumerate(MULTI_SUBPROCESS, EnumerationPool::Serial);
        let facts = scattering.process_facts.expect("a scattering card");
        assert_eq!(facts.core_legs.len(), scattering.sets.len());
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
