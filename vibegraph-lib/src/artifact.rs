//! On-disk artifact for a completed `vibegraph integrate` run: the adapted
//! VEGAS grids plus enough run metadata (seed, evaluation counts, process,
//! model, PDF set, the resolved run card) for a later phase to detect a mismatched
//! input rather than silently sampling against the wrong grid.
//!
//! Encoding is bincode, zstd-compressed — the same pairing already used for
//! the interned SM model blob ([`crate::ufo::sm`]).

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::runcard::RunCard;
use crate::ufo::identity::ModelIdentity;
use crate::vegas::VegasGrid;

/// Bumped whenever [`IntegrateArtifact`]'s shape changes in a way that would
/// break decoding an older file.
///
/// `2` carries one grid per phase-space channel in place of the single grid over
/// the channel mixture, with each channel's selection weight and its share of the
/// integral alongside — so a reader can neither mistake a channel's grid for the
/// whole map nor reweight a term without its `αⱼ`.
///
/// `3` adds [`IntegrateArtifact::model`], so a later phase can tell which model the
/// grids were trained under instead of inferring it from a process string that two
/// different models spell the same way.
///
/// `4` adds [`ChannelGrid::key`], naming the channel space each grid belongs to.
///
/// `5` adds [`ChannelGrid::sampler`], the summary of what the rule-based channel
/// composition chose for that channel — the map it draws through and the
/// propagator poles that map is shaped by. Versions 3 and 4 are still read,
/// through [`v3`] and [`v4`], and upgrade with no sampler recorded (`None`),
/// which is what those writers knew.
///
/// `6` replaces `ChannelSampler`'s single spine pole with
/// [`ChannelSampler::spine_poles_gev2`], one entry per rung in chain order: a
/// peripheral channel is a chain of rungs, and a scalar could only report the
/// first of them. A version-5 file records exactly that first pole, so it upgrades
/// to a one-entry list, which for the ladder-free processes a version-5 writer
/// could produce is the whole chain.
///
/// `7` changes no field at all: a version-6 file decodes through the same struct
/// as a version-7 one. What changed is what [`IntegrateArtifact::sigma_pb`] means
/// on a run whose scale needs a channel — `dynamical_scale_choice = -1` and not
/// every scale fixed. A version-6 writer read that channel from the sampler; from
/// this version, it is drawn per point from the per-configuration `AMP2`, so a
/// version-6 `sigma_pb` is a different run's answer, not a stale copy of this
/// one's. Because the schema did not move, [`read_from_path`](IntegrateArtifact::read_from_path)
/// decodes a version-6 file directly rather than through an `upgrade`, and — unlike
/// versions 3 through 5, whose upgrades normalise to `FORMAT_VERSION` because
/// nothing downstream read the field — it leaves the file's own recorded version in
/// place, so a caller can tell a pre-draw artifact from a post-draw one by
/// [`IntegrateArtifact::format_version`] alone. `vibegraph generate` is that
/// caller: it refuses to replay a `format_version < 7` artifact's `sigma_pb` as
/// `XSECUP` when the run card selects the clustering scale
/// ([`crate::coupling::scales::ScaleChoice::needs_channels`]).
pub const FORMAT_VERSION: u32 = 7;

/// The oldest schema version [`IntegrateArtifact::read_from_path`] still decodes.
pub const OLDEST_READABLE_VERSION: u32 = 3;

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
    #[error("zstd (de)compression failed: {0}")]
    Zstd(io::Error),
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
    /// One diagram's channel of a per-diagram multichannel over a single
    /// subprocess (fixed-energy beams).
    Diagram { diagram: usize },
    /// One diagram of one flavour group of a hadronic decomposition, whose
    /// channels are pooled across groups into a single mixture.
    GroupDiagram { group: usize, diagram: usize },
}

/// The map a channel's coordinates are drawn through, as the rule-based
/// composition derived it from the channel's diagram.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SamplerTopology {
    /// An all-timelike decay tree: every drawn invariant is a subsystem mass.
    Timelike,
    /// A peripheral t-channel spine: an ordered chain of spacelike rungs with a
    /// timelike subsystem hanging off each, and one left as the recoil.
    Spine,
}

/// One propagator pole a channel's map is shaped by, in GeV.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SamplerPole {
    pub mass: f64,
    pub width: f64,
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
    pub topology: SamplerTopology,
    /// The timelike poles the drawn subsystem invariants sit on, driving each
    /// invariant's Breit-Wigner importance map.
    pub resonances: Vec<SamplerPole>,
    /// The spacelike lines of the channel's diagram.
    pub t_channels: Vec<SamplerPole>,
    /// The pole locations `t_mass²` (GeV²) a peripheral channel draws its rungs'
    /// momentum transfers against, in chain order away from the first beam and
    /// *after* the regulating floor — so they differ from the corresponding
    /// `t_channels` entries wherever the floor bound, and `t_channels`' order,
    /// which is the diagram's, carries no kinematic meaning where this one does.
    /// Empty for an all-timelike tree.
    pub spine_poles_gev2: Vec<f64>,
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
    /// knew it. `None` in a file upgraded from a schema version that predates the
    /// field — a writer that recorded nothing, not a channel with no composition.
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
    /// The model the amplitudes were compiled from. A process string is not a
    /// model: `import model sm` and `import model sm-no_b_mass` spell the same
    /// `generate` line while giving different physics, so the model is banked and
    /// compared rather than assumed.
    pub model: ModelIdentity,
    pub pdf_set: String,
    pub pdf_member: u32,
    /// Factorization scale μF (GeV).
    pub mu_f: f64,
    /// Total hadronic collision energy √s (GeV).
    pub sqrt_s_had: f64,
    /// Evaluations per iteration the run was asked for; a multichannel run splits
    /// this across its channels by `αⱼ` (see [`ChannelGrid::neval`]).
    pub neval: usize,
    pub niter: usize,
    pub seed: u64,
    pub run_card: RunCard,
    /// One entry per phase-space channel, in the combiner's channel order.
    pub channels: Vec<ChannelGrid>,
    /// The summed cross section: `Σⱼ σⱼ`, with the channel errors in quadrature.
    pub sigma_pb: f64,
    pub sigma_err_pb: f64,
    pub chi2_per_dof: f64,
}

/// Prefix of the encoded artifact, decoded on its own so a file written by a
/// different schema version is refused before its body is interpreted.
#[derive(Debug, Deserialize)]
struct VersionHeader {
    format_version: u32,
}

/// Schema version 3, kept so artifacts banked before the channel key exists still
/// load.
///
/// A version-3 file could only have been written by one of two paths: the
/// Drell–Yan integrand, which banks a single grid over the whole map, or the
/// fixed-energy per-diagram multichannel, which banks one grid per diagram in
/// diagram order. The upgrade reads the key off that, which is exactly as much as
/// the older file knows — it is not a guess about a hadronic run, because no
/// version-3 writer could produce one.
pub mod v3 {
    use serde::Deserialize;

    use crate::runcard::RunCard;
    use crate::ufo::identity::ModelIdentity;
    use crate::vegas::VegasGrid;

    #[derive(Debug, Deserialize)]
    pub(super) struct ChannelGrid {
        pub alpha: f64,
        pub neval: usize,
        pub grid: VegasGrid,
        pub sigma_pb: f64,
        pub sigma_err_pb: f64,
        pub chi2_per_dof: f64,
    }

    #[derive(Debug, Deserialize)]
    pub(super) struct IntegrateArtifact {
        /// The version this file was actually written at, carried through the
        /// upgrade unchanged rather than normalised to `FORMAT_VERSION`: a reader
        /// downstream of an upgrade (`vibegraph generate`'s artifact-age guard) needs
        /// the file's own version, not the version the in-memory struct now matches.
        pub format_version: u32,
        pub process: String,
        pub model: ModelIdentity,
        pub pdf_set: String,
        pub pdf_member: u32,
        pub mu_f: f64,
        pub sqrt_s_had: f64,
        pub neval: usize,
        pub niter: usize,
        pub seed: u64,
        pub run_card: RunCard,
        pub channels: Vec<ChannelGrid>,
        pub sigma_pb: f64,
        pub sigma_err_pb: f64,
        pub chi2_per_dof: f64,
    }
}

/// Schema version 4, kept so artifacts banked before the sampler summary exists
/// still load. Every field is version 5's but for [`ChannelGrid::sampler`], which
/// a version-4 writer did not record and the upgrade therefore leaves `None`.
pub mod v4 {
    use serde::Deserialize;

    use super::ChannelKey;
    use crate::runcard::RunCard;
    use crate::ufo::identity::ModelIdentity;
    use crate::vegas::VegasGrid;

    #[derive(Debug, Deserialize)]
    pub(super) struct ChannelGrid {
        pub key: ChannelKey,
        pub alpha: f64,
        pub neval: usize,
        pub grid: VegasGrid,
        pub sigma_pb: f64,
        pub sigma_err_pb: f64,
        pub chi2_per_dof: f64,
    }

    #[derive(Debug, Deserialize)]
    pub(super) struct IntegrateArtifact {
        /// The version this file was actually written at, carried through the
        /// upgrade unchanged rather than normalised to `FORMAT_VERSION`: a reader
        /// downstream of an upgrade (`vibegraph generate`'s artifact-age guard) needs
        /// the file's own version, not the version the in-memory struct now matches.
        pub format_version: u32,
        pub process: String,
        pub model: ModelIdentity,
        pub pdf_set: String,
        pub pdf_member: u32,
        pub mu_f: f64,
        pub sqrt_s_had: f64,
        pub neval: usize,
        pub niter: usize,
        pub seed: u64,
        pub run_card: RunCard,
        pub channels: Vec<ChannelGrid>,
        pub sigma_pb: f64,
        pub sigma_err_pb: f64,
        pub chi2_per_dof: f64,
    }
}

/// Schema version 5, kept so artifacts banked before the rung chain still load.
/// Every field is version 6's but for the sampler's spine pole, which a version-5
/// writer recorded as a single `Option<f64>`.
///
/// That scalar is the whole chain, not a truncation of one: a version-5 writer
/// left every diagram with more than one spacelike line on the all-timelike tree,
/// so a file with a pole recorded has exactly one rung and the upgrade to a list
/// loses nothing.
pub mod v5 {
    use serde::Deserialize;

    use super::{ChannelKey, SamplerPole, SamplerTopology};
    use crate::runcard::RunCard;
    use crate::ufo::identity::ModelIdentity;
    use crate::vegas::VegasGrid;

    #[derive(Debug, Deserialize)]
    pub(super) struct ChannelSampler {
        pub topology: SamplerTopology,
        pub resonances: Vec<SamplerPole>,
        pub t_channels: Vec<SamplerPole>,
        pub spine_pole_gev2: Option<f64>,
    }

    #[derive(Debug, Deserialize)]
    pub(super) struct ChannelGrid {
        pub key: ChannelKey,
        pub alpha: f64,
        pub neval: usize,
        pub grid: VegasGrid,
        pub sigma_pb: f64,
        pub sigma_err_pb: f64,
        pub chi2_per_dof: f64,
        pub sampler: Option<ChannelSampler>,
    }

    #[derive(Debug, Deserialize)]
    pub(super) struct IntegrateArtifact {
        /// The version this file was actually written at, carried through the
        /// upgrade unchanged rather than normalised to `FORMAT_VERSION`: a reader
        /// downstream of an upgrade (`vibegraph generate`'s artifact-age guard) needs
        /// the file's own version, not the version the in-memory struct now matches.
        pub format_version: u32,
        pub process: String,
        pub model: ModelIdentity,
        pub pdf_set: String,
        pub pdf_member: u32,
        pub mu_f: f64,
        pub sqrt_s_had: f64,
        pub neval: usize,
        pub niter: usize,
        pub seed: u64,
        pub run_card: RunCard,
        pub channels: Vec<ChannelGrid>,
        pub sigma_pb: f64,
        pub sigma_err_pb: f64,
        pub chi2_per_dof: f64,
    }
}

impl v5::IntegrateArtifact {
    fn upgrade(self) -> IntegrateArtifact {
        IntegrateArtifact {
            format_version: self.format_version,
            process: self.process,
            model: self.model,
            pdf_set: self.pdf_set,
            pdf_member: self.pdf_member,
            mu_f: self.mu_f,
            sqrt_s_had: self.sqrt_s_had,
            neval: self.neval,
            niter: self.niter,
            seed: self.seed,
            run_card: self.run_card,
            channels: self
                .channels
                .into_iter()
                .map(|c| ChannelGrid {
                    key: c.key,
                    alpha: c.alpha,
                    neval: c.neval,
                    grid: c.grid,
                    sigma_pb: c.sigma_pb,
                    sigma_err_pb: c.sigma_err_pb,
                    chi2_per_dof: c.chi2_per_dof,
                    sampler: c.sampler.map(|s| ChannelSampler {
                        topology: s.topology,
                        resonances: s.resonances,
                        t_channels: s.t_channels,
                        spine_poles_gev2: s.spine_pole_gev2.into_iter().collect(),
                    }),
                })
                .collect(),
            sigma_pb: self.sigma_pb,
            sigma_err_pb: self.sigma_err_pb,
            chi2_per_dof: self.chi2_per_dof,
        }
    }
}

impl v4::IntegrateArtifact {
    fn upgrade(self) -> IntegrateArtifact {
        IntegrateArtifact {
            format_version: self.format_version,
            process: self.process,
            model: self.model,
            pdf_set: self.pdf_set,
            pdf_member: self.pdf_member,
            mu_f: self.mu_f,
            sqrt_s_had: self.sqrt_s_had,
            neval: self.neval,
            niter: self.niter,
            seed: self.seed,
            run_card: self.run_card,
            channels: self
                .channels
                .into_iter()
                .map(|c| ChannelGrid {
                    key: c.key,
                    alpha: c.alpha,
                    neval: c.neval,
                    grid: c.grid,
                    sigma_pb: c.sigma_pb,
                    sigma_err_pb: c.sigma_err_pb,
                    chi2_per_dof: c.chi2_per_dof,
                    sampler: None,
                })
                .collect(),
            sigma_pb: self.sigma_pb,
            sigma_err_pb: self.sigma_err_pb,
            chi2_per_dof: self.chi2_per_dof,
        }
    }
}

impl v3::IntegrateArtifact {
    fn upgrade(self) -> IntegrateArtifact {
        let sole = self.channels.len() == 1;
        IntegrateArtifact {
            format_version: self.format_version,
            process: self.process,
            model: self.model,
            pdf_set: self.pdf_set,
            pdf_member: self.pdf_member,
            mu_f: self.mu_f,
            sqrt_s_had: self.sqrt_s_had,
            neval: self.neval,
            niter: self.niter,
            seed: self.seed,
            run_card: self.run_card,
            channels: self
                .channels
                .into_iter()
                .enumerate()
                .map(|(j, c)| ChannelGrid {
                    key: if sole {
                        ChannelKey::Whole
                    } else {
                        ChannelKey::Diagram { diagram: j }
                    },
                    alpha: c.alpha,
                    neval: c.neval,
                    grid: c.grid,
                    sigma_pb: c.sigma_pb,
                    sigma_err_pb: c.sigma_err_pb,
                    chi2_per_dof: c.chi2_per_dof,
                    sampler: None,
                })
                .collect(),
            sigma_pb: self.sigma_pb,
            sigma_err_pb: self.sigma_err_pb,
            chi2_per_dof: self.chi2_per_dof,
        }
    }
}

impl IntegrateArtifact {
    /// The single trained grid of a run that was not split across channels.
    pub fn sole_grid(&self) -> Option<&VegasGrid> {
        match self.channels.as_slice() {
            [only] => Some(&only.grid),
            _ => None,
        }
    }

    /// Serialize and write to `path`, refusing to overwrite an existing file
    /// unless `force` is set.
    pub fn write_to_path(&self, path: &Path, force: bool) -> Result<(), ArtifactError> {
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
    /// The format version is read from the payload's prefix and dispatched on before
    /// the body is decoded, so a file written by another schema version is either
    /// decoded through that version's own reader or refused by name — never misread
    /// into whatever the current field order happens to accept.
    pub fn read_from_path(path: &Path) -> Result<Self, ArtifactError> {
        let compressed = std::fs::read(path).map_err(|source| ArtifactError::Io {
            path: path.display().to_string(),
            source,
        })?;
        let raw = zstd::decode_all(compressed.as_slice()).map_err(ArtifactError::Zstd)?;
        let header: VersionHeader = bincode::deserialize(&raw).map_err(ArtifactError::Decode)?;
        match header.format_version {
            // 6 and 7 share one schema (see `FORMAT_VERSION`'s doc), so a version-6
            // file decodes straight into the current struct with its own recorded
            // `format_version` intact — no upgrade function is needed or wanted.
            FORMAT_VERSION | 6 => bincode::deserialize(&raw).map_err(ArtifactError::Decode),
            5 => bincode::deserialize::<v5::IntegrateArtifact>(&raw)
                .map_err(ArtifactError::Decode)
                .map(v5::IntegrateArtifact::upgrade),
            4 => bincode::deserialize::<v4::IntegrateArtifact>(&raw)
                .map_err(ArtifactError::Decode)
                .map(v4::IntegrateArtifact::upgrade),
            3 => bincode::deserialize::<v3::IntegrateArtifact>(&raw)
                .map_err(ArtifactError::Decode)
                .map(v3::IntegrateArtifact::upgrade),
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
            model: ModelIdentity::interned_sm(SMRestrict::Default),
            pdf_set: "NNPDF23_lo_as_0130_qed".to_string(),
            pdf_member: 0,
            mu_f: 91.1880,
            sqrt_s_had: 13000.0,
            neval: 1000,
            niter: 2,
            seed: 42,
            run_card: RunCard::default(),
            channels: vec![one_channel(VegasGrid::new(3, 64, 1.5))],
            sigma_pb: 934.42,
            sigma_err_pb: 0.87,
            chi2_per_dof: 1.02,
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
        assert_eq!(reloaded.model, artifact.model);
        assert_eq!(reloaded.pdf_set, artifact.pdf_set);
        assert_eq!(reloaded.pdf_member, artifact.pdf_member);
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
        assert_eq!(
            reloaded.run_card.ebeam1.to_bits(),
            artifact.run_card.ebeam1.to_bits()
        );
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
                key: ChannelKey::GroupDiagram {
                    group: j / 2,
                    diagram: j % 2,
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

    /// Version 3's channel record, so the tests below can write a genuine version-3
    /// payload rather than the current one with a stamped-down version number.
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
    struct V3Artifact {
        format_version: u32,
        process: String,
        model: ModelIdentity,
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

    fn v3_channel(alpha: f64, grid: VegasGrid) -> V3ChannelGrid {
        V3ChannelGrid {
            alpha,
            neval: 1000,
            grid,
            sigma_pb: 934.42,
            sigma_err_pb: 0.87,
            chi2_per_dof: 1.02,
        }
    }

    fn write_v3(path: &Path, channels: Vec<V3ChannelGrid>) {
        let legacy = V3Artifact {
            format_version: 3,
            process: "p p > e+ e-".to_string(),
            model: ModelIdentity::interned_sm(SMRestrict::Default),
            pdf_set: "NNPDF23_lo_as_0130_qed".to_string(),
            pdf_member: 0,
            mu_f: 91.1880,
            sqrt_s_had: 13000.0,
            neval: 1000,
            niter: 2,
            seed: 42,
            run_card: RunCard::default(),
            channels,
            sigma_pb: 934.42,
            sigma_err_pb: 0.87,
            chi2_per_dof: 1.02,
        };
        let raw = bincode::serialize(&legacy).unwrap();
        let compressed = zstd::encode_all(raw.as_slice(), ZSTD_LEVEL).unwrap();
        std::fs::write(path, compressed).unwrap();
    }

    fn scratch_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "vibegraph-artifact-test-{tag}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A version-3 artifact still loads, and its channels take the key the writer
    /// that produced it implies: a lone grid is the whole map, several are one
    /// subprocess's diagrams in diagram order. Nothing older could have written a
    /// hadronic decomposition, so no version-3 channel upgrades to one.
    #[test]
    fn a_version_3_artifact_upgrades_to_the_current_schema() {
        let dir = scratch_dir("v3");

        let sole = dir.join("v3-sole.bin.zst");
        write_v3(&sole, vec![v3_channel(1.0, VegasGrid::new(3, 64, 1.5))]);
        let upgraded = IntegrateArtifact::read_from_path(&sole).expect("version 3 reads");
        // The schema is current; the recorded version is the file's own, not
        // normalised, so a reader downstream can still tell this artifact predates
        // the per-point AMP2 configuration draw introduced at format version 7.
        assert_eq!(upgraded.format_version, 3);
        assert_eq!(upgraded.channels[0].key, ChannelKey::Whole);
        assert!(upgraded.sole_grid().is_some());
        assert_eq!(upgraded.sigma_pb.to_bits(), 934.42f64.to_bits());

        let multi = dir.join("v3-multi.bin.zst");
        write_v3(
            &multi,
            (0..3)
                .map(|j| v3_channel(0.25 * (j + 1) as f64, VegasGrid::new(5, 16, 0.5)))
                .collect(),
        );
        let upgraded = IntegrateArtifact::read_from_path(&multi).expect("version 3 reads");
        let keys: Vec<ChannelKey> = upgraded.channels.iter().map(|c| c.key).collect();
        assert_eq!(
            keys,
            vec![
                ChannelKey::Diagram { diagram: 0 },
                ChannelKey::Diagram { diagram: 1 },
                ChannelKey::Diagram { diagram: 2 },
            ]
        );
        for (j, c) in upgraded.channels.iter().enumerate() {
            assert_eq!(c.alpha.to_bits(), (0.25 * (j + 1) as f64).to_bits());
        }

        std::fs::remove_file(&sole).ok();
        std::fs::remove_file(&multi).ok();
        std::fs::remove_dir(&dir).ok();
    }

    /// A version-4 artifact still loads. Every field it carried survives; the
    /// sampler summary it never held reads back as `None`, which says the writer
    /// recorded nothing rather than that the channel had no composition.
    #[test]
    fn a_version_4_artifact_upgrades_to_the_current_schema() {
        #[derive(Serialize)]
        struct V4ChannelGrid {
            key: ChannelKey,
            alpha: f64,
            neval: usize,
            grid: VegasGrid,
            sigma_pb: f64,
            sigma_err_pb: f64,
            chi2_per_dof: f64,
        }

        #[derive(Serialize)]
        struct V4Artifact {
            format_version: u32,
            process: String,
            model: ModelIdentity,
            pdf_set: String,
            pdf_member: u32,
            mu_f: f64,
            sqrt_s_had: f64,
            neval: usize,
            niter: usize,
            seed: u64,
            run_card: RunCard,
            channels: Vec<V4ChannelGrid>,
            sigma_pb: f64,
            sigma_err_pb: f64,
            chi2_per_dof: f64,
        }

        let legacy = V4Artifact {
            format_version: 4,
            process: "p p > e+ e-".to_string(),
            model: ModelIdentity::interned_sm(SMRestrict::Default),
            pdf_set: "NNPDF23_lo_as_0130_qed".to_string(),
            pdf_member: 0,
            mu_f: 91.1880,
            sqrt_s_had: 13000.0,
            neval: 1000,
            niter: 2,
            seed: 42,
            run_card: RunCard::default(),
            channels: (0..3)
                .map(|j| V4ChannelGrid {
                    key: ChannelKey::GroupDiagram {
                        group: j / 2,
                        diagram: j % 2,
                    },
                    alpha: 0.25 * (j + 1) as f64,
                    neval: 1000,
                    grid: VegasGrid::new(4, 16, 0.5),
                    sigma_pb: 934.42,
                    sigma_err_pb: 0.87,
                    chi2_per_dof: 1.02,
                })
                .collect(),
            sigma_pb: 934.42,
            sigma_err_pb: 0.87,
            chi2_per_dof: 1.02,
        };

        let dir = scratch_dir("v4");
        let path = dir.join("v4.bin.zst");
        let raw = bincode::serialize(&legacy).unwrap();
        let compressed = zstd::encode_all(raw.as_slice(), ZSTD_LEVEL).unwrap();
        std::fs::write(&path, compressed).unwrap();

        let upgraded = IntegrateArtifact::read_from_path(&path).expect("version 4 reads");
        assert_eq!(upgraded.format_version, 4);
        assert_eq!(upgraded.channels.len(), 3);
        for (j, c) in upgraded.channels.iter().enumerate() {
            assert_eq!(
                c.key,
                ChannelKey::GroupDiagram {
                    group: j / 2,
                    diagram: j % 2
                }
            );
            assert_eq!(c.alpha.to_bits(), (0.25 * (j + 1) as f64).to_bits());
            assert_eq!(c.sampler, None, "channel {j}");
        }
        assert_eq!(upgraded.sigma_pb.to_bits(), 934.42f64.to_bits());

        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    /// A version-5 artifact still loads, and its single spine pole becomes a
    /// one-rung chain — while a channel that recorded no pole comes back with an
    /// empty chain rather than a chain holding a zero, which is the one way this
    /// upgrade could quietly turn an all-timelike tree into a peripheral one.
    #[test]
    fn a_version_5_artifact_upgrades_to_the_current_schema() {
        #[derive(Serialize)]
        struct V5ChannelSampler {
            topology: SamplerTopology,
            resonances: Vec<SamplerPole>,
            t_channels: Vec<SamplerPole>,
            spine_pole_gev2: Option<f64>,
        }

        #[derive(Serialize)]
        struct V5ChannelGrid {
            key: ChannelKey,
            alpha: f64,
            neval: usize,
            grid: VegasGrid,
            sigma_pb: f64,
            sigma_err_pb: f64,
            chi2_per_dof: f64,
            sampler: Option<V5ChannelSampler>,
        }

        #[derive(Serialize)]
        struct V5Artifact {
            format_version: u32,
            process: String,
            model: ModelIdentity,
            pdf_set: String,
            pdf_member: u32,
            mu_f: f64,
            sqrt_s_had: f64,
            neval: usize,
            niter: usize,
            seed: u64,
            run_card: RunCard,
            channels: Vec<V5ChannelGrid>,
            sigma_pb: f64,
            sigma_err_pb: f64,
            chi2_per_dof: f64,
        }

        let sampler = |spine: Option<f64>| V5ChannelSampler {
            topology: match spine {
                Some(_) => SamplerTopology::Spine,
                None => SamplerTopology::Timelike,
            },
            resonances: vec![SamplerPole {
                mass: 91.1876,
                width: 2.4952,
            }],
            t_channels: spine
                .map(|_| {
                    vec![SamplerPole {
                        mass: 0.0,
                        width: 0.0,
                    }]
                })
                .unwrap_or_default(),
            spine_pole_gev2: spine,
        };

        let legacy = V5Artifact {
            format_version: 5,
            process: "p p > e+ e- j".to_string(),
            model: ModelIdentity::interned_sm(SMRestrict::Default),
            pdf_set: "NNPDF23_lo_as_0130_qed".to_string(),
            pdf_member: 0,
            mu_f: 91.1880,
            sqrt_s_had: 13000.0,
            neval: 1000,
            niter: 2,
            seed: 42,
            run_card: RunCard::default(),
            channels: vec![
                V5ChannelGrid {
                    key: ChannelKey::GroupDiagram {
                        group: 0,
                        diagram: 0,
                    },
                    alpha: 0.25,
                    neval: 1000,
                    grid: VegasGrid::new(4, 16, 0.5),
                    sigma_pb: 934.42,
                    sigma_err_pb: 0.87,
                    chi2_per_dof: 1.02,
                    sampler: Some(sampler(Some(0.4))),
                },
                V5ChannelGrid {
                    key: ChannelKey::GroupDiagram {
                        group: 0,
                        diagram: 1,
                    },
                    alpha: 0.75,
                    neval: 1000,
                    grid: VegasGrid::new(4, 16, 0.5),
                    sigma_pb: 934.42,
                    sigma_err_pb: 0.87,
                    chi2_per_dof: 1.02,
                    sampler: Some(sampler(None)),
                },
            ],
            sigma_pb: 934.42,
            sigma_err_pb: 0.87,
            chi2_per_dof: 1.02,
        };

        let dir = scratch_dir("v5");
        let path = dir.join("v5.bin.zst");
        let raw = bincode::serialize(&legacy).unwrap();
        let compressed = zstd::encode_all(raw.as_slice(), ZSTD_LEVEL).unwrap();
        std::fs::write(&path, compressed).unwrap();

        let upgraded = IntegrateArtifact::read_from_path(&path).expect("version 5 reads");
        assert_eq!(upgraded.format_version, 5);
        let peripheral = upgraded.channels[0].sampler.as_ref().expect("sampler");
        assert_eq!(peripheral.topology, SamplerTopology::Spine);
        assert_eq!(peripheral.spine_poles_gev2, vec![0.4]);
        let timelike = upgraded.channels[1].sampler.as_ref().expect("sampler");
        assert_eq!(timelike.topology, SamplerTopology::Timelike);
        assert!(timelike.spine_poles_gev2.is_empty());
        assert_eq!(upgraded.sigma_pb.to_bits(), 934.42f64.to_bits());

        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    /// A version-6 file shares version 7's schema exactly, so it decodes with no
    /// upgrade function at all — and, unlike versions 3 through 5, it must keep its
    /// own recorded version rather than reading back as `FORMAT_VERSION`. That is
    /// the field `vibegraph generate`'s artifact-age guard reads to tell a
    /// pre-draw `sigma_pb` from a post-draw one.
    #[test]
    fn a_version_6_artifact_keeps_its_own_recorded_version() {
        let mut artifact = sample_artifact();
        artifact.format_version = 6;
        let dir = std::env::temp_dir().join(format!(
            "vibegraph-artifact-test-v6-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("v6.bin.zst");
        let _ = std::fs::remove_file(&path);

        artifact.write_to_path(&path, false).expect("write");
        let reloaded = IntegrateArtifact::read_from_path(&path).expect("version 6 reads");
        assert_eq!(reloaded.format_version, 6);
        assert_eq!(reloaded.sigma_pb.to_bits(), artifact.sigma_pb.to_bits());

        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
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
            channels: vec![v3_channel(1.0, VegasGrid::new(3, 64, 1.5))],
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
}
