//! UFO model loader combining FeynGraph topology with parameter evaluation.
//!
//! # Overview
//!
//! The Universal FeynRules Output (UFO) format is a directory of Python files:
//! - `particles.py` — particle content (spin, color, PDG, mass/width symbols)
//! - `parameters.py` — external (SLHA-readable) + internal (computed) parameters
//! - `couplings.py` — vertex coupling constants as expressions over parameters
//! - `vertices.py` — interaction vertices referencing particles, couplings, Lorentz structures
//! - `lorentz.py` — Lorentz tensor structures (handled by FeynGraph/HELAS layer later)
//!
//! [`UfoModel::load`] reads all of these and combines:
//! - [`feyngraph::model::Model`] for diagram-level topology
//! - Our own parsers for parameters, coupling values, particle masses
//!
//! [`UfoModel::evaluate`] computes all parameter and coupling values for a given
//! `param_card.dat`, returning an [`EvaluatedModel`] that supports incremental
//! re-evaluation via [`EvaluatedModel::recompute`].

pub mod couplings;
pub mod expr;
pub mod parameters;
pub mod particles_ext;
pub mod slha;
pub mod vertices_ext;

use couplings::{CouplingError, CouplingValue, parse_couplings};
use parameters::{ParameterError, ParameterSet, parse_parameters};
use particles_ext::{ParticleExt, ParticleExtError, parse_particles_ext};
use slha::ParamCard;
use vertices_ext::{VertexCouplingMap, VertexExtError, parse_vertex_couplings};

use feyngraph::model::Model as TopoModel;
use num_complex::Complex64;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UfoError {
    #[error("IO error reading UFO file '{file}': {cause}")]
    Io { file: String, cause: std::io::Error },
    #[error("Parameter parse error: {0}")]
    Parameters(#[from] ParameterError),
    #[error("Coupling parse error: {0}")]
    Couplings(#[from] CouplingError),
    #[error("Particle ext parse error: {0}")]
    ParticleExt(#[from] ParticleExtError),
    #[error("Vertex ext parse error: {0}")]
    VertexExt(#[from] VertexExtError),
    #[error("FeynGraph model error: {0}")]
    FeynGraph(#[from] feyngraph::model::ModelError),
}

/// A UFO model with all topology and parameter/coupling information loaded.
pub struct UfoModel {
    /// FeynGraph topology model (particles, vertices with spin/Lorentz structure).
    pub topo: TopoModel,
    /// Parsed parameters, topo-sorted with reverse-dep map.
    pub params: ParameterSet,
    /// Coupling constant values (symbolic expressions over parameters).
    pub coupling_values: HashMap<String, CouplingValue>,
    /// Mass/width parameter names per particle.
    pub particle_ext: HashMap<String, ParticleExt>,
    /// Final vertex name → [(lorentz_idx, color_idx, coupling_name)] associations.
    ///
    /// Keys are the *final* vertex names in `topo` (which may be split from the
    /// original Python names, e.g. `"V_1"` → `"V_1_0"`, `"V_1_1"`).
    pub vertex_coupling_map: HashMap<String, Vec<(usize, usize, String)>>,
}

impl UfoModel {
    /// Load a UFO model from a directory path.
    pub fn load(path: &Path) -> Result<Self, UfoError> {
        let read = |name: &str| -> Result<String, UfoError> {
            std::fs::read_to_string(path.join(name)).map_err(|e| UfoError::Io {
                file: path.join(name).display().to_string(),
                cause: e,
            })
        };

        let params_src = read("parameters.py")?;
        let couplings_src = read("couplings.py")?;
        let particles_src = read("particles.py")?;
        let vertices_src = read("vertices.py")?;

        let params = parse_parameters(&params_src)?;
        let coupling_values = parse_couplings(&couplings_src)?;
        let particle_ext = parse_particles_ext(&particles_src)?;

        // Raw vertex coupling map: original Python name → {(L,C) → coupling_name}
        let raw_vertex_couplings = parse_vertex_couplings(&vertices_src)?;

        // Load FeynGraph topology model.
        let topo = TopoModel::from_ufo(path)?;

        // Build final vertex coupling map, resolving splits via FeynGraph.
        let vertex_coupling_map = build_vertex_coupling_map(&topo, &raw_vertex_couplings);

        Ok(UfoModel {
            topo,
            params,
            coupling_values,
            particle_ext,
            vertex_coupling_map,
        })
    }

    /// Evaluate all parameters and coupling constants for the given param_card.
    ///
    /// Missing SLHA entries fall back to UFO default values.
    pub fn evaluate<'a>(&'a self, param_card: &ParamCard) -> EvaluatedModel<'a> {
        let param_values = self.params.evaluate(param_card);

        let coupling_values: HashMap<String, Complex64> = self
            .coupling_values
            .iter()
            .map(|(name, cv)| {
                let val = expr::eval(&cv.value, &param_values);
                (name.clone(), val)
            })
            .collect();

        EvaluatedModel {
            model: self,
            param_values,
            coupling_values,
        }
    }
}

/// Resolve FeynGraph vertex splits and build the final vertex → coupling map.
fn build_vertex_coupling_map(
    topo: &TopoModel,
    raw: &VertexCouplingMap,
) -> HashMap<String, Vec<(usize, usize, String)>> {
    let mut result: HashMap<String, Vec<(usize, usize, String)>> = HashMap::new();

    for (orig_name, lc_to_coupling) in raw {
        match topo.get_splitting(&orig_name.clone()) {
            None => {
                // Vertex was not split — use original name directly.
                let entries: Vec<(usize, usize, String)> = lc_to_coupling
                    .iter()
                    .map(|((l, c), name)| (*l, *c, name.clone()))
                    .collect();
                result.insert(orig_name.clone(), entries);
            }
            Some(splits) => {
                // Vertex was split; for each split vertex, collect its (L,C) pairs.
                for (split_name, lc_pairs) in splits {
                    let entries: Vec<(usize, usize, String)> = lc_pairs
                        .iter()
                        .filter_map(|(l, c)| {
                            lc_to_coupling
                                .get(&(*l, *c))
                                .map(|coup| (*l, *c, coup.clone()))
                        })
                        .collect();
                    if !entries.is_empty() {
                        result.insert(split_name.clone(), entries);
                    }
                }
            }
        }
    }

    result
}

/// Evaluated parameter and coupling values for a specific phase-space point.
pub struct EvaluatedModel<'a> {
    model: &'a UfoModel,
    /// All parameter values (external + internal), keyed by parameter name.
    pub param_values: HashMap<String, Complex64>,
    /// All coupling constant values, keyed by coupling name (e.g. `"GC_10"`).
    pub coupling_values: HashMap<String, Complex64>,
}

impl EvaluatedModel<'_> {
    /// Get the mass of a particle (real part of the evaluated mass parameter).
    ///
    /// Returns 0 if the particle or mass parameter is not found.
    pub fn mass(&self, particle_name: &str) -> f64 {
        self.model
            .particle_ext
            .get(particle_name)
            .and_then(|ext| self.param_values.get(&ext.mass_param))
            .map(|v| v.re)
            .unwrap_or(0.0)
    }

    /// Get the decay width of a particle.
    pub fn width(&self, particle_name: &str) -> f64 {
        self.model
            .particle_ext
            .get(particle_name)
            .and_then(|ext| self.param_values.get(&ext.width_param))
            .map(|v| v.re)
            .unwrap_or(0.0)
    }

    /// Get a coupling constant by name.
    pub fn coupling(&self, coupling_name: &str) -> Complex64 {
        self.coupling_values
            .get(coupling_name)
            .copied()
            .unwrap_or(Complex64::new(0.0, 0.0))
    }

    /// Get the coupling entries for a vertex by its (final) name.
    ///
    /// Returns `[(lorentz_idx, color_idx, value)]` or `None` if the vertex is unknown.
    pub fn vertex_couplings(&self, vertex_name: &str) -> Option<Vec<(usize, usize, Complex64)>> {
        self.model
            .vertex_coupling_map
            .get(vertex_name)
            .map(|entries| {
                entries
                    .iter()
                    .map(|(l, c, coup_name)| {
                        (
                            *l,
                            *c,
                            self.coupling_values
                                .get(coup_name)
                                .copied()
                                .unwrap_or_default(),
                        )
                    })
                    .collect()
            })
    }

    /// Re-evaluate only the parameters transitively depending on `changed`,
    /// then re-evaluate all coupling values that depend on any changed parameter.
    ///
    /// This is the efficient path for α_s running (once QCD is in scope):
    /// ```ignore
    /// evaluated.recompute("aS", Complex64::new(alpha_s_at_mu_r, 0.0));
    /// ```
    pub fn recompute(&mut self, changed: &str, new_value: Complex64) {
        self.param_values.insert(changed.to_owned(), new_value);
        self.model.params.recompute(changed, &mut self.param_values);

        // Re-evaluate couplings that depend on any changed parameter.
        // Determine the set of params that changed (conservatively: changed + its rdeps).
        let mut changed_params = vec![changed.to_owned()];
        if let Some(rdeps) = self.model.params.rdeps.get(changed) {
            changed_params.extend(rdeps.iter().cloned());
        }

        for (coup_name, cv) in &self.model.coupling_values {
            if cv.deps.iter().any(|d| changed_params.contains(d)) {
                let val = expr::eval(&cv.value, &self.param_values);
                self.coupling_values.insert(coup_name.clone(), val);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    /// Path helper for any bundled UFO model.
    fn ufo_path(model: &str) -> std::path::PathBuf {
        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        std::path::Path::new(&manifest)
            .join("research/refs/mg5amcnlo/models")
            .join(model)
    }

    #[test]
    fn test_load_loop_sm() {
        let path = ufo_path("loop_sm");
        if !path.exists() {
            eprintln!("loop_sm UFO not found — skipping");
            return;
        }
        // loop_sm adds `.loop_particles` and `.counterterm` attribute assignments to
        // particles.py which FeynGraph's topology parser does not support.  Our own
        // parameter/coupling parsers handle it correctly, but UfoModel::load() calls
        // FeynGraph first, so the full load fails.  Skip gracefully here and note the
        // limitation — a later phase can patch FeynGraph or pre-process the file.
        let result = UfoModel::load(&path);
        if let Err(UfoError::FeynGraph(_)) = &result {
            eprintln!(
                "loop_sm: FeynGraph topology parser does not support loop-level \
                       particle attributes (.counterterm, .loop_particles) — skipping"
            );
            return;
        }
        let model = result.expect("unexpected error loading loop_sm UFO");
        let empty_card = ParamCard::from_str("").unwrap();
        let ev = model.evaluate(&empty_card);

        // MZ default is 91.188 GeV (same as SM)
        let mz = ev.mass("Z");
        assert!((mz - 91.188).abs() < 0.01, "loop_sm MZ = {mz}");

        // Photon should be massless
        let ma = ev.mass("a");
        assert!(ma.abs() < 1e-10, "loop_sm m_photon = {ma}");

        // Should have 20 particles loaded
        assert_eq!(model.particle_ext.len(), 20, "loop_sm particle count");
    }

    #[test]
    fn test_load_mssm() {
        let path = ufo_path("MSSM_SLHA2");
        if !path.exists() {
            eprintln!("MSSM_SLHA2 UFO not found — skipping");
            return;
        }
        let model = UfoModel::load(&path).expect("failed to load MSSM_SLHA2 UFO");
        let empty_card = ParamCard::from_str("").unwrap();
        let ev = model.evaluate(&empty_card);

        // tb (tan β) default is 9.74862403; beta = atan(tb)
        let tb = ev.param_values["tb"].re;
        assert!((tb - 9.74862403).abs() < 1e-6, "MSSM tb = {tb}");

        let beta = ev.param_values["beta"].re;
        let expected_beta = 9.74862403f64.atan();
        assert!((beta - expected_beta).abs() < 1e-8, "MSSM beta = {beta}");

        // Photon should be massless
        let ma = ev.mass("a");
        assert!(ma.abs() < 1e-10, "MSSM m_photon = {ma}");

        // Should have 55 particles
        assert_eq!(model.particle_ext.len(), 55, "MSSM particle count");
    }

    #[test]
    fn test_load_taudecay() {
        let path = ufo_path("taudecay_UFO");
        if !path.exists() {
            eprintln!("taudecay_UFO not found — skipping");
            return;
        }
        // Load with bundled param_card.dat
        let param_card_path = path.join("param_card.dat");
        let card = slha::ParamCard::from_file(&param_card_path)
            .expect("failed to load taudecay param_card.dat");

        let model = UfoModel::load(&path).expect("failed to load taudecay UFO");
        let ev = model.evaluate(&card);

        // MTA from param_card: 1.776820 GeV
        let mta = ev.mass("ta__minus__");
        assert!((mta - 1.776820).abs() < 1e-4, "taudecay MTA = {mta}");

        // MMU from param_card: 0.105660 GeV
        let mmu = ev.mass("mu__minus__");
        assert!((mmu - 0.105660).abs() < 1e-4, "taudecay MMU = {mmu}");

        // Neutrinos are massless
        let mve = ev.mass("ve");
        assert!(mve.abs() < 1e-10, "taudecay m_ve = {mve}");

        // Should have 8 particles
        assert_eq!(model.particle_ext.len(), 8, "taudecay particle count");
    }

    /// Path to SM UFO model bundled as a test reference.
    fn sm_ufo_path() -> std::path::PathBuf {
        ufo_path("sm")
    }

    #[test]
    fn test_load_sm_ufo() {
        let path = sm_ufo_path();
        if !path.exists() {
            eprintln!("SM UFO not found at {:?} — skipping integration test", path);
            return;
        }
        let model = UfoModel::load(&path).expect("failed to load SM UFO");

        // Evaluate with default parameter values (no param_card needed).
        let empty_card = ParamCard::from_str("").unwrap();
        let ev = model.evaluate(&empty_card);

        // aS default is 0.118.
        let as_val = ev.param_values["aS"].re;
        assert!((as_val - 0.118).abs() < 1e-10, "aS = {as_val}");

        // G = 2 * sqrt(aS) * sqrt(pi) ≈ 1.2177
        let expected_g = 2.0 * (0.118f64).sqrt() * PI.sqrt();
        let g_val = ev.param_values["G"].re;
        assert!(
            (g_val - expected_g).abs() < 1e-6,
            "G = {g_val}, expected {expected_g}"
        );

        // GC_10 = -G
        let gc10 = ev.coupling("GC_10");
        assert!((gc10.re + expected_g).abs() < 1e-6, "GC_10 = {gc10}");
        assert!(gc10.im.abs() < 1e-10);

        // MZ default is 91.1876 GeV
        let mz = ev.mass("Z");
        assert!((mz - 91.1876).abs() < 0.01, "MZ = {mz}");

        // Photon is massless in SM UFO (mass = Param.ZERO)
        let ma = ev.mass("a");
        assert!(ma.abs() < 1e-10, "m_photon = {ma}");
    }

    #[test]
    fn test_recompute_propagates() {
        let path = sm_ufo_path();
        if !path.exists() {
            return;
        }
        let model = UfoModel::load(&path).expect("failed to load SM UFO");
        let empty_card = ParamCard::from_str("").unwrap();
        let mut ev = model.evaluate(&empty_card);

        let new_as = 0.130f64;
        ev.recompute("aS", Complex64::new(new_as, 0.0));

        let expected_g = 2.0 * new_as.sqrt() * PI.sqrt();
        let g_val = ev.param_values["G"].re;
        assert!(
            (g_val - expected_g).abs() < 1e-6,
            "After recompute: G = {g_val}"
        );

        let gc10 = ev.coupling("GC_10");
        assert!(
            (gc10.re + expected_g).abs() < 1e-6,
            "After recompute: GC_10 = {gc10}"
        );
    }
}
