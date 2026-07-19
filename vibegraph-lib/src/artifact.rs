//! On-disk artifact for a completed `vibegraph integrate` run: the adapted
//! VEGAS grid plus enough run metadata (seed, evaluation counts, process,
//! PDF set, the resolved run card) for a later phase to detect a mismatched
//! input rather than silently sampling against the wrong grid.
//!
//! Encoding is bincode, zstd-compressed — the same pairing already used for
//! the interned SM model blob ([`crate::ufo::sm`]).

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::runcard::RunCard;
use crate::vegas::VegasGrid;

/// Bumped whenever [`IntegrateArtifact`]'s shape changes in a way that would
/// break decoding an older file.
pub const FORMAT_VERSION: u32 = 1;

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
    #[error("zstd (de)compression failed: {0}")]
    Zstd(io::Error),
}

/// The result of one `vibegraph integrate` run: a trained [`VegasGrid`] plus
/// the inputs that produced it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrateArtifact {
    pub format_version: u32,
    /// The process string this artifact was integrated for (`"p p > e+ e-"`).
    pub process: String,
    pub pdf_set: String,
    pub pdf_member: u32,
    /// Factorization scale μF (GeV).
    pub mu_f: f64,
    /// Total hadronic collision energy √s (GeV).
    pub sqrt_s_had: f64,
    pub neval: usize,
    pub niter: usize,
    pub seed: u64,
    pub run_card: RunCard,
    pub grid: VegasGrid,
    pub sigma_pb: f64,
    pub sigma_err_pb: f64,
    pub chi2_per_dof: f64,
}

impl IntegrateArtifact {
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
    pub fn read_from_path(path: &Path) -> Result<Self, ArtifactError> {
        let compressed = std::fs::read(path).map_err(|source| ArtifactError::Io {
            path: path.display().to_string(),
            source,
        })?;
        let raw = zstd::decode_all(compressed.as_slice()).map_err(ArtifactError::Zstd)?;
        bincode::deserialize(&raw).map_err(ArtifactError::Decode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vegas::VegasGrid;

    fn sample_artifact() -> IntegrateArtifact {
        IntegrateArtifact {
            format_version: FORMAT_VERSION,
            process: "p p > e+ e-".to_string(),
            pdf_set: "NNPDF23_lo_as_0130_qed".to_string(),
            pdf_member: 0,
            mu_f: 91.1880,
            sqrt_s_had: 13000.0,
            neval: 1000,
            niter: 2,
            seed: 42,
            run_card: RunCard::default(),
            grid: VegasGrid::new(3, 64, 1.5),
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
        assert_eq!(reloaded.pdf_set, artifact.pdf_set);
        assert_eq!(reloaded.pdf_member, artifact.pdf_member);
        assert_eq!(reloaded.mu_f.to_bits(), artifact.mu_f.to_bits());
        assert_eq!(reloaded.sqrt_s_had.to_bits(), artifact.sqrt_s_had.to_bits());
        assert_eq!(reloaded.neval, artifact.neval);
        assert_eq!(reloaded.niter, artifact.niter);
        assert_eq!(reloaded.seed, artifact.seed);
        assert_eq!(reloaded.grid.ndim(), artifact.grid.ndim());
        assert_eq!(reloaded.grid.nbins(), artifact.grid.nbins());
        for (a, b) in reloaded.grid.xi().iter().zip(artifact.grid.xi().iter()) {
            for (x, y) in a.iter().zip(b.iter()) {
                assert_eq!(x.to_bits(), y.to_bits());
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
        artifact.grid = grid.clone();

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
        let after = sample(&reloaded.grid);
        assert_eq!(before.integral.to_bits(), after.integral.to_bits());
        assert_eq!(before.std_dev.to_bits(), after.std_dev.to_bits());

        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }
}
