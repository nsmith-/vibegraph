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
pub const FORMAT_VERSION: u32 = 3;

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
        "artifact format version {found}, but this build reads version {expected} \
         (regenerate it with `vibegraph integrate`)"
    )]
    UnsupportedVersion { found: u32, expected: u32 },
    #[error("zstd (de)compression failed: {0}")]
    Zstd(io::Error),
}

/// One phase-space channel's trained grid and its share of the integral.
///
/// The cross section is estimated as `Σⱼ ∫ dΦ f·αⱼgⱼ/g`, one VEGAS pass per
/// channel, so a channel's grid is meaningful only together with the `alpha` its
/// term is weighted by. A run with no multichannel decomposition banks a single
/// entry with `alpha = 1`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelGrid {
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
    /// The format version is read from the payload's prefix and checked before the
    /// body is decoded, so a file written by another schema version is refused by
    /// name rather than misread into whatever the current field order happens to
    /// accept.
    pub fn read_from_path(path: &Path) -> Result<Self, ArtifactError> {
        let compressed = std::fs::read(path).map_err(|source| ArtifactError::Io {
            path: path.display().to_string(),
            source,
        })?;
        let raw = zstd::decode_all(compressed.as_slice()).map_err(ArtifactError::Zstd)?;
        let header: VersionHeader = bincode::deserialize(&raw).map_err(ArtifactError::Decode)?;
        if header.format_version != FORMAT_VERSION {
            return Err(ArtifactError::UnsupportedVersion {
                found: header.format_version,
                expected: FORMAT_VERSION,
            });
        }
        bincode::deserialize(&raw).map_err(ArtifactError::Decode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ufo::sm::SMRestrict;
    use crate::vegas::VegasGrid;

    fn one_channel(grid: VegasGrid) -> ChannelGrid {
        ChannelGrid {
            alpha: 1.0,
            neval: 1000,
            grid,
            sigma_pb: 934.42,
            sigma_err_pb: 0.87,
            chi2_per_dof: 1.02,
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
                alpha: 0.1 * (j + 1) as f64,
                neval: 1000 * (j + 1),
                grid: VegasGrid::new(2 + j, 16, 0.5),
                sigma_pb: 1.5 * (j + 1) as f64,
                sigma_err_pb: 0.01 * (j + 1) as f64,
                chi2_per_dof: 0.9,
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
        }

        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    /// An artifact written under a different schema version is refused by version,
    /// not decoded into whatever the current field order accepts. The payload here
    /// is the previous schema — no `model`, so every field after `process` sits one
    /// slot early — which is exactly the shape a silent misread would consume: a
    /// positional decode would take the PDF set's bytes for a model name and carry
    /// on.
    #[test]
    fn read_refuses_a_foreign_format_version() {
        #[derive(Serialize)]
        struct LegacyArtifact {
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
            channels: Vec<ChannelGrid>,
            sigma_pb: f64,
            sigma_err_pb: f64,
            chi2_per_dof: f64,
        }

        let legacy = LegacyArtifact {
            format_version: FORMAT_VERSION - 1,
            process: "p p > e+ e-".to_string(),
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
        };

        let dir = std::env::temp_dir().join(format!(
            "vibegraph-artifact-test-version-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("legacy.bin.zst");
        let raw = bincode::serialize(&legacy).unwrap();
        let compressed = zstd::encode_all(raw.as_slice(), ZSTD_LEVEL).unwrap();
        std::fs::write(&path, compressed).unwrap();

        let err = IntegrateArtifact::read_from_path(&path)
            .expect_err("a foreign format version must not decode");
        assert!(
            matches!(
                err,
                ArtifactError::UnsupportedVersion {
                    found,
                    expected
                } if found == FORMAT_VERSION - 1 && expected == FORMAT_VERSION
            ),
            "expected a version refusal, got {err}"
        );

        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }
}
