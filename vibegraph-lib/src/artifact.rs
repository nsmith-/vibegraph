//! On-disk artifact for a completed `vibegraph integrate` run: the adapted
//! VEGAS grids plus everything the run was integrated from (the proc card, the
//! model's identity, the PDF set, the integration's run card and the
//! generation settings it recorded), so a later step reads them back rather than
//! asking for them again.
//!
//! Encoding is bincode, zstd-compressed — the same pairing already used for
//! the interned SM model blob ([`crate::ufo::sm`]).

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::generate::GenerateSettings;
use crate::phasespace::maps::MapChoices;
use crate::runcard::RunCard;
use crate::ufo::identity::ModelIdentity;
use crate::vegas::{GridRule, VegasGrid};

/// Bumped whenever [`IntegrateArtifact`]'s shape or meaning changes. Every
/// artifact is written at this version.
///
/// `15` records the systematics request typed
/// ([`crate::generate::Systematics`]): the variations MadGraph's module would
/// compute, with every default filled in, in place of the card's text for
/// them. A version-14 file recorded the fields' values but not whether the
/// card wrote `systematics_arguments` on a line of its own, and MadGraph reads
/// the request from that list or from the `sys_*` fields according to exactly
/// that, so a version-14 file cannot be upgraded and
/// [`OLDEST_READABLE_VERSION`] is this version.
pub const FORMAT_VERSION: u32 = 15;

/// The oldest schema version [`IntegrateArtifact::read_from_path`] decodes.
pub(crate) const OLDEST_READABLE_VERSION: u32 = 15;

const ZSTD_LEVEL: i32 = 19;

#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("{path}: {source}")]
    Io { path: String, source: io::Error },
    #[error("{path} already exists (pass --force to overwrite)")]
    AlreadyExists { path: String },
    #[error("failed to encode artifact: {0}")]
    Encode(bincode::Error),
    #[error("failed to decode artifact: {0}")]
    Decode(bincode::Error),
    #[error(
        "artifact format version {found}, but this build reads versions {oldest}..={expected} \
         (regenerate it with `vibegraph integrate`)"
    )]
    UnsupportedVersion {
        found: u32,
        oldest: u32,
        expected: u32,
    },
    #[error(
        "refusing to write artifact format version {found}: this build reads versions \
         {oldest}..={expected}"
    )]
    UnwritableVersion {
        found: u32,
        oldest: u32,
        expected: u32,
    },
    #[error("zstd (de)compression failed: {0}")]
    Zstd(io::Error),
}

/// One file a run read, by its name in the set directory, and the lowercase-hex
/// SHA-256 of exactly the bytes that were parsed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileDigest {
    pub name: String,
    pub sha256: String,
}

/// The parton densities an integration read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PdfInput {
    /// Fixed-energy beams, or a decay: no density was read.
    None,
    /// LHAPDF id `lhaid`, which is member `member` of set `set`
    /// (`SetIndex + member = lhaid`). `files` are the set's `.info` and the
    /// member's `.dat`, in the order they were loaded.
    Lhapdf {
        lhaid: i64,
        set: String,
        member: u32,
        files: Vec<FileDigest>,
    },
}

/// A model parameter an integration bound away from the restricted model's
/// default: its name, its parameter-card block and code, and the value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MovedParam {
    pub name: String,
    pub block: String,
    pub code: Vec<i32>,
    pub value: f64,
}

/// What an integration read besides its cards: the parton densities and the
/// model parameters that differ from the model's defaults.
///
/// `params` is the only record of the parameter card an integration bound
/// (`ufo::bind::bind_card`): every external parameter whose value differs from
/// the restricted model's default, in the model's external order. A run with no
/// card, an empty card or a card restating the defaults records none.
/// `ufo::bind::bind_recorded` rebinds the model from it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunInputs {
    pub pdf: PdfInput,
    pub params: Vec<MovedParam>,
}

impl RunInputs {
    /// The inputs of a run that read `pdf` and moved no parameter.
    pub fn with_pdf(pdf: PdfInput) -> Self {
        RunInputs {
            pdf,
            params: Vec::new(),
        }
    }
}

/// Which channel of which decomposition a banked grid was trained on.
///
/// A grid's coordinate count does not identify its channel space. A hadronic run
/// prepends the two outer `(τ, y)` coordinates to each channel's own `3n − 4`, so
/// its per-channel grids can carry the same number of coordinates as a
/// fixed-energy run's at a different final multiplicity, and a channel index alone
/// does not say whether the channels came from one subprocess's diagrams or from
/// the pooled diagrams of several flavour groups. The key is banked so a reader
/// takes that from the file instead of inferring it from a shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelKey {
    /// The whole map, undecomposed: one grid carrying the entire integral.
    Whole,
    /// One integration channel of a multichannel over a single subprocess
    /// (fixed-energy beams).
    Channel { channel: usize },
    /// One integration channel of one flavour group of a hadronic decomposition,
    /// whose channels are pooled across groups into a single mixture.
    GroupChannel { group: usize, channel: usize },
    /// One integration channel of one flavour group of the hadronic
    /// decomposition of the processes with `final_state` outgoing legs, in a sum
    /// over phase spaces: each multiplicity, or each set of outgoing masses
    /// within one, pools its own groups' channels into a mixture of its own.
    /// `group` counts the multiplicity's groups across its sets of outgoing
    /// masses, in part order.
    MultiplicityChannel {
        final_state: usize,
        group: usize,
        channel: usize,
    },
    /// One hadronic integration channel whose map is shared by `pairs`
    /// `(group, diagram)` pairs of the processes with `final_state` outgoing
    /// legs, named by the first of them. The channel draws its points once for
    /// all of them, at their summed selection weight.
    MergedChannel {
        final_state: usize,
        group: usize,
        channel: usize,
        pairs: usize,
    },
}

/// The map a channel's coordinates are drawn through, as the rule-based
/// composition derived it from the channel's diagram.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum SamplerTopology {
    /// An all-timelike decay tree: every drawn invariant is a subsystem mass.
    Timelike,
    /// A peripheral t-channel spine: an ordered chain of spacelike rungs with a
    /// timelike subsystem hanging off each, and one left as the recoil.
    Spine,
}

/// One propagator pole a channel's map is shaped by, in GeV.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) struct SamplerPole {
    pub(crate) mass: f64,
    pub(crate) width: f64,
}

/// What the rule-based channel composition chose for one sampling channel.
///
/// The composition is derived from the channel's diagram rather than declared, so
/// a process whose poles or topology move — a changed width in the model, a
/// spacelike floor the cuts imply differently, a diagram that stopped being
/// peripheral — samples differently while still reporting the same cross section.
/// Recording the choice makes that visible in the run's own record instead of
/// only in a re-derivation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelSampler {
    pub(crate) topology: SamplerTopology,
    /// The timelike poles the drawn subsystem invariants sit on, driving each
    /// invariant's Breit-Wigner importance map.
    pub(crate) resonances: Vec<SamplerPole>,
    /// The spacelike lines of the channel's diagram.
    pub(crate) t_channels: Vec<SamplerPole>,
    /// The pole locations `t_mass²` (GeV²) a peripheral channel draws its rungs'
    /// momentum transfers against, in chain order away from the first beam and
    /// *after* the regulating floor — so they differ from the corresponding
    /// `t_channels` entries wherever the floor bound, and `t_channels`' order,
    /// which is the diagram's, carries no kinematic meaning where this one does.
    /// Empty for an all-timelike tree.
    pub(crate) spine_poles_gev2: Vec<f64>,
}

impl ChannelSampler {
    /// Read the composition off a built channel.
    pub fn of(channel: &crate::phasespace::DiagramChannel<f64>) -> Self {
        let spine_poles_gev2 = channel.spine_poles();
        ChannelSampler {
            topology: if spine_poles_gev2.is_empty() {
                SamplerTopology::Timelike
            } else {
                SamplerTopology::Spine
            },
            resonances: channel
                .resonances()
                .iter()
                .map(|r| SamplerPole {
                    mass: r.mass,
                    width: r.width,
                })
                .collect(),
            t_channels: channel
                .t_channels()
                .iter()
                .map(|t| SamplerPole {
                    mass: t.mass,
                    width: t.width,
                })
                .collect(),
            spine_poles_gev2,
        }
    }
}

/// One phase-space channel's trained grid and its share of the integral.
///
/// The cross section is estimated as `Σⱼ ∫ dΦ f·αⱼgⱼ/g`, one VEGAS pass per
/// channel, so a channel's grid is meaningful only together with the `alpha` its
/// term is weighted by. A run with no multichannel decomposition banks a single
/// entry with `alpha = 1`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelGrid {
    /// The channel this grid belongs to, in its own decomposition's terms.
    pub key: ChannelKey,
    /// The channel selection weight `αⱼ` its term carries.
    pub alpha: f64,
    /// Evaluations per iteration this channel received.
    pub neval: usize,
    /// The grid trained on this channel's term, over the channel's own
    /// coordinates (no channel-selection coordinate).
    pub grid: VegasGrid,
    /// This term's contribution to the cross section, in picobarns.
    pub sigma_pb: f64,
    pub sigma_err_pb: f64,
    pub chi2_per_dof: f64,
    /// What the rule-based composition chose for this channel, where the writer
    /// knew it.
    pub sampler: Option<ChannelSampler>,
}

/// The result of one `vibegraph integrate` run: the trained [`VegasGrid`]s plus
/// the inputs that produced them.
///
/// # A compiled-program cache would key on `(model, process)`
///
/// [`model`](Self::model) and [`process`](Self::process) together are the whole
/// input to amplitude compilation: model assets → diagram enumeration → HELAS
/// evaluator. A cache that let `generate` load a compiled
/// [`AmplitudeEvaluator`](crate::helas::eval::AmplitudeEvaluator) instead of
/// recompiling would key on exactly `(model.digest, process, compiler schema
/// version)` — the first two banked here, the third belonging to the cache entry
/// (it says which compiler wrote the program, which this artifact cannot know).
/// So no field is reserved for it: the key is already derivable, and adding the
/// cache needs no schema bump.
///
/// Nothing loads such a cache today, and the measurement says not to build one
/// yet: compilation costs 0.05–0.29 s for every process the CLI can run, against
/// ~13 s for a 20k-event `generate`. Three things would have to be settled first,
/// and none is a `derive`:
///
/// * **Nothing in `helas::eval` is serde-serialisable.** `Op`, `Node<T>`, `Sym`,
///   `Const`, `Folded` and the layout/constant-pool specs would all need it, and
///   their own schema version — the arena's encoding is an internal representation
///   that has changed repeatedly for performance, so it cannot ride the artifact's
///   version.
/// * **"The compiled program" is not one fixed object.** `folded_hel` is an
///   `OnceLock` built lazily on first `eval_m2`, and it is the large part. Whether
///   the expanded arena travels with a cached program, or is rebuilt on load, is a
///   decision about what the cache is even for.
/// * **A pruned evaluator carries a kinematic contract.** `prune_zero_helicities`
///   mutates the evaluator and is correct only for partonic-CM momenta with beams
///   along ±z; replayed under other kinematics it is silently wrong. The pruned
///   flag must not travel unless the contract travels with it and is rechecked on
///   load.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrateArtifact {
    pub format_version: u32,
    /// The process string this artifact was integrated for (`"p p > e+ e-"`).
    pub process: String,
    /// The proc card the run was integrated from, as its text: the process
    /// string is what this build made of it, and the card is what was asked for.
    pub proc_card: String,
    /// The model the amplitudes were compiled from. A process string is not a
    /// model: `import model sm` and `import model sm-no_b_mass` spell the same
    /// `generate` line while giving different physics, so the model is banked and
    /// compared rather than assumed.
    pub model: ModelIdentity,
    /// The parton densities and parameters the integration read.
    pub inputs: RunInputs,
    /// Factorization scale μF (GeV).
    pub mu_f: f64,
    /// Total hadronic collision energy √s (GeV).
    pub sqrt_s_had: f64,
    /// Evaluations per iteration the run was asked for; a multichannel run splits
    /// this across its channels by `αⱼ` (see [`ChannelGrid::neval`]).
    pub neval: usize,
    pub niter: usize,
    /// The seed the integration ran at.
    pub seed: u64,
    /// The run card's integration fields, as resolved.
    pub run_card: RunCard,
    /// The run card's generation fields, recorded for a generation to replay.
    /// Nothing in the integration reads them.
    pub generate: GenerateSettings,
    /// One entry per phase-space channel, in the combiner's channel order.
    pub channels: Vec<ChannelGrid>,
    /// The summed cross section: `Σⱼ σⱼ`, with the channel errors in quadrature.
    pub sigma_pb: f64,
    pub sigma_err_pb: f64,
    pub chi2_per_dof: f64,
    /// The phase-space maps the run integrated under, every choice settled. A
    /// generator rebuilds its channels from these.
    pub maps: MapChoices,
    /// The rule every channel's [`VegasGrid`] was refined under, recorded for
    /// provenance: replaying a trained grid does not refine it, so no reader
    /// needs the rule to sample. `None` is an artifact whose channels were
    /// trained by no VEGAS grid; no writer produces one yet.
    pub grid_rule: Option<GridRule>,
}

/// Prefix of the encoded artifact, decoded on its own so a file written by a
/// different schema version is refused before its body is interpreted.
#[derive(Debug, Deserialize)]
struct VersionHeader {
    format_version: u32,
}

impl IntegrateArtifact {
    /// The single trained grid of a run that was not split across channels.
    #[cfg(test)]
    pub(crate) fn sole_grid(&self) -> Option<&VegasGrid> {
        match self.channels.as_slice() {
            [only] => Some(&only.grid),
            _ => None,
        }
    }

    /// Serialize and write to `path`, refusing to overwrite an existing file
    /// unless `force` is set, and refusing a
    /// [`format_version`](Self::format_version) that
    /// [`read_from_path`](Self::read_from_path) would refuse.
    pub fn write_to_path(&self, path: &Path, force: bool) -> Result<(), ArtifactError> {
        if !(OLDEST_READABLE_VERSION..=FORMAT_VERSION).contains(&self.format_version) {
            return Err(ArtifactError::UnwritableVersion {
                found: self.format_version,
                oldest: OLDEST_READABLE_VERSION,
                expected: FORMAT_VERSION,
            });
        }
        if !force && path.exists() {
            return Err(ArtifactError::AlreadyExists {
                path: path.display().to_string(),
            });
        }
        let raw = bincode::serialize(self).map_err(ArtifactError::Encode)?;
        let compressed =
            zstd::encode_all(raw.as_slice(), ZSTD_LEVEL).map_err(ArtifactError::Zstd)?;
        std::fs::write(path, compressed).map_err(|source| ArtifactError::Io {
            path: path.display().to_string(),
            source,
        })
    }

    /// Read and decode a previously written artifact.
    ///
    /// The format version is read from the payload's prefix before the body is
    /// decoded, so a file written by another schema version is refused by name —
    /// never misread into whatever the current field order happens to accept.
    pub fn read_from_path(path: &Path) -> Result<Self, ArtifactError> {
        Self::read_hashed(path).map(|(artifact, _)| artifact)
    }

    /// [`read_from_path`](Self::read_from_path), with the SHA-256 of the file's
    /// bytes as read: the digest and the decoded artifact come from one read.
    pub fn read_hashed(path: &Path) -> Result<(Self, String), ArtifactError> {
        let compressed = std::fs::read(path).map_err(|source| ArtifactError::Io {
            path: path.display().to_string(),
            source,
        })?;
        let sha256 = crate::ufo::identity::digest_bytes(&compressed);
        let raw = zstd::decode_all(compressed.as_slice()).map_err(ArtifactError::Zstd)?;
        let header: VersionHeader = bincode::deserialize(&raw).map_err(ArtifactError::Decode)?;
        match header.format_version {
            FORMAT_VERSION => {
                let artifact: Self = bincode::deserialize(&raw).map_err(ArtifactError::Decode)?;
                Ok((artifact, sha256))
            }
            found => Err(ArtifactError::UnsupportedVersion {
                found,
                oldest: OLDEST_READABLE_VERSION,
                expected: FORMAT_VERSION,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::ufo::sm::SMRestrict;
    use crate::vegas::VegasGrid;

    fn one_channel(grid: VegasGrid) -> ChannelGrid {
        ChannelGrid {
            key: ChannelKey::Whole,
            alpha: 1.0,
            neval: 1000,
            grid,
            sigma_pb: 934.42,
            sigma_err_pb: 0.87,
            chi2_per_dof: 1.02,
            sampler: None,
        }
    }

    fn sample_artifact() -> IntegrateArtifact {
        IntegrateArtifact {
            format_version: FORMAT_VERSION,
            process: "p p > e+ e-".to_string(),
            proc_card: "import model sm\ngenerate p p > e+ e-\n".to_string(),
            model: ModelIdentity::interned_sm(SMRestrict::Default),
            inputs: RunInputs::with_pdf(PdfInput::Lhapdf {
                lhaid: 247000,
                set: "NNPDF23_lo_as_0130_qed".to_string(),
                member: 0,
                files: vec![
                    FileDigest {
                        name: "NNPDF23_lo_as_0130_qed.info".to_string(),
                        sha256: "f961a1".to_string(),
                    },
                    FileDigest {
                        name: "NNPDF23_lo_as_0130_qed_0000.dat".to_string(),
                        sha256: "5ac949".to_string(),
                    },
                ],
            }),
            mu_f: 91.1880,
            sqrt_s_had: 13000.0,
            neval: 1000,
            niter: 2,
            seed: 42,
            run_card: RunCard::default()
                .split()
                .expect("the default card splits")
                .0,
            generate: RunCard::default()
                .split()
                .expect("the default card splits")
                .1,
            channels: vec![one_channel(VegasGrid::new(3, 64, 1.5))],
            sigma_pb: 934.42,
            sigma_err_pb: 0.87,
            chi2_per_dof: 1.02,
            maps: MapChoices::LEGACY,
            grid_rule: Some(GridRule::UNBOUNDED),
        }
    }

    #[test]
    fn round_trips_through_bincode_zstd() {
        let artifact = sample_artifact();
        let dir =
            std::env::temp_dir().join(format!("vibegraph-artifact-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("roundtrip.bin.zst");
        let _ = std::fs::remove_file(&path);

        artifact.write_to_path(&path, false).expect("write");
        let reloaded = IntegrateArtifact::read_from_path(&path).expect("read");

        assert_eq!(reloaded.format_version, artifact.format_version);
        assert_eq!(reloaded.process, artifact.process);
        assert_eq!(reloaded.proc_card, artifact.proc_card);
        assert_eq!(reloaded.generate, artifact.generate);
        assert_eq!(reloaded.model, artifact.model);
        assert_eq!(reloaded.inputs, artifact.inputs);
        assert_eq!(reloaded.mu_f.to_bits(), artifact.mu_f.to_bits());
        assert_eq!(reloaded.sqrt_s_had.to_bits(), artifact.sqrt_s_had.to_bits());
        assert_eq!(reloaded.neval, artifact.neval);
        assert_eq!(reloaded.niter, artifact.niter);
        assert_eq!(reloaded.seed, artifact.seed);
        assert_eq!(reloaded.channels.len(), artifact.channels.len());
        for (rc, ac) in reloaded.channels.iter().zip(&artifact.channels) {
            assert_eq!(rc.alpha.to_bits(), ac.alpha.to_bits());
            assert_eq!(rc.neval, ac.neval);
            assert_eq!(rc.sigma_pb.to_bits(), ac.sigma_pb.to_bits());
            assert_eq!(rc.grid.ndim(), ac.grid.ndim());
            assert_eq!(rc.grid.nbins(), ac.grid.nbins());
            for (a, b) in rc.grid.xi().iter().zip(ac.grid.xi().iter()) {
                for (x, y) in a.iter().zip(b.iter()) {
                    assert_eq!(x.to_bits(), y.to_bits());
                }
            }
        }
        let card = |a: &IntegrateArtifact| -> Vec<(String, crate::runcard::ParamValue)> {
            a.run_card
                .iter()
                .map(|(n, v)| (n.to_string(), v.clone()))
                .collect()
        };
        assert_eq!(card(&reloaded), card(&artifact));
        assert!(reloaded.run_card.get("nevents").is_none());
        assert_eq!(reloaded.sigma_pb.to_bits(), artifact.sigma_pb.to_bits());

        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn refuses_to_overwrite_without_force() {
        let artifact = sample_artifact();
        let dir = std::env::temp_dir().join(format!(
            "vibegraph-artifact-test-force-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("existing.bin.zst");
        let _ = std::fs::remove_file(&path);

        artifact.write_to_path(&path, false).expect("first write");
        let err = artifact
            .write_to_path(&path, false)
            .expect_err("second write without --force must fail");
        assert!(matches!(err, ArtifactError::AlreadyExists { .. }));

        // --force overwrites cleanly.
        artifact
            .write_to_path(&path, true)
            .expect("forced overwrite");

        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    /// The grid survives the bincode+zstd round trip bit-for-bit, so a frozen
    /// sampling pass on the reloaded grid reproduces the in-memory grid's
    /// estimate exactly under the same seed — the property a later
    /// distributed sampling phase relies on.
    #[test]
    fn reloaded_grid_sample_frozen_reproduces() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        // Adapt a grid on a smooth synthetic integrand (no PDF dependency).
        let integrand = |u: &[f64]| 3.0 * u[0] * u[0] + 2.0 * u[1];
        let mut grid = VegasGrid::new(2, 48, 1.5);
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        grid.adapt(integrand, 5000, 6, &mut rng);

        let mut artifact = sample_artifact();
        artifact.channels = vec![one_channel(grid.clone())];

        let dir = std::env::temp_dir().join(format!(
            "vibegraph-artifact-test-frozen-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("frozen.bin.zst");
        let _ = std::fs::remove_file(&path);
        artifact.write_to_path(&path, true).unwrap();
        let reloaded = IntegrateArtifact::read_from_path(&path).unwrap();

        // Same seed, in-memory vs reloaded grid → bit-identical estimate.
        let sample = |g: &VegasGrid| {
            let mut r = ChaCha8Rng::seed_from_u64(99);
            g.sample_frozen(integrand, 4000, &mut r)
        };
        let before = sample(&grid);
        let after = sample(reloaded.sole_grid().expect("single-channel run"));
        assert_eq!(before.integral.to_bits(), after.integral.to_bits());
        assert_eq!(before.std_dev.to_bits(), after.std_dev.to_bits());

        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    /// Every channel's grid survives the round trip in channel order, and a
    /// multichannel artifact reports no sole grid — so a reader cannot silently
    /// take one channel's grid for the whole map.
    #[test]
    fn multichannel_artifact_round_trips_every_grid() {
        let mut artifact = sample_artifact();
        artifact.channels = (0..4)
            .map(|j| ChannelGrid {
                key: ChannelKey::GroupChannel {
                    group: j / 2,
                    channel: j % 2,
                },
                alpha: 0.1 * (j + 1) as f64,
                neval: 1000 * (j + 1),
                grid: VegasGrid::new(2 + j, 16, 0.5),
                sigma_pb: 1.5 * (j + 1) as f64,
                sigma_err_pb: 0.01 * (j + 1) as f64,
                chi2_per_dof: 0.9,
                sampler: Some(ChannelSampler {
                    topology: SamplerTopology::Spine,
                    resonances: vec![SamplerPole {
                        mass: 91.1876,
                        width: 2.4952,
                    }],
                    t_channels: vec![SamplerPole {
                        mass: 0.0,
                        width: 0.0,
                    }],
                    spine_poles_gev2: vec![100.0, 25.0],
                }),
            })
            .collect();
        assert!(artifact.sole_grid().is_none());

        let dir = std::env::temp_dir().join(format!(
            "vibegraph-artifact-test-multi-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("multi.bin.zst");
        let _ = std::fs::remove_file(&path);
        artifact.write_to_path(&path, true).unwrap();
        let reloaded = IntegrateArtifact::read_from_path(&path).unwrap();

        assert_eq!(reloaded.channels.len(), 4);
        for (j, (rc, ac)) in reloaded.channels.iter().zip(&artifact.channels).enumerate() {
            assert_eq!(rc.alpha.to_bits(), ac.alpha.to_bits(), "channel {j}");
            assert_eq!(rc.neval, ac.neval, "channel {j}");
            assert_eq!(rc.grid.ndim(), ac.grid.ndim(), "channel {j}");
            assert_eq!(rc.sigma_pb.to_bits(), ac.sigma_pb.to_bits(), "channel {j}");
            assert_eq!(rc.sampler, ac.sampler, "channel {j}");
        }

        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    fn scratch_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "vibegraph-artifact-test-{tag}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn write_refuses_a_version_this_build_cannot_read() {
        let dir = scratch_dir("unwritable");
        for version in [OLDEST_READABLE_VERSION - 1, FORMAT_VERSION + 1] {
            let mut artifact = sample_artifact();
            artifact.format_version = version;
            let path = dir.join(format!("v{version}.bin.zst"));
            let err = artifact
                .write_to_path(&path, true)
                .expect_err("an unreadable version is not written");
            assert!(
                matches!(
                    err,
                    ArtifactError::UnwritableVersion { found, oldest, expected }
                        if found == version
                            && oldest == OLDEST_READABLE_VERSION
                            && expected == FORMAT_VERSION
                ),
                "{err}"
            );
            assert!(!path.exists(), "version {version} left a file");
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    /// An artifact written under a schema version with no reader is refused by
    /// version, not decoded into whatever the current field order accepts. The
    /// payload here is version 2's shape — no `model`, so every field after
    /// `process` sits one slot early — which is exactly the shape a silent misread
    /// would consume: a positional decode would take the PDF set's bytes for a
    /// model name and carry on.
    #[test]
    fn read_refuses_a_format_version_with_no_reader() {
        #[derive(Serialize)]
        struct V3ChannelGrid {
            alpha: f64,
            neval: usize,
            grid: VegasGrid,
            sigma_pb: f64,
            sigma_err_pb: f64,
            chi2_per_dof: f64,
        }
        #[derive(Serialize)]
        struct V2Artifact {
            format_version: u32,
            process: String,
            pdf_set: String,
            pdf_member: u32,
            mu_f: f64,
            sqrt_s_had: f64,
            neval: usize,
            niter: usize,
            seed: u64,
            run_card: RunCard,
            channels: Vec<V3ChannelGrid>,
            sigma_pb: f64,
            sigma_err_pb: f64,
            chi2_per_dof: f64,
        }

        let legacy = V2Artifact {
            format_version: OLDEST_READABLE_VERSION - 1,
            process: "p p > e+ e-".to_string(),
            pdf_set: "NNPDF23_lo_as_0130_qed".to_string(),
            pdf_member: 0,
            mu_f: 91.1880,
            sqrt_s_had: 13000.0,
            neval: 1000,
            niter: 2,
            seed: 42,
            run_card: RunCard::default(),
            channels: vec![V3ChannelGrid {
                alpha: 1.0,
                neval: 1000,
                grid: VegasGrid::new(3, 64, 1.5),
                sigma_pb: 934.42,
                sigma_err_pb: 0.87,
                chi2_per_dof: 1.02,
            }],
            sigma_pb: 934.42,
            sigma_err_pb: 0.87,
            chi2_per_dof: 1.02,
        };

        let dir = scratch_dir("version");
        let path = dir.join("legacy.bin.zst");
        let raw = bincode::serialize(&legacy).unwrap();
        let compressed = zstd::encode_all(raw.as_slice(), ZSTD_LEVEL).unwrap();
        std::fs::write(&path, compressed).unwrap();

        let err = IntegrateArtifact::read_from_path(&path)
            .expect_err("a format version with no reader must not decode");
        assert!(
            matches!(
                err,
                ArtifactError::UnsupportedVersion { found, oldest, expected }
                    if found == OLDEST_READABLE_VERSION - 1
                        && oldest == OLDEST_READABLE_VERSION
                        && expected == FORMAT_VERSION
            ),
            "expected a version refusal, got {err}"
        );

        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    /// The file with its version stamped down to `version`, its body otherwise
    /// this build's own, so a reader that decoded by shape would accept it.
    fn stamped(artifact: &IntegrateArtifact, dir: &Path, version: u32) -> PathBuf {
        let path = dir.join(format!("v{version}.bin.zst"));
        artifact.write_to_path(&path, true).unwrap();
        let mut raw = zstd::decode_all(std::fs::read(&path).unwrap().as_slice()).unwrap();
        raw[..4].copy_from_slice(&version.to_le_bytes());
        std::fs::write(&path, zstd::encode_all(raw.as_slice(), ZSTD_LEVEL).unwrap()).unwrap();
        path
    }

    /// A version-14 file is refused by its version, with the message that says
    /// to re-integrate, however well its bytes would decode: it records the
    /// systematics fields' values but not whether the card wrote
    /// `systematics_arguments`, which decides MadGraph's request. Version 13,
    /// which records no checksum of its PDF files, is refused the same way.
    #[test]
    fn a_version_14_artifact_is_refused() {
        let dir = scratch_dir("v14");
        for version in [13, 14] {
            let path = stamped(&sample_artifact(), &dir, version);
            let err = IntegrateArtifact::read_from_path(&path).expect_err("an older file");
            assert!(
                matches!(
                    err,
                    ArtifactError::UnsupportedVersion {
                        found,
                        oldest: 15,
                        expected: 15
                    } if found == version
                ),
                "{err}"
            );
        }
        let path = stamped(&sample_artifact(), &dir, 14);
        let err = IntegrateArtifact::read_from_path(&path).expect_err("a version-14 file");
        assert!(err
            .to_string()
            .contains("regenerate it with `vibegraph integrate`"));
        std::fs::remove_dir_all(&dir).ok();
    }

    /// The fixed-energy record round-trips too, as `PdfInput::None`.
    #[test]
    fn an_artifact_that_read_no_density_records_none() {
        let dir = scratch_dir("nopdf");
        let mut artifact = sample_artifact();
        artifact.inputs = RunInputs::with_pdf(PdfInput::None);
        let path = dir.join("nopdf.bin.zst");
        artifact.write_to_path(&path, true).unwrap();
        let reloaded = IntegrateArtifact::read_from_path(&path).unwrap();
        assert_eq!(reloaded.inputs, artifact.inputs);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Moved parameters round-trip in order and bit for bit.
    #[test]
    fn moved_parameters_round_trip() {
        let dir = scratch_dir("params");
        let mut artifact = sample_artifact();
        artifact.inputs.params = vec![
            MovedParam {
                name: "MZ".to_string(),
                block: "MASS".to_string(),
                code: vec![23],
                value: 91.0 + f64::EPSILON * 64.0,
            },
            MovedParam {
                name: "ymt".to_string(),
                block: "YUKAWA".to_string(),
                code: vec![6],
                value: 250.0,
            },
        ];
        let path = dir.join("params.bin.zst");
        artifact.write_to_path(&path, true).unwrap();
        let reloaded = IntegrateArtifact::read_from_path(&path).unwrap();
        assert_eq!(reloaded.inputs, artifact.inputs);
        assert_eq!(
            reloaded.inputs.params[0].value.to_bits(),
            artifact.inputs.params[0].value.to_bits()
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
